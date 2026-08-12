# Aero Conformance Plan

## Test layers

1. Lexer tests cover golden token streams, source positions, Unicode, malformed
   input, unterminated constructs, and arbitrary-input panic resistance.
2. Parser tests cover each grammar production, invalid forms, multi-error
   recovery, span preservation, round-trip formatting, and arbitrary-input panic
   resistance.
3. Semantic tests cover positive typing, unknown-type rejection, explicit
   coercions, scopes, shadowing, calls, generics, traits, pattern exhaustiveness,
   control-flow returns, moves, borrows, and mutability.
4. Typed-IR tests snapshot types and control flow and run verifiers for unresolved
   names/types, definitions, terminators, dominance, and determinism.
5. Backend tests verify LLVM, produce objects and executables, check exact output
   or exit status, compare optimization levels, and exercise link failures.
6. Tooling tests confirm that CLI, formatter, LSP, docs, project manifests, and
   package operations use the canonical compiler path and surface failures.

## Required test forms

- Positive programs that identify the supported feature and expected phase.
- Compile-fail programs that assert failure phase, category, relevant span,
  essential message, and absence of executable output.
- Until remainder semantics are frozen, `%` is a required compile-fail corpus
  category across integer, float, mixed, nested, zero-RHS, and direct-module forms;
  tests must prove semantic failure, no unwind, and no requested artifact.
- Until complete tuple layout and projection semantics exist, tuple literal and
  tuple-index value expressions are a required recursive compile-fail category,
  including representative nested, function, root/direct-module, public-library,
  and CLI no-artifact forms. Tuple parsing and grouped scalar controls remain
  positive parser/semantic evidence, not tuple execution evidence.
- Until complete struct field typing/layout/projection exists, named field-access
  value expressions are a required recursive compile-fail category. Tests must
  preserve parser distinction from method calls and tuple indexing, retain prior
  receiver diagnostics, cover public/direct-module/CLI no-artifact routes, and keep
  array/index/iterator and tuple-free controls positive.
- Until complete struct name/field typing, layout, initialization, ownership, ABI,
  and lowering exist, StructLiteral values are a required recursive compile-fail
  category. Tests must retain declaration and parser-shape controls, visit field
  children in source order, preserve established child diagnostics, cover ordinary
  and default/nested source containers, and prove public/direct-module/CLI rejection
  without unwind, panic, or requested artifact.
- Until closure typing, capture, callable ABI, transport, lifetime, and invocation
  semantics are frozen, closure expressions are a required source-located compile-fail
  category. Retain parser shape and opening-pipe locations, reject before checked IR,
  independently reject unanalyzed AST, and prove no closure symbol/layout/LLVM or CLI
  artifact for inferred/explicit bindings, comparisons, arguments, returns,
  array/struct storage, captures, and calls.
- The CAP-010 required-only trait-dispatch slice must retain positive composition over
  multiple traits, concrete recursive CopyData structs, required methods, extra
  CopyData arguments, CopyData/`Void` results, multiple bounds/type parameters,
  owner reuse, direct modules, the representative program, and public CLI/native
  execution. Negative source and no-artifact CLI controls must reject every excluded
  declaration/impl/bound/call family before checked IR. Independent verifier
  corruption controls must reject helper identity, receiver target/mode/provenance,
  argument/result schema, callee, arity, and order changes. This is not a conformance
  claim for general traits or generics.
- The accepted CAP-011 fixed-window slice must retain one shared schema authority for
  generic-struct signature handoff, nested inference, structural body use, and
  concrete pre-IR specialization. Positive evidence covers recursive CopyData
  substitutions, multiple parameters, direct generic sides, reads, functional
  updates, modules, and the representative program. Negative evidence covers every
  frozen signature and body-use family in semantic and raw checked routes. Private
  identity mismatch and the existing concrete verifier corruption controls remain
  mandatory. Pinned Linux/Windows native gates must execute `int` and `char`
  specializations and reject the complete lower/upper-bound read/write product at
  `-O0` and `-O2`. This is not general generic or dynamic collection conformance.
- The accepted CAP-012 projected-call-loan slice must retain one shared projected-
  place classifier across assignment, semantic admission, and raw checked admission.
  Positive evidence covers every finite nonempty field/tuple/fixed-array path,
  literal and once-evaluated checked runtime selectors, recursive CopyData leaves,
  immutable and mutable loans, multiple roots, immutable projections from by-value
  CopyData parameters, owner reuse, modules, and the representative application.
  Negative evidence covers excluded roots, mutability/type mismatches, complete-root
  conflicts, selectors, temporaries, and stored aliases. Nine independent verifier
  mutations must reject root/source/type/mutability/call-operand/end corruption, and
  pinned Linux/Windows `-O0`/`-O2` gates must execute the product and reject both
  runtime-bounds failure directions. This is not stored-reference, general alias,
  lifetime/drop, ABI, or memory-safety conformance.
- Acyclic conditional ownership tests must distinguish mutually exclusive sibling
  consumption from post-merge uncertainty. For each admitted enum schema, cover
  missing else, both fallthrough arms, definitely returning arms, nested else-if,
  shadowing, multiple owners, mutable whole-owner replacement, Match/call/return use,
  and direct modules. Checked-IR corruption controls must reject serial, partial-merge,
  and cyclic double consumption. Loop-carried ownership changes remain compile-fail
  cases until a separately frozen fixed-point contract exists.
- Fresh loop-local enum tests must cover every checked statement loop (`while`, fixed-
  array `for`, and `loop`), zero/one/multiple iterations, nested nearest-loop targets,
  fallthrough, return, break, and continue; inferred/exact and immutable/mutable fresh
  bindings; constructor/call origins; every admitted payload schema and consumption;
  and exact `for` increment-before-header behavior. Corruption controls must accept a
  cyclic consumption only when every path executes its exact fresh result/place
  definition first, and reject bypass, within-iteration double consumption, or any
  unreset pre-loop owner. Outer-owner break/backedge joins and moved-target
  reinitialization remain compile-fail categories.
- Runtime-output tests with exact stdout, stderr, exit code, and declared sources
  of nondeterminism.
- Diagnostic snapshots normalized only for unstable machine paths or equivalent
  environmental details.
- Differential tests for a bounded, deterministic, well-typed stable subset:
  reference behavior equals unoptimized output equals optimized output.
- Fuzz targets for lexer, parser, semantic entry points, typed-IR verification,
  and bounded well-typed differential programs.
- Backend-equivalence tests with defined numerical tolerances and explicit proof
  that the selected hardware path ran.

## Real-program progression

The release suite has reached bounded generic-data-structure and ownership-intensive
telemetry stages. It will continue through file processing, a small parser/interpreter, a CPU
numerical workload; then real vector, matrix, and tensor accelerator workloads
after their runtime contracts exist.

## Platform matrix

- Required trusted core: Linux x86_64 and Windows x86_64 on pinned stable Rust and
  LLVM versions.
- Additional CPU targets are experimental until build, link, and runtime tests
  run in CI.
- ROCm and CUDA have independent compiler, object, link, hardware-execution,
  correctness, and performance gates recorded in `BACKEND_STATUS.md`.

## Release gates

- `./tools/test.sh` passes without test exclusions introduced by the release.
- Every stable matrix row is specified and end-to-end with positive, negative,
  diagnostic, and runtime evidence.
- All stable README/tutorial examples are executable tests.
- Generated LLVM for stable real programs verifies and links on every supported
  platform.
- No known invariant violation or unclassified high-severity defect remains in
  the stable core.
- Reproducibility, versioning, deprecation, installation, migration, artifact,
  and claims checks pass.

The current `conformance` command's deterministic checks are useful regression
tests. They are not, by repetition alone, a mechanized proof of formal semantics.

The accepted `CORE-010` production implementation routes conformance cases through checked
IR and mandatory internal verification without depending on an external LLVM tool.
A checked-IR failure is recorded in the complete requested report and produces a
nonzero result. Focused tests, the complete repository gate, three exact-diff
reviews, and all required public CI checks pass at head `db349ef`.

## Current integration checkpoint

Accepted CAP-012 adds one nonescaping projected CopyData call-loan conformance slice.
Positive evidence composes immutable and mutable nested field/tuple/fixed-array loans,
checked runtime selectors, multiple roots, recursive CopyData leaves, immutable
projections from by-value CopyData parameters, owner reuse, modules, and the
representative telemetry application. Semantic
and raw checked routes share the place classifier; exact checked identities and nine
corruption mutations prove root/source/type/mutability/call-lifecycle integrity before
LLVM. Focused 3/3, representative 3/3, the 242-library/35-binary complete gate, all
candidate-head results, protected PR #46 merge `49bcdfc3`, and exact merge-head
CI/Rust CI/CodeQL pass, including pinned LLVM/Clang 22 native `-O0`/`-O2` execution.
This is a bounded `PARTIAL` ownership slice, not stored references, general aliasing,
lifetimes/drop, stable ABI, or memory-safety conformance.

Accepted CAP-011 passes focused 4/4, representative 3/3, its private identity
mismatch control, check/doc/format/diff, and the complete repository gate with 241
library tests plus every integration and doc target. Exact candidate `dea5714e`, all
nine candidate-head results, protected PR #44 merge `34b81eee`, and exact merge-head
CI/Rust CI/CodeQL pass, including pinned Windows LLVM 22 native execution. This is a
bounded `PARTIAL` generic-container slice, not general generics or collections.

Accepted CAP-010 adds one required-only trait-dispatch conformance slice over
nongeneric recursive finite CopyData structs and direct whole-value generic parameters.
Positive evidence composes multiple traits, concrete impls, immutable receivers,
CopyData/`Void` signatures, owner reuse, modules, the representative application,
checked helper identities, verified LLVM, and native execution. Negative and verifier
corruption controls keep every excluded declaration/impl/bound/call form before trusted
IR. Focused 3/3, representative 3/3, the complete repository gate, all nine exact-
candidate public results, protected PR #42 integration, and exact merge-head CI/Rust
CI/CodeQL pass. This is a `PARTIAL` bounded class, not general trait/generic, ABI,
safety, or stability conformance.

Accepted CAP-009 adds a separately selected `stable-scalar-v0` conformance
lane. One exhaustive post-parse classifier must reject the complete complement of its
frozen scalar AST before semantics, checked IR, cache lookup, or artifacts. Positive
evidence runs the profile application and wrapping corpus through public `check`,
verified `build`, and `run`; rejects experimental numeric LLVM; externally verifies
LLVM and machine code; and compares native `-O0`/`-O2` results on pinned Linux and
Windows LLVM/Clang 22. Non-CPU and `--gpu` profile pairings fail before compilation.
Focused 10/10, the complete repository gate, all nine candidate-head checks, protected
PR #40 integration, and exact merge-head CI/Rust CI/CodeQL pass. This accepts only the
selected scalar profile as `STABLE`; no whole-language, release, ABI, safety, or
experimental-default stability gate is claimed.

`CORE-059` and `CORE-060` are accepted public for immutable and exclusive mutable
whole-place references over exact admitted Copy-data places, with pinned LLVM/Clang 22
verification and exact native exits 37 and 59. `CORE-061` extends only direct mutable
whole-owner reassignment across that same classified universe, without admitting
projected targets, mixed alias signatures, reference results, stable ABI, or general
lifetime/memory-safety claims.

The CORE-061 conformance gate traces a tracked two-module assignment program through lexing,
parsing, semantic ownership, exact recursive checked IR, independent verifier
corruptions, typed LLVM, external and machine verification, object/link, and exact
native exit 83. Local Clang execution is supporting evidence only; public acceptance
requires the pinned LLVM/Clang 22 lane and all eight repository checks. This composed
gate is periodic architecture evidence, not proof that every Aero language subsystem
or release criterion is coherent.

The authorized CORE-061 closure amendment is a negative architecture control within
the same milestone, not its executable capability. The focused 7-test matrix and the
complete 175/175 library plus 181/181 binary test surface prove that parsed closures
fail consistently before checked IR and cannot manufacture callable or LLVM state.
The exact amended repository-root gate passes. Public workflows and the pinned system
lane remain acceptance requirements.

Accepted CORE-064 supplies the next periodic architecture specimen: a
tracked two-module program composes unit, scalar, array/tuple/struct-payload enums with
constructor, call-result, and distinct-local replacement, then Match/call/return use.
Focused tests trace exact source semantics, checked owned-place identities, independent
corruption controls, private typed LLVM, CLI artifact hygiene, and the pinned exit-131
workflow. The exact root gate, all eight public checks, and LLVM/Clang 22 external
verification, machine verification, object/link, and exact native exit 131 pass. This
proves only the frozen whole-owner enum replacement boundary.
