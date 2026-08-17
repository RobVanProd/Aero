# Bootstrap Convergence Readiness

Status: CAP-048/H1 locally green contract candidate from accepted CAP-047/B1C merge
`0365e5c91bd503b198855b97b7f16054488d6dff`, tree
`e13bcc92f04e0f1aec44eafcfdccbe638c1405ad`, on 2026-08-16. Reviewed B1C
candidate `18a507c8fabfc79e24167c79bef516b531506914` has the identical tree;
protected PR #89 and all candidate and accepted-head workflows are green.
CAP-048 changes documentation only. Its focused claim/governance checks and
complete D:-redirected repository gate are green. It freezes what H1 must prove
before the first convergence implementation is accepted; it does not claim H1
or H2.

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

Accepted B1C proves a real but bounded pipeline:

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
| H1A — canonical source ingestion | The new compiler source's complete bytes, names, and token records are consumed under independent oracles before one exact unsupported-parser diagnostic | No grammar, semantic, IR, verifier, emitter, process, or convergence change |
| H1B — self-source syntax | The iterative Aero parser emits a validated flat AST for every construct actually present in `compiler.aero` | No type/ownership inference, checked IR, or backend widening in the parser task |
| H1C — self-source meaning | Aero semantic facts and authenticated checked IR cover the exact self-source AST with fail-before-IR negatives | Split the task whenever semantic and checked-IR authorities cannot fit in two phases |
| H1D — self-source verification and emission | Independent Aero verification accepts only the exact checked module; Aero emission produces canonical LLVM for the compiler | No host verifier or expected LLVM as admission authority |
| H1E — compiler interface and driver | The exact stdin/stdout compiler ABI and transactional stage driver execute the frozen protocol | The driver never parses source or emits LLVM |
| H1 final — convergence replay | Clean stage-1/stage-2 compilation, exact LLVM equality, deterministic artifacts, shared corpus, and Linux/Windows manifests all pass | A single stage-1 success is not convergence |

Any gate may be split into smaller red-first checkpoints. It may not absorb a
third compiler phase to preserve the table's name.

## Exact next checkpoint

Authorize H1A from accepted B1C plus this reviewed contract. Add
`examples/aero_self_host_v0/compiler.aero` as a copy-derived successor, an
independent source/token oracle, and one focused test. The red checkpoint must
show the accepted 8,192-byte boundary is the first failure when the compiler is
given the canonical self-source candidate. The green result must prove complete
source and token ingestion and then stop at the same independently predicted
first unsupported parser construct.

H1A may change only the new successor Aero source, its focused test, workflow
replay, ledger, and directly affected readiness documents. It must not modify
the Rust compiler, runtime ABI, accepted B1C product, grammar, semantic facts,
checked IR, verifier, LLVM emitter, host driver, manifest format, or claims.

## Explicit non-claims

CAP-048 is a contract. H1A will be ingestion capacity. Neither is stage
convergence, replacement of the Rust compiler, H1 completion, H2 self-hosting,
general modules, stable syntax/ABI, memory safety, optimization correctness,
performance, packaging, release readiness, or CPU/ROCm/CUDA parity.
