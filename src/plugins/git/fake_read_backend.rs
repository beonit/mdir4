use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Mutex,
};

use super::model::{
    DiffTarget, GitReadBackend, GitStatusRow, RepoRelativePath, RepositoryIdentity,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadCall {
    Discover(PathBuf),
    Status(RepositoryIdentity),
    Diff(RepoRelativePath, DiffTarget),
}

#[derive(Default)]
pub struct FakeGitReadBackend {
    discoveries: BTreeMap<PathBuf, Option<RepositoryIdentity>>,
    statuses: BTreeMap<PathBuf, Vec<GitStatusRow>>,
    diffs: BTreeMap<(PathBuf, RepoRelativePath, DiffTarget), String>,
    calls: Mutex<Vec<ReadCall>>,
}

impl FakeGitReadBackend {
    pub fn with_discovery(
        mut self,
        directory: impl Into<PathBuf>,
        repository: Option<RepositoryIdentity>,
    ) -> Self {
        self.discoveries.insert(directory.into(), repository);
        self
    }
    pub fn with_status(mut self, root: impl Into<PathBuf>, rows: Vec<GitStatusRow>) -> Self {
        self.statuses.insert(root.into(), rows);
        self
    }
    pub fn with_diff(
        mut self,
        root: impl Into<PathBuf>,
        path: RepoRelativePath,
        target: DiffTarget,
        diff: impl Into<String>,
    ) -> Self {
        self.diffs.insert((root.into(), path, target), diff.into());
        self
    }
    pub fn calls(&self) -> Vec<ReadCall> {
        self.calls.lock().unwrap().clone()
    }
}

impl GitReadBackend for FakeGitReadBackend {
    fn discover(&self, directory: &Path) -> Result<Option<RepositoryIdentity>, String> {
        self.calls
            .lock()
            .unwrap()
            .push(ReadCall::Discover(directory.into()));
        Ok(self.discoveries.get(directory).cloned().unwrap_or(None))
    }
    fn status(&self, repository: &RepositoryIdentity) -> Result<Vec<GitStatusRow>, String> {
        self.calls
            .lock()
            .unwrap()
            .push(ReadCall::Status(repository.clone()));
        Ok(self
            .statuses
            .get(&repository.worktree_root)
            .cloned()
            .unwrap_or_default())
    }
    fn diff(
        &self,
        repository: &RepositoryIdentity,
        path: &RepoRelativePath,
        target: DiffTarget,
    ) -> Result<String, String> {
        self.calls
            .lock()
            .unwrap()
            .push(ReadCall::Diff(path.clone(), target));
        self.diffs
            .get(&(repository.worktree_root.clone(), path.clone(), target))
            .cloned()
            .ok_or_else(|| "fake diff not configured".into())
    }
}
