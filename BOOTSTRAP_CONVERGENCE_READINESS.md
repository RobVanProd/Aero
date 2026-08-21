# Bootstrap Convergence Readiness

Status: CAP-049/H1A is a locally green ingestion candidate on top of the
CAP-048/H1 contract, from accepted CAP-047/B1C merge
`0365e5c91bd503b198855b97b7f16054488d6dff`, tree
`e13bcc92f04e0f1aec44eafcfdccbe638c1405ad`. CAP-048 froze what H1 must prove and
changed documentation only. CAP-049 is the first H1 prerequisite to execute: the
Aero-authored compiler now consumes its own complete source, name, and token
streams and stops at one independently predicted parser construct. Reaching that
required one separately scoped code-generator fix, CORE-093. Neither is H1B, H1,
H2, stage convergence, or any self-hosting claim, and neither is published or
accepted yet.

## Decision

H1 will use one canonical Aero source file and an exact binary-stream compiler
interface. The accepted Rust compiler builds stage 1 once. Stage 1 and stage 2
then compile the identical Aero source bytes without invoking Rust compiler
logic. Their emitted LLVM, repeated linked artifacts, diagnostics, and bounded
compiler corpus must satisfy the comparison frozen here.

H1 is intentionally decomposed. The current Aero-authored compiler cannot read,
parse, type, verify, or lower its own source, and those are independent compiler
authorities. Each prerequisite receives a separate ledger and failing test; no
checkpoint may cross more than two compiler phases or relabel partial progress
as convergence.

## Current executable boundary

CAP-049/H1A moved the first boundary. The canonical source
[`examples/aero_self_host_v0/compiler.aero`](examples/aero_self_host_v0/compiler.aero)
— 241,918 bytes, 5,563 LF bytes, SHA-256
`977a1f3e0562f2b6507873febcdf8fd3f59b2f3a1370327c500e0bdd7e6232ad` — is a
copy-derived successor of accepted B1C differing only in three ingestion bounds,
one new lexical token kind for a lone `&`, the matching token-record validator
bound, and one quadratic-to-linear rewrite of the located-token re-derivation.
Fed its own exact bytes it now consumes all 241,918, interns 571 names, records
31,062 located token records, and then stops at the independently predicted first
unsupported parser construct: `status = 10` at offset 16, line 1, column 17,
expecting `)` and finding an identifier. That is the `result` parameter of
`fn result_value(result: Result<int, int>)` — the first construct outside the
frozen `fn NAME ( ) -> int { return` skeleton. Every downstream phase reports
not-attempted, no LLVM byte is written, and no artifact is created.

Reaching that exposed one genuine compiler defect, fixed separately as CORE-093:
the code generator emitted each value's storage slot inline, so every checked
`ByteBuffer` result temporary inside a loop became a non-entry `alloca` that LLVM
never reclaims. A loop over a `ByteBuffer` therefore grew the stack once per
iteration, and self-input terminated with `STATUS_STACK_OVERFLOW` before any
diagnostic. Every static `alloca` is now emitted in the entry block.

The next boundary is therefore the self-source grammar, not capacity: the parser
stops at its fourth token, and the whole of H1B remains.

The accepted B1C predecessor remains a real but bounded pipeline:

- the Aero source
  [`runtime_ascii_toolchain_driver.aero`](examples/aero_frontend_v0/runtime_ascii_toolchain_driver.aero)
  is 241,941 bytes, 5,564 LF-delimited lines, and 23 top-level functions;
- its runtime frontend accepts at most 8,192 source bytes, 1,024 token records,
  1,024 names, and 512 syntax nodes;
- its parser accepts one frozen function/expression grammar, its semantic and
  checked-IR phases cover only that grammar, and its verifier/emitter cover the
  corresponding one-function module;
- its host driver supplies the fixed 34-byte source
  `fn score()->int{return 1+2*3-4/2;}` and accepts one exact 144-byte LLVM
  module; and
- the Rust stage-0 compiler is still required to compile the B1C Aero source.

Therefore the first self-input failure is not a mysterious bootstrap mismatch:
the accepted compiler stops at byte 8,192 before it has consumed its 241,941
source bytes. Raising only that bound next reaches token capacity and then the
unsupported self-source grammar. Hardcoded LLVM, expected-value parameters, a
host parser, or a copied compiler image cannot close any of those gaps.

## Canonical source bundle

The final bundle is exactly one tracked 7-bit-ASCII, LF-only file:

```text
examples/aero_self_host_v0/compiler.aero
```

The new path preserves every accepted F1/M1/B1 product byte-for-byte. It starts
as a copy-derived successor of B1C and evolves only through separately accepted
H1 prerequisite checkpoints. Before final stage replay, the convergence
manifest freezes its exact byte length and SHA-256. Both stage 1 and stage 2
receive those identical bytes.

There is no include expansion, generated source, conditional host branch,
module lookup, source rewrite, symlink, network fetch, timestamp, absolute path,
or semantic bundler. Positive modules and maintainable multi-file compiler
organization remain G1 work. The single file is the entire declared bootstrap
source and trust boundary.

## Stage protocol

| Stage | Producer | Exact input | Exact output | Forbidden dependency |
|---|---|---|---|---|
| Stage 0 | Accepted Rust `aero` at the H1 base | Canonical `compiler.aero` | Verified LLVM and a linked stage-1 executable | No dirty source, alternate compiler, or unrecorded tool |
| Stage 1 | Stage-1 Aero compiler | The same canonical bytes on binary stdin | `stage2.ll` on binary stdout | No Rust lexer/parser/semantic/IR/verifier/backend call |
| Stage 2 | Linked from verified `stage2.ll` | The same canonical bytes on binary stdin | `comparison.ll` on binary stdout | No stage-specific source or hidden precompiled image |

Stage 0 is part of the declared trust base. After stage 1 starts, the host may
only capture and hash complete byte streams, enforce transaction boundaries,
invoke the explicitly named LLVM/link tools, and compare artifacts. It cannot
parse Aero, construct checked IR, emit or repair LLVM, choose a diagnostic, or
substitute expected output.

## Compiler process interface

The final compiler consumes exactly one complete source stream from binary
stdin and closes it at EOF.

On success it:

- writes only canonical 7-bit-ASCII LLVM with LF line endings to binary stdout;
- writes zero stderr bytes;
- exits zero; and
- creates no file or child process itself.

On source or compiler failure it:

- writes zero LLVM/stdout bytes;
- writes one exact frozen ASCII diagnostic to stderr;
- exits nonzero; and
- creates no artifact.

It has no source path, shell, PATH lookup, environment fallback, expected-value
parameter, first-class host callback, or process API. Accepted scalar stdin,
scalar stdout, ByteBuffer ownership, and exact cleanup remain the only transport
and storage authorities unless a later ledger explicitly widens them.

## Convergence comparison

Stage 1 emits `stage2.ll`; stage 2 emits `comparison.ll`. H1 requires:

1. byte-for-byte equality of the two complete LLVM streams;
2. independent LLVM 22.1.8 assembly/verification of both streams;
3. identical hashes and empty stderr across repeated clean stage-1 and stage-2
   compilations;
4. identical deterministic link commands, runtime object, and Clang/lld 22.1.8
   tools for both streams;
5. byte-identical linked executables across repeated builds on each platform;
6. the same valid, invalid, allocation-failure, corruption, and source-to-native
   compiler corpus results from stage 1 and stage 2; and
7. identical compiler-emitted LLVM on Linux and Windows. Platform executable
   hashes may differ, but each platform must be internally reproducible and its
   manifest must explain that platform identity.

No output prefix is comparable. Capture completes before publication. If a
linked format contains unavoidable nondeterministic metadata, H1 stops until a
field-level rule freezes the exact field, extraction tool, allowed value, and
independent check. Ignoring an entire executable, object, section, timestamp,
path, or tool output after observing a mismatch is forbidden.

## Environment and artifact manifest

Every H1 replay emits one canonical per-platform manifest containing at least:

- schema version and platform role;
- accepted commit, tree, ordered parents, and clean worktree result;
- canonical source repository path, length, SHA-256, ASCII/LF validation, and
  proof that every stage received those bytes;
- stage-0 executable path role, SHA-256, version, and exact accepted source
  identity;
- Rust/Cargo identity used only for stage 0;
- LLVM assembler, optimizer when used, Clang, and lld 22.1.8 path roles,
  executable hashes, and complete version banners;
- runtime C source and compiled-object hashes;
- OS, architecture, locale, timezone, code-page/binary-stream settings, and the
  exact allowed environment-variable set;
- every command, argv element, deterministic flag, input hash, output hash,
  output size, exit code, stdout hash, and stderr hash;
- raw LLVM, bitcode, object, executable, and corpus-result identities; and
- transaction-relative artifact names plus cleanup result.

Absolute D: paths are recorded only as abstract roles so relocation does not
change compiler input. Every task-created worktree, target, temp file, stage,
manifest, native artifact, and log nevertheless resides physically on D:.
Network access and undeclared executables are absent.

## Trust base

H1 declares rather than hides:

- the exact accepted Rust stage-0 `aero` executable;
- the accepted Aero C runtime and its C compiler invocation;
- LLVM assembler and any optimizer used, Clang, lld, and operating-system
  executable loading/stream interfaces;
- the nonsemantic Rust convergence orchestrator that captures, authenticates,
  invokes, and compares but never parses or compiles Aero; and
- the tracked canonical source and conformance corpus.

The stage-1 and stage-2 compiler decisions—lexing, parsing, semantic analysis,
checked-IR construction, verification, diagnostics, and LLVM emission—must be
Aero-authored and exercised. A Rust helper behind an Aero wrapper fails H1.

## Required compiler corpus

Both stage compilers run the same frozen corpus. At minimum it contains:

- the canonical compiler source itself;
- the accepted F1/M1/B1 canonical arithmetic program and its exact native
  result;
- boundary-valid source, token, name, node, checked-record, and emitted-byte
  sizes needed by the compiler source;
- invalid byte/encoding, lexing, parsing, symbol, type, ownership, checked-IR,
  and LLVM forms with exact first diagnostics;
- deterministic allocator failure at every live allocation/reallocation edge;
- mutations that would bypass the verifier, reorder instructions, change an
  operand/type, truncate/extend output, or use a different source; and
- repeated O0/O2 Linux and Windows source-to-native execution.

The corpus grows with the self-source grammar. It does not imply support for the
whole experimental Rust compiler grammar or any public stable language.

## Ordered H1 prerequisites

| Gate | Required result | Frozen exclusions |
|---|---|---|
| H1A — canonical source ingestion — **locally green (CAP-049)** | The new compiler source's complete bytes, names, and token records are consumed under independent oracles before one exact unsupported-parser diagnostic | No grammar, semantic, IR, verifier, emitter, process, or convergence change |
| H1B — self-source syntax | The iterative Aero parser emits a validated flat AST for every construct actually present in `compiler.aero` | No type/ownership inference, checked IR, or backend widening in the parser task |
| H1C — self-source meaning | Aero semantic facts and authenticated checked IR cover the exact self-source AST with fail-before-IR negatives | Split the task whenever semantic and checked-IR authorities cannot fit in two phases |
| H1D — self-source verification and emission | Independent Aero verification accepts only the exact checked module; Aero emission produces canonical LLVM for the compiler | No host verifier or expected LLVM as admission authority |
| H1E — compiler interface and driver | The exact stdin/stdout compiler ABI and transactional stage driver execute the frozen protocol | The driver never parses source or emits LLVM |
| H1 final — convergence replay | Clean stage-1/stage-2 compilation, exact LLVM equality, deterministic artifacts, shared corpus, and Linux/Windows manifests all pass | A single stage-1 success is not convergence |

Any gate may be split into smaller red-first checkpoints. It may not absorb a
third compiler phase to preserve the table's name.

H1B's required result above is an obligation the H1B checkpoint table at
`:324-329` does not currently contain a checkpoint able to discharge. Measured
on 2026-08-18 and recorded in `TASK_LEDGER.md` under "The representation gap H1B
leaves": five of the six H1B checkpoints admit a construct **without
representing it** - the parameter, the `match` construct, the binding, the
assignment, the statement sequence, and, under CAP-053, the conditional and the
loop. Only H1B-5 is scheduled to create a node. On the complete canonical source
the accepted accounting produces 13,190 node records of which **154 are
reachable from a `root`**; 98.8% are orphans, and a representation that
satisfied the sentence above needs 23,509 nodes. Function 1 parses completely
and all four of its nodes are orphans.

The sentence above is not wrong and should not be weakened. The checkpoint table
is missing a row. Until it gets one, "H1B-6 green" means the H1B grammar is
admitted, **not** that this gate's required result is met, and an explicit
representation checkpoint should be ordered after H1B-5 and after H1B-6 rather
than absorbed into H1C - for the same reason `:367` refuses to absorb the
single-function coupling.

## Exact next checkpoint

Authorize H1B separately and red-first from the exact construct H1A stops at.
The compiler's iterative parser must emit a validated flat AST for every
construct actually present in `compiler.aero`, beginning with the typed
parameter list of `fn result_value(result: Result<int, int>) -> int`. Because the
self-source grammar is far wider than the frozen one-function skeleton — it
contains parameters, typed bindings, `if`/`else`, `while`, `match`, references,
`ByteBuffer` intrinsics, and multi-function modules — H1B must itself be split
into separately authorized red-first checkpoints, each crossing at most two
compiler authorities and each stopping at an independently predicted next
construct.

H1B may change the canonical Aero source, its focused tests, workflow replay,
ledger, and directly affected readiness documents. It must not widen type,
ownership, checked-IR, verifier, or backend authority inside the parser task, and
it must not modify the accepted B1C product or the frozen compiler process
interface.

### The self-source grammar is closed and measured

H1A's token census makes H1B's target finite rather than open-ended. The complete
canonical source uses 571 distinct names and exactly these constructs:

| Construct | Occurrences in `compiler.aero` |
|---|---|
| `fn` items | 23 |
| `let` bindings (`mut` on 473 of them) | 469 |
| `if` / `else` | 935 / 248 |
| `while` | 82 |
| `return` | 221 |
| assignment `=` | 2,756 |
| call or grouping `(` | 1,109 |
| reference `&` | 417 |
| `match` with `=>` arms | 1 (2 arms, in `result_value`) |
| declared types | `int`, `ByteBuffer`, `Result<int, int>` |

Equally important is what is absent. The source contains no `[`, `]`, `.`, `%`,
or `!` token at all, so H1B needs no array syntax, no field access, no modulo,
and no logical negation. Anything outside the table above must stay rejected.

The 23 signatures are narrower still. Every one of them returns `int`. They
declare 99 parameters in total, of which 98 are `int` and exactly one is
`Result<int, int>`; two functions take none, and the widest takes 67. No
parameter is a `ByteBuffer` or a reference — `ByteBuffer` appears only as a
local binding type, and `&`/`&mut` only as call arguments — so reference syntax
belongs to the call checkpoint, not the signature checkpoint.

### Ordered H1B checkpoints

The order below is the order the self-source *grammar* forces, and each
checkpoint is named by the construct at which the previous one stops.

Two corrections to that sentence, measured under CAP-052 and recorded here so no
later reader takes the original wording as evidence:

- It is **not** the order `compiler.aero` itself forces. Function 2 opens its
  body with `if`, so the construct the source forces after H1B-2 is control flow,
  not statements; and no function in the source has statements without control
  flow or a call, so no canonical function can parse at H1B-3 at all. The order
  still stands, on grammar dependency: an `if` or `while` body is a statement
  block, so H1B-4 cannot be specified without H1B-3, and H1B-5's call arguments
  are expressions inside statements.
- The naming rule runs out after H1B-2. CAP-051 parses function 1 completely and
  stops at the second `fn` item, which is excluded from every parser checkpoint
  below. **H1B-3, H1B-4 and H1B-5 therefore all leave the canonical
  self-ingestion stop exactly where CAP-051 put it** - offset 146, line 8,
  column 1 - and their forward evidence is focused probes only. That stop is a
  regression guard for those three checkpoints and must not be cited as progress
  by any of them.

| Checkpoint | Required result | Frozen exclusions |
|---|---|---|
| H1B-1 — typed parameter lists | The signature grammar accepts `fn NAME(p: T, ...) -> int` over the measured closed type set `int` and `Result<int, int>`. Parameters are recorded in their own bounded store and folded into the parse checksum | No syntax node is created for a parameter, because the node arena is what the semantic, checked-IR, and verifier phases count; parameters carry no type, ownership, or checked meaning; the body grammar is untouched |
| H1B-2 — `match` over `Result<int, int>` (locally green, CAP-051) | The single `Ok(...) => ..., Err(...) => ...` form the source actually uses, as `result_value`'s whole body. Dispatched on the leading token of the return expression, before the operand reduction runs, so the append-only node arena never has to retract a name-reference node. The construct creates no node and needs no new node kind, so the `1..=19` node-kind bound is unchanged | No general patterns, guards, enums, or match anywhere but a return expression |
| H1B-3 — statement blocks (locally green, CAP-052) | `let IDENT : int = EXPR ;`, `let mut IDENT : int = EXPR ;`, `IDENT = EXPR ;`, and `return EXPR ;`, in a body that is `{` followed by one or more statements followed by `}`. The skeleton's fixed `return` step is dissolved into the statement loop and `;` is demoted from a closing token to the return statement's own terminator, so the closing sequence shrinks to `}` then end-of-input with one entry point. A statement creates no syntax node, exactly as a parameter does not, so the `1..=19` node-kind bound is again unchanged | No control flow and no calls; no `ByteBuffer` or `Result<int, int>` binding type, because every one of those in the source is initialized by a call; a binding carries no type, ownership, mutability, scope, or checked meaning, and `mut` is matched and recorded rather than enforced |
| H1B-4 — control flow (locally green, CAP-053) | `if EXPR BLOCK`, with any number of `else if EXPR BLOCK` arms and an optional final `else BLOCK`, and `while EXPR BLOCK`, over the existing expression grammar. A `BLOCK` is CAP-052's statement sequence with two differences: it closes on `}` and nothing more, and the requirement that a `return` completed moves from the block to the function. Nesting is carried by a fourth bounded parse-group arena, one three-word record per nested block. Neither form creates a syntax node, so the `1..=19` node-kind bound is again unchanged. Within any block a `return` is the last statement and the only one - the rule CAP-052 froze and did not implement | No new expression forms; no `match` in a condition, because the source never writes one; no `else` without a preceding `if` body; a block carries no scope, reachability, liveness, or checked meaning, and a condition is not type-checked and is not required to be boolean |
| H1B-5 — calls and references (locally green, CAP-054) | `IDENT ( ARGS )` where the callee is an operand-position identifier immediately followed by `(`, and an argument may begin with `&` or `& mut` and may do so nowhere else. **The first H1B checkpoint that represents rather than only admits**: four node kinds take the node-kind bound from `1..=19` to `1..=23` — kind 20 the call, carrying its callee as `payload` and its argument list as `left`; kind 21 one argument-list cell; kinds 22 and 23 the two references. Open calls are carried by a fifth bounded parse-group arena, one three-word record each | No intrinsic knowledge, arity, type, ownership, borrow, or aliasing meaning; no callee that is not a bare identifier, so `(f)(a)`, `1(a)` and `f(a)(b)` are rejections; no `match` in an argument; no method, field or index syntax; no `ByteBuffer` or `Result<int, int>` binding type — see the gap below |
| H1B-6 — arena capacity (locally green, CAP-055) | **All five** parse-group record bounds — node, value, operator, block and call — raised from 512 to a uniform 65,536, under the same independent-oracle proof H1A used for tokens. The oracle carried no record ceiling of any kind before this checkpoint, so the proof required building the model rather than editing a literal. The verifier's own `512` — at `compiler.aero:5557` when CAP-055 was written and at **`:6018`** on the current tree — is untouched and recorded as debt | No grammar change; capacity only |

H1B is complete as *admitted grammar* when H1B-6 is green. It is **not**
complete in the sense of `:223`; see the representation gap recorded in
`TASK_LEDGER.md`. The gate that follows it is H1M, the module-shape gate,
described below.

Each checkpoint is separately authorized and red-first, crosses at most two
compiler authorities, and must stop at an independently predicted next construct.
H1B-6 is listed last but must be pulled earlier the moment a checkpoint's AST
exceeds 512 records; capacity is never allowed to masquerade as a grammar
failure.

The self-source requirement H1B-6 raises the bounds *to* was measured on
2026-08-18 and is recorded in `TASK_LEDGER.md` under "H1B-6 arena-capacity
measurement". Three results from it belong here because they change how this
paragraph should be read.

- The requirement is **13,190 node records, 13,144 value records and 4,157
  operator records** as a measured floor that no design choice can reduce, and
  **23,509 / 14,697 / 5,710** once the shapes H1B-4 and H1B-5 admit are costed;
  512 is exceeded by between 8x and 51x. A uniform bound of 65,536 is
  recommended, and costs nothing until used, because every record array is
  created by `bytes_new()` and grows by append rather than being preallocated.
- `value_records` and `operator_records` are **not stack depths**. Neither is
  ever decremented, so each counts every push over the whole parse. The deepest
  either stack actually gets on the complete canonical source is 5. "512 value
  records" has never been a limit on expression complexity.
- **H1B-6's bound list is five, not three.** CAP-053 added a block record store
  and CAP-054 a call record store, each with the same never-decremented shape
  and the same 512 bound, and both canonical requirements are already measured:
  **1,289 block records** cumulative at a peak live depth of 10, and **1,120
  call records** cumulative at a peak live depth of 3. Like the other three
  neither can be reached by any H1B-4 or H1B-5 probe. Measured on the source
  CAP-054 left at 293,592 bytes, the five requirements are **17,621 node,
  15,842 value, 6,030 operator, 1,289 block and 1,120 call records**.
- The pull-forward rule above does **not** fire at H1B-4 or H1B-5. Both leave
  the canonical stop at offset 146 with four nodes and are proven by focused
  probes of a few dozen tokens, so neither can exceed 512. **H1B-6 should be
  pulled ahead of the module-shape gate, not ahead of H1B-4 or H1B-5.**
- **Corrected under CAP-056/H1M-1, and left visible rather than restated.** This
  bullet used to say the rule fires at the module-shape gate "at once", with the
  node arena exhausted inside function 8 at line 154 of 6,085. Measured against
  the built product, it does not fire there either. Two causes, both recorded
  in `TASK_LEDGER.md` under CAP-056 Decision 4. The 512-at-line-154 figure was
  computed under the measurement's *projected* node policy — one node per
  statement, per conditional, per loop and per sequence element — which the
  product does not implement and which CAP-053 declined to implement; under the
  policy the product has, the cumulative node count at the end of function 7 is
  **325**. And the prediction assumed the canonical run would continue past line
  232, which the `int`-only binding type refuses. The canonical run at H1M-1
  holds **486 node, 449 value, 169 operator, 54 block and 9 call records**,
  which is inside the *replaced* 512 bound by 26 records and is 0.74% of the
  raised one. That fit is a fact about where the parse stops, not about
  capacity: the prefix is 1.74% of the source's bytes and 2.76% of its 17,621
  node records, and the nine functions past the stop carry the other 97.2%. The
  ratio between the whole-source projection and this partial actual is two
  different quantities and must not be read as the projection overshooting; see
  the CAP-056 outcome in `TASK_LEDGER.md`. H1B-6 was still ordered correctly — without it the checkpoint that
  admits the two binding types would exhaust 512 inside function 22 — but its
  stated trigger fires **there**, not at module shape.

  **Corrected under CAP-057, and left visible rather than restated.** "Inside
  function 22" is right for three of the five arenas and wrong for the one that
  fires first, so the parse would never reach function 22 at all. Measured on
  the current source with the instrument recorded under CAP-057, 512 is first
  crossed on the **node** arena inside item 16, `binary_precedence` (492 → 547);
  on the value arena inside item 17 (505 → 558); and on the operator, block and
  call arenas inside item 22 (350 → 6,043, 181 → 1,293 and 13 → 1,119). The node
  arena fires six functions earlier than this bullet states, and it is the one
  that governs. The conclusion — that H1B-6 was ordered correctly and that its
  trigger fires at the binding-type checkpoint rather than at module shape — is
  unchanged, and is in fact strengthened.

  A second staleness, recorded in the same place. The five-arena requirement
  this document states as **17,621 / 15,842 / 6,030 / 1,289 / 1,120** holds for
  the 293,592-byte source at `466701c`, where CAP-057's instrument reproduces
  all five exactly. CAP-056 then added 2,926 bytes to `compiler.aero`, which
  *is* the measured source, and the requirement for the current 296,584-byte
  tree is **17,700 / 15,921 / 6,051 / 1,293 / 1,120**. Any record citing the
  five-arena requirement should name the tree it holds for; the self-source
  grows with every checkpoint that edits the parser.

One qualification, so it is not discovered mid-checkpoint, **corrected under
CAP-054 and left visible rather than quietly restated**. This paragraph used to
say that `emitter_fixed_byte` needs 474 nodes under the current node policy and
that lifting it verbatim as a probe is the single way an H1B-4 or H1B-5 probe
could reach the bound. It needs **394**, and the sentence that followed is
false. The function is byte-identical between `f416067` and `7b0e929`, so the
figures are directly comparable, and 394 is derivable in one line from its own
token histogram without any parser model: 106 identifier tokens less 6 signature
identifiers, 181 integer leaves, 111 binary operators, no prefix operator, one
kind-18 return node and one kind-19 function node. Measured per function under
CAP-054's policy, **the 21 canonical functions with no `ByteBuffer` or
`Result<int, int>` binding all need at most 394 nodes**, so no canonical
function lifted verbatim can reach the bound at H1B-4 or H1B-5. The only
function above it is `run_runtime_ascii_llvm_emitter` at 15,553, and it carries
17 non-`int` bindings, so it is not parseable at either checkpoint at all.

### A second construct the checkpoint table does not own

Recorded under CAP-054 and placed beside the representation gap above, because
it has the same shape: a construct the source contains that no row of the table
admits.

CAP-052 excluded the `ByteBuffer` and `Result<int, int>` binding types with an
explicit reason — "every one of those in the source is initialized by a call".
**CAP-054 removed that reason and deliberately did not act on it**, because
admitting a binding type is statement-grammar work rather than "calls and
references". So 16 `ByteBuffer` bindings and 2 `Result<int, int>` bindings
remain inadmissible, they are confined to `read_input_value` and
`run_runtime_ascii_llvm_emitter`, and no checkpoint in the table admits them.

**Corrected under CAP-057.** That is 18 and this document says 19 sites two
paragraphs earlier; both cannot be right. The current source carries **17
`ByteBuffer` and 2 `Result<int, int>`** bindings, which is the 19. The 16
predates CAP-054, whose `calls` arena added the seventeenth at
`compiler.aero:521`. Enumerated: `:232`, `:515-531` and `:6761`. The confinement
to two functions is unchanged, and CAP-057 is the checkpoint that admits them.

The consequence is concrete and it costs evidence. All 451 `&` and `&mut`
operands in the source live in `run_runtime_ascii_llvm_emitter`, so **no
canonical function containing a reference can be lifted verbatim as a probe at
H1B-5**; references are proven by hand-written probes only. Admitting the two
binding types is small, precisely scoped, and is what unlocks the source's
largest function as canonical evidence.

### The single-function coupling must be split out, not absorbed

One boundary in this table is not the parser's alone. The accepted semantic,
checked-IR, verifier, and emitter phases all assume exactly one function: they
require `root == node_count`, one symbol, one fact per node, and one emitted
module body. The canonical source has 23 functions. Admitting a second `fn` item
therefore changes four downstream authorities at once and must not be smuggled
into a parser checkpoint. It gets its own ordered gate — module shape before
meaning — authorized only after H1B-1 through H1B-5 have proven the parser can
describe a single function completely. Until then, every checkpoint stops at the
second `fn` item, and that stop is the expected result rather than a defect.

**Corrected under CAP-056/H1M-1.** "Until then" ended at H1M-1, which is the
checkpoint that admits the second and subsequent `fn` item and the first to move
the canonical stop since CAP-051 set it. The sentence above is true of H1B-1
through H1B-6 and of nothing after them. From H1M-1 the canonical run parses
fourteen complete function items and stops at line 232, column 15, offset 5,203,
on `Result` in `let read: Result<int, int> = stdin_read_byte();` — the `int`-only
binding type CAP-052 froze, which is now the **only** construct in the whole
296,584-byte source that the accepted grammar plus module shape does not admit -
19 sites, one in `read_input_value` and eighteen in
`run_runtime_ascii_llvm_emitter`.

**Corrected again under CAP-057/H1M-1b, and left visible rather than
restated.** The paragraph above is true of H1M-1 and of nothing after it.
From CAP-057 the canonical run does not stop in the parser at all: it
consumes all 300,471 bytes, reaches `status = 0` with `root = node_count`
and 23 linked items, and is refused one phase later by the semantic group
at `semantic_status = 17` / `semantic_code = 2`, node 1, offset 98, line 3,
column 22. **There is no longer a canonical parser stop.** What pins the
canonical run is now the complete-parse vector, the walked 23-item chain,
and that relocated refusal - all three asserted, and the refusal predicted
and not modified.

### The module-shape gate, H1M

The gate is labelled **H1M** and split into three checkpoints, because it meets
five downstream authorities and `:331` caps a checkpoint at two. The split is
derived in `TASK_LEDGER.md` under CAP-056 rather than chosen: a parse-group
refusal is refuted by `compiler.aero:3680`, which requires `root == 0` whenever
`status != 0` and so would destroy the very result the gate exists to produce.
What refuses a multi-item module instead is the **downstream phases' own
already-implemented refusals**, which each checkpoint predicts and asserts and
none edits.

| Checkpoint | Required result | Frozen exclusions |
|---|---|---|
| H1M-1 — module item list (locally green from 12:40 on 2026-08-19, CAP-056; see the retraction below for why that time is stated) | A module is one or more `fn` items. A function item closes at its own `}` and the module then takes another `fn` item or end-of-input. The item list is **represented**: a kind-19 node's `right`, previously required to be `0`, carries the previous item's node id, so every item is reachable from `root` and `root == node_count` is preserved exactly. Crosses the parse group only. The semantic phase refuses a multi-item module with `semantic_status = 27` / `semantic_code = 3` where the module has no identifier, and with `semantic_status = 17` / `semantic_code = 2` where it has one — both predicted and asserted, neither modified | No new node kind; `1..=23` unchanged. No capacity change; the five parse-group bounds stay at 65,536 and the verifier's `512` stays at 512. No grammar change inside a function body. No `ByteBuffer` or `Result<int, int>` binding type. No zero-item module. No claim that the canonical source parses |
| H1M-2 — module meaning (**locally green across both stages**, CAP-058) | The semantic and checked-IR groups over N function items: N symbols, one fact per node, and the `node_count - 2` arithmetic at `compiler.aero:4480` generalized. Crosses two authorities. The verifier refuses a multi-function checked module by authentication; predicted and asserted, not modified. **Stage 2a implements the semantic group alone** — N symbols in source order, the kind-19 rule as a chain rule cross-checked against the symbol record, and `N` / `16N` for the module invariant — and a multi-item module is then refused by the checked-IR group's own `symbol_count != 1` at `compiler.aero:4583`, which is predicted and unmodified, so stage 2a crosses **one** authority. **Stage 2b implements the checked-IR group** — C1 through C8 generalized, one loop over every node, a placeholder value record for the two kinds that are not expressions, one Return per item, a `9 + 16N`-word header, and a result loop that scans rather than indexes — and the refusal relocates to the verifier, where `:5555` — **`:5878` on the current tree** — refuses `verified_function_count != 1` with `verified_status = 1`, `verified_word_index = 1`, `verified_code = 2`, `verified_expected = 1` and **`verified_actual = N`**. That vector was predicted before the product was edited and observed afterwards, and it is what pins the checkpoint | Not authorized here |
| H1M-3 — module verification and emission (**contracted, CAP-059**) | The verifier and emitter groups over N function items. Crosses two authorities. Takes ownership of the verifier's `512` — which is at `compiler.aero:6018` on the current tree, not `:5557`; see the correction below — or keeps every probe under 512 nodes, explicitly rather than by discovery. **CAP-059 contracts it and takes ownership**, raising the pair to `3..=65,536` from the verifier's own derivation rather than by analogy to CAP-055: the value is a parse-group node id, the parse group refuses an append at `node_count >= 65536` and issues `node_id = node_count` immediately afterwards, so 65,536 is the largest id a well-formed producer can emit. **Three corrections to this row's own scope follow from that contract.** It names one thing to generalize and a transcription of both groups finds **thirty-two** — twenty-five verifier sites and seven emitter sites — of which three are not consequences of "N function records": the verifier's result-to-instruction positional assumptions at `:6212`, `:6613` and `:6623`, which are stage 2b's C7 one authority down; the `verified_is_last` positional claim at `:6191`, which refuses every Return but the module's last as an illegal opcode; and the emitter's register-naming coincidence at `:6904` and `:7002`, **whose failure mode is not a refusal at all** — it emits well-formed LLVM referencing an undefined register and exits `91`, which no exit code in this product reports. The row also does not say what pins the checkpoint once the located-refusal chain ends, and the chain does end here: below the emitter the B1C driver makes only internal authentication checks and refuses no module shape. CAP-059's Decision 1 answers it, naming the emitted LLVM bytes load-bearing and grading two alternatives against them. And the row is silent on the `entry_function = N` default, which CAP-059's Decision 5 evidences by deriving rather than replaces | Not authorized beyond what CAP-059 contracts |

**H1M-2 is contracted under CAP-058 and implemented across both stages; see
`TASK_LEDGER.md`.** Stage 2a discharges the three semantic single-function
assumptions below — S1, S2 and S3 — and stage 2b discharges the eight checked-IR
ones, including `node_count - 2`, which this row names and which is now
`node_count - 2N`. A fourth correction to this row follows from
implementing it: the contract's own out-of-table grading predicts that CAP-056's
semantic model is undefined on probes D, E and F for carrying node kinds it has
never met, and it is undefined only on D, because that model returns at item 1's
function node before it reaches item 2's operators. Three corrections to this document's own H1M-2 row follow
from that contract and are recorded here rather than restated in it. The row
names three things to generalize - "N symbols, one fact per node, and the
`node_count - 2` arithmetic" - and a transcription of the semantic and
checked-IR groups against the current source finds **eleven** single-function
assumptions — three semantic and eight checked-IR — of which two are not
consequences of the three named. (**Eleven corrects an "eight" this paragraph
carried until stage 2b.** Eight is the size of the checked-IR table alone and
was promoted here to a total of both groups; stage 2a then computed `8 - 3 = 5`
from it and reported five open checked-IR sites where its own table showed
eight. The correction is recorded in full in `TASK_LEDGER.md`.) The two
unnamed ones are: the
result-derivation loop at `compiler.aero:5229-5257` assumes result `i` is
instruction record `i`, which fails as soon as per-item Return instructions
interleave, and the `instructions == results + 1` invariant at `:5305` counts
the one Return. The row also says "the `node_count - 2` arithmetic at
`compiler.aero:4480`"; the arithmetic is at **`:4619`** on the current tree, and
`:4480` names a line inside the semantic checksum. And the row's claim that "the
verifier refuses a multi-function checked module by authentication" is correct
and can now be stated exactly: `:5555`, **`:5878` on the current tree**, refuses `verified_function_count != 1`
with `verified_status = 1`, `verified_word_index = 1`, `verified_code = 2`,
`verified_expected = 1` and `verified_actual = N`, before it consults any other
header word.

One property of H1M-2 is not shared by any checkpoint before it and is recorded
because it changes what evidence is available. **The canonical source cannot
demonstrate H1M-2's capability at all.** The semantic phase refuses every kind-2
identifier node at `:4173-4216`, before the fact loop runs; the canonical
source's node 1 is such a node; and the two passes H1M-2 generalizes are symbol
emission and the fact loop, which sit either side of that refusal. So the
canonical run is refused before either generalization can be observed. Its role
at this checkpoint is a **negative control** - a 23-item, 17,985-node stress
input whose located refusal at node 1, offset 98, line 3, column 22 must be
unchanged - and the capability itself is evidenced only by hand-written
identifier-free multi-item probes.

H1M-1 does not complete the gate and does not make the canonical source parse.
The construct that does is the `ByteBuffer` / `Result<int, int>` binding type at
19 sites in two functions, and it is the last grammar work before the canonical
source parses end to end.

That work is **implemented** as CAP-057/H1M-1b; see the outcome in
`TASK_LEDGER.md`. It was contracted as **CAP-057/H1M-1b**, authored and
not implemented. It is numbered outside the H1M sequence deliberately: it is
statement-grammar work that crosses the parse group alone, and it neither
admits module shape nor touches meaning, verification or emission. Three
properties recorded there are not shared by any checkpoint before it — it is the
first to exercise CAP-055's raised bounds at scale, it is the one after which
the canonical stop no longer exists and must be replaced as the pinning
assertion, and it is the first whose own edit falls inside the region its
canonical run measures, because `compiler.aero` is both the product and the
source. The last of these makes its acceptance figure a procedure rather than a
frozen number, and the contract says so rather than freezing one.

All three held. The raised bounds were exercised: the whole-module
requirement is **17,985 node, 16,158 value, 6,165 operator, 1,302 block and
1,152 call records**, 27.4% of the raised bound and 35.1x the one it
replaced, so at 512 the parse cannot complete on any of the five. The
canonical stop was replaced before anything relied on it. And the
acceptance figure was derived from the diff rather than frozen ahead of it:
the pre-edit projection of 17,700 / 15,921 / 6,051 / 1,293 / 1,120 was
reproduced exactly by an independent instrument, a delta of
+285 / +237 / +114 / +9 / +32 was hand-derived from the diff, and the sum
matched the model and then the linked product on all five arenas at `-O0`
and `-O2`. The contract's own byte-proportional estimate of 27-81 nodes did
**not** survive: 3,887 bytes cost 285 nodes, because node cost tracks
expression structure and not bytes. Census: 240 reachable of 17,985,
98.665% orphaned, comparable to no earlier figure in this project.

#### Retraction: this document asserted H1M-1 green before any run said so

Recorded in place rather than overwritten, on the template CAP-055 set at
`1efc041`. The H1M-1 row above originally read "(locally green, CAP-056)". That
label was written before 10:52 on 2026-08-19, while the first focused run on the
changed product was still executing and **no** completed exit status existed for
the changed tree at all; the file's later 10:58 timestamp is a one-line
byte-count fix, not the origin of the claim. The full run record is tabulated in
`PROJECT_STATE.md` under the matching retraction.

What is actually evidenced, and what is not:

- **Evidenced.** The focused target `self_host_source_ingestion_tests` returned
  **45 passed, 0 failed, exit 0**, completed 10:57 and read from its own log, on
  `compiler.aero` SHA-256 `a839ff37…` and the focused test file SHA-256
  `082b9e0d…` — the exact tree, confirmed by the assertion strings being present
  in the linked test binary rather than assumed from timestamps. Every
  behavioural figure this document records for H1M-1 — the canonical stop, the
  five arena counts, the item chain, the downstream refusals — is covered by
  that run.
- **Not evidenced at the time of writing.** "Locally green" on this project
  means the complete repository-root gate. Attempt 1 returned **exit 101** at
  11:10, on an environment fault rather than a product one: `clang` could not
  write intermediate objects because `C:` was full, and `TMPDIR` does not
  redirect it — `TMP`/`TEMP` do. Attempt 2 returned **exit 0** at 11:55, and
  attempt 3, on the tree carrying these corrections, returned **exit 0** at
  12:40 with 998 passed, 0 failed, 16 ignored. The row above therefore carries a
  green from 12:40 and names that time, because the same row asserted the same
  status from before 10:52 with nothing behind it.

The rule this breaks is `TASK_LEDGER.md`'s *"A **ledger entry** must be written
after reading a completed exit status, never before."* It held exactly where it
was named — `TASK_LEDGER.md` claimed no CAP-056 result — and broke in the two
files it did not name. **Restated: any record, not only the ledger.**

#### Recorded debt this gate inherits: the verifier's `512`

CAP-055/H1B-6 raised **five** record bounds — node, value, operator, block and
call — from 512 to 65,536, and deliberately left a sixth `512` in place:
`compiler.aero:5557` requires `verified_function_node` to be within `3..=512`
and `:5561` reports `verified_expected = 512`. **Both line numbers are stale and
are corrected here rather than carried forward a fourth time: on the current
tree the pair is at `:6018` and the report at `:6022`.** CAP-059 takes ownership
of it and raises it to `3..=65,536`; see the H1M-3 row above and
`TASK_LEDGER.md`. It is now the **only** `512` left
in the product, and a test asserts that by exact list, so its survival is a
decision rather than an oversight.

It was left because it is not H1B's to widen. It constrains the verifier group,
which `:265-267` forbids the parser task to touch, and it cannot bite inside H1B
at all, because the verifier runs only on a complete `status == 0` pipeline that
no H1B checkpoint reaches. Raising a fourth — now sixth — number merely because
it shares a literal with the parse-group five is the specific mistake
`TASK_LEDGER.md`'s arena-capacity measurement warns against.

The debt is recorded **here**, at the gate that will meet it, rather than in a
paragraph about capacity. Two things about when it fires:

- It does **not** fire at module shape itself, which is a parse gate and does
  not reach the verifier. Module shape can be built without touching it.
- It fires at the first checkpoint that drives a complete pipeline over the real
  source, which is H1C/H1D, and it fires hard. A single canonical function,
  `run_runtime_ascii_llvm_emitter`, needs **16,355** nodes on its own against a
  bound of 512 — a factor of **31.9** — and the whole module needs 17,621, a
  factor of 34.4. 15 of the 23 functions are smaller only because they are
  smaller, not because anything caps them.

  **Corrected under CAP-056/H1M-1.** This bullet used to state the overrun as
  "over 12,000 nodes … a factor of 24". That figure is the arena-capacity
  measurement's *floor* policy on the smaller `f416067` source and predates
  CAP-054's call representation. Nothing depends on the difference and the
  conclusion is unchanged; the larger number is the true one and is recorded so
  the smaller is not cited later. CAP-056 additionally confirms that the debt
  does not fire at **any** of H1M-1, H1M-2 or H1M-3 over the canonical source:
  H1M-1 stops at `status = 12`, H1M-2's multi-function checked module is refused
  by authentication before any bound is consulted, and H1M-3 reaches the
  verifier only over hand-written probes.

Whichever checkpoint owns it should raise it under the verifier group's own
authority and with the verifier group's own independent-oracle proof, not by
inheriting CAP-055's.

## Explicit non-claims

CAP-048 is a contract. CAP-049/H1A is ingestion and tokenization capacity only —
the compiler reads its own source, it does not understand it. CORE-093 is a
code-generator stack-use fix. None of them is stage convergence, replacement of
the Rust compiler, H1 completion, H2 self-hosting, general modules, stable
syntax/ABI, memory safety, optimization correctness, performance, packaging,
release readiness, or CPU/ROCm/CUDA parity.
