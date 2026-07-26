# 제품 계약

이 문서는 원본 요구사항의 충돌과 모호성을 구현 가능한 기본 결정으로 고정한다.
사용자가 나중에 다른 결정을 내리면 이 문서와 관련 테스트를 함께 수정한다.

## 1. v1 범위 결정

### 포함

- Windows 10/11에서 Windows Terminal의 PowerShell profile, standalone PowerShell console
  host, standalone CMD console host로 실행
- Short/Long View
- 세로 우선 멀티 컬럼
- 공간적 상·하·좌·우 탐색과 페이지 탐색
- 다중 선택
- Rename, View, 기본 Edit, Copy, Move, MkDir, Delete
- MCD, QCD, F12 메뉴
- Classic Mdir 기본 테마와 외부 TOML 테마
- TOML 설정 저장과 사용자 키맵
- TestBackend, 단위/시나리오/스냅샷 테스트
- Windows 단일 `.exe` 배포

### 제외

- 마우스 입력
- Hex/바이너리 뷰어
- 비 UTF-8 인코딩 선택
- 정규식 검색, 코드 편집 기능, 구문 강조
- 압축 파일 내부 탐색, 네트워크 프로토콜, 2패널, Git 통합,
  범용/외부 플러그인
- 심볼릭 링크를 따라가는 재귀 복사/삭제
- 원본 Mdir 키 배치의 완전한 복제

### UI 언어

- v1의 built-in label, Help, command, dialog, progress와 error 문구는 영어로 작성한다.
- 파일명, 경로, 사용자 입력과 설정의 표시명은 번역하거나 ASCII로 바꾸지 않고 원문을
  Unicode/cell-width 규칙대로 표시한다.
- 다국어 UI/localization framework는 v1 범위가 아니다. 문서 언어가 한국어인 것은 UI 문구
  계약과 무관하다.

## 2. 기본 키 배치

현대식 키 배치를 v1 기본값으로 한다. 표시 문자열과 실제 입력 매핑은 하나의
`CommandRegistry`에서 생성한다.

| 키 | 동작 | 최초 단계 |
|---|---|---|
| F1 | Help | M1 |
| F2 | Rename | M2 |
| F3 | View | M2 |
| F4 | Edit | M2 |
| F5 | Copy | M2 |
| F6 | Move | M2 |
| F7 | MkDir | M2 |
| F8 | Delete | M2 |
| F9 | Reserved, 화면에는 `F9 ---` | M1 |
| F10 | MCD | M3 |
| F11 | QCD | M3 |
| F12 | Menu | M3 |
| Tab | Short/Long View 전환 | M3 |
| R | 현재 디렉터리 Refresh | M1 |
| Ctrl+Q | 종료 확인 | M1 |
| S | 정렬 key 순환: Name→Extension→Size→Date→Time | M2 |
| Ctrl+S | 정렬 Ascending/Descending 전환(Main context) | M2 |
| H | hidden 항목 표시 전환 | M2 |
| D | Local drive picker | M2 |

아직 구현되지 않은 기능도 기능키 바에 표시하되 비활성 스타일을 사용한다. 80열
축약 모드에서도 `1`부터 `12`까지 모두 표시하며 F9를 누락하지 않는다.

원본 Mdir 키맵은 정확한 조사 자료가 확보되기 전까지 v1 범위에서 제외한다.

## 3. 80×25 기준 레이아웃

기본 화면은 테두리 행을 별도로 소비하지 않고 정확히 25행을 사용한다.

| 행 | 높이 | 내용 |
|---|---:|---|
| 0 | 1 | 현재 경로 |
| 1~20 | 20 | 파일 목록 |
| 21 | 1 | 현재 항목 상세 |
| 22 | 1 | 디렉터리/선택/여유 공간 요약 |
| 23 | 1 | 메시지 또는 작업 진행률 |
| 24 | 1 | F1~F12 기능키 바 |

- 80×25 이상은 정상 레이아웃을 보장한다.
- 60×15 이상 80×25 미만은 축약 레이아웃으로 동작한다.
- 60×15 미만은 패닉 없이 `Terminal too small (minimum 60x15)`만 표시한다.
- 상단 원본 프로그램 메뉴는 표시하지 않는다. 전체 메뉴는 F12로 통합한다.
- 모달은 파일 목록 위에 중앙 정렬하며 경로, 상태, 기능키 바를 다시 배치하지 않는다.

## 4. 컬럼 계산

`ColumnCountMode`와 `ColumnWidthMode`를 분리한다.

```rust
enum ColumnCountMode { Auto, Fixed(u8) } // 1..=6
enum ColumnWidthMode { Compact, Normal, Wide, Custom(u16) }
```

목표 폭은 Compact 24셀, Normal 32셀, Wide 40셀, Custom 12~80셀이다. 기본값은
`Auto + Normal`이다.

Auto는 먼저 터미널 폭으로 최대 컬럼 수를 계산한다.

```text
rounded          = (usable_width + target_width / 2) / target_width
width_based_max  = clamp(rounded, 1, 6)
required         = ceil(max(entry_count, 1) / rows_per_column)
column_count     = min(width_based_max, max(required, 1))
```

폭 기준 최대치는 Normal에서 80→3, 120→4, 160→5다. 항목이 한 컬럼의 행 수 이하면
유효 컬럼은 1개이고 목록 폭 전체를 사용한다. 항목이 늘면 필요한 만큼 2, 3…개로
증가하되 폭 기준 최대치를 넘지 않는다. 최종 컬럼 폭이 12셀 미만이면 12셀 이상이 될
때까지 최대 컬럼 수를 줄인다.

- 유효 컬럼은 목록 폭을 빈틈없이 균등 분배한다.
- 마지막을 제외한 컬럼은 오른쪽 1셀을 cyan box-drawing `│` 경계로 사용한다.
- 경계 1셀은 내용 폭에서 제외해 이름/메타데이터와 겹치지 않는다.
- Fixed 모드는 항목 수에 따라 요청 컬럼 수를 자동 축소하지 않는다.

Fixed 계산:

1. 요청한 1~6개 컬럼을 먼저 사용한다.
2. 실제 폭이 컬럼당 12셀 미만이면 12셀 이상이 될 때까지 컬럼 수를 줄인다.
3. 가로 스크롤은 v1에서 만들지 않는다.
4. 나머지 셀은 왼쪽 컬럼부터 1셀씩 분배한다.

## 5. Short/Long View 표시

Short View는 실제 컬럼 폭에 따라 메타데이터를 단계적으로 줄인다.

| 컬럼 폭 | 내용 |
|---:|---|
| 12~27 | 이름 |
| 28~39 | 이름 + 우측 정렬 크기 또는 `<DIR>` |
| 40 이상 | 이름 + 크기/`<DIR>` + `MM-DD HH:mm` |

- 확장자는 이름에 포함한다.
- 표시 폭 계산은 바이트/문자 개수가 아니라 grapheme과 터미널 셀 폭을 사용한다.
- 긴 이름은 끝 말줄임표 `…`로 자른다.
- modified time은 파일 시스템의 raw timestamp를 **그 timestamp 시점의 OS local timezone**으로
  변환한다. worker가 주입된 time-zone port로 변환한 local minute를 state에 넣고 render는
  OS/clock/timezone API를 호출하지 않는다. test/snapshot은 fixed UTC offset을 주입한다.
- modified time이 없거나 개별 entry metadata 조회/시간 변환이 실패하면 목록 전체를
  실패시키지 않고 정확히 `----- --:--`를 표시한다. 합성 `..`도 이 fallback을 사용한다.
- 공통 entry attribute는 `read_only`, `hidden`, `system`, `archive` 네 bool이다. Windows는
  대응 file attribute를, Unix는 permission의 read-only와 basename 선행 `.`의 hidden을
  채우며 지원하지 않는 system/archive는 false다. Long View `Attr`은 `R/H/S/A` 순서로
  표시하고 false는 `-`다.
- 현재 항목의 전체 이름과 상세 정보는 행 21에 항상 표시한다.
- Long View는 단일 컬럼 표 `Name / Size / Date / Time / Attr`을 사용한다.

## 6. 목록과 탐색의 정확한 규칙

- 목록은 `..`(루트가 아닐 때), 디렉터리, 파일 순서다.
- `..`는 합계에 포함하지 않고 마킹할 수 없는 합성 항목이다.
- 정렬 옵션은 같은 그룹 안에서 적용한다.
- 기본 정렬은 Name Ascending, Directories First, 대소문자 비구분이다.
- 이름이 같은 경우 원본 이름, 전체 경로 순으로 안정적으로 정렬한다.
- 한 페이지는 `rows_per_column × column_count` 항목이다.
- 페이지 안에서는 세로 우선으로 채운다.

```text
page_start = (selected_index / page_capacity) * page_capacity
local      = selected_index - page_start
column     = local / rows_per_column
row        = local % rows_per_column
```

키 동작:

- Up/Down: 같은 컬럼의 위/아래 항목. 마지막 컬럼의 마지막 표시 항목에서 Down은 다음
  페이지 첫 항목으로, 페이지 첫 항목에서 Up은 이전 페이지 마지막 항목으로 이동한다.
- Left/Right: 같은 행의 이웃 컬럼. 대상 행이 없으면 대상 컬럼의 마지막 항목으로
  이동한다. 첫 컬럼의 Left와 마지막 컬럼의 Right는 이전/다음 페이지의 반대쪽 컬럼
  같은 행으로 연결하며, 짧은 페이지에서는 가장 가까운 유효 행을 선택한다.
- 전체 목록의 첫 경계와 마지막 경계에서는 더 이동하지 않는다.
- Home/End: 전체 목록의 처음/마지막.
- PgUp/PgDn: 한 페이지 용량만큼 이동하고, 없는 위치면 처음/마지막으로 제한한다.
- 리사이즈: 선택된 파일 경로는 유지하고 새 레이아웃에서 페이지를 재계산한다.
- 정렬 변경: 선택된 파일 경로를 다시 찾아 커서를 유지한다.

정렬/숨김/drive의 정확한 Main-context 계약:

- 기본은 `SortKey::Name`, `Ascending`, `DirectoriesFirst=true`, `show_hidden=true`다.
- 합성 `..`는 정렬/필터 밖에서 항상 첫 행이다. Directories First가 켜지면 directory/file
  group을 먼저 고정하고 선택한 key는 각 group 안에서만 적용한다.
- Name은 Unicode lowercase key, 원본 이름, normalized full path 순이다.
- Extension은 마지막 `.` 뒤 suffix를 사용하되 leading-dot-only 이름(`.gitignore`)과 trailing
  dot(`name.`)은 extension 없음으로 본다. suffix 비교 뒤 Name ascending tie-break를 쓴다.
- Size는 알려진 일반 파일 byte 수를 비교한다. directory/unknown size는 방향과 무관하게
  항상 마지막이며 그 안에서는 Name ascending이다.
- Date는 주입된 local minute의 `year-month-day` 뒤 `hour-minute`, Time은 `hour-minute` 뒤
  `year-month-day`를 비교한다. unavailable/변환 실패는 방향과 무관하게 항상 마지막이다.
  같은 local minute는 raw timestamp, Name, path 순으로 안정화한다.
- Descending은 선택 key의 유효 값 순서만 뒤집고 missing-last와 Name/path tie-break는 뒤집지
  않는다.
- H가 off로 바꾸면 `hidden=true` entry만 제외하고 `..`는 유지한다. filter/sort/drive 전환 뒤
  같은 EntryId를 찾고, 없으면 이전 visual index에 가장 가까운 항목으로 이동한다.
- D는 Windows logical drive root 목록을 비동기로 열고 Up/Down/Enter/Esc로 선택한다. 오류나
  빈 목록은 현재 directory를 바꾸지 않는다. S/Ctrl+S/H/D label과 Help는 CommandRegistry 한
  정의에서 생성한다.

## 7. 선택 규칙

- 커서와 마킹은 독립 상태다.
- Space는 현재 항목 마킹만 토글하고 커서를 움직이지 않는다.
- Insert는 토글 후 Down과 같은 규칙으로 한 칸 이동한다.
- Ctrl+A는 현재 디렉터리의 마킹 가능한 항목만 선택한다.
- 다른 디렉터리로 이동하면 마킹을 비운다.
- 같은 디렉터리 새로고침에서는 아직 존재하는 경로의 마킹만 유지한다.
- 파일 작업은 마킹이 비어 있으면 커서 항목, 아니면 마킹 항목 전체를 대상으로 한다.
- 선택 바이트 합계는 현재 알려진 일반 파일 크기만 더한다. 디렉터리 내부 크기를
  재귀 계산하지 않으며 디렉터리는 선택 개수에는 포함한다.

## 8. 파일 실행과 안전 정책

- Enter on directory: 해당 디렉터리 로드.
- Enter on regular file: Windows ShellExecute 계열 어댑터로 기본 연결 프로그램 실행.
- 실행 전에 셸 문자열을 조립하지 않는다.
- Delete 기본 동작은 휴지통 이동이다.
- 영구 삭제는 `Shift+F8`로만 시작하며 별도 경고 문구와 확인이 필요하다.
- Copy/Move/Delete는 경로 정규화 후 동일 원본/대상, 대상이 원본 하위인 경우를 거부한다.
- Copy/Move 대상 충돌 선택은 정확히 `Overwrite`, `Overwrite All`, `Skip`, `Skip All`,
  `Rename`, `Cancel` 여섯 가지다. `All`은 현재 OperationId의 이후 충돌에만 적용되고 다음
  작업으로 넘어가지 않는다. `Rename`은 충돌 없는 제안명을 다시 검증한다.
- 심볼릭 링크는 표시하지만 v1 재귀 작업에서 따라가지 않는다. 지원하지 않는 작업은 명시적 오류다.
- 부분 실패 결과에는 성공/실패/건너뜀 개수와 첫 오류를 표시한다.
- Copy는 내용과 디렉터리 구조를 필수 보존한다. 수정 시각과 읽기 전용 속성은 가능한
  플랫폼에서 best-effort로 보존하고, 실패하면 결과 경고에 포함한다.

## 9. 기본 편집기 범위

F4 Edit는 M2에 포함한다.

- UTF-8 및 UTF-8 BOM 텍스트만 편집
- 최대 5 MiB
- 입력, 삭제, 줄바꿈, 저장, 다른 이름으로 저장
- 전체 버퍼 스냅샷 방식 Undo/Redo 각 100단계
- 문자열 검색, 줄 번호
- grapheme 단위 커서 이동
- 외부 변경 감지는 저장 직전 수정 시각 비교 후 확인

전문 편집 기능과 인코딩 선택은 후순위다.

## v1 이후 Git/SSH Remote 확장과의 관계

- 위 제외 항목은 `v1.0` 계약을 뜻한다. Git 통합은 v1 완료 후 별도 `G0~G3`
  트랙에서 진행한다.
- 첫 Git 확장은 같은 실행 파일에 정적으로 포함되는 Rust trait 기반 built-in이다.
  외부 DLL/WASM/프로세스 플러그인을 로드하는 공개 SDK나 ABI는 만들지 않는다.
- Git 기능을 끄거나 Git 저장소 밖에 있을 때 기존 파일 탐색 동작, 키 배치, 렌더
  경로에는 Git I/O가 없어야 한다.
- Git 전용 제품 계약은 [`../plugins/git/01-product-contract.md`](../plugins/git/01-product-contract.md)가
  관리하며 v1 수용 기준과 릴리스 게이트를 변경하지 않는다.
- SSH/SFTP는 v1 제외를 유지하되 R1 이후 `S0~S3` Remote Drive 트랙으로 계획한다.
  인증 정보는 저장하지 않고 OpenSSH Host alias/config/agent/known_hosts에 위임한다.
- Remote는 generic plugin이 아니라 `LocationId + LocationPath` 기반 Core Location 유형이다.
  Remote 전용 계약은 [`../remote/01-product-contract.md`](../remote/01-product-contract.md)가
  관리하며 v1/Git 완료 기준을 변경하지 않는다.

## 10. MCD/QCD 결정

MCD:

- F10에서 전체 화면 트리를 연다.
- 현재 경로의 조상은 자동 확장한다.
- 자식 디렉터리는 Right/Enter 시 지연 로드한다.
- Up/Down 이동, Left 접기/부모, Right 펼치기/첫 자식, Enter 경로 확정, Esc 취소.
- 입력 문자는 로드된 노드와 최근 방문 경로를 부분 문자열로 필터한다.
- MCD 화면 안에서 F2는 현재 노드 재검색, F3는 드라이브 선택이다. Main의 F3 View와
  다른 screen-specific command context다.
- 전체 디스크 사전 색인은 v1 범위에서 제외한다.

QCD:

- F11에서 저장된 경로 목록을 연다.
- Enter 이동, Insert 현재 경로 추가, F2 이름/경로 수정, Delete 제거, Esc 닫기.
- 숫자 1~9는 보이는 첫 9개 항목으로 즉시 이동한다.

## 11. 설정과 테마

- 사용자 설정은 TOML 하나만 지원한다.
- Windows 기본 위치는 플랫폼 config directory 아래 `Mdir4/config.toml`이다.
- 저장은 임시 파일 작성 후 원자적 교체를 사용한다.
- 손상된 설정은 `.broken-<timestamp>`로 보존하고 기본값으로 시작하면서 경고한다.
- `column_mode = "auto"`에서는 저장된 fixed count를 무시한다.
- 테마는 `themes/*.toml`, 키맵은 `keymaps/*.toml`에서 추가 로드한다.
- `Classic Mdir`는 화면별 배경 토큰을 가진다. 메인은 검정, MCD는 파랑,
  대화상자는 자홍 계열을 기본으로 한다.

## 12. 성능 목표

- 키 입력 후 다음 프레임: 일반 디렉터리에서 50 ms 이내
- 10,000개 항목 정렬/레이아웃/렌더: 개발 기준 머신에서 100 ms 이내
- 렌더는 파일 시스템 또는 OS API를 호출하지 않는다.
- 100 ms 이상 걸릴 수 있는 파일 작업과 디렉터리 탐색은 작업 스레드에서 실행한다.
- 이벤트 루프는 작업 중에도 최소 20 FPS 수준으로 입력과 진행률을 처리한다.
