# Aero Project State

Last updated: 2026-08-02 (America/New_York)

## Current objective

Milestone 6 — implement the preregistered `CORE-006` fail-closed tuple-value
boundary tests-first, without inventing aggregate layout or projection semantics.

## Active hypothesis

The analyzer's existing exhaustive expression preflight can reject both tuple
value AST forms recursively across ordinary, discarded, closure, and nested routes
without changing parser/AST/types, arrays, IR shape, or backend contracts.

## Repository state

- Upstream: `https://github.com/RobVanProd/aero.git`
- Default branch: `master`
- Starting commit: `8f8c7337a4008082fd2a443fcc814b5847b8663f`
- Starting commit date: `2026-05-28T21:13:40-04:00`
- Current branch: `agent/aero-integration`
- Current commit: `704b3328b0c22b2fab02258ad81486c62fc3e1be`
- Last verified commit: `302211e6226c1580a8b4aec66790b37c03888db6`
- Worktree: clean before this `CORE-006` preregistration. `704b3328` contains only
  the accepted `CORE-005` control-document closure beyond the last fully verified
  candidate; no production behavior changed.

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
- Constant integer division by zero independently panics during IR constant folding.
  Unsupported tuple/match/field/method forms can still fabricate scalar zero, and
  some semantically accepted comparisons can panic or generate type-invalid LLVM.

## Exact next action

Commit the `CORE-006` control contract, then have one isolated owner add only the
focused red integration test file. Preserve the red checkpoint before permitting
the one-file semantic implementation.

## Unauthorized actions

Do not publish releases/packages/registry changes/benchmark claims; force-push or
rewrite history; delete substantial evidence; change the language's fundamental
identity; make incompatible syntax/semantics changes without a migration plan;
modify downstream repositories; spend money; use unbounded paid compute; handle
credentials; or perform destructive system operations.
