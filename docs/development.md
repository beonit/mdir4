# 개발 환경

## 필수 Rust 도구

프로젝트 루트의 `rust-toolchain.toml`이 stable Rust와 다음 구성요소를 지정한다.

- `cargo`
- `rustc`
- `rustfmt`
- `clippy`

프로젝트 디렉터리에서 Cargo 명령을 실행하면 rustup이 지정된 toolchain을 선택한다.

## macOS 설치

[공식 rustup 설치 안내](https://rust-lang.github.io/rustup/installation/)를 사용한다.

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
  | sh -s -- -y --profile minimal --default-toolchain stable

. "$HOME/.cargo/env"
rustup component add rustfmt clippy
```

새 터미널을 열거나 Cargo 환경 파일을 source한 뒤 검증한다.

## Windows PowerShell 설치

[Rust 공식 설치 페이지](https://www.rust-lang.org/tools/install)에서 `rustup-init.exe`를
받아 stable toolchain을 설치한다. MSVC 빌드 도구를 요청하면 안내에 따라
Visual Studio Build Tools의 C++ 도구와 Windows SDK를 설치한다.

새 PowerShell을 열고 다음을 실행한다.

```powershell
rustup default stable
rustup component add rustfmt clippy
```

## 설치 검증

```text
rustup show active-toolchain
rustc --version
cargo --version
rustfmt --version
cargo clippy --version
```

## 프로젝트 품질 명령

Cargo 패키지가 초기화된 뒤 모든 작업 완료 전에 실행한다.

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
```

## 릴리스 후보 패키지

Windows CI는 품질 게이트가 끝난 뒤 `cargo build --release --locked`를 실행하고 다음 명령으로
단일 실행 파일, README, 개발/설치 문서, 잠긴 의존성 라이선스 목록을 ZIP으로 묶는다.

```text
python scripts/package_release.py --binary target/release/mdir4.exe --target windows-x86_64
```

산출물은 `dist/mdir4-v0.1.0-windows-x86_64.zip`과 `dist/SHA256SUMS`다. R1-02 수동 시험에는
CI가 업로드한 ZIP을 내려받아 `SHA256SUMS`와 일치하는지 확인한 뒤 사용한다. 개발 머신에서
다시 빌드한 실행 파일로 대체하면 동일 RC로 취급하지 않는다.
