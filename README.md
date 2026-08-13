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
| **Type System** | Static scalar checks. Accepted CORE-072 adds exact Unicode `char` identity and equality/inequality across the complete existing recursive CopyData transport class. Accepted CAP-004 adds one explicit user-defined recursive-CopyData generic-struct substitution class; accepted CAP-005 adds bound-free whole-value generic transport functions over exact recursive finite CopyData substitutions; accepted CAP-006 adds exact explicit user-defined recursive-CopyData generic-enum specialization; accepted CAP-010 adds one required-only recursive-CopyData trait-bound static-dispatch class; accepted CAP-011 adds schema-driven specialization for bounded fixed-capacity generic container algorithms; and accepted CAP-013 gives those bounded paths one recursive primitive-alias identity and shared deterministic phase authority. General generic operations/impls/traits beyond those bounded classes, inference/defaults, broader trait-bound enforcement, and where-clause semantics remain parsed, quarantined, or unsupported. |
| **Memory** | Shallow move tracking plus bounded, publicly accepted whole-place immutable and mutable references and direct reassignment over the exact admitted recursive CopyData universe. Accepted CORE-064–079 add bounded owned-enum replacement, joins, reinitialization, Match results, and loop ownership. Accepted CORE-083–089 add non-escaping enum-reference replacement/observation and mixed/multiple-reference callable signatures. Accepted CORE-090 adds exact static writes through arbitrary finite field/tuple/fixed-array paths rooted at a mutable owned direct local CopyData value. Accepted CAP-002 adds guarded runtime indexes in that same direct-owner write class, and accepted CAP-012 adds immediate nonescaping immutable/mutable call loans over those finite nested CopyData places with conservative complete-root conflicts. Stored projected references, reference-target dynamic writes, partial moves, disjoint alias claims, free enum dereference/transport, enum aggregate storage, and general aliasing remain unsupported. No general borrow checker, general mutable-reference model, lifetime analysis, drop model, stable pointer ABI, or memory-safety guarantee. Reference results remain unsupported. |
| **Data Types** | Recursive finite CopyData composition and bounded positional recursive CopyData owned enums—including exact variants with two or more fields—are publicly accepted with exhaustive Match, internal transport, exact mutable whole-owner replacement/reinitialization, acyclic conditional ownership joins, and fresh per-iteration loop-local owners. Accepted CAP-008 adds identifier-or-`_` payload leaves and one optional final whole-arm wildcard across the complete admitted concrete enum class while preserving exhaustiveness. Accepted CORE-072 adds `char`; CORE-074–076 add typed Match results; and CORE-083–089 add bounded enum-reference and callable compositions. Accepted CORE-090 composes existing named fields, tuple constants, and integer-literal fixed-array indexes into exact mutable CopyData paths without adding a data topology. Accepted CAP-001 adds guarded runtime reads, CAP-002 adds guarded runtime-indexed writes over the same nonempty recursive CopyData fixed-array class, CAP-003 adds explicitly typed concrete recursive-CopyData `Option<T>`/`Result<T,E>` construction, owned transport/replacement, and exhaustive bound Match, CAP-004 adds explicit user-defined recursive-CopyData generic structs with deterministic substitution and checked identity/schema verification, CAP-006 adds bound-free unit/positional user-defined generic enums at exact recursive-CopyData applications through contextual construction and exhaustive Match, CAP-011 composes fixed arrays plus projected mutation into a bounded fixed-capacity generic CopyData container API, and CAP-012 admits only immediate call-scoped projected loans over that finite CopyData class. `print!` and `println!` are effect-only `Void`, not scalar values. Named-field/bounded/general generic enum variants, general generic substitution/error propagation, guards, nested destructuring and other wider patterns, carrier or enum aggregate/reference storage, enum fields/arrays, free enum dereference or transport through references, dynamic collections, stored/escaping projected references or partial moves, unsupported/cyclic structs, and broader storage or destructuring semantics remain unsupported. |
| **Control Flow** | Functions, if/else, while/checked fixed-array for/loop, and nearest-loop break/continue are partial. Accepted CORE-066 corrects checked `for` continue and proves fresh per-iteration enum consumption. Accepted CORE-077 adds exact balanced direct enum-owner restoration. Accepted CORE-079 iterates direct-enum header and exit joins to convergence across condition/iterable, fallthrough/continue, and break edges, with return paths excluded and nested transfers attributed to the nearest loop. Labels, loop expressions/break values, non-array checked iterators, non-enum fixed points, and general CFG ownership remain unsupported. Closure syntax is parsed-only; executable closure expressions fail closed before checked IR. |
| **Function calls** | Accepted CORE-068 centralizes exact named-call classification across both semantic paths and checked admission/lowering. Existing nongeneric functions over admitted scalar, recursive CopyData, owned-enum, and reference contracts remain supported; accepted CORE-087 composes one mutable reference with CopyData sides, CORE-088 adds immutable-reference sides, and CORE-089 admits two or more pairwise-disjoint mutable references with any immutable and CopyData sides in every declared order. CORE-090 permits already admitted calls as projected-assignment RHS values, and accepted CAP-012 admits immediate nonescaping projected CopyData arguments to ordinary nongeneric reference parameters with checked call-lifecycle identity. Accepted CAP-003 transports exact concrete recursive-CopyData `Option`/`Result` values, CAP-004 transports exact concrete user-defined generic CopyData structs through nongeneric internal parameters and results, CAP-005 specializes bound-free whole-value generic transport functions over exact recursive finite CopyData arguments, CAP-006 transports exact specialized user-defined generic enums through nongeneric parameters/results, CAP-010 specializes exact required-only trait method calls over recursive-CopyData structs to verifier-bound monomorphic helpers, and CAP-011 specializes exact fixed-container read/update algorithms inferred through generic-struct schemas. Missing or unsupported signatures, wrong arguments, mutable-root overlap with any other argument, and `Void` value use fail before checked IR. Overloads, conversions, broader trait-bounded or operational generic functions, generic-to-generic/recursive calls, general trait/closure calls, stored/escaping projected references, generic projected calls, reference results, question-mark propagation, and stable callable ABI remain unsupported. |
| **Intrinsic methods** | Accepted CORE-067 centralizes intrinsic method classification across semantics and checked IR. Exact recursive CopyData fixed-array `.len()`/`.is_empty()`, immutable compile-time String queries, and Array/Vec `.iter()` compatibility are the only admitted executable method forms. Runtime Strings, other collection methods, general dispatch, generic/trait methods, and callable ABI remain unsupported. |
| **Direct module source collection** | Root-level `mod x;` collects `x.aero` or `x/mod.aero` into the current flattened compilation unit. Accepted CORE-070 adds public library `compile_file(path, options)` over this exact collector and the checked library frontend; it returns in-memory LLVM and writes no artifact. Accepted CAP-007 adds artifact-free `check_program`/`check_file` APIs over the same checked preparation authority. Accepted CORE-071 preserves Rust-like `use` syntax and source locations but rejects executable use before checked IR. Accepted CORE-080 likewise preserves the founding direct/aliased dotted `import` syntax with a distinct AST identity and fail-closed diagnostic. Accepted CORE-081 makes the collector and compiler phases library-owned while preserving current flattened behavior. Positive import/name-resolution, `pub` visibility semantics, namespaces, recursive modules, cycle graphs, and separate compilation are not implemented. |
| **Selected profiles** | `stable-scalar-v0` remains Aero's only `STABLE` profile. Accepted CAP-014 created the separate CPU-only `exact-i32-array-v0` profile, classified `END_TO_END`; accepted CAP-018 widened that same profile with immutable exact-array result composition; and accepted CAP-019 adds fully initialized mutable owned locals, guarded direct element writes, and returned flat-array values. Broad integer and fixed-array support remains `PARTIAL`; the default experimental profile is not widened or reclassified. |
| **Codegen** | LLVM IR backend with optimization passes. In `exact-i32-array-v0`, admitted scalar and array leaves use exact wrapping LLVM `i32`; dynamic indexes take signed lower/upper trap paths before GEP address formation and then `sext i32`. This private selected-profile representation is not a public aggregate layout/ABI, SIMD, tensor, accelerator, or performance claim. |
| **CLI** | `aero build`, `aero run`, `aero check`, `aero test`, `aero fmt`, `aero doc`, `aero profile`, `aero graph-opt`, `aero quantize`, `aero registry`, `aero conformance`, `aero init`, `aero lsp`. Accepted CORE-081 removes the binary's duplicate compiler-phase module graph. Accepted CAP-007 makes check/build/run/profile/source-test validation consume one checked-program preparation authority; docs and LSP syntax diagnostics remain intentionally parse-only. |
| **LSP** | Syntax diagnostics, completion, hover, go-to-definition, document symbols |
| **Docs & Profiling** | Markdown API generation (`aero doc`), compilation stage timing + trace export (`aero profile`) |
| **Phase 8 Experimental Slice** | Textual graph rewriting to internal scalar helpers and scalar-`double` quantization helper rewriting with backend metadata. These are not device execution, real FP8/per-channel execution, or numerical-correctness evidence. The slice also includes local `registry.aero` search and dry-run planning plus 3 example cases and 4 deterministic regression checks (not formal-semantics proof). Live registry transport is quarantined pending a reviewed protocol and trust boundary. |
| **Diagnostics** | Colored errors, source snippets, "did you mean?" suggestions |

> **CAP-015 accepted project integration:** protected master now includes the bounded
> embedded character-record interpreter in the existing M1-001 telemetry application.
> The maintained source interprets exact grammar
> `T=<digit><digit>;H=<digit><digit>;`, obtains 42 from `T=17;H=08;`, materially uses
> that value as its third calibration sensor, proves results 0 and 297 at the numeric
> boundaries, and preserves exact output `telemetry score: 91` and exit 91. All ten
> first-malformed positions, three first-error precedence controls, and negative and
> equal-to-count parser-index traps are exercised. Checked metadata retains `char`,
> `[char; 10]`, and `Result<int, char>`; public and library LLVM agree; and Linux and
> Windows LLVM/Clang 22 `-O0`/`-O2` gates verify and execute the representative and
> trap controls. Exact candidate `dd9b1710abebf2f2318582cf94568c2f9a30ca8f`,
> protected PR #52 merge `b62696272f293f9f378f8a368cc818fcb8ef1074`, and shared
> tree `27f359bc5ca90212a06ce73b71759cac0533c1f0` are exact. Candidate push/PR CI
> `31597830488`/`31598146528`, Rust CI `31598146473`, CodeQL `31598144554`, and
> merge-head CI/Rust CI/CodeQL `31598634185`/`31598634090`/`31598633803` pass.
> CAP-015 changes no compiler production or language-profile code; both named profiles
> continue to reject the parser specimen. It adds no CAP-015 parser, grammar, profile,
> language-feature, stability, or conformance row.
> General-purpose text parsing, runtime Strings, serialization, runtime ingestion,
> file input, and Unicode text encoding/normalization remain unsupported; accepted
> CORE-072's bounded Unicode scalar `char` remains `PARTIAL`.

> **CAP-023 accepted product gate:** protected master now carries one source-embedded
> fixed-shape 3-input/2-hidden/2-output exact-`i32` zero-clamp-and-argmax CPU product
> over the unchanged `exact-i32-array-v0` profile.
>
> Accepted CAP-023 changes no parser, grammar, source semantics, language profile,
> semantic analysis, checked IR, verifier, backend, ABI, or capability classification.
> It is a zero-production product/evidence checkpoint over CAP-019's unchanged
> `exact-i32-array-v0` surface, composes the accepted CAP-020 flat matvec and CAP-021
> record-to-score product, and does not reverse CAP-022's mandatory
> runtime-acquisition `NO IMPLEMENTATION` stop.
>
> One source-embedded application convention `[int; 20]` with exact header `[2, 3, 2]`
> drives a flat 3-input/2-hidden/2-output wrapping-`i32` computation with
> strict-positive zero clamp, two biased logits, signed strict-greater argmax,
> lower-index tie selection, three independent malformed-header controls, and reread
> of all 140 source lanes after all seven by-value calls.
>
> Exact ordinary, wrapping, activation-boundary, and tie results are respectively
> `[1, 122, 167, 135, 181, 4940, 5573, 1]`,
> `[1, -24, 18, 2147483623, 0, -37, 2147483641, 1]`,
> `[1, -3, 0, 0, 0, 5, 4, 0]`, and `[1, 1, 2, 1, 2, 3, 3, 0]`; malformed results
> are eight zeros; public and native success is sentinel 91 with empty source
> stdout/stderr.
>
> Exact CAP-023 reviewed candidate
> `63e6b00b6294de61e3afd292a1e32e2b014714e2`, shared candidate/merge tree
> `4d234cdfde67f1083773e2c41be4ab92027769db`, accepted base and first merge parent
> `4bce540dfed6dfffa152067f4e00424501a6cdd8`, and protected PR #62 merge
> `e9b281504446465cfc8fcbe17c65cce92df0e83a` whose second parent is that candidate are
> immutable. Candidate push CI `31687464571`, PR CI `31687585904`, Rust CI
> `31687585893`, CodeQL `31687584263`, and aggregate candidate check `94407323731`;
> candidate push/PR compiler jobs `94406770929`/`94407177877`,
> stable/nightly/Windows LLVM 22 jobs
> `94407178006`/`94407178047`/`94407178042`, CodeQL Actions/Python/Rust jobs
> `94407175858`/`94407175752`/`94407175820`, and Actions/Python/Rust analyses
> `1612686978`/`1612687391`/`1612693654`; merge-head CI/Rust CI/CodeQL
> `31688093145`/`31688093150`/`31688092749`, exact merge
> compiler/stable/nightly/Windows LLVM 22 jobs
> `94408808914`/`94408809340`/`94408809458`/`94408809296`, merge CodeQL
> Actions/Python/Rust jobs `94408812427`/`94408812194`/`94408812175`, and
> default-branch Actions/Python/Rust analyses
> `1612715455`/`1612715345`/`1612721829` all pass.
>
> CAP-023 adds no general activation, ReLU, argmax, inference, tensor, matrix, record,
> recursive-array, runtime/file input, serialization, quantization, conversion, stable
> layout/ABI, performance, resource-usage, accelerator, safety, or language completion
> capability.
>
> Its record and topology are application conventions, its retained local artifacts
> remain mutable corroboration only, and CAP-019 remains the latest compiler/profile
> widening.
>
> The sole matrix change is the existing CPU backend-summary row remaining `PARTIAL`;
> no CAP-023 language feature or selected-profile row may be added.
>
> Current accepted public master is CAP-023 merge
> `e9b281504446465cfc8fcbe17c65cce92df0e83a`. The PR-only aggregate is correctly
> absent on the default branch. Default-branch Actions analysis `1612715455` contains
> only the pre-existing open alert #4 created 2026-08-09; Python and Rust analyses
> contain zero results; no new CAP-023 code-scanning alert exists.
>
> The selected Milestone 0, Milestone 1, and Milestone 2 exit gates are met for their
> bounded selected products; their broader milestone ambitions remain partial.
> Milestone 3 remains open. CAP-023 advances its application and reproducibility
> boundary but supplies no runtime ingestion, composed CopyData application profile,
> quantization, runtime-resource measurement, performance evidence, accelerator
> execution, or broader workload.

> **CAP-021 accepted product gate:** protected master now carries the source-embedded
> two-stage exact-`i32` scoring product over the existing CPU-only
> `exact-i32-array-v0` profile and accepted CAP-020 flat matvec.
>
> Accepted CAP-021 changes no parser, grammar, source semantics, language profile,
> semantic analysis, checked IR, verifier, backend, ABI, or capability classification;
> it is a zero-production product/evidence checkpoint over CAP-019's
> `exact-i32-array-v0` surface and composes the accepted CAP-020 flat matvec.
>
> The accepted application treats one source-embedded flat `[int; 17]` as an
> application record with exact header `[2, 3, 1]`, dynamically decodes input,
> row-major first-stage weights, first-stage bias, second-stage weights, and score bias
> into fully initialized flat locals, then composes the accepted 2x3 matvec with
> wrapping bias and affine scoring.
>
> The accepted scorer returns `[valid, raw0, raw1, hidden0, hidden1, score]`; its
> ordinary result is `[1, 122, 167, 135, 181, 4938]`, its wrapping result is
> `[1, -24, 18, 2147483623, -2147483631, -2147483627]`, an invalid header returns six
> zeros, both valid source records preserve and reread all 17 lanes, and the application
> exits `91`.
>
> Every dynamic read and write uses the existing signed bounds, trap-before-address,
> `sext`, typed-GEP, and same-pointer consumer authority; exact public and pinned
> Linux/Windows LLVM 22 verifier, O0/O2, native, and deterministic-emission evidence
> passes.
>
> Exact CAP-021 reviewed candidate
> `f91df56084540d30f3c8d09e71c5f30db280fd93`, shared candidate/merge tree
> `7e34b4b8e817a7aafaaabc6326fa0a4d616fcc91`, accepted base and first merge parent
> `df0626916d190d8a7580f783e3ac24a89f691617`, and protected PR #60 merge
> `59af445ea02c1759d337d698be9c4f4472587aaf` whose second parent is that candidate are
> immutable. Candidate push CI `31670574143`, PR CI `31670599830`, Rust CI
> `31670599826`, CodeQL `31670598033`, and aggregate candidate check `94354297550`;
> candidate push/PR compiler jobs `94354135184`/`94354214336`,
> stable/nightly/Windows LLVM 22 jobs
> `94354214389`/`94354214394`/`94354214410`, CodeQL Actions/Python/Rust jobs
> `94354210797`/`94354210770`/`94354210832`, and Actions/Python/Rust analyses
> `1611711722`/`1611712334`/`1611716646`; merge-head CI/Rust CI/CodeQL
> `31671091285`/`31671091296`/`31671091099`, exact merge
> compiler/stable/nightly/Windows LLVM 22 jobs
> `94355683766`/`94355683532`/`94355683515`/`94355683534`, merge CodeQL
> Actions/Python/Rust jobs `94355685544`/`94355685480`/`94355685574`, and
> default-branch Actions/Python/Rust analyses
> `1611737053`/`1611737605`/`1611740699` all pass.
>
> CAP-021 adds no tensor, matrix, struct, record, recursive-array, nested-array,
> serialization, runtime/file-input, quantization, activation, checked-overflow,
> stable layout/ABI, performance, accelerator, safety, general inference, or
> language-completion capability; the flat record is an application convention, not a
> source or physical type.
>
> CAP-019 remains the latest compiler/profile capability widening; CAP-020 and CAP-021
> are accepted product gates, not separate profiles or feature rows.
>
> The PR-only aggregate CodeQL check is correctly absent on the default branch; the
> sole open finding remains pre-existing Actions alert #4 from 2026-08-09, and no new
> CAP-021 alert surfaced.

> **CAP-020 accepted product gate:** protected master now carries the flat-buffer
> 2x3-by-3 matvec product gate over the existing CPU-only `exact-i32-array-v0`
> profile. Accepted CAP-020 changes no parser, grammar, source semantics, language
> profile, semantic analysis, checked IR, verifier, backend, ABI, or capability
> classification; it is a zero-production product/evidence checkpoint over CAP-019's
> `exact-i32-array-v0` surface.
>
> The accepted application encodes a 2x3 matrix as `[int; 6]`, consumes an `[int; 3]`
> vector, computes wrapping `row * 3 + column` in nested loops, returns a fully
> initialized mutable-produced `[i32; 2]`, preserves every input lane, produces
> ordinary and wrapping results `[50, 122]` and `[-2, 5]`, and exits `91`.
>
> The computed linear value flows through the existing signed bounds and
> trap-before-address authority before a `[6 x i32]` load, with corresponding guarded
> `[3 x i32]` load and `[2 x i32]` store.
>
> Exact CAP-020 reviewed candidate
> `3b61cd1ed34f910f556821942cd06301ba17dd50`, shared candidate/merge tree
> `800510de85bd82f3332126ad249c95da109dd3e1`, accepted base and first merge parent
> `13157687f3e955d1c8292ccca133c5a73e29e1a7`, and protected PR #58 merge
> `d9493d5123840b38ebab6ca275aaba3216728706` whose second parent is that candidate are
> immutable. Candidate push CI `31639493741`, PR CI `31639540134`, Rust CI
> `31639540030`, CodeQL `31639535638`, and aggregate candidate check `94258433541`;
> candidate stable/nightly/Windows LLVM 22 jobs
> `94258276078`/`94258275978`/`94258275899` and CodeQL Actions/Python/Rust jobs
> `94258264605`/`94258264489`/`94258264627`; merge-head CI/Rust CI/CodeQL
> `31640016314`/`31640016316`/`31640015733`, exact merge
> compiler/stable/nightly/Windows LLVM 22 jobs
> `94259869631`/`94259869676`/`94259869637`/`94259869559`, merge CodeQL
> Actions/Python/Rust jobs `94259873136`/`94259873164`/`94259873086`, and
> default-branch Actions/Python/Rust analyses
> `1610137115`/`1610137589`/`1610144660` all pass.
>
> CAP-020 adds no matrix type, recursive or nested arrays, static index proof,
> checked-overflow arithmetic, stable layout or ABI, performance, accelerator
> execution, general mutation, or safety claim.
>
> CAP-019 remains the latest compiler/profile capability widening; CAP-020 is an
> accepted product gate, not a separate profile or feature row.
>
> The sole open finding remains pre-existing Actions alert #4 from 2026-08-09; no new
> CAP-020 alert surfaced.

> **CAP-019 accepted:** protected master now adds initialized mutable exact-array
> production to the existing CPU-only `exact-i32-array-v0` profile. Accepted CAP-014
> created the CPU-only `exact-i32-array-v0` profile; accepted CAP-018 remains its
> immutable exact-array result-composition checkpoint; accepted CAP-019 widens that
> same profile with fully initialized mutable owned locals, direct projected element
> writes, and returned flat-array values rather than creating another profile.
> Accepted CAP-019 widens the existing flat nonempty exact-`Int` class to a fully
> initialized mutable owned local whose initializer is an admitted literal, immutable
> exact-array identifier, or acyclic ordinary call of the same count, plus direct
> `local[index] = exact_int_value` projected writes.
>
> The maintained eight-lane application copies an immutable input, increments every
> lane in a guarded loop, returns the whole array by value, feeds it into the accepted
> CPU kernel, preserves all eight source lanes, produces result `2035`, and exits `91`;
> Linux and Windows retain read traps and add negative/equal-to-count write traps under
> verified LLVM/Clang 22 `-O0`/`-O2` routes.
>
> Exact CAP-019 reviewed candidate
> `f2955bedd22708041e36ee90c65c4f08c443d740`, shared candidate/merge tree
> `c520729e7b081087bbe431e97d937fb77f519b37`, accepted base and first merge parent
> `84916e124752b8e7d228855a0969cd9eab8dba26`, and protected PR #56 merge
> `6ebeb0efb6e83ccc50e12d395e4add1c63ef48b4` whose second parent is that candidate are
> immutable. Candidate push/PR CI, Rust CI, CodeQL, and aggregate results
> `31627264709`/`31627385522`/`31627385563`/`31627405516`/`94217394313`;
> merge-head CI/Rust CI/CodeQL runs
> `31627880853`/`31627880924`/`31627880812`; merge jobs
> `94218938557`/`94218938794`/`94218938835`/`94218939033`/`94218943455`/
> `94218943514`/`94218943605`; and exact default-branch Actions/Python/Rust analyses
> `1609396076`/`1609396442`/`1609401493` all pass.
>
> The single selected `exact-i32-array-v0` row remains `END_TO_END`; broad integer
> and fixed-array support remains `PARTIAL`; `stable-scalar-v0` remains Aero's only
> `STABLE` profile. CAP-019 does not admit general mutable arrays, uninitialized or
> partial arrays, mutable parameters/results/aliases, references or escaping places,
> whole-array reassignment, zero/recursive/nested/repeat/non-Int arrays, stable
> aggregate ABI/layout, general parsing/string/file behavior, GPU execution,
> performance, or safety. CAP-013 remains the single shared specialization
> identity/phase authority; CAP-018 and CAP-019 add no specialization classifier.

> **CAP-018 accepted:** protected master now composes immutable exact flat-array values
> through the existing CPU-only `exact-i32-array-v0` profile. Accepted CAP-014 created
> the CPU-only `exact-i32-array-v0` profile; accepted CAP-018 widens that same profile
> with immutable exact-array results rather than creating another profile. Ordinary
> acyclic nongeneric functions may construct computed exact-`Int` literals, return an
> exact array by value, bind it immutably with inference or an exact annotation,
> forward or pass literal/identifier/call values, and index literal/identifier/call
> roots. The maintained N=8 application preserves source lane 127, produces transformed
> lane 128, computes 2035, and retains exit 91 through public and native Linux/Windows
> `-O0`/`-O2` routes.
>
> Exact candidate `409eca9ed2dd8b4ba79f34e14ecfefcc0386e3df`, shared tree
> `3073c881c883984f53fcde2f0b205acbec760145`, and protected PR #54 merge
> `c49ff17cab7fc0e8d4f552a71499929135c16c61` are exact. Candidate push/PR CI
> `31614934307`/`31614994226`, Rust CI `31614994253`, CodeQL `31614991761`, and
> merge-head CI/Rust CI/CodeQL `31615467151`/`31615467115`/`31615465499` pass. Exact
> default-branch Actions/Python/Rust analyses `1608636029`/`1608636345`/`1608644785`
> also pass.
>
> Accepted CAP-019 subsequently adds fully initialized mutable owned locals, guarded
> projected element writes, and returned flat-array values. General mutable arrays,
> mutable parameters/results/aliases, whole-array reassignment, recursion,
> zero/repeat/nested/non-Int arrays,
> neighboring aggregate shapes, modules/imports, constants/references, generics/traits,
> closures, I/O, allocation/drop, accelerators, stable ABI/layout, SIMD, tensors,
> performance, safety, stability, releases, and language completion remain excluded.
> Broad integers and fixed arrays remain `PARTIAL`; `stable-scalar-v0` remains Aero's
> only `STABLE` profile.

> **CAP-014 accepted:** protected master now includes the distinct CPU-only
> `exact-i32-array-v0` profile. It admits flat `[int; N]`/`[i32; N]` with
> `1 <= N <= i32::MAX`, explicitly annotated immutable literal locals, by-value
> nongeneric parameters, identifier call transport, and direct indexing by admitted
> scalar integer expressions. Its private LLVM lane uses exact wrapping `i32`; every
> dynamic access takes identity-linked signed lower/upper checks and a trap branch
> before GEP address formation, followed by `sext i32`. The tracked kernel exits 91,
> the wrapping-edge specimen exits 93, and negative/equal-to-count bounds controls
> trap through public, `-O0`, and `-O2` routes on pinned Linux and Windows LLVM/Clang
> 22. Focused 11/11 and the complete 259-library plus integration/CLI/doc/format/
> correctness-Clippy gate pass. Corrected candidate
> `226279dd174f26dc3cd1c7573798955bfe789f78`, protected PR #50 merge
> `ca09ebe3c1b981339c8bf56b360e62208ac900e1`, and shared tree
> `448e1c2ff397012804b886b904aa43bec63f2d37` are exact. Candidate push/PR CI
> `31570455915`/`31570461500`, Rust CI `31570461524`, CodeQL `31570456382`, and
> merge-head CI/Rust CI/CodeQL `31570823665`/`31570823712`/`31570823073` pass.
> CAP-018 subsequently closes CAP-014's immutable array-result limitation in this same
> profile, and CAP-019 adds initialized mutable local production with guarded direct
> element writes and returned values. General mutable arrays, mutable parameters/
> results/aliases, whole-array reassignment, neighboring aggregate shapes,
> modules/imports, constants, reference use, generics/traits, closures, I/O,
> allocation/drop, accelerators,
> ABI/layout, SIMD, tensors, performance, safety, stability, releases, and language
> completion remain excluded. Broad integers/fixed arrays remain `PARTIAL`, and
> `stable-scalar-v0` remains the only `STABLE` profile.

> **CAP-013 accepted:** protected master gives the already admitted generic-struct,
> generic-enum, generic-function, fixed-capacity `Window<T>`, and bounded trait-
> signature paths one canonical recursive specialization identity and deterministic
> phase plan. `int`/`i32` and `float`/`f64` now interoperate without duplicate private
> identities; telemetry mixes `Window<i32>`/`Window<int>` and trait `i32`/`int` at
> exact output/exit 91. Focused 9/9, established generic/trait 21/21, authority 7/7,
> representative 3/3, complete 249-library plus integration/doc/format/Clippy,
> corruption, and pinned LLVM 22 O0/O2 gates pass. Exact candidate `1ecf083`,
> protected PR #48 merge `856fc1e5`, shared tree `627582e2`, and all candidate and
> merge-head workflows pass. This does not add general generics/traits, new body
> semantics, reference specialization, collections, ABI/layout, allocation/drop, accelerators,
> safety, stability, releases, or completion.

> **CAP-012 accepted:** protected master composes
> immediate, nonescaping `&place` / `&mut place` arguments over nested finite
> CopyData fields, tuples, and fixed arrays with ordinary nongeneric reference calls.
> Shared classification, checked root/source/type/lifecycle metadata, verifier
> corruption controls, a richer telemetry application, and the complete gate pass.
> Exact candidate `79d1486`, all public candidate results, protected PR #46 merge
> `49bcdfc3`, and exact merge-head CI/Rust CI/CodeQL pass, including pinned Windows
> LLVM/Clang 22 native execution. This accepted slice does not add
> stored references, reference results, partial moves, disjoint alias reasoning,
> generic/method/trait call expansion, dynamic collections, lifetimes/drop, ABI,
> safety, or stability.

> **CAP-011 accepted:** protected master composes existing generic structs, fixed
> arrays, runtime bounds guards, projected
> mutation, and compile-time specialization into a reusable fixed-capacity
> `Window<T>` API. The telemetry program uses generic read/update functions for both
> `int` and `char`; focused 4/4, representative 3/3, identity, full-root, check, docs,
> format, and diff gates pass. Exact candidate `dea5714e`, all nine public results,
> protected PR #44 merge `34b81eee`, and exact merge-head checks pass, including
> pinned Windows LLVM 22 native execution. Dynamic collections, allocation, general generic
> operations/calls/borrowing, lifetimes/drop, public ABI, safety, and stability remain
> unsupported.

> **CAP-010 accepted:** protected master admits one required-only static-dispatch slice
> for nongeneric recursive-CopyData
> structs. A direct `T: Trait` generic parameter may call exact immutable `&self`
> methods with CopyData/`Void` signatures; concrete calls become verifier-bound
> monomorphic LLVM helpers. The telemetry application uses the same policy trait for
> `Sensor` and `Batch` and retains exact score/exit 91. Focused 3/3, representative
> 3/3, corruption controls, neighboring suites, and the full repository gate pass.
> Exact candidate `2e0bfde` passed all nine public results; protected PR #42 merged it
> as accepted master `f77f1a2` with identical tree, and exact merge-head CI, Rust CI,
> and CodeQL pass. Defaults, associated items, supertraits, generic
> traits/impls, dynamic dispatch, trait objects, non-CopyData targets, generic-to-
> generic calls, lifetimes/drop, ABI/FFI, safety, and general trait/generic support are
> not claimed.

> **CAP-009 accepted:** public library checking/compilation and
> CLI `check`, `build`, and `run` can explicitly select
> `--language-profile stable-scalar-v0`. One exhaustive pre-semantic classifier admits
> only the documented one-file, acyclic nongeneric `int`/`bool` function class, and a
> sealed checked-program identity selects its exact wrapping LLVM `i32` lane. The
> profile requires the CPU target and no `--gpu` selector. The default experimental
> profile remains unchanged. Focused 10/10, representative exits 91 and 93, and the
> complete repository gate pass. Exact candidate `bfd03ff` passed all nine public
> checks; protected PR #40 merged it as accepted master `1ef21c5` with identical tree,
> and exact merge-head CI, Rust CI, and CodeQL pass. This profile does not stabilize Aero as a whole,
> and modules/imports, floats/chars/strings/I/O, aggregates, enums/`Match`, references,
> closures, methods, general loops, division/remainder, traits/generics, recursion,
> allocation, lifetimes/drop, unsafe, ABI/FFI, accelerators, benchmarks, and releases
> remain excluded.

> **Project status after CAP-023:** Aero remains a Minimal Prototype in correctness
> recovery, not a complete or stable language.
> The accepted public baseline is protected CAP-023 merge.
> CAP-019 is Aero's latest accepted compiler/profile capability widening and remains
> the compiler boundary beneath the
> CAP-020, CAP-021, and CAP-023 product-only checkpoints.
> CAP-015 is the latest accepted project integration checkpoint.
> CAP-015 remains the accepted M1-001 representative-integration checkpoint. CAP-015
> changes no compiler production or language-profile code. CAP-016 and CAP-017
> remain completed readiness/architecture stops, not accepted capabilities;
> neither adds a profile or matrix row. CAP-013
> remains the single shared specialization identity/phase authority; CAP-018 and
> CAP-019 add no specialization classifier.
> CAP-011 and CAP-012 satisfy the roadmap's selected Milestone 2 exit product, while
> CAP-023 supplies another bounded selected product; general ownership, generics,
> collections, layout/ABI/destruction, and ordinary-program breadth remain partial.
>
> The fresh post-CAP-023 ranking uses 1--5 scores; higher risk/evidence favorability
> means safer and cheaper delivery:
>
> | Rank | Capability gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Favorable risk | Favorable evidence cost | Total |
> |---:|---|---:|---:|---:|---:|---:|---:|---:|
> | 1 | Accepted-head CAP-023 inference correctness/reproducibility/artifact-footprint evidence gate with no performance claim | 4 | 5 | 5 | 5 | 5 | 4 | 28 |
> | 2 | Exact CPU + recursive-CopyData application-profile composition readiness and red probe only | 5 | 5 | 5 | 5 | 2 | 2 | 24 |
> | 3 | Small quantized numerical-kernel readiness and red probe under one frozen cross-platform arithmetic-and-representation contract only | 5 | 5 | 3 | 5 | 1 | 1 | 20 |
>
> The next action is the accepted-head CAP-023 inference correctness,
> reproducibility, and artifact-footprint evidence gate with no performance claim.
> It is a zero-production immutable-evidence task, not compiler or benchmark work.
> Rank 2 is exact CPU plus recursive-CopyData application-profile composition
> readiness and a task-local red probe only, not implementation; it must freeze one
> bounded profile-composition contract without widening either existing profile.
> Rank 3 is small quantized numerical-kernel readiness and a task-local red probe only,
> not implementation; its arithmetic, representation, malformed-state, and
> cross-platform oracle contracts remain unfrozen. No row permits implementation.

> **CAP-008 accepted:** terminal `_ => fallback` and ignored
> payload leaves such as `Err(_)` execute across every already-admitted
> concrete enum class through one shared semantic/checked-admission arm resolver.
> Lowering retains one independently verified target per declared variant; `_` creates
> no binding or payload extraction. Focused 4/4, the complete 235-library/32-binary/
> integration/doc/format/Clippy/verifier-control gate, and official Windows LLVM/Clang
> 22.1.8 external, machine, native `-O0`/`-O2`, and public-run evidence pass at exact
> telemetry output and exit 91. Exact candidate `9ebd204`, protected PR #38 merge
> `a1716f8`, all candidate-head checks, and exact merge-head CI `31525340621`, Rust CI
> `31525340810`, and master-push/CodeQL `31525340605` pass. Guards, nested destructuring, general error
> propagation, collections, imports, ownership/drop expansion, stable ABI, safety,
> releases, benchmarks, and accelerators remain unsupported or unchanged.

> **CAP-007 accepted:** public artifact-free
> `check_program`/`check_file` APIs and library/CLI compile, check, build, run,
> profile, and source-test routes share one lex/parse/direct-module/semantic/
> checked-IR/internal-verification preparation authority. The prior semantic-only
> `aero test` false-success is closed. Focused 3/3 and the complete normalized
> 235-library/32-binary/84-integration/doc/format/Clippy gate pass. No source semantics
> or stability classification changes. Cached official Windows LLVM/Clang 22.1.8
> external/machine verification and native `-O0`/`-O2` exact output/exit 91 also pass;
> exact candidate `bfb7adb`, protected PR #35 merge `5a64aca`, every candidate-head
> check, and exact merge-head CI/Rust CI/CodeQL pass.

> **CAP-005 accepted:** bounded compile-time specialization admits unique top-level
> bound-free generic functions over exact recursive finite CopyData substitutions.
> Abstract type-parameter values
> may only be transported whole through direct parameters, explicitly typed locals,
> branch selection, reassignment, and return. The representative telemetry application
> uses one `choose<T>` helper for both `Reading<int>` and `Reading<char>` while
> preserving exact output and exit 91. Focused 5/5, shared-contract 2/2, corruption,
> compatibility, complete 232-library/32-binary/integration/doc/format/Clippy, and
> pinned LLVM/Clang 22.1.8 verification plus native `-O0`/`-O2` gates pass locally.
> Bounds/traits, generic methods/enums/impls, recursive or generic-to-generic calls,
> operations on abstract values, result-only inference, nested generic signatures,
> non-CopyData arguments, collections, lifetimes/drop, stable ABI, and general generic
> or safety claims remain unsupported. Exact candidate `68e2cd8`, all nine
> candidate-head checks, protected PR #31 merge `59f7e47b`, exact merge-head CI
> `31504122753`, Rust CI `31504122730`, and CodeQL `31504122424` pass.

> **CAP-006 accepted:** one bound-free user-defined generic enum can be
> specialized at exact explicit recursive finite CopyData arguments, constructed only
> from exact binding/reassignment/nongeneric parameter or result context, transported
> as an owned enum, and exhaustively matched through the existing checked pipeline.
> The representative application executes `Sample<Reading<int>>` and `Sample<char>`
> at unchanged output/exit 91. Focused 4/4, corruption, compatibility,
> representative 3/3, and complete 235-library/32-binary/83-integration/doc/format/
> Clippy local gates pass. Exact candidate `5f20a554`, candidate-head workflows,
> protected PR #33 merge `bdfd4f5a`, and exact merge-head CI/Rust CI/CodeQL pass;
> pinned LLVM/Clang 22 Linux/Windows `-O0`/`-O2` execution is green. Bounds/traits, named
> variants, nested generic templates, context-free inference, non-CopyData arguments,
> aggregate/reference storage, borrowing, public ABI, collections, lifetimes/drop,
> release, stability, and general safety remain unsupported.

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
> Linux/Windows LLVM/Clang 22.1.8 `-O0`/`-O2` execution pass. CAP-004 itself did not add
> generic functions/enums/impls/traits, inference/defaults, applications inside generic definitions, non-CopyData
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
> bounds, runtime checks, cyclic aggregates, generics beyond accepted CAP-005,
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
