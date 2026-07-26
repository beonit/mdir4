# 테스트 계획

## 1. 원칙

1. reducer, layout, navigation, sorting은 실제 terminal/OS 없이 테스트한다.
2. 현재 M1 snapshot은 Ratatui `TestBackend`의 문자를 비교하고 theme 단위 테스트가 일부
   Style을 검증한다. 문자+Style 통합 snapshot은 M1-13 완료 조건이다.
3. 실제 사용자 디렉터리, 홈, 저장소 루트를 파일 작업 테스트 대상으로 쓰지 않는다.
4. 시간, 여유 공간, 전송 속도, 현재 경로, 수정 시각은 주입한다.
5. 버그를 수정할 때는 먼저 가장 작은 회귀 테스트를 추가한다.
6. 새 snapshot은 기능 완료 증거가 아니라 사람이 검토할 대상이다.

## 2. 테스트 계층

| 계층 | 대상 | 위치 | 외부 I/O |
|---|---|---|---|
| Unit | text/layout/navigation/reducer/model | 각 모듈 또는 `tests/*.rs` | 없음 |
| Component | UI component→Buffer | `tests/ui_*.rs` | TestBackend |
| Scenario | key sequence→state/snapshot | `tests/scenarios.rs` | Memory ports |
| Integration | RealFileSystem/atomic save/작업 | `tests/file_operations.rs` | TempDir만 |
| Platform | ShellExecute/drive/trash | `#[cfg(windows)]` tests | Windows temp data |
| Manual | 실제 terminal/휴지통/UNC | release checklist | 명시된 test data |

## 3. 현재 M1 테스트 하네스 기준선

현재 production/test 구조를 사실대로 고정한다.

- component test는 필요한 `AppState`를 helper로 만들고 `TestBackend`에 렌더한다.
- scenario runner는 `tests/support/harness.rs`의 `run_file`이며 `TestContext` 타입은 아직 없다.
- filesystem은 YAML의 inline 목록으로 `MemoryFileSystemBuilder`를 구성한다.
- effect는 현재 test executor가 즉시 완료한다. pending queue나 worker-event 주입은 없다.
- `clock`은 parse/result 보존만 하며 state/render에 아직 주입하지 않는다.
- `disk.free_bytes`는 `DiskInfoLoaded` completion 값으로 사용한다.

M1-13은 이 구조를 공용 builder/port bundle로 정리하되 기존 scenario 경로를 유지한다.
M2-02부터 mutation fault를 추가하며, 카드보다 먼저 future API를 만들지 않는다.

## 4. Fixture 계약

현재 M1 YAML은 named fixture나 `tests/fixtures/` 디렉터리를 사용하지 않는다. 각 파일은
아래처럼 inline entry를 선언한다.

```yaml
filesystem:
  - { path: /work, kind: directory }
  - { path: /work/docs, kind: directory }
  - { path: /work/readme.txt, kind: file, size: 1024 }
```

현재 허용 kind는 `directory`, `file`, `other`다. 오류 fixture와 file contents는 아직
schema에 없다. M2-02에서 read/write/mutation port를 확장할 때 다음 의미의 reusable
builder를 순서대로 추가한다: empty, basic, many-files, long-names, unicode, nested,
mixed-types, errors. RealFileSystem test는 같은 의미를 `TempDir` 아래에만 만든다.

## 5. YAML 시나리오 형식

### 5.1 현재 version 1 schema

아래 예시는 현재 parser가 그대로 실행할 수 있는 형식이다.

```yaml
version: 1
terminal: { width: 80, height: 25 }
start_path: /work
filesystem:
  - { path: /work, kind: directory }
  - { path: /work/a.txt, kind: file, size: 11 }
clock: "2026-07-25T12:00:00Z"
disk: { free_bytes: 12288 }
steps:
  - { action: start }
  - { action: key, key: down }
  - { action: key, key: space }
  - { action: snapshot, name: selection }
assertions: { path: /work, selected: 1, marked: 1, free_bytes: 12288 }
snapshots: [selection]
```

현재 지원 step은 정확히 네 개다.

| action | 필드 | 동작 |
|---|---|---|
| `start` | 없음 | `Action::Started`와 발생 effect를 즉시 완료 |
| `key` | `key` | 실제 InputMapper를 통과 |
| `resize` | `width`, `height` | `Action::Resize` 적용 |
| `snapshot` | `name` | 문자 buffer를 캡처 |

`assertions`는 scenario 끝에서 path/selected index/marked count/optional free bytes를 한 번
검사한다. `snapshots`는 캡처해야 할 이름 목록이다. unknown top-level field와 unknown step은
거부하며 step parse 오류에는 파일 경로와 1-based step 번호가 포함된다.

### 5.2 카드별 확장 순서

- M1-13: `clock`→FixedClock과 `timezone_offset_minutes`→FixedTimeZone을 분리하고 fixture의
  optional RFC3339 `modified`, `attributes: [read_only, hidden, system, archive]`,
  `metadata_error`를 추가한다. named effect completion, 단계별 `assert`, 문자+Style snapshot
  serializer도 같은 카드에서 추가한다.
- M2-09: `worker_event`와 느린 I/O/progress/conflict/cancel step.
- M3-01 이후: restart/config load/save step.

M1-13이 추가할 fixture 예시는 다음과 같다. 이 field는 M1-13 전 parser에서는 거부되는 것이
정상이다.

```yaml
timezone_offset_minutes: 0
filesystem:
  - { path: /work/a.txt, kind: file, size: 11,
      modified: "2026-07-25T12:00:00Z", attributes: [read_only] }
  - { path: /work/unavailable.txt, kind: file, size: 0,
      metadata_error: permission_denied }
```

snapshot scenario의 `timezone_offset_minutes`는 constant `FixedTimeZone`용이다. DST 회귀는
별도 `FakeTimeZone` table에 두 UTC timestamp→서로 다른 `LocalMinute`를 넣어 OS timezone과
CI 환경을 읽지 않고 검증한다. conversion error row도 같은 table로 `----- --:--`를 만든다.

새 step을 문서에 추가하려면 parser test, 실행 test, diagnostic test가 같은 카드에 있어야 한다.

## 6. 필수 탐색 테스트 표

다음 표는 모두 독립 assert를 갖는다.

```text
A1 B1 C1
A2 B2 C2
A3 B3
A4 B4
```

| 시작 | 키 | 결과 |
|---|---|---|
| A2 | Right | B2 |
| B2 | Right | C2 |
| B3 | Right | C2 |
| C2 | Left | B2 |
| A1 | Left | A1 |
| A1 | Up | A1 |
| B4 | Down | B4 |
| C2 | Right | C2 |

추가:

- 0/1개
- rows-1/rows/rows+1
- capacity-1/capacity/capacity+1
- 2×capacity-1/정확히/초과
- 100/1,000/10,000개
- resize 전후 같은 EntryId
- sort/filter 전후 같은 EntryId 또는 명시적 fallback

## 7. 레이아웃 불변식

각 크기/모드 조합에서 property-style loop로 검사한다.

```text
모든 Rect는 viewport 안에 있다.
영역끼리 의도치 않게 겹치지 않는다.
column Rect 폭의 합 == list 폭.
column 수는 1..=6 또는 too_small에서 0.
rows_per_column == list.height.
page_capacity == rows_per_column * column_count.
page_capacity가 0이어도 탐색은 패닉하지 않는다.
렌더 후 viewport 밖 셀 쓰기가 없다.
```

Auto mode에서는 `entry_count <= rows_per_column`일 때 1열이 전체 list 폭을 사용하고,
경계를 넘을 때 필요한 유효 컬럼 수만 증가하는지도 별도로 검사한다. 2열 이상이면 각
경계 셀이 box-drawing `│`이고 파일 내용이 그 셀을 덮지 않아야 한다.

필수 폭: 0, 1, 11, 12, 59, 60, 79, 80, 81, 99, 100, 101, 119, 120,
121, 159, 160, 161, 200.

필수 높이: 0, 1, 14, 15, 24, 25, 26, 30, 40, 50.

## 8. Snapshot 규칙

현재 M1 serializer는 각 셀의 `symbol()`만 저장한다. 따라서 색/수식어 회귀는
`theme`/`palette` 단위 test로만 일부 검출된다. M1-13에서 snapshot record를
`symbol + fg + bg + modifiers`로 확장하고 80/100/120/160열을 모두 추가한다. 그 전에는
현재 문자 snapshot을 style-aware 증거라고 기록하지 않는다.

현재 파일은 Insta 기본 module/test 기반 이름을 사용한다. 아래 이름은 M1-13에서
style-aware serializer를 도입하며 함께 적용할 목표 규칙이다. M1-13 전에는 이름만 맞추기
위해 기존 snapshot을 대량 rename하지 않는다.

M1-13 이후 이름:

```text
<screen>__<fixture>__<width>x<height>__<state>.snap
```

예:

```text
main__basic__80x25__startup.snap
main__unicode__80x25__marked.snap
dialog_copy__mixed__80x25__conflict.snap
mcd__nested__80x25__search.snap
```

승인 절차:

1. 테스트 실패 diff를 확인한다.
2. 현재 문자 diff와 palette test를 확인한다. M1-13 이후에는 문자와 Style diff를 별도로
   확인한다.
3. 관련 제품 계약/작업 카드와 일치하는지 확인한다.
4. 의도된 경우에만 snapshot을 승인한다.
5. `.snap.new`가 없는지 확인한다.

Clock/Timezone/DiskInfo/Path/Modified/Speed가 고정되지 않은 snapshot은 추가하지 않는다.
M1-13부터 snapshot fixture는 built-in label/Help/message/error가 영어인지 명시적으로
assert한다. Unicode 파일명, 경로와 사용자 입력은 영어 검사 대상에서 제외하고 exact 원문
보존을 별도 assert한다.

## 9. 파일 작업 안전 테스트

모든 Copy/Move/Delete 테스트는 다음을 확인한다.

- 테스트 대상 절대 경로가 현재 TempDir 하위인지 assert한 뒤 쓰기/삭제
- source==destination 거부
- destination inside source 거부
- drive/root/current dir/parent entry 삭제 거부
- symlink를 따라가지 않음
- cancel에서 임시 파일 제거
- 실패에서 원본 보존
- Overwrite/Overwrite All/Skip/Skip All/Rename/Cancel 여섯 충돌 선택을 각각 검증
- overwrite all/skip all은 OperationId가 바뀌면 초기화
- 부분 성공 결과 개수 일치
- 작업 종료/패닉 후 worker join

`MemoryFileSystem`에는 다음 fault injection을 제공한다.

- n번째 read/write에서 실패
- disk full
- permission denied
- cross-device rename
- metadata disappeared
- 느린 read/write

## 10. 설정 테스트

- default serialize→parse roundtrip
- v1 최소 TOML
- unknown field 보존 또는 안전 무시
- enum 오타 fallback+warning
- column auto/fixed 우선순위
- corrupt file `.broken-*` 보존
- atomic write 중 실패해도 기존 파일 유지
- theme/keymap 일부 오류가 전체 설정을 망가뜨리지 않음
- UNC/Unicode/QCD 100개

M2-13 sort table은 Name/Extension/Size/Date/Time × Asc/Desc × DirectoriesFirst를 모두 돈다.
`.gitignore`, `archive.tar.gz`, `name.`, mixed case, unknown size/modified/timezone error와 같은
local minute/different raw timestamp를 포함하고, missing-last와 Name/path ascending tie-break를
양 방향에서 확인한다. Main S/Ctrl+S/H/D와 Editor Ctrl+S context 분리, show_hidden=true 기본,
hidden 선택 제거 fallback, drive empty/error no-op도 검증한다.

## 11. 성능 검증

성능 수치는 flaky CI gate 대신 명시적 smoke/benchmark로 나눈다.

- 현재 test `ten_thousand_entry_navigation_and_render_smoke`는 debug 기본 test이며
  End→Home→layout→render를 100 ms 안에 끝내는지만 검사한다. sort, key mapping, worker는
  측정하지 않는다.
- 현재 명령: `cargo test --locked ten_thousand_entry_navigation_and_render_smoke`
- M1-13 목표 명령: `cargo test --release --locked perf_smoke_10k -- --ignored --nocapture`
- M1-13은 10,000개 sort+layout+render 100 ms와 key→map→reduce→render 50 ms 측정을
  이름 있는 ignored smoke로 추가한다. 실행 환경/반복 횟수/결과를 progress에 기록한다.
- worker progress 최대 20 Hz
- idle event loop가 busy-loop하지 않는지 5초 관찰

성능 실패 시 먼저 측정 결과와 fixture를 기록하고, 추측으로 캐시를 추가하지 않는다.

## 12. CI 명령

필수:

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
```

snapshot CLI를 채택하면 추가:

```text
cargo insta test --all-features --check --locked
```

Windows 전용 test는 `#[cfg(windows)]`로 두되 Windows CI에서 실행 횟수가 0이 아닌지
별도 test 이름 또는 로그로 확인한다.

## 13. v1 이후 Git built-in 테스트

Git 확장은 v1 수용 범위에 포함하지 않는다. `R1` 이후 `G0~G3`을 시작할 때는
[`../plugins/git/04-test-plan.md`](../plugins/git/04-test-plan.md)의 FakeBackend,
repository fixture, snapshot, mutation/network 안전 규칙을 적용한다. Git 테스트 때문에
기존 테스트가 사용자 repository나 global Git 설정을 읽게 해서는 안 된다.

## 14. v1 이후 SSH Remote 테스트

Remote Drive도 v1 수용 범위에 포함하지 않는다. `R1` 이후 `S0~S3`을 시작할 때는
[`../remote/04-test-plan.md`](../remote/04-test-plan.md)의 FakeRemote,
격리 SSH server, host-key 검증, fault/transfer/snapshot 규칙을 적용한다. 기본 테스트가
사용자의 `~/.ssh`, agent, known_hosts 또는 실제 network를 읽어서는 안 된다.
