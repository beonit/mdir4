# Screen별 Layout 분리와 Full-screen 폭 회귀 방지 계획

상태: `In progress`
작성 기준일: 2026-07-30
대상 플랫폼: Linux, macOS

이 문서는 Git Commit Detail에서 화면 오른쪽 절반이 비는 현상을 단일 화면의 폭 보정으로
처리하지 않고, 파일 브라우저용 geometry가 full-screen mode로 누출되는 구조를 제거하기
위한 구현 계획이다.

## 1. 관찰된 현상

2026-07-30 Git Commit Detail 실제 화면에서 다음 현상이 확인됐다.

- 상단 path bar와 하단 function bar는 터미널 전체 폭을 사용한다.
- Commit detail과 변경 파일 목록은 화면 왼쪽의 좁은 영역에 표시된다.
- 선택 파일의 diff는 화면 중앙 부근에서 끝난다.
- 화면 오른쪽 약 절반은 아무 콘텐츠 없이 남는다.
- 긴 author, subject, path와 diff line이 사용 가능한 화면보다 훨씬 일찍 잘린다.

이 현상은 문자열의 Unicode cell-width 계산 실패가 아니다.
`pad_or_truncate()`와 `cell_width()`는 grapheme과 terminal cell 폭을 기준으로 자른다.
잘못된 값은 문자열 길이가 아니라 renderer에 전달된 body `Rect`의 폭이다.

## 2. 직접 원인

현재 `layout::calculate()`는 Main 파일 브라우저를 기준으로 다음 값을 함께 계산한다.

- 전체 viewport
- path/status/function bar
- 파일 목록 body
- Preview body
- adaptive file columns
- rows/page capacity

Preview가 활성화되고 terminal width가 Preview 임계값 이상이면 `metrics.list.width`는
전체 폭에서 Preview 폭을 뺀 browser 폭이 된다.

예를 들어 160열, Preview 50% 조건은 개념상 다음과 같다.

```text
viewport.width = 160
browser/list    = 80
preview         = 80
```

Git mode는 viewport 전체를 지우고 full-screen처럼 렌더하지만, Git Log, Git Diff와 Git
Commit Detail 본문에는 `metrics.list`를 그대로 사용한다. Commit Detail은 제한된 list 폭을
다시 42:58로 분할한다.

```text
viewport 160
└─ browser metrics.list 80
   ├─ commit/files 약 33
   ├─ separator 1
   └─ diff 약 46

남은 약 80열은 렌더 대상이 아니다.
```

## 3. 근본 문제

### 3.1 하나의 Layout 타입이 여러 화면 의미를 표현한다

현재 `LayoutMetrics`는 실질적으로 Browser Layout이지만 다음 full-screen renderer도
공유한다.

- Git Status, Log, Commit Detail, Branch, Stash, Diff
- Viewer
- Favorites
- Amazon Build
- 일부 Remote 화면

타입만으로는 renderer가 `metrics.list`를 사용해도 되는지, `metrics.preview`가 현재 mode에
의미가 있는지 판별할 수 없다.

### 3.2 화면별 Preview 정책이 없다

- Main Browser는 설정에 따라 Preview가 필요하다.
- Git Status는 changed-file diff preview가 필요할 수 있다.
- Git Log와 Git Diff는 browser Preview가 필요하지 않다.
- Git Commit Detail은 자체 좌우 분할을 가진다.
- Viewer는 단일 문서 전체 폭이 필요하다.

현재는 이 정책들이 모두 Browser의 `preview.enabled`와 폭 계산에 간접적으로 묶인다.

### 3.3 Full-screen geometry가 Main 상태에 의존한다

runtime은 현재 screen과 관계없이 Main의 `entries.len()`, `long_view`, browser settings로
metrics를 만든다. 따라서 Git/Viewer geometry가 뒤에 숨은 Main의 파일 개수와 view 설정에
간접적으로 의존할 수 있다.

### 3.4 기존 테스트의 계약도 모호하다

넓은 화면 테스트 일부는 list가 전체 폭이라고 기대하지만 기본 Preview는 활성화되어 있다.
그 결과 기대 폭 120과 실제 browser 폭 60이 충돌한다. Preview on/off를 명시하지 않은
테스트는 Browser 계약과 Document 계약을 구분하지 못한다.

## 4. 목표

- Browser Layout과 full-screen Layout을 타입과 계산 함수로 분리한다.
- Preview는 명시적으로 지원하는 mode에서만 body 폭에 영향을 준다.
- Git Log, Git Diff와 Viewer는 전체 document body를 사용한다.
- Git Status와 Git Commit Detail은 각각의 전용 split geometry를 사용한다.
- renderer가 자신에게 전달된 `Rect` 밖의 geometry를 다시 계산하지 않게 한다.
- 60열 최소 화면부터 넓은 화면까지 영역의 합, 경계, separator를 자동 검증한다.
- 새로운 full-screen mode가 Browser Layout을 재사용하면 테스트 또는 타입 단계에서
  발견되게 한다.

## 5. 비목표

- theme, diff 색상과 Git 데이터 모델 변경
- Preview의 제품 기본값 변경
- Git Commit Detail의 42:58 비율 자체를 사용자 설정으로 노출
- terminal 최소 크기 60x15 변경
- Main column navigation 알고리즘 재설계
- Remote protocol 또는 file operation 변경

## 6. 목표 Layout 모델

### 6.1 공통 Shell

```rust
pub struct ShellLayout {
    pub viewport: Rect,
    pub path_bar: Rect,
    pub body: Rect,
    pub status_bar: Rect,
    pub function_bar: Rect,
    pub too_small: bool,
}
```

Shell은 화면 공통 골격만 계산한다. Preview, file columns, split ratio를 알지 않는다.

### 6.2 Browser

```rust
pub struct BrowserLayout {
    pub shell: ShellLayout,
    pub list: Rect,
    pub preview: Option<Rect>,
    pub columns: Vec<Rect>,
    pub rows_per_column: usize,
    pub page_capacity: usize,
}
```

Main의 Preview와 adaptive column 계산은 Browser Layout만 소유한다.

### 6.3 단일 Document

```rust
pub struct DocumentLayout {
    pub shell: ShellLayout,
    pub document: Rect,
}
```

적용 대상:

- Git Log
- Git Branch
- Git Stash
- Git Diff
- Viewer
- full-width가 제품 계약인 Favorites/Amazon Build

### 6.4 Git Status

```rust
pub struct GitStatusLayout {
    pub shell: ShellLayout,
    pub changes: Rect,
    pub diff_preview: Option<Rect>,
    pub separator: Option<Rect>,
}
```

Git Status preview 정책은 Browser Preview와 별도로 명시한다. 초기 구현에서는 기존 설정값을
입력으로 재사용할 수 있지만 계산 함수와 결과 타입은 분리한다.

### 6.5 Git Commit Detail

```rust
pub struct GitCommitLayout {
    pub shell: ShellLayout,
    pub commit_and_files: Rect,
    pub separator: Rect,
    pub diff: Rect,
}
```

계산 규칙:

- separator: 1 cell
- 왼쪽 최소 폭: 24 cells
- 오른쪽 최소 폭: 32 cells
- 여유 폭에서는 42:58
- 각 영역의 합은 정확히 `shell.body.width`
- 최소 지원 화면에서 영역이 겹치거나 viewport 밖으로 나가지 않음

## 7. 소유권과 호출 경계

- `layout` module은 `Rect` 계산만 수행하고 App state나 renderer를 알지 않는다.
- UI top-level dispatcher가 현재 screen에 맞는 순수 계산 함수를 선택한다.
- Main navigation reducer는 Browser Layout만 사용한다.
- full-screen renderer에는 `BrowserLayout` 전체를 넘기지 않는다.
- renderer는 전달된 body/split `Rect` 안에서 text만 구성한다.
- terminal cell-width와 path truncation은 기존 text helper를 계속 사용한다.

화면 종류를 `layout` module이 직접 import하는 방식은 피한다. UI dispatcher가 다음과 같이
순수 함수를 조합한다.

```text
Screen::Main           -> calculate_browser(...)
Screen::GitStatus      -> calculate_git_status(...)
Screen::GitLog         -> calculate_document(...)
Screen::GitLogDetail   -> calculate_git_commit(...)
Screen::GitDiff        -> calculate_document(...)
Screen::Viewer         -> calculate_document(...)
```

## 8. 구현 카드

### LAY-01 기준선과 재현 테스트

- 현재 미커밋 기능 변경을 보존한다.
- 60/80/96/119/120/128/160/240열의 현재 geometry를 기록한다.
- Preview on/off와 35/50/65% 조합을 기록한다.
- 160열에서 Git Commit body 오른쪽 끝이 80열 부근에 머무는 failing test를 추가한다.
- 기존 layout test 실패를 Preview 계약 불명확과 실제 결함으로 분류한다.

완료 조건:

- 현재 screenshot 현상이 자동 테스트에서 재현된다.
- 수정 전 실패 이유가 width 값으로 명확히 출력된다.

### LAY-02 Shell Layout 추출

- viewport/path/body/status/function 영역을 `calculate_shell()`로 이동한다.
- too-small 처리를 Shell 계약으로 통합한다.
- 모든 Shell Rect가 viewport 내부인지 검사하는 helper를 추가한다.
- 기존 `LayoutMetrics`는 전환 기간 동안 Browser adapter로 유지한다.

완료 조건:

- full-screen 화면이 Browser Preview 계산 없이 공통 body를 얻을 수 있다.

### LAY-03 Browser Layout 명시화

- Preview, columns, long-view, page capacity 계산을 `calculate_browser()`로 이동한다.
- Preview on/off 테스트에서 기대 폭을 명시한다.
- 기존 navigation API가 Browser Layout만 받도록 변경한다.
- 기존 column 수와 row balancing 테스트의 의미를 유지한다.

완료 조건:

- Browser 테스트는 Preview 계약을 암묵적으로 추론하지 않는다.
- Main navigation 회귀가 없다.

### LAY-04 Git Layout 전환

- Git Log/Branch/Stash/Diff를 Document Layout으로 이관한다.
- Git Commit Detail을 Git Commit Layout으로 이관한다.
- Git Status를 Git Status Layout으로 이관한다.
- Git renderer에서 `metrics.list`와 `metrics.preview` 직접 접근을 제거한다.
- Git footer와 status bar는 Shell의 전체 폭을 사용한다.

완료 조건:

- Git full-screen renderer가 Browser Layout 필드를 참조하지 않는다.
- 160열 Commit Detail의 diff 영역 오른쪽 끝이 viewport 오른쪽 끝과 같다.

### LAY-05 다른 Full-screen 화면 이관

- Viewer를 Document Layout으로 이관한다.
- Favorites와 Amazon Build의 full-width 제품 계약을 확인하고 이관한다.
- Remote는 Preview가 의도된 화면이므로 별도 Remote/Browser 계열 Layout으로 명시한다.
- Locate, MCD, Editor처럼 이미 viewport를 직접 쓰는 화면은 boundary invariant만 확인한다.

완료 조건:

- Preview 설정이 Viewer와 full-width picker의 body 폭을 바꾸지 않는다.
- Preview를 의도한 화면만 split geometry를 가진다.

### LAY-06 테스트 계약 정리

- 기존 layout 테스트에 Preview 의도를 명시한다.
- preview-off full-width 기대와 preview-on split 기대를 별도 테스트로 나눈다.
- 실패하던 wide/column/UI 테스트를 계약에 맞춰 통과시킨다.
- 단순히 기대값을 현재 결함에 맞춰 낮추지 않는다.

완료 조건:

- layout/UI 테스트의 expected width가 어떤 mode의 계약인지 이름에서 드러난다.

### LAY-07 통합과 실제 터미널 검증

- Linux/macOS에서 80/120/160열 실제 화면 확인
- Preview on/off 전환 후 Git Log와 Viewer 폭이 불변인지 확인
- Git Status preview가 의도한 split을 유지하는지 확인
- screenshot 조건을 다시 실행하고 오른쪽 공백이 사라졌는지 확인
- 문서와 코드의 최종 타입/함수 이름을 동기화한다.

## 9. 유닛 테스트 계획

### 9.1 Shell invariant

대상 폭:

```text
59, 60, 80, 95, 96, 119, 120, 128, 160, 240
```

대상 높이:

```text
14, 15, 25, 40
```

검증:

- 최소 크기 미만은 `too_small`
- 최소 크기 이상에서 body/status/function 높이가 0이 아님
- 모든 Rect가 viewport 내부
- path + body + status + function이 세로로 겹치지 않음
- function bar가 viewport 마지막 행까지 정확히 도달

### 9.2 Browser Layout

조합:

- Preview on/off
- Preview 35/50/65%
- Auto/Fixed column
- Compact/Normal/Wide/Custom width
- short/long view

검증:

- Preview off: list width == body width
- Preview on: list + preview == body width
- column 폭 합 == list width
- column 간 overlap/gap 없음
- page capacity == rows per column × column count
- navigation이 기존 selection identity를 유지

### 9.3 Document Layout

검증:

- Preview 설정을 바꿔도 document Rect가 동일
- document == shell body
- 120열과 160열에서 오른쪽 끝 == viewport 오른쪽 끝
- Main entry count와 long-view 값이 결과를 바꾸지 않음

### 9.4 Git Status Layout

검증:

- preview off: changes == shell body
- preview on: changes + separator + diff == shell body
- 최소 좌우 폭 보장
- narrow terminal에서 의도한 fallback 적용
- Browser column_count 설정이 결과를 바꾸지 않음

### 9.5 Git Commit Layout

검증:

- `left.width + separator.width + right.width == body.width`
- 각 Rect의 x 좌표가 연속
- 60열 최소 화면에서 overlap 없음
- 80열과 160열에서 42:58 비율이 허용 오차 안에 있음
- diff right edge == body right edge
- Preview on/off, 비율 변경과 무관

### 9.6 Unicode와 긴 텍스트

대상:

- 긴 author/subject/reference
- 긴 repository-relative path
- 한글, 일본어, combining character, emoji
- rename의 `old/path → new/path`

검증:

- 각 render line의 cell width가 할당 Rect를 넘지 않음
- grapheme 중간 절단 없음
- selection background가 할당 행 폭을 정확히 채움

## 10. 렌더링 회귀 테스트

필수 snapshot:

| Screen | 크기 | 조건 |
|---|---:|---|
| Git Log | 80x25 | Preview on/off |
| Git Log | 160x40 | Preview on/off |
| Git Commit Detail | 80x25 | 긴 path와 diff |
| Git Commit Detail | 160x40 | screenshot 재현 corpus |
| Git Status | 160x40 | preview on/off |
| Git Diff | 160x40 | unified/side-by-side |
| Viewer | 160x40 | Preview 설정 on/off |

Snapshot 외에 geometry를 직접 검사한다. 공백도 정상 콘텐츠일 수 있으므로 문자열이
비었는지만 검사하지 않는다. 다음을 확인한다.

- renderer에 전달된 body의 right edge
- 마지막 column의 style/background
- separator 위치
- footer와 body의 폭 일치

## 11. 수용 기준

| ID | 기준 |
|---|---|
| LAY-ACC-01 | Git Log body가 Preview 설정과 무관하게 전체 폭을 사용 |
| LAY-ACC-02 | Git Commit Detail diff가 viewport 오른쪽 끝까지 사용 |
| LAY-ACC-03 | Git Status는 전용 preview split을 유지 |
| LAY-ACC-04 | Viewer body가 Preview 설정의 영향을 받지 않음 |
| LAY-ACC-05 | Main Browser Preview와 column navigation 회귀 없음 |
| LAY-ACC-06 | 60x15 이상에서 split overlap/zero-width 없음 |
| LAY-ACC-07 | Unicode line이 Rect cell 폭을 초과하지 않음 |
| LAY-ACC-08 | full-screen renderer의 Browser Layout 직접 참조 0건 |
| LAY-ACC-09 | 기존 wide layout 실패가 명시된 계약 아래 통과 |
| LAY-ACC-10 | 160열 실제 터미널에서 screenshot의 오른쪽 공백 재현 안 됨 |

## 12. 검증 명령

```bash
cargo fmt --all -- --check
cargo check --all-targets --locked
cargo test --lib layout::
cargo test --lib ui::tests::
cargo test --test navigation
cargo test --test scenarios
cargo test --all-targets --locked
```

전체 suite의 다른 기존 실패가 있으면 이 트랙의 회귀와 분리해 기록하되, layout/UI 관련
실패를 기존 문제라는 이유로 허용하지 않는다.

## 13. 재발 방지 규칙

- full-screen renderer는 Browser Layout을 인자로 받지 않는다.
- Preview는 화면 계약에서 명시한 경우에만 body 폭을 차감한다.
- renderer 내부에서 viewport 비율로 새로운 최상위 영역을 만들지 않는다.
- split 계산은 `layout`의 순수 함수 한 곳에 둔다.
- 새로운 full-screen screen에는 80열과 160열 테스트를 함께 추가한다.
- `LayoutSettings::default()`에 의존해 Preview 의도를 추론하는 테스트를 작성하지 않는다.
- Browser와 Document의 width 계약을 같은 테스트 이름이나 helper로 섞지 않는다.

## 14. 롤백과 단계적 이관

한 번에 모든 renderer를 바꾸지 않는다.

1. Shell과 새 Layout 타입을 기존 코드 옆에 추가
2. Git Log/Commit Detail을 먼저 이관해 screenshot 결함 제거
3. Git Status와 Git Diff 이관
4. Viewer와 기타 full-screen mode 이관
5. 모든 사용처가 제거된 뒤 기존 범용 필드를 Browser 타입으로 축소

각 단계는 독립적으로 빌드·테스트 가능해야 한다. 전환 중 adapter는 허용하지만 새
renderer가 다시 범용 `metrics.list`를 참조하는 것은 허용하지 않는다.
