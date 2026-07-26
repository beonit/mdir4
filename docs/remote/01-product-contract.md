# SSH Remote / Remote Drive 제품 계약

이 문서는 Remote Location의 사용자 동작과 보안 경계를 고정한다. Local에서는 v1 제품
계약이 우선하고, Remote에서는 이 문서가 우선한다.

Remote built-in label, Help, dialog와 error copy는 Core의 영어 UI 계약을 따른다. 등록
display name과 protocol path에서 만든 안전한 display text는 번역하지 않는다.

## 1. 제품 정의와 단계

Remote는 SFTP 파일 시스템을 Core `Location`으로 추가한다.

```text
Locations
├── Local
│   ├── C:    System
│   └── D:    Data
└── Remote
    ├── DEV   Development
    └── PROD  Production [RO]
```

| 단계 | 포함 | 제외/비활성 |
|---|---|---|
| S0 | identity/path/config, Local adapter, fake read backend, picker, connection model, backend ADR | production browsing/mutation |
| S1 | connect/reconnect, list/stat/lstat, symlink metadata, Viewer, Remote→Local Download | F4 Edit, 모든 Remote mutation |
| S2 | Upload, Local↔Remote Move, same-Remote Copy/Move, Rename, MkDir, permanent Delete, RO enforcement | cross-Remote Copy/Move, Edit |
| S3 | reusable TTL/LRU cache, SSH Host discovery/registration UI, resume/metadata/platform hardening | cross-Remote Copy/Move, Edit, Remote Git, SSH terminal |

Remote 오류는 Local, 다른 Remote, reducer/render, 앱 종료를 중단시키지 않는다.

## 2. canonical 설정

```toml
[remote]
connect_timeout_seconds = 15
operation_timeout_seconds = 60
# S3에서 아래 cache 두 필드가 schema에 추가된다.
directory_cache_ttl_seconds = 30
directory_cache_max_entries = 128

[[remote.locations]]
id = "dev"
name = "DEV"
description = "Development"
host = "dev"
root = "/home/ubuntu"
read_only = false

[[remote.locations]]
id = "prod"
name = "PROD"
description = "Production"
host = "production"
root = "/var/www"
read_only = true
```

S0 schema는 timeout과 `remote.locations`까지만 추가한다. S3-01 migration이 cache 두 필드를
추가한다. 구현되지 않은 미래 field를 미리 수용하거나 조용히 무시하지 않는다.

기본값과 유효성:

| 필드 | 기본/규칙 |
|---|---|
| `connect_timeout_seconds` | 15; `1..=300` |
| `operation_timeout_seconds` | 60; `1..=86400` |
| `directory_cache_ttl_seconds` | 30; `0`은 S3 reusable cache off, 최대 86400 |
| `directory_cache_max_entries` | 128; `0`은 cache off, 최대 4096 |
| `id` | 필수 stable ASCII slug `[a-z0-9][a-z0-9_-]{0,31}`; 대소문자 비구분 unique; rename하지 않음 |
| `name` | 필수 변경 가능한 표시명, trim된 exact 값 unique, 2~12 terminal cells, control/newline 금지 |
| `description` | 기본 `""`; 최대 60 cells, control/newline 금지 |
| `host` | 필수 OpenSSH alias; 공백/control/선행 `-`/URI/credential form 금지 |
| `root` | UTF-8 절대 remote path; `root_hex`와 상호 배타; `/` 허용 |
| `root_hex` | non-UTF-8 root용 lowercase even-length hex bytes; 첫 byte `2f`; `root`와 상호 배타 |
| `read_only` | 기본 `false` |

- identity와 cache key에는 `id`만 사용한다. `name`/`description` 변경은 marks, cursor,
  session, cache identity를 바꾸지 않는다.
- username, password, private key/path, passphrase, port, `HostName`, `ProxyJump`,
  `ProxyCommand` 필드는 unknown-field 오류다.
- 오류에는 파일과 `remote.locations[index].field`를 표시한다.
- 등록/편집/삭제는 Mdir4 설정만 원자적으로 바꾸며 `.ssh/config`, key, agent,
  `known_hosts`를 수정하지 않는다.

## 3. 인증과 보안

- `ConnectRequest`에는 stable Location id와 OpenSSH Host alias만 담고, 호출 시 별도의 Core
  `OperationContext`가 cancel token과 monotonic deadline을 전달한다. 어느 쪽에도 resolved
  endpoint나 credential을 넣지 않는다.
- production adapter는 사용자의 OpenSSH config/agent/known_hosts/host-key 검증을 사용한다.
- password, keyboard-interactive, host-key 확인 prompt가 필요하면 `InteractiveAuthRequired`로
  실패한다. UI에서 credential을 묻지 않는다.
- host-key 검증 비활성화 option/flag/fallback은 production과 test 모두 금지한다.
- 실행 인자는 shell string이 아닌 executable+argument array를 사용한다.
- state, `Debug`, Display, log, panic, snapshot에는 OpenSSH가 해석한 인증 username,
  resolved host/IP/port, private-key path, command line secret를 남기지 않는다. 등록된
  root/path byte에서 만든 안전한 display path는 인증 정보가 아니므로 경로 UI에 표시할 수
  있으며, 그 path segment가 우연히 username과 같은 문자열이어도 redaction 대상으로
  재해석하지 않는다.

사용자 오류 화면은 등록된 `name`, 안전한 display path, error kind만 표시할 수 있다.

## 4. identity와 경로 표시

- `LocationId`는 config `id`에서 만들며 안정적이다.
- entry identity는 `(LocationId, RemotePathBytes)`다. Location 정보를 두 번 중첩하지 않는다.
- protocol path는 원시 bytes를 보존한다. Unix의 non-UTF-8 이름도 read/stat/mutation
  roundtrip이 되거나, backend ADR에서 명시적으로 지원 불가로 거부해야 한다.
- display는 UTF-8 decode 가능한 부분은 그대로, 불가능한 byte는 deterministic escaped form
  (`\\xNN`)으로 렌더링한다. ellipsis 적용 전 display-cell width를 계산한다.
- display text를 protocol request로 다시 parse하거나 mutation target으로 사용하지 않는다.
- `..`/join/containment는 byte component 단위이며 string prefix 비교를 금지한다.

표시 예:

```text
DEV:/home/user/project
PROD:/var/www [RO]
```

기본 UI에 `ssh://user@host:port`나 resolved host를 표시하지 않는다.

## 5. Location picker와 command 맥락

- **Main 화면의 F3는 항상 View**다. MCD 화면 context의 F3 `Drive` 또는 F12 Menu의
  `Locations` command가 같은 Location picker를 연다. Viewer/Dialog 안의 F3 의미도 각
  context keymap이 결정한다.
- 화면과 Help는 같은 CommandRegistry definition에서 키 label을 생성한다.
- picker는 Local과 **등록된** Remote만 표시한다. 발견된 SSH Host는 S3의 별도
  `Discover SSH Hosts` 화면에 표시하며 자동 등록하지 않는다.
- Up/Down, Enter, Esc를 지원한다. Remote Enter는 비동기 connect/load를 시작한다.
- Location 전환 시 현재 marks를 비운다. stable id 기반 cursor/cache 복원은 S3부터다.

## 6. 탐색, metadata, Viewer

- Up/Down/Left/Right/Home/End/PgUp/PgDn/Space/Insert는 Local 목록 모델을 재사용한다.
- directory Enter는 같은 Remote에서 load하고 Backspace/`..`는 configured root에서 멈춘다.
- regular-file Enter와 F3 Viewer command는 Mdir4 Viewer를 연다. Remote regular file에 대한
  OS launcher/process spawn 호출 수는 항상 0이다.
- Remote Edit/Save는 S0~S3 전체 범위에서 제외한다. Remote file에서 F4는 disabled이고
  이유 `Remote editing is not supported.`를 표시한다.
- S1부터 `lstat` 결과로 symlink 자체 metadata를 표시한다. 탐색/재귀/전송은 symlink를
  기본적으로 따라가지 않는다. follow가 필요한 기능은 별도 명시적 command 전에는 없다.
- `R`/`Ctrl+R`은 같은 refresh command다. S1은 network reload, S3은 cache bypass다.

## 7. 연결, 취소, 결과 identity

```text
Disconnected ──connect──▶ Connecting ──success──▶ Connected
      ▲                         │                    │
      └──── disconnect/retry ───┴──failure──▶ Failed│
      └────────────── connection-lost event ────────┘
```

`ConnectionLost`는 별도 화면 state가 아니라 adapter event다. reducer는 이 event를
user-visible `Disconnected` state로 바꾸고, §8의 last-result 표시 규칙을 적용한다.

- connect/list/stat/lstat/read/transfer/mutation은 UI thread에서 실행하지 않는다.
- UI/control lane은 blocking worker와 별도로 보유한 thread-safe `CancelHandle`을 호출한다.
  worker가 자기 queue의 cancel 명령을 기다리는 설계는 금지한다.
- 모든 blocking request는 cancel token과 absolute monotonic deadline을 함께 받는다.
- Connecting/작업 중에도 Esc, resize, Help, Local 전환, quit가 동작한다.
- 자동 무한 재시도하지 않는다. 명시적 Reconnect만 새 session epoch를 시작한다.
- 화면 directory load 결과는 view generation, session lifetime은 session epoch, transfer와
  mutation terminal result는 `OperationId`로 식별한다.
- stale view/session 결과는 현재 화면에 적용하지 않지만, 시작된 operation의 terminal
  success/failure/cancel/cleanup 결과는 화면 이동 후에도 operation history에 반드시 남긴다.

## 8. S1 마지막 목록과 S3 cache

S1:

- 현재 화면의 마지막 성공 listing 하나를 `last_visible_listing`으로 유지할 수 있다.
- 연결이 끊기면 같은 화면에서 `[Disconnected — last result]`로 보여 줄 수 있다.
- 이 값은 다른 path 재방문, 앱 재시작, TTL hit, backend-call 회피에 사용하지 않는다.
- disconnected listing에서는 mutation command를 모두 비활성화한다.

S3:

- `(LocationId, RemotePathBytes)` key의 bounded TTL/LRU directory cache를 추가한다.
- injected monotonic Clock을 사용하고 render는 시간을 읽지 않는다.
- manual refresh는 cache를 우회한다. mutation 성공 후 영향 key를 invalidate하고 refresh 1회.
- cache hit/stale/expired 상태를 명시적으로 표시한다.

SSH session reuse는 cache가 아니다. session registry와 directory cache는 별도 수명이다.

## 9. transfer와 mutation

Remote 전용 F5/F6 command를 새로 만들지 않는다. Main CommandRegistry의 기존 `Copy`(F5)와
`Move`(F6)가 source/target Location을 포함한 destination dialog를 열며, dialog는 항상 두
Location name과 safe display path를 보여 준다.

| source → target | F5 Copy | F6 Move |
|---|---|---|
| Local → Local | Core v1 Copy | Core v1 Move |
| Remote → Local | S1 Download; local sibling temp 후 atomic publish | S2 Download 성공 후 remote source permanent delete |
| Local → Remote | S2 Upload; remote sibling temp 후 publish | S2 Upload 성공 후 local source delete |
| 같은 LocationId Remote → Remote | S2 server-copy 또는 같은 lane stream fallback | S2 rename 우선, 아니면 copy+permanent delete |
| 서로 다른 Remote | backend 호출 전 `Unsupported` | backend 호출 전 `Unsupported` |

S1에서 Remote source의 F5는 Local target에만 enabled다. Remote가 source나 target인 F6와
Local→Remote F5는 S2 전까지 `Remote move/upload requires Remote Drive S2.` 이유로 disabled다.
S2 Move의 delete는 copy/publish가 완전히 성공한 뒤에만 실행한다. Remote source를 지우는
Move와 same-Remote copy+delete fallback은 실행 전에
`Move will permanently remove the remote source after copy.` 확인을 요구한다. delete lease/
capability가 Busy/실패면 copy 성공, delete 미실행 partial result를 보존하고 rollback으로
성공한 destination을 몰래 지우지 않는다.

- destination dialog는 source/target Location과 path를 모두 표시한다.
- Local↔Remote 전송은 bounded two-endpoint bridge를 사용한다. Remote lane은 Local FS를,
  Core Local lane은 SSH session을 직접 호출하지 않으며 UI thread는 chunk를 전달하지 않는다.
- progress는 bytes/files/current item/cancel 가능 여부를 표시하고 최대 20 Hz로 coalesce한다.
- cancel/실패 시 temp cleanup을 시도하고 실패한 safe display path를 warning에 남긴다.
- `Overwrite`, `Overwrite All`, `Skip`, `Skip All`, `Rename`, `Cancel` 여섯 선택과 All의
  현재 OperationId 범위는 Core M2 conflict policy를 그대로 재사용한다.
- same-remote Move의 copy+delete fallback은 destructive step 전 명시적 확인이 필요하다.
- Remote Delete는 휴지통을 가정하지 않으며 Location/path/count와
  `Remote delete is permanent`를 표시한 별도 확인이 필요하다.
- local target 쓰기 또는 local source 삭제에 필요한 공통 mutation lease가 이미 active면
  기다리지 않고 `Busy`로 끝내며 local mutation 호출은 0회다. 앞선 remote-only phase가
  완료된 Move라면 copy 성공/delete 미실행 partial 결과를 명시한다.
- S3 resume는 명시적 opt-in이다. 첫 전송 전에 source 전체 SHA-256과 길이를 계산해
  source fingerprint를 만들고, committed source-prefix SHA-256/길이와 destination temp
  identity를 session-independent token에 보관한다. resume 때 source 전체 지문과 source
  prefix, 해당 route의 destination partial 길이·prefix 지문을 다시 검증한다. capability
  없음은 `Unsupported`, token identity/version 불일치는
  `ResumeTokenInvalid`, source 변경은 `ResumeSourceChanged`, partial 변경은
  `ResumePartialMismatch`다. 어느 경우도 자동 Restart/New으로 바꾸거나 final target을 publish하지
  않는다. fingerprint 준비와 prefix 검증도 progress/cancel/deadline 대상이다. 이어서 복사한
  전체 stream의 SHA-256이 준비한 source 지문과 일치해야만 publish한다. token은 process
  memory에만 있고 config/log/snapshot에 저장하지 않는다.

## 10. read-only와 capabilities

`read_only=true` 또는 capability 없음이면 UI와 backend 모두 같은 결과를 낸다.

| 작업 | RO | capability |
|---|---|---|
| Browse/Stat/Lstat/View/Download/Refresh | 허용 | `READ` |
| Remote→Local Move source | 금지 | `READ + DELETE` |
| Local→Remote Copy/Move target | 금지 | `UPLOAD` + Local source delete용 Core lease(Move만) |
| same-Remote Copy | 금지 | `SERVER_COPY` 또는 `READ + UPLOAD` |
| same-Remote Move | 금지 | `RENAME` 또는 `READ + UPLOAD + DELETE` |
| Rename/MkDir/Delete | 금지 | 각각 `RENAME`/`MKDIR`/`DELETE` |
| Edit/Save | 항상 금지 | Remote capability에 존재하지 않음 |

명령 비활성 이유 우선순위는 `Unsupported` → `ReadOnly` → `Disconnected` →
`Busy` → `CapabilityMissing`이다. reducer/planner는 effect를 만들지 않고 backend도
defense-in-depth로 `ReadOnly`/`Unsupported`를 반환한다.

## 11. 오류와 성능

canonical user-visible error kinds:

```text
InvalidConfig, UnknownLocation, InteractiveAuthRequired, HostKeyRejected,
Timeout, Cancelled, Busy, LimitReached, TooLarge, Disconnected, InvalidPath, PermissionDenied, NotFound,
AlreadyExists, NotEmpty, NoSpace, ReadOnly, Unsupported,
PathEncodingUnsupported, Protocol, Io, CleanupFailed, ResumeTokenInvalid,
ResumeSourceChanged, ResumePartialMismatch
```

- network 지연 중 일반 UI 입력→frame 50 ms 목표를 유지한다.
- timeout과 cancel은 구분하되 둘 다 session/stream/child cleanup을 보장한다.
- partial result는 succeeded/failed/skipped/bytes와 첫 안전한 오류를 보존한다.
- Remote 오류 뒤 Local 탐색과 정상 종료가 가능해야 한다.

## 12. 다른 트랙과의 경계

- Git built-in의 discover/backend job과 동적 decoration/status/view contribution은 Remote
  Location에서 항상 0이다. 정적 Git command definition은 `Local locations only.` 이유로
  disabled다. 이는 Git과 Remote 중 어느 트랙이 먼저 구현되어도 동일하다.
- Core Local mutation과 Git mutation은 같은 process-local mutation lease를 공유한다.
  Remote-only mutation은 Location별 serial lane이 직렬화한다. transfer가 local target을
  쓰거나 local source를 삭제할 때만 그 local 쓰기/삭제 구간에 공통 lease를 non-blocking으로
  추가 획득하며 active lease는 `Busy`로 거부한다.
- Remote Git, SSH terminal, Remote Edit, cross-Remote Copy/Move는 별도 제품 계약과 ADR 전에는
  구현하지 않는다.
