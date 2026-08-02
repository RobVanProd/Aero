# Aero Language Vision

## Purpose and intended users

Aero is intended to be a statically typed, ahead-of-time compiled language for
systems and accelerated-computing programs. It should let developers build
predictable native CPU programs and explicitly move suitable work to supported
accelerators without abandoning one coherent type, ownership, and error model.

The intended users are systems programmers, numerical-computing engineers, and
application developers who need native performance but want safer defaults and
more productive syntax than low-level C-family toolchains commonly provide.

## Design principles

- Semantics come before syntax count. A feature is useful only when the complete
  source-to-execution path is defined and tested.
- Safety properties must be enforced by the compiler or runtime, never inferred
  from an API name or documentation claim.
- Native performance must be predictable and measured on correct programs.
- CPU execution is the portable semantic reference. Accelerator execution has
  explicit capabilities, transfers, synchronization, failures, and numerical
  tolerances.
- Diagnostics, formatting, tooling, interoperability, and reproducibility are
  parts of the language experience rather than optional polish.
- Experimental work is welcome when it is isolated and classified honestly.
- Existing syntax and direction should be preserved unless evidence shows that
  they prevent correctness, safety, implementation coherence, or practical use.

## Distinctive advantages sought

- One language model spanning native CPU code and heterogeneous computation.
- Statically checked abstractions without hidden loss of type information.
- Explicit ownership and memory-space boundaries suitable for systems and device
  code.
- A small, trustworthy compiler core shared by the CLI, library API, and tooling.
- Reproducible evidence for behavior and performance claims.

These are design goals, not claims about the current implementation. Current
support is tracked in `SPEC_IMPLEMENTATION_MATRIX.md` and `BACKEND_STATUS.md`.

## Non-goals

- Reproducing another language feature-for-feature.
- Treating accepted syntax as completed functionality.
- Hiding unsupported constructs behind implicit integer or scalar fallbacks.
- Claiming accelerator support from flags, annotations, textual IR rewrites, or
  execution performed by an unrelated runtime.
- Making performance, formal-proof, or memory-safety claims without reproducible
  evidence appropriate to the claim.
- Preserving historical version labels at the expense of honest stability.

## Compatibility philosophy

Aero will follow semantic versioning once the stable surface is defined. Before
1.0, every release must clearly distinguish stable, experimental, and absent
features. A breaking syntax or semantic change requires a written decision,
compatibility impact, migration path, and tests. Existing behavior that violates
a documented safety or type invariant is a defect, not an implied compatibility
guarantee; its correction still requires release notes and migration guidance
when real programs may depend on it.

## CPU/GPU programming model direction

CPU execution is the required baseline and reference behavior. Accelerator work
must eventually define host and device code boundaries, memory spaces, transfer
ownership, synchronization, streams or queues, kernel invocation, layout and
shape rules, error handling, capability discovery, fallback behavior, and
numerical-equivalence policy. CPU, ROCm, and CUDA are versioned and tested as
separate backends. Automatic fallback must be observable and must not be
reported as device execution.

## Safety and performance philosophy

The compiler must reject invalid programs before code generation and must never
invent types or ownership semantics to continue. Unsafe operations, if added,
will be explicit, narrowly scoped, and documented with caller obligations.
Optimizations may be enabled only under stated legality conditions and must
preserve observable behavior. Performance work begins after correctness, uses
representative programs, reports end-to-end and component costs separately, and
follows `BENCHMARK_PROTOCOL.md`.
