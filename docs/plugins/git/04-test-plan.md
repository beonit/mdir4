# Git built-in 테스트 계획

## 1. 원칙

1. host/Git reducer/UI는 Fake로 먼저 검증하고 real backend는 같은 contract suite를 재사용한다.
2. 실제 repository는 `TempDir` 아래 disposable fixture만 사용한다.
3. home/global Git config/credential/public network에 의존하지 않는다.
4. hash/branch/author/date/path/지연은 fixture에 고정한다.
5. callback/render 중 filesystem/backend/clock 호출은 0이다.
6. mutation/network test는 before/after와 cancellation phase를 함께 검증한다.
7. snapshot은 문자뿐 아니라 cell별 namespaced style role도 assert한다.

공통 gate:

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
```

## 2. 계층과 Fake 분리

| 계층 | 대상 | test double |
|---|---|---|
| Host unit/contract | API, manager, collision, fault, worker | `FakePlugin` |
| G1 model/component/scenario | discovery/status/diff/UI | `FakeGitReadBackend` |
| G2 mutation | planner/lease/dialog/result | `FakeGitMutationBackend` |
| G3 auth/network | phase/progress/redaction/conflict | `FakeGitTransportBackend` |
| Backend conformance | Fake와 선택한 real adapter | capability별 동일 suite |
| Integration | disposable local repository/isolated remote | real adapter |
| Manual | Windows Terminal/helper/agent/host-key | throwaway 환경 |

Fake capability를 합치지 않는다. G1 Fake에는 mutation/network method가 없고 G2 Fake에는
credential/network method가 없다.

## 3. fixture 계약

G1:

| fixture | 필수 내용 |
|---|---|
| `not-repository` | negative discovery |
| `clean` | branch와 tracked clean file |
| `mixed` | **Clean + Modified/Added/Deleted/Renamed/Untracked/Ignored/Conflicted 7개** |
| `nested` | outer 안 inner, outer cache를 먼저 데울 수 있음 |
| `worktree` | metadata root와 worktree root가 다름 |
| `detached` / `unborn` | 고정 short hash / initial branch |
| `many-files` | 10,000 cached status |
| `backend-errors` | unavailable/permission/index.lock/timeout/panic |

G2는 `staged`, `dirty-branch`, `stash-current`, `stash-list`, `partial-failure`를 추가한다.
G3는 `two-clones`, `diverged`, `auth-required`, `unknown-host-key`, `remote-conflict`를 추가한다.

## 4. G0 host contract

필수 assert:

```text
disabled callback/job/contribution == 0
callback Err/panic => owner Faulted, Core/other plugin survives
faulted late result ignored; re-enable creates fresh generation/instance
duplicate plugin/command/view/status id rejected with reason
unknown/foreign/payload-type-mismatch result ignored safely
plugin read lane does not block Core/local mutation lane
queued/running cancellation returns one terminal result
shutdown cancels and joins workers; no orphan
view owner alone can update/close; close preserves Main path/cursor/marks
```

Decoration tests assert reserved cell width, StyledSpan ranges, invalid-width fault and
`plugin.<id>.*` style namespace. Command tests assert `Disabled { reason }`, user key override,
screen context, Core/plugin collision ordering, and registry-generated hint equality.

Dependency/source boundary:

- Core model/layout → Git module/backend crate 금지
- renderer/plugin callback → backend/filesystem/clock 금지
- backend → app/ui/ratatui 금지
- Fake read → mutation/transport dependency 금지

## 5. status, path와 row identity unit test

Collapsed table:

```text
Clean=' ' Modified=M Added=A Deleted=D Renamed=R
Untracked=? Ignored=! Conflicted=U
```

우선순위의 모든 pair와 대표 triple을 table test한다. cache type은 compile/source assertion으로
`BTreeMap<RepoRelativePath, ...>`임을 고정하고 `..`, absolute replacement, 다른 drive를 거부한다.

Status View refresh test:

1. active repository 전체 row를 repo-relative path로 표시하고 current directory로 filter하지 않음
2. stable identity row의 cursor/mark 보존
3. 새 row는 unmarked
4. marked row 삭제 시 set에서 제거
5. cursor row 삭제 시 old index에 가장 가까운 row
6. rename은 new path identity + old path metadata
7. repository identity 변경 시 first row + empty marks
8. normalized path sort와 동일 path status tiebreak

## 6. discovery/cache/refresh call matrix

| Event | disabled | non-local | cached non-repo | active repo |
|---|---:|---:|---:|---:|
| cursor/mark/sort/render | 0 | 0 | 0 | 0 |
| local directory change, exact cache miss | 0 | 해당 없음 | discover 1 | discover 1 |
| local directory change, exact cache hit | 0 | 해당 없음 | 0 | 0 |
| file operation completed | 0 | 0 | 0 | snapshot 1 |
| `Ctrl+R` (`RefreshAll`) | 0 | 0 | discover 1 | discover 1 + snapshot 1 |
| plugin enabled on local directory | manager가 1회 전달 | 0 | discover 1 | discover 1 + snapshot 1 |
| Git mutation/network terminal | 해당 없음 | 0 | 0 | snapshot 1 |

“non-repository backend 0” test는 status/diff와 unnamed automatic jobs가 0임을 뜻한다.
표의 named discovery trigger는 별도 assert한다. `Alt+G` nonrepo message도 backend 0이다.

중첩 회귀는 반드시 다음 두 순서를 포함한다.

```text
outer repo cache warm → first visit inner directory → discover inner → inner active
negative parent cache warm → first visit child repo → discover child → child active
parent repo submodule directory Enter → gitfile target discover → submodule repo active
```

또한 A refresh → B 이동/result → A late result, disable/re-enable, RefreshStatus→RefreshAll
upgrade, same-scope coalescing을 검증한다.

## 7. decoration/status/view snapshot

이름은 `git_<screen>__<fixture>__<width>x<height>__<state>.snap`이다. 필수 폭은
80/81/100/120/160, compact 60×15, too-small 59×14다.

Main snapshot:

- prefix enabled repository row는 clean/hidden 상태도 정확히 2셀 예약
- 각 column filename 시작 cell 동일
- prefix role만 Git namespaced role이고 filename/type/cursor/marked role 보존
- Unicode ellipsis가 cell boundary를 넘지 않음
- deleted path synthetic row 없음, descendant roll-up 없음
- Core status item이 Git summary 때문에 사라지지 않음

Git Status snapshot은 loading/empty/mixed/stale/error, disabled availability reason,
refresh insert/delete/rename을 포함한다. close 뒤 Main state도 reducer assert한다.

CommandRegistry/mapper 표는 F1 Help, F2 `---`, F3 Diff, F4 `---`, F5 Stage, F6 Unstage,
F7 Commit, F8 Discard, F9 Stash, F10 Log, F11 Branch, F12 Git Menu를 80/120열에서 검증한다.
G1에는 F5~F11 reason, G2에는 row/state별 availability를 assert한다. F12 menu는 G1
Refresh/DiffTarget/Close, G2 stash operations, G3 fetch/pull/push/remote/conflict를 단계별로
추가하며 Main F12 `Git > Clone…`과 `Ctrl+R != Fetch`도 mapper test한다.

각 단계의 snapshot/message test는 built-in label/Help/dialog/error를 canonical English
expected string과 비교한다. fixture에는 Unicode/Korean path, branch, author, commit message,
remote/ref를 넣고 이 사용자/저장소 text가 byte-for-byte 같은 display text로 남는지 별도로
assert한다. 단순히 모든 non-ASCII 문자를 금지하는 source scan으로 대체하지 않는다.

## 8. DiffTarget contract

| detailed state | 기본 target |
|---|---|
| index only | Staged |
| worktree only | Unstaged |
| index + worktree | Combined |

Combined는 staged section 다음 unstaged section이다. menu에서 가능한 target을 직접 고르는
test, rename new+old metadata, deleted path, binary, `8 MiB -1/정확히/+1`, cancel/deadline,
stale result, search/scroll snapshot을 둔다. 불가능한 target은 backend 호출 전에 reason을
표시한다.

## 9. 시나리오 확장

기존 scenario v1 field와 M1-13의 tagged action 형식을 유지한 채 optional `git` fixture와
generic plugin action을 추가한다. 기존 v1 scenario는 수정 없이 계속 통과하고, Git이 아직
구현되지 않은 단계에서는 아래 field/action을 명시적으로 거부해야 한다.

```yaml
version: 1
terminal: { width: 80, height: 25 }
start_path: /work/repo
filesystem:
  - { path: /work/repo, kind: directory }
clock: "2026-01-02T03:04:05Z"
disk: { free_bytes: 12288 }
git:
  fixture: mixed
steps:
  - { action: start }
  - { action: complete_plugin_effect, effect: discover }
  - { action: complete_plugin_effect, effect: snapshot }
  - { action: snapshot, name: main-decorated }
  - { action: key, key: alt+g }
  - { action: key, key: f3 }
  - { action: complete_plugin_effect, effect: diff }
  - { action: snapshot, name: diff }
  # Close Diff, then close Git Status.
  - { action: key, key: esc }
  - { action: key, key: esc }
  - action: assert
    path: /work/repo
    plugin.git.repository: repo
    plugin.git.view: none
assertions: { path: /work/repo, selected: 0, marked: 0, free_bytes: 12288 }
snapshots: [main-decorated, diff]
```

추가 step은 `complete_plugin_effect`, `fail_plugin_effect`, `cancel_plugin_effect`,
`set_plugin_enabled`, `assert.plugin`이다. unknown/duplicate/out-of-order completion은 파일명과
step 번호를 포함한 오류다. raw payload나 backend concrete type은 YAML에 노출하지 않는다.

## 10. real read backend conformance

- discover nonrepo/parent/nested/linked worktree
- Clean + 변경 상태 7개와 precedence
- ignored query/표시 분리
- branch/detached/unborn
- Staged/Unstaged/Combined, text/binary/renamed/deleted diff
- Unicode/space/leading-dash path
- deadline/cancel, repo deletion, lock/permission error

fixture helper는 write/delete 전 TempDir containment를 assert하고 local author/email/date를
주입한다. global/system config/credential helper/network를 G1/G2에서 차단하고 child/worker를
항상 join한다. G1 실행 전후 tree/index/HEAD는 동일해야 한다.

## 11. G2 mutation/lease 공통 test

```text
targets are repo-relative and inside worktree
Esc before submit 또는 active lease Busy => mutation call 0
Core file op and Git mutation never overlap
blocked mutation does not block plugin-read navigation/diff
confirmation has target/count and irreversible warning when needed
terminal result reports succeeded/failed/skipped
RefreshStatus exactly once after terminal result
unrelated files unchanged on error
```

작업별 before/after: Stage index만 변경, Unstage worktree 보존, Commit unstaged 보존,
Branch create HEAD/worktree 불변, Checkout dirty block, stash pop 실패 시 stash 보존,
Discard 승인 대상만 변경한다. Stash current는 tracked staged+unstaged만 저장하고
untracked/ignored를 보존한다. apply/pop은 staged state를 복원하며 실패 시 stash를 보존한다.
Discard는 tracked worktree를 index로만 되돌리고 index-only/untracked/conflict/mixed-invalid
selection은 calls 0이다. Stash current와 list/apply/pop/drop suite는 분리한다.

Stage matrix는 worktree tracked Modified/Deleted/Renamed와 untracked만 허용하고, Unstage
matrix는 index Added/Modified/Deleted/Renamed만 허용한다. ignored/conflicted/반대 side-only와
stale row, invalid row가 섞인 selection은 plan 전체를 backend 0회로 거부한다. rename은 old/new
pair를 함께 검증한다. Commit suite는 격리 HOME, system config 차단, repo-local identity를
사용하며 missing `user.name`/`user.email` 각각에서 mutation 0과 정확한 안내를 검증한다.
별도 case는 격리 HOME global-only 성공, repo/global 불일치에서 repo 우선, 결과 commit의
author/committer name+email 동일, system config와 `GIT_AUTHOR_*`/`GIT_COMMITTER_*`가 결과에
영향 0임을 확인한다.
G3 conflict context만 F5를 `Mark Resolved`로 override한다. active unmerged row, deletion/binary
확인, mixed-stale 전체 거부, common lease Busy 0회, 성공 뒤 index unmerged 0과 Continue
availability를 검증하고 일반 Status F5 정책이 완화되지 않았는지 회귀시킨다.

## 12. G3 auth/network test

첫 network test 전에 `FakeGitTransportBackend`로 다음을 통과해야 한다.

- OS helper/SSH agent availability만 전달하고 raw password/token/key byte API가 없음
- config/state/error/log/snapshot/progress의 secret scan 0건
- unknown/changed host key가 host/algorithm/fingerprint만 표시, 자동 accept 0
- TLS validation bypass 0
- `Queued/Resolving/Auth/Transferring/Applying/Terminal` phase 순서
- Queued/Auth/Transferring cancel, Applying의 `Finishing…`과 single terminal recovery
- capacity 1 Git Transport lane의 non-blocking submit, active G3 operation 중 두 번째 mutation
  `Busy`, cancel/error/panic/shutdown 뒤 worker join과 lease 반환

CI remote는 filesystem/local protocol 또는 격리 test server만 쓴다. two-clone
fetch/pull/push/rejection/conflict, offline/deadline, clone partial cleanup/preexisting target 보존을
검증한다. remote add/edit/remove는 invalid/credential-bearing URL, duplicate name, upstream
영향과 cancel/error redaction을 별도 검증한다. 실제 GitHub/GitLab/SSH는 승인된 throwaway
credential/repository 수동 checklist에서만 쓴다.

Fetch/Pull/Push/Clone 각각은 resolve/auth 뒤 첫 Transferring 전에 lease를 얻는다. active Core
mutation lease에서는 Busy가 되고 auth preflight 호출은 허용하되 transport transfer와
local/remote mutation call은 0인지 검증한다. Remote Manage는 config write 전, Conflict는 apply
전에 같은 조건을 검증한다. lease는 terminal/cancel/error/panic 뒤 반환되며, 순수 auth
preflight와 plugin-read lane은 lease를 점유하지 않는다.

## 13. 성능과 CI

이름 있는 release smoke 예:

```text
cargo test --release --locked git_perf_10k -- --ignored --nocapture
```

측정: disabled 전체 0, cached nonrepo named discover 외 0, 10k decoration/layout/render,
blocked read 중 navigation+Core mutation, coalescing call count, key→frame 50 ms 목표. 측정 없이
cache/pool/async runtime을 추가하지 않는다.

G0/G1 CI는 Git 미설치 Fake suite, 선택 real backend job, Windows, `.snap.new` 0,
read-only before/after 불변을 포함한다. G2/G3 Windows 수동 test는 throwaway repo/clone만 사용하고
OS/terminal/backend/helper/agent version과 결과를 progress에 기록한다.
