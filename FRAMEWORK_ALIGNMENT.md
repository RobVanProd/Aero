# Aero Framework Alignment

Last reviewed: 2026-08-11 (America/New_York)

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

Accepted public `CORE-082` is the first bounded step from the founding
`const identifier : type = expr;` and constant-expression-evaluation direction. It
admits only closed, exactly annotated primitive constants and erases them by checked
substitution before IR. It intentionally defers aggregate/global constants, static
storage, array-size expressions, arbitrary CTFE, import lookup, generics, ABI, and
runtime initialization. Bounded PR #6, all nine exact-head checks, protected merge,
three post-merge master workflows, and the pinned native exit-81 proof pass.

Accepted public `CORE-083` takes a harder ownership-composition step from the founding
borrowing direction. It permits an exclusive, non-escaping mutable whole-place
reference to an already admitted destructor-free enum and exact replacement through
one sole-parameter internal function, while the owner remains valid after the lexical
loan ends. One shared source classifier and an independent checked verifier retain
exact enum identity and reject reads, immutable aliases, escaping references, generic
stores, consumed owners, schema substitutions, aggregate storage, projections, partial
moves, and unsupported payloads. Bounded PR #8 passed every exact-head workflow,
merged as `680bc6ca`, and passed all three exact post-merge workflows; pinned
Linux/Windows LLVM 22 execution observes exact exit 83. This remains bounded evidence,
not general borrowing, lifetime/NLL, drop, stable ABI, or memory-safety evidence.

Accepted public `CORE-084` takes the complementary bounded step from the
founding model's multiple immutable-alias direction. An initialized immutable direct
owner of an already admitted destructor-free enum may have multiple non-escaping `&E`
aliases, including internal immutable-reference parameters, but may be observed only by
an exhaustive `Match *identifier` whose result is existing CopyData or `Void`. One shared
source classifier and an independent checked verifier preserve exact owner, reference,
schema, read, and dispatch identity through private pointer LLVM. The focused target is
4/4 and the complete local repository gate passes at 212 library and 32 binary tests;
bounded PR #10 passed corrected exact-head CI, Rust CI, and CodeQL, merged as
`ae0f0901`, and passed all three exact post-merge workflows. Stable/nightly Linux and
pinned Windows LLVM 22 execution observe exact exit 84. Free enum dereference,
mutable-owner immutable loans, enum transport or storage through a reference, reference
escape/results, lifetime/NLL/drop, stable ABI/FFI, and memory-safety claims remain
excluded. This is bounded framework-aligned evidence, not general reference or
memory-safety acceptance.

Accepted public `CORE-085` composes those two bounded directions without widening
either one: an initialized mutable direct owner of the admitted destructor-free enum
class may have multiple non-escaping immutable aliases, observed only through the
accepted exhaustive immutable-reference Match path. One shared source predicate and an
independent checked verifier preserve exact owner, reference, schema, overlapping-loan,
lexical-end, CFG-join, read, and dispatch identity through private pointer LLVM. Owner
mutation, move, mutable borrow, owned Match, escape, and loop-edge transport while a
loan is live fail before trusted LLVM. Bounded PR #12, its exact-head workflows,
protected merge `d0832c6f`, all three exact post-merge workflows, and pinned native
exit 85 pass. This is bounded evidence, not a general borrowing, lifetime/NLL, drop,
stable ABI, FFI, or memory-safety claim.

Accepted public `CORE-086` adds the complementary executable composition: an active
exclusive mutable enum reference may be observed repeatedly through exhaustive
`Match *identifier`, including before and after accepted whole-value replacement. The
shared reference-Match classifier preserves mutability, checked IR distinguishes the
mutable read identity, and the verifier requires exact active provenance, schema, and
adjacent dispatch. Homogeneous discarded `Void` enum Matches now share one result
contract across owned and both reference origins without fabricated storage; effect-only
`print!`/`println!` expressions no longer become `Int`. The complete root gate passes
214 library and 32 binary tests, and the two-module specimen externally verifies,
machine-verifies, object-lowers, links, and executes exact exit 86 under pinned local
LLVM/Clang 22.1.8. Bounded PR #13, its exact-head checks, protected merge
`e2014a17`, and all three post-merge workflows pass. This adds no free
enum extraction, reference transport, lifetime/NLL/drop, stable ABI, FFI, accelerator,
or general memory-safety claim.

Accepted public `CORE-087` composes the founding temporary mutable-borrow direction
with the already executable recursive CopyData call surface. One non-entry,
non-generic function may take exactly one admitted mutable whole-place reference plus
one or more independent CopyData parameters, with the reference in any declared
position. A single topology predicate and indexed call contract serve semantics,
checked admission, lowering, and independent verification. Direct owners, local alias
reborrows, parameter forwarding, admitted enum pointees, recursive aggregate sides,
and CopyData/`Void` results execute through the tracked direct-module native exit-87
specimen. Bounded PR #14 passed all nine exact-head checks, protected merge
`b07efe29`, and exact post-merge CI `31406731077`, Rust CI `31406731094`, and
CodeQL `31406730798`. This added no simultaneous references, projections, reference
escape/results/storage, new evaluation order beyond the independent side-argument
class, lifetime/NLL/drop, stable ABI/FFI, accelerator, or general memory-safety claim.

Accepted public `CORE-088` takes the next hard compositional step without changing
that reference boundary beyond one source-grounded class. A non-entry, non-generic
signature may contain exactly one admitted mutable reference, one or more admitted
immutable references, and any recursive CopyData companions in every declared order.
The one topology predicate is shared by source and checked signature classification;
the indexed call authority requires the mutable source to be independent from every
other argument, while immutable aliases may share their immutable source. An
independent verifier identity rule rejects raw-owner or active-mutable substitution
for immutable call operands. The focused target is 3/3, the corruption control is
1/1, the complete root gate is green, and the tracked two-module program verifies,
machine-verifies, object-lowers, links, and executes exact exit 88 under pinned
LLVM/Clang 22.1.8. Bounded PR #15, all nine exact-head checks, protected merge
`a7627aa1`, and post-merge CI/Rust CI/CodeQL pass. Multiple mutable parameters,
projections, reference escape/results/storage, NLL/lifetime/drop, stable ABI/FFI,
accelerators, and general memory-safety claims remain excluded.

Accepted public `CORE-089` closes that last signature-count partition without adding a
new reference shape. The shared classifier enumerates every mutable parameter in any
ordered non-entry, non-generic signature, proves pairwise-distinct roots disjoint from
all immutable-reference and CopyData argument trees, and supplies both semantic routes
and checked admission. Lowering emits one declared-order N-borrow/call/reverse-N-end
window; independent verification reconstructs its exact roots, operands, binders, and
adjacency. The focused target is 3/3, the corruption control is 1/1, the affected ring
is 19/19, and pinned local LLVM/Clang 22.1.8 executes exact public and independent exit
89. The exact root gate passes 216 library tests, 32 binary tests, every integration
target, and doc tests. Bounded PR #16 passed all nine exact-head checks, merged through
protected master as `7fbaaaa4`, and passed post-merge CI/Rust CI/CodeQL. Projections,
reference escape/
results/storage, NLL/lifetime/drop, stable ABI/FFI, accelerators, and general
memory-safety claims remain excluded.

Accepted public `CORE-090` takes a hard ownership step by closing one complete
recursive place class instead of one convenient selector shape. An initialized mutable
owned direct local recursive finite CopyData value may be followed by any nonempty
finite mix of declared struct fields, tuple constants, and nonnegative in-range
integer-literal fixed-array indexes, then receive an exact-type CopyData RHS. One
classifier owns root state, path resolution, bounds, schemas, and leaf identity across
both semantic routes and checked admission/lowering. The independent verifier traces
the typed projection chain back to its mutable owner. The focused target passes 1/1,
shared classifier and corruption controls pass 2/2, the affected ring passes 15/15,
the complete root gate is green, and pinned LLVM/Clang 22.1.8 executes the tracked
direct-module specimen at exact exit 90. Bounded PR #17 passed all nine exact-head
checks, merged through protected master as `12820561`, and passed exact post-merge
CI/Rust CI/CodeQL. Dynamic indexes,
projected borrowing, partial moves, enum/non-Copy subplaces, alias analysis,
NLL/lifetime/drop, stable ABI/FFI, accelerators, and general memory-safety claims remain
excluded.

The post-CORE-090 corrective checkpoint re-anchors work to the founding progression:
prove primitives, compose them, build useful programs, and close a milestone before
deepening another neighboring topology. It ranks a representative scalar application
and authoritative subset ahead of further projected-reference permutations. The
accepted `M1-001` supplies a deterministic multi-file telemetry-
policy program that composes the accepted functions, direct modules, constants,
control flow, aggregates, enums/`Match`, mutation, references, and projected writes.
It also replaces an ABI-nonconforming Windows raw-`i64` numeric-print workaround with
typed LLVM `double` arguments and an explicit variadic `printf` call type. Local
public CLI, independent verification, compile-fail, exact Windows LLVM/Clang 22.1.8
`-O0`/`-O2` exit-91, focused 3/3, and complete root evidence pass. Exact candidate
`e7a74e6` passed all nine checks, merged through protected PR #19 as `d7d1c768`, and
passed all post-merge workflows. The composed workflow is `END_TO_END`, while its
component language rows remain `PARTIAL`; it does not establish stable grammar, public ABI,
general ownership/memory safety, performance, or release readiness. Positive import/
name resolution remains strategically valuable, but its namespace, visibility,
collision, cycle, and cache contracts must be frozen before implementation.

Accepted CAP-001 follows that corrected progression by selecting a real-program and
false-success blocker instead of another ownership topology. Before the change,
nonconstant fixed-array reads could form unchecked LLVM `inbounds` addresses and an
out-of-range program could falsely succeed. The accepted capability applies one ordered bounds
guard to the complete existing nonempty recursive CopyData read class, traps before
conversion/access on failure, enriches the representative telemetry application with
computed reads, and retains its exact local Windows `-O0`/`-O2` exit 91. Focused,
representative, full-root, external LLVM, machine-verification, exact candidate-head,
protected-merge, and exact merge-head Linux/Windows evidence is green. This
does not add dynamic writes, collections, projected borrowing, stable runtime/ABI
semantics, or a general memory-safety claim.

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
does not define positive import/name-resolution semantics, namespaces, aliases,
visibility, recursive module graphs, cache identity, external LLVM verification, or a
canonical thin CLI; those remain separately specified work.

Accepted public `CORE-071` contains the prototype's different Rust-like `use` syntax rather
than treating it as the founding import model. The parser preserves direct, aliased,
and terminal-glob declarations plus the exact keyword location, but semantics and
checked admission reject executable use consistently before IR. This adds no positive
module, namespace, visibility, alias, glob, resolver, backend, or runtime behavior.

Accepted public `CORE-080` preserves the founding direct and aliased dotted `import`
grammar as syntax evidence while keeping the same fail-closed phase boundary. The AST
distinguishes founding dotted imports from Rust-like `use`; semantic preflight,
ordinary semantic analysis, and independent checked admission consume one shared
syntax-aware unsupported-import diagnostic before checked IR. Strict malformed forms,
library/file/direct-module routes, and public command artifact hygiene are covered.
This adds no lookup, binding, namespace, visibility, conflict, cycle, package,
backend, runtime, ABI, or positive import behavior. Exact implementation
`063953770ce92f00bae452f312c962c2996977bb` passes all nine exact-head checks and the
unchanged pinned Linux/Windows exits 149/223/227/229.

Accepted public `CORE-081` took the next harder integrated-tooling step. An exact red
architecture contract found 35 compiler modules independently instantiated by both the
binary and library. Compiler phases are now library-owned, while the binary keeps only
CLI-specific services and consumes narrow facades for compilation, direct-module cache
material, registry quarantine, and optimization. Resolver/IR representations remain
private. Unit, integration, architecture, all-features, static, documentation, and exact
root evidence is green; immutable public gates remain pending. This changes Rust ownership/type
identity only and adds no language, module-resolution, ABI, runtime, accelerator,
performance, release, or stability behavior.

Accepted public `CORE-072` advances the founding framework's built-in character and strong-
typing direction as one positive executable class. Exact Unicode scalar identity is
preserved from source through semantic and checked types, independent verification,
private LLVM, and native execution. A single primitive authority prevents semantic/
admission/verifier/backend drift, while the existing recursive finite CopyData grammar
carries characters through arrays, tuples, structs, owned enums and Match, references,
calls, control flow, direct modules, and CLI execution. The pinned local LLVM/Clang
22.1.8 lane executes exact exit 197. This does not establish normalization/grapheme,
string/printing, character arithmetic/order/cast, pattern, generic, stable layout/ABI,
ownership/lifetime, accelerator, performance, release, or stability semantics.

Accepted public `CORE-073` advances the founding ownership direction through one
deliberately acyclic whole-owner transition. An exact write may restore an admitted
destructor-free enum target from `Moved` or `MaybeMoved` to `Owned`; semantic analysis
and checked admission share one classifier, and the verifier independently reconstructs
the consumed predecessor paths and checked write kill. The exact source-to-native lane
passes locally at exit 199. This does not add partial moves, projections, borrow
provenance, enum aggregate storage, drop/destructor/lifetime behavior, loop fixed
points, stable layout/ABI, or a memory-safety claim. All eight exact-head checks and
the pinned LLVM/Clang 22.1.8 exit-199 lanes pass.

Accepted public `CORE-074` advances the founding description of Match as an
expression while preserving strong typing and one-owner semantics. Exhaustive
identifier-bound arms may now yield one same admitted owned enum only from fresh
constructors, exact non-consuming calls, or recursively fresh nested Matches. The
checked result identity and independent verifier retain distinct input/result schemas,
prove one target-dominated write per arm and all-path initialization, and carry the
single merged owner through call/return/re-Match/replacement. This does not add
identifier/conditional owner transport, aggregate Match results, broader patterns,
enum storage/borrowing/projection, partial moves, drop/lifetimes, stable ABI, or
generic/closure semantics. All eight exact-head checks and pinned native exit 203 pass.

Accepted public `CORE-075` advances the unique-owner direction across Match-result
control flow without changing representation. Initialized direct local owners and
owned parameters may be selected on mutually exclusive paths; one shared path
classifier derives exact post-Match ownership and rejects duplicate same-path moves or
loop effects. Existing checked enum-value/place-load provenance, result-place identity,
owned assignment, verifier CFG proof, and private LLVM layout remain authoritative.
Additional owned call consumption, external nested scrutinees, aggregate storage,
borrowing/projection, partial moves, drop/lifetimes, stable ABI, and general CFG
ownership remain excluded. All exact-head checks and pinned stable/nightly native exit
211 pass.

Accepted public `CORE-076` advances the founding description of Match as one strongly
typed expression over the complete already admitted value universe. One shared result
classifier admits identical exact recursive finite CopyData or the separately bounded
owned-enum class; one generic checked result place and independent CFG proof replace
the former scalar/enum topology split. Arrays, recursive tuples, structs, primitives,
and owned enums retain their exact private LLVM type with no default, coercion, public
layout, ABI, runtime, drop, or lifetime rule. Strings, references/results, unit/unary
tuples, dynamic collections, enum-in-aggregate storage, unsupported/cyclic structs,
wider patterns, and general ownership remain excluded. Exact implementation
`aefeb2d81fb5374e7373a4819f3c92f83a95eb35`, all eight exact-head checks, and pinned
stable/nightly LLVM/Clang 22.1.8 native exit 223 pass.

Accepted public `CORE-077` advances the founding unique-ownership direction through a
bounded structured-control class. A direct mutable admitted enum may be temporarily
consumed inside `while`, fixed-array `for`, or `loop` only when one shared edge rule
proves exact `Owned` state at entry, every condition/iterable edge, fallthrough or
`continue` backedge, and `break` exit. Semantic analysis and independent checked
admission provide snapshots to that rule, and verifier CFG dataflow independently
rejects fabricated repairs. The composed direct-module specimen is pinned for native
exit 227. Loop-carried moved state, projections/partial moves, aggregate enum storage,
borrowing, drop/lifetimes, stable ABI, imports, accelerators, and general memory-safety
or general non-enum fixed-point claims remain excluded. Exact implementation
`a93d8d38c5f2a2499ce036f659c13cb2ec4fefcb` passes all eight checks and pinned
stable/nightly LLVM/Clang 22.1.8 native exit 227 while preserving exits 149/223.

Accepted public `CORE-079` advances the framework's unique-ownership direction beyond
exact balanced edges to one convergent direct-enum loop class. A shared finite-lattice
classifier summarizes headers and exits for `while`, admitted fixed-array `for`, and
`loop`; source semantics and independent checked admission recheck from widened
headers, while the verifier retains its independent cyclic proof. This changes no enum
topology, aggregate storage, borrow, destructor/lifetime, ABI, module, runtime, or GPU
contract. Exact implementation `5b1ec7340db72354542ab325a9f75cad398857c2`
passes all nine exact-head checks. Stable/nightly Linux preserve exits 149/223/227 and
execute exit 229; pinned Windows LLVM/Clang 22.1.8 preserves exit 227 and executes exit
229 through both public and independent native paths.

Accepted public `CORE-078` advances portability/reproducibility without changing the
language. Exact implementation `70f59fd72e96246b2ebefdf1ae53a9b7f3280cfe`
pins the official Windows x86_64 LLVM/Clang 22.1.8 full archive by release SHA-256.
The public MSVC lane preserves the existing target/layout, rejects invalid source and
IR, externally and machine verifies, emits COFF, links, and returns exit 227 through
public and manual execution. All nine exact-head checks pass; stable/nightly Linux
preserve exits 149/223/227. This remains bounded evidence, not a general Windows or ABI
claim.

| Framework direction | Current evidence | Required next proof |
|---|---|---|
| Clear, strongly typed source language | Numeric, function, binding, and selected control-flow slices are partial; several composite forms are parser-only or fail closed. Closures are explicitly parsed-only and cannot acquire a fabricated scalar type or reach trusted IR. Accepted CORE-071 preserves Rust-like `use` syntax only for future work and rejects it before checked IR; accepted CORE-080 similarly preserves the founding dotted `import` syntax with a distinct AST identity and rejects it through the same shared authority. Accepted CORE-072 preserves exact Unicode-character identity across the existing recursive CopyData execution surface under one primitive authority and a pinned native exit-197 gate. Accepted CORE-067 closes fabricated intrinsic-method results; accepted CORE-068 similarly requires one exact named-call contract before semantic success or checked IR. Accepted ARCH-002 normalizes binding-annotation topology and phase routing without changing any accepted or quarantined type behavior. | A specified stable subset with exact positive, negative, diagnostic, and execution tests; separately freeze import/name-resolution semantics before any positive path |
| Ownership-based safety | Shallow move checks remain partial. CORE-048/053 through accepted CORE-066 establish bounded immutable/mutable whole-place ownership, internal reference transport, recursive finite CopyData composition, direct CopyData owner reassignment, owned enum transport/replacement, exact acyclic conditional joins, independent enum-owner CFG consumption proof, and fresh per-iteration enum owners. Accepted CORE-073–077 add acyclic reinitialization, owned Match results, and balanced loop restoration. Accepted CORE-079 adds convergent `Owned`/`Moved`/`MaybeMoved` direct-enum loop headers/exits under shared source/admission classification and independent verifier proof. Accepted CORE-083–089 compose enum reference behavior and mixed/multiple reference callable signatures. Accepted CORE-090 admits exact static projected writes through arbitrary finite field/tuple/fixed-array paths over mutable owned CopyData roots under one shared classifier. Accepted CAP-001 guards runtime fixed-array reads before conversion or address formation. Projected borrowing, dynamic writes, partial moves, reference results, escaping provenance, free enum dereference/transport, enum aggregate storage, stable reference ABI, general non-enum CFG ownership, NLL, drop, lifetime inference, and memory-safety claims remain absent; 16 broader semantic/lossy-shape Phase 5 tests remain quarantined. | Re-rank the next real-program blockers after accepted-truth synchronization rather than resume topology enumeration |
| Structs, arrays, enums, traits, and Match | CORE-043 through CORE-047 accept bounded all-Copy scalar/named-struct construction, projection, arrays, transport, and finite acyclic graphs. CORE-049 through CORE-052 accept unit/unary-scalar enums, exhaustive bound Match, and owned internal transport. CORE-058 through CORE-061 add flat tuples, whole-place references, and direct CopyData owner replacement. Accepted CORE-062 removes the executable CopyData topology whitelist. Accepted CORE-063 carries that recursive class through unary owned-enum payloads and exact bound Match under a pinned native exit-113 gate. Accepted CORE-064/065 add exact enum replacement and acyclic joins. Accepted CORE-069 generalizes positional variants to two or more recursive CopyData fields. Accepted CORE-073–077 add reinitialization, typed owned Match results, and balanced loop-owner restoration; accepted CORE-079 changes loop dataflow only, not enum topology. Accepted CORE-083–089 add bounded reference/enum compositions. Accepted CORE-090 composes existing recursive CopyData field, tuple, and fixed-array projections into exact static mutable paths without adding a data topology. Accepted CAP-001 adds guarded variable reads over the existing nonempty recursive CopyData fixed-array class. Generic/named-field enums, Option/Result Match, wildcard/guard/nested destructuring, enum fields/arrays, free enum dereference or transport through references, projected borrowing/partial moves, dynamic arrays/writes, unit/unary tuples, unsupported/cyclic structs, traits, and stable aggregate/reference/runtime ABI remain open. | Prioritize the freshly ranked blocker with the greatest real-program and roadmap payoff |
| Typed SSA-style IR and LLVM backend | LLVM text and a partial CPU object/link/run path exist. Pinned Linux LLVM 22 execution and bounded Windows x86_64 MSVC system evidence are accepted through CORE-078. Typed-IR invariants and verification remain incomplete. | Retain exact object/link/runtime gates on each supported platform and extend only with separately frozen contracts |
| Zero-cost performance | A benchmark protocol now exists, but no audited public Aero runtime or device performance claim passes it | Correct real programs, raw samples, reproducible baselines, and separately reported compile/runtime/resource costs |
| Modern concurrency | Interfaces and library-like helpers exist, but the language/runtime concurrency model is not end-to-end | Ownership-safe tasks/channels or another frozen model with race and runtime evidence |
| Integrated tooling | CLI, LSP, formatter, docs, project, registry, and conformance surfaces remain experimental. Accepted CORE-070 adds a checked file-aware library route; CORE-071 and CORE-080 preserve both import syntax families and fail closed without a resolver. Accepted public CORE-081 removes the 35-module binary/library compiler overlap and makes compiler phases library-owned; broader tool-path convergence remains pending. | One canonical compiler service shared by every tool, with failure and integration tests |
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
