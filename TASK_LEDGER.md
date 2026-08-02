# Aero Task Ledger

## AUDIT-001 — Specification and claims consistency

- Problem: Public documents may contradict the grammar, compiler, examples, and
  version/stability state.
- Evidence: README identifies `v1.0.0`; `src/compiler/Cargo.toml` is `0.3.0`.
- Priority: P0
- Dependencies: none
- Allowed subsystem: read-only README, docs, tutorials, RFCs, examples, CLI help
- Acceptance: cited contradiction/unsupported/untested claim report
- Owner: specification audit agent
- Status: complete
- Verification: repository searches and direct source/doc comparison
- Result commit: none (read-only)
- Decision: public/spec surface is contradictory and experimental; freeze an
  authoritative subset before semantic changes.

## AUDIT-002 — Frontend integrity

- Problem: Advertised grammar, spans, recovery, AST fidelity, and panic resistance
  have not been mapped.
- Evidence: lexer/parser/AST exist with uneven test families.
- Priority: P0
- Dependencies: none
- Allowed subsystem: read-only lexer, parser, AST, errors, formatter, grammar/tests
- Acceptance: cited pipeline and gap report with reproducer ideas
- Owner: frontend audit agent
- Status: complete
- Verification: source trace and test inventory
- Result commit: none (read-only)
- Decision: require fallible lexing/parsing and end-to-end spans; do not count
  lossy AST acceptance as support.

## AUDIT-003 — Type and ownership soundness

- Problem: Unsupported type forms may be accepted through scalar fallbacks and
  invalid programs may reach IR.
- Evidence: active calls return `Int`; annotations/returns are ignored; dormant
  validation maps unknown and composite parameter types to `Int`.
- Priority: P0
- Dependencies: none
- Allowed subsystem: read-only semantic/type/generic/pattern/ownership/IR boundary
- Acceptance: cited soundness findings, failure phases, reproducer sketches
- Owner: type-soundness audit agent
- Status: complete
- Verification: source trace and existing tests
- Result commit: none (read-only)
- Decision: active scalar fallbacks violate compiler invariants; function/type
  contracts require a later two-pass semantic slice.

## AUDIT-004 — IR and code generation

- Problem: Parsed/typed constructs may be dropped or approximated in IR/LLVM.
- Evidence: parse errors can emit invalid LLVM; boolean storage and CFG lowering
  violate LLVM invariants; multiple supported-looking constructs become zero.
- Priority: P0
- Dependencies: AUDIT-002, AUDIT-003 helpful but not blocking
- Allowed subsystem: read-only AST-to-IR-to-LLVM path and tests
- Acceptance: construct-by-construct lowering gaps and verifier coverage
- Owner: IR/code-generation audit agent
- Status: complete
- Verification: source trace, focused read-only commands
- Result commit: none
- Decision: make parser failure fatal first; typed IR, CFG validation, and LLVM
  verification remain P0 follow-ups.

## AUDIT-005 — Runtime and backend truth

- Problem: CPU, ROCm, CUDA, graph, and quantization maturity must be separated.
- Evidence: CLI explicitly says CUDA run target is unimplemented; ROCm message
  says runtime execution is staged for HIP launcher integration.
- Priority: P0
- Dependencies: none
- Allowed subsystem: read-only backend/runtime/graph/quantization code, tests/docs
- Acceptance: interface/object/link/execution/correctness/performance matrix
- Owner: runtime/backend audit agent
- Status: complete
- Verification: source trace and environment capability probes
- Result commit: none
- Decision: CPU is partial end-to-end; ROCm is object plumbing; CUDA is
  parsed/selectable only; graph/quantization helpers are experimental transforms.

## AUDIT-006 — Tooling and compiler API

- Problem: CLI, LSP, formatter, docs, profiler, init, modules, registry, and public
  compiler API may be divergent or incomplete.
- Evidence: binary and library both declare compiler modules;
  `compile_program` ignores `CompilerOptions`.
- Priority: P0
- Dependencies: none
- Allowed subsystem: read-only tooling/API source, CLI docs/tests
- Acceptance: command and API behavior classification with duplication map
- Owner: tooling audit agent
- Status: complete
- Verification: source trace and non-mutating CLI probes
- Result commit: none
- Decision: establish shared `Result`/exit behavior and canonical library boundary;
  keep live registry work disabled pending containment/payload review.

## AUDIT-007 — Test and fuzz coverage

- Problem: Test count alone does not establish phase, negative, runtime, fuzz, or
  cross-platform coverage.
- Evidence: baseline passes; conformance command currently reports 3 cases and 4
  deterministic checks.
- Priority: P0
- Dependencies: none
- Allowed subsystem: read-only tests, CI, harnesses, examples, fuzz configuration
- Acceptance: layered inventory and ranked untested risks
- Owner: testing/fuzzing audit agent
- Status: complete
- Verification: test listing, CI and harness inspection
- Result commit: none
- Decision: add compile-fail/no-artifact tests first; classify dormant/ignored
  tests individually before reactivation.

## AUDIT-008 — Benchmarks and public claims

- Problem: Every claim needs immutable reproducible evidence and correct scope.
- Evidence: README and `claim-verification/claims.json` contain historical and
  current benchmark statements.
- Priority: P0
- Dependencies: none
- Allowed subsystem: read-only README, benchmarks, scripts, claim-verification
- Acceptance: per-claim evidence/protocol classification and discrepancies
- Owner: benchmark/claims audit agent
- Status: complete
- Verification: artifact/hash/config/source review; no public claim changes
- Result commit: none
- Decision: quarantine invalid compilation claims; preserve lexer as partial
  historical evidence and GGUF as an external one-run observation.

## CORE-001 — Reject malformed syntax before IR/codegen

- Problem: Syntax errors are printed and converted to an empty AST; compilation
  then reports success and writes invalid LLVM.
- Evidence: `let = ;` through CLI `build` exits zero, prints both a parse error and
  success, and writes an unterminated `define i32 @main()` artifact.
- Priority: P0
- Primary hypothesis: switching public build/check/library entry points from the
  legacy infallible parser wrapper to the located fallible parser, and propagating
  `Result`, will stop malformed source before semantics without changing valid
  program behavior.
- Dependencies: committed audit baseline `1d9396067dfac294aebd1e6c29765e503c504040`
- Observed behavior: malformed syntax reaches semantic analysis, IR, graph
  transformation, file output, and a success message.
- Expected behavior: parser failure is returned as an API error; CLI build/check
  exit nonzero; build creates no requested LLVM artifact; no later-phase success
  message is printed.
- Smallest reproducer: `let = ;`
- Files allowed: `src/compiler/src/lib.rs`, `src/compiler/src/main.rs`, a new
  focused integration test under `src/compiler/tests/`, and the control documents
  affected by verified results.
- Files frozen: lexer tokenization rules, grammar, AST shape, semantic/type/
  ownership rules, IR and backend lowering, README/public claims.
- Frozen semantics: every parser error is fatal for compilation; valid syntax and
  all later-phase semantics are unchanged.
- Positive tests: a valid minimal program still compiles through the library.
- Negative tests: library rejects malformed syntax; CLI build/check return
  failure; requested build output does not exist.
- Runtime-output tests: not applicable because malformed input must never reach
  runtime.
- Diagnostic expectation: error contains a stable `Parse error` category and the
  parser's essential expected/found message.
- Regression risks: changing helper return types can miss a CLI caller; existing
  tests may rely on the legacy `parse(Vec<Token>) -> Vec<AstNode>` wrapper; stale
  output paths must not be mistaken for newly generated artifacts.
- Acceptance criteria: focused tests pass; `./tools/test.sh` passes; the manual
  reproducer exits nonzero and creates no output; no valid-program test regresses.
- Stop conditions: implementation requires grammar/recovery changes, changes the
  legacy parser API used by broad tests, crosses into type/IR semantics, or grows
  beyond the listed files.
- Owner: one isolated implementation agent; lead integrates.
- Status: integrated; independent review requested closure work
- Verification commands: focused Cargo integration test; manual CLI reproducer;
  `./tools/test.sh`.
- Result commit: `30b9b48658b0e1b1638b273341044dc2c8d64646`
- Final decision: core library/build/check/run compilation paths are corrected,
  but acceptance remains open until `CORE-001B` closes public parser fallbacks,
  strengthens failure evidence, and removes the failed-run directory leak.

## CORE-001B — Close fatal-parse public paths and regression evidence

- Problem: Independent review found that `aero profile` and `aero test` still use
  the legacy infallible parser; CLI regressions can pass because of later failures;
  malformed `run` creates and leaks an empty per-invocation directory.
- Evidence: frontend/API review and negative/regression review of
  `30b9b48658b0e1b1638b273341044dc2c8d64646` both returned changes requested.
- Priority: P0
- Primary hypothesis: using the located fallible parser in the two remaining
  compilation-oriented public commands, making test-suite failures affect status,
  cleaning run artifacts on compile failure, and asserting parser-specific output
  will close the accepted invariant without grammar or later-phase changes.
- Dependencies: `CORE-001` integrated at
  `30b9b48658b0e1b1638b273341044dc2c8d64646`.
- Observed behavior: malformed profile input reaches semantic/IR/codegen; malformed
  `*_test.aero` can be reported as passing; focused CLI tests only require nonzero;
  a malformed run leaves `target/aero-run/<nonce>` behind.
- Expected behavior: build/check/run/profile/test reject malformed input with a
  located `Parse error`; build/profile emit no requested artifact; run invokes no
  native tool and leaves no per-run directory; a malformed discovered test makes
  the test command fail.
- Smallest reproducers: root `let = ;`; imported module `mod broken;` with
  `broken.aero` containing `let = ;`; `malformed_test.aero` containing `let = ;`.
- Files allowed: `src/compiler/src/main.rs`, `src/compiler/src/profiler.rs`,
  `src/compiler/tests/fatal_parse_tests.rs`, and affected control documents.
- Files frozen: lexer rules, grammar, AST, semantic/type/ownership rules, IR,
  optimization and backend lowering, public claims, registry and LSP behavior.
- Frozen semantics: every parser error is fatal for compilation; valid inputs and
  all language semantics are unchanged. Nonzero process status for any surfaced
  compile/test failure is explicitly accepted by `DEC-005`.
- Positive tests: existing valid library/profile/compiler suites remain passing.
- Negative tests: parser-specific located diagnostics for build/check/run;
  malformed imported module; malformed profile with no trace; malformed discovered
  test with nonzero status; no failed-run directory remains.
- Runtime-output tests: verify native tool error strings are absent for malformed
  `run`; valid native execution is unchanged and outside this slice.
- Diagnostic expectation: `Parse error`, expected/found context, source filename,
  and line/column are observable before later-phase diagnostics.
- Regression risks: test discovery scans three directories; platform path display
  differs; cleanup must not delete intentionally retained ROCm outputs or artifacts
  from later native-tool failures.
- Acceptance criteria: focused integration tests pass on Windows; complete
  `./tools/test.sh` passes; manual root and module reproducers fail without outputs;
  second independent review returns approved.
- Stop conditions: grammar/recovery or semantic changes are required; cleanup
  crosses beyond compile failure; valid profile/test behavior changes beyond status.
- Owner: one isolated implementation agent; lead integrates.
- Status: preregistered
- Verification commands: focused Cargo integration test, manual CLI reproducers,
  `./tools/test.sh`.
- Result commit: pending
- Final decision: pending independent review.
