# Project Locate와 Search Index 구현 계획

이 문서는 Mdir4의 `Ctrl+L` Project Locate를 현재 source tree에 적용하고, 이후 Symbol/ctags와
Full-text 검색으로 확장하는 실행 순서다. 현재는 File Locate의 기본 경로(재귀 인덱싱, fuzzy
검색, Enter reveal, bounded worker, 디스크 캐시)가 구현되어 있으며, 이후 카드들은 이를
확장·강화하는 roadmap이다. 기존 활성 릴리스 작업을 자동으로 선점하지 않는다.

## 1. 목표와 사용자 계약

### 1.1 1차 범위: File Locate

- Main 화면에서 `Ctrl+L`을 누르면 현재 프로젝트의 File Locate를 연다.
- 프로젝트 루트 아래의 일반 파일을 재귀적으로 찾고 경로를 fuzzy ranking한다.
- 검색 중에도 현재 파일 목록과 Preview는 UI thread를 막지 않는다.
- `Up`/`Down`은 일치 결과를 이동하고 `Backspace`는 마지막 grapheme을 삭제한다.
- `Enter`는 결과 파일의 부모 디렉터리로 이동한 뒤 그 파일을 정확히 선택한다.
- File Locate의 `Enter`는 파일을 실행하거나 편집하지 않는다. 사용자는 이동 후 기존 `Enter`,
  `F3`, `F4`를 사용할 수 있다.
- `Esc`는 Locate 진입 전 경로와 선택을 보존한 채 모드를 닫는다.
- 결과가 인덱싱 뒤 삭제되었으면 이동하지 않고 stale 결과를 제거해 이유를 표시한다.

### 1.2 프로젝트 루트

프로젝트 루트는 다음 순서로 결정한다.

1. 현재 Local path 자신 또는 가장 가까운 조상에 `.git` directory/file이 있으면 그 조상
2. 향후 명시적인 `.mdir4-root` marker 또는 설정 override가 승인되면 해당 경로
3. 위 조건이 없으면 Locate를 연 시점의 현재 디렉터리

root identity와 cache key에는 canonical path를 사용한다. 화면과 navigation에는 원래
`PathBuf`/`OsString`을 보존하며 display string으로 path를 재구성하지 않는다. Git built-in
plugin 타입이나 Git subprocess에 Core Locate의 root 탐지를 의존시키지 않는다.

### 1.3 기본 검색/제외 정책

확장자 allowlist를 기본 정책으로 사용하지 않는다. `Makefile`, `Dockerfile`, `README`,
`.editorconfig`, `.env.example`처럼 확장자가 없거나 leading-dot인 유용한 파일을 놓치기
때문이다. 기본 후보는 ignore 정책을 통과한 모든 일반 파일이며 source/config/document
성격의 이름과 확장자는 ranking bonus로 다룬다.

기본 규칙:

- `.gitignore`, `.git/info/exclude`, global Git ignore와 `.ignore`를 적용한다.
- `.git/**`, `.hg/**`, `.svn/**`는 항상 제외한다.
- `node_modules/**`, `target/**`, `build/**`, `dist/**`, `__pycache__/**`,
  `.pytest_cache/**`는 기본 prune 대상이다.
- `.DS_Store`, `Thumbs.db`, `desktop.ini`와 Mdir4 temporary/backup 파일은 제외한다.
- `.github/**`, `.vscode/**`, `.gitignore`, `.editorconfig`, `.env.example` 같은 프로젝트
  설정은 일반 hidden 정책과 별도로 포함한다.
- symlink directory는 따라가지 않고 symlink file은 기본 결과에서 제외한다.
- directory 자체는 1차 File Locate 결과에 넣지 않는다.
- 사용자 추가 규칙은 project `.ignore` 또는 `[locate].exclude`로 제공한다.
- `show_hidden=false`에서 hidden 설정 파일을 선택한 경우 해당 target 하나만 transient reveal하고
  설정값을 변경하거나 저장하지 않는다. cursor/path가 바뀌면 다시 숨긴다.

`target`, `dist` 같은 이름은 `.gitignore`가 없는 프로젝트를 위한 안전 기본값이다. 사용자가
명시적으로 포함해야 하는 repository에는 `[locate].include` override를 제공한다. include와
exclude의 우선순위는 `LOC-00` 계약에 표와 예제로 고정한다.

### 1.4 후속 검색 문법

1차 File Locate는 prefix 없는 query만 구현한다. 공통 query/parser 타입은 다음 확장을
수용하되 `SYM-*` 또는 `TXT-*` 카드 전에 활성화하지 않는다.

```text
foo             file path fuzzy search
@foo            all symbol definitions
@class Foo      class definitions
@method render  method definitions
#preview        source text search
```

후속 결과 동작:

- File result `Enter`: 부모 디렉터리로 이동하고 파일 선택
- Symbol/Text result `Enter`: Viewer 또는 Editor를 열고 line/column으로 이동
- 모든 result의 `Alt+Enter`: 파일을 열지 않고 부모 디렉터리에서 선택

내부 Viewer/Editor line navigation과 외부 `$EDITOR`별 line argument는 별도 카드다. 위치 정보가
없는 provider 결과는 File result처럼 처리한다.

## 2. 비범위

### 2.1 File Locate 1차 구현 비범위

- 파일 내용 검색, reference/call hierarchy, rename symbol
- SSH Remote tree와 network filesystem SLA
- filesystem watcher와 OS journal 기반 실시간 증분 인덱싱
- `locate`, `plocate`, `rg`, `fd`, `fzf` executable을 필수 runtime dependency로 지정
- project root에 `tags`나 Mdir4 cache file을 사용자 승인 없이 생성
- binary content 판독 또는 MIME 분석

### 2.2 후속 단계에서도 별도 승인이 필요한 항목

- Language Server lifecycle, workspace trust와 compiler/build 실행
- Tree-sitter grammar bundle과 지원 language 정책
- reference graph, call hierarchy, inheritance graph
- Remote ctags 실행 또는 remote source를 local cache에 저장
- cache encryption과 multi-user shared cache

## 3. 선행 관계

```text
LOC-00 계약/ADR/기준선
  -> LOC-01 공통 search model/provider
      -> LOC-02 root/filter/path scan
          -> LOC-03 Locate worker와 fuzzy query
              -> LOC-04 persistent cache
                  -> LOC-05 Ctrl+L UI/input
                      -> LOC-06 Enter reveal/stale 처리
                          -> LOC-07 통합/SLA/문서 종료
                              -> SYM-01 symbol schema/capability
                                  -> SYM-02 Universal Ctags index
                                      -> SYM-03 symbol UI/source jump
                                          -> SYM-04 Vi tags export
                              -> TXT-01 on-demand text provider
                                  -> TXT-02 persistent full-text index
```

`LOC-04`와 `LOC-05`는 `LOC-03`의 stable completion contract 뒤에 병렬로 진행할 수 있다.
`SYM-01`과 `TXT-01`은 `LOC-07` 뒤에 서로 독립적으로 진행할 수 있다. Persistent text index는
on-demand text search의 corpus/latency 측정 없이 시작하지 않는다.

## 4. 목표 아키텍처

### 4.1 의존 방향

```text
Keyboard
  -> Input Mapper
      -> Search Action
          -> App reducer/state
              -> Locate worker request
                  -> ProjectRootResolver port
                  -> FileIndexProvider
                  -> SymbolIndexProvider (future)
                  -> TextSearchProvider (future)
              <- generation-bound completion
          -> UI Search overlay / Main reveal
```

- reducer, mapper와 renderer는 filesystem traversal, cache I/O 또는 subprocess를 호출하지 않는다.
- Core Local worker는 mutation/preview/directory load를 계속 소유한다.
- recursive scan과 fuzzy ranking은 별도 bounded Locate Read lane이 소유한다.
- provider resource/thread/process는 `AppState`가 아니라 runtime adapter가 소유한다.
- 모든 request/completion은 project identity, query generation과 index generation을 가진다.
- stale root/query/index completion은 state에 적용하지 않는다.

### 4.2 공통 타입

다음 형태를 기준으로 하되 정확한 이름은 `LOC-01`에서 코드와 함께 고정한다.

```rust
pub struct ProjectId(/* canonical local root identity */);

pub enum SearchScope {
    Files,
    Symbols(SymbolFilter),
    Text,
}

pub struct SearchQuery {
    pub scope: SearchScope,
    pub text: String,
    pub generation: u64,
}

pub struct SourceLocation {
    pub path: PathBuf,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub end_line: Option<u32>,
    pub pattern: Option<String>,
}

pub enum SearchHit {
    File { location: SourceLocation },
    Symbol {
        name: String,
        kind: SymbolKind,
        scope: Option<String>,
        language: Option<String>,
        signature: Option<String>,
        location: SourceLocation,
    },
    Text {
        excerpt: String,
        location: SourceLocation,
    },
}
```

`SearchHit`은 stable result identity, display label, match indices와 score를 직접 소유하지 않는다.
provider result envelope가 이 값을 함께 운반한다. original path bytes와 lossy display/match text를
구분하고 non-UTF-8 path도 File navigation 대상으로 유지한다.

### 4.3 1차 외부 crate

- `ignore`: recursive walk와 Git/project ignore semantics
- `nucleo`: interactive fuzzy ranking과 match indices
- 안정적인 project cache key를 위한 작은 hash crate 또는 동등한 내부 구현

추가 crate/version은 구현 시작 시 `Cargo.lock`, MSRV, license와 release binary size를 확인해
`LOC-00`에 기록한다. 외부 executable은 1차 필수 dependency가 아니다.

### 4.4 plocate provider 경계

`plocate`는 1차 backend가 아니다. 이후 필요하면 `FileIndexProvider`의 optional adapter로
추가한다. project 전용 DB를 사용하더라도 Mdir4 filter contract와 fuzzy ranking은 동일해야
한다. system global DB 결과를 그대로 노출하거나 project root 밖 결과를 state에 넣지 않는다.

## 5. 인덱스와 캐시 계약

### 5.1 메모리 캐시

- Locate runtime이 project별 immutable index snapshot을 보유한다.
- 최대 8 projects 또는 합계 256 MiB 중 먼저 도달하는 상한을 사용한다.
- 30분 idle entry를 LRU 순서로 제거하되 active project는 제거하지 않는다.
- 새 snapshot은 완성 전 현재 snapshot을 덮지 않는다.
- path index 기본 상한은 250,000 regular files다.
- 상한을 넘으면 partial result를 유지하고 명확한 truncation status를 표시한다.

### 5.2 디스크 캐시 위치

플랫폼별 user cache directory 아래 `mdir4/locate`를 사용한다.

```text
macOS:  ~/Library/Caches/mdir4/locate/
Linux:  ${XDG_CACHE_HOME:-~/.cache}/mdir4/locate/
```

Windows는 현재 release blocker가 아니지만 adapter 계약은
`%LOCALAPPDATA%\mdir4\cache\locate\`를 표현할 수 있어야 한다. `MDIR4_CONFIG`는 cache
directory를 암시하지 않는다. 테스트는 주입된 temporary cache root를 사용한다.

```text
locate/
├── catalog-v1
├── <project-hash>/
│   ├── manifest-v1
│   ├── paths-v1.idx
│   ├── symbols-v1.idx        # SYM 단계
│   ├── fulltext/             # TXT 단계
│   └── tags                  # 명시적 Vi export
└── cleanup.lock              # 필요성이 측정된 뒤에만 추가
```

cache record에는 file content를 저장하지 않는다. Path index는 canonical root, root-relative
native path representation, file kind, format/policy version과 build metadata만 저장한다.
non-UTF-8 path와 newline을 허용하도록 line-delimited plain text가 아닌 length-prefixed format을
사용한다.

### 5.3 디스크 보관과 권한

- idle 30일 후 제거
- 전체 최대 512 MiB, 최대 32 projects
- 두 상한 중 하나를 넘으면 last-used 순서로 제거
- directory/file은 가능한 플랫폼에서 user-only permission으로 생성
- temporary sibling에 write/flush/sync한 뒤 rename해 publish
- parse/version/checksum failure는 cache miss로 처리하고 원본 project를 변경하지 않음
- Settings 또는 명시적 command로 `Clear Locate Cache`를 제공하되 현재 카드와 별도로 승인

Symbol만 활성화하면 같은 512 MiB budget 안에서 shard별 LRU를 사용한다. Persistent Full-text를
활성화할 때는 별도 opt-in과 최대 2 GiB budget을 승인해야 한다.

### 5.4 보관과 최신성 분리

30일 보관은 30일 동안 fresh하다는 뜻이 아니다.

- 마지막 full validation이 60초 이내면 memory/disk snapshot을 즉시 사용한다.
- 60초를 넘으면 cached result를 먼저 표시하고 background rebuild를 시작한다.
- Mdir4 내부 create/rename/move/delete success는 해당 project index를 dirty 처리한다.
- `.gitignore`, `.ignore`, filter config 또는 format/provider version 변경은 full rebuild다.
- filesystem watcher는 1차에 넣지 않는다. 외부 변경은 background rebuild로 수렴한다.
- result 확정 시 target metadata를 재확인해 stale path navigation을 방지한다.

## 6. Fuzzy ranking과 결과 표시

- path 전체를 fuzzy 대상에 넣되 basename match에 가장 큰 bonus를 준다.
- exact basename > basename prefix > path-segment prefix > contiguous substring > fuzzy gaps
  순으로 기대 결과가 올라오게 한다.
- source/config/document 확장자와 conventional filename에는 작은 bonus만 주고 다른 일반 파일을
  완전히 제외하지 않는다.
- case matching은 smart/Unicode normalization을 사용한다.
- 동점은 root-relative path byte order 또는 scan-independent stable ID로 고정한다.
- query별 Top N만 UI에 복사하며 기본 100개, viewport에는 보이는 행만 render한다.
- match indices는 표시할 Top N에만 계산해 highlight 비용을 제한한다.
- empty query는 recent result가 아니라 deterministic path ranking을 표시한다.

Locate overlay 최소 정보:

```text
Locate  src/app                         12 / 8,421
  src/app.rs
  src/app/command_registry.rs
  tests/scenarios.rs
Index: ready · root /work/mdir4 · Ctrl+R Reindex
```

indexing, ready, refreshing, truncated, unavailable와 error 상태를 구분한다. cached 결과가 있으면
refreshing 중에도 입력과 선택을 허용한다.

## 7. Enter reveal과 source location

File result 확정 시 reducer는 `pending_reveal`에 target identity를 저장하고 target parent의
`LoadDirectory` Effect를 만든다. 해당 directory completion에서만 target path를 찾아 선택한다.

- target parent가 현재 directory여도 exact identity로 다시 선택한다.
- directory sort와 hidden filter 적용 뒤 선택한다.
- hidden target 하나는 transient reveal하되 `show_hidden` 설정은 변경하지 않는다.
- stale/missing/permission error는 이전 directory와 selection을 유지한다.
- 새 Locate/reveal generation이 시작되면 이전 completion은 무시한다.
- reveal completion 뒤 Preview는 정확히 한 번 target에 맞게 갱신한다.
- search typing 중에는 Core Preview read를 매 key마다 제출하지 않는다.

Symbol/Text 단계에서는 같은 `SourceLocation`을 사용한다. Viewer에 `jump_to_line`, Editor에
`set_cursor_line_column`을 추가하고 UTF-8 grapheme/line boundary를 검증한다. external editor는
editor별 adapter가 없는 한 path만 넘기며 임의의 `+line` syntax를 추측하지 않는다.

## 8. 작업 카드

### LOC-00 계약, ADR과 기준선 고정

- 목표: key, root, filter, worker, cache, SLA와 future provider 경계를 승인한다.
- 주 파일:
  - `docs/locate-search-implementation-plan.md`
  - `docs/architecture/adr-008-project-search-index.md` 신규
  - `docs/README.md`
- 작업:
  1. `Ctrl+L`, Enter/Esc/Alt+Enter 의미와 Main-only 1차 범위를 승인한다.
  2. root resolution과 hidden config transient reveal을 승인한다.
  3. Locate Read lane과 cache directory/permission/retention을 ADR로 기록한다.
  4. dependency license/MSRV/binary size baseline을 기록한다.
  5. 아래 SLA를 release benchmark target으로 승인한다.
  6. 현재 repository와 synthetic 10k/100k/250k tree 기준선을 기록한다.
- 완료: 미결정 동작 없이 failing contract test를 작성할 수 있음.

### LOC-01 공통 search model과 provider 경계

- 선행: LOC-00
- 목표: File 구현에 Symbol/Text 의미를 섞지 않고 확장 가능한 typed model을 만든다.
- 주 파일:
  - `src/model/search.rs` 신규
  - `src/model/mod.rs`
  - `src/ports/search.rs` 신규
  - `src/ports/mod.rs`
  - provider contract test 신규
- 작업:
  1. `ProjectId`, `SearchScope`, `SearchQuery`, `SourceLocation`, `SearchHit`을 추가한다.
  2. provider request/completion identity와 generation을 정의한다.
  3. native path와 display/match text를 분리한다.
  4. cancellation, truncation, unavailable와 partial completion을 typed result로 만든다.
  5. fake provider로 ordering/stale/error contract를 검증한다.
- 완료: OS I/O 없이 reducer/provider contract test가 통과함.

### LOC-02 root resolver, filter와 path scan

- 선행: LOC-01
- 목표: project root 아래의 원하는 일반 파일만 결정적으로 열거한다.
- 주 파일:
  - `src/ports/search.rs`
  - `src/adapters/project_index.rs` 신규
  - `src/adapters/mod.rs`
  - `Cargo.toml`, `Cargo.lock`
  - path scan integration test 신규
- 작업:
  1. ancestor `.git` directory/file 탐지와 fallback root를 구현한다.
  2. `ignore` walker와 built-in prune/override 규칙을 조합한다.
  3. symlink, permission denial, disappearing file과 non-UTF-8 path를 처리한다.
  4. 250k 상한과 progress/partial result를 구현한다.
  5. Memory/Fake fixture와 실제 TempDir에서 동일 filter matrix를 검증한다.
- 테스트:
  - nested `.gitignore`/`.ignore`, negation과 global ignore fixture
  - `.git` 제외와 `.github`/`.gitignore` 포함
  - target/node_modules 기본 prune와 explicit include
  - symlink cycle 0회, permission error partial success
  - non-UTF-8/newline filename round-trip
- 완료: UI thread 호출 없이 root-relative `IndexedPath` snapshot을 생성함.

### LOC-03 Locate worker와 fuzzy query

- 선행: LOC-02
- 목표: recursive scan과 query ranking을 bounded, non-blocking lane에서 실행한다.
- 주 파일:
  - `src/runtime.rs`
  - `src/runtime/locate.rs` 신규
  - `src/model/search.rs`
  - `Cargo.toml`, `Cargo.lock`
  - worker/fuzzy test 신규
- 작업:
  1. capacity가 주입 가능한 Locate worker와 non-blocking submit을 추가한다.
  2. 같은 project rebuild/query는 최신 generation으로 coalesce한다.
  3. root 변경, Esc와 shutdown cancellation을 구현한다.
  4. `nucleo` path matcher와 basename/path bonus를 구성한다.
  5. Top 100과 match indices만 state completion으로 보낸다.
  6. panic/closed/full queue를 typed unavailable/busy 결과로 바꾼다.
- 테스트:
  - 빠른 query generation 중 마지막 결과만 적용
  - scan 중 Esc/shutdown cleanup
  - exact/prefix/segment/fuzzy ordering과 stable ties
  - Unicode normalization/case, long path, empty query
  - queue full/coalescing을 sleep 없는 injected-capacity test로 검증
- 완료: 100k candidate query 중 UI thread에서 scan/ranking/I/O 0회.

### LOC-04 persistent cache와 lifecycle

- 선행: LOC-03
- 목표: 재실행 후 cached result를 먼저 표시하고 background refresh한다.
- 주 파일:
  - `src/ports/cache.rs` 신규 또는 search port 하위
  - `src/adapters/project_cache.rs` 신규
  - `src/runtime/locate.rs`
  - `src/config/schema.rs`
  - cache roundtrip/fault test 신규
- 작업:
  1. platform cache root resolver와 test override를 추가한다.
  2. versioned manifest와 length-prefixed path format을 구현한다.
  3. atomic publish, user-only permission과 corrupt cache fallback을 구현한다.
  4. memory 8/256 MiB/30분 LRU와 disk 32/512 MiB/30일 LRU를 구현한다.
  5. 60초 freshness, filter/provider version과 internal mutation invalidation을 연결한다.
  6. cleanup은 background에서 bounded하게 실행한다.
- 테스트:
  - cache hit/miss/version/policy/root mismatch
  - truncated/corrupt/partial write와 atomic recovery
  - non-UTF-8 path roundtrip
  - TTL/LRU/byte/project limit with injected Clock
  - mode/permission failure 시 project 원본 write 0회
- 완료: warm cache가 scan 없이 즉시 query 가능하고 refresh가 결과를 원자적으로 교체함.

### LOC-05 Ctrl+L input, reducer와 UI

- 선행: LOC-03; warm-cache 표시는 LOC-04와 통합
- 목표: Main에서 keyboard-only Locate workflow를 완성한다.
- 주 파일:
  - `src/app.rs`
  - `src/app/command_registry.rs`
  - `src/input/mapper.rs`
  - `src/ui.rs`
  - `tests/input_mapping.rs`
  - `tests/scenarios/locate.yml` 신규
- 작업:
  1. `Locate` CommandId 기본 `Ctrl+L`과 config name을 등록한다.
  2. Locate screen/state와 character/backspace/up/down/page/confirm/cancel/reindex Action을 추가한다.
  3. 기존 Main 900ms prefix type-ahead는 변경하지 않는다.
  4. Main을 배경으로 query/results/status overlay를 렌더한다.
  5. indexing/refreshing/ready/truncated/error와 match count를 표시한다.
  6. Unicode cursor와 narrow/too-small layout을 처리한다.
  7. 기존 `refresh = "Ctrl+L"` custom keymap collision에 명확한 diagnostic을 제공한다.
- 테스트:
  - Main `Ctrl+L`; 다른 modal/full-screen context에서는 미실행
  - character/grapheme backspace/navigation/Enter/Esc/Ctrl+R
  - no match/partial/refreshing/truncated/error snapshot
  - 60x15, 80x25, 120x40, Unicode/long path
  - custom keymap remap과 collision
- 완료: filesystem/cache/subprocess call이 없는 reducer/render snapshot으로 전체 모드가 재현됨.

### LOC-06 Enter reveal, hidden target와 stale path

- 선행: LOC-05
- 목표: result를 실제 Main directory selection으로 안전하게 연결한다.
- 주 파일:
  - `src/app.rs`
  - `src/model/directory.rs`
  - `src/runtime.rs`
  - `tests/directory_loading.rs`
  - `tests/scenarios/locate.yml`
- 작업:
  1. generation-bound `pending_reveal`을 추가한다.
  2. target metadata 재검증 뒤 parent directory load를 시작한다.
  3. completion에서 exact native path를 선택하고 Preview를 한 번 갱신한다.
  4. hidden target transient reveal과 해제 조건을 구현한다.
  5. stale/not-found/permission/parent-changed failure에서 기존 path/selection을 보존한다.
- 테스트:
  - same/nested parent, sort order와 selected identity
  - hidden target reveal 후 설정/재시작 값 불변
  - deleted/renamed target와 stale completion
  - rapid Enter/Esc/new Locate generation
  - Preview effect 정확히 1회
- 완료: Enter가 파일을 실행하지 않고 정확한 directory+selection으로 끝남.

### LOC-07 통합 수용, SLA와 문서 종료

- 선행: LOC-01~06
- 목표: 전체 contract, 성능, cache lifecycle과 실제 terminal 동작을 검증한다.
- 주 파일:
  - `tests/scenarios/locate.yml`
  - `tests/snapshots/*`
  - `src/snapshots/*`
  - `docs/README.md`
  - `README.md`
  - `docs/locate-search-implementation-plan.md`
  - `docs/implementation-plan/progress.md`
- 작업:
  1. 10k/100k/250k synthetic release benchmark와 fixture 생성 조건을 고정한다.
  2. cold index, warm disk cache, memory cache와 query latency를 각각 측정한다.
  3. rapid input 중 key-to-frame와 worker queue 상태를 측정한다.
  4. Linux/macOS에서 cache path/permission/atomic replacement와 `Ctrl+L` 전달을 확인한다.
  5. 실제 Git/non-Git project, ignore/hidden config와 stale file을 수동 확인한다.
  6. dependency license와 release package 크기를 기록한다.
  7. card 상태, test 이름, 날짜와 측정 결과를 progress ledger에 기록한다.
- 완료: 아래 `LOC-*` 수용 기준이 모두 통과하고 `.snap.new`가 0개임.

## 9. 후속 Symbol/ctags 카드

### SYM-01 symbol schema와 provider capability

- 선행: LOC-07
- 목표: ctags 설치 유무와 무관하게 typed symbol result와 unavailable 상태를 정의한다.
- 작업:
  1. class/method/function/struct/interface/module/variable 등 normalized `SymbolKind`를 정의한다.
  2. provider별 raw kind를 normalized kind로 매핑하고 unknown을 보존한다.
  3. name, scope, language, signature, definition/reference role과 `SourceLocation`을 정의한다.
  4. `ctags --version`, output format, JSON feature, language/field capability probe를 추가한다.
  5. subprocess unavailable/unsupported version은 File Locate를 막지 않는다.
- 완료: Fake Symbol provider로 query/filter/order/source location contract 통과.

### SYM-02 Universal Ctags index와 증분 shard

- 선행: SYM-01
- 목표: File Index와 같은 corpus에서 definition symbol을 background 생성한다.
- 작업:
  1. shell을 거치지 않는 structured process arguments를 사용한다.
  2. JSON Lines `_type`, output version과 `name/path/line/pattern/language/kind/scope`를 검증한다.
  3. stdout/stderr/entry/line length, deadline와 cancel 상한을 둔다.
  4. file fingerprint별 symbol shard를 저장하고 changed/deleted file record만 교체한다.
  5. ctags version/language/options 변경 시 symbols shard만 rebuild한다.
  6. malformed line은 bounded diagnostic으로 건너뛰되 silent total success로 보고하지 않는다.
- 완료: mixed-language fixture와 real throwaway repository에서 class/method/function 위치가 정확함.

### SYM-03 symbol query UI와 source jump

- 선행: SYM-02
- 목표: `@`, `@class`, `@method` query와 line 이동을 완성한다.
- 작업:
  1. query parser와 kind filter를 추가한다.
  2. exact symbol name, scope prefix, fuzzy name, path 순으로 ranking한다.
  3. result에 kind/scope/path:line을 표시한다.
  4. Viewer `jump_to_line`과 Editor `set_cursor_line_column`을 추가한다.
  5. stale line은 pattern fallback으로 재확인하고 실패하면 file reveal로 degrade한다.
- 완료: method/class query Enter가 정확한 source line을 열고 Alt+Enter는 file reveal만 수행함.

### SYM-04 Vi compatible tags export

- 선행: SYM-02
- 목표: Mdir 내부 index와 동일 corpus로 Universal Ctags 표준 tags 파일을 명시적으로 생성한다.
- 작업:
  1. 기본 destination은 project cache의 `<project-hash>/tags`다.
  2. project root `./tags` write는 사용자 명령/확인 없이 수행하지 않는다.
  3. 직접 tags 문법을 합성하지 않고 Universal Ctags output을 atomic publish한다.
  4. destination collision, symlink, permission과 dirty worktree 안내를 처리한다.
  5. generated file의 provider/options/corpus revision을 manifest에 기록한다.
- 완료: Vi/Neovim에서 class/method/function jump를 수동 확인하고 project write 기본값 0회.

## 10. 후속 Full-text 카드

### TXT-01 on-demand text provider

- 선행: LOC-07
- 목표: persistent content DB 없이 `#query`의 실제 사용량과 latency를 측정한다.
- 작업:
  1. File Index corpus만 대상으로 bounded background text search를 실행한다.
  2. binary/oversized file, result count, excerpt bytes, deadline와 cancellation 상한을 둔다.
  3. provider executable 또는 embedded library 선택은 ADR로 기록한다.
  4. query/result/stale source line UI를 Symbol과 공유한다.
- 완료: 실제 repository 측정으로 TXT-02 필요성을 판단할 수 있음.

### TXT-02 persistent full-text index

- 선행: TXT-01 측정과 별도 승인
- 목표: 대형 corpus의 반복 keyword query를 inverted index로 가속한다.
- 작업:
  1. embedded Rust full-text engine과 tokenization/language/Unicode 정책을 결정한다.
  2. content는 저장하지 않고 필요한 stored path/line/excerpt 정책만 최소화한다.
  3. per-file delete/add, commit generation과 crash recovery를 구현한다.
  4. opt-in, 최대 2 GiB, 30일 LRU와 Clear Cache UI를 제공한다.
  5. source size 대비 index ratio와 100 MiB/1 GiB corpus SLA를 측정한다.
- 완료: on-demand 대비 반복 query 이득이 측정되고 privacy/storage contract가 승인됨.

## 11. SLA와 측정 계약

release build, local SSD, application-cold/OS-warm 조건의 p95 목표다. CI debug test의 wall-clock을
완료 증거로 사용하지 않는다. network volume과 SSH Remote는 이 표의 대상이 아니다.

| ID | 규모/작업 | p95 목표 |
|---|---|---:|
| LOC-PERF-01 | 10,000 files cold full index | 250 ms 이하 |
| LOC-PERF-02 | 100,000 files cold full index | 1.5 s 이하 |
| LOC-PERF-03 | 250,000 files cold full index | 4 s 이하 |
| LOC-PERF-04 | cold indexing first partial result | 150 ms 이하 |
| LOC-PERF-05 | 100,000 files warm disk cache load | 100 ms 이하 |
| LOC-PERF-06 | 250,000 files warm disk cache load | 250 ms 이하 |
| LOC-PERF-07 | 100,000 candidates query-to-results | 50 ms 이하 |
| LOC-PERF-08 | 250,000 candidates query-to-results | 100 ms 이하 |
| LOC-PERF-09 | indexing/query 중 key-to-frame | 50 ms 이하 |
| SYM-PERF-01 | 1M LOC initial ctags index | 5 s 이하 목표 |
| SYM-PERF-02 | changed file symbol refresh | 300 ms 이하 목표 |
| SYM-PERF-03 | cached symbol query | 50 ms 이하 목표 |
| TXT-PERF-01 | 100 MiB initial persistent content index | 10 s 이하 목표 |
| TXT-PERF-02 | cached text query | 100 ms 이하 목표 |

2026-07-29 개발 환경의 방향성 기준선:

- 현재 repository의 ignore 적용 172 paths 열거: 약 0.01 s
- 6.5 GiB `target`을 포함한 230,508 paths 열거: 약 0.24 s

이 값은 `rg --files` 관찰치이며 Locate 구현의 수용 증거가 아니다. `LOC-07`에서 실제 provider,
serialization과 matcher를 포함해 같은 조건을 반복 측정하고 median/p95, hardware, filesystem,
build profile을 기록한다. 느린 환경에서는 UI non-blocking과 progress/cancel은 필수지만 cold
index wall-clock SLA failure는 환경과 corpus를 함께 보고한다.

## 12. 수용 기준

| ID | 기준 | 카드 | 자동 검증 | 수동 검증 |
|---|---|---|---|---|
| LOC-01 | Main `Ctrl+L`이 File Locate를 열고 custom keymap 가능 | LOC-05 | input/registry | terminal key 전달 |
| LOC-02 | nearest Git root, non-Git current-dir fallback | LOC-02 | root fixture | worktree |
| LOC-03 | Git/project ignore와 built-in prune 적용 | LOC-02 | ignore matrix | 실제 repository |
| LOC-04 | `.git` 제외, useful hidden config 포함 | LOC-02 | path fixture | `.github`/dotfile |
| LOC-05 | source/config bonus가 다른 일반 파일을 완전히 숨기지 않음 | LOC-03 | ranking table | 검색 감각 |
| LOC-06 | scan/query/cache I/O가 UI thread에서 0회 | LOC-03~05 | recording ports | rapid typing |
| LOC-07 | stale generation completion 무시 | LOC-03~06 | generation matrix | 없음 |
| LOC-08 | Enter가 parent directory로 이동하고 exact file 선택 | LOC-06 | reducer/scenario | nested path |
| LOC-09 | Enter가 파일 실행/편집을 호출하지 않음 | LOC-06 | recording launcher | 없음 |
| LOC-10 | hidden target transient reveal이 설정을 바꾸지 않음 | LOC-06 | config/reducer | 재시작 |
| LOC-11 | stale/missing target이 기존 위치를 보존 | LOC-06 | fault scenario | 외부 삭제 |
| LOC-12 | non-UTF-8/newline path cache와 navigation round-trip | LOC-02,04,06 | Unix test | 없음 |
| LOC-13 | memory cache 8/256 MiB/30분 LRU | LOC-04 | injected Clock/capacity | 없음 |
| LOC-14 | disk cache 32/512 MiB/30일 LRU와 atomic recovery | LOC-04 | lifecycle/fault | permission |
| LOC-15 | 60초 뒤 cached-first background refresh | LOC-04 | injected Clock | 외부 변경 |
| LOC-16 | 250k 상한에서 partial result와 truncation 표시 | LOC-02,05 | synthetic fixture | 없음 |
| LOC-17 | Unicode query/input/highlight가 panic 없이 일치 | LOC-03,05 | unit/snapshot | IME 입력 |
| LOC-18 | LOC-PERF-01~09 목표 측정 | LOC-07 | ignored release bench | Linux/macOS |
| LOC-19 | 기존 type-ahead/navigation/Preview/file operation 회귀 없음 | LOC-05~07 | full suite | walkthrough |
| SYM-01 | ctags unavailable이 File Locate를 막지 않음 | SYM-01 | fake capability | PATH 없음 |
| SYM-02 | class/method/function name/kind/scope/location 보존 | SYM-02 | JSON fixture | mixed repo |
| SYM-03 | `@` query Enter가 정확한 source line으로 이동 | SYM-03 | scenario | Viewer/Editor |
| SYM-04 | Vi tags export가 명시적이고 호환됨 | SYM-04 | output/atomic test | Vi/Neovim |
| TXT-01 | `#` 검색이 bounded/cancellable하고 source line 이동 | TXT-01 | provider/fault | large repo |

## 13. 공통 품질 게이트

각 카드의 기본 품질 게이트:

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
git diff --check
```

UI 카드는 관련 snapshot diff를 사람이 검토하고 `.snap.new`를 남기지 않는다. Performance
카드는 debug unit test에 불안정한 짧은 wall-clock assertion을 넣지 않고 이름 있는 ignored
release benchmark와 기록된 실행 조건을 사용한다. External ctags integration test는 isolated
fixture와 capability detection을 가지며, ctags 부재를 전체 기본 suite 실패로 만들지 않는다.

## 14. 구현 중 중단하고 결정해야 하는 조건

다음 상황은 구현자가 임의로 범위를 넓히지 않고 ADR과 이 계획을 먼저 갱신한다.

- `Ctrl+L`이 path/location bar와 충돌해 Project Locate 의미를 바꿔야 함
- project root가 nearest `.git`보다 workspace/monorepo manifest를 우선해야 함
- 기본 prune이 tracked source/config를 반복적으로 숨김
- 250k/256 MiB/512 MiB/30일/60초 상수를 바꿔야 함
- recursive scan을 Core Local mutation lane에서 실행해야만 하는 구조적 이유가 생김
- filesystem watcher, daemon 또는 OS journal이 1차 필수로 필요해짐
- `plocate`나 다른 executable을 필수 runtime dependency로 만들려 함
- cache에 source content, credentials 또는 repository 외부 path를 저장해야 함
- Universal Ctags JSON 미지원 환경을 필수 지원하기 위해 별도 parser가 필요함
- Symbol Enter의 기본 동작이 internal open이 아니라 external editor 실행이어야 함
- Full-text index가 2 GiB 또는 source size의 합의된 비율을 초과함

## 15. 구현 참고 자료

- [`ignore`와 ripgrep의 ignore/filter 동작](https://github.com/BurntSushi/ripgrep/blob/master/GUIDE.md)
- [`nucleo` high-level background matcher](https://docs.rs/nucleo/latest/nucleo/)
- [Universal Ctags JSON Lines 형식](https://docs.ctags.io/en/stable/man/ctags-json-output.5.html)
- [Universal Ctags output format과 Vi-compatible tags](https://docs.ctags.io/en/stable/output-format.html)
- [Tree-sitter code navigation tag vocabulary](https://tree-sitter.github.io/tree-sitter/4-code-navigation.html)
- [`plocate` project DB 검토용 `updatedb`](https://plocate.sesse.net/updatedb.8.html)
- [`plocate` query/database option](https://plocate.sesse.net/plocate.1.html)
