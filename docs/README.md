# Mdir4 문서 지도와 작업 순서

이 파일은 프로젝트 문서의 단일 진입점이다. 기능 계약, 구현 순서, 후속 확장을 섞어
읽지 않도록 현재 작업 트랙과 문서 우선순위를 고정한다.

플랫폼 범위는 Linux와 macOS다. Windows는 v1, Git built-in, SSH Remote Drive의 구현·수동
검증·출시 blocker에 포함하지 않는다. `requirements-original.md`의 Windows 문구는 원본 기록
보존용이며 현재 제품 계약보다 우선하지 않는다.

## 1. 현재 작업 순서

```text
M0 개발 기반
 └─ M0-05 저장소 기준선         완료
     └─ M1-13 계약 정합성 보정  완료
         └─ M2 파일 관리        완료
             └─ M3 기능 확장    완료
                 └─ R1-01 Linux/macOS RC 빌드/패키징  완료
                     └─ R1-02 실제 환경 수동 시험    다음 작업
                     ├─ G0→G1→G2→G3 Git built-in
                     └─ S0→S1→S2→S3 SSH Remote Drive
```

- 현재 활성 다음 카드는 `R1-02 고정 RC Linux/macOS 실제 환경 수동 시험`이다.
- Git과 SSH Remote는 제품 범위와 승인 조건이 독립적이다. 먼저 구현한 트랙은 generic
  unsupported fixture와 source-boundary test로 상대 트랙 부재를 검증하고 완료할 수 있다.
  둘 다 존재하는 시점부터 실제 Local/Remote 양방향 integration 행이 필수가 된다.
- Git `G3`의 “remote”는 Git fetch/push 네트워크를 뜻하고, SSH Remote Drive `S0~S3`와
  다른 기능이다.
- 아직 시작하지 않은 후속 기능을 M2/M3에 끼워 넣지 않는다.

## 2. v1 문서 읽기 순서

1. [`implementation-plan/01-product-contract.md`](implementation-plan/01-product-contract.md)
   — 사용자에게 보이는 확정 동작과 안전 정책
2. [`implementation-plan/progress.md`](implementation-plan/progress.md)
   — 현재 기준선, 첫 미완료 카드, 실제 실행 증거
3. 관련 [`architecture/`](architecture/) ADR과
   [`implementation-plan/02-architecture.md`](implementation-plan/02-architecture.md)
   — 타입, port, reducer/effect, 의존 방향
4. [`implementation-plan/03-task-cards.md`](implementation-plan/03-task-cards.md)
   — 선행 관계가 있는 실제 구현 카드
5. [`implementation-plan/04-test-plan.md`](implementation-plan/04-test-plan.md)
   — fixture, scenario, snapshot, fault 검증 규칙
6. [`implementation-plan/05-acceptance-matrix.md`](implementation-plan/05-acceptance-matrix.md)
   — 요구사항과 완료 증거 연결
7. [`implementation-plan/06-agent-runbook.md`](implementation-plan/06-agent-runbook.md)
   — 한 카드 선택·구현·검증·기록 절차

현재 구현을 이어갈 때는 `progress.md`의 첫 미완료 카드를 확인한 뒤 그 카드와 직접 관련된
ADR/아키텍처 절만 읽는다. 완료 카드 전체를 다시 구현하지 않는다.

## 3. 후속 트랙

| 트랙 | 시작 조건 | 제품 계약 | 카드 | 수용 기준 |
|---|---|---|---|---|
| Git built-in `G0~G3` | R1 완료; S0가 있으면 실제 Remote integration도 실행 | [`plugins/git/01-product-contract.md`](plugins/git/01-product-contract.md) | [`plugins/git/03-task-cards.md`](plugins/git/03-task-cards.md) | [`plugins/git/05-acceptance-matrix.md`](plugins/git/05-acceptance-matrix.md) |
| SSH Remote `S0~S3` | R1 완료 | [`remote/01-product-contract.md`](remote/01-product-contract.md) | [`remote/03-task-cards.md`](remote/03-task-cards.md) | [`remote/05-acceptance-matrix.md`](remote/05-acceptance-matrix.md) |

후속 트랙에서도 해당 폴더의 `README → 제품 계약 → 아키텍처 → 카드 → 테스트 → 수용표`
순서를 사용한다.

M4 이후 backlog에는 MCD viewport, 외부 `$EDITOR` 연결, Rename 중간 커서 편집이 기록되어
있다. 이 항목들은 현재 릴리스 작업을 선점하지 않는다.

## 4. 참고 문서와 규범성

주요 ADR 빠른 색인:

| ADR | 적용 범위 |
|---|---|
| [`ADR-001`](architecture/adr-001-reducer-effect.md) | reducer/effect와 I/O 분리 |
| [`ADR-002`](architecture/adr-002-shared-layout-navigation.md) | 적응형 column, `│`, page navigation 단일 geometry |
| [`ADR-003`](architecture/adr-003-worker-model.md) | v1 Core Local 단일 worker |
| [`ADR-004`](architecture/adr-004-built-in-plugin-boundary.md) | post-v1 built-in plugin 경계 |
| [`ADR-005`](architecture/adr-005-background-work-lanes.md) | M2/Git/Remote bounded lane, mutation lease, cancel/lifecycle |
| [`ADR-006`](architecture/adr-006-remote-location-foundation.md) | Remote Location identity와 SFTP 기반 경계 |
| [`ADR-007`](architecture/adr-007-adaptive-preview-pane.md) | 넓은 Main/Remote 화면의 적응형 Preview pane |

제안된 Preview pane의 설계 적용 순서와 카드별 완료 조건은
[`preview-pane-implementation-plan.md`](preview-pane-implementation-plan.md)에 있다. 이 트랙은
승인 전까지 현재 활성 `R1-02` 작업을 선점하지 않는다.

제안된 `Ctrl+L` Project Locate의 path index, cache/SLA와 후속 Symbol/ctags/Full-text 확장
순서는 [`locate-search-implementation-plan.md`](locate-search-implementation-plan.md)에 있다.
이 트랙도 `LOC-00` 승인 전까지 현재 활성 릴리스 작업을 선점하지 않는다.

제안된 full-screen View mode의 syntax highlighting, malformed source fallback, theme/search
합성과 worker 경계는
[`viewer-syntax-highlighting-implementation-plan.md`](viewer-syntax-highlighting-implementation-plan.md)에
있다. 이 트랙은 `SH-00` 승인 전까지 dependency나 source 변경을 시작하지 않는다.

Git/Viewer 등 full-screen mode가 Browser Preview geometry를 잘못 공유하는 문제의 근본
수정과 재발 방지 테스트 계획은
[`screen-layout-refactor-plan.md`](screen-layout-refactor-plan.md)에 있다.

| 문서 | 역할 | 규범성 |
|---|---|---|
| [`requirements-original.md`](requirements-original.md) | 최초 요구사항 원문 | 계약에 반영되기 전 참고 |
| [`spec-review.md`](spec-review.md) | 원문 충돌과 해소 결정 기록 | 결정 상태 확인용 |
| [`ui-reference/README.md`](ui-reference/README.md) | Mdir III 화면 자료 | 계약이 없는 시각 판단 참고 |
| [`architecture/`](architecture/) | 승인된 ADR | 해당 결정 범위에서 계약 다음 우선 |
| [`development.md`](development.md) | 도구 설치와 개발 명령 | 실행 절차 |

보존한 Core 원문 `requirements-original.md`의 SHA-256은
`fad8c711c46014ba943bbee604df6be286b8f22e1122186f9be0c9a0ad5d6930`이다. Git/Remote
원문과 UI 자료 hash는 각 트랙/참고 폴더 README에 기록한다.

우선순위:

1. 활성 트랙의 제품 계약
2. 승인된 ADR
3. 활성 트랙의 아키텍처와 작업 카드
4. 수용 기준과 테스트 계획
5. 원본 요구사항과 UI 참고 자료
6. 구현자의 추측

문서와 현재 코드가 다르면 이미 완료된 기능은 코드+자동 테스트를 증거로 계약/진행표를
즉시 갱신한다. 아직 구현하지 않은 기능은 제품 계약을 기준으로 구현한다.

## 5. 변경 규칙

- 사용자 동작, 키, 레이아웃, 삭제/인증 정책 변경: 제품 계약과 수용 기준을 먼저 수정.
- 공개 타입/worker/port/경계 변경: ADR 필요 여부를 검토하고 아키텍처 문서 수정.
- 카드 범위 변경: 선행 관계, 테스트, `progress.md`를 함께 수정.
- 후속 트랙 추가: v1 범위를 바꾸지 말고 별도 폴더와 단계 prefix를 사용.
- 역사적 test 수치는 보존하되 현재 기준선은 `progress.md` 상단에 한 번만 명시.
