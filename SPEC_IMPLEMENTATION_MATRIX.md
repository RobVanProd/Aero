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
