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
