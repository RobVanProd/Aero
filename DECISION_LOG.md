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
- Status: implemented locally for `CORE-011`; acceptance awaits exact review and public CI
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
  independently approved contract. The local candidate centralizes all inventoried
  callers on one crate-private collector, preserves the legacy zero-module key,
  matches the frozen V1 known vector, and makes both focused seven-test suites plus
  the complete repository gate pass. No namespace, visibility, recursive path,
  cycle graph, `CompilerOptions`, or general CLI-status behavior was added.
- Alternatives rejected: a one-line return only in `build`; treating the emitted
  artifact as a warning-success result; hashing root source alone; resolving after a
  cache hit; guessing a path for the library API; or implementing recursive module
  semantics without a frozen namespace/path decision.
- Revisit when: a module-system RFC fixes nested path and namespace semantics, a
  file-aware library API and `CompilerOptions` are designed, or full pipeline
  consolidation can cross more than the bounded source-collection phase.
