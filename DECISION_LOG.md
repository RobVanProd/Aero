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
- Status: implemented for `CORE-007`; full-gate and independent acceptance review
  pending
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

Implementation candidate: the tests-only red checkpoint is integrated as `7346edd`
and the one-line production change as `75dbfba`; public documentation is corrected
at `5dcb70b`. The exact clean candidate passes all 81 focused tests. The complete
repository gate and two non-owner reviews remain required before acceptance.
