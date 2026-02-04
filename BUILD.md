# Build & Quickstart

This repo contains the Aero compiler written in Rust (`src/compiler`).

## Prerequisites

- Rust toolchain (stable): <https://rustup.rs>
- LLVM tooling available on your PATH:
  - `clang`
  - `llc`

Install LLVM on common platforms:

- **Ubuntu/Debian:** `sudo apt-get install clang llvm`
- **macOS (Homebrew):** `brew install llvm` (then ensure Homebrew LLVM is on PATH)
- **Windows:** install LLVM from <https://llvm.org/> (or `winget install LLVM.LLVM`) and ensure `clang`/`llc` are on PATH

## Build the compiler

From the repo root:

```bash
cd src/compiler
cargo build --release
```

The compiler binary will be at:

- `src/compiler/target/release/aero` (Linux/macOS)
- `src\\compiler\\target\\release\\aero.exe` (Windows)

### Install (optional)

```bash
cargo install --path src/compiler
```

This installs `aero` into your Cargo bin directory (typically `~/.cargo/bin`).

## “Hello, world” (compile to LLVM IR)

```bash
# from repo root
./src/compiler/target/release/aero build examples/hello.aero -o hello.ll
```

If you installed via `cargo install`, you can instead run:

```bash
aero build examples/hello.aero -o hello.ll
```

## Run the included compiler smoke test

- **Linux/macOS:**
  ```bash
  chmod +x test_compiler.sh
  ./test_compiler.sh
  ```

- **Windows:**
  ```bat
  test_compiler.bat
  ```
