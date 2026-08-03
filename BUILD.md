# Build & Quickstart

This repo contains the Aero compiler written in Rust (`src/compiler`).

## Prerequisites

- Rust toolchain (stable): <https://rustup.rs>
- LLVM 22 tooling available on your PATH:
  - `clang`
  - `llc`
  - `opt` (preferred verifier)
  - `llvm-as` (fallback verifier)

At least one verifier (`opt` or `llvm-as`) is required before native lowering.
Confirm `clang --version`, `llc --version`, and either `opt --version` or
`llvm-as --version` report major version 22. Package names and installation
locations vary by platform; the stable Linux CI contract installs `clang-22` and
`llvm-22`, then places `/usr/lib/llvm-22/bin` first on PATH. On Windows, install an
LLVM 22 distribution from <https://llvm.org/> and use its `bin` directory.

## Build the compiler

From the repo root:

```bash
cargo build --release --manifest-path src/compiler/Cargo.toml
```

The compiler binary will be at:

- `src/compiler/target/release/aero` (Linux/macOS)
- `src\compiler\target\release\aero.exe` (Windows)

### Install (optional)

```bash
cargo install --path src/compiler
```

This installs `aero` into your Cargo bin directory (typically `~/.cargo/bin`).

## Windows PowerShell

From the repository root, build the compiler and add its release directory to
the current PowerShell session's PATH:

```powershell
cargo build --release --manifest-path src/compiler/Cargo.toml
$env:PATH = "$PWD\src\compiler\target\release;$env:PATH"
aero.exe --version
```

The compiler executable is `src\compiler\target\release\aero.exe`. Before using
`aero run`, ensure `clang.exe`, `llc.exe`, and either `opt.exe` or `llvm-as.exe`
from the same LLVM 22 distribution are also on PATH.

## CLI command summary

- `aero build <input.aero> -o <output.ll> [--target <cpu|rocm|cuda|gpu>] [--backend <cpu|rocm|cuda|gpu>] [--gpu <arch>]`: compile Aero source to LLVM IR with optional accelerator target metadata
- `aero run <input.aero> [--target <cpu|rocm|cuda|gpu>] [--backend <cpu|rocm|cuda|gpu>] [--gpu <arch>]`: compile and run an Aero program (artifacts are emitted under `target/aero-run`; ROCm path currently emits target object for HIP integration)
- `aero check <input.aero>`: type-check only (no code generation)
- `aero test`: discover and run `*_test.aero` files
- `aero fmt <input.aero>`: auto-format source
- `aero doc <input.aero> [-o <output.md>]`: generate Markdown API documentation from declarations
- `aero profile <input.aero> [-o <trace.json>]`: profile compiler stages and optionally emit Chrome trace JSON
- `aero graph-opt <input.ll> -o <output.ll> [--backend <cpu|cuda|rocm>] [--gpu <arch>] [--annotation-only]`: apply graph compilation with executable fused-kernel lowering
- `aero quantize <input.ll> -o <output.ll> --mode <int8|fp8-e4m3|fp8-e5m2> [--backend <cpu|cuda|rocm>] [--gpu <arch>] [--calibration <file>] [--per-channel] [--annotation-only]`: apply hardware-calibrated INT8/FP8 lowering interfaces
- `aero registry <subcommand>`: search a local index or create credential-free, network-free publish/install previews with `--dry-run`; live transport is quarantined pending a reviewed protocol and trust boundary
- `aero conformance [-o <report.json>]`: run 3 example cases and 4 deterministic regression checks (not a formal semantics proof)
- `aero init [path]`: create a project scaffold (`aero.toml` + `src/main.aero`)
- `aero lsp`: run the Aero language server over stdio (diagnostics, completion, hover, go-to-definition, document symbols)

Formal specification:
- `docs/language/aero_formal_language_specification.md`

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

## GGUF comparison benchmarking

Use the cross-framework harness to collect 20-run metrics and chart results:

```bash
# Synthetic validation (no model required)
python benchmarks/gguf/gguf_compare.py --config benchmarks/gguf/config.mock.json

# Real benchmark (edit backend commands as needed)
python benchmarks/gguf/gguf_compare.py --config benchmarks/gguf/config.rx7800xt.example.json

# Run only ROCm-named backends from a mixed config
python benchmarks/gguf/gguf_compare.py --config benchmarks/gguf/config.rx7800xt.example.json --backend rocm
```

Outputs are written under `benchmarks/results/gguf/` as JSON, Markdown, and HTML reports.
