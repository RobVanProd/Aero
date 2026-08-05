<div align="center">
  <h1>Aero</h1>
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

The founding direction is preserved in the
[original language design paper](<__Aero___%20A%20High-Performance,%20Ergonomic%20Programming%20Language.pdf>)
and the [strategy capture](<Aero%20Programming%20Language%20Framework%20-%20Claude.pdf>).
[Framework alignment](FRAMEWORK_ALIGNMENT.md) explains how that direction maps
to current evidence and the active roadmap. Aero is presently an experimental
minimal prototype under correctness recovery; historical `v1.0.0` labels are not
evidence of a stable release.

## Verified Results

The latest public-branch verification was run on 2026-05-28 at commit
`7d6ad2f865560cdcca4e30390430a7878c65fa69` on this local machine:

- CPU: AMD Ryzen 9 9950X 16-Core Processor
- GPU 0: Radeon RX 7900 XTX (`gfx1100`, PCI device `1002:744c`)
- GPU 1: AMD Radeon Graphics (`gfx1036`, PCI device `1002:13c0`)
- PyTorch: `2.5.1+rocm6.2`
- HIP: `6.2.41133-dd7f95766`

Verified current results:

- The public-branch run completed Rust Criterion lexer-only benchmarks. The reported
  median tokenization times ranged from 282.31 ns for `tokenize_simple_io` to
  21.507 us for `tokenize_large_program`.
- The GGUF harness completed a real local llama.cpp CLI ROCm inference run with
  `HIP_VISIBLE_DEVICES=0` on the Radeon RX 7900 XTX. The model was
  `/home/rob/models/mistralai_Mistral-Small-3.1-24B-Instruct-2503-Q4_K_M.gguf`;
  one measured run reported 44.94 eval tokens/s, 186.94 prompt eval tokens/s,
  and 2701.84 ms load time
  ([result JSON](claim-verification/results/aero_gguf_llama_cli_7900xtx_20260528T224200Z/claim_result.json)).

Invalid compilation measurements:

- `benchmarks/performance_benchmark.py` passed each bare source path directly to
  the Aero executable, without a required command such as `build`. The resulting
  public-master and post-reboot timing series measured the process's successful
  usage fallthrough, not Aero compilation, and are classified as invalid. Their
  raw artifacts remain available for audit; none of those compilation numbers is
  a verified performance result.

Historical lexer evidence:

- An earlier local lexer artifact from commit
  `f263568e06073317467416c2f954c73927c00f3e` is tracked as a historical
  artifact. Its Criterion lexer tokenization times ranged from 340.38 ns to
  22.956 us. This historical lexer record is separate from the invalid Python
  timing series and is not the latest public-branch verification
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

Use a POSIX shell from a machine with Rust and the documented LLVM 22/Clang
[build prerequisites](BUILD.md#prerequisites):

```bash
git clone https://github.com/RobVanProd/Aero.git
cd Aero
cargo build --release --manifest-path src/compiler/Cargo.toml
export PATH="$PWD/src/compiler/target/release:$PATH"
aero --version
aero init my_app
cd my_app
aero check src/main.aero
aero run src/main.aero
```

The generated `src/main.aero` is:

```aero
fn main() {
    println!("Hello, Aero!");
}
```

The final command must complete successfully and print `Output: Hello, Aero!`.
See the [Windows PowerShell instructions](BUILD.md) for the equivalent Windows
build and PATH commands, and consult the [backend capability status](BACKEND_STATUS.md)
before using an accelerator target.

## Experimental Command Examples

These commands are outside the minimal Quick Start. They expose experimental
compiler surfaces with the limitations recorded in the current compiler table
below and in [backend capability status](BACKEND_STATUS.md). They are not evidence
that every backend performs device execution.

CPU is the only current process-execution target. A ROCm `run` request can probe
temporary object emission, then returns unavailable because there is no HIP link
or device launch. CUDA has no active object, link, or launch path. The ambiguous
`gpu` target is rejected; select `cpu`, `rocm`, or `cuda` explicitly.

Graph optimization is a verified textual rewrite to internal scalar helpers;
backend labels do not establish device execution. Quantization uses
scalar-`double` helpers: there is no real FP8 representation, per-channel
execution, or numerical correctness proof.

```bash
aero run --target cpu src/main.aero
aero build src/main.aero -o main.rocm.ll --target rocm --gpu gfx1100
aero doc src/main.aero -o main.md
aero profile src/main.aero -o trace.json
aero graph-opt main.ll -o main.opt.ll --backend rocm --gpu gfx1100
aero quantize main.opt.ll -o main.int8.ll --mode int8 --backend rocm --gpu gfx1100 --calibration calib.json
python benchmarks/gguf/gguf_compare.py --config benchmarks/gguf/config.rx7800xt.example.json
aero registry search vision --index registry/index.json
aero registry publish . --dry-run
aero registry install vision-core --version 0.2.0 --target pkgs --dry-run
aero conformance -o conformance_report.json
aero lsp
```

## 🛠️ Current Compiler Surface (Experimental)

| Category | Features |
|----------|----------|
| **Type System** | Static scalar checks. Generic and trait syntax is parsed but quarantined; generic substitution, trait-bound enforcement, and where-clause semantics are not supported contracts. |
| **Memory** | Ownership/reference syntax and shallow move tracking only; no borrow checker, lifetime analysis, or memory-safety guarantee. |
| **Data Types** | Struct/enum declarations and syntax, arrays, tuples, strings, pattern matching; execution limits below |
| **Control Flow** | Functions, if/else, while/for loops, break/continue, closures |
| **Direct module source collection** | Root-level `mod x;` collects `x.aero` or `x/mod.aero` into the current flattened compilation unit. `use`, `pub` visibility semantics, namespaces, recursive modules, and cycle graphs are not implemented. |
| **Codegen** | LLVM IR backend with optimization passes |
| **CLI** | `aero build`, `aero run`, `aero check`, `aero test`, `aero fmt`, `aero doc`, `aero profile`, `aero graph-opt`, `aero quantize`, `aero registry`, `aero conformance`, `aero init`, `aero lsp` |
| **LSP** | Syntax diagnostics, completion, hover, go-to-definition, document symbols |
| **Docs & Profiling** | Markdown API generation (`aero doc`), compilation stage timing + trace export (`aero profile`) |
| **Phase 8 Experimental Slice** | Textual graph rewriting to internal scalar helpers and scalar-`double` quantization helper rewriting with backend metadata. These are not device execution, real FP8/per-channel execution, or numerical-correctness evidence. The slice also includes local `registry.aero` search and dry-run planning plus 3 example cases and 4 deterministic regression checks (not formal-semantics proof). Live registry transport is quarantined pending a reviewed protocol and trust boundary. |
| **Diagnostics** | Colored errors, source snippets, "did you mean?" suggestions |

> **Tuple status:** tuple literal, tuple-index, type, and pattern syntax is
> recognized. Tuple value construction and projection are not executable yet;
> trusted compiler paths reject those expressions before IR generation with
> `Tuple expressions are not supported.`

> **Struct value status:** one bounded scalar-struct value slice and its
> all-component-`Copy` internal function-transport extension are publicly accepted.
> A unique,
> non-generic, nonempty top-level struct with unique `int`/`i32`, `float`/`f64`, or
> `bool` fields can be constructed exactly by field name inside an admitted
> top-level function; construction fields may be reordered and are evaluated once
> in written order. Direct and local immutable projection use checked aggregate IR
> and verified LLVM named types. Such a struct is Copy because every admitted field
> is Copy: local aliases preserve the original, and exact-name internal function
> parameters, arguments, call results, and returns use checked named aggregates by
> value. This includes mixed scalar/struct signatures, forwarding, terminating direct
> recursion, and the flattened one-level direct-module route. Unsupported definitions,
> shapes, annotations, and contexts remain rejected before LLVM.
>
> The publicly accepted CORE-045 slice also permits fixed local arrays of one exact
> admitted all-scalar Copy struct. It covers literal, repeat, and typed-empty origins;
> element-wise Copy aliases; static length; compile-time constant in-bounds indexing
> and projection; and compiler-bounded iteration. Exact struct schema and count survive
> distinct checked array IR into typed `[N x %aero.struct.Name]` LLVM. The tracked
> multi-file example composes this with direct-module collection and passes pinned
> LLVM/Clang 22 verification, lowering, linking, and native exit 77.
>
> The current CORE-046 candidate extends only the compiler's existing flat fixed
> `int`/`float` and all-scalar Copy-struct arrays across non-`main` internal function
> parameters and returns. Exact element identity, count, and struct schema remain in
> shared source classification, logical checked IR, verification, and aggregate LLVM
> definitions, calls, loads, stores, and returns. Caller values remain usable after
> the by-value call. The full local repository gate passes; all public checks and
> pinned native execution are still required before acceptance.
>
> This does not provide non-Copy or destructive move semantics, assignment, general
> methods beyond the exact array `.len()`/`.iter()` forms,
> destructuring, Match, nested arrays, arrays as fields, dynamic
> bounds, runtime checks, recursive aggregates, generics, visibility, separate
> compilation, stable layout/ABI/FFI, general ownership/drop/lifetimes, heap storage,
> accelerator execution, or performance guarantees. Bool, String, non-Copy, nested,
> and otherwise unsupported arrays do not gain function transport. LLVM owns internal
> padding and alignment. `main` retains exact `i32 @main()`, and other method calls
> remain a distinct AST form.

> **Pattern matching status:** `match` syntax, arms, and patterns are recognized,
> but Match value evaluation is not executable yet. Trusted parsed source bodies,
> including default trait method bodies, inspect the scrutinee and arm bodies for
> established errors, then reject the Match before IR with
> `Match expressions are not supported.` This is a fail-closed boundary, not pattern
> typing, exhaustiveness, arm selection, or Match execution support.

Formal spec: `docs/language/aero_formal_language_specification.md`

## Looking Ahead

- GGUF-native model loader and runtime benchmarks on CUDA/ROCm
- Expanded optimizer and fused-kernel library coverage
- Additional formal semantics proofs beyond deterministic conformance checks

## License
MIT © RobVanProd and contributors. See LICENSE for details.
