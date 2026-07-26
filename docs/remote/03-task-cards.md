# SSH Remote / Remote Drive 작업 카드

## 실행 규칙

- 카드 ID 순서대로 진행한다. 선행 카드 증거가 없으면 다음 카드를 시작하지 않는다.
- 한 카드에서 fake/model과 production network/UI를 동시에 처음 만들지 않는다.
- 모든 카드의 마지막 두 checklist를 채우고
  [`../implementation-plan/progress.md`](../implementation-plan/progress.md)에 test/snapshot/ADR/
  수동 증거 링크를 남긴다.
- 실제 credential 또는 사용자의 home SSH 파일을 fixture/artifact로 복사하지 않는다.
- 완료 명령은 저장소 공통 gate(`fmt`, `clippy -- -D warnings`, `test`)에 카드별 명령을 더한다.

## Card ownership map

아래 표는 모든 Remote 카드의 **표준 소유권**이다. 카드 본문의 `파일`/`작업` 설명은 범위를
보충하지만 이 표의 primary production/test 경계를 바꾸지 않는다. 구현 중 새 파일이 꼭
필요하면 먼저 이 표와 아키텍처를 같은 변경에서 갱신한다. 한 카드는 자기 production 파일과
primary test/evidence 파일을 함께 수정하며, 다른 카드의 primary 파일을 건드릴 때는 그 카드의
선행 조건과 회귀 gate도 실행한다. `gate-only` 행은 production 코드를 추가하지 않고 증거 문서만
갱신한다.

| 카드 | Primary production file(s) | Primary test/evidence file(s) |
|---|---|---|
| S0-00 | `docs/architecture/adr-006-remote-location-foundation.md` | `tests/remote_backend_contract.rs`, `docs/remote/05-acceptance-matrix.md` |
| S0-01 | `src/location/id.rs`, `src/location/path.rs` | `tests/location_model.rs` |
| S0-02 | `src/location/config.rs`, `src/config/schema.rs` | `tests/config_roundtrip.rs`, `tests/remote_backend_contract.rs` |
| S0-03 | `src/location/manager.rs`, `src/location/capability.rs`, `src/ports/location_reader.rs`, `src/adapters/local_location.rs` | `tests/location_model.rs`, `tests/remote_backend_contract.rs` |
| S0-04 | `src/location/state.rs`, `src/runtime/job.rs`, `src/runtime/lane.rs` | `tests/remote_faults.rs` |
| S0-05 | `src/adapters/fake_remote.rs` | `tests/support/remote.rs`, `tests/remote_backend_contract.rs` |
| S0-06 | `src/location/reducer.rs`, `src/runtime/remote_lane.rs` | `tests/remote_faults.rs` |
| S0-07 | `src/location/ui/picker.rs`, `src/app/command_registry.rs` | `tests/remote_scenarios.rs`, `tests/scenarios/remote/location_picker.yml`, `tests/snapshots/remote/location_picker.snap` |
| S0-08 | `Cargo.toml`, `.github/workflows/ci.yml` | `tests/support/isolated_sshd.rs`, `tests/remote_real_integration.rs` |
| S0-09 | 없음 — gate-only(production 변경 금지) | `docs/remote/05-acceptance-matrix.md`, `docs/implementation-plan/progress.md` |
| S1-01 | `src/ports/remote_transport.rs`, `src/adapters/sftp/mod.rs`, `src/adapters/sftp/session.rs` | `tests/remote_backend_contract.rs`, `tests/remote_real_integration.rs` |
| S1-02 | `src/location/metadata.rs`, `src/location/manager.rs`, `src/adapters/sftp/session.rs` | `tests/remote_backend_contract.rs`, `tests/remote_scenarios.rs` |
| S1-03 | `src/location/ui/connection.rs`, `src/app/reducer.rs`, `src/app/command_registry.rs` | `tests/remote_scenarios.rs`, `tests/snapshots/remote/main.snap` |
| S1-04 | `src/app/reducer.rs`, `src/model/viewer.rs`, `src/ui/dialogs/viewer.rs`, `src/ports/location_reader.rs` | `tests/remote_scenarios.rs`, `tests/remote_faults.rs`, `tests/snapshots/remote/viewer.snap` |
| S1-05 | `src/operations/transfer_coordinator.rs`, `src/operations/transfer.rs`, `src/runtime/lane.rs` | `tests/remote_backend_contract.rs`, `tests/remote_faults.rs` |
| S1-06 | `src/location/state.rs`, `src/location/reducer.rs`, `src/location/ui/connection.rs` | `tests/remote_faults.rs`, `tests/remote_scenarios.rs` |
| S1-07 | 없음 — gate-only(production 변경 금지) | `docs/remote/05-acceptance-matrix.md`, `docs/implementation-plan/progress.md` |
| S2-01 | `src/ports/location_writer.rs`, `src/ports/remote_transport.rs`, `src/adapters/local_location.rs`, `src/adapters/fake_remote.rs`, `src/adapters/sftp/session.rs` | `tests/remote_backend_contract.rs`, `tests/remote_faults.rs` |
| S2-02 | `src/operations/transfer.rs`, `src/operations/planner.rs`, `src/runtime/lane.rs` | `tests/remote_backend_contract.rs`, `tests/remote_faults.rs` |
| S2-03 | `src/operations/transfer_coordinator.rs`, `src/operations/transfer.rs`, `src/adapters/sftp/session.rs` | `tests/remote_backend_contract.rs`, `tests/remote_faults.rs`, `tests/remote_real_integration.rs` |
| S2-04 | `src/operations/transfer.rs`, `src/operations/remote_mutation.rs`, `src/adapters/sftp/session.rs` | `tests/remote_backend_contract.rs`, `tests/remote_real_integration.rs` |
| S2-05 | `src/operations/remote_mutation.rs`, `src/app/reducer.rs`, `src/ui/dialogs/input.rs` | `tests/remote_backend_contract.rs`, `tests/remote_scenarios.rs` |
| S2-06 | `src/operations/remote_mutation.rs`, `src/ui/dialogs/confirm.rs`, `src/ui/dialogs/progress.rs` | `tests/remote_faults.rs`, `tests/remote_scenarios.rs`, `tests/snapshots/remote/delete_confirm.snap` |
| S2-07 | `src/location/capability.rs`, `src/location/manager.rs`, `src/location/reducer.rs`, `src/ports/location_writer.rs` | `tests/remote_backend_contract.rs`, `tests/remote_scenarios.rs` |
| S2-08 | `src/runtime/remote_lane.rs`, `src/operations/transfer_coordinator.rs`, `src/location/state.rs` | `tests/remote_faults.rs` |
| S2-09 | 없음 — gate-only(production 변경 금지) | `docs/remote/05-acceptance-matrix.md`, `docs/implementation-plan/progress.md` |
| S3-01 | `src/location/state.rs`, `src/location/config.rs` | `tests/remote_backend_contract.rs`, `tests/remote_scenarios.rs`, `tests/snapshots/remote/cache_state.snap` |
| S3-02 | `src/ports/ssh_host_discovery.rs`, `src/adapters/openssh_hosts.rs` | `tests/remote_backend_contract.rs` |
| S3-03 | `src/location/ui/picker.rs`, `src/location/config.rs` | `tests/remote_scenarios.rs`, `tests/snapshots/remote/registration.snap` |
| S3-04 | `Cargo.toml`, `src/operations/transfer.rs`, `src/ports/location_writer.rs`, `src/ports/remote_transport.rs`, `src/adapters/local_location.rs`, `src/app/reducer.rs`, `src/ui/dialogs/progress.rs` | `tests/remote_faults.rs`, `tests/remote_scenarios.rs`, `tests/remote_real_integration.rs` |
| S3-05 | `src/location/metadata.rs`, `src/location/path.rs`, `src/location/config.rs`, `src/adapters/sftp/mod.rs` | `tests/remote_backend_contract.rs`, `tests/remote_real_integration.rs` |
| S3-06 | 없음 — gate-only(production 변경 금지) | `docs/remote/05-acceptance-matrix.md`, `docs/implementation-plan/progress.md` |

# S0 — Location Foundation

## S0-00 Identity/Path/Cancellation ADR gate

- 선행: R1
- 목표: public type이나 SSH dependency를 고르기 전에 되돌리기 비싼 결정을 승인한다.
- 파일: 새 ADR, 최소 disposable spike, dependency comparison table
- 작업:
  1. stable config `id`와 mutable `name`/`description`, 중복 없는 `EntryId(LocationPath)`를 확정한다.
  2. protocol path bytes 보존, invalid UTF-8 display escaping, display→protocol 재변환 금지를 확정한다.
  3. worker 외부 thread-safe `CancelHandle/Token`과 monotonic deadline, 실제 preemption 방식을 확정한다.
  4. view generation/session epoch/OperationId를 분리한다.
  5. native/OpenSSH 기반 후보를 config/agent/known_hosts/path/lstat/cancel/Windows/license 기준으로 비교한다.
  6. 하나를 선택하거나 필수 조건 불충족으로 `blocked`를 기록한다. 계약을 약화해 통과시키지 않는다.
- 완료 조건: ADR에 선택/기각 이유, public type sketch, cancel sequence, byte-path fixture 결과가 있다.
- progress/evidence:
  - [ ] ADR와 comparison table 링크 기록
  - [ ] 최소 connect-cancel/path fidelity spike 명령·결과 기록

## S0-01 Location identity와 byte path 값 타입

- 선행: S0-00
- 목표: Local/Remote identity와 protocol/display path를 손실 없이 구분한다.
- 파일: `src/location/{id,path}.rs`, `tests/location_model.rs`
- 작업:
  1. `LocationId`, `LocationPath`, `PathWithinLocation`, `RemotePathBytes`, `EntryId`를 만든다.
  2. byte component normalize/join/parent/root containment를 구현한다.
  3. deterministic `\\xNN` display escape와 terminal-cell truncation 입력을 분리한다.
  4. `PathBuf` special prefix와 display string 기반 mutation을 source-boundary로 금지한다.
- 테스트: `/`, dot/dotdot, duplicate slash, Unicode, spaces, leading dash, newline, invalid UTF-8,
  root escape, DEV/PROD same bytes, display collision.
- progress/evidence:
  - [ ] `location_model` test 이름/결과 기록
  - [ ] source-boundary scan과 byte roundtrip fixture 기록

## S0-02 Remote config schema와 migration

- 선행: S0-01, Core M3 config
- 목표: stable id와 credential 없는 canonical schema를 안전하게 저장한다.
- 파일: `src/location/config.rs`, config migration/tests
- 작업:
  1. timeout defaults와 `id/name/description/host/root/root_hex/read_only`를 구현한다.
  2. 범위/unique/alias/root/unknown-field validation과 field path 오류를 구현한다.
  3. UTF-8 `root`와 non-UTF-8 `root_hex` 상호 배타 encoding을 roundtrip한다.
  4. 손상 config 보존과 원자적 저장을 Core config와 공유한다.
- 테스트: zero/one/many, stable id after display rename, duplicate id/name, invalid display/alias/root,
  forbidden credential fields, defaults, unknown field, legacy config.
- progress/evidence:
  - [ ] config roundtrip/migration test 결과 기록
  - [ ] serialized config/error/snapshot secret scan 기록

## S0-03 capabilities, LocationManager, Local adapter

- 선행: S0-01,02
- 목표: 기존 Local 동작을 location-aware port 뒤에서 회귀 없이 실행한다.
- 파일: `src/location/{capability,manager}.rs`, `src/ports/location_reader.rs`,
  `src/adapters/local_location.rs`
- 작업:
  1. S0 capability에는 `READ`만 정의하고 registered Location lookup/backend routing과
     effective capability 계산을 구현한다. mutation/`RESUME` bit는 예약값을 포함해 만들지
     않는다.
  2. 기존 Local FileSystem의 read 동작만 Location reader adapter로 감싼다. Local mutation은
     S2-01 전까지 기존 Core port/path를 그대로 사용하고 writer trait이나 adapter를 만들지
     않는다.
  3. AppState path/selection/marks identity를 LocationPath에 연결한다.
  4. Remote Edit capability를 만들지 않고 Remote/Git 공통 경계 test seam을 둔다.
- 테스트: Local full contract, unknown/duplicate Location, rename display identity preservation,
  switch clears marks, render I/O 0, Local fast-path call count.
- progress/evidence:
  - [ ] v1 full regression 결과 기록
  - [ ] capability/Local adapter contract 결과 기록

## S0-04 cancel primitive와 결과 identity

- 선행: S0-00,01
- 목표: blocking call과 독립적으로 취소하고 세 종류 결과를 섞지 않는다.
- 파일: Core `src/runtime/{job,lane}.rs` 재사용, `src/location/state.rs`
- 작업:
  1. M2의 thread-safe `CancelHandle/CancelToken`, monotonic Deadline, `OperationId`를
     재사용하고 부족한 visibility/adapter hook만 generic runtime에 보강한다.
  2. Remote 전용 `ViewGeneration`, `SessionEpoch`와 common OperationId를 담는 typed
     envelope를 구현한다.
  3. cancel callback/child close 등록과 race-safe exactly-once terminal state를 구현한다.
  4. timeout과 user cancel을 구분한다.
- 테스트: cancel before/during/after block, deadline, double cancel, late callback,
  generation/epoch/op mismatch, exactly one terminal result.
- progress/evidence:
  - [ ] controlled-gate cancellation test 결과 기록
  - [ ] thread/leak sanitizer 또는 counter 결과 기록

## S0-05 FakeRemote read backend

- 선행: S0-01~04
- 목표: 실제 서버 없이 S0/S1 read 상태와 fault를 결정적으로 재현한다.
- 파일: `src/adapters/fake_remote.rs`, `tests/support/remote.rs`
- 작업:
  1. connect/read_dir/lstat/open_read와 session capability/close를 구현한다.
  2. exact path/context/thread/call order를 기록한다.
  3. controlled gate, nth-call error/panic, disconnect, partial read, cancel/deadline을 지원한다.
  4. active session/stream/callback counters를 제공한다.
- 범위: write/publish/rename/mkdir/delete/partial upload는 아직 구현하지 않는다.
- progress/evidence:
  - [ ] Fake builder/default/order/fault contract 결과 기록
  - [ ] test가 Fake 내부 map을 직접 수정하지 않는 source scan 기록

## S0-06 connection reducer와 per-Location worker lane

- 선행: S0-03~05
- 목표: reducer는 pure하게 유지하고 Remote 지연을 Local과 분리한다.
- 파일: `src/location/reducer.rs`, `src/runtime/remote_lane.rs`
- 작업:
  1. Connect/Disconnect/Reconnect/Result action과 effect를 구현한다.
  2. Location별 기본 capacity 16 serial worker, active worker 기본 상한 4와 독립 control
     registry를 구현한다. full refresh는 최신 generation으로 coalesce, 나머지는 Busy,
     다섯 번째 Location은 LimitReached다.
  3. queue 밖에서 handle cancel/SessionControl close를 수행하고 sender close 뒤 정상 join한다.
     adapter deadline/cancel로 blocking call을 유한하게 만들며 timeout join/detach는 금지한다.
  4. view generation/session epoch/OperationId 적용 규칙을 구현한다.
  5. 각 job boundary의 panic을 failure로 바꾸고 그 Location session epoch만 lost 처리한다.
- 테스트: injected queue capacity 1/full coalesce/Busy/try_send 비차단, injected worker limit 1의
  second Location LimitReached, A→B late result, reconnect old epoch, cancel blocked connect,
  Local actions while blocked, Remote panic 뒤 해당 session만 lost/다른 Remote+Local 계속,
  quit/delete Location cleanup.
- progress/evidence:
  - [ ] reducer interleaving test 결과 기록
  - [ ] worker/session/thread leak 0 증거 기록

## S0-07 Location picker와 context keymap

- 선행: S0-02,03,06, Core M3 CommandRegistry
- 목표: MCD F3 Drive/F12 Locations에서 Local과 등록 Remote를 선택한다.
- 파일: `src/location/ui/picker.rs`, registry contribution, snapshots
- 작업:
  1. Local/Remote group, name/description/[RO]/connection state를 표시한다.
  2. MCD context F3 Drive, F12 Locations, picker Up/Down/Enter/Esc, Help를 registry에 둔다.
  3. Main F3 View와 Viewer/Dialog F3가 MCD F3 Drive mapping을 상속하지 않음을 test한다.
  4. Remote Enter는 connect effect, Local Enter는 Local load를 만든다.
  5. Core scenario v1의 기존 field를 유지한 채 optional Remote Location fixture, Remote
     effect completion/failure/blocking, 단계별 assertion action을 추가한다. 기존 v1 fixture는
     수정 없이 계속 통과해야 한다.
- 테스트: order, navigation, cancel preservation, context collision, disabled/connecting,
  60×15/80×25/120×40, wide/Unicode display.
- progress/evidence:
  - [ ] picker mapper/component tests와 snapshot 링크 기록
  - [ ] Help/표시 키/실제 mapping 일치 증거 기록

## S0-08 isolated `sshd` harness

- 선행: S0-00,05
- 목표: 사용자 환경과 network에 의존하지 않는 production adapter contract harness를 만든다.
- 파일: `tests/support/isolated_sshd.rs`, disposable container/process fixture, CI opt-in job
- 작업:
  1. TempDir home/config/key/known_hosts와 고정 fixture tree를 생성한다.
  2. host key를 known_hosts에 등록하고 검증을 유지한다.
  3. auth-required/changed-host-key/latency/disconnect/permission/path-byte fixture를 제공한다.
  4. mutation root guard와 teardown session/child/container leak assertion을 구현한다.
- 실행 계약: Cargo feature `remote-integration`, `tests/remote_real_integration.rs`의
  `#![cfg(feature = "remote-integration")]`, 각 real test의
  `#[ignore = "requires isolated sshd"]`를 함께 추가한다. 기본 gate는 network 0이고 stage
  gate/CI `remote-integration` job은 아래 exact command를 반드시 실행한다.
  `cargo test --locked --features remote-integration --test remote_real_integration -- --ignored --test-threads=1`
- 테스트: harness start/health/teardown, no real home access, parallel isolation, failure cleanup.
- progress/evidence:
  - [ ] 위 exact command와 isolated harness CI job URL 기록
  - [ ] 실제 home/network access 0과 teardown leak 0 기록

## S0-09 S0 foundation gate

- 선행: S0-01~08
- 목표: production browse 전에 foundation 계약을 닫는다.
- 검증: `S0-GATE-01~05`. gate를 먼저 축약 통과시키지 말고 각 gate가 가리키는 상세 행의
  Status/Evidence를 개별 갱신한다.
- progress/evidence:
  - [ ] 공통 gate + S0 suite 결과 기록
  - [ ] acceptance 행별 Evidence/Status 갱신

# S1 — Browse / View / Download

## S1-01 production RemoteSession read adapter

- 선행: S0-09
- 목표: ADR에서 선택한 transport를 read port와 독립 cancel control에 연결한다.
- 작업: batch connect, config/agent/known_hosts, capabilities, session reuse/replace/close,
  error normalization/redaction, connect/list/read cancel+deadline.
- 테스트: Fake conformance와 isolated `sshd`; shell string/host-key disable/interactive prompt 0.
- progress/evidence:
  - [ ] read adapter conformance와 isolated integration 결과 기록
  - [ ] auth/host-key/redaction/active child 0 증거 기록

## S1-02 listing, lstat, navigation

- 선행: S1-01, S0-06
- 목표: configured root와 하위 directory를 안전하게 탐색한다.
- 작업:
  1. read_dir와 lstat을 common entry/metadata로 normalize한다.
  2. symlink/broken link/other를 표시하고 follow하지 않는다.
  3. synthetic `..`, byte containment, directories-first stable sort를 적용한다.
  4. path/refresh마다 view generation을 올리고 stale listing을 버린다.
- 테스트: empty/basic/10k/Unicode/non-UTF8/permission/not-found/disconnect/root/symlink cycle.
- progress/evidence:
  - [ ] Fake+real listing/lstat contract 결과 기록
  - [ ] worker-only network/thread call 증거 기록

## S1-03 Remote Main UI와 disabled commands

- 선행: S1-02, S0-07
- 목표: Remote identity/state/capability를 Local 목록에서 명확히 표시한다.
- 작업: `DEV:/path`, `[RO]`, Connecting/Failed/Disconnected, Remote status summary,
  mutation/F4 disabled reason, regular-file Enter/F3 command mapping.
- 테스트: renderer backend 0, Remote regular-file OS launcher 0, F4 effect 0,
  connecting/empty/basic/RO/error/80×25/too-small snapshots.
- progress/evidence:
  - [ ] component/mapper/snapshot 결과 기록
  - [ ] launcher 0/F4 disabled source+call assertion 기록

## S1-04 Remote Viewer

- 선행: S1-02, Core M2 Viewer
- 목표: regular-file Enter 또는 Main-context F3로 Remote 파일을 안전하게 본다. Viewer가
  열린 뒤 F3 의미는 Viewer 자체 CommandRegistry context가 결정한다.
- 작업: Core Viewer와 같은 32 MiB 상한을 사용한다. metadata 길이가 초과면 `open_read` 0회로
  TooLarge, 길이가 없거나 변하면 Remote lane이 최대 `32 MiB + 1 byte`만 bounded memory로
  읽어 초과를 판정한다. private temp 파일은 만들지 않는다. cancel/error는 buffer를 버리고,
  UTF-8/BOM/NUL binary 판정과 scroll/search model은 Core Viewer를 재사용한다.
- 테스트: exact byte path, partial read, disconnect, cancel, timeout, symlink refusal,
  cleanup warning, Unicode/non-UTF8 display snapshot.
- progress/evidence:
  - [ ] Viewer scenarios/snapshots 기록
  - [ ] operation terminal result와 temp/stream leak 0 기록

## S1-05 Remote → Local Download

- 선행: S1-01, S1-03, Core M2 copy planner/worker/MutationCoordinator, ADR-005 transfer bridge
- 목표: explicit local destination에 temp+atomic publish로 다운로드한다.
- 작업: Main F5 `Copy`가 Remote source에서 Local-only destination dialog를 열게 한다. I/O 없는
  TransferCoordinator, capacity 2×256 KiB chunk handoff, Core Local endpoint의
  non-blocking lease+local sibling temp/publish와 Remote lane read endpoint, conflict policy,
  progress/cancel/deadline, metadata best effort, partial/cleanup result.
- 테스트: success/six conflict choices/cancel/timeout/disconnect/disk full/path escape,
  active lease Busy에서 양 endpoint calls 0, chunk backpressure, 한 endpoint failure가 반대쪽을
  cancel, existing destination preservation, screen switch terminal result.
- progress/evidence:
  - [ ] transfer fault matrix 결과 기록
  - [ ] existing destination/temp/OperationId assertions 기록

## S1-06 disconnect/reconnect와 last-visible listing

- 선행: S1-02~05
- 목표: 연결 문제 중 마지막 화면만 보존하고 reusable cache로 오인하지 않는다.
- 작업:
  1. 현재 path의 마지막 성공 listing 하나를 disconnected 화면에 표시한다.
  2. path 재방문/cache hit/backend-call 절감 API는 만들지 않는다.
  3. Enter Reconnect/Esc Local return, no infinite retry, old session epoch discard를 구현한다.
  4. blocked connect/list/view 중 resize/Help/quit/Local 전환을 유지한다.
- 테스트: last result 표시, 다른 path call count 1, reconnect races, cancel preemption, shutdown.
- progress/evidence:
  - [ ] controlled interleaving/scenario 결과 기록
  - [ ] key→frame와 session/stream leak 0 기록

## S1-07 S1 read-only Remote gate

- 선행: S1-01~06
- 목표: browse/view/download 범위를 Fake와 isolated `sshd`로 닫는다.
- 검증: `S1-GATE-01~05`. gate를 먼저 축약 통과시키지 말고 각 gate가 가리키는 상세 행을
  개별 갱신한다. 10k listing release smoke, secret scan, Windows OpenSSH 수동 matrix를
  Evidence에 링크한다.
- progress/evidence:
  - [ ] 자동/isolated real/성능 결과 기록
  - [ ] Windows 수동 행과 acceptance Status/Evidence 갱신

# S2 — Transfer / Mutation (Remote Drive MVP)

## S2-01 writer port와 FakeRemote write 확장

- 선행: S1-07
- 목표: mutation을 시작할 때만 writer/fault surface를 추가한다.
- 작업: 이 카드에서 `UPLOAD`/`RENAME`/`MKDIR`/`DELETE`/`SERVER_COPY` capability와 Local/Remote
  writer adapter를 처음 추가한다. `SessionAccess::ReadWrite`, temp New/write/flush/publish/
  discard, `copy_within`, rename, mkdir, remove, partial write, permission/disconnect, cleanup fault,
  RO direct rejection, exact call record를 구현한다. success/error/cancel/panic의 temp lifecycle을
  architecture §6 순서로 검증하며 `RESUME`은 추가하지 않는다.
- 테스트: writer builder/default/order, cancel/deadline, active writer/temp counters, read regression.
- progress/evidence:
  - [ ] Fake writer contract 결과 기록
  - [ ] S0/S1 read-only Fake API regression 기록

## S2-02 transfer planner와 local-endpoint mutation lease

- 선행: S2-01
- 목표: route/capability/RO/containment/conflict를 backend 전에 결정한다.
- 작업: Local↔Remote/same/cross Remote route, immutable temp plan, symlink policy,
  Remote-only Location lane 직렬화, local target 쓰기/local source 삭제 구간의 Core/Git 공통
  mutation lease.
- 테스트: route matrix, stable LocationId equality, root escape, same target/subtree, marks,
  RO/capability, cross-Remote backend calls 0, active local lease는 Busy/local mutation calls 0.
- progress/evidence:
  - [ ] planner/lease test 결과 기록
  - [ ] invalid plan backend calls 0 기록

## S2-03 Local↔Remote Copy/Move 완성

- 선행: S2-02, S1-05
- 목표: 기존 F5/F6와 location-aware destination dialog로 Upload와 양방향 Move를 완성한다.
- 작업: Local→Remote Copy는 Core read→Remote sibling temp/publish, Move는 그 성공 뒤
  non-blocking local lease로 source를 삭제한다. Remote→Local Move는 S1 Download 성공 뒤
  `DELETE` capability와 permanent warning 확인 후 Remote lane에서 source를 삭제한다. delete
  Busy/error는 destination 성공/delete 미실행 partial result다. 어느 lane도 상대 endpoint
  port를 직접 호출하지 않는다.
- 테스트: route별 F5/F6 availability/dialog, success/six conflict choices/partial/disconnect/
  permission/cancel/timeout/cleanup, 두 Move의 delete Busy/error partial, permanent warning,
  Unicode/non-UTF8/leading dash, screen switch terminal result.
- progress/evidence:
  - [ ] Fake+isolated real upload matrix 기록
  - [ ] incomplete final target 0/temp leak 0 기록

## S2-04 same-Remote Copy/Move

- 선행: S2-02,03
- 목표: 같은 LocationId session 안에서 Copy/Move를 구현한다.
- 작업: `SERVER_COPY`일 때만 `copy_within`, 아니면 같은 lane의 safe stream fallback,
  `RENAME` fast path 또는 `READ+UPLOAD+DELETE` fallback, no-follow recursion,
  copy+delete destructive confirmation, partial result.
- 테스트: rename/copy/fallback/cancel/partial delete/root/symlink/subtree; DEV→PROD calls 0.
- progress/evidence:
  - [ ] route별 Fake+real 결과 기록
  - [ ] cross-Remote zero-call와 partial summary 기록

## S2-05 Remote Rename과 MkDir

- 선행: S2-02
- 목표: Core dialog validation을 Remote byte path에 적용한다.
- 테스트: Unicode/non-UTF8 display, empty/dot/slash/control/exists/permission/disconnect/RO,
  success refresh exactly once.
- progress/evidence:
  - [ ] dialog/planner/backend test 결과 기록
  - [ ] success refresh 1회/invalid calls 0 기록

## S2-06 permanent Remote Delete

- 선행: S2-02
- 목표: 복구 불가능성을 숨기지 않는 file/tree delete를 구현한다.
- 작업: permanent wording, Location/path/count 확인, Esc zero calls, root refusal,
  no-follow recursion, progress/partial/cleanup.
- 테스트: cancel/confirm/file/tree/partial/disconnect/RO/root/symlink.
- progress/evidence:
  - [ ] confirmation snapshots와 mutation tests 기록
  - [ ] local trash adapter/wording 미사용 증거 기록

## S2-07 read-only/capability four-layer policy

- 선행: S2-03~06
- 목표: UI/reducer/planner/backend 모든 우회 경로를 차단한다.
- 테스트: Upload/Copy target/Move/Rename/MkDir/Delete와 F4 각각 command disabled,
  direct action effect 0, planner `ReadOnly`/`Unsupported`, backend direct rejection;
  Browse/Lstat/View/Download/Refresh 허용.
- progress/evidence:
  - [ ] four-layer matrix 결과 기록
  - [ ] mutation backend call 0 및 F4 항상 disabled 기록

## S2-08 progress/cancel/fault hardening

- 선행: S2-03~07
- 목표: 장시간 작업 중 응답성과 exactly-once terminal result를 보장한다.
- 작업: progress ≤20 Hz, independent handle cancel, deadline, session loss, cleanup warning,
  cancel/close/deadline로 유한하게 끝나는 shutdown lifecycle과 정상 join, current-view refresh
  1회.
- 테스트: controlled gates without sleep, resize/help/cancel/quit, out-of-order view/session,
  operation survives screen switch, endpoint/job panic은 해당 session만 lost, Local/다른 Remote
  계속, double completion, cleanup failure.
- progress/evidence:
  - [ ] fault/interleaving/performance 결과 기록
  - [ ] active worker/child/stream/temp leak 0 기록

## S2-09 Remote Drive MVP gate

- 선행: S2-01~08
- 목표: Fake와 isolated `sshd`, 플랫폼 수동 증거로 Remote mutation 범위를 닫는다.
- 검증: `S2-GATE-01~05`. 각 gate가 가리키는 상세 행을 개별 갱신하고 Local regression과
  Windows upload/download/bidirectional Move/cancel/RO 결과를 Evidence에 링크한다. Git이
  이미 있으면 실제 Git integration도 실행하고, 없으면 해당 `S2-INTEG-01` Evidence에
  `counterpart: N/A — Git not implemented`를 기록한다.
- progress/evidence:
  - [ ] 자동/isolated real/성능/secret scan 결과 기록
  - [ ] Windows 수동 행과 S2 acceptance Status/Evidence 갱신

# S3 — Cache / Registration / Hardening

## S3-01 reusable directory TTL/LRU cache

- 선행: S2-09
- 목표: S1 last-visible listing과 분리된 reusable cache를 추가한다.
- 작업: LocationId+path key, injected Clock, cache config migration/defaults, TTL, deterministic LRU bound,
  manual bypass, mutation invalidation, hit/stale/expired 표시.
- 테스트: off/default/hit/miss/expiry/eviction/manual refresh/reconnect/mutation/out-of-order.
- progress/evidence:
  - [ ] FixedClock/cache test와 snapshots 기록
  - [ ] hit calls 0/manual refresh 1/invalidation refresh 1 기록

## S3-02 OpenSSH Host discovery adapter

- 선행: S3-01, S0-02
- 목표: credential을 해석/저장하지 않고 등록 후보 alias만 제공한다.
- 작업: platform config path/Include, literal Host alias, wildcard/negation 처리,
  safe description, parsing error isolation; registered config와 별도 model.
- 테스트: comments/multiple Host/Include/cycle/wildcard/Unicode/error; real home access 0;
  반환 model에 username/key/hostname/port field 0.
- progress/evidence:
  - [ ] temp-config discovery contract 결과 기록
  - [ ] credential field/source scan 기록

## S3-03 Host browser와 Remote registration UI

- 선행: S3-02
- 목표: 발견 후보를 명시적 form으로 등록/편집/삭제한다.
- 작업: registered/discovered group, id/name/description/alias/root/RO form,
  validation, atomic save/delete, no auto-registration, id immutable edit policy.
- 테스트: cancel/duplicate/rename display/stable id/forbidden credential widget absence,
  snapshots, `.ssh` files before/after unchanged.
- progress/evidence:
  - [ ] UI/config scenarios와 snapshots 기록
  - [ ] auto-register 0 및 SSH file unchanged 기록

## S3-04 resume와 대용량 transfer

- 선행: S3-01
- 목표: capability 기반 explicit resume와 >4 GiB counters를 강화한다.
- 작업: 이 카드에서 `RESUME` capability를 처음 추가한다. S2 base port와 분리된
  `ResumableLocationWriter`/`ResumableLocationReadWriter`, session-independent `ResumeToken`,
  `inspect_temp`/`open_write_temp_resume`와 `SessionAccess::Resumable` variant를 S3에서
  추가한다. 첫 전송 전
  source 전체 SHA-256/길이와 stream 중 committed-prefix SHA-256, route별 destination partial 길이/지문
  검증을 구현한다. explicit Resume/Restart/Cancel, unsupported/invalid-token/source-changed/
  partial-mismatch별 정확한 오류, 사용자 선택 전 자동 restart 금지, reconnect/session epoch를
  적용한다. Download는 `open_read(offset)`과 Local writer resume, Upload는 local reader offset과
  Remote writer resume를 같은 contract로 검증한다. SHA-256 dependency audit를 progress에 남긴다.
- 테스트: matching, token identity/version mismatch, source byte/length mismatch, Download Local
  temp와 Upload Remote temp 각각의 partial length/prefix mismatch, unsupported,
  fingerprint/prefix cancel과 stream 중 source 변경, no-auto-restart/publish 0, reconnect,
  >1 GiB isolated smoke, >4 GiB Fake counters, cancel.
- progress/evidence:
  - [ ] capability/resume matrix 기록
  - [ ] 대용량 progress/cancel 수동·자동 증거 기록

## S3-05 metadata/path/platform hardening

- 선행: S3-01~04
- 목표: permission/symlink/path fidelity와 Windows 배포를 닫는다.
- 작업: portable permission theme, broken/cycle symlink display, path-byte limitations,
  slow/high-latency server, config migration, adapter packaging.
- 테스트: permission/lstat variants, invalid UTF-8 policy, latency/disconnect, migration,
  Windows/Linux/macOS matrix.
- progress/evidence:
  - [ ] metadata/path/latency regression 기록
  - [ ] 플랫폼 packaging/manual matrix 기록

## S3-06 S3 gate와 후속 경계

- 선행: S3-01~05
- 목표: cache/registration/hardening을 닫고 제외 기능을 다시 확인한다.
- 검증: `S3-GATE-01~04`와 이전 단계 전체 Local/Remote gate. 각 상세 행을 개별 갱신한다.
  Git이 있으면 실제 양방향 integration도 필수이고, 없으면 `S3-INTEG-01` Evidence에
  `counterpart: N/A — Git not implemented`를 기록한다.
- 제외 확인: Remote Edit, Remote Git, SSH terminal, cross-Remote Copy/Move의 production
  code/command/effect/backend 0.
- progress/evidence:
  - [ ] 전체 gate와 acceptance Status/Evidence 갱신
  - [ ] 제외 기능 source/command/backend zero-count 기록
