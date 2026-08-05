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
- Status: accepted and closed at public all-eight-green `032d0d0`.
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
- Closure evidence: exact snapshot `f6305e18`, tree `443aacdc`, diff `93fce8ae`,
  received three approvals and was published unchanged as `032d0d0`. Compiler
  `30872236535` / `30872238993`, stable/nightly Rust `30872239003`, CodeQL
  `30872237025`, and aggregate `91876507154` all pass.
- Audit handoff: `AUDIT-035` may independently re-rank all eleven remaining OPEN or
  PARTIALLY CONTROLLED risks only after its separate six-record authorization passes
  exact local, review, unchanged-publication, and all-eight public gates. It carries
  no implementation or capability-promotion authority and excludes every accepted
  slice including CORE-028.

## DEC-034 - Valueless immediate reference-to-tuple annotations must fail closed

- Date: 2026-08-03
- Status: accepted at public all-eight-green closure `7222b9a`.
- Decision: an uninitialized binding with outer `Type::Reference(inner, _)` and
  immediate `inner: Type::Tuple(_)` is unsupported syntax-to-IR behavior. Semantics
  must reject it after same-scope duplicate detection and before default `Ty::Int` or
  insertion. Checked admission must independently reject the exact AST before raw
  generation. Both reference-mutability flags are included.
- Basis: public-green AUDIT-035 authorization `f1cd972`, tree `b9c6270b`, passes
  compiler `30872922468` / `30872923806`, Rust `30872923874`, CodeQL `30872922858`,
  and aggregate `91878491979`. Independent rankings initially split between this
  exact R-002 form and R-005 local-closure arity admission. Targeted comparison was
  unanimous: this form silently becomes verifier-valid integer IR and can reach
  trusted LLVM/CPU publication, while R-005 is already verifier-contained before
  LLVM.
- Exact diagnostics: semantics returns
  ``Error: Variable `NAME` uses an unsupported tuple type annotation directly beneath a reference for an uninitialized binding.``
  Checked admission returns
  ``checked IR binding `NAME` uses an unsupported tuple type annotation directly beneath a reference for an uninitialized binding``
  and public compilation retains its existing semantic wrapper.
- Compatibility: CORE-028's exact `&(tuple)` acceptance control was explicitly
  quarantine evidence, not support. Rejection defines no tuple/reference value,
  initialization, assignment, representation, mutability, borrowing, ownership,
  lifetime, provenance, layout, ABI, or execution semantics.
- Excluded: outer tuple CORE-028; initialized bindings; scalar references; arrays or
  generics containing tuples; reference-to-reference-to-tuple; recursive type-tree
  rejection; all other annotations; parser/AST; raw IR; verifier; codegen; ABI;
  ownership; backends; workflows; dependencies; claims; matrix/capability promotion.
- Consequence at selection time: CORE-029 could preregister one tests-first file and
  then exactly two compiler phase files. R-002 would remain HIGH/CRITICAL and
  PARTIALLY CONTROLLED even if accepted. Stop on any compatibility decision,
  ownership/reference/tuple semantics, third phase, another annotation outcome, or
  valid-output change.
- Runner-up: R-005 exact zero-argument calls through parameterized local scalar
  closure aliases remain the next bounded candidate; mandatory verification already
  stops them before LLVM, and supplied-argument child precedence remains excluded.
- Evidence: corrected authorization `c0e1a90` is triple-approved and all-eight green.
  Corrected triple-approved tests-first `d12ba66`, tree `056a9d52`, publicly isolates
  exactly one 17/18 aggregate failure with five frozen false acceptances in compiler
  `30874817273` / `30874819174` and nightly Rust `30874819175`; stable is fail-fast
  cancelled, while CodeQL `30874817566` and aggregate `91884136725` pass. Exact
  implementation `29bd2e0`, tree `53282149`, diff `acc1c247`, passes focused 1/1,
  binding 18/18, the exact full local gate, compiler `30875100237` / `30875102914`,
  Rust `30875102909`, CodeQL `30875100762`, and aggregate `91884963697` after three
  exact approvals.
- Residual: this accepts fail-closed containment only. Tuple/reference/ownership
  support, initialized or deeper annotation enforcement, unchecked APIs, valid-
  output certification, and all broader R-002/R-005 surfaces remain unchanged.
  R-002 stays HIGH/CRITICAL and PARTIALLY CONTROLLED; no matrix cell or capability
  changes.
- Closure evidence: exact snapshot `6c7358be`, tree `66084b36`, diff `90bf540c`,
  received three approvals and was published unchanged as `7222b9a`. Compiler
  `30876033717` / `30876035730`, stable/nightly Rust `30876035761`, CodeQL
  `30876034500`, and aggregate `91887644623` all pass.
- Audit handoff: `AUDIT-036` may independently re-rank all eleven remaining OPEN or
  PARTIALLY CONTROLLED risks only after its separate six-record authorization passes
  exact local, review, unchanged-publication, and all-eight public gates. It excludes
  every accepted slice including CORE-029 and carries no implementation or
  capability-promotion authority.

## DEC-035 - Post-CORE-029 selection requires clean-head full-set reconciliation

- Date: 2026-08-04
- Status: accepted and complete at public-green AUDIT-036 head `f4ac505`.
- Decision: no next implementation is selected from CORE-029's runner-up or any prior
  ranking. Three independent read-only auditors must rank the complete remaining
  R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016 set from the
  exact public-green AUDIT-036 authorization head, excluding every accepted slice.
- Evidence boundary: static repository evidence only during ranking. No test, probe,
  benchmark, artifact, hardware action, external query, source, workflow, dependency,
  backend, package/release/registry, immutable claim-evidence, or `master` action is
  authorized. Passing authorization gates unlocks read-only ranking only and never
  implementation or capability authority.
- Selection rule: reconcile one bounded active residual with frozen semantics,
  deterministic tests-first feasibility, and at most two compiler phases, or record
  an explicit stop. Any later implementation requires its own reviewed contract and
  public failing regression evidence first.
- Result: corrected authorization snapshot `a805d1c9`, tree `3cdf89e6`, diff
  `40896f51`, received three approvals and was published unchanged as `f4ac505`.
  Compiler `30876975678` / `30876977928`, Rust `30876977905`, CodeQL
  `30876976155`, and aggregate `91890402326` pass. Three complete independent
  rankings unanimously select the exact R-002 valueless immediate array-of-tuple
  fallback over verifier-contained R-005.

## DEC-036 - Valueless immediate array-of-tuple annotations must fail closed

- Date: 2026-08-04
- Status: accepted and complete at public all-eight-green record closure `cd8add28`.
- Decision: for an uninitialized `let` annotation exactly shaped as
  `Type::Array(inner, _)` with immediate `inner: Type::Tuple(_)`, reject in semantic
  analysis after same-scope duplicate detection and independently in checked IR
  admission before raw generation. Array count does not affect the rejection.
- Rationale: current preservation tests prove the unsupported form is accepted;
  semantics silently substitutes `Ty::Int`, checked admission skips it, and raw
  generation can emit integer zero. This violates the repository's hard unsupported-
  source-type fallback rule and reaches farther than R-005, which mandatory
  verification already contains before LLVM.
- Boundary: CORE-030 may first add only the named regression in
  `binding_type_contract_tests.rs`, and only after this contract is public all-eight
  green. Implementation later may change only `semantic_analyzer.rs` and
  `ir_generator.rs` after reviewed public red evidence. Initialized, scalar-array,
  generic/Vec, reference-wrapped, deeper-nested, raw API, tuple/array support,
  bounds/layout/mutation/ABI/ownership, and backend behavior remain unchanged.
- Claim boundary: rejection is containment only. R-002 stays HIGH/CRITICAL and
  PARTIALLY CONTROLLED; no capability or matrix cell can move.
- Evidence: triple-approved authorization `1f13084` is public all-eight green.
  Triple-approved tests-first `bd28f6a` publicly reproduces exactly the five
  frozen false acceptances and otherwise preserves the suite. Triple-approved
  implementation `97c0f04`, tree `aa3a9e3f`, diff `06a104df`, changes only the
  semantic and checked-admission guards; focused 1/1, binding 19/19, the exact full
  local gate, both compiler jobs, stable/nightly Rust, all three CodeQL analyses,
  and aggregate pass.
- Closure evidence: exact snapshot `9b872297`, tree `8ab06d62`, diff `18ffa30d`,
  received three approvals and was published unchanged as `cd8add28`. Compiler
  `30879329940` / `30879332975`, Rust `30879332995` attempt 2, CodeQL
  `30879330627`, and aggregate `91897195358` pass. The initial transient Linux
  `ETXTBSY` test-fixture attempt passed on focused rerun without a file or ref change.

## DEC-037 - Post-CORE-030 selection requires clean-head full-set reconciliation

- Date: 2026-08-04
- Status: accepted and complete at public-green AUDIT-037 head `987188fc`.
- Decision: no implementation may be selected from AUDIT-036's R-005 runner-up or
  any earlier order. Three independent read-only auditors must rank the complete
  remaining R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016
  set from exact public-green CORE-030 closure `cd8add28`, excluding every accepted
  sub-slice through CORE-030.
- Evidence boundary: static immutable repository evidence only during ranking. No
  edit, test, build, formatter, probe, benchmark, artifact, hardware action,
  external query, workflow, dependency, backend, package/release/registry,
  immutable claim-evidence, history, or `master` action is authorized. Passing the
  authorization gates unlocks read-only ranking only.
- Selection rule: reconcile one distinct residual with frozen semantics,
  deterministic tests-first feasibility, and no more than two compiler phases, or
  record an explicit stop. Any test or implementation requires a separate reviewed
  six-record contract and public failing regression evidence first.
- Claim boundary: AUDIT-037 cannot change a risk status, matrix row/cell, capability
  class, backend claim, compatibility rule, or language semantics.
- Result: exact authorization snapshot `f4de8ef4`, tree `0b685659`, diff
  `d3a9974b`, received three approvals and was published unchanged as `987188fc`.
  Compiler `30880025888` / `30880028697`, Rust `30880028653`, CodeQL
  `30880025866`, and aggregate `91899286217` pass. Three complete rankings place
  R-002 first; after an exact-candidate split, targeted static reconciliation
  unanimously selects valueless `Array(Array(Tuple))` containment over the
  reference-array alternative.

## DEC-038 - Exact two-array-deep valueless tuple annotations must fail closed

- Date: 2026-08-04
- Status: accepted and complete at public all-eight-green record closure `45696091`.
- Decision: for an uninitialized `let` annotation exactly shaped as
  `Type::Array(Type::Array(Type::Tuple(_), _), _)`, reject in semantic analysis
  after same-scope duplicate detection and independently in checked IR admission
  before raw generation. Both array counts are irrelevant.
- Rationale: all three AUDIT-037 reviewers rank R-002 first. Initial candidates split
  between exact two-array and reference-array wrappers; targeted static reconciliation
  unanimously selects the two-array form because it has the same trusted reach and
  phase count without freezing reference mutability or ownership associations. Count
  fields can be wildcarded before IR without selecting array bounds/layout behavior.
- Boundary: CORE-031 may first add only the named regression in
  `binding_type_contract_tests.rs` after this contract is public all-eight green.
  Implementation later may change only `semantic_analyzer.rs` and checked admission
  in `ir_generator.rs` after reviewed public red evidence. Recursive/deeper matching,
  initialized, reference-array, scalar, generic/wrapped, raw API, array/tuple value,
  bounds/layout/mutation/ABI/ownership, valid-output, and backend behavior remain
  unchanged.
- Claim boundary: rejection is containment only. R-002 stays HIGH/CRITICAL and
  PARTIALLY CONTROLLED; no capability or matrix cell can move.
- Evidence: triple-approved authorization `ba57efec`, tree `c01bebe9`, passed all
  eight public checks. Triple-approved tests-first `6899cb1b`, tree `b7007735`,
  canonical diff `43063551`, publicly isolates exactly nine false acceptances in
  compiler `30881792006` / `30881794177` and nightly Rust `30881794186`; stable was
  fail-fast cancelled, while CodeQL `30881792351` and aggregate `91904645414` pass.
  Triple-approved implementation `4bc7a345`, tree `61361621`, canonical diff
  `349e34ee`, changes only the semantic and checked-admission guards; focused 1/1,
  binding 20/20, formatting, the exact full local gate, compiler `30882153355` /
  `30882155935`, stable/nightly Rust `30882155921`, CodeQL `30882154595`, and
  aggregate `91905705897` pass.
- Residual: exact rejection does not define nested-array/tuple values, defaults,
  bounds, layout, mutation, ABI, ownership, lowering, execution, or backend support.
  Candidate B and every named preservation boundary remain unchanged. No matrix
  cell, capability class, or R-002 likelihood/impact/status changes.
- Closure evidence: exact snapshot `45696091`, tree `480c3504`, canonical diff
  `d682b0f6`, received three approvals and was published unchanged. Compiler
  `30882630407` / `30882632698`, stable/nightly Rust `30882632696`, CodeQL
  `30882630822`, and aggregate `91907149874` all pass.

## DEC-039 - Post-CORE-031 selection requires clean-head full-set reconciliation

- Date: 2026-08-04
- Status: accepted and complete at public-green AUDIT-038 head `e4d58e59`.
- Decision: no next implementation is selected from CORE-031's preserved Candidate
  B or any earlier ranking. Three independent read-only auditors must rank the
  complete remaining R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/
  R-016 set from exact public-green CORE-031 closure `45696091`, excluding every
  accepted sub-slice through CORE-031.
- Evidence boundary: static immutable repository evidence only during ranking. No
  edit, test, build, formatter, probe, benchmark, artifact, hardware action,
  external query, workflow, dependency, backend, package/release/registry,
  immutable claim-evidence, history, or `master` action is authorized. Passing the
  authorization gates unlocks read-only ranking only.
- Selection rule: reconcile one distinct residual with frozen semantics,
  deterministic tests-first feasibility, and no more than two compiler phases, or
  record an explicit stop. Any test or implementation requires a separate reviewed
  six-record contract and public failing regression evidence first.
- Claim boundary: AUDIT-038 cannot change a risk status, matrix row/cell, capability
  class, backend claim, compatibility rule, or language semantics.
- Result: corrected authorization snapshot `e4d58e59`, tree `f265d8af`, canonical
  diff `31d09f92`, received three approvals and was published unchanged. Compiler
  `30883186212` / `30883188223`, Rust `30883188248`, CodeQL `30883186829`, and
  aggregate `91908783685` pass. All three complete rankings put R-002 first. After
  a two-candidate preference split, a final compatibility gate unanimously approves
  initialized immediate array-of-tuple containment; the triple-array candidate is
  preserved.

## DEC-040 - Initialized immediate array-of-tuple annotations must fail closed

- Date: 2026-08-04
- Status: complete at public all-eight-green closure `9c82cbfc`.
- Decision: for an initialized `let` annotation exactly shaped as
  `Type::Array(Type::Tuple(_), _)`, validate the initializer first, preserve existing
  initialized outer-tuple handling, then reject in semantics and independently in
  checked IR admission before mismatch handling or raw generation. Array count does
  not affect rejection. Apply the rule in every statement context each phase already
  traverses, including semantic generic scopes and checked generic impl methods; an
  earlier outer generic-construct diagnostic remains first.
- Rationale: the unsupported annotation is currently ignored at both trusted
  boundaries and raw generation discards it, allowing a scalar initializer to reach
  verifier-valid LLVM under an invalid aggregate annotation. CORE-025 already freezes
  child-before-unsupported-annotation ordering. The selected exact guard therefore
  adds no initializer, mismatch, array, or tuple compatibility meaning. All three
  AUDIT-038 reviewers approved that boundary after the initial candidate split.
- Boundary: CORE-032 may first add only the named regression in
  `binding_type_contract_tests.rs` after this contract is public all-eight green.
  Implementation later may change only `semantic_analyzer.rs` and checked admission
  in `ir_generator.rs` after reviewed public red evidence. Valueless, deeper-array,
  reference-wrapped, scalar/numeric, generic/wrapped, raw API, tuple/array value,
  bounds/layout/mutation/ABI/ownership, valid-output, and backend behavior remain
  unchanged.
- Claim boundary: rejection is containment only. R-002 stays HIGH/CRITICAL and
  PARTIALLY CONTROLLED; no capability or matrix cell can move.
- Superseded authorization: snapshot `58e46e34`, tree `b47c7427`, canonical diff
  `f36748c2`, passed its local gate but was rejected before publication by all three
  reviewers because its five-acceptance test contract omitted generic-impl and
  generic-function traversal. The corrected contract explicitly freezes those
  contexts and an eight-acceptance red surface.
- Evidence: corrected authorization `449f3536` is triple-approved and all-eight
  green. Rejected tests snapshot `1afe11d3` was never published because it omitted
  explicit child-valid array-literal coverage. Corrected tests-only `35eac8c4`, tree
  `b54a848b`, canonical diff `e600c2bc`, received three approvals and publicly
  reproduces only the exact eight-acceptance failure in compiler `30886282169` /
  `30886283814` and nightly Rust `30886284165`; CodeQL and aggregate pass.
- Accepted implementation `30d0d730`, tree `653346ce`, canonical diff `01e87768`,
  adds one exact nonrecursive guard in each authorized phase. Focused 1/1, binding
  21/21, formatting, two consecutive full gates, three exact reviews, compiler
  `30886856260` / `30886858878`, Rust `30886858960`, CodeQL `30886856518`, and
  aggregate `91919998289` pass. A preceding full-gate attempt returned exit 1 with
  output truncated before attribution; it remains recorded as unexplained and is not
  called a proven flake.
- Closure evidence: first closure snapshot `7d7fe3d6`, tree `18c904fd`, canonical
  diff `407c3c86`, passed its exact full gate but was rejected unpublished by all
  three reviewers because its state record left that completed gate as future work;
  one reviewer additionally required the known transient status to remain exact as
  exit 1. Second closure snapshot `48f2fd60`, tree `86175cc1`, canonical diff
  `9f0ab102`, resolved those findings and received two approvals but was rejected
  unpublished at P3 by the type reviewer because it omitted the successful closure
  gate's literal `exit 0`. The twice-corrected records preserve both rounds, and the
  fresh exact gate exits 0 (139/139 library, 149/149 binary, 7/7 doc, 21/21 binding).
  Approval, unchanged publication, and all eight public checks are the only next
  actions.
- Result: rejection is containment only. No tuple/array value, compatibility,
  default, bounds, layout, mutation, ABI, ownership, lowering, execution, backend,
  matrix, or capability decision follows. R-002 remains HIGH/CRITICAL and PARTIALLY
  CONTROLLED.
- Closure result: exact `9c82cbfc`, tree `b2a106ee`, canonical diff `fc672744`,
  received three approvals, was published unchanged, and passes compiler
  `30888222316` / `30888225734`, Rust `30888226011`, CodeQL `30888222480`, and
  aggregate `91924197947`.

## DEC-041 - AUDIT-039 resets residual ordering after CORE-032

- Date: 2026-08-04
- Status: complete, read-only, and clean at public-green `fa522b2c`.
- Decision: only after this six-record authorization is triple-approved, published
  unchanged, and all eight public checks pass, three independent read-only reviewers
  may re-rank the complete R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/R-012/
  R-013/R-016 set from exact clean public closure `9c82cbfc`. Every accepted slice
  through CORE-032 is excluded and no earlier runner-up or preservation candidate is
  privileged.
- Method: every reviewer must supply a complete evidence-cited ranking, one exact
  candidate or stop, trusted reach/containment, semantic decisions, phase count,
  deterministic failing specimen, and preservation controls. Reconciliation may
  select at most one unanimously bounded residual or record a stop.
- Authorization evidence: the prepared six-record tree's fresh exact full gate exits
  0 with 139/139 library, 149/149 binary, 7/7 doc, and 21/21 binding tests. Exact
  reviews, unchanged publication, and all eight checks are still pending.
- Boundary: ranking is static and read-only. It authorizes no source/test edit,
  build/probe/external query, semantics, workflow/dependency/backend action,
  capability/matrix/risk movement, artifact/claim publication, history action, or
  `master` change. Any later test or implementation requires a separately reviewed
  six-record contract and public failing regression evidence first.
- Result: exact authorization `fa522b2c`, tree `365a536d`, canonical diff
  `cefb797e`, received three approvals and passes compiler `30888751268` /
  `30888754238`, Rust `30888754262`, CodeQL `30888752230`, and aggregate
  `91925849313`. All rankings put R-002 first. Initial candidates split between the
  valueless three-array form historically labeled Candidate T (type/safety) and initialized two-array Candidate
  A (IR/codegen and backend). Targeted preference favored A two to one; the lead
  provisionally selected A for its smaller predicate/count/test surface and frozen
  initialized-child ordering. All three then approved exact A in a final
  compatibility gate. Candidate T and the reference-array Candidate B remain preserved.

## DEC-042 - Initialized exact two-array-deep tuple annotations must fail closed

- Date: 2026-08-04
- Status: complete at public all-eight-green closure `1ee9c71`.
- Decision: for an initialized binding whose annotation is exactly nonrecursive
  `Type::Array(Type::Array(Type::Tuple(_), _), _)`, validate the initializer and
  preserve existing initialized outer/immediate tuple diagnostics, then reject in
  semantics and independently in checked admission before mismatch/insertion, the
  generic-impl mismatch bypass, or raw generation. Counts are irrelevant.
- Diagnostics: semantic `Error: Variable \`{name}\` uses an unsupported tuple type annotation directly beneath two array layers for an initialized binding.`; checked
  `checked IR binding \`{name}\` uses an unsupported tuple type annotation directly beneath two array layers for an initialized binding`. Public compilation preserves
  its existing semantic-error wrapper.
- Tests-first boundary: reclassify both existing Candidate A acceptance rows into
  one aggregate with exactly 12 false acceptances—8 count/phase, 1 public, 2 generic
  impl, 1 semantic generic function—while preserving the checked generic-function
  outer rejection. No other test/source file may change at that stage.
- Claim boundary: rejection is containment only. Candidate T, reference-array
  Candidate B, other three-plus-depth, wrappers, tuple/array meaning, raw APIs,
  verifier/codegen, layout/bounds, ABI/
  ownership, valid output, backends, risk status, matrix cells, and capability
  classes remain unchanged. At authorization, the prepared six-record gate exited 0
  with 139/139, 149/149, 7/7, and 21/21; exact review, unchanged publication, and
  all eight checks were required before tests-first work.
- Authorization history: first snapshot `d0500865`, tree `d2378320`, canonical diff
  `97a15c9f`, passed its local gate but received one approval and two blocking reviews
  because one ledger sentence mislabeled Candidate T's valueless form as Candidate B.
  It remained unpublished; corrected records preserve both historical names exactly.
- Authorization result: corrected `66207215`, tree `357c2731`, canonical diff
  `96b5f403`, received three approvals and passes compiler `30890569245` /
  `30890571370`, Rust `30890571249`, CodeQL `30890569479`, and aggregate
  `91931557818`.
- Tests-first result: unpublished `7608b42c`, tree `5a2100ee`, canonical diff
  `d68b42ed`, was rejected because it omitted the initialized three-array-deep
  semantic/checked preservation control. Corrected `ac4cb2a5`, tree `852bff0b`,
  canonical diff `4ca50572`, received three approvals and publicly reproduces only
  the exact 12-acceptance 21/22 failure in compiler `30891243037` /
  `30891246443` and nightly Rust `30891247469`; stable was fail-fast cancelled,
  while CodeQL `30891241566` and aggregate `91933672071` pass.
- Implementation result: `76a6e802`, tree `d8391348`, established PowerShell
  full-index canonical diff `a75b59b2`, changes only the two authorized guards.
  Formatting, focused 1/1, binding 22/22, and the exact full local gate exit 0 with
  139/139 library, 149/149 binary, 7/7 claim, and 22/22 binding tests. The initial
  immutable review request supplied erroneous plain-diff `c17b1b6a`; two reviewers
  rejected only that evidence, not the source. Corrected identity review of the
  unchanged commit received three approvals. Compiler `30891890629` /
  `30891898590`, Rust `30891897083`, CodeQL `30891892219`, and aggregate
  `91935804190` pass.
- Result boundary: exact rejection adds no tuple/nested-array value, compatibility,
  default, bounds, layout, mutation, ABI, ownership, lowering, execution, backend,
  matrix, capability, or stability evidence. R-002 remains HIGH/CRITICAL and
  PARTIALLY CONTROLLED. Record closure is required before selecting another slice.
- Closure review history: first snapshot `fe90f583`, tree `90ac8ae6`, canonical diff
  `89fe6824`, changed only the six control records and passed its exact gate with
  139/139 library, 149/149 binary, 7/7 claim, and 22/22 binding tests. It received
  two approvals but was rejected at P1 before any independent push or branch-head
  publication because a late PROJECT_STATE subsection still treated tests-first and
  implementation as future work. First correction `19f688a`, tree `9d9c642f`,
  canonical diff `f885588c`, made that chronology historical, passed the same exact
  gate, received three approvals, and was pushed. Compiler `30893002336` /
  `30893005706`, Rust `30893006634`, CodeQL `30893002479`, and aggregate
  `91939375982` pass. Because `19f688a` was linear atop `fe90f583`, that push also
  made the rejected snapshot reachable as an ancestor, contradicting `19f688a`'s
  stronger never-published wording. The lead withheld closure acceptance and chose
  additive record correction rather than force-push or history rewrite. The final
  correction `1ee9c71`, tree `d0819881`, canonical diff `7303da47`, passed its
  fresh exact gate with 139/139 library, 149/149 binary, 7/7 claim, and 22/22
  binding tests, received three exact approvals, and was published unchanged.
  Compiler `30893527220` / `30893529999`, Rust `30893529992`, CodeQL
  `30893527445`, and aggregate `91941079083` pass. Public ancestry remains intact;
  no force-push or rewrite occurred.

## DEC-043 - AUDIT-040 resets residual ordering after CORE-033

- Date: 2026-08-04
- Status: complete, read-only, and clean at public-green `7b9ed83`.
- Decision: only after this six-record authorization passes its exact local gate, is
  triple-approved, published unchanged, and all eight public checks pass may three
  independent read-only reviewers re-rank the complete R-002/R-004/R-005/R-006/
  R-007/R-009/R-010/R-011/R-012/R-013/R-016 set from exact clean public closure
  `1ee9c71`. Every accepted slice through CORE-033 is excluded and no earlier
  runner-up, historical label, or preservation candidate is privileged.
- Method: every reviewer must supply a complete evidence-cited ranking, one exact
  candidate or stop, trusted reachability/containment, unresolved semantic choices,
  phase count, deterministic failing specimen, and preservation controls.
  Reconciliation may select at most one unanimously bounded residual or record a
  stop.
- Authorization evidence: first snapshot `c83ec3a`, tree `bb25e528`, canonical diff
  `c02f71e5`, changed exactly the six records and passed its exact gate with 139/139
  library, 149/149 binary, 7/7 claim, and 22/22 binding tests. Type/safety and
  backend/claim approved, but IR/codegen rejected at P1 because a late PROJECT_STATE
  subsection still treated accepted CORE-033 closure as future work. It was rejected
  before publication. The corrected tree's fresh exact gate exits 0 with 139/139
  library, 149/149 binary, 7/7 claim, and 22/22 binding tests. Corrected `7b9ed83`,
  tree `8dbe975e`, canonical diff `c4ba110a`, then received three exact approvals,
  was published unchanged, and passes compiler `30894708169` / `30894713332`, Rust
  `30894713411`, CodeQL `30894708736`, and aggregate `91944883143`.
- Boundary: ranking is static and read-only. It authorizes no source/test edit,
  build/probe/external query, semantics, workflow/dependency/backend action,
  capability/matrix/risk movement, artifact/claim publication, history action, or
  `master` change. Any later test or implementation requires a separately reviewed
  six-record contract and public failing regression evidence first.
- Result: the independent top candidates were valueless exact three-array tuple
  containment (type/safety), initialized exact immediate reference-to-tuple
  containment (IR/codegen), and immediate literal fixed-array bounds containment
  (backend/claim). Targeted comparison preferred reference containment two to one;
  compile-time versus runtime bounds policy remains unresolved, and the three-array
  shape has greater topology/count burden. The lead provisionally selected exact
  initialized immediate reference-to-tuple rejection. All three final compatibility
  reviews approved that exact predicate, both mutability flags, diagnostics, ordering,
  generic-context behavior, two-phase limit, and preservation boundary. The audit
  changed nothing.

## DEC-044 - Initialized immediate reference-to-tuple annotations must fail closed

- Date: 2026-08-04
- Status: closed at exact triple-approved public all-eight-green closure `d3811b00`.
- Decision: for an initialized binding whose annotation is exactly nonrecursive
  `Type::Reference(Type::Tuple(_), _)`, validate the initializer, preserve checked
  Void and all existing initialized tuple-shape diagnostics, then reject in semantics
  and independently in checked admission before mismatch, insertion, the generic-
  impl bypass, or raw generation. Both reference mutability flags are included.
- Diagnostics: semantic `Error: Variable \`{name}\` uses an unsupported tuple type annotation directly beneath a reference for an initialized binding.`; checked
  `checked IR binding \`{name}\` uses an unsupported tuple type annotation directly beneath a reference for an initialized binding`. Public compilation preserves its
  existing semantic-error prefix.
- Context: use only traversal already present in the two named phases. Generic impl
  and semantic generic-function bodies are covered; checked generic-function outer
  rejection and syntax-only generic trait defaults remain unchanged. No checked
  duplicate-binding rule is added.
- Tests-first boundary: only after authorization acceptance may the binding contract
  test file reclassify the two existing initialized immutable/mutable rows into one
  aggregate with exactly 30 frozen false acceptances across direct, public, top-level,
  generic, block, control-flow, loop, and non-generic-impl routes. All precedence and
  preservation rows remain green; implementation needs separately reviewed public-
  red evidence.
- Claim boundary: exact rejection is containment only. It defines no reference or
  tuple value, mutability, ownership, lifetime, layout, ABI, coercion, default,
  lowering, execution, bounds, backend, matrix, capability, or stability meaning.
  R-002 remains HIGH/CRITICAL and PARTIALLY CONTROLLED.
- Authorization evidence: first snapshot `7d4d7ca`, tree `b633abbb`, canonical diff
  `a901f4dc`, passed its exact full gate with 139/139 library, 149/149 binary, 7/7
  claim, and 22/22 binding tests. IR/codegen and backend/claim approved, but type/
  safety rejected it at P1 because TASK_LEDGER's final status still called the
  completed gate future work. It remained unpublished. The corrected six-record
  tree's fresh exact full gate exits 0 with 139/139 library, 149/149 binary, 7/7
  claim, and 22/22 binding tests.
- Acceptance evidence: corrected authorization `91d2686` is triple-approved and
  public green in compiler `30915838213` / `30915838191`, Rust `30915839059`,
  CodeQL `30915834128`, and aggregate `92013770932`. Triple-approved tests-only
  `296276f` publicly reproduces exactly 30 false acceptances as the sole 22/23
  binding failure in compiler `30916807388` / `30916811627` and nightly Rust
  `30916810937`; CodeQL `30916806193` passes. After three public-red approvals,
  exact two-file implementation `a1ffeaec`, tree `f0088e65`, canonical diff
  `7a3fdb11`, received three exact approvals and passes formatting, focused 1/1,
  binding 23/23, the exact full local gate, compiler `30917539648` / `30917544307`,
  stable/nightly Rust `30917537292`, CodeQL `30917534448`, and aggregate
  `92019545168`.
- Result: only the frozen false-success surface now rejects. The claim boundary and
  R-002/R-011/matrix/capability state are unchanged.
- Closure evidence: the prepared six-record closure's fresh exact full gate exits 0
  with 139/139 library, 149/149 binary, 7/7 claim, and 23/23 binding tests.
- Closure acceptance: exact `d3811b00`, tree `c01088c4`, canonical diff `2799eb32`,
  received three exact approvals and was published unchanged. Compiler
  `30918433816` / `30918438945`, stable/nightly Rust `30918439169`, all three
  CodeQL analyses in `30918434204`, and aggregate `92022619964` pass. CORE-034 is
  closed without moving a semantic, capability, matrix, risk, backend, artifact,
  claim, history, or `master` boundary.

## DEC-045 - Re-rank the complete residual set after CORE-034

- Date: 2026-08-04
- Status: complete, read-only, and clean at public-green authorization `a31342e8`;
  exact R selected unanimously by final compatibility review.
- Decision: authorize `AUDIT-041` only as a static, read-only, independent re-ranking
  of R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016 from exact
  clean public closure `d3811b00`. Exclude every accepted slice through CORE-034 and
  inherit no prior candidate, label, preservation row, or order.
- Method: each of the type/safety, IR/codegen, and backend/claim reviewers must rank
  all eleven residuals with file/symbol evidence, name one exact bounded candidate
  or stop, describe trusted reachability and containment, state semantic decisions
  and phase count, give one deterministic failing specimen and preservation controls,
  and distinguish rejection, helper simulation, annotations, LLVM text, object
  emission, and hardware execution.
- Selection boundary: the lead may reconcile to at most one unanimously bounded
  residual or an explicit stop. More than two compiler phases, unresolved semantics
  or compatibility, hardware dependence, unsupported-type fallback, or valid-output
  uncertainty requires a stop.
- Authority boundary: after this six-record authorization is locally green,
  triple-approved, published unchanged, and public all-eight green, the audit remains
  read-only. It grants no test, implementation, semantics, capability, matrix, risk,
  workflow, dependency, backend, artifact, claim, history, or `master` authority.
- Gate evidence: the prepared authorization's fresh exact full gate exits 0 with
  139/139 library, 149/149 binary, 7/7 claim, and 23/23 binding tests.
- Authorization acceptance: exact `a31342e8`, tree `fbcd78b6`, canonical diff
  `313a1f6b`, received three exact approvals and passes compiler `30919164807` /
  `30919167478`, Rust `30919168162`, CodeQL `30919164869`, and aggregate
  `92025101785`.
- Result: all rankings placed R-002 first. Initial exact candidates split V/I/R;
  targeted comparison ranked V/I/R once and R/I/V twice. All three final
  compatibility reviews approved only initialized exact nonrecursive positive-count
  `Reference(Array(Tuple))` rejection at semantic and checked-admission boundaries,
  with both mutability flags, the 34-red/4-green matrix, and no classification move.

## DEC-046 - Initialized positive-count reference-array-tuple annotations fail closed

- Date: 2026-08-04
- Status: closed at exact triple-approved public all-eight-green closure `60ad91f7`.
- Decision: for an initialized binding whose annotation is exactly nonrecursive
  `Type::Reference(Type::Array(Type::Tuple(_), count), _)` with `count > 0`, validate
  the initializer, preserve checked Void and all existing initialized tuple-shape
  diagnostics, then reject in semantics and checked admission before mismatch, the
  checked generic-impl bypass, binding insertion, or raw generation. Match both
  reference mutability flags without assigning mutability meaning.
- Diagnostics: semantic `Error: Variable \`{name}\` uses an unsupported tuple type annotation directly beneath an array directly beneath a reference for an initialized binding.`; checked `checked IR binding \`{name}\` uses an unsupported tuple type annotation directly beneath an array directly beneath a reference for an initialized binding`; public keeps the existing semantic prefix.
- Context: use only existing semantic/checked binding traversal. Checked generic-
  function outer rejection and syntax-only generic trait defaults remain unchanged.
- Tests-first boundary: after authorization acceptance, one binding-contract aggregate
  must reclassify the existing immutable rows and expose exactly 34 false acceptances;
  direct count one/two proves `count > 0`, the context matrix proves existing
  traversal, and four count-zero semantic/checked observations remain green. Any
  different count or diagnostic is a stop.
- Implementation boundary: after separately reviewed public-red evidence, only the
  semantic analyzer and checked IR admission may add exact guards. No parser, raw IR,
  verifier, codegen, CLI, runtime, backend, reference/array/tuple value, ownership,
  layout, ABI, bounds, lowering, execution, matrix, capability, risk, or claim change
  is authorized.
- Gate evidence: the prepared six-record authorization's fresh exact full gate exits
  0 with 139/139 library, 149/149 binary, 7/7 claim, and 23/23 binding tests.
- Acceptance evidence: authorization `b74b1d29`, tree `3fc2d78f`, canonical diff
  `64fbd1fe`, received three exact approvals and is public all-eight green in
  compiler `30921372203` / `30921374216`, Rust `30921376655`, CodeQL
  `30921371268`, and aggregate `92032740349`. Triple-approved tests-only
  `f04e80c9`, tree `03a9f274`, canonical diff `9e04b6ad`, publicly reproduces
  exactly 34 false acceptances as the sole 23/24 binding failure in compiler
  `30922180824` / `30922181281` and nightly job `92035312036` in Rust
  `30922181764`; stable job `92035312020` was fail-fast cancelled, while CodeQL
  `30922176056` and aggregate `92035461619` pass. Three public-red reviews approved
  implementation authority.
- Implementation evidence: exact `b8fd5a17`, tree `77bd2536`, canonical diff
  `2f1e9920`, adds only the two nonrecursive positive-count guards and received three
  exact approvals. Formatting, focused 1/1, binding 24/24, and the exact full local
  gate pass with 139/139 library, 149/149 binary, 7/7 claim, and 24/24 binding tests.
  Compiler `30922853658` / `30922859177`, stable/nightly Rust `30922863203`, all
  three CodeQL analyses in `30922853619`, and aggregate `92037794056` pass.
- Result: only the frozen false-success surface now rejects. Count zero and every
  other frozen residual remain controls, not supported semantics. R-002 remains
  HIGH/CRITICAL and PARTIALLY CONTROLLED; R-011, matrix, capability, backend,
  artifact, and claim boundaries are unchanged.
- Closure acceptance: exact `60ad91f7`, tree `978aa98f`, canonical diff `818a8112`,
  changed only the six control records, received three exact approvals, and was
  published unchanged. Compiler `30923835957` / `30923837627`, stable/nightly Rust
  `30923838264`, all three CodeQL analyses in `30923834264`, and aggregate
  `92041128413` pass. CORE-035 is closed without moving a semantic, capability,
  matrix, risk, backend, artifact, claim, history, or `master` boundary.

## DEC-047 - Re-rank the complete residual set after CORE-035

- Date: 2026-08-04
- Status: complete, read-only, and classification-neutral at exact corrected public
  all-eight-green authorization `2d8a0c54`; U selected by two-to-one reconciliation
  and approved by all three final compatibility reviews.
- Closure acceptance: exact `60ad91f7`, parent `b8fd5a17`, tree `978aa98f`, canonical
  diff `818a8112`, changed only the six control records, passed the exact full local
  gate with 139/139 library, 149/149 binary, 7/7 claim, and 24/24 binding tests,
  received three exact approvals, and was published unchanged. Compiler
  `30923835957` / `30923837627`, stable/nightly Rust `30923838264`, CodeQL
  `30923834264`, and aggregate `92041128413` pass. CORE-035 is closed without a
  semantic, capability, matrix, risk, backend, artifact, claim, history, or `master`
  movement.
- Decision: authorize `AUDIT-042` only as a static, read-only, independent re-ranking
  of R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016 from exact
  clean public closure `60ad91f7`. Exclude every accepted slice through CORE-035 and
  inherit no prior candidate, label, preservation row, or order.
- Method: each type/safety, IR/codegen, and backend/claim reviewer must rank all
  eleven residuals with file/symbol evidence, name one exact bounded candidate or
  stop, describe trusted reachability and containment, state semantic decisions and
  phase count, give one deterministic failing specimen and preservation controls,
  and distinguish rejection, helper simulation, annotations, LLVM text, object
  emission, and hardware execution.
- Selection boundary: the lead may reconcile to at most one unanimously bounded
  residual or an explicit stop. More than two compiler phases, unresolved semantics
  or compatibility, hardware dependence, unsupported-type fallback, or valid-output
  uncertainty requires a stop.
- Authority boundary: after this six-record authorization is locally green,
  triple-approved, published unchanged, and public all-eight green, the audit remains
  read-only. It grants no test, implementation, semantics, capability, matrix, risk,
  workflow, dependency, backend, artifact, claim, history, or `master` authority.
- Gate evidence: the prepared authorization's fresh exact full gate exits 0 with
  139/139 library, 149/149 binary, 7/7 claim, and 24/24 binding tests.
- Correction history: first snapshot `4ce0de0d`, tree `350984b8`, canonical diff
  `347278c3`, passed that gate but was rejected before any independent push or
  branch-head publication. Type/safety reported P1 stale active-hypothesis wording;
  IR/codegen reported that issue at P2 and stale DEC-046 closure status at P1;
  backend/claim independently reported the closure status at P1. The rejected
  snapshot remains in corrected ancestry, ranking did not begin, and the correction
  changes no authority or classification boundary.
- Corrected authorization acceptance: exact `2d8a0c54`, parent `4ce0de0d`, tree
  `45d1c184`, correction canonical diff `b36d3d9b`, and cumulative canonical diff
  from CORE-035 closure `478e947a`, changed only the six control records, passed two
  fresh exact full gates, received three fresh exact approvals, and was published
  unchanged. Compiler `30924946683` / `30924950615`, stable/nightly Rust
  `30924951134`, all three CodeQL analyses in `30924945035`, and aggregate
  `92044919183` pass.
- Ranking result: type/safety ranked R-002/R-011/R-005/R-004/R-013/R-009/R-012/
  R-006/R-010/R-016/R-007 and selected valueless exact nonrecursive
  `Reference(Array(Tuple))` U. IR/codegen ranked R-002/R-011/R-005/R-004/R-006/
  R-009/R-013/R-012/R-010/R-007/R-016 and selected valueless exact
  `Array(Array(Array(Tuple)))` T. Backend/claim ranked R-011/R-002/R-005/R-004/
  R-006/R-013/R-007/R-010/R-009/R-012/R-016 and selected direct nonnegative
  homogeneous scalar-literal array bounds B.
- Reconciliation: type/safety and IR/codegen ranked U > T > B and stopped B because
  compile-time rejection versus runtime bounds behavior is unresolved; backend/claim
  ranked B > U > T. The lead chose U two to one. All three final compatibility
  reviews approved U as exact, count-insensitive, two-phase containment. B remains
  stopped pending policy and T remains a bounded fallback. The audit performed no
  edit, test, build, formatter, probe, artifact, hardware action, or external query.
- Classification boundary: selection is authority only for a later reviewed contract.
  It changes no language semantics, implementation, risk, matrix, capability,
  backend, artifact, claim, history, or `master` state.

## DEC-048 - Reject valueless exact reference-array-tuple annotations before IR

- Date: 2026-08-04
- Status: implementation triple-approved and public all-eight green at `26d18924`;
  exact classification-neutral closure `3f042e18` is triple-approved, public, and
  all-eight green.
- Decision: for a valueless binding whose annotation is exactly nonrecursive
  `Type::Reference(Type::Array(Type::Tuple(_), count), ref_flag)`, preserve semantic
  duplicate precedence and all four existing valueless tuple-shape diagnostics, then
  reject in semantic analysis before fallback insertion and independently in checked
  IR admission before no-value admission/raw generation. Both reference flags, every
  count including zero, and every tuple arity are included; no recursive shape is.
- Diagnostics: semantic
  `Error: Variable \`{name}\` uses an unsupported tuple type annotation directly beneath an array directly beneath a reference for an uninitialized binding.`;
  checked
  `checked IR binding \`{name}\` uses an unsupported tuple type annotation directly beneath an array directly beneath a reference for an uninitialized binding`.
  Public compilation preserves the existing semantic-error prefix.
- Context: use only existing semantic/checked binding traversal. Direct, top-level,
  block/control-flow/loop, non-generic impl, generic impl, and semantic generic-
  function routes are covered. Checked generic-function outer rejection and syntax-
  only generic trait defaults remain unchanged. No checked duplicate rule is added.
- Tests-first boundary: after authorization acceptance, only the binding-contract
  test file may change. All four existing acceptance occurrence blocks containing
  five exact-U source rows must be reclassified while siblings remain. One aggregate
  must expose exactly 34 false acceptances across counts zero/one, both flags, both
  phases, public compilation, top-level, generic, block, control-flow, loop, and impl
  routes, with exactly 40 preservation observations green. Focused 0/1 and binding
  24/25 must be the sole expected failure after 139/139 library, 149/149 binary, and
  7/7 claim passes.
- Implementation boundary: only separately reviewed public-red evidence may
  authorize exact guards in `semantic_analyzer.rs` and `ir_generator.rs`. More than
  those two phases, a different count/diagnostic, or any semantic/compatibility/
  valid-output uncertainty is a stop.
- Claim boundary: fail-closed rejection defines no reference, array, tuple, value,
  default, mutability, ownership, lifetime, layout, ABI, coercion, bounds, lowering,
  execution, backend, matrix, capability, risk, or stability meaning. Initialized
  CORE-035 count-zero behavior stays unchanged. R-002 remains HIGH/CRITICAL and
  PARTIALLY CONTROLLED; R-011 remains open pending bounds policy.
- Authorization basis: exact corrected public all-eight-green AUDIT-042 head
  `2d8a0c54`, tree `45d1c184`, with compiler `30924946683` / `30924950615`, Rust
  `30924951134`, CodeQL `30924945035`, and aggregate `92044919183` green.
- Gate evidence: the prepared authorization's fresh exact repository-root full gate
  exits 0 with 139/139 library, 149/149 binary, 7/7 claim, and 24/24 binding tests.
- Authorization acceptance: exact `697bb3b4`, tree `b0cfd37b`, canonical binary diff
  `0a92ad7a`, changed only six records, passed two exact gates, received three exact
  approvals, and is public all-eight green in compiler `30927281281` / `30927293459`,
  Rust `30927289178`, CodeQL `30927280707`, and aggregate `92052974430`.
- Tests-first evidence: exact one-file `d52b117e`, tree `76a3b2e9`, canonical binary
  diff `c2d5e46a`, received three exact approvals and exposes precisely 34 unexpected
  acceptances as the sole focused 0/1 and binding 24/25 failure after 139/149/7
  passes. Push `30927952017`, PR `30927956714`, nightly `92055067840`, and stable
  `92055068009` test logs reproduce it; CodeQL `30927952240` and aggregate
  `92055178151` pass. Three public-red reviews approved two-phase implementation.
- Implementation evidence: exact `26d18924`, tree `8aec746c`, canonical binary diff
  `543f8a1c`, adds only the two guards with 33 insertions and no deletions. It is
  triple-approved; formatting, focused 1/1, binding 25/25, the exact full local gate,
  compiler `30928759703` / `30928760789`, stable/nightly Rust `30928758562`, all
  three CodeQL analyses in `30928754859`, and aggregate `92057919831` pass.
- Result: the exact valueless U surface now fails closed before IR. This decision
  defines no reference, array, tuple, default, ownership, lifetime, layout, ABI,
  bounds, lowering, execution, backend, matrix, capability, risk, or stability
  meaning. R-002 remains HIGH/CRITICAL and PARTIALLY CONTROLLED; R-011 stays open.
- Closure gates: the additively corrected six-record closure's fresh and verification
  exact full gates each exit 0 with 139/139 library, 149/149 binary, 7/7 claim, and
  25/25 binding tests.
- Closure correction: first snapshot `39c8564b`, tree `7932dd42`, canonical binary
  diff `2cb44b26`, passed two exact gates. Type/safety approved; IR/codegen and
  backend/claim rejected at P1 because PROJECT_STATE retained CORE-035 `b8fd5a17` as
  the current public implementation. It was not independently published and remains
  in corrected ancestry. The additive correction points current implementation state
  to `26d18924` without changing this decision or any classification.
- Second correction: first additive correction `799c4181`, tree `1c8a883f`, canonical
  binary diff `9a1f5cd8`, changed only six records. Type/safety approved, but IR/codegen
  rejected it at P1 because this decision's status still called the completed
  verification gate pending. Its review round stopped before publication. The second
  additive correction aligns that status with the two recorded green gates and
  changes no decision, evidence, semantics, implementation, or classification.
- Second-correction gate: the fresh exact repository-root full gate exits 0 with
  139/139 library, 149/149 binary, 7/7 claim, and 25/25 binding tests, plus all
  downstream suites.
- Closure acceptance: exact `3f042e18`, parent `799c4181`, tree `15d56e0c`, canonical
  binary diff `ee8cbed0`, changed only the six control records, received three exact
  approvals, and was published unchanged. Push CI `30930377220`, PR CI `30930379386`,
  stable/nightly Rust `30930380195`, all three CodeQL analyses in `30930375201`, and
  aggregate `92063404658` pass. CORE-036 is closed without classification movement.

## DEC-049 - Authorize clean-head read-only AUDIT-043 after CORE-036

- Date: 2026-08-04
- Status: corrected authorization `5276df5b` is triple-approved and public all-eight
  green; AUDIT-043 ranking and final compatibility reconciliation are complete and
  read-only.
- Decision: authorize only a static, read-only, independent re-ranking of the complete
  remaining R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016 set
  from exact clean public CORE-036 closure `3f042e18`. Exclude every accepted slice
  through CORE-036 and inherit no prior candidate, label, preservation row, or order.
- Method: each type/safety, IR/codegen, and backend/claim reviewer must rank all eleven
  residuals with tracked file/symbol evidence, trusted reachability, exact containment,
  unresolved choices, phase count, deterministic failing specimen and preservation
  controls, and one bounded candidate or explicit stop. Reconciliation may select at
  most one unanimously bounded residual or stop.
- Semantic boundary: this audit defines no semantics or behavior. The existing
  compile-time-versus-runtime bounds ambiguity remains a stop; B is not implementation
  authority. Rejection, tests, helper simulation, annotations, LLVM text, object
  emission, and hardware execution remain distinct evidence classes.
- Authority boundary: authorization changes only the six control records. Once all
  prerequisites pass, the audit is read-only and grants no test, implementation,
  semantics, capability, matrix, risk, workflow, dependency, backend, artifact,
  claim, history, or `master` authority.
- Pre-acceptance gate evidence at corrected snapshot `5276df5b` (historical;
  superseded by acceptance below): the prepared authorization's fresh and verification
  exact repository-root full gates each exited 0 with 139/139 library, 149/149 binary,
  7/7 claim, and 25/25 binding tests, plus all downstream suites. At that point exact
  review and public acceptance remained pending.
- Correction history: first authorization snapshot `cb43d1bb`, parent `3f042e18`,
  tree `f0f19f5d`, canonical binary diff `ead99a7b`, changed only six records and
  passed both exact gates. All three reviewers rejected it at P1 because this
  decision's status still called those completed gates required; type/safety also
  found PROJECT_STATE's “next immutable snapshot” wording stale for the already-
  committed review target. It was not published and no ranking began. The additive
  correction aligns current state and changes no authority, semantics,
  classification, or public claim.
- Pre-acceptance additive correction gate (historical; superseded below): the fresh
  exact repository-root full gate exited 0 with
  139/139 library, 149/149 binary, 7/7 claim, and 25/25 binding tests, plus all
  downstream suites. At that point fresh review and public acceptance remained
  pending.
- Corrected authorization acceptance: exact `5276df5b`, tree `c3eaf3cf`, correction
  diff `b8b7586f`, cumulative diff `fe5376dc`, received three fresh approvals and is
  public all-eight green in push CI `30931510621`, PR CI `30931515125`, Rust
  `30931515426`, CodeQL `30931509579`, and aggregate `92067252294`.
- Result: initial candidates split R-009 versus R-002 two-to-one. After comparing the
  active compiler false success, LSP scope, and guard duplication, all three approve
  exact valueless nonrecursive three-array-tuple R-002 as the sole conditional
  selection, with R-009 fallback and R-011 stopped. A separate green shared-classifier
  prerequisite is mandatory; no behavior or implementation is authorized here.

## DEC-050 - Authorize behavior-neutral binding-annotation classifier prerequisite

- Date: 2026-08-04
- Status: exact additive correction `1dcfd869` is triple-approved, published
  unchanged, and public all-eight green. Characterization remains a separate green
  test-only boundary; implementation remains unauthorized.
- Decision: classify only exact existing binding-annotation structure as
  `ExistingExplicitRejection(RejectKind)`,
  `MatchesExistingContractShape(ContractKind)`, or `PreserveExistingBehavior` from
  annotation tree plus initializer presence. Contract match is routing metadata, not
  support/enforcement; preserve is inert; matching is nonrecursive.
- Truth table: preserve all ten current initialized/valueless rejection categories,
  including the positive-count-only initialized reference-array-tuple rule; exact
  scalar/uppercase-String/positive numeric-array contract shapes; and preservation of
  all other counts, names, wrappers, generics, references, and deeper topologies.
- Boundary: share structural classification only. Keep phase-specific diagnostics,
  semantic duplicate, initialized RHS and checked Void precedence, mismatch/generic
  gates, traversal, trait/generic exclusions, fallback/insertion, raw APIs, valid LLVM,
  and CPU/ROCm/CUDA behavior unchanged. The later R-002 shape remains accepted.
- Workflow: six-record authorization, green characterization-only evidence, then a
  separate three-source-file behavior-neutral refactor. Combining behavior or adding
  another copied guard is prohibited. Any behavior delta, third phase, or claim move
  is a stop.
- Pre-acceptance authorization-gate evidence (historical; superseded by acceptance
  below): exact repository-root `./tools/test.sh` exited 0 with
  139/139 library, 149/149 binary, 7/7 claim, and 25/25 binding tests, plus all
  downstream suites. The verification exact gate independently exited 0 at the same
  counts with all downstream suites. At that point exact review and later boundaries
  remained pending.
- Review correction: first ARCH-001 snapshot `63d8d599`, parent `5276df5b`, tree
  `28cd120c`, canonical binary diff `9fef5adf`, received type/safety and IR/codegen
  approvals but a backend/claim P1 rejection because pre-acceptance AUDIT-043 pending/
  no-ranking evidence remained in present tense in five records beside the completed
  result. It was not published. The additive six-record correction makes only that
  chronology historical; DEC-050 and every behavior/capability boundary remain exact.
- Pre-acceptance additive chronology-correction gates (historical; superseded below):
  fresh and verification exact repository-root
  `./tools/test.sh` each exited 0 with 139/139 library, 149/149 binary, 7/7 claim, and
  25/25 binding tests, plus all downstream suites. At that point fresh exact review,
  unchanged publication, and public acceptance remained pending.
- Authorization acceptance: exact `1dcfd869`, parent `63d8d599`, tree `b537023c`,
  correction diff `e5ee8aa7`, cumulative diff `5208cb6e`, received three fresh exact
  approvals and is public all-eight green in push CI `30934518525`, PR CI
  `30934523152`, Rust `30934523078`, CodeQL `30934519513`, and aggregate
  `92077350363`. No characterization, source, behavior, or capability change occurred.
- Authorization-acceptance sync gates: fresh and verification exact repository-root
  `./tools/test.sh` each exit 0 with 139/139 library, 149/149 binary, 7/7 claim, and
  25/25 binding tests, plus all downstream suites. Exact review and public acceptance
  of the committed sync remain pending.
- Acceptance-sync review correction: first snapshot `4c18450a`, parent `1dcfd869`,
  tree `ea7b91c9`, canonical binary diff `7be565db`, received backend/claim and IR/
  codegen approvals but a type/safety P1 rejection because three records prematurely
  declared characterization eligible before sync acceptance. It was not published.
  The additive six-record correction restores the gate without changing DEC-050,
  behavior, capability, or claim state.
- Eligibility-correction gates: fresh and verification exact repository-root
  `./tools/test.sh` each exit 0 with 139/139 library, 149/149 binary, 7/7 claim, and
  25/25 binding tests, plus all downstream suites. Fresh review and public acceptance
  remain pending.

## DEC-051 - Bound mutable-reference transport to one direct call-scoped scalar loan

- Date: 2026-08-05
- Status: accepted public at exact implementation `e3ff1658039f8b9e20f18981c3d6198a07e79e92`;
  all eight checks and pinned native execution are green.
- Decision: admit only a unique non-generic internal function whose sole parameter is
  exact `&mut Int`, `&mut Float`, or `&mut Bool`, with a scalar or Void result. The
  sole admitted argument is direct `callee(&mut owner)` for an initialized owned local
  mutable scalar of the exact pointee. The loan begins at argument evaluation and ends
  immediately after the call; the callee may read and write through its parameter.
- Architecture: extend the shared reference-function contract and add one whole-call
  disposition using the existing local source facts. Semantic typing and checked
  admission consume that classification. Checked IR represents a distinct mutable
  reference signature and writable parameter binder; caller IR must contain an exact
  adjacent checked borrow/call/end sequence. The independent verifier proves binder
  coverage, pointee, active origin, write identity, and release topology before typed
  `double*`/`i1*` LLVM lowering.
- Exclusions: stored alias arguments, forwarding, reborrowing, mixed or multiple
  parameters, reference results, nonlocal/temporary/projected/non-scalar pointees,
  escape/storage/capture, NLL, lifetime inference, drop, stable ABI/FFI, accelerator
  meaning, and memory-safety claims remain unsupported.
- Evidence gate: exhaustive source/IR/LLVM/CLI and private verifier-corruption tests,
  the exact repository-root gate, all eight public checks, pinned LLVM/Clang 22
  verification and object/link stages, and exact native exit 251 are mandatory.
  Exact tree `4efca0a523ae60d0d3020f925e0567f430dad9dd` and stable patch ID
  `77377ea77150931b709898d2fdf2bbcd9713c1c1` pass push CI `30991851164`, PR CI
  `30991854370`, Rust CI `30991853837`, and CodeQL `30991850056`. Stable job
  `92259593558` uses LLVM/Clang 22.1.8, externally verifies, machine-verifies, object-
  lowers, links, and executes exact native exit 251 with 167/167 library and 173/173
  binary tests. `CORE-055` remains accepted and unchanged. PR checkpoint/merge strategy
  and a structured evidence manifest remain separately authorized scaling work.

## DEC-052 - Admit call-scoped child reborrows from mutable scalar references

- Date: 2026-08-05
- Status: locally green implementation candidate; public checks and pinned native
  execution remain pending.
- Decision: retain the exact accepted CORE-056 sole mutable scalar-reference signature
  and add the complete identifier-origin argument class: an initialized in-scope
  CORE-055 local `&mut Int`/`Float`/`Bool` alias or the current function's matching
  mutable-reference parameter. Passing either identifier creates a fresh synchronous
  child reborrow for the exact call; it does not move, copy, end, or escape the parent.
- Ownership rule: the parent is unavailable during the exact adjacent child-borrow/
  call/end interval and is restored afterward. A local parent alias's root owner remains
  unavailable until that alias's existing lexical end. Repeated calls, multi-hop
  forwarding, branch/loop use, arbitrary declaration order, direct modules, and
  terminating recursion are within this bounded class.
- Architecture: `classify_reference_call` remains the sole topology and source-fact
  decision. Its supported contract distinguishes direct-owner borrowing from mutable-
  reference identifier reborrowing; semantics, move tracking, checked admission, and
  checked lowering consume that shared result. Checked IR reuses mutable borrow/end
  identity for a child of an active local alias or mutable-reference parameter. The
  verifier independently proves parent provenance, exact pointee, non-overlap, parent
  exclusion, adjacency, and matching restoration before existing private typed-pointer
  LLVM lowering.
- Exclusions: scalar/immutable/moved/uninitialized identifiers, root-owner access while
  a local alias lives, overlapping children, `&mut *alias`, multiple or mixed signatures,
  reference results, non-scalar/projected/temporary pointees, relocation, reassignment,
  storage/capture/escape, explicit or inferred lifetime policy, NLL, drop, stable ABI/
  FFI, accelerator meaning, performance, release, stability, and memory-safety claims
  remain unsupported or preserved.
- Evidence gate: the exhaustive CORE-057 target, private verifier corruptions, focused
  compatibility targets, raw-path containment, CLI artifact hygiene, tracked composed
  module example, exact repository-root gate, all eight public checks, pinned LLVM/
  Clang 22 verification/object/link stages, and exact native exit 253 are mandatory.
  The new target, all focused suites, `cargo check --all-targets`, the full Cargo suite,
  and the exact record-synced repository-root gate pass at 169/169 library and 175/175
  binary tests. CORE-056 remains accepted.
  PR #4 stays draft and unmerged; checkpoint strategy and structured evidence generation
  remain separate scaling work.

## DEC-053 - Admit flat heterogeneous Copy-scalar tuples as a private product layout

- Date: 2026-08-05
- Status: locally green implementation candidate; public checks and pinned LLVM/Clang
  22 execution remain pending. DEC-052/CORE-057 is accepted public at exact commit
  `7c108ff0ae0e9686209378deec5ce1de61bff17b` with all eight checks and native exit
  253 green.
- Decision: inside admitted non-generic top-level functions, admit immutable tuples of
  arity two or greater whose ordered elements are exactly `Int`, `Float`, or `Bool`.
  The complete bounded class includes literal construction, inferred or exact binding,
  whole-value Copy aliases, repeated reads, constant in-bounds projection, immediate
  projection, scalar/tuple-only internal parameters and returns, forwarding, CFG,
  terminating direct recursion, and flattened direct modules.
- Shared contract: `tuple_contract` is the only source classifier for annotation,
  inferred element product, direct binding equality/mutability, execution context, and
  projection. The older binding-annotation table delegates initialized direct tuples;
  nested array/reference quarantines retain their established classifications. Enum-
  bearing function transport explicitly consumes the shared classifier in preserved
  context so tuple results or parameters do not broaden that ownership class.
- IR/backend: checked IR retains ordered `LogicalType::Tuple`, checked tuple allocation,
  and checked tuple field-pointer identities. The independent verifier proves minimum
  arity, exact scalar schema/order, field index/type, place/result separation,
  dominance, signature equality, and metadata stability. LLVM uses a private literal
  aggregate with `double` for Aero `Int`/`Float` physical compatibility and `i1` for
  `Bool`, with typed GEP/load/store and no pointer/integer conversion. This is not a
  stable source layout, calling convention, ABI, FFI, or zero-cost claim.
- Exclusions: unit/unary/nested/non-scalar tuples, mutable bindings or assignment,
  destructuring/patterns, dynamic/out-of-range projection, tuple arrays/fields/payloads/
  references, generic/impl/closure execution, tuple-bearing `main`, drop/destruction,
  heap, accelerator execution, performance, release, stability, and general aggregate-
  safety claims remain unsupported or preserved.
- Evidence gate: the exhaustive CORE-058 integration target proves parser retention,
  arbitrary scalar product shapes, binding/Copy/projection, source-order effects,
  calls/returns/CFG/recursion/modules, exact negative boundaries, checked identities,
  verifier corruptions, raw containment, CLI no-artifact hygiene, and LLVM anchors.
  All compiler unit and integration targets pass. The tracked composed module example
  builds and executes locally through Visual Studio Clang at exact exit 23. The exact
  repository-root gate, all eight public checks, and pinned external LLVM/Clang 22
  verification/object/link/native evidence are still mandatory before acceptance.
- Scaling boundary: PR #4 remains draft and unmerged. A controlled checkpoint/merge
  strategy, structured evidence manifest, and periodic broader release-eligibility
  system gate remain separately scoped work; this hard layout slice does not silently
  convert them into capability claims.

## DEC-054 - Generalize immutable references across exact admitted Copy-data places

- Date: 2026-08-05
- Status: accepted public at exact commit
  `5a78eb5d670045277532cc3cdc9a6144b1449895`, tree
  `03fbdd58e836532dc8a4f95a0bb3c0402b1e5f1c`, and stable patch ID
  `62a23bef479f22d3d9da22fc4bf753c7610c3e77`. All eight checks pass; stable job
  `92291545518` uses LLVM/Clang 22.1.8 and records 173/173 library tests, 179/179
  binary tests, external and machine verification, object/link, and native exit 37.
- Decision: immutable `&owner` and immutable-reference internal transport may use the
  exact already-admitted Copy-data place universe: `Int`, `Float`, `Bool`, CORE-058
  flat Copy-scalar tuples, fixed numeric arrays, fixed arrays of one exact Copy struct,
  and finite acyclic Copy structs. This admits no new value or layout class. Borrow
  origins remain initialized in-scope local or parameter identifiers; immutable
  aliases remain Copy and use the existing lexical validity/release policy.
- Shared contract: `copy_place_contract` is the sole source classifier for this class
  and returns `Supported`, `ExplicitlyRejected`, or `Preserved`. It delegates tuple
  products to `tuple_contract` and named/array schemas to `StructRegistry`. Local
  borrow, exact annotation, dereference, immutable-reference signature admission,
  semantic analysis, and checked admission consume the result; older topology guards
  do not acquire a second aggregate whitelist. The independent verifier does not trust
  source admission and proves the resulting recursive schema.
- Function product: a reference-bearing non-`main`, non-generic internal function may
  have any number/order of immutable-reference or owned parameters from that exact Copy
  universe and return an owned member or `Void`. Direct borrows, immutable-reference
  identifiers, aliases, forwarding, CFG, terminating recursion, and flattened direct
  modules are included. Reference results remain rejected. The CORE-056/057 mutable-
  reference product remains exactly one scalar mutable-reference parameter.
- IR/backend: checked immutable borrow and reference-parameter binders retain exact
  recursive `LogicalType` pointees. Aggregate dereference loads into a fresh aggregate
  Copy place. The verifier checks source-place schema, binder schema, place identity,
  dominance, and function equality. LLVM uses pointers to the already accepted private
  scalar/aggregate types, typed zero-offset GEP, loads/stores, and no pointer/integer
  conversion or unrelated bitcast. No stable layout, calling convention, ABI, FFI, or
  safety claim follows.
- Exclusions: mutable aggregate references; String, enum, reference, generic, or
  unsupported layouts; unit/unary/nested/non-scalar tuples; temporary, projected, or
  dereferenced borrow origins; uninitialized/moved/nonlocal owners; reference results;
  storage/capture/escape; NLL, explicit or inferred lifetime policy; drop; heap;
  concurrency; stable ABI/FFI; accelerator execution; performance; release; stability;
  and general memory-safety claims remain rejected or preserved.
- Accepted evidence: exhaustive classifier shapes, one full vertical integration target,
  verifier corruptions, reference/aggregate/tuple/enum compatibility, raw containment,
  deterministic LLVM, CLI artifact hygiene, direct modules, exact repository-root
  gate, local native exit 37, all eight public checks, and pinned LLVM/Clang 22 verify/
  machine-verify/object/link/native evidence pass at the accepted identity above.
- Scaling boundary: the three-way classifier directly addresses duplicated topology
  growth without broadening semantics. PR #4 remains an unmerged integration program;
  controlled checkpoint/merge strategy, structured checkpoint-manifest generation,
  and periodic broader release-eligibility system gates remain separately authorized
  scaling work and may not be displaced by a sequence of convenient bounded slices.

## DEC-055 - Generalize whole-place mutable references across admitted Copy-data

- Date: 2026-08-05
- Status: accepted public at exact commit
  `7c7a47a471460dfe2276ea63cc4964fa59ad54be`, tree
  `e9863de79a69766114020060a138c94357005351`, and stable patch ID
  `ec2c33060e33ca6e52894fa1a18daf5b5d9c6ba7`; all eight checks, 174/174
  library tests, 180/180 binary tests, and pinned native exit 59 are green.
- Decision: admit mutable references only to initialized mutable whole-owner places in
  the exact CORE-059 Copy-data universe. Mutable aliases retain the established
  exclusive lexical-loan model; whole dereference reads produce owned Copy values and
  whole dereference writes replace one exact pointee after evaluating the RHS once.
  Field, index, tuple-projection, dereference, temporary, and computed borrow origins
  remain rejected because projected provenance semantics are not frozen.
- Shared contract: `copy_place_contract` classifies immutable and mutable execution
  contexts as `Supported`, `ExplicitlyRejected`, or `Preserved`, delegating tuple and
  recursive named/array identity to their existing classifiers. Annotation, local
  borrow, dereference, assignment, the retained sole-mutable-reference signature,
  direct call, child reborrow, semantic analysis, and checked admission consume that
  one predicate. Scalar assignment IR remains distinct, but no mutable-reference phase
  retains a scalar-versus-aggregate pointee whitelist.
- Function product: retain exactly one mutable-reference parameter and no companions in
  a non-`main`, non-generic internal function. Its pointee and owned result may be any
  admitted Copy-data member, with `Void` also allowed as the result. Direct owner loans,
  alias child reborrows, forwarding, CFG, terminating recursion, and flattened direct
  modules are included. Reference results and mixed/multiple signatures remain rejected.
- IR/backend: checked IR introduces an exact typed mutable Copy-place owner and carries
  recursive schema through borrow, read, whole write, child loan, and end. Independent
  verification requires adjacent initialization, exact owner/reference/value schemas,
  active-loan provenance, and checked writes, forbidding generic stores through a
  reference. LLVM uses the existing private typed scalar/aggregate pointers, loads, and
  stores without pointer/integer conversion or unrelated bitcast. No stable layout,
  calling convention, ABI, FFI, or safety claim follows.
- Evidence gate: the exhaustive CORE-060 target, shared-classifier shape proof,
  verifier corruptions, adjacent mutable/immutable/tuple/aggregate/enum compatibility,
  raw containment, CLI no-artifact hygiene, tracked two-module example, full Cargo,
  exact root gate, all eight public checks, and pinned LLVM/Clang 22 verification,
  machine verification, object/link, and native exit 59 are mandatory. The accepted
  identity passes those local and public gates with 174/174 library tests, 180/180
  binary tests, every integration target, and pinned native exit 59.
- Scaling boundary: this hard ownership/layout/IR/backend slice and the shared classifier
  answer immediate combinatorial guard growth without authorizing projected semantics.
  PR #4 stays draft and unmerged. Checkpoint/merge strategy, a structured evidence
  manifest, and broader release-eligibility system design remain separate work, while
  the exit-59 composition is the required periodic system gate for this checkpoint.

## DEC-056 - Unify direct owned reassignment across admitted Copy-data

- Date: 2026-08-05
- Status: locally green implementation candidate; exact record-synced root, immutable
  commit identity, public checks, and pinned LLVM/Clang 22 native exit 83 remain
  pending. DEC-055/CORE-060 is accepted public at the exact identity above.
- Decision: admit `target = rhs;` only for the nearest initialized owned local
  identifier declared `let mut` when target and RHS have one exact admitted Copy-data
  type. This generalizes the accepted scalar statement to flat Copy tuples, fixed
  numeric arrays, fixed Copy-struct arrays, and finite acyclic Copy structs, including
  admitted zero-length arrays. RHS evaluation occurs once before whole-place
  replacement. No projected or partial target semantics are introduced.
- Shared contract: add one admitted owned-assignment execution context to
  `copy_place_contract`. The assignment classifier continues to own target topology,
  locality, initialization, mutability, ownership, and exact RHS equality, while the
  shared predicate alone owns Copy-data schema. Semantic analysis and checked admission
  consume that contract; unsupported String, enum, reference, generic, nested/non-Copy,
  projected, compound, chained, and expression-assignment topologies remain rejected or
  preserved.
- IR/backend: replace the scalar/aggregate checked split with one
  `CheckedMutableCopyPlaceAlloca` and one `CheckedCopyPlaceAssignment`. Exact logical
  schema survives adjacent initialization, dominance, active-loan exclusion, RHS type
  proof, and collision checks. LLVM retains the private `double`/`i1` scalar storage and
  uses exact typed tuple/array/struct whole stores without pointer/integer conversion or
  unrelated bitcast.
- Evidence gate: one exhaustive source-to-native target covers every admitted schema,
  exact/inferred bindings, calls, recursion, CFG, shadowing, borrow boundaries, direct
  modules, fail-closed negatives, verifier identity, raw containment, CLI artifact
  hygiene, and a tracked exit-83 composition. Focused implementation, classifier,
  verifier, and adjacent scalar/aggregate/reference targets, 174/174 library tests,
  180/180 binary tests, every integration target, the exact repository-root gate, and
  local native exit 83 pass. Immutable identity and the public pinned lane remain
  mandatory before acceptance.
- Scaling boundary: this is intentionally a hard ownership/mutation consolidation, not
  another convenient compile-time leaf. It reduces duplicated topology rules without
  authorizing projected semantics. PR #4 remains a draft integration program; its body
  must be synchronized to every accepted head. Controlled checkpoint/merge strategy,
  structured evidence-manifest generation, and broader release-eligibility design
  remain separate tasks, while the composed exit-83 lane supplies the periodic system
  trace for this checkpoint.

## DEC-057 - Keep closures parsed-only and fail closed before checked IR

- Date: 2026-08-05
- Status: locally green amendment to the CORE-061 candidate; the exact
  record-inclusive root gate and local native exit 83 pass. The commit containing this
  decision becomes the immutable amended candidate; public checks and pinned
  LLVM/Clang 22 system evidence remain pending. Pushed `a85f47b` is intermediate only.
- Decision: retain current closure tokens, AST parameter/body shape, and the opening
  `|` source location, but reject every executable closure expression with exactly
  `closure expressions are parsed but unsupported in executable code` at that
  location. The outer encountered closure wins deterministically without activating
  body semantics. Both semantic inference paths and an independent checked-admission
  guard consume one shared diagnostic contract.
- Lowering boundary: remove the legacy closure lowerer, generated `__closure_*`
  functions, compile-time callable-binding exception, and every unknown closure
  parameter/result-to-`i32` fallback. The deprecated raw API may quarantine a closure
  only as an inert unsupported value and may not manufacture `Ty::Fn`, a signature,
  capture environment, layout, call target, symbol, or LLVM definition. Trusted paths
  reject before checked IR.
- Negative product: inferred/explicit bindings, comparisons, function arguments and
  returns, arrays/struct storage, captures, binding calls, unknown annotations, nested
  source positions, direct modules, and `check`/`build`/`run` no-artifact behavior are
  covered. Parser retention and ordinary function/reference/enum/Match/tuple/array/
  module plus CORE-043 through CORE-060 behavior remain positive controls.
- Exclusions: captures, capture analysis, callable ABI, storage/transport, invocation,
  ownership/lifetimes, generic/trait closure integration, heap/runtime behavior,
  accelerator execution, and all positive closure semantics remain unfrozen and
  unsupported. This amendment is a false-success closure, not a closure capability;
  CORE-061 still ends in the executable direct-assignment/native-exit-83 slice.
- Scaling boundary: PR #4 stays draft and unmerged; controlled checkpoint/merge work,
  structured evidence-manifest generation, and broader release-eligibility gates
  remain separately authorized. The current composed native gate remains mandatory,
  and future selection must not avoid harder ownership/module/ABI/GPU work in favor of
  convenient bounded leaves.

## DEC-058 - Classify finite CopyData composition by one recursive contract

- Date: 2026-08-05
- Status: accepted public at exact implementation
  `e62fd7470d8cb929d57d0c063815d7a99005d768`, tree
  `d2aff21a54c42d1ce649ef6668d50a4908315738`, and stable patch ID
  `458feb5ebc1355d83793084009e5ea7895a22129`. All eight checks pass; stable job
  `92344809072` uses LLVM/Clang 22.1.8 and executes exact native exit 109.
- Decision: define executable `CopyData` as the least fixed point of `Int`, `Float`,
  `Bool`, fixed arrays of CopyData at any parsed count, tuples with at least two ordered
  CopyData elements, and unique nongeneric nonempty named structs whose declaration-
  ordered fields are CopyData and whose named dependency graph is finite and acyclic.
  Exact count, tuple order/arity, struct identity, and recursive field schema are type
  identity. One immutable registry-backed classifier resolves both `Type` and `Ty` and
  supplies exact `LogicalType`.
- Composition: inferred/exact bindings, literals and typed empty arrays, whole copies,
  direct mutable-owner reassignment, immutable/mutable whole-place references, exact
  internal parameters/results/calls/forwarding/terminating recursion, dynamic fixed-
  array indices, chained value projection, and flattened direct modules consume this
  contract. Immutable references remain copyable for established ownership tracking,
  but references are not stored CopyData.
- IR/backend: checked IR retains recursive schema for storage, function transport, and
  field/tuple/index places. The verifier independently proves finite valid schemas,
  exact construction/member/call/store identity, named-schema consistency, dominance,
  and corruption controls. LLVM recursively lowers literal tuples, fixed arrays, and
  private identified structs without fallback `i32`, pointer/integer conversion, or
  unrelated bitcast.
- Exclusions: unit/unary tuples; String/references/functions/closures/enums/generics/
  traits/Option/Result/collections as stored data; empty/duplicate/unresolved/generic/
  cyclic structs; dynamic arrays/slices; aggregate comparison/destructuring; projected
  borrow/write; contextual coercion; public layout/ABI/FFI; lifetime/drop/memory-safety;
  accelerator/performance/release/stability claims remain unsupported or separately
  governed.
- Evidence gate: the exhaustive target covers every immediate constructor pairing and
  the complete source product, fail-closed negatives, direct modules, checked metadata,
  verifier corruptions, deterministic LLVM, CLI artifact hygiene, and native exit 109.
  The local exact root gate passes 178/178 library and 184/184 binary tests, every
  integration/claim target, Phase 5 controls, and docs. CodeQL `31017349668`, push CI
  `31017352912`, PR Rust CI `31017357342`, and PR CI `31017358299` pass on the immutable
  implementation; the stable lane externally verifies, machine-verifies, object-lowers,
  links, and executes exact exit 109.
- Scaling boundary: this task directly addresses combinatorial aggregate topology rules
  through one shared classification without broadening unrelated semantics. PR #4 stays
  draft and unmerged; controlled checkpoint/merge strategy and structured evidence-
  manifest generation remain separate work. Hard ownership/module/runtime/accelerator
  classes must not be deferred indefinitely, and periodic source-to-native system gates
  remain mandatory.

## DEC-059 - Extend unary owned-enum payloads to recursive CopyData

- Date: 2026-08-05
- Status: accepted public at exact implementation
  `2a5c3c58192dc65116c436d6ae76da5829eeba52`, tree
  `8a5cef6b14214e76349a41f6997d5fa19595858f`, and stable patch ID
  `276af069807b6f59c233a2f281c1b0d0b8c899b8`, with verified native-link repair head
  `bebd0b6a87108219497187a5952688c95c397158`. Formatting,
  all-target/all-feature checking, correctness Clippy, docs, 179/179 library tests,
  185/185 binary tests, verifier corruptions, the exhaustive target, the exact root
  gate, all eight public checks, and pinned LLVM/Clang 22 native exit 113 pass.
- Decision: a unique nongeneric nonempty enum is executable when every variant is unit
  or contains exactly one value accepted by DEC-058's recursive `CopyData` contract.
  Variant order and the exact recursive payload schema are identity. `EnumRegistry`
  must delegate payload annotation classification to `StructRegistry`; semantic
  initialization, preflight, inference, checked admission, and verifier registration
  consume the same resolved binding/schema rather than scalar or topology placeholders.
- Value and ownership boundary: constructors evaluate one exact payload once. Match
  remains exhaustive with one explicit identifier binding per payload variant and a
  scalar result. The bound CopyData value may use already accepted projections and
  Copy operations, but the containing enum remains non-Copy and existing whole-enum
  move/transport rules remain unchanged.
- IR/backend: existing checked enum construction, extraction, dispatch, parameter,
  call, return, and schema identities carry recursive payloads. Unit-only and
  scalar-only schemas preserve their accepted private lowering. Any schema containing
  an aggregate payload uses a private `i32` tag plus one exact typed lane per
  payload-bearing variant, with typed zero values in inactive lanes. The verifier
  rejects unsupported nested leaves, conflicting named schemas, fallback scalar
  payloads, changed lane identity, and unguarded extraction before trusted codegen.
- Exclusions: multi-field/struct variants, wildcard/guard/nested destructuring,
  aggregate Match results, enum fields/arrays/general storage, enum borrowing or
  mutation, partial moves, generic/recursive enums, new CFG ownership, drop/lifetimes,
  public layout/ABI/FFI, closures, accelerators, performance, release, and stability
  remain unsupported or separately governed.
- Scaling boundary: this slice removes the scalar-versus-recursive payload partition
  by reusing a shared classifier. It does not authorize projected-place semantics or
  runtime bounds behavior, whose specifications remain ambiguous. PR #4 remains draft;
  checkpoint strategy, evidence-manifest automation, and periodic composed system gates
  remain active separate controls.
- Public evidence: CodeQL run `31022757247`, push CI `31022756615`, PR CI
  `31022760915`, and PR Rust CI `31022761529` pass on verified head `bebd0b6`.
  Stable job `92363420145` installs LLVM/Clang 22.1.8, proves the known-invalid
  verifier control, externally verifies, machine-verifies, object-lowers, explicitly
  links the private non-PIE executable, and observes exact exit 113; nightly job
  `92363420286` independently observes exit 113.

## DEC-060 - Extend direct owned replacement to admitted enums

- Date: 2026-08-05
- Status: accepted public at exact implementation
  `79aed71371e192a07218d437e882a863653b6826`, tree
  `ac80c49aca3fb875c44d132f930567e95d81f698`, and stable patch ID
  `1bb2c9c19f6d427122f83bffc59d3f18f0a5b3e4`. The
  focused exhaustive target, verifier corruption unit, affected compatibility ring,
  formatting, all-target/all-feature checking, correctness Clippy, docs, the exact root
  gate, and complete Rust surface pass at 180/180 library and 186/186 binary tests. All
  eight public checks pass; stable job `92376666972` proves the pinned LLVM/Clang 22
  external/machine/object/link/native exit-131 lane and nightly job `92376666842`
  repeats exit 131.
- Decision: the direct mutable owned-place universe is the union of DEC-058 recursive
  finite `CopyData` and the exact non-Copy enum schemas accepted by DEC-059. One shared
  classifier resolves target type, mutability, initialization, ownership, exact RHS,
  and supported/rejected/preserved disposition across semantic analysis and checked
  admission. It must not duplicate enum topology tables or manufacture a scalar type.
- Source semantics: an inferred or exact annotated `let mut` enum local may be replaced
  by a fresh exact variant, an exact enum-returning call, or a distinct initialized
  local of the same enum. The RHS evaluates once. A distinct local source moves and
  cannot be reused; the target becomes owned and initialized. Direct self-replacement
  is rejected. Existing conservative source-order/branch/loop ownership remains exact;
  no new join or path-sensitive behavior is inferred.
- IR/backend: `CheckedMutableOwnedPlaceAlloca` and
  `CheckedOwnedPlaceAssignment` supersede the CopyData-named checked identities for
  both admitted classes. The verifier independently requires valid exact recursive or
  enum schema, one adjacent initializer, dominance, collision freedom, exact
  place/value identity, and checked later writes. Only CopyData places may enter the
  accepted borrow identities. Private LLVM lowers an enum place and replacement with
  the already accepted exact enum type and typed load/store, without byte storage,
  fallback `i32`, bitcast, or public representation.
- Exclusions: enum fields/arrays/general storage, enum or projected borrowing, partial
  moves, multi-field/generic/recursive enums, new CFG ownership, drop/destructors,
  lifetimes, aggregate Match results, nested destructuring, stable layout/ABI/FFI,
  closures, accelerators, performance, release, and stability remain unsupported or
  separately governed.
- Scaling boundary: the shared owned-place classifier directly contains the reported
  topology-rule growth without broadening preserved shapes. This deliberately selects
  a hard ownership/storage slice. Its tracked multi-capability source-to-native gate is
  mandatory. PR #4 remains draft/unmerged; controlled checkpoint strategy and
  structured manifest generation remain separately authorized work.

## DEC-061 - Join admitted enum ownership across acyclic conditionals

- Date: 2026-08-05
- Status: accepted public at exact implementation
  `f4daeea6d7b032e686b4c7d184fe80ef38076665`, tree
  `7cd4ec6da2d9ce44f63741222a5b128396358bfe`, and stable patch ID
  `708c1a6cab096f89e76577212a241554225897a2`. The local gate, all eight public checks,
  and pinned LLVM/Clang 22 native exit 137 pass. This is not a merge or a
  safety/stability/ABI claim.
- Decision: ownership joins are admitted only for existing owners of exact non-Copy
  enum schemas accepted by DEC-059/060. Each sibling `if` arm starts from the same entry
  snapshot. Definitely returning arms are excluded, missing `else` contributes entry,
  and reachable states join as `Owned`, `Moved`, or `MaybeMoved`. Any later operation
  requiring ownership rejects a `MaybeMoved` value deterministically.
- Shared source contract: one `ownership_flow` classifier owns the structural return
  predicate, conditional join, and explicit loop rejection for semantic analysis and
  checked admission. Branch-local shadows remain local; multiple owners join
  independently. Consuming loop conditions and ownership changes reaching a backedge
  reject because no fixed-point proof is claimed.
- Independent proof: checked-IR verification derives enum owner identity from exact
  results and mutable places, maps loads back to the place owner, and computes consumed-
  owner unions across CFG predecessors and cycles. It accepts one consumption in each
  mutually exclusive arm and exact replacement, and rejects serial, partial-merge,
  cyclic, and unreplaced-place double consumption before trusted LLVM.
- Evidence: the exhaustive source/direct-module/checked-IR/verifier/LLVM/CLI target,
  focused corruption controls, affected compatibility ring, 182/182 library tests,
  188/188 binary tests, formatting, all-target/all-feature checking, correctness
  Clippy, docs, and exact repository-root gate pass locally. The tracked public workflow
  passes LLVM/Clang 22.1.8 external and machine verification, object lowering, explicit
  private link, and native exit 137 in stable job `92454648190`; nightly job
  `92454648318` repeats exit 137.
- Exclusions: loop fixed points, `break`/`continue` transport, conditional target
  reinitialization, partial moves, enum borrowing/projection/field or array storage,
  new enum topology, general CFG ownership or borrowing, drop/destructors, lifetimes,
  stable layout/ABI/FFI, closures, accelerators, performance, release, stability, and
  PR merge remain unsupported or separately governed.
- Scaling boundary: this hard CFG-ownership slice uses one class target and one shared
  classifier rather than growing duplicated topology rules. PR #4 remains a draft
  integration program; controlled checkpoint strategy, structured evidence-manifest
  generation, and periodic composed system gates remain active separate controls.
- Public evidence: CodeQL run `31049983361` passes actions job `92454645397`, Python
  job `92454645485`, Rust job `92454645478`, and aggregate `92454777531`; push CI
  `31049982243` / `92454638146`, PR CI `31049985250` / `92454648791`, and PR Rust CI
  `31049985417` pass. Stable job `92454648190` installs LLVM/Clang 22.1.8, rejects the
  known-invalid fixture, externally verifies, machine-verifies, object-lowers,
  explicitly links, and observes exit 137; nightly job `92454648318` repeats exit 137.

## DEC-062 - Admit only freshly defined enum owners on statement-loop cycles

- Date: 2026-08-05
- Status: accepted public for CORE-066 at exact implementation
  `e40804ea86888b38548fd5bf42926be2be7eb5ed`, tree
  `6cea8bbf63aa7aafb43fbb25152dd860f6684aae`, and stable patch ID
  `7c4e6ac77db90dc7c83048922382903958c09632`. The serialized exact root gate, all
  eight public checks, pinned LLVM/Clang 22.1.8, and stable/nightly native exit 149
  pass.
- Decision: every currently admitted unit-or-unary-CopyData enum may be freshly
  constructed or returned, locally bound, and consumed once per dynamic iteration of
  checked `while`, fixed-array `for`, or `loop`. Freshness is exact IR identity, not a
  source assertion: every path to a cyclic consumption must execute the matching typed
  result definition or place initialization during that iteration.
- Loop control: one lowering authority allocates statement-loop labels. `while` and
  `loop` continue at their headers. Checked `for` continues through a distinct shared
  increment block before returning to its header; natural fallthrough uses the same
  tail. Break targets the nearest exit. This corrects the previous admitted array-
  `for` behavior that skipped increment and could repeat forever.
- Ownership boundary: existing `ownership_flow` remains the sole semantic/admission
  authority for pre-loop owners. Any changed enum state in a condition or reachable
  backedge remains rejected. Break-exit ownership joins, moved-target reinitialization,
  partial moves, enum borrowing/storage/projection, loop labels/expressions/break
  values, non-array checked iterators, and general CFG ownership are not admitted.
- Independent proof and evidence: verifier fixed-point controls accept exact fresh
  enum result and mutable-place definitions on a cycle, and reject two consumptions
  after one definition, a predecessor bypassing definition, and an unreset outer
  owner. The exhaustive source/direct-module/IR/LLVM/CLI target, eleven affected
  integration targets, formatting, exact serialized root gate, and local Clang 19.1.5
  native exit 149 pass. The first unconstrained Windows root attempt exhausted linker/
  rustc memory; it is not correctness evidence and the one-job exact retry is green.
- Scaling boundary: this is the hard loop/CFG slice required to prevent selection from
  drifting toward only convenient compile-time values. It uses one class target and
  one authorization record, adds a composed source-to-native gate, and leaves PR #4
  checkpoint strategy and evidence-manifest automation for separate authorization.

## DEC-063 - Centralize intrinsic methods and admit recursive CopyData array queries

- Date: 2026-08-05
- Status: exact green local CORE-067 candidate; immutable commit identity, public
  workflows, pinned LLVM/Clang 22 native exit 167, and acceptance remain pending.
- Decision: one stage-aware classifier is the sole receiver/method/arity/static-
  provenance/context authority for intrinsic method semantic result types, checked
  admission, and trusted lowering. Its dispositions are supported, explicitly
  rejected, or preserved/quarantined. Specialized fixed-array and static-String
  helpers receive normalized query kinds and compute values only; they do not repeat
  method topology. Unsupported methods cannot acquire a fabricated semantic result or
  trusted scalar-zero value.
- Positive contract: exact zero-argument `.len()` and `.is_empty()` are admitted for
  any fixed array whose element resolves through the existing recursive finite
  CopyData classifier and whose count fits Aero `int`. Results are exact compile-time
  `int` and `bool`. Established immutable compile-time String `.len()`, `.is_empty()`,
  `.contains()`, `.starts_with()`, and `.ends_with()`, plus exact zero-argument
  Array/Vec `.iter()` compatibility, retain their accepted behavior.
- Rejection/quarantine: unknown or case-mismatched methods, wrong arity, unsupported
  receiver families, non-static String provenance, other collections, and nested
  scalar-result calls reject deterministically before checked IR. Generic/impl/trait
  syntax remains parser-retained or quarantined, not executable. The legacy unchecked
  IR helpers retain explicitly marked compatibility placeholders, but public checked
  compilation cannot reach them.
- Evidence: red-first proof reproduced semantic fabrication and missing recursive
  queries. Four classifier units, the exhaustive source/semantic/checked-IR/LLVM/CLI
  target, 29-target compatibility ring, formatting, all-target/all-feature correctness
  Clippy, docs, all integration targets, and exact root gate pass at 183/183 library
  tests. The tracked two-file system specimen links with local Clang 19.1.5 and executes
  exit 167; the pinned stable workflow still requires external and machine verification,
  object lowering, link, and the same native result.
- Exclusions: runtime/dynamic String operations, Option/Result/Vec/Map method
  semantics beyond established `.iter()`, heap allocation, callable or iterator ABI,
  general dispatch, generic/trait dispatch, closures, captures, new ownership or
  lifetime behavior, layout changes, new IR opcodes, stable ABI/FFI, accelerators,
  performance, release, stability, and PR merge remain unsupported or separate.
- Scaling boundary: this implements the requested shared supported/rejected/preserved
  classification before topology rules grow further. It follows a hard CFG/runtime
  milestone and includes a composed system gate, while PR checkpoint strategy and
  structured evidence-manifest generation remain separately authorized controls.
