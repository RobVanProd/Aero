use compiler::conformance::run_conformance_suite;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

const CAP014_ACCEPTANCE_EVIDENCE: [&str; 10] = [
    "226279dd174f26dc3cd1c7573798955bfe789f78",
    "ca09ebe3c1b981339c8bf56b360e62208ac900e1",
    "448e1c2ff397012804b886b904aa43bec63f2d37",
    "31570455915",
    "31570461500",
    "31570461524",
    "31570456382",
    "31570823665",
    "31570823712",
    "31570823073",
];

const CAP015_ACCEPTANCE_EVIDENCE: [&str; 10] = [
    "dd9b1710abebf2f2318582cf94568c2f9a30ca8f",
    "b62696272f293f9f378f8a368cc818fcb8ef1074",
    "27f359bc5ca90212a06ce73b71759cac0533c1f0",
    "31597830488",
    "31598146528",
    "31598146473",
    "31598144554",
    "31598634185",
    "31598634090",
    "31598633803",
];

const CAP018_ACCEPTANCE_EVIDENCE: [&str; 13] = [
    "409eca9ed2dd8b4ba79f34e14ecfefcc0386e3df",
    "3073c881c883984f53fcde2f0b205acbec760145",
    "c49ff17cab7fc0e8d4f552a71499929135c16c61",
    "31614934307",
    "31614994226",
    "31614994253",
    "31614991761",
    "31615467151",
    "31615467115",
    "31615465499",
    "1608636029",
    "1608636345",
    "1608644785",
];

const CAP019_ACCEPTANCE_EVIDENCE: [&str; 22] = [
    "f2955bedd22708041e36ee90c65c4f08c443d740",
    "c520729e7b081087bbe431e97d937fb77f519b37",
    "84916e124752b8e7d228855a0969cd9eab8dba26",
    "6ebeb0efb6e83ccc50e12d395e4add1c63ef48b4",
    "31627264709",
    "31627385522",
    "31627385563",
    "31627405516",
    "94217394313",
    "31627880853",
    "31627880924",
    "31627880812",
    "94218938557",
    "94218938794",
    "94218938835",
    "94218939033",
    "94218943455",
    "94218943514",
    "94218943605",
    "1609396076",
    "1609396442",
    "1609401493",
];

const CAP020_ACCEPTANCE_EVIDENCE: [&str; 28] = [
    "3b61cd1ed34f910f556821942cd06301ba17dd50",
    "800510de85bd82f3332126ad249c95da109dd3e1",
    "13157687f3e955d1c8292ccca133c5a73e29e1a7",
    "d9493d5123840b38ebab6ca275aaba3216728706",
    "31639493741",
    "31639540134",
    "31639540030",
    "31639535638",
    "94258433541",
    "94258276078",
    "94258275978",
    "94258275899",
    "94258264605",
    "94258264489",
    "94258264627",
    "31640016314",
    "31640016316",
    "31640015733",
    "94259869631",
    "94259869676",
    "94259869637",
    "94259869559",
    "94259873136",
    "94259873164",
    "94259873086",
    "1610137115",
    "1610137589",
    "1610144660",
];

const CAP021_ACCEPTANCE_EVIDENCE: [&str; 33] = [
    "f91df56084540d30f3c8d09e71c5f30db280fd93",
    "7e34b4b8e817a7aafaaabc6326fa0a4d616fcc91",
    "df0626916d190d8a7580f783e3ac24a89f691617",
    "59af445ea02c1759d337d698be9c4f4472587aaf",
    "31670574143",
    "31670599830",
    "31670599826",
    "31670598033",
    "94354297550",
    "94354135184",
    "94354214336",
    "94354214389",
    "94354214394",
    "94354214410",
    "94354210797",
    "94354210770",
    "94354210832",
    "1611711722",
    "1611712334",
    "1611716646",
    "31671091285",
    "31671091296",
    "31671091099",
    "94355683766",
    "94355683532",
    "94355683515",
    "94355683534",
    "94355685544",
    "94355685480",
    "94355685574",
    "1611737053",
    "1611737605",
    "1611740699",
];

const CAP023_ACCEPTANCE_EVIDENCE: [&str; 33] = [
    "63e6b00b6294de61e3afd292a1e32e2b014714e2",
    "4d234cdfde67f1083773e2c41be4ab92027769db",
    "4bce540dfed6dfffa152067f4e00424501a6cdd8",
    "e9b281504446465cfc8fcbe17c65cce92df0e83a",
    "31687464571",
    "31687585904",
    "31687585893",
    "31687584263",
    "94407323731",
    "94406770929",
    "94407177877",
    "94407178006",
    "94407178047",
    "94407178042",
    "94407175858",
    "94407175752",
    "94407175820",
    "1612686978",
    "1612687391",
    "1612693654",
    "31688093145",
    "31688093150",
    "31688092749",
    "94408808914",
    "94408809340",
    "94408809458",
    "94408809296",
    "94408812427",
    "94408812194",
    "94408812175",
    "1612715455",
    "1612715345",
    "1612721829",
];

const CAP024_ACCEPTANCE_EVIDENCE: [&str; 45] = [
    "617bfce86feb879ee5eef61b44cf4e2a5520f022",
    "9520f24e4f1626f16782a9775480f9653f6059bb",
    "918c9222eb61e2435e18847e30b946cd08013238",
    "2f7ec325e423461a8e867f4ee2573ae6dcf15dfd",
    "31764763341",
    "31764765501",
    "31764765563",
    "31764763584",
    "31764765495",
    "94658200345",
    "94658206474",
    "94658207134",
    "94658207170",
    "94658207086",
    "94658203257",
    "94658203263",
    "94658203316",
    "94658280067",
    "1617260890",
    "1617261159",
    "1617264144",
    "94658206500",
    "94658206555",
    "94659098928",
    "9205970753",
    "bd5e609b4ce829579331a23170d6d9e4fc4d5906cb32779876a78bc24294812c",
    "62780d81e9dcaa6e85c08d0805608a58283816dd062c3a8bb1a8c67971ac551f",
    "4b4cfce95459761dddd588e09abb3046854e0c2afb361f08a9553f180f013a34",
    "31765227712",
    "31765227675",
    "31765227317",
    "31765227673",
    "94659602474",
    "94659602479",
    "94659602493",
    "94659602501",
    "94659604078",
    "94659604103",
    "94659604064",
    "1617281747",
    "1617282341",
    "1617285598",
    "94659602932",
    "94659621233",
    "94659603455",
];

const CAP019_EVIDENCE_PREFIX: &str = "Exact CAP-019 reviewed candidate \
`f2955bedd22708041e36ee90c65c4f08c443d740`, shared candidate/merge tree \
`c520729e7b081087bbe431e97d937fb77f519b37`, accepted base and first merge parent \
`84916e124752b8e7d228855a0969cd9eab8dba26`, and protected PR #56 merge \
`6ebeb0efb6e83ccc50e12d395e4add1c63ef48b4` whose second parent is that candidate are immutable.";
const CAP020_EVIDENCE_PREFIX: &str = "Exact CAP-020 reviewed candidate \
`3b61cd1ed34f910f556821942cd06301ba17dd50`, shared candidate/merge tree \
`800510de85bd82f3332126ad249c95da109dd3e1`, accepted base and first merge parent \
`13157687f3e955d1c8292ccca133c5a73e29e1a7`, and protected PR #58 merge \
`d9493d5123840b38ebab6ca275aaba3216728706` whose second parent is that candidate are immutable.";
const CAP020_EVIDENCE_PARAGRAPH: &str = "Exact CAP-020 reviewed candidate \
`3b61cd1ed34f910f556821942cd06301ba17dd50`, shared candidate/merge tree \
`800510de85bd82f3332126ad249c95da109dd3e1`, accepted base and first merge parent \
`13157687f3e955d1c8292ccca133c5a73e29e1a7`, and protected PR #58 merge \
`d9493d5123840b38ebab6ca275aaba3216728706` whose second parent is that candidate are immutable. \
Candidate push CI `31639493741`, PR CI `31639540134`, Rust CI `31639540030`, \
CodeQL `31639535638`, and aggregate candidate check `94258433541`; candidate stable/nightly/Windows \
LLVM 22 jobs `94258276078`/`94258275978`/`94258275899` and CodeQL Actions/Python/Rust \
jobs `94258264605`/`94258264489`/`94258264627`; merge-head CI/Rust CI/CodeQL \
`31640016314`/`31640016316`/`31640015733`, exact merge compiler/stable/nightly/Windows \
LLVM 22 jobs `94259869631`/`94259869676`/`94259869637`/`94259869559`, merge CodeQL \
Actions/Python/Rust jobs `94259873136`/`94259873164`/`94259873086`, and default-branch \
Actions/Python/Rust analyses `1610137115`/`1610137589`/`1610144660` all pass.";
const CAP021_EVIDENCE_PREFIX: &str = "Exact CAP-021 reviewed candidate \
`f91df56084540d30f3c8d09e71c5f30db280fd93`, shared candidate/merge tree \
`7e34b4b8e817a7aafaaabc6326fa0a4d616fcc91`, accepted base and first merge parent \
`df0626916d190d8a7580f783e3ac24a89f691617`, and protected PR #60 merge \
`59af445ea02c1759d337d698be9c4f4472587aaf` whose second parent is that candidate are immutable.";
const CAP021_EVIDENCE_PARAGRAPH: &str = "Exact CAP-021 reviewed candidate \
`f91df56084540d30f3c8d09e71c5f30db280fd93`, shared candidate/merge tree \
`7e34b4b8e817a7aafaaabc6326fa0a4d616fcc91`, accepted base and first merge parent \
`df0626916d190d8a7580f783e3ac24a89f691617`, and protected PR #60 merge \
`59af445ea02c1759d337d698be9c4f4472587aaf` whose second parent is that candidate are immutable. \
Candidate push CI `31670574143`, PR CI `31670599830`, Rust CI `31670599826`, \
CodeQL `31670598033`, and aggregate candidate check `94354297550`; candidate push/PR compiler jobs \
`94354135184`/`94354214336`, stable/nightly/Windows LLVM 22 jobs \
`94354214389`/`94354214394`/`94354214410`, CodeQL Actions/Python/Rust jobs \
`94354210797`/`94354210770`/`94354210832`, and Actions/Python/Rust analyses \
`1611711722`/`1611712334`/`1611716646`; merge-head CI/Rust CI/CodeQL \
`31671091285`/`31671091296`/`31671091099`, exact merge compiler/stable/nightly/Windows \
LLVM 22 jobs `94355683766`/`94355683532`/`94355683515`/`94355683534`, merge CodeQL \
Actions/Python/Rust jobs `94355685544`/`94355685480`/`94355685574`, and default-branch \
Actions/Python/Rust analyses `1611737053`/`1611737605`/`1611740699` all pass.";
const CAP023_EVIDENCE_PREFIX: &str = "Exact CAP-023 reviewed candidate \
  `63e6b00b6294de61e3afd292a1e32e2b014714e2`, shared candidate/merge tree \
  `4d234cdfde67f1083773e2c41be4ab92027769db`, accepted base and first merge parent \
  `4bce540dfed6dfffa152067f4e00424501a6cdd8`, and protected PR #62 merge \
  `e9b281504446465cfc8fcbe17c65cce92df0e83a` whose second parent is that candidate are immutable.";
const CAP023_EVIDENCE_PARAGRAPH: &str = "Exact CAP-023 reviewed candidate \
  `63e6b00b6294de61e3afd292a1e32e2b014714e2`, shared candidate/merge tree \
  `4d234cdfde67f1083773e2c41be4ab92027769db`, accepted base and first merge parent \
  `4bce540dfed6dfffa152067f4e00424501a6cdd8`, and protected PR #62 merge \
  `e9b281504446465cfc8fcbe17c65cce92df0e83a` whose second parent is that candidate are immutable. \
  Candidate push CI `31687464571`, PR CI `31687585904`, Rust CI `31687585893`, \
  CodeQL `31687584263`, and aggregate candidate check `94407323731`; candidate push/PR compiler jobs \
  `94406770929`/`94407177877`, stable/nightly/Windows LLVM 22 jobs \
  `94407178006`/`94407178047`/`94407178042`, CodeQL Actions/Python/Rust jobs \
  `94407175858`/`94407175752`/`94407175820`, and Actions/Python/Rust analyses \
  `1612686978`/`1612687391`/`1612693654`; merge-head CI/Rust CI/CodeQL \
  `31688093145`/`31688093150`/`31688092749`, exact merge compiler/stable/nightly/Windows \
  LLVM 22 jobs `94408808914`/`94408809340`/`94408809458`/`94408809296`, merge CodeQL \
  Actions/Python/Rust jobs `94408812427`/`94408812194`/`94408812175`, and default-branch \
  Actions/Python/Rust analyses `1612715455`/`1612715345`/`1612721829` all pass.";
const CAP024_EVIDENCE_PREFIX: &str = "Exact CAP-024 reviewed candidate \
  `617bfce86feb879ee5eef61b44cf4e2a5520f022`, shared candidate/merge tree \
  `9520f24e4f1626f16782a9775480f9653f6059bb`, accepted base and first merge parent \
  `918c9222eb61e2435e18847e30b946cd08013238`, and protected PR #64 merge \
  `2f7ec325e423461a8e867f4ee2573ae6dcf15dfd` whose second parent is that candidate are immutable.";
const CAP024_EVIDENCE_PARAGRAPH: &str = "Exact CAP-024 reviewed candidate \
  `617bfce86feb879ee5eef61b44cf4e2a5520f022`, shared candidate/merge tree \
  `9520f24e4f1626f16782a9775480f9653f6059bb`, accepted base and first merge parent \
  `918c9222eb61e2435e18847e30b946cd08013238`, and protected PR #64 merge \
  `2f7ec325e423461a8e867f4ee2573ae6dcf15dfd` whose second parent is that candidate are immutable. \
  Candidate push CI `31764763341`, PR CI `31764765501`, Rust CI `31764765563`, CodeQL \
  `31764763584`, and CAP-024 evidence run `31764765495`; candidate push/PR compiler jobs \
  `94658200345`/`94658206474`, stable/nightly/Windows LLVM 22 jobs \
  `94658207134`/`94658207170`/`94658207086`, CodeQL Actions/Python/Rust jobs \
  `94658203257`/`94658203263`/`94658203316`, aggregate candidate CodeQL check \
  `94658280067`, and Actions/Python/Rust analyses \
  `1617260890`/`1617261159`/`1617264144` all pass. Candidate CAP-024 \
  Linux/Windows/aggregate jobs `94658206500`/`94658206555`/`94659098928` pass and artifact \
  `9205970753` carries fresh manifest \
  `bd5e609b4ce829579331a23170d6d9e4fc4d5906cb32779876a78bc24294812c` plus 132 fresh observations \
  `62780d81e9dcaa6e85c08d0805608a58283816dd062c3a8bb1a8c67971ac551f`; \
  its claim-bearing projection matches accepted canonical manifest \
  `4b4cfce95459761dddd588e09abb3046854e0c2afb361f08a9553f180f013a34`. \
  Merge-head CI `31765227712`, Rust CI `31765227675`, CodeQL `31765227317`, and CAP-024 \
  replay `31765227673`; exact merge compiler/stable/nightly/Windows LLVM 22 jobs \
  `94659602474`/`94659602479`/`94659602493`/`94659602501`, CodeQL Actions/Python/Rust jobs \
  `94659604078`/`94659604103`/`94659604064`, default-branch analyses \
  `1617281747`/`1617282341`/`1617285598`, and CAP-024 aggregate replay job \
  `94659602932` all pass. The two default-branch capture jobs \
  `94659621233`/`94659603455` are correctly skipped because protected master validates the tracked \
  bundle rather than replacing accepted observations.";
const CAP018_CANDIDATE_LEADIN: &str = "Exact CAP-018 candidate \
`409eca9ed2dd8b4ba79f34e14ecfefcc0386e3df`";

const CAP015_PRODUCT_BOUNDARY: &str = "General-purpose text parsing, runtime Strings, \
serialization, runtime ingestion, file input, and Unicode text encoding/normalization \
remain unsupported; accepted CORE-072's bounded Unicode scalar `char` remains `PARTIAL`.";
const STALE_CAP015_PRODUCT_BOUNDARY: &str = "general parsing, strings, serialization, runtime ingestion, file input, and unicode remain unsupported";

const CAP019_PROFILE_HISTORY: &str = "Accepted CAP-014 created the CPU-only \
`exact-i32-array-v0` profile; accepted CAP-018 remains its immutable exact-array \
result-composition checkpoint; accepted CAP-019 widens that same profile with fully \
initialized mutable owned locals, direct projected element writes, and returned \
flat-array values rather than creating another profile.";

const CAP019_VALUE_BOUNDARY: &str = "Accepted CAP-019 widens the existing flat nonempty \
exact-`Int` class to a fully initialized mutable owned local whose initializer is an \
admitted literal, immutable exact-array identifier, or acyclic ordinary call of the \
same count, plus direct `local[index] = exact_int_value` projected writes.";

const CAP019_PRODUCT_BOUNDARY: &str = "The maintained eight-lane application copies an \
immutable input, increments every lane in a guarded loop, returns the whole array by \
value, feeds it into the accepted CPU kernel, preserves all eight source lanes, \
produces result `2035`, and exits `91`; Linux and Windows retain read traps and add \
negative/equal-to-count write traps under verified LLVM/Clang 22 `-O0`/`-O2` routes.";

const CAP019_CLASSIFICATION_BOUNDARY: &str = "The single selected \
`exact-i32-array-v0` row remains `END_TO_END`; broad integer and fixed-array support \
remains `PARTIAL`; `stable-scalar-v0` remains Aero's only `STABLE` profile.";

const CAP019_EXCLUSION_BOUNDARY: &str = "CAP-019 does not admit general mutable arrays, \
uninitialized or partial arrays, mutable parameters/results/aliases, references or \
escaping places, whole-array reassignment, zero/recursive/nested/repeat/non-Int arrays, \
stable aggregate ABI/layout, general parsing/string/file behavior, GPU execution, \
performance, or safety.";

const CAP019_SPECIALIZATION_BOUNDARY: &str = "CAP-013 remains the single shared \
specialization identity/phase authority; CAP-018 and CAP-019 add no specialization \
classifier.";

const CAP020_ZERO_PRODUCTION_BOUNDARY: &str = "Accepted CAP-020 changes no parser, \
grammar, source semantics, language profile, semantic analysis, checked IR, verifier, \
backend, ABI, or capability classification; it is a zero-production product/evidence \
checkpoint over CAP-019's `exact-i32-array-v0` surface.";

const CAP020_PRODUCT_BOUNDARY: &str = "The accepted application encodes a 2x3 matrix as \
`[int; 6]`, consumes an `[int; 3]` vector, computes wrapping `row * 3 + column` in \
nested loops, returns a fully initialized mutable-produced `[i32; 2]`, preserves every \
input lane, produces ordinary and wrapping results `[50, 122]` and `[-2, 5]`, and exits \
`91`.";

const CAP020_GUARD_BOUNDARY: &str = "The computed linear value flows through the existing \
signed bounds and trap-before-address authority before a `[6 x i32]` load, with \
corresponding guarded `[3 x i32]` load and `[2 x i32]` store.";

const CAP020_EXCLUSION_BOUNDARY: &str = "CAP-020 adds no matrix type, recursive or nested \
arrays, static index proof, checked-overflow arithmetic, stable layout or ABI, \
performance, accelerator execution, general mutation, or safety claim.";

const CAP020_HISTORY_BOUNDARY: &str = "CAP-019 remains the latest compiler/profile \
capability widening; CAP-020 is an accepted product gate, not a separate profile or \
feature row.";

const CAP020_ALERT_BOUNDARY: &str = "The sole open finding remains pre-existing Actions \
alert #4 from 2026-08-09; no new CAP-020 alert surfaced.";

const CAP021_ZERO_PRODUCTION_BOUNDARY: &str = "Accepted CAP-021 changes no parser, \
grammar, source semantics, language profile, semantic analysis, checked IR, verifier, \
backend, ABI, or capability classification; it is a zero-production product/evidence \
checkpoint over CAP-019's `exact-i32-array-v0` surface and composes the accepted \
CAP-020 flat matvec.";

const CAP021_RECORD_BOUNDARY: &str = "The accepted application treats one \
source-embedded flat `[int; 17]` as an application record with exact header \
`[2, 3, 1]`, dynamically decodes input, row-major first-stage weights, first-stage \
bias, second-stage weights, and score bias into fully initialized flat locals, then \
composes the accepted 2x3 matvec with wrapping bias and affine scoring.";

const CAP021_RESULT_BOUNDARY: &str = "The accepted scorer returns \
`[valid, raw0, raw1, hidden0, hidden1, score]`; its ordinary result is \
`[1, 122, 167, 135, 181, 4938]`, its wrapping result is \
`[1, -24, 18, 2147483623, -2147483631, -2147483627]`, an invalid header returns six \
zeros, both valid source records preserve and reread all 17 lanes, and the application \
exits `91`.";

const CAP021_GUARD_BOUNDARY: &str = "Every dynamic read and write uses the existing \
signed bounds, trap-before-address, `sext`, typed-GEP, and same-pointer consumer \
authority; exact public and pinned Linux/Windows LLVM 22 verifier, O0/O2, native, and \
deterministic-emission evidence passes.";

const CAP021_EXCLUSION_BOUNDARY: &str = "CAP-021 adds no tensor, matrix, struct, record, \
recursive-array, nested-array, serialization, runtime/file-input, quantization, \
activation, checked-overflow, stable layout/ABI, performance, accelerator, safety, \
general inference, or language-completion capability; the flat record is an \
application convention, not a source or physical type.";

const CAP021_HISTORY_BOUNDARY: &str = "CAP-019 remains the latest compiler/profile \
capability widening; CAP-020 and CAP-021 are accepted product gates, not separate \
profiles or feature rows.";

const CAP021_ALERT_BOUNDARY: &str = "The PR-only aggregate CodeQL check is correctly \
absent on the default branch; the sole open finding remains pre-existing Actions alert \
#4 from 2026-08-09, and no new CAP-021 alert surfaced.";

const CAP023_ZERO_PRODUCTION_BOUNDARY: &str = "Accepted CAP-023 changes no parser, \
  grammar, source semantics, language profile, semantic analysis, checked IR, verifier, \
  backend, ABI, or capability classification. It is a zero-production product/evidence \
  checkpoint over CAP-019's unchanged `exact-i32-array-v0` surface, composes the accepted \
  CAP-020 flat matvec and CAP-021 record-to-score product, and does not reverse CAP-022's \
  mandatory runtime-acquisition `NO IMPLEMENTATION` stop.";

const CAP023_APPLICATION_BOUNDARY: &str = "One source-embedded application convention \
  `[int; 20]` with exact header `[2, 3, 2]` drives a flat 3-input/2-hidden/2-output \
  wrapping-`i32` computation with strict-positive zero clamp, two biased logits, signed \
  strict-greater argmax, lower-index tie selection, three independent malformed-header \
  controls, and reread of all 140 source lanes after all seven by-value calls.";

const CAP023_ORACLE_BOUNDARY: &str = "Exact ordinary, wrapping, activation-boundary, and \
  tie results are respectively `[1, 122, 167, 135, 181, 4940, 5573, 1]`, \
  `[1, -24, 18, 2147483623, 0, -37, 2147483641, 1]`, \
  `[1, -3, 0, 0, 0, 5, 4, 0]`, and `[1, 1, 2, 1, 2, 3, 3, 0]`; malformed results \
  are eight zeros; public and native success is sentinel 91 with empty source stdout/stderr.";

const CAP023_EXCLUSION_BOUNDARY: &str = "CAP-023 adds no general activation, ReLU, \
  argmax, inference, tensor, matrix, record, recursive-array, runtime/file input, \
  serialization, quantization, conversion, stable layout/ABI, performance, \
  resource-usage, accelerator, safety, or language completion capability.";

const CAP023_HISTORY_BOUNDARY: &str = "Its record and topology are application \
  conventions, its retained local artifacts remain mutable corroboration only, and \
  CAP-019 remains the latest compiler/profile widening.";

const CAP023_CLASSIFICATION_BOUNDARY: &str = "The sole matrix change is the existing CPU \
  backend-summary row remaining `PARTIAL`; no CAP-023 language feature or selected-profile \
  row may be added.";

const CAP023_ALERT_BOUNDARY: &str = "CAP-023 merge \
  `e9b281504446465cfc8fcbe17c65cce92df0e83a` is an accepted historical product checkpoint, \
  not the current public master. The PR-only aggregate is correctly absent \
  on the default branch. Default-branch Actions analysis `1612715455` contains only the \
  pre-existing open alert #4 created 2026-08-09; Python and Rust analyses contain zero \
  results; no new CAP-023 code-scanning alert exists.";

const CAP023_MILESTONE_BOUNDARY: &str = "The selected Milestone 0, Milestone 1, and \
  Milestone 2 exit gates are met for their bounded selected products; their broader \
  milestone ambitions remain partial. Milestone 3 remains open. CAP-023 advances its \
  application and reproducibility boundary but supplies no runtime ingestion, composed \
  CopyData application profile, quantization, runtime-resource measurement, performance \
  evidence, accelerator execution, or broader workload.";

const CAP024_CURRENT_HEAD_BOUNDARY: &str = "Current accepted public master and public \
  evidence checkpoint is protected CAP-024 merge \
  `2f7ec325e423461a8e867f4ee2573ae6dcf15dfd`, tree \
  `9520f24e4f1626f16782a9775480f9653f6059bb`; its ordered parents are accepted base \
  `918c9222eb61e2435e18847e30b946cd08013238` then reviewed candidate \
  `617bfce86feb879ee5eef61b44cf4e2a5520f022`.";

const CAP024_ZERO_PRODUCTION_BOUNDARY: &str = "CAP-024 is the current accepted public \
  evidence checkpoint and protected public master. It adds no compiler production, parser, \
  grammar, source semantics, profile, semantic analysis, checked IR, verifier, backend, \
  example, product oracle, runtime behavior, ABI, capability classification, benchmark, \
  resource-usage, performance, accelerator, safety, or general-inference capability. Its \
  only claim is immutable accepted-head CAP-023 correctness, within-platform target-artifact \
  reproducibility, exact observable behavior, and artifact byte-size footprint under the \
  closed recorded boundary.";

const CAP024_CLASSIFICATION_BOUNDARY: &str = "CAP-019 remains the latest compiler/profile \
  widening; CAP-023 remains the latest product checkpoint. The selected \
  `exact-i32-array-v0` row and the existing CAP-023 CPU backend-summary row remain \
  byte-identical, and CAP-024 adds no language, selected-profile, or backend-summary row.";

const CAP024_BUNDLE_BOUNDARY: &str = "The accepted catalog record remains \
  `aero_cap023_inference_correctness_918c9222_20260813`, status \
  `verified_correctness_reproducibility_only`, with exactly the tracked schema, canonical \
  88,734-byte manifest SHA-256 \
  `4b4cfce95459761dddd588e09abb3046854e0c2afb361f08a9553f180f013a34`, oracle, and \
  reproduction contract.";

const CAP024_ALERT_BOUNDARY: &str = "The PR-only aggregate CodeQL check is correctly absent \
  on the default branch. Default-branch Actions analysis `1617281747` carries only the \
  pre-existing open alert #4 created and last updated 2026-08-09; Python and Rust analyses \
  contain zero results, and no new CAP-024 alert exists.";

const CAP024_MILESTONE_BOUNDARY: &str = "The selected Milestone 0, Milestone 1, and \
  Milestone 2 exits remain met for their bounded selected products; broader ambitions \
  remain partial. Milestone 3 remains open. CAP-024 closes the prior accepted-head \
  correctness/reproducibility/artifact-footprint gap, but supplies no runtime ingestion, \
  composed CopyData application profile, quantization, runtime-resource measurement, \
  performance evidence, accelerator execution, or broader workload.";

const CAP016_LOCAL_MODDECL_STOP_BOUNDARY: &str = "Block-local `mod missing;` remains a \
  demonstrated invalid-program false success because the common statement parser accepts \
  it, `ModDecl` has no source location, and semantic plus checked admission silently discard \
  it. CAP-016 already audited that exact defect and found that trustworthy \
  placement/provenance rejection participates in the unfrozen module migration across more \
  than two compiler phases. No new module RFC or decision-changing evidence exists, so \
  CAP-016 remains a mandatory `NO IMPLEMENTATION` stop until its explicit re-entry condition \
  is met.";

const CAP015_M1_BOUNDARY: &str = "CAP-015 remains the accepted M1-001 \
representative-integration checkpoint. CAP-015 changes no compiler production or \
language-profile code.";

const CAP016_CAP017_STOP_BOUNDARY: &str = "CAP-016 and CAP-017 remain completed \
readiness/architecture stops, not accepted capabilities; neither adds a profile or \
matrix row.";

const CAP014_CONFORMANCE_HISTORY_BOUNDARY: &str = "That retained checkpoint records \
CAP-014's originally excluded mutable, write, and construction cases without treating \
them as the current selected-profile boundary; current negative evidence must exhaust \
the families still excluded after CAP-018 and CAP-019";
const CAP018_CONFORMANCE_HISTORY_BOUNDARY: &str = "Its retained historical negatives \
record the mutable binding/result/write boundary before CAP-019; current negative \
separation retains only the mutable forms CAP-019 still excludes";

const POST_CAP020_RANKING_HEADER: &str = "| Rank | Capability gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Favorable risk | Favorable evidence cost | Total |";

const POST_CAP020_RANKING_ROWS: [&str; 3] = [
    "| 1 | Source-embedded fixed-shape tensor-record decode plus two-stage flat-buffer exact-`i32` CPU scoring product gate | 5 | 5 | 5 | 5 | 4 | 4 | 28 |",
    "| 2 | Runtime byte/file acquisition readiness and red probe under one cross-platform bounded-owned-buffer contract | 5 | 5 | 5 | 4 | 1 | 1 | 21 |",
    "| 3 | Recursive exact-`i32` array / 2D matrix readiness deferred pending one shared recursive-shape contract | 3 | 3 | 4 | 5 | 2 | 2 | 19 |",
];

const POST_CAP020_DECISION_CONTRACTS: [&str; 9] = [
    "Before rank 1, accepted CAP-020 executes one directly initialized 2x3-by-3 flat matvec, but no maintained Aero-native product validates and decodes a source-embedded tensor-shaped record or composes that result into a second numerical stage. After rank 1, one fixed `[int; 17]` record with exact header `[2, 3, 1]` and flat input/weight/bias payload must be validated, decoded through guarded reads and fully initialized flat-array writes, consumed by the accepted 2x3 matvec and a second exact-Int affine scoring stage, preserve and reread every source lane, and produce independent ordinary and wrapping oracles plus exact public and native sentinel 91.",
    "Stop and rerank rank 1 if the exact `[int; 17]` product needs any compiler production change, new language or profile rule, partial or uninitialized array state, unchecked indexing, new arithmetic or quantization semantics, stable layout or ABI, or duplicated guard or type authority.",
    "Evidence that the complete record-to-score program is not expressible solely through accepted CAP-020 semantics, is only a restatement of the single matvec, or cannot define independent record, header, source, result, and wrapping oracles changes rank 1; clean zero-production execution makes runtime acquisition, not recursive syntax, the next hard boundary.",
    "Before rank 2, Aero computations consume only source-embedded fixed data and no trusted source program acquires external bytes. After rank 2 readiness, a task-local cross-platform probe and architecture map must locate the first failure and freeze path and byte identity, capacity and initialized count, partial-read and EOF behavior, typed error mapping, ownership and drop, runtime linkage, sandboxing and determinism, and Linux and Windows behavior, either yielding one bounded implementation contract within two compiler phases or an explicit mandatory stop without claiming I/O capability.",
    "Stop rank 2 before implementation if any contract item remains unfrozen, if allocation, drop, or runtime ABI must be invented, if platform behavior cannot be made equivalent and observable, if a useful slice crosses more than two compiler phases, or if invalid acquisition can reach trusted IR or backend generation without typed failure.",
    "Evidence that a caller-provided bounded byte slice or source-embedded record feeds the flagship boundary sooner without filesystem or runtime semantics would defer rank 2 implementation; an explicit runtime RFC plus a probe demonstrating one shared cross-platform ownership and error authority within the phase limit would permit later implementation ranking.",
    "Before rank 3, CAP-020 proves the target 2D matvec through flat `[int; 6]` storage while `exact-i32-array-v0` deliberately rejects nested arrays. After rank 3 readiness, only if it is reopened, a task-local `[[int; 3]; 2]` red probe and topology map must freeze depth, dimension-product bounds, value placements, nested mutation and alias rules, and nested-versus-flat physical identity under one source and physical shape authority, or record a mandatory stop without claiming recursive arrays.",
    "Stop rank 3 before implementation while flat encoding serves the target workload, or if any recursive-shape decision remains unfrozen, admission and lowering cannot share one canonical shape, the slice exceeds two compiler phases, or it requires stable aggregate layout or ABI, aliases, or rank-specific classifiers.",
    "Evidence of a concrete workload that flat buffers materially obscure, together with an explicit bounded shape decision and a probe proving one shared source and physical authority within two phases, would restore recursive arrays to implementation ranking; CAP-020's clean flat execution otherwise keeps them deferred.",
];

const POST_CAP021_RANKING_HEADER: &str = "| Rank | Capability gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Favorable risk | Favorable evidence cost | Total |";

const POST_CAP021_RANKING_ROWS: [&str; 3] = [
    "| 1 | Runtime byte/file acquisition readiness and red probe under one cross-platform bounded-owned-buffer contract | 5 | 5 | 5 | 4 | 1 | 1 | 21 |",
    "| 2 | Small quantized numerical-kernel readiness and red probe under one frozen cross-platform arithmetic-and-representation contract | 5 | 5 | 3 | 5 | 1 | 1 | 20 |",
    "| 3 | Recursive exact-`i32` array / 2D matrix readiness deferred pending one shared recursive-shape contract | 3 | 3 | 4 | 5 | 2 | 2 | 19 |",
];

const POST_CAP021_DECISION_CONTRACTS: [&str; 9] = [
    "Before rank 1, accepted CAP-021 validates and scores one source-embedded fixed `[int; 17]` record, but no trusted Aero source program acquires external bytes. After rank 1 readiness, a task-local cross-platform probe and architecture map must locate the first failure and freeze path and byte identity, capacity and initialized count, partial-read and EOF behavior, typed error mapping, ownership and drop, runtime linkage, sandboxing and determinism, and Linux and Windows behavior, either yielding one bounded implementation contract within two compiler phases or an explicit mandatory stop without claiming I/O capability.",
    "Stop rank 1 before implementation if any contract item remains unfrozen, if allocation, drop, or runtime ABI must be invented, if platform behavior cannot be made equivalent and observable, if a useful slice crosses more than two compiler phases, or if invalid acquisition can reach trusted IR or backend generation without typed failure.",
    "Evidence that a caller-provided bounded byte slice can feed the accepted record-to-score boundary without filesystem or runtime acquisition semantics would narrow the readiness target or defer later rank 1 implementation; an explicit runtime RFC plus a probe demonstrating one shared cross-platform ownership and error authority within the phase limit would permit later implementation ranking.",
    "Before rank 2, accepted CAP-021 executes exact wrapping `i32` matvec, bias, and affine scoring, but Aero has no frozen quantized representation, conversion, or arithmetic contract and no maintained quantized oracle. After rank 2 readiness, a task-local source-embedded red probe and architecture map must locate the first failure and freeze stored, accumulator, and result types and domains; scale and zero-point presence, representation, and scope; rounding and tie behavior; saturation and overflow behavior; conversion boundaries and operation order; calibration provenance; malformed-state rejection; the reference oracle; and Linux and Windows equivalence, either yielding one bounded implementation contract within two compiler phases or an explicit mandatory stop without claiming quantization capability.",
    "Stop rank 2 before implementation if any arithmetic or representation decision remains unfrozen, if the slice requires implicit conversion, fallback typing, or a second numerical authority, if it silently changes CAP-021 wrapping order or semantics, if malformed quantization state can reach trusted IR or backend generation, if a useful slice crosses more than two compiler phases, or if deterministic Linux and Windows oracle parity cannot be proved.",
    "Evidence that external-byte ownership and error semantics must be established before a quantized oracle can be meaningful, or that an exact-`i32` kernel advances the next workload without lossy representation, would defer rank 2 implementation; an explicit quantization RFC plus a probe demonstrating one shared cross-platform representation, arithmetic, and error authority within the phase limit would permit later implementation ranking.",
    "Before rank 3, accepted CAP-021 proves fixed-record decode and two-stage scoring through flat `[int; 17]`, `[int; 6]`, `[int; 3]`, and `[int; 2]` storage while `exact-i32-array-v0` deliberately rejects nested arrays. After rank 3 readiness, only if it is reopened, a task-local `[[int; 3]; 2]` red probe and topology map must freeze depth, dimension-product bounds, value placements, nested mutation and alias rules, and nested-versus-flat physical identity under one source and physical shape authority, or record a mandatory stop without claiming recursive arrays.",
    "Stop rank 3 before implementation while flat encoding serves the target workload, or if any recursive-shape decision remains unfrozen, admission and lowering cannot share one canonical shape, the slice exceeds two compiler phases, or it requires stable aggregate layout or ABI, aliases, or rank-specific classifiers.",
    "Evidence of a concrete workload that flat buffers materially obscure, together with an explicit bounded shape decision and a probe proving one shared source and physical authority within two phases, would restore recursive arrays to implementation ranking; CAP-021's clean flat record-to-score execution otherwise keeps them deferred.",
];

const POST_CAP023_RANKING_HEADER: &str = "| Rank | Capability gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Favorable risk | Favorable evidence cost | Total |";

const POST_CAP023_RANKING_ROWS: [&str; 3] = [
    "| 1 | Accepted-head CAP-023 inference correctness/reproducibility/artifact-footprint evidence gate with no performance claim | 4 | 5 | 5 | 5 | 5 | 4 | 28 |",
    "| 2 | Exact CPU + recursive-CopyData application-profile composition readiness and red probe only | 5 | 5 | 5 | 5 | 2 | 2 | 24 |",
    "| 3 | Small quantized numerical-kernel readiness and red probe under one frozen cross-platform arithmetic-and-representation contract only | 5 | 5 | 3 | 5 | 1 | 1 | 20 |",
];

const POST_CAP023_DECISION_CONTRACTS: [&str; 9] = [
    "Before rank 1, accepted CAP-023 proves one source-embedded fixed-shape 3-input/2-hidden/2-output exact-`i32` ReLU-and-argmax CPU inference product through deterministic verified LLVM, `llvm-as`, machine verification, native `-O0`/`-O2`, public execution, and independent ordinary, wrapping, activation-boundary, tie, malformed-header, and source-preservation oracles, but `claim-verification/` contains no accepted-head Aero-native inference correctness/reproducibility record and no artifact-footprint manifest. After rank 1, one immutable accepted-head evidence bundle must record the exact commit and clean-tree state, source/input/oracle hashes, pinned Linux and Windows toolchains and commands, deterministic LLVM/bitcode/assembly/executable hashes and byte sizes, exact exit/stdout/stderr results, failures, limitations, and a complete third-party reproduction procedure without timing, throughput, speedup, memory, energy, or performance claims.",
    "Stop and rerank rank 1 if CAP-023 is not accepted at the exact protected merge head, any recorded artifact cannot be regenerated byte-for-byte within its stated platform/toolchain boundary, correctness depends on retained mutable local artifacts rather than tracked inputs and commands, Linux and Windows results diverge, or the gate would require compiler production, source/profile semantics, benchmark timing, or a public performance claim.",
    "Evidence that an existing immutable accepted-head bundle already supplies the same source/oracle/toolchain/command/hash/size/result/failure/limitation contract, that artifact bytes are nondeterministic for an unfrozen reason, or that footprint capture cannot be separated from benchmark semantics changes rank 1; a clean zero-production correctness/reproducibility bundle advances Milestone 3 evidence but does not meet its performance or complete resource-usage exit.",
    "Before rank 2, accepted CAP-023 executes a flat exact-`i32` application convention inside `exact-i32-array-v0`, while accepted recursive finite CopyData structs, enums, `Result`, `Match`, and ownership slices remain bounded `PARTIAL` experimental capabilities that the selected CPU profile deliberately rejects. After rank 2 readiness, a task-local source probe and architecture map must identify the first composition failure and freeze whether one new application profile can reuse the exact-`i32` scalar/flat-array physical authority together with only already-accepted recursive CopyData aggregate, typed-result, `Match`, and bounded ownership contracts; define admitted types and operations, phase ownership, profile selection, physical identity, rejection boundaries, verifier evidence, and Linux and Windows oracles; and yield either one bounded later implementation contract within two compiler phases or an explicit mandatory stop without widening either existing profile.",
    "Stop rank 2 before implementation if composition requires changing `stable-scalar-v0` or `exact-i32-array-v0`, importing broad experimental defaults, inventing struct, enum, `Result`, layout, or ABI semantics, reconciling duplicate type, physical, or specialization authorities, adding recursive or nested exact arrays, crossing more than two compiler phases, or claiming general CopyData, ownership, error handling, inference, or safety.",
    "Evidence that the CAP-023 workload can materially exercise existing CopyData aggregates and typed failure under one bounded profile without new semantics and with one shared exact physical/verifier authority raises rank 2 toward implementation; evidence that a flat record remains sufficient, that the application needs runtime ingress first, or that composition requires broad layout or ownership contracts defers it.",
    "Before rank 3, accepted CAP-023 proves exact wrapping `i32` matvec, positive-only ReLU, two biased logits, and signed strict-greater argmax, but Aero has no frozen quantized stored, accumulator, or result representation; scale or zero-point contract; conversion, rounding, tie, saturation, or overflow behavior; calibration provenance; malformed-state rule; or maintained cross-platform quantized oracle. After rank 3 readiness, a task-local source-embedded probe and architecture map must locate the first failure and freeze every such decision plus operation order and Linux/Windows equivalence, yielding either one bounded later implementation contract within two compiler phases or an explicit mandatory stop without claiming quantization capability.",
    "Stop rank 3 before implementation if any arithmetic or representation decision remains unfrozen; if the slice requires implicit conversion, fallback typing, unfounded division or rounding semantics, or a second numerical authority; if the existing scalar-double helper is treated as source-language proof; if CAP-023 wrapping order changes; if malformed quantization state can reach trusted IR or backend generation; if the slice crosses more than two compiler phases; or if deterministic Linux and Windows oracle parity cannot be proved.",
    "Evidence that accepted-head artifact evidence and exact CPU plus CopyData application composition must precede a meaningful quantized oracle, or that exact `i32` continues to advance the next workload without lossy representation, keeps rank 3 at readiness scope; only an explicit quantization RFC plus a probe demonstrating one shared cross-platform representation, arithmetic, malformed-state, and oracle authority within the phase limit raises it toward implementation.",
];

const POST_CAP024_RANKING_HEADER: &str = "| Rank | Capability gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Favorable risk | Favorable evidence cost | Total |";

const POST_CAP024_RANKING_ROWS: [&str; 3] = [
    "| 1 | Exact CPU + recursive-CopyData application-profile composition readiness and red probe only | 5 | 5 | 5 | 5 | 2 | 2 | 24 |",
    "| 2 | Owned dynamic collection/streaming foundation readiness and red probe, including its shared allocation/ownership/drop architecture, only | 5 | 5 | 5 | 5 | 1 | 1 | 22 |",
    "| 3 | Small quantized numerical-kernel readiness and red probe under one frozen cross-platform arithmetic-and-representation contract only | 5 | 5 | 3 | 5 | 1 | 1 | 20 |",
];

const POST_CAP024_DECISION_CONTRACTS: [&str; 9] = [
    "Before rank 1, CAP-024 proves the exact accepted flat CAP-023 application and its immutable correctness/reproducibility boundary, while recursive finite CopyData structs, enums, typed `Result`, `Match`, and ownership slices remain separate bounded `PARTIAL` experimental authorities rejected by `exact-i32-array-v0`. After rank 1 readiness, a task-local source probe and architecture map must identify the first composition failure and freeze whether one new application profile can reuse exact-`i32` scalar/flat-array physical authority together with only already-accepted recursive CopyData, typed-result, `Match`, and bounded ownership contracts; define admitted types and operations, phase ownership, profile selection, physical identity, rejection boundaries, verifier evidence, and Linux and Windows oracles; and yield either one bounded later implementation contract within two compiler phases or an explicit mandatory stop without widening either existing profile.",
    "Stop rank 1 before implementation if composition requires changing `stable-scalar-v0` or `exact-i32-array-v0`, importing broad experimental defaults, inventing struct, enum, `Result`, layout, ABI, ownership, or error semantics, reconciling duplicate type, physical, or specialization authorities, adding recursive or nested exact arrays, crossing more than two compiler phases, or claiming general CopyData, inference, safety, or language completion.",
    "Evidence that the CAP-023 workload can materially exercise existing CopyData aggregates and typed failure under one bounded profile without new semantics and with one shared exact physical/verifier authority raises rank 1 toward later implementation; evidence that a flat record remains sufficient, that runtime ingress is prerequisite, or that composition requires broad layout or ownership contracts defers it and changes the decision.",
    "Before rank 2, accepted CAP-011 provides one fixed-capacity recursive-CopyData `Window<T>` algorithm and the representative program composes only statically bounded storage; Aero has no accepted owned dynamic collection, allocation, capacity growth, initialized-length, reallocation, alias, failure, or drop contract. Legacy `stdlib.rs` String/Vec helpers and their rejected checked-IR/backend instructions are not source-language authority. After rank 2 readiness, a task-local owned-collection/streaming source probe and architecture map must first freeze the public type/API name, then the minimal useful element class and operations; length/capacity/growth and initialized-state rules; allocation, failure, move/borrow/alias, reallocation, iteration/indexing, and drop behavior; one physical and verifier authority; rejection boundaries; and deterministic Linux and Windows oracles, yielding either one bounded later implementation contract within two compiler phases or a mandatory stop without claiming dynamic collections.",
    "Stop rank 2 before implementation if allocation, OOM/error, ownership, alias, reallocation invalidation, lifetime, drop, runtime ABI, or element destruction semantics remain unfrozen; if uninitialized elements can become observable; if legacy unchecked helpers or verifier-rejected instructions would be activated; if the useful slice crosses more than two compiler phases; or if invalid collection state can reach trusted IR/backend or Linux and Windows behavior cannot be made equivalent and observable.",
    "Evidence that fixed-capacity `Window<T>` plus flat source records serves the next useful workload, that runtime ingress is prerequisite, or that one owned collection requires broad allocator/drop/lifetime architecture keeps rank 2 at readiness scope; only an explicit collection RFC plus a probe demonstrating one shared cross-platform initialized-state, ownership, physical, error, and verifier authority within the phase limit raises it toward later implementation.",
    "Before rank 3, CAP-024 preserves exact wrapping `i32` matvec, positive-only zero clamp, two biased logits, and signed strict-greater argmax, but Aero has no frozen quantized stored, accumulator, or result representation; scale or zero-point contract; conversion, rounding, tie, saturation, or overflow behavior; calibration provenance; malformed-state rule; or maintained cross-platform quantized oracle. After rank 3 readiness, a task-local source-embedded red probe and architecture map must locate the first failure and freeze every such decision plus operation order and Linux/Windows equivalence, yielding either one bounded later implementation contract within two compiler phases or an explicit mandatory stop without claiming quantization capability.",
    "Stop rank 3 before implementation if any arithmetic or representation decision remains unfrozen; if the slice requires implicit conversion, fallback typing, unfounded division or rounding semantics, or a second numerical authority; if the scalar-double helper is treated as source-language proof; if CAP-023 wrapping order changes; if malformed quantization state can reach trusted IR or backend generation; if the slice crosses more than two compiler phases; or if deterministic Linux and Windows oracle parity cannot be proved.",
    "Evidence that exact CPU plus CopyData application composition must precede a meaningful quantized oracle, or that exact `i32` continues to advance the next workload without lossy representation, keeps rank 3 at readiness scope; only an explicit quantization RFC plus a probe demonstrating one shared cross-platform representation, arithmetic, malformed-state, and oracle authority within the phase limit raises it toward later implementation.",
];

const CAP021_CPU_MATRIX_ROW: &str = "| CPU | Y | Y | P | P | P; pinned Linux and bounded Windows x86_64 evidence accepted, including CAP-014 exact-i32-array-v0 kernel/wrapping/read-trap gates, CAP-018 immutable result composition, CAP-019 initialized mutable-local/result production with guarded projected writes and negative/equal write traps, CAP-020 flat-buffer 2x3-by-3 matvec product with identity-linked guarded [6]/[3]/[2] access and exact ordinary/wrapping/native oracles, and CAP-021 source-embedded flat [17]-lane record decode plus two-stage exact-i32 scoring with header/ordinary/wrapping/malformed/source-preservation oracles | P | P | PARTIAL |";

const CAP019_SELECTED_PROFILE_MATRIX_ROW: &str = "| Selected CPU-only `exact-i32-array-v0` profile (created by accepted `CAP-014`; widened by accepted `CAP-018` and accepted `CAP-019`) | Y | Y | Y | Y | Y | — | Y | Y | Y | Y | Y | Y | Y | END_TO_END |";

const CAP023_CPU_MATRIX_ROW: &str = "| CPU | Y | Y | P | P | P; pinned Linux and bounded Windows x86_64 evidence accepted, including CAP-014 exact-i32-array-v0 kernel/wrapping/read-trap gates, CAP-018 immutable result composition, CAP-019 initialized mutable-local/result production with guarded projected writes and negative/equal write traps, CAP-020 flat-buffer 2x3-by-3 matvec product with identity-linked guarded [6]/[3]/[2] access and exact ordinary/wrapping/native oracles, CAP-021 source-embedded flat [17]-lane record decode plus two-stage exact-i32 scoring with header/ordinary/wrapping/malformed/source-preservation oracles, and CAP-023 source-embedded flat [int; 20] 3-input/2-hidden/2-output exact-i32 zero-clamp/argmax product with header/ordinary/wrapping/activation-boundary/tie/malformed/source-preservation oracles | P | P | PARTIAL |";

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn repository_file(path: &str) -> String {
    let full_path = repository_root().join(path);
    fs::read_to_string(&full_path)
        .unwrap_or_else(|error| panic!("could not read {}: {error}", full_path.display()))
}

fn normalized_words(document: &str) -> String {
    document
        .lines()
        .map(|line| line.trim_start().strip_prefix('>').unwrap_or(line).trim())
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Clone, Copy)]
struct MarkdownFence {
    marker: char,
    length: usize,
    blockquote_depth: usize,
    list_indent: Option<usize>,
}

fn strip_blockquote_prefixes(line: &str) -> &str {
    blockquote_depth_and_content(line).1
}

fn blockquote_depth_and_content(mut line: &str) -> (usize, &str) {
    let mut depth = 0;
    loop {
        let candidate = line.trim_start_matches(' ');
        if candidate.len() + 3 < line.len() || !candidate.starts_with('>') {
            return (depth, line);
        }
        depth += 1;
        line = candidate[1..].strip_prefix(' ').unwrap_or(&candidate[1..]);
    }
}

fn valid_inline_html_tag_name(name: &str) -> bool {
    matches!(
        name,
        "a" | "abbr"
            | "b"
            | "br"
            | "code"
            | "del"
            | "em"
            | "i"
            | "kbd"
            | "mark"
            | "q"
            | "s"
            | "samp"
            | "small"
            | "span"
            | "strong"
            | "sub"
            | "sup"
            | "time"
            | "u"
            | "var"
            | "address"
            | "article"
            | "aside"
            | "base"
            | "basefont"
            | "blockquote"
            | "body"
            | "caption"
            | "center"
            | "col"
            | "colgroup"
            | "dd"
            | "details"
            | "dialog"
            | "dir"
            | "div"
            | "dl"
            | "dt"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "frame"
            | "frameset"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "head"
            | "header"
            | "hr"
            | "html"
            | "iframe"
            | "legend"
            | "li"
            | "link"
            | "main"
            | "menu"
            | "menuitem"
            | "nav"
            | "noframes"
            | "ol"
            | "optgroup"
            | "option"
            | "p"
            | "param"
            | "search"
            | "section"
            | "summary"
            | "table"
            | "tbody"
            | "td"
            | "tfoot"
            | "th"
            | "thead"
            | "title"
            | "tr"
            | "track"
            | "ul"
    )
}

fn valid_html_tag_tail(tail: &str, closing: bool) -> bool {
    let mut rest = tail.trim();
    if closing {
        return rest.is_empty();
    }
    if rest == "/" || rest.is_empty() {
        return true;
    }
    while !rest.is_empty() {
        if rest == "/" {
            return true;
        }
        let name_len = rest
            .chars()
            .take_while(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '_' | ':' | '.' | '-')
            })
            .map(char::len_utf8)
            .sum::<usize>();
        if name_len == 0
            || !rest[..name_len].chars().next().is_some_and(|character| {
                character.is_ascii_alphabetic() || matches!(character, '_' | ':')
            })
        {
            return false;
        }
        rest = rest[name_len..].trim_start();
        if let Some(value) = rest.strip_prefix('=') {
            rest = value.trim_start();
            let Some(first) = rest.chars().next() else {
                return false;
            };
            if matches!(first, '\'' | '"') {
                let Some(end) = rest[first.len_utf8()..].find(first) else {
                    return false;
                };
                rest = rest[first.len_utf8() + end + first.len_utf8()..].trim_start();
            } else {
                let value_len = rest
                    .chars()
                    .take_while(|character| {
                        !character.is_whitespace()
                            && !matches!(character, '\'' | '"' | '=' | '<' | '>' | '`')
                    })
                    .map(char::len_utf8)
                    .sum::<usize>();
                if value_len == 0 {
                    return false;
                }
                rest = rest[value_len..].trim_start();
            }
        }
    }
    true
}

fn html_tag_end(tag: &str) -> Option<usize> {
    let mut quote = None;
    for (position, character) in tag.char_indices() {
        if let Some(expected) = quote {
            if character == expected {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '>' => return Some(position),
            _ => {}
        }
    }
    None
}

fn valid_complete_html_tag(tag: &str) -> Option<(String, bool)> {
    if !tag.starts_with('<') || !tag.ends_with('>') {
        return None;
    }
    let raw_inside = &tag[1..tag.len() - 1];
    let closing = raw_inside.starts_with('/');
    let inside = raw_inside.strip_prefix('/').unwrap_or(raw_inside);
    let name_len = inside
        .chars()
        .take_while(|character| character.is_ascii_alphanumeric() || *character == '-')
        .map(char::len_utf8)
        .sum::<usize>();
    let tag_name = inside[..name_len].to_ascii_lowercase();
    (valid_inline_html_tag_name(&tag_name) && valid_html_tag_tail(&inside[name_len..], closing))
        .then_some((tag_name, closing))
}

fn html_heading_open_level(line: &str) -> Option<usize> {
    let trimmed = line.trim();
    let end = html_tag_end(trimmed)?;
    if !trimmed[end + 1..].trim().is_empty() {
        return None;
    }
    let (name, closing) = valid_complete_html_tag(&trimmed[..=end])?;
    (!closing && name.len() == 2 && name.starts_with('h'))
        .then(|| name[1..].parse::<usize>().ok())
        .flatten()
        .filter(|level| (1..=6).contains(level))
}

fn normalized_reference_label(label: &str) -> String {
    label
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn valid_link_label_before(line: &str, closing: usize) -> bool {
    line[..closing]
        .rfind('[')
        .is_some_and(|opening| !line[opening + 1..closing].contains(']'))
}

fn inline_link_destination_end(line: &str, start: usize) -> Option<usize> {
    let mut index = start;
    let mut depth = 1_u32;
    let mut angle = false;
    if line[index..].starts_with('<') {
        angle = true;
        index += 1;
    }
    while index < line.len() {
        let character = line[index..].chars().next()?;
        match character {
            '\\' => {
                index += character.len_utf8();
                if let Some(escaped) = line[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            }
            '<' | '>' if !angle => return None,
            '>' if angle => {
                index += 1;
                if line[index..].starts_with(')') {
                    return Some(index + 1);
                }
                return None;
            }
            character if character.is_whitespace() => return None,
            '(' if !angle => {
                depth += 1;
                index += character.len_utf8();
            }
            ')' if !angle => {
                depth -= 1;
                index += character.len_utf8();
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => index += character.len_utf8(),
        }
    }
    None
}

fn valid_reference_definition_suffix(suffix: &str) -> bool {
    let suffix = suffix.trim();
    if suffix.is_empty() {
        return false;
    }
    let valid_title = |title: &str| {
        let title = title.trim();
        let Some(opener) = title.chars().next() else {
            return false;
        };
        let closer = match opener {
            '"' => '"',
            '\'' => '\'',
            '(' => ')',
            _ => return false,
        };
        if title.len() < opener.len_utf8() + closer.len_utf8() || !title.ends_with(closer) {
            return false;
        }
        let body = &title[opener.len_utf8()..title.len() - closer.len_utf8()];
        let mut escaped = false;
        for character in body.chars() {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == closer {
                return false;
            }
        }
        !escaped
    };
    if let Some(angle) = suffix.strip_prefix('<') {
        return angle.find('>').is_some_and(|end| {
            if end == 0
                || angle[..end]
                    .chars()
                    .any(|character| character.is_whitespace() || matches!(character, '<' | '>'))
            {
                return false;
            }
            let remainder = &angle[end + 1..];
            remainder.trim().is_empty()
                || remainder.chars().next().is_some_and(char::is_whitespace)
                    && valid_title(remainder)
        });
    }
    let mut depth = 0_i32;
    let mut escaped = false;
    let mut destination_end = suffix.len();
    for (position, character) in suffix.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match character {
            '\\' => escaped = true,
            '(' => depth += 1,
            ')' if depth > 0 => depth -= 1,
            ')' => return false,
            character if character.is_whitespace() && depth == 0 => {
                destination_end = position;
                break;
            }
            '<' | '>' => return false,
            _ => {}
        }
    }
    if depth != 0 || destination_end == 0 {
        return false;
    }
    let remainder = &suffix[destination_end..];
    remainder.trim().is_empty() || valid_title(remainder)
}

fn reference_definition_metadata(document: &str) -> (Vec<String>, Vec<bool>) {
    let lines = document.lines().collect::<Vec<_>>();
    let mut labels = Vec::new();
    let mut hidden = vec![false; lines.len()];
    let mut index = 0;
    while index < lines.len() {
        let line = strip_blockquote_prefixes(lines[index]);
        let indentation = line.len() - line.trim_start_matches(' ').len();
        let trimmed = line.trim_start();
        if indentation > 3 || !trimmed.starts_with('[') {
            index += 1;
            continue;
        }
        let Some(label_end) = trimmed.find("]:") else {
            index += 1;
            continue;
        };
        if label_end <= 1 || trimmed[1..label_end].contains(']') {
            index += 1;
            continue;
        }
        let first_suffix = &trimmed[label_end + 2..];
        let mut candidate = first_suffix.to_owned();
        let mut valid_end = valid_reference_definition_suffix(&candidate).then_some(index);
        for continuation in index + 1..(index + 32).min(lines.len()) {
            let continuation_line = strip_blockquote_prefixes(lines[continuation]);
            if continuation_line.trim().is_empty() {
                break;
            }
            candidate.push('\n');
            candidate.push_str(continuation_line);
            if valid_reference_definition_suffix(&candidate) {
                valid_end = Some(continuation);
            }
        }
        let Some(end) = valid_end else {
            index += 1;
            continue;
        };
        labels.push(normalized_reference_label(&trimmed[1..label_end]));
        hidden[index..=end].fill(true);
        index = end + 1;
    }
    (labels, hidden)
}

fn strip_list_marker(line: &str) -> &str {
    let indentation = line.len() - line.trim_start_matches(' ').len();
    if indentation > 3 {
        return line;
    }
    let trimmed = &line[indentation..];
    if matches!(trimmed.as_bytes().get(0), Some(b'-' | b'*' | b'+'))
        && matches!(trimmed.as_bytes().get(1), Some(b' ' | b'\t'))
    {
        return &trimmed[2..];
    }
    let digits = trimmed
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .count();
    let ordered_suffix = trimmed[digits..]
        .strip_prefix('.')
        .or_else(|| trimmed[digits..].strip_prefix(')'));
    if digits > 0
        && digits <= 9
        && ordered_suffix
            .is_some_and(|suffix| matches!(suffix.as_bytes().first(), Some(b' ' | b'\t')))
    {
        return &trimmed[digits + 2..];
    }
    line
}

fn list_marker(line: &str) -> Option<(usize, &str, Option<u64>)> {
    let indentation = line.len() - line.trim_start_matches(' ').len();
    if indentation > 3 {
        return None;
    }
    let trimmed = &line[indentation..];
    if matches!(trimmed.as_bytes().get(0), Some(b'-' | b'*' | b'+'))
        && matches!(trimmed.as_bytes().get(1), Some(b' ' | b'\t'))
    {
        return Some((indentation + 2, &trimmed[2..], None));
    }
    let digits = trimmed
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .count();
    let suffix = trimmed[digits..]
        .strip_prefix('.')
        .or_else(|| trimmed[digits..].strip_prefix(')'))?;
    (digits > 0 && digits <= 9 && matches!(suffix.as_bytes().first(), Some(b' ' | b'\t'))).then(
        || {
            (
                indentation + digits + 2,
                &trimmed[digits + 2..],
                trimmed[..digits].parse::<u64>().ok(),
            )
        },
    )
}

fn strip_indentation(line: &str, width: usize) -> &str {
    let mut consumed_width = 0;
    let mut consumed_bytes = 0;
    for (offset, character) in line.char_indices() {
        let next_width = match character {
            ' ' => consumed_width + 1,
            '\t' => consumed_width + 4 - (consumed_width % 4),
            _ => break,
        };
        if next_width > width {
            break;
        }
        consumed_width = next_width;
        consumed_bytes = offset + character.len_utf8();
        if consumed_width == width {
            break;
        }
    }
    &line[consumed_bytes..]
}

fn fence_marker(line: &str) -> Option<(char, usize, &str)> {
    let indentation = line.len() - line.trim_start_matches(' ').len();
    if indentation > 3 {
        return None;
    }
    let line = &line[indentation..];
    let marker = line
        .chars()
        .next()
        .filter(|marker| matches!(marker, '`' | '~'))?;
    let length = line
        .chars()
        .take_while(|candidate| *candidate == marker)
        .count();
    (length >= 3).then(|| (marker, length, &line[length..]))
}

fn commonmark_indentation(line: &str) -> usize {
    let mut width = 0;
    for character in line.chars() {
        match character {
            ' ' => width += 1,
            '\t' => width += 4 - (width % 4),
            _ => break,
        }
    }
    width
}

fn visible_without_html_comments(
    line: &str,
    html_comment: &mut bool,
    code_span: &mut Option<usize>,
    inline_html: &mut Option<String>,
    reference_labels: &[String],
    completed_html_heading: &mut Option<usize>,
) -> String {
    let mut visible = String::new();
    let combined = inline_html.take().map(|prefix| format!("{prefix}\n{line}"));
    let line = combined.as_deref().unwrap_or(line);
    let mut index = 0;
    if combined.is_some() {
        let Some(end) = html_tag_end(line) else {
            *inline_html = Some(line.to_owned());
            return visible;
        };
        let candidate = &line[..=end];
        if let Some((name, closing)) = valid_complete_html_tag(candidate) {
            if !closing && name.len() == 2 && name.starts_with('h') {
                *completed_html_heading = name[1..]
                    .parse::<usize>()
                    .ok()
                    .filter(|level| (1..=6).contains(level));
            }
        } else {
            visible.push_str(candidate);
        }
        index = end + 1;
    }
    while index < line.len() {
        if *html_comment {
            if let Some(end) = line[index..].find("-->") {
                index += end + 3;
                *html_comment = false;
            } else {
                break;
            }
            continue;
        }
        if code_span.is_none() && line[index..].starts_with('\\') {
            let mut characters = line[index..].char_indices();
            let (_, slash) = characters.next().expect("escape character");
            visible.push(slash);
            if let Some((_, escaped)) = characters.next() {
                visible.push(escaped);
                index += slash.len_utf8() + escaped.len_utf8();
            } else {
                index += slash.len_utf8();
            }
            continue;
        }
        if line[index..].starts_with('`') {
            let length = line[index..]
                .chars()
                .take_while(|character| *character == '`')
                .count();
            if *code_span == Some(length) {
                *code_span = None;
            } else if code_span.is_none() {
                *code_span = Some(length);
            }
            visible.push_str(&line[index..index + length]);
            index += length;
            continue;
        }
        if code_span.is_none() && line[index..].starts_with("<!--") {
            *html_comment = true;
            index += 4;
            continue;
        }
        if code_span.is_none()
            && line[index..].starts_with("](")
            && valid_link_label_before(line, index)
        {
            let Some(end) = inline_link_destination_end(line, index + 2) else {
                visible.push(']');
                index += 1;
                continue;
            };
            visible.push(']');
            index = end;
            continue;
        }
        if code_span.is_none()
            && line[index..].starts_with("][")
            && valid_link_label_before(line, index)
        {
            if let Some(end) = line[index + 2..].find(']') {
                let label = normalized_reference_label(&line[index + 2..index + 2 + end]);
                if reference_labels.contains(&label) {
                    visible.push(']');
                    index += end + 3;
                    continue;
                }
            }
        }
        if code_span.is_none() && line[index..].starts_with('<') {
            let raw_inside = &line[index + 1..];
            let inside = raw_inside.strip_prefix('/').unwrap_or(raw_inside);
            let name_len = inside
                .chars()
                .take_while(|character| character.is_ascii_alphanumeric() || *character == '-')
                .map(char::len_utf8)
                .sum::<usize>();
            let tag_name = inside[..name_len].to_ascii_lowercase();
            if valid_inline_html_tag_name(&tag_name) {
                let Some(relative_end) = html_tag_end(&line[index..]) else {
                    *inline_html = Some(line[index..].to_owned());
                    break;
                };
                let candidate = &line[index..=index + relative_end];
                if let Some((validated_name, validated_closing)) =
                    valid_complete_html_tag(candidate)
                {
                    if !validated_closing
                        && validated_name.starts_with('h')
                        && validated_name.len() == 2
                    {
                        visible.push_str(&"#".repeat(validated_name[1..].parse().unwrap_or(1)));
                        visible.push(' ');
                    }
                    index += relative_end + 1;
                    continue;
                }
            }
        }
        let character = line[index..].chars().next().expect("visible character");
        visible.push(character);
        index += character.len_utf8();
    }
    visible
}

fn decode_contract_entities(line: &str) -> String {
    let mut decoded = String::with_capacity(line.len());
    let mut offset = 0;
    while let Some(relative) = line[offset..].find('&') {
        let start = offset + relative;
        decoded.push_str(&line[offset..start]);
        let Some(end_relative) = line[start + 1..].find(';') else {
            decoded.push_str(&line[start..]);
            return decoded;
        };
        let end = start + 1 + end_relative;
        let entity = &line[start + 1..end];
        if entity.is_empty()
            || entity.len() > 32
            || !entity
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '#')
        {
            decoded.push('&');
            offset = start + 1;
            continue;
        }
        let value = entity
            .strip_prefix("#x")
            .or_else(|| entity.strip_prefix("#X"))
            .and_then(|hex| u32::from_str_radix(hex, 16).ok())
            .or_else(|| {
                entity
                    .strip_prefix('#')
                    .and_then(|decimal| decimal.parse().ok())
            });
        if let Some(character) = value.and_then(char::from_u32) {
            decoded.push(character);
        } else if matches!(
            entity.to_ascii_lowercase().as_str(),
            "hyphen" | "minus" | "ndash" | "mdash" | "dash"
        ) {
            decoded.push('-');
        } else if matches!(
            entity.to_ascii_lowercase().as_str(),
            "nbsp"
                | "tab"
                | "newline"
                | "ensp"
                | "emsp"
                | "emsp13"
                | "emsp14"
                | "numsp"
                | "puncsp"
                | "thinsp"
                | "hairsp"
                | "mediumspace"
                | "negativemediumspace"
                | "negativethickspace"
                | "negativethinspace"
                | "negativeverythinspace"
                | "thickspace"
                | "verythinspace"
        ) {
            decoded.push(' ');
        } else {
            // CommonMark resolves the full HTML named-character-reference set.
            // Treat unneeded named values as semantic separators so their source
            // names cannot split a rendered capability phrase.
            decoded.push(' ');
        }
        offset = end + 1;
    }
    decoded.push_str(&line[offset..]);
    decoded
}

fn markdown_outside_fences(document: &str) -> String {
    let mut fence: Option<MarkdownFence> = None;
    let mut html_comment = false;
    let mut code_span = None;
    let mut inline_html: Option<String> = None;
    let (reference_labels, reference_definition_lines) = reference_definition_metadata(document);
    let mut list_content_indent = None;
    let mut paragraph_open = false;
    let mut html_heading = None;
    let mut rendered = String::with_capacity(document.len());
    for (line_index, raw_line) in document.lines().enumerate() {
        let (blockquote_depth, quoted_line) = blockquote_depth_and_content(raw_line);
        if fence.is_some_and(|open| {
            blockquote_depth < open.blockquote_depth
                || open.list_indent.is_some_and(|indent| {
                    !quoted_line.trim().is_empty()
                        && (list_marker(quoted_line).is_some()
                            || commonmark_indentation(quoted_line) < indent)
                })
        }) {
            fence = None;
        }
        let line = quoted_line;
        let marker = list_marker(line).filter(|(_, _, ordinal)| {
            !paragraph_open
                || list_content_indent.is_some()
                || ordinal.is_none()
                || *ordinal == Some(1)
        });
        let (line, next_list_indent, starts_list_item) = if let Some((indent, content, _)) = marker
        {
            (content, Some(indent), true)
        } else if let Some(indent) =
            list_content_indent.filter(|indent| commonmark_indentation(line) >= *indent)
        {
            (strip_indentation(line, indent), Some(indent), false)
        } else {
            (line, None, false)
        };
        if starts_list_item {
            rendered.push('\n');
            code_span = None;
            inline_html = None;
            paragraph_open = false;
        }
        if reference_definition_lines
            .get(line_index)
            .copied()
            .unwrap_or(false)
        {
            rendered.push('\n');
            paragraph_open = false;
            continue;
        }
        if line.trim().is_empty() {
            code_span = None;
            if let Some(unclosed) = inline_html.take() {
                rendered.push_str(&unclosed);
                rendered.push('\n');
            }
            paragraph_open = false;
        }
        if !line.trim().is_empty() {
            list_content_indent = next_list_indent;
        }
        let fence_line = strip_list_marker(line);
        if let Some(open) = fence {
            let closes = fence_marker(fence_line).is_some_and(|(marker, length, suffix)| {
                marker == open.marker
                    && length >= open.length
                    && suffix.chars().all(char::is_whitespace)
            });
            if closes {
                fence = None;
            }
            rendered.push('\n');
            continue;
        }
        if commonmark_indentation(line) >= 4 && !paragraph_open {
            rendered.push('\n');
            continue;
        }
        if let Some((marker, length, suffix)) = fence_marker(fence_line) {
            let valid_opener = marker == '~' || !suffix.contains('`');
            if valid_opener {
                fence = Some(MarkdownFence {
                    marker,
                    length,
                    blockquote_depth,
                    list_indent: next_list_indent.or(list_content_indent),
                });
                rendered.push('\n');
                continue;
            }
        }

        let trimmed = line.trim_start();
        if let Some(level) = html_heading_open_level(trimmed) {
            html_heading = Some(level);
            rendered.push('\n');
            paragraph_open = false;
            continue;
        }
        if html_heading
            .is_some_and(|level| trimmed.trim().eq_ignore_ascii_case(&format!("</h{level}>")))
        {
            html_heading = None;
            rendered.push('\n');
            paragraph_open = false;
            continue;
        }
        let mut completed_html_heading = None;
        let visible = visible_without_html_comments(
            line,
            &mut html_comment,
            &mut code_span,
            &mut inline_html,
            &reference_labels,
            &mut completed_html_heading,
        );
        let mut visible = decode_contract_entities(&visible);
        if let Some(level) = completed_html_heading {
            html_heading = Some(level);
        }
        if !visible.trim().is_empty()
            && let Some(level) = html_heading
        {
            visible.insert_str(0, &format!("{} ", "#".repeat(level)));
        }
        if !visible.trim().is_empty() {
            paragraph_open = true;
        }
        rendered.push_str(&visible);
        rendered.push('\n');
    }
    if let Some(unclosed) = inline_html {
        rendered.push_str(&unclosed);
        rendered.push('\n');
    }
    rendered
}

fn markdown_with_ordered_list_ranks(document: &str) -> String {
    let mut annotated = String::with_capacity(document.len());
    let mut paragraph_open = false;
    let mut ordered_list_open = false;
    for raw_line in document.lines() {
        let line = strip_blockquote_prefixes(raw_line);
        let indentation = line.len() - line.trim_start_matches(' ').len();
        let trimmed = &line[indentation..];
        let digits = trimmed
            .chars()
            .take_while(|character| character.is_ascii_digit())
            .count();
        let ordered = (digits > 0 && digits <= 9 && indentation <= 3)
            .then(|| {
                trimmed[digits..]
                    .strip_prefix('.')
                    .or_else(|| trimmed[digits..].strip_prefix(')'))
                    .filter(|suffix| matches!(suffix.as_bytes().first(), Some(b' ' | b'\t')))
                    .map(|suffix| (&trimmed[..digits], suffix.trim_start()))
            })
            .flatten()
            .filter(|(rank, _)| {
                !paragraph_open
                    || ordered_list_open
                    || rank.parse::<u64>().ok().is_some_and(|rank| rank == 1)
            });
        if let Some((rank, content)) = ordered {
            annotated.push_str(&" ".repeat(indentation));
            annotated.push_str("- Rank ");
            let normalized_rank = rank.parse::<u32>().unwrap_or_default().to_string();
            annotated.push_str(&normalized_rank);
            annotated.push(' ');
            annotated.push_str(content);
            paragraph_open = true;
            ordered_list_open = true;
        } else {
            annotated.push_str(line);
            if line.trim().is_empty() {
                paragraph_open = false;
                ordered_list_open = false;
            } else {
                paragraph_open = true;
                ordered_list_open = false;
            }
        }
        annotated.push('\n');
    }
    markdown_outside_fences(&annotated)
}

fn table_line(line: &str) -> &str {
    line.trim_start().strip_prefix('>').unwrap_or(line).trim()
}

fn normalized_markdown_paragraphs(document: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut current = Vec::new();
    for line in document.lines() {
        let line = table_line(line);
        if line.is_empty() {
            if !current.is_empty() {
                paragraphs.push(current.join(" "));
                current.clear();
            }
        } else {
            current.push(line);
        }
    }
    if !current.is_empty() {
        paragraphs.push(current.join(" "));
    }
    paragraphs
}

fn normalized_claim_records(document: &str) -> Vec<String> {
    let rendered = markdown_outside_fences(document);
    normalized_claim_records_from_rendered(&rendered)
}

fn normalized_claim_records_from_rendered(rendered: &str) -> Vec<String> {
    let mut records = Vec::new();
    let mut prose = Vec::new();
    let flush_prose = |records: &mut Vec<String>, prose: &mut Vec<&str>| {
        if !prose.is_empty() {
            let setext = (prose.len() >= 2)
                .then(|| prose.last())
                .flatten()
                .and_then(|line| {
                    let line = line.trim();
                    let marker = line.chars().next()?;
                    (line.len() >= 3
                        && matches!(marker, '=' | '-')
                        && line.chars().all(|candidate| candidate == marker))
                    .then_some(if marker == '=' { 1 } else { 2 })
                });
            if let Some(level) = setext {
                records.push(format!(
                    "{} {}",
                    "#".repeat(level),
                    prose[..prose.len() - 1].join(" ")
                ));
            } else {
                records.push(prose.join(" "));
            }
            prose.clear();
        }
    };
    for line in rendered.lines() {
        let line = table_line(line);
        if line.is_empty() {
            flush_prose(&mut records, &mut prose);
        } else if line.starts_with('|') {
            flush_prose(&mut records, &mut prose);
            records.push(line.to_owned());
        } else if atx_heading_level(line).is_some() {
            flush_prose(&mut records, &mut prose);
            records.push(line.to_owned());
        } else {
            prose.push(line);
        }
    }
    flush_prose(&mut records, &mut prose);
    records
}

fn atx_heading_level(record: &str) -> Option<usize> {
    let trimmed = record.trim_start();
    let level = trimmed
        .chars()
        .take_while(|character| *character == '#')
        .count();
    if !(1..=6).contains(&level) {
        return None;
    }
    trimmed[level..]
        .chars()
        .next()
        .is_none_or(char::is_whitespace)
        .then_some(level)
}

fn claim_heading(record: &str) -> Option<(usize, Option<String>)> {
    let words = semantic_words(record);
    let owner = capability_mentions(&words)
        .last()
        .map(|(_, _, owner)| owner.clone());
    if let Some(level) = atx_heading_level(record) {
        return Some((level, owner));
    }
    let trimmed = record.trim_start();
    let bold_label = trimmed.starts_with("**")
        && trimmed.find("**").is_some_and(|position| position == 0)
        && trimmed[2..].contains("**");
    (bold_label && owner.is_some()).then_some((3, owner))
}

fn table_cells(line: &str) -> Option<Vec<&str>> {
    let line = table_line(line);
    line.contains('|').then(|| {
        line.trim_start_matches('|')
            .trim_end_matches('|')
            .split('|')
            .map(str::trim)
            .collect()
    })
}

fn valid_markdown_delimiter_cell(cell: &str) -> bool {
    let cell = cell.trim();
    let core = cell.strip_prefix(':').unwrap_or(cell);
    let core = core.strip_suffix(':').unwrap_or(core);
    core.len() >= 3 && core.chars().all(|character| character == '-')
}

fn markdown_table_after_header_is_valid(document: &str, header: &str) -> bool {
    let rendered = markdown_outside_fences(document);
    let lines = rendered.lines().map(table_line).collect::<Vec<_>>();
    let positions = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| **line == header)
        .map(|(position, _)| position)
        .collect::<Vec<_>>();
    if positions.len() != 1 {
        return false;
    }
    let position = positions[0];
    let Some(header_cells) = table_cells(header) else {
        return false;
    };
    let Some(delimiter) = lines.get(position + 1).and_then(|line| table_cells(line)) else {
        return false;
    };
    let valid_delimiter = delimiter.len() == header_cells.len()
        && delimiter
            .iter()
            .all(|cell| valid_markdown_delimiter_cell(cell));
    if !valid_delimiter {
        return false;
    }

    let section_start = lines[..position]
        .iter()
        .rposition(|line| line.starts_with("## "))
        .unwrap_or(0);
    let section_end = lines[position + 1..]
        .iter()
        .position(|line| line.starts_with("## "))
        .map_or(lines.len(), |offset| position + 1 + offset);
    let mut table_end = position + 2;
    let mut data_rows = 0;
    while table_end < section_end {
        let Some(cells) = table_cells(lines[table_end]) else {
            break;
        };
        if cells.len() != header_cells.len() {
            return false;
        }
        data_rows += 1;
        table_end += 1;
    }
    if data_rows == 0 {
        return false;
    }
    !(section_start..section_end).any(|line_index| {
        table_cells(lines[line_index]).is_some() && !(position..table_end).contains(&line_index)
    })
}

fn assert_bounded_acceptance_evidence(
    document_name: &str,
    document: &str,
    capability: &str,
    identities: &[&str],
    require_order: bool,
) {
    let rendered = markdown_outside_fences(document);
    let paragraphs = normalized_markdown_paragraphs(&rendered);
    let matching = paragraphs
        .iter()
        .filter(|paragraph| paragraph.contains(identities[0]))
        .collect::<Vec<_>>();
    assert_eq!(
        matching.len(),
        1,
        "{document_name} must contain exactly one {capability} evidence paragraph"
    );
    let paragraph = matching[0];
    assert!(
        paragraph.len() < 2_000,
        "{document_name} has an unbounded {capability} evidence paragraph"
    );

    let mut positions = Vec::with_capacity(identities.len());
    let mut search_from = 0;
    for identity in identities {
        let count = paragraph.matches(identity).count();
        if matches!(
            capability,
            "CAP-018" | "CAP-019" | "CAP-020" | "CAP-021" | "CAP-023"
        ) {
            assert_eq!(
                count, 1,
                "{document_name} must bind {capability} evidence {identity} exactly once"
            );
        } else {
            assert!(
                count > 0,
                "{document_name} is missing {capability} evidence {identity}"
            );
        }
        let position = if require_order {
            search_from
                + paragraph[search_from..].find(identity).unwrap_or_else(|| {
                    panic!("{document_name} is missing ordered {capability} evidence {identity}")
                })
        } else {
            paragraph
                .find(identity)
                .expect("identity count already proved nonzero")
        };
        positions.push(position);
        if require_order {
            search_from = position + identity.len();
        }
    }
    if require_order {
        assert!(
            positions.windows(2).all(|pair| pair[0] < pair[1]),
            "{document_name} does not preserve ordered {capability} evidence"
        );
    }
    let start = *positions
        .iter()
        .min()
        .expect("nonempty evidence identities");
    let final_index = positions
        .iter()
        .enumerate()
        .max_by_key(|(_, position)| *position)
        .map(|(index, _)| index)
        .expect("nonempty evidence identities");
    let cursor = positions[final_index] + identities[final_index].len();
    let maximum_span = match capability {
        "CAP-019" => 1_200,
        "CAP-020" => 1_600,
        "CAP-021" => 1_900,
        "CAP-023" => 1_400,
        _ => 700,
    };
    assert!(
        cursor - start < maximum_span,
        "{document_name} detaches the {capability} evidence identities"
    );
    let conclusion = paragraph[cursor..].trim_start();
    if matches!(capability, "CAP-019" | "CAP-020" | "CAP-021" | "CAP-023") {
        assert_eq!(
            conclusion, "` all pass.",
            "{document_name} must terminate {capability} evidence with the exact all-pass conclusion"
        );
    } else {
        assert!(
            [
                "` pass.",
                "` pass,",
                "` also pass.",
                "` also pass,",
                "` all pass.",
                "` all pass,",
            ]
            .iter()
            .any(|prefix| conclusion.starts_with(prefix)),
            "{document_name} does not bind a passing {capability} conclusion to its exact evidence: {conclusion:?}"
        );
    }
    if matches!(
        capability,
        "CAP-018" | "CAP-019" | "CAP-020" | "CAP-021" | "CAP-023"
    ) {
        let normalized = paragraph.to_ascii_lowercase();
        for contradiction in ["fail", "pending", "not pass", "did not pass"] {
            assert!(
                !normalized.contains(contradiction),
                "{document_name} gives contradictory {capability} evidence: {contradiction}"
            );
        }
    }
}

fn assert_post_cap020_ranking_table(document_name: &str, document: &str) {
    let rendered = markdown_outside_fences(document);
    let semantic = semantic_words(&rendered);
    for stale_label in [
        &[
            "flat", "buffer", "exact", "i32", "2d", "matrix", "vector", "cpu", "product", "gate",
        ][..],
        &[
            "recursive",
            "exact",
            "i32",
            "array",
            "2d",
            "matrix",
            "readiness",
            "and",
            "red",
            "probe",
            "under",
            "one",
            "shared",
            "recursive",
            "shape",
            "authority",
        ][..],
        &[
            "runtime",
            "byte",
            "file",
            "acquisition",
            "into",
            "a",
            "bounded",
            "owned",
            "buffer",
        ][..],
    ] {
        assert!(
            !contains_semantic_phrase(&semantic, stale_label),
            "{document_name} retains a stale post-CAP-019 ranking row or label"
        );
    }
    let source_lines = rendered.lines().map(table_line).collect::<Vec<_>>();
    let rows = rendered
        .lines()
        .enumerate()
        .filter_map(|(source_index, line)| {
            table_cells(line).map(|cells| (source_index, table_line(line), cells))
        })
        .collect::<Vec<_>>();
    let mut indices = Vec::new();
    for expected in POST_CAP020_RANKING_ROWS {
        let expected_cells = table_cells(expected).expect("canonical ranking table row");
        let label = expected_cells[1];
        let matches = rows
            .iter()
            .filter(|(_, _, cells)| {
                cells
                    .get(1)
                    .is_some_and(|actual| actual.eq_ignore_ascii_case(label))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            matches.len(),
            1,
            "{document_name} must contain one unambiguous ranking row for {label}"
        );
        let (source_index, actual, _) = matches[0];
        assert_eq!(
            *actual, expected,
            "{document_name} changes the rank, scores, or total for {label}"
        );
        let cells = table_cells(actual).expect("canonical successor row");
        let scores = cells[2..8]
            .iter()
            .map(|cell| {
                cell.parse::<u8>().unwrap_or_else(|_| {
                    panic!("{document_name} gives {label} a nonnumeric score: {cell}")
                })
            })
            .collect::<Vec<_>>();
        assert!(
            scores.iter().all(|score| (1..=5).contains(score)),
            "{document_name} gives {label} a score outside 1..=5"
        );
        let total = cells[8]
            .parse::<u8>()
            .unwrap_or_else(|_| panic!("{document_name} gives {label} a nonnumeric total"));
        assert_eq!(
            scores.iter().sum::<u8>(),
            total,
            "{document_name} gives {label} an inconsistent total"
        );
        indices.push(*source_index);
    }
    assert!(
        indices.windows(2).all(|pair| pair[1] == pair[0] + 1),
        "{document_name} must preserve one consecutive ordered post-CAP-020 ranking"
    );
    let first = indices[0];
    let last = indices[2];
    assert!(
        first >= 2,
        "{document_name} detaches the ranking from its table header"
    );
    assert_eq!(
        source_lines[first - 2],
        POST_CAP020_RANKING_HEADER,
        "{document_name} changes the canonical post-CAP-020 ranking header"
    );
    assert_eq!(
        source_lines
            .iter()
            .filter(|line| **line == POST_CAP020_RANKING_HEADER)
            .count(),
        1,
        "{document_name} must contain exactly one canonical post-CAP-020 ranking header"
    );
    for (line_index, line) in rendered.lines().map(table_line).enumerate() {
        let Some(cells) = table_cells(line) else {
            continue;
        };
        let is_rank_total_header = cells
            .first()
            .is_some_and(|cell| cell.eq_ignore_ascii_case("rank"))
            && cells
                .last()
                .is_some_and(|cell| cell.eq_ignore_ascii_case("total"));
        if is_rank_total_header && line != POST_CAP020_RANKING_HEADER {
            let legacy_shape = cells.iter().any(|cell| cell == &"Risk")
                && cells.iter().any(|cell| cell == &"Evidence")
                && !cells.iter().any(|cell| cell == &"Favorable risk")
                && !cells.iter().any(|cell| cell == &"Favorable evidence cost")
                && matches!(cells.get(1), Some(&"Gap") | Some(&"Capability gap"));
            let heading = source_lines[..line_index]
                .iter()
                .rfind(|candidate| candidate.starts_with('#'))
                .copied()
                .unwrap_or("");
            let historical_heading = [
                "### ROADMAP-001 ranked gaps and M1-001 outcome",
                "### Post-M1 ranking and accepted CAP-001",
                "### Post-CAP-001 ranking and accepted CAP-002",
                "### Post-CAP-002 ranking and accepted CAP-003",
            ]
            .contains(&heading);
            let exact_audit_history = document_name == "CURRENT_CAPABILITY_AUDIT.md"
                && heading == "### ROADMAP-001 ranking and M1-001 outcome"
                && source_lines
                    .get(line_index..line_index + 5)
                    .is_some_and(|block| {
                        block
                            == [
                                "| Rank | Capability gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Risk | Evidence | Total |",
                                "|---:|---|---:|---:|---:|---:|---:|---:|---:|",
                                "| 1 | Freeze and certify one representative scalar application/subset, including `-O0`/`-O2` equivalence | 4 | 5 | 5 | 5 | 4 | 3 | 26 |",
                                "| 2 | Close the Milestone 0 canonical diagnostic/artifact and trusted-entrypoint contract | 3 | 5 | 5 | 5 | 3 | 3 | 24 |",
                                "| 3 | Implement positive module/import/name resolution after its graph and namespace semantics are frozen | 5 | 3 | 5 | 4 | 2 | 2 | 21 |",
                            ]
                    });
            let legacy_headers_under_heading = source_lines
                .iter()
                .enumerate()
                .filter(|(candidate_index, candidate)| {
                    let candidate_cells = table_cells(candidate).unwrap_or_default();
                    let same_legacy_shape = candidate_cells
                        .first()
                        .is_some_and(|cell| cell.eq_ignore_ascii_case("rank"))
                        && candidate_cells
                            .last()
                            .is_some_and(|cell| cell.eq_ignore_ascii_case("total"))
                        && candidate_cells.iter().any(|cell| cell == &"Risk")
                        && candidate_cells.iter().any(|cell| cell == &"Evidence");
                    if !same_legacy_shape {
                        return false;
                    }
                    source_lines[..*candidate_index]
                        .iter()
                        .rfind(|line| line.starts_with('#'))
                        .is_some_and(|candidate_heading| *candidate_heading == heading)
                })
                .count();
            assert!(
                legacy_shape
                    && (historical_heading || exact_audit_history)
                    && legacy_headers_under_heading == 1,
                "{document_name} adds a competing current ranking table header: {line}"
            );
        }
        let rank = cells.first().and_then(|cell| cell.parse::<u8>().ok());
        if !matches!(rank, Some(1..=3)) {
            continue;
        }
        let words = semantic_words(line);
        let expected = if contains_semantic_phrase(&words, &["tensor", "record"])
            && contains_semantic_phrase(&words, &["two", "stage"])
        {
            Some(POST_CAP020_RANKING_ROWS[0])
        } else if contains_semantic_phrase(&words, &["runtime", "byte", "file", "acquisition"])
            || (words.iter().any(|word| word == "runtime")
                && words.iter().any(|word| word == "acquisition"))
        {
            Some(POST_CAP020_RANKING_ROWS[1])
        } else if contains_semantic_phrase(&words, &["recursive", "exact", "i32", "array"])
            || (words.iter().any(|word| word == "recursive")
                && words.iter().any(|word| word == "matrix"))
        {
            Some(POST_CAP020_RANKING_ROWS[2])
        } else {
            None
        };
        if let Some(expected) = expected {
            assert_eq!(
                line, expected,
                "{document_name} adds a competing or reordered successor row: {line}"
            );
        }
    }
    let header = table_cells(source_lines[first - 2])
        .unwrap_or_else(|| panic!("{document_name} is missing the ranking table header"));
    let canonical_cell_count = table_cells(POST_CAP020_RANKING_ROWS[0])
        .expect("canonical ranking row")
        .len();
    assert_eq!(
        header.len(),
        canonical_cell_count,
        "{document_name} changes the ranking header cardinality"
    );
    assert_eq!(
        header.first().map(|cell| cell.to_ascii_lowercase()),
        Some("rank".to_owned()),
        "{document_name} does not bind the ranking to a Rank table"
    );
    assert_eq!(
        header.last().map(|cell| cell.to_ascii_lowercase()),
        Some("total".to_owned()),
        "{document_name} does not terminate the ranking header with Total"
    );
    let separator = table_cells(source_lines[first - 1])
        .unwrap_or_else(|| panic!("{document_name} is missing the ranking separator"));
    assert_eq!(
        separator.len(),
        canonical_cell_count,
        "{document_name} changes the ranking separator cardinality"
    );
    assert!(
        separator
            .iter()
            .all(|cell| !cell.is_empty() && cell.chars().all(|ch| matches!(ch, '-' | ':'))),
        "{document_name} has a malformed ranking separator"
    );
    assert!(
        source_lines
            .get(last + 1)
            .is_none_or(|line| line.is_empty()),
        "{document_name} appends an uncontracted row to the post-CAP-020 ranking table"
    );
}

fn assert_post_cap021_ranking_table(document_name: &str, document: &str) {
    const SEPARATOR: &str = "|---:|---|---:|---:|---:|---:|---:|---:|---:|";
    let rendered = markdown_outside_fences(document);
    let lines = rendered.lines().map(table_line).collect::<Vec<_>>();
    let exact_blocks = lines
        .iter()
        .enumerate()
        .filter(|(index, line)| {
            **line == POST_CAP021_RANKING_HEADER
                && lines.get(index + 1).is_some_and(|line| *line == SEPARATOR)
                && lines
                    .get(index + 2..index + 5)
                    .is_some_and(|rows| rows == POST_CAP021_RANKING_ROWS)
                && lines.get(index + 5).is_none_or(|line| line.is_empty())
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    assert_eq!(
        exact_blocks.len(),
        1,
        "{document_name} must contain one rendered exact post-CAP-021 ranking table"
    );
    let current_index = exact_blocks[0];
    if document_name == "Roadmap.md" {
        let heading = lines[..current_index]
            .iter()
            .rfind(|line| line.starts_with('#'))
            .copied()
            .unwrap_or("");
        assert_eq!(
            heading, "### Post-CAP-021 ranking",
            "Roadmap.md detaches the current ranking from its exact heading"
        );
    }

    for row in POST_CAP021_RANKING_ROWS {
        let cells = table_cells(row).expect("canonical post-CAP-021 row");
        let scores = cells[2..8]
            .iter()
            .map(|cell| cell.parse::<u8>().expect("numeric canonical score"))
            .collect::<Vec<_>>();
        assert!(scores.iter().all(|score| (1..=5).contains(score)));
        assert_eq!(
            scores.iter().sum::<u8>(),
            cells[8].parse::<u8>().expect("numeric canonical total")
        );
    }

    let canonical_header_count = lines
        .iter()
        .filter(|line| **line == POST_CAP021_RANKING_HEADER)
        .count();
    assert_eq!(
        canonical_header_count,
        if document_name == "Roadmap.md" { 2 } else { 1 },
        "{document_name} changes current/historical favorable-ranking topology"
    );

    let old_tensor_count = lines
        .iter()
        .filter(|line| **line == POST_CAP020_RANKING_ROWS[0])
        .count();
    let old_runtime_count = lines
        .iter()
        .filter(|line| **line == POST_CAP020_RANKING_ROWS[1])
        .count();
    assert_eq!(
        (old_tensor_count, old_runtime_count),
        if document_name == "Roadmap.md" {
            (1, 1)
        } else {
            (0, 0)
        },
        "{document_name} erases Roadmap history or retains the consumed post-CAP-020 table as current"
    );
    for (row_index, row) in POST_CAP021_RANKING_ROWS.iter().enumerate() {
        let expected = if document_name == "Roadmap.md" && row_index == 2 {
            2
        } else {
            1
        };
        assert_eq!(
            lines.iter().filter(|line| **line == *row).count(),
            expected,
            "{document_name} duplicates, omits, or changes post-CAP-021 row {}",
            row_index + 1
        );
    }

    if document_name == "Roadmap.md" {
        let historical_heading = lines
            .iter()
            .position(|line| *line == "### Post-CAP-020 ranking")
            .expect("Roadmap.md must preserve the post-CAP-020 heading");
        let historical_end = lines[historical_heading + 1..]
            .iter()
            .position(|line| line.starts_with("## "))
            .map_or(lines.len(), |offset| historical_heading + 1 + offset);
        let historical_block = (historical_heading + 1..historical_end).any(|index| {
            lines
                .get(index)
                .is_some_and(|line| *line == POST_CAP020_RANKING_HEADER)
                && lines.get(index + 1).is_some_and(|line| *line == SEPARATOR)
                && lines
                    .get(index + 2..index + 5)
                    .is_some_and(|rows| rows == POST_CAP020_RANKING_ROWS)
        });
        assert!(
            historical_block && historical_heading < current_index,
            "Roadmap.md must preserve the exact post-CAP-020 ranking before the current section"
        );
    }

    for (line_index, line) in lines.iter().enumerate() {
        let Some(cells) = table_cells(line) else {
            continue;
        };
        let rank_total = cells
            .first()
            .is_some_and(|cell| cell.eq_ignore_ascii_case("rank"))
            && cells
                .last()
                .is_some_and(|cell| cell.eq_ignore_ascii_case("total"));
        if !rank_total || *line == POST_CAP021_RANKING_HEADER {
            continue;
        }
        let heading = lines[..line_index]
            .iter()
            .rfind(|candidate| candidate.starts_with('#'))
            .copied()
            .unwrap_or("");
        let historical_heading = [
            "### ROADMAP-001 ranked gaps and M1-001 outcome",
            "### Post-M1 ranking and accepted CAP-001",
            "### Post-CAP-001 ranking and accepted CAP-002",
            "### Post-CAP-002 ranking and accepted CAP-003",
        ]
        .contains(&heading)
            || (document_name == "CURRENT_CAPABILITY_AUDIT.md"
                && heading == "### ROADMAP-001 ranking and M1-001 outcome");
        let legacy_shape = cells.iter().any(|cell| cell == &"Risk")
            && cells.iter().any(|cell| cell == &"Evidence")
            && !cells.iter().any(|cell| cell == &"Favorable risk")
            && !cells.iter().any(|cell| cell == &"Favorable evidence cost");
        assert!(
            historical_heading && legacy_shape,
            "{document_name} adds a competing current ranking header: {line}"
        );
    }
}

fn assert_post_cap023_ranking_table(document_name: &str, document: &str) {
    const SEPARATOR: &str = "|---:|---|---:|---:|---:|---:|---:|---:|---:|";
    let rendered = markdown_outside_fences(document);
    let lines = rendered.lines().map(table_line).collect::<Vec<_>>();
    let exact_blocks = lines
        .iter()
        .enumerate()
        .filter(|(index, line)| {
            **line == POST_CAP023_RANKING_HEADER
                && lines.get(index + 1).is_some_and(|line| *line == SEPARATOR)
                && lines
                    .get(index + 2..index + 5)
                    .is_some_and(|rows| rows == POST_CAP023_RANKING_ROWS)
                && lines.get(index + 5).is_none_or(|line| line.is_empty())
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    assert_eq!(
        exact_blocks.len(),
        1,
        "{document_name} must contain one rendered exact post-CAP-023 ranking table"
    );
    let current_index = exact_blocks[0];
    if document_name == "Roadmap.md" {
        let heading = lines[..current_index]
            .iter()
            .rfind(|line| line.starts_with('#'))
            .copied()
            .unwrap_or("");
        assert_eq!(
            heading, "### Post-CAP-023 ranking",
            "Roadmap.md detaches the current ranking from its exact heading"
        );
    }

    for row in POST_CAP023_RANKING_ROWS {
        let cells = table_cells(row).expect("canonical post-CAP-023 row");
        let scores = cells[2..8]
            .iter()
            .map(|cell| cell.parse::<u8>().expect("numeric canonical score"))
            .collect::<Vec<_>>();
        assert!(scores.iter().all(|score| (1..=5).contains(score)));
        assert_eq!(
            scores.iter().sum::<u8>(),
            cells[8].parse::<u8>().expect("numeric canonical total")
        );
        assert_eq!(
            lines.iter().filter(|line| **line == row).count(),
            1,
            "{document_name} duplicates, omits, or changes current post-CAP-023 row {row}"
        );
    }

    assert_eq!(
        lines
            .iter()
            .filter(|line| **line == POST_CAP023_RANKING_HEADER)
            .count(),
        if document_name == "Roadmap.md" { 3 } else { 1 },
        "{document_name} changes current/historical favorable-ranking topology"
    );
    for row in POST_CAP020_RANKING_ROWS {
        let expected = if document_name == "Roadmap.md" { 1 } else { 0 };
        let shared_recursive_row = row == POST_CAP020_RANKING_ROWS[2];
        assert_eq!(
            lines.iter().filter(|line| **line == row).count(),
            expected + usize::from(document_name == "Roadmap.md" && shared_recursive_row),
            "{document_name} erases Roadmap history or retains a consumed post-CAP-020 row as current: {row}"
        );
    }
    for row in POST_CAP021_RANKING_ROWS {
        let expected = if document_name == "Roadmap.md" { 1 } else { 0 };
        let shared_recursive_row = row == POST_CAP021_RANKING_ROWS[2];
        assert_eq!(
            lines.iter().filter(|line| **line == row).count(),
            expected + usize::from(document_name == "Roadmap.md" && shared_recursive_row),
            "{document_name} erases Roadmap history or retains a consumed post-CAP-021 row as current: {row}"
        );
    }

    for (line_index, line) in lines.iter().enumerate() {
        let Some(cells) = table_cells(line) else {
            continue;
        };
        let rank_total = cells
            .first()
            .is_some_and(|cell| cell.eq_ignore_ascii_case("rank"))
            && cells
                .last()
                .is_some_and(|cell| cell.eq_ignore_ascii_case("total"));
        if !rank_total || *line == POST_CAP023_RANKING_HEADER {
            continue;
        }
        let heading = lines[..line_index]
            .iter()
            .rfind(|candidate| candidate.starts_with('#'))
            .copied()
            .unwrap_or("");
        let legacy_heading = [
            "### ROADMAP-001 ranked gaps and M1-001 outcome",
            "### Post-M1 ranking and accepted CAP-001",
            "### Post-CAP-001 ranking and accepted CAP-002",
            "### Post-CAP-002 ranking and accepted CAP-003",
        ]
        .contains(&heading)
            || (document_name == "CURRENT_CAPABILITY_AUDIT.md"
                && heading == "### ROADMAP-001 ranking and M1-001 outcome");
        let legacy_shape = cells.iter().any(|cell| cell == &"Risk")
            && cells.iter().any(|cell| cell == &"Evidence")
            && !cells.iter().any(|cell| cell == &"Favorable risk")
            && !cells.iter().any(|cell| cell == &"Favorable evidence cost");
        assert!(
            legacy_heading && legacy_shape,
            "{document_name} adds a competing current ranking header: {line}"
        );
    }
    assert!(
        lines
            .get(current_index + 5)
            .is_none_or(|line| line.is_empty()),
        "{document_name} appends an uncontracted row to the post-CAP-023 ranking table"
    );
}

fn assert_post_cap024_ranking_table(document_name: &str, document: &str) {
    const SEPARATOR: &str = "|---:|---|---:|---:|---:|---:|---:|---:|---:|";
    let rendered = markdown_outside_fences(document);
    let lines = rendered.lines().map(table_line).collect::<Vec<_>>();
    let exact_blocks = lines
        .iter()
        .enumerate()
        .filter(|(index, line)| {
            **line == POST_CAP024_RANKING_HEADER
                && lines.get(index + 1).is_some_and(|line| *line == SEPARATOR)
                && lines
                    .get(index + 2..index + 5)
                    .is_some_and(|rows| rows == POST_CAP024_RANKING_ROWS)
                && lines.get(index + 5).is_none_or(|line| line.is_empty())
        })
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    assert_eq!(
        exact_blocks.len(),
        1,
        "{document_name} must contain one rendered exact post-CAP-024 ranking table"
    );
    let current_index = exact_blocks[0];
    if document_name == "Roadmap.md" {
        let heading = lines[..current_index]
            .iter()
            .rfind(|line| line.starts_with('#'))
            .copied()
            .unwrap_or("");
        assert_eq!(
            heading, "### Post-CAP-024 ranking",
            "Roadmap.md detaches the current ranking from its exact heading"
        );
    }

    for row in POST_CAP024_RANKING_ROWS {
        let cells = table_cells(row).expect("canonical post-CAP-024 row");
        let scores = cells[2..8]
            .iter()
            .map(|cell| cell.parse::<u8>().expect("numeric canonical score"))
            .collect::<Vec<_>>();
        assert!(scores.iter().all(|score| (1..=5).contains(score)));
        assert_eq!(
            scores.iter().sum::<u8>(),
            cells[8].parse::<u8>().expect("numeric canonical total")
        );
        let historical_count = if document_name == "Roadmap.md" {
            POST_CAP020_RANKING_ROWS
                .iter()
                .chain(&POST_CAP021_RANKING_ROWS)
                .chain(&POST_CAP023_RANKING_ROWS)
                .filter(|historical| **historical == row)
                .count()
        } else {
            0
        };
        assert_eq!(
            lines.iter().filter(|line| **line == row).count(),
            historical_count + 1,
            "{document_name} duplicates, omits, or changes current post-CAP-024 row {row}"
        );
    }

    assert_eq!(
        lines
            .iter()
            .filter(|line| **line == POST_CAP024_RANKING_HEADER)
            .count(),
        if document_name == "Roadmap.md" { 4 } else { 1 },
        "{document_name} changes current/historical favorable-ranking topology"
    );
    for row in POST_CAP020_RANKING_ROWS
        .iter()
        .chain(&POST_CAP021_RANKING_ROWS)
        .chain(&POST_CAP023_RANKING_ROWS)
    {
        let historical_count = if document_name == "Roadmap.md" {
            POST_CAP020_RANKING_ROWS
                .iter()
                .chain(&POST_CAP021_RANKING_ROWS)
                .chain(&POST_CAP023_RANKING_ROWS)
                .filter(|historical| *historical == row)
                .count()
        } else {
            0
        };
        let current_count = usize::from(POST_CAP024_RANKING_ROWS.contains(row));
        assert_eq!(
            lines.iter().filter(|line| *line == row).count(),
            historical_count + current_count,
            "{document_name} erases Roadmap history or retains a consumed ranking row as current: {row}"
        );
    }

    for (line_index, line) in lines.iter().enumerate() {
        let Some(cells) = table_cells(line) else {
            continue;
        };
        let rank_total = cells
            .first()
            .is_some_and(|cell| cell.eq_ignore_ascii_case("rank"))
            && cells
                .last()
                .is_some_and(|cell| cell.eq_ignore_ascii_case("total"));
        if !rank_total || *line == POST_CAP024_RANKING_HEADER {
            continue;
        }
        let heading = lines[..line_index]
            .iter()
            .rfind(|candidate| candidate.starts_with('#'))
            .copied()
            .unwrap_or("");
        let legacy_heading = [
            "### ROADMAP-001 ranked gaps and M1-001 outcome",
            "### Post-M1 ranking and accepted CAP-001",
            "### Post-CAP-001 ranking and accepted CAP-002",
            "### Post-CAP-002 ranking and accepted CAP-003",
        ]
        .contains(&heading)
            || (document_name == "CURRENT_CAPABILITY_AUDIT.md"
                && heading == "### ROADMAP-001 ranking and M1-001 outcome");
        let legacy_shape = cells.iter().any(|cell| cell == &"Risk")
            && cells.iter().any(|cell| cell == &"Evidence")
            && !cells.iter().any(|cell| cell == &"Favorable risk")
            && !cells.iter().any(|cell| cell == &"Favorable evidence cost");
        assert!(
            legacy_heading && legacy_shape,
            "{document_name} adds a competing current ranking header: {line}"
        );
    }
    assert!(
        lines
            .get(current_index + 5)
            .is_none_or(|line| line.is_empty()),
        "{document_name} appends an uncontracted row to the post-CAP-024 ranking table"
    );
}

fn ordered_decision_records_violation(section: &str, contracts: &[&str]) -> Option<String> {
    let rendered = markdown_outside_fences(section);
    let records = normalized_claim_records_from_rendered(&rendered)
        .into_iter()
        .map(|record| normalized_words(&record))
        .collect::<Vec<_>>();
    let mut previous = None;
    for contract in contracts {
        let positions = records
            .iter()
            .enumerate()
            .filter(|(_, record)| record.as_str() == *contract)
            .map(|(position, _)| position)
            .collect::<Vec<_>>();
        if positions.len() != 1 {
            return Some(format!(
                "decision record occurs {} times instead of once: {contract}",
                positions.len()
            ));
        }
        if previous.is_some_and(|previous| previous >= positions[0]) {
            return Some(format!("decision record is out of order: {contract}"));
        }
        previous = Some(positions[0]);
    }
    None
}

fn assert_exact_ordered_decision_records(document_name: &str, section: &str, contracts: &[&str]) {
    assert!(
        ordered_decision_records_violation(section, contracts).is_none(),
        "{document_name} must preserve each exact decision as one rendered ordered record: {:?}",
        ordered_decision_records_violation(section, contracts)
    );
}

fn cap023_matrix_violation(matrix: &str) -> Option<String> {
    let rendered = markdown_outside_fences(matrix);
    let all_cap023_rows = rendered
        .lines()
        .map(table_line)
        .filter(|line| {
            table_cells(line).is_some() && has_semantic_capability(&semantic_words(line), "023")
        })
        .collect::<Vec<_>>();
    if all_cap023_rows != [CAP023_CPU_MATRIX_ROW] {
        return Some(format!(
            "CAP-023 appears in an uncontracted matrix table row: {all_cap023_rows:?}"
        ));
    }
    let Some(classified_tables) = rendered
        .split_once("## Language features")
        .and_then(|(_, tail)| tail.split_once("## Evidence notes"))
        .map(|(section, _)| section)
    else {
        return Some("missing bounded classified-table region".to_owned());
    };
    let Some(language_features) = rendered
        .split_once("## Language features")
        .and_then(|(_, tail)| tail.split_once("## Compiler, tooling, and ecosystem surfaces"))
        .map(|(section, _)| section)
    else {
        return Some("missing bounded language-feature section".to_owned());
    };
    if language_features.lines().map(table_line).any(|line| {
        table_cells(line).is_some() && has_semantic_capability(&semantic_words(line), "023")
    }) {
        return Some("CAP-023 appears in a language-feature/profile row".to_owned());
    }
    let cap023_rows = classified_tables
        .lines()
        .map(table_line)
        .filter(|line| {
            table_cells(line).is_some() && has_semantic_capability(&semantic_words(line), "023")
        })
        .collect::<Vec<_>>();
    if cap023_rows != [CAP023_CPU_MATRIX_ROW] {
        return Some(format!(
            "CAP-023 matrix rows are not the sole exact CPU PARTIAL row: {cap023_rows:?}"
        ));
    }
    let Some(backend) = classified_tables
        .split_once("## Backend summary")
        .map(|(_, section)| section)
    else {
        return Some("missing bounded backend-summary section".to_owned());
    };
    let cpu_rows = backend
        .lines()
        .map(table_line)
        .filter(|line| {
            table_cells(line).is_some_and(|cells| {
                cells
                    .first()
                    .is_some_and(|label| label.eq_ignore_ascii_case("cpu"))
            })
        })
        .collect::<Vec<_>>();
    if cpu_rows != [CAP023_CPU_MATRIX_ROW] {
        return Some(format!(
            "backend summary does not retain one exact CPU CAP-023 PARTIAL row: {cpu_rows:?}"
        ));
    }
    None
}

fn cap024_matrix_violation(matrix: &str) -> Option<String> {
    let rendered = markdown_outside_fences(matrix);
    let Some(classified_tables) = rendered
        .split_once("## Language features")
        .and_then(|(_, tail)| tail.split_once("## Evidence notes"))
        .map(|(section, _)| section)
    else {
        return Some("missing bounded classified-table region".to_owned());
    };
    let classified_cap024_rows = classified_tables
        .lines()
        .map(table_line)
        .filter(|line| {
            table_cells(line).is_some() && has_semantic_capability(&semantic_words(line), "024")
        })
        .collect::<Vec<_>>();
    if !classified_cap024_rows.is_empty() {
        return Some(format!(
            "CAP-024 appears in a classified feature/profile/backend row: {classified_cap024_rows:?}"
        ));
    }
    let Some(language_features) = classified_tables
        .split_once("## Compiler, tooling, and ecosystem surfaces")
        .map(|(section, _)| section)
    else {
        return Some("missing bounded language-feature section".to_owned());
    };
    if language_features.lines().map(table_line).any(|line| {
        table_cells(line).is_some() && has_semantic_capability(&semantic_words(line), "024")
    }) {
        return Some("CAP-024 appears in a language-feature or selected-profile row".to_owned());
    }
    let Some(backend) = classified_tables
        .split_once("## Backend summary")
        .map(|(_, section)| section)
    else {
        return Some("missing bounded backend-summary section".to_owned());
    };
    let cpu_rows = backend
        .lines()
        .map(table_line)
        .filter(|line| {
            table_cells(line).is_some_and(|cells| {
                cells
                    .first()
                    .is_some_and(|label| label.eq_ignore_ascii_case("cpu"))
            })
        })
        .collect::<Vec<_>>();
    if cpu_rows != [CAP023_CPU_MATRIX_ROW] {
        return Some(format!(
            "backend summary does not retain one byte-identical CAP-023 CPU PARTIAL row: {cpu_rows:?}"
        ));
    }
    None
}

fn frozen_cap024_matrix_source_row_violation(matrix_source: &str) -> Option<String> {
    let Some(language_features) = matrix_source
        .split_once("## Language features")
        .and_then(|(_, tail)| tail.split_once("## Compiler, tooling, and ecosystem surfaces"))
        .map(|(section, _)| section)
    else {
        return Some("missing raw language-feature source region".to_owned());
    };
    let selected_profile_rows = language_features
        .lines()
        .filter(|line| line.contains("exact-i32-array-v0") || line.contains("CAP-018"))
        .collect::<Vec<_>>();
    if selected_profile_rows != [CAP019_SELECTED_PROFILE_MATRIX_ROW] {
        return Some(format!(
            "selected-profile source row is not the sole byte-identical frozen row: {selected_profile_rows:?}"
        ));
    }

    let Some(backend_summary) = matrix_source
        .split_once("## Backend summary")
        .and_then(|(_, tail)| tail.split_once("## Evidence notes"))
        .map(|(section, _)| section)
    else {
        return Some("missing raw backend-summary source region".to_owned());
    };
    let cpu_rows = backend_summary
        .lines()
        .filter(|line| {
            line.contains("| CPU |")
                || line.contains("&#124; CPU &#124;")
                || line.contains("&vert; CPU &vert;")
        })
        .collect::<Vec<_>>();
    if cpu_rows != [CAP023_CPU_MATRIX_ROW] {
        return Some(format!(
            "CPU source row is not the sole byte-identical frozen row: {cpu_rows:?}"
        ));
    }
    None
}

fn assert_cap014_acceptance_evidence(document_name: &str, document: &str) {
    assert_bounded_acceptance_evidence(
        document_name,
        document,
        "CAP-014",
        &CAP014_ACCEPTANCE_EVIDENCE,
        false,
    );
}

fn assert_cap015_acceptance_evidence(document_name: &str, document: &str) {
    assert_bounded_acceptance_evidence(
        document_name,
        document,
        "CAP-015",
        &CAP015_ACCEPTANCE_EVIDENCE,
        true,
    );
}

fn assert_cap018_acceptance_evidence(document_name: &str, document: &str) {
    assert_bounded_acceptance_evidence(
        document_name,
        document,
        "CAP-018",
        &CAP018_ACCEPTANCE_EVIDENCE,
        true,
    );
}

fn assert_cap019_acceptance_evidence(document_name: &str, document: &str) {
    let rendered = markdown_outside_fences(document);
    let normalized = normalized_words(&rendered);
    assert_eq!(
        normalized
            .matches("Exact CAP-019 reviewed candidate")
            .count(),
        1,
        "{document_name} must contain exactly one CAP-019 evidence lead-in"
    );
    assert_eq!(
        normalized.matches(CAP019_EVIDENCE_PREFIX).count(),
        1,
        "{document_name} is missing or duplicates the canonical CAP-019 evidence prefix"
    );
    for identity in &CAP019_ACCEPTANCE_EVIDENCE[4..] {
        assert_eq!(
            normalized.matches(identity).count(),
            1,
            "{document_name} must state CAP-019 run/job/analysis {identity} exactly once"
        );
    }
    let evidence_paragraphs = normalized_markdown_paragraphs(&rendered)
        .into_iter()
        .filter(|paragraph| paragraph.contains(CAP019_ACCEPTANCE_EVIDENCE[0]))
        .collect::<Vec<_>>();
    assert_eq!(
        evidence_paragraphs.len(),
        1,
        "{document_name} must contain one SHA-scoped CAP-019 evidence paragraph"
    );
    assert!(
        evidence_paragraphs[0].starts_with(CAP019_EVIDENCE_PREFIX),
        "{document_name} detaches the CAP-019 evidence lead-in from its SHA-scoped paragraph"
    );
    assert_bounded_acceptance_evidence(
        document_name,
        document,
        "CAP-019",
        &CAP019_ACCEPTANCE_EVIDENCE,
        true,
    );
}

fn assert_cap020_acceptance_evidence(document_name: &str, document: &str) {
    let rendered = markdown_outside_fences(document);
    let normalized = normalized_words(&rendered);
    let rendered_normalized = normalized_words(&rendered);
    assert_eq!(
        rendered_normalized
            .matches("Exact CAP-020 reviewed candidate")
            .count(),
        1,
        "{document_name} must contain exactly one CAP-020 evidence lead-in"
    );
    assert_eq!(
        rendered_normalized.matches(CAP020_EVIDENCE_PREFIX).count(),
        1,
        "{document_name} is missing or duplicates the canonical CAP-020 evidence prefix"
    );
    for identity in CAP020_ACCEPTANCE_EVIDENCE {
        assert_eq!(
            normalized.matches(identity).count(),
            1,
            "{document_name} must state CAP-020 evidence identity {identity} exactly once globally"
        );
    }
    let evidence_paragraphs = normalized_markdown_paragraphs(&rendered)
        .into_iter()
        .filter(|paragraph| paragraph.contains(CAP020_ACCEPTANCE_EVIDENCE[0]))
        .collect::<Vec<_>>();
    assert_eq!(
        evidence_paragraphs.len(),
        1,
        "{document_name} must contain one SHA-scoped CAP-020 evidence paragraph"
    );
    assert!(
        evidence_paragraphs[0].starts_with(CAP020_EVIDENCE_PREFIX),
        "{document_name} detaches the CAP-020 evidence lead-in from its SHA-scoped paragraph"
    );
    assert_eq!(
        evidence_paragraphs[0], CAP020_EVIDENCE_PARAGRAPH,
        "{document_name} changes CAP-020 evidence labels, roles, order, or conclusion"
    );
    assert_bounded_acceptance_evidence(
        document_name,
        document,
        "CAP-020",
        &CAP020_ACCEPTANCE_EVIDENCE,
        true,
    );
}

fn assert_cap021_acceptance_evidence(document_name: &str, document: &str) {
    let rendered = markdown_outside_fences(document);
    let normalized = normalized_words(&rendered);
    assert_eq!(
        normalized
            .matches("Exact CAP-021 reviewed candidate")
            .count(),
        1,
        "{document_name} must contain exactly one CAP-021 evidence lead-in"
    );
    assert_eq!(
        normalized.matches(CAP021_EVIDENCE_PREFIX).count(),
        1,
        "{document_name} is missing or duplicates the canonical CAP-021 evidence prefix"
    );
    for identity in CAP021_ACCEPTANCE_EVIDENCE {
        assert_eq!(
            normalized.matches(identity).count(),
            1,
            "{document_name} must state CAP-021 evidence identity {identity} exactly once globally"
        );
    }
    let evidence_paragraphs = normalized_markdown_paragraphs(&rendered)
        .into_iter()
        .filter(|paragraph| paragraph.contains(CAP021_ACCEPTANCE_EVIDENCE[0]))
        .collect::<Vec<_>>();
    assert_eq!(
        evidence_paragraphs.len(),
        1,
        "{document_name} must contain one SHA-scoped CAP-021 evidence paragraph"
    );
    assert_eq!(
        evidence_paragraphs[0], CAP021_EVIDENCE_PARAGRAPH,
        "{document_name} changes CAP-021 evidence labels, roles, order, or conclusion"
    );
    assert_bounded_acceptance_evidence(
        document_name,
        document,
        "CAP-021",
        &CAP021_ACCEPTANCE_EVIDENCE,
        true,
    );
}

fn cap023_evidence_violation(document: &str) -> Option<String> {
    let rendered = markdown_outside_fences(document);
    let normalized = normalized_words(&rendered);
    if normalized
        .matches("Exact CAP-023 reviewed candidate")
        .count()
        != 1
    {
        return Some("CAP-023 evidence lead-in must occur exactly once".to_owned());
    }
    if normalized.matches(CAP023_EVIDENCE_PREFIX).count() != 1 {
        return Some("CAP-023 evidence prefix is missing or duplicated".to_owned());
    }
    let has_alert_boundary = normalized.contains(CAP023_ALERT_BOUNDARY);
    for (position, identity) in CAP023_ACCEPTANCE_EVIDENCE.iter().enumerate() {
        let expected =
            usize::from(has_alert_boundary && (position == 3 || *identity == "1612715455")) + 1;
        let count = normalized.matches(identity).count();
        if count != expected {
            return Some(format!(
                "CAP-023 evidence identity {identity} occurs {count} times instead of {expected}"
            ));
        }
    }
    let evidence_paragraphs = normalized_markdown_paragraphs(&rendered)
        .into_iter()
        .filter(|paragraph| paragraph.contains(CAP023_ACCEPTANCE_EVIDENCE[0]))
        .collect::<Vec<_>>();
    if evidence_paragraphs.len() != 1 {
        return Some(format!(
            "CAP-023 SHA-scoped evidence paragraph count is {}",
            evidence_paragraphs.len()
        ));
    }
    let paragraph = &evidence_paragraphs[0];
    if paragraph.len() >= 2_000 {
        return Some("CAP-023 evidence paragraph exceeds its bounded size".to_owned());
    }
    if paragraph != CAP023_EVIDENCE_PARAGRAPH {
        return Some(
            "CAP-023 evidence labels, roles, order, punctuation, or terminal all-pass conclusion changed"
                .to_owned(),
        );
    }
    let roles = [
        ("candidate", CAP023_ACCEPTANCE_EVIDENCE[0]),
        ("tree", CAP023_ACCEPTANCE_EVIDENCE[1]),
        ("base", CAP023_ACCEPTANCE_EVIDENCE[2]),
        ("merge", CAP023_ACCEPTANCE_EVIDENCE[3]),
    ];
    let mut section_owner: Option<(usize, String)> = None;
    for record in normalized_claim_records_from_rendered(&rendered) {
        if record == CAP023_EVIDENCE_PARAGRAPH {
            continue;
        }
        if let Some((level, owner)) = claim_heading(&record) {
            if let Some(owner) = owner {
                section_owner = Some((level, owner));
            } else if section_owner
                .as_ref()
                .is_some_and(|(owned_level, _)| level <= *owned_level)
            {
                section_owner = None;
            }
        }
        let words = semantic_words(&record);
        if !has_semantic_capability(&words, "023")
            && section_owner.as_ref().map(|(_, owner)| owner.as_str()) != Some("023")
        {
            continue;
        }
        for (position, word) in words.iter().enumerate() {
            let Some((_, expected)) = roles.iter().find(|(role, _)| word == role) else {
                continue;
            };
            if matches!(word.as_str(), "candidate" | "merge")
                && words
                    .get(position + 1)
                    .is_some_and(|next| matches!(next.as_str(), "tree" | "parent"))
            {
                continue;
            }
            let mut identity_position = position + 1;
            while words.get(identity_position).is_some_and(|next| {
                matches!(next.as_str(), "commit" | "sha" | "hash" | "is" | "equals")
            }) {
                identity_position += 1;
            }
            let Some(identity) = words.get(identity_position) else {
                continue;
            };
            let identity_shaped = identity.len() >= 7
                && identity
                    .chars()
                    .all(|character| character.is_ascii_hexdigit());
            if identity_shaped && identity != expected {
                return Some(format!(
                    "CAP-023 evidence role {word} is assigned to the wrong identity {identity}"
                ));
            }
        }
    }
    None
}

fn assert_cap023_acceptance_evidence(document_name: &str, document: &str) {
    assert!(
        cap023_evidence_violation(document).is_none(),
        "{document_name} violates the canonical CAP-023 evidence contract: {:?}",
        cap023_evidence_violation(document)
    );
    assert_bounded_acceptance_evidence(
        document_name,
        document,
        "CAP-023",
        &CAP023_ACCEPTANCE_EVIDENCE,
        true,
    );
}

fn evidence_identity_shaped(word: &str) -> bool {
    word.len() >= 7 && word != "256" && word.chars().all(|character| character.is_ascii_hexdigit())
}

fn cap024_role_identity_violation(words: &[String]) -> Option<String> {
    const ROLE_SPECS: &[(&[&str], &[usize])] = &[
        (&["candidate", "push", "ci"], &[4]),
        (&["pr", "ci"], &[5]),
        (&["rust", "ci"], &[6, 29]),
        (&["cap", "024", "evidence", "run"], &[8]),
        (&["evidence", "run"], &[8]),
        (&["candidate", "push", "compiler", "jobs"], &[9]),
        (&["candidate", "push", "compiler", "job"], &[9]),
        (&["pr", "compiler", "jobs"], &[10]),
        (&["pr", "compiler", "job"], &[10]),
        (&["aggregate", "candidate", "codeql", "check"], &[17]),
        (&["codeql", "actions", "jobs"], &[14, 36]),
        (&["codeql", "actions", "job"], &[14, 36]),
        (&["actions", "analyses"], &[18, 39]),
        (&["actions", "analysis"], &[18, 39]),
        (&["python", "analyses"], &[19, 40]),
        (&["python", "analysis"], &[19, 40]),
        (&["rust", "analyses"], &[20, 41]),
        (&["rust", "analysis"], &[20, 41]),
        (&["linux", "jobs"], &[21, 43]),
        (&["linux", "job"], &[21, 43]),
        (&["windows", "jobs"], &[13, 22, 35, 44]),
        (&["windows", "job"], &[13, 22, 35, 44]),
        (&["aggregate", "replay", "job"], &[42]),
        (&["aggregate", "jobs"], &[23, 42]),
        (&["aggregate", "job"], &[23, 42]),
        (&["capture", "jobs"], &[43, 44]),
        (&["capture", "job"], &[43, 44]),
        (&["merge", "head", "ci"], &[28]),
        (&["merge", "compiler", "job"], &[32]),
        (&["replay", "run"], &[31]),
        (&["canonical", "manifest"], &[27]),
        (&["fresh", "manifest"], &[25]),
        (&["fresh", "observations"], &[26]),
        (&["fresh", "observation"], &[26]),
        (&["artifact"], &[24]),
        (&["stable", "job"], &[11, 33]),
        (&["nightly", "job"], &[12, 34]),
        (&["python", "job"], &[15, 37]),
        (&["rust", "job"], &[16, 38]),
        (&["codeql"], &[7, 30]),
        (&["candidate"], &[0]),
        (&["tree"], &[1]),
        (&["base"], &[2]),
        (&["merge"], &[3]),
    ];

    for (descriptor, expected_indices) in ROLE_SPECS {
        for position in words
            .windows(descriptor.len())
            .enumerate()
            .filter(|(_, candidate)| {
                candidate
                    .iter()
                    .map(String::as_str)
                    .eq(descriptor.iter().copied())
            })
            .map(|(position, _)| position)
        {
            let next = words.get(position + descriptor.len()).map(String::as_str);
            if descriptor.len() == 1
                && descriptor[0] == "candidate"
                && next.is_some_and(|word| matches!(word, "push" | "tree" | "check" | "codeql"))
            {
                continue;
            }
            if descriptor.len() == 1
                && descriptor[0] == "merge"
                && next.is_some_and(|word| {
                    matches!(word, "tree" | "parent" | "push" | "head" | "compiler")
                })
            {
                continue;
            }
            if descriptor.len() == 1
                && descriptor[0] == "codeql"
                && (next.is_some_and(|word| word == "actions")
                    || words[position.saturating_sub(3)..position]
                        .iter()
                        .any(|word| word == "aggregate"))
            {
                continue;
            }
            let identity = words[position + descriptor.len()..]
                .iter()
                .find(|identity| evidence_identity_shaped(identity));
            let Some(identity) = identity else {
                continue;
            };
            if !expected_indices
                .iter()
                .any(|index| identity.as_str() == CAP024_ACCEPTANCE_EVIDENCE[*index])
            {
                return Some(format!(
                    "CAP-024 evidence role {} is assigned to the wrong identity {identity}",
                    descriptor.join(" ")
                ));
            }
        }
    }

    for (position, identity) in words.iter().enumerate() {
        if !evidence_identity_shaped(identity)
            || CAP024_ACCEPTANCE_EVIDENCE.contains(&identity.as_str())
        {
            continue;
        }
        let role_prefix = &words[position.saturating_sub(8)..position];
        if role_prefix.iter().any(|word| {
            matches!(
                word.as_str(),
                "run" | "runs" | "job" | "jobs" | "analysis" | "analyses" | "ci" | "check"
            )
        }) {
            return Some(format!(
                "CAP-024 assigns an unknown run/job/analysis identity {identity}"
            ));
        }
    }
    None
}

fn cap024_evidence_violation(document: &str) -> Option<String> {
    let rendered = markdown_outside_fences(document);
    let normalized = normalized_words(&rendered);
    if normalized
        .matches("Exact CAP-024 reviewed candidate")
        .count()
        != 1
    {
        return Some("CAP-024 evidence lead-in must occur exactly once".to_owned());
    }
    if normalized.matches(CAP024_EVIDENCE_PREFIX).count() != 1 {
        return Some("CAP-024 evidence prefix is missing or duplicated".to_owned());
    }
    for (position, identity) in CAP024_ACCEPTANCE_EVIDENCE.iter().enumerate() {
        let expected = 1 + usize::from(matches!(position, 0..=3 | 27 | 39));
        let count = normalized.matches(identity).count();
        if count != expected {
            return Some(format!(
                "CAP-024 evidence identity {identity} occurs {count} times instead of {expected}"
            ));
        }
    }
    let evidence_paragraphs = normalized_markdown_paragraphs(&rendered)
        .into_iter()
        .filter(|paragraph| paragraph.starts_with(CAP024_EVIDENCE_PREFIX))
        .collect::<Vec<_>>();
    if evidence_paragraphs.len() != 1 {
        return Some(format!(
            "CAP-024 canonical evidence paragraph count is {}",
            evidence_paragraphs.len()
        ));
    }
    let paragraph = &evidence_paragraphs[0];
    if paragraph.len() >= 4_000 {
        return Some("CAP-024 evidence paragraph exceeds its bounded size".to_owned());
    }
    if paragraph != CAP024_EVIDENCE_PARAGRAPH {
        return Some(
            "CAP-024 evidence labels, roles, order, punctuation, results, or skip conclusion changed"
                .to_owned(),
        );
    }
    let mut search_from = 0;
    for identity in CAP024_ACCEPTANCE_EVIDENCE {
        let Some(offset) = paragraph[search_from..].find(identity) else {
            return Some(format!(
                "CAP-024 evidence paragraph is missing ordered identity {identity}"
            ));
        };
        search_from += offset + identity.len();
    }

    let mut section_owner: Option<(usize, String)> = None;
    for record in normalized_claim_records_from_rendered(&rendered) {
        if record == CAP024_EVIDENCE_PARAGRAPH {
            continue;
        }
        if let Some((level, owner)) = claim_heading(&record) {
            if let Some(owner) = owner {
                section_owner = Some((level, owner));
            } else if section_owner
                .as_ref()
                .is_some_and(|(owned_level, _)| level <= *owned_level)
            {
                section_owner = None;
            }
        }
        let words = semantic_words(&record);
        if !has_semantic_capability(&words, "024")
            && section_owner.as_ref().map(|(_, owner)| owner.as_str()) != Some("024")
        {
            continue;
        }
        if let Some(violation) = cap024_role_identity_violation(&words) {
            return Some(violation);
        }
    }
    None
}

fn assert_cap024_acceptance_evidence(document_name: &str, document: &str) {
    assert!(
        cap024_evidence_violation(document).is_none(),
        "{document_name} violates the canonical CAP-024 evidence contract: {:?}",
        cap024_evidence_violation(document)
    );
}

fn assert_post_cap020_successor_order(document_name: &str, document: &str) {
    let rendered = markdown_outside_fences(document);
    let normalized = rendered.to_ascii_lowercase();
    let tensor_record = normalized
        .find("source-embedded fixed-shape tensor-record decode plus two-stage flat-buffer exact-`i32` cpu scoring product gate")
        .unwrap_or_else(|| panic!("{document_name} is missing the ranked tensor-record gate"));
    let file_bytes = normalized[tensor_record..]
        .find("runtime byte/file acquisition readiness and red probe")
        .map(|offset| tensor_record + offset)
        .unwrap_or_else(|| panic!("{document_name} is missing the ranked runtime byte/file probe"));
    let recursive_array = normalized[file_bytes..]
        .find("recursive exact-`i32` array / 2d matrix readiness deferred")
        .map(|offset| file_bytes + offset)
        .unwrap_or_else(|| panic!("{document_name} is missing the deferred recursive-array probe"));
    assert!(
        tensor_record < file_bytes && file_bytes < recursive_array,
        "{document_name} does not preserve tensor record -> byte/file probe -> recursive deferral ordering"
    );
    assert!(
        !normalized.contains("cap-018-readiness"),
        "{document_name} invents CAP-018 readiness work after acceptance"
    );
    assert!(
        successor_order_violation(document).is_none(),
        "{document_name} states a contradictory post-CAP-020 successor order: {:?}",
        successor_order_violation(document)
    );
}

fn successor_order_violation(text: &str) -> Option<String> {
    for record in normalized_claim_records(text) {
        for clause in record.split(['.', ';', '!', '?']) {
            let words = semantic_words(clause);
            let mut categories = Vec::new();
            for (position, word) in words.iter().enumerate() {
                let nearby = &words[position..(position + 5).min(words.len())];
                if word == "tensor" && nearby.iter().any(|word| word == "record") {
                    categories.push((position, 1_u8));
                } else if word == "runtime" && nearby.iter().any(|word| word == "acquisition") {
                    categories.push((position, 2_u8));
                } else if word == "byte"
                    && nearby.iter().any(|word| word == "file")
                    && nearby.iter().any(|word| word == "acquisition")
                {
                    categories.push((position, 2_u8));
                } else if word == "recursive"
                    && nearby
                        .iter()
                        .any(|word| matches!(word.as_str(), "array" | "arrays" | "matrix"))
                {
                    categories.push((position, 3_u8));
                }
            }
            let nearest_category = |position: usize| {
                categories
                    .iter()
                    .min_by_key(|(candidate, _)| candidate.abs_diff(position))
                    .copied()
            };
            for (position, word) in words.iter().enumerate() {
                let claimed_rank = match word.as_str() {
                    "first" | "one" | "1" => Some(1),
                    "second" | "two" | "2" => Some(2),
                    "third" | "three" | "3" => Some(3),
                    _ => None,
                };
                if let Some(claimed_rank) = claimed_rank {
                    let has_rank_relation =
                        word.chars().all(|character| character.is_ascii_digit())
                            && words
                                .get(position.wrapping_sub(1))
                                .is_some_and(|word| word == "rank")
                            || words[position.saturating_sub(3)..position]
                                .iter()
                                .any(|word| {
                                    matches!(word.as_str(), "rank" | "ranks" | "ranked" | "placed")
                                })
                            || words
                                .get(position + 1)
                                .is_some_and(|word| word == "priority")
                            || words
                                .get(position.wrapping_sub(1))
                                .is_some_and(|word| word == "priority");
                    let unlike = words[..position].iter().rposition(|word| word == "unlike");
                    let attached_category = unlike
                        .and_then(|unlike| {
                            categories
                                .iter()
                                .filter(|(candidate, _)| *candidate < unlike)
                                .max_by_key(|(candidate, _)| *candidate)
                                .copied()
                        })
                        .or_else(|| nearest_category(position));
                    if has_rank_relation
                        && !preceded_by_local_negation(&words, position)
                        && attached_category.is_some_and(|(_, expected)| expected != claimed_rank)
                    {
                        return Some(clause.trim().to_owned());
                    }
                }
            }
            for (position, relation) in words.iter().enumerate() {
                let direction = match relation.as_str() {
                    "precede" | "precedes" | "before" | "ahead" => Some(true),
                    "follow" | "follows" | "after" | "behind" => Some(false),
                    _ => None,
                };
                let Some(left_before_right) = direction else {
                    continue;
                };
                if preceded_by_local_negation(&words, position) {
                    continue;
                }
                let left = categories
                    .iter()
                    .filter(|(candidate, _)| *candidate < position)
                    .max_by_key(|(candidate, _)| *candidate)
                    .map(|(_, rank)| *rank);
                let right = categories
                    .iter()
                    .filter(|(candidate, _)| *candidate > position)
                    .min_by_key(|(candidate, _)| *candidate)
                    .map(|(_, rank)| *rank);
                if matches!((left, right), (Some(left), Some(right)) if left != right && (left < right) != left_before_right)
                {
                    return Some(clause.trim().to_owned());
                }
            }
        }
    }
    None
}

fn post_cap021_successor_order_violation(text: &str) -> Option<String> {
    for record in normalized_claim_records(text) {
        for clause in record.split(['.', ';', '!', '?']) {
            let words = semantic_words(clause);
            let mut categories = Vec::new();
            for (position, word) in words.iter().enumerate() {
                let nearby = &words[position..(position + 7).min(words.len())];
                if (word == "runtime" && nearby.iter().any(|word| word == "acquisition"))
                    || (word == "byte"
                        && nearby.iter().any(|word| word == "file")
                        && nearby.iter().any(|word| word == "acquisition"))
                {
                    categories.push((position, 1_u8));
                } else if matches!(word.as_str(), "quantized" | "quantization") {
                    categories.push((position, 2_u8));
                } else if word == "recursive"
                    && nearby
                        .iter()
                        .any(|word| matches!(word.as_str(), "array" | "arrays" | "matrix"))
                {
                    categories.push((position, 3_u8));
                }
            }
            categories.sort_unstable();
            categories.dedup_by_key(|(_, rank)| *rank);
            let nearest_category = |position: usize| {
                categories
                    .iter()
                    .min_by_key(|(candidate, _)| candidate.abs_diff(position))
                    .copied()
            };
            for (position, word) in words.iter().enumerate() {
                let claimed_rank = match word.as_str() {
                    "first" | "one" | "1" => Some(1),
                    "second" | "two" | "2" => Some(2),
                    "third" | "three" | "3" => Some(3),
                    _ => None,
                };
                let Some(claimed_rank) = claimed_rank else {
                    continue;
                };
                let explicit_rank_subject = words
                    .get(position.wrapping_sub(1))
                    .is_some_and(|word| word == "rank");
                let category = if explicit_rank_subject {
                    let immediate_after = categories
                        .iter()
                        .filter(|(candidate, _)| {
                            *candidate > position && *candidate - position <= 3
                        })
                        .min_by_key(|(candidate, _)| *candidate)
                        .copied();
                    let recent_before = categories
                        .iter()
                        .filter(|(candidate, _)| {
                            *candidate < position && position - *candidate <= 8
                        })
                        .max_by_key(|(candidate, _)| *candidate)
                        .copied();
                    immediate_after
                        .or(recent_before)
                        .or_else(|| nearest_category(position))
                } else {
                    let follows_ranking_verb = words[position.saturating_sub(3)..position]
                        .iter()
                        .any(|word| {
                            matches!(word.as_str(), "rank" | "ranks" | "ranked" | "placed")
                        });
                    if follows_ranking_verb {
                        categories
                            .iter()
                            .filter(|(candidate, _)| *candidate < position)
                            .max_by_key(|(candidate, _)| *candidate)
                            .copied()
                            .or_else(|| nearest_category(position))
                    } else {
                        nearest_category(position)
                    }
                };
                let has_rank_relation = explicit_rank_subject
                    || (!word.chars().all(|character| character.is_ascii_digit())
                        && words[position.saturating_sub(3)..position]
                            .iter()
                            .any(|word| {
                                matches!(word.as_str(), "rank" | "ranks" | "ranked" | "placed")
                            }))
                    || words
                        .get(position + 1)
                        .is_some_and(|word| word == "priority")
                    || words
                        .get(position.wrapping_sub(1))
                        .is_some_and(|word| word == "priority");
                if has_rank_relation
                    && !preceded_by_local_negation(&words, position)
                    && category.is_some_and(|(_, expected)| expected != claimed_rank)
                {
                    return Some(format!(
                        "{} :: rank {claimed_rank} at {position}, category {category:?}, categories {categories:?}",
                        clause.trim()
                    ));
                }
            }
            for (position, relation) in words.iter().enumerate() {
                let left_before_right = match relation.as_str() {
                    "precede" | "precedes" | "before" | "ahead" => Some(true),
                    "follow" | "follows" | "after" | "behind" => Some(false),
                    _ => None,
                };
                let Some(left_before_right) = left_before_right else {
                    continue;
                };
                if preceded_by_local_negation(&words, position) {
                    continue;
                }
                let left = categories
                    .iter()
                    .filter(|(candidate, _)| *candidate < position)
                    .max_by_key(|(candidate, _)| *candidate)
                    .map(|(_, rank)| *rank);
                let right = categories
                    .iter()
                    .filter(|(candidate, _)| *candidate > position)
                    .min_by_key(|(candidate, _)| *candidate)
                    .map(|(_, rank)| *rank);
                if matches!((left, right), (Some(left), Some(right)) if left != right && (left < right) != left_before_right)
                {
                    return Some(clause.trim().to_owned());
                }
            }
        }
    }
    None
}

fn assert_post_cap021_successor_order(document_name: &str, document: &str) {
    let rendered = markdown_outside_fences(document);
    let normalized = normalized_words(&rendered);
    for row in POST_CAP021_RANKING_ROWS {
        assert!(
            normalized.contains(table_cells(row).expect("canonical successor row")[1]),
            "{document_name} is missing a post-CAP-021 successor"
        );
    }
    assert!(
        post_cap021_successor_order_violation(document).is_none(),
        "{document_name} states a contradictory post-CAP-021 successor order: {:?}",
        post_cap021_successor_order_violation(document)
    );
}

fn post_cap023_successor_order_violation(text: &str) -> Option<String> {
    let ranked_rendered = markdown_with_ordered_list_ranks(text);
    let ranked_lines = ranked_rendered.lines().map(table_line).collect::<Vec<_>>();
    for (header_index, header) in ranked_lines.iter().enumerate() {
        let Some(cells) = table_cells(header) else {
            continue;
        };
        let rank_column = cells.iter().position(|cell| {
            cell.eq_ignore_ascii_case("rank") || cell.eq_ignore_ascii_case("priority")
        });
        let capability_column = cells.iter().position(|cell| {
            cell.eq_ignore_ascii_case("capability")
                || cell.eq_ignore_ascii_case("capability gap")
                || cell.eq_ignore_ascii_case("work")
        });
        let (Some(rank_column), Some(capability_column)) = (rank_column, capability_column) else {
            continue;
        };
        if !ranked_lines
            .get(header_index + 1)
            .and_then(|line| table_cells(line))
            .is_some_and(|delimiter| {
                delimiter.len() == cells.len()
                    && delimiter
                        .iter()
                        .all(|cell| valid_markdown_delimiter_cell(cell))
            })
        {
            continue;
        }
        for row in ranked_lines[header_index + 2..]
            .iter()
            .take_while(|line| !line.is_empty() && table_cells(line).is_some())
        {
            let row_cells = table_cells(row).expect("table row");
            if row_cells.len() != cells.len() {
                continue;
            }
            let rank_words = semantic_words(row_cells[rank_column]);
            let claimed = rank_words.iter().find_map(|word| match word.as_str() {
                "one" | "first" | "top" | "highest" | "primary" => Some(1_u8),
                "two" | "second" => Some(2_u8),
                "three" | "third" | "last" | "lowest" => Some(3_u8),
                "four" | "fourth" => Some(4_u8),
                _ => word.parse::<u8>().ok(),
            });
            let category_words = semantic_words(row_cells[capability_column]);
            let expected = if category_words.iter().any(|word| word == "quantization")
                || category_words.iter().any(|word| word == "quantized")
            {
                Some(3)
            } else if category_words.iter().any(|word| word == "copydata")
                && category_words
                    .iter()
                    .any(|word| matches!(word.as_str(), "application" | "composition" | "profile"))
            {
                Some(2)
            } else if category_words.iter().any(|word| word == "accepted")
                && category_words.iter().any(|word| word == "head")
                && category_words
                    .iter()
                    .any(|word| matches!(word.as_str(), "evidence" | "reproducibility"))
            {
                Some(1)
            } else {
                None
            };
            if matches!((claimed, expected), (Some(claimed), Some(expected)) if claimed != expected)
            {
                return Some((*row).to_owned());
            }
        }
    }
    let mut section_category: Option<(usize, u8)> = None;
    let mut preserved_strategy_section: Option<usize> = None;
    for record in normalized_claim_records_from_rendered(&ranked_rendered) {
        let normalized_record = normalized_words(&record);
        let decision_record = normalized_record
            .strip_prefix("Rank ")
            .and_then(|rest| rest.split_once(' ').map(|(_, decision)| decision))
            .unwrap_or(&normalized_record);
        let record_words = semantic_words(&record);
        if let Some((level, _)) = claim_heading(&record) {
            if contains_semantic_phrase(&record_words, &["killer", "application", "direction"]) {
                preserved_strategy_section = Some(level);
            } else if preserved_strategy_section.is_some_and(|owned_level| level <= owned_level) {
                preserved_strategy_section = None;
            }
            let category = if record_words
                .iter()
                .any(|word| matches!(word.as_str(), "quantized" | "quantization"))
            {
                Some(3)
            } else if record_words.iter().any(|word| word == "copydata")
                && record_words
                    .iter()
                    .any(|word| matches!(word.as_str(), "application" | "profile" | "composition"))
            {
                Some(2)
            } else if has_semantic_capability(&record_words, "023")
                && record_words
                    .iter()
                    .any(|word| matches!(word.as_str(), "evidence" | "reproducibility"))
            {
                Some(1)
            } else {
                None
            };
            if let Some(category) = category {
                section_category = Some((level, category));
            } else if section_category.is_some_and(|(owned_level, _)| level <= owned_level) {
                section_category = None;
            }
        }
        if preserved_strategy_section.is_some() {
            continue;
        }
        if POST_CAP023_DECISION_CONTRACTS.contains(&decision_record)
            || record_words.first().is_some_and(|word| word == "stop")
        {
            continue;
        }
        for clause in record.split(['.', ';', '!', '?']) {
            let words = semantic_words(clause);
            let historical_clause = words.iter().any(|word| {
                matches!(
                    word.as_str(),
                    "historical" | "historically" | "former" | "formerly" | "previously"
                )
            }) || words.iter().any(|word| word == "before")
                && has_semantic_capability(&words, "023");
            if historical_clause
                && !words.iter().any(|word| {
                    matches!(
                        word.as_str(),
                        "now" | "current" | "today" | "still" | "remains"
                    )
                })
            {
                continue;
            }
            let mut categories = Vec::new();
            for position in 0..words.len() {
                let nearby = &words[position..(position + 10).min(words.len())];
                if (words[position] == "accepted"
                    && nearby.iter().any(|word| word == "head")
                    && nearby.iter().any(|word| word == "evidence"))
                    || ((words[position] == "cap023"
                        || (words[position] == "cap"
                            && words.get(position + 1).is_some_and(|word| word == "023")))
                        && nearby.iter().any(|word| {
                            matches!(word.as_str(), "evidence" | "reproducibility" | "footprint")
                        }))
                {
                    categories.push((position, 1_u8));
                } else if (words[position] == "copydata"
                    && nearby
                        .iter()
                        .any(|word| matches!(word.as_str(), "composition" | "profile")))
                    || (words[position] == "application"
                        && nearby.iter().any(|word| word == "profile")
                        && nearby.iter().any(|word| word == "composition"))
                {
                    categories.push((position, 2_u8));
                } else if matches!(words[position].as_str(), "quantized" | "quantization") {
                    categories.push((position, 3_u8));
                } else if (words[position] == "runtime"
                    && nearby
                        .iter()
                        .any(|word| matches!(word.as_str(), "acquisition" | "ingress" | "input")))
                    || (words[position] == "recursive"
                        && nearby.iter().any(|word| word == "arrays"))
                {
                    categories.push((position, 4_u8));
                }
            }
            categories.sort_unstable();
            categories.dedup_by_key(|(_, rank)| *rank);
            if categories.is_empty()
                && section_category.is_some()
                && words
                    .first()
                    .is_some_and(|word| matches!(word.as_str(), "it" | "this" | "that"))
            {
                categories.push((0, section_category.expect("section category").1));
            }
            let nearest_category = |position: usize| {
                categories
                    .iter()
                    .min_by_key(|(candidate, _)| candidate.abs_diff(position))
                    .copied()
            };
            let rank_claims = words
                .iter()
                .enumerate()
                .filter_map(|(position, word)| {
                    let claimed_rank = match word.as_str() {
                        "first" | "one" | "top" | "highest" | "primary" => Some(1),
                        "second" | "two" => Some(2),
                        "third" | "three" | "last" | "lowest" => Some(3),
                        "fourth" | "four" => Some(4),
                        _ => word.parse::<u8>().ok(),
                    }?;
                    let has_rank_relation = words
                        .get(position.wrapping_sub(1))
                        .is_some_and(|word| word == "rank")
                        || words[position.saturating_sub(3)..position]
                            .iter()
                            .any(|word| {
                                matches!(word.as_str(), "rank" | "ranks" | "ranked" | "placed")
                            })
                        || words
                            .get(position + 1)
                            .is_some_and(|word| word == "priority")
                        || words
                            .get(position.wrapping_sub(1))
                            .is_some_and(|word| word == "priority");
                    (has_rank_relation && !preceded_by_local_negation(&words, position))
                        .then_some((position, claimed_rank))
                })
                .collect::<Vec<_>>();
            let coordinated_rank_positions = if categories.len() > 1
                && rank_claims.len() == categories.len()
                && rank_claims
                    .first()
                    .zip(categories.first())
                    .is_some_and(|((claim, _), (category, _))| claim > category)
            {
                if categories
                    .iter()
                    .zip(&rank_claims)
                    .any(|((_, expected), (_, claimed))| expected != claimed)
                {
                    return Some(clause.trim().to_owned());
                }
                rank_claims
                    .iter()
                    .map(|(position, _)| *position)
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            for (position, word) in words.iter().enumerate() {
                let claimed_rank = match word.as_str() {
                    "first" | "one" | "top" | "highest" | "primary" => Some(1),
                    "second" | "two" => Some(2),
                    "third" | "three" | "last" | "lowest" => Some(3),
                    "fourth" | "four" => Some(4),
                    _ => word.parse::<u8>().ok(),
                };
                let Some(claimed_rank) = claimed_rank else {
                    continue;
                };
                if coordinated_rank_positions.contains(&position) {
                    continue;
                }
                let explicit_rank_subject = words
                    .get(position.wrapping_sub(1))
                    .is_some_and(|word| word == "rank");
                let has_rank_relation = explicit_rank_subject
                    || words[position.saturating_sub(3)..position]
                        .iter()
                        .any(|word| {
                            matches!(word.as_str(), "rank" | "ranks" | "ranked" | "placed")
                        })
                    || words
                        .get(position + 1)
                        .is_some_and(|word| word == "priority")
                    || words
                        .get(position.wrapping_sub(1))
                        .is_some_and(|word| word == "priority");
                if has_rank_relation
                    && !preceded_by_local_negation(&words, position)
                    && nearest_category(position)
                        .is_some_and(|(_, expected)| expected != claimed_rank)
                {
                    return Some(clause.trim().to_owned());
                }
            }
            for (position, relation) in words.iter().enumerate() {
                let left_before_right = match relation.as_str() {
                    "precede" | "precedes" | "before" | "ahead" => Some(true),
                    "follow" | "follows" | "after" | "behind" => Some(false),
                    _ => None,
                };
                let Some(left_before_right) = left_before_right else {
                    continue;
                };
                if preceded_by_local_negation(&words, position) {
                    continue;
                }
                let left = categories
                    .iter()
                    .filter(|(candidate, _)| *candidate < position)
                    .max_by_key(|(candidate, _)| *candidate)
                    .map(|(_, rank)| *rank);
                let right = categories
                    .iter()
                    .filter(|(candidate, _)| *candidate > position)
                    .min_by_key(|(candidate, _)| *candidate)
                    .map(|(_, rank)| *rank);
                if matches!((left, right), (Some(left), Some(right)) if left != right && (left < right) != left_before_right)
                {
                    return Some(clause.trim().to_owned());
                }
            }
        }
    }
    None
}

fn assert_post_cap023_successor_order(document_name: &str, document: &str) {
    let current_surface = if document_name == "Roadmap.md" {
        let historical_start = document
            .find("### Post-CAP-020 ranking")
            .expect("Roadmap.md historical post-CAP-020 ranking");
        let current_start = document
            .find("### Post-CAP-023 ranking")
            .expect("Roadmap.md current post-CAP-023 ranking");
        format!(
            "{}{}",
            &document[..historical_start],
            &document[current_start..]
        )
    } else if document_name == "CURRENT_CAPABILITY_AUDIT.md" {
        let historical_start = document
            .find("### ROADMAP-001 ranking and M1-001 outcome")
            .expect("CURRENT_CAPABILITY_AUDIT.md historical ROADMAP-001 ranking");
        let historical_end = document[historical_start..]
            .find("## Verified progress after the audit commit")
            .map(|offset| historical_start + offset)
            .expect("CURRENT_CAPABILITY_AUDIT.md post-ROADMAP-001 audit history");
        format!(
            "{}{}",
            &document[..historical_start],
            &document[historical_end..]
        )
    } else {
        document.to_owned()
    };
    let normalized = normalized_words(&markdown_outside_fences(&current_surface));
    for row in POST_CAP023_RANKING_ROWS {
        assert!(
            normalized.contains(table_cells(row).expect("canonical successor row")[1]),
            "{document_name} is missing a post-CAP-023 successor"
        );
    }
    assert!(
        post_cap023_successor_order_violation(&current_surface).is_none(),
        "{document_name} states a contradictory post-CAP-023 successor order: {:?}",
        post_cap023_successor_order_violation(&current_surface)
    );
}

fn post_cap024_category(words: &[String]) -> Option<u8> {
    if words
        .iter()
        .any(|word| matches!(word.as_str(), "quantized" | "quantization"))
    {
        Some(3)
    } else if words.iter().any(|word| {
        matches!(
            word.as_str(),
            "vec" | "vector" | "vectors" | "list" | "lists"
        )
    }) || words.iter().any(|word| word == "string")
        && words
            .iter()
            .any(|word| matches!(word.as_str(), "owned" | "dynamic"))
        || words
            .iter()
            .any(|word| matches!(word.as_str(), "collection" | "collections" | "streaming"))
            && words.iter().any(|word| {
                matches!(
                    word.as_str(),
                    "owned" | "dynamic" | "allocation" | "allocator" | "drop" | "foundation"
                )
            })
    {
        Some(2)
    } else if words.iter().any(|word| word == "copydata")
        && words
            .iter()
            .any(|word| matches!(word.as_str(), "application" | "profile" | "composition"))
    {
        Some(1)
    } else {
        None
    }
}

fn rank_word(word: &str) -> Option<u8> {
    match word {
        "one" | "first" | "top" | "highest" | "primary" => Some(1),
        "two" | "second" => Some(2),
        "three" | "third" | "last" | "lowest" => Some(3),
        "four" | "fourth" => Some(4),
        _ => word.parse::<u8>().ok(),
    }
}

fn post_cap024_successor_order_violation(text: &str) -> Option<String> {
    let ranked_rendered = markdown_with_ordered_list_ranks(text);
    let ranked_lines = ranked_rendered.lines().map(table_line).collect::<Vec<_>>();
    for (header_index, header) in ranked_lines.iter().enumerate() {
        let Some(cells) = table_cells(header) else {
            continue;
        };
        let rank_column = cells.iter().position(|cell| {
            cell.eq_ignore_ascii_case("rank") || cell.eq_ignore_ascii_case("priority")
        });
        let capability_column = cells.iter().position(|cell| {
            cell.eq_ignore_ascii_case("capability")
                || cell.eq_ignore_ascii_case("capability gap")
                || cell.eq_ignore_ascii_case("work")
        });
        let (Some(rank_column), Some(capability_column)) = (rank_column, capability_column) else {
            continue;
        };
        if !ranked_lines
            .get(header_index + 1)
            .and_then(|line| table_cells(line))
            .is_some_and(|delimiter| {
                delimiter.len() == cells.len()
                    && delimiter
                        .iter()
                        .all(|cell| valid_markdown_delimiter_cell(cell))
            })
        {
            continue;
        }
        for row in ranked_lines[header_index + 2..]
            .iter()
            .take_while(|line| !line.is_empty() && table_cells(line).is_some())
        {
            let row_cells = table_cells(row).expect("table row");
            if row_cells.len() != cells.len() {
                continue;
            }
            let claimed = semantic_words(row_cells[rank_column])
                .iter()
                .find_map(|word| rank_word(word));
            let expected = post_cap024_category(&semantic_words(row_cells[capability_column]));
            if matches!((claimed, expected), (Some(claimed), Some(expected)) if claimed != expected)
            {
                return Some((*row).to_owned());
            }
        }
    }

    let mut section_category: Option<(usize, u8)> = None;
    for record in normalized_claim_records_from_rendered(&ranked_rendered) {
        let normalized_record = normalized_words(&record);
        let decision_record = normalized_record
            .strip_prefix("Rank ")
            .and_then(|rest| rest.split_once(' ').map(|(_, decision)| decision))
            .unwrap_or(&normalized_record);
        if POST_CAP024_DECISION_CONTRACTS.contains(&decision_record) {
            continue;
        }
        if let Some((level, _)) = claim_heading(&record) {
            if let Some(category) = post_cap024_category(&semantic_words(&record)) {
                section_category = Some((level, category));
            } else if section_category.is_some_and(|(owned_level, _)| level <= owned_level) {
                section_category = None;
            }
        }
        for clause in record.split(['.', ';', '!', '?']) {
            let words = semantic_words(clause);
            let historical = words.iter().any(|word| {
                matches!(
                    word.as_str(),
                    "historical" | "historically" | "former" | "formerly" | "previously"
                )
            }) && !words.iter().any(|word| {
                matches!(
                    word.as_str(),
                    "now" | "current" | "today" | "still" | "remains"
                )
            });
            if historical {
                continue;
            }
            let mut categories = Vec::new();
            for (position, word) in words.iter().enumerate() {
                let category = if matches!(word.as_str(), "quantized" | "quantization") {
                    Some(3)
                } else if word == "copydata"
                    && words.iter().any(|word| {
                        matches!(word.as_str(), "application" | "profile" | "composition")
                    })
                {
                    Some(1)
                } else if matches!(
                    word.as_str(),
                    "vec" | "vector" | "vectors" | "list" | "lists"
                ) || word == "string"
                    && words
                        .iter()
                        .any(|word| matches!(word.as_str(), "owned" | "dynamic"))
                    || matches!(word.as_str(), "collection" | "collections" | "streaming")
                        && words.iter().any(|word| {
                            matches!(
                                word.as_str(),
                                "owned"
                                    | "dynamic"
                                    | "allocation"
                                    | "allocator"
                                    | "drop"
                                    | "foundation"
                            )
                        })
                {
                    Some(2)
                } else {
                    None
                };
                if let Some(category) = category {
                    categories.push((position, category));
                }
            }
            categories.sort_unstable();
            categories.dedup_by_key(|(_, category)| *category);
            if categories.is_empty()
                && section_category.is_some()
                && words
                    .first()
                    .is_some_and(|word| matches!(word.as_str(), "it" | "this" | "that"))
            {
                categories.push((0, section_category.expect("section category").1));
            }
            if categories.is_empty() {
                continue;
            }
            let nearest_category = |position: usize| {
                categories
                    .iter()
                    .min_by_key(|(candidate, _)| candidate.abs_diff(position))
                    .copied()
            };
            for (position, word) in words.iter().enumerate() {
                let Some(claimed) = rank_word(word) else {
                    continue;
                };
                let rank_relation = words
                    .get(position.wrapping_sub(1))
                    .is_some_and(|word| word == "rank")
                    || words[position.saturating_sub(3)..position]
                        .iter()
                        .any(|word| {
                            matches!(word.as_str(), "rank" | "ranks" | "ranked" | "placed")
                        })
                    || words
                        .get(position + 1)
                        .is_some_and(|word| word == "priority")
                    || words
                        .get(position.wrapping_sub(1))
                        .is_some_and(|word| word == "priority");
                if rank_relation
                    && !preceded_by_local_negation(&words, position)
                    && nearest_category(position).is_some_and(|(_, expected)| expected != claimed)
                {
                    return Some(clause.trim().to_owned());
                }
            }
            for (position, relation) in words.iter().enumerate() {
                let left_before_right = match relation.as_str() {
                    "precede" | "precedes" | "before" | "ahead" => Some(true),
                    "follow" | "follows" | "after" | "behind" => Some(false),
                    _ => None,
                };
                let Some(left_before_right) = left_before_right else {
                    continue;
                };
                if preceded_by_local_negation(&words, position) {
                    continue;
                }
                let left = categories
                    .iter()
                    .filter(|(candidate, _)| *candidate < position)
                    .max_by_key(|(candidate, _)| *candidate)
                    .map(|(_, rank)| *rank);
                let right = categories
                    .iter()
                    .filter(|(candidate, _)| *candidate > position)
                    .min_by_key(|(candidate, _)| *candidate)
                    .map(|(_, rank)| *rank);
                if matches!((left, right), (Some(left), Some(right)) if left != right && (left < right) != left_before_right)
                {
                    return Some(clause.trim().to_owned());
                }
            }
        }
    }
    None
}

fn assert_post_cap024_successor_order(document_name: &str, document: &str) {
    let current_surface = if document_name == "Roadmap.md" {
        let historical_start = document
            .find("### Post-CAP-020 ranking")
            .expect("Roadmap.md historical post-CAP-020 ranking");
        let current_start = document
            .find("### Post-CAP-024 ranking")
            .expect("Roadmap.md current post-CAP-024 ranking");
        format!(
            "{}{}",
            &document[..historical_start],
            &document[current_start..]
        )
    } else if document_name == "CURRENT_CAPABILITY_AUDIT.md" {
        let historical_start = document
            .find("### ROADMAP-001 ranking and M1-001 outcome")
            .expect("CURRENT_CAPABILITY_AUDIT.md historical ROADMAP-001 ranking");
        let historical_end = document[historical_start..]
            .find("## Verified progress after the audit commit")
            .map(|offset| historical_start + offset)
            .expect("CURRENT_CAPABILITY_AUDIT.md post-ROADMAP-001 audit history");
        format!(
            "{}{}",
            &document[..historical_start],
            &document[historical_end..]
        )
    } else {
        document.to_owned()
    };
    let normalized = normalized_words(&markdown_outside_fences(&current_surface));
    for row in POST_CAP024_RANKING_ROWS {
        assert!(
            normalized.contains(table_cells(row).expect("canonical successor row")[1]),
            "{document_name} is missing a post-CAP-024 successor"
        );
    }
    assert!(
        post_cap024_successor_order_violation(&current_surface).is_none(),
        "{document_name} states a contradictory post-CAP-024 successor order: {:?}",
        post_cap024_successor_order_violation(&current_surface)
    );
}

fn clause_words(clause: &str) -> Vec<&str> {
    clause
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '-')
        .filter(|word| !word.is_empty())
        .collect()
}

fn semantic_words(text: &str) -> Vec<String> {
    text.split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(|word| word.to_ascii_lowercase())
        .collect()
}

fn contains_semantic_phrase(words: &[String], phrase: &[&str]) -> bool {
    words.windows(phrase.len()).any(|candidate| {
        candidate
            .iter()
            .map(String::as_str)
            .eq(phrase.iter().copied())
    })
}

fn has_semantic_capability(words: &[String], number: &str) -> bool {
    words.iter().any(|word| word == &format!("cap{number}"))
        || words
            .windows(2)
            .any(|pair| pair[0] == "cap" && pair[1] == number)
}

fn stale_cap019_current_violation(document: &str) -> Option<String> {
    for record in normalized_claim_records(document) {
        for clause in record.split(['.', ';', '!', '?']) {
            let words = semantic_words(clause);
            if !has_semantic_capability(&words, "019") {
                continue;
            }
            let has = |word: &str| words.iter().any(|candidate| candidate == word);
            let explicit_current =
                has("current") || has("present") || has("today") || has("remains") || has("still");
            let historical = has("former")
                || has("formerly")
                || has("previously")
                || has("historical")
                || has("archived")
                || has("was")
                || has("were")
                || (has("before") && has_semantic_capability(&words, "020"));
            let state_positions = words
                .iter()
                .enumerate()
                .filter(|(_, word)| {
                    matches!(
                        word.as_str(),
                        "master"
                            | "baseline"
                            | "head"
                            | "status"
                            | "state"
                            | "ranking"
                            | "order"
                            | "successor"
                    )
                })
                .map(|(position, _)| position)
                .collect::<Vec<_>>();
            let cap019_state = state_positions.iter().copied().find(|position| {
                nearest_capability_owner(&words, *position, 1).as_deref() == Some("019")
            });
            let state_negated = cap019_state
                .is_some_and(|position| preceded_by_local_negation(&words, position))
                || contains_semantic_phrase(&words, &["no", "longer"])
                || contains_semantic_phrase(&words, &["ceased", "to", "be"]);
            let current_capability = explicit_current
                && has("accepted")
                && has("capability")
                && !(has("compiler") && has("profile"));
            let present_state_copula = cap019_state.is_some() && has("is") && !historical;
            let current_state = cap019_state.is_some()
                && (explicit_current || present_state_copula || has("post"))
                && (!historical
                    || has("today")
                    || has("current")
                    || has("remains")
                    || has("still"));
            let stale = !state_negated
                && (current_state
                    || current_capability
                    || (has("project") && has("status") && has("after") && cap019_state.is_some()));
            if stale {
                return Some(clause.trim().to_owned());
            }
        }
    }
    None
}

fn assert_no_stale_cap019_current_claims(document_name: &str, document: &str) {
    assert!(
        stale_cap019_current_violation(document).is_none(),
        "{document_name} presents CAP-019 as current state or current ranking: {:?}",
        stale_cap019_current_violation(document)
    );
}

fn stale_cap020_current_violation(document: &str) -> Option<String> {
    for record in normalized_claim_records(document) {
        for clause in record.split(['.', ';', '!', '?']) {
            let words = semantic_words(clause);
            if !has_semantic_capability(&words, "020") {
                continue;
            }
            let has = |word: &str| words.iter().any(|candidate| candidate == word);
            let state_positions = words
                .iter()
                .enumerate()
                .filter(|(_, word)| {
                    matches!(
                        word.as_str(),
                        "master"
                            | "baseline"
                            | "head"
                            | "status"
                            | "state"
                            | "ranking"
                            | "order"
                            | "successor"
                    )
                })
                .map(|(position, _)| position)
                .collect::<Vec<_>>();
            let cap020_state = state_positions.iter().copied().find(|position| {
                nearest_capability_owner(&words, *position, 1).as_deref() == Some("020")
            });
            let explicit_current = has("current")
                || has("latest")
                || has("present")
                || has("today")
                || has("remains")
                || has("still")
                || has("continues");
            let historical = has("former")
                || has("formerly")
                || has("previously")
                || has("historical")
                || has("archived")
                || has("was")
                || has("were")
                || (has("before") && has_semantic_capability(&words, "021"));
            let state_negated = cap020_state
                .is_some_and(|position| preceded_by_local_negation(&words, position))
                || contains_semantic_phrase(&words, &["no", "longer"])
                || contains_semantic_phrase(&words, &["ceased", "to", "be"]);
            let present_copula = cap020_state.is_some() && has("is") && !historical;
            if !state_negated
                && cap020_state.is_some()
                && (present_copula
                    || explicit_current && (!historical || has("today") || has("current")))
            {
                return Some(clause.trim().to_owned());
            }
        }
    }
    None
}

fn stale_pre_cap023_current_violation(document: &str) -> Option<String> {
    for record in normalized_claim_records(document) {
        for clause in record.split(['.', ';', '!', '?', ',']) {
            let words = semantic_words(clause);
            let target = ["021", "022"]
                .into_iter()
                .find(|target| has_semantic_capability(&words, target));
            let Some(target) = target else {
                continue;
            };
            let has = |word: &str| words.iter().any(|candidate| candidate == word);
            let current_marker = has("current")
                || has("latest")
                || has("present")
                || has("today")
                || has("baseline")
                || has("remains")
                || has("still")
                || has("continues")
                || has("incumbent");
            let explicit_history = has("former")
                || has("formerly")
                || has("previously")
                || has("historical")
                || has("archived");
            let strong_present = has("current")
                || has("present")
                || has("today")
                || ((has("remains") || has("still") || has("continues")) && !explicit_history);
            let state_position = words.iter().enumerate().find_map(|(position, word)| {
                matches!(
                    word.as_str(),
                    "master" | "baseline" | "head" | "status" | "state" | "ranking" | "order"
                )
                .then_some(position)
            });
            let target_owns_state = state_position.is_some_and(|position| {
                nearest_capability_owner(&words, position, 1).as_deref() == Some(target)
            });
            let historical = has("former")
                || has("formerly")
                || has("previously")
                || has("historical")
                || has("archived")
                || has("was")
                || has("were")
                || contains_semantic_phrase(&words, &["no", "longer"]);
            if target == "022" {
                if let Some(position) = words.iter().position(|word| word == "implementation") {
                    let promoted = words.iter().any(|word| {
                        matches!(
                            word.as_str(),
                            "approved"
                                | "authorized"
                                | "begun"
                                | "implemented"
                                | "proceed"
                                | "scheduled"
                                | "planned"
                                | "begins"
                                | "begin"
                                | "started"
                                | "underway"
                        )
                    });
                    if promoted && !relation_is_negated(&words, position, 1) {
                        return Some(clause.trim().to_owned());
                    }
                }
                if has("accepted")
                    && has("capability")
                    && !words.iter().enumerate().any(|(position, word)| {
                        word == "accepted" && preceded_by_local_negation(&words, position)
                    })
                {
                    return Some(clause.trim().to_owned());
                }
            }
            let state_negated =
                state_position.is_some_and(|position| preceded_by_local_negation(&words, position));
            let present_state = target_owns_state && (has("is") || has("holds")) && !historical;
            let explicit_current_capability = current_marker
                && has("accepted")
                && has("capability")
                && !(has("compiler") && has("profile"));
            if !state_negated
                && state_position.is_some()
                && (has("remains") || has("still") || has("continues"))
                && !explicit_history
                && !(has("compiler") && has("profile"))
            {
                return Some(clause.trim().to_owned());
            }
            if !state_negated
                && target_owns_state
                && (has("remains") || has("still") || has("continues"))
                && !explicit_history
            {
                return Some(clause.trim().to_owned());
            }
            if !state_negated
                && (!historical || strong_present)
                && ((current_marker && target_owns_state)
                    || present_state
                    || explicit_current_capability)
            {
                return Some(clause.trim().to_owned());
            }
        }
    }
    None
}

fn stale_cap023_current_head_violation(document: &str) -> Option<String> {
    for record in normalized_claim_records(document) {
        let mut carried_owner = None;
        for clause in record.split(['.', ';', '!', '?', ',']) {
            let words = semantic_words(clause);
            let mentions = capability_mentions(&words);
            if let Some((_, _, owner)) = mentions.last() {
                carried_owner = Some(owner.clone());
            }
            if !has_semantic_capability(&words, "023")
                && !(mentions.is_empty() && carried_owner.as_deref() == Some("023"))
            {
                continue;
            }
            let has = |word: &str| words.iter().any(|candidate| candidate == word);
            let state_positions = words
                .iter()
                .enumerate()
                .filter(|(_, word)| {
                    matches!(word.as_str(), "master" | "head" | "baseline" | "incumbent")
                })
                .map(|(position, _)| position)
                .collect::<Vec<_>>();
            let cap023_state = state_positions.iter().copied().find(|position| {
                nearest_capability_owner(&words, *position, 1).as_deref() == Some("023")
                    || mentions.is_empty() && carried_owner.as_deref() == Some("023")
            });
            let Some(state_position) = cap023_state else {
                continue;
            };
            let historical = has("historical")
                || has("formerly")
                || has("former")
                || has("previously")
                || has("archived")
                || has("was")
                || has("were")
                || (has("before") && has_semantic_capability(&words, "024"));
            let state_negated = preceded_by_local_negation(&words, state_position)
                || contains_semantic_phrase(&words, &["no", "longer"])
                || contains_semantic_phrase(&words, &["not", "current"])
                || contains_semantic_phrase(&words, &["ceased", "to", "be"]);
            let current = has("current")
                || has("latest")
                || has("present")
                || has("today")
                || has("protected")
                || has("remains")
                || has("still")
                || has("continues")
                || has("is");
            if !state_negated && current && !historical {
                return Some(clause.trim().to_owned());
            }
            if !state_negated
                && historical
                && (words.iter().any(|word| {
                    matches!(
                        word.as_str(),
                        "now" | "today" | "still" | "remains" | "continues"
                    )
                }) || words
                    .iter()
                    .any(|word| matches!(word.as_str(), "but" | "yet" | "however"))
                    && has("is"))
            {
                return Some(clause.trim().to_owned());
            }
        }
    }
    None
}

fn capability_mentions(words: &[String]) -> Vec<(usize, usize, String)> {
    let mut mentions = Vec::new();
    let mut index = 0;
    while index < words.len() {
        let compact = words[index]
            .strip_prefix("cap")
            .filter(|number| !number.is_empty() && number.chars().all(|ch| ch.is_ascii_digit()));
        let split = (words[index] == "cap")
            .then(|| words.get(index + 1))
            .flatten()
            .filter(|number| number.chars().all(|ch| ch.is_ascii_digit()))
            .map(String::as_str);
        if let Some(number) = compact.or(split) {
            let end = index + 1 + usize::from(words[index] == "cap");
            mentions.push((index, end, number.to_owned()));
            index = end;
            continue;
        }
        index += 1;
    }
    mentions
}

fn nearest_capability_owner(
    words: &[String],
    subject_start: usize,
    subject_len: usize,
) -> Option<String> {
    let subject_end = subject_start + subject_len;
    capability_mentions(words)
        .into_iter()
        .min_by_key(|(start, end, _)| {
            let distance = if *end <= subject_start {
                subject_start - *end
            } else if *start >= subject_end {
                *start - subject_end
            } else {
                0
            };
            (distance, usize::from(*start >= subject_end))
        })
        .map(|(_, _, number)| number)
}

fn has_affirmative_relation(words: &[String], subject_start: usize, subject_len: usize) -> bool {
    let fragment_start = words[..subject_start]
        .iter()
        .rposition(|word| matches!(word.as_str(), "but" | "yet" | "however" | "whereas"))
        .map_or(0, |position| position + 1);
    let fragment_end = words[subject_start + subject_len..]
        .iter()
        .position(|word| matches!(word.as_str(), "but" | "yet" | "however" | "whereas"))
        .map_or(words.len(), |position| {
            subject_start + subject_len + position
        });
    let start = fragment_start.max(subject_start.saturating_sub(12));
    let end = fragment_end.min(subject_start + subject_len + 8);
    words[start..end].iter().enumerate().any(|(offset, word)| {
        let positive = matches!(
            word.as_str(),
            "add"
                | "adds"
                | "added"
                | "support"
                | "supports"
                | "supported"
                | "enable"
                | "enables"
                | "enabled"
                | "provide"
                | "provides"
                | "provided"
                | "implement"
                | "implements"
                | "implemented"
                | "create"
                | "creates"
                | "created"
                | "admit"
                | "admits"
                | "admitted"
        );
        positive && !preceded_by_local_negation(words, start + offset)
    })
}

fn semantic_phrase_position(words: &[String], phrase: &[&str]) -> Option<usize> {
    words.windows(phrase.len()).position(|candidate| {
        candidate
            .iter()
            .map(String::as_str)
            .eq(phrase.iter().copied())
    })
}

fn relation_is_negated(words: &[String], subject_start: usize, subject_len: usize) -> bool {
    let fragment_start = words[..subject_start]
        .iter()
        .rposition(|word| matches!(word.as_str(), "but" | "yet" | "however" | "whereas"))
        .map_or(0, |position| position + 1);
    let fragment_end = words[subject_start + subject_len..]
        .iter()
        .position(|word| matches!(word.as_str(), "but" | "yet" | "however" | "whereas"))
        .map_or(words.len(), |position| {
            subject_start + subject_len + position
        });
    let relation_before = words[fragment_start..subject_start]
        .iter()
        .rposition(|word| {
            matches!(
                word.as_str(),
                "add"
                    | "adds"
                    | "added"
                    | "support"
                    | "supports"
                    | "supported"
                    | "enable"
                    | "enables"
                    | "enabled"
                    | "provide"
                    | "provides"
                    | "create"
                    | "creates"
                    | "change"
                    | "changes"
                    | "edit"
                    | "edits"
                    | "modify"
                    | "modifies"
                    | "implement"
                    | "implements"
                    | "guarantee"
                    | "guarantees"
                    | "stabilize"
                    | "stabilizes"
                    | "admit"
                    | "admits"
                    | "accept"
                    | "accepts"
                    | "require"
                    | "requires"
                    | "need"
                    | "needs"
                    | "offer"
                    | "offers"
                    | "give"
                    | "gives"
                    | "contain"
                    | "contains"
                    | "work"
                    | "works"
                    | "is"
                    | "are"
                    | "has"
                    | "have"
                    | "belong"
                    | "belongs"
                    | "credit"
                    | "credits"
                    | "own"
                    | "owns"
                    | "lack"
                    | "lacks"
                    | "exclude"
                    | "excludes"
                    | "omit"
                    | "omits"
                    | "reject"
                    | "rejects"
            )
        })
        .map(|position| fragment_start + position);
    let relation_after = words[subject_start + subject_len..fragment_end]
        .iter()
        .position(|word| {
            matches!(
                word.as_str(),
                "support"
                    | "supports"
                    | "supported"
                    | "enable"
                    | "enables"
                    | "enabled"
                    | "add"
                    | "adds"
                    | "added"
                    | "provide"
                    | "provides"
                    | "provided"
                    | "work"
                    | "works"
                    | "available"
                    | "complete"
                    | "accepted"
                    | "implemented"
                    | "is"
                    | "are"
                    | "was"
                    | "were"
                    | "remain"
                    | "remains"
                    | "belong"
                    | "belongs"
            )
        })
        .map(|position| subject_start + subject_len + position);
    let relation = match (relation_before, relation_after) {
        (Some(before), Some(after)) => {
            let before_distance = subject_start.saturating_sub(before);
            let after_distance = after.saturating_sub(subject_start + subject_len);
            Some(
                if after_distance <= 2 && after_distance <= before_distance {
                    after
                } else {
                    before
                },
            )
        }
        (before, after) => before.or(after),
    };
    let negative_verb = relation.is_some_and(|position| {
        matches!(
            words[position].as_str(),
            "lack" | "lacks" | "exclude" | "excludes" | "omit" | "omits" | "reject" | "rejects"
        )
    });
    let relation_follows_subject =
        relation.is_some_and(|position| position >= subject_start + subject_len);
    let local_start = relation.map_or(subject_start.saturating_sub(3), |position| {
        if relation_follows_subject {
            subject_start
        } else {
            position.saturating_sub(3).max(fragment_start)
        }
    });
    let local_end = relation.map_or(subject_start + subject_len, |position| {
        if relation_follows_subject {
            (position + 4).min(fragment_end)
        } else {
            subject_start + subject_len
        }
    });
    let local = &words[local_start..local_end];
    let explicit_negative = local.iter().enumerate().any(|(position, word)| {
        let not_only = word == "not" && local.get(position + 1).is_some_and(|next| next == "only");
        let double_negative_without = word == "without"
            && (position > 0 && matches!(local[position - 1].as_str(), "not" | "never")
                || position > 1 && local[position - 2] == "no" && local[position - 1] == "longer");
        matches!(word.as_str(), "not" | "no" | "never" | "without" | "cannot")
            && !not_only
            && !double_negative_without
    }) || local.windows(2).any(|pair| {
        pair[1] == "t"
            && matches!(
                pair[0].as_str(),
                "can"
                    | "isn"
                    | "doesn"
                    | "wasn"
                    | "won"
                    | "aren"
                    | "didn"
                    | "hasn"
                    | "haven"
                    | "couldn"
                    | "wouldn"
                    | "shouldn"
            )
    });
    let after_subject = &words[subject_start + subject_len..fragment_end];
    let negative_value_position = relation_follows_subject
        .then(|| {
            after_subject.iter().take(10).position(|word| {
                matches!(
                    word.as_str(),
                    "absent"
                        | "unsupported"
                        | "excluded"
                        | "omitted"
                        | "rejected"
                        | "unchanged"
                        | "deferred"
                        | "future"
                        | "open"
                )
            })
        })
        .flatten();
    let negative_value = negative_value_position.is_some()
        || (relation.is_none()
            && (after_subject.iter().any(|word| word == "readiness")
                || after_subject.iter().any(|word| word == "probe"))
            && (after_subject.iter().any(|word| word == "only") || relation_before.is_none()));
    let relation_prefix = &words[fragment_start..subject_start];
    let denied_acquisition = relation_prefix.iter().any(|word| word == "no")
        && relation_prefix
            .iter()
            .any(|word| matches!(word.as_str(), "source" | "program" | "programs"))
        && relation_prefix
            .iter()
            .any(|word| matches!(word.as_str(), "acquire" | "acquires" | "acquired"));
    let stop_if = relation_prefix.iter().any(|word| word == "stop")
        && relation_prefix.iter().any(|word| word == "if");
    let negative_value_is_negated = negative_value_position
        .is_some_and(|position| preceded_by_local_negation(after_subject, position));
    let explicit_double_negative = local.windows(2).any(|pair| {
        matches!(pair, [left, right] if matches!(left.as_str(), "not" | "never")
            && matches!(right.as_str(), "without" | "no"))
    }) || local.windows(3).any(|triple| {
        matches!(triple, [first, second, third]
            if first == "no" && second == "longer" && third == "without")
    });
    let double_negative = (negative_verb && explicit_negative)
        || negative_value_is_negated
        || explicit_double_negative;
    (!double_negative && (negative_verb || explicit_negative || negative_value))
        || denied_acquisition
        || stop_if
}

fn preceded_by_local_negation(words: &[String], position: usize) -> bool {
    let contrast_start = words[..position]
        .iter()
        .rposition(|word| matches!(word.as_str(), "but" | "yet" | "however"))
        .map_or(0, |index| index + 1);
    let start = contrast_start.max(position.saturating_sub(5));
    let local = &words[start..position];
    local.iter().enumerate().any(|(index, word)| {
        let not_only = word == "not" && local.get(index + 1).is_some_and(|next| next == "only");
        let double_negative = matches!(word.as_str(), "not" | "never")
            && local
                .get(index + 1)
                .is_some_and(|next| matches!(next.as_str(), "without" | "no"))
            || word == "no"
                && local.get(index + 1).is_some_and(|next| next == "longer")
                && local.get(index + 2).is_some_and(|next| next == "without");
        matches!(
            word.as_str(),
            "not" | "no" | "none" | "neither" | "never" | "cannot"
        ) && !not_only
            && !double_negative
    }) || local.windows(2).any(|pair| {
        pair[1] == "t"
            && matches!(
                pair[0].as_str(),
                "can"
                    | "isn"
                    | "doesn"
                    | "wasn"
                    | "won"
                    | "aren"
                    | "didn"
                    | "hasn"
                    | "haven"
                    | "couldn"
                    | "wouldn"
                    | "shouldn"
            )
    })
}

fn cap020_product_violation(text: &str) -> Option<String> {
    const EXCLUDED: &[&[&str]] = &[
        &["parser"],
        &["grammar"],
        &["source", "semantics"],
        &["profile"],
        &["language", "profile"],
        &["compiler", "profile"],
        &["language", "feature"],
        &["feature", "row"],
        &["semantic", "analysis"],
        &["checked", "ir"],
        &["verifier"],
        &["backend"],
        &["compiler", "production"],
        &["compiler"],
        &["compiler", "edits"],
        &["code", "generation"],
        &["codegen"],
        &["production"],
        &["source", "code"],
        &["production", "source", "code"],
        &["production", "compiler"],
        &["production", "code"],
        &["production", "source", "code"],
        &["compiler", "edits"],
        &["compiler", "changes"],
        &["separate", "profile"],
        &["new", "profile"],
        &["profile", "support"],
        &["profile", "supported"],
        &["profile", "is", "supported"],
        &["matrix", "type"],
        &["matrix", "types"],
        &["matrix", "syntax"],
        &["matrix", "support"],
        &["matrices"],
        &["matrices", "supported"],
        &["tensor", "type"],
        &["tensor", "types"],
        &["tensor", "syntax"],
        &["tensor", "support"],
        &["binary", "ingestion"],
        &["runtime", "ingestion"],
        &["file", "input"],
        &["file", "acquisition"],
        &["external", "bytes"],
        &["bounded", "owned", "buffer"],
        &["i", "o"],
        &["runtime", "abi"],
        &["allocation"],
        &["drop"],
        &["quantization"],
        &["inference", "completion"],
        &["stable", "abi"],
        &["abi", "stability"],
        &["stable", "layout"],
        &["layout", "guarantee"],
        &["performance", "claim"],
        &["performance", "guarantee"],
        &["accelerator", "execution"],
        &["accelerator", "support"],
        &["memory", "safety"],
        &["safety", "claim"],
        &["recursive", "arrays"],
        &["nested", "arrays"],
        &["static", "index", "proof"],
        &["checked", "overflow", "arithmetic"],
        &["general", "mutation"],
    ];
    for record in normalized_claim_records(text) {
        let mut carried_owner = None;
        for clause in record.split(['.', ';', '!', '?']) {
            let words = semantic_words(clause);
            let mentions = capability_mentions(&words);
            if let Some((_, _, owner)) = mentions.last() {
                carried_owner = Some(owner.clone());
            }
            let names_successor_without_cap = mentions.is_empty()
                && ((contains_semantic_phrase(&words, &["rank", "2"])
                    || contains_semantic_phrase(&words, &["rank", "3"])
                    || contains_semantic_phrase(&words, &["runtime", "acquisition"])
                    || contains_semantic_phrase(&words, &["recursive", "arrays"]))
                    && words.iter().any(|word| {
                        matches!(word.as_str(), "readiness" | "probe" | "deferred" | "future")
                    }));
            let fallback_owner = (!names_successor_without_cap)
                .then(|| carried_owner.clone())
                .flatten();
            let leading_stop_condition = words.first().is_some_and(|word| word == "stop")
                && words.iter().any(|word| word == "if");
            for phrase in EXCLUDED {
                for position in words
                    .windows(phrase.len())
                    .enumerate()
                    .filter(|(_, candidate)| {
                        candidate
                            .iter()
                            .map(String::as_str)
                            .eq(phrase.iter().copied())
                    })
                    .map(|(position, _)| position)
                {
                    let owner = nearest_capability_owner(&words, position, phrase.len())
                        .or_else(|| fallback_owner.clone());
                    if owner.as_deref() == Some("020")
                        && !leading_stop_condition
                        && !relation_is_negated(&words, position, phrase.len())
                    {
                        return Some(format!("{} :: {}", phrase.join(" "), clause.trim()));
                    }
                }
            }
            for performance_position in words
                .iter()
                .enumerate()
                .filter(|(_, word)| *word == "performance")
                .map(|(position, _)| position)
            {
                let owner = nearest_capability_owner(&words, performance_position, 1)
                    .or_else(|| fallback_owner.clone());
                let claims_performance = owner.as_deref() == Some("020")
                    && words.iter().any(|word| {
                        matches!(
                            word.as_str(),
                            "guarantee"
                                | "guarantees"
                                | "claim"
                                | "claims"
                                | "promise"
                                | "promises"
                        )
                    })
                    && !relation_is_negated(&words, performance_position, 1);
                if claims_performance {
                    return Some(clause.trim().to_owned());
                }
            }
        }
    }
    None
}

fn assert_no_cap020_overclaims(document_name: &str, document: &str) {
    assert!(
        cap020_product_violation(document).is_none(),
        "{document_name} promotes CAP-020 beyond its product-only boundary: {:?}",
        cap020_product_violation(document)
    );
}

fn cap021_product_violation(text: &str) -> Option<String> {
    const EXCLUDED: &[&[&str]] = &[
        &["parser"],
        &["grammar"],
        &["source", "semantics"],
        &["language", "profile"],
        &["compiler", "profile"],
        &["language", "feature"],
        &["feature", "row"],
        &["semantic", "analysis"],
        &["checked", "ir"],
        &["verifier"],
        &["backend"],
        &["compiler", "production"],
        &["production", "compiler"],
        &["production", "code"],
        &["production", "source", "code"],
        &["compiler", "edits"],
        &["compiler", "changes"],
        &["separate", "profile"],
        &["tensor", "type"],
        &["tensor", "types"],
        &["tensor", "syntax"],
        &["tensor", "support"],
        &["matrix", "type"],
        &["matrix", "types"],
        &["matrix", "syntax"],
        &["matrix", "support"],
        &["struct", "type"],
        &["struct", "types"],
        &["struct", "syntax"],
        &["record", "type"],
        &["record", "types"],
        &["record", "syntax"],
        &["record", "layout"],
        &["recursive", "arrays"],
        &["nested", "arrays"],
        &["serialization"],
        &["runtime", "ingestion"],
        &["runtime", "acquisition"],
        &["file", "input"],
        &["file", "acquisition"],
        &["external", "bytes"],
        &["bounded", "owned", "buffer"],
        &["i", "o"],
        &["runtime", "abi"],
        &["allocation"],
        &["drop"],
        &["quantization"],
        &["quantized", "arithmetic"],
        &["activation"],
        &["division"],
        &["checked", "overflow"],
        &["stable", "layout"],
        &["stable", "abi"],
        &["abi", "stability"],
        &["performance", "claim"],
        &["performance", "guarantee"],
        &["accelerator", "execution"],
        &["accelerator", "support"],
        &["memory", "safety"],
        &["safety", "claim"],
        &["general", "inference"],
        &["inference"],
        &["inference", "completion"],
        &["general", "mutation"],
        &["language", "completion"],
        &["stability", "claim"],
    ];
    for record in normalized_claim_records(text) {
        let mut carried_owner = None;
        for clause in record.split(['.', ';', '!', '?']) {
            let words = semantic_words(clause);
            let mentions = capability_mentions(&words);
            if let Some((_, _, owner)) = mentions.last() {
                carried_owner = Some(owner.clone());
            }
            let names_successor_without_cap = mentions.is_empty()
                && (((contains_semantic_phrase(&words, &["rank", "1"])
                    || contains_semantic_phrase(&words, &["rank", "2"])
                    || contains_semantic_phrase(&words, &["rank", "3"])
                    || contains_semantic_phrase(&words, &["runtime", "acquisition"])
                    || words
                        .iter()
                        .any(|word| matches!(word.as_str(), "quantized" | "quantization"))
                    || contains_semantic_phrase(&words, &["recursive", "arrays"]))
                    && words.iter().any(|word| {
                        matches!(word.as_str(), "readiness" | "probe" | "deferred" | "future")
                    }))
                    || ((contains_semantic_phrase(&words, &["task", "local"])
                        || contains_semantic_phrase(&words, &["architecture", "map"])
                        || contains_semantic_phrase(&words, &["mandatory", "stop"]))
                        && words.iter().any(|word| {
                            matches!(word.as_str(), "readiness" | "probe" | "deferred" | "stop")
                        })));
            let fallback_owner = (!names_successor_without_cap)
                .then(|| carried_owner.clone())
                .flatten();
            let leading_stop_condition = words.first().is_some_and(|word| word == "stop")
                && words.iter().any(|word| word == "if");
            for phrase in EXCLUDED {
                for position in words
                    .windows(phrase.len())
                    .enumerate()
                    .filter(|(_, candidate)| {
                        candidate
                            .iter()
                            .map(String::as_str)
                            .eq(phrase.iter().copied())
                    })
                    .map(|(position, _)| position)
                {
                    let subject_context = &words[position.saturating_sub(12)
                        ..(position + phrase.len() + 8).min(words.len())];
                    let successor_readiness_subject = subject_context.iter().any(|word| {
                        matches!(
                            word.as_str(),
                            "readiness" | "probe" | "deferred" | "ranking" | "ranks" | "order"
                        )
                    });
                    if successor_readiness_subject
                        && !has_affirmative_relation(&words, position, phrase.len())
                    {
                        continue;
                    }
                    let owner = nearest_capability_owner(&words, position, phrase.len())
                        .or_else(|| fallback_owner.clone());
                    if owner.as_deref() == Some("021")
                        && !leading_stop_condition
                        && !relation_is_negated(&words, position, phrase.len())
                    {
                        return Some(format!("{} :: {}", phrase.join(" "), clause.trim()));
                    }
                }
            }
        }
    }
    None
}

fn cap023_product_violation(text: &str) -> Option<String> {
    const EXCLUDED: &[&[&str]] = &[
        &["changes", "parser"],
        &["new", "parser"],
        &["parser", "capability"],
        &["parser", "changes"],
        &["parser", "support"],
        &["changes", "grammar"],
        &["grammar", "capability"],
        &["grammar", "changes"],
        &["grammar", "support"],
        &["source", "semantics"],
        &["language", "profile"],
        &["compiler", "profile"],
        &["language", "feature"],
        &["feature", "row"],
        &["selected", "profile", "row"],
        &["new", "profile"],
        &["profile", "support"],
        &["profile", "supported"],
        &["profile", "is", "supported"],
        &["semantic", "analysis", "changes"],
        &["semantic", "analysis", "support"],
        &["checked", "ir", "changes"],
        &["checked", "ir", "support"],
        &["verifier", "changes"],
        &["verifier", "support"],
        &["backend", "capability"],
        &["backend", "changes"],
        &["backend", "support"],
        &["changes", "backend"],
        &["abi", "capability"],
        &["abi", "changes"],
        &["abi", "support"],
        &["capability", "classification"],
        &["compiler", "production"],
        &["production", "compiler"],
        &["production", "code"],
        &["production", "source", "code"],
        &["compiler", "edits"],
        &["compiler", "changes"],
        &["separate", "profile"],
        &["general", "activation"],
        &["activation", "support"],
        &["activation", "capability"],
        &["general", "relu"],
        &["relu", "support"],
        &["relu", "capability"],
        &["general", "argmax"],
        &["argmax", "support"],
        &["argmax", "capability"],
        &["general", "inference"],
        &["inference", "support"],
        &["inference", "capability"],
        &["inference", "completion"],
        &["tensor", "type"],
        &["tensor", "syntax"],
        &["tensor", "support"],
        &["tensor", "operations"],
        &["matrix", "type"],
        &["matrix", "syntax"],
        &["matrix", "support"],
        &["record", "type"],
        &["record", "syntax"],
        &["record", "layout"],
        &["recursive", "array"],
        &["recursive", "arrays"],
        &["nested", "arrays"],
        &["serialization"],
        &["runtime", "ingestion"],
        &["runtime", "acquisition"],
        &["file", "input"],
        &["file", "acquisition"],
        &["external", "bytes"],
        &["bounded", "owned", "buffer"],
        &["runtime", "abi"],
        &["allocation"],
        &["drop"],
        &["quantization", "support"],
        &["quantization", "capability"],
        &["conversion", "semantics"],
        &["conversion", "support"],
        &["stable", "layout"],
        &["stable", "abi"],
        &["abi", "stability"],
        &["performance", "claim"],
        &["performance", "claims"],
        &["performance", "guarantee"],
        &["performance", "guarantees"],
        &["performance", "evidence"],
        &["resource", "usage", "claim"],
        &["resource", "usage", "claims"],
        &["resource", "usage", "evidence"],
        &["resource", "usage", "measurement"],
        &["accelerator", "execution"],
        &["accelerator", "support"],
        &["memory", "safety"],
        &["safety", "claim"],
        &["safety", "claims"],
        &["safety", "guarantee"],
        &["safety", "guarantees"],
        &["language", "completion"],
    ];
    let mut section_owner: Option<(usize, String)> = None;
    for record in normalized_claim_records(text) {
        if record == CAP023_EVIDENCE_PARAGRAPH || cap024_record_is_canonical(&record) {
            continue;
        }
        if let Some((level, owner)) = claim_heading(&record) {
            if let Some(owner) = owner {
                section_owner = Some((level, owner));
            } else if section_owner
                .as_ref()
                .is_some_and(|(owned_level, _)| level <= *owned_level)
            {
                section_owner = None;
            }
        }
        let mut carried_owner = section_owner.as_ref().map(|(_, owner)| owner.clone());
        for clause in record.split(['.', ';', '!', '?']) {
            let words = semantic_words(clause);
            let mentions = capability_mentions(&words);
            if let Some((_, _, owner)) = mentions.last() {
                carried_owner = Some(owner.clone());
            }
            let successor_without_owner = mentions.is_empty()
                && ((words.iter().any(|word| {
                    matches!(
                        word.as_str(),
                        "readiness" | "probe" | "deferred" | "ranking" | "rerank"
                    )
                }) && (contains_semantic_phrase(&words, &["rank", "2"])
                    || contains_semantic_phrase(&words, &["rank", "3"])
                    || words
                        .iter()
                        .any(|word| matches!(word.as_str(), "quantized" | "quantization"))))
                    || (words.first().is_some_and(|word| word == "stop")
                        && words.iter().any(|word| word == "if")));
            let fallback_owner = (!successor_without_owner)
                .then(|| carried_owner.clone())
                .flatten();
            let conditional_stop = words.first().is_some_and(|word| word == "stop")
                && words.iter().any(|word| word == "if");
            for phrase in EXCLUDED {
                for position in words
                    .windows(phrase.len())
                    .enumerate()
                    .filter(|(_, candidate)| {
                        candidate
                            .iter()
                            .map(String::as_str)
                            .eq(phrase.iter().copied())
                    })
                    .map(|(position, _)| position)
                {
                    let owner = nearest_capability_owner(&words, position, phrase.len())
                        .or_else(|| fallback_owner.clone());
                    let contrast_start = words[..position]
                        .iter()
                        .rposition(|word| {
                            matches!(word.as_str(), "but" | "yet" | "however" | "whereas")
                        })
                        .map_or(0, |position| position + 1);
                    let distributed_negative = words[contrast_start..position]
                        .iter()
                        .rposition(|word| matches!(word.as_str(), "no" | "without"))
                        .is_some_and(|negative| {
                            let negative = contrast_start + negative;
                            let double_negative = negative > contrast_start
                                && matches!(words[negative - 1].as_str(), "not" | "never")
                                || negative > contrast_start + 1
                                    && words[negative - 2] == "no"
                                    && words[negative - 1] == "longer";
                            !double_negative
                                && position - negative <= 24
                                && !words[negative + 1..position].iter().any(|word| {
                                    matches!(
                                        word.as_str(),
                                        "add"
                                            | "adds"
                                            | "admit"
                                            | "admits"
                                            | "create"
                                            | "creates"
                                            | "enable"
                                            | "enables"
                                            | "give"
                                            | "gives"
                                            | "guarantee"
                                            | "guarantees"
                                            | "has"
                                            | "implement"
                                            | "implements"
                                            | "is"
                                            | "offer"
                                            | "offers"
                                            | "provide"
                                            | "provides"
                                            | "support"
                                            | "supports"
                                    )
                                })
                        });
                    let direct_double_negative = position > 1
                        && matches!(words[position - 2].as_str(), "not" | "never")
                        && matches!(words[position - 1].as_str(), "without" | "no")
                        || position > 2
                            && words[position - 3] == "no"
                            && words[position - 2] == "longer"
                            && words[position - 1] == "without";
                    let direct_subject_denial = position > 0
                        && matches!(
                            words[position - 1].as_str(),
                            "not" | "no" | "without" | "never"
                        )
                        && !direct_double_negative;
                    if owner.as_deref() == Some("023")
                        && !conditional_stop
                        && !distributed_negative
                        && !direct_subject_denial
                        && !relation_is_negated(&words, position, phrase.len())
                    {
                        return Some(format!("{} :: {}", phrase.join(" "), clause.trim()));
                    }
                }
            }
            for subject in ["inference", "argmax", "relu"] {
                for position in words
                    .iter()
                    .enumerate()
                    .filter(|(_, word)| word.as_str() == subject)
                    .map(|(position, _)| position)
                {
                    let owner = nearest_capability_owner(&words, position, 1)
                        .or_else(|| fallback_owner.clone());
                    let bounded_product_description = words.iter().any(|word| word == "product")
                        && words.iter().any(|word| word == "evidence")
                        && words
                            .iter()
                            .any(|word| matches!(word.as_str(), "checkpoint" | "gate"))
                        && !words.iter().any(|word| {
                            matches!(
                                word.as_str(),
                                "general" | "support" | "supports" | "capability"
                            )
                        });
                    if owner.as_deref() == Some("023")
                        && !conditional_stop
                        && !bounded_product_description
                        && has_affirmative_relation(&words, position, 1)
                        && !relation_is_negated(&words, position, 1)
                    {
                        return Some(format!("{subject} :: {}", clause.trim()));
                    }
                }
            }
            let owns_cap023 =
                has_semantic_capability(&words, "023") || fallback_owner.as_deref() == Some("023");
            let class_promotion = words.iter().enumerate().any(|(position, word)| {
                matches!(word.as_str(), "stable" | "end_to_end")
                    && words[..position].iter().rev().take(5).any(|word| {
                        matches!(word.as_str(), "is" | "classified" | "becomes" | "became")
                    })
                    && !preceded_by_local_negation(&words, position)
            }) || contains_semantic_phrase(&words, &["end", "to", "end"])
                && words
                    .iter()
                    .any(|word| matches!(word.as_str(), "is" | "classified"));
            let widening = words.iter().enumerate().any(|(position, word)| {
                matches!(word.as_str(), "widen" | "widens" | "widened" | "widening")
                    && nearest_capability_owner(&words, position, 1).as_deref() == Some("023")
                    && words.iter().any(|word| {
                        matches!(
                            word.as_str(),
                            "compiler" | "profile" | "exact" | "i32" | "array"
                        )
                    })
            });
            if !conditional_stop && ((owns_cap023 && class_promotion) || widening) {
                return Some(clause.trim().to_owned());
            }
        }
    }
    None
}

fn signed_bracket_vectors(text: &str) -> Vec<(usize, Vec<i64>)> {
    let mut vectors = Vec::new();
    let mut offset = 0;
    while let Some((start, closer)) =
        text[offset..]
            .char_indices()
            .find_map(|(position, character)| match character {
                '[' => Some((offset + position, ']')),
                '(' => Some((offset + position, ')')),
                _ => None,
            })
    {
        let Some(end) = text[start + 1..]
            .find(closer)
            .map(|position| start + 1 + position)
        else {
            break;
        };
        let components = text[start + 1..end]
            .split(',')
            .map(str::trim)
            .map(str::parse::<i64>)
            .collect::<Result<Vec<_>, _>>();
        if let Ok(components) = components {
            if !components.is_empty() {
                vectors.push((start, components));
            }
        }
        offset = end + 1;
    }
    vectors
}

fn signed_integers(text: &str) -> Vec<i64> {
    let bytes = text.as_bytes();
    let mut values = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let signed = bytes[index] == b'-'
            && bytes
                .get(index + 1)
                .is_some_and(|next| next.is_ascii_digit());
        if !bytes[index].is_ascii_digit() && !signed {
            index += 1;
            continue;
        }
        let start = index;
        if signed {
            index += 1;
        }
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        let embedded = start
            .checked_sub(1)
            .and_then(|previous| bytes.get(previous))
            .is_some_and(u8::is_ascii_alphanumeric)
            || bytes.get(index).is_some_and(u8::is_ascii_alphanumeric);
        if !embedded {
            if let Ok(value) = text[start..index].parse::<i64>() {
                values.push(value);
            }
        }
    }
    values
}

fn vector_assertion_region_is_valid(
    clause: &str,
    start: usize,
    end: usize,
    vectors: &[(usize, Vec<i64>)],
    expected: &[i64],
) -> bool {
    let bracketed = vectors
        .iter()
        .filter(|(position, _)| (start..end).contains(position))
        .map(|(_, vector)| vector.as_slice())
        .collect::<Vec<_>>();
    if !bracketed.is_empty() {
        return bracketed.iter().all(|vector| *vector == expected);
    }
    let unbracketed = signed_integers(&clause[start..end]);
    unbracketed.len() < 2 || unbracketed == expected
}

fn scalar_assertion_values(
    words: &[String],
    keyword_position: usize,
    max_tokens: usize,
) -> Vec<i64> {
    let mut values = Vec::new();
    let suffix = &words[keyword_position + 1..];
    let mut index = 0;
    while index < suffix.len() && index < max_tokens {
        if matches!(
            suffix[index].as_str(),
            "after" | "because" | "through" | "while" | "under" | "with"
        ) {
            break;
        }
        let locally_negated = suffix[index.saturating_sub(3)..index]
            .iter()
            .any(|word| matches!(word.as_str(), "not" | "no" | "never"));
        if let Ok(value) = suffix[index].parse::<i64>() {
            if !locally_negated {
                values.push(value);
            }
        } else if suffix[index] == "ninety" {
            let ones = suffix.get(index + 1).map(String::as_str);
            let value = match ones {
                Some("one") => Some(91),
                Some("two") => Some(92),
                Some("three") => Some(93),
                _ => None,
            };
            if let Some(value) = value.filter(|_| !locally_negated) {
                values.push(value);
                index += 1;
            }
        }
        index += 1;
    }
    values
}

fn value_is_locally_negated(text: &str, value_position: usize) -> bool {
    let prefix = &text[..value_position];
    let fragment = prefix
        .rsplit([',', ';'])
        .next()
        .unwrap_or(prefix)
        .to_ascii_lowercase();
    let words = semantic_words(&fragment);
    words
        .iter()
        .rev()
        .take(4)
        .any(|word| matches!(word.as_str(), "not" | "no" | "never"))
        || words.windows(2).any(|pair| {
            pair[1] == "t"
                && matches!(
                    pair[0].as_str(),
                    "isn" | "doesn" | "wasn" | "aren" | "didn" | "can" | "won"
                )
        })
}

fn ascii_phrase_position(text: &str, phrase: &str) -> Option<usize> {
    let lower = text.to_ascii_lowercase();
    lower.match_indices(phrase).find_map(|(position, _)| {
        let before = position
            .checked_sub(1)
            .and_then(|index| lower.as_bytes().get(index));
        let after = lower.as_bytes().get(position + phrase.len());
        let bounded = before.is_none_or(|byte| !byte.is_ascii_alphanumeric())
            && after.is_none_or(|byte| !byte.is_ascii_alphanumeric());
        bounded.then_some(position)
    })
}

fn cap021_vector_region_is_valid(
    clause: &str,
    start: usize,
    end: usize,
    vectors: &[(usize, Vec<i64>)],
    expected: &[i64],
) -> bool {
    let bracketed = vectors
        .iter()
        .filter(|(position, _)| (start..end).contains(position))
        .map(|(_, vector)| vector.as_slice())
        .collect::<Vec<_>>();
    if !bracketed.is_empty() {
        return bracketed.iter().all(|vector| *vector == expected);
    }
    let integers = signed_integers(&clause[start..end]);
    integers.is_empty() || integers == expected
}

fn cap023_vector_region_is_valid(
    clause: &str,
    start: usize,
    end: usize,
    vectors: &[(usize, Vec<i64>)],
    expected: &[i64],
) -> bool {
    let bracketed = vectors
        .iter()
        .filter(|(position, _)| (start..end).contains(position))
        .map(|(_, vector)| vector.as_slice())
        .collect::<Vec<_>>();
    if !bracketed.is_empty() {
        return bracketed.iter().all(|vector| *vector == expected);
    }
    let integers = signed_integers(&clause[start..end]);
    integers.is_empty() || integers == expected
}

fn capability_byte_mentions(text: &str) -> Vec<(usize, usize, String)> {
    let lower = text.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut mentions = Vec::new();
    let mut index = 0;
    while index + 3 <= bytes.len() {
        if &bytes[index..index + 3] != b"cap"
            || index > 0 && bytes[index - 1].is_ascii_alphanumeric()
        {
            index += 1;
            continue;
        }
        let mut cursor = index + 3;
        if bytes.get(cursor) == Some(&b'-') {
            cursor += 1;
        } else {
            while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
                cursor += 1;
            }
        }
        let digits_start = cursor;
        while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
            cursor += 1;
        }
        if cursor > digits_start {
            mentions.push((index, cursor, lower[digits_start..cursor].to_owned()));
            index = cursor;
        } else {
            index += 1;
        }
    }
    mentions
}

fn cap021_owned_segments<'a>(clause: &'a str, carried_owner: Option<&str>) -> Vec<&'a str> {
    let mentions = capability_byte_mentions(clause);
    if mentions.is_empty() {
        return (carried_owner == Some("021"))
            .then_some(clause)
            .into_iter()
            .collect();
    }
    if mentions.len() == 1 {
        return (mentions[0].2 == "021")
            .then_some(clause)
            .into_iter()
            .collect();
    }
    mentions
        .iter()
        .enumerate()
        .filter(|(_, (_, _, owner))| owner == "021")
        .map(|(position, (start, _, _))| {
            let end = mentions
                .get(position + 1)
                .map_or(clause.len(), |(next, _, _)| *next);
            &clause[*start..end]
        })
        .collect()
}

fn cap021_oracle_violation(clause: &str) -> Option<String> {
    let words = semantic_words(clause);
    let lower = clause.to_ascii_lowercase();
    let vectors = signed_bracket_vectors(clause)
        .into_iter()
        .filter(|(position, _)| !value_is_locally_negated(clause, *position))
        .collect::<Vec<_>>();
    let ordinary = lower.find("ordinary");
    let wrapping = lower.find("wrapping");
    let invalid = lower.find("invalid").or_else(|| lower.find("malformed"));
    let qualitative_contradiction = words.iter().any(|word| {
        matches!(
            word.as_str(),
            "differ"
                | "differs"
                | "different"
                | "incorrect"
                | "wrong"
                | "mismatch"
                | "except"
                | "corrected"
                | "actually"
        )
    });
    if (ordinary.is_some() || wrapping.is_some()) && qualitative_contradiction {
        return Some(clause.trim().to_owned());
    }
    if invalid.is_some()
        && (words.iter().any(|word| word == "nonzero")
            || contains_semantic_phrase(&words, &["non", "zero"])
            || contains_semantic_phrase(&words, &["not", "zero"]))
    {
        return Some(clause.trim().to_owned());
    }
    if let Some(start) = ordinary {
        let end = [wrapping, invalid]
            .into_iter()
            .flatten()
            .filter(|end| *end > start)
            .min()
            .unwrap_or(clause.len());
        if !cap021_vector_region_is_valid(
            clause,
            start,
            end,
            &vectors,
            &[1, 122, 167, 135, 181, 4938],
        ) {
            return Some(clause.trim().to_owned());
        }
    }
    if let Some(start) = wrapping {
        let end = invalid.filter(|end| *end > start).unwrap_or(clause.len());
        if !cap021_vector_region_is_valid(
            clause,
            start,
            end,
            &vectors,
            &[1, -24, 18, 2147483623, -2147483631, -2147483627],
        ) {
            return Some(clause.trim().to_owned());
        }
    }
    if let Some(start) = invalid {
        let asserted = vectors
            .iter()
            .filter(|(position, _)| *position > start)
            .map(|(_, vector)| vector.as_slice())
            .collect::<Vec<_>>();
        if asserted.iter().any(|vector| *vector != [0, 0, 0, 0, 0, 0]) {
            return Some(clause.trim().to_owned());
        }
    }
    let returns_vector = words
        .iter()
        .any(|word| matches!(word.as_str(), "return" | "returns" | "result" | "results"))
        && !vectors.is_empty();
    if returns_vector
        && vectors.iter().any(|(_, vector)| {
            !matches!(
                vector.as_slice(),
                [1, 122, 167, 135, 181, 4938]
                    | [1, -24, 18, 2147483623, -2147483631, -2147483627]
                    | [0, 0, 0, 0, 0, 0]
                    | [2, 3, 1]
            )
        })
    {
        return Some(clause.trim().to_owned());
    }
    for keyword in ["exit", "exits", "sentinel", "terminate", "terminates"] {
        if let Some(position) = words.iter().position(|word| word == keyword) {
            if !preceded_by_local_negation(&words, position)
                && scalar_assertion_values(&words, position, 8)
                    .into_iter()
                    .any(|value| value != 91)
            {
                return Some(clause.trim().to_owned());
            }
        }
    }
    None
}

fn cap021_status_violation(text: &str) -> Option<String> {
    const NEGATABLE_STATUS: &[&[&str]] = &[
        &["remains", "a", "candidate"],
        &["a", "candidate"],
        &["candidate", "only"],
        &["pending", "acceptance"],
        &["acceptance", "is", "pending"],
        &["acceptance", "remains", "pending"],
        &["unaccepted"],
        &["proposed"],
        &["local", "only"],
        &["local", "candidate"],
        &["unpublished"],
        &["unmerged"],
        &["awaiting", "acceptance"],
        &["awaits", "acceptance"],
    ];
    const ABSOLUTE: &[&[&str]] = &[
        &["has", "not", "yet", "been", "accepted"],
        &["has", "not", "been", "accepted"],
        &["not", "yet", "accepted"],
        &["acceptance", "revoked"],
        &["acceptance", "reverted"],
        &["acceptance", "withdrawn"],
        &["acceptance", "rejected"],
        &["non", "zero", "production"],
        &["not", "zero", "production"],
        &["no", "longer", "zero", "production"],
    ];
    for record in normalized_claim_records(text) {
        if record == CAP021_EVIDENCE_PARAGRAPH {
            continue;
        }
        let mut carried_owner = None;
        for clause in record.split(['.', ';', '!', '?']) {
            let words = semantic_words(clause);
            if let Some((_, _, owner)) = capability_mentions(&words).last() {
                carried_owner = Some(owner.clone());
            }
            if !has_semantic_capability(&words, "021") && carried_owner.as_deref() != Some("021") {
                continue;
            }
            if let Some(phrase) = ABSOLUTE
                .iter()
                .find(|phrase| contains_semantic_phrase(&words, phrase))
            {
                return Some(phrase.join(" "));
            }
            for phrase in NEGATABLE_STATUS {
                if let Some(position) = semantic_phrase_position(&words, phrase) {
                    if !preceded_by_local_negation(&words, position) {
                        return Some(phrase.join(" "));
                    }
                }
            }
            for status in [
                "fail",
                "fails",
                "failed",
                "revoked",
                "reverted",
                "withdrawn",
            ] {
                if let Some(position) = words.iter().position(|word| word == status) {
                    let failed_program = matches!(status, "fail" | "fails" | "failed")
                        && words.get(position + 1).is_some_and(|word| {
                            matches!(word.as_str(), "program" | "programs" | "case" | "cases")
                        });
                    let acceptance_context = position <= 3
                        || words.iter().any(|word| {
                            matches!(
                                word.as_str(),
                                "ci" | "check" | "checks" | "merge" | "acceptance" | "evidence"
                            )
                        });
                    if !failed_program
                        && acceptance_context
                        && !preceded_by_local_negation(&words, position)
                    {
                        return Some(clause.trim().to_owned());
                    }
                }
            }
            if let Some(position) = words.iter().position(|word| word == "new") {
                if words[position..].iter().any(|word| word == "alert")
                    && !preceded_by_local_negation(&words, position)
                {
                    return Some(clause.trim().to_owned());
                }
            }

            for owned in cap021_owned_segments(clause, carried_owner.as_deref()) {
                if let Some(violation) = cap021_oracle_violation(owned) {
                    return Some(violation);
                }
            }
        }
    }
    None
}

fn capability_owned_segments<'a>(
    clause: &'a str,
    carried_owner: Option<&str>,
    capability: &str,
) -> Vec<&'a str> {
    let mentions = capability_byte_mentions(clause);
    if mentions.is_empty() {
        return (carried_owner == Some(capability))
            .then_some(clause)
            .into_iter()
            .collect();
    }
    if mentions.len() == 1 {
        return (mentions[0].2 == capability)
            .then_some(clause)
            .into_iter()
            .collect();
    }
    mentions
        .iter()
        .enumerate()
        .filter(|(_, (_, _, owner))| owner == capability)
        .map(|(position, (start, _, _))| {
            let prefix_start = position
                .checked_sub(1)
                .and_then(|previous| mentions.get(previous).map(|(_, end, _)| *end))
                .unwrap_or(0);
            let backward_owned = semantic_words(&clause[prefix_start..*start])
                .last()
                .is_some_and(|word| matches!(word.as_str(), "for" | "by" | "under" | "to"));
            let start = if backward_owned { prefix_start } else { *start };
            let end = mentions
                .get(position + 1)
                .map_or(clause.len(), |(next, _, _)| *next);
            &clause[start..end]
        })
        .collect()
}

fn cap023_oracle_violation(clause: &str) -> Option<String> {
    const ORDINARY: &[i64] = &[1, 122, 167, 135, 181, 4940, 5573, 1];
    const WRAPPING: &[i64] = &[1, -24, 18, 2147483623, 0, -37, 2147483641, 1];
    const ACTIVATION: &[i64] = &[1, -3, 0, 0, 0, 5, 4, 0];
    const TIE: &[i64] = &[1, 1, 2, 1, 2, 3, 3, 0];
    const MALFORMED: &[i64] = &[0, 0, 0, 0, 0, 0, 0, 0];
    const HEADER: &[i64] = &[2, 3, 2];

    let words = semantic_words(clause);
    let lower = clause.to_ascii_lowercase();
    let vectors = signed_bracket_vectors(clause)
        .into_iter()
        .filter(|(position, _)| !value_is_locally_negated(clause, *position))
        .collect::<Vec<_>>();
    let label_has_outcome = |position: usize| {
        [
            "oracle", "oracles", "output", "outputs", "result", "results", "return", "returns",
        ]
        .into_iter()
        .any(|keyword| {
            let is_dimension_word = |candidate: usize| {
                matches!(keyword, "output" | "outputs")
                    && candidate > 0
                    && lower.as_bytes()[candidate - 1] == b'-'
            };
            lower[position..].find(keyword).is_some_and(|distance| {
                let candidate = position + distance;
                distance <= 160 && !is_dimension_word(candidate)
            }) || lower[..position].rfind(keyword).is_some_and(|candidate| {
                position - candidate <= 160 && !is_dimension_word(candidate)
            })
        })
    };
    let ordinary =
        ascii_phrase_position(clause, "ordinary").filter(|position| label_has_outcome(*position));
    let wrapping =
        ascii_phrase_position(clause, "wrapping").filter(|position| label_has_outcome(*position));
    let activation = ascii_phrase_position(clause, "activation-boundary")
        .or_else(|| ascii_phrase_position(clause, "activation boundary"))
        .filter(|position| label_has_outcome(*position));
    let tie = ascii_phrase_position(clause, "tie").filter(|position| label_has_outcome(*position));
    let malformed = ascii_phrase_position(clause, "malformed")
        .or_else(|| ascii_phrase_position(clause, "invalid"))
        .filter(|position| label_has_outcome(*position));
    let qualitative_contradiction =
        words.iter().any(|word| {
            matches!(
                word.as_str(),
                "differ"
                    | "differs"
                    | "different"
                    | "incorrect"
                    | "wrong"
                    | "mismatch"
                    | "except"
                    | "corrected"
                    | "actually"
                    | "unavailable"
                    | "missing"
            )
        }) || contains_semantic_phrase(&words, &["no", "longer", "equals"])
            || contains_semantic_phrase(&words, &["has", "no", "ordinary", "oracle"])
            || contains_semantic_phrase(&words, &["has", "no", "wrapping", "oracle"]);
    if [ordinary, wrapping, activation, tie, malformed]
        .into_iter()
        .any(|position| position.is_some())
        && qualitative_contradiction
    {
        return Some(clause.trim().to_owned());
    }
    if malformed.is_some()
        && (words.iter().any(|word| word == "nonzero")
            || contains_semantic_phrase(&words, &["non", "zero"])
            || contains_semantic_phrase(&words, &["not", "zero"]))
    {
        return Some(clause.trim().to_owned());
    }

    if let Some(respectively) = ascii_phrase_position(clause, "respectively") {
        let mut labels = [
            (ordinary, ORDINARY),
            (wrapping, WRAPPING),
            (activation, ACTIVATION),
            (tie, TIE),
            (malformed, MALFORMED),
        ]
        .into_iter()
        .filter_map(|(position, expected)| position.map(|position| (position, expected)))
        .collect::<Vec<_>>();
        labels.sort_by_key(|(position, _)| *position);
        let first_label = labels
            .first()
            .map_or(respectively, |(position, _)| *position);
        let ordered = vectors
            .iter()
            .filter(|(position, _)| *position > first_label)
            .map(|(_, vector)| vector.as_slice())
            .collect::<Vec<_>>();
        if ordered.len() != labels.len()
            || labels
                .iter()
                .zip(&ordered)
                .any(|((_, expected), actual)| *expected != *actual)
        {
            return Some(clause.trim().to_owned());
        }
    } else {
        for (start, expected) in [
            (ordinary, ORDINARY),
            (wrapping, WRAPPING),
            (activation, ACTIVATION),
            (tie, TIE),
            (malformed, MALFORMED),
        ] {
            let Some(start) = start else {
                continue;
            };
            let end = [ordinary, wrapping, activation, tie, malformed]
                .into_iter()
                .flatten()
                .filter(|candidate| *candidate > start)
                .min()
                .unwrap_or(clause.len());
            if !cap023_vector_region_is_valid(clause, start, end, &vectors, expected) {
                return Some(clause.trim().to_owned());
            }
            let has_forward_vector = vectors
                .iter()
                .any(|(position, _)| (start..end).contains(position));
            if !has_forward_vector {
                if expected == MALFORMED && contains_semantic_phrase(&words, &["eight", "zeros"]) {
                    continue;
                }
                if let Some((_, nearest)) = vectors
                    .iter()
                    .filter(|(position, _)| position.abs_diff(start) <= 200)
                    .min_by_key(|(position, _)| position.abs_diff(start))
                {
                    if nearest.as_slice() != expected {
                        return Some(clause.trim().to_owned());
                    }
                } else {
                    let assertion = words.iter().any(|word| {
                        matches!(
                            word.as_str(),
                            "is" | "are" | "equals" | "equal" | "unavailable" | "missing"
                        )
                    }) || words.iter().any(|word| word == "no");
                    if assertion {
                        return Some(clause.trim().to_owned());
                    }
                }
            }
        }
    }

    if let Some(start) = ascii_phrase_position(clause, "header") {
        let application_header = !words
            .iter()
            .any(|word| matches!(word.as_str(), "ranking" | "table" | "column" | "columns"));
        let end = [ordinary, wrapping, activation, tie, malformed]
            .into_iter()
            .flatten()
            .filter(|position| *position > start)
            .min()
            .unwrap_or(clause.len());
        if application_header
            && !cap023_vector_region_is_valid(clause, start, end, &vectors, HEADER)
        {
            return Some(clause.trim().to_owned());
        }
        if application_header && clause[start..end].to_ascii_lowercase().contains("0x") {
            return Some(clause.trim().to_owned());
        }
    }
    if let Some(start) = malformed {
        if vectors
            .iter()
            .filter(|(position, _)| *position > start)
            .any(|(_, vector)| vector.as_slice() != MALFORMED)
        {
            return Some(clause.trim().to_owned());
        }
    }
    let returns_vector = words.iter().any(|word| {
        matches!(
            word.as_str(),
            "return" | "returns" | "result" | "results" | "oracle" | "oracles"
        )
    }) && vectors.iter().any(|(_, vector)| vector.len() > 1);
    if returns_vector
        && vectors
            .iter()
            .filter(|(_, vector)| vector.len() > 1)
            .any(|(_, vector)| {
                ![ORDINARY, WRAPPING, ACTIVATION, TIE, MALFORMED, HEADER]
                    .contains(&vector.as_slice())
            })
    {
        return Some(clause.trim().to_owned());
    }
    if [ordinary, wrapping, activation, tie, malformed]
        .into_iter()
        .flatten()
        .any(|start| lower[start..].contains("0x"))
    {
        return Some(clause.trim().to_owned());
    }
    for keyword in ["exit", "exits", "sentinel", "terminate", "terminates"] {
        if let Some(position) = words.iter().position(|word| word == keyword) {
            if !preceded_by_local_negation(&words, position)
                && scalar_assertion_values(&words, position, 8)
                    .into_iter()
                    .any(|value| value != 91)
            {
                return Some(clause.trim().to_owned());
            }
        }
    }
    None
}

fn cap023_status_violation(text: &str) -> Option<String> {
    const NEGATABLE_STATUS: &[&[&str]] = &[
        &["remains", "a", "candidate"],
        &["a", "candidate"],
        &["candidate", "only"],
        &["pending", "acceptance"],
        &["acceptance", "is", "pending"],
        &["acceptance", "remains", "pending"],
        &["unaccepted"],
        &["proposed"],
        &["local", "only"],
        &["local", "candidate"],
        &["unpublished"],
        &["unmerged"],
        &["awaiting", "acceptance"],
        &["awaits", "acceptance"],
    ];
    const ABSOLUTE: &[&[&str]] = &[
        &["has", "not", "yet", "been", "accepted"],
        &["has", "not", "been", "accepted"],
        &["not", "yet", "accepted"],
        &["acceptance", "revoked"],
        &["acceptance", "reverted"],
        &["acceptance", "withdrawn"],
        &["acceptance", "rejected"],
        &["non", "zero", "production"],
        &["not", "zero", "production"],
        &["no", "longer", "zero", "production"],
        &["not", "merged"],
        &["remains", "a", "draft"],
        &["is", "a", "draft"],
        &["no", "longer", "the", "current", "public", "master"],
        &["has", "been", "superseded"],
        &["remains", "pending", "review"],
    ];
    let mut section_owner: Option<(usize, String)> = None;
    for record in normalized_claim_records(text) {
        if record == CAP023_EVIDENCE_PARAGRAPH || cap024_record_is_canonical(&record) {
            continue;
        }
        if let Some((level, owner)) = claim_heading(&record) {
            if let Some(owner) = owner {
                section_owner = Some((level, owner));
            } else if section_owner
                .as_ref()
                .is_some_and(|(owned_level, _)| level <= *owned_level)
            {
                section_owner = None;
            }
        }
        let mut carried_owner = section_owner.as_ref().map(|(_, owner)| owner.clone());
        for clause in record.split(['.', ';', '!', '?']) {
            let words = semantic_words(clause);
            let explicit_owner = capability_mentions(&words)
                .last()
                .map(|(_, _, owner)| owner.clone());
            if let Some(owner) = &explicit_owner {
                carried_owner = Some(owner.clone());
            }
            if !has_semantic_capability(&words, "023") && carried_owner.as_deref() != Some("023") {
                continue;
            }
            let conditional_stop = words.first().is_some_and(|word| word == "stop")
                && words.iter().any(|word| word == "if");
            let historical_status = words.iter().any(|word| {
                matches!(
                    word.as_str(),
                    "before" | "prior" | "previously" | "formerly"
                )
            }) && words.iter().any(|word| {
                matches!(
                    word.as_str(),
                    "acceptance" | "accepted" | "candidate" | "unpublished" | "pr"
                )
            }) && !words.iter().any(|word| {
                matches!(
                    word.as_str(),
                    "remains" | "still" | "current" | "today" | "now"
                )
            });
            if historical_status {
                continue;
            }
            if !conditional_stop {
                if let Some(phrase) = ABSOLUTE
                    .iter()
                    .find(|phrase| contains_semantic_phrase(&words, phrase))
                {
                    return Some(phrase.join(" "));
                }
                for phrase in NEGATABLE_STATUS {
                    if let Some(position) = semantic_phrase_position(&words, phrase) {
                        if !preceded_by_local_negation(&words, position) {
                            return Some(phrase.join(" "));
                        }
                    }
                }
                for status in [
                    "fail",
                    "fails",
                    "failed",
                    "revoked",
                    "reverted",
                    "withdrawn",
                    "rejected",
                    "broken",
                    "error",
                    "pending",
                ] {
                    if let Some(position) = words.iter().position(|word| word == status) {
                        let failed_program = matches!(status, "fail" | "fails" | "failed")
                            && words.get(position + 1).is_some_and(|word| {
                                matches!(word.as_str(), "program" | "programs" | "case" | "cases")
                            });
                        let language_error_subject = status == "error"
                            && words[position + 1..(position + 4).min(words.len())]
                                .iter()
                                .any(|word| {
                                    matches!(
                                        word.as_str(),
                                        "propagation" | "handling" | "mapping" | "type" | "types"
                                    )
                                });
                        let acceptance_context = position <= 3
                            || words.iter().any(|word| {
                                matches!(
                                    word.as_str(),
                                    "ci" | "check" | "checks" | "merge" | "acceptance" | "evidence"
                                )
                            });
                        if !failed_program
                            && !language_error_subject
                            && acceptance_context
                            && !preceded_by_local_negation(&words, position)
                        {
                            return Some(clause.trim().to_owned());
                        }
                    }
                }
                if let Some(position) = words.iter().position(|word| word == "new") {
                    if words[position..].iter().any(|word| word == "alert")
                        && !preceded_by_local_negation(&words, position)
                    {
                        return Some(clause.trim().to_owned());
                    }
                }
                for alert in words
                    .iter()
                    .enumerate()
                    .filter(|(_, word)| word.as_str() == "alert")
                    .map(|(position, _)| position)
                {
                    let open = words[alert.saturating_sub(4)..(alert + 8).min(words.len())]
                        .iter()
                        .position(|word| word == "open")
                        .map(|position| alert.saturating_sub(4) + position);
                    let Some(open) = open else {
                        continue;
                    };
                    let alert_number = words[alert + 1..(alert + 5).min(words.len())]
                        .iter()
                        .find_map(|word| word.parse::<u64>().ok());
                    let preexisting_four = alert_number == Some(4)
                        && (contains_semantic_phrase(
                            &words[alert.saturating_sub(4)..(alert + 5).min(words.len())],
                            &["pre", "existing"],
                        ) || words[alert.saturating_sub(4)..alert]
                            .iter()
                            .any(|word| word == "preexisting"));
                    if !preexisting_four && !preceded_by_local_negation(&words, open) {
                        return Some(clause.trim().to_owned());
                    }
                }
                let mentions_alert = words
                    .iter()
                    .any(|word| matches!(word.as_str(), "alert" | "alerts"));
                let alert_four = words.iter().any(|word| word == "4");
                if mentions_alert
                    && ((alert_four
                        && words
                            .iter()
                            .any(|word| matches!(word.as_str(), "closed" | "resolved")))
                        || contains_semantic_phrase(&words, &["zero", "open", "alerts"]))
                {
                    return Some(clause.trim().to_owned());
                }
                let analysis_subject = words
                    .iter()
                    .any(|word| matches!(word.as_str(), "python" | "rust"))
                    && words
                        .iter()
                        .any(|word| matches!(word.as_str(), "analysis" | "analyses"));
                let nonzero_analysis = words.iter().any(|word| word == "nonzero")
                    || contains_semantic_phrase(&words, &["non", "zero"])
                    || (words.iter().any(|word| word == "one")
                        && words.iter().any(|word| {
                            matches!(word.as_str(), "result" | "results" | "finding" | "findings")
                        }))
                    || words.iter().enumerate().any(|(position, word)| {
                        word.parse::<i64>().is_ok_and(|value| value != 0)
                            && words[..position].iter().rev().take(5).any(|word| {
                                matches!(
                                    word.as_str(),
                                    "result" | "results" | "finding" | "findings"
                                )
                            })
                    });
                if analysis_subject && nonzero_analysis {
                    return Some(clause.trim().to_owned());
                }
                if words.iter().any(|word| word == "aggregate")
                    && words.iter().any(|word| word == "exists")
                    && contains_semantic_phrase(&words, &["default", "branch"])
                {
                    return Some(clause.trim().to_owned());
                }
                if words
                    .iter()
                    .any(|word| matches!(word.as_str(), "stdout" | "stderr"))
                    && (words.iter().any(|word| word == "nonempty")
                        || words.iter().any(|word| word == "diagnostics"))
                {
                    return Some(format!("stream contradiction: {}", clause.trim()));
                }
                if let Some(lanes) = words.iter().position(|word| word == "lanes") {
                    let asserted = words[..lanes]
                        .iter()
                        .rev()
                        .find_map(|word| word.parse::<u64>().ok());
                    if asserted.is_some_and(|value| value != 140) {
                        return Some(format!("source-lane contradiction: {}", clause.trim()));
                    }
                }
                if let Some(calls) = words.iter().position(|word| word == "calls") {
                    let asserted =
                        words[calls.saturating_sub(6)..calls]
                            .iter()
                            .rev()
                            .find_map(|word| match word.as_str() {
                                "six" => Some(6_u64),
                                "seven" => Some(7_u64),
                                _ => word.parse::<u64>().ok(),
                            });
                    if asserted.is_some_and(|value| value != 7) {
                        return Some(format!("call-count contradiction: {}", clause.trim()));
                    }
                }
            }
            if conditional_stop {
                continue;
            }
            for owned in capability_owned_segments(clause, carried_owner.as_deref(), "023") {
                if let Some(violation) = cap023_oracle_violation(owned) {
                    return Some(violation);
                }
            }
        }
    }
    None
}

fn cap023_milestone_violation(text: &str) -> Option<String> {
    let milestone_number = |words: &[String], position: usize| -> Option<u8> {
        if let Some(number) = words[position].strip_prefix('m') {
            if !number.is_empty() && number.chars().all(|character| character.is_ascii_digit()) {
                return number.parse().ok();
            }
        }
        if words[position] != "milestone" {
            return None;
        }
        words
            .get(position + 1)
            .and_then(|word| match word.as_str() {
                "zero" => Some(0),
                "one" => Some(1),
                "two" => Some(2),
                "three" => Some(3),
                _ => word.parse().ok(),
            })
    };
    let mut section: Option<(usize, Option<u8>)> = None;
    for record in normalized_claim_records(text) {
        if let Some((level, _)) = claim_heading(&record) {
            let heading_words = semantic_words(&record);
            let marker = heading_words
                .iter()
                .enumerate()
                .find_map(|(position, word)| {
                    (matches!(word.as_str(), "milestone" | "milestones")
                        || word.strip_prefix('m').is_some_and(|number| {
                            !number.is_empty()
                                && number.chars().all(|character| character.is_ascii_digit())
                        }))
                    .then_some(position)
                });
            if let Some(position) = marker {
                section = Some((level, milestone_number(&heading_words, position)));
            } else if section.is_some_and(|(owned_level, _)| level <= owned_level) {
                section = None;
            }
        }
        for clause in record.split(['.', ';', '!', '?']) {
            let words = semantic_words(clause);
            let milestone_positions = words
                .iter()
                .enumerate()
                .filter(|(position, word)| {
                    word.as_str() == "milestone"
                        || word.strip_prefix('m').is_some_and(|number| {
                            !number.is_empty()
                                && number.chars().all(|ch| ch.is_ascii_digit())
                                && !position
                                    .checked_sub(1)
                                    .and_then(|previous| words.get(previous))
                                    .is_some_and(|previous| previous == "post")
                        })
                })
                .map(|(position, _)| position)
                .collect::<Vec<_>>();
            for (completion, word) in words.iter().enumerate() {
                let relation_prefix = &words[completion.saturating_sub(6)..completion];
                let contrastively_denied = relation_prefix.windows(2).any(|pair| {
                    matches!(pair, [left, right] if (left == "rather" && right == "than")
                        || (left == "instead" && right == "of")
                        || (left == "without"
                            && matches!(right.as_str(), "making" | "claiming" | "calling" | "declaring" | "treating")))
                });
                if !matches!(
                    word.as_str(),
                    "achieve"
                        | "achieved"
                        | "achieves"
                        | "complete"
                        | "completed"
                        | "completes"
                        | "closed"
                        | "deliver"
                        | "delivered"
                        | "delivers"
                        | "done"
                        | "finished"
                        | "fulfill"
                        | "fulfilled"
                        | "fulfills"
                        | "conclude"
                        | "concluded"
                        | "concludes"
                        | "accomplished"
                        | "meet"
                        | "meets"
                        | "met"
                        | "ready"
                        | "satisfied"
                        | "ship"
                        | "shipped"
                        | "ships"
                ) || preceded_by_local_negation(&words, completion)
                    || contrastively_denied
                {
                    continue;
                }
                let milestone = milestone_positions
                    .iter()
                    .min_by_key(|position| position.abs_diff(completion))
                    .copied()
                    .filter(|position| position.abs_diff(completion) <= 10);
                let section_subject = milestone.is_none()
                    && section.is_some()
                    && (words.first().is_some_and(|word| {
                        matches!(word.as_str(), "it" | "this" | "that" | "all")
                    }) || words
                        .iter()
                        .any(|word| matches!(word.as_str(), "ambition" | "ambitions" | "broader")));
                if milestone.is_none() && !section_subject {
                    continue;
                }
                let marker = milestone.unwrap_or(completion);
                let range_start = marker.saturating_sub(4);
                let range = if marker < completion {
                    &words[range_start..=completion]
                } else {
                    &words[completion..=marker]
                };
                let conditional_stop = words[..completion].iter().any(|word| word == "stop")
                    && words[..completion].iter().any(|word| word == "if");
                if conditional_stop {
                    continue;
                }
                let audit = words[..=completion]
                    .iter()
                    .rposition(|word| word == "audit");
                let later_numbered_milestone = audit.is_some_and(|audit| {
                    milestone_positions.iter().any(|position| {
                        *position > audit
                            && *position <= completion
                            && milestone_number(&words, *position).is_some()
                    })
                });
                let audit_noun = audit.is_some()
                    && words[..=completion].iter().any(|word| word == "gap")
                    && !later_numbered_milestone;
                if audit_noun {
                    continue;
                }
                let number = milestone
                    .and_then(|position| milestone_number(&words, position))
                    .or_else(|| section.and_then(|(_, number)| number));
                let milestone_three = number == Some(3);
                let selected_exit = range.iter().any(|word| {
                    matches!(
                        word.as_str(),
                        "bounded" | "selected" | "exit" | "exits" | "gate" | "gates"
                    )
                });
                if milestone_three || !selected_exit {
                    return Some(clause.trim().to_owned());
                }
            }
        }
    }
    None
}

fn cap024_record_is_canonical(record: &str) -> bool {
    let normalized = normalized_words(record);
    [
        CAP024_EVIDENCE_PARAGRAPH,
        CAP024_CURRENT_HEAD_BOUNDARY,
        CAP024_ZERO_PRODUCTION_BOUNDARY,
        CAP024_CLASSIFICATION_BOUNDARY,
        CAP024_BUNDLE_BOUNDARY,
        CAP024_ALERT_BOUNDARY,
        CAP024_MILESTONE_BOUNDARY,
    ]
    .iter()
    .any(|canonical| normalized == *canonical)
        || POST_CAP024_DECISION_CONTRACTS
            .iter()
            .any(|canonical| normalized == *canonical)
}

fn cap024_product_violation(text: &str) -> Option<String> {
    const EXCLUDED: &[&[&str]] = &[
        &["compiler", "production"],
        &["compiler"],
        &["compiler", "edits"],
        &["code", "generation"],
        &["codegen"],
        &["production"],
        &["parser"],
        &["grammar"],
        &["source", "semantics"],
        &["source", "code"],
        &["language", "profile"],
        &["selected", "profile"],
        &["profile"],
        &["semantic", "analysis"],
        &["checked", "ir"],
        &["verifier"],
        &["backend"],
        &["example"],
        &["product", "oracle"],
        &["runtime", "behavior"],
        &["runtime", "ingestion"],
        &["runtime", "acquisition"],
        &["runtime", "byte", "file", "acquisition"],
        &["file", "input"],
        &["file", "ingestion"],
        &["abi"],
        &["capability", "classification"],
        &["classification"],
        &["language", "feature"],
        &["benchmark"],
        &["resource", "usage"],
        &["performance"],
        &["accelerator"],
        &["safety"],
        &["general", "inference"],
        &["activation"],
        &["relu"],
        &["argmax"],
        &["tensor"],
        &["matrix"],
        &["record"],
        &["recursive", "array"],
        &["recursive", "arrays"],
        &["serialization"],
        &["quantization"],
        &["conversion"],
        &["product", "capability"],
        &["product", "checkpoint"],
        &["compiler", "widening"],
        &["profile", "widening"],
    ];
    let mut section_owner: Option<(usize, String)> = None;
    for record in normalized_claim_records(text) {
        if cap024_record_is_canonical(&record) {
            continue;
        }
        if let Some((level, owner)) = claim_heading(&record) {
            if let Some(owner) = owner {
                section_owner = Some((level, owner));
            } else if section_owner
                .as_ref()
                .is_some_and(|(owned_level, _)| level <= *owned_level)
            {
                section_owner = None;
            }
        }
        let mut carried_owner = section_owner.as_ref().map(|(_, owner)| owner.clone());
        for clause in record.split(['.', ';', '!', '?']) {
            let words = semantic_words(clause);
            if let Some((_, _, owner)) = capability_mentions(&words).last() {
                carried_owner = Some(owner.clone());
            }
            if !has_semantic_capability(&words, "024") && carried_owner.as_deref() != Some("024") {
                continue;
            }
            let conditional = words.first().is_some_and(|word| word == "stop");
            if conditional {
                continue;
            }
            let standalone_classification = words
                .iter()
                .position(|word| {
                    matches!(
                        word.as_str(),
                        "stable" | "partial" | "experimental" | "designed"
                    )
                })
                .or_else(|| semantic_phrase_position(&words, &["end", "to", "end"]))
                .or_else(|| semantic_phrase_position(&words, &["parsed", "only"]));
            if let Some(position) = standalone_classification {
                let classification_context = words.iter().any(|word| {
                    matches!(
                        word.as_str(),
                        "class" | "classified" | "classification" | "row"
                    )
                });
                let direct_cap024_classification = capability_mentions(&words)
                    .iter()
                    .filter(|(_, end, owner)| owner == "024" && *end <= position)
                    .any(|(_, end, _)| {
                        words.get(*end).is_some_and(|word| {
                            matches!(word.as_str(), "is" | "remains" | "becomes")
                        }) && words[*end + 1..position].iter().all(|word| {
                            matches!(
                                word.as_str(),
                                "a" | "an" | "as" | "classified" | "currently" | "now" | "still"
                            )
                        })
                    });
                if !preceded_by_local_negation(&words, position)
                    && (classification_context || direct_cap024_classification)
                {
                    return Some(clause.trim().to_owned());
                }
            }
            for subject in EXCLUDED {
                let Some(position) = semantic_phrase_position(&words, subject) else {
                    continue;
                };
                let zero_production = subject.len() == 1
                    && subject[0] == "production"
                    && words
                        .get(position.wrapping_sub(1))
                        .is_some_and(|word| word == "zero");
                let reversed_zero_production = zero_production
                    && (words
                        .get(position.wrapping_sub(2))
                        .is_some_and(|word| matches!(word.as_str(), "non" | "not"))
                        || position >= 3
                            && words[position - 3..position]
                                .iter()
                                .map(String::as_str)
                                .eq(["no", "longer", "zero"]));
                if reversed_zero_production {
                    return Some(format!(
                        "CAP-024 contradicts zero-production status: {}",
                        clause.trim()
                    ));
                }
                if zero_production {
                    continue;
                }
                let extra_positive = words.iter().enumerate().any(|(verb_position, word)| {
                    matches!(
                        word.as_str(),
                        "change"
                            | "changes"
                            | "changed"
                            | "edit"
                            | "edits"
                            | "edited"
                            | "modify"
                            | "modifies"
                            | "modified"
                            | "widen"
                            | "widens"
                            | "widened"
                            | "promote"
                            | "promotes"
                            | "promoted"
                            | "classify"
                            | "classifies"
                            | "classified"
                            | "deliver"
                            | "delivers"
                            | "supply"
                            | "supplies"
                            | "establish"
                            | "establishes"
                            | "guarantee"
                            | "guarantees"
                            | "has"
                            | "is"
                            | "becomes"
                            | "proves"
                    ) && !preceded_by_local_negation(&words, verb_position)
                });
                if (has_affirmative_relation(&words, position, subject.len()) || extra_positive)
                    && !relation_is_negated(&words, position, subject.len())
                {
                    return Some(format!(
                        "CAP-024 overclaims {}: {}",
                        subject.join(" "),
                        clause.trim()
                    ));
                }
            }
        }
    }
    None
}

fn cap024_status_violation(text: &str) -> Option<String> {
    let mut section_owner: Option<(usize, String)> = None;
    for record in normalized_claim_records(text) {
        if cap024_record_is_canonical(&record) {
            continue;
        }
        if let Some((level, owner)) = claim_heading(&record) {
            if let Some(owner) = owner {
                section_owner = Some((level, owner));
            } else if section_owner
                .as_ref()
                .is_some_and(|(owned_level, _)| level <= *owned_level)
            {
                section_owner = None;
            }
        }
        let mut carried_owner = section_owner.as_ref().map(|(_, owner)| owner.clone());
        for clause in record.split(['.', ';', '!', '?']) {
            let words = semantic_words(clause);
            if let Some((_, _, owner)) = capability_mentions(&words).last() {
                carried_owner = Some(owner.clone());
            }
            if !has_semantic_capability(&words, "024") && carried_owner.as_deref() != Some("024") {
                continue;
            }
            if words.first().is_some_and(|word| word == "stop") {
                continue;
            }
            let historical = words.iter().any(|word| {
                matches!(
                    word.as_str(),
                    "before" | "prior" | "previously" | "formerly" | "historical"
                )
            }) && !words.iter().any(|word| {
                matches!(
                    word.as_str(),
                    "remains" | "still" | "current" | "today" | "now"
                )
            });
            if historical {
                continue;
            }
            if contains_semantic_phrase(&words, &["not", "accepted"])
                || contains_semantic_phrase(&words, &["not", "yet", "accepted"])
                || contains_semantic_phrase(&words, &["has", "not", "been", "accepted"])
                || contains_semantic_phrase(&words, &["has", "not", "yet", "been", "accepted"])
                || contains_semantic_phrase(&words, &["not", "merged"])
                || contains_semantic_phrase(&words, &["no", "longer", "accepted"])
                || contains_semantic_phrase(&words, &["not", "current", "public", "master"])
                || contains_semantic_phrase(&words, &["not", "the", "current", "public", "master"])
                || contains_semantic_phrase(&words, &["no", "longer", "current"])
                || contains_semantic_phrase(&words, &["no", "longer", "the", "current"])
                || contains_semantic_phrase(&words, &["has", "been", "superseded"])
            {
                return Some(clause.trim().to_owned());
            }
            for phrase in [&["awaits", "acceptance"][..], &["awaiting", "acceptance"]] {
                if semantic_phrase_position(&words, phrase)
                    .is_some_and(|position| !preceded_by_local_negation(&words, position))
                {
                    return Some(clause.trim().to_owned());
                }
            }
            if words
                .iter()
                .any(|word| matches!(word.as_str(), "alert" | "alerts"))
            {
                let alert_position = words
                    .iter()
                    .position(|word| matches!(word.as_str(), "alert" | "alerts"))
                    .expect("checked alert token");
                let new_alert = words
                    .iter()
                    .position(|word| word == "new")
                    .is_some_and(|position| !preceded_by_local_negation(&words, position));
                let open_alert = words
                    .iter()
                    .position(|word| word == "open")
                    .is_some_and(|position| !preceded_by_local_negation(&words, position));
                let surfaced_alert = words
                    .iter()
                    .position(|word| {
                        matches!(word.as_str(), "surfaced" | "created" | "exists" | "exist")
                    })
                    .is_some_and(|position| !preceded_by_local_negation(&words, position));
                let has_alert =
                    words
                        .iter()
                        .position(|word| word == "has")
                        .is_some_and(|position| {
                            position < alert_position
                                && !preceded_by_local_negation(&words, position)
                                && !words[position + 1..=alert_position]
                                    .iter()
                                    .any(|word| matches!(word.as_str(), "no" | "not" | "without"))
                        });
                let wrong_alert_number = words[alert_position + 1..]
                    .iter()
                    .enumerate()
                    .find_map(|(offset, word)| {
                        word.parse::<u32>().ok().map(|number| (offset, number))
                    })
                    .is_some_and(|(offset, number)| {
                        number != 4
                            && !preceded_by_local_negation(&words, alert_position + 1 + offset)
                    });
                if new_alert || open_alert || surfaced_alert || has_alert || wrong_alert_number {
                    return Some(clause.trim().to_owned());
                }
            }
            for status in [
                "candidate",
                "pending",
                "draft",
                "unaccepted",
                "revoked",
                "reverted",
                "withdrawn",
                "rejected",
                "failed",
                "broken",
                "superseded",
                "unmerged",
                "unpublished",
                "proposed",
            ] {
                if let Some(position) = words.iter().position(|word| word == status) {
                    if !preceded_by_local_negation(&words, position) {
                        return Some(clause.trim().to_owned());
                    }
                }
            }
            if contains_semantic_phrase(&words, &["local", "only"])
                && !words
                    .iter()
                    .position(|word| word == "local")
                    .is_some_and(|position| preceded_by_local_negation(&words, position))
            {
                return Some(clause.trim().to_owned());
            }
        }
    }
    None
}

fn stopped_capability_violation(text: &str) -> Option<String> {
    const POSITIVE: &[&[&str]] = &[
        &["authorized", "for", "implementation"],
        &["approved", "for", "implementation"],
        &["approved", "to", "proceed"],
        &["cleared", "for", "implementation"],
        &["implementation", "is", "authorized"],
        &["implementation", "is", "approved"],
        &["implementation", "is", "underway"],
        &["implementation", "is", "in", "progress"],
        &["implementation", "has", "begun"],
        &["implementation", "has", "started"],
        &["implementation", "is", "planned"],
        &["implementation", "is", "scheduled"],
        &["is", "implemented"],
        &["implements"],
        &["ready", "for", "implementation"],
        &["implementation", "ready"],
        &["implementation", "approval"],
        &["active", "implementation"],
        &["active", "development"],
        &["will", "implement"],
        &["may", "proceed"],
        &["can", "proceed"],
    ];
    let rendered = markdown_outside_fences(text);
    let mut section_owner: Option<(usize, String)> = None;
    let mut historical_section: Option<usize> = None;
    for record in normalized_claim_records_from_rendered(&rendered) {
        let normalized = normalized_words(&record);
        if normalized == CAP016_LOCAL_MODDECL_STOP_BOUNDARY
            || normalized == CAP023_ZERO_PRODUCTION_BOUNDARY
            || POST_CAP020_DECISION_CONTRACTS.contains(&normalized.as_str())
            || POST_CAP021_DECISION_CONTRACTS.contains(&normalized.as_str())
            || POST_CAP023_DECISION_CONTRACTS.contains(&normalized.as_str())
            || POST_CAP024_DECISION_CONTRACTS.contains(&normalized.as_str())
        {
            continue;
        }
        if let Some((level, owner)) = claim_heading(&record) {
            let heading_words = semantic_words(&record);
            let historical_ranking = heading_words.iter().any(|word| word == "post")
                && heading_words.iter().any(|word| word == "ranking")
                && capability_mentions(&heading_words)
                    .iter()
                    .any(|(_, _, owner)| matches!(owner.as_str(), "020" | "021" | "023"));
            if historical_ranking {
                historical_section = Some(level);
            } else if historical_section.is_some_and(|owned_level| level <= owned_level) {
                historical_section = None;
            }
            if let Some(owner) = owner {
                section_owner = Some((level, owner));
            } else if section_owner
                .as_ref()
                .is_some_and(|(owned_level, _)| level <= *owned_level)
            {
                section_owner = None;
            }
        }
        let mut carried_owner = section_owner.as_ref().map(|(_, owner)| owner.clone());
        for clause in record.split(['.', ';', '!', '?']) {
            let words = semantic_words(clause);
            if let Some((_, _, owner)) = capability_mentions(&words).last() {
                carried_owner = Some(owner.clone());
            }
            let runtime_acquisition_subject =
                contains_semantic_phrase(&words, &["runtime", "byte", "file", "acquisition"])
                    || contains_semantic_phrase(&words, &["runtime", "acquisition"]);
            let moddecl_subject = words.iter().any(|word| word == "moddecl");
            let continuation = words.first().is_some_and(|word| {
                matches!(
                    word.as_str(),
                    "it" | "its" | "this" | "that" | "implementation" | "work"
                )
            });
            let owner = if has_semantic_capability(&words, "016") || moddecl_subject {
                Some("016")
            } else if has_semantic_capability(&words, "022") || runtime_acquisition_subject {
                Some("022")
            } else if section_owner
                .as_ref()
                .is_some_and(|(_, owner)| matches!(owner.as_str(), "016" | "022"))
            {
                section_owner.as_ref().map(|(_, owner)| owner.as_str())
            } else if continuation {
                carried_owner.as_deref()
            } else {
                None
            };
            let Some(owner) = owner.filter(|owner| matches!(*owner, "016" | "022")) else {
                continue;
            };
            let historical = historical_section.is_some()
                || words.iter().any(|word| {
                    matches!(
                        word.as_str(),
                        "historical" | "historically" | "former" | "formerly" | "previously"
                    )
                }) && !words.iter().any(|word| {
                    matches!(
                        word.as_str(),
                        "now" | "current" | "today" | "still" | "remains"
                    )
                });
            let conditional_stop = words.iter().any(|word| word == "stop")
                && words
                    .iter()
                    .any(|word| matches!(word.as_str(), "if" | "unless" | "before" | "until"));
            if historical || conditional_stop {
                continue;
            }
            for phrase in POSITIVE {
                if let Some(position) = semantic_phrase_position(&words, phrase) {
                    let predicate_position = position + phrase.len() - 1;
                    if !preceded_by_local_negation(&words, predicate_position)
                        && !relation_is_negated(&words, position, phrase.len())
                    {
                        return Some(format!(
                            "stopped CAP-{owner} is promoted by {}: {}",
                            phrase.join(" "),
                            clause.trim()
                        ));
                    }
                }
            }
            for predicate in ["authorized", "approved", "cleared"] {
                if let Some(position) = words.iter().position(|word| word == predicate) {
                    if !preceded_by_local_negation(&words, position) {
                        return Some(format!(
                            "stopped CAP-{owner} receives current authorization: {}",
                            clause.trim()
                        ));
                    }
                }
            }
            if words.iter().any(|word| word == "implementation") {
                for predicate in [
                    "underway",
                    "active",
                    "current",
                    "now",
                    "begun",
                    "started",
                    "planned",
                    "scheduled",
                    "ready",
                ] {
                    if let Some(position) = words.iter().position(|word| word == predicate) {
                        if !preceded_by_local_negation(&words, position) {
                            return Some(format!(
                                "stopped CAP-{owner} has current implementation state: {}",
                                clause.trim()
                            ));
                        }
                    }
                }
            }
            for (position, word) in words.iter().enumerate() {
                let Some(rank) = rank_word(word) else {
                    continue;
                };
                let rank_relation = words
                    .get(position.wrapping_sub(1))
                    .is_some_and(|word| word == "rank")
                    || words[position.saturating_sub(3)..position]
                        .iter()
                        .any(|word| {
                            matches!(word.as_str(), "rank" | "ranks" | "ranked" | "priority")
                        })
                    || words
                        .get(position + 1)
                        .is_some_and(|word| word == "priority");
                if rank_relation && rank <= 3 && !preceded_by_local_negation(&words, position) {
                    return Some(format!(
                        "stopped CAP-{owner} receives current rank {rank}: {}",
                        clause.trim()
                    ));
                }
            }
        }
    }
    None
}

fn core_mentions(words: &[String]) -> Vec<(usize, usize, String)> {
    let mut mentions = Vec::new();
    let mut index = 0;
    while index < words.len() {
        let compact = words[index]
            .strip_prefix("core")
            .filter(|number| !number.is_empty() && number.chars().all(char::is_numeric));
        let split = (words[index] == "core")
            .then(|| words.get(index + 1))
            .flatten()
            .filter(|number| number.chars().all(char::is_numeric))
            .map(String::as_str);
        if let Some(number) = compact.or(split) {
            let end = index + 1 + usize::from(words[index] == "core");
            mentions.push((index, end, number.to_owned()));
            index = end;
            continue;
        }
        index += 1;
    }
    mentions
}

fn core090_overclaim_violation(text: &str) -> Option<String> {
    const EXCLUDED: &[&[&str]] = &[
        &["general", "memory", "safety"],
        &["memory", "safety"],
        &["complete", "ownership"],
        &["general", "ownership"],
        &["projected", "borrowing"],
        &["borrowing"],
        &["lifetime"],
        &["drop"],
        &["abi"],
        &["accelerator"],
        &["accelerators"],
    ];
    let rendered = markdown_outside_fences(text);
    let mut section_owner: Option<(usize, String)> = None;
    for record in normalized_claim_records_from_rendered(&rendered) {
        let words = semantic_words(&record);
        let heading_owner = core_mentions(&words)
            .into_iter()
            .find(|(start, _, _)| *start <= 1)
            .map(|(_, _, owner)| owner);
        let bold_core_heading = record.trim_start().starts_with("**")
            && record.trim_start()[2..].contains("**")
            && heading_owner.is_some();
        if let Some(level) = atx_heading_level(&record).or(bold_core_heading.then_some(3)) {
            if let Some(owner) = heading_owner {
                section_owner = Some((level, owner));
            } else if section_owner
                .as_ref()
                .is_some_and(|(owned_level, _)| level <= *owned_level)
            {
                section_owner = None;
            }
        }
        let mut carried_owner = section_owner.as_ref().map(|(_, owner)| owner.clone());
        for clause in record.split(['.', ';', '!', '?']) {
            let words = semantic_words(clause);
            if let Some((_, _, owner)) = core_mentions(&words).last() {
                carried_owner = Some(owner.clone());
            }
            if !core_mentions(&words)
                .iter()
                .any(|(_, _, owner)| owner == "090")
                && carried_owner.as_deref() != Some("090")
            {
                continue;
            }
            let classification_context = words.iter().any(|word| {
                matches!(
                    word.as_str(),
                    "class" | "classified" | "classification" | "row"
                )
            });
            let direct_classification = |position: usize| {
                let relation = words[..position]
                    .iter()
                    .rposition(|word| matches!(word.as_str(), "is" | "remains" | "becomes"));
                relation.is_some_and(|relation| {
                    words[relation + 1..position].iter().all(|word| {
                        matches!(
                            word.as_str(),
                            "a" | "an" | "as" | "classified" | "currently" | "now" | "still"
                        )
                    })
                })
            };
            let stable = words.iter().position(|word| word == "stable");
            let end_to_end = semantic_phrase_position(&words, &["end", "to", "end"]);
            for (position, label) in stable
                .map(|position| (position, "STABLE"))
                .into_iter()
                .chain(end_to_end.map(|position| (position, "END_TO_END")))
            {
                if !preceded_by_local_negation(&words, position)
                    && (classification_context || direct_classification(position))
                {
                    return Some(format!(
                        "CORE-090 is promoted to {label}: {}",
                        clause.trim()
                    ));
                }
            }
            let shared_predicate_denial = |subject_start: usize| {
                let fragment_start = words[..subject_start]
                    .iter()
                    .rposition(|word| {
                        matches!(word.as_str(), "but" | "yet" | "however" | "whereas")
                    })
                    .map_or(0, |position| position + 1);
                let relation = words[fragment_start..subject_start]
                    .iter()
                    .rposition(|word| {
                        matches!(
                            word.as_str(),
                            "provide"
                                | "provides"
                                | "support"
                                | "supports"
                                | "enable"
                                | "enables"
                                | "implement"
                                | "implements"
                                | "establish"
                                | "establishes"
                                | "guarantee"
                                | "guarantees"
                        )
                    })
                    .map(|position| fragment_start + position);
                let Some(relation) = relation else {
                    return false;
                };
                preceded_by_local_negation(&words, relation)
            };
            for subject in EXCLUDED {
                let Some(position) = semantic_phrase_position(&words, subject) else {
                    continue;
                };
                let positive = has_affirmative_relation(&words, position, subject.len())
                    || words.iter().enumerate().any(|(verb_position, word)| {
                        matches!(
                            word.as_str(),
                            "is" | "are"
                                | "has"
                                | "have"
                                | "guarantees"
                                | "establish"
                                | "establishes"
                                | "complete"
                                | "completes"
                        ) && !preceded_by_local_negation(&words, verb_position)
                    });
                if positive
                    && !relation_is_negated(&words, position, subject.len())
                    && !shared_predicate_denial(position)
                {
                    return Some(format!(
                        "CORE-090 overclaims {}: {}",
                        subject.join(" "),
                        clause.trim()
                    ));
                }
            }
        }
    }
    None
}

fn consumed_cap024_evidence_ranking_violation(text: &str) -> Option<String> {
    for record in normalized_claim_records(text) {
        if cap024_record_is_canonical(&record) {
            continue;
        }
        for clause in record.split(['.', ';', '!', '?']) {
            let words = semantic_words(clause);
            let evidence_subject =
                contains_semantic_phrase(&words, &["inference", "reproducibility"])
                    || (contains_semantic_phrase(&words, &["accepted", "head"])
                        && words.iter().any(|word| {
                            matches!(
                                word.as_str(),
                                "artifact"
                                    | "artifacts"
                                    | "footprint"
                                    | "reproducibility"
                                    | "evidence"
                                    | "bundle"
                                    | "gate"
                            )
                        }))
                    || (capability_mentions(&words)
                        .iter()
                        .any(|(_, _, owner)| matches!(owner.as_str(), "023" | "024"))
                        && (contains_semantic_phrase(&words, &["artifact", "footprint"])
                            || contains_semantic_phrase(&words, &["evidence", "gate"])));
            if !evidence_subject {
                continue;
            }
            let historical_or_completed = words.iter().any(|word| {
                matches!(
                    word.as_str(),
                    "historical"
                        | "historically"
                        | "previously"
                        | "former"
                        | "formerly"
                        | "completed"
                        | "closed"
                        | "consumed"
                )
            }) && !words.iter().any(|word| {
                matches!(
                    word.as_str(),
                    "current" | "now" | "still" | "remains" | "today"
                )
            });
            if historical_or_completed {
                continue;
            }
            let ranking = words.iter().enumerate().find(|(_, word)| {
                matches!(
                    word.as_str(),
                    "rank" | "ranks" | "ranked" | "priority" | "top" | "first" | "next"
                )
            });
            if ranking.is_some_and(|(position, _)| !preceded_by_local_negation(&words, position)) {
                return Some(clause.trim().to_owned());
            }
        }
    }
    None
}

fn cap020_status_violation(text: &str) -> Option<String> {
    const NEGATABLE_STATUS: &[&[&str]] = &[
        &["remains", "a", "candidate"],
        &["a", "candidate"],
        &["candidate", "only"],
        &["pending", "acceptance"],
        &["acceptance", "is", "pending"],
        &["acceptance", "remains", "pending"],
        &["unaccepted"],
        &["proposed"],
        &["local", "only"],
        &["local", "candidate"],
        &["unpublished"],
        &["unmerged"],
        &["awaiting", "acceptance"],
        &["awaits", "acceptance"],
    ];
    const ABSOLUTE_CONTRADICTIONS: &[&[&str]] = &[
        &["has", "not", "yet", "been", "accepted"],
        &["has", "not", "been", "accepted"],
        &["not", "yet", "accepted"],
        &["not", "yet", "published"],
        &["acceptance", "revoked"],
        &["acceptance", "reverted"],
        &["acceptance", "withdrawn"],
        &["acceptance", "rejected"],
        &["non", "zero", "production"],
        &["not", "zero", "production"],
        &["no", "longer", "zero", "production"],
    ];
    for paragraph in normalized_claim_records(text) {
        if paragraph == CAP020_EVIDENCE_PARAGRAPH {
            continue;
        }
        let mut carried_owner = None;
        for clause in paragraph.split(['.', ';', '!', '?']) {
            let words = semantic_words(clause);
            let mentions = capability_mentions(&words);
            if let Some((_, _, owner)) = mentions.last() {
                carried_owner = Some(owner.clone());
            }
            if !has_semantic_capability(&words, "020") && carried_owner.as_deref() != Some("020") {
                continue;
            }
            if let Some(phrase) = ABSOLUTE_CONTRADICTIONS
                .iter()
                .find(|phrase| contains_semantic_phrase(&words, phrase))
            {
                return Some(phrase.join(" "));
            }
            if contains_semantic_phrase(&words, &["did", "not", "pass"])
                || contains_semantic_phrase(&words, &["does", "not", "pass"])
                || contains_semantic_phrase(&words, &["not", "pass"])
            {
                return Some(clause.trim().to_owned());
            }
            for status in [
                "fail",
                "fails",
                "failed",
                "revoked",
                "reverted",
                "withdrawn",
            ] {
                if let Some(position) = words.iter().position(|word| word == status) {
                    let describes_failed_programs = matches!(status, "fail" | "fails" | "failed")
                        && words.get(position + 1).is_some_and(|word| {
                            matches!(word.as_str(), "program" | "programs" | "case" | "cases")
                        });
                    let acceptance_context = position <= 3
                        || words.iter().any(|word| {
                            matches!(
                                word.as_str(),
                                "ci" | "check"
                                    | "checks"
                                    | "run"
                                    | "merge"
                                    | "acceptance"
                                    | "evidence"
                            )
                        });
                    if !describes_failed_programs
                        && acceptance_context
                        && !preceded_by_local_negation(&words, position)
                    {
                        return Some(clause.trim().to_owned());
                    }
                }
            }
            if let Some(position) = words.iter().position(|word| word == "new") {
                if words[position..].iter().any(|word| word == "alert")
                    && !preceded_by_local_negation(&words, position)
                {
                    return Some(clause.trim().to_owned());
                }
            }
            if let Some(position) = words.iter().position(|word| word == "additional") {
                if words[position..].iter().any(|word| word == "finding")
                    && !preceded_by_local_negation(&words, position)
                {
                    return Some(clause.trim().to_owned());
                }
            }
            for phrase in NEGATABLE_STATUS {
                if let Some(position) = semantic_phrase_position(&words, phrase) {
                    if !preceded_by_local_negation(&words, position) {
                        return Some(phrase.join(" "));
                    }
                }
            }

            let lower = clause.to_ascii_lowercase();
            let vectors = signed_bracket_vectors(clause)
                .into_iter()
                .filter(|(position, _)| !value_is_locally_negated(clause, *position))
                .collect::<Vec<_>>();
            let ordinary_position = lower.find("ordinary");
            let wrapping_position = lower.find("wrapping");
            if let (Some(ordinary), Some(wrapping)) = (ordinary_position, wrapping_position) {
                if ordinary < wrapping {
                    let asserted_after_wrapping = vectors
                        .iter()
                        .filter(|(position, _)| *position > wrapping)
                        .map(|(_, vector)| vector.as_slice())
                        .collect::<Vec<_>>();
                    let ordinary_has_no_earlier_vector = !vectors
                        .iter()
                        .any(|(position, _)| *position > ordinary && *position < wrapping);
                    let combined_pair = ordinary_has_no_earlier_vector
                        && asserted_after_wrapping.len() == 2
                        && asserted_after_wrapping[0] == [50, 122]
                        && asserted_after_wrapping[1] == [-2, 5];
                    let separate_pair = vector_assertion_region_is_valid(
                        clause,
                        ordinary,
                        wrapping,
                        &vectors,
                        &[50, 122],
                    ) && vector_assertion_region_is_valid(
                        clause,
                        wrapping,
                        clause.len(),
                        &vectors,
                        &[-2, 5],
                    );
                    if !combined_pair && !separate_pair {
                        return Some(clause.trim().to_owned());
                    }
                }
            } else if let Some(ordinary) = ordinary_position {
                if !vector_assertion_region_is_valid(
                    clause,
                    ordinary,
                    clause.len(),
                    &vectors,
                    &[50, 122],
                ) {
                    return Some(clause.trim().to_owned());
                }
            } else if let Some(wrapping) = wrapping_position {
                if !vector_assertion_region_is_valid(
                    clause,
                    wrapping,
                    clause.len(),
                    &vectors,
                    &[-2, 5],
                ) {
                    return Some(clause.trim().to_owned());
                }
            }
            let returns_result_vector = words
                .iter()
                .any(|word| matches!(word.as_str(), "return" | "returns"))
                && !vectors.is_empty();
            if returns_result_vector
                && vectors
                    .iter()
                    .any(|(_, vector)| !matches!(vector.as_slice(), [50, 122] | [-2, 5]))
            {
                return Some(clause.trim().to_owned());
            }
            for keyword in [
                "exit",
                "exits",
                "sentinel",
                "return",
                "returns",
                "terminate",
                "terminates",
                "termination",
            ] {
                if let Some(position) = words.iter().position(|word| word == keyword) {
                    if preceded_by_local_negation(&words, position) {
                        continue;
                    }
                    if matches!(keyword, "return" | "returns") && returns_result_vector {
                        continue;
                    }
                    let value_anchor = if matches!(keyword, "terminate" | "terminates")
                        && words.get(position + 1).is_some_and(|word| word == "with")
                        && words.get(position + 2).is_some_and(|word| word == "code")
                    {
                        position + 2
                    } else if keyword == "termination"
                        && words.get(position + 1).is_some_and(|word| word == "code")
                    {
                        position + 1
                    } else {
                        position
                    };
                    let max_tokens = if matches!(keyword, "return" | "returns") {
                        3
                    } else {
                        8
                    };
                    if scalar_assertion_values(&words, value_anchor, max_tokens)
                        .into_iter()
                        .any(|value| value != 91)
                    {
                        return Some(clause.trim().to_owned());
                    }
                }
            }
        }
    }
    None
}

fn readiness_promotion_violation(text: &str) -> Option<String> {
    let rendered = markdown_outside_fences(text);
    readiness_promotion_violation_from_rendered(&rendered)
}

fn readiness_promotion_violation_from_rendered(rendered: &str) -> Option<String> {
    const POSITIVE: &[&[&str]] = &[
        &["approved", "to", "proceed"],
        &["authorized", "for", "implementation"],
        &["implementation", "is", "approved"],
        &["implementation", "is", "authorized"],
        &["implementation", "is", "underway"],
        &["implementation", "is", "in", "progress"],
        &["implementation", "has", "begun"],
        &["implementation", "has", "started"],
        &["implementation", "is", "scheduled"],
        &["implementation", "is", "planned"],
        &["ready", "for", "implementation"],
        &["implementation", "ready"],
        &["implementation", "approval"],
        &["implementation", "begins"],
        &["production", "implementation"],
        &["implementation", "task"],
        &["implementation", "work"],
        &["next", "implementation", "task"],
        &["is", "being", "implemented"],
        &["work", "is", "underway"],
        &["implementation", "may", "proceed"],
        &["may", "proceed"],
        &["can", "proceed"],
        &["cleared", "for", "implementation"],
        &["will", "implement"],
        &["is", "now", "implemented"],
        &["now", "an", "executable", "product", "gate"],
        &["active", "development"],
        &["will", "ship", "next"],
    ];
    for record in normalized_claim_records_from_rendered(rendered) {
        let mut carried_readiness = false;
        for sentence in record.split(['.', '!', '?']) {
            let sentence_words = semantic_words(sentence);
            let explicit_subject = contains_semantic_phrase(&sentence_words, &["rank", "1"])
                || contains_semantic_phrase(&sentence_words, &["rank", "one"])
                || contains_semantic_phrase(&sentence_words, &["rank", "2"])
                || contains_semantic_phrase(&sentence_words, &["rank", "two"])
                || contains_semantic_phrase(&sentence_words, &["rank", "3"])
                || contains_semantic_phrase(&sentence_words, &["rank", "three"])
                || contains_semantic_phrase(&sentence_words, &["runtime", "acquisition"])
                || contains_semantic_phrase(
                    &sentence_words,
                    &["runtime", "byte", "file", "acquisition"],
                )
                || sentence_words
                    .iter()
                    .any(|word| matches!(word.as_str(), "quantized" | "quantization"))
                || contains_semantic_phrase(&sentence_words, &["recursive", "arrays"])
                || contains_semantic_phrase(&sentence_words, &["recursive", "array"]);
            let continuation = carried_readiness
                && sentence_words.first().is_some_and(|word| {
                    matches!(
                        word.as_str(),
                        "its"
                            | "it"
                            | "this"
                            | "implementation"
                            | "work"
                            | "effort"
                            | "task"
                            | "development"
                    )
                });
            if !explicit_subject && !continuation {
                carried_readiness = false;
                continue;
            }
            carried_readiness = explicit_subject || continuation;
            for clause in sentence.split(';') {
                let clause_words = semantic_words(clause);
                let mut starts = vec![0];
                starts.extend(
                    clause_words
                        .iter()
                        .enumerate()
                        .filter(|(_, word)| {
                            matches!(word.as_str(), "but" | "yet" | "however" | "whereas")
                        })
                        .map(|(position, _)| position + 1),
                );
                starts.push(clause_words.len());
                for bounds in starts.windows(2) {
                    let fragment = &clause_words[bounds[0]..bounds[1]];
                    let approved_only_for_readiness = |position: usize| {
                        let suffix = &fragment[position..(position + 10).min(fragment.len())];
                        suffix.iter().any(|word| word == "only")
                            && suffix
                                .iter()
                                .any(|word| matches!(word.as_str(), "readiness" | "probe"))
                            && (suffix.iter().any(|word| word == "not")
                                || !suffix.iter().any(|word| word == "implementation"))
                    };
                    for phrase in POSITIVE {
                        for position in fragment
                            .windows(phrase.len())
                            .enumerate()
                            .filter(|(_, candidate)| {
                                candidate
                                    .iter()
                                    .map(String::as_str)
                                    .eq(phrase.iter().copied())
                            })
                            .map(|(position, _)| position)
                        {
                            if !preceded_by_local_negation(fragment, position)
                                && !approved_only_for_readiness(position)
                            {
                                return Some(format!("{}: {}", phrase.join(" "), record.trim()));
                            }
                        }
                    }
                    for (position, word) in fragment.iter().enumerate() {
                        if matches!(
                            word.as_str(),
                            "underway"
                                | "approved"
                                | "authorized"
                                | "executable"
                                | "cleared"
                                | "begun"
                                | "started"
                                | "implemented"
                                | "progress"
                                | "scheduled"
                                | "planned"
                                | "begins"
                        ) && !preceded_by_local_negation(fragment, position)
                            && !approved_only_for_readiness(position)
                        {
                            return Some(record.trim().to_owned());
                        }
                        if word == "active"
                            && fragment
                                .get(position + 1)
                                .is_some_and(|next| next == "development")
                            && !preceded_by_local_negation(fragment, position)
                        {
                            return Some(record.trim().to_owned());
                        }
                    }
                }
            }
        }
    }
    None
}

fn post_cap023_readiness_promotion_violation(text: &str) -> Option<String> {
    let mut readiness_records = Vec::new();
    let mut section_readiness: Option<(usize, u8)> = None;
    for record in normalized_claim_records(text) {
        let normalized = normalized_words(&record);
        if POST_CAP023_DECISION_CONTRACTS.contains(&normalized.as_str()) {
            continue;
        }
        if let Some((level, _)) = claim_heading(&record) {
            let trimmed = record.trim_start();
            let heading_text = if let Some(rest) = trimmed.strip_prefix("**") {
                rest.find("**").map_or(trimmed, |end| &trimmed[..end + 4])
            } else {
                trimmed
            };
            let heading_words = semantic_words(heading_text);
            let names_capability = !capability_mentions(&heading_words).is_empty();
            let heading_rank = if contains_semantic_phrase(&heading_words, &["rank", "2"])
                || contains_semantic_phrase(&heading_words, &["rank", "two"])
                || (!names_capability
                    && heading_words.iter().any(|word| word == "copydata")
                    && heading_words.iter().any(|word| {
                        matches!(word.as_str(), "application" | "profile" | "composition")
                    })) {
                Some(2)
            } else if contains_semantic_phrase(&heading_words, &["rank", "3"])
                || contains_semantic_phrase(&heading_words, &["rank", "three"])
                || !names_capability
                    && heading_words
                        .iter()
                        .any(|word| matches!(word.as_str(), "quantized" | "quantization"))
            {
                Some(3)
            } else {
                None
            };
            if let Some(rank) = heading_rank {
                section_readiness = Some((level, rank));
            } else if section_readiness.is_some_and(|(owned_level, _)| level <= owned_level) {
                section_readiness = None;
            }
        }
        for fragment in record.split(['.', ';', '!', '?', ',']) {
            let fragment_words = semantic_words(fragment);
            if fragment_words.is_empty() {
                continue;
            }
            let explicit_rank_two = contains_semantic_phrase(&fragment_words, &["rank", "2"])
                || contains_semantic_phrase(&fragment_words, &["rank", "two"]);
            let explicit_rank_three = contains_semantic_phrase(&fragment_words, &["rank", "3"])
                || contains_semantic_phrase(&fragment_words, &["rank", "three"])
                || fragment_words
                    .iter()
                    .any(|word| matches!(word.as_str(), "quantized" | "quantization"));
            let composition = fragment_words.iter().any(|word| word == "copydata")
                && fragment_words
                    .iter()
                    .any(|word| matches!(word.as_str(), "application" | "profile" | "composition"));
            let historical_core = fragment_words.iter().any(|word| word == "core")
                && fragment_words.iter().any(|word| word == "milestone");
            let historical_accepted = capability_mentions(&fragment_words)
                .iter()
                .any(|(_, _, owner)| owner != "023")
                && fragment_words.iter().any(|word| word == "accepted")
                && fragment_words.iter().any(|word| {
                    matches!(
                        word.as_str(),
                        "candidate" | "checks" | "evidence" | "green" | "merge"
                    )
                });
            let rank =
                if explicit_rank_two || composition && !historical_core && !historical_accepted {
                    Some(2)
                } else if explicit_rank_three {
                    Some(3)
                } else {
                    section_readiness.map(|(_, rank)| rank)
                };
            if let Some(rank) = rank {
                readiness_records.push(format!("Rank {rank} {}", fragment.trim()));
            }
        }
    }
    readiness_promotion_violation(&readiness_records.join("\n\n"))
}

fn post_cap024_readiness_promotion_violation(text: &str) -> Option<String> {
    let ranked_rendered = markdown_with_ordered_list_ranks(text);
    let mut readiness_records = Vec::new();
    let mut section_readiness: Option<(usize, u8)> = None;
    for record in normalized_claim_records_from_rendered(&ranked_rendered) {
        let normalized = normalized_words(&record);
        let decision_record = normalized
            .strip_prefix("Rank ")
            .and_then(|rest| rest.split_once(' ').map(|(_, decision)| decision))
            .unwrap_or(&normalized);
        if POST_CAP024_DECISION_CONTRACTS.contains(&decision_record) {
            continue;
        }
        if let Some((level, _)) = claim_heading(&record) {
            if let Some(rank) = post_cap024_category(&semantic_words(&record)) {
                section_readiness = Some((level, rank));
            } else if section_readiness.is_some_and(|(owned_level, _)| level <= owned_level) {
                section_readiness = None;
            }
        }
        for fragment in record.split(['.', ';', '!', '?', ',']) {
            let fragment_words = semantic_words(fragment);
            if fragment_words.is_empty() {
                continue;
            }
            let explicit_rank = fragment_words
                .windows(2)
                .find_map(|pair| (pair[0] == "rank").then(|| rank_word(&pair[1])).flatten());
            let category = post_cap024_category(&fragment_words);
            let rank = explicit_rank
                .or(category)
                .or_else(|| section_readiness.map(|(_, rank)| rank));
            if let Some(rank) = rank {
                readiness_records.push(format!("Rank {rank} {}", fragment.trim()));
            }
        }
    }
    readiness_promotion_violation(&readiness_records.join("\n\n"))
}

fn assert_cap020_contract_mutation_fixtures() {
    for accepted in [
        "CAP-020 is an accepted product gate, not a candidate and not pending acceptance.",
        "CAP-020 executes the flat 2x3-by-3 matrix-vector product gate.",
        "CAP-020 contains matrix-product evidence but adds no matrix type.",
        "CAP020 changes no parser or language profile.",
        "CAP 020 doesn't add file input or runtime ingestion.",
        "CAP-020 does not guarantee performance.",
        "CAP-020 is an accepted product gate, not a separate profile.",
        "CAP-020 proves the flat matvec; CAP-021 owns runtime acquisition.",
        "CAP-020 lacks matrix syntax, whereas CAP-021 adds matrix support.",
        "CAP-020 is product evidence, and matrix syntax belongs to CAP-021.",
        "Before CAP-020, matrix types were unsupported.",
        "CAP-020 is accepted. Matrix syntax remains future work.",
        "Stop rank 1 if CAP-020 would require file input or runtime ingestion.",
    ] {
        assert!(cap020_status_violation(accepted).is_none(), "{accepted}");
        assert!(cap020_product_violation(accepted).is_none(), "{accepted}");
    }
    for rejected in [
        "CAP-020 adds a matrix type.",
        "CAP020 parser supports records.",
        "CAP 020's parser works.",
        "| CAP020 | Parser | Supported |",
        "CAP-020 matrix type works.",
        "| CAP 020 | Matrix type | Y |",
        "CAP020 supports matrices.",
        "CAP020 changes production compiler code.",
        "Under CAP 020, file input is supported.",
        "CAP020 runtime ingestion works.",
        "CAP-020 supports recursive arrays.",
        "CAP-020 guarantees stable ABI and performance.",
        "CAP020 doesn't support parser, but it enables matrix types.",
        "Matrix syntax is supported by CAP-020.",
        "CAP-020 enables matrix types, but no file input.",
        "CAP-020 adds matrix syntax in this zero-production product gate.",
        "CAP-020 adds\n matrix types.",
        "CAP-020 does not change the parser and matrix syntax is supported.",
        "CAP-020 adds matrix syntax and recursive arrays remain open.",
        "CAP-020 does not lack matrix syntax.",
    ] {
        assert!(cap020_product_violation(rejected).is_some(), "{rejected}");
    }
    for accepted in [
        "No new CAP-020 alert surfaced.",
        "CAP-020 did not fail and was not revoked.",
        "CAP-020 failed programs trap correctly.",
        "CAP-020 is zero-production product evidence.",
        "CAP020 ordinary oracle is [50, 122].",
        "CAP020 wrapping oracle is [-2, 5].",
        "CAP020 exits 91.",
        "CAP020 ordinary oracle is not [50, 121], but is [50, 122].",
        "CAP020 does not exit 92; it exits 91.",
        "CAP-020 exits 91 after 2 stages.",
        "CAP-020 exits 91, not 92.",
        "CAP020 returns [50, 122].",
    ] {
        assert!(cap020_status_violation(accepted).is_none(), "{accepted}");
    }
    for rejected in [
        "CAP-020 remains a candidate.",
        "CAP020 remains a local candidate.",
        "CAP020 awaits acceptance.",
        "CAP020 is not yet published.",
        "CAP-020 has not yet been accepted.",
        "CAP-020 merge CI later failed and acceptance was revoked.",
        "A new CAP-020 alert surfaced.",
        "CAP020 failed.",
        "CAP020 acceptance was revoked.",
        "CAP020 is non-zero-production.",
        "CAP020 is no longer zero production.",
        "CAP020 ordinary oracle is [50, 121].",
        "CAP020 wrapping oracle is [-2, 6].",
        "CAP020 exits 92.",
        "CAP020 ordinary oracle is (50, 121).",
        "CAP020 returns 92.",
        "CAP020 exit is ninety-two.",
        "CAP-020 did not pass CI.",
        "CAP-020 passed. The ordinary oracle is [50, 121].",
        "CAP020 ordinary oracle is [50, 122] and [50, 121].",
        "CAP020 exits 91 or 92.",
        "CAP020 ordinary oracle is 50 and 121.",
        "CAP020 terminates with code 92.",
    ] {
        assert!(cap020_status_violation(rejected).is_some(), "{rejected}");
    }
    for accepted in [
        "CAP-019 remains the latest compiler/profile capability widening.",
        "CAP-019 was the public baseline before CAP-020.",
        "CAP-019 is the historical baseline before CAP-020.",
        "CAP-019 isn't the current public baseline.",
        "Current public head is CAP-020, after CAP-019.",
    ] {
        assert!(
            stale_cap019_current_violation(accepted).is_none(),
            "{accepted}"
        );
    }
    for rejected in [
        "CAP-019 is the present public baseline.",
        "Current public head: CAP 019.",
        "Project status after CAP-019.",
        "Post-CAP-019 successor order controls task selection.",
        "Historically important, CAP-019 remains master today.",
        "CAP-019 remains the current accepted capability.",
    ] {
        assert!(
            stale_cap019_current_violation(rejected).is_some(),
            "{rejected}"
        );
    }
    for accepted in [
        "Rank 2 remains readiness and a red probe only, not implementation.",
        "Stop rank 2 before implementation if the runtime contract remains unfrozen.",
        "An explicit RFC would permit future runtime acquisition implementation ranking only after a bounded probe.",
        "Rank 3 remains deferred and isn't executable.",
        "Rank 2 is approved only for a readiness probe, not implementation.",
        "Recursive arrays are not in active development.",
    ] {
        assert!(
            readiness_promotion_violation(accepted).is_none(),
            "{accepted}"
        );
    }
    for rejected in [
        "Rank 2 is approved to proceed with runtime acquisition implementation.",
        "Runtime acquisition implementation is now underway.",
        "Rank three is now an executable product gate.",
        "Recursive arrays move into active development.",
        "Rank 2 is executable.",
        "Rank 3 implementation is underway.",
        "Recursive arrays are approved for implementation.",
        "Rank 2 remains readiness only, but implementation is authorized.",
        "Rank 2 is not readiness only; implementation is approved.",
        "Runtime acquisition may proceed without further review.",
        "Rank 2 remains readiness only,\nimplementation is underway.",
        "Rank 2 remains readiness only. Its implementation is now authorized.",
        "Before implementation, rank 2 is approved to proceed.",
        "Future implementation ranking is approved for rank 2.",
        "Rank 2 remains readiness only. Work is underway.",
        "Rank 2 implementation has begun.",
        "Runtime acquisition is being implemented.",
        "Rank 2 implementation is in progress.",
    ] {
        assert!(
            readiness_promotion_violation(rejected).is_some(),
            "{rejected}"
        );
    }
    for accepted in [
        "The tensor-record gate ranks first.",
        "Runtime acquisition ranks second.",
        "Recursive arrays rank third.",
        "Runtime acquisition does not rank first.",
    ] {
        assert!(successor_order_violation(accepted).is_none(), "{accepted}");
    }
    for rejected in [
        "Runtime acquisition ranks first.",
        "The tensor-record gate ranks second.",
        "Recursive arrays rank second.",
        "The tensor-record gate follows runtime acquisition.",
        "Runtime acquisition ranks first and tensor-record ranks second.",
        "Recursive arrays precede the tensor-record gate.",
        "Recursive arrays precede runtime acquisition.",
        "Runtime acquisition comes before the tensor-record gate.",
        "Runtime acquisition is rank 1.",
        "Byte/file acquisition ranks first.",
        "Runtime acquisition, unlike the tensor-record gate, ranks first.",
        "Runtime acquisition is the first priority.",
    ] {
        assert!(successor_order_violation(rejected).is_some(), "{rejected}");
    }
    assert_eq!(
        normalized_markdown_paragraphs(&markdown_outside_fences(CAP020_EVIDENCE_PARAGRAPH)),
        [CAP020_EVIDENCE_PARAGRAPH]
    );
    let fenced_evidence = format!("```text\n{CAP020_EVIDENCE_PARAGRAPH}\n```");
    assert!(
        normalized_markdown_paragraphs(&markdown_outside_fences(&fenced_evidence)).is_empty(),
        "fenced CAP-020 evidence must not count as rendered public truth"
    );
    let mismatched_fence = format!("~~~text\n```\n{CAP020_EVIDENCE_PARAGRAPH}\n```\n~~~");
    assert!(
        normalized_markdown_paragraphs(&markdown_outside_fences(&mismatched_fence)).is_empty(),
        "a mismatched fence marker must not expose hidden CAP-020 evidence"
    );
    let longer_fence = format!("````text\n```\n{CAP020_EVIDENCE_PARAGRAPH}\n````");
    assert!(
        normalized_markdown_paragraphs(&markdown_outside_fences(&longer_fence)).is_empty(),
        "a shorter same-marker run must not close a longer Markdown fence"
    );
    let hidden_contradictions = "```text\nCAP-020 adds matrix types.\nRank 2 implementation is underway.\nRuntime acquisition ranks first.\n```";
    assert!(cap020_product_violation(hidden_contradictions).is_none());
    assert!(cap020_status_violation(hidden_contradictions).is_none());
    assert!(readiness_promotion_violation(hidden_contradictions).is_none());
    assert!(successor_order_violation(hidden_contradictions).is_none());
    assert!(
        cap020_product_violation("<!-- CAP-020 adds matrix types. -->").is_none(),
        "HTML-comment-hidden examples are not rendered capability claims"
    );
    assert!(
        cap020_product_violation("&#60;!-- CAP-020 adds matrix types. -->").is_some(),
        "entity-generated comment syntax remains visible on the one-pass live CAP-020 path"
    );
    assert_eq!(
        markdown_outside_fences("Inline ``` markers stay visible."),
        "Inline ``` markers stay visible.\n"
    );
    assert!(
        markdown_outside_fences("    CAP-020 adds matrix types.")
            .trim()
            .is_empty(),
        "indented Markdown code is not rendered capability prose"
    );
    assert!(
        markdown_outside_fences("\tCAP-020 adds matrix types.")
            .trim()
            .is_empty(),
        "tab-indented Markdown code is not rendered capability prose"
    );
    assert!(
        markdown_outside_fences("- ```text\n  CAP-020 adds matrix types.\n  ```")
            .trim()
            .is_empty(),
        "list-item fenced code is not rendered capability prose"
    );
    assert!(
        markdown_outside_fences(r"`<!--` CAP-020 adds matrix types. `-->`")
            .contains("CAP-020 adds matrix types"),
        "comment markers inside code spans remain visible Markdown"
    );
    let multiline_code_span = "`start\n<!--\nCAP-020 adds matrix types.\n`";
    assert!(
        markdown_outside_fences(multiline_code_span).contains("CAP-020 adds matrix types"),
        "multiline code spans must keep comment-like text visible"
    );
    for visible in [
        "]( CAP-023 adds general inference capability.",
        "][ CAP-023 adds general inference capability.",
        "<not-a-commonmark-tag CAP-023 adds general inference capability.>",
        "> ```text\n> hidden\n\nCAP-023 adds general inference capability.",
        "- ```text\n  hidden\n\nCAP-023 adds general inference capability.",
    ] {
        assert!(
            markdown_outside_fences(visible).contains("CAP-023 adds general inference capability"),
            "visible malformed/container-ended Markdown must not be hidden: {visible}"
        );
    }
    assert!(
        !markdown_outside_fences(
            "[ref]: https://example/CAP-023-adds-general-inference-capability"
        )
        .contains("CAP-023"),
        "link-reference destinations are not rendered prose"
    );
    assert!(
        markdown_outside_fences(r"\<!-- CAP-020 adds matrix types. -->")
            .contains("CAP-020 adds matrix types"),
        "escaped comment markers remain visible Markdown"
    );
    let matrix_header = "| Feature | Spec | Lex | Parse | Res | Ty | Own | TIR | BE | Exec | + | - | D | Docs | Class |";
    assert!(!markdown_table_after_header_is_valid(
        &format!("{matrix_header}\n\n| Row | Y |"),
        matrix_header
    ));
    assert!(!markdown_table_after_header_is_valid(
        &format!("{matrix_header}\n|---|---|\n| Row | Y |"),
        matrix_header
    ));
    let single_hyphen_delimiter = table_cells(matrix_header)
        .expect("matrix header")
        .iter()
        .map(|_| "-")
        .collect::<Vec<_>>()
        .join("|");
    assert!(!markdown_table_after_header_is_valid(
        &format!("{matrix_header}\n{single_hyphen_delimiter}\n| Row | Y |"),
        matrix_header
    ));
}

fn assert_cap021_contract_mutation_fixtures() {
    for accepted in [
        "CAP-021 is an accepted zero-production product gate, not a candidate.",
        "CAP-021 executes one flat source-embedded record-to-score application.",
        "CAP021 changes no parser, language profile, checked IR, verifier, or backend.",
        "CAP 021 adds no tensor type, matrix syntax, runtime ingestion, or quantization.",
        "CAP-021 is product evidence, not a separate profile or feature row.",
        "CAP-021 preserves CAP-020's flat matvec and adds no record type.",
        "Stop if CAP-021 would require runtime acquisition or a quantized representation.",
    ] {
        assert!(cap021_status_violation(accepted).is_none(), "{accepted}");
        assert!(cap021_product_violation(accepted).is_none(), "{accepted}");
    }
    for rejected in [
        "CAP-021 adds a tensor type.",
        "Matrix syntax is supported by CAP-021.",
        "CAP021's record type works.",
        "| CAP 021 | Quantization | Supported |",
        "CAP-021 implements runtime ingestion and file input.",
        "CAP-021 guarantees stable record layout and ABI.",
        "CAP-021 changes production compiler code.",
        "CAP-021 adds\n tensor syntax.",
        "CAP-021 changes no parser, but supports nested arrays.",
        "CAP-021 is zero-production and enables quantization.",
        "CAP-021 adds runtime acquisition readiness.",
        "CAP-021 readiness ranking supports runtime acquisition.",
        "CAP-021 adds quantization probe support.",
        "CAP&#45;021 adds tensor types.",
    ] {
        assert!(cap021_product_violation(rejected).is_some(), "{rejected}");
    }
    let list_continuation = "- Accepted CAP-021.\n    CAP-021 adds tensor types.";
    assert!(
        cap021_product_violation(list_continuation).is_some(),
        "rendered list continuation must remain visible to the CAP-021 claim scanner"
    );
    assert!(
        cap021_product_violation("&#60;!-- CAP-021 adds tensor types. -->").is_some(),
        "entity-generated comment syntax remains visible on the one-pass live CAP-021 path"
    );
    for accepted in [
        "No new CAP-021 alert surfaced.",
        "CAP-021 did not fail and was not revoked.",
        "CAP-021 failed programs trap correctly.",
        "CAP-021 ordinary result is [1, 122, 167, 135, 181, 4938].",
        "CAP-021 wrapping result is [1, -24, 18, 2147483623, -2147483631, -2147483627].",
        "CAP-021 invalid-header result is [0, 0, 0, 0, 0, 0].",
        "CAP-021 exits 91.",
        "CAP-021 does not exit 92; it exits 91.",
    ] {
        assert!(cap021_status_violation(accepted).is_none(), "{accepted}");
    }
    for rejected in [
        "CAP-021 remains a local candidate.",
        "CAP-021 acceptance was revoked.",
        "CAP-021 merge CI failed.",
        "A new CAP-021 alert surfaced.",
        "CAP-021 is no longer zero production.",
        "CAP-021 ordinary result is [1, 122, 167, 135, 181, 4937].",
        "CAP-021 wrapping result is [1, -24, 18, 2147483623, -2147483631, -2147483626].",
        "CAP-021 invalid-header result is [1, 0, 0, 0, 0, 0].",
        "CAP-021 exits 92.",
        "CAP-021 exits 91 or 92.",
        "CAP-021 ordinary result is [1, 122, 167, 135, 181, 4937], while CAP-021 remains an accepted product gate.",
        "CAP-021 ordinary result differs from the accepted oracle.",
        "CAP-021 invalid header returns a nonzero result.",
        "CAP-021 acceptance remains pending.",
    ] {
        assert!(cap021_status_violation(rejected).is_some(), "{rejected}");
    }
    for accepted in [
        "CAP-020 remains an accepted product component beneath CAP-021.",
        "CAP-020 was the public baseline before CAP-021.",
        "CAP-020 is not the current public baseline.",
        "Current public head is CAP-021, after CAP-020.",
    ] {
        assert!(
            stale_cap020_current_violation(accepted).is_none(),
            "{accepted}"
        );
    }
    for rejected in [
        "CAP-020 is the current public master.",
        "Current public head: CAP 020.",
        "CAP-020 remains our public baseline.",
        "The present project status is CAP-020.",
        "CAP-020 continues as the public baseline.",
    ] {
        assert!(
            stale_cap020_current_violation(rejected).is_some(),
            "{rejected}"
        );
    }
    for accepted in [
        "Rank 1 remains readiness and a red probe only, not implementation.",
        "Runtime acquisition is not in active development.",
        "Rank 2 quantization remains a readiness probe only.",
        "Rank 3 remains deferred and is not executable.",
    ] {
        assert!(
            readiness_promotion_violation(accepted).is_none(),
            "{accepted}"
        );
    }
    for rejected in [
        "Rank 1 runtime acquisition implementation is underway.",
        "Runtime acquisition is being implemented.",
        "Rank 2 quantized-kernel implementation has begun.",
        "Quantization is approved for implementation.",
        "Rank 3 recursive arrays are now an executable product gate.",
        "Rank 1 remains readiness only, but implementation is authorized.",
        "Rank 1 is production implementation.",
        "Runtime acquisition is the next implementation task.",
    ] {
        assert!(
            readiness_promotion_violation(rejected).is_some(),
            "{rejected}"
        );
    }
    for accepted in [
        "Runtime acquisition ranks first.",
        "The quantized numerical-kernel probe ranks second.",
        "Recursive arrays rank third.",
        "Quantization does not rank first.",
    ] {
        assert!(
            post_cap021_successor_order_violation(accepted).is_none(),
            "{accepted}"
        );
    }
    for rejected in [
        "Quantization ranks first.",
        "Runtime acquisition ranks second.",
        "Recursive arrays rank second.",
        "Runtime acquisition follows quantization.",
        "Recursive arrays precede runtime acquisition.",
        "Quantized arithmetic is rank 3.",
        "Quantization is rank one.",
        "Runtime acquisition is rank two.",
    ] {
        assert!(
            post_cap021_successor_order_violation(rejected).is_some(),
            "{rejected}"
        );
    }
    assert_eq!(
        normalized_markdown_paragraphs(&markdown_outside_fences(CAP021_EVIDENCE_PARAGRAPH)),
        [CAP021_EVIDENCE_PARAGRAPH]
    );
    let fenced = format!("```text\n{CAP021_EVIDENCE_PARAGRAPH}\n```");
    assert!(
        normalized_markdown_paragraphs(&markdown_outside_fences(&fenced)).is_empty(),
        "fenced CAP-021 evidence must not count as rendered public truth"
    );
}

fn assert_cap023_contract_mutation_fixtures() {
    let repeated_where_frozen = format!("{CAP023_EVIDENCE_PARAGRAPH}\n\n{CAP023_ALERT_BOUNDARY}");
    assert!(
        cap023_evidence_violation(&repeated_where_frozen).is_none(),
        "the frozen accepted-head/alert boundary may repeat its merge SHA and Actions analysis"
    );
    for wrong_role in [
        format!(
            "{CAP023_EVIDENCE_PARAGRAPH}\n\nCAP-023 accepted base SHA is {}.",
            CAP023_ACCEPTANCE_EVIDENCE[3]
        ),
        format!(
            "{CAP023_EVIDENCE_PARAGRAPH}\n\nCAP-023 merge commit SHA is {}.",
            CAP023_ACCEPTANCE_EVIDENCE[0]
        ),
        format!("{CAP023_EVIDENCE_PARAGRAPH}\n\nCAP-023 candidate SHA is deadbeef."),
        format!("{CAP023_EVIDENCE_PARAGRAPH}\n\nCAP-023 merge commit SHA is deadbeef."),
        format!("{CAP023_EVIDENCE_PARAGRAPH}\n\n## CAP-023\n\nCandidate SHA is deadbeef."),
    ] {
        assert!(
            cap023_evidence_violation(&wrong_role).is_some(),
            "CAP-023 Git identities must not be assigned conflicting roles"
        );
    }
    let wrapped_list = format!(
        "- {}",
        CAP023_EVIDENCE_PARAGRAPH.replace(" Candidate push CI", "\n  Candidate push CI")
    );
    assert!(
        cap023_evidence_violation(&wrapped_list).is_none(),
        "a true continuation of one list item must remain one evidence paragraph"
    );
    let (evidence_head, evidence_tail) = CAP023_EVIDENCE_PARAGRAPH
        .split_once(" Candidate push CI")
        .expect("canonical CAP-023 evidence split point");
    let sibling_items = format!("- {evidence_head}\n- Candidate push CI{evidence_tail}");
    assert!(
        cap023_evidence_violation(&sibling_items).is_some(),
        "sibling list items must not reconstruct the canonical evidence paragraph"
    );
    for rejected in [
        format!("```text\n{CAP023_EVIDENCE_PARAGRAPH}\n```"),
        format!("<!-- {CAP023_EVIDENCE_PARAGRAPH} -->"),
        format!("~~~text\n```\n{CAP023_EVIDENCE_PARAGRAPH}\n```\n~~~"),
        format!("    {CAP023_EVIDENCE_PARAGRAPH}"),
        format!("{CAP023_EVIDENCE_PARAGRAPH}\n\n{CAP023_EVIDENCE_PARAGRAPH}"),
        format!("{CAP023_EVIDENCE_PARAGRAPH}\n\nCandidate job 94407178006 also passed."),
        CAP023_EVIDENCE_PARAGRAPH.replace("Candidate push CI", "Candidate PR CI"),
        CAP023_EVIDENCE_PARAGRAPH.replace(
            "31687464571`, PR CI `31687585904",
            "31687585904`, PR CI `31687464571",
        ),
        CAP023_EVIDENCE_PARAGRAPH.replace("` all pass.", "` passed."),
    ] {
        assert!(cap023_evidence_violation(&rejected).is_some(), "{rejected}");
    }
    let raw_plus_fenced =
        format!("{CAP023_EVIDENCE_PARAGRAPH}\n\n```text\n{CAP023_EVIDENCE_PARAGRAPH}\n```");
    assert!(
        cap023_evidence_violation(&raw_plus_fenced).is_none(),
        "fenced duplicate evidence must not count"
    );
    let canonical_contract_surface = [
        CAP023_ZERO_PRODUCTION_BOUNDARY,
        CAP023_APPLICATION_BOUNDARY,
        CAP023_ORACLE_BOUNDARY,
        CAP023_EXCLUSION_BOUNDARY,
        CAP023_HISTORY_BOUNDARY,
        CAP023_CLASSIFICATION_BOUNDARY,
        CAP023_ALERT_BOUNDARY,
        CAP023_MILESTONE_BOUNDARY,
        POST_CAP023_DECISION_CONTRACTS[0],
        POST_CAP023_DECISION_CONTRACTS[1],
        POST_CAP023_DECISION_CONTRACTS[2],
        POST_CAP023_DECISION_CONTRACTS[3],
        POST_CAP023_DECISION_CONTRACTS[4],
        POST_CAP023_DECISION_CONTRACTS[5],
        POST_CAP023_DECISION_CONTRACTS[6],
        POST_CAP023_DECISION_CONTRACTS[7],
        POST_CAP023_DECISION_CONTRACTS[8],
    ]
    .join("\n\n");
    assert!(
        cap023_product_violation(&canonical_contract_surface).is_none(),
        "canonical CAP-023 boundaries/decisions must remain within the product contract: {:?}",
        cap023_product_violation(&canonical_contract_surface)
    );
    assert!(
        cap023_status_violation(&canonical_contract_surface).is_none(),
        "canonical CAP-023 boundaries/decisions must not contradict accepted status/oracles: {:?}",
        cap023_status_violation(&canonical_contract_surface)
    );
    assert!(
        cap023_milestone_violation(&canonical_contract_surface).is_none(),
        "canonical CAP-023 boundaries/decisions must preserve milestone truth: {:?}",
        cap023_milestone_violation(&canonical_contract_surface)
    );
    assert!(
        post_cap023_readiness_promotion_violation(&canonical_contract_surface).is_none(),
        "canonical post-CAP-023 decisions must keep rank 2/3 at readiness scope: {:?}",
        post_cap023_readiness_promotion_violation(&canonical_contract_surface)
    );
    let decision_list = POST_CAP023_DECISION_CONTRACTS
        .iter()
        .enumerate()
        .map(|(index, contract)| format!("{}. {contract}", index + 1))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        ordered_decision_records_violation(&decision_list, &POST_CAP023_DECISION_CONTRACTS)
            .is_none(),
        "canonical numbered decisions must remain nine distinct records"
    );
    let tabbed_decision_list = POST_CAP023_DECISION_CONTRACTS
        .iter()
        .enumerate()
        .map(|(index, contract)| format!("{}.\t{contract}", index + 1))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        ordered_decision_records_violation(&tabbed_decision_list, &POST_CAP023_DECISION_CONTRACTS)
            .is_none(),
        "tab-delimited CommonMark decisions must remain nine distinct records"
    );
    assert!(
        post_cap023_successor_order_violation(&decision_list).is_none(),
        "canonical post-CAP-023 decisions must not contradict their own ranking: {:?}",
        post_cap023_successor_order_violation(&decision_list)
    );
    let (decision_head, decision_tail) = POST_CAP023_DECISION_CONTRACTS[0]
        .split_once(" After rank 1")
        .expect("first CAP-023 decision split point");
    let split_decision = format!(
        "1. {decision_head}\n2. After rank 1{decision_tail}\n{}",
        POST_CAP023_DECISION_CONTRACTS[1..]
            .iter()
            .enumerate()
            .map(|(index, contract)| format!("{}. {contract}", index + 3))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        ordered_decision_records_violation(&split_decision, &POST_CAP023_DECISION_CONTRACTS)
            .is_some(),
        "sibling list items must not reconstruct one canonical decision"
    );
    let duplicate_decision = format!("{decision_list}\n10. {}", POST_CAP023_DECISION_CONTRACTS[8]);
    assert!(
        ordered_decision_records_violation(&duplicate_decision, &POST_CAP023_DECISION_CONTRACTS)
            .is_some(),
        "duplicate decisions must fail exact record cardinality"
    );
    let hidden_decisions = format!("```text\n{decision_list}\n```");
    assert!(
        ordered_decision_records_violation(&hidden_decisions, &POST_CAP023_DECISION_CONTRACTS)
            .is_some(),
        "fenced decisions must not count as public contract records"
    );
    let commented_decisions = format!("<!-- {decision_list} -->");
    assert!(
        ordered_decision_records_violation(&commented_decisions, &POST_CAP023_DECISION_CONTRACTS)
            .is_some(),
        "comment-hidden decisions must not count as public contract records"
    );
    let ten_digit_decisions = POST_CAP023_DECISION_CONTRACTS
        .iter()
        .map(|contract| format!("1234567890. {contract}"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        ordered_decision_records_violation(&ten_digit_decisions, &POST_CAP023_DECISION_CONTRACTS)
            .is_some(),
        "ten-digit prefixes are not CommonMark ordered-list markers"
    );
    let interrupted_decisions = format!(
        "Visible prose continues\n2. {}\n{}",
        POST_CAP023_DECISION_CONTRACTS[0],
        POST_CAP023_DECISION_CONTRACTS[1..]
            .iter()
            .enumerate()
            .map(|(index, contract)| format!("{}. {contract}", index + 3))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        ordered_decision_records_violation(&interrupted_decisions, &POST_CAP023_DECISION_CONTRACTS)
            .is_some(),
        "a non-1 ordered marker cannot interrupt a CommonMark paragraph"
    );

    for accepted in [
        "CAP-023 is an accepted zero-production inference product, not a general inference capability.",
        "CAP023 changes no parser, language profile, checked IR, verifier, backend, or ABI.",
        "CAP 023 adds no general activation, ReLU, argmax, tensor, matrix, or record capability.",
        "CAP-023 proves a fixed ReLU-and-argmax application product.",
        "Stop rank 2 before implementation if CAP-023 would require general CopyData or inference.",
        "CAP-023 supplies no performance evidence or accelerator execution.",
        "CAP-023 evidence is not compiler production or benchmark work.",
    ] {
        assert!(cap023_product_violation(accepted).is_none(), "{accepted}");
    }
    for rejected in [
        "CAP-023 adds general inference capability.",
        "CAP-023 adds parser support.",
        "CAP-023 changes backend behavior.",
        "CAP023 adds ReLU support.",
        "CAP 023 enables general argmax.",
        "CAP-023 adds a tensor type and record layout.",
        "CAP-023 implements runtime acquisition and file input.",
        "CAP-023 guarantees stable layout and ABI.",
        "CAP-023 changes compiler production code.",
        "CAP-023 adds no parser changes and enables general inference capability.",
        "CAP-023 provides quantization support.",
        "CAP-023 supplies performance evidence.",
        "CAP&#45;023 adds general inference capability.",
        "CAP&hyphen;023 adds general inference capability.",
        "CAP&minus;023 adds general inference capability.",
        "CAP-023 adds general&Tab;inference capability.",
        "CAP&NewLine;023 adds general inference capability.",
        "CAP-023 adds [general](https://example.invalid) inference capability.",
        "CAP-023 adds general <em>inference</em> capability.",
        "CAP-023 adds general <span\nclass=\"x\">inference</span> capability.",
        "[CAP-023 adds general inference capability]:",
        "&#60;!-- CAP-023 adds general inference capability. -->",
        "CAP-023 is not without general inference capability.",
        "CAP-023 is no longer without general inference capability.",
        "CAP-023 adds not only general inference capability but also quantization.",
        "CAP-023 adds a new profile.",
        "CAP-023 profile is supported.",
        "CAP-023 supports inference.",
        "CAP-023 implements argmax.",
        "CAP-023 enables ReLU.",
        "CAP-023 provides tensor operations.",
        "CAP-023 is classified END_TO_END.",
        "CAP-023 is STABLE.",
        "CAP-023 is the latest compiler/profile widening.",
        "CAP-023 widens exact-i32-array-v0.",
        "Stop if rank 2 requires inference. CAP-023 adds general inference capability.",
        "### CAP-023\n\nGeneral inference is now supported.",
        "## CAP-023\n\n### Scope\n\nGeneral inference is now supported.",
        "## CAP-023\n\n---\n\nGeneral inference is now supported.",
        "> **CAP-023 accepted product gate:**\n>\n> General inference is now supported.",
        "CAP-023\n=======\n\nGeneral inference is now supported.",
        "<h2>CAP-023</h2>\n\nGeneral inference is now supported.",
        "<h2>\nCAP-023\n</h2>\n\nGeneral inference is now supported.",
        "<h2 class=\"cap\">\nCAP-023\n</h2>\n\nGeneral inference is now supported.",
        "<h2\nclass=\"cap\">\nCAP-023\n</h2>\n\nGeneral inference is now supported.",
        "CAP-023 adds general\n    inference capability.",
        "CAP-023 adds [general](https://example.invalid/path) inference capability.",
        "CAP-023 adds [general](CAP-023 adds general inference capability) inference capability.",
        "CAP-023 adds general <div>inference</div> capability.",
        "[safe][CAP-023 adds general inference capability]",
        "[CAP-023 adds general inference capability]: (",
        "<em !\nCAP-023 adds general inference capability.>",
        "<em title='unterminated\nCAP-023 adds general inference capability.>",
        "CAP-023 supplies performance <span title=\">\">evidence</span>.",
        "CAP-023 supplies [performance][p] evidence.\n\n[p]: https://example.invalid",
        "R&D notes: CAP&hyphen;023 adds general inference capability.",
    ] {
        assert!(cap023_product_violation(rejected).is_some(), "{rejected}");
    }
    let list_continuation = "- Accepted CAP-023.\n  CAP-023 adds general inference capability.";
    assert!(
        cap023_product_violation(list_continuation).is_some(),
        "rendered list continuations must remain visible to the CAP-023 claim scanner"
    );
    let hidden_overclaim = "<!-- CAP&hyphen;023 adds general inference capability. -->";
    assert!(
        cap023_product_violation(hidden_overclaim).is_none(),
        "comment-hidden text must not become public truth"
    );
    assert!(
        cap023_product_violation(
            "[ref]: https://example/CAP-023-adds-general-inference-capability \"source\""
        )
        .is_none(),
        "a valid reference definition with a title is not rendered public truth"
    );
    for hidden_reference in [
        "[ref]: /url\n  \"CAP-023 adds general inference capability.\"",
        "[ref]: /url \"CAP-023 adds\ngeneral inference capability.\"",
    ] {
        assert!(
            cap023_product_violation(hidden_reference).is_none(),
            "a valid multiline reference definition is not rendered public truth: {hidden_reference}"
        );
    }
    let multiline_code_overclaim = "`start\n<!--\nCAP-023 adds general inference capability.\n`";
    assert!(
        cap023_product_violation(multiline_code_overclaim).is_some(),
        "multiline inline-code claims must remain visible to the CAP-023 scanner"
    );

    for accepted in [
        "CAP-023 is accepted, not a candidate.",
        "No new CAP-023 alert exists.",
        "CAP-023 ordinary result is [1, 122, 167, 135, 181, 4940, 5573, 1].",
        "CAP-023 wrapping result is [1, -24, 18, 2147483623, 0, -37, 2147483641, 1].",
        "CAP-023 activation-boundary result is [1, -3, 0, 0, 0, 5, 4, 0].",
        "CAP-023 tie result is [1, 1, 2, 1, 2, 3, 3, 0].",
        "CAP-023 malformed result is [0, 0, 0, 0, 0, 0, 0, 0].",
        "CAP-023 header is [2, 3, 2] and sentinel is 91.",
        "CAP-023 ordinary and wrapping results are [1, 122, 167, 135, 181, 4940, 5573, 1] and [1, -24, 18, 2147483623, 0, -37, 2147483641, 1], respectively.",
        "CAP-023 header is [2, 3, 2], and ordinary result is [1, 122, 167, 135, 181, 4940, 5573, 1].",
        "Stop and rerank rank 1 if CAP-023 is not accepted.",
        "Stop if the CAP-023 ordinary result is [1, 122, 167, 135, 181, 4940, 5572, 1].",
        "Before acceptance, CAP-023 was a local candidate.",
        "CAP-023 was an unpublished candidate prior to PR #62.",
    ] {
        assert!(cap023_status_violation(accepted).is_none(), "{accepted}");
    }
    for rejected in [
        "CAP-023 remains a local candidate.",
        "CAP-023 acceptance is pending.",
        "CAP-023 merge CI failed.",
        "A new CAP-023 alert surfaced.",
        "CAP-023 is no longer zero production.",
        "CAP-023 ordinary result is [1, 122, 167, 135, 181, 4940, 5572, 1].",
        "CAP-023 wrapping result is [1, -24, 18, 2147483623, 0, -37, 2147483640, 1].",
        "CAP-023 activation-boundary result is [1, -3, 1, 0, 0, 5, 4, 0].",
        "CAP-023 tie result is [1, 1, 2, 1, 2, 3, 3, 1].",
        "CAP-023 malformed result is [1, 0, 0, 0, 0, 0, 0, 0].",
        "CAP-023 header is [2, 3, 1].",
        "CAP-023 exits 92.",
        "CAP-023 ordinary result differs from the accepted oracle.",
        "CAP-023 ordinary result is 42.",
        "CAP-023 malformed result is [1].",
        "CAP-023 header is 1.",
        "CAP-023 ordinary and wrapping results are respectively [1, -24, 18, 2147483623, 0, -37, 2147483641, 1] and [1, 122, 167, 135, 181, 4940, 5573, 1].",
        "CAP-021 is historical, but the ordinary result is [1, 122, 167, 135, 181, 4940, 5572, 1] for CAP-023.",
        "CAP-023 is not merged.",
        "CAP-023 remains a draft.",
        "CAP-023 alert #5 is open.",
        "CAP-023 alert #5 is open only on Linux.",
        "A pre-existing alert remains, while CAP-023 alert #5 is open.",
        "CAP-023 ordinary result is [1, 122, 167, 135, 181, 4940, 5573, 1], except its last element is 2.",
        "### CAP-023\n\nThe ordinary result is [1, 122, 167, 135, 181, 4940, 5572, 1].",
        "CAP-023 ordinary result is 0x0.",
        "CAP-023 header is 0x1.",
        "CAP-023 result [1, -24, 18, 2147483623, 0, -37, 2147483641, 1] is the ordinary oracle.",
        "The CAP-023 ordinary result is the wrapping result.",
        "CAP-023 has no ordinary oracle.",
        "The CAP-023 wrapping result is unavailable.",
        "CAP-023 ordinary result no longer equals the accepted oracle.",
        "CAP-023 succeeds but stdout is nonempty.",
        "CAP-023 native stderr contains diagnostics.",
        "CAP-023 preserves only 139 source lanes.",
        "CAP-023 makes six by-value calls.",
        "CAP-023 merge CI is broken.",
        "CAP-023 acceptance was rejected.",
        "CAP-023 checks are pending.",
        "CAP-023 is no longer the current public master.",
        "CAP-023 has been superseded.",
        "CAP-023 remains pending review.",
        "CAP-023 pre-existing alert #4 is closed.",
        "CAP-023 alert #4 was resolved.",
        "CAP-023 now has zero open alerts.",
        "CAP-023 Python analysis contains one result.",
        "CAP-023 Rust analysis is nonzero.",
        "The CAP-023 PR-only aggregate exists on the default branch.",
        "CAP-023 is not only a candidate.",
        "CAP-023 is not only unaccepted.",
        "Before acceptance CAP-023 was a local candidate, but it has now been superseded.",
    ] {
        assert!(cap023_status_violation(rejected).is_some(), "{rejected}");
    }

    for accepted in [
        "CAP-021 was the accepted public master before CAP-023.",
        "CAP-021 remains an accepted historical product gate.",
        "CAP-022 remains a mandatory NO IMPLEMENTATION stop.",
        "CAP-021 remains the former public master.",
        "CAP-022 still has historical status as the mandatory stop.",
        "CAP-019 remains the latest compiler/profile widening.",
        "Current public master is CAP-023, after CAP-021 and CAP-022.",
    ] {
        assert!(
            stale_pre_cap023_current_violation(accepted).is_none(),
            "{accepted}"
        );
    }
    for rejected in [
        "CAP-021 is the current public master.",
        "Current public head is CAP 021.",
        "The latest accepted capability is CAP-021.",
        "CAP-022 is the current public baseline.",
        "Current project status is CAP&hyphen;022.",
        "CAP-022 implementation is underway.",
        "CAP-022 implementation is scheduled.",
        "CAP-022 implementation is planned.",
        "CAP-022 implementation begins next week.",
        "CAP-022 is an accepted runtime capability.",
        "CAP-021 remains the accepted public master.",
        "CAP-022 is still the public baseline.",
        "CAP-021 is the public master.",
        "CAP-022 is the public baseline.",
        "CAP-021 holds the public-head role.",
        "CAP-021 remains the public master because CAP-022 was a stopped proposal.",
    ] {
        assert!(
            stale_pre_cap023_current_violation(rejected).is_some(),
            "{rejected}"
        );
    }

    for accepted in [
        "Rank 2 composition remains readiness and a red probe only, not implementation.",
        "Rank 3 quantization remains a readiness probe only.",
        "Rank 2 may not proceed to implementation.",
        "CopyData application-profile composition remains readiness only.",
        "### Rank 2 — CopyData application-profile composition\n\nImplementation is not underway.",
    ] {
        assert!(
            post_cap023_readiness_promotion_violation(accepted).is_none(),
            "{accepted}"
        );
    }
    for rejected in [
        "Rank 2 composition implementation is underway.",
        "Rank 2 is authorized for implementation.",
        "Rank 3 quantized-kernel implementation has begun.",
        "Quantization is approved for implementation.",
        "CopyData application-profile composition implementation is underway.",
        "Recursive CopyData composition implementation is underway.",
        "The CopyData application profile is now being implemented.",
        "### Rank 2 — CopyData application-profile composition\n\nImplementation is underway.",
        "Accepted CAP-021 evidence remains historical; Recursive CopyData composition implementation is underway.",
        "Accepted CAP-021 evidence remains historical, while CopyData application-profile composition implementation is underway.",
        "Rank 2 implementation is scheduled.",
        "Rank 3 implementation is planned.",
        "Rank 2 implementation is not only underway.",
        "Rank 3 is not only approved for implementation.",
    ] {
        assert!(
            post_cap023_readiness_promotion_violation(rejected).is_some(),
            "{rejected}"
        );
    }

    for accepted in [
        "Accepted-head CAP-023 evidence ranks first.",
        "CopyData application-profile composition ranks second.",
        "Quantization ranks third.",
        "Quantization does not rank first.",
        "CopyData application-profile composition and quantization remain ranks 2 and 3.",
        "CAP-023 selects the highest signed logit; quantization remains excluded.",
        "Historically, quantization ranked first before CAP-023; it now ranks third.",
    ] {
        assert!(
            post_cap023_successor_order_violation(accepted).is_none(),
            "{accepted}"
        );
    }
    for rejected in [
        "Quantization ranks first.",
        "Accepted-head CAP-023 evidence ranks second.",
        "CopyData application-profile composition ranks third.",
        "Quantization precedes CAP-023 artifact evidence.",
        "Runtime byte/file acquisition ranks first.",
        "Recursive arrays rank third.",
        "CopyData application-profile composition and quantization rank 3 and 2.",
        "Quantization is now the top priority.",
        "1. Quantization\n2. CopyData application-profile composition\n3. Accepted-head CAP-023 evidence",
        "01. Quantization\n02. CopyData application-profile composition\n03. Accepted-head CAP-023 evidence",
        "| Priority | Capability |\n|---|---|\n| 1 | Quantization |\n| 2 | CopyData application-profile composition |\n| 3 | Accepted-head CAP-023 evidence |",
        "Quantization ranks fourth.",
        "Quantization does not only rank first.",
        "### Quantization\n\nIt ranks first.",
        "### CopyData application-profile composition\n\nThis is rank 3.",
        "Historically quantization ranked first, but it still ranks first.",
        "| Priority | Capability |\n|---|---|\n| 4 | Quantization |\n| 2 | CopyData application-profile composition |\n| 1 | Accepted-head CAP-023 evidence |",
    ] {
        assert!(
            post_cap023_successor_order_violation(rejected).is_some(),
            "{rejected}"
        );
    }

    for accepted in [
        CAP023_MILESTONE_BOUNDARY,
        "Milestone 3 remains open.",
        "The selected bounded Milestone 2 exit gate is met.",
        "The Milestone 0/1 exits were closed.",
        "Broader residuals remain excluded rather than milestone-complete.",
        "The selected Milestone 2 exit gate is met without making the broader milestone complete.",
        "The selected Milestone 2 is complete.",
        "The required post-CAP-004 milestone-gap audit and three-gap ranking are complete.",
        "Stop if Milestone 3 ships without performance evidence.",
        "The selected Milestone 2 is complete.",
    ] {
        assert!(cap023_milestone_violation(accepted).is_none(), "{accepted}");
    }
    for rejected in [
        "Milestone 3 is complete.",
        "All milestone ambitions are met.",
        "The broader Milestone 2 ambition is complete.",
        "Milestone 2 is complete.",
        "CAP-023 completes the milestone.",
        "All milestone ambitions are fulfilled.",
        "Milestone 3 has shipped.",
        "The milestone-gap audit is complete and Milestone 3 has shipped.",
        "M3 is complete.",
        "M2 is fully complete.",
        "Milestone 3 has concluded.",
        "The CAP-023 product proves Milestone 3 is complete.",
        "Artifact evidence shows the broader Milestone 2 ambition is finished.",
        "The milestone-gap audit says Milestone 3 is complete.",
        "## Milestones\n\nAll broader ambitions are complete.",
        "### Milestone 3\n\nIt has shipped.",
    ] {
        assert!(cap023_milestone_violation(rejected).is_some(), "{rejected}");
    }

    const SEPARATOR: &str = "|---:|---|---:|---:|---:|---:|---:|---:|---:|";
    let ranking = format!(
        "{POST_CAP023_RANKING_HEADER}\n{SEPARATOR}\n{}\n{}\n{}\n",
        POST_CAP023_RANKING_ROWS[0], POST_CAP023_RANKING_ROWS[1], POST_CAP023_RANKING_ROWS[2]
    );
    assert_post_cap023_ranking_table("CAP-023 ranking fixture", &ranking);
    assert_post_cap023_successor_order("CAP-023 ranking fixture", &ranking);
    let alternate = ranking.replace(
        "| 4 | 5 | 5 | 5 | 5 | 4 | 28 |",
        "| 5 | 5 | 5 | 5 | 5 | 4 | 29 |",
    );
    assert!(
        std::panic::catch_unwind(|| {
            assert_post_cap023_ranking_table("mutated CAP-023 ranking fixture", &alternate)
        })
        .is_err(),
        "an alternate current ranking must fail"
    );
    let reordered = format!(
        "{POST_CAP023_RANKING_HEADER}\n{SEPARATOR}\n{}\n{}\n{}\n",
        POST_CAP023_RANKING_ROWS[1], POST_CAP023_RANKING_ROWS[0], POST_CAP023_RANKING_ROWS[2]
    );
    assert!(
        std::panic::catch_unwind(|| {
            assert_post_cap023_ranking_table("reordered CAP-023 ranking fixture", &reordered)
        })
        .is_err(),
        "a reordered current ranking must fail"
    );
    let alternate_header = ranking.replace("Favorable risk", "Risk");
    assert!(
        std::panic::catch_unwind(|| {
            assert_post_cap023_ranking_table("alternate CAP-023 header fixture", &alternate_header)
        })
        .is_err(),
        "an alternate current ranking header must fail"
    );

    let matrix = format!(
        "## Language features\n\n| Feature | Class |\n|---|---|\n| Selected exact profile | END_TO_END |\n\n## Compiler, tooling, and ecosystem surfaces\n\n## Backend summary\n\n| Backend/surface | Selectable | IR transform | Object | Link | Real execution | Numerical checks | Performance evidence | Class |\n|---|---|---|---|---|---|---|---|---|\n{CAP023_CPU_MATRIX_ROW}\n\n## Evidence notes\n"
    );
    assert!(cap023_matrix_violation(&matrix).is_none());
    for rejected in [
        matrix.replace(CAP023_CPU_MATRIX_ROW, CAP021_CPU_MATRIX_ROW),
        matrix.replace("| CPU |", "| CPU inference |"),
        matrix.replace("| PARTIAL |", "| PARTIAL (CAP-023) |"),
        matrix.replace(
            "## Compiler, tooling, and ecosystem surfaces",
            "| CAP-023 | PARTIAL |\n\n## Compiler, tooling, and ecosystem surfaces",
        ),
        matrix.replace(
            "\n\n## Evidence notes",
            &format!("\n{CAP023_CPU_MATRIX_ROW}\n\n## Evidence notes"),
        ),
        format!("{matrix}\n| Feature | Status |\n|---|---|\n| CAP-023 profile | Supported |\n"),
    ] {
        assert!(cap023_matrix_violation(&rejected).is_some(), "{rejected}");
    }
}

fn assert_cap024_contract_mutation_fixtures() {
    let canonical_evidence_surface = [
        CAP024_EVIDENCE_PARAGRAPH,
        CAP024_CURRENT_HEAD_BOUNDARY,
        CAP024_BUNDLE_BOUNDARY,
        CAP024_ALERT_BOUNDARY,
    ]
    .join("\n\n");
    assert!(
        cap024_evidence_violation(&canonical_evidence_surface).is_none(),
        "the canonical CAP-024 evidence/identity surface must pass: {:?}",
        cap024_evidence_violation(&canonical_evidence_surface)
    );
    let (evidence_head, evidence_tail) = CAP024_EVIDENCE_PARAGRAPH
        .split_once(" Candidate push CI")
        .expect("canonical CAP-024 evidence split point");
    for rejected in [
        format!(
            "```text\n{CAP024_EVIDENCE_PARAGRAPH}\n```\n\n{CAP024_CURRENT_HEAD_BOUNDARY}\n\n{CAP024_BUNDLE_BOUNDARY}\n\n{CAP024_ALERT_BOUNDARY}"
        ),
        format!(
            "<!-- {CAP024_EVIDENCE_PARAGRAPH} -->\n\n{CAP024_CURRENT_HEAD_BOUNDARY}\n\n{CAP024_BUNDLE_BOUNDARY}\n\n{CAP024_ALERT_BOUNDARY}"
        ),
        format!(
            "- {evidence_head}\n- Candidate push CI{evidence_tail}\n\n{CAP024_CURRENT_HEAD_BOUNDARY}\n\n{CAP024_BUNDLE_BOUNDARY}\n\n{CAP024_ALERT_BOUNDARY}"
        ),
        format!("{canonical_evidence_surface}\n\n{CAP024_EVIDENCE_PARAGRAPH}"),
        format!(
            "{canonical_evidence_surface}\n\nCAP-024 accepted base SHA is {}.",
            CAP024_ACCEPTANCE_EVIDENCE[3]
        ),
        format!(
            "{canonical_evidence_surface}\n\nCAP-024 merge commit SHA is {}.",
            CAP024_ACCEPTANCE_EVIDENCE[0]
        ),
        format!("{canonical_evidence_surface}\n\nCAP-024 candidate SHA is deadbeef."),
        format!("{canonical_evidence_surface}\n\nCAP-024 tree identity is deadbeef."),
        format!("{canonical_evidence_surface}\n\nCAP-024 base identifier is deadbeef."),
        format!("{canonical_evidence_surface}\n\nCAP-024 merge exact SHA is deadbeef."),
        format!("{canonical_evidence_surface}\n\nCAP-024 evidence run is deadbeef."),
        format!("{canonical_evidence_surface}\n\nCAP-024 Linux job is deadbeef."),
        format!("{canonical_evidence_surface}\n\nCAP-024 Actions analysis is deadbeef."),
        format!("{canonical_evidence_surface}\n\nCAP-024 unknown job is deadbeef."),
        format!("{canonical_evidence_surface}\n\nCAP-024 canonical manifest SHA is deadbeef."),
        canonical_evidence_surface.replace("Candidate push CI", "Candidate PR CI"),
        canonical_evidence_surface.replace(
            "31764763341`, PR CI `31764765501",
            "31764765501`, PR CI `31764763341",
        ),
    ] {
        assert!(cap024_evidence_violation(&rejected).is_some(), "{rejected}");
    }
    let raw_plus_fenced =
        format!("{canonical_evidence_surface}\n\n```text\n{CAP024_EVIDENCE_PARAGRAPH}\n```");
    assert!(
        cap024_evidence_violation(&raw_plus_fenced).is_none(),
        "fenced duplicate CAP-024 evidence must not count"
    );

    let canonical_contract_surface = [
        CAP024_CURRENT_HEAD_BOUNDARY,
        CAP024_ZERO_PRODUCTION_BOUNDARY,
        CAP024_CLASSIFICATION_BOUNDARY,
        CAP024_BUNDLE_BOUNDARY,
        CAP024_ALERT_BOUNDARY,
        CAP024_MILESTONE_BOUNDARY,
    ]
    .into_iter()
    .chain(POST_CAP024_DECISION_CONTRACTS)
    .collect::<Vec<_>>()
    .join("\n\n");
    assert!(
        cap024_product_violation(&canonical_contract_surface).is_none(),
        "canonical CAP-024 boundaries/decisions must remain evidence-only: {:?}",
        cap024_product_violation(&canonical_contract_surface)
    );
    assert!(
        cap024_status_violation(&canonical_contract_surface).is_none(),
        "canonical CAP-024 boundaries/decisions must retain accepted status: {:?}",
        cap024_status_violation(&canonical_contract_surface)
    );
    assert!(
        post_cap024_readiness_promotion_violation(&canonical_contract_surface).is_none(),
        "canonical post-CAP-024 decisions must keep all successors at readiness scope: {:?}",
        post_cap024_readiness_promotion_violation(&canonical_contract_surface)
    );

    for accepted in [
        "CAP-024 is accepted, not a candidate.",
        "Before acceptance, CAP-024 was an unpublished candidate.",
        "No new CAP-024 alert exists.",
        "CAP-024 does not await acceptance.",
        "CAP-024 is not awaiting acceptance.",
    ] {
        assert!(cap024_status_violation(accepted).is_none(), "{accepted}");
    }
    for rejected in [
        "CAP-024 remains a candidate.",
        "CAP-024 acceptance is pending.",
        "CAP-024 acceptance was revoked.",
        "CAP-024 merge CI failed.",
        "CAP-024 is not the current public master.",
        "CAP-024 is no longer the current public master.",
        "CAP-024 has been superseded.",
        "CAP-024 remains unmerged.",
        "CAP-024 is unpublished.",
        "CAP-024 is a proposed checkpoint.",
        "CAP-024 is local-only.",
        "CAP-024 is not yet accepted.",
        "CAP-024 has not been accepted.",
        "CAP-024 has not yet been accepted.",
        "CAP-024 awaits acceptance.",
        "CAP-024 is awaiting acceptance.",
        "A new CAP-024 alert #5 is open.",
        "CAP-024 alert #5 exists.",
    ] {
        assert!(cap024_status_violation(rejected).is_some(), "{rejected}");
    }

    for accepted in [
        "CAP-024 adds no compiler production, profile, product, or performance capability.",
        "CAP-024 is an evidence checkpoint, not a product capability.",
        "CAP-024 is zero-production evidence.",
        "CAP-024 classification is absent and it adds no classification.",
        "CAP-024 milestone progress remains partial.",
        "CAP-024 supplies no performance evidence.",
        "CAP-024 changes no production source code.",
        "CAP-024 does not change the compiler.",
        "CAP-024 makes no compiler edits.",
        "CAP-024 does not modify code generation.",
        "CAP-024 adds no runtime/file ingestion, activation, tensor, serialization, quantization, or conversion capability.",
    ] {
        assert!(cap024_product_violation(accepted).is_none(), "{accepted}");
    }
    for rejected in [
        "Before rank 1, CAP-024 adds compiler production.",
        "After rank 1, CAP-024 changes backend behavior.",
        "CAP-024 adds a profile.",
        "CAP-024 changes production source code.",
        "CAP-024 changes the compiler.",
        "CAP-024 makes compiler edits.",
        "CAP-024 modifies code generation.",
        "CAP-024 changes codegen.",
        "CAP-024 adds a product capability.",
        "CAP-024 is the latest product checkpoint.",
        "CAP-024 supplies performance evidence.",
        "CAP-024 is non-zero-production.",
        "CAP-024 is not zero-production.",
        "CAP-024 is no longer zero-production.",
        "CAP-024 is STABLE.",
        "CAP-024 is END_TO_END.",
        "CAP-024 is classified PARTIAL.",
        "CAP-024 is PARSED_ONLY.",
        "CAP-024 is EXPERIMENTAL.",
        "CAP-024 is DESIGNED.",
        "CAP-024 adds a classification.",
        "CAP-024 adds general inference capability.",
        "CAP-024 adds runtime ingestion.",
        "CAP-024 supports runtime byte/file acquisition.",
        "CAP-024 enables file input.",
        "CAP-024 adds activation support.",
        "CAP-024 implements ReLU and argmax.",
        "CAP-024 supports tensors, matrices, records, and recursive arrays.",
        "CAP-024 adds serialization.",
        "CAP-024 adds quantization support and conversion semantics.",
        "### CAP-024\n\nGeneral inference is now supported.",
        "```text\nCAP-024 is an evidence checkpoint.\n```\n\nCAP-024 adds a profile.",
    ] {
        assert!(cap024_product_violation(rejected).is_some(), "{rejected}");
    }
    let prefixed_canonical_boundary =
        format!("CAP-024 adds a profile. {CAP024_ZERO_PRODUCTION_BOUNDARY}");
    assert!(
        cap024_product_violation(&prefixed_canonical_boundary).is_some(),
        "a contradictory prefix must not inherit canonical-record exemption"
    );

    for accepted in [
        "CAP-023 is an accepted historical product checkpoint, not the current public master.",
        "CAP-023 was the current public master before CAP-024.",
        "CAP-023 remains the latest product checkpoint.",
    ] {
        assert!(
            stale_cap023_current_head_violation(accepted).is_none(),
            "{accepted}"
        );
    }
    for rejected in [
        "CAP-023 is the current public master.",
        "Latest accepted public master is protected CAP-023 merge.",
        "The accepted baseline remains CAP-023.",
        "CAP-023 was historical, but it is now the current public head.",
    ] {
        assert!(
            stale_cap023_current_head_violation(rejected).is_some(),
            "{rejected}"
        );
    }

    let decision_list = POST_CAP024_DECISION_CONTRACTS
        .iter()
        .enumerate()
        .map(|(index, contract)| format!("{}. {contract}", index + 1))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        ordered_decision_records_violation(&decision_list, &POST_CAP024_DECISION_CONTRACTS)
            .is_none(),
        "canonical CAP-024 decisions must remain nine ordered rendered records"
    );
    let (decision_head, decision_tail) = POST_CAP024_DECISION_CONTRACTS[0]
        .split_once(" After rank 1 readiness")
        .expect("first CAP-024 decision split point");
    let split_decision = format!(
        "1. {decision_head}\n2. After rank 1 readiness{decision_tail}\n{}",
        POST_CAP024_DECISION_CONTRACTS[1..]
            .iter()
            .enumerate()
            .map(|(index, contract)| format!("{}. {contract}", index + 3))
            .collect::<Vec<_>>()
            .join("\n")
    );
    for rejected in [
        split_decision,
        format!("```text\n{decision_list}\n```"),
        format!("<!-- {decision_list} -->"),
        format!("{decision_list}\n10. {}", POST_CAP024_DECISION_CONTRACTS[8]),
    ] {
        assert!(
            ordered_decision_records_violation(&rejected, &POST_CAP024_DECISION_CONTRACTS)
                .is_some(),
            "{rejected}"
        );
    }

    for accepted in [
        "Rank 1 composition remains readiness and red-probe only, not implementation.",
        "Rank 2 dynamic collections remain readiness only.",
        "Rank 3 quantization remains a red probe only.",
        "Owned String remains a readiness-only collection probe, not implementation-ready.",
    ] {
        assert!(
            post_cap024_readiness_promotion_violation(accepted).is_none(),
            "{accepted}"
        );
    }
    for rejected in [
        "Rank 1 composition implementation is underway.",
        "CopyData application-profile composition is authorized for implementation.",
        "Rank 1 composition is ready for implementation.",
        "Rank 2 dynamic collection implementation has begun.",
        "Owned collection/streaming work is approved to proceed.",
        "Owned Vec<T> foundation has implementation approval.",
        "Vec implementation is underway.",
        "Rank 3 quantized-kernel implementation is planned.",
        "Quantization is cleared for implementation.",
        "Rank 3 quantization is implementation-ready.",
    ] {
        assert!(
            post_cap024_readiness_promotion_violation(rejected).is_some(),
            "{rejected}"
        );
    }

    for accepted in [
        "CopyData application-profile composition ranks first.",
        "Owned dynamic collection/streaming foundation ranks second.",
        "Owned Vec<T> foundation ranks second.",
        "Quantization ranks third.",
        "Historically accepted-head CAP-023 evidence ranked first before CAP-024.",
    ] {
        assert!(
            post_cap024_successor_order_violation(accepted).is_none(),
            "{accepted}"
        );
    }
    for rejected in [
        "Quantization ranks first.",
        "CopyData application-profile composition ranks second.",
        "Owned dynamic collection/streaming foundation ranks third.",
        "Owned Vec<T> foundation ranks first.",
        "Vector collection support ranks third.",
        "Quantization precedes CopyData application-profile composition.",
        "1. Quantization\n2. Owned dynamic collection/streaming foundation\n3. CopyData application-profile composition",
        "| Priority | Capability |\n|---|---|\n| 1 | Quantization |\n| 2 | Owned dynamic collection/streaming foundation |\n| 3 | CopyData application-profile composition |",
    ] {
        assert!(
            post_cap024_successor_order_violation(rejected).is_some(),
            "{rejected}"
        );
    }

    for accepted in [
        CAP016_LOCAL_MODDECL_STOP_BOUNDARY,
        CAP023_ZERO_PRODUCTION_BOUNDARY,
        "CAP-016 remains a mandatory NO IMPLEMENTATION stop.",
        "CAP-022 runtime byte/file acquisition is not authorized for implementation.",
        "Stop if CAP-022 runtime acquisition implementation is attempted.",
        "Historically, CAP-022 ranked first before its mandatory stop.",
    ] {
        assert!(
            stopped_capability_violation(accepted).is_none(),
            "{accepted}"
        );
    }
    for rejected in [
        "CAP-016 ModDecl implementation is underway.",
        "ModDecl implementation is underway.",
        "CAP-016 currently implements ModDecl.",
        "### CAP-016\n\nModule-resolution implementation is authorized.",
        "CAP-016 ranks first for implementation.",
        "CAP-022 runtime byte/file acquisition implementation is planned.",
        "CAP-022 runtime byte/file acquisition is implemented.",
        "Runtime byte/file acquisition implementation is underway.",
        "Runtime byte/file acquisition is authorized for implementation.",
        "### CAP-022\n\nIt is cleared for implementation.",
        "CAP-022 ranks first.",
    ] {
        assert!(
            stopped_capability_violation(rejected).is_some(),
            "{rejected}"
        );
    }

    for accepted in [
        "CORE-090 remains PARTIAL and is not STABLE or END_TO_END.",
        "CORE-090 does not provide general memory safety or complete ownership.",
        "CORE-090 does not establish general memory safety and complete ownership.",
        "CORE-090 leaves projected borrowing, lifetime/drop, stable ABI, and accelerators excluded.",
        "Historically, CORE-090 was never STABLE.",
        "### CORE-090\n\nProjected borrowing remains excluded.",
    ] {
        assert!(
            core090_overclaim_violation(accepted).is_none(),
            "{accepted}"
        );
    }
    for rejected in [
        "CORE-090 is STABLE.",
        "CORE-090 is classified END_TO_END.",
        "CORE-090 provides general memory safety.",
        "CORE-090 establishes complete ownership.",
        "CORE-090 does not provide general memory safety, but it establishes complete ownership.",
        "CORE-090 supports projected borrowing.",
        "CORE-090 implements lifetime and drop semantics.",
        "CORE-090 guarantees a stable ABI and accelerator execution.",
        "### CORE-090\n\nIt provides general memory safety.",
    ] {
        assert!(
            core090_overclaim_violation(rejected).is_some(),
            "{rejected}"
        );
    }

    for accepted in [
        "CAP-024 completed the accepted-head inference reproducibility evidence gate.",
        "Historically, accepted-head CAP-023 inference reproducibility ranked first before CAP-024.",
    ] {
        assert!(
            consumed_cap024_evidence_ranking_violation(accepted).is_none(),
            "{accepted}"
        );
    }
    for rejected in [
        "Inference reproducibility is still priority 1.",
        "Accepted-head CAP-023 artifact-footprint evidence ranks first.",
        "CAP-023 artifact-footprint evidence remains the top-ranked task.",
        "CAP-024 accepted-head evidence is the top priority.",
    ] {
        assert!(
            consumed_cap024_evidence_ranking_violation(rejected).is_some(),
            "{rejected}"
        );
    }

    const SEPARATOR: &str = "|---:|---|---:|---:|---:|---:|---:|---:|---:|";
    let ranking = format!(
        "{POST_CAP024_RANKING_HEADER}\n{SEPARATOR}\n{}\n{}\n{}\n",
        POST_CAP024_RANKING_ROWS[0], POST_CAP024_RANKING_ROWS[1], POST_CAP024_RANKING_ROWS[2]
    );
    assert_post_cap024_ranking_table("CAP-024 ranking fixture", &ranking);
    assert_post_cap024_successor_order("CAP-024 ranking fixture", &ranking);
    let alternate = ranking.replace(
        "| 5 | 5 | 5 | 5 | 2 | 2 | 24 |",
        "| 5 | 5 | 5 | 5 | 3 | 2 | 25 |",
    );
    assert!(
        std::panic::catch_unwind(|| {
            assert_post_cap024_ranking_table("mutated CAP-024 ranking fixture", &alternate)
        })
        .is_err(),
        "an alternate CAP-024 ranking must fail"
    );
    let reordered = format!(
        "{POST_CAP024_RANKING_HEADER}\n{SEPARATOR}\n{}\n{}\n{}\n",
        POST_CAP024_RANKING_ROWS[1], POST_CAP024_RANKING_ROWS[0], POST_CAP024_RANKING_ROWS[2]
    );
    assert!(
        std::panic::catch_unwind(|| {
            assert_post_cap024_ranking_table("reordered CAP-024 ranking fixture", &reordered)
        })
        .is_err(),
        "a reordered CAP-024 ranking must fail"
    );

    let matrix = format!(
        "{ranking}\n## Language features\n\n| Feature | Class |\n|---|---|\n| Selected exact profile | END_TO_END |\n\n## Compiler, tooling, and ecosystem surfaces\n\n## Backend summary\n\n| Backend/surface | Selectable | IR transform | Object | Link | Real execution | Numerical checks | Performance evidence | Class |\n|---|---|---|---|---|---|---|---|---|\n{CAP023_CPU_MATRIX_ROW}\n\n## Evidence notes\n"
    );
    assert!(cap024_matrix_violation(&matrix).is_none());
    for rejected in [
        matrix.replace("| Selected exact profile |", "| CAP-024 selected profile |"),
        matrix.replace("| CPU |", "| CAP-024 CPU |"),
        matrix.replace(CAP023_CPU_MATRIX_ROW, CAP021_CPU_MATRIX_ROW),
        matrix.replace(
            "## Compiler, tooling, and ecosystem surfaces",
            "| CAP-024 | PARTIAL |\n\n## Compiler, tooling, and ecosystem surfaces",
        ),
    ] {
        assert!(cap024_matrix_violation(&rejected).is_some(), "{rejected}");
    }

    let raw_matrix = format!(
        "## Language features\n\n{CAP019_SELECTED_PROFILE_MATRIX_ROW}\n\n## Compiler, tooling, and ecosystem surfaces\n\n## Backend summary\n\n{CAP023_CPU_MATRIX_ROW}\n\n## Evidence notes\n"
    );
    assert!(frozen_cap024_matrix_source_row_violation(&raw_matrix).is_none());
    for rejected in [
        raw_matrix.replace(
            CAP019_SELECTED_PROFILE_MATRIX_ROW,
            &format!("<!-- {CAP019_SELECTED_PROFILE_MATRIX_ROW} -->"),
        ),
        raw_matrix.replace(
            CAP023_CPU_MATRIX_ROW,
            &CAP023_CPU_MATRIX_ROW.replace('|', "&#124;"),
        ),
    ] {
        assert!(
            frozen_cap024_matrix_source_row_violation(&rejected).is_some(),
            "raw source-byte identity must reject comment/entity substitutes: {rejected}"
        );
    }
}

fn has_capability_token(words: &[&str], capability: &str) -> bool {
    let compact = capability.replace('-', "");
    let number = capability
        .strip_prefix("cap-")
        .expect("canonical CAP capability token");
    words
        .iter()
        .any(|word| *word == capability || *word == compact)
        || words.windows(2).any(|pair| pair == ["cap", number])
}

fn assert_cap020_boundaries(document_name: &str, document: &str) {
    let rendered = markdown_outside_fences(document);
    let normalized = normalized_words(&rendered);
    let normalized_lower = normalized.to_ascii_lowercase();
    assert_no_stale_cap019_current_claims(document_name, document);
    assert_no_cap020_overclaims(document_name, document);
    assert!(
        cap020_status_violation(document).is_none(),
        "{document_name} contradicts accepted CAP-020 status or evidence: {:?}",
        cap020_status_violation(document)
    );
    for expected in [
        CAP019_PROFILE_HISTORY,
        CAP019_VALUE_BOUNDARY,
        CAP019_PRODUCT_BOUNDARY,
        CAP019_CLASSIFICATION_BOUNDARY,
        CAP019_EXCLUSION_BOUNDARY,
        CAP019_SPECIALIZATION_BOUNDARY,
        CAP020_ZERO_PRODUCTION_BOUNDARY,
        CAP020_PRODUCT_BOUNDARY,
        CAP020_GUARD_BOUNDARY,
        CAP020_EXCLUSION_BOUNDARY,
        CAP020_HISTORY_BOUNDARY,
        CAP020_ALERT_BOUNDARY,
        CAP015_M1_BOUNDARY,
        CAP016_CAP017_STOP_BOUNDARY,
    ] {
        assert!(
            normalized.contains(expected),
            "{document_name} is missing the canonical accepted-history boundary: {expected}"
        );
    }

    for stale in [
        "CAP-014 remains Aero's latest accepted compiler/profile capability",
        "The next action is `CAP-016-MODULE-RESOLUTION-READINESS`",
        "`CAP-016-MODULE-RESOLUTION-READINESS` is the next action",
        "The post-CAP-015 order begins with `CAP-016-MODULE-RESOLUTION-READINESS`",
        "The post-CAP-015 ranking now controls task selection",
        "Current accepted public master is CAP-015",
        "baseline is protected CAP-015",
        "Project status after CAP-015",
        "Within this profile, array results",
        "CAP-018 local candidate",
        "CAP-018 candidate (not accepted)",
        "CAP-019 local candidate",
        "CAP-019 candidate (not accepted)",
        "CAP-019 candidate only",
        "CAP-020 local candidate",
        "CAP-020 candidate (not accepted)",
        "CAP-020 candidate only",
        "CAP-020 acceptance pending",
        "Project status after CAP-018",
        "baseline is protected CAP-018",
        "Current accepted public master is CAP-018",
        "Latest accepted compiler/profile master is protected CAP-018",
        "CAP-018 is Aero's latest accepted compiler/profile capability",
        "CAP-018 is the latest accepted compiler/profile capability",
        "post-CAP-018",
        "The next action is mutable loop-produced exact-`i32` flat-array results",
        "Mutable loop-produced exact-`i32` flat-array results rank first",
        "This acceptance does not admit mutable array production",
        "projected or mutable array writes",
        "mutable selected-profile array results/writes",
        "Project status after CAP-019",
        "baseline is protected CAP-019",
        "Current accepted public master is CAP-019",
        "Latest accepted compiler/profile master is protected CAP-019 merge",
        "The post-CAP-019 order begins",
        "### Post-CAP-019 ranking",
        "The fresh post-CAP-019 capability-gap order",
        "The next action is the flat-buffer exact-`i32` 2D matrix-vector CPU product gate",
        "Flat-buffer exact-`i32` 2D matrix-vector CPU product gate ranks first",
    ] {
        assert!(
            !normalized_lower.contains(&stale.to_ascii_lowercase()),
            "{document_name} retains stale post-CAP-018 wording: {stale}"
        );
    }
    for counterclaim in [
        "cap-018 creates a new profile",
        "cap-018 created a new profile",
        "cap-018 adds a new profile",
        "cap-018 is a new profile",
        "cap-018 is a separate profile",
        "cap-018 remains a candidate",
        "cap-018 is not accepted",
        "cap-018 remains unaccepted",
        "cap-018 is pending acceptance",
        "cap-018 has not been accepted",
        "cap-018 is not yet accepted",
        "cap-019 creates a new profile",
        "cap-019 created a new profile",
        "cap-019 adds a new profile",
        "cap-019 is a new profile",
        "cap-019 is a separate profile",
        "cap-019 remains a candidate",
        "cap-019 is not accepted",
        "cap-019 remains unaccepted",
        "cap-019 is pending acceptance",
        "cap-019 acceptance is pending",
        "cap-019 acceptance remains pending",
        "cap-019 has not been accepted",
        "cap-019 is not yet accepted",
        "cap-019 widens `stable-scalar-v0`",
        "cap-019 adds general mutable arrays",
        "cap-019 accepts general mutable arrays",
        "cap-020 remains a candidate",
        "cap-020 is not accepted",
        "cap-020 remains unaccepted",
        "cap-020 is pending acceptance",
        "cap-020 acceptance is pending",
        "cap-020 acceptance remains pending",
        "cap-020 has not been accepted",
        "cap-020 is not yet accepted",
        "cap-020 is the latest compiler/profile capability widening",
        "cap-020 is a compiler/profile capability widening",
        "cap-020 widens `exact-i32-array-v0`",
        "cap-020 creates a new profile",
        "cap-020 adds a new profile",
        "cap-020 is a matrix type",
        "cap-014 is not the profile origin",
        "cap-014 is no longer the profile origin",
        "cap-014 is the latest accepted compiler/profile capability",
        "cap-014 remains the latest accepted compiler/profile capability",
        "latest accepted compiler/profile capability is cap-014",
        "cap-015 changes compiler production",
        "cap-015 changes language-profile code",
        "cap-015 widens `exact-i32-array-v0`",
        "cap-016 is accepted",
        "cap-017 is accepted",
        "cap-016 is an accepted capability",
        "cap-017 is an accepted capability",
        "cap-016 and cap-017 are accepted capabilities",
        "cap-016 is the next implementation",
        "cap-017 is the next implementation",
        "accepted cap-016",
        "accepted cap-017",
        "cap-016 adds a profile",
        "cap-017 adds a profile",
        "cap-016 implements",
        "cap-017 implements propagation",
    ] {
        assert!(
            !normalized_lower.contains(counterclaim),
            "{document_name} contradicts accepted history: {counterclaim}"
        );
    }

    for paragraph in normalized_markdown_paragraphs(&rendered) {
        let paragraph_lower = paragraph.to_ascii_lowercase();
        let cap020_prefix_lower = CAP020_EVIDENCE_PREFIX.to_ascii_lowercase();
        let cap019_prefix_lower = CAP019_EVIDENCE_PREFIX.to_ascii_lowercase();
        let cap018_leadin_lower = CAP018_CANDIDATE_LEADIN.to_ascii_lowercase();
        let status_text = paragraph_lower
            .strip_prefix(&cap020_prefix_lower)
            .or_else(|| paragraph_lower.strip_prefix(&cap019_prefix_lower))
            .unwrap_or(&paragraph_lower)
            .replace(&cap018_leadin_lower, "");
        for clause in status_text.split(['.', ';', '!', '?']) {
            let clause = clause.trim();
            let words = clause_words(clause);
            let semantic = semantic_words(clause);
            let has_word = |expected: &str| words.contains(&expected);
            let denies_acceptance = contains_semantic_phrase(&semantic, &["not", "accepted"])
                || contains_semantic_phrase(&semantic, &["not", "yet", "accepted"])
                || contains_semantic_phrase(&semantic, &["has", "not", "been", "accepted"]);
            if has_capability_token(&words, "cap-019") {
                assert!(
                    !has_word("candidate")
                        && !has_word("pending")
                        && !has_word("unaccepted")
                        && !denies_acceptance,
                    "{document_name} gives CAP-019 a candidate or unaccepted status: {clause}"
                );
                let excluded_subject = [
                    "general mutable array",
                    "uninitialized",
                    "partial array",
                    "mutable parameter",
                    "mutable result",
                    "mutable alias",
                    "reference",
                    "escaping place",
                    "whole-array reassignment",
                    "zero array",
                    "recursive array",
                    "nested array",
                    "repeat array",
                    "non-int array",
                    "stable aggregate abi",
                    "stable abi",
                    "stable layout",
                    "general parsing",
                    "runtime string",
                    "file input",
                    "gpu execution",
                    "performance",
                    "safety",
                ]
                .iter()
                .any(|subject| clause.contains(subject));
                if excluded_subject {
                    for (index, word) in words.iter().enumerate() {
                        if !matches!(
                            *word,
                            "admit"
                                | "admits"
                                | "accept"
                                | "accepts"
                                | "add"
                                | "adds"
                                | "support"
                                | "supports"
                                | "enable"
                                | "enables"
                                | "stabilize"
                                | "stabilizes"
                                | "guarantee"
                                | "guarantees"
                                | "provide"
                                | "provides"
                                | "implement"
                                | "implements"
                                | "deliver"
                                | "delivers"
                        ) {
                            continue;
                        }
                        let negated = words[index.saturating_sub(3)..index]
                            .iter()
                            .any(|prior| matches!(*prior, "not" | "no" | "without" | "never"));
                        assert!(
                            negated,
                            "{document_name} gives CAP-019 an excluded capability: {clause}"
                        );
                    }
                }
            }
            if has_capability_token(&words, "cap-018") {
                assert!(
                    !has_word("candidate")
                        && !has_word("pending")
                        && !has_word("unaccepted")
                        && !denies_acceptance,
                    "{document_name} gives CAP-018 a candidate or unaccepted status: {clause}"
                );
            }
            for capability in ["cap-016", "cap-017"] {
                if !has_capability_token(&words, capability) {
                    continue;
                }
                let acceptance_negative = clause.contains("not accepted")
                    || clause.contains("not an accepted")
                    || clause.contains("not a capability")
                    || clause.contains("not capabilities");
                let implementation_negative = clause.contains("not implementation")
                    || clause.contains("not an implementation")
                    || clause.contains("no implementation")
                    || clause.contains("neither is an implementation");
                let profile_negative = clause.contains("not a profile")
                    || clause.contains("no profile")
                    || clause.contains("does not add a profile")
                    || clause.contains("neither adds a profile");
                assert!(
                    !(has_word("accepted") && !acceptance_negative),
                    "{document_name} promotes {capability} to accepted status: {clause}"
                );
                assert!(
                    !(has_word("capability") && !acceptance_negative),
                    "{document_name} promotes {capability} to a capability: {clause}"
                );
                let implementation_claim =
                    has_word("implements") || has_word("implemented") || has_word("implementation");
                assert!(
                    !(implementation_claim && !implementation_negative),
                    "{document_name} promotes {capability} to implementation: {clause}"
                );
                assert!(
                    !(has_word("profile") && !profile_negative),
                    "{document_name} promotes {capability} to a profile: {clause}"
                );
            }
        }
    }
}

fn assert_cap021_boundaries(document_name: &str, document: &str) {
    let rendered = markdown_outside_fences(document);
    let readiness_surface = if document_name == "Roadmap.md" {
        let historical_start = document
            .find("### Post-CAP-020 ranking")
            .expect("Roadmap.md historical post-CAP-020 ranking");
        let current_start = document[historical_start..]
            .find("### Post-CAP-021 ranking")
            .map(|offset| historical_start + offset)
            .expect("Roadmap.md current post-CAP-021 ranking");
        let current_end = document[current_start..]
            .find("### Post-CAP-023 ranking")
            .map(|offset| current_start + offset)
            .expect("Roadmap.md post-CAP-023 ranking after CAP-021 history");
        format!(
            "{}{}",
            &document[..historical_start],
            &document[current_start..current_end]
        )
    } else {
        document.to_owned()
    };
    let normalized = normalized_words(&rendered);
    let lower = normalized.to_ascii_lowercase();
    assert!(
        stale_cap020_current_violation(document).is_none(),
        "{document_name} presents CAP-020 as current public state: {:?}",
        stale_cap020_current_violation(document)
    );
    assert!(
        cap021_product_violation(document).is_none(),
        "{document_name} promotes CAP-021 beyond its product-only boundary: {:?}",
        cap021_product_violation(document)
    );
    assert!(
        cap021_status_violation(document).is_none(),
        "{document_name} contradicts accepted CAP-021 status or evidence: {:?}",
        cap021_status_violation(document)
    );
    assert!(
        readiness_promotion_violation(&readiness_surface).is_none(),
        "{document_name} promotes a post-CAP-021 readiness-only successor: {:?}",
        readiness_promotion_violation(&readiness_surface)
    );
    for expected in [
        CAP021_ZERO_PRODUCTION_BOUNDARY,
        CAP021_RECORD_BOUNDARY,
        CAP021_RESULT_BOUNDARY,
        CAP021_GUARD_BOUNDARY,
        CAP021_EXCLUSION_BOUNDARY,
        CAP021_HISTORY_BOUNDARY,
        CAP021_ALERT_BOUNDARY,
    ] {
        assert_eq!(
            normalized.matches(expected).count(),
            1,
            "{document_name} must state the canonical CAP-021 boundary exactly once: {expected}"
        );
    }
    for stale in [
        "Current accepted public master is CAP-020",
        "baseline is protected CAP-020 product merge",
        "Latest accepted public master is protected CAP-020 product merge",
        "Project status after CAP-020",
        "CAP-021 remains a candidate",
        "CAP-021 acceptance is pending",
        "CAP-021 is not accepted",
        "CAP-021 has not been accepted",
    ] {
        assert!(
            !lower.contains(&stale.to_ascii_lowercase()),
            "{document_name} retains stale or contradictory CAP-021 truth: {stale}"
        );
    }
    if document_name != "Roadmap.md" {
        for consumed in [
            "### Post-CAP-020 ranking",
            "The next action is the source-embedded fixed-shape tensor-record decode plus two-stage flat-buffer exact-`i32` CPU scoring product gate",
            "Source-embedded fixed-shape tensor-record decode plus two-stage flat-buffer exact-`i32` CPU scoring product gate ranks first",
        ] {
            assert!(
                !lower.contains(&consumed.to_ascii_lowercase()),
                "{document_name} retains consumed post-CAP-020 next-action wording: {consumed}"
            );
        }
    }
}

fn assert_cap023_boundaries(document_name: &str, document: &str) {
    let rendered = markdown_outside_fences(document);
    let current_surface = if document_name == "Roadmap.md" {
        let historical_start = document
            .find("### Post-CAP-020 ranking")
            .expect("Roadmap.md historical post-CAP-020 ranking");
        let current_start = document
            .find("### Post-CAP-023 ranking")
            .expect("Roadmap.md current post-CAP-023 ranking");
        format!(
            "{}{}",
            &document[..historical_start],
            &document[current_start..]
        )
    } else if document_name == "CURRENT_CAPABILITY_AUDIT.md" {
        let historical_start = document
            .find("### ROADMAP-001 ranking and M1-001 outcome")
            .expect("CURRENT_CAPABILITY_AUDIT.md historical ROADMAP-001 ranking");
        let historical_end = document[historical_start..]
            .find("## Verified progress after the audit commit")
            .map(|offset| historical_start + offset)
            .expect("CURRENT_CAPABILITY_AUDIT.md post-ROADMAP-001 audit history");
        format!(
            "{}{}",
            &document[..historical_start],
            &document[historical_end..]
        )
    } else {
        document.to_owned()
    };
    let readiness_surface = if document_name == "Roadmap.md" {
        let current_start = current_surface
            .find("### Post-CAP-023 ranking")
            .expect("Roadmap.md current post-CAP-023 readiness surface");
        &current_surface[current_start..]
    } else {
        current_surface.as_str()
    };
    let normalized = normalized_words(&rendered);
    let current_lower =
        normalized_words(&markdown_outside_fences(&current_surface)).to_ascii_lowercase();
    assert!(
        stale_pre_cap023_current_violation(&current_surface).is_none(),
        "{document_name} presents CAP-021 or CAP-022 as current public state: {:?}",
        stale_pre_cap023_current_violation(&current_surface)
    );
    assert!(
        cap023_product_violation(document).is_none(),
        "{document_name} promotes CAP-023 beyond its product/evidence boundary: {:?}",
        cap023_product_violation(document)
    );
    assert!(
        cap023_status_violation(document).is_none(),
        "{document_name} contradicts accepted CAP-023 status or oracles: {:?}",
        cap023_status_violation(document)
    );
    assert!(
        cap023_milestone_violation(document).is_none(),
        "{document_name} overclaims CAP-023 milestone completion: {:?}",
        cap023_milestone_violation(document)
    );
    assert!(
        post_cap023_readiness_promotion_violation(readiness_surface).is_none(),
        "{document_name} promotes a post-CAP-023 rank-2/3 readiness successor: {:?}",
        post_cap023_readiness_promotion_violation(readiness_surface)
    );
    for expected in [
        CAP023_ZERO_PRODUCTION_BOUNDARY,
        CAP023_APPLICATION_BOUNDARY,
        CAP023_ORACLE_BOUNDARY,
        CAP023_EXCLUSION_BOUNDARY,
        CAP023_HISTORY_BOUNDARY,
        CAP023_CLASSIFICATION_BOUNDARY,
        CAP023_ALERT_BOUNDARY,
    ] {
        assert_eq!(
            normalized.matches(expected).count(),
            1,
            "{document_name} must state the canonical CAP-023 boundary exactly once: {expected}"
        );
    }
    assert_eq!(
        normalized.matches(CAP023_MILESTONE_BOUNDARY).count(),
        1,
        "{document_name} must state the canonical CAP-023 milestone truth exactly once"
    );
    for stale in [
        "Current accepted public master is CAP-021",
        "baseline is protected CAP-021 product merge",
        "Latest accepted public master is protected CAP-021 product merge",
        "Project status after CAP-021",
        "Current accepted public master is CAP-022",
        "baseline is protected CAP-022",
        "Project status after CAP-022",
        "CAP-023 remains a candidate",
        "CAP-023 local candidate",
        "CAP-023 candidate only",
        "CAP-023 acceptance is pending",
    ] {
        assert!(
            !current_lower.contains(&stale.to_ascii_lowercase()),
            "{document_name} retains stale or contradictory CAP-023 truth: {stale}"
        );
    }
    for consumed in [
        "### Post-CAP-021 ranking",
        "Runtime byte/file acquisition readiness and red probe under one cross-platform bounded-owned-buffer contract ranks first",
        "The next action is runtime byte/file acquisition readiness and a red probe under one cross-platform bounded-owned-buffer contract",
        "The post-CAP-021 order begins with runtime byte/file acquisition readiness and a red probe under one cross-platform bounded-owned-buffer contract",
    ] {
        assert!(
            !current_lower.contains(&consumed.to_ascii_lowercase()),
            "{document_name} retains consumed post-CAP-021 next-action wording: {consumed}"
        );
    }
}

fn assert_cap024_boundaries(document_name: &str, document: &str) {
    let rendered = markdown_outside_fences(document);
    let current_surface = if document_name == "Roadmap.md" {
        let historical_start = document
            .find("### Post-CAP-020 ranking")
            .expect("Roadmap.md historical post-CAP-020 ranking");
        let current_start = document
            .find("### Post-CAP-024 ranking")
            .expect("Roadmap.md current post-CAP-024 ranking");
        format!(
            "{}{}",
            &document[..historical_start],
            &document[current_start..]
        )
    } else if document_name == "CURRENT_CAPABILITY_AUDIT.md" {
        let historical_start = document
            .find("### ROADMAP-001 ranking and M1-001 outcome")
            .expect("CURRENT_CAPABILITY_AUDIT.md historical ROADMAP-001 ranking");
        let historical_end = document[historical_start..]
            .find("## Verified progress after the audit commit")
            .map(|offset| historical_start + offset)
            .expect("CURRENT_CAPABILITY_AUDIT.md post-ROADMAP-001 audit history");
        format!(
            "{}{}",
            &document[..historical_start],
            &document[historical_end..]
        )
    } else {
        document.to_owned()
    };
    let normalized = normalized_words(&rendered);
    let current_lower =
        normalized_words(&markdown_outside_fences(&current_surface)).to_ascii_lowercase();
    assert!(
        stale_cap023_current_head_violation(&current_surface).is_none(),
        "{document_name} presents CAP-023 as the current public head: {:?}",
        stale_cap023_current_head_violation(&current_surface)
    );
    assert!(
        cap024_product_violation(document).is_none(),
        "{document_name} promotes CAP-024 beyond its evidence-only boundary: {:?}",
        cap024_product_violation(document)
    );
    assert!(
        cap024_status_violation(document).is_none(),
        "{document_name} contradicts accepted CAP-024 status: {:?}",
        cap024_status_violation(document)
    );
    assert!(
        cap023_milestone_violation(document).is_none(),
        "{document_name} overclaims post-CAP-024 milestone completion: {:?}",
        cap023_milestone_violation(document)
    );
    assert!(
        post_cap024_readiness_promotion_violation(&current_surface).is_none(),
        "{document_name} promotes a post-CAP-024 readiness-only successor: {:?}",
        post_cap024_readiness_promotion_violation(&current_surface)
    );
    assert!(
        stopped_capability_violation(&current_surface).is_none(),
        "{document_name} promotes a mandatory CAP-016/CAP-022 stop: {:?}",
        stopped_capability_violation(&current_surface)
    );
    assert!(
        consumed_cap024_evidence_ranking_violation(&current_surface).is_none(),
        "{document_name} restores the completed accepted-head evidence gate to current ranking: {:?}",
        consumed_cap024_evidence_ranking_violation(&current_surface)
    );
    for expected in [
        CAP024_CURRENT_HEAD_BOUNDARY,
        CAP024_ZERO_PRODUCTION_BOUNDARY,
        CAP024_CLASSIFICATION_BOUNDARY,
        CAP024_BUNDLE_BOUNDARY,
        CAP024_ALERT_BOUNDARY,
        CAP024_MILESTONE_BOUNDARY,
        CAP016_LOCAL_MODDECL_STOP_BOUNDARY,
    ] {
        assert_eq!(
            normalized.matches(expected).count(),
            1,
            "{document_name} must state the canonical CAP-024/retained-stop boundary exactly once: {expected}"
        );
    }
    for stale in [
        "Current accepted public master is CAP-023",
        "Latest accepted public master is protected CAP-023",
        "baseline is protected CAP-023 merge",
        "Project status after CAP-023",
        "CAP-024 remains a candidate",
        "CAP-024 local candidate",
        "CAP-024 candidate only",
        "CAP-024 acceptance is pending",
        "CAP-024 acceptance was revoked",
        "CAP-024 merge CI failed",
    ] {
        assert!(
            !current_lower.contains(&stale.to_ascii_lowercase()),
            "{document_name} retains stale or contradictory CAP-024 truth: {stale}"
        );
    }
    for consumed in [
        "### Post-CAP-023 ranking",
        "Accepted-head CAP-023 inference correctness/reproducibility/artifact-footprint evidence gate with no performance claim",
        "Accepted-head CAP-023 evidence ranks first",
        "The next action is the accepted-head CAP-023 inference correctness/reproducibility/artifact-footprint evidence gate",
    ] {
        assert!(
            !current_lower.contains(&consumed.to_ascii_lowercase()),
            "{document_name} retains consumed post-CAP-023 current-ranking wording: {consumed}"
        );
    }
}

fn assert_core090_accepted_partial_history(
    readme: &str,
    project_state: &str,
    audit: &str,
    matrix: &str,
    alignment: &str,
    roadmap: &str,
) {
    for (document_name, document) in [
        ("README.md", readme),
        ("PROJECT_STATE.md", project_state),
        ("CURRENT_CAPABILITY_AUDIT.md", audit),
        ("SPEC_IMPLEMENTATION_MATRIX.md", matrix),
        ("FRAMEWORK_ALIGNMENT.md", alignment),
        ("Roadmap.md", roadmap),
    ] {
        assert!(
            core090_overclaim_violation(document).is_none(),
            "{document_name} contradicts the bounded accepted-PARTIAL CORE-090 history: {:?}",
            core090_overclaim_violation(document)
        );
    }
    let readme = normalized_words(&markdown_outside_fences(readme));
    assert!(readme.contains(
        "CORE-090 is an accepted public static projected CopyData assignment checkpoint"
    ));
    assert!(readme.contains(
        "Dynamic indexes, projected borrowing, partial moves, enum/non-Copy subplaces, alias analysis, lifetime/drop, stable ABI/FFI, accelerators, and general memory-safety claims remain excluded"
    ));

    let project_state = normalized_words(&markdown_outside_fences(project_state));
    for expected in [
        "Milestone 111 `CORE-090` is accepted public at exact candidate head",
        "af68d0e842ed2973087d2e3c78d2a19546e29ff7",
        "8455a06a4473a826ef1ea180e291e2ddb790bed0",
        "ca00cdb70fc0a1940fa94126c49774b99d03c515",
        "128205615c53156138c4effa740b61ab455a760f",
        "The accepted class closes statically addressed projected CopyData assignment",
        "Dynamic/computed target indexes, projected borrowing, partial moves, enum/non-Copy subplaces, alias analysis, NLL/lifetime/drop, public layout, stable ABI/FFI, accelerators, and general memory-safety claims remain excluded",
    ] {
        assert!(
            project_state.contains(expected),
            "PROJECT_STATE.md must preserve CORE-090/Milestone 111 accepted PARTIAL fact: {expected}"
        );
    }

    let audit = normalized_words(&markdown_outside_fences(audit));
    assert!(
        audit
            .contains("Accepted CORE-083 through CORE-090 are valid bounded Milestone 2 fragments")
    );
    assert!(audit.contains(
        "These checkpoints remain `PARTIAL`; they do not establish general borrowing, lifetime/drop, memory safety, public layout/ABI, or a stable language"
    ));
    for identity in [
        "af68d0e842ed2973087d2e3c78d2a19546e29ff7",
        "8455a06a4473a826ef1ea180e291e2ddb790bed0",
        "128205615c53156138c4effa740b61ab455a760f",
    ] {
        assert!(
            audit.contains(identity),
            "CURRENT_CAPABILITY_AUDIT.md must preserve CORE-090 identity {identity}"
        );
    }

    const CORE090_MATRIX_ROW: &str = "| Static projected CopyData assignment (`CORE-090` accepted) | Y | Y | Y | P | Y | Y | Y | Y | P | Y | Y | Y | Y | PARTIAL |";
    let matrix_rendered = markdown_outside_fences(matrix);
    assert_eq!(
        matrix_rendered
            .lines()
            .map(table_line)
            .filter(|line| *line == CORE090_MATRIX_ROW)
            .count(),
        1,
        "SPEC_IMPLEMENTATION_MATRIX.md must preserve the exact accepted PARTIAL CORE-090 row"
    );
    let matrix = normalized_words(&matrix_rendered);
    assert!(matrix.contains(
        "Accepted public `CORE-090` admits exactly one nonempty static projection path rooted at an initialized mutable owned direct local recursive finite CopyData value"
    ));
    assert!(matrix.contains("the row therefore remains `PARTIAL`"));

    let alignment = normalized_words(&markdown_outside_fences(alignment));
    assert!(alignment.contains(
        "Accepted public `CORE-090` takes a hard ownership step by closing one complete recursive place class instead of one convenient selector shape"
    ));
    assert!(alignment.contains(
        "Dynamic indexes, projected borrowing, partial moves, enum/non-Copy subplaces, alias analysis, NLL/lifetime/drop, stable ABI/FFI, accelerators, and general memory-safety claims remain excluded"
    ));
    assert!(alignment.contains(
        "The accepted recursive CopyData, enum/Match, projected-place, generic, trait, and reference slices remain bounded and `PARTIAL`"
    ));

    let roadmap = normalized_words(&markdown_outside_fences(roadmap));
    for expected in [
        "## Corrective checkpoint after CORE-090",
        "The original milestone exits below remain controlling",
        "foundational Milestone 0 contracts and broader Milestone 1 feature invariants remain partial",
        "The previously accumulated Milestone 2 fragments remain bounded",
        "accepted CORE-083 through CORE-090 are useful but partial Milestone 2 reference, ownership",
        "The selected Milestone 2 exit gate is met",
    ] {
        assert!(
            roadmap.contains(expected),
            "Roadmap.md must preserve the post-CORE-090 accepted-PARTIAL audit fact: {expected}"
        );
    }
}

fn run_aero(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aero"))
        .args(args)
        .output()
        .expect("run Aero CLI")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn cli_implementation_version_is_manifest_derived_on_every_existing_route() {
    let expected_version = format!("Aero compiler version {PACKAGE_VERSION}");
    for flag in ["--version", "-v"] {
        let output = run_aero(&[flag]);
        assert!(output.status.success(), "{flag}: {}", stderr(&output));
        assert_eq!(stdout(&output).trim(), expected_version);
    }

    let no_command = run_aero(&[]);
    assert_eq!(no_command.status.code(), Some(2));
    assert_eq!(
        stdout(&no_command).lines().next(),
        Some(format!("Aero Programming Language Compiler v{PACKAGE_VERSION}").as_str())
    );

    let bare_version = run_aero(&["version"]);
    assert_eq!(bare_version.status.code(), Some(2));
    assert!(stderr(&bare_version).contains("Unknown command: version"));

    let main = repository_file("src/compiler/src/main.rs");
    assert!(
        main.contains(r#"env!("CARGO_PKG_VERSION")"#),
        "main must interpolate Cargo's package version"
    );
    assert!(
        !main.contains(PACKAGE_VERSION),
        "main must not contain the current package-version literal"
    );
    assert!(!main.contains(&format!("Aero compiler version {PACKAGE_VERSION}")));
    assert!(!main.contains(&format!(
        "Aero Programming Language Compiler v{PACKAGE_VERSION}"
    )));
}

#[test]
fn conformance_counts_and_compatibility_schema_remain_unchanged() {
    let report = run_conformance_suite();
    assert_eq!((report.passed_cases, report.total_cases), (3, 3));
    assert_eq!(
        (
            report.passed_mechanized_checks,
            report.total_mechanized_checks
        ),
        (4, 4)
    );

    let json = serde_json::to_value(&report).expect("serialize conformance report");
    let object = json.as_object().expect("report JSON object");
    for key in [
        "total_mechanized_checks",
        "passed_mechanized_checks",
        "failed_mechanized_checks",
        "mechanized_checks",
    ] {
        assert!(object.contains_key(key), "missing compatibility key {key}");
    }
}

#[test]
fn conformance_is_presented_as_deterministic_regression_evidence() {
    let output = run_aero(&["conformance"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let output_text = stdout(&output);
    assert!(output_text.contains("Conformance cases: 3/3 passed | Determinism checks: 4/4 passed"));
    assert!(!output_text.contains("Mechanized checks"));

    let help = run_aero(&["--help"]);
    assert!(help.status.success(), "{}", stderr(&help));
    let help_text = stdout(&help);
    assert!(help_text.contains("Run deterministic regression checks"));
    assert!(
        !help_text
            .to_ascii_lowercase()
            .contains("formal conformance")
    );
    assert!(!help_text.to_ascii_lowercase().contains("mechanized checks"));

    let build = repository_file("BUILD.md");
    assert!(build.contains("deterministic regression checks"));
    assert!(!build.contains("CLI command summary (v1.0.0)"));
    assert!(
        !build
            .to_ascii_lowercase()
            .contains("formal conformance suite")
    );
    assert!(!build.to_ascii_lowercase().contains("mechanized semantics"));
}

#[test]
fn current_repository_surfaces_state_only_evidenced_capabilities() {
    assert_cap020_contract_mutation_fixtures();
    assert_cap021_contract_mutation_fixtures();
    assert_cap023_contract_mutation_fixtures();
    assert_cap024_contract_mutation_fixtures();
    let task_ledger = repository_file("TASK_LEDGER.md");
    let readme_source = repository_file("README.md");
    let readme = markdown_outside_fences(&readme_source);
    assert!(readme.contains(
        "Accepted CAP-004 adds one explicit user-defined recursive-CopyData generic-struct substitution class"
    ));
    assert!(
        readme.contains("accepted CAP-005 adds bound-free whole-value generic transport functions")
    );
    assert!(readme.contains(
        "accepted CAP-006 adds exact explicit user-defined recursive-CopyData generic-enum specialization"
    ));
    assert!(readme.contains("**CAP-007 accepted:** public artifact-free"));
    assert!(readme.contains(
        "Accepted CAP-007 makes check/build/run/profile/source-test validation consume one checked-program preparation authority"
    ));
    assert!(!readme.contains("CAP-007 local candidate (not accepted)"));
    assert!(readme.contains("**CAP-008 accepted:** terminal `_ => fallback`"));
    assert!(readme.contains("terminal `_ => fallback` and ignored"));
    assert!(readme.contains("payload leaves such as `Err(_)`"));
    assert!(!readme.contains("CAP-008 local candidate (not accepted)"));
    assert!(readme.contains("**CAP-009 accepted:**"));
    assert!(readme.contains("--language-profile stable-scalar-v0"));
    assert!(readme.contains("**CAP-010 accepted:**"));
    assert!(!readme.contains("CAP-010 local candidate (not accepted master)"));
    assert!(readme.contains("**CAP-011 accepted:**"));
    assert!(!readme.contains("CAP-011 local candidate (not accepted)"));
    assert!(readme.contains("**CAP-012 accepted:**"));
    assert!(readme.contains("**CAP-013 accepted:** protected master gives"));
    assert!(readme.contains("**CAP-014 accepted:** protected master now includes"));
    assert!(
        readme.contains("**CAP-015 accepted project integration:** protected master now includes")
    );
    assert!(readme.contains("**CAP-018 accepted:**"));
    assert!(readme.contains("**CAP-019 accepted:**"));
    assert!(readme.contains("**CAP-020 accepted product gate:**"));
    assert!(readme.contains("**CAP-021 accepted product gate:**"));
    assert!(readme.contains("**CAP-023 accepted"));
    assert!(readme.contains("Project status after CAP-024"));
    assert!(readme.contains("baseline is protected CAP-024 merge"));
    assert!(readme.contains("b62696272f293f9f378f8a368cc818fcb8ef1074"));
    assert!(readme.contains("e9b281504446465cfc8fcbe17c65cce92df0e83a"));
    assert!(
        readme.contains("CAP-019 is Aero's latest accepted compiler/profile capability widening")
    );
    assert!(readme.contains("CAP-015 is the latest accepted project integration checkpoint"));
    assert!(readme.contains("`exact-i32-array-v0` profile, classified `END_TO_END`"));
    assert!(readme.contains("Broad integer and fixed-array support remains `PARTIAL`"));
    assert!(readme.contains("`stable-scalar-v0` remains Aero's only `STABLE` profile"));
    assert!(!readme.contains("CAP-012 candidate—not accepted"));
    assert!(!readme.contains("baseline is protected CAP-011 merge"));
    assert!(!readme.contains("baseline is protected CAP-012 merge"));
    assert!(!readme.contains("baseline is protected CAP-013 merge"));
    assert!(!readme.contains("Project status after CAP-014"));
    assert!(!readme.contains("Project status after CAP-015"));
    assert!(!readme.contains("Project status after CAP-018"));
    assert!(!readme.contains("Project status after CAP-019"));
    assert!(!readme.contains("Project status after CAP-020"));
    assert!(!readme.contains("Project status after CAP-021"));
    assert!(!readme.contains("Project status after CAP-022"));
    assert!(!readme.contains("CAP-013 candidate (not accepted)"));
    assert!(!readme.contains("CAP-014 candidate (not accepted)"));
    assert!(!readme.contains("`CAP-015-READINESS`"));
    assert!(!readme.contains(
        "next ranked product target is an explicitly profiled exact fixed-width integer"
    ));

    let audit_source = repository_file("CURRENT_CAPABILITY_AUDIT.md");
    let audit = markdown_outside_fences(&audit_source);
    assert!(audit.contains(
        "Accepted CAP-007 closes the canonical checked-entrypoint and artifact mechanism"
    ));
    assert!(!audit.contains(
        "no authoritative stable subset or single canonical diagnostic contract is frozen"
    ));
    assert!(audit.contains("CAP-008 accepted: nonbinding wildcard enum Match"));
    assert!(audit.contains("protected CAP-024 merge"));
    assert!(!audit.contains("this record is its bounded acceptance synchronization candidate"));
    assert!(audit.contains("CAP-009 accepted: enforceable `stable-scalar-v0`"));
    assert!(audit.contains("CAP-010 accepted: required-only CopyData trait-bound static dispatch"));
    assert!(!audit.contains("CAP-010 local candidate"));
    assert!(audit.contains("CAP-011 accepted: generic fixed-window algorithms"));
    assert!(!audit.contains("CAP-011 local candidate"));
    assert!(audit.contains("CAP-012 accepted: nonescaping projected CopyData call loans"));
    assert!(
        audit.contains("CAP-013 accepted: canonical specialization identity and phase authority")
    );
    assert!(audit.contains("CAP-014 accepted: exact `i32` fixed-array CPU reference kernel"));
    assert!(
        audit.contains("CAP-015 accepted: embedded character-record representative integration")
    );
    assert!(audit.contains("CAP-018 accepted: immutable exact-array value/result composition"));
    assert!(audit.contains("CAP-019 accepted: initialized mutable exact-array production"));
    assert!(audit.contains("CAP-020 accepted: flat-buffer 2x3-by-3 matvec product gate"));
    assert!(
        audit.contains("CAP-021 accepted: source-embedded two-stage exact-i32 scoring product")
    );
    assert!(audit.contains("CAP-023 accepted:"));
    assert!(audit.contains("CAP-024 accepted:"));
    assert!(audit.contains("b62696272f293f9f378f8a368cc818fcb8ef1074"));
    assert!(audit.contains("c49ff17cab7fc0e8d4f552a71499929135c16c61"));
    assert!(audit.contains(
        "CORE-043 through CORE-090 and accepted CAP-001 through CAP-013 implemented substantial typed"
    ));
    assert!(!audit.contains(
        "CORE-043 through CORE-090 and accepted CAP-001 through CAP-014 implemented substantial typed"
    ));
    assert!(audit.contains("selected Milestone 2 exit gate"));
    assert!(!audit.contains("CAP-012 candidate only"));
    assert!(!audit.contains("protected CAP-012 compiler-capability merge"));
    assert!(!audit.contains("protected CAP-013 compiler-capability merge"));
    assert!(!audit.contains("`CAP-015-READINESS`"));
    assert!(audit.contains("selected-profile row is therefore `STABLE`"));

    let alignment_source = repository_file("FRAMEWORK_ALIGNMENT.md");
    let alignment = markdown_outside_fences(&alignment_source);
    assert!(alignment.contains(
        "accepted CAP-007 makes library compile/check plus CLI check/build/run/profile/source-test validation consume one canonical checked-program authority"
    ));
    assert!(alignment.contains("Accepted CAP-009 advances the founding Stabilize direction"));
    assert!(
        alignment
            .contains("Accepted CAP-010 advances the founding preference for traits and generics")
    );
    assert!(alignment.contains("Accepted CAP-011 advances the founding generic-data-"));
    assert!(
        alignment
            .contains("Accepted CAP-012 advances the founding ownership-and-borrowing direction")
    );
    assert!(alignment.contains(
        "Accepted CAP-013 advances the founding generic and compile-time monomorphization"
    ));
    assert!(alignment.contains(
        "Accepted CAP-014 advances the founding high-performance and data-pipeline direction"
    ));
    assert!(alignment.contains(
        "Accepted CAP-015 advances the founding composition and execution-quality direction"
    ));
    assert!(alignment.contains(
        "Accepted CAP-018 advances the founding high-performance and data-pipeline direction"
    ));
    assert!(alignment.contains(
        "Accepted CAP-019 advances the founding high-performance and data-pipeline direction"
    ));
    assert!(alignment.contains(
        "Accepted CAP-020 advances the founding high-performance and data-pipeline direction"
    ));
    assert!(alignment.contains(
        "Accepted CAP-021 advances the founding high-performance and data-pipeline direction"
    ));
    assert!(alignment.contains("Accepted CAP-023 advances"));
    assert!(alignment.contains("Accepted CAP-024 advances"));
    assert!(alignment.contains("b62696272f293f9f378f8a368cc818fcb8ef1074"));
    assert!(alignment.contains("c49ff17cab7fc0e8d4f552a71499929135c16c61"));
    assert!(!alignment.contains("`CAP-015-READINESS`"));
    assert!(alignment.contains("satisfy the roadmap's selected Milestone 2 exit gate"));
    assert!(alignment.contains("Aero remains\na Minimal Prototype"));
    assert!(!alignment.contains("Projected borrowing, reference-target dynamic writes"));
    assert!(!alignment.contains("close the remaining Milestone 2 exit half"));

    let project_state_source = repository_file("PROJECT_STATE.md");
    let project_state = markdown_outside_fences(&project_state_source);
    assert!(project_state.contains("CAP-009 accepted: enforceable `stable-scalar-v0`"));
    assert!(project_state.contains("Protected PR #40 merged it as accepted master"));
    assert!(
        project_state
            .contains("CAP-010 accepted: required-only CopyData trait-bound static dispatch")
    );
    assert!(!project_state.contains("CAP-010 local candidate"));
    assert!(
        project_state
            .contains("CAP-011 accepted: fixed-capacity generic CopyData container algorithms")
    );
    assert!(!project_state.contains("CAP-011 candidate:"));
    assert!(project_state.contains("CAP-012 accepted: nonescaping projected CopyData call loans"));
    assert!(project_state.contains("#46 merged it as accepted master"));
    assert!(project_state.contains("49bcdfc3b23d2e1cc22fa3f0f36446fcffbf6e92"));
    assert!(project_state.contains("selected Milestone 2 exit gate"));
    assert!(!project_state.contains("CAP-012 candidate (not accepted)"));
    assert!(
        project_state
            .contains("CAP-013 accepted: canonical specialization identity and phase authority")
    );
    assert!(
        project_state.contains("CAP-014 accepted: exact `i32` fixed-array CPU reference kernel")
    );
    assert!(
        project_state
            .contains("CAP-015 accepted: embedded character-record representative integration")
    );
    assert!(
        project_state.contains("CAP-018 accepted: immutable exact-array value/result composition")
    );
    assert!(project_state.contains("CAP-019 accepted: initialized mutable exact-array production"));
    assert!(project_state.contains("CAP-020 accepted: flat-buffer 2x3-by-3 matvec product gate"));
    assert!(
        project_state
            .contains("CAP-021 accepted: source-embedded two-stage exact-i32 scoring product")
    );
    assert!(project_state.contains("CAP-023 accepted:"));
    assert!(project_state.contains("CAP-024 accepted:"));
    assert!(project_state.contains(
        "Current accepted public master and public evidence checkpoint is protected CAP-024 merge"
    ));
    assert!(project_state.contains("b62696272f293f9f378f8a368cc818fcb8ef1074"));
    assert!(project_state.contains("c49ff17cab7fc0e8d4f552a71499929135c16c61"));
    assert!(!project_state.contains("Current accepted public master is CAP-012"));
    assert!(!project_state.contains("Current accepted public master is CAP-013"));
    assert!(!project_state.contains("Current accepted public master is CAP-014"));
    assert!(!project_state.contains("`CAP-015-READINESS`"));
    assert!(
        !project_state
            .contains("next ranked product target is an explicitly profiled exact fixed-width")
    );

    let matrix_source = repository_file("SPEC_IMPLEMENTATION_MATRIX.md");
    let matrix = markdown_outside_fences(&matrix_source);
    assert!(
        cap023_matrix_violation(&matrix_source).is_none(),
        "SPEC_IMPLEMENTATION_MATRIX.md violates the CAP-023 sole-row contract: {:?}",
        cap023_matrix_violation(&matrix_source)
    );
    assert!(
        cap024_matrix_violation(&matrix_source).is_none(),
        "SPEC_IMPLEMENTATION_MATRIX.md violates the CAP-024 no-row contract: {:?}",
        cap024_matrix_violation(&matrix_source)
    );
    assert!(
        frozen_cap024_matrix_source_row_violation(&matrix_source).is_none(),
        "SPEC_IMPLEMENTATION_MATRIX.md changes a byte-frozen selected-profile/CPU source row: {:?}",
        frozen_cap024_matrix_source_row_violation(&matrix_source)
    );
    assert!(matrix.contains("Accepted CAP-009 adds an explicitly selected `stable-scalar-v0`"));
    assert!(matrix.contains("Selected `stable-scalar-v0` profile (accepted `CAP-009`)"));
    assert!(matrix.contains("Accepted CAP-010 adds one bounded partial row"));
    assert!(matrix.contains(
        "Required-only recursive-CopyData trait-bound static dispatch (accepted `CAP-010`)"
    ));
    assert!(matrix.contains("| STABLE |"));
    assert!(matrix.contains("Latest accepted public master is protected CAP-024 merge"));
    assert!(
        normalized_words(&matrix)
            .contains("Latest accepted project-integration master is protected CAP-015 merge")
    );
    assert!(matrix.contains("Accepted CAP-011 composes the existing generic-struct"));
    assert!(matrix.contains("Nonescaping projected CopyData call loans (accepted `CAP-012`)"));
    assert!(matrix.contains(
        "Canonical alias identity and shared bounded-specialization phase authority (accepted `CAP-013`)"
    ));
    let language_features = matrix
        .split_once("## Language features")
        .expect("matrix language-feature section")
        .1
        .split_once("## Compiler, tooling, and ecosystem surfaces")
        .expect("matrix compiler/tooling section")
        .0;
    let language_feature_rows = language_features
        .lines()
        .filter_map(|line| table_cells(line).map(|cells| (table_line(line), cells)))
        .collect::<Vec<_>>();
    for expected_header in [
        "| Feature | Spec | Lex | Parse | Res | Ty | Own | TIR | BE | Exec | + | - | D | Docs | Class |",
        "| Surface | Interface | Shared compiler truth | Artifact/result | Failure tests | Integration evidence | Docs | Class |",
        "| Backend/surface | Selectable | IR transform | Object | Link | Real execution | Numerical checks | Performance evidence | Class |",
    ] {
        assert!(
            markdown_table_after_header_is_valid(&matrix, expected_header),
            "matrix must render the frozen header, same-width delimiter, and data rows: {expected_header}"
        );
        assert_eq!(
            matrix
                .lines()
                .map(table_line)
                .filter(|line| *line == expected_header)
                .count(),
            1,
            "matrix must preserve exactly one closed-classification header: {expected_header}"
        );
    }
    assert_eq!(
        matrix
            .lines()
            .filter_map(table_cells)
            .filter(|cells| {
                cells.last().is_some_and(|cell| {
                    cell.eq_ignore_ascii_case("class")
                        || cell.eq_ignore_ascii_case("classification")
                })
            })
            .count(),
        3,
        "matrix must expose exactly the three frozen classification tables"
    );
    let mut classified_cell_count = None;
    for line in matrix.lines().map(table_line) {
        let Some(cells) = table_cells(line) else {
            classified_cell_count = None;
            continue;
        };
        if cells
            .last()
            .is_some_and(|classification| classification.eq_ignore_ascii_case("class"))
        {
            classified_cell_count = Some(cells.len());
            continue;
        }
        if classified_cell_count.is_none() {
            continue;
        }
        assert_eq!(
            cells.len(),
            classified_cell_count.expect("active classified table header"),
            "matrix classified row changes its table cardinality: {line}"
        );
        if cells
            .iter()
            .all(|cell| !cell.is_empty() && cell.chars().all(|ch| matches!(ch, '-' | ':')))
        {
            continue;
        }
        let classification = cells
            .last()
            .unwrap_or_else(|| panic!("matrix classified row has no final cell: {line}"));
        assert!(
            [
                "STABLE",
                "END_TO_END",
                "PARTIAL",
                "PARSED_ONLY",
                "EXPERIMENTAL",
                "DESIGNED",
                "ABSENT",
            ]
            .iter()
            .any(|expected| classification.eq_ignore_ascii_case(expected)),
            "matrix classified row has an unknown/decorated classification: {line}"
        );
    }
    let exact_profile_rows = language_features
        .lines()
        .map(table_line)
        .filter(|line| {
            if table_cells(line).is_none() {
                return false;
            }
            let words = semantic_words(line);
            contains_semantic_phrase(&words, &["exact", "i32", "array", "v0"])
                || has_semantic_capability(&words, "018")
                || has_semantic_capability(&words, "019")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        exact_profile_rows,
        [CAP019_SELECTED_PROFILE_MATRIX_ROW],
        "CAP-014/CAP-018/CAP-019 must classify in exactly one widened profile row"
    );
    assert!(
        !language_features
            .lines()
            .map(semantic_words)
            .any(|words| has_semantic_capability(&words, "020")),
        "CAP-020 must not create a language-feature or profile row"
    );
    assert!(
        !language_features
            .lines()
            .map(semantic_words)
            .any(|words| has_semantic_capability(&words, "021")),
        "CAP-021 must not create a language-feature or profile row"
    );
    assert!(
        !language_features
            .lines()
            .map(semantic_words)
            .any(|words| has_semantic_capability(&words, "023")),
        "CAP-023 must not create a language-feature or profile row"
    );
    assert!(
        !language_features
            .lines()
            .map(semantic_words)
            .any(|words| has_semantic_capability(&words, "024")),
        "CAP-024 must not create a language-feature or profile row"
    );
    let cap020_matrix_rows = matrix
        .lines()
        .map(table_line)
        .filter(|line| {
            table_cells(line).is_some() && has_semantic_capability(&semantic_words(line), "020")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        cap020_matrix_rows,
        [CAP023_CPU_MATRIX_ROW],
        "CAP-020/CAP-021 history and CAP-023 product evidence must share the sole CPU row"
    );
    let stable_rows = language_feature_rows
        .iter()
        .filter(|(_, cells)| {
            cells
                .last()
                .is_some_and(|classification| classification.eq_ignore_ascii_case("stable"))
        })
        .map(|(line, _)| *line)
        .collect::<Vec<_>>();
    assert_eq!(
        stable_rows,
        [
            "| Selected `stable-scalar-v0` profile (accepted `CAP-009`) | Y | Y | Y | Y | Y | — | Y | Y | Y | Y | Y | Y | Y | STABLE |"
        ],
        "stable-scalar-v0 must remain the only STABLE language-feature row"
    );
    let end_to_end_rows = language_feature_rows
        .iter()
        .filter(|(_, cells)| {
            cells
                .last()
                .is_some_and(|classification| classification.eq_ignore_ascii_case("end_to_end"))
        })
        .map(|(line, _)| *line)
        .collect::<Vec<_>>();
    assert_eq!(
        end_to_end_rows,
        [CAP019_SELECTED_PROFILE_MATRIX_ROW],
        "exact-i32-array-v0 must remain the only END_TO_END language-feature row"
    );
    assert!(language_features.contains(
        "| Integers/floats and arithmetic | Y | P | Y | — | P | — | P | P | P | Y | P | P | Y | PARTIAL |"
    ));
    assert!(language_features.contains(
        "| Fixed arrays | Y | Y | Y | P | P | P | P | P | P | Y | P | P | Y | PARTIAL |"
    ));
    for (label, expected) in [
        (
            "Integers/floats and arithmetic",
            "| Integers/floats and arithmetic | Y | P | Y | — | P | — | P | P | P | Y | P | P | Y | PARTIAL |",
        ),
        (
            "Fixed arrays",
            "| Fixed arrays | Y | Y | Y | P | P | P | P | P | P | Y | P | P | Y | PARTIAL |",
        ),
    ] {
        let matches = language_feature_rows
            .iter()
            .filter(|(_, cells)| {
                cells
                    .first()
                    .is_some_and(|actual| actual.eq_ignore_ascii_case(label))
            })
            .map(|(line, _)| *line)
            .collect::<Vec<_>>();
        assert_eq!(
            matches,
            [expected],
            "matrix must preserve one canonical {label} classification row"
        );
    }
    assert!(matrix.contains(
        "| Representative scalar application/conformance subset (`M1-001`, enriched by accepted `CAP-015`) | Y | Y | Verified LLVM plus exact Linux/Windows native output and exit 91 | Y | Y | Y | END_TO_END |"
    ));
    let cap015_matrix_rows = matrix
        .lines()
        .map(table_line)
        .filter(|line| line.starts_with('|') && line.to_ascii_lowercase().contains("cap-015"))
        .collect::<Vec<_>>();
    assert_eq!(
        cap015_matrix_rows,
        [
            "| Representative scalar application/conformance subset (`M1-001`, enriched by accepted `CAP-015`) | Y | Y | Verified LLVM plus exact Linux/Windows native output and exit 91 | Y | Y | Y | END_TO_END |"
        ],
        "CAP-015 must classify exactly once, as M1-001 representative integration"
    );
    assert!(matrix.contains(
        "The selected-profile row is `END_TO_END`; broad integers and\nfixed arrays remain `PARTIAL`, and `stable-scalar-v0` remains the only `STABLE`"
    ));
    assert!(matrix.contains("b62696272f293f9f378f8a368cc818fcb8ef1074"));
    assert!(matrix.contains("c49ff17cab7fc0e8d4f552a71499929135c16c61"));
    assert!(!matrix.contains("`CAP-015-READINESS`"));
    assert!(
        !matrix.contains("Latest accepted compiler-capability master is protected CAP-011 merge")
    );
    assert!(!matrix.contains("CAP-012 is a candidate, not an accepted row"));
    assert!(
        !matrix.contains("Latest accepted compiler-capability master is protected CAP-012 merge")
    );
    assert!(
        !matrix.contains("Latest accepted compiler-capability master is protected CAP-013 merge")
    );

    let roadmap_source = repository_file("Roadmap.md");
    let roadmap = markdown_outside_fences(&roadmap_source);
    assert!(roadmap.contains("CAP-010 is an accepted Milestone 2 capability"));
    assert!(!roadmap.contains("CAP-010 is a local green Milestone 2 candidate"));
    assert!(roadmap.contains("CAP-011 is an accepted Milestone 2 capability"));
    assert!(!roadmap.contains("CAP-011 is the current local Milestone 2 candidate"));
    assert!(roadmap.contains("CAP-012 is an accepted Milestone 2 capability"));
    assert!(roadmap.contains("CAP-013 is an accepted cross-capability architecture"));
    assert!(
        roadmap
            .contains("CAP-014 is accepted as the first bounded Milestone 3 CPU computation slice")
    );
    assert!(roadmap.contains(
        "CAP-015 is accepted as a bounded representative application integration checkpoint"
    ));
    assert!(
        roadmap.contains("CAP-018 is accepted as immutable exact-array value/result composition")
    );
    assert!(roadmap.contains("CAP-019 is accepted as initialized mutable exact-array production"));
    assert!(roadmap.contains(
        "CAP-020 is accepted as a zero-production flat-buffer 2x3-by-3 matvec product gate"
    ));
    assert!(normalized_words(&roadmap).contains(
        "CAP-021 is accepted as a zero-production source-embedded two-stage exact-i32 scoring product gate"
    ));
    assert!(normalized_words(&roadmap).contains("CAP-023 is accepted"));
    assert!(normalized_words(&roadmap).contains("CAP-024 is accepted"));
    for historical_ranking in [
        "### ROADMAP-001 ranked gaps and M1-001 outcome\n\nScores are 1--5 with higher better; `Risk` and `Evidence` are delivery favorability,\nso 5 means lower risk or lower evidence cost.\n\n| Rank | Gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Risk | Evidence | Total |\n|---:|---|---:|---:|---:|---:|---:|---:|---:|\n| 1 | Representative scalar application plus frozen subset and optimization-equivalence gate (accepted as M1-001) | 4 | 5 | 5 | 5 | 4 | 3 | 26 |\n| 2 | Canonical Milestone 0 diagnostic/artifact and trusted-entrypoint contract | 3 | 5 | 5 | 5 | 3 | 3 | 24 |\n| 3 | Positive import/module name resolution after namespace and graph semantics are frozen | 5 | 3 | 5 | 4 | 2 | 2 | 21 |",
        "### Post-M1 ranking and accepted CAP-001\n\nThe required post-M1 comparison is complete. Scores retain the same 1--5 convention;\n`Risk` and `Evidence` reward more favorable delivery.\n\n| Rank | Gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Risk | Evidence | Total |\n|---:|---|---:|---:|---:|---:|---:|---:|---:|\n| 1 | Verified runtime reads from fixed arrays (accepted `CAP-001`) | 5 | 4 | 5 | 5 | 3 | 4 | 26 |\n| 2 | Canonical Milestone 0 diagnostic/artifact and trusted-entrypoint contract | 3 | 5 | 5 | 5 | 3 | 3 | 24 |\n| 3 | Positive import/module name resolution after namespace and graph semantics are frozen | 5 | 3 | 5 | 4 | 2 | 2 | 21 |",
        "### Post-CAP-001 ranking and accepted CAP-002\n\nThe CAP-001 accepted-truth synchronization is complete. A fresh comparison uses the\nsame 1--5 scoring convention; `Risk` and `Evidence` reward more favorable delivery.\n\n| Rank | Gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Risk | Evidence | Total |\n|---:|---|---:|---:|---:|---:|---:|---:|---:|\n| 1 | Checked runtime-indexed fixed-array assignment (accepted `CAP-002`) | 5 | 4 | 5 | 5 | 3 | 4 | 26 |\n| 2 | Canonical Milestone 0 diagnostic/artifact and trusted-entrypoint contract | 3 | 5 | 5 | 5 | 3 | 3 | 24 |\n| 3 | Positive import/module name resolution after namespace and graph semantics are frozen | 5 | 3 | 5 | 4 | 2 | 2 | 21 |",
        "### Post-CAP-002 ranking and accepted CAP-003\n\nThe CAP-002 accepted-truth synchronization and corrective milestone audit selected a\nbroader ordinary-program capability rather than another reference or index topology.\nThe comparison used the same 1--5 scoring convention; `Risk` and `Evidence` reward\nmore favorable delivery.\n\n| Rank | Gap | Real-program usefulness | Roadmap criticality | Architectural leverage | Correctness/safety | Risk | Evidence | Total |\n|---:|---|---:|---:|---:|---:|---:|---:|---:|\n| 1 | Explicitly typed `Option`/`Result` construction, transport, and exhaustive `Match` (accepted `CAP-003`) | 5 | 5 | 5 | 5 | 2 | 3 | 25 |\n| 2 | Canonical Milestone 0 diagnostic/artifact and trusted-entrypoint contract | 3 | 5 | 5 | 5 | 3 | 3 | 24 |\n| 3 | Positive import/module name resolution after namespace and graph semantics are frozen | 5 | 3 | 5 | 4 | 2 | 2 | 21 |",
    ] {
        assert_eq!(
            roadmap.matches(historical_ranking).count(),
            1,
            "Roadmap.md must preserve each historical ranking record exactly once"
        );
    }
    assert!(roadmap.contains("b62696272f293f9f378f8a368cc818fcb8ef1074"));
    assert!(roadmap.contains("c49ff17cab7fc0e8d4f552a71499929135c16c61"));
    assert!(!roadmap.contains("`CAP-015-READINESS`"));
    assert!(!roadmap.contains("product task is an explicitly profiled exact fixed-width integer"));
    assert!(roadmap.contains("selected Milestone 2 exit gate is met"));
    assert!(!roadmap.contains("CAP-012 is the current candidate, not accepted capability"));
    assert!(!roadmap.contains("The milestone remains open because no"));
    let normalized_roadmap = normalized_words(&roadmap);
    assert!(normalized_roadmap.contains(
        "Scores are 1--5 with higher better; `Risk` and `Evidence` are delivery favorability, so 5 means lower implementation risk or lower evidence cost."
    ));
    let historical_ranking_heading = "### Post-CAP-020 ranking";
    assert_eq!(
        roadmap.matches(historical_ranking_heading).count(),
        1,
        "Roadmap.md must preserve one historical post-CAP-020 ranking section"
    );
    let historical_tail = roadmap_source
        .split_once(historical_ranking_heading)
        .expect("unique historical post-CAP-020 ranking heading")
        .1;
    let historical_section = historical_tail
        .split_once("\n### ")
        .map_or(historical_tail, |(section, _)| section);
    assert_post_cap020_ranking_table(
        "Roadmap.md historical Post-CAP-020 section",
        historical_section,
    );
    assert_post_cap020_successor_order(
        "Roadmap.md historical Post-CAP-020 section",
        historical_section,
    );
    assert_exact_ordered_decision_records(
        "Roadmap.md historical Post-CAP-020 section",
        historical_section,
        &POST_CAP020_DECISION_CONTRACTS,
    );
    let normalized_historical_section =
        normalized_words(&markdown_outside_fences(historical_section));
    let historical_product_authorization = "The ranking favors a material record-to-kernel composition before crossing the runtime boundary. Rank 1 is the only executable product authorization; ranks 2 and 3 remain readiness/probe decisions, and the stopped module and propagation designs stay closed.";
    assert!(
        normalized_historical_section.contains(historical_product_authorization),
        "Roadmap.md must preserve the bounded historical rank-1 product authorization"
    );
    let historical_readiness_surface =
        normalized_historical_section.replacen(historical_product_authorization, "", 1);
    assert!(
        readiness_promotion_violation_from_rendered(&historical_readiness_surface).is_none(),
        "Roadmap.md adds a readiness-to-implementation promotion inside preserved post-CAP-020 history: {:?}",
        readiness_promotion_violation_from_rendered(&historical_readiness_surface)
    );
    let mut previous_historical_position = None;
    for contract in POST_CAP020_DECISION_CONTRACTS {
        assert_eq!(
            normalized_historical_section.matches(contract).count(),
            1,
            "Roadmap.md must preserve each historical post-CAP-020 decision contract: {contract}"
        );
        let position = normalized_historical_section
            .find(contract)
            .expect("historical post-CAP-020 decision contract position");
        if let Some(previous) = previous_historical_position {
            assert!(
                previous < position,
                "Roadmap.md reorders the historical post-CAP-020 decision contracts"
            );
        }
        previous_historical_position = Some(position);
    }

    let ranking_heading = "### Post-CAP-021 ranking";
    assert_eq!(
        roadmap.matches(ranking_heading).count(),
        1,
        "Roadmap.md must preserve one historical post-CAP-021 ranking section"
    );
    let ranking_tail = roadmap_source
        .split_once(ranking_heading)
        .expect("unique post-CAP-021 ranking heading")
        .1;
    let ranking_section = ranking_tail
        .split_once("\n### Post-CAP-023 ranking")
        .map_or(ranking_tail, |(section, _)| section);
    let normalized_ranking_section = normalized_words(&markdown_outside_fences(ranking_section));
    assert_post_cap021_ranking_table(
        "Roadmap.md historical Post-CAP-021 section",
        ranking_section,
    );
    assert_post_cap021_successor_order(
        "Roadmap.md historical Post-CAP-021 section",
        ranking_section,
    );
    assert_exact_ordered_decision_records(
        "Roadmap.md historical Post-CAP-021 section",
        ranking_section,
        &POST_CAP021_DECISION_CONTRACTS,
    );
    assert!(
        readiness_promotion_violation(ranking_section).is_none(),
        "Roadmap.md promotes a successor inside preserved post-CAP-021 history: {:?}",
        readiness_promotion_violation(ranking_section)
    );
    let mut previous_position = None;
    for contract in POST_CAP021_DECISION_CONTRACTS {
        let expected_global_count = if POST_CAP020_DECISION_CONTRACTS.contains(&contract) {
            2
        } else {
            1
        };
        assert_eq!(
            normalized_roadmap.matches(contract).count(),
            expected_global_count,
            "Roadmap.md must preserve each post-CAP-021 decision contract with its scoped historical/current cardinality: {contract}"
        );
        let position = normalized_ranking_section.find(contract).unwrap_or_else(|| {
            panic!("Roadmap.md detaches a decision contract from the post-CAP-021 section: {contract}")
        });
        if let Some(previous) = previous_position {
            assert!(
                position > previous,
                "Roadmap.md reorders the post-CAP-021 decision contracts"
            );
        }
        previous_position = Some(position);
    }

    let historical_cap023_heading = "### Post-CAP-023 ranking";
    assert_eq!(
        roadmap.matches(historical_cap023_heading).count(),
        1,
        "Roadmap.md must preserve one historical post-CAP-023 ranking section"
    );
    let historical_cap023_tail = roadmap_source
        .split_once(historical_cap023_heading)
        .expect("unique post-CAP-023 ranking heading")
        .1;
    let historical_cap023_section = historical_cap023_tail
        .split_once("\n### Post-CAP-024 ranking")
        .map_or(historical_cap023_tail, |(section, _)| section);
    let normalized_historical_cap023_section =
        normalized_words(&markdown_outside_fences(historical_cap023_section));
    assert_post_cap023_ranking_table(
        "Roadmap.md historical Post-CAP-023 section",
        historical_cap023_section,
    );
    assert_post_cap023_successor_order(
        "Roadmap.md historical Post-CAP-023 section",
        historical_cap023_section,
    );
    assert_exact_ordered_decision_records(
        "Roadmap.md historical Post-CAP-023 section",
        historical_cap023_section,
        &POST_CAP023_DECISION_CONTRACTS,
    );
    assert!(
        post_cap023_readiness_promotion_violation(historical_cap023_section).is_none(),
        "Roadmap.md promotes a successor inside preserved post-CAP-023 history: {:?}",
        post_cap023_readiness_promotion_violation(historical_cap023_section)
    );
    let mut previous_position = None;
    for contract in POST_CAP023_DECISION_CONTRACTS {
        assert_eq!(
            normalized_roadmap.matches(contract).count(),
            1,
            "Roadmap.md must preserve each historical post-CAP-023 decision contract exactly once: {contract}"
        );
        let position = normalized_historical_cap023_section
            .find(contract)
            .unwrap_or_else(|| {
                panic!("Roadmap.md detaches a decision contract from the historical post-CAP-023 section: {contract}")
            });
        if let Some(previous) = previous_position {
            assert!(
                position > previous,
                "Roadmap.md reorders the historical post-CAP-023 decision contracts"
            );
        }
        previous_position = Some(position);
    }

    let current_ranking_heading = "### Post-CAP-024 ranking";
    assert_eq!(
        roadmap.matches(current_ranking_heading).count(),
        1,
        "Roadmap.md must contain one current post-CAP-024 ranking section"
    );
    let current_ranking_tail = roadmap_source
        .split_once(current_ranking_heading)
        .expect("unique post-CAP-024 ranking heading")
        .1;
    let current_ranking_section = current_ranking_tail
        .split_once("\n## ")
        .map_or(current_ranking_tail, |(section, _)| section);
    let normalized_current_ranking_section =
        normalized_words(&markdown_outside_fences(current_ranking_section));
    assert_exact_ordered_decision_records(
        "Roadmap.md current Post-CAP-024 section",
        current_ranking_section,
        &POST_CAP024_DECISION_CONTRACTS,
    );
    assert!(
        post_cap024_readiness_promotion_violation(current_ranking_section).is_none(),
        "Roadmap.md promotes a post-CAP-024 successor to implementation: {:?}",
        post_cap024_readiness_promotion_violation(current_ranking_section)
    );
    let mut previous_position = None;
    for contract in POST_CAP024_DECISION_CONTRACTS {
        assert_eq!(
            normalized_roadmap.matches(contract).count(),
            1,
            "Roadmap.md must state each current post-CAP-024 decision contract exactly once: {contract}"
        );
        let position = normalized_current_ranking_section.find(contract).unwrap_or_else(|| {
            panic!("Roadmap.md detaches a decision contract from the post-CAP-024 section: {contract}")
        });
        if let Some(previous) = previous_position {
            assert!(
                position > previous,
                "Roadmap.md reorders the post-CAP-024 decision contracts"
            );
        }
        previous_position = Some(position);
    }
    let mut previous_heading = None;
    for heading in [
        "### ROADMAP-001 ranked gaps and M1-001 outcome",
        "### Post-M1 ranking and accepted CAP-001",
        "### Post-CAP-001 ranking and accepted CAP-002",
        "### Post-CAP-002 ranking and accepted CAP-003",
        "### Post-CAP-020 ranking",
        "### Post-CAP-021 ranking",
        "### Post-CAP-023 ranking",
        "### Post-CAP-024 ranking",
    ] {
        assert_eq!(
            roadmap.matches(heading).count(),
            1,
            "Roadmap.md must preserve exactly one historical/current ranking heading: {heading}"
        );
        let position = roadmap
            .find(heading)
            .expect("required Roadmap ranking heading");
        if let Some(previous) = previous_heading {
            assert!(
                position > previous,
                "Roadmap.md reorders historical/current ranking sections"
            );
        }
        previous_heading = Some(position);
    }

    let conformance_source = repository_file("CONFORMANCE_PLAN.md");
    let conformance = markdown_outside_fences(&conformance_source);

    assert_core090_accepted_partial_history(
        &readme_source,
        &project_state_source,
        &audit_source,
        &matrix_source,
        &alignment_source,
        &roadmap_source,
    );

    let normalized_task_ledger = normalized_words(&task_ledger);
    assert!(
        normalized_task_ledger
            .contains("CAP-015 remains the M1-001 representative-integration checkpoint")
    );
    assert!(!task_ledger.contains("CAP-015 remains the M1-001 parser/integration checkpoint"));
    assert!(
        conformance
            .contains("Accepted CAP-010 adds one required-only trait-dispatch conformance slice")
    );
    assert!(conformance.contains("Accepted CAP-011 passes focused 4/4"));
    assert!(conformance.contains("accepted CAP-012 projected-call-loan slice"));
    assert!(conformance.contains(
        "Accepted CAP-012 adds one nonescaping projected CopyData call-loan conformance slice"
    ));
    assert!(conformance.contains(
        "Accepted CAP-013 adds one canonical specialization identity/order conformance slice"
    ));
    assert!(conformance.contains("accepted CAP-014 `exact-i32-array-v0` slice"));
    assert!(conformance.contains("Accepted CAP-014 adds one selected `exact-i32-array-v0`"));
    assert!(
        conformance
            .contains("Accepted CAP-015 enriches the existing M1-001 representative application")
    );
    assert!(conformance.contains("Accepted CAP-018 widens the existing `exact-i32-array-v0` lane"));
    assert!(conformance.contains("Accepted CAP-019 widens the existing `exact-i32-array-v0` lane"));
    assert!(conformance.contains(
        "Accepted CAP-020 adds one zero-production flat-buffer 2x3-by-3 matvec product gate"
    ));
    assert!(conformance.contains(
        "Accepted CAP-021 adds one zero-production source-embedded flat-record two-stage scoring product gate"
    ));
    assert!(
        normalized_words(&conformance).contains(
            "Accepted CAP-021 adds one zero-production source-embedded flat-record two-stage scoring product gate to the maintained conformance evidence. It is product evidence only"
        ),
        "CONFORMANCE_PLAN.md must preserve CAP-021 as zero-production product evidence only"
    );
    assert!(conformance.contains("This selected lane is `END_TO_END`"));
    assert!(conformance.contains("`stable-scalar-v0` remains the only `STABLE` profile"));
    assert!(conformance.contains("b62696272f293f9f378f8a368cc818fcb8ef1074"));
    assert!(conformance.contains("c49ff17cab7fc0e8d4f552a71499929135c16c61"));
    assert!(!conformance.contains("`CAP-015-READINESS`"));
    for (document_name, document) in [
        ("README.md", readme_source.as_str()),
        ("CURRENT_CAPABILITY_AUDIT.md", audit_source.as_str()),
        ("FRAMEWORK_ALIGNMENT.md", alignment_source.as_str()),
        ("PROJECT_STATE.md", project_state_source.as_str()),
        ("SPEC_IMPLEMENTATION_MATRIX.md", matrix_source.as_str()),
        ("Roadmap.md", roadmap_source.as_str()),
        ("CONFORMANCE_PLAN.md", conformance_source.as_str()),
    ] {
        assert_cap014_acceptance_evidence(document_name, document);
        assert_cap015_acceptance_evidence(document_name, document);
        assert_cap018_acceptance_evidence(document_name, document);
        assert_cap019_acceptance_evidence(document_name, document);
        assert_cap020_acceptance_evidence(document_name, document);
        assert_cap021_acceptance_evidence(document_name, document);
        assert_cap023_acceptance_evidence(document_name, document);
        assert_cap024_acceptance_evidence(document_name, document);
        assert_cap020_boundaries(document_name, document);
        assert_cap021_boundaries(document_name, document);
        assert_cap023_boundaries(document_name, document);
        assert_cap024_boundaries(document_name, document);
    }
    for (document_name, document, expected_boundaries) in [
        ("README.md", readme.as_str(), 1),
        ("CURRENT_CAPABILITY_AUDIT.md", audit.as_str(), 1),
        ("FRAMEWORK_ALIGNMENT.md", alignment.as_str(), 1),
        ("PROJECT_STATE.md", project_state.as_str(), 1),
        ("SPEC_IMPLEMENTATION_MATRIX.md", matrix.as_str(), 2),
        ("Roadmap.md", roadmap.as_str(), 1),
        ("CONFORMANCE_PLAN.md", conformance.as_str(), 2),
    ] {
        let normalized = normalized_words(document);
        assert!(
            normalized.contains("CAP-015 changes no compiler production or language-profile code"),
            "{document_name} blurs the CAP-015 integration/compiler boundary"
        );
        assert_eq!(
            normalized.matches(CAP015_PRODUCT_BOUNDARY).count(),
            expected_boundaries,
            "{document_name} does not preserve every CAP-015 product-claim boundary"
        );
        assert!(
            !normalized
                .to_ascii_lowercase()
                .contains(STALE_CAP015_PRODUCT_BOUNDARY),
            "{document_name} resurrects the overbroad String/Unicode denial"
        );
    }
    let cap024_successor_documents = [
        ("README.md", readme_source.as_str()),
        ("CURRENT_CAPABILITY_AUDIT.md", audit_source.as_str()),
        ("FRAMEWORK_ALIGNMENT.md", alignment_source.as_str()),
        ("PROJECT_STATE.md", project_state_source.as_str()),
        ("SPEC_IMPLEMENTATION_MATRIX.md", matrix_source.as_str()),
        ("Roadmap.md", roadmap_source.as_str()),
        ("CONFORMANCE_PLAN.md", conformance_source.as_str()),
    ];
    assert_eq!(
        cap024_successor_documents.len(),
        7,
        "the CAP-024 successor-order contract must cover all seven cumulative truth documents"
    );
    for (document_name, document) in cap024_successor_documents {
        assert_post_cap024_successor_order(document_name, document);
        assert_exact_ordered_decision_records(
            document_name,
            document,
            &POST_CAP024_DECISION_CONTRACTS,
        );
        if document_name != "Roadmap.md" {
            let normalized = normalized_words(&markdown_outside_fences(document));
            for historical in POST_CAP023_DECISION_CONTRACTS {
                assert_eq!(
                    normalized.matches(historical).count(),
                    0,
                    "{document_name} retains a post-CAP-023 decision outside Roadmap history: {historical}"
                );
            }
        }
    }
    for (document_name, document) in [
        ("README.md", readme_source.as_str()),
        ("CURRENT_CAPABILITY_AUDIT.md", audit_source.as_str()),
        ("FRAMEWORK_ALIGNMENT.md", alignment_source.as_str()),
        ("PROJECT_STATE.md", project_state_source.as_str()),
        ("SPEC_IMPLEMENTATION_MATRIX.md", matrix_source.as_str()),
        ("Roadmap.md", roadmap_source.as_str()),
        ("CONFORMANCE_PLAN.md", conformance_source.as_str()),
    ] {
        assert_post_cap024_ranking_table(document_name, document);
    }
    let classified_matrix = matrix
        .split_once("## Language features")
        .expect("matrix language-feature section")
        .1
        .split_once("## Evidence notes")
        .expect("matrix evidence notes")
        .0;
    let cap021_matrix_rows = classified_matrix
        .lines()
        .map(table_line)
        .filter(|line| {
            table_cells(line).is_some() && has_semantic_capability(&semantic_words(line), "021")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        cap021_matrix_rows,
        [CAP023_CPU_MATRIX_ROW],
        "CAP-021 may appear in exactly one matrix row, as evidence in the existing CPU platform row"
    );
    let cap023_matrix_rows = classified_matrix
        .lines()
        .map(table_line)
        .filter(|line| {
            table_cells(line).is_some() && has_semantic_capability(&semantic_words(line), "023")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        cap023_matrix_rows,
        [CAP023_CPU_MATRIX_ROW],
        "CAP-023 may appear in exactly one matrix row, as evidence in the existing CPU PARTIAL row"
    );
    let cap024_matrix_rows = classified_matrix
        .lines()
        .map(table_line)
        .filter(|line| {
            table_cells(line).is_some() && has_semantic_capability(&semantic_words(line), "024")
        })
        .collect::<Vec<_>>();
    assert!(
        cap024_matrix_rows.is_empty(),
        "CAP-024 must be absent from every classified feature/profile/backend row: {cap024_matrix_rows:?}"
    );
    let backend_summary = matrix
        .split_once("## Backend summary")
        .expect("matrix backend summary")
        .1
        .split_once("## Evidence notes")
        .expect("matrix evidence notes")
        .0;
    let cpu_rows = backend_summary
        .lines()
        .map(table_line)
        .filter(|line| {
            table_cells(line).is_some_and(|cells| {
                cells
                    .first()
                    .is_some_and(|label| label.eq_ignore_ascii_case("cpu"))
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(
        cpu_rows,
        [CAP023_CPU_MATRIX_ROW],
        "matrix must preserve exactly one CPU row with CAP-020/CAP-021 history and CAP-023 product evidence"
    );
    let normalized_conformance = normalized_words(&conformance);
    assert!(normalized_conformance.contains(CAP014_CONFORMANCE_HISTORY_BOUNDARY));
    assert!(normalized_conformance.contains(CAP018_CONFORMANCE_HISTORY_BOUNDARY));
    assert!(!normalized_conformance.contains(
        "Negative evidence must exhaust the excluded mutable/write/construction/element/profile/ target families"
    ));
    assert!(
        !normalized_conformance
            .contains("Negative separation retains mutable bindings/results and writes, recursion")
    );
    assert!(readme.contains(
        "General generic operations/impls/traits beyond those bounded classes, inference/defaults, broader trait-bound enforcement, and where-clause semantics remain parsed, quarantined, or unsupported."
    ));
    assert!(readme.contains(
        "No general borrow checker, general mutable-reference model, lifetime analysis, drop model, stable pointer ABI, or memory-safety guarantee."
    ));
    assert!(readme.contains("3 example cases and 4 deterministic regression checks"));
    assert!(!readme.contains("formal conformance + mechanized checks"));
    assert!(
        !readme
            .contains("| **Type System** | Static typing, generics, trait bounds, where clauses |")
    );
    assert!(!readme.contains(
        "| **Memory** | Ownership, move semantics, shared & mutable references, borrow checker |"
    ));

    let claude = repository_file("CLAUDE.md");
    assert!(claude.contains("## Current Evidence Status"));
    assert!(claude.contains("Minimal prototype / correctness recovery"));
    assert!(!claude.contains("## Current Phase: Phase 5 (Advanced Features) — COMPLETE"));
    assert!(!claude.contains("Ownership and move semantics (DONE)"));
    assert!(!claude.contains("References and borrowing syntax (DONE)"));
    assert!(!claude.contains("Borrow checker enforcement (DONE)"));
    assert!(!claude.contains("Generics: type params, trait bounds, where clauses (DONE)"));
    assert!(!claude.contains("Traits: registry, completeness checking, bound enforcement (DONE)"));
    assert!(
        !claude.contains("174 tests passing (63 unit + 52 optimizer + 59 frontend integration)")
    );
    assert!(!claude.contains("37/38 Phase 5 spec tests passing"));
    assert!(!claude.contains("## Completed Phases"));
    assert!(!claude.contains(
        "Phase 5: Advanced Features (ownership, borrowing, borrow checker, generics, traits)"
    ));

    let tutorial_one = repository_file("tutorials/01-getting-started.md");
    assert!(tutorial_one.contains("deterministic regression checks"));
    assert!(tutorial_one.contains("not a formal semantics proof"));
    assert!(tutorial_one.contains("conceptual ownership and borrowing design"));
    assert!(!tutorial_one.contains("conformance and mechanized checks"));
    assert!(!tutorial_one.contains("against the formal suite"));
    assert!(!tutorial_one.contains("Aero's key memory safety feature"));

    let tutorial_two = repository_file("tutorials/02-core-features.md");
    assert!(tutorial_two.contains("design-only ownership and borrowing model"));
    assert!(!tutorial_two.contains("powerful features for memory safety"));

    let tutorial_four = repository_file("tutorials/04-data-structures.md");
    assert!(tutorial_four.contains("**Current implementation boundary:**"));
}

#[test]
fn normative_safety_documents_are_visibly_design_targets() {
    for path in [
        "docs/language/aero_formal_language_specification.md",
        "docs/language/aero_type_system.md",
        "docs/language/aero_ownership_borrowing.md",
        "tutorials/03-ownership-borrowing.md",
    ] {
        let document = repository_file(path);
        let introduction = document.lines().take(12).collect::<Vec<_>>().join("\n");
        assert!(
            introduction.contains("**Design target — not current implementation evidence.**"),
            "missing leading design-target notice in {path}"
        );
    }

    let formal = repository_file("docs/language/aero_formal_language_specification.md");
    assert!(formal.contains("v1.0.0 remains a language design target"));
    assert!(formal.contains("not the compiler package version"));
    assert!(formal.contains("not a conformance or stability claim"));
}

#[test]
fn grammar_and_core_tutorial_are_visibly_design_targets() {
    let mut missing = Vec::new();
    for path in [
        "docs/language/aero_grammar.md",
        "tutorials/02-core-features.md",
    ] {
        let document = repository_file(path);
        let introduction = document.lines().take(12).collect::<Vec<_>>().join("\n");
        for (requirement, expected) in [
            (
                "leading design-target marker",
                "**Design target — not current implementation evidence.**",
            ),
            ("v1 design authority", "Aero v1.0.0 design target"),
            (
                "current compiler boundary",
                "not the currently implemented compiler subset",
            ),
            (
                "conformance and stability boundary",
                "not conformance or stability evidence",
            ),
            ("current capability audit", "CURRENT_CAPABILITY_AUDIT.md"),
            ("implementation matrix", "SPEC_IMPLEMENTATION_MATRIX.md"),
        ] {
            if !introduction.contains(expected) {
                missing.push(format!("{path}: missing {requirement}"));
            }
        }
    }

    let grammar = repository_file("docs/language/aero_grammar.md");
    let unqualified_authority =
        "definitive guide for implementing the lexer and parser components of the Aero compiler";
    if grammar.contains(unqualified_authority) {
        missing
            .push("docs/language/aero_grammar.md: unqualified compiler authority remains".into());
    }
    let normative_boundary = "Every EBNF production below is part of the normative Aero v1.0.0 design target, not a statement of current compiler conformance.";
    if !grammar.contains(normative_boundary) {
        missing.push("docs/language/aero_grammar.md: missing normative v1 boundary".into());
    }

    assert!(
        missing.is_empty(),
        "missing grammar/tutorial authority boundaries:\n{}",
        missing.join("\n")
    );
}

#[test]
fn historical_completion_records_are_visibly_archived() {
    for path in [
        "todo.md",
        "docs/demos/builtin_collections_demo.md",
        "docs/demos/collection_string_demo.md",
        "docs/demos/enum_pattern_demo.md",
        "docs/demos/struct_generation_demo.md",
        "docs/tasks/TASK_10_1_LLVM_STRUCT_GENERATION_SUMMARY.md",
        "docs/tasks/TASK_10_2_ENUM_PATTERN_IR_GENERATION_SUMMARY.md",
        "docs/tasks/TASK_10_3_COLLECTION_STRING_GENERATION_SUMMARY.md",
        "docs/tasks/TASK_11_BUILTIN_COLLECTIONS_LIBRARY_SUMMARY.md",
    ] {
        let document = repository_file(path);
        let document = document
            .lines()
            .take(12)
            .flat_map(str::split_whitespace)
            .filter(|word| *word != ">")
            .collect::<Vec<_>>()
            .join(" ")
            .to_ascii_lowercase();
        assert!(
            document.contains("historical"),
            "missing leading historical label in {path}"
        );
        assert!(
            document.contains("not current")
                || document.contains("not active")
                || document.contains("not evidence")
                || document.contains("does not describe the active"),
            "missing leading non-capability qualification in {path}"
        );
    }
}

#[test]
fn repository_remains_explicitly_experimental_without_stability_claims() {
    let readme = repository_file("README.md");
    assert!(readme.contains("Experimental systems language and compiler repository"));
    assert!(readme.contains("minimal prototype under correctness recovery"));

    let project_state = repository_file("PROJECT_STATE.md");
    assert!(project_state.contains("Repository stability: experimental"));
    assert!(
        project_state.contains("Formal conformance: three example cases plus four deterministic")
    );
    assert!(
        !project_state
            .contains("Publish this bounded record-only CAP-004 accepted-truth synchronization")
    );
    assert!(
        project_state
            .contains("CAP-013 accepted: canonical specialization identity and phase authority")
    );
    assert!(
        project_state.contains("CAP-014 accepted: exact `i32` fixed-array CPU reference kernel")
    );
    assert!(
        project_state.contains("CAP-005 accepted: bound-free CopyData generic transport functions")
    );
    assert!(project_state.contains("59f7e47b476871fae8cecdf7e40900e0d1f1b377"));
    assert!(!project_state.contains("CAP-005 exact local candidate (not yet accepted)"));
    assert!(
        project_state.contains("CAP-006 accepted: explicit user-defined generic CopyData enums")
    );
    assert!(project_state.contains("bdfd4f5a282043ee957c1bf03975e266de5b9b6c"));
    assert!(normalized_words(&project_state).contains(CAP016_CAP017_STOP_BOUNDARY));
    assert!(!project_state.contains("`CAP-015-READINESS`"));
    assert!(!project_state.contains("exact next action is this bounded"));
}
