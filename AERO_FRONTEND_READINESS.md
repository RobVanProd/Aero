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

CAP-043/M1A is accepted through protected PR #85. Reviewed candidate
`1cfa7acc09c741d219c57ebe04f1e6c26949838e` merged as
`2eaa3bdd9de886453d8556d457d49dbb937770ae`; their tree is identically
`35129ad5194354acafe082f3fcd55629ed767f27`, and candidate plus accepted-head
workflows are terminal-green.

CAP-044/M1B is accepted through protected PR #86. Reviewed candidate
`a14d30d1c37c3b34626a6ec8c74848e2bc8f8a2c` merged as
`f51ea2d63b886c1615f522ea3d14bf7baefead1a`; their tree is identically
`bca690421a34862063a0bc9315c74873f261f354`. Candidate and accepted-head CI,
stable/nightly Rust, Windows LLVM 22 native, CodeQL, and evidence workflows are
terminal-green.

CAP-045/B1A is accepted through protected PR #87. Reviewed candidate
`5d36aacc0ffadf149eb6b4920ee59cd5d175c113` merged as
`3054db736cbde2c53ade068e7a8d608b510feb63`; their tree is identically
`f534988d9264a236c36f8ed9b02e08dad7cceba7`. All candidate and accepted-head
workflows are terminal-green.

CAP-046/B1B is the current product-only local candidate on that exact accepted
head. It preserves B1A byte-for-byte, then rereads only the authenticated
serialized `checked_ir` into a fourteenth direct `emitted_llvm` owner. It emits
one deterministic 144-byte, host-neutral LLVM module for the bounded module and
does not change Rust compiler/runtime production or invoke a toolchain. B1B is
not accepted until its complete local and protected gates finish.

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

## Frozen M1B product

- M1B runs only after complete F1/M1A success and consumes the retained node,
  origin, symbol, and fact owners. It does not rescan, reparse, or infer a type.
- The admitted numeric subset is the M1A-successful integer expression family
  whose every intermediate fits signed i32 and whose divisors are nonzero.
  Overflow status is 1 and zero-divisor status is 2; earlier frontend or
  semantic failure skips M1B with no serialized module.
- Six-word value records use immediate or one-based SSA-result operands and a
  base-32768 signed magnitude, including the exact i32-min representation.
  Opcodes are Add, Sub, Mul, Div, Neg, and Return.
- The serialized little-endian module contains one header, one function, one
  reachable entry block, contiguous eleven-word instructions, and contiguous
  six-word result definitions. Results are defined before use and Return is the
  final instruction.
- Three new direct ByteBuffer owners hold value scratch, instruction scratch,
  and serialized checked IR. Together with M1A there are exactly twelve direct
  owners, all destroyed exactly once in reverse declaration order.
- The canonical source yields 9 values, 5 instructions, 4 results, 104 words,
  root `Result(4): Int`, checked checksum 355067, and silent native exit 91.

## Frozen B1A product

- B1A begins only after complete M1B success and consumes only the serialized
  `checked_ir` owner plus its own scalar state and `verified_results` owner. It
  cannot use predecessor AST, semantic, construction, or checksum state as an
  acceptance oracle.
- The verifier independently checks little-endian framing, exact one-function/
  one-block topology, contiguous instruction and result records, opcode/type/
  origin rules, backward-only SSA uses, result-definition equality, and the
  root/Return relation.
- Arithmetic is independently evaluated as exact signed i32 with explicit
  overflow, divide-by-zero, and `i32::MIN / -1` rejection. Four verifier result
  records encode signed values in the frozen base-32768 representation.
- Statuses 1 through 8 separate framing, topology, instruction, operand/SSA,
  arithmetic, result, root, and allocation failures. The first failure records
  an exact word, record, code, expected value, and actual value.
- The product has exactly thirteen direct ByteBuffer owners. The new owner is
  declared after `checked_ir`; all owners are destroyed once in reverse order
  on success, corruption, allocation failure, and every earlier-phase exit.
- The canonical 104-word M1B module verifies 5 instructions and 4 results,
  evaluates root 5, seals to checksum 592819, and returns silent native exit 91.
  This is an independent bounded verifier, not an LLVM emitter or general IR
  verifier.

## Frozen B1B product

- B1B begins only after actual B1A success, complete counts, and a disabled
  verifier fault selector. Earlier failures and every enabled selector skip
  emission with an empty output and zero seal.
- The emitter rereads only immutable authenticated `checked_ir` words plus the
  actual B1A seal/count scalars. It does not consult source, names, tokens,
  nodes, semantic facts, construction scratch, verifier-result scratch, or
  expected-value parameters as admission authority.
- The exact mapping covers `add`, `sub`, `mul`, `sdiv`, integer negation as
  `sub i32 0, L`, and terminal `ret`, with deterministic `%rN` identities and
  unsigned decimal rendering. Output is ASCII/LF only and contains no target,
  path, timestamp, debug metadata, or source-name claim.
- The canonical module is exactly 144 bytes, MD5
  `fd2390d17d448d4539a72bf1991314dc`, and B1B seal 611963. LLVM 22 accepts it
  and an independent native caller observes result 5 at O0 and O2.
- `emitted_llvm` is the sole new fourteenth ByteBuffer owner. Canonical success
  uses 14 allocations, 58 reallocations, and 14 reverse-order deallocations;
  every injected failure leaves zero live allocations and no size mismatch.
- This is one bounded in-memory LLVM text emitter. It is not file/process
  output, object generation, linking, a general backend, or self-hosting.

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
system target, and doc tests. Protected publication and accepted-head replay
are complete.

Reviewed CAP-043 candidate `1cfa7acc09c741d219c57ebe04f1e6c26949838e`
merged through PR #85 as `2eaa3bdd9de886453d8556d457d49dbb937770ae`
with identical tree `35129ad5194354acafe082f3fcd55629ed767f27`.
Candidate and accepted-head CI, stable/nightly Rust, Windows LLVM 22 native,
CodeQL, and evidence workflows are terminal-green; CAP-043 is accepted.

CAP-044 preserves separate ledger commit `1a09eb7` and red commit `a498efd`;
its first test failure was only
`CAP-044 intentional product red: tracked runtime ASCII checked IR is absent`.
The focused M1B target is 10/10 green. Its independent M1A-compatible model
constructs and validates the complete value/instruction/result/module state and
checksum without using the Aero product as expected state. Accepted Rust
checked-IR overlap supplies only projected result/signature/block behavior.
Coverage includes every frozen opcode, exact i32 edges, overflow, direct and
derived zero divisors, every earlier phase failure with zero IR, source and
record mutations, malformed SSA/module cases, deterministic failure at every
allocation/reallocation boundary, exact twelve-owner cleanup, source/file LLVM
equality, LLVM 22, O0/O2, public CPU, accelerator artifact hygiene, and Linux/
Windows replay. Accepted M1A is 7/7, F1B 4/4, F1A 3/3, and D1 3/3 green.
The complete D:-redirected root gate passes formatting, correctness Clippy,
309 library tests, 35 binary tests, every integration/native/system target, and
doc tests. Reviewed candidate `a14d30d1c37c3b34626a6ec8c74848e2bc8f8a2c`
merged through PR #86 as `f51ea2d63b886c1615f522ea3d14bf7baefead1a`
with identical tree `bca690421a34862063a0bc9315c74873f261f354`.
All 13 candidate checks and accepted-head CI `31938072475`, Rust CI
`31938072465`, CodeQL `31938071907`, and evidence `31938072658` are
terminal-green; CAP-044 is accepted.

CAP-045 preserves separate ledger commit `422acb5`, pin-seal commit `05ab6b6`,
and red-first commit `b5ad993`; its first failing assertion was only
`CAP-045 intentional product red: tracked runtime ASCII checked IR verifier is
absent`. The independent Rust model accepts every frozen opcode and count edge,
recomputes canonical seal 592819, and rejects every framing, topology,
instruction, SSA, arithmetic, result, and root corruption family. The tracked
product is green through source/file compilation, external LLVM 22 verification,
O0/O2 native execution, public CPU execution, accelerator artifact hygiene,
direct verifier fault replay, and the complete 66-allocation failure sweep with
13 initial allocations, 53 reallocations, 13 deallocations, and zero leaks.
Accepted M1B is 10/10, M1A 7/7, F1B 4/4, F1A 3/3, and D1 3/3 green.
The complete D:-redirected root gate passes formatting, correctness Clippy,
309 library tests, 35 binary tests, every integration/native/system target,
and doc tests.
Reviewed candidate `5d36aacc0ffadf149eb6b4920ee59cd5d175c113` merged through
PR #87 as `3054db736cbde2c53ade068e7a8d608b510feb63` with identical tree
`f534988d9264a236c36f8ed9b02e08dad7cceba7`. All 13 candidate checks and
accepted-head CI `31946571509`, Rust CI `31946571387`, CodeQL `31946571049`,
and evidence `31946571478` are terminal-green; CAP-045 is accepted.

CAP-046 preserves ledger-only commit `cbc71a6` and red-first commit `f52ff37`;
the red checkpoint passed three independent oracle tests and failed only with
`CAP-046 intentional product red: tracked runtime ASCII LLVM emitter is
absent`. Local implementation commit `4078b2f` adds the product, expanded
focused proof, and Linux/Windows workflow steps without changing Rust compiler
or runtime production. The focused target is 5/5 green: exact Aero-owned bytes
are captured at cleanup and equal the independent 144-byte oracle at O0/O2;
LLVM 22 and native result 5 pass; all seven B1A corruption families, outside
and same-value enabled selectors, all 72 allocation thresholds, exact cleanup,
public CPU execution, and accelerator artifact hygiene pass.
The accepted B1A/M1B/M1A/F1B/F1A/D1 predecessor ring is green, and the complete
D:-redirected root gate exits 0 with formatting, correctness Clippy, 309 library
tests, 35 binary tests, every integration/native/system target, and doc tests.
Protected publication remains pending, so CAP-046 is not yet accepted.

## Remaining gaps

CAP-044, CAP-045, and CAP-046 process only the frozen F1B/M1A bootstrap
grammar. They do not implement the complete experimental Rust token/grammar surface,
declarations beyond one function, multiple statements, calls, arrays, records,
`Match`, Unicode/UTF-8,
strings, characters, floats, macros, modules/imports, path/file input, error
recovery, general scopes or symbols, general types or ownership, dynamic
arithmetic, general checked-IR verification, file/process output, or a compiler
driver. B1A is an independent Aero verifier for the one exact M1B format; B1B
emits LLVM only for that authenticated bounded format. Neither is a verifier
or backend for the production Rust checked-IR universe. The
F1A/F1B/M1A/M1B/B1A/B1B surface is a selected bootstrap subset,
not a claim that unsupported stage-0 syntax disappeared from Aero or that the
Rust front end has been replaced.

The token record intentionally stores spans into the still-live input owner.
It is not an owned String and may not outlive or escape that owner. The
ByteBuffer remains a bounded source resource rather than a public collection or
stable ABI.

## Exact next dependency

Finish CAP-046/B1B validation from exact accepted CAP-045 merge `3054db7` and
protect its bounded in-memory LLVM emitter through candidate, merge, and
accepted-head replay without changing compiler production, runtime, profile,
or accepted predecessor behavior. After B1B acceptance, freeze B1C separately
and red-first for the file/process/toolchain driver. Do not infer additional
semantics, call the Rust front end replaced, or call the project self-hosted
before B1C and H1/H2 independently close.
