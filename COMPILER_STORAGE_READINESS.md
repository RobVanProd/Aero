# Deterministic Compiler Storage Readiness

Status: CAP-040/D1 accepted as protected merge
`104d72dfb78921db68421c7ebd45e30dcbc3d804`, tree
`abd136d0cbc9066714148e0919010a697ccd350e`, 2026-08-15.

Reviewed candidate `f712800a23b622fb589d6af089b4c35b529faf90` and the protected merge
share the same tree. The focused D1 target passes 3/3 tests from D:-resident
worktree, build-target, and temporary roots. The complete root gate passes
formatting, correctness Clippy, 309 library tests, 35 binary tests, every
integration/native/system target, and doc tests. Every candidate check and
accepted-head CI `31920949979`, Rust CI `31920949994`, CodeQL `31920949457`,
and evidence `31920949972` are terminal-success.

## Decision

D1 is a bounded Aero-authored compiler-data product built entirely from the
accepted R1/R2 source surface. It deliberately does not add a collection type,
compiler intrinsic, runtime symbol, parser rule, checked instruction, backend
path, or driver behavior.

Five explicit `ByteBuffer` owners contain:

1. the complete binary input;
2. canonical name spans;
3. token records;
4. an append-only scope log; and
5. flat AST records.

Rust supplies an independent oracle and native harness only. It does not build,
mutate, validate, or traverse the Aero product's logical tables.

## Input and names

The evidence input is a sequence of one-byte lengths followed by identifier
bytes. Length is `1..=63`; the first byte is ASCII letter or underscore and the
remainder is ASCII letter, digit, or underscore. EOF is valid only between
complete entries. This is a binary bootstrap harness, not Aero text lexing,
UTF-8, file I/O, or grammar support.

The name table stores canonical `(start, length)` spans into the live input
owner. Names receive deterministic 1-based IDs on first occurrence. Repeated
names compare exact bytes and reuse the first ID. Zero is never a valid name
ID.

## Serialized tables

Every logical word is a nonnegative exact `int` serialized as four
little-endian bytes. The high byte is at most 127, keeping reconstruction within
`i32::MAX`. Table lengths must equal their checked fixed-width record counts.

| Table | Record |
|---|---|
| Names | `(start, length)` |
| Tokens | `(kind=1, start, length, name_id)` |
| Scope log | `(name_id, leaf_node_id)` |
| AST arena | `(kind, payload, left_id, right_id)` |

Reverse scope-log traversal defines the latest binding. No hash iteration,
pointer identity, host collection, or hidden allocation is an authority.

## Flat arena

Node IDs are 1-based append positions. Kind 1 is a name leaf with one valid
name ID and zero children. Kind 2 is a sequence node with zero payload and two
nonzero child IDs strictly below its own ID. Each token appends one leaf; each
token after the first then appends a sequence joining the previous root to the
new leaf. Empty input has root 0; otherwise the root is the final node.

The strict lower-ID rule makes construction and validation cycle-free without
recursive heap objects. Full traversal rejects unknown kinds, malformed
payloads, missing/forward/self children, invalid names, record-width drift, and
a noncanonical root.

## Deterministic product

The tracked product is
[`deterministic_compiler_storage.aero`](examples/compiler_storage_v0/deterministic_compiler_storage.aero).
It reads stdin, owns all five buffers, constructs and validates every table,
and compares independent expected counts, root, and checksum.

The checksum begins at 17 and applies
`(checksum * 31 + word) % 1000003` to decoded name, token, scope, and node words
in table order. Separators 991, 992, 993, and 994 follow their tables, then the
name count, token count, node count, and root. The canonical tracked fixture is
`alpha`, `beta`, `alpha`, `_x9`; it has 3 names, 4 tokens, 7 nodes, root 7,
checksum 639832, and silent native exit 91.

The product uses a small scalar read helper because R2 intentionally requires
each stdin read to directly initialize an explicit typed `Result<int, int>`.
Reusable scalar scratch bindings are declared outside repeated loops so the
current LLVM lowering does not accumulate loop-local stack allocations. This
is a product-shape constraint, not a new language or backend guarantee.

## Failure and ownership evidence

Every read, push, and get result is consumed. Invalid framing or identifier
bytes, truncation, allocation/reallocation failure, corrupt reads, malformed
records, bad IDs, or arena corruption returns a deterministic non-91 status.
All owners are compiler-dropped exactly once in reverse order on every exit.

The independent test covers empty, shadowing, maximum-length, malformed, wrong
oracle, and corrupted-child cases. Its large case contains 5,638 input bytes,
1,025 tokens, 2,049 nodes, and IDs above 255. Deterministic allocator injection
proves the exact 5 allocations, 13 reallocations, 5 deallocations, immediate
failure handling, and zero live allocations for the canonical case.

LLVM is deterministic and externally verified with LLVM 22. Native O0/O2 and
the public CPU runner return 91 with no application output. ROCm and CUDA routes
reject before artifacts. Linux and Windows workflow steps independently replay
the tracked product. The complete local and protected gates are green.

## Explicit exclusions

D1 is not a general collection API, `Vec`, `String`, owned UTF-8, a production
lexer/parser, the Rust compiler's AST replacement, recursive heap storage,
buffer transport across function boundaries, maps, file/path I/O, modules,
semantic analysis, checked-IR construction, code generation, accelerator
execution, memory-safety/stability/performance proof, release readiness, or
self-hosting.

## Next dependency

CAP-041/F1A is accepted as protected merge
`4bdfcb206f541356aa83987084a9d2feffbe511c`. It consumes accepted R2 bytes and
emits canonical name spans plus located D1-style token records. CAP-042/F1B is
the current local product-only candidate: it preserves that scanner, consumes
the retained records without re-lexing, and emits one-based lower-child D1
nodes for the frozen single-function expression grammar; see
[`AERO_FRONTEND_READINESS.md`](AERO_FRONTEND_READINESS.md). D1's framing remains
evidence scaffolding, not source syntax, and the candidate does not add a
general AST collection or replace the Rust front end.
