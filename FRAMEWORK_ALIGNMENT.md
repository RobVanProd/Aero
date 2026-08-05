# Aero Framework Alignment

Last reviewed: 2026-08-04 (America/New_York)

## Source basis

Aero's founding direction is preserved in two tracked artifacts:

- [`__Aero___ A High-Performance, Ergonomic Programming Language.pdf`](<__Aero___%20A%20High-Performance,%20Ergonomic%20Programming%20Language.pdf>)
  is the primary nine-page language vision and high-level implementation path.
- [`Aero Programming Language Framework - Claude.pdf`](<Aero%20Programming%20Language%20Framework%20-%20Claude.pdf>)
  preserves the opening of a strategy discussion about execution quality and a
  killer application.

The second artifact is incomplete. It is a single browser-printed page, begins
with off-page conversation text in its extractable content, and visibly ends in
the middle of the memory-safety measurement section. Its preserved recommendations
are useful, but missing pages or conclusions must not be inferred.

## Authority and interpretation

The source PDFs define intended outcomes and project direction. They are not
evidence that a feature is implemented. Aero uses the following authority order:

1. Accepted specifications and RFCs define intended language semantics.
2. `SPEC_IMPLEMENTATION_MATRIX.md` and `BACKEND_STATUS.md` state what current
   source paths and backends actually do.
3. Tests, retained artifacts, and reproducible runs provide implementation
   evidence.
4. `DECISION_LOG.md`, `TASK_LEDGER.md`, and `PROJECT_STATE.md` control active
   engineering boundaries.
5. `Roadmap.md` sequences future work; it does not certify completion.

When a vision claim and current evidence differ, public documentation must say
that the feature is designed, parsed, partial, experimental, or absent. Syntax
recognition alone is never implementation evidence.

## Founding direction retained

The primary framework establishes these durable goals:

- a statically typed, ahead-of-time compiled systems language combining native
  performance with clear, productive syntax;
- explicit simplicity and zero-cost abstractions;
- memory and concurrency safety without a tracing garbage collector, based on
  ownership and borrowing;
- composition through structs, algebraic data types, pattern matching, traits,
  and generics instead of inheritance-heavy object models;
- a lean standard library plus an integrated package, editor, formatting, and
  documentation toolchain;
- an LLVM-first bootstrap compiler, eventually self-hosted;
- transparent open-source governance through public RFCs and working teams; and
- an implementation progression of Design -> Minimal Prototype -> Self-Host ->
  Stabilize -> Optimize.

`LANGUAGE_VISION.md` specializes this direction for coherent CPU and accelerator
programming. That is a strategic extension of the original high-performance,
data-pipeline, and AI-infrastructure direction, not proof of current GPU support.

## Current alignment and gaps

| Framework direction | Current evidence | Required next proof |
|---|---|---|
| Clear, strongly typed source language | Numeric, function, binding, and selected control-flow slices are partial; several composite forms are parser-only or fail closed | A specified stable subset with exact positive, negative, diagnostic, and execution tests |
| Ownership-based safety | Shallow move/borrow checks exist; 22 active strict Phase 5 tests cover lexer/parser retention only, while 16 semantic/lossy-shape tests remain quarantined and lifetime provenance is absent | Active CFG-aware move, borrow, lifetime, and destruction conformance |
| Structs, enums, traits, and Match | CORE-043 provides an accepted bounded scalar-struct value slice. Accepted CORE-044 follows the framework's all-components-Copy rule for local aliases and exact by-value internal function transport through shared source classification, logical checked IR, verification, LLVM, and a pinned exit-63 public native gate; non-Copy ownership, stable ABI, recursive aggregates, enums, traits, and Match remain open | Extend aggregate composition or non-Copy ownership without broadening layout/ABI semantics, then deliver an ADT/Match slice with the same proof depth |
| Typed SSA-style IR and LLVM backend | LLVM text and a partial CPU object/link/run path exist; typed-IR invariants and verification are incomplete | Fallible typed IR, structural verifier, LLVM verifier, object/link/runtime gates on supported platforms |
| Zero-cost performance | A benchmark protocol now exists, but no audited public Aero runtime or device performance claim passes it | Correct real programs, raw samples, reproducible baselines, and separately reported compile/runtime/resource costs |
| Modern concurrency | Interfaces and library-like helpers exist, but the language/runtime concurrency model is not end-to-end | Ownership-safe tasks/channels or another frozen model with race and runtime evidence |
| Integrated tooling | CLI, LSP, formatter, docs, project, registry, and conformance surfaces exist but are experimental and use divergent compiler paths | One canonical compiler service shared by every tool, with failure and integration tests |
| Open governance | MIT licensing, a code of conduct, community guidance, and an RFC template are tracked | A functioning public proposal/review process tied to compatibility and release decisions |
| Self-hosting | The bootstrap compiler is written in Rust | A sufficiently expressive and stable language core, then a staged Aero compiler bootstrap with reproducibility checks |

The project is therefore in **Minimal Prototype / correctness recovery**, not
Stabilize or Optimize. Historical `v1.0.0` and completed-phase labels are not
accepted evidence of maturity.

## Execution-quality scorecard

Execution quality is measured across independent gates rather than a single test
count or speed number:

1. **Language correctness:** specified behavior, compile-fail coverage, exact
   runtime results, no false success, no compiler panic, and no artifact for an
   invalid program.
2. **Compiler integrity:** deterministic phase outputs, fallible typed IR,
   verifier-clean LLVM, valid objects/executables, and equivalent optimized and
   unoptimized results.
3. **Safety:** active ownership/borrow tests, fuzz and property testing, sanitizer
   runs for the compiler/runtime, and explicit unsafe boundaries.
4. **Performance:** correctness-gated compilation time, runtime, peak memory,
   binary size, and energy where measurable, with raw samples and environmental
   controls under `BENCHMARK_PROTOCOL.md`.
5. **Developer experience:** real-project builds, accurate source spans,
   actionable diagnostics, editor behavior, reproducible dependencies, and a
   canonical CLI/library/tooling pipeline.
6. **Portability and reproducibility:** pinned toolchains, immutable inputs and
   artifacts, Linux and Windows core gates, and separately proven accelerator
   targets.

Large suites such as SPEC CPU, PARSEC, or concurrency benchmarks become useful
only after Aero can compile the required programs correctly. Until then, small
vertical programs and differential tests are the authoritative measures.

## Killer-application direction

The preserved strategy artifact identifies AI/ML infrastructure as the strongest
initial adoption domain. Aero will treat that as the lead wedge, while retaining
systems, data-pipeline, game/graphics, and embedded use cases from the primary
framework.

The first flagship must be an **Aero-native, reproducible infrastructure
workload**, not a wrapper around an unrelated runtime. A suitable progression is:

1. a correct CPU reference workload for binary/tensor data ingestion and a small
   quantized numerical kernel;
2. an end-to-end streaming or inference component that exercises structs, enums,
   ownership, error handling, collections, and parallel work;
3. comparison with equivalent established implementations under one correctness
   oracle and matched measurement boundaries; and
4. optional ROCm/CUDA lowering only after captured proof of Aero-generated object,
   transfer, launch, synchronization, result equivalence, and fallback behavior.

The tracked GGUF/llama.cpp result is valuable external reference evidence. It is
not Aero execution and cannot satisfy the flagship or backend gates by itself.

The flagship is eligible when it demonstrates a distinctive Aero advantage,
fits the proven language surface, has a reproducible baseline, exposes failures
honestly, and can grow by independently testable vertical slices.
