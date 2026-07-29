# ADR-007: 넓은 브라우저 화면의 적응형 Preview pane

## 상태

Proposed

## 맥락

Mdir4의 Main과 SSH Remote 브라우저에서 선택한 텍스트 파일을 화면 오른쪽에 즉시
확인하고 싶다. Preview는 Settings에서 켜고 끄며, 충분히 넓은 화면에서만 파일 목록과
나란히 표시한다. 화면이 좁아지면 설정을 변경하지 않고 자동으로 숨겨야 한다. Git에서
수정된 텍스트 파일은 일반 본문 대신 unified diff를 기본으로 보여준다. Preview 너비는
마우스가 아니라 단축키로 조절한다.

현재 `LayoutMetrics`는 전체 viewport가 하나의 파일 목록이라는 전제를 가진다. render와
navigation은 같은 metrics를 사용하므로 UI만 오른쪽을 덮는 방식은 선택 좌표, page 용량과
표시 열을 어긋나게 한다. ADR-002는 panel split이 도입될 때 별도 결정을 요구한다.

현재 Local은 bounded file read, `ViewerState`의 UTF-8 판정과 Git diff backend를 가지고
있다. Remote는 byte-preserving path와 `read_dir`만 지원하며 file read와 Remote worktree의
Git status/diff는 아직 없다. Remote라는 이유로 Preview 전체를 비활성화해서는 안 된다.

## 요구사항과 비목표

### 요구사항

- Preview on/off는 Settings에서 관리한다.
- 기본값은 On이며, 오른쪽 50%를 사용한다.
- Preview 너비는 단축키로 5%씩 조절하고 재시작 뒤에도 유지한다.
- Preview는 terminal 폭이 120 columns 이상일 때만 표시한다.
- 좁아져 자동으로 숨겨져도 enabled와 width 설정은 유지한다.
- Local과 Remote 모두 동일한 표시 규칙을 사용한다.
- UTF-8 또는 UTF-8 BOM 텍스트 파일만 본문을 표시한다.
- Local/Remote Git status가 Modified인 텍스트 파일은 본문 대신 unified diff를 표시한다.
- 모든 file, Git과 Remote I/O는 Effect와 background lane에서 실행한다.

### 비목표

- Preview pane 자체의 focus, 검색, 독립 scroll 또는 편집
- Markdown 렌더링, syntax highlighting, 이미지/Hex/binary Preview
- 마우스 drag resize
- Preview 안의 side-by-side Git diff
- Added, Deleted, Renamed, Copied, Untracked 파일의 diff 의미 확장
- Remote라는 이유만으로 Preview를 끄는 정책

## 검토한 선택지

| 선택 | 장점 | 단점 |
|---|---|---|
| UI가 기존 목록 위에 Preview를 덮어 그림 | 변경량이 작음 | render와 navigation geometry가 달라져 ADR-002 위반 |
| `LayoutMetrics`에 Preview 필드를 직접 추가 | 타입 수가 적음 | 파일 목록 geometry와 workspace split 책임이 결합됨 |
| 상위 `WorkspaceLayout`이 Browser와 Preview rect를 계산 | 기존 목록 계산 재사용, 단일 geometry 유지 | 한 단계의 layout 타입과 translation이 추가됨 |
| Preview를 별도 full-screen `Screen`으로 구현 | Viewer 재사용이 쉬움 | 요청한 동시 파일 목록/Preview 경험이 아님 |

## 결정

### 1. 상위 workspace geometry

`LayoutMetrics` 위에 다음 순수 계산 결과를 둔다.

```rust
pub struct WorkspaceLayout {
    pub browser: LayoutMetrics,
    pub preview: Option<Rect>,
}

pub struct PreviewLayoutSettings {
    pub enabled: bool,
    pub width_percent: u8,
}
```

`calculate_workspace(viewport, layout_settings, preview_settings, entry_count, long_view)`가
render와 navigation이 공유하는 단일 원본이다.

- `enabled == false` 또는 `viewport.width < 120`이면 `preview`는 `None`이고 browser는 기존
  전체 viewport를 사용한다.
- Preview가 보이면 width는 `35..=65`로 clamp한 비율을 사용한다.
- Browser와 Preview는 각각 최소 60 columns를 확보한다. 120 columns에서는 실제 비율과
  관계없이 60/60으로 clamp한다.
- Preview의 왼쪽 border cell이 시각적 separator이므로 별도 1-column gap을 만들지 않는다.
- 기존 column/page 계산은 전체 viewport가 아니라 `browser` rect에서 수행한다.
- rect의 origin을 보존하도록 layout 계산을 `Rect` 기반으로 일반화하고, 기존 Viewport API는
  전체 화면 wrapper로 유지한다.

Preview 표시 여부는 파생 결과다. resize는 persisted setting을 변경하거나 content를
삭제하지 않는다. 다시 넓어졌을 때 target identity가 같으면 cache를 즉시 재사용한다.

### 2. 설정과 상태

Config에는 다음 fragment를 추가한다.

```rust
#[serde(default)]
pub struct PreviewConfig {
    pub enabled: bool,
    pub width_percent: u8,
}
```

기본값은 `true`, `50`이다. Settings 화면은 enabled만 노출한다. `width_percent`는 command로
변경하지만 `config_from_state`에 포함되어 정상 종료 또는 다음 config save에서 보존된다.
잘못된 외부 TOML 값은 load 시 `35..=65`로 clamp한다.

AppState에는 설정과 비동기 결과 identity를 소유하는 `PreviewState`를 둔다.

```rust
pub struct PreviewState {
    pub enabled: bool,
    pub width_percent: u8,
    pub generation: u64,
    pub target: Option<PreviewTarget>,
    pub content: PreviewContent,
}

pub enum PreviewTarget {
    Local(PathBuf),
    Remote { alias: SshHostAlias, path: RemotePath },
}

pub enum PreviewContent {
    Empty,
    Loading,
    Text(ViewerState),
    Diff(ViewerState),
    Unsupported,
    TooLarge,
    Error(String),
}
```

`PreviewTarget`은 display string이 아니라 Local/Remote identity를 보존한다. Remote invalid
UTF-8 path도 SFTP text Preview target으로 사용할 수 있다.

### 3. reducer와 Effect 흐름

render는 immutable state만 읽는다. 선택, directory, Git status, Settings apply 또는 resize
후 Preview target이 달라졌는지 reducer가 조정한다.

현재 큰 reducer의 모든 선택 Action에 Preview 분기를 복제하지 않는다. public `reduce`가
기존 로직을 `reduce_inner`로 실행한 뒤 `reconcile_preview`를 호출한다.

```text
Action
  -> reduce_inner(state, action) -> Vec<Effect>
  -> reconcile_preview(state, effects)
  -> Preview load Effect 추가 또는 stale state 정리
```

`reconcile_preview`도 reducer 내부의 순수 상태 전이이며 I/O를 호출하지 않는다. target이
바뀌면 generation을 증가시키고 Loading으로 전환한다. completion은 generation과 정확한
target이 모두 일치할 때만 적용한다. 아직 시작하지 않은 같은 view의 Preview read는 최신
generation으로 coalesce한다.

### 4. 텍스트 판정과 크기 제한

Preview read 상한은 `PREVIEW_MAX_BYTES = 1 MiB`로 고정하며 사용자 설정으로 노출하지 않는다.
adapter는 metadata만 신뢰하지 않고 최대 `limit + 1` byte까지 bounded read한 뒤 TooLarge를
판정한다.

텍스트 판정은 Local의 외부 `file` 명령에 의존하지 않는다. Local과 Remote 모두
`ViewerState::decode`의 다음 규칙을 사용한다.

- NUL byte가 있으면 Binary/Unsupported
- UTF-8 BOM은 제거
- 나머지가 valid UTF-8이면 Text
- invalid UTF-8이면 Binary/Unsupported

directory, parent, symlink와 other entry는 읽지 않고 `Unsupported`를 표시한다. 지원하지
않는 target을 선택해도 pane 자체는 유지해 파일 선택마다 layout이 흔들리지 않게 한다.

### 5. Git diff 우선순위

Git diff보다 먼저 선택 파일이 text인지 확인한다. Modified 상태라도 binary나 TooLarge이면
diff를 요청하지 않는다.

```text
bounded file read
  -> non-text: Unsupported/TooLarge
  -> text + Clean/unknown Git status: Text
  -> text + Modified: diff Effect
       -> non-empty diff: Diff
       -> empty/error: cached Text fallback + non-blocking message
```

Local은 기존 `git_modified_paths`와 `LoadGitDiffForPath` backend를 재사용한다. full-screen
Git diff renderer에서 unified diff의 line/style 생성만 작은 순수 helper로 추출하고 Preview
renderer와 공유한다. Preview는 full-screen Git diff의 origin, search, side-by-side 상태를
변경하지 않는다.

초기 계약은 현재 browser cache가 의미 있게 제공하는 `GitStatus::Modified`만 포함한다.
다른 status의 표시 의미는 별도 계약과 테스트 없이 확대하지 않는다.

### 6. Remote capability

Remote 여부는 visibility 조건이 아니다. Remote backend에 bounded file read capability를
추가하고 해당 Location lane에서 실행한다.

```rust
pub trait RemoteReadBackend: Send + Sync {
    fn read_dir(&self, path: &RemotePath) -> Result<RemoteDirectoryListing, RemoteReadError>;
    fn read_file(
        &self,
        path: &RemotePath,
        max_bytes: usize,
    ) -> Result<Vec<u8>, RemoteReadError>;
}
```

SFTP adapter는 `OPEN -> READ -> CLOSE`를 사용하고 error, TooLarge, deadline과 cancel에서도
handle을 정리한다. UI state는 SFTP handle/process/stream을 소유하지 않는다.

Remote Git status/diff는 Git built-in의 별도 read backend가 해당 SSH Location lane을 통해
실행한다. Remote Git capability가 없거나 repository가 아니면 text Preview로 fallback한다.
Git unavailable은 Preview unavailable과 같지 않다. Git command로 안전하게 표현할 수 없는
byte path는 diff만 생략하고 SFTP text Preview는 유지한다.

### 7. 입력과 렌더

기본 command는 다음과 같다.

- `Alt+[` — Preview width 5% 감소
- `Alt+]` — Preview width 5% 증가

두 command는 CommandRegistry에 등록해 custom keymap 대상이 되게 한다. Main과 Remote의
screen별 이동 mapping보다 먼저 처리한다. Preview가 자동 숨김 상태에서도 비율은 변경할 수
있고 message bar에 다음 표시 비율을 알린다.

Preview는 focus를 갖지 않는다. header에는 선택 path와 `TEXT`, `DIFF`, `BINARY`, `TOO LARGE`,
`ERROR` 중 mode를 표시하고 body는 첫 visible line부터 그린다. 기존 Viewer의 document line
추출과 text cell truncation helper를 재사용한다.

Settings, Help와 Main 위 dialog는 underlying Main workspace의 split을 유지한다. Viewer,
Editor, Git, MCD, Favorites, Amazon Build 같은 full-screen mode에는 Preview를 합성하지 않는다.
Remote screen에는 Main과 같은 workspace split을 적용한다.

## 결과와 트레이드오프

### 장점

- render와 navigation이 같은 browser geometry를 사용한다.
- Local/Remote가 동일한 Preview state와 text 판정을 공유한다.
- resize, 빠른 선택 이동과 늦은 I/O completion이 결정적으로 처리된다.
- Remote Git이 없어도 Remote text Preview는 사용할 수 있다.
- 기존 Viewer와 Git diff 모델을 복제하지 않고 재사용한다.

### 비용

- 모든 browser navigation call site가 상위 workspace geometry를 사용하도록 바뀐다.
- 선택 이동이 background read를 발생시키므로 coalescing과 stale-result 검증이 필요하다.
- Remote file read와 Remote Git read backend가 현재 Remote 범위를 확장한다.
- 120 columns에서는 60/60 최소 폭 때문에 width shortcut의 실제 표시 변화가 없을 수 있다.

### 완화

- layout 순수 테스트를 content I/O보다 먼저 완료한다.
- Preview 전용 1 MiB 상한과 latest-generation coalescing을 사용한다.
- Remote Git 실패는 text Preview로 fallback하고 pane 전체 오류로 승격하지 않는다.
- width command message에 요청 비율과 실제 clamp 여부를 표시한다.

## 재검토 조건

- Preview에 focus, 독립 scroll/search 또는 edit가 필요할 때
- Markdown/image/Hex 같은 복수 renderer가 필요할 때
- 두 개 이상의 보조 pane이나 자유 panel 배치가 필요할 때
- 측정 결과 1 MiB read 또는 120-column threshold가 실제 사용성을 해칠 때
- Added/Untracked/Deleted 등 Modified 외 Git status의 Preview 계약을 추가할 때
