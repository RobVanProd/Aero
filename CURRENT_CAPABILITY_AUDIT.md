# Aero Current Capability Audit

Audit commit: `8f8c7337a4008082fd2a443fcc814b5847b8663f`

Audit date: 2026-08-02

Branch: `agent/aero-integration`

## Verified progress after the audit commit

- At integration commit `6ce859220634c40c696397a3df178faea51f1912`,
  malformed root and applicable direct-module syntax is fatal in the library and
  build/check/run/test/profile paths. Located negative tests cover status and
  no-artifact behavior, and the full repository gate passes.
- That commit closed the audited parser-to-codegen false-success reproducer only;
  at that point the trusted lexer was still infallible. Typing/IR fallbacks remain,
  and no backend or public performance claim was upgraded.
- At `b9883181414886b6b2775b149599da29faed933e`, trusted repository
  compilation/validation paths also use strict fallible lexing. Unexpected
  characters, invalid/non-finite numbers, and unterminated strings now reject root
  and applicable direct-module inputs before semantics or output. LSP lexical
  diagnostics use UTF-16 positions; recovery lexing remains compatibility/editor
  infrastructure and is not a stability claim.

## Executive finding

Aero is an experimental compiler repository with a passing Rust regression
suite and several useful implemented slices, but it is not currently a
trustworthy 1.0 language implementation. The advertised surface substantially
exceeds the enforced source semantics and end-to-end execution evidence. Most
seriously, invalid source can be changed by lexing, function and type contracts
are ignored, unsupported expressions are assigned convenient scalar types or
lowered to zero, and the compiler lacks a genuinely typed, fallible IR boundary.

This conclusion does not discard the existing work. It classifies the current
implementation so that useful components can be completed behind explicit
correctness gates.

## Baseline and environment

- Upstream `master` was cloned at
  `8f8c7337a4008082fd2a443fcc814b5847b8663f` and the integration branch was
  created before changes.
- Rust was initially absent. After installing stable Rust `1.97.1`, Cargo
  `1.97.1`, rustfmt, and Clippy, `./tools/test.sh` passed.
- Executed tests: 106 library tests, 111 binary tests, and 59 active frontend
  integration tests; 38 `phase5_tests` are ignored. Doc tests also passed.
- The current conformance command reports three cases and four repeatability
  checks. It does not prove the formal semantics.
- The documented root-level `cargo build --release` fails with exit 101 because
  the repository root has no `Cargo.toml`; the compiler manifest is under
  `src/compiler/`.

## Specification and public-surface audit

- Versioning is contradictory: the package is `0.3.0`, while README, formal spec,
  and CLI print `1.0.0`; no language/package version distinction is documented.
- Formal and split grammar documents disagree about keywords, formatted strings,
  struct-field delimiters, rebinding, and lifetime maturity.
- The README flagship snippet uses grouped imports, named arguments, and absent
  `aeronum`/`aeronn` packages that the active repository cannot compile.
- Several shipped examples use legacy top-level statements outside the normative
  module grammar. `examples/scoping.aero` also conflicts with the documented and
  active same-scope rebinding rule.
- CLI help describes CPU/ROCm/CUDA `run`, but ROCm stops at object generation and
  CUDA run is explicitly unimplemented. CLI `test` analyzes test files but does
  not execute them.
- Root collection demo/test files and completion documents are not proof of
  end-to-end collection safety, UTF-8 behavior, or performance.

## Frontend audit

- The lexer cannot return lexical errors. It discards unexpected characters,
  maps malformed or overflowing numerics to zero, and emits tokens for
  unterminated strings. Defined lexical diagnostic variants are unused.
- Tokens carry only a starting line/column and AST nodes carry no source spans.
  LSP diagnostics synthesize one-character ranges; f-string subparsing loses the
  original source position.
- Parsing loses meaningful accepted information: most visibility, several
  generic bounds/arguments, f-string identity, and closure block statements.
  Untyped closure parameters are assigned `i32` in the parser.
- Panic recovery can consume the next valid statement and returns no partial AST.
  The public parser API can panic if its token vector lacks EOF.
- The documented grammar and parser differ in both directions. Booleans/chars,
  many operators, expression control flow, module bodies, richer types and
  patterns are representative documented-but-unparsed forms.
- `aero fmt` is line trimming and newline normalization, not syntax-aware
  formatting, and it overwrites the source directly.
- Many lexer/parser test source files are dormant and stale; the active suite has
  no invalid-lexing, recovery-retention, span, grammar-coverage, or formatter
  preservation coverage.

## Type, ownership, and phase-boundary audit

- The initial audit found that active call inference ignored the callee name,
  function environment, arity, parameter types, and return type, then returned
  `Int`. At `8d5d8e7`, monomorphic numeric/void top-level functions instead use
  collected declarations, exact call checks, and matching numeric/void IR results.
  Noneligible signatures remain permissive and uncertified.
- A dormant call validator confirms the mandate's suspected fallbacks: unknown
  named, array, tuple, reference, and generic parameter types map to `Int`.
- Initialized exact numeric `let` annotations are enforced at `bc9a148`.
  Uninitialized and non-numeric annotations remain outside that slice. Numeric/void
  function return declarations are enforced at `8d5d8e7`; boolean, generic,
  composite, and other declarations remain open. Unknown named types are assumed
  to be structs, and unknown backend type strings lower as LLVM `double`.
- Field access, tuples, struct/enum construction, matches, closures, and unknown
  methods can bypass subtree validation and acquire `Int`. IR lowering replaces
  several of these forms, including borrow/deref, with integer zero.
- Array inference checks only the first element; index types are not constrained;
  backend lowering uses `double` array storage and truncates float indices.
- Move tracking is shallow and skipped in initializers, returns, block tails, and
  nested calls. Borrow state has no lifetime/provenance analysis or scope-based
  release, and mutable references are classified `Copy`.
- Generic calls are not instantiated. Trait checks compare method names rather
  than signatures and are skipped at many call positions. Dormant generic and
  pattern modules use additional fallbacks and are not compiled into the active
  library or binary.
- At `bc9a148`, tested block, branch, loop, and function exits restore both the
  semantic compatibility table and IR binding map, including scalar and callable
  shadowing. Broader analyzer persistence and phase-boundary risks remain open.
- Semantic analysis returns the original AST rather than a typed representation;
  IR generation is infallible and backend-local code invents missing types. No
  LLVM verifier is part of the active library compile pipeline.

## IR and code-generation audit

- Legacy `parse()` prints an error and returns an empty AST. Semantics accepts it,
  IR creates an empty `main`, and codegen can emit an unterminated LLVM function.
  The `build` command writes this text without verification.
- IR registers, loads, stores, and calls are not typed. Scalar slots are emitted
  as `double` while comparisons produce `i1`, so stored/loaded/returned boolean
  programs can generate type-invalid LLVM.
- `CORE-003` makes checked function `if` arms and reachable void epilogues
  terminator-aware. General CFG lowering can still append statements after a
  terminator and other loop/break/continue or unreachable shapes remain
  uncertified; multiple terminators or invalid blocks may still result.
- At `302211e`, integer, float, mixed, zero-RHS, unary, nested, nonnumeric, root,
  and direct-module modulo nodes whose operands type successfully are rejected by
  shared semantics with one stable diagnostic before IR. Parsing and precedence
  remain explicit; no remainder execution semantics are claimed.
- Adjacent read-only probes found separate boundaries: constant integer `/0` panics
  during IR folding; string comparison passes semantics then panics in IR; tuple,
  field, match, and unknown-method expressions can skip subtree validation and
  compile as fabricated scalar zero. They are not bundled into `CORE-005`.
- `AUDIT-011` isolated the tuple family for `CORE-006`: both tuple literals and
  tuple projections are assigned invented `int` semantics and become zero in both
  IR expression paths. The valid specified expression `(7, 9).0` passes trusted
  checking/building but stores zero. Nested tuple nodes can also evade validation
  beneath parent forms whose inference inspects only part or none of the subtree.
  At `1fa67a2`, tuple syntax/types/patterns are retained while tuple value
  expressions fail closed recursively with one stable diagnostic; fields, matches,
  methods, closures, and other composites remain independent unimplemented
  boundaries.
  Exact clean candidate `cbbe049` is accepted after the complete repository gate
  and two independent reviews; constructed AST callers that bypass semantics and
  tuple layout/execution remain explicitly outside this boundary.
- `AUDIT-012` compared the next three phase failures. Every named FieldAccess form
  tested—including a valid struct-literal projection and undeclared/call receivers—
  passes trusted compilation and becomes zero with no field GEP or receiver call.
  All six string comparison operators pass same-type semantic validation then panic
  in IR. Immediate/computed integer zero division panics in host constant folding,
  while variable, unary, float, and mixed zero forms follow different paths. The
  field family is selected for `CORE-007`; string comparison and division policy
  remain separate, explicitly open tasks.
- At exact reviewed `4e10d479`, trusted active semantic preflight recursively rejects
  named field value expressions after first preserving any established receiver
  diagnostic. The complete gate and two independent non-owner reviews pass. Named
  field syntax remains parsed; struct layout, projection execution, assignment,
  method behavior, and direct AST-to-IR callers remain outside this accepted boundary.
- `AUDIT-013` compared the next open failure mechanisms at exact clean `9fc7d0e`.
  String comparisons are blocked on trustworthy operand typing and equality/order
  policy; zero division is blocked on integer/runtime/IEEE policy; MethodCall is
  blocked on a pre-IR capability discriminator that preserves real array `.iter()`.
  Match is one parser-preserved AST family with no active value-preserving route:
  inference invents `Int`, both IR paths return zero without children, and a
  23-case matrix confirms root, nested, module, and closure false successes. Match
  is selected for fail-closed preregistration; no execution semantics are inferred.
- At `c826294`, active semantic preflight visits a reached Match scrutinee and arm
  bodies in the frozen order, then rejects the Match with one stable diagnostic
  before IR. Exact documented `08e7c2c` passed the full gate but was rejected in
  independent review: parser-retained default trait method bodies are not analyzed,
  so Match there succeeds through the public API/check/build and build writes an
  artifact. A structural audit found no second parsed expression-bearing container
  escape. Match tokens, parser AST, arms, and patterns remain available; pattern
  binding/typing/exhaustiveness, evaluation, result unification, enum layout,
  ownership, IR, and backend execution remain outside this rejected candidate.
  Direct semantic bypass can also still reach dormant zero stubs.
- Unimplemented methods, aggregates, references, and ADTs are either changed to
  zero or dropped. Match retains dormant inference/IR stubs; ordinary analyzed
  paths reject it first, but default trait bodies currently bypass that funnel.
  Several IR instruction variants have no codegen arm and
  are silently ignored by the wildcard arm.
- Library/build paths do not invoke an LLVM verifier. CI object/link/runtime
  coverage is limited to four scalar CPU examples.

## Runtime and backend audit

- CPU has a real LLVM-to-object-to-link-to-process path when `llc`/`clang` are
  available. Four small Linux CI programs check exit codes. The current Windows
  audit host lacks those tools, so local execution is environment-blocked.
- ROCm retargets LLVM and attempts an AMDGPU object; it has no link or HIP launch
  path. CUDA run/object/link/launch is absent. GPU auto-detection probes tools or
  environment rather than a verified usable device and may silently select CPU.
- Graph "executable kernels" are ordinary internal scalar-double LLVM helpers;
  backend selection affects names/metadata, not a device ABI, transfers, launch,
  or synchronization.
- Quantization likewise emits backend-named scalar-double helpers. FP8 does not
  perform FP8 representation/rounding, `per_channel` is metadata-only, and INT8
  multiplication/division scaling is algebraically incorrect. Tests assert text
  and counters rather than numerical reference equivalence.
- The tracked llama.cpp ROCm GGUF run is correctly external reference evidence,
  not Aero device execution.

## Benchmark validity finding

The README's tracked 19-case "compilation" series is invalid as a compilation
measurement. `benchmarks/performance_benchmark.py` invokes
`cargo run --release -- <sourcefile>`, but the CLI requires a `build`, `run`, or
other command. It prints `Unknown command` and exits zero, which the harness
counts as a successful compilation. A current-session probe reproduced exit zero
for this path. Those numbers measure Cargo startup/unknown-command handling and
must not support Aero compilation-speed claims.

All five entries in `claim-verification/claims.json` reference existing files,
but none meets the full benchmark protocol. The public and historical compilation
claims share the invalid command. The lexer Criterion run is genuine historical
microbenchmark evidence, but retained output does not justify calling the center
estimate a median and lacks raw samples/hashes/correctness checks. The GGUF entry
is a genuine one-run external llama.cpp observation with zero warmups, truncated
output, no correctness gate, incomplete hashes, and inconsistent top-level versus
artifact commit attribution. The blocked/omitted GPU-claim record is accurate.

No current public Aero runtime, device, graph, or quantization performance claim
passes `BENCHMARK_PROTOCOL.md`. Existing evidence remains preserved and must be
reclassified rather than deleted.

## Tooling and API audit

- `build`, `check`, `graph-opt`, `test`, unknown commands, missing inputs, failed
  output writes, and failing conformance paths commonly return process status
  zero. Automation cannot use the CLI status as a correctness signal.
- The library and binary compile their own module instances. The library ignores
  every `CompilerOptions` field; the CLI never calls `compile_program`, and its
  advertised optimizer objects are often constructed but unused.
- `check` omits module resolution and uses the legacy parser; `test` only
  discovers and analyzes source instead of compiling/executing it; `profile`
  times a divergent subset; LSP diagnostics are syntax-only and symbol features
  use token heuristics rather than semantic scopes/types/modules.
- The module resolver loads one level and has no recursive graph/cycle handling.
  Project initialization creates a manifest that compiler commands do not use.
- Live registry publish sends path/size/hash metadata without package contents
  and accepts any successful HTTP status without a response contract. Install
  joins registry-provided package name/version into the destination without
  containment validation, creating a path-escape risk. Live registry operations
  remain unauthorized during this audit.

## Testing and fuzzing audit

- The default run executes 106 library, 111 binary, and 59 frontend tests, but 78
  names are duplicated by compiling overlapping modules into both library and
  binary targets. The result has at most 198 distinct active test names, not 276
  independent behaviors.
- All 38 `phase5_tests` are ignored. They cover ownership/moves, borrowing,
  generics, traits, and combined integration—the areas with the strongest public
  safety/abstraction claims.
- Another 299 source tests are dormant because their files/modules are not linked
  into Cargo targets; several reference removed APIs and cannot be treated as a
  ready-to-enable suite.
- Eight active snapshots exist, but at least one blesses a binary expression
  whose semantic output still has `ty: None`.
- Criterion benches are not correctness gates and some discard compiler errors.
  CLI `aero test` discovers no repository Aero test programs and succeeds with a
  warning. `error_examples.aero` is not connected to a compile-fail harness.
- There is no active arbitrary-input fuzzing, property testing, differential
  execution, compile-fail artifact check, LLVM verifier suite, Windows CI, or
  ROCm/CUDA hardware execution job.

## Audit completion

All eight requested read-only areas were completed in bounded waves. The audit
supports a correctness-first Milestone 0: close phase-boundary false success,
then function/type contracts and typed IR/CFG invariants, before expanding the
language or backend surface.

## Initial capability conclusion

- `STABLE`: no audited feature.
- `END_TO_END`: not yet assigned pending executable and negative evidence review.
- `PARTIAL`: numeric core, bindings, functions/control flow, strings, arrays,
  basic CPU LLVM path, diagnostics.
- `PARSED_ONLY`: function contracts, annotations, generics, traits, pattern
  matching, modules/visibility, closures, and several structured forms.
- `EXPERIMENTAL`: accelerator interfaces, graph/quantization transforms,
  collections, LSP, formatting, documentation/profiling/project/registry tooling,
  and the conformance command.

See `SPEC_IMPLEMENTATION_MATRIX.md` and `BACKEND_STATUS.md` for stage-level
classification. These labels describe evidence at the audited commit and do not
promise future compatibility.
