# Scoped semantics for the Aero-authored compiler

## Authorization and checkpoint sequence

The user authorized scoped names, types and ownership after published AST head
`6d98b07eb7cf1b00d6be5cb6ea61cfe42cdd613d`. Its completed local gate is in the
commit; all13 exact-head public checks were read successful before this work.

This milestone is deliberately staged. **H1C-SCALAR-001** is the first complete
semantic-only slice: lexical scalar bindings, exact int/Bool expression types,
Copy facts, immutable/mutable assignment rules and block scope. Follow separately
with resolved calls and concrete Result/Match, then accepted ByteBuffer
owners/immediate loans. Checked lowering remains a later contract. Completing
the scalar slice does not complete H1C or establish general ownership safety.

## Existing authority

- `docs/language/aero_formal_language_specification.md` sections4.1/4.2:
  same-scope rebinding rejects, nested shadowing is permitted, signatures are
  globally visible and Return must match its declared type.
- Rust `ScopeManager::define_variable` and `get_variable` in
  `src/compiler/src/semantic_analyzer.rs`: duplicate current-scope names reject;
  lookup is nearest enclosing scope. Parameters are immutable initialized
  bindings in the same scope as the direct function body. The Let initializer
  is analyzed before the new binding is introduced.
- The existing Aero arithmetic semantic rules remain authoritative for kinds
  1,3..19: int1, Bool2, structural0; arithmetic/logic/Return diagnostics and
  unsupported remainder retain their rules. No annotation payload is blindly
  copied to a fact: annotation2 is Result, **not** semantic Bool2.
- `OWNED_BYTE_BUFFER_READINESS.md` and the accepted byte-buffer source contract
  admit direct function-local owners/moves only, with immediate nonescaping
  intrinsic loans. They do not authorize block-local owners or generic borrows.

## First slice: frozen meaning

Admit annotation24 only with payload1 (`int`), parameters25, lets26/27,
assignment28, statement cells29, Block30, branch pair31, If32, While33,
descriptor38, and the existing scalar expression/Return/Function kinds.
No parser grammar changes. Require explicit int initialization, int assignment
and int Return; conditions must be Bool produced by existing expression rules.
Parameters and immutable Lets cannot be assigned; mutable Lets can.

All scalar expressions carry the existing Copy convention `[node,type,1]`.
Annotation/parameter/declaration/assignment/statement/block/branch/control/
descriptor/Return/Function nodes are non-values `[node,0,0]`, not fabricated
ints. Keep one fact per node and exact12-byte records. A resolved Identifier
has its binding's exact int type; its declaration does not itself become a value.

Same name in distinct functions is isolated. Nested Block shadowing is valid,
including `let x:int=x;` using an outer x in its initializer. A binding is visible
only after its complete initializer; after closing a Block it is not visible.
Parameters share the function body's outer Block scope. Assignment lookup uses
the target token before its RHS; no implicit declaration or mutability change.
Reject duplicate function names for modules using this new rich semantic route.
The unchanged compact arithmetic route retains its historical contract.

Calls20, argument cells21, references22/23, non-int annotations24, and Match
kinds34..37 remain unsupported on the new route, which applies only when the
module contains any kind24..38. A module without those kinds retains its entire
historical compact route, including unsupported calls/references/name diagnostics,
not just positive arithmetic LLVM. First classify the *complete authenticated*
module for this closed subset, then analyze names/types. Thus an unsupported
module receives located28/code=first unsupported kind in ascending node order,
with zero facts, rather than speculative partial name/type meaning. This is an
explicit successor diagnostic-priority change, not preservation of the old
representation-only identifier refusal. Keep the old model intact in tests.
An int Identifier in an admitted scalar module still rejects17/code2 when
unresolved. All structure/source authentication precedes either route.

## Bounded storage and independent evidence

No existing bound may increase. Nodes/controls remain65536, tokens262144,
names16384 and source1048576 bytes. The lead approves these append-only stores
before production edits:

- Reverse-node contexts `[functionNode,scopeBlockNode,assignmentTargetEvent]`,
  three words/node, at most65536 records/786432 bytes, addressed by
  `(node_count-node)*12`.
- Scalar bindings `[nameId,functionNode,scopeBlockNode,declarationNode,mutable]`,
  five words/binding, at mostnode_count records/1310720 bytes. Only admitted
  int declarations activate; their type is fixed by validated annotation1.

These are logical record-byte ceilings, not exact allocator sizes. The unchanged
ByteBuffer doubling rule permits context capacity up to1048576 allocated bytes
and binding capacity up to2097152 allocated bytes. Derive and grade these physical
capacities separately; reserved/poisoned tails are not logical records. No source,
token, node or statement ceiling is increased by this distinction.

Reverse postorder follows the authenticated contiguous subtree start indices.
Each function establishes its own outer-body Block context; parameters occupy
that same scope. A nested Block's already appended context records its parent;
its children use that Block until crossing its start, then a direct context
lookup restores the parent. Outer-body scope is self and does not unwind for
parameters. A previous Function establishes its own context, not the later one's.
No separate history or resolution table is necessary.

Assignments contain no nested assignments in this grammar. Reverse traversal
marks an assignment-target event at that assignment's subtree start (first RHS
node). A name-only forward pass first handles that target before its RHS, then
Identifier resolution, parameter activation, completed Let activation and
same-scope duplicates. Unknown/duplicate/immutable failures have zero facts.
Global duplicate functions are checked before this name pass. The independent
oracle may traverse source order recursively; the product must remain iterative.
A second ascending fact pass uses completed binding records but admits only
declarations with declarationNode<useNode, selecting nearest enclosing scope
identities rather than a recent declaration in an unrelated scope. This keeps
initializer activation and shadowing exact without back-patching.

Mixed-invalid diagnostic ordering is an explicit successor choice: a Let's
initializer name failure is reported before checking its same-scope duplicate,
because this pass activates/checks completed declarations. Rust currently checks
same-scope duplicates before analyzing the initializer. Neither choice changes
valid binding visibility or admits an invalid program; do not claim identical
Rust diagnostic ordering for this mixed-invalid case. Freeze a regression for
`let x:int=1;let x:int=missing;` producing17/code2 with zero facts.

Heap failures26/8(context),26/9(bindings) use the current
authenticated node's exact location. These occur only after complete source/
origin authentication; earlier unlocated26/4..7 behavior stays unchanged. No
fake capacity error or missing record may silently select an outer binding.
Independently derive first-use owner identities/cleanup/growth thresholds and
require exact observer capture before claiming the tables correct.

No mutable table API, generic ByteBuffer parameter or recursive product
traversal is available.
Stop rather than exceed canonical self-source capacity or impose a new syntax
cutoff. An independent source/AST census precedes changed native self-input.

The new semantic oracle is independent of the product and the old
representation-refusal model. It can consume independently constructed connected
ASTs; hand-derived facts and Rust semantic overlap are additional authorities.
Full-vector native runs at O0/O2 must authenticate actual facts and diagnostics,
not parser-only exit80. The accepted signature ABI has68 fields:67 pipeline
fields plus parameter count. Preserve all fields, not a truncated67-value check.
Preserve old oracle tables and use explicit successor
grading where behavior changes. Count-only acceptance is insufficient.

## Fail-before-lowering boundary and diagnostics

Successful rich scalar analysis finishes all facts and invariant checks, then
reports **semantic29/code1**, at the authenticated root Function token, with
semantic root type1 and fact_count=node_count. This means typed scalar semantics
complete / lowering unsupported, not a source error or successful compilation.
The existing checked-entry guard sees nonzero status and never attempts IR.
Checked IR, verifier, emitter, driver, main and runtime remain byte-identical.
Keep compact arithmetic successful status0 and emitted LLVM exact.

For the new rich route, freeze duplicate function30/code1; duplicate binding
30/code2; unresolved assignment target30/code3; Let mismatch31/code1 and
assignment mismatch31/code2 (expected1/actual inferred type); condition mismatch
31/code3 (expected2/actual inferred type); immutable assignment32/code1.
Use the offending authenticated declaration/assignment/control origin.
Unresolved Identifier remains17/code2. Existing operator/Return diagnostics
remain unchanged. Stop/failure never enables later phases. Fact prefixes on
type failure are legitimate only if independently predicted and graded exactly.

## Acceptance and mandatory stops

Before product edits, genuine native reds for parameter use, mutable scalar
assignment, nested shadowing/scope exit, duplicate declarations, initializer
visibility, condition and annotation mismatch, and immutable parameter/Let
assignment. Afterward: hand facts, independent model/native equality O0/O2;
function isolation, early Returns, nested loops/else-if; malformed AST/origin
and prior allocation/capacity failures; new store first allocation/reallocation
failure, exact counters/cleanup and zero leaks; exact arithmetic LLVM; unchanged
downstream groups; independent canonical budget; then root ./tools/test.sh.

No failing test may be weakened or skipped. No widening to Result, calls, buffer
ownership, arbitrary references, implicit coercion, new grammar, lowering or
backend is authorized by this first slice. Report any semantic ambiguity first.

## Later ownership obligations already identified

Resolved calls must use the separate declared-function namespace and checked
arity/types; ByteBuffer and byte-I/O function names are reserved. Concrete Result
binders are arm-local and Ok/Err must be exhaustive; unknown constructors do not
gain int meaning. Reject aggregate comparisons that trusted checked admission
does not support, even where the old semantic helper was permissive.

Concrete Result is **non-Copy** in the accepted type contract: bindings,
by-value arguments and Match consumption can move it. Later support must mirror
reachable-path Owned/Moved/MaybeMoved joins and loop behavior, or reject explicitly
unimplemented flow before facts/lowering. It must not use one unconditional
source-order move log across mutually exclusive branches. These obligations are
why Result ownership is not folded into the scalar Copy checkpoint.

For ByteBuffer, do not activate a mutable loan at the textual `&mut` token:
`bytes_push(&mut bytes, bytes_len(&bytes))` is already accepted because the nested
immutable scalar-argument loan ends before the outer mutable loan begins. Buffer
reassignment, function transport and owner creation/moves under control flow
remain outside accepted authority. stdin/stdout/stderr intrinsics additionally
require their exact direct annotated Result initializer contexts; bytes_get/
bytes_push Results can be immediately passed to a declared Result parameter.
These requirements are future slice boundaries, not implementation claims.
