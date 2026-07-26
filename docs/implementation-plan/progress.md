# 구현 진행 상황

마지막 갱신: 2026-07-25

## 현재 기준선

- 브랜치: `main`
- Git 커밋: `760a90f` 최초 기준선
- Rust 도구 체인: stable 1.97.1 설치됨
- 코드: 기본 디렉터리 탐색이 가능한 Rust/Ratatui 애플리케이션
- 문서: 원본 요구사항, 요구사항 검토, UI 참고 이미지, 구현 계획 작성됨
- 현재 자동 기준선: 전체 93 tests passed, Clippy 경고 0
- 다음 구현 카드: `R1-02 고정 RC Linux/macOS 실제 환경 수동 시험`
- 후속 문서: Git built-in `G0~G3`, SSH Remote `S0~S3` 작성됨; production 구현 미착수

체크박스 `[ ]`에는 `미착수`, `진행`, `외부 대기`가 모두 포함될 수 있다. 실제 실행 순서는
위의 “다음 구현 카드”가 권위다. `M0-03`은 외부 대기이므로 로컬 에이전트가 반복 실행하지
않고 R1-02를 진행한다.

## 단계 요약

| 트랙 | 상태 | 다음 조건/카드 |
|---|---|---|
| M0 | 로컬 기준선 완료 | Linux build 증거는 R1-01에서 추가 |
| M1 | 완료 | M2 인계 완료 |
| M2 | 완료 | M3 인계 완료 |
| M3 | 완료 | R1 인계 완료 |
| R1 | 진행 | R1-02 Linux/macOS 실제 환경 수동 시험 |
| Git G0~G3 | 계획 완료, 구현 미착수 | R1 후 G0-01 |
| SSH Remote S0~S3 | 계획 완료, 구현 미착수 | R1 후 S0-00 |

## M0

- [x] M0-01 Rust 도구 체인 설치와 환경 기록
  - 완료일: 2026-07-24
  - 변경: `rust-toolchain.toml`, `docs/development.md`
  - 검증: 로그인 zsh에서 rustup/rustc/cargo/rustfmt/clippy 버전 확인
  - 증거: rustc 1.97.1, cargo 1.97.1, stable-aarch64-apple-darwin
  - 남은 위험: Linux toolchain은 R1-01 build 환경에서 검증
- [x] M0-02 Cargo 패키지와 최소 모듈 생성
  - 완료일: 2026-07-24
  - 변경: `Cargo.toml`, `Cargo.lock`, `src/main.rs`, `src/lib.rs`, `.gitignore`
  - 검증: main→library run source inspection과 실제 build/fmt/clippy/test 성공
  - 증거: mdir4 0.1.0, edition 2024, Cargo.lock과 후속 실제 tests 생성
  - 결정: pre-v1은 rolling stable이며 `package.rust-version`/MSRV와 placeholder
    `crate_builds()`를 약속하지 않음; R1-01이 RC compiler version 기록
  - 남은 위험: dependency별 MSRV 약속은 v1 이후 별도 audit 전 미정
- [x] M0-03 품질 파이프라인
  - 구현일: 2026-07-25
  - 변경: `.github/workflows/ci.yml`
  - 로컬 검증: workflow YAML, Bash snapshot 검사, fmt/clippy/test `--locked` 성공
  - 결정: local-only release이므로 push/원격 CI URL은 완료 조건에서 제외
  - 상태: 로컬 gate 성공, Linux/macOS workflow 정의 완료
- [x] M0-04 터미널 수명주기와 복구
  - 완료일: 2026-07-25
  - 변경: `src/runtime.rs`
  - 검증: 정상 drop, 부분 초기화 실패, panic unwind 복구 순서 단위 테스트
  - 수동 검증: 실제 PTY에서 Ctrl+Q 종료 후 cursor/alternate screen/raw mode 복구
  - 증거: 전체 15 tests passed, Clippy 경고 0
  - 남은 위험: 없음
- [x] M0-05 저장소 기준선 확정
  - 완료일: 2026-07-25
  - 상태: commit 0개, 모든 프로젝트 파일 untracked; 첫 commit은 사용자 승인 전 보류
  - 검토: 후보 101개, secret 실값/build 산출물/임시 파일/`.snap.new` 0
  - 증거: `docs/baseline/m0-05-precommit.sha256`에 후보 전체 content SHA-256 기록 후
    `shasum -a 256 -c`로 즉시 재검증
  - 검증: fmt, Clippy `-D warnings`, 전체 64 tests 통과
  - 금지: 사용자 승인 전 stage/commit/push

## M1

- [x] M1-01 핵심 모델과 reducer 골격
  - 완료일: 2026-07-24
  - 변경: `src/app.rs`, `src/lib.rs`
  - 검증: reducer 단위 테스트
  - 증거: Action→State/Effect 흐름과 2개 reducer test
  - 남은 위험: 파일 작업 Action/Effect는 M2에서 확장
- [x] M1-02 Unicode 셀 폭과 말줄임
  - 완료일: 2026-07-24
  - 변경: `src/ui.rs`
  - 검증: Unicode 셀 폭 단위 테스트와 80×25 snapshot
  - 증거: `unicode_truncation_respects_cell_width`
  - 남은 위험: 편집기 grapheme 이동은 M2에서 별도 구현
- [x] M1-03 LayoutEngine과 컬럼 계산
  - 완료일: 2026-07-25
  - 변경: `src/layout.rs`, `src/app.rs`, `src/ui.rs`, `src/runtime.rs`
  - 검증: Auto/Fixed, Compact/Normal/Wide/Custom, 크기·경계·불변식 테스트
  - 증거: 전체 23 tests passed, 80×25 snapshot 유지, Clippy 경고 0
  - 남은 위험: 설정 UI와 영속화는 M3-01/M3-09에서 연결
- [x] M1-04 FileSystem 포트와 메모리 구현
  - 완료일: 2026-07-25
  - 변경: `src/ports/filesystem.rs`, `src/adapters/memory_fs.rs`,
    `tests/memory_filesystem.rs`, `tests/support/builders.rs`, module exports
  - 검증: empty/basic/unicode/nested/permission-denied fixture, Windows drive/UNC
    lexical normalization, 호출 기록과 오류 구분 테스트; 전체 품질 게이트
  - 증거: 대상 7 tests passed, 전체 30 tests passed, Clippy 경고 0
  - 후속 해소: 실제 디스크 adapter는 M1-05, runtime worker/port 주입은 M1-11에서 완료
- [x] M1-05 실제 디렉터리 로드와 정렬
  - 완료일: 2026-07-25
  - 변경: `src/adapters/real_fs.rs`, `src/model/directory.rs`, `src/fs.rs`,
    `src/app.rs`, `src/runtime.rs`, `tests/directory_loading.rs`, module exports
  - 검증: Directories First + Name Ascending/tie-break/Unicode, native·drive·UNC
    root parent, empty/permission error, metadata fallback, TempDir integration
  - 수동 검증: 실제 PTY에서 현재 작업공간 로드 후 Ctrl+Q 종료, terminal 복구
  - 증거: 대상 6 integration tests, 전체 39 tests passed, Clippy 경고 0
  - 후속 해소: 디렉터리 I/O는 M1-11에서 worker로 분리
- [x] M1-06 공간 탐색과 페이지 계산
  - 완료일: 2026-07-25
  - 변경: `src/layout/navigation.rs`, `src/layout.rs`, `tests/navigation.rs`
  - 검증: index↔page/row/column, A/B/C 양방향·Nearest, page capacity 경계,
    0/1/100/1,000/10,000개, resize 선택 경로 유지
  - 증거: 대상 3 tests, 전체 42 tests passed, Clippy 경고 0
  - 남은 위험: 없음
- [x] M1-07 마킹과 선택 합계
  - 완료일: 2026-07-25
  - 변경: `src/model/selection.rs`, `src/model/mod.rs`, `src/fs.rs`,
    `src/app.rs`, `tests/selection.rs`
  - 검증: Space/Insert/Ctrl+A, parent 제외, cursor/marked 대상, 디렉터리 포함 합계,
    same-directory refresh 교집합·선택 경로 유지, directory change clear
  - 증거: 대상 4 tests, 전체 46 tests passed, Clippy 경고 0
  - 남은 위험: 없음
- [x] M1-08 Classic Mdir 테마와 스타일 역할
  - 완료일: 2026-07-25
  - 변경: `src/theme/{mod,schema,classic}.rs`, `src/ui/palette.rs`, `src/ui.rs`,
    module exports
  - 검증: 파일 종류별 role, Normal/Cursor/Marked/Cursor+Marked 우선순위,
    palette Style unit test와 기존 80×25 문자 snapshot 유지, UI 색상 상수 경계 검색
  - 증거: 신규 2 style tests, 전체 48 tests passed, Clippy 경고 0
  - 남은 위험: 외부 테마 로드는 M3-04 범위
- [x] M1-09 CommandRegistry와 입력 매핑
  - 완료일: 2026-07-25
  - 변경: 정규화된 `KeyChord`, 화면 우선 `InputMapper`, 단일 `CommandRegistry`
  - 검증: press/repeat/release, F1~F12와 disabled F9, 표시-command/key 1:1 invariant
  - 증거: 대상 5 tests, 전체 53 tests passed, Clippy 경고 0
- [x] M1-10 메인 화면과 기본 Help 렌더링
  - 완료일: 2026-07-25
  - 변경: metrics-only render, 3단계 Short View, registry Help, 종료 확인 overlay
  - 검증: empty/basic/unicode/marked/help/quit/80x25/120x40/too-small snapshots
  - 증거: 9 screen snapshots, 전체 54 tests passed, Clippy 경고 0
- [x] M1-11 런타임과 기본 Effect 실행
  - 완료일: 2026-07-25
  - 변경: 단일 effect worker, DiskInfo/FileLauncher 포트, 50 ms tick과 dirty redraw
  - 검증: effect 완료 순서, 정확한 launch path, Ctrl+Q 확인, idle tick 비-redraw
  - 증거: 신규 3 tests, 전체 57 tests passed, Clippy 경고 0
- [x] M1-12 YAML 시나리오 최소 하네스 기준선
  - 완료일: 2026-07-25
  - 변경: versioned YAML schema, fixture clock/disk/fs/terminal, 실제 mapper/reducer/render 재생
  - 검증: startup/navigation/selection/resize, parser file+step error, 10,000 항목 smoke
  - 증거: 대상 3 tests와 4 scenario snapshots, 전체 60 tests passed, Clippy 경고 0
  - 후속 보정: Auto Short View는 현재 항목을 담는 최소 컬럼 수(폭 기준 최대 이내)만
    사용하고 유효 컬럼을 균등 확장한다. 컬럼 `│` border와 Up/Down/Left/Right 페이지
    경계 연결을 추가했다.
  - 현재 증거: 적응형 1/2/최대열, border cell, 네 방향 페이지 경계 test 포함
    전체 64 tests passed, Clippy 경고 0
  - 회귀 test: `auto_columns_expand_and_grow_with_the_entry_count`,
    `column_separator_uses_box_drawing_border_cells`,
    `down_and_up_cross_page_boundaries_at_the_last_and_first_visible_items`,
    `left_and_right_cross_pages_and_preserve_the_nearest_row`
- [x] M1-13 M1 계약 정합성 보정과 종료
  - 선행: M0-05
  - 남은 계약: raw modified+주입 timezone+R/H/S/A, disabled F-key style/reason, Registry 기반 Ctrl+Q
    힌트, 재사용 text helper, FixedClock/effect/assert scenario, style snapshot matrix,
    이름 있는 release performance smoke, built-in English copy/Unicode 사용자 경로 보존
  - 완료 범위: M1-13 소유 자동 gap 0과 공통 gate. TEST-07 외부 CI는 R1-01,
    참고 이미지/ShellExecute/drive·UNC/terminal 수동 증거는 R1-02에서 닫는다.
  - 완료일: 2026-07-25
  - 증거: metadata/timezone/attributes, 공통 Unicode text helper, Registry Ctrl+Q와
    M1 snapshot 갱신, 전체 품질 게이트 성공

## M2

- [x] M2-01 공통 대화상자 상태와 렌더러
- [x] M2-02 파일 I/O·mutation 포트와 결정적 Fake
- [x] M2-03 Rename과 MkDir
- [x] M2-04 Viewer 로드·decode 모델
- [x] M2-05 Viewer UI와 검색
- [x] M2-06 EditorBuffer
- [x] M2-07 Editor UI와 안전 저장
- [x] M2-08 작업 계획과 충돌 모델
- [x] M2-09 Worker와 진행률 프로토콜
- [x] M2-10 Copy
- [x] M2-11 Move
- [x] M2-12 Delete와 휴지통
- [x] M2-13 정렬, 숨김 파일, 드라이브
- [x] M2-14 M2 통합 수용

완료일: 2026-07-25

- 공통 modal, Rename/MkDir, Viewer/검색, Editor/검색/Undo/Redo/안전 저장,
  Copy/Move/Delete, 시스템 Trash, 정렬/숨김/드라이브 선택을 영어 UI에 연결했다.
- capacity 16 bounded worker, OperationId, cancellation/deadline, 진행률, mutation lease와
  UI↔worker conflict round-trip을 구현했다.
- Copy 충돌 6종, cross-device Move, 외부 저장 충돌, atomic replace, 보호 삭제를 자동 검증했다.
- 공통 게이트: fmt, Clippy `-D warnings`, 전체 81 tests 성공.
- Linux/macOS 실제 Trash/mount/파일명 수동 감각은 R1-02가 소유한다.

## M3

- [x] M3-01 TOML 설정 로드/저장/복구
- [x] M3-02 사용자 키맵
- [x] M3-03 Long View와 Tab 전환
- [x] M3-04 외부 테마
- [x] M3-05 MCD 트리 모델
- [x] M3-06 MCD UI와 비동기 로드
- [x] M3-07 QCD
- [x] M3-08 F12 Menu와 Help 완성
- [x] M3-09 설정 화면
- [x] M3-10 M3 통합 수용

완료일: 2026-07-25

- TOML 설정의 원자적 저장, 손상 파일 복구, 시작 경로 fallback과 사용자 키맵 진단을 구현했다.
- Short/Long View, 내장·외부 테마, MCD 비동기 트리, QCD CRUD/재정렬을 연결했다.
- F12 메뉴와 Help가 Command Registry를 공유하고, 설정 화면은 preview/apply/cancel을 지원한다.
- 공통 게이트: fmt, Clippy `-D warnings`, 전체 93 tests 성공.
- Linux/macOS 실제 터미널·파일시스템 감각 검증은 고정 RC를 만든 뒤 R1-02가 소유한다.

## R1

- [x] R1-01 릴리스 후보 빌드와 패키징
  - 구현일: 2026-07-25
  - 변경: release profile, cross-platform packager, dependency license inventory,
    Linux/macOS artifact build job, RC record
  - 로컬 검증: locked release build, macOS arm64 ZIP 구성과 SHA-256, 공통 93-test gate 성공
  - Linux 검증: Ubuntu 26.04 arm64 Lima VM, rustc/cargo 1.97.1, fmt/Clippy/93 tests/release 성공
  - artifact-only smoke: Linux/macOS ZIP을 새 임시 디렉터리에 풀어 directory load,
    Ctrl+Q/Enter 정상 종료와 terminal restoration 성공
  - R1-02 진행: macOS 고정 ZIP으로 한글/긴 경로와 10,000개 파일(`Items 10001`)을
    adaptive multi-column 화면에서 로드·확인
  - 고정 hash: macOS ZIP `f35d2956...34840f`, Linux ZIP `bf9b6685...7664b2`
  - 상태: 완료; source commit `ee73764`
- [ ] R1-02 고정 RC Linux/macOS 실제 환경 수동 시험
- [ ] R1-03 v1.0 최종 게이트

## v1 이후 Git built-in

상세 카드는 [`../plugins/git/03-task-cards.md`](../plugins/git/03-task-cards.md), 수용 기준은
[`../plugins/git/05-acceptance-matrix.md`](../plugins/git/05-acceptance-matrix.md)에 있다.
현재 v1 구현 순서에는 포함하지 않는다.

- [ ] G0 Built-in Extension Host
- [ ] G1 Read-only Git
- [ ] G2 Local Git Mutations
- [ ] G3 Remote Git Operations

## v1 이후 SSH Remote / Remote Drive

상세 계약은 [`../remote/README.md`](../remote/README.md), 작업 카드는
[`../remote/03-task-cards.md`](../remote/03-task-cards.md), 수용 기준은
[`../remote/05-acceptance-matrix.md`](../remote/05-acceptance-matrix.md)에 있다.
현재 v1 완료 조건과 제품 범위는 Git과 독립적이다. 한 후속 트랙만 존재할 때 상대 트랙의
실환경 행은 `N/A — not implemented`로 기록해 그 단계 완료를 막지 않는다. 두 트랙이 모두
존재하는 첫 단계 gate부터 공용 path/config/runtime integration 행을 필수로 실행한다.

- [ ] S0 Location Foundation
- [ ] S1 Remote Browse/View
- [ ] S2 Transfer/Mutation — Remote Drive MVP
- [ ] S3 Cache/Registration/Hardening

## 진행 메모

M1~M3 기능은 사용 가능하고 현재 93-test 회귀 기준선이 있다. 첫 commit은
사용자 승인 전 만들지 않았으며 다음 구현 순서는 `R1-01`이다.
`M0-03` workflow는 로컬 검증됐지만 GitHub 원격 실행 증거가 없어 외부 대기 상태다.
Linux/macOS launcher와 terminal 조합 증거는 고정 RC를 만든 뒤 R1-02에서 기록한다.
Git built-in과 SSH Remote 계획은 문서만 추가했으며 현재 v1 순서를 바꾸지 않는다.

## 사용자 검토 권장 결정

아래는 구현을 구체화하기 위해 기본값을 정한 항목이다. 사용자가 변경하지 않으면
제품 계약대로 진행한다.

- 현대식 F키 배치를 기본으로 하고 원본 키맵 프리셋은 v1에서 제외
- F4 기본 Editor를 M2에 포함
- Short View는 컬럼 폭에 따라 메타데이터를 단계적으로 표시
- 상단 번호 메뉴는 제외하고 F12로 통합
- 설정 형식은 TOML 하나
- MCD 검색은 로드된 노드와 최근 경로 대상, 전체 디스크 색인은 제외
- symlink는 재귀 작업에서 따라가지 않음

## 결정 필요

제품 결정 blocker는 없다. 단, M0-05의 첫 commit 생성은 사용자 승인이 필요하다.

## Acceptance evidence ledger

상태는 `미착수 / 부분검증 / 자동검증 / 외부대기 / 수동검증 / 완료`를 사용한다. 정적 ID와
검증 방법은 `05-acceptance-matrix.md`가 권위이며, 실제 test 이름/CI URL/수동 환경은 이
ledger에 추가한다. 범위 표기만으로 R1 완료를 선언하지 않고 R1-03에서 모든 ID를 개별
evidence로 펼친다.

| ID/범위 | 현재 상태 | 현재 증거 또는 다음 행동 |
|---|---|---|
| ARC-01, ARC-03 | 자동검증 | reducer/input, Memory/Real read tests |
| ARC-02 | 자동검증 | worker/port 분리와 render I/O 없음 |
| UI-01,02,04,05 | 자동검증 | 문자 snapshots, layout/adaptive tests |
| UI-03 | 부분검증 | Unicode path 있음; M1-13 long-path 추가 |
| UI-07, KEY-04 | 자동검증 | raw modified/local timezone/fallback/RHSA와 availability snapshot |
| UI-08 | 부분검증 | 현재 M1 snapshots은 English copy/Unicode 이름을 포함; M1-13 명시 검증과 M2/M3 gate 남음 |
| NAV-01~06, SEL-01~03 | 자동검증 | navigation/selection/palette regression |
| KEY-01 | 자동검증 | F1~F12 문자 렌더/registry tests |
| KEY-02 | 자동검증 | mapping invariant와 Ctrl+Q snapshot |
| KEY-05 | 부분검증 | Registry의 R/Refresh→Reload와 snapshots 있음; M1-13 전용 reducer scenario 및 hardcoded hint 제거 |
| THEME-01 | 부분검증 | M1-13 style snapshot; R1-02 수동 reference 비교 |
| THEME-02 | 자동검증 | file-role style table tests |
| FS-01,02 | 부분검증 | launcher/parent 자동; R1-02 Linux/macOS mount 수동 남음 |
| TEST-01,02,05 | 자동검증 | 현재 93-test 기준선 |
| TEST-03 | 부분검증 | 4 YAML flows; M1-13 FixedClock/named completion/assert 남음 |
| TEST-04,06, PERF-01 | 미착수 | M1-13 snapshot matrix/style serializer/release smoke |
| PERF-02 | 부분검증 | debug End/Home/layout/render만 존재; M1-13 full release smoke |
| TEST-07 | 부분검증 | macOS local no-terminal gate 성공; R1-01 Linux build 로그 추가 |
| UI-06, KEY-03, FS-03~11, VIEW-01~04 | 자동검증 | M2 reducer/worker/tempdir/MemoryFS/sort/Trash tests |
| THEME-03 | 자동검증 | 내장 테마와 외부 TOML 상속/검증 tests |
| MCD-01~03, QCD-01, MENU-01, CFG-01~03 | 자동검증 | M3 model/reducer/render/config tests |
| ENV-01,02 | 미착수 | R1-01 RC와 R1-02 동일 hash 수동 matrix |

## 검증 증거

| 날짜 | 카드 | 명령/환경 | 결과 | 증거 |
|---|---|---|---|---|
| 2026-07-24 | 계획 기준선 | `rustc --version` | command not found | M0-01 필요 |
| 2026-07-24 | M0-01 | 로그인 zsh에서 toolchain 명령 5개 | 성공 | stable 1.97.1, rustfmt/clippy 설치 |
| 2026-07-24 | 기본 탐색 | `cargo clippy ... -D warnings` | 성공 | 경고 0 |
| 2026-07-24 | 기본 탐색 | `cargo test --all-targets --all-features` | 성공 | 11 passed |
| 2026-07-24 | 기본 탐색 | 실제 PTY 80×24 | 성공 | docs 진입, architecture 진입/복귀, Help, 종료 복구 |
| 2026-07-25 | M0-03 | CI와 동일한 `--locked` 품질 게이트 | 성공 | 11 passed, Clippy 경고 0 |
| 2026-07-25 | M0-03 | workflow YAML/Bash 구문과 snapshot 검사 | 성공 | 로컬 정적 검증 |
| 2026-07-25 | M0-04 | terminal lifecycle/rollback/unwind tests | 성공 | 전체 15 passed |
| 2026-07-25 | M0-04 | 실제 PTY Ctrl+Q 종료 | 성공 | cursor/alternate screen/raw mode 복구 |
| 2026-07-25 | M1-03 | layout mode/boundary/invariant tests | 성공 | 전체 23 passed |
| 2026-07-25 | M1-04 | MemoryFileSystem 계약 테스트와 전체 품질 게이트 | 성공 | 대상 7개, 전체 30개, Clippy 경고 0 |
| 2026-07-25 | M1-05 | DirectoryListing/RealFileSystem/TempDir와 전체 품질 게이트 | 성공 | 대상 6개, 전체 39개, Clippy 경고 0 |
| 2026-07-25 | M1-05 | 실제 PTY 작업공간 로드/종료 | 성공 | 목록·상태 표시와 terminal 복구 |
| 2026-07-25 | M1-06 | navigation table/boundary/large-list와 전체 품질 게이트 | 성공 | 대상 3개, 전체 42개, Clippy 경고 0 |
| 2026-07-25 | M1-07 | selection/refresh/operation-target와 전체 품질 게이트 | 성공 | 대상 4개, 전체 46개, Clippy 경고 0 |
| 2026-07-25 | M1-08 | Classic theme role/style와 전체 품질 게이트 | 성공 | 신규 2개, 전체 48개, Clippy 경고 0 |
| 2026-07-25 | M1-09 | input normalization/registry invariant와 전체 게이트 | 성공 | 대상 5개, 전체 53개, Clippy 경고 0 |
| 2026-07-25 | M1-10 | 필수 UI 상태와 크기 snapshot | 성공 | 9 snapshots, 전체 54개, Clippy 경고 0 |
| 2026-07-25 | M1-11 | worker/effect/launcher/idle runtime | 성공 | 신규 3개, 전체 57개, Clippy 경고 0 |
| 2026-07-25 | M1-12 | YAML scenarios와 10,000 항목 smoke | 성공 | 대상 3개, 전체 60개, Clippy 경고 0 |
| 2026-07-25 | M1 layout 보정 | 항목 수 적응형 Auto columns와 snapshot 갱신 | 성공 | 신규 1개, 전체 61개, Clippy 경고 0 |
| 2026-07-25 | M1 UX 보정 | box border와 네 방향 페이지 경계 연결 | 성공 | 신규 3개, 전체 64개, Clippy 경고 0 |
