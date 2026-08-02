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
- Status: complete
- Verification commands: focused Cargo integration test; manual CLI reproducer;
  `./tools/test.sh`.
- Result commit: `30b9b48658b0e1b1638b273341044dc2c8d64646`
- Final decision: accepted together with `CORE-001B`; the original false-success
  path is closed and the remaining public compiler paths are covered below.

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
- Status: complete
- Verification commands: focused Cargo integration test, manual CLI reproducers,
  `./tools/test.sh`.
- Result commits: `56be7fb1`, `45dd11df`, and `6ce85922` on the integration
  branch (isolated owner commits `4fca5e1d`, `ce474b5a`, and `9691d02b`).
- Final decision: accepted at
  `6ce859220634c40c696397a3df178faea51f1912` after two independent approvals.
  Focused tests pass 11/11; the full repository gate passes; manual root and
  imported-module builds exit 1 with located parser diagnostics and no output.
  Direct-module valid probes for `check` and `test` retain status zero. Recursive
  modules and the pre-existing helper-level process exit are separate tasks.

## CORE-002 — Reject lexically invalid source before parsing

- Problem: The lexer prints and skips unexpected characters, substitutes zero for
  failed numeric parses, and emits string tokens even when the closing quote is
  absent. Invalid source can therefore acquire different valid semantics.
- Evidence: at `6ce85922`, `let value = 1@;` exits 0 and emits a stored `1.0`;
  `let value = 9223372036854775808;` exits 0 and emits a stored `0.0`. Both write
  LLVM and report success. An unterminated string is rejected only later as a
  parser error, after the lexer has fabricated a completed string token.
- Priority: P0
- Primary hypothesis: an additive strict located-token API returning the existing
  `CompilerError` lexer variants can reject lexical corruption while preserving
  the legacy recovery API for non-codegen callers and existing parser tests.
- Dependencies: fatal parser milestone closed by `b3f0f4466214e80a161754e028fd023a1ab73200`.
- Observed behavior: unexpected input is discarded; integer overflow becomes zero;
  malformed exponents become zero; non-finite floats are accepted; unterminated
  ordinary and formatted strings become literal tokens.
- Expected behavior: strict lexing returns the first located lexical error with no
  token stream. Library/build/check/run/test/profile/doc/conformance and LSP
  diagnostics use strict lexing; build/run emit no artifact and do not reach native
  tools; profile and doc emit no requested output; applicable direct modules follow
  the same rule.
- Smallest reproducers: `let value = 1@;`, integer `9223372036854775808`,
  `let value = 1e+;`, `let value = 1e9999;`, and `let value = "unterminated`.
- Files allowed: `src/compiler/src/lexer.rs`, `src/compiler/src/lib.rs`,
  `src/compiler/src/main.rs`, `src/compiler/src/profiler.rs`,
  `src/compiler/src/doc_generator.rs`, `src/compiler/src/lsp.rs`,
  `src/compiler/src/parser.rs` only for interpolation-expression strict lexing, a
  new focused integration test under `src/compiler/tests/`, affected control
  documents, and—by preregistered review amendment—
  `src/compiler/src/conformance.rs` only for strict lex/fallible parse migration.
- Files frozen: token enum and meanings, accepted lexical forms, parser grammar and
  recovery, AST, semantic/type/ownership rules, IR/optimization/backend lowering,
  module recursion, formatter behavior, registry, claims and public language docs.
- Frozen semantics: every previously valid finite literal/token sequence produces
  the same tokens. Strict mode never skips or substitutes. Legacy `tokenize` and
  `tokenize_with_locations` remain source-compatible but are recovery-only and
  ineligible for artifact-producing paths.
- Positive tests: strict and legacy APIs agree on representative valid tokens;
  valid library, module, profiler, doc, LSP and full suites remain passing.
- Negative tests: located strict-API errors for unexpected character, integer
  overflow, malformed/non-finite float, unterminated string and f-string; library
  propagation; CLI root/direct-module rejection and no outputs; LSP diagnostic.
- Runtime-output tests: malformed `run` invokes no native tool and leaves no nonce
  directory. Valid native execution is unchanged and unavailable on this host.
- Diagnostic expectation: stable `Lex error` category plus existing
  `Unexpected character`, `Invalid number format`, or `Unterminated string literal`
  text and filename/line/column. LSP lexical diagnostics use an `aero-lexer` source,
  include the category, and convert source columns to default UTF-16 protocol
  positions. Invalid f-string interpolation is rejected during parsing with its
  underlying lexical error until end-to-end subexpression spans exist.
- Regression risks: accidentally changing legacy behavior; duplicating the large
  scanner; accepting infinity as a float; losing filename ownership across callers;
  LSP symbol recovery must remain available independently of strict diagnostics.
- Acceptance criteria: tests are red only for the specified corruptions before
  implementation; focused tests and `./tools/test.sh` pass; manual unexpected-char
  and overflow builds exit nonzero and create no fresh artifact; two independent
  reviews approve the exact integration SHA.
- Stop conditions: a token/AST/grammar change is required; strict scanning cannot
  be added without duplicating the lexer; location correction requires a new span
  model; accepted numeric or escape semantics would change; files exceed the list.
- Owner: one isolated implementation agent; lead integrates.
- Status: complete
- Review amendment: independent review of initial integration `379ec1e` requested
  changes because public conformance still fed recovery tokens to semantics/IR,
  doc skipped direct-module validation, and LSP labeled lexical errors as parser
  diagnostics with scalar rather than UTF-16 columns. Scope was expanded before
  corrective code. Direct-module doc validation must not change valid doc content;
  conformance lexical/parse failures become explicit failed results, never panics.
- Verification commands: strict lexer unit tests, focused Cargo integration tests,
  manual CLI reproducers, LSP diagnostic test, and `./tools/test.sh`.
- Result commits: initial integration `379ec1e6`; review closure `fefe59e2` and
  `b9883181` (isolated owner commits `9f4aa19e`, `b4ddc953`, `e9ad19d3`).
- Final decision: accepted at
  `b9883181414886b6b2775b149599da29faed933e` after two independent approvals.
  Strict integration tests pass 12/12; conformance 3/3; LSP 9/9; doc 2/2;
  fatal parse 11/11; full gate passes. Fresh unexpected-character, overflow, and
  broken-module doc probes exit 1 with located lexical errors and no outputs.
  LSP recovery indexing is the only intentional production recovery consumer;
  strict lexing returns the first error and recursive modules remain separate work.

## CORE-003 — Enforce numeric function contracts before lowering

- Problem: Named calls are inferred as `int`; declared signatures and return types
  are not registered or checked; missing returns are replaced with zero; invalid
  programs therefore reach IR and artifact output.
- Evidence: at `b535de0f`, undefined calls, too few/many arguments, mismatched
  primitive arguments and returns, missing/bare/value returns, duplicate functions,
  and using a void call as a value all pass `check`, pass `build`, and produce LLVM.
- Priority: P0
- Primary hypothesis: a declaration pass over top-level function names and eligible
  primitive signatures, followed by checked body/call analysis, can reject invalid
  contracts before IR while preserving forward references and recursion. A matching
  preregistered return-type map in IR can remove the second `int` fallback for valid
  primitive calls without changing the backend instruction model.
- Dependencies: strict trusted frontend boundary closed by
  `b535de0f5723e26664d818c076db4e451ff35315`.
- Eligible contract syntax: monomorphic `int`/`i32` and `float`/`f64` parameter
  and return types; omitted return type means `void`. Call arguments must match
  exactly. Numeric promotion remains an operator rule and is not applied at call
  boundaries. Boolean signatures are explicitly excluded because the current
  backend converts call results through `double` while boolean control-flow and
  returns require `i1`.
- Observed behavior: `SemanticAnalyzer::analyze` is one pass; its function table is
  unused; calls always return `Ty::Int`; function return annotations are ignored.
  `IrGenerator` independently assigns `Ty::Int` and a result register to every call
  and can insert a default scalar-zero return.
- Expected behavior: all top-level function names are globally visible before body
  analysis; duplicate and undefined names fail; eligible calls check exact arity and
  types and infer the declared result; explicit and tail returns match the declared
  type; void calls are valid only in value-discarding statement position; eligible
  non-void bodies must conservatively return on all paths or end in a tail value.
  Valid primitive calls retain their result type through IR, and valid void statement
  calls lower without a result register.
- Smallest reproducers: `missing(1);`; `fn f(x: i32) -> i32 { return x; } f();`;
  `fn f(x: i32) -> i32 { return x; } f(1.0);`;
  `fn f() -> i32 { return 1.0; }`; `fn f() -> i32 { }`;
  `fn f() { return 1; }`; duplicate `fn f() {}` declarations; and
  `fn log() {} let value = log();`.
- Files allowed: `src/compiler/src/semantic_analyzer.rs`,
  `src/compiler/src/ir_generator.rs`, one new focused integration test under
  `src/compiler/tests/`, and affected control documents. Tests may inspect generated
  LLVM through the public API; production `code_generator.rs` is frozen.
- Files frozen: lexer, parser, AST, annotations on `let`, implicit conversion rules,
  booleans, generics, traits, composites, strings, methods, closures, ownership behavior,
  optimizer, IR instruction shape, code-generator production, runtime/backends,
  modules beyond existing direct-module concatenation, and public claims.
- Frozen semantics: generic or non-eligible declared calls retain their pre-task
  behavior and are not counted as contract support. Named calls must resolve to a
  top-level declaration; if existing supported built-ins or closure calls require a
  separate callable registry, stop rather than silently exempting unknown names.
  Semantic diagnostics are not promised source locations because the AST has none.
- Positive tests: exact numeric calls; call before declaration; direct recursion;
  explicit and tail returns; void statement call; float-return call participating in
  float arithmetic; direct-module declaration visibility.
- Negative tests: undefined and duplicate functions; too few/many arguments; both
  `i32`/`f64` mismatch directions; explicit/tail return mismatch; missing return;
  bare return in non-void; value return in void; void call used as a value; direct
  module mismatch. CLI build failures create no requested artifact.
- Runtime-output tests: not applicable on this host because LLVM tools are absent;
  generated LLVM is inspected for typed/void call shape and float opcode selection.
- Diagnostic expectation: stable semantic category at public boundaries and
  function name plus expected/actual arity or type. No fabricated span.
- Regression risks: generic and trait tests currently rely on permissive calls;
  function scopes use compatibility symbol tables; method names are not globally
  unique; the binary and library compile separate copies of the analyzer; IR has two
  call-generation paths; codegen reconstructs signatures independently.
- Acceptance criteria: preregistered tests fail for the listed false accepts before
  implementation; focused tests pass; `./tools/test.sh` passes; fresh CLI root and
  direct-module negative probes exit nonzero with no output; two independent reviews
  approve the exact integration SHA.
- Scope amendment: pre-implementation IR review excluded boolean signatures. A
  boolean-returning call is converted from `i1` to the backend's internal `double`
  register form, while boolean branches/returns may consume that register as `i1`.
  Certifying booleans would therefore require the frozen code-generator phase.
- Stop conditions: parser/AST or IR instruction changes are required; production
  code-generator changes are required; valid generic/composite/method/closure or
  ownership behavior regresses; exact primitive calls cannot be lowered consistently;
  definite-return checking requires a CFG redesign rather than the conservative
  `return`/block/if-else structure in this slice.
- Owner: one isolated implementation agent; lead integrates.
- Status: complete.
- Verification commands: focused Cargo integration/unit tests, public API LLVM
  assertions, manual CLI reproducers, and `./tools/test.sh`.
- Review amendments: the first candidate was rejected because nested/branch tail
  values were treated as function returns even though IR discarded them, completed
  `if` arms received branches after terminators, void closure bodies fabricated a
  value, `int` parameters were not canonicalized, and closure bindings leaked across
  lexical scopes. A second review found reachable partial-return void merges without
  an epilogue. The accepted implementation restricts implicit return to the outer
  function tail, restores callable bindings at scope exit, treats local closures as
  shadowing top-level functions, and emits only terminator-safe branches/epilogues.
- Verification result: exact clean candidate
  `8d5d8e7cc92f712fccc3af65cc4f06a1d7b1dd9a` passed 13/13 focused contract tests,
  112 library tests, 119 binary tests, 11 fatal-parse tests, 59 frontend tests, and
  12 strict-lex tests under `./tools/test.sh`; all 38 pre-existing phase-five tests
  remain ignored. Fresh black-box closure-shadowing and invalid nested-tail probes
  produced the expected calls or nonzero failures with no artifact. Two independent
  reviewers approved that exact SHA after reproducing the focused suite.
- Scope closure: monomorphic numeric and void top-level function boundaries are
  controlled for this slice. Boolean signatures, annotations, generics, composites,
  methods, string contracts, richer closures, and unreachable statements after a
  terminator remain uncertified. No parser, AST, IR-instruction, code-generator, or
  backend production file changed.
- Result commits: `2d4f3ca` (red tests), `5e58922`, `bf30d62`, `1ab6a91`,
  `554e39e`, and accepted code candidate `8d5d8e7`.

## CORE-004 — Enforce initialized numeric binding annotations

- Problem: `let` annotations are preserved by the parser but discarded by active
  semantics and both IR paths. Numeric cross-family mismatches therefore pass
  validation and artifact generation while later uses inherit the initializer type
  rather than the declared type.
- Evidence: at `4df60153`, all four literal mismatches (`int`/`i32 = 1.5` and
  `float`/`f64 = 1`), both numeric named-call mismatch directions, and a mismatch in
  a direct module exit zero from both `check` and `build` and emit an LLVM artifact.
  Matching aliases also compile. Scalar locals lower through the existing unified
  `double` slot representation regardless of annotation.
- Priority: P0.
- Dependencies: `CORE-001B`, `CORE-002`, and `CORE-003` complete.
- Contract: for an initialized binding whose annotation is exactly `int`, `i32`,
  `float`, or `f64`, canonicalize aliases to `Ty::Int`/`Ty::Float` and require the
  initializer's already-inferred type to equal that canonical type. Reject a
  mismatch before ownership mutation, variable registration, IR, or artifact output.
  The variable is registered with the canonical declared type.
- Existing inference: no assignment-site conversion is added. A mixed numeric
  expression may satisfy a float annotation only when the existing expression rules
  already infer `Ty::Float`; numeric function results use the accepted `CORE-003`
  return contract. `let mut` follows the same initialization rule.
- Eligible positions: root, function body, nested block/control-flow scope, and
  existing directly resolved modules, all through the active semantic `let` arm.
- Smallest reproducers: `let value: int = 1.5;`,
  `let value: float = 1;`, `fn one() -> int { 1 } let value: f64 = one();`,
  and `fn ratio() -> float { 1.5 } let value: i32 = ratio();`.
- Files allowed: `src/compiler/src/semantic_analyzer.rs`,
  `src/compiler/src/ir_generator.rs`, one new focused integration test under
  `src/compiler/tests/`, and affected control documents.
- Files frozen: lexer, parser, AST, type/IR instruction shapes, code generator,
  optimizer, function contracts, implicit conversion rules,
  booleans, strings, arrays, tuples, references, generics, composites, closures,
  ownership behavior, runtime/backends, and public claims.
- Frozen semantics: unannotated bindings preserve inference. Non-numeric annotations
  and annotations without an initializer retain pre-task behavior but are not counted
  as supported annotation contracts. Reassignment has no parser/AST form and is out
  of scope. Same-scope rebinding remains rejected and nested shadowing remains valid.
- Positive tests: all four exact alias/literal forms; aliases initialized from
  identifiers; exact numeric function results; a float mixed-arithmetic result;
  `let mut`; and nested cross-family shadowing with valid initializers. Public LLVM
  checks assert existing numeric function ABI/casts, not integer local stack slots.
- Negative tests: both literal mismatch directions for both aliases; mixed-expression
  result mismatch; numeric function-result mismatch in both directions; function-local
  and nested binding mismatches; and a direct-module mismatch. CLI `check`/`build`
  failures are nonzero and create no requested artifact.
- Diagnostic expectation: stable semantic category with binding name and canonical
  expected/actual type, for example ``Variable `value` type annotation mismatch:
  expected int, actual float``. No fabricated source span.
- Red checkpoint: matching cases pass before implementation; every mismatch test
  must demonstrate the current false accept and artifact behavior before production
  code changes.
- Review amendment: exact candidate `5fa5a5eba41ce33fcf06767992efa1b810f9ebaa`
  passed all 12 focused tests but was independently rejected. After an inner
  cross-family shadow ended, IR still loaded the inner `%ptr1` value rather than the
  outer `%ptr0` value because scope exit restored only `Ty::Fn` bindings. This is a
  silent valid-program miscompile and proves that the original compile-success-only
  shadowing test was insufficient.
- Corrective contract: every IR lexical scope already represented by an explicit
  block, `if` arm, `while`, `for`, or `loop` must restore the complete pre-scope
  symbol-table snapshot, including scalar and callable bindings. Alternate `if` arms
  start from the same pre-branch snapshot. Storage and instructions produced inside
  a scope remain in IR, but their source names must not remain resolvable afterward.
  This generalizes the accepted callable-only restoration without changing IR shape.
- Corrective red checkpoint: add public LLVM regressions proving that a use after a
  nested block and after a control-flow arm loads the outer numeric slot, while all
  accepted `CORE-003` callable-shadowing/restoration tests remain green. The new
  regressions must fail on `5fa5a5e` before the IR correction.
- Second review amendment: exact candidate
  `b6b0eba1d560e2782d052ab503ac46b9c09cbbdb` fixed scalar/callable IR restoration,
  passed 14/14 focused tests and the complete gate, but was independently rejected.
  A name declared only in the `then` arm remains in the semantic analyzer's flat
  compatibility table, so the `else` arm is falsely accepted; corrected IR scope
  restoration then removes the name and public compilation panics with
  `Undeclared variable` instead of returning a semantic error.
- Semantic scope amendment: snapshot and restore the private compatibility
  `symbol_table` alongside every existing semantic block, function, and loop scope,
  and reset the private snapshot stack at each `analyze()` call. `ScopeManager`
  remains authoritative; compatibility lookup may serve only names visible in the
  current lexical scope. This must not change public analyzer APIs or ownership state.
- Second corrective red checkpoint: public compilation of a then-arm-only name used
  in `else`, a block-only name used after the block, and loop-body-only names used
  afterward must return an undeclared-variable semantic error without unwinding.
  Independent same-name declarations in both `if` arms remain valid. These tests
  must panic or falsely accept on `b6b0eba` before the semantic correction.
- Regression risks: the binary and library compile separate analyzer copies; the
  compatibility symbol table still outlives lexical scopes; non-numeric inference
  contains fallback types; direct callers can construct AST/IR without semantic
  validation; scalar backend storage is not a typed binding representation.
- Stop conditions: satisfying the amended contract requires parser/AST, IR
  instruction, code-generator, ownership, or conversion-policy changes; exact
  equality cannot be established from the active initializer inference; a valid
  existing numeric program regresses; or the focused change would make a
  noneligible annotation appear supported. Stop again if full snapshot restoration
  changes callable scope behavior, requires a new IR representation, or exposes an
  assignment/merge policy not frozen here. Stop if compatibility-table isolation
  requires removing public APIs, altering `ScopeManager`, or redesigning ownership.
- Owner: one isolated implementation agent; lead integrates.
- Status: complete at `bc9a14820af6b3127a8dad5fffd4ccf55d9c9d2f`.
- Acceptance criteria: red evidence is preserved; focused positive/negative/public
  API and CLI artifact tests pass; `./tools/test.sh` passes; and two independent
  reviewers approve the exact clean integration SHA.
- Candidate commits: `ccbb144` (red tests) and `5fa5a5e` (semantic implementation;
  rejected for scalar IR scope leakage), `39b5f40` (scope-provenance red tests), and
  `b6b0eba` (IR restoration; rejected for compatibility-table scope panic), and
  `1ec3c6b` (semantic-scope red tests). Accepted result commit: `bc9a148`.
- Accepted behavior: initialized exact numeric annotations are enforced before
  ownership mutation or IR; the compatibility table and complete IR binding map are
  restored at the existing lexical scope exits; invalid cross-scope references
  return semantic errors rather than reaching an IR panic. Uninitialized and
  non-numeric annotations, reassignment, and typed local storage remain uncertified.
- Verification: exact clean SHA `bc9a148` passes 18/18 focused annotation and scope
  tests, 13/13 numeric function-contract tests, and `./tools/test.sh` (112 library,
  119 binary, 11 fatal-parse, 59 frontend, 13 function-contract, 18 annotation,
  and 12 strict-lex tests; 38 pre-existing phase-five tests remain ignored).
  Two independent reviewers approved that exact SHA after no-unwind, diagnostic,
  artifact, binding-provenance, callable-restoration, and analyzer-reuse probes.

## CORE-005 — Reject unsupported modulo before IR

- Problem: `%` is lexed, parsed, and accepted as numeric by active semantics, but
  neither IR expression path nor the IR/backend instruction set implements
  remainder. A semantically accepted program therefore reaches an infallible IR
  panic instead of receiving a language diagnostic.
- Evidence: at `c000d916`, integer literals, integer variables in a function,
  floats, mixed numeric operands, and a zero right operand all make CLI `check`
  exit 0. CLI `build` exits 101 at `ir_generator.rs` with `Unsupported binary
  operation: % for type Int/Float`, writes no requested artifact, and public
  `compile_program` unwinds. Multiplication/division controls build successfully.
- Priority: P0.
- Dependencies: `CORE-001B` through `CORE-004` complete.
- Decision: retain `%` in the lexer, grammar, AST, and parser so source structure
  and precedence remain explicit, but reject it in shared binary type inference
  with the exact semantic category ``Binary operator `%` is not supported.`` No
  `%` expression is eligible for IR or artifact generation in this slice.
- Compatibility consequence: this is a documented temporary grammar/conformance
  exception. It changes `check` from false success to a diagnostic, but no `%`
  program has a successful artifact path at the audited commit. The parser form is
  preserved for a future complete remainder design.
- Why not lower now: the repository does not freeze negative-operand, floating,
  mixed, signed-zero, NaN/infinity, or zero-divisor semantics. Integer locals are
  represented as LLVM `double`, so mapping both source families to `frem` would not
  establish faithful integer remainder semantics.
- Smallest reproducer: `fn main() { let left: int = 5; let right: int = 2; let
  value: int = left % right; }`.
- Files allowed: `src/compiler/src/types.rs`, one new focused integration test
  under `src/compiler/tests/`, `tutorials/02-core-features.md`, and affected
  project-control documents.
- Files frozen: lexer, grammar, parser, AST, semantic analyzer structure, IR,
  code generator, optimizer, numeric promotion for `+ - * /`, annotations,
  function contracts, constant division behavior, all other unsupported
  expressions, runtime/backends, and public stability claims.
- Positive tests: parser still produces `BinaryOp::Modulo` at multiplicative
  precedence; public compilation and CLI `check`/`build` continue to accept
  representative `+ - * /` programs and preserve their LLVM markers.
- Negative tests: integer literal, integer identifier/function, float, mixed,
  zero-RHS, nested comparison, root expression, and direct-module `%` forms.
  Public compilation is wrapped to prove it returns `Err` without unwinding. CLI
  `check` and `build` exit nonzero with the stable diagnostic and create no
  requested artifact.
- Red checkpoint: on `c000d916`, the parser/adjacent controls pass, all `%` checks
  falsely succeed, and every `%` public/build compilation unwinds before a result.
- Regression risks: `infer_binary_type` is shared by both analyzer paths and a
  public compatibility helper; binary and library pipelines compile separate module
  copies; docs and examples advertise `%`; direct callers can bypass semantics and
  invoke the infallible IR generator with a constructed modulo AST.
- Stop conditions: implementation requires any IR/backend change, selects remainder
  execution semantics, changes parsing/precedence, changes another arithmetic
  operator, weakens the diagnostic/no-artifact contract, or reveals an existing
  successful `%` artifact/runtime path. Stop if the rejection cannot be expressed
  in shared type inference without semantic-analyzer redesign.
- Owner: one isolated implementation agent; lead integrates.
- Status: complete at reviewed candidate
  `302211e6226c1580a8b4aec66790b37c03888db6`.
- Acceptance criteria: red evidence is preserved; focused positive, negative,
  diagnostic, no-unwind, direct-module, and CLI artifact tests pass;
  `./tools/test.sh` passes; tutorial/capability records label `%` unsupported; and
  two independent reviewers approve the exact clean integration SHA.
- Verification commands: focused `cargo test --test unsupported_modulo_tests`,
  applicable arithmetic/function regressions, `cargo fmt --all -- --check`, and
  the required `./tools/test.sh` gate.
- Red checkpoint: owner commit `3eeeca5`, integrated as `3a6c988`, preserves
  3 passing parser/adjacent controls and 11 expected failures on the preregistration
  base. Production owner commit `fc2e23f` is integrated as `028bb5e`; tutorial
  correction is `302211e`.
- Accepted result: shared inference returns the frozen diagnostic for every `%`
  node whose operands can be typed. Ill-formed operand subtrees retain their earlier
  diagnostic. Lexer/parser/AST/semantic structure/IR/backend/optimizer are unchanged,
  and `+ - * /` retain their prior typing and LLVM markers.
- Verification: exact clean SHA `302211e` passes 14/14 focused modulo tests,
  13/13 function-contract tests, 18/18 annotation tests, formatting, and
  `./tools/test.sh` (112 library, 119 binary, 11 fatal-parse, 59 frontend,
  13 function-contract, 18 annotation, 12 strict-lex, and 14 modulo tests;
  38 pre-existing phase-five tests remain ignored). Two non-owner reviewers approve
  the exact SHA after fresh shared-helper, nonnumeric, unary/negative, nested,
  root/direct-module, diagnostic, no-unwind, no-panic, and artifact probes.
- Result commit: `028bb5e` for production behavior; exact accepted integration
  candidate including tests and public documentation: `302211e`.

## CORE-006 — Reject unsupported tuple value expressions before IR

- Problem: tuple literals and tuple-index expressions are parsed and assigned an
  invented scalar `int` type, while both active IR expression paths replace the
  entire value with integer zero. Valid source such as
  `fn main() { let value: int = (7, 9).0; }` therefore passes `check` and `build`
  but stores zero rather than seven.
- Evidence: at `704b3328`, `(11, 22)` and `(11, 22).1` both emit an artifact whose
  function body allocates a scalar, stores `double 0`, and contains neither tuple
  constant. A tuple hidden in another expression can also bypass subtree
  validation. Arrays, array indexing, and `.iter()` have distinct nonzero lowering
  and remain positive controls.
- Priority: P0.
- Dependencies: `CORE-001B` through `CORE-005` complete; `AUDIT-011` complete.
- Decision: retain tuple literal, tuple-index, tuple-type, and tuple-pattern syntax,
  but reject every `Expression::TupleLiteral` and `Expression::TupleIndex` in the
  active semantic expression preflight with the exact diagnostic
  `Tuple expressions are not supported.` No tuple value expression is eligible
  for IR or artifact generation in this slice.
- Boundary: rejection is recursive, including tuple nodes used as a root, binding
  initializer, return/tail/discarded expression, condition/iterable, or beneath an
  array element, call or method argument/object, struct field, enum payload, match
  scrutinee/arm, borrow/dereference, unary/binary/logical expression, closure body,
  field object, or index object/index. Parent forms are not thereby certified.
- Diagnostic ordering: once the preflight reaches an unsupported tuple node, the
  frozen tuple diagnostic wins over analysis of that tuple's children. Errors in
  source evaluated before reaching that node retain their existing order. No source
  span is fabricated.
- Compatibility consequence: tuple value programs that previously reported false
  success now fail during semantics. This is a documented temporary conformance
  exception, not removal of a working execution path: no audited tuple literal or
  projection preserved its values in an artifact.
- Syntax retained: tuple types and patterns remain representable; parenthesized
  scalar grouping such as `(7)` remains valid. Tuple structs and tuple-like enum
  declarations are separate syntax and behavior and are frozen.
- Files allowed: `src/compiler/src/semantic_analyzer.rs`, one new focused
  integration test under `src/compiler/tests/`, and affected language/control
  documentation.
- Files frozen: lexer, grammar, parser, AST, type representation, tuple layout and
  indexing semantics, IR, code generator, optimizer, arrays/indexing/iteration,
  structs, enums, match behavior, fields, methods, closures, ownership, numeric
  contracts/conversions, runtime/backends, and public stability claims.
- Positive tests: parser still distinguishes tuple literal and tuple projection;
  grouped scalar expressions, numeric arithmetic, arrays, array indexing, and
  `.iter()` retain their accepted behavior and existing LLVM markers.
- Negative tests: direct literal/projection, function return/tail, discarded/root,
  direct-module, and representative nested locations. Public compilation is
  wrapped to prove it returns `Err` without unwinding; CLI `check` and `build` must
  exit nonzero with the exact diagnostic and create no requested artifact.
- Red checkpoint: on `704b3328`, parser and adjacent positive controls must pass;
  every tuple negative must demonstrate false acceptance, fabricated-zero output,
  or a later pipeline failure before production code changes.
- Regression risks: the existing preflight was introduced for nested void calls;
  mutable and immutable inference functions are duplicated; initialization
  traversal skips composites; binary and library targets compile analyzer copies;
  closure and discarded-value routes have special handling; constructed AST callers
  can bypass semantics and invoke infallible IR directly.
- Stop conditions: implementation requires tuple layout, projection execution,
  index bounds, parser/AST/type/IR/backend changes, changes another expression
  family's semantics, more than one production file, or cannot cover all active
  semantic routes through one shared recursive preflight. Stop if a currently
  value-preserving tuple execution path is found or if an array/index/iterator,
  grouped scalar, numeric function, or lexical-scope regression appears.
- Owner: one isolated implementation agent; lead integrates.
- Status: complete at reviewed candidate
  `cbbe049bee7664abb3ca9b8d1faaa865345eb440`.
- Acceptance criteria: red evidence is preserved; focused parser, positive,
  negative, nested, diagnostic, no-unwind, direct-module, and CLI no-artifact tests
  pass; prior focused suites and `./tools/test.sh` pass; user-facing tuple claims are
  corrected; and two independent reviewers approve the exact clean integration SHA.
- Verification commands: focused `cargo test --test unsupported_tuple_tests`,
  applicable function/annotation/modulo regressions, `cargo fmt --all -- --check`,
  and the required `./tools/test.sh` gate.
- Red checkpoint: owner commit `89b8b22`, integrated as `6a75f93`, preserves
  3 passing parser/adjacent controls and 13 expected false-accept or fabricated-zero
  failures on the preregistration base. No negative route depended on a panic.
- Candidate result: owner production commit `1a180c7` is integrated as `1fa67a2`;
  user-facing tuple status and the implementation matrix are corrected at `669588d`.
  The single production file renames the existing exhaustive semantic helper to
  `preflight_expression` and immediately rejects both tuple value AST forms before
  inspecting their children. Parser/AST/type/IR/backend files are unchanged.
- Candidate verification: exact clean SHA `cbbe049` passes 16/16 tuple tests,
  14/14 modulo tests, 13/13 function-contract tests, 18/18 annotation tests,
  12/12 strict-lex tests, formatting, and `./tools/test.sh` (112 library,
  119 binary, 11 fatal-parse, 59 frontend, 13 function-contract, 18 annotation,
  12 strict-lex, 14 modulo, and 16 tuple tests; 38 pre-existing phase-five tests
  remain ignored).
- Independent acceptance: Reviewer A approved the exhaustive-preflight structure
  after fresh nested public probes, void/diagnostic-precedence checks, constructed
  AST-root coverage, and 61 focused regressions. Reviewer B approved after an
  independently authored 18-route public/CLI matrix: 18/18 exact public errors
  without unwind, 18/18 nonzero CLI checks, 18/18 nonzero builds, no panics, and no
  requested artifacts, plus seven tuple-free controls. Both reviewers inspected the
  final documentation-only candidate delta, changed no repository file, and approved
  exact clean `cbbe049` with no P0-P3 finding.
- Accepted result: tuple literal and tuple-index syntax remains explicit, but every
  tuple value node reached by trusted semantic analysis fails before IR with
  `Tuple expressions are not supported.` Parenthesized grouping and the tested
  array/index/iterator/numeric/function slices retain their prior behavior. Direct
  constructed-AST callers, tuple types/patterns, and all parent composite semantics
  remain outside the accepted boundary.

## CORE-007 — Reject unsupported field-access value expressions before IR

- Problem: every active `Expression::FieldAccess` is assigned an invented scalar
  `int` type and both IR expression paths replace the entire expression with zero
  without evaluating its receiver. Valid specified source such as
  `Point { x: 7 }.x` therefore compiles but does not preserve seven; even
  `missing.field` is falsely accepted.
- Evidence: at `52d3415`, literal, bound, function-call, real struct-literal,
  undeclared, chained, top-level, function, argument, and nested field expressions
  pass semantics and public/CLI compilation. Artifacts contain a scalar zero, no
  field GEP, and—where the receiver is a call—no receiver call. A direct child
  module containing an undeclared receiver also passes `check`/`build` and emits an
  artifact. Numeric/array/index/iterator controls have distinct real lowering.
- Priority: P0.
- Dependencies: `CORE-001B` through `CORE-006` complete; `AUDIT-012` complete.
- Candidate comparison: string/string comparisons for all six operators pass
  semantics then panic in both IR paths, but complete recursive detection needs
  trustworthy operand typing and an equality/order policy. Immediate integer `/ 0`
  panics in host constant folding, while variable, unary, float, and mixed zero
  forms behave differently; a coherent fix needs constant-evaluation and arithmetic
  exception semantics. Field access is one AST family with no value-preserving path
  and no layout policy required for fail-closed rejection.
- Decision: retain dot syntax and the `Expression::FieldAccess` AST node, but reject
  every field-access value expression in active recursive semantic preflight with
  the exact diagnostic `Field access expressions are not supported.` No named field
  projection is eligible for IR or artifact generation in this slice.
- Diagnostic ordering: preflight the receiver first, then return the field-access
  diagnostic. Existing tuple and void-call diagnostics in the receiver therefore
  retain precedence. Otherwise the field diagnostic wins before receiver type/name
  inference, including for a plain undeclared receiver. A chain reports the first
  inner field node reached. No source span is fabricated.
- Boundary: rejection is recursive at roots, bindings, explicit/tail returns,
  discarded expressions, conditions, iterables, and closure bodies, and beneath
  arrays/repeats/indexing, calls/method arguments, struct fields, enum payloads,
  matches, borrows/dereferences, unary/binary/logical forms, prints, and other field
  receivers. Parent forms are not thereby certified.
- Syntax retained: parser behavior must continue to distinguish `base.field` from
  `base.method()` and `(1, 2).0`. Struct declarations/literals remain representable;
  this slice does not certify their storage or execution.
- Files allowed: `src/compiler/src/semantic_analyzer.rs`, one new focused
  `src/compiler/tests/unsupported_field_access_tests.rs`, `README.md`,
  `tutorials/04-data-structures.md`, `SPEC_IMPLEMENTATION_MATRIX.md`, and affected
  project-control/capability/conformance documents.
- Files frozen: lexer, grammar, parser, AST, types, struct registries/construction/
  layout/lookup/assignment/ownership/ABI, method calls, tuple projection, string
  comparisons, division semantics, IR, code generator, optimizer, runtime/backends,
  arrays/indexing/iteration, numeric promotion/contracts/annotations, dormant mutable
  inference, direct constructed-AST behavior, and public stability claims unrelated
  to fields.
- Positive tests: parser retains FieldAccess and distinguishes MethodCall/TupleIndex;
  grouped numeric arithmetic/comparison, arrays, indexing, `.iter()`, struct syntax
  and literals without projection, and prior tuple/modulo/function/annotation/strict
  slices retain their pre-task behavior and expected LLVM markers.
- Negative tests: literal, bound, undeclared, function-call, struct-literal, chained,
  root/binding/discarded/explicit-return/tail, direct-module, non-first array,
  closure, and representative nested positions. Public compilation must return the
  exact semantic `Err` without unwind. CLI `check`/`build` must exit nonzero without
  panic or requested artifact. Tuple and void-call receivers retain their existing
  diagnostics.
- Red checkpoint: on `52d3415`, parser/positive/precedence controls must pass;
  ordinary field negatives must demonstrate false acceptance, fabricated zero, or
  dropped receiver evaluation before production changes. No negative may rely on an
  unrelated parse failure.
- Regression risks: preflight ordering is now observable; parser dot conversion
  could accidentally affect methods; mutable/immutable inference and binary/library
  compiler modules are duplicated; field assignment may be a distinct future path;
  direct callers can bypass semantics and invoke infallible zero-stub IR.
- Stop conditions: any trusted value-preserving field path is found; implementation
  requires another production file, field declarations/types/layout/lookup,
  assignment/evaluation/ownership policy, or parser/AST/IR/backend changes; active
  routes cannot be covered through the shared preflight; tuple/void diagnostic
  precedence changes; or method/array/index/iterator/numeric/struct-syntax/prior
  focused behavior regresses.
- Owner: one isolated implementation agent; lead integrates.
- Status: accepted at exact reviewed candidate
  `4e10d4799b7873741a5eae9c66ac352b1709d75c`.
- Acceptance criteria: red evidence is preserved; focused parser, positive,
  negative, recursive, diagnostic-order, no-unwind, direct-module, and CLI
  no-artifact tests pass; prior focused suites and `./tools/test.sh` pass; public
  field claims are corrected; and two non-owner reviewers approve the exact clean
  documented integration SHA.
- Verification commands: focused
  `cargo test --test unsupported_field_access_tests`, prior applicable focused
  suites, `cargo fmt --all -- --check`, and required `./tools/test.sh`.
- Red checkpoint: owner commit `f363a03`, integrated as `7346edd`, preserves five
  parser/adjacent/receiver-precedence controls and three expected aggregated failures
  covering 13 public routes plus root/direct-module CLI check and build. All ordinary
  fields falsely succeed; builds create requested artifacts and call receivers are
  dropped. No negative relies on panic.
- Candidate result: owner production commit `142807f` is integrated as `75dbfba`.
  It adds one receiver-first error return to the existing field preflight arm; no
  inference stub or parser/AST/type/IR/backend file changes. Public field status and
  the split implementation-matrix row are corrected at `5dcb70b`.
- Candidate verification: exact clean `4e10d4799b7873741a5eae9c66ac352b1709d75c`
  passes 8/8 field tests, 16/16 tuple, 14/14 modulo, 13/13 function-contract,
  18/18 annotation, and 12/12 strict-lex tests (81 focused tests total), plus
  `./tools/test.sh` with 112 library, 119 binary, 11 fatal-parse, 59 frontend,
  13 function-contract, 18 annotation, 12 strict-lex, 14 modulo, 16 tuple, and
  8 field tests. The 38 pre-existing phase-five tests remain ignored.
- Independent acceptance: Reviewer A approved after structural inspection and
  25/25 fresh public-route probes, receiver-diagnostic precedence checks, a
  constructed AST-root probe, five positive controls, and all 81 focused tests.
  Reviewer B approved after an independent 27-route public/check/build matrix,
  direct-module and no-artifact checks, receiver precedence, parser distinctions,
  and positive LLVM-marker controls. Both changed no repository file and approved
  exact clean `4e10d4799b7873741a5eae9c66ac352b1709d75c` with no P0-P3 finding.
- Accepted result: every named field value expression reached through trusted
  semantic analysis fails before IR with
  `Field access expressions are not supported.` Receiver-first tuple and void-call
  diagnostics, method-call syntax, tuple projection syntax, and the tested numeric,
  struct-syntax, array, index, and iterator controls retain their prior behavior.
  Direct AST-to-IR bypass, dormant inference stubs, field assignment, struct
  execution/layout/ownership, unknown methods, and parent composites remain outside
  the accepted boundary.

## AUDIT-013 — Compare the next unsupported-expression failure boundaries

- Objective: compare all-six string comparisons, MethodCall fabricated values,
  Match/composite fabricated values, and integer/float zero division on the exact
  clean post-`CORE-007` closure without changing repository files or choosing
  string, method-dispatch, arithmetic, pattern, ownership, or aggregate semantics.
- Audited commit: `9fc7d0e8f7a955b59924d31df968fcf61bfaaa80`.
- String comparison: all six `String/String` operators pass semantics and panic in
  both IR comparison paths. A 46-case public harness and 64 CLI invocations cover
  literals, bindings, declared-string calls, recursive positions, direct modules,
  mixed rejection, established diagnostic precedence, and numeric controls. A
  syntax-only preflight cannot identify bound/call-derived strings; active return
  typing is not trustworthy for nonnumeric calls; rejecting every Comparison would
  regress accepted numeric behavior. Equality/order and representation policy remain
  unresolved, so this family is not eligible for a bounded implementation.
- Method calls: semantic inference ignores argument arity/types and assigns many
  named or unknown methods invented types. Both IR paths evaluate the receiver but
  drop every argument and return zero except for zero-argument `.iter()` whose
  lowered receiver is Array/Vec. Fresh public/CLI/module/LLVM probes confirm zero,
  no-op, dropped-call, and legacy numeric-loop behavior. Ordinary array `.iter()`
  and `.iter().iter()` are value-preserving controls, while semantic receiver type
  alone does not predict lowering capability. A shared pre-IR capability/provenance
  contract is required; blanket rejection or a name/type exception is ineligible.
- Division by zero: immediate/computed integer forms panic in host constant folding;
  float literals fold to positive infinity; variable, unary, mixed, parameter, and
  closure forms emit `fdiv`; zero-stub parents can suppress division entirely. A
  15-case, 45-outcome matrix confirms that one syntax does not denote one current
  failure mechanism. Integer exception, constant evaluation, runtime, and IEEE
  policy must be decided separately.
- Match: the parser preserves one `Expression::Match` family with a scrutinee and
  every arm. Active preflight already visits the scrutinee and arm bodies; both
  inference paths invent `Int`; both IR paths replace the whole expression with zero
  without visiting children. No value-preserving source Match route or active Match
  lowering was found. Twenty-three cases produced 69 public/check/build outcomes:
  20 ordinary forms falsely succeeded with zero or dropped evaluation and three
  established child diagnostics retained precedence. Root Match emitted an empty
  `main`; direct modules and closure-body lowering were also false successes.
- Adjacent composites: StructLiteral, EnumVariant, Borrow, and Deref also fabricate
  zero or drop children, but selecting them intersects aggregate or ownership policy.
  Closure lowering preserves real body behavior and is not eligible for blanket
  rejection.
- Selection: Match is the only compared one-node family with no active
  value-preserving route, no required type/layout/arithmetic/ownership policy, and a
  complete existing recursive preflight location. A fail-closed slice can retain
  parser/AST/pattern representation and reject after child traversal in one
  production file.
- Verification: focused active Match parser test passes; numeric comparison,
  array/index/iterator, and numeric function controls pass. One auditor independently
  reran `./tools/test.sh` on exact clean `9fc7d0e` with exit zero. All three auditors
  ended at the exact SHA with clean worktrees and removed external probes.
- Status: complete; select Match for `CORE-008` preregistration. String comparison,
  MethodCall capability, division semantics, aggregates, and ownership remain
  separate open tasks.

## CORE-008 — Reject unsupported Match value expressions before IR

- Problem: every active `Expression::Match` is assigned invented `Int` semantics;
  both IR expression paths replace the entire Match with integer zero without
  evaluating its scrutinee or any arm. Root Match can produce an empty LLVM `main`,
  and Match can suppress calls and even compiler-panicking expressions.
- Evidence: `AUDIT-013` at `9fc7d0e` exercised 23 Match programs across 69 public,
  CLI-check, and CLI-build outcomes. Twenty ordinary root, binding, return,
  discarded, nested, array, call, closure, module, string/Option, and hidden-division
  forms falsely succeeded with zero or dropped evaluation. Three Match forms with
  field, tuple, or void-valued children retained those established diagnostics.
- Priority: P0.
- Dependencies: `CORE-001B` through `CORE-007` accepted; `AUDIT-013` complete at
  `648662b`.
- Decision: retain `match` tokens, parser grammar, `Expression::Match`, MatchArm, and
  Pattern representation, but reject every Match value expression in a trusted
  parsed source body with exactly
  `Match expressions are not supported.` No Match is eligible for IR or artifact
  generation in this slice. Expression roots reached by normal analysis use active
  recursive semantic preflight. Because default trait method bodies are parsed but
  not semantically analyzed, they require a syntax-only statement/block traversal
  that funnels each contained expression root into that same preflight without
  activating name, parameter, return, trait, type, ownership, or pattern semantics.
- Diagnostic ordering: keep the existing Match preflight traversal unchanged:
  inspect the scrutinee first and then every arm body in source order; return the
  Match diagnostic only after those children pass. Existing tuple, field, and
  void-call diagnostics inside the Match therefore retain precedence. Otherwise
  the Match diagnostic wins before Match inference, including when an identifier,
  pattern, or name would later require resolution. Nested traversal reports the
  first unsupported node reached under the existing outer-node rules; tuple nodes
  still reject before their children. No source span is fabricated.
- Boundary: rejection is recursive at expression roots, bindings, discarded forms,
  explicit/tail returns, conditions, iterables, and closure bodies, and beneath
  arrays/repeats/indexing, calls and method arguments/receivers, struct fields, enum
  payloads, other Match scrutinees/arms, borrows/dereferences, unary/binary/logical
  forms, prints, and field receivers. Default trait bodies are traversed in source
  statement order and then through the optional tail expression; nested statement
  containers preserve their existing condition/iterable-before-body order. Parent
  forms and default trait methods are not thereby certified as executable.
- Syntax retained: the active parser test for `let result = match x { 1 => 10,
  2 => 20, _ => 0 };` must continue to produce one Match with three arms. Pattern
  parsing and AST construction remain available for future implementation, but
  pattern binding, type checking, guards, exhaustiveness, reachability, evaluation
  order, and result typing are not executable capabilities.
- Files allowed: `src/compiler/src/semantic_analyzer.rs`, one new focused
  `src/compiler/tests/unsupported_match_tests.rs`, `README.md`,
  `tutorials/04-data-structures.md`, `SPEC_IMPLEMENTATION_MATRIX.md`, explicit
  current-status notices in historical enum/Match task or demo documents, and
  affected project-control/capability/conformance documents.
- Files frozen: lexer, grammar, parser, AST, pattern representation, types, enum and
  struct registries/construction/layout/discriminants/payloads, name binding,
  exhaustiveness/reachability, match guards, ownership/evaluation/runtime semantics,
  string comparisons, method calls, division semantics, IR, code generator,
  optimizer, backends, arrays/indexing/iteration, numeric contracts/annotations,
  tuple/field/modulo boundaries, dormant mutable inference, direct constructed-AST
  behavior, and unrelated public stability claims.
- Positive tests: Match parser shape; ordinary numeric arithmetic and comparison;
  numeric function calls; if/while behavior; arrays, indexing, and zero-argument
  array `.iter()` including its existing LLVM markers; struct/enum declaration and
  construction syntax without Match execution; standalone strings; and all prior
  tuple/field/modulo/function/annotation/strict suites retain their accepted behavior.
- Negative tests: minimal literal Match; bound/undeclared/call/string/Option
  scrutinees; undeclared/call/multiple arm bodies; root, binding, discarded,
  explicit/tail return, binary, non-first array, call argument, closure body, nested
  Match, hidden `/0`, direct module, representative recursive parent positions, and
  direct/tail/nested Match placements in default trait method bodies. Public
  compilation must return the exact semantic `Err` without unwind. CLI `check`/
  `build` must exit nonzero without panic or requested artifact.
- Diagnostic-precedence tests: tuple scrutinee, field arm, and void-valued arm keep
  their exact accepted diagnostics; tuple containing Match keeps the tuple
  diagnostic; Match inside a field receiver is reached before the outer field
  diagnostic. These tests freeze traversal only, not Match evaluation semantics.
- Red checkpoint: on clean `648662b`, parser/positive/precedence controls must pass.
  Ordinary Match negatives must preserve false acceptance, fabricated zero, empty
  root CFG, dropped calls, or suppressed `/0` before production changes. No negative
  may rely on an unrelated parse error or compiler panic as its expected result.
- Regression risks: returning before child traversal would change accepted
  diagnostics; restructuring the arm could accidentally imply pattern semantics;
  binary/library compiler modules and both IR paths are duplicated; historical enum
  helpers can be mistaken for active Match lowering; direct callers can bypass
  semantics and still reach zero-stub IR. Calling full block or statement analysis
  for a default trait body would silently expand this task into unregistered trait,
  parameter, return, name, ownership, and type semantics.
- Stop conditions: any trusted value-preserving Match route is found; implementation
  requires another production file or pattern/name/type/exhaustiveness/layout/
  ownership/evaluation semantics; parser/AST/IR/backend changes are needed; active
  routes cannot be covered by the shared preflight; child diagnostic precedence
  changes; or parser, numeric, function, array/index/iterator, struct/enum syntax, or
  any prior accepted focused boundary regresses.
- Owner: one isolated tests/implementation owner; lead integrates and owns the
  diagnostic/compatibility decision.
- Status: accepted at exact clean `b74d91a` after the complete gate and two new
  non-owner approvals.
- Acceptance criteria: red evidence is preserved; focused parser, recursive,
  positive, negative, diagnostic-order, no-unwind, direct-module, CLI no-panic and
  no-artifact tests pass; prior focused suites and `./tools/test.sh` pass; current
  public Match claims are corrected without deleting historical evidence; and two
  non-owner reviewers approve the exact clean documented integration SHA.
- Verification commands: focused `cargo test --test unsupported_match_tests`, prior
  applicable focused suites, `cargo fmt --all -- --check`, and required
  `./tools/test.sh`.
- Red checkpoint: owner commit `17e17c2`, integrated as `851731c`, adds exactly one
  598-line focused test file. Nine aggregated tests produce the preregistered
  5-pass/4-fail split: parser, adjacent capability, array-iterator CLI, and four
  child-precedence controls pass; ordinary public, recursive-parent public, root
  CLI, and direct-module CLI fail only because Match is not yet rejected.
- Red evidence: all 21 ordinary public forms falsely compile, including fabricated
  zero, empty root `main`, dropped scrutinee/arm calls, and two suppressed `/0`
  forms. Twelve of 15 recursive parents falsely compile with zero/drop behavior;
  field receiver and if/while conditions return their current outer diagnostics.
  Root and module CLI check/build exit zero, builds create requested artifacts, and
  artifacts preserve zero/drop evidence. No case fails parsing or unwinds. The
  active Match parser test passes 1/1; prior field/modulo/tuple controls pass 38/38;
  formatting passes. The full gate remains intentionally deferred until production.
- Candidate result: owner production commit `aed4d0e` is integrated as `c826294`.
  It adds exactly one error return after the existing Match scrutinee/arm preflight
  traversal. Both Match inference stubs and all parser/AST/type/IR/backend files are
  unchanged. The owner and lead independently pass the complete 90/90 focused
  matrix plus formatting and `cargo check --all-targets`.
- Public candidate documentation: README and the data-structures tutorial now state
  that Match is parsed but not executable and give the exact diagnostic; the matrix
  records negative/diagnostic coverage and no typed-IR/backend/execution support.
  Two historical enum/Match design summaries retain their content under explicit
  current-capability notices rather than serving as active implementation evidence.
- Initial full gate: exact clean documented candidate `08e7c2c` passed 112 library,
  119 binary, 11 fatal-parser, 59 frontend, 13 function-contract, 18 annotation,
  12 strict-lexing, 8 field, 9 Match, 14 modulo, and 16 tuple tests; 38 Phase 5
  tests remained intentionally ignored. Formatting, documentation, and Clippy gates
  passed.
- Initial review result: **REJECT** exact `08e7c2c`. Structural reviewer A found
  that a Match inside a parsed default `TraitMethod.body` succeeds through
  `compile_program`, `aero check`, and `aero build`, and build writes LLVM because
  `Statement::TraitDef` registers names without visiting default bodies. Its 33-route
  matrix passed 32 routes and found this sole parsed expression-container escape;
  7/7 precedence probes passed. Reviewer B's independent 41-route matrix approved
  the routes it exercised but did not cover trait defaults, so that approval cannot
  overcome the counterexample. No production acceptance exists.
- Corrective checkpoint: freeze a syntax-only block/statement walk for default trait
  bodies, add public/CLI/no-artifact and child-precedence regressions that fail on
  `c826294`, then change only `semantic_analyzer.rs`. Do not call `analyze_block` or
  `analyze_statement` for default bodies. The full gate and two new exact-SHA
  non-owner approvals were required after correction and are recorded below.
- Corrective red result: owner tests commit `58bb732`, integrated as `ad5e24d`, adds
  six aggregated tests without changing the original nine. On rejected production,
  the suite is exactly 11 pass / 4 fail: eight public default-body placements are
  falsely accepted; tuple/field/void child diagnostics are skipped; root and direct-
  module check/build exit zero and build requested artifacts. Parser retention, an
  unresolved-name default-body positive, and a required-signature positive pass; no
  source fails parsing, unwinds, or panics.
- Corrective production result: owner `a3f4f29`, integrated as `a12f38e`, changes
  only `semantic_analyzer.rs`. Dedicated syntax-only block/statement helpers visit
  expression roots in the frozen order and are called only for `Some(Block)` default
  trait bodies. Full semantic block/statement analysis remains inactive, required
  signatures remain untouched, and type-parameter scope cleanup occurs before an
  error returns. Owner and lead independently pass all 96 focused tests; formatting
  and the owner's all-target check pass. The subsequent complete gate is recorded
  below.
- Corrective complete gate: exact clean documented `b74d91a` passes 112 library,
  119 binary, 11 fatal-parser, 59 frontend, 13 function-contract, 18 annotation,
  12 strict-lexing, 8 field, 15 Match, 14 modulo, and 16 tuple tests; all 38 Phase 5
  tests remain intentionally ignored. Formatting, Clippy correctness, and doc tests
  pass.
- Corrective reviews: Reviewer A approves after an exhaustive 17-variant structural
  audit, 96/96 focused tests, 44/44 fresh public negatives, 12/12 traversal/precedence
  routes, 7/7 syntax-only positives, parser retention, and 10/10 CLI operations.
  Reviewer B independently approves after 70 fresh Match routes plus five precedence
  routes succeed across public/check/build for 225/225 rejection outcomes, three
  syntax-only default-body controls produce positive artifacts, and numeric/function/
  array/index/iterator controls remain green. Neither review found an unwind, panic,
  negative artifact, trusted false success, or additional parsed-container escape.
- Acceptance: exact clean integration SHA
  `b74d91adeda04688ec37598beebffad458538c39` satisfies `CORE-008`. Direct constructed-
  AST-to-IR bypass and actual Match/default-trait execution semantics remain open and
  outside this boundary.

## AUDIT-014 — Compare the next fabricated-value and panic boundaries

- Objective: compare StructLiteral, EnumVariant, Borrow, Deref, string comparison,
  MethodCall, and integer/float zero division on exact clean post-`CORE-008` closure
  without changing repository files or selecting aggregate layout, ownership,
  dispatch, string, arithmetic, runtime, or IEEE semantics.
- Audited commit: `a61172aeee3dff7ecc3a3595e4d098377f28991b`.
- StructLiteral: no active value-preserving source route exists. Both IR paths return
  scalar zero without visiting fields; names, declarations, field sets/order/
  duplicates/types, annotations, and children are not validated. A 61-route public
  aggregate matrix found 19/24 Struct cases falsely accepted; 12 CLI operations all
  exited zero and six builds wrote zero/drop artifacts. The existing child-first
  preflight arm is complete across ordinary and default-trait bodies.
- EnumVariant: equally non-executable, but `Some`/`None`/`Ok`/`Err` parser sugar and
  Option/Result inference already produce distinct child, payload, name, and modulo
  diagnostics. Twenty-seven of 36 Enum routes still falsely succeeded, but freezing
  all-enum versus custom-only policy is a prerequisite; it is not bundled with
  StructLiteral.
- Borrow/Deref: both IR paths return zero without visiting operands. Borrow also
  mutates shallow ownership state for exact direct-let shapes and seven active tests
  depend on those diagnostics; Deref has real reference/non-reference inference
  diagnostics but no active semantic acceptance test. A 61-observation matrix ranks
  Deref-only ahead of Borrow, but both intersect ownership/reference policy and are
  deferred behind the cleaner Struct boundary.
- String comparison: all six operators remain accepted by semantics and can unwind
  in IR; equality/inequality versus ordering and authoritative operand/runtime policy
  are unresolved. Blanket comparison rejection would regress numeric execution.
- MethodCall: the sole proven value-preserving route is exactly zero-argument
  `.iter()` on an inferred Array/Vec receiver. Other known/unknown/wrong-arity forms
  fabricate zero and drop arguments, but a typed capability discriminator must also
  cover syntax-only default trait bodies before a trustworthy rejection slice.
- Division: immediate/computed integer zero can panic while variable, unary, float,
  mixed, closure, default-body, and dropped-argument forms diverge or emit artifacts.
  Integer/runtime/overflow/promotion and floating IEEE policy remain prerequisites.
- Selection: StructLiteral alone is the only compared family with no active value-
  preserving route, no special built-in exception, no established semantic error to
  replace after successful child preflight, and one complete existing preflight arm.
  Active construction-positive controls must be explicitly reclassified as parser-
  retention plus declaration-only controls rather than silently broken.
- Status: complete; preregister `CORE-009`. EnumVariant, Borrow/Deref, MethodCall,
  string comparison, and division remain separate open tasks.

## CORE-009 — Reject unsupported struct construction expressions before IR

- Problem: every `Expression::StructLiteral` is assigned `Ty::Struct(name)` without
  validating that the struct exists or that fields exist, are unique, complete, in
  range, or type-correct. Both IR paths replace the entire construction with integer
  zero without evaluating any field, so calls and invalid expressions disappear and
  aggregate values become fabricated scalars.
- Evidence: `AUDIT-014` at `a61172a` found 19/24 StructLiteral routes falsely accepted
  across root, binding, return, discarded, call, array, closure, default-trait,
  nested, unknown-name, missing/extra/duplicate/wrong-field, and dropped-child cases.
  Root/module check and build exit zero and create artifacts containing zero but no
  field calls, layout, type, or aggregate operations. Direct aggregate arithmetic
  can fold fabricated zero to one or panic after the invented `Int` type.
- Priority: P0.
- Dependencies: `CORE-001B` through accepted `CORE-008`; `AUDIT-014` complete.
- Decision: retain struct declarations, construction grammar, parser, AST, and field
  syntax, but reject every StructLiteral reached through trusted parsed-source
  preflight with exactly `Struct construction expressions are not supported.` No
  StructLiteral is eligible for inference, IR, or artifact generation in this slice.
- Diagnostic ordering: visit field values in source order using the existing
  preflight traversal, then return the Struct diagnostic. Established tuple, field,
  Match, and void-as-value child diagnostics retain precedence. A nested Struct
  reports the first inner Struct reached in field order. Modulo, undeclared names,
  field declaration/name/type validation, and other inference-only errors are not
  activated under this slice; after syntax preflight they yield the outer Struct
  diagnostic. No source span is fabricated.
- Boundary: rejection is recursive from ordinary expression roots, bindings,
  discarded/explicit/tail returns, conditions, iterables, closures, default trait
  bodies, nested functions/impls/traits, and every composite currently traversed by
  shared preflight, including EnumVariant payloads. Parent expressions and structs
  themselves are not thereby certified.
- Syntax retained: parsing `Point { x: 7, y: 9 }` must still produce one
  StructLiteral named `Point` with two ordered fields. Struct declarations and
  generic construction syntax remain parser-visible; struct layout, initialization,
  field validation, moves, ABI, IR, backend emission, and execution remain absent.
- Compatibility decision: supersede the adjacent-construction positives frozen by
  `CORE-007`/`CORE-008`. Rewrite them to assert StructLiteral parser retention and
  public declaration-only/numeric/array/string/enum controls. Rewrite active
  ownership/trait tests to use declared struct-typed parameters rather than runtime
  construction so they continue testing move and trait rules without claiming
  aggregate execution.
- Files allowed: `src/compiler/src/semantic_analyzer.rs`, one new
  `src/compiler/tests/unsupported_struct_literal_tests.rs`, targeted control-only
  edits in `src/compiler/tests/unsupported_field_access_tests.rs`,
  `src/compiler/tests/unsupported_match_tests.rs`, and
  `src/compiler/tests/frontend_tests.rs`, plus README, data-structures tutorial,
  specification matrix, historical status notices, and project-control documents.
- Files frozen: lexer, grammar, parser, AST, struct/enum registries and definitions,
  field lookup/assignment, generic resolution, ownership semantics, inference,
  types, IR, code generator, optimizer, backends, EnumVariant/Option/Result,
  Borrow/Deref, MethodCall, comparison/division, and every prior accepted boundary.
- Positive tests: StructLiteral parser shape; struct declarations without
  construction; generic annotation parsing; declared struct-typed parameter move
  and trait-bound controls; custom Enum and Option/Result construction retain their
  current separate behavior; numeric/function/control-flow/array/index/iterator/
  string controls; and every prior focused boundary.
- Negative tests: known/unknown construction; missing/extra/duplicate/wrong fields;
  root, binding, discarded, explicit/tail return, condition, call argument, non-first
  array, closure, nested Struct, EnumVariant payload, default trait and nested
  declaration containers, dropped field calls, direct module, and representative
  recursive parents. Public compilation must return the exact semantic error without
  unwind. CLI check/build must exit nonzero without panic or requested artifact.
- Diagnostic-precedence tests: tuple, field, Match, and known void-call field values
  retain their accepted diagnostics; nested Struct uses first-field/source order;
  modulo and undeclared-field children yield the Struct diagnostic because their
  inference remains intentionally unreachable.
- Red checkpoint: on clean `a61172a`, parser/declaration/enum/numeric/array and prior
  boundary controls must pass. Struct negatives must preserve false acceptance,
  fabricated zero, empty/drop behavior, omitted calls, and artifact creation before
  production changes. No negative may rely on parse failure or unwind.
- Regression risks: returning before fields changes child precedence; invoking field
  inference invents aggregate semantics; existing public-positive controls and trait
  tests can fail for the intended compatibility change unless reclassified; dormant
  struct IR/codegen helpers can be mistaken for active source lowering.
- Stop conditions: any trusted value-preserving StructLiteral route appears; the
  change needs another production file or declaration/field/type/layout/ownership/
  evaluation semantics; parser/AST/IR/backend changes are needed; default bodies or
  a parsed parent bypass preflight; child ordering changes; EnumVariant behavior or
  any prior accepted focused boundary regresses.
- Owner: one isolated tests/implementation owner; lead owns compatibility and exact
  diagnostic decisions. Two non-owner reviews are required after the complete gate.
- Status: accepted at exact reviewed candidate
  `daa024dbf10d1defe06d8ab200c2d21c0a9c1dc6`. Tests-only red checkpoint accepted at
  exact integration commit `1e76a0610ef778303548096ef634a5f02b678fe9`;
  production candidate integrated at
  `a8879310fe04a28b368437d1932e01972b7e9cee`; public truth and the complete gate
  pass at exact `3410f1f`; coordinated control corrections and a fresh complete
  gate pass at exact `daa024d`.
- Verification: focused Struct suite plus modified control suites, all prior focused
  boundaries, formatting/all-target check, required `./tools/test.sh`, then exact-SHA
  structural and black-box reviews.

Tests-only red evidence: owner commit `042b905` is integrated as `1e76a06` and
changes only the four authorized test files. The aggregate suite covers more than
41 parser, public, recursive, default/nested, precedence, and CLI routes across nine
tests. Independent lead execution is exactly 3 pass / 6 expected fail: ordinary
StructLiteral forms are accepted and lowered as zero/drop artifacts; recursive and
container forms either accept or surface the current outer diagnostic; CLI root and
direct-module check/build exit zero, and both builds create the requested LLVM.
Parser shape, declaration/Enum/Option/numeric/function/array/index/iterator controls,
and established tuple/field/Match/void child precedence pass. No negative depends on
parse failure or unwind. Reclassified controls pass 59 frontend, 8 field, and 15
Match tests; formatting and `git diff --check` pass.

Production evidence: owner commit `bf6a7ef` is integrated as `a887931` and adds
exactly one return to `semantic_analyzer.rs` after existing StructLiteral field
preflight. The owner passes 9 Struct, 59 frontend, 8 field, 15 Match, 16 tuple, 14
modulo, 13 function-contract, 18 numeric-annotation, and 12 strict-lexing tests,
formatting, all-target check, and `./tools/test.sh`. The lead independently passes
the same 164 focused tests on the integration candidate. Struct children retain
source order and the established tuple/field/Match/void precedence; inference-only
modulo/name children reach the outer Struct diagnostic. No parser, AST, inference,
types, IR, backend, EnumVariant, ownership, method, comparison, division, or prior
boundary file changed.

Acceptance evidence: the lead reran the complete repository gate on exact clean
`daa024d`; all suites, formatting, Clippy correctness, all-target compilation, and
doc tests pass. Reviewer A verified the five-control-document correction delta,
the complete status/resumption model, production/public-truth immutability,
formatting, and 9/9 Struct tests. Reviewer B independently verified the same exact
delta and state model, production/public-truth immutability, and 9/9 Struct tests.
Both reviewers approve exact `daa024d` with no P0-P3 findings.

## AUDIT-015 — Trace founding framework into current engineering control

- Objective: inspect both tracked founding PDFs completely to the extent preserved,
  distinguish project vision from implementation evidence, and reconcile their
  roadmap and killer-application direction with the audited repository.
- Sources: the complete nine-page `__Aero___ A High-Performance, Ergonomic
  Programming Language.pdf` and the single-page `Aero Programming Language Framework
  - Claude.pdf`.
- Artifact limitation: the Claude PDF is a truncated browser print. It visibly ends
  during memory-safety measurement guidance; extractable off-page text also begins
  mid-conversation. No missing continuation or final recommendations are inferred.
- Findings: the primary paper freezes the high-level goals of explicit simplicity,
  zero-cost native performance, ownership safety, compositional data/traits,
  concurrency, tooling, open governance, LLVM bootstrap, eventual self-hosting, and
  Design -> Minimal Prototype -> Self-Host -> Stabilize -> Optimize. The strategy
  capture selects AI/ML infrastructure as the lead adoption wedge and calls for
  performance, compiler-quality, safety, resource, and reproducibility measurement.
- Repository comparison: the current compiler is a Minimal Prototype under
  correctness recovery. Scalar/frontend slices and CPU LLVM are partial; composite
  values, ownership, typed IR, concurrency, tooling convergence, and device execution
  do not meet the founding outcomes. No audited public Aero runtime/device claim
  currently passes the benchmark protocol.
- Control result: `FRAMEWORK_ALIGNMENT.md` records the authority hierarchy, gap
  matrix, quality scorecard, and Aero-native AI/ML flagship progression. The public
  README links the founding sources and removes the v1.0 heading implication.
  `Roadmap.md` is replaced with evidence-gated milestones aligned to the founding
  progression. `LANGUAGE_VISION.md`, `PROJECT_STATE.md`, and `DECISION_LOG.md` carry
  the traceability decision.
- Boundary consequence: no code, language semantics, backend status, benchmark
  claim, or `CORE-009` decision changes. The audit prevents vision prose from being
  reused as completion evidence.
- Status: complete; documentation verification passed and founding-framework
  alignment was published to draft PR #4 at exact `fba121f`.

## AUDIT-016 — Rank the post-StructLiteral compiler-integrity boundary

- Objective: re-audit the open semantic/IR/backend false-success families after
  `CORE-009`, reproduce the highest-severity active failures, rank bounded next
  slices, and identify every decision required before implementation.
- Audit base: production exact `3410f1f`; the compiler and tests are byte-identical
  through published clean head `555fea2`, so the evidence remains current.
- Ranked candidates: (1) fallible typed scalar IR admission and verification,
  (2) canonical library/CLI pipeline, (3) MethodCall fail-closed while preserving
  zero-argument Array/Vec `.iter()`, (4) custom EnumVariant fail-closed while
  preserving Option/Result policy, and (5) Deref-only fail-closed.
- Fresh evidence: string comparison and constant integer `1 / 0` unwind in IR
  generation; a stored comparison result emits `icmp ... i1` followed by an invalid
  `store double %reg...` and is reported successful; ordinary methods, Deref,
  custom enums, `Some`, and `Ok` fabricate zero/drop behavior; codegen silently
  ignores unmatched instructions; no in-process IR or LLVM verifier exists.
- Pipeline evidence: library, CLI build/run, profiler, and conformance call the
  infallible IR APIs through duplicated orchestration. A missing direct module fails
  `check` but build can continue and write an artifact; usage and unknown-command
  status defects remain separate. These pipeline/status defects are not silently
  folded into the selected IR contract.
- Selection: `CORE-010` is the typed scalar admission/verification boundary. It has
  greater risk and implementation breadth than the other candidates, but removes a
  class of panics and invalid artifacts while directly advancing founding-roadmap
  Milestone 1. MethodCall, EnumVariant, Deref, ownership, and aggregate semantics
  are not implemented by this choice; any currently fabricated form reaching the
  checked IR boundary must be rejected explicitly.
- Prerequisite audits: three independent read-only audits traced primitive/storage
  representation, LLVM verifier policy, and every public/trusted caller. Their
  decisions are frozen in `CORE-010` and `DEC-015` below before tests or production.
- Status: complete; read-only audit, probes, ranking, and prerequisite decisions
  recorded. No compiler file changed.

## CORE-010 — Add checked typed scalar IR admission and verification

- Problem: `Value::Reg` and core instructions carry no type; stack slots are always
  lowered as `double`; Boolean comparisons produce `i1` values that can be stored as
  `double`; function types are reconstructed from strings; IR/codegen helpers panic
  or silently drop unsupported inputs. Trusted library/CLI/profile paths can unwind,
  emit invalid LLVM, or report success without a valid artifact contract.
- Priority: P0.
- Dependencies: accepted `CORE-009`; `AUDIT-016` and three prerequisite audits
  complete at clean published head `555fea2`.
- Hypothesis: explicit logical value/place/signature types, mandatory in-process IR
  verification, checked additive APIs, exhaustive codegen, and final LLVM module
  verification can stop panics and invalid artifacts without inventing aggregates,
  ownership, dispatch, or new arithmetic semantics.
- Logical types: the admitted set is `Int`, `Float`, `Bool`, `Void`, restricted
  print-only string immediates, existing local fixed numeric arrays, and restricted
  compile-time callable aliases for non-capturing scalar closures. Results and
  places use distinct identifiers. Every result, slot/pointee, array element, call
  parameter/result, branch condition, and return has an explicit logical type.
  `Void` is never an operand; void calls have no result and void functions return no
  value. The synthesized process entry remains `i32 @main()` returning zero.
- Compatibility representation: active `int`/`i32` remains one logical signed
  integer family with the accepted `i32` function ABI; `float`/`f64` remains `double`;
  `bool` is `i1`. Existing local numeric `double` lowering and fixed numeric-array
  storage remain compatibility details in this slice because integer overflow,
  division, and full array semantics are not specified. Boolean results/slots must
  use `i1` consistently. Unknown types may never default to `double`. Integer
  literals outside the admitted `i32` range fail admission rather than truncate.
- Arithmetic boundary: preserve accepted exact numeric contracts and mixed
  Int-to-Float promotion. Checked constant folding returns errors rather than
  panicking; constant integer division by zero is rejected. This does not certify
  dynamic integer overflow/division or IEEE/runtime behavior and does not authorize
  a physical all-`i32` local migration. If correct typing requires deciding those
  semantics, stop and split an RFC-backed arithmetic task. If in-range literal
  operands would fold outside `i32`, the checked path must leave the existing
  logical operation unfurled rather than materialize, truncate, wrap, or reject a
  constant result; its runtime overflow behavior remains explicitly uncertified.
- Arrays/strings: admit only existing local fixed numeric array/index/`.iter()`
  controls as opaque legacy storage with explicit logical element metadata. Do not
  broaden mixed/empty inference, bounds, parameter/return ABI, or Boolean arrays.
  String literals and immutable bindings that remain compile-time aliases are
  eligible only for established print/println handling; comparison, memory-slot
  storage, calls, returns, and other string operations fail with a checked error.
- Unsupported source forms: the checked path must never convert MethodCall,
  EnumVariant/Option/Result, Deref/Borrow, tuple/struct/field/Match, or another
  unadmitted form to a scalar. Existing shared semantic diagnostics keep precedence;
  exact zero-argument Array/Vec `.iter()` remains the sole admitted MethodCall.
  This is a generic IR safety boundary, not implementation of those language forms.
- Closure boundary: preserve only the established non-capturing closures whose
  parameters and expression result are admitted Int/Float/Bool scalars. A closure
  binding is a compile-time callable symbol plus explicit scalar signature; it has
  no runtime operand, place, allocation, store, or fabricated numeric ID. Preserve
  lexical callable shadowing. Captures, escape, assignment, passing/returning a
  closure value, composite/string signatures, and closure-object ABI remain
  unsupported and fail admission.
- Compatibility reclassification: supersede the `CORE-009` control requirement that
  custom Enum, `Some`, and `Ok` construction continue compiling through fabricated
  zero. Their syntax/AST shape and declarations remain positive parser/frontend
  controls, while every runtime construction becomes an exact checked-IR negative.
  Reclassify ordinary MethodCall, Deref/Borrow, and any other zero/drop fallback the
  same way while preserving Array/Vec `.iter()` and existing semantic-preflight
  diagnostics. This change is authorized because the prior success was never
  value-preserving execution evidence; no enum/ownership/method semantics are added.
- Internal verifier: a pure-Rust verifier is mandatory before every checked LLVM
  lowering. It checks definitions/uses and required dominance, place/pointee and
  load/store agreement, operator/result types, calls, returns, labels/targets,
  Boolean branches, exactly one final terminator with no following instruction in
  every represented block whether reachable or not, supported GEP forms, and
  rejects every unsupported instruction explicitly. Unreachable-block removal,
  SSA/phi construction, and general CFG redesign remain outside this slice.
- LLVM verifier: verify final LLVM text after graph transformation and retargeting,
  before caching, writing, or native tools. Standalone `graph-opt` and `quantize`
  accept arbitrary LLVM, so their input and final transformed output both require
  external verification; they never use `InternalOnly`. This adds verification at
  their command wrappers without changing either transform. Use LLVM 22
  `opt -passes=verify
  -disable-output -` first and `llvm-as -o - -` as the fallback; `clang`, `llc`, and
  `llc -verify-machineinstrs` are downstream checks, not verifier substitutes.
  Prefer explicit `AERO_LLVM_OPT`/`AERO_LLVM_AS` paths, then versioned tools, then
  unversioned tools only when their parsed major is 22. Record path and version.
- Unavailable-tool policy: in-process verification is never optional. Text-only
  build may use `PreferExternal`: actual tool absence yields a visible
  `InternalOnly` warning/status, while any discovered verifier rejection, launch
  error, timeout, or wrong version fails with no artifact. `Required` applies to
  CI/evidence gates, `run`, and object/executable paths; absence also fails. The
  library's existing string-returning `compile_program` promises in-process
  verification only and does not claim external LLVM verification. No LLVM Rust
  crate or native link dependency is added.
- Cache policy: the existing cache stores final LLVM text only and carries no typed-
  IR/internal-verification provenance. A cache hit may be published only after an
  available external LLVM verifier accepts it. Under `PreferExternal`, actual tool
  absence bypasses the cache and rebuilds through checked IR; only that fresh path
  may return visible `InternalOnly`. A found verifier failure is fatal. Do not add
  cache provenance or change cache storage in this slice.
- Mode selection: `run`, object/executable paths, `graph-opt`, and `quantize` force
  `Required`. Text `build` uses `Required` when either
  `--require-llvm-verifier` is present or `AERO_REQUIRE_LLVM_VERIFIER=1|true`, and
  otherwise uses `PreferExternal`; a forced command/flag cannot be downgraded by
  environment. CI sets the environment variable explicitly. If `AERO_LLVM_OPT` or
  `AERO_LLVM_AS` is set, that exact override is authoritative: absence, wrong major,
  launch failure, timeout, or rejection is fatal and does not fall through to PATH.
  Without overrides, discovery tries versioned `opt-22`, then versioned
  `llvm-as-22`, then compatible unversioned opt/llvm-as; a found/rejecting opt never
  falls back, while actual opt absence may proceed to llvm-as.
- Profiler/conformance policy: profiler uses `PreferExternal`; absence is recorded
  visibly as `InternalOnly` in the printed profile and trace metadata, while a found
  verifier failure returns an error before trace publication. Conformance performs
  checked IR generation/internal verification only and remains external-tool-
  independent. A checked IR failure becomes a failed conformance result; the CLI
  still writes the complete report when requested and then exits nonzero whenever a
  case or mechanized check failed.
- `check` contract: evolve `aero check` from semantic-only to frontend validity plus
  typed-IR admission/internal verification, still without LLVM emission, external
  tools, or artifacts. It does not promise final LLVM/backend representability.
  CLI help and capability documentation must say this exactly. CLI `test`, missing-
  module continuation, unknown-command status, and pipeline consolidation remain
  separately tracked.
- API compatibility: add structured `IrGenerationError`, `IrVerificationError`,
  `CodeGenerationError`, and `LlvmVerificationError` types plus checked
  `IrGenerator::try_generate_ir` and method/free `try_generate_code`. Keep
  `compile_program(Result<String, String>)` source-compatible and migrate every
  trusted library, CLI build/run, profiler, and conformance caller to checked APIs.
  Retain current `generate_ir`/`generate_code` entry points only as documented,
  deprecated unchecked compatibility shims excluded from trusted-path claims until
  a major break. Checked APIs may never return empty/partial IR or embed errors as
  output.
- Checked API verification: method/free `try_generate_code` accepts the existing raw
  private IR shape and always invokes the internal verifier before emission, even
  when `try_generate_ir` already verified its output. A pipeline error enum retains
  `IrVerificationError` as a distinct variant so callers render the IR Verification
  phase rather than relabeling it Code Generation. Malformed-IR tests call this
  checked boundary directly. No public verified-token or private IR exposure is
  introduced in this slice.
- Legacy API behavior: deprecated `generate_ir -> HashMap` and method/free
  `generate_code -> String` retain a separate unchecked legacy implementation and
  its historical panic/silent behavior for source compatibility; they are not
  implemented by unwrapping the checked APIs and are excluded from every trusted
  compiler/correctness claim. They may preserve historical partial/silent output but
  may not gain new adapter fallbacks, newly implicit panics, or error-text output.
  Removal, rather than semantic alteration, occurs at a major compatibility boundary.
- Diagnostic contract: preserve `Lex error:`, `Parse error:`, and
  `Semantic Analysis Error:`. Add stable `IR Generation Error:`,
  `IR Verification Error:`, `Code Generation Error:`, and
  `LLVM Verification Error:` phase prefixes. Build/run retain their outer `error:`
  rendering. `check` preserves its existing raw semantic diagnostic exactly and
  uses the new phase prefix only for IR Generation/Verification failures. Profiler
  preserves `Semantic analysis failed:` and uses the new labels for later phases.
  No production `catch_unwind` adapter is allowed.
- Failure/artifact ordering: IR Generation, IR Verification, and Code Generation
  errors return before graph lowering or retargeting. On source build/run/profile
  routes, LLVM Verification runs on the final transformed/retargeted text and
  returns before cache publication, filesystem writes, native tools, or trace
  output. Standalone graph-opt/quantize additionally verify arbitrary input before
  transformation and final output before publication.
  Run cleanup removes its temporary artifact directory. Cache hits are reverified.
  Tests use fresh paths and prove nonzero status, no panic/unwind text, no requested
  artifact, and no empty run directory. The sole requested-failure-report exception
  is conformance: it deliberately writes a complete report containing the failed
  case/check and then exits nonzero.
- Red tests: public no-unwind errors for string comparison and constant integer
  division by zero; the three accepted-but-invalid Boolean slot/return/call shapes;
  i32 range rejection; malformed typed IR covering duplicate/use-before-definition,
  bad store/load, non-Bool branch, wrong call/result/return, void operand, labels,
  reachable and unreachable terminators, GEP, and unsupported instructions; no
  silent codegen wildcard; in-range operands whose constant result exceeds `i32`
  remain an unfurled logical operation without panic or fabricated constant;
  admitted closure bindings produce callable aliases/signatures with no runtime
  allocation/store/ID, while capture/escape/composite closure forms fail admission.
- External red matrix: injected verifier accept/reject/missing/launch-error/timeout/
  wrong-version outcomes; `check` identical with and without LLVM tools; text build
  exposes `InternalOnly` only for absence; strict/run failure creates no artifact;
  explicit override and flag/environment/forced-command precedence; versioned and
  unversioned discovery; absent opt plus accepted llvm-as; rejecting opt with no
  fallback; verified source-build bytes are post-graph and post-retarget; rejected
  output is neither cached nor written; cache hits are reverified; graph-opt and
  quantize reject invalid input/final output without artifacts; cache hit plus
  missing verifier bypasses and rebuilds checked IR; valid graph-opt and quantize
  controls prove input verification, final-output verification, then publication;
  profiler missing verifier succeeds with visible `InternalOnly` in stdout and trace
  metadata, while found rejection is nonzero with no trace; injected conformance
  checked-IR failure writes the complete requested failure report then exits nonzero;
  pinned LLVM 22 CI accepts the positive corpus and rejects a known-invalid fixture,
  then existing llc/clang/runtime checks remain green.
- Positive controls: Int/Float/Bool alloc-store-load, comparison and direct/loaded
  Boolean branches, numeric/void calls and returns, mixed promotion, numeric
  closures, fixed numeric arrays/index/`.iter()` loops, print-only string bindings,
  all prior accepted fail-closed diagnostics, reclassified parser/declaration
  controls for unsupported forms, four CPU examples, formatting, all-targets, and
  full gate.
- Files allowed: `src/compiler/src/ir.rs`, one new IR verifier module, one new LLVM
  verifier adapter, `ir_generator.rs`, `code_generator.rs`, minimal propagation in
  `lib.rs`, `main.rs`, `profiler.rs`, and `conformance.rs`; focused tests; LLVM CI
  workflows; verification-only command-wrapper edits for `graph-opt`/`quantize`;
  targeted reclassification edits in existing unsupported-form control tests;
  public capability/help and project-control documents.
- Files frozen: lexer, parser, AST syntax, semantic language rules except required
  phase routing, ownership, aggregate/enum/method implementations, graph/backend
  transform algorithms, `Cargo.toml`/`Cargo.lock`, registry, benchmark claims, and
  releases.
- Risks: typed metadata can diverge from legacy physical numeric lowering; verifier
  activation can expose a broad invalid-LLVM corpus; graph transforms/cache can
  bypass checks; deprecation can preserve an unsafe direct caller; CFG validation
  can accidentally redesign control flow; tool discovery/version/timeout/temp-file
  handling can become platform-dependent.
- Stop conditions: a second trusted untyped path remains; accepted positive controls
  require integer/array/ownership/aggregate/dispatch semantics not frozen here;
  preserving admitted closures requires a runtime closure object/capture ABI;
  LLVM 22 exposes broad redesign rather than bounded repairs; any cache/transform/
  output route bypasses verification; API status cannot be represented honestly;
  `check` becomes external-tool-dependent; or the change unexpectedly crosses a
  compiler phase beyond this preregistered IR/backend/pipeline propagation.
- Owner: one lead-owned vertical slice with tests-first checkpoints. Independent
  representation and backend reviewers must approve the exact clean candidate.
- Red checkpoint: exact public commit
  `26560a45905015b7891ddebeb749d0097c05cbaa`, exact staged diff hash
  `c01fc2365eb5b415c022be997062e4605812b62b`. Three independent reviewers approve
  the exact diff with no P0-P3 findings. Local evidence is 1 pass / 7 intentional
  failures for typed admission and 3 pass / 9 intentional failures for the external
  LLVM CLI matrix; checked public/private targets stop only on preregistered missing
  checked APIs and injected seams. Reclassified parser/declaration controls pass.
- CI red evidence: both compiler workflows install pinned LLVM 22 and prove the
  known-invalid fixture is rejected. Rust stable/nightly pass LLVM verification and
  execution of the four positive CPU examples before Cargo reaches the deliberate
  checked-API compilation failures. No setup, parser, unrelated semantic, or positive
  LLVM-corpus failure invalidates the checkpoint.
- Status: tests/CI-only red checkpoint accepted. Production implementation is now
  authorized within the frozen files, semantics, ordering, compatibility, and stop
  conditions above; no production candidate is accepted yet.
