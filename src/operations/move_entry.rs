use std::path::Path;

use crate::{
    model::operation::OperationSummary,
    ports::filesystem::{FileSystem, FsError},
};

use super::{copy::copy_entry, planner::validate_transfer};

pub fn move_entry(
    filesystem: &(impl FileSystem + ?Sized),
    source: &Path,
    target: &Path,
) -> Result<OperationSummary, FsError> {
    validate_transfer(source, target).map_err(|_| FsError::InvalidPath {
        operation: crate::ports::filesystem::FsOperation::Rename,
        path: target.to_path_buf(),
    })?;
    match filesystem.rename(source, target) {
        Ok(()) => Ok(OperationSummary {
            succeeded: 1,
            ..OperationSummary::default()
        }),
        Err(FsError::CrossDevice { .. }) => {
            let mut summary = copy_entry(filesystem, source, target)?;
            match filesystem.remove(source, true) {
                Ok(()) => Ok(summary),
                Err(error) => {
                    summary.failed += 1;
                    summary.first_error = Some(format!("Copied, but source remains: {error}"));
                    Ok(summary)
                }
            }
        }
        Err(error) => Err(error),
    }
}
