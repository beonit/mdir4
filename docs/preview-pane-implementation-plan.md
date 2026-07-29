# Adaptive Preview pane 구현 순서

이 문서는 [ADR-007](architecture/adr-007-adaptive-preview-pane.md)의 설계를 현재 source tree에
적용하는 실행 순서다. 구현 상태를 주장하는 문서가 아니며, 모든 카드는 현재 `Proposed`다.
기존 `R1-02` 활성 작업을 자동으로 선점하지 않는다. Preview 구현을 시작하기로 결정하면
아래 순서와 완료 조건을 따른다.

## 1. 범위 요약

- Settings에서 Preview on/off
- 기본 On, 기본 너비 50%
- `Alt+[`/`Alt+]` 또는 custom keymap으로 너비 조절
- 120 columns 이상에서만 Main/Remote browser 오른쪽에 표시
- UTF-8 텍스트만 표시, 1 MiB 상한
- Git Modified 텍스트 파일은 unified diff 우선
- Remote를 blanket disable하지 않고 SFTP read와 가능한 Remote Git capability 사용

## 2. 선행 관계

```text
P0 계약 고정
  -> P1 workspace layout
      -> P2 config/state/input/skeleton
          -> P3 Local text Preview
              -> P4 Local Git diff
              -> P5 Remote text Preview
                  -> P6 Remote Git diff
                      -> P7 통합 검증과 문서 종료
```

P4와 P5는 P3 뒤에 병렬 진행할 수 있다. P6은 P4의 Git Preview 의미와 P5의 Remote bounded
read/lane 계약이 모두 완료되어야 시작한다.

## 3. 공통 구현 규칙

- 각 카드는 기재된 테스트가 먼저 실패하는 것을 확인한 뒤 production 코드를 작성한다.
- reducer와 render는 filesystem, Git, SSH 또는 subprocess를 직접 호출하지 않는다.
- render와 navigation은 같은 `WorkspaceLayout.browser`를 사용한다.
- completion은 generation과 target identity가 모두 일치할 때만 적용한다.
- Preview 실패는 directory navigation과 다른 full-screen mode를 막지 않는다.
- Remote path는 display string으로 round-trip하지 않는다.
- 기존 사용자 변경과 관련 없는 파일은 수정하지 않는다.

모든 카드의 기본 품질 게이트:

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
git diff --check
```

UI 카드에서는 관련 snapshot diff를 사람이 검토하고 `.snap.new`를 남기지 않는다.

## 4. 작업 카드

### P0 Preview 계약과 기준선 고정

- 선행: ADR-007 승인
- 목표: 구현 중 재해석하지 않도록 threshold, width, text, Git과 Remote fallback을 고정한다.
- 파일:
  - `docs/architecture/adr-007-adaptive-preview-pane.md`
  - `docs/preview-pane-implementation-plan.md`
- 작업:
  1. 기본 On, 기본 50%, 35~65%, 5% step과 120-column threshold를 승인한다.
  2. Preview는 focus/scroll/edit가 없는 read-only pane임을 승인한다.
  3. Git 범위가 Modified만인지 확인한다.
  4. Remote Git unavailable 시 text fallback임을 확인한다.
  5. 구현 시작 시 `progress.md`에 Preview 트랙과 첫 카드 P1을 기록한다.
- 검증: 문서 링크와 요구사항/비목표/재검토 조건 검토.
- 완료: 미결정 항목 없이 P1 테스트를 작성할 수 있음.

### P1 WorkspaceLayout과 적응형 split

- 선행: P0
- 목표: content나 I/O 없이 browser/Preview geometry를 단일 원본으로 만든다.
- 주 파일:
  - `src/layout.rs`
  - `src/layout/navigation.rs`
  - `src/app.rs`
  - `src/ui.rs`
  - `tests/navigation.rs`
  - `tests/scenarios/resize.yml`
- 작업:
  1. `PreviewLayoutSettings`와 `WorkspaceLayout`을 추가한다.
  2. layout 계산을 origin-aware `Rect` 기반으로 일반화하고 기존 Viewport wrapper를 유지한다.
  3. 120 미만은 전체 browser, 120 이상은 최소 60/60 split을 계산한다.
  4. Preview width를 35~65%로 clamp한다.
  5. Main과 Remote navigation이 `WorkspaceLayout.browser` metrics만 사용하게 한다.
  6. UI에는 border와 placeholder만 그리되 아직 file read를 추가하지 않는다.
- 테스트:
  - width 119/120/121
  - ratio 0/35/50/65/100 입력 clamp
  - 두 rect가 viewport를 gap/overlap 없이 보존
  - Preview Off가 기존 metrics와 완전히 동일
  - split 이후 arrow/page/home/end가 표시된 browser cell과 일치
  - resize 전후 선택 entry identity 유지
- 완료: Preview Off snapshot이 불필요하게 바뀌지 않고 layout/navigation 단위 테스트 통과.

### P2 Config, PreviewState, Settings와 width command

- 선행: P1
- 목표: 설정, runtime state와 키 입력을 연결하고 I/O 없는 Preview skeleton을 완성한다.
- 주 파일:
  - `src/config/schema.rs`
  - `src/config/mod.rs`
  - `src/app.rs`
  - `src/app/command_registry.rs`
  - `src/input/mapper.rs`
  - `src/ui.rs`
  - `tests/config_roundtrip.rs`
  - `tests/input_mapping.rs`
- 작업:
  1. `PreviewConfig { enabled, width_percent }`와 serde default를 추가한다.
  2. `SettingsDraft`와 Settings 화면에 Preview On/Off를 추가한다.
  3. `PreviewState`, `PreviewTarget`, `PreviewContent`를 모델 모듈에 추가한다.
  4. startup config와 `config_from_state`에 Preview 설정을 연결한다.
  5. `PreviewWidthDecrease/Increase` CommandId와 Action을 추가한다.
  6. 기본 `Alt+[`/`Alt+]`를 Main/Remote screen mapping보다 먼저 처리한다.
  7. hidden 상태에서 width 변경 시 다음 표시 비율 message를 제공한다.
- 테스트:
  - 구 config load 시 Off/50
  - enabled/width TOML roundtrip
  - 잘못된 width clamp
  - Settings Apply/Cancel
  - Main/Remote 기본키와 custom keymap
  - full-screen Viewer/Editor/Git에서는 width command 비활성
  - resize hide/show가 enabled와 width를 변경하지 않음
- 완료: 재시작 후 설정이 복원되고 Preview skeleton의 표시 조건이 계약과 일치.

### P3 Local 텍스트 Preview와 stale-result 방어

- 선행: P2
- 목표: 선택한 Local UTF-8 text를 비동기로 읽고 오른쪽 pane에 렌더한다.
- 주 파일:
  - `src/model/preview.rs` 신규
  - `src/model/viewer.rs`
  - `src/app.rs`
  - `src/runtime.rs`
  - `src/ui.rs`
  - `src/adapters/memory_fs.rs`
  - `tests/scenarios/selection.yml`
- 작업:
  1. `PREVIEW_MAX_BYTES = 1 MiB`와 Local Preview Effect/completion을 추가한다.
  2. public reducer wrapper 뒤 `reconcile_preview`를 추가한다.
  3. target 변경 시 generation을 증가시키고 pending read를 최신 target으로 coalesce한다.
  4. `FileSystem::read_file`을 bounded read 경로로 사용한다.
  5. `ViewerState::decode`로 UTF-8/BOM/NUL 판정을 공유한다.
  6. Text/Loading/Binary/TooLarge/Error/Unsupported header와 body를 렌더한다.
  7. directory/parent/symlink/other는 read Effect를 만들지 않는다.
- 테스트:
  - plain UTF-8, BOM, empty file
  - NUL, invalid UTF-8, 정확히 1 MiB, 1 MiB + 1
  - permission/not-found 오류
  - 빠른 A->B 선택 후 A completion 무시
  - directory reload와 같은 path의 새 generation
  - narrow hidden 상태에서는 새 Preview read를 만들지 않음
  - 다시 넓어졌을 때 같은 target cache 재사용 또는 정확히 한 번 load
- 완료: Local navigation을 막지 않고 모든 text/non-text 상태가 결정적으로 렌더됨.

### P4 Local Git Modified diff 우선 표시

- 선행: P3
- 목표: Local Git Modified 텍스트 파일의 기본 content를 unified diff로 바꾼다.
- 주 파일:
  - `src/app.rs`
  - `src/runtime.rs`
  - `src/plugins/git/model.rs`
  - `src/plugins/git/real_backend.rs`
  - `src/ui.rs`
  - `src/plugins/git/fake_read_backend.rs`
  - `tests/git_fake_read_backend.rs`
  - `tests/git_read_integration.rs`
- 작업:
  1. text decode 성공 뒤 current Git cache가 Modified이면 diff Effect를 생성한다.
  2. 기존 `LoadGitDiffForPath` backend를 Preview identity와 generation을 보존하는 completion으로
     재사용한다.
  3. full-screen/Preview가 공유하는 unified diff line/style helper를 추출한다.
  4. non-empty diff는 `PreviewContent::Diff`로 표시한다.
  5. empty/error/stale diff는 cached Text로 fallback하며 navigation을 막지 않는다.
  6. full-screen GitDiff의 origin/search/side-by-side state와 Preview state를 분리한다.
- 테스트:
  - Modified text -> DIFF
  - Clean text -> TEXT
  - Modified binary/TooLarge -> diff Effect 0회
  - Git status가 text load 뒤 도착해도 정확히 한 번 diff 전환
  - empty/error diff -> cached text
  - Preview stale diff completion 무시
  - full-screen Git diff 동작과 snapshot 불변
- 완료: Local Modified 텍스트의 기본 Preview가 diff이고 기존 Git 화면 회귀 없음.

### P5 Remote bounded file read와 텍스트 Preview

- 선행: P3
- 목표: Remote를 비활성화하지 않고 SFTP에서 선택 text를 Preview한다.
- 주 파일:
  - `src/remote/backend.rs`
  - `src/remote/sftp.rs`
  - `src/remote/fake.rs`
  - `src/remote/lane.rs`
  - `src/app.rs`
  - `src/runtime.rs`
  - `src/ui.rs`
  - `tests/directory_loading.rs` 또는 Remote 전용 contract test 신규
- 작업:
  1. `RemoteReadBackend::read_file(path, max_bytes)`를 추가한다.
  2. SFTP `OPEN/READ/CLOSE` packet과 bounded `limit + 1` 판정을 구현한다.
  3. success/error/TooLarge/cancel/deadline에서 remote handle과 child를 정리한다.
  4. `RemoteView` 선택 identity로 `LoadRemotePreview` Effect를 만든다.
  5. 해당 SSH Location lane에서 실행하고 alias/path/generation completion을 검증한다.
  6. Remote text도 Local과 같은 `ViewerState::decode` 및 renderer를 사용한다.
  7. invalid UTF-8 remote path bytes를 display string으로 재구성하지 않는다.
- 테스트:
  - Fake Remote UTF-8/BOM/binary/TooLarge/error
  - byte-preserving path request identity
  - SFTP packet partial read와 EOF
  - stale session/path/generation completion 무시
  - 한 Remote preview timeout이 Local navigation을 막지 않음
  - Remote width split과 navigation geometry
- 완료: Git 유무와 관계없이 Remote text Preview가 Local과 같은 규칙으로 동작.

### P6 Remote Git Modified diff 우선 표시

- 선행: P4, P5
- 목표: capability가 있는 Remote worktree에서 Modified text를 diff로 표시한다.
- 주 파일:
  - `src/plugins/git/model.rs`
  - `src/plugins/git/real_backend.rs` 또는 Remote Git read adapter 신규
  - `src/plugins/git/state.rs`
  - `src/remote/lane.rs`
  - `src/app.rs`
  - `src/runtime.rs`
  - Git/Remote integration test 신규
- 작업:
  1. Remote directory의 repository/status cache identity를 alias + RemotePath로 정의한다.
  2. Remote Git read job을 해당 SSH Location lane에서 실행한다.
  3. command argument를 구조화하고 shell interpolation을 금지한다.
  4. Modified status와 text 판정이 모두 확인된 target만 Remote diff를 요청한다.
  5. repository 없음, Git 없음, 안전하게 표현할 수 없는 byte path와 command failure는 cached
     Text로 fallback한다.
  6. Remote session epoch/path/generation이 다른 status/diff completion을 무시한다.
- 테스트:
  - Remote Modified text -> DIFF
  - Remote clean 또는 non-repository -> TEXT
  - Remote Git unavailable -> TEXT, Preview enabled 유지
  - binary/TooLarge -> Remote Git diff 호출 0회
  - invalid byte path -> SFTP TEXT 유지, Git diff skip
  - stale session/status/diff result 무시
  - quoting/injection contract
- 완료: Remote 여부가 Preview visibility 조건에 없고 가능한 Remote Modified text는 diff 우선.

### P7 통합 수용, 성능과 문서 종료

- 선행: P2~P6
- 목표: 전체 동작을 snapshot/scenario/실제 terminal에서 검증하고 문서를 현재 상태와 맞춘다.
- 주 파일:
  - `src/snapshots/*`
  - `tests/snapshots/*`
  - `tests/scenarios/resize.yml`
  - `tests/scenarios/selection.yml`
  - `docs/README.md`
  - `docs/preview-pane-implementation-plan.md`
  - `docs/implementation-plan/progress.md`
- 작업:
  1. Off, Local Text, Local Diff, Remote Text, Remote Diff snapshot을 추가한다.
  2. 80/100/119/120/121/160 width scenario를 실행한다.
  3. 빠른 key repeat 중 queue/coalescing과 render responsiveness를 측정한다.
  4. Linux/macOS terminal에서 `Alt+[`/`Alt+]` 전달을 확인한다.
  5. custom keymap fallback을 수동 확인한다.
  6. 실제 SSH host에서 text, binary, large file, non-repository와 Modified file을 확인한다.
  7. 완료 카드, test 이름, 날짜와 결과를 progress에 기록한다.
- 검증:
  - 공통 품질 게이트 전체
  - `.snap.new` 0개
  - key->reduce->render 50 ms 목표를 Preview Off/ready cache에서 유지
  - background read/diff 중 key input이 blocking되지 않음
- 완료: 아래 수용표의 자동 항목이 모두 통과하고 Linux/macOS 및 실제 SSH 증거가 기록됨.

## 5. 수용 기준

| ID | 기준 | 카드 | 자동 검증 | 수동 검증 |
|---|---|---|---|---|
| PREV-01 | Settings On/Off와 On/50 default | P2 | config/reducer roundtrip | 재시작 |
| PREV-02 | 120 이상 표시, 미만 자동 숨김 | P1,P2 | 119/120/121 layout test | 연속 resize |
| PREV-03 | 35~65%, 5% 단축키 조절 | P1,P2 | layout/input test | Linux/macOS key 전달 |
| PREV-04 | browser navigation과 render geometry 일치 | P1 | navigation/scenario | 키 감각 |
| PREV-05 | UTF-8 text만, 1 MiB bounded read | P3,P5 | local/remote matrix | 없음 |
| PREV-06 | stale result가 현재 target을 덮지 않음 | P3~P6 | generation tests | 빠른 key repeat |
| PREV-07 | Local Modified text는 unified diff 우선 | P4 | fake+real Git tests | 실제 repository |
| PREV-08 | Remote text Preview는 Git 없이도 동작 | P5 | Fake Remote tests | 실제 SSH |
| PREV-09 | 가능한 Remote Modified text는 diff 우선 | P6 | Git/Remote integration | 실제 SSH repository |
| PREV-10 | Git/Remote 오류가 navigation을 막지 않음 | P4~P6 | fault/lane tests | connection loss |
| PREV-11 | Preview Off에서 기존 UI와 동작 회귀 없음 | P1~P7 | 기존 전체 suite | walkthrough |

## 6. 구현 중 중단하고 결정해야 하는 조건

다음 상황은 구현자가 임의로 범위를 넓히지 않고 ADR과 이 계획을 먼저 갱신한다.

- Preview focus, scroll, search 또는 edit가 필요해짐
- 120/35/65/1 MiB 상수를 바꿔야 함
- Added/Untracked/Deleted 등 Modified 외 Git status를 포함해야 함
- Remote Git을 안전한 structured argument로 실행할 수 없음
- Remote SFTP read가 현재 cancel/deadline 계약을 만족하지 못함
- layout split 때문에 기존 column/navigation contract를 보존할 수 없음
