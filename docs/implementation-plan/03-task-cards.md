# 작업 카드

## 사용법

- 위에서 아래 순서로 진행한다.
- `선행` 카드가 모두 완료되기 전에는 시작하지 않는다.
- 한 카드에서는 기재된 산출물만 수정한다. 구조상 다른 파일이 꼭 필요하면
  `progress.md`에 이유를 남긴다.
- 각 카드의 테스트가 먼저 실패하는 것을 확인하고 구현한다.
- 완료 후 `progress.md`의 체크박스, 날짜, 커밋/테스트 증거를 갱신한다.

## 실행 색인

완료된 M0/M1 세부 카드는 역사적 계약으로 보존한다. 새 작업자는 파일에서 눈에 띄는 첫
unchecked 항목을 임의로 고르지 말고 `progress.md` 상단의 “다음 구현 카드”를 따른다.
완료 카드의 `파일`은 당시 목표 구조일 수 있다. 실제 산출물은 `progress.md`의 `변경`과
현재 source tree가 권위이며, 계획에만 있는 디렉터리를 완료 카드와 맞추려고 새로 만들지
않는다.

| 순서 | 카드/범위 | 현재 의미 |
|---|---|---|
| 1 | M0-05 | 활성: untracked 전체 tree의 검토 기준선 확정 |
| 2 | M1-13 | M1 계약/증거 gap 종료 |
| 3 | M2-01과 M2-02 | 독립 시작 가능: dialog 기반과 I/O port 기반 |
| 4 | M2-03~07 | Rename/MkDir, Viewer, Editor |
| 5 | M2-08~14 | planner/worker 후 Copy→Move, Delete, view, 통합 |
| 6 | M3-01~10 | config core 후 각 기능이 자기 config fragment 추가 |
| 7 | R1-01→02→03 | RC package → 동일 hash 수동 시험 → 최종 승인 |

`M0-03`은 로컬 구현 완료/원격 CI 외부 대기다. GitHub 상태가 바뀌지 않았으면 반복해서
수행하지 않으며 R1 최종 gate 전에 증거만 회수한다.

```text
M0-05 → M1-13 ─┬→ M2-01 dialog ───────────┐
                └→ M2-02 I/O ports ───────┼→ M2 기능 카드 → M2-14
                                           └→ M3 → R1-01 → R1-02 → R1-03
```

---

# M0 — 개발 기반

## M0-01 Rust 도구 체인 설치와 환경 기록

- 선행: 없음
- 목표: Linux와 macOS에서 동일한 stable Rust 기반을 쓴다.
- 작업:
  1. 개발 머신에 rustup으로 stable toolchain, rustfmt, clippy를 설치한다.
  2. `rust-toolchain.toml`에 `stable`, `rustfmt`, `clippy`를 선언한다.
  3. `docs/development.md`에 Linux/macOS 설치/검증 명령을 기록한다.
  4. 실제 `rustc -V`, `cargo -V` 결과를 `progress.md`에 기록한다.
- 검증: `rustc -V`, `cargo -V`, `cargo fmt --version`, `cargo clippy -V`
- 완료: 새 셸에서 네 명령이 성공하고 버전이 기록됨.

## M0-02 Cargo 패키지와 최소 모듈 생성

- 선행: M0-01
- 목표: `mdir4` 단일 binary/library 패키지를 빌드한다.
- 파일: `Cargo.toml`, `Cargo.lock`, `src/main.rs`, `src/lib.rs`, `.gitignore`
- 작업:
  1. 저장소 루트에서 `cargo init --bin --name mdir4 .`을 실행한다.
  2. edition 2024를 명시한다. pre-v1은 `rust-toolchain.toml`의 rolling stable을 사용하고
     `package.rust-version`/MSRV는 선언하지 않는다. R1-01이 고정 RC compiler version을
     기록하며, MSRV 약속은 별도 dependency audit 전에는 만들지 않는다.
  3. `lib.rs`를 만들고 `main.rs`는 library의 `run()`만 호출하게 한다.
  4. `02-architecture.md`의 의존성 중 M0/M1에 필요한 것만 추가한다.
  5. Cargo.lock을 생성한다.
- 검증: `main.rs`가 library `run()`만 호출하는 source inspection과 공통 완료 게이트 3개.
  별도 `crate_builds()` placeholder는 이후 실제 library/unit tests가 있으면 만들지 않는다.
- 완료: 경고 없이 debug build와 test 성공.

## M0-03 CI 품질 파이프라인

- 선행: M0-02
- 목표: Linux와 macOS에서 동일한 품질 게이트를 자동 실행한다.
- 파일: `.github/workflows/ci.yml`
- 작업:
  1. `ubuntu-latest`, `macos-latest` matrix를 만든다.
  2. stable toolchain + rustfmt + clippy를 설치한다.
  3. fmt, clippy `-D warnings`, test를 순서대로 실행한다.
  4. Cargo 캐시는 lockfile 기준으로 사용한다.
  5. snapshot `.snap.new`가 남으면 실패하도록 검사한다.
- 검증: workflow YAML 구문 확인, 로컬 공통 게이트.
- 완료: local-only release에서는 로컬 gate가 성공하고 두 OS build job이 정의되면 된다.
  원격 저장소를 사용할 경우 최초 GitHub 실행 결과를 추가 증거로 기록한다.

## M0-04 터미널 수명주기와 복구

- 선행: M0-02
- 목표: 정상 종료, 오류, 패닉에서 raw mode와 alternate screen을 복구한다.
- 파일: `src/runtime.rs`, `src/error.rs`, `src/main.rs`
- 작업:
  1. `TerminalGuard`가 진입 시 raw mode/alternate screen/cursor hide를 수행한다.
  2. `Drop`에서 cursor show/alternate leave/raw disable을 역순 실행한다.
  3. 패닉 훅은 기존 훅을 보존하고 터미널 복구 후 호출한다.
  4. 테스트에서는 실제 terminal을 켜지 않도록 작은 backend 진입점을 분리한다.
- 테스트: 복구 호출 순서를 recording adapter로 단위 테스트.
- 검증: 앱 실행 후 Ctrl+Q, 강제 오류 후 셸 입력/에코가 정상인지 수동 확인.
- 완료: 수동 확인 결과를 `progress.md`에 기록.

## M0-05 저장소 기준선 확정

- 선행: M0-02, M0-04
- 목표: 이후 카드의 변경 범위와 사용자 기존 파일을 확실히 구분할 기준선을 만든다.
- 현재 사실: Git 저장소에는 아직 커밋이 없고 모든 프로젝트 파일이 untracked다.
- 작업:
  1. `git status --short`와 `git ls-files --others --exclude-standard`로 기준선 후보를 기록한다.
  2. secret, 임시 파일, build 산출물, `.snap.new`가 포함되지 않았는지 검토한다.
  3. 공통 품질 게이트를 실행하고 결과와 전체 파일 목록을 `progress.md`에 기록한다.
  4. 사용자에게 기준선 commit 생성 승인을 받는다. 승인 전에는 stage/commit/push하지 않는다.
  5. 승인되면 첫 commit hash를 기록한다. remote 생성과 push는 별도 승인 전까지 보류할 수 있다.
  6. 사용자가 commit을 보류하면 `docs/baseline/m0-05-precommit.sha256`을 마지막으로 생성한다.
     `.git`, `target`, ignored 파일, `.snap.new`와 manifest 자신만 제외하고
     `git ls-files --cached --others --exclude-standard`의 모든 후보를 path 오름차순
     `SHA-256  path` 형식으로 기록한다. 다음 카드 시작 전에 전 항목 hash를 재검증한다.
- 검증: `git status --short`, `git diff --check`, untracked 파일 목록 수동 검토, 공통 게이트.
- 완료: 첫 commit hash가 기록되었거나, 사용자가 commit을 보류한 경우 품질 게이트 결과와
  모든 후보 파일의 content SHA-256 manifest가 기록되고 즉시 재검증됨. 파일 목록만 있는
  manifest는 기준선으로 인정하지 않는다.

---

# M1 — 화면 및 탐색

## M1-01 핵심 모델과 reducer 골격

- 선행: M0-02
- 목표: UI 없는 상태에서 Action→State/Effect 흐름이 컴파일된다.
- 파일: `src/app/*`, `src/model/entry.rs`, `src/model/directory.rs`
- 작업:
  1. `AppState`, `Screen`, `Action`, `Effect`, `Viewport`를 정의한다.
  2. 빈 reducer는 `Started`, `Resize`, `RequestQuit`, `ConfirmQuit`만 처리한다.
  3. 모든 상태에 명시적 `Default`를 구현한다.
  4. 상태 전이는 `reduce()` 밖에서 직접 하지 않는다는 모듈 문서를 추가한다.
- 테스트: 각 Action의 상태/Effect 결과 table test.
- 완료: reducer가 crossterm/ratatui/실제 FS에 의존하지 않음.

## M1-02 Unicode 셀 폭과 말줄임

- 선행: M0-02
- 목표: 한글, 일본어, 결합 문자, emoji를 컬럼 밖으로 출력하지 않는다.
- 파일: `src/layout/text.rs`, `tests/text_width.rs`
- 작업:
  1. grapheme 단위 iterator와 terminal cell width 합산 helper를 만든다.
  2. `truncate_end(text, max_cells, ellipsis)`를 구현한다.
  3. `pad_or_truncate`가 정확히 요청 셀 수를 만든다.
  4. 0, 1셀 폭과 폭이 없는 결합 문자 입력을 처리한다.
- 테스트: ASCII, `한글파일.txt`, `日本語.txt`, e+combining mark, emoji, 매우 긴 이름.
- 완료: 모든 결과의 실제 표시 폭을 assert.

## M1-03 LayoutEngine과 컬럼 계산

- 선행: M1-01
- 목표: 모든 터미널 크기에서 결정적 `LayoutMetrics`를 계산한다.
- 파일: `src/layout/metrics.rs`, `src/layout/engine.rs`, `tests/layout_boundaries.rs`
- 작업:
  1. 제품 계약의 5개 기본 영역을 Rect로 계산한다.
  2. Auto/Fixed와 Compact/Normal/Wide/Custom 공식을 구현한다.
  3. Auto에서는 항목을 담는 최소 유효 컬럼 수만 사용하고 폭 기준 최대치까지 늘린다.
  4. 컬럼 Rect의 합이 list 폭과 정확히 같게 나머지를 배분한다.
  5. 컬럼 사이 `│` 경계 1셀을 내용 폭에서 예약한다.
  6. 너무 작은 화면에서는 `too_small=true`, 빈 columns를 반환한다.
- 테스트:
  - 59×14, 60×15, 79×24, 80×25, 100×30, 120×40, 160×50
  - 79/80/81, 119/120/121, 159/160/161 경계
  - fixed 1~6, custom 11/12/80/81 clamp
- 완료: 겹침, 범위 밖 Rect, 0 나눗셈 없음.

## M1-04 FileSystem 포트와 메모리 구현

- 선행: M1-01
- 목표: 실제 디스크 없이 디렉터리 읽기와 오류를 재현한다.
- 파일: `src/ports/filesystem.rs`, `src/adapters/memory_fs.rs`, `tests/support/builders.rs`
- 작업:
  1. `FileSystem`, `FsError`, `EntryMetadata`를 정의한다.
  2. `MemoryFileSystemBuilder`로 파일, 디렉터리, 메타데이터, 권한 오류를 만든다.
  3. 경로 정규화는 Windows drive/UNC test data를 지원한다.
  4. 호출 기록을 제공해 불필요한 I/O를 assert할 수 있게 한다.
- 테스트: empty/basic/unicode/nested/permission-denied fixture.
- 완료: production UI 의존성 없이 전 테스트 성공.

## M1-05 실제 디렉터리 로드와 정렬

- 선행: M1-04
- 목표: RealFileSystem 결과를 안정적인 DirectoryListing으로 변환한다.
- 파일: `src/adapters/real_fs.rs`, `src/model/directory.rs`, `src/app/reducer.rs`
- 작업:
  1. `read_dir`에서 개별 항목 metadata 실패를 전체 실패와 구분한다.
  2. 루트가 아니면 합성 `..`를 앞에 추가한다.
  3. Directories First + Name Ascending 기본 정렬을 구현한다.
  4. `Started`/`LoadDirectory` effect와 `DirectoryLoaded` action을 연결한다.
  5. 로딩/빈 목록/오류 메시지 상태를 구분한다.
- 테스트: 대소문자, 같은 이름 tie-break, Unicode, 접근 거부, 빈 폴더.
- 완료: 실제 임시 디렉터리 integration test 성공.

## M1-06 공간 탐색과 페이지 계산

- 선행: M1-03, M1-05
- 목표: 제품 계약의 인덱스/행/열/페이지 규칙을 순수 함수로 구현한다.
- 파일: `src/layout/navigation.rs`, `src/app/reducer.rs`, `tests/navigation.rs`
- 작업:
  1. index↔page/row/column 변환 함수를 만든다.
  2. Up/Down/Left/Right/Nearest와 네 방향의 페이지 경계 연결을 구현한다.
  3. Home/End/PgUp/PgDn을 구현한다.
  4. 빈 목록, 1개, 마지막 불완전 컬럼을 처리한다.
  5. Resize 후 EntryId를 유지하고 새 index를 찾는다.
- 테스트: 원본 요구사항의 A1/B1/C1 표 전체와 역방향, 용량±1, 100/1,000/10,000개.
- 완료: 모든 탐색은 O(1), resize 재탐색만 O(n) 허용.

## M1-07 마킹과 선택 합계

- 선행: M1-05, M1-06
- 목표: 커서와 마킹을 독립적으로 관리한다.
- 파일: `src/model/selection.rs`, `src/app/reducer.rs`, `tests/selection.rs`
- 작업:
  1. marked `EntryId` set과 합계 계산을 구현한다.
  2. Space, Insert, Ctrl+A를 구현한다.
  3. `..`는 마킹하지 않는다.
  4. 디렉터리 이동 시 clear, refresh 시 교집합을 적용한다.
  5. 파일 작업 대상 선택 helper를 만든다.
- 테스트: cursor+marked 네 시각 상태, 디렉터리 포함 합계, 항목 삭제 후 refresh.
- 완료: 중복 마킹과 stale 경로 없음.

## M1-08 Classic Mdir 테마와 스타일 역할

- 선행: M0-02
- 목표: 화면 코드가 직접 색상 상수를 쓰지 않는다.
- 파일: `src/theme/schema.rs`, `src/theme/classic.rs`, `src/ui/palette.rs`
- 작업:
  1. 제품 계약의 화면별/역할별 theme token을 정의한다.
  2. 검정 main, 파랑 MCD, 자홍 dialog, cyan border/status 기본값을 만든다.
  3. Normal/Cursor/Marked/Cursor+Marked 우선순위를 함수로 만든다.
  4. 16색 터미널에서만 표현 가능한 기본 팔레트를 사용한다.
- 테스트: 각 파일 종류와 선택 조합의 최종 Style assert.
- 완료: `ui/`에서 `Color::...` 직접 사용이 theme adapter 외에는 없음.

## M1-09 CommandRegistry와 입력 매핑

- 선행: M1-01
- 목표: 기능키 바/Help/실제 키가 같은 정의를 사용한다.
- 파일: `src/input/key.rs`, `src/input/mapper.rs`, `src/app/command_registry.rs`
- 작업:
  1. crossterm KeyEvent를 repeat/release까지 고려해 KeyChord로 정규화한다.
  2. Main screen의 방향키, Enter, Backspace, Home/End, PgUp/PgDn,
     Space, Insert, Ctrl+A, R Refresh, Ctrl+Q, F1~F12를 등록한다.
  3. 미구현 F키는 disabled command로 유지한다.
  4. dialog/screen별 매핑이 main mapping보다 우선한다.
- 테스트: 모든 표시 command가 정확히 한 key mapping을 갖는지, F9 표시 여부.
- 완료: UI에 하드코딩한 기능키 문자열이 없음.

## M1-10 메인 화면과 기본 Help 렌더링

- 선행: M1-02, M1-03, M1-07, M1-08, M1-09
- 목표: AppState를 80×25 TestBackend에 정확히 그리고 F1 도움말을 제공한다.
- 파일: `src/ui/mod.rs`, `src/ui/components/path_bar.rs`,
  `file_list.rs`, `status_bar.rs`, `message_bar.rs`, `function_bar.rs`, `help.rs`
- 작업:
  1. render 함수는 `&AppState`, `&LayoutMetrics`만 받는다.
  2. Short View의 3단계 정보 밀도를 구현한다.
  3. 현재/마킹/둘 다 스타일을 구분한다.
  4. 경로, 현재 항목, 합계, free space placeholder, 메시지, F키를 그린다.
  5. F1 Help는 현재 활성 command를 CommandRegistry에서 읽어 표시한다.
  6. 작은 화면 안내와 최소한의 종료 확인 overlay를 구현한다.
- 스냅샷: empty, basic, unicode, marked, help, 80×25, 120×40, too-small.
- 완료: 버퍼 경계를 벗어난 셀 쓰기, 렌더 중 port 호출, 하드코딩 기능키 문자열이 없음.

## M1-11 런타임과 기본 Effect 실행

- 선행: M0-04, M1-05, M1-09, M1-10
- 목표: 실제 키로 탐색하고 디렉터리에 진입/복귀한다.
- 파일: `src/runtime.rs`, `src/ports/mod.rs`, `src/adapters/*`
- 작업:
  1. input→mapper→reducer→effects→completion action 큐를 구현한다.
  2. resize, 50 ms tick, redraw dirty flag를 처리한다.
  3. LoadDirectory용 단일 worker/채널과 DiskInfo, LaunchFile effect executor를 연결한다.
  4. Enter directory/file, Backspace, Ctrl+Q 확인을 구현한다.
  5. ShellExecute 어댑터는 셸 문자열을 만들지 않는다.
- 테스트: recording backend로 action/effect 순서와 launch path assert.
- 수동: macOS/Linux 터미널에서 탐색/리사이즈/파일 연결 실행.
- 완료: 렌더 루프가 대기 중 CPU를 계속 점유하지 않음.

## M1-12 YAML 시나리오 최소 하네스 기준선

- 선행: M1-06~M1-11
- 목표: 키 입력 파일을 실제 앱과 동일한 reducer/render 경로로 재생한다.
- 파일: `tests/support/harness.rs`, `tests/scenarios.rs`, `tests/scenarios/*.yml`
- 작업:
  1. `version`, `terminal`, `start_path`, inline `filesystem`, `clock`, `disk`,
     `steps`를 갖는 versioned schema를 구현한다.
  2. 현재 step은 `start`, `key`, `resize`, `snapshot`만 지원한다.
  3. key step은 InputMapper를 거치게 한다.
  4. 알 수 없는 action/field는 파일명과 step 번호가 포함된 오류를 낸다.
  5. startup, navigation, selection, resize 시나리오를 추가한다.
- 현재 한계: clock은 schema/result에 보존되지만 app state/render에 주입되지 않는다.
  disk.free_bytes는 즉시 `DiskInfoLoaded` completion에 사용한다. named effect completion과
  단계별 assertion step은 M1-13에서 추가한다.
- 검증: M1 전체 게이트와 10,000항목 성능 smoke test.
- 완료: 현재 구현된 최소 시나리오 schema와 M1 기본 탐색 흐름을 자동 검증.

## M1-13 M1 계약 정합성 보정과 종료

- 선행: M0-05, M1-01~M1-12
- 목표: 문서가 요구하지만 현재 64-test 기준선이 아직 증명하지 못하는 M1 계약을 닫는다.
- 파일: `src/model/*`, filesystem adapters, `src/layout/text.rs`, `src/ui.rs`, `src/ui/*`,
  command registry, scenario harness/snapshots, performance tests
- 작업:
  1. `EntryMetadata`/`FileEntry`에 `Option<SystemTime>` modified와
     `EntryAttributes { read_only, hidden, system, archive }`를 추가한다. directory-load
     worker의 `TimeZonePort`가 timestamp 시점 OS local time을 `LocalMinute`로 바꾸며,
     render는 `MM-DD HH:mm` 또는 unavailable `----- --:--`만 그린다. metadata/timezone
     변환 실패는 해당 entry fallback이고 listing 전체는 유지한다.
  2. `ui.rs`의 grapheme/cell-width helper를 `layout/text.rs`로 이동하고 Viewer/Editor가
     재사용할 공개 경계를 만든다.
  3. 기능키 바를 CommandRegistry에서만 생성하고 disabled command를 별도 style과 이유로
     표시한다. 기본 힌트도 실제 계약인 `Ctrl+Q Quit`와 일치시킨다.
  4. 현재 YAML v1 schema의 clock을 `FixedClock`에 실제 주입하고, fixture의 optional
     RFC3339 `modified`, `attributes`, `metadata_error`와 top-level
     `timezone_offset_minutes`를 `FixedTimeZone`에 연결한다. 이미 동작하는 disk completion은
     유지하고 named effect completion과 단계별 assertion을 최소 지원한다. clock을 modified
     값이나 timezone으로 암묵 사용하지 않으며 unsupported future step/field는 거부한다.
  5. 문자와 Style을 함께 직렬화하는 snapshot helper를 만들고 80/100/120/160열 matrix를
     채운다.
  6. 실제 sort+layout+render와 key→reduce→render를 측정하는 이름 있는 release performance
     smoke test를 추가한다. 임계치는 환경 변동을 고려해 ignored/manual gate로 유지한다.
  7. built-in label/Help/message/error가 영어이고 Unicode 파일명/경로는 원문 그대로인지
     현재 M1 화면과 scenario에서 고정한다.
- 필수 회귀: long path, render port call-count, modified known/missing/error, DST가 다른 두
  timestamp와 fixed-offset snapshot, R/H/S/A mapping, disabled/enabled F-key, Ctrl+Q hint,
  Unicode width helper, FixedClock/FixedTimeZone 분리, 기존 disk completion, named effect
  completion, bad scenario step diagnostic, style diff, 10,000 entries, built-in English copy와
  Unicode 사용자 경로 보존.
- 완료: 이 카드가 소유한 자동 항목(ARC-02, UI-03/07/08의 M1 부분, KEY-02/04, THEME-01 자동 부분,
  TEST-03/04/06, PERF-01/02)의 실제 test 이름/상태가 `progress.md` ledger에 연결되고
  미증명 자동 항목이 0개이며 공통 게이트가 모두 성공함. TEST-07 외부 CI와 THEME-01/FS-01/02
  Linux/macOS 수동 부분은 각각 R1-01/R1-02 owner로 ledger에 남기며 M2 진입을 막지 않는다.

---

# M2 — 파일 관리

## M2-01 공통 대화상자 상태와 렌더러

- 선행: M1-13
- 목표: 입력/확인/오류/진행 대화상자가 같은 규칙을 사용한다.
- 파일: `src/app/state.rs`, `src/ui/dialogs/{mod,input,confirm,progress}.rs`
- 작업: focus, text input, Enter confirm, Esc cancel, modal key capture, 중앙 배치 구현.
- 테스트: 너무 긴 경로/Unicode 입력/작은 화면/취소 시 상태 불변.
- 스냅샷: input, confirm, error, progress.
- 완료: dialog 중 main command가 실행되지 않음.

## M2-02 파일 I/O·mutation 포트와 결정적 Fake

- 선행: M1-13
- 목표: Viewer/Editor/파일 작업이 OS API를 직접 호출하지 않는 공통 기능 경계를 만든다.
- 파일: `src/ports/filesystem.rs`, `src/adapters/{memory_fs,real_fs}.rs`, contract tests
- 작업:
  1. 기존 read_dir/metadata에 read stream/bytes, create_dir, rename, temp writer,
     atomic publish, remove, copy metadata와 symlink metadata capability를 분리해 추가한다.
  2. mutation capability는 작은 operation 단위로 두고 재귀/충돌 정책은 planner가 소유한다.
  3. MemoryFileSystem에 n번째 read/write 실패, disk full, permission denied,
     cross-device rename, short write, slow I/O fault를 추가한다.
  4. Real/Memory adapter에 같은 contract suite를 적용하되 실제 쓰기는 TempDir 하위만 허용한다.
  5. UI/reducer/model이 `std::fs`를 직접 참조하지 않는 dependency test를 추가한다.
- 완료: 뒤 카드가 새 OS 호출 없이 포트와 Fake만으로 red/green 테스트를 작성할 수 있음.

## M2-03 Rename과 MkDir

- 선행: M2-01, M2-02
- 목표: F2/F7 작업을 validation→effect→result 흐름으로 구현한다.
- 파일: `src/app/reducer.rs`, `src/adapters/real_fs.rs`, 관련 dialogs
- 작업:
  1. 기존 이름 prefill과 전체 선택.
  2. 빈 이름, `.`, `..`, Windows 금지 문자/예약명/후행 점·공백 거부.
  3. 동일 경로, 기존 대상 충돌을 명시.
  4. 성공 후 refresh하고 새 EntryId에 커서를 둔다.
- 테스트: `CON`, `NUL`, `a:b`, 한글 이름, case-only rename, 권한 오류.
- 완료: MemoryFS와 platform tempdir integration test 성공.

## M2-04 Viewer 로드·decode 모델

- 선행: M2-02
- 목표: 렌더와 분리된 읽기 전용 문서와 비동기 load effect를 만든다.
- 파일: `src/model/viewer.rs`, `src/app/state.rs`, reducer/effects, filesystem port
- 작업:
  1. UTF-8/BOM 판별, binary NUL 탐지, 32 MiB 상한.
  2. loading/ready/binary/too-large/error/cancel 상태를 명시한다.
  3. line index와 search result를 OS/terminal 없는 순수 모델로 만든다.
  4. stale load result를 generation으로 폐기한다.
- 테스트: empty, CRLF/LF, BOM, invalid UTF-8, binary, too-large, stale/error.
- 완료: viewer는 원본 파일을 쓰지 않음.

## M2-05 Viewer UI와 검색

- 선행: M2-01, M2-04
- 목표: F3에서 문서를 키보드로 탐색하고 검색한다.
- 파일: `src/ui/dialogs/viewer.rs`, reducer/input/scenarios
- 작업:
  1. line viewport, Up/Down/PgUp/PgDn/Home/End와 Esc를 구현한다.
  2. Ctrl+F 문자열 검색과 다음/이전 결과를 구현한다.
  3. M1-13 text helper로 탭, 넓은 Unicode, 긴 줄을 정확히 렌더한다.
  4. loading/error/binary/too-large 상태와 취소 후 원래 선택 복원을 표시한다.
- 스냅샷: empty/loading/text/unicode/search/error/binary/too-large/too-small.
- 완료: UI thread와 render에서 파일 read가 0회임.

## M2-06 EditorBuffer

- 선행: M1-13
- 목표: 렌더와 무관한 편집 버퍼를 완성한다.
- 파일: `src/model/editor.rs`, `tests/editor_buffer.rs`
- 작업:
  1. line/grapheme cursor와 selection 없는 insert/delete/newline.
  2. whole-buffer Undo/Redo 100단계.
  3. dirty flag, 원본 modified timestamp, 5 MiB 제한.
  4. 문자열 search와 결과 이동.
- 테스트: 한글/emoji/결합문자, 줄 시작/끝, 빈 문서, undo branch.
- 완료: byte offset 중간을 절대 자르지 않음.

## M2-07 Editor UI와 안전 저장

- 선행: M2-01, M2-02, M2-06
- 목표: F4 Edit/Save/Save As를 완성한다.
- 파일: `src/ui/dialogs/editor.rs`, `src/app/reducer.rs`, `src/adapters/real_fs.rs`
- 작업:
  1. 줄 번호, viewport, cursor, status, 도움말을 렌더한다.
  2. Ctrl+S, Ctrl+Shift+S, Ctrl+Z/Y, Ctrl+F, Esc를 매핑한다.
  3. temp write + flush + replace로 저장한다.
  4. 저장 직전 외부 수정 시각이 다르면 overwrite 확인.
  5. dirty 상태 Esc는 Save/Discard/Cancel 확인.
- 테스트: 저장 실패 시 원본 유지, Save As 충돌, 외부 변경.
- 완료: 실패 경로에서도 dirty buffer를 잃지 않음.

## M2-08 작업 계획과 충돌 모델

- 선행: M2-01, M2-02
- 목표: Copy/Move/Delete 실행 전에 전체 작업과 위험을 검증한다.
- 파일: `src/model/operation.rs`, `src/operations/planner.rs`
- 작업:
  1. ADR-005 공통 `OperationId`를 `src/runtime/job.rs`에 정의하고 item plan, total bytes,
     conflict, decision enum이 이를 사용한다.
  2. 동일 경로, 대상이 원본 하위, 중복 대상, symlink를 거부/표시.
  3. `Overwrite`, `OverwriteAll`, `Skip`, `SkipAll`, `Rename`, `Cancel` 여섯 상태 전이를
     구현하고 All 범위를 현재 OperationId로 제한한다.
  4. rename suggestion은 `name (1).ext`부터 충돌 없이 증가.
- 테스트: 파일/폴더 혼합, 중첩 선택, case-only path, 충돌 all 범위.
- 완료: planner는 실제 쓰기를 하지 않음.

## M2-09 Worker와 진행률 프로토콜

- 선행: M2-08
- 목표: M1의 worker 프로토콜을 확장해 장시간 작업 중 UI 입력과 진행률을 유지한다.
- 파일: `src/operations/worker.rs`, `src/runtime.rs`, `src/app/reducer.rs`
- 작업:
  1. 공통 `CancelHandle/CancelToken`, monotonic Deadline, JobControl과 기본 capacity 16의
     bounded non-blocking Core Local sender, non-blocking `MutationCoordinator/MutationLease`를
     `src/runtime/{job,lane}.rs`에 구현한다.
     기존 worker command/event에 mutation job과 안전한 종료 join을 추가한다.
  2. 진행률은 최대 20 Hz로 coalesce한다.
  3. conflict request/response와 cancel token을 구현한다.
  4. OperationFinished 후 refresh effect를 보낸다.
- 테스트: capacity 1에서 refresh 최신값 coalesce, full user submit/active mutation Busy와
  UI `try_send` 비차단, 가짜 느린 FS에서 tick/키 처리, 취소, worker panic 변환,
  cancel/deadline/error/panic 뒤 terminal exactly once와 join.
- 완료: UI thread에서 copy/read loop가 실행되지 않음.

## M2-10 Copy

- 선행: M2-08, M2-09
- 목표: F5 파일/디렉터리 복사를 구현한다.
- 파일: `src/operations/copy.rs`, copy dialog, integration tests
- 작업:
  1. 대상 경로 입력과 최근 경로 초기값.
  2. 디렉터리 생성 후 재귀 복사, symlink 미추적.
  3. chunk 단위 byte progress와 파일 count.
  4. 취소 시 진행 중 임시 파일 제거, 완료 파일은 결과에 보고.
  5. 충돌 결정을 planner 규칙대로 적용.
- 테스트: 0 byte, 큰 가짜 파일, nested, 충돌 6종, permission/disk-full 모의.
- 완료: 원본과 복사본 내용/메타데이터 핵심값 검증.

## M2-11 Move

- 선행: M2-10
- 목표: F6 이동을 같은 볼륨 rename과 cross-volume copy+delete로 처리한다.
- 파일: `src/operations/move_entry.rs`, move dialog, tests
- 작업:
  1. 먼저 atomic rename을 시도한다.
  2. cross-device 오류에서 Copy 성공 후 원본 삭제.
  3. 원본 삭제 실패 시 “복사됨/원본 남음” 부분 성공으로 보고.
  4. 선택/커서를 결과 디렉터리 정책에 맞게 refresh.
- 테스트: rename success, cross-device mock, copy fail, delete fail, cancel.
- 완료: 실패했는데 원본과 대상이 모두 사라지는 경로가 없음.

## M2-12 Delete와 휴지통

- 선행: M2-08, M2-09
- 목표: F8 휴지통, Shift+F8 영구 삭제를 안전하게 구현한다.
- 파일: `src/ports/trash.rs`, `src/adapters/system_trash.rs`,
  `src/operations/delete.rs`, delete dialog
- 작업:
  1. 기본 확인창은 항목/바이트 합계를 표시한다.
  2. F8은 Trash port, Shift+F8은 경고 강화 후 recursive delete.
  3. root, drive root, current directory, `..` 삭제를 거부한다.
  4. symlink는 링크 자체만 처리하고 따라가지 않는다.
  5. 항목별 실패를 결과에 보존한다.
- 테스트: RecordingTrash, tempdir permanent delete, protected targets.
- 완료: 실제 홈/작업공간을 대상으로 하는 테스트가 없음.

## M2-13 정렬, 숨김 파일, 드라이브

- 선행: M1-13, M2-01, M2-02
- 목표: 메뉴가 없어도 Registry의 S/Ctrl+S/H/D로 정렬/숨김/드라이브 이동을 쓸 수 있다.
- 파일: `src/model/directory.rs`, `src/ports/disk.rs`,
  `src/adapters/windows_disk.rs`, input/reducer
- 작업:
  1. Main `SortKeyNext(S)`, `SortDirectionToggle(Ctrl+S)`, `ToggleHidden(H)`,
     `OpenDrivePicker(D)` CommandId와 Help label을 등록한다. Editor의 Ctrl+S는 screen context로
     분리한다.
  2. 기본 Name/Ascending/DirectoriesFirst/show_hidden=true와 제품 계약의 Extension/Size/
     Date/Time, missing-last, stable tie-break를 구현한다.
  3. Directories First를 독립 설정으로 유지하고 Windows Hidden attribute를 필터한다.
  4. logical drive roots/free space adapter와 Up/Down/Enter/Esc picker를 구현한다.
  5. 정렬/필터/drive 뒤 현재 EntryId 유지 또는 이전 visual index의 가장 가까운 항목을 선택한다.
- 테스트: command/context/Help, dotfile/trailing-dot/mixed-case tie, Asc/Desc, unknown size/time,
  dir/file group, hidden default/toggle, selected-hidden fallback, empty/error/drive picker.
- 완료: M2 Help가 네 command를 Registry에서 표시하고 comparator table의 미정 행이 없음.

## M2-14 M2 통합 수용

- 선행: M2-03~M2-13
- 목표: 파일 관리 기능을 시나리오와 임시 디렉터리에서 종단 검증한다.
- 작업:
  1. 모든 dialog snapshot 추가.
  2. rename→edit→copy→move→trash 흐름 시나리오 작성.
  3. 각 작업의 권한 오류/충돌/취소 시나리오 작성.
  4. worker 중 resize/input 시나리오 작성.
  5. Linux/macOS integration test는 임시 폴더만 사용.
  6. M2 built-in dialog/progress/error는 영어이고 Unicode 파일명/사용자 입력은 보존되는지
     snapshot과 state에서 검증한다.
- 완료: M2 acceptance 항목과 공통 게이트 전부 성공.

---

# M3 — Mdir 확장

## M3-01 TOML 설정 로드/저장/복구

- 선행: M2 완료
- 목표: versioned config core와 현재 존재하는 설정만 안전하게 복원한다.
- 파일: `src/config/*`, `tests/config_roundtrip.rs`
- 작업:
  1. `version = 1` schema와 Default를 정의한다.
  2. 마지막 경로, view, column, sort, hidden, theme만 core fragment에 저장한다.
  3. 모르는 필드는 허용하고 잘못된 알려진 값은 경고한다.
  4. atomic save와 broken file 보존을 구현한다.
  5. 존재하지 않는 마지막 경로는 사용자 홈→현재 경로 순으로 fallback.
- 경계: 아직 모델이 없는 keymap/QCD/MCD history를 미리 정의하지 않는다. 각 기능 카드가
  자기 config fragment, default, migration, roundtrip test를 함께 추가한다.
- 테스트: roundtrip, partial config, unknown field, corrupt TOML, unwritable dir.
- 완료: 설정 I/O 실패가 앱 시작/종료를 막지 않음.

## M3-02 사용자 키맵

- 선행: M1-10, M3-01
- 목표: CommandRegistry를 TOML override로 재구성한다.
- 파일: `src/config/schema.rs`, `src/app/command_registry.rs`, keymap parser
- 작업:
  1. command id와 KeyChord 문자열 schema를 정의한다.
  2. 중복 키, 알 수 없는 command, 필수 Esc/confirm 충돌을 진단한다.
  3. 오류가 있으면 해당 항목만 기본값으로 fallback.
  4. 기능키 바와 Help가 재구성 결과를 즉시 사용한다.
  5. keymap config fragment와 version migration/roundtrip을 함께 추가한다.
- 테스트: valid override, duplicate, invalid chord, modal precedence.
- 완료: 표시와 실제 mapping 불일치 검출 테스트 성공.

## M3-03 Long View와 Tab 전환

- 선행: M1-09, M3-01
- 목표: 단일 컬럼 상세 표와 Short View 상태 저장을 구현한다.
- 파일: `src/ui/components/long_view.rs`, reducer/input/config
- 작업:
  1. 동적 column width와 헤더를 그린다.
  2. 이름에 남은 폭을 주고 숫자 열은 우측 정렬한다.
  3. Tab 전환 시 EntryId와 페이지 내 가시성을 유지한다.
  4. 80열에서 열 우선순위에 따라 Attr→Time→Date 순으로 숨긴다.
  5. Long/Short 선택 config fragment와 roundtrip을 추가한다.
- 스냅샷: 80/100/120열, unicode, missing time.
- 완료: Short/Long 왕복 후 동일 항목 선택.

## M3-04 외부 테마

- 선행: M1-08, M3-01
- 목표: 재빌드 없이 TOML 테마를 추가/선택한다.
- 파일: `src/theme/schema.rs`, theme loader, F12 options 연결 준비
- 작업:
  1. 모든 필수 token schema와 16색 이름 parser를 만든다.
  2. 누락 token은 기반 테마에서 상속한다.
  3. 잘못된 테마는 무시하고 오류 메시지를 표시한다.
  4. built-in Classic/DOS Blue/Dark/Mono/Light를 제공한다.
- 테스트: inheritance, bad color, missing name, all cursor/marked contrast.
- 완료: 모든 built-in theme 전체 화면 snapshot 최소 1개.

## M3-05 MCD 트리 모델

- 선행: M3-01
- 목표: 렌더 없이 지연 로드/접기/선택/검색 가능한 트리를 만든다.
- 파일: `src/mcd/tree.rs`, `src/mcd/visible_rows.rs`, tests
- 작업:
  1. node id, path, depth, expansion/loading/error 상태를 정의한다.
  2. 현재 경로 조상 chain을 확장한다.
  3. visible row flatten을 반복문으로 구현해 깊은 트리 stack overflow를 피한다.
  4. Up/Down/Left/Right 규칙과 loaded/history filter를 구현한다.
  5. tree connector segment를 계산한다.
  6. history fragment와 migration/roundtrip은 이 카드에서 처음 추가한다.
- 테스트: empty drive, deep 1000, denied child, collapse, filter, Unicode.
- 완료: 같은 path node 중복 없음.

## M3-06 MCD UI와 비동기 로드

- 선행: M3-05
- 목표: F10 전체 화면 MCD를 참고 이미지 스타일로 구현한다.
- 파일: `src/ui/components/mcd_tree.rs`, reducer/effects/adapters
- 작업:
  1. 파랑 배경, 제목, tree connector, cursor, 현재 경로, key bar를 렌더한다.
  2. 자식 load와 F2 rescan을 worker에서 실행한다.
  3. F3 drive picker와 Enter navigate를 연결한다.
  4. Esc는 원래 경로/선택을 변경하지 않는다.
- 스냅샷: reference-like tree, loading, error, search, 80×25.
- 완료: MCD에서 main list state가 손상되지 않음.

## M3-07 QCD

- 선행: M3-01, M2-01
- 목표: F11 즐겨찾기 디렉터리를 관리한다.
- 파일: `src/ui/dialogs/qcd.rs`, config/reducer/input
- 작업:
  1. label/path/position 모델과 최대 100개 제한.
  2. Enter/Insert/F2/Delete/Esc/숫자 1~9 구현.
  3. 중복 path는 기존 항목 편집 제안.
  4. 존재하지 않거나 접근 거부된 경로는 이동하지 않고 유지.
  5. QCD config fragment와 migration/roundtrip을 이 카드에서 함께 추가한다.
- 테스트: CRUD, reorder 안정성, 숫자 키, invalid UNC.
- 완료: 재시작 roundtrip 성공.

## M3-08 F12 Menu와 Help 완성

- 선행: M3-02~M3-07
- 목표: 모든 v1 기능을 메뉴와 Help에서 발견할 수 있다.
- 파일: `src/ui/dialogs/menu.rs`, `src/ui/components/help.rs`, registry
- 작업:
  1. File/View/Directory/Tools/Options/Quit 계층을 만든다.
  2. 메뉴 항목은 직접 로직을 호출하지 않고 기존 command action을 dispatch한다.
  3. 비활성 기능은 이유와 함께 disabled 표시.
  4. Help는 현재 screen과 keymap을 기준으로 생성한다.
- 테스트: 모든 menu leaf에 command id 존재, Esc/Left/Right/Enter 탐색.
- 스냅샷: root/submenu/disabled/custom keymap help.
- 완료: 중복 구현된 기능 로직 없음.

## M3-09 설정 화면

- 선행: M3-01~M3-04, M3-08
- 목표: 파일 직접 편집 없이 핵심 설정을 변경한다.
- 파일: settings dialog, reducer/config
- 작업:
  1. theme, view, column count/width, sort, hidden, keymap 선택.
  2. 변경 전 preview와 Apply/Cancel.
  3. 잘못된 custom width는 UI 단계에서 제한.
  4. Apply 성공 시 config save effect, 실패 시 상태는 유지하고 경고.
- 테스트: cancel rollback, apply, write failure, resize during dialog.
- 완료: 주요 설정 전체가 키보드만으로 변경 가능.

## M3-10 M3 통합 수용

- 선행: M3-01~M3-09
- 목표: 재시작을 포함한 v1 기능 흐름을 검증한다.
- 작업:
  1. MCD→directory→QCD add→restart→QCD navigate 시나리오.
  2. custom theme/keymap load와 fallback 시나리오.
  3. Short/Long/resize/selection 유지 시나리오.
  4. F12 모든 메뉴 leaf smoke test.
  5. M3 menu/settings/MCD/QCD built-in copy는 영어이고 사용자 저장 이름/경로는 보존되는지
     검증한다.
- 완료: acceptance matrix의 M3 항목과 공통 게이트 성공.

---

# R1 — Linux/macOS v1.0 릴리스

## R1-01 릴리스 후보 빌드와 패키징

- 선행: M3 완료
- 목표: 수동 시험에 사용할 불변 release candidate artifact를 만든다.
- 작업:
  1. 공통 게이트와 Linux/macOS build를 성공시킨다.
  2. `cargo build --release --locked`로 플랫폼별 단일 `mdir4`를 만든다.
  3. 기본 설정/테마를 내장하고 필수 문서와 license 목록을 묶는다.
  4. [`../releases/v1.0-rc-template.md`](../releases/v1.0-rc-template.md)를 복사해
     `docs/releases/v1.0-rc.md`를 만들고 source commit, `rustc -Vv`, `cargo -V`, target triple,
     Cargo.lock SHA-256, artifact SHA-256과 build 환경을 기록한다.
  5. clean Linux/macOS 환경에서 artifact만으로 시작되는지 smoke 확인한다.
- 원격 CI는 선택 사항이다. local-only release이면 build 명령과 환경을 증거로 기록한다.
- 완료: 이후 수동 시험 대상의 hash가 고정되고 개발 머신 절대 경로가 노출되지 않음.

## R1-02 고정 RC Linux/macOS 실제 환경 수동 시험

- 선행: R1-01
- 목표: R1-01에서 고정한 플랫폼별 SHA-256 artifact를 Linux/macOS에서 확인한다.
- 체크:
  - Linux 일반 터미널과 macOS Terminal/iTerm 계열
  - 시작/종료/패닉 복구
  - drive/UNC/긴 경로/한글 경로
  - 연결 프로그램 실행
  - 휴지통과 영구 삭제
  - 리사이즈, Alt+Tab 복귀, 빠른 키 반복
  - 10,000개 디렉터리와 1 GiB 복사 진행률
- 완료: `docs/releases/v1.0-rc.md`의 같은 artifact hash 행에 OS/terminal/결과를 기록하고
  screenshot은 `docs/releases/assets/v1.0-rc/` 상대 경로로 연결.

## R1-03 v1.0 최종 게이트

- 선행: R1-02
- 목표: 문서, 자동 테스트, 수동 시험, 패키지를 한 번에 승인한다.
- 작업:
  1. 공통 게이트와 Linux/macOS release build 재실행.
  2. `.snap.new`, ignored failing test, TODO acceptance가 없는지 검사.
  3. acceptance matrix 모든 v1 ID를 `progress.md` evidence ledger와
     `docs/releases/v1.0-rc.md`의
     테스트/수동 증거에 연결한다.
  4. 알려진 제한을 README와 release note에 기록.
- 금지: R1-02 이후 코드가 바뀌면 기존 수동 증거를 재사용하지 말고 R1-01부터 새 RC로 반복한다.
- 완료: `docs/releases/v1.0-rc.md`의 Final approval가 승인자/날짜/source+artifact hash와 함께
  채워지고 release note 제한 사항이 README와 일치함.
- 완료: `progress.md`에서 v1 범위의 `M0~R1` 카드가 모두 완료되고 v1 미해결 blocker가 0개다.
  `G0~G3`와 `S0~S3`는 R1 이후 독립 트랙이므로 이 조건에 포함하지 않는다.

---

# v1 이후 — Git built-in

Git 확장은 현재 v1 카드에 포함하지 않는다. `R1-03` 완료 후
[`../plugins/git/03-task-cards.md`](../plugins/git/03-task-cards.md)의 `G0`부터 시작한다.
Git 제품 계약, 테스트, 수용 기준도 같은 폴더의 문서를 우선하며 범용 외부 plugin은
해당 후속 계획에도 포함하지 않는다.

# v1 이후 — SSH Remote / Remote Drive

SSH/SFTP는 현재 v1 카드에 포함하지 않는다. `R1-03` 완료 후 제품 범위는 Git과 분리해
[`../remote/03-task-cards.md`](../remote/03-task-cards.md)의 `S0`부터 시작할
수 있다. 둘 다 구현할 때는 공용 path/config/runtime 통합 선행 규칙을 먼저 확인한다.
Remote 제품 계약, 보안, fake/real backend, 수용 기준은 같은 폴더 문서를 우선한다.
