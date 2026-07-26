use std::{
    io,
    path::{Path, PathBuf},
    time::SystemTime,
};

use thiserror::Error;

use crate::fs::{EntryAttributes, EntryKind, FileEntry};

pub trait FileSystem: Send + Sync {
    fn read_dir(&self, path: &Path) -> Result<Vec<FileEntry>, FsError>;

    fn metadata(&self, path: &Path) -> Result<EntryMetadata, FsError>;

    fn symlink_metadata(&self, path: &Path) -> Result<EntryMetadata, FsError> {
        self.metadata(path)
    }

    fn read_file(&self, path: &Path, _max_bytes: usize) -> Result<Vec<u8>, FsError> {
        Err(FsError::Unsupported {
            operation: FsOperation::ReadFile,
            path: path.to_path_buf(),
        })
    }

    fn create_dir(&self, path: &Path) -> Result<(), FsError> {
        Err(FsError::Unsupported {
            operation: FsOperation::CreateDirectory,
            path: path.to_path_buf(),
        })
    }

    fn rename(&self, from: &Path, _to: &Path) -> Result<(), FsError> {
        Err(FsError::Unsupported {
            operation: FsOperation::Rename,
            path: from.to_path_buf(),
        })
    }

    fn write_file_atomic(&self, path: &Path, _contents: &[u8]) -> Result<(), FsError> {
        Err(FsError::Unsupported {
            operation: FsOperation::WriteFile,
            path: path.to_path_buf(),
        })
    }

    fn copy_file(&self, source: &Path, _target: &Path) -> Result<u64, FsError> {
        Err(FsError::Unsupported {
            operation: FsOperation::CopyFile,
            path: source.to_path_buf(),
        })
    }

    fn remove(&self, path: &Path, _recursive: bool) -> Result<(), FsError> {
        Err(FsError::Unsupported {
            operation: FsOperation::Remove,
            path: path.to_path_buf(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FsOperation {
    ReadDirectory,
    ReadMetadata,
    ReadFile,
    CreateDirectory,
    Rename,
    WriteFile,
    CopyFile,
    Remove,
}

impl std::fmt::Display for FsOperation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadDirectory => formatter.write_str("read directory"),
            Self::ReadMetadata => formatter.write_str("read metadata"),
            Self::ReadFile => formatter.write_str("read file"),
            Self::CreateDirectory => formatter.write_str("create directory"),
            Self::Rename => formatter.write_str("rename"),
            Self::WriteFile => formatter.write_str("write file"),
            Self::CopyFile => formatter.write_str("copy file"),
            Self::Remove => formatter.write_str("remove"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FsError {
    #[error("{operation} failed because the path was not found: {path}", path = .path.display())]
    NotFound {
        operation: FsOperation,
        path: PathBuf,
    },
    #[error("path is not a directory: {path}", path = .path.display())]
    NotDirectory { path: PathBuf },
    #[error("permission denied while attempting to {operation}: {path}", path = .path.display())]
    PermissionDenied {
        operation: FsOperation,
        path: PathBuf,
    },
    #[error("invalid path for {operation}: {path}", path = .path.display())]
    InvalidPath {
        operation: FsOperation,
        path: PathBuf,
    },
    #[error("{operation} failed for {path}: {kind:?}", path = .path.display())]
    Io {
        operation: FsOperation,
        path: PathBuf,
        kind: io::ErrorKind,
    },
    #[error("target already exists while attempting to {operation}: {path}", path = .path.display())]
    AlreadyExists {
        operation: FsOperation,
        path: PathBuf,
    },
    #[error("file is too large: {path}", path = .path.display())]
    TooLarge { path: PathBuf },
    #[error("operation is not supported: {operation} for {path}", path = .path.display())]
    Unsupported {
        operation: FsOperation,
        path: PathBuf,
    },
    #[error("rename crosses filesystem boundaries: {path}", path = .path.display())]
    CrossDevice { path: PathBuf },
    #[error("operation was cancelled: {path}", path = .path.display())]
    Cancelled { path: PathBuf },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntryMetadata {
    pub kind: EntryKind,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub attributes: EntryAttributes,
}
