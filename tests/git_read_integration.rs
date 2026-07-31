use mdir4::plugins::git::{
    branch::{GitCliBranchBackend, validate_branch_name},
    history::{GitCliHistoryBackend, GitHistoryBackend},
    local::{GitCliMutationBackend, GitMutationBackend, MutationKind, plan_targets},
    model::{DiffTarget, GitReadBackend},
    real_backend::GitCliReadBackend,
    stash::{GitCliStashBackend, GitStashBackend},
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
fn directory_status_returns_one_cached_map_source_for_the_refresh() {
    let temp = tempdir().unwrap();
    git(temp.path(), &["init", "-q"]);
    fs::write(temp.path().join("scratch.txt"), "new\n").unwrap();

    let snapshot = GitCliReadBackend::directory_status(temp.path())
        .unwrap()
        .unwrap();
    assert_eq!(
        snapshot.worktree_root.canonicalize().unwrap(),
        temp.path().canonicalize().unwrap()
    );
    assert_eq!(snapshot.rows.len(), 1);
    assert_eq!(
        snapshot.rows[0].status,
        mdir4::plugins::git::model::GitStatus::Untracked
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
fn cli_mutation_backend_amends_head_without_changing_its_message() {
    let temp = tempdir().unwrap();
    git(temp.path(), &["init", "-q"]);
    git(temp.path(), &["config", "user.name", "Test"]);
    git(
        temp.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    fs::write(temp.path().join("note.txt"), "one\n").unwrap();
    git(temp.path(), &["add", "note.txt"]);
    git(temp.path(), &["commit", "-qm", "original subject"]);
    fs::write(temp.path().join("note.txt"), "two\n").unwrap();
    git(temp.path(), &["add", "note.txt"]);

    GitCliMutationBackend::new(temp.path())
        .execute(&mdir4::plugins::git::local::MutationPlan {
            kind: MutationKind::Amend,
            targets: Vec::new(),
        })
        .unwrap();

    let log = Command::new("git")
        .arg("-C")
        .arg(temp.path())
        .args(["log", "--format=%s"])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&log.stdout).trim(),
        "original subject"
    );
    assert_eq!(
        fs::read_to_string(temp.path().join("note.txt")).unwrap(),
        "two\n"
    );
}

#[test]
fn cli_branch_backend_fetches_and_prunes_configured_remotes() {
    let root = tempdir().unwrap();
    let repository = root.path().join("work");
    let remote = root.path().join("remote.git");
    fs::create_dir(&repository).unwrap();
    fs::create_dir(&remote).unwrap();
    git(&repository, &["init", "-q"]);
    git(&remote, &["init", "--bare", "-q"]);
    git(
        &repository,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );

    GitCliBranchBackend.fetch(&repository).unwrap();
}

#[test]
fn cli_mutation_backend_stashes_all_changes_and_discards_only_tracked_changes() {
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
    fs::write(temp.path().join("new.txt"), "new\n").unwrap();
    let backend = GitCliMutationBackend::new(temp.path());
    backend
        .execute(&mdir4::plugins::git::local::MutationPlan {
            kind: MutationKind::Stash {
                message: "work".into(),
            },
            targets: Vec::new(),
        })
        .unwrap();
    let status = Command::new("git")
        .arg("-C")
        .arg(temp.path())
        .args(["status", "--porcelain=v1"])
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&status.stdout).trim().is_empty());
    let stash = Command::new("git")
        .arg("-C")
        .arg(temp.path())
        .args(["stash", "list"])
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&stash.stdout).contains("work"));

    let stash_backend = GitCliStashBackend;
    let entries = stash_backend.list(temp.path()).unwrap();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].message.ends_with(": work"));
    stash_backend
        .apply(temp.path(), &entries[0].reference)
        .unwrap();
    assert_eq!(
        fs::read_to_string(temp.path().join("note.txt")).unwrap(),
        "two\n"
    );
    assert!(temp.path().join("new.txt").exists());
    stash_backend
        .drop(temp.path(), &entries[0].reference)
        .unwrap();
    assert!(stash_backend.list(temp.path()).unwrap().is_empty());

    git(
        temp.path(),
        &["restore", "--staged", "--worktree", "note.txt"],
    );
    fs::remove_file(temp.path().join("new.txt")).unwrap();
    fs::write(temp.path().join("note.txt"), "three\n").unwrap();
    let path = mdir4::plugins::git::model::RepoRelativePath::new("note.txt").unwrap();
    backend
        .execute(&mdir4::plugins::git::local::MutationPlan {
            kind: MutationKind::Discard,
            targets: vec![path],
        })
        .unwrap();
    assert_eq!(
        fs::read_to_string(temp.path().join("note.txt")).unwrap(),
        "one\n"
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
    git(temp.path(), &["branch", "feature/log-label"]);

    let backend = GitCliHistoryBackend;
    let entries = backend.log(temp.path(), 10).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].subject, "second subject");
    assert!(entries[0].references.contains("feature/log-label"));
    let detail = backend.detail(temp.path(), &entries[0].hash).unwrap();
    assert!(detail.summary.contains("second subject"));
    assert!(detail.summary.contains("Test User"));
    assert_eq!(detail.files.len(), 1);
    assert_eq!(detail.files[0].status, "M");
    assert_eq!(detail.files[0].path, std::path::PathBuf::from("note.txt"));
    let diff = backend
        .file_diff(temp.path(), &entries[0].hash, &detail.files[0])
        .unwrap();
    assert!(diff.contains("-one"));
    assert!(diff.contains("+two"));
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

#[test]
fn cli_branch_backend_rebases_current_branch_onto_selected_target_and_rejects_dirty_worktrees() {
    let temp = tempdir().unwrap();
    git(temp.path(), &["init", "-q"]);
    git(temp.path(), &["config", "user.name", "Test"]);
    git(
        temp.path(),
        &["config", "user.email", "test@example.invalid"],
    );
    fs::write(temp.path().join("note.txt"), "base\n").unwrap();
    git(temp.path(), &["add", "note.txt"]);
    git(temp.path(), &["commit", "-qm", "base"]);
    let backend = GitCliBranchBackend;
    let base = backend
        .list(temp.path())
        .unwrap()
        .into_iter()
        .find(|branch| branch.current)
        .unwrap()
        .name;
    backend.create(temp.path(), "feature/rebase").unwrap();
    backend.checkout(temp.path(), "feature/rebase").unwrap();
    fs::write(temp.path().join("feature.txt"), "feature\n").unwrap();
    git(temp.path(), &["add", "feature.txt"]);
    git(temp.path(), &["commit", "-qm", "feature"]);
    backend.checkout(temp.path(), &base).unwrap();
    fs::write(temp.path().join("base.txt"), "base update\n").unwrap();
    git(temp.path(), &["add", "base.txt"]);
    git(temp.path(), &["commit", "-qm", "base update"]);
    backend.checkout(temp.path(), "feature/rebase").unwrap();
    backend.rebase(temp.path(), &base).unwrap();
    assert!(temp.path().join("base.txt").exists());
    assert!(temp.path().join("feature.txt").exists());
    fs::write(temp.path().join("dirty.txt"), "dirty\n").unwrap();
    assert!(backend.rebase(temp.path(), &base).is_err());
}
