# ADR-002: 렌더와 탐색이 하나의 LayoutMetrics 사용

## 상태

Accepted

## 맥락

Mdir4의 Short View는 항목을 한 열씩 위에서 아래로 채운 뒤 다음 열로 이동한다. terminal
폭, 목록 높이, column width profile, 항목 수와 고정/자동 설정에 따라 열 수와 page 용량이
달라진다. 렌더와 방향키가 다른 공식을 사용하면 다음 오류가 발생한다.

- 화면에 보이는 셀과 선택 index가 어긋난다.
- resize 또는 79/80/81열 경계에서 cursor가 건너뛴다.
- 항목이 적은데 좁은 열만 사용해 오른쪽 공간을 버린다.
- page 끝에서 방향키가 멈추거나 다른 row로 이동한다.
- separator가 파일명 폭을 침범하거나 빈 row에서 끊겨 보인다.

## 고려한 선택지

| 선택 | 장점 | 단점 |
|---|---|---|
| render와 reducer가 각자 계산 | 각 모듈 코드가 짧아 보임 | 경계값 불일치와 off-by-one 위험 |
| 렌더된 buffer 셀을 역으로 탐색 | 화면과 항상 일치 | 상태 로직이 Ratatui buffer에 결합 |
| 순수 LayoutEngine 결과 공유 | 결정적이고 UI 독립 | metrics 타입을 안정적으로 관리해야 함 |

## 결정

`calculate_for_entries(viewport, settings, entry_count) -> LayoutMetrics`를 레이아웃의 단일
원본으로 사용한다. navigation과 render는 독자 공식을 갖지 않는다. 현재처럼 두 시점에
다시 계산하더라도 동일 state 입력에는 동등한 metrics가 나와야 하며, 이후 필요하면 한
event cycle의 결과를 cache할 수 있다.

```rust
pub struct LayoutMetrics {
    pub viewport: Rect,
    pub path_bar: Rect,
    pub list: Rect,
    pub item_detail: Rect,
    pub directory_summary: Rect,
    pub message_bar: Rect,
    pub function_bar: Rect,
    pub columns: Vec<Rect>,
    pub rows_per_column: usize,
    pub page_capacity: usize,
    pub too_small: bool,
}
```

## 자동 열 수 공식

too-small이 아니면 다음 순서로 계산한다.

1. `rows = list.height`로 둔다.
2. 선택한 width profile과 viewport 폭으로 `max_columns_by_width`를 구한다.
3. 각 열이 `MIN_COLUMN_WIDTH` 미만이면 열 수를 줄인다.
4. Auto mode에서 항목 수가 요구하는 열 수를 계산한다.

```text
required_columns = ceil(max(entry_count, 1) / rows)
effective_columns = min(max_columns_by_width, required_columns)
page_capacity = rows * effective_columns
```

`rows == 0`인 too-small 상태에서는 열과 page capacity를 0으로 하고 나눗셈하지 않는다.
Auto mode의 결과는 최소 한 열이다.

이 공식 때문에 `entry_count <= rows`이면 정확히 한 열이며 그 열이 목록 전체 너비를
사용한다. 항목 수가 늘면 두 열, 세 열 순으로 필요한 만큼만 늘어나고 폭이 허용하는 최대
열 수에서 멈춘다. Fixed mode는 항목 수에 따라 열 수를 줄이지 않으며, 최소 열 폭을
지키기 위한 하향 조정만 허용한다.

열 Rect는 목록 전체를 빈틈과 겹침 없이 정확히 분할한다. 나머지 폭은 앞쪽 열부터 한
셀씩 배분하며 마지막 열의 right edge는 항상 list right edge와 같다.

## 열 separator 계약

두 열 이상일 때 각 **마지막 열을 제외한 열 Rect의 마지막 한 셀**은 separator 전용이다.

```text
content_width = column.width - 1   // non-last column
content_width = column.width       // last column
separator = '│'                    // U+2502 BOX DRAWINGS LIGHT VERTICAL
```

separator는 ASCII pipe `|`가 아니라 box-drawing 문자 `│`다. filename/size 포맷은
`content_width`만 사용하므로 separator cell을 덮어쓰지 않는다. separator는 개별 entry
문자열에 붙이지 않고 column 높이 전체에 widget border로 렌더한다. 따라서 항목이 없는
row에서도 path bar 아래부터 item-detail bar 위까지 연속된다.

## index와 화면 좌표

목록은 vertical-first다. 유효한 metrics에서 index `i`의 위치는 다음과 같다.

```text
page_start = floor(i / page_capacity) * page_capacity
local      = i - page_start
column     = floor(local / rows_per_column)
row        = local % rows_per_column
```

역변환은 다음과 같으며 결과가 `entry_count`보다 작을 때만 유효하다.

```text
index = page_start + column * rows_per_column + row
```

마지막 page의 마지막 열은 짧을 수 있다. 다른 열로 이동할 때 같은 row가 없으면 그 열의
가장 가까운 마지막 row를 사용한다.

## 네 방향 page 경계 이동

page 경계는 탐색의 끝이 아니다. 다만 Up/Down은 열 안의 세로 이동이고 Left/Right는 열
사이의 가로 이동이므로 page 경계가 아닌 열 가장자리에서 임의로 다음/이전 열로 wrap하지
않는다.

- **Up**: 같은 열에서 이전 row로 이동한다. 현재 index가 page의 첫 항목이면 이전 page의
  마지막 항목으로 이동한다. 다른 열의 첫 row에서는 현재 항목을 유지한다.
- **Down**: 같은 열에서 다음 row로 이동한다. 현재 index가 page의 마지막 항목이고 다음
  page가 있으면 다음 page의 첫 항목으로 이동한다. 마지막 열이 아닌 열의 마지막 row에서는
  현재 항목을 유지한다.
- **Left**: 같은 page의 이전 열에서 가능한 가장 가까운 row로 이동한다. 첫 열에서 이전
  page가 있으면 이전 page의 마지막 열, 가장 가까운 row로 이동한다.
- **Right**: 같은 page의 다음 열에서 가능한 가장 가까운 row로 이동한다. 마지막 열에서
  다음 page가 있으면 다음 page의 첫 열, 가장 가까운 row로 이동한다.

Up/Down의 page crossing은 연속 index 흐름을 유지하고 Left/Right의 page crossing은 row를
가능한 한 보존한다. 첫 항목의 Up/Left와 마지막 항목의 Down/Right는 현재 항목을 유지한다.

PageUp/PageDown은 `page_capacity`만큼 index를 이동하고 `[0, entry_count - 1]`로 clamp한다.
Home/End는 각각 첫 항목과 마지막 항목으로 이동한다. 빈 목록 또는 capacity 0은 언제나
index 0을 반환하며 panic하지 않는다.

## resize와 identity

Resize Action을 먼저 state에 적용한 뒤 새 metrics를 계산한다. resize는 선택된 entry의
경로 identity와 marked set을 바꾸지 않고 좌표와 page만 재계산한다. navigation과 render가
각각 계산할 때도 같은 최신 state/viewport를 입력으로 사용한다.

## 테스트 의무

다음은 이 ADR의 필수 회귀다.

- 79/80/81, 119/120/121, 159/160/161 폭 경계
- 항목 수 `0`, `1`, `rows`, `rows + 1`, `capacity`, `capacity + 1`
- Auto 한 열 전체 폭, 필요 열 증가, Fixed mode 불변
- 열 Rect 전체 폭 보존과 remainder 배분
- `│` separator가 모든 list row에서 동일 x를 점유하고 entry가 덮지 않음
- 짧은 마지막 열로 Left/Right 이동 시 nearest-row clamp
- Up/Down/Left/Right의 이전/다음 page crossing
- resize 전후 selected entry identity와 mark 보존
- too-small과 0 capacity에서 panic 없음

## 근거

- 한 순수 함수에 경계 테스트를 집중할 수 있다.
- TestBackend 없이 navigation을 빠르게 검증할 수 있다.
- 항목이 적은 일반적인 디렉터리는 가독성을 위해 전체 폭을 사용한다.
- render와 reducer가 같은 page geometry를 사용하므로 화면과 cursor가 어긋나지 않는다.

## 감수하는 비용

- `entry_count`도 layout 입력이므로 listing 변경마다 metrics를 다시 계산한다.
- separator 전용 한 셀만큼 non-last column의 내용 폭이 줄어든다.
- viewport와 view 설정이 AppState에 있어야 한다.

계산 비용은 열 수가 최대 여섯 개이고 정수 연산뿐이므로 허용한다.

## 재검토 조건

panel split, 자유 크기 widget 또는 horizontal scrolling이 도입되어 하나의 Short View
geometry로 표현할 수 없을 때 별도 ADR로 재검토한다. 그 전에는 render나 navigation에
독자적인 열/page 공식을 추가하지 않는다.
