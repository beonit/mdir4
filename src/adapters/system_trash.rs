use std::path::Path;

use crate::ports::trash::{Trash, TrashError};

pub struct SystemTrash;

impl Trash for SystemTrash {
    fn move_to_trash(&self, path: &Path) -> Result<(), TrashError> {
        trash::delete(path).map_err(|error| TrashError::Failed {
            path: path.to_path_buf(),
            message: error.to_string(),
        })
    }
}
