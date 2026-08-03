# Aero Project State

Last updated: 2026-08-03 (America/New_York)

## Current objective

Milestone 26 `AUDIT-029` preregistration — `CORE-022` is complete and exact status
sync `21153f3` is triple-reviewed, all-eight public green, and clean. The next work is
a full-set, read-only residual-risk feasibility ranking; no implementation is
authorized.

## Active hypothesis

A delta-aware comparison of all eleven remaining residuals can select one distinct,
semantically frozen, deterministic tests-first slice within two compiler phases—or
stop explicitly—without inheriting AUDIT-028 ordering or repeating an accepted
boundary.

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
- `CORE-020` record-only closure `5a8cd06`, tree `df4a04a`, passes compiler runs
  `30835593703`/`30835597576`, stable/nightly Rust run `30835597620`, all three
  analyses in CodeQL run `30835594365`, and aggregate `91759990615`. The selected
  unsupported-options boundary is closed; real option behavior and broad R-006
  convergence remain open.
- `AUDIT-027` public basis `aa3e7a8`, tree `4caa5c33`, passes compiler runs
  `30836250279`/`30836251909`, stable/nightly Rust run `30836255407`, all three
  analyses in CodeQL run `30836248101`, and aggregate `91762198170`. The worktree
  remained clean and auditors used static repository evidence only.
- Accepted public `CORE-019` final-state sync head:
  `25dec51e7fb24a5dd835712568242d685af649cf`. Three independent reviewers approved
  exact record-only diff `a3cd465fab08c4c9b6b238c7aadd4a39a4d06c3d` and tree
  `46828e7d715c6489eb2c7a661a7ef95b7cb4555b` with no P0-P3 findings. Both compiler-
  test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate CodeQL pass
  in runs `30830484863`, `30830489796`, `30830490379`, and `30830483828`, with
  aggregate check `91743120769`; draft PR #4 remains open and mergeable and upstream
  `master` remains `8f8c733`.
- Accepted public `CORE-019` closure head:
  `63b66295544d41634f790face005d0fcfc64b41a`. Three independent reviewers approved
  corrected record-only diff `b4fd6bc195f70712fbcd0f022d5dcbbcad7128c9` and tree
  `2e88685021de6a7948e6b5ffb69250676764f7f5` with no P0-P3 findings. Both compiler-
  test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate CodeQL pass
  in runs `30829963152`, `30829970545`, `30829968789`, and `30829962982`, with
  aggregate check `91741344282`; draft PR #4 remains open and mergeable and upstream
  `master` remains `8f8c733`.
- Accepted public `CORE-018` final-state sync head:
  `d0bd54e93ff9fda9e769dd29abcec02a1f550e9a`. Three independent reviewers approved
  corrected exact diff `a4034521b5976f4c737871d5be7e93d2a1f34bfb` and tree
  `21e72079679550b73935b56d87e4e062fc48d88e` with no P0-P3 findings after correction
  of one CUDA stage-precision defect. Both compiler-test jobs, stable/nightly Rust,
  all three CodeQL analyses, and aggregate CodeQL pass in runs `30824106058`,
  `30824111861`, `30824110412`, and `30824105642`, with aggregate check
  `91721342986`; draft PR #4 remains open and mergeable and upstream `master` remains
  `8f8c733`.
- Accepted public `CORE-018` closure head:
  `2e0e17fde6d9b11c2f5705c45b23468e0b04cbf0`. Three independent reviewers approved
  exact record-only diff `3d0a17f75e74446d5db0a132084fb3ca7973c6ed` and tree
  `83c9676f905dde55d5da52ed3961607c2aec9d55` with no P0-P3 findings. Both compiler-
  test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate CodeQL pass
  in runs `30823259890`, `30823261072`, `30823260717`, and `30823257183`, with
  aggregate check `91718428033`; draft PR #4 remains open and mergeable and upstream
  `master` remains `8f8c733`.
- Public `CORE-016` preregistration head:
  `1575914e7ab1f3c70793c77a1d82b7b3a78bb441`. Three independent reviewers approved
  exact staged diff `321fb61c3932cd0663bc5bcbc0aecb02361ab010` and tree
  `4933dc2e9297cc5d7d0742c28081571e3fc23c5f` with no P0-P3 findings after the first
  snapshot was rejected and corrected. Both compiler-test jobs, stable/nightly Rust,
  all three CodeQL analyses, and aggregate CodeQL pass; draft PR #4 is mergeable.
- Public `CORE-016` tests-only red head:
  `4b94dbd55465d2f94c2e7840f26ce5f73e571f30`. Three independent reviewers approved
  exact staged diff `b734773e6f1f4bb9c9561dc089e72b103e3b4e25` and tree
  `488687b20c882c78c8e801d46cdb0bf817d7f421` with no P0-P3 findings. Both
  compiler-test jobs and nightly Rust reproduce the intended 2-pass/5-fail target;
  stable reached its test step before matrix fail-fast cancellation. All three
  CodeQL analyses and aggregate CodeQL pass; draft PR #4 remains mergeable.
- Public `CORE-016` implementation head:
  `cc984d0afe4c63f3c322f8da7c34fc666f8ec072`. Three independent reviewers approved
  exact canonical staged diff `e0c2bbb61f33ea53e1c07d472a21a631170c22e7`
  and tree `8d5ba37b0a58c715cf72721ade23471c5fa4fa7c` with no P0-P3 findings. Both
  compiler-test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate
  CodeQL pass; draft PR #4 is open, mergeable, and remains intentionally draft.
- Accepted public `CORE-016` closure head:
  `ea036f2e71a4f67b1f8c6f711488f02f65fc4ad5`. Three independent reviewers approved
  exact record-only diff `7b24a58e7475700423dc66da368a22b97f9c31e8` and tree
  `4c7f526617ecb8e3a0c28622f8eca44dac627981` with no P0-P3 findings. Both
  compiler-test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate
  CodeQL pass; draft PR #4 remains open and mergeable.
- Accepted public `CORE-016` final-state sync head:
  `8869ecab0a7aadb51d9da193bf480a6fa97a9b3e`. Three independent reviewers approved
  corrected exact diff `8379a2c67e4b72c54d92f66480bd836805582589` and tree
  `4318bd3f0eea4dda7f6264ac5e9ae1694d0d5960` with no P0-P3 findings after two stale
  state anchors were rejected and fixed. Both compiler-test jobs, stable/nightly Rust,
  all three CodeQL analyses, and aggregate CodeQL pass; draft PR #4 is mergeable.
- Public `CORE-017` preregistration head:
  `2c61535092f22f2f513aac0fcee9d34d9c621212`. Three independent reviewers approved
  exact diff `ebe348e00721596f768b900547b9d19b56e44df4` and tree
  `1d890b93351e54fb6903aa952957494a517d40a9` with no P0-P3 findings. Both
  compiler-test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate
  CodeQL pass; draft PR #4 is open and mergeable.
- Public `CORE-017` implementation head:
  `8be8c21696cf98602c82e1e5e4fdfc6bf10e9777`. After the first snapshot was rejected
  for an underasserted method body, all three independent reviewers approved corrected
  exact diff `a417c7e3c076e7ff6951ce9c181ea99d6bdfa3b6` and tree
  `83bf4f0ba8f973e7ec39167e53114cf5714fd03b` with no P0-P3 findings. Both
  compiler-test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate
  CodeQL pass; draft PR #4 is open and mergeable.
- Accepted public `CORE-017` closure head:
  `3dd3bb41d601ddfe5f7ac2722cde39bad124973d`. Three independent reviewers approved
  exact record-only diff `3239da0b313f819bad7beef69cea8b6bd5e658a8` and tree
  `166ec7a5e4156da1cefeb9f921a31714461c6839` with no P0-P3 findings. Both
  compiler-test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate
  CodeQL pass; draft PR #4 remains open and mergeable.
- Accepted public `CORE-017` final-state sync head:
  `9ddc571ac47f1c2ffcf7a737e4be442f01c0f78b`. Three independent reviewers approved
  exact record-only diff `1c5af4fe131ad73eebecc6b17cc2428686ec431e` and tree
  `20ab4e6b87ead659a138e57bc27c073f817d15cb` with no P0-P3 findings. Both compiler-
  test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate CodeQL
  pass in runs `30814906589`, `30814909709`, `30814909985`, and `30814903903`; draft
  PR #4 remains open and mergeable and upstream `master` remains `8f8c733`.
- Accepted public `CORE-015` final-state sync head:
  `c612f3bea133f308cd71c6f8e5fb9ad708e51e6b`. Three independent reviewers approved
  exact staged diff `674b1831accef7b714ba21799249f346cc5a7491` and tree
  `224b9d790115de92d381a956e4487725325140f2` with no P0-P3 findings. Both
  compiler-test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate
  CodeQL pass; draft PR #4 is mergeable.
- Accepted public closure head:
  `5d7aae0f5626813249b6de983a229dbbb1e4fef8`. Three independent reviewers approved
  exact closure-record diff `a8e4059e71991c9d7a274234f91dd225bea61c01` and tree
  `19fea4153397958656b57adac6b70556d4a997c9` with no P0-P3 findings. Both public
  compiler-test jobs, stable and nightly Rust, all three CodeQL analyses, and
  aggregate CodeQL pass.
- Accepted public `CORE-015` implementation head:
  `3f0578d69926e15a81c4d8fa6105c99c982cbe02`. Three independent reviewers approved
  exact staged diff `3a909f5813def06d4f7cfb27f8650908410ac724` and tree
  `3effac84a84d56f43abcf99c65161c3da7753d6e` with no P0-P3 findings. Both public
  compiler-test jobs, stable and nightly Rust, all three CodeQL analyses, and
  aggregate CodeQL pass. The accepted closure record above completes this evidence.
- Public `CORE-015` tests-only red checkpoint:
  `b203ea429b5a039705be5a5b11998e6dc59f5a24`. Three independent reviewers approved
  its exact staged diff `e158ad61282617a63dade4976a7c23fe53aa0af8` and tree
  `db2ac2959f9815fab5d4b649e563b59c83459dfe` with no P0-P3 findings. Both public
  compiler-test jobs and Rust nightly reproduce exactly the new target's 8 passing /
  8 intended failing split; Rust stable is matrix-cancelled after nightly failure.
  All three CodeQL analyses and aggregate CodeQL pass. This is red evidence, not an
  accepted implementation.
- Public `CORE-015` preregistration checkpoint:
  `4f31f0ca3941389f2cc730136c2540301ee5bfe0`. Three independent reviewers approved
  its exact staged diff `9316f77aed456729624c2d86afaf7110487af84b` and tree
  `bd782da2b5881c1eb50a614400d73b1bb924b033` with no P0-P3 findings. All eight
  public checks pass and the draft PR is cleanly mergeable.
- Prior accepted `CORE-014` closure head:
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
  `8be8c21696cf98602c82e1e5e4fdfc6bf10e9777`.
- Previous public implementation record: `CORE-015` changed only the two
  preregistered production phases, `src/compiler/src/semantic_analyzer.rs` and
  `src/compiler/src/ir_generator.rs`, plus the focused test and these minimal evidence
  records. The focused test adds implementation-review regression controls for
  numeric-array child ordering, single-pass deep nesting, nested index traversal, and
  stub-only method/closure/format/custom-enum boundaries. Several would reject the public red
  implementation, but remain inside its already-failing semantic group and do not
  change the published 8/8 group outcome. It also corrects one green-side assertion
  from the noncanonical `Semantic Error:` fragment to Aero's frozen public
  `Semantic Analysis Error:` phase prefix; that assertion was unreachable while the
  red cases were false accepts. Semantics
  now balances generic scopes on every
  exit, enforces the closed binding selector, and validates numeric-array elements
  and indexes outside generic scopes. Checked admission independently enforces the
  direct/non-generic selector and binary metadata while preserving generic-impl
  annotation quarantine. The focused target passes 16/16. The exact
  `./tools/test.sh` gate passes formatting, correctness Clippy, 139 library tests,
  148 binary tests, every active integration target, and doc tests; 38 pre-existing
  Phase 5 tests remain ignored. Three exact implementation reviews approved diff
  `3a909f5813def06d4f7cfb27f8650908410ac724` / tree
  `3effac84a84d56f43abcf99c65161c3da7753d6e`; public commit `3f0578d` passes the
  complete CI matrix. Three fresh closure reviewers approved exact record diff
  `a8e4059e71991c9d7a274234f91dd225bea61c01` / tree
  `19fea4153397958656b57adac6b70556d4a997c9`; public closure commit `5d7aae0` also
  passes all eight checks. `CORE-015` is accepted at that closure head; its four-record
  final-state sync is public and green at `c612f3b`.
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
- Public version: compiler CLI/banner presentation is manifest-derived package
  `0.3.0`; language `v1.0.0` material is a design target, not current conformance,
  stability, compatibility, or release evidence (`CORE-016`, `ea036f2`).
- Library compiler options: accepted `CORE-020` preserves defaults and rejects
  nondefaults before lexing; option meanings remain unimplemented.
- Compiler architecture: binary and library declare overlapping modules.

## Known blockers and regressions

- No known baseline regression.
- The local shell required Rust installation before tests could run.
- Real backend verification may be blocked by absent LLVM/GPU toolchains or
  hardware; absence will be recorded rather than simulated.
- Accepted `a4327be` removes the false CPU delegated-nonzero success line while
  preserving exact child status/output/cleanup. `run_aero_program` still calls `exit`
  internally after cleanup, and that separate helper/API architecture boundary
  remains open.
- Accepted `CORE-022` at `2a42324` uses final-entry, non-following preflight before
  any `aero init` create/write and prevents the reproduced dangling-source partial
  manifest. General rollback, atomicity, ancestor-symlink policy, and race freedom
  remain open.
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
- Accepted `CORE-015` at `5d7aae0` closes its four selected active false successes;
  the final records are public and green at `c612f3b`. `AUDIT-022` at that clean head
  reproduces R-008: package `0.3.0` versus CLI/banner `1.0.0`, deterministic reruns
  presented as mechanized/formal proof, and current-facing README, CLAUDE, tutorial,
  generic, and ownership safety claims beyond implementation evidence. Residual
  R-002 custom/contextual annotation work is stopped on unresolved nominal/generic
  meaning; remaining R-011 needs typed aggregate execution work. The ignored Phase 5
  backlog remains open:
  an explicit 38-test ignored run passes 36 and fails 2 but contains recovery/stub
  assumptions that prevent bulk activation.

## Exact next action

Obtain three exact approvals, publish this full-local-gate-green six-record
`AUDIT-029` preregistration unchanged, and require all eight public checks before the
read-only audit. Do not begin tests or implementation; repeat accepted slices; invent
semantics; expand init preflight claims; alter workflows/dependencies; publish
benchmarks/packages/releases; modify immutable claim evidence; or touch `master`.

## Unauthorized actions

Do not publish releases/packages/registry changes/benchmark claims; force-push or
rewrite history; delete substantial evidence; change the language's fundamental
identity; make incompatible syntax/semantics changes without a migration plan;
modify downstream repositories; spend money; use unbounded paid compute; handle
credentials; or perform destructive system operations.
