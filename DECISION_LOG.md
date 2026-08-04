# Aero Decision Log

## DEC-001 — Stability is evidence-based, not label-based

- Date: 2026-08-02
- Status: accepted
- Decision: Treat the repository and all unaudited features as experimental until
  their rows satisfy the gates in `SPEC_IMPLEMENTATION_MATRIX.md`. Historical
  `1.0.0` labels do not establish stability.
- Evidence: `src/compiler/Cargo.toml` declares `0.3.0`; README and CLI declare
  `1.0.0`; README also calls the repository experimental. Stable-release gates
  have not been demonstrated in the current audit.
- Alternatives rejected: accepting the largest existing version label; changing
  package or CLI versions before completing the compatibility audit.
- Compatibility consequences: none yet. A later version unification requires a
  documented migration/release decision and tests.
- Revisit when: the capability audit and version policy are complete, or all 1.0
  release gates are satisfied.

## DEC-002 — Determinism checks are regression evidence, not formal proof

- Date: 2026-08-02
- Status: accepted
- Decision: Keep deterministic lexer/parser/IR/lowering checks, but classify them
  as executable regression checks. Do not call them mechanized formal-semantics
  proofs without a proof system, model, and checked correspondence.
- Evidence: the current conformance report contains three program cases and four
  equality-by-repetition checks implemented in `src/compiler/src/conformance.rs`.
- Alternatives rejected: deleting useful checks; treating repeatability as proof.
- Compatibility consequences: documentation terminology may need correction;
  executable behavior does not change.
- Revisit when: Aero adopts an explicit formalization and machine-checked proof
  workflow connected to compiler behavior.

## DEC-003 — One canonical compiler pipeline

- Date: 2026-08-02
- Status: direction accepted; implementation pending audit
- Decision: The library will be the canonical compiler implementation. The CLI,
  LSP, benchmarks, and other tools should consume shared library phase APIs and
  options rather than compiling through separately declared module copies.
- Evidence: `src/compiler/src/lib.rs` and `src/compiler/src/main.rs` independently
  declare overlapping compiler modules; current behavior can diverge silently.
- Alternatives rejected: maintaining parallel pipelines; making the binary the
  canonical API and leaving library consumers approximate.
- Compatibility consequences: internal imports and visibility will change; CLI
  behavior must be locked by regression tests before refactoring.
- Revisit when: audit shows a component cannot safely be shared without a
  deliberate query/API boundary.

## DEC-004 — Parser failure is fatal

- Date: 2026-08-02
- Status: accepted for `CORE-001`
- Decision: Any parser diagnostic rejects the compilation before semantic
  analysis, IR generation, optimization, backend lowering, or artifact output.
  Library entry points return an error and CLI entry points return nonzero.
- Evidence: the active legacy wrapper converts parser failure to an empty AST;
  `let = ;` then exits zero and writes an unterminated LLVM function.
- Alternatives rejected: preserving partial/empty AST compilation; relying on a
  later LLVM verifier to reject source syntax errors; changing grammar/recovery
  behavior in the same slice.
- Compatibility consequences: callers that treated malformed input as a
  successful empty program will now receive an error. That behavior violated the
  formal syntax and compiler invariants and is not a compatibility guarantee.
- Revisit when: a future IDE-only recovery API is designed. Such an API must keep
  erroneous nodes and diagnostics explicit and must remain ineligible for codegen.

## DEC-005 — Surfaced compiler failures use failing process status

- Date: 2026-08-02
- Status: accepted for compiler-oriented CLI commands
- Decision: When `build`, `check`, `run`, `profile`, or the discovered test suite
  surfaces a parse, semantic, artifact-write, or test failure, the command returns
  nonzero. A printed error with status zero is not an acceptable compiler result.
- Evidence: review of `CORE-001` observed that the necessary `Result` propagation
  also corrected pre-existing zero statuses for semantic and output-write failures;
  `aero test` likewise printed failed cases but returned success.
- Alternatives rejected: classify a printed compilation failure as CLI success;
  weaken parser propagation to preserve accidental zero-status behavior.
- Compatibility consequences: scripts that relied on erroneous zero statuses must
  use the corrected exit contract. Language syntax and semantics do not change.
- Revisit when: CLI error categories are represented by a shared typed diagnostic
  and exit-status API under the canonical library pipeline.

## DEC-006 — Strict lexing is mandatory for trusted compilation

- Date: 2026-08-02
- Status: accepted for `CORE-002`
- Decision: Artifact-producing and validation entry points must use a fallible
  lexer. An unexpected character, invalid number, or unterminated string produces
  a located error and no token stream. A legacy recovery lexer may temporarily
  remain for compatibility, tests, and editor symbol recovery, but its output is
  not eligible for semantic analysis or artifact generation.
- Evidence: `@` is currently printed then discarded; overflowing integers become
  zero; unterminated strings become completed tokens. Two invalid programs compile
  successfully to observably different LLVM values.
- Alternatives rejected: add an ignorable error token; rely on the parser to notice
  changed token streams; remove legacy APIs and rewrite broad parser tests in the
  same slice; silently clamp or substitute invalid numbers.
- Compatibility consequences: invalid programs that previously compiled after
  mutation now fail. Valid tokenization and the legacy public function signatures
  remain unchanged in this slice.
- Revisit when: every consumer has migrated to a diagnostic-accumulating lexer and
  the recovery API can be made explicit or removed with a migration plan.

Review clarification: public conformance is not exempt merely because its current
fixtures are hardcoded. Its semantic and IR checks must consume strict located
tokens and fallible parsing. Documentation validates declared direct modules before
writing output. LSP lexical diagnostics identify the lexer and use UTF-16 positions;
editor symbol indexing remains the sole intentional recovery consumer in these paths.

## DEC-007 — Numeric call boundaries require exact types

- Date: 2026-08-02
- Status: accepted for `CORE-003`
- Decision: For the first checked function-contract slice, monomorphic numeric
  call arguments must exactly match declared parameter types after alias
  canonicalization (`int`/`i32`, `float`/`f64`). Omitted return type is
  `void`. Primitive non-void functions must return a matching value on all
  conservatively recognized paths or provide a matching body-tail expression.
- Evidence: the formal language specification says argument arity and types must
  match and return expressions must unify with the declared return type. It lists
  implicit `int`-to-`float` promotion under mixed arithmetic, not function calls.
  The dormant call validator also uses exact equality.
- Alternatives rejected: infer every call as `int`; silently widen call arguments;
  accept missing returns and synthesize zero; claim generic/composite support in the
  same slice; validate semantically while retaining a contradictory IR call type;
  include boolean signatures despite their unresolved `i1`/`double` lowering.
- Compatibility consequences: invalid programs that previously emitted artifacts
  now fail before lowering. Forward references and recursion become valid through
  declaration collection. Generic and non-primitive contract behavior remains
  experimental and is not certified by this decision. Boolean function contracts
  remain open until their backend representation is coherent.
- Revisit when: a typed conversion/coercion policy is specified, generic
  instantiation is implemented, or control-flow analysis gains a typed CFG and a
  divergence/never type.

Implementation note: exact clean candidate
`8d5d8e7cc92f712fccc3af65cc4f06a1d7b1dd9a` was accepted after two corrective
review rounds, 13 focused tests, the complete repository gate, fresh black-box
artifact checks, and approval by two independent reviewers. The implementation
preserves public function-table shape, recognizes only outer function body tails
as implicit returns, restores lexical callable bindings, and emits terminator-safe
checked `if` arms and reachable void epilogues. Boolean/richer signatures and the
general unreachable-after-terminator problem remain explicitly outside DEC-007.

## DEC-008 — Numeric binding annotations constrain initialization exactly

- Date: 2026-08-02
- Status: accepted for `CORE-004`
- Decision: An initialized binding annotated `int`/`i32` or `float`/`f64` must
  receive an initializer of the same canonical numeric type. Existing expression
  inference runs first; no binding-site widening, narrowing, or invented value is
  permitted.
- Evidence: the parser preserves the annotation but semantics and IR discard it.
  At `4df60153`, every numeric cross-family literal and checked-function-result
  mismatch tested by the boundary audit passes `check`/`build` and emits an artifact.
  The formal rules give expressions a type and require declared function boundaries
  to unify; they specify integer-to-float promotion for mixed arithmetic, not for
  binding assignment. Backend local slots are uniformly `double`, so annotation-site
  coercion would require a larger typed-IR/backend decision.
- Alternatives rejected: continue treating the annotation as documentation; silently
  coerce integer initializers to float; narrow float initializers to integer; alter
  mixed-arithmetic inference; claim that a `double` stack slot proves source binding
  type; include non-numeric or uninitialized declarations in the same slice.
- Compatibility consequences: previously accepted initialized numeric mismatches
  become semantic errors and cannot emit artifacts. Exact aliases, inferred bindings,
  existing mixed-arithmetic results, mutable bindings, and nested shadowing retain
  their behavior. Non-numeric and uninitialized annotation semantics remain
  experimental and uncertified.
- Revisit when: Aero specifies general assignment conversions, adds reassignment and
  definite-initialization semantics, or introduces a typed local-storage IR.

Review amendment: candidate `5fa5a5e` correctly enforced the semantic annotation
boundary but was rejected because a nested scalar shadow remained active in IR after
its lexical block. A post-block call loaded the inner float slot, converted it to
`i32`, and passed the wrong value. `CORE-004` therefore opens only the existing
IR-generator symbol-snapshot restoration sites: complete scalar and callable
bindings must be restored at lexical scope exit. Weakening nested shadowing to fit
the miscompile is rejected; parser/AST, IR shape, code generation, and assignment
semantics remain frozen.

Second review amendment: `b6b0eba` restored IR bindings correctly but exposed a
semantic compatibility-table leak: a then-arm-only scalar was accepted in `else`,
then panicked during IR lookup. `CORE-004` therefore also restores that private flat
table at the analyzer's existing lexical/function/loop scope exits. The structured
`ScopeManager` remains authoritative. Invalid cross-scope references must stop with
a semantic error; treating a compiler panic as an acceptable consequence of legacy
compatibility is rejected.

Implementation closure: exact clean candidate `bc9a148` was accepted after the two
review-rejected candidates and corrective red checkpoints. It passes 18/18 focused
annotation/scope tests, 13/13 function-contract tests, the complete repository gate,
and two independent reviews. The accepted implementation restores private semantic
compatibility snapshots and complete IR bindings at existing scope exits while
enforcing exact initialized numeric annotations. It does not certify uninitialized
or non-numeric annotations, reassignment, typed local storage, or general fallible IR.

## DEC-009 — Parsed modulo fails closed until remainder semantics are frozen

- Date: 2026-08-02
- Status: accepted for `CORE-005`
- Decision: Aero continues to lex and parse `%` with multiplicative precedence,
  but active shared type inference rejects every modulo expression with
  ``Binary operator `%` is not supported.`` Such expressions cannot reach IR.
- Evidence: at `c000d916`, semantics accepts integer, float, mixed, and zero-RHS
  modulo. Both active IR expression paths omit it and panic; `check` reports false
  success, `build` exits 101, and public `compile_program` unwinds. The IR has no
  remainder instruction and integer locals use a unified LLVM `double` storage path.
- Alternatives rejected: map every modulo to LLVM `frem`; add `Mod`/`FMod` without
  freezing observable semantics; keep `check` successful while `build` panics; or
  remove `%` from lexing/parsing and lose explicit source structure. A `frem` patch
  would decide integer, negative, floating, mixed, zero-divisor, NaN, infinity, and
  signed-zero behavior without a language contract.
- Compatibility consequences: syntactically valid `%` programs now receive an
  earlier stable diagnostic. This is a temporary formal-grammar conformance
  exception, not a removal of an executable compatibility guarantee: no audited `%`
  source successfully generated an artifact. Tutorial and capability records must
  identify the operator as recognized but unsupported.
- Revisit when: remainder behavior is specified for integer and floating operands,
  zero and exceptional inputs are defined, integer representation is trustworthy,
  and one owner can implement semantics, IR, backend, runtime, and conformance tests
  as a complete vertical slice.

Implementation closure: exact clean candidate `302211e` was accepted. The one-file
production change is `028bb5e`; it adds a dedicated `%` error arm to shared binary
inference and leaves `+ - * /` unchanged. Fourteen focused tests, the full repository
gate, corrected tutorial text, and two non-owner reviews prove trusted public/CLI
paths return the frozen diagnostic without unwind, panic, or artifact. Constructed
AST callers that bypass semantics and all remainder execution behavior remain open.

## DEC-010 — Tuple values fail closed until aggregate semantics are implemented

- Date: 2026-08-02
- Status: accepted for `CORE-006`
- Decision: Aero retains tuple value syntax, tuple types, tuple patterns, and tuple
  indexing syntax, but active semantics recursively rejects every tuple literal and
  tuple-index expression with `Tuple expressions are not supported.` Tuple structs
  and tuple-like enum declarations are outside this decision.
- Evidence: at `704b3328`, the valid specified expression `(7, 9).0` succeeds in
  both `check` and `build`, yet the generated LLVM stores zero. The analyzer maps
  both tuple AST forms to `int`; both IR expression paths replace them with an
  integer-zero constant. Hidden tuple nodes can evade shallow parent inference.
- Alternatives rejected: implement layout/projection without a typed aggregate IR;
  reject only tuple indexing while tuple literals still fabricate zero; reject only
  literals while a constructed projection remains accepted; or patch only the two
  direct inference arms and leave nested tuples under skipped parents eligible.
- Compatibility consequences: tuple value source changes from silent miscompilation
  or false success to a stable early diagnostic. Parenthesized scalar grouping and
  the established array/index/iterator slice are unchanged. Tuple types and patterns
  remain parsed but are not certified as executable tuple semantics.
- Revisit when: tuple element types, layout, construction, projection, bounds,
  ownership/copy behavior, ABI, typed IR, backend lowering, and end-to-end positive
  tests can be delivered as one coherent vertical slice.

Implementation closure: the tests-only red commit is integrated as `6a75f93` and
the one-file production change as `1fa67a2`; public status documentation is corrected
at `669588d`. Exact clean candidate `cbbe049` passes all 73 focused tests and the
complete repository gate. Two non-owner reviewers approved that exact SHA with no
P0-P3 finding after independent structural, public-library, CLI, nested-expression,
diagnostic-ordering, no-unwind, no-panic, and no-artifact probes. Constructed AST
callers that bypass semantics and tuple layout/execution remain open by design.

## DEC-011 — Named field values fail closed until struct projection is implemented

- Date: 2026-08-02
- Status: accepted for `CORE-007` at exact reviewed candidate
  `4e10d4799b7873741a5eae9c66ac352b1709d75c`
- Decision: Aero retains named field-access syntax and its AST node, but active
  semantic preflight recursively rejects every field-access value expression with
  `Field access expressions are not supported.` The receiver is preflighted first
  so already accepted tuple and void-call diagnostics retain precedence; otherwise
  the field diagnostic occurs before receiver inference.
- Evidence: at `52d3415`, `Point { x: 7 }.x`, bound/literal/call/undeclared/chained
  receivers, nested forms, and direct modules falsely succeed and emit zero without
  a field GEP. Receiver calls are dropped. Both semantic inference paths invent
  `int`; both IR expression paths return integer zero without visiting the receiver.
- Alternatives rejected: implement struct layout/projection without a typed
  aggregate contract; continue silent zero lowering; reject methods with dot syntax
  even though the parser represents MethodCall separately; select string comparison
  policy across six operators; or patch only immediate integer `/ 0` while computed,
  variable, float, mixed, and unary-zero semantics remain unresolved.
- Compatibility consequences: named field value source changes from silent
  miscompilation/false success to one early diagnostic. Method calls, tuple indexing,
  struct declaration/literal syntax, arrays/indexing/iteration, and adjacent numeric
  behavior retain their existing behavior but are not newly certified beyond their
  established slices.
- Revisit when: struct identity, field definitions/types, layout, construction,
  projection, assignment, ownership, evaluation order, ABI, typed IR, backend
  lowering, and end-to-end positive tests can ship as one coherent vertical slice.

Implementation closure: the tests-only red checkpoint is integrated as `7346edd`,
the one-line production change as `75dbfba`, and public documentation as `5dcb70b`.
Exact clean `4e10d4799b7873741a5eae9c66ac352b1709d75c` passes all 81 focused tests and
the complete repository gate. Two non-owner reviewers approved that exact SHA with
no P0-P3 finding after independent structural, public-library, CLI, module,
nested-expression, diagnostic-ordering, no-unwind, no-panic, no-artifact, parser,
and positive-control probes. Direct semantic bypass, field assignment, struct
execution/layout/ownership, unknown methods, and parent composites remain open by
design.

## DEC-012 — Match values fail closed until pattern semantics and lowering exist

- Date: 2026-08-02
- Status: accepted at exact clean `b74d91a`
- Decision: Aero retains Match syntax, its AST node, arms, and Pattern
  representation, but every Match value expression in a trusted parsed source body
  must reach active semantic preflight and reject with
  `Match expressions are not supported.` The existing
  scrutinee-first, arm-body-in-source-order traversal remains first so accepted
  child diagnostics retain precedence; the Match error occurs before invented
  result-type inference. Default trait method bodies are parser-retained but excluded
  from full semantic analysis, so a dedicated syntax-only statement/block walk must
  visit their expression roots in source order and reuse `preflight_expression`.
  This walk must not resolve names, bind parameters, infer return types, validate
  traits, or activate ownership, pattern, or execution semantics.
- Evidence: `AUDIT-013` found no active value-preserving Match path. Across 23 cases
  and 69 public/check/build outcomes, 20 ordinary Match forms falsely succeeded with
  zero or dropped evaluation; field, tuple, and void-valued child controls retained
  their established diagnostics. Both semantic inference paths return `Int`; both
  IR paths return zero without visiting Match children. Root Match can emit an empty
  `main`, and hidden calls or `/0` disappear.
- Alternatives rejected: implement pattern binding/exhaustiveness and enum lowering
  without a typed aggregate/CFG contract; continue fabricated-zero behavior; reject
  all comparisons and regress numeric controls; reject all MethodCall and regress
  array `.iter()`; patch selected `/0` syntax without arithmetic/runtime/IEEE policy;
  or bundle struct/enum/borrow/deref families that require layout or ownership rules.
- Compatibility consequences: source Match programs that falsely reported success
  will receive one stable early diagnostic. Parsing, AST construction, and future
  tooling inspection remain available. Existing child diagnostics and established
  numeric/function/array/index/iterator and prior fail-closed boundaries are
  preserved. Parent composites are not certified by recursion.
- Revisit when: pattern binding and typing, exhaustiveness/reachability, scrutinee
  evaluation, arm selection and result unification, enum/aggregate representation,
  ownership/destruction, typed CFG/IR, backend lowering, and end-to-end positive and
  negative tests can ship as one coherent vertical slice.

Initial candidate: the tests-only red checkpoint is integrated as `851731c` and the
one-line production change as `c826294`. Exact clean documented `08e7c2c` passed the
complete repository gate, all 90 focused tests, formatting, and
`cargo check --all-targets`, but independent review rejected it. Reviewer A proved
that a Match in a default trait method body bypasses analysis and succeeds through
the public API, check, and build, with build writing an artifact. Its structural
audit found no second parsed expression-bearing container escape. Reviewer B
approved 41 other routes but did not exercise trait defaults; acceptance therefore
remains denied.

Corrective amendment: preserve the failed route as a regression before production
changes. Add a structural block/statement preflight funnel used only for default
`TraitMethod.body` values, preserve statement and child order, and keep full trait
method analysis inactive. Calling `analyze_block` or `analyze_statement` for those
bodies is outside this decision. A new complete gate and two new exact-SHA non-owner
approvals are mandatory.

Corrective candidate: tests-only owner `58bb732` is integrated as `ad5e24d`; the
four-test red split proves public, precedence, root CLI, and module CLI false success
without parse failure, unwind, or panic. Production owner `a3f4f29` is integrated as
`a12f38e`; it adds only syntax-level block/statement preflight helpers and the
default-body hook in `semantic_analyzer.rs`. The original nine and new six Match
tests plus all prior focused boundaries pass 96/96 independently for owner and lead.
The exact clean documented candidate `b74d91a` subsequently passed the complete
repository gate and two new non-owner reviews. Reviewer A exhaustively checked all
17 statement variants and 44 fresh public negatives; Reviewer B checked 75 fresh
negative/precedence routes across 225 public/check/build outcomes. Both approved with
no trusted false success, panic, unwind, or negative artifact. `CORE-008` is accepted
at that SHA; direct constructed-AST bypass and Match/default-trait execution remain
outside this decision.

## DEC-013 — Struct construction fails closed until aggregate semantics exist

- Date: 2026-08-02
- Status: accepted at exact reviewed candidate
  `daa024dbf10d1defe06d8ab200c2d21c0a9c1dc6`
- Decision: Aero retains struct declarations and StructLiteral syntax/AST, but every
  StructLiteral in a trusted parsed source body will visit field expressions in
  source order and then reject with
  `Struct construction expressions are not supported.` No declaration lookup, field
  validation, type inference, layout, ownership, IR, or execution semantics are
  introduced by this boundary.
- Evidence: `AUDIT-014` found no value-preserving StructLiteral route. Nineteen of 24
  public cases falsely succeeded, both IR paths return scalar zero without fields,
  and root/module builds write artifacts that omit field calls and aggregate text.
  Unknown structs and missing/extra/duplicate/wrong fields are accepted. Existing
  source construction positives are therefore false capability claims, not runtime
  compatibility evidence.
- Diagnostic consequences: tuple, field, Match, and void-as-value field errors keep
  precedence through existing preflight. Inference-only modulo/name/type diagnostics
  are not newly activated and the Struct error wins after preflight. Existing field/
  Match controls are explicitly reclassified to parser retention and declaration-
  only compilation; active move/trait controls use struct-typed parameters instead
  of fabricated runtime values.
- Alternatives rejected: combine EnumVariant despite Option/Result policy; reject
  Borrow despite active shallow ownership diagnostics; select Deref while replacing
  real non-reference errors; reject all MethodCall and regress `.iter()`; reject all
  comparisons and regress numeric operations; or patch selected zero divisors
  without arithmetic/runtime/IEEE policy.
- Revisit when: struct name/field validation, aggregate result typing, layout,
  initialization order, ownership/destruction, ABI, typed IR/backend emission, and
  positive end-to-end execution can ship as one coherent vertical slice.

Tests-first checkpoint: exact integration commit `1e76a06` changes only the four
authorized test files. Independent lead verification reproduces exactly 3 passing
and 6 expected-failing aggregate tests. Failures expose accepted StructLiteral
routes, competing outer diagnostics, fabricated zero/drop LLVM, successful CLI
status, and root/module artifacts; no source fails parsing or unwinds. The rewritten
frontend, field, and Match controls pass 59/59, 8/8, and 15/15 respectively. The
production decision remains unchanged.

Production candidate: owner `bf6a7ef` is integrated as exact `a887931`. It changes
only `semantic_analyzer.rs` by returning the frozen diagnostic after the existing
source-order recursive field traversal. Owner validation passes the Struct/prior
focused matrix, formatting, all-target check, and complete repository gate. Lead
validation independently passes 164/164 focused Struct/frontend/field/Match/tuple/
modulo/function/annotation/strict tests. No inference, declaration/field validation,
layout, ownership, IR, or backend behavior changes. Public documentation, a fresh
documented complete gate at `3410f1f`, and the public capability/historical notices
are complete. The lead reran the complete gate on exact clean `daa024d`. Two fresh
non-owner reviewers then verified that its delta from `3410f1f` is limited to the
five coordinated control documents, that all stale resumption/status text is
corrected, and that production and public-truth files are unchanged. Both approve
exact `daa024d` with no P0-P3 findings; `CORE-009` is accepted at that SHA.

## DEC-014 — Founding framework directs the roadmap but does not certify status

- Date: 2026-08-02
- Status: accepted
- Decision: the two tracked founding PDFs are explicit project inputs. The primary
  nine-page paper defines durable language and implementation direction. The Claude
  artifact contributes execution-quality and AI/ML-infrastructure strategy only to
  the extent preserved in its one visible, truncated page. Neither artifact is
  evidence that a feature, phase, backend, benchmark, or release is complete.
- Authority: accepted specifications/RFCs define intended semantics; the capability
  and backend matrices plus executable evidence define current status; control logs
  define active work; the roadmap sequences future gates. A conflict is resolved by
  reporting the evidenced implementation honestly while retaining the vision as a
  goal.
- Consequences: Aero is classified as Minimal Prototype / correctness recovery, not
  Stabilize or Optimize. Historical completed-phase and v1.0 labels are not maturity
  evidence. `Roadmap.md` is replaced with evidence-gated milestones that retain the
  founding Design -> Minimal Prototype -> Self-Host -> Stabilize -> Optimize path.
- Killer application: AI/ML infrastructure is the lead adoption wedge. The first
  qualifying flagship must be Aero-native and correctness-gated on CPU before any
  accelerator claim. External llama.cpp evidence remains a baseline/reference and
  does not demonstrate Aero execution.
- Measurement: execution quality is multi-dimensional across language correctness,
  compiler/IR integrity, safety, performance/resources, developer experience, and
  reproducibility. Large benchmark suites are deferred until Aero can compile their
  required programs correctly.
- Compatibility: this decision changes status and sequencing documentation only. It
  does not freeze new syntax/semantics, broaden a backend, accept a benchmark claim,
  or alter the preregistered `CORE-009` boundary.

## DEC-015 — Checked logical IR precedes physical numeric redesign

- Date: 2026-08-02
- Status: accepted for `CORE-010` at public head
  `db349ef81f145ee571c053f73fb03c831cea719a`
- Decision: Aero will add explicit logical types for admitted scalar results,
  places, arrays, calls, branches, and returns; a mandatory in-process IR verifier;
  additive checked IR/codegen APIs; exhaustive codegen errors; and final external
  LLVM module verification. No trusted compiler route may select a public unchecked
  compatibility helper as its boundary or consume output that has not passed the
  checked wrapper's preflight and mandatory verification once this slice is accepted.
- Representation: `Int`, `Float`, `Bool`, and `Void` remain distinct in IR;
  restricted string immediates and fixed numeric arrays have explicit limited
  roles. Admitted non-capturing scalar closures are compile-time callable aliases
  with explicit signatures and no runtime value/place/ID; capture and escape remain
  unsupported. Function ABI remains `int`/`i32` -> `i32`, `float`/`f64` -> `double`, and
  `bool` -> `i1`. Boolean slots/results use `i1`; results and places cannot share an
  identifier kind; void has no operand. Unknown source/IR types never map to
  `double` by default.
- Compatibility limit: the accepted legacy local numeric `double` representation
  and numeric-array storage are preserved initially behind logical type metadata.
  The founding vision favors exact native types, but repository specifications do
  not settle integer overflow/division and active tests explicitly preserve current
  scalar lowering. A physical all-`i32` migration would silently choose those
  semantics, so it requires a later RFC-backed decision rather than this repair.
- Admission consequence: checked constant folding cannot panic; out-of-range `i32`
  literals, constant integer division by zero, string comparison, unsupported
  expression-to-scalar fallback, and type-invalid Boolean storage/calls/returns are
  structured failures. Dynamic overflow/division, richer arrays, strings, and
  aggregates remain uncertified. A fold whose in-range operands produce an out-of-
  range result remains an unfurled logical operation; this slice neither wraps,
  truncates, materializes, nor rejects the result as a new overflow policy.
- `check`: `aero check` will perform frontend validation plus typed-IR admission and
  the in-process verifier, without emitting LLVM or consulting external tools. This
  is stronger than its current semantic-only implementation but still does not
  promise final backend representability. Help and capability text must state the
  migration.
- API: preserve `compile_program -> Result<String, String>`. Add checked
  `try_generate_ir`/`try_generate_code` APIs with structured errors, migrate every
  trusted caller, and retain existing unchecked IR helpers only as excluded
  compatibility surfaces until a major break. Method/free `generate_code` is marked
  deprecated; public raw `IrGenerator::generate_ir` is not yet deprecated and must
  be separately deprecated/restricted before removal. Checked compatibility never
  means empty/partial output, embedded error text, or panic catching in production.
- Checked/legacy boundary: every `try_generate_code` entry re-verifies raw private
  IR and preserves IR Verification as its own error variant. `try_generate_ir`
  internally reuses the raw generator only after checked preflight/mode activation
  and then mandatorily verifies its result; trusted callers never enter or consume
  that raw public boundary directly. Deprecated unchecked codegen retains its
  separate legacy implementation and historical behavior. The public unchecked
  APIs are deprecated/restricted then removed at a major boundary rather than
  gaining new adapter fallbacks, error-text output, or newly implicit panics.
- Diagnostics: preserve existing Lex/Parse/Semantic prefixes and add stable IR
  Generation, IR Verification, Code Generation, and LLVM Verification phase
  prefixes. IR-generation/verification/codegen errors precede transformations. On
  source compiler routes, external LLVM verification follows transformation/
  retargeting and precedes cache/write, native tools, and trace publication;
  standalone graph-opt/quantize also verify input before transformation. `check` keeps its current raw
  semantic text and prefixes only new IR failures; profiler keeps its established
  semantic wording and uses the new later-phase labels.
- LLVM policy: the pure-Rust IR verifier is always required. Final post-transform,
  post-retarget LLVM uses LLVM 22 `opt -passes=verify` with `llvm-as` fallback.
  Text build may visibly report `InternalOnly` only when no verifier exists; a found
  verifier failure is fatal. Run/object/evidence/CI paths require the tool. No LLVM
  Rust dependency is introduced. `clang`/`llc` remain downstream evidence, not
  substitutes for module verification.
- LLVM route/mode policy: standalone `graph-opt` and `quantize` require external
  verification of arbitrary input and final output. Run and object paths are also
  always `Required`. Text build selects `Required` by
  `--require-llvm-verifier` or `AERO_REQUIRE_LLVM_VERIFIER=1|true`; CI sets the
  environment explicitly. Forced command/flag status cannot be downgraded.
  Explicit `AERO_LLVM_OPT`/`AERO_LLVM_AS` paths are authoritative and fail closed;
  otherwise discovery uses LLVM-22 versioned tools before version-validated
  unversioned tools. A rejecting found verifier never triggers a fallback.
- Verifier process containment: the timeout is an end-to-end process-tree deadline,
  not only a wait on the direct wrapper. Unix verification runs in a dedicated
  process group. Windows creates the verifier suspended, assigns it to a kill-on-
  close job before its first instruction, then resumes it. Target-specific `libc`
  and `windows-sys` dependencies are authorized only for this containment boundary;
  no LLVM Rust binding or broader dependency expansion is introduced.
- Cache: final-LLVM string entries lack typed-IR provenance. They are usable only
  when an available external verifier accepts them. Under `PreferExternal`, missing
  tools force a fresh checked-IR rebuild; cached text can never be labeled
  `InternalOnly`. Cache schema/provenance redesign remains outside this slice.
- Tooling-result policy: profiler uses `PreferExternal` and records
  `InternalOnly` visibly in profile/trace metadata only on absence. Conformance uses
  checked internal IR only, records failures in its report, writes a requested full
  failure report, and exits nonzero when any case or mechanized check fails.
- Unsupported-form compatibility: current successful custom Enum/`Some`/`Ok`,
  ordinary MethodCall, and Deref/Borrow lowering is fabricated-zero behavior, not a
  positive execution contract. `CORE-010` explicitly supersedes those construction
  controls: syntax/declarations stay positive, runtime forms become checked-IR
  negatives, and Array/Vec `.iter()` remains admitted. Print-only immutable string
  aliases remain allowed; broader string operations do not.
- Red-checkpoint acceptance: exact public commit
  `26560a45905015b7891ddebeb749d0097c05cbaa` carries only focused tests, test-only
  hooks, reclassified controls, and LLVM 22 CI changes. Its exact staged diff hash is
  `c01fc2365eb5b415c022be997062e4605812b62b`; three independent reviewers approve it
  with no P0-P3 findings. Both public compiler workflows install LLVM 22 and reject
  the known-invalid fixture. Rust stable/nightly verify and execute the four positive
  CPU examples before reaching the deliberate missing checked-API failures. The red
  evidence therefore authorizes bounded production implementation under this
  decision without accepting any production behavior yet.
- Production acceptance: the implementation follows the frozen additive
  API and logical-type boundary, re-verifies checked/private IR at checked codegen,
  rejects unsupported or invalid forms with structured phase errors, and verifies
  final transformed/retargeted LLVM before cache, write, trace, or native-tool
  publication according to the required/preferred policy. Focused contracts and
  the complete repository gate pass. Three independent reviewers approved exact
  implementation diff `9534765a46b130d215a1d1e869de234163bb0daf` and exact
  mixed-entry CI repair `d5f0fd3891da5cff75bd5306006e993ca4b4f301`
  with no P0-P3 findings. Rust stable/nightly, both compiler-test workflows, and
  all CodeQL jobs pass at public head `db349ef`. Review closure also routes every
  successful named conformance case through checked IR and proves an immediate
  verifier descendant cannot escape the bounded timeout.
- Alternatives rejected: immediate physical `i32` lowering without overflow/
  division policy; keeping Boolean/numeric guessing; LLVM-only verification after
  unsafe IR; making ordinary Cargo builds depend on llvm-sys/Inkwell; treating
  `llc -verify-machineinstrs` as module verification; making `check` PATH-dependent;
  or breaking every existing public helper signature before additive migration.
- Revisit when: integer and array RFCs authorize exact physical lowering; a major
  API boundary can remove unchecked helpers; or an in-process LLVM construction
  architecture justifies a native LLVM dependency.

## DEC-016 — Direct modules fail closed before cache; namespaces remain unfrozen

- Date: 2026-08-02
- Status: accepted and implemented at `a711dd5f3802095a4ecbe2dea3d45003675e7459`
- Decision: every trusted path that accepts an entry-file context will use one
  strict direct-module source collector before semantic analysis, checked IR, cache
  lookup, or artifact publication. It preserves the current root-relative
  `x.aero` then `x/mod.aero` search order and source-order AST flattening only as a
  compatibility boundary, not as evidence of a complete module system.
- Source-only API: the existing `compile_program(source, options)` cannot resolve a
  file-backed declaration because it has no entry path. It will reject `mod`
  explicitly rather than consult the current directory, ignore the declaration, or
  add an unfrozen public API. Module-free callers remain source-compatible.
- Nested modules: a module source containing another `mod` declaration will be
  rejected explicitly. This is fail-closed compliance with the requirement that
  unresolved/circular declarations not compile, but it is not recursive resolution
  or cycle-graph evidence. Nested base-directory rules, namespaces, `use`, `pub`,
  visibility, and duplicate-name semantics remain unfrozen.
- Cache: module discovery and strict parsing precede cache lookup. The zero-module
  identity remains MD5 over the exact existing UTF-8
  `<root-source>::target=<target>::gpu=<gpu>` string. A module-bearing identity is
  MD5 over `AERO_MODULE_CACHE_V1\0`, then `frame("root", root_source)`,
  `frame("target", target)`, `frame("gpu", gpu)`, the raw unsigned 64-bit big-endian
  module count, and ordered `frame("name", name)`,
  `frame("candidate", candidate)`, `frame("source", source)` triples. Each frame is
  the exact lowercase ASCII label bytes shown above, NUL, unsigned 64-bit big-endian
  payload byte length, and raw UTF-8 payload bytes. The candidate is exactly
  `<name>.aero` or `<name>/mod.aero` using `/`; it is not canonicalized and includes
  no entry directory, drive, working directory, host separator, symlink target, or
  host case normalization. Deletion fails before lookup; an exact byte change or
  move between candidates cannot hit the prior module-bearing entry. Tests must
  also prove the legacy no-module key remains a real verified hit. Existing MD5 is
  retained for cache compatibility and is not treated as a security primitive.
- Caller policy: build/run, check, discovered test, profile, and documentation share
  the collector. Documentation validates module source but continues to render the
  root declarations only; `aero test` remains a semantic checker, not an execution
  runner, until separately redesigned. Command wrappers may add their established
  outer presentation, but may not change the shared inner phase failure.
- Claim policy: current public text must describe only direct source collection.
  Multi-file namespace, import, visibility, recursive graph, and cycle handling are
  not claimed until positive and negative end-to-end evidence exists.
- Implementation evidence: public red checkpoint `9c31820` records the exact
  independently approved contract. The accepted implementation centralizes all inventoried
  callers on one crate-private collector, preserves the legacy zero-module key,
  matches the frozen V1 known vector, and makes both focused seven-test suites plus
  the complete repository gate pass. Three independent reviewers approved exact
  diff `60fe607413ebc03e9aa5d6296d9067d8cc95d89d` and tree
  `7c57c082e9d5f68afd5c6a4769d9d531a0116642` with no P0-P3 findings; all public
  checks pass at `a711dd5`. No namespace, visibility, recursive path, cycle graph,
  `CompilerOptions`, or general CLI-status behavior was added.
- Alternatives rejected: a one-line return only in `build`; treating the emitted
  artifact as a warning-success result; hashing root source alone; resolving after a
  cache hit; guessing a path for the library API; or implementing recursive module
  semantics without a frozen namespace/path decision.
- Revisit when: a module-system RFC fixes nested path and namespace semantics, a
  file-aware library API and `CompilerOptions` are designed, or full pipeline
  consolidation can cross more than the bounded source-collection phase.

## DEC-017 — Quarantine live registry transport before designing its protocol

- Date: 2026-08-02
- Status: accepted and implemented at `6780a23cd8b63df124477c7db1190d61dd25f3b8`
- Decision: every HTTP-backed registry entry (`search_live_registry`,
  `publish_live`, and `install_live`) must fail with the exact stable diagnostic
  `live registry transport is disabled pending a reviewed protocol and trust boundary`
  before credential resolution, package/target filesystem access, process spawn,
  HTTP, download, digest handling, response acceptance, or writes. The CLI must
  return nonzero for each live attempt. Direct function callers retain the same guard
  even if the CLI is bypassed. `publish_live` and `install_live` reject for both
  values of their existing `dry_run` boolean; CLI previews never route through those
  live functions.
- Preserved local surface: local-index search, `build_publish_preview`, and
  `build_install_plan` remain available. CLI publish/install `--dry-run` use only
  those local preview/plan paths; local search and dry-runs do not resolve explicit,
  environment, default-file, or token-file credentials and do not invoke transport.
- Security basis: resolved package name/version are remote-controlled path material;
  the current join is not contained. The current publish request omits file content
  and has no versioned response/acceptance contract. Keeping either mutation active
  while repairing one symptom would expose the other incomplete trust boundaries.
  Read-only live search shares the same unaudited auth/HTTP machinery, so the
  quarantine is transport-wide rather than mutation-only.
- Evidence required before production: tests-only red controls must cover all three
  CLI live routes with exact nonzero diagnostics, invalid credentials and unavailable
  transport so guard ordering is observable, malicious install name/target inputs
  with no created destination, direct search rejection, both boolean modes of direct
  publish/install rejection, credential-free local search, credential-free CLI
  publish/install dry-runs, and existing offline positives.
- Files allowed: `src/compiler/src/registry.rs`, minimal registry dispatch/help in
  `src/compiler/src/main.rs`, one focused registry integration test file, existing
  registry unit tests, `README.md`, `BUILD.md`,
  `tutorials/01-getting-started.md`,
  `docs/language/aero_formal_language_specification.md`, and registry/capability/
  project-control documentation. Public workflow and specification-status text must
  state that live transport is currently quarantined while retaining the future
  design direction plus local-search and dry-run examples.
- Files frozen: lexer/parser/AST/semantics/IR/codegen/backend/cache/module behavior;
  package archive format, dependency resolution, URL/response/auth protocol,
  path-sanitization/containment design, general CLI status, benchmark execution,
  Cargo dependencies, releases, external registries, and `master`.
- Stop conditions: implementation requires a protocol choice, archive encoding,
  server response schema, credential migration, path normalization/containment,
  symlink/overwrite policy, dependency solver, more than the registry dispatch
  boundary, or any real external registry call. Encountering one of those conditions
  stops the slice; it does not justify inventing semantics or weakening the guard.
- Alternatives rejected: validating only `..`; sanitizing remote names into a local
  filename; trusting a digest to establish path safety; adding file bytes without a
  package format; accepting any 2xx response; disabling install while leaving publish
  or authenticated live search active; and fixing general CLI statuses in this slice.
- Revisit when: a versioned registry/package RFC freezes payload, response, auth,
  URL, archive, digest/signature, destination, overwrite/symlink, and dependency
  contracts with adversarial transport tests. Re-enablement requires a separate
  reviewed decision and cannot be inferred from `CORE-012`.
- Implementation evidence: public red checkpoint `57c4ec7` records the exact
  independently approved failing contract. The accepted implementation makes one
  shared guard first in every direct live function and checks it in CLI live dispatch
  before auth; local search and CLI dry-runs use only local credential/network-free
  paths. Exact implementation diff `05e55496f6664713192b2dbf94eca785abe2931d`
  and tree `85ed76ab0141409796e167704e4100dd4d15c26f` passed focused/full local gates and
  three independent reviews with no P0-P3 findings. All eight public checks pass at
  `6780a23`. No registry protocol or re-enablement was accepted.

## DEC-018 — CLI-owned process status is a typed public correctness boundary

- Date: 2026-08-02
- Status: implemented and accepted at public `CORE-013` head
  `a78dd004aa37c39212711027b777698118d9dc02`; focused/full local gates, three exact
  implementation reviews, and all eight public checks pass
- Decision: outcomes owned by the CLI before delegated program execution must use
  one typed status boundary: `0` for completed work and explicit help/version, `1`
  for operational or compiler failure, and `2` for invalid invocation. Printing a
  diagnostic never converts failure into success. No CLI-owned error path may rely
  on Rust `main` falling through to zero.
- Invocation class: no command, unknown top-level or registry command, missing or
  extra operands, bad/incomplete options, and unrecognized target/backend/mode
  values are status `2`. Top-level help/version must be standalone. `test` and `lsp`
  accept no operands, `check` and `fmt` exactly one input, and `init` zero or one
  path. Existing option order and duplicate-option behavior remain frozen rather
  than redesigned. A recognized but unavailable target fails operationally as it
  does today; it is not relabeled invalid invocation.
- Success class: standalone top-level `-h`/`--help` and `-v`/`--version`, plus
  standalone explicit `registry help|-h|--help`, return `0`. Registry help is made
  explicit so it no longer shares the unknown-subcommand fallthrough.
- Operational class: source/input/output/report failures; compiler, verifier,
  registry, initialization, LSP, and discovered-test failures; and failed
  conformance return `1`. Existing successful command behavior and diagnostic text
  remain compatible.
- Delegated-execution exception: after valid CPU `run` dispatch reaches program
  execution, `run_aero_program` continues to terminate with the compiled program's
  arbitrary exit code. It may equal `1`, `2`, or another value, so the numeric codes
  are not globally unique without command context. This is intentional pass-through,
  not a CLI-owned classification. Refactoring or remapping that helper is frozen.
- Benchmark consequence: the tracked `performance_benchmark.py` driver invokes a
  source path without a command. Under this contract that route returns `2`, so it
  cannot be counted as a successful compilation. This is fail-closed containment,
  not benchmark repair. The lexer and external llama.cpp records retain their
  separately audited qualifications.
  Both tracked Python compilation series are reclassified as invalid measurements;
  their artifacts remain intact. Benchmark code is not run or changed, and no
  performance statement is upgraded.
- Evidence required: a tests-only process matrix spanning every command family must
  fail on current zero-status branches, assert diagnostic text and stream placement,
  exercise bounded successful paths for every changed family, preserve an arbitrary
  delegated-program status, and assert exact `1` for representative parser/compiler,
  verifier/native-tool, registry-quarantine, discovered-test, conformance/report,
  init, and output-write failures. Unrecognized backend/target/mode values are exact
  `2`; recognized-but-unavailable execution and accepted graph/quantize operational
  failures are exact `1`. Failed direct writes and partial init assert status/no
  success message; atomic rollback is not promised. Exact review and the complete
  repository gate are required before publication.
- Alternatives rejected: treating all failures as `1`; keeping no-argument or
  unknown commands successful because help text is printed; fixing only the bare
  benchmark source path; parsing stderr in automation; rewriting the benchmark in
  the status slice; mapping delegated program results into `0/1/2`; promising
  transactional rollback; introducing a CLI dependency; or changing command
  maturity to justify an exit code.
- Revisit when: a separate CLI architecture task can return statuses instead of
  terminating within helpers, a command-maturity task implements real test/runtime
  behavior, or a benchmark task supplies correctness gates and protocol-complete
  immutable evidence. None is inferred from `CORE-013`.

## DEC-019 — The public Quick Start is an executable generated-project contract

- Date: 2026-08-02
- Status: implemented at public `c56b1d5` and closed at public `1535ce2`; exact
  implementation diff
  `687dd5f3d6360dfd7822e7809944f63d4caccfdd` and tree
  `869fca43edb8b5888bdec01d0bfc7cdecfa451a5` received three independent approvals,
  focused/full local gates pass, and all eight public checks pass. Stable Linux CI
  completed the exact documented path with external LLVM 22 verification, status
  zero, and exactly one `Output: Hello, Aero!` line. Exact closure diff
  `6e05c26763ed3a1c6e4ec359361867f76e9d4c4c` and tree
  `b3a6bf38769579dbfc0fa0da5c4881620f7129c3` received three approvals, and all
  eight public checks pass at `1535ce2`
- Decision: the canonical first-run path begins at the repository root, builds with
  `cargo build --release --manifest-path src/compiler/Cargo.toml`, places
  `src/compiler/target/release` on `PATH`, initializes a fresh project with
  `aero init`, checks its generated `src/main.aero`, and runs that same source on
  CPU when the documented LLVM 22/Clang prerequisites are present. The checked-in
  project scaffold is the only flagship program for this slice.
- Platform contract: README supplies truthful POSIX commands and points Windows
  users to exact PowerShell build/PATH/executable commands in `BUILD.md`; focused
  static tests bind both surfaces. Commands must remain valid after changing into
  the generated project; examples may not depend on a nonexistent root Cargo
  manifest or root `target/release` directory.
- Honesty boundary: unsupported `aeronum`/`aeronn` imports, grouped imports, named
  arguments, and model/distributed behavior are removed from the executable
  flagship. Accelerator, graph, quantization, registry, benchmark, LSP, and other
  experimental surfaces remain outside the minimal Quick Start and retain their
  separate capability qualifications. ROCm object generation and unavailable CUDA
  execution may not be described as successful device execution.
- Evidence required: focused tests must parse the Quick Start and Windows build
  sections, require the exact POSIX and PowerShell manifest/binary paths plus
  generated-project commands, reject unsupported flagship dependencies, and
  process-test `init` plus `check` in an isolated workspace. Stable Linux CI must
  put `/usr/lib/llvm-22/bin` first on `PATH`, assert that the resolved unversioned
  `clang` and `llc` paths come from that directory and both report major version 22,
  while retaining the exact `opt-22`/`llvm-as-22` verifier overrides. It must then
  execute the documented root build/init/check/CPU-run path, capture only the
  `aero run` output with status zero, and require exactly one anchored
  `Output: Hello, Aero!` line. The full repository gate and exact independent
  reviews remain mandatory.
- Frozen boundaries: no compiler or CLI behavior, project scaffold, language
  syntax/semantics, version/stability policy, backend implementation, package or
  registry protocol, benchmark driver/result, dependency, release, or `master`
  change is authorized. If the existing scaffold cannot complete the promised path
  without one of those changes, stop and narrow the documentation rather than
  expanding implementation scope.
- Alternatives rejected: adding a root Cargo workspace solely to preserve the bad
  command; keeping a conceptual model as the flagship; labeling absent packages as
  runnable; testing only Markdown strings; treating ROCm object emission as a run;
  repairing every tutorial/example in one slice; or choosing a 0.x/1.0 version.
- Revisit when: a separately reviewed grammar/version/backend decision supplies the
  evidence needed to expand beyond the generated CPU project. None is inferred from
  `CORE-014`.

## DEC-020 — Selected initialized binding annotations are exact pre-IR contracts

- Date: 2026-08-02
- Status: accepted at public closure head `5d7aae0`; preregistration approved at
  `4f31f0c`, tests-only red checkpoint independently approved and public at `b203ea4`,
  and independently approved production candidate public and green at `3f0578d`
- Decision: existing exact scalar annotation behavior for `int`/`i32`/`float`/`f64`
  remains unchanged wherever semantics fully analyzes binding statements; syntax-
  preflighted trait default bodies remain outside that enforcement. `CORE-015` adds
  a closed, nonrecursive, binding-local rule only in fully analyzed code when active
  semantic generic type-parameter scopes are empty: `Type::Named("bool")`,
  `Type::Named("String")`, and one-dimensional
  `Type::Array(Type::Named(name), count)` when `name` is one of the four numeric
  spellings and `count > 0`. A selected initialized annotation must exactly equal the
  fully inferred value `Ty`. Numeric aliases normalize as before; `bool` maps to
  `Ty::Bool` and canonical `String` to `Ty::String`. Lowercase `string` is not selected. Checked IR
  mirrors numeric/bool/canonical-String/fixed-numeric binding equality for non-generic
  and direct constructed AST; its existing rejection of generic functions before
  body admission is unchanged, and binding comparisons are skipped within generic
  impl contexts that checked IR currently traverses. Equality adds no conversion,
  subtyping, defaulting, layout, or execution semantics. The canonical String control
  does not decide owned `String` versus slice semantics. A mismatch identifies the
  binding plus expected/actual types.
- Array consequence: outside active semantic generic scopes, a numeric array literal
  whose first successfully inferred element is `Ty::Int` or `Ty::Float` infers every
  element left-to-right and requires exact homogeneity after preserving child errors;
  indexing an inferred numeric fixed array requires `int`. A selected explicit fixed
  numeric-array annotation also matches element type and exact count. Mixed numeric
  promotion remains limited to already defined arithmetic operators and is not
  extended to array elements or binding assignment. Nonnumeric and generic-scope
  array inference/index behavior remains unchanged. Empty literals and zero-length
  annotations are not selected: semantics retains its `[Int; 0]` default and existing
  annotation-ignore outcomes, while checked IR retains rejection for no logical
  element type before binding comparison. Typed zero-length array repeats remain
  admitted at their existing boundaries with their annotations ignored.
- Boundary: semantics owns the public diagnostic and must stop trusted source before
  IR. Checked IR repeats the contract for public constructed-AST callers and must
  derive a binary result from the admitted operands/operator rather than trust the
  caller's optional `Expression::Binary.ty`. When that metadata is present it is an
  assertion that must equal the derived type; disagreement is rejected before
  binding comparison or lowering, while absence retains local derivation. The new
  binding rule must not alter recursive/global `admission_type`. Lowercase `string`,
  custom names, explicit generic/reference/tuple forms, flat nonnumeric arrays,
  nested arrays, and arrays wrapping excluded forms retain pre-task annotation-ignore
  behavior. Inside semantic generic scopes, `T`, `bool`, `String`, `string`, fixed
  numeric annotations, numeric-array inference, and numeric-array indexing also
  retain pre-task behavior. These are quarantined gaps, not supported contracts.
  Checked IR's pre-body rejection of generic functions remains a rejection control,
  while generic-impl binding annotations retain their existing ignored behavior.
  Default trait bodies remain syntax-only in checked IR and are not represented as
  type-checked by this decision. The binary metadata assertion applies to every
  otherwise admitted checked expression, including a generic-impl method, but performs
  no substitution and changes no annotation mapping. Uninitialized declarations,
  reassignment, aggregate bounds/layout, mutation, slices, ownership, generics, and
  backend execution remain separate. Every generic-scope push in active semantics
  must be balanced on success and error so public analyzer reuse cannot turn stale
  state into a false generic-scope exemption.
- Evidence required: a tests-only red matrix must demonstrate the four selected
  reproduced artifact false successes, semantic phase-order defects for mixed numeric
  arrays/non-int indexes, later-child diagnostic precedence, a spoofed direct-AST
  binary-type assertion, direct checked-IR enforcement, exact controls, root/direct-
  module library/CLI routes, no unwinds, nonzero statuses, and absent failed artifacts.
  Direct checked-IR negatives must include both int-from-float and float-from-int
  scalar mismatches. A non-generic impl must reject selected mismatches in semantics
  and checked IR, paired with the generic-impl preservation controls below.
  Green direct semantic and checked-IR preservation controls must pin lowercase
  `string`, custom/generic/reference/tuple annotations, flat bool/String/string arrays,
  nested arrays, and arrays wrapping excluded forms. Semantic-only generic-function
  controls must pin deliberately mismatched `T`/`bool`/`String`/`string` annotations,
  fixed numeric-array annotations, mixed numeric arrays, and float numeric-array
  indexes. They must also preserve numeric-scalar mismatch rejection in fully analyzed
  generic function/impl bodies, while generic trait defaults retain syntax-preflight
  acceptance. Separate semantic-only controls pin nonnumeric array heterogeneity and
  non-integer indexing. Checked IR must retain generic-function rejection before body
  admission, generic-impl annotation-ignore behavior, mixed-numeric-array admission,
  existing non-integer-index rejection, and syntax-only trait bodies. The universal
  binary metadata rule requires spoof-rejection tests on an unannotated or excluded
  binding and inside a generic impl, plus matching-metadata and absent-metadata passing
  controls. Phase-specific empty-array controls must preserve semantic acceptance of
  unannotated, `[int; 0]`, and `[float; 0]` empty bindings and checked-IR rejection of
  each before annotation equality. Direct semantic and checked-IR green controls must
  also preserve deliberately mismatched typed zero-length repeats in both directions:
  `[float; 0] = [1; 0]` and `[int; 0] = [1.5; 0]`. These directly bind the `count > 0`
  eligibility guard. A same-analyzer reuse test must trigger a failing generic-impl
  numeric mismatch, then prove a non-generic selected mismatch still rejects after
  scope cleanup. These are quarantine controls, not support. Focused and complete
  gates plus three exact reviews are mandatory before each publication stage.
- Tests-only red evidence: the 16-test target is exactly 8 preservation passes and
  8 intended contract failures. The exact repository gate passes formatting and
  correctness Clippy before stopping on that target; an all-target no-fail-fast run
  identifies it as the only failing target. Root and direct-module invalid bindings
  both return success from check/build and publish requested LLVM artifacts. This is
  failure evidence only and establishes no production behavior.
- Red publication and implementation evidence: three independent reviewers approved
  exact red diff `e158ad61282617a63dade4976a7c23fe53aa0af8` and tree
  `db2ac2959f9815fab5d4b649e563b59c83459dfe` with no P0-P3 findings. Public commit
  `b203ea429b5a039705be5a5b11998e6dc59f5a24` reproduces only the frozen 8/8 target
  failures in both compiler-test jobs and Rust nightly; stable is matrix-cancelled,
  while all CodeQL checks pass. The local two-phase candidate makes all 16 focused
  groups green. The focused test delta adds implementation-review regression controls
  for numeric-array child ordering, single-pass deep nesting, nested index traversal,
  and stub-only method/closure/format/custom-enum boundaries. Several would reject the public red
  implementation, but remain within its already-failing semantic group and do not
  alter the published 8/8 group result. Its public-library assertion also corrects
  `Semantic Error:` to the frozen `Semantic Analysis Error:` phase prefix; that branch
  was unreachable in the red false-accept state. The exact complete gate passes 139
  library tests, 148 binary tests, every active integration target, formatting,
  correctness Clippy, and doc tests with the existing 38 Phase 5 ignores unchanged.
  Three independent reviewers approved exact implementation diff
  `3a909f5813def06d4f7cfb27f8650908410ac724` and tree
  `3effac84a84d56f43abcf99c65161c3da7753d6e` with no P0-P3 findings. Public commit
  `3f0578d69926e15a81c4d8fa6105c99c982cbe02` passes both compiler-test jobs,
  stable/nightly Rust, all three CodeQL analyses, and aggregate CodeQL. Three fresh
  reviewers approved exact closure-record diff
  `a8e4059e71991c9d7a274234f91dd225bea61c01` and tree
  `19fea4153397958656b57adac6b70556d4a997c9` with no P0-P3 findings. Public closure
  commit `5d7aae0f5626813249b6de983a229dbbb1e4fef8` also passes both compiler-test jobs,
  stable/nightly Rust, all three CodeQL analyses, and aggregate CodeQL. `CORE-015` is
  accepted at that closure head.
- Alternatives rejected: checking annotations only in CLI; keeping nonnumeric
  annotations documentary; relying on LLVM verification; checking only the first
  array element; promoting mixed array values to float; dropping array length from
  equality; trusting caller-supplied binary metadata; adding lowercase `string` as
  an alias; enforcing new forms inside a generic semantic scope; changing parser/type
  representation; enabling unsupported composites; or broadening into ownership,
  assignment, default trait type checking, or codegen.
- Revisit when: a separately frozen conversion/subtyping policy, definite assignment,
  generic substitution, aggregate execution, or full trait-body type checker is
  ready for an independently tested vertical slice. None is inferred by `CORE-015`.

## DEC-021 — Separate compiler package version from the v1 language design target

- Date: 2026-08-03
- Status: accepted and closed for `CORE-016` at public-green record-only closure
  `ea036f2`.
- Decision: `src/compiler/Cargo.toml` remains the single source for the compiler
  package/implementation version. The CLI obtains that value at compile time with
  `env!("CARGO_PKG_VERSION")`. Standalone `-v` and `--version` print exactly
  `Aero compiler version <package-version>` and exit zero. The no-command banner
  prints `Aero Programming Language Compiler v<package-version>` before the existing
  help and exits two. The bare `version` word remains an unknown command with status
  two; this decision does not add a command or change the package version.
- Design-version boundary: `v1.0.0` remains a historical/future Aero language design
  target in the consolidated language document. It is not the current compiler
  package version, a conformance result, a stable compatibility promise, or release
  evidence. Current-facing language, type-system, ownership, tutorial, task, and demo
  documents must visibly distinguish design/reference or historical helper material
  from supported implementation behavior.
- Conformance boundary: the existing three example cases, four deterministic
  repetition checks, their algorithms, counts, exit behavior, and JSON report schema
  remain unchanged. Compatibility field and Rust symbol names containing
  `mechanized` remain unchanged in this slice. Console/help/current docs describe the
  four checks as deterministic regression checks, not mechanized semantics or formal
  proof. `CONFORMANCE_PLAN.md` remains the evidence authority for that limitation.
- Current capability boundary: README and build guidance may claim only behavior
  supported by the accepted capability audit. Generic syntax, trait bounds,
  ownership/move/borrow parsing or shallow tracking does not establish an enforced
  generic type system, borrow checker, lifetime safety, or memory-safety guarantee.
  The repository-facing `CLAUDE.md` status and current getting-started/tutorial
  guidance are part of this boundary, not exempt internal or aspirational claims.
- Evidence required: a tests-only red target must bind both version flags, the
  no-command banner/status, the unchanged unknown `version` command, exact package
  metadata, static use of `env!("CARGO_PKG_VERSION")` rather than a hard-coded
  current implementation version, conformance console/help wording and counts, the
  unchanged JSON compatibility fields, current README/build/CLAUDE/Getting Started
  claims, visible design-target notices, and visible archive notices on claim-heavy
  historical task/demo records. The
  focused target and complete repository gate require three exact independent reviews
  at preregistration, red, implementation, and closure publication stages.
- Tests-only red evidence: the new seven-test target has two passing preservation
  groups for the unchanged 3/4 conformance counts/compatibility schema and explicit
  experimental status, plus exactly five intended failures for CLI version sourcing,
  conformance presentation, current repository claims, design-target notices, and
  historical notices. Formatting and correctness Clippy pass. The complete gate
  reaches only that target after 139 library tests, 148 binary tests, and every prior
  integration target pass; `--no-fail-fast` additionally proves doc tests pass and
  only this new target fails. The established 38 Phase 5 ignores remain unchanged.
- Public red and implementation evidence: three reviewers approved exact public-red
  diff `b734773e6f1f4bb9c9561dc089e72b103e3b4e25` and tree
  `488687b20c882c78c8e801d46cdb0bf817d7f421`; commit `4b94dbd` reproduced the
  intended 2/5 matrix in both compiler-test jobs and nightly Rust while CodeQL stayed
  green. The accepted implementation derives CLI presentation from
  `CARGO_PKG_VERSION`, reclassifies current/design/history prose without changing
  semantics or report compatibility, passes its focused 7/7 claim target and 7/7 CLI
  status target, and passes exact `./tools/test.sh` including doc tests.
- Implementation acceptance evidence: three reviewers approved exact diff
  `e0c2bbb61f33ea53e1c07d472a21a631170c22e7` and tree
  `8d5ba37b0a58c715cf72721ade23471c5fa4fa7c` with no P0-P3 findings. Public
  `cc984d0` passes both compiler-test jobs, stable/nightly Rust, all three CodeQL
  analyses, and aggregate CodeQL without changing Cargo metadata, language semantics,
  conformance compatibility, backend, benchmark, registry, or release state.
- Closure evidence: three reviewers approved exact record-only diff
  `7b24a58e7475700423dc66da368a22b97f9c31e8` and tree
  `4c7f526617ecb8e3a0c28622f8eca44dac627981` with no P0-P3 findings. Public closure
  `ea036f2` passes both compiler-test jobs, stable/nightly Rust, all three CodeQL
  analyses, and aggregate CodeQL. The decision is accepted without choosing or
  publishing a new package or language version.
- Alternatives rejected: changing Cargo to `1.0.0`; choosing a new language version;
  adding a `version` subcommand; renaming JSON fields; calling deterministic reruns
  mechanized semantics; deleting design documents; presenting parsed or dormant
  ownership/generic helpers as safety enforcement; publishing a release.
- Revisit when: compatibility policy and release evidence justify a language or
  package version transition, or a separately reviewed formal-semantics system and
  conformance corpus justify stronger proof claims.

## DEC-022 — Separate strict syntax retention from quarantined Phase 5 semantics

- Date: 2026-08-03
- Status: accepted and closed for `CORE-017` at public-green record-only closure
  `3dd3bb4`.
- Decision: the existing Phase 5 target remains one 38-test inventory, but only 22
  existing syntax tests may become active: four exact strict-token tests and 18 exact
  strict parser-retention tests. Exactly 16 remain ignored: all 14 semantic tests and
  two generic-impl parser tests whose target arguments/bounds are not retained.
- Evidence boundary: active tests use `try_tokenize_with_locations` and
  `parse_with_locations`, assert exact retained output, and are named as syntax shape.
  Passing them is parsed-only evidence, not ownership, borrow-checker, generic/trait
  enforcement, layout, execution, conformance, compatibility, or stability evidence.
- Quarantine boundary: do not execute the 16 ignored tests for acceptance. Their
  broad results are confounded by recovery, absent checks, unrelated unsupported
  constructs, unfrozen ownership semantics, or missing generic-impl AST retention.
- Alternatives rejected: bulk activation of 36 passing tests; counting all 24 syntax
  tests despite discarded generic-impl target data; activating shallow semantic tests
  and thereby freezing an incomplete ownership model; changing production to make a
  selected test pass; deleting or rewriting failures as success.
- Revisit when: generic-impl target argument/bound retention has a separate AST/parser
  contract, or ownership/trait semantics have a frozen model and diagnostic-specific
  trusted-path tests.
- Implementation evidence: exactly 22 selected tests use strict fallible helpers and exact
  token/retained-AST assertions; exactly 16 carry explicit quarantine reasons.
  Focused execution is 22 passed / 0 failed / 16 ignored, the full list is 38, and the
  ignored-only list is the exact frozen 16 without execution. Three current evidence
  documents preserve the parsed-only/no-semantic-uplift boundary. Exact
  `./tools/test.sh` passes.
- Implementation acceptance evidence: after one review-rejected underasserted method
  body was corrected, three reviewers approved exact diff
  `a417c7e3c076e7ff6951ce9c181ea99d6bdfa3b6` and tree
  `83bf4f0ba8f973e7ec39167e53114cf5714fd03b` with no P0-P3 findings. Public
  `8be8c21` passes both compiler-test jobs, stable/nightly Rust, all three CodeQL
  analyses, and aggregate CodeQL without changing production, semantic, IR/backend,
  benchmark, registry, release, package, or `master` state.
- Closure evidence: three reviewers approved exact record-only diff
  `3239da0b313f819bad7beef69cea8b6bd5e658a8` and tree
  `166ec7a5e4156da1cefeb9f921a31714461c6839` with no P0-P3 findings. Public closure
  `3dd3bb4` passes both compiler-test jobs, stable/nightly Rust, all three CodeQL
  analyses, and aggregate CodeQL. The decision is accepted without claiming semantic,
  generic/trait, execution, conformance, compatibility, or stability uplift.

## DEC-023 — `run` success requires execution; backend labels require stage evidence

- Date: 2026-08-03
- Status: accepted at public closure `2e0e17f` after public tests-only red checkpoint
  `427fb4c` and exact three-review-approved implementation `8bde0ff`. Focused CLI
  10/10, claims 7/7, the complete repository gate, and all eight implementation and
  closure checks pass. Exact closure diff `3d0a17f7` and tree `83c9676f` received
  three independent approvals with no P0-P3 findings. This final-state sync changes
  records only; R-007 hardware execution evidence remains open.
- Decision: a successful `aero run` means a program process actually executed. CPU
  keeps its existing object/link/process path and delegated exit status. ROCm may
  retain required LLVM verification and temporary AMDGPU object probing, but even a
  successful regular-file emission postcondition returns operational status `1`
  because Aero has no HIP link or launch. File existence is not object validity or
  usability. An `llc` zero exit without the requested regular file is also an
  operational failure. CUDA remains operationally unavailable and recommends CPU,
  not ROCm, for execution.
- Target decision: `gpu` is not a capability target. `build` and `run` reject the
  alias through both `--target gpu` and `--backend gpu` as ambiguous invocation status
  `2` and require explicit `cpu`, `rocm`, or `cuda`. The internal auto-detection helper
  remains experimental and unchanged; this decision does not certify explicit
  accelerator targets.
- Claim decision: graph output is externally verified textual extraction of internal
  scalar helpers. Quantization output is externally verified scalar-double helper
  transformation using default or sample-derived scale. Backend labels and retained
  `executable*` compatibility fields are not device execution. Current CLI/docs must
  deny device ABI/link/launch, real FP8 representation, executed per-channel
  behavior, numerical correctness, and Aero GGUF performance evidence.
- Compatibility boundary: preserve command names, explicit CPU/ROCm/CUDA flags,
  algorithms and generated instruction/helper bodies, existing report schema/field
  names/counts, every non-`notes` report value, external verifier policy, CPU status
  behavior, and immutable evidence. Wording-only changes to
  `QuantizationReport.notes` are the sole report-value exception. Additive non-
  semantic LLVM comments and CLI presentation are allowed only for the frozen stage
  telemetry. Disabling the non-runnable Aero ROCm example is an evidence correction,
  not benchmark execution or deletion.
- Alternatives rejected: treating object generation as execution success; silently
  choosing CPU for `gpu`; implementing HIP/CUDA launch in a claim slice; renaming
  report fields; fixing quantization mathematics without frozen semantics; deleting
  experimental transforms; publishing the external llama.cpp result as Aero proof.
- Revisit when: a separate decision specifies device discovery or a persistent object
  command, or hardware-gated Aero link/launch/transfer/synchronization/result evidence
  satisfies `BACKEND_STATUS.md`. None is inferred from `CORE-018`.

## DEC-024 — `aero test` reports source analysis, not test execution

- Date: 2026-08-03
- Status: accepted and closed for `CORE-019` at exact three-review-approved
  public-green record-only closure `63b6629`; this final sync changes records only.
- Decision: retain the `test` command name and its exact direct, nonrecursive scan of
  `examples`, `tests`, and `.` for names ending `_test.aero` or `_tests.aero`. For
  each discovered source, retain read, strict parse, direct-module collection, and
  semantic analysis only. The command must state that scope and that no tests were
  executed. It may not use `run`, `Running`, `passed`, or `test result` to describe a
  successfully analyzed source or summary.
- Exact presentation contract: start with `Analyzing Aero test sources (parse, direct
  modules, semantics only; no execution)...`; label each source `Analyzing <path>`;
  report success as `<name> analysis completed (not executed)`; report failure as
  `<name> analysis failed: <diagnostic>`; summarize discovered sources as `analysis
  result: <completed> completed, <failed> failed, <total> total; no tests were
  executed`; and warn on an empty scan with `no Aero test source files found
  (*_test.aero, *_tests.aero); no tests were executed`. ANSI styling may wrap the
  existing labels/checkmarks but may not change those visible words.
- Help/documentation contract: CLI help says `Discover and semantically analyze
  *_test.aero files (no execution)`. `BUILD.md` says the command discovers and
  semantically analyzes the two filename suffixes and does not execute tests or
  generate IR.
- Evidence: public compiler runs `30828281313` and `30828277960` fail only the exact
  two frozen CLI contracts at 9/2; nightly in Rust run `30828281681` fails identically
  while stable is cancelled during tests by permitted fail-fast; all four CodeQL
  checks pass. Exact three-review-approved implementation `2fe580d` passes focused
  11/11, exact `./tools/test.sh`, and all eight public checks in compiler runs
  `30829084150`/`30829086467`, Rust `30829088650`, CodeQL `30829082758`, and
  aggregate `91738325685`.
- Closure evidence: corrected record-only closure `63b6629`, tree `2e886850`, passes
  exact `./tools/test.sh` and all eight public checks in compiler runs
  `30829963152`/`30829970545`, Rust `30829968789`, CodeQL `30829962982`, and
  aggregate `91741344282`.
- Final-sync evidence: exact three-review-approved record-only sync `25dec51`, tree
  `46828e7d`, passes all eight public checks in compiler runs
  `30830484863`/`30830489796`, Rust `30830490379`, CodeQL `30830483828`, and
  aggregate `91743120769`. No new semantic decision is made by `AUDIT-026`.
- Compatibility boundary: preserve command spelling, argument arity, scan directories
  and order, filename suffixes, nonrecursive traversal, direct-module behavior,
  semantic diagnostics, discovered/completed/failure counts, status `0` for no files
  or all successful analyses, status `1` when any discovered source fails read/parse/
  module/semantic analysis, and status `2` for malformed invocation.
- Excluded: executing Aero tests, selecting a test ABI/assertion convention, checked
  IR, LLVM, codegen, native tools, runtime result comparison, artifacts, recursive
  discovery, sorting/deduplication, language/type/ownership/aggregate semantics,
  backend behavior, optimizer claims, `CompilerOptions`, benchmarks, workflows,
  dependencies, package/release/registry state, and `master`.
- Revisit when a separate decision specifies executable test entrypoints, isolation,
  fixtures, assertions, result protocol, and process/runtime behavior.

## DEC-025 — Unsupported nondefault `CompilerOptions` fail closed

- Date: 2026-08-03
- Status: accepted and closed for the selected `CORE-020` ignored-option boundary at
  exact three-review-approved, public-green implementation `70cb0ad`; broader option
  semantics and R-006 convergence remain open.
- Decision: preserve public `CompilerOptions`, its `optimize`, `debug_info`, and
  `target` fields, `Debug`/`Clone`/`Default` derives, default values, and
  `compile_program(&str, CompilerOptions) -> Result<String, String>`. The only
  currently supported value is exactly `CompilerOptions::default()`:
  `optimize == false`, `debug_info == false`, and an empty `target`.
- Fail-closed contract: if `optimize` or `debug_info` is true, or `target` is nonempty,
  `compile_program` returns exactly `Unsupported CompilerOptions: only
  CompilerOptions::default() is supported; optimize, debug_info, and target behavior
  is not implemented`. Validation precedes lexing, so this diagnostic wins even when
  source text is malformed. It is an error, not a warning or silent normalization.
- Evidence and rationale: at basis head `25dec51`, `AUDIT-026` found 62 in-repository
  calls, all default, and no CLI consumer. Static tracing showed the accepted value
  was never read, so all values entered the same checked parse/semantic/IR/codegen
  path. Temporary probe claims are excluded because probe creation exceeded the audit
  stop boundary. Silent
  success falsely implies field behavior; explicit unsupported rejection is the
  smallest truth-preserving correction and adds no language or option semantics.
- Compatibility decision: external nondefault consumers are unknown and may change
  from `Ok` or a source error to this earlier `Err`. The lead accepts that behavioral
  break for an experimental `0.3.0` facade because preserving false success would be
  less safe. Source construction, field access, signatures, derives, and every default
  caller remain compatible.
- Excluded: defining optimization levels/passes, debug formats/metadata, target names
  or triples, whitespace normalization, CLI `BuildTarget`/`BuildConfig` mapping,
  environment-based defaults, parser/semantic/IR/codegen/backend changes, artifacts,
  workflows/dependencies, benchmarks, package/release/registry state, and `master`.
- Revisit only when a separate decision specifies one option's accepted values,
  observable behavior, diagnostics, pipeline phase, platform/toolchain requirements,
  compatibility plan, and positive/negative evidence end to end.
- Preregistration evidence: exact three-review-approved `fae1374`, tree `8c807c17`,
  passes all eight public checks in compiler runs `30833300163`/`30833300408`, Rust
  `30833301841`, CodeQL `30833296979`, and aggregate `91752384364`.
- Tests-first evidence: exact three-review-approved `037f44d`, tree `edd8d33e`, diff
  `be3ab875`, produces exact 1/1 failures in compiler runs `30833844930`/
  `30833845633` and nightly in Rust run `30833845526`; stable is cancelled during
  tests by fail-fast. CodeQL `30833844647` and aggregate `91754222422` are all green.
- Local implementation evidence: one pre-lexing guard is focused 2/2, the four frozen
  preservation targets are 40/40, and exact `./tools/test.sh` passes. These local
  results do not close the decision before exact review and all-eight public green.
- Implementation acceptance: exact staged tree `7c8b2ce1e93c82ca5f42100723431688e7505a22`
  and diff `33e5883e84d82c6a2fa105b7fdfad7d7cebc6ad8` received three independent
  approvals with no P0-P3 findings and were published as
  `70cb0ad1afe3e3649e14a3faca444d8cd16589cb`. Compiler runs
  `30834445685`/`30834446600`, Rust `30834446605`, CodeQL `30834443841`, and
  aggregate `91756251121` all pass.
- Closure acceptance: exact record-only tree
  `df4a04a5891139f83ae355aff74bd6726de1057a` and diff
  `85ef52a4090096042db3339e53fdfd2835302531` received three independent approvals
  with no P0-P3 findings and were published as
  `5a8cd06d740f2c3c87843983371cffc9251f8cfe`. Compiler runs
  `30835593703`/`30835597576`, Rust `30835597620`, CodeQL `30835594365`, and
  aggregate `91759990615` all pass. This evidence closes only DEC-025's selected
  unsupported-options boundary; it does not decide any option meaning or broader
  R-006 architecture.
- Next-decision boundary: `AUDIT-027` is read-only re-ranking, not a semantic
  decision. No new implementation may be selected until its exact preregistration is
  public-green and its three independent findings are reconciled by the lead.

## DEC-026 — Delegated CPU success wording requires exit zero

- Date: 2026-08-03
- Status: accepted by exact reviewed/public-green implementation `a4327be` under
  `CORE-021`; record-only final-state sync pending.
- Decision: after a CPU child executes, print the exact existing `Program executed
  successfully.` line only when `status.code().unwrap_or(-1) == 0`. Every exit still
  prints exact `Exit code: N`. Nonzero exits receive no replacement success/failure
  wording in this slice.
- Preservation: keep exact arbitrary delegated statuses, including `1`, `2`, and
  `7`, plus the current internal/printed `status.code().unwrap_or(-1)` signal
  fallback and platform propagation; do not reinterpret them as CLI-owned classes. Preserve
  child stdout/stderr presentation, compilation/linking/verifier behavior, artifact
  cleanup before status propagation, cleanup/error precedence, zero-exit wording,
  and fail-closed ROCm/CUDA behavior.
- Compatibility decision: removing a false success line from nonzero CPU runs is an
  intentional output compatibility change. Exact process status and all truthful
  output remain compatible; external text parsers are unknown.
- Excluded: replacement wording, helper-return or internal-`exit` refactoring,
  status remapping, project-init containment/rollback, executable `aero test`,
  compiler pipeline or option convergence, language/type/ownership/IR/codegen/
  backend semantics, workflows/dependencies, packages/releases, and `master`.
- Evidence: immutable audit basis `aa3e7a8`, tree `4caa5c33`, is all-eight public
  green in compiler runs `30836250279`/`30836251909`, Rust `30836255407`, CodeQL
  `30836248101`, and aggregate `91762198170`. All three auditors rank R-013 first;
  after A/B/C reconciliation, two rank this presentation slice first and one ranks it
  second behind entry-aware `init` preflight. The lead selects this exact boundary.
- Tests-first evidence: preregistration `a61ea24` is all-eight public green. Exact
  three-review-approved tests-only `0873f65`, tree `51ec7d0a`, diff `f75a6360`,
  reproduces focused 10/1 locally and in compiler runs `30839264536` /
  `30839272375`; nightly in Rust run `30839272429` fails identically while stable is
  cancelled during tests by fail-fast. CodeQL `30839264268` and aggregate
  `91772180985` pass. The implementation changes only the frozen success-line
  condition and passes focused CLI 11/11, backend-claim 7/7, and the exact full local
  gate. Exact tree `0ad98c82`, diff `2dbbc395`, received three approvals and was
  published as `a4327be`; compiler `30839860335` / `30839862442`, Rust
  `30839862423`, CodeQL `30839859840`, and aggregate `91774125621` all pass.
- Final closure: corrected record-only tree `8a4c2d77`, diff `5abbf3a7`, received
  three approvals and was published as `b99e445`. Compiler `30840427466` /
  `30840426655`, Rust `30840428215`, CodeQL `30840415565`, and aggregate
  `91775938704` all pass. DEC-026 is closed only for its selected presentation
  boundary; residual R-013 command semantics remain open.
- Next-decision boundary: `AUDIT-028` is read-only full-risk re-ranking. It cannot
  inherit AUDIT-027 ordering, make a semantic choice, or authorize implementation.
  Lead reconciliation and a separately frozen task contract are required afterward.

## DEC-027 — Init destination entries are occupied even when dangling

- Date: 2026-08-03
- Status: implemented at `2a42324` and closed at exact public record `aa29a00` under
  `CORE-022`.
- Decision: before `aero init` creates a directory or writes a file, inspect the
  `aero.toml` destination and then `src/main.aero` without following symlinks. Any
  existing directory entry, including a dangling symlink, is occupied. Only an exact
  not-found result means available; every other inspection error fails operationally
  before writes with `failed to inspect project destination PATH: ERROR`.
- Preservation: retain manifest-first diagnostic precedence and the exact existing
  manifest/source refusal wording; create no new directory or file on refusal;
  preserve all preexisting entries, the target root,
  successful generated bytes/result, package naming, CLI status/stream behavior, and
  all compiler/backend paths.
- Compatibility decision: the reproduced dangling-source case changes from a failed
  source write after partial manifest publication to a preflight refusal with no new
  file. This intentional containment does not promise general rollback, transaction
  atomicity, race freedom, or symlink resolution/removal.
- Excluded: post-write cleanup/rollback, temporary-file transactions, TOCTOU claims,
  filesystem permission/ownership policy beyond fail-closed inspection, helper-exit
  or CLI-status refactoring, compiler/language/backend changes, dependencies,
  workflows, benchmarks, packages/releases, and `master`.
- Evidence: public-green audit basis `399e04f`, tree `e61762fd`, passes compiler
  `30841015776` / `30841022060`, Rust `30841023011`, CodeQL `30841017756`, and
  aggregate `91777920315`. R-013 is the only residual all three auditors rank in the
  top two (positions 1/2/2). R-011 is stopped on unfrozen bounds-failure semantics;
  R-002 remains a wider runner-up with phase/scope disagreement.
- Acceptance: preregistration `045339d` passes all eight public checks. Triple-
  reviewed tests-only `7cd8aba` reproduces exact compiler 10/1 in `30843119793` /
  `30843125522` and nightly Rust `30843124314`; stable is fail-fast cancelled during
  tests, while CodeQL `30843121127` and aggregate `91784962909` pass. Triple-reviewed
  one-file implementation `2a42324` passes focused binary units 3/3, CLI 11/11, the
  exact full local gate, compiler `30843592298` / `30843592784`, Rust `30843595560`,
  CodeQL `30843589175`, and aggregate `91786468184`.
- Final closure: triple-reviewed tree `e740df48`, diff `3eb8264b`, was published
  unchanged as `aa29a00` and passes compiler `30844324249` / `30844328660`, Rust
  `30844328850`, CodeQL `30844325051`, and aggregate `91788926688`.
- Next-decision boundary: public-green status synchronization `21153f3` authorizes no
  implementation. `AUDIT-029` must independently rank every remaining OPEN or
  PARTIALLY CONTROLLED residual, exclude accepted sub-slices, and return one distinct
  bounded candidate or a stop. Any implementation still requires a separate frozen
  decision/task contract.

## DEC-028 - Boolean helper functions use exact semantic contracts

- Date: 2026-08-03
- Status: accepted and closed at exact public record `0b88530` under `CORE-023`.
- Decision: for monomorphic non-entry top-level helper functions, source `bool`
  parameters and returns participate in the existing exact function-contract path as
  `Ty::Bool`. Calls use exact arity and equality checks, Boolean results infer
  `Ty::Bool`, and explicit/tail/missing returns use the existing return controls.
- Basis: public-green audit commit `0e5cba1`, tree `6ac88db4`, passes compiler
  `30845609442` / `30845612610`, Rust `30845612328`, CodeQL `30845609103`, and
  aggregate `91793190047`. All three auditors ranked all eleven residuals. Their top
  selections were R-002 Boolean contracts, R-010 grammar-authority containment, and
  R-009 parser UTF-16 columns; R-012 was the common second-place evidence slice.
  The lead selects R-002 because it is the highest-severity active fail-open compiler
  defect and remains one semantic phase with a deterministic direct-analyzer red.
- Compatibility decision: invalid Boolean helper calls/returns that direct semantics
  accepted now fail earlier with established contract diagnostics; valid Boolean
  helper results formerly mis-inferred as `Int` become `Ty::Bool`. No coercion or new
  source semantics is introduced because binding equality and checked-IR Boolean
  function lowering to LLVM `i1` are already active evidence.
- Preservation: numeric/void contracts and diagnostics; entry-point semantics/ABI;
  forward/recursive/direct-module visibility; closure shadowing; exact Boolean
  binding equality; checked IR/codegen `i1`; and every unrelated compiler/tooling/
  backend result.
- Test binding: the single aggregate direct-semantic target must preserve both sides
  of the entry boundary: current analyzer acceptance of
  `fn main() -> bool { return 1; }` is retained only as quarantined entry behavior,
  while `fn main() -> i32 { return 1.0; }` retains its current numeric mismatch
  rejection. Production must use a helper-specific Boolean mapping while preserving
  the existing numeric/void entry mapping.
- Excluded: `main`, strings, custom/contextual/structural names, generics, arrays,
  tuples, references, closures, methods, implicit conversion/defaulting, parser/AST,
  IR/verifier/codegen/backend/ABI changes, and any claim of general R-002 closure.
- Revisit excluded types only through separately frozen semantics, compatibility,
  phase boundaries, deterministic negative evidence, and end-to-end preservation.
- Evidence: corrected preregistration `1c28a7b`, tree `ce4e0aa1`, passes compiler
  `30848164601` / `30848168070`, Rust `30848169186`, CodeQL `30848164733`, and
  aggregate `91801596136`. Triple-reviewed tests-only `c3f6e90`, tree `3fd13263`,
  reproduces exact compiler 13/1 in `30848723940` / `30848725388` and nightly Rust
  `30848725757`; stable is fail-fast cancelled, while CodeQL `30848722802` and
  aggregate `91803430236` pass. Triple-reviewed implementation `67ccdf2`, tree
  `c0b538c9`, passes focused and preservation gates, exact `./tools/test.sh`, compiler
  `30850000615` / `30850005598`, Rust `30850005670`, CodeQL `30850001251`, and
  aggregate `91807553635`.
- Final closure: exact triple-reviewed tree `71ac4da7`, diff `adba01a1`, was
  published unchanged as `0b88530` and passes compiler `30850519757` /
  `30850524194`, stable/nightly Rust `30850524148`, CodeQL `30850520457`, and
  aggregate `91809289681`.
- Residual: DEC-028 accepts only non-entry monomorphic Boolean helper contracts.
  Boolean `main`, other entry/ABI validation, String/custom/contextual/structural/
  generic/composite/reference/closure/method contracts, coercion/defaulting, and
  broader R-002 closure remain undecided or quarantined.

## DEC-029 - LSP parser diagnostics use UTF-16 character coordinates

- Date: 2026-08-03
- Status: accepted at public record closure `226b7fb`. Preregistration `b8fb1d2` is
  all-eight green; tests-only `ab8508e` reproduces the intended scalar/UTF-16
  mismatch; exact triple-reviewed implementation tree `79ccfca1` and diff
  `74bfbcea` pass focused, LSP, full-local, and all-eight public gates. Corrected
  exact closure tree `1337945c`, diff `861b5ec3`, passes compiler `30854853182` /
  `30854856449`, Rust `30854856190`, CodeQL `30854853829`, and aggregate
  `91823492290` after three fresh approvals with no P0-P3 findings.
- Decision: retain internal parser `SourceLocation` as one-based line and Unicode-
  scalar column data, but project parser diagnostic start columns to zero-based
  UTF-16 character offsets at the LSP boundary using the complete source line.
- Basis: exact `AUDIT-030` authorization `d4e3c75`, tree `9a07c10c`, passes compiler
  `30851275589` / `30851278460`, stable/nightly Rust `30851278586`, CodeQL
  `30851276053`, and aggregate `91811764009`. All three independent rankings place
  R-009 in their top three; two rank it first, and all find its semantics frozen.
- Compatibility decision: non-BMP-prefix parser ranges intentionally change from
  scalar offsets to protocol-correct UTF-16 offsets. Preserve ASCII coordinates,
  the existing synthetic one-unit end range, severity, source label, message,
  multi-error order, and every non-LSP diagnostic.
- Excluded: lexer/parser/AST/recovery/source-location changes, token or AST spans,
  semantic/IR/verifier/codegen/ABI/backend behavior, symbol/completion positions,
  grammar authority, and any claim that broader R-009 is controlled.
- Revisit full ranges only through a separately frozen end-to-end span and recovery
  model with positive, negative, retention, and protocol evidence.

## DEC-030 - Initialized outer tuple binding annotations fail closed

- Date: 2026-08-03
- Status: accepted and record-closed at public `b0fe242`.
- Decision: an initialized binding with exact outer AST annotation `Type::Tuple(_)`
  is unsupported and must reject after existing RHS validation but before binding
  insertion in both direct semantic analysis and checked IR admission. The rule
  applies wherever those statement paths are traversed, including generic-impl
  contexts that otherwise bypass selected annotation comparisons.
- Basis: exact public-green `AUDIT-031` authorization `ba258c6`, tree `651762a8`,
  passes compiler `30855407928` / `30855410819`, Rust `30855410731`, CodeQL
  `30855409113`, and aggregate `91825280915`. All three auditors rank the distinct
  active false success above R-010 after targeted reconciliation.
- Diagnostics: direct semantics identifies the variable and unsupported initialized
  tuple annotation; checked admission identifies the checked binding and same
  unsupported annotation. Existing duplicate/child/void/unsupported-expression and
  outer-generic diagnostics retain precedence.
- Excluded: tuple values/projections, uninitialized or nested tuple annotations,
  parameters/returns, generic type/parameter semantics, references/ownership/
  coercions, type conversion, unchecked compatibility APIs, verifier/codegen/ABI/
  layout/backend, grammar rules, and capability promotion.
- Revisit tuple annotation support only with separately frozen tuple type/value,
  representation, layout, ABI, ownership, lowering, and execution contracts.
- Evidence: triple-reviewed preregistration `722d4d1` is all-eight green. Corrected
  triple-reviewed tests-only `39ccd9c` produces exactly 16 passed/1 failed in compiler
  runs `30857467570` / `30857469931` and the nightly job in Rust `30857470046`,
  with stable fail-fast cancelled and CodeQL/aggregate green. Triple-reviewed
  implementation `1ec8beb`, tree `ac2c8fdd`, passes focused 1/1, binding 17/17,
  the exact full local gate, compiler `30857775577` / `30857777431`, stable/nightly
  Rust `30857777314`, CodeQL `30857775231`, and aggregate `91832840108`.
- Residual: this accepts fail-closed rejection only. R-002 remains PARTIALLY
  CONTROLLED, tuple remains PARSED_ONLY, and no tuple type/value support, ABI,
  layout, execution, ownership, generic substitution, or stability is inferred.
- Closure evidence: corrected six-record closure `b0fe242`, tree `2a5d233f`, diff
  `98916b4d`, received three approvals and passes compiler `30858384541` /
  `30858387195`, stable/nightly Rust `30858387193`, CodeQL `30858385234`, and
  aggregate `91834740790`.
- Audit handoff: `AUDIT-032` may independently re-rank all eleven remaining OPEN or
  PARTIALLY CONTROLLED risks from a clean public head only after its separate exact
  six-record authorization is reviewed, published unchanged, and all-eight green.
  It carries no implementation or capability-promotion authority.

## DEC-031 - Known scalar top-level arity fails at checked admission before IR

- Date: 2026-08-03
- Status: accepted and record-closed at public `0a940ea`.
- Decision: checked direct-AST admission must reject an exact-arity mismatch for a
  known eligible top-level helper before raw IR generation. Eligibility requires
  exactly one top-level declaration for the name; a verifier-valid function symbol
  other than reserved `printf`; verifier-valid, pairwise-distinct parameter symbols;
  no generic parameters; a non-entry name; parameters all admitted scalar
  `Int`/`Float`/`Bool`; and an admitted scalar result or omitted `Void`.
- Ordering: validate all supplied arguments left-to-right, including surplus
  arguments; resolve an admitted local callable first; preserve Void-as-value
  rejection; only then compare the eligible top-level signature's exact arity.
- Diagnostic: exact Admission display is `call to \`NAME\` has ACTUAL arguments but
  its signature requires EXPECTED`, matching the existing verifier `CallArity`
  text. The verifier remains an independent defensive boundary.
- Basis: public-green `AUDIT-032` authorization `b6b1c63`, tree `c8803965`, passes
  compiler `30858876643` / `30858879497`, Rust `30858879480`, CodeQL `30858875767`,
  and aggregate `91836318450`. All three complete independent rankings, followed
  by targeted reconciliation, place this distinct one-phase R-005 defect above
  R-010 once the eligibility and precedence boundaries are frozen.
- Excluded: source semantics; parameter type comparisons; conversions/coercions;
  unknown targets; local callable, method, constructor, entry, generic, composite,
  or reference behavior; raw generation; IR/verifier/codegen/ABI/backend changes;
  and any broader R-005 or capability claim.
- Preservation boundary: malformed or reserved function symbols, malformed or
  duplicate parameter symbols, and duplicate top-level declaration identities are
  ineligible. Their current generation/verifier failures retain precedence over the
  new arity guard.
- Revisit argument type admission or broader callable contracts only through a
  separately frozen semantic and compatibility decision with tests-first evidence.
- Evidence: corrected triple-reviewed authorization `7dc3eac` is all-eight public
  green. Corrected triple-reviewed tests-only `1538a3e`, tree `8f3cd8fb`, reproduces
  exactly 6 passed/1 failed in compiler runs `30861809364` / `30861811517` and the
  nightly job in Rust `30861811567`, with stable fail-fast cancelled and CodeQL/
  aggregate green. Triple-reviewed implementation `8c2b2ec`, tree `eabd8939`, passes
  focused 1/1, checked-IR 7/7, the exact full local gate, compiler `30862232159` /
  `30862233829`, stable/nightly Rust `30862233777`, CodeQL `30862232615`, and
  aggregate `91846586968`.
- Residual: this accepts only the checked-admission phase-order guard. R-005 remains
  HIGH/CRITICAL and PARTIALLY CONTROLLED; unchecked APIs, argument typing, other
  signatures/callables, IR/verifier/codegen/backend behavior, and every capability
  claim remain open or unchanged.
- Closure evidence: corrected six-record closure `0a940ea`, tree `6ec4c609`, diff
  `4e1db178`, received three approvals and passes compiler `30862783787` /
  `30862786131`, stable/nightly Rust `30862786150`, CodeQL `30862784231`, and
  aggregate `91848258218`. Superseded snapshot `615c00b9` was rejected before
  publication for stale gate chronology.
- Audit handoff: `AUDIT-033` may independently re-rank all eleven remaining OPEN or
  PARTIALLY CONTROLLED risks from this clean public head only after its separate exact
  six-record authorization is reviewed, published unchanged, and all-eight green. It
  carries no implementation or capability-promotion authority.

## DEC-032 - Grammar and core tutorial are design targets, not compiler evidence

- Date: 2026-08-03
- Status: accepted and record-closed at public `d649c2d`.
- Decision: `docs/language/aero_grammar.md` and `tutorials/02-core-features.md` must
  visibly classify their v1.0.0 material as intended normative design, not evidence of
  the currently implemented compiler subset, conformance, or stability. Both must
  point readers to `CURRENT_CAPABILITY_AUDIT.md` and `SPEC_IMPLEMENTATION_MATRIX.md`.
- Grammar authority: preserve every EBNF production while replacing only the
  introduction's unqualified definitive-compiler-guide sentence with a normative v1
  design-target/current-conformance boundary.
- Basis: public-green `AUDIT-033` authorization `544b1ba`, tree `cdc3a085`, passes
  compiler `30863291761` / `30863294642`, Rust `30863294655`, CodeQL `30863292940`,
  and aggregate `91849762353`. Three complete independent rankings and final targeted
  reconciliation unanimously place this zero-phase R-010 containment above the
  stopped R-005 argument-type candidate.
- Excluded: grammar reconciliation; any production/example/language/version/compiler/
  workflow/dependency/capability/backend/claim-evidence change; and any claim that
  R-010 is closed or current compiler conformance is established.
- Evidence: triple-reviewed authorization `3574704` is all-eight public green.
  Triple-reviewed tests-first `f57cf2e`, tree `8a99d994`, reproduces exactly 7
  passed/1 failed in compiler `30864786831` / `30864789388` and nightly Rust
  `30864789399`, with only the selected authority contract red; stable is fail-fast
  cancelled, while CodeQL `30864787921` and aggregate `91854279316` pass. Corrected
  triple-reviewed implementation `b3e7910`, tree `2728bbc6`, diff `90e1c4b6`,
  passes focused 1/1, version-claim 8/8, the exact full local gate, compiler
  `30865344667` / `30865346597`, stable/nightly Rust `30865346602`, CodeQL
  `30865345043`, and aggregate `91855955012`. Superseded snapshot `01615da` was
  rejected before publication for an extra final-newline mutation.
- Residual: this accepts only visible authority containment. R-010 remains HIGH/HIGH
  and OPEN; grammar compatibility, current conformance, executable examples,
  migration, parser/AST/semantic convergence, stability, and every capability claim
  remain unproved or unchanged.
- Closure evidence: exact six-record closure `d649c2d`, tree `b5ad7ee2`, diff
  `d4281863`, received three approvals and passes compiler `30865772404` /
  `30865775196`, stable/nightly Rust `30865775214`, CodeQL `30865772793`, and
  aggregate `91857289172`.
- Audit handoff: `AUDIT-034` may independently re-rank all eleven remaining OPEN or
  PARTIALLY CONTROLLED risks from this clean public head only after its separate
  exact six-record authorization is reviewed, published unchanged, and all-eight
  green. It carries no implementation or capability-promotion authority.
- Revisit actual grammar compatibility only through a separately frozen authority,
  migration, parser/AST/semantic, executable-example, and compatibility contract.

## DEC-033 - Unsupported uninitialized outer tuple annotations must fail closed

- Date: 2026-08-03
- Status: accepted implementation at public all-eight-green `e051452`; final record
  closure pending.
- Decision: an exact valueless binding with outer annotation `Type::Tuple(_)` is not
  supported syntax-to-IR behavior. Semantics must reject it after existing same-scope
  duplicate detection and before the current default `Ty::Int` or binding insertion.
  Checked admission must independently reject the exact AST before generation. A
  test that records current acceptance is quarantine evidence, not compatibility or
  tuple-support authority.
- Basis: public-green `AUDIT-034` authorization `45783af`, tree `f1baa457`, passes
  compiler `30866227485` / `30866229553`, Rust `30866229554`, CodeQL
  `30866227939`, and aggregate `91858665436`. Three complete independent rankings
  and targeted reconciliation unanimously select this R-002 public false success:
  semantics silently maps the unsupported tuple annotation to `Int`, checked
  admission skips it, and raw generation can fabricate integer zero. This violates
  both hard unsupported-type-fallback and invalid-before-IR rules.
- Exact boundary: only
  `Statement::Let { type_annotation: Some(Type::Tuple(_)), value: None, .. }`.
  Semantics returns
  ``Error: Variable `NAME` uses an unsupported tuple type annotation for an uninitialized binding.``
  Checked admission returns
  ``checked IR binding `NAME` uses an unsupported tuple type annotation for an uninitialized binding``
  and public compilation surfaces the exact semantic diagnostic through its existing
  wrapper. There is no RHS child. Existing duplicate-name semantics remains first;
  checked admission gains no duplicate-name policy.
- Runner-up: R-005 zero-argument direct calls through parameterized local closure
  aliases should eventually stop at admission, but the mandatory verifier already
  rejects them before LLVM. All calls with supplied arguments remain stopped because
  a new outer arity rejection could mask incompletely admitted child failures.
- Excluded: initialized tuple bindings and their CORE-025 diagnostics/child order;
  nested tuples under another outer annotation; tuple values/projections/patterns/
  defaults/layout/support; all other valueless annotations; parser/AST; generic type
  redesign; unchecked APIs; raw generation, verifier, codegen, ABI, ownership,
  backends, workflows, dependencies, claims, and capability promotion.
- Consequence at selection time: `CORE-028` could preregister one tests-first file
  and then exactly two compiler phase files. R-002 would remain HIGH/CRITICAL and
  PARTIALLY CONTROLLED even if this slice were accepted. Stop if a compatibility
  decision, tuple semantics, a third phase, another annotation outcome, or valid
  generated output would change.
- Evidence: corrected authorization `4cc682f` is triple-approved and all-eight green.
  Triple-approved tests-first `3fb5f7a`, tree `f12a6c6b`, publicly isolates one
  16/1 aggregate failure with exactly the five frozen false acceptances in compiler
  `30871003009` / `30871004997` and stable/nightly Rust `30871005020`; CodeQL
  `30871003987` and aggregate `91872902124` pass. Triple-approved implementation
  `e051452`, tree `63985b2d`, diff `79830403`, passes focused 1/1, binding 17/17,
  the exact full local gate, compiler `30871337443` / `30871335738`, Rust
  `30871337440`, CodeQL `30871336117`, and aggregate `91873866339`.
- Residual: this accepts fail-closed containment only. Tuple support, nested or other
  unsupported annotation enforcement, unchecked APIs, valid-output certification,
  and all broader R-002/R-005 surfaces remain unchanged. R-002 stays HIGH/CRITICAL
  and PARTIALLY CONTROLLED; no matrix cell or capability changes.
