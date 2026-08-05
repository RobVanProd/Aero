# Aero Framework Alignment

Last reviewed: 2026-08-05 (America/New_York)

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

Accepted `CORE-061` combines a hard executable ownership slice with closure
false-success containment. Accepted `CORE-062` takes the next framework-
aligned architecture step: every finite composition of already admitted Copy scalars,
fixed arrays, arity-at-least-two tuples, and acyclic named structs is classified once,
retained exactly through checked IR, independently verified, lowered to private typed
LLVM, and exercised by a multi-module native exit-109 system gate. This follows the
framework's compositional strong-typing direction without claiming stable layout, ABI,
memory safety, generics, heap data, or accelerator execution.

Accepted `CORE-063` composes those recursive CopyData values through
the already accepted unary owned-enum and exhaustive Match architecture. It deliberately
keeps the enum non-Copy, preserves scalar layout compatibility, and adds exact typed
aggregate payload lanes without introducing multi-field variants, nested destructuring,
general enum storage/borrowing, generic enums, new CFG ownership, or a public ABI. The
pinned LLVM/Clang 22 system lane externally verifies, machine-verifies, object-lowers,
links the private non-PIE executable, and observes exact native exit 113.

Accepted `CORE-064` supplies the next ownership step: exact whole-owner
replacement of those already admitted enums. It generalizes the shared mutable-place
contract and checked identities, preserves non-Copy moves, independently verifies exact
enum place/value schema, and lowers only to the existing private enum representation.
It does not admit enum borrowing, projection, aggregate storage, partial moves, new CFG
joins, drop/lifetimes, or a stable ABI. All eight public checks pass, and the pinned
LLVM/Clang 22 lane externally verifies, machine-verifies, object-lowers, links, and
observes exact native exit 131.

| Framework direction | Current evidence | Required next proof |
|---|---|---|
| Clear, strongly typed source language | Numeric, function, binding, and selected control-flow slices are partial; several composite forms are parser-only or fail closed. Closures are explicitly parsed-only and cannot acquire a fabricated scalar type or reach trusted IR. | A specified stable subset with exact positive, negative, diagnostic, and execution tests; separately freeze closure typing/capture/call semantics before any positive closure path |
| Ownership-based safety | Shallow move checks remain partial. CORE-048/053 through accepted CORE-064 establish bounded immutable/mutable whole-place ownership, internal reference transport, recursive finite CopyData composition, direct CopyData owner reassignment, owned enum transport, and exact enum owner replacement while preserving immutable-reference copy behavior without storing references as CopyData. Mutable projections, reference results, escaping provenance, stable reference ABI, general CFG ownership, NLL, drop, lifetime inference, and memory-safety claims remain absent; 16 broader semantic/lossy-shape Phase 5 tests remain quarantined. | Freeze another hard CFG-aware ownership, module, runtime, or execution slice |
| Structs, arrays, enums, traits, and Match | CORE-043 through CORE-047 accept bounded all-Copy scalar/named-struct construction, projection, arrays, transport, and finite acyclic graphs. CORE-049 through CORE-052 accept unit/unary-scalar enums, exhaustive bound Match, and owned internal transport. CORE-058 through CORE-061 add flat tuples, whole-place references, and direct CopyData owner replacement. Accepted CORE-062 removes the executable CopyData topology whitelist. Accepted CORE-063 carries that recursive class through unary owned-enum payloads and exact bound Match under a pinned native exit-113 gate. Accepted CORE-064 adds exact whole-owner enum replacement under a pinned native exit-131 gate. Generic/multi-field enums, Option/Result Match, wildcard/guard/nested destructuring, enum fields/arrays/borrowing/projection, unit/unary tuples, unsupported/cyclic structs, dynamic arrays, traits, and stable aggregate/reference ABI remain open. | Address another hard generic/module/runtime/ownership class under separately frozen semantics |
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
