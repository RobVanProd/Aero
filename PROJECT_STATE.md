# Aero Project State

Last updated: 2026-08-17 (America/New_York)

## Current objective

### H1 objective from accepted B1C: bootstrap convergence contract

The current accepted public master is CAP-047/B1C merge
`0365e5c91bd503b198855b97b7f16054488d6dff`, tree
`e13bcc92f04e0f1aec44eafcfdccbe638c1405ad`. Reviewed candidate
`18a507c8fabfc79e24167c79bef516b531506914` has the identical tree. Protected
PR #89 and all 13 candidate checks passed. Accepted-head CI `31975615300`, Rust
CI `31975615304` (stable, nightly, and Windows LLVM 22), CodeQL `31975614982`,
and evidence `31975615309` are terminal-success.

CAP-047/B1C preserves ledger-only
commit `4c88952` freezes output/driver semantics, red-first commit `8f9d472`
passed the independent stream and LLVM/toolchain oracles before failing only
because the bounded product/driver was absent, and red record `6246d87`
preserves that result. Digest-scope amendment `418b4ad` authorizes only the
cumulative whole-file sentinels necessarily changed by the new runtime,
checked instruction, and verifier authority.

The new `exact-i32-byte-io-v0` profile admits one direct explicit
`stdout_write_byte(int) -> Result<int, int>` operation. Semantic analysis and
independent checked-IR generation both enforce the exact source context; the
verifier checks its scalar operand/result and reserved symbol; the backend
lowers it only after verification to the conditional C runtime call. Raw byte
output is binary on Windows, flushes before success, and maps write/setup/range
failures to sticky typed errors without continuing output.

The tracked Aero product preserves B1B, independently rereads and authenticates
its 144-byte module, then emits each byte in order without adding a ByteBuffer
owner. Canonical output has attempted/status/code/index/length
`1/0/0/-1/144`, seal 506643, empty stderr, and exit 91. Every injected output
failure position exposes only the exact prior prefix, stops immediately, exits
non-success, and preserves exact 14/58/14 allocation cleanup.

The host command `bootstrap-drive-b1c` requires explicit emitter and LLVM
directories, an absent absolute output directory, and O0 or O2. It captures and
authenticates the entire stream before publishing, requires exact LLVM/Clang
22.1.8, uses direct argv with no shell or PATH fallback, verifies/assembles/
lowers/links a fixed observer, and removes only its newly created transaction
directory on failure. The focused target is 8/8 green in 315.37 seconds; its
root-gate replay is 8/8 green in 442.60 seconds. The complete accepted
B1B/B1A/M1B/M1A/F1B/F1A/D1 ring is green. Formatting, correctness Clippy,
`git diff --check`, and the complete D:-redirected root gate are green at 312
library tests, 36 binary tests, every integration/native/system target, and doc
tests. Candidate `18a507c` merged normally as `0365e5c` with identical tree
`e13bcc9`; candidate and accepted-head replay are complete and green.

B1C is still one bounded backend handoff, not a self-compiling compiler. The
Rust stage-0 compiler is required to build the Aero emitter; the grammar,
semantic universe, serialized checked IR, and LLVM mapping remain deliberately
small. H1 stage-0/stage-1/stage-2 convergence and H2 reproducible self-hosting
remain open. H1 is now the active ledger-first task: freeze the canonical Aero
compiler source bundle, stage interfaces, environment/toolchain manifest,
comparison contract, and failure rules before convergence behavior. Every
task-created worktree, Cargo target, temporary file, native harness, LLVM
artifact, and log remains on D:. See
[`BOOTSTRAP_DRIVER_READINESS.md`](BOOTSTRAP_DRIVER_READINESS.md),
[`AERO_FRONTEND_READINESS.md`](AERO_FRONTEND_READINESS.md), and
[`SELF_HOSTING_ROADMAP.md`](SELF_HOSTING_ROADMAP.md).

CAP-048 is the locally green documentation-only H1 contract candidate. It records the
accepted compiler's exact 241,941-byte self-source versus 8,192-byte input
boundary, freezes a new single-file canonical source path, defines the stage
process and comparison protocol, and orders H1A through final replay. See
[`BOOTSTRAP_CONVERGENCE_READINESS.md`](BOOTSTRAP_CONVERGENCE_READINESS.md).

CAP-049/H1A is the first H1 prerequisite to execute and is locally green. The new
canonical source
[`examples/aero_self_host_v0/compiler.aero`](examples/aero_self_host_v0/compiler.aero)
is 241,918 bytes, 5,563 LF bytes, 7-bit ASCII, SHA-256
`977a1f3e0562f2b6507873febcdf8fd3f59b2f3a1370327c500e0bdd7e6232ad`. It is a
copy-derived successor of accepted B1C differing only in six mechanically
reconstructed ways: three raised ingestion bounds (1,048,576 source bytes,
262,144 token records, 16,384 names), one new lexical token kind 37 for a lone
`&`, the matching token-record validator bound, and one quadratic-to-linear
rewrite of the located-token re-derivation. Fed its own exact bytes it consumes
all 241,918, interns 571 names, records 31,062 located token records, and then
stops at the independently predicted first unsupported parser construct —
`status = 10`, offset 16, line 1, column 17, expecting `)` and finding an
identifier, with zero nodes. All 67 independently derived expectation values
match at O0 and O2, no output byte is written, and allocation accounting
balances with zero live allocations. The accepted 34-byte canonical program is
preserved exactly: exit 91 and the identical 144-byte module, MD5
`fd2390d17d448d4539a72bf1991314dc`. The focused target
`self_host_source_ingestion_tests` is 8/8 green.

CAP-050a and CAP-050/H1B-1 are the next two H1 prerequisites to execute and are
locally green. CAP-050a added the bounded parameter store - one owner, its
counter, a 68th expectation value, and a validated `989` checksum region -
without any parser rule, so that a later grammar failure would be unambiguous.
CAP-050 then added the parameter sub-machine itself. Between the `(` and the
`->` the parser now accepts either an immediate `)` or a nonempty list
`IDENT : TYPE ( , IDENT : TYPE )*`, where `TYPE` is exactly the identifier `int`
or the exact sequence `Result < int , int >`. Each admitted parameter appends one
two-word record to the store; no syntax node is created for a parameter, because
the node arena is what the semantic, checked-IR, and verifier phases count. Any
other token, type identifier, trailing comma, missing colon, or unbalanced
generic form is an exact located rejection. The canonical source
[`examples/aero_self_host_v0/compiler.aero`](examples/aero_self_host_v0/compiler.aero)
is now 252,044 bytes, 5,792 LF bytes, 7-bit ASCII, SHA-256
`bc1c6418a86df54ea9e36c8df4f7b2c667a170b6f96510ebd06ba95a48e12d6d`, and remains a
copy-derived successor of accepted B1C asserted byte for byte: the six CAP-049
ingestion differences, seven CAP-050a store differences, and four CAP-050
sub-machine differences. Fed its own exact bytes it now admits the
`result : Result < int , int >` parameter list and the `int` return type, records
one parameter, reduces the body's leading `match` identifier to one
name-reference node, and stops at the independently predicted next construct -
`status = 10`, offset 68, line 2, column 18, expecting `;` and finding an
identifier, with one node and one parameter. All 68 independently derived
expectation values match at O0 and O2, no output byte is written, and allocation
accounting balances with zero live allocations. The accepted 34-byte canonical
program is preserved exactly: exit 91 and the identical 144-byte module, MD5
`fd2390d17d448d4539a72bf1991314dc`. Ten focused signature probes - three
positive shapes, six mandated negatives, and one body probe - are run against the
real linked product. The focused target `self_host_source_ingestion_tests` is
10/10 green.

CORE-093 is the code-generator fix CAP-049 uncovered and is locally green. The
generator emitted each value's storage slot inline, so every checked `ByteBuffer`
result temporary inside a loop became a non-entry `alloca` that LLVM never
reclaims — 423 of them inside loop bodies in the compiler's own module. A loop
over a `ByteBuffer` therefore grew the stack once per iteration, and self-input
terminated with `STATUS_STACK_OVERFLOW` before any diagnostic. Every static
`alloca` now moves to the entry block in original relative order; dynamic
allocas never move. The focused target `entry_block_alloca_tests` is 3/3 green:
the tracked `examples/loop_stack_stability/main.aero` specimen survives 800,000
checked `ByteBuffer` operations at O0 and O2, and all eight accepted `.aero`
products plus the specimen emit no `alloca` outside an entry block while still
passing required LLVM verification.

H1A is ingestion and tokenization only, and CAP-050/H1B-1 adds signature syntax
only. The compiler reads its own source and now admits its function signatures;
a parameter still means nothing to the type, ownership, checked-IR, verifier, or
backend authorities, and no body, expression, or item beyond the first function's
signature parses. The rest of H1B, H1C through H1E, and the final stage replay
remain open, and no self-hosting claim follows.

The checkpoint sections below are retained chronological records. Any
present-tense `current` or `latest` wording inside an older checkpoint is scoped
to that historical checkpoint and is superseded by this current-objective
section.

### CAP-024 accepted: immutable accepted-head inference evidence checkpoint

Exact CAP-024 reviewed candidate `617bfce86feb879ee5eef61b44cf4e2a5520f022`,
shared candidate/merge tree `9520f24e4f1626f16782a9775480f9653f6059bb`,
accepted base and first merge parent `918c9222eb61e2435e18847e30b946cd08013238`,
and protected PR #64 merge `2f7ec325e423461a8e867f4ee2573ae6dcf15dfd`
whose second parent is that candidate are immutable. Candidate push CI `31764763341`,
PR CI `31764765501`, Rust CI `31764765563`, CodeQL `31764763584`, and CAP-024
evidence run `31764765495`; candidate push/PR compiler jobs
`94658200345`/`94658206474`, stable/nightly/Windows LLVM 22 jobs
`94658207134`/`94658207170`/`94658207086`, CodeQL Actions/Python/Rust jobs
`94658203257`/`94658203263`/`94658203316`, aggregate candidate CodeQL check
`94658280067`, and Actions/Python/Rust analyses
`1617260890`/`1617261159`/`1617264144` all pass. Candidate CAP-024
Linux/Windows/aggregate jobs `94658206500`/`94658206555`/`94659098928` pass and
artifact `9205970753` carries fresh manifest
`bd5e609b4ce829579331a23170d6d9e4fc4d5906cb32779876a78bc24294812c` plus 132
fresh observations
`62780d81e9dcaa6e85c08d0805608a58283816dd062c3a8bb1a8c67971ac551f`; its
claim-bearing projection matches accepted canonical manifest
`4b4cfce95459761dddd588e09abb3046854e0c2afb361f08a9553f180f013a34`.
Merge-head CI `31765227712`, Rust CI `31765227675`, CodeQL `31765227317`, and
CAP-024 replay `31765227673`; exact merge compiler/stable/nightly/Windows LLVM 22
jobs `94659602474`/`94659602479`/`94659602493`/`94659602501`, CodeQL
Actions/Python/Rust jobs `94659604078`/`94659604103`/`94659604064`, default-branch
analyses `1617281747`/`1617282341`/`1617285598`, and CAP-024 aggregate replay job
`94659602932` all pass. The two default-branch capture jobs
`94659621233`/`94659603455` are correctly skipped because protected master validates
the tracked bundle rather than replacing accepted observations.

Current accepted public master and public evidence checkpoint is protected CAP-024 merge
`2f7ec325e423461a8e867f4ee2573ae6dcf15dfd`, tree
`9520f24e4f1626f16782a9775480f9653f6059bb`; its ordered parents are accepted base
`918c9222eb61e2435e18847e30b946cd08013238` then reviewed candidate
`617bfce86feb879ee5eef61b44cf4e2a5520f022`.

CAP-024 is the current accepted public evidence checkpoint and protected public master.
It adds no compiler production, parser, grammar, source semantics, profile, semantic
analysis, checked IR, verifier, backend, example, product oracle, runtime behavior, ABI,
capability classification, benchmark, resource-usage, performance, accelerator, safety,
or general-inference capability. Its only claim is immutable accepted-head CAP-023
correctness, within-platform target-artifact reproducibility, exact observable behavior,
and artifact byte-size footprint under the closed recorded boundary.

CAP-019 remains the latest compiler/profile widening; CAP-023 remains the latest product
checkpoint. The selected `exact-i32-array-v0` row and the existing CAP-023 CPU
backend-summary row remain byte-identical, and CAP-024 adds no language,
selected-profile, or backend-summary row.

The accepted catalog record remains
`aero_cap023_inference_correctness_918c9222_20260813`, status
`verified_correctness_reproducibility_only`, with exactly the tracked schema, canonical
88,734-byte manifest SHA-256
`4b4cfce95459761dddd588e09abb3046854e0c2afb361f08a9553f180f013a34`, oracle, and
reproduction contract.

The PR-only aggregate CodeQL check is correctly absent on the default branch.
Default-branch Actions analysis `1617281747` carries only the pre-existing open alert #4
created and last updated 2026-08-09; Python and Rust analyses contain zero results, and
no new CAP-024 alert exists.

The selected Milestone 0, Milestone 1, and Milestone 2 exits remain met for their bounded
selected products; broader ambitions remain partial. Milestone 3 remains open. CAP-024
closes the prior accepted-head correctness/reproducibility/artifact-footprint gap, but
supplies no runtime ingestion, composed CopyData application profile, quantization,
runtime-resource measurement, performance evidence, accelerator execution, or broader
workload.

Block-local `mod missing;` remains a demonstrated invalid-program false success because
the common statement parser accepts it, `ModDecl` has no source location, and semantic
plus checked admission silently discard it. CAP-016 already audited that exact defect
and found that trustworthy placement/provenance rejection participates in the unfrozen
module migration across more than two compiler phases. No new module RFC or
decision-changing evidence exists, so CAP-016 remains a mandatory `NO IMPLEMENTATION`
stop until its explicit re-entry condition is met.

CAP-022 remains a mandatory `NO IMPLEMENTATION` runtime-acquisition stop.

### CAP-023 accepted: source-embedded fixed-shape exact-i32 inference product

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
`[1, -3, 0, 0, 0, 5, 4, 0]`, and `[1, 1, 2, 1, 2, 3, 3, 0]`; malformed results
are eight zeros; public and native success is sentinel 91 with empty source
stdout/stderr.

Exact CAP-023 reviewed candidate `63e6b00b6294de61e3afd292a1e32e2b014714e2`,
shared candidate/merge tree `4d234cdfde67f1083773e2c41be4ab92027769db`,
accepted base and first merge parent `4bce540dfed6dfffa152067f4e00424501a6cdd8`,
and protected PR #62 merge `e9b281504446465cfc8fcbe17c65cce92df0e83a`
whose second parent is that candidate are immutable. Candidate push CI `31687464571`,
PR CI `31687585904`, Rust CI `31687585893`, CodeQL `31687584263`, and aggregate
candidate check `94407323731`; candidate push/PR compiler jobs
`94406770929`/`94407177877`, stable/nightly/Windows LLVM 22 jobs
`94407178006`/`94407178047`/`94407178042`, CodeQL Actions/Python/Rust jobs
`94407175858`/`94407175752`/`94407175820`, and Actions/Python/Rust analyses
`1612686978`/`1612687391`/`1612693654`; merge-head CI/Rust CI/CodeQL
`31688093145`/`31688093150`/`31688092749`, exact merge
compiler/stable/nightly/Windows LLVM 22 jobs
`94408808914`/`94408809340`/`94408809458`/`94408809296`, merge CodeQL
Actions/Python/Rust jobs `94408812427`/`94408812194`/`94408812175`, and
default-branch Actions/Python/Rust analyses
`1612715455`/`1612715345`/`1612721829` all pass.

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

### CAP-021 accepted: source-embedded two-stage exact-i32 scoring product

Accepted CAP-021 changes no parser, grammar, source semantics, language profile,
semantic analysis, checked IR, verifier, backend, ABI, or capability classification;
it is a zero-production product/evidence checkpoint over CAP-019's
`exact-i32-array-v0` surface and composes the accepted CAP-020 flat matvec.

The accepted application treats one source-embedded flat `[int; 17]` as an application
record with exact header `[2, 3, 1]`, dynamically decodes input, row-major first-stage
weights, first-stage bias, second-stage weights, and score bias into fully initialized
flat locals, then composes the accepted 2x3 matvec with wrapping bias and affine
scoring.

The accepted scorer returns `[valid, raw0, raw1, hidden0, hidden1, score]`; its ordinary
result is `[1, 122, 167, 135, 181, 4938]`, its wrapping result is
`[1, -24, 18, 2147483623, -2147483631, -2147483627]`, an invalid header returns six
zeros, both valid source records preserve and reread all 17 lanes, and the application
exits `91`.

Every dynamic read and write uses the existing signed bounds, trap-before-address,
`sext`, typed-GEP, and same-pointer consumer authority; exact public and pinned
Linux/Windows LLVM 22 verifier, O0/O2, native, and deterministic-emission evidence
passes.

Exact CAP-021 reviewed candidate `f91df56084540d30f3c8d09e71c5f30db280fd93`,
shared candidate/merge tree `7e34b4b8e817a7aafaaabc6326fa0a4d616fcc91`,
accepted base and first merge parent `df0626916d190d8a7580f783e3ac24a89f691617`,
and protected PR #60 merge `59af445ea02c1759d337d698be9c4f4472587aaf`
whose second parent is that candidate are immutable. Candidate push CI `31670574143`,
PR CI `31670599830`, Rust CI `31670599826`, CodeQL `31670598033`, and aggregate
candidate check `94354297550`; candidate push/PR compiler jobs
`94354135184`/`94354214336`, stable/nightly/Windows LLVM 22 jobs
`94354214389`/`94354214394`/`94354214410`, CodeQL Actions/Python/Rust jobs
`94354210797`/`94354210770`/`94354210832`, and Actions/Python/Rust analyses
`1611711722`/`1611712334`/`1611716646`; merge-head CI/Rust CI/CodeQL
`31671091285`/`31671091296`/`31671091099`, exact merge
compiler/stable/nightly/Windows LLVM 22 jobs
`94355683766`/`94355683532`/`94355683515`/`94355683534`, merge CodeQL
Actions/Python/Rust jobs `94355685544`/`94355685480`/`94355685574`, and
default-branch Actions/Python/Rust analyses
`1611737053`/`1611737605`/`1611740699` all pass.

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

CAP-021 remains accepted product history beneath CAP-023. CAP-019 remains the latest
accepted compiler/profile capability widening, and CAP-015 remains the latest accepted
project integration checkpoint.

### CAP-020 accepted: flat-buffer 2x3-by-3 matvec product gate

Accepted CAP-020 changes no parser, grammar, source semantics, language profile,
semantic analysis, checked IR, verifier, backend, ABI, or capability classification;
it is a zero-production product/evidence checkpoint over CAP-019's
`exact-i32-array-v0` surface.

The accepted application encodes a 2x3 matrix as `[int; 6]`, consumes an `[int; 3]`
vector, computes wrapping `row * 3 + column` in nested loops, returns a fully
initialized mutable-produced `[i32; 2]`, preserves every input lane, produces ordinary
and wrapping results `[50, 122]` and `[-2, 5]`, and exits `91`.

The computed linear value flows through the existing signed bounds and
trap-before-address authority before a `[6 x i32]` load, with corresponding guarded
`[3 x i32]` load and `[2 x i32]` store.

Exact CAP-020 reviewed candidate `3b61cd1ed34f910f556821942cd06301ba17dd50`,
shared candidate/merge tree `800510de85bd82f3332126ad249c95da109dd3e1`,
accepted base and first merge parent `13157687f3e955d1c8292ccca133c5a73e29e1a7`,
and protected PR #58 merge `d9493d5123840b38ebab6ca275aaba3216728706`
whose second parent is that candidate are immutable. Candidate push CI `31639493741`,
PR CI `31639540134`, Rust CI `31639540030`, CodeQL `31639535638`, and aggregate
candidate check `94258433541`; candidate stable/nightly/Windows LLVM 22 jobs
`94258276078`/`94258275978`/`94258275899` and CodeQL Actions/Python/Rust jobs
`94258264605`/`94258264489`/`94258264627`; merge-head CI/Rust CI/CodeQL
`31640016314`/`31640016316`/`31640015733`, exact merge
compiler/stable/nightly/Windows LLVM 22 jobs
`94259869631`/`94259869676`/`94259869637`/`94259869559`, merge CodeQL
Actions/Python/Rust jobs `94259873136`/`94259873164`/`94259873086`, and default-branch
Actions/Python/Rust analyses `1610137115`/`1610137589`/`1610144660` all pass.

CAP-020 adds no matrix type, recursive or nested arrays, static index proof,
checked-overflow arithmetic, stable layout or ABI, performance, accelerator execution,
general mutation, or safety claim.

CAP-019 remains the latest compiler/profile capability widening; CAP-020 is an
accepted product gate, not a separate profile or feature row.

The sole open finding remains pre-existing Actions alert #4 from 2026-08-09; no new
CAP-020 alert surfaced.

### CAP-019 accepted: initialized mutable exact-array production

Accepted CAP-014 created the CPU-only `exact-i32-array-v0` profile; accepted CAP-018
remains its immutable exact-array result-composition checkpoint; accepted CAP-019
widens that same profile with fully initialized mutable owned locals, direct projected
element writes, and returned flat-array values rather than creating another profile.
Accepted CAP-019 widens the existing flat nonempty exact-`Int` class to a fully
initialized mutable owned local whose initializer is an admitted literal, immutable
exact-array identifier, or acyclic ordinary call of the same count, plus direct
`local[index] = exact_int_value` projected writes.

The maintained eight-lane application copies an immutable input, increments every lane
in a guarded loop, returns the whole array by value, feeds it into the accepted CPU
kernel, preserves all eight source lanes, produces result `2035`, and exits `91`;
Linux and Windows retain read traps and add negative/equal-to-count write traps under
verified LLVM/Clang 22 `-O0`/`-O2` routes. Focused profile tests pass 12/12, focused
exact backend tests pass 5/5, stable backend control passes 1/1, the exact integration
passes 17/17, and the complete repository gate and native system gates pass.

Exact CAP-019 reviewed candidate
`f2955bedd22708041e36ee90c65c4f08c443d740`, shared candidate/merge tree
`c520729e7b081087bbe431e97d937fb77f519b37`, accepted base and first merge parent
`84916e124752b8e7d228855a0969cd9eab8dba26`, and protected PR #56 merge
`6ebeb0efb6e83ccc50e12d395e4add1c63ef48b4` whose second parent is that candidate are
immutable. Candidate push/PR CI, Rust CI, CodeQL, and aggregate results
`31627264709`/`31627385522`/`31627385563`/`31627405516`/`94217394313`;
merge-head CI/Rust CI/CodeQL runs
`31627880853`/`31627880924`/`31627880812`; merge jobs
`94218938557`/`94218938794`/`94218938835`/`94218939033`/`94218943455`/
`94218943514`/`94218943605`; and exact default-branch Actions/Python/Rust analyses
`1609396076`/`1609396442`/`1609401493` all pass.

The single selected `exact-i32-array-v0` row remains `END_TO_END`; broad integer and
fixed-array support remains `PARTIAL`; `stable-scalar-v0` remains Aero's only `STABLE`
profile. CAP-019 does not admit general mutable arrays, uninitialized or partial
arrays, mutable parameters/results/aliases, references or escaping places, whole-array
reassignment, zero/recursive/nested/repeat/non-Int arrays, stable aggregate ABI/layout,
general parsing/string/file behavior, GPU execution, performance, or safety. CAP-013
remains the single shared specialization identity/phase authority; CAP-018 and CAP-019
add no specialization classifier.

CAP-015 remains the accepted M1-001 representative-integration checkpoint. CAP-015
changes no compiler production or language-profile code. CAP-016 and CAP-017 remain
completed readiness/architecture stops, not accepted capabilities; neither adds a
profile or matrix row. This retained section records CAP-019's immutable
compiler/profile widening history beneath the accepted CAP-020, CAP-021, and CAP-023
product checkpoints.

### CAP-018 accepted: immutable exact-array value/result composition

Accepted CAP-018 widened the CAP-014-created CPU-only `exact-i32-array-v0` profile
with immutable exact-array results. Ordinary nongeneric functions can construct,
return, bind, forward, pass, and index immutable exact flat `[int; N]`/`[i32; N]`
values for `1 <= N <= i32::MAX`. One shared recursive profile classifier covers the
complete admitted root class--array literal, exact-array identifier, or ordinary named
acyclic call--across result, inferred or annotated binding, call-argument, and
literal/call/identifier index-object placement. Semantic analysis, checked IR,
independent verification, and LLVM production remained unchanged and authoritative.

The maintained eight-lane CPU application constructed and returned an array whose
first source lane 127 became 128, forwarded it through an ordinary helper, consumed it
in the dot-plus-bias kernel, observed independent result 2035, preserved the original
Copy source, and retained exact exit 91. Focused profile authority tests passed 12/12,
the exact-array integration passed 14/14, stable-profile controls passed 10/10,
focused backend controls passed 4/4, and the complete repository and pinned
Linux/Windows LLVM/Clang 22 `-O0`/`-O2` gates passed.

Exact candidate `409eca9ed2dd8b4ba79f34e14ecfefcc0386e3df`, tree
`3073c881c883984f53fcde2f0b205acbec760145`, and protected PR #54 merge
`c49ff17cab7fc0e8d4f552a71499929135c16c61` are immutable. Candidate push CI
`31614934307`, PR CI `31614994226`, Rust CI `31614994253`, and CodeQL
`31614991761` pass. Exact merge-head CI `31615467151`, Rust CI
`31615467115`, and CodeQL `31615465499` pass; default-branch Actions, Python,
and Rust analyses `1608636029`, `1608636345`, and `1608644785` also pass.

CAP-014 remains the profile origin and Aero's first bounded Milestone 3 CPU slice;
CAP-018 remains its immutable result-composition checkpoint. CAP-019 subsequently
adds initialized mutable owned locals, guarded direct element writes, and returned
flat-array values within that same selected profile. General mutable arrays, mutable
parameters/results/aliases, whole-array reassignment, recursion, empty/repeat/nested/
non-integer arrays, structs/enums/tuples/references inside the named profile,
modules/imports, constants, methods, generics/traits, closures, dynamic collections,
allocation/drop, I/O, accelerators, stable layout or ABI, performance, safety,
release eligibility, and Aero as a whole remain excluded.

### CAP-015 accepted: embedded character-record representative integration

Accepted CAP-015 enriches the existing M1-001 representative telemetry application
with one exact embedded-data path. The maintained program interprets the fixed
`[char; 10]` record `T=<digit><digit>;H=<digit><digit>;`, returns the first
unexpected character through `Result<int, char>`, and computes
`temperature * 2 + humidity` on success. Canonical input `T=17;H=08;` produces 42;
the application materially uses that value as its third calibration sensor while
retaining exact stdout `telemetry score: 91` and exit 91. Boundary inputs produce 0
and 297, all ten malformed positions return the exact offending character, and three
multi-error controls preserve left-to-right first-error precedence.

CAP-015 changes no compiler production or language-profile code. The tracked
integration consumes already accepted experimental character identity, fixed-array
reads with pre-address bounds guards, concrete `Result` transport and exhaustive
`Match`, direct modules, and the canonical checked-program pipeline. Raw and semantic
checked metadata retain `char`, `[char; 10]`, and `Result<int, char>`; public and
library parser-function LLVM agree; each of the ten dynamic reads binds its own
lower/upper predicates, conjunction, branch, trap/safe blocks, conversion, and
`[10 x i32]` GEP; and equality-only ASCII decoding introduces no character-to-
numeric conversion. Both `stable-scalar-v0` and `exact-i32-array-v0` continue to
reject the module-free parser specimen.

Exact candidate `dd9b1710abebf2f2318582cf94568c2f9a30ca8f` and protected PR #52
merge `b62696272f293f9f378f8a368cc818fcb8ef1074` share tree
`27f359bc5ca90212a06ce73b71759cac0533c1f0`. Candidate push/PR CI
`31597830488`/`31598146528`, Rust CI `31598146473`, and CodeQL `31598144554`
pass. Exact merge-head CI `31598634185`, Rust CI `31598634090`, and CodeQL
`31598633803` pass, including stable, nightly, pinned Windows LLVM/Clang 22, public
`check`/verified `build`/`run`, LLVM and machine verification, native `-O0`/`-O2`
output/exit 91, and clean negative/equal-to-count bounds traps.

CAP-014 remains the origin of Aero's exact-array profile and first bounded Milestone 3
CPU computation slice; accepted CAP-019 now supplies its latest compiler/profile
capability. CAP-015 remains the latest separately classified M1-001 project
integration checkpoint and enriches the existing representative `END_TO_END` row only; it adds no
parser, grammar, profile, feature, stability, or conformance row.
General-purpose text parsing, runtime Strings, serialization, runtime ingestion, file
input, and Unicode text encoding/normalization remain unsupported; accepted CORE-072's
bounded Unicode scalar `char` remains `PARTIAL`.
General error propagation, variable-width input, dynamic collections, allocation/
drop, I/O, public ABI, safety, performance, releases, and language completion also
remain outside the accepted claim. CAP-015's immutable evidence remains the protected
merge `b62696272f293f9f378f8a368cc818fcb8ef1074`; CAP-019's later protected merge
`6ebeb0efb6e83ccc50e12d395e4add1c63ef48b4` remains its immutable
compiler-widening checkpoint.

### CAP-014 accepted: exact `i32` fixed-array CPU reference kernel

Accepted CAP-014 is Aero's first bounded Milestone 3 CPU computation slice. The
distinct CPU-only `exact-i32-array-v0` profile composes the accepted exact wrapping
scalar lane with flat `[int; N]`/`[i32; N]` values for `1 <= N <= i32::MAX`. It
admits explicitly annotated immutable local literals, by-value nongeneric parameters,
identifier call transport, and direct scalar indexing. The representative eight-lane
dot-product-plus-bias program executes at the independent-oracle exit 91, while a
separate edge specimen proves wrapping array-lane arithmetic at exit 93 without
`nsw`/`nuw`. Every dynamic access uses signed bounds guards before address formation
and sign-extends the checked index for the LLVM GEP.

One pre-semantic profile authority owns the complete admitted source class and one
profile-aware backend mapping lowers its verified logical `Int` leaves to LLVM `i32`.
Semantic analysis, checked-IR generation, and independent verification remain
unchanged; the established checked identities and schemas are consumed rather than
reimplemented. Focused CAP-014 evidence passes 11/11, exact profile/backend controls
pass 7/7, CLI routing passes 35/35, stable-profile separation passes 10/10, legacy
fixed-array controls pass 12/12, and the complete repository gate passes 259 library
tests plus every integration, CLI, documentation, formatting, and correctness-Clippy
target. Pinned LLVM/Clang 22 verifies and executes the exact O0/O2 behavior on Linux
and Windows.

Corrected candidate `226279dd174f26dc3cd1c7573798955bfe789f78` and protected
PR #50 merge `ca09ebe3c1b981339c8bf56b360e62208ac900e1` share tree
`448e1c2ff397012804b886b904aa43bec63f2d37`. Candidate push/PR CI
`31570455915`/`31570461500`, Rust CI `31570461524`, and CodeQL `31570456382`
pass. Exact merge-head CI `31570823665`, Rust CI `31570823712`, and CodeQL
`31570823073` also pass, including stable, nightly, and pinned Windows LLVM 22 jobs.

This does not widen `stable-scalar-v0`. Accepted CAP-018 subsequently widened this
same named profile for immutable exact array results rooted in literals, identifiers,
or ordinary acyclic calls across result, binding, call-argument, and index-object
placements. Accepted CAP-019 then added fully initialized mutable owned locals,
guarded direct element writes, and returned flat-array values. General mutable arrays,
mutable parameters/results/aliases, whole-array reassignment, empty/repeat/nested
arrays, recursion, non-integer elements,
struct/tuple/enum or reference use, modules/imports, constants, methods,
generics/traits, closures, dynamic collections, allocation/drop, I/O, accelerators,
and non-CPU target pairing remain rejected. The profile does not stabilize aggregate layout, callable ABI,
serialization, packages, SIMD, quantization, tensors, performance, safety, Aero as a
whole, or release eligibility. Aero remains a Minimal Prototype in correctness
recovery. CAP-014 remains the profile origin beneath accepted CAP-018 and CAP-019;
CAP-015 remains the separate representative-integration checkpoint.

### CAP-013 accepted: canonical specialization identity and phase authority

Accepted CAP-013 closes an observed cross-feature specialization defect: the
primitive contract already treats `int`/`i32` and `float`/`f64` as aliases, but
generic structs, enums, fixed-capacity containers, and trait signatures previously
could reject equivalent values or manufacture duplicate private identities. One
shared authority now owns recursive specialization keys, canonical rendering/parsing,
feature-tagged private framing, alias equivalence, source-symbol validation, and the
deterministic idempotent struct -> enum -> function phase plan used by both semantic
analysis and semantic-independent checked admission. Feature modules retain their
frozen validation, rewriting, substitution restrictions, diagnostics, and the single
generic-function body classifier.

The representative telemetry application now passes a `Window<i32>` seed through
canonical `Window<int>` generic algorithms and implements an `int`-declared trait
method with an `i32` parameter while preserving exact output and exit 91. Focused
alias/identity evidence passes 9/9, established generic/trait controls pass 21/21,
shared authority tests pass 7/7, representative tests pass 3/3, and the complete
repository gate passes 249 library tests plus every CLI, integration, Windows-system,
format, correctness-Clippy, and doc target. Exact candidate
`1ecf0831149b99abd55e3d0a48d06eecaa8099b6` passed push CI `31562192900`, PR CI
`31562195772`, Rust/Windows LLVM 22 `31562195887`, and CodeQL `31562193465`.
Protected PR #48 merged it as accepted master
`856fc1e5f310b2b458f97d7b6aebb1ecf5c28572`; candidate and merge share tree
`627582e27613803949c82ecfa52915ba55db2f0f`, and exact merge-head CI
`31562387501`, Rust/Windows LLVM 22 `31562387504`, and CodeQL `31562387066` pass.

This is bounded alias interoperability and shared compile-time architecture, not
general generics or traits. Nested generic templates, generic impl targets, reference
specialization, dynamic collections, allocation/drop, layout/ABI, accelerators,
performance, releases, safety, stability, and language completion remain excluded.
CAP-013 remains the accepted specialization architecture beneath CAP-014; no new
specialization refactor is selected merely because its feature modules are large.

### CAP-012 accepted: nonescaping projected CopyData call loans

Accepted CAP-012 composes the finite projected-place and
call-bounded reference authorities so ordinary nongeneric helpers can immediately
observe or update a nested field/tuple/fixed-array CopyData place. One shared place
classifier owns root facts, recursive path/type identity, and checked selectors for
both projected assignment and call loans; semantic analysis and independent checked
admission consume that classifier. Checked IR carries the complete owner root,
projected source, exact logical types, mutability, and explicit call-end lifecycle for
independent verification before LLVM.

The telemetry application now reads
`batch.sensors[index].value` and updates `batch.meta.0` through ordinary reference
helpers while retaining exact output/exit 91. Focused 3/3, representative 3/3,
surrounding reference/assignment suites, nine verifier-corruption mutations, format,
correctness Clippy, 242 library tests, 35 CLI tests, every integration target, and doc
tests pass. Exact candidate `79d14866061184bc619ce5c92603c0964a31e74d`
passed push CI `31552115552`, PR CI `31552154519`, Rust CI `31552154532`, and
CodeQL `31552152210`, including pinned LLVM/Clang 22 native execution. Protected PR
#46 merged it as accepted master `49bcdfc3b23d2e1cc22fa3f0f36446fcffbf6e92`;
candidate and merge share tree `83496c29ce1b8529ed3b18e6bb0d0fb42fdb7333`,
and exact merge-head CI `31554826581`, Rust CI `31554826582`, and CodeQL
`31554826233` pass. Together with CAP-011's generic data structure, this satisfies
the selected Milestone 2 exit gate without promoting the wider milestone surface.

Stored or escaping references, reference results/fields/containers, partial moves,
disjoint-projection alias claims, generic/method/trait call expansion, dynamic
collections, lifetimes/NLL/drop/unsafe, stable ABI, safety, and stability remain
outside the accepted slice.

### CAP-011 accepted: fixed-capacity generic CopyData container algorithms

Accepted CAP-011 composes generic structs, generic
function specialization, fixed arrays, runtime bounds guards, and projected mutation
into one useful `Window<T>` API. A generic function may accept an exact parametric
CopyData container, infer concrete element types through its schema, read structural
paths, update a copied mutable local, and return the specialized container. Telemetry
uses the same API for `int` calibration and `char` markers while preserving expected
output/exit 91.

Focused generic tests pass 4/4, representative tests pass 3/3, the identity corruption
control passes, and the complete root gate passes 241 library tests plus all
integration, doc, format, and correctness-Clippy gates. Exact candidate
`dea5714e87e1fe3c059717b3e5aa23b8860b62f2` passed all nine public results,
including pinned Windows LLVM/Clang 22 native execution. Protected PR #44 merged it as
accepted master `34b81eee1d598cbd00693f40301f1a4f72481d07`; candidate and merge share
tree `b16d5f68e791c969a0aa81723801b2d1ae7c7f41`, and exact merge-head CI
`31546637614`, Rust CI `31546637544`, and CodeQL `31546636973` pass.

This is not dynamic collection or general generic-operation support. Parametric
construction, operators, arbitrary calls, borrowing, generic-to-generic calls,
references, enums, allocation, lifetimes/drop, public ABI, safety, and stability stay
outside the accepted slice.

### CAP-010 accepted: required-only CopyData trait-bound static dispatch

Accepted CAP-010 admits one deliberately bounded form of behavioral polymorphism:
unique nongeneric required-only traits, exact immutable `&self` methods over recursive
finite CopyData parameters/results, unique complete impls for nongeneric recursive
CopyData structs, and direct trait-method calls inside whole-value generic functions.
Calls specialize to deterministic private monomorphic helpers; checked signatures bind
the helper identity to its concrete receiver and the independent verifier rejects
identity, schema, callee, arity/order, and borrow-provenance corruption.

The representative telemetry application now implements one policy trait for both
`Sensor` and `Batch` and obtains its observed value and bias through the same generic
algorithm while retaining exact score/exit 91. Focused trait tests pass 3/3,
representative tests pass 3/3, affected generic/reference/module/closure suites pass,
and the complete repository gate passes 240 library tests, 35 binary tests, all
integration targets, doc tests, formatting, and correctness-denying Clippy. Exact
candidate `2e0bfde46022b9617095319452e44d14b0ef31f8` passed all nine public results,
including pinned Windows LLVM/Clang 22 native execution. Protected PR #42 merged it as
accepted master `f77f1a227032008ab3ceadf2e2e3dcaed3b225e9`; candidate and merge share
tree `0217e2fa98728b2ad101be1ecc85481a8f4145b8`, and exact merge-head CI
`31540589044`, Rust CI `31540589040`, and CodeQL `31540588732` pass.

Default methods, associated items, supertraits, where-only bounds, generic traits or
impls, inherent impls, enum/primitive/non-CopyData targets, trait objects/dynamic
dispatch, generic-to-generic calls, references as generic arguments, collection or
resource receivers, captures, lifetimes/drop, public ABI/FFI, accelerators, safety,
stability, benchmarks, releases, and general trait/generic semantics remain excluded.

### CAP-009 accepted: enforceable `stable-scalar-v0`

Accepted CAP-009 adds an explicitly selected
`stable-scalar-v0` language profile to public library compilation/checking and CLI
`check`, `build`, and `run`. One exhaustive post-parse classifier rejects every
out-of-profile AST form before module resolution, semantic analysis, checked IR,
cache lookup, LLVM generation, or artifacts. A validated `CheckedProgram` privately
carries its profile into code generation, preventing callers from pairing checked IR
with a different physical representation. The default experimental profile and its
cache identities remain byte-for-byte unchanged.

The selected profile freezes one-file, acyclic nongeneric `int`/`bool` functions,
initialized direct scalar bindings and assignments, direct calls and returns,
`if`/`else`, `while`, and the enumerated scalar expression grammar. Its physical
integer lane is exact wrapping LLVM `i32`; CLI target pairing is CPU-only; generated profile LLVM contains no
`double`, `fptosi`, `sitofp`, `nsw`, or `nuw`. The growing profile application exits
91, and its wrapping-boundary companion exits 93. Focused 10/10, affected library/CLI/
cache/API contracts, formatting, correctness-denying Clippy, documentation, and the
complete repository gate pass. Exact candidate
`bfd03ff770afc4aad69dc9925b8ad244d6761ec0` passed all nine public checks,
including pinned Windows LLVM/Clang 22 external, machine, native `-O0`/`-O2`, and
public-route evidence. Protected PR #40 merged it as accepted master
`1ef21c564ec564379e611002b1b321d910a991a3`; candidate and merge share tree
`29a481d53b05dc36c96f2eb90beac394d3e5bef6`, and exact merge-head CI
`31534644903`, Rust CI `31534644999`, and CodeQL `31534643685` pass.

The profile excludes modules/imports, floats, characters,
strings and I/O, aggregates, enums/`Match`, references, closures, methods, general
loops, division/remainder, traits/generics, recursion, allocation, lifetimes/drop,
unsafe, public ABI/FFI, accelerators, benchmarks, and releases. It does not make Aero,
the default compiler profile, private IR, or the whole language stable.

### CAP-008 accepted: nonbinding wildcard enum Match

Accepted CAP-008 adds terminal whole-arm `_ => fallback` and ignored
payload leaves such as `Err(_)` across the complete already-admitted concrete enum
class: unit, unary, positional multi-field, concrete `Option`/`Result`, and explicit
generic-enum specializations. One shared `EnumRegistry::resolve_arms` authority owns
mapping and binding classification for semantic analysis, semantic-independent checked
admission, and checked lowering. Wildcards create no binding or payload extraction;
explicit arms remain unique and exact; one optional whole-arm wildcard must be final
and covers only otherwise-unmapped declared variants.

The verifier was not weakened. Lowering materializes one unique checked target per
declared variant, so a shared source fallback remains mutually exclusive and retains
the existing dispatch and result-dominance invariants. Focused 4/4, affected enum and
carrier suites, the representative application, and the complete normalized 235-
library/32-binary/integration/doc/format/Clippy/verifier-control gate pass. Official
Windows LLVM/Clang 22.1.8 external and machine verification, native `-O0`/`-O2`, and
public `run` all retain exact telemetry output and exit 91.

Exact candidate `9ebd204407f09d14092bb4ed874e19afc5bf6105` passed push CI
`31524994573`, PR CI `31525033075`, Rust CI `31525033103`, and CodeQL
`31525030957`. Protected PR #38 merged it as
`a1716f8400052213c8f08e5c397a3d779437dff6`; candidate and merge share tree
`c3dab0e762b27b5f49d8b25b424f24f7d5d66c1b`, with ordered parents prior accepted
master `2ba1d33e302439b129f538533ecf5187b07aa34a` then exact candidate. Exact
merge-head CI `31525340621`, Rust CI `31525340810`, and master-push/CodeQL
`31525340605` pass. Guards, nested destructuring, whole-enum binding, error
propagation, collections, imports, aggregate/reference enum storage, partial moves,
lifetimes/drop, stable ABI, safety, releases, benchmarks, and accelerators do not move.

### CAP-007 accepted: canonical checked-program entrypoint contract

Accepted CAP-007 adds public artifact-free
`check_program` and `check_file` library APIs and one library-owned preparation
authority for lexing, fatal parsing, direct modules, semantic analysis, checked IR,
and mandatory internal verification. Library compilation plus CLI `check`, optimized
`build`/`run`, `profile`, and source discovery under `aero test` now consume that
authority. A semantics-pass program rejected by checked admission can no longer be
reported as a completed source test.

Focused canonical-pipeline tests pass 3/3, and the complete normalized local gate
passes 235 library tests, 32 binary tests, all 84 integration targets, doc tests,
formatting, correctness-denying Clippy, examples, and repository/verifier controls.
Cached official Windows LLVM/Clang 22.1.8 also externally and machine verifies the
representative product; public `run` and native `-O0`/`-O2` executions retain exact
output and exit 91.

Exact candidate `bfb7adb36af4c2d5bef65d615a2f8e32bff7902b` passed candidate push
CI `31519719961`, PR CI `31519753426`, Rust CI `31519753421`, and CodeQL
`31519751604`. Protected PR #35 merged it as
`5a64acaffa5e7f7167823861a45bc49c6bb670b4`; candidate and merge share tree
`3ab515666d310a7e72f28347ecd2561297ce0eca`, with ordered parents prior accepted
master `80f35b4bcf1892838c3e43bdb5dfb0fd21caed06` then exact candidate. Exact
merge-head CI `31520069468`, Rust CI `31520069315`, and CodeQL `31520068887`
pass, including stable/nightly and pinned Windows native execution. No source
semantics, IR schema, backend lowering, runtime behavior, release, benchmark, or
stability claim changes. Aero remains a Minimal Prototype in correctness recovery.

### CAP-006 accepted: explicit user-defined generic CopyData enums

Accepted CAP-006 adds one bounded user-defined generic algebraic-data-type class.
One shared idempotent authority
retains and rejects generic-enum bounds, validates the frozen declaration/application
class, and specializes exact explicit recursive finite CopyData arguments into private
nongeneric enum definitions before both semantic analysis and raw checked admission.
Existing owned-enum construction, transport, replacement, control flow, exhaustive
`Match`, checked IR, independent verification, and LLVM remain the executable path.

The representative telemetry application now defines `Sample<T>` and composes
`Sample<Reading<int>>` with `Sample<char>` through ordinary nongeneric functions and
exhaustive `Match` while preserving exact output and exit 91. Focused CAP-006 tests
pass 4/4; malformed/private-schema corruption and affected compatibility controls
pass; and the complete local gate passes 235 library tests, 32 binary tests, all 83
integration targets, doc tests, formatting, correctness-denying Clippy, and diff
hygiene. Public `check` and `build` emit deterministic LLVM with source-readable
specialization identities. Pinned LLVM/Clang 22 externally and machine-verifies the
product; Linux stable/nightly and Windows `-O0`/`-O2` representative executions retain
exact output and exit 91.

Exact candidate `5f20a5543503736422115145b12fd6c593c5eeb3` passed push CI
`31514226887`, PR CI `31514272313`, Rust CI `31514272209`, and CodeQL
`31514269749`. Protected PR #33 merged it as
`bdfd4f5a282043ee957c1bf03975e266de5b9b6c`; candidate and merge share tree
`98cd0e3fa09ad255d4e58cd4c860dcf0271ea915`, with ordered parents prior accepted
master `233716c7521cb0186eb781549e12e67b277009a6` then exact candidate. Merge-head
CI `31514651356` and CodeQL `31514652091` pass. Merge-head Rust CI
`31514651355` initially hit a Windows runner certificate-chain failure during checkout;
its unchanged-SHA failed-job rerun passes stable, nightly, and the pinned Windows
LLVM/Clang 22 native system gate.

Bounds/traits, named variants, nested generic templates, generic-to-generic use,
context-free inference, non-CopyData arguments, aggregate/reference storage,
borrowing, projected access, partial moves, drop/lifetimes/unsafe, collections, public
layout/ABI/FFI, accelerators, releases, benchmarks, stability, production, and general
memory-safety claims remain excluded. Aero remains a Minimal Prototype in correctness
recovery, not a complete or stable language.

### CAP-005 accepted: bound-free CopyData generic transport functions

Accepted CAP-005 adds one bounded
compile-time specialization class for unique top-level, bound-free generic functions.
Every type parameter is inferred from an exact whole-parameter occurrence, every
substitution is already admitted recursive finite CopyData, and the parametric body
may only copy whole values through direct parameters, explicitly typed direct locals,
branch selection, reassignment, and return. One shared idempotent authority validates
the template and call product, emits private nongeneric specializations before both
semantic analysis and checked admission, and gives the independent verifier enough
identity data to reject signature or substitution forgery. No runtime polymorphism,
boxing, type tags, or second executable semantic path is introduced.

The focused product passes 5/5, its shared-contract controls pass 2/2,
the private-identity corruption control and all affected compatibility rings pass,
and the representative telemetry program uses the same `choose<T>` helper for
`Reading<int>` and `Reading<char>` while retaining exact output and exit 91. The
complete local repository gate passes 232 library tests, 32 binary tests, every
integration target, doc tests, formatting, and correctness-denying Clippy. Pinned
LLVM/Clang 22.1.8 externally verifies and machine-verifies the candidate; independent
Windows `-O0` and `-O2` executables and public `run` produce exact output and exit 91.
Exact candidate `68e2cd8ef7766f3073bc9bf9714dafb0431525ef` passed all nine
candidate-head checks and protected PR #31 merged it as
`59f7e47b476871fae8cecdf7e40900e0d1f1b377`. Candidate and merge share tree
`e0e6ac4d0bdec61fff30efee387b870ce63a282c`. Exact merge-head CI
`31504122753`, Rust CI `31504122730`, and CodeQL `31504122424` pass.

Bounds and traits, generic methods/impls/enums, recursion, generic-to-generic calls,
operations or projection on abstract values, expected-result inference, nested generic
applications in signatures, non-CopyData arguments, collections, heap/drop/lifetimes,
public ABI, accelerators, releases, benchmarks, stability, and general memory-safety
claims remain excluded.

CAP-005 remains accepted chronology through protected merge
`59f7e47b476871fae8cecdf7e40900e0d1f1b377`. Bounded PR #31 carried exact candidate
`68e2cd8ef7766f3073bc9bf9714dafb0431525ef`; candidate and merge share tree
`e0e6ac4d0bdec61fff30efee387b870ce63a282c`, with ordered parents prior accepted
master `d20480ec9d12a1d2b58f01666ff6fd762544b22e` then the exact candidate. Candidate
CI runs `31502822144` and `31502857390`, Rust CI `31502857406`, and CodeQL
`31502855546` pass. Exact merge-head CI `31504122753`, Rust CI `31504122730`, and
CodeQL `31504122424` also pass.

Accepted `CAP-004` adds Aero's first executable user-defined generic data definition:
a uniquely named `struct Name<T, ...>` can be instantiated at exact explicit recursive
finite CopyData arguments and used through existing construction, projection, Copy
transport, arrays/tuples/struct fields, projected mutation, immutable whole-place
references, nongeneric functions, checked IR, independent verification, and LLVM.
One shared idempotent pre-semantic/pre-admission authority performs exact substitution
and derives private identities that commit to canonical source spelling and field
schema; source diagnostics and LLVM retain readable identities such as
`Reading<int>`. The representative telemetry application executes `Reading<int>` and
`Reading<char>` while preserving exact output and exit 91. Focused tests are 5/5,
generic-contract controls are 4/4, the verifier corruption control passes, and the
complete 229-library/32-binary/integration/doc root gate plus correctness-denying
Clippy pass locally. All nine exact-candidate checks, protected integration, and all
exact merge-head workflows pass; pinned LLVM/Clang 22.1.8 stable Linux and Windows
lanes verify, lower, link, and execute the representative program at `-O0` and `-O2`.
CAP-004 itself did not add generic functions/enums/impls/traits, inference/defaults, generic applications inside
generic definitions, non-CopyData arguments, collections, heap/drop/lifetimes, public
ABI, accelerators, releases, benchmarks, and general memory-safety claims remain
excluded.

Accepted `CAP-003` admits explicitly typed concrete `Option<T>` and
`Result<T, E>` values only when all
type arguments are already admitted recursive finite CopyData. One shared,
idempotent contextual normalizer maps exact annotations, constructors, function
transport, reassignment, and bound exhaustive `Match` to collision-checked private
non-generic enum identities, then reuses the accepted enum ownership, checked-IR,
verification, and LLVM contracts. All four constructors require an exact context; no
missing argument is inferred or defaulted. The representative telemetry application
now executes both success and error `Result<int, char>` paths while retaining exact
score 91. Focused carrier tests are 4/4, the shared normalizer controls are 4/4,
adjacent enum/ownership rings pass, representative tests are 3/3, and the complete
224-library/32-binary/integration/doc root gate plus correctness-denying Clippy pass
locally. All nine exact-candidate public checks, the protected merge, and all exact
merge-head workflows pass. Pinned LLVM/Clang 22.1.8 Linux and Windows lanes verify,
lower, link, and execute the representative program at `-O0` and `-O2` with exact
output and exit 91, and the bounds-failure corpus passes. General generics,
context-free constructor inference, question-mark propagation, String errors,
carrier aggregate/reference storage, borrowing, nested carriers, dynamic collections,
stable ABI/FFI, accelerators, releases, benchmarks, and general memory-safety claims
remain excluded.

Accepted `CAP-001` has frozen implementation
`77a5c6457f3e87d79a0d1987bf02b4db32b875de`, tree
`555e5290fe8ece199a64d8b0b7450ee411f75b23`, stable patch ID
`b80e014125340632d7f9fe411434ed75ad1f838a`. Its post-M1 ranking selected
verified runtime fixed-array reads ahead of the canonical Milestone 0 trusted-
entrypoint contract and positive import/name resolution. Before the change, a
nonconstant out-of-range `int` index could reach unchecked LLVM `inbounds` address
formation and falsely execute successfully. Every runtime index read over the
existing nonempty recursive CopyData fixed-array universe now uses one ordered
nonnegative/below-count guard, terminates before conversion or access on failure,
and retains direct constant-index lowering plus compile-time constant-bounds
diagnostics. Dynamic projected writes, projected borrowing, collections, stable
runtime/ABI behavior, and general memory safety remain excluded.

CAP-001 enriches the representative telemetry application with computed index reads
and adds a two-file negative/equal-to-count runtime-failure corpus. Focused 4/4,
representative 3/3, all 218 library tests, adjacent array/verifier/workflow controls,
the complete 218-library/32-binary root gate, and LLVM 22 external and machine
verification pass. All nine candidate-head checks pass. Exact merge-head Linux
stable and pinned Windows LLVM 22 gates both record representative `-O0`/`-O2` exit
91 and the failure corpus passing at both optimization levels.

Accepted `CAP-002` extends the existing mutable owned direct-local recursive CopyData
projection class so every fixed-array selector may be either a retained in-range
constant or an exact semantic `int` evaluated at runtime. Target selectors execute
once in source order before the RHS; each runtime selector reuses the accepted
CAP-001 nonnegative/below-count guard before later selectors, RHS evaluation, address
formation, or memory access. In-range execution stores one exact CopyData leaf. The
representative telemetry application now fills its sensor array in a bounded loop,
and two runtime-failure specimens require negative and equal-to-count write
indexes to trap before an effectful RHS. Focused 5/5 assignment tests, representative
3/3, all 220 library tests, all 32 binary tests, every integration target, formatting,
correctness-denying Clippy, and doc tests pass locally. Exact candidate-head CI runs
`31464531658` and `31464562605`, Rust CI `31464562651`, and CodeQL `31464562623`
pass, including stable/nightly Linux and pinned Windows LLVM 22 `-O0`/`-O2` execution.
Protected merge and all exact merge-head workflows pass. Writes through
references, projected borrowing, partial moves, slices/collections, compound
assignment, non-CopyData places, stable trap/ABI behavior, accelerators, releases,
benchmarks, and general memory-safety claims remain excluded.

Milestone 111 `CORE-090` is accepted public at exact candidate head
`af68d0e842ed2973087d2e3c78d2a19546e29ff7`, tree
`8455a06a4473a826ef1ea180e291e2ddb790bed0`. Its frozen implementation commit is
`ca00cdb70fc0a1940fa94126c49774b99d03c515`, tree
`701b2d9a6a371fdfba8a6e11ff34c44f023731df`, with stable patch ID
`623c295190227eda07712cad34a36ec81c2e43c7`. Bounded PR #17 passed all nine
exact-head checks and merged through protected master as
`128205615c53156138c4effa740b61ab455a760f`, with the exact candidate tree and
ordered parents prior master `7fbaaaa4e26e50edd667771f6bce268e2429043e`
then the candidate. Post-merge CI `31421325338`, Rust CI `31421326818`, and
CodeQL `31421323785` pass on that exact merge.

The accepted class closes statically addressed projected CopyData assignment: an
initialized mutable owned direct-local CopyData root may be followed by any nonempty
finite mix of declared struct fields, tuple constants, and nonnegative in-range
integer-literal fixed-array indexes, and the exact CopyData leaf may be replaced by an
exact-type RHS. Semantic analysis, semantic-independent checked admission, and
lowering consume one shared contract; independent verification reconstructs the
projection root and requires a typed mutable owner before admitting the leaf write.
Focused execution passes 1/1, classifier and verifier corruption controls pass 2/2,
the affected ring passes 15/15, the root gate passes 218 library and 32 binary tests
plus every integration/doc target, and pinned LLVM/Clang 22.1.8 executes exact native
exit 90 on Linux and Windows public lanes. Dynamic/computed target indexes, projected
borrowing, partial moves, enum/non-Copy subplaces, alias analysis, NLL/lifetime/drop,
public layout, stable ABI/FFI, accelerators, and general memory-safety claims remain
excluded.

`ROADMAP-001` is accepted through bounded protected PR #18 as master
`2bfa75d017e8b13fca3b22c148f11102e39f35bd`, with candidate tree
`c66ca823b3f3e5f681c12b99eee86efa7aaa79c5` and green exact post-merge CI
`31424978028`, Rust CI `31424978021`, and CodeQL `31424975839`. It changed no
compiler behavior; it selected `M1-001` as the highest-leverage next checkpoint.

`M1-001` is accepted public at exact candidate head
`e7a74e600d2e4dc7a58a383ae14282b7923e1a5e`, tree
`193c9590af18eed16fb126affcecc1dd617f98a4`. Its frozen implementation commit is
`a6705709f98b6eee76ecf68a778949c52282bb5b`, tree
`ce53892fe952b902985d3fd9644fc69bd18bc7a0`, with stable patch ID
`6f702defa4ba8dba9d123c6ce249a68cd010126e`. Bounded PR #19 passed all nine
exact-head checks and merged through protected master as
`d7d1c7682911503470a19c97acb72d231824b193`, with a tree exactly equal to the
candidate and ordered parents prior master `2bfa75d017e8b13fca3b22c148f11102e39f35bd`
then the candidate. Post-merge CI `31429517811`, Rust CI `31429517729`, and CodeQL
`31429517265` pass on that exact merge. The accepted capability introduces the
tracked three-file representative telemetry application and compile-fail corpus,
composes the accepted scalar/direct-module/function/control-flow/CopyData/enum/
reference/projected-write surface, and returns 91 after printing one computed result.
It also closes one shared CPU-backend false-success: numeric `print!`/`println!`
arguments remain typed LLVM `double` values and each call spells the explicit
variadic `printf` callee type, allowing the target backend to satisfy the Windows x64
vararg ABI. Local public `check`, verified `build`, and `run` pass; official LLVM/
Clang 22.1.8 verifies the IR and machine instructions, and Windows `-O0`/`-O2`
executables have identical exact stdout/stderr and exit 91. The focused target passes
3/3 and the complete root gate passes 218 library tests, 32 binary tests, every
integration target, formatting, correctness-denying Clippy, and doc tests. Linux and
Windows exact-head workflows, protected merge, and post-merge verification all pass.
The representative application/conformance-subset workflow is
therefore `END_TO_END`; its component language rows remain bounded `PARTIAL` and this
is not a stable grammar, general ABI, ownership/memory-safety, optimizer-performance,
or release claim. The interrupted neighboring projected-borrow experiment remains
quarantined locally.

Milestone 110 `CORE-089` is accepted public at exact candidate head
`82cec768ec117d18517e5624a9afa012c4fe66c6`, tree
`e2f0da6ffd591a6ee22cf6585e48159d914cfd81`. Its frozen implementation commit is
`1ce9ff3db9157dbade2886c6327b092e3c5503f4`, tree
`70970690a5758dc1020949ca4669c299451adc08`, with stable patch ID
`b92e829b88dee5ef97b944d6d8c3765ae1665141`. Bounded PR #16 passed all nine
exact-head checks and merged through protected master as
`7fbaaaa4e26e50edd667771f6bce268e2429043e`, with the exact candidate tree and
ordered parents prior master `a7627aa107a137f00ad17b4175de438e7d43f4e0` then the
candidate. Post-merge CI `31416437489`, Rust CI `31416438721`, and CodeQL
`31416437379` pass on that exact merge. The accepted class admits every ordered
non-entry, non-generic signature containing two or more pairwise-disjoint mutable
whole-place references, any immutable references, and recursive finite CopyData
companions under one shared classifier and one exact N-borrow/call/reverse-N-end
window. Projected calls, reference results/escape/storage/capture, NLL/lifetime/drop,
public layout, stable ABI/FFI, accelerators, and general memory-safety claims remain
excluded.

Milestone 109 `CORE-088` is accepted public at exact candidate head
`4e807b59c32890ee31406bfc10d4720a8f3228ab`. Its frozen implementation commit is
`f0833a80aa6e91d71faf837879072c63ef1eba15`, tree
`83445e55b5847ced27f5843fbc3cb50debdcb995`, with stable patch ID
`6f87a3a53cccb60f413229749988573a85c4c617`. Bounded PR #15 passed all nine
exact-head checks and merged through protected master as
`a7627aa107a137f00ad17b4175de438e7d43f4e0`, tree
`a8cc832796b0802549291d9b633c13979cfef623`, with ordered parents prior master
`b07efe291c8d2e03e6796a6e74284f7304f8162c` then the exact candidate. Post-merge
CI `31410739806`, Rust CI `31410739830`, and CodeQL `31410738951` pass on that
exact merge. The accepted class composes exactly one mutable whole-place reference
with one or more immutable whole-place references and any recursive finite CopyData
companions in every declared order, under one shared classifier and an exact adjacent
borrow/call/end window. Multiple mutable parameters remained excluded in CORE-088.

Milestone 108 `CORE-087` is accepted public at exact candidate head
`0c144276906d5038066867e6a8689de1124d6d60`. Its frozen implementation commit is
`a1e5c697e34eb7be1b7ce46da21a3fadfda53ca0`, tree
`d3c0b317133e3a1ecfc1b7d72551585418928820`, with stable patch ID
`0312d58f4422bd8c8e598b0cbf4c63f208097e5d`. Bounded PR #14 passed all nine
exact-head checks and merged through protected master as
`b07efe291c8d2e03e6796a6e74284f7304f8162c`, tree
`337f5350d401d1fed65d383920cd6d23c892a940`, with ordered parents prior master
`e2014a1762664461ad8fc952cece6f9fa39aa4c2` then the exact candidate. Post-merge
CI `31406731077`, Rust CI `31406731094`, and CodeQL `31406730798` pass on that
exact merge.

The accepted CORE-087 class composes exactly one mutable whole-place reference with
one or more independent recursive finite CopyData parameters in any declared
position. Its focused target is 3/3, verifier corruption control is 1/1, complete
repository gate is green, and the pinned Linux/Windows LLVM 22 system specimen
executes exact exit 87. Multiple reference parameters, owner-dependent side
arguments, projections, reference results or escape/storage/capture, lifetime/drop,
stable ABI/FFI, accelerators, and general memory-safety claims did not move in that
checkpoint.

Milestone 107 `CORE-086` is accepted public at candidate head
`e4447599b7989ba1276d4f32ddc8832900877132`, with frozen implementation commit
`e12d6fe83cc1c8bd085a0b61e3d2f131003bfd81`, tree
`319f7a8136a852781aa852816a24c609097f15ca`, and stable patch ID
`b8d0c027561176156ae8dedcb4faf2f200ad2d8e`. Bounded PR #13 merged through
protected master without head substitution as
`e2014a1762664461ad8fc952cece6f9fa39aa4c2`, tree
`4caeee7d9602783525bb87856908ac459447184d`, with ordered parents old master then
the exact candidate. Post-merge CI `31401045285`, Rust CI `31401052016`, and
CodeQL `31401039261` pass on that exact merge. The accepted slice adds exhaustive
observation through active exclusive mutable enum references and homogeneous discarded
`Void` Match results without adding enum extraction, reference escape, general
borrowing, lifetime, memory-safety, layout, or ABI claims.

Milestone 106 `CORE-085` is accepted public at candidate head
`f605a328c405a72766f44558c5615eb1e77f50ea`, with frozen implementation commit
`97d3b5deebd5bc0ac19c47760abe5110344718a9`. Bounded PR #12 merged through
protected master without head substitution as
`d0832c6f4442095703a5f040eacb539a4a3774a7`, tree
`96f8eb67b1cddfcee33839a86f84b5c90fa38e23`, with ordered parents old master then
the exact candidate. Post-merge CI `31395036300`, Rust CI `31395036291`, and CodeQL
`31395035620` pass on that exact merge. The accepted class permits multiple
non-escaping immutable `&E` aliases from an initialized mutable direct enum owner,
with exact lexical ends, CFG loan-state proof, post-loan assignment or mutable
replacement, and pinned native exit 85. This is bounded public acceptance, not a
release or a general borrowing, lifetime, memory-safety, layout, or ABI claim.

Milestone 105 `CORE-084` is accepted public at corrected candidate head
`0ebef3ac7b19cb14390368ffcc432fbba55815a8`, tree
`cbe48fe2207d699660aeca3c0ffc985e5f4e074f`, from verified base
`23474fb45e57bed5c80ceed81597060aee2663f5`. Its implementation commit is
`47b5142b312786975b53934cd943f6efd04e5575`, with tree
`e325ce0427cc7db0de70960c2ae9e1c0892ae015` and stable patch ID
`c62abff2165a996eba67ede77d53048b66c05958`. Corrective specimen commit
`980fc6e81ecf7b4e486d0cbfc4b25df3cf159b33`, tree
`c34ea237aa577d9ee37fdb653d3355c43d521fe7`, changes only the tracked packet
score so its native system exit is exactly 84. Bounded PR #10 carried four commits
across 27 files and merged through protected master without head substitution as
`ae0f090169b161b6c73537ff59eb0bdcbc5c7901`, with the candidate tree and ordered
parents old master then corrected candidate. The exact five-file acceptance record
then merged through bounded PR #11 as
`01a1bb65413253309dd5d74c1faddee8721bd7f2`, tree
`644ee11832f9c0c109ab3f53214a2d02f3ff2402`.

The accepted checkpoint admits immutable references only to the already admitted non-generic,
destructor-free enum class. Initialized immutable direct local enum owners may have
multiple non-escaping `&E` aliases, and non-entry internal functions may accept exact
immutable enum-reference parameters. The sole enum read is exhaustive
`match *identifier { ... }` with a CopyData or `Void` result; free enum dereference,
mutable-owner immutable loans, mutable-reference reads, transport/storage/escape, and
all new lifetime, drop, layout, ABI, FFI, safety, or stability semantics remain rejected.
Checked IR uses exact immutable enum owner-place and Match-read identities, and the
independent verifier ties schema, immutable reference provenance, read adjacency, and
dispatch use together while rejecting generic-load substitution. The focused target is
4/4 and the exact repository-root gate passes formatting, correctness-denying Clippy,
212 library tests, 32 binary tests, every integration target, and doc tests. The tracked
two-module specimen passes local public CLI check/build; this workstation lacks the
required LLVM 22 verifier, so native execution is not claimed locally. Candidate-head
CI `31362774499` and `31362777496`, Rust CI `31362777546`, and CodeQL
`31362775267` pass on the exact corrected head; stable/nightly Linux and pinned Windows
LLVM/Clang 22 execute exact native exit 84. Post-merge CI `31362984522`, Rust CI
`31362984536`, and CodeQL `31362984082` pass on the implementation merge. Acceptance-
record post-merge CI `31364094279`, Rust CI `31364094313`, and CodeQL `31364094135`
pass on exact accepted master. This is bounded public acceptance, not a release or
general memory-safety/reference/ABI claim.

Milestone 104 `CORE-083` is accepted public at candidate head
`43e9bfc7d07414737c8978c3fc3621b984a1146d`, implementation commit
`524b2c2a01f62764d307b19b6cd9a5c9786f2283`, implementation tree
`74a3709525f7f8195b79d2dfe79139e4de56667c`, and stable patch ID
`fa81a9df55a28d5f1d32e7e13a0459a45746b269`. Bounded PR #8 carried two commits
across 20 files from verified base `469cdc4ab5d968d42dec6af81823796e3db14f71`
and merged through protected master as `680bc6ca0bc6284e773083504ed24367b34fae73`
with tree `6d6a5a7a7c332e6ba7d6e64f82c177df599a5541`. Its parents are exactly that base
and accepted candidate head. Candidate-head CI `31291291046`, Rust CI `31291291039`,
and CodeQL `31291290190` pass; post-merge CI `31291435878`, Rust CI `31291435867`,
and CodeQL `31291435769` also pass on the exact merge.

One shared reference-pointee classifier now composes the existing recursive CopyData
universe with admitted destructor-free enum schemas for mutable use only. An
initialized mutable direct enum owner can be borrowed or locally reborrowed into an
exact sole `&mut E` parameter and replaced as a whole. Checked IR carries the exact
enum schema through the loan, parameter, write, and lexical end, while the independent
verifier rejects generic stores, enum loads through the reference, schema substitution,
consumed-owner borrows, bad loan identity, and malformed layouts. The focused target is
5/5; the exact repository-root gate passes formatting, correctness-denying Clippy, 211
library tests, 32 binary tests, every integration target, and doc tests. Stable/nightly
Linux and pinned Windows LLVM/Clang 22.1.8 verify the tracked composed specimen at exact
native exit 83. Immutable enum references, reads through enum references, escape or
reference results, aggregate enum storage, projections, partial moves, and new
lifetime/drop/ABI semantics remain rejected. This is bounded public acceptance, not a
release or general memory-safety/reference/ABI claim.

Milestone 103 `CORE-082` is accepted public at implementation commit
`d0312e809792448b6c8c0fc4159dd0f37dbc10ba`, tree
`516d9f1ac2cfb1406a2ae0cb8716c40869d316f9`, and stable patch ID
`a0aff6d2b730fc6499cc08b0d6e1d85d484aeb47`. Bounded PR #6 carried exact
candidate head `3edfa1398450ebbd077ddc645efb461ae718b27e` in two commits and
20 files, passed all nine exact-head checks, and merged through protected master as
`962bb49f1428a36b8ba7cf897778e4c0bab2ac09`. Post-merge CI
`31287892380`, Rust CI `31287892372`, and CodeQL `31287892204` pass on that
exact merge. Closed primitive `const NAME: TYPE = EXPR;` declarations now use one
shared evaluator and are substituted before checked IR with no storage, symbol,
layout, ABI, or general CTFE claim. The tracked multi-file specimen returns 81 through
pinned LLVM/Clang 22.1.8 public and manual native execution; generated LLVM SHA-256 is
`AD2DFA947E03AF257F717E0FF5B2E9AB04281B49CC18D0D1230A8B12414050C6`.

The governing next-task rule is now project-level payoff, not adjacent topology. The
roadmap checkpoint below selects a representative scalar conformance application as
the shortest credible Milestone 1 path; positive imports remain a high-leverage later
candidate only after lookup, namespace, visibility, collision, cycle, and cache
semantics are frozen. Bounded checkpoint PRs, deliberate hard-capability progress,
proportional evidence, and periodic composed source-to-native system gates remain
active. No new compiler capability may stack on a red or unpublished record.

Milestone 102 `CHECKPOINT-001` is accepted public and integrated through the corrected
solo-maintainer `CHECKPOINT-003` handoff. PR #4 merged the exact frozen head
`9b13feb2bf5159a9ca7d6902f97f0b280c78b471` with merge commit
`bf5f8a9625d9c25910195680213d2fe6b289d7e6`. Its parent order is old master then the
frozen integration head, and its tree is byte-for-byte the accepted tree
`6d5825a1a32c4674e59e6fd7a5953efe5c51c641`; the integration branch remains retained.
Master protection keeps strict app-bound checks, conversation resolution, administrator
enforcement, and force-push/deletion denial. Required approvals are zero because
`RobVanProd` is the only human contributor and GitHub rejects author self-approval.
Post-merge CI `31285477963`, Rust CI `31285477942`, and CodeQL `31285475312` pass on the
exact merge SHA, as does the local repository-root gate. This is an integration result,
not a release, completion, stability, safety, performance, or production-readiness claim.

Milestone 101 `CORE-081` is accepted public at exact commit
`aae33a1774ea558cc782aed6389fbff73419b5b4`, tree
`42de741f3d62e84c463462a10d061b7869c905eb`, and stable patch ID
`8591b1a6bf334acbc0a388b6fa8d43ec78df194d`. The exact red proved 35 duplicate
binary/library compiler modules; compiler phases now have one library-owned Rust
identity, the binary keeps only CLI services, resolver/IR representations remain
private, and no distinct test vanished. All nine exact-head checks pass. Both compiler
jobs record 207 library and 32 binary tests; pinned Linux/Windows lanes preserve exits
149/223/227/229. No source, semantic, LLVM, cache, backend, runtime, or ABI rule moves.

Milestone 100 `CORE-080` is accepted public at exact commit
`063953770ce92f00bae452f312c962c2996977bb`, tree
`5c33799270fdce9d28984ffa8aaf2cda7cf1404e`, and stable patch ID
`a8bcc38f684d615305a258399bc5318472e36be7`. The founding direct/aliased dotted
`import a.b [as c];` syntax retains a distinct AST identity and source location while
executable use fails closed before checked IR through one shared authority. All nine
exact-head checks pass; pinned Linux and Windows LLVM/Clang 22.1.8 preserve exits
149/223/227/229. No lookup, binding, alias, namespace, visibility, recursive graph,
cache, backend, runtime, or ABI rule follows.

Milestone 99 `CORE-079` is accepted public at exact commit
`5b1ec7340db72354542ab325a9f75cad398857c2`, tree
`930152ff617e104025fc512337b0b31b1c187c08`, and stable patch ID
`f89e01b1f9a2e15aa3fb7a45111b7321da8d4977`. One phase-neutral classifier joins
`Owned`/`Moved`/`MaybeMoved` loop headers and exits, semantic analysis and independent
checked admission recheck widened headers, and the verifier independently proves
cyclic consumption/repair. All nine exact-head checks pass. Stable/nightly Linux
preserve exits 149/223/227 and execute exit 229; the SHA-256-pinned Windows LLVM/Clang
22.1.8 lane externally/machine verifies, emits COFF, links through Clang/MSVC, and
returns 229 through public and manual native execution. No general ownership, memory-
safety, aggregate, drop/lifetime, ABI, module, runtime, or accelerator claim follows.

Milestone 98 `CORE-078` is accepted public at exact commit
`70f59fd72e96246b2ebefdf1ae53a9b7f3280cfe`, tree
`b7a2f41877ab812140248ecce10d3157bdab29ac`, and stable patch ID
`a85fca8b087a98a89c81cb6c2eb35de67a249f9e`. The official 862,053,924-byte
`clang+llvm-22.1.8-x86_64-pc-windows-msvc.tar.xz` archive is pinned by SHA-256
`d96c2cc1736f4eb7fa43cb9bbdf56d93551a9ae0a9aadb9c99c3c3b2b712a234`; exact
`opt`, `llvm-as`, `llc`, and `clang` 22.1.8 prove the existing MSVC target/layout,
invalid-build artifact hygiene, external and machine verification, COFF object
generation, Clang/MSVC linking, public `run`, manual execution, and exit 227. All nine
exact-head checks pass, while stable/nightly Linux preserve exits 149/223/227. The two
installer-based predecessors remain rejected evidence. No general Windows, stable ABI,
packaging, release, safety, performance, or accelerator claim follows.

Milestone 97 `CORE-077` is accepted public at exact commit
`a93d8d38c5f2a2499ce036f659c13cb2ec4fefcb`, tree
`2efbeed06e0a303aa5c07d3352d7c536fcd92dcd`, and stable patch ID
`d64092de918ad990b79d94f6193607783e3acc55`. A direct mutable admitted
destructor-free enum may be consumed and reinitialized inside `while`, fixed-array
`for`, or `loop` only when every reachable condition/iterable edge, fallthrough or
`continue` backedge, and `break` exit restores exact `Owned`. Semantic analysis and
independent checked admission feed one phase-neutral edge classifier; verifier CFG
controls reject missing, bypassed, one-path, generic-store, and wrong-schema repair.
All eight exact-head checks pass in push CI `31085620279`, PR CI `31085622212`, Rust
`31085622180`, CodeQL `31085620081`, and aggregate check `92564358585`. Stable and
nightly LLVM/Clang 22.1.8 preserve exits 149/223 and execute exact exit 227. No general
loop fixed point, borrow checker, drop/lifetime, stable ABI, import, accelerator,
release, safety, or merge claim follows.

Milestone 96 `CORE-076` is accepted public at exact implementation commit
`aefeb2d81fb5374e7373a4819f3c92f83a95eb35`, tree
`34e58b2943d6c01efd245753f4b3ca18a338d595`, and stable patch ID
`ef7bd0a42de1bda040a4e435fb9c51e0765160b4`. One
shared result classifier now admits every identical exact value in the existing
recursive finite CopyData universe—primitives, fixed arrays including zero length,
arity-two-or-more tuples, and finite acyclic named structs—or the separately constrained
owned-enum class from CORE-074/075. Semantic inference and independent checked
admission consume the same CopyData authority; enum-bearing function signatures no
longer carry a tuple-only topology exception.

All admitted Match results now lower through one
`CheckedMatchResultPlaceAlloca { result_type, dispatch_schema }`, exact typed arm
assignments, all-path initialization proof, one merged load, and the existing private
LLVM type. Generic stores, wrong types or values, missing/repeated writes, bypass,
premature/duplicate loads, and enum ownership fabrication remain fail closed. The
tracked two-module specimen composes arrays, recursive tuples, structs, nested Match,
calls, returns, bindings, reassignment, projection/indexing, and references and is
pinned for LLVM/Clang 22 native exit 223. All eight exact-head checks pass in push CI
`31081503050`, PR CI `31081506213`, Rust `31081506169`, CodeQL `31081503119`,
and aggregate check `92551229284`. Stable and nightly pinned LLVM/Clang 22.1.8
preserve the older unit-enum exit 149 specimen and execute the unified result specimen
at exact exit 223. Generic stores, public layout/ABI, runtime behavior, and new CopyData
ownership semantics remain excluded.

Milestone 95 `CORE-075` is accepted public at exact implementation commit
`50a3e03d0bdbc0e7deddde747bc19df0621c1257`, tree
`c31e261a32072f7eca473d940641bbbfef3b6b21`, and stable patch ID
`395de4d78694be56b45a310b87df1f98568217eb`. All eight exact-head checks pass;
stable and nightly pinned LLVM/Clang 22.1.8 externally verify, machine-verify,
object-lower, link, and execute exact exit 211. Accepted CORE-076 generalizes the
checked result-place identity and admits exact CopyData aggregate results without
weakening CORE-075's direct-owner rules.

Milestone 93 `CORE-073` is accepted public at exact implementation commit
`ef2eaa380cccf32e21df8938479e30bcd467cdaa`, tree
`88f3b0c0d542bcce77e2b53de0c3bf737fb6f629`, and stable patch ID
`0714282415bb51f11fedb6dada583dcb8d136f6d`. One shared assignment authority
classifies ordinary enum replacement and exact acyclic `Moved`/`MaybeMoved`
reinitialization, while independent checked verification proves predecessor
consumption and the checked write kill. All eight exact-head public checks pass;
stable/nightly pinned LLVM/Clang 22.1.8 externally and machine verify, object-lower,
link, and execute exact exit 199. Every loop-contained reinitialization, partial move/
projection, enum borrow/storage expansion, drop/lifetime rule, and general CFG fixed
point remains unsupported.

Milestone 92 `CORE-072` is accepted public at exact implementation commit
`4693f11d18135d76b5a7ec16b385563c07272955`, tree
`42d6262bdd82e9934f47db8a42f103aa18b6448c`, and stable patch ID
`5104478eec2ca922fa70200720d3a3bb1ed2fc98`. Exact raw Unicode scalars and the
frozen `\\n`, `\\r`, `\\t`, `\\\\`, `\\'`, `\\\"`,
`\\0`, and `\\xHH` escapes retain character identity from token and AST through
`Ty::Char`, `LogicalType::Char`, `ImmChar`, independent checked verification, and
private `i32` LLVM. One shared primitive authority now owns source/type/logical-type,
CopyData, predicate, physical-type, zero, and alignment facts for all admitted scalar
kinds. Equality and inequality are executable; arithmetic, ordering, casts, printing,
literal-pattern execution, runtime strings, and public ABI remain unsupported.

The complete existing recursive CopyData transport surface carries `char` through
bindings and replacement, non-escaping immutable/mutable references, calls/results,
arrays including zero length, tuples, named structs, unit/unary/multi-field enums,
identifier-bound Match, control flow, flattened direct modules, library compilation,
and public `check`/`build`/`run`. The red target first stopped at strict lexing; a
prepublication completeness audit also exposed the old three-scalar Match-result
table. The implemented matrix passes 9/9. The full surface passes at 190/190 library and 196/196
binary tests plus every integration and benchmark target; formatting, checking,
correctness Clippy, docs, verifier corruption controls, and the exact repository-root
gate pass. Official LLVM/Clang 22.1.8 externally verifies, machine-verifies,
object-lowers, links, and executes the two-file program at exact exit 197. All eight
exact-head public checks pass, including independent stable/nightly pinned
LLVM/Clang 22.1.8 exit-197 lanes.

Milestone 91 `CORE-071` is accepted public at exact implementation commit
`5fc15622188e4e80a319e4c7d6c4bab17a7c8366`, tree
`ed1f33ede282d01bcd975d83d1e1197424403fef`, and stable patch ID
`e5b9d98b4f9c1a1d47ddf0dbe227f0feec78dc55`. Rust-like direct, aliased, and
terminal-glob `use` declarations retain their syntax and exact keyword location but
reject through one shared diagnostic before checked IR. All eight exact-head public
checks pass and pinned stable/nightly LLVM/Clang 22.1.8 preserve native exit 193.
This is containment only: it adds no founding dotted `import`, lookup, namespace,
visibility, recursive graph, cache, backend, or runtime semantics.

Milestone 90 `CORE-070` is accepted public at exact implementation commit
`365c28a3e4fdd306ec4c1a4837545ddbe3dac6a3`, tree
`2e1146cf0c4f7468de0c8fa0dde85a13cdd79a21`, and stable patch ID
`1263a11601e3cb7f9f776e4e154f3de158feaa6d`. Public `compile_file(path, options)`
adds the bounded file-aware checked library route over the accepted direct-module
collector. All eight exact-head public checks pass; stable and nightly pinned
LLVM/Clang 22.1.8 preserve the composed exact native exit 193. No broader module,
CLI, cache, option, external-verifier, runtime, release, or stability claim follows.

Milestone 89 `ARCH-002` is accepted public at exact implementation commit
`aca3fe21ece4a7f90de0b41b5e336c15ac589505`, tree
`3c5466e8d6821b8443ecba919bde2ad568923355`, and stable patch ID
`cec753bde549b9ea1fc4a3aa7e820d754f7d8798`. It replaces nested binding-annotation
shape rules with one normalized leaf/wrapper-path classifier shared by semantic
analysis and checked admission without changing any supported, rejected, or
quarantined behavior. All eight public checks pass, and stable/nightly pinned
LLVM/Clang 22.1.8 independently preserve the composed exact native exit 193. No
language feature or matrix cell moved.

Milestone 88 `CORE-069` is accepted public at exact implementation commit
`99ea287843bc0c1262045d31a60f18b03fa0558f`, tree
`175b44d7ea2e10615553d4cd062ad13fd1e2e6e0`, and stable patch ID
`143a4cf9669e2c4168ba899d7edebeea7e1cd297`. It admits positional multi-field
variants for non-generic owned enums when every field is in the complete existing
recursive finite CopyData class, while preserving unit/unary identities and whole-enum
ownership. All eight public checks pass on the exact candidate. Stable and nightly
LLVM/Clang 22.1.8 independently externally verify, machine-verify, object-lower, link,
and execute the tracked direct-module program with exact exit 193. PR #4 remains open,
draft, and unmerged. Named-field/generic variants, Option/Result admission, wildcard/
guard/nested Match, enum aggregate storage/borrowing/projection, partial ownership,
new lifetime/drop/CFG semantics, stable ABI/FFI, accelerators, release, performance,
safety, and merge remain excluded.

Milestone 87 `CORE-068` has an accepted public implementation at exact commit
`55b61c31fc6dd822097daa5d4f371d04ec0d6264`, tree
`81c2be5d1ee6abdd7382c8674f68f553613efd6f`, and stable patch ID
`155bbfa5310e1289fccb82c339108d8a44bdbfca`; its accepted records head is
`9175add20896a3ac79d99acec053b82cd12b48a1` and its all-eight closure passes. One
shared function-call classifier now owns named-target availability, exact parameter
and result contracts, argument types,
value-versus-discarded use, and preserved context for both semantic inference paths
and checked admission/lowering. Missing or unsupported signatures no longer become
`Int`, and the trusted checked path cannot emit a call or choose a result layout
without an accepted exact contract. Legacy unchecked lowering remains explicitly
quarantined from `try_generate_ir`.

The supported slice preserves exact nongeneric functions over `Void`, scalar types,
recursive finite CopyData, admitted owned enums, and established immutable/mutable
reference parameters. It does not add overloading, conversion, generic/trait or
closure calls, reference results, callable ABI, layout, lifetime, or dispatch
semantics. Red-first evidence reproduced the semantic, legacy-validator, raw-admission,
and trusted-lowering false-successes. The classifier units, exhaustive contract target,
affected compatibility ring, formatting, all-target/all-feature checks, correctness
Clippy, docs, and exact root gate pass at 185/185 library tests. The tracked two-file
system specimen resolves its direct module, emits deterministic Windows LLVM, links
with Visual Studio Clang 19.1.5, and executes exact exit 181. This host accurately
reports `InternalOnly` because LLVM 22 is absent. All eight candidate-head checks pass.
Pinned stable LLVM/Clang 22.1.8 rejects the known-invalid fixture, externally verifies,
machine-verifies, object-lowers, privately non-PIE links, and executes exact exit 181;
nightly independently repeats external verification and exit 181. The first Rust CI
attempt hit an unrelated transient Linux `ETXTBSY` in an existing fake-verifier test;
the unchanged exact candidate passed both stable and nightly on attempt 2.

Milestone 86 `CORE-067` is accepted public at exact implementation commit
`e7525bf039339909c8f4f5cc68262fdf498079e0` and additive records head
`41eb0ee61eec53964ee21e7cb5cc5eabbefcf656`. One shared intrinsic-method classifier
closes fabricated result and lowering fallbacks while admitting only the exact
recursive CopyData fixed-array and established static String/Array/Vec query class.
Both implementation-head and records-head all-eight public sets pass; pinned stable
LLVM/Clang 22.1.8 executes exact exit 167 and nightly repeats it.

Milestone 85 `CORE-066` is accepted public at exact implementation commit
`e40804ea86888b38548fd5bf42926be2be7eb5ed`, tree
`6cea8bbf63aa7aafb43fbb25152dd860f6684aae`, and stable patch ID
`7c4e6ac77db90dc7c83048922382903958c09632`. Starting from accepted records head
`b55407836a3d76b05c7c8b8b2514fd4354e66b2b`, it certifies fresh per-iteration owners
of every already-admitted enum schema across
`while`, checked fixed-array `for`, and `loop` statement CFGs. Exact constructor and
function-result definitions reset verifier ownership only when every path to a
consumption executes that definition. Pre-loop owners still may not change across a
condition or reachable backedge, and moved-target reinitialization and break-exit
ownership joins remain unsupported.

The red-first target exposed that admitted `for` `continue` jumped directly to its
header and skipped the index increment. One shared statement-loop label allocator and
one shared `for` iteration tail now route `continue` through an explicit increment
block for both array and legacy lowering, while `while` and `loop` retain header
targets. The exhaustive target, new fresh-result/place fixed-point corruption controls,
11 affected integration targets, formatting, and a serialized exact repository-root
gate pass. The first unconstrained Windows root attempt exhausted concurrent linker/
rustc memory; the exact retry with one Cargo build job passed and is the only gate
evidence. The tracked direct-module example builds deterministically, and local Visual
Studio Clang 19.1.5 executes exact exit 149. All eight candidate-head public checks
pass. Stable job `92463336662` installs LLVM/Clang 22.1.8, rejects the invalid fixture,
externally verifies, machine-verifies, object-lowers, explicitly links, and executes
exact exit 149; nightly job `92463336701` independently repeats exit 149. A records-
only acceptance closure and its fresh all-eight checks are the next administrative
actions; no merge, release, ABI, general loop ownership, or safety claim is authorized.

Milestone 84 `CORE-065` is accepted public at exact implementation commit
`f4daeea6d7b032e686b4c7d184fe80ef38076665`, tree
`7cd4ec6da2d9ce44f63741222a5b128396358bfe`, and stable patch ID
`708c1a6cab096f89e76577212a241554225897a2`. It adds exact acyclic conditional
ownership joins over the enum class already accepted by CORE-063/064. One shared
classifier gives every sibling `if` arm the same entry ownership, excludes definitely
returning arms from the merge, and joins reachable states as `Owned`, `Moved`, or
`MaybeMoved`. Later use of `MaybeMoved` fails deterministically. Conditional or direct
enum ownership changes that reach a loop backedge remain rejected because no fixed-point
loop semantics are claimed.

Semantic analysis and checked admission consume the same classifier. The independent
checked-IR verifier follows exact enum result/place identities through calls, returns,
Match dispatch, mutable-place initialization, replacement, and CFG predecessor unions;
it accepts one consumption in each mutually exclusive arm and rejects serial,
post-partial-merge, and cyclic double consumption. The exhaustive target, corruption
controls, affected compatibility ring, 182/182 library tests, 188/188 binary tests,
formatting, all-target/all-feature checking, correctness Clippy, docs, and the exact
repository-root gate pass. All eight public checks pass on the exact implementation.
Stable job `92454648190` installs LLVM/Clang 22.1.8, proves the known-invalid control,
externally and machine verifies, object-lowers, explicitly links the private executable,
and observes exact native exit 137; nightly job `92454648318` independently repeats
exit 137. No general CFG ownership, loop fixed point, break/continue transport, enum
borrowing or aggregate storage, partial moves, drop/lifetimes, stable ABI/FFI, release,
or stability claim follows.

Milestone 83 `CORE-064` is accepted public at exact implementation commit
`79aed71371e192a07218d437e882a863653b6826`, tree
`ac80c49aca3fb875c44d132f930567e95d81f698`, and stable patch ID
`1bb2c9c19f6d427122f83bffc59d3f18f0a5b3e4`. It admits exact whole-owner
reassignment for the already accepted
unit-or-unary-`CopyData` enum class. One shared owned-place classifier now covers
recursive `CopyData` and admitted enum places across semantics and checked admission;
the generalized checked allocation/assignment identities retain exact enum schema,
and verified private LLVM uses exact typed loads and stores. Direct distinct-local
replacement moves the source, direct self-replacement fails closed, and only the
`CopyData` subset remains borrowable.

The focused exhaustive target, verifier corruption controls, affected compatibility
ring, 180/180 library tests, 186/186 binary tests, formatting,
all-target/all-feature checking, correctness Clippy, docs, and the exact repository-root
gate pass locally. The tracked two-module system specimen is pinned to native exit 131.
All eight public checks pass through CodeQL `31026627490`, PR CI `31026630294`,
push CI `31026630855`, and PR Rust CI `31026630282`. Stable job `92376666972`
installs LLVM/Clang 22.1.8, rejects the known-invalid verifier control, externally
verifies, machine-verifies, object-lowers, explicitly links the private non-PIE
executable, and observes exact native exit 131; nightly job `92376666842` repeats exit
131. No enum
projection, borrowing, array/struct storage, partial move, new CFG ownership, drop,
lifetime, stable layout/ABI/FFI, accelerator, performance, release, or stability claim
follows.

Milestone 82 `CORE-063` is accepted public at exact implementation commit
`2a5c3c58192dc65116c436d6ae76da5829eeba52`, tree
`8a5cef6b14214e76349a41f6997d5fa19595858f`, and stable patch ID
`276af069807b6f59c233a2f281c1b0d0b8c899b8`, with verified native-link repair head
`bebd0b6a87108219497187a5952688c95c397158`. It extends the
accepted unit-or-unary-scalar owned-enum class to exactly one recursive `CopyData`
payload per payload-bearing variant by delegating annotation admission to the accepted
`StructRegistry` classifier. Unit and scalar-only enum layouts retain their accepted
private forms; any schema containing an aggregate payload lowers to a private typed
product with an `i32` tag and one exact typed lane per payload-bearing variant.

Construction, exhaustive identifier-bound `Match`, arm-local CopyData projections,
whole-enum moves, internal parameters/results/calls, and flattened direct modules pass
through exact checked enum schemas and independent verifier controls. The exhaustive
target covers arrays, tuples, finite acyclic Copy structs, mixed unit/scalar/aggregate
variants, unsupported recursive leaves/topologies, malformed schemas, and artifact
hygiene. Rustfmt, all-target/all-feature checking, correctness Clippy, docs, 179/179
library tests, 185/185 binary tests, and the exact repository-root `./tools/test.sh`
gate pass. All eight public checks pass through CodeQL `31022757247`, push CI
`31022756615`, PR CI `31022760915`, and PR Rust CI `31022761529`. Stable job
`92363420145` installs LLVM/Clang 22.1.8, proves the known-invalid verifier control,
externally verifies, machine-verifies, object-lowers, explicitly links the private
non-PIE executable, and records exact native exit 113; nightly job `92363420286`
repeats exit 113. No stable layout,
ABI/FFI, general enum storage/borrowing/mutation, aggregate Match result, nested
destructuring, generic enum, new CFG ownership, closure, accelerator, performance,
release, or stability claim follows.

Milestone 81 `CORE-062` is accepted public at exact implementation commit
`e62fd7470d8cb929d57d0c063815d7a99005d768`, tree
`d2aff21a54c42d1ce649ef6668d50a4908315738`, and stable patch ID
`458feb5ebc1355d83793084009e5ea7895a22129`. One shared `StructRegistry` classifier
resolves both parsed annotations and semantic types into the least-fixed-point grammar
`Int | Float | Bool | [CopyData; N] | tuple(CopyData, ...) | finite acyclic named
struct(CopyData fields)`. It replaces executable scalar/flat-tuple/numeric-array/
Copy-struct-array topology whitelists across semantic analysis, checked admission,
Copy-place ownership operations, function transport, independent verification, and
private LLVM lowering.

The accepted slice admits Bool arrays, nested arrays, arrays of tuples, tuples containing
arrays/tuples/structs, and structs containing every admitted aggregate constructor.
Exact recursive schema survives inferred and annotated bindings, whole Copy aliases,
direct owner reassignment, immutable/mutable whole-place references, internal calls and
returns, forwarding, terminating recursion, dynamic fixed-array indices, chained value
projection, and flattened direct modules. The verifier independently validates finite
recursive schemas and corruption controls; LLVM uses exact literal structs, fixed
arrays, and private identified named structs with no fallback `i32`, pointer/integer
conversion, or unrelated bitcast. The focused exhaustive target and local native exit
109 pass. The exact repository-root gate passes rustfmt, all-target/all-feature
correctness Clippy, 178/178 library tests, 184/184 binary tests, every integration and
claim target, the 22-active/16-quarantined Phase 5 split, and doc tests. All eight public
checks pass through CodeQL run `31017349668`, push CI `31017352912`, PR Rust CI
`31017357342`, and PR CI `31017358299`. Stable job `92344809072` installs LLVM/Clang
22.1.8, rejects the known-invalid verifier control, externally verifies LLVM,
machine-verifies, object-lowers, links, and executes exact native exit 109.

Unit/unary tuples; String, references as stored data, closures, enums, generics, traits,
Option/Result/collections, empty/duplicate/unresolved/generic/cyclic structs, dynamic
arrays/slices, aggregate comparison/destructuring, projected borrow/write, contextual
coercion, stable ABI/FFI/layout, memory-safety, accelerator, performance, release, and
stability claims remain excluded. Immutable references remain copyable values for the
existing ownership model, but references are not recursively stored `CopyData`.

Milestone 80 `CORE-061` is accepted public at exact implementation commit
`de6fc0d5c503d2dcb03944d58312a130bac1ba05`, tree
`9ad23f5ad5cff17d3b69fdef31b9a4c7289ade42`, and stable patch ID
`e358319e7402f345ca414cc57bb18c0414b81cd4`. All eight public checks pass. The pinned
LLVM/Clang 22 lane externally verifies LLVM, machine-verifies, object-lowers, links,
and executes exact native exit 83 with 175/175 library and 181/181 binary tests.
CORE-061 admits direct whole-owner reassignment over its then-frozen Copy-data universe
and keeps closures parsed-only and fail-closed before checked IR; it does not establish
projected assignment/borrowing, general lifetimes, stable ABI/FFI, memory safety, or
positive closure semantics.

Milestone 79 `CORE-060` is accepted public at exact implementation commit
`7c7a47a471460dfe2276ea63cc4964fa59ad54be`, tree
`e9863de79a69766114020060a138c94357005351`, and stable patch ID
`ec2c33060e33ca6e52894fa1a18daf5b5d9c6ba7`. All eight public checks pass. Stable job
`92301482760` uses LLVM/Clang 22.1.8 for external verification, machine verification,
object lowering, linking, and exact native exit 59, with 174/174 library and 180/180
binary tests. CORE-060 accepts exclusive whole-place mutable references over admitted
Copy data without establishing projected origins/writes, general lifetimes, stable
ABI/FFI, or memory safety.

Milestone 78 `CORE-059` is accepted public at exact implementation commit
`5a78eb5d670045277532cc3cdc9a6144b1449895`, tree
`03fbdd58e836532dc8a4f95a0bb3c0402b1e5f1c`, and stable patch ID
`62a23bef479f22d3d9da22fc4bf753c7610c3e77`. All eight public checks pass. Stable
job `92291545518` uses LLVM/Clang 22.1.8 for external verification, machine
verification, object lowering, linking, and exact native exit 37, with 173/173
library and 179/179 binary tests. CORE-059 accepts immutable references over exact
admitted Copy-data places without establishing mutable aggregate references, general
lifetimes, stable ABI/FFI, or memory safety.

Milestone 77 `CORE-058` is accepted public at exact implementation commit
`421a0a9fe6e4df1f35f703a58e50ec41bea9e148`, tree
`a2c486de7519b4c71631651e11152e17eb4ebf0b`, and stable patch ID
`58ebaf3c42cfca1ebc0a3125b4ff01ad946e29a0`. All eight public checks pass. Stable
job `92281869112` uses LLVM/Clang 22.1.8 for external verification, machine
verification, object lowering, linking, and exact native exit 23, with 171/171
library and 177/177 binary tests. CORE-058 accepts the bounded flat immutable
Copy-scalar tuple product and does not establish general tuple semantics, stable
layout/ABI/FFI, drop, or aggregate-safety claims.

Milestone 76 `CORE-057` is accepted public at exact implementation commit
`7c108ff0ae0e9686209378deec5ce1de61bff17b`, tree
`ba4d4987cdc2986e5ce4f7ed252b2a25b9602ad1`, and stable patch ID
`e699e8cae16a708d745ac400548d07a622ed71c7`. All eight public checks pass, and the
pinned LLVM/Clang 22 lane externally verifies, machine-verifies, object-lowers, links,
and records exact native exit 253 with 169/169 library and 175/175 binary tests.
CORE-057 accepts call-scoped child reborrows from an initialized CORE-055 local mutable
scalar alias or the current mutable-reference parameter into the exact CORE-056 sole
mutable-reference signature. It does not establish general reborrowing, NLL, lifetimes,
stable ABI/FFI, or memory safety.

Milestone 75 `CORE-056` is accepted public at exact implementation commit
`e3ff1658039f8b9e20f18981c3d6198a07e79e92`, tree
`4efca0a523ae60d0d3020f925e0567f430dad9dd`, and stable patch ID
`77377ea77150931b709898d2fdf2bbcd9713c1c1`. All eight public checks pass. Stable
job `92259593558` uses LLVM/Clang 22.1.8 for external verification, machine
verification, object lowering, linking, and exact native exit 251, with 167/167
library and 173/173 binary tests. CORE-056 accepts exactly one direct call-scoped
mutable scalar owner loan into a sole mutable-reference parameter; it does not
establish stored-alias transport, reborrowing, general lifetimes, stable ABI/FFI, or a
memory-safety claim.

Milestone 74 `CORE-055` is accepted public at exact implementation commit
`1f6ea726ad87f079592d136cb374ff6481d4acec`, tree
`a3dd566a80b8555c6dcf417a0528fb13d75a2380`, and stable patch ID
`2c9f030dd64d4ec86835a1f9ed87322a96f3fcc7`. All eight public checks pass. Stable
job `92251942540` uses LLVM/Clang 22.1.8 for external verification, machine
verification, object lowering, linking, and exact native exit 239, with 166/166
library and 172/172 binary tests. CORE-055 accepts one non-escaping non-`Copy` local
mutable alias to an initialized owned scalar with checked reads, writes, and lexical
release; it does not establish general mutable references, NLL, lifetimes, drop,
stable ABI/FFI, or a memory-safety claim.

Milestone 73 `CORE-054` is accepted public at exact implementation commit
`6ef3e44f8c7910815031c12e880ac874141cef5c`, tree
`b6fe360fa42dfefef48492423a481da930279c8f`, and stable patch ID
`7cfa95a31f53381e4bc373ebc07d09d76a0d76fc`. All eight public checks pass. Stable
run `30986603008`, job `92242692711`, uses LLVM/Clang 22.1.8 for external
verification, machine verification, object lowering, linking, and exact native exit
227, with 165/165 library and 171/171 binary tests. CORE-054 accepts explicit exact
`Int`/`Float`/`Bool` reassignment of initialized owned local `let mut` bindings across
real branch and loop CFG without assignment expressions, compound syntax, aggregate
assignment, mutable-reference semantics, NLL, drop, stable ABI, or safety claims.

Milestone 72 `CORE-053` is accepted public at exact implementation commit
`b4aec4a01312088807750b0e40150cee87dc2131`, tree
`197e3b9ee615d32da569d55740891a14bcaced27`, and stable patch ID
`a78d4d38c0bf8266b1f724d69c5ff97d28d2c5d0`. All eight public checks pass. Stable
job `92235191630` uses LLVM/Clang 22.1.8 for external verification, machine
verification, object lowering, linking, and exact native exit 211, with 163/163
library and 169/169 binary tests. CORE-053 accepts non-escaping immutable
`Int`/`Float`/`Bool` references across exact internal parameters and calls without
reference results, mutable references, stable pointer ABI/FFI, lifetime inference,
NLL, drop, or a memory-safety claim.

Milestone 71 `CORE-052` is accepted public at exact implementation commit
`93a4a29e0b50f8d16ce6e2f845306b4ffcb37738`, tree
`eefd479e97754f1f069b67c640c2c27d179e28fe`, and stable patch ID
`8b0d7132e75ca8010fee3a39da021b320383565e`. All eight public checks pass. Stable job
`92227409386` uses LLVM/Clang 22.1.8 for external verification, machine verification,
object lowering, linking, and exact native exit 197, with 162/162 library and 168/168
binary tests. CORE-052 accepts whole-value internal transport of every unit or unary
scalar-payload enum schema admitted by the shared registry without stable layout,
calling convention, ABI, FFI, or general CFG ownership.

Milestone 70 `CORE-051` is accepted public at exact implementation commit
`babb1cd543fb36e13ec16458889f336ad5549a49`, tree
`6b8382ed0370c67994ee519a892f149c3ffe4825`, and stable patch ID
`2aaf5bee97f294f90c9494b364267deb250601b8`. All eight public checks pass. Stable job
`92223344697` uses LLVM/Clang 22.1.8 for external verification, machine verification,
object lowering, linking, and exact native exit 181, and passes 162/162 library plus
168/168 binary tests. CORE-051 accepts local construction and exhaustive bound Match
for unit/unary-scalar payload schemas without stable layout/ABI/FFI or broader payload
topologies.

Milestone 69 `CORE-050` is accepted public at exact implementation commit
`13f000358bdab33a2a8f5618bdbe80ffc50a1ed9`, tree
`e0228d7f0b056137abe1cc29e8078668ec0872fd`, and stable patch ID
`ee4eb0b8efc4847e30091d0293eed746b40851fa`. All eight public checks pass. Stable
job `92215771782` uses LLVM/Clang 22.1.8 for external verification, machine
verification, object lowering, linking, and exact native exit 173, and passes 161/161
library plus 167/167 binary tests. CORE-050 accepts only exact internal owned transport
for the unit enums established by CORE-049; it creates no public integer identity,
layout, ABI, or FFI contract.

Milestone 68 `CORE-049` is accepted public for explicit exhaustive matches over owned
local unit enums. The admitted definition is exactly one unique top-level non-generic,
nonempty enum containing unique unit variants and no same-name struct. Exact
payload-free constructors may initialize or move through immutable local bindings.
Matching an identifier consumes it because unit enums are not `Copy`; one shared
registry/classifier owns definition, constructor, annotation, exhaustive-arm,
uniform-result, execution-context, and nested consumed-scrutinee decisions across
semantic analysis and checked admission.

Supported matches contain exactly one payload-free `Enum::Variant` arm per declared
variant in any order and return one exact scalar type: `Int`, `Float`, or `Bool`.
Nested matches and already-admitted scalar parents execute only the selected arm.
Checked IR retains a distinct schema-bearing `LogicalType::Enum`, exact variant result,
and exhaustive dispatch terminator. Independent verification rejects malformed,
conflicting, incomplete, undefined, non-dominating, colliding, or transport-leaking
enum IR before LLVM. Verified LLVM uses an internal `i32` and `switch` without making
the enum a source `Int` or establishing stable layout/ABI.

Exact implementation `b38a6b0927c747909918b5ebf3c0f6b58d0727dd`, tree
`80829d3a74ddf2b6edfa247b75205b0a0ec799cc`, and stable patch ID
`c22f9210b9756645022be636cb98d24678d5a60f` pass all eight public checks. Push CI
`30975499818`, PR CI `30975502408`, Rust CI `30975502412`, CodeQL `30975500460`,
and aggregate `92208615520` are green. Stable job `92208529644` uses LLVM/Clang
22.1.8, resolves `signals`, reports `ExternalVerified` through `opt-22`,
machine-verifies and object-lowers with `llc-22`, links with `clang-22`, records exact
native exit 149, and passes 160/160 library plus 166/166 binary tests. The local host
truthfully remains `InternalOnly` because LLVM 22 is absent.

Payload/generic/mixed enums, Option/Result Match, wildcard/binding/guard patterns,
enum parameters/results/arrays/struct fields/references, mutation, borrowing, public
discriminants, stable layout/ABI/FFI, heap/drop semantics, accelerator execution,
performance, release, and stability claims remain excluded. Existing unsupported
Match classes retain their established fail-closed boundary and diagnostic precedence.

Milestone 67 `CORE-048` is accepted public for non-escaping immutable
references to existing local `Int`, `Float`, and `Bool` places. One shared classifier
now admits only `&x` for an initialized scalar identifier, exact inferred or annotated
local aliases, copied aliases, and scalar dereference. It explicitly rejects mutable
references, temporary/non-identifier origins, and non-scalar pointees while preserving
the existing behavior-neutral annotation cases that do not initialize a reference.
Reference parameters/results, aggregate storage, mutation, assignment, NLL, drop, and
general provenance remain outside the class.

Checked IR carries each admitted borrow as a fresh `CheckedImmutableBorrow` alias
place with exact scalar pointee metadata. The verifier independently rejects malformed,
undefined, non-dominating, duplicate, colliding, unsupported, or type-mismatched alias
places before LLVM. Verified lowering emits a zero-offset typed pointer derivation and
exact `double`/`i1` loads; the deprecated raw path does not activate the checked
instruction. The exhaustive focused target, verifier corruption controls, Phase 5,
frontend, binding-characterization, and checked-IR suites pass locally. The two-file
direct-module example checked-builds and the stable/nightly workflow is prepared for
pinned LLVM/Clang 22 verification, machine verification, object lowering, linking,
and exact native exit 127. The exact repository-root gate passes 159 library and 165
binary tests plus every formatting, correctness-Clippy, integration, and doc gate.
The local CLI reports `InternalOnly` because no LLVM 22 verifier is installed, while
the generated composed LLVM retains typed `double`/`i1` alias derivations and loads.
Exact implementation commit `98c21b9012a5d6581c31c67a0378f20363e0688d`, tree
`c222f99545270628686cae0524d92464a0db7848`, and stable patch ID
`c21aeefc5852d78a55aa1003fcd4363087e713c1` pass all eight public checks. Push CI
`30973047024`, PR CI `30973049411`, Rust CI `30973049412`, CodeQL
`30973047727`, and aggregate `92201382503` are green. Stable job `92201296160`
uses LLVM/Clang 22.1.8, resolves the direct module, externally verifies,
machine-verifies, object-lowers, links, and records exact native exit 127 plus 159/159
library and 165/165 binary passes. CORE-048 makes no general memory-safety claim.

Milestone 66 `CORE-047` is accepted public at exact implementation commit
`a1dcc3fbef3ce0e4750a1476b348940a966bf609`, tree
`15cf5d3451e1e02576c506d0bb4df4e3a62ab07c`, and stable patch ID
`2959bdc7d39ebe4a3d5e390f469fa9673033f9b6`. It replaces the scalar-only struct
definition decision with one recursive, memoized graph classifier. Unique,
non-generic, nonempty top-level definitions are admitted when their field graph is
acyclic and every field is an admitted scalar, another admitted named struct, or a
flat fixed numeric/struct array. Forward references and arbitrary finite named depth
are supported; ambiguous, unknown, non-Copy, empty, generic, direct-nested-array,
Bool-array, self-cycle, mutual-cycle, zero-array-mediated-cycle, and dependent
definitions remain fail-closed before IR.

The candidate carries exact recursive `Ty` and `LogicalType` contracts through
construction, contextual empty fields, independent Copy aliases, chained projection,
array length/index/iteration through fields, internal parameters/results, and flat
arrays of the newly admitted structs. Aggregate field children are evaluated once in
written order, loaded as whole values, and stored by declaration index. Semantic
preflight and checked admission no longer maintain enumerated receiver-topology lists;
both recursively type the receiver and consume the same registry decision. Checked IR
and LLVM preserve exact named/array schemas without claiming stable layout or ABI.

The exhaustive CORE-047 target, graph-classifier unit, recursive checked-IR positive/
corruption controls, and adjacent CORE-043 through CORE-046 suites pass locally. The
exact repository-root gate is formatting and correctness-Clippy clean and passes 157
library tests, 163 binary tests, every integration target, and doc tests. The tracked
direct-module example checked-builds into typed LLVM and the stable/nightly workflow
is prepared to run pinned LLVM/Clang 22 verification, machine verification, object
lowering, linking, and exact native exit 107. Push CI `30970850067`, PR CI
`30970852129`, Rust CI `30970852144`, CodeQL `30970850979`, and aggregate check
`92194686649` pass all eight checks. Stable Linux job `92194611441` uses pinned
LLVM/Clang 22.1.8, externally verifies, machine-verifies, object-lowers, links, and
records exact native exit 107. No release, stable ABI/layout, general ownership,
accelerator, or performance claim is made.

CORE-046 remains accepted through exact public implementation commit
`056ca334df08176dafac815c1df78f3e90ed660a` and records head
`77c6095f3878883978f9afa2f0064656106945ca`. Push CI `30968327941`, PR CI
`30968330538`, Rust CI `30968330548`, CodeQL `30968328500`, and aggregate
`92187139555` pass all eight checks. Stable Linux job `92187043157` uses pinned
LLVM/Clang 22.1.8 and records exact native exit 91.

CORE-043 remains accepted at exact implementation commit
`92b19cf729daa4e3e90d4591495e493573c89e51` and exact public synchronization head
`ef2d41b71e6509ec7c1464af53eedb0685d9a123`. Its stable Linux job `92163717297`
used LLVM/Clang 22.1.8 and produced native exit 53.

CORE-042 is accepted public at `e77276c8dcd42f6adaca7ac31e60a2d5a6fe0308`.
Its flattened direct-module composition example passed the exact native exit-47 gate,
all eight public checks passed, and PR #4 was synchronized. It remains compatibility
composition rather than a module system.

## Integration scaling controls

- PR #4 is an integration program, not a normal review-sized change. Do not let its
  review surface grow without bound: keep it draft, keep the front page synchronized
  to the accepted head, and design a separately authorized checkpoint/merge strategy
  before normal reviewability is lost. This does not authorize merging to `master`.
- Milestone selection must not optimize indefinitely for convenient compile-time
  slices. CORE-043 took the first aggregate/layout/IR vertical step, CORE-044 took a
  bounded ownership and internal by-value function-boundary step, and CORE-045 takes
  nested aggregate storage through checked IR and LLVM. CORE-046 deliberately takes
  a harder ownership/internal-ABI boundary by moving those arrays through internal
  calls. CORE-047 now takes recursive named aggregate classification, layout, field
  Copy behavior, and transport rather than selecting another scalar leaf. CORE-048
  takes immutable ownership/provenance and typed pointer representation through
  checked verification and native execution. CORE-049 takes an owned non-Copy ADT,
  exhaustive control flow, schema-bearing checked IR, and internal backend
  representation through native execution. CORE-050 deliberately takes the harder
  ownership/function-boundary class by moving that non-Copy identity through exact
  internal calls and returns. CORE-051 takes the next ADT/runtime-representation step:
  exact owned scalar payload construction, arm-local extraction, and private tagged
  aggregate lowering rather than another compile-time leaf. CORE-052 now carries that
  tagged non-Copy identity through exact internal function boundaries, shared ownership
  effects, checked call/return verification, and aggregate SSA lowering. Accepted
  CORE-053 takes immutable borrow provenance across an internal pointer-bearing call
  boundary, with one whole-signature topology classifier and checked parameter-place
  proof. Accepted CORE-054 takes explicit mutable state across branches and loops with
  one shared whole-assignment classifier and checked place/write proof. Accepted
  CORE-055 takes the harder exclusive-loan/provenance boundary with non-`Copy` aliases,
  dereference writes, lexical release, and checked active-loan verification. Accepted
  CORE-056 carries that exclusive provenance across an internal writable pointer
  boundary with a call-scoped loan and independently verified callee identity.
  Accepted CORE-057 takes the adjacent hard parent/child provenance class by
  reborrowing local aliases and mutable-reference parameters across synchronous
  internal calls. Accepted CORE-058 adds a heterogeneous private product layout;
  accepted CORE-059 generalizes immutable provenance over all admitted Copy-data;
  accepted CORE-060 takes whole-place mutable aggregate provenance and replacement;
  accepted CORE-061 unifies direct scalar/aggregate whole-owner mutation, and CORE-062
  removes the combinatorial aggregate-topology whitelists by taking recursive layout,
  function transport, verification, and native execution rather than another convenient
  compile-time leaf. Deeper CFG
  ownership, runtime representation, stable ABI, full module semantics, and real
  accelerator execution remain mandatory hard classes for later frozen decisions.
- Evidence remains proportional for current work, while chronology/identity boilerplate
  is a candidate for generation from a future structured checkpoint manifest. Such a
  manifest must be separately authorized and must not become a new source of semantic
  truth.
- Periodic system gates must compose multiple accepted capabilities through source,
  semantics, logical checked IR, verification, LLVM, native execution, documentation,
  and release-eligibility classification. CORE-042 and CORE-043 provide accepted
  composed gates; CORE-044 adds an accepted ownership/function-boundary system gate
  backed by pinned native CI. CORE-045's accepted multi-file exit-77 gate composes direct
  modules, structs, Copy function transport, fixed arrays, scalar control flow, and
  compile-time Strings through pinned native CI. CORE-046's accepted exit-91 gate adds
  exact flat-array transport across multiple function boundaries to that composition.
  CORE-047's accepted exit-107 gate adds forward/deep acyclic aggregate graphs, array
  fields, chained projection, and arrays of the resulting structs through pinned
  stable-Linux native execution. CORE-048's accepted exit-127 gate adds local immutable
  scalar-reference provenance and typed alias/load lowering to the same composition.
  CORE-049's accepted exit-149 gate adds owned unit-enum identity, conservative move
  effects, exhaustive nested dispatch, independent enum/CFG verification, and native
  selected-arm execution to that system trace. CORE-050's accepted exit-173 gate adds exact
  enum-bearing signatures, cross-function ownership transfer, direct checked SSA
  binding, call/return verification, and module-composed execution. CORE-051's accepted
  exit-181 gate adds mixed scalar payloads, exact bound-arm types, selected-lane checked
  verification, and private tagged aggregate LLVM. CORE-052's accepted exit-197 gate adds
  payload-enum producers, forwarding, consumers, aggregate parameter/call/return flow,
  exact ownership transfer, and module-composed execution. CORE-053's accepted exit-211
  gate adds direct and aliased scalar borrows, pointer-bearing parameters/calls,
  forwarding, recursion, modules, and composition with enums, Copy aggregates, arrays,
  Strings, and control flow. CORE-054's accepted exit-227 gate adds sequential,
  branch-selected, and loop-carried Int/Float/Bool mutation through explicit checked
  writes while retaining that broader composition. CORE-055's accepted exit-239 gate
  adds exclusive local mutable aliases, typed dereference loads/stores, lexical owner
  reuse, and Bool/Float/Int loan provenance. CORE-056's accepted exit-251 gate adds
  direct mutable loans across internal calls, writable checked parameter binders, and
  exact post-call owner reuse. CORE-057's accepted exit-253 gate adds local-alias and
  parameter child reborrows, multi-hop forwarding, terminating recursion, exact parent
  restoration, and continued root-owner exclusion. CORE-058's accepted exit-23 gate
  adds flat heterogeneous tuple construction, Copy, projection, transport, and private
  product LLVM. CORE-059's accepted exit-37 gate composes immutable references over
  recursive structs, tuples, numeric arrays, Copy-struct arrays, calls, recursion, and
  modules. CORE-060's accepted exit-59 gate adds exclusive whole-place mutable loans,
  aggregate writes and reads, child reborrows, exact checked schemas, and native
  execution. CORE-061's accepted exit-83 gate composes direct mutable whole-owner
  replacement across scalars, tuples, arrays, recursive structs, borrow boundaries,
  CFG, calls, enums, Strings, and direct modules under one checked place/write identity.
  CORE-062's accepted exit-109 gate composes recursive arrays, tuples, and named structs
  through source, semantics, checked IR, independent verification, LLVM, direct modules,
  ownership operations, and native execution under the pinned public LLVM/Clang 22 lane.
  Accepted CORE-083 adds an exit-83 direct-module specimen that composes
  primitive constants, compile-time String length, arrays, tuples, structs, unit and
  multi-field enums, exhaustive Match, direct mutable enum loans, alias/reborrow, exact
  checked schema, and post-call owner observation. Its complete compiler gate and
  pinned Linux/Windows native execution pass on the exact candidate and protected merge.
  Accepted CORE-084 adds a distinct exit-84 direct-module specimen
  that composes immutable enum owners, multiple aliases, immutable enum-reference
  parameters, repeated exhaustive Match reads, owner preservation, primitive constants,
  compile-time String length, arrays, tuples, and owned enum use. Its complete local
  compiler gate, corrected exact-head Linux/Windows native evidence, protected merge,
  and all three post-merge workflows pass.
  Accepted CORE-085 adds a distinct exit-85 direct-module specimen that composes
  mutable enum owners, multiple immutable aliases, exact immutable-reference parameters,
  post-loan whole-owner assignment and mutable replacement, arrays, tuples, compile-time
  String observation, and final owned Match. Its focused source-to-LLVM and corruption
  controls, complete repository gate, exact-head workflows, protected merge, and all
  three post-merge workflows pass.
  Local candidate CORE-086 adds a distinct exit-86 direct-module specimen that composes
  direct and named-alias mutable enum-reference observation, repeated read/write/read,
  homogeneous discarded Void Match, post-loan owned Match, modules, const, arrays,
  tuples, structs, Bool, and static String observation. Its focused and complete root
  gates plus local pinned LLVM/Clang 22.1.8 public/manual native execution pass; public
  exact-head and protected-merge proof remain pending.
  Local slice tests alone never establish whole-language coherence.

`CORE-041` is accepted public at `a69b7899a3dc05f663b6a68ea307ea37f5f1f401`.
Its exact local gate passed 146/146 library, 156/156 CLI, 7/7 claim, 28/28 binding,
the exhaustive four-predicate aggregate and both classifier roots, every downstream
suite, and doc tests. Three fresh exact reviewers approved. Push CI `30954270043`,
PR CI `30954273804`, stable/nightly Rust `30954274208`, CodeQL `30954270620`, and
aggregate `92143624037` pass all eight checks; stable Linux job `92143515440` used
LLVM/Clang 22.1.8 and built, LLVM-verified, machine-verified, object-lowered, linked,
and executed the exact String-predicate example with exit 43. PR #4 is synchronized
to this accepted head and remains draft.

`CORE-040` is accepted public at `edd63f3c59de38b19d92aebec1b6915240b5e5a5`.
Its exact local gate passed 145/145 library, 155/155 CLI, 7/7 claim, 28/28 binding,
the exhaustive equality aggregate, every downstream suite, and doc tests. Three fresh
exact reviewers approved. Push CI `30951517745`, PR CI `30951522726`, stable/nightly
Rust `30951522837`, CodeQL `30951520564`, and aggregate `92134642374` pass all eight
checks; stable Linux job `92134492264` built, LLVM-verified, machine-verified,
object-lowered, linked, and executed the exact String-equality example with exit 41.

`CORE-039` is accepted public at `7709eec6b5eb18249a756225ff7c368ccbed5341`.
Its exact local gate passed 144/144 library, 154/154 CLI, 7/7 claim, 28/28 binding,
both CORE-038/CORE-039 aggregates, every downstream suite, and doc tests. Three exact
reviewers approved. Push CI `30948007054`, PR CI `30948009660`, stable/nightly Rust
`30948009897`, CodeQL `30948007588`, and aggregate `92122867238` pass; stable Linux
job `92122711179` built, LLVM-verified, machine-verified, object-lowered, linked, and
executed the static-string-length example with exact exit 33.

`CORE-038` is accepted public at `25805f561fa32d2f89463cb32ff5d0c5adff7acb`.
Its exact local gate passed 143/143 library, 153/153 CLI, 7/7 claim, 28/28 binding,
and the fixed-array-length class 1/1 plus every downstream suite. Push CI
`30944853303`, PR CI `30944856497`, stable/nightly Rust `30944856525`, CodeQL
`30944854080`, and aggregate `92112319035` pass; stable Linux job `92112187769`
built, LLVM-verified, machine-verified, object-lowered, linked, and executed the
fixed-array-length example with exact exit 37.

## Completed CORE-062 hypothesis

One recursive CopyData contract can replace scalar/flat/numeric/container-specific
topology guards without broadening unsupported leaves or ownership semantics. Exact
source, semantic, checked-IR, verifier, LLVM, CLI, and pinned native evidence accepts
that hypothesis at `e62fd747`. Projected targets/origins, new signature or lifetime
policy, reference results, NLL, drop, stable ABI/FFI, memory-safety, and accelerator
claims remain outside the accepted class.

The completed `AUDIT-032` hypothesis was:

A full-set, delta-aware comparison from the clean accepted head can select one
bounded, reproducible residual—or an explicit stop—without inheriting prior
rankings or repeating accepted slices. The audit remains static and read-only;
semantic ambiguity, more than two compiler phases, hardware needs, or unsupported
capability claims are stop conditions rather than implementation invitations.

## Founding-framework checkpoint

- The tracked nine-page primary design paper is now treated as Aero's governing
  vision input, not as current implementation evidence.
- The tracked Claude strategy PDF is a truncated one-page capture. Its preserved
  execution-quality guidance and AI/ML-infrastructure recommendation are usable;
  absent continuation is not inferred.
- The primary paper establishes fixed-size arrays and compile-time array-size
  computation as design directions, but neither founding PDF specifies String
  `.len()` semantics. `CORE-039` is authorized by the compiler's two existing explicit
  String-`len` semantic tables, the tracked built-in collections summary's character
  count/UTF-8 contract, the type-system document's Unicode scalar definition, and the
  bounded checked literal contract—not by inventing detail absent from the PDFs.
- `FRAMEWORK_ALIGNMENT.md` records source authority, current gaps, an execution-
  quality scorecard, and the Aero-native AI/ML infrastructure flagship direction.
- `Roadmap.md` now follows the founding Design -> Minimal Prototype -> Self-Host ->
  Stabilize -> Optimize path through explicit evidence gates. Current position is
  Minimal Prototype / correctness recovery; historical v1.0/completed-phase labels
  do not establish stability.
- This checkpoint does not broaden `CORE-009` or authorize speculative aggregate,
  ownership, accelerator, or benchmark semantics.

## Repository state

- Upstream: `https://github.com/RobVanProd/aero.git`
- Default branch: `master`
- Starting commit: `8f8c7337a4008082fd2a443fcc814b5847b8663f`
- Starting commit date: `2026-05-28T21:13:40-04:00`
- Current branch: `agent/aero-integration`
- Public draft PR: `https://github.com/RobVanProd/Aero/pull/4`
- Current accepted public ARCH-001 eligibility head:
  `241e39e5426f3edcbd47d72150b7dd1bcefda31e`, parent `4c18450a`, tree
  `de8b5cdef10d11aba29ed2e0186c086ca04c0c44`, correction canonical binary diff
  `07e29dd01bd4848541d62bf49cc5600c328cb1b9`, cumulative sync diff from
  `1dcfd869` `7c6e746f5c38ff30038903cab256ca7665a43bbf`. The corrected sync is
  triple-approved and public all-eight green in push CI `30935499915`, PR CI
  `30935511275`, stable/nightly Rust `30935508627`, all three CodeQL analyses in
  `30935500629`, and aggregate `92080638204`.
- Current accepted public AUDIT-043 authorization head:
  `5276df5b6f3369bd2b6fc78a7a39289e8609ed00`, parent `cb43d1bb`, tree
  `c3eaf3cf244f6f6e7e97423e2bcc6a1a8b44dc58`, correction canonical binary diff
  `b8b7586ffef867f48e440d5ee8dbff9b5a653c39`, cumulative diff from CORE-036
  `fe5376dc943881e8d15f852e8a66dd795797a3c8`. The corrected six-record
  authorization is triple-approved and public all-eight green in push CI
  `30931510621`, PR CI `30931515125`, stable/nightly Rust `30931515426`, all three
  CodeQL analyses in `30931509579`, and aggregate `92067252294`.
- Accepted public CORE-036 closure:
  `3f042e18766d4675d04e0ba7e0289b7aac43d7ea`, parent `799c4181`, tree
  `15d56e0ceb0715543b03f7338505901906b59d60`, canonical binary diff
  `ee8cbed07657edf21559205c0bc23b7bb0f40a53`. The exact six-record second
  additive correction changed 62 lines in and 8 lines out, passed its fresh exact
  gate, received three exact approvals, and was published unchanged. Push CI
  `30930377220`, PR CI `30930379386`, stable/nightly Rust `30930380195`, all three
  CodeQL analyses in `30930375201`, and aggregate `92063404658` pass.
- Accepted public CORE-036 implementation:
  `26d18924a7fe59eb99a6ed40de2f435b30093c7b`, parent `d52b117e`, tree
  `8aec746cd3786eb839b7df705c26726d8341e9fa`, canonical binary diff
  `543f8a1ccb8d737587877d0886614d48fe747881`. The exact two-file CORE-036
  implementation is triple-approved; compiler `30928759703` / `30928760789`,
  stable/nightly Rust `30928758562`, all three CodeQL analyses in `30928754859`,
  and aggregate `92057919831` pass.
- Accepted public CORE-035 closure:
  `60ad91f7d6ab3d9881346ab5b98f1d0e161d6629`, parent `b8fd5a17`, tree
  `978aa98fbab94ebc1a949a3e4f7eb023ee922281`, canonical diff
  `818a811299fc57d185f12eafe6e422569a0eea4f`. The six-record snapshot passed its
  exact full local gate at 139/139 library, 149/149 binary, 7/7 claim, and 24/24
  binding tests, received three approvals, and was published unchanged. Compiler
  `30923835957` / `30923837627`, stable/nightly Rust `30923838264`, all three
  CodeQL analyses in `30923834264`, and aggregate `92041128413` pass.
- Accepted public `CORE-033` closure
  `1ee9c71b555bec8066277cb9c64a7a7a2a3ff498`, parent `19f688a`, tree
  `d081988164fba75fcfe7af8788fbd010bb5a158d`, established PowerShell full-index
  canonical diff `7303da4793f01fd7d532f24030849761536835d0`, passed its exact
  full local gate with 139/139 library, 149/149 binary, 7/7 claim, and 22/22 binding
  tests, received three exact approvals, and was published unchanged. Compiler
  `30893527220` / `30893529999`, stable/nightly Rust `30893529992`, all three
  CodeQL analyses in `30893527445`, and aggregate `91941079083` pass. The additive
  correction preserves the public ancestry of rejected `fe90f583` without rewrite;
  R-002 and every capability/matrix classification remain unchanged.
- Accepted public `CORE-033` implementation
  `76a6e80233a1854602fb134e3c4367d80a7b0e81`, tree `d8391348`, established
  PowerShell full-index canonical diff `a75b59b2`, passes formatting, focused 1/1,
  binding 22/22, the exact full local gate exit 0, compiler `30891890629` /
  `30891898590`, stable/nightly Rust `30891897083`, all three CodeQL analyses in
  `30891892219`, and aggregate `91935804190` after corrected-identity triple
  approval. Only semantic analysis and checked admission changed; R-002 remains
  HIGH/CRITICAL and PARTIALLY CONTROLLED, and no capability/matrix class moves.
- Triple-reviewed public CORE-033 tests-first `ac4cb2a5`, tree `852bff0b`, canonical
  diff `4ca50572`, reproduces the sole 21/22 binding failure with exactly 12 frozen
  acceptances in compiler `30891243037` / `30891246443` and nightly Rust
  `30891247469`; stable was fail-fast cancelled, while CodeQL `30891241566` and
  aggregate `91933672071` pass. Rejected unpublished `7608b42c` omitted the
  initialized three-array-deep semantic/checked preservation control.
- Corrected CORE-033 authorization `66207215`, tree `357c2731`, canonical diff
  `96b5f403`, passes compiler `30890569245` / `30890571370`, Rust `30890571249`,
  CodeQL `30890569479`, and aggregate `91931557818` after three exact approvals.
  Rejected unpublished `d0500865` mislabeled Candidate T as Candidate B.
- Accepted public read-only `AUDIT-038` authorization
  `e4d58e59ff831df4d530e6de9c9ff31964af86d7`, tree `f265d8af`, canonical diff
  `31d09f92`, passes compiler `30883186212` / `30883188223`, stable/nightly Rust
  `30883188248`, all three CodeQL analyses in `30883186829`, and aggregate
  `91908783685` after three exact approvals. All complete rankings place R-002
  first; final compatibility reconciliation unanimously approves initialized exact
  array-of-tuple containment. No file changed during ranking and no capability/risk
  status moved.
- Accepted public `CORE-031` record closure
  `45696091d9ba10f97e1ce42b9372f330c3b4199b`, tree `480c3504`, canonical diff
  `d682b0f6`, passes compiler `30882630407` / `30882632698`, stable/nightly Rust
  `30882632696`, all three CodeQL analyses in `30882630822`, and aggregate
  `91907149874` after three exact approvals. The closure was published unchanged
  from a clean worktree; `master` and `origin/master` remain `8f8c733`.
- Accepted public `CORE-031` implementation
  `4bc7a3453f0829fca11929e0826abd4ed06fb962`, tree `61361621`, canonical diff
  `349e34ee`, passes focused 1/1, binding 20/20, formatting, the exact full local
  gate, compiler `30882153355` / `30882155935`, stable/nightly Rust
  `30882155921`, all three CodeQL analyses in `30882154595`, and aggregate
  `91905705897` after three exact approvals. Only semantics and checked admission
  changed; R-002 and every capability/matrix classification remain unchanged.
- Triple-reviewed public CORE-031 tests-first `6899cb1b`, tree `b7007735`,
  canonical diff `43063551`, reproduces exactly nine frozen false acceptances in
  compiler `30881792006` / `30881794177` and nightly Rust `30881794186`; stable
  was fail-fast cancelled. CodeQL `30881792351` and aggregate `91904645414` pass.
  Triple-reviewed authorization `ba57efec`, tree `c01bebe9`, diff `1fb56631`,
  passes all eight public checks in compiler `30881170087` / `30881172516`, Rust
  `30881172590`, CodeQL `30881170763`, and aggregate `91902778624`.
- Accepted public read-only `AUDIT-037` authorization
  `987188fc265481d0de4c3021bcc5c3161aaeed12`, tree `0b685659`, diff
  `d3a9974b`, passes compiler `30880025888` / `30880028697`, stable/nightly Rust
  `30880028653`, all three CodeQL analyses in `30880025866`, and aggregate
  `91899286217` after three exact approvals. All three complete rankings place
  R-002 first; targeted comparison unanimously selects exact array-array-tuple
  containment. No file changed during ranking and no capability/risk status moved.
- Accepted public `CORE-030` record closure
  `cd8add28d1a6533b9955dbb4fcb86670c61eba88`, tree `8ab06d62`, diff
  `18ffa30d`, passes compiler `30879329940` / `30879332975`, stable/nightly Rust
  `30879332995` attempt 2, all three CodeQL analyses in `30879330627`, and
  aggregate `91897195358` after three exact approvals. Rust attempt 1 failed on
  transient Linux `ETXTBSY` while executing the unchanged fake-verifier fixture;
  the focused rerun passed both jobs without a file or ref change. The closure was
  published from a clean worktree; `master` and `origin/master` remain `8f8c733`.
- Accepted public `CORE-030` implementation
  `97c0f04f32c28a8a541fec51ec2bef175aaa6032`, tree `aa3a9e3f`, diff
  `06a104df`, passes focused 1/1, binding 19/19, formatting, the exact full local
  gate, compiler `30878810762` / `30878812430`, stable/nightly Rust
  `30878812406`, all three CodeQL analyses in `30878811198`, and aggregate
  `91895661773` after three exact approvals. Only semantics and checked admission
  changed; R-002 and every capability/matrix classification remain unchanged.
- Triple-reviewed public CORE-030 tests-first `bd28f6a`, tree `f12d7fd8`, diff
  `88e2cc0b`, reproduces the exact five frozen false acceptances in compiler
  `30878470029` / `30878471826` and nightly Rust in `30878471848`; stable was
  fail-fast cancelled. CodeQL `30878470495` and aggregate `91894672603` pass.
  Triple-reviewed authorization `1f13084`, tree `5b43f60d`, diff `869c39bb`, passes
  all eight public checks in compiler `30878148019` / `30878151278`, Rust
  `30878151307`, CodeQL `30878148144`, and aggregate `91893736780`.
- Accepted public read-only `AUDIT-036` authorization
  `f4ac505040f866126f2de3ccdcc1ed202711cd46`, tree `3cdf89e6`, diff
  `40896f51`, passes compiler `30876975678` / `30876977928`, stable/nightly Rust
  `30876977905`, all three CodeQL analyses in `30876976155`, and aggregate
  `91890402326` after three exact approvals. The worktree stayed clean; all three
  complete rankings select exact R-002 valueless immediate array-of-tuple
  containment. No capability or risk status changes.
- Accepted public `CORE-029` record closure
  `7222b9a02d7bbbcd00fd4c3af54e9169be567298`, tree `66084b36`, diff
  `90bf540c`, passes compiler `30876033717` / `30876035730`, stable/nightly Rust
  `30876035761`, all three CodeQL analyses in `30876034500`, and aggregate
  `91887644623` after three exact approvals. PR #4 remains open/draft; upstream
  `master` remains `8f8c733`.
- First `AUDIT-036` authorization snapshot `d6f24b8c`, tree `ac21692e`, diff
  `898b7869`, passed its fresh exact full local gate but was rejected before
  publication at P3 for retaining August 3 dates after local midnight and at P2
  because one sentence made the absence of implementation authority sound
  conditional on public gate completion. The corrected records use August 4 and
  state separately that AUDIT-036 never grants implementation/capability authority;
  passing gates unlocks read-only rankings only.
- Accepted public `CORE-029` implementation
  `29bd2e0167df5e33fdd49622d32620eac6176979`, tree `53282149`, diff
  `acc1c247`, passes focused 1/1, binding 18/18, formatting, the exact full local
  gate, compiler `30875100237` / `30875102914`, stable/nightly Rust
  `30875102909`, all three CodeQL analyses in `30875100762`, and aggregate
  `91884963697` after three exact approvals. Only semantics and checked admission
  changed; no capability is promoted.
- Triple-reviewed public tests-first
  `d12ba66ae015070399d7783e82ff7ca60e60dc42`, tree `056a9d52`, diff
  `61c19e70`, reproduces exactly one 17/18 aggregate failure with the five frozen
  false acceptances in compiler `30874817273` / `30874819174` and nightly Rust
  `30874819175`; stable is fail-fast cancelled. CodeQL `30874817566` and aggregate
  `91884136725` pass. Superseded snapshot `d824c1ca` was rejected at P2 before
  publication for missing two mutable preservation controls.
- Corrected public `CORE-029` authorization
  `c0e1a90ce8c57373fb7ee5d0210107566866519d`, tree `3960cc07`, diff
  `b8df373a`, passes compiler `30874315655` / `30874317720`, stable/nightly Rust
  `30874317762`, CodeQL `30874316881`, and aggregate `91882644806` after three
  exact approvals. Superseded snapshot `535f876d` was rejected at P2 before
  publication for missing exact immutable/mutable duplicate specimens.
- First `CORE-029` closure snapshot `bb56132a`, tree `3cef5fa5`, diff `5dac576d`,
  passed its fresh exact full local gate but was unanimously rejected at P3 before
  publication because the ledger and Exact next action treated that completed gate
  as future work. Second snapshot `8a15bb03`, tree `43c8b5d5`, diff `325314be`,
  corrected that tense and passed a fresh gate rerun but was rejected at P3 before
  publication because it omitted the first rejected closure from the chronology.
  Third snapshot `be7dfda8`, tree `39080f71`, diff `d1b7f5ba`, recorded the first
  rejection and passed another fresh gate rerun but was rejected at P3 before
  publication because it omitted the second rejected snapshot. The current records
  preserve all three stages, state the gate is complete, begin the handoff with
  reviews/publication, and pass a fresh exact full-gate rerun.
- Public read-only `AUDIT-035` authorization
  `f1cd972f8d982c40c7c5afa2f270551763c19c2a`, tree `b9c6270b`, diff
  `7f221d2a`, passes compiler `30872922468` / `30872923806`, stable/nightly Rust
  `30872923874`, all three CodeQL analyses in `30872922858`, and aggregate
  `91878491979` after three exact approvals. Three independent complete rankings and
  unanimous targeted reconciliation select CORE-029's exact R-002 boundary; the
  worktree remained clean and no audit probe/test/artifact/external query ran.
- First `CORE-029` authorization snapshot `535f876d`, tree `763238a5`, diff
  `67c9ddde`, was rejected at P2 before publication because its tests-first contract
  lacked exact immutable and mutable immediate-reference-to-tuple duplicate-
  precedence specimens. The corrected contract adds both already-green controls and
  preserves the one-test/five-failure arithmetic.
- Accepted public `CORE-028` record closure
  `032d0d05f6fa4cfe3ac01e6add2b6fc4443cb338`, tree `443aacdc`, diff
  `93fce8ae`, passes compiler `30872236535` / `30872238993`, stable/nightly Rust
  `30872239003`, all three analyses in CodeQL `30872237025`, and aggregate
  `91876507154` after three exact approvals. PR #4 remains open and draft; upstream
  `master` remains `8f8c733`.
- First `AUDIT-035` authorization snapshot `b07adf20`, tree `fc524b74`, diff
  `4c64dc65`, was rejected at P2 before publication because two records made the
  absence of implementation/capability authority sound conditional on gate
  completion. The corrected contract states separately that gates unlock read-only
  ranking only and that AUDIT-035 never grants implementation/capability authority.
  Corrected snapshot `5f8bdd43`, tree `ec2f7791`, diff `4c6defae`, fixed that P2 but
  was rejected at P3 before publication because Exact next action still instructed a
  future actor to run the already completed local gate. Corrected authorization
  `f1cd972` was published unchanged and passed all eight checks before ranking began.
- Accepted public `CORE-028` implementation
  `e051452470cb0f17ee4d9940b989ee3bef10d333`, tree `63985b2d`, diff
  `79830403`, passes focused 1/1, binding 17/17, the exact full local gate at
  139/139 library and 149/149 binary tests plus every active integration/doc test,
  compiler `30871337443` / `30871335738`, stable/nightly Rust `30871337440`, all
  three CodeQL analyses in `30871336117`, and aggregate `91873866339` after three
  exact approvals. Only semantics and checked admission changed; no capability is
  promoted.
- Triple-reviewed public tests-first
  `3fb5f7a687bd22f3f7002e112da1fabb2ec2e791`, tree `f12a6c6b`, diff
  `77320dc5`, reproduces exactly 16 passed/1 failed in both compiler runs
  `30871003009` / `30871004997` and stable/nightly Rust `30871005020`, with the
  one aggregate test reporting exactly five frozen false acceptances. CodeQL
  `30871003987` and aggregate `91872902124` pass.
- Public corrected `CORE-028` authorization
  `4cc682fc7f2deb1e2b47fd3f5548990ac44794e8`, tree `be2987d0`, diff
  `7a658443`, passes compiler `30867953738` / `30867951091`, stable/nightly Rust
  `30867953730`, CodeQL `30867951533`, and aggregate `91863823065` after three
  exact approvals. Superseded snapshot `696dcaad` was rejected at P2 before
  publication; the corrected contract adds tuple-specific duplicate and valueless
  nested-shape preservation controls.
- First `CORE-028` closure snapshot `a20548ec`, tree `8250ce11`, diff `f0f181f9`,
  was rejected at P2 before publication because historical authorization text in the
  matrix and project state remained in contradictory present tense. Corrected
  snapshot `5cc3ccb8`, tree `2f935a66`, diff `f11da400`, qualified those statements
  but was also rejected at P2 before publication because the canonical R-002 row
  still ended at CORE-025 and listed the now-closed exact valueless outer-tuple case
  as residual. The current closure adds CORE-028 to that row and narrows the residual
  without changing likelihood, impact, or PARTIALLY CONTROLLED status. Third
  snapshot `782bc8fb`, tree `1914aaf7`, diff `e1962dbb`, contained that correction
  but was rejected at P2 before publication because the ledger described its fresh
  final-tree gate as both passed and pending. Accepted closure `032d0d0` records one
  unambiguous fresh exact-gate result and the complete public evidence above.
- Public `AUDIT-034` authorization
  `45783af9b54277b83dd58fc9d6162163c451bbb3`, tree `f1baa457`, diff
  `1e8563ae`, passes compiler `30866227485` / `30866229553`, stable/nightly Rust
  `30866229554`, all three analyses in CodeQL `30866227939`, and aggregate
  `91858665436` after three exact approvals. Three complete read-only rankings and
  final targeted reconciliation unanimously select exact R-002 uninitialized outer-
  tuple rejection for preregistered `CORE-028`; the worktree remained clean.
- First `CORE-028` authorization snapshot `696dcaad`, tree `fa86e465`, diff
  `b5d8ea46`, was rejected at P2 before publication because the proposed tests did
  not exercise tuple-specific duplicate precedence or valueless tuple nesting under
  non-tuple array/reference/generic outer annotations. The corrected records require
  those exact controls; at that rejected snapshot, no test or compiler edit was
  authorized.
- Accepted public `CORE-027` record closure
  `d649c2d8a9db1fdf51a5065e90ae79d5240412f4`, tree `b5ad7ee2`, diff `d4281863`,
  passes compiler `30865772404` / `30865775196`, stable/nightly Rust
  `30865775214`, all three analyses in CodeQL `30865772793`, and aggregate
  `91857289172` after three exact approvals. PR #4 remains open and draft; upstream
  `master` remains `8f8c733`.
- Accepted public `CORE-027` implementation
  `b3e79103ec4238abb3c4e07beddc0ef9cf07f1b8`, tree `2728bbc6`, diff `90e1c4b6`,
  passes focused 1/1, version-claim 8/8, exact `./tools/test.sh` at 139/139 library
  and 149/149 binary tests plus every active integration and doc test, compiler
  `30865344667` / `30865346597`, stable/nightly Rust `30865346602`, all three
  analyses in CodeQL `30865345043`, and aggregate `91855955012`. The first
  implementation snapshot `01615da` was rejected at P2 before publication for an
  incidental final-newline change; corrected exact snapshot `7d3322a`, tree
  `2728bbc6`, received three approvals and was published unchanged.
- Triple-reviewed tests-first `f57cf2ee5656769c1ee62c16c426dd5818138bc8`, tree
  `8a99d994`, diff `1018ee35`, publicly reproduces exactly 7 passed/1 failed in
  compiler `30864786831` / `30864789388` and nightly Rust in run `30864789399`:
  only `grammar_and_core_tutorial_are_visibly_design_targets` fails. Stable is
  fail-fast cancelled; all three CodeQL analyses in `30864787921` and aggregate
  `91854279316` pass.
- Public `CORE-027` authorization `35747046f4018707bad716f1ca6266c3f2af2cfb`,
  tree `a3aa2dc2`, diff `f07e82d5`, passes compiler `30864498927` /
  `30864501308`, stable/nightly Rust `30864501289`, CodeQL `30864499437`, and
  aggregate `91853381216` after three exact approvals.
- Public `AUDIT-033` authorization
  `544b1ba3a5080a162425bd206330ed48c69ac16c`, tree `cdc3a085`, diff `8a242e5d`,
  passes compiler `30863291761` / `30863294642`, stable/nightly Rust `30863294655`,
  all three analyses in CodeQL `30863292940`, and aggregate `91849762353`. Three
  complete read-only rankings and targeted reconciliation select exact R-010
  documentation-authority containment. The worktree remained clean throughout.
- Accepted public `CORE-026` record closure
  `0a940eadae5974abb11154c0e484f4178bfed144`, tree `6ec4c609`, diff `4e1db178`,
  passes compiler `30862783787` / `30862786131`, stable/nightly Rust `30862786150`,
  all three analyses in CodeQL `30862784231`, and aggregate `91848258218`. Three
  exact reviewers approved the corrected closure after rejecting superseded snapshot
  `615c00b9` for stale gate chronology. PR #4 remains open and draft; upstream
  `master` remains `8f8c733`.
- Accepted public `CORE-026` implementation
  `8c2b2ecd88fcbecd2423254376cc00f0c3f0fcc3`, tree `eabd8939`, diff `c4623bc1`,
  passes focused 1/1, checked-IR 7/7, exact `./tools/test.sh` at 139/139 library and
  149/149 binary tests plus every active integration and doc test, compiler
  `30862232159` / `30862233829`, stable/nightly Rust `30862233777`, all three
  analyses in CodeQL `30862232615`, and aggregate `91846586968`. Three exact
  reviewers found no P0-P3 issue. Only checked admission changed; verifier, generated
  IR for valid programs, source semantics, codegen, ABI, and backends are unchanged.
- Triple-reviewed tests-only `1538a3e384e0ccee8f55c335295687c2d5c5e07a`, tree
  `8f3cd8fb`, diff `5b3b2519`, publicly reproduces exactly 6 passed/1 failed in
  compiler `30861809364` / `30861811517` and nightly Rust in run `30861811567`:
  only `known_scalar_top_level_call_arity_fails_at_checked_admission` fails, with
  too-few and too-many results still at Verification rather than Admission. Stable
  is fail-fast cancelled. CodeQL `30861809624` and aggregate `91845291318` pass.
- Public `CORE-026` authorization `7dc3eac9f8bf0729a8bcc91481c8a36d2f0a8bd1`,
  tree `5f06ef4f`, diff `8bd00ae0`, passes compiler `30861160746` / `30861162982`,
  stable/nightly Rust `30861162836`, CodeQL `30861160881`, and aggregate
  `91843332635` after three exact approvals. Earlier authorization and tests-first
  snapshots were rejected before publication until signature eligibility, gate
  chronology, caller-first ordering, and composite/reference result controls were
  explicit.
- Public-green read-only `AUDIT-032` authorization
  `b6b1c639de35904521679f995db3418112d78f6a`, tree `c8803965`, diff `891bb8a4`,
  passes compiler `30858876643` / `30858879497`, stable/nightly Rust `30858879480`,
  CodeQL `30858875767`, and aggregate `91836318450`. Its three complete rankings
  and targeted reconciliation selected the exact one-phase CORE-026 boundary.
- Accepted public `CORE-025` record closure
  `b0fe242c0bfaf4aaf3030f36ec333de700dd18a3`, tree `2a5d233f`, diff `98916b4d`,
  passes compiler `30858384541` / `30858387195`, stable/nightly Rust `30858387193`,
  all three analyses in CodeQL `30858385234`, and aggregate `91834740790`. Three
  exact reviewers approved the corrected closure. PR #4 remains open, draft, and
  mergeable; upstream `master` remains `8f8c733`.
- Accepted public `CORE-025` implementation
  `1ec8bebc04e6dbc30a47ef011bdb8fae334194a9`, tree `ac2c8fdd`, diff `b765db31`,
  passes focused 1/1, binding 17/17, exact `./tools/test.sh` at 139/139 library and
  149/149 binary tests, compiler `30857775577` / `30857777431`, stable/nightly Rust
  `30857777314`, all three analyses in CodeQL `30857775231`, and aggregate
  `91832840108`. Three exact reviewers found no P0-P3 issue. Exact initialized outer
  tuple binding annotations now stop after child validation in semantics and checked
  admission; tuple capability remains unpromoted and broader R-002 remains
  PARTIALLY CONTROLLED.
- Triple-reviewed tests-only `39ccd9c97cdd25126f925febd93308c071878f58`,
  tree `5b05499f`, diff `765bca0b`, produces exactly 16 passed/1 failed in compiler
  `30857467570` / `30857469931` and nightly Rust `30857470046`, with only the five-
  boundary tuple-annotation target failing. Stable is fail-fast cancelled. CodeQL
  `30857468030` and aggregate `91831822409` pass. A prior snapshot was rejected P2
  because it matched diagnostic fragments; the accepted snapshot requires exact
  complete strings, including child-precedence controls.
- Public `CORE-025` preregistration
  `722d4d129abde2f6b276770aa03b05c933580de5`, tree `5eed3943`, diff `32520171`,
  passes compiler `30856866754` / `30856869023`, stable/nightly Rust `30856869057`,
  CodeQL `30856866855`, and aggregate `91829921341`.
- Public `AUDIT-031` authorization
  `ba258c6e424454930b670d9c3e95f0b027ff33cf`, tree `651762a8`, diff `20115b18`,
  passes compiler `30855407928` / `30855410819`, stable/nightly Rust `30855410731`,
  all three analyses in CodeQL `30855409113`, and aggregate `91825280915`. Three
  auditors completed full read-only rankings; targeted reconciliation selects the
  exact R-002 tuple-annotation containment and leaves R-010 as runner-up.
- Accepted public `CORE-024` record closure
  `226b7fbb89cfe9854b54d776ff5416bed516a670`, tree `1337945c`, diff `861b5ec3`,
  passes compiler `30854853182` / `30854856449`, stable/nightly Rust `30854856190`,
  all three analyses in CodeQL `30854853829`, and aggregate `91823492290`. Three
  fresh reviewers approved after rejecting a prior stale-chronology snapshot. PR #4
  remains open, draft, and mergeable; upstream `master` remains `8f8c733`.
- Public `CORE-024` preregistration `b8fb1d2`, tree `1af5d40e`, diff `2f359899`,
  passes compiler `30853169966` / `30853174632`, Rust `30853174801`, CodeQL
  `30853170646`, and aggregate `91818026672`.
- Triple-reviewed tests-only `ab8508e`, tree `477c0ebf`, diff `af30f207`, produces
  exact 148/149 in compiler `30853599874` / `30853602996` and stable/nightly Rust
  `30853603035`; only the parser UTF-16 regression fails at scalar `20` versus
  expected `21`. CodeQL `30853601414` and aggregate `91819440238` pass.
- Accepted public `CORE-024` implementation
  `a3d110ecb963b30665f4996bfada4f453a8d1557`, tree `79ccfca1`, diff `74bfbcea`,
  passes focused 1/1, LSP 10/10, the exact full local gate, compiler
  `30854094706` / `30854099595`, stable/nightly Rust `30854099899`, all three
  analyses in CodeQL `30854094981`, and aggregate `91821038577`. Three exact
  reviewers found no P0-P3 issue. The draft PR remains open; `master` is untouched.
- `CORE-020` record-only closure `5a8cd06`, tree `df4a04a`, passes compiler runs
  `30835593703`/`30835597576`, stable/nightly Rust run `30835597620`, all three
  analyses in CodeQL run `30835594365`, and aggregate `91759990615`. The selected
  unsupported-options boundary is closed; real option behavior and broad R-006
  convergence remain open.
- `AUDIT-027` public basis `aa3e7a8`, tree `4caa5c33`, passes compiler runs
  `30836250279`/`30836251909`, stable/nightly Rust run `30836255407`, all three
  analyses in CodeQL run `30836248101`, and aggregate `91762198170`. The worktree
  remained clean and auditors used static repository evidence only.
- Accepted public `CORE-019` final-state sync head:
  `25dec51e7fb24a5dd835712568242d685af649cf`. Three independent reviewers approved
  exact record-only diff `a3cd465fab08c4c9b6b238c7aadd4a39a4d06c3d` and tree
  `46828e7d715c6489eb2c7a661a7ef95b7cb4555b` with no P0-P3 findings. Both compiler-
  test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate CodeQL pass
  in runs `30830484863`, `30830489796`, `30830490379`, and `30830483828`, with
  aggregate check `91743120769`; draft PR #4 remains open and mergeable and upstream
  `master` remains `8f8c733`.
- Accepted public `CORE-019` closure head:
  `63b66295544d41634f790face005d0fcfc64b41a`. Three independent reviewers approved
  corrected record-only diff `b4fd6bc195f70712fbcd0f022d5dcbbcad7128c9` and tree
  `2e88685021de6a7948e6b5ffb69250676764f7f5` with no P0-P3 findings. Both compiler-
  test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate CodeQL pass
  in runs `30829963152`, `30829970545`, `30829968789`, and `30829962982`, with
  aggregate check `91741344282`; draft PR #4 remains open and mergeable and upstream
  `master` remains `8f8c733`.
- Accepted public `CORE-018` final-state sync head:
  `d0bd54e93ff9fda9e769dd29abcec02a1f550e9a`. Three independent reviewers approved
  corrected exact diff `a4034521b5976f4c737871d5be7e93d2a1f34bfb` and tree
  `21e72079679550b73935b56d87e4e062fc48d88e` with no P0-P3 findings after correction
  of one CUDA stage-precision defect. Both compiler-test jobs, stable/nightly Rust,
  all three CodeQL analyses, and aggregate CodeQL pass in runs `30824106058`,
  `30824111861`, `30824110412`, and `30824105642`, with aggregate check
  `91721342986`; draft PR #4 remains open and mergeable and upstream `master` remains
  `8f8c733`.
- Accepted public `CORE-018` closure head:
  `2e0e17fde6d9b11c2f5705c45b23468e0b04cbf0`. Three independent reviewers approved
  exact record-only diff `3d0a17f75e74446d5db0a132084fb3ca7973c6ed` and tree
  `83c9676f905dde55d5da52ed3961607c2aec9d55` with no P0-P3 findings. Both compiler-
  test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate CodeQL pass
  in runs `30823259890`, `30823261072`, `30823260717`, and `30823257183`, with
  aggregate check `91718428033`; draft PR #4 remains open and mergeable and upstream
  `master` remains `8f8c733`.
- Public `CORE-016` preregistration head:
  `1575914e7ab1f3c70793c77a1d82b7b3a78bb441`. Three independent reviewers approved
  exact staged diff `321fb61c3932cd0663bc5bcbc0aecb02361ab010` and tree
  `4933dc2e9297cc5d7d0742c28081571e3fc23c5f` with no P0-P3 findings after the first
  snapshot was rejected and corrected. Both compiler-test jobs, stable/nightly Rust,
  all three CodeQL analyses, and aggregate CodeQL pass; draft PR #4 is mergeable.
- Public `CORE-016` tests-only red head:
  `4b94dbd55465d2f94c2e7840f26ce5f73e571f30`. Three independent reviewers approved
  exact staged diff `b734773e6f1f4bb9c9561dc089e72b103e3b4e25` and tree
  `488687b20c882c78c8e801d46cdb0bf817d7f421` with no P0-P3 findings. Both
  compiler-test jobs and nightly Rust reproduce the intended 2-pass/5-fail target;
  stable reached its test step before matrix fail-fast cancellation. All three
  CodeQL analyses and aggregate CodeQL pass; draft PR #4 remains mergeable.
- Public `CORE-016` implementation head:
  `cc984d0afe4c63f3c322f8da7c34fc666f8ec072`. Three independent reviewers approved
  exact canonical staged diff `e0c2bbb61f33ea53e1c07d472a21a631170c22e7`
  and tree `8d5ba37b0a58c715cf72721ade23471c5fa4fa7c` with no P0-P3 findings. Both
  compiler-test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate
  CodeQL pass; draft PR #4 is open, mergeable, and remains intentionally draft.
- Accepted public `CORE-016` closure head:
  `ea036f2e71a4f67b1f8c6f711488f02f65fc4ad5`. Three independent reviewers approved
  exact record-only diff `7b24a58e7475700423dc66da368a22b97f9c31e8` and tree
  `4c7f526617ecb8e3a0c28622f8eca44dac627981` with no P0-P3 findings. Both
  compiler-test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate
  CodeQL pass; draft PR #4 remains open and mergeable.
- Accepted public `CORE-016` final-state sync head:
  `8869ecab0a7aadb51d9da193bf480a6fa97a9b3e`. Three independent reviewers approved
  corrected exact diff `8379a2c67e4b72c54d92f66480bd836805582589` and tree
  `4318bd3f0eea4dda7f6264ac5e9ae1694d0d5960` with no P0-P3 findings after two stale
  state anchors were rejected and fixed. Both compiler-test jobs, stable/nightly Rust,
  all three CodeQL analyses, and aggregate CodeQL pass; draft PR #4 is mergeable.
- Public `CORE-017` preregistration head:
  `2c61535092f22f2f513aac0fcee9d34d9c621212`. Three independent reviewers approved
  exact diff `ebe348e00721596f768b900547b9d19b56e44df4` and tree
  `1d890b93351e54fb6903aa952957494a517d40a9` with no P0-P3 findings. Both
  compiler-test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate
  CodeQL pass; draft PR #4 is open and mergeable.
- Public `CORE-017` implementation head:
  `8be8c21696cf98602c82e1e5e4fdfc6bf10e9777`. After the first snapshot was rejected
  for an underasserted method body, all three independent reviewers approved corrected
  exact diff `a417c7e3c076e7ff6951ce9c181ea99d6bdfa3b6` and tree
  `83bf4f0ba8f973e7ec39167e53114cf5714fd03b` with no P0-P3 findings. Both
  compiler-test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate
  CodeQL pass; draft PR #4 is open and mergeable.
- Accepted public `CORE-017` closure head:
  `3dd3bb41d601ddfe5f7ac2722cde39bad124973d`. Three independent reviewers approved
  exact record-only diff `3239da0b313f819bad7beef69cea8b6bd5e658a8` and tree
  `166ec7a5e4156da1cefeb9f921a31714461c6839` with no P0-P3 findings. Both
  compiler-test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate
  CodeQL pass; draft PR #4 remains open and mergeable.
- Accepted public `CORE-017` final-state sync head:
  `9ddc571ac47f1c2ffcf7a737e4be442f01c0f78b`. Three independent reviewers approved
  exact record-only diff `1c5af4fe131ad73eebecc6b17cc2428686ec431e` and tree
  `20ab4e6b87ead659a138e57bc27c073f817d15cb` with no P0-P3 findings. Both compiler-
  test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate CodeQL
  pass in runs `30814906589`, `30814909709`, `30814909985`, and `30814903903`; draft
  PR #4 remains open and mergeable and upstream `master` remains `8f8c733`.
- Accepted public `CORE-015` final-state sync head:
  `c612f3bea133f308cd71c6f8e5fb9ad708e51e6b`. Three independent reviewers approved
  exact staged diff `674b1831accef7b714ba21799249f346cc5a7491` and tree
  `224b9d790115de92d381a956e4487725325140f2` with no P0-P3 findings. Both
  compiler-test jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate
  CodeQL pass; draft PR #4 is mergeable.
- Accepted public closure head:
  `5d7aae0f5626813249b6de983a229dbbb1e4fef8`. Three independent reviewers approved
  exact closure-record diff `a8e4059e71991c9d7a274234f91dd225bea61c01` and tree
  `19fea4153397958656b57adac6b70556d4a997c9` with no P0-P3 findings. Both public
  compiler-test jobs, stable and nightly Rust, all three CodeQL analyses, and
  aggregate CodeQL pass.
- Accepted public `CORE-015` implementation head:
  `3f0578d69926e15a81c4d8fa6105c99c982cbe02`. Three independent reviewers approved
  exact staged diff `3a909f5813def06d4f7cfb27f8650908410ac724` and tree
  `3effac84a84d56f43abcf99c65161c3da7753d6e` with no P0-P3 findings. Both public
  compiler-test jobs, stable and nightly Rust, all three CodeQL analyses, and
  aggregate CodeQL pass. The accepted closure record above completes this evidence.
- Public `CORE-015` tests-only red checkpoint:
  `b203ea429b5a039705be5a5b11998e6dc59f5a24`. Three independent reviewers approved
  its exact staged diff `e158ad61282617a63dade4976a7c23fe53aa0af8` and tree
  `db2ac2959f9815fab5d4b649e563b59c83459dfe` with no P0-P3 findings. Both public
  compiler-test jobs and Rust nightly reproduce exactly the new target's 8 passing /
  8 intended failing split; Rust stable is matrix-cancelled after nightly failure.
  All three CodeQL analyses and aggregate CodeQL pass. This is red evidence, not an
  accepted implementation.
- Public `CORE-015` preregistration checkpoint:
  `4f31f0ca3941389f2cc730136c2540301ee5bfe0`. Three independent reviewers approved
  its exact staged diff `9316f77aed456729624c2d86afaf7110487af84b` and tree
  `bd782da2b5881c1eb50a614400d73b1bb924b033` with no P0-P3 findings. All eight
  public checks pass and the draft PR is cleanly mergeable.
- Prior accepted `CORE-014` closure head:
  `1535ce2a214f512c140535e7c42799af1f920d5c`. Its exact reviewed staged diff is
  `6e05c26763ed3a1c6e4ec359361867f76e9d4c4c` and tree is
  `b3a6bf38769579dbfc0fa0da5c4881620f7129c3`. Three independent reviewers
  approved it with no P0-P3 findings after two backend-evidence precision
  corrections; all eight public checks pass and the draft PR is cleanly mergeable.
- Accepted `CORE-014` implementation:
  `c56b1d561930a042eeff214196fd1b4f05a77fb6`. Its exact reviewed staged diff is
  `687dd5f3d6360dfd7822e7809944f63d4caccfdd` and tree is
  `869fca43edb8b5888bdec01d0bfc7cdecfa451a5`. Three independent reviewers
  approved it with no P0-P3 findings, the focused 5/5 target and exact complete
  local gate pass, and all eight public checks pass. Stable Linux CI resolved the
  documented LLVM 22 tools, completed build/init/check/run, returned status zero,
  and observed exactly one anchored `Output: Hello, Aero!` line.
- Historical `CORE-014` tests-only red checkpoint:
  `fc77e9979f996aaa0110ba48246b24ebca67acbd`. Its three intended Quick Start
  contracts failed after all earlier public test steps passed; this remains red
  evidence, not an accepted implementation.
- Prior clean `CORE-013` acceptance closure:
  `18526ff7a80db222c1348496f24f710d09249dfc`. All eight public checks pass.
- The `CORE-014` red checkpoint's exact reviewed staged diff is
  `b02c2bad25a28ec069303c02fa39de68b64561e8` and tree is
  `f301087d2749d4425bc7d913b3109b1b7aab64e2`. Three independent reviewers
  approved it with no P0-P3 findings. Focused local evidence was two controls
  passing and exactly three intended documentation/workflow failures. Public
  compiler-test and stable Rust jobs reproduced those same failures; the unchanged
  nightly matrix job was cancelled by matrix fail-fast after stable failed.
- Accepted `CORE-013` implementation code commit:
  `a78dd004aa37c39212711027b777698118d9dc02`. All eight implementation checks pass.
- Prior `CORE-012` acceptance-documentation head:
  `b7bb42958e78fb97ea0d991fa3f4cdb40bbcce2f`.
- Earlier published project-control checkpoint:
  `c0c044256a5922605e0dde8446b4c40cb250fd56`.
- Published `CORE-012` tests-only red checkpoint:
  `57c4ec70190822cb4552d313e5e7ea0f2dc5cbed`; exact staged diff
  `4058775145e68aa9a5512853c04b0dde04730464`, tree
  `227254ef8177d8e15b69c42bd1e2d94c1442879a`. Three independent reviewers
  approved the snapshot with no P0-P3 findings. Direct registry evidence was
  7 pass / 5 intentional failures; the CLI matrix was 0 pass / 6 intentional
  failures. The full gate stopped at 134 pass / the same five intended failures.
- Published accepted `CORE-012` implementation:
  `6780a23cd8b63df124477c7db1190d61dd25f3b8`; exact reviewed diff
  `05e55496f6664713192b2dbf94eca785abe2931d`, tree
  `85ed76ab0141409796e167704e4100dd4d15c26f`. Direct registry tests pass 12/12 in
  both library and binary targets; CLI quarantine/local/dry-run/help tests pass 7/7.
  The complete `./tools/test.sh` gate passes 139 library, 148 binary, every active
  integration suite, formatting, correctness Clippy, and doc tests; 38 pre-existing
  Phase 5 tests remain explicitly ignored. Three independent reviewers approved the
  exact snapshot with no P0-P3 findings. Both compiler-test workflows, Rust stable/
  nightly, all CodeQL language analyses, and aggregate CodeQL pass publicly.
- Published accepted `CORE-011` head:
  `a711dd5f3802095a4ecbe2dea3d45003675e7459`; exact reviewed implementation
  diff `60fe607413ebc03e9aa5d6296d9067d8cc95d89d`, tree
  `7c57c082e9d5f68afd5c6a4769d9d531a0116642`.
- Published `CORE-011` tests-only red checkpoint:
  `9c31820fdc5a252e29d5c62c96ff89f5a4a63eb8`; exact staged diff
  `badb9d0e8d6059927d949994b39f617fe2f404a8`, tree
  `540a187db87aff5ec0b2964b0c140c6caf9402a4`. Three independent reviewers
  approved the snapshot with no P0-P3 findings. Local red evidence was 2 pass / 5
  intentional failures in the module matrix and 3 pass / 4 intentional failures
  in the cache matrix; the full binary suite had only those four intended failures.
- Accepted `CORE-011` implementation: the shared collector is crate-private,
  every preregistered file-backed caller uses it, source-only `compile_program`
  rejects `mod`, nested declarations fail explicitly, module collection precedes
  cache lookup, and the frozen V1 identity excludes host paths. Both focused
  seven-test suites and the complete `./tools/test.sh` gate pass. Three independent
  reviewers approved the exact implementation snapshot with no P0-P3 findings.
  Both compiler-test jobs, Rust stable/nightly, all CodeQL language analyses, and
  the aggregate CodeQL check pass at the public implementation head.
- Published accepted `CORE-010` head:
  `db349ef81f145ee571c053f73fb03c831cea719a`.
- Checked-IR/LLVM-verifier implementation commit:
  `d08653c646edae33693f91e2b2f446c76f1bd8a6`; exact reviewed staged diff
  `9534765a46b130d215a1d1e869de234163bb0daf`, tree
  `e0e720f398b1201b4d798101eea4059fc1de56b2`.
- Linux mixed-entry CI repair: exact reviewed staged diff
  `d5f0fd3891da5cff75bd5306006e993ca4b4f301`, tree
  `782b4d5319d73248bee825683e403b8265eb4fbc`, integrated as
  `db349ef81f145ee571c053f73fb03c831cea719a`.
- Published integration head / accepted `CORE-010` red checkpoint:
  `26560a45905015b7891ddebeb749d0097c05cbaa`.
- Founding-framework alignment is published at
  `fba121f0213b7f604d4c73032019c872680a3136`.
- `CORE-009` tests-only red checkpoint:
  `1e76a0610ef778303548096ef634a5f02b678fe9`. The new nine-test aggregate suite
  is exactly 3 pass / 6 expected fail on production: parser/positive controls and
  established child precedence pass, while ordinary, recursive, default/nested,
  ordering/inference-only, root CLI, and direct-module CLI families expose false
  success, wrong outer diagnostics, zero/drop lowering, successful status, and
  requested artifacts. No failure relies on a parse error or unwind. Reclassified
  controls independently pass 59 frontend, 8 field, and 15 Match tests; formatting
  and diff checks pass.
- `CORE-009` production candidate: owner `bf6a7ef`, integrated as exact
  `a8879310fe04a28b368437d1932e01972b7e9cee`. The only production change is one
  return after existing source-order recursive StructLiteral field preflight. The
  exact diagnostic is `Struct construction expressions are not supported.` Owner
  verification passes the complete gate plus the focused matrix. Lead verification
  independently passes 9 Struct, 59 frontend, 8 field, 15 Match, 16 tuple, 14
  modulo, 13 function-contract, 18 numeric-annotation, and 12 strict-lexing tests.
  Public documentation and the complete gate pass at `3410f1f`; coordinated
  project-control corrections and the new exact-candidate gate pass at `daa024d`.
  Two fresh non-owner reviewers approve exact `daa024d` with no P0-P3 findings.
- `CORE-009` closure is published at exact
  `555fea27e6cb8e0a07df20b5189dfc2b5aebce46` on draft PR #4. Both compiler-test
  jobs, Rust stable/nightly, and all CodeQL jobs pass at that public head.
- `AUDIT-016` is complete. Fresh evidence ranks fallible typed scalar IR admission
  and verification above pipeline consolidation, MethodCall, custom EnumVariant,
  and Deref slices. String comparison and constant `1 / 0` unwind; Boolean storage
  can emit type-invalid LLVM; untyped codegen can silently ignore instructions.
- `CORE-010` and `DEC-015` define the checked additive APIs, logical
  Int/Float/Bool/Void representation, legacy numeric-storage compatibility limit,
  stronger tool-independent `check` admission, mandatory pure-Rust IR verification,
  and LLVM 22 external verification modes. The accepted public implementation now
  enforces that contract on trusted paths.
- The isolated `CORE-010` tests/CI-only red checkpoint is published at exact
  `26560a45905015b7891ddebeb749d0097c05cbaa`; its exact staged diff hash is
  `c01fc2365eb5b415c022be997062e4605812b62b`. Three independent reviewers approve
  that exact diff with no P0-P3 findings. Local evidence records typed admission as
  1 pass / 7 intentional failures and the external LLVM CLI matrix as 3 pass / 9
  intentional failures; the remaining checked public/private targets stop only on
  the preregistered missing API and injected-seam symbols. Parser/declaration
  controls for reclassified unsupported forms remain green.
- Public CI confirms the environment/corpus side of the checkpoint. Both compiler
  workflows install LLVM 22 and prove `opt-22` rejects the known-invalid fixture.
  Rust stable/nightly install LLVM 22 plus Clang 22 and pass all four CPU example
  verification/execution steps, with `opt-22` preceding `llc-22`/`clang-22`. Test
  jobs then fail at the deliberate checked-API contract boundary. This is accepted
  red evidence, not an accepted production candidate.
- The accepted `CORE-010` production implementation makes all focused typed-admission,
  checked-IR, LLVM-verifier, cache, conformance, profiler, and compatibility
  controls green. `cargo check --all-targets` and the complete `./tools/test.sh`
  repository gate pass. External verification is enforced after final graph/target
  transformation and before cache/write/native publication; tool-independent
  `check` and conformance remain internal-verifier paths. Three independent
  reviewers approved the exact implementation and CI-repair diffs with no P0-P3
  findings. Rust stable/nightly, both compiler-test workflows, and all CodeQL jobs
  pass at public head `db349ef`.
- Accepted `CORE-008` candidate:
  `b74d91adeda04688ec37598beebffad458538c39`. All trusted parsed source bodies,
  including default trait method bodies, route Match expression roots through the
  existing child-first preflight before IR. The complete gate and two fresh
  independent reviews pass.
- Accepted `CORE-009` candidate and complete gate: exact clean
  `daa024dbf10d1defe06d8ab200c2d21c0a9c1dc6` passes 112 library, 119 binary,
  11 fatal-parser, 59 frontend, 13 function-contract, 18 numeric-annotation,
  12 strict-lexing, 8 field, 15 Match, 14 modulo, 9 StructLiteral, and 16 tuple
  tests. All 38 Phase 5 tests remain intentionally ignored. Formatting, Clippy
  correctness, all-target compilation, and doc tests pass.
- Current accepted public full-gate implementation commit:
  `aca3fe21ece4a7f90de0b41b5e336c15ac589505` (`ARCH-002`, behavior-neutral).
- Previous public implementation record: `CORE-015` changed only the two
  preregistered production phases, `src/compiler/src/semantic_analyzer.rs` and
  `src/compiler/src/ir_generator.rs`, plus the focused test and these minimal evidence
  records. The focused test adds implementation-review regression controls for
  numeric-array child ordering, single-pass deep nesting, nested index traversal, and
  stub-only method/closure/format/custom-enum boundaries. Several would reject the public red
  implementation, but remain inside its already-failing semantic group and do not
  change the published 8/8 group outcome. It also corrects one green-side assertion
  from the noncanonical `Semantic Error:` fragment to Aero's frozen public
  `Semantic Analysis Error:` phase prefix; that assertion was unreachable while the
  red cases were false accepts. Semantics
  now balances generic scopes on every
  exit, enforces the closed binding selector, and validates numeric-array elements
  and indexes outside generic scopes. Checked admission independently enforces the
  direct/non-generic selector and binary metadata while preserving generic-impl
  annotation quarantine. The focused target passes 16/16. The exact
  `./tools/test.sh` gate passes formatting, correctness Clippy, 139 library tests,
  148 binary tests, every active integration target, and doc tests; 38 pre-existing
  Phase 5 tests remain ignored. Three exact implementation reviews approved diff
  `3a909f5813def06d4f7cfb27f8650908410ac724` / tree
  `3effac84a84d56f43abcf99c65161c3da7753d6e`; public commit `3f0578d` passes the
  complete CI matrix. Three fresh closure reviewers approved exact record diff
  `a8e4059e71991c9d7a274234f91dd225bea61c01` / tree
  `19fea4153397958656b57adac6b70556d4a997c9`; public closure commit `5d7aae0` also
  passes all eight checks. `CORE-015` is accepted at that closure head; its four-record
  final-state sync is public and green at `c612f3b`.
  One earlier full-gate attempt stopped in the unchanged
  `cli_status_contract_tests`; that target immediately passed 7/7 in isolation and
  the unchanged complete gate passed on rerun. The interruption is not reproduced
  or attributed to `CORE-014`, but remains residual pre-existing flake uncertainty.

## Environment and verification

- Host: Windows x86_64
- Shell for baseline gate: Git Bash launched from PowerShell
- Rust: `rustc 1.97.1 (8bab26f4f 2026-07-14)`
- Cargo: `cargo 1.97.1 (c980f4866 2026-06-30)`
- Required command: `./tools/test.sh`
- Windows invocation used: prepend `C:\Users\usa50\.cargo\bin` to `PATH`, then
  run Git Bash with `./tools/test.sh`.
- Baseline result: PASS on the starting commit; formatting, Clippy correctness,
  unit/integration tests, and doc tests completed with no test failures.
- Initial environment issue: Rust was absent and the first two baseline attempts
  stopped at `cargo: command not found`; the stable minimal toolchain plus
  `rustfmt` and `clippy` were installed, then the gate passed.
- LLVM tools in the Windows environment: `clang`, `llc`, `opt`, and `llvm-as`
  unavailable on discovered paths.
- `CORE-010` red CI at `26560a4`: pinned LLVM 22 installation and known-invalid
  rejection pass in both compiler jobs; stable/nightly LLVM verification and native
  execution of `return15`, `variables`, `mixed`, and `float_ops` pass. The subsequent
  Cargo test failures are confined to the intentional missing checked APIs and
  private injected seams recorded by the red checkpoint.
- Upstream recheck before `CORE-001`: `origin/master` still equals the recorded
  starting commit.
- Remote CI: CI run `26611985062`, Rust CI run `26611985038`, and latest CodeQL
  run `30685526232` all completed successfully for upstream commit
  `8f8c7337a4008082fd2a443fcc814b5847b8663f`.
- `CORE-001B` verification at `6ce85922`: focused fatal-parse tests 11/11;
  complete gate PASS with 106 library, 111 binary, and 59 frontend tests; 38
  pre-existing phase-five tests remain ignored.
- Fresh manual root and imported-module builds both exited 1, reported the source
  file at `1:5`, and created no requested LLVM artifact.
- `CORE-002` verification at `b988318`: full gate PASS with 111 library, 118
  binary, 11 fatal-parse, 12 strict-lexing, and 59 frontend tests; 38 pre-existing
  phase-five tests remain ignored. Conformance remains 3/3 cases and 4/4 checks.
- Fresh manual unexpected-character and overflow builds, plus direct-module docs,
  exited 1 with located lexical errors and created no requested output.
- `CORE-003` verification at `8d5d8e7`: focused function-contract tests 13/13;
  full gate PASS with 112 library, 119 binary, 11 fatal-parse, 59 frontend,
  13 function-contract, and 12 strict-lex tests. The 38 pre-existing phase-five
  tests remain ignored. Two independent reviewers approved the exact clean SHA;
  fresh black-box invalid-program probes exited nonzero and wrote no LLVM artifact.
- `CORE-004` verification at `bc9a148`: focused numeric annotation and lexical-scope
  tests 18/18; function-contract tests 13/13; full gate PASS with 112 library,
  119 binary, 11 fatal-parse, 59 frontend, 13 function-contract, 18 annotation,
  and 12 strict-lex tests. The 38 pre-existing phase-five tests remain ignored.
  Two independent reviewers approved the exact clean SHA after public no-unwind,
  scope-provenance, artifact, callable-restoration, and analyzer-reuse probes.
- `CORE-005` verification at `302211e`: focused modulo tests 14/14;
  function-contract tests 13/13; annotation tests 18/18; full gate PASS with
  112 library, 119 binary, 11 fatal-parse, 59 frontend, 13 function-contract,
  18 annotation, 12 strict-lex, and 14 modulo tests. The 38 pre-existing phase-five
  tests remain ignored. Two non-owner reviewers approved the exact clean SHA after
  fresh shared-helper and public/CLI diagnostic, no-unwind, no-panic, module,
  precedence, nonnumeric, unary, nested, positive-control, and artifact probes.
- `CORE-006` verification at `cbbe049`: focused tuple tests 16/16; modulo 14/14;
  function-contract 13/13; annotation 18/18; strict-lex 12/12; complete gate PASS
  with 112 library, 119 binary, 11 fatal-parse, 59 frontend, 13 function-contract,
  18 annotation, 12 strict-lex, 14 modulo, and 16 tuple tests. The 38 pre-existing
  phase-five tests remain ignored. Two non-owner reviewers approved exact clean
  `cbbe049` after fresh structural and 18-route black-box public/CLI diagnostic,
  no-unwind, no-panic, no-artifact, nesting, precedence, and positive-control probes.
- `CORE-007` verification at `4e10d479`: focused field tests 8/8; tuple 16/16;
  modulo 14/14; function-contract 13/13; annotation 18/18; strict-lex 12/12;
  complete gate PASS with 112 library, 119 binary, 11 fatal-parse, 59 frontend,
  13 function-contract, 18 annotation, 12 strict-lex, 14 modulo, 16 tuple, and
  8 field tests. The 38 pre-existing phase-five tests remain ignored. Two non-owner
  reviewers approved exact clean `4e10d479` after independent 25-route public and
  27-route public/CLI/module matrices, plus structural, nesting, precedence,
  no-unwind, no-panic, no-artifact, parser-distinction, and positive-control probes.

## Audit agents

- `AUDIT-001` specification: complete; 3 high, 6 medium, 1 low finding.
- `AUDIT-002` frontend: complete; silent lexical corruption, lost semantics/spans,
  recovery/API panic risks, and dormant coverage confirmed.
- `AUDIT-003` type soundness: complete; active integer/double/zero fallbacks,
  unenforced contracts, ownership/generic gaps, and scope leakage confirmed.
- `AUDIT-004` IR/code generation: complete; untyped values, invalid boolean/CFG
  lowering, parse-to-invalid-LLVM false success, and absent verification confirmed.
- `AUDIT-005` runtime/backends: complete; CPU execution path separated from ROCm
  object plumbing and absent CUDA run; graph/quantization claims reclassified.
- `AUDIT-006` tooling: complete; duplicate pipelines, ignored options, status-code
  failures, heuristic LSP, shallow modules, and registry risks confirmed.
- `AUDIT-007` tests/fuzzing: complete; duplicates, 38 ignored, 299 dormant, and
  absent compile-fail/fuzz/differential/verifier/hardware gates inventoried.
- `AUDIT-008` benchmarks/claims: complete; compilation series invalid, lexer
  evidence partial, GGUF external/single-run, and protocol gaps classified.
- `AUDIT-009` numeric binding boundary: complete after two review amendments;
  parser retention, semantic/IR discard, seven black-box false-accept families,
  unified-double local storage, IR scalar leakage, and semantic compatibility-table
  leakage are characterized and the eligible slice is controlled at `bc9a148`.
- `AUDIT-010` unsupported-expression boundary: complete; three independent audits
  agree that `%` is the smallest bounded fail-open family. Five numeric forms pass
  semantic `check` then panic in both public/CLI compilation. Constant integer `/0`,
  unsupported comparisons, and invented-zero aggregates/methods are separate tasks.
  The selected `%` boundary is controlled at `302211e`.
- `AUDIT-011` fabricated-zero expression families: complete at `704b3328`. Three
  independent read-only audits compared fields, tuples, matches, methods, closures,
  arrays, structs, enums, borrows, and nested forms. Two selected tuple literals and
  tuple projections as the smallest coherent family; one ranked field access first
  by AST-form count but confirmed the tuple family's shared zero behavior. The lead
  selected tuples because `(7, 9).0` is a valid specification-backed value expression
  that silently emits zero, while field access intersects broader struct semantics.
- `CORE-006` tuple value boundary: accepted at exact clean `cbbe049`. Both
  non-owner reviewers approved after independent structural and black-box probes;
  trusted public/CLI routes reject tuple literals/projections before IR with one
  exact diagnostic, no unwind/panic, and no requested artifact.
- `AUDIT-012` adjacent failure boundaries: complete at clean `52d3415`. Three
  independent audits compared field access, all-six string comparisons, and
  constant/variable/float/mixed zero division across semantics, both IR paths,
  public compilation, CLI, modules, artifacts, nesting, and controls. All ranked
  FieldAccess first as the only one-node silent-miscompile family with no active
  value-preserving path and no required layout/arithmetic policy.
- `CORE-007` field value boundary: accepted at exact clean `4e10d479`. The
  tests-only red checkpoint is `7346edd`, the one-line receiver-first production
  behavior is `75dbfba`, and user-facing field status and the matrix are corrected
  at `5dcb70b`. Both non-owner reviewers approved after independent structural and
  black-box attempts to falsify the complete-gate candidate.
- `AUDIT-013` next-boundary comparison: complete at exact clean `9fc7d0e`. String
  comparison requires trustworthy operand typing and operator policy; MethodCall
  rejection requires a pre-IR capability predicate that preserves real array
  `.iter()`; zero division requires integer/runtime/IEEE policy. Match alone has one
  AST family, complete existing recursive preflight, no active value-preserving
  path, and no required execution semantics. A 23-case Match matrix produced 69
  public/check/build outcomes: 20 false successes and three established child
  diagnostics with retained precedence.
- `CORE-008` Match value boundary: preregistered at audit closure `648662b` under
  `DEC-012`. Exact diagnostic is `Match expressions are not supported.` Existing
  child-first scrutinee/arm traversal and tuple/field/void diagnostic precedence are
  frozen. Parser/AST/patterns remain; pattern/exhaustiveness/type/layout/ownership/
  evaluation/IR/backend semantics are explicitly outside the slice.
- `CORE-008` tests-only red checkpoint: owner `17e17c2`, integrated as `851731c`.
  Focused result is exactly 5 pass / 4 expected fail. All 21 ordinary Match forms
  falsely compile; 12/15 recursive parents falsely compile; root and direct-module
  check/build exit zero and create artifacts. Failure evidence records fabricated
  zero, empty root CFG, dropped calls, suppressed `/0`, current outer diagnostics,
  and artifact creation. No case parses incorrectly or unwinds. Parser 1/1 and prior
  field/modulo/tuple controls 38/38 pass; formatting passes.
- `CORE-008` production candidate: owner `aed4d0e`, integrated as `c826294`. The
  production diff is one error return after existing Match child traversal. Owner
  and lead independently pass 90/90 focused Match/field/tuple/modulo/function/
  annotation/strict tests, formatting, and `cargo check --all-targets`. Public docs
  and the matrix now classify Match as parsed but explicitly non-executable; two
  historical design summaries carry current-capability notices.
- `CORE-008` initial full gate: exact clean `08e7c2c` passed the complete repository
  gate (112 library, 119 binary, 11 fatal-parser, 59 frontend, and all focused
  boundary suites; 38 Phase 5 tests ignored), documentation, formatting, and Clippy.
- `CORE-008` initial review: rejected. Reviewer A found the sole parsed
  expression-bearing container escape: `TraitMethod.body` is retained by the parser,
  while `Statement::TraitDef` registers only required names and never visits default
  bodies. Match in such a body returns `Ok` through `compile_program`, check, and
  build, and build writes LLVM. Reviewer A passed 32/33 fresh routes and 7/7 frozen
  precedence probes. Reviewer B approved its independent 41-route matrix but did not
  include trait defaults; that approval is superseded by the counterexample.
- `CORE-008` corrective red: owner `58bb732`, integrated as `ad5e24d`. Six new
  aggregated tests preserve an exact 11-pass/4-fail red split on rejected production,
  covering eight default-body placements, tuple/field/void precedence, root/module
  CLI no-artifact contracts, parser retention, and syntax-only positive controls.
- `CORE-008` corrective production: owner `a3f4f29`, integrated as `a12f38e`. Only
  `semantic_analyzer.rs` changes: exhaustive syntax-only block/statement preflight
  plus a default-body hook with cleanup-safe type-parameter scope handling. Owner
  and lead pass 96/96 focused tests; formatting and all-target compilation pass.
- `CORE-008` corrective acceptance: exact clean documented `b74d91a` passes the full
  gate (112 library, 119 binary, 11 fatal-parser, 59 frontend, 13 function, 18
  annotation, 12 strict, 8 field, 15 Match, 14 modulo, 16 tuple; 38 Phase 5 ignored).
  Reviewer A approves after the exhaustive structural audit and 44 fresh public
  negatives; Reviewer B independently approves 225/225 outcomes across 75 negative/
  precedence routes. Syntax-only positives, parser retention, child order, no-panic/
  no-unwind/no-artifact behavior, and prior controls all pass.

## Current capability classification

Initial audit classification; see `CURRENT_CAPABILITY_AUDIT.md` and
`SPEC_IMPLEMENTATION_MATRIX.md` for stage evidence:

- Compiler regression baseline: passing locally.
- Repository stability: experimental.
- Formal conformance: three example cases plus four deterministic pipeline
  checks; this is not formal semantics proof.
- CPU source-to-LLVM/object/link/process path: present when external tools are
  available; current evidence is four small Linux CI exit-code programs plus the
  generated-project status/output path accepted by `CORE-014`.
- ROCm: interface/retarget/object-generation plumbing; no link/launch path or
  current-session hardware execution evidence.
- CUDA: selectable interface; CLI source states run support is not implemented.
- Public version: compiler CLI/banner presentation is manifest-derived package
  `0.3.0`; language `v1.0.0` material is a design target, not current conformance,
  stability, compatibility, or release evidence (`CORE-016`, `ea036f2`).
- Library compiler options: accepted `CORE-020` preserves defaults and rejects
  nondefaults before lexing; option meanings remain unimplemented.
- Compiler architecture: accepted public CORE-081 removes the exact 35-module binary/
  library overlap; broader tool-path convergence remains pending.

## Known blockers and regressions

- No known baseline regression.
- The local shell required Rust installation before tests could run.
- Real backend verification may be blocked by absent LLVM/GPU toolchains or
  hardware; absence will be recorded rather than simulated.
- Accepted `a4327be` removes the false CPU delegated-nonzero success line while
  preserving exact child status/output/cleanup. `run_aero_program` still calls `exit`
  internally after cleanup, and that separate helper/API architecture boundary
  remains open.
- Accepted `CORE-022` at `2a42324` uses final-entry, non-following preflight before
  any `aero init` create/write and prevents the reproduced dangling-source partial
  manifest. General rollback, atomicity, ancestor-symlink policy, and race freedom
  remain open.
- Legacy recovery lexing remains public for compatibility and LSP symbol recovery;
  trusted repository paths no longer feed it into semantics, IR, or artifacts.
- Numeric and void top-level function contracts are controlled at `8d5d8e7`.
  Initialized exact numeric `let` annotations are controlled at `bc9a148`;
  uninitialized, non-numeric, boolean/generic/composite contracts remain open.
- Function-local branch and epilogue termination improved in `CORE-003`, but the
  broader pre-existing unreachable-after-terminator CFG risk remains open.
- The tested scalar/callable IR scope exits and semantic compatibility scopes are
  controlled at `bc9a148`; general AST-to-IR fallibility, unsupported-expression
  fallbacks, and analyzer/backend invariants remain open.
- At `302211e`, `%` remains parsed but is rejected by shared semantic inference
  before IR with one stable diagnostic across trusted public and CLI paths.
  Negative/float/zero remainder execution semantics remain intentionally undecided.
- At `1fa67a2`, tuple literals and tuple projections remain parsed but are rejected
  recursively by active semantic preflight with one stable diagnostic before IR.
  Tuple types/patterns remain parsed; tuple layout and execution remain unimplemented.
- `CORE-010` now turns constant integer division by zero, string comparison,
  ordinary MethodCall, custom enum construction, Deref/Borrow, and other unsupported
  scalar fallbacks into checked errors on trusted paths; typed Boolean storage/calls/
  returns and checked IR/codegen verification are controlled at accepted `db349ef`.
  Dynamic division/overflow, aggregates, ownership, and direct callers of public
  unchecked compatibility APIs remain uncertified.
- At candidate `3410f1f`, StructLiteral values remain parser-visible but are
  rejected after source-order field preflight before inference or IR. Struct
  layout, initialization, ownership, ABI, lowering, and execution remain open.
- At accepted compiler head `a711dd5`, a root-level missing direct module fails
  every inventoried trusted file-backed route before cache lookup or publication;
  source-only `compile_program` rejects `ModDecl`, and nested modules fail explicitly.
  Module-bearing cache identity includes the exact ordered direct-source set while
  retaining the legacy no-module key. General CLI status handling, namespaces,
  visibility/import semantics, recursive graphs, and full pipeline consolidation
  remain separate.
- Accepted public `CORE-013` at `a78dd00` contains the false-success CLI boundary.
  The tracked
  `performance_benchmark.py` compilation timings are an invalid measurement of a
  bare-source usage path; public and historical lexer evidence remain separately
  qualified, and the external GGUF record remains reference-only.
- `AUDIT-021` at clean public head `1535ce2` reproduced active R-002 false success:
  initialized `String`, `bool`, custom named, fixed-array element-type, and
  fixed-array length annotation mismatches all pass `check` and `build`, with a
  requested LLVM artifact. Exact String/bool and homogeneous-array controls pass.
  An uninitialized read fails in semantics. Mixed `[1, 2.5]` and float indexing
  fail closed without artifacts, but only after semantic success. R-004 remains
  stopped because the mutable-reference Copy/provenance defect requires unfrozen
  ownership work across more than two phases.
- Accepted `CORE-015` at `5d7aae0` closes its four selected active false successes;
  the final records are public and green at `c612f3b`. `AUDIT-022` at that clean head
  reproduces R-008: package `0.3.0` versus CLI/banner `1.0.0`, deterministic reruns
  presented as mechanized/formal proof, and current-facing README, CLAUDE, tutorial,
  generic, and ownership safety claims beyond implementation evidence. Residual
  R-002 custom/contextual annotation work is stopped on unresolved nominal/generic
  meaning; remaining R-011 needs typed aggregate execution work. The ignored Phase 5
  backlog remains open:
  an explicit 38-test ignored run passes 36 and fails 2 but contains recovery/stub
  assumptions that prevent bulk activation.

## CORE-032 accepted implementation

- Corrected authorization `449f3536` is triple-approved and public all-eight green
  in compiler `30885443132` / `30885447315`, Rust `30885447416`, CodeQL
  `30885443837`, and aggregate `91915624793`.
- Corrected tests-first `35eac8c4` is triple-approved and publicly fails only the
  named 20/21 binding regression with exactly eight acceptances in compiler
  `30886282169` / `30886283814` and nightly Rust `30886284165`; stable is fail-fast
  cancelled. CodeQL `30886281888` and aggregate `91918210639` pass. Rejected
  `1afe11d3` was never published because it omitted explicit array-literal target
  coverage.
- Implementation `30d0d730`, tree `653346ce`, canonical diff `01e87768`, is
  triple-approved and public all-eight green in compiler `30886856260` /
  `30886858878`, Rust `30886858960`, CodeQL `30886856518`, and aggregate
  `91919998289`. The first local full-gate attempt is preserved as an unexplained
  truncated exit-1 result; two consecutive unchanged-tree full reruns passed.
- This is fail-closed containment only. R-002 remains HIGH/CRITICAL and PARTIALLY
  CONTROLLED; tuple/array values, compatibility, layout, ABI, ownership, raw APIs,
  valid-output certification, backends, matrix cells, and capability classes do not
  move.
- First closure snapshot `7d7fe3d6`, tree `18c904fd`, canonical diff `407c3c86`,
  passed its exact full gate but was rejected unpublished by all three reviewers:
  it incorrectly left the completed closure gate as future work, and the type review
  also required the known implementation-gate status to remain exact as exit 1.
  Second snapshot `48f2fd60`, tree `86175cc1`, canonical diff `9f0ab102`, resolved
  those findings and received two approvals but was rejected unpublished at P3 by
  the type reviewer because the successful closure gate omitted literal `exit 0`.
  The twice-corrected six-record tree records both rounds, and its fresh exact gate
  exits 0 with 139/139 library, 149/149 binary, 7/7 doc, and 21/21 binding tests.
- Exact closure `9c82cbfc`, tree `b2a106ee`, canonical diff `fc672744`, received
  three exact approvals, was published unchanged, and passes compiler
  `30888222316` / `30888225734`, Rust `30888226011`, CodeQL `30888222480`, and
  aggregate `91924197947`.

## AUDIT-039 authorization

- Basis: exact clean public CORE-032 closure `9c82cbfc`; complete residual set
  R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016.
- Method: three independent complete evidence-cited rankings; exclude every accepted
  slice through CORE-032; name one bounded candidate or stop with reachability,
  containment, semantic choices, phase count, deterministic failing specimen, and
  preservation controls; inherit no earlier candidate/order.
- Boundary: after authorization acceptance, ranking is read-only: no edits, tests,
  builds, formatters, probes, artifacts, hardware action, or external query. It
  grants no test, implementation, semantics, capability, matrix, risk, workflow,
  dependency, backend, claim, history, or `master` authority.
- Gate: the prepared six-record authorization's fresh exact full gate exits 0 with
  139/139 library, 149/149 binary, 7/7 doc, and 21/21 binding tests. Ranking began
  only after those gates and is complete below.

## AUDIT-039 result and CORE-033 authorization

- Exact AUDIT-039 authorization `fa522b2c`, tree `365a536d`, canonical diff
  `cefb797e`, is triple-approved and public all-eight green in compiler
  `30888751268` / `30888754238`, Rust `30888754262`, CodeQL `30888752230`, and
  aggregate `91925849313`.
- All three complete rankings put R-002 first. Type/safety initially selected the
  valueless three-array form historically labeled Candidate T; IR/codegen and backend selected initialized
  two-array Candidate A. Preference comparison favored A two to one. The lead
  provisionally selected A for its smaller predicate/count/test surface and frozen
  initializer ordering; all three explicitly approved exact A at the final
  compatibility gate. The audit changed nothing.
- CORE-033 freezes initialized exact nonrecursive `Array(Array(Tuple))` rejection in
  semantic and checked admission only. Duplicate/RHS/Void/existing diagnostics stay
  first; generic impl is covered; semantic generic-function traversal is covered;
  checked generic function retains its outer rejection. Candidate T, the reference-
  array form historically labeled Candidate B, other three-plus depth, wrappers,
  raw/verifier/codegen, ABI/ownership, valid output, and backends are preserved.
- Tests-first reclassified both existing Candidate A acceptance rows and added one
  aggregate with exactly 12 false acceptances: 8 count/phase, 1 public, 2 generic
  impl, and 1 semantic generic function. The checked generic-function result stayed
  green. Corrected tests `ac4cb2a5` reproduced that exact public-red surface.
- Corrected authorization `66207215`, tests-first `ac4cb2a5`, and implementation
  `76a6e802` are complete at their recorded review and public gates. Implementation
  is limited to the exact semantic and checked-admission guards. Final closure
  `1ee9c71` is triple-approved and public all-eight green in compiler `30893527220`
  / `30893529999`, Rust `30893529992`, CodeQL `30893527445`, and aggregate
  `91941079083`; CORE-033 is complete.
- First authorization snapshot `d0500865`, tree `d2378320`, canonical diff
  `97a15c9f`, passed its local gate but received one approval and two blocking reviews
  because one ledger sentence mislabeled Candidate T's valueless form as Candidate B.
  It remained unpublished; the corrected records keep both historical identities
  unambiguous.
- First closure snapshot `fe90f583`, tree `90ac8ae6`, canonical diff `89fe6824`,
  passed its exact gate and received two approvals but was rejected at P1 before any
  independent push or branch-head publication because this section still described
  tests-first as mandatory future work and said tests/implementation had not begun.
  First correction `19f688a`, tree `9d9c642f`, canonical diff `f885588c`, records
  accepted chronology, passed the exact gate, received three approvals, and was
  pushed. Compiler `30893002336` / `30893005706`, Rust `30893006634`, CodeQL
  `30893002479`, and aggregate `91939375982` pass.
- Because `19f688a` was committed linearly atop `fe90f583`, its push also made the
  rejected snapshot publicly reachable as an ancestor. The lead identified that
  `19f688a`'s never-published wording was therefore inaccurate, withheld closure
  acceptance, and retained published history. Final additive correction `1ee9c71`,
  tree `d0819881`, canonical diff `7303da47`, stated that ancestry exactly, passed
  its gate, received three approvals, was published unchanged, and passed all eight
  public checks without reopening source or test work.

## AUDIT-040 authorization

- Basis: exact clean public CORE-033 closure `1ee9c71`; complete residual set
  R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016.
- Method: three independent complete evidence-cited rankings; exclude every accepted
  slice through CORE-033; name one bounded candidate or stop with reachability,
  containment, semantic choices, phase count, deterministic failing specimen, and
  preservation controls; inherit no earlier candidate or order.
- Boundary: after authorization acceptance, ranking is read-only: no edits, tests,
  builds, formatters, probes, artifacts, hardware action, or external query. It
  grants no test, implementation, semantics, capability, matrix, risk, workflow,
  dependency, backend, claim, history, or `master` authority.
- Gate/review history: first snapshot `c83ec3a`, tree `bb25e528`, canonical diff
  `c02f71e5`, changed exactly the six control documents and passed its exact gate
  with 139/139 library, 149/149 binary, 7/7 claim, and 22/22 binding tests. Type/
  safety and backend/claim approved, but IR/codegen rejected at P1 because the late
  CORE-033 subsection at lines 1023-1044 still called closure future work. It was
  rejected before publication. The corrected authorization's fresh exact gate exits
  0 with 139/139 library, 149/149 binary, 7/7 claim, and 22/22 binding tests.
- Corrected authorization `7b9ed83b0663c4effcd63d2d9963e21b1416f54d`, tree
  `8dbe975eff15e8b6741c2bd848b90cf1958cdcdf`, canonical diff
  `c4ba110a613064bf06a27ef943b3fc819c049b97`, received three exact approvals and
  was published unchanged. Compiler `30894708169` / `30894713332`, stable/nightly
  Rust `30894713411`, all three CodeQL analyses in `30894708736`, and aggregate
  `91944883143` pass.
- Independent rankings selected valueless exact three-array tuple containment
  (type/safety), initialized exact immediate reference-to-tuple containment
  (IR/codegen), and fixed-array immediate literal bounds containment (backend/claim).
  Targeted comparison ranked reference/three-array/bounds twice and bounds/reference/
  three-array once. The bounds candidate remains stopped on conflicting compile-time
  versus runtime policy; the three-array candidate remains bounded but has the
  larger topology/count burden. All three final compatibility reviews approve the
  exact reference candidate below. The audit changed no file or classification.

## CORE-034 accepted implementation

- Exact behavior: reject only initialized `Statement::Let` annotations exactly
  shaped as `Type::Reference(Type::Tuple(_), _)`, for both reference mutability
  flags and without recursive descent.
- Ordering/diagnostics: semantic duplicate detection and RHS validation remain
  first; checked RHS validation and Void rejection remain first; existing initialized
  outer-, one-array-, and two-array tuple diagnostics stay first. Then semantic emits
  `Error: Variable \`{name}\` uses an unsupported tuple type annotation directly beneath a reference for an initialized binding.` and checked admission emits
  `checked IR binding \`{name}\` uses an unsupported tuple type annotation directly beneath a reference for an initialized binding` before mismatch, insertion, the
  generic-impl bypass, or raw generation. The public semantic prefix is unchanged.
- Context: apply only wherever semantic analysis and checked admission already
  traverse bindings. Generic impl and semantic generic-function bodies are covered;
  checked generic functions retain their earlier outer rejection, and trait default
  bodies remain syntax-only.
- Tests-first: after authorization acceptance, only
  `src/compiler/tests/binding_type_contract_tests.rs` may change. Reclassify, never
  silently delete, the two existing initialized immutable/mutable acceptance rows
  into one new aggregate. It must report exactly 30 unexpected acceptances before
  implementation: direct 4, public 2, top-level 4, generic impl 4, semantic generic
  function 2, and seven recursive/non-generic-impl contexts at both phases 14.
  Checked generic-function outer rejection and all precedence/preservation rows stay
  green. Expected focused result is 0/1 and binding aggregate 22/23 with only the new
  test failing after 139/139 library, 149/149 binary, and 7/7 claim tests pass.
- Later implementation may change only `src/compiler/src/semantic_analyzer.rs` and
  `src/compiler/src/ir_generator.rs`, adding the two exact guards after separately
  reviewed public-red evidence. Raw IR, verifier, codegen, CLI, and backends cannot
  change.
- Preservation: all CORE-025/028/029/030/031/032/033 diagnostics and ordering;
  valueless direct reference rejection; duplicate/RHS/Void precedence; scalar and
  double references; reference-around-array, array-around-reference, generic,
  deeper, and wrapped forms; generic-function checked rejection; trait defaults;
  tuple/reference values, ownership, layout, ABI, raw compatibility APIs, valid
  numeric output, CPU/ROCm/CUDA behavior, and every capability/matrix classification.
- Gate: first authorization snapshot `7d4d7ca`, tree `b633abbb`, canonical diff
  `a901f4dc`, passed its exact full gate with 139/139 library, 149/149 binary, 7/7
  claim, and 22/22 binding tests. IR/codegen and backend/claim approved, but type/
  safety rejected it at P1 because TASK_LEDGER's final status still called that gate
  future work. It remained unpublished. The corrected authorization's fresh exact
  full gate exits 0 with 139/139 library, 149/149 binary, 7/7 claim, and 22/22
  binding tests.
- Corrected authorization `91d2686943ec601877db5ac658a20e590b86f0fb`, tree
  `bd9116b20cf75e84e5bc228757bd41f98e2f609e`, canonical diff
  `19458d5799c6c37cfe9543e8dc8e897d86e3e655`, received three fresh exact
  approvals and was published unchanged. Compiler `30915838213` / `30915838191`,
  stable/nightly Rust `30915839059`, all three CodeQL analyses in `30915834128`,
  and aggregate `92013770932` pass.
- Tests-only `296276f6d0c4d733f28cd82c8245bf805f34d634`, tree
  `9b1ad9d1dd075984adf32cdf8ba5e17e6cdda7a4`, canonical diff
  `79b7ef9d695bdfd616442469519ca6d4fd18525e`, reclassified the two existing
  rows and received three exact approvals. Focused 0/1 and binding 22/23 reproduced
  exactly 30 unexpected acceptances; the full local gate exited 1 only there after
  139/139 library, 149/149 binary, and 7/7 claim tests. Public compiler
  `30916807388` / `30916811627` and nightly Rust `30916810937` reproduce the same
  sole 30-observation failure; stable was cancelled after the matrix failure. All
  three CodeQL analyses in `30916806193` pass; aggregate `92017066864` is skipped.
  Three separate public-red reviews approved implementation authority.
- Accepted implementation `a1ffeaecbe46f04611c818ce5d59d2be26128191`, tree
  `f0088e650336d7705701ad8351dfbe2405f4ff21`, canonical diff
  `7a3fdb11a4f0c4645adcbb35174607f6f327b366`, adds only the two exact guards:
  13 semantic and 10 checked-admission lines. Formatting, focused 1/1, binding
  23/23, and the exact full local gate exit 0 with 139/139 library, 149/149 binary,
  7/7 claim, and 23/23 binding tests pass. Three exact reviews approved it; compiler
  `30917539648` / `30917544307`, stable/nightly Rust `30917537292`, all three
  CodeQL analyses in `30917534448`, and aggregate `92019545168` pass.
- Result: the initialized exact immediate reference-to-tuple false-success surface
  now fails closed before mismatch, insertion, generic-impl bypass, or raw generation.
  Every frozen child/diagnostic/exclusion/valid-output control passes. This adds no
  reference/tuple value, mutability, ownership, lifetime, layout, ABI, coercion,
  lowering, execution, bounds, backend, matrix, capability, or stability meaning;
  R-002 remains HIGH/CRITICAL and PARTIALLY CONTROLLED.
- Closure gate: the prepared six-record closure's fresh exact repository-root full
  gate exits 0 with 139/139 library, 149/149 binary, 7/7 claim, and 23/23 binding
  tests.
- Exact closure `d3811b00b15a594b9f3094f23cf61469db1133c8`, parent `a1ffeaec`,
  tree `c01088c482ee798202a71b4fe7dec859628ecf3e`, canonical diff
  `2799eb326c2e9097c53bad793b043293078a891a`, changed only the six control
  records with 153 insertions/19 deletions, received three exact approvals, and was
  published unchanged. Compiler `30918433816` / `30918438945`, stable/nightly Rust
  `30918439169`, all three CodeQL analyses in `30918434204`, and aggregate
  `92022619964` pass. CORE-034 is closed without a capability, matrix, risk, backend,
  artifact, claim, history, or `master` movement.

## AUDIT-041 authorization

- Basis: exact clean public CORE-034 closure `d3811b00b15a594b9f3094f23cf61469db1133c8`,
  tree `c01088c482ee798202a71b4fe7dec859628ecf3e`, canonical diff
  `2799eb326c2e9097c53bad793b043293078a891a`, with the all-eight public evidence
  above and a clean worktree. All accepted slices through CORE-034 are excluded.
- Scope: independently rank the complete remaining R-002/R-004/R-005/R-006/R-007/
  R-009/R-010/R-011/R-012/R-013/R-016 set. Each reviewer must cite every rank,
  identify one exact bounded candidate or stop, explain trusted reachability and
  containment, state semantic choices and phase count, propose one deterministic
  failing specimen plus preservation controls, and distinguish rejection, helper
  simulation, annotations, LLVM text, object emission, and hardware execution.
- Boundary: after authorization acceptance, the audit is static and read-only: no
  edit, test, build, formatter, probe, benchmark, artifact, hardware action, or
  external query. Candidate V three-array containment, Candidate L bounds, every
  other historical label, and every prior order are non-authoritative inputs.
- Selection: the lead may reconcile the three complete rankings to at most one
  unanimously bounded residual or record a stop. An unresolved semantic or
  compatibility choice, more than two compiler phases, hardware requirement,
  unsupported-source-type fallback, or valid-output uncertainty is a stop.
- Authorization files were limited to the six control records. A fresh exact full
  local gate, three exact reviews, unchanged publication, and all eight public checks
  were required before ranking began. AUDIT-041 grants no regression,
  implementation, semantics, capability, matrix, risk, workflow, dependency,
  backend, artifact, claim, history, or `master` authority.
- The prepared six-record authorization's fresh exact repository-root full gate
  exits 0 with 139/139 library, 149/149 binary, 7/7 claim, and 23/23 binding tests.
- Authorization `a31342e88a84d919098c64eac416979786a7957c`, parent `d3811b00`,
  tree `fbcd78b6ee139c4ab14bf0c7c5c6e86ab3a87dce`, canonical diff
  `313a1f6bd43c22593faa652921f90a2907a5622d`, received three exact approvals and
  was published unchanged. Compiler `30919164807` / `30919167478`, stable/nightly
  Rust `30919168162`, all three CodeQL analyses in `30919164869`, and aggregate
  `92025101785` pass.
- Independent rankings all placed R-002 first but selected valueless exact three-
  array tuple containment (type/safety), initialized exact three-array tuple
  containment (IR/codegen), and initialized positive-count immediate reference-
  around-array-of-tuple containment (backend/claim). Targeted comparison ranked
  V/I/R once and R/I/V twice. The lead provisionally selected R on readiness and
  smaller, shallower topology; all three final compatibility reviews approved the
  exact R predicate, diagnostics, ordering, 34-red/4-green matrix, two-phase boundary,
  context behavior, and preservation set.
- AUDIT-041 changed no file or classification. Bounds remains stopped pending policy;
  valueless and initialized exact three-array shapes remain bounded residuals rather
  than implementation authority.

## CORE-035 authorization history

- Exact behavior: reject only initialized `Statement::Let` annotations exactly
  shaped as `Type::Reference(Type::Array(Type::Tuple(_), count), _)` when
  `count > 0`, for both reference mutability flags and without recursive descent.
- Ordering/diagnostics: semantic duplicate detection and RHS validation remain
  first; checked RHS validation and Void rejection remain first; all existing
  initialized outer-, one-array-, two-array-, and immediate-reference-to-tuple
  diagnostics stay first. Then semantic emits `Error: Variable \`{name}\` uses an unsupported tuple type annotation directly beneath an array directly beneath a reference for an initialized binding.` and checked admission emits
  `checked IR binding \`{name}\` uses an unsupported tuple type annotation directly beneath an array directly beneath a reference for an initialized binding` before mismatch, the checked generic-impl bypass, insertion, or raw generation. The public
  semantic prefix is unchanged.
- Context: apply only wherever semantic analysis and checked admission already
  traverse bindings. Direct/top-level, explicit block, if branches, while, for,
  loop, non-generic impl, generic impl, and semantic generic-function bodies are
  covered. Checked generic functions retain their outer rejection; generic trait
  default bodies remain syntax-only.
- Tests-first was permitted after authorization acceptance only in
  `src/compiler/tests/binding_type_contract_tests.rs`. The existing immutable
  count-one acceptance rows had to be reclassified, never silently deleted, into one
  new
  aggregate. It must report exactly 34 unexpected acceptances before implementation:
  direct counts one/two x both flags x both phases 8; public count one both flags 2;
  top-level count one both flags at both phases 4; generic impl count one both flags
  at both phases 4; semantic generic functions count one both flags 2; and seven
  immutable count-one block/control-flow/loop/non-generic-impl contexts at both
  phases 14. Count zero for both flags at both phases is an exact four-observation
  green preservation matrix. Expected focused result is 0/1 and binding aggregate
  23/24 with only the new test failing after 139/139 library, 149/149 binary, and
  7/7 claim tests pass.
- Required green evidence: duplicate/RHS/Void precedence; every accepted
  CORE-025/028/029/030/031/032/033/034 diagnostic and ordering rule; count-zero and
  valueless target shapes; scalar and numeric-array references; array-around-
  reference, double/deeper reference, deeper array, generic, and wrapped forms;
  checked generic-function outer rejection; syntax-only trait defaults; and valid
  numeric-array LLVM output.
- Later implementation was limited to `src/compiler/src/semantic_analyzer.rs` and
  `src/compiler/src/ir_generator.rs`, adding two exact guards after separately
  reviewed public-red evidence. Parser, raw IR, verifier, codegen, LLVM verification,
  CLI, runtime, and backends could not change.
- Authorization changes were limited to the six control records. The exact full
  local gate, three exact reviews, unchanged publication, and all eight public checks
  were required before tests-first and were satisfied. No reference/array/tuple
  value, mutability, ownership,
  lifetime, layout, ABI, coercion, bounds, lowering, execution, backend, matrix,
  capability, risk, artifact, claim, history, or `master` meaning follows.
- The prepared six-record authorization's fresh exact repository-root full gate
  exits 0 with 139/139 library, 149/149 binary, 7/7 claim, and 23/23 binding tests.
- Stop on any predicate broader than the exact positive-count immediate topology,
  different red/green count, diagnostic or precedence drift, recursive matching,
  new traversal, third compiler phase, valid-output effect, workflow/dependency,
  artifact/claim, history, or `master` action.

## CORE-035 accepted implementation

- Authorization `b74b1d299f1cef15cc38d22e29fe1a6f16cb8ec0`, parent `a31342e8`,
  tree `3fc2d78f0a9d9cd7637343a9ef551a2dbd549758`, canonical diff
  `64fbd1fe82d59a52163578787fce084df7847858`, changed only the six control
  records, passed two exact local gates, received three approvals, and was published
  unchanged. Compiler `30921372203` / `30921374216`, Rust `30921376655`, CodeQL
  `30921371268`, and aggregate `92032740349` pass.
- Tests-only `f04e80c92db723de432b2502a055afea13fffed7`, parent `b74b1d29`, tree
  `03a9f27452498d84a063546d207f1c5781326d4f`, canonical diff
  `9e04b6adaf6e436d90d8f2e50a138a3cfad2251c`, changed one test file and received
  three approvals. Focused 0/1 and binding 23/24 exposed exactly 34 false successes;
  the local, push `30922180824`, PR `30922181281`, and nightly job `92035312036`
  runs passed 139/139 library, 149/149 binary, and 7/7 claim before that sole
  failure. Stable job `92035312020` was fail-fast cancelled. CodeQL `30922176056`
  and aggregate `92035461619` pass; three public-red reviews authorized implementation.
- Implementation `b8fd5a177d4916baf9a850f0857a83d57d71db66` adds only the two
  frozen guards: 21 lines in semantic analysis and 15 in checked admission. It is
  triple-approved; formatting, focused 1/1, binding 24/24, the exact full local gate,
  both compiler jobs, stable/nightly Rust, all three CodeQL analyses, and aggregate
  pass at the run IDs above.
- Result: initialized positive-count exact `Reference(Array(Tuple))` bindings now
  fail closed after child and prior diagnostics and before trusted IR publication.
  Count zero, valueless, deeper/wrapped/mixed, scalar/numeric, generic, traversal,
  and valid-output controls remain unchanged. This defines no reference/array/tuple
  value, ownership, lifetime, layout, ABI, bounds, lowering, execution, or backend
  capability. R-002 remains HIGH/CRITICAL and PARTIALLY CONTROLLED.
- Closure `60ad91f7`, tree `978aa98f`, canonical diff `818a8112`, changed only the
  six control records, passed the exact local gate above, received three approvals,
  and is public all-eight green in compiler `30923835957` / `30923837627`, Rust
  `30923838264`, CodeQL `30923834264`, and aggregate `92041128413`. CORE-035 is
  closed without a capability, matrix, risk, backend, artifact, claim, history, or
  `master` movement.

## AUDIT-042 authorization

- Basis: exact clean public CORE-035 closure `60ad91f7`, tree `978aa98f`, canonical
  diff `818a8112`, with the all-eight evidence above and a clean worktree. Every
  accepted slice through CORE-035 is excluded.
- Scope: independently rank the complete remaining R-002/R-004/R-005/R-006/R-007/
  R-009/R-010/R-011/R-012/R-013/R-016 set. Each reviewer must cite every rank,
  identify one exact bounded candidate or stop, explain trusted reachability and
  containment, state semantic choices and phase count, propose one deterministic
  failing specimen plus preservation controls, and distinguish rejection, helper
  simulation, annotations, LLVM text, object emission, and hardware execution.
- Boundary: after authorization acceptance, the audit is static and read-only: no
  edit, test, build, formatter, probe, benchmark, artifact, hardware action, or
  external query. V/I three-array containment, literal bounds, every historical
  label, and every prior order are non-authoritative inputs.
- Selection: the lead may reconcile the three complete rankings to at most one
  unanimously bounded residual or record a stop. An unresolved semantic or
  compatibility choice, more than two compiler phases, hardware requirement,
  unsupported-source-type fallback, or valid-output uncertainty is a stop.
- Authorization files are limited to the six control records. A fresh exact full
  local gate, three exact reviews, unchanged publication, and all eight public checks
  are required before ranking begins. AUDIT-042 grants no regression,
  implementation, semantics, capability, matrix, risk, workflow, dependency,
  backend, artifact, claim, history, or `master` authority.
- The prepared six-record authorization's fresh exact repository-root full gate exits
  0 with 139/139 library, 149/149 binary, 7/7 claim, and 24/24 binding tests.
- First authorization snapshot `4ce0de0d`, tree `350984b8`, canonical diff
  `347278c3`, passed that exact gate but was rejected before any independent push or
  branch-head publication. Type/safety found a P1 stale active hypothesis; IR/codegen
  found the same issue at P2 plus P1 stale DEC-046 closure status; backend/claim
  independently found the stale closure status at P1. The snapshot is retained in
  corrected authorization ancestry; ranking never began.
- Corrected authorization `2d8a0c54`, parent `4ce0de0d`, tree `45d1c184`, correction
  canonical diff `b36d3d9b`, and cumulative canonical diff from CORE-035 closure
  `478e947a`, passed two fresh exact full gates, received three fresh exact approvals,
  and was published unchanged. Compiler `30924946683` / `30924950615`, stable/
  nightly Rust `30924951134`, all three CodeQL analyses in `30924945035`, and
  aggregate `92044919183` pass.
- Complete independent ranking orders were: type/safety R-002/R-011/R-005/R-004/
  R-013/R-009/R-012/R-006/R-010/R-016/R-007 with U; IR/codegen R-002/R-011/R-005/
  R-004/R-006/R-009/R-013/R-012/R-010/R-007/R-016 with T; backend/claim R-011/
  R-002/R-005/R-004/R-006/R-013/R-007/R-010/R-009/R-012/R-016 with B.
- In targeted comparison, type/safety and IR/codegen ranked U > T > B and stopped B
  pending compile-time-versus-runtime bounds policy; backend/claim ranked B > U > T.
  The lead chose U two to one. All three final compatibility reviews approved only
  exact count-insensitive valueless U in two phases. Type/safety's nonblocking P1
  requires reclassification of every existing exact-U acceptance occurrence.
- AUDIT-042 is complete with no edit, test, build, formatter, probe, artifact,
  hardware action, or external query during ranking. B remains stopped and T remains
  fallback. No capability, matrix, risk, backend, artifact, or claim class moves.

## CORE-036 authorization

- Scope: valueless `Statement::Let` only, exact nonrecursive
  `Type::Reference(Type::Array(Type::Tuple(_), count), ref_flag)`, both flags, every
  count including zero, and every tuple arity. Initialized, double/deeper reference,
  deeper array, array-around-reference, generic/wrapped, scalar-reference, numeric-
  array-reference, and all other shapes are excluded.
- Order: semantic duplicate and the four existing valueless tuple-shape diagnostics
  remain first, followed by the exact new semantic rejection before `Ty::Int`
  fallback. Checked admission preserves those four diagnostics, adds no duplicate
  rule, then rejects before no-value admission/raw generation. Existing traversal
  only; checked generic-function outer rejection and syntax-only trait defaults stay.
- Diagnostics: semantic
  `Error: Variable \`{name}\` uses an unsupported tuple type annotation directly beneath an array directly beneath a reference for an uninitialized binding.`;
  checked
  `checked IR binding \`{name}\` uses an unsupported tuple type annotation directly beneath an array directly beneath a reference for an uninitialized binding`.
  Public compilation retains its existing semantic prefix.
- Tests first: only the binding-contract test file may change after this six-record
  authorization is accepted. Reclassify the four existing occurrence blocks/five
  source rows (immutable+mutable near 1519, immutable near 1661, immutable near 1863,
  and immutable near 2114) while preserving siblings. The new aggregate must report
  exactly 34 unexpected acceptances and 40 green preservation observations. Expected
  red is focused 0/1 and binding 24/25 only, after 139/139 library, 149/149 binary,
  and 7/7 claim tests pass.
- Implementation later: only separately reviewed public-red evidence may authorize
  exact guards in `semantic_analyzer.rs` and `ir_generator.rs`. Parser, raw IR,
  verifier, codegen, CLI, runtime, workflow, dependency, backend, artifact, and claim
  surfaces stay untouched.
- Classification: rejection is containment, not support. It defines no reference,
  array, tuple, default, mutability, ownership, lifetime, layout, ABI, bounds,
  lowering, execution, backend, or compatibility meaning. R-002 remains HIGH/
  CRITICAL and PARTIALLY CONTROLLED; R-011 remains open; no classification moves.

## CORE-036 implementation and closure

- Authorization `697bb3b4`, parent `2d8a0c54`, tree `b0cfd37b`, canonical binary
  diff `0a92ad7a`, changed only the six control records, passed two exact local gates,
  received three exact approvals, and is public all-eight green in compiler
  `30927281281` / `30927293459`, Rust `30927289178`, CodeQL `30927280707`, and
  aggregate `92052974430`.
- Tests-only `d52b117e`, parent `697bb3b4`, tree `76a3b2e9`, canonical binary diff
  `c2d5e46a`, changed one file, reclassified all four occurrence blocks/five rows,
  passed formatting, and received three approvals. Focused 0/1 and binding 24/25
  isolate exactly 34 unexpected acceptances after 139/139 library, 149/149 binary,
  and 7/7 claim passes locally and in push `30927952017`, PR `30927956714`, nightly
  `92055067840`, and stable `92055068009` test logs. CodeQL `30927952240` and
  aggregate `92055178151` pass; three public-red reviews approved implementation.
- Implementation `26d18924`, parent `d52b117e`, tree `8aec746c`, canonical binary
  diff `543f8a1c`, adds 17 semantic and 16 checked-admission lines only. It is triple-
  approved; formatting, focused 1/1, binding 25/25, the exact full local gate,
  compiler `30928759703` / `30928760789`, stable/nightly Rust `30928758562`, all
  three CodeQL analyses in `30928754859`, and aggregate `92057919831` pass.
- Result: the frozen valueless exact nonrecursive reference-array-tuple surface now
  fails closed after existing diagnostics and before fallback/raw generation. Every
  initialized/deeper/wrapped/mixed/scalar/numeric/traversal/valid-output control is
  preserved. This is containment, not supported value/lowering/execution evidence.
  R-002 remains HIGH/CRITICAL and PARTIALLY CONTROLLED; R-011 remains open; no
  capability, matrix, risk, backend, artifact, or claim class moves.
- Closure acceptance: exact `3f042e18766d4675d04e0ba7e0289b7aac43d7ea`, parent
  `799c4181`, tree `15d56e0ceb0715543b03f7338505901906b59d60`, canonical
  binary diff `ee8cbed07657edf21559205c0bc23b7bb0f40a53`, changed only the six
  control records with 62 insertions and 8 deletions. Its fresh correction gate,
  three exact approvals, unchanged publication, push CI `30930377220`, PR CI
  `30930379386`, stable/nightly Rust `30930380195`, all three CodeQL analyses in
  `30930375201`, and aggregate `92063404658` pass. CORE-036 is closed without any
  semantic, capability, matrix, risk, backend, artifact, or claim movement.
- Correction history: first closure snapshot `39c8564b`, parent `26d18924`, tree
  `7932dd42`, canonical binary diff `2cb44b26`, changed only six records and passed
  two exact full gates. Type/safety approved, but IR/codegen and backend/claim each
  rejected it at P1 because Repository state still named CORE-035 `b8fd5a17` as the
  current public implementation head. The rejected snapshot was not independently
  pushed or made branch head. It remains in corrected ancestry; this additive
  correction updates the current pointer to `26d18924` and changes no evidence,
  semantics, implementation, classification, or authority boundary.
- First additive correction `799c4181`, parent `39c8564b`, tree `1c8a883f`, canonical
  binary diff `9a1f5cd8`, changed only the six records. Type/safety approved it, but
  IR/codegen rejected P1 because DEC-048's status still called the completed
  verification gate pending while the later evidence recorded both gates green. The
  review round stopped before publication. It remains in second-correction ancestry;
  exact accepted correction `3f042e18` aligns that status and changes no other
  boundary.

## AUDIT-043 authorization boundary

- Basis: exact clean public CORE-036 closure `3f042e18`. Exclude every accepted slice
  through CORE-036 and independently rank the complete remaining R-002/R-004/R-005/
  R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016 set.
- Method: type/safety, IR/codegen, and backend/claim reviewers must each rank all
  eleven residuals with file/symbol evidence, trusted reachability, exact containment,
  unresolved choices, phase count, deterministic failing specimen and preservation
  controls, and one bounded candidate or explicit stop. No prior candidate, label,
  preservation row, or order is privileged.
- Authority: static read-only inspection only after this six-record authorization is
  locally green, triple-approved, published unchanged, and public all-eight green.
  It grants no test, implementation, semantics, capability, matrix, risk, workflow,
  dependency, backend, artifact, claim, history, or `master` authority.
- Stop: incomplete ranking, unresolved semantics or compatibility, more than two
  compiler phases, hardware dependence, unsupported valid-output claim, or inability
  to specify deterministic tests-first evidence. Bounds candidate B remains stopped
  while compile-time rejection versus runtime behavior is unfrozen.
- Correction history: first authorization snapshot `cb43d1bb`, parent `3f042e18`,
  tree `f0f19f5d`, canonical binary diff `ead99a7b`, changed only six records and
  passed both gates. All three reviewers rejected it at P1 because DEC-049 status
  still called the completed gates required; type/safety also found “next immutable
  snapshot” stale for the already-committed review target. It was not published and
  no ranking began. The additive correction aligns current state and changes no
  other boundary.
- Corrected authorization acceptance: exact `5276df5b`, parent `cb43d1bb`, tree
  `c3eaf3cf`, correction canonical binary diff `b8b7586f`, and cumulative diff from
  CORE-036 `fe5376dc`, changed only six records, passed its correction gate, received
  three fresh exact approvals, and is public all-eight green in push CI `30931510621`,
  PR CI `30931515125`, Rust `30931515426`, CodeQL `30931509579`, and aggregate
  `92067252294`.
- Ranking result: type/safety initially selected bounded R-009 UTF-16 LSP coordinate
  correction; IR/codegen and backend/claim selected exact valueless nonrecursive
  `Array(Array(Array(Tuple)))` containment under R-002. All ranked the complete
  eleven-risk set and kept R-011 bounds stopped. Final compatibility unanimously
  selected R-002 over R-009, but only after a separate behavior-neutral shared-
  classifier task closes green. AUDIT-043 was entirely read-only and moved no
  semantics, risk, matrix, capability, backend, artifact, or claim state.

## ARCH-001 authorization boundary

- Historical scope: this section records the earlier isolated, behavior-neutral
  ARCH-001 authorization. CORE-037 separately supersedes only its sequencing/file
  restriction under the user's current executable-milestone mandate; the exact
  nonrecursive disposition table, diagnostics, precedence, generic gates, and raw
  compatibility constraints remain binding.
- Classifier: `BindingAnnotationDisposition` with only
  `ExistingExplicitRejection(RejectKind)`,
  `MatchesExistingContractShape(ContractKind)`, and `PreserveExistingBehavior`.
  “Contract shape” is routing metadata, not support or capability. Preserve is inert:
  it performs no inference, fallback, rejection, quarantine, traversal, or diagnostic.
- Input: exact annotation tree plus initializer presence. Classification is
  nonrecursive. The ten existing initialized/valueless rejection rules, their count
  conditions, both reference flags, and boundary-specific diagnostics must remain
  exact. Contract kinds are only current `i32`/`int`, `f64`/`float`, `bool`, uppercase
  `String`, and positive one-level numeric arrays; existing generic/context gates stay
  outside the classifier.
- Order: semantic duplicate precedence; initialized RHS validation; checked RHS/Void;
  mismatch/generic-impl behavior; valueless fallback; checked generic-function outer
  rejection; trait syntax-only behavior; and all traversal remain byte-for-byte and
  behavior-for-behavior unchanged. Raw generation and unchecked compatibility APIs do
  not call the classifier.
- Workflow: after six-record authorization acceptance, add green characterization/
  parity evidence in the binding contract test only. After separate review and public
  green, refactor only `ast.rs`, `semantic_analyzer.rs`, and `ir_generator.rs`.
  ARCH-001 must keep the later exact three-array R-002 specimen accepted and all valid
  LLVM byte-identical. Any behavior delta or third compiler phase is a stop.
- Review history: first ARCH-001 snapshot `63d8d599`, parent `5276df5b`, tree
  `28cd120c`, canonical binary diff `9fef5adf`, changed exactly the six records and
  received two approvals. Backend/claim rejected P1 because five records retained
  superseded pre-acceptance AUDIT-043 pending/no-ranking evidence in present tense.
  It was not published and no characterization/source edit began. The additive six-
  record correction is chronology-only and changes no ARCH-001 boundary.
- Authorization acceptance: exact additive correction `1dcfd869`, parent `63d8d599`,
  tree `b537023c`, correction diff `e5ee8aa7`, cumulative diff `5208cb6e`, received
  three fresh exact approvals and was published unchanged. Push CI `30934518525`, PR
  CI `30934523152`, Rust `30934523078`, CodeQL `30934519513`, and aggregate
  `92077350363` pass all eight checks on the exact PR/remote head. No source/test edit
  or classification/capability movement occurred.
- Acceptance-sync review history: first snapshot `4c18450a`, parent `1dcfd869`, tree
  `ea7b91c9`, canonical binary diff `7be565db`, received backend/claim and IR/codegen
  approvals but type/safety rejected P1 because three records declared
  characterization already eligible while the sync remained pending. It was not
  published and no test/source edit began. This additive six-record correction changes
  only those eligibility statements and review chronology.

## Exact next action

Freeze H1 separately and ledger-first from accepted CAP-047/B1C merge
`0365e5c91bd503b198855b97b7f16054488d6dff`. H1 must define the
canonical Aero compiler source bundle, exact stage-0/stage-1/stage-2
interfaces, environment and toolchain manifest, comparison contract, and
failure rules before any convergence implementation. Do not treat the bounded
B1C product as a general compiler or a self-hosting claim.

CAP-048 freezes that contract without changing behavior. After its protected
acceptance, authorize H1A red-first for complete canonical source/token
ingestion and stop at the independently predicted first unsupported parser
construct.

## Historical post-CAP-024 ranking

The post-CAP-024 order keeps every successor at readiness and task-local red-probe scope.
CAP-022 remains a mandatory `NO IMPLEMENTATION` runtime-acquisition stop, and CAP-016's
module/RFC stop remains closed.

| Rank | Capability gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Favorable risk | Favorable evidence cost | Total |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | Exact CPU + recursive-CopyData application-profile composition readiness and red probe only | 5 | 5 | 5 | 5 | 2 | 2 | 24 |
| 2 | Owned dynamic collection/streaming foundation readiness and red probe, including its shared allocation/ownership/drop architecture, only | 5 | 5 | 5 | 5 | 1 | 1 | 22 |
| 3 | Small quantized numerical-kernel readiness and red probe under one frozen cross-platform arithmetic-and-representation contract only | 5 | 5 | 3 | 5 | 1 | 1 | 20 |

`Favorable risk` and `Favorable evidence cost` are delivery-favorability scores:
higher means lower implementation risk or lower evidence cost.

Before rank 1, CAP-024 proves the exact accepted flat CAP-023 application and its
immutable correctness/reproducibility boundary, while recursive finite CopyData structs,
enums, typed `Result`, `Match`, and ownership slices remain separate bounded `PARTIAL`
experimental authorities rejected by `exact-i32-array-v0`. After rank 1 readiness, a
task-local source probe and architecture map must identify the first composition failure
and freeze whether one new application profile can reuse exact-`i32` scalar/flat-array
physical authority together with only already-accepted recursive CopyData, typed-result,
`Match`, and bounded ownership contracts; define admitted types and operations, phase
ownership, profile selection, physical identity, rejection boundaries, verifier evidence,
and Linux and Windows oracles; and yield either one bounded later implementation contract
within two compiler phases or an explicit mandatory stop without widening either existing
profile.

Stop rank 1 before implementation if composition requires changing `stable-scalar-v0` or
`exact-i32-array-v0`, importing broad experimental defaults, inventing struct, enum,
`Result`, layout, ABI, ownership, or error semantics, reconciling duplicate type,
physical, or specialization authorities, adding recursive or nested exact arrays,
crossing more than two compiler phases, or claiming general CopyData, inference, safety,
or language completion.

Evidence that the CAP-023 workload can materially exercise existing CopyData aggregates
and typed failure under one bounded profile without new semantics and with one shared
exact physical/verifier authority raises rank 1 toward later implementation; evidence
that a flat record remains sufficient, that runtime ingress is prerequisite, or that
composition requires broad layout or ownership contracts defers it and changes the
decision.

Before rank 2, accepted CAP-011 provides one fixed-capacity recursive-CopyData
`Window<T>` algorithm and the representative program composes only statically bounded
storage; Aero has no accepted owned dynamic collection, allocation, capacity growth,
initialized-length, reallocation, alias, failure, or drop contract. Legacy `stdlib.rs`
String/Vec helpers and their rejected checked-IR/backend instructions are not
source-language authority. After rank 2 readiness, a task-local
owned-collection/streaming source probe and architecture map must first freeze the public
type/API name, then the minimal useful element class and operations;
length/capacity/growth and initialized-state rules; allocation, failure,
move/borrow/alias, reallocation, iteration/indexing, and drop behavior; one physical and
verifier authority; rejection boundaries; and deterministic Linux and Windows oracles,
yielding either one bounded later implementation contract within two compiler phases or a
mandatory stop without claiming dynamic collections.

Stop rank 2 before implementation if allocation, OOM/error, ownership, alias,
reallocation invalidation, lifetime, drop, runtime ABI, or element destruction semantics
remain unfrozen; if uninitialized elements can become observable; if legacy unchecked
helpers or verifier-rejected instructions would be activated; if the useful slice crosses
more than two compiler phases; or if invalid collection state can reach trusted IR/backend
or Linux and Windows behavior cannot be made equivalent and observable.

Evidence that fixed-capacity `Window<T>` plus flat source records serves the next useful
workload, that runtime ingress is prerequisite, or that one owned collection requires
broad allocator/drop/lifetime architecture keeps rank 2 at readiness scope; only an
explicit collection RFC plus a probe demonstrating one shared cross-platform
initialized-state, ownership, physical, error, and verifier authority within the phase
limit raises it toward later implementation.

Before rank 3, CAP-024 preserves exact wrapping `i32` matvec, positive-only zero clamp,
two biased logits, and signed strict-greater argmax, but Aero has no frozen quantized
stored, accumulator, or result representation; scale or zero-point contract; conversion,
rounding, tie, saturation, or overflow behavior; calibration provenance; malformed-state
rule; or maintained cross-platform quantized oracle. After rank 3 readiness, a task-local
source-embedded red probe and architecture map must locate the first failure and freeze
every such decision plus operation order and Linux/Windows equivalence, yielding either
one bounded later implementation contract within two compiler phases or an explicit
mandatory stop without claiming quantization capability.

Stop rank 3 before implementation if any arithmetic or representation decision remains
unfrozen; if the slice requires implicit conversion, fallback typing, unfounded division
or rounding semantics, or a second numerical authority; if the scalar-double helper is
treated as source-language proof; if CAP-023 wrapping order changes; if malformed
quantization state can reach trusted IR or backend generation; if the slice crosses more
than two compiler phases; or if deterministic Linux and Windows oracle parity cannot be
proved.

Evidence that exact CPU plus CopyData application composition must precede a meaningful
quantized oracle, or that exact `i32` continues to advance the next workload without lossy
representation, keeps rank 3 at readiness scope; only an explicit quantization RFC plus a
probe demonstrating one shared cross-platform representation, arithmetic,
malformed-state, and oracle authority within the phase limit raises it toward later
implementation.

## Unauthorized actions

Do not publish releases/packages/registry changes/benchmark claims; force-push or
rewrite history; delete substantial evidence; change the language's fundamental
identity; make incompatible syntax/semantics changes without a migration plan;
modify downstream repositories; spend money; use unbounded paid compute; handle
credentials; or perform destructive system operations.
