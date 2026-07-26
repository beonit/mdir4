# Mdir4 구현 계획

이 폴더는 구현 담당자가 프로젝트 배경을 추측하지 않고, 작은 작업 하나를 골라
완료할 수 있도록 만든 실행 계획이다.

전체 트랙과 현재 활성 단계는 먼저 [`../README.md`](../README.md)에서 확인한다.

## 문서 읽는 순서

1. [`01-product-contract.md`](01-product-contract.md) — 확정된 동작과 범위
2. [`progress.md`](progress.md) — 현재 기준선, 첫 미완료 카드, 실제 증거
3. 관련 [`../architecture/`](../architecture/) ADR와
   [`02-architecture.md`](02-architecture.md) — 현재/목표 모듈, 타입, 의존 방향
4. [`03-task-cards.md`](03-task-cards.md) — 해당 활성 카드의 순서와 완료 조건
5. [`04-test-plan.md`](04-test-plan.md) — 테스트 작성 규칙과 필수 케이스
6. [`05-acceptance-matrix.md`](05-acceptance-matrix.md) — 요구사항 추적표
7. [`06-agent-runbook.md`](06-agent-runbook.md) — 에이전트 작업 절차

원본 요구사항은 [`../requirements-original.md`](../requirements-original.md), 충돌 검토는
[`../spec-review.md`](../spec-review.md), 화면 자료는
[`../ui-reference/README.md`](../ui-reference/README.md)에 있다.

v1.0 이후의 내장 Git 확장 계획은 [`../plugins/git/README.md`](../plugins/git/README.md),
SSH Remote/Remote Drive 계획은 [`../remote/README.md`](../remote/README.md)에
있다. 두 트랙은 현재 `M0~R1`의 완료 조건을 변경하지 않는다.

## 문서 우선순위

서로 다른 설명이 충돌하면 다음 순서로 판단한다.

1. `01-product-contract.md`의 명시적 결정
2. 승인된 ADR
3. 활성 트랙의 `02-architecture.md`와 `03-task-cards.md`
4. `05-acceptance-matrix.md`와 `04-test-plan.md`
5. `requirements-original.md`
6. UI 참고 이미지
7. 구현자의 추측

추측으로 행동해야 하는 상황이면 구현하지 말고 `progress.md`의 `결정 필요`에 기록한다.

## 프로젝트 분류와 제약

- 제품 유형: 로컬 단일 사용자 Linux/macOS TUI
- 첫 배포: Linux 및 macOS native architecture 단일 실행 파일
- 구조: 단일 Cargo 패키지의 모듈형 모놀리스
- 핵심 제약: 80×25, 키보드 전용 조작, 결정적 TestBackend 렌더링
- Windows는 v1 지원·검증 범위에 포함하지 않는다.
- 현재 환경: Rust stable 1.97.1 도구 체인과 Cargo/rustfmt/Clippy가 설치되어 있다.
- v1 범위 밖: 2패널, 압축 탐색, SFTP/SSH, Git 통합, 범용/외부 플러그인,
  내장 터미널, Hex Viewer
- 후속 계획: Rust trait 기반 Git built-in `G0~G3`과 Location 기반 SSH Remote
  `S0~S3`. DLL/WASM 기반 외부 플러그인은 포함하지 않는다.

## 단계와 릴리스 의미

`M0~M3`는 내부 마일스톤이다. 원본 문서의 “MVP 완료”는 M1만을 뜻하지 않고
M0~M3를 모두 끝낸 `v1.0`으로 정의한다.

| 단계 | 목표 | 종료 조건 |
|---|---|---|
| M0 | 개발 기반 | Rust/Cargo, 기본 앱, CI, 품질 명령 정상 |
| M1 | 화면 및 탐색 | 실제 디렉터리를 80×25에서 탐색하고 스냅샷 통과 |
| M2 | 파일 관리 | Rename/View/Edit/Copy/Move/MkDir/Delete가 테스트 더블과 실제 임시 폴더에서 동작 |
| M3 | Mdir 확장 | MCD/QCD/Menu/Long View/설정/사용자 키맵·테마 |
| R1 | v1.0 릴리스 | Linux/macOS 빌드, 전체 수용 기준, 문서와 패키징 완료 |
| G0 | 내장 확장 기반 | 일반화된 최소 Plugin API, Manager, 설정, 격리 테스트 |
| G1 | 읽기 전용 Git | 저장소/브랜치/상태/Diff를 비동기로 표시 |
| G2 | 로컬 Git 작업 | Stage/Commit/Log/Branch/Stash/Discard 안전 처리 |
| G3 | 원격 Git 작업 | 별도 설계 승인 후 Fetch/Pull/Push/인증/충돌 처리 |
| S0 | Location 기반 | Local/Remote identity, config, fake, transport ADR |
| S1 | Remote 탐색 | SSH alias로 연결해 browse/stat/view/download |
| S2 | Remote Drive MVP | upload/copy/move/rename/mkdir/delete/RO/취소 |
| S3 | Remote 강화 | cache, host browser, 등록 UI, resume/metadata |

## 공통 완료 게이트

모든 작업 카드는 아래 조건을 만족해야 완료다.

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
```

UI가 바뀐 작업은 관련 Insta 스냅샷을 검토해야 한다. 단순히 새 스냅샷을 승인해
테스트를 통과시키면 완료로 인정하지 않는다.

## 계획 변경 규칙

- 공개 타입, 키 배치, 레이아웃 알고리즘, 파일 삭제 정책을 바꾸면 ADR을 추가한다.
- 작업 범위를 넓힐 때는 먼저 수용 기준과 테스트 카드를 갱신한다.
- 한 작업 카드가 1~2일을 넘을 것으로 보이면 기능을 더 작은 카드로 나눈다.
- 다음 마일스톤 기능을 “김에” 구현하지 않는다.
