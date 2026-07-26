# Git built-in 제품 계약

이 문서는 `R1` 이후 built-in Git 확장의 사용자 동작과 안전 경계를 고정한다. v1 파일
관리자 계약과 충돌하면 v1 계약이 우선한다. Git 구현은 `G0 → G1 → G2 → G3` 순서이며,
앞 단계의 금지 범위를 뒤 단계 코드로 미리 우회하지 않는다.

Git built-in의 label, Help, dialog와 error copy도 Core의 영어 UI 계약을 따른다. repository
path, branch, author와 commit message는 번역하지 않고 안전한 원문 display를 유지한다.

## 1. 단계와 기본 범위

| 단계 | 제공 기능 | 저장소/네트워크 변경 |
|---|---|---|
| G0 | Git을 모르는 built-in extension host | 없음 |
| G1 | Local Location의 repository/status/diff | 없음 |
| G2 | stage/commit/branch/stash/discard | 로컬 저장소 변경 |
| G3 | fetch/pull/push/clone/remote/conflict | 네트워크와 로컬 저장소 변경 |

- G0는 정적으로 링크된 built-in만 지원한다. 외부 plugin ABI, 동적 로딩, sandbox는 제외다.
- G1은 `Local Location`에서만 동작한다. SSH Remote를 포함한 다른 Location에서는
  discovery/backend job과 동적 decoration/status/view contribution을 만들지 않는다. 전역
  CommandRegistry의 Git command definition은 유지하되 `Local locations only.` 이유로
  unavailable이다. Git과 SSH Remote 트랙의 구현 순서와 무관하게 이 규칙을 지킨다.
- G1의 backend는 read-only다. mutation method를 미리 추가하지 않는다.
- G3 production 코드는 인증·전송·취소·복구 계약이 승인된 뒤에만 시작한다.

## 2. 설정과 명령의 단일 원본

G0의 설정 schema는 generic `[plugins]` enable/config 보관만 제공한다. 아래 Git section과
Git instance 등록은 `G1-00`에서 처음 추가한다.

```toml
[plugins.git]
enabled = true
show_status_summary = true
show_file_status_prefix = true
show_branch = true
show_untracked = true
show_ignored = false
refresh_on_directory_change = true
refresh_on_file_operation = true

[keymap]
"plugin.git.open_status" = "Alt+G"
```

- Git 설정에 shortcut 문자열을 중복 저장하지 않는다. 기본 키와 사용자 override,
  function bar 표시는 `CommandRegistry`가 단일 원본이다.
- `enabled = false`이면 manager가 Git callback 자체를 호출하지 않는다. discovery, backend
  생성, worker job, command/view, decoration/status contribution은 모두 0이다.
- runtime disable은 pending request를 취소하고 generation을 올린 뒤 Git UI를 즉시
  제거한다. 취소를 무시한 늦은 결과도 적용하지 않는다.
- 설정 저장 실패는 현재 세션 toggle을 되돌리지 않고 비치명 경고를 표시한다.
- unknown plugin/config key는 round-trip 보존하고 경고한다.

## 3. repository discovery와 non-repository 규칙

`discover(path)`는 해당 local directory에서 부모 방향으로 탐색해 가장 가까운 repository를
반환한다. metadata root와 worktree root는 분리한다.

- directory별 discovery cache key는 정규화한 절대 local directory다. 값은 repository
  identity 또는 negative result다.
- 같은 directory의 유효한 cache는 재사용할 수 있다. 바깥 repository가 cache되어 있어도
  처음 방문한 하위 directory는 nested repository일 수 있으므로 반드시 그 directory를
  discover한다. 부모의 positive/negative 결과를 자식에 전파하지 않는다.
- nested repository에 들어가면 내부 identity, 나오면 바깥 identity로 교체한다.
- linked worktree의 metadata root와 worktree root를 혼동하지 않는다.
- bare repository는 Main 통합 대상이 아니며 명시적 unsupported 상태다.
- parent repository 목록에서 submodule은 G1의 전용 aggregate 상태 없이 일반 directory다.
  그 directory에 Enter하면 exact-directory discovery를 실행하고 gitfile이 가리키는 내부
  repository identity를 활성화한다.

backend 호출 규칙은 다음과 같다.

| 상태/이벤트 | 허용 호출 |
|---|---|
| plugin disabled | 모든 callback/job/backend 호출 0 |
| non-local Location | discovery/status/diff와 동적 decoration/status/view contribution 0; 정적 command는 disabled |
| cached non-repository에서 cursor/mark/render/file-op | 0 |
| local directory changed | exact-directory cache miss/invalid이면 discover 1 |
| non-repository에서 `Ctrl+R` | exact-directory cache를 무효화하고 discover 1 |
| repository에서 file operation 완료 | snapshot 1, discovery 0 |
| repository에서 `Ctrl+R` | `RefreshAll`: discovery 1 후 snapshot 1 |

따라서 “non-repository fast path 0회”는 status/diff와 이름 없는 자동 작업을 뜻한다.
명시된 directory-change/enable/`RefreshAll` discovery는 예외다. non-repository의
`Alt+G`는 backend를 호출하지 않고 `Not a Git repository.`를 표시한다.

## 4. 상태 모델, row identity와 표시

G1 Main에는 항목당 하나의 collapsed status만 표시한다.

| 상태 | prefix | 의미 |
|---|---:|---|
| Clean | 공백 | 변경 없음 |
| Modified | `M` | 추적 파일 내용/메타 변경 |
| Added | `A` | index에 추가됨 |
| Deleted | `D` | 삭제됨 |
| Renamed | `R` | rename으로 인식됨 |
| Untracked | `?` | 추적되지 않음 |
| Ignored | `!` | ignore 규칙에 일치 |
| Conflicted | `U` | 해결되지 않은 충돌 |

우선순위는 `Conflicted > Deleted > Renamed > Added > Modified > Untracked > Ignored >
Clean`이다. 필수 backend fixture는 **Clean + 변경 상태 7개**를 모두 포함한다.

- cache는 `BTreeMap<RepoRelativePath, DetailedGitStatus>`다. OS `PathBuf`를 cache key로
  사용하지 않는다.
- Main은 실제 `FileEntry`와 정확히 일치하는 path만 장식한다. descendant roll-up이나
  삭제된 synthetic entry는 만들지 않는다.
- prefix 기능이 켜지면 repository의 모든 행에 `상태 1셀 + 공백 1셀`을 예약한다.
  clean/숨김 설정인 상태도 예약 폭은 2셀이다.
- `show_ignored = false`, `show_untracked = false`는 장식만 숨기며 파일을 숨기지 않는다.
- prefix는 namespaced style role만 사용하고 filename/file-type/marked/cursor style을
  덮어쓰지 않는다.

Git Status View의 범위는 **현재 active repository 전체**다. Main의 현재 directory로
filter하지 않고 repo-relative path를 표시한다. 안정 identity는
`(RepositoryIdentity, RepoRelativePath)`다. rename은
new path가 identity이고 old path는 row metadata다. 기본 정렬은 normalized path 오름차순,
같은 path면 collapsed status 우선순위다. refresh 후에는 같은 identity의 cursor와 mark만
보존한다. cursor row가 사라지면 이전 index에서 가장 가까운 row를 선택하고, 새 row는
자동 mark하지 않는다. repository identity가 바뀌면 cursor는 첫 row, marks는 빈 집합이다.

필수 namespaced theme role:

```text
plugin.git.modified   plugin.git.added      plugin.git.deleted
plugin.git.renamed    plugin.git.untracked  plugin.git.ignored
plugin.git.conflicted plugin.git.branch     plugin.git.stale
```

## 5. status summary와 화면

repository 안에서는 남는 폭에 다음 순서의 Git summary를 기여한다.

```text
Files 42 | Selected 0 | Free 120 GB | main | U:1 M:3 A:1 ?:2
```

- Core의 Files/Selected/Free를 먼저 보존한다. Git item은 full → compact → hidden이다.
- branch, conflict, modified, added, deleted, renamed, untracked 순으로 축약한다.
- loading은 `Git: loading...`, stale cache는 stale style과 이유를 표시한다.
- ahead/behind는 G3 전에는 표시하지 않는다.

Main 기본 진입은 `Alt+G`, 전체 수동 refresh는 `Ctrl+R`이다. Git Status View는
Up/Down/Home/End/PgUp/PgDn, Space, Enter/F3, Esc를 제공한다. G2 이전 F5~F11은
disabled style과 구체적 availability reason을 표시한다. 화면을 닫아도 Main의 path,
cursor, marks는 바뀌지 않는다.

Git Status context의 기능키는 다음이 단일 기본값이다. 12개 slot은 항상 표시하며 아직
도입되지 않았거나 현재 row/state에서 불가능한 command는 `---`로 숨기지 않고 label과
구체적 disabled reason을 사용한다.

| 키 | CommandId | label | 최초 단계 |
|---|---|---|---|
| F1 | Core Help | Help | G1 |
| F2 | 없음 | `---` | — |
| F3 / Enter | `plugin.git.diff` | Diff | G1 |
| F4 | 없음 | `---` | — |
| F5 | `plugin.git.stage` | Stage | G2 |
| F6 | `plugin.git.unstage` | Unstage | G2 |
| F7 | `plugin.git.commit` | Commit | G2 |
| F8 | `plugin.git.discard_worktree` | Discard | G2 |
| F9 | `plugin.git.stash_current` | Stash | G2 |
| F10 | `plugin.git.open_log` | Log | G2 |
| F11 | `plugin.git.open_branches` | Branch | G2 |
| F12 | `plugin.git.open_menu` | Git Menu | G1 |

G1 Git Menu는 Refresh All, 가능한 Diff Target 선택, Close를 제공한다. G2에서 Stash
List/Apply/Pop/Drop을 추가한다. G3에서 active repository의 Fetch/Pull/Push/Remote Manage와
conflict Continue/Abort를 추가한다. Clone은 repository가 없어도 Local Main의 F12 Menu
`Git > Clone…`에서 `plugin.git.clone`으로 진입한다. G3 network command에는 별도 기본
function key를 만들지 않으며 `Ctrl+R`은 fetch가 아니라 로컬 `RefreshAll`이다.

## 6. Diff Viewer와 DiffTarget

```rust
pub enum DiffTarget {
    Staged,
    Unstaged,
    Combined,
}
```

- index 변경만 있으면 기본 target은 `Staged`, worktree 변경만 있으면 `Unstaged`, 둘 다
  있으면 `Combined`다. Combined는 staged section 다음 unstaged section 순서다.
- rename은 new path identity와 old path metadata를 backend에 전달해 rename header를
  보존한다. deleted path는 존재를 요구하지 않는다.
- 사용자는 Git Menu에서 세 target 중 가능한 target을 명시적으로 고를 수 있다.
- diff는 worker에서 생성하며 G1 기본 출력 상한 `8 MiB`(8,388,608 bytes), deadline,
  cancellation을 적용한다. 상한을 넘으면 partial text를 보여 주지 않고 TooLarge 상태다.
- binary/too-large/deleted/renamed/backend error는 패닉 대신 이유를 표시한다.
- 기존 Viewer의 scroll/search/viewport를 재사용하며 Main/Git Status state를 손상하지 않는다.

## 7. refresh, generation, cancellation

refresh 종류:

```text
RefreshStatus = 현재 repository snapshot만 다시 읽음
RefreshAll    = 현재 directory discovery cache 무효화 → discover → repository면 snapshot
```

- directory changed는 exact-directory discovery를, file operation/Git mutation 완료는
  `RefreshStatus`를, `Ctrl+R`은 `RefreshAll`을 요청한다.
- cursor/mark/sort/render는 refresh trigger가 아니다.
- job/result는 `(plugin id, plugin generation, repository identity, request id)`를 가진다.
- 같은 scope의 pending refresh는 coalesce한다. 더 강한 `RefreshAll`은 pending
  `RefreshStatus`를 대체할 수 있다.
- queued/running job은 thread-safe cancellation token과 deadline을 받는다. backend가 즉시
  중단하지 못해도 결과는 stale 검증 후 폐기한다.
- renderer와 plugin callback은 immutable state만 사용하며 filesystem/backend/clock을
  호출하지 않는다.

## 8. G2 mutation 안전 정책

- Git mutation은 Core file mutation과 같은 **공통 mutation lease**를 얻어 겹침을 막는다.
  plugin read lane과는 분리해 status/diff가 local copy/delete worker를 막지 않는다.
- active lease가 있으면 새 mutation은 queue에서 기다리지 않고 `Busy`로 거부하며 backend
  mutation은 0회다. 확인 대화상자에서 취소해도 backend는 0회다. 실행 중인 결과만
  OperationId로 추적한다.
- Stage/Unstage는 marks가 없으면 cursor, 있으면 marks 전체를 대상으로 한다. Stage는
  worktree 쪽의 tracked Modified/Deleted/Renamed와 untracked를 허용하고 ignored/conflicted와
  worktree delta가 없는 row를 거부한다. rename은 old/new path 쌍을 하나의 target으로
  검증한다. Unstage는 index 쪽 Added/Modified/Deleted/Renamed만 허용하고 worktree-only,
  untracked, ignored, conflicted row를 거부한다. 선택에 unsupported 또는 stale row가 하나라도
  있으면 전체 plan을 lease/backend 전에 거부한다. preflight를 통과한 plan의 실행 중 backend
  오류만 partial result가 될 수 있으며, Stage와 Unstage를 서로 암묵 변환하지 않는다.
  이 금지는 일반 Git Status View의 F5 Stage 계약이다. G3 conflict context는 별도 F5
  `Mark Resolved` command로만 unmerged index entry를 stage할 수 있다.
- Commit은 staged 항목이 없거나 message가 공백이면 실행하지 않는다. author와 committer는
  backend가 repository-local `user.name`/`user.email`, 그다음 user-global config 순서로
  해석하고 그 한 쌍을 author와 committer 모두에 명시적으로 전달한다. system config와
  `GIT_AUTHOR_*`/`GIT_COMMITTER_*` 환경 변수는 앱 identity source로 사용하지 않는다. 앱
  dialog/config/state에는 identity를 복사하거나 저장하지 않는다. name/email 중
  하나라도 없으면 mutation 0회와 `Git author identity is not configured. Configure user.name
  and user.email for this repository or user account.`를 표시한다. 자동 test는 격리 HOME과
  repository-local 또는 격리 HOME의 user-global identity를 사용해 개발자 machine의
  global/system config에 의존하지 않는다.
- Checkout은 dirty working tree를 먼저 검사하고 영향과 거부 이유를 표시한다. 강제
  checkout은 기본 범위가 아니다.
- F9 `Stash Current Changes`는 tracked 파일의 staged+unstaged 변경만 포함하고 untracked와
  ignored는 항상 제외한다. tracked 변경이 없으면 backend 호출 없이
  `No tracked changes to stash.`다. Apply/Pop은 staged 상태까지 복원하는 정책을 사용하며
  conflict/failure에서는 stash를 보존하고 Pop은 완전 성공 뒤에만 drop한다.
- `Discard local changes`는 `git revert`가 아니다. 대상과 복구 불가 문구를 표시하고
  별도 확인을 받는다. F8은 tracked worktree Modified/Deleted/Renamed만 index 상태로
  복원하고 staged 변경은 보존한다. index-only row는 `Unstage before discarding staged
  changes.`, untracked는 `Untracked deletion is not supported.`, conflicted row는
  `Resolve or abort the conflict first.`로 backend 전에 거부한다. selection에 unsupported
  row가 하나라도 있으면 전체 plan을 거부해 mutation 0회다.
- 부분 실패는 succeeded/failed/skipped 수와 첫 오류를 보고하고 terminal result 뒤
  `RefreshStatus`를 정확히 한 번 요청한다.
- branch delete는 G2 범위에서 연기한다.

## 9. G3 인증·전송 안전 정책

- 앱 config/state/snapshot/log/error에는 password, token, private-key bytes,
  authorization header를 저장하지 않는다.
- integration은 OS credential helper와 SSH agent/OS keychain을 사용하며 raw secret을 앱
  callback으로 반환하지 않는 방식만 승인한다. 이것을 만족하지 않는 backend는 선택하지
  않는다.
- unknown SSH host key는 host, algorithm, fingerprint를 보여 주고 사용자가 승인한 정책에
  따라 OS `known_hosts` 또는 명시적 session-only trust를 사용한다. 자동 accept는 금지다.
- TLS certificate/host-key 검증 우회와 shell command string은 금지다.
- network operation은 `Queued → Resolving/Auth → Transferring → Applying → Terminal` phase를
  보고한다. Queued~Transferring은 취소 가능하다. Applying처럼 backend가 원자적으로 끝내야
  하는 phase는 UI에 `Finishing…`으로 표시하고 결과 후 복구/refresh한다.
- G3 network operation은 기본 capacity 16의 single Git Transport lane을 사용한다. 한 G3
  operation이 Queued 이상이면 다른 G3 mutation command는 대기열에 쌓지 않고 `Busy`다.
  UI/cancel control과 G1/G2 read-only 화면은 계속 응답한다.
- Pull은 승인된 strategy만 사용하고 암묵적 force/reset을 하지 않는다. Push force는
  별도 승인 없이는 UI/backend 모두 제공하지 않는다.
- Git remote add/edit/remove는 name과 URL을 검증하고 영향받는 upstream을 확인창에
  표시한다. password/token을 포함한 URL은 저장하기 전에 거부한다.
- clone 실패/취소는 plugin이 새로 만든 partial target만 정리하며 기존 파일은 삭제하지 않는다.
- conflict view의 F5는 `Mark Resolved`다. active conflict의 cursor/marks만 대상으로 하며
  path가 아직 unmerged인지, resolved worktree file 또는 명시적으로 선택한 deletion인지
  preflight한다. deletion과 binary resolution은 대상 path를 보여 주는 별도 확인이 필요하다.
  mixed stale/invalid selection은 전체 거부하고, 성공 시 Core Local lane의 공통 mutation
  lease 아래 index에 반영한다. 일반 Git Status View의 conflicted Stage 차단은 유지한다.
  Continue는 unmerged entry가 0일 때만 enabled다.
- Fetch(ref), Pull(worktree/index), Push의 remote/ref update, remote config 변경, Clone의
  local target, conflict Continue/Abort는 모두 mutation으로 분류한다. Fetch/Pull/Push/Clone은
  `Queued → Resolving/Auth`를 lease 없이 수행할 수 있지만, 성공한 auth 뒤 첫
  `Transferring` 진입 직전에 공통 lease를 non-blocking으로 얻고 Terminal cleanup까지
  보유한다. active면 `Busy`로 끝내며 resolve/auth preflight 호출은 있을 수 있어도 transport
  transfer와 local/remote mutation 호출은 0회다. Remote Manage는 config write 직전,
  Conflict Continue/Abort는 apply 직전에 같은 규칙으로 얻는다. 어느 operation도 lease를
  기다리는 queue를 만들지 않는다.

## 10. 오류, 격리와 성능

- 모든 plugin callback은 `Result<_, PluginError>`다. manager는 callback을 `catch_unwind`
  경계에서 호출한다.
- error/panic plugin은 session `Faulted`가 되어 contribution과 pending result를 버리고,
  다른 plugin/Core는 계속 동작한다. 사용자가 다시 enable하면 새 generation의 깨끗한
  instance로 재생성한다.
- `GitError`는 operation, redacted repository/path, category, retry 가능 여부를 가진다.
- 일반 navigation key→frame 50 ms 목표를 Git enabled에서도 유지한다.
- 10,000 cached status decoration/layout/render smoke를 둔다.
- disabled는 전체 0회, cached non-repository는 명시된 discover 외 status/diff 0회를
  call-count test로 고정한다.

## 11. 의도적으로 연기한 항목

- 실제 backend는 비교 spike와 ADR에서 하나만 선택한다.
- submodule 전용 표시, 2문자 index/worktree 표시, branch delete, commit graph 고급 UI는 후속이다.
- external plugin SDK/ABI는 이 계획의 완료 조건이 아니다.
- G3 인증 integration과 pull strategy는 G3-00 승인 전 추측하지 않는다.
