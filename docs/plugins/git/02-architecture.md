# Git built-in 아키텍처

## 1. 경계와 소유권

```text
App Runtime / Reducer
        │ HostEvent, PluginCommand
        ▼
  PluginManager ───────── generic contribution ───────► Renderer/Registry
        │
        ▼
    GitPlugin ── GitState/GitCache/UI
        │
        ├── GitReadBackend       (G1)
        ├── GitMutationBackend   (G2)
        └── GitTransportBackend  (G3)
```

Core의 `FileEntry`, layout, renderer는 Git status/backend crate를 모른다. composition root만
built-in factory를 알고, G0에서는 `FakePlugin`만 등록한다. Git factory/config/commands는
G1-00에서 처음 추가한다. Git은 `Local Location`에서만 활성 contribution을 만든다.

G0의 host event는 Remote 타입에 의존하지 않는 generic path capability를 제공한다.

```rust
pub enum HostPathContext {
    Local { directory: NormalizedLocalDirectory },
    Unsupported { kind: HostLocationKindId },
}
```

Git만 먼저 구현해도 `Local`만 사용하면 된다. 이후 S0가 Remote를 추가하면 이를
`Unsupported`로 매핑하며 Git API에 Remote path/LocationId를 넣지 않는다.

## 2. 목표 소스 트리

```text
src/plugins/
  api.rs                 generic Result/error/contribution/effect 계약
  manager.rs             등록, fault boundary, toggle, view ownership
  worker.rs              별도 plugin-read lane
  git/
    config.rs
    model.rs
    state.rs
    reducer.rs
    backend.rs            GitReadBackend
    fake_read_backend.rs
    history_backend.rs    G2: GitHistoryBackend/FakeGitHistoryBackend
    discovery.rs
    decoration.rs
    status_summary.rs
    ui/{status_view,diff_view}.rs
    real_backend/         backend ADR 뒤 생성
    local/                G2: mutation port/fake/UI
    transport/            G3: auth/transport fake와 real adapter
      worker.rs           bounded single Git Transport lane
```

카드가 요구하기 전 파일/trait을 만들지 않는다. 특히 G1 fake가 mutation을 구현하게 하지 않는다.

## 3. generic plugin API

Rust object-safety를 위한 작은 형태 변경은 가능하지만 아래 책임은 유지한다.

```rust
pub trait Plugin: Send {
    fn id(&self) -> PluginId;
    fn set_enabled(&mut self, enabled: bool)
        -> Result<Vec<PluginEffect>, PluginError>;
    fn on_host_event(&mut self, event: &HostEvent)
        -> Result<Vec<PluginEffect>, PluginError>;
    fn handle_result(&mut self, result: PluginResult)
        -> Result<Vec<PluginEffect>, PluginError>;
    fn handle_command(&mut self, command: PluginCommand)
        -> Result<Vec<PluginEffect>, PluginError>;

    fn decorations(&self, entry: &FileEntry)
        -> Result<Vec<FileDecoration>, PluginError>;
    fn status_items(&self, width: u16)
        -> Result<Vec<StatusItem>, PluginError>;
    fn commands(&self, context: CommandContext)
        -> Result<Vec<CommandContribution>, PluginError>;
    fn view(&self) -> Result<Option<PluginView>, PluginError>;
}
```

- callback은 cache/state만 읽거나 바꾸며 I/O를 하지 않는다.
- manager는 정확한 callback 호출 하나만 `catch_unwind` 경계로 감싼다.
  `AssertUnwindSafe`는 안전 불변식을 설명하고 해당 경계를 test한 경우에만 좁게 사용한다.
- `Err` 또는 panic이 난 plugin은 session `Faulted { redacted_reason }`가 된다. manager는
  contribution과 pending result를 버리되 다른 plugin/Core를 계속 실행한다.
- 사용자가 faulted plugin을 다시 enable하면 old generation/instance를 폐기하고 factory로
  새 instance를 만든다. 단순 flag 변경으로 오염된 state를 재사용하지 않는다.
- `PluginError`와 사용자 메시지는 payload/debug secret을 포함하지 않는다.

## 4. contribution과 collision 규칙

```rust
pub struct FileDecoration {
    pub slot: DecorationSlot,
    pub text: StyledText,
    pub reserved_cells: u16,
    pub priority: u8,
}

pub struct StatusItem {
    pub id: StatusItemId,
    pub full: Vec<StyledSpan>,
    pub compact: Vec<StyledSpan>,
    pub priority: u8,
}

pub struct StyleRoleId(String); // validated namespaced id, not a closed Core enum

pub struct CommandContribution {
    pub id: CommandId,
    pub context: CommandContext,
    pub default_key: Option<KeyChord>,
    pub label: String,
    pub availability: CommandAvailability,
}

pub enum CommandAvailability {
    Enabled,
    Disabled { reason: String },
}

pub struct PluginView {
    pub id: ViewId,
    pub owner: PluginId,
    pub command_context: CommandContext,
    pub model: PluginViewModel,
}
```

- plugin-owned id와 `StyleRoleId`는 `plugin.<plugin-id>.*` namespace여야 한다. Core의 닫힌
  `ThemeRole` enum에 Git variant를 추가하지 않는다. plugin은 built-in default style map을
  등록하고, 사용자 theme에 role이 없거나 잘못되면 generic plugin decoration fallback을
  사용한다.
- duplicate PluginId는 startup error다. duplicate command/view/status id는 먼저 등록된 것을
  조용히 쓰지 않고 해당 plugin을 fault 처리한다.
- 같은 key collision은 사용자 override가 우선이다. 그다음 active screen context, Core,
  plugin priority/id 순으로 결정하고 Settings에 충돌 이유를 노출한다.
- 동시에 active view 하나만 screen stack을 소유한다. open은 manager가 owner를 기록하고,
  close/result는 같은 owner만 바꿀 수 있다. plugin disable/fault 시 그 view를 안전하게 닫는다.
- `reserved_cells`를 layout 입력에서 먼저 빼며 `text` 실제 폭이 이를 넘으면 contribution을
  오류로 격리한다. Git prefix는 항상 2셀이다.
- filename/cursor/marked style과 decoration span은 서로 다른 cell range다. status item도
  styled span을 보존한 채 full→compact→hidden으로 축약한다.

## 5. effect, payload와 취소

```rust
pub struct PluginEffect {
    pub plugin_id: PluginId,
    pub generation: u64,
    pub request_id: RequestId,
    pub job_kind: JobKind,
    pub control: JobControl,
    pub job: Box<dyn PluginJob>,
}

pub trait PluginJob: Send {
    fn run(self: Box<Self>, context: PluginOperationContext) -> PluginJobOutcome;
}

pub struct PluginOperationContext {
    pub job: JobControl,
    pub progress: ProgressSender,
}

pub struct PluginResult {
    pub plugin_id: PluginId,
    pub generation: u64,
    pub request_id: RequestId,
    pub outcome: Result<PluginPayload, PluginError>,
}
```

`JobControl`, `OperationId`, `CancelHandle/CancelToken`과 monotonic `Deadline`은 Core M2의
`src/runtime/job.rs` 타입을 그대로 사용한다. 이 절은 plugin 전용 취소/deadline 타입을 새로
정의하지 않는다. `ProgressSender`만 plugin owner/generation/request envelope를 덧붙이는
G0 adapter다.

`PluginPayload`는 owner plugin만 downcast/해석할 수 있는 internal sendable payload다. manager는
plugin id/generation/request ownership을 검증한 뒤 전달한다. type mismatch, unknown result,
duplicate terminal result는 오류 기록 후 폐기한다. progress는 request owner/generation이
맞을 때만 적용한다.

`JobControl`의 cancel handle은 worker queue 밖 control path에서도 thread-safe하게 signal할
수 있다. cancel/deadline이 발생하면 가능한 backend 중단을 요청하고, terminal result는 반드시
한 번 보낸다. 즉시 중단할 수 없는 backend result도 generation/request 검증으로 state에
적용되지 않는다.

## 6. background work lane

다음 topology는
[`ADR-005`](../../architecture/adr-005-background-work-lanes.md)의 승인 계약을 따른다.

```text
UI/reducer
  ├── Core/local mutation lane : single FIFO + shared MutationLease
  ├── Plugin read lane         : separate single FIFO (G1 status/diff/log)
  └── Git transport lane       : G3 single FIFO + shared MutationLease

future SSH Remote session      : location별 serial session worker, bounded globally
```

- G0-03의 plugin read lane은 기존 local copy/move/delete lane과 분리한다. 느린 status/diff가
  local file operation을 막지 않는다.
- 처음에는 `std::thread` + channel, lane별 동시 실행 1이다. 측정 없이 Tokio/thread pool을
  추가하지 않는다.
- shutdown은 queue close → cancel/deadline로 blocking call 종료 → join 순서다. orphan
  thread를 남기지 않는다.
- G2 Git mutation은 Core file operation과 같은 `MutationLease`를 받아 local mutation lane에서
  직렬화한다. plugin read는 immutable snapshot만 읽으며 mutation 완료 terminal event 뒤
  `RefreshStatus`를 한 번 요청한다.
- G3-02는 기본 capacity 16의 bounded Git transport lane 하나를 추가한다. resolving/auth와
  Git transport/backend affinity를 이 lane이 소유하고 Core Local/Plugin Read worker에서 network
  I/O를 실행하지 않는다. 한 G3 operation이 Queued 이상이면 다른 G3 mutation command는
  queue에 쌓지 않고 `Busy`다.
- Fetch/Pull/Push/Clone은 auth 뒤 첫 Transferring 직전 공통 `MutationLease`를 non-blocking으로
  얻어 Terminal cleanup까지 보유한다. active lease면 auth preflight는 이미 있었을 수 있지만
  transfer와 local/remote mutation은 0회다. Remote Manage는 config write 전, Conflict는 apply
  전에 얻는다. lease는 cancel/error/panic에서도 RAII로 반환한다.
- conflict-context F5 `Mark Resolved`는 network lane이 아니라 opaque `LocalMutationJob`으로
  Core Local lane에 제출하며, 일반 Status F5의 conflicted-row 차단과 별도 command id/state를
  사용한다.

## 7. Git state와 cache

```rust
pub struct GitState {
    pub enabled: bool,
    pub generation: u64,
    pub active_local_directory: Option<NormalizedLocalDirectory>,
    pub active_repository: Option<RepositoryInfo>,
    pub discovery: DiscoveryState,
    pub cache: Option<GitCache>,
    pub refresh: RefreshState,
    pub active_view: GitView,
    pub message: Option<GitMessage>,
}

pub struct GitCache {
    pub repository: RepositoryIdentity,
    pub branch: BranchDisplay,
    pub statuses: BTreeMap<RepoRelativePath, DetailedGitStatus>,
    pub updated_at: Timestamp,
}
```

- `RepositoryIdentity`는 canonical metadata/worktree identity이며 display path와 분리한다.
- `HostPathContext::Unsupported`에서는 active directory/repository/cache와 동적
  decoration/status/view contribution이 없고 discover/job도 제출하지 않는다. 정적으로
  등록된 command definition의 availability만 pure하게 `Disabled`로 계산한다.
- discovery cache는 `NormalizedLocalDirectory → DiscoveryOutcome`이다. outer 결과를 방문하지
  않은 child에 상속하지 않아 nested repository를 놓치지 않는다.
- current directory의 row decoration만 cache에서 filter한다.
- `Timestamp`는 injected Clock에서 worker result에 기록하고 render에서 읽지 않는다.
- `RefreshAll`은 exact-directory discovery entry를 무효화하고 discover 후 snapshot한다.
  `RefreshStatus`는 current identity snapshot만 다시 읽는다.

## 8. read backend와 DiffTarget

```rust
pub trait GitReadBackend: Send + Sync {
    fn discover(&self, directory: &NormalizedLocalDirectory, ctx: &PluginOperationContext)
        -> Result<DiscoveryOutcome, GitError>;
    fn snapshot(&self, repository: &RepositoryInfo, ctx: &PluginOperationContext)
        -> Result<GitSnapshot, GitError>;
    fn diff(
        &self,
        repository: &RepositoryInfo,
        row: &StatusRow,
        target: DiffTarget,
        ctx: &PluginOperationContext,
    ) -> Result<GitDiff, GitError>;
}
```

G1 trait에는 mutation/network method가 없다. `StatusRow`는 stable identity, new path, optional
old path, detailed index/worktree state를 가져 renamed/deleted diff를 모호하지 않게 한다.
DiffTarget의 기본 선택과 Combined 순서는 제품 계약을 따른다.

실제 backend 선택 spike는 gix/git2/CLI의 Windows 배포, nested/worktree, Clean+7 statuses,
diff, deadline/cancel, license, binary size를 같은 fixture로 비교한다. 선택하지 않은 dependency와
spike production code는 제거한다.

## 9. G2/G3 capability 분리

```text
FakeGitReadBackend       G1 discover/snapshot/diff만
FakeGitMutationBackend   G2 stage/commit/branch/stash/discard만
FakeGitHistoryBackend    G2 log page/commit detail/diff만
FakeGitTransportBackend  G3 auth/network/progress/conflict만
```

real adapter도 동일한 capability 경계를 유지한다. read-only configuration이면 mutation command는
`Backend is read-only.` 이유로 unavailable이다.

G3 transport adapter는 raw credential bytes를 앱에 반환하지 않는 OS helper/agent 경계,
host-key callback의 fingerprint-only payload, phase별 cancellation을 제공해야 한다. remote/auth
코드는 G1/G2 module에 들어가지 않는다.

## 10. 금지 의존성

- Core model/layout → `plugins::git` 또는 Git backend crate
- renderer → Git backend/state direct read
- plugin callback/reducer/render → filesystem/process/network/clock
- backend → app/ui/ratatui
- fake read backend → mutation/transport backend
- G1/G2 → credential/remote network module
- shell string 실행, raw secret state/logging, render-triggered refresh
