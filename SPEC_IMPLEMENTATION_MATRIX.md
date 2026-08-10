# Aero Specification-to-Implementation Matrix

Audit basis: `8f8c7337a4008082fd2a443fcc814b5847b8663f`.

This matrix records stages independently. `Y` means direct evidence for the
listed slice, `P` means partial or known-defective support, `N` means absent,
`?` means not yet verified, and `—` means not applicable. The only feature-level
classifications are `ABSENT`, `DESIGNED`, `PARSED_ONLY`, `PARTIAL`,
`EXPERIMENTAL`, `END_TO_END`, and `STABLE`. No row is `STABLE` during the initial
audit.

Abbreviations: `Res` name resolution; `Ty` type checking; `Own` ownership
checking; `TIR` typed/structured IR; `BE` LLVM or other backend lowering; `Exec`
successful execution; `+/-/D` positive, negative, and diagnostic tests.

## Language features

| Feature | Spec | Lex | Parse | Res | Ty | Own | TIR | BE | Exec | + | - | D | Docs | Class |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| Integers/floats and arithmetic | Y | P | Y | — | P | — | P | P | P | Y | P | P | Y | PARTIAL |
| Booleans | Y | Y | Y | — | P | — | P | P | P | Y | Y | Y | Y | PARTIAL |
| Unicode characters | Y | Y | Y | — | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Bindings and mutability | Y | Y | Y | P | P | P | P | P | P | Y | P | P | Y | PARTIAL |
| Primitive compile-time const (accepted `CORE-082`) | Y | Y | Y | P | P | — | P | P | P | Y | Y | Y | Y | PARTIAL |
| Type annotations | Y | Y | Y | P | P | N | P | P | P | Y | P | P | Y | PARTIAL |
| Comparisons/logical/unary ops | Y | Y | P | — | P | — | P | P | ? | Y | P | P | Y | PARTIAL |
| Functions and returns | Y | Y | Y | P | P | P | P | P | P | Y | P | P | Y | PARTIAL |
| Function-call signatures | Y | Y | Y | P | P | N | P | P | P | Y | P | P | Y | PARTIAL |
| Shared function-call classification | Y | Y | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| If/else | Y | Y | P | P | P | P | P | P | P | Y | P | P | Y | PARTIAL |
| While/for/loop/break/continue | Y | Y | P | P | P | P | P | P | P | Y | P | P | Y | PARTIAL |
| Strings and formatting | Y | P | P | — | P | P | P | P | P | Y | P | P | Y | PARTIAL |
| Shared intrinsic method classification | P | Y | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Fixed arrays | Y | Y | Y | P | P | P | P | P | ? | Y | P | P | Y | PARTIAL |
| Fixed arrays of all-scalar Copy structs | Y | — | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Tuples | Y | Y | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Struct declarations | Y | Y | P | P | P | N | P | P | P | Y | Y | Y | Y | PARTIAL |
| Struct construction | Y | Y | Y | P | P | N | P | P | P | Y | Y | Y | Y | PARTIAL |
| Named field access | Y | Y | Y | P | P | N | P | P | P | Y | Y | Y | Y | PARTIAL |
| All-scalar struct Copy transport | Y | â€” | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Acyclic named Copy aggregates | Y | - | Y | P | P | P | P | P | Y | Y | Y | Y | Y | PARTIAL |
| Recursive finite CopyData composition | Y | - | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Unary recursive CopyData enum payloads | Y | - | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Positional multi-field recursive CopyData enum variants | Y | - | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Enums and construction | Y | Y | P | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Pattern matching | Y | Y | P | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Fresh owned-enum Match-expression results | Y | Y | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Conditional direct-owner enum Match results | Y | Y | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Unified typed CopyData/owned-enum Match results | Y | Y | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Generics and substitutions | Y | Y | P | P | P | P | N | N | N | P | P | P | Y | PARSED_ONLY |
| Traits, bounds, and impls | Y | Y | P | P | P | P | N | N | N | P | P | P | Y | PARSED_ONLY |
| Moves | Y | — | Y | P | P | P | ? | ? | ? | P | P | P | Y | PARTIAL |
| Direct mutable Copy-place reassignment | Y | Y | Y | P | Y | Y | Y | Y | P | Y | Y | Y | Y | PARTIAL |
| Direct mutable owned-enum reassignment | Y | Y | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Acyclic conditional owned-enum joins | Y | Y | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Acyclic moved/maybe-moved enum reinitialization | Y | Y | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Fresh per-iteration owned enums in statement loops | Y | Y | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Balanced loop-carried owned-enum reinitialization | Y | Y | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Convergent direct-enum loop ownership fixed points | Y | Y | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Local immutable Copy-place references | Y | Y | Y | P | P | P | P | P | Y | Y | Y | Y | Y | PARTIAL |
| Mutable/general references | Y | Y | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Mutable whole-place enum references (`CORE-083`) | Y | Y | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Immutable enum-reference Match reads (`CORE-084`) | Y | Y | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Mutable-owner immutable enum loans (`CORE-085` accepted) | Y | Y | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Mutable enum-reference Match reads and homogeneous `Void` Match results (`CORE-086` accepted) | Y | Y | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Mixed mutable-reference and CopyData signatures (`CORE-087` accepted) | Y | Y | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Mixed exclusive/shared-reference signatures (`CORE-088` accepted) | Y | Y | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Multiple exclusive-reference signatures (`CORE-089` accepted) | Y | Y | Y | P | P | P | P | P | P | Y | Y | Y | Y | PARTIAL |
| Static projected CopyData assignment (`CORE-090` accepted) | Y | Y | Y | P | Y | Y | Y | Y | P | Y | Y | Y | Y | PARTIAL |
| Closures | P | P | P | N | N | N | N | N | N | N | Y | Y | Y | PARSED_ONLY |
| Modules/imports/visibility | Y | Y | P | P | N | N | N | N | N | P | P | P | Y | PARSED_ONLY |
| Standard collections | P | Y | P | N | P | P | P | P | ? | P | P | P | P | EXPERIMENTAL |
| C/foreign-function interface | P | ? | ? | ? | ? | ? | ? | ? | ? | ? | ? | ? | P | DESIGNED |

Accepted public `CORE-069` supplies the evidence for the existing positional multi-
field recursive CopyData enum row without promoting it beyond `PARTIAL`. Accepted
public `ARCH-002` only normalizes binding-annotation classification and phase routing;
it changes no language-feature stage, evidence cell, or feature-level class.
Accepted public `CORE-071` adds exact negative/diagnostic evidence for the already
parsed-only Rust-like `use` subset: parsed path/alias/glob data and source location are
preserved, while semantics and checked admission fail closed before IR. It does not
change the combined modules/imports/visibility row or implement name resolution.
Accepted public `CORE-080` adds syntax evidence for the founding direct/aliased dotted
`import` grammar under a distinct AST identity and the same syntax-aware unsupported-
import authority. Executable imports still fail before checked IR, so no positive
semantic, IR, backend, execution, or stability cell changes and the row remains
`PARSED_ONLY`.

Accepted public `CORE-082` adds only exact annotated primitive constants whose
closed expressions use literals, prior lexical constants, and the already admitted
primitive operators. One shared contract evaluates and substitutes them before checked
IR; no constant storage, global, symbol, public layout/ABI, aggregate constant,
cross-module lookup, runtime dependency, or general CTFE is claimed. Bounded PR #6,
all nine exact-head checks, protected merge `962bb49f`, and the three exact post-merge
master workflows pass; the row remains `PARTIAL`.

Accepted public `CORE-083` adds only non-escaping mutable whole-place references to the
already admitted destructor-free enum class. One shared pointee classifier serves
semantics and independent checked admission; exact schema-bearing
loan/parameter/write/end identities reach independently verified private pointer LLVM.
The focused target is 5/5 and the exact repository-root gate is green at 211 library
and 32 binary tests. Bounded PR #8, all exact-head checks, protected merge
`680bc6ca`, the three exact post-merge master workflows, and pinned Linux/Windows LLVM
22 native exit 83 pass. Immutable enum references, reads or Match through a reference,
reference results/escape/storage/capture, projected or partial mutation, enum aggregate
storage, generic/named-field/String-payload enums, drop/lifetimes/NLL, stable ABI/FFI,
and memory-safety claims remain absent; the row therefore remains `PARTIAL`.

Accepted public `CORE-084` adds only repeated exhaustive Match reads through
non-escaping immutable references to initialized immutable direct owners of that same
admitted enum class. One shared classifier serves semantic analysis and independent
checked admission; exact owner-place and immutable Match-read identities reach private
pointer LLVM, and the verifier ties schema, reference provenance, read adjacency, and
dispatch use together. The focused target is 4/4 and the exact repository-root gate is
green at 212 library and 32 binary tests. Bounded PR #10, corrected exact-head CI/Rust
CI/CodeQL, protected merge `ae0f0901`, all three exact post-merge master workflows, and
pinned Linux/Windows LLVM 22 native exit 84 pass. Free enum dereference or transport,
mutable-owner loans,
mutable-reference reads, reference results/escape/storage/capture, aggregate enum
storage, unsupported enums, drop/lifetimes/NLL, stable ABI/FFI, and memory-safety claims
remain absent; the row therefore remains `PARTIAL`.

Accepted public `CORE-085` adds only multiple non-escaping immutable references to an
initialized mutable direct owner of that admitted enum class. One shared predicate
controls source admission and live-loan loop-edge exclusion; exact reference, source,
schema, and lexical-end identities reach private pointer LLVM; and the independent
verifier counts overlapping aliases and proves identical loan state at CFG joins.
Bounded PR #12, its exact-head checks, protected merge `d0832c6f`, the three exact
post-merge master workflows, and pinned Linux/Windows native exit 85 pass. Mutation,
move, mutable borrow,
owned Match, escape, free enum dereference/transport, reference results or storage,
aggregate enum storage, unsupported enums, lifetime/NLL/drop, stable ABI/FFI, and
memory-safety claims remain absent; the row therefore remains `PARTIAL`.

Accepted public `CORE-086` adds exhaustive Match observation through the existing
active exclusive mutable enum-reference class. The same source classifier handles
immutable and mutable reference Match while preserving mutability; checked IR retains
distinct identities, and the verifier requires exact active provenance, schema, and
immediately adjacent dispatch. One shared result contract also admits homogeneous
discarded `Void` Matches for owned, immutable-reference, and mutable-reference
scrutinees without result storage. `print!` and `println!` are effect-only `Void` in
both semantic routes and remain rejected in every value context. The focused target is
5/5, the exact repository gate is green at 214 library and 32 binary tests, and the
two-module specimen externally verifies and executes under local pinned LLVM/Clang
22.1.8 at exact exit 86. Bounded PR #13, its exact-head checks, protected merge
`e2014a17`, and all three post-merge workflows pass. Raw enum extraction/transport, overlap,
escape, new lifetime/NLL/drop, layout/ABI, accelerator, safety, or stability semantics
remain absent; the row therefore remains `PARTIAL`.

Accepted public `CORE-087` composes exactly one admitted mutable whole-place reference
parameter with one or more recursive finite CopyData parameters under one shared
signature-topology predicate. Reference-first, -middle, and -last forms, arbitrary side
counts, CopyData and admitted enum pointees, direct owners, alias reborrows, parameter
forwarding, CopyData/`Void` results, and direct modules execute through checked IR,
independent verification, private pointer LLVM, and pinned native exit 87. Side
arguments must be independent of the reference source and retain their relative
evaluation order before the adjacent borrow/call/end window. The focused target is 3/3
and the verifier corruption control is 1/1. Bounded PR #14 passed all nine exact-head
checks, merged through protected master as `b07efe29`, and passed post-merge CI
`31406731077`, Rust CI `31406731094`, and CodeQL `31406730798`. Multiple references,
projections, reference results/escape/storage/capture, lifetime/NLL/drop, public
layout/ABI/FFI, accelerators, and memory-safety claims remain absent from CORE-087;
the row therefore remains `PARTIAL`.

Accepted public `CORE-088` closes the complete next signature class: exactly one
admitted mutable whole-place reference, one or more admitted immutable whole-place
references, and zero or more recursive CopyData companions in every declared order.
One shared predicate serves semantic classification and independent checked signature
verification. The existing indexed call authority proves the sole mutable source
independent from every other argument; immutable arguments may repeat an immutable
source. Checked verification additionally requires each immutable call operand to be
an exact immutable-borrow or immutable-parameter identity, rejecting raw-owner and
active-mutable substitutions. The focused target is 3/3, its corruption control is
1/1, the full root gate is green, and pinned LLVM/Clang 22.1.8 executes public and
independent native exit 88. Bounded PR #15 passed all nine exact-head checks, merged
through protected master as `a7627aa1`, and passed post-merge CI `31410739806`, Rust
CI `31410739830`, and CodeQL `31410738951`. Multiple mutable
parameters, projections, reference results/escape/storage/capture, lifetime/NLL/drop,
public layout/ABI/FFI, accelerators, and memory-safety claims remain absent; the row
therefore remains `PARTIAL`.

Accepted public `CORE-089` closes the complete remaining multiple-exclusive-reference
signature partition under the same shared classifier: two or more mutable whole-place
references, any admitted immutable references, and recursive CopyData companions in
every declared order and count. The call contract enumerates every mutable parameter,
requires pairwise-distinct mutable roots disjoint from every non-mutable argument tree,
and lowers one exact declared-order N-borrow/call/reverse-N-end window. Independent
verification reconstructs the same roots and rejects duplicate/overlapping operands,
window reordering or separation, raw owners, binder corruption, and forged ends. The
focused target is 3/3, the corruption control is 1/1, the affected reference ring is
19/19, and pinned local LLVM/Clang 22.1.8 executes public and independent native exit
89. The full root gate is green at 216 library tests, 32 binary tests, every
integration target, and doc tests. Bounded PR #16 passed all nine exact-head checks,
merged through protected master as `7fbaaaa4`, and passed post-merge CI, Rust CI, and
CodeQL. Projections, reference
results/escape/storage/capture, lifetime/NLL/drop, public layout/ABI/FFI, accelerators,
and memory-safety claims remain absent; the row therefore remains `PARTIAL`.

Accepted public `CORE-090` admits exactly one nonempty static projection path rooted
at an initialized mutable owned direct local recursive finite CopyData value. The path
may contain any finite mix of declared named fields, tuple constants, and nonnegative
in-range integer-literal fixed-array indexes; the exact CopyData leaf accepts only an
exact-type RHS. One shared classifier serves both semantic routes and checked
admission/lowering, while the verifier independently reconstructs the projection root
and requires an existing typed mutable-owner allocation. The focused target passes
1/1, shared classifier and corruption controls pass 2/2, the affected ring passes
15/15, the full root gate is green at 218 library tests and 32 binary tests plus every
integration target and doc tests, and pinned local LLVM/Clang 22.1.8 executes the
tracked direct-module specimen at exact exit 90. Bounded PR #17 passed all nine
exact-head checks, merged through protected master as `12820561`, and passed exact
post-merge CI, Rust CI, and CodeQL.
Dynamic/computed indexes, projected borrows, partial moves, enum/non-Copy subplaces,
alias analysis, NLL/lifetime/drop, public layout/ABI/FFI, accelerators, and memory-safety
claims remain absent; the row therefore remains `PARTIAL`.

Accepted public `CORE-072` splits the prior combined Boolean/character row and moves
only the Unicode-character slice from design-only to bounded partial execution. Exact
raw/escaped scalars, type identity, equality/inequality, the complete existing
recursive CopyData transport class, checked IR, independent verification, private
LLVM, public CLI execution, and native exit 197 have positive/negative/diagnostic
evidence. Arithmetic/order/casts, strings/printing, executable literal patterns,
generic/trait behavior, public layout/ABI/FFI, accelerators, and stability remain
unsupported; the row cannot move beyond `PARTIAL`.

Accepted public `CORE-073` adds only acyclic whole-owner reinitialization for the
already admitted destructor-free enum class. Exact writes from `Moved` or
`MaybeMoved` restore the target to `Owned` through one shared transition authority,
while independent verification proves predecessor consumption, exact schema/value,
dominance, and the checked write kill. Every loop-contained reinitialization, partial
move/projection, enum aggregate storage or borrowing, destructor/drop/lifetime rule,
and general CFG fixed point remains unsupported; the row stays `PARTIAL` and does not
broaden the aggregate topology.

Accepted public `CORE-074` admits only fresh owned-enum results from exhaustive
identifier-bound Match expressions. Every arm must produce the same already admitted
enum through a constructor, exact call without additional owned-enum consumption, or
recursively fresh nested Match. Exact checked result/dispatch schemas, one distinct
target-dominated assignment per arm, all-path initialization, one merged load, and
later ownership are independently verified. The row remains `PARTIAL`.

Accepted public `CORE-075` adds direct initialized local/owned-parameter result origins
of the exact enum schema. A shared dynamic-path classifier permits the same owner only
across mutually exclusive arms, derives all-path `Moved` and partial-path `MaybeMoved`,
and rejects same-path duplicates and loop effects. Checked enum values or checked
mutable-place loads feed the checked result place and checked assignment; independent
verifier CFG ownership proof rejects post-merge source reuse. Calls consuming another
owner, external nested scrutinees, aggregate enum storage, wider patterns, enum
borrowing/projection, partial moves, drop/lifetimes, stable ABI, and general CFG
semantics remain excluded; the row stays `PARTIAL`. All exact-head public checks and
pinned stable/nightly native exit 211 pass.

Accepted public `CORE-076` unifies exhaustive Match results over the complete existing
recursive finite CopyData universe and the separately constrained owned-enum class.
Semantic inference and checked admission consume one result classifier; primitives,
fixed arrays including zero length, arity-two-or-more tuples, finite acyclic structs,
and owned enums lower through one generic checked result place, exact typed arm writes,
all-path initialization, and one merged load. Wrong type/value metadata, generic
stores, missing/repeated/bypassed writes, premature/duplicate loads, and enum-owner
fabrication reject independently. Strings, reference results, unit/unary tuples,
dynamic collections, enum-in-CopyData storage, cyclic/unsupported structs, wider
patterns, runtime/layout/ABI/drop/lifetime, accelerator, release, safety, and stability
semantics remain excluded; the row stays `PARTIAL`. All eight exact-head checks and
pinned stable/nightly LLVM/Clang 22.1.8 native exit 223 pass at exact implementation
`aefeb2d81fb5374e7373a4819f3c92f83a95eb35`.

Accepted public `CORE-077` adds only the balanced loop-carried row. An exact direct
mutable admitted enum must enter `while`, fixed-array `for`, or `loop` as `Owned` and
must be restored to exactly `Owned` on every reachable condition/iterable edge,
fallthrough or `continue` backedge, and `break` exit. Return paths do not join and
nested transfers belong to the nearest loop. Semantic analysis and independent checked
admission collect snapshots but consume one edge classifier; verifier CFG proof rejects
missing, bypassed, one-path, generic-store, wrong-schema, cycle, and exit repairs. The
Exact implementation `a93d8d38c5f2a2499ce036f659c13cb2ec4fefcb`, all eight
exact-head checks, and pinned stable/nightly LLVM/Clang 22.1.8 native exit 227 pass
while preserving exits 149/223. Projections/partial moves, enum storage/borrowing,
drop/lifetimes, stable ABI, imports, accelerators, release, safety, and general
non-enum loop dataflow remain excluded; the row stays `PARTIAL`.

Accepted public `CORE-079` adds the convergent fixed-point row without changing enum
topology or backend layout. One shared classifier joins `Owned`/`Moved`/`MaybeMoved`
at loop headers and exits; semantic analysis and independent checked admission recheck
`while`, admitted fixed-array `for`, and `loop` until the finite header stabilizes.
The existing verifier remains the independent cycle proof. Exact implementation
`5b1ec7340db72354542ab325a9f75cad398857c2`, all nine exact-head checks, pinned
stable/nightly exits 149/223/227/229, and bounded Windows public/manual exit 229 pass.
The ownership row remains `PARTIAL`; no broader enum, ownership, ABI, or safety
capability follows.

Accepted public `CORE-078` changes no language or matrix row. Exact implementation
`70f59fd72e96246b2ebefdf1ae53a9b7f3280cfe` adds one Windows x86_64 CPU evidence
lane using the official full LLVM/Clang 22.1.8 x86_64 MSVC archive pinned by SHA-256.
It preserves the existing MSVC triple/layout, fails artifact-free on invalid source,
externally and machine verifies, emits COFF, links through Clang/MSVC, and executes the
public and manual paths at exit 227. All nine exact-head checks pass, while Linux
stable/nightly preserve exits 149/223/227. The CPU row remains `PARTIAL`; this bounded
evidence is not general Windows or stable ABI support.

## Compiler, tooling, and ecosystem surfaces

| Surface | Interface | Shared compiler truth | Artifact/result | Failure tests | Integration evidence | Docs | Class |
|---|---:|---:|---:|---:|---:|---:|---|
| Library `compile_program` | Y | P | LLVM text or located parse error | Y | P | P | PARTIAL |
| Library `compile_file` | Y | P | In-memory checked LLVM text or file-located failure; direct `mod` only | Y | P | P | PARTIAL |
| Compiler options | Y | N | Default path preserved; accepted CORE-020 rejects nondefaults before lexing | Y | Y | P | PARSED_ONLY |
| CLI build/check | Y | N | P; surfaced compile failures nonzero | Y | P | Y | PARTIAL |
| CLI run | Y | N | CPU executes; accepted CORE-018 makes ROCm a temporary regular-file probe followed by status 1/no execution; CUDA status 1 | Y | P | Y | PARTIAL |
| CLI test | Y | N | Semantic analysis only; explicitly reports no execution; failures nonzero | Y | P | Y | PARTIAL |
| Formatter | Y | N | Text trimming | N | N | P | EXPERIMENTAL |
| Diagnostics/source spans | Y | P | Point/one-char ranges | P | P | Y | PARTIAL |
| LSP | Y | N | P | P | P | Y | EXPERIMENTAL |
| Documentation generator | Y | P | Markdown | P | P | Y | EXPERIMENTAL |
| Profiler | Y | N | Timing/trace or located parse error | Y | P | Y | EXPERIMENTAL |
| Project initialization | Y | — | Project files | Y | P | Y | EXPERIMENTAL |
| Module resolver | Y | P | Resolved source | P | P | Y | EXPERIMENTAL |
| Registry | Y | — | Local search and dry-run plans; live transport quarantined | Y | N | Y | EXPERIMENTAL |
| Conformance command | Y | P | 3 cases + 4 deterministic checks | P | P | P | EXPERIMENTAL |
| Package lock/reproducible resolution | P | ? | ? | ? | ? | P | DESIGNED |

## Backend summary

Detailed stage evidence lives in `BACKEND_STATUS.md`.

| Backend/surface | Selectable | IR transform | Object | Link | Real execution | Numerical checks | Performance evidence | Class |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| CPU | Y | Y | P | P | P; pinned Linux and bounded Windows x86_64 evidence accepted | P | P | PARTIAL |
| ROCm | Y | Y | P, temporary/unchecked at AUDIT-024 | N | N | N | External llama.cpp only | EXPERIMENTAL |
| CUDA | Y | P | N | N | N | N | N | PARSED_ONLY |
| Graph compilation | Y | Y | — | — | Internal scalar-helper transform only | N | N | EXPERIMENTAL |
| Quantization | Y | Y | — | — | Scalar-double helper transform only | N | N | EXPERIMENTAL |

## Evidence notes

- Accepted public `CORE-068` implementation
  `55b61c31fc6dd822097daa5d4f371d04ec0d6264` removes semantic and checked-call
  fallbacks that fabricated `Int` or deferred undefined calls. One phase-aware
  supported/explicitly-rejected/preserved classifier supplies both semantic paths and
  checked admission/lowering with exact target, parameter, result, argument, and use-
  context facts. Existing nongeneric signatures over admitted scalars, recursive
  CopyData, owned enums, and reference parameters remain bounded; unsupported
  annotations, generic/trait/closure calls, wrong arity/type, and `Void` value use
  reject before checked IR. Classifier units, the exhaustive topology target,
  compatibility ring, and exact root gate pass at 185/185 library tests. The tracked
  direct-module specimen links locally with Clang 19.1.5 and executes exit 181. All
  eight candidate-head checks pass; stable LLVM/Clang 22.1.8 rejects the invalid
  fixture, externally and machine-verifies, object-lowers, links, and executes exit
  181, while nightly repeats verification and exit 181. No row becomes `END_TO_END`
  or `STABLE`.

- Accepted public `CORE-067` implementation
  `e7525bf039339909c8f4f5cc68262fdf498079e0` removes both duplicate semantic method
  tables and routes semantic inference, checked admission, and trusted lowering
  through one stage-aware supported/explicitly-rejected/preserved classifier. Unknown
  or unimplemented methods no longer acquire fabricated `Int`, `Bool`, `String`,
  `Void`, `Option`, or `Vec` results, and the checked lowering path has no scalar-zero
  fallback. Normalized leaf helpers compute only fixed-array or static-String values.
  Exact zero-argument `.len()` and `.is_empty()` execute for fixed arrays of any
  already admitted recursive CopyData element; established static String queries and
  Array/Vec `.iter()` compatibility remain exact. The focused aggregate, classifier
  units, 29-target compatibility ring, and exact root gate pass at 183/183 library
  tests. The tracked direct-module specimen links with local Clang 19.1.5 and executes
  exit 167. All eight candidate-head checks pass; stable LLVM/Clang 22.1.8 rejects the
  known-invalid fixture, externally and machine-verifies the specimen, object-lowers,
  links, and executes exit 167, while nightly repeats exit 167. This accepts only the
  frozen bounded method class; no row becomes `END_TO_END` or `STABLE`.

- Accepted `CORE-066` at exact implementation
  `e40804ea86888b38548fd5bf42926be2be7eb5ed`, tree
  `6cea8bbf63aa7aafb43fbb25152dd860f6684aae`, and stable patch ID
  `7c4e6ac77db90dc7c83048922382903958c09632` admits exact fresh
  enum constructors and enum-returning call results inside every currently checked
  statement-loop form, with inferred/exact and immutable/mutable locals, all accepted
  recursive CopyData payload schemas, every accepted consuming operation, nested
  loops, return, break, and continue. Pre-loop enum ownership must remain unchanged at
  conditions/backedges. The exhaustive test exposed and closed the existing array-
  `for` continue defect: one centralized loop-label allocator and shared `for`
  iteration tail now send continue through an explicit increment block. Independent
  verifier controls accept exact fresh result/place definitions on cycles and reject
  bypassed definition, two consumptions per iteration, and unreset outer-owner cycles.
  The focused target, affected compatibility ring, formatting, exact serialized root
  gate, deterministic direct-module LLVM, and local Clang 19.1.5 native exit 149 pass.
  All eight public checks pass; pinned LLVM/Clang 22.1.8 rejects invalid IR, externally
  verifies, machine-verifies, object-lowers, links, and executes stable/nightly exit
  149. No general loop ownership, moved-target reinitialization, ABI, or safety claim
  moves.

- `CORE-065` is accepted public at exact implementation
  `f4daeea6d7b032e686b4c7d184fe80ef38076665`, tree
  `7cd4ec6da2d9ce44f63741222a5b128396358bfe`, and stable patch ID
  `708c1a6cab096f89e76577212a241554225897a2`. It adds exact acyclic `if` ownership
  joins over admitted enums. Sibling arms begin from one shared entry snapshot; definitely
  returning arms do not reach the join; reachable states join to `Owned`, `Moved`, or
  `MaybeMoved`; and later use of `MaybeMoved` fails before IR. The same classifier is
  consumed by semantic analysis and checked admission. Independent checked-IR CFG
  dataflow tracks enum result/place identity and rejects serial, partial-merge, and
  cyclic double consumption while accepting mutually exclusive sibling consumption.
  Loop-carried ownership changes remain rejected pending fixed-point semantics. The
  exhaustive source/IR/verifier/LLVM/CLI target, corruption unit, compatibility ring,
  formatting, all-target/all-feature checking, correctness Clippy, docs, exact root
  gate, 182 library tests, and 188 binary tests pass. All eight public checks pass;
  stable LLVM/Clang 22.1.8 proves the known-invalid control and exact native exit 137,
  while nightly repeats exit 137. No general CFG ownership, stable ABI, or safety claim
  follows.

- `CORE-064` is accepted public at exact implementation
  `79aed71371e192a07218d437e882a863653b6826`, tree
  `ac80c49aca3fb875c44d132f930567e95d81f698`, and stable patch ID
  `1bb2c9c19f6d427122f83bffc59d3f18f0a5b3e4`. It
  generalizes the direct mutable-place classifier and checked identities from the
  recursive `CopyData` subset to an owned-place class that also contains every enum
  admitted by CORE-063. Exact inferred/annotated mutable enum locals accept constructor,
  enum-returning call, and distinct-local replacement; a distinct local is moved,
  self-replacement rejects, and only CopyData places remain borrowable. The independent
  verifier requires exact schema/place/value identity, adjacent single initialization,
  dominance, and checked later writes. Private LLVM uses exact typed enum load/store
  without scalar fallback or public layout. The exhaustive target, formatting,
  all-target/all-feature checking, correctness Clippy, docs, exact root gate, and
  complete Rust test surface pass at 180 library and 186 binary tests. All eight public
  checks pass. Stable job `92376666972` uses LLVM/Clang 22.1.8 for the known-invalid
  control, external and machine verification, object lowering, explicit private
  non-PIE linking, and exact native exit 131; nightly job `92376666842` repeats exit
  131.

- `CORE-063` is accepted public at exact implementation
  `2a5c3c58192dc65116c436d6ae76da5829eeba52`, tree
  `8a5cef6b14214e76349a41f6997d5fa19595858f`, and stable patch ID
  `276af069807b6f59c233a2f281c1b0d0b8c899b8`, with verified native-link repair head
  `bebd0b6a87108219497187a5952688c95c397158`, for unique nongeneric
  enums whose variants are unit or carry exactly one recursive `CopyData` payload.
  Enum admission delegates to the accepted recursive struct/array/tuple classifier;
  semantic preflight, inference, checked IR, the independent verifier, and private LLVM
  retain the same exact declaration-ordered schema. The exhaustive target covers
  construction, exhaustive identifier-bound Match, projections, internal transport,
  direct modules, corruption controls, and fail-closed unsupported leaves/topologies.
  Formatting, all-target/all-feature checking, correctness Clippy, docs, 179 library
  and 185 binary tests, and the exact root gate pass. All eight public checks pass;
  stable job `92363420145` uses LLVM/Clang 22.1.8 for external/machine verification,
  object lowering, explicit private non-PIE linking, and exact native exit 113. This is
  not a stable enum layout/ABI claim.

- `CORE-062` is accepted public for recursive finite CopyData composition at exact
  implementation `e62fd7470d8cb929d57d0c063815d7a99005d768`, tree
  `d2aff21a54c42d1ce649ef6668d50a4908315738`, and stable patch ID
  `458feb5ebc1355d83793084009e5ea7895a22129`.
  One registry-backed least-fixed-point classifier owns annotation and semantic-type
  admission for scalars, fixed arrays, arity-at-least-two tuples, and finite acyclic
  unique nongeneric nonempty named structs. Semantic analysis, checked IR, Copy-place
  ownership operations, function transport, and private LLVM consume the exact
  recursive schema; the verifier independently checks it. Bool/nested arrays,
  aggregate-bearing tuples, tuple/array-bearing structs, aliases, reassignment,
  immutable/mutable whole-place references, calls/results/recursion, dynamic fixed-
  array indices, chained projections, and direct modules pass the exhaustive target
  and local native exit 109. The exact root gate passes 178 library and 184 binary
  tests plus every integration/claim/doc target. All eight public checks pass; stable
  job `92344809072` uses LLVM/Clang 22.1.8 for external and machine verification,
  object/link gates, the known-invalid verifier control, and exact native exit 109.
  Unsupported leaves/topologies, aggregate comparison/destructuring, projected loans/
  writes, new lifetime/drop/ABI/safety semantics, and accelerator or performance claims
  remain absent.

- `CORE-061` is accepted public for direct whole-owner reassignment over its frozen
  Copy-data universe and closure false-success containment. Exact commit `de6fc0d`
  passes all eight checks and the pinned LLVM/Clang 22 exit-83 system lane with 175
  library and 181 binary tests. One checked mutable Copy-place allocation/assignment
  retains exact schema; executable closures remain `PARSED_ONLY` and reject before
  checked IR without a callable or fallback `i32` layout.

- The `CORE-061-CLOSURE` amendment keeps closure syntax and its opening source
  location parsed, but executable closures are unsupported. One shared diagnostic is
  consumed by both semantic inference paths and an independent checked-admission
  guard. The former lowerer that manufactured callable identities and defaulted
  unknown parameter/result types to `i32` is removed; the deprecated raw path produces
  no closure type, signature, environment, layout, symbol, call target, or LLVM
  definition. Negative coverage includes inferred and explicit bindings, comparisons,
  arguments, returns, array/struct storage, captures, calls, direct modules, and CLI
  no-artifact behavior. This is `PARSED_ONLY`, not partial closure execution.

- `CORE-059` and `CORE-060` are accepted public for immutable and exclusive mutable
  whole-place references over the exact previously admitted Copy-data universe:
  scalars, flat Copy-scalar tuples, fixed numeric arrays, fixed Copy-struct arrays, and
  finite acyclic Copy structs. One three-way classifier owns supported, explicitly
  rejected, and preserved schemas across semantic and checked admission. Exact
  recursive pointee/loan/write identity is retained through checked IR, independent
  verification, and private typed-pointer LLVM. Exact public implementations `5a78eb5`
  and `7c7a47a` pass all eight checks with pinned native exits 37 and 59. Projected
  origins/writes, reference results, new layouts, stable ABI/FFI, general lifetime/
  drop/safety, accelerator, performance, release, and stability claims remain absent.

- `CORE-058` moves only the bounded flat Copy-scalar tuple slice from parser-only to
  partial execution: arity at least two; exact ordered `Int`/`Float`/`Bool` elements;
  immutable binding and whole-value Copy; constant projection; scalar/tuple-only
  internal calls/returns; direct modules; checked tuple identities; independent
  verification; typed literal-aggregate LLVM; and pinned native exit 23. Exact public
  commit `421a0a9` passes all eight checks with 171 library and 177 binary tests.
  Unit/unary/nested/non-scalar, mutable, destructured,
  contained, generic/impl/closure, process-entry, public ABI/FFI, drop, accelerator,
  performance, release, and stability surfaces remain absent or quarantined. This
  bounded move supersedes DEC-010 only for the listed product; its other tuple
  rejection boundaries remain active.

- The lexer cannot return errors and currently converts some invalid input into
  valid tokens, so otherwise present lexical cells remain partial.
- At `bc9a148`, initialized `int`/`i32` and `float`/`f64` binding annotations are
  enforced exactly through active semantics, with positive, negative, diagnostic,
  artifact, and lexical-scope coverage. Uninitialized and non-numeric annotations
  remain unenforced or uncertified. At `8d5d8e7`, active semantic and IR paths also
  enforce monomorphic numeric/void top-level function calls and returns; boolean,
  generic, composite, method, string, and richer closure contracts remain open.
- Many declarations lose visibility, bounds, arguments, or source locations in
  the AST; parser presence alone therefore does not imply faithful parsing.
- Accepted public `CORE-081` removes the exact 35-module CLI/library overlap and makes
  compiler phases library-owned. The binary retains only CLI-specific modules and uses
  narrow hidden service bridges for direct-module cache material and registry
  quarantine without exposing resolver/IR representations. Architecture, unit,
  integration, all-features, static, documentation, and exact root gates are green;
  immutable public evidence remains pending, so tooling stays `EXPERIMENTAL` and no
  language row moves.
- Current conformance determinism checks are useful regression evidence, not
  formal-semantics proof.
- `AUDIT-022` at clean public head `c612f3b` reproduces a compiler package version
  of `0.3.0` while CLI version routes/banner present `1.0.0`. Reviewed public red
  `4b94dbd` binds that mismatch and the overstated three-example/four-repetition
  claims at exactly two preservation passes/five failures. The accepted `CORE-016`
  implementation derives CLI presentation from package metadata and
  classifies those checks and design/history documents without changing capability
  behavior; its focused claim and CLI targets pass 7/7 and exact full gate passes.
  Exact three-review-approved implementation `cc984d0` and record-only closure
  `ea036f2` each pass all eight public checks. R-008 is controlled for this selected
  claim boundary. Language semantics, package version, report schema, conformance
  algorithms, underlying capability classes, and release state remain frozen.
- `AUDIT-023` at clean accepted head `8869eca` classifies the 38 ignored Phase 5
  tests: 22 exact strict lexer/parser-retention candidates and 16 quarantines (14
  semantic plus 2 generic-impl retention gaps). `CORE-017` selects only test/evidence
  classification. Even if active, the 22 remain `PARSED_ONLY` evidence and do not
  change language semantics, capability classes, IR/backend behavior, or stability.
- Public-green preregistration `2c61535` freezes that boundary. Exact three-review-
  approved implementation `8be8c21` has exactly 22 strict lexer/parser-retention
  passes and 16 explicit quarantines with no production change; the exact full gate
  and all eight public checks pass. Exact three-review-approved record-only closure
  `3dd3bb4` also passes all eight public checks. R-012 is partially controlled for
  those 22 accepted `PARSED_ONLY` tests only; the 16 quarantines, 299 dormant tests,
  Cargo overlap, and all semantic/execution rows remain unchanged.
- `AUDIT-024` at clean public head `9ddc571` confirms CPU as the only real process
  execution route. ROCm reaches an unchecked temporary `llc` object and incorrectly
  returns status zero without link/launch; CUDA returns unavailable. The `gpu` alias
  is a tool/environment heuristic, graph output is verified textual internal scalar
  helpers, and quantization is scalar-double helper transformation without real FP8,
  per-channel execution, numerical proof, or device execution. Triple-approved
  tests-only `427fb4c` reproduced the exact public red split. Exact three-review-
  approved implementation `8bde0ff` passes CLI 10/10, claims 7/7, the complete gate,
  and all eight public checks with fail-closed ROCm/CUDA, explicit targets, exact
  non-device scalar-helper telemetry, and the Aero GGUF route disabled. The selected
  boundary is accepted at exact three-review-approved public record-only closure
  `2e0e17f`, which also passes all eight checks. No backend row is promoted and R-007
  remains open.
- `AUDIT-025` at clean accepted head `d0bd54e` confirms that `aero test` performs
  strict parse, direct-module collection, and semantic analysis only while current
  CLI/help/BUILD wording claims sources run and pass. `CORE-019` selects wording and
  exact CLI tests only; all stages, statuses, counts, discovery behavior, and
  capability classes remain frozen. Ignored nondefault `CompilerOptions` remain a
  separate R-006 runner-up.
- Triple-reviewed public tests-only `6728a39` proves that boundary with exact 9/2
  compiler/nightly failures, permitted stable fail-fast cancellation during tests,
  and four green CodeQL checks. Exact three-review-approved implementation `2fe580d`
  is focused 11/11, exact-full-gate green, and all-eight-public-checks green. The
  selected presentation boundary is accepted without promoting any matrix row or
  capability class and without adding test execution, IR, codegen, or runtime.
- Exact three-review-approved corrected record-only closure `63b6629` also passes the
  full gate and all eight public checks. No row/class promotion or execution evidence
  is inferred from closure.
- Exact three-review-approved final-state sync `25dec51` also passes all eight public
  checks. `AUDIT-026` is read-only and cannot promote any row or define
  `CompilerOptions` behavior.
- Public-green `AUDIT-026` preregistration `2c61ff9` supports the completed read-only
  finding: all 62 in-repository library callers used defaults, while every nondefault
  option was ignored across checked compilation. DEC-025 and preregistered `CORE-020`
  select pre-lexing rejection of nondefaults while preserving the facade and exact
  default behavior. At that preregistration checkpoint, the compiler-options row
  remained `PARSED_ONLY` with ignored behavior and no promotion.
- Exact three-review-approved preregistration `fae1374` passes all eight checks.
  Exact tests-only `037f44d` proves the ignored-option boundary publicly at 1/1 while
  all four CodeQL checks pass. The local one-guard candidate is focused 2/2,
  preservation 40/40, and full-gate green. The row remains `PARSED_ONLY`: explicit
  unsupported rejection is claim containment, not implemented option semantics; public
  implementation acceptance was pending at that checkpoint.
- Exact three-review-approved implementation `70cb0ad` passes all eight public checks.
  The accepted boundary preserves default LLVM/diagnostics and rejects nondefaults
  before lexing. The row remains `PARSED_ONLY`: no optimizer, debug-information,
  target-selection, CLI mapping, IR, codegen, or backend behavior is implemented.
- Exact three-review-approved record-only closure `5a8cd06` passes all eight public
  checks in compiler runs `30835593703`/`30835597576`, Rust `30835597620`, CodeQL
  `30835594365`, and aggregate `91759990615`. It changes no row or capability class.
  `AUDIT-027` is preregistered as read-only re-ranking and cannot promote a row.
- Public-green `AUDIT-027` basis `aa3e7a8` completes the read-only comparison. All
  auditors rank R-013 first; DEC-026 and preregistered `CORE-021` select only removal
  of the CPU success phrase for delegated nonzero exits while preserving exact child
  behavior. No compiler/backend row or capability class is changed or promoted.
- Exact three-review-approved tests-only `0873f65`, tree `51ec7d0a`, diff `f75a6360`,
  publicly reproduces the selected delegated-exit false-success boundary in compiler
  runs `30839264536` / `30839272375` and nightly Rust run `30839272429`; stable is
  cancelled during tests by fail-fast. CodeQL `30839264268` and aggregate
  `91772180985` pass. The one-condition production implementation passes focused CLI
  11/11, backend-claim 7/7, and the exact full local gate. Exact tree `0ad98c82`, diff
  `2dbbc395`, received three approvals and was published as `a4327be`; compiler
  `30839860335` / `30839862442`, Rust `30839862423`, CodeQL `30839859840`, and
  aggregate `91774125621` all pass. The selected presentation boundary is accepted
  without changing or promoting any compiler/backend row.
- Corrected record-only closure `b99e445`, tree `8a4c2d77`, diff `5abbf3a7`, passes
  compiler `30840427466` / `30840426655`, stable/nightly Rust `30840428215`, CodeQL
  `30840415565`, and aggregate `91775938704`. `AUDIT-028` is a preregistered
  read-only full-risk ranking and cannot change or promote any matrix row.
- Public-green `AUDIT-028` basis `399e04f` completes the full-risk ranking. R-013 is
  the only universal top-two residual; DEC-027 and preregistered `CORE-022` select
  only fail-closed `aero init` destination-entry preflight before writes. This
  project-tooling boundary does not change or promote a compiler/backend matrix row.
- Accepted `CORE-022` implementation `2a42324` makes final-entry `aero init`
  preflight non-following and fail-closed before writes. Exact tests-only `7cd8aba`
  reproduces Linux compiler 10/1; implementation passes focused/local gates and all
  eight public checks in compiler `30843592298` / `30843592784`, Rust `30843595560`,
  CodeQL `30843589175`, and aggregate `91786468184`. This project-tooling containment
  promotes no language, IR, codegen, CPU, ROCm, or CUDA matrix row.
- Exact record closure `aa29a00`, tree `e740df48`, diff `3eb8264b`, is triple-reviewed
  and passes compiler `30844324249` / `30844328660`, Rust `30844328850`, CodeQL
  `30844325051`, and aggregate `91788926688`. `CORE-022` is closed without changing
  any compiler/backend matrix classification.
- Public-green status synchronization `21153f3` passes compiler `30844798322` /
  `30844802332`, Rust `30844802044`, CodeQL `30844799426`, and aggregate
  `91790481511`. Preregistered read-only `AUDIT-029` ranks the complete residual set
  but cannot change or promote any matrix row.
- `AUDIT-029` completed from all-eight public-green basis `0e5cba1`, tree
  `6ac88db4`. The independent top selections are R-002 Boolean helper contracts,
  R-010 grammar-authority containment, and R-009 parser UTF-16 columns; R-012 is the
  common evidence-only runner-up. Lead reconciliation selects R-002's active one-
  phase semantic inconsistency. Checked IR already maps Boolean helper definitions,
  returns, calls, and storage to LLVM `i1`, but semantics registers only numeric/void
  contracts, accepts invalid Boolean calls/returns, and defaults other declared call
  results to `Int`. Preregistered `CORE-023` adds no matrix promotion: it freezes
  only exact `Ty::Bool` contracts for monomorphic non-entry helpers, with parser,
  AST, IR, verifier, codegen, ABI, generics, composites, coercions, and broader R-002
  closure excluded until separate evidence.
- Accepted `CORE-023` implementation `67ccdf2` closes only the semantic contract gap
  for monomorphic non-entry Boolean helpers. Triple-reviewed tests-only `c3f6e90`
  publicly reproduces the three semantic discrepancies; the one-file implementation
  passes focused/preservation/full gates and all eight checks in compiler
  `30850000615` / `30850005598`, Rust `30850005670`, CodeQL `30850001251`, and
  aggregate `91807553635`. Boolean helper parameters/returns now use exact `Ty::Bool`
  and valid calls infer `Ty::Bool`; checked IR/codegen remain unchanged and retain
  existing LLVM `i1` evidence. This is a PARTIAL function/type-contract improvement,
  not entry/ABI, generic/composite, execution, backend, or stability closure.
- Exact triple-reviewed record closure `0b88530`, tree `71ac4da7`, diff `adba01a1`,
  passes compiler `30850519757` / `30850524194`, stable/nightly Rust `30850524148`,
  CodeQL `30850520457`, and aggregate `91809289681`. No matrix row is promoted:
  `CORE-023` accepts only its non-entry monomorphic Boolean semantic sub-slice.
  `AUDIT-030` is a preregistered read-only ranking of all eleven residuals and cannot
  change implementation or capability classification.
- `AUDIT-030` is complete at public-green authorization `d4e3c75`. All three
  rankings place R-009 parser UTF-16 projection in their top three; two rank it
  first. `CORE-024` preregisters only an LSP coordinate adapter with one synthetic
  UTF-16-unit end range. It changes no grammar, parser, AST, recovery, semantic, IR,
  codegen, ABI, execution, or backend stage, and adds no matrix promotion before
  tests-first and accepted public evidence.
- Triple-reviewed tests-only `ab8508e` reproduces the selected parser-coordinate
  defect as the sole 148/149 failure across both compiler jobs and stable/nightly
  Rust. Exact triple-reviewed one-file implementation `a3d110e`, tree `79ccfca1`,
  diff `74bfbcea`, passes the focused regression, all LSP tests, the full local gate,
  and all eight public checks. Parser diagnostic start coordinates after non-BMP
  prefixes now project from scalar source columns to UTF-16 at the LSP boundary;
  internal locations, lexical diagnostics, the synthetic one-unit end range, and
  every parse/semantic/IR/backend stage remain unchanged. Diagnostics/source spans
  stays PARTIAL and LSP stays EXPERIMENTAL; this is not token/AST span, recovery, or
  end-to-end range evidence.
- Corrected exact record closure `226b7fb`, tree `1337945c`, diff `861b5ec3`, is
  triple-reviewed and all-eight public green in compiler `30854853182` /
  `30854856449`, Rust `30854856190`, CodeQL `30854853829`, and aggregate
  `91823492290`. Diagnostics/source spans remain PARTIAL and LSP remains
  EXPERIMENTAL. Preregistered read-only `AUDIT-031` may re-rank residual risks but
  cannot change a matrix cell, capability class, or implementation.
- Public-green read-only `AUDIT-031` authorization `ba258c6` selects a distinct
  R-002 containment for `CORE-025`: initialized exact outer tuple binding annotations
  currently disappear at semantic and checked-admission boundaries, allowing the
  scalar RHS type to win. The task may add rejection only, after child validation
  and before generation. Tuple remains PARSED_ONLY; no tuple value, layout, ABI,
  lowering, execution, ownership, generic-type, nested-annotation, or matrix
  promotion is authorized before separate accepted evidence.
- Accepted `CORE-025` implementation `1ec8beb`, tree `ac2c8fdd`, supplies that
  bounded evidence. Corrected tests-only `39ccd9c` publicly reproduces exactly 16
  passed/1 failed in compiler and nightly Rust, with only the selected five-boundary
  target red; the two-file implementation passes focused 1/1, binding 17/17, the exact
  full gate, compiler `30857775577` / `30857777431`, stable/nightly Rust
  `30857777314`, CodeQL `30857775231`, and aggregate `91832840108`. Semantic and
  checked-admission guards now reject only initialized exact outer tuple binding
  annotations after child validation and before insertion/generation. Tuple remains
  PARSED_ONLY; no matrix cell or tuple value/layout/ABI/ownership/lowering/execution
  capability is promoted, and uninitialized/nested annotations remain quarantined.
- Corrected exact `CORE-025` record closure `b0fe242`, tree `2a5d233f`, diff
  `98916b4d`, is triple-approved and all-eight public green in compiler
  `30858384541` / `30858387195`, Rust `30858387193`, CodeQL `30858385234`, and
  aggregate `91834740790`. Tuple remains PARSED_ONLY and no matrix cell changes.
  Preregistered read-only `AUDIT-032` may re-rank all eleven residual risks only
  after its own exact gates; it cannot change a matrix cell, capability class, or
  implementation.
- Public-green read-only `AUDIT-032` authorization `b6b1c63` identifies a bounded
  R-005 checked-admission phase-order defect: wrong-arity direct checked-AST calls
  to known admitted scalar top-level helpers reach raw IR and fail only in verifier
  `CallArity`. `CORE-026` preregisters tests-first rejection before generation for
  only nongeneric, non-entry scalar/Void signatures, with existing child, local-
  callable, and Void-use precedence preserved. No matrix cell changes before
  accepted implementation evidence; source semantics, valid lowering, verifier,
  codegen, ABI, backend, and every broader callable/type surface remain unchanged.
- The first `CORE-026` authorization review rejected ambiguous malformed/duplicate
  signature eligibility at P2 before publication. The corrected boundary admits an
  arity guard only for one verifier-valid, unique, non-reserved top-level declaration
  and requires controls preserving current verifier failures for every excluded
  signature. This correction changes no matrix cell or implementation.
- Accepted `CORE-026` implementation `8c2b2ec`, tree `eabd8939`, supplies only that
  bounded fail-before-IR evidence. Corrected tests-only `1538a3e` publicly reproduces
  exactly 6 passed/1 failed with only the selected phase-order target red; the one-
  file implementation passes focused 1/1, checked-IR 7/7, the exact full gate, both
  compiler jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate.
  Eligible known scalar/Void direct checked-AST arity mismatches now reject during
  Admission after child/local/Void precedence and before raw IR. Valid lowering,
  malformed or duplicate signature behavior, source semantics, verifier defense,
  codegen, ABI, and backends remain unchanged. No matrix cell or capability class is
  promoted, and broader R-005 remains PARTIALLY CONTROLLED.
- Corrected exact `CORE-026` record closure `0a940ea`, tree `6ec4c609`, diff
  `4e1db178`, is triple-approved and all-eight public green in compiler
  `30862783787` / `30862786131`, Rust `30862786150`, CodeQL `30862784231`, and
  aggregate `91848258218`. No matrix cell changes. Preregistered read-only
  `AUDIT-033` may re-rank all eleven residual risks only after its own exact gates;
  it cannot change a matrix cell, capability class, or implementation.
- Public-green read-only `AUDIT-033` authorization `544b1ba` selects only R-010
  documentation-authority containment for `CORE-027`: the split grammar and core-
  features tutorial must visibly distinguish the normative v1 design target from
  current compiler capability evidence. Every EBNF production, example, compiler
  behavior, and existing matrix cell remains unchanged. R-010 remains OPEN and no
  capability is promoted before separate accepted evidence.
- Accepted `CORE-027` implementation `b3e7910`, tree `2728bbc6`, supplies only that
  classification boundary. Tests-first `f57cf2e` publicly isolates the one expected
  authority-contract failure; the corrected two-document implementation passes the
  focused and full version-claim contracts, exact full local gate, both compiler
  jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate. Every EBNF
  production and tutorial code example is unchanged, the grammar remains normative
  intended v1 design, and no parser, semantic, IR, verifier, codegen, ABI, backend,
  row, cell, or capability class changes. R-010 remains HIGH/HIGH and OPEN.
- Exact `CORE-027` record closure `d649c2d`, tree `b5ad7ee2`, diff `d4281863`, is
  triple-approved and all-eight public green in compiler `30865772404` /
  `30865775196`, Rust `30865775214`, CodeQL `30865772793`, and aggregate
  `91857289172`. No matrix cell changes. Preregistered read-only `AUDIT-034` may
  re-rank all eleven residual risks only after its own exact gates; it cannot change
  a matrix cell, capability class, or implementation.
- At `6ce85922`, trusted library/build/check/run/test/profile parser paths reject
  malformed root and applicable direct-module sources with located errors. Lexer
  failures remain uncontrolled, and shared compiler truth remains partial.
- At `b988318`, trusted repository source paths use one strict fallible scanner;
  located lexical failures are covered across library/CLI/modules/docs/profile/LSP,
  while the source-compatible recovery API remains restricted to editor indexing,
  tests, benchmarks, and manual compatibility tooling.
- At `8d5d8e7`, declaration collection, exact numeric arity/type checks, conservative
  numeric return checking, void-value rejection, and matching call result types are
  covered by 13 focused tests plus the full gate and dual independent review. This
  does not certify booleans, generics, composites, or all CFG shapes.
- At `bc9a148`, exact initialized numeric annotations and the binding visibility
  needed to enforce them are covered by 18 focused tests, the full gate, and dual
  independent review. This does not add typed local slots, conversions,
  reassignment, definite initialization, or non-numeric annotation support.
- `AUDIT-021` at clean public head `1535ce2` proves the remaining initialized
  binding surface is not merely uncertified: String/bool/custom-name/fixed-array
  type and fixed-array length mismatches pass check/build and publish LLVM because
  semantics and checked IR discard the annotation. Mixed arrays and non-int indexes
  fail only after semantic success. `CORE-015` preserves existing numeric scalar
  enforcement and, outside active semantic generic scopes, selects `bool`, canonical
  `String`, and nonempty flat fixed numeric arrays. It closes four reproduced
  annotation false successes, adds numeric all-element/count/index typing, and verifies optional
  binary-type metadata in checked IR after semantic operand inference remains
  unchanged. Lowercase `string`, contextual/structural annotations, nonnumeric arrays,
  and new generic-scope annotation/array behavior retain pre-task outcomes under green
  controls. No recursive mapping, conversion,
  representation, layout, or execution change is selected.
- At `c000d91`, `%` is specified, lexed, parsed, and numerically typed but absent
  from IR/backend lowering. Integer, float, mixed, and zero-RHS forms pass semantic
  `check` then panic in IR. `CORE-005` deliberately preregisters a temporary
  fail-closed semantic diagnostic rather than inventing remainder semantics.
- At `302211e`, the preregistered `%` boundary is controlled: syntax and precedence
  are preserved, while shared semantics rejects typed operands before IR across
  public, CLI, and direct-module paths. Fourteen focused tests, the complete gate,
  corrected tutorial wording, and two independent reviews support this partial
  classification; no remainder execution behavior is claimed.
- At exact integrated `CORE-009` production candidate `a887931`, named/generic struct declarations and
  StructLiteral parser shape remain visible, but trusted parsed source bodies visit
  field values in source order and reject construction before inference/IR with
  `Struct construction expressions are not supported.` Construction name/field/type
  validation, layout, initialization, ownership, ABI, lowering, and execution remain
  absent. Historical struct code-generator helpers do not make this source path
  executable.
- The accepted `CORE-010` production implementation adds checked logical scalar IR,
  internal invariant verification, exhaustive checked codegen, and qualified LLVM
  22 verification on trusted publication paths. Focused contracts, the complete
  repository gate, three exact-diff reviews, and all required public CI checks pass
  at head `db349ef`. This does not certify unresolved physical integer, aggregate,
  ownership, or backend semantics.
- Accepted public `CORE-013` at `a78dd00` classifies the two
  `performance_benchmark.py` compilation series as
  invalid measurement evidence because they invoked a bare source path, while the
  public and historical Criterion lexer records retain their separate qualifications
  and the one-run external llama.cpp observation remains reference-only.

This file must be tightened as audit items close. A row may become `END_TO_END`
only with source-to-execution evidence and all applicable positive, negative,
diagnostic, documentation, and backend gates. `STABLE` additionally requires a
declared compatibility policy and release-level coverage.

## AUDIT-034 / CORE-028 classification boundary

- Public-green read-only `AUDIT-034` authorization `45783af`, tree `f1baa457`,
  passes both compiler jobs, stable/nightly Rust, all three CodeQL analyses, and the
  aggregate check. Three complete independent rankings and unanimous targeted
  reconciliation select one exact R-002 fail-open declaration form.
- A binding with `value: None` and outer annotation `Type::Tuple(_)` is not an
  implemented tuple feature. At the pre-CORE-028 audit basis, semantics silently
  selected `Ty::Int`, checked admission skipped the statement, and raw generation
  could create integer zero. `CORE-028` therefore selected rejection in semantics
  and checked admission only, before insertion or generation, with existing
  duplicate-name semantics first.
- This containment cannot change a matrix cell: it adds no tuple value, layout,
  assignment, ownership, ABI, lowering, execution, or backend evidence. Initialized
  CORE-025 behavior, nested tuple shapes, other valueless annotations, valid IR/LLVM,
  and every current capability class remain unchanged. R-002 remains HIGH/CRITICAL
  and PARTIALLY CONTROLLED.
- Accepted public `CORE-028` implementation `e051452`, tree `63985b2d`, supplies
  only that rejection boundary after triple-reviewed public red evidence. Focused
  1/1, binding 17/17, the exact full local gate, both compiler jobs, stable/nightly
  Rust, all three CodeQL analyses, and aggregate pass. Exact outer tuple annotations
  on valueless bindings no longer fall back to `Int` at the two trusted boundaries.
  No tuple value/layout/lowering/execution evidence was added, so every matrix row,
  cell, and capability class remains unchanged.
- Exact six-record CORE-028 closure `032d0d0`, tree `443aacdc`, diff `93fce8ae`, is
  triple-approved and all-eight public green in compiler `30872236535` /
  `30872238993`, Rust `30872239003`, CodeQL `30872237025`, and aggregate
  `91876507154`. No matrix cell changes.
- Preregistered read-only `AUDIT-035` may re-rank the same complete eleven-risk set
  only after its separate exact authorization gates. It must exclude every accepted
  slice including CORE-028, cannot inherit AUDIT-034's order, and cannot change a
  matrix row, capability class, source, test, workflow, dependency, or backend.

## AUDIT-035 / CORE-029 classification boundary

- Triple-approved read-only AUDIT-035 authorization `f1cd972`, tree `b9c6270b`,
  passes both compiler jobs, stable/nightly Rust, all three CodeQL analyses, and the
  aggregate check. Three independent complete rankings and unanimous targeted
  reconciliation select one distinct R-002 fail-open annotation shape.
- A valueless binding with outer `Type::Reference(inner, _)` and immediate
  `inner: Type::Tuple(_)` is not implemented reference or tuple behavior. At the
  pre-CORE-029 audit basis it became `Ty::Int`, was skipped by checked admission,
  and could become `ImmInt(0)` in raw generation. CORE-029 therefore selected
  rejection at semantics and checked admission only, before insertion/generation
  and with duplicate semantics first.
- This containment cannot change a matrix cell: it adds no tuple/reference value,
  initialization, assignment, representation, mutability, borrowing, ownership,
  lifetime, provenance, layout, ABI, lowering, execution, or backend evidence. Outer
  tuple CORE-028, initialized bindings, non-tuple references, deeper nesting, valid
  IR/LLVM, and every capability class remain unchanged. R-002 stays HIGH/CRITICAL
  and PARTIALLY CONTROLLED.
- Accepted public `CORE-029` implementation `29bd2e0`, tree `53282149`, supplies
  only that exact non-recursive rejection after triple-reviewed public red evidence.
  Focused 1/1, binding 18/18, formatting, the exact full local gate, both compiler
  jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate pass.
  Valueless immediate reference-to-tuple annotations no longer fall back to `Int`
  at the two trusted boundaries. No tuple/reference value, representation, ownership,
  lowering, execution, or backend evidence was added, so every matrix row, cell, and
  capability class remains unchanged.
- Exact six-record CORE-029 closure `7222b9a`, tree `66084b36`, diff `90bf540c`, is
  triple-approved and all-eight public green in compiler `30876033717` /
  `30876035730`, Rust `30876035761`, CodeQL `30876034500`, and aggregate
  `91887644623`. No matrix cell changes.
- Preregistered read-only `AUDIT-036` may re-rank the same complete eleven-risk set
  only after its separate exact authorization gates. It excludes every accepted
  slice including CORE-029, cannot inherit AUDIT-035's order, and cannot change a
  matrix row, capability class, source, test, workflow, dependency, or backend.
- Corrected read-only `AUDIT-036` authorization `f4ac505`, tree `3cdf89e6`, diff
  `40896f51`, is triple-approved and all-eight public green in compiler
  `30876975678` / `30876977928`, Rust `30876977905`, CodeQL `30876976155`, and
  aggregate `91890402326`. All three complete rankings select exact R-002 valueless
  immediate array-of-tuple fallback over verifier-contained R-005.
- Accepted public CORE-030 implementation `97c0f04`, tree `aa3a9e3f`, diff
  `06a104df`, turns only that one unsupported valueless annotation into semantic and
  checked-admission rejection after triple-reviewed authorization and public
  tests-first red evidence. Focused 1/1, binding 19/19, the exact full local gate,
  both compiler jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate
  pass. Rejection supplies no array/tuple value, default, bounds, layout, mutation,
  ABI, ownership, lowering, execution, or backend evidence. Therefore every matrix
  row, cell, and capability class remains unchanged, and R-002 remains
  HIGH/CRITICAL and PARTIALLY CONTROLLED.
- Exact six-record CORE-030 closure `cd8add28`, tree `8ab06d62`, diff `18ffa30d`,
  is triple-approved and public all-eight green in compiler `30879329940` /
  `30879332975`, Rust `30879332995` attempt 2, CodeQL `30879330627`, and
  aggregate `91897195358`. The initial Rust fixture race passed on focused rerun
  without a file or ref change. No matrix cell changes.
- Preregistered read-only AUDIT-037 may re-rank the complete remaining eleven-risk
  set from that exact clean public head only after its separate authorization gates.
  It excludes all accepted slices through CORE-030, inherits no prior order, and
  cannot change a matrix row, cell, capability class, source, test, workflow,
  dependency, backend, semantics, or claim.
- Triple-approved read-only AUDIT-037 authorization `987188fc`, tree `0b685659`,
  is public all-eight green. Three complete rankings place R-002 first; targeted
  static reconciliation unanimously selects only the exact valueless
  `Array(Array(Tuple))` fallback over the reference-array alternative.
- Preregistered CORE-031 may turn only that unsupported exact two-array-deep
  valueless annotation into semantic and checked-admission rejection after separate
  contract and public tests-first gates. Rejection adds no nested-array/tuple value,
  default, bounds, layout, mutation, ABI, ownership, lowering, execution, or backend
  evidence. Every matrix row, cell, and capability class remains unchanged; R-002
  remains HIGH/CRITICAL and PARTIALLY CONTROLLED.
- Accepted public CORE-031 implementation `4bc7a345`, tree `61361621`, canonical
  diff `349e34ee`, turns only that exact unsupported form into semantic and checked-
  admission rejection after triple-reviewed authorization and public expected-red
  evidence. Focused 1/1, binding 20/20, the exact full local gate, both compiler
  jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate pass.
  Candidate B, initialized and third-plus-depth forms, scalar arrays, generic and
  reference wrappers, raw IR, verifier/codegen, ABI/ownership, valid-output scope,
  and every backend remain unchanged. Rejection supplies no nested-array/tuple
  value, default, bounds, layout, mutation, lowering, execution, or backend evidence;
  therefore every matrix row, cell, and capability class remains unchanged, and
  R-002 remains HIGH/CRITICAL and PARTIALLY CONTROLLED.
- Exact six-record CORE-031 closure `45696091`, tree `480c3504`, canonical diff
  `d682b0f6`, is triple-approved and public all-eight green in compiler
  `30882630407` / `30882632698`, Rust `30882632696`, CodeQL `30882630822`, and
  aggregate `91907149874`. No matrix cell changes.
- Preregistered read-only AUDIT-038 may re-rank the complete remaining eleven-risk
  set only after its separate exact authorization gates. It must exclude every
  accepted slice through CORE-031, inherit neither Candidate B nor any prior order,
  and cannot change a matrix row, cell, capability class, source, test, workflow,
  dependency, backend, semantics, or claim.
- Corrected read-only AUDIT-038 authorization `e4d58e59`, tree `f265d8af`, canonical
  diff `31d09f92`, is triple-approved and public all-eight green. Three complete
  rankings put R-002 first; after an exact-candidate split, final compatibility
  reconciliation unanimously approves only initialized immediate `Array(Tuple)`
  containment. The valueless triple-array candidate remains preserved.
- Preregistered CORE-032 may turn only that unsupported initialized immediate
  array-of-tuple annotation into semantic and checked-admission rejection after
  separate contract and public tests-first gates, in every generic/impl statement
  context those phases already traverse while preserving earlier outer-generic
  rejection. The initial five-acceptance authorization snapshot was rejected before
  publication for omitting that context; the corrected contract freezes eight
  accepts. Rejection adds no tuple/array
  compatibility, value, default, bounds, layout, mutation, ABI, ownership, lowering,
  execution, or backend evidence. Every matrix row, cell, and capability class
  remains unchanged; R-002 remains HIGH/CRITICAL and PARTIALLY CONTROLLED.
- Corrected CORE-032 authorization `449f3536` is triple-approved and public all-eight
  green. Corrected tests-first `35eac8c4` publicly proves exactly eight acceptances
  and only the named 20/21 regression after rejected unpublished `1afe11d3` omitted
  explicit array-literal target coverage.
- Accepted public implementation `30d0d730`, tree `653346ce`, canonical diff
  `01e87768`, adds only exact semantic and checked-admission rejection. Focused 1/1,
  binding 21/21, formatting, two consecutive exact full gates, all three reviews,
  compiler `30886856260` / `30886858878`, Rust `30886858960`, CodeQL
  `30886856518`, and aggregate `91919998289` pass. The first full-gate attempt is
  retained as an unexplained truncated exit-1 result. No tuple/array value,
  compatibility, layout, lowering, execution, backend, matrix-cell, or capability-
  class evidence was added; R-002 remains HIGH/CRITICAL and PARTIALLY CONTROLLED.
- First closure snapshot `7d7fe3d6` passed its exact gate but was rejected
  unpublished because its state record treated that gate as future work and generic
  nonzero wording lost the known exit 1 above. Second snapshot `48f2fd60`, tree
  `86175cc1`, canonical diff `9f0ab102`, resolved those findings and received two
  approvals but was rejected unpublished at P3 by the type reviewer because the
  successful closure gate lacked literal `exit 0`. The twice-corrected records
  preserve both rounds; their fresh exact gate exits 0 with 139/139 library, 149/149
  binary, 7/7 doc, and 21/21 binding tests. Exact closure `9c82cbfc`, tree
  `b2a106ee`, canonical diff `fc672744`, is triple-approved and public all-eight
  green in compiler `30888222316` / `30888225734`, Rust `30888226011`, CodeQL
  `30888222480`, and aggregate `91924197947`. No matrix cell moves.
- Preregistered read-only AUDIT-039 may re-rank only the complete remaining
  R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016 set from exact
  clean public closure `9c82cbfc`. It must exclude all accepted slices through
  CORE-032, inherit no prior candidate/order, and distinguish rejection, simulation,
  annotation, LLVM text, object emission, and hardware execution.
- Exact AUDIT-039 authorization `fa522b2c`, tree `365a536d`, canonical diff
  `cefb797e`, is triple-approved and public all-eight green. All rankings put R-002
  first. Initial candidates split between valueless three-array Candidate T and
  initialized two-array Candidate A; targeted preference favored A two to one, and
  final compatibility reconciliation unanimously approved exact A.
- Preregistered CORE-033 may reject only initialized exact nonrecursive
  `Array(Array(Tuple))` after initializer and existing initialized diagnostics at
  semantic and checked boundaries. Its tests-first red surface is exactly 12
  acceptances after explicit reclassification of both existing A acceptance rows.
  Candidate T, reference-array Candidate B, other deeper/wrapped shapes, and every
  tuple/array value, layout, bounds, ownership, ABI, lowering, execution, backend,
  matrix, risk, and capability state remain unchanged.
- The prepared CORE-033 six-record authorization's fresh exact full gate exits 0
  with 139/139 library, 149/149 binary, 7/7 doc, and 21/21 binding tests. At that
  authorization stage, no test or source change was permitted before three exact
  reviews, unchanged publication, and all eight public checks.
- First authorization snapshot `d0500865`, tree `d2378320`, canonical diff
  `97a15c9f`, passed its local gate but was rejected unpublished by two reviewers
  because one ledger sentence mislabeled Candidate T as Candidate B. The correction
  changes no matrix, capability, risk, or behavior boundary.
- Corrected CORE-033 authorization `66207215`, tree `357c2731`, canonical diff
  `96b5f403`, is triple-approved and public all-eight green. First tests snapshot
  `7608b42c` was rejected unpublished for omitting the initialized three-array-deep
  green control. Corrected tests-only `ac4cb2a5`, tree `852bff0b`, canonical diff
  `4ca50572`, publicly proves exactly 12 acceptances as the sole 21/22 failure in
  compiler `30891243037` / `30891246443` and nightly Rust `30891247469`; CodeQL
  `30891241566` and aggregate `91933672071` pass.
- Accepted implementation `76a6e802`, tree `d8391348`, established PowerShell
  full-index canonical diff `a75b59b2`, adds only the exact semantic and checked-
  admission rejection. Formatting, focused 1/1, binding 22/22, the exact full local
  gate exit 0, three corrected-identity approvals, compiler `30891890629` /
  `30891898590`, Rust `30891897083`, CodeQL `30891892219`, and aggregate
  `91935804190` pass. The initial review request's erroneous plain-diff `c17b1b6a`
  changed no commit or tree.
- Rejection supplies no tuple/nested-array value, compatibility, bounds, layout,
  mutation, ABI, ownership, lowering, execution, or backend evidence. Candidate T,
  reference-array Candidate B, all other deeper/wrapped forms, every matrix row,
  cell, and capability class remain unchanged; R-002 stays HIGH/CRITICAL and
  PARTIALLY CONTROLLED.
- First six-record closure snapshot `fe90f583`, tree `90ac8ae6`, canonical diff
  `89fe6824`, changed only the control records and passed its exact gate with
  139/139 library, 149/149 binary, 7/7 claim, and 22/22 binding tests. It received
  two approvals but was rejected before independent push or branch-head publication
  because stale PROJECT_STATE wording could reopen tests-first and implementation.
  First correction `19f688a`, tree `9d9c642f`, canonical diff `f885588c`, fixed the
  wording, passed the same gate, received three approvals, and is public all-eight
  green in compiler `30893002336` / `30893005706`, Rust `30893006634`, CodeQL
  `30893002479`, and aggregate `91939375982`. Its linear push also made rejected
  parent `fe90f583` publicly reachable as an ancestor, invalidating the stronger
  never-published wording. Final additive correction changes no matrix cell; exact
  gate exits 0 with 139/139 library, 149/149 binary, 7/7 claim, and 22/22 binding
  tests. Exact correction `1ee9c71`, tree `d0819881`, canonical diff `7303da47`,
  received three approvals, was published unchanged, and passes compiler
  `30893527220` / `30893529999`, stable/nightly Rust `30893529992`, all three
  CodeQL analyses in `30893527445`, and aggregate `91941079083`. No matrix cell
  moves.
- Preregistered read-only AUDIT-040 required re-ranking only the complete remaining
  R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016 set from exact
  clean public closure `1ee9c71`. It must exclude all accepted slices through
  CORE-033, inherit no prior candidate/label/order, and distinguish rejection,
  simulation, annotation, LLVM text, object emission, and hardware execution.
- First authorization snapshot `c83ec3a`, tree `bb25e528`, canonical diff
  `c02f71e5`, passed its exact full gate with 139/139 library, 149/149 binary, 7/7
  claim, and 22/22 binding tests. Type/safety and backend/claim approved, but IR/
  codegen rejected at P1 because a late PROJECT_STATE subsection still treated
  accepted CORE-033 closure as future work. It was rejected before publication.
  Corrected authorization `7b9ed83`, tree `8dbe975e`, canonical diff `c4ba110a`,
  passed its fresh exact gate with 139/139 library, 149/149 binary, 7/7 claim, and
  22/22 binding tests, received three exact approvals, and was published unchanged.
  Compiler `30894708169` / `30894713332`, stable/nightly Rust `30894713411`, all
  three CodeQL analyses in `30894708736`, and aggregate `91944883143` pass.
- AUDIT-040 completed read-only. Type/safety selected valueless exact three-array
  tuple containment; IR/codegen selected initialized exact immediate reference-to-
  tuple containment; backend/claim selected immediate nonnegative literal fixed-
  array bounds containment. Targeted comparison preferred reference containment two
  to one, and all three final compatibility reviews approved that exact predicate.
  Literal bounds remains stopped pending separately frozen compile-time-versus-
  runtime policy; the three-array candidate remains a bounded fallback with greater
  topology and count burden. No matrix row, cell, capability class, or risk moved.
- Preregistered CORE-034 may reject only initialized exact nonrecursive
  `Type::Reference(Type::Tuple(_), _)` in semantic analysis and checked admission,
  after initializer validation and all existing initialized tuple-shape diagnostics,
  for both reference mutability flags. Only after the six-record authorization is
  locally green, triple-approved, published unchanged, and public all-eight green
  may one tests-first aggregate reclassify two existing acceptance rows and expose
  exactly 30 false acceptances. Implementation requires separately reviewed public-
  red evidence and remains limited to the semantic analyzer and checked IR admission.
  First authorization snapshot `7d4d7ca`, tree `b633abbb`, canonical diff
  `a901f4dc`, passed its exact full gate with 139/139 library, 149/149 binary, 7/7
  claim, and 22/22 binding tests. IR/codegen and backend/claim approved, but type/
  safety rejected it at P1 because TASK_LEDGER's final status still called the
  completed gate future work. It remained unpublished. The corrected authorization's
  fresh exact full gate exits 0 with 139/139 library, 149/149 binary, 7/7 claim, and
  22/22 binding tests. Rejection adds no reference or tuple value, mutability,
  ownership, lifetime, layout, ABI, coercion, lowering, execution,
  bounds, backend, or stability evidence. Every matrix row and cell remains exactly
  unchanged; R-002 stays HIGH/CRITICAL and PARTIALLY CONTROLLED.
- Corrected authorization `91d2686` is triple-approved and public all-eight green.
  Triple-approved tests-only `296276f` publicly proves exactly 30 false acceptances
  as the sole 22/23 binding failure in compiler `30916807388` / `30916811627` and
  nightly Rust `30916810937`; CodeQL `30916806193` passes. Three public-red reviews
  approved implementation authority.
- Exact two-phase implementation `a1ffeaec`, tree `f0088e65`, canonical diff
  `7a3fdb11`, adds only nonrecursive semantic and checked-admission rejection. It is
  triple-approved and passes the exact full local gate, compiler `30917539648` /
  `30917544307`, stable/nightly Rust `30917537292`, all three CodeQL analyses in
  `30917534448`, and aggregate `92019545168`. No matrix row or cell moves: tuple,
  reference, ownership, layout, ABI, lowering, execution, bounds, and backend cells
  retain their prior classifications.
- The prepared six-record closure's fresh exact full gate exits 0 with 139/139
  library, 149/149 binary, 7/7 claim, and 23/23 binding tests. No matrix cell moves.
- Exact six-record closure `d3811b00`, tree `c01088c4`, canonical diff `2799eb32`,
  is triple-approved and public all-eight green in compiler `30918433816` /
  `30918438945`, stable/nightly Rust `30918439169`, all three CodeQL analyses in
  `30918434204`, and aggregate `92022619964`. CORE-034 is closed; no matrix cell
  moves.
- Preregistered read-only AUDIT-041 must independently re-rank the complete remaining
  R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016 set from exact
  clean public closure `d3811b00`, exclude every accepted slice through CORE-034,
  inherit no prior candidate/label/order, and distinguish rejection, helper
  simulation, annotations, LLVM text, object emission, and hardware execution.
  It cannot move a matrix row or cell; ranking begins only after its separate
  six-record authorization is locally green, triple-approved, published unchanged,
  and public all-eight green.
- The prepared AUDIT-041 authorization's fresh exact full gate exits 0 with 139/139
  library, 149/149 binary, 7/7 claim, and 23/23 binding tests. No matrix cell moves.
- Exact AUDIT-041 authorization `a31342e8`, tree `fbcd78b6`, canonical diff
  `313a1f6b`, is triple-approved and public all-eight green. Three complete rankings
  put R-002 first; initial V/I/R candidates split, targeted comparison prefers R two
  to one, and all final compatibility reviews approve only initialized exact
  nonrecursive positive-count `Reference(Array(Tuple))` containment.
- Accepted CORE-035 authorization preregistered rejection of that exact shape for
  both reference mutability flags at semantic and checked-admission boundaries only
  after child and existing initialized diagnostics. Its tests-first evidence was
  required to expose exactly 34 false
  acceptances and preserve four count-zero observations. Rejection defines no
  reference/array/tuple value, ownership, layout, ABI, bounds, lowering, execution,
  backend, or stability evidence. Every matrix row and cell remains unchanged;
  R-002 stays HIGH/CRITICAL and PARTIALLY CONTROLLED.
- The prepared CORE-035 authorization's fresh exact full gate exits 0 with 139/139
  library, 149/149 binary, 7/7 claim, and 23/23 binding tests. No matrix cell moves.
- Exact authorization `b74b1d29`, tree `3fc2d78f`, canonical diff `64fbd1fe`, is
  triple-approved and public all-eight green. Triple-approved tests-only `f04e80c9`,
  tree `03a9f274`, canonical diff `9e04b6ad`, publicly proves exactly 34 false
  acceptances as the sole 23/24 binding failure in compiler `30922180824` /
  `30922181281` and nightly job `92035312036` in Rust `30922181764`; stable was
  fail-fast cancelled, while CodeQL `30922176056` and aggregate `92035461619` pass.
  Three public-red reviews approved implementation authority.
- Exact implementation `b8fd5a17`, tree `77bd2536`, canonical diff `2f1e9920`, adds
  only nonrecursive semantic and checked-admission rejection. It is triple-approved;
  focused 1/1, binding 24/24, the exact full local gate, compiler `30922853658` /
  `30922859177`, stable/nightly Rust `30922863203`, all three CodeQL analyses in
  `30922853619`, and aggregate `92037794056` pass.
- Rejection supplies no reference/array/tuple value, compatibility, ownership,
  lifetime, bounds, layout, ABI, lowering, execution, or backend evidence. Count
  zero and every deeper/wrapped residual remain unimplemented controls. Therefore
  every matrix row, cell, and capability class remains unchanged; R-002 stays
  HIGH/CRITICAL and PARTIALLY CONTROLLED. The six-record closure's exact full local
  gate passes with 139/139 library, 149/149 binary, 7/7 claim, and 24/24 binding
  tests.
- Exact CORE-035 closure `60ad91f7`, tree `978aa98f`, canonical diff `818a8112`, is
  triple-approved and public all-eight green in compiler `30923835957` /
  `30923837627`, stable/nightly Rust `30923838264`, all three CodeQL analyses in
  `30923834264`, and aggregate `92041128413`. CORE-035 is closed; no matrix row,
  cell, capability, risk, backend, artifact, or claim classification moves.
- Preregistered read-only AUDIT-042 must independently re-rank only the complete
  remaining R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016
  set from exact clean public closure `60ad91f7`. It must exclude all accepted slices
  through CORE-035, inherit no prior candidate, label, or order, and distinguish
  rejection, helper simulation, annotations, LLVM text, object emission, and
  hardware execution. It cannot change a matrix row or cell; ranking begins only
  after its six-record authorization is locally green, triple-approved, published
  unchanged, and public all-eight green.
- The prepared AUDIT-042 authorization's fresh exact full gate exits 0 with 139/139
  library, 149/149 binary, 7/7 claim, and 24/24 binding tests. No matrix cell moves.
- First authorization snapshot `4ce0de0d`, tree `350984b8`, canonical diff
  `347278c3`, passed that gate but was rejected before independent push or branch-
  head publication for stale active-hypothesis and closure-status wording. It remains
  in corrected ancestry; ranking did not begin and no matrix cell moves.
- Corrected AUDIT-042 authorization `2d8a0c54`, tree `45d1c184`, correction canonical
  diff `b36d3d9b`, and cumulative canonical diff `478e947a`, is triple-approved and
  public all-eight green in compiler `30924946683` / `30924950615`, stable/nightly
  Rust `30924951134`, all three CodeQL analyses in `30924945035`, and aggregate
  `92044919183`.
- Three complete rankings selected U/T/B respectively. Targeted comparison selected
  valueless exact nonrecursive `Reference(Array(Tuple))` U two to one; all three
  final compatibility reviews approved only that exact two-phase containment. Direct
  literal bounds B remains stopped pending compile-time-versus-runtime policy;
  valueless exact three-array tuple T remains a bounded fallback. AUDIT-042 was read-
  only and moves no matrix row or cell.
- Preregistered CORE-036 may reject only a valueless exact nonrecursive
  `Type::Reference(Type::Array(Type::Tuple(_), count), ref_flag)` for both flags and
  all counts at semantic and checked-admission boundaries, after existing duplicate/
  tuple-shape diagnostics and before fallback/raw generation. Its tests-first file
  must reclassify all four existing acceptance occurrence blocks/five exact source
  rows, expose exactly 34 false acceptances, and preserve exactly 40 observations.
  Implementation remains two exact guards after separately reviewed public-red
  evidence.
- That proposed rejection supplies no reference, array, tuple, default, mutability,
  ownership, lifetime, layout, ABI, bounds, lowering, execution, or backend support.
  Initialized count-zero behavior remains unchanged. Every matrix row and cell,
  including tuples parsed-only, references/fixed arrays partial, bounds unresolved,
  and CPU/ROCm/CUDA separated, remains exactly unchanged. Authorization had to be
  triple-approved, published unchanged, and public all-eight green before tests-first
  work. Its fresh and verification gates passed with 139/139 library, 149/149 binary,
  7/7 claim, and 24/24 binding tests; `697bb3b4` satisfies the review/publication
  prerequisites below.
- CORE-036 authorization `697bb3b4`, tree `b0cfd37b`, canonical binary diff
  `0a92ad7a`, is triple-approved and public all-eight green in compiler
  `30927281281` / `30927293459`, Rust `30927289178`, CodeQL `30927280707`, and
  aggregate `92052974430`.
- Triple-approved tests-only `d52b117e`, tree `76a3b2e9`, canonical binary diff
  `c2d5e46a`, publicly proves exactly 34 false acceptances as the sole binding 24/25
  failure in push `30927952017`, PR `30927956714`, nightly `92055067840`, and the
  stable `92055068009` test step. CodeQL `30927952240` and aggregate `92055178151`
  pass; three public-red reviews approved implementation.
- Exact implementation `26d18924`, tree `8aec746c`, canonical binary diff
  `543f8a1c`, adds only nonrecursive semantic and checked-admission rejection. It is
  triple-approved; focused 1/1, binding 25/25, the full local gate, compiler
  `30928759703` / `30928760789`, stable/nightly Rust `30928758562`, CodeQL
  `30928754859`, and aggregate `92057919831` pass.
- Rejection supplies no reference/array/tuple/default/compatibility/ownership/
  lifetime/layout/ABI/bounds/lowering/execution/backend evidence. Every matrix row
  and cell remains unchanged: tuples parsed-only, references/fixed arrays partial,
  bounds unresolved, and CPU/ROCm/CUDA separated. R-002 stays HIGH/CRITICAL and
  PARTIALLY CONTROLLED; R-011 stays open. The additively corrected six-record
  closure's fresh and verification exact full gates each exit 0 with 139/139 library,
  149/149 binary, 7/7 claim, and 25/25 binding tests. Exact acceptance `3f042e18`
  below closes CORE-036 without matrix movement.
- First closure snapshot `39c8564b`, tree `7932dd42`, canonical binary diff
  `2cb44b26`, passed two gates but received two P1 rejections because PROJECT_STATE's
  current implementation pointer remained at CORE-035 `b8fd5a17`. It was not
  independently published. The additive corrected closure names `26d18924`,
  preserves the rejected snapshot in ancestry, and moves no matrix row or cell.
- First additive correction `799c4181`, tree `1c8a883f`, canonical binary diff
  `9a1f5cd8`, received type/safety approval but IR/codegen rejected P1 because DEC-048
  still called the completed verification gate pending. The review round stopped
  before publication. The second additive correction aligns that status and moves no
  matrix row or cell.
- Its fresh exact repository-root full gate exits 0 with 139/139 library, 149/149
  binary, 7/7 claim, and 25/25 binding tests, plus all downstream suites.
- Exact CORE-036 closure `3f042e18`, tree `15d56e0c`, canonical binary diff
  `ee8cbed0`, changed only six records, received three exact approvals, and is public
  all-eight green in compiler `30930377220` / `30930379386`, Rust `30930380195`,
  CodeQL `30930375201`, and aggregate `92063404658`. CORE-036 is closed; every matrix
  row and cell remains unchanged.
- Preregistered AUDIT-043 may independently re-rank only the complete remaining
  eleven-risk set from exact clean public closure `3f042e18`, excluding every
  accepted slice through CORE-036 and inheriting no U/T/B label or order. It remains
  read-only and cannot move a matrix row or cell, capability, risk, backend, artifact,
  or claim. Bounds B remains stopped pending compile-time-versus-runtime policy;
  tuples remain parsed-only, references/fixed arrays partial, bounds unresolved, and
  CPU/ROCm/CUDA separate.
- Pre-acceptance evidence at corrected snapshot `5276df5b` (historical; superseded by
  the result below): the prepared authorization's fresh and verification exact full
  gates each exited 0 with 139/139 library, 149/149 binary, 7/7 claim, and 25/25
  binding tests, plus all downstream suites. At that point exact review and public
  acceptance remained pending; no matrix row or cell moved.
- First authorization snapshot `cb43d1bb`, tree `f0f19f5d`, canonical binary diff
  `ead99a7b`, passed both gates but all three reviewers rejected P1 because DEC-049
  status still called them required; type/safety also found stale “next immutable
  snapshot” wording. Nothing was published and no ranking began. The additive
  correction moves no matrix row or cell.
- Pre-acceptance additive correction evidence (historical; superseded below): its
  fresh exact repository-root full gate exited 0 with 139/139 library, 149/149 binary,
  7/7 claim, and 25/25 binding tests, plus all downstream suites. At that point fresh
  review and public acceptance remained pending; no matrix row or cell moved.
- Corrected AUDIT-043 authorization `5276df5b`, tree `c3eaf3cf`, correction diff
  `b8b7586f`, cumulative diff `fe5376dc`, is triple-approved and public all-eight
  green in compiler `30931510621` / `30931515125`, Rust `30931515426`, CodeQL
  `30931509579`, and aggregate `92067252294`. Complete rankings and final
  compatibility unanimously select conditional exact R-002 after a separate neutral
  classifier; R-009 remains fallback and bounds stopped. No matrix row or cell moves.
- Preregistered ARCH-001 may refactor only exact existing annotation disposition into
  a shared nonrecursive classifier with rejection, contract-shape-routing, and inert
  preserve outcomes. It must keep all phase diagnostics/order/traversal, generic/
  trait gates, raw compatibility, valid LLVM, and the later R-002 shape unchanged.
  Tuples remain parsed-only, references/fixed arrays partial, bounds unresolved, and
  CPU/ROCm/CUDA separate. The refactor is not capability or support evidence.
- Its pre-acceptance fresh and verification exact authorization gates (historical;
  superseded below) each exited 0 with 139/139 library, 149/149 binary, 7/7 claim,
  and 25/25 binding tests, plus all downstream suites. At that point review and later
  acceptance remained pending; no matrix row or cell moved.
- First ARCH-001 snapshot `63d8d599`, tree `28cd120c`, diff `9fef5adf`, was not
  published after a valid P1 chronology rejection. This additive six-record correction
  labels superseded AUDIT-043 pending/no-ranking evidence historical; no matrix row or
  cell moves.
- The additive correction's pre-acceptance fresh and verification exact full gates
  (historical; superseded below) each exited 0 with
  139/139 library, 149/149 binary, 7/7 claim, and 25/25 binding tests, plus all
  downstream suites. At that point fresh review and public acceptance remained
  pending; no matrix row or cell moved.
- Exact `1dcfd869`, tree `b537023c`, correction diff `e5ee8aa7`, cumulative diff
  `5208cb6e`, is triple-approved and public all-eight green in compiler CI
  `30934518525` / `30934523152`, Rust `30934523078`, CodeQL `30934519513`, and
  aggregate `92077350363`. No matrix row or cell moves; separate green
  characterization evidence becomes eligible only after the six-record acceptance
  sync below is accepted public all-eight green.
- The six-record acceptance sync's fresh and verification exact full gates each exit 0
  with 139/139 library, 149/149 binary, 7/7 claim, and 25/25 binding tests, plus all
  downstream suites. Exact review and public sync acceptance remain pending; no matrix
  row or cell moves.
- First acceptance-sync snapshot `4c18450a`, tree `ea7b91c9`, diff `7be565db`, was
  not published after a type/safety P1 found three premature eligibility statements.
  The additive six-record correction restores the sync gate; no matrix row or cell
  moves.
- Its fresh and verification exact full gates each exit 0 with 139/139 library,
  149/149 binary, 7/7 claim, and 25/25 binding tests, plus all downstream suites.
  Fresh review and public acceptance remain pending; no matrix row or cell moves.
- Local pre-publication CORE-040 evidence adds only exact `==` and `!=` for two
  compile-time decoded String values retained through bounded immutable provenance.
  One classifier owns operator eligibility and exact content equality; checked
  admission and lowering produce logical Bool IR without String layout or a runtime
  helper. Ordering, concatenation, dynamic/mutable/signature strings, noncanonical
  annotations, impl/generic/closure/trait/raw paths, normalization, ownership, ABI,
  GPU, and general operator overloading remain excluded. The exact root gate passes
  145/145 library, 155/155 CLI, 7/7 claim, 28/28 binding, the exhaustive equality
  aggregate, all downstream suites, and doc tests. The Strings and comparisons rows
  remain `PARTIAL`; pinned LLVM 22 native exit-41 evidence, exact review, publication,
  and public checks are still pending.
- First candidate `ec6369d` was rejected unpublished because a closure pre-scan could
  mask earlier diagnostics. The additive correction removes that scan, retains one
  shared equality classifier behind child-first context-aware admission, adds both
  precedence intersections to the exhaustive aggregate, and passes the same exact
  145/145 library and 155/155 CLI root gate. No matrix cell or class changes; corrected
  exact review and public native exit-41 evidence remain pending.
- Corrected compiler candidate `ed9ad3e` closed the closure-precedence defect and was
  retained unpublished after record-only P3 findings. The evidence now accurately says
  closure/impl contexts validate children and bypass the static equality classifier,
  and the immutable successor identity is reported externally in present tense. No
  compiler, test, workflow, matrix, capability, or claim boundary changes.
- Exact `edd63f3`, tree `8ef355a6`, is the accepted public CORE-040 successor. Three
  fresh exact reviews and all eight public checks pass; stable Linux job `92134492264`
  ran checked build, pinned LLVM 22 verification, machine verification/object
  lowering, Clang link, and native exit 41. This closes only the enumerated compile-time
  exact-String equality class. Strings and comparisons remain `PARTIAL`; ordering,
  concatenation, runtime/dynamic String operations, layout, ABI, ownership, and GPU
  support remain unchanged.
- Preregistered CORE-041 may add only the complete four-method Boolean-predicate class
  for trusted compile-time decoded Strings: `is_empty`, `contains`, `starts_with`, and
  `ends_with`. One classifier must own method, arity, receiver/argument trust, and exact
  sequence results; checked admission and active checked lowering may consume it to
  produce logical Bool constants. Dynamic/runtime Strings, normalization, patterns,
  noncanonical annotations, collections, impl/generic/closure/trait/raw paths, other
  methods, layout, ABI, ownership, and accelerators remain excluded. No matrix row or
  cell moves at authorization; Strings remain `PARTIAL`, and no capability claim moves
  until the exhaustive aggregate and pinned exit-43 native gate pass.
- Local pre-publication CORE-041 evidence implements that exact four-method class under
  one classifier consumed by checked admission and active checked lowering. The
  expanded exhaustive aggregate, both classifier roots, adjacent CORE-038/039/040 and
  admission/verifier contracts, and exact root gate pass at 146/146 library and
  156/156 CLI tests. The example contains all four predicates and the stable-Linux CI
  route requires pinned LLVM 22 verify/lower/link plus exact native exit 43. Strings
  remain `PARTIAL`; no dynamic/runtime String, general method dispatch, normalization,
  layout, ABI, ownership, accelerator, stability, or performance cell moves. Exact
  review, unchanged publication, all-eight checks, and native Linux evidence remain
  pending.
- Exact `a69b7899`, tree `e3722300`, is the accepted public CORE-041 successor. Three
  fresh exact reviews and all eight public checks pass; stable Linux job `92143515440`
  used LLVM/Clang 22.1.8 for checked build, LLVM and machine verification, object
  lowering, link, and native exit 43. This closes only the four enumerated predicates
  on trusted compile-time Strings. Strings remain `PARTIAL`; runtime/dynamic String,
  normalization, layout, ABI, ownership, accelerator, stability, and performance
  claims remain unchanged.
- Preregistered CORE-042 may certify one complete executable composition class for the
  existing flattened direct-module source collector: both physical layouts, file-first
  precedence, mixed direct modules, and unqualified unique accepted monomorphic
  scalar/Void calls across root -> module, module -> module, and module -> root. Its
  system-level example must combine accepted arrays and compile-time String classes
  through checked IR, pinned LLVM 22, link, and native exit 47. This does not implement
  `use`, `pub`, namespaces, recursive paths, cycles, packages, separate compilation,
  module ABI, ownership, runtime layout, or GPU execution. The Modules/imports/
  visibility row remains `PARSED_ONLY` and the module-resolver surface remains
  `EXPERIMENTAL` at authorization; no cell moves before exhaustive and public evidence.
- CORE-042's first red aggregate exposed nondeterministic checked LLVM definition
  serialization across identical multi-function CLI builds. Separately authorized
  prerequisite CORE-042A permits only exact-name ordering at the checked/public code-
  generation emission boundary, preserving unchecked compatibility and all language,
  IR, ABI, runtime, backend, and module semantics. This compiler-integrity correction
  moves no capability cell and makes no general reproducible-build claim.
- The resulting local CORE-042 aggregate passes the complete frozen direct-module
  composition product 1/1; exact root formatting, correctness Clippy, 146 library,
  156 CLI, all integration, and doc gates pass. The tracked three-file example and
  pinned LLVM/Clang 22 exit-47 workflow are present. Modules/imports/visibility remains
  `PARSED_ONLY` and the resolver remains `EXPERIMENTAL` until exact review, unchanged
  publication, all-eight public checks, and stable Linux native evidence complete.
- Exact unpublished candidate `91f96b5` is rejected: its production boundary remains
  substantively bounded, but exact-name order was not asserted, unreadable/wrong-return/
  child-precedence cells were absent, and sorted LLVM did not observe ordered multi-
  module collection. Its evidence also used a noncanonical hash and stale chronology.
  The additive tests/records correction now passes: exact-name order, unreadable and
  wrong-return rejection, child-before-arity precedence, and both direct-module
  declaration-order permutations are explicit. Production and native workflow remain
  unchanged; no capability cell moves. Exact successor review and public evidence are
  still required.

## CORE-043 bounded struct-value evidence

- Local CORE-043 evidence promotes only unique top-level, non-generic, nonempty
  structs whose unique fields collapse to `Int`, `Float`, or `Bool`. Exact named
  construction, source-order child evaluation, direct/local named projection,
  logical checked-IR schemas and verified LLVM named aggregates are covered. Exact
  implementation commit `92b19cf729daa4e3e90d4591495e493573c89e51` passes all eight
  public checks; stable Linux job `92163717297` verifies, lowers, links, and executes
  the tracked program with native exit 53 under LLVM/Clang 22.1.8.
- Struct ownership, moves/copies, parameters/returns, aggregate recursion, methods,
  assignment, destructuring, Match, generics, visibility, separate compilation,
  ABI/layout guarantees, heap/drop/lifetimes, and accelerators remain absent or
  excluded. The three struct rows are `PARTIAL`, never general struct support.

## CORE-044 bounded scalar-struct Copy-transport evidence

- Accepted CORE-044 implementation `da21a76cf92f2faf680a6284b4789fc401fed8fe`
  classifies only CORE-043 structs whose every field is already an admitted `Int`,
  `Float`, or `Bool` scalar. It proves local Copy
  aliases and original reuse; exact-name by-value internal function parameters,
  arguments, call results, and returns; mixed scalar/struct signatures; forwarding,
  terminating direct recursion, immediate projection, and flattened direct-module
  composition. The all-scalar struct Copy-transport row is `PARTIAL`, not general
  ownership or function ABI support.
- A distinct logical checked-function definition carries exact aggregate signatures.
  Verifier corruption controls reject empty/invalid/conflicting schemas, distinct
  struct identities, aggregate places used as values, wrong returns, and widened
  `main`; checked LLVM uses named aggregate load/store/call/return forms. Deprecated
  unchecked generation does not acquire the checked aggregate path.
- The exact local root gate passes 152 library and 160 CLI tests plus every active
  integration target, formatting, correctness Clippy, and doc tests. The tracked
  exit-63 example builds through the CLI. Push CI `30963297077`, PR CI `30963298874`,
  Rust CI `30963298877`, CodeQL `30963297658`, and aggregate `92171836058` pass all
  eight public checks. Stable job `92171725623` uses Ubuntu LLVM/Clang 22.1.8 for
  external verification, machine verification, object/link, and exact native exit 63.
- String/custom/nested/recursive/array/tuple/reference/generic fields and non-Copy
  ownership, partial/destructive moves, mutation, destructuring, Match, methods,
  heap/drop/lifetimes, stable layout/ABI/FFI, separate compilation, recursive module
  semantics, accelerators, performance, release, and stability claims remain absent
  or excluded. Existing broader struct rows remain `PARTIAL`.

## CORE-045 bounded fixed Copy-struct-array evidence

- Accepted CORE-045 implementation `54c02828413b505a1488b4333ae9db91d3773a32`
  admits only local fixed arrays whose element is one
  exact accepted CORE-044 all-scalar Copy struct. It covers nonempty literals,
  single-evaluation repeat including zero, exact typed empty arrays, immutable and
  mutable bindings, element-wise Copy aliases with original reuse, static `.len()`,
  compile-time constant in-bounds indexing and projection, normalized `.iter()`
  iteration, local arrays inside admitted functions, and flattened direct modules.
- One shared `StructRegistry` contract owns exact element, annotation, initializer,
  and source-index classification. Distinct logical checked array allocation and
  element-pointer instructions carry the exact struct schema and count; verifier
  corruption controls reject malformed/conflicting schemas, descriptor crossover,
  wrong storage and index types, constant out-of-bounds access, and legacy numeric
  GEP crossover. Checked LLVM uses `[N x %aero.struct.Name]`, exact aggregate loads
  and stores, and typed GEPs rather than the legacy numeric `double` fallback.
- Focused and adjacent suites pass, and exact root `./tools/test.sh` passes 154/154
  library and 161/161 CLI tests plus every active integration, formatting,
  correctness Clippy, and doc gate. The multi-file tracked example resolves a direct
  module and builds through the CLI into typed aggregate LLVM. Push CI `30966127286`,
  PR CI `30966129490`, Rust CI `30966129402`, CodeQL `30966127813`, and aggregate
  `92180425964` pass all eight checks; stable job `92180365622` externally verifies,
  machine-verifies, object-lowers, links, and records exact native exit 77. The
  specific row and broader Fixed arrays/struct rows remain `PARTIAL`; no stable ABI/layout,
  dynamic bounds, mutation, array function transport, non-Copy ownership, runtime,
  accelerator, release, or performance claim moves.

## CORE-046 accepted internal fixed Copy-array transport evidence

- Accepted CORE-046 implementation `056ca334df08176dafac815c1df78f3e90ed660a`
  admits only flat fixed arrays already
  executable before this task: `int`/`i32`, `float`/`f64`, or one exact CORE-044
  all-scalar Copy struct. Non-`main` internal parameters, arguments, call results,
  explicit/tail returns, forwarding, mixed signatures, terminating direct recursion,
  zero-length arrays, direct modules, and existing length/index/projection/iteration
  operations retain exact element identity, count, and struct schema. Caller values
  remain usable after the by-value call.
- The generalized Copy-function contract is the shared source authority for supported,
  explicitly rejected, and preserved signature topology. Exact logical arrays survive
  checked signatures, parameter places, aggregate loads/stores, calls, and returns;
  verifier corruption controls reject unsupported/nested elements and mismatched
  count/schema/value-place identities. Checked LLVM uses internal `[N x double]` and
  `[N x %aero.struct.Name]` aggregates without a stable ABI claim.
- The exhaustive focused aggregate and adjacent containment suites pass. Root
  `./tools/test.sh` passes 155/155 library and 161/161 CLI tests plus every active
  integration, formatting, correctness Clippy, and doc gate. The authorized
  multi-file example checked-builds into exact typed aggregate LLVM. Push CI
  `30968327941`, PR CI `30968330538`, Rust CI `30968330548`, CodeQL `30968328500`,
  and aggregate `92187139555` pass all eight public checks; stable job `92187043157`
  externally verifies, machine-verifies, object-lowers, links, and records exact
  native exit 91. This specific row remains `PARTIAL`/internal-only rather than stable
  ABI or general array/function support.
  Bool/String/non-Copy/nested arrays, arrays as fields, mutation, dynamic bounds,
  process-entry arrays, separate compilation, ABI/FFI, accelerators, performance,
  release, and stability claims remain excluded.

## CORE-047 accepted acyclic named Copy-aggregate evidence

- Accepted CORE-047 replaces scalar-only struct classification with
  one recursive, memoized graph decision. Unique, non-generic, nonempty definitions
  with valid unique fields admit scalars, another admitted named struct, or a flat
  fixed numeric/struct array. Forward references and arbitrary finite acyclic named
  depth are supported. Ambiguous, unknown, empty, generic, non-Copy, direct nested-
  array, Bool-array, self/mutual/zero-array-mediated-cycle, and dependent definitions
  remain rejected before IR.
- Exact recursive schemas drive construction, contextual empty fields, whole-value
  Copy aliases, chained projection, array operations through fields, internal
  parameters/results, and flat arrays of the new aggregate structs. Semantic preflight
  and checked admission no longer duplicate receiver topology; each recursively types
  the receiver and consumes the shared registry result. Recursive verifier controls
  reject conflicting/cyclic/unsupported schemas before deterministic named LLVM types.
- The exhaustive focused target, graph and verifier unit controls, tracked direct-
  module check/build, and adjacent CORE-043 through CORE-046 suites pass locally. The
  exact repository-root gate is formatting and correctness-Clippy clean and passes
  157 library tests, 163 binary tests, every integration target, and doc tests. The
  exact implementation is `a1dcc3fbef3ce0e4750a1476b348940a966bf609`, tree
  `15cf5d3451e1e02576c506d0bb4df4e3a62ab07c`, stable patch ID
  `2959bdc7d39ebe4a3d5e390f469fa9673033f9b6`. All eight public checks pass. Stable
  Linux job `92194611441` uses LLVM/Clang 22.1.8 for external verification, machine
  verification, object lowering, linking, and exact native exit 107, and records 157
  library plus 163 binary passes. Exec is direct evidence for this exact slice and is
  now `Y`; the row remains `PARTIAL` because the exclusions below remain open.
- Direct nested/Bool arrays, cyclic or non-Copy aggregates, mutation, dynamic bounds,
  move/drop/lifetime behavior, process-entry aggregates, separate compilation, stable
  layout/ABI/FFI, accelerators, performance, release, and stability remain excluded.

## CORE-048 accepted local immutable scalar references

- Accepted CORE-048 admits only non-escaping immutable aliases of
  initialized local/parameter `Int`, `Float`, or `Bool` places. Direct `&x`, inferred
  or exact annotations, copied/multiple aliases, immediate dereference, nested lexical
  use, owner reuse, and all already-admitted scalar consumers are covered. One shared
  classifier drives mutable/unsupported-origin/unsupported-pointee rejection in both
  semantic analysis and checked admission without extending binding-topology lists.
- `CheckedImmutableBorrow` defines a fresh exact alias place. Verifier controls reject
  malformed identifiers, undefined/non-dominating sources, duplicates, result/place
  collisions, unsupported metadata, and source/pointee mismatches. LLVM emits typed
  zero-offset pointer derivations and exact scalar loads, with no integer-pointer cast
  and no activation in the deprecated raw route.
- The exhaustive target, private corruption controls, frontend/Phase 5/binding/checked-
  IR neighbors, tracked direct-module check/build, and CLI artifact-hygiene controls
  pass locally. The exact root gate passes 159 library and 165 binary tests plus every
  downstream formatting, correctness-Clippy, integration, and doc gate. The composed
  direct module builds to typed `double`/`i1` alias and load LLVM, with local status
  accurately `InternalOnly` because LLVM 22 is absent. Exact implementation
  `98c21b9012a5d6581c31c67a0378f20363e0688d` passes all eight public checks. Stable
  job `92201296160` uses LLVM/Clang 22.1.8 for external and machine verification,
  object lowering, linking, exact native exit 127, and 159/165 test passes; execution
  is now `Y`. The row remains `PARTIAL`, not a general borrow-checker or memory-safety
  claim.
- Mutable borrowing/dereference, non-scalar or temporary origins, function reference
  ABI, escaping/aggregate references, assignment/mutation, NLL, owner drop/resource
  ownership, FFI/stable pointer ABI, accelerators, performance, release, and stability
  remain excluded.

## CORE-049 accepted owned unit-enum exhaustive Match

- Accepted CORE-049 admits exactly one unique top-level, non-generic, nonempty enum
  definition containing unique unit variants and no same-name struct. Payload-free
  constructors initialize immutable inferred or exact local bindings; local aliasing
  moves the source, and matching an identifier consumes it because the enum is not
  `Copy`. One shared registry/classifier owns definition, constructor, annotation,
  execution-context, exhaustive-arm, scalar-result, and nested consumption topology
  across semantic analysis and checked admission.
- An admitted Match contains exactly one explicit payload-free arm for every declared
  variant in arbitrary order, with uniform exact `Int`, `Float`, or `Bool` results.
  Nested matches and existing scalar parents are supported, only the selected arm
  executes, and possible-arm nested consumption is conservative at the expression join.
  Unsupported Match topologies retain their established fail-closed behavior.
- Checked IR introduces a distinct schema-bearing `LogicalType::Enum`; the current
  shared forms use exact `CheckedEnumVariant` and exhaustive `CheckedEnumDispatch`.
  Independent verification rejects malformed/empty/duplicate/conflicting schemas,
  invalid indices, immediate/undefined/non-dominating/wrong-enum values, incomplete/
  duplicate/missing targets, identifier-kind collisions, and excluded aggregate or
  function transport. LLVM lowers the verified local identity to internal `i32` and
  `switch i32`; this is not source `Int` and defines no stable enum ABI.
- The exhaustive target passes 1/1, unsupported Match containment passes 15/15, and
  the exact root gate passes 160 library and 166 binary tests plus formatting,
  correctness Clippy, every integration target, and doc tests. Exact implementation
  `b38a6b0927c747909918b5ebf3c0f6b58d0727dd`, tree
  `80829d3a74ddf2b6edfa247b75205b0a0ec799cc`, stable patch ID
  `c22f9210b9756645022be636cb98d24678d5a60f`, passes all eight public checks. Stable
  job `92208529644` uses LLVM/Clang 22.1.8 for external and machine verification,
  object lowering, linking, exact native exit 149, and 160/166 test passes. Execution
  is `Y` only for this bounded class; the broader enum/Match row remains `PARTIAL`.
- Payload/struct/generic/mixed enums, Option/Result Match, wildcard/binding/guard/
  literal patterns, enum parameters/results/calls/aggregates/references, mutation,
  borrowing, public discriminants, stable layout/ABI/FFI, heap/drop behavior,
  accelerators, performance, release, and stability remain excluded.

## CORE-050 accepted internal owned unit-enum transport

- CORE-050 admits enum-bearing signatures only for unique top-level non-generic
  non-`main` functions and exact CORE-049 unit-enum annotations. Other parameters and
  results are restricted to already-admitted by-value scalars, finite acyclic Copy
  structs, and flat fixed Copy arrays. Multiple/mixed parameters, multiple enum names,
  enum producers, and Void enum consumers are covered by one shared signature resolver;
  unsupported annotation topologies fail before IR.
- Constructors, owned locals/parameters, and admitted call results may cross exact
  internal function boundaries. Passing a named enum transfers ownership. Call results
  may bind, move, feed another call, return, or serve directly as an exhaustive Match
  scrutinee. One recursive consumed-name classifier closes nested calls, argument lists,
  Match evaluation, duplicate consumption, use-after-call, and arm reuse across semantic
  analysis and checked admission.
- Checked enum-bearing functions retain exact `LogicalType::Enum` schemas and use direct
  then-named `CheckedUnitEnumParameter` SSA binders rather than generic storage.
  CORE-052 later generalizes that binder name and schema class. Independent
  verification proves binder/signature coverage and identity, call argument/result and
  return types, dominance, and global enum-schema consistency. LLVM lowers only verified
  enum parameters/calls/returns to direct internal `i32`; no source `Int`, public
  discriminant, layout, calling convention, stable ABI, or FFI is established.
- Exact implementation `13f000358bdab33a2a8f5618bdbe80ffc50a1ed9`, tree
  `e0228d7f0b056137abe1cc29e8078668ec0872fd`, and stable patch ID
  `ee4eb0b8efc4847e30091d0293eed746b40851fa` pass the exhaustive target, verifier
  corruption cases, adjacent controls, exact root gate, and all eight public checks.
  Stable job `92215771782` uses LLVM/Clang 22.1.8 for external and machine
  verification, object lowering, linking, exact native exit 173, and 161/167 test
  passes. The broader enum/function/ownership rows remain `PARTIAL`.
- Payload/struct/generic/mixed enums, Option/Result, enum arrays/struct fields/references,
  mutation, borrowing, equality, casts, printing, `main`, closures/nested functions,
  traits, recursive CFG ownership, loop state, stable ABI/FFI, accelerators, performance,
  release, and stability remain excluded.

## CORE-051 accepted owned unary scalar-payload enums

- CORE-051 generalizes the shared enum classifier to unique top-level non-generic,
  nonempty definitions whose declaration-ordered variants are unit or carry exactly
  one `int`/`i32`, `float`/`f64`, or `bool` payload. Mixed schemas are admitted.
  Empty/multi-field tuples, struct variants, non-scalar/nested/generic payloads,
  ambiguous definitions, and type-name collisions share one explicit rejected or
  quarantined disposition rather than topology-specific phase guards.
- Construction requires payload presence and the exact declared scalar type. An
  exhaustive Match requires one identifier payload binding for each payload variant,
  scopes that initialized immutable Copy scalar only to its arm, and consumes the
  non-`Copy` enum. Unit arms remain payload-free. Missing/foreign/duplicate arms,
  wildcard/literal/nested payload patterns, binding leakage, scrutinee shadowing,
  use-after-Match, and result-type mismatches fail before checked IR.
- Current shared checked IR uses `EnumSchema`, `CheckedEnumVariant`,
  `CheckedEnumPayload`, and `CheckedEnumDispatch`. Independent verification proves
  schema identity, construction payload presence/type/dominance, exhaustive unique
  targets, selected-variant guarding, exact extraction type/source, global consistency,
  and transport containment. Verified local payload enums lower privately as
  `{ i32, double, i1 }` with deterministic zero in inactive lanes. No public
  discriminant, memory layout, calling convention, stable ABI, or FFI is established.
- The exhaustive target, private verifier corruption cases, CORE-049/050 and adjacent
  binding/struct/typed-admission controls, and exact root gate pass locally: 162
  library and 168 binary tests plus formatting, correctness Clippy, every integration
  target, and docs. The composed `signals` example reaches LLVM with 12 tagged
  aggregate insertions, 16 selected-lane extractions, six switches, and SHA-256
  `42930069C175CE245EEA0C2CFBF0F01B0D0B21FD1FD9AB2B9B587BC6990D39CC`.
  Exact implementation `babb1cd543fb36e13ec16458889f336ad5549a49`, tree
  `6b8382ed0370c67994ee519a892f149c3ffe4825`, and stable patch ID
  `2aaf5bee97f294f90c9494b364267deb250601b8` pass all eight public checks. Stable job
  `92223344697` uses LLVM/Clang 22.1.8 for external/machine verification, object/link,
  exact native exit 181, and 162/168 test passes.
- Fields, arrays, references, borrowing, mutation,
  equality, casts, printing, copying, heap/drop, Option/Result Match, guards,
  destructuring, general CFG ownership, loop state, stable ABI/FFI, accelerators,
  performance, release, and stability remain excluded. The broad enum and Match rows
  remain `PARTIAL`.

## CORE-052 accepted owned scalar-payload enum transport

- CORE-052 removes the final unit-only condition from the one shared enum transport
  annotation resolver. Every exact schema already admitted by `EnumRegistry`—unit or
  any declaration-ordered mix of unary `Int`/`Float`/`Bool` variants—may now occur in
  exact internal parameters and results alongside existing admitted by-value scalars,
  finite acyclic Copy structs, and flat fixed Copy arrays. Multiple enum parameters,
  multiple enum names, consumers, producers, identity/forwarding chains, direct
  constructors, named moved values, call results, and direct-result Match are covered.
- Passing or returning a named enum transfers ownership and invalidates its source; the
  Copy payload does not make the containing enum Copy. The existing recursive consumed-
  owned-value classifier closes nested calls, argument lists, duplicate consumption,
  use after call, and Match-arm reuse. Containers, references, fields, mutation,
  borrowing, partial payload moves, general CFG/NLL, drop, and unsupported contexts
  remain fail-closed.
- Checked IR renames the unit-specific binder to one `CheckedEnumParameter` carrying
  the exact schema. Independent verification proves schema validity/global identity,
  signature/binder coverage, entry-block placement, call argument/result and return
  equality, dominance, and transported construction/extraction/dispatch integrity.
  Unit transport remains private `i32`; payload transport remains private
  `{ i32, double, i1 }` through internal definitions, parameters, calls, and returns.
  This is not a stable layout, calling convention, ABI, or FFI contract.
- The exhaustive target, payload-specific verifier corruption cases, CORE-050/051
  controls, and exact root gate pass locally: 162 library and 168 binary tests plus
  formatting, correctness Clippy, every integration target, and docs. The composed
  module example reaches LLVM with seven aggregate-returning definitions, ten aggregate
  calls, eight aggregate returns, seven switches, 48 extractions, 45 insertions, and
  SHA-256 `AD23CC66B1579D18870F05E3C63481C781209033F63F5A653872FB88B77160B5`.
  Exact implementation `93a4a29e0b50f8d16ce6e2f845306b4ffcb37738`, tree
  `eefd479e97754f1f069b67c640c2c27d179e28fe`, and stable patch ID
  `8b0d7132e75ca8010fee3a39da021b320383565e` pass all eight public checks. Stable job
  `92227409386` uses LLVM/Clang 22.1.8 for external/machine verification, object/link,
  exact native exit 197, and 162/168 test passes.
- Enum arrays/fields/references, aggregate/non-scalar/multi-field/struct/generic/
  recursive payloads, Option/Result transport or Match, process-entry/closure/nested/
  trait/impl contexts, mutation, borrowing, equality, printing, heap/drop, recursive
  CFG ownership, stable ABI/FFI, accelerators, performance, release, and stability
  remain excluded. The enum, pattern, function, and ownership rows remain `PARTIAL`.

## CORE-053 accepted immutable scalar-reference parameter transport

- CORE-053 admits reference-bearing signatures only for unique top-level non-generic
  non-`main` functions. Any number and declaration order of immutable `&int`, `&float`,
  or `&bool` parameters may mix only with by-value `Int`/`Float`/`Bool`; the result is
  `Int`, `Float`, `Bool`, or `Void`. One whole-signature classifier gives each topology
  a supported, explicitly rejected, or preserved disposition across semantic analysis
  and checked admission, preventing duplicated phase-local guard products.
- Exact arguments are direct borrows of supported places, supported local aliases or
  copied aliases, or admitted immutable-reference parameters forwarded through calls.
  Multiple shared borrows, arbitrary parameter order, call chains, module calls, and
  terminating direct recursion are covered. Mutable references, reference results,
  temporary or non-scalar pointees, aggregate companions, storage/capture, escaping,
  NLL, drop, and lifetime-sensitive forms remain fail-closed.
- Checked IR adds `LogicalType::ImmutableReference` and one explicit
  `CheckedImmutableReferenceParameter` entry-block place binder. Independent
  verification proves exact signature/binder coverage, uniqueness, name and pointee
  identity, dominance, and pointer-bearing call arguments. Verified LLVM uses internal
  `double*` for `Int`/`Float` and `i1*` for `Bool`, typed zero-offset pointer derivation,
  and scalar loads without pointer/integer conversions. No public pointer identity,
  stable calling convention, layout, ABI, FFI, or memory-safety claim is established.
- The exhaustive target, binder/call corruption matrix, CORE-048 and enum/aggregate
  controls, and exact root gate pass locally: 163 library and 169 binary tests plus
  formatting, correctness Clippy, every integration target, and docs. The composed
  `borrows` module example reaches LLVM with four `double*` definitions, one `i1*`
  definition, five typed pointer calls, seven parameter GEPs, zero pointer/integer
  casts, and SHA-256
  `5EA298FBD6CB9A96F525EC680AA250EB93F46D19DAD0763733C9C17726924685`.
  Exact implementation `b4aec4a01312088807750b0e40150cee87dc2131`, tree
  `197e3b9ee615d32da569d55740891a14bcaced27`, and stable patch ID
  `a78d4d38c0bf8266b1f724d69c5ff97d28d2c5d0` pass all eight public checks. Stable job
  `92235191630` uses LLVM/Clang 22.1.8 for external/machine verification, object/link,
  exact native exit 211, and 163/169 test passes.
- General reference transport, mutable loans, reference results, aggregate/storage
  references, lifetime inference, NLL, drop/destruction, stable ABI/FFI, accelerators,
  performance, release, and stability remain excluded. Function, ownership, and
  reference rows remain `PARTIAL`.

## CORE-054 accepted mutable local scalar reassignment

- CORE-054 parses `target = value;` as an explicit statement and admits it only inside
  admitted function bodies when `target` is the nearest initialized, owned local
  `let mut` of exact type `Int`, `Float`, or `Bool` and `value` has the same logical
  type. Sequential writes, nested/shadowed locals, branches, compiler-bounded `for`,
  `while`-carried state, internal calls, and one-level direct modules are included.
- One whole-assignment classifier gives supported, explicitly rejected, and preserved
  dispositions from resolved topology, locality, initialization, mutability, ownership,
  and exact type. Semantic analysis and checked admission consume that classifier;
  they do not maintain topology-specific duplicate guard tables.
- Checked IR adds `CheckedMutableScalarAlloca` and `CheckedScalarAssignment`.
  Independent verification requires supported metadata, collision-free and dominating
  place/value definitions, exact target/value type, one adjacent initialization store,
  and the explicit checked identity for every later source write. A generic `Alloca`
  cannot substitute as the target, and a later raw `Store` cannot substitute for a
  checked source assignment.
- Verified lowering keeps the established private `double` representation for
  `Int`/`Float` and `i1` for `Bool`, and writes then reloads the same stack place across
  real branch and loop CFG. The exhaustive source/IR/LLVM/CLI target and private
  corruption matrix pass locally. The tracked direct-module system gate composes
  immutable scalar-reference transport, payload/unit enums, Copy aggregates/arrays,
  compile-time String length, calls, and control flow and passes exact stable native
  exit 227.
- The exact repository-root gate is formatting and correctness-Clippy clean and passes
  165 library tests, 171 binary tests, every integration target, and doc tests. The
  composed example emits 10 function definitions, 18 `double` and five `i1` allocas,
  27 `double` and seven `i1` stores, 26 `double` and six `i1` loads, and zero
  pointer/integer casts in a 10,798-byte LLVM artifact with SHA-256
  `a7104e3db6d7f0775cd5722e8f7c672fb2b11711803e5d130e140acb88b04b17`.
  Exact implementation `6ef3e44f8c7910815031c12e880ac874141cef5c`, tree
  `b6fe360fa42dfefef48492423a481da930279c8f`, and stable patch ID
  `7cfa95a31f53381e4bc373ebc07d09d76a0d76fc` pass all eight public checks. Stable
  job `92242692711` uses LLVM/Clang 22.1.8 for external/machine verification,
  object/link, and exact native exit 227. The Windows host accurately reports
  `InternalOnly` because LLVM 22 is absent.
- Immutable locals/parameters, unknown or uninitialized targets, borrowed or moved
  targets, non-identifier targets, String/aggregate/reference/function values,
  implicit conversion, assignment expressions/chaining/compound forms, general or
  escaping mutable references, NLL, drop/destruction, stable ABI/FFI, accelerators, performance,
  release, and stability remain excluded. Assignment, ownership, control-flow, and IR
  rows remain `PARTIAL`.

## CORE-055 candidate non-escaping local mutable scalar references

- CORE-055 admits one direct `&mut owner` local alias inside an admitted function when
  `owner` is an initialized mutable `Int`, `Float`, or `Bool`. Inferred or exact
  annotations, `*alias` reads, exact `*alias = value;` writes, sequential/branch/loop
  execution, nested shadowing, function bodies, direct modules, and lexical release
  followed by owner reuse are included. Mutable aliases are non-`Copy` and cannot be
  relocated or reassigned.
- The evolved shared local-reference classifier resolves mode, direct identifier
  topology, source locality/initialization/mutability/ownership, and exact scalar
  pointee once. Semantic analysis and checked admission consume that disposition;
  mutable dereference assignment consumes the same reference contract instead of a
  second topology guard table.
- Checked IR adds `CheckedMutableBorrow`,
  `CheckedMutableDereferenceAssignment`, and `CheckedMutableBorrowEnd`. Independent
  verification requires a declared initialized mutable scalar source, exact and
  dominating alias/source/value identities, one active exclusive loan, exact pointee
  metadata, checked writes, and an exact lexical release triple. It rejects generic
  allocas, raw stores, immutable-borrow substitution, owner access during the loan,
  competing loans, wrong release identity, and use after release.
- Verified LLVM derives typed zero-offset `double*` aliases for `Int`/`Float` and `i1*`
  aliases for `Bool`, then performs exact typed loads/stores without pointer/integer
  conversion. The exhaustive source/IR/LLVM/CLI target, verifier corruption matrix,
  full Rust suite, and tracked direct-module checked build pass locally at 166 library
  and 172 binary tests. The stable workflow requires external/machine verification,
  object/link, and exact native exit 239 before public acceptance.
- Mutable-reference parameters/results, reference relocation, escaping/storage/capture,
  non-identifier or projected origins, String/aggregate/nested pointees, reborrowing,
  NLL, lifetime inference, drop/destruction, stable ABI/FFI, accelerators, performance,
  release, stability, and general memory-safety claims remain excluded. Reference,
  ownership, assignment, control-flow, and IR rows remain `PARTIAL`.
