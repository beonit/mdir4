# ADR-004: Git을 위한 최소 built-in 확장 경계

- 상태: 승인
- 날짜: 2026-07-25
- 적용 시점: v1.0 `R1` 완료 후 `G0`

## 배경

Git 상태와 작업을 Mdir4에 추가하려면 파일 목록 장식, 상태바, 명령, 전체 화면 뷰,
background 작업이 필요하다. 이를 `FileEntry`나 core reducer에 Git 전용 필드와 분기로
직접 넣으면 파일 관리자 핵심이 Git 구현에 종속된다. 반대로 첫 기능부터 안정적인
외부 플러그인 ABI, DLL 또는 WASM 호스트를 만들면 배포·보안·버전 호환성 문제가 실제
Git 가치보다 먼저 커진다.

기존 ADR-001과 ADR-003은 상태 전이와 외부 I/O를 reducer/effect/worker로 분리하도록
정했다. 첨부 초안의 동기식 `on_directory_changed`가 직접 저장소를 검색하는 방식은 이
원칙과 충돌한다.

## 결정

1. Git은 v1.0 이후 같은 Cargo 패키지와 실행 파일에 정적으로 포함되는 built-in
   plugin으로 구현한다.
2. Core에는 Git 타입이 아니라 작은 `Plugin`, `PluginManager`, contribution 타입만 둔다.
   `FileEntry`에는 Git 상태 필드를 추가하지 않는다.
3. Plugin callback은 상태 변경과 캐시 조회만 수행한다. 저장소 탐색, status, diff와
   모든 변경 명령은 `PluginEffect`를 통해 worker에서 실행하고 결과 event로 돌아온다.
4. 첫 API는 Git에 실제로 필요한 장식, 상태 항목, command, view, worker job만 지원한다.
   미래 플러그인을 예상한 범용 서비스 컨테이너나 안정 ABI는 만들지 않는다.
5. 동적 Rust library는 ABI 안정성 때문에 사용하지 않는다. DLL/WASM/process/RPC
   외부 플러그인은 Git built-in API가 안정화된 뒤 별도 ADR로만 검토한다.
6. Git backend는 plugin 내부 port다. UI와 core는 `gix`, `git2`, `git` CLI를 직접
   import하거나 실행하지 않는다.

## 결과

### 장점

- Git을 끄면 manager가 contribution과 job을 만들지 않아 기존 탐색 경로가 유지된다.
- Fake backend로 UI와 상태 전이를 실제 저장소 없이 결정적으로 검증할 수 있다.
- backend와 향후 외부 플러그인 방식의 선택을 Git UI에서 분리할 수 있다.
- plugin 오류를 사용자 메시지로 격리하고 파일 탐색을 계속할 수 있다.

### 비용과 제한

- 동적 설치/제거와 third-party plugin 배포는 지원하지 않는다.
- `Plugin` API는 초기에는 내부 API이며 호환성을 보장하지 않는다.
- full-screen plugin view와 worker job을 위한 host 경계가 추가된다.
- 추상화는 Git이 실제로 요구하는 순서로만 늘려야 하며 빈 plugin skeleton을 만들지 않는다.

## 검토한 대안

### Core에 Git을 직접 구현

초기 코드는 적지만 `FileEntry`, reducer, renderer, 설정이 Git에 결합되고 비활성화 시
격리 보장이 약해져 기각했다.

### 처음부터 WASM 또는 process plugin

외부 확장성은 좋지만 capability, sandbox, IPC, API versioning, 배포를 먼저 해결해야
하므로 현재 범위를 초과해 기각했다.

### Rust dynamic library

Rust ABI와 dependency/toolchain 호환성을 공개 계약으로 만들기 어려워 기각했다.

## 재검토 조건

- 두 번째 built-in plugin이 Git용 API로 구현되지 않을 때
- Git plugin이 core private type을 반복적으로 요구할 때
- 사용자가 외부 plugin 설치를 실제 배포 요구사항으로 승인할 때
- worker 격리만으로 plugin fault를 충분히 막지 못한다는 증거가 생길 때

