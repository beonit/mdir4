use mdir4::{
    plugins::git::{
        local::{
            CommitIdentity, FakeGitMutationBackend, MutationKind, execute_with_lease, plan_commit,
            plan_targets, preflight_stage,
        },
        model::{GitStatus, RepoRelativePath},
    },
    runtime::lane::MutationCoordinator,
};

#[test]
fn mutation_plan_rejects_empty_targets_and_shared_lease_rejects_overlap() {
    assert!(plan_targets(MutationKind::Stage, Vec::new()).is_err());
    let plan = plan_targets(
        MutationKind::Stage,
        vec![RepoRelativePath::new("src/lib.rs").unwrap()],
    )
    .unwrap();
    let backend = FakeGitMutationBackend::default();
    let coordinator = MutationCoordinator::default();
    execute_with_lease(&backend, &coordinator, &plan).unwrap();
    assert_eq!(backend.calls().len(), 1);
    let active = coordinator.try_acquire().unwrap();
    assert!(execute_with_lease(&backend, &coordinator, &plan).is_err());
    drop(active);
}

#[test]
fn stage_preflight_rejects_mixed_invalid_selection_before_backend_execution() {
    let valid = RepoRelativePath::new("changed.txt").unwrap();
    let invalid = RepoRelativePath::new("ignored.txt").unwrap();
    assert!(
        preflight_stage(
            MutationKind::Stage,
            &[
                (valid.clone(), GitStatus::Modified),
                (invalid, GitStatus::Ignored)
            ]
        )
        .is_err()
    );
    let plan = preflight_stage(MutationKind::Stage, &[(valid, GitStatus::Untracked)]).unwrap();
    assert!(matches!(plan.kind, MutationKind::Stage));
}

#[test]
fn commit_preflight_requires_message_staged_change_and_explicit_identity() {
    let identity = CommitIdentity {
        name: "Ada".into(),
        email: "ada@example.test".into(),
    };
    assert!(plan_commit("", 1, Some(identity.clone())).is_err());
    assert!(plan_commit("message", 0, Some(identity.clone())).is_err());
    assert!(plan_commit("message", 1, None).is_err());
    let (plan, selected) = plan_commit("message", 1, Some(identity.clone())).unwrap();
    assert!(matches!(plan.kind, MutationKind::Commit { .. }));
    assert_eq!(selected, identity);
}
