# Mdir4 Git built-in 확장 계획

이 폴더는 사용자 제공 Git 계획을 Mdir4의 v1 이후 실행 트랙으로 재정렬한 문서 묶음이다.
아직 Git production 구현은 시작하지 않는다. v1 `R1`이 끝난 뒤 `G0 → G1 → G2 → G3`
순서로 진행한다.

## 읽는 순서

1. [`01-product-contract.md`](01-product-contract.md) — 확정 사용자 동작과 안전 정책
2. [`ADR-004`](../../architecture/adr-004-built-in-plugin-boundary.md),
   [`ADR-005`](../../architecture/adr-005-background-work-lanes.md) — 승인된 plugin/lane 계약
3. [`02-architecture.md`](02-architecture.md) — host/plugin/backend/worker 경계
4. [`03-task-cards.md`](03-task-cards.md) — 선행 관계가 있는 작은 구현 카드
5. [`04-test-plan.md`](04-test-plan.md) — Fake, fixture, scenario, real contract
6. [`05-acceptance-matrix.md`](05-acceptance-matrix.md) — 단계별 완료 증거
7. [`requirements-original.md`](requirements-original.md) — 사용자 원문→현재 계약 추적표
8. [`requirements-source.md`](requirements-source.md) — 보존한 사용자 원문

원문 SHA-256은
`98ef4ed1749c1947856fb6a13c4a7fbbf77f7cb179e35ed656d9196686398f15`다. 원문과 canonical
계약이 다르면 위 순서대로 제품 계약이 우선하고 추적표에 차이를 남긴다.

프로젝트 전체 순서는 [`../../README.md`](../../README.md), 현재 상태는
[`../../implementation-plan/progress.md`](../../implementation-plan/progress.md), 에이전트 절차는
[`../../implementation-plan/06-agent-runbook.md`](../../implementation-plan/06-agent-runbook.md)를
따른다.

## 실행 순서

```text
R1 v1.0
  │
  ▼
G0 Generic Host
  │  FakePlugin만 사용; Git production code 없음
  ▼
G1 Local Read-only Git
  │  Fake read → backend spike/ADR → real read
  ▼
G2 Local Mutations
  │  Core file operation과 공통 mutation lease
  ▼
G3 Auth/Transport → Fetch/Pull/Push → Remote management/Clone → Conflict
```

| 단계 | 사용자 가치 | 허용되는 side effect | 종료 기준 |
|---|---|---|---|
| G0 | extension host 준비 | 없음 | PLUG-01~11 |
| G1 | Local repository/status/diff | 없음 | GIT-01~25 |
| G2 | stage/commit/branch/stash/discard | 로컬 repository 변경 | LOCAL-01~13 |
| G3 | remote Git 작업 | network+repository 변경 | GITNET-01~12 |

Git은 `Local Location`에만 적용한다. SSH Remote 등 다른 Location에서는 track 구현 순서와
상관없이 discovery/backend job과 동적 decoration/status/view contribution이 0이다. 전역
CommandRegistry에 등록된 Git command definition은 남되 `Local locations only.` 이유로
disabled다.

## 재검토에서 고정한 핵심 결정

| 쟁점 | 확정 결정 |
|---|---|
| disabled/nonrepo 호출 | disabled는 전체 0; cached nonrepo status/diff는 0; directory/enable/`RefreshAll`만 named discover |
| callback 실패 | `Result<_, PluginError>` + manager `catch_unwind`; faulted plugin만 격리·재생성 |
| worker | G1 plugin-read, G2 Core Local, G3 Git Transport lane을 분리하고 mutation은 공통 lease |
| generic/Git 설정 | G0는 generic config만, Git factory/config/default key는 G1-00 |
| command/view/style | namespaced id/role, availability reason, reserved cell, owner-checked view lifecycle |
| UI language | built-in label/Help/dialog/error는 영어; path/branch/author/message/remote/ref는 원문 보존 |
| repository cache | exact directory discovery cache와 `BTreeMap<RepoRelativePath,...>`; warm outer 뒤 inner 재탐색 |
| status selection | stable row identity, refresh intersection/fallback 규칙 |
| diff | Staged/Unstaged/Combined와 rename old/new metadata를 명시 |
| Fake | read/mutation/transport capability별 분리 |
| G2 직렬화 | Core file operation과 Git mutation은 common mutation lease |
| G3 순서 | auth/transport/redaction/host-key/cancel 기반이 Fetch보다 먼저 |
| credential | OS helper/agent 경계만 사용하고 raw secret은 app API/state/config/log에 없음 |

background lane topology는
[`ADR-005`](../../architecture/adr-005-background-work-lanes.md)로 확정됐다. G0-03/G2-01은
공통 primitive와 local lease를, G3-02는 bounded Git Transport lane을 구현한다. 측정 전
pool/Tokio 도입은 범위 밖이다.

## 범위 제어

- G0에서 Archive/SFTP/외부 ABI를 함께 만들지 않는다.
- G1 완료 전 Stage/Commit 코드나 mutation method를 read backend에 넣지 않는다.
- G2 완료 전 network/credential/remote tracking 계산을 넣지 않는다.
- G3-00/02가 끝나기 전 실제 network operation을 구현하지 않는다.
- branch delete, submodule 전용 상태, external SDK는 후속 범위다.
- 카드 완료 시 카드 checkbox, acceptance ID, 최상위 progress 증거를 함께 갱신한다.
