# Aero Current Capability Audit

Audit commit: `8f8c7337a4008082fd2a443fcc814b5847b8663f`

Audit date: 2026-08-02

Branch: `agent/aero-integration`

## Verified progress after the audit commit

- `CORE-080` is locally green for exact founding dotted-import syntax retention and
  fail-closed executable admission. Direct and optional-alias `import a.b [as c];`
  declarations retain their dotted path, a syntax identity distinct from Rust-like
  `use`, and the exact keyword location. Both forms consume one syntax-aware diagnostic
  authority; existing CORE-071 wording remains byte-exact. Semantic preflight, normal
  semantics, and independent checked admission reject executable imports before
  checked IR, while source/file libraries, collected modules, and check/build/run leave
  no artifacts. The focused target passes 13/13 after an exact 8/9 parser red; malformed
  forms, the compatibility ring, all-features, 195-library, integration, doctest,
  formatting, check, correctness-Clippy, docs, diff, and exact root gates pass. Public
  exact-head checks remain pending. No lookup, binding, alias meaning, namespace,
  visibility, graph, cache, backend, runtime, or ABI behavior is implemented.

- `CORE-079` is accepted public at exact commit
  `5b1ec7340db72354542ab325a9f75cad398857c2`, tree
  `930152ff617e104025fc512337b0b31b1c187c08`, and stable patch
  `f89e01b1f9a2e15aa3fb7a45111b7321da8d4977`. One shared classifier joins finite
  ownership states at the headers and exits of `while`, admitted fixed-array `for`,
  and `loop`; semantic analysis and independent checked admission recheck widened
  headers, and the verifier retains its independent cyclic proof. All nine exact-head
  checks pass. Stable/nightly preserve exits 149/223/227 and execute exit 229; pinned
  Windows LLVM/Clang 22.1.8 externally/machine verifies, emits COFF, links through
  Clang/MSVC, and returns 229 through public/manual execution. Broader ownership,
  aggregate, drop/lifetime, ABI, module, runtime, and accelerator claims remain absent.

- `CORE-078` is accepted public at exact commit
  `70f59fd72e96246b2ebefdf1ae53a9b7f3280cfe`, tree
  `b7a2f41877ab812140248ecce10d3157bdab29ac`, and stable patch
  `a85fca8b087a98a89c81cb6c2eb35de67a249f9e`. Its SHA-256-pinned official full
  Windows LLVM/Clang 22.1.8 archive supplies exact `opt`/`llvm-as`/`llc`/`clang`,
  preserves the x86_64 MSVC target/layout, rejects invalid source and IR without an
  artifact, externally and machine verifies, emits COFF, links with Clang/MSVC, and
  returns exact 227 through both public and independent execution. All nine exact-head
  checks pass; stable/nightly Linux preserve exits 149/223/227. The two installer-based
  predecessors remain rejected evidence, and no general Windows, ABI, packaging,
  release, safety, performance, or accelerator claim follows.

- `CORE-077` is accepted public for balanced loop-carried owned-enum reinitialization
  at exact commit `a93d8d38c5f2a2499ce036f659c13cb2ec4fefcb`, tree
  `2efbeed06e0a303aa5c07d3352d7c536fcd92dcd`, and stable patch
  `d64092de918ad990b79d94f6193607783e3acc55`. One shared edge classifier requires
  each direct mutable admitted enum to
  be exactly `Owned` at loop entry and at every reachable `while` condition, `for`
  iterable, fallthrough/`continue` backedge, and `break` exit. Semantic analysis and
  independent checked admission provide phase-specific snapshots to that same rule;
  verifier CFG controls independently reject missing, bypassed, one-path, generic-
  store, wrong-schema, cycle, and exit repairs. `while`, fixed-array `for`, `loop`,
  return/nonjoining paths, nearest nested-loop transfers, every admitted enum schema,
  and every CORE-073 origin are covered. All eight exact-head checks pass in push CI
  `31085620279`, PR CI `31085622212`, Rust `31085622180`, CodeQL `31085620081`, and
  aggregate check `92564358585`; stable/nightly pinned LLVM/Clang 22.1.8 preserve
  exits 149/223 and execute exact exit 227. Loop-
  carried `Moved`/`MaybeMoved`, projections/partial moves, enum storage/borrowing,
  drop/lifetimes, stable ABI, imports, accelerators, release, and safety remain excluded.

- `CORE-076` is accepted public at exact commit
  `aefeb2d81fb5374e7373a4819f3c92f83a95eb35`, tree
  `34e58b2943d6c01efd245753f4b3ca18a338d595`, and stable patch
  `ef7bd0a42de1bda040a4e435fb9c51e0765160b4`. One classifier
  admits identical exact recursive finite CopyData or the separately constrained owned-
  enum result class. One generic checked result place and one verifier CFG proof now
  cover primitives, fixed arrays including zero length, recursive tuples, finite acyclic
  structs, and owned enums without shape-specific result opcodes. The 194-library/200-
  binary complete compiler surface, every integration/doc target, focused corruption
  controls, format/check/Clippy/docs, direct modules, CLI check/build artifact hygiene,
  deterministic LLVM, and the exact root gate pass locally. The tracked composed
  specimen passes stable/nightly LLVM/Clang 22.1.8 native exit 223 while preserving the
  older unit-enum exit 149 gate. All eight exact-head checks pass in push CI
  `31081503050`, PR CI `31081506213`, Rust `31081506169`, CodeQL `31081503119`,
  and aggregate check `92551229284`. Strings, references/results, unit/unary tuples, dynamic
  collections, enum-in-CopyData storage, cyclic/unsupported structs, wider patterns,
  ABI/runtime/drop/lifetime, accelerator, release, and safety semantics remain excluded.

- `CORE-075` is accepted public at exact commit
  `50a3e03d0bdbc0e7deddde747bc19df0621c1257`, tree
  `c31e261a32072f7eca473d940641bbbfef3b6b21`, and stable patch
  `395de4d78694be56b45a310b87df1f98568217eb`. Its all-eight exact-head checks and
  stable/nightly pinned LLVM/Clang 22.1.8 native exit 211 pass. The shared dynamic-path
  classifier derives exact `Moved`/`MaybeMoved`/`Owned` state for direct-owner enum
  Match results; accepted CORE-076 generalizes only the typed result universe and checked
  result-place identity.

- `CORE-074` is accepted public for fresh owned-enum Match results at exact commit
  `b2bd320e6960c2e4f539911b28a251b32b2b9b89`, tree
  `fc330eacc2a014a22a5e4805bcad337ee67565be`, and stable patch
  `ba5e862467387eb2b4043e6c7384d88462832093`. All eight exact-head checks pass;
  stable/nightly pinned LLVM/Clang 22.1.8 externally verify, machine-verify,
  object-lower, link, and execute exact exit 203. CORE-075 locally supersedes only
  the direct-identifier result exclusion.

- `CORE-073` is accepted public for acyclic whole-owner enum reinitialization at
  `ef2eaa380cccf32e21df8938479e30bcd467cdaa`, tree
  `88f3b0c0d542bcce77e2b53de0c3bf737fb6f629`, and stable patch
  `0714282415bb51f11fedb6dada583dcb8d136f6d`.
  One shared assignment authority classifies ordinary replacement and exact
  `Moved`/`MaybeMoved` reinitialization; semantic analysis and checked admission
  consume the same transition and establish `Owned`. Independent checked verification
  reconstructs predecessor consumption and proves exact schema/value identity,
  dominance, and the checked write kill; missing writes, generic stores, wrong schemas,
  and non-dominating values fail closed. Exhaustive source/CLI/module/IR evidence spans
  unit, scalar, char, multi-field, array/struct/matrix payloads; alias/call/Match/
  assignment consumption; constructor/call/distinct-owner origins; exact acyclic joins;
  later Match/call/return; and deterministic LLVM. All 190 library and 196 binary tests
  plus every integration/benchmark target pass, as do formatting, check, correctness
  Clippy, docs, verifier corruption controls, and the exact root gate. All eight
  exact-head public checks pass, while pinned stable/nightly LLVM/Clang 22.1.8
  externally verifies, machine-verifies, object-lowers, links, and executes exact exit
  199. Every loop-contained reinitialization,
  projection/partial move, borrow/storage expansion, destructor/drop/lifetime rule,
  stable ABI, and general CFG fixed point remains unsupported.

- `CORE-072` is accepted public at exact implementation commit
  `4693f11d18135d76b5a7ec16b385563c07272955`, tree
  `42d6262bdd82e9934f47db8a42f103aa18b6448c`, and stable patch ID
  `5104478eec2ca922fa70200720d3a3bb1ed2fc98`. Exact raw
  Unicode scalars and the frozen eight escape forms retain distinct `char` identity
  from token and AST through semantic and logical types, `ImmChar`, independent
  verification, and private `i32` LLVM. A single primitive authority supplies the
  source/type/logical/CopyData/predicate/physical/zero/alignment facts consumed across
  the trusted pipeline. The complete existing recursive CopyData class carries char
  through bindings/replacement, references, calls/results, arrays, tuples, structs,
  owned enums and Match, control flow, direct modules, libraries, and public CLI
  execution. The red target stopped at strict lexing; a prepublication completeness
  audit added a failing Char Match-result control for the old three-scalar table. The
  implemented target passes 9/9, and all 190 library and 196 binary tests plus every integration/benchmark target pass,
  and formatting, checking, correctness Clippy, docs, verifier corruption controls,
  and the exact root gate pass. Official LLVM/Clang 22.1.8 externally verifies,
  machine-verifies, object-lowers, links, and executes the two-file candidate locally
  at exact exit 197. All eight exact-head public checks pass, including independent
  stable/nightly pinned LLVM/Clang 22.1.8 exit-197 lanes. No arithmetic/order/cast/string/printing/pattern,
  stable ABI, ownership/lifetime, accelerator, or broader primitive semantics follow.

- `CORE-071` is accepted public at exact implementation
  `5fc15622188e4e80a319e4c7d6c4bab17a7c8366`, tree
  `ed1f33ede282d01bcd975d83d1e1197424403fef`, and stable patch ID
  `e5b9d98b4f9c1a1d47ddf0dbe227f0feec78dc55`. Rust-like `use` declarations
  retain their parsed path, alias/glob shape, and exact keyword location, but all
  trusted semantic routes and independent checked admission reject them with one
  deterministic source-located unsupported diagnostic before checked IR. Source/file
  libraries, direct modules, and `check`/`build`/`run` fail without requested/native
  artifacts. The exact implementation passes all eight public checks and pinned
  stable/nightly LLVM/Clang 22.1.8 native exit 193. No import, namespace, alias, glob,
  visibility, resolver, recursive-module, cache, ABI, backend, or runtime semantics
  are added.

- `CORE-070` is accepted public at exact implementation
  `365c28a3e4fdd306ec4c1a4837545ddbe3dac6a3`, tree
  `2e1146cf0c4f7468de0c8fa0dde85a13cdd79a21`, and stable patch ID
  `1263a11601e3cb7f9f776e4e154f3de158feaa6d`. Public
  `compile_file(path, options)` supplies exact root-file diagnostics and the accepted
  root-relative direct-module context while sharing one semantic, checked-IR,
  in-process verification, and checked-codegen sequence with `compile_program`.
  Module-free LLVM remains byte-identical, both accepted module layouts and multiple
  ordered root modules pass, unsupported options reject before I/O, and every tested
  root/module failure returns without artifacts. The new contract is 5/5, its focused
  compatibility ring is 45/45, and all-target/all-feature tests, formatting, checking,
  correctness Clippy, docs, and the exact 187-library/193-binary root gate pass. All
  eight public checks and stable/nightly pinned LLVM/Clang 22.1.8 native exit 193 pass
  on the exact head. This does not add imports, use/pub,
  namespaces, recursion, caching, external verification, output writes, or CLI
  convergence.

- `ARCH-002` is accepted public at exact implementation
  `aca3fe21ece4a7f90de0b41b5e336c15ac589505`, tree
  `3c5466e8d6821b8443ecba919bde2ad568923355`, and stable patch ID
  `cec753bde549b9ea1fc4a3aa7e820d754f7d8798`. Its shared normalized annotation
  topology is behavior-neutral: all supported, explicitly rejected, and preserved/
  quarantined results, diagnostics, checked IR, and LLVM remain unchanged. All eight
  public checks pass, and stable/nightly LLVM/Clang 22.1.8 preserve exact native exit
  193. No matrix cell moved.

- `CORE-069` is accepted public at exact implementation
  `99ea287843bc0c1262045d31a60f18b03fa0558f`, tree
  `175b44d7ea2e10615553d4cd062ad13fd1e2e6e0`, and stable patch ID
  `143a4cf9669e2c4168ba899d7edebeea7e1cd297`. Exact positional variants with two or
  more fields are admitted when every field belongs to the existing recursive finite
  CopyData class. Construction, exhaustive identifier-bound Match, ordered checked-IR
  fields, independent verification, private LLVM, transport, reassignment, and
  composed control flow retain one shared schema authority. All eight public checks
  pass. Stable and nightly LLVM/Clang 22.1.8 externally verify, machine-verify,
  object-lower, link, and execute the tracked direct-module program with exact exit
  193. Named-field/generic variants, broader patterns, enum storage/borrowing/
  projection, partial moves, stable ABI, and accelerator claims remain excluded.

- `CORE-068` is accepted public at exact implementation
  `55b61c31fc6dd822097daa5d4f371d04ec0d6264`, tree
  `81c2be5d1ee6abdd7382c8674f68f553613efd6f`, and stable patch ID
  `155bbfa5310e1289fccb82c339108d8a44bdbfca`. One shared function-call classifier supplies exact
  semantic results and checked admission/lowering for already admitted nongeneric
  signatures. Missing or unsupported contracts, legacy unknown annotation shapes,
  wrong arguments, and `Void` value use reject deterministically instead of becoming
  `Int` or an undefined LLVM call. Classifier units, the exhaustive topology target,
  the affected compatibility ring, and the exact root gate pass at 185/185 library
  tests. The tracked direct-module system specimen links with local Clang 19.1.5 and
  executes exact exit 181. All eight candidate-head checks pass. Pinned stable
  LLVM/Clang 22.1.8 rejects the invalid control, externally and machine-verifies,
  object-lowers, links, and executes exit 181; nightly repeats external verification
  and exit 181. No generic/trait/closure call, conversion, overload, reference-result,
  callable ABI, lifetime, layout, stability, safety, or performance claim follows.

- `CORE-067` is accepted public at exact implementation
  `e7525bf039339909c8f4f5cc68262fdf498079e0`, tree
  `a41eb54122cd1b358ddd3d5c590d738bce98ae29`, and stable patch ID
  `ddcbeaf010903474568bb7f79a79457d7b955d25`. A single
  stage-aware intrinsic-method classifier now supplies semantic result types, checked
  admission, and trusted lowering; unsupported receiver/method/arity/provenance
  products reject through one diagnostic rather than semantic fallback types. Exact
  recursive CopyData fixed-array `.len()` and `.is_empty()` are admitted as static
  `int`/`bool`, while established static String queries and Array/Vec `.iter()` remain
  compatible. Four leaf/shared classifier units, the exhaustive target, 29 affected
  integration targets, and the exact root gate pass at 183/183 library tests. The
  direct-module program links with local Clang 19.1.5 and executes exit 167. All eight
  candidate-head checks pass; stable LLVM/Clang 22.1.8 supplies external/machine/
  object/link/native exit-167 evidence and nightly repeats exit 167. No general
  dispatch, collection, String runtime, ABI, ownership, safety, or stability claim
  follows.

- `CORE-066` is accepted public at exact implementation
  `e40804ea86888b38548fd5bf42926be2be7eb5ed`, tree
  `6cea8bbf63aa7aafb43fbb25152dd860f6684aae`, and stable patch ID
  `7c4e6ac77db90dc7c83048922382903958c09632`. Its exhaustive class target admits
  fresh constructor/call-result enum owners across checked `while`, fixed-array `for`,
  and `loop` statements and keeps every changed pre-loop enum state rejected at
  conditions/backedges. Red-first evidence found array-`for` continue skipping its
  increment; centralized loop labels and a shared `for` tail now route it through an
  explicit `for_continue_` block. Verifier controls accept fresh exact result/place
  definitions on cycles and reject bypass, double consumption, or unreset outer
  owners. Eleven affected targets, formatting, the serialized exact root gate,
  deterministic two-file checked LLVM, and local Visual Studio Clang 19.1.5 exit 149
  pass. All eight public checks pass. Pinned stable LLVM/Clang 22.1.8 rejects the
  invalid control, externally verifies, machine-verifies, object-lowers, links, and
  executes exact exit 149; nightly repeats exit 149. This does not establish general
  loop ownership.
- `CORE-065` is accepted public at exact implementation
  `f4daeea6d7b032e686b4c7d184fe80ef38076665`, tree
  `7cd4ec6da2d9ce44f63741222a5b128396358bfe`, and stable patch ID
  `708c1a6cab096f89e76577212a241554225897a2`, for exact acyclic conditional
  ownership joins over the non-Copy enum class accepted by CORE-063/064.
  One shared classifier gives sibling `if` arms the same entry state, excludes
  definitely returning arms, joins reachable fallthrough as `Owned`, `Moved`, or
  `MaybeMoved`, and rejects loop-carried changes without claiming a fixed point.
  Semantic analysis and checked admission both consume that classifier. Independent
  checked-IR dataflow tracks exact enum result/place owners across predecessor unions,
  calls, returns, Match dispatch, initialization, replacement, and cycles. The focused
  target, corruption controls, affected compatibility ring, formatting, all-target/
  all-feature checking, correctness Clippy, docs, exact root gate, 182/182 library
  tests, and 188/188 binary tests pass. All eight public checks pass. Stable job
  `92454648190` uses LLVM/Clang 22.1.8 for the known-invalid verifier control, external
  and machine verification, object lowering, explicit private linking, and native exit
  137; nightly job `92454648318` repeats exit 137.
- `CORE-064` is accepted public at exact implementation
  `79aed71371e192a07218d437e882a863653b6826`, tree
  `ac80c49aca3fb875c44d132f930567e95d81f698`, and stable patch ID
  `1bb2c9c19f6d427122f83bffc59d3f18f0a5b3e4`. One
  shared owned-place classifier admits either recursive finite CopyData or an exact
  CORE-063 enum schema for direct mutable whole-owner replacement. Inferred and exact
  mutable enum locals accept fresh constructors, exact enum-returning calls, and
  distinct-local moves; self-replacement and unsupported targets fail closed before
  trusted LLVM. Generalized checked identities, independent schema/place/value verifier
  controls, and private typed enum loads/stores cover the target. The focused exhaustive
  target, formatting, all-target/all-feature checking, correctness Clippy, docs, exact
  root gate, and complete Rust test surface pass at 180/180 library and 186/186 binary
  tests. All eight public checks pass. Stable job `92376666972` installs LLVM/Clang
  22.1.8, proves the known-invalid control, externally and machine verifies, object-
  lowers, explicitly links the private executable, and observes native exit 131;
  nightly job `92376666842` repeats exit 131.
- `CORE-063` is accepted public at exact implementation
  `2a5c3c58192dc65116c436d6ae76da5829eeba52`, tree
  `8a5cef6b14214e76349a41f6997d5fa19595858f`, and stable patch ID
  `276af069807b6f59c233a2f281c1b0d0b8c899b8`, with verified native-link repair head
  `bebd0b6a87108219497187a5952688c95c397158`. It extends the accepted
  owned unit/unary-scalar enum class to unary recursive `CopyData` payloads by consuming
  the accepted registry classifier, exact checked schemas, independent recursive
  verifier controls, and private typed LLVM lanes. The exhaustive arrays/tuples/structs,
  Match, transport, module, corruption, and artifact-hygiene target passes with the
  exact root gate, formatting, all-target/all-feature checking, correctness Clippy,
  docs, 179/179 library tests, and 185/185 binary tests. All eight public checks pass.
  Stable job `92363420145` uses LLVM/Clang 22.1.8 for the known-invalid verifier
  control, external and machine verification, object lowering, the explicit private
  non-PIE link, and exact native exit 113; nightly job `92363420286` repeats exit 113.
- `CORE-062` is accepted public at exact implementation
  `e62fd7470d8cb929d57d0c063815d7a99005d768`, tree
  `d2aff21a54c42d1ce649ef6668d50a4908315738`, and stable patch ID
  `458feb5ebc1355d83793084009e5ea7895a22129`. A single registry-backed classifier
  resolves parsed annotations and semantic types across scalars, fixed arrays,
  arity-at-least-two tuples, and finite
  acyclic unique nongeneric nonempty named structs. Semantics, checked IR, Copy-place
  ownership operations, exact internal function transport, independent verification,
  and private LLVM consume the recursive schema. The exhaustive product, verifier
  corruptions, direct module, no-artifact controls, local native exit 109, 178/178
  library tests, 184/184 binary tests, every integration/claim target, Phase 5 controls,
  docs, formatting, all-target checking, correctness Clippy, and exact root gate pass.
  All eight public checks pass; stable job `92344809072` uses LLVM/Clang 22.1.8 for the
  known-invalid verifier control, external and machine verification, object/link gates,
  and exact native exit 109.
- `CORE-061` is accepted public at exact implementation
  `de6fc0d5c503d2dcb03944d58312a130bac1ba05`, tree
  `9ad23f5ad5cff17d3b69fdef31b9a4c7289ade42`, and patch ID
  `e358319e7402f345ca414cc57bb18c0414b81cd4`. All eight checks and the pinned
  LLVM/Clang 22 native exit-83 lane pass with 175/175 library and 181/181 binary tests.
  It admits direct whole-owner reassignment over its frozen Copy-data class and keeps
  closures parsed-only/fail-closed before checked IR without a callable or fallback
  `i32` layout.
- `CORE-060` is accepted public at exact implementation
  `7c7a47a471460dfe2276ea63cc4964fa59ad54be`, tree
  `e9863de79a69766114020060a138c94357005351`, and stable patch ID
  `ec2c33060e33ca6e52894fa1a18daf5b5d9c6ba7`. All eight checks pass; stable job
  `92301482760` uses LLVM/Clang 22.1.8 for external verification, machine verification,
  object lowering, linking, and exact native exit 59 with 174/174 library and 180/180
  binary tests. The accepted class is exclusive whole-place mutable Copy-data
  references, not projected origins/writes, general lifetimes, stable ABI/FFI, or a
  memory-safety guarantee.
- `CORE-059` is accepted public at exact implementation
  `5a78eb5d670045277532cc3cdc9a6144b1449895`, tree
  `03fbdd58e836532dc8a4f95a0bb3c0402b1e5f1c`, and stable patch ID
  `62a23bef479f22d3d9da22fc4bf753c7610c3e77`. All eight checks pass; stable job
  `92291545518` uses LLVM/Clang 22.1.8 for external verification, machine
  verification, object lowering, linking, and exact native exit 37 with 173/173
  library and 179/179 binary tests. The accepted class is immutable reference
  transport over existing Copy-data places, not mutable aggregate borrowing, general
  lifetimes, stable ABI/FFI, or a memory-safety guarantee.
- `CORE-056` is accepted public at exact implementation
  `e3ff1658039f8b9e20f18981c3d6198a07e79e92`, tree
  `4efca0a523ae60d0d3020f925e0567f430dad9dd`, and stable patch ID
  `77377ea77150931b709898d2fdf2bbcd9713c1c1`. All eight public checks pass;
  stable Linux job `92259593558` uses LLVM/Clang 22.1.8 and executes exact native
  exit 251 with 167/167 library and 173/173 binary tests. The accepted class is one
  direct call-scoped mutable scalar owner loan into a sole mutable-reference parameter,
  not stored-alias transport, reborrowing, general lifetimes, stable ABI, or a safety
  guarantee.
- `CORE-055` is accepted public at exact implementation
  `1f6ea726ad87f079592d136cb374ff6481d4acec`, tree
  `a3dd566a80b8555c6dcf417a0528fb13d75a2380`, and stable patch ID
  `2c9f030dd64d4ec86835a1f9ed87322a96f3fcc7`. All eight public checks pass;
  stable Linux job `92251942540` uses LLVM/Clang 22.1.8 and executes exact native
  exit 239 with 166/166 library and 172/172 binary tests. The accepted class is one
  non-escaping local mutable scalar alias, not general borrow checking or a safety
  guarantee.
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
- `CORE-011` tests-only red evidence is public at `9c31820`; three independent
  reviewers approved exact diff `badb9d0e8d6059927d949994b39f617fe2f404a8`
  and tree `540a187db87aff5ec0b2964b0c140c6caf9402a4`. The accepted implementation
  centralizes all inventoried file-backed routes on one crate-private
  direct-source collector, rejects source-only and nested declarations, and makes
  module discovery precede a deterministic module-aware cache key. Both focused
  seven-test suites and the complete repository gate pass. Three independent
  reviewers approved exact implementation diff
  `60fe607413ebc03e9aa5d6296d9067d8cc95d89d` and tree
  `7c57c082e9d5f68afd5c6a4769d9d531a0116642`; every public check passes at
  accepted head `a711dd5f3802095a4ecbe2dea3d45003675e7459`.
- `CORE-012` is accepted at `6780a23cd8b63df124477c7db1190d61dd25f3b8`:
  every live registry route now fails closed before credentials, I/O, transport, or
  writes, while local search and network-free dry-run plans remain available. The
  exact full gate and all public checks pass; the documentation closure is public at
  `b7bb42958e78fb97ea0d991fa3f4cdb40bbcce2f`.

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
- `AUDIT-020` reproduced that the former documented root-level
  `cargo build --release` command exited 101 because the repository root has no
  `Cargo.toml`; accepted public `CORE-014` at `c56b1d5` instead selects the compiler
  manifest under `src/compiler/` explicitly and passes the exact generated-project
  path in stable Linux CI.

## Specification and public-surface audit

- Versioning is contradictory: the package is `0.3.0`, while README, formal spec,
  and CLI print `1.0.0`; no language/package version distinction is documented.
- Formal and split grammar documents disagree about keywords, formatted strings,
  struct-field delimiters, rebinding, and lifetime maturity.
- The public `CORE-014` red checkpoint proves the former README flagship used
  grouped imports, named arguments, and absent `aeronum`/`aeronn` packages that the
  active repository cannot compile. Accepted public `c56b1d5` replaces it with the
  exact existing generated CPU project. All eight checks pass; stable Linux CI
  resolves LLVM 22 tooling and executes build/init/check/run with status zero and
  exactly one anchored `Output: Hello, Aero!` line. Windows commands are statically
  contract-tested but are not claimed as Windows end-to-end execution evidence.
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
  `AUDIT-021` at `1535ce2` confirms every other initialized binding annotation is
  discarded by semantics and checked IR: String/bool/custom-name/array-element/
  array-count mismatches pass check/build and publish LLVM for the inferred value.
  Uninitialized reads already fail in semantics. Numeric/void function return
  declarations are enforced at `8d5d8e7`; generic/composite function contracts and
  reassignment remain open. Unknown named types are assumed to be structs, and
  unknown backend type strings lower as LLVM `double`.
- At the initial audit, field access, tuples, struct/enum construction, matches,
  closures, and unknown methods could bypass subtree validation and acquire `Int`; IR
  lowering replaced several forms with integer zero. Subsequent bounded slices closed
  several named classes. The current CORE-061 closure amendment specifically removes
  both `Closure => Int` inference stubs, rejects before checked IR, and removes the
  callable/unknown-type-to-`i32` lowerer. Other independently listed historical classes
  retain their own current records.
- Array semantics checks only the first element and discards index type. Checked IR
  now rejects non-int indexes and the internal verifier rejects mixed logical stores,
  so reproduced cases fail without artifacts but after semantic success. Backend
  lowering still uses `double` array storage; bounds/layout remain uncertified.
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
- At `a12f38e`, a dedicated syntax-only block/statement traversal also funnels every
  expression root in parser-retained default trait method bodies into the existing
  preflight. It preserves statement/child order and required signatures without
  activating name, parameter, return, type, trait, ownership, or pattern analysis.
  The corrective red suite captured public and CLI artifact false successes first;
  all 15 Match tests and 81 prior focused boundary tests now pass independently for
  owner and lead. Exact clean documented `b74d91a` passes the complete gate and two
  fresh non-owner reviews: one exhaustive structural review and 44-route public
  matrix, plus an independent 75-route/225-outcome public/check/build matrix. No
  trusted false success, panic, unwind, or negative artifact remains in this accepted
  boundary.
- Unimplemented methods, aggregates, references, and ADTs are either changed to
  zero or dropped. Match retains dormant inference/IR stubs, but corrected trusted
  parsed-source paths now reject it first. Several IR instruction variants have no
  codegen arm and
  are silently ignored by the wildcard arm.
- `AUDIT-014` compared the next open families at exact clean `a61172a`. StructLiteral
  has no active value-preserving source path, receives an invented named type without
  declaration/field validation, and becomes scalar zero in both IR paths without
  children; 19/24 public routes falsely succeeded and root/module builds wrote zero/
  drop artifacts. EnumVariant is also non-executable but intersects Option/Result
  sugar and diagnostics. Borrow mutates shallow ownership state, Deref retains
  reference diagnostics, MethodCall must preserve typed zero-argument Array/Vec
  `.iter()`, string comparison needs operand/operator policy, and division needs
  integer/runtime/IEEE policy. StructLiteral alone is selected for `CORE-009`.
- At exact integrated candidate `a887931`, trusted parsed-source preflight visits
  StructLiteral field values in source order and then rejects construction with
  `Struct construction expressions are not supported.` The 164-test focused matrix
  passes independently for owner and lead. Parser/declaration visibility remains;
  struct name/field/type validation, layout, initialization, ownership, ABI, IR,
  backend emission, and execution remain absent. The complete documented gate passes
  at exact `3410f1f`; coordinated control corrections, a fresh complete gate, and
  two independent approvals with no P0-P3 findings pass at exact reviewed `daa024d`.
- `AUDIT-016` re-ranked the open compiler-integrity families after StructLiteral.
  String comparison and constant integer `1 / 0` can unwind in IR generation;
  comparison bindings can successfully emit `i1` results into `double` slots;
  untyped function/signature reconstruction and the codegen wildcard can emit or
  silently omit invalid instructions. Library, CLI build/run, profiler, and
  conformance use duplicated infallible IR APIs. `CORE-010` is preregistered to add
  checked logical scalar admission, internal IR verification, and final LLVM module
  verification. Its generic boundary rejects unadmitted MethodCall/EnumVariant/
  Deref fallbacks without implementing their language semantics; those remain later
  language-specific implementation slices.
  This audit finding does not itself improve a capability.
- The accepted `CORE-010` production implementation adds logical scalar/place/
  function metadata, mandatory internal IR verification, fallible checked IR and
  codegen APIs, and LLVM 22 verification of final transformed/retargeted modules
  before trusted artifact publication. It makes the focused red contracts and the
  complete repository gate green. Three exact-diff reviews and all required public
  CI checks pass at head `db349ef`; this promotes only the selected checked scalar
  IR/publication boundary and does not promote unresolved language/backend rows.
- At the original audit basis, library/build paths did not invoke an LLVM verifier.
  Current CI object/link/runtime coverage remains limited to four pre-existing
  scalar exit-code examples plus the generated-project status/output path accepted
  by `CORE-014`.

## Runtime and backend audit

- CPU has a real LLVM-to-object-to-link-to-process path when `llc`/`clang` are
  available. Linux CI covers four small scalar programs by exit code plus the
  generated `Hello, Aero!` project by status and anchored output. The current
  Windows audit host lacks those tools, so local execution is environment-blocked.
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
other command. At the `AUDIT-019` reproduction basis it printed `Unknown command`
and exited zero, which the harness counted as a successful compilation. The
Accepted public `CORE-013` at `a78dd00` makes that bare-source invocation fail closed with status
`2`; it does not retroactively validate or repair the timings. Those numbers measure
Cargo startup/unknown-command handling and must not support Aero compilation-speed
claims.

All six public entries in `claim-verification/claims.json` reference existing
files, but none meets the full benchmark protocol. The two compilation claims share
the invalid command and are classified `invalid_measurement`. The current and split
historical lexer Criterion records retain their separate qualifications; retained
output does not justify strengthening their statistics and lacks raw
samples/hashes/correctness checks. The GGUF entry is a genuine one-run external
llama.cpp observation with zero warmups, truncated output, no correctness gate,
incomplete hashes, and inconsistent top-level versus artifact commit attribution.
The blocked/omitted GPU-claim record is accurate.

The split historical lexer Criterion run is genuine historical microbenchmark
evidence; that preservation does not rehabilitate either Python compilation series.

No current public Aero runtime, device, graph, or quantization performance claim
passes `BENCHMARK_PROTOCOL.md`. Existing evidence remains preserved; accepted
`CORE-013` classifies the two invalid Python compilation series without deleting or
upgrading any artifact.

## Tooling and API audit

- Before `CORE-013`, `build`, `check`, `graph-opt`, `test`, unknown commands, missing
  inputs, failed output writes, and failing conformance paths commonly returned
  process status zero. Accepted `CORE-013` now gives automation a typed `0/1/2`
  correctness signal for CLI-owned outcomes; delegated CPU program statuses remain contextual
  arbitrary pass-through values.
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

### Post-CORE-010 module-boundary recheck (`AUDIT-017`)

- At the `AUDIT-017` / accepted-`CORE-010` basis, `check` and `profile` strictly
  loaded direct modules, while build caught a missing-module error, continued
  compilation, exited zero, and wrote the requested LLVM artifact for the same
  source.
- At that basis, final-LLVM cache lookup preceded lexing/parsing/module discovery
  and used only root source plus target/GPU configuration. Module changes or
  deletion did not participate in cache identity, so a verified cache hit could
  bypass current module state on the reusable optimizer path.
- Before `CORE-011`, build, check/test, profile, and docs repeated direct module
  load/strict lex/parse logic. The source-only public library API lacked a file
  context and silently accepted `ModDecl`. The accepted closure below centralizes
  this direct source set and fails closed; it does not certify the still-absent
  namespace, `use`, `pub`, recursive graph, or cycle-analysis implementation.
- No-argument, unknown-command, and several missing-input paths still return status
  zero, but they remain a separate command-result task because the selected module
  boundary is specifically about dependency admission, cache identity, and compiler
  artifact publication.
- Accepted closure: `CORE-011` now routes build/run, check, discovered test,
  profile, and docs through the shared collector; source-only `compile_program`
  rejects `mod`. Missing, malformed, and nested module sources stop before later
  phases or publication. Exact source bytes and stable relative candidates enter
  the V1 module-bearing cache identity before lookup, while the no-module key is
  unchanged. This remains direct flattened source collection only; namespaces,
  `use`, `pub`, recursive graphs, cycles, and general CLI status are still absent.

### Post-CORE-011 risk re-ranking (`AUDIT-018`)

- Audit basis: clean published project-control head
  `8598a4c343f5592880bde66cbd99e78083d2a236`; accepted compiler behavior remains
  `a711dd5f3802095a4ecbe2dea3d45003675e7459`. Upstream `master` remains the original
  `8f8c7337a4008082fd2a443fcc814b5847b8663f`.
- Registry: live search, publish, and install remain directly callable. Publish
  inventories only path/length/SHA-256 metadata, sends no package bytes, discards the
  response body, and labels any successful HTTP result accepted. Install joins
  registry-controlled resolved name/version into the selected destination and writes
  it without component validation or containment. CLI auth lookup currently occurs
  before publish/install dry-run branching and for offline search.
- Command/benchmark: unknown commands, malformed `build`/`check` usage, missing
  build/check inputs, and the benchmark's bare source-path invocation all reproduced
  status zero. The bare source path prints `Unknown command`, proving the tracked
  compilation timing measures successful non-compilation. This remains R-013/R-015
  and is ranked immediately after the registry quarantine.
- Arrays: `[1, 2.5]` passes semantics and checked admission, then is rejected by the
  mandatory in-process verifier for an Int/Float store mismatch; `[1, 2][1.5]` is
  rejected by checked admission. Both `check` and `build` return nonzero and publish
  no requested LLVM artifact. R-011 therefore remains a phase-order/type-contract
  defect, but no fresh false-success artifact outranks the active registry boundary.
- Ownership: public safety claims still exceed shallow move tracking, lifetime
  provenance is absent, and mutable references are classified as Copy. A credible
  closure requires a frozen ownership model plus CFG/provenance work across more
  than the permitted bounded phases; changing one Copy predicate would imply safety
  it does not provide. R-004 remains critical and explicitly stopped, not waived.
- Selection: `CORE-012` was selected to quarantine every HTTP-backed registry entry with one
  stable fail-closed guard before credential, filesystem, or transport activity.
  Offline index search and non-network publish/install previews remain available.
  This does not design or validate a registry protocol and cannot re-enable live
  behavior.
- Public red evidence: `57c4ec7` contains only test-scoped direct controls plus the
  focused CLI matrix. Three independent reviewers approved exact diff
  `4058775145e68aa9a5512853c04b0dde04730464` and tree
  `227254ef8177d8e15b69c42bd1e2d94c1442879a`. Direct evidence was 7 pass / 5
  intentional failures; CLI evidence was 0 pass / 6 intentional failures.
- Accepted implementation: every live function and CLI live branch now hits
  the frozen guard before credentials or side effects. Local search and CLI dry-runs
  bypass auth and use only local helpers. Direct registry targets pass 12/12 each;
  the CLI matrix/help pass 7/7; the complete repository gate is green. Three
  independent reviewers approved exact diff
  `05e55496f6664713192b2dbf94eca785abe2931d` and tree
  `85ed76ab0141409796e167704e4100dd4d15c26f` with no P0-P3 findings. Both compiler-
  test workflows, Rust stable/nightly, every CodeQL language analysis, and aggregate
  CodeQL pass at accepted public head
  `6780a23cd8b63df124477c7db1190d61dd25f3b8`.

### Post-CORE-012 CLI and benchmark revalidation (`AUDIT-019`)

- Historical audit basis: clean public documentation head
  `b7bb42958e78fb97ea0d991fa3f4cdb40bbcce2f`; accepted production behavior at that
  point was exact `6780a23cd8b63df124477c7db1190d61dd25f3b8`. Accepted public
  `CORE-013` at `a78dd004aa37c39212711027b777698118d9dc02` supersedes the
  status/claim findings below.
- An initial argument-dropping process batch was invalid and discarded. The corrected
  pre-implementation explicit-argument probe showed status zero for no command,
  unknown command, bare
  benchmark source path, malformed and missing-input build/run/check/fmt/doc/profile/
  graph-opt/quantize routes, registry no/unknown subcommand, and malformed
  conformance. Static inspection found the same fallthrough for failed output writes
  and ignored extra operands in check/fmt/test/lsp.
- `performance_benchmark.py` sends the bare source path and would have accepted the
  former zero status; accepted `CORE-013` now returns `2`. The shell harness
  declares its compile/run work simulated. No benchmark was run. The tracked Python
  compilation numbers remain invalid measurements and their raw evidence is
  preserved under that classification.
- R-004 remains stopped on unfrozen multi-phase ownership semantics; reproduced
  R-011 arrays fail closed before output. Accepted `CORE-013` now
  provides a typed CLI-owned `0` success / `1` operational failure / `2` invocation
  failure contract plus evidence-preserving quarantine of the affected compilation
  claims.
  Delegated CPU-program exits remain arbitrary pass-through values; write rollback
  remains non-atomic and out of scope. Benchmark code, command maturity, and compiler
  phases remain frozen.

## Testing and fuzzing audit

- At the original audit basis the default run executed 106 library, 111 binary,
  and 59 frontend tests, with 78 overlapping library/binary names. At clean public
  head `1535ce2`, Cargo lists 139 library and 148 binary unit tests with 105 shared
  names. `cargo test --tests -- --list` reports 557 target entries and 437 distinct
  displayed names; neither count proves independent behavior coverage.
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

### Post-CORE-014 binding-contract recheck (`AUDIT-021`)

- Five initialized annotation mismatches—String from int, bool from int, an unknown
  named type from int, fixed float-array value under an int-array annotation, and a
  two-element value under a three-element annotation—each pass CLI check/build and
  create the requested LLVM artifact. Exact String, comparison-produced bool, and
  homogeneous integer-array controls pass.
- Mixed numeric arrays and float indexes return nonzero and create no artifact, but
  build traces record semantic success before an internal-verifier or checked-IR
  error. Direct source inspection confirms semantics infers only the first array
  element and ignores the index result; checked IR ignores binding annotations.
- Direct source inspection also finds two checked-boundary parity defects relevant
  to exact equality: checked IR trusts optional caller-supplied `Binary.ty` ahead of
  operand inference, and it maps lowercase `string` to `Ty::String` while active
  semantics maps that spelling to the distinct named `Ty::Struct("string")`.
- Uninitialized reads fail in semantics. Borrow/deref sources fail in checked IR,
  while mutable references remain classified Copy and lifetime provenance remains
  absent. This keeps R-004 stopped rather than pretending a one-predicate edit would
  establish ownership safety.
- `CORE-015` preserves existing numeric scalar enforcement and, outside active
  semantic generic scopes, adds a closed binding-local rule for `bool`, canonical
  `String`, and nonempty one-dimensional fixed arrays over the four numeric spellings.
  It closes four of five reproduced false successes, adds all-element numeric-array
  inference/count/integer indexes in the same scope, and verifies binary type
  metadata. Lowercase `string`, custom/contextual/structural annotations, nonnumeric
  arrays, and all new generic-scope annotation/array behavior must retain pre-task
  outcomes under required green controls. No global recursive mapping, conversion, new type,
  assignment, ownership, aggregate lowering, or backend claim is selected.
- The approved tests-only red target proves that frozen boundary with 8 passing
  preservation groups and exactly 8 intended failing contract groups. Three
  independent reviewers approved exact diff
  `e158ad61282617a63dade4976a7c23fe53aa0af8` and tree
  `db2ac2959f9815fab5d4b649e563b59c83459dfe`; it is public at `b203ea4`. Both
  compiler-test jobs and Rust nightly reproduce the same intended target failure,
  stable is matrix-cancelled, and all CodeQL checks pass.
- The public two-file production candidate now makes the focused matrix pass 16/16.
  Its test delta also adds implementation-review regression controls for numeric-array
  child ordering, single-pass deep nesting, nested index traversal, stub-only
  method/closure/format/custom-enum boundaries, and unsupported-child precedence. Several controls would reject the
  public red implementation, but they live inside its already-failing semantic group
  and therefore do not change the published 8/8 group outcome. One green-side
  public-library assertion was also corrected to the established
  `Semantic Analysis Error:` prefix. The exact repository gate passes formatting,
  correctness Clippy, 139 library tests, 148 binary tests, all active integration
  targets, and doc tests; the 38 established Phase 5 ignores remain unchanged. Three
  independent reviewers approved exact implementation diff
  `3a909f5813def06d4f7cfb27f8650908410ac724` and tree
  `3effac84a84d56f43abcf99c65161c3da7753d6e` with no P0-P3 findings. Public commit
  `3f0578d69926e15a81c4d8fa6105c99c982cbe02` passes both compiler-test jobs,
  stable/nightly Rust, all three CodeQL analyses, and aggregate CodeQL. Three fresh
  reviewers then approved exact closure-record diff
  `a8e4059e71991c9d7a274234f91dd225bea61c01` and tree
  `19fea4153397958656b57adac6b70556d4a997c9`; public closure commit
  `5d7aae0f5626813249b6de983a229dbbb1e4fef8` also passes all eight checks.
  `CORE-015` is accepted at that public closure head without expanding the explicit
  exclusions or any backend capability claim.

### Post-CORE-015 public-claims recheck (`AUDIT-022`)

- The accepted final-state sync is public at clean head `c612f3b`; draft PR #4 is
  mergeable and all eight checks pass. Upstream `master` remains `8f8c733`.
- Cargo metadata reports package `compiler 0.3.0`, while standalone `-v` and
  `--version` print `Aero compiler version 1.0.0` and the no-command help banner
  prints `Aero Programming Language Compiler v1.0.0`. The unadvertised bare
  `version` word is already an unknown command with status two.
- `aero conformance` reports three example cases and four repeatability checks. The
  four named checks rerun tokenization, parsing, checked IR, and lowering to compare
  deterministic output; they are regression evidence, not mechanized formal
  semantics. The current console, help, BUILD, README, and consolidated language
  document overstate that evidence.
- README still lists generics/trait bounds/where clauses and a borrow checker as the
  current language surface. The type-system and ownership documents plus Tutorial 3
  state compile-time memory-safety enforcement that the audit disproves: lifetimes
  and provenance are absent and mutable references remain classified Copy.
- `CLAUDE.md` labels Phase 5, borrow-checker enforcement, generics, and traits
  complete. Tutorial 1 calls the four repetitions mechanized checks and the command
  a formal suite; its next-step list also presents ownership as an active memory-
  safety feature. Tutorial 2 calls the following ownership material an active
  memory-safety feature rather than design-only. These current-facing surfaces are
  part of R-008; Tutorial 4 already carries an explicit implementation boundary.
- Claim-heavy collection/string demo and task summaries have no visible historical
  status notice. Struct and enum records already carry such notices. `todo.md` still
  presents completed phases and version `1.0.0` as current project status.
- An explicit run of all 38 ignored `phase5_tests` passes 36 and fails 2. One failure
  relies on recovery parsing dropping unsupported assignment before semantics; the
  other expects accepted struct/method/borrow behavior outside frozen support. The
  passing set therefore cannot be activated blindly as current capability evidence.
- R-004 remains stopped on multi-phase ownership/provenance decisions. Residual R-002
  custom/contextual annotations require a separately frozen nominal-name/generic-
  substitution contract. Remaining R-011 aggregate execution requires typed IR,
  bounds, layout, and backend work; R-012 needs test-by-test recovery/stub
  classification. R-006, R-009, and R-010 are broader than this bounded workflow;
  R-007 lacks device evidence; R-016 is medium/medium. R-008 is the highest bounded
  active public false claim.
  `CORE-016` selects manifest-derived CLI presentation and visible evidence-based
  documentation classification. No package version, report schema, semantics,
  backend, benchmark, registry, release, or master behavior is selected.
- Reviewed public red commit `4b94dbd` binds this boundary with exactly two
  preservation passes and five intended failures. The subsequent implementation
  reports compiler package `0.3.0` through both version flags and the no-command
  banner, labels the unchanged three cases/four repetitions as deterministic
  regression evidence, and visibly classifies unsupported current/design/historical
  claims. Focused claim and CLI targets each pass 7/7 and exact `./tools/test.sh`
  passes including doc tests. These results became accepted implementation evidence
  at `cc984d0`; by themselves they do not elevate any capability class.
- Exact three-review-approved implementation `cc984d0` is now public and passes both
  compiler-test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate
  CodeQL. The preceding candidate results are therefore implementation evidence, but
  exact three-review-approved record-only closure `ea036f2` is also public with all
  eight checks green. R-008 is controlled for this selected claim boundary. No
  capability class is elevated, and actual ownership/type/backend gaps remain open.

### Post-CORE-016 ignored-test recheck (`AUDIT-023`)

- Clean accepted public head `8869eca` is mergeable with all eight checks green;
  upstream `master` remains `8f8c733`.
- The explicit Phase 5 ignored target reproduces 38 run / 36 pass / 2 fail. Its four
  lexer, twenty parser, and fourteen semantic tests all use recovery lexing; parser
  and semantic tests also use a compatibility parser that can convert failure to an
  empty AST.
- Only 22 tests are conservative activation candidates: four exact strict-token and
  eighteen exact strict parser-retention contracts. Two generic-impl tests remain
  quarantined because target arguments and impl bounds are skipped/discarded. All 14
  semantic tests remain quarantined because broad outcomes mix five genuine shallow
  negatives, positive smoke, unrelated unsupported-construct false positives, and
  two recovery/unsupported failures.
- R-004 remains the highest conceptual risk but stops on unfrozen multi-phase
  lifetime/provenance semantics. R-012 is the highest bounded action and may move only
  to partially controlled under `CORE-017`. No syntax test can elevate a capability
  class or establish ownership, borrow checking, generic/trait enforcement, or
  execution.
- Public-green preregistration `2c61535` binds the conservative 22/16 split. Exact
  three-review-approved implementation `8be8c21` passes exactly 22 strict syntax-
  retention tests with 16 explicit quarantines and 38 total listed entries. Current
  CLAUDE/framework/roadmap lines classify this as parsed-only evidence, and the exact
  full gate passes. Exact three-review-approved record-only closure `3dd3bb4` is also
  public with all eight checks green. R-012 is partially controlled for this selected
  evidence-classification boundary; Cargo overlap and 299 dormant tests remain, and
  no semantic or execution capability class changes.

### Post-CORE-017 backend truth recheck (`AUDIT-024`)

- The audit basis is clean public head `9ddc571`, tree `20ab4e6`, with all eight
  checks green. The root also reran the seven-test CLI status contract and the graph
  and quantization unit filters (3/3 and 5/5 in each duplicated lib/bin target); all
  passed before any edit.
- CPU `run` has a real externally verified host object/link/process path and passes
  the child status through. ROCm `run` requires LLVM verification and invokes `llc`
  for a temporary AMDGPU object, but it neither checks that the object exists nor
  links, launches, synchronizes, or executes anything; it then returns status zero.
  CUDA `run` correctly returns operational status `1` and has no object/link/launch
  path.
- The `gpu` alias is an environment/tool-presence heuristic, not usable-device
  detection. It can silently choose CPU and does not probe CUDA capability.
- Graph compilation externally verifies textual LLVM transformation into ordinary
  internal scalar-`double` helpers. Backend names and existing `executable*` report
  fields do not establish a device ABI or execution. Quantization likewise emits
  scalar-`double` helpers using default or sample-derived scale; it has no FP8
  representation/rounding, executed per-channel behavior, numerical proof, or device
  execution.
- Current README, BUILD, tutorial, CLI help/reporting, quantization notes, and the
  enabled Aero ROCm GGUF example exceed that evidence. All 27 declared immutable
  claim artifacts exist, and `claim-verification/` already classifies the real GGUF
  run as external llama.cpp reference evidence and Aero GPU claims as blocked.
- All three independent read-only reviewers classify the ROCm zero-status path as a
  P1 false success and recommend a tests-first fail-closed correction. `CORE-018`
  freezes operational status `1` for object-only ROCm `run`, a regular-file emission
  postcondition, explicit target selection, and stage-accurate current claims while
  preserving algorithms and compatibility field names. R-007 remains OPEN because
  no Aero accelerator execution or correctness evidence exists.
- The exact tests-only tree `4a65ecf7` was independently approved and published as
  `427fb4c`. Both compiler jobs and stable/nightly failed on the prescribed new
  contracts while all four CodeQL checks passed. The local implementation candidate
  now passes CLI 10/10 and claims 7/7: CPU is unchanged; ROCm checks only regular-file
  emission then returns status 1 without link/launch; CUDA returns status 1; ambiguous
  `gpu` fails before source access; graph/quant output states non-device scalar-helper
  scope; and the nonexistent Aero GGUF route is disabled while external references
  remain exact. Exact implementation tree `d10567be` received three independent
  approvals and was published as `8bde0ff`; the complete local gate and all eight
  public checks pass. Exact record-only diff `3d0a17f7` and tree `83c9676f` then
  received three independent approvals with no P0-P3 findings and were published as
  closure `2e0e17f`. Its two compiler jobs, stable/nightly Rust, all three CodeQL
  analyses, and aggregate CodeQL pass. The selected false-success/current-claim
  boundary is accepted, but R-007 remains open because no accelerator execution or
  correctness evidence was produced.

### Post-CORE-018 clean-head risk re-ranking (`AUDIT-025`)

- The accepted final-state sync is public at clean head `d0bd54e`, tree `21e72079`;
  draft PR #4 is open/mergeable, all eight checks pass, and upstream `master` remains
  `8f8c733`.
- `aero test` discovers direct `*_test.aero`/`*_tests.aero` files and performs strict
  parsing, direct-module collection, and semantic analysis only. It does not admit
  checked IR, generate code, execute a process, or compare runtime results, but its
  comment, progress output, success summary, help, BUILD guide, and an existing CLI
  contract say `run`, `Running`, or `passed`. Accepted DEC-016 already classifies the
  command as a semantic checker rather than an execution runner.
- Two independent auditors rank that direct user-facing false assurance as the next
  bounded P1; the IR/codegen auditor instead ranks ignored public nondefault
  `CompilerOptions` first. The lead selects `aero test` claim containment because it
  is a documented CLI surface with direct user reach and requires presentation/tests
  only. Fail-closed nondefault `CompilerOptions` is the bounded runner-up.
- R-002 and R-004 remain more severe conceptually but stop on unresolved nominal/
  generic and ownership/provenance semantics across multiple phases. R-005 public
  unchecked API retirement needs a major-boundary compatibility policy; R-007 needs
  hardware; R-009/R-010 and broad R-006 convergence are architectural; R-011 needs
  bounds/layout/execution semantics; R-012 remains a per-slice classification backlog;
  and R-016 needs a toolchain policy.
- `CORE-019` therefore freezes wording-only CLI truth. Discovery, parsing, module
  collection, semantic analysis, counts, and statuses remain unchanged; no test
  execution, checked IR, codegen, runtime, language semantic, or capability work is
  selected.

### CORE-019 public red checkpoint and accepted implementation

- Triple-reviewed tests-only commit `6728a39`, tree `5337c877`, reproduces the exact
  public boundary: CI runs `30828281313` and `30828277960` fail only the two frozen
  CLI contracts at 9/2; Rust run `30828281681` has the same 9/2 nightly failure and
  a permitted fail-fast cancellation during stable tests; CodeQL run `30828277876`
  and aggregate check `91735622062` provide all four green CodeQL checks.
- Exact three-review-approved implementation `2fe580d`, tree `1e530e65`, changes
  only `src/compiler/src/main.rs` and `BUILD.md` presentation plus authorized records.
  Focused CLI is 11/11, exact `./tools/test.sh` passes, and all eight public checks
  pass in compiler runs `30829084150`/`30829086467`, Rust `30829088650`, CodeQL
  `30829082758`, and aggregate `91738325685`. The selected claim boundary is
  accepted. No checked IR, codegen, process execution, discovery, diagnostic, count,
  status, language-semantic, or backend capability is added or promoted.
- Exact three-review-approved corrected record-only closure `63b6629`, tree
  `2e886850`, passes exact `./tools/test.sh` and all eight public checks in compiler
  runs `30829963152`/`30829970545`, Rust `30829968789`, CodeQL `30829962982`, and
  aggregate `91741344282`. `CORE-019` is complete at that selected boundary; broader
  R-013 command behavior and executable-test design remain open.

### Post-CORE-019 clean-head audit contract (`AUDIT-026`)

- Accepted public final-state sync `25dec51`, tree `46828e7d`, passes both compiler
  jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate CodeQL. PR #4
  is open/draft/mergeable and upstream `master` remains `8f8c733`.
- The next task is read-only re-ranking. It must reproduce the public
  `CompilerOptions { optimize, debug_info, target }` facade being ignored by
  `compile_program`, compare reach/severity/phase count/compatibility ambiguity with
  remaining risks, and recommend one bounded next action or stop. It does not define
  option semantics, authorize code/tests, or change any capability class.

### Completed `AUDIT-026` findings and `CORE-020` selection

- Public preregistration `2c61ff9`, tree `ff20cf43`, passed compiler runs
  `30831057824`/`30831063857`, Rust `30831066619`, CodeQL `30831055856`, and
  aggregate `91744957183`; all eight checks are green.
- At audited head `25dec51`, `CompilerOptions` publicly exposed `optimize`,
  `debug_info`, and `target`, derived `Debug`, `Clone`, and `Default`, and was
  described as compiler options for benchmarking. `compile_program` accepted the
  value as `_options` but did not read it. Its checked parse-to-codegen path was
  therefore identical for every field value.
- At audited head `25dec51`, the repository had 62 direct calls outside the
  definition: 28 across five benchmarks and 34 across thirteen test files. Every call
  constructed `CompilerOptions::default()`; no in-repository nondefault construction
  existed. No CLI route consumed this public type, and CLI CPU/ROCm/CUDA
  `BuildTarget`/`BuildConfig` orchestration was a separate private surface.
- Static tracing at audited head `25dec51` proved `_options` had no read before or
  within the checked pipeline;
  existing binding (16), checked-IR (6), fatal-parse (11), and module (7) targets
  remained 40/40 green. Two attempted temporary external probes exceeded the audit's
  read-only/external-artifact stop boundary: one reported dynamic results and one was
  interrupted. Both are excluded from accepted findings. Their named executables no
  longer exist; two inert PDB files remain in the user temp directory and are not
  repository or publication artifacts.
- All three independent type/safety, IR/codegen, and backend/claim auditors rank this
  as the best bounded next action under R-006. External nondefault consumers cannot
  be inventoried, so returning an error is a behavior compatibility change, although
  public names, layout, signature, construction, and default behavior remain intact.
- Lead-owned DEC-025 selects `CORE-020`: exactly the default tuple
  `(false, false, String::new())` remains supported. Any true Boolean option or
  nonempty target returns one stable unsupported-options error before lexing. This
  contains false assurance without defining optimization, debug information, target
  selection, CLI mapping, IR, codegen, or backend semantics. No capability row or
  class is promoted by the audit or preregistration.
- Exact three-review-approved closure/preregistration `fae1374`, tree `8c807c17`,
  passes all eight public checks in compiler runs `30833300163`/`30833300408`, Rust
  `30833301841`, CodeQL `30833296979`, and aggregate `91752384364`.
- Exact three-review-approved tests-only `037f44d`, tree `edd8d33e`, reproduces the
  public red boundary: compiler runs `30833844930`/`30833845633` and nightly in Rust
  run `30833845526` fail only the frozen target at 1/1; stable is cancelled during
  tests by permitted matrix fail-fast. CodeQL `30833844647` and aggregate
  `91754222422` provide all four green security checks.
- Before publication, the local implementation candidate added one guard at
  `compile_program` before the
  first lexer call and changes no other compiler phase. The new contract is 2/2,
  binding/checked-IR/fatal-parse/module preservation is 40/40, and exact
  `./tools/test.sh` passes. Public API/default values and byte-exact default LLVM and
  parse diagnostics are preserved. No optimizer, debug, target, CLI, IR, codegen, or
  backend behavior was added; public green acceptance remained pending at that
  checkpoint.
- Exact three-review-approved implementation `70cb0ad`, tree `7c8b2ce1`, diff
  `33e5883e`, passes both compiler runs `30834445685`/`30834446600`, stable/nightly
  Rust run `30834446605`, all three CodeQL analyses in `30834443841`, and aggregate
  `91756251121`. `CORE-020` is accepted at the selected ignored-option boundary.
  Compiler options remain `PARSED_ONLY`: unsupported rejection is enforced, but no
  optimization, debug-information, target, CLI, IR, codegen, or backend semantics are
  implemented. Broad R-006 convergence remains open.
- Exact three-review-approved record-only closure `5a8cd06`, tree `df4a04a`, diff
  `85ef52a4`, passes compiler runs `30835593703`/`30835597576`, stable/nightly Rust
  run `30835597620`, all three analyses in CodeQL run `30835594365`, and aggregate
  `91759990615`. It adds no compiler behavior and makes no capability promotion.

### Next clean-head audit contract (`AUDIT-027`)

- Basis: the exact commit that publishes this six-record contract, but only after its
  local full gate, three independent approvals, and all eight public checks pass. Its
  only delta from accepted closure `5a8cd06` is current control-record state.
- Observed state: R-002 and R-004 retain high raw safety impact but require unresolved
  type/ownership decisions; R-005 needs an unchecked-API policy; R-006 still includes
  duplicated orchestration and undefined option meanings; R-007 needs real hardware;
  R-009/R-010 are architectural; R-011 needs aggregate bounds/layout/execution;
  R-012 is a per-slice ignored-test backlog; R-013 retains delegated-exit, rollback,
  executable-test, command-maturity, and helper-architecture boundaries; and R-016
  needs a toolchain policy.
- Method: use repository source, tracked evidence, existing tests, and public check
  records only. Compare active reproducibility, reach, severity, frozen semantics,
  phase count, compatibility ambiguity, and testability. Three independent auditors
  must rank the same candidates and report evidence plus remaining uncertainty.
- Output: recommend one bounded tests-first task or an explicit stop. The audit may
  update only the six current control records after findings are reconciled; it may
  not define language/toolchain/hardware semantics, create probes or artifacts, edit
  code/tests/workflows/dependencies, or promote any matrix row or capability class.

### Completed `AUDIT-027` findings and `CORE-021` selection

- Exact three-review-approved public basis `aa3e7a8`, tree `4caa5c33`, passes
  compiler runs `30836250279`/`30836251909`, stable/nightly Rust run `30836255407`,
  all three analyses in CodeQL run `30836248101`, and aggregate `91762198170`.
  Auditors made no changes, ran no tests or probes, and left the worktree clean.
- All three auditors rank R-013 first by active reproducibility, reach, semantic
  readiness, phase count, compatibility ambiguity, and tests-first feasibility.
  Remaining raw-critical type, ownership, and unchecked-API work requires unfrozen
  semantics or a major-boundary policy; hardware needs real devices; spans, grammar,
  aggregates, and compiler convergence are architectural; the next dormant-test
  slice and supported-toolchain policy are not yet frozen.
- At the audited basis, the CPU execution branch obtains the delegated status, then
  unconditionally prints `Program executed successfully.` and `Exit code: N`.
  The process contract deterministically supplies exit 7 and currently requires the
  false success line. Cleanup completes before the wrapper propagates the exact child
  status; ROCm/CUDA remain fail closed.
- Reconciliation compared three R-013 slices. Two auditors rank the nonzero false-
  success presentation first; one ranks dangling-entry `init` containment first and
  presentation second. The lead selects presentation because it affects every
  nonzero CPU child, has a cross-platform deterministic regression, changes zero
  compiler phases, and preserves status/cleanup semantics. Entry-aware `init`
  preflight remains the bounded runner-up; hidden helper termination remains open.
- DEC-026 and `CORE-021` freeze one output condition only. This is claim containment,
  not language, execution, backend, safety, or stability capability; no matrix row or
  capability class is promoted.
- The exact three-review-approved tests-only checkpoint `0873f65`, tree `51ec7d0a`,
  diff `f75a6360`, now binds delegated exits `0/1/2/7`, exact status, the complete
  normalized CLI-owned presentation suffix, child output markers, and cleanup after
  all four cases execute. Local and public evidence is exact 10/1 on the false
  nonzero success line: compiler `30839264536` / `30839272375` and nightly in Rust
  `30839272429`; stable is cancelled during its test step by fail-fast. CodeQL
  `30839264268` and aggregate `91772180985` pass. The one-condition implementation
  passes focused CLI 11/11, backend-claim 7/7, and the exact full local gate. Exact
  tree `0ad98c82`, diff `2dbbc395`, received three approvals and was published as
  `a4327be`; compiler `30839860335` / `30839862442`, Rust `30839862423`, CodeQL
  `30839859840`, and aggregate `91774125621` all pass. The selected truthful
  presentation boundary is accepted without promoting any language, execution,
  backend, safety, or stability capability.
- Corrected exact record-only closure `b99e445`, tree `8a4c2d77`, diff `5abbf3a7`,
  is triple-approved and all-eight public green in compiler `30840427466` /
  `30840426655`, Rust `30840428215`, CodeQL `30840415565`, and aggregate
  `91775938704`. `AUDIT-028` is preregistered to compare every remaining OPEN or
  PARTIALLY CONTROLLED risk from a clean public head; it is read-only and cannot add
  or promote a capability.
- Public-green `AUDIT-028` basis `399e04f` completes that full-set comparison. The
  independent top threes are R-011/R-013/R-002, R-013/R-012/R-002, and
  R-002/R-013/R-010. R-013 is the only universal top-two residual. DEC-027 and
  preregistered `CORE-022` select only non-following, fail-closed destination-entry
  preflight before `aero init` writes. R-011 bounds behavior remains unfrozen; R-002
  remains a wider runner-up. This is project-tooling containment and adds no language,
  compiler, backend, filesystem-atomicity, safety, or stability capability.
- `CORE-022` is implemented and accepted at `2a42324`. Triple-reviewed tests-only
  `7cd8aba` produces exact Linux compiler 10/1 in `30843119793` / `30843125522` and
  nightly Rust `30843124314`; stable is fail-fast cancelled during tests, while all
  CodeQL analyses pass. Triple-reviewed implementation `2a42324` passes focused 3/3
  and 11/11 plus the exact local gate, compiler `30843592298` / `30843592784`, Rust
  `30843595560`, CodeQL `30843589175`, and aggregate `91786468184`. This controls only
  final-entry init preflight; rollback, atomicity, race freedom, ancestor symlinks,
  and every compiler/backend capability remain outside the result.
- Exact triple-reviewed record closure `aa29a00`, tree `e740df48`, diff `3eb8264b`,
  passes compiler `30844324249` / `30844328660`, Rust `30844328850`, CodeQL
  `30844325051`, and aggregate `91788926688`. `CORE-022` is complete only for the
  selected final-entry preflight; R-013 and all broader capability boundaries retain
  their recorded residual status.
- Exact triple-reviewed status synchronization `21153f3` is all-eight public green in
  compiler `30844798322` / `30844802332`, Rust `30844802044`, CodeQL `30844799426`,
  and aggregate `91790481511`. `AUDIT-029` is preregistered as a read-only,
  delta-aware ranking of all eleven remaining OPEN or PARTIALLY CONTROLLED risks. It
  cannot repeat an accepted slice, authorize implementation, or promote any
  capability.
- Exact triple-approved `AUDIT-029` basis `0e5cba1`, tree `6ac88db4`, is all-eight
  public green in compiler `30845609442` / `30845612610`, Rust `30845612328`,
  CodeQL `30845609103`, and aggregate `91793190047`. The three independent full
  rankings recommend different top slices: R-002 Boolean helper contracts, R-010
  grammar-authority containment, and R-009 parser UTF-16 columns. R-012 is their
  common second-place evidence-only slice. Lead reconciliation selects R-002 because
  semantics currently accepts invalid Boolean helper calls/returns and mis-infers
  valid Boolean call results as `Int`, while checked IR already has active exact
  `bool`/LLVM-`i1` function evidence. `CORE-023` freezes only monomorphic non-entry
  helper contracts in the semantic phase. No current capability is promoted before
  tests-first, implementation, full-gate, review, and public evidence.
- `CORE-023` now supplies that evidence. Corrected preregistration `1c28a7b` is
  all-eight public green. Triple-reviewed tests-only `c3f6e90` reproduces exact
  compiler 13/1 in `30848723940` / `30848725388` and nightly Rust `30848725757`,
  with only the Boolean helper contract target failing on the three frozen
  discrepancies; stable is fail-fast cancelled and CodeQL remains green. Triple-
  reviewed one-file implementation `67ccdf2` passes focused 1/1, function contracts
  14/14, binding controls 16/16, Boolean checked-IR `i1` 1/1, the exact local gate,
  compiler `30850000615` / `30850005598`, Rust `30850005670`, CodeQL
  `30850001251`, and aggregate `91807553635`. Non-entry monomorphic Boolean helper
  contracts are now controlled in semantics. R-002 remains PARTIALLY CONTROLLED;
  entry/ABI and every excluded type/shape remain open or quarantined, and no general
  type-safety, execution, stability, or backend capability is inferred.
- Exact triple-reviewed `CORE-023` record closure `0b88530`, tree `71ac4da7`, diff
  `adba01a1`, passes compiler `30850519757` / `30850524194`, stable/nightly Rust
  `30850524148`, CodeQL `30850520457`, and aggregate `91809289681`. This closes only
  the selected semantic contract boundary and promotes no broader capability.
  `AUDIT-030` is preregistered to rank all eleven remaining OPEN or PARTIALLY
  CONTROLLED risks from that clean head without inherited ordering, repeated slices,
  implementation authority, or unsupported language/backend claims.
- `AUDIT-030` is complete from exact public-green authorization `d4e3c75`, tree
  `9a07c10c`, with compiler `30851275589` / `30851278460`, Rust `30851278586`,
  CodeQL `30851276053`, and aggregate `91811764009`. The three independent rankings
  place R-009 in every top three and two rank it first. Parser diagnostics currently
  expose scalar columns directly to LSP while lexical diagnostics already project
  UTF-16. The lead selects that distinct one-file presentation correction under
  `CORE-024`; no current capability is promoted before tests-first and public proof.
  R-010 grammar containment is the runner-up. Entry contracts, dormant inventory,
  and every other residual retain their recorded stops.
- `CORE-024` now supplies the selected adapter evidence. Triple-reviewed
  preregistration `b8fb1d2` is all-eight public green. Triple-reviewed tests-only
  `ab8508e` produces exact 148/149 compiler and stable/nightly results in
  `30853599874` / `30853602996` / `30853603035`: only the astral-prefix parser
  coordinate target fails at scalar `20` versus required UTF-16 `21`; CodeQL and
  aggregate remain green. Exact triple-reviewed one-file implementation `a3d110e`,
  tree `79ccfca1`, diff `74bfbcea`, passes the focused regression 1/1, all LSP tests
  10/10, the exact full local gate, compiler `30854094706` / `30854099595`, Rust
  `30854099899`, CodeQL `30854094981`, and aggregate `91821038577`. Parser
  diagnostic starts after non-BMP prefixes are now protocol-correct UTF-16 offsets;
  the synthetic one-unit end, scalar internal locations, lexical path, and every
  compiler/backend stage are unchanged. LSP remains EXPERIMENTAL, diagnostics remain
  PARTIAL, and R-009 remains OPEN for real spans and recovery retention.
- Corrected exact `CORE-024` record closure `226b7fb`, tree `1337945c`, diff
  `861b5ec3`, received three fresh approvals with no P0-P3 findings after rejection
  of a stale-chronology snapshot. Compiler `30854853182` / `30854856449`, Rust
  `30854856190`, CodeQL `30854853829`, and aggregate `91823492290` all pass. This
  completes only the selected parser-start adapter; no capability row is promoted.
  `AUDIT-031` is preregistered to re-rank all eleven remaining OPEN or PARTIALLY
  CONTROLLED risks from a clean public head without inherited ordering, repeated
  slices, implementation authority, or unsupported language/backend claims.
- Public-green `AUDIT-031` authorization `ba258c6`, tree `651762a8`, passes compiler
  `30855407928` / `30855410819`, Rust `30855410731`, CodeQL `30855409113`, and
  aggregate `91825280915`. All three complete read-only rankings were reconciled on
  one distinct R-002 defect: an initialized exact outer tuple annotation silently
  disappears at direct semantics and checked admission, allowing a scalar RHS type
  to reach generation. Targeted reconciliation ranks that two-phase fail-closed
  containment above R-010's zero-phase grammar notice. `CORE-025` preregisters only
  categorical initialized outer-tuple rejection after child validation, including
  traversed generic statement contexts; tuple values/layout/ABI and every nested/
  uninitialized/generic-type/reference boundary remain unchanged. No capability is
  promoted before tests-first and public evidence.
- `CORE-025` now supplies that evidence without promoting tuple capability.
  Preregistration `722d4d1` is all-eight public green. After a P2 review corrected
  fragment-only diagnostics to exact equality, triple-reviewed tests-only
  `39ccd9c`, tree `5b05499f`, reproduces exactly 16 passed/1 failed in compiler
  `30857467570` /
  `30857469931` and nightly Rust `30857470046`; stable is fail-fast cancelled,
  while CodeQL `30857468030` and aggregate `91831822409` pass. Triple-reviewed
  two-phase implementation `1ec8beb`, tree `ac2c8fdd`, passes focused 1/1, binding
  17/17, the exact full gate, compiler `30857775577` / `30857777431`, stable/nightly
  Rust `30857777314`, CodeQL `30857775231`, and aggregate `91832840108`.
  Initialized exact outer tuple binding annotations now fail after child validation
  in semantics and independently at checked admission before binding insertion or
  generation, including traversed generic-impl contexts. Tuple values, uninitialized
  and nested annotations, layout, ABI, ownership, lowering, and execution remain
  absent or quarantined; tuple stays PARSED_ONLY and R-002 stays PARTIALLY CONTROLLED.
- Corrected exact `CORE-025` record closure `b0fe242`, tree `2a5d233f`, diff
  `98916b4d`, received three approvals and passes compiler `30858384541` /
  `30858387195`, stable/nightly Rust `30858387193`, CodeQL `30858385234`, and
  aggregate `91834740790`. This closes only the selected fail-closed annotation
  boundary; no capability class or matrix row is promoted.
- `AUDIT-032` is preregistered to re-rank the complete remaining R-002/R-004/R-005/
  R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016 set from a clean public head.
  It excludes every accepted sub-slice, must begin read-only only after its own
  exact review/public gates, and carries no implementation or capability authority.
- `AUDIT-032` authorization `b6b1c63`, tree `c8803965`, is triple-approved and
  all-eight green in compiler `30858876643` / `30858879497`, stable/nightly Rust
  `30858879480`, CodeQL `30858875767`, and aggregate `91836318450`. Three complete
  independent rankings and targeted reconciliation identify a distinct R-005
  phase-order defect: direct checked-AST arity mismatches for known otherwise-
  admitted scalar top-level helpers reach raw IR and fail only in verification.
- All three auditors rank the narrow one-phase admission guard above R-010 after
  freezing eligibility to nongeneric, non-entry scalar/Void signatures and
  preserving argument-child, local-callable, and Void-as-value precedence.
  `CORE-026` authorizes tests-first evidence for only this fail-before-IR boundary.
  Source semantics, accepted output, verifier defense, argument types, other
  callables/signatures, and every capability classification remain unchanged.
- Exact review rejected the first `CORE-026` authorization at P2: scalar-shaped but
  verifier-invalid/reserved signatures and duplicate top-level identities were not
  excluded, so an arity guard could mask existing verifier failures. The corrected
  boundary additionally requires one declaration, verifier-valid unique symbols,
  and a non-reserved function identity; dedicated controls must preserve every
  excluded verifier failure. No implementation or capability change occurred.
- Corrected `CORE-026` authorization `7dc3eac` is triple-approved and all-eight
  public green. After a separate review rejected caller-after-callee ordering and
  missing composite/reference result controls, corrected triple-reviewed tests-only
  `1538a3e`, tree `8f3cd8fb`, publicly reproduces exactly 6 passed/1 failed: only the
  selected phase-order target remains at Verification instead of Admission. Stable
  is fail-fast cancelled; CodeQL and aggregate pass.
- Triple-reviewed one-phase implementation `8c2b2ec`, tree `eabd8939`, passes focused
  1/1, checked-IR 7/7, the exact full gate, compiler `30862232159` / `30862233829`,
  stable/nightly Rust `30862233777`, CodeQL `30862232615`, and aggregate
  `91846586968`. Eligible direct checked-AST wrong-arity calls now fail at Admission
  before raw IR after all frozen precedence checks. Malformed/duplicate signatures,
  accepted programs, verifier defense, source semantics, codegen, ABI, and backends
  remain unchanged. R-005 stays PARTIALLY CONTROLLED and no capability is promoted.
- Corrected exact `CORE-026` record closure `0a940ea`, tree `6ec4c609`, diff
  `4e1db178`, received three approvals and passes compiler `30862783787` /
  `30862786131`, stable/nightly Rust `30862786150`, CodeQL `30862784231`, and
  aggregate `91848258218`. This closes only the selected checked-admission phase-
  order guard; no capability class or matrix row is promoted.
- `AUDIT-033` is preregistered to re-rank the complete remaining R-002/R-004/R-005/
  R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016 set from the clean public closure.
  It excludes every accepted sub-slice, must begin read-only only after its own exact
  review/public gates, and carries no implementation or capability authority.
- `AUDIT-033` authorization `544b1ba`, tree `cdc3a085`, is triple-approved and
  all-eight green in compiler `30863291761` / `30863294642`, stable/nightly Rust
  `30863294655`, CodeQL `30863292940`, and aggregate `91849762353`. Three complete
  independent rankings and final targeted reconciliation select R-010 authority
  containment in the split grammar and core-features tutorial.
- A distinct R-005 argument-type admission defect remains stopped: admission-returned
  scalar `Ty` is not yet proof that logical/unary/local-callable/unknown child forms
  satisfy verifier contracts, so an outer guard could mask an earlier child failure.
  `CORE-027` authorizes tests-first evidence only for visible design-target notices
  and one grammar introduction authority sentence. No production, example, compiler
  behavior, matrix cell, or capability classification may change.
- Triple-reviewed `CORE-027` authorization `3574704` is all-eight public green.
  Triple-reviewed tests-first `f57cf2e`, tree `8a99d994`, then publicly reproduces
  exactly 7 passed/1 failed in compiler `30864786831` / `30864789388` and nightly
  Rust `30864789399`: only the new authority contract fails; stable is fail-fast
  cancelled, while CodeQL `30864787921` and aggregate `91854279316` pass.
- Corrected exact implementation `b3e7910`, tree `2728bbc6`, diff `90e1c4b6`,
  passes focused 1/1, version-claim 8/8, the exact full gate, compiler
  `30865344667` / `30865346597`, stable/nightly Rust `30865346602`, CodeQL
  `30865345043`, and aggregate `91855955012`. The first snapshot was rejected
  before publication for an extra final-newline mutation; the corrected snapshot
  preserves the grammar's original EOF representation and received three approvals.
  Both documents now carry the visible boundary and current-record pointers, while
  every production, example, compiler behavior, and capability classification is
  unchanged. R-010 remains HIGH/HIGH and OPEN.
- Exact `CORE-027` record closure `d649c2d`, tree `b5ad7ee2`, diff `d4281863`,
  received three approvals and passes compiler `30865772404` / `30865775196`,
  stable/nightly Rust `30865775214`, CodeQL `30865772793`, and aggregate
  `91857289172`. This closes only the documentation-authority containment; no
  capability class or matrix row is promoted.
- `AUDIT-034` is preregistered to re-rank the complete remaining R-002/R-004/R-005/
  R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016 set from the clean public
  closure. It excludes every accepted sub-slice, including the CORE-027 notices,
  must begin strictly read-only only after its own exact review/public gates, and
  carries no implementation or capability authority.

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

## AUDIT-034 reconciliation and CORE-028 boundary

- Exact `AUDIT-034` authorization snapshot `26c1eda`, tree `f1baa457`, diff
  `1e8563ae`, received three approvals and was published unchanged as `45783af`.
  Compiler `30866227485` / `30866229553`, stable/nightly Rust `30866229554`,
  all three CodeQL analyses in `30866227939`, and aggregate `91858665436` pass.
- Type/safety ranks R-002/R-009/R-012/R-005/R-011/R-004/R-013/R-010/R-006/
  R-016/R-007. IR/codegen ranks R-005/R-012/R-002/R-013/R-011/R-004/R-010/
  R-009/R-006/R-007/R-016. Backend/claim ranks R-005/R-002/R-011/R-016/R-013/
  R-012/R-006/R-009/R-010/R-004/R-007. The read-only worktree remained clean.
  R-012's recorded population is confirmed as 16 disconnected files/299 tests.
- Final targeted reconciliation unanimously selects one exact R-002 public false
  success: a valueless binding with outer `Type::Tuple(_)` annotation silently
  becomes `Ty::Int` in semantics, is skipped by checked admission, and can become
  `(ImmInt(0), Ty::Int)` in raw generation. The existing acceptance assertions are
  quarantine evidence; the invariant forbidding unsupported-type fallback freezes
  rejection without defining tuple defaults, layout, ABI, or execution.
- At selection time, DEC-033 and preregistered `CORE-028` permitted only exact
  semantic and checked-admission diagnostics for that outer valueless tuple form,
  with existing semantic duplicate-name precedence first. Initialized tuple
  behavior, nested tuple shapes,
  every other valueless annotation, raw generation, verifier, codegen, ABI,
  ownership, and backends remain unchanged. No matrix row or capability is promoted;
  R-002 remains HIGH/CRITICAL and PARTIALLY CONTROLLED.
- The R-005 runner-up is a zero-argument call through a local closure alias whose
  admitted scalar signature requires parameters. It remains stopped for a later
  contract: it reaches raw IR but is already rejected by mandatory verification
  before LLVM, and supplied-argument variants have unresolved child-precedence risk.

## CORE-028 accepted implementation

- Corrected authorization snapshot `ba78f713`, tree `be2987d0`, diff `7a658443`,
  received three approvals and was published unchanged as `4cc682f`; all eight
  public checks pass. Superseded snapshot `696dcaad` was rejected at P2 before
  publication for insufficient tuple-specific precedence and outer-shape controls.
- Triple-reviewed tests-first `3fb5f7a`, tree `f12a6c6b`, publicly reproduces one
  exact 16/1 aggregate failure across both compiler jobs and stable/nightly Rust:
  ordinary semantics, checked admission, public compilation, generic-impl semantics,
  and generic-impl admission all falsely accept the valueless outer tuple. Duplicate
  precedence and scalar/array/reference/generic outer-shape controls remain green;
  CodeQL and its aggregate pass.
- Triple-reviewed implementation `e051452`, tree `63985b2d`, diff `79830403`, adds
  only two exact guards in semantics and checked admission. Focused 1/1, binding
  17/17, the full local gate, both compiler jobs, stable/nightly Rust, all three
  CodeQL analyses, and aggregate pass.
- Exact outer `Type::Tuple(_)` valueless bindings now fail before fake `Int`,
  insertion, or generation. This is rejection evidence, not tuple implementation.
  Initialized tuple behavior, nested tuple shapes, other valueless annotations, raw
  generation, verifier, codegen, ABI, ownership, and CPU/ROCm/CUDA behavior remain
  unchanged. No capability class or matrix cell changes; R-002 remains HIGH/CRITICAL
  and PARTIALLY CONTROLLED.

## CORE-028 closure and AUDIT-035 boundary

- Exact six-record closure snapshot `f6305e18`, tree `443aacdc`, diff `93fce8ae`,
  received three approvals and was published unchanged as `032d0d0`. Compiler
  `30872236535` / `30872238993`, stable/nightly Rust `30872239003`, all three CodeQL
  analyses in `30872237025`, and aggregate `91876507154` pass.
- CORE-028 adds only fail-closed containment for the exact valueless outer tuple
  annotation. Other uninitialized annotations, tuple annotations beneath non-tuple
  outer shapes, tuple support, valid-output certification, unchecked APIs, ABI,
  ownership, and CPU/ROCm/CUDA behavior remain unchanged. R-002 remains
  HIGH/CRITICAL and PARTIALLY CONTROLLED; no capability class or matrix cell moves.
- `AUDIT-035` is preregistered to re-rank the complete remaining R-002/R-004/R-005/
  R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016 set from its exact clean public
  authorization head. It excludes every accepted sub-slice including CORE-028, must
  remain static and read-only, and cannot inherit the AUDIT-034 ordering. Ranking
  cannot begin until its exact local/review/public gates pass. AUDIT-035 never carries
  implementation or capability authority; any later test or source change requires
  a separately reviewed task contract and tests-first evidence.

## AUDIT-035 reconciliation and CORE-029 boundary

- Corrected AUDIT-035 authorization snapshot `bcb05d52`, tree `b9c6270b`, diff
  `7f221d2a`, received three approvals and was published unchanged as `f1cd972`.
  Compiler `30872922468` / `30872923806`, stable/nightly Rust `30872923874`, all
  three CodeQL analyses in `30872922858`, and aggregate `91878491979` pass.
- Type/safety ranks R-002/R-009/R-011/R-005/R-012/R-013/R-004/R-010/R-006/R-016/
  R-007. IR/codegen ranks R-005/R-002/R-011/R-012/R-013/R-009/R-006/R-016/R-010/
  R-004/R-007. Backend/claim ranks R-005/R-002/R-011/R-013/R-012/R-010/R-009/
  R-004/R-006/R-016/R-007. The immutable worktree remained clean; no tests, probes,
  artifacts, hardware actions, or external queries ran.
- Final targeted reconciliation unanimously selects exact R-002 valueless immediate
  reference-to-tuple rejection. Unlike the verifier-contained R-005 runner-up, this
  unsupported source shape becomes `Ty::Int` in semantics, is skipped by checked
  admission, becomes `ImmInt(0)` in raw generation, and can reach trusted LLVM/CPU
  publication as valid scalar IR.
- DEC-034 and preregistered CORE-029 permit only two exact rejection guards for outer
  `Type::Reference` with immediate tuple referent and `value: None`, after semantic
  duplicate precedence and before fake integer state or generation. Both mutable and
  immutable flags are included. No recursive type check, tuple/reference support,
  ownership meaning, matrix row, or capability is authorized. R-002 remains
  HIGH/CRITICAL and PARTIALLY CONTROLLED.

## CORE-029 accepted implementation

- Corrected authorization `c0e1a90`, tree `3960cc07`, is triple-approved and all-
  eight public green. The first authorization snapshot was rejected at P2 before
  publication for missing exact immutable/mutable duplicate-precedence specimens.
- Corrected triple-reviewed tests-first `d12ba66`, tree `056a9d52`, publicly
  reproduces one exact 17/18 aggregate failure containing only the five frozen false
  acceptances. Both duplicate controls and immutable/mutable scalar, nested,
  initialized, and second-reference-layer controls remain green. The first test
  snapshot was rejected at P2 before publication for missing two mutable controls;
  CodeQL and its aggregate pass on the corrected red commit.
- Triple-reviewed implementation `29bd2e0`, tree `53282149`, diff `acc1c247`, adds
  only the two exact non-recursive guards in semantics and checked admission.
  Focused 1/1, binding 18/18, formatting, the exact full local gate, both compiler
  jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate pass.
- Valueless immediate reference-to-tuple annotations now stop after semantic
  duplicate detection and before fake `Int`, insertion, or raw generation at both
  trusted boundaries. This is rejection evidence, not tuple/reference/ownership
  implementation. Initialized and deeper forms, other annotations, raw IR,
  verifier, codegen, ABI, and CPU/ROCm/CUDA behavior remain unchanged. No capability
  class or matrix cell changes; R-002 remains HIGH/CRITICAL and PARTIALLY CONTROLLED.

## CORE-029 closure and AUDIT-036 boundary

- Exact closure snapshot `6c7358be`, tree `66084b36`, diff `90bf540c`, received
  three approvals and was published unchanged as `7222b9a`. Compiler
  `30876033717` / `30876035730`, stable/nightly Rust `30876035761`, all three
  CodeQL analyses in `30876034500`, and aggregate `91887644623` pass.
- CORE-029 adds only fail-closed containment for the exact valueless immediate
  reference-to-tuple annotation. Other uninitialized/nested annotations,
  tuple/reference/ownership support, valid-output certification, unchecked APIs,
  ABI, and CPU/ROCm/CUDA behavior remain unchanged. R-002 remains HIGH/CRITICAL and
  PARTIALLY CONTROLLED; no capability class or matrix cell moves.
- `AUDIT-036` is preregistered to re-rank the complete remaining R-002/R-004/R-005/
  R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016 set from its exact clean public
  authorization head. It excludes every accepted sub-slice including CORE-029, must
  remain static and read-only, cannot inherit AUDIT-035's ordering, and carries no
  implementation or capability authority. Ranking cannot begin until the separate
  exact local/review/public gates pass.

## AUDIT-036 reconciliation and CORE-030 boundary

- Corrected exact authorization snapshot `a805d1c9`, tree `3cdf89e6`, diff
  `40896f51`, received three approvals and was published unchanged as `f4ac505`.
  Compiler `30876975678` / `30876977928`, stable/nightly Rust `30876977905`, all
  three CodeQL analyses in `30876976155`, and aggregate `91890402326` pass.
- Type/safety ranks R-002/R-005/R-012/R-009/R-011/R-004/R-013/R-010/R-006/R-016/
  R-007. IR/codegen ranks R-002/R-005/R-011/R-012/R-013/R-006/R-009/R-004/R-010/
  R-016/R-007. Backend/claim ranks R-002/R-005/R-011/R-004/R-006/R-012/R-013/
  R-010/R-009/R-016/R-007. The audit remained static, read-only, and clean.
- All three select the exact valueless immediate array-of-tuple annotation over the
  R-005 runner-up. Current semantics replaces this unsupported annotation with
  `Ty::Int`, checked admission skips it, and raw generation can emit verifier-valid
  integer zero. R-005 already fails mandatory verification before LLVM.
- DEC-036 and preregistered CORE-030 permit only non-recursive semantic and checked-
  admission rejection for `Type::Array(Type::Tuple(_), count)` on a valueless
  binding, with semantic duplicate precedence preserved. Tests must be public red
  first under a separately gated contract.
- This selection changes no current capability. Initialized and deeper forms,
  scalar arrays, generic/Vec and reference wrappers, tuple/array support, bounds,
  layout, mutation, ABI, ownership, raw APIs, verifier, codegen, and every backend
  remain unchanged. R-002 remains HIGH/CRITICAL and PARTIALLY CONTROLLED.

## CORE-030 accepted implementation

- Triple-approved authorization `1f13084` is public all-eight green. Triple-reviewed
  tests-first `bd28f6a` publicly reproduces only the exact five frozen acceptances:
  semantic and checked admission at zero and nonzero counts plus the public route.
  Duplicate precedence and every initialized, scalar, wrapped, deeper, prior-
  diagnostic, and valid-output preservation control stay green.
- Triple-reviewed implementation `97c0f04`, tree `aa3a9e3f`, diff `06a104df`, adds
  only the two exact non-recursive guards in semantic analysis and checked IR
  admission. Focused 1/1, binding 19/19, formatting, the exact full local gate,
  compiler `30878810762` / `30878812430`, stable/nightly Rust `30878812406`, all
  three CodeQL analyses in `30878811198`, and aggregate `91895661773` pass.
- A valueless immediate array-of-tuple annotation now stops after semantic duplicate
  detection and before fake `Int`, insertion, or raw generation at both trusted
  boundaries. Initialized and deeper forms, scalar arrays, generic/Vec and reference
  wrappers, other annotations, raw IR, verifier, codegen, ABI, ownership, and every
  backend remain unchanged. This is containment, not array/tuple support; R-002
  remains HIGH/CRITICAL and PARTIALLY CONTROLLED, with no capability or matrix-cell
  change.

## CORE-030 closure and AUDIT-037 boundary

- Exact closure snapshot `9b872297`, tree `8ab06d62`, diff `18ffa30d`, received
  three approvals and was published unchanged as `cd8add28`. Compiler
  `30879329940` / `30879332975`, stable/nightly Rust `30879332995` attempt 2,
  all three CodeQL analyses in `30879330627`, and aggregate `91897195358` pass.
  Rust attempt 1 hit transient Linux `ETXTBSY` in the unchanged fake-verifier test;
  both focused rerun jobs passed without a source, test, workflow, or ref change.
- CORE-030 remains exact fail-closed containment only. All excluded annotation
  shapes, tuple/array behavior, raw APIs, verifier/codegen, ABI/ownership, valid-
  output certification, and CPU/ROCm/CUDA capability remain unchanged. R-002 stays
  HIGH/CRITICAL and PARTIALLY CONTROLLED; no matrix cell or capability class moves.
- Preregistered read-only AUDIT-037 must independently re-rank the complete remaining
  eleven-risk set from exact clean public closure `cd8add28`, excluding every
  accepted slice through CORE-030 and inheriting no prior order. It may select one
  bounded residual or a stop only after separate authorization gates; it never
  grants test, implementation, semantics, or capability authority.

## AUDIT-037 reconciliation and CORE-031 boundary

- Exact AUDIT-037 authorization snapshot `f4de8ef4`, tree `0b685659`, diff
  `d3a9974b`, received three approvals and was published unchanged as `987188fc`.
  Compiler `30880025888` / `30880028697`, stable/nightly Rust `30880028653`, all
  three CodeQL analyses in `30880025866`, and aggregate `91899286217` pass.
- Type/safety ranks R-002/R-005/R-012/R-011/R-004/R-013/R-009/R-010/R-006/R-016/
  R-007. IR/codegen ranks R-002/R-005/R-011/R-012/R-013/R-006/R-009/R-004/R-010/
  R-016/R-007. Backend/claim ranks R-002/R-005/R-011/R-013/R-004/R-012/R-006/
  R-009/R-010/R-016/R-007. The audit stayed static, read-only, and clean.
- Type/safety and IR initially select exact valueless array-array-tuple fallback;
  backend initially selects reference-array-tuple. Targeted static comparison makes
  all three select the two-array form: equal trusted reach and two-phase feasibility,
  with no reference mutability/ownership association. R-005 remains second and is
  verifier-contained before LLVM.
- DEC-038 and preregistered CORE-031 permit only nonrecursive semantic and checked-
  admission rejection for the exact valueless two-array-deep tuple shape, with both
  counts ignored and semantic duplicate precedence preserved. The reference-array
  form and three-or-more array layers stay accepted. Tests must be public red first
  under a separately gated contract.
- This selection changes no current capability. Nested-array/tuple values, defaults,
  bounds, layout, mutation, ABI, ownership, raw APIs, verifier, codegen, and all
  backends remain unchanged. R-002 stays HIGH/CRITICAL and PARTIALLY CONTROLLED.

## CORE-031 accepted implementation

- Exact authorization `ba57efec`, tree `c01bebe9`, is triple-approved and public
  all-eight green. Exact tests-first `6899cb1b`, tree `b7007735`, canonical diff
  `43063551`, publicly reproduces exactly nine false acceptances: four count
  combinations at semantic and checked boundaries plus public compilation. Both
  compiler runs and nightly Rust fail only the new 19/20 binding aggregate after
  139/139 library, 149/149 binary, and 7/7 claim tests; stable is fail-fast
  cancelled, while all CodeQL checks pass.
- Triple-approved implementation `4bc7a345`, tree `61361621`, canonical diff
  `349e34ee`, adds only two nonrecursive guards. Both require a valueless outer
  `Array` whose immediate child is `Array` and whose immediate grandchild is
  `Tuple`; counts are wildcarded. Semantic duplicate precedence and prior exact
  tuple diagnostics are unchanged.
- Focused 1/1, binding 20/20, formatting, the exact full local gate, compiler
  `30882153355` / `30882155935`, stable/nightly Rust `30882155921`, all three
  CodeQL analyses in `30882154595`, and aggregate `91905705897` pass.
- Candidate B, initialized forms, scalar/nested-scalar arrays, generic/reference
  wrappers, array-of-reference, reference-wrapped target, and third-plus array depth
  remain preserved. Raw IR, verifier, codegen, ABI, ownership, valid-output scope,
  and CPU/ROCm/CUDA behavior are unchanged. This is exact fail-closed containment,
  not nested-array or tuple support; no capability class or matrix cell moves, and
  R-002 remains HIGH/CRITICAL and PARTIALLY CONTROLLED.

## CORE-031 closure and AUDIT-038 boundary

- Exact closure `45696091`, tree `480c3504`, canonical diff `d682b0f6`, received
  three approvals and is public all-eight green in compiler `30882630407` /
  `30882632698`, stable/nightly Rust `30882632696`, all three CodeQL analyses in
  `30882630822`, and aggregate `91907149874`.
- CORE-031 remains exact fail-closed containment only. Candidate B and every other
  excluded annotation shape, tuple/nested-array behavior, raw API, verifier/codegen,
  ABI/ownership, valid-output certification, and CPU/ROCm/CUDA capability remain
  unchanged. R-002 stays HIGH/CRITICAL and PARTIALLY CONTROLLED; no matrix cell or
  capability class moves.
- Preregistered read-only AUDIT-038 must independently re-rank the complete remaining
  eleven-risk set from exact clean public closure `45696091`, excluding every
  accepted slice through CORE-031 and inheriting no prior candidate or order. It may
  select one bounded residual or a stop only after separate authorization gates; it
  never grants test, implementation, semantics, or capability authority.

## AUDIT-038 reconciliation and CORE-032 boundary

- Corrected AUDIT-038 authorization `e4d58e59`, tree `f265d8af`, canonical diff
  `31d09f92`, received three approvals and is public all-eight green in compiler
  `30883186212` / `30883188223`, stable/nightly Rust `30883188248`, all three
  CodeQL analyses in `30883186829`, and aggregate `91908783685`. Rejected
  `89bc5709` was never published because its current-state wording lagged the local
  gate.
- Type/safety ranks R-002/R-011/R-005/R-012/R-004/R-013/R-009/R-006/R-010/R-016/
  R-007. IR/codegen ranks R-002/R-005/R-011/R-009/R-006/R-012/R-013/R-004/R-010/
  R-016/R-007. Backend/claim ranks R-002/R-005/R-011/R-004/R-013/R-012/R-010/
  R-006/R-009/R-016/R-007. The audit stayed static, read-only, and clean.
- Initial candidates split between initialized exact `Array(Tuple)` and valueless
  exact `Array(Array(Array(Tuple)))`. Both have trusted scalar-IR reach and two-phase
  feasibility. Targeted preference comparison remained split, so the lead selected
  the initialized form provisionally based on its smaller predicate/test surface and
  CORE-025's established initializer-child ordering. A final compatibility gate made
  all three reviewers explicitly approve that exact boundary without a semantic or
  phase blocker. The triple-array form remains accepted and preserved.
- DEC-040 and preregistered CORE-032 permit only semantic and checked-admission
  rejection for initialized immediate `Array(Tuple)`, count ignored, after child
  validation and existing outer-tuple handling. The rule applies wherever those
  statement paths are already traversed, including generic impls; semantic generic-
  function traversal is in scope, while checked admission retains its earlier outer
  generic-function rejection. Tests must be public red first under a separately
  gated contract. The first five-acceptance authorization snapshot was rejected
  before publication because it omitted those contexts; the corrected red surface
  contains eight acceptances.
- This selection changes no capability. Tuple/array values, compatibility, defaults,
  bounds, layout, mutation, ABI, ownership, raw APIs, verifier/codegen, and all
  backends remain unchanged. R-002 stays HIGH/CRITICAL and PARTIALLY CONTROLLED.

## CORE-032 accepted implementation

- Corrected authorization `449f3536`, tree `24edc1fe`, canonical diff `d65f6b75`,
  is triple-approved and public all-eight green in compiler `30885443132` /
  `30885447315`, Rust `30885447416`, CodeQL `30885443837`, and aggregate
  `91915624793`.
- The first tests-only snapshot `1afe11d3`, tree `aa4154b0`, canonical diff
  `9600c937`, was rejected before publication because it silently removed the
  child-valid array-literal target specimen. Corrected `35eac8c4`, tree `b54a848b`,
  canonical diff `e600c2bc`, received three approvals and publicly reproduces only
  the named 20/21 failure with exactly eight acceptances in compiler `30886282169` /
  `30886283814` and nightly Rust `30886284165`; stable is fail-fast cancelled,
  while CodeQL `30886281888` and aggregate `91918210639` pass.
- Exact implementation `30d0d730`, tree `653346ce`, canonical diff `01e87768`, adds
  22 lines only in semantic analysis and checked admission. Focused 1/1, binding
  21/21, formatting, and two consecutive exact full gates pass after an earlier
  unexplained truncated exit-1 attempt. Three reviewers approved with no P0-P3.
  Compiler `30886856260` / `30886858878`, stable/nightly Rust `30886858960`, all
  three CodeQL analyses in `30886856518`, and aggregate `91919998289` pass.
- Initialized immediate `Array(Tuple)` annotations now fail after initializer
  validation at both trusted boundaries, including traversed generic contexts.
  Candidate T/B, valueless and deeper/wrapped annotations, tuple/array values and
  compatibility, raw APIs, verifier/codegen, ABI/ownership, valid-output behavior,
  and CPU/ROCm/CUDA remain unchanged. This adds no capability evidence or matrix
  movement; R-002 stays HIGH/CRITICAL and PARTIALLY CONTROLLED.
- First closure snapshot `7d7fe3d6` passed its exact gate but was rejected
  unpublished by all three reviewers because the state record still called that
  gate future work; the type review also required the known exit 1 above instead of
  generic nonzero. Second snapshot `48f2fd60`, tree `86175cc1`, canonical diff
  `9f0ab102`, resolved those findings and received two approvals but was rejected
  unpublished at P3 by the type reviewer because the successful closure gate lacked
  literal `exit 0`. The twice-corrected six-record tree records both review rounds;
  its fresh exact gate exits 0 with 139/139 library, 149/149 binary, 7/7 doc, and
  21/21 binding tests. Exact closure `9c82cbfc`, tree `b2a106ee`, canonical diff
  `fc672744`, then received three approvals, was published unchanged, and passes
  compiler `30888222316` / `30888225734`, Rust `30888226011`, CodeQL `30888222480`,
  and aggregate `91924197947`.

## AUDIT-039 reconciliation and CORE-033 boundary

- The complete remaining set is still R-002/R-004/R-005/R-006/R-007/R-009/R-010/
  R-011/R-012/R-013/R-016. AUDIT-039 must rank all eleven independently from exact
  clean public closure `9c82cbfc`, exclude every accepted slice through CORE-032,
  and inherit neither Candidate T/B nor any earlier ordering.
- Each reviewer must provide evidence for every rank, one bounded candidate or stop,
  trusted reach and containment, unresolved semantic choices, phase count, one exact
  deterministic failing specimen, and preservation controls. Rejection, annotation,
  simulation, LLVM text, object emission, and hardware execution remain distinct.
- Exact authorization `fa522b2c`, tree `365a536d`, canonical diff `cefb797e`, is
  triple-approved and public all-eight green in compiler `30888751268` /
  `30888754238`, Rust `30888754262`, CodeQL `30888752230`, and aggregate
  `91925849313`.
- Type/safety ranks R-002/R-011/R-005/R-012/R-004/R-013/R-009/R-006/R-010/R-016/
  R-007. IR/codegen ranks R-002/R-005/R-011/R-006/R-009/R-012/R-013/R-004/R-010/
  R-016/R-007. Backend/claim ranks R-002/R-005/R-011/R-004/R-013/R-006/R-012/
  R-009/R-010/R-016/R-007. The audit stayed static, read-only, and clean.
- Initial candidates split: type/safety selected the valueless exact three-array form
  historically labeled Candidate T; IR/codegen and backend selected initialized exact two-array Candidate
  A. Preference comparison favored A two to one. The lead provisionally selected A
  because it has two rather than three count dimensions, exactly 12 rather than 20
  red acceptances, a smaller predicate, and CORE-032-frozen initializer ordering.
  All three then explicitly approved exact A with no semantic or phase blocker.
- Preregistered CORE-033 may reject only initialized exact `Array(Array(Tuple))` at
  semantic and checked boundaries after initializer/existing diagnostics. It adds no
  tuple/nested-array meaning. Candidate T, reference-array Candidate B, other deeper/
  wrapped shapes, raw APIs, valid output, backends, risk status, matrix cells, and
  capability classes remain unchanged.
- The prepared CORE-033 authorization's fresh exact full gate exits 0 with 139/139
  library, 149/149 binary, 7/7 doc, and 21/21 binding tests. At that stage,
  tests-first remained forbidden until three exact reviews, unchanged publication,
  and all eight checks.
- First authorization snapshot `d0500865`, tree `d2378320`, canonical diff
  `97a15c9f`, passed its local gate but was rejected unpublished by two reviewers
  after one ledger sentence called Candidate T's valueless form Candidate B. The
  corrected records keep Candidate T and reference-array Candidate B distinct.

## CORE-033 accepted implementation

- Corrected authorization `66207215`, tree `357c2731`, canonical diff `96b5f403`,
  received three approvals and is public all-eight green in compiler `30890569245`
  / `30890571370`, Rust `30890571249`, CodeQL `30890569479`, and aggregate
  `91931557818`.
- First tests-only `7608b42c`, tree `5a2100ee`, canonical diff `d68b42ed`, was
  rejected unpublished because it omitted an explicit initialized three-array-deep
  semantic/checked green control. Corrected `ac4cb2a5`, tree `852bff0b`, canonical
  diff `4ca50572`, received three approvals and publicly isolates exactly 12 false
  acceptances as the sole 21/22 binding failure in compiler `30891243037` /
  `30891246443` and nightly Rust `30891247469`; stable was fail-fast cancelled,
  while CodeQL `30891241566` and aggregate `91933672071` pass.
- Accepted implementation `76a6e802`, tree `d8391348`, established PowerShell
  full-index canonical diff `a75b59b2`, adds exactly 31 lines in semantic analysis
  and checked admission. Formatting, focused 1/1, binding 22/22, and the exact full
  local gate exit 0 with 139/139 library, 149/149 binary, 7/7 claim, and 22/22
  binding tests. An initial review request used erroneous plain-diff `c17b1b6a`;
  corrected identity review of the unchanged commit received three approvals.
  Compiler `30891890629` / `30891898590`, stable/nightly Rust `30891897083`, all
  three CodeQL analyses in `30891892219`, and aggregate `91935804190` pass.
- Initialized exact nonrecursive `Array(Array(Tuple))` annotations now reject after
  initializer validation and existing diagnostics at both trusted boundaries.
  Candidate T, reference-array Candidate B, all deeper/wrapped or valueless forms,
  tuple/nested-array meaning, raw APIs, verifier/codegen, ABI/ownership, valid-output
  certification, and CPU/ROCm/CUDA remain unchanged. This is containment, not a
  capability promotion; R-002 stays HIGH/CRITICAL and PARTIALLY CONTROLLED and no
  matrix cell moves.
- First six-record closure snapshot `fe90f583`, tree `90ac8ae6`, canonical diff
  `89fe6824`, changed only the control records and passed its exact gate with
  139/139 library, 149/149 binary, 7/7 claim, and 22/22 binding tests. It received
  two approvals but was rejected at P1 before independent push or branch-head
  publication because PROJECT_STATE retained stale future tests/implementation
  wording. First correction `19f688a`, tree `9d9c642f`, canonical diff `f885588c`,
  made that chronology historical, passed the same exact gate, received three
  approvals, and was pushed. Compiler `30893002336` / `30893005706`, Rust
  `30893006634`, CodeQL `30893002479`, and aggregate `91939375982` pass. That push
  also made rejected parent `fe90f583` publicly reachable as an ancestor, so its
  never-published wording was inaccurate and closure was withheld. Additive record
  correction `1ee9c71`, tree `d0819881`, canonical diff `7303da47`, passed its
  fresh exact gate with 139/139 library, 149/149 binary, 7/7 claim, and 22/22
  binding tests, received three exact approvals, and was published unchanged.
  Compiler `30893527220` / `30893529999`, stable/nightly Rust `30893529992`, all
  three CodeQL analyses in `30893527445`, and aggregate `91941079083` pass.

## AUDIT-040 result and CORE-034 authorization boundary

- The complete remaining set is still R-002/R-004/R-005/R-006/R-007/R-009/R-010/
  R-011/R-012/R-013/R-016. AUDIT-040 ranked all eleven independently from exact
  clean public closure `1ee9c71`, excluded every accepted slice through CORE-033,
  and inherited neither Candidate T/B/A nor any earlier ordering.
- First authorization snapshot `c83ec3a`, tree `bb25e528`, canonical diff
  `c02f71e5`, passed its exact repository-root full gate with 139/139 library,
  149/149 binary, 7/7 claim, and 22/22 binding tests. Type/safety and backend/claim
  approved, but IR/codegen rejected at P1 because a late PROJECT_STATE subsection
  still treated accepted CORE-033 closure as future work. It was rejected before
  publication. Corrected authorization `7b9ed83`, tree `8dbe975e`, canonical diff
  `c4ba110a`, passed its fresh exact gate with 139/139 library, 149/149 binary, 7/7
  claim, and 22/22 binding tests, received three exact approvals, and was published
  unchanged. Compiler `30894708169` / `30894713332`, stable/nightly Rust
  `30894713411`, all three CodeQL analyses in `30894708736`, and aggregate
  `91944883143` pass.
- Type/safety ranked R-002/R-005/R-004/R-011/R-009/R-012/R-013/R-006/R-010/R-016/
  R-007 and selected valueless exact three-array tuple containment. IR/codegen
  ranked R-002/R-012/R-005/R-013/R-011/R-016/R-010/R-006/R-009/R-004/R-007 and
  selected initialized exact immediate reference-to-tuple containment. Backend/
  claim ranked R-011/R-005/R-002/R-004/R-006/R-013/R-009/R-010/R-007/R-012/R-016
  and selected immediate nonnegative literal fixed-array bounds containment.
- Targeted comparison preferred reference containment two to one. Literal bounds
  remains stopped pending a separately frozen compile-time-versus-runtime bounds
  policy. Exact three-array valueless containment remains bounded but has greater
  topology and test-count burden. All three final compatibility reviews approved
  only initialized exact nonrecursive `Type::Reference(Type::Tuple(_), _)`
  rejection at semantic and checked-admission boundaries, for both mutability
  flags and with the frozen diagnostic/ordering/context boundary. AUDIT-040 was
  read-only and changed no capability, matrix cell, risk status, source, or test.
- CORE-034 is now preregistered to reclassify exactly two existing acceptance rows
  and reproduce exactly 30 unexpected acceptances in one tests-first aggregate only
  after this six-record authorization passes its fresh full gate, receives three
  exact approvals, is published unchanged, and passes all eight public checks.
  Implementation remains limited to the semantic analyzer and checked IR admission
  after separately reviewed public-red evidence. This defines no reference or tuple
  value, mutability, ownership, lifetime, layout, ABI, coercion, lowering, execution,
  backend, or stability capability. R-002 remains HIGH/CRITICAL and PARTIALLY
  CONTROLLED; no capability or matrix classification moves. First authorization
  snapshot `7d4d7ca`, tree `b633abbb`, canonical diff `a901f4dc`, passed its exact
  full gate with 139/139 library, 149/149 binary, 7/7 claim, and 22/22 binding tests.
  IR/codegen and backend/claim approved, but type/safety rejected it at P1 because
  TASK_LEDGER's final status still called the completed gate future work. It remained
  unpublished. The corrected authorization's fresh exact full gate exits 0 with
  139/139 library, 149/149 binary, 7/7 claim, and 22/22 binding tests.

## CORE-034 accepted implementation

- Corrected authorization `91d2686`, tree `bd9116b2`, canonical diff `19458d57`,
  received three exact approvals and passes compiler `30915838213` / `30915838191`,
  Rust `30915839059`, CodeQL `30915834128`, and aggregate `92013770932`.
- Triple-approved tests-only `296276f`, tree `9b1ad9d1`, canonical diff `79b7ef9d`,
  reclassified both prior acceptance rows. Focused 0/1, binding 22/23, local full-
  gate and compiler `30916807388` / `30916811627` plus nightly Rust `30916810937`
  all isolate exactly 30 false acceptances after 139/139 library, 149/149 binary,
  and 7/7 claim passes. CodeQL `30916806193` passes. Three public-red reviews
  approved the two-phase implementation boundary.
- Exact implementation `a1ffeaec`, tree `f0088e65`, canonical diff `7a3fdb11`, adds
  only the semantic and checked-admission guards. It received three exact approvals;
  formatting, focused 1/1, binding 23/23, the full local gate at 139/139 library,
  149/149 binary, 7/7 claim, and 23/23 binding tests, compiler `30917539648` /
  `30917544307`, stable/nightly Rust `30917537292`, all three CodeQL analyses in
  `30917534448`, and aggregate `92019545168` pass.
- Classification is unchanged. This is exact fail-closed containment, not reference
  or tuple value/lowering/execution evidence. Tuples remain parsed-only, references
  remain partial, R-002 remains HIGH/CRITICAL and PARTIALLY CONTROLLED, R-011 remains
  open without a bounds policy, and no backend, matrix, capability, or stability
  class moves.
- The prepared six-record closure's fresh exact full gate exits 0 with 139/139
  library, 149/149 binary, 7/7 claim, and 23/23 binding tests.
- Exact closure `d3811b00`, tree `c01088c4`, canonical diff `2799eb32`, received
  three exact approvals and is public all-eight green in compiler `30918433816` /
  `30918438945`, Rust `30918439169`, CodeQL `30918434204`, and aggregate
  `92022619964`. CORE-034 is closed without changing a capability classification.

## AUDIT-041 authorization boundary

- The audit basis is exact clean public closure `d3811b00`; every accepted slice
  through CORE-034 is excluded. The complete remaining set remains R-002/R-004/
  R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016.
- Three independent reviewers must rank all eleven residuals from that immutable
  head, inherit no earlier candidate or order, and identify at most one exact bounded
  candidate or stop with evidence, reachability, containment, semantic decisions,
  phase count, deterministic failing specimen, and preservation controls.
- The authorization and audit cannot change implementation or classification.
  Tuples remain parsed-only, references remain partial, R-002 remains HIGH/CRITICAL
  and PARTIALLY CONTROLLED, R-011 remains open without a bounds policy, and every
  other capability, backend, artifact, and claim boundary remains unchanged.
- No ranking begins before the six-record authorization passes a fresh full local
  gate, three exact reviews, unchanged publication, and all eight public checks.
- The prepared authorization's fresh exact full gate exits 0 with 139/139 library,
  149/149 binary, 7/7 claim, and 23/23 binding tests. No capability class moves.
- Exact authorization `a31342e8`, tree `fbcd78b6`, canonical diff `313a1f6b`, is
  triple-approved and public all-eight green in compiler `30919164807` /
  `30919167478`, Rust `30919168162`, CodeQL `30919164869`, and aggregate
  `92025101785`.
- Three complete rankings all place R-002 first but initially select V valueless
  three-array, I initialized three-array, and R initialized positive-count immediate
  reference-array-tuple containment. Targeted comparison prefers R two to one; all
  three final compatibility reviews approve exact R with both mutability flags,
  34 red observations, four count-zero green observations, and a two-phase ceiling.
- AUDIT-041 is complete, read-only, and classification-neutral. Bounds remains
  stopped pending policy; V and I remain residuals rather than authority.

## CORE-035 authorization history

- Only initialized exact nonrecursive positive-count
  `Type::Reference(Type::Array(Type::Tuple(_), count), _)` may be rejected, after
  child and existing initialized diagnostics, at semantic and checked-admission
  boundaries. This is containment before IR, not reference, array, or tuple support.
- Tests-first was required to reclassify existing immutable acceptance evidence and
  expose exactly 34 false acceptances while four count-zero semantic/checked controls
  stayed green. Later implementation was limited to two exact guards in two phases.
- No capability class moves: tuples remain parsed-only, references and fixed arrays
  remain partial, R-002 remains HIGH/CRITICAL and PARTIALLY CONTROLLED, R-011 remains
  open without bounds policy, and backend/hardware/claim classes remain unchanged.
- A fresh local gate, three exact reviews, unchanged publication, and all eight
  public checks were required before the tests-only change and are satisfied by the
  acceptance evidence below.
- The prepared six-record authorization's fresh exact full gate exits 0 with 139/139
  library, 149/149 binary, 7/7 claim, and 23/23 binding tests. No capability moves.

## CORE-035 accepted implementation

- Authorization `b74b1d29`, tree `3fc2d78f`, canonical diff `64fbd1fe`, received
  three exact approvals and is public all-eight green in compiler `30921372203` /
  `30921374216`, Rust `30921376655`, CodeQL `30921371268`, and aggregate
  `92032740349`.
- Triple-approved tests-only `f04e80c9`, tree `03a9f274`, canonical diff
  `9e04b6ad`, reclassified both prior immutable acceptance rows. Focused 0/1,
  binding 23/24, local full-gate, public compiler `30922180824` / `30922181281`,
  and nightly job `92035312036` in Rust `30922181764` all isolate exactly 34 false
  acceptances after 139/139 library, 149/149 binary, and 7/7 claim passes. Stable
  job `92035312020` was fail-fast cancelled; CodeQL `30922176056` and aggregate
  `92035461619` pass. Three public-red reviews approved the two-phase boundary.
- Exact implementation `b8fd5a17`, tree `77bd2536`, canonical diff `2f1e9920`, adds
  only 21 semantic-analyzer and 15 checked-admission lines. It received three exact
  approvals; formatting, focused 1/1, binding 24/24, the full local gate at 139/139
  library, 149/149 binary, 7/7 claim, and 24/24 binding tests, compiler
  `30922853658` / `30922859177`, stable/nightly Rust `30922863203`, all three
  CodeQL analyses in `30922853619`, and aggregate `92037794056` pass.
- Classification is unchanged. The exact rejection is containment before IR, not
  reference, array, or tuple value/lowering/execution evidence. Tuples remain
  parsed-only; references and fixed arrays remain partial; R-002 remains
  HIGH/CRITICAL and PARTIALLY CONTROLLED; R-011 remains open without a bounds
  policy; and no backend, matrix, capability, artifact, or stability class moves.
- Exact closure `60ad91f7`, tree `978aa98f`, canonical diff `818a8112`, received
  three exact approvals and is public all-eight green in compiler `30923835957` /
  `30923837627`, Rust `30923838264`, CodeQL `30923834264`, and aggregate
  `92041128413`. CORE-035 is closed without changing a capability classification.

## AUDIT-042 authorization boundary

- Exact CORE-035 closure `60ad91f7`, tree `978aa98f`, canonical diff `818a8112`,
  received three exact approvals and is public all-eight green in compiler
  `30923835957` / `30923837627`, Rust `30923838264`, CodeQL `30923834264`, and
  aggregate `92041128413`. CORE-035 is closed without a capability classification
  change.
- The AUDIT-042 basis is that exact clean public closure. Every accepted slice
  through CORE-035 is excluded; the complete remaining set is R-002/R-004/R-005/
  R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016.
- Three independent reviewers must rank all eleven residuals from the immutable
  head, inherit no earlier candidate or order, and identify at most one exact bounded
  candidate or stop with evidence, reachability, containment, semantic decisions,
  phase count, a deterministic failing specimen, and preservation controls.
- The authorization and audit cannot change implementation or classification.
  Tuples remain parsed-only; references and fixed arrays remain partial; R-002 stays
  HIGH/CRITICAL and PARTIALLY CONTROLLED; R-011 remains open without bounds policy;
  and every backend, artifact, hardware-execution, and claim boundary is unchanged.
- Ranking could begin only after a fresh exact full local gate, three exact reviews,
  unchanged publication, and all eight public checks. Those prerequisites are
  satisfied by corrected authorization `2d8a0c54`; the completed audit remained
  read-only and selected at most one bounded candidate.
- The prepared authorization's fresh exact full gate exits 0 with 139/139 library,
  149/149 binary, 7/7 claim, and 24/24 binding tests. No capability class moves.
- First authorization snapshot `4ce0de0d`, tree `350984b8`, canonical diff
  `347278c3`, passed that gate but was rejected before independent push or branch-
  head publication because PROJECT_STATE retained the completed CORE-035 hypothesis
  as active and DEC-046 retained pending-closure status. The corrected authorization
  ancestry preserves that snapshot; ranking never began and no capability moves.

## AUDIT-042 accepted result

- Corrected authorization `2d8a0c54`, tree `45d1c184`, correction canonical diff
  `b36d3d9b`, and cumulative diff from CORE-035 closure `478e947a`, changed only six
  control records, passed two fresh exact full gates, received three fresh exact
  approvals, and was published unchanged. Compiler `30924946683` / `30924950615`,
  stable/nightly Rust `30924951134`, CodeQL `30924945035`, and aggregate
  `92044919183` pass.
- On that immutable public head, type/safety selected valueless exact nonrecursive
  `Reference(Array(Tuple))` U, IR/codegen selected valueless exact
  `Array(Array(Array(Tuple)))` T, and backend/claim selected direct nonnegative
  scalar-literal fixed-array bounds B. The first two reviewers ranked U > T > B and
  stopped B on unresolved compile-time-versus-runtime policy; backend/claim ranked
  B > U > T. The lead selected U two to one, and all three final compatibility
  reviews approved its exact two-phase boundary.
- AUDIT-042 performed no edit, test, build, formatter, probe, artifact, hardware
  action, or external query. It is complete and classification-neutral. Tuples remain
  parsed-only; references and fixed arrays remain partial; R-002 stays HIGH/CRITICAL
  and PARTIALLY CONTROLLED; R-011 remains open; no backend, hardware-execution,
  artifact, matrix, capability, or claim class moves.

## CORE-036 authorization boundary

- CORE-036 may only fail closed a valueless annotation exactly shaped as
  nonrecursive `Type::Reference(Type::Array(Type::Tuple(_), count), ref_flag)` in
  semantic analysis and checked IR admission. Both reference flags, all counts
  including zero, and all tuple arities are matched. Initialized and every deeper,
  wrapped, mixed, scalar, and numeric-array form remain outside the guard.
- Existing semantic duplicate precedence and four valueless tuple-shape diagnostics
  remain first. The exact new semantic and checked diagnostics distinguish an
  uninitialized binding and run before fallback insertion or raw generation. Existing
  traversal only is used; checked generic-function outer rejection and syntax-only
  generic trait defaults do not move.
- The tests-first change must reclassify all four existing exact-U acceptance
  occurrence blocks containing five source rows, expose exactly 34 false acceptances,
  and retain exactly 40 preservation observations. Before implementation, focused
  0/1 and binding 24/25 must be the only expected-red result after the 139/139,
  149/149, and 7/7 suites pass. After the separately reviewed public-red boundary,
  only the semantic analyzer and checked IR generator may change.
- This is proposed containment before IR, not supported value or lowering evidence.
  It defines no reference/array/tuple/default/mutability/ownership/lifetime/layout/
  ABI/bounds/execution/backend semantics. Every capability and claim classification
  remains unchanged; R-002 stays HIGH/CRITICAL and PARTIALLY CONTROLLED and R-011
  remains open. The prepared authorization's fresh full gate exits 0 with 139/139
  library, 149/149 binary, 7/7 claim, and 24/24 binding tests. The verification gate,
  three exact approvals, unchanged publication, and all eight public checks required
  before tests-first work are satisfied by `697bb3b4` below.

## CORE-036 accepted implementation

- Authorization `697bb3b4`, tree `b0cfd37b`, canonical binary diff `0a92ad7a`, is
  triple-approved and public all-eight green in compiler `30927281281` /
  `30927293459`, Rust `30927289178`, CodeQL `30927280707`, and aggregate
  `92052974430`.
- Triple-approved tests-only `d52b117e`, tree `76a3b2e9`, canonical binary diff
  `c2d5e46a`, reclassified all five exact-U rows. Local, push `30927952017`, PR
  `30927956714`, nightly `92055067840`, and stable `92055068009` test logs all pass
  139/149/7 then isolate exactly 34 acceptances as the sole binding 24/25 failure.
  CodeQL `30927952240` and aggregate `92055178151` pass; three public-red reviews
  approved implementation authority.
- Exact implementation `26d18924`, tree `8aec746c`, canonical binary diff
  `543f8a1c`, adds 17 semantic and 16 checked-admission lines only. It is triple-
  approved; formatting, focused 1/1, binding 25/25, the full local gate, compiler
  `30928759703` / `30928760789`, stable/nightly Rust `30928758562`, CodeQL
  `30928754859`, and aggregate `92057919831` pass.
- Classification is unchanged. The exact rejection is containment before IR, not
  reference/array/tuple/default/ownership/layout/ABI/bounds/lowering/execution
  evidence. Tuples remain parsed-only; references and fixed arrays remain partial;
  R-002 remains HIGH/CRITICAL and PARTIALLY CONTROLLED; R-011 remains open; no
  backend, hardware-execution, artifact, matrix, capability, or claim class moves.
- A six-record closure was prepared from clean public all-eight-green implementation
  `26d18924`. The additively corrected tree's fresh and verification exact full gates
  each exit 0 with 139/139 library, 149/149 binary, 7/7 claim, and 25/25 binding
  tests. Exact acceptance `3f042e18` below closes CORE-036.
- First closure snapshot `39c8564b`, tree `7932dd42`, canonical binary diff
  `2cb44b26`, passed two gates but was rejected before independent publication. Type/
  safety approved; IR/codegen and backend/claim found P1 stale PROJECT_STATE current-
  implementation authority at CORE-035 `b8fd5a17`. The corrected ancestry preserves
  that snapshot and now points to public CORE-036 implementation `26d18924`; no
  capability class moves.
- First additive correction `799c4181`, tree `1c8a883f`, canonical binary diff
  `9a1f5cd8`, received type/safety approval but IR/codegen rejected P1 because DEC-048
  still called the completed verification gate pending. Review stopped before
  publication. The second additive correction aligns that status with the recorded
  two green gates and moves no capability class.
- The second additive correction's fresh exact full gate exits 0 with 139/139 library,
  149/149 binary, 7/7 claim, and 25/25 binding tests, plus all downstream suites.

## CORE-036 closure and AUDIT-043 boundary

- Exact closure `3f042e18`, tree `15d56e0c`, canonical binary diff `ee8cbed0`, changed
  only the six control records, received three exact approvals, and is public all-
  eight green in push CI `30930377220`, PR CI `30930379386`, stable/nightly Rust
  `30930380195`, all three CodeQL analyses in `30930375201`, and aggregate
  `92063404658`. CORE-036 is closed with no capability movement.
- Preregistered AUDIT-043 may only re-rank the complete remaining eleven-risk set
  from exact clean public closure `3f042e18`, excluding every accepted slice through
  CORE-036 and inheriting no U/T/B label or order. It is static and read-only after a
  separate six-record authorization is locally green, triple-approved, published
  unchanged, and public all-eight green.
- The audit defines no language, type, ownership, memory, aggregate, ABI, bounds,
  lowering, execution, tooling, backend, accelerator, or claim semantics. Bounds B
  remains stopped on unresolved compile-time-versus-runtime policy. Tuples stay
  parsed-only, references/fixed arrays stay partial, R-002 remains HIGH/CRITICAL and
  PARTIALLY CONTROLLED, R-011 remains open, and no capability class moves.
- Pre-acceptance evidence at corrected snapshot `5276df5b` (historical; superseded by
  the result below): the prepared six-record authorization's fresh and verification
  exact full gates each exited 0 with 139/139 library, 149/149 binary, 7/7 claim, and
  25/25 binding tests, plus all downstream suites. At that point exact review,
  unchanged publication, and public all-eight acceptance remained pending; no ranking
  had begun.
- First authorization snapshot `cb43d1bb`, tree `f0f19f5d`, canonical binary diff
  `ead99a7b`, passed both gates but all three reviewers rejected P1 because DEC-049
  status still called them required; type/safety also found stale “next immutable
  snapshot” wording. Nothing was published and no ranking began. The additive
  correction changes no capability class or authority.
- Pre-acceptance additive correction evidence (historical; superseded below): its
  fresh exact full gate exited 0 with 139/139 library,
  149/149 binary, 7/7 claim, and 25/25 binding tests, plus all downstream suites.
  At that point fresh review and public acceptance remained pending; no capability
  class moved.

## AUDIT-043 result and ARCH-001 boundary

- Corrected authorization `5276df5b`, tree `c3eaf3cf`, correction diff `b8b7586f`,
  cumulative diff `fe5376dc`, is triple-approved and public all-eight green in push
  CI `30931510621`, PR CI `30931515125`, Rust `30931515426`, CodeQL `30931509579`,
  and aggregate `92067252294`.
- Complete rankings initially selected R-009 once and exact valueless three-array-
  tuple R-002 twice. Final compatibility unanimously selects R-002 only after a
  separate behavior-neutral shared classifier closes green. R-009 remains bounded
  fallback; R-011 remains stopped. The audit was read-only and moves no capability.
- ARCH-001 may classify only exact current rejection, contract-shape routing, and
  preserved behavior from annotation topology plus initializer presence. It must be
  nonrecursive, diagnostic-free, inference-free, and behavior-neutral across semantic
  and checked boundaries; generic/context gates remain external. The selected R-002
  shape must remain accepted. No reference/array/tuple/default/ownership/layout/ABI/
  bounds/lowering/execution/backend support or capability evidence follows.
- Its pre-acceptance fresh and verification exact authorization gates (historical;
  superseded below) each exited 0 with 139/139 library, 149/149 binary, 7/7 claim,
  and 25/25 binding tests, plus all downstream suites. At that point review,
  publication, and public acceptance remained pending; no capability moved.
- First ARCH-001 snapshot `63d8d599`, tree `28cd120c`, diff `9fef5adf`, was not
  published after a valid P1 chronology rejection: five records retained superseded
  AUDIT-043 pending/no-ranking evidence in present tense. The additive six-record
  correction makes it historical and changes no capability or classifier boundary.
- The additive correction's pre-acceptance fresh and verification exact full gates
  (historical; superseded below) each exited 0 with
  139/139 library, 149/149 binary, 7/7 claim, and 25/25 binding tests, plus all
  downstream suites. At that point fresh review and public acceptance remained
  pending; no capability moved.
- Exact `1dcfd869`, tree `b537023c`, correction diff `e5ee8aa7`, cumulative diff
  `5208cb6e`, is triple-approved and public all-eight green in compiler CI
  `30934518525` / `30934523152`, Rust `30934523078`, CodeQL `30934519513`, and
  aggregate `92077350363`. Authorization acceptance moves no capability; a separate
  green characterization boundary becomes eligible only after the six-record
  acceptance sync below is accepted public all-eight green.
- The six-record acceptance sync's fresh and verification exact full gates each exit 0
  with 139/139 library, 149/149 binary, 7/7 claim, and 25/25 binding tests, plus all
  downstream suites. Exact review and public sync acceptance remain pending; no
  capability moves.
- First acceptance-sync snapshot `4c18450a`, tree `ea7b91c9`, diff `7be565db`, was
  not published after type/safety found one P1: three records prematurely said
  characterization was already eligible while the sync remained pending. The
  additive six-record correction restores the exact gate and moves no capability.
- Its fresh and verification exact full gates each exit 0 with 139/139 library,
  149/149 binary, 7/7 claim, and 25/25 binding tests, plus all downstream suites.
  Fresh review and public acceptance remain pending; no capability moves.

## CORE-065 accepted conditional enum ownership boundary

- The admitted state is limited to existing local and parameter owners of the exact
  non-Copy enum schemas accepted by CORE-063/064. Each acyclic `if` sibling starts from
  one entry snapshot; branch-local shadows do not alter the outer join, and multiple
  owners join independently.
- Reachable `Owned`/`Moved` states join exactly. A mixed reachable state becomes
  `MaybeMoved`; later use, borrow, move, Match, call, return, assignment-source use, or
  target replacement rejected deterministically at this checkpoint. A missing `else` contributes the
  entry state, while a definitely returning arm contributes no fallthrough state.
- Loop-condition consumption and ownership changes reaching a loop backedge reject
  through the same classifier. Loop fixed points, `break`/`continue` transport,
  conditional reinitialization, and general CFG ownership were not admitted. Accepted
  CORE-073 later supersedes only the exact acyclic whole-owner
  reinitialization boundary; loop-contained reinitialization remains rejected.
- The verifier independently maps enum-valued loads back to their owning place and
  computes consumed-owner unions through the checked CFG. It accepts mutually
  exclusive sibling consumption and exact place replacement, while rejecting serial,
  post-partial-merge, cyclic, and unreplaced-place double consumption.
- Accepted evidence is 182 library and 188 binary tests plus the focused source/direct-
  module/IR/verifier/LLVM/CLI target, formatting, all-target/all-feature checking,
  correctness Clippy, docs, and the exact root gate. All eight public checks and pinned
  LLVM/Clang 22 native exit 137 pass on exact implementation `f4daeea`.

## CORE-064 accepted owned-enum reassignment boundary

- The admitted target is an initialized mutable local whose exact logical type is
  either recursive finite CopyData or one already admitted non-Copy enum schema. One
  classifier owns this distinction across semantic analysis and checked admission.
- Enum replacement accepts an exact constructor, exact enum-returning call, or a
  distinct initialized enum local. The distinct source is moved; the target becomes
  owned and initialized; direct self-replacement rejects. Existing conservative
  source-order ownership remains unchanged.
- `CheckedMutableOwnedPlaceAlloca` and `CheckedOwnedPlaceAssignment` replace the former
  CopyData-named identities for both classes. Independent verification requires exact
  type/schema, adjacent one-time initialization, dominance, collision freedom, and
  checked later writes. Borrow identities continue to accept CopyData only.
- Verified LLVM uses exact private enum allocas, loads, and stores. It introduces no
  fallback `i32`, byte layout, bitcast, public discriminant, stable layout, ABI, or FFI.
- Accepted evidence is 180 library and 186 binary tests plus formatting,
  all-target/all-feature checking, correctness Clippy, docs, the exact root gate, and
  the exhaustive source/IR/verifier/LLVM/CLI target. All eight public checks and the
  pinned LLVM/Clang 22 external/machine/object/link/native exit-131 lane pass on exact
  implementation `79aed71`.

## CORE-063 accepted unary recursive CopyData enum boundary

- The exact selected grammar is a unique nongeneric nonempty enum whose variants are
  unit or contain exactly one value from the accepted recursive `CopyData` grammar.
  Enum values remain non-Copy. Match results were scalar at this checkpoint; local
  candidate CORE-074 later supersedes only the exact fresh owned-enum result class.
- `EnumRegistry` delegates payload annotation classification to `StructRegistry`;
  semantic initialization/preflight/inference and checked admission consume resolved
  arm binding types instead of scalar placeholders or topology-specific rules.
- Checked enum construction, payload extraction, dispatch, parameters, calls, returns,
  and schema registration retain exact recursive types. The verifier rejects unsupported
  nested leaves, conflicting named schemas, scalar fallback payloads, and changed lane
  identity before trusted LLVM generation.
- Unit-only and scalar-only schemas preserve their accepted private layout. Schemas
  containing aggregate payloads use a private tag plus one exact typed lane for each
  payload-bearing variant; inactive lanes are typed zero values. No public ABI follows.
- The exact local gate is green at 179 library and 185 binary tests. All eight public
  checks pass on verified head `bebd0b6`; pinned LLVM/Clang 22 externally verifies,
  machine-verifies, object-lowers, explicitly links the private non-PIE executable, and
  records exact native exit 113. No stable layout/ABI claim follows.

## CORE-062 accepted recursive CopyData boundary

- The exact admitted grammar is `Int | Float | Bool | [CopyData; N] | tuple` of at
  least two CopyData elements `|` finite acyclic named struct with unique admitted
  CopyData fields. It is least-fixed-point recursive and depth agnostic. Exact array
  count, tuple order/arity, struct identity, and declaration-ordered fields are schema.
- One `StructRegistry` contract resolves both source `Type` and semantic `Ty` to exact
  `LogicalType`. Tuple, array, struct, binding, function, Copy-place, semantic, checked-
  admission, verifier, and backend consumers no longer keep executable per-container
  scalar/flat/numeric/Copy-struct whitelists. The historical broad `Ty::is_copy_type`
  is quarantined from trusted execution; immutable references remain separately Copy
  only for established ownership tracking.
- Positive evidence covers every immediate constructor pairing; Bool/nested arrays;
  aggregate-bearing tuples and structs; zero arrays; inferred/exact bindings; whole
  aliases/reassignment; immutable/mutable whole references; calls, results, forwarding,
  terminating recursion; dynamic fixed-array indices; chained field/tuple/index
  projection; flattened direct modules; deterministic checked IR/LLVM; and local native
  exit 109. The exact root gate passes 178 library and 184 binary tests plus all
  integration, claim, Phase 5, and doc controls.
- Negative evidence keeps unit/unary tuples, String/reference/function/closure/enum/
  generic/trait/collection leaves, malformed or cyclic structs, aggregate comparison,
  projected borrowing/writing, exact mismatches, constant out-of-bounds indexing, raw
  checked-IR bypass, schema corruption, and requested-artifact hygiene fail closed.
- This accepted slice establishes neither stable layout/ABI/FFI nor general ownership,
  lifetime/drop, memory safety, accelerator execution, performance, release, or
  stability. Exact implementation `e62fd747` passes all eight public checks and the
  pinned LLVM/Clang 22 external/machine verification, object/link, and exact exit-109
  system lane; PR #4 remains draft and unmerged.
