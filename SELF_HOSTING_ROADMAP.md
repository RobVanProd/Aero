# Aero Self-Hosting Roadmap

Last reviewed: 2026-08-15 (America/New_York)

This is the canonical dependency path from Aero's current Rust bootstrap
compiler to a reproducible Aero-authored compiler. It records gates, not dates.
The current accepted baseline is CAP-033 merge
`9ea6fcb4e703158b712c16f83fd30285316d5609`, tree
`d77ef5d67936449ef99942db4a9e68d16580ecdf`.

For current feature truth, use
[`SPEC_IMPLEMENTATION_MATRIX.md`](SPEC_IMPLEMENTATION_MATRIX.md) and
[`PROJECT_STATE.md`](PROJECT_STATE.md). For the founding direction and broader
roadmap, use [`FRAMEWORK_ALIGNMENT.md`](FRAMEWORK_ALIGNMENT.md) and
[`Roadmap.md`](Roadmap.md). Exact task authorization remains in
[`TASK_LEDGER.md`](TASK_LEDGER.md).

## What counts as self-hosting

Aero is self-hosted only when all of the following are true:

1. The compiler implementation used for the claim is authored in Aero.
2. The trusted Rust stage-0 compiler builds that source into a stage-1 Aero
   compiler.
3. Stage 1 compiles the same canonical Aero compiler source into stage 2 without
   calling the Rust compiler for lexing, parsing, semantic analysis, checked IR,
   verification, or code generation.
4. Stage 1 and stage 2 satisfy a comparison contract frozen before the claim.
   The target is byte-identical compiler-emitted LLVM plus a canonical linked
   artifact manifest; any deliberately ignored platform metadata must be named
   and independently checked.
5. Both stages pass the same compiler conformance, negative-program,
   source-to-native, and clean-checkout system gates on every claimed platform.
6. The complete trust base is listed: stage 0, the Aero runtime, LLVM/Clang,
   linker, operating-system interfaces, and bootstrap scripts or source-bundle
   tooling.

An Aero-written lexer sample, accepted syntax, generated LLVM text, an Aero
wrapper around Rust compiler logic, or a compiler-shaped benchmark is not
self-hosting. Those can be valuable checkpoints only when labeled precisely.

## Current position

The project remains bootstrapped by the accepted Rust stage 0, and **K1 is now
accepted**: the repository tracks an Aero-written
function that scans a source-embedded `[int; 64]` and logical length into 24
fixed token-kind/start/length records. An independent Rust oracle covers 18
valid and invalid fixtures, the public exact-profile routes pass, deterministic
LLVM is verified, protected Linux and Windows O0/O2 native execution returns 91
with empty output, and candidate plus post-merge gates are green. No compiler
production file changed. K1 remains fixed-storage evidence, not runtime source
ingestion, dynamic token storage, Unicode lexing, a production-lexer
replacement, or self-hosting.

P1's public selector red is preserved. CAP-033 is accepted, so every normalized
surface witness now carries exact file-scope or enclosing-function origin. The
resumed P1 implementation reached its first whole-repository compile gate and
correctly stopped: one existing characterization exhaustively matches the three
accepted `LanguageProfile` variants even though its local loop supplies only two.
CAP-034 is the narrow test-only prerequisite that makes that local expectation
future-selector-safe without changing any accepted compiler behavior. P1 resumes
only after CAP-034 is protected and must explicitly record the public Rust enum
source-compatibility consequence of adding its fourth selector.

The first hard runtime blocker is **R1: owned bytes**. Legacy Vec-shaped IR names
exist in [`ir.rs`](src/compiler/src/ir.rs), but the checked verifier and backend
deliberately reject every Vec instruction. They are dormant historical surface,
not an allocator, collection contract, or implementation. The LLVM backend
declares `printf` and `llvm.trap`; it has no accepted `malloc`, `realloc`,
`free`, or file-read runtime path.

## Dependency path

```mermaid
flowchart TD
    S0["S0: accepted Rust stage-0 baseline"] --> K1["K1: bounded Aero lexer kernel"]
    K1 --> P1["P1: compiler-oriented exact record/Result profile"]
    P1 --> R1["R1: owned byte buffer, allocation, growth, drop"]
    R1 --> R2["R2: whole-stream byte input"]
    R1 --> D1["D1: owned compiler collections and flat AST arena"]
    R2 --> F1["F1: Aero lexer and parser over runtime input"]
    D1 --> F1
    F1 --> M1["M1: Aero semantic analysis and checked IR"]
    M1 --> B1["B1: Aero verifier, LLVM emitter, and driver"]
    B1 --> H1["H1: stage-0 to stage-1 to stage-2 convergence"]
    H1 --> H2["H2: accepted reproducible self-hosting claim"]
    R2 -. "later maintainable source graph" .-> G1["G1: positive modules/imports/namespaces"]
    D1 -.-> G1
    G1 -.-> F1
```

K1 is executable specification work, not a substitute for R1 or R2. A first
single-file or deterministically bundled bootstrap compiler may proceed before
the complete G1 module graph, provided the bundle step is non-semantic,
reproducible, declared in the trust base, and does not call the Rust compiler.
Positive modules remain required before Aero can claim a maintainable native
compiler project rather than only a bootstrap bundle.

## Stage gates

| Stage | Status | Required product | Evidence to advance | Explicit non-claim |
|---|---|---|---|---|
| S0 — trusted bootstrap baseline | **Accepted** | Protected Rust compiler checkpoint with one checked preparation route, independent checked-IR verification, deterministic selected-profile LLVM, and native gates | Exact accepted commit/tree, full root gate, stable/nightly, Windows LLVM 22, CodeQL, and accepted-head workflow evidence | The Rust compiler is not self-hosted and the whole language is not stable |
| K1 — bounded compiler kernel | **Accepted** | Aero function that lexes a fixed ASCII buffer and logical length into fixed `[status, count, kind, start, length, ...]` storage | Independent Rust oracle; 18 valid/invalid fixtures; deterministic verified LLVM; protected Linux/Windows O0/O2 native parity; exact candidate/merge/post-merge evidence | No runtime text, file input, Unicode, dynamic tokens, production-lexer replacement, or self-hosting |
| P1 — selected compiler subset | **Selector red preserved; CAP-033 locally green pending acceptance** | A post-semantic exact profile for the already frozen record, concrete `Result`, exhaustive `Match`, flat exact-array, `int`, and `bool` surface | Red-first pre-IR admission; exact root/function context; CAP-030 surface witness consumption; CAP-029 authentication; CAP-026 exact layout; unchanged existing profiles | No general enums, generics, references, modules, allocation, ABI, or broad stability |
| R1 — owned bytes | **Blocked** | One byte-specific owned growable buffer with length, capacity, initialized range, allocation failure, move, alias, reallocation invalidation, and exactly-once destruction contracts | RFC/readiness proof; source negatives; checked ownership identity; verifier corruption matrix; allocation-failure and drop tests; sanitizer or equivalent runtime evidence | Dormant Vec IR and simplified stdlib helpers do not satisfy this gate |
| R2 — host byte input | **Blocked on R1** | Deterministic whole-stream byte ingestion, preferably stdin before general path semantics | Empty/short/large input, partial reads, EOF, invalid length, I/O failure, Linux/Windows parity, no uninitialized bytes | `println!`/`printf` output is not input or file I/O |
| D1 — compiler data model | **Future** | Owned token storage, interned or owned names, maps/sets required by scopes, and a flat append-only AST arena using integer node IDs | Growth/failure/drop evidence; cycle-free arena validation; deterministic iteration; large-source stress; no host collection substitution | The current Rust `String`/`Vec`/`Box` AST is not available to Aero source |
| G1 — source graph | **Future; may trail first bundled bootstrap** | Positive modules/imports, namespaces, collision and cycle rules, visibility, canonical file identity, and deterministic traversal | Multi-file positive/negative corpus, cycle and ambiguity diagnostics, cache identity, cross-platform path rules | Current direct module collection and parsed-but-rejected imports are not this gate |
| F1 — Aero front end | **Future** | Aero lexer and parser consuming R2 bytes and producing D1 tokens/AST with deterministic locations and diagnostics | Differential oracle against the accepted grammar; malformed-source corpus; fuzz/property tests; bounded-memory failure behavior | K1's fixed kernel is not the production front end |
| M1 — semantic compiler core | **Future** | Aero name/type/ownership analysis, normalized profile facts, checked IR construction, and fail-before-backend behavior | Differential valid/invalid corpus; checked-IR structural equality or frozen equivalence; corruption controls; determinism | Parsing success does not prove semantics or ownership safety |
| B1 — trusted Aero backend path | **Future** | Aero checked-IR verifier, deterministic LLVM emitter, and a driver that invokes the declared LLVM/link trust base | Invalid IR rejection, LLVM verifier, O0/O2 object lowering, native system corpus, artifact hygiene | Emitting plausible LLVM without independent verification is not a compiler gate |
| H1 — bootstrap convergence | **Future** | Stage 0 builds stage 1; stage 1 builds stage 2 from the same canonical Aero compiler source | Clean isolated builds, frozen environment/toolchain manifest, raw LLVM comparison, canonical linked-artifact comparison, repeated-build equality | A single successful stage-1 build is not convergence |
| H2 — accepted self-hosting | **Future** | Protected, reproducible H1 result plus the complete declared platform and conformance surface | Immutable manifests/artifacts, independent replay, exact candidate/merge identity, post-merge replay, truthful documentation | No stability, memory-safety, performance, accelerator, or release claim follows automatically |

## Current gaps and their owners

| Gap | Current repository evidence | Owning work before the gate can close |
|---|---|---|
| Runtime-sized source bytes | Selected exact execution uses fixed nonempty integer arrays. General runtime/file input remains excluded. | R1 then R2; do not widen fixed-array evidence into a runtime-ingestion claim |
| Allocation and destruction | No accepted allocator ABI or drop path exists. Vec instructions at `ir.rs:358-383` are rejected at `ir_verifier.rs:867-873` and `code_generator.rs:1192-1222`. | One ownership/runtime vertical slice with allocation-failure and exactly-once-drop semantics |
| Owned text and names | The Rust lexer builds `String` values and returns `Vec<LocatedToken>`; Aero runtime String ownership is not accepted. | Byte buffer first, then an explicit UTF-8/ASCII and owned-name contract; the first bootstrap subset may remain documented ASCII |
| Recursive compiler data | `ast.rs` uses `String`, `Vec`, and `Box` throughout expressions, statements, patterns, types, and blocks. | Prefer D1 flat arenas and integer IDs before recursive heap objects |
| Compiler-safe selected surface | `stable-scalar-v0` is too narrow; `exact-i32-array-v0` rejects records, typed carriers, modules, I/O, and allocation. | P1 may admit only already frozen record/Result composition after CAP-026 through CAP-030 prerequisites |
| Input errors and typed failure | Experimental typed carriers exist, but no compiler-oriented selected profile and runtime I/O failure contract compose them. | P1 for bounded typed failure; R2 for actual host failures |
| Modules and names | Root `mod` collection is flattened and bounded; executable imports, namespaces, visibility, recursive graphs, cycles, and separate compilation are absent. | G1 after owned bytes/names and deterministic collections; a declared single-file bootstrap may precede it |
| Front-end fidelity | The production lexer/parser are Rust and allocate dynamically. Accepted CAP-031 handles only fixed source-embedded ASCII storage. | K1 establishes bounded token-policy evidence; F1 replaces fixed storage only after R1/R2/D1 |
| Semantic and ownership fidelity | The Rust semantic analyzer uses multiple scopes, registries, maps, fixed-point ownership state, and normalization authorities. | M1 must port bounded authorities with differential and corruption evidence rather than re-infer convenient types |
| Checked IR and verification | Current checked IR and verifier are trusted Rust authority. | M1 constructs it; B1 independently verifies it before emission |
| Code emission and driver | LLVM text, cache, CLI, process execution, object lowering, and linking are Rust-owned. | B1 plus a narrow runtime/process/file contract; LLVM and the linker can remain declared external tools |
| Bootstrap reproducibility | Product and claim evidence are reproducible, but no stage compiler or convergence manifest exists. | Freeze H1 comparison inputs, environment, ignored metadata, and failure rules before the first stage build |
| Cross-platform trust | Existing Linux and Windows gates cover bounded accepted programs, not an Aero compiler executable. | Every bootstrap stage must run the same corpus on each claimed host; CPU self-hosting does not imply ROCm/CUDA execution |

## Minimum compiler subset

The bootstrap compiler should target the smallest source subset that can express
the compiler correctly, not all experimental Aero syntax. Its contract must be
selected explicitly and fail closed before IR. Expected ingredients are:

- exact `int`/`i32`, `bool`, fixed byte/int buffers, records, concrete typed
  failure, exhaustive `Match`, loops, and ordinary nongeneric calls;
- owned byte buffers and deterministic flat arenas;
- no implicit conversion, fallback typing, hidden Rust collection, or unchecked
  allocator path;
- ASCII source first if frozen as such, with Unicode deferred rather than
  partially claimed;
- CPU execution first; ROCm and CUDA remain separate capability programs.

This list is a dependency target, not authorization for the proposed P1 profile
or R1 runtime semantics. Each behavior-changing slice still requires its own
ledger and failing regression first.

## Evidence checklist for every bootstrap checkpoint

- Exact accepted base commit and tree.
- Frozen positive behavior, negative boundary, allowed files, risks, and stop
  conditions in `TASK_LEDGER.md`.
- A failing regression before any behavior change.
- Independent oracle or corruption control; self-checks alone are insufficient.
- Focused tests, neighboring compatibility rings, formatting, correctness
  Clippy, `git diff --check`, and `./tools/test.sh`.
- Deterministic emitted artifacts and empty/unrequested-artifact checks.
- Linux and Windows native evidence for every claimed CPU product.
- Exact candidate, protected merge, and post-merge workflow identities.
- Explicit remaining exclusions and no automatic stability, safety,
  performance, accelerator, or release promotion.

## Exact next task

Execute and protect `CAP-034-LANGUAGE-PROFILE-TEST-EXTENSIBILITY` from accepted
CAP-033, then resume CAP-032/P1 from that protected head. P1 must
consume the context-bound CAP-030 surface witnesses and
CAP-029 verified-IR authentication to admit only the frozen compiler-oriented
combination of exact `int`/`bool`, flat arrays, nongeneric records, concrete
typed `Result`, and exhaustive `Match`. Existing profiles and public behavior
must remain byte-for-byte and diagnostic-for-diagnostic unchanged; the new
public enum variant is the one explicitly documented source-compatibility
expansion.

Stop rather than infer missing expression types, duplicate semantic/profile
authority, widen to general enums or generics, change checked IR, introduce a
third compiler phase, or claim allocation, runtime input, modules, ABI,
accelerator execution, or self-hosting. R1 remains a separate ownership/runtime
readiness contract after P1. The roadmap advances only when evidence closes a
gate.

## Deliberately absent schedule

This roadmap does not estimate calendar time. Progress is measured by accepted
dependency gates because implementation speed does not remove semantic,
ownership, verification, or reproducibility obligations.
