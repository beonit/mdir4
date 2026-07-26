# Mdir III UI Reference

Mdir III 스타일 UI를 구현할 때 참고할 원본 화면 자료다. 임시 다운로드 경로의 파일을
그대로 복사했으며, GIF는 애니메이션 형식을 유지했다.

| 파일 | 크기 | 구현 시 참고할 요소 |
|---|---:|---|
| `mdir-main-screen-animated.gif` | 648×428 | 메인 파일 목록, 고밀도 메타데이터, 색상, 상·하단 상태 영역 |
| `mdir-main-screen-dialog.jpg` | 474×313 | 메인 화면 위 모달 대화상자, 자홍색 대화상자 스타일 |
| `mdir-change-directory-tree.webp` | 640×421 | MCD 디렉터리 트리, 선택 행, 트리 연결선, 하단 키 안내 |
| `mdir-short-view-single-column.png` | 716×398 | 항목 수가 한 화면 행 이하일 때 1열로 넓게 쓰는 Short View |
| `mdir-short-view-multi-column.png` | 640×400 | 항목 수가 많을 때 같은 화면을 여러 열로 채우는 Short View |

## 해석 원칙

- 이 자료는 원본 Mdir III의 시각·상호작용 참고 자료다.
- 현대식 요구사항과 다른 부분은 자동으로 원본을 우선하지 않는다.
- 특히 Short View의 정보량, 상단 메뉴, 기능키 배치는
  `../spec-review.md`의 해소 기록과 `../implementation-plan/01-product-contract.md`를 따른다.
- 두 Short View 이미지는 적응형 열 수 결정의 시각적 근거다. 정확한 열 계산과 `│` 경계,
  페이지 탐색은 제품 계약 §4/§6과 ADR-002가 우선한다.

## SHA-256

```text
99ce3662cd21b8aeb625ad6cb07a36e775d0cc7a0497725e71b7bbd719546f0d  mdir-main-screen-animated.gif
0f2fcdd4506d3ba6cb79a2a0c7ced02bfc34a785b9b2e126e6d1abda0ebe9e2b  mdir-main-screen-dialog.jpg
bc25d032112bac207442131600ac6d9769d514795169ea960e2bdb2090759715  mdir-change-directory-tree.webp
61323cbac9e9003d2c60fd01bb184845b51cad226aebdb9ba5742999e366a768  mdir-short-view-single-column.png
01444a41845a6eaba6205181de77ff3f0f17c0fddc3453b86c701d21674f123d  mdir-short-view-multi-column.png
```
