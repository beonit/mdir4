# ADR-001: Reducer와 Effect로 상태 변경과 I/O 분리

## 상태

Accepted

## 맥락

키 입력, 파일 시스템, 시간, 디스크 정보와 파일 실행을 실제 OS와 TestBackend에서
동일하게 재현해야 한다. 키 이벤트가 UI를 직접 변경하면 시나리오 테스트와 작업 중
응답성을 보장하기 어렵다.

## 고려한 선택지

| 선택 | 장점 | 단점 |
|---|---|---|
| 이벤트 핸들러에서 상태와 I/O 직접 처리 | 초기 코드가 짧음 | 테스트가 OS에 결합되고 상태 전이가 분산됨 |
| App 객체에 포트를 주입해 메서드에서 동기 처리 | mocking 가능 | 긴 작업이 UI를 막고 순수 전이 검증이 어려움 |
| 순수 reducer + 명시적 Effect | 결정적 테스트, I/O 분리, 비동기 확장 | Action/Effect boilerplate 증가 |

## 결정

모든 기능은 `Action → reduce(&mut AppState) → Vec<Effect>` 흐름을 사용한다. Effect
실행 결과는 completion Action으로 다시 dispatch한다. render는 읽기 전용이다.

## 근거

- 원본 요구사항이 입력/상태/레이아웃/렌더 분리를 명시한다.
- TestBackend와 키 시나리오 재생이 production 경로를 그대로 사용할 수 있다.
- 파일 작업 worker를 추가해도 상태 전이 모델을 바꿀 필요가 없다.

## 감수하는 비용

- 단순 Rename에도 Action/Effect/result 타입이 필요하다.
- 구현자가 reducer 밖에서 상태를 바꾸지 않도록 규율과 테스트가 필요하다.

## 완화

- Action을 화면이 아니라 사용자 의도 중심으로 명명한다.
- 반복 dialog/result 패턴은 공통 타입으로 묶되 범용 프레임워크로 과도하게 추상화하지
  않는다.

## 재검토 조건

Reducer가 수백 개의 unrelated Action으로 비대해져 모듈별 reducer 분리가 필요할 때.
