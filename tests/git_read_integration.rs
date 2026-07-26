use mdir4::plugins::git::{
    branch::{GitCliBranchBackend, validate_branch_name},
    history::{GitCliHistoryBackend, GitHistoryBackend},
    local::{GitCliMutationBackend, GitMutationBackend, MutationKind, plan_targets},
    model::{DiffTarget, GitReadBackend},
    real_backend::GitCliReadBackend,
};
use std::{fs, process::Command};
use tempfile::tempdir;

fn git(directory: &std::path::Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(directory)
            .args(args)
            .status()
            .unwrap()
            .success()
    );
}

#[test]
fn cli_read_backend_discovers_status_and_reads_unstaged_and_combined_diffs_without_mutating_repo() {
    let temp = tempdir().unwrap();
    git(temp.path(), &["init", "-q"]);
    git(temp.path(), &["config", "user.name", "Test"]);
    git(
        temp.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    fs::write(temp.path().join("note.txt"), "one\n").unwrap();
    git(temp.path(), &["add", "note.txt"]);
    git(temp.path(), &["commit", "-qm", "initial"]);
    fs::write(temp.path().join("note.txt"), "two\n").unwrap();
    let backend = GitCliReadBackend;
    let repo = backend.discover(temp.path()).unwrap().unwrap();
    let rows = backend.status(&repo).unwrap();
    assert_eq!(rows.len(), 1);
    assert!(
        backend
            .diff(&repo, &rows[0].path, DiffTarget::Unstaged)
            .unwrap()
            .contains("-one")
    );
    assert!(
        backend
            .diff(&repo, &rows[0].path, DiffTarget::Combined)
            .unwrap()
            .contains("+two")
    );
}

#[test]
fn cli_mutation_backend_stages_and_unstages_only_the_requested_path() {
    let temp = tempdir().unwrap();
    git(temp.path(), &["init", "-q"]);
    git(temp.path(), &["config", "user.name", "Test"]);
    git(
        temp.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    fs::write(temp.path().join("new.txt"), "new\n").unwrap();
    let backend = GitCliMutationBackend::new(temp.path());
    let path = mdir4::plugins::git::model::RepoRelativePath::new("new.txt").unwrap();

    backend
        .execute(&plan_targets(MutationKind::Stage, vec![path.clone()]).unwrap())
        .unwrap();
    let cached = Command::new("git")
        .arg("-C")
        .arg(temp.path())
        .args(["diff", "--cached", "--name-only"])
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&cached.stdout).contains("new.txt"));

    backend
        .execute(&plan_targets(MutationKind::Unstage, vec![path]).unwrap())
        .unwrap();
    let status = Command::new("git")
        .arg("-C")
        .arg(temp.path())
        .args(["status", "--porcelain=v1"])
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&status.stdout).starts_with("?? new.txt"));
}

#[test]
fn cli_mutation_backend_commits_staged_changes_with_the_configured_identity() {
    let temp = tempdir().unwrap();
    git(temp.path(), &["init", "-q"]);
    git(temp.path(), &["config", "user.name", "Test"]);
    git(
        temp.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    fs::write(temp.path().join("new.txt"), "new\n").unwrap();
    let backend = GitCliMutationBackend::new(temp.path());
    let path = mdir4::plugins::git::model::RepoRelativePath::new("new.txt").unwrap();
    backend
        .execute(&plan_targets(MutationKind::Stage, vec![path]).unwrap())
        .unwrap();
    backend
        .execute(&mdir4::plugins::git::local::MutationPlan {
            kind: MutationKind::Commit {
                message: "add new file".into(),
            },
            targets: Vec::new(),
        })
        .unwrap();
    let subject = Command::new("git")
        .arg("-C")
        .arg(temp.path())
        .args(["log", "-1", "--format=%s"])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&subject.stdout).trim(),
        "add new file"
    );
}

#[test]
fn cli_history_backend_lists_commits_and_reads_the_selected_detail() {
    let temp = tempdir().unwrap();
    git(temp.path(), &["init", "-q"]);
    git(temp.path(), &["config", "user.name", "Test User"]);
    git(
        temp.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    fs::write(temp.path().join("note.txt"), "one\n").unwrap();
    git(temp.path(), &["add", "note.txt"]);
    git(temp.path(), &["commit", "-qm", "first subject"]);
    fs::write(temp.path().join("note.txt"), "two\n").unwrap();
    git(temp.path(), &["commit", "-am", "second subject", "-q"]);

    let backend = GitCliHistoryBackend;
    let entries = backend.log(temp.path(), 10).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].subject, "second subject");
    let detail = backend.detail(temp.path(), &entries[0].hash).unwrap();
    assert!(detail.contains("second subject"));
    assert!(detail.contains("Test User"));
}

#[test]
fn cli_branch_backend_lists_current_branch_and_creates_a_valid_branch() {
    let temp = tempdir().unwrap();
    git(temp.path(), &["init", "-q"]);
    git(temp.path(), &["config", "user.name", "Test"]);
    git(
        temp.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    fs::write(temp.path().join("note.txt"), "one\n").unwrap();
    git(temp.path(), &["add", "note.txt"]);
    git(temp.path(), &["commit", "-qm", "initial"]);
    let backend = GitCliBranchBackend;
    let original = backend
        .list(temp.path())
        .unwrap()
        .into_iter()
        .find(|branch| branch.current)
        .unwrap()
        .name;
    backend.create(temp.path(), "feature/demo").unwrap();
    backend.checkout(temp.path(), "feature/demo").unwrap();
    let branches = backend.list(temp.path()).unwrap();
    assert!(branches.iter().any(|branch| branch.name == "feature/demo"));
    assert!(
        branches
            .iter()
            .any(|branch| branch.name == "feature/demo" && branch.current)
    );
    fs::write(temp.path().join("note.txt"), "dirty\n").unwrap();
    assert!(backend.checkout(temp.path(), &original).is_err());
    assert!(validate_branch_name("bad name").is_err());
}
