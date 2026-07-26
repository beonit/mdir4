# Mdir III 스타일 현대식 파일 관리자 요구사항

## 1. 프로젝트 개요

### 1.1 목적

DOS 시절 Mdir III의 핵심 사용 경험을 현대 Windows 환경에서 재구현한다.

단순히 Mdir III의 외형을 흉내 내는 것이 아니라 다음 특징을 핵심 UX로 유지한다.

* 키보드 중심 파일 관리
* 높은 정보 밀도
* 멀티 컬럼 파일 목록
* 상하좌우 방향키 탐색
* F1~F12 기능키 중심 조작
* 빠른 응답성
* Mdir 스타일 색상 테마
* 터미널 기반 UI
* 자동화 가능한 화면 및 키 입력 테스트

### 1.2 우선 지원 환경

1차 목표:

* Windows 10
* Windows 11
* Windows Terminal
* PowerShell / CMD 환경

향후:

* Linux
* macOS

# 2. 기술 스택

## 2.1 기본 기술

개발 언어:

```text
Rust
```

TUI 프레임워크:

```text
Ratatui
```

터미널 Backend:

```text
Crossterm
```

Snapshot 테스트:

```text
Insta
```

설정 파일:

```text
TOML 또는 JSON
```

## 2.2 기술 선택 이유

Ratatui의 셀 기반 렌더링 모델을 사용하여 Mdir III와 같은 고정밀 터미널 레이아웃을 구현한다.

실행 환경에서는:

```text
CrosstermBackend
```

를 사용하고,

자동 테스트에서는:

```text
TestBackend
```

를 사용한다.

따라서 실제 터미널을 실행하지 않고도 동일한 UI를 메모리에서 렌더링하여 테스트할 수 있어야 한다.

# 3. 프로그램 아키텍처

프로그램은 입력, 상태, 레이아웃, 렌더링을 분리한다.

```text
Keyboard
   │
   ▼
Input Mapper
   │
   ▼
Action
   │
   ▼
Application State
   │
   ▼
Layout Engine
   │
   ▼
Screen Rendering
   │
   ├──────────────► CrosstermBackend
   │                    │
   │                    ▼
   │               Real Terminal
   │
   └──────────────► TestBackend
                        │
                        ▼
                    Snapshot
```

중요 원칙:

**키 이벤트가 UI를 직접 변경하지 않는다.**

예:

```text
Right Arrow
    ↓
Action::MoveRight
    ↓
App::update()
    ↓
selectedIndex 변경
    ↓
render()
```

이 구조를 모든 기능에서 유지한다.

# 4. 메인 화면

기본 화면은 다음 영역으로 구성한다.

```text
┌──────────────────────────────────────────────────────────────┐
│ C:\WORK\PROJECT                                              │
├──────────────────────────────────────────────────────────────┤
│ README.TXT       TEST.EXE         ARCHIVE.ZIP                │
│ CONFIG.SYS       M.EXE            DATA.DAT                   │
│ AUTOEXEC.BAT     MSET.EXE         IMAGE.JPG                  │
│ COMMAND.COM      VV.EXE           NOTE.TXT                   │
│                                                              │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│ README.TXT   12,430 bytes   2026-07-24 13:20   RW           │
│ Files 24  Dirs 6   Selected 3 / 15.2 MB   Free 124.3 GB     │
├──────────────────────────────────────────────────────────────┤
│ F1 Help F2 Rename F3 View F4 Edit F5 Copy F6 Move ...       │
└──────────────────────────────────────────────────────────────┘
```

화면 구성:

1. 현재 경로 영역
2. 파일 목록 영역
3. 파일/디렉터리 상태 영역
4. F1~F12 기능키 영역

# 5. 현재 경로

화면 상단에 현재 디렉터리를 표시한다.

예:

```text
C:\Users\User\Downloads
```

지원 대상:

```text
C:\
D:\
Z:\

\\SERVER\Share\
```

긴 경로는 화면 폭에 맞게 축약할 수 있지만 내부적으로는 전체 경로를 유지한다.

# 6. 파일 목록

## 6.1 Short View

Mdir 스타일 기본 화면이다.

파일을 여러 세로 컬럼에 표시한다.

```text
README.TXT       TEST.EXE         ARCHIVE.ZIP
CONFIG.SYS       M.EXE            DATA.DAT
AUTOEXEC.BAT     MSET.EXE         IMAGE.JPG
COMMAND.COM      VV.EXE           NOTE.TXT
```

파일 배치는 세로 우선이다.

```text
1        5        9
2        6       10
3        7       11
4        8       12
```

즉:

```text
Top
 ↓
Bottom
 ↓
Next Column
```

# 7. 멀티 컬럼

멀티 컬럼은 본 프로젝트의 핵심 기능이다.

## 7.1 컬럼 모드

다음 설정을 제공한다.

```text
Auto
1
2
3
4
5
6
```

### Auto

화면 폭과 컬럼 최소 폭을 기준으로 자동 결정한다.

개념적으로:

```text
columnCount =
    usableWidth / minimumColumnWidth
```

최소/최대 범위를 적용한다.

```text
minColumns = 1
maxColumns = 6
```

### Fixed

사용자가 컬럼 수를 지정하면 가능한 범위에서 해당 값을 유지한다.

# 8. 컬럼 폭

설정:

```text
Auto
Compact
Normal
Wide
Custom
```

Custom에서는 문자 셀 기준으로 지정한다.

예:

```text
Column Width = 22
```

파일명이 표시 폭보다 길 경우 기본적으로 말줄임 처리한다.

예:

```text
very_long_filename_for_project.txt

↓

very_long_filename_f…
```

향후 다음 옵션을 제공할 수 있다.

```text
Truncate End
Truncate Middle
Horizontal Scroll
```

# 9. 방향키 탐색

방향키의 의미는 화면상의 공간적 위치와 일치해야 한다.

```text
↑    위 항목

↓    아래 항목

←    왼쪽 컬럼

→    오른쪽 컬럼
```

예:

```text
A1      B1      C1
A2      B2      C2
A3      B3      C3
A4      B4      C4
```

현재 위치가:

```text
B2
```

일 경우:

```text
UP      → B1
DOWN    → B3
LEFT    → A2
RIGHT   → C2
```

내부적으로 최소한 다음 정보를 관리한다.

```text
selectedIndex
row
column
rowsPerColumn
columnCount
```

기본 관계:

```text
index = column * rowsPerColumn + row
```

# 10. 마지막 컬럼 처리

마지막 컬럼에는 다른 컬럼보다 파일 수가 적을 수 있다.

예:

```text
A1      B1      C1
A2      B2      C2
A3      B3
A4      B4
```

이 경우 존재하지 않는 좌표로 이동할 때 기본 정책은:

```text
Nearest
```

이다.

예:

```text
B3 + RIGHT
→ C2
```

추후 옵션:

```text
Nearest
Strict
Wrap
```

### Nearest

목표 컬럼에서 가장 가까운 행으로 이동.

### Strict

해당 행에 항목이 없으면 이동하지 않음.

### Wrap

논리적 다음/이전 항목으로 이동.

# 11. 기본 탐색 키

```text
↑ ↓ ← →       파일 이동

Enter          디렉터리 진입 / 파일 실행

Backspace      상위 디렉터리

Home           첫 항목

End            마지막 항목

PgUp           이전 페이지

PgDn           다음 페이지

Space          선택 / 선택 해제

Insert         선택 후 다음 항목

Ctrl+A         전체 선택

Esc            현재 작업 취소
```

# 12. 파일 선택

다중 선택을 지원한다.

다음 상태를 구분한다.

```text
Normal
Cursor
Marked
Cursor + Marked
```

선택된 파일은 현재 커서와 별도로 시각적으로 구분되어야 한다.

하단 상태창에는:

```text
Selected 4 / 128.4 MB
```

형태로 표시한다.

# 13. Long View

상세 파일 정보를 보여주는 모드이다.

```text
Name              Size        Date        Time     Attr
-------------------------------------------------------
README.TXT        12,430      2026-07-24  13:20    RW
TEST.EXE         824,320      2026-07-24  12:40    RW
DATA.ZIP       3,142,123      2026-07-23  21:12    RW
```

Short View와 Long View 사이를 키보드로 빠르게 전환할 수 있어야 한다.

# 14. F1~F12 기능키

화면 최하단에 기능키를 항상 표시한다.

초기 기능 정의:

```text
F1    Help
F2    Rename
F3    View
F4    Edit
F5    Copy
F6    Move
F7    MkDir
F8    Delete
F9    Reserved / 추가 확인
F10   MCD
F11   QCD
F12   Menu
```

원본 Mdir III 동작을 추가 조사하면서 최종 키 배치는 조정할 수 있다.

F1~F12 키의 표시와 실제 Key Mapping은 동일한 설정 소스를 사용해야 한다.

# 15. F1 Help

현재 사용 가능한 키를 표시한다.

```text
Navigation

↑ ↓          Move Up / Down
← →          Move Column
Enter        Open
Backspace    Parent
PgUp/PgDn    Page
Home/End     First / Last

File

F2           Rename
F3           View
F4           Edit
F5           Copy
F6           Move
F7           Make Directory
F8           Delete
```

# 16. F2 Rename

현재 파일/디렉터리의 이름을 변경한다.

```text
Rename

Old : README.TXT
New : README_OLD.TXT

Enter : Rename
Esc   : Cancel
```

요구사항:

* 기존 파일명 자동 입력
* 동일 이름 검사
* OS에서 허용하지 않는 파일명 검사
* Unicode 지원

# 17. F3 View

읽기 전용 파일 뷰어.

MVP 지원:

* TXT
* LOG
* JSON
* XML
* CSV
* Markdown
* 기타 일반 텍스트

키:

```text
↑ ↓
PgUp PgDn
Home End
Ctrl+F
Esc
```

향후:

```text
Hex Viewer
Encoding Selection
```

# 18. F4 Edit

간단한 내장 텍스트 편집기.

MVP:

* 입력
* 삭제
* 저장
* 다른 이름으로 저장
* Undo / Redo
* 검색
* 줄 번호

전문 코드 편집기는 범위에서 제외한다.

# 19. F5 Copy

현재 파일 또는 선택된 파일을 복사한다.

```text
Copy

3 Files / 15.2 MB

From:
C:\WORK

To:
D:\BACKUP

Enter : Copy
Esc   : Cancel
```

충돌 처리:

```text
Overwrite
Overwrite All
Skip
Skip All
Rename
Cancel
```

# 20. F6 Move

Copy와 동일한 인터페이스를 사용한다.

대상 파일을 이동한다.

# 21. F7 MkDir

새 디렉터리를 생성한다.

```text
Create Directory

Name:
NEW_DIRECTORY

Enter : Create
Esc   : Cancel
```

# 22. F8 Delete

파일 또는 디렉터리를 삭제한다.

기본적으로 확인창을 표시한다.

```text
Delete selected files?

3 Files
15.2 MB

Enter : Delete
Esc   : Cancel
```

Windows에서는 기본적으로 휴지통 이동을 우선한다.

별도 명령 또는 설정을 통해 영구 삭제를 지원할 수 있다.

# 23. F10 MCD

Directory Change 기능을 제공한다.

목적:

* 디렉터리 빠른 탐색
* 디렉터리 검색
* 이전 방문 경로 접근

구체적인 원본 동작은 별도 조사 후 상세화한다.

# 24. F11 QCD

자주 사용하는 디렉터리에 빠르게 접근한다.

예:

```text
1  C:\WORK
2  C:\SOURCE
3  D:\DOWNLOAD
4  D:\BACKUP
5  E:\ARCHIVE
```

사용자가 QCD 항목을 추가/수정/삭제할 수 있어야 한다.

# 25. F12 Menu

전체 기능 메뉴를 제공한다.

```text
File
  Open
  Rename
  Copy
  Move
  Delete
  Properties

View
  Short
  Long
  Column
  Sort
  Hidden Files

Directory
  MCD
  QCD
  Drive

Tools
  Search
  Terminal

Options
  Theme
  Keyboard
  Layout
  Settings

Quit
```

# 26. 하단 상태창

파일 목록 아래에 현재 상태를 표시한다.

## 현재 파일 정보

```text
README.TXT   12,430 bytes   2026-07-24 13:20   RW
```

표시 대상:

* 전체 파일명
* 크기
* 날짜
* 시간
* 속성

## 현재 디렉터리 정보

```text
Files 24   Dirs 6   Selected 3 / 15.2 MB   Free 124.3 GB
```

표시 대상:

* 파일 수
* 디렉터리 수
* 선택 수
* 선택 파일 총 크기
* 드라이브 여유 공간

# 27. 파일 작업 상태

복사/이동 등 장시간 작업에서는 상태 영역에 진행률을 표시한다.

```text
Copying README.ISO

38 / 150 Files
324 MB / 1.4 GB
38%
125 MB/s
```

가능하면 UI 자체는 파일 작업 중에도 응답 가능하게 유지한다.

# 28. 기능키 상태바

항상 화면 최하단에 위치한다.

예:

```text
F1 Help  F2 Rename  F3 View  F4 Edit
F5 Copy  F6 Move    F7 MkDir F8 Delete
F9 ---   F10 MCD    F11 QCD  F12 Menu
```

화면 폭이 부족하면 축약 표시한다.

```text
1Help 2Ren 3View 4Edit 5Copy 6Move 7Dir 8Del 10MCD 11QCD 12Menu
```

# 29. 테마 시스템

테마는 UI 로직과 완전히 분리한다.

기본 제공:

```text
Classic Mdir
DOS Blue
Dark
Mono
Light
```

기본값:

```text
Classic Mdir
```

# 30. Classic Mdir 테마

주요 역할별 색상을 정의한다.

```text
Background
Normal File
Directory
Executable
Archive
Cursor
Marked File
Cursor + Marked
Border
Status Bar
Function Key
Warning
Error
Dialog
```

# 31. 파일 형식별 색상

확장자 그룹별 색상을 설정할 수 있다.

예:

```text
Directory
→ Cyan

.exe .com .bat .cmd
→ Green

.zip .rar .7z .arj
→ Magenta

.txt .md .ini .cfg
→ Light Gray

.jpg .png .gif .bmp
→ Yellow
```

색상은 테마에서 변경 가능해야 한다.

# 32. 사용자 테마

외부 설정 파일을 통해 테마를 추가할 수 있어야 한다.

예:

```toml
name = "Classic Mdir"

background = "dark_blue"
foreground = "gray"

directory = "cyan"
executable = "green"
archive = "magenta"

selected_background = "cyan"
selected_foreground = "white"

marked = "yellow"

warning = "yellow"
error = "red"
```

프로그램을 다시 빌드하지 않고 테마 변경이 가능해야 한다.

# 33. 정렬

지원:

```text
Name
Extension
Size
Date
Time
```

각 방식:

```text
Ascending
Descending
```

옵션:

```text
Directories First
```

# 34. 창 크기 변경

터미널 크기가 변경되면 즉시 레이아웃을 다시 계산한다.

예:

```text
80x25
→ 3 Columns

120x30
→ 4 Columns

160x40
→ 5 Columns
```

단, 실제 컬럼 수는 컬럼 폭과 UI 영역에 따라 계산한다.

리사이즈 전후에도 동일한 파일이 선택된 상태를 유지해야 한다.

# 35. 설정 저장

프로그램 종료 후 다음 상태를 유지한다.

```text
마지막 디렉터리
테마
Short / Long View
Column Mode
Column Count
Column Width
정렬 방식
숨김 파일 표시 여부
QCD 항목
창 관련 설정
사용자 단축키
```

# 36. 파일 시스템 추상화

실제 파일 시스템 접근과 UI를 분리한다.

인터페이스 개념:

```text
FileSystem
 ├ RealFileSystem
 └ TestFileSystem
```

실제 실행:

```text
RealFileSystem
```

테스트:

```text
TestFileSystem
```

을 사용한다.

이를 통해 실제 PC의 파일 상태와 무관하게 동일한 테스트 결과를 얻을 수 있어야 한다.

# 37. 자동 테스트 요구사항

자동 테스트는 필수 개발 범위로 한다.

테스트는 크게 세 단계로 구분한다.

## 37.1 Unit Test

UI와 무관한 내부 로직을 검사한다.

예:

```text
MoveRight
MoveLeft
MoveUp
MoveDown

Column Calculation

Sorting

Selection

Path Handling
```

# 38. Navigation Test

멀티 컬럼 탐색을 집중적으로 테스트한다.

예:

```text
A1      B1      C1
A2      B2      C2
A3      B3
A4      B4
```

테스트:

```text
A2 + RIGHT → B2

B2 + RIGHT → C2

B3 + RIGHT → C2

C2 + LEFT → B2

A1 + LEFT → A1

A1 + UP → A1
```

경계 동작을 반드시 테스트한다.

# 39. 입력 시나리오 테스트

키 입력 시나리오는 파일로 저장할 수 있어야 한다.

예:

```yaml
name: multi-column-navigation

terminal:
  width: 80
  height: 25

filesystem:
  fixture: many-files

steps:
  - key: DOWN
  - key: DOWN

  - key: RIGHT
  - snapshot: after-right

  - key: RIGHT
  - snapshot: after-second-right

  - key: LEFT
  - snapshot: after-left
```

테스트 러너는 이 파일을 읽어 동일한 키 입력을 순서대로 재생한다.

# 40. 가상 터미널 테스트

자동 테스트에서는 실제 터미널 대신 Ratatui `TestBackend`를 사용한다.

예:

```text
TestBackend
80 x 25

↓

Render

↓

Screen Buffer
```

화면 버퍼를 Snapshot과 비교한다.

# 41. Snapshot 테스트

기준 화면을 Golden Snapshot으로 관리한다.

예:

```text
tests/
  snapshots/
    startup_80x25.snap
    short_3column.snap
    navigation_right.snap
    copy_dialog.snap
    resize_120x30.snap
```

화면 변화 발생 시 기존 Snapshot과 비교한다.

의도하지 않은 변경은 테스트 실패로 처리한다.

# 42. 테스트 터미널 크기

최소 다음 화면 크기를 자동 테스트한다.

```text
80 x 25
100 x 30
120 x 40
160 x 50
```

특히 80x25는 DOS 스타일 레이아웃의 기준 테스트로 취급한다.

# 43. 레이아웃 경계 테스트

다음 파일 개수를 반드시 테스트한다.

```text
0 Files
1 File

1 Column Capacity - 1
1 Column Capacity
1 Column Capacity + 1

1 Page Capacity - 1
1 Page Capacity
1 Page Capacity + 1

100 Files
1,000 Files
10,000 Files
```

목적은 Column/Page 계산에서 발생하는 off-by-one 오류를 방지하는 것이다.

# 44. 자동 컬럼 테스트

Auto Column 모드에서는 터미널 크기에 따른 컬럼 계산 결과를 검사한다.

예:

```text
Width 79
Width 80
Width 81

Width 99
Width 100
Width 101
```

처럼 경계값을 집중적으로 검사한다.

# 45. Resize 테스트

예:

```text
80x25
→ 3 Columns

Resize

120x30
→ 4 Columns
```

검증:

* 선택 파일 유지
* Scroll 위치 정상
* Column 계산 정상
* 상태바 정상
* F키 바 정상
* 화면 밖 출력 없음

# 46. 테마 테스트

Snapshot에는 문자뿐 아니라 셀 스타일도 검사할 수 있어야 한다.

검사 대상:

```text
Foreground
Background
Bold
Selected
Marked
```

Classic Mdir 테마에서:

```text
Directory
Executable
Archive
Selected File
Marked File
```

색상이 올바르게 적용되는지 테스트한다.

# 47. 동적 데이터 테스트

Snapshot 결과가 실행 시점에 따라 변하지 않도록 외부 값을 주입 가능하게 한다.

예:

```text
Clock
Disk Free Space
Transfer Speed
Current Path
File Modification Time
```

테스트 환경에서는:

```text
Clock = 2026-01-01 12:00
Free Space = 100 GB
Transfer Speed = 10 MB/s
```

처럼 고정값을 사용한다.

# 48. 테스트 Fixture

실제 사용자 디렉터리를 테스트 대상으로 사용하지 않는다.

예:

```text
fixtures/
  empty/

  basic/
    README.TXT
    TEST.EXE
    CONFIG.INI

  many-files/
    FILE001.TXT
    FILE002.TXT
    ...
    FILE200.TXT

  long-names/

  unicode/
    한글파일.txt
    日本語.txt

  nested/

  mixed-types/
```

# 49. Snapshot 실패 결과

화면 Snapshot이 달라진 경우 사람이 쉽게 차이를 확인할 수 있어야 한다.

예:

```text
EXPECTED                 ACTUAL

A1   B1   C1             A1   B1   C1
A2  [B2]  C2             A2   B2  [C2]
A3   B3                  A3   B3
```

Snapshot 변경은 개발자가 검토 후 승인하는 방식으로 관리한다.

# 50. CI 테스트

GitHub Actions 등 CI 환경에서도 다음 테스트가 실행되어야 한다.

```text
cargo test
snapshot validation
scenario tests
layout tests
```

실제 터미널 환경이 없는 CI에서도 모든 UI 테스트가 실행 가능해야 한다.

# 51. 테스트 가능성을 위한 핵심 설계 원칙

다음 기능을 직접 OS API에 결합하지 않는다.

```text
Keyboard
Filesystem
Clock
Disk Information
Terminal Size
Clipboard
Process Execution
```

각 기능은 교체 가능한 인터페이스 또는 wrapper를 통해 접근한다.

이를 통해:

```text
Production Implementation
Test Implementation
```

을 분리한다.

# 52. MVP 1 — 화면 및 탐색

우선 구현:

* Ratatui 기본 프로그램
* Crossterm
* 메인 화면
* 현재 경로
* 파일 목록
* Short View
* Auto Column
* Fixed Column
* 상/하 이동
* 좌/우 컬럼 이동
* Enter
* Backspace
* Home
* End
* PgUp
* PgDn
* 선택
* 하단 상태창
* F1~F12 바
* Classic Mdir 테마

동시에 구현:

* TestBackend
* 기본 Navigation Unit Test
* Layout Snapshot Test

# 53. MVP 2 — 파일 관리

추가:

* Rename
* View
* Copy
* Move
* MkDir
* Delete
* Sort
* Drive 이동

테스트:

* Dialog Snapshot
* 파일 작업 Mock 테스트
* 에러 처리 테스트

# 54. MVP 3 — Mdir 기능 확장

추가:

* MCD
* QCD
* F12 Menu
* Long View
* 컬럼 설정
* 테마 선택
* 설정 저장
* 사용자 단축키

# 55. 향후 기능

후순위:

```text
파일 검색
내장 Editor 강화
Hex Viewer
압축 파일 탐색
압축 / 해제
내장 Terminal
SFTP
SSH
네트워크 탐색
Git 상태 표시
2 Panel Mode
Plugin System
```

# 56. UX 원칙

가장 중요한 UX 원칙:

**마우스 없이 모든 핵심 기능을 사용할 수 있어야 한다.**

주요 작업은 가능한 한:

```text
방향키
→ 파일 선택
→ Function Key
→ Enter
```

형태로 완료되어야 한다.

팝업에서는 일관되게:

```text
Enter = 실행 / 확인
Esc   = 취소 / 닫기
```

규칙을 사용한다.

# 57. 프로젝트 핵심 정체성

본 프로젝트의 핵심은 DOS처럼 보이는 UI 자체가 아니다.

핵심 요소는 다음 네 가지다.

1. 멀티 컬럼 파일 탐색
2. 상하좌우 방향키 기반 공간 탐색
3. F1~F12 중심의 즉시 실행
4. 높은 정보 밀도와 빠른 키보드 조작

새로운 기능을 추가하더라도 이 네 가지 특성을 훼손하지 않는다.

# 58. 개발 원칙

기능 구현 시 다음 우선순위를 따른다.

```text
정확한 키보드 동작
        ↓
자동 테스트 가능성
        ↓
레이아웃 일관성
        ↓
성능
        ↓
부가 기능
```

특히 UI 구현과 동시에 해당 기능의 자동 테스트를 작성한다.

기능이 구현되었으나 자동 테스트가 불가능하다면 구조적으로 UI와 상태 로직이 과도하게 결합되어 있지 않은지 먼저 검토한다.

# 59. 완료 기준

MVP 완료의 최소 기준은 다음과 같다.

* Windows에서 단일 실행 파일로 구동
* 80x25 터미널에서 정상 표시
* Short View 멀티 컬럼 정상 동작
* 좌우 방향키 컬럼 이동 정상
* 상하 방향키 행 이동 정상
* 창 크기 변경 시 레이아웃 정상 재계산
* 파일 선택 가능
* 기본 파일 관리 가능
* F1~F12 상태바 제공
* Classic Mdir 테마 제공
* 설정 저장
* 모든 Navigation 테스트 통과
* 주요 화면 Snapshot 테스트 통과
* 키 입력 시나리오 자동 재생 가능
* CI 환경에서 실제 터미널 없이 UI 테스트 가능
