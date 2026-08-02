# Aero Project State

Last updated: 2026-08-02 (America/New_York)

## Current objective

Milestone 8 corrective cycle — close the default-trait-body escape that caused the
first `CORE-008` candidate to fail independent acceptance review.

## Active hypothesis

A syntax-only statement/block traversal limited to default trait method bodies can
funnel their expression roots into existing child-first preflight without activating
uncertified trait, name, type, return, ownership, pattern, or execution semantics.

## Repository state

- Upstream: `https://github.com/RobVanProd/aero.git`
- Default branch: `master`
- Starting commit: `8f8c7337a4008082fd2a443fcc814b5847b8663f`
- Starting commit date: `2026-05-28T21:13:40-04:00`
- Current branch: `agent/aero-integration`
- Rejected `CORE-008` behavior candidate:
  `c8262944a3462b9956be107bdf8dc71cdd9df067`; exact documented review SHA
  `08e7c2c8eccc06cdcdd216b6973a68d24e751e32`. Ordinary analyzed public/CLI routes
  reject Match before IR, but parsed default trait method bodies bypass preflight
  and can still reach artifact generation. No production acceptance exists.
- Last full-gate commit: `08e7c2c8eccc06cdcdd216b6973a68d24e751e32`.
- Worktree: corrective documentation in progress after exact-SHA review rejection.

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

## Current capability classification

Initial audit classification; see `CURRENT_CAPABILITY_AUDIT.md` and
`SPEC_IMPLEMENTATION_MATRIX.md` for stage evidence:

- Compiler regression baseline: passing locally.
- Repository stability: experimental.
- Formal conformance: three example cases plus four deterministic pipeline
  checks; this is not formal semantics proof.
- CPU source-to-LLVM/object/link/process path: present when external tools are
  available; current evidence is four small Linux CI programs.
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
- Constant integer division by zero independently panics during IR constant folding.
  Float, variable, unary, and mixed zero forms diverge. Unsupported methods and
  other composite forms can still fabricate scalar zero, and some semantically
  accepted comparisons can panic or generate type-invalid LLVM. Ordinary analyzed
  Match values fail closed in rejected candidate `c826294`, but default trait method
  bodies bypass preflight; direct AST-to-IR bypass also remains open.
  MethodCall, string comparison, division, aggregates, and ownership are separate.
  Named field values remain fail-closed in accepted `CORE-007` at reviewed
  `4e10d479`.

## Exact next action

Commit the corrective boundary amendment, add failing public and CLI regressions for
Match in default trait method bodies, and prove they fail on `c826294` without parse
errors or unwinds. Then add only a syntax-only default-body statement/block preflight
funnel in `semantic_analyzer.rs`; do not invoke full block/statement analysis.

## Unauthorized actions

Do not publish releases/packages/registry changes/benchmark claims; force-push or
rewrite history; delete substantial evidence; change the language's fundamental
identity; make incompatible syntax/semantics changes without a migration plan;
modify downstream repositories; spend money; use unbounded paid compute; handle
credentials; or perform destructive system operations.
