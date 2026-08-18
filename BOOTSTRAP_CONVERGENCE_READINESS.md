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
| H1B-4 — control flow | `if` / `else if` / `else` and `while` over the existing expression grammar | No new expression forms |
| H1B-5 — calls and references | Call expressions with argument lists and `&`/`&mut` operands | No intrinsic knowledge; a call is a syntax node |
| H1B-6 — arena capacity | Node, value, and operator record bounds raised from 512 to the measured self-source requirement, under the same independent-oracle proof H1A used for tokens | No grammar change; capacity only |

Each checkpoint is separately authorized and red-first, crosses at most two
compiler authorities, and must stop at an independently predicted next construct.
H1B-6 is listed last but must be pulled earlier the moment a checkpoint's AST
exceeds 512 records; capacity is never allowed to masquerade as a grammar
failure.

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

## Explicit non-claims

CAP-048 is a contract. CAP-049/H1A is ingestion and tokenization capacity only —
the compiler reads its own source, it does not understand it. CORE-093 is a
code-generator stack-use fix. None of them is stage convergence, replacement of
the Rust compiler, H1 completion, H2 self-hosting, general modules, stable
syntax/ABI, memory safety, optimization correctness, performance, packaging,
release readiness, or CPU/ROCm/CUDA parity.
