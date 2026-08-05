# Aero Development Roadmap

Last updated: 2026-08-05 (America/New_York)

This roadmap translates Aero's founding Design -> Minimal Prototype -> Self-Host
-> Stabilize -> Optimize path into evidence-gated engineering milestones. A
milestone title or implemented interface does not certify a feature. Current
capability is defined only by `SPEC_IMPLEMENTATION_MATRIX.md`,
`BACKEND_STATUS.md`, tests, and retained artifacts.

The project is currently in **Minimal Prototype / correctness recovery**.
Historical completed-phase and `v1.0.0` labels do not mean that Aero is stable,
self-hosted, or release-ready.

Current integration work is at `CORE-061`: direct mutable whole-owner reassignment over
the already admitted Copy-data universe, with one shared schema predicate, unified
checked scalar/aggregate place identity, and a composed native exit-83 gate. Its
authorized closure amendment preserves closure parsing while rejecting executable
closure use before checked IR and removes all callable/unknown-type-to-`i32` lowering
fallbacks; it does not add closure semantics. This
continues the harder ownership, aggregate representation, and mutation work in
Milestone 2 rather than optimizing only for convenient compile-time features. CORE-060
whole-place mutable references are accepted public. Projected borrows/writes, deeper CFG
ownership, stable ABI, full module semantics, and real GPU execution remain unresolved.
PR #4 is still a draft integration program; a controlled checkpoint/merge strategy and
structured evidence-manifest automation require separate authorization.

## Milestone 0 - Establish compiler truth (in progress)

- Make invalid lexical and syntactic input fatal on every trusted path.
- Close false-success paths that invent values or silently drop unsupported
  expressions.
- Enforce bounded function, return, binding, and scope contracts before IR.
- Make compiler failures nonzero and prevent invalid-program artifacts.
- Inventory every language, tooling, backend, test, and benchmark claim by stage.

Exit gate: no unclassified critical false-success defect in the chosen stable
subset; one canonical diagnostic/artifact contract; full repository gate and
independent review for each accepted boundary.

## Milestone 1 - Trustworthy scalar CPU core

- Freeze an authoritative grammar and type subset.
- Introduce fallible typed IR with CFG and ownership/type invariants.
- Verify LLVM before object generation.
- Prove source -> semantic analysis -> IR -> object -> link -> execution on Linux
  and Windows for representative scalar programs.
- Add differential optimized/unoptimized runtime tests and real compile-fail
  corpora.

Exit gate: the selected scalar subset is `END_TO_END`, with exact output,
diagnostic, verifier, platform, and reproducibility evidence.

## Milestone 2 - Safe compositional language core

- Implement structs, enums, Match, tuples, and collections as typed aggregate
  vertical slices with defined layout, evaluation order, ABI, and destruction.
- Complete generic substitution, trait dispatch/bounds, and error types.
- Replace shallow ownership tracking with CFG-aware moves, borrows, lifetimes,
  and explicit unsafe boundaries.
- Preserve the 22 active strict Phase 5 syntax-retention tests and replace or recover the 16 quarantined semantic/lossy-shape tests only after their missing contracts are frozen.

Exit gate: at least one real ownership-intensive program and one generic data
structure pass conformance, LLVM verification, and runtime checks without ignored
tests standing in for required behavior.

## Milestone 3 - Aero-native AI/ML infrastructure flagship

- Build a correct CPU reference workload for binary/tensor ingestion and a small
  quantized numerical kernel.
- Grow it into a streaming data or inference component that exercises Aero's
  types, ownership, aggregates, errors, collections, and parallel execution.
- Compare against equivalent established implementations using one correctness
  oracle and the measurement boundaries in `BENCHMARK_PROTOCOL.md`.
- Retain raw inputs, outputs, samples, hashes, toolchains, failures, and artifacts.

Exit gate: a third party can reproduce a useful Aero-native result and understand
its correctness, resource usage, performance, and limitations. External llama.cpp
or framework execution alone does not satisfy this milestone.

## Milestone 4 - Coherent concurrency and tooling

- Freeze an ownership-safe task/channel, structured-concurrency, or equivalent
  model and prove its runtime behavior.
- Converge CLI, library, module resolver, LSP, formatter, documentation generator,
  project tooling, package manager, registry, profiler, and conformance runner on
  one compiler service.
- Add dependency locking, sandbox/trust policy, source-span accuracy, and
  integration tests for failures as well as successful workflows.

Exit gate: tools agree on source meaning and status, concurrent programs have
defined safety/runtime behavior, and a fresh project builds reproducibly.

## Milestone 5 - Proven heterogeneous execution

- Define host/device boundaries, memory spaces, transfers, synchronization,
  streams/queues, kernel ABI, capability discovery, and fallback observability.
- Prove CPU reference equivalence separately for ROCm and CUDA.
- Capture Aero-generated target IR, object/link or loader artifacts, real device
  launch, transfers, synchronization, numerical results, and failure paths.
- Measure compile, transfer, kernel, and end-to-end costs separately.

Exit gate: each named backend independently meets every gate in
`BACKEND_STATUS.md`; a backend flag or transformed helper is insufficient.

## Milestone 6 - Self-host

- Specify the compiler bootstrap boundary and reproducible stage process.
- Implement enough of the compiler in Aero to compile itself in controlled stages.
- Compare stage outputs, diagnostics, runtime behavior, and build reproducibility.
- Keep the Rust bootstrap compiler available until the Aero implementation is
  independently trustworthy.

Exit gate: a documented clean bootstrap produces equivalent verified artifacts
on supported platforms.

## Milestone 7 - Stabilize

- Define the supported surface, compatibility policy, versioning, deprecation,
  migration, release, and governance processes.
- Make all stable examples executable tests and eliminate known invariant
  violations from the release surface.
- Run public RFC review for major semantic commitments.
- Publish an honest pre-1.0 or 1.0 release only when release gates pass.

Exit gate: every stable claim is traceable to specification and end-to-end
evidence, with no historical label substituting for proof.

## Milestone 8 - Optimize and grow the ecosystem

- Add legal, measured optimizations after semantic equivalence is established.
- Expand libraries, platforms, tooling, documentation, and the flagship workload.
- Use larger suites such as SPEC CPU, PARSEC, and domain benchmarks when Aero can
  compile their required workloads correctly.
- Revisit custom backends only where LLVM cannot meet a demonstrated need.

Exit gate: optimization and ecosystem claims remain reproducible, correctness-
gated, and tied to real user workloads.

See `FRAMEWORK_ALIGNMENT.md` for source traceability and the execution-quality
and killer-application rationale behind this sequence.
