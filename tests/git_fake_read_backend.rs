use mdir4::plugins::git::{
    fake_read_backend::{FakeGitReadBackend, ReadCall},
    model::{
        DiffTarget, GitReadBackend, GitStatus, GitStatusRow, RepoRelativePath, RepositoryIdentity,
    },
};
use std::path::PathBuf;

#[test]
fn fake_read_backend_records_read_only_calls_in_order() {
    let repo = RepositoryIdentity {
        metadata_root: PathBuf::from("/repo/.git"),
        worktree_root: PathBuf::from("/repo"),
    };
    let path = RepoRelativePath::new("src/lib.rs").unwrap();
    let backend = FakeGitReadBackend::default()
        .with_discovery("/repo", Some(repo.clone()))
        .with_status(
            "/repo",
            vec![GitStatusRow {
                path: path.clone(),
                status: GitStatus::Modified,
                old_path: None,
            }],
        )
        .with_diff("/repo", path.clone(), DiffTarget::Unstaged, "diff");
    assert_eq!(
        backend.discover(std::path::Path::new("/repo")).unwrap(),
        Some(repo.clone())
    );
    assert_eq!(backend.status(&repo).unwrap().len(), 1);
    assert_eq!(
        backend.diff(&repo, &path, DiffTarget::Unstaged).unwrap(),
        "diff"
    );
    assert_eq!(backend.calls().len(), 3);
    assert!(matches!(backend.calls()[0], ReadCall::Discover(_)));
}
