# Mdir4 SSH Remote / Remote Drive 계획

플랫폼 범위는 Linux/macOS다. 하위 문서에 남은 Windows 수동 검증 표현은 원본 계획 기록이며
Linux/macOS 동등 검증으로 해석하고 Windows 결과를 완료 조건으로 요구하지 않는다.

이 폴더는 SSH/SFTP 파일 시스템을 Core의 새로운 `Location` 유형으로 추가하는 v1 이후
로드맵이다. 별도 FTP 클라이언트나 범용 plugin 계획이 아니다. 사용자는 Local drive와
등록된 Remote를 같은 Location picker에서 선택하고 같은 목록 UX로 탐색한다.

Remote 구현은 아직 시작하지 않았다. 기존 v1 `M0~R1` 완료 조건과 Git `G0~G3` 완료
조건을 바꾸지 않는다.

## 문서 권위와 읽는 순서

1. [`01-product-contract.md`](01-product-contract.md) — 사용자 동작, 설정, 보안, 범위
2. [`ADR-005`](../architecture/adr-005-background-work-lanes.md) — 승인된 lane/cancel/lease 계약
3. [`02-architecture.md`](02-architecture.md) — identity/path/cancel/worker/backend 계약
4. [`03-task-cards.md`](03-task-cards.md) — 유일한 실행 순서와 카드별 증거
5. [`04-test-plan.md`](04-test-plan.md) — fake, isolated `sshd`, fault, snapshot, 수동 검증
6. [`05-acceptance-matrix.md`](05-acceptance-matrix.md) — 단계별 승인 기준
7. [`requirements-original.md`](requirements-original.md) — 보존한 사용자 원문

S0-00이 transport/path ADR을 승인하면 그 ADR을 이 목록의 ADR-005 다음에 추가한 뒤 S0-01을
시작한다.

충돌 시 위 순서에서 앞선 문서가 제품 의미를 정하고, 뒤 문서는 구현/검증 방법만
구체화한다. 공통 작업 절차는
[`../implementation-plan/06-agent-runbook.md`](../implementation-plan/06-agent-runbook.md),
실제 상태와 증거는
[`../implementation-plan/progress.md`](../implementation-plan/progress.md)에 기록한다.

## 고정한 결정

| 항목 | 확정 결정 |
|---|---|
| 릴리스 | R1 이후 독립 `S0~S3`; S2가 Remote Drive MVP |
| 모델 | Core Location 확장; Remote를 `PathBuf`나 plugin으로 위장하지 않음 |
| 설정 identity | stable `id`와 변경 가능한 `name`/`description`을 분리 |
| 경로 | protocol bytes를 보존하고 display text와 절대 재혼합하지 않음 |
| 취소 | worker 밖의 thread-safe `CancelHandle`이 token을 신호; 모든 blocking call은 deadline도 받음 |
| 결과 identity | 화면 load는 view generation, 연결은 session epoch, 작업은 `OperationId`로 분리 |
| 인증/대상 | `~/.ssh/config`의 literal Host alias/config/agent/known_hosts에 위임; credential UI/저장 없음 |
| read-only | UI/reducer/planner/backend 모든 계층에서 mutation 차단 |
| Viewer | F3와 regular-file Enter는 Mdir4 Viewer; OS launcher 호출 0 |
| picker key | Main F3는 View; MCD context F3 Drive와 F12 Locations가 picker를 엶 |
| Editor | Remote Edit/Save는 S0~S3 전체에서 제외; F4 disabled |
| cache | S1은 마지막 visible listing만 유지; reusable TTL/LRU cache는 S3 |
| resume | S3 opt-in; source 전체와 partial prefix SHA-256/길이 검증, mismatch별 오류, 자동 restart 없음 |
| SSH Host discovery | `~/.ssh/config`(+Include)의 literal Host alias가 picker의 직접 접속 대상; optional override만 Mdir4 config에 저장 |
| Git | Remote에서 Git discover/job/dynamic contribution 0; 정적 command는 disabled; 별도 후속 ADR 전 Local 전용 |

## 실행 순서

```text
R1
 └─ S0-00 Contract/ADR Gate
      └─ S0 Location Foundation
           └─ S1 Remote Browse/View/Download
                └─ S2 Transfer/Mutation ── Remote Drive MVP
                     └─ S3 Cache/Registration/Hardening
```

| 단계 | 사용자 가치 | 종료 조건 |
|---|---|---|
| S0 | Local/Remote identity와 picker, deterministic fake | 경로/취소/backend ADR와 foundation gate |
| S1 | connect, browse, stat/lstat, Viewer, Download | read-only remote 수용 + isolated `sshd` |
| S2 | Upload, Local↔Remote Move, same-remote Copy/Move, Rename/MkDir/Delete | RO/취소/부분 실패/실서버 mutation gate |
| S3 | TTL/LRU cache, host discovery/registration, resume 강화 | migration/대용량/플랫폼 hardening gate |

## 범위 제어

- `S0-00` 승인 전 production SSH dependency와 Location public type을 확정하지 않는다.
- S0 Fake는 connect/list/stat/lstat/read까지만 구현한다. write/mutation fault는 S2 카드에서
  Fake 계약을 확장한다.
- S1 완료 전 Remote mutation command를 활성화하지 않는다.
- Remote Edit/Save, SSH terminal, Remote Git, 서로 다른 Remote 간 Copy/Move는 S3까지도 제외한다.
- 설정에는 인증 username/password/private-key path/passphrase를 넣지 않고, runtime
  state/log/snapshot에는 OpenSSH가 해석한 인증 username과 endpoint/credential을 넣지 않는다.
  다만 사용자가 등록한 root/path byte의 안전한 표시 문자열은 경로 UI에 표시할 수 있다.
- 각 카드 완료 시 카드의 `progress/evidence` checklist를 채우고 최상위 progress에 증거를
  링크한다. 체크되지 않은 카드는 완료로 표시하지 않는다.

## 사용자 원문 요구사항 추적성

기준 원문은 사용자가 제공한
[`Mdir4 SSH Remote / Remote Drive 요구사항`](requirements-original.md) 1~45절이며 SHA-256은
`99f44e40bf5c167563493232a2f2d8cab9ee042dbb9cded5c08fe848f1d78691`이다.
중복 문장을 복사해 세 번째 규격으로 만들지 않고 아래 표로 canonical 계약과 수용 기준에
연결한다.

| 원문 절 | 요구 주제 | canonical 계약 | 수용 ID / 카드 |
|---|---|---|---|
| 1~2, 11, 13, 43 | SSH/SFTP를 Local과 같은 Location UX로 제공 | 제품 계약 1, 4~6 | 단계별 `LOC`/`UI`/`BROWSE` 상세 행; S0-01~07 |
| 3~7, 44 | OpenSSH alias/config/agent, credential 미관리, host-key 검증 | 제품 계약 2~3 | 단계별 `CFG`/`AUTH` 상세 행; S0-00,02,08, S1-01 |
| 8~10, 31~32 | Remote 정의, 짧은 표시명, RO, 등록 삭제 격리 | 제품 계약 2, 10 | `S0-LOC-*`, `S2-RO-*`, `S3-REG-*`; S0-02, S2-07, S3-03 |
| 12, 28~30 | Location picker, SSH Host 후보/등록 | 제품 계약 5 | `S0-UI-*`, `S3-REG-*`; S0-07, S3-02~03 |
| 14~16, 42 | FileSystem 추상화와 기본 read/write 기능 | 아키텍처 1~8 | 단계별 `CAP`/`ASYNC`/`TEST` 상세 행; S0-03~06, S1, S2 |
| 17~19 | Local↔Remote와 same-Remote transfer, cross-Remote 제외 | 제품 계약 9 | `S1-TRANSFER-*`, `S2-TRANSFER-*`; S1-05, S2-01~04 |
| 20~27 | 상태 표시, async, cache/session, disconnect/reconnect | 제품 계약 6~8, 11 | 단계별 `ASYNC`/`BROWSE`, `S3-CACHE-*`; S1-03/06, S3-01 |
| 33~34, 40~41 | SSH terminal/Remote Git/고급 기능 후속 | 제품 계약 12 | 단계별 `INTEG`, `S3-EXCL-01`; S3-06 |
| 35~38 | Fake, snapshot, 지연, RO 자동 검증 | 테스트 계획 2~13 | 단계별 `TEST`/`CANCEL`, `S2-RO-*`; S0-05, S2-01/07/08 |
| 39, 45 | Phase 1/MVP와 완료 조건 | 제품 계약 1, 단계 승인 | `S0-GATE-*`, `S1-GATE-*`, `S2-GATE-*` |

원문을 구체화하면서 고정한 차이:

- 사람이 보는 Remote 이름과 영구 identity 충돌을 막기 위해 config에 immutable `id`와
  optional `description`을 추가했다.
- non-UTF-8 configured root도 protocol bytes를 잃지 않도록 `root`와 상호 배타인
  `root_hex`를 추가했다.
- 원문의 “가능하면 마지막 캐시 목록”은 S1 현재 화면의 `last_visible_listing`과 S3 reusable
  TTL/LRU cache로 분리했다.
- SSH Host discovery/등록 UI는 탐색 MVP의 선행 조건이 아니므로 S3로 이동했다.
- symlink 자체 `lstat`은 안전한 S1 탐색에 필요해 강화 단계보다 앞당겼다.
- 전송 progress/cancel은 비동기 MVP의 안전 조건이므로 S1/S2부터 필수다.
- 원문이 RO 금지 목록에 둔 Edit Save는 Remote 전체 MVP에서 명시적으로 제외하고 F4를 항상
  disabled로 고정했다.
- Main의 F3 View와 MCD context의 F3 Drive가 충돌하지 않도록 context keymap을 명시했다.
