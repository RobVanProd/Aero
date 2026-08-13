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
- The accepted CAP-013 specialization-authority slice must retain one recursive
  canonical type key and one deterministic struct -> enum -> function phase plan for
  semantic and raw checked admission. Positive evidence must mix `int`/`i32` and
  `float`/`f64` through every already-admitted generic struct, enum, function,
  fixed-capacity container, and bounded trait-signature path while emitting one
  private identity. Separation controls must preserve Char versus Int, Bool, user
  names, tuple order/arity, array counts, feature namespaces, and schema/signature
  identity. Canonical framing corruption, repeated normalization, declaration-order
  permutation, representative execution, and pinned Linux/Windows LLVM 22 O0/O2
  evidence remain mandatory. This is not general generic/trait, reference
  specialization, collection, ABI/layout, allocation/drop, or stability conformance.
- Historical evidence for the accepted CAP-014 `exact-i32-array-v0` slice must retain
  one shared profile
  authority for both source and checked logical array roles and an independent
  fail-closed backend instruction boundary. Positive evidence must cover flat
  `[int; N]`/`[i32; N]` with `1 <= N <= i32::MAX`, explicitly annotated immutable
  literal locals, by-value nongeneric parameters, identifier call transport, direct
  scalar indexing, the exit-91 kernel, and the exit-93 wrapping specimen. That
  retained checkpoint records CAP-014's originally excluded mutable, write, and
  construction cases without treating them as the current selected-profile boundary;
  current negative evidence must exhaust the families still excluded after CAP-018
  and CAP-019 and preserve exact `stable-scalar-v0` rejection plus experimental byte
  parity. LLVM-shape evidence must require `[N x i32]`, wrapping `mul`/`add`
  without `nsw`/`nuw`, and no aggregate `double`, conversion, vector, or excluded
  checked instruction. Every dynamic access must have one identity-linked signed
  lower/upper guard, trap branch before GEP, and `sext i32`; constant indexes must add
  no dynamic guard. Negative and equal-to-count runtime controls must trap. Public
  library and CLI `check`/verified-`build`/`run`, external LLVM verification, machine
  verification, and native `-O0`/`-O2` execution are required on pinned Linux and
  Windows LLVM/Clang 22. This is selected-profile conformance only, not broad array,
  ABI/layout, SIMD, tensor, performance, accelerator, safety, or stability conformance.
- Historical CAP-018 checkpoint evidence must retain one shared immutable array-value
  classifier over the existing flat nonempty exact-`Int` shape. Positive evidence
  covers literal, identifier, and acyclic ordinary-call roots across explicit return,
  inferred and annotated immutable bindings, ordinary-call arguments, and
  literal/identifier/call index objects; computed elements, `int`/`i32` aliases, and
  original-source preservation are mandatory. Its retained historical negatives
  record the mutable binding/result/write boundary before CAP-019; current negative
  separation retains only the mutable forms CAP-019 still excludes, plus recursion,
  repeat/zero/nested/non-Int and mismatched arrays, unsupported roots, stable-profile
  rejection, and experimental controls.
  Raw and semantic checked evidence, corruption controls, exact `[N x i32]`
  definition/call/return anchors, public required-verifier routes, and pinned
  Linux/Windows LLVM/Clang 22 `-O0`/`-O2` execution must agree. This widens CAP-014's
  existing `END_TO_END` profile row; it is not a new profile, general-array, ABI,
  performance, safety, or stability conformance claim.
- The accepted CAP-019 widening must retain one shared selected-profile authority for
  fully initialized mutable owned flat exact-`Int` locals. Positive evidence covers
  literal, immutable exact-array identifier, and acyclic ordinary-call initializers of
  the same count; direct `local[index] = exact_int_value` writes in guarded loops;
  source preservation; whole-array by-value return; and ordinary kernel consumption.
  Negative separation rejects mutable-identifier alias initialization, uninitialized
  or partial arrays, mutable parameters/results/aliases, references and escaping
  places, whole-array reassignment, zero/repeat/recursive/nested/non-Int arrays, and
  every unsupported root, profile, and target before trusted backend emission. Backend
  evidence must retain exact `[N x i32]` mutable allocas, reject whole-array checked
  assignment, prove four kernel guards plus an identity-linked same-pointer `i32`
  projected store, and prove the write fixtures store exact `i32 9`. The maintained
  Linux and Windows LLVM/Clang 22 `-O0`/`-O2` lanes must preserve the two read traps
  and add negative and equal-to-count write traps while retaining exact result 2035
  and exit 91. This is the same selected `END_TO_END` profile, not general array,
  mutation, ABI/layout, performance, safety, or stability conformance.
- The accepted CAP-015 representative integration must retain one exact embedded
  `[char; 10]` grammar, equality-only ASCII digit classification, exact
  `Result<int, char>` checked metadata, and material consumption of canonical value 42
  by the existing exit-91 telemetry oracle. Positive evidence retains 0 and 297
  boundaries; negative evidence retains all ten first-malformed-position identities,
  three first-error precedence cases, profile rejection, forbidden numeric-character
  representations, and negative/equal-to-count trap-before-GEP controls. The Linux and
  Windows representative lanes must each retain public check/verified-build/run,
  external LLVM and machine verification, Clang `-O0`/`-O2`, exact stdout/exit 91,
  and the same runtime-failure loop.
  CAP-015 changes no compiler production or language-profile code.
  It enriches only M1-001 `END_TO_END` application evidence;
  it is not a parser, grammar, feature, profile, or promoted component conformance row.
  General-purpose text parsing, runtime Strings, serialization, runtime ingestion,
  file input, and Unicode text encoding/normalization remain unsupported; accepted
  CORE-072's bounded Unicode scalar `char` remains `PARTIAL`.
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

The release suite has reached bounded generic-data-structure, ownership-intensive,
canonical-specialization telemetry, embedded-character representative integration,
immutable exact fixed-array composition, initialized mutable flat-array production,
maintained flat-buffer matvec, source-embedded flat-record two-stage scoring, and
fixed-shape ReLU-and-argmax inference product stages. Accepted CAP-014 created the
CPU-only `exact-i32-array-v0` profile; accepted CAP-018 remains its immutable exact-array
result-composition checkpoint; accepted CAP-019 widens that same profile with fully
initialized mutable owned locals, direct projected element writes, and returned
flat-array values rather than creating another profile. CAP-020, CAP-021, and CAP-023
add product evidence over that unchanged profile rather than new feature semantics or
profiles. CAP-015 remains the accepted M1-001 representative-integration checkpoint.
CAP-015 changes no compiler production or language-profile code. CAP-016 and CAP-017
remain completed readiness/architecture stops, not accepted capabilities; neither adds
a profile or matrix row. CAP-022 remains a mandatory runtime-acquisition
`NO IMPLEMENTATION` stop. CAP-013 remains the single shared specialization identity/
phase authority; CAP-018 and CAP-019 add no specialization classifier.

| Rank | Capability gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Favorable risk | Favorable evidence cost | Total |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | Accepted-head CAP-023 inference correctness/reproducibility/artifact-footprint evidence gate with no performance claim | 4 | 5 | 5 | 5 | 5 | 4 | 28 |
| 2 | Exact CPU + recursive-CopyData application-profile composition readiness and red probe only | 5 | 5 | 5 | 5 | 2 | 2 | 24 |
| 3 | Small quantized numerical-kernel readiness and red probe under one frozen cross-platform arithmetic-and-representation contract only | 5 | 5 | 3 | 5 | 1 | 1 | 20 |

The accepted-head inference correctness/reproducibility/artifact-footprint evidence
gate ranks first and makes no performance claim. Exact CPU plus recursive-CopyData
application-profile composition ranks second at readiness/red-probe scope only. Small
quantized numerical-kernel work ranks third at readiness/red-probe scope under one
frozen cross-platform arithmetic-and-representation contract. Ranks 2 and 3 authorize
no production implementation; real vector, matrix, tensor, ingestion, and accelerator
workloads still require their separate contracts.

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

Accepted CAP-023 adds one zero-production source-embedded fixed-shape
ReLU-and-argmax inference product gate to the maintained conformance evidence. It is
product evidence only: it adds no feature semantics, profile, or conformance row.

Exact CAP-023 reviewed candidate
`63e6b00b6294de61e3afd292a1e32e2b014714e2`, shared candidate/merge tree
`4d234cdfde67f1083773e2c41be4ab92027769db`, accepted base and first merge parent
`4bce540dfed6dfffa152067f4e00424501a6cdd8`, and protected PR #62 merge
`e9b281504446465cfc8fcbe17c65cce92df0e83a` whose second parent is that candidate are
immutable. Candidate push CI `31687464571`, PR CI `31687585904`, Rust CI
`31687585893`, CodeQL `31687584263`, and aggregate candidate check `94407323731`;
candidate push/PR compiler jobs `94406770929`/`94407177877`, stable/nightly/Windows
LLVM 22 jobs `94407178006`/`94407178047`/`94407178042`, CodeQL Actions/Python/Rust
jobs `94407175858`/`94407175752`/`94407175820`, and Actions/Python/Rust analyses
`1612686978`/`1612687391`/`1612693654`; merge-head CI/Rust CI/CodeQL
`31688093145`/`31688093150`/`31688092749`, exact merge compiler/stable/nightly/Windows
LLVM 22 jobs `94408808914`/`94408809340`/`94408809458`/`94408809296`, merge CodeQL
Actions/Python/Rust jobs `94408812427`/`94408812194`/`94408812175`, and default-branch
Actions/Python/Rust analyses `1612715455`/`1612715345`/`1612721829` all pass.

Accepted CAP-023 changes no parser, grammar, source semantics, language profile,
semantic analysis, checked IR, verifier, backend, ABI, or capability classification.
It is a zero-production product/evidence checkpoint over CAP-019's unchanged
`exact-i32-array-v0` surface, composes the accepted CAP-020 flat matvec and CAP-021
record-to-score product, and does not reverse CAP-022's mandatory runtime-acquisition
`NO IMPLEMENTATION` stop.

One source-embedded application convention `[int; 20]` with exact header `[2, 3, 2]`
drives a flat 3-input/2-hidden/2-output wrapping-`i32` computation with strict-positive
zero clamp, two biased logits, signed strict-greater argmax, lower-index tie selection,
three independent malformed-header controls, and reread of all 140 source lanes after
all seven by-value calls.

Exact ordinary, wrapping, activation-boundary, and tie results are respectively
`[1, 122, 167, 135, 181, 4940, 5573, 1]`,
`[1, -24, 18, 2147483623, 0, -37, 2147483641, 1]`,
`[1, -3, 0, 0, 0, 5, 4, 0]`, and `[1, 1, 2, 1, 2, 3, 3, 0]`; malformed results are
eight zeros; public and native success is sentinel 91 with empty source stdout/stderr.

CAP-023 adds no general activation, ReLU, argmax, inference, tensor, matrix, record,
recursive-array, runtime/file input, serialization, quantization, conversion, stable
layout/ABI, performance, resource-usage, accelerator, safety, or language completion
capability.

Its record and topology are application conventions, its retained local artifacts
remain mutable corroboration only, and CAP-019 remains the latest compiler/profile
widening.

The sole matrix change is the existing CPU backend-summary row remaining `PARTIAL`;
no CAP-023 language feature or selected-profile row may be added.

Current accepted public master is CAP-023 merge
`e9b281504446465cfc8fcbe17c65cce92df0e83a`. The PR-only aggregate is correctly absent
on the default branch. Default-branch Actions analysis `1612715455` contains only the
pre-existing open alert #4 created 2026-08-09; Python and Rust analyses contain zero
results; no new CAP-023 code-scanning alert exists.

The selected Milestone 0, Milestone 1, and Milestone 2 exit gates are met for their
bounded selected products; their broader milestone ambitions remain partial.
Milestone 3 remains open. CAP-023 advances its application and reproducibility
boundary but supplies no runtime ingestion, composed CopyData application profile,
quantization, runtime-resource measurement, performance evidence, accelerator
execution, or broader workload.

The retained CAP-021 checkpoint remains historical accepted product evidence.

Accepted CAP-021 adds one zero-production source-embedded flat-record two-stage scoring product gate to the maintained conformance evidence. It is product evidence only:
it adds no feature semantics, profile, or conformance row.

Accepted CAP-021 changes no parser, grammar, source semantics, language profile, semantic analysis, checked IR, verifier, backend, ABI, or capability classification; it is a zero-production product/evidence checkpoint over CAP-019's `exact-i32-array-v0` surface and composes the accepted CAP-020 flat matvec.

The accepted application treats one source-embedded flat `[int; 17]` as an application record with exact header `[2, 3, 1]`, dynamically decodes input, row-major first-stage weights, first-stage bias, second-stage weights, and score bias into fully initialized flat locals, then composes the accepted 2x3 matvec with wrapping bias and affine scoring.

The accepted scorer returns `[valid, raw0, raw1, hidden0, hidden1, score]`; its ordinary result is `[1, 122, 167, 135, 181, 4938]`, its wrapping result is `[1, -24, 18, 2147483623, -2147483631, -2147483627]`, an invalid header returns six zeros, both valid source records preserve and reread all 17 lanes, and the application exits `91`.

Every dynamic read and write uses the existing signed bounds, trap-before-address, `sext`, typed-GEP, and same-pointer consumer authority; exact public and pinned Linux/Windows LLVM 22 verifier, O0/O2, native, and deterministic-emission evidence passes.

Exact CAP-021 reviewed candidate `f91df56084540d30f3c8d09e71c5f30db280fd93`, shared candidate/merge tree `7e34b4b8e817a7aafaaabc6326fa0a4d616fcc91`, accepted base and first merge parent `df0626916d190d8a7580f783e3ac24a89f691617`, and protected PR #60 merge `59af445ea02c1759d337d698be9c4f4472587aaf` whose second parent is that candidate are immutable. Candidate push CI `31670574143`, PR CI `31670599830`, Rust CI `31670599826`, CodeQL `31670598033`, and aggregate candidate check `94354297550`; candidate push/PR compiler jobs `94354135184`/`94354214336`, stable/nightly/Windows LLVM 22 jobs `94354214389`/`94354214394`/`94354214410`, CodeQL Actions/Python/Rust jobs `94354210797`/`94354210770`/`94354210832`, and Actions/Python/Rust analyses `1611711722`/`1611712334`/`1611716646`; merge-head CI/Rust CI/CodeQL `31671091285`/`31671091296`/`31671091099`, exact merge compiler/stable/nightly/Windows LLVM 22 jobs `94355683766`/`94355683532`/`94355683515`/`94355683534`, merge CodeQL Actions/Python/Rust jobs `94355685544`/`94355685480`/`94355685574`, and default-branch Actions/Python/Rust analyses `1611737053`/`1611737605`/`1611740699` all pass.

CAP-021 adds no tensor, matrix, struct, record, recursive-array, nested-array, serialization, runtime/file-input, quantization, activation, checked-overflow, stable layout/ABI, performance, accelerator, safety, general inference, or language-completion capability; the flat record is an application convention, not a source or physical type.

CAP-019 remains the latest compiler/profile capability widening; CAP-020 and CAP-021 are accepted product gates, not separate profiles or feature rows.

The PR-only aggregate CodeQL check is correctly absent on the default branch; the sole open finding remains pre-existing Actions alert #4 from 2026-08-09, and no new CAP-021 alert surfaced.

Accepted CAP-020 adds one zero-production flat-buffer 2x3-by-3 matvec product gate to
the maintained conformance evidence without adding feature semantics.

Accepted CAP-020 changes no parser, grammar, source semantics, language profile,
semantic analysis, checked IR, verifier, backend, ABI, or capability classification;
it is a zero-production product/evidence checkpoint over CAP-019's
`exact-i32-array-v0` surface.

The accepted application encodes a 2x3 matrix as `[int; 6]`, consumes an `[int; 3]`
vector, computes wrapping `row * 3 + column` in nested loops, returns a fully
initialized mutable-produced `[i32; 2]`, preserves every input lane, produces ordinary
and wrapping results `[50, 122]` and `[-2, 5]`, and exits `91`.

The computed linear value flows through the existing signed bounds and
trap-before-address authority before a `[6 x i32]` load, with corresponding guarded
`[3 x i32]` load and `[2 x i32]` store.

CAP-020 adds no matrix type, recursive or nested arrays, static index proof,
checked-overflow arithmetic, stable layout or ABI, performance, accelerator execution,
general mutation, or safety claim.

CAP-019 remains the latest compiler/profile capability widening; CAP-020 is an accepted
product gate, not a separate profile or feature row.

The sole open finding remains pre-existing Actions alert #4 from 2026-08-09; no new
CAP-020 alert surfaced.

Exact CAP-020 reviewed candidate `3b61cd1ed34f910f556821942cd06301ba17dd50`,
shared candidate/merge tree `800510de85bd82f3332126ad249c95da109dd3e1`, accepted
base and first merge parent `13157687f3e955d1c8292ccca133c5a73e29e1a7`, and
protected PR #58 merge `d9493d5123840b38ebab6ca275aaba3216728706` whose second
parent is that candidate are immutable. Candidate push CI `31639493741`, PR CI
`31639540134`, Rust CI `31639540030`, CodeQL `31639535638`, and aggregate candidate check
`94258433541`; candidate stable/nightly/Windows LLVM 22 jobs
`94258276078`/`94258275978`/`94258275899` and CodeQL Actions/Python/Rust jobs
`94258264605`/`94258264489`/`94258264627`; merge-head CI/Rust CI/CodeQL
`31640016314`/`31640016316`/`31640015733`, exact merge
compiler/stable/nightly/Windows LLVM 22 jobs
`94259869631`/`94259869676`/`94259869637`/`94259869559`, merge CodeQL
Actions/Python/Rust jobs `94259873136`/`94259873164`/`94259873086`, and default-branch
Actions/Python/Rust analyses `1610137115`/`1610137589`/`1610144660` all pass.

Accepted CAP-019 widens the existing `exact-i32-array-v0` lane with initialized mutable
flat-array production. Accepted CAP-014 created the CPU-only `exact-i32-array-v0`
profile; accepted CAP-018 remains its immutable exact-array result-composition
checkpoint; accepted CAP-019 widens that same profile with fully initialized mutable
owned locals, direct projected element writes, and returned flat-array values rather
than creating another profile. Accepted CAP-019 widens the existing flat nonempty
exact-`Int` class to a fully initialized mutable owned local whose initializer is an
admitted literal, immutable exact-array identifier, or acyclic ordinary call of the
same count, plus direct `local[index] = exact_int_value` projected writes. The
maintained eight-lane application copies an immutable input, increments every lane in
a guarded loop, returns the whole array by value, feeds it into the accepted CPU
kernel, preserves all eight source lanes, produces result `2035`, and exits `91`;
Linux and Windows retain read traps and add negative/equal-to-count write traps under
verified LLVM/Clang 22 `-O0`/`-O2` routes.

Exact CAP-019 reviewed candidate `f2955bedd22708041e36ee90c65c4f08c443d740`,
shared candidate/merge tree `c520729e7b081087bbe431e97d937fb77f519b37`,
accepted base and first merge parent `84916e124752b8e7d228855a0969cd9eab8dba26`,
and protected PR #56 merge `6ebeb0efb6e83ccc50e12d395e4add1c63ef48b4`
whose second parent is that candidate are immutable. Candidate push/PR/Rust/CodeQL
runs `31627264709`, `31627385522`, `31627385563`, and `31627405516`, plus candidate
aggregate check `94217394313`; merge-head CI/Rust/CodeQL runs `31627880853`,
`31627880924`, and `31627880812`; merge-head compiler, Windows LLVM 22, nightly,
stable, Actions CodeQL, Python CodeQL, and Rust CodeQL jobs `94218938557`,
`94218938794`, `94218938835`, `94218939033`, `94218943455`, `94218943514`, and
`94218943605`; and default-branch Actions/Python/Rust analyses `1609396076`,
`1609396442`, and `1609401493` all pass.

The single selected `exact-i32-array-v0` row remains `END_TO_END`; broad integer and
fixed-array support remains `PARTIAL`; `stable-scalar-v0` remains Aero's only `STABLE`
profile. CAP-019 does not admit general mutable arrays, uninitialized or partial
arrays, mutable parameters/results/aliases, references or escaping places, whole-array
reassignment, zero/recursive/nested/repeat/non-Int arrays, stable aggregate ABI/layout,
general parsing/string/file behavior, GPU execution, performance, or safety. CAP-013
remains the single shared specialization identity/phase authority; CAP-018 and CAP-019
add no specialization classifier.

Accepted CAP-018 widens the existing `exact-i32-array-v0` lane
with immutable exact-array results. Literal, identifier, and call sources compose
through returns, inferred/annotated immutable bindings, ordinary calls, and indexing;
the maintained N=8 application preserves source lane 127, produces lane 128, computes
2035, and retains exit 91 through public and pinned Linux/Windows LLVM/Clang 22
`-O0`/`-O2` routes. Accepted CAP-014 created the CPU-only `exact-i32-array-v0`
profile; accepted CAP-018 widens that same profile with immutable exact-array results
rather than creating another profile.

Exact candidate `409eca9ed2dd8b4ba79f34e14ecfefcc0386e3df`, shared tree
`3073c881c883984f53fcde2f0b205acbec760145`, and protected PR #54 merge
`c49ff17cab7fc0e8d4f552a71499929135c16c61` are exact. Candidate push/PR CI
`31614934307`/`31614994226`, Rust CI `31614994253`, CodeQL `31614991761`, and
merge-head CI/Rust CI/CodeQL `31615467151`/`31615467115`/`31615465499` pass. Exact
default-branch Actions/Python/Rust analyses `1608636029`/`1608636345`/`1608644785`
also pass. CAP-019 subsequently closes the bounded initialized-local, guarded-write,
and returned-value delta while recursive/general arrays, stable ABI/layout,
performance, safety, and stability remain excluded.

Accepted CAP-015 enriches the existing M1-001 representative application with one
exact source-embedded `[char; 10]` telemetry record. Ten identity-linked signed
bounds guards protect ten runtime reads; equality-only ASCII classification returns
the first unexpected `char` through exact `Result<int, char>` metadata; exhaustive
`Match` supplies canonical value 42, which is materially consumed while preserving
exact stdout `telemetry score: 91` and exit 91. Boundary results 0 and 297, all ten
malformed positions, three first-error precedence cases, absence of forbidden numeric-
character conversion/representation, exact raw/semantic/public LLVM parity, both
profile rejections, and negative/equal-to-count trap controls are retained. The same
existing Linux and Windows representative lanes perform public check/required-
verifier build/run, external LLVM and machine verification, Clang `-O0`/`-O2`, exact
output/exit, and clean runtime-failure checks.
CAP-015 remains the accepted M1-001 representative-integration checkpoint. CAP-015
changes no compiler production or language-profile code. CAP-019 remains the latest
accepted compiler/profile capability widening, CAP-018 remains its immutable-result
checkpoint, and both widen CAP-014's first Milestone 3 CPU slice; CAP-020, CAP-021, and
CAP-023 add product evidence only, while CAP-015 only enriches the M1-001
`END_TO_END` row. General-purpose
text parsing, runtime Strings, serialization, runtime ingestion, file input, and
Unicode text encoding/normalization remain unsupported; accepted CORE-072's bounded
Unicode scalar `char` remains `PARTIAL`.

Exact candidate `dd9b1710abebf2f2318582cf94568c2f9a30ca8f`, protected PR #52
merge `b62696272f293f9f378f8a368cc818fcb8ef1074`, and shared tree
`27f359bc5ca90212a06ce73b71759cac0533c1f0` are immutable. Candidate push/PR CI
`31597830488`/`31598146528`, Rust CI `31598146473`, CodeQL `31598144554`, and
merge-head CI/Rust CI/CodeQL `31598634185`/`31598634090`/`31598633803` all pass,
including pinned Linux/Windows LLVM/Clang 22 representative and bounds-trap gates.
This accepted evidence adds no CAP-015 parser/profile row and promotes no neighboring
component classification.

Accepted CAP-014 adds one selected `exact-i32-array-v0` conformance lane. Its
CPU-only profile admits the exact flat fixed-array source class and rejects the full
complement before trusted backend emission; exact wrapping `i32` storage/arithmetic,
two identity-linked signed bounds blocks in the kernel, trap-before-GEP order, and
zero dynamic guards in the constant-index wrapping specimen are mechanically checked.
Focused 11/11, the complete 259-library plus integration/CLI/doc/format/Clippy gate,
the exit-91 kernel, exit-93 wrapping specimen, and both bounds-trap controls pass.
Corrected candidate `226279dd174f26dc3cd1c7573798955bfe789f78`, protected PR
#50 merge `ca09ebe3c1b981339c8bf56b360e62208ac900e1`, and shared tree
`448e1c2ff397012804b886b904aa43bec63f2d37` are exact. Candidate push/PR CI
`31570455915`/`31570461500`, Rust CI `31570461524`, CodeQL `31570456382`, and
merge-head CI/Rust CI/CodeQL `31570823665`/`31570823712`/`31570823073` pass,
including pinned Linux stable/nightly and Windows LLVM 22 public/O0/O2 execution.
This selected lane is `END_TO_END`; broad integer/fixed-array rows remain `PARTIAL`,
and `stable-scalar-v0` remains the only `STABLE` profile.

Accepted CAP-013 adds one canonical specialization identity/order conformance slice
across already admitted generic structs, enums, functions, fixed-capacity containers,
and bounded trait signatures. Mixed `int`/`i32` and `float`/`f64` programs now share
one recursive private identity; semantic and raw checked routes share the same phase
orchestrator; feature policy remains separate. Focused 9/9, existing generic/trait
21/21, authority 7/7, representative 3/3, complete 249-library plus integration/doc/
format/Clippy, corruption, and pinned LLVM/Clang 22 O0/O2 gates pass. Exact candidate
`1ecf083`, protected PR #48 merge `856fc1e5`, shared tree `627582e2`, and all exact
candidate/merge-head CI, Rust/Windows-native, and CodeQL results pass. This remains a
bounded `PARTIAL` specialization slice, not general generics or traits.

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
