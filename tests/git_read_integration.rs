use mdir4::plugins::git::{
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
