# 아키텍처

이 문서는 구현되어 있는 구조와 앞으로 만들 구조를 섞지 않는다. 각 절의 표시는 다음과
같다.

- **현재(M1 기준선)**: 저장소에 지금 존재하는 코드다.
- **M1-13 목표**: M2에 들어가기 전에 닫아야 하는 계약 보정이다.
- **M2 목표**: 해당 작업 카드가 구현할 구조이며 아직 현재 코드로 간주하지 않는다.
- **post-v1 목표**: R1 이후 Git 또는 SSH Remote 단계에서만 도입한다.

구현 여부와 다음 카드는 [`progress.md`](progress.md), 구체적인 완료 조건은
[`03-task-cards.md`](03-task-cards.md)를 기준으로 판정한다.

## 1. 변하지 않는 기본 원칙

Mdir4는 단일 Cargo 패키지의 모듈형 모놀리스다. 상태 전이와 외부 효과는
reducer/effect 경계로 분리한다.

```text
Crossterm Event
      │
      ▼
InputMapper ──► Action
                    │
                    ▼
              reduce(state, action)
                    │
         ┌──────────┴──────────┐
         ▼                     ▼
   mutated state           Vec<Effect>
                                 │
                                 ▼
                          background executor
                                 │
                                 ▼
                         completion Action

AppState + LayoutMetrics ──► render(Frame) ──► Crossterm/TestBackend
```

- reducer는 OS I/O를 하지 않는다.
- render는 immutable state만 읽고 port, 시계, worker를 호출하지 않는다.
- Effect 결과는 completion Action으로만 상태에 들어온다.
- InputMapper와 CommandRegistry가 키와 활성 상태의 단일 원본이다.
- layout/navigation은 Ratatui buffer와 OS에 의존하지 않는 순수 계산이다.

세부 결정은 [ADR-001](../architecture/adr-001-reducer-effect.md)과
[ADR-002](../architecture/adr-002-shared-layout-navigation.md)를 따른다.

---

## 2. 현재 M1 기준선

### 2.1 실제 소스 트리

아래 트리는 현재 production source의 구조다. 목표 트리와 혼동하지 않는다.

```text
src/
  main.rs
  lib.rs
  runtime.rs
  app.rs
  app/
    command_registry.rs
  fs.rs
  input/
    mod.rs
    key.rs
    mapper.rs
  layout.rs
  layout/
    navigation.rs
  model/
    mod.rs
    directory.rs
    selection.rs
  ports/
    mod.rs
    filesystem.rs
    disk.rs
    launcher.rs
  adapters/
    mod.rs
    real_fs.rs
    memory_fs.rs
    recording.rs
    system_disk.rs
    system_launcher.rs
  theme/
    mod.rs
    schema.rs
    classic.rs
  ui.rs
  ui/
    palette.rs
```

빈 디렉터리나 미래 모듈을 미리 만들지 않는다. 다음 절의 목표 트리는 관련 카드가 실제
코드를 필요로 할 때만 생성한다.

### 2.2 실제 상태와 Effect

현재 `AppState`, `Screen`, `Effect`의 기준선은 다음과 같다. Viewer, Editor, dialog stack,
operation state는 아직 없다.

```rust
pub enum Screen {
    Main,
    Help,
    QuitConfirm,
}

pub enum Effect {
    LoadDirectory(PathBuf),
    LoadDiskInfo(PathBuf),
    LaunchFile(PathBuf),
}

pub struct AppState {
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected: usize,
    pub marked: HashSet<EntryId>,
    pub viewport: Viewport,
    pub layout_settings: LayoutSettings,
    pub screen: Screen,
    pub message: Option<String>,
    pub free_space: Option<u64>,
    pub should_quit: bool,
}
```

현재 reducer는 `Started`, directory/disk/launcher completion, 탐색, page 이동, mark,
select-all, open/parent/reload, help와 quit-confirm Action을 처리한다. 런타임은
`InputMapper → reduce → EffectWorker → completion Action` 순서를 유지한다.

### 2.3 실제 entry와 read-only 파일 시스템 port

현재 `EntryId`는 선택/mark를 유지하기 위한 로컬 경로 alias다. 영구 filesystem ID가
아니다.

```rust
pub type EntryId = PathBuf;

pub struct FileEntry {
    pub path: PathBuf,
    pub name: OsString,
    pub kind: EntryKind,
    pub size: u64,
}

pub struct EntryMetadata {
    pub kind: EntryKind,
    pub size: u64,
}

pub trait FileSystem: Send + Sync {
    fn read_dir(&self, path: &Path) -> Result<Vec<FileEntry>, FsError>;
    fn metadata(&self, path: &Path) -> Result<EntryMetadata, FsError>;
}
```

따라서 현재 port는 **read-only**다. Rename, MkDir, Viewer read, Editor write, Copy/Move/Delete
method가 이미 존재한다고 가정하면 안 된다. 이 기능 경계는 M2-02가 도입한다.

`DirectoryListing`이 directory-first/name/path 순으로 정렬하고 `..` 합성 항목을
삽입한다. 내부 경로는 `PathBuf/OsString`을 유지하며 표시 직전에만 손실 문자열 변환을
허용한다.

### 2.4 실제 layout과 worker

현재 layout은 render와 navigation 모두
`calculate_for_entries(viewport, settings, entry_count)`라는 같은 순수 계산 계약을
사용한다. 현재 코드는 navigation과 draw 시점에 각각 다시 계산하지만 동일 state 입력에는
동일한 metrics가 나오며 별도 공식을 두지 않는다. Auto mode는 항목 수가 한 열에 들어오면 한 열을 전체 폭으로
확장하고, 필요할 때만 폭이 허용하는 최대 열 수까지 늘린다. 열 경계의 마지막 한 셀에는
box-drawing 문자 `│`를 그린다. 정확한 공식과 page 경계 이동은
[ADR-002](../architecture/adr-002-shared-layout-navigation.md)가 단일 계약이다.

현재 `runtime::EffectWorker`는 `std::thread` 하나와 `std::sync::mpsc` request/completion
채널 하나를 사용한다. 세 Effect를 직렬 실행하고 Drop에서 `Stop`을 보낸 뒤 join한다.
이 구조는 M1의 작은 read/launch 범위에는 충분하지만 장시간 mutation, Git read, 복수
Remote session을 같은 queue에 넣는 목표 구조는 아니다.

---

## 3. M1-13 목표: M2 진입 전 기준선 보정

M1-13은 기능 범위를 넓히는 카드가 아니라 현재 구현과 M1 계약을 맞추는 closure다.

1. `EntryMetadata`와 `FileEntry`에 raw modified time과 아래 네 file attribute를 추가한다.
   개별 metadata 실패는 listing 전체 실패가 아니라 unavailable fallback이다.

   ```rust
   pub struct EntryMetadata {
       pub kind: EntryKind,
       pub size: u64,
       pub modified: Option<SystemTime>,
       pub attributes: EntryAttributes,
   }

   pub struct EntryAttributes {
       pub read_only: bool,
       pub hidden: bool,
       pub system: bool,
       pub archive: bool,
   }

   pub struct LocalMinute {
       pub year: i32,
       pub month: u8,
       pub day: u8,
       pub hour: u8,
       pub minute: u8,
   }

   pub trait TimeZonePort: Send + Sync {
       fn local_minute(&self, instant: SystemTime) -> Result<LocalMinute, TimeZoneError>;
   }
   ```

   directory-load worker가 `TimeZonePort`를 호출해 `LocalMinute`를 `FileEntry`에 보존한다.
   production port는 timestamp 시점의 OS local timezone/DST를 적용하고 scenario snapshot은
   `FixedTimeZone` offset을 쓴다. DST unit test에는 timestamp별 `LocalMinute` 또는 오류를
   table로 반환하는 `FakeTimeZone`을 쓴다. render는 포맷된 parts만 읽어 `MM-DD HH:mm` 또는
   `----- --:--`를 그리며 clock/timezone/FS를 호출하지 않는다. raw `SystemTime`은 M2 Editor
   stale-write 검사에 재사용한다.
2. 현재 `ui.rs`에 private로 있는 grapheme/cell-width/말줄임 helper를
   `src/layout/text.rs`로 이동한다. layout, file list, M2 Viewer/Editor가 같은 규칙을
   재사용한다.
3. 기능키 text와 활성 상태는 CommandRegistry에서만 얻는다. disabled command는 별도
   style과 이유를 가지며 기본 힌트는 실제 `Ctrl+Q Quit` 계약과 일치한다.
4. scenario의 `clock`은 `FixedClock`, modified 변환은 별도 `FixedTimeZone`에 실제 주입하고
   기존 DiskInfo completion을 유지한다. named effect completion과 문자+Style snapshot
   경계를 만든다. Clock 값을 파일 modified time으로 대신 사용하지 않는다.

이 단계가 끝나기 전에는 아래 M2 목표 타입을 현재 구현처럼 문서나 테스트에서 참조하지
않는다.

---

## 4. M2 목표 소스 구조

M2는 현재 모듈을 필요에 따라 아래 책임으로 분해한다. 파일 이름은 책임 경계이며 모든
파일을 한 카드에서 일괄 생성하라는 뜻이 아니다.

```text
src/
  main.rs                   terminal 시작/복구와 runtime 호출
  lib.rs                    테스트가 사용할 공개 모듈
  runtime.rs                event loop, Action queue, lane completion 수신
  error.rs                  AppError와 사용자 메시지 변환

  app/
    mod.rs
    action.rs
    effect.rs
    state.rs
    reducer.rs              상태 전이의 유일한 진입점
    command_registry.rs

  input/
    mod.rs
    key.rs
    mapper.rs

  model/
    entry.rs
    directory.rs
    selection.rs
    viewer.rs
    editor.rs
    operation.rs

  layout/
    mod.rs
    metrics.rs
    engine.rs
    navigation.rs
    text.rs                 M1-13에서 추출한 Unicode cell helper

  ui/
    mod.rs
    palette.rs
    components/
    dialogs/
      input.rs
      confirm.rs
      progress.rs
      viewer.rs
      editor.rs
      conflict.rs

  ports/
    filesystem.rs           M2-02 read/mutation capability
    clock.rs
    disk.rs
    launcher.rs
    trash.rs

  adapters/
    real_fs.rs
    memory_fs.rs
    system_clock.rs
    system_disk.rs
    system_launcher.rs
    system_trash.rs

  operations/
    planner.rs
    worker.rs
    copy.rs
    move_entry.rs
    delete.rs

  runtime/
    job.rs                OperationId, CancelHandle/Token, Deadline, JobControl
    lane.rs               bounded non-blocking Core Local submit/backpressure
```

## 5. M2 상태 모델 목표

M2의 상태는 화면별 세부 상태를 명시적으로 소유한다. 아래는 책임 모양을 고정하는 목표
예시이며 실제 필드 추가는 각 카드가 테스트와 함께 수행한다.

```rust
pub struct AppState {
    pub screen: Screen,
    pub directory: DirectoryState,
    pub selection: SelectionState,
    pub viewport: Viewport,
    pub view: ViewSettings,
    pub sort: SortSettings,
    pub dialog: Option<DialogState>,
    pub operation: Option<OperationState>,
    pub message: Option<UserMessage>,
    pub should_quit: bool,
}

pub enum Screen {
    Main,
    Help,
    Viewer(ViewerState),
    Editor(EditorState),
    QuitConfirm,
}
```

- `Action::KeyPressed` 자체가 기능을 수행하지 않는다. mapper가 현재 Screen/Dialog에 맞는
  의도 Action으로 변환한다.
- dialog가 열려 있으면 modal input이 main command를 가로챈다.
- load/operation result는 request 또는 generation identity를 확인한 뒤 적용한다.
- refresh 후 selection/mark 보존은 경로 identity를 기준으로 결정적으로 처리한다.

## 6. M2-02 파일 I/O·mutation port 목표

M2-02 전에는 이 절의 method가 구현됐다고 가정하지 않는다. M2-02는 Viewer/Editor와 파일
작업이 공유할 작은 capability를 Real/Memory adapter에 함께 추가하고 동일 contract suite로
검증한다.

```rust
pub trait FileReader: Send + Sync {
    fn read_dir(&self, path: &Path) -> Result<Vec<FileEntry>, FsError>;
    fn metadata(&self, path: &Path) -> Result<EntryMetadata, FsError>;
    fn symlink_metadata(&self, path: &Path) -> Result<EntryMetadata, FsError>;
    fn open_read(&self, path: &Path) -> Result<Box<dyn Read + Send>, FsError>;
}

pub trait FileMutator: Send + Sync {
    fn create_dir(&self, path: &Path) -> Result<(), FsError>;
    fn rename(&self, from: &Path, to: &Path) -> Result<(), FsError>;
    fn create_temp_near(&self, target: &Path)
        -> Result<(TempFileId, Box<dyn Write + Send>), FsError>;
    fn publish_temp(&self, temp: TempFileId, target: &Path) -> Result<(), FsError>;
    fn remove_file(&self, path: &Path) -> Result<(), FsError>;
    fn remove_dir(&self, path: &Path) -> Result<(), FsError>;
    fn copy_basic_metadata(&self, from: &Path, to: &Path) -> Result<(), FsError>;
}

pub trait FileSystem: FileReader + FileMutator {}
```

구체 signature는 M2-02의 red test가 Rust ownership에 맞게 확정할 수 있지만 다음 책임은
바꾸지 않는다.

- 재귀 순회, overwrite/skip, symlink와 containment 정책은 port가 아니라 planner가
  소유한다.
- 저장은 target 인접 temp write와 atomic publish 경계를 사용한다.
- Memory adapter는 n번째 실패, short write, disk-full, permission, cross-device와 slow I/O를
  결정적으로 재현한다.
- production adapter의 mutation integration test는 TempDir 아래에서만 실행한다.
- UI, reducer와 model은 `std::fs`를 직접 호출하지 않는다.

## 7. LayoutMetrics 계약

렌더와 탐색은 같은 `LayoutMetrics`를 사용한다.

```rust
pub struct LayoutMetrics {
    pub viewport: Rect,
    pub path_bar: Rect,
    pub list: Rect,
    pub item_detail: Rect,
    pub directory_summary: Rect,
    pub message_bar: Rect,
    pub function_bar: Rect,
    pub columns: Vec<Rect>,
    pub rows_per_column: usize,
    pub page_capacity: usize,
    pub too_small: bool,
}
```

`rows_per_column == list.height`, `page_capacity == rows_per_column * columns.len()`이다.
too-small, 빈 목록과 0 capacity를 모든 navigation 함수가 안전하게 처리한다. 공식, separator
cell, Up/Down/Left/Right page crossing은 이 문서에 중복 정의하지 않고
[ADR-002](../architecture/adr-002-shared-layout-navigation.md)를 따른다.

## 8. 작업 실행과 mutation 직렬화

- **현재 M1**: `runtime::EffectWorker` 한 개가 directory/disk/launch Effect를 직렬 실행한다.
- **M2**: Core Local lane에서 local scan과 mutation을 실행한다. mutation은 한 번에 하나만
  실행하고 progress/conflict/cancel completion을 Action으로 돌려보낸다.
- **post-v1**: Git read와 SSH Remote I/O를 Local lane에 합치지 않는다.

`std::thread + mpsc`, lane 소유권, Remote cancel control path, shutdown/join과 Core/Git
mutation lease의 단일 계약은
[ADR-005](../architecture/adr-005-background-work-lanes.md)다. Tokio 또는 thread pool은
측정과 별도 ADR 없이 도입하지 않는다.

M2-08/09가 ADR-005의 neutral `OperationId`, cancel/deadline/JobControl과 기본 capacity 16의
Core Local `LaneSender`를 처음 구현한다. queue full에서는 같은 view refresh만 최신 것으로
coalesce하고 나머지는 UI를 block하지 않고 `Busy`다. Git/Remote는 이 타입을 재정의하지
않고 그대로 재사용한다.

## 9. 오류와 의존 방향

- 라이브러리 계층은 `thiserror` 기반 구체 오류를 반환한다.
- OS 오류는 작업, 안전하게 표시한 경로, 오류 종류를 포함한 사용자 메시지로 변환한다.
- terminal raw mode/alternate screen은 RAII guard와 panic hook으로 복구한다.
- `model`, `layout`은 `ui`, Crossterm event와 실제 adapter를 import하지 않는다.
- `ui`는 filesystem, clock, disk, launcher와 worker를 호출하지 않는다.
- `adapters`는 `ui`를 import하지 않는다.
- `tests/support`가 production 모듈로 역유입되지 않는다.
- `main.rs`에는 composition root와 terminal lifecycle 이외의 비즈니스 로직을 넣지 않는다.

## 10. 의존성 정책

의존성은 해당 카드에서 실제 필요할 때만 추가한다. 표준 라이브러리 대안, 유지 상태,
라이선스, Windows 지원과 binary 영향을 확인하고 이유를 `progress.md`에 기록한다.
`Cargo.lock`을 재현 기준으로 사용한다.

| 범주 | 목적 |
|---|---|
| ratatui, crossterm | TUI와 입력 |
| serde, toml/YAML parser | 설정, 테마, 시나리오 |
| unicode-width, unicode-segmentation | cell width와 grapheme |
| thiserror | 구조화된 오류 |
| time 계열 | 결정적 날짜 포맷 |
| trash/Windows API adapter | 휴지통, launcher, drive |
| insta, tempfile | snapshot과 격리된 integration test |

<a id="git-extension-boundary"></a>
## 11. post-v1 Git built-in 확장 경계

R1 완료 전에는 plugin 추상화나 Git backend를 Core에 미리 추가하지 않는다. G0 이후에도
Core의 `FileEntry`, layout과 reducer에는 Git 전용 타입을 넣지 않는다.

- generic Plugin contribution/effect만 Core가 안다.
- Git read job은 Core Local lane과 분리된 bounded Plugin Read lane을 사용한다.
- Stage/Commit/Branch처럼 worktree/index를 바꾸는 job은 Core 파일 mutation과 같은
  mutation lease를 획득한다.
- Git callback/job panic은 plugin fault로 격리하고 파일 탐색 runtime을 종료하지 않는다.
- SSH Remote Location에서는 Git discover/status job과 동적 decoration/status/view
  contribution을 생성하지 않는다. 정적 Git command는 이유와 함께 disabled일 수 있다.

상세 API는 [Git 아키텍처](../plugins/git/02-architecture.md)와
[ADR-004](../architecture/adr-004-built-in-plugin-boundary.md), 실행 lane은
[ADR-005](../architecture/adr-005-background-work-lanes.md)를 따른다.

<a id="remote-location-boundary"></a>
## 12. post-v1 SSH Remote Location 경계

SSH Remote는 Git plugin이 아니라 Core Location 확장이다. R1 전에는 이 추상화를 미리
추가하지 않는다.

- Remote path를 `PathBuf`의 가상 prefix로 인코딩하지 않는다.
- Location identity와 location 내부 path를 분리한다.
- UI/reducer/navigation은 transport/session을 모르며 모든 I/O를 Effect로 요청한다.
- 연결된 Remote Location마다 직렬 worker/session을 소유해 Local과 다른 Remote를 막지
  않는다.
- cancel은 blocking session job 뒤에 enqueue하지 않고 독립 `CancelHandle` control path로
  전달한다.
- Git built-in은 Remote Location에서 비활성이다.

상세 identity/transport/cache 계약은
[SSH Remote 아키텍처](../remote/02-architecture.md), worker 수명과 취소는
[ADR-005](../architecture/adr-005-background-work-lanes.md)를 따른다.
