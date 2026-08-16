# Aero Front-End Readiness

Last reviewed: 2026-08-15 (America/New_York)

This record tracks the bounded route from accepted runtime bytes and compiler
storage to an Aero-authored front end. It does not redefine the language. The
Rust lexer and parser remain the accepted stage-0 authorities until the full F1
gate is independently closed.

## Current decision

CAP-041/F1A is a locally green product-only candidate based on accepted D1
merge `104d72dfb78921db68421c7ebd45e30dcbc3d804`, tree
`abd136d0cbc9066714148e0919010a697ccd350e`. It changes no compiler production
or runtime file. One Aero-authored program consumes accepted R2 binary stdin,
retains the complete source in an accepted ByteBuffer, interns canonical name
spans, and emits located serialized token records through two further accepted
ByteBuffer owners.

F1 is not closed. CAP-041 deliberately supplies only F1A runtime lexing. A
separate ledger-first F1B parser must consume the frozen token records and emit
D1 flat AST records under a frozen grammar and diagnostic contract. Only after
that composed product passes differential, malformed-source, determinism,
failure, and protected replay gates may F1 advance toward M1.

## Frozen F1A product

- Encoding is raw 7-bit ASCII from stdin, bounded at 8,192 bytes.
- Byte offsets are zero-based; line and column are one-based. LF advances the
  line and resets the next column to one. CR, tab, and space advance one column.
- The token vocabulary is K1's exact 0-through-36 vocabulary: EOF,
  identifiers, decimal integers, the nine selected keywords, delimiters, and
  arithmetic/comparison/logical/arrow operators.
- Spaces and line terminators, `//` comments, and nonnested `/* ... */`
  comments are skipped. Maximal munch applies to every accepted two-byte token
  and comment opener.
- Identifiers are ASCII `[A-Za-z_][A-Za-z0-9_]*`, at most 63 bytes. Ordinary
  identifiers use deterministic 1-based NameIds from their first exact span;
  keywords and non-name tokens use zero.
- Name records are `(start, length)`. Token records are
  `(kind, start, length, line, column, name_id)`. Every logical word is four
  explicit little-endian bytes, preserving D1's nonnegative i32 contract.
- Up to 1,024 real tokens and 1,024 unique names are admitted. Success appends
  one located EOF token. Errors preserve completed logical records and omit EOF.
- Status and source location distinguish input I/O, input length, non-ASCII,
  unsupported syntax or unterminated block comments, identifier length, token
  capacity, name capacity, allocation failure, and internal corruption.
- A deterministic checksum covers retained source bytes, decoded name/token
  records, status/location, and counts. The tracked main returns 91 only after
  exact agreement with an independent oracle.

## Evidence

The committed red checkpoint is exact: the new target ran 2/3 and failed only
with `CAP-041 intentional product red: tracked runtime ASCII lexer is absent`.
Implementation commit `6290c99` adds the product and Linux/Windows workflow
replay after that failure was preserved.

The focused target is 3/3 green. Its independent Rust scanner constructs the
entire expected source/name/token/checksum model without calling the Aero
product. A separate overlap check compares supported kinds and locations to the
accepted strict Rust lexer without using Rust tokens as expected F1A state.
Coverage includes every accepted kind, repeated names, CR/LF locations,
comments, maximal munch, non-ASCII and unsupported bytes, unterminated block
comments, 63/64-byte identifiers, 1,024/1,025 token and name boundaries, 8,192/
8,193 input bytes, and deterministic generated streams.

The tracked Aero source and generated oracle fixtures check through source and
file APIs, produce byte-identical checked LLVM on repetition, verify under LLVM
22, and run silently with exit 91 at O0 and O2. Wrong expected checksums and a
keyword-classifier mutation do not return 91. A deterministic test runtime
checks allocator/reallocator/deallocator counts, injected failure boundaries,
exact-size destruction, and zero leaks. The public CPU runner exits 91 with no
application output, while ROCm and CUDA reject before requested artifacts.

K1, R1C, R2, and D1 neighboring product targets remain green. The complete
D:-redirected root gate also passes formatting, correctness Clippy, 309 library
tests, 35 binary tests, every integration/native/system target, and doc tests.
The record-inclusive and unchanged exact-content reruns pass the same complete
surface. Protected candidate/merge replay remains required before CAP-041 can
be called accepted.

## Remaining gaps

CAP-041 does not parse. It also does not implement the complete experimental
Rust token vocabulary, Unicode/UTF-8, strings, characters, floats, macros,
modules/imports, path/file input, general collections, recursive heap objects,
semantic analysis, checked-IR construction, verification, LLVM emission, or a
compiler driver. The F1A grammar is the selected bootstrap subset, not a claim
that unsupported stage-0 syntax disappeared from Aero.

The token record intentionally stores spans into the still-live input owner.
It is not an owned String and may not outlive or escape that owner. The
ByteBuffer remains a bounded source resource rather than a public collection or
stable ABI.

## Exact next dependency

After protected CAP-041 acceptance, authorize F1B ledger-first and red-first.
The first parser slice should consume the accepted F1A records without
re-lexing and emit D1 `(kind, payload, left_id, right_id)` nodes for one frozen
single-function grammar. It must freeze precedence, associativity, expected-
token diagnostics, source-order error selection, node topology, and a complete
independent oracle before product mutation. It must stop rather than add parser
or AST semantics to the Rust compiler, transport a ByteBuffer owner, or infer an
unsupported type.
