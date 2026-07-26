use std::path::{Path, PathBuf};

use thiserror::Error;

pub trait Trash: Send + Sync {
    fn move_to_trash(&self, path: &Path) -> Result<(), TrashError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TrashError {
    #[error("cannot move path to Trash: {path}: {message}", path = .path.display())]
    Failed { path: PathBuf, message: String },
}
