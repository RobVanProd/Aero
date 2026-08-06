# Aero Development Roadmap

Last updated: 2026-08-06 (America/New_York)

This roadmap translates Aero's founding Design -> Minimal Prototype -> Self-Host
-> Stabilize -> Optimize path into evidence-gated engineering milestones. A
milestone title or implemented interface does not certify a feature. Current
capability is defined only by `SPEC_IMPLEMENTATION_MATRIX.md`,
`BACKEND_STATUS.md`, tests, and retained artifacts.

The project is currently in **Minimal Prototype / correctness recovery**.
Historical completed-phase and `v1.0.0` labels do not mean that Aero is stable,
self-hosted, or release-ready.

Current integration work has accepted `CORE-063` publicly: unary owned enums carry the
accepted recursive CopyData grammar through construction, exhaustive
identifier-bound Match, internal transport, checked IR, independent verification,
private typed LLVM, direct modules, and the pinned LLVM/Clang 22 native-exit-113 gate.
Accepted CORE-061 keeps closures parsed-only/fail-closed and
accepted CORE-062 supplies the recursive CopyData classifier. Projected borrows/writes,
deeper CFG ownership, stable ABI, full module semantics, generic/named-field enums, and
real GPU execution remain unresolved.
Accepted `CORE-064` adds exact direct whole-owner replacement
for that admitted enum class through a shared owned-place classifier, generalized
checked identities, independent verifier controls, and private typed LLVM loads/stores.
All eight public checks and the pinned LLVM/Clang 22 native-exit-131 gate pass; enum
borrowing/projection/storage, new CFG ownership, drop/lifetimes, and stable ABI remain
unsupported.
Accepted `CORE-065` adds exact acyclic conditional joins for those enum owners:
mutually exclusive arms begin from one ownership snapshot, definitely returning arms
do not join, uncertain fallthrough becomes `MaybeMoved`, and checked-IR CFG dataflow
rejects double consumption after partial merges or across cycles. Loop fixed points,
break/continue transport, and general CFG ownership remain unsupported. The
182-library/188-binary/root gates, all eight public checks, and pinned LLVM/Clang 22
native-exit-137 lane pass.
Accepted `CORE-066` adds fresh per-iteration enum owners across
checked `while`, fixed-array `for`, and `loop`. Red-first evidence closed the admitted
`for` continue nontermination defect by routing continue through an explicit shared
increment block. The exhaustive/verifier/compatibility/root gates, all eight public
checks, and pinned LLVM/Clang 22.1.8 external/machine/object/link/native exit 149 pass.
Outer-owner backedge/exit joins, moved-target reinitialization, loop labels/expressions,
and general CFG ownership remain unsupported.
Accepted public `CORE-067` closes the remaining intrinsic-method false-
success tables behind one shared semantic/admission/lowering classifier and admits
exact recursive CopyData fixed-array `.len()`/`.is_empty()` constants. Static String
queries and Array/Vec `.iter()` compatibility remain bounded; runtime Strings,
collections, generic dispatch, iterator ABI, heap behavior, and closures do not move.
Focused, classifier, compatibility, and 183-library/root gates pass. All eight public
checks pass; pinned stable LLVM/Clang 22.1.8 externally and machine-verifies, object-
lowers, links, and executes exit 167, while nightly repeats exit 167.
Accepted public `CORE-068` closes the corresponding ordinary-function-call false-
success boundary behind one exact classifier consumed by both semantic paths and
checked admission/lowering. Missing or unsupported contracts cannot become `Int` or
an LLVM call. The 185-library/root gates and the composed local Clang 19.1.5 native-
exit-181 program pass. All eight public checks pass; pinned stable LLVM/Clang 22.1.8
rejects the invalid control, externally and machine-verifies, object-lowers, links,
and executes exit 181, while nightly repeats verification and exit 181. Generic/trait/
closure calls, overloads/conversions, reference results, new ownership/lifetime
behavior, stable ABI, and runtime collection semantics do not move.
Accepted public `CORE-069` admits exact positional variants with two or more fields
when every field is in the already accepted recursive finite CopyData class. One
schema authority covers construction, ordered bound Match, internal transport,
whole-owner reassignment/control flow, checked IR, verification, private LLVM, and the
pinned stable/nightly LLVM/Clang 22.1.8 native-exit-193 system gate. Named-field/
generic variants, broader patterns, enum storage/borrowing/projection, partial moves,
stable ABI, and accelerators remain excluded.
Accepted public `ARCH-002` then normalizes binding annotations to a leaf plus ordered
wrapper path and routes semantic analysis and checked admission through one supported/
explicitly-rejected/preserved policy. Depth-four characterization, byte-identical LLVM
evidence, all eight public checks, and the unchanged pinned native exit-193 lane
protect the boundary; no language feature, matrix cell, or runtime behavior moves.
Accepted public `CORE-070` adds file-aware library compilation through that same
checked library frontend and the existing root-level direct-module collector. Its
module-free parity, direct-module success/failure matrix, and complete local gates are
green. It is only a partial compiler-service convergence step: imports, namespaces,
recursive modules, external verification, cache behavior, and thin-CLI convergence
remain open. Exact implementation `365c28a3e4fdd306ec4c1a4837545ddbe3dac6a3`
passes all eight public checks and the unchanged pinned native exit-193 lane.
Accepted public `CORE-071` preserves parsed Rust-like direct/aliased/glob `use`
declarations and exact locations but rejects them consistently in semantics and
independent checked admission. Its exact implementation passes all eight public checks
and the unchanged pinned native exit-193 lane; no namespace, visibility, resolver,
backend, or runtime semantics move.
Local candidate `CORE-080` additionally preserves the founding direct/aliased dotted
`import` syntax, exact location, and distinct AST identity while routing it through the
same fail-closed phase boundary. Focused 13/13, compatibility, complete all-features,
static, documentation, and root gates pass. No positive name-resolution or import
semantics move; immutable exact-head evidence remains pending.
Accepted public `CORE-072` then adds exact Unicode `char` as a distinct CopyData leaf
under one shared primitive authority. Raw/escaped literals, equality/inequality,
bindings/replacement, references, calls/results, arrays, tuples, structs, owned enums
and Match, control flow, direct modules, libraries, public CLI paths, checked IR,
independent verification, and private LLVM compose in one two-file native exit-197
system specimen. The 9/9 focused target, 190-library/196-binary complete surface, exact
root gate, and local official LLVM/Clang 22.1.8 system lane pass. Character arithmetic,
ordering, casts, strings/printing, literal-pattern execution, generic behavior, stable
ABI/FFI, and accelerators remain excluded. All eight exact-head public checks and the
stable/nightly pinned LLVM/Clang 22.1.8 exit-197 lanes pass.
Accepted public `CORE-073` adds the next hard ownership slice: exact acyclic whole-owner
reinitialization for already admitted destructor-free enums. One shared transition
classifier permits `Moved`/`MaybeMoved` to become exactly `Owned`; the verifier
independently proves predecessor consumption, schema/value identity, dominance, and
the checked write kill. The exhaustive source-to-native surface and local official
LLVM/Clang 22.1.8 exit-199 gate pass. Every loop-contained reinitialization, partial
move/projection, borrow/storage expansion, drop/lifetime behavior, and general CFG
fixed point remains excluded pending separate semantics and evidence. All eight
exact-head public checks and pinned LLVM/Clang 22.1.8 exit-199 lanes pass.
Accepted public `CORE-074` then adds a hard ADT/control-flow/ownership slice: an
exhaustive Match may yield one fresh owned enum when all arms have the same admitted
schema and their origin is a constructor, exact non-consuming enum-returning call, or
recursively fresh nested Match. Exact checked result/dispatch identities and verifier
CFG proof prevent missing, bypassed, repeated, post-merge, or wrong-schema fabrication.
The composed direct-module/check/build gates, all-eight public set, and pinned native
exit 203 pass.
Accepted public `CORE-075` adds exact initialized direct local/owned-parameter result
origins and a shared dynamic-path ownership join. Same-owner mutually exclusive arms,
different owners, fresh/direct mixtures, and recursively admitted leaves compose;
same-path duplicates, loop effects, additional owned call consumption, and external
nested scrutinees reject. It reuses the existing checked result place, enum-value/place-
load provenance, verifier CFG ownership proof, and private enum layout. Aggregate
results/storage, broader patterns, borrowing/projection, partial moves, drop/lifetimes,
stable ABI, and generic/closure semantics remain separate work. Exact implementation
`50a3e03d0bdbc0e7deddde747bc19df0621c1257`, all eight exact-head checks, and the
pinned stable/nightly LLVM/Clang 22.1.8 native exit 211 lanes pass.
Accepted public `CORE-076` unifies exhaustive Match results over the complete already
admitted value universe: one shared classifier accepts one identical recursive finite
CopyData type or the existing constrained owned-enum class, one generic checked result
place carries every arm through exact typed whole-place assignment, and independent CFG
verification proves all-path initialization and one merged load. Arrays (including
zero-length), tuples, finite acyclic structs, primitives, and owned enums retain their
existing private LLVM types. Exact implementation
`aefeb2d81fb5374e7373a4819f3c92f83a95eb35`, all eight exact-head checks, and both
pinned stable/nightly LLVM/Clang 22.1.8 native exit-223 lanes pass while preserving the
older exit-149 specimen. Strings, references,
unit/unary tuples, dynamic collections, cyclic/unsupported structs, enum aggregate
storage, wider patterns, stable ABI, runtime, drop/lifetimes, and general ownership
remain separately frozen.
Accepted public `CORE-077` admits exact balanced loop-carried reinitialization for a
direct mutable admitted destructor-free enum. `while`, fixed-array `for`, and `loop`
share one rule: entry, condition/iterable, every reachable fallthrough or `continue`
backedge, and every `break` exit must be exactly `Owned`; return paths do not join and
nested transfers attach to the nearest loop. Semantic analysis and independent checked
admission provide snapshots to one phase-neutral classifier, while verifier CFG controls
reject missing, bypassed, one-path, generic-store, wrong-schema, cycle, and exit repairs.
Exact implementation `a93d8d38c5f2a2499ce036f659c13cb2ec4fefcb`, all eight
exact-head checks, and pinned stable/nightly LLVM/Clang 22.1.8 native exit 227 pass
while preserving exits 149/223. Partial moves, projections, enum storage/borrowing,
drop/lifetimes, stable ABI, imports, accelerators, release, safety, and general non-enum
loop dataflow remain separate.
Accepted public `CORE-078` adds no language behavior. Exact implementation
`70f59fd72e96246b2ebefdf1ae53a9b7f3280cfe` pins the official Windows x86_64
LLVM/Clang 22.1.8 full archive by release SHA-256 and proves the existing MSVC
target/layout, invalid-source/IR rejection, external/machine verification, COFF object
generation, Clang/MSVC linking, public `run`, manual execution, and exact exit 227.
All nine exact-head checks pass while Linux stable/nightly preserve exits 149/223/227.
No stable ABI, general Windows, packaging, accelerator, release, safety, or performance
claim follows.
Accepted public `CORE-079` then replaces equality-to-first-entry loop ownership with
one convergent direct-enum header/exit summary shared by semantic analysis and
independent checked admission. `while`, admitted fixed-array `for`, and `loop` recheck
from widened `Owned`/`Moved`/`MaybeMoved` headers; post-loop state conservatively joins
false/exhaustion and nearest-loop break exits, while the existing verifier independently
proves cyclic consumption and repair. Exact implementation
`5b1ec7340db72354542ab325a9f75cad398857c2` passes all nine exact-head checks;
stable/nightly Linux preserve exits 149/223/227 and execute exit 229, while pinned
Windows LLVM/Clang 22.1.8 preserves exit 227 and executes exit 229 through public and
independent native paths.
PR #4 is still a draft integration program; a controlled checkpoint/merge strategy and
structured evidence-manifest automation require separate authorization.

## Milestone 0 - Establish compiler truth (in progress)

- Make invalid lexical and syntactic input fatal on every trusted path.
- Close false-success paths that invent values or silently drop unsupported
  expressions.
- Enforce bounded function, return, binding, and scope contracts before IR.
- Make compiler failures nonzero and prevent invalid-program artifacts.
- Inventory every language, tooling, backend, test, and benchmark claim by stage.

Exit gate: no unclassified critical false-success defect in the chosen stable
subset; one canonical diagnostic/artifact contract; full repository gate and
independent review for each accepted boundary.

## Milestone 1 - Trustworthy scalar CPU core

- Freeze an authoritative grammar and type subset.
- Introduce fallible typed IR with CFG and ownership/type invariants.
- Verify LLVM before object generation.
- Prove source -> semantic analysis -> IR -> object -> link -> execution on Linux
  and Windows for representative scalar programs.
- Add differential optimized/unoptimized runtime tests and real compile-fail
  corpora.

Exit gate: the selected scalar subset is `END_TO_END`, with exact output,
diagnostic, verifier, platform, and reproducibility evidence.

## Milestone 2 - Safe compositional language core

- Implement structs, enums, Match, tuples, and collections as typed aggregate
  vertical slices with defined layout, evaluation order, ABI, and destruction.
- Complete generic substitution, trait dispatch/bounds, and error types.
- Replace shallow ownership tracking with CFG-aware moves, borrows, lifetimes,
  and explicit unsafe boundaries.
- Preserve the 22 active strict Phase 5 syntax-retention tests and replace or recover the 16 quarantined semantic/lossy-shape tests only after their missing contracts are frozen.

Exit gate: at least one real ownership-intensive program and one generic data
structure pass conformance, LLVM verification, and runtime checks without ignored
tests standing in for required behavior.

## Milestone 3 - Aero-native AI/ML infrastructure flagship

- Build a correct CPU reference workload for binary/tensor ingestion and a small
  quantized numerical kernel.
- Grow it into a streaming data or inference component that exercises Aero's
  types, ownership, aggregates, errors, collections, and parallel execution.
- Compare against equivalent established implementations using one correctness
  oracle and the measurement boundaries in `BENCHMARK_PROTOCOL.md`.
- Retain raw inputs, outputs, samples, hashes, toolchains, failures, and artifacts.

Exit gate: a third party can reproduce a useful Aero-native result and understand
its correctness, resource usage, performance, and limitations. External llama.cpp
or framework execution alone does not satisfy this milestone.

## Milestone 4 - Coherent concurrency and tooling

- Freeze an ownership-safe task/channel, structured-concurrency, or equivalent
  model and prove its runtime behavior.
- Converge CLI, library, module resolver, LSP, formatter, documentation generator,
  project tooling, package manager, registry, profiler, and conformance runner on
  one compiler service.
- Add dependency locking, sandbox/trust policy, source-span accuracy, and
  integration tests for failures as well as successful workflows.

Exit gate: tools agree on source meaning and status, concurrent programs have
defined safety/runtime behavior, and a fresh project builds reproducibly.

## Milestone 5 - Proven heterogeneous execution

- Define host/device boundaries, memory spaces, transfers, synchronization,
  streams/queues, kernel ABI, capability discovery, and fallback observability.
- Prove CPU reference equivalence separately for ROCm and CUDA.
- Capture Aero-generated target IR, object/link or loader artifacts, real device
  launch, transfers, synchronization, numerical results, and failure paths.
- Measure compile, transfer, kernel, and end-to-end costs separately.

Exit gate: each named backend independently meets every gate in
`BACKEND_STATUS.md`; a backend flag or transformed helper is insufficient.

## Milestone 6 - Self-host

- Specify the compiler bootstrap boundary and reproducible stage process.
- Implement enough of the compiler in Aero to compile itself in controlled stages.
- Compare stage outputs, diagnostics, runtime behavior, and build reproducibility.
- Keep the Rust bootstrap compiler available until the Aero implementation is
  independently trustworthy.

Exit gate: a documented clean bootstrap produces equivalent verified artifacts
on supported platforms.

## Milestone 7 - Stabilize

- Define the supported surface, compatibility policy, versioning, deprecation,
  migration, release, and governance processes.
- Make all stable examples executable tests and eliminate known invariant
  violations from the release surface.
- Run public RFC review for major semantic commitments.
- Publish an honest pre-1.0 or 1.0 release only when release gates pass.

Exit gate: every stable claim is traceable to specification and end-to-end
evidence, with no historical label substituting for proof.

## Milestone 8 - Optimize and grow the ecosystem

- Add legal, measured optimizations after semantic equivalence is established.
- Expand libraries, platforms, tooling, documentation, and the flagship workload.
- Use larger suites such as SPEC CPU, PARSEC, and domain benchmarks when Aero can
  compile their required workloads correctly.
- Revisit custom backends only where LLVM cannot meet a demonstrated need.

Exit gate: optimization and ecosystem claims remain reproducible, correctness-
gated, and tied to real user workloads.

See `FRAMEWORK_ALIGNMENT.md` for source traceability and the execution-quality
and killer-application rationale behind this sequence.
