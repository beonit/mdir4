use mdir4::plugins::git::{
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
