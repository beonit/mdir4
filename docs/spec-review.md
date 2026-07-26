# Mdir4 요구사항 충돌 검토와 결정 기록

최종 검토일: 2026-07-25

이 문서는 원본 요구사항과 후속 요구사항에서 발견한 충돌을 기록한다. 아직 결정되지 않은
질문 목록이 아니라, 어떤 계약에서 어떻게 해소했는지 찾는 색인이다. 실제 구현 동작은
각 트랙의 제품 계약이 우선한다.

## 1. 현재 결론

- v1 `M0~R1`을 막는 미결정 사항은 없다.
- 기본 탐색은 동작하고 현재 64 tests/Clippy 0 기준선이 있다. 다만 M1 제품 계약 전체가
  증명된 것은 아니므로 `M0-05 → M1-13`을 닫은 뒤 M2-01/02를 시작한다.
- Git built-in `G0~G3`과 SSH Remote Drive `S0~S3`은 R1 이후 제품/승인 범위가 분리된
  후속 트랙이다. 둘 다 구현하면 공용 path/config/runtime integration 순서를 적용한다.
- 실제 Git backend, 실제 SFTP transport와 Git G3 인증은 해당 후속 카드의 명시적
  비교/보안 gate에서 결정한다. Remote resume의 안전 계약은 source/partial SHA-256과
  mismatch별 오류로 확정했고, S3-04가 dependency/performance와 구현 증거를 닫는다. 어느
  항목도 현재 v1 blocker가 아니다.

## 2. v1 원문 충돌 해소

| 항목 | 원문 충돌/모호성 | 확정 결정 | 근거 문서 |
|---|---|---|---|
| F4 Edit | 기능표에는 있으나 초기 단계 목록에서 누락 | 기본 UTF-8 Editor를 M2-06/07에 포함 | [`implementation-plan/01-product-contract.md`](implementation-plan/01-product-contract.md) §9 |
| F1~F12 | 항상 표시 규칙과 F9 누락 축약 예시 | F9 `---` 포함 12개 항상 표시 | 제품 계약 §2 |
| Short View | 이름만 표시 예시와 크기/시각 참고 화면 | 컬럼 폭에 따른 3단계 정보 밀도 | 제품 계약 §5 |
| 원본/현대 키맵 | 참고 화면과 요구 키가 다름 | 현대식 기본, 원본 preset은 v1 제외 | 제품 계약 §2 |
| 상단 번호 메뉴 | 참고 화면에는 있으나 본문은 F12 | 상단 메뉴 제외, F12로 통합 | 제품 계약 §3 |
| Classic 색 | 단일 blue 예시와 화면별 검정/파랑/자홍 | 화면 역할별 theme token | 제품 계약 §11 |
| Fixed column | 좁을 때 정책 불명 | 최소 12셀까지 유지 후 컬럼 수 감소, 가로 scroll 없음 | 제품 계약 §4 |
| Auto column | 폭만으로 최대 컬럼 계산 시 적은 목록이 한쪽에 몰림 | 폭 기준 최대치 안에서 항목을 담는 최소 유효 컬럼 수 사용 | 제품 계약 §4 |
| 컬럼 경계 | 참고 화면의 cyan 경계가 계약에 없음 | 마지막을 제외한 유효 컬럼에 `│` border 1셀 | 제품 계약 §4 |
| 페이지 경계 | 방향키가 페이지 끝에서 멈출지 불명 | 마지막/첫 경계에서 Up/Down/Left/Right로 인접 페이지 연결 | 제품 계약 §6 |
| 설정 형식 | TOML/JSON 후보 | TOML 하나, scenario만 YAML | 제품 계약 §11 |
| MVP 의미 | MVP 1과 전체 release 혼동 | M0~M3 이후 R1을 v1.0으로 정의 | [`implementation-plan/README.md`](implementation-plan/README.md) |
| Rust 최소 버전 | stable toolchain과 명시 MSRV 요구가 혼재 | pre-v1은 edition 2024+rolling stable, R1에 RC compiler 기록; MSRV 약속은 별도 audit | M0-02/R1-01 |
| `..`/합계 | 합성 항목의 선택/합계 규칙 불명 | mark/합계 제외, root에는 없음 | 제품 계약 §6/7 |
| Delete | 휴지통/영구 삭제 경계 불명 | F8 휴지통, Shift+F8 별도 영구 삭제 확인 | 제품 계약 §8 |
| UI 언어 | 영어 UI 요청과 Unicode 파일명 표시가 혼동될 수 있음 | built-in copy는 영어, 사용자 파일명/경로는 원문 보존 | 제품 계약 §1 |

## 3. 현재 구현과 문서 동기화 규칙

M1 기본 탐색에서 구현되어 계약에 반영한 UX 보정:

- Auto column은 전체 entry 수와 rows를 사용해 `1..width_based_max` 유효 컬럼을 계산한다.
- 컬럼 border는 Ratatui box-drawing `│`를 사용하고 내용 폭에서 1셀을 예약한다.
- 마지막 화면 항목의 Down과 다음/이전 페이지의 Left/Right/Up이 페이지를 연결한다.
- Ctrl+A는 toggle이 아니라 현재 mark 가능한 항목을 모두 선택하는 idempotent 동작이다.
- 종료 기본 키는 Ctrl+Q이며 일반 `Q`는 계약에 포함하지 않는다.
- Enter regular file은 worker를 통해 플랫폼 기본 연결 프로그램으로 연다.

아직 M1 종료를 막는 계약 차이는 `M1-13`에서 닫는다.

- 40셀 이상 Short View의 raw modified→timestamp 시점 OS-local `MM-DD HH:mm`,
  `----- --:--` fallback과 R/H/S/A 모델
- disabled F-key의 별도 style/이유와 Registry 기반 `Ctrl+Q` 힌트
- 재사용 가능한 Unicode text helper
- FixedClock과 named effect completion/assertion을 실제로 쓰는 scenario runner
- Style-aware 80/100/120/160 snapshot과 이름 있는 release performance smoke

이 동작을 바꾸려면 구현만 수정하지 말고 제품 계약, navigation/input test와 snapshot을
동시에 갱신한다.

## 4. Git built-in 충돌 해소

| 항목 | 충돌 | 결정 |
|---|---|---|
| v1 범위 | 원문은 plugin 추가, v1은 제외 | R1 이후 G0~G3 |
| plugin 형태 | external 가능성 | 첫 버전은 정적 built-in, 공개 ABI 없음 |
| callback I/O | 동기 callback 예시 | callback은 effect만 생성, worker에서 I/O |
| F키 | Main과 Git View 기능이 다름 | 화면별 command context |
| Revert | `git revert`/local discard 혼동 | `Discard local changes`로 명명 |
| actual backend | gix/git2/CLI 후보 | G1 spike+ADR 후 하나만 선택 |
| “Remote” 명칭 | Git remote와 SSH Remote Drive 혼동 | 수용 ID는 `GITNET-*`, 제품명은 Git network operations |

상세 결정은 [`plugins/git/README.md`](plugins/git/README.md)를 따른다.

## 5. SSH Remote 요구사항 충돌 해소

| 항목 | 충돌 | 결정 |
|---|---|---|
| v1 범위 | v1은 SSH/SFTP 제외 | R1 이후 S0~S3 |
| path identity | Local `PathBuf`만으로 DEV/PROD 구분 불가 | `LocationId + LocationPath` |
| 설정 예시 | `[[remote]]`/`[[remote.locations]]` 혼재 | `[[remote.locations]]` |
| 인증 | 자체 key/password 가능성 | OpenSSH alias/config/agent/known_hosts에 전부 위임 |
| refresh | 기존 R과 원문 Ctrl+R | 같은 command action의 두 key mapping |
| 삭제 | Local trash를 SFTP에 적용 불가 | Remote permanent 경고/별도 확인 |
| read-only | UI disable만으로 우회 가능 | command/reducer/planner/backend 4계층 차단 |
| transport | native/OpenSSH 후보 | S0 spike+ADR 후 하나만 선택 |
| 서로 다른 Remote 간 Copy/Move | relay/server transfer 방식이 원문에 없음 | S0~S3의 command/effect/backend에서 제외; 별도 제품 계약과 ADR 전 구현 금지 |
| SSH terminal/Remote Git | 파일 시스템 범위와 혼재 | 별도 후속 계약 전 제외 |

상세 결정은 [`remote/README.md`](remote/README.md)를 따른다.

## 6. 의도적으로 열린 후속 gate

다음은 구현자가 임의로 선택하지 않는다.

| Gate | 카드 | 완료 조건 |
|---|---|---|
| Git 실제 backend | G1-10 | Windows/기능/license/취소 비교 ADR |
| Git network 인증/충돌 | G3-00 | 별도 보안/복구 계약 승인 |
| SFTP transport와 path/cancel 기반 | S0-00 | identity/path/cancel/배포 비교 ADR 승인 |
| Remote resume | S3-04 | capability, token/source/partial SHA-256·길이, mismatch별 실패/no-auto-restart test |
| Remote Git/SSH Terminal | S3 이후 | 별도 제품 계약과 경계 ADR |

이 gate는 해당 후속 단계만 멈춘다. v1 또는 다른 후속 트랙을 막지 않는다.
