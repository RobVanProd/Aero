# Aero Development Roadmap

Last updated: 2026-08-12 (America/New_York)

This roadmap translates Aero's founding Design -> Minimal Prototype -> Self-Host
-> Stabilize -> Optimize path into evidence-gated engineering milestones. A
milestone title or implemented interface does not certify a feature. Current
capability is defined only by `SPEC_IMPLEMENTATION_MATRIX.md`,
`BACKEND_STATUS.md`, tests, and retained artifacts.

The project is currently in **Minimal Prototype / correctness recovery**.
Historical completed-phase and `v1.0.0` labels do not mean that Aero is stable,
self-hosted, or release-ready.

CAP-018 is accepted as immutable exact-array value/result composition inside
CAP-014's existing CPU-only `exact-i32-array-v0` profile. Accepted CAP-014 created
the CPU-only `exact-i32-array-v0` profile; accepted CAP-018 widens that same profile
with immutable exact-array results rather than creating another profile. Ordinary nongeneric
functions may construct, return, bind, forward, pass, and index immutable exact flat
`[int; N]`/`[i32; N]` values for `1 <= N <= i32::MAX`. One shared recursive profile
classifier covers literal, exact-array identifier, and ordinary named acyclic-call
roots across result, inferred/annotated binding, call-argument, and index-object
placements. It adds no profile or specialization architecture and changes no semantic,
checked-IR, verifier, or LLVM production contract.

The maintained eight-lane application now transforms source lane 127 to 128 in a
returned array, forwards it through an ordinary helper, consumes it in the CPU kernel,
observes result 2035, preserves the original Copy source, and retains exact exit 91.
Exact candidate `409eca9ed2dd8b4ba79f34e14ecfefcc0386e3df`, tree
`3073c881c883984f53fcde2f0b205acbec760145`, and protected PR #54 merge
`c49ff17cab7fc0e8d4f552a71499929135c16c61` are immutable. Candidate push/PR/Rust/
CodeQL runs `31614934307`, `31614994226`, `31614994253`, and `31614991761` pass.
Merge-head CI/Rust/CodeQL runs `31615467151`, `31615467115`, and `31615465499`
pass, and default-branch Actions/Python/Rust analyses `1608636029`, `1608636345`, and
`1608644785` pass. CAP-014 remains the profile origin and first bounded Milestone 3
CPU slice; CAP-018 is the latest accepted compiler/profile capability. CAP-015 remains
the accepted M1-001 representative-integration checkpoint. CAP-015 changes no compiler
production or language-profile code. CAP-016 and CAP-017 remain completed
readiness/architecture stops, not accepted capabilities; neither adds a profile or matrix row.
CAP-013 remains the shared
specialization identity/phase authority.

CAP-014 is accepted as the first bounded Milestone 3 CPU computation slice. Its
distinct CPU-only `exact-i32-array-v0` profile composes exact wrapping LLVM `i32`
arithmetic with flat fixed integer arrays, by-value nongeneric calls, guarded runtime
indexing, and one tracked eight-lane dot-product-plus-bias kernel at independent-
oracle exit 91. A second specimen proves wrapping edge behavior at exit 93. The
profile's one shared source classifier and one verifier-fed physical mapping leave
semantic analysis, checked IR, and independent verification unchanged. Exact
candidate `226279dd174f26dc3cd1c7573798955bfe789f78`, protected PR #50 merge
`ca09ebe3c1b981339c8bf56b360e62208ac900e1`, shared tree
`448e1c2ff397012804b886b904aa43bec63f2d37`, candidate push/PR CI
`31570455915`/`31570461500`, Rust CI `31570461524`, CodeQL `31570456382`, and
merge-head CI/Rust CI/CodeQL `31570823665`/`31570823712`/`31570823073` all pass,
including pinned Linux and Windows LLVM 22 gates. This is one private bounded CPU
reference computation. Accepted CAP-018 subsequently admits immutable exact array
results rooted in literals, identifiers, or ordinary acyclic calls across result,
binding, call-argument, and index-object placements within this same profile.
Empty/repeat/nested arrays, recursion, projected or mutable array writes, non-integer elements, surrounding aggregate or
reference use, modules/imports, constants, methods, generics/traits, closures, collections,
allocation/drop, I/O, accelerators, and non-CPU target pairing remain rejected. It
does not stabilize ABI/layout, serialization, packages, SIMD, tensor/quantized
infrastructure, performance, safety, Aero as a whole, or Milestone 3 completion.

CAP-015 is accepted as a bounded representative application integration checkpoint.
It enriches the existing M1-001 telemetry application with one exact embedded
`[char; 10]` record, `T=<digit><digit>;H=<digit><digit>;`, whose guarded reads,
equality-only decimal classification, explicit `Result<int, char>`, and exhaustive
`Match` produce canonical value 42 and feed that value materially into the established
exact-output/exit-91 computation. The gate also retains boundary results 0 and 297,
all ten first-malformed-position identities, three first-error precedence controls,
and negative/equal-to-count trap-before-access specimens across public and pinned
Linux/Windows LLVM/Clang 22 `-O0`/`-O2` routes.
CAP-015 changes no compiler production or language-profile code.
CAP-014 remains Aero's exact-array profile origin and first Milestone 3 CPU slice;
CAP-018 is its latest accepted compiler/profile capability. CAP-015 remains the latest
separately classified project-integration checkpoint and only enriches the existing M1-001
`END_TO_END` application evidence. Both named profiles continue to reject the parser.
General-purpose text parsing, runtime Strings, serialization, runtime ingestion, file
input, and Unicode text encoding/normalization remain unsupported; accepted CORE-072's
bounded Unicode scalar `char` remains `PARTIAL`.

Exact candidate `dd9b1710abebf2f2318582cf94568c2f9a30ca8f`, protected PR #52
merge `b62696272f293f9f378f8a368cc818fcb8ef1074`, and shared tree
`27f359bc5ca90212a06ce73b71759cac0533c1f0` are immutable. Candidate push/PR CI
`31597830488`/`31598146528`, Rust CI `31598146473`, CodeQL `31598144554`, and
merge-head CI/Rust CI/CodeQL `31598634185`/`31598634090`/`31598633803` all pass.
This evidence does not add a CAP-015 parser, grammar, feature, profile, stability, or
conformance row and does not promote any neighboring component capability.

CAP-013 is an accepted cross-capability architecture and executable-composition
checkpoint. It promotes the prior specialization watch only because concrete drift
was observed: primitive aliases could be rejected across generic/trait boundaries or
produce duplicate private identities. One recursive canonical key, framing contract,
and deterministic struct -> enum -> function phase plan now serve semantic and raw
checked admission, while feature-specific validation and the single generic-function
body classifier remain separate. The representative telemetry program mixes
`Window<i32>`/`Window<int>` and trait `i32`/`int` signatures at exact output/exit 91.
Focused, corruption, complete-root, pinned LLVM 22 O0/O2, exact candidate, protected
PR #48 merge `856fc1e5f310b2b458f97d7b6aebb1ecf5c28572`, and exact merge-head gates
pass. This does not complete generics or traits.

The post-CAP-018 ranking now controls task selection. First is mutable loop-produced
exact-`i32` flat-array results. Second is one bounded recursive exact-array/2D matrix
pipeline under a shared recursive shape authority. Third is runtime byte/file
acquisition into a bounded owned buffer. The exact scored ranking and full before/
after/stop/change-mind contracts appear below. CAP-016 and CAP-017 remain completed
stops: no import semantics or propagation syntax may be invented merely to revive
their earlier rank.
Mutable loop-produced exact-`i32` flat-array results rank first.

CAP-012 is an accepted Milestone 2 capability. It closes the remaining selected exit
half by letting the representative telemetry program call ordinary
helpers over nested CopyData state through immediate, conservatively root-scoped
immutable and mutable loans. The implementation shares projected-place classification with
assignment and adds independently verified checked-IR loan identity/lifecycle plus
Linux/Windows runtime-bounds fixtures. Local complete gates, exact candidate
`79d14866061184bc619ce5c92603c0964a31e74d`, all public candidate results,
protected PR #46 merge `49bcdfc3b23d2e1cc22fa3f0f36446fcffbf6e92`,
and exact merge-head CI/Rust CI/CodeQL pass, including pinned LLVM/Clang 22 native
execution. Together with CAP-011, the selected Milestone 2 exit gate is met. This
does not complete all Milestone 2 ambitions: stored/escaping references, general alias
reasoning/lifetimes, generic reference-call expansion, and general ownership remain
out of scope.

CAP-011 is an accepted Milestone 2 capability selected by the post-CAP-010 gap
ranking. It turns earlier generic-definition, whole-value specialization, fixed-array,
bounds-check, and projected-mutation primitives into one reusable fixed-capacity
`Window<T>` data structure with generic checked reads and functional updates. The
representative application now uses it for both integers and characters. Local
focused, representative, identity, complete-root, check, docs, formatting, and diff
gates pass. Exact candidate `dea5714e`, all nine public results, protected PR #44
merge `34b81eee`, and exact merge-head CI/Rust CI/CodeQL pass, including pinned
Windows LLVM 22 native execution. This does not complete collections or general
generics.

Accepted CAP-013 resolves the prior specialization architectural watch at the proven
shared seam: canonical recursive identity, equivalence, private framing, and phase
order. It deliberately does not merge policy-specific template validation,
substitution restrictions, rewriting, diagnostics, or body classification. Reopen
architecture work only on new cross-feature drift or a real multi-capability blocker,
not file size or neighboring permutations.

CAP-010 is an accepted Milestone 2 capability and the highest-payoff result of the
post-CAP-009 milestone-gap audit: required-only
nongeneric traits can supply immutable-receiver behavior to whole-value recursive
CopyData generic functions through deterministic static specialization. The
representative telemetry program uses one trait across both `Sensor` and `Batch`, so
the change advances real generic algorithm composition instead of another neighboring
topology. Focused, corruption, representative, compatibility, full-root, formatting,
Clippy, exact-candidate, pinned Linux/Windows native, protected PR #42, and exact
merge-head gates pass at accepted master `f77f1a227032008ab3ceadf2e2e3dcaed3b225e9`.

This slice did not complete traits or generics. Collections, general generic
operations, associated/default items, dynamic dispatch, broader patterns and error
propagation, lifetimes/drop/unsafe, and public ABI/destruction remain open. CAP-011
and CAP-012 later supplied the selected generic-container and ownership-intensive-
program exit product.

CAP-009 is an accepted Milestone 0 capability selected by the required
post-CAP-008 audit and three-gap ranking. Before it, programs could execute inside the
experimental compiler but could not request enforcement of a frozen cross-platform
source and representation contract. After it, public library and CLI routes can select
`stable-scalar-v0`, fail closed on every AST shape outside its one-file acyclic
nongeneric `int`/`bool` class, and execute representative exit-91 and wrapping exit-93
programs through an exact LLVM `i32` lane at `-O0` and `-O2`. One post-parse classifier
owns source admission; a private checked-program profile identity selects physical
representation without duplicating feature guards; the CLI profile is CPU-only.
Focused 10/10, the complete repository gate, all nine exact-candidate checks, protected
PR #40 integration, and exact merge-head CI/Rust CI/CodeQL pass. Accepted public master
is `1ef21c564ec564379e611002b1b321d910a991a3`, and the Milestone 0 selected-
stable-subset exit is met. CAP-009 does not stabilize Aero as a whole or add any of its
explicitly excluded aggregate, module, generic, ownership, runtime, ABI, accelerator,
benchmark, or release behavior.

CAP-008 is an accepted Milestone 2 capability selected by the required
post-CAP-007 roadmap audit and three-gap ranking. Its real-program delta is concise,
exhaustive fallback handling: ordinary Aero code may write `Err(_)` for an ignored
payload and one final `_ => fallback` for otherwise-uncovered variants across the
already-admitted concrete enum classes. The one shared enum-arm resolver remains the
semantic and checked-admission authority, and the independent verifier's one-target-
per-variant contract remains intact. Focused 4/4, complete root/verifier, representative,
and pinned Windows LLVM/Clang 22.1.8 external/machine/native/public-run gates pass at
unchanged exact output/exit 91. Exact candidate `9ebd204`, protected PR #38 merge
`a1716f8`, shared tree `c3dab0e`, all candidate-head checks, and exact merge-head CI
`31525340621`, Rust CI `31525340810`, and master-push/CodeQL `31525340605` pass.
It does not supply guards, nested destructuring, general error propagation, collections,
imports, ownership/drop expansion, stable ABI, or safety.

The prior accepted compiler-capability checkpoint was protected CAP-007 merge
`5a64acaffa5e7f7167823861a45bc49c6bb670b4`. Exact candidate `bfb7adb`,
candidate-head checks, protected PR #35, merge-head CI/Rust CI/CodeQL, the full root
gate, and pinned LLVM/Clang 22 stable Linux/Windows representative execution at
`-O0`/`-O2` pass. Accepted CAP-007 closes the canonical checked-program entrypoint
mechanism without adding source semantics. Accepted CAP-006 adds explicit user-defined bound-free generic
enums at exact recursive finite CopyData applications through contextual construction,
owned transport/replacement, and exhaustive `Match`. Accepted CAP-005 adds bounded
compile-time specialization of
bound-free whole-value generic transport functions over recursive finite CopyData.
Accepted CAP-004 adds bounded explicit user-defined generic
recursive-CopyData structs with deterministic compile-time substitution, checked
identity/schema verification, and composition through existing functions, aggregates,
mutation, and references. Accepted CAP-003 separately supplies bounded explicitly typed
concrete recursive-CopyData `Option<T>` and `Result<T, E>` construction, owned
transport, replacement, and exhaustive bound `Match`. Context-free inference, general
generic substitution/error propagation, trait-bounded or operational generic
functions, bounded/named-field/general generic enums and traits, carrier
aggregate/reference storage, collections, lifetimes/drop, stable layout/ABI, memory
safety, accelerators, and release claims remain excluded. Accepted CORE-082
remains a bounded Milestone 1 primitive-constant slice; accepted CORE-083 through
CORE-090 are useful but partial Milestone 2 reference, ownership, and aggregate-
composition fragments.

Accepted CAP-007 supplies the ranked Milestone 0 canonical
checked-program entrypoint mechanism. Public artifact-free `check_program` and
`check_file` APIs, library compilation, and CLI check/build/run/profile/source-test
validation share one library-owned lex/parse/direct-module/semantic/checked-IR/
internal-verification authority. Focused 3/3 and the complete normalized
235-library/32-binary/84-integration/doc/format/Clippy gate pass. Cached official
Windows LLVM/Clang 22.1.8 also externally verifies,
machine-verifies, and executes the representative product at `-O0`/`-O2` with exact
output/exit 91. Exact candidate `bfb7adb`, protected PR #35 merge `5a64aca`, all
candidate-head checks, and exact merge-head CI/Rust CI/CodeQL pass. CAP-007 does not
make any language feature `STABLE`; stable-subset classification remains open.

The required post-CAP-004 milestone-gap audit and three-gap ranking are complete in
the CAP-005 authorization. Bound-free recursive-CopyData generic transport functions
and the Milestone 0 canonical diagnostic/artifact contract tied at 24; an owned
dynamic collection foundation scored 22. CAP-005 won the tie because one reusable
type-independent helper creates a larger immediate real-program delta without
inventing trait or collection semantics. Its accepted implementation uses one
shared compile-time specialization authority that handles the bounded whole-value
transport class, the representative program uses `choose<T>` at `Reading<int>` and
`Reading<char>`, the complete 232-library/32-binary repository gate passes, and pinned
LLVM/Clang 22.1.8 verifies and executes exact `-O0`/`-O2` output/exit 91. Exact
candidate `68e2cd8`, protected PR #31 merge `59f7e47b`, and exact merge-head CI
`31504122753`, Rust CI `31504122730`, and CodeQL `31504122424` pass.

CAP-004 was selected by the required fresh post-CAP-003 milestone-gap and three-gap
ranking. Its accepted real-program delta is reusable data definition: one
`struct Reading<T>` now supplies independently typed `Reading<int>` and
`Reading<char>` values in the representative application instead of duplicated
concrete records. One shared pre-semantic/pre-admission authority performs exact
substitution; independent verification binds each private identity to its logical
schema. Local focused, corruption, compatibility, representative, complete root,
formatting, and correctness-denying Clippy evidence passes, as do the exact public
candidate, protected merge, merge-head, and pinned LLVM 22 gates. This closes one
bounded generic-data-definition requirement, not general generics or collections.

CAP-003 was selected by the required fresh milestone-gap and three-gap ranking. Its
accepted real-program delta is explicit absence/recoverable-failure
transport: bounded Aero functions can return, move, replace, pass, and exhaustively
inspect concrete recursive-CopyData `Option<T>` and `Result<T, E>` values rather than
sentinel scalars. This is a closed monomorphic built-in-family slice, not general
generic substitution or a complete error model. Local focused, shared-normalizer,
adjacent ownership, representative-program, complete root, formatting, and
correctness-denying Clippy evidence passes, as do the exact public candidate,
protected-merge, merge-head, and pinned LLVM 22 gates.

Current integration work has accepted `CORE-063` publicly: unary owned enums carry the
accepted recursive CopyData grammar through construction, exhaustive
identifier-bound Match, internal transport, checked IR, independent verification,
private typed LLVM, direct modules, and the pinned LLVM/Clang 22 native-exit-113 gate.
Accepted CORE-061 keeps closures parsed-only/fail-closed and
accepted CORE-062 supplies the recursive CopyData classifier. Projected borrows/writes,
deeper CFG ownership, stable ABI, full module semantics, named-field, bounded, or
general generic enums, and
real GPU execution remain unresolved.

Accepted CAP-006 is the selected post-CAP-005 capability. It adds exact explicit
user-defined recursive-CopyData generic enum
specialization and composes `Sample<Reading<int>>` with `Sample<char>` in the growing
representative application at unchanged output/exit 91. The complete local repository
gate, pinned LLVM/Clang 22 Linux/Windows native evidence, protected PR #33, and exact
merge-head workflows are green. Bounds/traits,
general substitution/operations, named variants, collections, lifetimes/drop, and
public ABI remain open.

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
| Milestone 0 | Checked public CLI failures are nonzero and artifact-clean; accepted CAP-007 supplies one canonical checked-program entrypoint and artifact contract across frozen trusted validation routes; accepted CAP-009 classifies and executes the selected `stable-scalar-v0` subset as `STABLE`. | The wider compiler, component feature rows, experimental default, ABI, ownership, modules, aggregates, and release surface remain unstabilized. | Milestone 0 exit is met; preserve the selected profile while ranking the next broad real-program capability. |
| Milestone 1 | Trusted build/run routes verify LLVM before object generation; accepted M1-001 supplies one maintained representative application, compile-fail corpus, Linux/Windows exact execution, and `-O0`/`-O2` equivalence classified `END_TO_END`. Accepted CAP-015 enriches that same application with exact embedded-character interpretation through guarded reads and typed `Result`, materially composed into the unchanged exit-91 oracle. | The bounded conformance subset is authoritative for its workflow, while component language rows and the wider checked-IR/CFG/ownership invariant surface remain `PARTIAL`; CAP-015 adds no parser or profile row. | No remaining M1-001 exit item; broader grammar, invariants, and ordinary-program breadth are later capability work and do not become `STABLE` through this gate. |
| Milestone 2 | Structs, fixed arrays, tuples, enums, `Match`, CopyData composition, bounded ownership, references, projected mutation, accepted CAP-003 concrete CopyData `Option`/`Result` carriers, accepted CAP-004 explicit generic CopyData structs, accepted CAP-005 bound-free whole-value generic transport functions, accepted CAP-006 explicit user-defined recursive-CopyData generic enums, accepted CAP-008 nonbinding enum wildcards, accepted CAP-010 required-only CopyData trait dispatch, accepted CAP-011 fixed-capacity generic container algorithms, accepted CAP-012 projected call loans, and accepted CAP-013 canonical bounded-specialization identity/phase authority have executable slices. CAP-011 and CAP-012 satisfy the selected generic-data-structure plus ownership-intensive-program exit product. | Layout and evaluation behavior are private and bounded; ownership is not general. CAP-003 is monomorphic built-in-family evidence; CAP-004 through CAP-006 supply exact generic data-definition, function-transport, and enum-specialization slices; CAP-008 adds only terminal whole-arm and payload-leaf wildcards; CAP-010 adds one verifier-bound static-dispatch class; CAP-011 and CAP-012 close only the selected exit gate; CAP-013 closes alias/identity/phase drift without adding general semantics. | The selected Milestone 2 exit gate is met. General collections and generic operations, broader traits, broader patterns/error propagation, general lifetimes/drop/unsafe, public ABI/destruction, and ordinary-program breadth remain higher-milestone capability work. |
| Milestone 3 | Accepted CAP-014 supplies the first bounded exact-`i32` fixed-array CPU reference kernel with guarded indexing and Linux/Windows native oracle evidence; accepted CAP-018 widens the same profile with immutable exact-array construction, result, binding, forwarding, argument, and direct-index composition. | The CPU lane remains one private named profile over flat integer arrays. Mutable loop-produced results, recursive arrays/2D matrices, and the larger workload remain open. CAP-015's embedded-literal evidence belongs only to M1-001 representative integration; runtime ingestion, file input, general parsing, broader error propagation, collections, streaming, quantization/tensors, resource measurement, and accelerator execution remain open. | The milestone exit is not met. Follow the post-CAP-018 ranking, beginning with mutable flat-array production; keep recursive shapes and runtime file acquisition within their stated stop contracts. |

Accepted CAP-009 closes the selected-stable-subset portion of the Milestone 0 row with
protected exact-head and merge-head evidence. It does not promote neighboring
component rows or the wider language beyond their recorded classifications.

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

### Post-CAP-002 ranking and accepted CAP-003

The CAP-002 accepted-truth synchronization and corrective milestone audit selected a
broader ordinary-program capability rather than another reference or index topology.
The comparison used the same 1--5 scoring convention; `Risk` and `Evidence` reward
more favorable delivery.

| Rank | Gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Risk | Evidence | Total |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | Explicitly typed `Option`/`Result` construction, transport, and exhaustive `Match` (accepted `CAP-003`) | 5 | 5 | 5 | 5 | 2 | 3 | 25 |
| 2 | Canonical Milestone 0 diagnostic/artifact and trusted-entrypoint contract | 3 | 5 | 5 | 5 | 3 | 3 | 24 |
| 3 | Positive import/module name resolution after namespace and graph semantics are frozen | 5 | 3 | 5 | 4 | 2 | 2 | 21 |

Before CAP-003, Aero retained carrier syntax but functions could not truthfully return
absence or recoverable failure: semantics fabricated missing types and checked
admission rejected construction. Accepted CAP-003 removes those defaults and admits
all four constructors only under an exact concrete recursive-CopyData binding,
reassignment, nongeneric parameter, or nongeneric result context. The values reuse the
owned-enum pipeline for moves, replacement, internal transport, and exhaustive bound
`Match`. The representative telemetry application executes both `Ok` and `Err` paths
while retaining exact output and exit 91. Focused 4/4, normalizer/corruption 4/4,
representative 3/3, 224-library/32-binary root, all nine candidate checks, protected
PR #26, exact merge-head workflows, and pinned Linux/Windows LLVM 22.1.8 `-O0`/`-O2`
gates pass.

What would change the decision: evidence that private carrier normalization changes
observable source identity, that enum ownership is unsound for these values, or that a
general substitution engine is required would have stopped the slice. Accepted
CAP-003 does not settle context-free inference, general generics, question-mark
propagation, collections, String errors, aggregate/reference storage, public ABI, or
lifetimes/drop. The required fresh three-gap ranking was completed before CAP-004 and
selected the accepted explicit generic-struct slice; ranks 2 and 3 were not automatic
follow-ons. The next fresh ranking against accepted CAP-004 is recorded in the
CAP-005 authorization and selected the now-accepted generic-transport slice. The
fresh CAP-006 ranking against accepted CAP-005 selected explicit user-defined
recursive-CopyData generic enums (25) ahead of the canonical Milestone 0 contract
(24) and owned dynamic collections (22). CAP-006 is accepted through protected PR #33
and exact merge-head evidence; no neighboring generic topology inherits priority.

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

Accepted CAP-007 closes the canonical entrypoint mechanism for the currently frozen
trusted validation routes and is full-root/public-system green. The exit remains in
progress until the chosen stable subset has no unclassified critical false-success
defect.

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

Accepted CAP-011 closes the generic-data-structure half of this exit with one
fixed-capacity recursive-CopyData `Window<T>` API, exact-head conformance, verified
LLVM, and Linux/Windows native execution. Accepted CAP-012 closes the other half by
composing nested immutable/mutable CopyData loans into the representative telemetry
application through checked IR, verified LLVM, and pinned Linux/Windows native
execution. The selected exit gate is met; general ownership, generics, collections,
layout/ABI/destruction, and the broader milestone ambitions remain partial.

### Post-CAP-018 ranking

CAP-018 closed immutable exact-array value/result composition inside CAP-014's existing
profile. CAP-016 and CAP-017 separately completed their architecture probes and stopped
because the missing namespace/visibility and propagation-syntax contracts are not
founded; they are not implementation successors. Scores are 1--5 with higher better;
`Risk` and `Evidence` are delivery favorability, so 5 means lower implementation risk
or lower evidence cost.

| Rank | Gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Risk | Evidence | Total |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | Mutable loop-produced exact-`i32` flat-array results | 5 | 5 | 5 | 5 | 3 | 3 | 26 |
| 2 | Bounded nonempty recursive exact-`i32` array / 2D matrix pipeline under one shared recursive shape authority | 5 | 5 | 5 | 4 | 2 | 2 | 23 |
| 3 | Runtime byte/file acquisition into a bounded owned buffer | 5 | 5 | 5 | 4 | 1 | 1 | 21 |

1. Mutable loop-produced exact-`i32` flat-array results.

   Before, `exact-i32-array-v0` can construct, return, bind, forward, and consume
   immutable flat array values, but cannot initialize a mutable flat array, transform
   it with guarded runtime-indexed loop writes, and return the produced value. After,
   one ordinary bounded nongeneric function can initialize an exact flat array, update
   its lanes once through checked loop selectors, return it by value, and feed it into
   the accepted CPU kernel while preserving wrapping, bounds-trap, and source-ownership
   oracles.

   Stop and rerank if the complete flat mutation/result class needs new source mutation
   semantics, partial/uninitialized-array semantics, reference escape, stable
   ABI/layout, more than the authorized profile/backend phase pair, or any guard duplicated
   outside the shared profile and projected-place authorities.

   What would change our mind: Evidence that the checked mutation path cannot prove
   whole-array initialization and returned-value identity without a new IR/verifier
   contract, or that one shared recursive-array slice can safely subsume this capability
   at comparable scope and risk, would change the decision.

2. Bounded nonempty recursive exact-`i32` array / 2D matrix pipeline under one shared
   recursive shape authority.

   Before, the exact CPU profile admits only flat nonempty integer arrays, so no trusted
   exact 2D matrix value can be produced, transported, indexed across both dimensions,
   and consumed by a matrix-shaped CPU computation. After, one canonical recursive
   exact-array shape authority should admit a separately bounded nonempty integer matrix
   class and execute a composed 2D kernel through checked multidimensional bounds,
   verified LLVM, and Linux/Windows native oracles—without rank-, dimension-, or
   argument-order-specific classifiers.

   Stop if depth/product bounds are unfrozen, recursive physical mapping cannot consume
   the same canonical shape as source admission, a stable aggregate ABI or new ownership
   semantics are required, or implementation starts adding rank-specific topology
   guards.

   What would change our mind: If a task-local probe shows the useful matrix workload is
   already expressible clearly with row 1 plus flat indexing, defer recursive syntax;
   if the recursive authority safely absorbs row 1 within the same bounded phase/risk
   envelope, combine the work instead of stacking topology milestones.

3. Runtime byte/file acquisition into a bounded owned buffer.

   Before, Aero can interpret only source-embedded character/array data; it cannot
   acquire external workload bytes. After the eventual capability, a cross-platform
   program should acquire a size-bounded owned byte buffer, report a frozen typed
   acquisition failure, and hand validated bytes to parsing and CPU computation.

   This row is a strategic gap, not an authorizable implementation: stop until path
   and byte identity, capacity/initialized-count, partial-read/EOF, error mapping,
   ownership/drop, runtime linkage, sandboxing, determinism, and Linux/Windows behavior
   are explicitly frozen.

   What would change our mind: Evidence that a platform-neutral caller-provided byte
   slice or embedded-binary source unlocks the flagship workload sooner and safely
   avoids filesystem/runtime semantics would replace or reorder this row.

The ranking favors broad executable composition and a shortest credible path toward
the Aero-native CPU workload. It authorizes no implementation by itself, and it does
not revive the stopped module or propagation designs.

## Milestone 3 - Aero-native AI/ML infrastructure flagship

- Accepted CAP-014 provides the first bounded CPU computation toward this milestone:
  an exact wrapping `i32` fixed-array kernel with guarded dynamic indexing and a
  cross-platform native oracle. Accepted CAP-018 widens that same profile with
  immutable exact-array result composition. Together they still do not meet the
  milestone exit.
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
