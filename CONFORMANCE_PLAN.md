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

The release suite will grow through: a basic CLI program; functions and
recursion; structs and enums; pattern matching; a generic data structure; an
ownership-intensive program; file processing; a small parser/interpreter; a CPU
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
