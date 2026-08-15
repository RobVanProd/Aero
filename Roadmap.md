# Aero Development Roadmap

Last updated: 2026-08-13 (America/New_York)

This roadmap translates Aero's founding Design -> Minimal Prototype -> Self-Host
-> Stabilize -> Optimize path into evidence-gated engineering milestones. A
milestone title or implemented interface does not certify a feature. Current
capability is defined only by `SPEC_IMPLEMENTATION_MATRIX.md`,
`BACKEND_STATUS.md`, tests, and retained artifacts.

The project is currently in **Minimal Prototype / correctness recovery**.
Historical completed-phase and `v1.0.0` labels do not mean that Aero is stable,
self-hosted, or release-ready.

CAP-019 is accepted as initialized mutable exact-array production inside CAP-014's
existing CPU-only `exact-i32-array-v0` profile. Accepted CAP-014 created the CPU-only
`exact-i32-array-v0` profile; accepted CAP-018 remains its immutable exact-array
result-composition checkpoint; accepted CAP-019 widens that same profile with fully
initialized mutable owned locals, direct projected element writes, and returned
flat-array values rather than creating another profile. Accepted CAP-019 widens the
existing flat nonempty exact-`Int` class to a fully initialized mutable owned local
whose initializer is an admitted literal, immutable exact-array identifier, or acyclic
ordinary call of the same count, plus direct
`local[index] = exact_int_value` projected writes. The maintained eight-lane
application copies an immutable input, increments every lane in a guarded loop,
returns the whole array by value, feeds it into the accepted CPU kernel, preserves all
eight source lanes, produces result `2035`, and exits `91`; Linux and Windows retain
read traps and add negative/equal-to-count write traps under verified LLVM/Clang 22
`-O0`/`-O2` routes. The single selected `exact-i32-array-v0` row remains
`END_TO_END`; broad integer and fixed-array support remains `PARTIAL`;
`stable-scalar-v0` remains Aero's only `STABLE` profile. CAP-019 does not admit general
mutable arrays, uninitialized or partial arrays, mutable parameters/results/aliases,
references or escaping places, whole-array reassignment,
zero/recursive/nested/repeat/non-Int arrays, stable aggregate ABI/layout, general
parsing/string/file behavior, GPU execution, performance, or safety.

CAP-020 is accepted as a zero-production flat-buffer 2x3-by-3 matvec product gate.
Accepted CAP-020 changes no parser, grammar, source semantics, language profile,
semantic analysis, checked IR, verifier, backend, ABI, or capability classification;
it is a zero-production product/evidence checkpoint over CAP-019's
`exact-i32-array-v0` surface. The accepted application encodes a 2x3 matrix as
`[int; 6]`, consumes an `[int; 3]` vector, computes wrapping `row * 3 + column` in
nested loops, returns a fully initialized mutable-produced `[i32; 2]`, preserves every
input lane, produces ordinary and wrapping results `[50, 122]` and `[-2, 5]`, and exits
`91`. The computed linear value flows through the existing signed bounds and
trap-before-address authority before a `[6 x i32]` load, with corresponding guarded `[3 x i32]`
load and `[2 x i32]` store. CAP-020 adds no matrix type, recursive or nested arrays,
static index proof, checked-overflow arithmetic, stable layout or ABI, performance,
accelerator execution, general mutation, or safety claim. CAP-019 remains the latest
compiler/profile capability widening; CAP-020 is an accepted product gate, not a
separate profile or feature row.

CAP-021 is accepted as a zero-production source-embedded two-stage exact-i32 scoring
product gate.

Accepted CAP-021 changes no parser, grammar, source semantics, language profile,
semantic analysis, checked IR, verifier, backend, ABI, or capability classification;
it is a zero-production product/evidence checkpoint over CAP-019's
`exact-i32-array-v0` surface and composes the accepted CAP-020 flat matvec.

The accepted application treats one source-embedded flat `[int; 17]` as an
application record with exact header `[2, 3, 1]`, dynamically decodes input,
row-major first-stage weights, first-stage bias, second-stage weights, and score bias
into fully initialized flat locals, then composes the accepted 2x3 matvec with
wrapping bias and affine scoring.

The accepted scorer returns `[valid, raw0, raw1, hidden0, hidden1, score]`; its
ordinary result is `[1, 122, 167, 135, 181, 4938]`, its wrapping result is
`[1, -24, 18, 2147483623, -2147483631, -2147483627]`, an invalid header returns six
zeros, both valid source records preserve and reread all 17 lanes, and the application
exits `91`.

Every dynamic read and write uses the existing signed bounds, trap-before-address,
`sext`, typed-GEP, and same-pointer consumer authority; exact public and pinned
Linux/Windows LLVM 22 verifier, O0/O2, native, and deterministic-emission evidence
passes.

CAP-021 adds no tensor, matrix, struct, record, recursive-array, nested-array,
serialization, runtime/file-input, quantization, activation, checked-overflow, stable
layout/ABI, performance, accelerator, safety, general inference, or
language-completion capability; the flat record is an application convention, not a
source or physical type.

CAP-019 remains the latest compiler/profile capability widening; CAP-020 and CAP-021
are accepted product gates, not separate profiles or feature rows.

The PR-only aggregate CodeQL check is correctly absent on the default branch; the sole
open finding remains pre-existing Actions alert #4 from 2026-08-09, and no new CAP-021
alert surfaced.

CAP-023 is accepted as a zero-production fixed-shape ReLU-and-argmax inference
product/evidence checkpoint.

Exact CAP-023 reviewed candidate
`63e6b00b6294de61e3afd292a1e32e2b014714e2`, shared candidate/merge tree
`4d234cdfde67f1083773e2c41be4ab92027769db`, accepted base and first merge parent
`4bce540dfed6dfffa152067f4e00424501a6cdd8`, and protected PR #62 merge
`e9b281504446465cfc8fcbe17c65cce92df0e83a` whose second parent is that candidate are
immutable. Candidate push CI `31687464571`, PR CI `31687585904`, Rust CI
`31687585893`, CodeQL `31687584263`, and aggregate candidate check `94407323731`;
candidate push/PR compiler jobs `94406770929`/`94407177877`, stable/nightly/Windows
LLVM 22 jobs `94407178006`/`94407178047`/`94407178042`, CodeQL Actions/Python/Rust
jobs `94407175858`/`94407175752`/`94407175820`, and Actions/Python/Rust analyses
`1612686978`/`1612687391`/`1612693654`; merge-head CI/Rust CI/CodeQL
`31688093145`/`31688093150`/`31688092749`, exact merge compiler/stable/nightly/Windows
LLVM 22 jobs `94408808914`/`94408809340`/`94408809458`/`94408809296`, merge CodeQL
Actions/Python/Rust jobs `94408812427`/`94408812194`/`94408812175`, and default-branch
Actions/Python/Rust analyses `1612715455`/`1612715345`/`1612721829` all pass.

Accepted CAP-023 changes no parser, grammar, source semantics, language profile,
semantic analysis, checked IR, verifier, backend, ABI, or capability classification.
It is a zero-production product/evidence checkpoint over CAP-019's unchanged
`exact-i32-array-v0` surface, composes the accepted CAP-020 flat matvec and CAP-021
record-to-score product, and does not reverse CAP-022's mandatory runtime-acquisition
`NO IMPLEMENTATION` stop.

One source-embedded application convention `[int; 20]` with exact header `[2, 3, 2]`
drives a flat 3-input/2-hidden/2-output wrapping-`i32` computation with strict-positive
zero clamp, two biased logits, signed strict-greater argmax, lower-index tie selection,
three independent malformed-header controls, and reread of all 140 source lanes after
all seven by-value calls.

Exact ordinary, wrapping, activation-boundary, and tie results are respectively
`[1, 122, 167, 135, 181, 4940, 5573, 1]`,
`[1, -24, 18, 2147483623, 0, -37, 2147483641, 1]`,
`[1, -3, 0, 0, 0, 5, 4, 0]`, and `[1, 1, 2, 1, 2, 3, 3, 0]`; malformed results are
eight zeros; public and native success is sentinel 91 with empty source stdout/stderr.

CAP-023 adds no general activation, ReLU, argmax, inference, tensor, matrix, record,
recursive-array, runtime/file input, serialization, quantization, conversion, stable
layout/ABI, performance, resource-usage, accelerator, safety, or language completion
capability.

Its record and topology are application conventions, its retained local artifacts
remain mutable corroboration only, and CAP-019 remains the latest compiler/profile
widening.

The sole matrix change is the existing CPU backend-summary row remaining `PARTIAL`;
no CAP-023 language feature or selected-profile row may be added.

CAP-023 merge `e9b281504446465cfc8fcbe17c65cce92df0e83a` is an accepted historical
product checkpoint, not the current public master. The PR-only aggregate is correctly
absent on the default branch. Default-branch Actions analysis `1612715455` contains
only the pre-existing open alert #4 created 2026-08-09; Python and Rust analyses contain
zero results; no new CAP-023 code-scanning alert exists.

The selected Milestone 0, Milestone 1, and Milestone 2 exit gates are met for their
bounded selected products; their broader milestone ambitions remain partial.
Milestone 3 remains open. CAP-023 advances its application and reproducibility
boundary but supplies no runtime ingestion, composed CopyData application profile,
quantization, runtime-resource measurement, performance evidence, accelerator
execution, or broader workload.

CAP-024 is accepted as the current evidence-only checkpoint.

Exact CAP-024 reviewed candidate
`617bfce86feb879ee5eef61b44cf4e2a5520f022`, shared candidate/merge tree
`9520f24e4f1626f16782a9775480f9653f6059bb`, accepted base and first merge parent
`918c9222eb61e2435e18847e30b946cd08013238`, and protected PR #64 merge
`2f7ec325e423461a8e867f4ee2573ae6dcf15dfd` whose second parent is that candidate are
immutable. Candidate push CI `31764763341`, PR CI `31764765501`, Rust CI
`31764765563`, CodeQL `31764763584`, and CAP-024 evidence run `31764765495`; candidate
push/PR compiler jobs `94658200345`/`94658206474`, stable/nightly/Windows LLVM 22 jobs
`94658207134`/`94658207170`/`94658207086`, CodeQL Actions/Python/Rust jobs
`94658203257`/`94658203263`/`94658203316`, aggregate candidate CodeQL check
`94658280067`, and Actions/Python/Rust analyses
`1617260890`/`1617261159`/`1617264144` all pass. Candidate CAP-024
Linux/Windows/aggregate jobs `94658206500`/`94658206555`/`94659098928` pass and artifact
`9205970753` carries fresh manifest
`bd5e609b4ce829579331a23170d6d9e4fc4d5906cb32779876a78bc24294812c` plus 132 fresh
observations `62780d81e9dcaa6e85c08d0805608a58283816dd062c3a8bb1a8c67971ac551f`;
its claim-bearing projection matches accepted canonical manifest
`4b4cfce95459761dddd588e09abb3046854e0c2afb361f08a9553f180f013a34`.
Merge-head CI `31765227712`, Rust CI `31765227675`, CodeQL `31765227317`, and CAP-024
replay `31765227673`; exact merge compiler/stable/nightly/Windows LLVM 22 jobs
`94659602474`/`94659602479`/`94659602493`/`94659602501`, CodeQL Actions/Python/Rust jobs
`94659604078`/`94659604103`/`94659604064`, default-branch analyses
`1617281747`/`1617282341`/`1617285598`, and CAP-024 aggregate replay job
`94659602932` all pass. The two default-branch capture jobs
`94659621233`/`94659603455` are correctly skipped because protected master validates
the tracked bundle rather than replacing accepted observations.

Current accepted public master and public evidence checkpoint is protected CAP-024
merge `2f7ec325e423461a8e867f4ee2573ae6dcf15dfd`, tree
`9520f24e4f1626f16782a9775480f9653f6059bb`; its ordered parents are accepted base
`918c9222eb61e2435e18847e30b946cd08013238` then reviewed candidate
`617bfce86feb879ee5eef61b44cf4e2a5520f022`.

CAP-024 is the current accepted public evidence checkpoint and protected public master.
It adds no compiler production, parser, grammar, source semantics, profile, semantic
analysis, checked IR, verifier, backend, example, product oracle, runtime behavior,
ABI, capability classification, benchmark, resource-usage, performance, accelerator,
safety, or general-inference capability. Its only claim is immutable accepted-head
CAP-023 correctness, within-platform target-artifact reproducibility, exact observable
behavior, and artifact byte-size footprint under the closed recorded boundary.

CAP-019 remains the latest compiler/profile widening; CAP-023 remains the latest
product checkpoint. The selected `exact-i32-array-v0` row and the existing CAP-023 CPU
backend-summary row remain byte-identical, and CAP-024 adds no language,
selected-profile, or backend-summary row.

The accepted catalog record remains
`aero_cap023_inference_correctness_918c9222_20260813`, status
`verified_correctness_reproducibility_only`, with exactly the tracked schema, canonical
88,734-byte manifest SHA-256
`4b4cfce95459761dddd588e09abb3046854e0c2afb361f08a9553f180f013a34`, oracle, and
reproduction contract.

The PR-only aggregate CodeQL check is correctly absent on the default branch.
Default-branch Actions analysis `1617281747` carries only the pre-existing open alert
#4 created and last updated 2026-08-09; Python and Rust analyses contain zero results,
and no new CAP-024 alert exists.

The selected Milestone 0, Milestone 1, and Milestone 2 exits remain met for their
bounded selected products; broader ambitions remain partial. Milestone 3 remains open.
CAP-024 closes the prior accepted-head correctness/reproducibility/artifact-footprint
gap, but supplies no runtime ingestion, composed CopyData application profile,
quantization, runtime-resource measurement, performance evidence, accelerator
execution, or broader workload.

Block-local `mod missing;` remains a demonstrated invalid-program false success because
the common statement parser accepts it, `ModDecl` has no source location, and semantic
plus checked admission silently discard it. CAP-016 already audited that exact defect
and found that trustworthy placement/provenance rejection participates in the
unfrozen module migration across more than two compiler phases. No new module RFC or
decision-changing evidence exists, so CAP-016 remains a mandatory `NO IMPLEMENTATION`
stop until its explicit re-entry condition is met.

CAP-018 is accepted as immutable exact-array value/result composition and remains the
historical immutable-result checkpoint in this same profile.

Exact CAP-019 reviewed candidate
`f2955bedd22708041e36ee90c65c4f08c443d740`, shared candidate/merge tree
`c520729e7b081087bbe431e97d937fb77f519b37`, accepted base and first merge parent
`84916e124752b8e7d228855a0969cd9eab8dba26`, and protected PR #56 merge
`6ebeb0efb6e83ccc50e12d395e4add1c63ef48b4` whose second parent is that candidate
are immutable. Candidate push/PR CI `31627264709`/`31627385522`, Rust CI
`31627385563`, CodeQL `31627405516`, and aggregate `94217394313`; merge-head
CI/Rust CI/CodeQL `31627880853`/`31627880924`/`31627880812`, exact merge jobs
`94218938557`/`94218938794`/`94218938835`/`94218939033`/`94218943455`/
`94218943514`/`94218943605`, and default-branch Actions/Python/Rust analyses
`1609396076`/`1609396442`/`1609401493` all pass.

Exact CAP-020 reviewed candidate
`3b61cd1ed34f910f556821942cd06301ba17dd50`, shared candidate/merge tree
`800510de85bd82f3332126ad249c95da109dd3e1`, accepted base and first merge parent
`13157687f3e955d1c8292ccca133c5a73e29e1a7`, and protected PR #58 merge
`d9493d5123840b38ebab6ca275aaba3216728706` whose second parent is that candidate are
immutable. Candidate push CI `31639493741`, PR CI `31639540134`, Rust CI
`31639540030`, CodeQL `31639535638`, and aggregate candidate check `94258433541`;
candidate stable/nightly/Windows LLVM 22 jobs
`94258276078`/`94258275978`/`94258275899` and CodeQL Actions/Python/Rust jobs
`94258264605`/`94258264489`/`94258264627`; merge-head CI/Rust CI/CodeQL
`31640016314`/`31640016316`/`31640015733`, exact merge compiler/stable/nightly/Windows
LLVM 22 jobs `94259869631`/`94259869676`/`94259869637`/`94259869559`, merge CodeQL
Actions/Python/Rust jobs `94259873136`/`94259873164`/`94259873086`, and default-branch
Actions/Python/Rust analyses `1610137115`/`1610137589`/`1610144660` all pass.

Exact CAP-021 reviewed candidate
`f91df56084540d30f3c8d09e71c5f30db280fd93`, shared candidate/merge tree
`7e34b4b8e817a7aafaaabc6326fa0a4d616fcc91`, accepted base and first merge parent
`df0626916d190d8a7580f783e3ac24a89f691617`, and protected PR #60 merge
`59af445ea02c1759d337d698be9c4f4472587aaf` whose second parent is that candidate are
immutable. Candidate push CI `31670574143`, PR CI `31670599830`, Rust CI `31670599826`,
CodeQL `31670598033`, and aggregate candidate check `94354297550`; candidate push/PR
compiler jobs `94354135184`/`94354214336`, stable/nightly/Windows LLVM 22 jobs
`94354214389`/`94354214394`/`94354214410`, CodeQL Actions/Python/Rust jobs
`94354210797`/`94354210770`/`94354210832`, and Actions/Python/Rust analyses
`1611711722`/`1611712334`/`1611716646`; merge-head CI/Rust CI/CodeQL
`31671091285`/`31671091296`/`31671091099`, exact merge compiler/stable/nightly/Windows
LLVM 22 jobs `94355683766`/`94355683532`/`94355683515`/`94355683534`, merge CodeQL
Actions/Python/Rust jobs `94355685544`/`94355685480`/`94355685574`, and default-branch
Actions/Python/Rust analyses `1611737053`/`1611737605`/`1611740699` all pass.

The sole open finding remains pre-existing Actions alert #4 from 2026-08-09; no new
CAP-020 alert surfaced.

Exact CAP-018 candidate `409eca9ed2dd8b4ba79f34e14ecfefcc0386e3df`, tree
`3073c881c883984f53fcde2f0b205acbec760145`, and protected PR #54 merge
`c49ff17cab7fc0e8d4f552a71499929135c16c61` are immutable. Candidate push/PR/Rust/
CodeQL runs `31614934307`, `31614994226`, `31614994253`, and `31614991761` pass.
Merge-head CI/Rust/CodeQL runs `31615467151`, `31615467115`, and `31615465499`
pass, and default-branch Actions/Python/Rust analyses `1608636029`, `1608636345`, and
`1608644785` pass.

CAP-014 remains the profile origin and first bounded Milestone 3 CPU slice. CAP-015
remains the accepted M1-001 representative-integration checkpoint. CAP-015 changes no
compiler production or language-profile code. CAP-016 and CAP-017 remain completed
readiness/architecture stops, not accepted capabilities; neither adds a profile or
matrix row. CAP-013 remains the single shared specialization identity/phase authority;
CAP-018 and CAP-019 add no specialization classifier.

CAP-014 is accepted as the first bounded Milestone 3 CPU computation slice. Its
distinct CPU-only `exact-i32-array-v0` profile composes exact wrapping LLVM `i32`
arithmetic with flat fixed integer arrays, by-value nongeneric calls, guarded runtime
indexing, and one tracked eight-lane dot-product-plus-bias kernel at independent-
oracle exit 91. A second specimen proves wrapping edge behavior at exit 93. The
profile's one shared source classifier and one verifier-fed physical mapping leave
semantic analysis, checked IR, and independent verification unchanged. Exact
candidate `226279dd174f26dc3cd1c7573798955bfe789f78`, protected PR #50 merge
`ca09ebe3c1b981339c8bf56b360e62208ac900e1`, shared tree
`448e1c2ff397012804b886b904aa43bec63f2d37`, candidate push/PR CI
`31570455915`/`31570461500`, Rust CI `31570461524`, CodeQL `31570456382`, and
merge-head CI/Rust CI/CodeQL `31570823665`/`31570823712`/`31570823073` all pass,
including pinned Linux and Windows LLVM 22 gates. This is one private bounded CPU
reference computation. Accepted CAP-018 subsequently admits immutable exact array
results rooted in literals, identifiers, or ordinary acyclic calls across result,
binding, call-argument, and index-object placements within this same profile. Accepted
CAP-019 subsequently admits only fully initialized owned mutable locals, guarded direct
element writes, and returned flat-array values in the selected class. Recursive or
nested arrays, mutable parameters/results/aliases, references or escaping places,
whole-array reassignment, non-integer elements, surrounding aggregate use,
modules/imports, constants, methods, generics/traits, closures, collections,
allocation/drop, I/O, accelerators, and non-CPU target pairing remain rejected. The
cumulative lane does not stabilize ABI/layout, serialization, packages, SIMD,
tensor/quantized infrastructure, performance, safety, Aero as a whole, or Milestone 3
completion.

CAP-015 is accepted as a bounded representative application integration checkpoint.
It enriches the existing M1-001 telemetry application with one exact embedded
`[char; 10]` record, `T=<digit><digit>;H=<digit><digit>;`, whose guarded reads,
equality-only decimal classification, explicit `Result<int, char>`, and exhaustive
`Match` produce canonical value 42 and feed that value materially into the established
exact-output/exit-91 computation. The gate also retains boundary results 0 and 297,
all ten first-malformed-position identities, three first-error precedence controls,
and negative/equal-to-count trap-before-access specimens across public and pinned
Linux/Windows LLVM/Clang 22 `-O0`/`-O2` routes.
CAP-015 changes no compiler production or language-profile code.
CAP-014 remains Aero's exact-array profile origin and first Milestone 3 CPU slice;
CAP-018 remains its immutable-result checkpoint, and CAP-019 is Aero's latest accepted
compiler/profile capability. CAP-015 remains the latest separately classified project-
integration checkpoint and only enriches the existing M1-001 `END_TO_END` application
evidence. Both named profiles continue to reject the parser.
General-purpose text parsing, runtime Strings, serialization, runtime ingestion, file
input, and Unicode text encoding/normalization remain unsupported; accepted CORE-072's
bounded Unicode scalar `char` remains `PARTIAL`.

Exact candidate `dd9b1710abebf2f2318582cf94568c2f9a30ca8f`, protected PR #52
merge `b62696272f293f9f378f8a368cc818fcb8ef1074`, and shared tree
`27f359bc5ca90212a06ce73b71759cac0533c1f0` are immutable. Candidate push/PR CI
`31597830488`/`31598146528`, Rust CI `31598146473`, CodeQL `31598144554`, and
merge-head CI/Rust CI/CodeQL `31598634185`/`31598634090`/`31598633803` all pass.
This evidence does not add a CAP-015 parser, grammar, feature, profile, stability, or
conformance row and does not promote any neighboring component capability.

CAP-013 is an accepted cross-capability architecture and executable-composition
checkpoint. It promotes the prior specialization watch only because concrete drift
was observed: primitive aliases could be rejected across generic/trait boundaries or
produce duplicate private identities. One recursive canonical key, framing contract,
and deterministic struct -> enum -> function phase plan now serve semantic and raw
checked admission, while feature-specific validation and the single generic-function
body classifier remain separate. The representative telemetry program mixes
`Window<i32>`/`Window<int>` and trait `i32`/`int` signatures at exact output/exit 91.
Focused, corruption, complete-root, pinned LLVM 22 O0/O2, exact candidate, protected
PR #48 merge `856fc1e5f310b2b458f97d7b6aebb1ecf5c28572`, and exact merge-head gates
pass. This does not complete generics or traits.

Before CAP-024, the post-CAP-023 ranking controlled task selection. Historically, the
accepted-head inference correctness/reproducibility/artifact-footprint evidence gate
with no performance claim ranked first. Historically, exact CPU plus recursive-CopyData
application-profile composition ranked second at readiness and red-probe scope only,
and a small quantized numerical kernel ranked third at readiness and red-probe scope
only. The exact historical scored order and full before/after/stop/change-mind contracts
appear below. CAP-016 and CAP-017 remain
completed stops, and CAP-022 remains a mandatory `NO IMPLEMENTATION` stop: no import,
propagation, or runtime-ingestion semantics may be invented to revive those paths.

CAP-012 is an accepted Milestone 2 capability. It closes the remaining selected exit
half by letting the representative telemetry program call ordinary
helpers over nested CopyData state through immediate, conservatively root-scoped
immutable and mutable loans. The implementation shares projected-place classification with
assignment and adds independently verified checked-IR loan identity/lifecycle plus
Linux/Windows runtime-bounds fixtures. Local complete gates, exact candidate
`79d14866061184bc619ce5c92603c0964a31e74d`, all public candidate results,
protected PR #46 merge `49bcdfc3b23d2e1cc22fa3f0f36446fcffbf6e92`,
and exact merge-head CI/Rust CI/CodeQL pass, including pinned LLVM/Clang 22 native
execution. Together with CAP-011, the selected Milestone 2 exit gate is met. This
does not complete all Milestone 2 ambitions: stored/escaping references, general alias
reasoning/lifetimes, generic reference-call expansion, and general ownership remain
out of scope.

CAP-011 is an accepted Milestone 2 capability selected by the post-CAP-010 gap
ranking. It turns earlier generic-definition, whole-value specialization, fixed-array,
bounds-check, and projected-mutation primitives into one reusable fixed-capacity
`Window<T>` data structure with generic checked reads and functional updates. The
representative application now uses it for both integers and characters. Local
focused, representative, identity, complete-root, check, docs, formatting, and diff
gates pass. Exact candidate `dea5714e`, all nine public results, protected PR #44
merge `34b81eee`, and exact merge-head CI/Rust CI/CodeQL pass, including pinned
Windows LLVM 22 native execution. This does not complete collections or general
generics.

Accepted CAP-013 resolves the prior specialization architectural watch at the proven
shared seam: canonical recursive identity, equivalence, private framing, and phase
order. It deliberately does not merge policy-specific template validation,
substitution restrictions, rewriting, diagnostics, or body classification. Reopen
architecture work only on new cross-feature drift or a real multi-capability blocker,
not file size or neighboring permutations.

CAP-010 is an accepted Milestone 2 capability and the highest-payoff result of the
post-CAP-009 milestone-gap audit: required-only
nongeneric traits can supply immutable-receiver behavior to whole-value recursive
CopyData generic functions through deterministic static specialization. The
representative telemetry program uses one trait across both `Sensor` and `Batch`, so
the change advances real generic algorithm composition instead of another neighboring
topology. Focused, corruption, representative, compatibility, full-root, formatting,
Clippy, exact-candidate, pinned Linux/Windows native, protected PR #42, and exact
merge-head gates pass at accepted master `f77f1a227032008ab3ceadf2e2e3dcaed3b225e9`.

This slice did not complete traits or generics. Collections, general generic
operations, associated/default items, dynamic dispatch, broader patterns and error
propagation, lifetimes/drop/unsafe, and public ABI/destruction remain open. CAP-011
and CAP-012 later supplied the selected generic-container and ownership-intensive-
program exit product.

CAP-009 is an accepted Milestone 0 capability selected by the required
post-CAP-008 audit and three-gap ranking. Before it, programs could execute inside the
experimental compiler but could not request enforcement of a frozen cross-platform
source and representation contract. After it, public library and CLI routes can select
`stable-scalar-v0`, fail closed on every AST shape outside its one-file acyclic
nongeneric `int`/`bool` class, and execute representative exit-91 and wrapping exit-93
programs through an exact LLVM `i32` lane at `-O0` and `-O2`. One post-parse classifier
owns source admission; a private checked-program profile identity selects physical
representation without duplicating feature guards; the CLI profile is CPU-only.
Focused 10/10, the complete repository gate, all nine exact-candidate checks, protected
PR #40 integration, and exact merge-head CI/Rust CI/CodeQL pass. Accepted public master
is `1ef21c564ec564379e611002b1b321d910a991a3`, and the Milestone 0 selected-
stable-subset exit is met. CAP-009 does not stabilize Aero as a whole or add any of its
explicitly excluded aggregate, module, generic, ownership, runtime, ABI, accelerator,
benchmark, or release behavior.

CAP-008 is an accepted Milestone 2 capability selected by the required
post-CAP-007 roadmap audit and three-gap ranking. Its real-program delta is concise,
exhaustive fallback handling: ordinary Aero code may write `Err(_)` for an ignored
payload and one final `_ => fallback` for otherwise-uncovered variants across the
already-admitted concrete enum classes. The one shared enum-arm resolver remains the
semantic and checked-admission authority, and the independent verifier's one-target-
per-variant contract remains intact. Focused 4/4, complete root/verifier, representative,
and pinned Windows LLVM/Clang 22.1.8 external/machine/native/public-run gates pass at
unchanged exact output/exit 91. Exact candidate `9ebd204`, protected PR #38 merge
`a1716f8`, shared tree `c3dab0e`, all candidate-head checks, and exact merge-head CI
`31525340621`, Rust CI `31525340810`, and master-push/CodeQL `31525340605` pass.
It does not supply guards, nested destructuring, general error propagation, collections,
imports, ownership/drop expansion, stable ABI, or safety.

The prior accepted compiler-capability checkpoint was protected CAP-007 merge
`5a64acaffa5e7f7167823861a45bc49c6bb670b4`. Exact candidate `bfb7adb`,
candidate-head checks, protected PR #35, merge-head CI/Rust CI/CodeQL, the full root
gate, and pinned LLVM/Clang 22 stable Linux/Windows representative execution at
`-O0`/`-O2` pass. Accepted CAP-007 closes the canonical checked-program entrypoint
mechanism without adding source semantics. Accepted CAP-006 adds explicit user-defined bound-free generic
enums at exact recursive finite CopyData applications through contextual construction,
owned transport/replacement, and exhaustive `Match`. Accepted CAP-005 adds bounded
compile-time specialization of
bound-free whole-value generic transport functions over recursive finite CopyData.
Accepted CAP-004 adds bounded explicit user-defined generic
recursive-CopyData structs with deterministic compile-time substitution, checked
identity/schema verification, and composition through existing functions, aggregates,
mutation, and references. Accepted CAP-003 separately supplies bounded explicitly typed
concrete recursive-CopyData `Option<T>` and `Result<T, E>` construction, owned
transport, replacement, and exhaustive bound `Match`. Context-free inference, general
generic substitution/error propagation, trait-bounded or operational generic
functions, bounded/named-field/general generic enums and traits, carrier
aggregate/reference storage, collections, lifetimes/drop, stable layout/ABI, memory
safety, accelerators, and release claims remain excluded. Accepted CORE-082
remains a bounded Milestone 1 primitive-constant slice; accepted CORE-083 through
CORE-090 are useful but partial Milestone 2 reference, ownership, and aggregate-
composition fragments.

Accepted CAP-007 supplies the ranked Milestone 0 canonical
checked-program entrypoint mechanism. Public artifact-free `check_program` and
`check_file` APIs, library compilation, and CLI check/build/run/profile/source-test
validation share one library-owned lex/parse/direct-module/semantic/checked-IR/
internal-verification authority. Focused 3/3 and the complete normalized
235-library/32-binary/84-integration/doc/format/Clippy gate pass. Cached official
Windows LLVM/Clang 22.1.8 also externally verifies,
machine-verifies, and executes the representative product at `-O0`/`-O2` with exact
output/exit 91. Exact candidate `bfb7adb`, protected PR #35 merge `5a64aca`, all
candidate-head checks, and exact merge-head CI/Rust CI/CodeQL pass. CAP-007 does not
make any language feature `STABLE`; stable-subset classification remains open.

The required post-CAP-004 milestone-gap audit and three-gap ranking are complete in
the CAP-005 authorization. Bound-free recursive-CopyData generic transport functions
and the Milestone 0 canonical diagnostic/artifact contract tied at 24; an owned
dynamic collection foundation scored 22. CAP-005 won the tie because one reusable
type-independent helper creates a larger immediate real-program delta without
inventing trait or collection semantics. Its accepted implementation uses one
shared compile-time specialization authority that handles the bounded whole-value
transport class, the representative program uses `choose<T>` at `Reading<int>` and
`Reading<char>`, the complete 232-library/32-binary repository gate passes, and pinned
LLVM/Clang 22.1.8 verifies and executes exact `-O0`/`-O2` output/exit 91. Exact
candidate `68e2cd8`, protected PR #31 merge `59f7e47b`, and exact merge-head CI
`31504122753`, Rust CI `31504122730`, and CodeQL `31504122424` pass.

CAP-004 was selected by the required fresh post-CAP-003 milestone-gap and three-gap
ranking. Its accepted real-program delta is reusable data definition: one
`struct Reading<T>` now supplies independently typed `Reading<int>` and
`Reading<char>` values in the representative application instead of duplicated
concrete records. One shared pre-semantic/pre-admission authority performs exact
substitution; independent verification binds each private identity to its logical
schema. Local focused, corruption, compatibility, representative, complete root,
formatting, and correctness-denying Clippy evidence passes, as do the exact public
candidate, protected merge, merge-head, and pinned LLVM 22 gates. This closes one
bounded generic-data-definition requirement, not general generics or collections.

CAP-003 was selected by the required fresh milestone-gap and three-gap ranking. Its
accepted real-program delta is explicit absence/recoverable-failure
transport: bounded Aero functions can return, move, replace, pass, and exhaustively
inspect concrete recursive-CopyData `Option<T>` and `Result<T, E>` values rather than
sentinel scalars. This is a closed monomorphic built-in-family slice, not general
generic substitution or a complete error model. Local focused, shared-normalizer,
adjacent ownership, representative-program, complete root, formatting, and
correctness-denying Clippy evidence passes, as do the exact public candidate,
protected-merge, merge-head, and pinned LLVM 22 gates.

Current integration work has accepted `CORE-063` publicly: unary owned enums carry the
accepted recursive CopyData grammar through construction, exhaustive
identifier-bound Match, internal transport, checked IR, independent verification,
private typed LLVM, direct modules, and the pinned LLVM/Clang 22 native-exit-113 gate.
Accepted CORE-061 keeps closures parsed-only/fail-closed and
accepted CORE-062 supplies the recursive CopyData classifier. Projected borrows/writes,
deeper CFG ownership, stable ABI, full module semantics, named-field, bounded, or
general generic enums, and
real GPU execution remain unresolved.

Accepted CAP-006 is the selected post-CAP-005 capability. It adds exact explicit
user-defined recursive-CopyData generic enum
specialization and composes `Sample<Reading<int>>` with `Sample<char>` in the growing
representative application at unchanged output/exit 91. The complete local repository
gate, pinned LLVM/Clang 22 Linux/Windows native evidence, protected PR #33, and exact
merge-head workflows are green. Bounds/traits,
general substitution/operations, named variants, collections, lifetimes/drop, and
public ABI remain open.

Accepted `CORE-064` adds exact direct whole-owner replacement
for that admitted enum class through a shared owned-place classifier, generalized
checked identities, independent verifier controls, and private typed LLVM loads/stores.
All eight public checks and the pinned LLVM/Clang 22 native-exit-131 gate pass; enum
borrowing/projection/storage, new CFG ownership, drop/lifetimes, and stable ABI remain
unsupported.
Accepted `CORE-065` adds exact acyclic conditional joins for those enum owners:
mutually exclusive arms begin from one ownership snapshot, definitely returning arms
do not join, uncertain fallthrough becomes `MaybeMoved`, and checked-IR CFG dataflow
rejects double consumption after partial merges or across cycles. Loop fixed points,
break/continue transport, and general CFG ownership remain unsupported. The
182-library/188-binary/root gates, all eight public checks, and pinned LLVM/Clang 22
native-exit-137 lane pass.
Accepted `CORE-066` adds fresh per-iteration enum owners across
checked `while`, fixed-array `for`, and `loop`. Red-first evidence closed the admitted
`for` continue nontermination defect by routing continue through an explicit shared
increment block. The exhaustive/verifier/compatibility/root gates, all eight public
checks, and pinned LLVM/Clang 22.1.8 external/machine/object/link/native exit 149 pass.
Outer-owner backedge/exit joins, moved-target reinitialization, loop labels/expressions,
and general CFG ownership remain unsupported.
Accepted public `CORE-067` closes the remaining intrinsic-method false-
success tables behind one shared semantic/admission/lowering classifier and admits
exact recursive CopyData fixed-array `.len()`/`.is_empty()` constants. Static String
queries and Array/Vec `.iter()` compatibility remain bounded; runtime Strings,
collections, generic dispatch, iterator ABI, heap behavior, and closures do not move.
Focused, classifier, compatibility, and 183-library/root gates pass. All eight public
checks pass; pinned stable LLVM/Clang 22.1.8 externally and machine-verifies, object-
lowers, links, and executes exit 167, while nightly repeats exit 167.
Accepted public `CORE-068` closes the corresponding ordinary-function-call false-
success boundary behind one exact classifier consumed by both semantic paths and
checked admission/lowering. Missing or unsupported contracts cannot become `Int` or
an LLVM call. The 185-library/root gates and the composed local Clang 19.1.5 native-
exit-181 program pass. All eight public checks pass; pinned stable LLVM/Clang 22.1.8
rejects the invalid control, externally and machine-verifies, object-lowers, links,
and executes exit 181, while nightly repeats verification and exit 181. Generic/trait/
closure calls, overloads/conversions, reference results, new ownership/lifetime
behavior, stable ABI, and runtime collection semantics do not move.
Accepted public `CORE-069` admits exact positional variants with two or more fields
when every field is in the already accepted recursive finite CopyData class. One
schema authority covers construction, ordered bound Match, internal transport,
whole-owner reassignment/control flow, checked IR, verification, private LLVM, and the
pinned stable/nightly LLVM/Clang 22.1.8 native-exit-193 system gate. Named-field/
generic variants, broader patterns, enum storage/borrowing/projection, partial moves,
stable ABI, and accelerators remain excluded.
Accepted public `ARCH-002` then normalizes binding annotations to a leaf plus ordered
wrapper path and routes semantic analysis and checked admission through one supported/
explicitly-rejected/preserved policy. Depth-four characterization, byte-identical LLVM
evidence, all eight public checks, and the unchanged pinned native exit-193 lane
protect the boundary; no language feature, matrix cell, or runtime behavior moves.
Accepted public `CORE-070` adds file-aware library compilation through that same
checked library frontend and the existing root-level direct-module collector. Its
module-free parity, direct-module success/failure matrix, and complete local gates are
green. It is only a partial compiler-service convergence step: imports, namespaces,
recursive modules, external verification, cache behavior, and thin-CLI convergence
remain open. Exact implementation `365c28a3e4fdd306ec4c1a4837545ddbe3dac6a3`
passes all eight public checks and the unchanged pinned native exit-193 lane.
Accepted public `CORE-071` preserves parsed Rust-like direct/aliased/glob `use`
declarations and exact locations but rejects them consistently in semantics and
independent checked admission. Its exact implementation passes all eight public checks
and the unchanged pinned native exit-193 lane; no namespace, visibility, resolver,
backend, or runtime semantics move.
Accepted public `CORE-080` additionally preserves the founding direct/aliased dotted
`import` syntax, exact location, and distinct AST identity while routing it through the
same fail-closed phase boundary. Focused 13/13, compatibility, complete all-features,
static, documentation, and root gates pass. No positive name-resolution or import
semantics move; all nine exact-head checks and pinned exits 149/223/227/229 pass.
Accepted public `CORE-081` removes the exact 35-module compiler overlap between
binary and library. Compiler phases and direct-module collection/cache material are
library-owned, while the binary retains CLI-specific modules. Architecture, unit,
integration, all-features, static, documentation, and exact root gates pass; immutable
public evidence passes all nine exact-head checks.
Accepted `CHECKPOINT-001` and corrected solo-maintainer `CHECKPOINT-003` close the
283-commit/226-file handoff before another language slice is stacked. PR #4 merged exact
frozen head `9b13feb2` as merge commit `bf5f8a96`; its tree equals accepted tree
`6d5825a1`, the integration branch remains, strict app-bound protection remains, and
exact-SHA post-merge CI/Rust CI/CodeQL pass. Successor work starts from verified master
in one bounded positive vertical slice per PR. No release, safety, stability,
performance, or production-readiness claim follows.
Accepted public `CORE-072` then adds exact Unicode `char` as a distinct CopyData leaf
under one shared primitive authority. Raw/escaped literals, equality/inequality,
bindings/replacement, references, calls/results, arrays, tuples, structs, owned enums
and Match, control flow, direct modules, libraries, public CLI paths, checked IR,
independent verification, and private LLVM compose in one two-file native exit-197
system specimen. The 9/9 focused target, 190-library/196-binary complete surface, exact
root gate, and local official LLVM/Clang 22.1.8 system lane pass. Character arithmetic,
ordering, casts, strings/printing, literal-pattern execution, generic behavior, stable
ABI/FFI, and accelerators remain excluded. All eight exact-head public checks and the
stable/nightly pinned LLVM/Clang 22.1.8 exit-197 lanes pass.
Accepted public `CORE-073` adds the next hard ownership slice: exact acyclic whole-owner
reinitialization for already admitted destructor-free enums. One shared transition
classifier permits `Moved`/`MaybeMoved` to become exactly `Owned`; the verifier
independently proves predecessor consumption, schema/value identity, dominance, and
the checked write kill. The exhaustive source-to-native surface and local official
LLVM/Clang 22.1.8 exit-199 gate pass. Every loop-contained reinitialization, partial
move/projection, borrow/storage expansion, drop/lifetime behavior, and general CFG
fixed point remains excluded pending separate semantics and evidence. All eight
exact-head public checks and pinned LLVM/Clang 22.1.8 exit-199 lanes pass.
Accepted public `CORE-074` then adds a hard ADT/control-flow/ownership slice: an
exhaustive Match may yield one fresh owned enum when all arms have the same admitted
schema and their origin is a constructor, exact non-consuming enum-returning call, or
recursively fresh nested Match. Exact checked result/dispatch identities and verifier
CFG proof prevent missing, bypassed, repeated, post-merge, or wrong-schema fabrication.
The composed direct-module/check/build gates, all-eight public set, and pinned native
exit 203 pass.
Accepted public `CORE-075` adds exact initialized direct local/owned-parameter result
origins and a shared dynamic-path ownership join. Same-owner mutually exclusive arms,
different owners, fresh/direct mixtures, and recursively admitted leaves compose;
same-path duplicates, loop effects, additional owned call consumption, and external
nested scrutinees reject. It reuses the existing checked result place, enum-value/place-
load provenance, verifier CFG ownership proof, and private enum layout. Aggregate
results/storage, broader patterns, borrowing/projection, partial moves, drop/lifetimes,
stable ABI, and generic/closure semantics remain separate work. Exact implementation
`50a3e03d0bdbc0e7deddde747bc19df0621c1257`, all eight exact-head checks, and the
pinned stable/nightly LLVM/Clang 22.1.8 native exit 211 lanes pass.
Accepted public `CORE-076` unifies exhaustive Match results over the complete already
admitted value universe: one shared classifier accepts one identical recursive finite
CopyData type or the existing constrained owned-enum class, one generic checked result
place carries every arm through exact typed whole-place assignment, and independent CFG
verification proves all-path initialization and one merged load. Arrays (including
zero-length), tuples, finite acyclic structs, primitives, and owned enums retain their
existing private LLVM types. Exact implementation
`aefeb2d81fb5374e7373a4819f3c92f83a95eb35`, all eight exact-head checks, and both
pinned stable/nightly LLVM/Clang 22.1.8 native exit-223 lanes pass while preserving the
older exit-149 specimen. Strings, references,
unit/unary tuples, dynamic collections, cyclic/unsupported structs, enum aggregate
storage, wider patterns, stable ABI, runtime, drop/lifetimes, and general ownership
remain separately frozen.
Accepted public `CORE-077` admits exact balanced loop-carried reinitialization for a
direct mutable admitted destructor-free enum. `while`, fixed-array `for`, and `loop`
share one rule: entry, condition/iterable, every reachable fallthrough or `continue`
backedge, and every `break` exit must be exactly `Owned`; return paths do not join and
nested transfers attach to the nearest loop. Semantic analysis and independent checked
admission provide snapshots to one phase-neutral classifier, while verifier CFG controls
reject missing, bypassed, one-path, generic-store, wrong-schema, cycle, and exit repairs.
Exact implementation `a93d8d38c5f2a2499ce036f659c13cb2ec4fefcb`, all eight
exact-head checks, and pinned stable/nightly LLVM/Clang 22.1.8 native exit 227 pass
while preserving exits 149/223. Partial moves, projections, enum storage/borrowing,
drop/lifetimes, stable ABI, imports, accelerators, release, safety, and general non-enum
loop dataflow remain separate.
Accepted public `CORE-078` adds no language behavior. Exact implementation
`70f59fd72e96246b2ebefdf1ae53a9b7f3280cfe` pins the official Windows x86_64
LLVM/Clang 22.1.8 full archive by release SHA-256 and proves the existing MSVC
target/layout, invalid-source/IR rejection, external/machine verification, COFF object
generation, Clang/MSVC linking, public `run`, manual execution, and exact exit 227.
All nine exact-head checks pass while Linux stable/nightly preserve exits 149/223/227.
No stable ABI, general Windows, packaging, accelerator, release, safety, or performance
claim follows.
Accepted public `CORE-079` then replaces equality-to-first-entry loop ownership with
one convergent direct-enum header/exit summary shared by semantic analysis and
independent checked admission. `while`, admitted fixed-array `for`, and `loop` recheck
from widened `Owned`/`Moved`/`MaybeMoved` headers; post-loop state conservatively joins
false/exhaustion and nearest-loop break exits, while the existing verifier independently
proves cyclic consumption and repair. Exact implementation
`5b1ec7340db72354542ab325a9f75cad398857c2` passes all nine exact-head checks;
stable/nightly Linux preserve exits 149/223/227 and execute exit 229, while pinned
Windows LLVM/Clang 22.1.8 preserves exit 227 and executes exit 229 through public and
independent native paths.
PR #4's integration program is now merged and verified. Bounded successor PRs and
structured evidence-manifest automation remain active scaling controls.

## Corrective checkpoint after CORE-090

The original milestone exits below remain controlling. M1-001 closes the bounded
representative-program and optimization-equivalence portion of Milestone 1, while
foundational Milestone 0 contracts and broader Milestone 1 feature invariants remain
partial. The previously accumulated Milestone 2 fragments remain bounded.

### Milestone gap audit

| Roadmap area | Met | Partial | Open |
|---|---|---|---|
| Milestone 0 | Checked public CLI failures are nonzero and artifact-clean; accepted CAP-007 supplies one canonical checked-program entrypoint and artifact contract across frozen trusted validation routes; accepted CAP-009 classifies and executes the selected `stable-scalar-v0` subset as `STABLE`. | The wider compiler, component feature rows, experimental default, ABI, ownership, modules, aggregates, and release surface remain unstabilized. | Milestone 0 exit is met; preserve the selected profile while ranking the next broad real-program capability. |
| Milestone 1 | Trusted build/run routes verify LLVM before object generation; accepted M1-001 supplies one maintained representative application, compile-fail corpus, Linux/Windows exact execution, and `-O0`/`-O2` equivalence classified `END_TO_END`. Accepted CAP-015 enriches that same application with exact embedded-character interpretation through guarded reads and typed `Result`, materially composed into the unchanged exit-91 oracle. | The bounded conformance subset is authoritative for its workflow, while component language rows and the wider checked-IR/CFG/ownership invariant surface remain `PARTIAL`; CAP-015 adds no parser or profile row. | No remaining M1-001 exit item; broader grammar, invariants, and ordinary-program breadth are later capability work and do not become `STABLE` through this gate. |
| Milestone 2 | Structs, fixed arrays, tuples, enums, `Match`, CopyData composition, bounded ownership, references, projected mutation, accepted CAP-003 concrete CopyData `Option`/`Result` carriers, accepted CAP-004 explicit generic CopyData structs, accepted CAP-005 bound-free whole-value generic transport functions, accepted CAP-006 explicit user-defined recursive-CopyData generic enums, accepted CAP-008 nonbinding enum wildcards, accepted CAP-010 required-only CopyData trait dispatch, accepted CAP-011 fixed-capacity generic container algorithms, accepted CAP-012 projected call loans, and accepted CAP-013 canonical bounded-specialization identity/phase authority have executable slices. CAP-011 and CAP-012 satisfy the selected generic-data-structure plus ownership-intensive-program exit product. | Layout and evaluation behavior are private and bounded; ownership is not general. CAP-003 is monomorphic built-in-family evidence; CAP-004 through CAP-006 supply exact generic data-definition, function-transport, and enum-specialization slices; CAP-008 adds only terminal whole-arm and payload-leaf wildcards; CAP-010 adds one verifier-bound static-dispatch class; CAP-011 and CAP-012 close only the selected exit gate; CAP-013 closes alias/identity/phase drift without adding general semantics. | The selected Milestone 2 exit gate is met. General collections and generic operations, broader traits, broader patterns/error propagation, general lifetimes/drop/unsafe, public ABI/destruction, and ordinary-program breadth remain higher-milestone capability work. |
| Milestone 3 | Accepted CAP-014 supplies the first bounded exact-`i32` fixed-array CPU reference kernel with guarded dynamic indexing and Linux/Windows native oracle evidence; accepted CAP-018 widens the same profile with immutable exact-array value/result composition; accepted CAP-019 adds initialized mutable owned locals, guarded projected writes, and returned flat-array values; accepted CAP-020 adds the zero-production flat-buffer 2x3-by-3 matvec product gate; accepted CAP-021 adds source-embedded flat-record decode and two-stage scoring; accepted CAP-023 adds the zero-production fixed-shape 3-input/2-hidden/2-output ReLU-and-argmax inference product/evidence checkpoint over that unchanged profile; accepted CAP-024 closes the immutable evidence gap for the exact accepted CAP-023 application. | The CPU lane remains one private named profile over flat integer arrays. CAP-023 adds an application convention and reproducibility evidence only, with no runtime ingestion, composed CopyData profile, general parsing or error propagation, collections, streaming, quantization/tensors, recursive arrays/matrices, resource measurement, performance evidence, accelerator execution, or larger-workload capability. CAP-022's runtime-acquisition stop remains mandatory, and CAP-015's embedded-literal evidence belongs only to M1-001 representative integration. | The milestone exit is not met. Exact CPU plus recursive-CopyData application-profile composition ranks first, an owned dynamic collection/streaming foundation ranks second, and the small quantized numerical kernel ranks third; all remain readiness/red-probe scope only. |

Accepted CAP-009 closes the selected-stable-subset portion of the Milestone 0 row with
protected exact-head and merge-head evidence. It does not promote neighboring
component rows or the wider language beyond their recorded classifications.

### ROADMAP-001 ranked gaps and M1-001 outcome

Scores are 1--5 with higher better; `Risk` and `Evidence` are delivery favorability,
so 5 means lower risk or lower evidence cost.

| Rank | Gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Risk | Evidence | Total |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | Representative scalar application plus frozen subset and optimization-equivalence gate (accepted as M1-001) | 4 | 5 | 5 | 5 | 4 | 3 | 26 |
| 2 | Canonical Milestone 0 diagnostic/artifact and trusted-entrypoint contract | 3 | 5 | 5 | 5 | 3 | 3 | 24 |
| 3 | Positive import/module name resolution after namespace and graph semantics are frozen | 5 | 3 | 5 | 4 | 2 | 2 | 21 |

`M1-001` is accepted. Its fixed-size telemetry-policy program composes direct modules,
functions, constants, control flow, structs, arrays/tuples, enums and `Match`,
mutation, references, and CORE-090 projected writes. Its red exposed a shared Windows
variadic-call false-success, closed in the backend by retaining typed LLVM `double`
arguments and spelling the explicit variadic `printf` call type rather than passing raw
`i64` bits. Public `check`, verified `build`, and `run`; independent LLVM and machine
verification; exact local Windows `-O0`/`-O2` stdout/stderr/exit 91; the three-case
compile-fail corpus; focused 3/3; and the full 218-library/32-binary root gate pass.
Exact candidate `e7a74e6` passed all nine checks, merged through protected PR #19 as
`d7d1c768`, and passed post-merge CI, Rust CI, and CodeQL.

Real-program delta: before `M1-001`, users cannot point to any application-shaped Aero
program covered by an authoritative end-to-end subset contract. Accepted M1-001 now
supplies that program and classifies only its bounded conformance workflow as
`END_TO_END`; individual language features remain `PARTIAL`. Unspecified semantics,
nonportable behavior, optimizer divergence, or evidence that a different task closes
later gaps more safely would still change future decisions. Before another
implementation, at least three remaining gaps must be re-ranked against this accepted
baseline rather than inheriting the old order.

### Post-M1 ranking and accepted CAP-001

The required post-M1 comparison is complete. Scores retain the same 1--5 convention;
`Risk` and `Evidence` reward more favorable delivery.

| Rank | Gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Risk | Evidence | Total |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | Verified runtime reads from fixed arrays (accepted `CAP-001`) | 5 | 4 | 5 | 5 | 3 | 4 | 26 |
| 2 | Canonical Milestone 0 diagnostic/artifact and trusted-entrypoint contract | 3 | 5 | 5 | 5 | 3 | 3 | 24 |
| 3 | Positive import/module name resolution after namespace and graph semantics are frozen | 5 | 3 | 5 | 4 | 2 | 2 | 21 |

Before CAP-001, ordinary variable indexing could compile to unchecked LLVM `inbounds`
address formation and an out-of-range program could falsely succeed. Accepted CAP-001
adds one backend-wide ordered bounds guard for every nonconstant read over the existing
recursive CopyData fixed-array class, enriches the representative telemetry program
with computed reads, and adds negative/equal-to-count runtime controls. Focused,
representative, root, LLVM/machine, exact candidate-head, protected-merge, and exact
merge-head Linux/Windows `-O0`/`-O2` gates pass. The source contract is only a runtime
bounds error; the private trap has no stable status, diagnostic, ABI, or recovery
promise. Dynamic writes, projected borrowing, collections, and general memory safety
remain open. After accepted-truth synchronization, the next task requires a fresh
three-gap ranking against this stronger baseline.

What would change the decision: evidence that runtime bounds errors must be recoverable,
that the private trap can be optimized past an access, that retained array counts are
not independently trustworthy, or that this class requires an unresolved ownership or
stable-ABI decision stops CAP-001. A neighboring receiver/index permutation does not.

### Post-CAP-001 ranking and accepted CAP-002

The CAP-001 accepted-truth synchronization is complete. A fresh comparison uses the
same 1--5 scoring convention; `Risk` and `Evidence` reward more favorable delivery.

| Rank | Gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Risk | Evidence | Total |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | Checked runtime-indexed fixed-array assignment (accepted `CAP-002`) | 5 | 4 | 5 | 5 | 3 | 4 | 26 |
| 2 | Canonical Milestone 0 diagnostic/artifact and trusted-entrypoint contract | 3 | 5 | 5 | 5 | 3 | 3 | 24 |
| 3 | Positive import/module name resolution after namespace and graph semantics are frozen | 5 | 3 | 5 | 4 | 2 | 2 | 21 |

Before CAP-002, accepted CAP-001 makes `values[index]` safe to read, but ordinary
programs still cannot write `values[index] = value` or update nested fixed state in a
loop. Accepted CAP-002 admits runtime `int` selectors throughout the existing mutable
owned projected CopyData assignment class, evaluates selectors once left-to-right
before the RHS, and guards every dynamic selector before later selectors, effects,
address formation, or memory access. The representative telemetry application now
fills its sensor table in a bounded loop. Local focused, representative, and complete
repository gates pass. Exact candidate `577e601`, all nine candidate-head checks,
protected PR #23 merge `62ccc6a`, and exact merge-head CI/Rust CI/CodeQL pass.
Reference-target writes, projected borrowing, collections, compound
assignment, non-CopyData places, stable trap/ABI semantics, releases, benchmarks, and
general memory safety remain excluded.

What would change the decision: evidence that target-before-RHS ordering conflicts with
accepted assignment semantics; that checked selector identity cannot remain exact
through independent verification; that alias/lifetime or stable-layout decisions are
required for the admitted direct-owner class; or that a higher-ranked task can unlock
more real programs with lower correctness risk. Another argument ordering or selector
permutation does not justify changing the architecture.

### Post-CAP-002 ranking and accepted CAP-003

The CAP-002 accepted-truth synchronization and corrective milestone audit selected a
broader ordinary-program capability rather than another reference or index topology.
The comparison used the same 1--5 scoring convention; `Risk` and `Evidence` reward
more favorable delivery.

| Rank | Gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Risk | Evidence | Total |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | Explicitly typed `Option`/`Result` construction, transport, and exhaustive `Match` (accepted `CAP-003`) | 5 | 5 | 5 | 5 | 2 | 3 | 25 |
| 2 | Canonical Milestone 0 diagnostic/artifact and trusted-entrypoint contract | 3 | 5 | 5 | 5 | 3 | 3 | 24 |
| 3 | Positive import/module name resolution after namespace and graph semantics are frozen | 5 | 3 | 5 | 4 | 2 | 2 | 21 |

Before CAP-003, Aero retained carrier syntax but functions could not truthfully return
absence or recoverable failure: semantics fabricated missing types and checked
admission rejected construction. Accepted CAP-003 removes those defaults and admits
all four constructors only under an exact concrete recursive-CopyData binding,
reassignment, nongeneric parameter, or nongeneric result context. The values reuse the
owned-enum pipeline for moves, replacement, internal transport, and exhaustive bound
`Match`. The representative telemetry application executes both `Ok` and `Err` paths
while retaining exact output and exit 91. Focused 4/4, normalizer/corruption 4/4,
representative 3/3, 224-library/32-binary root, all nine candidate checks, protected
PR #26, exact merge-head workflows, and pinned Linux/Windows LLVM 22.1.8 `-O0`/`-O2`
gates pass.

What would change the decision: evidence that private carrier normalization changes
observable source identity, that enum ownership is unsound for these values, or that a
general substitution engine is required would have stopped the slice. Accepted
CAP-003 does not settle context-free inference, general generics, question-mark
propagation, collections, String errors, aggregate/reference storage, public ABI, or
lifetimes/drop. The required fresh three-gap ranking was completed before CAP-004 and
selected the accepted explicit generic-struct slice; ranks 2 and 3 were not automatic
follow-ons. The next fresh ranking against accepted CAP-004 is recorded in the
CAP-005 authorization and selected the now-accepted generic-transport slice. The
fresh CAP-006 ranking against accepted CAP-005 selected explicit user-defined
recursive-CopyData generic enums (25) ahead of the canonical Milestone 0 contract
(24) and owned dynamic collections (22). CAP-006 is accepted through protected PR #33
and exact merge-head evidence; no neighboring generic topology inherits priority.

## Milestone 0 - Establish compiler truth (selected exit met; broader scope partial)

- Make invalid lexical and syntactic input fatal on every trusted path.
- Close false-success paths that invent values or silently drop unsupported
  expressions.
- Enforce bounded function, return, binding, and scope contracts before IR.
- Make compiler failures nonzero and prevent invalid-program artifacts.
- Inventory every language, tooling, backend, test, and benchmark claim by stage.

Exit gate: no unclassified critical false-success defect in the chosen stable
subset; one canonical diagnostic/artifact contract; full repository gate and
independent review for each accepted boundary.

Accepted CAP-007 closes the canonical entrypoint mechanism for the currently frozen
trusted validation routes and is full-root/public-system green. Accepted CAP-009
classifies and executes the selected `stable-scalar-v0` subset as `STABLE`, so the
selected exit gate is met. The wider compiler, component feature rows, experimental
default, ABI, ownership, modules, aggregates, and release surface remain partial.

## Milestone 1 - Trustworthy scalar CPU core

- Freeze an authoritative grammar and type subset.
- Introduce fallible typed IR with CFG and ownership/type invariants.
- Verify LLVM before object generation.
- Prove source -> semantic analysis -> IR -> object -> link -> execution on Linux
  and Windows for representative scalar programs.
- Add differential optimized/unoptimized runtime tests and real compile-fail
  corpora.

Exit gate: the selected scalar subset is `END_TO_END`, with exact output,
diagnostic, verifier, platform, and reproducibility evidence.

## Milestone 2 - Safe compositional language core

- Implement structs, enums, Match, tuples, and collections as typed aggregate
  vertical slices with defined layout, evaluation order, ABI, and destruction.
- Complete generic substitution, trait dispatch/bounds, and error types.
- Replace shallow ownership tracking with CFG-aware moves, borrows, lifetimes,
  and explicit unsafe boundaries.
- Preserve the 22 active strict Phase 5 syntax-retention tests and replace or recover the 16 quarantined semantic/lossy-shape tests only after their missing contracts are frozen.

Exit gate: at least one real ownership-intensive program and one generic data
structure pass conformance, LLVM verification, and runtime checks without ignored
tests standing in for required behavior.

Accepted CAP-011 closes the generic-data-structure half of this exit with one
fixed-capacity recursive-CopyData `Window<T>` API, exact-head conformance, verified
LLVM, and Linux/Windows native execution. Accepted CAP-012 closes the other half by
composing nested immutable/mutable CopyData loans into the representative telemetry
application through checked IR, verified LLVM, and pinned Linux/Windows native
execution. The selected exit gate is met; general ownership, generics, collections,
layout/ABI/destruction, and the broader milestone ambitions remain partial.

### Post-CAP-020 ranking

CAP-020 closes the maintained flat 2x3-by-3 matvec product gap without changing
compiler production or the selected profile. Its successful flat encoding satisfies
the prior recursive-array deferral trigger. CAP-016 and CAP-017 remain completed
architecture stops because the missing namespace/visibility and propagation-syntax
contracts are not founded; neither is an implementation successor. Source-embedded
fixed-shape tensor-record decode plus two-stage flat-buffer exact-`i32` CPU scoring
product gate ranks first. Scores are 1--5 with higher better; `Risk` and `Evidence` are
delivery favorability, so 5 means lower implementation risk or lower evidence cost.

| Rank | Capability gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Favorable risk | Favorable evidence cost | Total |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | Source-embedded fixed-shape tensor-record decode plus two-stage flat-buffer exact-`i32` CPU scoring product gate | 5 | 5 | 5 | 5 | 4 | 4 | 28 |
| 2 | Runtime byte/file acquisition readiness and red probe under one cross-platform bounded-owned-buffer contract | 5 | 5 | 5 | 4 | 1 | 1 | 21 |
| 3 | Recursive exact-`i32` array / 2D matrix readiness deferred pending one shared recursive-shape contract | 3 | 3 | 4 | 5 | 2 | 2 | 19 |

Before rank 1, accepted CAP-020 executes one directly initialized 2x3-by-3 flat matvec,
but no maintained Aero-native product validates and decodes a source-embedded
tensor-shaped record or composes that result into a second numerical stage. After rank
1, one fixed `[int; 17]` record with exact header `[2, 3, 1]` and flat input/weight/bias
payload must be validated, decoded through guarded reads and fully initialized
flat-array writes, consumed by the accepted 2x3 matvec and a second exact-Int affine
scoring stage, preserve and reread every source lane, and produce independent ordinary
and wrapping oracles plus exact public and native sentinel 91.

Stop and rerank rank 1 if the exact `[int; 17]` product needs any compiler production
change, new language or profile rule, partial or uninitialized array state, unchecked
indexing, new arithmetic or quantization semantics, stable layout or ABI, or duplicated
guard or type authority.

Evidence that the complete record-to-score program is not expressible solely through
accepted CAP-020 semantics, is only a restatement of the single matvec, or cannot define
independent record, header, source, result, and wrapping oracles changes rank 1; clean
zero-production execution makes runtime acquisition, not recursive syntax, the next
hard boundary.

Before rank 2, Aero computations consume only source-embedded fixed data and no trusted
source program acquires external bytes. After rank 2 readiness, a task-local
cross-platform probe and architecture map must locate the first failure and freeze path
and byte identity, capacity and initialized count, partial-read and EOF behavior, typed
error mapping, ownership and drop, runtime linkage, sandboxing and determinism, and
Linux and Windows behavior, either yielding one bounded implementation contract within
two compiler phases or an explicit mandatory stop without claiming I/O capability.

Stop rank 2 before implementation if any contract item remains unfrozen, if allocation,
drop, or runtime ABI must be invented, if platform behavior cannot be made equivalent
and observable, if a useful slice crosses more than two compiler phases, or if invalid
acquisition can reach trusted IR or backend generation without typed failure.

Evidence that a caller-provided bounded byte slice or source-embedded record feeds the
flagship boundary sooner without filesystem or runtime semantics would defer rank 2
implementation; an explicit runtime RFC plus a probe demonstrating one shared
cross-platform ownership and error authority within the phase limit would permit later
implementation ranking.

Before rank 3, CAP-020 proves the target 2D matvec through flat `[int; 6]` storage while
`exact-i32-array-v0` deliberately rejects nested arrays. After rank 3 readiness, only if
it is reopened, a task-local `[[int; 3]; 2]` red probe and topology map must freeze depth,
dimension-product bounds, value placements, nested mutation and alias rules, and
nested-versus-flat physical identity under one source and physical shape authority, or
record a mandatory stop without claiming recursive arrays.

Stop rank 3 before implementation while flat encoding serves the target workload, or
if any recursive-shape decision remains unfrozen, admission and lowering cannot share
one canonical shape, the slice exceeds two compiler phases, or it requires stable
aggregate layout or ABI, aliases, or rank-specific classifiers.

Evidence of a concrete workload that flat buffers materially obscure, together with an
explicit bounded shape decision and a probe proving one shared source and physical
authority within two phases, would restore recursive arrays to implementation ranking;
CAP-020's clean flat execution otherwise keeps them deferred.

The ranking favors a material record-to-kernel composition before crossing the runtime
boundary. Rank 1 is the only executable product authorization; ranks 2 and 3 remain
readiness/probe decisions, and the stopped module and propagation designs stay closed.

### Post-CAP-021 ranking

CAP-021 closes the maintained source-embedded record-to-score product gap without
changing compiler production or the selected profile. Runtime byte/file acquisition
readiness and red probe under one cross-platform bounded-owned-buffer contract ranks
first. Small quantized numerical-kernel readiness ranks second, and recursive
exact-`i32` array/2D matrix readiness remains deferred at rank 3. Scores are 1--5 with
higher better; `Risk` and `Evidence` are delivery favorability, so 5 means lower
implementation risk or lower evidence cost.

| Rank | Capability gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Favorable risk | Favorable evidence cost | Total |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | Runtime byte/file acquisition readiness and red probe under one cross-platform bounded-owned-buffer contract | 5 | 5 | 5 | 4 | 1 | 1 | 21 |
| 2 | Small quantized numerical-kernel readiness and red probe under one frozen cross-platform arithmetic-and-representation contract | 5 | 5 | 3 | 5 | 1 | 1 | 20 |
| 3 | Recursive exact-`i32` array / 2D matrix readiness deferred pending one shared recursive-shape contract | 3 | 3 | 4 | 5 | 2 | 2 | 19 |

Before rank 1, accepted CAP-021 validates and scores one source-embedded fixed
`[int; 17]` record, but no trusted Aero source program acquires external bytes. After
rank 1 readiness, a task-local cross-platform probe and architecture map must locate
the first failure and freeze path and byte identity, capacity and initialized count,
partial-read and EOF behavior, typed error mapping, ownership and drop, runtime
linkage, sandboxing and determinism, and Linux and Windows behavior, either yielding
one bounded implementation contract within two compiler phases or an explicit
mandatory stop without claiming I/O capability.

Stop rank 1 before implementation if any contract item remains unfrozen, if
allocation, drop, or runtime ABI must be invented, if platform behavior cannot be made
equivalent and observable, if a useful slice crosses more than two compiler phases, or
if invalid acquisition can reach trusted IR or backend generation without typed
failure.

Evidence that a caller-provided bounded byte slice can feed the accepted
record-to-score boundary without filesystem or runtime acquisition semantics would
narrow the readiness target or defer later rank 1 implementation; an explicit runtime
RFC plus a probe demonstrating one shared cross-platform ownership and error authority
within the phase limit would permit later implementation ranking.

Before rank 2, accepted CAP-021 executes exact wrapping `i32` matvec, bias, and affine
scoring, but Aero has no frozen quantized representation, conversion, or arithmetic
contract and no maintained quantized oracle. After rank 2 readiness, a task-local
source-embedded red probe and architecture map must locate the first failure and freeze
stored, accumulator, and result types and domains; scale and zero-point presence,
representation, and scope; rounding and tie behavior; saturation and overflow
behavior; conversion boundaries and operation order; calibration provenance;
malformed-state rejection; the reference oracle; and Linux and Windows equivalence,
either yielding one bounded implementation contract within two compiler phases or an
explicit mandatory stop without claiming quantization capability.

Stop rank 2 before implementation if any arithmetic or representation decision remains
unfrozen, if the slice requires implicit conversion, fallback typing, or a second
numerical authority, if it silently changes CAP-021 wrapping order or semantics, if
malformed quantization state can reach trusted IR or backend generation, if a useful
slice crosses more than two compiler phases, or if deterministic Linux and Windows
oracle parity cannot be proved.

Evidence that external-byte ownership and error semantics must be established before a
quantized oracle can be meaningful, or that an exact-`i32` kernel advances the next
workload without lossy representation, would defer rank 2 implementation; an explicit
quantization RFC plus a probe demonstrating one shared cross-platform representation,
arithmetic, and error authority within the phase limit would permit later
implementation ranking.

Before rank 3, accepted CAP-021 proves fixed-record decode and two-stage scoring through
flat `[int; 17]`, `[int; 6]`, `[int; 3]`, and `[int; 2]` storage while
`exact-i32-array-v0` deliberately rejects nested arrays. After rank 3 readiness, only
if it is reopened, a task-local `[[int; 3]; 2]` red probe and topology map must freeze
depth, dimension-product bounds, value placements, nested mutation and alias rules, and
nested-versus-flat physical identity under one source and physical shape authority, or
record a mandatory stop without claiming recursive arrays.

Stop rank 3 before implementation while flat encoding serves the target workload, or
if any recursive-shape decision remains unfrozen, admission and lowering cannot share
one canonical shape, the slice exceeds two compiler phases, or it requires stable
aggregate layout or ABI, aliases, or rank-specific classifiers.

Evidence of a concrete workload that flat buffers materially obscure, together with an
explicit bounded shape decision and a probe proving one shared source and physical
authority within two phases, would restore recursive arrays to implementation ranking;
CAP-021's clean flat record-to-score execution otherwise keeps them deferred.

All three successors remain readiness and task-local red-probe work; none authorizes
production implementation.

### Post-CAP-023 ranking

CAP-023 closes the maintained source-embedded fixed-shape ReLU-and-argmax inference
product gap without changing compiler production or the selected profile. The
accepted-head inference correctness/reproducibility/artifact-footprint evidence gate
ranks first with no performance claim. Exact CPU plus recursive-CopyData
application-profile composition readiness ranks second, and small quantized
numerical-kernel readiness ranks third. Scores are 1--5 with higher better; `Risk` and
`Evidence` are delivery favorability, so 5 means lower implementation risk or lower
evidence cost.

| Rank | Capability gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Favorable risk | Favorable evidence cost | Total |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | Accepted-head CAP-023 inference correctness/reproducibility/artifact-footprint evidence gate with no performance claim | 4 | 5 | 5 | 5 | 5 | 4 | 28 |
| 2 | Exact CPU + recursive-CopyData application-profile composition readiness and red probe only | 5 | 5 | 5 | 5 | 2 | 2 | 24 |
| 3 | Small quantized numerical-kernel readiness and red probe under one frozen cross-platform arithmetic-and-representation contract only | 5 | 5 | 3 | 5 | 1 | 1 | 20 |

Before rank 1, accepted CAP-023 proves one source-embedded fixed-shape
3-input/2-hidden/2-output exact-`i32` ReLU-and-argmax CPU inference product through
deterministic verified LLVM, `llvm-as`, machine verification, native `-O0`/`-O2`,
public execution, and independent ordinary, wrapping, activation-boundary, tie,
malformed-header, and source-preservation oracles, but `claim-verification/` contains
no accepted-head Aero-native inference correctness/reproducibility record and no
artifact-footprint manifest. After rank 1, one immutable accepted-head evidence bundle
must record the exact commit and clean-tree state, source/input/oracle hashes, pinned
Linux and Windows toolchains and commands, deterministic
LLVM/bitcode/assembly/executable hashes and byte sizes, exact exit/stdout/stderr
results, failures, limitations, and a complete third-party reproduction procedure
without timing, throughput, speedup, memory, energy, or performance claims.

Stop and rerank rank 1 if CAP-023 is not accepted at the exact protected merge head,
any recorded artifact cannot be regenerated byte-for-byte within its stated
platform/toolchain boundary, correctness depends on retained mutable local artifacts
rather than tracked inputs and commands, Linux and Windows results diverge, or the
gate would require compiler production, source/profile semantics, benchmark timing,
or a public performance claim.

Evidence that an existing immutable accepted-head bundle already supplies the same
source/oracle/toolchain/command/hash/size/result/failure/limitation contract, that
artifact bytes are nondeterministic for an unfrozen reason, or that footprint capture
cannot be separated from benchmark semantics changes rank 1; a clean zero-production
correctness/reproducibility bundle advances Milestone 3 evidence but does not meet its
performance or complete resource-usage exit.

Before rank 2, accepted CAP-023 executes a flat exact-`i32` application convention
inside `exact-i32-array-v0`, while accepted recursive finite CopyData structs, enums,
`Result`, `Match`, and ownership slices remain bounded `PARTIAL` experimental
capabilities that the selected CPU profile deliberately rejects. After rank 2
readiness, a task-local source probe and architecture map must identify the first
composition failure and freeze whether one new application profile can reuse the
exact-`i32` scalar/flat-array physical authority together with only already-accepted
recursive CopyData aggregate, typed-result, `Match`, and bounded ownership contracts;
define admitted types and operations, phase ownership, profile selection, physical
identity, rejection boundaries, verifier evidence, and Linux and Windows oracles; and
yield either one bounded later implementation contract within two compiler phases or
an explicit mandatory stop without widening either existing profile.

Stop rank 2 before implementation if composition requires changing
`stable-scalar-v0` or `exact-i32-array-v0`, importing broad experimental defaults,
inventing struct, enum, `Result`, layout, or ABI semantics, reconciling duplicate type,
physical, or specialization authorities, adding recursive or nested exact arrays,
crossing more than two compiler phases, or claiming general CopyData, ownership, error
handling, inference, or safety.

Evidence that the CAP-023 workload can materially exercise existing CopyData
aggregates and typed failure under one bounded profile without new semantics and with
one shared exact physical/verifier authority raises rank 2 toward implementation;
evidence that a flat record remains sufficient, that the application needs runtime
ingress first, or that composition requires broad layout or ownership contracts defers
it.

Before rank 3, accepted CAP-023 proves exact wrapping `i32` matvec, positive-only ReLU,
two biased logits, and signed strict-greater argmax, but Aero has no frozen quantized
stored, accumulator, or result representation; scale or zero-point contract;
conversion, rounding, tie, saturation, or overflow behavior; calibration provenance;
malformed-state rule; or maintained cross-platform quantized oracle. After rank 3
readiness, a task-local source-embedded probe and architecture map must locate the
first failure and freeze every such decision plus operation order and Linux/Windows
equivalence, yielding either one bounded later implementation contract within two
compiler phases or an explicit mandatory stop without claiming quantization
capability.

Stop rank 3 before implementation if any arithmetic or representation decision
remains unfrozen; if the slice requires implicit conversion, fallback typing,
unfounded division or rounding semantics, or a second numerical authority; if the
existing scalar-double helper is treated as source-language proof; if CAP-023 wrapping
order changes; if malformed quantization state can reach trusted IR or backend
generation; if the slice crosses more than two compiler phases; or if deterministic
Linux and Windows oracle parity cannot be proved.

Evidence that accepted-head artifact evidence and exact CPU plus CopyData application
composition must precede a meaningful quantized oracle, or that exact `i32` continues
to advance the next workload without lossy representation, keeps rank 3 at readiness
scope; only an explicit quantization RFC plus a probe demonstrating one shared
cross-platform representation, arithmetic, malformed-state, and oracle authority
within the phase limit raises it toward implementation.

### Post-CAP-024 ranking

CAP-024 closes the prior immutable evidence gap. Every successor remains readiness and
task-local red-probe work only; none authorizes implementation. Scores are 1--5 with
higher better; `Risk` and `Evidence` are delivery favorability, so 5 means lower
implementation risk or lower evidence cost.

| Rank | Capability gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Favorable risk | Favorable evidence cost | Total |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | Exact CPU + recursive-CopyData application-profile composition readiness and red probe only | 5 | 5 | 5 | 5 | 2 | 2 | 24 |
| 2 | Owned dynamic collection/streaming foundation readiness and red probe, including its shared allocation/ownership/drop architecture, only | 5 | 5 | 5 | 5 | 1 | 1 | 22 |
| 3 | Small quantized numerical-kernel readiness and red probe under one frozen cross-platform arithmetic-and-representation contract only | 5 | 5 | 3 | 5 | 1 | 1 | 20 |

Before rank 1, CAP-024 proves the exact accepted flat CAP-023 application and its
immutable correctness/reproducibility boundary, while recursive finite CopyData
structs, enums, typed `Result`, `Match`, and ownership slices remain separate bounded
`PARTIAL` experimental authorities rejected by `exact-i32-array-v0`. After rank 1
readiness, a task-local source probe and architecture map must identify the first
composition failure and freeze whether one new application profile can reuse
exact-`i32` scalar/flat-array physical authority together with only already-accepted
recursive CopyData, typed-result, `Match`, and bounded ownership contracts; define
admitted types and operations, phase ownership, profile selection, physical identity,
rejection boundaries, verifier evidence, and Linux and Windows oracles; and yield
either one bounded later implementation contract within two compiler phases or an
explicit mandatory stop without widening either existing profile.

Stop rank 1 before implementation if composition requires changing
`stable-scalar-v0` or `exact-i32-array-v0`, importing broad experimental defaults,
inventing struct, enum, `Result`, layout, ABI, ownership, or error semantics,
reconciling duplicate type, physical, or specialization authorities, adding recursive
or nested exact arrays, crossing more than two compiler phases, or claiming general
CopyData, inference, safety, or language completion.

Evidence that the CAP-023 workload can materially exercise existing CopyData
aggregates and typed failure under one bounded profile without new semantics and with
one shared exact physical/verifier authority raises rank 1 toward later implementation;
evidence that a flat record remains sufficient, that runtime ingress is prerequisite,
or that composition requires broad layout or ownership contracts defers it and changes
the decision.

Before rank 2, accepted CAP-011 provides one fixed-capacity recursive-CopyData
`Window<T>` algorithm and the representative program composes only statically bounded
storage; Aero has no accepted owned dynamic collection, allocation, capacity growth,
initialized-length, reallocation, alias, failure, or drop contract. Legacy `stdlib.rs`
String/Vec helpers and their rejected checked-IR/backend instructions are not
source-language authority. After rank 2 readiness, a task-local
owned-collection/streaming source probe and architecture map must first freeze the
public type/API name, then the minimal useful element class and operations;
length/capacity/growth and initialized-state rules; allocation, failure,
move/borrow/alias, reallocation, iteration/indexing, and drop behavior; one physical
and verifier authority; rejection boundaries; and deterministic Linux and Windows
oracles, yielding either one bounded later implementation contract within two compiler
phases or a mandatory stop without claiming dynamic collections.

Stop rank 2 before implementation if allocation, OOM/error, ownership, alias,
reallocation invalidation, lifetime, drop, runtime ABI, or element destruction
semantics remain unfrozen; if uninitialized elements can become observable; if legacy
unchecked helpers or verifier-rejected instructions would be activated; if the useful
slice crosses more than two compiler phases; or if invalid collection state can reach
trusted IR/backend or Linux and Windows behavior cannot be made equivalent and
observable.

Evidence that fixed-capacity `Window<T>` plus flat source records serves the next
useful workload, that runtime ingress is prerequisite, or that one owned collection
requires broad allocator/drop/lifetime architecture keeps rank 2 at readiness scope;
only an explicit collection RFC plus a probe demonstrating one shared cross-platform
initialized-state, ownership, physical, error, and verifier authority within the phase
limit raises it toward later implementation.

Before rank 3, CAP-024 preserves exact wrapping `i32` matvec, positive-only zero clamp,
two biased logits, and signed strict-greater argmax, but Aero has no frozen quantized
stored, accumulator, or result representation; scale or zero-point contract;
conversion, rounding, tie, saturation, or overflow behavior; calibration provenance;
malformed-state rule; or maintained cross-platform quantized oracle. After rank 3
readiness, a task-local source-embedded red probe and architecture map must locate the
first failure and freeze every such decision plus operation order and Linux/Windows
equivalence, yielding either one bounded later implementation contract within two
compiler phases or an explicit mandatory stop without claiming quantization
capability.

Stop rank 3 before implementation if any arithmetic or representation decision
remains unfrozen; if the slice requires implicit conversion, fallback typing,
unfounded division or rounding semantics, or a second numerical authority; if the
scalar-double helper is treated as source-language proof; if CAP-023 wrapping order
changes; if malformed quantization state can reach trusted IR or backend generation;
if the slice crosses more than two compiler phases; or if deterministic Linux and
Windows oracle parity cannot be proved.

Evidence that exact CPU plus CopyData application composition must precede a meaningful
quantized oracle, or that exact `i32` continues to advance the next workload without
lossy representation, keeps rank 3 at readiness scope; only an explicit quantization
RFC plus a probe demonstrating one shared cross-platform representation, arithmetic,
malformed-state, and oracle authority within the phase limit raises it toward later
implementation.

## Milestone 3 - Aero-native AI/ML infrastructure flagship

- Accepted CAP-014 provides the first bounded CPU computation toward this milestone:
  an exact wrapping `i32` fixed-array kernel with guarded dynamic indexing and a
  cross-platform native oracle. Accepted CAP-018 widens that same profile with
  immutable exact-array result composition, and accepted CAP-019 adds initialized
  mutable flat-array production, guarded writes, and by-value results. Accepted
  CAP-020 adds the zero-production flat 2x3-by-3 matvec product with exact ordinary,
  wrapping, guarded-access, public, and Linux/Windows native evidence over that
  unchanged profile. Its successful flat encoding satisfies the recursive-shape
  trigger, so recursive syntax is deferred. Accepted CAP-021 composes that matvec into
  a zero-production source-embedded flat `[int; 17]` record decode and two-stage
  exact-`i32` scoring product with header, ordinary, wrapping, malformed, source-
  preservation, public, and Linux/Windows native evidence over the unchanged profile.
  Accepted CAP-023 composes the maintained products into one source-embedded flat
  `[int; 20]` 3-input/2-hidden/2-output zero-clamp-and-argmax application with exact
  ordinary, wrapping, activation-boundary, tie, malformed-header, source-preservation,
  public, and Linux/Windows evidence over the same profile. Accepted CAP-024 closes the
  accepted-head correctness/reproducibility/artifact-footprint evidence gap for that
  exact product without a performance claim. Exact CPU plus recursive-CopyData
  application-profile composition, an owned dynamic collection/streaming foundation,
  and the small quantized numerical kernel remain readiness/red-probe work only.
  Together these still do not meet the milestone exit.
- Build a correct CPU reference workload for binary/tensor ingestion and a small
  quantized numerical kernel.
- Grow it into a streaming data or inference component that exercises Aero's
  types, ownership, aggregates, errors, collections, and parallel execution.
- Compare against equivalent established implementations using one correctness
  oracle and the measurement boundaries in `BENCHMARK_PROTOCOL.md`.
- Retain raw inputs, outputs, samples, hashes, toolchains, failures, and artifacts.

Exit gate: a third party can reproduce a useful Aero-native result and understand
its correctness, resource usage, performance, and limitations. External llama.cpp
or framework execution alone does not satisfy this milestone.

## Milestone 4 - Coherent concurrency and tooling

- Freeze an ownership-safe task/channel, structured-concurrency, or equivalent
  model and prove its runtime behavior.
- Converge CLI, library, module resolver, LSP, formatter, documentation generator,
  project tooling, package manager, registry, profiler, and conformance runner on
  one compiler service.
- Add dependency locking, sandbox/trust policy, source-span accuracy, and
  integration tests for failures as well as successful workflows.

Exit gate: tools agree on source meaning and status, concurrent programs have
defined safety/runtime behavior, and a fresh project builds reproducibly.

## Milestone 5 - Proven heterogeneous execution

- Define host/device boundaries, memory spaces, transfers, synchronization,
  streams/queues, kernel ABI, capability discovery, and fallback observability.
- Prove CPU reference equivalence separately for ROCm and CUDA.
- Capture Aero-generated target IR, object/link or loader artifacts, real device
  launch, transfers, synchronization, numerical results, and failure paths.
- Measure compile, transfer, kernel, and end-to-end costs separately.

Exit gate: each named backend independently meets every gate in
`BACKEND_STATUS.md`; a backend flag or transformed helper is insufficient.

## Milestone 6 - Self-host

- Specify the compiler bootstrap boundary and reproducible stage process.
- Implement enough of the compiler in Aero to compile itself in controlled stages.
- Compare stage outputs, diagnostics, runtime behavior, and build reproducibility.
- Keep the Rust bootstrap compiler available until the Aero implementation is
  independently trustworthy.

Exit gate: a documented clean bootstrap produces equivalent verified artifacts
on supported platforms.

## Milestone 7 - Stabilize

- Define the supported surface, compatibility policy, versioning, deprecation,
  migration, release, and governance processes.
- Make all stable examples executable tests and eliminate known invariant
  violations from the release surface.
- Run public RFC review for major semantic commitments.
- Publish an honest pre-1.0 or 1.0 release only when release gates pass.

Exit gate: every stable claim is traceable to specification and end-to-end
evidence, with no historical label substituting for proof.

## Milestone 8 - Optimize and grow the ecosystem

- Add legal, measured optimizations after semantic equivalence is established.
- Expand libraries, platforms, tooling, documentation, and the flagship workload.
- Use larger suites such as SPEC CPU, PARSEC, and domain benchmarks when Aero can
  compile their required workloads correctly.
- Revisit custom backends only where LLVM cannot meet a demonstrated need.

Exit gate: optimization and ecosystem claims remain reproducible, correctness-
gated, and tied to real user workloads.

See `FRAMEWORK_ALIGNMENT.md` for source traceability and the execution-quality
and killer-application rationale behind this sequence.
