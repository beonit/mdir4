# 수용 기준 추적표

이 파일은 정적 요구사항→카드→검증 추적표다. 실제 상태, test 이름, 실행 날짜와 수동
증거는 `progress.md`의 `Acceptance evidence ledger`에 기록한다. 표에 검증 방법이 적혀
있어도 실제 evidence가 없으면 완료가 아니다.

| ID | 수용 기준 | 단계/카드 | 자동 검증 | 수동 검증 |
|---|---|---|---|---|
| ENV-01 | Linux/macOS 단일 실행 파일 실행 | R1-01~03 | 플랫폼별 release build | 두 OS 실행 |
| ENV-02 | Linux 일반 터미널과 macOS Terminal/iTerm 계열 정상 | R1-02 | 해당 없음 | 플랫폼별 terminal checklist |
| ARC-01 | 키 이벤트가 UI를 직접 변경하지 않음 | M1-01,09,11 | reducer/input dependency test | 없음 |
| ARC-02 | 렌더 중 OS/FS 호출 없음 | M1-10,13 | recording ports call count | 없음 |
| ARC-03 | 실제/Test FS 교체 가능 | M1-04,05 | 동일 contract tests | 없음 |
| ARC-04 | Core Local queue가 bounded/non-blocking이고 full coalesce/Busy가 결정적 | M2-09 | injected-capacity backpressure | 없음 |
| UI-01 | 80×25 정상 표시 | M1-03,10 | golden snapshot | Linux/macOS 확인 |
| UI-02 | 60×15 미만 안전 안내 | M1-03,10 | boundary snapshot | 없음 |
| UI-03 | 현재 경로 전체 내부 유지/화면 축약 | M1-02,10 | Unicode/long path snapshot | 없음 |
| UI-04 | Short View 1~6 컬럼 | M1-03,10 | mode matrix | 없음 |
| UI-05 | Auto 폭 최대 80→3, 120→4, 160→5; 항목 수에 따라 1열부터 적응 | M1-03 | exact/adaptive unit tests | 없음 |
| UI-06 | Long View 전환과 선택 유지 | M3-03 | scenario+snapshot | 없음 |
| UI-07 | 40셀 이상 Short View가 OS-local `MM-DD HH:mm`, unavailable fallback과 R/H/S/A를 결정적으로 표시 | M1-13 | fixed-timezone metadata snapshot | 없음 |
| UI-08 | built-in UI 문구는 영어이고 파일명/경로/사용자 입력은 원문을 보존 | M1-13,M2-14,M3-10 | message/snapshot matrix | Linux/macOS walkthrough |
| NAV-01 | 세로 우선 배치 | M1-03,06 | index mapping tests | 없음 |
| NAV-02 | 상하좌우 공간 탐색 | M1-06 | navigation table | 키 감각 확인 |
| NAV-03 | 마지막 컬럼 Nearest | M1-06 | B3→C2 test | 없음 |
| NAV-04 | Home/End/PgUp/PgDn | M1-06 | capacity boundary tests | 없음 |
| NAV-05 | resize 후 같은 파일 유지 | M1-06,12 | resize scenario | 연속 resize |
| NAV-06 | Up/Down/Left/Right가 표시 경계에서 인접 페이지 연결 | M1-06 | four-direction boundary tests | 키 감각 확인 |
| SEL-01 | Cursor/Marked 4상태 | M1-07,08,10 | style/state tests | 없음 |
| SEL-02 | Space/Insert/Ctrl+A | M1-07 | reducer tests | 없음 |
| SEL-03 | 선택 수와 바이트 합계 | M1-07,10 | state+snapshot | 없음 |
| KEY-01 | F1~F12 전부 항상 표시 | M1-09,10,13 | 80열 registry snapshot | 없음 |
| KEY-02 | 표시와 실제 mapping 단일 원본 | M1-10,13,M3-02 | registry invariant | 없음 |
| KEY-03 | 마우스 없이 핵심 기능 사용 | M3-08~10 | full scenario | 수동 walkthrough |
| KEY-04 | disabled F키가 별도 style과 이유를 표시 | M1-13 | availability/style tests | 없음 |
| KEY-05 | R Refresh 표시와 실제 mapping이 Registry 한 정의를 사용 | M1-09,13 | registry/reducer scenario | 없음 |
| THEME-01 | Classic Mdir 기본 | M1-08,10,13 | palette unit+style-aware snapshot | 참고 이미지 비교 |
| THEME-02 | 파일 형식별 색상 | M1-08 | style table tests | 없음 |
| THEME-03 | 외부 theme 무재빌드 적용 | M3-04 | load scenario | 파일 교체 확인 |
| FS-01 | Enter 디렉터리/파일 실행 | M1-11 | recording launcher | ShellExecute |
| FS-02 | Backspace 상위 이동 | M1-11 | scenario | drive/UNC root |
| FS-03 | Rename Unicode/검증 | M2-03 | unit+tempdir | Linux/macOS 파일명 |
| FS-04 | MkDir | M2-03 | unit+tempdir | 없음 |
| FS-05 | Viewer 텍스트 탐색/검색 | M2-04,05 | state+snapshot | 큰 파일 감각 |
| FS-06 | 기본 Editor 기능 | M2-06,07 | buffer+save tests | 입력 감각 |
| FS-07 | Copy 충돌 6종/진행률 | M2-08~10 | fault scenarios | 큰 파일 |
| FS-08 | Move와 cross-volume fallback | M2-11 | mocked device test | 실제 드라이브 |
| FS-09 | Delete 기본 휴지통 | M2-12 | RecordingTrash | Linux/macOS 휴지통 |
| FS-10 | 영구 삭제 별도 확인 | M2-12 | tempdir safety tests | 경고 문구 |
| FS-11 | 작업 중 UI 응답 | M2-09,14 | slow worker scenario | 큰 파일+resize |
| VIEW-01 | Name/Ext/Size/Date/Time의 exact comparator, missing-last, stable tie-break | M2-13 | full sort table | 없음 |
| VIEW-02 | Directories First | M1-05,M2-13 | sort tests | 없음 |
| VIEW-03 | show_hidden=true 기본, H 토글과 선택 fallback | M2-13 | attr/filter tests | Linux/macOS 확인 |
| VIEW-04 | Main S/Ctrl+S/H/D command와 Help, Editor Ctrl+S context가 단일 Registry 계약을 지킴 | M2-13 | command/context scenarios | 키 감각 확인 |
| MCD-01 | 트리 이동/지연 로드 | M3-05,06 | tree tests+scenario | reference 비교 |
| MCD-02 | 검색/최근 경로 | M3-05,06 | filter tests | 없음 |
| MCD-03 | F2 Rescan/F3 Drive | M3-06 | recording effects | 실제 drive |
| QCD-01 | 추가/수정/삭제/숫자 이동 | M3-07 | reducer+roundtrip | 없음 |
| MENU-01 | F12 전체 기능 메뉴 | M3-08 | every leaf mapping | walkthrough |
| CFG-01 | 마지막 상태 저장 | M3-01 | restart scenario | 실제 재시작 |
| CFG-02 | TOML 손상 복구 | M3-01 | broken-file tests | 없음 |
| CFG-03 | 사용자 키맵 | M3-02 | override scenario | 키 변경 확인 |
| TEST-01 | Unit/navigation/layout 자동화 | M1-12 이후 | CI | 없음 |
| TEST-02 | TestBackend snapshot | M1-10 이후 | CI | diff review |
| TEST-03 | YAML 키/effect/assert 시나리오 재생 | M1-12,13 | parser/run tests | 없음 |
| TEST-04 | 80/100/120/160 크기 | M1-13 | snapshot matrix | 없음 |
| TEST-05 | 0~10,000 항목 경계 | M1-06,12 | boundary tests | 없음 |
| TEST-06 | 문자와 스타일을 통합 snapshot 검사 | M1-13 | style-aware serializer | 없음 |
| TEST-07 | 실제 terminal 없는 자동 gate | M0-03 | Linux/macOS build 또는 CI | 없음 |
| PERF-01 | key→map→reduce→render 50 ms 목표 | M1-13 | ignored release smoke | 체감 확인 |
| PERF-02 | 10,000개 sort+layout+render 100 ms 목표 | M1-13 | ignored release smoke | 없음 |

## v1.0 승인 규칙

- `progress.md` ledger에서 표의 모든 ID가 `완료`여야 한다.
- 자동 검증 항목은 test 이름 또는 CI 실행 링크가 있어야 한다.
- 수동 검증 항목은 OS/terminal/날짜/결과를 기록해야 한다.
- “테스트하기 어려움”은 면제 사유가 아니다. 포트 또는 상태 분리를 먼저 검토한다.

## 현재 M1 증거와 종료 gap

2026-07-25 기준 전체 64 tests와 Clippy 경고 0이다. 적응형 컬럼, `│` 경계와 네 방향
페이지 연결까지 회귀 검증한다. 현재 자동 증거가 있는 범위는 다음과 같다.

- ARC-01,03: input/reducer와 Memory/Real filesystem read tests
- UI-01,02,04,05: 80×25/too-small 문자 snapshot과 layout mode/adaptive tests
- NAV-01~06, SEL-01~03: mapping/capacity/resize/page boundary와 selection/palette tests
- KEY-01, THEME-02: F1~F12 문자 렌더와 file-role palette unit tests
- FS-01~02: recording launcher worker와 기본 parent navigation scenario
- TEST-01,02,05: unit/component/문자 snapshot/10,000-entry debug smoke

다음은 `M1-13`이 직접 닫아야 하는 자동 증거 gap이다.

- ARC-02, UI-03: recording-port render boundary와 long-path snapshot
- UI-08의 M1 부분: built-in English copy와 Unicode 사용자 path 보존
- UI-07, KEY-04: raw modified+OS-local/fallback/RHSA와 disabled F-key style/reason
- KEY-02: 기본 `Q Quit` 잔존 제거와 Registry 단일 원본
- THEME-01 자동 부분: 문자+Style 통합 snapshot
- TEST-03: clock 주입/effect completion/단계별 assertion
- TEST-04/06: 100/160열 및 문자+Style 통합 snapshot
- PERF-01/02: 실제 범위를 측정하는 이름 있는 ignored release smoke

다음 항목은 v1.0 최종 승인에는 필요하지만 `M1-13` 완료 조건이 아니다.

- TEST-07: 원격 CI는 선택 사항이며 local-only release는 플랫폼별 build/test 로그를 연결한다.
- THEME-01 참고 이미지 비교, FS-01 실제 ShellExecute, FS-02 drive/UNC와 terminal 조합은
  `R1-02`의 동일 hash RC 수동 증거로 남는다.

따라서 위 자동 gap이 0이고 공통 게이트가 통과하면 M1-13을 완료하고 M2로 진행할 수 있다.
R1 이관 항목은 ledger에서 owner와 상태를 유지하며 R1-03 전에는 v1.0 완료로 표시하지 않는다.

## v1 이후 Git 확장

Git built-in은 이 v1.0 표의 완료 조건에 포함하지 않는다. `R1` 완료 후 별도
[`../plugins/git/05-acceptance-matrix.md`](../plugins/git/05-acceptance-matrix.md)의
`PLUG-*`, `GIT-*`, `LOCAL-*`, `GITNET-*` 기준을 단계별로 사용한다.

## v1 이후 SSH Remote / Remote Drive

SSH Remote는 이 v1.0 표나 Git `GITNET-*`(fetch/push)와 별개다. `R1` 완료 후
[`../remote/05-acceptance-matrix.md`](../remote/05-acceptance-matrix.md)의
단계별 상세 ID와 `S0-GATE-*` → `S1-GATE-*` → `S2-GATE-*` → `S3-GATE-*`를 사용한다.
