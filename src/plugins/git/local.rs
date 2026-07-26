use std::sync::Mutex;

use crate::{
    plugins::git::model::{GitStatus, RepoRelativePath},
    runtime::lane::MutationCoordinator,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MutationKind {
    Stage,
    Unstage,
    Commit { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutationPlan {
    pub kind: MutationKind,
    pub targets: Vec<RepoRelativePath>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitIdentity {
    pub name: String,
    pub email: String,
}

pub fn plan_commit(
    message: impl Into<String>,
    staged: usize,
    identity: Option<CommitIdentity>,
) -> Result<(MutationPlan, CommitIdentity), String> {
    let message = message.into();
    if message.trim().is_empty() {
        return Err("Commit message cannot be blank.".into());
    }
    if staged == 0 {
        return Err("No staged changes to commit.".into());
    }
    let identity = identity
        .filter(|identity| !identity.name.trim().is_empty() && !identity.email.trim().is_empty())
        .ok_or_else(|| "Git author name and email are required.".to_string())?;
    Ok((
        MutationPlan {
            kind: MutationKind::Commit { message },
            targets: Vec::new(),
        },
        identity,
    ))
}

pub trait GitMutationBackend: Send + Sync {
    fn execute(&self, plan: &MutationPlan) -> Result<(), String>;
}

#[derive(Default)]
pub struct FakeGitMutationBackend {
    calls: Mutex<Vec<MutationPlan>>,
    failure: Mutex<Option<String>>,
}

impl FakeGitMutationBackend {
    pub fn calls(&self) -> Vec<MutationPlan> {
        self.calls.lock().unwrap().clone()
    }
    pub fn fail_with(&self, message: impl Into<String>) {
        *self.failure.lock().unwrap() = Some(message.into());
    }
}

impl GitMutationBackend for FakeGitMutationBackend {
    fn execute(&self, plan: &MutationPlan) -> Result<(), String> {
        self.calls.lock().unwrap().push(plan.clone());
        self.failure.lock().unwrap().clone().map_or(Ok(()), Err)
    }
}

pub fn plan_targets(
    kind: MutationKind,
    targets: Vec<RepoRelativePath>,
) -> Result<MutationPlan, String> {
    if targets.is_empty() {
        return Err("Select at least one repository file.".into());
    }
    Ok(MutationPlan { kind, targets })
}

pub fn preflight_stage(
    kind: MutationKind,
    rows: &[(RepoRelativePath, GitStatus)],
) -> Result<MutationPlan, String> {
    let allowed = |status| match kind {
        MutationKind::Stage => matches!(
            status,
            GitStatus::Modified
                | GitStatus::Added
                | GitStatus::Deleted
                | GitStatus::Renamed
                | GitStatus::Untracked
        ),
        MutationKind::Unstage => matches!(
            status,
            GitStatus::Modified | GitStatus::Added | GitStatus::Deleted | GitStatus::Renamed
        ),
        MutationKind::Commit { .. } => false,
    };
    if rows.is_empty() || rows.iter().any(|(_, status)| !allowed(*status)) {
        return Err("Selected files cannot be used by this Git operation.".into());
    }
    plan_targets(kind, rows.iter().map(|(path, _)| path.clone()).collect())
}

pub fn execute_with_lease(
    backend: &dyn GitMutationBackend,
    coordinator: &MutationCoordinator,
    plan: &MutationPlan,
) -> Result<(), String> {
    let _lease = coordinator
        .try_acquire()
        .map_err(|_| "Another local mutation is running.".to_string())?;
    backend.execute(plan)
}
