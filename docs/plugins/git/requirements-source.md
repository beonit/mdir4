# Mdir Git Integration Plugin 계획

## 1. 개요

### 목적

Mdir III 스타일 파일 관리자에 Git 기능을 추가한다.

Git 기능은 파일 관리자 Core에 직접 결합하지 않고 **독립적인 Built-in Plugin**으로 구현한다.

```text
Mdir Core
    │
    └── Plugin Manager
            │
            └── Git Plugin
```

Git 플러그인을 비활성화할 경우 일반 파일 관리자 기능과 성능에 영향을 최소화해야 한다.

---

# 2. 기본 원칙

Git 통합은 다음 원칙을 따른다.

1. Mdir Core는 Git을 알지 못한다.
2. Git 정보는 Plugin API를 통해 파일 목록에 제공한다.
3. Git 처리는 UI thread를 가능한 한 block하지 않는다.
4. Git 저장소가 아닌 디렉터리에서는 아무 작업도 하지 않는다.
5. Git Plugin은 설정으로 완전히 비활성화할 수 있다.
6. Git 기능 때문에 기존 Mdir 키보드 UX가 변경되어서는 안 된다.
7. Git 상태 조회와 Git 변경 작업을 명확하게 분리한다.

---

# 3. 단계별 목표

Git 통합은 세 단계로 나눈다.

## Phase 1 — Git 정보 표시

읽기 전용 기능 중심.

```text
Repository Detection
Branch
File Status
.gitignore
Status Bar
Git Status View
Diff Viewer
```

Git을 모르더라도 파일 관리자로 사용하는 데 방해가 없어야 한다.

## Phase 2 — Local Git 작업

```text
Stage
Unstage
Commit
Log
Branch
Checkout
Stash
```

로컬 저장소 작업을 지원한다.

## Phase 3 — Remote Git 작업

```text
Fetch
Pull
Push
Clone
Remote
Authentication
Conflict Workflow
```

원격 저장소 기능을 지원한다.

Phase 3는 별도 모듈로 취급한다.

---

# 4. Plugin Architecture

초기 버전에서는 동적 DLL이나 WASM 플러그인을 사용하지 않는다.

Rust trait 기반 Built-in Plugin으로 구현한다.

```rust
trait Plugin {
    fn name(&self) -> &'static str;

    fn on_directory_changed(&mut self, path: &Path);

    fn decorate_file(
        &self,
        file: &FileEntry,
    ) -> Option<FileDecoration>;

    fn status_line(&self) -> Option<StatusItem>;

    fn handle_action(
        &mut self,
        action: &Action,
    ) -> PluginResult;
}
```

Git Plugin:

```rust
struct GitPlugin {
    enabled: bool,

    repository: Option<GitRepository>,
    root: Option<PathBuf>,
    branch: Option<String>,

    statuses: HashMap<PathBuf, GitFileStatus>,

    refresh_state: RefreshState,
}
```

---

# 5. Plugin Manager

Core에는 Plugin Manager를 둔다.

```text
App
 │
 ├── FileSystem
 ├── Layout
 ├── Navigation
 ├── Theme
 │
 └── PluginManager
       │
       ├── GitPlugin
       ├── ArchivePlugin
       └── Future Plugins
```

파일 렌더링 흐름:

```text
FileEntry
    │
    ▼
PluginManager
    │
    ├── GitPlugin
    │      └── Git Decoration
    │
    └── Other Plugins
    │
    ▼
DecoratedFileEntry
    │
    ▼
Renderer
```

Core FileEntry에는 Git 전용 필드를 추가하지 않는다.

금지:

```rust
struct FileEntry {
    ...
    git_status: GitStatus,
}
```

권장:

```rust
struct FileEntry {
    path: PathBuf,
    name: String,
    ...
}

struct FileDecoration {
    prefix: Option<String>,
    suffix: Option<String>,
    style: Option<StyleHint>,
}
```

---

# 6. Repository Detection

현재 경로가 Git 저장소 내부인지 자동으로 감지한다.

예:

```text
C:\Projects\Mdir\src\ui
```

현재 디렉터리에 `.git`이 없더라도 상위 디렉터리를 검사하여:

```text
C:\Projects\Mdir
```

을 Repository Root로 감지해야 한다.

상태:

```text
NotRepository

Repository {
    root
    worktree
}
```

디렉터리가 변경되었을 때만 repository detection을 다시 수행한다.

---

# 7. Git File Status

최소 다음 상태를 지원한다.

```rust
enum GitFileStatus {
    Clean,
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
    Ignored,
    Conflicted,
}
```

필요하면 Working Tree와 Index 상태를 분리한다.

향후:

```rust
struct GitStatus {
    index: GitIndexStatus,
    worktree: GitWorktreeStatus,
}
```

예:

```text
MM
M_
_M
A_
_D
??
UU
```

Phase 1에서는 사용자가 이해하기 쉬운 단일 상태로 단순화해도 된다.

---

# 8. 파일 목록 표시

Short View의 정보 밀도를 크게 낮추면 안 된다.

기본 표시 후보:

```text
 M README.md
 A new.rs
 ? temp.txt
   Cargo.toml
 D old.rs
```

상태 문자:

```text
M  Modified
A  Added
D  Deleted
R  Renamed
?  Untracked
!  Ignored
U  Conflict
```

Clean 파일은 공백으로 표시한다.

---

# 9. Git 상태 색상

Git 상태를 파일명 색상 또는 별도 decoration으로 표현할 수 있다.

기본적으로 기존 파일 타입 테마를 완전히 덮어쓰지 않는다.

예:

```text
File Type Color
        +
Git Decoration
```

즉:

```text
README.md
```

의 파일 종류 색상은 그대로 유지하면서 앞의:

```text
M
```

만 Git Modified 스타일을 사용할 수 있다.

테마 항목:

```text
git.modified
git.added
git.deleted
git.renamed
git.untracked
git.ignored
git.conflicted
git.branch
```

---

# 10. Git 상태 표시 On/Off

설정:

```toml
[plugins.git]

enabled = true
show_file_status = true
show_branch = true
show_ignored = false
```

런타임에서도 Git decoration을 끌 수 있어야 한다.

---

# 11. 하단 Status Bar

Git Repository 내부에서는 상태바에 Git 정보를 추가할 수 있다.

예:

```text
main | M:3 ?:2 | Files 42 | Selected 0 | Free 120 GB
```

항목:

```text
main      Branch
M:3       Modified
A:1       Added
?:2       Untracked
U:1       Conflict
```

상태바 폭이 부족하면 자동 축약한다.

예:

```text
main M3 ?2
```

---

# 12. Ahead / Behind

향후 remote tracking branch가 존재하면:

```text
main ↑2 ↓1
```

표시를 지원한다.

의미:

```text
↑2
로컬 commit이 remote보다 2개 앞섬

↓1
remote commit이 로컬보다 1개 앞섬
```

이 정보는 파일 이동 때마다 계산하지 않는다.

다음 시점에서만 갱신한다.

```text
Repository Load
Refresh
Fetch
Commit
Checkout
Push
Pull
```

---

# 13. Git 명령 진입

Git 기능 진입용 기본 단축키:

```text
Alt+G
```

`Alt+G`는 Git Status View를 연다.

단축키는 설정 가능해야 한다.

---

# 14. Git Status View

예:

```text
┌──────────────────── Git Status ──────────────────────┐
│ Repository : C:\Projects\Mdir                       │
│ Branch     : main                                   │
│                                                     │
│ M  src/app.rs                                       │
│ M  src/ui/file_list.rs                              │
│ A  src/git/mod.rs                                   │
│ ?  test.txt                                         │
│                                                     │
├─────────────────────────────────────────────────────┤
│ F3 Diff  F5 Stage  F6 Unstage  F7 Commit  F10 Log  │
└─────────────────────────────────────────────────────┘
```

기존 Mdir Navigation 원칙을 따른다.

```text
↑ ↓
파일 이동

Space
선택

Enter
파일 또는 Diff

Esc
Git View 종료
```

---

# 15. Git View 내 기능키

초기안:

```text
F1    Help
F3    Diff
F5    Stage
F6    Unstage
F7    Commit
F8    Revert
F9    Stash
F10   Log
F11   Branch
F12   Git Menu
```

기능키 표시 내용은 Context에 따라 변경할 수 있다.

---

# 16. Diff Viewer

Phase 1부터 지원한다.

Git 상태 파일에서:

```text
F3
```

또는:

```text
Enter
```

를 통해 Diff를 볼 수 있다.

예:

```diff
@@ -21,6 +21,8 @@
 fn render() {
     draw_header();
+    draw_git_status();
+    draw_status_bar();
 }
```

기능:

```text
↑ ↓
스크롤

PgUp PgDn
페이지

Home End

Ctrl+F
검색

Esc
닫기
```

가능하면 기존 Mdir File Viewer 구성요소를 재사용한다.

---

# 17. Stage

Phase 2.

파일 선택 후:

```text
F5
```

Stage.

다중 선택도 지원한다.

예:

```text
Selected: 3

Stage selected files?

Enter : Stage
Esc   : Cancel
```

---

# 18. Unstage

Phase 2.

```text
F6
```

사용.

다중 파일 Unstage 지원.

---

# 19. Commit

Phase 2.

```text
F7
```

Commit Dialog:

```text
┌──────────────── Commit ────────────────┐
│                                       │
│ Message                               │
│                                       │
│ Fix multi-column navigation           │
│                                       │
│                                       │
│ Staged: 4 files                       │
│                                       │
│ Ctrl+Enter Commit                     │
│ Esc        Cancel                     │
└───────────────────────────────────────┘
```

Commit message 입력은 multiline을 지원할 수 있다.

Phase 1에는 포함하지 않는다.

---

# 20. Git Log

Phase 2.

예:

```text
● a83f298  Fix multi-column navigation
● e2833aa  Add snapshot test
● 7166abd  Implement Git plugin
● 190aba1  Initial commit
```

표시 대상:

```text
Hash
Subject
Author
Date
Branch / Tag
```

선택한 commit에서:

```text
Enter
```

를 누르면 commit 상세 또는 diff를 표시할 수 있다.

---

# 21. Branch View

Phase 2.

```text
Branches

* main
  develop
  feature/git-plugin
  feature/theme
```

기능:

```text
Enter
Checkout

Insert
Create

Delete
Delete Branch
```

현재 파일 작업에 영향을 줄 수 있으므로 dirty working tree를 반드시 확인한다.

---

# 22. Stash

Phase 2 후반.

지원:

```text
Stash
Stash Pop
Stash Apply
Stash Drop
Stash List
```

초기에는 단순:

```text
Stash Current Changes
```

만 제공해도 된다.

---

# 23. Revert / Discard

매우 위험한 동작이므로 명확한 확인을 요구한다.

예:

```text
Discard local changes?

src/app.rs

This operation cannot be undone.

Enter : Discard
Esc   : Cancel
```

다중 파일 처리 시 파일 개수도 표시한다.

---

# 24. Refresh

Git 상태는 background에서 refresh 가능해야 한다.

기본:

```text
Directory Changed
       ↓
Repository Detect
       ↓
Status Refresh
```

추가 refresh 조건:

```text
File Operation Complete
Git Operation Complete
Manual Refresh
```

키:

```text
Ctrl+R
```

---

# 25. Performance

큰 Git Repository에서 매 frame마다 Git status를 계산해서는 안 된다.

금지:

```text
render()
    ↓
git status
    ↓
render()
```

권장:

```text
Directory Changed
     ↓
Background Git Scan
     ↓
Git State Cache
     ↓
Render Cache
```

렌더링은 캐시된 데이터만 읽는다.

---

# 26. Git Cache

예:

```rust
struct GitCache {
    repository_root: PathBuf,
    branch: String,
    statuses: HashMap<PathBuf, GitFileStatus>,
    updated_at: Instant,
}
```

동일 repository 안에서 디렉터리를 이동하는 경우 repository 자체를 다시 열 필요가 없도록 한다.

---

# 27. 비동기 처리

Git status 조회가 UI thread를 block하지 않도록 한다.

예:

```text
UI Thread
    │
    ├─ Navigation
    ├─ Render
    └─ Action
          │
          ▼
     Git Worker
          │
          ▼
     GitResult
          │
          ▼
      App Event
```

Git 정보 로딩 중에는:

```text
Git: loading...
```

정도로 표시할 수 있다.

---

# 28. Git Backend

초기 검토 대상:

```text
gix
```

또는:

```text
git2
```

Backend는 Plugin UI에서 직접 사용하지 않는다.

추상화:

```rust
trait GitBackend {
    fn discover(&self, path: &Path) -> Result<RepositoryInfo>;

    fn status(
        &self,
        repository: &RepositoryInfo,
    ) -> Result<Vec<GitFileState>>;

    fn diff(
        &self,
        path: &Path,
    ) -> Result<GitDiff>;
}
```

향후 backend 교체가 가능해야 한다.

---

# 29. Git CLI Backend 가능성

라이브러리 외에 시스템 `git` CLI를 사용하는 backend도 선택적으로 지원할 수 있다.

```text
GitBackend
 ├── GixBackend
 └── CliBackend
```

CliBackend는 다음 명령 등을 사용한다.

```text
git status --porcelain
git diff
git log
```

단, CLI 파싱과 process 실행은 별도 모듈로 분리한다.

---

# 30. Error Handling

Git 오류는 파일 관리자 전체를 종료시키지 않는다.

예:

```text
Git repository unavailable
```

또는:

```text
Git Error: index.lock exists
```

Git Plugin에 문제가 생기더라도 파일 탐색은 계속 가능해야 한다.

---

# 31. Repository 밖의 동작

Git Repository가 아닌 경우:

```text
Git: -
```

를 반드시 표시할 필요는 없다.

기본적으로 Git 관련 UI 자체를 숨긴다.

`Alt+G`를 누르면:

```text
Not a Git repository.
```

정도만 표시한다.

---

# 32. Nested Repository

중첩된 repository를 지원한다.

예:

```text
project/
 ├ .git/
 │
 └ vendor/
      └ library/
           └ .git/
```

`vendor/library`로 진입하면 가장 가까운 Git repository를 활성 repository로 사용한다.

---

# 33. Worktree

Git Worktree를 고려한다.

Repository Root와 Worktree Root를 별도로 관리할 수 있어야 한다.

---

# 34. Submodule

초기 Phase에서는 Submodule을 일반 디렉터리처럼 취급한다.

향후 상태 표시:

```text
S
```

등의 별도 Git decoration을 추가할 수 있다.

Phase 1 필수 기능은 아니다.

---

# 35. Gitignore

Ignored 파일 처리:

기본:

```text
show_ignored = false
```

파일 관리자는 실제 파일 자체를 숨기는 것이 아니라 Git decoration에서 ignored 표시 여부만 제어한다.

즉 Mdir의 Hidden File 설정과 Git Ignore는 별개다.

---

# 36. Plugin 설정

예:

```toml
[plugins.git]

enabled = true

show_status = true
show_branch = true
show_untracked = true
show_ignored = false

status_prefix = true

refresh_on_directory_change = true
refresh_on_file_operation = true

shortcut = "Alt+G"
```

---

# 37. Plugin 테스트

Git Plugin도 기존 자동 테스트 구조에 통합한다.

Fixture:

```text
tests/
  fixtures/
    git/
      clean/
      modified/
      added/
      deleted/
      renamed/
      untracked/
      ignored/
      conflict/
      many-files/
```

---

# 38. Git Status Unit Test

검증:

```text
Modified → M
Added → A
Deleted → D
Renamed → R
Untracked → ?
Ignored → !
Conflict → U
```

---

# 39. Git Navigation Snapshot

예:

```yaml
name: git-status-view

terminal:
  width: 80
  height: 25

fixture:
  git: modified

steps:
  - snapshot: main

  - key: ALT_G
  - snapshot: git-status

  - key: DOWN
  - key: F3
  - snapshot: diff-view
```

---

# 40. Main View Snapshot

Git 적용 전:

```text
README.md      Cargo.toml      src
```

Git 적용 후:

```text
M README.md      Cargo.toml      src
```

이 변경으로 인해 컬럼 정렬이 깨지지 않는지 테스트한다.

특히 다음 화면 폭을 테스트한다.

```text
80
81
100
120
160
```

---

# 41. Git 상태 변경 테스트

예:

```text
Initial
README.md = Clean

파일 수정

Refresh

README.md = Modified
```

Fake GitBackend를 사용하여 실제 Git repository 없이도 테스트 가능하게 한다.

---

# 42. Fake Git Backend

```rust
struct FakeGitBackend {
    repository: Option<RepositoryInfo>,
    statuses: HashMap<PathBuf, GitFileStatus>,
}
```

테스트에서:

```text
README.md → Modified
main.rs   → Added
temp.txt  → Untracked
```

상태를 직접 주입할 수 있어야 한다.

---

# 43. Snapshot 결정성

테스트에서 다음 값은 고정한다.

```text
Branch
Commit Hash
Commit Date
Author
Remote Status
```

실제 사용자 Git 설정이나 저장소 상태에 의존하지 않는다.

---

# 44. Phase 1 범위

필수:

```text
Plugin Manager
Git Plugin
Git Backend Interface
Repository Detection
Repository Root
Branch
Git File Status
Untracked
Ignored 처리
File Decoration
Git Theme
Status Bar
Git Status View
Diff Viewer
Refresh
Fake Backend
Unit Test
Snapshot Test
```

Phase 1에서는 Git repository를 변경하지 않는다.

즉 **읽기 전용 Git 통합**이다.

---

# 45. Phase 2 범위

추가:

```text
Stage
Unstage
Commit
Log
Branch View
Branch Checkout
Branch Create
Stash
Discard
```

Git repository 변경 작업이 시작되는 단계다.

각 destructive operation에는 별도 테스트와 확인 UI가 필요하다.

---

# 46. Phase 3 범위

추가:

```text
Fetch
Pull
Push
Clone
Remote
HTTPS Authentication
SSH Authentication
Credential Handling
Conflict Resolution
```

Remote 기능은 별도 설계 문서를 만드는 것을 권장한다.

---

# 47. 향후 External Plugin

Git Plugin 구조가 안정화된 이후 Plugin API를 외부로 확장할 수 있다.

후보:

```text
WASM Plugin
Process-based Plugin
RPC Plugin
```

초기에는 ABI 안정성 문제 때문에 Rust Dynamic Library Plugin은 사용하지 않는다.

---

# 48. 향후 적용 가능한 Plugin

Git Plugin이 Plugin API의 첫 Reference Implementation 역할을 한다.

이후 같은 구조로:

```text
Archive Plugin

7-Zip Plugin

SFTP Plugin

SSH Plugin

Image Metadata Plugin

Hash Plugin

Preview Plugin

GitHub Plugin

File Search Plugin
```

등을 구현할 수 있다.

---

# 49. 완료 기준 — Phase 1

Git Plugin Phase 1 완료 조건:

* Git Plugin을 설정으로 활성/비활성 가능
* Git repository 자동 감지
* 중첩 repository 감지
* 현재 branch 표시
* Modified 표시
* Added 표시
* Deleted 표시
* Renamed 표시
* Untracked 표시
* Conflict 표시
* 파일 목록의 멀티 컬럼 레이아웃 유지
* Status Bar Git 정보 표시
* Alt+G Git Status View
* F3 Diff
* Git 작업 중 UI freeze 없음
* Git repository가 아닌 위치에서 정상 동작
* Git Plugin 오류가 Core를 종료시키지 않음
* FakeGitBackend 테스트 가능
* Git Unit Test 통과
* Git Snapshot Test 통과
* CI 환경에서 실제 Git 저장소 없이 대부분의 UI 테스트 가능

---

# 50. 개발 순서

```text
1. Plugin trait
      ↓
2. PluginManager
      ↓
3. GitBackend trait
      ↓
4. FakeGitBackend
      ↓
5. Repository Detection
      ↓
6. Git Status Model
      ↓
7. File Decoration
      ↓
8. Main View Git Snapshot
      ↓
9. Status Bar
      ↓
10. Git Status View
      ↓
11. Diff Viewer
      ↓
12. Async Refresh
      ↓
13. Real Git Backend
      ↓
14. Phase 1 Integration Test
```

중요한 점은 **실제 Git 라이브러리부터 붙이지 않는 것**이다.

먼저:

```text
Plugin API
+
Fake Git Backend
+
UI
+
자동 테스트
```

를 완성한 뒤 실제 `gix` 또는 `git2` backend를 연결한다.

이렇게 하면 Git 구현 방식이 변경되어도 UI와 Core 구조를 거의 수정하지 않아도 된다.

---

# 51. 최종 구조

```text
                    ┌─────────────────┐
                    │    Mdir Core    │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │ Plugin Manager  │
                    └────────┬────────┘
                             │
                 ┌───────────▼───────────┐
                 │      Git Plugin       │
                 └─────┬──────────┬──────┘
                       │          │
              ┌────────▼───┐  ┌───▼──────────┐
              │ GitBackend │  │ Git UI       │
              └──────┬─────┘  │              │
                     │        │ Status       │
             ┌───────┴─────┐  │ Diff         │
             │             │  │ Log          │
        FakeBackend   RealBackend             │
                       │      │              │
                       │      └──────────────┘
                       ▼
                 gix / git2 / CLI
```

Git Plugin은 Mdir Plugin 시스템의 첫 구현체이자 향후 다른 확장 기능을 설계하기 위한 기준 Plugin으로 사용한다.
