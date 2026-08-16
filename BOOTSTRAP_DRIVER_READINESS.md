# Bounded Bootstrap Driver Readiness

Status: CAP-047/B1C is implemented locally from accepted CAP-046/B1B merge
`3219d7f08a92f9d18334a37315e10cfde6fba931`, tree
`055dfe065ada29b62f22864d879a9c3e18e17c93`, on 2026-08-16. The focused
8/8 product boundary and the complete accepted B1B/B1A/M1B/M1A/F1B/F1A/D1
predecessor ring are green. The complete D:-redirected root gate is also green.
Candidate publication and protected accepted-head replay are still pending, so
this is not yet an accepted public capability.

## Decision

B1C closes one bounded handoff from an Aero-authored, independently verified
serialized module to the declared external LLVM 22.1.8 toolchain. It does not
add file paths or process execution to the Aero language. Aero emits an exact
binary stdout stream one byte at a time; a narrow Rust host command validates
that complete stream before publishing any artifact and then invokes explicitly
named LLVM tools without a shell or PATH fallback.

This is the smallest trustworthy driver boundary for the existing B1B product.
The Aero program remains responsible for frontend, semantic, checked-IR,
verification, and LLVM-emission decisions inside the bounded product. The host
driver owns only stream capture, immutable byte authentication, filesystem
transaction boundaries, and invocation of the declared assembler/compiler/
linker trust base.

## Source and runtime boundary

The new fail-closed selector is `exact-i32-byte-io-v0`. It composes the accepted
exact integer, record/Result, ByteBuffer, and byte-input surfaces and reserves
exactly:

```aero
stdout_write_byte(value: int) -> Result<int, int>
```

Only a direct nongeneric source-function call with exactly one `int` argument
and the accepted explicit typed `Result<int, int>` binding context is admitted.
Inferred, discarded, nested, first-class, overloaded, user-defined, impl,
trait, generic, wrong-arity, and wrong-type uses fail before checked IR. Earlier
profiles retain their prior behavior and do not acquire binary output.

The runtime exports exactly:

```c
int32_t aero_stdout_write_byte(int32_t value);
```

Values `0..=255` write and flush one raw byte and return zero. Sticky `-1`
means write or flush failure, sticky `-2` means Windows binary-mode setup
failure, and sticky `-3` means an out-of-range argument. A failed call writes no
byte; every later call returns the same failure without output. POSIX converts a
broken pipe into the `-1` result instead of terminating the process. Source maps
zero to `Ok(0)` and a negative raw status to `Err(0 - raw)`.

## Checked and backend boundary

`Inst::CheckedStdoutWriteByte { result, value }` consumes one verified logical
`Int` and defines one verified logical `Int` raw status. Independent checked-IR
generation revalidates the source context. The verifier owns definition order,
SSA identity, operand type, result type, and reserved runtime-symbol exclusion.
Only `exact-i32-byte-io-v0` may lower the verified instruction, exactly as:

```llvm
call i32 @aero_stdout_write_byte(i32 VALUE)
```

The runtime declaration is emitted only when used. A raw call, `Print`, legacy
I/O instruction, forged checked result, wrong-profile instruction, or backend-
only symbol cannot substitute for the checked operation.

## Aero-authored B1C product

The tracked product
[`runtime_ascii_toolchain_driver.aero`](examples/aero_frontend_v0/runtime_ascii_toolchain_driver.aero)
preserves the accepted B1B predecessor region byte-for-byte. It adds no
ByteBuffer owner. Only after actual B1B success and an independent reread of the
complete immutable 144-byte module may it call `stdout_write_byte` once for each
byte in order.

Canonical success is:

- B1B module length 144;
- module MD5 `fd2390d17d448d4539a72bf1991314dc`;
- B1B seal 611963;
- output attempted 1, status/runtime code 0, failure index `-1`;
- 144 successfully flushed bytes and B1C seal 506643;
- exactly the accepted 144 bytes on stdout, empty stderr, and exit 91; and
- unchanged 14 allocations, 58 reallocations, 14 deallocations, zero leaks,
  and zero size mismatches.

An output failure exposes at most the exact already-flushed prefix, records the
first failing byte index and positive runtime error code, stops immediately,
releases all owners, and makes the tracked expected-state check exit 95. Stdout
is irreversible; direct shell redirection is therefore outside the driver's
transactional artifact guarantee.

## External driver contract

The host command is exactly:

```text
aero bootstrap-drive-b1c <emitter-executable> --llvm-bin <directory> \
  --output-dir <new-absolute-directory> --opt-level <0|2>
```

It requires an explicit emitter executable, an explicit LLVM directory, an
absolute output directory that does not exist, and optimization level 0 or 2.
It never searches PATH and never invokes a shell. It writes exactly the frozen
34-byte canonical source `fn score()->int{return 1+2*3-4/2;}` to the tracked
Aero emitter's stdin and closes it; it neither parses nor parameterizes those
bytes. It captures stdout/stderr and requires exit 91, empty stderr, exactly
144 bytes, and the frozen MD5 before it creates or writes the output transaction.

The driver requires exact LLVM/Clang 22.1.8 identities, writes `module.ll`,
verifies and assembles it with `llvm-as`, lowers it to an O0/O2 object with
Clang, compiles a fixed C observer, links, and runs the probe. Success requires
`aero_b1_entry() == 5` and probe exit 91. On any later failure it removes only
the directory it created. It never overwrites a path or publishes a partial
module/object/executable.

## Evidence at the local implementation checkpoint

- The source/IR/verifier unit boundary is 3/3 green.
- The focused B1C target is 8/8 green in 315.37 seconds; its exact root-gate
  replay is also 8/8 green in 442.60 seconds.
- Runtime tests cover raw NUL, CR, LF, `0x1a`, `0xff`, invalid range, sticky
  errors, Windows binary mode, and broken-output behavior.
- The tracked product verifies under LLVM 22 and emits the exact 144-byte module
  at O0 and O2.
- The external driver runs the actual tracked Aero B1C executable at O0/O2,
  succeeds at both levels, and rejects an existing output path, malformed
  stream, wrong emitter behavior, wrong tool identity, and PATH-only tools
  without leaving artifacts.
- Every output failure position from 0 through 143 exposes only the exact
  prefix, stops, exits non-success, and preserves exact 14/58/14 cleanup.
- The accepted R1/R2 digest controls and the complete D1/F1/M1/B1 predecessor
  ring are green with D:-resident build and temporary roots.
- The complete repository-root gate exits zero with 312 library tests, 36
  binary tests, every integration/native/system target, and doc tests green.
  Formatting, correctness-denying Clippy, and `git diff --check` are green.
- Protected Linux and Windows workflow replay is present but has not yet run on
  a published candidate.

## What B1C closes—and what it does not

B1C closes the bounded B1 backend path once protected acceptance completes:
authenticated Aero-emitted LLVM can cross a raw-byte boundary into an explicit
LLVM/link toolchain and produce a verified native artifact without asking the
Rust frontend, semantic analyzer, IR generator, verifier, or backend to make a
decision for that module.

It does not produce a compiler executable capable of compiling its own source.
The current Aero-authored product still accepts one frozen expression grammar,
one function/block module shape, fixed opcodes, no paths/files/modules, and no
general diagnostics or source graph. The Rust stage-0 compiler is still needed
to compile the B1C emitter itself. Stage-1/stage-2 convergence, a frozen compiler
source bundle, broader frontend/semantic/IR coverage, and reproducible compiler
artifact comparison remain H1 work. General stdout/text/file/process APIs,
general LLVM lowering, memory-safety, stability, performance, accelerator, and
release claims remain excluded.

## Next dependency

Freeze the exact CAP-047 candidate and complete protected
candidate/merge/accepted-head replay. After acceptance, freeze H1 before adding
behavior: define the canonical Aero compiler source bundle, exact stage-0 to
stage-1 interface, stage-1 to stage-2 invocation, comparison manifest,
environment/toolchain identity, diagnostics, and stop rules. Any expansion of
the bounded grammar, module topology, semantic universe, serialized checked IR,
or LLVM emitter must remain a separate red-first checkpoint rather than being
hidden inside the convergence driver.
