# Git built-in 작업 카드

## 실행 규칙

1. production 구현은 v1 `R1` 완료 뒤 시작한다.
2. 한 번에 카드 하나만 진행하고 선행 카드의 test/evidence를 확인한다.
3. 실패 테스트 → 최소 구현 → 카드 테스트 → 공통 gate → progress 증거 순서를 지킨다.
4. G0에는 Git-specific production code를, G1에는 mutation/network capability를 넣지 않는다.
5. 카드의 `진행` checkbox는 실제 증거 없이 체크하지 않는다.

공통 gate:

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
```

## 카드 소유권 맵 (규범)

아래 표가 각 카드의 **표준 primary ownership**이다. 카드 본문의 `작업`/`테스트` 항목은
이 표의 경로를 상속한다. 아직 없는 경로는 해당 카드가 시작될 때 처음 만들며, 선행 카드에서
미리 만들지 않는다. 표에 없는 파일 변경이 꼭 필요하면 카드 진행 증거에 이유와 영향을 남긴다.
`tests/*.rs`는 Cargo가 직접 발견하는 integration test target이고, 공용 fixture만
`tests/support/`에 둔다. 설계·통합 수용 카드는 Rust production 파일을 억지로 만들지 않고
결정 문서 또는 acceptance/progress 기록을 primary 산출물로 소유한다.

| 카드 | Primary production / design file(s) | Primary test / evidence file(s) |
|---|---|---|
| G0-01 | `src/plugins/mod.rs`, `src/plugins/api.rs` | `tests/plugin_api.rs` |
| G0-02 | `src/plugins/manager.rs` | `tests/plugin_manager.rs` |
| G0-03 | `src/plugins/worker.rs` | `tests/plugin_worker.rs`, `tests/scenarios/plugin_host.yml` |
| G0-04 | `src/plugins/api.rs`, `src/plugins/manager.rs`, `src/app/command_registry.rs`, `src/ui.rs` | `tests/plugin_contributions.rs`, `tests/snapshots/plugin_contributions__host_contract.snap` |
| G0-05 | `src/plugins/manager.rs`, `src/plugins/api.rs` | `tests/plugin_views.rs` |
| G0-06 | `src/config/plugins.rs`, `src/plugins/manager.rs`, `src/app.rs` | `tests/plugin_config.rs`, `docs/plugins/git/05-acceptance-matrix.md`, `docs/implementation-plan/progress.md` |
| G1-00 | `src/plugins/git/mod.rs`, `src/plugins/git/config.rs`, `src/app.rs` | `tests/git_factory_config.rs` |
| G1-01 | `src/plugins/git/model.rs`, `src/plugins/git/backend.rs` | `tests/git_model.rs` |
| G1-02 | `src/plugins/git/fake_read_backend.rs` | `tests/support/git_fixtures.rs`, `tests/git_fake_read_backend.rs` |
| G1-03 | `src/plugins/git/state.rs`, `src/plugins/git/reducer.rs` | `tests/git_reducer.rs` |
| G1-04 | `src/plugins/git/discovery.rs` | `tests/git_discovery.rs` |
| G1-05 | `src/plugins/git/decoration.rs`, `src/plugins/git/config.rs` | `tests/git_decoration.rs`, `tests/snapshots/git_main__mixed__80x25__ready.snap` |
| G1-06 | `src/plugins/git/status_summary.rs` | `tests/git_status_summary.rs`, `tests/snapshots/git_main__mixed__80x25__status_summary.snap` |
| G1-07 | `src/plugins/git/ui/status_view.rs` | `tests/git_status_view.rs`, `tests/snapshots/git_status__mixed__80x25__ready.snap` |
| G1-08 | `src/plugins/git/ui/diff_view.rs`, `src/plugins/git/backend.rs` | `tests/git_diff_view.rs`, `tests/snapshots/git_diff__mixed__80x25__combined.snap` |
| G1-09 | `src/plugins/git/reducer.rs`, `src/plugins/git/mod.rs` | `tests/git_triggers.rs`, `tests/git_perf.rs`, `tests/scenarios/git_read.yml` |
| G1-10 | `docs/plugins/git/adr-read-backend-selection.md` | `tests/git_backend_conformance.rs`, `docs/implementation-plan/progress.md` |
| G1-11 | `src/plugins/git/real_backend/mod.rs` | `tests/git_backend_conformance.rs`, `tests/git_read_integration.rs` |
| G1-12 | 없음 — integration/acceptance gate | `tests/git_g1_acceptance.rs`, `docs/plugins/git/05-acceptance-matrix.md`, `docs/implementation-plan/progress.md` |
| G2-01 | `src/plugins/git/local/backend.rs`, `src/plugins/git/local/fake_backend.rs`, `src/plugins/git/local/planner.rs` | `tests/git_mutation_contract.rs` |
| G2-02 | `src/plugins/git/local/stage.rs` | `tests/git_stage_unstage.rs` |
| G2-03 | `src/plugins/git/local/commit.rs` | `tests/git_commit.rs` |
| G2-04 | `src/plugins/git/history_backend.rs`, `src/plugins/git/ui/log_view.rs` | `tests/git_history.rs`, `tests/snapshots/git_log__many__80x25__ready.snap` |
| G2-05 | `src/plugins/git/local/branch.rs`, `src/plugins/git/ui/branch_view.rs` | `tests/git_branch.rs`, `tests/snapshots/git_branch__many__80x25__ready.snap` |
| G2-06 | `src/plugins/git/local/checkout.rs` | `tests/git_checkout.rs` |
| G2-07 | `src/plugins/git/local/stash.rs` | `tests/git_stash_create.rs` |
| G2-08 | `src/plugins/git/local/stash.rs`, `src/plugins/git/ui/stash_view.rs` | `tests/git_stash_manage.rs`, `tests/snapshots/git_stash__stash-list__80x25__ready.snap` |
| G2-09 | `src/plugins/git/local/discard.rs` | `tests/git_discard.rs` |
| G2-10 | 없음 — integration/acceptance gate | `tests/git_local_mutations.rs`, `docs/plugins/git/05-acceptance-matrix.md`, `docs/implementation-plan/progress.md` |
| G3-00 | `docs/plugins/git/adr-remote-operations.md` | `docs/plugins/git/05-acceptance-matrix.md`, `docs/implementation-plan/progress.md` |
| G3-01 | `src/plugins/git/transport/model.rs`, `src/plugins/git/transport/reducer.rs` | `tests/git_remote_metadata.rs` |
| G3-02 | `src/plugins/git/transport/backend.rs`, `src/plugins/git/transport/fake_backend.rs`, `src/plugins/git/transport/worker.rs`, `src/plugins/git/transport/redaction.rs` | `tests/git_transport_contract.rs`, `tests/git_transport_worker.rs`, `tests/git_secret_scan.rs` |
| G3-03 | `src/plugins/git/transport/fetch.rs` | `tests/git_fetch.rs` |
| G3-04 | `src/plugins/git/transport/pull.rs` | `tests/git_pull.rs` |
| G3-05 | `src/plugins/git/transport/push.rs` | `tests/git_push.rs` |
| G3-06 | `src/plugins/git/transport/remote_manage.rs` | `tests/git_remote_manage.rs` |
| G3-07 | `src/plugins/git/transport/clone.rs` | `tests/git_clone.rs` |
| G3-08 | `src/plugins/git/transport/conflict.rs`, `src/plugins/git/ui/conflict_view.rs` | `tests/git_conflict.rs`, `tests/snapshots/git_conflict__remote-conflict__80x25__ready.snap` |
| G3-09 | 없음 — integration/security/acceptance gate | `tests/git_remote_integration.rs`, `tests/git_secret_scan.rs`, `docs/plugins/git/05-acceptance-matrix.md`, `docs/implementation-plan/progress.md` |

---

# G0 — Generic built-in extension host

## G0-01 Result 기반 Plugin API와 FakePlugin

- 선행: R1 완료, built-in plugin boundary ADR
- 목표: Git을 모르는 callback/result/effect/payload 계약을 고정한다.
- 작업: `PluginId`, `Plugin`, `PluginError`, generic contribution, `PluginEffect`,
  `PluginResult`, opaque owner payload, request/generation, Remote 타입을 모르는
  `HostPathContext::{Local,Unsupported}`를 정의한다. 모든 callback은
  `Result<_, PluginError>`이고 test-only FakePlugin은 error/panic/payload mismatch를 주입한다.
- 테스트: ordering key, duplicate id 표현, payload owner/type mismatch, API에 Git 타입 없음.
- 완료: FakePlugin만으로 callback/contribution/effect/result를 종단 assert한다.
- 금지: GitPlugin, dynamic ABI, serde trait object, service locator.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G0-02 PluginManager, collision과 fault isolation

- 선행: G0-01
- 목표: plugin 하나의 error/panic이 Core나 다른 plugin을 중단하지 않게 한다.
- 작업: static factory 등록, duplicate plugin/command/view/status id 거부, callback
  `catch_unwind`, `Faulted` session, contribution/result drop, disable fast path를 구현한다.
  re-enable은 generation을 올리고 factory에서 새 instance를 만든다.
- 테스트: disabled callback 0, error/panic 뒤 다른 contribution 유지, collision별 이유,
  faulted late result 무시, clean re-enable.
- 완료: Core reducer에 plugin id별 match가 없고 PLUG-02/03/04가 통과한다.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G0-03 Plugin read lane, cancellation과 shutdown

- 선행: G0-02, ADR-005 background-work lane
- 목표: plugin read I/O가 UI와 local mutation을 막지 않게 한다.
- 작업: 기존 Core/local mutation lane과 **별도 single FIFO plugin-read lane**을 만든다.
  M2 공통 OperationId/CancelToken/Deadline/JobControl을 재사용하고 기본 capacity 16의
  non-blocking sender, throttled progress, terminal result exactly once,
  stale generation 폐기, queue close/cancel/deadline-backed join을 구현한다. Core scenario v1에
  tagged generic plugin effect completion/failure/cancel과 단계별 plugin assertion action을
  additive하게 추가하고 기존 fixture regression을 유지한다.
- 테스트: injected capacity 1에서 duplicate refresh 최신값 coalesce와 noncoalescible Busy,
  submit UI 비차단, blocked plugin read 중 navigation과 Core fake mutation 완료,
  queued/running cancel, out-of-order result, panic 변환, shutdown join/no orphan.
- 완료: callback/render에서 job run 0이고 느린 Git read가 copy/delete lane을 점유하지 않는다.
- 금지: 측정 없는 pool/Tokio, 기존 single worker에 plugin job 혼합.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G0-04 Decoration/status/command host 연결

- 선행: G0-02~03
- 목표: generic contribution을 layout/status/CommandRegistry에 안전하게 연결한다.
- 작업: reserved cell 계산, StyledText/StyledSpan, 확장 가능한 namespaced `StyleRoleId`와
  default/fallback map, status
  full→compact→hidden, `CommandAvailability::Disabled { reason }`, context/key collision
  resolution을 구현한다.
- 테스트: prefix 0/1/2셀, 79/80/81/120열, invalid width fault, cursor+marked style,
  status Core 우선, custom key collision과 표시 이유.
- 완료: FakePlugin snapshot에서 style role/cell range/command hint가 registry와 일치한다.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G0-05 PluginView ownership과 screen lifecycle

- 선행: G0-02, G0-04
- 목표: plugin view가 screen stack을 독점/누수하지 않고 Main state를 보존한다.
- 작업: owner PluginId, namespaced ViewId, command context, open/close ownership 검증,
  disable/fault 시 safe close, unknown/foreign result drop을 구현한다.
- 테스트: duplicate view id, foreign close/result, Esc, disable/fault while open, Main
  path/cursor/marks 보존.
- 완료: active view는 하나이고 owner만 갱신할 수 있다.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G0-06 Generic plugin config와 G0 수용

- 선행: G0-01~05, v1 config/keymap/theme
- 목표: Git을 등록하지 않고 generic plugin enable/config/keymap round-trip을 완성한다.
- 작업: versioned `[plugins]` map, unknown key 보존/경고, generic Settings toggle,
  namespaced command override, save-failure session behavior를 구현한다. FakePlugin만 등록한다.
- 테스트: round-trip/corrupt/unknown, toggle/restart/save failure, generic label/Help/error English와
  plugin 제공 Unicode text 원문 보존, PLUG-01~11 전체.
- 완료: Git module/config/default command가 production tree에 없고 G0 acceptance가 모두 연결된다.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

---

# G1 — Local read-only Git

## G1-00 Git factory, 설정과 Local-only 등록

- 선행: G0-06
- 목표: composition root에 첫 Git instance와 명령/설정을 명시적으로 등록한다.
- 작업: `GitPluginFactory`, `[plugins.git]`의 정리된 key, namespaced theme role,
  `plugin.git.open_status` 기본 Alt+G를 등록한다. shortcut은 Git config에 넣지 않는다.
  `HostPathContext::Local`이 아니면 backend callback/job과 동적 decoration/status/view
  contribution은 0이고, 정적 command definition은 이유와 함께 unavailable이다. S0의
  `LocationId` 타입을 Git state/API에 복제하지 않는다.
- 테스트: default/round-trip/override, toggle, Local/generic Unsupported matrix, duplicate
  registration. S0가 이미 있으면 실제 SSH Remote matrix도 실행한다.
- 완료: G0 generic schema 변경 없이 Git을 enable/disable할 수 있다.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G1-01 모델, row identity와 read-only port

- 선행: G1-00
- 목표: repository/status/diff를 UI와 독립적으로 정의한다.
- 작업: RepositoryIdentity, metadata/worktree root, `RepoRelativePath`, Clean+7 statuses,
  detailed/collapsed precedence, branch/detached/unborn, `StatusRowId`, rename old/new metadata,
  `DiffTarget::{Staged,Unstaged,Combined}`, read-only discover/snapshot/diff trait을 만든다.
- 테스트: 전체 precedence, path traversal 거부, stable row identity/sort, target default,
  Combined order, redacted error.
- 완료: backend/process 없이 model unit test가 통과한다.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G1-02 FakeGitReadBackend와 fixture builder

- 선행: G1-01
- 목표: G1 read 경로만 결정적으로 주입한다.
- 작업: directory별 discover, repository별 snapshot/diff, call log, controlled gate,
  nth error/panic/cancel을 제공한다. Clean+7, nested, worktree, 10k fixture를 만들고 scenario
  v1의 optional `git.fixture`를 이 Fake builder에만 연결한다.
- 테스트: default/call ordering, latency 없이 block, error/panic/cancel, old/new rename.
- 완료: Fake에 mutation/network method가 없고 이후 G1 test가 실제 Git 설정을 쓰지 않는다.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G1-03 reducer, cache와 RefreshAll

- 선행: G1-02, G0-03
- 목표: I/O 없는 reducer로 generation/cache/refresh를 완성한다.
- 작업: disabled/not-repo/loading/ready/stale/error, exact-directory discovery cache,
  `BTreeMap<RepoRelativePath,...>`, request coalescing, `RefreshStatus`/`RefreshAll`, stale
  identity/generation 검증을 구현한다.
- 테스트: out-of-order, status→all upgrade, error-after-success, disable/enable, repo removed,
  unknown/duplicate terminal result.
- 완료: state transition test에서 backend를 직접 호출하지 않는다.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G1-04 discovery, nested repository와 negative cache

- 선행: G1-03
- 목표: 현재 local directory의 가장 가까운 repository를 놓치지 않는다.
- 작업: exact-directory positive/negative cache, parent/child/sibling 이동, outer→inner→outer,
  linked worktree, bare/removed repo를 구현한다. outer cache를 새 child에 상속하지 않는다.
- 테스트: **outer cache를 먼저 데운 뒤 inner 진입**, negative parent 뒤 nested child 진입,
  sibling, worktree, non-local 호출 0.
- 완료: cursor/render 0회, cache miss directory change discover 1회 규칙이 통과한다.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G1-05 file decoration과 Git theme

- 선행: G1-03~04, G0-04
- 목표: file/selection style과 column invariant를 보존하는 2셀 prefix를 표시한다.
- 작업: exact RepoRelativePath matching, Clean+7 prefix, 2셀 고정 예약,
  `show_file_status_prefix/show_untracked/show_ignored`, namespaced role, Unicode ellipsis를 구현한다.
- 테스트: 상태표/설정 조합, clean/hidden 예약 폭, cursor/marked 조합, no descendant roll-up,
  deleted synthetic 없음, 80/81/100/120/160 style-aware snapshot.
- 완료: Core FileEntry 변경 없이 멀티컬럼 시작 cell이 유지된다.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G1-06 branch/status summary

- 선행: G1-03, G1-05
- 목표: Core status item을 보존하며 branch와 count를 표시한다.
- 작업: branch/detached/unborn, U/M/A/D/R/? counts, styled full/compact item,
  loading/stale/error, disabled/nonrepo empty contribution을 구현한다.
- 테스트: 폭 축약, long Unicode branch, priority, zero counts, Core item 우선.
- 완료: `show_status_summary/show_branch`가 독립적으로 동작한다.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G1-07 Git Status View와 안정 선택

- 선행: G1-03, G1-06, G0-05
- 목표: 변경 목록을 별도 view로 탐색하고 refresh에도 cursor/marks를 예측 가능하게 보존한다.
- 작업: path 정렬, stable row identity, refresh intersection/fallback, navigation/mark,
  Enter/F3/Esc, F1~F12 고정 slot과 G1 F1/F3/F12 command, G2 F5~F11의 단계/상태별 disabled
  reason, loading/error/empty/nonrepo 화면을 구현한다. F12 Git Menu shell은 Refresh All,
  Diff Target, Close만 먼저 제공한다.
- 테스트: insert/delete/rename refresh 전후 cursor/marks, repository identity change,
  custom keymap, too-small, close Main-state preservation.
- 완료: 새 row는 자동 mark되지 않고 사라진 cursor는 nearest index 규칙을 따른다.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G1-08 Diff Viewer

- 선행: G1-07, v1 Viewer component
- 목표: staged/unstaged/combined/rename/delete diff를 worker에서 읽는다.
- 작업: row identity/old path/target validation, target default/menu, Combined section order,
  loading/result/error generation, Viewer scroll/search, diff style, 8 MiB size/deadline/cancel을 구현한다.
- 테스트: three targets, both-side default, rename/delete, stale/cancel, binary/too-large/error,
  8 MiB -1/정확히/+1, Unicode path, snapshot.
- 완료: diff callback/render backend 호출 0, Main/Status state 손상 0.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G1-09 trigger matrix, 오류 격리와 성능

- 선행: G1-03~08
- 목표: 이름 붙은 trigger 외 backend 호출을 금지하고 UI 응답성을 검증한다.
- 작업: directory changed, file operation, enable, `RefreshStatus`, `RefreshAll`을 연결한다.
  backend error/panic/index.lock를 nonfatal로 바꾸고 10k smoke를 추가한다.
- 테스트: disabled 전체 0; non-local backend/dynamic contribution 0과 command disabled reason;
  cached nonrepo cursor/mark/render/file-op/status/diff 0; directory/enable/Ctrl+R discover만
  계약대로; slow read 중 navigation/Core mutation 완료.
- 완료: nonrepo 규칙의 예외까지 call-count 표와 이름 있는 release smoke로 증명한다.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G1-10 production backend 비교 spike와 ADR

- 선행: G1-01~09
- 목표: 같은 contract로 gix/git2/CLI를 비교해 하나만 선택한다.
- 작업: Windows build/배포, nested/worktree, Clean+7, diff targets, deadline/cancel,
  license/size를 표로 남긴다. 선택하지 않은 dependency/code를 제거한다.
- 테스트: 동일 fixture 결과와 Windows CI 또는 명시적 미검증 위험.
- 완료: production dependency 하나, 승인 ADR, 미충족이면 `결정 필요` 상태.
- 진행: [ ] fixture [ ] 비교 [ ] ADR [ ] dependency 정리 [ ] progress 증거

## G1-11 production read backend

- 선행: G1-10
- 목표: 선택 backend가 Fake read contract를 충족한다.
- 작업: discover/snapshot/diff, nested/worktree/detached/unborn, Clean+7, rename policy,
  size/deadline/cancel, unavailable error를 구현한다.
- 테스트: TempDir, isolated config, Fake와 동일 conformance suite, Windows CI.
- 완료: global Git config/credential/network 의존 0, repository before/after 불변.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] conformance [ ] 공통 gate [ ] progress 증거

## G1-12 G1 통합 수용

- 선행: G1-00~11
- 목표: Local read-only Git release candidate를 승인한다.
- 작업: Main→Alt+G→Diff→Esc, RefreshAll, nested/worktree/nonrepo/error/10k,
  disable/enable, Local/generic Unsupported matrix, width/style snapshot, built-in English
  message와 Unicode path/branch/author 원문 보존을 실행하고 GIT-01~25를 연결한다. S0가 있으면
  실제 SSH Remote matrix도 필수이고, 없으면 해당 수동 증거를
  `N/A — S0 not implemented`로 기록한다.
- 수동: Windows Terminal의 소형/대형 throwaway repository와 빠른 탐색.
- 완료: G1 acceptance 전부 완료, repository 불변, G2 command reason 표시.
- 진행: [ ] 자동 시나리오 [ ] snapshot 검토 [ ] Windows [ ] 공통 gate [ ] progress 증거

---

# G2 — Local Git mutations

## G2-01 mutation port, Fake와 공통 MutationLease

- 선행: G1-12, Core mutation lease 계약
- 목표: read port와 분리한 plan/confirm/job/result 기반을 만든다.
- 작업: `GitMutationBackend`, `FakeGitMutationBackend`, operation id, repo-relative planner,
  availability reason, common MutationLease의 active-lease `Busy` 거부, cancel/partial result,
  terminal refresh 1회를 구현한다.
- 테스트: target escape/empty, active lease/confirm cancel=backend 0, Core op과 overlap 0,
  read lane 독립, partial failure, refresh exactly once.
- 완료: Fake read backend에는 mutation method가 없다.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G2-02 F5 Stage와 F6 Unstage

- 선행: G2-01
- 작업: marks-or-cursor target과 F5/F6를 구현한다. Stage는 worktree tracked
  Modified/Deleted/Renamed와 untracked만, Unstage는 index Added/Modified/Deleted/Renamed만
  허용한다. ignored/conflicted/해당 side delta 없음은 이유와 함께 거부하고 rename old/new 쌍을
  검증한다. unsupported/stale가 섞인 selection은 전체 preflight를 거부하고, 승인된 plan의
  실행 중 오류만 partial result로 보고한다.
- 테스트: single/multi/fallback/stale/cancel/partial, 각 index/worktree status 조합,
  rename pair, mixed-invalid backend 0, temp repo index before/after.
- 완료: unrelated worktree content 보존, terminal refresh 1회.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G2-03 F7 Commit

- 선행: G2-02
- 작업: multiline dialog, staged count, blank/no-staged guard, Ctrl+Enter/Esc, repository-local
  뒤 user-global 순서의 backend identity resolution, missing-identity 안내, branch/status refresh를
  구현한다. 선택한 identity를 author/committer 모두에 명시하고 system config/author·committer
  environment는 source로 쓰지 않는다. 앱 state/config에는 identity를 저장하지 않는다.
- 테스트: Unicode multiline, empty guards, missing name/email 각각 mutation 0, cancel/error/success,
  unstaged preservation. temp HOME global-only 성공, repo/global 값이 다를 때 repo 우선,
  resulting author/committer equality를 검사하고 system config와 identity env를 차단한다.
- 완료: 개발자 machine의 global user.name/email에 의존하지 않고 temp repo commit이 통과한다.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G2-04 Log와 commit detail

- 선행: G1-12, G0-03
- 작업: mutation port와 분리한 read-only `GitHistoryBackend`/`FakeGitHistoryBackend`에
  `log_page`, `commit_detail`, `commit_diff`를 추가하고 F10 Log의 paged query,
  hash/subject/author/fixed date/ref, detail/diff, loading/error를 구현한다.
- 테스트: empty/one/many/merge/detached/paging/Unicode fixed snapshot.
- 완료: query는 plugin-read lane에서만 실행된다.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G2-05 F11 Branch list/create

- 선행: G2-01
- 작업: deterministic sort/current marker, create dialog, Git ref-name/duplicate validation을 구현한다.
- 테스트: unborn, valid/invalid/Unicode/duplicate/backend error/cancel.
- 완료: create 뒤 list/status terminal refresh 각 계약 횟수만 실행.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G2-06 Checkout과 dirty preflight

- 선행: G2-05
- 작업: dirty snapshot, 영향/거부 이유, safe checkout, 사라진 Main cursor fallback을 구현한다.
- 테스트: clean, dirty blocked, conflict, cancel, selected file removed, Core op lease collision.
- 완료: force checkout과 branch delete는 제공하지 않는다.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G2-07 F9 Stash current changes

- 선행: G2-01
- 작업: optional message, tracked staged+unstaged만 포함, untracked/ignored 제외,
  no-tracked-change guard, create operation/result/refresh만 구현한다.
- 테스트: no changes/untracked-only zero-call, staged/unstaged/both, ignored preservation,
  Unicode message, cancel/failure/success.
- 완료: list/apply/pop/drop 없이 current stash contract가 독립 통과한다.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G2-08 Stash list/apply/pop/drop

- 선행: G2-07
- 작업: stable stash identity/list, apply/pop, conflict result, pop failure preserves stash,
  staged-state restore, drop irreversible confirm과 F12 Git Menu 진입을 구현한다.
- 테스트: empty/list, apply conflict, pop failure preservation, drop cancel/success.
- 완료: 각 terminal result 후 status/list refresh 횟수가 고정된다.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G2-09 F8 Discard tracked worktree changes

- 선행: G2-01~02
- 작업: tracked worktree Modified/Deleted/Renamed을 index로 복원하고 staged는 보존한다.
  index-only/untracked/conflict reason, selection all-or-reject preflight, absolute/relative target
  안내, irreversible confirm, 실행 중 partial result를 구현한다.
- 테스트: Esc no-op, modified/deleted/renamed, both sides preserves index, index-only/untracked/
  conflict/mixed-selection backend 0, 실행 중 partial failure.
- 완료: `git revert`로 표기/구현하지 않고 unsupported는 이유를 표시한다.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G2-10 G2 통합 수용

- 선행: G2-01~09
- 작업: stage→commit→branch→checkout→stash→apply, cancel/error/partial, Core op serialization,
  blocked operation 중 navigation/resize, before/after를 disposable repo에서 검증한다.
- 완료: mutation dialog/confirm/result English와 Unicode branch/author/commit message 원문 보존을
  포함한 LOCAL-01~13, 공통 gate, Windows throwaway repo 증거, network 호출 0.
- 진행: [ ] 자동 흐름 [ ] lease 검증 [ ] Windows [ ] 공통 gate [ ] progress 증거

---

# G3 — Git remote operations

## G3-00 auth/transport/conflict 설계 게이트

- 선행: G2-10
- 목표: raw-secret 없는 integration, host-key/TLS, pull strategy, phase/cancel, cleanup을 승인한다.
- 필수 결정: OS credential helper/SSH agent만 사용, raw secret callback 금지, known_hosts 정책,
  force push 기본 금지, `Queued→Resolving/Auth→Transferring→Applying→Terminal`, auth 뒤/첫
  Transferring 전 lease 획득과 Terminal 반환, interrupted clone cleanup.
- 완료: threat/recovery/backend capability/test fixture 문서의 미결정 0.
- 진행: [ ] threat model [ ] auth 결정 [ ] cancel/recovery [ ] backend 검증 [ ] 승인 증거

## G3-01 remote metadata와 cached ahead/behind

- 선행: G3-00
- 작업: network 없이 remote/default/tracking, detached/no-upstream, cached `↑/↓`, named refresh만 구현한다.
- 테스트: no/multiple remote, no tracking, detached, fixed graph, render/cursor graph call 0.
- 완료: GITNET-02.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G3-02 authentication/transport/redaction 기반

- 선행: G3-00
- 목표: 첫 network operation보다 먼저 OS helper/agent와 transport 안전 경계를 구현한다.
- 작업: `GitTransportBackend`, `FakeGitTransportBackend`, helper/agent availability,
  host-key fingerprint decision, TLS error, phase/progress/cancel, redaction scanner를 구현한다.
  M2 JobControl/LaneSender를 재사용해 기본 capacity 16의 single Git Transport lane을 만들고,
  G3 operation이 Queued 이상일 때 다른 G3 mutation submit은 `Busy`로 거부한다.
- 테스트: raw secret API/state/config/log/snapshot 0, helper/agent unavailable, prompt cancel,
  unknown/changed host key, injected capacity, 동시 G3 submit, deadline, phase별 cancel,
  panic/error/shutdown join.
- 완료: 실제 fetch 없이 GITNET-03/04/05 기반을 Fake로 증명한다.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 보안 scan [ ] 공통 gate [ ] progress 증거

## G3-03 Fetch

- 선행: G3-01, G3-02
- 작업: Git Menu Fetch command와 approved transport의 timeout/cancel/progress/error,
  auth 뒤 첫 Transferring 직전 non-blocking mutation lease, terminal refresh를 구현한다.
- 테스트: isolated remote, offline, auth/host-key/TLS failure, queued/auth/transfer cancel,
  active lease에서 auth preflight 허용/transfer와 ref mutation 0, progress throttle,
  interrupted worker.
- 완료: 실패 후 worktree/index 불변, credential 비노출. GITNET-06.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G3-04 Pull

- 선행: G3-03
- 작업: Git Menu Pull command, 승인 strategy의 fetch+integrate, dirty/diverged preflight,
  auth 뒤 첫 Transferring 직전 non-blocking mutation lease, conflict transition,
  Applying phase의 finishing UX를 구현한다.
- 테스트: fast-forward/up-to-date/dirty/diverged/conflict, pre-apply cancel, applying terminal recovery.
- 완료: 암묵적 force/reset 없음. GITNET-07.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G3-05 Push

- 선행: G3-03
- 작업: Git Menu Push command, remote/ref/upstream 표시, rejection, phase/cancel과 tracking ref
  update를 포함해 auth 뒤 첫 Transferring 직전 얻고 Terminal까지 보유하는 non-blocking
  mutation lease를 구현한다.
- 테스트: success/no-upstream/non-fast-forward/auth/cancel/finishing.
- 완료: 승인 없으면 force push API/UI 없음. GITNET-08.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G3-06 Remote add/edit/remove

- 선행: G3-03
- 작업: remote name/URL validation, add/edit/remove confirmation, credential-bearing URL 거부,
  default/tracking 영향, Git Menu Remote Manage command, config write 전 non-blocking mutation
  lease와 refresh를 구현한다.
- 테스트: duplicate/invalid name, malformed/credential URL, cancel, current upstream 영향,
  backend partial failure와 redaction.
- 완료: 다른 remote/ref와 config secret을 손상하지 않음. GITNET-09.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G3-07 Clone

- 선행: G3-03
- 작업: empty target validation, plugin-created partial directory identity, phase/progress/cancel,
  Main F12 `Git > Clone…` command를 구현한다. resolve/auth 성공 뒤 partial target 생성과 첫
  Transferring 직전에 non-blocking mutation lease를 얻고 failure cleanup과 completion 후 Local
  Location 진입 제안을 구현한다.
- 테스트: nonempty target, auth 실패 시 target 생성 0, active lease에서 auth preflight 허용 뒤
  target/transfer 0, queued/transfer cancel, network failure cleanup, preexisting target preservation,
  symlink/containment, success.
- 완료: plugin이 이번 OperationId로 만든 partial target만 정리하고 사용자 기존 파일 삭제 0.
  GITNET-10.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G3-08 Conflict workflow

- 선행: G3-04, G2 local operations
- 작업: conflict row/diff와 context F5 `Mark Resolved`, resolution check/continue/abort/restart
  recovery state machine을 구현한다. Mark Resolved는 active unmerged cursor/marks만 허용하고
  deletion/binary를 별도 확인하며 Core Local lane의 공통 lease 아래 index를 갱신한다. Git Menu는
  conflict state에서 Continue/Abort를 표시하고 Continue는 unmerged 0일 때만 enabled이며 local
  apply 전 non-blocking mutation lease를 얻는다.
- 테스트: text/binary/delete-modify/rename, 일반 Status F5 conflict 차단, Mark Resolved
  single/multi/mixed-stale, cancel/active lease backend 0, partial resolution, Continue
  disabled→enabled, restart, abort.
- 완료: 자동 내용 병합 편집기는 별도 승인 없이는 제공하지 않는다. GITNET-11.
- 진행: [ ] 실패 테스트 [ ] 구현 [ ] 카드 테스트 [ ] 공통 gate [ ] progress 증거

## G3-09 G3 통합 수용

- 선행: G3-01~08
- 작업: isolated two-clone fetch/pull/push/conflict, offline/cancel/auth redaction,
  clone cleanup, settings round-trip, Windows helper/agent/host-key를 검증한다.
- 완료: auth/network/conflict English copy와 remote/ref/path/user input 원문 보존을 포함한
  GITNET-01~12, 보안 checklist, 공통 gate, Windows 증거.
- 진행: [ ] 자동 흐름 [ ] 보안 scan [ ] Windows [ ] 공통 gate [ ] progress 증거
