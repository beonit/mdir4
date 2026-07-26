# ADR-005: 기능 영역별 bounded background work lane

## 상태

Accepted — M2에서 Core Local lane으로 시작하고 G0/G3/S0에서 단계적으로 확장

## 관계

- [ADR-001](adr-001-reducer-effect.md)의 reducer/effect 흐름을 유지한다.
- [ADR-003](adr-003-worker-model.md)의 `std::thread + mpsc`, UI 비차단과 로컬 mutation
  직렬화 결정을 유지한다.
- ADR-003의 “하나의 I/O worker”는 v1 Core Local 작업 범위다. Git과 SSH Remote가 추가된
  뒤에도 모든 I/O를 하나의 전역 queue에 넣으라는 결정으로 해석하지 않는다.
- [ADR-004](adr-004-built-in-plugin-boundary.md)의 plugin effect 격리 경계를 실행 계층에
  적용한다.

## 맥락

M1의 `EffectWorker`는 directory read, disk info와 file launch를 worker 하나에서 직렬
실행한다. v1의 작은 로컬 범위에는 적합하지만 다음 작업을 같은 queue에 넣으면 서로
관련 없는 기능이 멈춘다.

- 대형 local copy가 Git status 갱신을 막는다.
- 느린 Git backend가 local directory 탐색을 막는다.
- 한 SSH host의 timeout이 Local과 다른 Remote Location을 막는다.
- blocking Remote session method 뒤에 cancel을 enqueue하면 현재 호출을 중단할 수 없다.
- 여러 mutation 경로가 서로 다른 worker에 있으면 같은 worktree를 동시에 바꿀 수 있다.

반대로 범용 async runtime과 thread pool을 먼저 도입하면 실제 동시성 요구보다 scheduling,
lifetime과 shutdown 복잡도가 커진다.

## 고려한 선택지

| 선택 | 장점 | 단점 |
|---|---|---|
| 모든 작업을 전역 worker 하나에 직렬화 | 가장 단순, mutation 충돌 없음 | 느린 plugin/Remote가 파일 탐색 전체를 막음 |
| 모든 job마다 새 thread | 서로 덜 막힘 | thread/session 무제한, 순서·종료·취소가 불명확 |
| Tokio + async task | 풍부한 취소/동시성 도구 | blocking FS/SSH adapter에는 별도 thread가 필요하고 v1 복잡도 증가 |
| 기능별 bounded serial lane | 격리, 결정적 순서, 작은 구현 | lane supervisor와 공통 mutation lease가 필요 |

## 결정 요약

UI thread는 입력, reducer와 render만 담당한다. 외부 I/O는 표준
`std::thread`와 `std::sync::mpsc`/`sync_channel`로 구성한 작은 serial lane에서 실행한다.
초기 구현에는 Tokio, 범용 thread pool 또는 fire-and-forget thread를 사용하지 않는다.

```text
                         completion Action
                    ┌────────────────────────┐
                    │                        ▼
UI Action queue ─► LaneSupervisor ─► Core Local lane (1 worker)
       │            │
       │            ├──────────────► Plugin Read lane (1 worker, bounded)
       │            │
       │            ├──────────────► Git Transport lane (G3, 1 worker, bounded)
       │            │
       │            └──────────────► Remote lane registry
       │                                  ├─ Location A (1 serial worker)
       │                                  └─ Location B (1 serial worker)
       │
       └────────────── Cancel/Shutdown control handles
```

각 job은 operation/request identity, owner, generation/session epoch, deadline과 결과를 적용할
대상을 담은 envelope로 전달한다. reducer는 completion identity를 검증한 뒤 state에
적용한다.

M2가 다음 neutral primitive의 단일 소유자다. `OperationId`, `CancelHandle/CancelToken`,
monotonic `Deadline`, `JobControl`, bounded `LaneSender`는 `src/runtime/job.rs`와
`src/runtime/lane.rs`에 두고 Git/Remote가 다시 정의하지 않는다. G0/S0는 필요한 경우 이
모듈을 이동/공개 범위만 조정하고 타입을 복제하지 않는다.

기본 상한은 implementation constant로 고정한다.

| 상한 | 기본값 |
|---|---:|
| Core Local request queue | 16 |
| Plugin Read request queue | 16 |
| Git Transport request queue | 16 |
| Remote Location별 request queue | 16 |
| 동시에 active인 Remote Location worker | 4 |

test는 같은 생성자에 1~2의 작은 capacity를 주입해 full 경계를 sleep 없이 만든다. 사용자
설정으로 노출하거나 값을 바꾸려면 측정 결과와 ADR 갱신이 필요하다.

<a id="lane-ownership"></a>
## Lane 소유권 계약

| Lane | 도입 시점 | 동시 실행 | 소유 작업 | 소유하면 안 되는 작업 |
|---|---:|---:|---|---|
| Core Local | M1/M2 | 1 | local directory/metadata/read, Core mutation, opaque G2 `LocalMutationJob`, disk/launch effect | Git read/network transport, SSH session/network I/O |
| Plugin Read | G0 | 1 | discover, status, diff/log처럼 read-only인 built-in plugin job | Core file mutation, Git mutation, Remote I/O |
| Git Transport | G3 | 1 | Git resolving/auth/transfer와 network-coupled ref/worktree operation | Core adapter, SSH Remote session, read-only refresh |
| Remote Location | S0/S1 | Location마다 1 | 해당 Location의 connect/session/list/stat/stream/mutation | Local FS, 다른 Location session, Git job |

Local↔Remote transfer는 lane 소유권의 예외가 아니다. I/O 없는 `TransferCoordinator`가
capacity 2 × 256 KiB chunk channel과 공통 OperationId/control만 소유하고, Local endpoint
job은 Core Local lane, Remote endpoint job은 해당 Remote lane에서 실행한다. Remote lane이
Local FS를 직접 호출하거나 Core lane이 SSH session을 직접 호출하는 구현은 금지한다.

### Core Local lane

- M1 `EffectWorker`를 발전시킨 lane이다.
- Core와 G2 local-only mutation은 반드시 이 lane에서 한 번에 하나만 실행한다. G3의
  network-coupled mutation은 Git Transport lane에서 공통 lease로 직렬화한다.
- G2의 Git mutation은 Git 전용 타입을 Core에 노출하지 않는 generic
  `LocalMutationJob` envelope로 이 lane에 제출한다. Git backend/model 소유권은 plugin에
  남지만 실행 순서는 Core mutation과 같다.
- progress/conflict/cancel/terminal result를 Action으로 보낸다.
- directory refresh는 mutation terminal result 뒤에 같은 lane으로 coalesce한다.
- MCD 또는 read job이 active mutation의 안전한 완료를 방해하지 않도록 command 활성 상태와
  queue 정책을 적용한다.
- request queue는 기본 16의 bounded non-blocking sender다. 같은 view의 아직 시작하지 않은
  refresh/load는 최신 generation 하나로 합치고, 합칠 수 없는 full submit과 active mutation
  중 새 mutation은 `Busy` completion으로 즉시 되돌린다.

### Plugin Read lane

- 정확히 worker thread 하나에서 read-only plugin job을 직렬 실행한다.
- 기본 16의 `sync_channel` 또는 동등한 명시적 상한을 사용한다. 무제한 channel로 바꾸지
  않는다.
- 같은 plugin/repository의 아직 시작하지 않은 refresh는 최신 generation 하나로
  coalesce한다.
- UI thread의 submit은 blocking send를 사용하지 않는다. queue가 가득 차면 중복 refresh는
  합치고, 합칠 수 없는 사용자 명령은 `Busy` completion으로 되돌린다.
- disabled/faulted plugin은 새 job을 제출하지 않으며 queued job result도 적용하지 않는다.

### Git Transport lane

- G3-02에서 Git built-in이 worker 하나와 기본 capacity 16의 bounded sender를 만든다. G0/G1/G2
  구현을 위해 미리 만들지 않는다.
- resolving/auth/transport와 network에 결합된 ref/worktree update만 이 lane에서 실행한다.
  G1 status/diff/log는 Plugin Read, G2 local-only mutation은 generic `LocalMutationJob`으로 Core
  Local lane을 계속 사용한다.
- 한 G3 operation이 `Queued` 이상이면 다른 G3 mutation command는 sender에 쌓지 않고
  `Busy` completion을 즉시 돌려준다. cancel은 job queue 밖 공통 control handle로 전달한다.
- Fetch/Pull/Push/Clone은 auth가 성공한 뒤 첫 `Transferring` 직전에 공통 mutation lease를
  non-blocking으로 얻는다. 성공하면 Terminal cleanup까지 보유하고, active면 auth preflight
  호출만 허용한 채 transfer와 local/remote mutation 0회로 끝낸다.
- Remote Manage는 config write 직전, Conflict Continue/Abort는 apply 직전에 lease를 얻는다.
  success/error/cancel/panic 모든 경로에서 RAII guard를 반환한다.
- conflict-context `Mark Resolved`는 network operation이 아니므로 G2 mutation과 같은 opaque
  `LocalMutationJob`으로 Core Local lane에 제출하고 공통 lease를 얻는다.

### Remote Location lane

- Connecting 또는 Connected Location마다 session을 소유한 serial worker 하나를 둔다.
- 한 Location 안에서는 connect/list/stat/read/write/mutation 순서를 보존한다.
- 서로 다른 Location과 Core Local lane은 독립이므로 한 host timeout이 다른 탐색을
  막지 않는다.
- Location별 request queue는 기본 16, 동시에 active 가능한 Remote worker는 기본 4다.
  queue full은 동일 view refresh를 최신 generation으로 coalesce하고 그 밖은 `Busy`, worker
  상한에서 새 Location connect는 `LimitReached`를 반환한다.
- session/stream/child는 UI state가 아니라 해당 lane이 소유한다.
- disconnect, config 삭제와 session epoch 변경 시 그 lane의 오래된 result를 state/cache에
  적용하지 않는다.

<a id="ui-nonblocking"></a>
## UI 비차단과 backpressure

UI thread에서 금지하는 동작은 다음과 같다.

- filesystem, Git backend, SSH transport와 child process 직접 호출
- blocking `send`, `recv`, `join`, mutex 대기
- worker가 끝날 때까지 busy loop 또는 sleep

UI는 `try_send`로 job/control 요청을 제출하고 `try_recv`로 completion을 Action queue에
넣는다. terminal event poll은 기존 최대 50 ms 주기를 유지한다. progress event는 worker
쪽에서 최대 20 Hz로 coalesce해 render 폭주를 막는다. queue full, cancellation pending과
shutdown pending도 명시적 state로 렌더한다.

<a id="mutation-lease"></a>
## Core/Git 공통 mutation lease

Core file mutation과 Git mutation은 같은 local filesystem/worktree를 바꿀 수 있으므로
서로 다른 lane에 있어도 동시에 실행하면 안 된다. `MutationCoordinator`가 process 내부
단일 active mutation lease를 소유한다.

```text
Core Copy/Move/Delete ─┐
                      ├─► MutationCoordinator ─► RAII MutationLease
Git Stage/Commit/... ─┘
```

- Core Local mutation은 실행 직전에 lease를 얻는다.
- Git Stage/Unstage/Commit/Stash/Branch처럼 G2 local-only index/worktree/ref를 바꾸는 job도
  generic `LocalMutationJob`으로 Core Local lane에 들어가 backend mutation 전에 같은 lease를
  얻는다. G3 network-coupled mutation은 Git Transport lane에 남되 위와 같은 lease를 얻는다.
  Plugin Read lane에서는 mutation을 실행하지 않는다.
- lease가 있으면 다른 mutation command는 disabled 또는 queued가 아니라 `Busy`로
  명확히 거부한다. 사용자 승인 없이 두 mutation을 병렬 실행하지 않는다.
- lease는 RAII guard이며 success, error, cancel와 panic unwind에서 항상 반환된다.
- terminal result 뒤 관련 directory와 Git status refresh를 한 번씩 coalesce한다.
- Remote-only mutation은 해당 Location lane의 직렬화로 보호한다. transfer가 local target을
  쓰거나 local source를 삭제하면 해당 local 쓰기/삭제 호출 전에 공통 mutation lease도
  non-blocking으로 얻는다. 이미 active면 기다리거나 queue하지 않고 `Busy`로 끝내며 local
  mutation 호출은 0회다. 앞선 remote-only phase가 이미 완료됐다면 그 partial 결과도
  숨기지 않는다. Remote→Local Move의 remote source delete는 해당 Location lane에서 download
  publish 뒤 실행하며 실패하면 destination 성공/delete 미실행 partial result다.

lease는 OS 전체 lock이나 다른 Mdir4 process와의 동기화를 보장하지 않는다. 외부 변경은
metadata 재검증과 각 operation의 안전 저장/충돌 계약으로 감지한다.

<a id="cancel-control"></a>
## 취소와 deadline control path

취소 요청을 현재 blocking call과 같은 serial job queue 뒤에 넣지 않는다. 특히
`RemoteSession::cancel(&mut self)`처럼 session worker만 호출할 수 있는 API는 실행 중인
`connect/read/write`를 선점하지 못하므로 취소 계약으로 사용하지 않는다.

각 취소 가능한 job은 시작 전에 다음을 등록한다.

```rust
pub trait CancelHandle: Send + Sync {
    fn request_cancel(&self);
}

pub struct JobControl {
    pub operation_id: OperationId,
    pub deadline: Instant,
    pub cancel: Arc<dyn CancelHandle>,
}
```

- UI/control 요청은 `LaneSupervisor`의 thread-safe handle registry를 통해
  `request_cancel()`을 직접 호출한다. 작업 queue가 비워질 때까지 기다리지 않는다.
- adapter는 operation별 deadline을 실제 blocking API, socket/process 또는 반복 read/write에
  적용한다.
- stream loop는 chunk 사이에서 cooperative token을 확인한다.
- process adapter는 별도 child-kill handle을 등록하고 shell 문자열 대신 executable과
  argument array를 사용한다.
- cancellation은 terminal `Cancelled`, deadline은 `TimedOut` result를 반드시 한 번
  생성한다. reducer가 stale result를 화면에 적용하지 않더라도 operation audit/cleanup
  상태는 보존한다.
- “cancel requested”는 완료가 아니다. handle 호출 후 resource가 닫히고 terminal result가
  도착할 때까지 상태를 구분한다.

transport가 독립 cancel handle과 deadline을 제공하지 못하면 S0 backend gate를 통과할 수
없다.

<a id="result-identity"></a>
## 결과 identity와 stale 처리

서로 다른 수명의 숫자를 하나의 generation에 합치지 않는다.

- Core directory/view load: view generation 또는 request ID
- Plugin read: PluginId + repository identity + plugin generation + RequestId
- Remote session: LocationId + session epoch
- Remote listing/view: LocationId + view generation + RequestId
- file/transfer mutation: OperationId와 terminal operation state

새 path/refresh로 오래된 listing result가 stale이 되어도 transfer terminal result 자체를
버리면 안 된다. 화면 listing 적용 여부와 operation 완료/cleanup 기록 여부를 분리한다.

<a id="panic-isolation"></a>
## 오류와 panic 격리

각 lane은 job 경계에서 panic을 포착해 구조화된 failure completion으로 변환하고 worker
loop 전체와 UI runtime을 종료하지 않는다. 구현은 `catch_unwind` 사용 범위를 job 호출
주위로 제한하고 무조건 `AssertUnwindSafe`로 넓히지 않는다.

- Core Local job panic: active operation을 failed로 종료하고 lease/temp resource를
  정리한다. 다음 job이 오염된 adapter state를 재사용하지 않게 한다.
- Plugin job/callback panic: 해당 plugin을 session `Faulted`로 전환하고 queued
  contribution/job/result를 폐기한다. 다시 enable할 때 generation과 state를 reset한다.
- Remote job panic: 해당 session epoch를 lost로 만들고 session/stream/child를 닫는다. 다른
  Location lane은 계속 동작한다.

일반 오류, cancel과 panic은 모두 사용자에게 redacted summary만 보여 주며 raw credential,
URL secret, environment와 full command line을 log/snapshot에 넣지 않는다.

<a id="lifecycle"></a>
## 종료, cleanup과 join

lane과 resource를 detach하지 않는다. 정상 종료 순서는 다음과 같다.

1. supervisor가 새 submit을 거부하고 shutdown 상태를 Action으로 알린다.
2. active job의 `CancelHandle`을 모두 호출한다.
3. Remote stream/session/child와 temp writer가 cleanup하도록 control signal을 보낸다.
4. request sender를 닫고 worker가 terminal result/cleanup을 마치게 한다.
5. Core Local, Plugin Read, Git Transport(구현된 경우), 각 Remote worker의 `JoinHandle`을 모두
   join한다.
6. 그 뒤 terminal RAII guard를 복구하고 process를 종료한다.

`std::thread::join`에는 timeout이 없으므로 adapter의 blocking call은 deadline과 cancel/close로
반드시 유한하게 끝나야 한다. 종료 시 thread를 누락하거나 의도적으로 leak하는 fallback은
허용하지 않는다. app shutdown, plugin disable, Location disconnect/config 삭제와 test Drop
모두 같은 lifecycle helper를 사용한다.

## 구현 순서

1. **공통 M2**: 현재 EffectWorker를 bounded Core Local lane protocol과 공통
   OperationId/Cancel/Deadline/JobControl로 확장하고 local mutation 하나만 허용한다.
2. **Git branch**: G0가 공통 primitive로 bounded Plugin Read lane을 추가하고, G2가 Git local
   mutation에 공통 MutationLease를 적용하며, G3가 bounded Git Transport lane과
   network-coupled mutation lease 경계를 추가한다.
3. **Remote branch**: S0/S1이 공통 primitive로 Remote lane supervisor, per-Location worker와
   session epoch를 추가하고, S2가 local endpoint transfer에 MutationLease와 공통 OperationId를
   적용한다.

Git branch와 Remote branch는 M2 이후 서로의 선행 조건이 아니다. 한쪽이 없어도 generic
unsupported seam으로 독립 승인할 수 있고, 둘 다 존재하는 첫 gate부터 실제 교차 test를 연다.

앞 단계가 필요로 하지 않는 lane, generic executor 또는 async abstraction을 미리 만들지
않는다.

## 테스트 의무

- 느린 Plugin Read 중에도 Core directory Action/render가 진행됨
- 느린/취소 중인 Git transport에서도 UI와 Plugin Read가 진행되며 Core mutation은 lease
  규칙대로 Busy 또는 실행됨
- 한 Remote timeout 중에도 Local과 다른 Remote가 진행됨
- 각 lane의 동시 실행 수가 계약값 1을 넘지 않음
- bounded queue full에서 UI submit이 block하지 않고 coalesce/Busy가 결정적임
- Core mutation과 Git mutation의 최대 동시 실행이 합계 1임
- cancel이 blocking job queue 뒤에 대기하지 않고 handle을 즉시 호출함
- deadline/cancel/error/panic마다 terminal result가 정확히 한 번 도착함
- stale listing은 미적용하지만 transfer terminal cleanup 기록은 보존함
- plugin panic은 plugin만 Faulted로 만들고 Core/다른 plugin은 계속 동작함
- Remote panic은 해당 Location session만 잃음
- Drop/shutdown 후 worker/child/session/temp resource와 JoinHandle이 남지 않음

테스트는 barrier, recording fake, FixedClock과 짧은 deterministic deadline을 사용한다.
실제 sleep과 외부 network에 의존하는 timing assertion은 integration gate로 분리한다.

## 결과

### 장점

- 느린 Git/SSH 작업이 기본 파일 탐색을 막지 않는다.
- 영역별 순서는 직렬이라 cache, session과 conflict 처리가 결정적이다.
- queue/thread/session 수가 bounded라 부하와 shutdown을 설명할 수 있다.
- 공통 mutation lease가 Core와 Git의 동시 변경을 막는다.
- 범용 async runtime 없이 현재 blocking adapter를 안전하게 수용한다.

### 비용

- supervisor, queue full 정책과 lifecycle test가 추가된다.
- cross-lane 결과 identity와 refresh coalescing이 필요하다.
- adapter가 cancel/deadline handle을 제공해야 한다.

## 재검토 조건

- 측정 결과 serial Plugin Read 또는 Location lane이 승인된 성능 목표를 충족하지 못할 때
- 하나의 Remote Location 안에서 독립 stream 병렬화가 제품 요구로 승인될 때
- 세 개 이상 기능 영역이 동일한 lane boilerplate를 반복해 generic executor가 실제로
  단순해질 때
- 선택한 production transport가 async runtime 없이는 검증된 cancellation/deadline을
  제공할 수 없을 때

재검토 시에도 UI 비차단, bounded concurrency, result identity, mutation lease와
cleanup/join 계약은 유지한다.
