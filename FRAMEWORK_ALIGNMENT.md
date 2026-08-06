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

`CORE-065` is accepted public for the next bounded ownership step: exact acyclic
conditional joins over those admitted enum owners.
Mutually exclusive siblings share one entry snapshot, returning arms are excluded from
the merge, and uncertain fallthrough becomes `MaybeMoved` rather than false success or
blanket rejection. Independent checked-IR CFG dataflow proves enum owner consumption by
identity. Loop-carried changes remain fail-closed pending a fixed-point model. All eight
public checks and the pinned LLVM/Clang 22 native-exit-137 system lane pass.

Accepted `CORE-066` closes the complementary loop-local class.
Each dynamic iteration may construct or receive a fresh admitted enum owner and consume
it once through the existing binding/call/Match/return/owned-assignment contracts.
Independent checked-IR fixed-point controls require the exact definition before each
cyclic consumption. The red-first proof also closed a real runtime defect: checked
array-`for` continue now reaches a shared increment block instead of jumping directly
to the header. The tracked direct-module program executes locally at exact exit 149,
and the complete serialized root gate passes. All eight public checks pass; pinned
LLVM/Clang 22.1.8 rejects the invalid fixture, externally verifies, machine-verifies,
object-lowers, links, and executes exact exit 149, with nightly repeating exit 149.
Outer-owner backedge/exit joins, moved-target reinitialization, loop expressions/
labels, and general CFG ownership remain deliberately excluded.

Accepted public `CORE-067` advances the framework's strong-typing and
zero-cost-abstraction direction without claiming general method dispatch. One shared
classifier replaces duplicated semantic/admission topology tables, rejects unsupported
methods before checked IR, and supplies exact compile-time lowering for recursive
CopyData fixed-array `.len()` and `.is_empty()`. Existing immutable compile-time String
queries and Array/Vec `.iter()` compatibility are preserved. The tracked composed
program crosses direct modules, structs, tuples, nested arrays, semantics, checked IR,
verification, LLVM, machine verification, object/link, and exact native exit 167 on
pinned LLVM/Clang 22.1.8; nightly repeats exit 167. Runtime Strings, collections,
heap or iterator ABI, generic/trait dispatch, closures, ownership changes, stable ABI,
and accelerator execution remain excluded, consistent with Minimal Prototype /
correctness recovery.

Accepted public `CORE-068` advances the founding framework's exact function-signature,
compile-time type-checking, source-order, and typed-IR direction without declaring a
stable call ABI. One shared classifier replaces semantic and admission fallbacks that
invented `Int`, and trusted checked lowering now requires an admitted exact result
contract. The composed direct-module program crosses ordinary calls, recursive
CopyData, owned enums and Match, immutable/mutable whole-place references, control
flow, checked IR, verification, deterministic LLVM, local linking, and exact native
exit 181. All eight public checks pass; pinned LLVM/Clang 22.1.8 rejects the invalid
control, externally verifies, machine-verifies, object-lowers, links, and executes the
same exit, with nightly repeating verification and exit 181. Generic/trait/closure
calls, overloads/conversions, reference results, new ownership/lifetime behavior,
layout, stable ABI, and accelerators remain excluded.

Accepted public `CORE-069` follows the founding enum grammar directly: a variant's optional
parenthesized `type_list` can now contain two or more positional fields when every
field belongs to the already accepted recursive finite CopyData class. The same enum
classifier owns declaration, construction, Match binding, transport, and whole-owner
semantics. Checked IR and its independent verifier preserve exact ordered fields;
private LLVM uses one product lane per multi-field variant while unchanged unit and
unary schemas retain their prior identities and representation. The composed
direct-module candidate crosses structs, tuples, arrays, source-order mutation,
owned enum transport/reassignment/control flow, exhaustive Match, checked IR,
verification, deterministic LLVM, and exact exit 193. All eight public checks pass;
stable and nightly LLVM/Clang 22.1.8 independently externally verify, machine-verify,
object-lower, link, and execute exact exit 193. Named-field/generic variants, nested/
wildcard/guard patterns, enum storage/borrowing/projection, partial moves, new lifetime/
drop/CFG semantics, stable ABI, and accelerators remain excluded.

Accepted public `ARCH-002` addresses the growing annotation-policy topology without
adding a language feature. It normalizes an annotation to one leaf plus an ordered
wrapper path, then returns one shared supported, explicitly rejected, or preserved/
quarantined disposition to semantic analysis and checked admission. A
characterization-first depth-four product and byte-identical LLVM corpus protect every
accepted and quarantined boundary. All eight public checks and the unchanged pinned
native exit-193 lane pass, while every framework capability and exclusion stays fixed.

Accepted public `CORE-070` takes a bounded step toward the framework's integrated tooling
direction. Library callers can compile an exact root file, including the already
accepted root-level direct-module collector, through the same checked library frontend
as `compile_program`. It returns verified in-memory LLVM and writes no artifact. This
does not define the founding dotted-import grammar, namespaces, aliases, visibility,
recursive module graphs, cache identity, external LLVM verification, or a canonical
thin CLI; those remain separately specified work.

Candidate `CORE-071` contains the prototype's different Rust-like `use` syntax rather
than treating it as the founding import model. The parser preserves direct, aliased,
and terminal-glob declarations plus the exact keyword location, but semantics and
checked admission reject executable use consistently before IR. This adds no positive
module, namespace, visibility, alias, glob, resolver, backend, or runtime behavior.

| Framework direction | Current evidence | Required next proof |
|---|---|---|
| Clear, strongly typed source language | Numeric, function, binding, and selected control-flow slices are partial; several composite forms are parser-only or fail closed. Closures are explicitly parsed-only and cannot acquire a fabricated scalar type or reach trusted IR. Candidate CORE-071 likewise preserves Rust-like `use` syntax only for future work and rejects it before checked IR. Accepted CORE-067 closes fabricated intrinsic-method results; accepted CORE-068 similarly requires one exact named-call contract before semantic success or checked IR. Accepted ARCH-002 normalizes binding-annotation topology and phase routing without changing any accepted or quarantined type behavior. | A specified stable subset with exact positive, negative, diagnostic, and execution tests; separately freeze closure and import/name-resolution semantics before any positive path |
| Ownership-based safety | Shallow move checks remain partial. CORE-048/053 through accepted CORE-066 establish bounded immutable/mutable whole-place ownership, internal reference transport, recursive finite CopyData composition, direct CopyData owner reassignment, owned enum transport/replacement, exact acyclic conditional joins, independent enum-owner CFG consumption proof, and fresh per-iteration enum owners without transporting an outer moved owner. Mutable projections, reference results, escaping provenance, outer-owner loop joins, stable reference ABI, general CFG ownership, NLL, drop, lifetime inference, and memory-safety claims remain absent; 16 broader semantic/lossy-shape Phase 5 tests remain quarantined. | Freeze another hard module, runtime, ownership, or execution slice |
| Structs, arrays, enums, traits, and Match | CORE-043 through CORE-047 accept bounded all-Copy scalar/named-struct construction, projection, arrays, transport, and finite acyclic graphs. CORE-049 through CORE-052 accept unit/unary-scalar enums, exhaustive bound Match, and owned internal transport. CORE-058 through CORE-061 add flat tuples, whole-place references, and direct CopyData owner replacement. Accepted CORE-062 removes the executable CopyData topology whitelist. Accepted CORE-063 carries that recursive class through unary owned-enum payloads and exact bound Match under a pinned native exit-113 gate. Accepted CORE-064 adds exact whole-owner enum replacement under a pinned native exit-131 gate. Accepted CORE-065 composes those operations across acyclic `if` joins under a pinned native exit-137 gate. Accepted CORE-069 generalizes positional variants to two or more recursive CopyData fields and exact field bindings under pinned native exit 193. Generic/named-field enums, Option/Result Match, wildcard/guard/nested destructuring, enum fields/arrays/borrowing/projection, unit/unary tuples, unsupported/cyclic structs, dynamic arrays, traits, and stable aggregate/reference ABI remain open. | Separately freeze another hard generic/module/runtime/ownership class; do not infer broader enum or ABI support from CORE-069 |
| Typed SSA-style IR and LLVM backend | LLVM text and a partial CPU object/link/run path exist; typed-IR invariants and verification are incomplete | Fallible typed IR, structural verifier, LLVM verifier, object/link/runtime gates on supported platforms |
| Zero-cost performance | A benchmark protocol now exists, but no audited public Aero runtime or device performance claim passes it | Correct real programs, raw samples, reproducible baselines, and separately reported compile/runtime/resource costs |
| Modern concurrency | Interfaces and library-like helpers exist, but the language/runtime concurrency model is not end-to-end | Ownership-safe tasks/channels or another frozen model with race and runtime evidence |
| Integrated tooling | CLI, LSP, formatter, docs, project, registry, and conformance surfaces exist but are experimental and use divergent compiler paths. Accepted CORE-070 adds a checked file-aware library route over the existing direct-module collector, reducing one library/file-context gap without converging the CLI. Candidate CORE-071 closes one silent-use boundary but adds no resolver. | One canonical compiler service shared by every tool, with failure and integration tests |
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
