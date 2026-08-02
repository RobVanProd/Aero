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
