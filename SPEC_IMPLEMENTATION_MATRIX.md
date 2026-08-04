# Aero Specification-to-Implementation Matrix

Audit basis: `8f8c7337a4008082fd2a443fcc814b5847b8663f`.

This matrix records stages independently. `Y` means direct evidence for the
listed slice, `P` means partial or known-defective support, `N` means absent,
`?` means not yet verified, and `—` means not applicable. The only feature-level
classifications are `ABSENT`, `DESIGNED`, `PARSED_ONLY`, `PARTIAL`,
`EXPERIMENTAL`, `END_TO_END`, and `STABLE`. No row is `STABLE` during the initial
audit.

Abbreviations: `Res` name resolution; `Ty` type checking; `Own` ownership
checking; `TIR` typed/structured IR; `BE` LLVM or other backend lowering; `Exec`
successful execution; `+/-/D` positive, negative, and diagnostic tests.

## Language features

| Feature | Spec | Lex | Parse | Res | Ty | Own | TIR | BE | Exec | + | - | D | Docs | Class |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| Integers/floats and arithmetic | Y | P | Y | — | P | — | P | P | P | Y | P | P | Y | PARTIAL |
| Booleans/chars | Y | N | N | — | P | — | N | N | N | N | N | N | Y | DESIGNED |
| Bindings and mutability | Y | Y | Y | P | P | P | P | P | P | Y | P | P | Y | PARTIAL |
| Type annotations | Y | Y | Y | P | P | N | P | P | P | Y | P | P | Y | PARTIAL |
| Comparisons/logical/unary ops | Y | Y | P | — | P | — | P | P | ? | Y | P | P | Y | PARTIAL |
| Functions and returns | Y | Y | Y | P | P | P | P | P | P | Y | P | P | Y | PARTIAL |
| Function-call signatures | Y | Y | Y | P | P | N | P | P | P | Y | P | P | Y | PARTIAL |
| If/else | Y | Y | P | P | P | P | P | P | P | Y | P | P | Y | PARTIAL |
| While/for/loop/break/continue | Y | Y | P | P | P | P | P | P | P | Y | P | P | Y | PARTIAL |
| Strings and formatting | Y | P | P | — | P | P | P | P | P | Y | P | P | Y | PARTIAL |
| Fixed arrays | Y | Y | Y | P | P | P | P | P | ? | Y | P | P | Y | PARTIAL |
| Tuples | Y | Y | Y | P | N | N | N | N | N | P | Y | Y | Y | PARSED_ONLY |
| Struct declarations | Y | Y | P | N | N | N | N | N | N | P | P | P | Y | PARSED_ONLY |
| Struct construction | Y | Y | Y | N | N | N | N | N | N | P | Y | Y | Y | PARSED_ONLY |
| Named field access | Y | Y | Y | N | N | N | N | N | N | P | Y | Y | Y | PARSED_ONLY |
| Enums and construction | Y | Y | P | N | P | P | P | P | ? | P | P | P | Y | PARTIAL |
| Pattern matching | Y | Y | P | N | N | N | N | N | N | P | Y | Y | Y | PARSED_ONLY |
| Generics and substitutions | Y | Y | P | P | P | P | N | N | N | P | P | P | Y | PARSED_ONLY |
| Traits, bounds, and impls | Y | Y | P | P | P | P | N | N | N | P | P | P | Y | PARSED_ONLY |
| Moves | Y | — | Y | P | P | P | ? | ? | ? | P | P | P | Y | PARTIAL |
| Shared/mutable references | Y | Y | Y | P | P | P | ? | ? | ? | P | P | P | Y | PARTIAL |
| Closures | P | P | P | N | N | N | P | P | ? | P | N | N | P | PARSED_ONLY |
| Modules/imports/visibility | Y | Y | P | P | N | N | N | N | N | P | P | P | Y | PARSED_ONLY |
| Standard collections | P | Y | P | N | P | P | P | P | ? | P | P | P | P | EXPERIMENTAL |
| C/foreign-function interface | P | ? | ? | ? | ? | ? | ? | ? | ? | ? | ? | ? | P | DESIGNED |

## Compiler, tooling, and ecosystem surfaces

| Surface | Interface | Shared compiler truth | Artifact/result | Failure tests | Integration evidence | Docs | Class |
|---|---:|---:|---:|---:|---:|---:|---|
| Library `compile_program` | Y | P | LLVM text or located parse error | Y | P | P | PARTIAL |
| Compiler options | Y | N | Default path preserved; accepted CORE-020 rejects nondefaults before lexing | Y | Y | P | PARSED_ONLY |
| CLI build/check | Y | N | P; surfaced compile failures nonzero | Y | P | Y | PARTIAL |
| CLI run | Y | N | CPU executes; accepted CORE-018 makes ROCm a temporary regular-file probe followed by status 1/no execution; CUDA status 1 | Y | P | Y | PARTIAL |
| CLI test | Y | N | Semantic analysis only; explicitly reports no execution; failures nonzero | Y | P | Y | PARTIAL |
| Formatter | Y | N | Text trimming | N | N | P | EXPERIMENTAL |
| Diagnostics/source spans | Y | P | Point/one-char ranges | P | P | Y | PARTIAL |
| LSP | Y | N | P | P | P | Y | EXPERIMENTAL |
| Documentation generator | Y | P | Markdown | P | P | Y | EXPERIMENTAL |
| Profiler | Y | N | Timing/trace or located parse error | Y | P | Y | EXPERIMENTAL |
| Project initialization | Y | — | Project files | Y | P | Y | EXPERIMENTAL |
| Module resolver | Y | P | Resolved source | P | P | Y | EXPERIMENTAL |
| Registry | Y | — | Local search and dry-run plans; live transport quarantined | Y | N | Y | EXPERIMENTAL |
| Conformance command | Y | P | 3 cases + 4 deterministic checks | P | P | P | EXPERIMENTAL |
| Package lock/reproducible resolution | P | ? | ? | ? | ? | P | DESIGNED |

## Backend summary

Detailed stage evidence lives in `BACKEND_STATUS.md`.

| Backend/surface | Selectable | IR transform | Object | Link | Real execution | Numerical checks | Performance evidence | Class |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| CPU | Y | Y | P | P | P | P | P | PARTIAL |
| ROCm | Y | Y | P, temporary/unchecked at AUDIT-024 | N | N | N | External llama.cpp only | EXPERIMENTAL |
| CUDA | Y | P | N | N | N | N | N | PARSED_ONLY |
| Graph compilation | Y | Y | — | — | Internal scalar-helper transform only | N | N | EXPERIMENTAL |
| Quantization | Y | Y | — | — | Scalar-double helper transform only | N | N | EXPERIMENTAL |

## Evidence notes

- The lexer cannot return errors and currently converts some invalid input into
  valid tokens, so otherwise present lexical cells remain partial.
- At `bc9a148`, initialized `int`/`i32` and `float`/`f64` binding annotations are
  enforced exactly through active semantics, with positive, negative, diagnostic,
  artifact, and lexical-scope coverage. Uninitialized and non-numeric annotations
  remain unenforced or uncertified. At `8d5d8e7`, active semantic and IR paths also
  enforce monomorphic numeric/void top-level function calls and returns; boolean,
  generic, composite, method, string, and richer closure contracts remain open.
- Many declarations lose visibility, bounds, arguments, or source locations in
  the AST; parser presence alone therefore does not imply faithful parsing.
- The CLI and library declare overlapping compiler modules. Tooling rows cannot
  claim shared compiler truth until that duplication is removed.
- Current conformance determinism checks are useful regression evidence, not
  formal-semantics proof.
- `AUDIT-022` at clean public head `c612f3b` reproduces a compiler package version
  of `0.3.0` while CLI version routes/banner present `1.0.0`. Reviewed public red
  `4b94dbd` binds that mismatch and the overstated three-example/four-repetition
  claims at exactly two preservation passes/five failures. The accepted `CORE-016`
  implementation derives CLI presentation from package metadata and
  classifies those checks and design/history documents without changing capability
  behavior; its focused claim and CLI targets pass 7/7 and exact full gate passes.
  Exact three-review-approved implementation `cc984d0` and record-only closure
  `ea036f2` each pass all eight public checks. R-008 is controlled for this selected
  claim boundary. Language semantics, package version, report schema, conformance
  algorithms, underlying capability classes, and release state remain frozen.
- `AUDIT-023` at clean accepted head `8869eca` classifies the 38 ignored Phase 5
  tests: 22 exact strict lexer/parser-retention candidates and 16 quarantines (14
  semantic plus 2 generic-impl retention gaps). `CORE-017` selects only test/evidence
  classification. Even if active, the 22 remain `PARSED_ONLY` evidence and do not
  change language semantics, capability classes, IR/backend behavior, or stability.
- Public-green preregistration `2c61535` freezes that boundary. Exact three-review-
  approved implementation `8be8c21` has exactly 22 strict lexer/parser-retention
  passes and 16 explicit quarantines with no production change; the exact full gate
  and all eight public checks pass. Exact three-review-approved record-only closure
  `3dd3bb4` also passes all eight public checks. R-012 is partially controlled for
  those 22 accepted `PARSED_ONLY` tests only; the 16 quarantines, 299 dormant tests,
  Cargo overlap, and all semantic/execution rows remain unchanged.
- `AUDIT-024` at clean public head `9ddc571` confirms CPU as the only real process
  execution route. ROCm reaches an unchecked temporary `llc` object and incorrectly
  returns status zero without link/launch; CUDA returns unavailable. The `gpu` alias
  is a tool/environment heuristic, graph output is verified textual internal scalar
  helpers, and quantization is scalar-double helper transformation without real FP8,
  per-channel execution, numerical proof, or device execution. Triple-approved
  tests-only `427fb4c` reproduced the exact public red split. Exact three-review-
  approved implementation `8bde0ff` passes CLI 10/10, claims 7/7, the complete gate,
  and all eight public checks with fail-closed ROCm/CUDA, explicit targets, exact
  non-device scalar-helper telemetry, and the Aero GGUF route disabled. The selected
  boundary is accepted at exact three-review-approved public record-only closure
  `2e0e17f`, which also passes all eight checks. No backend row is promoted and R-007
  remains open.
- `AUDIT-025` at clean accepted head `d0bd54e` confirms that `aero test` performs
  strict parse, direct-module collection, and semantic analysis only while current
  CLI/help/BUILD wording claims sources run and pass. `CORE-019` selects wording and
  exact CLI tests only; all stages, statuses, counts, discovery behavior, and
  capability classes remain frozen. Ignored nondefault `CompilerOptions` remain a
  separate R-006 runner-up.
- Triple-reviewed public tests-only `6728a39` proves that boundary with exact 9/2
  compiler/nightly failures, permitted stable fail-fast cancellation during tests,
  and four green CodeQL checks. Exact three-review-approved implementation `2fe580d`
  is focused 11/11, exact-full-gate green, and all-eight-public-checks green. The
  selected presentation boundary is accepted without promoting any matrix row or
  capability class and without adding test execution, IR, codegen, or runtime.
- Exact three-review-approved corrected record-only closure `63b6629` also passes the
  full gate and all eight public checks. No row/class promotion or execution evidence
  is inferred from closure.
- Exact three-review-approved final-state sync `25dec51` also passes all eight public
  checks. `AUDIT-026` is read-only and cannot promote any row or define
  `CompilerOptions` behavior.
- Public-green `AUDIT-026` preregistration `2c61ff9` supports the completed read-only
  finding: all 62 in-repository library callers used defaults, while every nondefault
  option was ignored across checked compilation. DEC-025 and preregistered `CORE-020`
  select pre-lexing rejection of nondefaults while preserving the facade and exact
  default behavior. At that preregistration checkpoint, the compiler-options row
  remained `PARSED_ONLY` with ignored behavior and no promotion.
- Exact three-review-approved preregistration `fae1374` passes all eight checks.
  Exact tests-only `037f44d` proves the ignored-option boundary publicly at 1/1 while
  all four CodeQL checks pass. The local one-guard candidate is focused 2/2,
  preservation 40/40, and full-gate green. The row remains `PARSED_ONLY`: explicit
  unsupported rejection is claim containment, not implemented option semantics; public
  implementation acceptance was pending at that checkpoint.
- Exact three-review-approved implementation `70cb0ad` passes all eight public checks.
  The accepted boundary preserves default LLVM/diagnostics and rejects nondefaults
  before lexing. The row remains `PARSED_ONLY`: no optimizer, debug-information,
  target-selection, CLI mapping, IR, codegen, or backend behavior is implemented.
- Exact three-review-approved record-only closure `5a8cd06` passes all eight public
  checks in compiler runs `30835593703`/`30835597576`, Rust `30835597620`, CodeQL
  `30835594365`, and aggregate `91759990615`. It changes no row or capability class.
  `AUDIT-027` is preregistered as read-only re-ranking and cannot promote a row.
- Public-green `AUDIT-027` basis `aa3e7a8` completes the read-only comparison. All
  auditors rank R-013 first; DEC-026 and preregistered `CORE-021` select only removal
  of the CPU success phrase for delegated nonzero exits while preserving exact child
  behavior. No compiler/backend row or capability class is changed or promoted.
- Exact three-review-approved tests-only `0873f65`, tree `51ec7d0a`, diff `f75a6360`,
  publicly reproduces the selected delegated-exit false-success boundary in compiler
  runs `30839264536` / `30839272375` and nightly Rust run `30839272429`; stable is
  cancelled during tests by fail-fast. CodeQL `30839264268` and aggregate
  `91772180985` pass. The one-condition production implementation passes focused CLI
  11/11, backend-claim 7/7, and the exact full local gate. Exact tree `0ad98c82`, diff
  `2dbbc395`, received three approvals and was published as `a4327be`; compiler
  `30839860335` / `30839862442`, Rust `30839862423`, CodeQL `30839859840`, and
  aggregate `91774125621` all pass. The selected presentation boundary is accepted
  without changing or promoting any compiler/backend row.
- Corrected record-only closure `b99e445`, tree `8a4c2d77`, diff `5abbf3a7`, passes
  compiler `30840427466` / `30840426655`, stable/nightly Rust `30840428215`, CodeQL
  `30840415565`, and aggregate `91775938704`. `AUDIT-028` is a preregistered
  read-only full-risk ranking and cannot change or promote any matrix row.
- Public-green `AUDIT-028` basis `399e04f` completes the full-risk ranking. R-013 is
  the only universal top-two residual; DEC-027 and preregistered `CORE-022` select
  only fail-closed `aero init` destination-entry preflight before writes. This
  project-tooling boundary does not change or promote a compiler/backend matrix row.
- Accepted `CORE-022` implementation `2a42324` makes final-entry `aero init`
  preflight non-following and fail-closed before writes. Exact tests-only `7cd8aba`
  reproduces Linux compiler 10/1; implementation passes focused/local gates and all
  eight public checks in compiler `30843592298` / `30843592784`, Rust `30843595560`,
  CodeQL `30843589175`, and aggregate `91786468184`. This project-tooling containment
  promotes no language, IR, codegen, CPU, ROCm, or CUDA matrix row.
- Exact record closure `aa29a00`, tree `e740df48`, diff `3eb8264b`, is triple-reviewed
  and passes compiler `30844324249` / `30844328660`, Rust `30844328850`, CodeQL
  `30844325051`, and aggregate `91788926688`. `CORE-022` is closed without changing
  any compiler/backend matrix classification.
- Public-green status synchronization `21153f3` passes compiler `30844798322` /
  `30844802332`, Rust `30844802044`, CodeQL `30844799426`, and aggregate
  `91790481511`. Preregistered read-only `AUDIT-029` ranks the complete residual set
  but cannot change or promote any matrix row.
- `AUDIT-029` completed from all-eight public-green basis `0e5cba1`, tree
  `6ac88db4`. The independent top selections are R-002 Boolean helper contracts,
  R-010 grammar-authority containment, and R-009 parser UTF-16 columns; R-012 is the
  common evidence-only runner-up. Lead reconciliation selects R-002's active one-
  phase semantic inconsistency. Checked IR already maps Boolean helper definitions,
  returns, calls, and storage to LLVM `i1`, but semantics registers only numeric/void
  contracts, accepts invalid Boolean calls/returns, and defaults other declared call
  results to `Int`. Preregistered `CORE-023` adds no matrix promotion: it freezes
  only exact `Ty::Bool` contracts for monomorphic non-entry helpers, with parser,
  AST, IR, verifier, codegen, ABI, generics, composites, coercions, and broader R-002
  closure excluded until separate evidence.
- Accepted `CORE-023` implementation `67ccdf2` closes only the semantic contract gap
  for monomorphic non-entry Boolean helpers. Triple-reviewed tests-only `c3f6e90`
  publicly reproduces the three semantic discrepancies; the one-file implementation
  passes focused/preservation/full gates and all eight checks in compiler
  `30850000615` / `30850005598`, Rust `30850005670`, CodeQL `30850001251`, and
  aggregate `91807553635`. Boolean helper parameters/returns now use exact `Ty::Bool`
  and valid calls infer `Ty::Bool`; checked IR/codegen remain unchanged and retain
  existing LLVM `i1` evidence. This is a PARTIAL function/type-contract improvement,
  not entry/ABI, generic/composite, execution, backend, or stability closure.
- Exact triple-reviewed record closure `0b88530`, tree `71ac4da7`, diff `adba01a1`,
  passes compiler `30850519757` / `30850524194`, stable/nightly Rust `30850524148`,
  CodeQL `30850520457`, and aggregate `91809289681`. No matrix row is promoted:
  `CORE-023` accepts only its non-entry monomorphic Boolean semantic sub-slice.
  `AUDIT-030` is a preregistered read-only ranking of all eleven residuals and cannot
  change implementation or capability classification.
- `AUDIT-030` is complete at public-green authorization `d4e3c75`. All three
  rankings place R-009 parser UTF-16 projection in their top three; two rank it
  first. `CORE-024` preregisters only an LSP coordinate adapter with one synthetic
  UTF-16-unit end range. It changes no grammar, parser, AST, recovery, semantic, IR,
  codegen, ABI, execution, or backend stage, and adds no matrix promotion before
  tests-first and accepted public evidence.
- Triple-reviewed tests-only `ab8508e` reproduces the selected parser-coordinate
  defect as the sole 148/149 failure across both compiler jobs and stable/nightly
  Rust. Exact triple-reviewed one-file implementation `a3d110e`, tree `79ccfca1`,
  diff `74bfbcea`, passes the focused regression, all LSP tests, the full local gate,
  and all eight public checks. Parser diagnostic start coordinates after non-BMP
  prefixes now project from scalar source columns to UTF-16 at the LSP boundary;
  internal locations, lexical diagnostics, the synthetic one-unit end range, and
  every parse/semantic/IR/backend stage remain unchanged. Diagnostics/source spans
  stays PARTIAL and LSP stays EXPERIMENTAL; this is not token/AST span, recovery, or
  end-to-end range evidence.
- Corrected exact record closure `226b7fb`, tree `1337945c`, diff `861b5ec3`, is
  triple-reviewed and all-eight public green in compiler `30854853182` /
  `30854856449`, Rust `30854856190`, CodeQL `30854853829`, and aggregate
  `91823492290`. Diagnostics/source spans remain PARTIAL and LSP remains
  EXPERIMENTAL. Preregistered read-only `AUDIT-031` may re-rank residual risks but
  cannot change a matrix cell, capability class, or implementation.
- Public-green read-only `AUDIT-031` authorization `ba258c6` selects a distinct
  R-002 containment for `CORE-025`: initialized exact outer tuple binding annotations
  currently disappear at semantic and checked-admission boundaries, allowing the
  scalar RHS type to win. The task may add rejection only, after child validation
  and before generation. Tuple remains PARSED_ONLY; no tuple value, layout, ABI,
  lowering, execution, ownership, generic-type, nested-annotation, or matrix
  promotion is authorized before separate accepted evidence.
- Accepted `CORE-025` implementation `1ec8beb`, tree `ac2c8fdd`, supplies that
  bounded evidence. Corrected tests-only `39ccd9c` publicly reproduces exactly 16
  passed/1 failed in compiler and nightly Rust, with only the selected five-boundary
  target red; the two-file implementation passes focused 1/1, binding 17/17, the exact
  full gate, compiler `30857775577` / `30857777431`, stable/nightly Rust
  `30857777314`, CodeQL `30857775231`, and aggregate `91832840108`. Semantic and
  checked-admission guards now reject only initialized exact outer tuple binding
  annotations after child validation and before insertion/generation. Tuple remains
  PARSED_ONLY; no matrix cell or tuple value/layout/ABI/ownership/lowering/execution
  capability is promoted, and uninitialized/nested annotations remain quarantined.
- Corrected exact `CORE-025` record closure `b0fe242`, tree `2a5d233f`, diff
  `98916b4d`, is triple-approved and all-eight public green in compiler
  `30858384541` / `30858387195`, Rust `30858387193`, CodeQL `30858385234`, and
  aggregate `91834740790`. Tuple remains PARSED_ONLY and no matrix cell changes.
  Preregistered read-only `AUDIT-032` may re-rank all eleven residual risks only
  after its own exact gates; it cannot change a matrix cell, capability class, or
  implementation.
- Public-green read-only `AUDIT-032` authorization `b6b1c63` identifies a bounded
  R-005 checked-admission phase-order defect: wrong-arity direct checked-AST calls
  to known admitted scalar top-level helpers reach raw IR and fail only in verifier
  `CallArity`. `CORE-026` preregisters tests-first rejection before generation for
  only nongeneric, non-entry scalar/Void signatures, with existing child, local-
  callable, and Void-use precedence preserved. No matrix cell changes before
  accepted implementation evidence; source semantics, valid lowering, verifier,
  codegen, ABI, backend, and every broader callable/type surface remain unchanged.
- The first `CORE-026` authorization review rejected ambiguous malformed/duplicate
  signature eligibility at P2 before publication. The corrected boundary admits an
  arity guard only for one verifier-valid, unique, non-reserved top-level declaration
  and requires controls preserving current verifier failures for every excluded
  signature. This correction changes no matrix cell or implementation.
- Accepted `CORE-026` implementation `8c2b2ec`, tree `eabd8939`, supplies only that
  bounded fail-before-IR evidence. Corrected tests-only `1538a3e` publicly reproduces
  exactly 6 passed/1 failed with only the selected phase-order target red; the one-
  file implementation passes focused 1/1, checked-IR 7/7, the exact full gate, both
  compiler jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate.
  Eligible known scalar/Void direct checked-AST arity mismatches now reject during
  Admission after child/local/Void precedence and before raw IR. Valid lowering,
  malformed or duplicate signature behavior, source semantics, verifier defense,
  codegen, ABI, and backends remain unchanged. No matrix cell or capability class is
  promoted, and broader R-005 remains PARTIALLY CONTROLLED.
- Corrected exact `CORE-026` record closure `0a940ea`, tree `6ec4c609`, diff
  `4e1db178`, is triple-approved and all-eight public green in compiler
  `30862783787` / `30862786131`, Rust `30862786150`, CodeQL `30862784231`, and
  aggregate `91848258218`. No matrix cell changes. Preregistered read-only
  `AUDIT-033` may re-rank all eleven residual risks only after its own exact gates;
  it cannot change a matrix cell, capability class, or implementation.
- Public-green read-only `AUDIT-033` authorization `544b1ba` selects only R-010
  documentation-authority containment for `CORE-027`: the split grammar and core-
  features tutorial must visibly distinguish the normative v1 design target from
  current compiler capability evidence. Every EBNF production, example, compiler
  behavior, and existing matrix cell remains unchanged. R-010 remains OPEN and no
  capability is promoted before separate accepted evidence.
- Accepted `CORE-027` implementation `b3e7910`, tree `2728bbc6`, supplies only that
  classification boundary. Tests-first `f57cf2e` publicly isolates the one expected
  authority-contract failure; the corrected two-document implementation passes the
  focused and full version-claim contracts, exact full local gate, both compiler
  jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate. Every EBNF
  production and tutorial code example is unchanged, the grammar remains normative
  intended v1 design, and no parser, semantic, IR, verifier, codegen, ABI, backend,
  row, cell, or capability class changes. R-010 remains HIGH/HIGH and OPEN.
- Exact `CORE-027` record closure `d649c2d`, tree `b5ad7ee2`, diff `d4281863`, is
  triple-approved and all-eight public green in compiler `30865772404` /
  `30865775196`, Rust `30865775214`, CodeQL `30865772793`, and aggregate
  `91857289172`. No matrix cell changes. Preregistered read-only `AUDIT-034` may
  re-rank all eleven residual risks only after its own exact gates; it cannot change
  a matrix cell, capability class, or implementation.
- At `6ce85922`, trusted library/build/check/run/test/profile parser paths reject
  malformed root and applicable direct-module sources with located errors. Lexer
  failures remain uncontrolled, and shared compiler truth remains partial.
- At `b988318`, trusted repository source paths use one strict fallible scanner;
  located lexical failures are covered across library/CLI/modules/docs/profile/LSP,
  while the source-compatible recovery API remains restricted to editor indexing,
  tests, benchmarks, and manual compatibility tooling.
- At `8d5d8e7`, declaration collection, exact numeric arity/type checks, conservative
  numeric return checking, void-value rejection, and matching call result types are
  covered by 13 focused tests plus the full gate and dual independent review. This
  does not certify booleans, generics, composites, or all CFG shapes.
- At `bc9a148`, exact initialized numeric annotations and the binding visibility
  needed to enforce them are covered by 18 focused tests, the full gate, and dual
  independent review. This does not add typed local slots, conversions,
  reassignment, definite initialization, or non-numeric annotation support.
- `AUDIT-021` at clean public head `1535ce2` proves the remaining initialized
  binding surface is not merely uncertified: String/bool/custom-name/fixed-array
  type and fixed-array length mismatches pass check/build and publish LLVM because
  semantics and checked IR discard the annotation. Mixed arrays and non-int indexes
  fail only after semantic success. `CORE-015` preserves existing numeric scalar
  enforcement and, outside active semantic generic scopes, selects `bool`, canonical
  `String`, and nonempty flat fixed numeric arrays. It closes four reproduced
  annotation false successes, adds numeric all-element/count/index typing, and verifies optional
  binary-type metadata in checked IR after semantic operand inference remains
  unchanged. Lowercase `string`, contextual/structural annotations, nonnumeric arrays,
  and new generic-scope annotation/array behavior retain pre-task outcomes under green
  controls. No recursive mapping, conversion,
  representation, layout, or execution change is selected.
- At `c000d91`, `%` is specified, lexed, parsed, and numerically typed but absent
  from IR/backend lowering. Integer, float, mixed, and zero-RHS forms pass semantic
  `check` then panic in IR. `CORE-005` deliberately preregisters a temporary
  fail-closed semantic diagnostic rather than inventing remainder semantics.
- At `302211e`, the preregistered `%` boundary is controlled: syntax and precedence
  are preserved, while shared semantics rejects typed operands before IR across
  public, CLI, and direct-module paths. Fourteen focused tests, the complete gate,
  corrected tutorial wording, and two independent reviews support this partial
  classification; no remainder execution behavior is claimed.
- At exact integrated `CORE-009` production candidate `a887931`, named/generic struct declarations and
  StructLiteral parser shape remain visible, but trusted parsed source bodies visit
  field values in source order and reject construction before inference/IR with
  `Struct construction expressions are not supported.` Construction name/field/type
  validation, layout, initialization, ownership, ABI, lowering, and execution remain
  absent. Historical struct code-generator helpers do not make this source path
  executable.
- The accepted `CORE-010` production implementation adds checked logical scalar IR,
  internal invariant verification, exhaustive checked codegen, and qualified LLVM
  22 verification on trusted publication paths. Focused contracts, the complete
  repository gate, three exact-diff reviews, and all required public CI checks pass
  at head `db349ef`. This does not certify unresolved physical integer, aggregate,
  ownership, or backend semantics.
- Accepted public `CORE-013` at `a78dd00` classifies the two
  `performance_benchmark.py` compilation series as
  invalid measurement evidence because they invoked a bare source path, while the
  public and historical Criterion lexer records retain their separate qualifications
  and the one-run external llama.cpp observation remains reference-only.

This file must be tightened as audit items close. A row may become `END_TO_END`
only with source-to-execution evidence and all applicable positive, negative,
diagnostic, documentation, and backend gates. `STABLE` additionally requires a
declared compatibility policy and release-level coverage.

## AUDIT-034 / CORE-028 classification boundary

- Public-green read-only `AUDIT-034` authorization `45783af`, tree `f1baa457`,
  passes both compiler jobs, stable/nightly Rust, all three CodeQL analyses, and the
  aggregate check. Three complete independent rankings and unanimous targeted
  reconciliation select one exact R-002 fail-open declaration form.
- A binding with `value: None` and outer annotation `Type::Tuple(_)` is not an
  implemented tuple feature. At the pre-CORE-028 audit basis, semantics silently
  selected `Ty::Int`, checked admission skipped the statement, and raw generation
  could create integer zero. `CORE-028` therefore selected rejection in semantics
  and checked admission only, before insertion or generation, with existing
  duplicate-name semantics first.
- This containment cannot change a matrix cell: it adds no tuple value, layout,
  assignment, ownership, ABI, lowering, execution, or backend evidence. Initialized
  CORE-025 behavior, nested tuple shapes, other valueless annotations, valid IR/LLVM,
  and every current capability class remain unchanged. R-002 remains HIGH/CRITICAL
  and PARTIALLY CONTROLLED.
- Accepted public `CORE-028` implementation `e051452`, tree `63985b2d`, supplies
  only that rejection boundary after triple-reviewed public red evidence. Focused
  1/1, binding 17/17, the exact full local gate, both compiler jobs, stable/nightly
  Rust, all three CodeQL analyses, and aggregate pass. Exact outer tuple annotations
  on valueless bindings no longer fall back to `Int` at the two trusted boundaries.
  No tuple value/layout/lowering/execution evidence was added, so every matrix row,
  cell, and capability class remains unchanged.
- Exact six-record CORE-028 closure `032d0d0`, tree `443aacdc`, diff `93fce8ae`, is
  triple-approved and all-eight public green in compiler `30872236535` /
  `30872238993`, Rust `30872239003`, CodeQL `30872237025`, and aggregate
  `91876507154`. No matrix cell changes.
- Preregistered read-only `AUDIT-035` may re-rank the same complete eleven-risk set
  only after its separate exact authorization gates. It must exclude every accepted
  slice including CORE-028, cannot inherit AUDIT-034's order, and cannot change a
  matrix row, capability class, source, test, workflow, dependency, or backend.

## AUDIT-035 / CORE-029 classification boundary

- Triple-approved read-only AUDIT-035 authorization `f1cd972`, tree `b9c6270b`,
  passes both compiler jobs, stable/nightly Rust, all three CodeQL analyses, and the
  aggregate check. Three independent complete rankings and unanimous targeted
  reconciliation select one distinct R-002 fail-open annotation shape.
- A valueless binding with outer `Type::Reference(inner, _)` and immediate
  `inner: Type::Tuple(_)` is not implemented reference or tuple behavior. At the
  pre-CORE-029 audit basis it became `Ty::Int`, was skipped by checked admission,
  and could become `ImmInt(0)` in raw generation. CORE-029 therefore selected
  rejection at semantics and checked admission only, before insertion/generation
  and with duplicate semantics first.
- This containment cannot change a matrix cell: it adds no tuple/reference value,
  initialization, assignment, representation, mutability, borrowing, ownership,
  lifetime, provenance, layout, ABI, lowering, execution, or backend evidence. Outer
  tuple CORE-028, initialized bindings, non-tuple references, deeper nesting, valid
  IR/LLVM, and every capability class remain unchanged. R-002 stays HIGH/CRITICAL
  and PARTIALLY CONTROLLED.
- Accepted public `CORE-029` implementation `29bd2e0`, tree `53282149`, supplies
  only that exact non-recursive rejection after triple-reviewed public red evidence.
  Focused 1/1, binding 18/18, formatting, the exact full local gate, both compiler
  jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate pass.
  Valueless immediate reference-to-tuple annotations no longer fall back to `Int`
  at the two trusted boundaries. No tuple/reference value, representation, ownership,
  lowering, execution, or backend evidence was added, so every matrix row, cell, and
  capability class remains unchanged.
- Exact six-record CORE-029 closure `7222b9a`, tree `66084b36`, diff `90bf540c`, is
  triple-approved and all-eight public green in compiler `30876033717` /
  `30876035730`, Rust `30876035761`, CodeQL `30876034500`, and aggregate
  `91887644623`. No matrix cell changes.
- Preregistered read-only `AUDIT-036` may re-rank the same complete eleven-risk set
  only after its separate exact authorization gates. It excludes every accepted
  slice including CORE-029, cannot inherit AUDIT-035's order, and cannot change a
  matrix row, capability class, source, test, workflow, dependency, or backend.
- Corrected read-only `AUDIT-036` authorization `f4ac505`, tree `3cdf89e6`, diff
  `40896f51`, is triple-approved and all-eight public green in compiler
  `30876975678` / `30876977928`, Rust `30876977905`, CodeQL `30876976155`, and
  aggregate `91890402326`. All three complete rankings select exact R-002 valueless
  immediate array-of-tuple fallback over verifier-contained R-005.
- Accepted public CORE-030 implementation `97c0f04`, tree `aa3a9e3f`, diff
  `06a104df`, turns only that one unsupported valueless annotation into semantic and
  checked-admission rejection after triple-reviewed authorization and public
  tests-first red evidence. Focused 1/1, binding 19/19, the exact full local gate,
  both compiler jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate
  pass. Rejection supplies no array/tuple value, default, bounds, layout, mutation,
  ABI, ownership, lowering, execution, or backend evidence. Therefore every matrix
  row, cell, and capability class remains unchanged, and R-002 remains
  HIGH/CRITICAL and PARTIALLY CONTROLLED.
- Exact six-record CORE-030 closure `cd8add28`, tree `8ab06d62`, diff `18ffa30d`,
  is triple-approved and public all-eight green in compiler `30879329940` /
  `30879332975`, Rust `30879332995` attempt 2, CodeQL `30879330627`, and
  aggregate `91897195358`. The initial Rust fixture race passed on focused rerun
  without a file or ref change. No matrix cell changes.
- Preregistered read-only AUDIT-037 may re-rank the complete remaining eleven-risk
  set from that exact clean public head only after its separate authorization gates.
  It excludes all accepted slices through CORE-030, inherits no prior order, and
  cannot change a matrix row, cell, capability class, source, test, workflow,
  dependency, backend, semantics, or claim.
- Triple-approved read-only AUDIT-037 authorization `987188fc`, tree `0b685659`,
  is public all-eight green. Three complete rankings place R-002 first; targeted
  static reconciliation unanimously selects only the exact valueless
  `Array(Array(Tuple))` fallback over the reference-array alternative.
- Preregistered CORE-031 may turn only that unsupported exact two-array-deep
  valueless annotation into semantic and checked-admission rejection after separate
  contract and public tests-first gates. Rejection adds no nested-array/tuple value,
  default, bounds, layout, mutation, ABI, ownership, lowering, execution, or backend
  evidence. Every matrix row, cell, and capability class remains unchanged; R-002
  remains HIGH/CRITICAL and PARTIALLY CONTROLLED.
- Accepted public CORE-031 implementation `4bc7a345`, tree `61361621`, canonical
  diff `349e34ee`, turns only that exact unsupported form into semantic and checked-
  admission rejection after triple-reviewed authorization and public expected-red
  evidence. Focused 1/1, binding 20/20, the exact full local gate, both compiler
  jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate pass.
  Candidate B, initialized and third-plus-depth forms, scalar arrays, generic and
  reference wrappers, raw IR, verifier/codegen, ABI/ownership, valid-output scope,
  and every backend remain unchanged. Rejection supplies no nested-array/tuple
  value, default, bounds, layout, mutation, lowering, execution, or backend evidence;
  therefore every matrix row, cell, and capability class remains unchanged, and
  R-002 remains HIGH/CRITICAL and PARTIALLY CONTROLLED.
- Exact six-record CORE-031 closure `45696091`, tree `480c3504`, canonical diff
  `d682b0f6`, is triple-approved and public all-eight green in compiler
  `30882630407` / `30882632698`, Rust `30882632696`, CodeQL `30882630822`, and
  aggregate `91907149874`. No matrix cell changes.
- Preregistered read-only AUDIT-038 may re-rank the complete remaining eleven-risk
  set only after its separate exact authorization gates. It must exclude every
  accepted slice through CORE-031, inherit neither Candidate B nor any prior order,
  and cannot change a matrix row, cell, capability class, source, test, workflow,
  dependency, backend, semantics, or claim.
- Corrected read-only AUDIT-038 authorization `e4d58e59`, tree `f265d8af`, canonical
  diff `31d09f92`, is triple-approved and public all-eight green. Three complete
  rankings put R-002 first; after an exact-candidate split, final compatibility
  reconciliation unanimously approves only initialized immediate `Array(Tuple)`
  containment. The valueless triple-array candidate remains preserved.
- Preregistered CORE-032 may turn only that unsupported initialized immediate
  array-of-tuple annotation into semantic and checked-admission rejection after
  separate contract and public tests-first gates, in every generic/impl statement
  context those phases already traverse while preserving earlier outer-generic
  rejection. The initial five-acceptance authorization snapshot was rejected before
  publication for omitting that context; the corrected contract freezes eight
  accepts. Rejection adds no tuple/array
  compatibility, value, default, bounds, layout, mutation, ABI, ownership, lowering,
  execution, or backend evidence. Every matrix row, cell, and capability class
  remains unchanged; R-002 remains HIGH/CRITICAL and PARTIALLY CONTROLLED.
- Corrected CORE-032 authorization `449f3536` is triple-approved and public all-eight
  green. Corrected tests-first `35eac8c4` publicly proves exactly eight acceptances
  and only the named 20/21 regression after rejected unpublished `1afe11d3` omitted
  explicit array-literal target coverage.
- Accepted public implementation `30d0d730`, tree `653346ce`, canonical diff
  `01e87768`, adds only exact semantic and checked-admission rejection. Focused 1/1,
  binding 21/21, formatting, two consecutive exact full gates, all three reviews,
  compiler `30886856260` / `30886858878`, Rust `30886858960`, CodeQL
  `30886856518`, and aggregate `91919998289` pass. The first full-gate attempt is
  retained as an unexplained truncated exit-1 result. No tuple/array value,
  compatibility, layout, lowering, execution, backend, matrix-cell, or capability-
  class evidence was added; R-002 remains HIGH/CRITICAL and PARTIALLY CONTROLLED.
- First closure snapshot `7d7fe3d6` passed its exact gate but was rejected
  unpublished because its state record treated that gate as future work and generic
  nonzero wording lost the known exit 1 above. Second snapshot `48f2fd60`, tree
  `86175cc1`, canonical diff `9f0ab102`, resolved those findings and received two
  approvals but was rejected unpublished at P3 by the type reviewer because the
  successful closure gate lacked literal `exit 0`. The twice-corrected records
  preserve both rounds; their fresh exact gate exits 0 with 139/139 library, 149/149
  binary, 7/7 doc, and 21/21 binding tests. Exact closure `9c82cbfc`, tree
  `b2a106ee`, canonical diff `fc672744`, is triple-approved and public all-eight
  green in compiler `30888222316` / `30888225734`, Rust `30888226011`, CodeQL
  `30888222480`, and aggregate `91924197947`. No matrix cell moves.
- Preregistered read-only AUDIT-039 may re-rank only the complete remaining
  R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016 set from exact
  clean public closure `9c82cbfc`. It must exclude all accepted slices through
  CORE-032, inherit no prior candidate/order, and distinguish rejection, simulation,
  annotation, LLVM text, object emission, and hardware execution.
- Exact AUDIT-039 authorization `fa522b2c`, tree `365a536d`, canonical diff
  `cefb797e`, is triple-approved and public all-eight green. All rankings put R-002
  first. Initial candidates split between valueless three-array Candidate T and
  initialized two-array Candidate A; targeted preference favored A two to one, and
  final compatibility reconciliation unanimously approved exact A.
- Preregistered CORE-033 may reject only initialized exact nonrecursive
  `Array(Array(Tuple))` after initializer and existing initialized diagnostics at
  semantic and checked boundaries. Its tests-first red surface is exactly 12
  acceptances after explicit reclassification of both existing A acceptance rows.
  Candidate T, reference-array Candidate B, other deeper/wrapped shapes, and every
  tuple/array value, layout, bounds, ownership, ABI, lowering, execution, backend,
  matrix, risk, and capability state remain unchanged.
- The prepared CORE-033 six-record authorization's fresh exact full gate exits 0
  with 139/139 library, 149/149 binary, 7/7 doc, and 21/21 binding tests. At that
  authorization stage, no test or source change was permitted before three exact
  reviews, unchanged publication, and all eight public checks.
- First authorization snapshot `d0500865`, tree `d2378320`, canonical diff
  `97a15c9f`, passed its local gate but was rejected unpublished by two reviewers
  because one ledger sentence mislabeled Candidate T as Candidate B. The correction
  changes no matrix, capability, risk, or behavior boundary.
- Corrected CORE-033 authorization `66207215`, tree `357c2731`, canonical diff
  `96b5f403`, is triple-approved and public all-eight green. First tests snapshot
  `7608b42c` was rejected unpublished for omitting the initialized three-array-deep
  green control. Corrected tests-only `ac4cb2a5`, tree `852bff0b`, canonical diff
  `4ca50572`, publicly proves exactly 12 acceptances as the sole 21/22 failure in
  compiler `30891243037` / `30891246443` and nightly Rust `30891247469`; CodeQL
  `30891241566` and aggregate `91933672071` pass.
- Accepted implementation `76a6e802`, tree `d8391348`, established PowerShell
  full-index canonical diff `a75b59b2`, adds only the exact semantic and checked-
  admission rejection. Formatting, focused 1/1, binding 22/22, the exact full local
  gate exit 0, three corrected-identity approvals, compiler `30891890629` /
  `30891898590`, Rust `30891897083`, CodeQL `30891892219`, and aggregate
  `91935804190` pass. The initial review request's erroneous plain-diff `c17b1b6a`
  changed no commit or tree.
- Rejection supplies no tuple/nested-array value, compatibility, bounds, layout,
  mutation, ABI, ownership, lowering, execution, or backend evidence. Candidate T,
  reference-array Candidate B, all other deeper/wrapped forms, every matrix row,
  cell, and capability class remain unchanged; R-002 stays HIGH/CRITICAL and
  PARTIALLY CONTROLLED.
- First six-record closure snapshot `fe90f583`, tree `90ac8ae6`, canonical diff
  `89fe6824`, changed only the control records and passed its exact gate with
  139/139 library, 149/149 binary, 7/7 claim, and 22/22 binding tests. It received
  two approvals but was rejected before independent push or branch-head publication
  because stale PROJECT_STATE wording could reopen tests-first and implementation.
  First correction `19f688a`, tree `9d9c642f`, canonical diff `f885588c`, fixed the
  wording, passed the same gate, received three approvals, and is public all-eight
  green in compiler `30893002336` / `30893005706`, Rust `30893006634`, CodeQL
  `30893002479`, and aggregate `91939375982`. Its linear push also made rejected
  parent `fe90f583` publicly reachable as an ancestor, invalidating the stronger
  never-published wording. Final additive correction changes no matrix cell; exact
  gate exits 0 with 139/139 library, 149/149 binary, 7/7 claim, and 22/22 binding
  tests. Exact correction `1ee9c71`, tree `d0819881`, canonical diff `7303da47`,
  received three approvals, was published unchanged, and passes compiler
  `30893527220` / `30893529999`, stable/nightly Rust `30893529992`, all three
  CodeQL analyses in `30893527445`, and aggregate `91941079083`. No matrix cell
  moves.
- Preregistered read-only AUDIT-040 required re-ranking only the complete remaining
  R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016 set from exact
  clean public closure `1ee9c71`. It must exclude all accepted slices through
  CORE-033, inherit no prior candidate/label/order, and distinguish rejection,
  simulation, annotation, LLVM text, object emission, and hardware execution.
- First authorization snapshot `c83ec3a`, tree `bb25e528`, canonical diff
  `c02f71e5`, passed its exact full gate with 139/139 library, 149/149 binary, 7/7
  claim, and 22/22 binding tests. Type/safety and backend/claim approved, but IR/
  codegen rejected at P1 because a late PROJECT_STATE subsection still treated
  accepted CORE-033 closure as future work. It was rejected before publication.
  Corrected authorization `7b9ed83`, tree `8dbe975e`, canonical diff `c4ba110a`,
  passed its fresh exact gate with 139/139 library, 149/149 binary, 7/7 claim, and
  22/22 binding tests, received three exact approvals, and was published unchanged.
  Compiler `30894708169` / `30894713332`, stable/nightly Rust `30894713411`, all
  three CodeQL analyses in `30894708736`, and aggregate `91944883143` pass.
- AUDIT-040 completed read-only. Type/safety selected valueless exact three-array
  tuple containment; IR/codegen selected initialized exact immediate reference-to-
  tuple containment; backend/claim selected immediate nonnegative literal fixed-
  array bounds containment. Targeted comparison preferred reference containment two
  to one, and all three final compatibility reviews approved that exact predicate.
  Literal bounds remains stopped pending separately frozen compile-time-versus-
  runtime policy; the three-array candidate remains a bounded fallback with greater
  topology and count burden. No matrix row, cell, capability class, or risk moved.
- Preregistered CORE-034 may reject only initialized exact nonrecursive
  `Type::Reference(Type::Tuple(_), _)` in semantic analysis and checked admission,
  after initializer validation and all existing initialized tuple-shape diagnostics,
  for both reference mutability flags. Only after the six-record authorization is
  locally green, triple-approved, published unchanged, and public all-eight green
  may one tests-first aggregate reclassify two existing acceptance rows and expose
  exactly 30 false acceptances. Implementation requires separately reviewed public-
  red evidence and remains limited to the semantic analyzer and checked IR admission.
  First authorization snapshot `7d4d7ca`, tree `b633abbb`, canonical diff
  `a901f4dc`, passed its exact full gate with 139/139 library, 149/149 binary, 7/7
  claim, and 22/22 binding tests. IR/codegen and backend/claim approved, but type/
  safety rejected it at P1 because TASK_LEDGER's final status still called the
  completed gate future work. It remained unpublished. The corrected authorization's
  fresh exact full gate exits 0 with 139/139 library, 149/149 binary, 7/7 claim, and
  22/22 binding tests. Rejection adds no reference or tuple value, mutability,
  ownership, lifetime, layout, ABI, coercion, lowering, execution,
  bounds, backend, or stability evidence. Every matrix row and cell remains exactly
  unchanged; R-002 stays HIGH/CRITICAL and PARTIALLY CONTROLLED.
- Corrected authorization `91d2686` is triple-approved and public all-eight green.
  Triple-approved tests-only `296276f` publicly proves exactly 30 false acceptances
  as the sole 22/23 binding failure in compiler `30916807388` / `30916811627` and
  nightly Rust `30916810937`; CodeQL `30916806193` passes. Three public-red reviews
  approved implementation authority.
- Exact two-phase implementation `a1ffeaec`, tree `f0088e65`, canonical diff
  `7a3fdb11`, adds only nonrecursive semantic and checked-admission rejection. It is
  triple-approved and passes the exact full local gate, compiler `30917539648` /
  `30917544307`, stable/nightly Rust `30917537292`, all three CodeQL analyses in
  `30917534448`, and aggregate `92019545168`. No matrix row or cell moves: tuple,
  reference, ownership, layout, ABI, lowering, execution, bounds, and backend cells
  retain their prior classifications.
- The prepared six-record closure's fresh exact full gate exits 0 with 139/139
  library, 149/149 binary, 7/7 claim, and 23/23 binding tests. No matrix cell moves.
- Exact six-record closure `d3811b00`, tree `c01088c4`, canonical diff `2799eb32`,
  is triple-approved and public all-eight green in compiler `30918433816` /
  `30918438945`, stable/nightly Rust `30918439169`, all three CodeQL analyses in
  `30918434204`, and aggregate `92022619964`. CORE-034 is closed; no matrix cell
  moves.
- Preregistered read-only AUDIT-041 must independently re-rank the complete remaining
  R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016 set from exact
  clean public closure `d3811b00`, exclude every accepted slice through CORE-034,
  inherit no prior candidate/label/order, and distinguish rejection, helper
  simulation, annotations, LLVM text, object emission, and hardware execution.
  It cannot move a matrix row or cell; ranking begins only after its separate
  six-record authorization is locally green, triple-approved, published unchanged,
  and public all-eight green.
- The prepared AUDIT-041 authorization's fresh exact full gate exits 0 with 139/139
  library, 149/149 binary, 7/7 claim, and 23/23 binding tests. No matrix cell moves.
- Exact AUDIT-041 authorization `a31342e8`, tree `fbcd78b6`, canonical diff
  `313a1f6b`, is triple-approved and public all-eight green. Three complete rankings
  put R-002 first; initial V/I/R candidates split, targeted comparison prefers R two
  to one, and all final compatibility reviews approve only initialized exact
  nonrecursive positive-count `Reference(Array(Tuple))` containment.
- Accepted CORE-035 authorization preregistered rejection of that exact shape for
  both reference mutability flags at semantic and checked-admission boundaries only
  after child and existing initialized diagnostics. Its tests-first evidence was
  required to expose exactly 34 false
  acceptances and preserve four count-zero observations. Rejection defines no
  reference/array/tuple value, ownership, layout, ABI, bounds, lowering, execution,
  backend, or stability evidence. Every matrix row and cell remains unchanged;
  R-002 stays HIGH/CRITICAL and PARTIALLY CONTROLLED.
- The prepared CORE-035 authorization's fresh exact full gate exits 0 with 139/139
  library, 149/149 binary, 7/7 claim, and 23/23 binding tests. No matrix cell moves.
- Exact authorization `b74b1d29`, tree `3fc2d78f`, canonical diff `64fbd1fe`, is
  triple-approved and public all-eight green. Triple-approved tests-only `f04e80c9`,
  tree `03a9f274`, canonical diff `9e04b6ad`, publicly proves exactly 34 false
  acceptances as the sole 23/24 binding failure in compiler `30922180824` /
  `30922181281` and nightly job `92035312036` in Rust `30922181764`; stable was
  fail-fast cancelled, while CodeQL `30922176056` and aggregate `92035461619` pass.
  Three public-red reviews approved implementation authority.
- Exact implementation `b8fd5a17`, tree `77bd2536`, canonical diff `2f1e9920`, adds
  only nonrecursive semantic and checked-admission rejection. It is triple-approved;
  focused 1/1, binding 24/24, the exact full local gate, compiler `30922853658` /
  `30922859177`, stable/nightly Rust `30922863203`, all three CodeQL analyses in
  `30922853619`, and aggregate `92037794056` pass.
- Rejection supplies no reference/array/tuple value, compatibility, ownership,
  lifetime, bounds, layout, ABI, lowering, execution, or backend evidence. Count
  zero and every deeper/wrapped residual remain unimplemented controls. Therefore
  every matrix row, cell, and capability class remains unchanged; R-002 stays
  HIGH/CRITICAL and PARTIALLY CONTROLLED. The six-record closure's exact full local
  gate passes with 139/139 library, 149/149 binary, 7/7 claim, and 24/24 binding
  tests.
- Exact CORE-035 closure `60ad91f7`, tree `978aa98f`, canonical diff `818a8112`, is
  triple-approved and public all-eight green in compiler `30923835957` /
  `30923837627`, stable/nightly Rust `30923838264`, all three CodeQL analyses in
  `30923834264`, and aggregate `92041128413`. CORE-035 is closed; no matrix row,
  cell, capability, risk, backend, artifact, or claim classification moves.
- Preregistered read-only AUDIT-042 must independently re-rank only the complete
  remaining R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016
  set from exact clean public closure `60ad91f7`. It must exclude all accepted slices
  through CORE-035, inherit no prior candidate, label, or order, and distinguish
  rejection, helper simulation, annotations, LLVM text, object emission, and
  hardware execution. It cannot change a matrix row or cell; ranking begins only
  after its six-record authorization is locally green, triple-approved, published
  unchanged, and public all-eight green.
- The prepared AUDIT-042 authorization's fresh exact full gate exits 0 with 139/139
  library, 149/149 binary, 7/7 claim, and 24/24 binding tests. No matrix cell moves.
- First authorization snapshot `4ce0de0d`, tree `350984b8`, canonical diff
  `347278c3`, passed that gate but was rejected before independent push or branch-
  head publication for stale active-hypothesis and closure-status wording. It remains
  in corrected ancestry; ranking did not begin and no matrix cell moves.
- Corrected AUDIT-042 authorization `2d8a0c54`, tree `45d1c184`, correction canonical
  diff `b36d3d9b`, and cumulative canonical diff `478e947a`, is triple-approved and
  public all-eight green in compiler `30924946683` / `30924950615`, stable/nightly
  Rust `30924951134`, all three CodeQL analyses in `30924945035`, and aggregate
  `92044919183`.
- Three complete rankings selected U/T/B respectively. Targeted comparison selected
  valueless exact nonrecursive `Reference(Array(Tuple))` U two to one; all three
  final compatibility reviews approved only that exact two-phase containment. Direct
  literal bounds B remains stopped pending compile-time-versus-runtime policy;
  valueless exact three-array tuple T remains a bounded fallback. AUDIT-042 was read-
  only and moves no matrix row or cell.
- Preregistered CORE-036 may reject only a valueless exact nonrecursive
  `Type::Reference(Type::Array(Type::Tuple(_), count), ref_flag)` for both flags and
  all counts at semantic and checked-admission boundaries, after existing duplicate/
  tuple-shape diagnostics and before fallback/raw generation. Its tests-first file
  must reclassify all four existing acceptance occurrence blocks/five exact source
  rows, expose exactly 34 false acceptances, and preserve exactly 40 observations.
  Implementation remains two exact guards after separately reviewed public-red
  evidence.
- That proposed rejection supplies no reference, array, tuple, default, mutability,
  ownership, lifetime, layout, ABI, bounds, lowering, execution, or backend support.
  Initialized count-zero behavior remains unchanged. Every matrix row and cell,
  including tuples parsed-only, references/fixed arrays partial, bounds unresolved,
  and CPU/ROCm/CUDA separated, remains exactly unchanged. Authorization must be
  triple-approved, published unchanged, and public all-eight green before tests-first
  work. Its fresh exact full gate exits 0 with 139/139 library, 149/149 binary, 7/7
  claim, and 24/24 binding tests; a verification gate remains before review.
