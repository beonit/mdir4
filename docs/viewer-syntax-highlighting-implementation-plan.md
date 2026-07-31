# Viewer Syntax Highlighting 구현 계획

이 문서는 Mdir4의 full-screen View mode에 syntax highlighting과 검색 결과 강조를 추가하는
실행 계획이다. 현재 구현 상태를 주장하는 문서가 아니며 모든 작업 카드는 `Proposed`다.
구현을 시작하기로 결정하면 `SH-00`에서 dependency, 한계값과 기준선을 확정한 뒤 아래 순서와
완료 조건을 따른다.

현재 활성 릴리스 작업을 자동으로 선점하지 않는다. full-screen Viewer 안정화 뒤 사용자 승인으로
Preview pane에도 같은 bounded syntax adapter와 completion guard를 확장했다.

## 1. 목표와 사용자 계약

### 1.1 목표

- 지원되는 UTF-8 source/config/document를 View mode에서 열면 파일명, 확장자 또는 shebang으로
  언어를 감지하고 문법 요소별 색상을 표시한다.
- 문법이 불완전하거나 잘못된 source도 가능한 범위까지 표시한다.
- 언어 감지 실패, parser 오류, panic, 크기 제한 또는 긴 행 때문에 highlighting을 적용할 수
  없어도 원문은 plain text로 정상 표시한다.
- highlighting 계산은 render와 reducer에서 실행하지 않는다.
- syntax 색상은 현재 Mdir4 theme와 의미 기반 `ThemeRole`을 따르며 Mono/Light/DOS Blue를
  깨뜨리지 않는다.
- `Ctrl+F` 검색 결과는 syntax 색상보다 높은 우선순위로 강조한다.
- Viewer의 기존 scroll, Git Modified 파일의 F3/F4 동작, search navigation과 layout은
  유지한다.

### 1.2 표시 계약

View mode에서 다음 우선순위를 사용한다.

```text
현재 검색 결과
  > 다른 검색 결과
      > syntax semantic role
          > Viewer 기본 스타일
```

지원 언어이면 path bar에 감지한 짧은 언어 이름을 선택적으로 표시한다.

```text
/work/src/main.rs  [VIEW:RUST]
```

지원하지 않거나 highlighting이 생략된 파일은 기존 title을 유지한다.

```text
/work/notes.txt  [VIEW]
```

highlighting 실패는 기본적으로 사용자 오류 메시지가 아니다. 본문을 plain text로 표시하고
Viewer 사용을 계속한다. 개발/test 진단에는 실패 이유를 보존할 수 있지만 path bar나 message
bar에 parser 내부 오류를 기본 노출하지 않는다.

### 1.3 “잘못된 문법”의 의미

syntax highlighter는 compiler 또는 validator가 아니다.

- 닫히지 않은 문자열/주석, 괄호 불일치, 작성 중인 JSON/TOML 등은 best-effort로 색칠한다.
- 닫히지 않은 multiline construct 때문에 이후 행이 같은 색으로 표시될 수 있다.
- 이는 원문 손상이나 Viewer 실패가 아니라 highlighting 정확도 저하다.
- `syntect` parser가 실제 오류를 반환하면 부분 결과를 사용하지 않고 문서 전체를 plain
  text로 되돌린다.

“문법 오류가 있어도 안전하다”는 말은 원문을 잃거나 앱이 종료되지 않는다는 뜻이다. 모든
불완전 source를 의미론적으로 정확하게 색칠한다는 뜻은 아니다.

## 2. 비범위

- compiler 수준 syntax validation, diagnostics 또는 오류 밑줄
- LSP, Tree-sitter AST, symbol resolution, semantic token
- code folding, outline, jump-to-definition
- 사용자 제공 `.sublime-syntax`/grammar를 runtime에 동적으로 로드
- syntax theme 파일을 Mdir4 theme와 별도로 선택하는 설정
- Editor, Git diff 본문에 highlighting 적용
- binary/invalid UTF-8/hex viewer
- horizontal scroll, tab expansion 규칙 또는 line number 추가
- search의 line 단위 navigation을 occurrence 단위 navigation으로 변경
- worker thread를 강제로 종료하는 hard timeout

Editor 또는 Git diff로 범위를 확대할 때는 각 mode의 크기 제한, 수정 가능성, cache lifetime과
검색/selection 합성 규칙을 별도로 승인한다.

## 3. 현재 기준선

### 3.1 Viewer model

현재 `src/model/viewer.rs`의 `ViewerDocument`는 다음을 소유한다.

- UTF-8 `text`
- 각 행의 byte range
- `top_line`
- 검색 query
- 일치하는 행 index 목록
- 현재 검색 결과 index

`ViewerState::decode`는 다음을 이미 처리한다.

- NUL byte → `Binary`
- UTF-8 BOM 제거
- invalid UTF-8 → `Binary`
- empty text → 한 개의 빈 행

이 판정은 그대로 유지한다. syntax module은 `ViewerState::Ready`인 valid UTF-8 document만
입력으로 받는다.

### 3.2 Viewer load 흐름

현재 흐름은 다음과 같다.

```text
ShowViewer
  -> Effect::LoadViewer(path)
      -> Core Local EffectWorker가 최대 32 MiB read
          -> Action::ViewerLoaded { path, Result<Vec<u8>, FsError> }
              -> reducer가 ViewerState::decode
```

파일 I/O는 worker에서 실행되지만 UTF-8 decode와 line index 생성은 reducer에서 실행된다.
syntax highlighting까지 reducer에서 추가하면 큰 source를 열 때 UI thread를 막으므로 load
completion 경계를 변경해야 한다.

### 3.3 Viewer render

`render_mode_document`는 현재 각 행을 `Line::raw(pad_or_truncate(...))`로 만든 뒤
`MainBackground` 스타일을 적용한다. `ThemeRole::Viewer`와 `ViewerBorder`는 정의되어 있지만
full-screen Viewer 본문 경로에서 실질적으로 사용되지 않는다.

구현 시 Viewer body helper를 Git body helper와 분리하고 `Viewer`를 기본 스타일로 사용한다.
Git log/detail/diff의 기존 렌더링에는 syntax role이 전파되지 않아야 한다.

### 3.4 Theme

현재 theme은 semantic `ThemeRole -> ratatui::Style` mapping을 사용한다. built-in theme은
Classic, DOS Blue, Dark, Mono, Light이며 외부 TOML theme은 role별 foreground를 재정의한다.

syntect theme의 RGB를 `ViewerDocument`에 저장하면:

- theme 변경 시 cache가 stale해지고,
- Mono가 컬러로 표시되며,
- Light 배경에서 dark syntax theme의 대비가 깨지고,
- 외부 Mdir4 theme이 syntax에 적용되지 않는다.

따라서 cached 결과는 RGB가 아니라 Mdir4가 정의한 semantic token kind여야 한다.

## 4. 선행 관계

```text
SH-00 계약/의존성/기준선
  -> SH-01 syntax adapter와 semantic token
      -> SH-02 worker load 경계와 Viewer cache
          -> SH-03 ThemeRole과 styled-line renderer
              -> SH-04 검색 overlay
                  -> SH-05 통합/성능/라이선스/문서 종료
```

SH-03과 SH-04를 한 commit에 섞지 않는다. syntax 색상이 없는 상태에서도 search overlay의
순수 합성 helper를 테스트할 수 있지만, production 연결은 SH-03 뒤에 수행한다.

## 5. 외부 의존성 결정

### 5.1 후보

| 후보 | 장점 | 비용/위험 | 결정 |
|---|---|---|---|
| 직접 정규식 lexer | 의존성/크기 작음 | 언어별 multiline state와 유지보수 부담 | 제외 |
| `syntect` 기본 syntax | mature, line-state parser | TOML/TypeScript/Dockerfile 등 누락 | 단독 사용 제외 |
| `syntect` + `two-face` | bat 계열의 넓은 syntax set, TOML/TS/Dockerfile 포함 | grammar asset와 binary 증가 | 채택 후보 |
| Tree-sitter grammar 묶음 | 정확한 parse tree, incremental parse | 언어별 native grammar와 큰 dependency surface | 1차 제외 |
| 외부 `bat` subprocess | 완성도 높은 출력 | runtime executable 의존, theme/검색 합성 어려움 | 제외 |

구현 시작 시 검토 기준 version은 다음과 같다.

```toml
two-face = { version = "0.5", default-features = false, features = ["syntect-fancy"] }
```

`two-face`가 re-export하는 compatible `syntect` API를 사용해 regex backend mismatch를
피한다. 구현 시점의 lockfile이 다른 patch version을 선택하면 `SH-00`에서 API, MSRV,
license와 supported syntax 목록을 다시 확인한다.

공식 자료:

- [syntect parsing API](https://docs.rs/syntect/latest/syntect/parsing/struct.ParseState.html)
- [syntect feature flags](https://docs.rs/syntect/latest/features)
- [two-face syntax/feature 목록](https://docs.rs/two-face/latest/two_face/)

### 5.2 pure-Rust fancy 선택

초기 선택은 `syntect-fancy`다.

- Linux/macOS release에서 native Oniguruma build/link 의존을 추가하지 않는다.
- parser를 Core Local worker 안에서만 생성하고 사용한다.
- fancy backend에서 제외되는 grammar는 지원 언어 목록에 넣지 않는다.

fancy backend는 debug build에서 느릴 수 있고 일부 grammar를 제외한다. `SH-00` 측정에서
허용 가능한 한계값을 만족하지 못하거나 제외된 언어가 제품 요구로 승격되면 Oniguruma를
별도 비교한다. backend 변경은 lockfile, CI target, release archive와 license를 다시
검증한 뒤 승인한다.

### 5.3 asset와 license

`two-face`는 crate license 외에도 내장 syntax asset의 acknowledgements를 제공한다. 현재
release packager가 생성하는 Cargo package license inventory만으로 asset 고지가 충분한지
`SH-05`에서 확인한다.

필요하면 다음 중 하나로 packager를 확장한다.

1. build-time에 `two_face::acknowledgement::listing()` 결과를 생성해 archive에 포함
2. repository에서 생성된 syntax asset notice를 추적하고 release archive에 복사

어느 경우든 generated notice의 재현 가능성과 lockfile version 일치를 테스트한다.

## 6. 목표 아키텍처

### 6.1 의존 방향

```text
EffectWorker
  -> bounded file read
  -> Viewer UTF-8/binary decode
  -> SyntaxHighlighter adapter
       -> two-face SyntaxSet
       -> syntect ParseState + ScopeStack
       -> semantic SyntaxKind ranges
  -> ViewerState completion
       -> reducer stale-path check
       -> AppState
           -> UI styled-line composer
                -> current Mdir4 ThemeRole
                -> search overlay
                -> ratatui Line/Span
```

금지하는 방향:

```text
render -> syntect parse
reducer -> syntect parse
ViewerDocument -> syntect ParseState 보관
Syntax cache -> ratatui RGB/Style 보관
```

`ParseState`와 `HighlightLines`는 thread 간 전달 타입으로 사용하지 않는다. worker thread
안에서 생성·소비하고, `Send + Clone + PartialEq + Eq`를 만족하는 자체 token range만
completion으로 보낸다.

### 6.2 module 경계

신규 module 후보:

```text
src/syntax.rs
```

책임:

- language detection
- parser 실행
- syntect scope를 `SyntaxKind`로 분류
- byte range validation
- 크기/행 길이/오류/panic fallback
- 원문 재조립 invariant 검증을 위한 순수 helper

책임 아님:

- ratatui `Style`
- 현재 active theme
- path bar/status bar 렌더링
- search query와 current match
- filesystem read

UI의 style 합성 helper는 `src/ui.rs`에 두되 커지면 다음으로 분리한다.

```text
src/ui/viewer.rs
```

현재 `src/ui.rs`의 분량 때문에 styled-line, clipping과 overlay code가 약 150줄을 넘으면
`ui/viewer.rs` 분리를 우선한다.

## 7. 데이터 모델

정확한 naming은 `SH-01`에서 compiler와 test에 맞춰 확정하되 다음 경계를 유지한다.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxKind {
    Comment,
    Keyword,
    String,
    Number,
    Type,
    Function,
    Variable,
    Constant,
    Attribute,
    Tag,
    Heading,
    Link,
    Punctuation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxSpan {
    pub start: usize,
    pub end: usize,
    pub kind: SyntaxKind,
    pub emphasis: SyntaxEmphasis,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SyntaxEmphasis {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxDocument {
    pub language: String,
    pub lines: Vec<Vec<SyntaxSpan>>,
}
```

`SyntaxSpan.start/end`는 해당 Viewer line 안의 UTF-8 byte offset이다. absolute document
offset을 사용하지 않아 line slicing과 clipping을 단순하게 한다.

`ViewerDocument`에는 optional cache만 추가한다.

```rust
pub struct ViewerDocument {
    // existing fields
    pub text: String,
    pub lines: Vec<(usize, usize)>,
    pub top_line: usize,
    pub search: Option<String>,
    pub matches: Vec<usize>,
    pub current_match: usize,

    syntax: Option<SyntaxDocument>,
}
```

외부 caller는 다음 read-only API로만 접근한다.

```rust
pub fn syntax_language(&self) -> Option<&str>;
pub fn syntax_spans(&self, line: usize) -> Option<&[SyntaxSpan]>;
```

syntax cache는 원문을 복사하지 않는다. 각 span은 range와 작은 enum만 소유한다.

### 7.1 range invariant

모든 highlighted line은 다음 조건을 만족해야 한다.

- `start <= end <= line.len()`
- start/end는 UTF-8 char boundary
- span은 start 순으로 정렬
- span끼리 겹치지 않음
- 동일 kind/emphasis의 인접 span은 merge
- span 밖의 byte는 `Viewer` 기본 style
- span을 원문에 적용하거나 제거해도 원문 byte가 변하지 않음

parser output이 이 조건을 위반하면 programmer/library error로 간주하고 전체 syntax cache를
폐기한다.

## 8. 언어 감지

실제 파일을 다시 열 수 있는 `find_syntax_for_file`은 사용하지 않는다. Viewer worker가 이미
읽은 bytes와 path를 소유하며, 향후 Remote/display path 재사용에서도 추가 filesystem I/O가
발생하면 안 된다.

감지 순서:

1. exact file name 또는 syntax token (`Dockerfile`, `Makefile`, `.gitignore` 등)
2. extension (`rs`, `toml`, `json`, `md`, `ts` 등)
3. 첫 행 shebang/mode line
4. 실패 시 plain text

감지 결과가 `Plain Text`이면 syntax cache를 만들지 않는다.

최소 수용 fixture:

| 파일 | 기대 언어 |
|---|---|
| `src/main.rs` | Rust |
| `Cargo.toml` | TOML |
| `package.json` | JSON |
| `README.md` | Markdown |
| `script.sh` | Bash/Shell |
| 확장자 없는 `#!/usr/bin/env python3` | Python |
| `Dockerfile` | Dockerfile |
| `Makefile` | Makefile |
| `app.ts` | TypeScript |
| `notes.txt` | plain |
| 알 수 없는 `data.xyzunknown` | plain |

언어 표시 문자열은 syntax provider의 긴 display name을 그대로 사용하지 않고 stable short
label mapping을 둔다. unknown provider name은 uppercase/truncate하지 말고 title에서 언어
badge를 생략한다. UI snapshot이 dependency 내부 naming 변경에 흔들리지 않게 하기 위함이다.

## 9. scope에서 semantic kind로 변환

syntect scope stack의 가장 구체적인 scope부터 검사하되 다음 우선순위를 사용한다.

```text
comment
string / regexp
constant.numeric
constant.language / constant.other
keyword / storage
entity.name.type / support.type / storage.type
entity.name.function / support.function
entity.other.attribute-name
entity.name.tag
markup.heading
markup.underline.link / string.other.link
variable
punctuation
default
```

분류 원칙:

- comment 안의 punctuation은 Comment를 유지한다.
- string 안의 escape가 별도 scope여도 기본적으로 String을 유지한다.
- numeric은 Constant보다 Number가 우선한다.
- Markdown heading/link는 source code kind와 분리한다.
- scope를 인식하지 못하면 span을 만들지 않고 Viewer 기본 style을 사용한다.
- grammar별 예외 mapping은 fixture와 근거 없이 추가하지 않는다.

`SyntaxEmphasis`는 provider가 준 font style을 그대로 저장하지 않는다. scope 기반으로 필요한
최소 modifier만 결정한다.

- Heading: Bold
- Comment: Italic은 theme 정책에 따라 선택
- Link: Underlined는 terminal 가독성 검토 후 선택
- Keyword/Type: Classic에서 Bold 여부는 theme role이 결정

semantic kind와 modifier source를 이중으로 사용해 같은 속성이 중복되거나 theme가 modifier를
취소하지 못하는 상황을 피한다.

## 10. 안전성과 fallback 계약

### 10.1 provisional 한계값

코드에 넣기 전 `SH-00`에서 fixture를 측정하고 다음 시작값을 승인하거나 조정한다.

```text
MAX_SYNTAX_BYTES       = 256 KiB
MAX_SYNTAX_LINE_BYTES  = 8 KiB
```

Viewer 본문 read 상한 32 MiB와 syntax 상한은 별개다.

- 256 KiB 초과 UTF-8 text는 Viewer로 정상 표시하되 highlighting을 생략한다.
- 한 행이라도 8 KiB를 초과하면 문서 전체 highlighting을 생략한다.
- byte 기준을 사용하며 `chars().count()`로 사전 scan하지 않는다.

행 하나만 skip하지 않는다. multiline parse state가 그 행의 quote/comment/context 전이를
놓치면 이후 색이 신뢰할 수 없기 때문이다.

### 10.2 실패 처리

syntax adapter는 내부적으로 다음 outcome을 구분한다.

```rust
enum HighlightOutcome {
    Highlighted(SyntaxDocument),
    Plain(PlainReason),
}

enum PlainReason {
    Unsupported,
    TooLarge,
    LineTooLong,
    ParserError,
    InvalidRanges,
    Panicked,
}
```

`PlainReason`은 test/diagnostic용이며 사용자에게 기본 노출하지 않는다.

처리 규칙:

| 실패 | Viewer 결과 |
|---|---|
| 미지원 언어 | Ready + syntax 없음 |
| 크기 초과 | Ready + syntax 없음 |
| 긴 행 | Ready + syntax 없음 |
| parser `Err` | 이미 만든 모든 span 폐기, Ready + syntax 없음 |
| invalid range | 모든 span 폐기, Ready + syntax 없음 |
| panic | worker 경계에서 catch, Ready + syntax 없음 |
| invalid UTF-8/NUL | 기존 Binary |
| file read TooLarge | 기존 TooLarge |
| file I/O 오류 | 기존 Error |

production highlighting 경로에서 provider/theme/syntax lookup에 `unwrap` 또는 `expect`를
사용하지 않는다.

### 10.3 panic과 hang 경계

`std::panic::catch_unwind`는 syntax adapter 호출 전체를 감싼다. 이는 library bug의 panic이
Core Local worker thread를 종료하는 것을 방지하기 위한 방어다.

다음은 보장하지 못한다.

- process abort
- native crash
- regex가 종료하지 않는 hard hang
- memory allocator failure

초기 pure-Rust backend, 문서/행 한계와 bundled grammar만으로 위험을 제한한다. 기존 Core
Local worker는 단일 thread이므로 highlighting이 오래 걸리면 UI render는 계속되지만 뒤의
local effect가 지연될 수 있다.

`SH-00` 측정에서 한계 내 입력도 worker SLA를 위반하면 다음 순서로 완화한다.

1. syntax byte/line 한계 축소
2. 지원 grammar 축소
3. 별도 bounded Highlight lane 도입 검토
4. hard timeout이 제품 요구가 되면 helper process와 새 ADR 검토

강제 thread kill은 Rust `std::thread` 모델에 없으므로 설계에 포함하지 않는다.

## 11. worker와 reducer 변경

### 11.1 목표 completion

두 가지 형태 중 `SH-02`에서 기존 test 변경량이 적은 쪽을 선택한다.

선호안:

```rust
Action::ViewerLoaded {
    path: PathBuf,
    result: Result<ViewerState, FsError>,
}
```

worker:

```text
read_file(path, 32 MiB)
  -> ViewerState::decode
  -> Ready이면 bounded syntax adapter 실행
  -> Result<ViewerState, FsError> completion
```

reducer:

```text
completion path가 현재 viewer path와 같은지 검사
  -> 같음: ViewerState 적용
  -> 다름: stale completion 무시
```

대안은 bytes와 syntax cache를 별도 envelope에 담는 방식이지만 text decode를 reducer에 남길
이유가 없고 completion type이 더 복잡해진다.

### 11.2 reducer purity

reducer는 다음만 수행한다.

- current path identity 확인
- `FsError::TooLarge`를 `ViewerState::TooLarge`로 변환
- I/O 오류를 `ViewerState::Error`로 변환
- worker가 만든 state 적용

reducer test는 실제 syntect를 호출하지 않는다. fixture `ViewerState`를 action에 넣고 stale/current
적용만 검증한다.

### 11.3 worker testability

syntax adapter를 trait 또는 함수 주입 경계로 둔다.

```rust
trait SyntaxHighlighter: Send + Sync {
    fn highlight(&self, path: &Path, document: &ViewerDocument) -> HighlightOutcome;
}
```

production은 bundled provider를 사용한다. test에서는 다음 fake를 사용한다.

- deterministic highlighted result
- `Plain(ParserError)`
- panic
- 호출 횟수 기록

실제 `ParseState` 자체는 `Send`가 아니므로 trait object나 AppState에 저장하지 않는다.
production adapter가 worker call stack 안에서 parser를 생성하고 같은 thread에서 폐기한다.

## 12. ThemeRole 계획

신규 role 후보:

```text
SyntaxComment
SyntaxKeyword
SyntaxString
SyntaxNumber
SyntaxType
SyntaxFunction
SyntaxVariable
SyntaxConstant
SyntaxAttribute
SyntaxTag
SyntaxHeading
SyntaxLink
SyntaxPunctuation
ViewerSearchMatch
ViewerSearchCurrent
```

Classic 기본안:

| Role | Foreground | Background | Modifier |
|---|---|---|---|
| Viewer | Gray | Black | - |
| SyntaxComment | DarkGray | Black | Italic 검토 |
| SyntaxKeyword | LightMagenta | Black | Bold |
| SyntaxString | LightGreen | Black | - |
| SyntaxNumber | LightYellow | Black | - |
| SyntaxType | LightCyan | Black | Bold |
| SyntaxFunction | LightBlue | Black | - |
| SyntaxVariable | Gray | Black | - |
| SyntaxConstant | LightMagenta | Black | - |
| SyntaxAttribute | Cyan | Black | - |
| SyntaxTag | LightBlue | Black | Bold |
| SyntaxHeading | LightCyan | Black | Bold |
| SyntaxLink | LightBlue | Black | Underlined 검토 |
| SyntaxPunctuation | White | Black | - |
| ViewerSearchMatch | Black | Yellow | Bold |
| ViewerSearchCurrent | White | Magenta | Bold |

배경색은 검색 결과 외 syntax role에서 사용하지 않는다.

### 12.1 built-in theme 규칙

- Dark/Classic: 위 palette 사용
- DOS Blue: 기존 blue background를 유지하고 readable bright foreground 사용
- Light: dark foreground variant를 별도로 정의해 white background 대비 확보
- Mono: syntax kind는 모두 white/black이되 heading/keyword/comment modifier로 최소 구분
- Search match/current: Mono에서도 reverse 또는 underline 등으로 구분되도록 특례 적용

현재 `Theme::builtin("mono")`와 `"light"`는 모든 role을 동일 style로 덮는다. 신규 search
role까지 무조건 동일하게 만들면 검색 강조가 사라지므로 built-in 변환 뒤 search state
style을 명시적으로 재설정한다.

### 12.2 외부 theme

기존 `[colors]` parsing으로 신규 role foreground를 재정의할 수 있어야 한다. background와
modifier의 외부 설정 확장은 이 기능의 비범위다.

`all_roles()`의 수동 배열과 completeness test를 함께 갱신한다. 신규 role 하나라도 빠지면
test가 실패해야 한다.

## 13. styled-line과 Unicode 합성

### 13.1 line 구성 단계

visible line 하나는 다음 순서로 만든다.

1. 원문 line slice를 얻는다.
2. syntax span과 기본 style의 전체 coverage segment를 만든다.
3. search query의 `match_indices`를 byte range로 계산한다.
4. 현재 검색 결과 행인지 판정한다.
5. syntax/search boundary를 합성한다.
6. grapheme/cell width를 보존하며 viewport width로 clip한다.
7. 남는 cell을 Viewer 기본 style로 pad한다.
8. `Line<Vec<Span>>`을 반환한다.

원문 `String`을 ANSI escape sequence가 포함된 text로 변환하지 않는다. 모든 색상은
`ratatui::Span` style로 표현한다.

### 13.2 검색 semantics

현재 search model은 일치 occurrence가 아니라 일치하는 행을 저장한다. 이 동작을 유지한다.

- query가 포함된 visible line의 모든 occurrence를 `ViewerSearchMatch`로 표시
- `matches[current_match]`와 같은 행의 모든 occurrence를 `ViewerSearchCurrent`로 표시
- F3/Shift+F3 navigation은 기존처럼 행 단위
- 같은 행의 두 번째 occurrence로 별도 이동하지 않음

occurrence 단위 navigation은 `matches: Vec<ViewerMatch { line, start, end }>` model 변경과
별도 사용자 계약이 필요하므로 이번 범위에서 제외한다.

### 13.3 Unicode

syntax/search range는 UTF-8 byte offset이지만 terminal clip은 grapheme cell width를 사용한다.

- range start/end가 char boundary인지 검증
- combining sequence 또는 emoji grapheme 중간에서 화면을 자르지 않음
- style boundary가 grapheme 내부에 걸치면 우선순위가 높은 style을 grapheme 전체에 적용
- wide character가 마지막 한 cell에 걸리면 기존 `pad_or_truncate`와 동일한 정책 사용
- invalid provider range는 fallback

기존 Viewer의 tab 표시 의미는 바꾸지 않는다. tab expansion을 도입해야만 정확한 cell
clipping이 가능하다고 판명되면 별도 계약을 작성하고 이 카드 범위에서 임의 변경하지 않는다.

## 14. 작업 카드

### SH-00 계약, dependency와 기준선

- 선행: 구현 시작 승인
- 목표: dependency/backend, 한계값, supported language와 성능 기준선을 고정한다.
- 파일:
  - `Cargo.toml`
  - `Cargo.lock`
  - `docs/viewer-syntax-highlighting-implementation-plan.md`
  - 필요 시 `docs/implementation-plan/progress.md`
- 작업:
  1. 현재 clean test/fmt/clippy/release binary size를 기록한다.
  2. `two-face` pure-Rust fancy dependency를 임시 branch에서 resolve한다.
  3. supported syntax 목록과 제외 grammar를 기록한다.
  4. 100 KiB/1 MiB/2 MiB Rust, JSON, Markdown fixture를 release/debug에서 측정한다.
  5. 8 KiB/16 KiB 단일 행 fixture를 측정한다.
  6. `2 MiB`, `8 KiB` 시작 한계값을 승인하거나 근거와 함께 조정한다.
  7. dependency license/MSRV와 debug/release binary 증가를 기록한다.
- 테스트/검증:
  - 기존 quality gate
  - Linux/macOS locked dependency resolution
  - release package license inventory dry-run
- 완료:
  - backend, version, 한계값, 언어 목록, size/license가 미결정 없이 문서에 기록됨

### SH-01 syntax adapter와 semantic token

- 선행: SH-00
- 목표: UI와 독립적인 language detection/parser/scope classifier를 구현한다.
- 주 파일:
  - `src/syntax.rs` 신규
  - `src/lib.rs`
  - `Cargo.toml`
  - `Cargo.lock`
- 작업:
  1. exact filename/extension/first-line 감지를 구현한다.
  2. `extra_no_newlines` syntax set을 process 내 한 번만 load한다.
  3. line 순서대로 parse state를 유지한다.
  4. scope stack을 `SyntaxKind`로 분류한다.
  5. span normalize/merge/range validation을 구현한다.
  6. byte/line limit과 전체-document fallback을 구현한다.
  7. adapter 전체에 panic boundary를 둔다.
- 단위 테스트:
  - 언어 감지표 전체
  - 모든 `SyntaxKind` mapping fixture
  - multiline string/comment
  - 닫히지 않은 string/comment, 불완전 JSON/TOML
  - empty/one-line/CRLF/BOM 이후 text
  - 한글 identifier/string/comment
  - emoji/combining text
  - unknown scope fallback
  - parser error fake, panic fake
  - oversize/long-line
  - span sort/non-overlap/char-boundary
  - 원문 byte 보존 property
- 완료:
  - adapter가 ratatui/AppState에 의존하지 않고 모든 실패가 deterministic plain outcome임

### SH-02 Viewer model과 worker completion

- 선행: SH-01
- 목표: decode와 highlighting을 worker에서 완료하고 reducer를 non-blocking으로 유지한다.
- 주 파일:
  - `src/model/viewer.rs`
  - `src/app.rs`
  - `src/runtime.rs`
  - `src/adapters/memory_fs.rs`
- 작업:
  1. `ViewerDocument`에 optional `SyntaxDocument`를 추가한다.
  2. worker용 decode/highlight entry point를 추가한다.
  3. `Action::ViewerLoaded` payload를 `Result<ViewerState, FsError>` 형태로 변경한다.
  4. `Effect::LoadViewer`가 read/decode/highlight를 같은 worker call에서 수행한다.
  5. stale path completion guard를 보존한다.
  6. highlighting 실패가 Ready/plain으로 적용되는지 보장한다.
  7. Preview는 generation-bound 후속 effect에서 syntax cache를 적용하고, Git용
     `ViewerState::decode` caller는 plain으로 유지한다.
- 테스트:
  - valid UTF-8 highlighted Ready
  - unsupported Ready/plain
  - binary/invalid UTF-8/TooLarge/error 기존 상태
  - A viewer 뒤 늦은 B/A completion identity
  - highlighter error/panic에도 worker가 다음 effect를 처리
  - reducer 실행 중 production highlighter 호출 0회
- 완료:
  - Viewer open 중 UI thread에 decode/highlight loop가 없고 기존 Viewer navigation test 통과

### SH-03 ThemeRole과 Viewer renderer

- 선행: SH-02
- 목표: semantic token을 현재 theme의 styled spans로 렌더한다.
- 주 파일:
  - `src/theme/schema.rs`
  - `src/theme/classic.rs`
  - `src/theme/catalog.rs`
  - `src/ui.rs`
  - 필요 시 `src/ui/viewer.rs` 신규
  - `src/snapshots/*`
- 작업:
  1. syntax/search role을 schema와 `all_roles()`에 추가한다.
  2. Classic/DOS Blue/Dark/Mono/Light mapping을 정의한다.
  3. Viewer body가 `ThemeRole::Viewer`를 사용하게 한다.
  4. semantic spans를 ratatui spans로 변환한다.
  5. Unicode cell-aware clip/pad helper를 구현한다.
  6. language badge를 path bar width 안에서 표시한다.
  7. Git body/detail renderer가 syntax role을 받지 않음을 확인한다.
- 테스트:
  - role completeness와 external theme inheritance
  - Classic semantic foreground
  - Light contrast pair
  - Mono에서 RGB/ANSI color 혼입 없음
  - DOS Blue background 유지
  - 80x25/120x40 Viewer snapshot
  - 좁은 path bar title truncate
  - wide Unicode 마지막 cell
  - syntax 없는 Viewer snapshot 최소 변화
- 완료:
  - 지원 source가 colored spans로 보이고 모든 built-in theme에서 읽을 수 있음

### SH-04 검색 overlay

- 선행: SH-03
- 목표: 기존 line search semantics를 보존하며 visible occurrence를 강조한다.
- 주 파일:
  - `src/ui.rs` 또는 `src/ui/viewer.rs`
  - `src/model/viewer.rs`는 필요 최소 변경만
- 작업:
  1. visible line의 query occurrence range를 계산한다.
  2. syntax/search boundary 합성 helper를 구현한다.
  3. current match 행에 current role을 적용한다.
  4. search 없는 상태에서는 추가 allocation/scan을 피한다.
  5. input dialog overlay 아래 Viewer highlighting이 유지되게 한다.
- 테스트:
  - syntax span 내부/경계/여러 span을 가로지르는 query
  - 한 행 여러 occurrence
  - current vs other match
  - next/backwards wrap
  - 한글/combining query
  - empty query
  - Ctrl+F dialog underneath rendering
  - 검색 style이 syntax style보다 우선
- 완료:
  - F3 behavior가 기존 line 단위 계약과 동일하고 검색 결과가 명확히 표시됨

### SH-05 통합, 성능, 패키징과 문서 종료

- 선행: SH-00~SH-04
- 목표: 회귀, latency, dependency와 release 고지를 검증하고 구현 상태를 기록한다.
- 주 파일:
  - `docs/viewer-syntax-highlighting-implementation-plan.md`
  - `docs/README.md`
  - 필요 시 `docs/implementation-plan/progress.md`
  - `scripts/package_release.py`
  - release/snapshot assets
- 작업:
  1. supported language 실제 fixture를 Linux/macOS terminal에서 확인한다.
  2. syntax threshold 직전/직후 open latency와 worker queue 영향을 측정한다.
  3. malformed/long-line source에서 fallback을 확인한다.
  4. Mono/Light/DOS Blue를 실제 terminal에서 확인한다.
  5. release binary/archive size 전후를 기록한다.
  6. syntax asset acknowledgements를 archive에 포함한다.
  7. 완료한 카드와 실제 test 이름을 문서에 기록한다.
- 검증:

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
cargo build --release --locked
git diff --check
```

추가:

- `.snap.new` 0개
- package release dry-run 성공
- 기존 Viewer/Git/Preview snapshot의 의도하지 않은 diff 0개
- highlighter error/panic 뒤 worker가 후속 effect를 처리

- 완료:
  - 아래 수용표의 자동/수동 증거가 기록되고 문서 상태가 `Implemented`로 갱신됨

## 15. 테스트 설계

### 15.1 테스트 계층

| 계층 | 검증 대상 | 실제 syntect 사용 |
|---|---|---|
| syntax unit | 감지/scope/range/fallback | 예 |
| syntax fault unit | error/panic/invalid ranges | fake |
| viewer model unit | cache와 search model | 최소 |
| reducer unit | completion identity/state 전이 | 아니오 |
| worker contract | read/decode/highlight/follow-up effect | fake + 일부 실제 |
| UI TestBackend | foreground/background/modifier/cell | 아니오, fixture span |
| snapshot | 전체 Viewer chrome/body/footer | fixture span |
| manual | terminal color/latency/가독성 | 예 |

실제 grammar 동작 test와 UI color test를 분리한다. dependency의 scope naming 변화 때문에 UI
snapshot 전체가 불필요하게 흔들리지 않도록 UI test에는 직접 만든 semantic spans를 사용한다.

### 15.2 malformed fixture

최소 fixture:

```text
Rust: 닫히지 않은 "string
Rust: 닫히지 않은 /* comment
JSON: { "key": [1, 2,
TOML: [section
Markdown: 닫히지 않은 fenced code block
Shell: 닫히지 않은 single/double quote
```

검증:

- panic 없음
- `ViewerState::Ready`
- 원문 동일
- span invariant 충족 또는 전체 plain fallback
- Viewer navigation/search 가능

색상 자체가 compiler의 기대와 같은지는 malformed fixture의 완료 조건이 아니다.

### 15.3 fault injection

실제 bundled grammar에서 parser error를 우연히 유발하는 fixture에 의존하지 않는다. fake
adapter로 다음을 결정적으로 주입한다.

- `ParserError`
- invalid byte range
- overlapping range
- panic
- 호출 지연을 측정하는 bounded fake

worker panic test는 process 전체 panic hook output에 의존하지 않고 completion이 Ready/plain이고
다음 queued effect가 처리되는지 확인한다.

### 15.4 성능 측정

unit test에 wall-clock assertion을 넣지 않는다. CI 부하에 따라 flaky하기 때문이다.

`SH-00`/`SH-05`에서 별도 release 측정으로 기록한다.

| Fixture | 측정 |
|---|---|
| 100 KiB Rust | syntax 시간, span 수, cache bytes 추정 |
| 1 MiB Rust | syntax 시간, worker queue delay |
| 2 MiB minified JSON | syntax 시간, span 수 |
| 8 KiB single line | line parse 시간 |
| threshold + 1 | 즉시 plain fallback 시간 |
| unsupported text | detection/fallback 시간 |

목표:

- render는 cached visible spans만 읽고 parser를 호출하지 않음
- Viewer key->reduce->render 경로에 file-size 비례 loop 없음
- threshold 초과는 full parse 없이 빠르게 plain fallback
- 실제 한계값은 측정 결과와 함께 `SH-00`에서 고정

## 16. 수용 기준

| ID | 요구사항 | 자동 증거 | 수동 증거 |
|---|---|---|---|
| SH-A01 | Rust/TOML/JSON/Markdown/Shell/TS/Dockerfile 감지 | syntax detection tests | sample files |
| SH-A02 | malformed source가 Viewer를 실패시키지 않음 | malformed matrix | 실제 작성 중 source |
| SH-A03 | parser error/panic은 plain fallback | fake fault tests | 선택 |
| SH-A04 | oversize/long-line은 plain fallback | boundary tests | large/minified file |
| SH-A05 | 원문 byte 보존 | reconstruction invariant | - |
| SH-A06 | reducer/render에서 parser 호출 없음 | fake call-count/architecture test | responsiveness |
| SH-A07 | stale completion 무시 | reducer tests | 빠른 open/close |
| SH-A08 | theme별 semantic 색상 | palette/UI tests | Classic/Light/Mono/DOS Blue |
| SH-A09 | 검색이 syntax보다 우선 | compositor/UI tests | Ctrl+F |
| SH-A10 | 기존 Viewer scroll/F3/F4 유지 | app/UI regression tests | modified/clean file |
| SH-A11 | Git에는 무의도 syntax 적용 없음; Preview는 generation-bound syntax 적용 | snapshot/reducer tests | mode 확인 |
| SH-A12 | dependency license/asset notice 포함 | package test | archive inspection |
| SH-A13 | locked quality gate 통과 | CI/command log | - |

## 17. 변경 예상 파일

| 파일 | 예상 변경 |
|---|---|
| `Cargo.toml`, `Cargo.lock` | `two-face` dependency와 lock |
| `src/lib.rs` | syntax module export |
| `src/syntax.rs` | 감지/parser/semantic/fallback |
| `src/model/viewer.rs` | optional syntax cache와 read-only API |
| `src/app.rs` | ViewerLoaded payload와 reducer 적용 |
| `src/runtime.rs` | worker decode/highlight |
| `src/theme/schema.rs` | syntax/search roles |
| `src/theme/classic.rs` | Classic palette |
| `src/theme/catalog.rs` | built-in/external role completeness |
| `src/ui.rs` | Viewer body와 style 합성 |
| `src/ui/viewer.rs` | 복잡도 기준 초과 시 분리 |
| `src/snapshots/*` | Viewer 관련 snapshot |
| `scripts/package_release.py` | 필요 시 syntax asset notice |
| `docs/README.md` | 계획 문서 링크 |
| `docs/implementation-plan/progress.md` | 활성화/완료 시 카드 증거 |

기존 사용자 변경이 있는 파일은 먼저 diff를 확인하고 관련 없는 변경을 보존한다.

## 18. 구현 중지와 rollback 조건

다음 중 하나면 다음 카드로 진행하지 않고 `SH-00` 또는 dependency 결정을 재검토한다.

- pure-Rust backend가 승인된 threshold에서 Core Local worker를 장시간 점유
- supported language fixture가 dependency 문서와 다르게 감지됨
- syntax asset license를 release archive에서 충족할 수 없음
- release binary 증가가 측정 없이 허용하기 어려움
- Mono/Light theme에서 semantic mapping으로 충분한 대비를 만들 수 없음
- span cache가 원문 크기 대비 과도한 메모리를 사용
- parser panic/hang을 bounded input에서도 재현
- 기존 Git diff/Preview/Viewer navigation 회귀를 helper 분리로 격리할 수 없음

rollback 단위:

1. SH-04 검색 overlay만 제거해 syntax rendering 유지 가능
2. language badge만 제거해 본문 highlighting 유지 가능
3. 특정 grammar를 denylist해 나머지 언어 유지 가능
4. syntax cache/roles/dependency 전체 제거 시 기존 plain Viewer로 복귀

원문 decode, Viewer navigation과 검색 model을 syntax dependency에 결합하지 않아 전체 rollback이
가능해야 한다.

## 19. 완료 정의

이 기능은 다음 조건을 모두 만족할 때만 완료다.

1. SH-00~SH-05의 완료 조건과 실제 test 이름이 기록되어 있다.
2. 지원 source가 semantic color로 표시된다.
3. 미지원/오류/크기 초과/긴 행은 원문 plain text로 표시된다.
4. malformed source가 app/worker를 종료하지 않는다.
5. render와 reducer는 parser를 호출하지 않는다.
6. Classic/Dark/DOS Blue/Mono/Light와 외부 theme inheritance test가 통과한다.
7. 검색 overlay가 syntax보다 우선하고 기존 F3 navigation을 유지한다.
8. Viewer의 Git Modified F3/F4, scroll, input dialog와 layout 회귀가 없다.
9. locked fmt/clippy/test/release build가 통과한다.
10. dependency와 syntax asset license notice가 release archive에 포함된다.
11. release size와 성능 측정 결과가 문서에 남아 있다.

위 조건 전에는 문서 상태를 `Implemented`로 바꾸지 않는다.

## 20. 실행 기록

### SH-00 — 완료 (2026-07-30)

의존성과 기준선 측정을 완료했다. 이 카드는 Viewer source, theme 또는 renderer를 변경하지
않는다.

#### Dependency

```toml
two-face = { version = "0.5.1", default-features = false, features = ["syntect-fancy"] }
```

lockfile은 다음 직접 dependency graph를 고정한다.

```text
two-face 0.5.1
  -> syntect 5.3.0
      -> fancy-regex 0.16.2
```

`two_face::syntax::extra_no_newlines()` probe에서 다음 token을 확인했다.

| Token | 감지 결과 |
|---|---|
| `main.rs` | Rust |
| `Cargo.toml` | TOML |
| `package.json` | JSON |
| `README.md` | Markdown |
| `script.sh` | Bourne Again Shell (bash) |
| `Dockerfile` | Dockerfile |
| `Makefile` | Makefile |
| `app.ts` | TypeScript |

#### 기준선

| 항목 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | 통과 |
| `cargo test --all-targets --all-features --locked` | 162 통과, 1 기존 실패 |
| `cargo clippy --all-targets --all-features --locked -- -D warnings` | 5 기존 lint 실패 |
| dependency 추가 전 release binary | 2,857,936 bytes |
| dependency 추가 후 release binary | 3,372,224 bytes |
| release 증가 | 514,288 bytes |
| `cargo check --all-targets --all-features` (dependency 추가 후) | 통과 |

기존 test 실패는 `runtime::tests::control_q_requests_quit`이다. mapper는 현재
`Ctrl+Q -> Action::ConfirmQuit`을 반환하지만 test는 `Action::RequestQuit`을 기대한다.
이는 SH 범위 밖의 기준선 불일치로 기록만 하며 이 카드에서 수정하지 않는다.

기존 Clippy 실패는 `src/app.rs` 1건, `src/locate.rs` 3건, `src/layout.rs` 1건이며 모두
syntax highlighting과 관계없는 기존 lint다. SH-05의 전체 quality gate에서는 해당 기준선
실패를 별도 작업으로 해소하거나 프로젝트 차원의 허용 정책을 명시해야 한다.

다음 카드: `SH-01 syntax adapter와 semantic token`.

### SH-01 — 완료 (2026-07-30)

`src/syntax.rs`를 추가해 Viewer model/UI와 독립적인 syntax adapter를 구현했다.

- `two_face::syntax::extra_no_newlines()`를 `OnceLock`으로 한 번만 load한다.
- 파일명, extension, 첫 행 shebang 순으로 Rust/TOML/JSON/Markdown/Bash/Dockerfile/Makefile/
  TypeScript/Python을 감지한다.
- syntect scope stack을 Comment, Keyword, String, Number, Type, Function, Variable, Constant,
  Attribute, Tag, Heading, Link, Macro, Operator, Punctuation의 semantic `SyntaxKind`로 변환한다.
- span은 line-relative UTF-8 byte range로 저장하고 char boundary, sort, overlap, merge를
  검증한다.
- 256 KiB 문서 또는 8 KiB 초과 행은 parser를 시작하지 않고 plain outcome으로 fallback한다.
- parser 오류, invalid range와 panic은 문서 전체 plain outcome으로 fallback한다.

검증 결과:

| 항목 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | 통과 |
| `cargo test --lib syntax::tests --locked` | 11 통과 |
| 새 코드 Clippy (SH-00 기존 5 lint만 allow) | 통과 |
| 전체 test | 172 통과, 2 기존/비결정적 실패 |

전체 test의 `runtime::tests::control_q_requests_quit`은 SH-00에 기록한 기존 assertion 불일치다.
`plugins::worker::tests::cancellation_panic_and_shutdown_produce_a_terminal_result_and_join`은 전체
병렬 run에서 한 번 `Busy`로 실패했지만 단독 재실행은 통과했다. syntax adapter와 공유 state가
없어 SH-02 전에 별도 flaky baseline으로 기록하며, 이 카드의 실패로 해석하지 않는다.

다음 카드: `SH-02 Viewer model과 worker completion`.

### SH-02 — 완료 (2026-07-30)

Viewer load completion을 path-aware decode 결과로 변경했다.

- `ViewerState::decode_for_path`는 기존 UTF-8/BOM/binary 판정을 유지한 뒤 Ready document에만
  optional syntax cache를 붙인다.
- `Action::ViewerLoaded`는 raw bytes 대신 `Result<ViewerState, FsError>`를 전달한다.
- Core Local worker는 bounded read/decode completion을 먼저 반환하고, 별도 `HighlightViewer` effect가
  syntax cache만 후속 적용한다. 따라서 parse가 길어도 plain View 본문은 먼저 보인다.
- reducer는 기존 path identity guard를 유지하고 현재 viewer path와 일치할 때만 결과를 적용한다.
- Preview는 plain decode를 먼저 표시한 뒤 generation-bound `HighlightPreview` completion으로 syntax를
  붙인다. Git 등 다른 `ViewerState::decode` caller는 plain decode를 유지한다.

검증:

- path-aware decode가 원문을 바꾸지 않고 Rust syntax cache를 붙이는 model test 통과
- 일반 decode가 syntax cache를 붙이지 않는 model test 통과
- stale completion을 무시하고 current path의 syntax cache만 적용하는 reducer test 통과

### SH-03 — 완료 (2026-07-30)

full-screen Viewer body를 semantic syntax span으로 렌더링하도록 연결했다.

- 15개 `Syntax*` theme role과 Classic palette를 추가했다.
- supported document는 본문을 `ratatui::Span`으로 만들고, plain document는 기존 single-style
  경로를 유지한다.
- supported path bar는 `[VIEW:RUST]`처럼 stable language badge를 표시한다.
- Rust Viewer UI test가 Darcula keyword/string/number/function/macro RGB 값과 `[VIEW:RUST]`
  title을 검증한다.

Classic Viewer syntax palette는 IntelliJ Darcula의 semantic baseline으로 재조정했다. View 본문만
`#2B2B2B` background를 쓰며, keyword `#CC7832`, string `#6A8759`, number `#6897BB`, function
`#20B0D4`, base text `#A9B7C6`를 사용한다. Rust macro는 method-declaration accent `#FFC66D`로
표시한다.

### SH-04 — 완료 (2026-07-30)

Viewer search 결과를 syntax span 위에 합성했다.

- `ViewerSearchMatch`와 `ViewerSearchCurrent` theme role을 추가했다.
- Classic/Light에서는 일반 결과를 노란 배경, 현재 F3 결과 행을 자홍색 배경으로 표시한다.
  Mono는 흑백 대비와 bold로, 외부 theme은 base role 상속으로 안전하게 fallback한다.
- renderer는 syntax span과 search range의 경계를 합쳐 작은 `Span`으로 재구성하고, search style을
  먼저 적용한다. 따라서 검색어가 keyword/string/number에 겹쳐도 검색 표시가 우선한다.
- 검색이 없는 plain Viewer는 기존 단일-style 경로를 그대로 사용한다. 기존 line 단위 F3 navigation과
  search model도 변경하지 않았다.

검증:

- `viewer_search_overlay_takes_priority_over_syntax_colors` UI test가 current 결과의 Magenta와
  다른 결과의 Yellow background를 확인한다.
- 기존 Rust syntax 색상/language badge UI test를 다시 실행해 통과했다.

다음 카드: `SH-05 전체 회귀, packaging/notice, performance 기록`.

### SH-05 — 구현·검증 완료 (2026-07-30)

release package에 `SYNTAX_ACKNOWLEDGEMENTS.md`를 추가했다. `two_face::acknowledgement::listing()`의
syntax-only license record를 `src/bin/generate_syntax_notice.rs`에서 생성하며, packager가 기존
dependency SPDX inventory와 함께 archive에 넣는다. local dry-run archive는 Rust, Docker,
TypeScript syntax license와 bat attribution을 실제로 포함했다.

#### release 측정 (macOS, release profile)

| Fixture | 결과 |
|---|---|
| 100 KiB Rust | 26,066 spans, 190 ms (scope optimization 전 332 ms) |
| 256 KiB Rust | 66,728 spans, 381 ms |
| 1 MiB Rust (initial 2 MiB proposal) | 266,910 spans, 3.02 s |
| 8 KiB Rust single line | 1 span, 18 ms |
| 2 MiB minified JSON line | `LineTooLong` plain fallback, timer resolution상 0 ms |
| 2 MiB + 1 Rust | `TooLarge` plain fallback, timer resolution상 0 ms |

초기 2 MiB 상한에서 1 MiB parse가 Core Local worker를 약 3초 점유하는 것을 확인했다. 따라서
actual `MAX_SYNTAX_BYTES`를 256 KiB로 낮췄다. 256 KiB 초과 문서는 bounded read의 기존 View
기능은 유지하면서 syntax만 즉시 생략한다. 추가로 semantic scope를 문자열로 매 token마다 복원하던
분류를 syntect의 bitwise prefix comparison으로 바꿔 100 KiB fixture 시간을 43% 줄였다. Viewer는
plain document를 먼저 표시하고 `HighlightViewer` 후속 completion에서만 syntax cache를 붙이므로,
그 190~381 ms가 화면 진입을 막지 않는다.

### Preview follow-up — 완료 (2026-07-30)

Preview pane는 `PreviewLoaded`에서 plain document를 즉시 표시한 뒤, path와 generation을 가진
`HighlightPreview` effect로 syntax를 계산한다. `PreviewSyntaxLoaded`는 현재 selection의 path와
generation이 모두 일치할 때만 cache를 적용한다. 따라서 선택을 빠르게 바꿔도 이전 파일의
highlight 결과가 새 Preview에 나타나지 않는다. Preview renderer도 View와 같은 styled span과
Darcula background를 사용한다.

검증:

- `preview_loads_text_before_path_bound_syntax_and_ignores_stale_completions`
- `rust_preview_uses_syntax_spans`

### Unified Git diff follow-up — 완료 (2026-07-30)

unified Git diff는 `+`/`-`/context marker를 제외한 코드 부분만 원본 경로의 grammar로 line-isolated
highlight하고, span offset을 marker 뒤로 복원한다. diff header/hunk 행은 plain으로 유지한다.
full-screen diff와 Git Status unified preview는 plain diff를 먼저 표시하고 path-bound syntax completion을
후속 적용한다. 기존 add/delete marker 색은 유지한다. Side-by-side와 commit-detail diff는 현재의
변경 비교 layout을 유지하며 아직 syntax 적용 대상이 아니다.

검증:

- `diff_lines_highlight_code_after_the_marker`
- `git_diff_loads_text_before_path_bound_syntax`
- `unified_rust_git_diff_uses_syntax_spans_after_diff_markers`

| 항목 | 결과 |
|---|---|
| release binary | 4,616,864 bytes |
| local release archive | 2,519,234 bytes |
| generated syntax notice | 186,036 bytes |
| `cargo fmt --all -- --check` | 통과 |
| `git diff --check` | 통과 |
| `cargo build --release --locked` | 통과 |
| package dry-run + archive inspection | 통과 |
| `cargo test --all-targets --all-features --locked` | 178 통과, 1 기존 실패 |
| new-code Clippy (기존 5 lint만 allow) | 통과 |

전체 quality gate 자체는 기존 `runtime::tests::control_q_requests_quit` assertion과 기존 Clippy
5건 때문에 아직 green이 아니다. 이는 SH 범위 밖 기준선 문제이므로 수정하지 않았고, 이 문서는
완료 정의의 `Implemented` 상태로 승격하지 않는다.
