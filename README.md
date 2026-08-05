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
| **Type System** | Static scalar checks. Generic and trait syntax is parsed but quarantined; generic substitution, trait-bound enforcement, and where-clause semantics are not supported contracts. |
| **Memory** | Shallow move tracking plus bounded, publicly accepted whole-place immutable and mutable references and direct reassignment over the exact admitted recursive CopyData universe. Accepted CORE-064 extends direct whole-owner replacement to admitted enums; accepted CORE-065 adds exact acyclic conditional joins; accepted CORE-066 adds fresh per-iteration enum owners only. Projected writes/borrows, outer-owner loop joins/reinitialization, and general aliasing remain unsupported. No general borrow checker, general mutable-reference model, lifetime analysis, drop model, stable pointer ABI, or memory-safety guarantee. Reference results remain unsupported. |
| **Data Types** | Recursive finite CopyData composition and bounded unit-or-unary recursive CopyData owned enums with exhaustive bound Match, internal transport, exact mutable whole-owner replacement, acyclic conditional ownership joins, and fresh per-iteration loop-local owners are publicly accepted. CORE-066 reuses those exact schemas and adds no representation or topology. Enum fields, arrays, borrowing, projection, and broader storage/destructuring/generic semantics remain unsupported. |
| **Control Flow** | Functions, if/else, while/checked fixed-array for/loop, and nearest-loop break/continue are partial. Accepted CORE-066 corrects checked `for` continue so it reaches the index increment before the header and proves fresh per-iteration enum consumption; labels, loop expressions/break values, non-array checked iterators, and outer-owner loop transport remain unsupported. Closure syntax is parsed-only; executable closure expressions fail closed before checked IR. |
| **Intrinsic methods** | Accepted CORE-067 centralizes intrinsic method classification across semantics and checked IR. Exact recursive CopyData fixed-array `.len()`/`.is_empty()`, immutable compile-time String queries, and Array/Vec `.iter()` compatibility are the only admitted executable method forms. Runtime Strings, other collection methods, general dispatch, generic/trait methods, and callable ABI remain unsupported. |
| **Direct module source collection** | Root-level `mod x;` collects `x.aero` or `x/mod.aero` into the current flattened compilation unit. `use`, `pub` visibility semantics, namespaces, recursive modules, and cycle graphs are not implemented. |
| **Codegen** | LLVM IR backend with optimization passes |
| **CLI** | `aero build`, `aero run`, `aero check`, `aero test`, `aero fmt`, `aero doc`, `aero profile`, `aero graph-opt`, `aero quantize`, `aero registry`, `aero conformance`, `aero init`, `aero lsp` |
| **LSP** | Syntax diagnostics, completion, hover, go-to-definition, document symbols |
| **Docs & Profiling** | Markdown API generation (`aero doc`), compilation stage timing + trace export (`aero profile`) |
| **Phase 8 Experimental Slice** | Textual graph rewriting to internal scalar helpers and scalar-`double` quantization helper rewriting with backend metadata. These are not device execution, real FP8/per-channel execution, or numerical-correctness evidence. The slice also includes local `registry.aero` search and dry-run planning plus 3 example cases and 4 deterministic regression checks (not formal-semantics proof). Live registry transport is quarantined pending a reviewed protocol and trust boundary. |
| **Diagnostics** | Colored errors, source snippets, "did you mean?" suggestions |

> **Closure status:** closures are parsed but unsupported in executable code. The
> opening `|` location is retained and `check`, `build`, and `run` reject closure use
> with one source-located diagnostic before checked IR. The legacy lowering fallback
> that manufactured callable identities and mapped unknown parameter/result types to
> `i32` is removed. Captures, calls, storage/transport, callable ABI, lifetime behavior,
> generics, and closure LLVM generation are not implemented.

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
> bounds, runtime checks, cyclic aggregates, generics, visibility, separate
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
> repeats exit 137. Loop fixed points, `break`/`continue` transport, conditional
> reinitialization, enum borrowing/storage/projection, partial moves, drop/lifetimes,
> stable ABI/FFI, and general CFG ownership remain unsupported.

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
