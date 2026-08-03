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
  Retain current `generate_ir`/`generate_code` entry points only as documented
  unchecked compatibility shims excluded from trusted-path claims until a major
  break. The method/free `generate_code` shims are marked deprecated; public raw
  `IrGenerator::generate_ir` is not yet marked deprecated and remains a separately
  recorded residual API risk. Checked APIs may never return empty/partial IR or
  embed errors as output.
- Checked API verification: method/free `try_generate_code` accepts the existing raw
  private IR shape and always invokes the internal verifier before emission, even
  when `try_generate_ir` already verified its output. A pipeline error enum retains
  `IrVerificationError` as a distinct variant so callers render the IR Verification
  phase rather than relabeling it Code Generation. Malformed-IR tests call this
  checked boundary directly. No public verified-token or private IR exposure is
  introduced in this slice.
- Legacy API behavior: public unchecked `generate_ir -> HashMap` retains historical
  panic/silent behavior when selected directly. `try_generate_ir` may reuse its
  internal engine only after checked preflight/mode activation and must verify the
  result before returning it. Deprecated method/free `generate_code -> String`
  retains a separate unchecked legacy implementation. Trusted callers select only
  the checked boundaries and never consume raw unchecked output. The unchecked APIs
  may preserve historical partial/silent direct behavior but may not gain adapter
  fallbacks, newly implicit panics, or error-text output. Deprecating/restricting raw
  `generate_ir` and removing the unchecked APIs, rather than altering their direct
  semantics, occurs at a major compatibility boundary.
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
  public capability/help and project-control documents. Backend review amendment:
  `src/compiler/Cargo.toml` and `Cargo.lock` may add only target-specific `libc` and
  `windows-sys` process-containment support required to enforce the frozen verifier
  process-tree deadline; this does not authorize an LLVM binding or other dependency.
- Files frozen: lexer, parser, AST syntax, semantic language rules except required
  phase routing, ownership, aggregate/enum/method implementations, graph/backend
  transform algorithms, all Cargo dependency changes other than the bounded
  process-containment amendment above, registry, benchmark claims, and releases.
- Risks: typed metadata can diverge from legacy physical numeric lowering; verifier
  activation can expose a broad invalid-LLVM corpus; graph transforms/cache can
  bypass checks; deprecation can preserve an unsafe direct caller; CFG validation
  can accidentally redesign control flow; tool discovery/version/timeout/temp-file
  handling can become platform-dependent. Windows containment must create the child
  suspended, attach its job before its first instruction, and resume only after
  assignment; Unix containment must create and terminate a dedicated process group.
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
- Production implementation evidence: the bounded implementation now carries additive
  checked IR/type metadata, fallible admission, mandatory internal verification,
  exhaustive checked code generation, the LLVM 22 verifier adapter, and trusted-
  path propagation through library, CLI, profiler, and conformance callers. The
  focused checked-IR and LLVM-verifier contracts pass, as do the existing
  compatibility controls and the complete `./tools/test.sh` repository gate.
- Review amendments: independent exact review rejected a direct-child-only timeout
  because descendants retained inherited handles beyond the deadline. The bounded
  correction authorizes the two target-specific process APIs above, contains Unix
  process groups, and creates Windows verifier wrappers suspended so job assignment
  precedes all child execution. An immediate-descendant regression must prove no
  marker survives timeout. The same review requires successful named conformance
  cases to pass checked IR rather than stopping after semantics.
- Status: accepted at public head
  `db349ef81f145ee571c053f73fb03c831cea719a`. Three independent reviewers approved
  exact implementation diff `9534765a46b130d215a1d1e869de234163bb0daf`
  and exact mixed-entry CI repair `d5f0fd3891da5cff75bd5306006e993ca4b4f301`
  with no P0-P3 findings. The complete local gate, Rust stable/nightly, both public
  compiler-test workflows, and all CodeQL jobs pass. The PR remains draft; this
  acceptance does not authorize merge, release, or broader capability claims.

## AUDIT-017 — Revalidate the post-CORE-010 module and pipeline boundary

- Objective: re-audit the highest-ranked open pipeline candidate after `CORE-010`,
  reproduce each trusted module-source route, and select the smallest phase boundary
  that prevents false success without claiming unimplemented namespace semantics.
- Audit base: clean published head
  `34cc0e088f1b579ebecfb4498c55cde2cb23aaad`; accepted compiler behavior remains
  exact `db349ef81f145ee571c053f73fb03c831cea719a`.
- Specification evidence: the formal language specification requires `mod x;` to
  resolve as `x.aero` or `x/mod.aero` and requires circular dependencies to be
  rejected. The README advertises multi-file projects, `use`, and `pub`, while the
  implementation only flattens direct module ASTs and does not enforce namespaces,
  imports, visibility, recursive paths, or a module graph.
- Fresh black-box evidence: for a valid root containing `mod absent;`, `build`
  exits zero and writes the requested LLVM artifact after printing a resolver error.
  `check` and `profile` both exit one and publish no artifact. A nonexistent build
  input, no arguments, and an unknown command also exit zero, but those status-only
  defects are separate because they do not cross the module-source boundary or
  publish compiler output.
- Cache evidence: `build` consults its final-LLVM cache before lexing, parsing, or
  module resolution. Its key contains only root source plus target/GPU selection.
  A cache hit can therefore bypass a missing module, and module source changes do
  not change cache identity. The cache is process-local today, but the private
  reusable optimizer path and its tests make this an active correctness contract,
  not a hypothetical persistent-cache concern.
- Pipeline evidence: `build`, `check`/`test`, `profile`, and `doc` independently
  repeat direct-module resolution and module lex/parse logic. The public
  `compile_program(source, options)` has no entry path and silently analyzes a
  `ModDecl` without resolving it. The binary and library still compile distinct
  module trees, and `CompilerOptions` remains ignored; resolving either broader
  architecture issue would cross the bounded source-collection slice.
- Selection: `CORE-011` owns canonical fail-closed direct-module source collection,
  module-aware cache identity, and source-only library rejection. It does not
  implement namespaces, `use`, `pub`, recursive module paths, cycle-graph semantics,
  execution by `aero test`, general CLI status cleanup, or full library/CLI pipeline
  consolidation.
- Status: complete; current behavior, specification delta, cache hazard, caller
  inventory, scope, and stop conditions are recorded before tests or production.

## CORE-011 — Make direct-module source collection canonical and fail closed

- Problem: trusted compiler routes disagree when a declared source file is missing;
  `build` can report success and publish LLVM, cached LLVM can bypass module state,
  and the source-only library API silently ignores the unresolved declaration.
- Priority: P0 because a declared dependency can be absent while a trusted build
  publishes an artifact and reports success.
- Dependencies: accepted `CORE-010`; complete `AUDIT-017`; `DEC-016` below freezes
  the bounded compatibility and failure contract.
- Hypothesis: one shared direct-module collection boundary, invoked before cache or
  semantic/IR admission, can make existing routes agree without choosing namespace,
  visibility, recursive layout, or module-ABI semantics.
- Direct resolution contract: for each root-level `mod x;` in source order, resolve
  first `<root-dir>/x.aero`, then `<root-dir>/x/mod.aero`; read, strictly lex with
  the resolved filename, and fatally parse the whole source. Append the accepted
  direct-module ASTs in declaration/source order exactly as current compatible
  compilation does. Resolution, read, lex, or parse failure returns before
  semantics, IR, cache lookup/publication, output writes, native tools, profile
  trace writes, test pass accounting, or documentation writes.
- Unsupported graph contract: a direct module containing another `mod` declaration
  must fail with one stable explicit unsupported diagnostic. This rejects nested
  and circular declarations rather than silently treating them as resolved, but it
  does not claim recursive path, namespace, import, visibility, or cycle-analysis
  implementation. Those semantics need a separate specification-backed slice.
- Library contract: `compile_program(source, options)` remains source-compatible,
  but because it has no entry-file context it must reject any `mod` declaration at
  the shared module boundary. It may not ignore the declaration or guess a working
  directory. A future file-aware public API requires its own API/options decision.
- Caller contract: `build` and `run`, `check`, discovered `test`, `profile`, and
  `doc` use the shared resolver/parser boundary. Documentation remains root-only
  after validating direct module sources; discovered tests remain semantic-only in
  this slice. Command-specific outer rendering may remain, but the inner module
  diagnostic and failure point are shared.
- Cache contract: root parsing and direct-module collection precede cache lookup.
  With zero modules, the key remains the MD5 of the exact existing UTF-8 string
  `<root-source>::target=<target>::gpu=<gpu>`. With modules, use the domain-separated
  byte stream `AERO_MODULE_CACHE_V1\0`, then `frame("root", root_source)`,
  `frame("target", target)`, `frame("gpu", gpu)`, the raw unsigned 64-bit big-endian
  module count, and, for each module in declaration order, `frame("name", name)`,
  `frame("candidate", candidate)`, and `frame("source", source)`. A field frame is
  the exact lowercase ASCII label bytes shown above, NUL, unsigned 64-bit big-endian
  payload byte length, then raw UTF-8 payload bytes. The relative candidate is exactly
  `<name>.aero` or `<name>/mod.aero` with `/`, never an absolute/canonical host path,
  so drive letters, working directory, separator, symlink target, and host case
  normalization cannot enter identity. Hash the completed byte stream with the
  existing MD5 cache mechanism. Changing exact source bytes, moving between the two
  candidates, or deleting a module cannot reuse the prior module-bearing entry.
  Cache hits retain all `CORE-010` re-verification/publication rules; this cache is
  a correctness accelerator, not a security-integrity primitive.
- Claim contract: public capability text must classify modules as direct source
  collection only. `use`, `pub`, namespace isolation, recursive modules, and cycles
  are not executable multi-file evidence and must not remain implied as implemented.
- Red tests: missing direct module through public `compile_program`, CLI `build`,
  `run`, `check`, `profile`, discovered `test`, and `doc`; exact nonzero/no requested
  artifact contracts; run must stop before native tools and clean its nonce artifact
  directory; `x.aero` and `x/mod.aero` positive resolution; malformed direct-module
  diagnostic preservation; nested-module explicit rejection; and reusable-optimizer
  cache sequences proving a byte-only source mutation misses, a same-byte move from
  `x.aero` to `x/mod.aero` misses, deletion fails before cache lookup/output, and the
  legacy no-module identity remains an actual verified hit.
- Positive controls: module-free public compilation; existing valid direct-module
  function visibility under compatible flattened AST behavior; existing malformed
  module lex/parse and semantic failures; external LLVM verification/cache controls;
  formatting, all-target compilation, and the complete repository gate.
- Files allowed: `src/compiler/src/module_resolver.rs`; minimal propagation in
  `lib.rs`, `main.rs`, `profiler.rs`, and `doc_generator.rs`; focused module/cache
  tests; README/capability/project-control documentation.
- Files frozen: lexer token rules, parser grammar, AST shapes, semantic name/type
  rules, IR/codegen/backend/graph/quantization algorithms, `CompilerOptions`
  semantics, `aero test` execution semantics, general command dispatch/status,
  registry, benchmarks, Cargo dependencies, releases, and master.
- Risks: centralizing across the separately compiled binary/library module trees can
  accidentally change diagnostics; pre-cache parsing reduces cache-hit speed;
  path-based fingerprints can be nondeterministic; flattening can be mistaken for
  real module namespaces; and a narrow missing-file return could leave alternate
  trusted callers divergent.
- Stop conditions: correct behavior requires deciding nested-module base paths,
  namespace/visibility/import semantics, graph cycles beyond explicit rejection,
  manifest/project lookup, `CompilerOptions`, or another compiler phase outside
  source collection and caller propagation; any caller still resolves/parses direct
  modules independently; a module-bearing cache hit can precede collection; or the
  exact versioned framing/candidate representation above cannot be implemented
  deterministically without host-dependent path normalization.
- Owner: lead-owned tests-first vertical slice. Independent type/IR/backend reviewers
  must approve the exact clean candidate before publication.
- Red checkpoint: public commit
  `9c31820fdc5a252e29d5c62c96ff89f5a4a63eb8` records the independently approved
  exact test snapshot (`badb9d0e8d6059927d949994b39f617fe2f404a8`, tree
  `540a187db87aff5ec0b2964b0c140c6caf9402a4`). Before implementation, the module
  matrix was 2 pass / 5 intentional failures, the cache matrix was 3 pass / 4
  intentional failures, and the full binary suite had 139 pass / only the same four
  intentional cache failures.
- Implementation candidate: one crate-private collector now owns direct resolution,
  strict lexing, fatal parsing, and nested-module rejection for all preregistered
  callers. Build/run collect before cache lookup; the versioned framed cache key
  matches the frozen known vector while preserving the exact no-module legacy key.
  Source-only `compile_program` rejects module declarations. Both focused seven-test
  suites and the complete `./tools/test.sh` gate pass.
- Acceptance: exact implementation diff
  `60fe607413ebc03e9aa5d6296d9067d8cc95d89d`, tree
  `7c57c082e9d5f68afd5c6a4769d9d531a0116642`, was approved by three independent
  reviewers with no P0-P3 findings and published as
  `a711dd5f3802095a4ecbe2dea3d45003675e7459`. Both compiler-test jobs, Rust
  stable/nightly, all CodeQL language analyses, and the aggregate CodeQL check pass.
- Status: accepted. Namespace/import/visibility semantics, recursive module graphs,
  general CLI status, `CompilerOptions`, and pipeline consolidation remain separate.

## AUDIT-018 — Re-rank open risks after CORE-011

- Objective: reproduce the highest-impact open compiler/tooling/security candidates
  at the clean accepted head, apply the two-phase and frozen-semantics stop rules,
  and select exactly one bounded next slice before tests or production changes.
- Audit base: clean published head
  `8598a4c343f5592880bde66cbd99e78083d2a236`; accepted compiler behavior remains
  exact `a711dd5f3802095a4ecbe2dea3d45003675e7459`; upstream `master` remains
  `8f8c7337a4008082fd2a443fcc814b5847b8663f`.
- Fresh command evidence: unknown command, malformed `build`/`check` usage, missing
  build/check source, and a bare benchmark source path all exit zero. The benchmark
  path prints `Unknown command`; the tracked Python harness times that exact
  successful non-compilation and the shell harness explicitly simulates work.
- Fresh aggregate evidence: mixed numeric `[1, 2.5]` passes semantic analysis and
  checked admission but fails the in-process verifier on an Int/Float store mismatch;
  a float array index fails checked admission. Both check/build routes are nonzero
  and produce no requested artifact, so this is still R-011 but not a new publication
  escape.
- Registry evidence: the active install path formats server-controlled resolved
  name/version into a destination join without containment. Publish sends only file
  metadata, ignores the response body, and reports accepted after transport success.
  Live search shares the same unaudited credential/curl boundary; CLI credential
  resolution also precedes local/dry-run separation.
- Ownership stop: the specification requires move invalidation, exclusive mutable
  borrow, non-overlapping shared/mutable borrows, and lifetime validity. The analyzer
  has no lifetime provenance and classifies mutable references as Copy. A one-line
  predicate change would not establish the advertised safety model; credible closure
  crosses unfrozen CFG/provenance/type phases and is stopped under repository rules.
- Ranking: (1) R-017 live registry boundary, critical impact and bounded fail-closed
  containment; (2) R-004 ownership, critical but stopped on scope/semantics; (3)
  R-013/R-015 command status and false benchmark, high/high; (4) R-011 arrays,
  high/high but currently fail closed before output; then backend/claims/span/grammar
  work under the existing register.
- Selection: `CORE-012` quarantines all live registry transport before credentials,
  filesystem activity, or process/network calls. It preserves credential-free local
  search and non-network dry-run preview/plan behavior. It does not repair or enable
  a registry protocol.
- Status: complete; reproductions, ranking, stop rationale, and selected boundary are
  recorded before focused tests or production changes.

## CORE-012 — Quarantine incomplete live registry transport

- Problem: unaudited live registry functions are active despite incomplete payload,
  response, authentication, destination, and overwrite contracts. A registry response
  can influence an uncontained install write path; publish can report acceptance
  without sending package content.
- Priority: P0 containment because remote input and credentials can reach process,
  network, and filesystem mutation through an incomplete critical-impact boundary.
- Dependencies: accepted `CORE-011`; complete `AUDIT-018`; `DEC-017` freezes the
  quarantine and preserved local behavior.
- Hypothesis: one shared registry guard, called by every live function and by CLI
  dispatch before auth, can make the incomplete surface fail closed without choosing
  package, path, auth, response, dependency, or general command semantics.
- Failure contract: exact inner diagnostic is
  `live registry transport is disabled pending a reviewed protocol and trust boundary`.
  Live search/publish/install return this error before all credential, filesystem,
  process, HTTP, digest, response, or write activity. CLI renders it as an error and
  exits nonzero. `publish_live` and `install_live` reject for both values of their
  existing `dry_run` boolean. No live route may fall through to `curl` or return
  preview/success.
- Preserved contract: offline index search remains functional. Publish `--dry-run`
  produces the existing local manifest preview; install `--dry-run` produces the
  existing local plan. These local/dry-run routes neither resolve registry auth nor
  invoke live functions/transport and do not create an install destination.
- Red tests: direct live search expects the exact guard; direct publish/install each
  expect it with `dry_run=false` and `dry_run=true`, before invalid package/target/
  endpoint inputs can act. CLI live search/publish/install expect the same diagnostic,
  nonzero status, no target/package output, and precedence over an empty credential
  or unavailable transport. Local-index search and both CLI dry-run routes use empty
  credential inputs as positive proofs that auth resolution is skipped; existing
  registry unit positives remain green.
- Files allowed: `src/compiler/src/registry.rs`; minimal registry branches/help in
  `src/compiler/src/main.rs`; focused registry unit/integration tests; `README.md`,
  `BUILD.md`, `tutorials/01-getting-started.md`,
  `docs/language/aero_formal_language_specification.md`, capability/matrix, and
  project-control documentation.
- Files frozen: compiler frontend/semantics/IR/codegen/backend/module/cache behavior;
  package/archive/dependency formats; URL, auth, response, path-containment,
  symlink/overwrite, and signature semantics; general CLI status; benchmarks; Cargo
  dependencies; releases/external registry state; and `master`.
- Positive controls: complete existing registry unit suite; offline search ordering;
  exact publish preview fields/hash; exact install plan/trust fields; CLI help truth;
  README/BUILD/tutorial/formal-specification-status/matrix live-quarantine and
  local/dry-run workflow truth while preserving future design wording; formatting,
  all-target compilation, and complete `./tools/test.sh` gate.
- Risks: a CLI-only guard could leave direct live calls active; a library-only guard
  could still read credentials before failure; moving auth resolution could change
  local behavior; a dry-run could accidentally resolve/download; dormant transport
  code could be mistaken for support; broad status cleanup could escape the slice.
- Stop conditions: any need to specify/re-enable network transport, encode package
  bytes, interpret server responses, migrate credentials, validate/canonicalize remote
  paths, choose overwrite/symlink behavior, solve dependencies, touch another compiler
  phase, or contact a real registry. Stop rather than implementing those semantics.
- Owner: lead-owned tests-first vertical slice. Independent type/IR/backend reviewers
  must approve the exact preregistration, red snapshot, and implementation candidate
  before publication.
- Red checkpoint: public commit
  `57c4ec70190822cb4552d313e5e7ea0f2dc5cbed` records the independently approved
  tests-only snapshot (exact diff `4058775145e68aa9a5512853c04b0dde04730464`,
  tree `227254ef8177d8e15b69c42bd1e2d94c1442879a`). Direct registry evidence was
  7 pass / 5 intentional failures; CLI evidence was 0 pass / 6 intentional failures.
  The full gate passed formatting/correctness Clippy and stopped at 134 pass / only
  the same five intended direct failures before Cargo reached later test targets.
  Three independent reviewers found no P0-P3 issues.
- Implementation candidate: one crate-private guard is the first operation in each
  live function and is checked by CLI live dispatch before auth. Local search and
  CLI publish/install dry-runs skip auth and use the existing local helpers. Both
  direct registry targets pass 12/12, the CLI matrix including help truth passes 7/7,
  and exact `./tools/test.sh` passes 139 library, 148 binary, every active integration
  suite, formatting, correctness Clippy, and doc tests; 38 Phase 5 tests remain
  intentionally ignored. Public truth now describes quarantine while preserving the
  future design target.
- Acceptance: exact implementation diff
  `05e55496f6664713192b2dbf94eca785abe2931d`, tree
  `85ed76ab0141409796e167704e4100dd4d15c26f`, was approved by three independent
  reviewers with no P0-P3 findings and published as
  `6780a23cd8b63df124477c7db1190d61dd25f3b8`. Both compiler-test workflows,
  Rust stable/nightly, all CodeQL language analyses, and aggregate CodeQL pass.
- Status: accepted. Live registry transport remains disabled; package/payload,
  response, auth, URL, destination, overwrite/symlink, digest/signature, dependency,
  and re-enablement semantics remain separate and unimplemented.

## AUDIT-019 — Revalidate the CLI false-success and benchmark boundary

- Objective: at the clean accepted `CORE-012` head, inventory every top-level CLI
  success/failure boundary, reproduce the tracked benchmark's exact compiler
  invocation, compare R-013/R-015 with the remaining open risks, and select one
  bounded tooling slice before adding tests or production behavior.
- Audit base: clean public documentation head
  `b7bb42958e78fb97ea0d991fa3f4cdb40bbcce2f`; accepted production behavior is
  `6780a23cd8b63df124477c7db1190d61dd25f3b8`; upstream `master` remains
  `8f8c7337a4008082fd2a443fcc814b5847b8663f`.
- Process correction: an initial PowerShell `ProcessStartInfo` batch accidentally
  omitted intended arguments and repeatedly exercised the no-argument route. Its
  results were discarded. The corrected explicit-argument probe below is the only
  process matrix admitted as evidence.
- Corrected command evidence: no arguments; an unknown top-level command; malformed
  `build`, `run`, `check`, `fmt`, `doc`, `profile`, `graph-opt`, and `quantize`;
  missing input files for each applicable command; registry with no or unknown
  subcommand; and malformed conformance all print help, usage, or an error but exit
  zero. Standalone top-level help/version also exit zero. `registry help`,
  `registry --help`, and every other unknown registry subcommand currently share the
  same zero-status fallthrough rather than an explicit help contract.
- Static dispatcher evidence: `main.rs` uses bare `return` after invocation and
  operational errors across all major branches. Failed formatter/doc/graph/quantize
  writes only print diagnostics. `check`, `fmt`, and `test` ignore extra operands;
  `lsp` ignores operands and would start the server; unknown top-level and registry
  commands fall through naturally. Compiler, registry, verifier, conformance, init,
  and discovered-test failures that already call `exit(1)` are preserved controls.
- Benchmark evidence: `performance_benchmark.py::run_compilation_benchmark` invokes
  `cargo run --release -- <source-file>` without the required `build` command and
  accepts return code zero as compilation. The bare source path is the corrected
  unknown-command case above. `benchmarks/harness/run_benchmarks.sh` explicitly
  sleeps and reports simulated compilation/execution. No benchmark was run and no
  performance result was generated during this audit.
- Risk comparison: R-004 remains critical but stopped because credible ownership
  closure requires unfrozen lifetime/alias/CFG/provenance semantics across more than
  two compiler phases. R-011 remains high/high but the reproduced array cases fail
  closed before publication. R-013/R-015 are high/high, externally observable, and
  share a bounded outer-dispatch cause. Backend/version/grammar/coverage risks do not
  offer a smaller correctness containment than truthful process status.
- Selection: `CORE-013` establishes one typed CLI-owned status contract and
  reclassifies the affected compilation measurements as invalid while preserving
  all evidence. Making a bare source path a usage failure stops the legacy Python
  driver from recording successful non-compilation, but does not make that driver a
  valid benchmark. Benchmark execution remains quarantined.
- Status: complete; corrected reproduction, full dispatcher inventory, risk ranking,
  process correction, and the bounded `CORE-013` contract are recorded before tests
  or production edits.

## CORE-013 — Make CLI statuses truthful and quarantine invalid compilation claims

- Problem: user-visible usage, input, output, and dispatch failures commonly print
  an error and return status zero. Automation cannot distinguish success, and the
  tracked Python compilation driver records an unknown command as successful work.
- Priority: P1 tooling correctness. The boundary is broad within the dispatcher but
  confined to one outer CLI phase and existing evidence classification.
- Dependencies: accepted `CORE-012`; complete `AUDIT-019`; `DEC-018` freezes status,
  arity, claim-preservation, and benchmark-quarantine rules.
- Status contract: for outcomes owned by CLI dispatch before delegated program
  execution, introduce one typed status boundary. Completed commands, standalone
  `-h`/`--help`, standalone `-v`/`--version`, and standalone explicit
  `registry help|-h|--help` return `0`. Invocation errors return `2`: no command;
  unknown top-level or registry command; missing, extra, or malformed operands;
  unrecognized option/target/backend/mode values; and incomplete option values.
  Operational, compiler, verifier, test, report, filesystem, registry, init, and LSP
  failures return `1`. Existing diagnostic wording and stdout/stderr placement remain
  unless a new strict-arity diagnostic or explicit registry-help branch is required.
- Arity contract: top-level help/version are standalone; `test` and `lsp` accept no
  operands; `check` and `fmt` accept exactly one input; `init` accepts zero or one
  path. Existing build/run/doc/profile/graph/quantize/registry/conformance option
  languages remain otherwise unchanged. Duplicate option policy is frozen at its
  current behavior and is not redesigned in this slice.
- Execution exception: after a valid CPU `run` successfully reaches delegated
  program execution, `run_aero_program` continues to pass through the program's
  arbitrary exit code, including values equal to `1`, `2`, or outside `0/1/2`. Those
  delegated statuses are not CLI-owned classifications and the numeric codes are not
  globally unique without command context. The helper's internal termination remains
  frozen. `aero test` remains discovery plus strict parse/module/semantic analysis;
  this slice makes only its invocation and existing failure status truthful and does
  not claim execution.
- Output/publication contract: usage, missing-input, compiler, verifier, and other
  proven pre-publication failures remain nonzero and create no requested new output.
  A direct output-write or project-initialization failure must return `1` and must
  not print the corresponding success message, but existing non-atomic write and
  partial-initialization behavior is frozen and may leave a created, truncated, or
  partial path. Transactional publication and rollback require a separate task.
  Successful command behavior and the accepted CORE-010/011/012 phase, cache,
  module, verifier, and registry guards remain unchanged.
- Benchmark/claim contract: the Python and simulated shell benchmark drivers are
  frozen and must not be executed. The bare-source invocation must return `2`, so it
  cannot enter the Python driver's successful timing set. README, benchmark guide,
  claim index, audit, matrix, and project controls must label the two tracked Python
  compilation series invalid measurements rather than current/historical Aero
  compilation evidence. Raw artifacts remain preserved. Lexer and external
  llama.cpp records keep their separately audited qualifications; no number is
  added, upgraded, rerun, or generalized.
- Red tests: a new process-level CLI matrix must prove exact CLI-owned `0/1/2`
  classes for help/version, no command, unknown and bare-source commands, every major
  command's malformed invocation, strict extra-operand cases, missing inputs,
  registry help/unknown/malformed/local failure, conformance malformed/report
  failure, init malformed/operational failure, and failed output writes. Every
  changed error/help branch must assert the established or newly frozen diagnostic
  text and stdout/stderr channel; output-write/partial-init cases assert status and
  absence of a success line, not rollback. Pre-publication cases retain exact
  requested-new-artifact negatives.
- Backend/error distinctions: unrecognized build/run targets, graph backends, and
  quantization modes/backends are exact invocation `2`; a recognized but unavailable
  CUDA run target is exact operational `1`. Accepted graph/quantize configurations
  that fail LLVM verification, calibration loading, or output writing are exact `1`.
  Representative parser/compiler, verifier/native-tool, registry-quarantine,
  discovered-test, conformance/report, init, and write failures also assert exact
  `1`, not merely nonzero.
- Positive controls: process-level success/output controls for every altered command
  family in isolated workspaces, including test discovery, doc/profile outputs,
  graph/quantize with deterministic verifier controls, init, LSP with controlled
  stdin EOF, registry, and conformance; a delegated CPU-run exit value outside
  `0/1/2` remains passed through; complete existing registry/module/checked-IR/LLVM
  CLI suites; accepted CPU/CUDA/ROCm graph/quantize interfaces retain current
  behavior without implying device execution; formatting; all-target compilation;
  exact complete `./tools/test.sh`; and static claim/evidence-preservation checks.
- Files allowed: `src/compiler/src/main.rs`; one focused
  `src/compiler/tests/cli_status_contract_tests.rs`; minimal public help/status text
  in `README.md`, `BUILD.md`, `benchmarks/README.md`, `BENCHMARK_PROTOCOL.md`,
  `claim-verification/claims.json`, `SPEC_IMPLEMENTATION_MATRIX.md`, and project-
  control/capability documents.
- Files frozen: lexer/parser/AST/semantics/IR/codegen/backend/module/cache/registry
  implementation; compiler API architecture; command feature maturity; test
  execution semantics; `run_aero_program`; benchmark Python/shell/Rust/GGUF code and
  result artifacts; Cargo dependencies; versions; releases; external state; and
  `master`.
- Risks: a mechanical status edit can misclassify runtime errors as usage, map a
  delegated program's exit status into a CLI-owned class, change a
  successful help path, start the LSP during testing, mutate a file while probing,
  suppress established diagnostics, or imply that fail-closed legacy benchmark
  behavior is valid measurement. Process tests must isolate writable paths and avoid
  starting servers or running benchmark drivers.
- Stop conditions: implementation requires a CLI framework/dependency, command-
  feature redesign, compiler-pipeline consolidation, `run_aero_program` refactor or
  exit remapping, transactional output/rollback semantics, test execution semantics,
  benchmark harness repair/rerun, new performance claims, language/backend semantics,
  registry re-enablement, or more than the outer dispatcher and evidence
  classification. Stop rather than broadening the slice.
- Owner: lead-owned tests-first vertical slice. Independent type/IR/backend reviewers
  must approve the exact preregistration, tests-only red checkpoint, implementation
  candidate, and acceptance closure before publication.
- Red checkpoint: public commit `d405fc9` records the independently approved
  tests-only matrix. It reproduced exactly three existing success controls and four
  aggregate failures covering the frozen invocation, operational/publication, and
  claim-classification gaps.
- Implementation evidence: one typed outer `CliStatus` boundary now maps completed
  CLI-owned work to `0`, operational failure to `1`, and invalid invocation to `2`.
  Strict arity, direct-write failure, explicit registry help, and all previously
  false-zero branches are controlled without refactoring delegated CPU execution.
  The exact focused matrix passes 7/7 and the complete `./tools/test.sh` gate passes.
  The two `performance_benchmark.py` compilation series are invalid measurements;
  raw artifacts, current/historical Lexer evidence, and the external llama.cpp
  reference remain preserved and separately qualified. No benchmark was executed.
- Acceptance evidence: exact implementation diff
  `ea0c37c0a0af4f51867a5e6b0d0be2aa010f2d7c` and tree
  `b62a1fa5b1d5443b0197917dbd52f2c16239c0f5` received three independent approvals
  with no P0-P3 findings. Public implementation commit
  `a78dd004aa37c39212711027b777698118d9dc02` passes both compiler-test workflows,
  Rust stable/nightly, all three CodeQL language analyses, and aggregate CodeQL.
- Acceptance closure: exact documentation diff
  `38de08f60880cbba9c89b1557aa019b058edc4e6` and tree
  `41fc99f38b5dfbde0acfa5ac5fbd4be308230a66` received three fresh independent
  approvals with no P0-P3 findings after correction of one rejected stale-head
  label. Public closure commit `18526ff7a80db222c1348496f24f710d09249dfc`
  passes both compiler-test workflows, Rust stable/nightly, all three CodeQL
  language analyses, and aggregate CodeQL.
- Status: complete at accepted public closure head `18526ff`.

## AUDIT-020 — Re-rank open risks after truthful CLI acceptance

- Objective: at the clean accepted `CORE-013` closure, reproduce the public first-run
  workflow, compare every remaining open risk for impact, evidence, phase count, and
  frozen-policy dependencies, and select one bounded slice before tests or production
  edits.
- Audit base: clean public documentation head
  `18526ff7a80db222c1348496f24f710d09249dfc`; accepted production behavior is
  `a78dd004aa37c39212711027b777698118d9dc02`; upstream `master` remains
  `8f8c7337a4008082fd2a443fcc814b5847b8663f`.
- Reproduction: from the repository root, the README command
  `cargo build --release` exits `101` because no root `Cargo.toml` exists. Its next
  PATH entry points to nonexistent root `target/release`. The corrected root command
  `cargo build --release --manifest-path src/compiler/Cargo.toml` exits `0` and
  produces `src/compiler/target/release/aero.exe` on this host. The existing release
  binary initializes an isolated generated project with status `0`, creates both
  `aero.toml` and `src/main.aero`, and checks that source with status `0`.
- Flagship evidence: the README presents absent `aeronum`/`aeronn` packages plus
  grouped imports, named arguments, method calls, and distributed/model behavior as
  a directly playable flagship. The capability audit already establishes that this
  snippet cannot compile on the active repository. The existing generated source
  `fn main() { println!("Hello, Aero!"); }` is the bounded executable replacement.
- Risk comparison: R-004 ownership, R-006 pipeline convergence, R-009 spans, R-010
  grammar authority, R-011 typed aggregates, and R-012 coverage restoration require
  unfrozen semantics or more than two phases. R-007 requires backend/device evidence
  unavailable on the current host. R-008 requires an explicit release/language
  version policy. R-016 is medium/medium and its support policy is also unfrozen.
  R-014 is high/medium, externally visible on the first command, already has a valid
  generated-project core, and can be controlled within documentation, tests, and CI.
- Selection: `CORE-014` makes only the minimal generated CPU project the executable
  Quick Start, corrects root manifest/binary paths, and makes CI execute that path.
  Experimental commands remain documented outside the minimal first-run contract
  with their existing qualifications.
- Audit restrictions: no benchmark driver/result was run or edited; no compiler,
  language, backend, project scaffold, version, registry, package, release, or master
  behavior changed. One isolated temporary generated-project directory was left under
  the host temporary directory after destructive cleanup was denied; it is not part
  of the repository or admitted evidence beyond the recorded init/check statuses.
- Status: complete; reproduction, risk ranking, selection, and the bounded contract
  are recorded before tests or implementation.

## CORE-014 — Make the public generated-project Quick Start executable

- Problem: the first README build command and binary path are invalid from the root,
  while the advertised flagship depends on unsupported syntax and absent packages.
  No gate executes the documented generated-project workflow end to end.
- Priority: P1 public correctness. This affects the first user-visible path but can
  be contained to documentation, one focused contract test, and stable Linux CI.
- Dependencies: accepted `CORE-013` closure `18526ff`; complete `AUDIT-020`;
  `DEC-019` freezes the generated-project contract and its honesty boundary.
- Canonical POSIX contract: from repository root, build with
  `cargo build --release --manifest-path src/compiler/Cargo.toml`, export
  `$PWD/src/compiler/target/release` on `PATH`, run `aero --version`, initialize a
  fresh `my_app`, enter it, run `aero check src/main.aero`, then run that same source
  on CPU with documented LLVM 22/Clang tools available. The generated program's
  observable output is `Hello, Aero!`.
- Windows contract: `BUILD.md` supplies equivalent PowerShell build/PATH commands
  and the exact `src\\compiler\\target\\release\\aero.exe` location. The focused
  static contract must assert the exact Windows manifest, PATH, and executable
  fragments. The README remains a POSIX block and links to the platform detail rather
  than mixing shells.
- Public-surface boundary: replace the unsupported model snippet with the exact
  existing generated source. Keep accelerator, graph, quantization, registry,
  benchmark, LSP, and other command examples outside the minimal Quick Start and link
  their capability limits. Do not relabel ROCm object generation or absent CUDA
  execution as a successful run.
- Red tests: add one focused integration target that isolates the README Quick Start
  section and requires the exact manifest/binary/generated-project commands plus a
  capability-status link; rejects `aeronum`, `aeronn`, and the conceptual Transformer
  from that section; parses the Windows section of `BUILD.md` and requires the exact
  PowerShell manifest-build, PATH, and executable-location fragments; and
  process-tests `aero init` followed by `aero check` against a fresh workspace.
  Before documentation edits, both static platform contracts must fail while the
  generated-project process control passes.
- CI acceptance: on stable Linux, prepend `/usr/lib/llvm-22/bin` to `PATH` before the
  Quick Start commands and assert `command -v clang` and `command -v llc` resolve
  inside that directory plus `clang --version` and `llc --version` report major 22.
  Keep `AERO_LLVM_OPT=/usr/bin/opt-22` and
  `AERO_LLVM_AS=/usr/bin/llvm-as-22` authoritative for verification. Then execute the
  documented root release build, PATH export, version, fresh init, check, and CPU run
  in an isolated runner directory. Capture only the `aero run` command, require status
  zero, and require exactly one line matching `^Output: Hello, Aero!$`; a source
  literal or another compiler message is not acceptable output proof. Nightly and
  existing example/gate coverage remain unchanged.
- Full acceptance: focused tests, exact complete `./tools/test.sh`, workflow syntax,
  three exact independent reviews, and the complete public CI matrix must pass before
  closure. Documentation text is evidence only for the commands actually executed.
- Files allowed: new
  `src/compiler/tests/quick_start_contract_tests.rs`; `README.md`; `BUILD.md`;
  `tutorials/01-getting-started.md`; `.github/workflows/rust.yml`; and minimal risk,
  decision, matrix, ledger, capability, and project-control records.
- Files frozen: all compiler/CLI/project-init/library implementation; lexer, parser,
  AST, semantics, IR, codegen, module, cache, verifier, backend, registry, benchmark,
  example source, Cargo dependency/version/lock data, release/package configuration,
  external state, and `master`.
- Risks: a string-only test can bless commands CI never runs; shell paths can become
  invalid after `cd`; output matching can confuse compiler diagnostics with program
  output; installed versioned tools can leave unversioned discovery absent or bound
  to another major; a workflow edit can silently cover only one command; Windows
  instructions can drift behind Linux CI; or removing the model snippet can erase
  vision rather than label it conceptual. Tests must isolate both platform sections,
  CI must bind and verify the selected native tools then execute the actual path, and
  founding AI/ML direction remains in `LANGUAGE_VISION.md`/`FRAMEWORK_ALIGNMENT.md`
  without a runnable claim.
- Stop conditions: the generated project requires compiler/CLI/scaffold changes;
  native execution requires backend behavior changes rather than documented tools;
  correctness depends on selecting 0.x versus 1.0, admitting new syntax/packages, or
  repairing unrelated tutorials/examples; more than documentation/tests/workflow is
  needed; or any benchmark/device/release/registry action becomes necessary. Stop
  rather than broaden or weaken the contract.
- Owner: lead-owned tests-first documentation/CI slice. Independent type, IR/codegen,
  and backend/claim reviewers must approve the exact preregistration, tests-only red
  checkpoint, implementation candidate, and acceptance closure before publication.
- Red checkpoint: public commit `fc77e9979f996aaa0110ba48246b24ebca67acbd`
  records the independently approved tests-only contract. Its exact reviewed staged
  diff is `b02c2bad25a28ec069303c02fa39de68b64561e8` and tree is
  `f301087d2749d4425bc7d913b3109b1b7aab64e2`. Local evidence was exactly two
  passing controls and three intended README/BUILD/workflow failures. Public
  compiler-test and stable Rust jobs reproduced only those failures after their
  earlier steps passed; nightly reached its test step and was cancelled by the
  unchanged matrix fail-fast behavior after stable failed.
- Implementation candidate: README now uses the exact manifest-qualified release
  build, binary PATH, generated source, init/check/run sequence, platform and backend
  links; advanced commands are separately labeled experimental. `BUILD.md` supplies
  the exact PowerShell path and names the complete LLVM 22 native toolchain,
  including the required `opt`/`llvm-as` verifier choice. A review-found omission
  received a focused corrective assertion first, reproducing exactly 4 pass / 1
  intended prerequisite failure before the documentation correction. The
  getting-started tutorial uses the same generated project. Stable Linux CI binds
  unversioned LLVM/Clang discovery to LLVM 22, retains verifier overrides, and
  executes the exact fresh-project path with status and anchored-output proof.
  Focused tests pass 5/5 and exact complete
  `./tools/test.sh` passes locally. No compiler, scaffold, backend, benchmark,
  version, dependency, registry, release, or master behavior changed.
  One prior complete-gate attempt stopped in the unchanged
  `cli_status_contract_tests`; the target immediately passed 7/7 in isolation and
  the unchanged full gate passed on rerun. This is retained as residual pre-existing
  flake uncertainty, not classified as a `CORE-014` regression.
- Acceptance evidence: the exact implementation diff
  `687dd5f3d6360dfd7822e7809944f63d4caccfdd` and tree
  `869fca43edb8b5888bdec01d0bfc7cdecfa451a5` received three independent approvals
  with no P0-P3 findings. Public implementation commit
  `c56b1d561930a042eeff214196fd1b4f05a77fb6` passes all eight checks. Its stable
  Linux job resolved LLVM 22 `clang`, `llc`, and external verifier tooling, then
  completed the documented build/init/check/run path with status zero and exactly
  one anchored `Output: Hello, Aero!` line. Nightly Rust, both compiler-test jobs,
  all three CodeQL language analyses, and aggregate CodeQL also pass.
- Acceptance closure: exact documentation diff
  `6e05c26763ed3a1c6e4ec359361867f76e9d4c4c` and tree
  `b3a6bf38769579dbfc0fa0da5c4881620f7129c3` received three independent approvals
  with no P0-P3 findings after correction of stale CPU-evidence and CodeQL-count
  wording. Public closure commit
  `1535ce2a214f512c140535e7c42799af1f920d5c` passes both compiler-test workflows,
  stable/nightly Rust, all three CodeQL language analyses, and aggregate CodeQL.
- Status: complete at accepted public closure head `1535ce2`.

## AUDIT-021 — Re-rank open compiler-integrity risks after executable Quick Start

- Objective: from the clean accepted `CORE-014` closure, retest the remaining
  high/critical type and ownership risks, compare them with high/high aggregate,
  backend, grammar, diagnostic, and coverage risks, and select one bounded slice
  before adding tests or production edits.
- Audit base: clean public head
  `1535ce2a214f512c140535e7c42799af1f920d5c`; accepted compiler behavior remains
  `a78dd004aa37c39212711027b777698118d9dc02`; upstream `master` remains
  `8f8c7337a4008082fd2a443fcc814b5847b8663f`. Draft PR #4 has exactly eight
  successful checks and reports a clean merge state.
- Reproduced R-002 false success: each of `let value: String = 1`,
  `let value: bool = 1`, `let value: Widget = 1`,
  `let values: [int; 2] = [1.0, 2.0]`, and
  `let values: [int; 3] = [1, 2]` passes CLI `check`, exits zero from CLI `build`,
  and creates the requested LLVM artifact. The semantic analyzer validates only
  numeric binding annotations; checked IR admission ignores every binding
  annotation. Exact `String = "ready"`, comparison-produced `bool`, and homogeneous
  integer-array controls pass both commands and produce artifacts.
- Aggregate phase evidence: mixed `[1, 2.5]` and `[1, 2][1.5]` both return nonzero
  and produce no artifact, but the build trace first reports semantic success. The
  mixed literal reaches the internal verifier and fails an Int/Float store; the
  index reaches checked IR admission and fails `array index must be Int`. Array
  semantics currently infer only the first element and discard the index type.
- Checked-boundary parity evidence: public checked admission prefers optional
  caller-supplied `Expression::Binary.ty` over its operand-derived result, so a
  constructed `float`-annotated binding can spoof `1 + 2` as float and reach
  lowering as a contextual promotion. It also treats lowercase `string` as
  `Ty::String`, while active semantics maps that spelling to the distinct named
  `Ty::Struct("string")`. Neither divergence is an authorized conversion or alias.
- Adjacent critical controls: an uninitialized annotated read returns nonzero in
  semantics and produces no artifact, so it is not part of the reproduced false
  success. A mutable-reference-copy probe also returns nonzero without an artifact,
  but only because checked IR rejects borrow/deref after semantic success. R-004
  remains critical: correcting mutable-reference Copy alone would falsely imply a
  borrow/lifetime model, while credible closure requires unfrozen CFG/provenance
  work across more than two phases.
- Risk comparison: R-005 trusted checked publication routes remain controlled and
  restricting public unchecked compatibility APIs requires a major-boundary policy.
  R-006, R-009, R-010, and full R-012 restoration span more than two phases; R-007
  needs device evidence; R-008 and R-016 need explicit version/toolchain policy.
  R-011's reproduced cases already fail closed, although too late. R-002 is the
  highest-impact active false-success boundary; four reproduced forms can be closed
  in semantics plus checked IR without changing syntax, representation, or backend
  lowering, while the contextual custom-name case remains open.
- Current coverage inventory: Cargo lists 139 library and 148 binary unit tests,
  with 105 names shared by both compiled module trees. `cargo test --tests -- --list`
  reports 557 test-target entries and 437 distinct displayed names; those names are
  not claimed as independent behaviors. All 38 `phase5_tests` remain ignored.
  R-012 remains open and is not used as evidence for type-contract closure.
- Selection: existing exact scalar `int`/`i32`/`float`/`f64` annotation behavior
  remains unchanged in fully analyzed bindings; syntax-preflighted trait defaults
  remain outside it. Outside active semantic generic scopes, `CORE-015` adds a closed,
  nonrecursive binding-local rule for exact `Type::Named("bool")`,
  `Type::Named("String")`, and one-dimensional
  `Type::Array(Type::Named(name), count)` when `name` is one of the four numeric
  spellings and `count > 0`. This selects four of five reproduced annotation false
  successes. In the same non-generic semantic scope, numeric arrays validate every element left-to-right,
  exact homogeneity/count, and integer indexes before IR. Checked admission mirrors
  numeric/bool/String/fixed-numeric equality for direct/non-generic AST and separately
  derives/verifies binary result metadata. Lowercase `string` is excluded. No
  conversion, alias, generic, or aggregate-execution behavior is implemented.
- Audit restrictions: temporary probe sources and requested LLVM outputs were
  isolated under `.audit-021`, inspected, then deleted; the empty directory is
  untracked. No benchmark, registry, release, device, compiler source, test, or
  external artifact was changed or published.
- Status: complete; the reproduction, ranking, and selected boundary are recorded
  before test or production changes.

## CORE-015 — Enforce selected initialized binding types before IR

- Problem: selected `String`, `bool`, and nonempty fixed numeric-array binding
  annotations are discarded. Invalid typed programs can therefore pass semantics and checked IR,
  publish valid LLVM for the inferred value type, and report success. The reproduced
  custom-name false success remains open because arbitrary named annotations overlap
  in-scope generic parameters. Array semantics also infer only the first numeric
  element and ignore index type, leaving checked IR/verifier to reject related invalid
  programs after the semantic boundary.
- Priority: P1 compiler correctness / R-002 HIGH-CRITICAL. Unlike the adjacent
  ownership and architecture gaps, this is an active false-success artifact boundary
  with an exact two-phase correction.
- Dependencies: accepted `CORE-014` closure `1535ce2`; completed `AUDIT-021`; and
  `DEC-020`, which freezes exact selected binding equality and array inference
  without defining conversions or new aggregate behavior.
- Frozen semantics: the existing numeric scalar contract remains active wherever
  semantics fully analyzes bindings. Syntax-preflighted trait default bodies retain
  pre-task behavior. New eligibility requires fully analyzed code with no active
  generic type-parameter scope and is syntax-level, nonrecursive, and binding-local:
  exactly `bool`, canonical
  `String`, or a nonempty one-dimensional fixed array over
  `int`/`i32`/`float`/`f64`. A selected
  initialized annotation must equal the fully inferred `Ty`; bool maps to `Ty::Bool`,
  canonical String to `Ty::String`, and numeric aliases normalize as before. The new
  mapper must not change global/recursive AST mapping. An exact annotation preserves
  the inferred type; a mismatch reports the binding plus expected/actual types before
  lowering. Lowercase `string`, custom `Widget`, explicit generic/reference/tuple
  annotations, flat nonnumeric arrays, nested arrays, and arrays wrapping excluded
  forms keep pre-task annotation-ignore behavior. Inside a semantic generic scope,
  contextual `T`/`bool`/`String`/`string` and fixed numeric-array annotations also keep
  pre-task behavior across generic function, impl, and trait containers. None is
  represented as supported. No conversion, subtyping, structural fabrication, or
  String ownership/slice decision occurs.
- Array contract: outside active semantic generic scopes, a numeric array literal
  whose first successfully inferred element is `Ty::Int` or `Ty::Float` infers every
  element left-to-right. After preserving any child diagnostic, every later element
  must exactly equal the first; indexing an inferred numeric fixed array requires
  `int`; and a selected fixed numeric-array annotation exactly matches type and count.
  Mixed numeric elements are not promoted. Generic-scope numeric arrays and all
  nonnumeric array inference/index behavior remain unchanged, as do empty/nested
  admission, bounds, mutation, slices, layout, and execution. Specifically, empty
  literals and zero-length annotations are excluded: semantics retains the `[Int; 0]`
  default plus current annotation-ignore acceptance, while checked IR retains its
  no-logical-element-type rejection before binding comparison. Typed zero-length
  array repeats remain admitted at existing boundaries with annotations ignored.
- Boundary coverage: active `SemanticAnalyzer` routes must reject first. Public
  checked `IrGenerator::try_generate_ir` must independently reject constructed AST
  callers that bypass semantics. Checked admission derives binary results from the
  admitted operands/operator; optional `Expression::Binary.ty` is an assertion, not
  an inference input, and must exactly match that result before binding comparison
  or lowering. Existing recursive/global `admission_type` behavior for excluded forms
  is frozen. Checked IR's existing rejection of generic functions before body
  admission is frozen; selected binding comparisons must also be skipped in generic
  impl contexts, while trait bodies remain syntax-only. The binary metadata assertion
  applies to every otherwise admitted checked expression, including generic-impl
  methods, but performs no generic substitution or annotation remapping.
  Every semantic generic-scope push must be balanced on success and error before the
  public analyzer returns so reuse cannot inherit a false scope exemption.
  Library compilation, root/direct-module CLI check, and
  root/direct-module CLI build must surface the semantic failure; failed builds
  create no requested artifact. Syntax-only default trait bodies never reach IR and
  remain outside full type checking; this task must not imply otherwise.
- Uninitialized boundary: declarations without values and their later-use error are
  unchanged; Aero has no active reassignment statement in this slice. Existing
  duplicate-binding, child-error, numeric annotation, scope, function-contract,
  unsupported-expression, excluded-annotation, and checked-verifier diagnostic
  precedence must remain.
- Red tests: add `src/compiler/tests/binding_type_contract_tests.rs`. It must prove
  direct semantic and checked-IR failure for String/bool/array element/array length
  mismatches plus direct checked-IR failure for int-from-float and float-from-int
  scalar mismatches;
  full left-to-right numeric homogeneity; mixed numeric literal
  and non-int index failure in the semantic phase; later-element child-error
  precedence; direct constructed-AST rejection of spoofed binary result metadata on
  an unannotated or excluded binding and within a generic impl; passing controls for
  matching and absent binary metadata; semantic and checked-IR selected-mismatch
  rejection in a non-generic impl;
  exact numeric/String/bool/fixed-numeric-array controls; unannotated controls;
  uninitialized-use and duplicate-name controls; library no-unwind errors; and
  isolated root/direct-module check/build status, diagnostics, and no-artifact
  behavior. Green preservation controls at direct semantic and checked-IR boundaries
  must pin current annotation-ignore behavior for lowercase `string`, custom
  `Widget`, explicit generic/reference/tuple forms, flat bool/String/string arrays,
  nested arrays, and arrays wrapping each excluded form. Semantic-only generic-
  function/impl controls must pin deliberately mismatched in-scope `T`, `bool`,
  `String`, `string`, and fixed numeric-array annotations, plus mixed numeric arrays
  and float numeric-array indexes; another must preserve numeric-scalar mismatch
  rejection in fully analyzed generic function/impl bodies. Generic trait defaults
  must retain syntax-preflight acceptance. Separate semantic-only controls must
  preserve representative nonnumeric array heterogeneity and non-integer indexing. Checked-IR
  controls must retain generic-function rejection before body admission, generic-impl
  annotation-ignore and mixed-numeric-array behavior, existing non-integer-index
  rejection, and syntax-only generic trait bodies. These are quarantine controls,
  not support. Phase-specific empty-array controls must preserve semantic acceptance
  for unannotated, `[int; 0]`, and `[float; 0]` bindings and direct checked-IR rejection
  for all three before annotation equality. Direct semantic and checked-IR green
  controls must preserve `[float; 0] = [1; 0]` and `[int; 0] = [1.5; 0]` typed repeats;
  these deliberately mismatched types directly bind the `count > 0` selector guard.
  A same-analyzer test must first fail a generic impl on an existing numeric mismatch
  and then prove a non-generic selected mismatch still rejects after balanced scope
  cleanup.
  Before production edits, only the frozen new negative assertions may fail; every
  preservation/control case and all prior tests must pass.
- Full acceptance: focused red/green matrices, exact complete `./tools/test.sh`,
  three exact independent reviews of preregistration, tests-only red, implementation,
  and closure, followed by the complete public CI matrix at each published green
  checkpoint. A verifier rejection is not acceptable semantic evidence.
- Files allowed: new `src/compiler/tests/binding_type_contract_tests.rs`;
  `src/compiler/src/semantic_analyzer.rs`; `src/compiler/src/ir_generator.rs`; and
  minimal ledger, decision, risk, capability, matrix, backend-count, and project
  control records.
- Files frozen: lexer, parser, AST/type representation, code generator, IR verifier,
  LLVM verifier, CLI dispatch/status taxonomy, modules/cache, ownership/borrowing,
  function/generic/trait semantics, aggregate layout/execution, backends, workflow,
  dependencies, version/release policy, registry, benchmarks/claims, examples,
  scaffold, external state, and `master`.
- Risks: exact equality can accidentally reject a documented alias or introduce a
  conversion; first-element errors can mask later child failures; semantic and IR
  selected predicates can drift; caller metadata can override derived types;
  lowercase `string`, a selected-spelling generic parameter, or an excluded recursive
  form can be captured; error cleanup can leak a generic scope; zero length can be
  mistaken for a selected array; array count can be confused with bounds; cached/
  public builds can retain stale artifacts; or the change can appear to
  certify String, bool, arrays, custom, generic, or ownership execution broadly.
  Tests must bind exact spelling/shape eligibility, excluded green controls, phase
  identity, metadata parity, source order, fresh outputs, and non-promotion.
- Stop conditions: correctness needs a conversion/subtyping/defaulting policy, a
  parser/AST/type-representation change, assignment or definite-initialization
  semantics, ownership/provenance, a third compiler phase, array layout/codegen,
  generic substitution, a change to any generic-scope annotation/array or excluded
  annotation/array behavior, or any backend/version/benchmark/release/registry action.
  Stop rather than infer policy, widen the slice, or weaken a failing contract.
- Owner: lead-owned two-phase vertical slice. Independent type, IR/codegen, and
  backend/claim reviewers must approve the exact preregistration, tests-only red
  checkpoint, implementation candidate, and acceptance closure before publication.
- Tests-only red evidence: the new 16-test target is exactly 8 passing preservation
  groups and 8 intended failing contract groups on public preregistration head
  `4f31f0c`. The failures cover the selected semantic and direct checked-IR binding
  matrix, numeric-array semantic ordering, universal binary metadata, non-generic
  impls, analyzer reuse after a generic error, library no-unwind behavior, and both
  root/direct-module CLI layouts. Both invalid CLI builds publish the requested LLVM
  artifact. The exact `./tools/test.sh` passes formatting and correctness Clippy and
  stops on only the intended new target; `cargo test --no-fail-fast` proves all prior
  active targets pass, with 38 pre-existing Phase 5 tests still ignored. Three
  independent reviewers approved exact diff
  `e158ad61282617a63dade4976a7c23fe53aa0af8` and tree
  `db2ac2959f9815fab5d4b649e563b59c83459dfe` with no P0-P3 findings. Public red
  commit `b203ea429b5a039705be5a5b11998e6dc59f5a24` reproduces the same 8/8 target in
  both compiler-test jobs and Rust nightly; stable is matrix-cancelled, while every
  CodeQL analysis and the aggregate CodeQL check pass.
- Implementation evidence: the local candidate changes only the two preregistered
  production phases, plus the focused test and minimal records. The focused test adds
  implementation-review regression controls for numeric-array child ordering,
  single-pass deep nesting, nested index traversal, and stub-only
  method/closure/format/custom-enum boundaries. Several
  would reject the public red implementation, but remain inside its already-failing
  semantic group and do not change the published 8/8 group outcome. One public-library
  assertion also now uses the already frozen `Semantic Analysis Error:` phase prefix
  instead of the mistaken `Semantic Error:` fragment; it was unreachable under the
  red false accepts. The focused target passes 16/16, including all preservation
  controls. The exact `./tools/test.sh` passes formatting, correctness Clippy, 139
  library tests, 148 binary tests, every integration target, and doc tests; only the
  established 38 Phase 5 tests remain ignored.
- Implementation review/publication: three independent reviewers approved exact diff
  `3a909f5813def06d4f7cfb27f8650908410ac724` and tree
  `3effac84a84d56f43abcf99c65161c3da7753d6e` with no P0-P3 findings. Public commit
  `3f0578d69926e15a81c4d8fa6105c99c982cbe02` passes both compiler-test jobs,
  stable/nightly Rust, all three CodeQL analyses, and aggregate CodeQL.
- Acceptance closure: three fresh independent reviewers approved exact record diff
  `a8e4059e71991c9d7a274234f91dd225bea61c01` and tree
  `19fea4153397958656b57adac6b70556d4a997c9` with no P0-P3 findings. Public closure
  commit `5d7aae0f5626813249b6de983a229dbbb1e4fef8` passes both compiler-test jobs,
  stable/nightly Rust, all three CodeQL analyses, and aggregate CodeQL.
- Status: complete; accepted at public closure head `5d7aae0`. The four-record
  final-state sync is public and green at `c612f3b` after exact three-review approval.

## AUDIT-022 — Re-rank public claims and dormant-test risk after CORE-015

- Scope: read-only recheck at clean accepted public head
  `c612f3bea133f308cd71c6f8e5fb9ad708e51e6b`; upstream `master` remains
  `8f8c7337a4008082fd2a443fcc814b5847b8663f`. Draft PR #4 is mergeable and all
  eight checks pass.
- Version evidence: Cargo metadata reports package `compiler 0.3.0`; standalone
  `-v` and `--version` both print `Aero compiler version 1.0.0` with status zero;
  the no-command path prints `Aero Programming Language Compiler v1.0.0` before
  help and exits two. Bare `version` remains an unknown command with status two.
- Conformance evidence: the command passes three example cases and four checks named
  lexer/parser/IR/lowering determinism. Source inspection confirms deterministic
  repeated-output comparison, not mechanized formal semantics. Console/help/BUILD
  call them mechanized/formal, while `CONFORMANCE_PLAN.md` already records the
  narrower truth.
- Safety/coverage evidence: README presents generics/trait bounds/where clauses and
  a borrow checker as current features. The type-system, ownership, and Tutorial 3
  documents claim compile-time memory-safety enforcement despite absent lifetime
  provenance and mutable references classified Copy. Four collection/string task or
  demo records lack the historical notices already present on struct/enum records;
  `todo.md` presents completed phases and `1.0.0` as current.
- Additional current-surface evidence: `CLAUDE.md` labels Phase 5, borrow-checker
  enforcement, generics, and traits complete. Tutorial 1 calls the command a formal
  suite and its four deterministic repetitions mechanized checks, then presents
  ownership as an active memory-safety feature. Tutorial 2 repeats that safety claim;
  Tutorial 4 already carries an explicit implementation boundary.
- Ignored-test evidence: `cargo test --manifest-path src/compiler/Cargo.toml --test
  phase5_tests -- --ignored --test-threads=1` runs 38 tests, with 36 pass and 2 fail.
  `test_semantic_cannot_mutate_through_immutable_ref` relies on recovery parsing that
  drops an unsupported assignment before semantics; `test_semantic_trait_method_borrows_self`
  expects accepted struct/method/borrow behavior outside frozen support. Passing
  ignored tests may share recovery/stub assumptions, so bulk activation would create
  false coverage rather than controlled behavior.
- Ranking: R-004 remains conceptually critical but needs unfrozen ownership/
  provenance across more than two phases. Residual R-002 custom/contextual
  annotation enforcement is stopped because an arbitrary name can denote a nominal
  type or in-scope generic and needs a separately frozen resolution/substitution
  contract. Remaining R-011 aggregate execution needs typed aggregate IR, bounds,
  layout, and backend work. Full R-012 needs test-by-test recovery/stub classification
  rather than bulk activation. R-006, R-009, and R-010 are broader architectural or
  cross-phase work; R-007 requires unavailable device evidence; R-016 is
  medium/medium. R-008 is the highest bounded active public false claim and can be
  corrected without inventing language semantics.
- Selection: `CORE-016` makes Cargo's existing package version the CLI presentation
  source, distinguishes the language v1 design target, labels repeatability checks
  accurately, and adds visible status notices to current and historical documents.
  No package version, parser/type/ownership semantics, conformance algorithm/report
  schema, backend, benchmark, registry, release, external artifact, or master change
  is selected.
- Commands: `git status -sb`; `git rev-parse HEAD`; `gh pr view 4 --json ...`;
  direct CLI version/no-command/conformance invocations; Cargo metadata; targeted
  `rg`; ignored Phase 5 test execution; source and documentation inspection.
- Changes/artifacts: none during the audit. The ignored-test log is local temporary
  evidence only. No benchmark, registry, release, device, or external artifact was
  created or published.
- Status: complete; reproduction, comparison, frozen direction, and stop boundary are
  recorded before new tests or production changes.

## CORE-016 — Make public version, conformance, and safety claims evidence-based

- Problem: the compiler package is `0.3.0`, but its version routes and banner present
  `1.0.0`. Three example conformance cases and four deterministic repeatability checks
  are described as formal/mechanized proof. Current-facing documents present parsed,
  shallow, or dormant generics/ownership features as an enforced borrow checker and
  memory-safety guarantee. These are active public false claims under R-008.
- Priority: P1 public correctness / R-008 HIGH-HIGH. Unlike adjacent ownership,
  architecture, span, grammar, ignored-test, or backend gaps, this correction needs
  no unfrozen language semantics and touches only one CLI presentation surface plus
  documentation.
- Dependencies: accepted `CORE-015` closure `5d7aae0`, accepted final-state sync
  `c612f3b`, completed `AUDIT-022`, and proposed `DEC-021`.
- Frozen version contract: keep `src/compiler/Cargo.toml` and `Cargo.lock` unchanged.
  At compile time, `env!("CARGO_PKG_VERSION")` supplies the implementation version.
  `-v` and `--version` print exactly `Aero compiler version <package-version>` and
  exit zero. No command prints a hard-coded implementation version. With no command,
  the first line is exactly `Aero Programming Language Compiler v<package-version>`,
  followed by existing help, with status two. Bare `version` remains unknown/status
  two. Do not choose or publish a new version.
- Frozen design contract: preserve the consolidated language `v1.0.0` material as a
  future/historical design target. Add a prominent notice that it is not current
  package version, implemented conformance, compatibility promise, or release
  evidence. Add equivalent design-only notices to the type-system and ownership
  documents and Tutorial 3. Preserve their content; no specification rule changes.
  Replace `CLAUDE.md`'s completed Phase 5 claims with a concise evidence-status
  section aligned to the accepted capability audit.
- Frozen conformance contract: retain the existing three cases, four checks,
  algorithms, order, counts, statuses, JSON shape, and Rust/JSON names containing
  `mechanized`. Console summary uses `Determinism checks: 4/4 passed`; help and BUILD
  call them deterministic regression checks. Do not call the command formal proof or
  mechanized semantics. `CONFORMANCE_PLAN.md` is unconditionally frozen.
- Current/history documentation contract: README's current-surface table must state
  that generic/trait/where syntax is parsed or quarantined rather than enforced; that
  ownership/borrow syntax and shallow tracking do not establish a borrow checker,
  lifetime safety, or memory-safety guarantee; and that conformance is three examples
  plus four deterministic regression checks. BUILD drops a fixed version header and
  uses the same conformance classification. Tutorial 1 uses the same conformance
  language and Tutorial 1/2 mark ownership/safety follow-ons as conceptual design
  material; Tutorial 4's existing implementation-boundary notice remains a control.
  `todo.md` becomes a visible historical planning snapshot rather than current
  completion/version evidence. Add visible
  historical-helper notices to `docs/demos/builtin_collections_demo.md`,
  `docs/demos/collection_string_demo.md`,
  `docs/tasks/TASK_10_3_COLLECTION_STRING_GENERATION_SUMMARY.md`, and
  `docs/tasks/TASK_11_BUILTIN_COLLECTIONS_LIBRARY_SUMMARY.md`. Existing struct/enum
  notices remain required controls.
- Red tests: add `src/compiler/tests/version_claim_contract_tests.rs`. It must derive
  the expected package version from the compiling package; process-test `-v`,
  `--version`, no-command, and bare `version` outputs/statuses; prove no hard-coded
  literal implementation-version presentation by statically binding main's exact
  `env!("CARGO_PKG_VERSION")` interpolation; bind the conformance console/help labels
  and unchanged 3/4 counts; serialize a report and preserve the compatibility
  `mechanized_*` fields; verify README/BUILD/CLAUDE/Tutorial 1/Tutorial 2 current-
  surface language; require design-target notices on the consolidated/type/ownership/
  Tutorial 3 documents and retain Tutorial 4's current implementation notice;
  require archive notices on todo and all eight claim-heavy
  struct/enum/collection demo/task records; and retain the current experimental/no-
  stability boundary. Existing `cli_status_contract_tests.rs` version assertions
  must remain exact but derive their expectation from the package version; their
  status and extra-operand contracts must not weaken.
  Before production/docs edits, only the new frozen negative assertions may fail and
  every prior test must pass.
- Full acceptance: focused red/green target, exact complete `./tools/test.sh`, and
  three independent exact diff/tree reviews at preregistration, tests-only red,
  implementation, and closure. Each published green checkpoint must pass both
  compiler-test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate
  CodeQL. Public red must reproduce only the intended new target failures.
- Files allowed: new `src/compiler/tests/version_claim_contract_tests.rs`;
  only the three existing version-string expectations in
  `src/compiler/tests/cli_status_contract_tests.rs`; `src/compiler/src/main.rs`;
  `README.md`; `BUILD.md`; `CLAUDE.md`; `todo.md`;
  `docs/language/aero_formal_language_specification.md`;
  `docs/language/aero_type_system.md`; `docs/language/aero_ownership_borrowing.md`;
  `tutorials/01-getting-started.md`; `tutorials/02-core-features.md`;
  `tutorials/03-ownership-borrowing.md`; the four collection/string demo/task records
  named above; and minimal task, decision, risk, capability, matrix, and project
  control records.
- Files frozen: Cargo manifest/lock/dependencies; lexer, parser, AST/type system,
  semantic analyzer, IR, verifier, code generator, module/cache, CLI status taxonomy
  beyond the frozen presentation strings, conformance cases/check algorithms/report
  schema, `CONFORMANCE_PLAN.md`, ownership implementation, backend/workflow,
  examples/scaffold, registry,
  benchmarks/claims evidence, release/tag/package state, external state, and `master`.
- Risks: compile-time version interpolation can drift between binary and docs; a
  documentation correction can accidentally rewrite the design target; compatibility
  fields can be renamed; accurate qualifications can be buried or contradicted by a
  table; historical helpers can still read as current support; or changing help text
  can alter established status/output structure. Tests must bind exact routes,
  headings/notices, compatibility fields, counts, and experimental status.
- Stop conditions: any need to choose/change a language or package version, change
  Cargo metadata, add a command, rename report fields, alter conformance algorithms,
  change parser/type/ownership semantics, delete design history, claim formal proof,
  modify a backend/workflow, run or publish a benchmark, release/package/register,
  touch a third compiler phase, or change `master`. Stop rather than infer policy.
- Owner: lead-owned bounded CLI/docs truth slice. Independent type, IR/codegen, and
  backend/claim reviewers must approve the exact preregistration, tests-only red,
  implementation candidate, and acceptance closure before each publication.
- Preregistration review/publication: the first six-record snapshot was rejected by
  all three reviewers for an impossible existing-test scope, omitted CLAUDE/Tutorial
  claims, a weak package-source assertion, ambiguous `CONFORMANCE_PLAN.md` handling,
  and incomplete residual-risk ranking. The corrected snapshot adds the missing
  surfaces, narrows the existing-test permission, binds static package interpolation,
  freezes the plan, and records R-002/R-011/R-012 stops. Three reviewers approved
  exact diff `321fb61c3932cd0663bc5bcbc0aecb02361ab010` and tree
  `4933dc2e9297cc5d7d0742c28081571e3fc23c5f` with no P0-P3 findings. Public commit
  `1575914e7ab1f3c70793c77a1d82b7b3a78bb441` passes both compiler-test jobs,
  stable/nightly Rust, all three CodeQL analyses, and aggregate CodeQL.
- Tests-only red evidence: new `version_claim_contract_tests.rs` contains seven
  groups. The conformance-count/schema preservation group and repository-
  experimental-status group pass. Exactly five groups fail on the frozen gaps: both
  version flags expose `1.0.0` instead of package `0.3.0`; console/help/BUILD retain
  formal/mechanized wording; README/CLAUDE/Tutorial 1/2 retain current unsupported
  claims; four normative/tutorial records lack design-target notices; and todo plus
  four collection/string task/demo records lack historical notices. The focused
  target is exactly 2 passed / 5 failed.
- Red review correction: the first staged red snapshot was rejected because an
  append-only disclaimer could leave exact README/CLAUDE/Tutorial contradictions,
  a dead `env!` plus a literal version constant could spoof package sourcing, and a
  buried footer could satisfy an unbounded notice search. The corrected target
  explicitly rejects each identified stale claim, rejects the current package
  version literal anywhere in `main.rs`, and requires design/archive notices within
  the first 12 lines while normalizing wrapped prose. These additions stay inside
  the same five failing groups; the focused target remains exactly 2 passed / 5
  failed.
- A second exact review found that four additional obsolete CLAUDE ownership/syntax/
  test-count lines and the completed-phases heading could survive beside a new status
  section. The current-surface group now rejects that entire stale Phase 5 status
  block; this remains within the same intended failing group.
- Red full-gate evidence: after formatting the new test mechanically, exact
  `./tools/test.sh` passes formatting and correctness Clippy, 139 library tests, 148
  binary tests, and all prior integration targets, with the established 38 Phase 5
  tests remaining ignored, before stopping only on the new 2/5 target. A separate Cargo
  `--no-fail-fast` run reproduces the same sole failing target and proves doc tests
  pass. No production or public-claim file changed.
- Red review/publication: all three independent reviewers approved exact staged diff
  `b734773e6f1f4bb9c9561dc089e72b103e3b4e25` and tree
  `488687b20c882c78c8e801d46cdb0bf817d7f421` with no P0-P3 findings. Public commit
  `4b94dbd55465d2f94c2e7840f26ce5f73e571f30` reproduces 2 passed / 5 failed in both
  compiler-test jobs and nightly Rust. The stable matrix job reached its test step
  before fail-fast cancellation; all three CodeQL analyses and aggregate CodeQL pass.
  CI runs `30791641180`, `30791643961`, and Rust run `30791643936` preserve the
  intended red evidence without a production change.
- Implementation candidate: `main.rs` now derives both version routes and the
  no-command banner from `env!("CARGO_PKG_VERSION")`; the existing CLI status target
  derives its three exact expectations from the same package metadata. Console/help/
  BUILD describe the unchanged four checks as deterministic regression checks.
  Current README, CLAUDE, and Tutorial 1/2 claims are bounded to audited behavior;
  language-design and Tutorial 3 notices distinguish the v1.0.0 target; todo and four
  claim-heavy collection/string records are visibly historical. Cargo metadata,
  conformance algorithms/count/order/schema, `mechanized_*` compatibility names,
  language semantics, and backend/release state are unchanged.
- Green-test correction: the archive-intro normalizer ignores a standalone Markdown
  blockquote marker (`>`). Public red never reached this pre-existing wrapped Task
  10.2 notice because the same group failed earlier; the correction preserves the
  frozen first-12-lines intent without weakening any required text or moving a
  failure. This is disclosed as an implementation-side preservation correction.
- Local verification: `version_claim_contract_tests` passes 7/7; the complete
  `cli_status_contract_tests` target passes 7/7; exact `./tools/test.sh` exits zero
  after formatting/correctness Clippy, 139 library tests, 148 binary tests, every
  integration target including the new 7/7 target, and doc tests. The established 38
  Phase 5 ignored tests remain unchanged.
- Implementation review/publication: the type/safety, IR/codegen, and backend/claim
  reviewers independently approved exact canonical staged diff
  `e0c2bbb61f33ea53e1c07d472a21a631170c22e7` and tree
  `8d5ba37b0a58c715cf72721ade23471c5fa4fa7c` with no P0-P3 findings. Public commit
  `cc984d0afe4c63f3c322f8da7c34fc666f8ec072` passes both compiler-test jobs,
  stable/nightly Rust, all three CodeQL analyses, and aggregate CodeQL. CI runs are
  `30792532836`, `30792536017`, Rust `30792536010`, and CodeQL `30792533602`.
- Closure review/publication: all three independent reviewers approved exact
  record-only diff `7b24a58e7475700423dc66da368a22b97f9c31e8` and tree
  `4c7f526617ecb8e3a0c28622f8eca44dac627981` with no P0-P3 findings. Public closure
  `ea036f2e71a4f67b1f8c6f711488f02f65fc4ad5` passes both compiler-test jobs,
  stable/nightly Rust, all three CodeQL analyses, and aggregate CodeQL. CI runs are
  `30793042965`, `30793045668`, Rust `30793045614`, and CodeQL `30793042681`.
- Status: complete and accepted at public closure `ea036f2`. R-008 is controlled for
  the selected version/conformance/current-design-history public-claim boundary;
  actual ownership/type/backend capability gaps remain separately open under their
  existing risks. This final-state sync changes records only.

## AUDIT-023 — Classify ignored Phase 5 evidence after CORE-016

- Scope: read-only recheck at clean accepted public head
  `8869ecab0a7aadb51d9da193bf480a6fa97a9b3e`; upstream `master` remains
  `8f8c7337a4008082fd2a443fcc814b5847b8663f`. Draft PR #4 is open, mergeable, and
  all eight checks pass.
- Reproduction: `cargo test --manifest-path src/compiler/Cargo.toml --test
  phase5_tests -- --ignored --test-threads=1` runs 38 tests and reproduces exactly 36
  passed / 2 failed. `test_semantic_cannot_mutate_through_immutable_ref` loses an
  unsupported assignment through compatibility parser recovery and analyzes an empty
  AST. `test_semantic_trait_method_borrows_self` reaches unsupported struct/method
  behavior before the claimed receiver-borrow property.
- Inventory: the target contains 4 lexer, 20 parser-shape, and 14 semantic tests. All
  38 use recovery `lexer::tokenize`; all 20 parser tests and all semantic tests use
  compatibility `parser::parse`, which prints a parse error and returns an empty AST
  on failure. The 14 semantic tests contain broad `is_ok`/`is_err` assertions.
- Semantic classification: only five passing negatives reach the named current
  shallow control (use-after-move, call move, mutable/immutable conflict, double
  mutable conflict, and missing trait method). Two passing positives are at most
  narrow shallow-state companions. Two other positives do not exercise the named
  ownership transfer/borrow registration, and three passing negatives are unrelated
  unsupported-struct false positives. Both failing tests are recovery/unsupported
  confounded. All 14 remain quarantined; none establishes lifetimes, provenance,
  borrow release, generic substitution, trait dispatch, or memory safety.
- Syntax classification: four lexer tests use length or negative-token assertions;
  18 parser tests can bind exact retained token/AST shape through strict
  `try_tokenize_with_locations` and fallible `parse_with_locations`. Bound tests must
  assert exact ordered `trait_bounds`, enum tests exact payload types, and borrow/
  dereference tests exact operands. Two generic-impl tests remain quarantined because
  parser/AST currently skip target type arguments and discard impl bounds; one test's
  comment also names a bound absent from its source.
- Ranking: R-004 remains highest conceptual severity but stops on an unfrozen CFG/
  lifetime/provenance model spanning more than two phases. R-012 is the highest
  bounded action: test/evidence-only strict syntax classification. R-007 is the next
  audit-only claims/evidence priority but device closure needs unavailable hardware.
  R-010/R-006/R-009 are broad grammar/architecture/span work; R-011 remains partially
  controlled; R-016 remains medium/medium and needs a support policy.
- Selection: `CORE-017` activates exactly 4 strict lexer and 18 strict parser-retention
  tests, leaves exactly 14 semantic plus 2 generic-impl tests ignored, and updates
  three current evidence documents. R-012 may become only partially controlled; 299
  dormant tests and overlapping binary/library test compilation remain excluded.
- Changes/artifacts: none. No benchmark, registry, release, device, or external
  artifact was created or published.
- Status: complete; the reproduction, per-test classification, conservative 22/16
  split, residual ranking, and stop boundary precede any test or documentation edit.

## CORE-017 — Recover strict Phase 5 syntax evidence without semantic uplift

- Problem: all 38 Phase 5 tests are ignored and enter through recovery helpers. The
  36/2 ignored result mixes useful syntax shape, genuine shallow controls, positive
  no-error smoke, unrelated-error false positives, and two confounded failures.
- Priority: P1 evidence correctness / R-012 HIGH-HIGH. This is the highest bounded
  remaining correction because it changes one test target and evidence documents
  without choosing ownership, generic, trait, grammar, IR, or backend semantics.
- Dependencies: accepted `CORE-016` closure `ea036f2`, accepted final-state sync
  `8869eca`, completed `AUDIT-023`, and proposed `DEC-022`.
- Frozen active set: activate exactly 22 existing tests—4 lexer and 18 parser tests.
  Use strict located lexing and fallible located parsing. Lexer tests assert entire
  token streams including EOF. Parser tests assert exact retained AST names, types,
  operands, mutability, payloads, receivers, bodies, ordering, and flattened ordered
  function `trait_bounds`. Names/comments must say strict token or retained parser
  shape, not enforcement or execution.
- Frozen quarantine: exactly 16 tests remain ignored—all 14 semantic tests plus
  `test_parse_generic_impl_block` and
  `test_parse_impl_trait_for_generic_struct`. Each ignore reason must state its
  recovery/unsupported/unfrozen semantic or target-argument/bound-retention blocker.
  Do not execute or report the 16 as acceptance passes.
- Exact activatable parser set: immutable/mutable reference parameter; immutable/
  mutable borrow expression; dereference expression; generic function; multiple-
  parameter generic function; generic struct; generic enum; generic type annotation;
  single/multiple-method trait definitions; impl Trait for a non-generic type;
  single/multiple inline bounds; where-clause bounds; generic struct/reference field;
  and generic function/bound/reference combination.
- Documentation contract: update only current evidence lines in `CLAUDE.md`,
  `FRAMEWORK_ALIGNMENT.md`, and `Roadmap.md` from all-38-ignored wording to exactly 22
  active strict syntax-retention tests and 16 quarantined tests. State explicitly that
  this is parsed-only evidence and not ownership, borrow-checker, generic/trait
  enforcement, execution, conformance, or stability evidence. Preserve historical
  checkpoint statements elsewhere.
- Allowed files: `src/compiler/tests/phase5_tests.rs`; `CLAUDE.md`;
  `FRAMEWORK_ALIGNMENT.md`; `Roadmap.md`; and minimal task, decision, risk, capability,
  matrix, and project control records.
- Frozen files/surfaces: every `src/compiler/src/**` production file; Cargo manifest/
  lock/dependencies; all other tests; tools/test.sh; workflows; grammar/spec rules;
  lexer/parser/AST behavior; semantic/ownership/borrow/trait/generic implementation;
  IR/codegen/execution/layout/ABI; backends/devices; README/BUILD; benchmark, registry,
  release/version/package state; external artifacts; and `master`.
- Stop conditions: any selected test fails strict lex/parse or needs production
  changes; any need to retain generic-impl target arguments/bounds; any test count
  other than exactly 38 total / 22 active / 16 ignored; any weakening/deletion; or any
  claim that syntax retention establishes semantics or execution. Stop and re-audit
  rather than implement a parser, AST, semantic, or backend fix.
- Acceptance: `--list` proves exactly 38 tests; default focused target is exactly 22
  passed / 0 failed / 16 ignored; `--ignored --list` identifies exactly the frozen 16
  without running them; exact `./tools/test.sh` passes; three independent exact diff/
  tree reviews approve preregistration, implementation, and closure; each published
  green checkpoint passes both compiler-test jobs, stable/nightly Rust, all three
  CodeQL analyses, and aggregate CodeQL. No artificial red checkpoint is required
  because this is evidence reclassification with production behavior frozen.
- Owner: lead-owned test/evidence slice with independent type/safety, IR/parser, and
  backend/claim review at every publication boundary.
- Preregistration review/publication: all three independent reviewers approved exact
  six-record diff `ebe348e00721596f768b900547b9d19b56e44df4` and tree
  `1d890b93351e54fb6903aa952957494a517d40a9` with no P0-P3 findings. Public commit
  `2c61535092f22f2f513aac0fcee9d34d9c621212` passes both compiler-test jobs,
  stable/nightly Rust, all three CodeQL analyses, and aggregate CodeQL. CI runs are
  `30794999601`, `30795002178`, Rust `30795002200`, and CodeQL `30794999815`.
- Implementation candidate: one shared strict-token helper and one strict-parse
  helper make the selected tests fail closed. Four lexer tests now assert complete
  token streams including EOF and the `&` versus `&&` distinction. Eighteen parser
  tests bind exact retained references/borrow/deref operands, generic parameter and
  enum payload types, trait receivers/method bodies, ordered inline/where bounds, and
  combined reference/bound shapes. Names say strict token or retained shape. No
  production source changed.
- Quarantine implementation: exactly 16 `#[ignore = "quarantined: ..."]` attributes
  remain on the 14 semantic and two generic-impl tests, with per-test blockers. The
  three current evidence documents say 22 syntax-retention tests are active and 16
  remain quarantined, and explicitly deny semantic/execution evidence.
- Focused evidence: default `phase5_tests` is exactly 22 passed / 0 failed / 16
  ignored. `--list` reports exactly 38 entries. `--ignored --list` reports exactly the
  frozen 16 names without executing them. Inventory source counts are 4 strict lexer,
  18 strict parser, and 16 ignored.
- Full-gate evidence: exact `./tools/test.sh` exits zero after formatting,
  correctness Clippy, 139 library tests, 148 binary tests, every integration target
  including Phase 5's 22 passed / 16 ignored result, and doc tests.
- Implementation review/publication: the first exact snapshot was rejected by the
  IR/parser reviewer because the non-generic impl test accepted any nonempty method
  body. The assertion was tightened to exactly one `return "point";` statement and no
  tail expression, all evidence gates were rerun, and no prior approval was reused.
  The type/safety, IR/parser, and backend/claim reviewers then independently approved
  corrected exact diff `a417c7e3c076e7ff6951ce9c181ea99d6bdfa3b6` and tree
  `83bf4f0ba8f973e7ec39167e53114cf5714fd03b` with no P0-P3 findings. Public commit
  `8be8c21696cf98602c82e1e5e4fdfc6bf10e9777` passes both compiler-test jobs,
  stable/nightly Rust, all three CodeQL analyses, and aggregate CodeQL. CI runs are
  `30796167886`, `30796170222`, Rust `30796170162`, and CodeQL `30796168359`.
- Closure review/publication: all three independent reviewers approved exact
  record-only diff `3239da0b313f819bad7beef69cea8b6bd5e658a8` and tree
  `166ec7a5e4156da1cefeb9f921a31714461c6839` with no P0-P3 findings. Public closure
  `3dd3bb41d601ddfe5f7ac2722cde39bad124973d` passes both compiler-test jobs,
  stable/nightly Rust, all three CodeQL analyses, and aggregate CodeQL. CI runs are
  `30814475780`, `30814478296`, Rust `30814478178`, and CodeQL `30814475319`.
- Status: complete and accepted at public closure `3dd3bb4`. R-012 is partially
  controlled for the selected strict Phase 5 syntax-evidence classification; Cargo
  overlap, 299 dormant tests, and every semantic capability claim remain open. This
  final-state sync changes records only.

## AUDIT-024 — Re-audit R-007 execution and backend claims after CORE-017

- Task ID/date/basis: `AUDIT-024`, 2026-08-03, clean public
  `9ddc571ac47f1c2ffcf7a737e4be442f01c0f78b`, tree
  `20ab4e6b87ead659a138e57bc27c073f817d15cb`; integration branch exactly matches
  origin, PR #4 is open/draft/mergeable, all eight checks pass, and upstream master
  remains `8f8c733`.
- Observed behavior: CPU `run` verifies, objects/links, executes a host process, and
  passes through its status. ROCm `run` requires verification and can invoke `llc`
  for a temporary AMDGPU object, prints object validation, never checks that object
  exists, never links or launches, then returns status zero. CUDA returns operational
  `1`. The `gpu` selector uses environment/tool presence and can silently choose CPU.
- Claim finding: graph compilation writes externally verified LLVM containing
  internal scalar-`double` helpers; quantization writes externally verified scalar-
  `double` helpers using fixed/default or sample-derived scales. Names, comments,
  counters, and backend labels are not device execution, real FP8, per-channel
  execution, numerical correctness, or hardware calibration. Current CLI help/
  reporting, README, BUILD, Tutorial 1, quantization notes, and enabled Aero ROCm
  GGUF configuration exceed that evidence.
- Preserved evidence: all 27 paths declared by `claim-verification/claims.json` exist.
  Its GGUF result is accurately external llama.cpp reference evidence and its Aero
  GPU claims remain blocked. `claim-verification/**`, formal design specifications,
  and experimental implementations are frozen.
- Verification: root ran the seven-test `cli_status_contract_tests` target (7/7),
  graph unit filter (3/3 in library and 3/3 in binary), and quantization unit filter
  (5/5 in library and 5/5 in binary). Three independent read-only auditors traced
  type/safety, IR/codegen, and backend/claim boundaries and unanimously classified
  the ROCm zero-status route as P1 false success. No files changed and no hardware,
  benchmark, or artifact command ran.
- Remaining uncertainty: no ROCm/CUDA device was probed; object usability, graph
  semantic equivalence, and quantization numerical correctness remain unproved.
- Risk/recommendation: R-007 remains OPEN HIGH/HIGH. Take the bounded tests-first
  fail-closed/status/claim slice below. Do not infer device capability from a green
  CPU/LLVM gate.
- Status: complete, read-only; result commit none.

## CORE-018 — Fail object-only execution closed and reclassify backend claims

- Task ID/owner: `CORE-018`; lead-owned CLI/backend-claim vertical slice with
  independent type/safety, IR/codegen, and backend/claim review at every publication
  boundary.
- Observed behavior: explicit ROCm `run` can report zero without execution; `llc`
  success lacks an object postcondition; `gpu` is an ambiguous heuristic target;
  current graph/quant/GGUF wording exceeds the implementation and immutable evidence.
- Hypothesis: exact status, postcondition, target-selection, and claim contracts can
  control the false-success surface while preserving all experimental transforms and
  avoiding device or numerical semantics.
- Frozen execution semantics: CPU `run` and delegated child status are unchanged.
  ROCm may invoke `llc` to emit a temporary target object from externally verified
  LLVM, but must require the requested `.o` path to be a regular file before reporting
  the stage. Regular-file existence is only an emission postcondition, not object
  validity or usability. The exact stage line is `ROCm object stage complete: llc
  produced a temporary file; no link or execution occurred.` ROCm must then return
  operational `1` with the exact diagnostic `ROCm run is unavailable: HIP link and
  device launch are not implemented; no program was executed.` A zero-status `llc`
  without a regular output file fails with exact diagnostic `ROCm object generation
  failed: llc reported success but did not create the requested regular object file.`
  CUDA remains operational `1`, says object/link/device launch are unavailable and no
  program executed, and recommends CPU only. Preserve existing cleanup attempts and
  cleanup-error precedence; tests prove no temporary artifacts remain on covered
  ordinary success/error paths.
- Frozen target semantics: both public spellings `--target gpu` and `--backend gpu`
  are rejected for both build and run with invocation status `2` and the exact core
  diagnostic `target \`gpu\` is ambiguous and does not prove a usable device; choose
  cpu, rocm, or cuda explicitly`. Explicit targets remain accepted; internal auto-
  detection code is preserved but unused by these public routes.
- Frozen claim semantics: graph instruction/helper bodies and existing report fields/
  schema/values remain unchanged. Only additive non-semantic LLVM comments plus CLI
  stdout/help may add `execution_scope=internal-scalar-helper` and
  `device_execution=false`; current docs use the same stage terms. Quantization
  instruction/helper bodies, report schema/field names/counts, and every non-`notes`
  report value remain unchanged. Wording-only changes to `QuantizationReport.notes`
  are the sole report-value exception; additive non-semantic LLVM comments, CLI
  stdout/help, those notes, and current docs may describe scalar-double helper
  transformation with default/sample-derived scaling and explicitly deny device
  execution, real FP8 representation, per-channel execution, and numerical proof.
  The quantization claim test binds this exact notes exception. The example
  `aero_rocm` GGUF backend is disabled with a reason because its source/arguments/
  execution path do not exist; external backends and evidence are preserved.
- Tests-first red contract: add two ROCm fake-tool tests—one `llc` writes the requested
  regular object and one returns zero without it—plus one exact ambiguous-`gpu`
  rejection test that matrices build/run × `--target`/`--backend`, all at status `2`,
  to `cli_status_contract_tests.rs`. Before implementation that target must be exactly
  7 passed / 3 failed. Add `backend_claim_contract_tests.rs` with exactly seven tests:
  two green preservation tests for the formal design-target notice and immutable
  external-GGUF qualification, and five intended failures binding CLI stage/help,
  current README/BUILD/tutorial wording, graph telemetry, quantization boundaries,
  and the disabled Aero GGUF example. The red target must be exactly 2 passed / 5
  failed. `--no-fail-fast` must prove every other target and doc tests green; public
  red may fail only both compiler-test jobs and stable/nightly Rust while all four
  CodeQL checks remain green.
- Allowed files: `src/compiler/src/main.rs`; wording/adjacent telemetry only in
  `src/compiler/src/graph_compiler.rs` and `src/compiler/src/quantization.rs`;
  `src/compiler/tests/cli_status_contract_tests.rs`;
  `src/compiler/tests/backend_claim_contract_tests.rs`; `README.md`; `BUILD.md`;
  `BACKEND_STATUS.md`; `tutorials/01-getting-started.md`;
  `benchmarks/gguf/README.md`; `benchmarks/gguf/config.rx7800xt.example.json`; and
  minimal updates to the six task/decision/risk/capability/matrix/project records.
- Frozen files/surfaces: Cargo manifests/lock/dependencies; workflows/test runner;
  parser, AST, semantics, checked IR, codegen algorithms, graph/quant algorithms and
  serialized field names; GPU discovery implementation; object/linker flags and
  verifier policy; claim-verification/results; benchmark runners/results; founding
  PDFs/formal spec/grammar; packages/registry/releases; external artifacts; master.
- Acceptance: failing tests first; focused red matrices exactly 7/3 and 2/5; three
  exact diff/tree approvals before every publication; implementation focused targets
  10/10 and 7/7 green; existing CPU child-status, CUDA unavailable, verifier-before-
  publication, graph, and quant controls green; exact `./tools/test.sh` green; then
  all eight public checks green for implementation, closure, and final-state sync.
- Risks: scripts may have relied on incorrect ROCm zero status or heuristic CPU
  fallback; broad wording assertions could erase design-only material; field renames
  could break consumers; shared run changes could disturb CPU status; claiming object
  existence as validity could overstate evidence.
- Stop conditions: any HIP/CUDA ABI, linker, runtime, device discovery, memory
  transfer, synchronization, launch, result comparison, performance run/claim, real
  FP8/per-channel/numerical definition, graph/quant algorithm change, report field
  rename, persistent ROCm artifact, language semantic change, dependency/workflow
  change, more than two compiler phases, unexpected red baseline, external artifact,
  package/release/registry action, or master modification.
- Tests-first evidence: exact three-review-approved diff
  `ee9b26ddf59a41cfb55a4b8df8e23300c14d0696`, tree
  `4a65ecf7325c2b90380f9b7023765ca35145a372`, was published as tests-only commit
  `427fb4c`. Local evidence was exactly CLI 7/3 and claims 2/5, with only those two
  targets red under `--no-fail-fast`. Public compiler runs `30821149904` and
  `30821155003` plus stable/nightly run `30821156690` failed as designed; CodeQL
  run `30821150397` and aggregate `91711261389` passed.
- Implementation candidate: explicit `gpu` rejection, ROCm regular-file/fail-closed
  behavior, CUDA no-execution wording, exact graph/quant stage telemetry, wording-
  only quantization notes, current documentation, and the disabled Aero GGUF route
  are implemented within the allowed files. Focused targets are CLI 10/10 and
  claims 7/7 green; graph/quant bodies, schemas, non-note values, CPU behavior,
  external backends, immutable evidence, and benchmark results remain unchanged.
- Gate evidence: exact Windows Git Bash `./tools/test.sh` passes, including 139
  library, 148 binary, claims 7/7, CLI 10/10, every remaining integration target,
  the explicit 22/16 Phase 5 split, and doc tests.
- Implementation acceptance: exact diff
  `7984dbce4a543223482b628fe7b473cd81a6a628`, tree
  `d10567bec4713c6623772606f4c3ea0a1418f37d`, received three independent approvals
  with no P0-P3 findings and was published as `8bde0ff0189d1636a86757bf20ee3814ec3f932a`.
  Compiler runs `30822531693` and `30822533924`, stable/nightly run `30822533179`,
  CodeQL run `30822528126`, and aggregate `91715952709` all pass.
- Closure acceptance: exact record-only diff
  `3d0a17f75e74446d5db0a132084fb3ca7973c6ed`, tree
  `83c9676f905dde55d5da52ed3961607c2aec9d55`, received three independent approvals
  with no P0-P3 findings and was published as `2e0e17fde6d9b11c2f5705c45b23468e0b04cbf0`.
  Compiler runs `30823259890` and `30823261072`, stable/nightly run `30823260717`,
  CodeQL run `30823257183`, and aggregate `91718428033` all pass.
- Status: complete and accepted at public closure `2e0e17f`; the selected object-only
  false success and current claim surface are controlled. R-007 remains open because
  no Aero accelerator execution or correctness evidence was produced. This final-
  state sync changes records only.
- Final-state sync: corrected exact diff
  `a4034521b5976f4c737871d5be7e93d2a1f34bfb`, tree
  `21e72079679550b73935b56d87e4e062fc48d88e`, received three independent approvals
  with no P0-P3 findings and was published as `d0bd54e93ff9fda9e769dd29abcec02a1f550e9a`.
  Compiler runs `30824106058` and `30824111861`, stable/nightly run `30824110412`,
  CodeQL run `30824105642`, and aggregate `91721342986` all pass.

## AUDIT-025 — Re-rank remaining compiler-integrity risks after CORE-018

- Task ID/date/basis: `AUDIT-025`, 2026-08-03, accepted clean public head
  `d0bd54e93ff9fda9e769dd29abcec02a1f550e9a`, tree
  `21e72079679550b73935b56d87e4e062fc48d88e`; integration branch exactly matches
  origin, PR #4 is open/draft/mergeable, all eight checks pass, and upstream
  `master` remains `8f8c733`.
- Observed behavior: `CORE-018` controls its selected object-only/current-claim
  boundary, while R-004, R-006, R-007, R-009, and R-010 remain open; R-002, R-005,
  R-011, and R-012 retain explicit partial boundaries; and R-016 remains lower
  likelihood/impact. The next implementation slice has not been selected.
- Hypothesis: independent clean-head reproduction and phase/scope comparison can
  identify the highest-severity bounded active false success without inventing
  ownership, aggregate, grammar, span, backend, or compatibility semantics.
- Frozen semantics: none. This task is read-only evidence collection and ranking.
  Existing accepted contracts, exclusions, compatibility surfaces, capability
  classes, and risk states remain unchanged until a separately reviewed task freezes
  an exact boundary.
- Allowed files/changes: read-only auditors change no files. Root may later record the
  completed audit and a separately frozen next-task contract in the six control
  records; no source, tests, schemas, workflows, dependencies, immutable evidence,
  benchmarks, artifacts, package/release/registry state, or `master` change is
  authorized by this audit.
- Acceptance: reproduce candidate risks at exact clean head; cite file/symbol or line
  evidence and commands; distinguish parser, semantic, IR, codegen, runtime, tooling,
  and documentation stages; compare severity, user reach, semantic ambiguity, phase
  count, and regression-test feasibility; obtain independent type/safety, IR/codegen,
  and backend/claim reports; recommend one bounded next action or report that every
  candidate stops on an unfrozen decision.
- Risks: smoke tests may conceal recovery/stub behavior; public APIs may differ from
  trusted compiler paths; apparently local fixes may cross more than two phases;
  current documentation may be mistaken for semantics; CPU evidence may be mistaken
  for accelerator evidence.
- Stop conditions: any repository mutation outside this ledger preregistration;
  language or ownership decision; hardware/device probe; benchmark run or claim;
  package, registry, release, workflow, dependency, immutable-evidence, external
  artifact, or `master` change; destructive command; or implementation work before a
  bounded tests-first contract receives exact review.
- Findings: `aero test` directly scans `examples`, `tests`, and `.` for the two
  filename suffixes, then reads, strictly parses with direct modules, and performs
  semantic analysis only. It does not admit checked IR, generate code, execute, or
  compare runtime results, but its comment, progress/success/summary output, help,
  BUILD description, and one CLI control say `run`, `Running`, or `passed`.
- Ranking: two auditors select that direct user-facing P1 first; the IR/codegen
  auditor selects silently ignored nondefault public `CompilerOptions` first. The
  lead selects the documented CLI claim because it has direct user reach and needs
  presentation/tests only; fail-closed nondefault options are the bounded runner-up.
  R-002/R-004/R-005/R-007/R-009/R-010/R-011/R-012/R-016 stop or defer for the
  semantic, compatibility, hardware, architectural, evidence-scope, or policy reasons
  recorded in the current audit/risk documents.
- Verification at `AUDIT-025` basis head `d0bd54e`: root confirmed the unused
  `_options` facade and 62 default-only
  repository calls; focused checked-IR 6/6, fatal-parse 11/11, and module-pipeline
  7/7 pass. Independent focused evidence includes binding 16/16, active frontend
  21/21, CLI 10/10, and backend claims 7/7. No benchmark or hardware ran.
- Remaining uncertainty: external nondefault-option consumers were not inventoried;
  ownership/provenance, nominal/generic resolution, aggregate bounds/layout, and
  executable-test semantics remain unfrozen. One auditor's standalone Windows
  options probe failed at linking and produced no claimed runtime result.
- Status: complete, read-only; result commit none. Recommend `CORE-019` below.

## CORE-019 — Make semantic-only `aero test` presentation truthful

- Task ID/owner: `CORE-019`; lead-owned CLI presentation/tests slice with independent
  type/safety, IR/codegen, and backend/claim review at every publication boundary.
- Problem/priority: P1 user-facing correctness / R-013 HIGH-HIGH. The command analyzes
  source but claims files run and pass, so successful analysis can be mistaken for
  test execution. Accepted DEC-016 already classifies it as a semantic checker.
- Dependencies: accepted `CORE-018` final sync `d0bd54e`, complete `AUDIT-025`, and
  proposed `DEC-024`.
- Frozen behavior: preserve command name and exact arity; direct, nonrecursive scan of
  `examples`, `tests`, and `.` in current order; `_test.aero`/`_tests.aero` suffixes;
  read, strict parse, direct-module collection, and semantic analysis; discovered,
  completed, failure, and total counts; status `0` for no sources/all successful
  analyses, `1` for any read/parse/module/semantic failure, and `2` for bad invocation.
- Frozen visible wording: initial `Analyzing Aero test sources (parse, direct modules,
  semantics only; no execution)...`; per source `Analyzing <path>`; success `<name>
  analysis completed (not executed)`; failure `<name> analysis failed: <diagnostic>`;
  summary `analysis result: <completed> completed, <failed> failed, <total> total; no
  tests were executed`; empty warning `no Aero test source files found
  (*_test.aero, *_tests.aero); no tests were executed`. Existing ANSI styling may
  wrap labels/checkmarks. Successful command output may not use `Running`, `passed`,
  `test result`, `Compiling test suite`, or `Program executed successfully`.
- Help/docs: exact help description `Discover and semantically analyze *_test.aero
  files (no execution)`. BUILD describes both suffixes and states no test execution
  or IR generation. README's command inventory is unchanged because it makes no
  execution claim.
- Tests first: change only `src/compiler/tests/cli_status_contract_tests.rs`. Update
  the existing successful-command control and add one exact semantic-only presentation
  contract covering success, empty discovery, help/BUILD, forbidden wording, status
  preservation, and a failure-summary control. With production/docs unchanged, the
  target must be exactly 9 passing / 2 failing tests; every other target remains at
  baseline. Exact three-review approval precedes publication of that red checkpoint.
- Implementation files: `src/compiler/src/main.rs`, `BUILD.md`, the tests file, and
  current state/capability/decision/risk/matrix/ledger records only. Smallest complete
  change; no other file is authorized.
- Acceptance: preregistration passes the exact full gate and all eight public checks.
  The tests-only target is exactly 9/2 red; both compiler-test checks fail only on
  those two contracts, Rust CI concludes non-green with each matrix job that reaches
  `cargo test` failing only there (normal fail-fast cancellation is permitted), and
  all four CodeQL checks pass. Implementation is 11/11 green with existing status/
  failure/direct-module contracts and exact `./tools/test.sh` green. Exact review
  precedes every publication; implementation, closure, and final sync each require
  all eight public checks green.
- Risks: scripts may parse old human-readable wording; broad absence assertions could
  reject unrelated diagnostics; traversal order may vary by filesystem; styling may
  hide wording changes; claim cleanup could accidentally add checked IR or change
  status/count behavior.
- Stop conditions: executable-test ABI/assertions/fixtures/isolation/result semantics;
  recursive or sorted discovery; checked IR, LLVM, codegen, native tools, runtime,
  process launch, artifacts; status/count/diagnostic change; language/type/ownership/
  aggregate/backend/optimizer/`CompilerOptions` behavior; dependency/workflow,
  benchmark, immutable evidence, external artifact, package/release/registry, or
  `master` change.
- Status: complete and accepted at public closure `63b6629`; the selected semantic-
  only `aero test` presentation boundary is controlled. Broader R-013 command behavior
  and executable-test design remain open. This final-state sync changes records only.
  Triple-reviewed preregistration `e6f332f`
  passed all eight checks (compiler `30826689358`/`30826692656`, Rust `30826691808`,
  CodeQL `30826689156`, aggregate `91730202225`). Triple-reviewed tests-only commit
  `6728a39`, tree `5337c877`, diff `f703a4b5`, is public: compiler runs
  `30828281313`/`30828277960` fail only the two frozen contracts at 9/2; nightly in
  Rust run `30828281681` fails identically and stable is cancelled during its test
  step by permitted fail-fast; CodeQL run `30828277876` plus aggregate `91735622062`
  provide all four green CodeQL checks. Exact three-review-approved implementation
  `2fe580d`, tree `1e530e65`, diff `8c119a32`, passes focused CLI 11/11, exact
  `./tools/test.sh`, and all eight public checks (compiler `30829084150`/`30829086467`,
  Rust `30829088650`, CodeQL `30829082758`, aggregate `91738325685`). No capability
  is promoted.
- Closure acceptance: corrected exact record-only diff `b4fd6bc195f70712fbcd0f022d5dcbbcad7128c9`,
  tree `2e88685021de6a7948e6b5ffb69250676764f7f5`, received three independent approvals
  with no P0-P3 findings and was published as `63b66295544d41634f790face005d0fcfc64b41a`.
  Compiler runs `30829963152`/`30829970545`, Rust `30829968789`, CodeQL `30829962982`,
  and aggregate `91741344282` all pass.
- Final-state sync: exact diff `a3cd465fab08c4c9b6b238c7aadd4a39a4d06c3d`,
  tree `46828e7d715c6489eb2c7a661a7ef95b7cb4555b`, received three independent approvals
  with no P0-P3 findings and was published as `25dec51e7fb24a5dd835712568242d685af649cf`.
  Compiler runs `30830484863`/`30830489796`, Rust `30830490379`, CodeQL `30830483828`,
  and aggregate `91743120769` all pass.

## AUDIT-026 — Re-rank remaining compiler-integrity risks after CORE-019

- Task ID/date/basis: `AUDIT-026`, 2026-08-03, accepted clean public head
  `25dec51e7fb24a5dd835712568242d685af649cf`, tree
  `46828e7d715c6489eb2c7a661a7ef95b7cb4555b`; integration branch matches origin,
  PR #4 is open/draft/mergeable, all eight checks pass, and upstream `master` remains
  `8f8c7337a4008082fd2a443fcc814b5847b8663f`.
- Observed behavior at basis head `25dec51`: `CORE-019` controlled its selected
  presentation boundary. The public `CompilerOptions` facade exposed `optimize`,
  `debug_info`, and `target` while `compile_program` named the value `_options` and
  did not read it. Remaining
  safety, compatibility, backend, diagnostic, aggregate, and tooling risks retain
  their current open/partial boundaries.
- Hypothesis: independently reproducing ignored nondefault options and comparing user
  reach, severity, phase count, compatibility ambiguity, and regression feasibility
  can identify the next bounded false success without inventing option semantics.
- Frozen semantics: none. This is read-only evidence collection and ranking; no field
  value is defined as valid, invalid, equivalent, optimizing, debug-emitting, or a
  target selector.
- Allowed files/changes: auditors make no repository changes. Root may later record
  the completed audit and a separately frozen next-task contract in the six control
  records. No source, tests, workflows, dependencies, benchmarks, artifacts,
  package/release/registry state, or `master` change is authorized.
- Acceptance: reproduce default/nondefault facade behavior without claiming option
  semantics; inventory direct callers and current documentation; distinguish library
  checked compilation from CLI target/backend orchestration; compare remaining risks;
  obtain independent type/safety, IR/codegen, and backend/claim reports; recommend one
  bounded next action or stop on an unresolved semantic/compatibility decision.
- Risks: option names may be mistaken for specified behavior; identical LLVM text may
  not prove every internal path identical; source callers may not represent external
  consumers; a fail-closed change may be compatibility-breaking; target strings may
  be confused with CLI CPU/ROCm/CUDA behavior.
- Stop conditions: repository mutation outside this preregistration; option semantics,
  compatibility decision, code/test implementation, language/ownership/aggregate
  decision, hardware/device probe, benchmark run/claim, workflow/dependency change,
  immutable evidence, external artifact, package/release/registry action, destructive
  command, or `master` modification.
- Findings: all three independent auditors traced the public struct at
  `src/compiler/src/lib.rs` and ranked ignored nondefault options as the best bounded
  next candidate under R-006. At basis head `25dec51`, the 62 callers outside the
  definition comprised 28 calls across five benchmarks and 34 across thirteen test
  files; all used `CompilerOptions::default()`, none constructed a nondefault value,
  and the private CLI target/configuration path did not consume the public type.
  Static phase tracing
  at basis head `25dec51` proved `_options` had no read before or within lexing,
  parsing, direct-module collection, semantics, checked IR, or checked code generation.
  No dynamic probe
  result is accepted as audit evidence.
- Commands/evidence: auditors used read-only `rg` inventories and source tracing; the
  IR/codegen audit ran the existing binding (16), checked-IR (6), fatal-parse (11),
  and module-pipeline (7) targets, all 40/40 green. No auditor edited, staged, or
  committed repository files.
- Process deviation and resolution: two auditors attempted temporary external `rustc`
  probes even though the frozen stop conditions prohibited an external artifact. One
  probe reported results; the type/safety phase probe was interrupted and returned no
  result. The lead excludes both probes from every accepted finding and decision.
  Exact read-only checks confirm their two named executables no longer exist; two
  inert PDB byproducts remain in `%TEMP%`. Nothing was published or added to the
  repository. This recorded exclusion and cleanup verification resolves the audit
  deviation without treating probe output as evidence.
- Remaining uncertainty/regression risk: external nondefault consumers are unknown;
  identical outputs do not define intended meanings; fail-closed precedence changes
  their runtime result and can mask a source diagnostic. Broad CLI/library pipeline
  duplication remains open.
- Status: complete at public-green preregistration
  `2c61ff994b8ee903d84d7b0d116503ef3dc7dcfb`, tree
  `ff20cf4332f4bfd54c8b2c20b8364100557ee59b`. Compiler runs
  `30831057824`/`30831063857`, Rust `30831066619`, CodeQL `30831055856`, and
  aggregate `91744957183` provide all eight green checks. Recommended next action is
  the separately frozen `CORE-020`; no capability is promoted by this audit.

## CORE-020 — Fail silently ignored nondefault `CompilerOptions` closed

- Task ID/date/owner: `CORE-020`, 2026-08-03, lead-owned vertical slice under R-006
  and accepted DEC-025.
- Observed behavior before implementation: public `compile_program` accepted
  `CompilerOptions` but named it `_options` and ignored every field. `optimize = true`,
  `debug_info = true`, any nonempty `target`, and combinations could return the same
  successful LLVM as defaults, falsely implying unsupported behavior.
- Hypothesis: one guard at the public library boundary can stop that false success
  before lexing while preserving the full default path and avoiding option, CLI, IR,
  codegen, or backend semantics.
- Frozen semantics: preserve the public struct, fields, derives/default, function
  signature, and exact default parse/modules/semantics/checked-IR/checked-codegen
  output and diagnostic behavior. Exactly `(false, false, String::new())` is supported.
  Any true Boolean field or nonempty `target` returns exactly `Unsupported
  CompilerOptions: only CompilerOptions::default() is supported; optimize, debug_info,
  and target behavior is not implemented` before lexing; malformed source with a
  nondefault option returns this options error. Do not trim or interpret `target`.
- Tests first: add only
  `src/compiler/tests/compiler_options_contract_tests.rs` with two tests. One default
  preservation test must pass against unchanged production, requiring byte-identical
  LLVM for one valid source and the exact existing parse diagnostic for one invalid
  source against literals captured at parent `2c61ff9`. One comprehensive nondefault
  contract must fail, covering
  each Boolean field, a normal nonempty target, a whitespace-only target, a combined
  value, valid source, and the known strict lexical failure `let value = 1@;`; every
  nondefault case must require the exact options diagnostic. The test must evaluate
  all table cases and aggregate mismatches before its final assertion, so the first
  failure cannot hide an uncovered field/precedence case. This proves no target
  trimming and validation before lexing. The target must be exactly 1 passed / 1
  failed; every other target remains baseline. Exact three-review approval precedes
  red publication.
- Implementation files: `src/compiler/src/lib.rs`, the new test file, and the six
  current control records only. The production change is the smallest boundary guard;
  rustdoc may state the supported-default contract. No other file is authorized.
- Acceptance: this preregistration passes exact `./tools/test.sh`, three independent
  reviews, and all eight public checks. Tests-only publication reproduces exactly 1/1
  in both compiler checks and every Rust matrix job that reaches this target, with
  fail-fast cancellation permitted and all four CodeQL checks green. Implementation
  is focused 2/2; existing binding/checked-IR/fatal-parse/module targets are 40/40;
  exact full gate, three independent reviews, and all eight public checks pass.
- Risks: external nondefault callers change from `Ok`/source errors to an earlier
  `Err`; exact string assertions make diagnostic edits deliberate; guard placement
  could accidentally alter defaults; broad tests could imply option meanings; CLI
  target configuration may be confused with this library facade.
- Stop conditions: any optimization, debug-info, target-name/triple, CLI target/
  backend mapping, environment default, warning/normalization, public API/derive/
  default/signature change; parser/modules/semantics/IR/codegen/backend/runtime or
  artifact behavior; more than the boundary plus tests/records; workflow/dependency,
  benchmark, immutable-evidence, package/release/registry, external-artifact,
  destructive-system, or `master` change. Stop on any unexpected second compiler
  phase or semantic decision.
- Preregistration publication: exact staged tree
  `8c807c17d08482994a2ebd16a0208e6394bcfa5b` and diff
  `a466887f940aa12a71f8ec6f454575a9b4547044` received three independent approvals
  with no P0-P3 findings after two rejected snapshots were corrected. Published as
  `fae1374b18a10e229ab01d601d16536154b4c4c6`; compiler runs
  `30833300163`/`30833300408`, Rust `30833301841`, CodeQL `30833296979`, and
  aggregate `91752384364` all pass.
- Tests-only evidence: exact staged tree
  `edd8d33e73353c63494c734326ebf95042a24192` and diff
  `be3ab875f662a4306b3b75049762303a1198e0af` received three independent approvals
  with no P0-P3 findings and were published as
  `037f44d93ea20c2143f75ed3a3b8bf5d4e840f24`. Locally the target is exact 1/1,
  exact `./tools/test.sh` fails there, and `--all-targets --no-fail-fast` proves every
  other target green. Public compiler runs `30833844930`/`30833845633` and nightly in
  Rust run `30833845526` fail only the exact 1/1 target; stable is cancelled during
  its test step by permitted fail-fast. CodeQL `30833844647` and aggregate
  `91754222422` provide all four green checks.
- Local implementation: `src/compiler/src/lib.rs` renames `_options` to `options`,
  documents default-only support, and returns the exact frozen error if either Boolean
  is true or `target` is nonempty before the lexer call. No other compiler phase is
  changed. Focused contract is 2/2; binding 16/16, checked IR 6/6, fatal parse 11/11,
  and modules 7/7 total 40/40; exact `./tools/test.sh` passes.
- Files changed by implementation candidate: `src/compiler/src/lib.rs` plus the six
  authorized current control records. The public tests file is unchanged.
- Remaining uncertainty/regression risk: external nondefault consumers now receive an
  earlier error; real option semantics and broad CLI/library convergence remain
  undefined.
- Implementation acceptance: exact staged tree
  `7c8b2ce1e93c82ca5f42100723431688e7505a22` and diff
  `33e5883e84d82c6a2fa105b7fdfad7d7cebc6ad8` received three independent approvals
  with no P0-P3 findings and were published as
  `70cb0ad1afe3e3649e14a3faca444d8cd16589cb`. Compiler runs
  `30834445685`/`30834446600`, stable/nightly Rust run `30834446605`, CodeQL run
  `30834443841`, and aggregate `91756251121` provide all eight green checks.
- Accepted result: every nondefault option fails with the exact frozen diagnostic
  before lexing, including whitespace-only targets; defaults retain the exact parent
  LLVM and parse diagnostic. Public API shape/defaults are unchanged. No option/CLI/
  IR/codegen/backend semantics or capability class is added.
- Closure acceptance: exact staged tree
  `df4a04a5891139f83ae355aff74bd6726de1057a` and diff
  `85ef52a4090096042db3339e53fdfd2835302531` received three independent approvals
  with no P0-P3 findings and were published as
  `5a8cd06d740f2c3c87843983371cffc9251f8cfe`. Compiler runs
  `30835593703`/`30835597576`, stable/nightly Rust run `30835597620`, CodeQL run
  `30835594365`, and aggregate `91759990615` provide all eight green checks.
- Status: complete for the selected ignored-option boundary at closure `5a8cd06`.
  Real option semantics and broad R-006 convergence remain open; no capability or
  class is promoted.

## AUDIT-027 — Clean-head remaining-risk re-ranking

- Task ID/date/owner: `AUDIT-027`, 2026-08-03, lead-owned read-only audit with three
  independent type/safety, IR/codegen, and backend/claim auditors.
- Observed behavior: after accepted `CORE-020` closure `5a8cd06`, no next
  implementation is authorized. R-002/R-004 retain unresolved type/ownership
  semantics; R-005 needs a major-boundary unchecked-API policy; R-006 retains
  duplicated orchestration and undefined options; R-007 lacks real hardware evidence;
  R-009/R-010 are architectural; R-011 needs aggregate bounds/layout/execution;
  R-012 is an ignored-test backlog requiring slice classification; R-013 retains
  delegated-exit, rollback, executable-test, command-maturity, and helper-architecture
  boundaries; and R-016 needs a toolchain policy.
- Hypothesis: comparing active reproducibility, reach, severity, semantic readiness,
  phase count, compatibility ambiguity, and tests-first feasibility at one clean
  public head can identify one bounded next correction without inventing semantics.
- Frozen audit semantics: use the exact commit publishing this contract as the basis,
  only after its local full gate, three approvals, and all eight public checks are
  green. Its only delta from `5a8cd06` is the six current control records. Treat
  existing risk descriptions and capability labels as hypotheses to verify, not
  permission to promote or implement them. Recommend one bounded tests-first task or
  stop explicitly if no candidate satisfies repository constraints.
- Allowed files/actions: auditors are read-only and may inspect tracked source,
  history, tests, current control documents, `claim-verification/`, and public GitHub
  check metadata. They may run existing non-artifact-producing tests only when static
  evidence is insufficient and must report them exactly. After reconciliation, only
  the lead may edit the six current control records: `PROJECT_STATE.md`,
  `CURRENT_CAPABILITY_AUDIT.md`, `DECISION_LOG.md`, `INITIAL_RISK_REGISTER.md`,
  `SPEC_IMPLEMENTATION_MATRIX.md`, and `TASK_LEDGER.md`.
- Acceptance: this preregistration/final-state sync passes exact `./tools/test.sh`,
  three independent exact-snapshot reviews, and all eight public checks. The later
  audit reports the required nine evidence fields, ranks the same candidate set, ties
  each conclusion to file/symbol or line evidence, reconciles disagreements, and
  makes no code, test, workflow, dependency, package, claim-evidence, or capability
  change.
- Risks: stale historical claims can be mistaken for current behavior; broad risks
  can conceal a bounded slice; repeated tests can create artifacts or alter state;
  raw severity can pressure unresolved semantics; external consumers/hardware remain
  unknowable from repository evidence alone.
- Stop conditions: any new probe/source/executable/artifact, ignored-test activation,
  code/test/workflow/dependency edit, language/type/ownership/aggregate/option/
  toolchain/hardware semantic decision, benchmark or performance claim, package/
  release/registry action, immutable-evidence mutation, destructive system action,
  branch/history rewrite, or `master` change. Stop rather than select work crossing
  more than two compiler phases or lacking frozen semantics and a failing-test path.
- Preregistration acceptance: exact staged tree
  `4caa5c339810412f7f96ba673dac2d6ec8301094` and diff
  `bd174a08c90b07f03810ef2ce6ed9aab7ba18d0a` received three independent approvals
  with no P0-P3 findings after an omitted R-013 candidate was corrected. Published
  as `aa3e7a8d29f73c59e8495b3c18702abb16a4f9c6`; compiler runs
  `30836250279`/`30836251909`, stable/nightly Rust run `30836255407`, CodeQL run
  `30836248101`, and aggregate `91762198170` provide all eight green checks.
- Findings: all three auditors rank R-013 first. The full set was compared without
  edits, tests, probes, or artifacts. R-012 is next actionable evidence debt but its
  slice is unclassified; R-002/R-004/R-005 need semantics or major-version policy;
  R-006/R-009/R-010/R-011 are architectural or semantic; R-007 needs hardware; and
  R-016 needs a supported-toolchain decision.
- Slice reconciliation: A suppresses false success wording on nonzero CPU children;
  B returns delegated status instead of exiting inside the helper; C contains a
  dangling destination entry before `init` writes a partial manifest. Backend/claim
  and IR/codegen auditors rank A first; type/safety ranks C first and A second. The
  lead selects A for its every-nonzero reach, direct false claim, cross-platform
  deterministic test, zero compiler phases, and preserved status/cleanup contract.
  C remains the bounded runner-up; B has no failing observable contract.
- Status: complete at immutable public basis `aa3e7a8`; no capability or class is
  promoted. Recommended next action is the separately frozen `CORE-021`.

## CORE-021 — Truthful delegated CPU exit presentation

- Task ID/date/owner: `CORE-021`, 2026-08-03, lead-owned R-013 tooling slice under
  accepted DEC-026.
- Observed behavior: `run_aero_program_with_artifacts` obtains a CPU child's exit via
  `status.code().unwrap_or(-1)`, unconditionally prints `Program executed
  successfully.`, then prints `Exit code: N`. The process test supplies exit 7 and
  currently requires both lines, so a failing delegated program is presented as
  successful even though the exact nonzero status is passed through after cleanup.
- Hypothesis: condition only the success line on `exit_code == 0`; this removes false
  presentation without changing execution, status, output forwarding, cleanup, CLI
  classification, compiler phases, or backend behavior.
- Frozen semantics: for delegated CPU exits `0`, `1`, `2`, and `7`, preserve the
  exact process status and exact `Exit code: N`. Preserve signal fallback `-1` and
  print no success line for it. Preserve deterministic child stdout and stderr
  presentation, artifact cleanup before status propagation, cleanup/error precedence,
  verifier/object/link/process behavior, and exact zero-exit success wording. Print
  no replacement success/failure phrase for nonzero. Delegated `1`/`2` remain child
  statuses, not CLI-owned `CliStatus` classifications. ROCm/CUDA are unchanged.
- Tests first: change only
  `src/compiler/tests/cli_status_contract_tests.rs`. Extend the existing delegated-
  exit process contract within its single test function to cover exits `0`, `1`,
  `2`, and `7`, exact stdout/stderr markers, exact status/exit line, empty temporary
  run-artifact directories, success wording required only for zero, and forbidden
  for every nonzero. Reuse one configurable fake native tool rather than compiling
  one per exit. On unchanged production the focused target must be exactly 10 passed /
  1 failed, with only this test function failing at its final aggregate assertion;
  all four cases must execute before assertion. Exact three-review approval precedes
  public red publication.
- Implementation files: only `src/compiler/src/main.rs`, the unchanged tests-first
  file, and the six current control records. Production adds one `exit_code == 0`
  condition around the existing success print. No replacement wording or refactor.
- Acceptance: tests-only public compiler checks and every Rust matrix job reaching
  the target reproduce exact 10/1 while all four CodeQL checks pass; fail-fast
  cancellation is recorded rather than hidden. Implementation produces focused
  11/11, exact `./tools/test.sh`, three independent exact-snapshot approvals, and all
  eight public checks while preserving every frozen output/status/cleanup control.
- Risks: external scripts may parse the misleading line; changing it is intentionally
  incompatible for nonzero runs. Test helpers can accidentally compile repeatedly or
  create security-noisy executables; production could remap child `1`/`2`, suppress
  zero success, change stdout/stderr ordering, exit before cleanup, or affect GPU
  paths.
- Stop conditions: any child-status remapping; new wording; helper-return/internal-
  exit refactor; cleanup/error-precedence change; init containment/rollback; `aero
  test` execution; parser/semantics/IR/codegen/backend/option behavior; new native
  probe outside the existing isolated process fixture; workflow/dependency, benchmark,
  package/release/registry, immutable-evidence, destructive-system, history rewrite,
  or `master` change. Stop on any second production phase or semantic decision.
- Status: implementation accepted; record-only closure pending. The exact six-record
  preregistration was accepted at `a61ea24` after the full local gate, three
  independent approvals, and all eight public checks (compiler `30837838305` /
  `30837843933`, Rust `30837844778`, CodeQL `30837838554`, aggregate
  `91767404453`). The corrected tests-only checkpoint `0873f65`, tree
  `51ec7d0ac705c42d2d08eb5e94ce4f2c3892d617`, diff
  `f75a636090b5ff148779fba10b94ab0394e3e79f`, received three exact approvals
  after reviewers rejected two weaker candidates. Locally it formats cleanly,
  produces exact focused 10/1, makes the required gate fail only on that target,
  and makes the no-fail-fast all-target run report exactly one failed target.
  Public compiler runs `30839264536` / `30839272375` reproduce 10/1; nightly in
  Rust run `30839272429` reproduces it and stable is cancelled during its test step
  by matrix fail-fast. CodeQL run `30839264268` passes actions/Python/Rust analyses
  and aggregate `91772180985` passes. Production now has only the frozen
  `exit_code == 0` guard in `src/compiler/src/main.rs`. Formatting, focused CLI
  11/11, backend-claim 7/7, and exact `./tools/test.sh` are green. Exact staged tree
  `0ad98c8223b0cbb23764df9fd964b9f50c2315b6` and diff
  `2dbbc39582abbfa5b82a467e4ad7a5ca15ed3f83` received three independent approvals
  with no P0-P3 findings and were published unchanged as `a4327be`. All eight public
  checks pass in compiler runs `30839860335` / `30839862442`, stable/nightly Rust
  run `30839862423`, CodeQL run `30839859840`, and aggregate `91774125621`.
  `CORE-021` therefore accepts only the frozen delegated-exit presentation boundary.
  This exact six-record final-state sync passes the full local gate; exact review and
  public closure remain pending.

- Final closure acceptance: corrected exact record-only tree
  `8a4c2d7733d9073ee375dbebbcc3e2221a807df2` and diff
  `5abbf3a7cc2eaf718d5e76183114339254cb2898` received three independent approvals
  with no P0-P3 findings after two reviewers rejected stale local-gate wording. It was
  published unchanged as `b99e445`. Compiler runs `30840427466` / `30840426655`,
  stable/nightly Rust run `30840428215`, CodeQL run `30840415565`, and aggregate
  `91775938704` all pass. `CORE-021` is complete; R-013 remains partially controlled
  because init containment/rollback, executable-test design, command maturity, and
  helper architecture remain open.

## AUDIT-028 — Re-rank remaining risks after CORE-021

- Task ID/date/owner: `AUDIT-028`, 2026-08-03, lead-owned read-only audit with three
  independent type/safety, IR/codegen, and backend/claim auditors.
- Observed behavior: `CORE-021` is accepted and no next implementation is authorized.
  The remaining OPEN or PARTIALLY CONTROLLED set is R-002, R-004, R-005, R-006,
  R-007, R-009, R-010, R-011, R-012, R-013, and R-016. Their residuals range from
  unresolved language semantics and compiler architecture to evidence debt, command
  containment, absent hardware proof, and reproducibility policy.
- Hypothesis: an independent clean-head comparison of severity, reach, current
  fail-open behavior, semantic readiness, phase count, compatibility ambiguity, and
  deterministic tests-first feasibility can identify one bounded next correction
  without silently choosing semantics or confusing evidence with capability.
- Frozen audit semantics: the audit basis is the exact commit publishing this
  contract, but only after its exact full local gate, three approvals, and all eight
  public checks pass. Each auditor must inspect every listed residual, rank the full
  set, identify the strongest currently observable evidence for the leaders, state
  stop reasons for semantically or architecturally unready work, and propose at most
  three bounded candidate slices with explicit failing-test paths and phase counts.
  Omission of any listed risk invalidates the audit. No ranking by title/severity
  alone and no inherited AUDIT-027 ordering.
- Allowed files: this preregistration may change only `TASK_LEDGER.md`,
  `DECISION_LOG.md`, `CURRENT_CAPABILITY_AUDIT.md`, `PROJECT_STATE.md`,
  `SPEC_IMPLEMENTATION_MATRIX.md`, and `INITIAL_RISK_REGISTER.md`. The audit itself
  is read-only and changes no file, index, worktree, branch, issue, PR, workflow,
  dependency, artifact, claim evidence, or external state.
- Acceptance: this exact six-record preregistration passes `./tools/test.sh`, three
  exact-snapshot reviews with no P0-P3, and all eight public checks. Then all three
  auditors report the required nine fields, the lead reconciles disagreements and
  records one selection or explicit stop, and a separate frozen task contract must
  pass before any test or implementation change.
- Risks: stale records can omit a newly controlled residual; reviewers may conflate
  hardware flags with execution, ignored tests with capability, untrusted public APIs
  with trusted compiler paths, or bounded containment with full semantic closure.
- Stop conditions: any edit/test/probe/artifact during the audit; implementation or
  semantic decision before reconciliation; missing risk; selection spanning more
  than two compiler phases; unsupported source-type default; invalid-program path to
  IR/backend; hardware/performance/stability claim without immutable evidence;
  workflow/dependency, benchmark, package/release/registry, history rewrite,
  destructive-system, or `master` action.
- Status: preregistered and full-local-gate green; audit work is prohibited until
  this exact contract is triple-approved, published, and all-eight public green.

- Preregistration acceptance: exact staged tree
  `e61762fdea546581175d75232e136591b45c83a1` and diff
  `82880ead62a023a6f329dea163aa5573c9461db4` received three independent approvals
  with no P0-P3 findings and were published unchanged as `399e04f`. Compiler runs
  `30841015776` / `30841022060`, stable/nightly Rust run `30841023011`, CodeQL run
  `30841017756`, and aggregate `91777920315` all pass.
- Findings: all three auditors inspected and ranked all eleven residuals without
  edits, tests, probes, artifacts, or external queries. Type/safety ranks R-011,
  R-013, R-002 first; IR/codegen ranks R-013, R-012, R-002; backend/claim ranks
  R-002, R-013, R-010. R-013 is the only risk ranked in every top two.
- Reconciliation: the lead selects only R-013's entry-aware `aero init` preflight.
  It is directly observable, deterministic in the existing Unix fixture, changes
  zero compiler phases, and follows the established no-overwrite policy. R-011 is
  stopped because compile-error versus trap/unchecked bounds semantics are not
  frozen; R-002 remains a runner-up but auditors disagree on one-versus-two-phase
  enforcement and broader contract interactions. R-004/R-005/R-006/R-007/R-009/
  R-010/R-011/R-016 retain their explicit semantic, architectural, hardware, or
  policy stops; R-012 remains evidence debt.
- Status: complete, read-only, result commit none. The selection authorizes only the
  separately frozen `CORE-022` contract below, not tests or implementation yet.

## CORE-022 — Refuse occupied init destinations before writes

- Task ID/date/owner: `CORE-022`, 2026-08-03, lead-owned R-013 project-tooling slice
  under DEC-027, with independent type/safety, IR/codegen, and backend/claim review
  at every publication boundary.
- Observed behavior: `init_project` checks `Path::exists()` for `aero.toml` and
  `src/main.aero`, creates `src`, writes the manifest, then writes the source. A
  dangling `src/main.aero` symlink returns false from `exists()`, so the manifest is
  published before the source write fails. The established Unix CLI fixture requires
  that partial manifest as current evidence.
- Hypothesis: inspect destination directory entries without following symlinks before
  directory creation or file writes. Treat only `NotFound` as available; refuse every
  existing entry with the existing manifest/source refusal wording, and fail closed
  on any other inspection error. This prevents the reproduced partial write without
  deleting/following user entries or promising general rollback.
- Frozen semantics: `aero.toml` and `src/main.aero` are occupied by any existing
  filesystem entry, including a dangling symlink. Manifest is checked first, then
  source, preserving current diagnostic precedence. Occupancy returns exact existing
  `refusing to overwrite existing manifest: PATH` or `refusing to overwrite existing
  source file: PATH`; an inspection failure returns exact
  `failed to inspect project destination PATH: ERROR` before any create/write.
  Preserve the target root, blocker, symlink, and every preexisting entry byte-for-
  byte. On refusal create no new directory or file. Preserve successful init content/
  result, package-name inference, CLI status `1`, error stream, and all unrelated
  command/compiler/backend behavior. This is preflight containment, not transactional
  rollback, atomicity, TOCTOU elimination, or a general filesystem policy.
- Tests first: change only `src/compiler/tests/cli_status_contract_tests.rs`. In the
  established `#[cfg(unix)]` dangling-source fixture, require exact operational
  status `1`, the existing source-refusal diagnostic, no success text, no created
  manifest, and preservation of the dangling entry and blocker. All assertions join
  the existing aggregate failure list. Unchanged Linux production must make exactly
  that established test fail; Windows cannot create/exercise this Unix fixture and
  must be reported as a platform limitation, not as red evidence. Exact three-review
  approval precedes public tests-only publication.
- Implementation files: only `src/compiler/src/project_init.rs`, the unchanged
  tests-first file, and the six current control records. Use `symlink_metadata` (or an
  equivalently non-following standard-library inspection) in one preflight helper;
  only `io::ErrorKind::NotFound` means available. No dependency or refactor.
- Acceptance: tests-only Linux public compiler checks and every Rust job reaching the
  fixture reproduce one focused target failure while CodeQL stays green; permitted
  fail-fast cancellation is recorded. Implementation makes focused CLI 11/11 on
  Linux, preserves the Windows 11/11 target with the Unix case compiled out, passes
  exact `./tools/test.sh`, three exact-snapshot reviews, and all eight public checks.
- Risks: following/removing the symlink, deleting the blocker, changing manifest-
  first precedence, treating permission/I/O errors as absence, claiming rollback,
  platform-specific diagnostic paths, or broadening into general filesystem races.
- Stop conditions: cleanup/rollback after writes; temp-file/rename transaction;
  symlink resolution/removal; permission/ownership policy beyond fail-closed
  inspection; CLI status/helper-exit change; parser/semantics/IR/codegen/backend
  behavior; new dependency/workflow, benchmark, package/release/registry, immutable
  evidence, history rewrite, destructive-system, or `master` action.
- Preregistration acceptance: exact tree `60978f638e40bec51f79fcb8010c990025da3baa`
  and diff `691be8a860657dc02cb137e8bdb9aac66765db91` received three approvals
  with no P0-P3 findings and were published unchanged as `045339d`. Compiler
  `30842636280` / `30842636899`, Rust `30842636893`, CodeQL `30842634423`, and
  aggregate `91783336329` all pass.
- Tests-first checkpoint: exact one-test-file tree
  `70abf47c4bb0ddf01554202d2c5ca28d6eb15713` and diff
  `9e8157369b1f7f7f398e8319aeaecabafe46d5ee` received three approvals and were
  published unchanged as `7cd8aba`. Both compiler runs `30843119793` /
  `30843125522` reproduce exactly 10/1: only
  `pre_publication_and_established_operational_failures_return_one` fails, and its
  aggregate report contains only the missing source-refusal text and unexpected
  partial manifest. Nightly in Rust `30843124314` reproduces the same failure;
  stable reaches `Run tests` and is cancelled by matrix fail-fast. CodeQL
  `30843121127` passes all three analyses and aggregate `91784962909` passes.
- Implementation acceptance: exact one-production-file tree
  `a61f2c5b0665e70ccde2d7346e794fd214c67c8c` and diff
  `99000b4700e75f6d65409ea947d8234c52b64059` received three approvals with no
  P0-P3 findings and were published unchanged as `2a42324`. Corrected focused binary
  unit tests pass 3/3, CLI passes 11/11, and exact `./tools/test.sh` passes locally on
  Windows; the dangling-symlink case remains compiled out there. Public compiler
  `30843592298` / `30843592784`, stable/nightly Rust `30843595560`, CodeQL
  `30843589175`, and aggregate `91786468184` all pass on Linux.
- Result: `symlink_metadata` now checks manifest then source before any create/write;
  every existing final entry is occupied, only `NotFound` is available, and other
  inspection errors fail with the frozen diagnostic. The accepted test binds no
  partial manifest plus exact symlink-target and blocker-byte preservation. No
  rollback, atomicity, TOCTOU, ancestor-symlink, language, compiler, backend, safety,
  stability, package, release, benchmark, or hardware capability is claimed.
- Final closure: exact six-record tree `e740df4883a87d8a764b143d0edb7cdaf5ada30c`
  and diff `3eb8264b7f89be8090db0995d415125125020675` received three approvals with no
  P0-P3 findings and were published unchanged as `aa29a00`. Compiler
  `30844324249` / `30844328660`, stable/nightly Rust `30844328850`, CodeQL
  `30844325051`, and aggregate `91788926688` all pass.
- Status: complete at record closure `aa29a00`. Only the selected final-entry init
  preflight is accepted; R-013 remains partially controlled and every excluded
  boundary remains open.

## AUDIT-029 — Clean-head residual-risk feasibility ranking

- Task ID/date/owner: `AUDIT-029`, 2026-08-03, lead-owned reconciliation with
  independent type/safety, IR/codegen, and backend/claim read-only auditors.
- Observed behavior: `CORE-022` is complete at public closure `aa29a00`; exact
  triple-reviewed status synchronization `21153f3`, tree `d667ce37`, diff
  `c69c5a1e`, passes compiler `30844798322` / `30844802332`, stable/nightly Rust
  `30844802044`, CodeQL `30844799426`, and aggregate `91790481511`. No next
  implementation is authorized. The remaining OPEN or PARTIALLY CONTROLLED set is
  R-002, R-004, R-005, R-006, R-007, R-009, R-010, R-011, R-012, R-013, and R-016.
- Hypothesis: a clean-head, delta-aware full-set ranking can identify the highest-
  severity bounded residual that has frozen semantics, deterministic tests-first
  evidence, and at most two compiler phases, without reselecting an accepted slice or
  mistaking records, annotations, flags, simulation, or object emission for language
  or hardware capability.
- Frozen audit semantics: the audit basis is the exact commit publishing this
  contract, only after its exact full local gate, three approvals, and all eight
  public checks pass. Each auditor must inspect and rank every listed residual while
  separating accepted sub-slices from still-open behavior. No inherited AUDIT-028
  ordering and no candidate may merely repeat CORE-022 or another accepted slice.
  Each auditor proposes at most three candidates and, for each, cites current
  observable evidence, exact source/test paths, expected phase count, unresolved
  compatibility or semantic choices, a deterministic failing-test route, and stop
  conditions. A risk may recur only through a distinct open boundary. Unsupported
  semantics, more than two compiler phases, absent deterministic evidence, or a
  hardware/performance/stability claim without immutable proof is a stop, not a
  lower-confidence implementation recommendation.
- Allowed files: this preregistration may change only `TASK_LEDGER.md`,
  `DECISION_LOG.md`, `CURRENT_CAPABILITY_AUDIT.md`, `PROJECT_STATE.md`,
  `SPEC_IMPLEMENTATION_MATRIX.md`, and `INITIAL_RISK_REGISTER.md`. The audit itself
  is strictly read-only and changes no file, index, worktree, branch, issue, PR,
  workflow, dependency, artifact, claim evidence, or external state.
- Acceptance: this exact six-record preregistration passes `./tools/test.sh`, three
  exact-snapshot reviews with no P0-P3 findings, unchanged publication, and all eight
  public checks. Then all three auditors report the required nine fields and complete
  ranking; the lead reconciles one distinct bounded selection or an explicit stop.
  A separately frozen task contract must pass before any test or implementation edit.
- Risks: stale accepted-boundary accounting; risk-title ranking; accidental semantic
  invention; treating public unchecked compatibility APIs as trusted compiler paths;
  treating ignored tests as capability; or confusing CPU, ROCm, and CUDA evidence.
- Stop conditions: omitted residual; edit/test/probe/artifact/external query during
  the audit; implementation before reconciliation and a separate task contract;
  unsupported source-type default; invalid-program path to IR/backend; more than two
  compiler phases; workflow/dependency, benchmark, package/release/registry,
  immutable evidence, history rewrite, destructive-system, or `master` action.
- Status: preregistered and full-local-gate green. Audit work is prohibited until
  this exact contract passes three exact reviews, unchanged publication, and all
  eight public checks.

- Authorization evidence: exact triple-approved commit
  `0e5cba17abec65b96a9f04ddd3450ef10cd9fa40`, tree
  `6ac88db4e3c3316886c363e2be4430ad83dd7533`, and diff
  `161cbee6fcedad0054fcd9931c1d8b8424797f89` pass compiler runs
  `30845609442` / `30845612610`, stable/nightly Rust run `30845612328`, CodeQL
  run `30845609103`, and aggregate `91793190047`.
- Findings: all three auditors completed the required read-only nine-field reports
  and ranked all eleven residuals. Type/safety ranks R-002/R-012/R-011/R-004/R-005/
  R-013/R-009/R-006/R-010/R-007/R-016. IR/codegen ranks R-010/R-012/R-002/R-011/
  R-005/R-004/R-009/R-006/R-013/R-016/R-007. Backend/claim ranks R-009/R-012/
  R-002/R-005/R-004/R-011/R-006/R-013/R-010/R-007/R-016. R-012 is the common
  evidence-only runner-up; the three top implementation candidates are distinct.
- Reconciliation: the lead selects R-002's monomorphic Boolean helper-function
  semantic contracts. Current semantics omits `bool` when registering function
  contracts, accepts invalid Boolean calls and returns, and types other declared
  calls as `Int`, while checked IR already admits exact Boolean function signatures
  as LLVM `i1`. The slice is deterministic, one compiler phase, and uses already
  supported exact Boolean equality without defining a new source type or coercion.
  R-009's parser-column UTF-16 adapter and R-010's grammar-authority notice remain
  bounded follow-ups. R-011 remains stopped on unfrozen bounds behavior; R-004/
  R-005/R-006/R-007/R-013/R-016 retain their recorded semantic, architectural,
  hardware, compatibility, or policy stops.
- Status: complete, strictly read-only, result commit none. Only the separately
  frozen `CORE-023` contract below may proceed; no test or implementation was
  authorized or executed by this audit.

## CORE-023 - Enforce Boolean helper-function contracts in semantics

- Task ID/date/owner: `CORE-023`, 2026-08-03, lead-owned one-phase R-002 semantic
  vertical slice under DEC-028, with independent type/safety, IR/codegen, and
  backend/claim review at every publication boundary.
- Observed behavior: `SemanticAnalyzer::analyze` registers a function contract only
  when every monomorphic parameter and return passes `numeric_contract_type`, which
  maps `int`/`i32` and `float`/`f64` but not `bool`. Consequently semantic analysis
  accepts `identity_bool(1)` and `fn broken() -> bool { return 1; }`, while a valid
  `let selected: bool = truth();` is rejected because a noncontract function call is
  inferred as `Int`. Checked IR independently maps Boolean helper-function
  definitions, calls, returns, and storage to LLVM `i1`, so later admission can mask
  the earlier fail-open boundary for invalid programs.
- Hypothesis: extend the existing monomorphic top-level function-contract path to
  exact Boolean parameter and return types for non-entry helpers. Reuse the current
  arity, parameter mismatch, return mismatch, all-path return, forward-call,
  recursion, and direct-module behavior. A valid Boolean call then infers `Ty::Bool`;
  invalid Boolean calls and returns stop in semantics before checked IR.
- Frozen semantics: source `bool` maps exactly to `Ty::Bool`; no implicit conversion,
  truthiness, numeric default, or coercion is introduced. Scope is monomorphic,
  non-entry, top-level helper functions and their direct calls, including forward,
  recursive, and direct-module visibility already used by numeric contracts. Exact
  existing diagnostics apply: arity mismatch; parameter `NAME` type mismatch with
  expected/actual contract names; function return type mismatch; and must-return-on-
  all-paths. Existing numeric aliases/contracts, void statements/value rejection,
  closure shadowing, Boolean binding equality, and checked-IR LLVM `i1` behavior are
  preservation controls. `main` entry semantics and ABI are unchanged.
- Tests first: change only `src/compiler/tests/function_contract_tests.rs`. Add one
  aggregate direct-semantic target using strict lexing, located parsing, and a fresh
  `SemanticAnalyzer` per specimen. It must require rejection of an `int` passed to a
  `bool` parameter with function/parameter/expected-bool/actual-int fragments;
  rejection of an `int` returned from a `bool` helper with function/expected-bool/
  actual-int fragments; and acceptance of a Boolean helper call assigned to a
  `bool` binding. The same aggregate must preserve direct-semantic entry behavior:
  `fn main() -> bool { return 1; }` retains its current analyzer acceptance strictly
  as quarantined entry behavior, not program-validity evidence, while
  `fn main() -> i32 { return 1.0; }` retains its existing numeric return-mismatch
  rejection. The unchanged basis must fail exactly this target with the three helper
  discrepancies reported together while both entry controls pass. Review and publish
  this tests-only tree before production work.
- Implementation files: only `src/compiler/src/semantic_analyzer.rs`, the unchanged
  tests-first file, and these six control records. Keep the existing contract data
  and validation flow. Use a function-contract-specific helper mapping that extends
  the existing numeric mapping with `bool`, and select it only for non-entry
  monomorphic helpers; retain the existing numeric/void mapping for `main`. Do not
  add `bool` to shared `numeric_contract_type`, which also feeds binding/array logic,
  or skip all entry registration. No parser, AST, IR, verifier, codegen, ABI, module-
  collection, CLI, backend, dependency, or workflow change.
- Acceptance: tests-only focused public evidence reproduces exactly one failing
  `function_contract_tests` target in each compiler job and every stable/nightly Rust
  job that reaches it, with fail-fast cancellation recorded exactly and all CodeQL
  analyses green. Implementation passes the focused function-contract target,
  existing Boolean binding and checked-IR `i1` preservation targets, direct-module
  function controls, both direct-semantic `main` preservation specimens, exact
  `./tools/test.sh`, three exact-snapshot reviews, unchanged publication, and all
  eight public checks. Invalid in-scope Boolean helper calls/returns must fail in
  direct semantics before IR; valid selected helper calls must infer `Ty::Bool`.
- Compatibility decision: previously fail-open direct semantic consumers now receive
  the existing exact contract diagnostics for invalid in-scope Boolean helper calls/
  returns; valid Boolean helper results previously mis-typed as `Int` become
  `Ty::Bool`. This is an intentional experimental-front-end correction. Syntax,
  public Rust signatures, entry behavior, numeric/void behavior, and already-checked
  `i1` lowering remain compatible.
- Risks: accidentally registering `main`; broadening String/custom/generic/array/
  tuple/reference/closure/method contracts; changing numeric/void diagnostics;
  applying coercion; desynchronizing semantic and checked-IR signatures; or letting
  a new or in-scope invalid Boolean non-entry helper program reach IR/artifact
  generation.
- Stop conditions: any entry-point or ABI change; any parser/AST, IR, verifier,
  codegen, backend, layout, ownership, generic, aggregate, String, custom-name,
  method, closure, coercion, or defaulting change; more than the semantic compiler
  phase; unsupported type fallback; any new or in-scope invalid non-entry helper path
  beyond semantics; new dependency/workflow, benchmark, package/release/registry,
  immutable claim evidence, history rewrite, destructive-system, or `master` action.
- Status: preregistered and full-local-gate green. No test or production edit is
  authorized until this exact six-record snapshot passes three exact reviews,
  unchanged publication, and all eight public checks.

- Preregistration acceptance: corrected exact tree
  `ce4e0aa117fe25beafeb85b3b6e03d083086155f` and diff
  `ace3e88d14b2bf90e5f7aad35b317606e72755c9` received three approvals with no
  P0-P3 findings and were published unchanged as
  `1c28a7ba05c476be8a128f44895e2649913cba85`. Compiler runs `30848164601` /
  `30848168070`, stable/nightly Rust run `30848169186`, CodeQL run `30848164733`,
  and aggregate `91801596136` all pass.
- Tests-first checkpoint: exact one-test-file tree
  `3fd13263a6338fed3491c2c869e79f854a3f194f` and diff
  `bd17adefe08bc9d2039336bfb39b7b15d95e9663` received three approvals and were
  published unchanged as `c3f6e90ec8fcdf9ae11c3ad0f54d0ef1a8c06f18`. Compiler
  runs `30848723940` / `30848725388` and nightly in Rust run `30848725757`
  reproduce exactly 13/1: only
  `boolean_helper_contracts_stop_in_semantics_without_changing_main` fails, with
  exactly the frozen invalid-parameter acceptance, invalid-return acceptance, and
  valid-result `Int` mismatch. Stable is cancelled by matrix fail-fast before
  completion. CodeQL `30848722802` passes all three analyses and aggregate
  `91803430236` passes.
- Implementation acceptance: exact one-production-file tree
  `c0b538c976b0cbdd3b264b0a106318b71e248de1` and diff
  `b1ecc6eea0a5da8d9a9d742c2422b5d36268c48c` received three approvals with no
  P0-P3 findings and were published unchanged as
  `67ccdf255381f2742217ab6e9f1307aba4ac7077`. Focused CORE-023 passes 1/1,
  function contracts 14/14, binding preservation 16/16, Boolean checked-IR `i1`
  preservation 1/1, and exact `./tools/test.sh` passes locally. Public compiler runs
  `30850000615` / `30850005598`, stable/nightly Rust `30850005670`, CodeQL
  `30850001251`, and aggregate `91807553635` all pass.
- Result: non-entry monomorphic helpers use a function-contract-specific mapper that
  composes the unchanged numeric mapping with exact named `bool`. Parameters and
  returns share that mapper, so invalid Boolean arguments/returns stop in semantics
  and valid Boolean calls infer `Ty::Bool`. Exact `main` continues to use the prior
  numeric/void mapper; the shared binding/array mapper and every generic, String,
  custom, composite, parser, IR, verifier, codegen, ABI, CLI, and backend boundary
  remain unchanged. The quarantined Boolean-entry gap remains open and is not
  program-validity evidence.
- Final closure: exact six-record tree
  `71ac4da77dcec4d26c70bac3f807e43a3d1580d9` and diff
  `adba01a19ca0c5c3d5372eae954371ea0c72cbda` received three approvals with no
  P0-P3 findings and were published unchanged as
  `0b88530a5d9510877a7bbc1df407fb45b9e136a9`. Compiler runs
  `30850519757` / `30850524194`, stable/nightly Rust `30850524148`, CodeQL
  `30850520457`, and aggregate `91809289681` all pass.
- Status: complete at record closure `0b88530`. Only monomorphic non-entry Boolean
  helper contracts are accepted; R-002 remains PARTIALLY CONTROLLED and every
  excluded entry, type, shape, coercion, and ABI boundary remains open or
  quarantined.

## AUDIT-030 - Clean-head residual-risk feasibility ranking

- Task ID/date/owner: `AUDIT-030`, 2026-08-03, lead-owned reconciliation with
  independent type/safety, IR/codegen, and backend/claim read-only auditors.
- Observed behavior: `CORE-023` is complete at public closure `0b88530`. Exact
  non-entry monomorphic Boolean helper contracts now fail closed in semantics, but
  R-002 remains PARTIALLY CONTROLLED. The remaining OPEN or PARTIALLY CONTROLLED set
  is R-002, R-004, R-005, R-006, R-007, R-009, R-010, R-011, R-012, R-013, and
  R-016. No next implementation is authorized.
- Hypothesis: a clean-head, delta-aware full-set ranking can identify the highest-
  severity bounded residual with frozen semantics, deterministic tests-first
  evidence, and at most two compiler phases, without reselecting an accepted slice
  or treating records, annotations, flags, simulation, or object emission as
  language or hardware capability.
- Frozen audit semantics: the audit basis is the exact commit publishing this
  contract, only after its full local gate, three exact approvals, unchanged
  publication, and all eight public checks pass. Each auditor must inspect and rank
  every listed residual while separating accepted sub-slices from still-open
  behavior. No inherited AUDIT-029 ordering is allowed, and no candidate may repeat
  CORE-023 or another accepted slice. Each auditor proposes at most three candidates
  and, for each, cites current observable evidence, exact source/test paths,
  expected phase count, unresolved compatibility or semantic choices, a
  deterministic failing-test route, and stop conditions. A risk may recur only
  through a distinct open boundary. Unsupported semantics, more than two compiler
  phases, absent deterministic evidence, or a hardware/performance/stability claim
  without immutable proof is a stop, not a lower-confidence recommendation.
- Allowed files: this preregistration may change only `TASK_LEDGER.md`,
  `DECISION_LOG.md`, `CURRENT_CAPABILITY_AUDIT.md`, `PROJECT_STATE.md`,
  `SPEC_IMPLEMENTATION_MATRIX.md`, and `INITIAL_RISK_REGISTER.md`. The audit itself
  is strictly read-only and changes no file, index, worktree, branch, issue, PR,
  workflow, dependency, artifact, claim evidence, or external state.
- Acceptance: this exact six-record preregistration passes `./tools/test.sh`, three
  exact-snapshot reviews with no P0-P3 findings, unchanged publication, and all
  eight public checks. Then all three auditors report the required nine fields and
  complete ranking; the lead reconciles one distinct bounded selection or an
  explicit stop. Any test or implementation edit requires a separately frozen task
  contract.
- Risks: stale accepted-boundary accounting; risk-title ranking; accidental semantic
  invention; treating public unchecked compatibility APIs as trusted compiler paths;
  treating ignored tests as capability; or confusing CPU, ROCm, and CUDA evidence.
- Stop conditions: omitted residual; edit/test/probe/artifact/external query during
  the audit; implementation before reconciliation and a separate task contract;
  unsupported source-type default; invalid-program path to IR/backend; more than two
  compiler phases; workflow/dependency, benchmark, package/release/registry,
  immutable evidence, history rewrite, destructive-system, or `master` action.
- Status: preregistered and full-local-gate green. Audit work is prohibited until
  this exact contract passes three exact reviews, unchanged publication, and all
  eight public checks.

- Authorization evidence: exact triple-approved commit
  `d4e3c75b043bad75b714113af8d98aafd8c79b75`, tree
  `9a07c10ccab196ace4b90ba372751f3913301284`, and diff
  `18e3a2e98984b0de85e19e1b8d8486704c36b77c` pass compiler runs
  `30851275589` / `30851278460`, stable/nightly Rust `30851278586`, CodeQL
  `30851276053`, and aggregate `91811764009`.
- Findings: all three auditors completed the required read-only nine-field reports
  and independently ranked all eleven residuals. Type/safety ranks R-009/R-010/
  R-012/R-002/R-004/R-005/R-011/R-013/R-006/R-016/R-007. IR/codegen ranks
  R-009/R-010/R-012/R-011/R-002/R-005/R-004/R-013/R-006/R-016/R-007.
  Backend/claim ranks R-002/R-010/R-009/R-012/R-005/R-004/R-006/R-013/R-011/
  R-016/R-007. R-010 is the universal second-place containment slice, and R-009 is
  the only candidate in every top three that all auditors find fully frozen.
- Reconciliation: the lead selects R-009's parser-diagnostic UTF-16 projection.
  Parser errors currently expose Unicode-scalar columns directly as LSP character
  offsets, while the same LSP module already converts lexical scalar columns to
  UTF-16. The correction is one tooling file, zero compiler phases, deterministic,
  and changes no parser or language semantics. R-002 entry validation is not
  selected because valid entry forms and the quarantined direct-analyzer
  compatibility change are not unanimously frozen. R-010 grammar-authority
  containment remains the bounded runner-up. R-012 remains conditional on an exact
  dormant-inventory definition. Every other risk retains its semantic,
  architectural, policy, evidence, or hardware stop.
- Status: complete, strictly read-only, result commit none. Only the separately
  frozen `CORE-024` contract below may proceed; this audit authorized no test or
  implementation edit.

## CORE-024 - Project parser diagnostic columns to LSP UTF-16

- Task ID/date/owner: `CORE-024`, 2026-08-03, lead-owned zero-compiler-phase R-009
  LSP presentation slice under DEC-029, with independent type/safety, IR/codegen,
  and backend/claim review at every publication boundary.
- Observed behavior: `syntax_diagnostics` retains the complete source string but
  passes parser failures to `diagnostics_from_error` without it. The parser adapter
  subtracts one from `SourceLocation.column` and writes that Unicode-scalar offset
  directly into `LspPosition.character`. In the same file, lexical diagnostics
  already project scalar columns by summing `char::len_utf16`. Because the lexer
  advances source columns once per Rust `char`, a parser error after a non-BMP
  character is one UTF-16 code unit too early.
- Hypothesis: pass the source string through parser single/multi-error diagnostic
  conversion and project the zero-based scalar start column to UTF-16 exactly at the
  LSP boundary. Parser, AST, internal `SourceLocation`, recovery, semantics, IR,
  verifier, codegen, CLI, and backend behavior remain unchanged.
- Frozen behavior: internal parser locations remain one-based line and Unicode-
  scalar columns. LSP parser-diagnostic lines remain zero-based. Start character is
  the sum of UTF-16 widths of source-line characters before the scalar column.
  Preserve the current synthetic one-UTF-16-unit parser end range, severity `1`,
  source label `aero-parser`, exact message rendering, recursive multi-error order,
  ASCII positions, and empty-valid-program behavior. This is coordinate projection,
  not a token/AST span model or recovery claim.
- Tests first: change only the unit-test module in `src/compiler/src/lsp.rs`. Add one
  target `parser_diagnostic_columns_use_utf16_coordinates` using strict
  `syntax_diagnostics` on `let text = "😀"; let ;`. Require one parser diagnostic at
  line `0`, UTF-16 start/end `21/22`, unchanged source label/severity, and a parser
  message rather than a lexical wrapper. The unchanged basis must fail exactly that
  target with scalar `20/21`; existing ASCII, multi-error, valid-program, and lexical
  UTF-16 tests must pass. Review and publish this tests-only tree before production.
- Implementation files: only `src/compiler/src/lsp.rs`, the tests-first content in
  that file, and these six control records. Thread `source` through the private
  parser diagnostic adapter and calculate the start column from the selected source
  line. Do not change lexer projection, parser/lexer APIs outside this private LSP
  adapter, source locations, diagnostic messages, or any compiler phase.
- Acceptance: tests-only public evidence reproduces exactly one failing LSP unit
  target in each compiler job and every stable/nightly Rust job that reaches it,
  with fail-fast cancellation recorded exactly and all CodeQL analyses green.
  Implementation passes the focused new target, all existing LSP tests, exact
  `./tools/test.sh`, three exact-snapshot reviews, unchanged publication, and all
  eight public checks. The astral-prefix parser diagnostic must be `21/22`; ASCII and
  lexical controls must remain byte-for-byte equivalent at their asserted boundary.
- Compatibility decision: LSP consumers of parser diagnostics after non-BMP source
  characters intentionally receive protocol-correct UTF-16 coordinates instead of
  scalar offsets. ASCII results and all internal/compiler diagnostics are unchanged.
  This accepts no new program and changes no language or ABI behavior.
- Risks: off-by-one conversion; converting lines twice; using UTF-8 bytes instead of
  UTF-16 code units; changing end-width/span semantics; reordering multi-errors;
  changing lexical diagnostics; or misrepresenting this adapter as full trustworthy
  source ranges.
- Stop conditions: any lexer, parser, AST, recovery, `SourceLocation`, semantic, IR,
  verifier, codegen, ABI, CLI, symbol/completion-position, backend, or grammar change;
  any token-width/AST-span redesign; more than this LSP presentation layer; new
  dependency/workflow, benchmark, package/release/registry, immutable claim evidence,
  history rewrite, destructive-system, or `master` action.
- Status: preregistered and full-local-gate green. No test or production edit is
  authorized until this exact six-record snapshot passes three exact reviews,
  unchanged publication, and all eight public checks.
