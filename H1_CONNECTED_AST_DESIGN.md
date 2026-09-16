# Connected AST for the canonical Aero compiler

Task: `SELFHOST-AST-DESIGN-001`. **Design only; no executable capability or
test result is claimed.** The lead must review and freeze this representation
before implementation. CAP-059 stage 3b must finish before AST product edits.

## Purpose and existing authority

H1B requires a validated flat AST for every construct in the canonical compiler.
The current parser admits that syntax but discards much of its structure. This
design preserves the admitted grammar. Proposed node tags are internal encoding
choices, not new language meaning, admission rules, or inferred types.

The Rust AST supplies the distinctions to preserve: `src/compiler/src/ast.rs`
defines `Statement` at line 106, `MatchArm`/`Pattern` at 192, and
`Parameter`/`Block`/`Type` at 253. `parser.rs:319` represents an `else if` as a
nested `If` in the enclosing `else` branch. The closed self-source grammar in
`BOOTSTRAP_CONVERGENCE_READINESS.md` remains the scope, not the entire Rust AST.

Source references below describe `examples/aero_self_host_v0/compiler.aero`
before CAP-059 edits; use the named parser states when line numbers move.

| Lost information | Current site |
|---|---|
| Parameter ownership and AST linkage | State 2 / `param_store`, lines 1482-1663: global name/type pairs only |
| Binding name, mutability, annotation, initializer relationship | State 47, lines 1830-1955: checked then discarded |
| Assignment target and statement order | States 45/47, lines 1713-1955: no statement nodes |
| Nested returns and all but final return relationship | State 49, lines 1982-1990: latest `body_root` wins |
| Conditions, branch association and loop bodies | States 51-54, lines 2008-2106: control stack only |
| Match subject, patterns, binders and both arm roots | States 41/43/18, lines 2108-2203 and 2873-2888 |

Calls, argument order and references already have nodes 20-23. Module items
already link through kind-19 `right`. Retain these representations.

## Proposed node records

Keep four nonnegative words `[kind, payload, left, right]`, one-based IDs, and
zero for absent links. Every nonzero node link points to an earlier node.
Existing kinds 1-23 retain their meanings; the table proposes additions.

| Kind | Role | Payload | Left | Right |
|---|---|---|---|---|
| 24 | Annotation | 1=int, 2=Result<int,int>, 3=ByteBuffer | 0 | 0 |
| 25 | Parameter | Name ID | Annotation node | Previous parameter or 0 |
| 26 | Immutable let | Name ID | Annotation node | Initializer expression |
| 27 | Mutable let | Name ID | Annotation node | Initializer expression |
| 28 | Assignment | Target name ID | Value expression | 0 |
| 29 | Statement cell | 0 | Previous cell or 0 | Statement node |
| 30 | Block | 0 | Final statement cell | 0 |
| 31 | Branch pair | 0 | Then block | Else block, nested If, or 0 |
| 32 | If | 0 | Condition | Branch pair |
| 33 | While | 0 | Condition | Body block |
| 34 | Pattern binder | Name ID | 0 | 0 |
| 35 | Match arm | Constructor name ID | Binder node | Arm expression |
| 36 | Two-arm sequence | 0 | First arm | Second arm |
| 37 | Match expression | 0 | Scrutinee identifier node | Arm sequence |
| 38 | Function descriptor | Return annotation code 1 | Final parameter or 0 | Body block |

Parameters and statement cells are reverse-linked; visit the previous link
before the current element to recover source order. Iterative traversals use
explicit bounded work stacks, not recursive host assistance. Annotation codes
encode exact parsed spellings: parameter types stay int/Result<int,int>, local
bindings additionally allow ByteBuffer, and function returns remain int.
Unknown spellings must retain their existing rejection, never default to int.

Each annotation occurrence owns its annotation node. Pattern binder nodes
represent declarations, not identifier-use expressions. Preserve the actual
constructor names and source arm order: the currently admitted match grammar
accepts identifiers, and this representation must not silently rewrite them to
Ok/Err. Constructor validity and binder scope belong to later semantic work.

Kind 18 represents every Return statement at its own source location. Kind 19
retains its name payload and previous-item `right`; its `left` is either the
existing compact Return node or a kind-38 descriptor.

## Compact compatibility without orphan nodes

For exactly zero parameters and exactly one Return statement, retain the
existing kind-19 -> kind-18 path. Its int return annotation is implicit in the
unchanged admitted grammar. This preserves the arithmetic subset's AST and
downstream contracts while allowing rich functions to fail closed separately.

Hold the first completed statement ID in the block's parser registers without
creating a statement cell. When another statement starts, promote the held
statement into the first cell and continue the sequence. At function close,
emit no wrapper for the compact case; otherwise materialize any pending cell,
the Block, and the descriptor. Nested blocks always materialize their Block.
Do not construct wrappers and then abandon them. Emit the Return once, at its
semicolon; remove the duplicate function-close Return append. Parameter nodes
and annotations are created only when actual parameters occur.

## Parser continuation and origin changes

State 2 attaches parameter nodes to a per-function tail, reset at each `fn`.
State 45 captures statement origin and assignment name; state 47 retains let
name, mutability and annotation. State 18 routes a completed expression to its
statement, condition or match arm. State 49 appends ordinary statements rather
than overwriting `body_root`. States 21-23 finalize the body and module item.

The three-word block frame in states 52-54 is insufficient. Freeze an expanded
continuation record containing kind, parent frame, parent statement state,
parent pending-first statement, parent sequence tail/count, condition, pending
then block, and construct start/line/column. Restore all parent state when a
child closes. Finalize a While immediately; delay an If until lookahead proves
whether an else follows. An else-if uses a nested If continuation. A token read
to prove there is no else must be reused for the enclosing statement dispatch.
Replacement continuation records may be appended; they are parser work records,
not AST nodes. No AST node requires back-patching.

States 41/43 retain the match subject, constructor/binder names and locations,
and each completed arm root. Build both arms and then the Match node after the
closing brace. Never reuse the second arm's expression as the whole match.

Every AST node receives one authenticated origin. Retain exact declaration and
binder locations, not the interner's first occurrence of their name. Structural
anchors are explicit: Block uses `{`, descriptor uses `fn`, branch pair uses
`if`, statement cell shares its statement's leading token, arm sequence uses
`match`. Validate these role-dependent mappings against source and tokens;
synthetic wrappers do not have one universal token kind. Keep node/origin counts
equal. Extend the validators near lines 3730-3826 and 3980-4059 accordingly.

## Invariants and acceptance probes

- On complete parse, root equals node count and every AST node is reachable
  from root. Auxiliary value/operator/call/block/parameter work stores are not
  AST nodes. Validate legal child kinds, name/type domains, and backward edges.
- Normalize to names, annotations and ordered structural fields, erasing only
  node IDs and compact wrappers. Compare against the Rust parser's AST for the
  closed subset, using an independent traversal rather than a copied parser.
- Hand-derive graphs for typed parameters; mutable let/assignment/return;
  nested while/if; asymmetric else-if; and returns in different nested blocks.
- Include canonical `result_value`, calls with ordered borrowed arguments, and
  two functions with different parameter lists to detect ownership leakage.
- Use equal-size adversarial pairs that swap assignments, branches, arm bodies,
  binders or parameter ownership. Counts alone must not satisfy the tests.
- Check each canonical function and the complete source for normalized
  structural equality, zero orphans and exact node/origin accounting.
- Preserve malformed-syntax diagnostics and arithmetic LLVM bytes, including
  CAP-059 multi-function output. Exercise exhaustion and allocation failures at
  new append edges. Derive counts from the chosen representation, not byte size.
- Literal-only rich forms must stop before checked IR until meaning/lowering
  exists; absence of identifier nodes must not bypass this boundary.

## Implementation boundaries and unresolved measurements

The representation checkpoint owns parser plus semantic-origin authentication,
with one vertical owner. It adds a frozen located refusal for unsupported rich
structure before checked IR, without giving structural nodes invented scalar
facts or weakening `fact_count == node_count`. Keep the compact arithmetic path
unchanged. A universal descriptor rewrite would also require checked-IR edits
and thus cross three phases; that is why the compact encoding is deliberate.

Later contracts separately cover scoped names/types/ownership, checked lowering,
and matching verifier/emitter support. Parse success is never a meaning claim.
Before implementation the lead must freeze refusal priority/code, tag numbers,
continuation layout, capacity accounting, and exact probe vectors. New source
and arena totals require measurement after the representation is selected;
historical census numbers are not acceptance values for the forthcoming diff.
