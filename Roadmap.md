# Aero Development Roadmap

Last updated: 2026-08-11 (America/New_York)

This roadmap translates Aero's founding Design -> Minimal Prototype -> Self-Host
-> Stabilize -> Optimize path into evidence-gated engineering milestones. A
milestone title or implemented interface does not certify a feature. Current
capability is defined only by `SPEC_IMPLEMENTATION_MATRIX.md`,
`BACKEND_STATUS.md`, tests, and retained artifacts.

The project is currently in **Minimal Prototype / correctness recovery**.
Historical completed-phase and `v1.0.0` labels do not mean that Aero is stable,
self-hosted, or release-ready.

The current accepted public and compiler-capability master is protected CAP-002 merge
`62ccc6ad13c04a0cf17ba7922716ff0d66c3f22a`. Exact candidate `577e601`, all nine
exact-head checks, protected PR #23, post-merge CI/Rust CI/CodeQL, the full root gate,
and pinned LLVM/Clang 22 Linux/Windows representative plus runtime-failure execution at
`-O0`/`-O2` pass. CAP-002 adds guarded runtime fixed-array writes throughout the
existing mutable owned recursive CopyData projection class. Reference-target writes,
collections, projected borrowing, partial moves, lifetime/drop, stable layout/ABI,
memory safety, accelerators, and release claims remain excluded. Accepted CORE-082
remains a bounded Milestone 1 primitive-constant slice; accepted CORE-083 through
CORE-090 are useful but partial Milestone 2 reference, ownership, and aggregate-
composition fragments.

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
Accepted public `CORE-080` additionally preserves the founding direct/aliased dotted
`import` syntax, exact location, and distinct AST identity while routing it through the
same fail-closed phase boundary. Focused 13/13, compatibility, complete all-features,
static, documentation, and root gates pass. No positive name-resolution or import
semantics move; all nine exact-head checks and pinned exits 149/223/227/229 pass.
Accepted public `CORE-081` removes the exact 35-module compiler overlap between
binary and library. Compiler phases and direct-module collection/cache material are
library-owned, while the binary retains CLI-specific modules. Architecture, unit,
integration, all-features, static, documentation, and exact root gates pass; immutable
public evidence passes all nine exact-head checks.
Accepted `CHECKPOINT-001` and corrected solo-maintainer `CHECKPOINT-003` close the
283-commit/226-file handoff before another language slice is stacked. PR #4 merged exact
frozen head `9b13feb2` as merge commit `bf5f8a96`; its tree equals accepted tree
`6d5825a1`, the integration branch remains, strict app-bound protection remains, and
exact-SHA post-merge CI/Rust CI/CodeQL pass. Successor work starts from verified master
in one bounded positive vertical slice per PR. No release, safety, stability,
performance, or production-readiness claim follows.
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
PR #4's integration program is now merged and verified. Bounded successor PRs and
structured evidence-manifest automation remain active scaling controls.

## Corrective checkpoint after CORE-090

The original milestone exits below remain controlling. M1-001 closes the bounded
representative-program and optimization-equivalence portion of Milestone 1, while
foundational Milestone 0 contracts and broader Milestone 1 feature invariants remain
partial. The previously accumulated Milestone 2 fragments remain bounded.

### Milestone gap audit

| Roadmap area | Met | Partial | Open |
|---|---|---|---|
| Milestone 0 | Checked public CLI failures are nonzero and artifact-clean; current lexical/syntactic controls and claim inventory are established. | False-success containment, function/binding/scope contracts, full gates, and independent verification are strong per accepted slice but not frozen as one selected stable-subset contract. | One canonical diagnostic/artifact contract and closure of remaining critical trusted-entrypoint residuals for the selected subset. |
| Milestone 1 | Trusted build/run routes verify LLVM before object generation; accepted M1-001 supplies one maintained representative application, compile-fail corpus, Linux/Windows exact execution, and `-O0`/`-O2` equivalence classified `END_TO_END`. | The bounded conformance subset is authoritative for its workflow, while component language rows and the wider checked-IR/CFG/ownership invariant surface remain `PARTIAL`. | No remaining M1-001 exit item; broader grammar, invariants, and ordinary-program breadth are later capability work and do not become `STABLE` through this gate. |
| Milestone 2 | Structs, fixed arrays, tuples, enums, `Match`, CopyData composition, bounded ownership, references, and projected mutation have executable slices. | Layout and evaluation behavior are private and bounded; ownership is not general. | Collections, generics, traits, error types, general lifetimes/drop/unsafe, public ABI/destruction, a generic data structure, and an ownership-intensive real program. |

### ROADMAP-001 ranked gaps and M1-001 outcome

Scores are 1--5 with higher better; `Risk` and `Evidence` are delivery favorability,
so 5 means lower risk or lower evidence cost.

| Rank | Gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Risk | Evidence | Total |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | Representative scalar application plus frozen subset and optimization-equivalence gate (accepted as M1-001) | 4 | 5 | 5 | 5 | 4 | 3 | 26 |
| 2 | Canonical Milestone 0 diagnostic/artifact and trusted-entrypoint contract | 3 | 5 | 5 | 5 | 3 | 3 | 24 |
| 3 | Positive import/module name resolution after namespace and graph semantics are frozen | 5 | 3 | 5 | 4 | 2 | 2 | 21 |

`M1-001` is accepted. Its fixed-size telemetry-policy program composes direct modules,
functions, constants, control flow, structs, arrays/tuples, enums and `Match`,
mutation, references, and CORE-090 projected writes. Its red exposed a shared Windows
variadic-call false-success, closed in the backend by retaining typed LLVM `double`
arguments and spelling the explicit variadic `printf` call type rather than passing raw
`i64` bits. Public `check`, verified `build`, and `run`; independent LLVM and machine
verification; exact local Windows `-O0`/`-O2` stdout/stderr/exit 91; the three-case
compile-fail corpus; focused 3/3; and the full 218-library/32-binary root gate pass.
Exact candidate `e7a74e6` passed all nine checks, merged through protected PR #19 as
`d7d1c768`, and passed post-merge CI, Rust CI, and CodeQL.

Real-program delta: before `M1-001`, users cannot point to any application-shaped Aero
program covered by an authoritative end-to-end subset contract. Accepted M1-001 now
supplies that program and classifies only its bounded conformance workflow as
`END_TO_END`; individual language features remain `PARTIAL`. Unspecified semantics,
nonportable behavior, optimizer divergence, or evidence that a different task closes
later gaps more safely would still change future decisions. Before another
implementation, at least three remaining gaps must be re-ranked against this accepted
baseline rather than inheriting the old order.

### Post-M1 ranking and accepted CAP-001

The required post-M1 comparison is complete. Scores retain the same 1--5 convention;
`Risk` and `Evidence` reward more favorable delivery.

| Rank | Gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Risk | Evidence | Total |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | Verified runtime reads from fixed arrays (accepted `CAP-001`) | 5 | 4 | 5 | 5 | 3 | 4 | 26 |
| 2 | Canonical Milestone 0 diagnostic/artifact and trusted-entrypoint contract | 3 | 5 | 5 | 5 | 3 | 3 | 24 |
| 3 | Positive import/module name resolution after namespace and graph semantics are frozen | 5 | 3 | 5 | 4 | 2 | 2 | 21 |

Before CAP-001, ordinary variable indexing could compile to unchecked LLVM `inbounds`
address formation and an out-of-range program could falsely succeed. Accepted CAP-001
adds one backend-wide ordered bounds guard for every nonconstant read over the existing
recursive CopyData fixed-array class, enriches the representative telemetry program
with computed reads, and adds negative/equal-to-count runtime controls. Focused,
representative, root, LLVM/machine, exact candidate-head, protected-merge, and exact
merge-head Linux/Windows `-O0`/`-O2` gates pass. The source contract is only a runtime
bounds error; the private trap has no stable status, diagnostic, ABI, or recovery
promise. Dynamic writes, projected borrowing, collections, and general memory safety
remain open. After accepted-truth synchronization, the next task requires a fresh
three-gap ranking against this stronger baseline.

What would change the decision: evidence that runtime bounds errors must be recoverable,
that the private trap can be optimized past an access, that retained array counts are
not independently trustworthy, or that this class requires an unresolved ownership or
stable-ABI decision stops CAP-001. A neighboring receiver/index permutation does not.

### Post-CAP-001 ranking and accepted CAP-002

The CAP-001 accepted-truth synchronization is complete. A fresh comparison uses the
same 1--5 scoring convention; `Risk` and `Evidence` reward more favorable delivery.

| Rank | Gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Risk | Evidence | Total |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | Checked runtime-indexed fixed-array assignment (accepted `CAP-002`) | 5 | 4 | 5 | 5 | 3 | 4 | 26 |
| 2 | Canonical Milestone 0 diagnostic/artifact and trusted-entrypoint contract | 3 | 5 | 5 | 5 | 3 | 3 | 24 |
| 3 | Positive import/module name resolution after namespace and graph semantics are frozen | 5 | 3 | 5 | 4 | 2 | 2 | 21 |

Before CAP-002, accepted CAP-001 makes `values[index]` safe to read, but ordinary
programs still cannot write `values[index] = value` or update nested fixed state in a
loop. Accepted CAP-002 admits runtime `int` selectors throughout the existing mutable
owned projected CopyData assignment class, evaluates selectors once left-to-right
before the RHS, and guards every dynamic selector before later selectors, effects,
address formation, or memory access. The representative telemetry application now
fills its sensor table in a bounded loop. Local focused, representative, and complete
repository gates pass. Exact candidate `577e601`, all nine candidate-head checks,
protected PR #23 merge `62ccc6a`, and exact merge-head CI/Rust CI/CodeQL pass.
Reference-target writes, projected borrowing, collections, compound
assignment, non-CopyData places, stable trap/ABI semantics, releases, benchmarks, and
general memory safety remain excluded.

What would change the decision: evidence that target-before-RHS ordering conflicts with
accepted assignment semantics; that checked selector identity cannot remain exact
through independent verification; that alias/lifetime or stable-layout decisions are
required for the admitted direct-owner class; or that a higher-ranked task can unlock
more real programs with lower correctness risk. Another argument ordering or selector
permutation does not justify changing the architecture.

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
