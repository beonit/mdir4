use std::path::Path;

use crate::ports::filesystem::{FileSystem, FsError, FsOperation};

pub fn permanent_delete(
    filesystem: &(impl FileSystem + ?Sized),
    current_directory: &Path,
    target: &Path,
) -> Result<(), FsError> {
    if target == current_directory || target.parent().is_none() || target.file_name().is_none() {
        return Err(FsError::InvalidPath {
            operation: FsOperation::Remove,
            path: target.to_path_buf(),
        });
    }
    filesystem.remove(target, true)
}
