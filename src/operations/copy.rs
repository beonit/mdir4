use std::path::Path;

use crate::{
    fs::EntryKind,
    model::operation::{ConflictDecision, ConflictPolicy, OperationId, OperationSummary},
    ports::filesystem::{FileSystem, FsError},
    runtime::job::CancelToken,
};

use super::planner::validate_transfer;

pub fn copy_entry(
    filesystem: &(impl FileSystem + ?Sized),
    source: &Path,
    target: &Path,
) -> Result<OperationSummary, FsError> {
    validate_transfer(source, target).map_err(|_| FsError::InvalidPath {
        operation: crate::ports::filesystem::FsOperation::CopyFile,
        path: target.to_path_buf(),
    })?;
    let mut summary = OperationSummary::default();
    copy_recursive(filesystem, source, target, &mut summary)?;
    Ok(summary)
}

pub fn copy_entry_cancellable(
    filesystem: &(impl FileSystem + ?Sized),
    source: &Path,
    target: &Path,
    cancel: &CancelToken,
) -> Result<OperationSummary, FsError> {
    validate_transfer(source, target).map_err(|_| FsError::InvalidPath {
        operation: crate::ports::filesystem::FsOperation::CopyFile,
        path: target.to_path_buf(),
    })?;
    let mut summary = OperationSummary::default();
    copy_recursive_cancellable(filesystem, source, target, &mut summary, cancel)?;
    Ok(summary)
}

fn copy_recursive_cancellable(
    filesystem: &(impl FileSystem + ?Sized),
    source: &Path,
    target: &Path,
    summary: &mut OperationSummary,
    cancel: &CancelToken,
) -> Result<(), FsError> {
    if cancel.is_cancelled() {
        return Err(FsError::Cancelled {
            path: source.to_path_buf(),
        });
    }
    let metadata = filesystem.symlink_metadata(source)?;
    match metadata.kind {
        EntryKind::File => {
            let bytes = filesystem.copy_file(source, target)?;
            summary.bytes += bytes;
            summary.succeeded += 1;
        }
        EntryKind::Directory => {
            filesystem.create_dir(target)?;
            for child in filesystem.read_dir(source)? {
                copy_recursive_cancellable(
                    filesystem,
                    &child.path,
                    &target.join(&child.name),
                    summary,
                    cancel,
                )?;
            }
            summary.succeeded += 1;
        }
        EntryKind::Parent | EntryKind::Other => {
            return Err(FsError::Unsupported {
                operation: crate::ports::filesystem::FsOperation::CopyFile,
                path: source.to_path_buf(),
            });
        }
    }
    Ok(())
}

pub fn copy_entry_with_conflicts(
    filesystem: &(impl FileSystem + ?Sized),
    operation: OperationId,
    source: &Path,
    target: &Path,
    mut decide: impl FnMut(&Path, &Path) -> ConflictDecision,
) -> Result<OperationSummary, FsError> {
    let mut policy = ConflictPolicy::new(operation);
    copy_with_policy(
        filesystem,
        operation,
        source,
        target,
        &mut policy,
        &mut decide,
    )
}

fn copy_with_policy(
    filesystem: &(impl FileSystem + ?Sized),
    operation: OperationId,
    source: &Path,
    target: &Path,
    policy: &mut ConflictPolicy,
    decide: &mut impl FnMut(&Path, &Path) -> ConflictDecision,
) -> Result<OperationSummary, FsError> {
    if filesystem.symlink_metadata(target).is_ok() {
        let decision = policy
            .remembered(operation)
            .unwrap_or_else(|| decide(source, target));
        policy.apply(&decision);
        match decision {
            ConflictDecision::Overwrite | ConflictDecision::OverwriteAll => {
                filesystem.remove(target, true)?;
            }
            ConflictDecision::Skip | ConflictDecision::SkipAll => {
                return Ok(OperationSummary {
                    skipped: 1,
                    ..OperationSummary::default()
                });
            }
            ConflictDecision::Rename(path) => {
                return copy_with_policy(filesystem, operation, source, &path, policy, decide);
            }
            ConflictDecision::Cancel => {
                return Err(FsError::Cancelled {
                    path: source.to_path_buf(),
                });
            }
        }
    }
    copy_entry(filesystem, source, target)
}

fn copy_recursive(
    filesystem: &(impl FileSystem + ?Sized),
    source: &Path,
    target: &Path,
    summary: &mut OperationSummary,
) -> Result<(), FsError> {
    let metadata = filesystem.symlink_metadata(source)?;
    match metadata.kind {
        EntryKind::File => {
            let bytes = filesystem.copy_file(source, target)?;
            summary.bytes += bytes;
            summary.succeeded += 1;
        }
        EntryKind::Directory => {
            filesystem.create_dir(target)?;
            for child in filesystem.read_dir(source)? {
                copy_recursive(filesystem, &child.path, &target.join(&child.name), summary)?;
            }
            summary.succeeded += 1;
        }
        EntryKind::Parent | EntryKind::Other => {
            return Err(FsError::Unsupported {
                operation: crate::ports::filesystem::FsOperation::CopyFile,
                path: source.to_path_buf(),
            });
        }
    }
    Ok(())
}
