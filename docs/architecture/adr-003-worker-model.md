# ADR-003: 표준 스레드와 채널 기반 파일 작업

## 상태

Accepted — v1 Core Local 범위; post-v1 lane 확장은 ADR-005

## 맥락

복사, 이동, 삭제, MCD 디렉터리 로드는 UI를 막지 않아야 하지만 v1은 로컬 단일 사용자
TUI이며 네트워크 서비스나 대규모 async 생태계가 필요하지 않다.

## 고려한 선택지

| 선택 | 장점 | 단점 |
|---|---|---|
| UI 스레드에서 동기 실행 | 가장 단순 | 진행 중 입력/렌더 불가능 |
| Tokio async runtime | 취소/동시성 도구 풍부 | 파일 I/O는 여전히 blocking, 복잡도와 의존성 증가 |
| std thread + mpsc | 작은 의존성, blocking FS와 자연스럽게 맞음 | backpressure/structured concurrency를 직접 관리 |

## 결정

UI 스레드와 하나의 **Core Local** I/O worker를 표준 `std::thread`와 채널로 연결한다.
로컬 디렉터리 스캔과 변경 작업은 한 번에 하나의 job만 실행하고 progress를 최대 20 Hz로
전달한다. conflict 선택 응답은 역방향 command 채널로 보낸다. cancel은 blocking worker
queue 뒤에 갇히지 않도록 ADR-005의 queue 밖 thread-safe `CancelHandle`로 신호하고 worker와
adapter가 cooperative하게 관측한다. 변경 작업 중에는 MCD와 다른 로컬 변경 작업을
비활성화한다.

이 결정은 post-v1 Git read나 SSH Remote blocking I/O까지 하나의 전역 queue에 넣으라는
뜻이 아니다. 기능별 bounded lane, 공통 mutation lease와 독립 cancel handle은
[ADR-005](adr-005-background-work-lanes.md)가 확장한다.

## 근거

- 로컬 파일 작업은 blocking API가 중심이다.
- 동시에 여러 변경 작업을 실행할 제품 요구가 없다.
- 단일 작업 정책이 충돌 대화상자와 데이터 안전성을 단순화한다.

## 감수하는 비용

- 앱 종료 시 worker join과 취소 타임아웃을 직접 구현한다.
- 긴 변경 작업 중 새 MCD 탐색을 시작할 수 없다.

## 완화

- read-only MCD load는 동일 worker를 사용하고 변경 작업이 없을 때만 시작한다.
- worker event protocol을 독립 테스트한다.
- 장기 정지는 UI에서 “취소 요청 중”으로 표시하고 강제 종료로 데이터를 훼손하지 않는다.

## 재검토 조건

Core Local 안에서도 승인된 병렬 copy가 필요하거나 현재 단일 lane이 측정된 응답성 목표를
충족하지 못할 때. Git/SSH Remote로 인한 기능별 lane 확장은 ADR-005에서 이미 결정했다.
