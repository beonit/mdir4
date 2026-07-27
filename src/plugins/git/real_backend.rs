use std::{
    path::{Path, PathBuf},
    process::Command,
};

use super::model::{
    DiffTarget, DirectoryStatus, GitReadBackend, GitStatus, GitStatusRow, RepoRelativePath,
    RepositoryIdentity,
};

#[derive(Default)]
pub struct GitCliReadBackend;

fn status_from_code(code: &str) -> GitStatus {
    if code == "??" {
        GitStatus::Untracked
    } else if code == "!!" {
        GitStatus::Ignored
    } else if matches!(code, "DD" | "AU" | "UD" | "UA" | "DU" | "AA" | "UU") {
        GitStatus::Conflicted
    } else if code.contains('A') {
        GitStatus::Added
    } else if code.contains('D') {
        GitStatus::Deleted
    } else if code.contains('R') {
        GitStatus::Renamed
    } else if code.contains('C') {
        GitStatus::Copied
    } else {
        GitStatus::Modified
    }
}

impl GitCliReadBackend {
    fn run(directory: &Path, arguments: &[&str]) -> Result<String, String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(directory)
            .args(arguments)
            .output()
            .map_err(|_| "Git is unavailable.".to_string())?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout)
                .trim_end()
                .to_string())
        } else {
            Err("Git read operation failed.".into())
        }
    }

    pub fn directory_status(directory: &Path) -> Result<Option<DirectoryStatus>, String> {
        let backend = Self;
        let Some(repository) = backend.discover(directory)? else {
            return Ok(None);
        };
        let rows = backend.status(&repository)?;
        let canonical_directory = directory
            .canonicalize()
            .unwrap_or_else(|_| directory.to_path_buf());
        let canonical_root = repository
            .worktree_root
            .canonicalize()
            .unwrap_or_else(|_| repository.worktree_root.clone());
        let directory_prefix = canonical_directory
            .strip_prefix(canonical_root)
            .unwrap_or(Path::new(""))
            .to_path_buf();
        Ok(Some(DirectoryStatus {
            worktree_root: repository.worktree_root,
            directory_prefix,
            rows,
        }))
    }
}

impl GitReadBackend for GitCliReadBackend {
    fn discover(&self, directory: &Path) -> Result<Option<RepositoryIdentity>, String> {
        let root = match Self::run(directory, &["rev-parse", "--show-toplevel"]) {
            Ok(root) => root,
            Err(_) => return Ok(None),
        };
        let root = PathBuf::from(root);
        let metadata = Self::run(&root, &["rev-parse", "--git-dir"])?;
        let metadata_root = if Path::new(&metadata).is_absolute() {
            PathBuf::from(metadata)
        } else {
            root.join(metadata)
        };
        Ok(Some(RepositoryIdentity {
            metadata_root,
            worktree_root: root,
        }))
    }
    fn status(&self, repository: &RepositoryIdentity) -> Result<Vec<GitStatusRow>, String> {
        let output = Self::run(
            &repository.worktree_root,
            &["status", "--porcelain=v1", "-z"],
        )?;
        let mut rows = Vec::new();
        let mut fields = output.split('\0').filter(|field| !field.is_empty());
        while let Some(field) = fields.next() {
            if field.len() < 4 {
                continue;
            }
            let code = &field[..2];
            let status = status_from_code(code);
            let path = &field[3..];
            if matches!(status, GitStatus::Renamed | GitStatus::Copied) {
                let _old_path = fields.next();
            }
            if let Ok(path) = RepoRelativePath::new(path) {
                rows.push(GitStatusRow {
                    path,
                    status,
                    old_path: None,
                });
            }
        }
        Ok(rows)
    }
    fn diff(
        &self,
        repository: &RepositoryIdentity,
        path: &RepoRelativePath,
        target: DiffTarget,
    ) -> Result<String, String> {
        let mut args = vec!["diff", "--no-ext-diff"];
        match target {
            DiffTarget::Staged => args.push("--cached"),
            DiffTarget::Combined => args.push("HEAD"),
            DiffTarget::Unstaged => {}
        }
        let path = path
            .as_path()
            .to_str()
            .ok_or_else(|| "Invalid repository path.".to_string())?;
        args.extend(["--", path]);
        Self::run(&repository.worktree_root, &args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_unmerged_porcelain_code_has_conflict_emphasis() {
        for code in ["DD", "AU", "UD", "UA", "DU", "AA", "UU"] {
            assert_eq!(status_from_code(code), GitStatus::Conflicted);
        }
    }
}
