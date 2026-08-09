# Compiler Invariants

These are non-negotiable acceptance conditions for Aero's trusted compiler
path. A violation is a correctness defect even if an existing test expects it.

## Source and diagnostics

- Every accepted source construct has one documented meaning.
- Every source-level value has one well-defined type.
- Unknown, unresolved, unsupported, or malformed types never silently become a
  different type.
- Parser recovery may accumulate diagnostics, but recovered placeholders cannot
  acquire invented semantics or reach code generation.
- Diagnostics refer to valid source ranges wherever source locations exist.
- Essential diagnostic category, phase, and message remain stable under tests.
- A failed compilation returns a failing process/API result and creates no
  executable output.

## Resolution, typing, and ownership

- Typed IR contains no unresolved names, type variables, named-type placeholders,
  or unchecked generic substitutions.
- Every call is resolved to a declared callable with checked arity and argument
  and return types.
- Coercions are explicit in the specification and typed representation.
- Move, copy, borrow, mutability, lifetime, and destruction behavior is defined
  before it is advertised as safe.
- Invalid programs cannot advance to IR or backend generation.
- Compiler phases do not recover by fabricating names, values, types, blocks, or
  ownership states.

## IR and backend

- Typed IR is deterministic for identical inputs and options.
- Every basic block has valid control flow, terminators, dominance, and value
  definitions before backend lowering.
- Backend lowering preserves defined source semantics.
- Optimization cannot change observable behavior.
- Generated LLVM must pass LLVM verification before object generation.
- Object generation and linking failures are surfaced as failures.
- CPU fallback and device paths follow explicit equivalence and observability
  rules.
- Selecting a backend is not evidence that code executed on that backend.

## API, tooling, and builds

- The compiler library is the canonical implementation; CLI and tooling must not
  maintain divergent compiler pipelines.
- Every public compiler option either changes a defined, tested behavior or is
  rejected/deprecated as unsupported.
- Formatter and LSP behavior consume compiler truth instead of approximate,
  separately evolving syntax or semantic models.
- Builds are reproducible from declared inputs, dependency locks, toolchain and
  target information.
- Stable claims are backed by positive, negative, diagnostic, and execution
  evidence appropriate to the claim.
