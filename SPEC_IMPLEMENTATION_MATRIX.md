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
  `inner: Type::Tuple(_)` is not implemented reference or tuple behavior. It currently
  becomes `Ty::Int`, is skipped by checked admission, and can become `ImmInt(0)` in
  raw generation. CORE-029 therefore preregisters rejection at semantics and checked
  admission only, before insertion/generation and with duplicate semantics first.
- This containment cannot change a matrix cell: it adds no tuple/reference value,
  initialization, assignment, representation, mutability, borrowing, ownership,
  lifetime, provenance, layout, ABI, lowering, execution, or backend evidence. Outer
  tuple CORE-028, initialized bindings, non-tuple references, deeper nesting, valid
  IR/LLVM, and every capability class remain unchanged. R-002 stays HIGH/CRITICAL
  and PARTIALLY CONTROLLED.
