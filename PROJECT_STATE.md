# Aero Project State

Last updated: 2026-08-02 (America/New_York)

## Current objective

Milestone 16 tests-only red review — independently verify the exact `CORE-015`
binding-type contract before publishing its failing tests. The preregistration is
public and green at `4f31f0c`; no production source has changed. Accepted `CORE-014`
remains closed at `1535ce2`; benchmark execution remains quarantined.

## Active hypothesis

`CORE-015` can close four of the five highest-severity reproduced type-contract false
successes without inventing conversions. Existing numeric scalar enforcement remains
unchanged. Outside active semantic generic scopes, the new binding-local predicate
selects `bool`, canonical `String`, and one-dimensional fixed arrays over the four
numeric spellings with positive count; numeric arrays infer every element plus an
integer index before IR. Lowercase `string`, custom/contextual/structural annotations, nonnumeric arrays,
and all new annotation/array behavior inside generic semantic scopes remains
quarantined under explicit preservation controls. The slice does not admit a new
type, conversion, assignment, ownership, aggregate-layout, or backend behavior.

## Founding-framework checkpoint

- The tracked nine-page primary design paper is now treated as Aero's governing
  vision input, not as current implementation evidence.
- The tracked Claude strategy PDF is a truncated one-page capture. Its preserved
  execution-quality guidance and AI/ML-infrastructure recommendation are usable;
  absent continuation is not inferred.
- `FRAMEWORK_ALIGNMENT.md` records source authority, current gaps, an execution-
  quality scorecard, and the Aero-native AI/ML infrastructure flagship direction.
- `Roadmap.md` now follows the founding Design -> Minimal Prototype -> Self-Host ->
  Stabilize -> Optimize path through explicit evidence gates. Current position is
  Minimal Prototype / correctness recovery; historical v1.0/completed-phase labels
  do not establish stability.
- This checkpoint does not broaden `CORE-009` or authorize speculative aggregate,
  ownership, accelerator, or benchmark semantics.

## Repository state

- Upstream: `https://github.com/RobVanProd/aero.git`
- Default branch: `master`
- Starting commit: `8f8c7337a4008082fd2a443fcc814b5847b8663f`
- Starting commit date: `2026-05-28T21:13:40-04:00`
- Current branch: `agent/aero-integration`
- Public draft PR: `https://github.com/RobVanProd/Aero/pull/4`
- Current public preregistration head:
  `4f31f0ca3941389f2cc730136c2540301ee5bfe0`. Three independent reviewers approved
  its exact staged diff `9316f77aed456729624c2d86afaf7110487af84b` and tree
  `bd782da2b5881c1eb50a614400d73b1bb924b033` with no P0-P3 findings. All eight
  public checks pass and the draft PR is cleanly mergeable.
- Current public acceptance-closure head:
  `1535ce2a214f512c140535e7c42799af1f920d5c`. Its exact reviewed staged diff is
  `6e05c26763ed3a1c6e4ec359361867f76e9d4c4c` and tree is
  `b3a6bf38769579dbfc0fa0da5c4881620f7129c3`. Three independent reviewers
  approved it with no P0-P3 findings after two backend-evidence precision
  corrections; all eight public checks pass and the draft PR is cleanly mergeable.
- Accepted `CORE-014` implementation:
  `c56b1d561930a042eeff214196fd1b4f05a77fb6`. Its exact reviewed staged diff is
  `687dd5f3d6360dfd7822e7809944f63d4caccfdd` and tree is
  `869fca43edb8b5888bdec01d0bfc7cdecfa451a5`. Three independent reviewers
  approved it with no P0-P3 findings, the focused 5/5 target and exact complete
  local gate pass, and all eight public checks pass. Stable Linux CI resolved the
  documented LLVM 22 tools, completed build/init/check/run, returned status zero,
  and observed exactly one anchored `Output: Hello, Aero!` line.
- Historical `CORE-014` tests-only red checkpoint:
  `fc77e9979f996aaa0110ba48246b24ebca67acbd`. Its three intended Quick Start
  contracts failed after all earlier public test steps passed; this remains red
  evidence, not an accepted implementation.
- Prior clean `CORE-013` acceptance closure:
  `18526ff7a80db222c1348496f24f710d09249dfc`. All eight public checks pass.
- The `CORE-014` red checkpoint's exact reviewed staged diff is
  `b02c2bad25a28ec069303c02fa39de68b64561e8` and tree is
  `f301087d2749d4425bc7d913b3109b1b7aab64e2`. Three independent reviewers
  approved it with no P0-P3 findings. Focused local evidence was two controls
  passing and exactly three intended documentation/workflow failures. Public
  compiler-test and stable Rust jobs reproduced those same failures; the unchanged
  nightly matrix job was cancelled by matrix fail-fast after stable failed.
- Accepted `CORE-013` implementation code commit:
  `a78dd004aa37c39212711027b777698118d9dc02`. All eight implementation checks pass.
- Prior `CORE-012` acceptance-documentation head:
  `b7bb42958e78fb97ea0d991fa3f4cdb40bbcce2f`.
- Earlier published project-control checkpoint:
  `c0c044256a5922605e0dde8446b4c40cb250fd56`.
- Published `CORE-012` tests-only red checkpoint:
  `57c4ec70190822cb4552d313e5e7ea0f2dc5cbed`; exact staged diff
  `4058775145e68aa9a5512853c04b0dde04730464`, tree
  `227254ef8177d8e15b69c42bd1e2d94c1442879a`. Three independent reviewers
  approved the snapshot with no P0-P3 findings. Direct registry evidence was
  7 pass / 5 intentional failures; the CLI matrix was 0 pass / 6 intentional
  failures. The full gate stopped at 134 pass / the same five intended failures.
- Published accepted `CORE-012` implementation:
  `6780a23cd8b63df124477c7db1190d61dd25f3b8`; exact reviewed diff
  `05e55496f6664713192b2dbf94eca785abe2931d`, tree
  `85ed76ab0141409796e167704e4100dd4d15c26f`. Direct registry tests pass 12/12 in
  both library and binary targets; CLI quarantine/local/dry-run/help tests pass 7/7.
  The complete `./tools/test.sh` gate passes 139 library, 148 binary, every active
  integration suite, formatting, correctness Clippy, and doc tests; 38 pre-existing
  Phase 5 tests remain explicitly ignored. Three independent reviewers approved the
  exact snapshot with no P0-P3 findings. Both compiler-test workflows, Rust stable/
  nightly, all CodeQL language analyses, and aggregate CodeQL pass publicly.
- Published accepted `CORE-011` head:
  `a711dd5f3802095a4ecbe2dea3d45003675e7459`; exact reviewed implementation
  diff `60fe607413ebc03e9aa5d6296d9067d8cc95d89d`, tree
  `7c57c082e9d5f68afd5c6a4769d9d531a0116642`.
- Published `CORE-011` tests-only red checkpoint:
  `9c31820fdc5a252e29d5c62c96ff89f5a4a63eb8`; exact staged diff
  `badb9d0e8d6059927d949994b39f617fe2f404a8`, tree
  `540a187db87aff5ec0b2964b0c140c6caf9402a4`. Three independent reviewers
  approved the snapshot with no P0-P3 findings. Local red evidence was 2 pass / 5
  intentional failures in the module matrix and 3 pass / 4 intentional failures
  in the cache matrix; the full binary suite had only those four intended failures.
- Accepted `CORE-011` implementation: the shared collector is crate-private,
  every preregistered file-backed caller uses it, source-only `compile_program`
  rejects `mod`, nested declarations fail explicitly, module collection precedes
  cache lookup, and the frozen V1 identity excludes host paths. Both focused
  seven-test suites and the complete `./tools/test.sh` gate pass. Three independent
  reviewers approved the exact implementation snapshot with no P0-P3 findings.
  Both compiler-test jobs, Rust stable/nightly, all CodeQL language analyses, and
  the aggregate CodeQL check pass at the public implementation head.
- Published accepted `CORE-010` head:
  `db349ef81f145ee571c053f73fb03c831cea719a`.
- Checked-IR/LLVM-verifier implementation commit:
  `d08653c646edae33693f91e2b2f446c76f1bd8a6`; exact reviewed staged diff
  `9534765a46b130d215a1d1e869de234163bb0daf`, tree
  `e0e720f398b1201b4d798101eea4059fc1de56b2`.
- Linux mixed-entry CI repair: exact reviewed staged diff
  `d5f0fd3891da5cff75bd5306006e993ca4b4f301`, tree
  `782b4d5319d73248bee825683e403b8265eb4fbc`, integrated as
  `db349ef81f145ee571c053f73fb03c831cea719a`.
- Published integration head / accepted `CORE-010` red checkpoint:
  `26560a45905015b7891ddebeb749d0097c05cbaa`.
- Founding-framework alignment is published at
  `fba121f0213b7f604d4c73032019c872680a3136`.
- `CORE-009` tests-only red checkpoint:
  `1e76a0610ef778303548096ef634a5f02b678fe9`. The new nine-test aggregate suite
  is exactly 3 pass / 6 expected fail on production: parser/positive controls and
  established child precedence pass, while ordinary, recursive, default/nested,
  ordering/inference-only, root CLI, and direct-module CLI families expose false
  success, wrong outer diagnostics, zero/drop lowering, successful status, and
  requested artifacts. No failure relies on a parse error or unwind. Reclassified
  controls independently pass 59 frontend, 8 field, and 15 Match tests; formatting
  and diff checks pass.
- `CORE-009` production candidate: owner `bf6a7ef`, integrated as exact
  `a8879310fe04a28b368437d1932e01972b7e9cee`. The only production change is one
  return after existing source-order recursive StructLiteral field preflight. The
  exact diagnostic is `Struct construction expressions are not supported.` Owner
  verification passes the complete gate plus the focused matrix. Lead verification
  independently passes 9 Struct, 59 frontend, 8 field, 15 Match, 16 tuple, 14
  modulo, 13 function-contract, 18 numeric-annotation, and 12 strict-lexing tests.
  Public documentation and the complete gate pass at `3410f1f`; coordinated
  project-control corrections and the new exact-candidate gate pass at `daa024d`.
  Two fresh non-owner reviewers approve exact `daa024d` with no P0-P3 findings.
- `CORE-009` closure is published at exact
  `555fea27e6cb8e0a07df20b5189dfc2b5aebce46` on draft PR #4. Both compiler-test
  jobs, Rust stable/nightly, and all CodeQL jobs pass at that public head.
- `AUDIT-016` is complete. Fresh evidence ranks fallible typed scalar IR admission
  and verification above pipeline consolidation, MethodCall, custom EnumVariant,
  and Deref slices. String comparison and constant `1 / 0` unwind; Boolean storage
  can emit type-invalid LLVM; untyped codegen can silently ignore instructions.
- `CORE-010` and `DEC-015` define the checked additive APIs, logical
  Int/Float/Bool/Void representation, legacy numeric-storage compatibility limit,
  stronger tool-independent `check` admission, mandatory pure-Rust IR verification,
  and LLVM 22 external verification modes. The accepted public implementation now
  enforces that contract on trusted paths.
- The isolated `CORE-010` tests/CI-only red checkpoint is published at exact
  `26560a45905015b7891ddebeb749d0097c05cbaa`; its exact staged diff hash is
  `c01fc2365eb5b415c022be997062e4605812b62b`. Three independent reviewers approve
  that exact diff with no P0-P3 findings. Local evidence records typed admission as
  1 pass / 7 intentional failures and the external LLVM CLI matrix as 3 pass / 9
  intentional failures; the remaining checked public/private targets stop only on
  the preregistered missing API and injected-seam symbols. Parser/declaration
  controls for reclassified unsupported forms remain green.
- Public CI confirms the environment/corpus side of the checkpoint. Both compiler
  workflows install LLVM 22 and prove `opt-22` rejects the known-invalid fixture.
  Rust stable/nightly install LLVM 22 plus Clang 22 and pass all four CPU example
  verification/execution steps, with `opt-22` preceding `llc-22`/`clang-22`. Test
  jobs then fail at the deliberate checked-API contract boundary. This is accepted
  red evidence, not an accepted production candidate.
- The accepted `CORE-010` production implementation makes all focused typed-admission,
  checked-IR, LLVM-verifier, cache, conformance, profiler, and compatibility
  controls green. `cargo check --all-targets` and the complete `./tools/test.sh`
  repository gate pass. External verification is enforced after final graph/target
  transformation and before cache/write/native publication; tool-independent
  `check` and conformance remain internal-verifier paths. Three independent
  reviewers approved the exact implementation and CI-repair diffs with no P0-P3
  findings. Rust stable/nightly, both compiler-test workflows, and all CodeQL jobs
  pass at public head `db349ef`.
- Accepted `CORE-008` candidate:
  `b74d91adeda04688ec37598beebffad458538c39`. All trusted parsed source bodies,
  including default trait method bodies, route Match expression roots through the
  existing child-first preflight before IR. The complete gate and two fresh
  independent reviews pass.
- Accepted `CORE-009` candidate and complete gate: exact clean
  `daa024dbf10d1defe06d8ab200c2d21c0a9c1dc6` passes 112 library, 119 binary,
  11 fatal-parser, 59 frontend, 13 function-contract, 18 numeric-annotation,
  12 strict-lexing, 8 field, 15 Match, 14 modulo, 9 StructLiteral, and 16 tuple
  tests. All 38 Phase 5 tests remain intentionally ignored. Formatting, Clippy
  correctness, all-target compilation, and doc tests pass.
- Last accepted public full-gate code commit:
  `a78dd004aa37c39212711027b777698118d9dc02`.
- Worktree: the new tests-only `CORE-015` target plus minimal evidence updates; no
  production source has changed. Its focused result is exactly 8 passing preservation
  groups and 8 intended failing contract groups. Failures expose all four selected
  binding false successes, semantic numeric-array element/index/child ordering,
  checked-IR numeric and selected binding parity, universal binary metadata,
  non-generic impl behavior, generic-scope cleanup after error, public library
  no-unwind rejection, and root/direct-module CLI status/diagnostic/artifact behavior.
  Both CLI layouts exit zero from `check` and `build`; each build publishes its
  requested LLVM artifact. The exact `./tools/test.sh` gate passes formatting and
  correctness Clippy, then stops only at this intended red target. A separate
  `cargo test --no-fail-fast` proves every prior active target passes and reports
  this target as the sole failure (8 pass / 8 fail); 38 pre-existing Phase 5 tests
  remain ignored. Production compiler behavior remains the accepted public
  implementation `a78dd00` because `CORE-014` changed no compiler code.
  One earlier full-gate attempt stopped in the unchanged
  `cli_status_contract_tests`; that target immediately passed 7/7 in isolation and
  the unchanged complete gate passed on rerun. The interruption is not reproduced
  or attributed to `CORE-014`, but remains residual pre-existing flake uncertainty.

## Environment and verification

- Host: Windows x86_64
- Shell for baseline gate: Git Bash launched from PowerShell
- Rust: `rustc 1.97.1 (8bab26f4f 2026-07-14)`
- Cargo: `cargo 1.97.1 (c980f4866 2026-06-30)`
- Required command: `./tools/test.sh`
- Windows invocation used: prepend `C:\Users\usa50\.cargo\bin` to `PATH`, then
  run Git Bash with `./tools/test.sh`.
- Baseline result: PASS on the starting commit; formatting, Clippy correctness,
  unit/integration tests, and doc tests completed with no test failures.
- Initial environment issue: Rust was absent and the first two baseline attempts
  stopped at `cargo: command not found`; the stable minimal toolchain plus
  `rustfmt` and `clippy` were installed, then the gate passed.
- LLVM tools in the Windows environment: `clang`, `llc`, `opt`, and `llvm-as`
  unavailable on discovered paths.
- `CORE-010` red CI at `26560a4`: pinned LLVM 22 installation and known-invalid
  rejection pass in both compiler jobs; stable/nightly LLVM verification and native
  execution of `return15`, `variables`, `mixed`, and `float_ops` pass. The subsequent
  Cargo test failures are confined to the intentional missing checked APIs and
  private injected seams recorded by the red checkpoint.
- Upstream recheck before `CORE-001`: `origin/master` still equals the recorded
  starting commit.
- Remote CI: CI run `26611985062`, Rust CI run `26611985038`, and latest CodeQL
  run `30685526232` all completed successfully for upstream commit
  `8f8c7337a4008082fd2a443fcc814b5847b8663f`.
- `CORE-001B` verification at `6ce85922`: focused fatal-parse tests 11/11;
  complete gate PASS with 106 library, 111 binary, and 59 frontend tests; 38
  pre-existing phase-five tests remain ignored.
- Fresh manual root and imported-module builds both exited 1, reported the source
  file at `1:5`, and created no requested LLVM artifact.
- `CORE-002` verification at `b988318`: full gate PASS with 111 library, 118
  binary, 11 fatal-parse, 12 strict-lexing, and 59 frontend tests; 38 pre-existing
  phase-five tests remain ignored. Conformance remains 3/3 cases and 4/4 checks.
- Fresh manual unexpected-character and overflow builds, plus direct-module docs,
  exited 1 with located lexical errors and created no requested output.
- `CORE-003` verification at `8d5d8e7`: focused function-contract tests 13/13;
  full gate PASS with 112 library, 119 binary, 11 fatal-parse, 59 frontend,
  13 function-contract, and 12 strict-lex tests. The 38 pre-existing phase-five
  tests remain ignored. Two independent reviewers approved the exact clean SHA;
  fresh black-box invalid-program probes exited nonzero and wrote no LLVM artifact.
- `CORE-004` verification at `bc9a148`: focused numeric annotation and lexical-scope
  tests 18/18; function-contract tests 13/13; full gate PASS with 112 library,
  119 binary, 11 fatal-parse, 59 frontend, 13 function-contract, 18 annotation,
  and 12 strict-lex tests. The 38 pre-existing phase-five tests remain ignored.
  Two independent reviewers approved the exact clean SHA after public no-unwind,
  scope-provenance, artifact, callable-restoration, and analyzer-reuse probes.
- `CORE-005` verification at `302211e`: focused modulo tests 14/14;
  function-contract tests 13/13; annotation tests 18/18; full gate PASS with
  112 library, 119 binary, 11 fatal-parse, 59 frontend, 13 function-contract,
  18 annotation, 12 strict-lex, and 14 modulo tests. The 38 pre-existing phase-five
  tests remain ignored. Two non-owner reviewers approved the exact clean SHA after
  fresh shared-helper and public/CLI diagnostic, no-unwind, no-panic, module,
  precedence, nonnumeric, unary, nested, positive-control, and artifact probes.
- `CORE-006` verification at `cbbe049`: focused tuple tests 16/16; modulo 14/14;
  function-contract 13/13; annotation 18/18; strict-lex 12/12; complete gate PASS
  with 112 library, 119 binary, 11 fatal-parse, 59 frontend, 13 function-contract,
  18 annotation, 12 strict-lex, 14 modulo, and 16 tuple tests. The 38 pre-existing
  phase-five tests remain ignored. Two non-owner reviewers approved exact clean
  `cbbe049` after fresh structural and 18-route black-box public/CLI diagnostic,
  no-unwind, no-panic, no-artifact, nesting, precedence, and positive-control probes.
- `CORE-007` verification at `4e10d479`: focused field tests 8/8; tuple 16/16;
  modulo 14/14; function-contract 13/13; annotation 18/18; strict-lex 12/12;
  complete gate PASS with 112 library, 119 binary, 11 fatal-parse, 59 frontend,
  13 function-contract, 18 annotation, 12 strict-lex, 14 modulo, 16 tuple, and
  8 field tests. The 38 pre-existing phase-five tests remain ignored. Two non-owner
  reviewers approved exact clean `4e10d479` after independent 25-route public and
  27-route public/CLI/module matrices, plus structural, nesting, precedence,
  no-unwind, no-panic, no-artifact, parser-distinction, and positive-control probes.

## Audit agents

- `AUDIT-001` specification: complete; 3 high, 6 medium, 1 low finding.
- `AUDIT-002` frontend: complete; silent lexical corruption, lost semantics/spans,
  recovery/API panic risks, and dormant coverage confirmed.
- `AUDIT-003` type soundness: complete; active integer/double/zero fallbacks,
  unenforced contracts, ownership/generic gaps, and scope leakage confirmed.
- `AUDIT-004` IR/code generation: complete; untyped values, invalid boolean/CFG
  lowering, parse-to-invalid-LLVM false success, and absent verification confirmed.
- `AUDIT-005` runtime/backends: complete; CPU execution path separated from ROCm
  object plumbing and absent CUDA run; graph/quantization claims reclassified.
- `AUDIT-006` tooling: complete; duplicate pipelines, ignored options, status-code
  failures, heuristic LSP, shallow modules, and registry risks confirmed.
- `AUDIT-007` tests/fuzzing: complete; duplicates, 38 ignored, 299 dormant, and
  absent compile-fail/fuzz/differential/verifier/hardware gates inventoried.
- `AUDIT-008` benchmarks/claims: complete; compilation series invalid, lexer
  evidence partial, GGUF external/single-run, and protocol gaps classified.
- `AUDIT-009` numeric binding boundary: complete after two review amendments;
  parser retention, semantic/IR discard, seven black-box false-accept families,
  unified-double local storage, IR scalar leakage, and semantic compatibility-table
  leakage are characterized and the eligible slice is controlled at `bc9a148`.
- `AUDIT-010` unsupported-expression boundary: complete; three independent audits
  agree that `%` is the smallest bounded fail-open family. Five numeric forms pass
  semantic `check` then panic in both public/CLI compilation. Constant integer `/0`,
  unsupported comparisons, and invented-zero aggregates/methods are separate tasks.
  The selected `%` boundary is controlled at `302211e`.
- `AUDIT-011` fabricated-zero expression families: complete at `704b3328`. Three
  independent read-only audits compared fields, tuples, matches, methods, closures,
  arrays, structs, enums, borrows, and nested forms. Two selected tuple literals and
  tuple projections as the smallest coherent family; one ranked field access first
  by AST-form count but confirmed the tuple family's shared zero behavior. The lead
  selected tuples because `(7, 9).0` is a valid specification-backed value expression
  that silently emits zero, while field access intersects broader struct semantics.
- `CORE-006` tuple value boundary: accepted at exact clean `cbbe049`. Both
  non-owner reviewers approved after independent structural and black-box probes;
  trusted public/CLI routes reject tuple literals/projections before IR with one
  exact diagnostic, no unwind/panic, and no requested artifact.
- `AUDIT-012` adjacent failure boundaries: complete at clean `52d3415`. Three
  independent audits compared field access, all-six string comparisons, and
  constant/variable/float/mixed zero division across semantics, both IR paths,
  public compilation, CLI, modules, artifacts, nesting, and controls. All ranked
  FieldAccess first as the only one-node silent-miscompile family with no active
  value-preserving path and no required layout/arithmetic policy.
- `CORE-007` field value boundary: accepted at exact clean `4e10d479`. The
  tests-only red checkpoint is `7346edd`, the one-line receiver-first production
  behavior is `75dbfba`, and user-facing field status and the matrix are corrected
  at `5dcb70b`. Both non-owner reviewers approved after independent structural and
  black-box attempts to falsify the complete-gate candidate.
- `AUDIT-013` next-boundary comparison: complete at exact clean `9fc7d0e`. String
  comparison requires trustworthy operand typing and operator policy; MethodCall
  rejection requires a pre-IR capability predicate that preserves real array
  `.iter()`; zero division requires integer/runtime/IEEE policy. Match alone has one
  AST family, complete existing recursive preflight, no active value-preserving
  path, and no required execution semantics. A 23-case Match matrix produced 69
  public/check/build outcomes: 20 false successes and three established child
  diagnostics with retained precedence.
- `CORE-008` Match value boundary: preregistered at audit closure `648662b` under
  `DEC-012`. Exact diagnostic is `Match expressions are not supported.` Existing
  child-first scrutinee/arm traversal and tuple/field/void diagnostic precedence are
  frozen. Parser/AST/patterns remain; pattern/exhaustiveness/type/layout/ownership/
  evaluation/IR/backend semantics are explicitly outside the slice.
- `CORE-008` tests-only red checkpoint: owner `17e17c2`, integrated as `851731c`.
  Focused result is exactly 5 pass / 4 expected fail. All 21 ordinary Match forms
  falsely compile; 12/15 recursive parents falsely compile; root and direct-module
  check/build exit zero and create artifacts. Failure evidence records fabricated
  zero, empty root CFG, dropped calls, suppressed `/0`, current outer diagnostics,
  and artifact creation. No case parses incorrectly or unwinds. Parser 1/1 and prior
  field/modulo/tuple controls 38/38 pass; formatting passes.
- `CORE-008` production candidate: owner `aed4d0e`, integrated as `c826294`. The
  production diff is one error return after existing Match child traversal. Owner
  and lead independently pass 90/90 focused Match/field/tuple/modulo/function/
  annotation/strict tests, formatting, and `cargo check --all-targets`. Public docs
  and the matrix now classify Match as parsed but explicitly non-executable; two
  historical design summaries carry current-capability notices.
- `CORE-008` initial full gate: exact clean `08e7c2c` passed the complete repository
  gate (112 library, 119 binary, 11 fatal-parser, 59 frontend, and all focused
  boundary suites; 38 Phase 5 tests ignored), documentation, formatting, and Clippy.
- `CORE-008` initial review: rejected. Reviewer A found the sole parsed
  expression-bearing container escape: `TraitMethod.body` is retained by the parser,
  while `Statement::TraitDef` registers only required names and never visits default
  bodies. Match in such a body returns `Ok` through `compile_program`, check, and
  build, and build writes LLVM. Reviewer A passed 32/33 fresh routes and 7/7 frozen
  precedence probes. Reviewer B approved its independent 41-route matrix but did not
  include trait defaults; that approval is superseded by the counterexample.
- `CORE-008` corrective red: owner `58bb732`, integrated as `ad5e24d`. Six new
  aggregated tests preserve an exact 11-pass/4-fail red split on rejected production,
  covering eight default-body placements, tuple/field/void precedence, root/module
  CLI no-artifact contracts, parser retention, and syntax-only positive controls.
- `CORE-008` corrective production: owner `a3f4f29`, integrated as `a12f38e`. Only
  `semantic_analyzer.rs` changes: exhaustive syntax-only block/statement preflight
  plus a default-body hook with cleanup-safe type-parameter scope handling. Owner
  and lead pass 96/96 focused tests; formatting and all-target compilation pass.
- `CORE-008` corrective acceptance: exact clean documented `b74d91a` passes the full
  gate (112 library, 119 binary, 11 fatal-parser, 59 frontend, 13 function, 18
  annotation, 12 strict, 8 field, 15 Match, 14 modulo, 16 tuple; 38 Phase 5 ignored).
  Reviewer A approves after the exhaustive structural audit and 44 fresh public
  negatives; Reviewer B independently approves 225/225 outcomes across 75 negative/
  precedence routes. Syntax-only positives, parser retention, child order, no-panic/
  no-unwind/no-artifact behavior, and prior controls all pass.

## Current capability classification

Initial audit classification; see `CURRENT_CAPABILITY_AUDIT.md` and
`SPEC_IMPLEMENTATION_MATRIX.md` for stage evidence:

- Compiler regression baseline: passing locally.
- Repository stability: experimental.
- Formal conformance: three example cases plus four deterministic pipeline
  checks; this is not formal semantics proof.
- CPU source-to-LLVM/object/link/process path: present when external tools are
  available; current evidence is four small Linux CI exit-code programs plus the
  generated-project status/output path accepted by `CORE-014`.
- ROCm: interface/retarget/object-generation plumbing; no link/launch path or
  current-session hardware execution evidence.
- CUDA: selectable interface; CLI source states run support is not implemented.
- Public version: inconsistent (`0.3.0` package versus `1.0.0` README/CLI).
- Library compiler options: accepted but ignored by `compile_program`.
- Compiler architecture: binary and library declare overlapping modules.

## Known blockers and regressions

- No known baseline regression.
- The local shell required Rust installation before tests could run.
- Real backend verification may be blocked by absent LLVM/GPU toolchains or
  hardware; absence will be recorded rather than simulated.
- `run_aero_program` still calls `exit` internally after a valid CPU process;
  this pre-existing hidden termination violates the desired helper/API boundary
  and remains a separate tooling task.
- Legacy recovery lexing remains public for compatibility and LSP symbol recovery;
  trusted repository paths no longer feed it into semantics, IR, or artifacts.
- Numeric and void top-level function contracts are controlled at `8d5d8e7`.
  Initialized exact numeric `let` annotations are controlled at `bc9a148`;
  uninitialized, non-numeric, boolean/generic/composite contracts remain open.
- Function-local branch and epilogue termination improved in `CORE-003`, but the
  broader pre-existing unreachable-after-terminator CFG risk remains open.
- The tested scalar/callable IR scope exits and semantic compatibility scopes are
  controlled at `bc9a148`; general AST-to-IR fallibility, unsupported-expression
  fallbacks, and analyzer/backend invariants remain open.
- At `302211e`, `%` remains parsed but is rejected by shared semantic inference
  before IR with one stable diagnostic across trusted public and CLI paths.
  Negative/float/zero remainder execution semantics remain intentionally undecided.
- At `1fa67a2`, tuple literals and tuple projections remain parsed but are rejected
  recursively by active semantic preflight with one stable diagnostic before IR.
  Tuple types/patterns remain parsed; tuple layout and execution remain unimplemented.
- `CORE-010` now turns constant integer division by zero, string comparison,
  ordinary MethodCall, custom enum construction, Deref/Borrow, and other unsupported
  scalar fallbacks into checked errors on trusted paths; typed Boolean storage/calls/
  returns and checked IR/codegen verification are controlled at accepted `db349ef`.
  Dynamic division/overflow, aggregates, ownership, and direct callers of public
  unchecked compatibility APIs remain uncertified.
- At candidate `3410f1f`, StructLiteral values remain parser-visible but are
  rejected after source-order field preflight before inference or IR. Struct
  layout, initialization, ownership, ABI, lowering, and execution remain open.
- At accepted compiler head `a711dd5`, a root-level missing direct module fails
  every inventoried trusted file-backed route before cache lookup or publication;
  source-only `compile_program` rejects `ModDecl`, and nested modules fail explicitly.
  Module-bearing cache identity includes the exact ordered direct-source set while
  retaining the legacy no-module key. General CLI status handling, namespaces,
  visibility/import semantics, recursive graphs, and full pipeline consolidation
  remain separate.
- Accepted public `CORE-013` at `a78dd00` contains the false-success CLI boundary.
  The tracked
  `performance_benchmark.py` compilation timings are an invalid measurement of a
  bare-source usage path; public and historical lexer evidence remain separately
  qualified, and the external GGUF record remains reference-only.
- `AUDIT-021` at clean public head `1535ce2` reproduced active R-002 false success:
  initialized `String`, `bool`, custom named, fixed-array element-type, and
  fixed-array length annotation mismatches all pass `check` and `build`, with a
  requested LLVM artifact. Exact String/bool and homogeneous-array controls pass.
  An uninitialized read fails in semantics. Mixed `[1, 2.5]` and float indexing
  fail closed without artifacts, but only after semantic success. R-004 remains
  stopped because the mutable-reference Copy/provenance defect requires unfrozen
  ownership work across more than two phases.

## Exact next action

Stage and independently review the exact `CORE-015` tests-only red snapshot. Only
after all three reviewers approve that exact diff and tree may it be committed and
published; production implementation begins only after the public red checkpoint is
recorded. Do not change lexer/parser syntax, type representation, conversions,
assignment, ownership, aggregate layout/codegen, backend behavior, version policy,
benchmarks, registry, release state, or `master`.

## Unauthorized actions

Do not publish releases/packages/registry changes/benchmark claims; force-push or
rewrite history; delete substantial evidence; change the language's fundamental
identity; make incompatible syntax/semantics changes without a migration plan;
modify downstream repositories; spend money; use unbounded paid compute; handle
credentials; or perform destructive system operations.
