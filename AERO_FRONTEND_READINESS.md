# Aero Front-End Readiness

Last reviewed: 2026-08-16 (America/New_York)

This record tracks the bounded route from accepted runtime bytes and compiler
storage to an Aero-authored front end. It does not redefine the language. The
Rust lexer and parser remain the accepted stage-0 authorities until the full F1
gate is independently closed.

## Current decision

CAP-041/F1A is accepted as protected PR #83 merge
`4bdfcb206f541356aa83987084a9d2feffbe511c`, tree
`5bfe506bfc6714e32f6453ad5ddc233923298b54`. It changes no compiler production
or runtime file. Its Aero-authored program consumes accepted R2 binary stdin,
retains the complete source in an accepted ByteBuffer, interns canonical name
spans, and emits located serialized token records through two further accepted
ByteBuffer owners.

CAP-042/F1B is the current product-only candidate on that exact accepted head.
It preserves the F1A scanner in the same Aero function, consumes the retained
token records without re-lexing, and uses iterative value/operator stacks to
emit append-only D1 `(kind, payload, left_id, right_id)` records. It changes no
compiler production or runtime file. F1 is not accepted until this composed
product passes its complete local and protected replay gates.

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

## Frozen F1B product

- The admitted grammar is exactly one zero-parameter function returning
  `int`, with one `return expression;` statement and EOF. Identifiers and
  decimal `0..=2147483647` literals are primaries; grouping, prefix `!`/`-`,
  and the frozen arithmetic, comparison, equality, `&&`, and `||` operators
  are admitted.
- Binary operators are left-associative with the frozen precedence order;
  prefix operators are right-associative. Parsing is iterative. The product
  transports no ByteBuffer and adds no recursive compiler representation.
- Node kinds 1 through 19 represent literals, identifiers, prefix/binary
  expressions, return, and function. IDs are one-based append positions;
  every child ID is lower than its parent; the root is the final function
  node.
- F1A limits remain 8,192 bytes, 1,024 real tokens/names, and 63-byte names.
  F1B adds explicit 512-node and 512-entry value/operator-stack bounds.
- Parser statuses 10 through 16 distinguish fixed-token mismatch, expression
  state, wrong return type, i32 overflow, node/stack exhaustion, and internal
  corruption without changing F1A statuses 1 through 9.
- One checksum covers source, names, located tokens, parser status/diagnostic,
  nodes, counts, and root. The canonical source
  `fn score()->int{return 1+2*3-4/2%2;}` yields 2 names, 22 real tokens,
  13 nodes, root 13, checksum 846139, and silent exit 91.

## Evidence

CAP-041 preserved its exact red-first history and is now protected and
accepted. CAP-042 likewise has separate ledger commit `376dfa0` and red commit
`b513fba`; its first test failure was only
`CAP-042 intentional product red: tracked runtime ASCII parser is absent`.

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

The CAP-042 proof adds an independent parser/node/diagnostic/checksum oracle,
an accepted Rust lexer/parser overlap control that does not supply expected
nodes, every operator and precedence boundary, associativity, grouping and
unary chains, malformed input, exact capacity boundaries, deterministic
allocator-failure cleanup, LLVM 22 verification, O0/O2 native replay, public
CPU execution, accelerator artifact hygiene, and Linux/Windows workflow
contracts. Its focused target is 4/4 green; accepted F1A and D1 remain separate
3/3-green neighboring controls. The tracked product also passes its direct
`exact-i32-byte-input-v0` check. The D:-redirected root gate passes formatting,
correctness Clippy, 309 library tests, 35 binary tests, every integration/
native/system target, and doc tests. Protected publication remains pending.

## Remaining gaps

CAP-042 parses only the frozen bootstrap grammar. It does not implement the
complete experimental Rust token/grammar surface, declarations beyond one
function, multiple statements, calls, arrays, records, `Match`, Unicode/UTF-8,
strings, characters, floats, macros, modules/imports, path/file input, error
recovery, semantic analysis, checked-IR construction, verification, LLVM
emission, or a compiler driver. The F1A/F1B grammar is a selected bootstrap
subset, not a claim that unsupported stage-0 syntax disappeared from Aero.

The token record intentionally stores spans into the still-live input owner.
It is not an owned String and may not outlive or escape that owner. The
ByteBuffer remains a bounded source resource rather than a public collection or
stable ABI.

## Exact next dependency

Finish CAP-042/F1B validation from exact accepted CAP-041 head, then protect it
through candidate, merge, and accepted-head replay without changing compiler
production, runtime, or accepted F1A/D1 behavior. Only after that acceptance
may the bounded composed F1 gate advance toward a separately ledgered M1
semantic/checked-IR slice. Do not infer semantics from parser nodes or call the
project self-hosted before M1, B1, and H1/H2 independently close.
