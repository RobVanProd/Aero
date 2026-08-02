<div align="center">
  <h1>Aero v1.0.0</h1>
  <p><strong>Experimental systems language and compiler repository</strong></p>
  <a href="https://github.com/RobVanProd/Aero/stargazers">
    <img src="https://img.shields.io/github/stars/RobVanProd/Aero?style=social" alt="GitHub stars">
  </a>
  <a href="https://opensource.org/licenses/MIT">
    <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="MIT License">
  </a>
  <a href="https://github.com/RobVanProd/Aero/actions/workflows/ci.yml">
    <img src="https://github.com/RobVanProd/Aero/actions/workflows/ci.yml/badge.svg" alt="CI Status">
  </a>
</div>

Aero contains a compiler, language examples, benchmark harnesses, and
experimental GPU/runtime interfaces. This README only lists benchmark claims
that are backed by tracked artifacts under
[`claim-verification/`](claim-verification/).

## Verified Results

The latest public-branch verification was run on 2026-05-28 at commit
`7d6ad2f865560cdcca4e30390430a7878c65fa69` on this local machine:

- CPU: AMD Ryzen 9 9950X 16-Core Processor
- GPU 0: Radeon RX 7900 XTX (`gfx1100`, PCI device `1002:744c`)
- GPU 1: AMD Radeon Graphics (`gfx1036`, PCI device `1002:13c0`)
- PyTorch: `2.5.1+rocm6.2`
- HIP: `6.2.41133-dd7f95766`

Verified current results:

- `bash ./scripts/run_performance_benchmarks.sh` completed with exit code 0
  on the public `master` branch. The Python harness measured 19 compilation
  benchmark cases with mean times from 0.0401563915 s to 0.6799075492 s. The
  highest mean was `function_performance.aero`, whose median was 0.0417738980 s
  and whose max run was 6.4224741870 s.
- The same run completed Rust Criterion lexer-only benchmarks. The reported
  median tokenization times ranged from 282.31 ns for `tokenize_simple_io` to
  21.507 us for `tokenize_large_program`.
- The GGUF harness completed a real local llama.cpp CLI ROCm inference run with
  `HIP_VISIBLE_DEVICES=0` on the Radeon RX 7900 XTX. The model was
  `/home/rob/models/mistralai_Mistral-Small-3.1-24B-Instruct-2503-Q4_K_M.gguf`;
  one measured run reported 44.94 eval tokens/s, 186.94 prompt eval tokens/s,
  and 2701.84 ms load time
  ([result JSON](claim-verification/results/aero_gguf_llama_cli_7900xtx_20260528T224200Z/claim_result.json)).
- An earlier local benchmark artifact from commit
  `f263568e06073317467416c2f954c73927c00f3e` is tracked as a historical
  artifact. It completed `bash ./scripts/run_performance_benchmarks.sh` on
  the Radeon RX 7900 XTX system and reported Python harness compilation mean
  times from 0.0385207161 s to 0.0473603038 s plus Criterion lexer
  tokenization times from 340.38 ns to 22.956 us. This historical artifact is
  not the latest public-branch verification
  ([result JSON](claim-verification/results/aero_post_reboot_7900xtx_20260528T190000Z/claim_result.json)).

Blocked or omitted claims:

- GPT-2 training vs PyTorch is omitted because this repo does not contain a
  fresh Aero GPT-2 training artifact from the current public branch.
- GPU 4096x4096 Aero matmul speedup is omitted because no current public-branch
  Aero matmul artifact or rerun verified it.
- NCCL/MPI multi-GPU scaling is omitted because no current public-branch
  multi-GPU scaling artifact or rerun verified it.
- Aero-vs-llama.cpp-vs-PyTorch GGUF comparison claims are omitted because this
  verification captured only a single llama.cpp CLI ROCm backend run.
- HIP/vector-add claims are omitted here because no current Aero artifact or
  rerun in this repo verified them.

## 📦 Quick Start

```bash
git clone https://github.com/RobVanProd/Aero.git
cd Aero
cargo build --release
export PATH="$PWD/target/release:$PATH"

# Initialize a new project scaffold
aero init my_app
cd my_app

# Compile + run
aero run src/main.aero

# ROCm-targeted compile path
aero run --target rocm --gpu gfx1100 src/main.aero

# Backend alias form (equivalent to --target)
aero run --backend rocm --gpu gfx1100 src/main.aero

# Auto-detect local GPU backend (ROCm/CUDA/CPU fallback)
aero run --target gpu src/main.aero

# Type-check only (no codegen)
aero check src/main.aero

# Generate Markdown API docs from source
aero doc src/main.aero -o main.md

# Profile compilation pipeline and export trace JSON
aero profile src/main.aero -o trace.json

# Apply graph compilation with executable fusion (CPU/CUDA/ROCm)
aero graph-opt main.ll -o main.opt.ll --backend rocm --gpu gfx1100

# Apply hardware-calibrated quantization lowering (INT8/FP8)
aero quantize main.opt.ll -o main.int8.ll --mode int8 --backend rocm --gpu gfx1100 --calibration calib.json

# Run the GGUF benchmark harness when configured locally
python benchmarks/gguf/gguf_compare.py --config benchmarks/gguf/config.rx7800xt.example.json

# Registry search (offline index or live transport)
aero registry search vision --live --registry https://registry.aero/api/v1

# Run formal conformance + mechanized checks
aero conformance -o conformance_report.json

# Language server for editor integration (stdio)
aero lsp
```

Try the flagship example directly in the Interactive Playground:

```aero
use aeronum::Array;
use aeronn::{Transformer, Sequential};

fn main() {
    let mut model = Transformer::new(layers: 6, dim: 384, heads: 6);
    model.to("distributed", 4);
    // Training behavior depends on the available runtime backend.
}
```

## 🛠️ Compiler Features (v1.0.0)

| Category | Features |
|----------|----------|
| **Type System** | Static typing, generics, trait bounds, where clauses |
| **Memory** | Ownership, move semantics, shared & mutable references, borrow checker |
| **Data Types** | Structs, enums, arrays, tuples, strings, pattern matching |
| **Control Flow** | Functions, if/else, while/for loops, break/continue, closures |
| **Modules** | `mod`/`use` imports, `pub` visibility, multi-file projects |
| **Codegen** | LLVM IR backend with optimization passes |
| **CLI** | `aero build`, `aero run`, `aero check`, `aero test`, `aero fmt`, `aero doc`, `aero profile`, `aero graph-opt`, `aero quantize`, `aero registry`, `aero conformance`, `aero init`, `aero lsp` |
| **LSP** | Syntax diagnostics, completion, hover, go-to-definition, document symbols |
| **Docs & Profiling** | Markdown API generation (`aero doc`), compilation stage timing + trace export (`aero profile`) |
| **Phase 8 Runtime Slice** | Hardware-calibrated INT8/FP8 lowering (CPU/CUDA/ROCm), executable fused-kernel backend generation, live `registry.aero` transport/auth/trust model, formal conformance + mechanized checks |
| **Diagnostics** | Colored errors, source snippets, "did you mean?" suggestions |

> **Tuple status:** tuple literal, tuple-index, type, and pattern syntax is
> recognized. Tuple value construction and projection are not executable yet;
> trusted compiler paths reject those expressions before IR generation with
> `Tuple expressions are not supported.`

> **Named field status:** `value.field` syntax is recognized, but named field
> projection is not executable yet. Trusted compiler paths reject field-access
> value expressions before IR generation with
> `Field access expressions are not supported.` Method calls use a distinct AST
> form and are not part of this field-access boundary.

> **Pattern matching status:** `match` syntax, arms, and patterns are recognized,
> but Match value evaluation is not executable yet. Analyzed expression paths inspect
> the scrutinee and arm bodies for established errors, then reject the Match with
> `Match expressions are not supported.` A known candidate gap remains for parsed
> default trait method bodies, which currently bypass that preflight; Match must not
> be treated as executable or safe there.

Formal spec: `docs/language/aero_formal_language_specification.md`

## Looking Ahead

- GGUF-native model loader and runtime benchmarks on CUDA/ROCm
- Expanded optimizer and fused-kernel library coverage
- Additional formal semantics proofs beyond deterministic conformance checks

## License
MIT © RobVanProd and contributors. See LICENSE for details.
