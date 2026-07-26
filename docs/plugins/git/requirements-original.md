# 사용자 제공 Git 요구사항 추적본

이 문서는 사용자가 제공한 `Mdir Git Integration Plugin 계획` 원문의 요구를 잃지 않고
현재 실행 문서로 추적하기 위한 색인이다. 원문 전체는 저장소에 함께 보존하며 이 파일은
원문을 대체하지 않는다.

- 원문: [`requirements-source.md`](requirements-source.md)
- SHA-256: `98ef4ed1749c1947856fb6a13c4a7fbbf77f7cb179e35ed656d9196686398f15`
- 원문 제목: `# Mdir Git Integration Plugin 계획`
- 원문 구성: 1~51절

## 원문 → 현재 계약/카드/수용 추적

| 원문 절 | 원문 주제 | 현재 계약/아키텍처 | 작업 카드 | 수용 ID |
|---|---|---|---|---|
| 1~5, 47~48, 51 | built-in plugin, Core 분리, 향후 확장 | `01` §1, `02` §1~4/§10 | G0-01~06, G1-00 | PLUG-01~11, GIT-01 |
| 6, 31~33 | parent/nested/worktree/nonrepo detection | `01` §3, `02` §7 | G1-03~04, G1-11 | GIT-03~06 |
| 7~9, 35 | Clean+7 status, prefix, theme, ignored | `01` §4, `02` §7 | G1-01~05 | GIT-07~12 |
| 10, 13, 36 | enable/display 설정, Alt+G/key override | `01` §2/§5 | G1-00, G1-05~07 | GIT-01, GIT-10, GIT-14 |
| 11~12 | branch/status bar, ahead/behind | `01` §5 | G1-06, G3-01 | GIT-08/13, GITNET-02 |
| 14~16 | Git Status View, context F-key, Diff | `01` §5~6, `02` §4/§8 | G1-07~08 | GIT-14~17 |
| 17~23 | Stage/Unstage/Commit/Log/Branch/Stash/Discard | `01` §8 | G2-01~10 | LOCAL-01~13 |
| 24~27 | refresh/cache/background/performance | `01` §3/§7/§10, `02` §5~7 | G0-03, G1-03/09 | PLUG-06/07, GIT-18/19/24 |
| 28~29 | gix/git2/CLI backend 후보 | `02` §8 | G1-10~11 | GIT-22~23 |
| 30 | Git 오류가 Core를 종료하지 않음 | `01` §10, `02` §3 | G0-02, G1-09 | PLUG-04/05, GIT-20 |
| 34 | submodule 후순위 | `01` §3/§11 | 후속 범위 | 단계 gate에서 제외 |
| 37~43 | fixture/unit/Fake/snapshot/결정성 | `04` §1~10 | 각 카드 실패 테스트 | GIT-09/11/12/21 |
| 44 | Phase 1 read-only 범위 | `01` §1 | G1-00~12 | GIT-01~25 |
| 45 | Phase 2 local mutation 범위 | `01` §8 | G2-01~10 | LOCAL-01~13 |
| 46 | Phase 3 remote/auth/conflict | `01` §9 | G3-00~09 | GITNET-01~12 |
| 49 | Phase 1 완료 기준 | `05` G0/G1 표 | G1-12 | PLUG-01~11, GIT-01~25 |
| 50 | Fake/UI 우선 개발 순서 | `03` 전체 | G0 → G1 Fake → spike → real | 단계 승인 규칙 |

파일명 약어: `01`은 [`01-product-contract.md`](01-product-contract.md), `02`는
[`02-architecture.md`](02-architecture.md), `03`은 [`03-task-cards.md`](03-task-cards.md),
`04`는 [`04-test-plan.md`](04-test-plan.md), `05`는
[`05-acceptance-matrix.md`](05-acceptance-matrix.md)다.

## 원문 표현을 그대로 구현하지 않은 결정

| 원문 제안 | 확정 변경 | 이유/검증 |
|---|---|---|
| callback이 값을 직접 반환하고 오류 경계가 없음 | 모든 callback `Result`, manager `catch_unwind`, Faulted 재생성 | PLUG-04/05 |
| Git/nonrepo에서는 “아무 작업도 하지 않음” | cached nonrepo status/diff는 0; directory/enable/RefreshAll만 named discover 허용 | GIT-03/18 |
| `HashMap<PathBuf, ...>` cache | `BTreeMap<RepoRelativePath, ...>` | 결정성, worktree 탈출 방지; GIT-07 |
| `shortcut = "Alt+G"`를 Git config에 저장 | CommandRegistry/keymap만 단일 원본 | GIT-01 |
| `show_status`, `status_prefix` | `show_status_summary`, `show_file_status_prefix` | summary와 row prefix 의미 분리 |
| 하나의 `FakeGitBackend` | read/mutation/transport Fake 분리 | 단계별 capability 누수 방지; LOCAL-01 |
| Branch Delete | G2 이후로 연기 | 원문도 dirty/destructive 위험을 명시하며 현 카드에 복구 계약이 없음 |
| F8 `Revert` | `Discard local changes` | `git revert <commit>`과 혼동 방지; LOCAL-11 |
| 단일 background Git worker 예시 | Core mutation lane과 별도 plugin-read lane, G2는 공통 mutation lease | UI/파일 작업 상호 blocking 방지; PLUG-06, LOCAL-03 |
| Fetch 뒤에 Authentication 구현 가능 | 인증/전송/redaction 기반을 Fetch 전에 완료 | raw secret/host-key/cancel 안전 gate; GITNET-03~06 |
| Clean과 “8 statuses”라는 혼용 가능성 | Clean + 변경 상태 7개로 통일 | enum 전체 8개를 정확히 표현; GIT-09 |

원문 요구를 삭제하거나 단계 밖으로 옮길 때는 이 표, 제품 계약, 카드, 수용 ID를 한 변경에서
함께 갱신한다.
