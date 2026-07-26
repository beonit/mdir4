# SSH Remote / Remote Drive 아키텍처

이 문서는 S0 이후의 목표 구조다. production dependency와 public type은 `S0-00` ADR gate
승인 후에만 확정한다.

## 1. 불변 경계

```text
InputMapper / CommandRegistry
             │ Action
             ▼
          Reducer ───────────────► RemoteEffect
             ▲                         │
             │ completion              ▼
             └──────────── Runtime / Remote lanes
                                      ├─ per-location serial worker
UI/control ── CancelHandle ────────────┤  (blocking connect/I/O)
                                      └─ completion/progress channel
                                               │
                                        LocationReader/Writer
                                               │
                                      Fake or production adapter
```

- reducer/render/InputMapper는 filesystem, SSH, network, Clock을 호출하지 않는다.
- UI state는 stream/session/child process를 소유하지 않는다.
- blocking worker가 자기 queue에서 cancel command를 받을 때까지 기다리는 구조를 금지한다.
- Local adapter와 Remote adapter는 Location port를 공유하지만 Remote path를 `PathBuf`로
  인코딩하지 않는다.

## 2. 목표 소스 트리

카드가 요구할 때만 파일을 만든다.

```text
src/
  location/
    id.rs                 LocationId, LocationDefinition
    path.rs               LocationPath, PathWithinLocation, RemotePathBytes
    metadata.rs           LocationEntry, lstat/symlink metadata
    capability.rs
    state.rs              view/session/operation state
    reducer.rs
    manager.rs
    config.rs
    ui/picker.rs
    ui/connection.rs
  ports/
    location_reader.rs
    location_writer.rs
    remote_transport.rs
    ssh_host_discovery.rs  # S3 only
  runtime/
    remote_lane.rs
    # OperationId/cancel/deadline/JobControl은 Core runtime 모듈 재사용
  adapters/
    local_location.rs
    fake_remote.rs
    sftp/                  # S0-00 ADR가 고른 하나만
    openssh_hosts.rs       # S3 only
  operations/
    transfer_coordinator.rs
    transfer.rs
    remote_mutation.rs
tests/
  location_model.rs
  remote_backend_contract.rs
  remote_scenarios.rs
  remote_faults.rs
  remote_real_integration.rs
  support/isolated_sshd.rs
  fixtures/remote/
  scenarios/remote/
  snapshots/remote/
```

## 3. identity와 byte-preserving path

```rust
#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub struct LocationId(String); // config의 stable id

pub struct LocationDefinition {
    pub id: LocationId,
    pub name: String,
    pub description: String,
    pub kind: LocationKind,
    pub read_only: bool,
}

pub enum LocationKind {
    Local { root: PathBuf },
    Remote { host_alias: SshHostAlias, root: RemotePathBytes },
}

pub struct LocationPath {
    pub location: LocationId,
    pub path: PathWithinLocation,
}

pub enum PathWithinLocation {
    Local(PathBuf),
    Remote(RemotePathBytes),
}

pub struct EntryId(pub LocationPath);

pub struct RemotePathBytes(Vec<u8>);
pub struct RemotePathDisplay(String);
```

`EntryId`가 Location을 다시 포함한 `LocationPath`와 별도 Location을 함께 가지는 중복 구조는
금지한다. 공통 규칙:

- `LocationId`는 display name이 아니라 immutable config `id`다.
- `RemotePathBytes`는 `/`로 구분되는 절대 protocol bytes를 보존한다.
- normalize/join/parent/root containment는 byte component로 수행한다.
- display는 reversible identity가 아니다. invalid UTF-8 byte는 deterministic `\\xNN`으로
  escape하고 terminal-cell truncation은 별도 단계다.
- adapter request는 항상 `RemotePathBytes`를 받고 display string을 다시 parse하지 않는다.
- backend가 byte fidelity를 제공할 수 없으면 `PathEncodingUnsupported`로 거부한다.

## 4. metadata와 symlink

```rust
pub struct LocationMetadata {
    pub file_type: LocationFileType,
    pub len: u64,
    pub modified: Option<SystemTime>,
    pub permissions: PortablePermissions,
    pub symlink_target: Option<RemotePathBytes>,
}

pub enum LocationFileType {
    File,
    Directory,
    Symlink,
    Other,
}
```

- S1 `read_dir` entry와 `lstat`은 symlink 자체 metadata를 반환한다.
- `stat_follow`는 port 기본 method가 아니다. 후속 command가 명시적으로 필요할 때만 추가한다.
- directory Enter, recursive copy/delete, Viewer는 symlink를 자동 follow하지 않는다.
- display permission과 protocol permission을 분리하며 unsupported field는 `None`이다.

## 5. capabilities와 command enablement

capability type은 단계별로 확장한다. S0 코드는 `READ`만 정의한다. S2-01이 mutation bit
`UPLOAD`/`RENAME`/`MKDIR`/`DELETE`/`SERVER_COPY`를 추가하고 S3-04가 `RESUME`을 추가한다.
아래는 S3 이후의 최종 모양이며, 앞 단계에서 후속 bit나 placeholder를 미리 만들지 않는다.

```rust
bitflags! {
    pub struct LocationCapabilities: u32 {
        const READ        = 1 << 0;
        const UPLOAD      = 1 << 1;
        const RENAME      = 1 << 2;
        const MKDIR       = 1 << 3;
        const DELETE      = 1 << 4;
        const SERVER_COPY = 1 << 5;
        const RESUME      = 1 << 6;
    }
}
```

- Remote Edit capability는 존재하지 않는다.
- effective capability는 backend capability에서 config `read_only` mutation bit를 제거한 값이다.
- command availability는 kind 직접 match가 아니라 effective capability, connection state,
  operation lease로 계산한다.
- backend는 raw capability와 관계없이 `read_only`를 다시 검사한다.

## 6. read/write port의 단계적 확장

S0/S1 read contract:

```rust
pub trait LocationReader: Send {
    fn read_dir(&mut self, path: &LocationPath, ctx: &OperationContext)
        -> Result<Vec<LocationEntry>, LocationError>;
    fn lstat(&mut self, path: &LocationPath, ctx: &OperationContext)
        -> Result<LocationMetadata, LocationError>;
    fn open_read(&mut self, path: &LocationPath, offset: u64, ctx: &OperationContext)
        -> Result<Box<dyn Read + Send>, LocationError>;
}
```

S1 Remote Viewer는 `lstat` length가 32 MiB를 넘으면 `open_read`를 호출하지 않는다. length를
신뢰할 수 없거나 read 중 변하면 Remote lane에서 정확히 `32 MiB + 1 byte`까지만 bounded
`Vec<u8>`로 읽어 TooLarge를 판정하고 buffer를 폐기한다. private temp 파일이나 OS launcher를
사용하지 않으며 cancel/deadline/error에서도 buffer/stream을 worker가 정리한다.

S2에서 base writer contract와 Fake writer를 추가한다. 이 단계에는 resume token/method가
존재하지 않는다.

```rust
pub struct LocationTempHandle {
    pub id: TempId,
    pub writer: Box<dyn TempWrite + Send>,
}

pub trait LocationWriter: Send {
    fn open_write_temp(&mut self, target: &LocationPath, ctx: &OperationContext)
        -> Result<LocationTempHandle, LocationError>;
    fn publish_temp(&mut self, temp: TempId, target: &LocationPath, ctx: &OperationContext)
        -> Result<(), LocationError>;
    fn discard_temp(&mut self, temp: TempId, ctx: &OperationContext)
        -> Result<(), LocationError>;
    fn rename(&mut self, from: &LocationPath, to: &LocationPath, ctx: &OperationContext)
        -> Result<(), LocationError>;
    fn create_dir(&mut self, path: &LocationPath, ctx: &OperationContext)
        -> Result<(), LocationError>;
    fn remove(&mut self, path: &LocationPath, kind: RemoveKind, ctx: &OperationContext)
        -> Result<(), LocationError>;
    fn copy_within(&mut self, from: &LocationPath, to: &LocationPath,
        ctx: &OperationContext) -> Result<(), LocationError>;
}
```

temp lifecycle은 정확히 다음과 같다.

1. 성공은 `open_write_temp → write/flush → writer drop → publish_temp` 순서다.
2. write/flush/cancel/timeout 실패는 writer를 먼저 drop한 뒤 `discard_temp`를 정확히 한 번
   호출한다.
3. writer drop 뒤 publish가 실패해도 `discard_temp`를 한 번 호출한다.
4. endpoint panic은 lane supervisor가 잡고 소유한 TempId cleanup guard로 writer drop 뒤
   `discard_temp`를 실행한다. `Drop` 안에서 blocking I/O를 하거나 오류를 숨기지 않는다.
5. discard 실패는 원래 terminal result를 바꾸지 않고 `CleanupFailed` warning과 safe display
   path를 추가한다. 기존 final target은 덮어쓰거나 삭제하지 않는다.

Reader-only fake가 dummy mutation을 구현하게 하지 않는다. 같은 Remote가 아닌 server-side
rename/copy는 planner에서 거부하고 writer에 전달하지 않는다. `copy_within`은
`SERVER_COPY`가 있을 때만 호출하며 없으면 S2의 bounded stream fallback을 사용한다.

S3-04에서만 다음 extension trait과 token을 추가한다. resume 가능한 session만
`ResumableLocationReadWriter`를 구현하고 `SessionAccess::Resumable`로 반환한다. `RESUME`
capability가 있는데 이 variant가 아니거나, 이 variant인데 capability가 없으면 `Protocol`로
session을 격리한다. downcast, `Option`, S2 dummy method를 쓰지 않는다.

```rust
pub struct SourceFingerprint {
    pub byte_len: u64,
    pub sha256: [u8; 32],
}

pub const RESUME_TOKEN_VERSION: u8 = 1;

pub struct ResumeToken {
    pub version: u8,
    pub source_entry: EntryId,
    pub target: LocationPath,
    pub temp: TempId,
    pub committed_len: u64,
    pub source_fingerprint: SourceFingerprint,
    pub committed_prefix_sha256: [u8; 32],
}

pub struct TempResumeState {
    pub byte_len: u64,
    pub prefix_sha256: [u8; 32],
}

pub trait ResumableLocationWriter: LocationWriter {
    fn inspect_temp(&mut self, token: &ResumeToken, ctx: &OperationContext)
        -> Result<TempResumeState, LocationError>;
    fn open_write_temp_resume(&mut self, target: &LocationPath, token: &ResumeToken,
        ctx: &OperationContext) -> Result<LocationTempHandle, LocationError>;
}

pub trait ResumableLocationReadWriter: LocationReadWriter {
    fn resumable_writer(&mut self) -> &mut dyn ResumableLocationWriter;
}
```

`open_write_temp_resume`는 source와 temp 검증을 통과한 token만 받는다. 첫 전송을 시작하기 전에 source
전체 SHA-256/길이를 계산하고 stream 중 committed prefix SHA-256을 누적한다. resume
전에 token version/source/target/temp identity, 현재 source 전체 SHA-256/길이, Download는
Local temp, Upload는 Remote temp의 정확한 길이와 source/destination committed-prefix SHA-256을
검증한다. `RESUME` 없음은 `Unsupported`,
token identity/version 불일치는 `ResumeTokenInvalid`, source 지문 불일치는
`ResumeSourceChanged`, temp 길이/지문 불일치는 `ResumePartialMismatch`다. 이 네 오류는
base `open_write_temp`로 조용히 바뀌지 않고 사용자가 Restart를 선택하기 전 temp/final target을
바꾸지 않는다. prefix를 다시 읽은 hasher state로 이어지는 전체 source stream SHA-256도
준비한 `source_fingerprint.sha256`과 publish 직전에 같아야 한다. fingerprint/prefix read는
일반 transfer처럼 progress, cancel, deadline을 받는다. `ResumeToken`은 process memory에만
두며 앱 재시작, config, log, snapshot을 넘겨 저장하지 않는다. SHA-256 dependency는
S3-04에서만 추가하고 license/maintenance/binary
영향을 progress에 기록한다.

## 7. 독립 cancel control과 deadline

```rust
pub struct OperationContext {
    pub job: JobControl, // Core M2의 OperationId/CancelToken/monotonic Deadline
    pub view_generation: Option<ViewGeneration>,
    pub session_epoch: SessionEpoch,
}
```

M2 Core runtime이 소유한 `(CancelHandle, CancelToken)`과 `OperationId/Deadline`을 재사용한다.
S0는 새 cancel/operation 타입을 만들지 않고 Remote 전용 view/session identity만 envelope에
추가한다. handle은 control registry에, token은 blocking adapter call에 전달한다. adapter는
다음 중 선택 transport가 지원하는 실제 preemption을 사용해야 한다.

- child/process kill handle을 cancel state에 등록
- socket/session close handle을 별도 thread-safe control object로 노출
- bounded read/write와 token poll

token poll만 하고 무기한 blocking call을 허용하는 구현은 conformance 실패다. deadline은
UI Clock이 아닌 monotonic clock 기준 absolute 값이다. timeout은 runtime이 handle을
cancel한 뒤 `Timeout`, 사용자 cancel은 `Cancelled`로 normalize한다.

## 8. Remote transport 계약

```rust
pub trait RemoteTransportFactory: Send + Sync {
    fn connect(
        &self,
        request: ConnectRequest,
        ctx: &OperationContext,
    ) -> Result<ConnectedRemote, RemoteError>;
}

pub struct ConnectedRemote {
    pub session: Box<dyn RemoteSession>,
    pub control: Arc<dyn SessionControl>,
}

pub trait SessionControl: Send + Sync {
    fn interrupt(&self);
    fn close(&self);
}

pub trait RemoteSession: Send {
    fn capabilities(&self) -> LocationCapabilities;
    fn access(&mut self) -> SessionAccess<'_>;
}

pub trait LocationReadWriter: Send {
    fn reader(&mut self) -> &mut dyn LocationReader;
    fn writer(&mut self) -> &mut dyn LocationWriter;
}

pub enum SessionAccess<'a> {
    ReadOnly(&'a mut dyn LocationReader),
    ReadWrite(&'a mut dyn LocationReadWriter),
    // S3-04에서만 추가한다.
    Resumable(&'a mut dyn ResumableLocationReadWriter),
}
```

S0/S1 reader fake는 `ReadOnly`, S2 writer fake/production session은 `ReadWrite`를 반환한다.
S3 resume-capable session만 `Resumable`을 반환한다. advertised capability와 access variant가
맞지 않으면 `Protocol`로 session을 격리한다. trait object downcast로 writer를 찾지 않는다.

`ConnectRequest`는 LocationId와 alias만 소유한다. cancel/deadline은 공통 context에 있다.
username/password/key/port/resolved hostname을 model에 추가하지 않는다.

S0-00 비교 ADR은 OpenSSH config/Include/ProxyJump, agent, known_hosts, non-interactive failure,
arbitrary-byte path fidelity, lstat, cancel/deadline, session reuse, Windows 배포, license,
maintenance, binary size를 동일 fixture로 비교한다. 어느 후보도 필수 조건을 만족하지 않으면
단계를 blocked로 남기고 계약을 조용히 약화하지 않는다.

## 9. 세 가지 결과 identity

```rust
pub struct ViewGeneration(u64);
pub struct SessionEpoch(u64);

pub struct RemoteState {
    pub active_location: LocationId,
    pub path: LocationPath,
    pub view_generation: ViewGeneration,
    pub sessions: BTreeMap<LocationId, SessionState>,
    pub operations: BTreeMap<OperationId, OperationState>,
    pub last_visible_listing: Option<LastVisibleListing>, // S1
    pub directory_cache: Option<DirectoryCache>,          // S3
}
```

- path 변경/manual refresh마다 view generation 증가. 다른 generation listing은 폐기한다.
- connect/reconnect/session replacement마다 해당 Location의 session epoch 증가. 이전 session
  completion은 현재 connection state를 덮지 못한다.
- transfer/mutation은 화면과 독립된 OperationId를 가진다. progress는 operation에만 적용하고
  terminal result와 cleanup warning은 화면 이동/refresh 뒤에도 보존한다.
- operation이 끝났다는 이유로 현재 view를 무조건 바꾸지 않는다. 영향받은 directory가
  현재 view이면 generation-aware refresh 1회를 요청한다.

## 10. worker topology와 cleanup

```text
RemoteRuntime
├─ control registry: OperationId → CancelHandle, LocationId → SessionControl
├─ DEV serial worker: connect/list/stat/read/write
├─ PROD serial worker: connect/list/stat/read/write
└─ completion/progress receiver (progress ≤20 Hz)
```

- Remote Location별 직렬 worker/session owner 하나를 기본으로 한다.
- Location별 queue 기본 16, active Remote worker 기본 4를 사용한다. full refresh는 최신
  generation 하나로 coalesce하고 그 밖은 `Busy`, 다섯 번째 Location connect는
  `LimitReached`다. UI submit은 `try_send`이며 block하지 않는다.
- Local mutation lane 및 Git plugin lane과 분리해 Remote 지연이 Local을 막지 않는다.
- Remote-only mutation은 해당 Location lane이 직렬화한다. transfer가 local target을 쓰거나
  local source를 삭제할 때만 local 쓰기/삭제 구간 전에 Core/Git 공통 mutation lease를
  non-blocking으로 획득한다. active lease면 queue/wait 없이 `Busy` terminal result를 내고
  local mutation backend 호출은 0회다.
- app quit/Location 삭제/reconnect는 queue 밖 handle로 cancel하고 session/stream/child를
  close한 뒤 sender를 닫고 worker를 정상 join한다. `std::thread::join` timeout이나 thread
  detach/leak fallback은 사용하지 않는다.
- join이 끝날 수 있도록 모든 blocking adapter call은 cancel/close와 deadline으로 유한하게
  종료되어야 한다. cleanup 실패는 숨기지 않고 terminal warning으로 남긴다.
- 측정 전 Tokio/thread pool을 도입하지 않는다. std thread+channel 기준이며 변경은 ADR 대상이다.

## 11. S1 last-visible listing과 S3 cache

```rust
pub struct LastVisibleListing {
    pub location: LocationId,
    pub path: RemotePathBytes,
    pub listing: DirectoryListing,
}

pub struct DirectoryCacheKey {
    pub location: LocationId,
    pub path: RemotePathBytes,
}

pub struct DirectoryCacheEntry {
    pub listing: DirectoryListing,
    pub loaded_at: MonotonicInstant,
}
```

S1 value는 현재 화면의 연결 끊김 표시만 위한 단일 snapshot이다. lookup API, TTL, LRU,
재방문 backend-call 절감이 없다. S3 `DirectoryCache`만 injected Clock, TTL, deterministic LRU,
manual bypass, mutation invalidation을 가진다. session reuse와 양쪽 cache를 혼동하지 않는다.

## 12. transfer planner

```rust
pub enum TransferRoute {
    LocalToRemote,
    RemoteToLocal,
    SameRemote,
    DifferentRemoteUnsupported,
}
```

planner는 immutable plan에 source/target LocationPath, capability, RO, containment, conflict,
temp/publish strategy, symlink policy를 담는다. worker는 plan 밖에서 path를 조합하지 않는다.

Local↔Remote는 새 전역 I/O worker를 만들지 않고 `TransferCoordinator`가 두 기존 lane을
연결한다.

```text
Remote endpoint job (Remote Location lane)
        │  bounded chunks: capacity 2 × 256 KiB
        ▼
TransferCoordinator (OperationId/control/result만, I/O 없음)
        │
        ▼
Local endpoint job (Core Local lane + 필요한 MutationLease)
```

- Remote lane은 Local FileSystem port를 호출하지 않고 Core Local lane은 SSH/session을
  호출하지 않는다.
- Download는 Core endpoint가 non-blocking lease를 얻고 local sibling temp를 만든 뒤
  `Ready`를 보낸 경우에만 Remote read job을 시작한다. Busy면 Remote/Local transfer backend
  call 0이다. lease는 publish/cleanup terminal까지 Core endpoint가 소유한다.
- Upload는 Core endpoint가 local source를 읽고 Remote endpoint가 temp/write/publish한다.
  Copy는 local mutation lease가 필요 없고, Local→Remote Move의 local source delete만 별도
  lease를 non-blocking으로 얻는다. Remote→Local Move는 download publish 뒤 Remote lane에서
  source를 permanent delete한다. 어느 방향이든 delete Busy/capability/error면 destination
  성공/delete 미실행 partial result이며 성공한 destination을 자동 rollback하지 않는다.
- 한 endpoint error/cancel/deadline/panic은 공통 control handle로 반대 endpoint와 chunk
  channel을 닫는다. coordinator는 두 endpoint cleanup을 모아 terminal result를 정확히 한
  번 보낸다. UI thread는 chunk send/receive나 join을 하지 않는다.
- 같은 Remote의 server copy/stream fallback은 한 Location lane 안에서 실행하고 cross-Remote는
  job 제출 전에 거부한다.

- upload: remote sibling temp → stream → publish rename.
- download: local sibling temp → stream → Core atomic publish.
- same-remote Move: rename 우선; fallback copy+delete는 별도 확인.
- cross-Remote: effect/backend call 0.
- progress byte는 실제 stream committed byte만 단조 증가한다.

## 13. 오류와 redaction

```rust
pub enum RemoteErrorKind {
    InvalidConfig,
    UnknownLocation,
    InteractiveAuthRequired,
    HostKeyRejected,
    Timeout,
    Cancelled,
    Busy,
    LimitReached,
    TooLarge,
    Disconnected,
    InvalidPath,
    PermissionDenied,
    NotFound,
    AlreadyExists,
    NotEmpty,
    NoSpace,
    ReadOnly,
    Unsupported,
    PathEncodingUnsupported,
    Protocol,
    Io,
    CleanupFailed,
    ResumeTokenInvalid,
    ResumeSourceChanged,
    ResumePartialMismatch,
}
```

오류는 LocationId, OperationId, kind, safe display path만 보존한다. resolved endpoint,
credential, key path, environment, raw process command는 `Debug`/Display/log/snapshot에서
redact한다.

## 14. Git과 공통 integration rule

- `LocationKind::Remote`이면 Git discover/backend job과 동적 decoration/status/view
  contribution 생성 수는 0이다. 정적 command availability query는 backend 없이 disabled
  reason만 반환할 수 있다.
- Git이 먼저 구현되어도 Remote가 먼저 구현되어도 같은 source-boundary/integration test를
  통과해야 한다.
- Core Local/Git mutation과 Remote transfer의 local endpoint 쓰기/삭제 구간만 Core mutation
  lease를 공유한다. 획득은 non-blocking이며 active lease면 `Busy`다. Remote-only mutation은
  Location별 lane 직렬화로 보호한다.
- Remote Git은 별도 ADR가 transport, executable, credential, cancellation을 정의하기 전
  capability에 추가하지 않는다.
