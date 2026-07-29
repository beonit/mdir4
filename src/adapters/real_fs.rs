use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    fs::{EntryAttributes, EntryKind, FileEntry},
    ports::filesystem::{EntryMetadata, FileSystem, FsError, FsOperation},
};

#[derive(Debug, Clone, Copy, Default)]
pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn read_dir(&self, path: &Path) -> Result<Vec<FileEntry>, FsError> {
        let operation = FsOperation::ReadDirectory;
        let directory = fs::read_dir(path).map_err(|error| map_error(error, operation, path))?;
        let mut entries = Vec::new();

        for result in directory {
            let entry = result.map_err(|error| map_error(error, operation, path))?;
            let entry_path = entry.path();
            let metadata = resolve_entry_metadata(
                fs::symlink_metadata(&entry_path).map(|metadata| metadata_from_std(&metadata)),
                || {
                    entry
                        .file_type()
                        .map(|file_type| kind_from_file_type(&file_type))
                },
            );
            entries.push(FileEntry {
                path: entry_path,
                name: entry.file_name(),
                kind: metadata.kind,
                size: metadata.size,
                modified: metadata.modified,
                local_modified: None,
                attributes: metadata.attributes,
            });
        }

        Ok(entries)
    }

    fn metadata(&self, path: &Path) -> Result<EntryMetadata, FsError> {
        fs::metadata(path)
            .map(|metadata| metadata_from_std(&metadata))
            .map_err(|error| map_error(error, FsOperation::ReadMetadata, path))
    }

    fn symlink_metadata(&self, path: &Path) -> Result<EntryMetadata, FsError> {
        fs::symlink_metadata(path)
            .map(|metadata| metadata_from_std(&metadata))
            .map_err(|error| map_error(error, FsOperation::ReadMetadata, path))
    }

    fn read_file(&self, path: &Path, max_bytes: usize) -> Result<Vec<u8>, FsError> {
        let operation = FsOperation::ReadFile;
        let metadata = fs::metadata(path).map_err(|error| map_error(error, operation, path))?;
        if metadata.len() > max_bytes as u64 {
            return Err(FsError::TooLarge {
                path: path.to_path_buf(),
            });
        }
        let mut file = File::open(path).map_err(|error| map_error(error, operation, path))?;
        let mut contents = Vec::with_capacity(metadata.len() as usize);
        std::io::Read::by_ref(&mut file)
            .take(max_bytes as u64 + 1)
            .read_to_end(&mut contents)
            .map_err(|error| map_error(error, operation, path))?;
        if contents.len() > max_bytes {
            return Err(FsError::TooLarge {
                path: path.to_path_buf(),
            });
        }
        Ok(contents)
    }

    fn create_dir(&self, path: &Path) -> Result<(), FsError> {
        fs::create_dir(path).map_err(|error| map_error(error, FsOperation::CreateDirectory, path))
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<(), FsError> {
        if to.exists() && !same_lexical_path(from, to) {
            return Err(FsError::AlreadyExists {
                operation: FsOperation::Rename,
                path: to.to_path_buf(),
            });
        }
        fs::rename(from, to).map_err(|error| map_error(error, FsOperation::Rename, from))
    }

    fn write_file_atomic(&self, path: &Path, contents: &[u8]) -> Result<(), FsError> {
        let operation = FsOperation::WriteFile;
        let parent = path.parent().ok_or_else(|| FsError::InvalidPath {
            operation,
            path: path.to_path_buf(),
        })?;
        let (temporary, mut file) = create_temp(parent, path)?;
        let result = (|| {
            file.write_all(contents)
                .and_then(|()| file.flush())
                .and_then(|()| file.sync_all())
                .map_err(|error| map_error(error, operation, path))?;
            drop(file);
            if !path.exists() {
                return fs::rename(&temporary, path)
                    .map_err(|error| map_error(error, operation, path));
            }
            let backup = unique_sibling(path, "backup")?;
            fs::rename(path, &backup).map_err(|error| map_error(error, operation, path))?;
            match fs::rename(&temporary, path) {
                Ok(()) => {
                    let _ = fs::remove_file(backup);
                    Ok(())
                }
                Err(error) => {
                    let _ = fs::rename(&backup, path);
                    Err(map_error(error, operation, path))
                }
            }
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }

    fn copy_file(&self, source: &Path, target: &Path) -> Result<u64, FsError> {
        let operation = FsOperation::CopyFile;
        let parent = target.parent().ok_or_else(|| FsError::InvalidPath {
            operation,
            path: target.to_path_buf(),
        })?;
        let (temporary, mut output) = create_temp(parent, target)?;
        let result = (|| {
            let mut input =
                File::open(source).map_err(|error| map_error(error, operation, source))?;
            let bytes = io::copy(&mut input, &mut output)
                .map_err(|error| map_error(error, operation, source))?;
            output
                .flush()
                .map_err(|error| map_error(error, operation, target))?;
            output
                .sync_all()
                .map_err(|error| map_error(error, operation, target))?;
            drop(output);
            if target.exists() {
                return Err(FsError::AlreadyExists {
                    operation,
                    path: target.to_path_buf(),
                });
            }
            fs::rename(&temporary, target).map_err(|error| map_error(error, operation, target))?;
            if let Ok(metadata) = fs::metadata(source) {
                let _ = fs::set_permissions(target, metadata.permissions());
            }
            Ok(bytes)
        })();
        if result.is_err() {
            let _ = fs::remove_file(temporary);
        }
        result
    }

    fn remove(&self, path: &Path, recursive: bool) -> Result<(), FsError> {
        let metadata = fs::symlink_metadata(path)
            .map_err(|error| map_error(error, FsOperation::Remove, path))?;
        if metadata.file_type().is_symlink() || metadata.is_file() {
            fs::remove_file(path).map_err(|error| map_error(error, FsOperation::Remove, path))
        } else if recursive {
            fs::remove_dir_all(path).map_err(|error| map_error(error, FsOperation::Remove, path))
        } else {
            fs::remove_dir(path).map_err(|error| map_error(error, FsOperation::Remove, path))
        }
    }
}

fn create_temp(parent: &Path, target: &Path) -> Result<(PathBuf, File), FsError> {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let operation = FsOperation::WriteFile;
    for _ in 0..32 {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let name = target
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("file");
        let temporary = parent.join(format!(".{name}.mdir4-{id}.tmp"));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => return Ok((temporary, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(map_error(error, operation, target)),
        }
    }
    Err(FsError::AlreadyExists {
        operation,
        path: target.to_path_buf(),
    })
}

fn unique_sibling(target: &Path, suffix: &str) -> Result<PathBuf, FsError> {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let parent = target.parent().ok_or_else(|| FsError::InvalidPath {
        operation: FsOperation::WriteFile,
        path: target.to_path_buf(),
    })?;
    let name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("file");
    for _ in 0..32 {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(".{name}.mdir4-{suffix}-{id}"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(FsError::AlreadyExists {
        operation: FsOperation::WriteFile,
        path: target.to_path_buf(),
    })
}

fn same_lexical_path(left: &Path, right: &Path) -> bool {
    left.as_os_str() == right.as_os_str()
}

fn resolve_entry_metadata(
    metadata: io::Result<EntryMetadata>,
    fallback_kind: impl FnOnce() -> io::Result<EntryKind>,
) -> EntryMetadata {
    metadata.unwrap_or_else(|_| EntryMetadata {
        kind: fallback_kind().unwrap_or(EntryKind::Other),
        size: 0,
        modified: None,
        attributes: EntryAttributes::default(),
    })
}

fn metadata_from_std(metadata: &fs::Metadata) -> EntryMetadata {
    let kind = if metadata.is_dir() {
        EntryKind::Directory
    } else if metadata.is_file() {
        EntryKind::File
    } else {
        EntryKind::Other
    };
    EntryMetadata {
        kind,
        size: if kind == EntryKind::File {
            metadata.len()
        } else {
            0
        },
        modified: metadata.modified().ok(),
        attributes: attributes_from_std(metadata),
    }
}

fn attributes_from_std(metadata: &fs::Metadata) -> EntryAttributes {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        let value = metadata.file_attributes();
        return EntryAttributes {
            read_only: metadata.permissions().readonly(),
            hidden: value & 0x2 != 0,
            system: value & 0x4 != 0,
            archive: value & 0x20 != 0,
            executable: false,
            unix_mode: None,
        };
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        EntryAttributes {
            read_only: metadata.permissions().readonly(),
            executable: metadata.permissions().mode() & 0o111 != 0,
            unix_mode: Some(metadata.permissions().mode()),
            ..EntryAttributes::default()
        }
    }
    #[cfg(all(not(unix), not(windows)))]
    {
        EntryAttributes {
            read_only: metadata.permissions().readonly(),
            ..EntryAttributes::default()
        }
    }
}

fn kind_from_file_type(file_type: &fs::FileType) -> EntryKind {
    if file_type.is_dir() {
        EntryKind::Directory
    } else if file_type.is_file() {
        EntryKind::File
    } else {
        EntryKind::Other
    }
}

fn map_error(error: io::Error, operation: FsOperation, path: &Path) -> FsError {
    match error.kind() {
        io::ErrorKind::NotFound => FsError::NotFound {
            operation,
            path: path.to_path_buf(),
        },
        io::ErrorKind::PermissionDenied => FsError::PermissionDenied {
            operation,
            path: path.to_path_buf(),
        },
        io::ErrorKind::NotADirectory => FsError::NotDirectory {
            path: path.to_path_buf(),
        },
        io::ErrorKind::AlreadyExists => FsError::AlreadyExists {
            operation,
            path: path.to_path_buf(),
        },
        io::ErrorKind::CrossesDevices => FsError::CrossDevice {
            path: path.to_path_buf(),
        },
        kind => FsError::Io {
            operation,
            path: path.to_path_buf(),
            kind,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn successful_metadata_does_not_request_fallback_file_type() {
        let fallback_called = Cell::new(false);

        let metadata = resolve_entry_metadata(
            Ok(EntryMetadata {
                kind: EntryKind::File,
                size: 12,
                modified: None,
                attributes: EntryAttributes::default(),
            }),
            || {
                fallback_called.set(true);
                Ok(EntryKind::Other)
            },
        );

        assert_eq!(metadata.size, 12);
        assert!(!fallback_called.get());
    }

    #[test]
    fn individual_metadata_failure_uses_file_type_without_failing_listing() {
        let metadata = resolve_entry_metadata(
            Err(io::Error::new(io::ErrorKind::PermissionDenied, "denied")),
            || Ok(EntryKind::Directory),
        );

        assert_eq!(
            metadata,
            EntryMetadata {
                kind: EntryKind::Directory,
                size: 0,
                modified: None,
                attributes: EntryAttributes::default(),
            }
        );
    }

    #[test]
    fn unknown_entry_is_preserved_when_all_metadata_calls_fail() {
        let metadata = resolve_entry_metadata(
            Err(io::Error::new(io::ErrorKind::PermissionDenied, "denied")),
            || Err(io::Error::other("unavailable")),
        );

        assert_eq!(metadata.kind, EntryKind::Other);
        assert_eq!(metadata.size, 0);
    }
}
