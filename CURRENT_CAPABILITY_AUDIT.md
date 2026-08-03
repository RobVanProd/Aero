# Aero Current Capability Audit

Audit commit: `8f8c7337a4008082fd2a443fcc814b5847b8663f`

Audit date: 2026-08-02

Branch: `agent/aero-integration`

## Verified progress after the audit commit

- At integration commit `6ce859220634c40c696397a3df178faea51f1912`,
  malformed root and applicable direct-module syntax is fatal in the library and
  build/check/run/test/profile paths. Located negative tests cover status and
  no-artifact behavior, and the full repository gate passes.
- That commit closed the audited parser-to-codegen false-success reproducer only;
  at that point the trusted lexer was still infallible. Typing/IR fallbacks remain,
  and no backend or public performance claim was upgraded.
- At `b9883181414886b6b2775b149599da29faed933e`, trusted repository
  compilation/validation paths also use strict fallible lexing. Unexpected
  characters, invalid/non-finite numbers, and unterminated strings now reject root
  and applicable direct-module inputs before semantics or output. LSP lexical
  diagnostics use UTF-16 positions; recovery lexing remains compatibility/editor
  infrastructure and is not a stability claim.
- `CORE-011` tests-only red evidence is public at `9c31820`; three independent
  reviewers approved exact diff `badb9d0e8d6059927d949994b39f617fe2f404a8`
  and tree `540a187db87aff5ec0b2964b0c140c6caf9402a4`. The accepted implementation
  centralizes all inventoried file-backed routes on one crate-private
  direct-source collector, rejects source-only and nested declarations, and makes
  module discovery precede a deterministic module-aware cache key. Both focused
  seven-test suites and the complete repository gate pass. Three independent
  reviewers approved exact implementation diff
  `60fe607413ebc03e9aa5d6296d9067d8cc95d89d` and tree
  `7c57c082e9d5f68afd5c6a4769d9d531a0116642`; every public check passes at
  accepted head `a711dd5f3802095a4ecbe2dea3d45003675e7459`.
- `CORE-012` is accepted at `6780a23cd8b63df124477c7db1190d61dd25f3b8`:
  every live registry route now fails closed before credentials, I/O, transport, or
  writes, while local search and network-free dry-run plans remain available. The
  exact full gate and all public checks pass; the documentation closure is public at
  `b7bb42958e78fb97ea0d991fa3f4cdb40bbcce2f`.

## Executive finding

Aero is an experimental compiler repository with a passing Rust regression
suite and several useful implemented slices, but it is not currently a
trustworthy 1.0 language implementation. The advertised surface substantially
exceeds the enforced source semantics and end-to-end execution evidence. Most
seriously, invalid source can be changed by lexing, function and type contracts
are ignored, unsupported expressions are assigned convenient scalar types or
lowered to zero, and the compiler lacks a genuinely typed, fallible IR boundary.

This conclusion does not discard the existing work. It classifies the current
implementation so that useful components can be completed behind explicit
correctness gates.

## Baseline and environment

- Upstream `master` was cloned at
  `8f8c7337a4008082fd2a443fcc814b5847b8663f` and the integration branch was
  created before changes.
- Rust was initially absent. After installing stable Rust `1.97.1`, Cargo
  `1.97.1`, rustfmt, and Clippy, `./tools/test.sh` passed.
- Executed tests: 106 library tests, 111 binary tests, and 59 active frontend
  integration tests; 38 `phase5_tests` are ignored. Doc tests also passed.
- The current conformance command reports three cases and four repeatability
  checks. It does not prove the formal semantics.
- `AUDIT-020` reproduced that the former documented root-level
  `cargo build --release` command exited 101 because the repository root has no
  `Cargo.toml`; accepted public `CORE-014` at `c56b1d5` instead selects the compiler
  manifest under `src/compiler/` explicitly and passes the exact generated-project
  path in stable Linux CI.

## Specification and public-surface audit

- Versioning is contradictory: the package is `0.3.0`, while README, formal spec,
  and CLI print `1.0.0`; no language/package version distinction is documented.
- Formal and split grammar documents disagree about keywords, formatted strings,
  struct-field delimiters, rebinding, and lifetime maturity.
- The public `CORE-014` red checkpoint proves the former README flagship used
  grouped imports, named arguments, and absent `aeronum`/`aeronn` packages that the
  active repository cannot compile. Accepted public `c56b1d5` replaces it with the
  exact existing generated CPU project. All eight checks pass; stable Linux CI
  resolves LLVM 22 tooling and executes build/init/check/run with status zero and
  exactly one anchored `Output: Hello, Aero!` line. Windows commands are statically
  contract-tested but are not claimed as Windows end-to-end execution evidence.
- Several shipped examples use legacy top-level statements outside the normative
  module grammar. `examples/scoping.aero` also conflicts with the documented and
  active same-scope rebinding rule.
- CLI help describes CPU/ROCm/CUDA `run`, but ROCm stops at object generation and
  CUDA run is explicitly unimplemented. CLI `test` analyzes test files but does
  not execute them.
- Root collection demo/test files and completion documents are not proof of
  end-to-end collection safety, UTF-8 behavior, or performance.

## Frontend audit

- The lexer cannot return lexical errors. It discards unexpected characters,
  maps malformed or overflowing numerics to zero, and emits tokens for
  unterminated strings. Defined lexical diagnostic variants are unused.
- Tokens carry only a starting line/column and AST nodes carry no source spans.
  LSP diagnostics synthesize one-character ranges; f-string subparsing loses the
  original source position.
- Parsing loses meaningful accepted information: most visibility, several
  generic bounds/arguments, f-string identity, and closure block statements.
  Untyped closure parameters are assigned `i32` in the parser.
- Panic recovery can consume the next valid statement and returns no partial AST.
  The public parser API can panic if its token vector lacks EOF.
- The documented grammar and parser differ in both directions. Booleans/chars,
  many operators, expression control flow, module bodies, richer types and
  patterns are representative documented-but-unparsed forms.
- `aero fmt` is line trimming and newline normalization, not syntax-aware
  formatting, and it overwrites the source directly.
- Many lexer/parser test source files are dormant and stale; the active suite has
  no invalid-lexing, recovery-retention, span, grammar-coverage, or formatter
  preservation coverage.

## Type, ownership, and phase-boundary audit

- The initial audit found that active call inference ignored the callee name,
  function environment, arity, parameter types, and return type, then returned
  `Int`. At `8d5d8e7`, monomorphic numeric/void top-level functions instead use
  collected declarations, exact call checks, and matching numeric/void IR results.
  Noneligible signatures remain permissive and uncertified.
- A dormant call validator confirms the mandate's suspected fallbacks: unknown
  named, array, tuple, reference, and generic parameter types map to `Int`.
- Initialized exact numeric `let` annotations are enforced at `bc9a148`.
  `AUDIT-021` at `1535ce2` confirms every other initialized binding annotation is
  discarded by semantics and checked IR: String/bool/custom-name/array-element/
  array-count mismatches pass check/build and publish LLVM for the inferred value.
  Uninitialized reads already fail in semantics. Numeric/void function return
  declarations are enforced at `8d5d8e7`; generic/composite function contracts and
  reassignment remain open. Unknown named types are assumed to be structs, and
  unknown backend type strings lower as LLVM `double`.
- Field access, tuples, struct/enum construction, matches, closures, and unknown
  methods can bypass subtree validation and acquire `Int`. IR lowering replaces
  several of these forms, including borrow/deref, with integer zero.
- Array semantics checks only the first element and discards index type. Checked IR
  now rejects non-int indexes and the internal verifier rejects mixed logical stores,
  so reproduced cases fail without artifacts but after semantic success. Backend
  lowering still uses `double` array storage; bounds/layout remain uncertified.
- Move tracking is shallow and skipped in initializers, returns, block tails, and
  nested calls. Borrow state has no lifetime/provenance analysis or scope-based
  release, and mutable references are classified `Copy`.
- Generic calls are not instantiated. Trait checks compare method names rather
  than signatures and are skipped at many call positions. Dormant generic and
  pattern modules use additional fallbacks and are not compiled into the active
  library or binary.
- At `bc9a148`, tested block, branch, loop, and function exits restore both the
  semantic compatibility table and IR binding map, including scalar and callable
  shadowing. Broader analyzer persistence and phase-boundary risks remain open.
- Semantic analysis returns the original AST rather than a typed representation;
  IR generation is infallible and backend-local code invents missing types. No
  LLVM verifier is part of the active library compile pipeline.

## IR and code-generation audit

- Legacy `parse()` prints an error and returns an empty AST. Semantics accepts it,
  IR creates an empty `main`, and codegen can emit an unterminated LLVM function.
  The `build` command writes this text without verification.
- IR registers, loads, stores, and calls are not typed. Scalar slots are emitted
  as `double` while comparisons produce `i1`, so stored/loaded/returned boolean
  programs can generate type-invalid LLVM.
- `CORE-003` makes checked function `if` arms and reachable void epilogues
  terminator-aware. General CFG lowering can still append statements after a
  terminator and other loop/break/continue or unreachable shapes remain
  uncertified; multiple terminators or invalid blocks may still result.
- At `302211e`, integer, float, mixed, zero-RHS, unary, nested, nonnumeric, root,
  and direct-module modulo nodes whose operands type successfully are rejected by
  shared semantics with one stable diagnostic before IR. Parsing and precedence
  remain explicit; no remainder execution semantics are claimed.
- Adjacent read-only probes found separate boundaries: constant integer `/0` panics
  during IR folding; string comparison passes semantics then panics in IR; tuple,
  field, match, and unknown-method expressions can skip subtree validation and
  compile as fabricated scalar zero. They are not bundled into `CORE-005`.
- `AUDIT-011` isolated the tuple family for `CORE-006`: both tuple literals and
  tuple projections are assigned invented `int` semantics and become zero in both
  IR expression paths. The valid specified expression `(7, 9).0` passes trusted
  checking/building but stores zero. Nested tuple nodes can also evade validation
  beneath parent forms whose inference inspects only part or none of the subtree.
  At `1fa67a2`, tuple syntax/types/patterns are retained while tuple value
  expressions fail closed recursively with one stable diagnostic; fields, matches,
  methods, closures, and other composites remain independent unimplemented
  boundaries.
  Exact clean candidate `cbbe049` is accepted after the complete repository gate
  and two independent reviews; constructed AST callers that bypass semantics and
  tuple layout/execution remain explicitly outside this boundary.
- `AUDIT-012` compared the next three phase failures. Every named FieldAccess form
  tested—including a valid struct-literal projection and undeclared/call receivers—
  passes trusted compilation and becomes zero with no field GEP or receiver call.
  All six string comparison operators pass same-type semantic validation then panic
  in IR. Immediate/computed integer zero division panics in host constant folding,
  while variable, unary, float, and mixed zero forms follow different paths. The
  field family is selected for `CORE-007`; string comparison and division policy
  remain separate, explicitly open tasks.
- At exact reviewed `4e10d479`, trusted active semantic preflight recursively rejects
  named field value expressions after first preserving any established receiver
  diagnostic. The complete gate and two independent non-owner reviews pass. Named
  field syntax remains parsed; struct layout, projection execution, assignment,
  method behavior, and direct AST-to-IR callers remain outside this accepted boundary.
- `AUDIT-013` compared the next open failure mechanisms at exact clean `9fc7d0e`.
  String comparisons are blocked on trustworthy operand typing and equality/order
  policy; zero division is blocked on integer/runtime/IEEE policy; MethodCall is
  blocked on a pre-IR capability discriminator that preserves real array `.iter()`.
  Match is one parser-preserved AST family with no active value-preserving route:
  inference invents `Int`, both IR paths return zero without children, and a
  23-case matrix confirms root, nested, module, and closure false successes. Match
  is selected for fail-closed preregistration; no execution semantics are inferred.
- At `c826294`, active semantic preflight visits a reached Match scrutinee and arm
  bodies in the frozen order, then rejects the Match with one stable diagnostic
  before IR. Exact documented `08e7c2c` passed the full gate but was rejected in
  independent review: parser-retained default trait method bodies are not analyzed,
  so Match there succeeds through the public API/check/build and build writes an
  artifact. A structural audit found no second parsed expression-bearing container
  escape. Match tokens, parser AST, arms, and patterns remain available; pattern
  binding/typing/exhaustiveness, evaluation, result unification, enum layout,
  ownership, IR, and backend execution remain outside this rejected candidate.
  Direct semantic bypass can also still reach dormant zero stubs.
- At `a12f38e`, a dedicated syntax-only block/statement traversal also funnels every
  expression root in parser-retained default trait method bodies into the existing
  preflight. It preserves statement/child order and required signatures without
  activating name, parameter, return, type, trait, ownership, or pattern analysis.
  The corrective red suite captured public and CLI artifact false successes first;
  all 15 Match tests and 81 prior focused boundary tests now pass independently for
  owner and lead. Exact clean documented `b74d91a` passes the complete gate and two
  fresh non-owner reviews: one exhaustive structural review and 44-route public
  matrix, plus an independent 75-route/225-outcome public/check/build matrix. No
  trusted false success, panic, unwind, or negative artifact remains in this accepted
  boundary.
- Unimplemented methods, aggregates, references, and ADTs are either changed to
  zero or dropped. Match retains dormant inference/IR stubs, but corrected trusted
  parsed-source paths now reject it first. Several IR instruction variants have no
  codegen arm and
  are silently ignored by the wildcard arm.
- `AUDIT-014` compared the next open families at exact clean `a61172a`. StructLiteral
  has no active value-preserving source path, receives an invented named type without
  declaration/field validation, and becomes scalar zero in both IR paths without
  children; 19/24 public routes falsely succeeded and root/module builds wrote zero/
  drop artifacts. EnumVariant is also non-executable but intersects Option/Result
  sugar and diagnostics. Borrow mutates shallow ownership state, Deref retains
  reference diagnostics, MethodCall must preserve typed zero-argument Array/Vec
  `.iter()`, string comparison needs operand/operator policy, and division needs
  integer/runtime/IEEE policy. StructLiteral alone is selected for `CORE-009`.
- At exact integrated candidate `a887931`, trusted parsed-source preflight visits
  StructLiteral field values in source order and then rejects construction with
  `Struct construction expressions are not supported.` The 164-test focused matrix
  passes independently for owner and lead. Parser/declaration visibility remains;
  struct name/field/type validation, layout, initialization, ownership, ABI, IR,
  backend emission, and execution remain absent. The complete documented gate passes
  at exact `3410f1f`; coordinated control corrections, a fresh complete gate, and
  two independent approvals with no P0-P3 findings pass at exact reviewed `daa024d`.
- `AUDIT-016` re-ranked the open compiler-integrity families after StructLiteral.
  String comparison and constant integer `1 / 0` can unwind in IR generation;
  comparison bindings can successfully emit `i1` results into `double` slots;
  untyped function/signature reconstruction and the codegen wildcard can emit or
  silently omit invalid instructions. Library, CLI build/run, profiler, and
  conformance use duplicated infallible IR APIs. `CORE-010` is preregistered to add
  checked logical scalar admission, internal IR verification, and final LLVM module
  verification. Its generic boundary rejects unadmitted MethodCall/EnumVariant/
  Deref fallbacks without implementing their language semantics; those remain later
  language-specific implementation slices.
  This audit finding does not itself improve a capability.
- The accepted `CORE-010` production implementation adds logical scalar/place/
  function metadata, mandatory internal IR verification, fallible checked IR and
  codegen APIs, and LLVM 22 verification of final transformed/retargeted modules
  before trusted artifact publication. It makes the focused red contracts and the
  complete repository gate green. Three exact-diff reviews and all required public
  CI checks pass at head `db349ef`; this promotes only the selected checked scalar
  IR/publication boundary and does not promote unresolved language/backend rows.
- At the original audit basis, library/build paths did not invoke an LLVM verifier.
  Current CI object/link/runtime coverage remains limited to four pre-existing
  scalar exit-code examples plus the generated-project status/output path accepted
  by `CORE-014`.

## Runtime and backend audit

- CPU has a real LLVM-to-object-to-link-to-process path when `llc`/`clang` are
  available. Linux CI covers four small scalar programs by exit code plus the
  generated `Hello, Aero!` project by status and anchored output. The current
  Windows audit host lacks those tools, so local execution is environment-blocked.
- ROCm retargets LLVM and attempts an AMDGPU object; it has no link or HIP launch
  path. CUDA run/object/link/launch is absent. GPU auto-detection probes tools or
  environment rather than a verified usable device and may silently select CPU.
- Graph "executable kernels" are ordinary internal scalar-double LLVM helpers;
  backend selection affects names/metadata, not a device ABI, transfers, launch,
  or synchronization.
- Quantization likewise emits backend-named scalar-double helpers. FP8 does not
  perform FP8 representation/rounding, `per_channel` is metadata-only, and INT8
  multiplication/division scaling is algebraically incorrect. Tests assert text
  and counters rather than numerical reference equivalence.
- The tracked llama.cpp ROCm GGUF run is correctly external reference evidence,
  not Aero device execution.

## Benchmark validity finding

The README's tracked 19-case "compilation" series is invalid as a compilation
measurement. `benchmarks/performance_benchmark.py` invokes
`cargo run --release -- <sourcefile>`, but the CLI requires a `build`, `run`, or
other command. At the `AUDIT-019` reproduction basis it printed `Unknown command`
and exited zero, which the harness counted as a successful compilation. The
Accepted public `CORE-013` at `a78dd00` makes that bare-source invocation fail closed with status
`2`; it does not retroactively validate or repair the timings. Those numbers measure
Cargo startup/unknown-command handling and must not support Aero compilation-speed
claims.

All six public entries in `claim-verification/claims.json` reference existing
files, but none meets the full benchmark protocol. The two compilation claims share
the invalid command and are classified `invalid_measurement`. The current and split
historical lexer Criterion records retain their separate qualifications; retained
output does not justify strengthening their statistics and lacks raw
samples/hashes/correctness checks. The GGUF entry is a genuine one-run external
llama.cpp observation with zero warmups, truncated output, no correctness gate,
incomplete hashes, and inconsistent top-level versus artifact commit attribution.
The blocked/omitted GPU-claim record is accurate.

The split historical lexer Criterion run is genuine historical microbenchmark
evidence; that preservation does not rehabilitate either Python compilation series.

No current public Aero runtime, device, graph, or quantization performance claim
passes `BENCHMARK_PROTOCOL.md`. Existing evidence remains preserved; accepted
`CORE-013` classifies the two invalid Python compilation series without deleting or
upgrading any artifact.

## Tooling and API audit

- Before `CORE-013`, `build`, `check`, `graph-opt`, `test`, unknown commands, missing
  inputs, failed output writes, and failing conformance paths commonly returned
  process status zero. Accepted `CORE-013` now gives automation a typed `0/1/2`
  correctness signal for CLI-owned outcomes; delegated CPU program statuses remain contextual
  arbitrary pass-through values.
- The library and binary compile their own module instances. The library ignores
  every `CompilerOptions` field; the CLI never calls `compile_program`, and its
  advertised optimizer objects are often constructed but unused.
- `check` omits module resolution and uses the legacy parser; `test` only
  discovers and analyzes source instead of compiling/executing it; `profile`
  times a divergent subset; LSP diagnostics are syntax-only and symbol features
  use token heuristics rather than semantic scopes/types/modules.
- The module resolver loads one level and has no recursive graph/cycle handling.
  Project initialization creates a manifest that compiler commands do not use.
- Live registry publish sends path/size/hash metadata without package contents
  and accepts any successful HTTP status without a response contract. Install
  joins registry-provided package name/version into the destination without
  containment validation, creating a path-escape risk. Live registry operations
  remain unauthorized during this audit.

### Post-CORE-010 module-boundary recheck (`AUDIT-017`)

- At the `AUDIT-017` / accepted-`CORE-010` basis, `check` and `profile` strictly
  loaded direct modules, while build caught a missing-module error, continued
  compilation, exited zero, and wrote the requested LLVM artifact for the same
  source.
- At that basis, final-LLVM cache lookup preceded lexing/parsing/module discovery
  and used only root source plus target/GPU configuration. Module changes or
  deletion did not participate in cache identity, so a verified cache hit could
  bypass current module state on the reusable optimizer path.
- Before `CORE-011`, build, check/test, profile, and docs repeated direct module
  load/strict lex/parse logic. The source-only public library API lacked a file
  context and silently accepted `ModDecl`. The accepted closure below centralizes
  this direct source set and fails closed; it does not certify the still-absent
  namespace, `use`, `pub`, recursive graph, or cycle-analysis implementation.
- No-argument, unknown-command, and several missing-input paths still return status
  zero, but they remain a separate command-result task because the selected module
  boundary is specifically about dependency admission, cache identity, and compiler
  artifact publication.
- Accepted closure: `CORE-011` now routes build/run, check, discovered test,
  profile, and docs through the shared collector; source-only `compile_program`
  rejects `mod`. Missing, malformed, and nested module sources stop before later
  phases or publication. Exact source bytes and stable relative candidates enter
  the V1 module-bearing cache identity before lookup, while the no-module key is
  unchanged. This remains direct flattened source collection only; namespaces,
  `use`, `pub`, recursive graphs, cycles, and general CLI status are still absent.

### Post-CORE-011 risk re-ranking (`AUDIT-018`)

- Audit basis: clean published project-control head
  `8598a4c343f5592880bde66cbd99e78083d2a236`; accepted compiler behavior remains
  `a711dd5f3802095a4ecbe2dea3d45003675e7459`. Upstream `master` remains the original
  `8f8c7337a4008082fd2a443fcc814b5847b8663f`.
- Registry: live search, publish, and install remain directly callable. Publish
  inventories only path/length/SHA-256 metadata, sends no package bytes, discards the
  response body, and labels any successful HTTP result accepted. Install joins
  registry-controlled resolved name/version into the selected destination and writes
  it without component validation or containment. CLI auth lookup currently occurs
  before publish/install dry-run branching and for offline search.
- Command/benchmark: unknown commands, malformed `build`/`check` usage, missing
  build/check inputs, and the benchmark's bare source-path invocation all reproduced
  status zero. The bare source path prints `Unknown command`, proving the tracked
  compilation timing measures successful non-compilation. This remains R-013/R-015
  and is ranked immediately after the registry quarantine.
- Arrays: `[1, 2.5]` passes semantics and checked admission, then is rejected by the
  mandatory in-process verifier for an Int/Float store mismatch; `[1, 2][1.5]` is
  rejected by checked admission. Both `check` and `build` return nonzero and publish
  no requested LLVM artifact. R-011 therefore remains a phase-order/type-contract
  defect, but no fresh false-success artifact outranks the active registry boundary.
- Ownership: public safety claims still exceed shallow move tracking, lifetime
  provenance is absent, and mutable references are classified as Copy. A credible
  closure requires a frozen ownership model plus CFG/provenance work across more
  than the permitted bounded phases; changing one Copy predicate would imply safety
  it does not provide. R-004 remains critical and explicitly stopped, not waived.
- Selection: `CORE-012` was selected to quarantine every HTTP-backed registry entry with one
  stable fail-closed guard before credential, filesystem, or transport activity.
  Offline index search and non-network publish/install previews remain available.
  This does not design or validate a registry protocol and cannot re-enable live
  behavior.
- Public red evidence: `57c4ec7` contains only test-scoped direct controls plus the
  focused CLI matrix. Three independent reviewers approved exact diff
  `4058775145e68aa9a5512853c04b0dde04730464` and tree
  `227254ef8177d8e15b69c42bd1e2d94c1442879a`. Direct evidence was 7 pass / 5
  intentional failures; CLI evidence was 0 pass / 6 intentional failures.
- Accepted implementation: every live function and CLI live branch now hits
  the frozen guard before credentials or side effects. Local search and CLI dry-runs
  bypass auth and use only local helpers. Direct registry targets pass 12/12 each;
  the CLI matrix/help pass 7/7; the complete repository gate is green. Three
  independent reviewers approved exact diff
  `05e55496f6664713192b2dbf94eca785abe2931d` and tree
  `85ed76ab0141409796e167704e4100dd4d15c26f` with no P0-P3 findings. Both compiler-
  test workflows, Rust stable/nightly, every CodeQL language analysis, and aggregate
  CodeQL pass at accepted public head
  `6780a23cd8b63df124477c7db1190d61dd25f3b8`.

### Post-CORE-012 CLI and benchmark revalidation (`AUDIT-019`)

- Historical audit basis: clean public documentation head
  `b7bb42958e78fb97ea0d991fa3f4cdb40bbcce2f`; accepted production behavior at that
  point was exact `6780a23cd8b63df124477c7db1190d61dd25f3b8`. Accepted public
  `CORE-013` at `a78dd004aa37c39212711027b777698118d9dc02` supersedes the
  status/claim findings below.
- An initial argument-dropping process batch was invalid and discarded. The corrected
  pre-implementation explicit-argument probe showed status zero for no command,
  unknown command, bare
  benchmark source path, malformed and missing-input build/run/check/fmt/doc/profile/
  graph-opt/quantize routes, registry no/unknown subcommand, and malformed
  conformance. Static inspection found the same fallthrough for failed output writes
  and ignored extra operands in check/fmt/test/lsp.
- `performance_benchmark.py` sends the bare source path and would have accepted the
  former zero status; accepted `CORE-013` now returns `2`. The shell harness
  declares its compile/run work simulated. No benchmark was run. The tracked Python
  compilation numbers remain invalid measurements and their raw evidence is
  preserved under that classification.
- R-004 remains stopped on unfrozen multi-phase ownership semantics; reproduced
  R-011 arrays fail closed before output. Accepted `CORE-013` now
  provides a typed CLI-owned `0` success / `1` operational failure / `2` invocation
  failure contract plus evidence-preserving quarantine of the affected compilation
  claims.
  Delegated CPU-program exits remain arbitrary pass-through values; write rollback
  remains non-atomic and out of scope. Benchmark code, command maturity, and compiler
  phases remain frozen.

## Testing and fuzzing audit

- At the original audit basis the default run executed 106 library, 111 binary,
  and 59 frontend tests, with 78 overlapping library/binary names. At clean public
  head `1535ce2`, Cargo lists 139 library and 148 binary unit tests with 105 shared
  names. `cargo test --tests -- --list` reports 557 target entries and 437 distinct
  displayed names; neither count proves independent behavior coverage.
- All 38 `phase5_tests` are ignored. They cover ownership/moves, borrowing,
  generics, traits, and combined integration—the areas with the strongest public
  safety/abstraction claims.
- Another 299 source tests are dormant because their files/modules are not linked
  into Cargo targets; several reference removed APIs and cannot be treated as a
  ready-to-enable suite.
- Eight active snapshots exist, but at least one blesses a binary expression
  whose semantic output still has `ty: None`.
- Criterion benches are not correctness gates and some discard compiler errors.
  CLI `aero test` discovers no repository Aero test programs and succeeds with a
  warning. `error_examples.aero` is not connected to a compile-fail harness.
- There is no active arbitrary-input fuzzing, property testing, differential
  execution, compile-fail artifact check, LLVM verifier suite, Windows CI, or
  ROCm/CUDA hardware execution job.

### Post-CORE-014 binding-contract recheck (`AUDIT-021`)

- Five initialized annotation mismatches—String from int, bool from int, an unknown
  named type from int, fixed float-array value under an int-array annotation, and a
  two-element value under a three-element annotation—each pass CLI check/build and
  create the requested LLVM artifact. Exact String, comparison-produced bool, and
  homogeneous integer-array controls pass.
- Mixed numeric arrays and float indexes return nonzero and create no artifact, but
  build traces record semantic success before an internal-verifier or checked-IR
  error. Direct source inspection confirms semantics infers only the first array
  element and ignores the index result; checked IR ignores binding annotations.
- Direct source inspection also finds two checked-boundary parity defects relevant
  to exact equality: checked IR trusts optional caller-supplied `Binary.ty` ahead of
  operand inference, and it maps lowercase `string` to `Ty::String` while active
  semantics maps that spelling to the distinct named `Ty::Struct("string")`.
- Uninitialized reads fail in semantics. Borrow/deref sources fail in checked IR,
  while mutable references remain classified Copy and lifetime provenance remains
  absent. This keeps R-004 stopped rather than pretending a one-predicate edit would
  establish ownership safety.
- `CORE-015` preserves existing numeric scalar enforcement and, outside active
  semantic generic scopes, adds a closed binding-local rule for `bool`, canonical
  `String`, and nonempty one-dimensional fixed arrays over the four numeric spellings.
  It closes four of five reproduced false successes, adds all-element numeric-array
  inference/count/integer indexes in the same scope, and verifies binary type
  metadata. Lowercase `string`, custom/contextual/structural annotations, nonnumeric
  arrays, and all new generic-scope annotation/array behavior must retain pre-task
  outcomes under required green controls. No global recursive mapping, conversion, new type,
  assignment, ownership, aggregate lowering, or backend claim is selected.
- The approved tests-only red target proves that frozen boundary with 8 passing
  preservation groups and exactly 8 intended failing contract groups. Three
  independent reviewers approved exact diff
  `e158ad61282617a63dade4976a7c23fe53aa0af8` and tree
  `db2ac2959f9815fab5d4b649e563b59c83459dfe`; it is public at `b203ea4`. Both
  compiler-test jobs and Rust nightly reproduce the same intended target failure,
  stable is matrix-cancelled, and all CodeQL checks pass.
- The public two-file production candidate now makes the focused matrix pass 16/16.
  Its test delta also adds implementation-review regression controls for numeric-array
  child ordering, single-pass deep nesting, nested index traversal, stub-only
  method/closure/format/custom-enum boundaries, and unsupported-child precedence. Several controls would reject the
  public red implementation, but they live inside its already-failing semantic group
  and therefore do not change the published 8/8 group outcome. One green-side
  public-library assertion was also corrected to the established
  `Semantic Analysis Error:` prefix. The exact repository gate passes formatting,
  correctness Clippy, 139 library tests, 148 binary tests, all active integration
  targets, and doc tests; the 38 established Phase 5 ignores remain unchanged. Three
  independent reviewers approved exact implementation diff
  `3a909f5813def06d4f7cfb27f8650908410ac724` and tree
  `3effac84a84d56f43abcf99c65161c3da7753d6e` with no P0-P3 findings. Public commit
  `3f0578d69926e15a81c4d8fa6105c99c982cbe02` passes both compiler-test jobs,
  stable/nightly Rust, all three CodeQL analyses, and aggregate CodeQL. Three fresh
  reviewers then approved exact closure-record diff
  `a8e4059e71991c9d7a274234f91dd225bea61c01` and tree
  `19fea4153397958656b57adac6b70556d4a997c9`; public closure commit
  `5d7aae0f5626813249b6de983a229dbbb1e4fef8` also passes all eight checks.
  `CORE-015` is accepted at that public closure head without expanding the explicit
  exclusions or any backend capability claim.

### Post-CORE-015 public-claims recheck (`AUDIT-022`)

- The accepted final-state sync is public at clean head `c612f3b`; draft PR #4 is
  mergeable and all eight checks pass. Upstream `master` remains `8f8c733`.
- Cargo metadata reports package `compiler 0.3.0`, while standalone `-v` and
  `--version` print `Aero compiler version 1.0.0` and the no-command help banner
  prints `Aero Programming Language Compiler v1.0.0`. The unadvertised bare
  `version` word is already an unknown command with status two.
- `aero conformance` reports three example cases and four repeatability checks. The
  four named checks rerun tokenization, parsing, checked IR, and lowering to compare
  deterministic output; they are regression evidence, not mechanized formal
  semantics. The current console, help, BUILD, README, and consolidated language
  document overstate that evidence.
- README still lists generics/trait bounds/where clauses and a borrow checker as the
  current language surface. The type-system and ownership documents plus Tutorial 3
  state compile-time memory-safety enforcement that the audit disproves: lifetimes
  and provenance are absent and mutable references remain classified Copy.
- `CLAUDE.md` labels Phase 5, borrow-checker enforcement, generics, and traits
  complete. Tutorial 1 calls the four repetitions mechanized checks and the command
  a formal suite; its next-step list also presents ownership as an active memory-
  safety feature. Tutorial 2 calls the following ownership material an active
  memory-safety feature rather than design-only. These current-facing surfaces are
  part of R-008; Tutorial 4 already carries an explicit implementation boundary.
- Claim-heavy collection/string demo and task summaries have no visible historical
  status notice. Struct and enum records already carry such notices. `todo.md` still
  presents completed phases and version `1.0.0` as current project status.
- An explicit run of all 38 ignored `phase5_tests` passes 36 and fails 2. One failure
  relies on recovery parsing dropping unsupported assignment before semantics; the
  other expects accepted struct/method/borrow behavior outside frozen support. The
  passing set therefore cannot be activated blindly as current capability evidence.
- R-004 remains stopped on multi-phase ownership/provenance decisions. Residual R-002
  custom/contextual annotations require a separately frozen nominal-name/generic-
  substitution contract. Remaining R-011 aggregate execution requires typed IR,
  bounds, layout, and backend work; R-012 needs test-by-test recovery/stub
  classification. R-006, R-009, and R-010 are broader than this bounded workflow;
  R-007 lacks device evidence; R-016 is medium/medium. R-008 is the highest bounded
  active public false claim.
  `CORE-016` selects manifest-derived CLI presentation and visible evidence-based
  documentation classification. No package version, report schema, semantics,
  backend, benchmark, registry, release, or master behavior is selected.
- Reviewed public red commit `4b94dbd` binds this boundary with exactly two
  preservation passes and five intended failures. The subsequent implementation
  reports compiler package `0.3.0` through both version flags and the no-command
  banner, labels the unchanged three cases/four repetitions as deterministic
  regression evidence, and visibly classifies unsupported current/design/historical
  claims. Focused claim and CLI targets each pass 7/7 and exact `./tools/test.sh`
  passes including doc tests. These results became accepted implementation evidence
  at `cc984d0`; by themselves they do not elevate any capability class.
- Exact three-review-approved implementation `cc984d0` is now public and passes both
  compiler-test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate
  CodeQL. The preceding candidate results are therefore implementation evidence, but
  exact three-review-approved record-only closure `ea036f2` is also public with all
  eight checks green. R-008 is controlled for this selected claim boundary. No
  capability class is elevated, and actual ownership/type/backend gaps remain open.

### Post-CORE-016 ignored-test recheck (`AUDIT-023`)

- Clean accepted public head `8869eca` is mergeable with all eight checks green;
  upstream `master` remains `8f8c733`.
- The explicit Phase 5 ignored target reproduces 38 run / 36 pass / 2 fail. Its four
  lexer, twenty parser, and fourteen semantic tests all use recovery lexing; parser
  and semantic tests also use a compatibility parser that can convert failure to an
  empty AST.
- Only 22 tests are conservative activation candidates: four exact strict-token and
  eighteen exact strict parser-retention contracts. Two generic-impl tests remain
  quarantined because target arguments and impl bounds are skipped/discarded. All 14
  semantic tests remain quarantined because broad outcomes mix five genuine shallow
  negatives, positive smoke, unrelated unsupported-construct false positives, and
  two recovery/unsupported failures.
- R-004 remains the highest conceptual risk but stops on unfrozen multi-phase
  lifetime/provenance semantics. R-012 is the highest bounded action and may move only
  to partially controlled under `CORE-017`. No syntax test can elevate a capability
  class or establish ownership, borrow checking, generic/trait enforcement, or
  execution.
- Public-green preregistration `2c61535` binds the conservative 22/16 split. Exact
  three-review-approved implementation `8be8c21` passes exactly 22 strict syntax-
  retention tests with 16 explicit quarantines and 38 total listed entries. Current
  CLAUDE/framework/roadmap lines classify this as parsed-only evidence, and the exact
  full gate passes. Exact three-review-approved record-only closure `3dd3bb4` is also
  public with all eight checks green. R-012 is partially controlled for this selected
  evidence-classification boundary; Cargo overlap and 299 dormant tests remain, and
  no semantic or execution capability class changes.

### Post-CORE-017 backend truth recheck (`AUDIT-024`)

- The audit basis is clean public head `9ddc571`, tree `20ab4e6`, with all eight
  checks green. The root also reran the seven-test CLI status contract and the graph
  and quantization unit filters (3/3 and 5/5 in each duplicated lib/bin target); all
  passed before any edit.
- CPU `run` has a real externally verified host object/link/process path and passes
  the child status through. ROCm `run` requires LLVM verification and invokes `llc`
  for a temporary AMDGPU object, but it neither checks that the object exists nor
  links, launches, synchronizes, or executes anything; it then returns status zero.
  CUDA `run` correctly returns operational status `1` and has no object/link/launch
  path.
- The `gpu` alias is an environment/tool-presence heuristic, not usable-device
  detection. It can silently choose CPU and does not probe CUDA capability.
- Graph compilation externally verifies textual LLVM transformation into ordinary
  internal scalar-`double` helpers. Backend names and existing `executable*` report
  fields do not establish a device ABI or execution. Quantization likewise emits
  scalar-`double` helpers using default or sample-derived scale; it has no FP8
  representation/rounding, executed per-channel behavior, numerical proof, or device
  execution.
- Current README, BUILD, tutorial, CLI help/reporting, quantization notes, and the
  enabled Aero ROCm GGUF example exceed that evidence. All 27 declared immutable
  claim artifacts exist, and `claim-verification/` already classifies the real GGUF
  run as external llama.cpp reference evidence and Aero GPU claims as blocked.
- All three independent read-only reviewers classify the ROCm zero-status path as a
  P1 false success and recommend a tests-first fail-closed correction. `CORE-018`
  freezes operational status `1` for object-only ROCm `run`, a regular-file emission
  postcondition, explicit target selection, and stage-accurate current claims while
  preserving algorithms and compatibility field names. R-007 remains OPEN because
  no Aero accelerator execution or correctness evidence exists.
- The exact tests-only tree `4a65ecf7` was independently approved and published as
  `427fb4c`. Both compiler jobs and stable/nightly failed on the prescribed new
  contracts while all four CodeQL checks passed. The local implementation candidate
  now passes CLI 10/10 and claims 7/7: CPU is unchanged; ROCm checks only regular-file
  emission then returns status 1 without link/launch; CUDA returns status 1; ambiguous
  `gpu` fails before source access; graph/quant output states non-device scalar-helper
  scope; and the nonexistent Aero GGUF route is disabled while external references
  remain exact. Exact implementation tree `d10567be` received three independent
  approvals and was published as `8bde0ff`; the complete local gate and all eight
  public checks pass. Exact record-only diff `3d0a17f7` and tree `83c9676f` then
  received three independent approvals with no P0-P3 findings and were published as
  closure `2e0e17f`. Its two compiler jobs, stable/nightly Rust, all three CodeQL
  analyses, and aggregate CodeQL pass. The selected false-success/current-claim
  boundary is accepted, but R-007 remains open because no accelerator execution or
  correctness evidence was produced.

### Post-CORE-018 clean-head risk re-ranking (`AUDIT-025`)

- The accepted final-state sync is public at clean head `d0bd54e`, tree `21e72079`;
  draft PR #4 is open/mergeable, all eight checks pass, and upstream `master` remains
  `8f8c733`.
- `aero test` discovers direct `*_test.aero`/`*_tests.aero` files and performs strict
  parsing, direct-module collection, and semantic analysis only. It does not admit
  checked IR, generate code, execute a process, or compare runtime results, but its
  comment, progress output, success summary, help, BUILD guide, and an existing CLI
  contract say `run`, `Running`, or `passed`. Accepted DEC-016 already classifies the
  command as a semantic checker rather than an execution runner.
- Two independent auditors rank that direct user-facing false assurance as the next
  bounded P1; the IR/codegen auditor instead ranks ignored public nondefault
  `CompilerOptions` first. The lead selects `aero test` claim containment because it
  is a documented CLI surface with direct user reach and requires presentation/tests
  only. Fail-closed nondefault `CompilerOptions` is the bounded runner-up.
- R-002 and R-004 remain more severe conceptually but stop on unresolved nominal/
  generic and ownership/provenance semantics across multiple phases. R-005 public
  unchecked API retirement needs a major-boundary compatibility policy; R-007 needs
  hardware; R-009/R-010 and broad R-006 convergence are architectural; R-011 needs
  bounds/layout/execution semantics; R-012 remains a per-slice classification backlog;
  and R-016 needs a toolchain policy.
- `CORE-019` therefore freezes wording-only CLI truth. Discovery, parsing, module
  collection, semantic analysis, counts, and statuses remain unchanged; no test
  execution, checked IR, codegen, runtime, language semantic, or capability work is
  selected.

### CORE-019 public red checkpoint and accepted implementation

- Triple-reviewed tests-only commit `6728a39`, tree `5337c877`, reproduces the exact
  public boundary: CI runs `30828281313` and `30828277960` fail only the two frozen
  CLI contracts at 9/2; Rust run `30828281681` has the same 9/2 nightly failure and
  a permitted fail-fast cancellation during stable tests; CodeQL run `30828277876`
  and aggregate check `91735622062` provide all four green CodeQL checks.
- Exact three-review-approved implementation `2fe580d`, tree `1e530e65`, changes
  only `src/compiler/src/main.rs` and `BUILD.md` presentation plus authorized records.
  Focused CLI is 11/11, exact `./tools/test.sh` passes, and all eight public checks
  pass in compiler runs `30829084150`/`30829086467`, Rust `30829088650`, CodeQL
  `30829082758`, and aggregate `91738325685`. The selected claim boundary is
  accepted. No checked IR, codegen, process execution, discovery, diagnostic, count,
  status, language-semantic, or backend capability is added or promoted.
- Exact three-review-approved corrected record-only closure `63b6629`, tree
  `2e886850`, passes exact `./tools/test.sh` and all eight public checks in compiler
  runs `30829963152`/`30829970545`, Rust `30829968789`, CodeQL `30829962982`, and
  aggregate `91741344282`. `CORE-019` is complete at that selected boundary; broader
  R-013 command behavior and executable-test design remain open.

### Post-CORE-019 clean-head audit contract (`AUDIT-026`)

- Accepted public final-state sync `25dec51`, tree `46828e7d`, passes both compiler
  jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate CodeQL. PR #4
  is open/draft/mergeable and upstream `master` remains `8f8c733`.
- The next task is read-only re-ranking. It must reproduce the public
  `CompilerOptions { optimize, debug_info, target }` facade being ignored by
  `compile_program`, compare reach/severity/phase count/compatibility ambiguity with
  remaining risks, and recommend one bounded next action or stop. It does not define
  option semantics, authorize code/tests, or change any capability class.

### Completed `AUDIT-026` findings and `CORE-020` selection

- Public preregistration `2c61ff9`, tree `ff20cf43`, passed compiler runs
  `30831057824`/`30831063857`, Rust `30831066619`, CodeQL `30831055856`, and
  aggregate `91744957183`; all eight checks are green.
- At audited head `25dec51`, `CompilerOptions` publicly exposed `optimize`,
  `debug_info`, and `target`, derived `Debug`, `Clone`, and `Default`, and was
  described as compiler options for benchmarking. `compile_program` accepted the
  value as `_options` but did not read it. Its checked parse-to-codegen path was
  therefore identical for every field value.
- At audited head `25dec51`, the repository had 62 direct calls outside the
  definition: 28 across five benchmarks and 34 across thirteen test files. Every call
  constructed `CompilerOptions::default()`; no in-repository nondefault construction
  existed. No CLI route consumed this public type, and CLI CPU/ROCm/CUDA
  `BuildTarget`/`BuildConfig` orchestration was a separate private surface.
- Static tracing at audited head `25dec51` proved `_options` had no read before or
  within the checked pipeline;
  existing binding (16), checked-IR (6), fatal-parse (11), and module (7) targets
  remained 40/40 green. Two attempted temporary external probes exceeded the audit's
  read-only/external-artifact stop boundary: one reported dynamic results and one was
  interrupted. Both are excluded from accepted findings. Their named executables no
  longer exist; two inert PDB files remain in the user temp directory and are not
  repository or publication artifacts.
- All three independent type/safety, IR/codegen, and backend/claim auditors rank this
  as the best bounded next action under R-006. External nondefault consumers cannot
  be inventoried, so returning an error is a behavior compatibility change, although
  public names, layout, signature, construction, and default behavior remain intact.
- Lead-owned DEC-025 selects `CORE-020`: exactly the default tuple
  `(false, false, String::new())` remains supported. Any true Boolean option or
  nonempty target returns one stable unsupported-options error before lexing. This
  contains false assurance without defining optimization, debug information, target
  selection, CLI mapping, IR, codegen, or backend semantics. No capability row or
  class is promoted by the audit or preregistration.
- Exact three-review-approved closure/preregistration `fae1374`, tree `8c807c17`,
  passes all eight public checks in compiler runs `30833300163`/`30833300408`, Rust
  `30833301841`, CodeQL `30833296979`, and aggregate `91752384364`.
- Exact three-review-approved tests-only `037f44d`, tree `edd8d33e`, reproduces the
  public red boundary: compiler runs `30833844930`/`30833845633` and nightly in Rust
  run `30833845526` fail only the frozen target at 1/1; stable is cancelled during
  tests by permitted matrix fail-fast. CodeQL `30833844647` and aggregate
  `91754222422` provide all four green security checks.
- Before publication, the local implementation candidate added one guard at
  `compile_program` before the
  first lexer call and changes no other compiler phase. The new contract is 2/2,
  binding/checked-IR/fatal-parse/module preservation is 40/40, and exact
  `./tools/test.sh` passes. Public API/default values and byte-exact default LLVM and
  parse diagnostics are preserved. No optimizer, debug, target, CLI, IR, codegen, or
  backend behavior was added; public green acceptance remained pending at that
  checkpoint.
- Exact three-review-approved implementation `70cb0ad`, tree `7c8b2ce1`, diff
  `33e5883e`, passes both compiler runs `30834445685`/`30834446600`, stable/nightly
  Rust run `30834446605`, all three CodeQL analyses in `30834443841`, and aggregate
  `91756251121`. `CORE-020` is accepted at the selected ignored-option boundary.
  Compiler options remain `PARSED_ONLY`: unsupported rejection is enforced, but no
  optimization, debug-information, target, CLI, IR, codegen, or backend semantics are
  implemented. Broad R-006 convergence remains open.
- Exact three-review-approved record-only closure `5a8cd06`, tree `df4a04a`, diff
  `85ef52a4`, passes compiler runs `30835593703`/`30835597576`, stable/nightly Rust
  run `30835597620`, all three analyses in CodeQL run `30835594365`, and aggregate
  `91759990615`. It adds no compiler behavior and makes no capability promotion.

### Next clean-head audit contract (`AUDIT-027`)

- Basis: the exact commit that publishes this six-record contract, but only after its
  local full gate, three independent approvals, and all eight public checks pass. Its
  only delta from accepted closure `5a8cd06` is current control-record state.
- Observed state: R-002 and R-004 retain high raw safety impact but require unresolved
  type/ownership decisions; R-005 needs an unchecked-API policy; R-006 still includes
  duplicated orchestration and undefined option meanings; R-007 needs real hardware;
  R-009/R-010 are architectural; R-011 needs aggregate bounds/layout/execution;
  R-012 is a per-slice ignored-test backlog; R-013 retains delegated-exit, rollback,
  executable-test, command-maturity, and helper-architecture boundaries; and R-016
  needs a toolchain policy.
- Method: use repository source, tracked evidence, existing tests, and public check
  records only. Compare active reproducibility, reach, severity, frozen semantics,
  phase count, compatibility ambiguity, and testability. Three independent auditors
  must rank the same candidates and report evidence plus remaining uncertainty.
- Output: recommend one bounded tests-first task or an explicit stop. The audit may
  update only the six current control records after findings are reconciled; it may
  not define language/toolchain/hardware semantics, create probes or artifacts, edit
  code/tests/workflows/dependencies, or promote any matrix row or capability class.

### Completed `AUDIT-027` findings and `CORE-021` selection

- Exact three-review-approved public basis `aa3e7a8`, tree `4caa5c33`, passes
  compiler runs `30836250279`/`30836251909`, stable/nightly Rust run `30836255407`,
  all three analyses in CodeQL run `30836248101`, and aggregate `91762198170`.
  Auditors made no changes, ran no tests or probes, and left the worktree clean.
- All three auditors rank R-013 first by active reproducibility, reach, semantic
  readiness, phase count, compatibility ambiguity, and tests-first feasibility.
  Remaining raw-critical type, ownership, and unchecked-API work requires unfrozen
  semantics or a major-boundary policy; hardware needs real devices; spans, grammar,
  aggregates, and compiler convergence are architectural; the next dormant-test
  slice and supported-toolchain policy are not yet frozen.
- At the audited basis, the CPU execution branch obtains the delegated status, then
  unconditionally prints `Program executed successfully.` and `Exit code: N`.
  The process contract deterministically supplies exit 7 and currently requires the
  false success line. Cleanup completes before the wrapper propagates the exact child
  status; ROCm/CUDA remain fail closed.
- Reconciliation compared three R-013 slices. Two auditors rank the nonzero false-
  success presentation first; one ranks dangling-entry `init` containment first and
  presentation second. The lead selects presentation because it affects every
  nonzero CPU child, has a cross-platform deterministic regression, changes zero
  compiler phases, and preserves status/cleanup semantics. Entry-aware `init`
  preflight remains the bounded runner-up; hidden helper termination remains open.
- DEC-026 and `CORE-021` freeze one output condition only. This is claim containment,
  not language, execution, backend, safety, or stability capability; no matrix row or
  capability class is promoted.
- The exact three-review-approved tests-only checkpoint `0873f65`, tree `51ec7d0a`,
  diff `f75a6360`, now binds delegated exits `0/1/2/7`, exact status, the complete
  normalized CLI-owned presentation suffix, child output markers, and cleanup after
  all four cases execute. Local and public evidence is exact 10/1 on the false
  nonzero success line: compiler `30839264536` / `30839272375` and nightly in Rust
  `30839272429`; stable is cancelled during its test step by fail-fast. CodeQL
  `30839264268` and aggregate `91772180985` pass. The one-condition implementation
  passes focused CLI 11/11, backend-claim 7/7, and the exact full local gate. Exact
  tree `0ad98c82`, diff `2dbbc395`, received three approvals and was published as
  `a4327be`; compiler `30839860335` / `30839862442`, Rust `30839862423`, CodeQL
  `30839859840`, and aggregate `91774125621` all pass. The selected truthful
  presentation boundary is accepted without promoting any language, execution,
  backend, safety, or stability capability.
- Corrected exact record-only closure `b99e445`, tree `8a4c2d77`, diff `5abbf3a7`,
  is triple-approved and all-eight public green in compiler `30840427466` /
  `30840426655`, Rust `30840428215`, CodeQL `30840415565`, and aggregate
  `91775938704`. `AUDIT-028` is preregistered to compare every remaining OPEN or
  PARTIALLY CONTROLLED risk from a clean public head; it is read-only and cannot add
  or promote a capability.
- Public-green `AUDIT-028` basis `399e04f` completes that full-set comparison. The
  independent top threes are R-011/R-013/R-002, R-013/R-012/R-002, and
  R-002/R-013/R-010. R-013 is the only universal top-two residual. DEC-027 and
  preregistered `CORE-022` select only non-following, fail-closed destination-entry
  preflight before `aero init` writes. R-011 bounds behavior remains unfrozen; R-002
  remains a wider runner-up. This is project-tooling containment and adds no language,
  compiler, backend, filesystem-atomicity, safety, or stability capability.
- `CORE-022` is implemented and accepted at `2a42324`. Triple-reviewed tests-only
  `7cd8aba` produces exact Linux compiler 10/1 in `30843119793` / `30843125522` and
  nightly Rust `30843124314`; stable is fail-fast cancelled during tests, while all
  CodeQL analyses pass. Triple-reviewed implementation `2a42324` passes focused 3/3
  and 11/11 plus the exact local gate, compiler `30843592298` / `30843592784`, Rust
  `30843595560`, CodeQL `30843589175`, and aggregate `91786468184`. This controls only
  final-entry init preflight; rollback, atomicity, race freedom, ancestor symlinks,
  and every compiler/backend capability remain outside the result.
- Exact triple-reviewed record closure `aa29a00`, tree `e740df48`, diff `3eb8264b`,
  passes compiler `30844324249` / `30844328660`, Rust `30844328850`, CodeQL
  `30844325051`, and aggregate `91788926688`. `CORE-022` is complete only for the
  selected final-entry preflight; R-013 and all broader capability boundaries retain
  their recorded residual status.
- Exact triple-reviewed status synchronization `21153f3` is all-eight public green in
  compiler `30844798322` / `30844802332`, Rust `30844802044`, CodeQL `30844799426`,
  and aggregate `91790481511`. `AUDIT-029` is preregistered as a read-only,
  delta-aware ranking of all eleven remaining OPEN or PARTIALLY CONTROLLED risks. It
  cannot repeat an accepted slice, authorize implementation, or promote any
  capability.
- Exact triple-approved `AUDIT-029` basis `0e5cba1`, tree `6ac88db4`, is all-eight
  public green in compiler `30845609442` / `30845612610`, Rust `30845612328`,
  CodeQL `30845609103`, and aggregate `91793190047`. The three independent full
  rankings recommend different top slices: R-002 Boolean helper contracts, R-010
  grammar-authority containment, and R-009 parser UTF-16 columns. R-012 is their
  common second-place evidence-only slice. Lead reconciliation selects R-002 because
  semantics currently accepts invalid Boolean helper calls/returns and mis-infers
  valid Boolean call results as `Int`, while checked IR already has active exact
  `bool`/LLVM-`i1` function evidence. `CORE-023` freezes only monomorphic non-entry
  helper contracts in the semantic phase. No current capability is promoted before
  tests-first, implementation, full-gate, review, and public evidence.

## Audit completion

All eight requested read-only areas were completed in bounded waves. The audit
supports a correctness-first Milestone 0: close phase-boundary false success,
then function/type contracts and typed IR/CFG invariants, before expanding the
language or backend surface.

## Initial capability conclusion

- `STABLE`: no audited feature.
- `END_TO_END`: not yet assigned pending executable and negative evidence review.
- `PARTIAL`: numeric core, bindings, functions/control flow, strings, arrays,
  basic CPU LLVM path, diagnostics.
- `PARSED_ONLY`: function contracts, annotations, generics, traits, pattern
  matching, modules/visibility, closures, and several structured forms.
- `EXPERIMENTAL`: accelerator interfaces, graph/quantization transforms,
  collections, LSP, formatting, documentation/profiling/project/registry tooling,
  and the conformance command.

See `SPEC_IMPLEMENTATION_MATRIX.md` and `BACKEND_STATUS.md` for stage-level
classification. These labels describe evidence at the audited commit and do not
promise future compatibility.
