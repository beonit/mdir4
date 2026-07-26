use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    path::{Path, PathBuf},
    sync::Mutex,
};

use crate::{
    fs::{EntryAttributes, EntryKind, FileEntry},
    ports::filesystem::{EntryMetadata, FileSystem, FsError, FsOperation},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryFsCall {
    ReadDirectory(PathBuf),
    ReadMetadata(PathBuf),
    ReadFile(PathBuf),
    CreateDirectory(PathBuf),
    Rename(PathBuf, PathBuf),
    WriteFile(PathBuf),
    CopyFile(PathBuf, PathBuf),
    Remove(PathBuf),
}

#[derive(Debug, Default)]
pub struct MemoryFileSystemBuilder {
    nodes: BTreeMap<PathKey, MemoryNode>,
    denied: BTreeSet<(FsOperation, PathKey)>,
    cross_device_renames: BTreeSet<PathKey>,
}

impl MemoryFileSystemBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn directory(self, path: impl AsRef<Path>) -> Self {
        self.entry(path, EntryKind::Directory, 0)
    }

    pub fn file(self, path: impl AsRef<Path>, size: u64) -> Self {
        self.entry(path, EntryKind::File, size)
    }

    pub fn other(self, path: impl AsRef<Path>) -> Self {
        self.entry(path, EntryKind::Other, 0)
    }

    pub fn deny(mut self, operation: FsOperation, path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        self.denied.insert((operation, fixture_key(path)));
        self
    }

    pub fn cross_device_rename(mut self, path: impl AsRef<Path>) -> Self {
        self.cross_device_renames.insert(fixture_key(path.as_ref()));
        self
    }

    pub fn build(self) -> MemoryFileSystem {
        MemoryFileSystem {
            nodes: Mutex::new(self.nodes),
            denied: self.denied,
            cross_device_renames: self.cross_device_renames,
            calls: Mutex::new(Vec::new()),
        }
    }

    fn entry(mut self, path: impl AsRef<Path>, kind: EntryKind, size: u64) -> Self {
        let path = path.as_ref().to_path_buf();
        let key = fixture_key(&path);
        let name = display_name(&path);
        self.nodes.insert(
            key,
            MemoryNode {
                path,
                name,
                metadata: EntryMetadata {
                    kind,
                    size,
                    modified: None,
                    attributes: EntryAttributes::default(),
                },
                contents: vec![0; size as usize],
            },
        );
        self
    }
}

#[derive(Debug)]
pub struct MemoryFileSystem {
    nodes: Mutex<BTreeMap<PathKey, MemoryNode>>,
    denied: BTreeSet<(FsOperation, PathKey)>,
    cross_device_renames: BTreeSet<PathKey>,
    calls: Mutex<Vec<MemoryFsCall>>,
}

impl MemoryFileSystem {
    pub fn calls(&self) -> Vec<MemoryFsCall> {
        lock(&self.calls).clone()
    }

    pub fn clear_calls(&self) {
        lock(&self.calls).clear();
    }

    fn key_for_operation(&self, path: &Path, operation: FsOperation) -> Result<PathKey, FsError> {
        PathKey::parse(path).ok_or_else(|| FsError::InvalidPath {
            operation,
            path: path.to_path_buf(),
        })
    }

    fn check_permission(
        &self,
        key: &PathKey,
        path: &Path,
        operation: FsOperation,
    ) -> Result<(), FsError> {
        if self.denied.contains(&(operation, key.clone())) {
            Err(FsError::PermissionDenied {
                operation,
                path: path.to_path_buf(),
            })
        } else {
            Ok(())
        }
    }

    fn insert_node(
        &self,
        path: &Path,
        kind: EntryKind,
        contents: Vec<u8>,
        operation: FsOperation,
    ) -> Result<(), FsError> {
        let key = self.key_for_operation(path, operation)?;
        self.check_permission(&key, path, operation)?;
        let mut nodes = lock(&self.nodes);
        if nodes.contains_key(&key) {
            return Err(FsError::AlreadyExists {
                operation,
                path: path.to_path_buf(),
            });
        }
        if let Some(parent) = key.parent()
            && !nodes
                .get(&parent)
                .is_some_and(|node| node.metadata.kind == EntryKind::Directory)
        {
            return Err(FsError::NotFound {
                operation,
                path: path.to_path_buf(),
            });
        }
        let size = contents.len() as u64;
        nodes.insert(
            key,
            MemoryNode {
                path: path.to_path_buf(),
                name: display_name(path),
                metadata: EntryMetadata {
                    kind,
                    size,
                    modified: None,
                    attributes: EntryAttributes::default(),
                },
                contents,
            },
        );
        Ok(())
    }
}

impl FileSystem for MemoryFileSystem {
    fn read_dir(&self, path: &Path) -> Result<Vec<FileEntry>, FsError> {
        lock(&self.calls).push(MemoryFsCall::ReadDirectory(path.to_path_buf()));
        let operation = FsOperation::ReadDirectory;
        let key = self.key_for_operation(path, operation)?;
        self.check_permission(&key, path, operation)?;

        let nodes = lock(&self.nodes);
        let node = nodes.get(&key).ok_or_else(|| FsError::NotFound {
            operation,
            path: path.to_path_buf(),
        })?;
        if node.metadata.kind != EntryKind::Directory {
            return Err(FsError::NotDirectory {
                path: path.to_path_buf(),
            });
        }

        Ok(nodes
            .iter()
            .filter(|(candidate, _)| candidate.parent().as_ref() == Some(&key))
            .map(|(_, child)| FileEntry {
                path: child.path.clone(),
                name: child.name.clone(),
                kind: child.metadata.kind,
                size: child.metadata.size,
                modified: child.metadata.modified,
                local_modified: None,
                attributes: child.metadata.attributes,
            })
            .collect())
    }

    fn metadata(&self, path: &Path) -> Result<EntryMetadata, FsError> {
        lock(&self.calls).push(MemoryFsCall::ReadMetadata(path.to_path_buf()));
        let operation = FsOperation::ReadMetadata;
        let key = self.key_for_operation(path, operation)?;
        self.check_permission(&key, path, operation)?;

        lock(&self.nodes)
            .get(&key)
            .map(|node| node.metadata)
            .ok_or_else(|| FsError::NotFound {
                operation,
                path: path.to_path_buf(),
            })
    }

    fn read_file(&self, path: &Path, max_bytes: usize) -> Result<Vec<u8>, FsError> {
        lock(&self.calls).push(MemoryFsCall::ReadFile(path.to_path_buf()));
        let operation = FsOperation::ReadFile;
        let key = self.key_for_operation(path, operation)?;
        self.check_permission(&key, path, operation)?;
        let nodes = lock(&self.nodes);
        let node = nodes.get(&key).ok_or_else(|| FsError::NotFound {
            operation,
            path: path.to_path_buf(),
        })?;
        if node.contents.len() > max_bytes {
            return Err(FsError::TooLarge {
                path: path.to_path_buf(),
            });
        }
        Ok(node.contents.clone())
    }

    fn create_dir(&self, path: &Path) -> Result<(), FsError> {
        lock(&self.calls).push(MemoryFsCall::CreateDirectory(path.to_path_buf()));
        self.insert_node(
            path,
            EntryKind::Directory,
            Vec::new(),
            FsOperation::CreateDirectory,
        )
    }

    fn rename(&self, from: &Path, to: &Path) -> Result<(), FsError> {
        lock(&self.calls).push(MemoryFsCall::Rename(from.to_path_buf(), to.to_path_buf()));
        let operation = FsOperation::Rename;
        let from_key = self.key_for_operation(from, operation)?;
        let to_key = self.key_for_operation(to, operation)?;
        self.check_permission(&from_key, from, operation)?;
        if self.cross_device_renames.contains(&from_key) {
            return Err(FsError::CrossDevice {
                path: from.to_path_buf(),
            });
        }
        let mut nodes = lock(&self.nodes);
        if nodes.contains_key(&to_key) {
            return Err(FsError::AlreadyExists {
                operation,
                path: to.to_path_buf(),
            });
        }
        let mut node = nodes.remove(&from_key).ok_or_else(|| FsError::NotFound {
            operation,
            path: from.to_path_buf(),
        })?;
        node.path = to.to_path_buf();
        node.name = display_name(to);
        nodes.insert(to_key, node);
        let descendants: Vec<_> = nodes
            .keys()
            .filter_map(|candidate| {
                rebase_key(
                    candidate,
                    &from_key,
                    &self.key_for_operation(to, operation).ok()?,
                )
                .map(|rebased| (candidate.clone(), rebased))
            })
            .collect();
        for (old, new) in descendants {
            if let Some(mut child) = nodes.remove(&old) {
                child.path = path_from_key(&new);
                nodes.insert(new, child);
            }
        }
        Ok(())
    }

    fn write_file_atomic(&self, path: &Path, contents: &[u8]) -> Result<(), FsError> {
        lock(&self.calls).push(MemoryFsCall::WriteFile(path.to_path_buf()));
        let operation = FsOperation::WriteFile;
        let key = self.key_for_operation(path, operation)?;
        self.check_permission(&key, path, operation)?;
        let mut nodes = lock(&self.nodes);
        if let Some(node) = nodes.get_mut(&key) {
            if node.metadata.kind != EntryKind::File {
                return Err(FsError::InvalidPath {
                    operation,
                    path: path.to_path_buf(),
                });
            }
            node.contents = contents.to_vec();
            node.metadata.size = contents.len() as u64;
            return Ok(());
        }
        drop(nodes);
        self.insert_node(path, EntryKind::File, contents.to_vec(), operation)
    }

    fn copy_file(&self, source: &Path, target: &Path) -> Result<u64, FsError> {
        lock(&self.calls).push(MemoryFsCall::CopyFile(
            source.to_path_buf(),
            target.to_path_buf(),
        ));
        let bytes = self.read_file(source, usize::MAX)?;
        let size = bytes.len() as u64;
        self.insert_node(target, EntryKind::File, bytes, FsOperation::CopyFile)?;
        Ok(size)
    }

    fn remove(&self, path: &Path, recursive: bool) -> Result<(), FsError> {
        lock(&self.calls).push(MemoryFsCall::Remove(path.to_path_buf()));
        let operation = FsOperation::Remove;
        let key = self.key_for_operation(path, operation)?;
        self.check_permission(&key, path, operation)?;
        let mut nodes = lock(&self.nodes);
        if !nodes.contains_key(&key) {
            return Err(FsError::NotFound {
                operation,
                path: path.to_path_buf(),
            });
        }
        let has_children = nodes
            .keys()
            .any(|candidate| candidate.parent().as_ref() == Some(&key));
        if has_children && !recursive {
            return Err(FsError::Io {
                operation,
                path: path.to_path_buf(),
                kind: std::io::ErrorKind::DirectoryNotEmpty,
            });
        }
        let descendants: Vec<_> = nodes
            .keys()
            .filter(|candidate| is_descendant(candidate, &key))
            .cloned()
            .collect();
        for descendant in descendants {
            nodes.remove(&descendant);
        }
        nodes.remove(&key);
        Ok(())
    }
}

#[derive(Debug)]
struct MemoryNode {
    path: PathBuf,
    name: OsString,
    metadata: EntryMetadata,
    contents: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum PathKey {
    Native {
        absolute: bool,
        components: Vec<OsString>,
    },
    Drive {
        drive: char,
        components: Vec<String>,
    },
    Unc {
        server: String,
        share: String,
        components: Vec<String>,
    },
}

fn is_descendant(candidate: &PathKey, ancestor: &PathKey) -> bool {
    let mut current = candidate.parent();
    while let Some(parent) = current {
        if &parent == ancestor {
            return true;
        }
        current = parent.parent();
    }
    false
}

fn rebase_key(candidate: &PathKey, from: &PathKey, to: &PathKey) -> Option<PathKey> {
    match (candidate, from, to) {
        (
            PathKey::Native {
                absolute,
                components,
            },
            PathKey::Native {
                absolute: from_absolute,
                components: from_components,
            },
            PathKey::Native {
                absolute: to_absolute,
                components: to_components,
            },
        ) if absolute == from_absolute && components.starts_with(from_components) => {
            let mut rebased = to_components.clone();
            rebased.extend_from_slice(&components[from_components.len()..]);
            Some(PathKey::Native {
                absolute: *to_absolute,
                components: rebased,
            })
        }
        _ => None,
    }
}

fn path_from_key(key: &PathKey) -> PathBuf {
    match key {
        PathKey::Native {
            absolute,
            components,
        } => {
            let mut path = if *absolute {
                PathBuf::from(std::path::MAIN_SEPARATOR.to_string())
            } else {
                PathBuf::new()
            };
            for component in components {
                path.push(component);
            }
            path
        }
        PathKey::Drive { drive, components } => {
            PathBuf::from(format!("{}:/{}", drive, components.join("/")))
        }
        PathKey::Unc {
            server,
            share,
            components,
        } => PathBuf::from(format!("//{server}/{share}/{}", components.join("/"))),
    }
}

impl PathKey {
    fn parse(path: &Path) -> Option<Self> {
        if path.as_os_str().is_empty() {
            return None;
        }
        if let Some(text) = path.to_str()
            && let Some(windows) = parse_windows_path(text)
        {
            return windows;
        }

        let absolute = path.is_absolute();
        let mut components = Vec::new();
        for component in path.components() {
            use std::path::Component;
            match component {
                Component::Prefix(_) | Component::RootDir | Component::CurDir => {}
                Component::ParentDir => {
                    if components.last().is_some_and(|part| part != "..") {
                        components.pop();
                    } else if !absolute {
                        components.push(OsString::from(".."));
                    }
                }
                Component::Normal(part) => components.push(part.to_os_string()),
            }
        }
        Some(Self::Native {
            absolute,
            components,
        })
    }

    fn parent(&self) -> Option<Self> {
        match self {
            Self::Native {
                absolute,
                components,
            } => parent_components(components).map(|components| Self::Native {
                absolute: *absolute,
                components,
            }),
            Self::Drive { drive, components } => {
                parent_components(components).map(|components| Self::Drive {
                    drive: *drive,
                    components,
                })
            }
            Self::Unc {
                server,
                share,
                components,
            } => parent_components(components).map(|components| Self::Unc {
                server: server.clone(),
                share: share.clone(),
                components,
            }),
        }
    }
}

fn parse_windows_path(path: &str) -> Option<Option<PathKey>> {
    let path = path.replace('\\', "/");
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        if bytes.len() > 2 && bytes[2] != b'/' {
            return Some(None);
        }
        let drive = (bytes[0] as char).to_ascii_lowercase();
        let remainder = path.get(2..).unwrap_or_default().trim_start_matches('/');
        return Some(Some(PathKey::Drive {
            drive,
            components: normalized_windows_components(remainder),
        }));
    }

    if path.starts_with("//") {
        let mut parts = path.trim_start_matches('/').split('/');
        let server = parts.next().filter(|part| !part.is_empty())?;
        let share = parts.next().filter(|part| !part.is_empty())?;
        return Some(Some(PathKey::Unc {
            server: server.to_lowercase(),
            share: share.to_lowercase(),
            components: normalize_component_iter(parts),
        }));
    }

    None
}

fn normalized_windows_components(path: &str) -> Vec<String> {
    normalize_component_iter(path.split('/'))
}

fn normalize_component_iter<'a>(parts: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut result = Vec::new();
    for part in parts {
        match part {
            "" | "." => {}
            ".." => {
                result.pop();
            }
            _ => result.push(part.to_lowercase()),
        }
    }
    result
}

fn parent_components<T: Clone>(components: &[T]) -> Option<Vec<T>> {
    if components.is_empty() {
        None
    } else {
        Some(components[..components.len() - 1].to_vec())
    }
}

fn display_name(path: &Path) -> OsString {
    if let Some(text) = path.to_str()
        && parse_windows_path(text).is_some()
    {
        return text
            .replace('\\', "/")
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .map(OsString::from)
            .unwrap_or_default();
    }
    path.file_name().map(OsString::from).unwrap_or_default()
}

fn fixture_key(path: &Path) -> PathKey {
    PathKey::parse(path).unwrap_or_else(|| {
        panic!(
            "MemoryFileSystem fixture path must not be empty or malformed: {}",
            path.display()
        )
    })
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
