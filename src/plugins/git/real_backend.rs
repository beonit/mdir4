use std::{
    path::{Path, PathBuf},
    process::Command,
};

use super::model::{
    DiffTarget, GitReadBackend, GitStatus, GitStatusRow, RepoRelativePath, RepositoryIdentity,
};

#[derive(Default)]
pub struct GitCliReadBackend;

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
        let output = Self::run(&repository.worktree_root, &["status", "--porcelain=v1"])?;
        let mut rows = Vec::new();
        for line in output.lines() {
            if line.len() < 4 {
                continue;
            }
            let code = &line[..2];
            let status = if code == "??" {
                GitStatus::Untracked
            } else if code == "!!" {
                GitStatus::Ignored
            } else if matches!(code, "UU" | "AA" | "DD") {
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
            };
            if let Ok(path) = RepoRelativePath::new(&line[3..]) {
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
