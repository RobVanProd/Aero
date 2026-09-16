# Connected AST for the canonical Aero compiler

Task: `SELFHOST-AST-DESIGN-001`. **Design only; no executable capability or
test result is claimed.** The lead reviewed the representation on 2026-09-16;
the precise decisions below supersede the initial proposal. Implementation
still requires its own ledger entry and a failing native regression. CAP-059's
full gate must finish before AST product edits.

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

## Node records

Keep four nonnegative words `[kind, payload, left, right]`, one-based IDs, and
zero for absent links. Every nonzero node link points to an earlier node.
Existing kinds 1-23 retain their meanings; the table defines additions.

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
Do not construct wrappers and then abandon them. A Return's semicolon stores
its expression and exact return-token origin. Append the Return only after its
own block's closing brace is validated. The admitted grammar already requires
Return to be terminal in its block, so nothing can consume it before that
close. This preserves the compact predecessor's successful and malformed-input
append ordering. Parameter nodes and annotations are created only for actual
parameters. Rich partial-parse products necessarily acquire their new nodes;
preserve syntax error priority and location, not obsolete rich arena counts.

## Parser continuation and origin changes

State 2 attaches parameter nodes to a per-function tail, reset at each `fn`.
State 45 captures statement origin and assignment name; state 47 retains let
name, mutability and annotation. State 18 routes a completed expression to its
statement, condition or match arm. State 49 appends ordinary statements rather
than overwriting `body_root`. States 21-23 finalize the body and module item.

The three-word block frame in states 52-54 becomes a 20-word immutable
continuation record. Field indices are:

| Fields | Meaning |
|---|---|
| 0 | Construct: 1=If, 2=While |
| 1 | Previous active continuation or 0 |
| 2 | Resume: 0=complete enclosing statement, 1=complete parent's else-if child |
| 3 | Phase: 1=then/while body, 2=await else, 3=else body, 4=await else-if child |
| 4–7 | Parent block state, statement count, held first statement, cell tail |
| 8–10 | Parent opening-brace offset, line, column |
| 11–14 | Parent pending Return expression, return offset, line, column |
| 15–16 | Condition node and completed then-block node (or 0) |
| 17–19 | Construct's original if/while offset, line, column |

Push only after condition and opening brace validate; reset the active child
block registers. Complete the child Block before restoring every saved parent
register. While completion immediately completes the parent statement. An If
appends a phase-2 replacement record and reads, without consuming, its possible
else token. A non-else token is reused by parent statement dispatch. An else
body uses phase 3. Else-if uses phase 4 with child resume 1, returning its If
node directly to the parent's branch pair without an enclosing statement cell.
Several else-if frames may unwind using the same unconsumed lookahead.

Every physical replacement record consumes the existing 65,536-record capacity;
replacement does not increase logical depth. The maximum control store is
5,242,880 bytes. Reset transient statement/expression dispatch registers after
restoration; this grammar cannot suspend an enclosing expression across a
control-flow block. These are work records, not AST nodes; no node is back-patched.

States 41/43 retain the match subject, constructor/binder names and locations,
and each completed arm root. Append the scrutinee identifier before either arm,
each binder at its token, and each arm at its comma before the next binder or
expression. At the closing brace append the arm-sequence and Match nodes.
Never reuse the second arm's expression as the whole match.

Every AST node receives one authenticated origin. Retain exact declaration and
binder locations, not the interner's first occurrence of their name. Structural
anchors are explicit: Block uses `{`, descriptor uses `fn`, branch pair uses
`if`, statement cell shares its statement's leading token, arm sequence uses
`match`. Validate these role-dependent mappings against source and tokens;
synthetic wrappers do not have one universal token kind. Keep node/origin counts
equal. Extend the validators near lines 3730-3826 and 3980-4059 accordingly.

Authenticate new name payloads against the specific declaration/binder token,
not merely its token kind or the interner's first occurrence. Type annotations
authenticate their full allowed spelling. Let nodes use the leading let token;
the adjacent optional mut token and following name retain and authenticate the
precise declaration location. Statement cells copy their child's exact origin.
Branch pairs share their owning If's origin; descriptors share their Function's
origin. Match and arm-sequence share the exact identifier spelled `match`.

## Linear connectivity authentication

The append order above forms contiguous postorder occurrence subtrees. Maintain
an append-only start-index store with one i32 per AST node, bounded by 65,536
words (262,144 bytes). It needs no mutable bitmap or quadratic ownership scan.
First validate payload domains, legal child kinds and strictly backward edges.
Then enforce the following independently of logical left/right ordering:

- Leaf: its start is its own ID.
- One child: the child ID must equal this ID minus one; inherit its start.
- Two children: IDs must differ. The later child's ID must equal this ID minus
  one; the earlier child's ID must equal the later child's stored start minus
  one. Inherit the earlier child's start.
- Complete module: root equals node count and its stored start is one.

These local interval checks and the root condition prove that every occurrence
belongs to the module exactly once. Reversed parameter/module links and
reverse-built argument cells retain their semantic ordering; only the interval
check orders the two child IDs numerically. Missing nodes, duplicated ownership,
overlapping subtrees and gaps are rejected before meaning or checked IR.

## Located refusal before meaning

Preserve parser/structural authentication, complete origin authentication,
function-symbol construction and the existing identifier-use scan in that order.
Then scan new kinds 24–38 in ascending node order before inference. If found,
report semantic status 28, code equal to that first node's kind, at its exact
authenticated origin; facts remain empty and downstream phases unattempted.
Identifier refusal remains status 17/code 2 and retains priority. Truthful match
structure changes the canonical first use to scrutinee node 3 at offset 68,
line 2, column 18; preserving its former location would conceal the new syntax.

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
Before implementation the lead must freeze exact probe vectors and observable
native AST capture. New source and arena totals require an independent derivation
from this representation before native replay; historical census numbers are
not acceptance values for the forthcoming diff.
