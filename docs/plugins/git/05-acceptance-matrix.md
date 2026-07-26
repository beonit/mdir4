# Git built-in 수용 기준 추적표

카드별 진행은 [`03-task-cards.md`](03-task-cards.md)의 inline checklist에, test/CI/수동
증거와 단계 상태는
[`../../implementation-plan/progress.md`](../../implementation-plan/progress.md)에 기록한다.
이 표의 기본 상태는 `미착수`, 기본 `Evidence`는 `—`다. 단계 승인 시에는
`상태`와 `Evidence`를 함께 갱신하며, 증거가 없으면 완료로 바꾸지 않는다.

Git와 SSH Remote는 서로 독립된 단계다. GIT-02 자동 검증은 항상 generic
`Unsupported` fixture를 사용한다. 실제 SSH Remote 수동 증거는 S0가 구현된
경우에만 필요하다. S0 미구현 시 Evidence에 `N/A — S0 not implemented`를
기록해도 G1 승인을 차단하지 않는다.

## G0 — generic host

| ID | 수용 기준 | 카드 | 자동 증거 | 상태 | Evidence |
|---|---|---|---|---|---|
| PLUG-01 | Core model/layout이 Git 타입/backend를 모름 | G0-01~05 | dependency/source boundary | 미착수 | — |
| PLUG-02 | disabled plugin callback/job/contribution 0 | G0-02,06 | call-count contract | 미착수 | — |
| PLUG-03 | id/order/key collision이 결정적이고 이유를 표시 | G0-01,02,04 | ordering/collision tests | 미착수 | — |
| PLUG-04 | callback Err/panic이 Core/다른 plugin을 종료하지 않음 | G0-02 | fault scenarios | 미착수 | — |
| PLUG-05 | fault re-enable은 새 instance/generation | G0-02 | late-result/re-enable | 미착수 | — |
| PLUG-06 | capacity 16 plugin read lane이 Core mutation과 분리되고 full coalesce/Busy가 non-blocking | G0-03 | injected-capacity/blocked-lane | 미착수 | — |
| PLUG-07 | 공통 cancel/deadline/OperationId 재사용과 stale/shutdown이 terminal 1회/join 보장 | G0-03 | worker interleavings/type boundary | 미착수 | — |
| PLUG-08 | reserved cell/extensible style fallback/availability reason/view ownership | G0-04,05 | FakePlugin snapshots | 미착수 | — |
| PLUG-09 | generic config/keymap/toggle round-trip | G0-06 | config scenarios | 미착수 | — |
| PLUG-10 | G0 production에 Git module/config/default command 없음 | G0-06 | source/tree assertion | 미착수 | — |
| PLUG-11 | generic host built-in label/Help/error는 영어이고 plugin 제공 text의 원문을 보존 | G0-04~06 | message/snapshot/source scan | 미착수 | — |

## G1 — Local read-only Git

| ID | 수용 기준 | 카드 | 자동 증거 | 수동 증거 | 상태 | Evidence |
|---|---|---|---|---|---|---|
| GIT-01 | Git 등록/설정/key 기본값은 G1-00에서만 추가 | G1-00 | config/registry tests | 설정 화면 | 미착수 | — |
| GIT-02 | generic Unsupported/non-local context에서 discover/job/dynamic contribution 0이고 정적 command는 이유와 함께 disabled | G0-01,G1-00,04,12 | path-context call matrix | SSH Remote 화면 | 미착수 | — |
| GIT-03 | cached nonrepo는 named discovery 외 status/diff 0 | G1-03,04,09 | exact call matrix | Alt+G message | 미착수 | — |
| GIT-04 | 부모 repository 자동 감지 | G1-04,11 | fake+real contract | 실제 repo | 미착수 | — |
| GIT-05 | warm outer/negative cache/submodule 진입에서도 가장 가까운 repo 선택 | G1-04,11 | nested/submodule regression | 실제 nested | 미착수 | — |
| GIT-06 | metadata root/worktree root 분리 | G1-01,04,11 | worktree contract | worktree | 미착수 | — |
| GIT-07 | cache key가 normalized RepoRelativePath | G1-01,03 | type/path tests | 없음 | 미착수 | — |
| GIT-08 | branch/detached/unborn 표시 | G1-01,06,11 | model/snapshot | 없음 | 미착수 | — |
| GIT-09 | Clean + 변경 상태 7개 mapping/precedence | G1-01,02,05,11 | mapping/conformance | 없음 | 미착수 | — |
| GIT-10 | ignored/untracked 설정은 파일을 숨기지 않음 | G1-05 | setting scenarios | 없음 | 미착수 | — |
| GIT-11 | 2셀 prefix가 file/cursor/marked style을 보존 | G1-05 | cell/style asserts | reference 검토 | 미착수 | — |
| GIT-12 | 멀티컬럼/Unicode width invariant | G1-05 | width snapshot matrix | 80×25 | 미착수 | — |
| GIT-13 | styled branch/count summary가 Core item을 보존 | G1-06 | width tests | resize | 미착수 | — |
| GIT-14 | Git Status View가 repo 전체를 표시하고 F1~F12 context/메뉴/navigation/close를 보존 | G1-07,08 | scope/key/menu scenarios | 키 감각 | 미착수 | — |
| GIT-15 | refresh 후 stable row cursor/mark 규칙 | G1-07 | insert/delete/rename tests | 없음 | 미착수 | — |
| GIT-16 | Staged/Unstaged/Combined 기본·명시 target | G1-01,08 | diff target suite | 실제 diff | 미착수 | — |
| GIT-17 | rename/delete/binary/8 MiB 경계/diff error nonfatal | G1-08 | error/boundary snapshots | 없음 | 미착수 | — |
| GIT-18 | RefreshStatus/RefreshAll/trigger/coalescing 규칙 | G1-03,09 | call matrix | Ctrl+R | 미착수 | — |
| GIT-19 | backend 지연 중 navigation/Core mutation 응답 | G1-09 | blocked lane scenario | 대형 repo | 미착수 | — |
| GIT-20 | backend 오류/panic 뒤 파일 탐색 가능 | G1-09 | fault scenarios | unavailable | 미착수 | — |
| GIT-21 | Fake read backend로 G1 전체 CI 검증 | G1-02~09 | Linux/Windows CI | 없음 | 미착수 | — |
| GIT-22 | 실제 backend 선택 근거/ADR와 dependency 하나 | G1-10 | artifact/source check | Windows build | 미착수 | — |
| GIT-23 | real backend가 Fake read contract를 충족하고 repo 불변 | G1-11,12 | conformance/before-after | throwaway repo | 미착수 | — |
| GIT-24 | 10,000 status에서도 입력 목표 유지 | G1-09,12 | named release smoke | 체감 확인 | 미착수 | — |
| GIT-25 | Status/Diff/availability built-in copy는 영어이고 path/branch/author를 원문 보존 | G1-05~08,12 | message/snapshot matrix | Windows walkthrough | 미착수 | — |

## G2 — local mutations

| ID | 수용 기준 | 카드 | 자동 증거 | 수동 증거 | 상태 | Evidence |
|---|---|---|---|---|---|---|
| LOCAL-01 | read/mutation Fake와 port가 분리 | G2-01 | API/source boundary | 없음 | 미착수 | — |
| LOCAL-02 | target이 worktree를 벗어나지 않음 | G2-01 | planner properties | 없음 | 미착수 | — |
| LOCAL-03 | Core op/Git mutation이 공통 lease로 직렬화 | G2-01,10 | overlap=0 scenario | 동시 작업 | 미착수 | — |
| LOCAL-04 | active lease는 Busy/no-op, confirmation 취소는 no-op이며 backend mutation call 0 | G2-01~09 | cancel scenarios | 없음 | 미착수 | — |
| LOCAL-05 | Stage/Unstage side별 status·rename 정책, mixed-invalid 전체 거부, single/multi/partial | G2-02 | fake+real policy matrix | throwaway repo | 미착수 | — |
| LOCAL-06 | blank/no-staged/missing-identity Commit 금지, repo→user-global 우선순위와 author/committer 고정, ambient env/system 차단, unstaged 보존 | G2-03 | isolated-config dialog/repo tests | 없음 | 미착수 | — |
| LOCAL-07 | 별도 history read port의 F10 Log/detail/diff가 결정적으로 표시 | G2-04 | fake/history snapshots | 실제 log | 미착수 | — |
| LOCAL-08 | Branch create validation과 dirty checkout 안전 | G2-05,06 | ref/dirty scenarios | throwaway repo | 미착수 | — |
| LOCAL-09 | F9 Stash current가 tracked staged+unstaged만 보관하고 untracked/ignored를 보존 | G2-07 | fake+real | throwaway repo | 미착수 | — |
| LOCAL-10 | F12 Stash list/apply/pop/drop이 staged state와 실패 시 stash를 보존 | G2-08 | state scenarios | throwaway repo | 미착수 | — |
| LOCAL-11 | F8 Discard는 tracked worktree만 index로 복원하고 unsupported selection은 calls 0 | G2-09 | cancel/confirm/policy | 문구 검토 | 미착수 | — |
| LOCAL-12 | terminal refresh 1회, UI 응답, G2 network 0 | G2-01~10 | count/network-deny | resize/탐색 | 미착수 | — |
| LOCAL-13 | mutation dialog/confirm/result built-in copy는 영어이고 branch/author/commit message를 원문 보존 | G2-02~10 | message/snapshot matrix | Windows walkthrough | 미착수 | — |

## G3 — auth/network/conflict

| ID | 수용 기준 | 카드 | 자동 증거 | 수동 증거 | 상태 | Evidence |
|---|---|---|---|---|---|---|
| GITNET-01 | auth/transport/conflict gate의 미결정 0 | G3-00 | doc/ADR check | 보안 검토 | 미착수 | — |
| GITNET-02 | ahead/behind를 render/cursor에서 계산하지 않음 | G3-01 | call count | 없음 | 미착수 | — |
| GITNET-03 | network 전에 helper/agent/host-key/redaction 및 bounded Git Transport lane 완성 | G3-02 | Fake transport/worker suite | helper/agent | 미착수 | — |
| GITNET-04 | raw secret가 API/state/config/log/snapshot에 없음 | G3-00,02 | source/redaction scan | credential 시험 | 미착수 | — |
| GITNET-05 | unknown/changed host key 자동 승인 0 | G3-02 | fingerprint scenarios | SSH host | 미착수 | — |
| GITNET-06 | Git Menu Fetch timeout/phase cancel/error, auth 뒤 transfer 전 lease와 Busy 격리 | G3-03 | isolated remote+lease/call count | offline | 미착수 | — |
| GITNET-07 | Pull dirty/diverged/conflict/finishing 정책 | G3-04 | two-clone | throwaway remote | 미착수 | — |
| GITNET-08 | Push rejection/no-upstream, 기본 force 없음 | G3-05 | isolated remote | throwaway remote | 미착수 | — |
| GITNET-09 | remote add/edit/remove가 name/URL/upstream을 안전하게 검증 | G3-06 | validation/redaction tests | throwaway remote | 미착수 | — |
| GITNET-10 | Clone 실패/취소 cleanup이 기존 대상을 보존 | G3-07 | cleanup/containment tests | 없음 | 미착수 | — |
| GITNET-11 | command/menu 진입, conflict-context Mark Resolved, continue/abort/restart, mutation lease와 전체 Windows 흐름 | G3-03~09 | state/two-clone/index/lease | Windows | 미착수 | — |
| GITNET-12 | auth/network/conflict built-in copy는 영어이고 remote/ref/path/user input을 원문 보존 | G3-02~09 | message/redaction/snapshot matrix | Windows walkthrough | 미착수 | — |

## 단계 승인 규칙

- G0: PLUG-01~11. FakePlugin만으로 증명하고 Git production code는 없어야 한다.
- G1: GIT-01~25. Local read-only이며 G2 command는 availability reason과 함께 disabled다.
- G2: LOCAL-01~13. disposable repository, shared mutation lease, network-deny 증거가 필요하다.
- G3: GITNET-01~12. 인증 기반(G3-02)보다 fetch가 먼저 구현되면 단계 실패다.
