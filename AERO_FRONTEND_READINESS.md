# Aero Front-End Readiness

Last reviewed: 2026-08-16 (America/New_York)

This record tracks the bounded route from accepted runtime bytes and compiler
storage to an Aero-authored front end. It does not redefine the language. The
Rust lexer and parser remain the accepted stage-0 authorities until the full F1
gate is independently closed.

## Current decision

CAP-042/F1B is accepted as protected PR #84 merge
`35020e9d14ae58cd8a2bbd34d81f7930aa537be5`, tree
`baab4ce63fc48a4fc55b6fa56b2cc1416a447c8e`. Reviewed candidate
`e42d6aa290bcb5e052e5c7c51702b484b4af1877` has the identical tree. It preserves
the accepted F1A scanner in one Aero function, consumes the retained token
records without re-lexing, and uses iterative value/operator stacks to emit
append-only D1 `(kind, payload, left_id, right_id)` records. All candidate and
accepted-head workflows are terminal-green. No compiler production or runtime
file changed.

CAP-043/M1A is the current product-only candidate on that exact accepted head.
It copies the accepted F1B product, records one source/token origin for every
flat node at the parser's existing append decision, and then performs a bounded
two-pass name/type/ownership classification. It changes no Rust compiler,
runtime, profile, grammar, checked-IR, verifier, backend, or driver file. M1A is
not M1B: the product emits semantic facts but does not construct checked IR.

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

## Frozen M1A product

- Every completed F1B node has one append-ordered origin record
  `(node_id, offset, line, column, token_kind)`. The origin append is attempted
  before the unchanged four-word node append and is validated against retained
  source and located-token storage.
- A complete source-order name prepass runs before type classification. The
  frozen one-function grammar declares no value names, so the first identifier
  expression is an undeclared-name error and no unsupported value is silently
  assigned a type.
- The closed logical universe is `Void`, `Int`, and `Bool`; admitted expression
  values are `Copy`. One function symbol and append-ordered
  `(node_id, logical_type, ownership)` facts describe the bounded result.
- The iterative type pass preserves accepted Rust semantic phase and operand
  order for arithmetic, comparisons, logical operators, unary operators, and
  the `int` return. Modulo remains syntactically represented by F1B but is an
  explicit M1A unsupported-operation error.
- Nine direct ByteBuffer owners hold source, names, tokens, nodes, value and
  operator stacks, origins, symbols, and facts. They never escape or move by
  value and are destroyed exactly once in reverse declaration order.
- The canonical source `fn score()->int{return 1+2*3-4/2;}` yields 2 names,
  20 real tokens, 11 nodes, root 11, frontend checksum 586661, one symbol,
  11 origins, 11 facts, semantic root type Int, semantic checksum 827574, and
  silent native exit 91.

## Evidence

CAP-041 and CAP-042 preserve their exact red-first histories and are protected
and accepted. CAP-042 has separate ledger commit `376dfa0` and red commit
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

The accepted CAP-042 proof adds an independent parser/node/diagnostic/checksum oracle,
an accepted Rust lexer/parser overlap control that does not supply expected
nodes, every operator and precedence boundary, associativity, grouping and
unary chains, malformed input, exact capacity boundaries, deterministic
allocator-failure cleanup, LLVM 22 verification, O0/O2 native replay, public
CPU execution, accelerator artifact hygiene, and Linux/Windows workflow
contracts. Its focused target is 4/4 green; accepted F1A and D1 remain separate
3/3-green neighboring controls. The tracked product also passes its direct
`exact-i32-byte-input-v0` check. The D:-redirected root gate passes formatting,
correctness Clippy, 309 library tests, 35 binary tests, every integration/
native/system target, and doc tests. Protected PR #84 and accepted-head replay
are terminal-green.

CAP-043 preserves separate ledger commit `b6bba12` and red commit `669d2ba`;
its first test failure was only
`CAP-043 intentional product red: tracked runtime ASCII semantic facts are absent`.
The focused M1A target is 7/7 green. An independent scanner/parser/origin/
semantic oracle covers all closed node, type, ownership, operator, diagnostic,
phase-order, checksum, mutation, capacity, and allocation rules without using
the Aero product as expected state. A separate accepted Rust semantic overlap
control compares only success and first-error families. Accepted F1B is 4/4,
F1A is 3/3, and D1 is 3/3 green. LLVM 22, O0/O2 native, public CPU,
accelerator-artifact hygiene, nine-owner cleanup, and Linux/Windows replay are
green locally. The complete D:-redirected root gate passes formatting,
correctness Clippy, 309 library tests, 35 binary tests, every integration/native/
system target, and doc tests. Protected publication remains pending.

## Remaining gaps

CAP-043 analyzes only the frozen F1B bootstrap grammar. It does not implement the
complete experimental Rust token/grammar surface, declarations beyond one
function, multiple statements, calls, arrays, records, `Match`, Unicode/UTF-8,
strings, characters, floats, macros, modules/imports, path/file input, error
recovery, general scopes or symbols, general types or ownership, constant
evaluation, checked-IR construction, verification, LLVM emission, or a compiler
driver. The F1A/F1B/M1A surface is a selected bootstrap subset, not a claim that
unsupported stage-0 syntax disappeared from Aero or that the Rust front end has
been replaced.

The token record intentionally stores spans into the still-live input owner.
It is not an owned String and may not outlive or escape that owner. The
ByteBuffer remains a bounded source resource rather than a public collection or
stable ABI.

## Exact next dependency

Finish CAP-043/M1A validation from exact accepted CAP-042 head, then protect it
through candidate, merge, and accepted-head replay without changing compiler
production, runtime, profile, or accepted F1B/F1A/D1 behavior. After M1A
acceptance, freeze M1B separately and red-first: consume authenticated origins,
symbol, and semantic facts to construct the first bounded checked-IR product.
Do not infer additional semantics, call the Rust front end replaced, or call the
project self-hosted before M1B, B1, and H1/H2 independently close.
