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

## CLI command summary (v1.0.0)

- `aero build <input.aero> -o <output.ll>`: compile Aero source to LLVM IR
- `aero run <input.aero>`: compile and run an Aero program
- `aero check <input.aero>`: type-check only (no code generation)
- `aero test`: discover and run `*_test.aero` files
- `aero fmt <input.aero>`: auto-format source
- `aero doc <input.aero> [-o <output.md>]`: generate Markdown API documentation from declarations
- `aero profile <input.aero> [-o <trace.json>]`: profile compiler stages and optionally emit Chrome trace JSON
- `aero init [path]`: create a project scaffold (`aero.toml` + `src/main.aero`)
- `aero lsp`: run the Aero language server over stdio (diagnostics, completion, hover, go-to-definition, document symbols)

## "Hello, world" (compile to LLVM IR)

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
