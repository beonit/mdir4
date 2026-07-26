# 구현 에이전트 실행 절차

이 문서는 프로젝트 지식이 적은 에이전트가 작업 카드를 안전하게 처리하는 절차다.

## 1. 작업 시작 전

반드시 다음 순서로 수행한다.

1. [`../README.md`](../README.md)에서 활성 트랙과 문서 우선순위를 확인한다.
2. 활성 트랙의 `README.md`와 `01-product-contract.md`를 읽는다.
3. `progress.md` 상단의 “다음 구현 카드”와 선행 카드 상태를 확인한다. 단순히 첫 `[ ]`를
   고르지 않는다. `외부 대기` 카드는 외부 상태가 바뀌기 전 반복하지 않는다.
4. 선택한 카드와 관련 `02-architecture.md`, `04-test-plan.md`, 수용 기준 행을 읽는다.
5. `git status --short`로 기존 변경을 확인한다.
6. 첫 commit이 없고 untracked 파일이 있으면 `git diff`가 내용을 보여주지 않는다는 점을
   확인한다. M0-05가 끝나기 전에는 후속 구현을 시작하거나 사용자 승인 없이 commit하지 않는다.
7. 기존 사용자 변경을 삭제하거나 덮어쓰지 않는다.
8. 카드의 목표를 한 문장으로 다시 적고, 수정 예정 파일을 나열한다.

선행 카드가 끝나지 않았거나 요구사항이 충돌하면 코드를 쓰지 말고
`progress.md > 결정 필요`에 구체적으로 기록한다.

## 2. 한 번에 할 일

- 카드 하나만 수행한다.
- 카드에 없는 리팩터링, 라이브러리 교체, 새 기능을 함께 하지 않는다.
- 단, 컴파일을 위해 꼭 필요한 작은 변경은 허용하며 완료 보고에 이유를 쓴다.
- 새 public API가 필요하면 먼저 테스트에서 원하는 사용 형태를 작성한다.
- 파일이 400줄을 넘거나 함수가 60줄을 넘으면 책임 분리를 검토한다. 숫자만 맞추기
  위한 기계적 분리는 하지 않는다.

## 3. 구현 순서

각 카드는 다음 순서로 처리한다.

```text
1. 현재 동작/파일 확인
2. 가장 작은 실패 테스트 작성
3. 해당 테스트 실패 확인
4. 최소 구현
5. 대상 테스트 실행
6. 전체 fmt/clippy/test
7. diff 자체 검토
8. progress 갱신
9. 완료 보고
```

테스트가 먼저 실패하지 않았다면 테스트가 실제 요구사항을 검증하는지 확인한다.

## 4. 표준 명령

빠른 반복:

```text
cargo test --locked <test_name>
```

카드 완료:

```text
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
git diff --check
git status --short
```

UI 카드:

```text
cargo insta test --all-features --check --locked
```

`cargo insta`를 아직 설치하지 않은 단계면 `cargo test` 실패와 `.snap.new` 존재 여부를
확인한다. snapshot 승인 명령을 무조건 실행하지 않는다.

## 5. 파일 작업 안전

- 테스트용 쓰기/삭제는 `TempDir` 또는 MemoryFileSystem만 사용한다.
- 홈, 저장소 루트, 환경 변수로만 결정된 경로, glob을 삭제 대상으로 쓰지 않는다.
- 삭제 전 대상이 test temp root의 자손인지 assert한다.
- 사용자 파일을 대상으로 수동 시험해야 하면 먼저 정확한 경로와 복구 방식을
  사용자에게 알리고 허가를 받는다.
- Copy/Move/Delete 오류를 성공으로 삼키지 않는다.

## 6. 아키텍처 경계 검사

작업 완료 전에 다음을 검색한다.

```text
src/ui.rs와 src/ui/**에서 std::fs, FileSystem, Clock, DiskInfo 호출이 없는가?
model에서 ratatui/crossterm 의존이 없는가?
layout은 geometry용 ratatui::layout::Rect 외 Frame/Widget/Buffer와 crossterm event를 쓰지 않는가?
main.rs에 상태 전이 또는 파일 작업 로직이 없는가?
기능키 문자열이 CommandRegistry 밖에 중복되지 않았는가?
렌더 함수가 &mut AppState를 받지 않는가?
```

경계를 깨야만 구현할 수 있다면 임시 우회 코드를 넣지 말고 ADR 변경을 제안한다.

## 7. 실패 처리

### 컴파일 실패

1. 첫 번째 원인 오류부터 고친다.
2. 연쇄 오류 전체를 한꺼번에 추측하지 않는다.
3. dependency API가 예상과 다르면 설치된 버전의 공식 문서를 확인한다.
4. 버전을 임의로 낮추기 전에 Cargo.lock과 선언된 `package.rust-version`을 확인한다. 현재
   pre-v1은 MSRV를 선언하지 않고 rolling stable을 쓰므로 새 MSRV를 임의로 약속하지 않는다.

### 테스트 실패

1. 실패한 expected/actual과 fixture를 확인한다.
2. 기존 테스트를 삭제하거나 assert를 약하게 만들지 않는다.
3. 제품 계약이 틀렸다고 판단되면 코드가 아니라 결정 문서를 먼저 변경한다.

### Snapshot 실패

1. 문자 diff 확인.
2. foreground/background/modifier diff 확인.
3. 요구된 변화인지 카드와 대조.
4. 무관한 화면 변화가 섞이면 구현을 분리.

### 환경 차이

- OS 의존 테스트는 이유가 명확할 때만 `#[cfg(windows)]`.
- 단순히 CI에서 실패한다는 이유로 ignore하지 않는다.
- Windows adapter와 portable core의 contract test를 분리한다.

## 8. 결정이 필요한 조건

다음은 구현자가 독자적으로 바꾸지 않는다.

- 기본 키 배치
- 80×25 행 배치
- 항목 적응형 Auto Column 공식, `│` 경계와 최소 12셀
- Delete 기본 휴지통 정책
- symlink 미추적 정책
- TOML 설정 형식
- reducer/effect 구조
- 마일스톤 간 기능 이동
- 새로운 runtime(async/Tokio 등) 도입

변경 제안은 다음 형식으로 `progress.md`에 기록한다.

```text
문제:
막힌 카드:
현재 계약:
선택지 A / 장단점:
선택지 B / 장단점:
권장:
결정 전 안전하게 계속할 수 있는 작업:
```

## 9. progress 갱신 형식

카드 완료 시:

```text
- [x] M1-03 LayoutEngine과 컬럼 계산
  - 완료일: YYYY-MM-DD
  - 변경: src/layout/engine.rs, tests/layout_boundaries.rs
  - 검증: cargo test --locked layout_boundaries; 전체 gate
  - 증거: commit SHA 또는 현재 diff
  - 남은 위험: 없음
```

부분 완료는 체크하지 않는다. `진행 메모`에 현재 실패/다음 한 단계를 적는다.

## 10. 최종 완료 보고 템플릿

```text
결과:
- 무엇이 동작하게 되었는지 1~3문장

변경 파일:
- path: 역할

검증:
- 실행한 명령과 결과
- 수동 확인이 있으면 환경과 결과

결정/가정:
- 기존 계약을 그대로 따른 항목
- 불가피하게 추가한 가정

남은 사항:
- 다음 카드
- 알려진 위험 또는 없음
```

## 11. 낮은 기능 에이전트용 요청 템플릿

아래 템플릿에서 트랙 문서와 카드 ID만 바꿔 전달할 수 있다. v1은
`docs/implementation-plan`, Git은 `docs/plugins/git`, SSH Remote는 `docs/remote`다.

```text
docs/README.md에서 현재 활성 트랙을 확인한 뒤,
<TRACK-DOC>/README.md부터 지정된 순서대로 읽고 작업 카드 <CARD-ID> 하나만 구현하라.

규칙:
1. progress.md에서 선행 카드 완료를 먼저 확인한다.
2. 카드에 적힌 파일과 범위를 벗어나지 않는다.
3. 가장 작은 실패 테스트를 먼저 작성하고 실패를 확인한다.
4. docs/implementation-plan/06-agent-runbook.md를 그대로 따른다.
5. fmt, clippy -D warnings, 전체 test를 실행한다.
6. snapshot을 자동 승인하지 않는다.
7. 완료한 경우에만 progress.md 체크박스와 검증 증거를 갱신한다.
8. 결정이 필요하면 추측해 구현하지 말고 정해진 형식으로 기록한다.
9. 저장소 첫 commit은 사용자가 명시적으로 승인한 경우에만 만든다.
```

## 12. 리뷰 에이전트용 체크리스트

- 카드의 수용 기준을 실제로 만족하는가?
- happy path뿐 아니라 명시된 오류/경계 test가 있는가?
- reducer가 Effect 없이 I/O를 호출하지 않는가?
- 동일한 레이아웃 계산을 탐색과 렌더가 공유하는가?
- Unicode 셀 폭과 PathBuf/OsString 경계를 지키는가?
- 실패 시 사용자 데이터가 보존되는가?
- test가 implementation detail이 아니라 관찰 가능한 계약을 검사하는가?
- progress 증거와 실제 명령 결과가 일치하는가?
