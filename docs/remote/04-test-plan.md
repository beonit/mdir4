# SSH Remote / Remote Drive 테스트 계획

## 1. 원칙

1. model/reducer/UI/operation은 단계에 맞는 `FakeRemote` 계약으로 먼저 검증한다.
2. 기본 `cargo test --locked`는 network, 실제 home, SSH agent, known_hosts를 읽지 않는다.
3. production adapter는 임시 home/config/key/known_hosts를 가진 isolated `sshd`에서 검증한다.
4. test도 host-key verification을 끄지 않는다.
5. reducer/render/InputMapper의 backend/network/Clock 호출 수는 0이다.
6. 지연/fault test는 sleep이 아니라 controlled gate와 명시적 completion을 사용한다.
7. mutation은 before/after, terminal result, temp/session/stream/child leak를 함께 검사한다.
8. 모든 identity assertion은 display name이 아니라 stable LocationId/protocol bytes를 사용한다.

## 2. 계층과 단계

| 계층 | 대상 | backend | 최초 단계 |
|---|---|---|---|
| Unit | identity/path/config/cancel/state/capability/planner | 없음 | S0 |
| Read contract | connect/read_dir/lstat/open_read | Fake read + real | S0/S1 |
| Write contract | temp/publish/rename/mkdir/remove | Fake writer + real | S2 |
| Component | picker/Main/status/dialog/Viewer | 단계별 Fake | S0~S2 |
| Scenario | key→effect→completion→snapshot | 단계별 Fake | S0~S3 |
| Integration | auth/connect/read/write/cancel | isolated `sshd` | S0 harness, S1/S2 adapter |
| Fault | timeout/disconnect/partial/cancel/cleanup | controlled Fake/isolated proxy | S0~S2 |
| Manual | Windows OpenSSH/agent/ProxyJump/packaging | 전용 test host | S1~S3 gate |

## 3. S0/S1 Fake read 계약

```rust
FakeRemote::read_builder()
    .location("dev", remote_root_bytes())
    .connect_ok("dev")
    .listing("dev", b"/home/dev", basic_listing())
    .lstat(b"/home/dev/link", symlink_metadata())
    .read_bytes(b"/home/dev/readme", b"hello")
    .gate(RequestKind::ReadDirectory, ControlledGate::new())
    .disconnect_on_call(4)
    .build()
```

필수 기록/제어:

- stable LocationId, protocol path bytes, OperationId, view generation/session epoch, deadline
- call order/thread, session create/reuse/close, active stream/callback count
- connect/read_dir/lstat/open_read result
- controlled block/unblock, out-of-order completion, disconnect, partial read, cancel/timeout,
  nth-call panic과 해당 Location session-only isolation

S0/S1 Fake public API에는 write/publish/rename/mkdir/remove/partial upload가 없어야 한다.

## 4. S2 Fake writer 확장

```rust
let fake = FakeRemote::write_builder(read_fixture)
    .write_temp_ok(target_bytes())
    .publish_ok()
    .rename_ok()
    .mkdir_ok()
    .remove_ok()
    .partial_write(4096, RemoteErrorKind::Disconnected)
    .cleanup_error(RemoteErrorKind::CleanupFailed)
    .build();
```

추가 기록:

- committed bytes, temp id/path, publish/rename/remove exact order
- read-only direct call rejection, capability mismatch
- active writer/temp artifact count, cleanup success/failure
- `SessionAccess` ReadOnly/ReadWrite와 advertised capability 일치
- `SERVER_COPY`의 `copy_within`/stream fallback

### S3 resume Fake 확장

S3-04에서만 Fake에 `RESUME`과 `SessionAccess::Resumable`을 추가한다. ResumeToken은
version/source/target/temp identity, source 전체 SHA-256/길이, Download Local temp·Upload
Remote temp 각각의 partial 길이/committed-prefix SHA-256과 offset을 각각 검증한다.
mismatch별 `ResumeTokenInvalid`/`ResumeSourceChanged`/`ResumePartialMismatch`, capability 없음의
`Unsupported`, 모든 실패에서 restart/publish 0회를 assert한다. fingerprint/prefix 단계
cancel/deadline과 stream 중 source 변경의 final hash mismatch도 publish 0회여야 한다.

Fake 자체 contract를 먼저 test하고 UI/scenario가 내부 map을 직접 수정하지 않게 한다.

## 5. identity/path/config 표

| 입력 | 기대 |
|---|---|
| `dev` display name `DEV→Development` 변경 | LocationId/session/cache/marks identity 유지 |
| DEV와 PROD의 같은 `/a` bytes | 서로 다른 EntryId |
| `/a//b/./c` | component normalize |
| root 아래 `/a/../b` | root 안에서 normalize |
| root 밖 `../../` | backend 호출 전 거부 |
| Unicode/space/leading `-`/newline | exact byte roundtrip, safe display |
| invalid UTF-8 bytes | `\\xNN` display; exact protocol mutation 또는 explicit unsupported |
| 서로 다른 byte path의 동일 display 가능성 | EntryId는 다르고 잘못된 target 호출 0 |

Config matrix:

- S0 timeout defaults와 S3 cache defaults, 각각 최소/최대/범위 밖 및 migration
- stable id slug/duplicate, mutable name exact-duplicate/trim/width/control, description width/control
- Host alias 공백/control/선행 dash/URI credential
- UTF-8 `root`/non-UTF-8 `root_hex` 상호 배타, absolute/relative/empty root
- `read_only` omitted/true/false
- forbidden username/password/private_key/port/HostName/ProxyJump/ProxyCommand
- legacy config without `[remote]`, unknown field의 index+field diagnostic

## 6. cancel/deadline preemption

다음 test는 “worker queue에 cancel command를 넣는 것”만으로 통과할 수 없다.

1. connect가 controlled block 안에 진입한다.
2. UI/control thread가 queue 밖 `CancelHandle::cancel()`을 호출한다.
3. adapter block이 풀리기 전에 cancel/close callback이 관측된다.
4. worker가 제한 시간 안에 `Cancelled` terminal result를 반환한다.
5. session/stream/child/callback count가 0이다.

동일 절차를 connect/read_dir/lstat/open_read/write/publish에 단계별 적용한다. deadline test는
injected monotonic deadline을 넘겨 `Timeout`을 반환하며 user cancel과 구분한다. cancel
before start, during call, after terminal, double cancel, cancel-vs-timeout race를 포함한다.

## 7. view/session/operation interleaving

필수 scenario:

1. DEV view load 시작 → 다른 path load 성공 → 첫 listing 폐기.
2. DEV connect epoch 1 시작 → reconnect epoch 2 성공 → epoch 1 성공/실패 폐기.
3. download Operation A 시작 → Local 화면 전환 → A terminal result/history와 cleanup 보존.
4. transfer 중 session loss → reconnect → old progress 폐기하되 operation terminal result 보존.
5. current directory mutation 성공 → 영향 view에 refresh 정확히 1회.
6. app shutdown → handle cancel/session close/sender close → blocking call의 finite 종료 → normal
   join → leak 0. join timeout이나 detached thread fallback은 0회다.

각 단계에서 visible Location/path, view generation, session epoch, operation status, fake call
count를 assert한다.

## 8. metadata/symlink/backend conformance

Fake와 production adapter 공통 S1 read suite:

- connect success, unknown alias, interactive-auth-required, host-key rejection, timeout
- empty/basic/10,000 listing과 directories-first stable sort
- lstat file/directory/symlink/broken symlink/other, permission/not-found
- open_read empty/small/large/offset, Unicode/space/leading dash/invalid UTF-8 policy
- Viewer metadata `>32 MiB` open_read 0, unknown/changing length에서 `32 MiB + 1 byte` bounded
  memory TooLarge, private temp/OS launcher 0, cancel buffer/stream cleanup
- symlink를 directory Enter/Viewer/recursive operation에서 자동 follow하지 않음
- configured root containment와 `..` stop
- session reuse/reconnect replacement/close/drop/cancel/deadline

S2 write suite:

- temp write/flush/writer-drop/publish와 error/cancel/panic의 discard 순서, discard failure warning
- rename, mkdir, remove file/tree no-follow policy
- invalid path/overwrite/exists/not-empty/no-space/permission/read-only/disconnect/partial/cleanup
- same LocationId validation, cross-Remote backend call 0
- `SessionAccess` mismatch Protocol, SERVER_COPY on/off call path

S3 resume suite:

- valid/mismatched token, resumable access/capability mismatch, unsupported explicit Restart/Cancel
- source full/prefix and route destination temp hash/length mismatch, publish/restart 0

## 9. command/UI/snapshot matrix

snapshot 이름:

```text
remote_<screen>__<fixture>__<width>x<height>__<state>.snap
```

화면:

- picker: local+registered remote, RO, empty, error; MCD-context F3 Drive/F12 Locations 표시
- Main: Connecting/empty/basic/Unicode/non-UTF8/RO/Failed/Disconnected-last-result/error
- Viewer: loading/text/binary/too-large/disconnected/cancel
- transfer: destination/conflict/progress/cancel/result/cleanup warning
- Rename/MkDir/Delete permanent confirmation/read-only reason
- S3 cache states, Host browser, registration form

폭 60/80/81/100/120/160, 높이 15/25/40, `59×14` too-small을 대표 fixture에 배분한다.

각 snapshot/mapper test에서 확인:

- `DEV:/path`, `[RO]`, connection/last-result/cache state가 구분됨
- resolved host/IP/인증 username/private-key path/secret 없음. 등록된 root/path에서 만든 안전한
  display path는 유지됨
- Main F3 View, MCD F3 Drive, Viewer/Dialog F3 context가 충돌하지 않음
- regular-file Enter/F3는 Viewer; OS launcher/process spawn 0
- Remote file F4 disabled와 `Remote editing is not supported.` 이유
- irreversible Delete 문구가 Local trash와 구분됨

## 10. scenario schema

Remote scenario는 Core scenario v1의 기존 필드를 유지하고 아래 optional field/action을
runner가 실제로 지원한 뒤 사용한다. 기존 v1 fixture/parser는 수정 없이 그대로 통과해야
하며, 아직 지원하지 않는 Remote field/action은 계속 unknown-field/action 오류를 내야 한다.

```yaml
version: 1
terminal: { width: 80, height: 25 }
start_path: /local
filesystem:
  - { path: /local, kind: directory }
remote_locations:
  - { id: dev, name: DEV, host: dev, root_hex: 2f686f6d652f646576,
      read_only: false, fixture: remote-basic }
clock: "2026-01-02T03:04:05Z"
disk: { free_bytes: 12288 }
steps:
  - { action: start }
  - { action: open_locations }
  - { action: key, key: down }
  - { action: key, key: enter }
  - { action: complete_remote_effect, effect: connect }
  - { action: complete_remote_effect, effect: read_directory }
  - { action: snapshot, name: remote-root }
  - action: assert
    location_id: dev
    view_generation: 1
    session_epoch: 1
    active_sessions: 1
assertions: { path: /home/dev, location_id: dev, selected: 0, marked: 0 }
snapshots: [remote-root]
```

필수 additive extension:

- top-level `remote_locations`와 final assertion의 optional `location_id`
- `complete_remote_effect`, `fail_remote_effect`, `block_remote_effect`,
  `unblock_remote_effect`
- `cancel_operation`, `disconnect_remote`, `reconnect_remote`, `advance_clock`
- `assert` action: location/path bytes/view generation/session epoch/operation/pending/calls/artifacts

모든 key는 실제 InputMapper를 거친다. runner가 지원하지 않는 field/step을 문서 예시에만
추가하지 않는다. schema 추가 카드가 runner implementation+unknown-field test를 함께 낸다.

## 11. S1 last-result와 S3 cache test 분리

S1:

- disconnect 후 현재 path의 마지막 listing 표시
- 다른 path 이동/재방문은 backend call 1
- TTL/LRU/cache hit API와 `[Cached]` 표시 없음
- reconnect는 새 session epoch와 network load

S3:

- cache off/default/hit/miss/expiry/deterministic eviction
- hit backend calls 0, manual R/Ctrl+R calls 1
- mutation invalidation과 current-view refresh 1
- stale/expired/cache-hit snapshot, injected Clock만 사용

## 12. read-only/capability/Editor matrix

| 작업 | command/UI | reducer effect | planner | backend direct |
|---|---:|---:|---:|---:|
| Edit/Save | 항상 disabled | 0 | Unsupported | method 없음 |
| RO Upload/Move/Rename/MkDir/Delete | disabled | 0 | ReadOnly | ReadOnly |
| capability 없는 mutation | disabled | 0 | Unsupported | Unsupported |
| Browse/Lstat/View/Download/Refresh | enabled if connected | 허용 | 허용 | 허용 |

비활성 이유 우선순위와 Help/status label도 test한다.

## 13. transfer/fault matrix

| Route/command | 최초 단계 | Conflict | Cancel/timeout | Disconnect | RO | Cleanup |
|---|---|---|---|---|---|---|
| Local→Remote F5 Copy | S2 | six choices | 필수 | partial | target RO 금지 | remote temp |
| Local→Remote F6 Move | S2 | six choices | 필수 | partial | target RO 금지 | remote temp; publish 뒤 local source delete |
| Remote→Local F5 Copy/Download | S1 | six choices | 필수 | partial | source RO 허용 | local temp |
| Remote→Local F6 Move | S2 | six choices | 필수 | partial | source RO 금지 | local temp; publish 뒤 remote source delete |
| same LocationId Remote F5 Copy | S2 | six choices | 필수 | partial | RO 금지 | remote temp/server copy |
| same LocationId Remote F6 Move | S2 | six choices | 필수 | partial | RO 금지 | rename 또는 remote temp+delete |
| 서로 다른 Remote F5/F6 | `Unsupported` | 해당 없음 | 해당 없음 | 해당 없음 | 해당 없음 | backend calls 0 |

progress는 0/1/chunk boundary/>4 GiB를 포함하고 committed bytes만 단조 증가, 최대 20 Hz를
검사한다. 화면 이동 뒤 terminal result, cleanup warning, current-view refresh 조건도 assert한다.

Local↔Remote bridge는 injected chunk capacity 1로 backpressure를 만들고 Remote lane의 Local FS
calls 0, Core lane의 SSH calls 0을 thread/call recorder로 assert한다. Download는 local lease
Ready 전 Remote calls 0, 한 endpoint failure/panic은 반대 endpoint cancel과 terminal 1회,
cleanup 뒤 channel/temp/lease/session leak 0이어야 한다.

## 14. isolated `sshd` harness와 gate

기본 공통 gate에는 network test가 없다. S0-08이 Cargo feature `remote-integration`을 만들고
`tests/remote_real_integration.rs` 전체를 그 feature로 compile-gate하며 각 test는
`#[ignore = "requires isolated sshd"]`다. S0/S1/S2 stage gate와 전용 Ubuntu CI job은 정확히
다음을 실행한다.

```text
cargo test --locked --features remote-integration --test remote_real_integration -- --ignored --test-threads=1
```

feature를 켠 compile-only 결과나 ignored 0개 실행은 통과 증거가 아니다. test output의 실행
개수, fixture 방식(container/process), teardown counters와 CI URL을 progress에 기록한다.

- disposable container/process, TempDir home/config/key/known_hosts, 격리 root를 사용한다.
- fixture host key를 known_hosts에 등록하고 verification을 유지한다.
- password/keyboard-interactive disabled와 prompt-required failure를 분리한다.
- changed host key, permission, disconnect, latency, invalid-byte path fixture를 제공한다.
- mutation/delete 전 target이 fixture root 안인지 harness 자체가 assert한다.
- teardown 후 remote file/session/stream/child/container leak 0을 assert한다.
- harness test가 실패하면 production adapter S1/S2 integration을 실행하지 않는다.
- ProxyJump는 isolated two-hop fixture가 있으면 자동, 없으면 S1/S3 수동 미검증 행으로 남긴다.

## 15. 보안, Git 경계, 성능

보안 scan 대상:

- config serialized form, error Debug/Display/log/panic
- snapshots/CI artifact/process argument recording
- host-key disable option, credential field/widget/source identifier

Git integration은 구현 순서 양쪽을 test한다.

- Git만 활성 + Local: 기존 Git contract 유지
- Git 활성 + Remote 진입: discover/job/dynamic contribution/backend calls 0, 정적 command는
  disabled reason 표시
- Remote 먼저 + 나중에 Git 등록: 동일 zero-call
- Core Local/Git과 Remote transfer local endpoint가 active common lease를 만나면 wait/queue
  없이 Busy, local mutation calls 0; Remote-only 작업은 Location lane에서 직렬, 서로 다른
  Location/Local UI는 non-blocking

Git 트랙이 아직 없으면 S0의 generic unsupported/source-boundary test만 실행하고 Evidence에
`counterpart: N/A — Git not implemented`를 기록한다. Git이 추가된 뒤에는 그 시점의 새
`INTEG` regression ID에 위 실제 통합 증거를 기록하며 이전 단계의 Passed 행을 다시 열지
않는다.

자동 성능:

- 10,000 entry byte normalize+sort+layout+render release smoke 목표 100 ms
- blocked network 중 1,000 navigation/resize/help action과 key→frame 목표
- progress 초당 20회 이하
- cache hit 0 calls/manual refresh 1 call(S3)

Windows 수동:

- Windows 10/11 각각에서 Windows Terminal의 PowerShell profile, standalone PowerShell
  console host, standalone CMD console host
- built-in OpenSSH config/Include/agent/known_hosts rejection/ProxyJump(ADR 범위)
- offline/connection loss, 1 GiB upload/download cancel, packaging dependency
- Remote 오류 후 Local 탐색/종료/terminal 복구
