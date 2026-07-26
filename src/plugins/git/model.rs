use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RepoRelativePath(PathBuf);

impl RepoRelativePath {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, String> {
        let path = path.into();
        if path.is_absolute()
            || path.components().any(|part| {
                matches!(
                    part,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err("repository-relative path must not escape the worktree".into());
        }
        Ok(Self(path))
    }
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GitStatus {
    Clean,
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Ignored,
    Conflicted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiffTarget {
    Staged,
    Unstaged,
    Combined,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitStatusRow {
    pub path: RepoRelativePath,
    pub status: GitStatus,
    pub old_path: Option<RepoRelativePath>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryIdentity {
    pub metadata_root: PathBuf,
    pub worktree_root: PathBuf,
}

pub trait GitReadBackend: Send + Sync {
    fn discover(&self, directory: &Path) -> Result<Option<RepositoryIdentity>, String>;
    fn status(&self, repository: &RepositoryIdentity) -> Result<Vec<GitStatusRow>, String>;
    fn diff(
        &self,
        repository: &RepositoryIdentity,
        path: &RepoRelativePath,
        target: DiffTarget,
    ) -> Result<String, String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repo_relative_path_rejects_absolute_and_parent_escape() {
        assert!(RepoRelativePath::new("src/lib.rs").is_ok());
        assert!(RepoRelativePath::new("../secret").is_err());
        assert!(RepoRelativePath::new("/tmp/file").is_err());
    }
}
