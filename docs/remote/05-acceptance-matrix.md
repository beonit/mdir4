# SSH Remote / Remote Drive 수용 기준

Remote production은 아직 구현되지 않았다. 모든 상세 행과 gate의 초기 상태는
`Not started`, Evidence는 `—`다. 허용 상태는 `Not started`, `In progress`, `Passed`,
`Blocked`뿐이다. 완료로 바꿀 때 Status와 실제 test/ADR/snapshot/CI/수동 링크를 같은 변경에서
갱신한다.

각 상세 ID는 정확히 한 단계에만 속한다. 후속 단계가 이전 계약을 깨지 않았다는 증거는
후속 단계의 별도 regression ID에 기록하며 이전 행을 다시 열거나 “S0 부분 Passed”처럼 쓰지
않는다. 각 gate의 `필수 상세 ID`가 모두 개별 `Passed`이고 Evidence가 있어야 gate를
`Passed`로 바꿀 수 있다.

Git과 Remote는 독립 트랙이다. counterpart가 아직 없으면 해당 단계의 `INTEG` 행은 generic
unsupported seam 자동 test로 `Passed` 처리하고 Evidence에
`counterpart: N/A — Git not implemented`를 함께 기록한다. Git이 존재하는 단계에서는 실제
양방향 test가 필수다. `Not applicable`은 Status가 아니다.

## S0 — Location foundation

| ID | 수용 기준 | 카드 | 자동 검증 | 수동 검증 | Status | Evidence |
|---|---|---|---|---|---|---|
| S0-LOC-01 | immutable config id가 Location identity이고 mutable name/description과 분리됨 | S0-01,02 | identity/config roundtrip | 없음 | Not started | — |
| S0-LOC-02 | EntryId에 Location이 중복되지 않고 DEV/PROD same path가 구분됨 | S0-01 | type/identity unit | 없음 | Not started | — |
| S0-LOC-03 | protocol path bytes 보존, display→protocol 재변환 0 | S0-00,01 | byte fixture/source scan | escaped name | Not started | — |
| S0-LOC-04 | byte-component normalize/parent/root containment가 backend 전 결정됨 | S0-01 | containment table/calls 0 | 없음 | Not started | — |
| S0-LOC-05 | Local adapter가 v1 동작/fast path를 회귀시키지 않음 | S0-03 | v1 full gate/call count | Local walkthrough | Not started | — |
| S0-CAP-01 | S0 capability는 READ만 정의하고 mutation/RESUME bit·placeholder가 없음 | S0-03 | capability unit/source | 없음 | Not started | — |
| S0-CAP-02 | DOWNLOAD/Edit capability가 없고 Remote F4는 disabled definition만 가짐 | S0-03 | enum/source/mapper | F4 reason | Not started | — |
| S0-CFG-01 | base timeout/schema default/range와 `.ssh/config` literal Host discovery가 결정적으로 동작함 | S0-02 | config/discovery tests | 설정 확인 | Not started | — |
| S0-CFG-02 | optional id/name/description/host/root/root_hex/RO override와 field-path 오류가 정확함 | S0-02 | invalid matrix | 오류 문구 | Not started | — |
| S0-CFG-03 | credential/resolved endpoint 관련 config field가 없음 | S0-02 | schema/secret scan | 없음 | Not started | — |
| S0-CANCEL-01 | queue 밖 thread-safe handle이 controlled blocking Fake call을 preempt함 | S0-00,04,06 | block sequence | 없음 | Not started | — |
| S0-CANCEL-02 | 모든 foundation request가 monotonic deadline을 받고 Timeout/Cancelled를 구분함 | S0-04,06 | deadline/race | 없음 | Not started | — |
| S0-CANCEL-03 | cancel/timeout/quit 뒤 callback/worker/session handle leak와 orphan join이 0 | S0-04,06 | leak/join counters | process 확인 | Not started | — |
| S0-ASYNC-01 | Fake connect/list/lstat/read job이 UI thread에서 0회 | S0-05,06 | thread recording | 없음 | Not started | — |
| S0-ASYNC-02 | stale ViewGeneration result가 현재 listing을 덮지 않음 | S0-04,06 | out-of-order | 없음 | Not started | — |
| S0-ASYNC-03 | stale SessionEpoch result가 current connection을 덮지 않음 | S0-04,06 | reconnect race | 없음 | Not started | — |
| S0-ASYNC-04 | OperationId terminal/cleanup state가 view identity와 분리됨 | S0-04,06 | identity scenario | 없음 | Not started | — |
| S0-ASYNC-05 | 한 Remote lane fault가 Local/다른 Remote/runtime을 막지 않음 | S0-05,06 | lane fault | 오류 후 Local | Not started | — |
| S0-ASYNC-06 | queue 16/worker 4 상한과 coalesce/Busy/LimitReached가 non-blocking | S0-06 | injected capacity/limit | 동시 상태 | Not started | — |
| S0-ASYNC-07 | Remote job panic은 해당 session epoch만 잃고 다른 lane은 계속됨 | S0-05,06 | panic isolation | 없음 | Not started | — |
| S0-AUTH-01 | ConnectRequest는 id/alias만, common context는 cancel/deadline만 전달 | S0-00,04 | request recording/type scan | 없음 | Not started | — |
| S0-AUTH-02 | transport ADR가 OpenSSH config/agent/known_hosts/noninteractive/path/cancel/Windows를 승인 | S0-00 | ADR checklist/spike | 검토 | Not started | — |
| S0-AUTH-03 | isolated harness에 host-key bypass/auto-accept가 없음 | S0-08 | changed-key/source scan | 없음 | Not started | — |
| S0-AUTH-04 | state/error/log/snapshot에 credential/resolved endpoint/key path가 없음 | S0-02,05,06 | redaction scan | 오류 화면 | Not started | — |
| S0-UI-01 | Local과 `.ssh/config`에서 발견한 literal Remote alias가 같은 picker에 표시됨 | S0-07 | component snapshot | 키 감각 | Not started | — |
| S0-UI-02 | Main F3 View/MCD F3 Drive/F12 Locations context가 충돌하지 않음 | S0-07 | context mapper | 각 화면 | Not started | — |
| S0-UI-03 | picker Up/Down/Enter/Esc와 English Help label이 Registry와 일치하고 사용자 name/path를 보존 | S0-07 | mapper/scenario/snapshot | 키 감각 | Not started | — |
| S0-INTEG-01 | generic seam이 Remote를 Unsupported로 매핑하고 Git job/dynamic contribution 0을 증명 | S0-03,09 | generic seam 또는 actual Git | 조건부 | Not started | — |
| S0-TEST-01 | Fake read surface가 deterministic하고 write/mutation method가 없음 | S0-05 | Fake contract | 없음 | Not started | — |
| S0-TEST-02 | exact feature command의 isolated sshd harness가 home/network 격리와 teardown을 증명 | S0-08 | harness self-test | 없음 | Not started | — |
| S0-TEST-03 | foundation scenario/snapshot이 실제 schema와 context keymap을 통과 | S0-07,09 | parser/scenario | 없음 | Not started | — |

## S1 — Remote browse, Viewer and Download

| ID | 수용 기준 | 카드 | 자동 검증 | 수동 검증 | Status | Evidence |
|---|---|---|---|---|---|---|
| S1-LOC-01 | configured root에서 Backspace/Enter가 멈추고 browse call이 root 밖으로 나가지 않음 | S1-02 | navigation/calls 0 | root Backspace | Not started | — |
| S1-CAP-01 | READ가 Browse/Lstat/View/Download/Refresh를 허용하고 F4/mutation은 disabled | S1-02,03 | command/capability | 상태 reason | Not started | — |
| S1-CANCEL-01 | production connect/list/lstat/read를 queue 밖 handle이 실제 중단 | S1-01 | controlled+isolated | slow cancel | Not started | — |
| S1-CANCEL-02 | production blocking read request가 deadline과 Timeout/Cancelled를 구분 | S1-01,04,05 | deadline matrix | timeout 문구 | Not started | — |
| S1-CANCEL-03 | browse/view/download cancel·quit 뒤 session/stream/buffer/temp/worker leak 0 | S1-01,04,05,06 | resource counters | process 확인 | Not started | — |
| S1-ASYNC-01 | connect/list/lstat/open_read가 UI thread에서 0회 | S1-01~05 | thread recording | slow network | Not started | — |
| S1-ASYNC-02 | 빠른 path 변경 뒤 stale listing/viewer result가 미적용 | S1-02,04 | out-of-order | path 이동 | Not started | — |
| S1-ASYNC-03 | reconnect 뒤 old session result가 미적용 | S1-06 | reconnect race | 재연결 | Not started | — |
| S1-ASYNC-04 | Download terminal/cleanup이 화면 전환 뒤 operation history에 남음 | S1-05 | operation scenario | 전송 중 이동 | Not started | — |
| S1-ASYNC-05 | network 지연 중 탐색/resize/help/Local 전환/quit 가능 | S1-06 | controlled gate | offline/slow | Not started | — |
| S1-ASYNC-06 | read error/panic이 해당 session만 잃고 Local/다른 Remote는 계속됨 | S1-01,06 | fault/panic scenario | 오류 후 Local | Not started | — |
| S1-AUTH-01 | production connect request가 stable id/OpenSSH alias boundary를 지킴 | S1-01 | request recording | 실제 alias | Not started | — |
| S1-AUTH-02 | OpenSSH config/Include/ProxyJump/agent/known_hosts를 우회 없이 사용 | S1-01 | isolated integration | Windows | Not started | — |
| S1-AUTH-03 | password/keyboard-interactive prompt 없이 InteractiveAuthRequired | S1-01 | auth-required fixture | 미등록 agent | Not started | — |
| S1-AUTH-04 | unknown/changed host key 자동 승인과 validation-disable가 0 | S1-01 | changed-key/source scan | changed key | Not started | — |
| S1-AUTH-05 | production error/log/snapshot에 credential/resolved endpoint/key path가 0 | S1-01,07 | redaction scan | 오류 화면 | Not started | — |
| S1-UI-01 | Remote built-in copy는 영어이고 등록 name/protocol display path를 번역하지 않음 | S1-03,04,05 | snapshot/message scan | 80×25 | Not started | — |
| S1-BROWSE-01 | read_dir/lstat/navigation과 symlink no-follow가 Fake/real에서 일치 | S1-02 | conformance | 실제 DEV | Not started | — |
| S1-BROWSE-02 | Local과 같은 adaptive columns/navigation/selection UX | S1-02,03 | navigation/snapshot | 키 감각 | Not started | — |
| S1-BROWSE-03 | path/RO/connection/last-result 상태가 모호하지 않음 | S1-03,06 | snapshot matrix | 80×25 | Not started | — |
| S1-BROWSE-04 | Enter/F3 Viewer가 32 MiB bounded memory를 지키고 OS launcher/private temp 0 | S1-03,04 | size/thread/process/temp matrix | Viewer | Not started | — |
| S1-BROWSE-05 | disconnect는 현재 last-visible listing만 보존하고 retry/cache hit를 만들지 않음 | S1-06 | state/call count | network 끊기 | Not started | — |
| S1-BROWSE-06 | R/Ctrl+R이 cache 없는 network reload 한 번을 실행 | S1-02 | command/call count | refresh | Not started | — |
| S1-TRANSFER-01 | Remote source F5가 Local-only dialog와 two-lane bridge/local lease/temp publish를 사용 | S1-05 | route/ownership/fault | Download | Not started | — |
| S1-TRANSFER-02 | Download가 six conflict choices, monotonic ≤20 Hz progress, cancel/timeout을 처리 | S1-05 | conflict/progress | 큰 파일 | Not started | — |
| S1-TRANSFER-03 | Download 실패/cancel temp cleanup과 cleanup warning이 기존 target을 보존 | S1-05 | artifact assertion | 강제 중단 | Not started | — |
| S1-LAST-01 | last-visible listing이 reusable TTL/LRU cache/session registry와 API로 분리됨 | S1-06 | API/source test | 상태 확인 | Not started | — |
| S1-TEST-01 | production reader가 Fake read contract와 동일하고 사용자 home/network를 기본 test에서 읽지 않음 | S1-01,02,07 | conformance | 없음 | Not started | — |
| S1-TEST-02 | exact isolated command가 real browse/view/download를 실행(ignored 0 아님) | S1-01,07 | remote-integration CI | Windows adapter | Not started | — |
| S1-TEST-03 | S1 scenario/snapshot과 fault fixture가 실제 schema를 통과 | S1-03~07 | parser/scenario | 없음 | Not started | — |
| S1-PERF-01 | 10k listing과 blocked network key→frame 목표를 release에서 충족 | S1-07 | release perf | 체감 확인 | Not started | — |
| S1-PLAT-01 | Windows 10/11 OpenSSH config/agent/known_hosts/ProxyJump와 browse/view/download 수동 matrix 통과 | S1-07 | 없음 | 6개 terminal/host 조합 | Not started | — |

## S2 — Remote Drive transfer and mutation

| ID | 수용 기준 | 카드 | 자동 검증 | 수동 검증 | Status | Evidence |
|---|---|---|---|---|---|---|
| S2-LOC-01 | mutation source/target root containment와 same/subtree 검사가 backend 전에 끝남 | S2-02 | route/planner calls 0 | 없음 | Not started | — |
| S2-CAP-01 | S2가 UPLOAD/RENAME/MKDIR/DELETE/SERVER_COPY를 처음 추가하고 route별 RO가 command/planner/backend에서 일치 | S2-01,02,07 | capability/route matrix | PROD reason | Not started | — |
| S2-CAP-02 | DOWNLOAD/Edit capability가 추가되지 않고 Remote F4는 계속 disabled | S2-01,07 | enum/source/effect 0 | F4 reason | Not started | — |
| S2-CANCEL-01 | upload/move/mutation을 queue 밖 handle이 중단 | S2-03~08 | controlled gates | 큰 파일 cancel | Not started | — |
| S2-CANCEL-02 | write/delete blocking call이 deadline과 Timeout/Cancelled를 구분 | S2-03~08 | deadline/race | timeout | Not started | — |
| S2-CANCEL-03 | cancel/timeout/quit 뒤 writer/temp/stream/child/worker leak 0 | S2-03~08 | leak counters | process 확인 | Not started | — |
| S2-ASYNC-01 | write/publish/rename/mkdir/remove가 UI thread에서 0회 | S2-01~08 | thread recording | slow mutation | Not started | — |
| S2-ASYNC-02 | transfer/mutation terminal과 cleanup이 화면 전환 뒤 보존 | S2-03~08 | operation scenario | 전송 중 이동 | Not started | — |
| S2-ASYNC-03 | slow write 중 resize/help/Local 전환/quit가 동작 | S2-08 | controlled gate | offline/slow | Not started | — |
| S2-ASYNC-04 | endpoint/job panic은 해당 session만 잃고 반대 endpoint/Local/다른 Remote를 정리·유지 | S2-08 | panic/fault | 오류 후 Local | Not started | — |
| S2-SEC-01 | mutation/transfer state/log/snapshot secret scan 0 | S2-09 | redaction scan | 오류 화면 | Not started | — |
| S2-TRANSFER-01 | Local→Remote F5 Upload가 remote temp+publish로 incomplete final name 0 | S2-03 | Fake+real route | Upload | Not started | — |
| S2-TRANSFER-02 | F6 Local→Remote/Remote→Local Move가 copy 성공 뒤 source delete하고 Busy/error partial을 보존 | S2-03 | two-route lease/delete matrix | 양방향 Move | Not started | — |
| S2-TRANSFER-03 | same LocationId F5/F6가 server copy/rename 또는 안전한 stream/copy-delete fallback | S2-04 | route/real contract | 실제 DEV | Not started | — |
| S2-TRANSFER-04 | 서로 다른 Remote F5/F6가 effect/backend 호출 전 Unsupported | S2-02,04 | zero-call | 없음 | Not started | — |
| S2-TRANSFER-05 | S2 route가 six conflict choices, monotonic/coalesced progress, cancel/timeout을 처리 | S2-03,04,08 | conflict/fault | 큰 파일 | Not started | — |
| S2-TRANSFER-06 | success는 writer-drop 뒤 publish, error/cancel/panic은 writer-drop 뒤 discard 1회이며 cleanup warning/source-delete partial이 숨겨지지 않음 | S2-01,03,04,08 | call order/artifact/result | 강제 중단 | Not started | — |
| S2-MUT-01 | Rename/MkDir이 byte-path validation과 성공 refresh 1회를 지킴 | S2-05 | Fake+real | 실제 DEV | Not started | — |
| S2-MUT-02 | Delete가 permanent English warning/별도 확인/root guard를 사용 | S2-06 | confirm/calls | 문구 확인 | Not started | — |
| S2-MUT-03 | recursive copy/delete가 symlink를 follow하지 않고 partial result를 보고 | S2-04,06 | symlink/fault | fixture | Not started | — |
| S2-RO-01 | RO mutation/F6 command가 disabled이고 이유 표시 | S2-07 | command matrix | PROD | Not started | — |
| S2-RO-02 | direct action/planner/backend 우회도 effect 0/ReadOnly | S2-07 | four-layer calls | 없음 | Not started | — |
| S2-RO-03 | RO Browse/Lstat/View/Download/Refresh는 허용 | S2-07 | allow matrix | PROD | Not started | — |
| S2-UI-01 | F5/F6 route dialog가 source/target Location/path를 영어로 표시하고 사용자 text를 보존 | S2-03,04,09 | mapper/dialog snapshots | 키 감각 | Not started | — |
| S2-INTEG-01 | Git이 있으면 Remote에서 discover/job/dynamic contribution 0, 없으면 generic seam 재검증 | S2-02,09 | both-order/unsupported | 조건부 | Not started | — |
| S2-INTEG-02 | Local endpoint write/delete가 common lease, Remote-only mutation은 Location lane 직렬화 | S2-02,03,09 | contention/overlap 0 | 장시간 작업 | Not started | — |
| S2-TEST-01 | Fake writer가 S2에서만 추가되고 read regression/RO/fault contract를 지킴 | S2-01 | Fake contract | 없음 | Not started | — |
| S2-TEST-02 | exact isolated command가 upload/move/mutation을 실제 실행 | S2-09 | remote-integration CI | Windows adapter | Not started | — |
| S2-TEST-03 | S2 route/RO/fault scenario와 snapshot이 실제 schema를 통과 | S2-09 | parser/scenario | 없음 | Not started | — |
| S2-PERF-01 | blocked transfer/mutation에서도 UI 목표와 progress ≤20 Hz를 충족 | S2-08,09 | release perf | 체감 확인 | Not started | — |
| S2-PLAT-01 | Windows 10/11 upload/download/bidirectional Move/cancel/RO 수동 matrix 통과 | S2-09 | 없음 | 6개 terminal/host 조합 | Not started | — |

## S3 — Cache, registration and hardening

| ID | 수용 기준 | 카드 | 자동 검증 | 수동 검증 | Status | Evidence |
|---|---|---|---|---|---|---|
| S3-CFG-01 | cache field migration/default/range와 기존 S0 config가 roundtrip됨 | S3-01 | config migration | 설정 확인 | Not started | — |
| S3-CACHE-01 | TTL/LRU cache가 last-visible listing/session registry와 분리됨 | S3-01 | API/source | 상태 확인 | Not started | — |
| S3-CACHE-02 | key/default/off/expiry/eviction/invalidation이 injected Clock으로 결정적 | S3-01 | cache/call matrix | 재방문 | Not started | — |
| S3-CACHE-03 | R/Ctrl+R manual refresh가 cache를 우회하고 mutation refresh 1회를 지킴 | S3-01 | command/call count | refresh | Not started | — |
| S3-REG-01 | SSH Host discovery가 후보를 자동 등록하지 않고 credential field를 반환하지 않음 | S3-02,03 | discovery/config calls 0 | browser | Not started | — |
| S3-REG-02 | 등록/편집/삭제가 stable id를 지키고 Mdir config만 원자적으로 변경 | S3-03 | before/after | SSH 파일 확인 | Not started | — |
| S3-REG-03 | registration widget에 credential field가 없고 name/description 변경이 identity를 보존 | S3-03 | UI/config/secret scan | UI 확인 | Not started | — |
| S3-HARD-01 | S3가 RESUME과 non-optional resumable access variant를 처음 추가하고 token/source와 route별 destination partial SHA-256·길이를 검증하며 mismatch별 오류 뒤 자동 restart/publish 0 | S3-04 | capability/identity/hash/partial | 대용량 재연결 | Not started | — |
| S3-HARD-02 | lstat/permission/non-UTF8 path/slow server/config migration/플랫폼 배포를 검증 | S3-05 | conformance/migration | 플랫폼 matrix | Not started | — |
| S3-SEC-01 | host discovery/resume/cache state·log·snapshot secret scan 0 | S3-02~06 | redaction scan | 오류 화면 | Not started | — |
| S3-UI-01 | cache/registration/resume built-in copy는 영어이고 사용자 name/path를 보존 | S3-01~06 | message/snapshot | walkthrough | Not started | — |
| S3-INTEG-01 | 조건부 actual Git integration과 Local/Remote/common lease 전체 regression | S3-06 | both-order/full gate | 조건부 | Not started | — |
| S3-TEST-01 | S3 scenario/snapshot/isolated real suite와 Windows/Linux/macOS 범위를 기록 | S3-06 | full automated matrix | platform | Not started | — |
| S3-EXCL-01 | Remote Edit/Git/SSH terminal/cross-Remote Copy/Move가 command/effect/backend에 없음 | S3-06 | source/zero-call | 없음 | Not started | — |

## 단계 승인 gate

아래 `필수 상세 ID`는 축약된 범위가 아니라 해당 행들을 개별 확인하라는 목록이다. `선행
Gate`는 상세 ID 대신 쓰는 항목이 아니다. Evidence에 “모두 통과”만 적지 말고 상세 행과 선행
gate를 먼저 각각 Passed로 갱신한다.

| Gate ID | 단계 | 필수 상세 ID | 선행 Gate | 카드 | Status | Evidence |
|---|---|---|---|---|---|---|
| S0-GATE-01 | S0 model/config | S0-LOC-01/02/03/04/05, S0-CAP-01/02, S0-CFG-01/02/03 | 없음 | S0-01~03 | Not started | — |
| S0-GATE-02 | S0 runtime | S0-CANCEL-01/02/03, S0-ASYNC-01/02/03/04/05/06/07 | 없음 | S0-04~06 | Not started | — |
| S0-GATE-03 | S0 picker | S0-UI-01/02/03, S0-TEST-03 | 없음 | S0-07 | Not started | — |
| S0-GATE-04 | S0 security harness | S0-AUTH-01/02/03/04, S0-TEST-02 | 없음 | S0-00,08 | Not started | — |
| S0-GATE-05 | S0 final | S0-INTEG-01, S0-TEST-01 | S0-GATE-01/02/03/04 | S0-09 | Not started | — |
| S1-GATE-01 | S1 production/security | S1-CAP-01, S1-CANCEL-01/02/03, S1-AUTH-01/02/03/04/05 | S0-GATE-05 | S1-01 | Not started | — |
| S1-GATE-02 | S1 browse/view | S1-LOC-01, S1-UI-01, S1-BROWSE-01/02/03/04/06 | S1-GATE-01 | S1-02~04 | Not started | — |
| S1-GATE-03 | S1 download/fault | S1-ASYNC-01/02/03/04/05/06, S1-BROWSE-05, S1-TRANSFER-01/02/03, S1-LAST-01 | S1-GATE-01/02 | S1-05,06 | Not started | — |
| S1-GATE-04 | S1 automated final | S1-TEST-01/02/03, S1-PERF-01 | S1-GATE-01/02/03 | S1-07 | Not started | — |
| S1-GATE-05 | S1 Windows | S1-PLAT-01 | S1-GATE-01/02/03/04 | S1-07 | Not started | — |
| S2-GATE-01 | S2 writer/planner | S2-LOC-01, S2-CAP-01/02, S2-TEST-01, S2-INTEG-02 | S1-GATE-05 | S2-01,02 | Not started | — |
| S2-GATE-02 | S2 route/mutation | S2-TRANSFER-01/02/03/04/05/06, S2-MUT-01/02/03, S2-UI-01 | S2-GATE-01 | S2-03~06 | Not started | — |
| S2-GATE-03 | S2 RO/fault/security | S2-RO-01/02/03, S2-CANCEL-01/02/03, S2-ASYNC-01/02/03/04, S2-SEC-01 | S2-GATE-01/02 | S2-07,08 | Not started | — |
| S2-GATE-04 | S2 automated final | S2-INTEG-01, S2-TEST-02/03, S2-PERF-01 | S2-GATE-01/02/03 | S2-09 | Not started | — |
| S2-GATE-05 | S2 Windows MVP | S2-PLAT-01 | S2-GATE-01/02/03/04 | S2-09 | Not started | — |
| S3-GATE-01 | S3 cache | S3-CFG-01, S3-CACHE-01/02/03 | S2-GATE-05 | S3-01 | Not started | — |
| S3-GATE-02 | S3 registration | S3-REG-01/02/03 | S3-GATE-01 | S3-02,03 | Not started | — |
| S3-GATE-03 | S3 hardening | S3-HARD-01/02, S3-SEC-01, S3-UI-01 | S3-GATE-01/02 | S3-04,05 | Not started | — |
| S3-GATE-04 | S3 final | S3-INTEG-01, S3-TEST-01, S3-EXCL-01 | S3-GATE-01/02/03 | S3-06 | Not started | — |

단계 완료 규칙은 다음과 같다.

- S0: `S0-GATE-01~05`가 모두 Passed여야 S0 완료다.
- S1: `S1-GATE-01~05`가 모두 Passed여야 S1 완료다.
- S2: `S2-GATE-01~05`가 모두 Passed여야 Remote Drive MVP 완료다.
- S3: `S3-GATE-01~04`가 모두 Passed여야 S3 완료다.

S3 완료 뒤에도 Remote Edit, Remote Git, SSH terminal, cross-Remote Copy/Move는 별도 제품
계약/ADR 전까지 제외다.
