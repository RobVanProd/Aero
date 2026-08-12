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

const CAP020_CPU_MATRIX_ROW: &str = "| CPU | Y | Y | P | P | P; pinned Linux and bounded Windows x86_64 evidence accepted, including CAP-014 exact-i32-array-v0 kernel/wrapping/read-trap gates, CAP-018 immutable result composition, CAP-019 initialized mutable-local/result production with guarded projected writes and negative/equal write traps, and CAP-020 flat-buffer 2x3-by-3 matvec product with identity-linked guarded [6]/[3]/[2] access and exact ordinary/wrapping/native oracles | P | P | PARTIAL |";

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

fn table_cells(line: &str) -> Option<Vec<&str>> {
    let line = table_line(line);
    line.starts_with('|')
        .then(|| line.trim_matches('|').split('|').map(str::trim).collect())
}

fn assert_bounded_acceptance_evidence(
    document_name: &str,
    document: &str,
    capability: &str,
    identities: &[&str],
    require_order: bool,
) {
    let paragraphs = normalized_markdown_paragraphs(document);
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
        if matches!(capability, "CAP-018" | "CAP-019" | "CAP-020") {
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
        _ => 700,
    };
    assert!(
        cursor - start < maximum_span,
        "{document_name} detaches the {capability} evidence identities"
    );
    let conclusion = paragraph[cursor..].trim_start();
    if matches!(capability, "CAP-019" | "CAP-020") {
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
    if matches!(capability, "CAP-018" | "CAP-019" | "CAP-020") {
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
    let semantic = semantic_words(document);
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
    let source_lines = document.lines().map(table_line).collect::<Vec<_>>();
    let rows = document
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
    let normalized = normalized_words(document);
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
    let evidence_paragraphs = normalized_markdown_paragraphs(document)
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
    let normalized = normalized_words(document);
    assert_eq!(
        normalized
            .matches("Exact CAP-020 reviewed candidate")
            .count(),
        1,
        "{document_name} must contain exactly one CAP-020 evidence lead-in"
    );
    assert_eq!(
        normalized.matches(CAP020_EVIDENCE_PREFIX).count(),
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
    let evidence_paragraphs = normalized_markdown_paragraphs(document)
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

fn assert_post_cap020_successor_order(document_name: &str, document: &str) {
    let normalized = document.to_ascii_lowercase();
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

fn assert_no_stale_cap019_current_claims(document_name: &str, document: &str) {
    for paragraph in normalized_markdown_paragraphs(document) {
        for clause in paragraph.split(['.', ';', '!', '?']) {
            let words = semantic_words(clause);
            if !has_semantic_capability(&words, "019") {
                continue;
            }
            let has = |word: &str| words.iter().any(|candidate| candidate == word);
            let stale = (has("master") && (has("current") || has("latest") || has("baseline")))
                || (has("baseline") && has("protected"))
                || (has("project") && has("status") && has("after"))
                || (has("post") && (has("ranking") || has("order")));
            assert!(
                !stale,
                "{document_name} presents CAP-019 as current state or current ranking: {clause}"
            );
        }
    }
}

fn assert_no_cap020_overclaims(document_name: &str, document: &str) {
    let subjects: &[&[&str]] = &[
        &["parser"],
        &["grammar"],
        &["source", "semantics"],
        &["language", "profile"],
        &["compiler", "profile"],
        &["profile"],
        &["language", "feature"],
        &["feature", "row"],
        &["capability", "classification"],
        &["semantic", "analysis"],
        &["checked", "ir"],
        &["verifier"],
        &["backend"],
        &["compiler", "production"],
        &["matrix", "type"],
        &["matrix", "support"],
        &["tensor", "type"],
        &["tensor", "support"],
        &["binary", "ingestion"],
        &["quantization"],
        &["inference", "completion"],
        &["stable", "abi"],
        &["abi", "stability"],
        &["stable", "layout"],
        &["layout", "guarantee"],
        &["performance"],
        &["accelerator", "execution"],
        &["accelerator", "support"],
        &["memory", "safety"],
        &["safety", "claim"],
        &["safety"],
        &["recursive", "arrays"],
        &["nested", "arrays"],
        &["static", "index", "proof"],
        &["checked", "overflow", "arithmetic"],
        &["general", "mutation"],
    ];
    let positive_verbs = [
        "is",
        "has",
        "change",
        "changes",
        "changed",
        "add",
        "adds",
        "added",
        "create",
        "creates",
        "created",
        "admit",
        "admits",
        "accept",
        "accepts",
        "support",
        "supports",
        "enable",
        "enables",
        "implement",
        "implements",
        "implemented",
        "widen",
        "widens",
        "widened",
        "stabilize",
        "stabilizes",
        "stabilized",
        "guarantee",
        "guarantees",
        "provide",
        "provides",
        "provided",
        "deliver",
        "delivers",
        "delivered",
        "introduce",
        "introduces",
        "introduced",
        "certify",
        "certifies",
        "complete",
        "completes",
    ];
    for paragraph in normalized_markdown_paragraphs(document) {
        let mut cap020_context = false;
        for clause in paragraph.split(['.', ';', '!', '?']) {
            let words = semantic_words(clause);
            let explicit_cap020 = has_semantic_capability(&words, "020");
            let explicit_other_capability = ["014", "015", "016", "017", "018", "019"]
                .iter()
                .any(|number| has_semantic_capability(&words, number));
            if explicit_cap020 {
                cap020_context = true;
            } else if explicit_other_capability {
                cap020_context = false;
            }
            if !cap020_context {
                continue;
            }
            for start in 0..words.len() {
                for subject in subjects {
                    if start + subject.len() > words.len()
                        || !words[start..start + subject.len()]
                            .iter()
                            .map(String::as_str)
                            .eq(subject.iter().copied())
                    {
                        continue;
                    }
                    let search_start = start.saturating_sub(12);
                    if let Some(verb_index) = (search_start..start)
                        .rev()
                        .find(|index| positive_verbs.contains(&words[*index].as_str()))
                    {
                        let negated = words[verb_index.saturating_sub(3)..start + subject.len()]
                            .iter()
                            .any(|word| {
                                matches!(word.as_str(), "not" | "no" | "without" | "never")
                            });
                        assert!(
                            negated,
                            "{document_name} promotes CAP-020 beyond its product-only boundary: {}",
                            subject.join(" ")
                        );
                    }

                    let subject_end = start + subject.len();
                    if subject_end + 1 < words.len()
                        && matches!(words[subject_end].as_str(), "is" | "are")
                        && matches!(
                            words[subject_end + 1].as_str(),
                            "guaranteed"
                                | "stable"
                                | "complete"
                                | "supported"
                                | "implemented"
                                | "enabled"
                                | "accepted"
                                | "provided"
                                | "certified"
                                | "changed"
                                | "widened"
                                | "added"
                        )
                    {
                        let passive_negated = words
                            [start.saturating_sub(3)..(subject_end + 4).min(words.len())]
                            .iter()
                            .any(|word| {
                                matches!(word.as_str(), "not" | "no" | "without" | "never")
                            });
                        assert!(
                            passive_negated,
                            "{document_name} passively promotes CAP-020 beyond its product-only boundary: {}",
                            subject.join(" ")
                        );
                    }
                }
            }
        }
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
    let normalized = normalized_words(document);
    let normalized_lower = normalized.to_ascii_lowercase();
    assert_no_stale_cap019_current_claims(document_name, document);
    assert_no_cap020_overclaims(document_name, document);
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

    for paragraph in normalized_markdown_paragraphs(document) {
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
            let has_word = |expected: &str| words.contains(&expected);
            if has_capability_token(&words, "cap-020") {
                assert!(
                    !has_word("candidate")
                        && !has_word("pending")
                        && !has_word("unaccepted")
                        && !has_word("proposed")
                        && !has_word("unpublished")
                        && !has_word("unmerged")
                        && !has_word("awaiting")
                        && !has_word("local-only")
                        && !(has_word("local") && has_word("only"))
                        && !(has_word("not") && has_word("published"))
                        && !(has_word("awaits") && has_word("acceptance"))
                        && !(has_word("not") && has_word("accepted")),
                    "{document_name} gives CAP-020 a candidate or unaccepted status: {clause}"
                );
            }
            if has_capability_token(&words, "cap-019") {
                assert!(
                    !has_word("candidate")
                        && !has_word("pending")
                        && !has_word("unaccepted")
                        && !(has_word("not") && has_word("accepted")),
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
                        && !(has_word("not") && has_word("accepted")),
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
    let task_ledger = repository_file("TASK_LEDGER.md");
    let readme = repository_file("README.md");
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
    assert!(readme.contains("Project status after CAP-020"));
    assert!(readme.contains("baseline is protected CAP-020 product merge"));
    assert!(readme.contains("b62696272f293f9f378f8a368cc818fcb8ef1074"));
    assert!(readme.contains(
        "The next action is the source-embedded fixed-shape tensor-record decode plus two-stage flat-buffer exact-`i32` CPU scoring product gate"
    ));
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
    assert!(!readme.contains("CAP-013 candidate (not accepted)"));
    assert!(!readme.contains("CAP-014 candidate (not accepted)"));
    assert!(!readme.contains("`CAP-015-READINESS`"));
    assert!(!readme.contains(
        "next ranked product target is an explicitly profiled exact fixed-width integer"
    ));

    let audit = repository_file("CURRENT_CAPABILITY_AUDIT.md");
    assert!(audit.contains(
        "Accepted CAP-007 closes the canonical checked-entrypoint and artifact mechanism"
    ));
    assert!(!audit.contains(
        "no authoritative stable subset or single canonical diagnostic contract is frozen"
    ));
    assert!(audit.contains("CAP-008 accepted: nonbinding wildcard enum Match"));
    assert!(audit.contains("protected CAP-020 product merge"));
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
    assert!(normalized_words(&audit).contains(
        "The post-CAP-020 order begins with the source-embedded fixed-shape tensor-record decode plus two-stage flat-buffer exact-`i32` CPU scoring product gate"
    ));
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

    let alignment = repository_file("FRAMEWORK_ALIGNMENT.md");
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
    assert!(
        alignment.contains("Source-embedded fixed-shape tensor-record decode plus two-stage flat-buffer exact-`i32` CPU scoring product gate ranks first")
    );
    assert!(alignment.contains("b62696272f293f9f378f8a368cc818fcb8ef1074"));
    assert!(alignment.contains("c49ff17cab7fc0e8d4f552a71499929135c16c61"));
    assert!(!alignment.contains("`CAP-015-READINESS`"));
    assert!(alignment.contains("satisfy the roadmap's selected Milestone 2 exit gate"));
    assert!(alignment.contains("Aero remains\na Minimal Prototype"));
    assert!(!alignment.contains("Projected borrowing, reference-target dynamic writes"));
    assert!(!alignment.contains("close the remaining Milestone 2 exit half"));

    let project_state = repository_file("PROJECT_STATE.md");
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
    assert!(project_state.contains("Current accepted public master is CAP-020"));
    assert!(project_state.contains("b62696272f293f9f378f8a368cc818fcb8ef1074"));
    assert!(project_state.contains("c49ff17cab7fc0e8d4f552a71499929135c16c61"));
    assert!(
        normalized_words(&project_state)
            .contains("Source-embedded fixed-shape tensor-record decode plus two-stage flat-buffer exact-`i32` CPU scoring product gate ranks first")
    );
    assert!(!project_state.contains("Current accepted public master is CAP-012"));
    assert!(!project_state.contains("Current accepted public master is CAP-013"));
    assert!(!project_state.contains("Current accepted public master is CAP-014"));
    assert!(!project_state.contains("`CAP-015-READINESS`"));
    assert!(
        !project_state
            .contains("next ranked product target is an explicitly profiled exact fixed-width")
    );

    let matrix = repository_file("SPEC_IMPLEMENTATION_MATRIX.md");
    assert!(matrix.contains("Accepted CAP-009 adds an explicitly selected `stable-scalar-v0`"));
    assert!(matrix.contains("Selected `stable-scalar-v0` profile (accepted `CAP-009`)"));
    assert!(matrix.contains("Accepted CAP-010 adds one bounded partial row"));
    assert!(matrix.contains(
        "Required-only recursive-CopyData trait-bound static dispatch (accepted `CAP-010`)"
    ));
    assert!(matrix.contains("| STABLE |"));
    assert!(matrix.contains("Latest accepted public master is protected CAP-020 product merge"));
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
    let mut in_classified_table = false;
    for line in matrix.lines().map(table_line) {
        if !line.starts_with('|') {
            in_classified_table = false;
            continue;
        }
        let cells = table_cells(line).expect("pipe table row");
        if cells
            .last()
            .is_some_and(|classification| classification.eq_ignore_ascii_case("class"))
        {
            in_classified_table = true;
            continue;
        }
        if !in_classified_table
            || cells
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
            let lower = line.to_ascii_lowercase();
            lower.starts_with('|')
                && (lower.contains("exact-i32-array-v0")
                    || lower.contains("cap-018")
                    || lower.contains("cap-019"))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        exact_profile_rows,
        [
            "| Selected CPU-only `exact-i32-array-v0` profile (created by accepted `CAP-014`; widened by accepted `CAP-018` and accepted `CAP-019`) | Y | Y | Y | Y | Y | — | Y | Y | Y | Y | Y | Y | Y | END_TO_END |"
        ],
        "CAP-014/CAP-018/CAP-019 must classify in exactly one widened profile row"
    );
    assert!(
        !language_features.to_ascii_lowercase().contains("cap-020"),
        "CAP-020 must not create a language-feature or profile row"
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
        [
            "| Selected CPU-only `exact-i32-array-v0` profile (created by accepted `CAP-014`; widened by accepted `CAP-018` and accepted `CAP-019`) | Y | Y | Y | Y | Y | — | Y | Y | Y | Y | Y | Y | Y | END_TO_END |"
        ],
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

    let roadmap = repository_file("Roadmap.md");
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
    assert!(
        normalized_words(&roadmap)
            .contains("Source-embedded fixed-shape tensor-record decode plus two-stage flat-buffer exact-`i32` CPU scoring product gate ranks first")
    );
    assert!(roadmap.contains("The milestone exit is not met"));
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
    let ranking_heading = "### Post-CAP-020 ranking";
    assert_eq!(
        roadmap.matches(ranking_heading).count(),
        1,
        "Roadmap.md must contain one post-CAP-020 ranking section"
    );
    let ranking_tail = roadmap
        .split_once(ranking_heading)
        .expect("unique post-CAP-020 ranking heading")
        .1;
    let ranking_section = ranking_tail
        .split_once("\n## ")
        .map_or(ranking_tail, |(section, _)| section);
    let normalized_ranking_section = normalized_words(ranking_section);
    for clause in ranking_section.split(['.', ';', '!', '?']) {
        let words = semantic_words(clause);
        let has = |word: &str| words.iter().any(|candidate| candidate == word);
        let implementation_claim = words.iter().any(|word| {
            matches!(
                word.as_str(),
                "implement" | "implements" | "implemented" | "implementation"
            )
        });
        let explicitly_deferred =
            contains_semantic_phrase(&words, &["stop", "rank", "2", "before", "implementation"])
                || contains_semantic_phrase(
                    &words,
                    &["stop", "rank", "3", "before", "implementation"],
                )
                || contains_semantic_phrase(&words, &["not", "implementation"])
                || contains_semantic_phrase(&words, &["not", "an", "implementation"])
                || contains_semantic_phrase(&words, &["no", "implementation"])
                || contains_semantic_phrase(
                    &words,
                    &["not", "authorized", "for", "implementation"],
                )
                || contains_semantic_phrase(&words, &["not", "approved", "for", "implementation"])
                || (contains_semantic_phrase(&words, &["rank", "2", "readiness"])
                    && (contains_semantic_phrase(&words, &["implementation", "contract"])
                        || contains_semantic_phrase(&words, &["mandatory", "stop"]))
                    && !has("proceed")
                    && !has("approved")
                    && !has("authorized"))
                || contains_semantic_phrase(
                    &words,
                    &["would", "defer", "rank", "2", "implementation"],
                )
                || contains_semantic_phrase(
                    &words,
                    &[
                        "would",
                        "restore",
                        "recursive",
                        "arrays",
                        "to",
                        "implementation",
                        "ranking",
                    ],
                )
                || (has("readiness") && has("only") && !has("proceed"))
                || (has("deferred") && !contains_semantic_phrase(&words, &["not", "deferred"]))
                || (has("later") && (has("permit") || has("ranking")));
        let positive_progress = has("proceed")
            || has("proceeds")
            || has("approved")
            || has("authorized")
            || has("cleared")
            || contains_semantic_phrase(&words, &["will", "implement"])
            || contains_semantic_phrase(&words, &["may", "implement"]);
        for rank in ["2", "3"] {
            let names_rank = words
                .windows(2)
                .any(|pair| pair[0] == "rank" && pair[1] == rank);
            assert!(
                !(names_rank
                    && (implementation_claim || positive_progress)
                    && !explicitly_deferred),
                "Roadmap.md promotes readiness-only rank {rank} to implementation: {clause}"
            );
        }
        let names_runtime = has("runtime") && has("acquisition");
        let names_recursive = has("recursive") && (has("array") || has("arrays"));
        assert!(
            !((names_runtime || names_recursive)
                && (implementation_claim || positive_progress)
                && !explicitly_deferred),
            "Roadmap.md promotes a readiness-only successor to implementation: {clause}"
        );
    }
    let mut previous_position = None;
    for contract in POST_CAP020_DECISION_CONTRACTS {
        assert_eq!(
            normalized_roadmap.matches(contract).count(),
            1,
            "Roadmap.md must state each post-CAP-020 decision contract exactly once: {contract}"
        );
        let position = normalized_ranking_section.find(contract).unwrap_or_else(|| {
            panic!("Roadmap.md detaches a decision contract from the post-CAP-020 section: {contract}")
        });
        if let Some(previous) = previous_position {
            assert!(
                position > previous,
                "Roadmap.md reorders the post-CAP-020 decision contracts"
            );
        }
        previous_position = Some(position);
    }

    let conformance = repository_file("CONFORMANCE_PLAN.md");

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
    assert!(
        conformance
            .contains("Source-embedded fixed-shape tensor-record decode plus two-stage flat-buffer exact-`i32` CPU scoring product gate ranks first")
    );
    assert!(conformance.contains("This selected lane is `END_TO_END`"));
    assert!(conformance.contains("`stable-scalar-v0` remains the only `STABLE` profile"));
    assert!(conformance.contains("b62696272f293f9f378f8a368cc818fcb8ef1074"));
    assert!(conformance.contains("c49ff17cab7fc0e8d4f552a71499929135c16c61"));
    assert!(!conformance.contains("`CAP-015-READINESS`"));
    for (document_name, document) in [
        ("README.md", readme.as_str()),
        ("CURRENT_CAPABILITY_AUDIT.md", audit.as_str()),
        ("FRAMEWORK_ALIGNMENT.md", alignment.as_str()),
        ("PROJECT_STATE.md", project_state.as_str()),
        ("SPEC_IMPLEMENTATION_MATRIX.md", matrix.as_str()),
        ("Roadmap.md", roadmap.as_str()),
        ("CONFORMANCE_PLAN.md", conformance.as_str()),
    ] {
        assert_cap014_acceptance_evidence(document_name, document);
        assert_cap015_acceptance_evidence(document_name, document);
        assert_cap018_acceptance_evidence(document_name, document);
        assert_cap019_acceptance_evidence(document_name, document);
        assert_cap020_acceptance_evidence(document_name, document);
        assert_cap020_boundaries(document_name, document);
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
    for (document_name, document) in [
        ("README.md", readme.as_str()),
        ("CURRENT_CAPABILITY_AUDIT.md", audit.as_str()),
        ("FRAMEWORK_ALIGNMENT.md", alignment.as_str()),
        ("PROJECT_STATE.md", project_state.as_str()),
        ("SPEC_IMPLEMENTATION_MATRIX.md", matrix.as_str()),
        ("Roadmap.md", roadmap.as_str()),
        ("CONFORMANCE_PLAN.md", conformance.as_str()),
    ] {
        assert_post_cap020_successor_order(document_name, document);
    }
    for (document_name, document) in [
        ("README.md", readme.as_str()),
        ("CURRENT_CAPABILITY_AUDIT.md", audit.as_str()),
        ("FRAMEWORK_ALIGNMENT.md", alignment.as_str()),
        ("PROJECT_STATE.md", project_state.as_str()),
        ("SPEC_IMPLEMENTATION_MATRIX.md", matrix.as_str()),
        ("Roadmap.md", roadmap.as_str()),
        ("CONFORMANCE_PLAN.md", conformance.as_str()),
    ] {
        assert_post_cap020_ranking_table(document_name, document);
    }
    let cap020_matrix_rows = matrix
        .lines()
        .map(table_line)
        .filter(|line| line.starts_with('|') && line.to_ascii_lowercase().contains("cap-020"))
        .collect::<Vec<_>>();
    assert_eq!(
        cap020_matrix_rows,
        [CAP020_CPU_MATRIX_ROW],
        "CAP-020 may appear in exactly one matrix row, as evidence in the existing CPU platform row"
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
        [CAP020_CPU_MATRIX_ROW],
        "matrix must preserve exactly one CPU row with CAP-020 product evidence"
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
    assert!(project_state.contains("Current accepted public master is CAP-020"));
    assert!(!project_state.contains("`CAP-015-READINESS`"));
    assert!(!project_state.contains("exact next action is this bounded"));
}
