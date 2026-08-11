<div align="center">
  <h1>Aero</h1>
  <p><strong>Experimental systems language and compiler repository</strong></p>
  <a href="https://github.com/RobVanProd/Aero/stargazers">
    <img src="https://img.shields.io/github/stars/RobVanProd/Aero?style=social" alt="GitHub stars">
  </a>
  <a href="https://opensource.org/licenses/MIT">
    <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="MIT License">
  </a>
  <a href="https://github.com/RobVanProd/Aero/actions/workflows/ci.yml">
    <img src="https://github.com/RobVanProd/Aero/actions/workflows/ci.yml/badge.svg" alt="CI Status">
  </a>
</div>

Aero contains a compiler, language examples, benchmark harnesses, and
experimental GPU/runtime interfaces. This README only lists benchmark claims
that are backed by tracked artifacts under
[`claim-verification/`](claim-verification/).

The founding direction is preserved in the
[original language design paper](<__Aero___%20A%20High-Performance,%20Ergonomic%20Programming%20Language.pdf>)
and the [strategy capture](<Aero%20Programming%20Language%20Framework%20-%20Claude.pdf>).
[Framework alignment](FRAMEWORK_ALIGNMENT.md) explains how that direction maps
to current evidence and the active roadmap. Aero is presently an experimental
minimal prototype under correctness recovery; historical `v1.0.0` labels are not
evidence of a stable release.

## Verified Results

The latest public-branch verification was run on 2026-05-28 at commit
`7d6ad2f865560cdcca4e30390430a7878c65fa69` on this local machine:

- CPU: AMD Ryzen 9 9950X 16-Core Processor
- GPU 0: Radeon RX 7900 XTX (`gfx1100`, PCI device `1002:744c`)
- GPU 1: AMD Radeon Graphics (`gfx1036`, PCI device `1002:13c0`)
- PyTorch: `2.5.1+rocm6.2`
- HIP: `6.2.41133-dd7f95766`

Verified current results:

- The public-branch run completed Rust Criterion lexer-only benchmarks. The reported
  median tokenization times ranged from 282.31 ns for `tokenize_simple_io` to
  21.507 us for `tokenize_large_program`.
- The GGUF harness completed a real local llama.cpp CLI ROCm inference run with
  `HIP_VISIBLE_DEVICES=0` on the Radeon RX 7900 XTX. The model was
  `/home/rob/models/mistralai_Mistral-Small-3.1-24B-Instruct-2503-Q4_K_M.gguf`;
  one measured run reported 44.94 eval tokens/s, 186.94 prompt eval tokens/s,
  and 2701.84 ms load time
  ([result JSON](claim-verification/results/aero_gguf_llama_cli_7900xtx_20260528T224200Z/claim_result.json)).

Invalid compilation measurements:

- `benchmarks/performance_benchmark.py` passed each bare source path directly to
  the Aero executable, without a required command such as `build`. The resulting
  public-master and post-reboot timing series measured the process's successful
  usage fallthrough, not Aero compilation, and are classified as invalid. Their
  raw artifacts remain available for audit; none of those compilation numbers is
  a verified performance result.

Historical lexer evidence:

- An earlier local lexer artifact from commit
  `f263568e06073317467416c2f954c73927c00f3e` is tracked as a historical
  artifact. Its Criterion lexer tokenization times ranged from 340.38 ns to
  22.956 us. This historical lexer record is separate from the invalid Python
  timing series and is not the latest public-branch verification
  ([result JSON](claim-verification/results/aero_post_reboot_7900xtx_20260528T190000Z/claim_result.json)).

Blocked or omitted claims:

- GPT-2 training vs PyTorch is omitted because this repo does not contain a
  fresh Aero GPT-2 training artifact from the current public branch.
- GPU 4096x4096 Aero matmul speedup is omitted because no current public-branch
  Aero matmul artifact or rerun verified it.
- NCCL/MPI multi-GPU scaling is omitted because no current public-branch
  multi-GPU scaling artifact or rerun verified it.
- Aero-vs-llama.cpp-vs-PyTorch GGUF comparison claims are omitted because this
  verification captured only a single llama.cpp CLI ROCm backend run.
- HIP/vector-add claims are omitted here because no current Aero artifact or
  rerun in this repo verified them.

## 📦 Quick Start

Use a POSIX shell from a machine with Rust and the documented LLVM 22/Clang
[build prerequisites](BUILD.md#prerequisites):

```bash
git clone https://github.com/RobVanProd/Aero.git
cd Aero
cargo build --release --manifest-path src/compiler/Cargo.toml
export PATH="$PWD/src/compiler/target/release:$PATH"
aero --version
aero init my_app
cd my_app
aero check src/main.aero
aero run src/main.aero
```

The generated `src/main.aero` is:

```aero
fn main() {
    println!("Hello, Aero!");
}
```

The final command must complete successfully and print `Output: Hello, Aero!`.
See the [Windows PowerShell instructions](BUILD.md) for the equivalent Windows
build and PATH commands, and consult the [backend capability status](BACKEND_STATUS.md)
before using an accelerator target.

## Experimental Command Examples

These commands are outside the minimal Quick Start. They expose experimental
compiler surfaces with the limitations recorded in the current compiler table
below and in [backend capability status](BACKEND_STATUS.md). They are not evidence
that every backend performs device execution.

CPU is the only current process-execution target. A ROCm `run` request can probe
temporary object emission, then returns unavailable because there is no HIP link
or device launch. CUDA has no active object, link, or launch path. The ambiguous
`gpu` target is rejected; select `cpu`, `rocm`, or `cuda` explicitly.

Graph optimization is a verified textual rewrite to internal scalar helpers;
backend labels do not establish device execution. Quantization uses
scalar-`double` helpers: there is no real FP8 representation, per-channel
execution, or numerical correctness proof.

```bash
aero run --target cpu src/main.aero
aero build src/main.aero -o main.rocm.ll --target rocm --gpu gfx1100
aero doc src/main.aero -o main.md
aero profile src/main.aero -o trace.json
aero graph-opt main.ll -o main.opt.ll --backend rocm --gpu gfx1100
aero quantize main.opt.ll -o main.int8.ll --mode int8 --backend rocm --gpu gfx1100 --calibration calib.json
python benchmarks/gguf/gguf_compare.py --config benchmarks/gguf/config.rx7800xt.example.json
aero registry search vision --index registry/index.json
aero registry publish . --dry-run
aero registry install vision-core --version 0.2.0 --target pkgs --dry-run
aero conformance -o conformance_report.json
aero lsp
```

## 🛠️ Current Compiler Surface (Experimental)

| Category | Features |
|----------|----------|
| **Type System** | Static scalar checks. Accepted CORE-072 adds exact Unicode `char` identity and equality/inequality across the complete existing recursive CopyData transport class. Accepted CAP-004 adds one explicit user-defined recursive-CopyData generic-struct substitution class. General generic functions/enums/impls/traits, inference/defaults, trait-bound enforcement, and where-clause semantics remain parsed, quarantined, or unsupported. |
| **Memory** | Shallow move tracking plus bounded, publicly accepted whole-place immutable and mutable references and direct reassignment over the exact admitted recursive CopyData universe. Accepted CORE-064–079 add bounded owned-enum replacement, joins, reinitialization, Match results, and loop ownership. Accepted CORE-083–089 add non-escaping enum-reference replacement/observation and mixed/multiple-reference callable signatures. Accepted CORE-090 adds exact static writes through arbitrary finite field/tuple/fixed-array paths rooted at a mutable owned direct local CopyData value. Accepted CAP-002 adds guarded runtime indexes in that same direct-owner write class. Projected borrowing, reference-target dynamic writes, partial moves, free enum dereference/transport, enum aggregate storage, and general aliasing remain unsupported. No general borrow checker, general mutable-reference model, lifetime analysis, drop model, stable pointer ABI, or memory-safety guarantee. Reference results remain unsupported. |
| **Data Types** | Recursive finite CopyData composition and bounded positional recursive CopyData owned enums—including exact variants with two or more fields—are publicly accepted with exhaustive identifier-bound Match, internal transport, exact mutable whole-owner replacement/reinitialization, acyclic conditional ownership joins, and fresh per-iteration loop-local owners. Accepted CORE-072 adds `char`; CORE-074–076 add typed Match results; and CORE-083–089 add bounded enum-reference and callable compositions. Accepted CORE-090 composes existing named fields, tuple constants, and integer-literal fixed-array indexes into exact mutable CopyData paths without adding a data topology. Accepted CAP-001 adds guarded runtime reads, CAP-002 adds guarded runtime-indexed writes over the same nonempty recursive CopyData fixed-array class, CAP-003 adds explicitly typed concrete recursive-CopyData `Option<T>`/`Result<T,E>` construction, owned transport/replacement, and exhaustive bound Match, and CAP-004 adds explicit user-defined recursive-CopyData generic structs with deterministic substitution and checked identity/schema verification. `print!` and `println!` are effect-only `Void`, not scalar values. Named-field/generic enum variants, general generic substitution/error propagation, wider patterns, carrier or enum aggregate/reference storage, enum fields/arrays, free enum dereference or transport through references, dynamic collections, projected borrowing or partial moves, unsupported/cyclic structs, and broader storage or destructuring semantics remain unsupported. |
| **Control Flow** | Functions, if/else, while/checked fixed-array for/loop, and nearest-loop break/continue are partial. Accepted CORE-066 corrects checked `for` continue and proves fresh per-iteration enum consumption. Accepted CORE-077 adds exact balanced direct enum-owner restoration. Accepted CORE-079 iterates direct-enum header and exit joins to convergence across condition/iterable, fallthrough/continue, and break edges, with return paths excluded and nested transfers attributed to the nearest loop. Labels, loop expressions/break values, non-array checked iterators, non-enum fixed points, and general CFG ownership remain unsupported. Closure syntax is parsed-only; executable closure expressions fail closed before checked IR. |
| **Function calls** | Accepted CORE-068 centralizes exact named-call classification across both semantic paths and checked admission/lowering. Existing nongeneric functions over admitted scalar, recursive CopyData, owned-enum, and reference contracts remain supported; accepted CORE-087 composes one mutable reference with CopyData sides, CORE-088 adds immutable-reference sides, and CORE-089 admits two or more pairwise-disjoint mutable references with any immutable and CopyData sides in every declared order. CORE-090 permits already admitted calls as projected-assignment RHS values but does not add projected arguments. Accepted CAP-003 transports exact concrete recursive-CopyData `Option`/`Result` values, and CAP-004 transports exact concrete user-defined generic CopyData structs, through nongeneric internal parameters and results. Missing or unsupported signatures, wrong arguments, mutable-root overlap with any other argument, and `Void` value use fail before checked IR. Overloads, conversions, generic functions, general trait/closure calls, projected arguments, reference results, question-mark propagation, and stable callable ABI remain unsupported. |
| **Intrinsic methods** | Accepted CORE-067 centralizes intrinsic method classification across semantics and checked IR. Exact recursive CopyData fixed-array `.len()`/`.is_empty()`, immutable compile-time String queries, and Array/Vec `.iter()` compatibility are the only admitted executable method forms. Runtime Strings, other collection methods, general dispatch, generic/trait methods, and callable ABI remain unsupported. |
| **Direct module source collection** | Root-level `mod x;` collects `x.aero` or `x/mod.aero` into the current flattened compilation unit. Accepted CORE-070 adds public library `compile_file(path, options)` over this exact collector and the checked library frontend; it returns in-memory LLVM and writes no artifact. Accepted CORE-071 preserves Rust-like `use` syntax and source locations but rejects executable use before checked IR. Accepted CORE-080 likewise preserves the founding direct/aliased dotted `import` syntax with a distinct AST identity and fail-closed diagnostic. Accepted CORE-081 makes the collector and compiler phases library-owned while preserving current flattened behavior. Positive import/name-resolution, `pub` visibility semantics, namespaces, recursive modules, cycle graphs, and separate compilation are not implemented. |
| **Codegen** | LLVM IR backend with optimization passes |
| **CLI** | `aero build`, `aero run`, `aero check`, `aero test`, `aero fmt`, `aero doc`, `aero profile`, `aero graph-opt`, `aero quantize`, `aero registry`, `aero conformance`, `aero init`, `aero lsp`. Accepted CORE-081 removes the binary's duplicate compiler-phase module graph so these surfaces share the canonical library compiler implementation. |
| **LSP** | Syntax diagnostics, completion, hover, go-to-definition, document symbols |
| **Docs & Profiling** | Markdown API generation (`aero doc`), compilation stage timing + trace export (`aero profile`) |
| **Phase 8 Experimental Slice** | Textual graph rewriting to internal scalar helpers and scalar-`double` quantization helper rewriting with backend metadata. These are not device execution, real FP8/per-channel execution, or numerical-correctness evidence. The slice also includes local `registry.aero` search and dry-run planning plus 3 example cases and 4 deterministic regression checks (not formal-semantics proof). Live registry transport is quarantined pending a reviewed protocol and trust boundary. |
| **Diagnostics** | Colored errors, source snippets, "did you mean?" suggestions |

> **Project status after CAP-004:** Aero remains a Minimal Prototype in correctness
> recovery, not a complete or stable language. Accepted public and compiler-capability
> baseline is protected CAP-004 merge
> `e4c515b9566a7d8fcb4f66c975c4e1769607515f`.
> The corrective roadmap prioritizes one
> growing representative scalar application over more neighboring reference/topology partitions. Positive
> import/name resolution remains high leverage but requires separately frozen namespace
> and graph semantics first.

> **CAP-005 candidate, not accepted public capability:** the active bounded candidate
> adds compile-time specialization for unique top-level bound-free generic functions
> over exact recursive finite CopyData substitutions. Abstract type-parameter values
> may only be transported whole through direct parameters, explicitly typed locals,
> branch selection, reassignment, and return. The representative telemetry application
> uses one `choose<T>` helper for both `Reading<int>` and `Reading<char>` while
> preserving exact output and exit 91. Focused 5/5, shared-contract 2/2, corruption,
> compatibility, complete 232-library/32-binary/integration/doc/format/Clippy, and
> pinned LLVM/Clang 22.1.8 verification plus native `-O0`/`-O2` gates pass locally.
> Bounds/traits, generic methods/enums/impls, recursive or generic-to-generic calls,
> operations on abstract values, result-only inference, nested generic signatures,
> non-CopyData arguments, collections, lifetimes/drop, stable ABI, and general generic
> or safety claims remain unsupported. The compiler-surface table above continues to
> describe accepted CAP-004 until protected CAP-005 integration is complete.

> **CAP-004 accepted:** one unique nonempty user-defined generic struct can be
> instantiated with exact explicit recursive finite CopyData arguments. One shared
> idempotent authority performs deterministic substitution before semantic analysis and
> independent checked admission; verifier controls bind each private concrete identity
> to canonical source spelling and its exact field schema. Multiple instantiations,
> concrete nesting, existing aggregate/mutation/reference composition, and nongeneric
> function transport execute. The representative telemetry application uses
> `Reading<int>` and `Reading<char>` while preserving score 91. Focused 5/5, contract
> 4/4, corruption, compatibility, complete 229-library/32-binary local root, formatting,
> and Clippy gates pass. Exact candidate `a1554cab`, all nine candidate checks, protected
> PR #28 merge `e4c515b9`, exact merge-head CI/Rust CI/CodeQL, and pinned stable
> Linux/Windows LLVM/Clang 22.1.8 `-O0`/`-O2` execution pass. Generic functions/enums/
> impls/traits, inference/defaults, applications inside generic definitions, non-CopyData
> arguments, collections, heap/drop/lifetimes, stable ABI, and memory-safety claims
> remain unsupported.

> **CAP-003 accepted:** explicitly typed concrete recursive-
> CopyData `Option<T>` and `Result<T, E>` values can be constructed, moved, replaced,
> transported through nongeneric internal functions, and exhaustively matched in the
> accepted bounded class. All four constructors require an exact binding, assignment,
> parameter, or result context; missing type arguments never default. The
> representative application executes success and error branches while preserving
> score 91. Focused, adjacent, representative, and complete 224-library/32-binary
> local gates are green. Exact candidate `6a20ecfc`, all nine public checks, protected
> PR #26 merge `e6677941`, exact merge-head workflows, and pinned Linux/Windows LLVM
> 22.1.8 `-O0`/`-O2` gates pass. General generics,
> error propagation, carrier aggregate/reference storage, String errors, stable ABI,
> and memory-safety claims remain unsupported.

> **CAP-002 accepted:** runtime `int` selectors can update the existing
> mutable owned direct-local recursive CopyData projection class. Selectors execute
> exactly once in source order before the RHS, and each runtime selector uses the
> CAP-001 nonnegative/below-count guard before later selectors, RHS effects, address
> formation, or memory access. The representative telemetry application now fills its
> sensor array in a bounded loop. Focused 5/5, representative 3/3, and the complete
> 220-library/32-binary local gate pass. Exact candidate `577e601` passed all nine
> checks, protected PR #23 merged it as `62ccc6a`, and exact merge-head CI, Rust CI,
> and CodeQL pass. Writes through
> references, projected borrowing, partial moves, collections, stable trap/ABI
> semantics, and general memory safety remain unsupported.

> **M1-001 accepted:** the tracked multi-file telemetry-policy
> application now composes accepted functions, constants, control flow, aggregates,
> enums/`Match`, mutation, references, and projected writes. Local public
> `check`/verified `build`/`run`, independent LLVM/machine verification, the three-case
> compile-fail corpus, exact Windows LLVM/Clang 22.1.8 `-O0`/`-O2` output and exit 91,
> focused 3/3, and the complete 218-library/32-binary root gate pass. Its backend fix
> preserves numeric `print!`/`println!` values as typed LLVM `double` arguments under
> an explicit variadic `printf` call type instead of the prior Windows raw-`i64`
> workaround. Exact candidate `e7a74e6` passed all nine checks, merged through protected
> PR #19 as `d7d1c768`, and passed post-merge CI, Rust CI, and CodeQL. The composed
> workflow is `END_TO_END`; its component features remain `PARTIAL`. This is not a stable
> grammar, public ABI, general ownership/memory-safety, performance, or release claim.

> **CAP-001 accepted:** nonconstant `int` reads from
> the existing nonempty recursive CopyData fixed-array class now use one ordered
> nonnegative/below-count guard. Failure traps before conversion or address formation;
> success performs the typed `inbounds` access. Constant bounds diagnostics are
> unchanged. The representative telemetry application now uses computed indexes and
> retains exact local Windows `-O0`/`-O2` output/exit 91; negative and equal-to-count
> runtime specimens build and machine-verify. Focused 4/4, representative 3/3, all 218
> library tests, and the complete root gate pass. Exact candidate `71b0db4` passed all
> nine checks, protected PR #21 merged it as `25c1e2b`, and exact merge-head CI, Rust
> CI, CodeQL, Linux stable, and pinned Windows LLVM 22 gates pass.
> Dynamic writes, projected borrowing, collections, stable trap/status/ABI semantics,
> and general memory safety remain unsupported.

> **Closure status:** closures are parsed but unsupported in executable code. The
> opening `|` location is retained and `check`, `build`, and `run` reject closure use
> with one source-located diagnostic before checked IR. The legacy lowering fallback
> that manufactured callable identities and mapped unknown parameter/result types to
> `i32` is removed. Captures, calls, storage/transport, callable ABI, lifetime behavior,
> generics, and closure LLVM generation are not implemented.

> **Character status:** accepted public CORE-072 admits one exact Unicode scalar per raw
> single-quoted literal or the frozen eight escape forms. `char` remains distinct from
> `int` and `bool` through semantic/checked identity and lowers privately to `i32`.
> Equality/inequality and the complete existing recursive CopyData transport surface
> execute under a pinned LLVM/Clang 22.1.8 exit-197 gate. Arithmetic, ordering, casts,
> strings/printing, executable literal patterns, generic behavior, stable ABI/FFI,
> accelerators, and broader character semantics remain unsupported.

> **Import-declaration status:** accepted public CORE-071 keeps parsed Rust-like direct,
> optional-`as`, and terminal-glob `use` declarations for future work. Accepted CORE-080
> additionally preserves the founding direct and optional-`as` dotted `import` grammar.
> The AST retains each exact keyword location and distinguishes the two syntax families.
> Semantic analysis and independent checked admission reject executable declarations
> through one syntax-aware authority before checked IR; library and CLI routes leave no
> requested or native artifact. Neither checkpoint defines lookup, bindings, namespaces,
> alias/glob meaning, visibility, re-exports, recursion, cache identity, backend, or
> runtime behavior. CORE-080 passes all nine exact-head public checks.

> **Binding-annotation architecture:** Accepted public ARCH-002 normalizes each annotation
> to one leaf plus an ordered array/reference wrapper path and shares the resulting
> supported, explicitly rejected, or preserved/quarantined disposition across semantic
> analysis and checked admission. This is behavior-neutral: it adds no annotation,
> tuple, reference, generic, ownership, layout, ABI, or backend capability. Exact-head
> parity, all eight public checks, and the unchanged pinned native exit-193 lane pass.

> **Tuple status:** CORE-058 is publicly accepted for flat
> immutable tuples of arity two or greater whose elements are exactly `Int`,
> `Float`, or `Bool`. It covers inferred or exact bindings, Copy aliases, constant
> in-bounds projection, scalar/tuple-only internal calls and returns, direct modules,
> checked tuple identities, independent verification, typed literal-aggregate LLVM,
> and native exit 23. Unit/unary/nested or non-scalar tuples, mutation,
> destructuring, containers/fields/payloads/references, generic/impl/closure contexts,
> tuple-bearing `main`, public layout/ABI/FFI, drop, accelerator, performance, release,
> and stability claims remain unsupported. Exact implementation `421a0a9` passes all
> eight public checks; pinned LLVM/Clang 22.1.8 externally verifies, machine-verifies,
> object-lowers, links, and records exact native exit 23 with 171 library and 177
> binary tests.

> **Struct value status:** one bounded scalar-struct value slice and its
> all-component-`Copy` internal function-transport extension are publicly accepted.
> A unique,
> non-generic, nonempty top-level struct with unique `int`/`i32`, `float`/`f64`, or
> `bool` fields can be constructed exactly by field name inside an admitted
> top-level function; construction fields may be reordered and are evaluated once
> in written order. Direct and local immutable projection use checked aggregate IR
> and verified LLVM named types. Such a struct is Copy because every admitted field
> is Copy: local aliases preserve the original, and exact-name internal function
> parameters, arguments, call results, and returns use checked named aggregates by
> value. This includes mixed scalar/struct signatures, forwarding, terminating direct
> recursion, and the flattened one-level direct-module route. Unsupported definitions,
> shapes, annotations, and contexts remain rejected before LLVM.
>
> The publicly accepted CORE-045 slice also permits fixed local arrays of one exact
> admitted all-scalar Copy struct. It covers literal, repeat, and typed-empty origins;
> element-wise Copy aliases; static length; compile-time constant in-bounds indexing
> and projection; and compiler-bounded iteration. Exact struct schema and count survive
> distinct checked array IR into typed `[N x %aero.struct.Name]` LLVM. The tracked
> multi-file example composes this with direct-module collection and passes pinned
> LLVM/Clang 22 verification, lowering, linking, and native exit 77.
>
> The publicly accepted CORE-046 slice extends only the compiler's existing flat fixed
> `int`/`float` and all-scalar Copy-struct arrays across non-`main` internal function
> parameters and returns. Exact element identity, count, and struct schema remain in
> shared source classification, logical checked IR, verification, and aggregate LLVM
> definitions, calls, loads, stores, and returns. Caller values remain usable after
> the by-value call. The full local repository gate and all eight public checks pass;
> pinned Linux LLVM/Clang 22 externally verifies, lowers, links, and executes the
> multi-file system example with exact native exit 91.
>
> The publicly accepted CORE-047 slice composes those accepted Copy components into
> unique, non-generic, nonempty acyclic named aggregate graphs. Fields may be admitted
> scalars, another admitted named struct, or a flat fixed numeric/struct array; forward
> references and arbitrary finite named depth are resolved by one graph classifier.
> Construction, independent Copy aliases, chained projection, array operations through
> fields, internal parameters/results, and flat arrays of the new structs retain exact
> recursive schemas through checked IR and LLVM. Its exhaustive target, tracked direct-
> module example, and full repository gate pass at 157 library and 163 binary tests;
> all eight public checks pass, and pinned Linux LLVM/Clang 22 externally verifies,
> machine-verifies, object-lowers, links, and executes exact native exit 107.
>
> This does not provide non-Copy or destructive move semantics, projected/partial aggregate assignment, general
> methods beyond the exact array `.len()`/`.iter()` forms,
> destructuring, Match, direct nested arrays, Bool arrays, dynamic
> bounds, runtime checks, cyclic aggregates, generics beyond accepted CAP-004,
> visibility, separate
> compilation, stable layout/ABI/FFI, general ownership/drop/lifetimes, heap storage,
> accelerator execution, or performance guarantees. Bool, String, non-Copy, nested,
> and otherwise unsupported arrays do not gain function transport. LLVM owns internal
> padding and alignment. `main` retains exact `i32 @main()`, and other method calls
> remain a distinct AST form.

> **Reference status:** CORE-048 is an accepted bounded capability, not a general
> borrow checker. It supports immutable `&x` only when `x` is an initialized
> local or parameter `Int`, `Float`, or `Bool`; inferred/exact local aliases may be
> copied and dereferenced into already-supported scalar contexts. Checked IR records a
> fresh read-only alias place, verifies its exact pointee and dominance, and lowers it
> as a typed zero-offset pointer derivation plus scalar load. Exact implementation
> `98c21b9` passes 159 library and 165 binary tests plus every downstream gate and all
> eight public checks. Stable Linux used LLVM/Clang 22.1.8 for external verification,
> machine verification, object lowering, linking, and exact native exit 127.
>
> CORE-053 is publicly accepted for passing those same non-escaping immutable
> scalar references into unique non-generic internal functions. One whole-signature
> classifier admits arbitrary declaration order and count mixed only with by-value
> `Int`/`Float`/`Bool`; checked parameter-place binders and the independent verifier
> preserve exact pointee, dominance, coverage, and pointer-bearing calls. LLVM uses
> internal `double*` for `Int`/`Float` and `i1*` for `Bool` with no pointer/integer
> conversion. Exact implementation `b4aec4a` passes all eight public checks. Stable
> Linux uses LLVM/Clang 22.1.8 for external verification, machine verification, object
> lowering, linking, and exact native exit 211, with 163 library and 169 binary tests.
>
> CORE-055 is publicly accepted for a direct local `&mut owner` alias when `owner`
> is an initialized mutable `Int`, `Float`, or `Bool`. Mutable aliases are non-`Copy`;
> exact `*alias` reads and `*alias = value;` writes retain alias/source/pointee identity
> through checked borrow, write, and lexical-end instructions. The verifier rejects
> competing loans, owner access during the loan, raw-store substitution, wrong release
> identity, and use after release. Exact implementation `1f6ea72` passes all eight
> public checks. Stable Linux uses LLVM/Clang 22.1.8 for external verification, machine
> verification, object lowering, linking, and exact native exit 239, with 166 library
> and 172 binary tests.
>
> CORE-056 is publicly accepted for exactly one mutable scalar-reference parameter
> on a non-generic internal function and exactly one direct `callee(&mut owner)` call
> argument. The temporary exclusive loan is represented as an adjacent checked
> borrow/call/end sequence; the callee receives a distinct writable checked parameter
> binder. One shared whole-call classifier owns topology and source facts across
> semantics and checked admission. LLVM uses the existing private `double*`/`i1*`
> representation without pointer/integer conversion. Exact implementation `e3ff165`
> passes all eight public checks. Stable Linux uses LLVM/Clang 22.1.8 for external
> verification, machine verification, object lowering, linking, and exact native exit
> 251, with 167 library and 173 binary tests.
>
> CORE-057 is publicly accepted for passing an initialized in-scope CORE-055 local
> mutable scalar alias, or the current mutable-reference parameter, to that same exact
> signature. The identifier creates a child reborrow for the adjacent call without
> moving or copying its parent. The verifier requires active local-alias or parameter
> provenance, exact pointee, child-borrow/call/end adjacency, parent exclusion during
> the child, and restoration afterward. Repeated calls, multi-hop forwarding, branches,
> loops, direct modules, and terminating recursion are covered. Exact implementation
> `7c108ff` passes all eight public checks; the pinned LLVM/Clang lane externally
> verifies, lowers, links, and records exact native exit 253 with 169 library and 175
> binary tests.
>
> CORE-059 is publicly accepted for immutable references over every exact
> already-admitted Copy-data place: `Int`/`Float`/`Bool`, flat Copy-scalar tuples,
> fixed numeric arrays, fixed arrays of one exact Copy struct, and finite acyclic Copy
> structs. One `copy_place_contract` classifies supported, explicitly rejected, and
> preserved topology across source semantics and checked admission. Exact recursive
> pointee schemas survive borrowing, aliases, dereference Copy, projection/array
> consumers, arbitrary immutable-reference/owned-Copy internal signatures, CFG,
> forwarding, recursion, direct modules, independent verification, and private typed-
> pointer LLVM. Focused and complete compiler suites pass at 173 library and 179
> binary tests. Exact implementation `5a78eb5` passes all eight public checks; pinned
> LLVM/Clang 22.1.8 externally verifies, machine-verifies, object-lowers, links, and
> executes the tracked direct-module program at exact exit 37.
>
> CORE-060 is publicly accepted for exclusive whole-place mutable references over that
> same Copy-data universe. Exact mutable owner, alias, dereference read/write, child
> reborrow, function transport, lexical end, and recursive schema identity survive
> checked IR and independent verification. Exact implementation `7c7a47a` passes all
> eight public checks; pinned LLVM/Clang 22.1.8 externally verifies, machine-verifies,
> object-lowers, links, and executes exact native exit 59 with 174 library and 180
> binary tests.
>
> `&mut *alias`, mutable-reference results, temporary or projected borrow origins,
> projected mutable origins or writes, escape, relocation, alias reassignment,
> storage/capture, NLL, lifetime inference, drop, stable pointer ABI/FFI, and any memory-
> safety guarantee remain unsupported. A local alias's root owner remains borrowed until
> lexical alias end. The Windows host accurately remains `InternalOnly` because LLVM 22
> is absent locally.

> **Mutation status:** CORE-054 is publicly accepted for semicolon-terminated
> `target = value;` statements inside admitted functions. `target` must resolve to the
> nearest initialized, owned local `let mut` of exact type `Int`, `Float`, or `Bool`,
> and `value` must have the same logical type. Sequential writes, nested/shadowed
> bindings, branches, compiler-bounded `for`, `while`-carried state, internal calls,
> and one-level direct modules retain one place identity. One shared classifier owns
> topology, mutability, ownership, and exact-type admission across semantic analysis
> and checked admission. Checked mutable-place and assignment instructions are
> independently verified before typed `double`/`i1` allocation, store, and load LLVM.
> Exact implementation `6ef3e44` passes 165 library and 171 binary tests plus all eight
> public checks; pinned Linux LLVM/Clang 22.1.8 externally verifies, machine-verifies,
> object-lowers, links, and executes exact native exit 227.
> CORE-061 is publicly accepted for the same direct whole-owner statement across its
> then-frozen exact Copy-data types. One owned-assignment context delegates schema
> classification to `copy_place_contract`; one checked mutable Copy-place allocation
> and assignment cover scalars and aggregates. Exact implementation `de6fc0d` passes
> 175 library and 181 binary tests plus all eight public checks; pinned LLVM/Clang 22
> externally verifies, machine-verifies, object-lowers, links, and executes native exit
> 83. Accepted CORE-062 subsequently replaces the topology list with recursive CopyData.
> Immutable locals/parameters, unknown or uninitialized
> targets, borrowed targets, String/references/unsupported layouts, projected or
> non-identifier targets, assignment values/chaining/compound syntax, NLL, drop, stable
> ABI, and memory-safety claims remain excluded.
>
> CORE-064 is publicly accepted for direct whole-owner reassignment of the
> exact enum class accepted by CORE-063. One shared owned-place classifier serves
> semantic analysis and checked admission; generalized checked allocation/assignment
> identities and the independent verifier preserve exact enum schema; private LLVM
> uses typed enum loads/stores. A distinct local RHS is moved and direct
> self-replacement rejects. The exhaustive target and complete Rust suite pass at 180
> library and 186 binary tests. Exact implementation `79aed71` passes all eight public
> checks; stable job `92376666972` uses LLVM/Clang 22.1.8 for the known-invalid control,
> external and machine verification, object lowering, explicit private non-PIE linking,
> and exact native exit 131; nightly job `92376666842` repeats exit 131. Enum borrowing,
> projection, array/field storage,
> partial moves, new CFG ownership, drop/lifetimes, stable ABI/FFI, and general enum
> mutation remain unsupported.

> CORE-065 is publicly accepted for exact acyclic conditional joins over the existing
> admitted enum owners. Sibling `if` arms begin
> from one ownership snapshot; definitely returning arms do not reach the merge; mixed
> reachable ownership becomes `MaybeMoved` and later use rejects deterministically.
> Semantic analysis and checked admission share this classifier, while independent
> checked-IR dataflow follows exact enum result/place identities through CFG predecessor
> unions and rejects serial, partial-merge, cyclic, or unreplaced-place double
> consumption. The focused and root gates pass with 182 library and 188 binary tests.
> Exact implementation `f4daeea` passes all eight public checks; stable LLVM/Clang
> 22.1.8 proves the known-invalid control and exact native exit 137, while nightly
> repeats exit 137. CORE-073 separately supersedes only the exact acyclic whole-owner
> reinitialization exclusion; loop fixed points, `break`/`continue` transport and
> loop-contained reinitialization, enum borrowing/storage/projection, partial moves,
> drop/lifetimes, stable ABI/FFI, and general CFG ownership remain unsupported.

> CORE-073 is publicly accepted for exact acyclic whole-owner reinitialization
> of the already admitted destructor-free enum class. A single assignment-transition
> authority classifies ordinary replacement and `Moved`/`MaybeMoved` reinitialization;
> semantics and checked admission restore exactly `Owned`, and independent verifier
> dataflow proves predecessor consumption, exact schema/value identity, dominance, and
> the checked write kill. Exhaustive source, direct-module, CLI, corruption, and private
> LLVM evidence passes with the full 190-library/196-binary surface, all eight exact-
> head checks, and pinned stable/nightly LLVM/Clang 22.1.8 native exit 199. Every
> lexically loop-contained reinitialization,
> projected/partial write, enum borrow/storage expansion, destructor/drop/lifetime rule,
> stable ABI, and general CFG fixed point remains rejected.

> CORE-074 is publicly accepted for fresh owned-enum results from exhaustive
> Match expressions. Every arm must yield the same already admitted enum through a
> constructor, an exact call with no additional owned-enum consumption, or a recursively
> fresh nested Match. One checked result-place identity retains distinct result/dispatch
> schemas; independent verification proves one dispatch-target-dominated write per arm,
> all-path initialization, one merged load, and valid later ownership. The result can be
> bound, called, returned, re-Matched, replaced, or acyclically reinitialized. The full
> 191-library/197-binary gates, all eight exact-head checks, and pinned stable/nightly
> native exit 203 pass. Conditional owner transport,
> aggregate Match results, broader patterns, storage/borrowing/projection, partial moves,
> drop/lifetimes, stable ABI, and generic/closure semantics remain unsupported.

> CORE-075 is publicly accepted for direct-owner results across exhaustive
> Match control flow. Exact initialized local owners or owned parameters may be selected
> on mutually exclusive paths; one shared dynamic-path classifier rejects same-path
> duplicates and loop effects while deriving all-path `Moved` or partial-path
> `MaybeMoved`. Existing checked enum provenance, the CORE-074 result place, checked
> assignment, verifier CFG ownership proof, and private LLVM layout are reused. The
> 192-library/198-binary complete compiler surface, exact root gate, all eight exact-head
> public checks, and pinned stable/nightly LLVM/Clang 22.1.8 native exit 211 lanes pass
> at exact implementation `50a3e03d0bdbc0e7deddde747bc19df0621c1257`.
> Additional owned call consumption, external nested scrutinees, aggregate storage,
> borrowing/projection, partial moves, drop/lifetimes, stable ABI, and general CFG
> semantics remain unsupported.

> CORE-076 is publicly accepted for unified typed results across exhaustive
> Match control flow. One shared classifier accepts one identical exact recursive finite
> CopyData type or the existing constrained owned-enum class. One generic checked result
> place, exact typed whole-place arm assignments, and independent verifier CFG proof
> replace the former primitive/enum topology split while preserving private LLVM types.
> Arrays including zero length, recursive tuples, finite acyclic structs, primitives,
> nested Matches, and owned enums pass the complete local gates. Exact implementation
> `aefeb2d81fb5374e7373a4819f3c92f83a95eb35`, all eight exact-head checks, and pinned
> stable/nightly LLVM/Clang 22.1.8 native exit 223 pass while preserving the older
> exit-149 specimen. String/reference/
> unit results, dynamic collections, cyclic/unsupported structs, enum aggregate storage,
> wider patterns, runtime/drop/lifetimes, stable ABI, and general ownership remain
> unsupported.

> CORE-077 is publicly accepted for balanced loop-carried reinitialization of
> an exact direct mutable admitted enum owner. Entry, condition/iterable, every reachable
> fallthrough or `continue` backedge, and every `break` exit must be exactly `Owned`;
> return paths do not join, and nested transfers attach to the nearest loop. Semantic
> analysis and independent checked admission feed one shared edge classifier, while
> verifier CFG dataflow rejects missing, bypassed, one-path, generic-store, wrong-schema,
> cycle, and exit repairs. Exact implementation
> `a93d8d38c5f2a2499ce036f659c13cb2ec4fefcb`, all eight exact-head checks, and
> pinned stable/nightly LLVM/Clang 22.1.8 native exit 227 pass while preserving exits
> 149/223. Loop-carried `Moved`/
> `MaybeMoved`, projections/partial moves, enum storage/borrowing, drop/lifetimes,
> stable ABI, imports, accelerators, release, safety, and general loop fixed points
> remain unsupported.

> CORE-078 is accepted public infrastructure, not a language feature. Exact
> implementation `70f59fd72e96246b2ebefdf1ae53a9b7f3280cfe` pins the official full
> LLVM/Clang 22.1.8 Windows x86_64 archive by SHA-256. Exact tools prove the existing
> MSVC target/layout, invalid-build hygiene, external/machine verification, COFF object
> generation, Clang/MSVC linking, public `run`, manual execution, and exit 227. All
> nine exact-head checks pass while Linux stable/nightly preserve exits 149/223/227.
> No stable ABI, general Windows, packaging, release, safety, accelerator, or
> performance claim follows.

> CORE-079 is accepted public for convergent direct-enum loop ownership at exact
> implementation `5b1ec7340db72354542ab325a9f75cad398857c2`.
> One phase-neutral classifier joins `Owned`/`Moved`/`MaybeMoved` at `while`, admitted
> fixed-array `for`, and `loop` headers and exits; semantics and independent checked
> admission recheck only while the finite header widens, and the verifier retains its
> independent cyclic proof. All nine exact-head checks pass. Stable/nightly Linux
> preserve exits 149/223/227 and execute exact exit 229; pinned Windows LLVM/Clang
> 22.1.8 preserves exit 227 and executes exit 229 through public and independent native
> paths. This adds no broader ownership, storage, borrow, lifetime, ABI, or safety claim.

> CORE-080 is accepted public containment for the founding dotted-import
> grammar. Direct and aliased forms retain exact syntax identity and source location;
> malformed forms fail parsing, while executable forms fail through one deterministic
> shared diagnostic before checked IR. Focused 13/13, the compatibility ring, complete
> all-features, static, documentation, diff-hygiene, and exact root gates pass. No lookup,
> namespace, visibility, package, backend, runtime, or positive import behavior is
> implemented. Exact implementation `063953770ce92f00bae452f312c962c2996977bb`
> passes all nine exact-head checks and preserves pinned native exits 149/223/227/229.

> CORE-081 is the locally green canonical-compiler-graph candidate. An exact red architecture
> test proved 35 compiler modules were independently declared by both binary and
> library. Compiler phases are now library-owned; the binary retains only CLI-specific
> modules and consumes narrow service facades without exposing resolver or raw-IR
> representations. Architecture, unit, integration, all-features, static, documentation,
> diff-hygiene, and exact root gates pass; immutable public evidence remains pending. This changes no source
> semantics, diagnostic, checked IR/LLVM, cache identity, CLI status, backend, or ABI.

> CORE-083 is an accepted public mutable enum reference checkpoint. An
> initialized mutable direct owner of an already admitted destructor-free enum may be
> borrowed or locally reborrowed into one exact `&mut E` parameter and replaced only as
> a whole. One shared pointee classifier serves semantics and independent checked
> admission; schema-bearing checked loan/parameter/write/end identities are independently
> verified before private pointer LLVM. The focused target is 5/5 and the exact root
> gate is green at 211 library and 32 binary tests. Bounded PR #8 passed every
> candidate-head check, merged as `680bc6ca`, and passed CI, Rust CI, and CodeQL on the
> exact merge; pinned Linux/Windows LLVM 22 execution observes exit 83. Immutable enum references, reads or
> Match through a reference, reference results/escape/storage, projections/partial
> mutation, aggregate enum storage, unsupported enums, lifetime/NLL/drop, stable ABI,
> and memory-safety claims remain excluded.

> CORE-084 is an accepted public immutable enum-reference Match checkpoint.
> Initialized immutable direct owners of the already
> admitted destructor-free enum class may have multiple non-escaping `&E` aliases and
> exact internal immutable-reference parameters. The sole enum read is exhaustive
> `match *identifier` with an existing CopyData or `Void` result. Exact checked owner
> and read identities are independently verified before private pointer LLVM; generic
> loads cannot substitute. Focused tests are 4/4 and the exact root gate passes at 212
> library and 32 binary tests; the tracked composed specimen passes local CLI check/build
> while stable/nightly Linux and pinned Windows LLVM 22 execute exact native exit 84.
> Bounded PR #10 passed corrected exact-head CI, Rust CI, and CodeQL, merged as
> `ae0f0901`, and passed all three exact post-merge workflows. No additional local
> native result was part of that checkpoint's acceptance. Free
> enum dereference/transport, mutable-owner immutable loans, mutable-reference reads,
> reference results/escape/storage, unsupported enums, lifetime/NLL/drop, stable ABI,
> FFI, and memory-safety claims remain excluded.

> CORE-085 is an accepted public mutable-owner immutable enum-loan checkpoint.
> Initialized mutable direct owners of the admitted destructor-free enum class may have
> multiple non-escaping `&E` aliases, including exact internal immutable-reference
> parameters, and may be observed only by the accepted exhaustive `match *identifier`
> path. One shared source predicate controls admission and live-loan loop edges; checked
> IR preserves exact reference/source/schema/end identity; and the verifier counts
> overlapping aliases and proves identical state at CFG joins. Owner mutation, move,
> mutable borrow, owned Match, and escape remain rejected while any alias is live. The
> focused target is 4/4, the dedicated corruption control passes, and bounded PR #12,
> exact-head workflows, protected merge `d0832c6f`, all three post-merge workflows, and
> pinned native exit 85 pass. Free enum dereference/transport, reference
> results/escape/storage, unsupported enums, lifetime/NLL/drop, stable ABI, FFI, and
> memory-safety claims remain excluded.

> CORE-086 is an accepted public mutable enum-reference observation checkpoint. Active
> exclusive local aliases and exact sole mutable-reference parameters may be observed by
> repeated exhaustive `match *identifier`, including before and after accepted
> whole-value replacement. A shared classifier, distinct checked mutable-read identity,
> and independent provenance/schema/adjacency verification close the class. Homogeneous
> discarded `Void` Matches share one contract across owned, immutable-reference, and
> mutable-reference scrutinees without result storage; `print!`/`println!` remain
> effect-only. Focused tests are 5/5, the exact root gate passes 214 library and 32
> binary tests, and local pinned LLVM/Clang 22.1.8 executes the two-module specimen at
> exact exit 86. Bounded PR #13, exact-head checks, protected merge `e2014a17`, and all
> three post-merge workflows pass. Raw enum
> extraction/transport, escape, overlap, new lifetime/NLL/drop, stable ABI/FFI,
> accelerator, safety, and stability semantics remain excluded.

> CORE-087 is an accepted public mixed-signature checkpoint. Exactly one admitted
> mutable whole-place reference parameter may be composed with one or more independent
> recursive CopyData parameters, with the reference first, middle, or last. One shared
> indexed topology contract controls both semantic routes, checked admission, lowering,
> and independent verification; direct owners, local alias reborrows, parameter
> forwarding, recursive aggregate sides, enum pointees, and CopyData/`Void` results
> execute in the tracked native exit-87 specimen. The focused target passes 3/3 and the
> verifier corruption control passes 1/1. Bounded PR #14 passed all nine exact-head
> checks, merged through protected master as `b07efe29`, and passed exact post-merge CI,
> Rust CI, and CodeQL. Multiple references, owner-dependent sides, projections,
> reference results or escape/storage/capture, lifetime/NLL/drop, stable ABI/FFI,
> accelerators, and general memory-safety claims remained excluded.

> CORE-088 is an accepted public mixed exclusive/shared-reference checkpoint. Every
> non-entry, non-generic ordered signature with exactly one admitted mutable reference,
> one or more admitted immutable references, and any recursive CopyData companions is
> accepted under the same topology predicate. The mutable source must be independent
> from every other argument; immutable arguments may share an immutable source.
> Checked verification requires exact immutable-borrow/parameter identities and the
> adjacent mutable borrow/call/end window. Focused tests pass 3/3, the corruption
> control passes 1/1, the complete repository gate is green, and pinned LLVM/Clang
> 22.1.8 executes the direct-module specimen through public and independent native
> paths at exit 88. Bounded PR #15 passed all nine exact-head checks, merged through
> protected master as `a7627aa1`, and passed post-merge CI, Rust CI, and CodeQL.
> Multiple mutable parameters,
> projections, reference results/escape/storage/capture, lifetime/NLL/drop, stable
> ABI/FFI, accelerators, and general memory-safety claims remain excluded.

> CORE-089 is an accepted public multiple exclusive-reference checkpoint. Every
> non-entry, non-generic ordered signature with two or more admitted mutable references,
> any admitted immutable references, and recursive CopyData companions is classified by
> the same shared authority. Mutable roots must be pairwise distinct and disjoint from
> every non-mutable argument tree. Lowering emits one declared-order N-borrow/call/
> reverse-N-end window, and independent verification reconstructs its roots, operands,
> binders, and adjacency. Focused tests pass 3/3, the corruption control passes 1/1,
> the affected reference ring passes 19/19, and pinned local LLVM/Clang 22.1.8 executes
> the direct-module specimen through public and independent native paths at exit 89.
> The exact root gate passes 216 library tests, 32 binary tests, every integration
> target, and doc tests. Bounded PR #16 passed all nine exact-head checks, merged through
> protected master as `7fbaaaa4`, and passed post-merge CI, Rust CI, and CodeQL.
> Projections, reference results/escape/
> storage/capture, lifetime/NLL/drop, stable ABI/FFI, accelerators, and general
> memory-safety claims remain excluded.

> CORE-090 is an accepted public static projected CopyData assignment checkpoint. One
> shared classifier accepts any nonempty finite mix of declared struct fields, tuple
> constants, and nonnegative in-range integer-literal fixed-array indexes rooted at an
> initialized mutable owned direct local recursive finite CopyData value, with an
> exact-type CopyData RHS. Semantic analysis, semantic-independent checked admission,
> and lowering consume that contract; the verifier independently traces typed
> projections back to the mutable owner. Focused execution passes 1/1, classifier and
> corruption controls pass 2/2, the affected ring passes 15/15, and the complete root
> gate passes 218 library tests, 32 binary tests, every integration target, and doc
> tests. Pinned LLVM/Clang 22.1.8 executes the tracked direct-module specimen at exact
> exit 90 on Linux and Windows; bounded PR #17 merged through protected master as
> `12820561`, and its exact post-merge workflows pass.
> Dynamic indexes, projected borrowing,
> partial moves, enum/non-Copy subplaces, alias analysis, lifetime/drop, stable ABI/FFI,
> accelerators, and general memory-safety claims remain excluded.

> **Pattern matching status:** CORE-049 accepts one bounded owned unit-enum class:
> unique top-level non-generic enums with one or more unit variants, exact payload-free
> construction, immutable local moves, and exhaustive matches containing exactly one
> explicit arm per variant with uniform `Int`, `Float`, or `Bool` results. Unit enums
> remain non-`Copy`; matching an identifier consumes it, nested possible-arm consumption
> is conservative, and reuse fails before IR. Checked IR preserves distinct enum identity
> and exhaustive dispatch, the verifier independently rejects malformed schemas and CFG,
> and LLVM uses an internal `i32` plus `switch` without creating a public integer identity
> or ABI. Exact implementation `b38a6b0` passes 160 library and 166 binary tests plus all
> eight public checks; pinned Linux LLVM/Clang 22.1.8 externally verifies, machine-verifies,
> object-lowers, links, and executes the composed module with exact native exit 149.
> CORE-050 is also publicly accepted at exact implementation `13f0003`. It extends
> those unit enums through internal owned parameters, arguments, call results, and
> returns using one shared signature resolver and consumed-name classifier, direct
> checked SSA parameter binders, exact call/return verification, and internal `i32`
> LLVM flow. All eight public checks pass, and pinned Linux LLVM/Clang 22.1.8
> externally verifies, machine-verifies, object-lowers, links, and executes exact
> native exit 173.
>
> CORE-051 is publicly accepted for owned local enums whose variants are
> unit or carry exactly one `int`, `float`, or `bool` payload. Construction requires
> the exact declared scalar type; exhaustive Match requires one identifier binding
> for each payload arm, scopes that Copy scalar to the selected arm, and consumes the
> non-`Copy` enum. One shared schema classifier serves semantics, checked admission,
> IR, verification, and lowering. Checked IR retains construction, selected payload
> extraction, and exhaustive dispatch; verified LLVM uses a private
> `{ i32, double, i1 }` aggregate with deterministic inactive lanes. Exact implementation
> `babb1cd5` passes all eight public checks; stable job `92223344697` uses pinned
> LLVM/Clang 22.1.8 for external and machine verification, object/link, and exact native
> exit 181, with 162 library and 168 binary tests. Aggregate storage, references,
> non-scalar/multi-field/struct/generic payloads, Option/Result matching, wildcard/
> guard/nested destructuring, mutation, stable layout/ABI, and general pattern
> matching remain unsupported. Other Match topologies retain the fail-closed boundary.
>
> CORE-052 is publicly accepted and carries every supported unit or unary
> scalar-payload enum schema through exact internal parameters, arguments, call results,
> and returns. One shared transport annotation resolver admits the complete schema class;
> `CheckedEnumParameter` and the independent verifier preserve exact binder/signature/
> call/return identity and ownership transfer. Unit enums remain private `i32`; payload
> enums remain private `{ i32, double, i1 }` SSA values. Exact implementation
> `93a4a29e` passes 162 library and 168 binary tests plus all eight public checks;
> stable job `92227409386` uses LLVM/Clang 22.1.8 for external and machine verification,
> object/link, and exact native exit 197. This creates no stable layout, public calling
> convention, ABI, FFI,
> aggregate enum storage, borrowing, mutation, drop, or general CFG ownership claim.
>
> CORE-063 is publicly accepted for unary payloads drawn from the
> accepted recursive CopyData grammar: fixed arrays, arity-at-least-two tuples, and
> finite acyclic named Copy structs in addition to scalars. Exact construction,
> exhaustive identifier-bound Match, arm-local projections, owned internal transport,
> direct modules, checked schemas, verifier corruption controls, and private typed LLVM
> pass the exhaustive target and exact root gate at 179 library and 185 binary tests.
> Unit/scalar layouts retain their accepted private forms; aggregate schemas use a
> private tag plus exact typed lanes. Exact implementation `2a5c3c5` with verified
> native-link repair head `bebd0b6` passes all eight public checks; stable job
> `92363420145` uses LLVM/Clang 22.1.8 for the known-invalid control, external and
> machine verification, object lowering, explicit private non-PIE link, and native exit
> 113. This is not a stable layout/ABI claim.

Formal spec: `docs/language/aero_formal_language_specification.md`

## Looking Ahead

- GGUF-native model loader and runtime benchmarks on CUDA/ROCm
- Expanded optimizer and fused-kernel library coverage
- Additional formal semantics proofs beyond deterministic conformance checks

## License
MIT © RobVanProd and contributors. See LICENSE for details.
