# Whole-Stream Byte Input Readiness Contract

Status: CAP-039/R2 accepted at protected merge
`5c791393be5a251c187274d591174f7667866886`, tree
`06ee7ada90315432ce26d706f348685e2ee5458f`, 2026-08-15.

Reviewed candidate `c020d477f6bfd188b0008249b8287d4d6d5051c5` has the
identical tree. All 13 candidate checks passed. Accepted-head CI
`31918179906`, Rust CI `31918179914` (stable, nightly, and Windows LLVM 22),
CodeQL `31918179970`, and accepted-head evidence `31918179909` are
terminal-success. Protected PR #81 records the exact candidate, ordered merge
parents, merge, and tree identity.

## Decision

R2 reads binary standard input before it introduces file paths, path encoding,
module identity, seekability, or text decoding. The runtime exposes one scalar
read operation; Aero source owns the whole-stream loop and stores successful
bytes through the accepted R1 `ByteBuffer` API. This is the smallest input gate
that materially advances an Aero-authored front end without hiding collection
or stream control in Rust.

The implementation is one bounded vertical slice with two compiler
authorities:

1. source/profile semantics plus independent checked-IR production; and
2. checked-IR schema/verifier plus LLVM backend lowering.

The C runtime ABI and CPU driver transport are outside those semantic compiler
phases. Stop if input requires a third compiler authority or changes accepted
R1 ownership/resource rules.

## Frozen runtime ABI

The production runtime adds exactly:

```c
int32_t aero_stdin_read_byte(void);
```

Its result is closed:

| Raw result | Meaning |
|---:|---|
| `0..=255` | one consumed byte |
| `-1` | clean EOF; repeated reads remain EOF |
| `-2` | input or binary-mode failure; repeated reads remain failed |

The operation blocks according to ordinary host stdin behavior. It consumes no
byte on EOF/error and exposes no buffer, pointer, capacity, host collection, or
uninitialized storage. On Windows the runtime switches stdin to binary mode
before the first read so CR/LF, `0x1a`, NUL, and `0xff` are preserved exactly.
POSIX uses the byte stream unchanged. Runtime setup failure is input failure.

There is deliberately no external length argument, eliminating an invalid
length channel. The accepted R1 buffer remains the only initialized-length and
capacity authority.

## Frozen checked contract

Add exactly `Inst::CheckedStdinReadByte { result }`. It defines one logical
`Int`, takes no operands, has no resource/place identity, and has no hidden
buffer effect. The independent verifier owns SSA uniqueness and type metadata
and rejects forged/duplicate results through the existing definition rules.
The backend emits exactly `call i32 @aero_stdin_read_byte()` only after checked
verification and only for the new selected profile. Earlier profiles reject
the instruction before LLVM. The declaration appears only when used.

No raw generic call, historical Vec instruction, host callback, inline syscall,
or backend-only spelling may bypass this checked instruction.

## Frozen source/profile contract

Add exactly the selector `exact-i32-byte-input-v0`. It composes the complete
accepted `exact-i32-byte-buffer-v0` surface without changing that accepted
profile and reserves one additional zero-argument intrinsic:

```aero
stdin_read_byte() -> Result<int, int>
```

The source result maps raw `0..=255` to `Ok(byte)` and a raw negative sentinel
to `Err(0 - raw)`, therefore EOF is `Err(1)` and I/O/setup failure is `Err(2)`.
The runtime contract produces no other negative value; a deterministic mock
may return `-3` only as a corruption control and must surface `Err(3)`, never a
byte or fallback. The Result must be consumed in the accepted explicit typed
context; discarded, inferred, first-class, overloaded, user-defined, or
argument-bearing forms are rejected before IR.

The intrinsic is valid only inside a direct nongeneric source function under
the new profile. Existing profiles continue to resolve the spelling as an
ordinary unknown function. `check`, compilation, and cache lookup never read
stdin. Only execution of the emitted CPU program consumes input.

## Whole-stream product

The tracked Aero product performs the complete loop:

1. create one explicit mutable `ByteBuffer`;
2. call `stdin_read_byte()` repeatedly;
3. push each `Ok(byte)` through accepted `bytes_push`;
4. stop only on `Err(1)` EOF;
5. surface any other read or push error without treating it as EOF; and
6. inspect the initialized buffer and return the frozen native sentinel.

The public CPU runner must inherit its own stdin into the compiled child while
continuing to capture stdout/stderr and remove its isolated artifacts. An input
program on ROCm/CUDA is rejected before object/link/execution artifacts.

## Required evidence

- Red-first absent selector/intrinsic/checked instruction/runtime symbol.
- Exact preservation of accepted R1 source behavior, owner cleanup, allocator
  ABI, verifier rules, LLVM, native result, and every earlier profile.
- Runtime C harnesses for NUL, CR/LF, `0x1a`, `0xff`, repeated EOF, closed/broken
  stdin failure, and no byte consumption after failure.
- Verifier/backend corruption matrix: duplicate result, wrong profile, missing
  declaration/call lowering, raw/generic bypass, and deterministic LLVM.
- Source/file `check` and compile parity; discarded/inferred/argument-bearing/
  user-collision/ordinary-profile/accelerator negatives.
- Empty, short binary, growth-crossing, and large input with exact length,
  content/checksum, EOF, cleanup, and zero leaks.
- Deterministic partial-prefix then injected I/O failure proving the prefix
  remains initialized and is dropped exactly once.
- LLVM 22 verification, O0/O2 native parity, and public `aero run` stdin
  forwarding on Linux and Windows.
- Formatting, correctness Clippy, `git diff --check`, complete root gate,
  protected candidate/merge identity, and accepted-head replay.

## Explicit exclusions

R2 does not add file/path input, arguments, seek, nonblocking I/O, cancellation,
timeouts, output writes, Unicode/UTF-8 decoding, `String`, general streams,
buffer function transport, stored aliases, general collections, owned compiler
names, flat AST arenas, modules, a production front end, accelerator execution,
memory-safety/stability/performance claims, release readiness, or self-hosting.

## Mandatory stops

Stop if Windows and POSIX cannot preserve identical binary bytes and sentinel
meaning; `aero run` cannot forward stdin without changing earlier program
behavior; source admission needs inference or a second AST walk; checked IR or
backend can call the runtime before verification; EOF/error can expose an
uninitialized byte; the operation must mutate a ByteBuffer behind the R1
verifier; the accepted R1 runtime/resource/profile changes; a third compiler
phase is required; or any failure can silently become EOF, success, or `Int`.

## Next dependency after R2

R2 is accepted. CAP-040/D1 is now the locally green successor: deterministic
owned token/name storage and a flat append-only AST arena with integer IDs.
See [`COMPILER_STORAGE_READINESS.md`](COMPILER_STORAGE_READINESS.md). R2
supplies bytes only; D1 remains separately unaccepted until its full gate and
protected replay close.
