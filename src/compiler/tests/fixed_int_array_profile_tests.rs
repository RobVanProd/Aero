use compiler::{
    CompilerOptions, IrGenerator, LanguageProfile, LogicalType, SemanticAnalyzer, check_file,
    check_program, compile_file, compile_program, parse_with_locations,
    try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

const FIXED_INT_ARRAY_PROGRAM: &str =
    include_str!("../../../examples/fixed_int_array_v0/main.aero");
const FIXED_INT_ARRAY_WRAPPING_EDGES: &str =
    include_str!("../../../examples/fixed_int_array_v0/wrapping_edges.aero");
const NEGATIVE_RUNTIME_INDEX: &str =
    include_str!("../../../examples/fixed_int_array_v0/runtime_fail/negative_index.aero");
const EQUAL_TO_COUNT_RUNTIME_INDEX: &str =
    include_str!("../../../examples/fixed_int_array_v0/runtime_fail/equal_to_count_index.aero");
const NEGATIVE_RUNTIME_WRITE_INDEX: &str =
    include_str!("../../../examples/fixed_int_array_v0/runtime_fail/negative_write_index.aero");
const EQUAL_TO_COUNT_RUNTIME_WRITE_INDEX: &str = include_str!(
    "../../../examples/fixed_int_array_v0/runtime_fail/equal_to_count_write_index.aero"
);
const FLAT_MATVEC_PRODUCT: &str =
    include_str!("../../../examples/fixed_int_array_v0/flat_matvec.aero");
const TENSOR_RECORD_SCORING: &str =
    include_str!("../../../examples/fixed_int_array_v0/tensor_record_scoring.aero");
const EXPECTED_FLAT_MATVEC_PRODUCT: &str = r#"fn matvec_2x3(matrix: [int; 6], vector: [int; 3]) -> [i32; 2] {
    let mut output: [i32; 2] = [0, 0];
    let mut row: int = 0;
    while row < 2 {
        let mut column: int = 0;
        let mut accumulator: int = 0;
        while column < 3 {
            accumulator = accumulator + matrix[row * 3 + column] * vector[column];
            column = column + 1;
        }
        output[row] = accumulator;
        row = row + 1;
    }
    return output;
}

fn main() -> int {
    let ordinary_matrix: [int; 6] = [1, 2, 3, 4, 5, 6];
    let ordinary_vector: [int; 3] = [7, 8, 9];
    let wrapping_matrix: [int; 6] = [2147483647, 0, 0, -2147483648, -1, 2];
    let wrapping_vector: [int; 3] = [2, 1, 3];

    let ordinary_result: [int; 2] = matvec_2x3(ordinary_matrix, ordinary_vector);
    let wrapping_result: [int; 2] = matvec_2x3(wrapping_matrix, wrapping_vector);

    if ordinary_matrix[0] == 1 && ordinary_matrix[1] == 2
        && ordinary_matrix[2] == 3 && ordinary_matrix[3] == 4
        && ordinary_matrix[4] == 5 && ordinary_matrix[5] == 6
        && ordinary_vector[0] == 7 && ordinary_vector[1] == 8
        && ordinary_vector[2] == 9 {
        if wrapping_matrix[0] == 2147483647 && wrapping_matrix[1] == 0
            && wrapping_matrix[2] == 0 && wrapping_matrix[3] == -2147483647 - 1
            && wrapping_matrix[4] == -1 && wrapping_matrix[5] == 2
            && wrapping_vector[0] == 2 && wrapping_vector[1] == 1
            && wrapping_vector[2] == 3 {
            if ordinary_result[0] == 50 && ordinary_result[1] == 122
                && wrapping_result[0] == -2 && wrapping_result[1] == 5 {
                return 91;
            }
        }
    }
    return 1;
}
"#;

const EXPECTED_TENSOR_RECORD_SCORING: &str = r#"fn matvec_2x3(matrix: [int; 6], vector: [int; 3]) -> [i32; 2] {
    let mut output: [i32; 2] = [0, 0];
    let mut row: int = 0;
    while row < 2 {
        let mut column: int = 0;
        let mut accumulator: int = 0;
        while column < 3 {
            accumulator = accumulator + matrix[row * 3 + column] * vector[column];
            column = column + 1;
        }
        output[row] = accumulator;
        row = row + 1;
    }
    return output;
}

fn affine_2_to_1(values: [int; 2], weights: [int; 2], bias: int) -> int {
    let mut score: int = bias;
    let mut lane: int = 0;
    while lane < 2 {
        score = score + values[lane] * weights[lane];
        lane = lane + 1;
    }
    return score;
}

fn decode_and_score(record: [int; 17]) -> [i32; 6] {
    let mut result: [i32; 6] = [0, 0, 0, 0, 0, 0];
    let mut header: [i32; 3] = [0, 0, 0];
    let mut header_index: int = 0;
    while header_index < 3 {
        header[header_index] = record[header_index];
        header_index = header_index + 1;
    }

    if header[0] == 2 && header[1] == 3 && header[2] == 1 {
        let mut input: [i32; 3] = [0, 0, 0];
        let mut input_index: int = 0;
        while input_index < 3 {
            input[input_index] = record[3 + input_index];
            input_index = input_index + 1;
        }

        let mut first_weights: [i32; 6] = [0, 0, 0, 0, 0, 0];
        let mut weight_index: int = 0;
        while weight_index < 6 {
            first_weights[weight_index] = record[6 + weight_index];
            weight_index = weight_index + 1;
        }

        let mut first_bias: [i32; 2] = [0, 0];
        let mut bias_index: int = 0;
        while bias_index < 2 {
            first_bias[bias_index] = record[12 + bias_index];
            bias_index = bias_index + 1;
        }

        let mut score_weights: [i32; 2] = [0, 0];
        let mut score_weight_index: int = 0;
        while score_weight_index < 2 {
            score_weights[score_weight_index] = record[14 + score_weight_index];
            score_weight_index = score_weight_index + 1;
        }

        let mut score_bias: [i32; 1] = [0];
        let mut score_bias_index: int = 0;
        while score_bias_index < 1 {
            score_bias[score_bias_index] = record[16 + score_bias_index];
            score_bias_index = score_bias_index + 1;
        }

        let raw: [i32; 2] = matvec_2x3(first_weights, input);
        let mut hidden: [i32; 2] = [0, 0];
        let mut hidden_index: int = 0;
        while hidden_index < 2 {
            hidden[hidden_index] = raw[hidden_index] + first_bias[hidden_index];
            hidden_index = hidden_index + 1;
        }

        let score: int = affine_2_to_1(hidden, score_weights, score_bias[0]);
        result[0] = 1;
        result[1] = raw[0];
        result[2] = raw[1];
        result[3] = hidden[0];
        result[4] = hidden[1];
        result[5] = score;
    }
    return result;
}

fn records_equal(left: [int; 17], right: [int; 17]) -> int {
    let mut equal: int = 1;
    let mut index: int = 0;
    while index < 17 {
        if left[index] != right[index] {
            equal = 0;
        }
        index = index + 1;
    }
    return equal;
}

fn main() -> int {
    let ordinary_record: [int; 17] =
        [2, 3, 1, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
    let wrapping_record: [int; 17] =
        [2, 3, 1, 2, -3, 5, 2147483647, 4, -2, -2147483648, -1, 3,
            2147483647, 2147483647, 2147483647, -1, 13];
    let malformed_record: [int; 17] =
        [2, 4, 1, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];

    let ordinary_result: [i32; 6] = decode_and_score(ordinary_record);
    let wrapping_result: [i32; 6] = decode_and_score(wrapping_record);
    let malformed_result: [i32; 6] = decode_and_score(malformed_record);
    let ordinary_preserved: int = records_equal(
        ordinary_record,
        [2, 3, 1, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17]
    );
    let wrapping_preserved: int = records_equal(
        wrapping_record,
        [2, 3, 1, 2, -3, 5, 2147483647, 4, -2, -2147483648, -1, 3,
            2147483647, 2147483647, 2147483647, -1, 13]
    );

    if ordinary_preserved == 1 && wrapping_preserved == 1 {
        if ordinary_result[0] == 1
            && ordinary_result[1] == 122 && ordinary_result[2] == 167
            && ordinary_result[3] == 135 && ordinary_result[4] == 181
            && ordinary_result[5] == 4938 {
            if wrapping_result[0] == 1
                && wrapping_result[1] == -24 && wrapping_result[2] == 18
                && wrapping_result[3] == 2147483623
                && wrapping_result[4] == -2147483631
                && wrapping_result[5] == -2147483627 {
                if malformed_result[0] == 0 && malformed_result[1] == 0
                    && malformed_result[2] == 0 && malformed_result[3] == 0
                    && malformed_result[4] == 0 && malformed_result[5] == 0 {
                    return 91;
                }
            }
        }
    }
    return 1;
}
"#;

const IMMUTABLE_ARRAY_VALUE_COMPOSITION: &str = r#"
fn transform(values: [int; 3]) -> [i32; 3] {
    return [values[0] * 2, values[1] + 3, values[2] - 1];
}

fn identity(values: [i32; 3]) -> [int; 3] {
    return values;
}

fn return_call(values: [int; 3]) -> [i32; 3] {
    return identity(values);
}

fn score(values: [int; 3]) -> int {
    return values[0] + values[1] + values[2];
}

fn main() -> int {
    let source = [2, 3, 4];
    let computed = transform(source);
    let annotated: [i32; 3] = return_call(computed);
    let alias: [int; 3] = annotated;
    let copied = alias;
    let nested_score: int = score(identity(transform(source)));
    let literal_score: int = score([7, 8, 9]);
    let call_index: int = transform(source)[0];
    let literal_index: int = [10, 11, 12][1];
    if source[0] == 2 && copied[1] == 6 && annotated[2] == 3
        && nested_score == 13 && literal_score == 24
        && call_index == 4 && literal_index == 11 {
        return 91;
    }
    return 1;
}
"#;

const MUTABLE_ARRAY_RESULT_PRODUCTION: &str = r#"
fn seed() -> [int; 4] {
    return [3, 5, 7, 9];
}

fn from_literal() -> [i32; 2] {
    let mut output: [int; 2] = [10, 20];
    output[0] = output[0] + 1;
    return output;
}

fn from_identifier(source: [int; 4]) -> [i32; 4] {
    let mut output: [i32; 4] = source;
    let mut index: int = 0;
    let mut delta: int = 0;
    while index < 4 {
        output[index] = source[index] * 2 + delta;
        index = index + 1;
        delta = delta + 1;
    }
    return output;
}

fn from_call() -> [int; 4] {
    let mut output = seed();
    output[3] = output[3] + 1;
    return output;
}

fn score(values: [int; 4]) -> int {
    return values[0] + values[1] + values[2] + values[3];
}

fn main() -> int {
    let source: [i32; 4] = seed();
    let looped: [int; 4] = from_identifier(source);
    let literal = from_literal();
    let called = from_call();
    let consumed: int = score(looped);
    if source[0] == 3 && source[3] == 9
        && looped[0] == 6 && looped[1] == 11
        && looped[2] == 16 && looped[3] == 21
        && literal[0] == 11 && literal[1] == 20
        && called[3] == 10 && consumed == 54 {
        return 91;
    }
    return 1;
}
"#;

fn exact_options() -> CompilerOptions {
    CompilerOptions {
        language_profile: LanguageProfile::ExactI32ArrayV0,
        ..CompilerOptions::default()
    }
}

fn reference_kernel() -> i32 {
    let source: [i32; 8] = [127, 1_073_741_824, -128, 64, -64, 7, -3, 11];
    let mut transformed = source;
    for lane in &mut transformed {
        *lane = lane.wrapping_add(1);
    }
    let right: [i32; 8] = [8, 2, -7, 6, -5, 4, -3, 2];
    let result = transformed
        .into_iter()
        .zip(right)
        .fold(2_147_483_001_i32, |accumulator, (left, right)| {
            accumulator.wrapping_add(left.wrapping_mul(right))
        });
    assert_eq!(
        source,
        [127, 1_073_741_824, -128, 64, -64, 7, -3, 11],
        "Copy-array source must remain readable in every lane"
    );
    assert_eq!(
        transformed,
        [128, 1_073_741_825, -127, 65, -63, 8, -2, 12],
        "returned mutable transform must update every lane"
    );
    assert_eq!(result, 2035, "mutable array-result kernel oracle drifted");
    result
}

fn reference_flat_matvec(matrix: [i32; 6], vector: [i32; 3]) -> [i32; 2] {
    let mut result = [0_i32; 2];
    for row in 0..2 {
        let mut accumulator = 0_i32;
        for column in 0..3 {
            let linear_index = row * 3 + column;
            accumulator =
                accumulator.wrapping_add(matrix[linear_index].wrapping_mul(vector[column]));
        }
        result[row] = accumulator;
    }
    result
}

const ORDINARY_TENSOR_RECORD: [i32; 17] =
    [2, 3, 1, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
const WRAPPING_TENSOR_RECORD: [i32; 17] = [
    2,
    3,
    1,
    2,
    -3,
    5,
    i32::MAX,
    4,
    -2,
    i32::MIN,
    -1,
    3,
    i32::MAX,
    i32::MAX,
    i32::MAX,
    -1,
    13,
];
const MALFORMED_TENSOR_RECORD: [i32; 17] =
    [2, 4, 1, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TensorRecordOracle {
    first_products: [i32; 6],
    raw: [i32; 2],
    hidden: [i32; 2],
    score_products: [i32; 2],
    result: [i32; 6],
}

fn reference_tensor_record(record: [i32; 17]) -> TensorRecordOracle {
    if [record[0], record[1], record[2]] != [2, 3, 1] {
        return TensorRecordOracle {
            first_products: [0; 6],
            raw: [0; 2],
            hidden: [0; 2],
            score_products: [0; 2],
            result: [0; 6],
        };
    }

    let input = [record[3], record[4], record[5]];
    let first_weights = [
        record[6], record[7], record[8], record[9], record[10], record[11],
    ];
    let first_bias = [record[12], record[13]];
    let score_weights = [record[14], record[15]];
    let score_bias = record[16];
    let first_products = [
        first_weights[0].wrapping_mul(input[0]),
        first_weights[1].wrapping_mul(input[1]),
        first_weights[2].wrapping_mul(input[2]),
        first_weights[3].wrapping_mul(input[0]),
        first_weights[4].wrapping_mul(input[1]),
        first_weights[5].wrapping_mul(input[2]),
    ];
    let raw = [
        0_i32
            .wrapping_add(first_products[0])
            .wrapping_add(first_products[1])
            .wrapping_add(first_products[2]),
        0_i32
            .wrapping_add(first_products[3])
            .wrapping_add(first_products[4])
            .wrapping_add(first_products[5]),
    ];
    let hidden = [
        raw[0].wrapping_add(first_bias[0]),
        raw[1].wrapping_add(first_bias[1]),
    ];
    let score_products = [
        hidden[0].wrapping_mul(score_weights[0]),
        hidden[1].wrapping_mul(score_weights[1]),
    ];
    let score = score_bias
        .wrapping_add(score_products[0])
        .wrapping_add(score_products[1]);

    TensorRecordOracle {
        first_products,
        raw,
        hidden,
        score_products,
        result: [1, raw[0], raw[1], hidden[0], hidden[1], score],
    }
}

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new(test_name: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after Unix epoch")
            .as_nanos();
        let sequence = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "aero-fixed-int-array-profile-{test_name}-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create fresh fixed-int-array profile workspace");
        Self { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let temp_dir = std::env::temp_dir();
        let expected_name = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("aero-fixed-int-array-profile-"));
        if self.root.starts_with(temp_dir) && expected_name {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn write_program(workspace: &TestWorkspace) -> PathBuf {
    let source = workspace.path("main.aero");
    fs::write(&source, FIXED_INT_ARRAY_PROGRAM).expect("write fixed-int-array source");
    source
}

fn run_cli(workspace: &TestWorkspace, args: &[&Path]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_aero"));
    command.current_dir(&workspace.root);
    for argument in args {
        command.arg(argument);
    }
    command
        .output()
        .expect("run Aero fixed-int-array profile route")
}

fn combined_output(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn occurrences(haystack: &str, needle: &str) -> usize {
    haystack.match_indices(needle).count()
}

fn llvm_function_body<'a>(llvm: &'a str, signature: &str) -> &'a str {
    assert_eq!(
        occurrences(llvm, signature),
        1,
        "LLVM must contain exactly one `{signature}`"
    );
    let start = llvm
        .find(signature)
        .expect("unique LLVM function signature");
    let remaining = &llvm[start..];
    let end = remaining
        .find("\n}\n")
        .map(|offset| offset + 3)
        .expect("LLVM function terminator");
    &remaining[..end]
}

fn ssa_definition(line: &str) -> &str {
    let value = line
        .trim()
        .split_once(" = ")
        .map(|(value, _)| value)
        .expect("SSA definition line");
    let suffix = value
        .strip_prefix("%reg")
        .or_else(|| value.strip_prefix("%ptr"))
        .expect("canonical Aero SSA register or pointer");
    assert!(
        !suffix.is_empty() && suffix.chars().all(|character| character.is_ascii_digit()),
        "Aero SSA values must use a numeric register or pointer suffix: {value}"
    );
    value
}

fn ssa_rhs<'a>(function: &'a str, value: &str) -> &'a str {
    let prefix = format!("{value} = ");
    let definitions = function
        .lines()
        .filter_map(|line| line.trim().strip_prefix(&prefix))
        .collect::<Vec<_>>();
    assert_eq!(
        definitions.len(),
        1,
        "SSA value `{value}` must have exactly one definition:\n{function}"
    );
    definitions[0]
}

fn loaded_i32_pointer(function: &str, value: &str) -> String {
    let pointer = ssa_rhs(function, value)
        .strip_prefix("load i32, i32* ")
        .and_then(|rhs| rhs.strip_suffix(", align 4"))
        .unwrap_or_else(|| panic!("`{value}` must be an exact i32 load:\n{function}"));
    let suffix = pointer
        .strip_prefix("%ptr")
        .expect("loaded i32 pointer must use Aero pointer SSA");
    assert!(
        !suffix.is_empty() && suffix.chars().all(|character| character.is_ascii_digit()),
        "loaded i32 pointer must have a numeric suffix: {pointer}"
    );
    pointer.to_string()
}

fn assert_identity_linked_guard_consumer(
    function: &str,
    index: &str,
    upper_bound: i32,
    aggregate: &str,
    consumer: &str,
) {
    let lines = function.lines().collect::<Vec<_>>();
    let lower_suffix = format!(" = icmp sge i32 {index}, 0");
    let lower_positions = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim().ends_with(&lower_suffix))
        .map(|(position, _)| position)
        .collect::<Vec<_>>();
    assert_eq!(
        lower_positions.len(),
        1,
        "index `{index}` must enter exactly one lower guard:\n{function}"
    );
    let lower_position = lower_positions[0];
    let lower = ssa_definition(lines[lower_position]);
    let upper_line = lines
        .get(lower_position + 1)
        .expect("upper guard after lower guard");
    let expected_upper = format!("icmp slt i32 {index}, {upper_bound}");
    assert_eq!(
        upper_line.trim().split_once(" = ").map(|(_, rhs)| rhs),
        Some(expected_upper.as_str()),
        "upper guard must reuse the exact index"
    );
    let upper = ssa_definition(upper_line);
    let conjunction_line = lines
        .get(lower_position + 2)
        .expect("guard conjunction after upper guard");
    let expected_conjunction = format!("and i1 {lower}, {upper}");
    assert_eq!(
        conjunction_line
            .trim()
            .split_once(" = ")
            .map(|(_, rhs)| rhs),
        Some(expected_conjunction.as_str()),
        "guard conjunction must reuse lower and upper predicates"
    );
    let conjunction = ssa_definition(conjunction_line);
    let branch = lines
        .get(lower_position + 3)
        .expect("guard branch after conjunction")
        .trim();
    let branch_prefix = format!("br i1 {conjunction}, label %aero.bounds.safe.");
    assert!(
        branch.starts_with(&branch_prefix),
        "guard branch must reuse the conjunction: {branch}"
    );
    let labels = branch
        .strip_prefix(&format!("br i1 {conjunction}, label %"))
        .expect("guard branch predicate")
        .split_once(", label %")
        .expect("guard branch labels");
    let safe_label = labels.0;
    let trap_label = labels.1;
    let place = safe_label
        .strip_prefix("aero.bounds.safe.")
        .expect("safe label prefix");
    assert_eq!(
        trap_label,
        format!("aero.bounds.trap.{place}"),
        "safe and trap labels must share one projected place"
    );
    assert_eq!(lines[lower_position + 4].trim(), format!("{trap_label}:"));
    assert_eq!(lines[lower_position + 5].trim(), "call void @llvm.trap()");
    assert_eq!(lines[lower_position + 6].trim(), "unreachable");
    assert_eq!(lines[lower_position + 7].trim(), format!("{safe_label}:"));
    let extension_line = lines[lower_position + 8];
    let expected_extension = format!("sext i32 {index} to i64");
    assert_eq!(
        extension_line.trim().split_once(" = ").map(|(_, rhs)| rhs),
        Some(expected_extension.as_str()),
        "sign extension must reuse the guarded index"
    );
    let extension = ssa_definition(extension_line);
    let gep_line = lines[lower_position + 9];
    let pointer = ssa_definition(gep_line);
    assert!(
        gep_line.contains(&format!(
            "getelementptr inbounds {aggregate}, {aggregate}* %ptr"
        )) && gep_line.ends_with(&format!(", i64 0, i64 {extension}")),
        "guarded address must reuse the sign-extended index: {gep_line}"
    );
    let consumer_target = format!("i32* {pointer}, align 4");
    let consumer_lines = lines[lower_position + 10..]
        .iter()
        .take_while(|line| line.trim() != "}")
        .filter(|line| {
            line.contains(&consumer_target)
                && if consumer == "load i32" {
                    line.contains(" = load i32, ")
                } else {
                    line.trim().starts_with(consumer)
                }
        })
        .count();
    assert_eq!(
        consumer_lines, 1,
        "guarded pointer `{pointer}` must feed exactly one `{consumer}` consumer"
    );
}

fn ssa_value_for_rhs(function: &str, rhs: &str) -> String {
    let suffix = format!(" = {rhs}");
    let definitions = function
        .lines()
        .filter(|line| line.trim().ends_with(&suffix))
        .collect::<Vec<_>>();
    assert_eq!(
        definitions.len(),
        1,
        "expected exactly one SSA definition for `{rhs}`:\n{function}"
    );
    ssa_definition(definitions[0]).to_string()
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DynamicI32Access {
    aggregate: String,
    base: String,
    pointer: String,
    index: String,
    consumer: String,
    value: String,
    line: usize,
    consumer_line: usize,
}

fn static_array_lane_pointer(function: &str, aggregate: &str, base: &str, lane: usize) -> String {
    ssa_value_for_rhs(
        function,
        &format!("getelementptr inbounds {aggregate}, {aggregate}* {base}, i64 0, i64 {lane}"),
    )
}

fn dynamic_i32_accesses(function: &str, aggregate: &str) -> Vec<DynamicI32Access> {
    let address = format!("getelementptr inbounds {aggregate}, {aggregate}* ");
    function
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains(&address) && line.contains(", i64 %reg"))
        .map(|(line, gep)| {
            let pointer = ssa_definition(gep).to_string();
            let gep_rhs = gep
                .trim()
                .split_once(" = ")
                .map(|(_, rhs)| rhs)
                .expect("dynamic aggregate GEP RHS");
            let address_rhs = gep_rhs
                .strip_prefix(&address)
                .unwrap_or_else(|| panic!("dynamic aggregate GEP changed shape: {gep}"));
            let (base, _) = address_rhs
                .split_once(", i64 0, i64 ")
                .expect("dynamic aggregate GEP base and extension");
            assert!(
                base.strip_prefix("%ptr")
                    .is_some_and(|suffix| !suffix.is_empty()
                        && suffix.chars().all(|character| character.is_ascii_digit())),
                "dynamic aggregate GEP must use one exact local base: {gep}"
            );
            let extension = gep
                .rsplit_once(", i64 ")
                .map(|(_, extension)| extension.trim())
                .expect("dynamic aggregate GEP extension");
            let index = ssa_rhs(function, extension)
                .strip_prefix("sext i32 ")
                .and_then(|rhs| rhs.strip_suffix(" to i64"))
                .unwrap_or_else(|| {
                    panic!("dynamic aggregate GEP omitted exact sign extension: {gep}")
                })
                .to_string();
            let load_target = format!("load i32, i32* {pointer}, align 4");
            let store_target = format!(", i32* {pointer}, align 4");
            let loads = function
                .lines()
                .enumerate()
                .filter(|(_, line)| line.trim().contains(&load_target))
                .collect::<Vec<_>>();
            let stores = function
                .lines()
                .enumerate()
                .filter(|(_, line)| {
                    let line = line.trim();
                    line.starts_with("store i32 ") && line.ends_with(&store_target)
                })
                .collect::<Vec<_>>();
            assert_eq!(
                loads.len() + stores.len(),
                1,
                "dynamic pointer `{pointer}` must have exactly one scalar consumer:\n{function}"
            );
            if let Some((consumer_line, load)) = loads.first() {
                DynamicI32Access {
                    aggregate: aggregate.to_string(),
                    base: base.to_string(),
                    pointer,
                    index,
                    consumer: "load i32".to_string(),
                    value: ssa_definition(load).to_string(),
                    line,
                    consumer_line: *consumer_line,
                }
            } else {
                let value = stores[0]
                    .1
                    .trim()
                    .strip_prefix("store i32 ")
                    .and_then(|line| line.strip_suffix(&store_target))
                    .expect("dynamic i32 store value")
                    .to_string();
                DynamicI32Access {
                    aggregate: aggregate.to_string(),
                    base: base.to_string(),
                    pointer,
                    index,
                    consumer: "store i32".to_string(),
                    value,
                    line,
                    consumer_line: stores[0].0,
                }
            }
        })
        .collect()
}

fn assert_identity_linked_dynamic_accesses(
    function: &str,
    aggregate: &str,
    upper_bound: i32,
    expected_consumers: &[&str],
) -> Vec<String> {
    let accesses = dynamic_i32_accesses(function, aggregate);
    assert_eq!(
        accesses
            .iter()
            .map(|access| access.consumer.as_str())
            .collect::<Vec<_>>(),
        expected_consumers,
        "dynamic {aggregate} consumer order drifted:\n{function}"
    );
    for access in &accesses {
        assert_identity_linked_guard_consumer(
            function,
            &access.index,
            upper_bound,
            aggregate,
            &access.consumer,
        );
    }
    accesses.into_iter().map(|access| access.value).collect()
}

#[derive(Clone, Debug)]
struct LlvmBlock {
    name: String,
    start: usize,
    end: usize,
    successors: Vec<String>,
}

fn llvm_blocks(function: &str) -> Vec<LlvmBlock> {
    let lines = function.lines().collect::<Vec<_>>();
    let starts = lines
        .iter()
        .enumerate()
        .filter_map(|(line, text)| {
            let label = text.trim().strip_suffix(':')?;
            (!label.is_empty()
                && !label.contains(char::is_whitespace)
                && !label.starts_with("define "))
            .then(|| (label.to_string(), line))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        starts.first().map(|(name, _)| name.as_str()),
        Some("entry"),
        "LLVM function must start with an entry block:\n{function}"
    );
    let mut blocks = Vec::new();
    for (position, (name, start)) in starts.iter().enumerate() {
        let end = starts
            .get(position + 1)
            .map(|(_, next)| *next)
            .unwrap_or(lines.len() - 1);
        let terminator = lines[*start + 1..end]
            .iter()
            .rev()
            .map(|line| line.trim())
            .find(|line| {
                line.starts_with("br ") || line.starts_with("ret ") || *line == "unreachable"
            })
            .unwrap_or_else(|| panic!("LLVM block `{name}` omitted a terminator:\n{function}"));
        let successors = if let Some(target) = terminator.strip_prefix("br label %") {
            vec![target.to_string()]
        } else if let Some(branch) = terminator.strip_prefix("br i1 ") {
            let (_, targets) = branch
                .split_once(", label %")
                .expect("conditional branch true target");
            let (true_target, false_target) = targets
                .split_once(", label %")
                .expect("conditional branch false target");
            vec![true_target.to_string(), false_target.to_string()]
        } else {
            Vec::new()
        };
        blocks.push(LlvmBlock {
            name: name.clone(),
            start: *start,
            end,
            successors,
        });
    }
    for block in &blocks {
        for successor in &block.successors {
            assert!(
                blocks.iter().any(|candidate| &candidate.name == successor),
                "block `{}` targets missing block `{successor}`:\n{function}",
                block.name
            );
        }
    }
    blocks
}

fn llvm_block_for_line<'a>(blocks: &'a [LlvmBlock], line: usize) -> &'a LlvmBlock {
    blocks
        .iter()
        .find(|block| block.start <= line && line < block.end)
        .unwrap_or_else(|| panic!("LLVM line {line} is outside parsed basic blocks"))
}

fn llvm_block<'a>(blocks: &'a [LlvmBlock], name: &str) -> &'a LlvmBlock {
    blocks
        .iter()
        .find(|block| block.name == name)
        .unwrap_or_else(|| panic!("LLVM block `{name}` is missing"))
}

fn llvm_predecessors(blocks: &[LlvmBlock], name: &str) -> Vec<String> {
    blocks
        .iter()
        .filter(|block| block.successors.iter().any(|successor| successor == name))
        .map(|block| block.name.clone())
        .collect()
}

fn llvm_reachable(blocks: &[LlvmBlock], from: &str, target: &str) -> bool {
    let mut pending = vec![from.to_string()];
    let mut visited = Vec::new();
    while let Some(name) = pending.pop() {
        if name == target {
            return true;
        }
        if visited.contains(&name) {
            continue;
        }
        visited.push(name.clone());
        pending.extend(llvm_block(blocks, &name).successors.iter().cloned());
    }
    false
}

fn llvm_dominates(blocks: &[LlvmBlock], dominator: &str, dominated: &str) -> bool {
    let entry = blocks
        .iter()
        .position(|block| block.name == "entry")
        .expect("entry block");
    let dominator = blocks
        .iter()
        .position(|block| block.name == dominator)
        .expect("dominator block");
    let dominated = blocks
        .iter()
        .position(|block| block.name == dominated)
        .expect("dominated block");
    let mut dominators = vec![vec![true; blocks.len()]; blocks.len()];
    dominators[entry].fill(false);
    dominators[entry][entry] = true;
    loop {
        let previous = dominators.clone();
        for block in 0..blocks.len() {
            if block == entry {
                continue;
            }
            let predecessors = blocks
                .iter()
                .enumerate()
                .filter(|(_, candidate)| {
                    candidate
                        .successors
                        .iter()
                        .any(|successor| successor == &blocks[block].name)
                })
                .map(|(position, _)| position)
                .collect::<Vec<_>>();
            dominators[block].fill(true);
            if predecessors.is_empty() {
                dominators[block].fill(false);
            } else {
                for candidate in 0..blocks.len() {
                    dominators[block][candidate] = predecessors
                        .iter()
                        .all(|predecessor| previous[*predecessor][candidate]);
                }
            }
            dominators[block][block] = true;
        }
        if dominators == previous {
            break;
        }
    }
    dominators[dominated][dominator]
}

fn exact_argument_array_base(function: &str, aggregate: &str, argument: &str) -> String {
    let prefix = format!("store {aggregate} %aero.arg.{argument}, {aggregate}* ");
    let suffix = ", align 8";
    let stores = function
        .lines()
        .filter_map(|line| line.trim().strip_prefix(&prefix))
        .filter_map(|line| line.strip_suffix(suffix))
        .collect::<Vec<_>>();
    assert_eq!(
        stores.len(),
        1,
        "argument `{argument}` must have one exact {aggregate} local:\n{function}"
    );
    assert!(
        stores[0]
            .strip_prefix("%ptr")
            .is_some_and(|suffix| !suffix.is_empty()
                && suffix.chars().all(|character| character.is_ascii_digit())),
        "argument `{argument}` local must use an Aero pointer SSA"
    );
    stores[0].to_string()
}

fn exact_static_i32_load(function: &str, aggregate: &str, base: &str, lane: usize) -> String {
    let rhs = format!("getelementptr inbounds {aggregate}, {aggregate}* {base}, i64 0, i64 {lane}");
    let pointer = ssa_value_for_rhs(function, &rhs);
    ssa_value_for_rhs(function, &format!("load i32, i32* {pointer}, align 4"))
}

fn assert_zero_initialized_array_local(function: &str, aggregate: &str, width: usize, base: &str) {
    let blocks = llvm_blocks(function);
    assert_eq!(
        occurrences(function, &format!("{base} = alloca {aggregate}, align 8")),
        1,
        "decoded {aggregate} destination must be one exact local `{base}`"
    );
    let copy_suffix = format!(", {aggregate}* {base}, align 8");
    let copies = function
        .lines()
        .filter(|line| {
            let line = line.trim();
            line.starts_with(&format!("store {aggregate} %reg")) && line.ends_with(&copy_suffix)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        copies.len(),
        1,
        "decoded local `{base}` must receive one fully initialized aggregate copy"
    );
    let initialized_value = copies[0]
        .trim()
        .strip_prefix(&format!("store {aggregate} "))
        .and_then(|line| line.strip_suffix(&copy_suffix))
        .expect("decoded aggregate copy value");
    let initialized_rhs = ssa_rhs(function, initialized_value);
    let initializer_base = initialized_rhs
        .strip_prefix(&format!("load {aggregate}, {aggregate}* "))
        .and_then(|rhs| rhs.strip_suffix(", align 8"))
        .unwrap_or_else(|| panic!("decoded local `{base}` omitted initialized aggregate load"));
    assert_ne!(initializer_base, base);
    assert_eq!(
        occurrences(
            function,
            &format!("{initializer_base} = alloca {aggregate}, align 8")
        ),
        1,
        "decoded local `{base}` initializer must be a unique alloca"
    );
    let copy_line = function
        .lines()
        .position(|line| line == copies[0])
        .expect("initialized aggregate copy line");
    let aggregate_load_line = function
        .lines()
        .position(|line| line.trim().starts_with(&format!("{initialized_value} = ")))
        .expect("initialized aggregate load line");
    assert!(
        aggregate_load_line < copy_line,
        "decoded local `{base}` aggregate load must precede its destination copy"
    );
    let aggregate_load_block = llvm_block_for_line(&blocks, aggregate_load_line);
    let copy_block = llvm_block_for_line(&blocks, copy_line);
    assert!(llvm_dominates(
        &blocks,
        &aggregate_load_block.name,
        &copy_block.name
    ));
    for lane in 0..width {
        let pointer = ssa_value_for_rhs(
            function,
            &format!(
                "getelementptr inbounds {aggregate}, {aggregate}* {initializer_base}, i64 0, i64 {lane}"
            ),
        );
        assert_eq!(
            occurrences(function, &format!("store i32 0, i32* {pointer}, align 4")),
            1,
            "decoded local `{base}` initializer lane {lane} must be written exactly once"
        );
        let initializer_line = function
            .lines()
            .position(|line| line.trim() == format!("store i32 0, i32* {pointer}, align 4"))
            .expect("decoded zero initializer line");
        assert!(
            initializer_line < aggregate_load_line,
            "decoded local `{base}` lane {lane} must initialize before aggregate load"
        );
        let initializer_block = llvm_block_for_line(&blocks, initializer_line);
        assert!(
            llvm_dominates(&blocks, &initializer_block.name, &aggregate_load_block.name),
            "decoded local `{base}` lane {lane} initializer must dominate aggregate load"
        );
    }
}

#[derive(Clone, Debug)]
struct CountedLoop {
    start: String,
    body: String,
    end: String,
    update: String,
}

fn assert_exact_counted_loop(
    function: &str,
    blocks: &[LlvmBlock],
    index_slot: &str,
    width: i32,
) -> CountedLoop {
    let mut loop_branches = Vec::new();
    for (line, text) in function.lines().enumerate() {
        let branch = text.trim();
        let Some(branch) = branch.strip_prefix("br i1 ") else {
            continue;
        };
        let Some((predicate, targets)) = branch.split_once(", label %") else {
            continue;
        };
        let Some((body, end)) = targets.split_once(", label %") else {
            continue;
        };
        if !body.starts_with("while_body_") || !end.starts_with("while_end_") {
            continue;
        }
        let condition = ssa_rhs(function, predicate);
        let Some(index) = condition
            .strip_prefix("icmp slt i32 ")
            .and_then(|rhs| rhs.strip_suffix(&format!(", {width}")))
        else {
            continue;
        };
        if loaded_i32_pointer(function, index) == index_slot {
            loop_branches.push((line, body.to_string(), end.to_string()));
        }
    }
    assert_eq!(
        loop_branches.len(),
        1,
        "index local `{index_slot}` must control one exact 0..{width} loop"
    );
    let (condition_line, body, end) = loop_branches.pop().expect("exact counted loop");
    let start = llvm_block_for_line(blocks, condition_line).name.clone();
    assert!(
        start.starts_with("while_start_"),
        "counted loop condition must live in a while-start block"
    );
    assert_eq!(
        llvm_block(blocks, &start).successors,
        vec![body.clone(), end.clone()],
        "counted loop condition must branch to its exact body and exit"
    );

    let store_suffix = format!(", i32* {index_slot}, align 4");
    let stores = function
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let line = line.trim();
            line.starts_with("store i32 ") && line.ends_with(&store_suffix)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        stores.len(),
        2,
        "counted loop index `{index_slot}` must have only initialization and induction writes"
    );
    let initial = stores
        .iter()
        .find(|(_, line)| line.trim() == format!("store i32 0, i32* {index_slot}, align 4"))
        .expect("counted loop exact zero initialization");
    let update = stores
        .iter()
        .find(|(_, line)| line.trim() != format!("store i32 0, i32* {index_slot}, align 4"))
        .expect("counted loop induction write");
    let update_value = update
        .1
        .trim()
        .strip_prefix("store i32 ")
        .and_then(|line| line.strip_suffix(&store_suffix))
        .expect("counted loop induction value");
    let previous_index = ssa_rhs(function, update_value)
        .strip_prefix("add i32 ")
        .and_then(|rhs| rhs.strip_suffix(", 1"))
        .unwrap_or_else(|| panic!("counted loop `{index_slot}` must increment by exact one"));
    assert_eq!(
        loaded_i32_pointer(function, previous_index),
        index_slot,
        "counted loop increment must reread its exact index local"
    );
    let initial_block = llvm_block_for_line(blocks, initial.0);
    assert_eq!(
        initial_block.successors,
        vec![start.clone()],
        "counted loop zero initialization must enter its exact condition"
    );
    let update_block = llvm_block_for_line(blocks, update.0);
    assert_eq!(
        update_block.successors,
        vec![start.clone()],
        "counted loop induction update must backedge to its exact condition"
    );
    assert!(llvm_dominates(blocks, &body, &update_block.name));
    assert!(!llvm_reachable(blocks, &end, &body));
    CountedLoop {
        start,
        body,
        end,
        update: update_block.name.clone(),
    }
}

#[derive(Clone, Debug)]
struct DecodeLoopBinding {
    name: &'static str,
    destination: DynamicI32Access,
    source: DynamicI32Access,
    counted: CountedLoop,
}

fn assert_header_static_lanes_backreference_dynamic_copy(
    function: &str,
    header: &DecodeLoopBinding,
) {
    assert_eq!(header.name, "header");
    let header_index_slot = loaded_i32_pointer(function, &header.destination.index);
    for lane in 0..3 {
        let static_pointer = ssa_value_for_rhs(
            function,
            &format!(
                "getelementptr inbounds [3 x i32], [3 x i32]* {}, i64 0, i64 {lane}",
                header.destination.base
            ),
        );
        let static_load = ssa_value_for_rhs(
            function,
            &format!("load i32, i32* {static_pointer}, align 4"),
        );
        assert!(
            function.contains(&format!("icmp eq i32 {static_load}, ")),
            "header static lane {lane} must feed an exact equality comparison"
        );
    }
    let source_raw_index = if ssa_rhs(function, &header.source.index).starts_with("load i32") {
        header.source.index.clone()
    } else {
        panic!("header record source must use its unoffset loop index")
    };
    assert_eq!(
        loaded_i32_pointer(function, &source_raw_index),
        header_index_slot,
        "header record load and header destination store must reuse one induction local"
    );
    assert_eq!(
        header.destination.value, header.source.value,
        "same record load value must feed the exact header destination store"
    );
}

fn assert_decode_loop_binding(
    function: &str,
    blocks: &[LlvmBlock],
    record_base: &str,
    name: &'static str,
    expected_aggregate: &str,
    width: i32,
    offset: i32,
    source: &DynamicI32Access,
    all_accesses: &[DynamicI32Access],
) -> DecodeLoopBinding {
    assert_eq!(source.aggregate, "[17 x i32]");
    assert_eq!(
        source.base, record_base,
        "{name} must read the record local"
    );
    assert_eq!(source.consumer, "load i32");
    assert_identity_linked_guard_consumer(function, &source.index, 17, "[17 x i32]", "load i32");
    let raw_index = if offset == 0 {
        source.index.clone()
    } else {
        ssa_rhs(function, &source.index)
            .strip_prefix(&format!("add i32 {offset}, "))
            .unwrap_or_else(|| {
                panic!("{name} record access must use exact offset {offset}: {source:?}")
            })
            .to_string()
    };
    let destinations = all_accesses
        .iter()
        .filter(|access| access.consumer == "store i32" && access.value == source.value)
        .collect::<Vec<_>>();
    assert_eq!(
        destinations.len(),
        1,
        "{name} record load must feed exactly one decoded destination store"
    );
    let destination = (*destinations[0]).clone();
    assert_eq!(
        destination.aggregate, expected_aggregate,
        "{name} must decode into its exact local aggregate type"
    );
    assert_identity_linked_guard_consumer(
        function,
        &destination.index,
        width,
        expected_aggregate,
        "store i32",
    );
    let index_slot = loaded_i32_pointer(function, &raw_index);
    assert_eq!(
        loaded_i32_pointer(function, &destination.index),
        index_slot,
        "{name} source offset and destination must reuse the same loop-index local"
    );
    assert_zero_initialized_array_local(
        function,
        expected_aggregate,
        usize::try_from(width).expect("positive decode width"),
        &destination.base,
    );
    let counted = assert_exact_counted_loop(function, blocks, &index_slot, width);
    for line in [
        source.line,
        source.consumer_line,
        destination.line,
        destination.consumer_line,
    ] {
        let block = llvm_block_for_line(blocks, line);
        assert!(
            llvm_dominates(blocks, &counted.body, &block.name),
            "{name} access in `{}` must be owned by its exact loop body",
            block.name
        );
        assert!(
            llvm_dominates(blocks, &counted.start, &block.name),
            "{name} access in `{}` must be dominated by its exact loop condition",
            block.name
        );
    }
    DecodeLoopBinding {
        name,
        destination,
        source: source.clone(),
        counted,
    }
}

fn assert_tensor_header_gate(
    function: &str,
    blocks: &[LlvmBlock],
    header: &DecodeLoopBinding,
    payload: &[DecodeLoopBinding],
) {
    assert_eq!(header.name, "header");
    assert_header_static_lanes_backreference_dynamic_copy(function, header);
    assert_eq!(
        payload.len(),
        5,
        "header gate must own all five payload decoders"
    );
    let expected_header = [2, 3, 1];
    let predicates = expected_header
        .into_iter()
        .enumerate()
        .map(|(lane, expected)| {
            let value =
                exact_static_i32_load(function, "[3 x i32]", &header.destination.base, lane);
            ssa_value_for_rhs(function, &format!("icmp eq i32 {value}, {expected}"))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        function
            .lines()
            .filter(|line| line.contains(" = icmp eq i32 "))
            .count(),
        3,
        "only exact static header lanes 0/1/2 may form equality predicates"
    );
    let first = ssa_value_for_rhs(
        function,
        &format!("and i1 {}, {}", predicates[0], predicates[1]),
    );
    let gate = ssa_value_for_rhs(function, &format!("and i1 {first}, {}", predicates[2]));
    let gate_branches = function
        .lines()
        .enumerate()
        .filter_map(|(line, text)| {
            text.trim()
                .strip_prefix(&format!("br i1 {gate}, label %"))
                .map(|targets| (line, targets))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        gate_branches.len(),
        1,
        "exact header predicate must branch once"
    );
    let (gate_line, targets) = gate_branches[0];
    let (true_target, false_target) = targets
        .split_once(", label %")
        .expect("header gate true/false labels");
    let gate_block = llvm_block_for_line(blocks, gate_line);
    assert_eq!(
        gate_block.successors,
        vec![true_target.to_string(), false_target.to_string()]
    );
    assert_eq!(
        llvm_predecessors(blocks, true_target),
        vec![gate_block.name.clone()],
        "only the validated true edge may enter payload decoding"
    );

    for binding in payload {
        for block in [
            &binding.counted.start,
            &binding.counted.body,
            &binding.counted.end,
            &llvm_block_for_line(blocks, binding.source.line).name,
            &llvm_block_for_line(blocks, binding.destination.line).name,
        ] {
            assert!(
                llvm_dominates(blocks, true_target, block),
                "validated true edge must dominate {} decoder block `{block}`",
                binding.name
            );
            assert!(
                !llvm_reachable(blocks, false_target, block),
                "false header edge must bypass {} decoder block `{block}`",
                binding.name
            );
        }
    }

    let false_block = llvm_block(blocks, false_target);
    assert_eq!(
        false_block.successors.len(),
        1,
        "invalid header edge must bypass directly to one merge"
    );
    let merge = &false_block.successors[0];
    let false_instructions = function
        .lines()
        .skip(false_block.start + 1)
        .take(false_block.end - false_block.start - 1)
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        false_instructions,
        vec![format!("br label %{merge}")],
        "invalid header edge must perform no payload work"
    );
    let merge_block = llvm_block(blocks, merge);
    assert!(
        function
            .lines()
            .skip(merge_block.start + 1)
            .take(merge_block.end - merge_block.start - 1)
            .any(|line| line.trim().starts_with("ret [6 x i32] ")),
        "post-payload merge must return the initialized result"
    );
    let merge_predecessors = llvm_predecessors(blocks, merge);
    assert_eq!(merge_predecessors.len(), 2);
    assert!(merge_predecessors.contains(&false_target.to_string()));
    let payload_predecessor = merge_predecessors
        .iter()
        .find(|predecessor| predecessor.as_str() != false_target)
        .expect("validated payload predecessor");
    assert!(llvm_dominates(blocks, true_target, payload_predecessor));
    assert!(llvm_reachable(blocks, true_target, merge));
}

fn exact_aggregate_load(function: &str, aggregate: &str, base: &str) -> String {
    let rhs = format!("load {aggregate}, {aggregate}* {base}, align 8");
    let values = function
        .lines()
        .filter(|line| line.trim().ends_with(&format!(" = {rhs}")))
        .map(ssa_definition)
        .map(str::to_string)
        .collect::<Vec<_>>();
    assert_eq!(
        values.len(),
        1,
        "expected exactly one aggregate load from `{base}`: {rhs}"
    );
    values[0].clone()
}

fn assert_result_array_contract(
    function: &str,
    blocks: &[LlvmBlock],
    gate_header: &DecodeLoopBinding,
    raw_base: &str,
    hidden_base: &str,
    score_value: &str,
) {
    let result_return = function
        .lines()
        .find_map(|line| line.trim().strip_prefix("ret [6 x i32] "))
        .expect("decode result return value");
    let result_load_rhs = ssa_rhs(function, result_return);
    let result_base = result_load_rhs
        .strip_prefix("load [6 x i32], [6 x i32]* ")
        .and_then(|rhs| rhs.strip_suffix(", align 8"))
        .expect("decode result return must be an exact aggregate load");
    assert_zero_initialized_array_local(function, "[6 x i32]", 6, result_base);
    assert_ne!(
        result_base, gate_header.destination.base,
        "header and result must own distinct locals"
    );
    let result_load_line = function
        .lines()
        .position(|line| {
            line.trim()
                == format!("{result_return} = load [6 x i32], [6 x i32]* {result_base}, align 8")
        })
        .expect("decode result aggregate load line");
    let return_line = function
        .lines()
        .position(|line| line.trim() == format!("ret [6 x i32] {result_return}"))
        .expect("decode result return line");
    assert!(result_load_line < return_line);
    assert_eq!(
        llvm_block_for_line(blocks, result_load_line).name,
        llvm_block_for_line(blocks, return_line).name,
        "merge must load and return the same result aggregate"
    );

    let gate_line = function
        .lines()
        .enumerate()
        .filter(|(_, line)| line.trim().starts_with("br i1 ") && line.contains("label %if_then_"))
        .map(|(line, _)| line)
        .find(|line| *line > gate_header.destination.consumer_line)
        .expect("decode header gate line");
    let result_copy_line = function
        .lines()
        .position(|line| {
            let line = line.trim();
            line.starts_with("store [6 x i32] %reg")
                && line.ends_with(&format!(", [6 x i32]* {result_base}, align 8"))
        })
        .expect("initialized result destination copy");
    let aggregate_result_stores = function
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            let line = line.trim();
            line.starts_with("store [6 x i32] ")
                && line.ends_with(&format!(", [6 x i32]* {result_base}, align 8"))
        })
        .map(|(line, _)| line)
        .collect::<Vec<_>>();
    assert_eq!(
        aggregate_result_stores,
        vec![result_copy_line],
        "result local must have only its one complete zero-initialized aggregate copy"
    );
    assert!(
        result_copy_line < gate_line,
        "complete zero result initialization must precede header gating"
    );
    assert!(llvm_dominates(
        blocks,
        &llvm_block_for_line(blocks, result_copy_line).name,
        &llvm_block_for_line(blocks, gate_line).name,
    ));

    let expected_values = [
        "1".to_string(),
        exact_static_i32_load(function, "[2 x i32]", raw_base, 0),
        exact_static_i32_load(function, "[2 x i32]", raw_base, 1),
        exact_static_i32_load(function, "[2 x i32]", hidden_base, 0),
        exact_static_i32_load(function, "[2 x i32]", hidden_base, 1),
        score_value.to_string(),
    ];
    let result_writes = expected_values
        .iter()
        .enumerate()
        .map(|(lane, value)| {
            let pointer_rhs = format!(
                "getelementptr inbounds [6 x i32], [6 x i32]* {result_base}, i64 0, i64 {lane}"
            );
            let pointer = ssa_value_for_rhs(function, &pointer_rhs);
            let store = format!("store i32 {value}, i32* {pointer}, align 4");
            let lines = function
                .lines()
                .enumerate()
                .filter(|(_, line)| line.trim() == store)
                .map(|(line, _)| line)
                .collect::<Vec<_>>();
            assert_eq!(
                lines.len(),
                1,
                "valid decode result lane {lane} must store its exact value identity"
            );
            lines[0]
        })
        .collect::<Vec<_>>();
    let result_gep_prefix =
        format!("getelementptr inbounds [6 x i32], [6 x i32]* {result_base}, i64 0, i64 ");
    let rooted_result_geps = function
        .lines()
        .enumerate()
        .filter_map(|(line, text)| {
            let (_, rhs) = text.trim().split_once(" = ")?;
            let lane = rhs.strip_prefix(&result_gep_prefix)?;
            Some((line, ssa_definition(text).to_string(), lane.to_string()))
        })
        .collect::<Vec<_>>();
    let mut rooted_result_stores = Vec::new();
    for (_, pointer, lane) in &rooted_result_geps {
        let lane = lane.parse::<usize>().unwrap_or_else(|_| {
            panic!("result local must use only static lane GEPs, found {lane}")
        });
        let store_suffix = format!(", i32* {pointer}, align 4");
        let stores = function
            .lines()
            .enumerate()
            .filter(|(_, line)| {
                let line = line.trim();
                line.starts_with("store i32 ") && line.ends_with(&store_suffix)
            })
            .map(|(line, _)| line)
            .collect::<Vec<_>>();
        assert_eq!(
            stores.len(),
            1,
            "result lane {lane} pointer {pointer} must have exactly one scalar store"
        );
        rooted_result_stores.push((lane, stores[0]));
    }
    rooted_result_stores.sort_unstable();
    assert_eq!(
        rooted_result_stores
            .iter()
            .map(|(lane, _)| *lane)
            .collect::<Vec<_>>(),
        (0..6).collect::<Vec<_>>(),
        "result local must expose exactly the six static valid-result lanes"
    );
    assert_eq!(
        rooted_result_stores
            .iter()
            .map(|(_, line)| *line)
            .collect::<Vec<_>>(),
        result_writes,
        "every scalar store rooted at the exact result local must be one linked valid-result write"
    );

    let gate_targets = function.lines().nth(gate_line).expect("header gate").trim();
    let (true_target, false_target) = gate_targets
        .strip_prefix("br i1 ")
        .and_then(|branch| branch.split_once(", label %"))
        .and_then(|(_, targets)| targets.split_once(", label %"))
        .expect("header gate true/false targets");
    let merge_block = llvm_block_for_line(blocks, result_load_line);
    for line in &result_writes {
        let block = llvm_block_for_line(blocks, *line);
        assert!(
            llvm_dominates(blocks, true_target, &block.name),
            "all six result writes must remain on the valid-header edge"
        );
        assert!(
            !llvm_reachable(blocks, false_target, &block.name),
            "invalid-header flow must not reach a scalar result write"
        );
        assert_ne!(
            block.name, merge_block.name,
            "shared merge/return block must not mutate the result local"
        );
        assert!(
            *line < result_load_line,
            "valid result writes must complete before the shared aggregate load/return"
        );
    }
}

fn assert_affine_accumulator_contract(function: &str) {
    let blocks = llvm_blocks(function);
    let values_base = exact_argument_array_base(function, "[2 x i32]", "values");
    let weights_base = exact_argument_array_base(function, "[2 x i32]", "weights");
    let accesses = dynamic_i32_accesses(function, "[2 x i32]");
    assert_eq!(accesses.len(), 2);
    assert_eq!(accesses[0].base, values_base);
    assert_eq!(accesses[1].base, weights_base);
    for access in &accesses {
        assert_eq!(access.consumer, "load i32");
        assert_identity_linked_guard_consumer(function, &access.index, 2, "[2 x i32]", "load i32");
    }
    let index_slot = loaded_i32_pointer(function, &accesses[0].index);
    assert_eq!(loaded_i32_pointer(function, &accesses[1].index), index_slot);
    let counted = assert_exact_counted_loop(function, &blocks, &index_slot, 2);
    let product = ssa_value_for_rhs(
        function,
        &format!("mul i32 {}, {}", accesses[0].value, accesses[1].value),
    );
    let bias_homes = function
        .lines()
        .enumerate()
        .filter_map(|(line, text)| {
            text.trim()
                .strip_prefix("store i32 %aero.arg.bias, i32* ")
                .and_then(|rhs| rhs.strip_suffix(", align 4"))
                .map(|pointer| (line, pointer))
        })
        .collect::<Vec<_>>();
    assert_eq!(bias_homes.len(), 1);
    let (bias_home_line, bias_home) = bias_homes[0];
    let bias_value = ssa_value_for_rhs(function, &format!("load i32, i32* {bias_home}, align 4"));
    let accumulator_stores = function
        .lines()
        .enumerate()
        .filter_map(|(line, text)| {
            text.trim()
                .strip_prefix(&format!("store i32 {bias_value}, i32* "))
                .and_then(|rhs| rhs.strip_suffix(", align 4"))
                .map(|pointer| (line, pointer))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        accumulator_stores.len(),
        1,
        "exact bias value must initialize one accumulator pointer"
    );
    let (bias_store_line, accumulator_slot) = accumulator_stores[0];
    assert!(bias_home_line < bias_store_line);
    assert!(bias_store_line < llvm_block(&blocks, &counted.start).start);
    assert!(llvm_dominates(
        &blocks,
        &llvm_block_for_line(&blocks, bias_store_line).name,
        &counted.start,
    ));
    let accumulator_loads = function
        .lines()
        .filter_map(|line| {
            let rhs = line.trim().split_once(" = ")?;
            (rhs.1 == format!("load i32, i32* {accumulator_slot}, align 4"))
                .then(|| rhs.0.to_string())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        accumulator_loads.len(),
        2,
        "affine accumulator must be read once per loop iteration IR and once at exit"
    );
    let add_links = accumulator_loads
        .iter()
        .flat_map(|value| {
            [
                (value.as_str(), format!("add i32 {value}, {product}")),
                (value.as_str(), format!("add i32 {product}, {value}")),
            ]
        })
        .filter_map(|(value, rhs)| {
            function
                .lines()
                .find(|line| line.trim().ends_with(&format!(" = {rhs}")))
                .map(|line| (value.to_string(), ssa_definition(line).to_string()))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        add_links.len(),
        1,
        "affine product must add to exactly one load from the exact accumulator"
    );
    let (pre_add, sum) = &add_links[0];
    let pre_add_line = function
        .lines()
        .position(|line| line.trim().starts_with(&format!("{pre_add} = load i32")))
        .expect("affine pre-add accumulator load line");
    assert!(llvm_dominates(
        &blocks,
        &counted.body,
        &llvm_block_for_line(&blocks, pre_add_line).name,
    ));
    assert_eq!(
        occurrences(
            function,
            &format!("store i32 {sum}, i32* {accumulator_slot}, align 4")
        ),
        1,
        "affine sum must update the same accumulator pointer"
    );
    let returned = accumulator_loads
        .iter()
        .find(|value| function.contains(&format!("ret i32 {value}")))
        .expect("post-loop accumulator load must feed return");
    let return_line = function
        .lines()
        .position(|line| line.trim() == format!("ret i32 {returned}"))
        .expect("affine return line");
    assert_eq!(
        llvm_block_for_line(&blocks, return_line).name,
        counted.end,
        "affine must return the exact accumulator only on loop exit"
    );
}

fn exact_literal_array_bases(function: &str, values: &[i32]) -> Vec<String> {
    let aggregate = format!("[{} x i32]", values.len());
    let bases = function
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_suffix(&format!(" = alloca {aggregate}, align 8"))
        })
        .filter(|base| {
            values.iter().enumerate().all(|(lane, value)| {
                let gep = format!(
                    "getelementptr inbounds {aggregate}, {aggregate}* {base}, i64 0, i64 {lane}"
                );
                let pointers = function
                    .lines()
                    .filter(|line| line.trim().ends_with(&format!(" = {gep}")))
                    .map(ssa_definition)
                    .collect::<Vec<_>>();
                pointers.len() == 1
                    && occurrences(
                        function,
                        &format!("store i32 {value}, i32* {}, align 4", pointers[0]),
                    ) == 1
            })
        })
        .map(str::to_string)
        .collect::<Vec<_>>();
    assert!(
        !bases.is_empty(),
        "exact literal array {values:?} must identify at least one local"
    );
    bases
}

fn assert_literal_array_initialized_before_load(
    function: &str,
    values: &[i32],
    base: &str,
    loaded_value: &str,
) {
    let aggregate = format!("[{} x i32]", values.len());
    let load_line = function
        .lines()
        .position(|line| {
            line.trim()
                == format!("{loaded_value} = load {aggregate}, {aggregate}* {base}, align 8")
        })
        .expect("literal aggregate load line");
    for (lane, value) in values.iter().enumerate() {
        let pointer = static_array_lane_pointer(function, &aggregate, base, lane);
        let store_line = function
            .lines()
            .position(|line| line.trim() == format!("store i32 {value}, i32* {pointer}, align 4"))
            .expect("literal lane initializer line");
        assert!(
            store_line < load_line,
            "literal {aggregate} lane {lane} must initialize before aggregate transport"
        );
    }
}

fn exact_aggregate_call_lines(
    function: &str,
    callee: &str,
    return_type: &str,
    argument_types: &[&str],
) -> Vec<(usize, Vec<String>)> {
    let prefix = format!("call {return_type} @{callee}(");
    function
        .lines()
        .enumerate()
        .filter_map(|(line, text)| {
            let rhs = text.trim().split_once(" = ")?.1;
            let arguments = rhs.strip_prefix(&prefix)?.strip_suffix(')')?;
            let arguments = arguments.split(", ").collect::<Vec<_>>();
            if arguments.len() != argument_types.len() {
                return None;
            }
            let values = arguments
                .iter()
                .zip(argument_types)
                .map(|(argument, aggregate)| {
                    argument
                        .strip_prefix(&format!("{aggregate} "))
                        .unwrap_or_else(|| {
                            panic!("{callee} call argument changed type: {argument}")
                        })
                        .to_string()
                })
                .collect::<Vec<_>>();
            Some((line, values))
        })
        .collect()
}

fn assert_main_source_preservation(function: &str) {
    let ordinary = [2, 3, 1, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
    let wrapping = [
        2,
        3,
        1,
        2,
        -3,
        5,
        i32::MAX,
        4,
        -2,
        i32::MIN,
        -1,
        3,
        i32::MAX,
        i32::MAX,
        i32::MAX,
        -1,
        13,
    ];
    let malformed = [2, 4, 1, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
    let ordinary_bases = exact_literal_array_bases(function, &ordinary);
    let wrapping_bases = exact_literal_array_bases(function, &wrapping);
    let malformed_bases = exact_literal_array_bases(function, &malformed);
    assert_eq!(
        ordinary_bases.len(),
        2,
        "ordinary source and expected locals"
    );
    assert_eq!(
        wrapping_bases.len(),
        2,
        "wrapping source and expected locals"
    );
    assert_eq!(malformed_bases.len(), 1, "one malformed source local");
    let malformed_source = malformed_bases[0].clone();

    let decode_calls =
        exact_aggregate_call_lines(function, "decode_and_score", "[6 x i32]", &["[17 x i32]"]);
    assert_eq!(decode_calls.len(), 3);
    let decoded_bases = decode_calls
        .iter()
        .map(|(_, arguments)| {
            ssa_rhs(function, &arguments[0])
                .strip_prefix("load [17 x i32], [17 x i32]* ")
                .and_then(|rhs| rhs.strip_suffix(", align 8"))
                .expect("decode call argument must load an exact source record local")
                .to_string()
        })
        .collect::<Vec<_>>();
    assert!(ordinary_bases.contains(&decoded_bases[0]));
    assert!(wrapping_bases.contains(&decoded_bases[1]));
    assert_eq!(decoded_bases[2], malformed_source);
    for ((_, arguments), (values, base)) in decode_calls.iter().zip([
        (ordinary.as_slice(), decoded_bases[0].as_str()),
        (wrapping.as_slice(), decoded_bases[1].as_str()),
        (malformed.as_slice(), decoded_bases[2].as_str()),
    ]) {
        assert_literal_array_initialized_before_load(function, values, base, &arguments[0]);
    }
    assert_ne!(decoded_bases[0], decoded_bases[1]);
    assert_ne!(decoded_bases[0], decoded_bases[2]);
    assert_ne!(decoded_bases[1], decoded_bases[2]);
    let last_decode = decode_calls.last().expect("three decode calls").0;

    let compare_calls = exact_aggregate_call_lines(
        function,
        "records_equal",
        "i32",
        &["[17 x i32]", "[17 x i32]"],
    );
    assert_eq!(compare_calls.len(), 2);
    assert!(
        compare_calls.iter().all(|(line, _)| *line > last_decode),
        "both source-preservation comparisons must follow all three decode calls"
    );
    let load_base = |value: &str| {
        ssa_rhs(function, value)
            .strip_prefix("load [17 x i32], [17 x i32]* ")
            .and_then(|rhs| rhs.strip_suffix(", align 8"))
            .unwrap_or_else(|| panic!("`{value}` must load one exact record local"))
    };
    assert_eq!(load_base(&compare_calls[0].1[0]), decoded_bases[0]);
    assert_eq!(load_base(&compare_calls[1].1[0]), decoded_bases[1]);
    for ((_, arguments), expected_bases) in
        compare_calls.iter().zip([&ordinary_bases, &wrapping_bases])
    {
        let right_base = load_base(&arguments[1]);
        assert!(expected_bases.iter().any(|base| base == right_base));
        assert_literal_array_initialized_before_load(
            function,
            if expected_bases == &ordinary_bases {
                &ordinary
            } else {
                &wrapping
            },
            right_base,
            &arguments[1],
        );
        assert_ne!(
            load_base(&arguments[0]),
            right_base,
            "source preservation must compare a source local against a separate expected local"
        );
    }
}

fn assert_records_equal_contract(function: &str) {
    let blocks = llvm_blocks(function);
    let left_base = exact_argument_array_base(function, "[17 x i32]", "left");
    let right_base = exact_argument_array_base(function, "[17 x i32]", "right");
    assert_ne!(left_base, right_base);
    let accesses = dynamic_i32_accesses(function, "[17 x i32]");
    assert_eq!(
        accesses.len(),
        2,
        "records_equal needs exactly two guarded reads"
    );
    assert_eq!(accesses[0].base, left_base);
    assert_eq!(accesses[1].base, right_base);
    for access in &accesses {
        assert_eq!(access.consumer, "load i32");
        assert_identity_linked_guard_consumer(
            function,
            &access.index,
            17,
            "[17 x i32]",
            "load i32",
        );
    }
    let index_slot = loaded_i32_pointer(function, &accesses[0].index);
    assert_eq!(
        loaded_i32_pointer(function, &accesses[1].index),
        index_slot,
        "left and right guarded reads must reuse the exact induction local"
    );
    let counted = assert_exact_counted_loop(function, &blocks, &index_slot, 17);
    for access in &accesses {
        for line in [access.line, access.consumer_line] {
            let block = llvm_block_for_line(&blocks, line);
            assert!(llvm_dominates(&blocks, &counted.body, &block.name));
            assert!(llvm_dominates(&blocks, &counted.start, &block.name));
        }
    }

    let mismatch = ssa_value_for_rhs(
        function,
        &format!("icmp ne i32 {}, {}", accesses[0].value, accesses[1].value),
    );
    let mismatch_branches = function
        .lines()
        .enumerate()
        .filter_map(|(line, text)| {
            text.trim()
                .strip_prefix(&format!("br i1 {mismatch}, label %"))
                .map(|targets| (line, targets))
        })
        .collect::<Vec<_>>();
    assert_eq!(mismatch_branches.len(), 1);
    let (mismatch_line, targets) = mismatch_branches[0];
    let (different, equal) = targets
        .split_once(", label %")
        .expect("records_equal mismatch branch labels");
    let mismatch_block = llvm_block_for_line(&blocks, mismatch_line);
    assert_eq!(
        mismatch_block.successors,
        vec![different.to_string(), equal.to_string()]
    );
    let different_block = llvm_block(&blocks, different);
    let equal_block = llvm_block(&blocks, equal);
    assert_eq!(different_block.successors, equal_block.successors);
    assert_eq!(different_block.successors.len(), 1);
    let update = &different_block.successors[0];
    assert_eq!(update, &counted.update);

    let zero_stores = function
        .lines()
        .enumerate()
        .filter_map(|(line, text)| {
            text.trim()
                .strip_prefix("store i32 0, i32* ")
                .and_then(|rhs| rhs.strip_suffix(", align 4"))
                .map(|pointer| (line, pointer))
        })
        .filter(|(line, _)| {
            let block = llvm_block_for_line(&blocks, *line);
            block.name == different
        })
        .collect::<Vec<_>>();
    assert_eq!(
        zero_stores.len(),
        1,
        "mismatch true edge must perform one equality update"
    );
    let equality_slot = zero_stores[0].1;
    let equality_initializations = function
        .lines()
        .enumerate()
        .filter(|(_, line)| line.trim() == format!("store i32 1, i32* {equality_slot}, align 4"))
        .map(|(line, _)| line)
        .collect::<Vec<_>>();
    assert_eq!(
        equality_initializations.len(),
        1,
        "records_equal must initialize equality to true exactly once"
    );
    let equality_initializer_block = llvm_block_for_line(&blocks, equality_initializations[0]);
    assert!(
        equality_initializations[0] < llvm_block(&blocks, &counted.start).start,
        "equality initialization must precede the 0..17 loop start"
    );
    assert!(
        llvm_dominates(&blocks, &equality_initializer_block.name, &counted.start),
        "equality initialization block must dominate the 0..17 loop"
    );
    assert_eq!(
        function
            .lines()
            .filter(|line| {
                let line = line.trim();
                line.starts_with("store i32 ")
                    && line.ends_with(&format!(", i32* {equality_slot}, align 4"))
            })
            .count(),
        2,
        "only initialization and mismatch may write the equality local"
    );
    let equal_instructions = function
        .lines()
        .skip(equal_block.start + 1)
        .take(equal_block.end - equal_block.start - 1)
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(equal_instructions, vec![format!("br label %{update}")]);
    let returned = ssa_value_for_rhs(
        function,
        &format!("load i32, i32* {equality_slot}, align 4"),
    );
    let end_block = llvm_block(&blocks, &counted.end);
    assert_eq!(
        llvm_block_for_line(
            &blocks,
            function
                .lines()
                .position(|line| line.trim() == format!("ret i32 {returned}"))
                .expect("records_equal return")
        )
        .name,
        end_block.name
    );
    assert_eq!(
        occurrences(function, &format!("ret i32 {returned}")),
        1,
        "records_equal loop exit must return the exact equality local"
    );
}

fn guarded_index_for_bound(function: &str, upper_bound: i32) -> String {
    let lines = function.lines().collect::<Vec<_>>();
    let mut indexes = Vec::new();
    for pair in lines.windows(2) {
        let Some((_, lower_rhs)) = pair[0].trim().split_once(" = ") else {
            continue;
        };
        let Some(index) = lower_rhs
            .strip_prefix("icmp sge i32 ")
            .and_then(|rhs| rhs.strip_suffix(", 0"))
        else {
            continue;
        };
        let expected_upper = format!("icmp slt i32 {index}, {upper_bound}");
        if pair[1]
            .trim()
            .split_once(" = ")
            .is_some_and(|(_, rhs)| rhs == expected_upper)
        {
            indexes.push(index.to_string());
        }
    }
    assert_eq!(
        indexes.len(),
        1,
        "expected exactly one guarded index with upper bound {upper_bound}:\n{function}"
    );
    indexes.pop().expect("unique guarded index")
}

fn workflow_named_step<'a>(workflow: &'a str, name: &str) -> &'a str {
    let header = format!("    - name: {name}");
    assert_eq!(
        occurrences(workflow, &header),
        1,
        "workflow must contain exactly one `{header}`"
    );
    let start = workflow.find(&header).expect("unique workflow step header");
    let remaining = &workflow[start..];
    let end = remaining[header.len()..]
        .find("\n    - name:")
        .map_or(remaining.len(), |offset| header.len() + offset);
    &remaining[..end]
}

fn linux_bounds_loop_names(step: &str) -> Vec<&str> {
    let header = "for bounds_file in ";
    assert_eq!(
        occurrences(step, header),
        1,
        "Linux step bounds loop drifted"
    );
    let list = step
        .split_once(header)
        .expect("Linux bounds loop header")
        .1
        .split_once("; do")
        .expect("Linux bounds loop terminator")
        .0;
    list.split_whitespace()
        .filter(|token| *token != "\\")
        .collect()
}

fn windows_bounds_loop_names(step: &str) -> Vec<&str> {
    let header = "foreach ($boundsFile in @(";
    assert_eq!(
        occurrences(step, header),
        1,
        "Windows step bounds loop drifted"
    );
    let list = step
        .split_once(header)
        .expect("Windows bounds loop header")
        .1
        .split_once(")) {")
        .expect("Windows bounds loop terminator")
        .0;
    list.split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(|name| name.trim_matches('"'))
        .collect()
}

fn llvm_body_without_public_route_headers(llvm: &str) -> Vec<&str> {
    llvm.lines()
        .filter(|line| {
            !line.starts_with("target datalayout = ")
                && !line.starts_with("target triple = ")
                && !line.starts_with("; aero.graph_compilation")
        })
        .collect()
}

fn assert_dynamic_guard_sequences(llvm: &str, aggregate: &str, expected: usize) {
    for anchor in ["icmp sge i32", "call void @llvm.trap()", "sext i32"] {
        assert_eq!(
            occurrences(llvm, anchor),
            expected,
            "expected {expected} exact dynamic-array occurrences of `{anchor}`:\n{llvm}"
        );
    }

    let mut cursor = 0;
    for _ in 0..expected {
        let lower = llvm[cursor..]
            .find("icmp sge i32")
            .map(|offset| cursor + offset)
            .expect("signed lower guard");
        let upper = llvm[lower..]
            .find("icmp slt i32")
            .map(|offset| lower + offset)
            .expect("signed upper guard");
        let conjunction = llvm[upper..]
            .find("and i1")
            .map(|offset| upper + offset)
            .expect("combined bounds predicate");
        let branch = llvm[conjunction..]
            .find("br i1")
            .map(|offset| conjunction + offset)
            .expect("guard branch");
        let trap = llvm[branch..]
            .find("call void @llvm.trap()")
            .map(|offset| branch + offset)
            .expect("trap branch");
        let safe = llvm[trap..]
            .find("aero.bounds.safe.")
            .map(|offset| trap + offset)
            .expect("safe label");
        let extension = llvm[safe..]
            .find("sext i32")
            .map(|offset| safe + offset)
            .expect("post-guard sign extension");
        let address_anchor = format!("getelementptr inbounds {aggregate}");
        let address = llvm[extension..]
            .find(&address_anchor)
            .map(|offset| extension + offset)
            .expect("post-guard array address");
        assert!(
            lower < upper
                && upper < conjunction
                && conjunction < branch
                && branch < trap
                && trap < safe
                && safe < extension
                && extension < address
        );
        cursor = address + address_anchor.len();
    }
}

fn dynamic_array_gep_scopes<'a>(llvm: &'a str, aggregate: &str) -> Vec<(&'a str, usize, &'a str)> {
    let address_anchor = format!("getelementptr inbounds {aggregate}");
    llvm.match_indices(&address_anchor)
        .filter_map(|(address, _)| {
            let line_start = llvm[..address]
                .rfind('\n')
                .map_or(0, |position| position + 1);
            let line_end = llvm[address..]
                .find('\n')
                .map_or(llvm.len(), |position| address + position);
            let line = &llvm[line_start..line_end];
            if !line.contains(", i64 %reg") {
                return None;
            }
            let pointer = line
                .split_once(" = ")
                .map(|(pointer, _)| pointer.trim())
                .expect("dynamic array pointer definition");
            let function_start = llvm[..line_start]
                .rfind("\ndefine ")
                .map(|position| position + 1)
                .expect("dynamic array GEP enclosing function start");
            let function_end = llvm[line_end..]
                .find("\n}")
                .map(|position| line_end + position + 2)
                .expect("dynamic array GEP enclosing function end");
            Some((
                pointer,
                line_start - function_start,
                &llvm[function_start..function_end],
            ))
        })
        .collect()
}

fn assert_guarded_dynamic_array_reads_and_writes(
    llvm: &str,
    aggregate: &str,
    expected_reads: usize,
    expected_writes: usize,
) {
    let dynamic_geps = dynamic_array_gep_scopes(llvm, aggregate);
    assert_eq!(
        dynamic_geps.len(),
        expected_reads + expected_writes,
        "expected {expected_reads} dynamic read GEP(s) and {expected_writes} dynamic write GEP(s) for {aggregate}:\n{llvm}"
    );

    let mut loads = 0;
    let mut stores = 0;
    for (pointer, definition, function) in dynamic_geps {
        let load = format!("load i32, i32* {pointer}, align 4");
        let store_prefix = "store i32 ";
        let store_target = format!("i32* {pointer}, align 4");
        let load_positions = function
            .match_indices(&load)
            .map(|(position, _)| position)
            .collect::<Vec<_>>();
        let store_positions = function
            .match_indices(&store_target)
            .filter_map(|(position, _)| {
                let line_start = function[..position].rfind('\n').map_or(0, |line| line + 1);
                let line_end = function[position..]
                    .find('\n')
                    .map_or(function.len(), |line| position + line);
                function[line_start..line_end]
                    .contains(store_prefix)
                    .then_some(position)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            load_positions.len() + store_positions.len(),
            1,
            "guarded pointer {pointer} must feed exactly one scalar read or write:\n{llvm}"
        );
        assert!(
            load_positions
                .iter()
                .chain(store_positions.iter())
                .all(|position| *position > definition),
            "guarded pointer {pointer} was consumed before its GEP"
        );
        loads += load_positions.len();
        stores += store_positions.len();
    }
    assert_eq!(
        (loads, stores),
        (expected_reads, expected_writes),
        "dynamic GEP consumers diverged for {aggregate}:\n{llvm}"
    );
}

fn assert_single_dynamic_array_store_value(llvm: &str, aggregate: &str, value: i32) {
    let dynamic_geps = dynamic_array_gep_scopes(llvm, aggregate);
    assert_eq!(
        dynamic_geps.len(),
        1,
        "expected one dynamic array store GEP"
    );
    let (pointer, definition, function) = dynamic_geps[0];
    let store = format!("store i32 {value}, i32* {pointer}, align 4");
    let store_position = function
        .find(&store)
        .unwrap_or_else(|| panic!("dynamic pointer {pointer} omitted exact `{store}`:\n{llvm}"));
    assert!(
        store_position > definition,
        "dynamic array store consumed {pointer} before its GEP"
    );
}

#[test]
fn fixed_int_array_profile_is_selectable_on_public_check() {
    assert_eq!(reference_kernel(), 2035);
    let workspace = TestWorkspace::new("check-red");
    let source = write_program(&workspace);
    let output = run_cli(
        &workspace,
        &[
            Path::new("check"),
            &source,
            Path::new("--language-profile"),
            Path::new("exact-i32-array-v0"),
        ],
    );
    assert!(
        output.status.success(),
        "exact-i32-array-v0 check is absent or rejected its complete kernel:\n{}",
        combined_output(&output)
    );
}

#[test]
fn fixed_int_array_profile_is_selectable_on_public_build() {
    let workspace = TestWorkspace::new("build-red");
    let source = write_program(&workspace);
    let llvm = workspace.path("kernel.ll");
    let output = run_cli(
        &workspace,
        &[
            Path::new("build"),
            &source,
            Path::new("-o"),
            &llvm,
            Path::new("--language-profile"),
            Path::new("exact-i32-array-v0"),
        ],
    );
    assert!(
        output.status.success() && llvm.is_file(),
        "exact-i32-array-v0 build is absent or omitted LLVM:\n{}",
        combined_output(&output)
    );
    let public_llvm = fs::read_to_string(&llvm).expect("public build must write readable LLVM");
    for route_header in [
        "; aero.graph_compilation=enabled",
        "; aero.graph_compilation.execution_scope=internal-scalar-helper",
        "; aero.graph_compilation.device_execution=false",
        "; aero.graph_compilation.backend=cpu",
        "; aero.graph_compilation.executable_fusion=true",
    ] {
        assert!(
            public_llvm.lines().any(|line| line == route_header),
            "public build omitted route header `{route_header}`"
        );
    }
    let library_llvm = compile_file(&source, exact_options())
        .expect("file library route should emit the same exact LLVM");
    assert_eq!(
        llvm_body_without_public_route_headers(&public_llvm),
        llvm_body_without_public_route_headers(&library_llvm),
        "public and library routes must have byte-identical LLVM bodies after the public route applies its documented graph and host-target framing"
    );
}

#[test]
fn fixed_int_array_profile_is_selectable_on_public_run() {
    let workspace = TestWorkspace::new("run-red");
    let source = write_program(&workspace);
    let output = run_cli(
        &workspace,
        &[
            Path::new("run"),
            &source,
            Path::new("--language-profile"),
            Path::new("exact-i32-array-v0"),
        ],
    );
    assert_eq!(
        output.status.code(),
        Some(91),
        "exact-i32-array-v0 run is absent or diverged from the oracle:\n{}",
        combined_output(&output)
    );
}

#[test]
fn stable_scalar_profile_still_rejects_the_array_kernel() {
    let error = compile_program(
        FIXED_INT_ARRAY_PROGRAM,
        CompilerOptions {
            language_profile: LanguageProfile::StableScalarV0,
            ..CompilerOptions::default()
        },
    )
    .expect_err("stable-scalar-v0 must retain its frozen array exclusion");
    assert!(error.contains("Language Profile Error: stable-scalar-v0 rejects"));

    let result_only = "fn make() -> [int; 1] { return [1]; } fn main() -> int { return 0; }";
    let error = compile_program(
        result_only,
        CompilerOptions {
            language_profile: LanguageProfile::StableScalarV0,
            ..CompilerOptions::default()
        },
    )
    .expect_err("stable-scalar-v0 must reject array results independently of parameters");
    assert_eq!(
        error,
        "Language Profile Error: stable-scalar-v0 rejects function result types"
    );
}

#[test]
fn experimental_profile_retains_the_legacy_double_array_lane() {
    let implicit = compile_program(FIXED_INT_ARRAY_PROGRAM, CompilerOptions::default())
        .expect("experimental array kernel control should compile");
    let explicit = compile_program(
        FIXED_INT_ARRAY_PROGRAM,
        CompilerOptions {
            language_profile: LanguageProfile::Experimental,
            ..CompilerOptions::default()
        },
    )
    .expect("explicit experimental array kernel control should compile");

    assert_eq!(implicit, explicit);
    assert!(implicit.contains("[8 x double]"));
    assert!(!implicit.contains("[8 x i32]"));
}

#[test]
fn exact_profile_is_shared_by_source_and_file_library_routes() {
    check_program(FIXED_INT_ARRAY_PROGRAM, exact_options())
        .expect("source check should admit the exact fixed-array kernel");
    let source_llvm = compile_program(FIXED_INT_ARRAY_PROGRAM, exact_options())
        .expect("source compile should emit the exact fixed-array kernel");

    let workspace = TestWorkspace::new("library-route-parity");
    let source = write_program(&workspace);
    check_file(&source, exact_options()).expect("file check should share exact admission");
    let file_llvm = compile_file(&source, exact_options())
        .expect("file compile should share exact physical lowering");
    assert_eq!(source_llvm, file_llvm);

    fs::write(&source, "mod missing; fn main() -> int { return 0; }")
        .expect("write unresolved module attempt");
    assert_eq!(
        check_file(&source, exact_options()).expect_err("profile must precede module resolution"),
        "Language Profile Error: exact-i32-array-v0 rejects module declarations"
    );
}

#[test]
fn general_checked_pipeline_already_owns_the_complete_immutable_array_value_class() {
    let tokens = try_tokenize_with_locations(IMMUTABLE_ARRAY_VALUE_COMPOSITION, None)
        .expect("immutable array composition control should lex");
    let ast =
        parse_with_locations(tokens).expect("immutable array composition control should parse");
    IrGenerator::new()
        .try_generate_ir(ast.clone())
        .expect("independent raw checked admission should already own immutable array composition");
    let mut analyzer = SemanticAnalyzer::new();
    let (_, analyzed) = analyzer
        .analyze(ast)
        .expect("general semantics should already own immutable array composition");
    let checked = IrGenerator::new()
        .try_generate_ir(analyzed)
        .expect("checked IR should already own immutable array composition");
    let metadata = format!("{:#?}", checked.metadata());
    for anchor in [
        "\"transform\": FunctionMetadata",
        "\"identity\": FunctionMetadata",
        "element: Int",
        "count: 3",
    ] {
        assert!(
            metadata.contains(anchor),
            "checked metadata omitted array-composition identity `{anchor}`:\n{metadata}"
        );
    }

    let llvm = compile_program(
        IMMUTABLE_ARRAY_VALUE_COMPOSITION,
        CompilerOptions::default(),
    )
    .expect("experimental control should retain general immutable array composition");
    assert!(llvm.contains("define [3 x double] @transform("));
    assert!(llvm.contains("call [3 x double] @identity("));
}

#[test]
fn general_pipeline_already_owns_initialized_mutable_array_result_production() {
    let tokens = try_tokenize_with_locations(MUTABLE_ARRAY_RESULT_PRODUCTION, None)
        .expect("mutable array-result production control should lex");
    let ast =
        parse_with_locations(tokens).expect("mutable array-result production control should parse");

    let checked = IrGenerator::new()
        .try_generate_ir(ast.clone())
        .expect("raw-AST checked admission should already own initialized array mutation");
    let exact_array_four = LogicalType::Array {
        element: Box::new(LogicalType::Int),
        count: 4,
    };
    let from_identifier = &checked.metadata().functions["from_identifier"];
    assert_eq!(
        from_identifier.signature.parameters,
        vec![("source".to_string(), exact_array_four.clone())],
        "raw checked metadata must retain the exact Array<Int, 4> input identity"
    );
    assert_eq!(
        from_identifier.signature.result, exact_array_four,
        "raw checked metadata must retain the exact Array<Int, 4> result identity"
    );
    let output_place = from_identifier
        .places
        .values()
        .find(|place| place.name.as_deref() == Some("output"))
        .expect("raw checked metadata must retain the mutable output place");
    assert_eq!(
        output_place.pointee, from_identifier.signature.result,
        "mutable output place and returned array must share one logical identity"
    );
    assert!(
        from_identifier
            .results
            .values()
            .any(|result| result == &from_identifier.signature.result),
        "raw checked metadata must retain an Array<Int, 4> produced value"
    );
    let checked_debug = format!("{checked:#?}");
    for marker in [
        "CheckedMutableOwnedPlaceAlloca",
        "CheckedOwnedPlaceAssignment",
        "element: Int",
        "count: 4",
    ] {
        assert!(
            checked_debug.contains(marker),
            "raw checked IR omitted mutable array-result marker `{marker}`:\n{checked_debug}"
        );
    }

    let mut analyzer = SemanticAnalyzer::new();
    analyzer
        .analyze(ast)
        .expect("general semantics should already own initialized array mutation and return");

    let implicit = compile_program(MUTABLE_ARRAY_RESULT_PRODUCTION, CompilerOptions::default())
        .expect("experimental control should compile mutable array-result production");
    let explicit = compile_program(
        MUTABLE_ARRAY_RESULT_PRODUCTION,
        CompilerOptions {
            language_profile: LanguageProfile::Experimental,
            ..CompilerOptions::default()
        },
    )
    .expect("explicit experimental control should compile mutable array-result production");
    assert_eq!(implicit, explicit);
    for anchor in [
        "define [2 x double] @from_literal()",
        "define [4 x double] @from_identifier([4 x double]",
        "define [4 x double] @from_call()",
        "alloca [2 x double], align 8",
        "alloca [4 x double], align 8",
        "store [2 x double]",
        "load [2 x double]",
        "store [4 x double]",
        "load [4 x double]",
        "store double",
        "ret [4 x double]",
        "call i32 @score([4 x double]",
    ] {
        assert!(
            implicit.contains(anchor),
            "experimental control omitted `{anchor}`:\n{implicit}"
        );
    }
    for forbidden in ["[2 x i32]", "[4 x i32]"] {
        assert!(
            !implicit.contains(forbidden),
            "experimental mutable-owner storage leaked `{forbidden}`:\n{implicit}"
        );
    }
}

#[test]
fn exact_profile_admits_initialized_mutable_array_result_production() {
    let mut failures = Vec::new();

    if let Err(error) = check_program(MUTABLE_ARRAY_RESULT_PRODUCTION, exact_options()) {
        if !error.starts_with("Language Profile Error: exact-i32-array-v0 rejects ") {
            failures.push(format!(
                "exact source check crossed the pre-semantic boundary: {error}"
            ));
        }
        failures.push(format!(
            "exact source check rejected mutable array-result production: {error}"
        ));
    }

    match compile_program(MUTABLE_ARRAY_RESULT_PRODUCTION, exact_options()) {
        Err(error) => failures.push(format!(
            "exact source compilation rejected mutable array-result production: {error}"
        )),
        Ok(first) => {
            let second = compile_program(MUTABLE_ARRAY_RESULT_PRODUCTION, exact_options())
                .expect("a second exact mutable array-result compilation should remain admitted");
            assert_eq!(
                first, second,
                "exact mutable array-result LLVM must be deterministic"
            );
            for anchor in [
                "define [2 x i32] @from_literal()",
                "define [4 x i32] @from_identifier([4 x i32] %aero.arg.source)",
                "define [4 x i32] @from_call()",
                "store [4 x i32]",
                "store i32",
                "ret [4 x i32]",
                "call i32 @score([4 x i32]",
            ] {
                assert!(
                    first.contains(anchor),
                    "exact mutable array-result LLVM omitted `{anchor}`:\n{first}"
                );
            }
            for forbidden in ["double", "fptosi", "sitofp", " nsw ", " nuw "] {
                assert!(
                    !first.contains(forbidden),
                    "exact mutable array-result LLVM leaked `{forbidden}`:\n{first}"
                );
            }
            assert_dynamic_guard_sequences(&first, "[4 x i32]", 2);
            assert_guarded_dynamic_array_reads_and_writes(&first, "[4 x i32]", 1, 1);
        }
    }

    let workspace = TestWorkspace::new("mutable-array-result-production-red");
    let source = workspace.path("main.aero");
    fs::write(&source, MUTABLE_ARRAY_RESULT_PRODUCTION)
        .expect("write mutable array-result production source");
    let llvm = workspace.path("mutable-array-result.ll");

    let check = run_cli(
        &workspace,
        &[
            Path::new("check"),
            &source,
            Path::new("--language-profile"),
            Path::new("exact-i32-array-v0"),
        ],
    );
    if !check.status.success() {
        failures.push(format!(
            "public check rejected mutable array-result production: {}",
            combined_output(&check)
        ));
    }

    let build = run_cli(
        &workspace,
        &[
            Path::new("build"),
            &source,
            Path::new("-o"),
            &llvm,
            Path::new("--language-profile"),
            Path::new("exact-i32-array-v0"),
        ],
    );
    if !build.status.success() {
        assert!(
            !llvm.exists(),
            "a rejected exact mutable array-result build must not leave an LLVM artifact"
        );
        failures.push(format!(
            "public build rejected mutable array-result production: {}",
            combined_output(&build)
        ));
    } else if !llvm.is_file() {
        failures.push("public build admitted the source but omitted its LLVM artifact".to_string());
    }

    let run = run_cli(
        &workspace,
        &[
            Path::new("run"),
            &source,
            Path::new("--language-profile"),
            Path::new("exact-i32-array-v0"),
        ],
    );
    if run.status.code() != Some(91) {
        failures.push(format!(
            "public run did not preserve the source Copy and consume the returned result (status={:?}): {}",
            run.status.code(),
            combined_output(&run)
        ));
    }

    assert!(
        failures.is_empty(),
        "CAP-019 mutable exact-array result production is not complete:\n{}",
        failures.join("\n")
    );
}

#[test]
fn mutable_array_result_production_retains_profile_separation_boundaries() {
    let exact_rejections = [
        (
            "immutable assignment root",
            "fn main() -> int { let values: [int; 2] = [1, 2]; values[0] = 3; return values[0]; }",
            "projected assignment targets rooted in immutable exact-array bindings",
        ),
        (
            "wrong-count initializer",
            "fn seed() -> [int; 2] { return [1, 2]; } fn bad() -> [i32; 3] { let mut values: [int; 3] = seed(); return values; } fn main() -> int { return 0; }",
            "array literal counts that differ from their annotations",
        ),
        (
            "non-Int initializer",
            "fn bad() -> [int; 2] { let mut values: [i32; 2] = [1 < 2, 2 < 3]; return values; } fn main() -> int { return 0; }",
            "array literal elements other than exact Int expressions",
        ),
        (
            "nested initializer",
            "fn main() -> int { let mut values: [[int; 1]; 1] = [[1]]; return 0; }",
            "binding annotation types",
        ),
        (
            "mutable alias initializer",
            "fn main() -> int { let mut source: [int; 2] = [1, 2]; let mut alias: [i32; 2] = source; return alias[0]; }",
            "mutable exact-array values as initializer sources",
        ),
        (
            "inferred mutable alias initializer",
            "fn main() -> int { let mut source: [int; 2] = [1, 2]; let mut alias = source; return alias[0]; }",
            "mutable exact-array values as initializer sources",
        ),
        (
            "uninitialized mutable array",
            "fn main() -> int { let mut values: [int; 2]; return 0; }",
            "uninitialized bindings",
        ),
        (
            "array-valued element store",
            "fn main() -> int { let mut values: [int; 2] = [1, 2]; values[0] = [3, 4]; return values[0]; }",
            "array values in exact integer expressions",
        ),
        (
            "non-Int runtime selector",
            "fn main() -> int { let mut values: [int; 2] = [1, 2]; let selector: bool = 1 < 2; values[selector] = 3; return values[0]; }",
            "non-Int values in exact integer expressions",
        ),
        (
            "non-Int stored value",
            "fn main() -> int { let mut values: [int; 2] = [1, 2]; let flag: bool = 1 < 2; values[0] = flag; return values[0]; }",
            "non-Int values in exact integer expressions",
        ),
        (
            "array reference escape",
            "fn main() -> int { let mut values: [int; 2] = [1, 2]; let view = &mut values; values[0] = 3; return values[0]; }",
            "reference expressions",
        ),
        (
            "whole-array reassignment",
            "fn main() -> int { let mut values: [int; 2] = [1, 2]; values = [3, 4]; return values[0]; }",
            "array writes",
        ),
        (
            "recursive array transform",
            "fn bad(source: [int; 1]) -> [i32; 1] { let mut output: [int; 1] = source; output[0] = output[0] + 1; return bad(output); } fn main() -> int { return 0; }",
            "recursive function call cycles",
        ),
    ];

    match check_program(MUTABLE_ARRAY_RESULT_PRODUCTION, exact_options()) {
        Err(error) => assert_eq!(
            error, "Language Profile Error: exact-i32-array-v0 rejects mutable array bindings",
            "only the known CAP-019 red barrier may defer the prospective exclusion matrix"
        ),
        Ok(()) => {
            for (label, source, expected) in exact_rejections {
                let error = check_program(source, exact_options())
                    .expect_err("excluded mutable exact-array topology must fail closed");
                assert!(
                    error.starts_with("Language Profile Error: exact-i32-array-v0 rejects "),
                    "{label} crossed the profile boundary: {error}"
                );
                assert!(
                    error.contains(expected),
                    "{label}: expected `{expected}`, got: {error}"
                );
                assert!(
                    !error.contains("Semantic Analysis Error")
                        && !error.contains("IR Generation Error"),
                    "{label} reached a later compiler phase: {error}"
                );
            }
        }
    }

    let result_only = "fn make() -> [int; 1] { return [1]; } fn main() -> int { return 0; }";
    assert_eq!(
        check_program(
            result_only,
            CompilerOptions {
                language_profile: LanguageProfile::StableScalarV0,
                ..CompilerOptions::default()
            }
        ),
        Err("Language Profile Error: stable-scalar-v0 rejects function result types".to_string())
    );

    let mutation_only =
        "fn main() -> int { let mut values: [int; 2] = [1, 2]; values[0] = 3; return values[0]; }";
    assert_eq!(
        check_program(
            mutation_only,
            CompilerOptions {
                language_profile: LanguageProfile::StableScalarV0,
                ..CompilerOptions::default()
            }
        ),
        Err(
            "Language Profile Error: stable-scalar-v0 rejects binding annotation types".to_string()
        )
    );
}

#[test]
fn exact_profile_admits_the_complete_immutable_array_value_composition_class() {
    check_program(IMMUTABLE_ARRAY_VALUE_COMPOSITION, exact_options())
        .expect("exact profile should admit every frozen immutable array value placement");
    let first = compile_program(IMMUTABLE_ARRAY_VALUE_COMPOSITION, exact_options())
        .expect("exact profile should lower immutable array value composition");
    let second = compile_program(IMMUTABLE_ARRAY_VALUE_COMPOSITION, exact_options())
        .expect("exact profile should lower deterministically");
    assert_eq!(
        first, second,
        "exact immutable-array LLVM must be deterministic"
    );

    for anchor in [
        "define [3 x i32] @transform([3 x i32] %aero.arg.values)",
        "define [3 x i32] @identity([3 x i32] %aero.arg.values)",
        "define [3 x i32] @return_call([3 x i32] %aero.arg.values)",
        "call [3 x i32] @transform([3 x i32]",
        "call [3 x i32] @identity([3 x i32]",
        "call [3 x i32] @return_call([3 x i32]",
        "store [3 x i32]",
        "load [3 x i32]",
        "ret [3 x i32]",
        "getelementptr inbounds [3 x i32]",
    ] {
        assert!(
            first.contains(anchor),
            "missing exact composition anchor `{anchor}`:\n{first}"
        );
    }
    for forbidden in ["[3 x double]", "fptosi", "sitofp", " nsw ", " nuw "] {
        assert!(
            !first.contains(forbidden),
            "exact array value composition leaked `{forbidden}`:\n{first}"
        );
    }
}

#[test]
fn exact_array_value_composition_retains_topology_and_mutability_separation() {
    let cases = [
        (
            "zero result",
            "fn bad() -> [int; 0] { return []; } fn main() -> int { return 0; }",
            "function result types",
        ),
        (
            "nested result",
            "fn bad() -> [[int; 1]; 1] { return [[1]]; } fn main() -> int { return 0; }",
            "function result types",
        ),
        (
            "non-int result",
            "fn bad() -> [bool; 1] { return [1 < 2]; } fn main() -> int { return 0; }",
            "function result types",
        ),
        (
            "repeat source",
            "fn bad() -> [int; 2] { return [1; 2]; } fn main() -> int { return 0; }",
            "array bindings without direct literal initializers",
        ),
        (
            "wrong result count",
            "fn bad() -> [int; 3] { return [1, 2]; } fn main() -> int { return 0; }",
            "array value source count mismatch",
        ),
        (
            "non-int computed element",
            "fn bad() -> [int; 1] { return [1 < 2]; } fn main() -> int { return 0; }",
            "array literal elements other than exact Int expressions",
        ),
        (
            "aggregate process result",
            "fn main() -> [int; 1] { return [1]; }",
            "entrypoints other than exact `fn main() -> int`",
        ),
    ];

    for (label, source, expected) in cases {
        let error = check_program(source, exact_options())
            .expect_err("excluded exact-array value topology must fail closed");
        assert!(
            error.starts_with("Language Profile Error: exact-i32-array-v0 rejects "),
            "{label} escaped profile admission: {error}"
        );
        assert!(
            error.contains(expected),
            "{label}: wrong diagnostic: {error}"
        );
        assert!(
            !error.contains("Semantic Analysis Error") && !error.contains("IR Generation Error")
        );
    }
}

#[test]
fn exact_profile_emits_guarded_mutable_i32_array_kernel() {
    let llvm = compile_program(FIXED_INT_ARRAY_PROGRAM, exact_options())
        .expect("exact fixed-array kernel should compile");

    for anchor in [
        "define i32 @dot_with_bias([8 x i32] %aero.arg.left, [8 x i32] %aero.arg.right, i32 %aero.arg.bias)",
        "define [8 x i32] @increment_each_lane([8 x i32] %aero.arg.values)",
        "define [8 x i32] @forward_array([8 x i32] %aero.arg.values)",
        "alloca [8 x i32], align 8",
        "store [8 x i32]",
        "load [8 x i32]",
        "call i32 @dot_with_bias([8 x i32]",
        "call [8 x i32] @increment_each_lane([8 x i32]",
        "call [8 x i32] @forward_array([8 x i32]",
        "ret [8 x i32]",
        "load i32",
        "store i32",
        "mul i32",
        "add i32",
        "icmp sge i32",
        "icmp slt i32",
        "sext i32",
        "getelementptr inbounds [8 x i32]",
        "declare void @llvm.trap()",
        "ret i32",
    ] {
        assert!(
            llvm.contains(anchor),
            "missing exact-array anchor `{anchor}`:\n{llvm}"
        );
    }
    for forbidden in [
        "[8 x double]",
        "double",
        "fptosi",
        "sitofp",
        " nsw ",
        " nuw ",
        "[8 x i8]",
        "<8 x i32>",
    ] {
        assert!(
            !llvm.contains(forbidden),
            "exact fixed-array LLVM leaked forbidden representation `{forbidden}`:\n{llvm}"
        );
    }

    assert_dynamic_guard_sequences(&llvm, "[8 x i32]", 4);
    assert_guarded_dynamic_array_reads_and_writes(&llvm, "[8 x i32]", 3, 1);
}

#[test]
fn exact_profile_executes_flat_row_major_matvec_product() {
    assert_eq!(
        FLAT_MATVEC_PRODUCT, EXPECTED_FLAT_MATVEC_PRODUCT,
        "tracked flat matvec source bytes drifted from the frozen product contract"
    );
    let ordinary_matrix = [1_i32, 2, 3, 4, 5, 6];
    let ordinary_vector = [7_i32, 8, 9];
    let wrapping_matrix = [i32::MAX, 0, 0, i32::MIN, -1, 2];
    let wrapping_vector = [2_i32, 1, 3];
    assert_eq!(
        reference_flat_matvec(ordinary_matrix, ordinary_vector),
        [50, 122]
    );
    assert_eq!(
        reference_flat_matvec(wrapping_matrix, wrapping_vector),
        [-2, 5]
    );
    assert_eq!(ordinary_matrix, [1, 2, 3, 4, 5, 6]);
    assert_eq!(ordinary_vector, [7, 8, 9]);
    assert_eq!(wrapping_matrix, [i32::MAX, 0, 0, i32::MIN, -1, 2]);
    assert_eq!(wrapping_vector, [2, 1, 3]);

    check_program(FLAT_MATVEC_PRODUCT, exact_options())
        .expect("flat matvec product should pass exact-profile checking");
    let first = compile_program(FLAT_MATVEC_PRODUCT, exact_options())
        .expect("flat matvec product should compile without production changes");
    let second = compile_program(FLAT_MATVEC_PRODUCT, exact_options())
        .expect("flat matvec product should compile deterministically");
    assert_eq!(first, second, "flat matvec LLVM must be deterministic");

    for anchor in [
        "define [2 x i32] @matvec_2x3([6 x i32] %aero.arg.matrix, [3 x i32] %aero.arg.vector)",
        "alloca [6 x i32], align 8",
        "alloca [3 x i32], align 8",
        "alloca [2 x i32], align 8",
        "store [6 x i32]",
        "store [3 x i32]",
        "store [2 x i32]",
        "load [2 x i32]",
        "ret [2 x i32]",
        "declare void @llvm.trap()",
    ] {
        assert!(
            first.contains(anchor),
            "matvec LLVM omitted `{anchor}`:\n{first}"
        );
    }
    assert_eq!(
        occurrences(&first, "call [2 x i32] @matvec_2x3([6 x i32]"),
        2,
        "main must execute the same flat matvec helper for ordinary and wrapping oracles"
    );
    for forbidden in [
        "double",
        "fptosi",
        "sitofp",
        " nsw ",
        " nuw ",
        "<2 x i32>",
        "<3 x i32>",
        "<6 x i32>",
        "[2 x [",
        "[3 x [",
        "[6 x [",
    ] {
        assert!(
            !first.contains(forbidden),
            "flat matvec leaked forbidden representation `{forbidden}`:\n{first}"
        );
    }

    let function = llvm_function_body(
        &first,
        "define [2 x i32] @matvec_2x3([6 x i32] %aero.arg.matrix, [3 x i32] %aero.arg.vector)",
    );
    let function_lines = function.lines().collect::<Vec<_>>();
    let multiplication = function_lines
        .iter()
        .filter(|line| line.contains(" = mul i32 ") && line.trim_end().ends_with(", 3"))
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(
        multiplication.len(),
        1,
        "matvec must contain exactly one row-times-three calculation:\n{function}"
    );
    let row_offset = ssa_definition(multiplication[0]);
    let row_value = ssa_rhs(function, row_offset)
        .strip_prefix("mul i32 ")
        .and_then(|rhs| rhs.strip_suffix(", 3"))
        .expect("row offset must multiply one SSA row value by three");
    let row_slot = loaded_i32_pointer(function, row_value);
    let additions = function_lines
        .iter()
        .filter(|line| line.contains(&format!(" = add i32 {row_offset}, ")))
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(
        additions.len(),
        1,
        "row offset must feed exactly one linear-index addition:\n{function}"
    );
    let linear_index = ssa_definition(additions[0]);
    let column_value = ssa_rhs(function, linear_index)
        .strip_prefix(&format!("add i32 {row_offset}, "))
        .expect("linear index must add a column SSA value to the row offset");
    let column_slot = loaded_i32_pointer(function, column_value);
    assert_ne!(
        row_slot, column_slot,
        "row and column loop identities must remain independent mutable slots"
    );
    assert_identity_linked_guard_consumer(function, linear_index, 6, "[6 x i32]", "load i32");

    let vector_index = guarded_index_for_bound(function, 3);
    assert_eq!(
        loaded_i32_pointer(function, &vector_index),
        column_slot,
        "the vector guard must reload the same loop-column slot used by linear indexing"
    );
    assert_identity_linked_guard_consumer(function, &vector_index, 3, "[3 x i32]", "load i32");
    let output_index = guarded_index_for_bound(function, 2);
    assert_eq!(
        loaded_i32_pointer(function, &output_index),
        row_slot,
        "the output guard must reload the same loop-row slot used by linear indexing"
    );
    assert_identity_linked_guard_consumer(function, &output_index, 2, "[2 x i32]", "store i32");
    assert_eq!(occurrences(function, "icmp sge i32"), 3);
    assert_eq!(occurrences(function, "call void @llvm.trap()"), 3);
    assert_eq!(occurrences(function, "sext i32"), 3);
    for anchor in [
        "store [6 x i32] %aero.arg.matrix",
        "store [3 x i32] %aero.arg.vector",
        "store [2 x i32]",
        "load [2 x i32]",
        "ret [2 x i32]",
    ] {
        assert!(
            function.contains(anchor),
            "matvec function omitted aggregate transport `{anchor}`:\n{function}"
        );
    }
    assert_guarded_dynamic_array_reads_and_writes(&first, "[6 x i32]", 1, 0);
    assert_guarded_dynamic_array_reads_and_writes(&first, "[3 x i32]", 1, 0);
    assert_guarded_dynamic_array_reads_and_writes(&first, "[2 x i32]", 0, 1);

    let workspace = TestWorkspace::new("flat-matvec-public-routes");
    let source = workspace.path("flat_matvec.aero");
    fs::write(&source, FLAT_MATVEC_PRODUCT).expect("write flat matvec product source");
    check_file(&source, exact_options()).expect("file library check should admit flat matvec");
    let file_llvm = compile_file(&source, exact_options())
        .expect("file library compile should admit flat matvec");
    assert_eq!(
        file_llvm, first,
        "source and file library matvec LLVM diverged"
    );
    for command in ["check", "build"] {
        let llvm = workspace.path("flat_matvec.ll");
        let arguments = if command == "build" {
            vec![
                Path::new(command),
                &source,
                Path::new("-o"),
                &llvm,
                Path::new("--require-llvm-verifier"),
                Path::new("--language-profile"),
                Path::new("exact-i32-array-v0"),
            ]
        } else {
            vec![
                Path::new(command),
                &source,
                Path::new("--language-profile"),
                Path::new("exact-i32-array-v0"),
            ]
        };
        let output = run_cli(&workspace, &arguments);
        assert!(
            output.status.success(),
            "public {command} rejected flat matvec product:\n{}",
            combined_output(&output)
        );
        if command == "build" {
            let public = fs::read_to_string(&llvm).expect("public matvec LLVM artifact");
            assert_eq!(
                llvm_body_without_public_route_headers(&public),
                llvm_body_without_public_route_headers(&first),
                "public and library matvec LLVM bodies diverged"
            );
        }
    }
    let output = run_cli(
        &workspace,
        &[
            Path::new("run"),
            &source,
            Path::new("--language-profile"),
            Path::new("exact-i32-array-v0"),
        ],
    );
    assert_eq!(
        output.status.code(),
        Some(91),
        "public matvec run diverged:\n{}",
        combined_output(&output)
    );
    let public_output = combined_output(&output);
    assert_eq!(occurrences(&public_output, "Exit code: 91"), 1);
}

#[test]
fn exact_profile_executes_guarded_tensor_record_two_stage_scoring_product() {
    assert_eq!(
        TENSOR_RECORD_SCORING, EXPECTED_TENSOR_RECORD_SCORING,
        "tracked tensor-record scoring source bytes drifted from the frozen CAP-021 contract"
    );

    let ordinary = reference_tensor_record(ORDINARY_TENSOR_RECORD);
    assert_eq!(ordinary.first_products, [28, 40, 54, 40, 55, 72]);
    assert_eq!(ordinary.raw, [122, 167]);
    assert_eq!(ordinary.hidden, [135, 181]);
    assert_eq!(ordinary.score_products, [2025, 2896]);
    assert_eq!(ordinary.result, [1, 122, 167, 135, 181, 4938]);

    let wrapping = reference_tensor_record(WRAPPING_TENSOR_RECORD);
    assert_eq!(wrapping.first_products, [-2, -12, -10, 0, 3, 15]);
    assert_eq!(wrapping.raw, [-24, 18]);
    assert_eq!(wrapping.hidden, [2_147_483_623, -2_147_483_631]);
    assert_eq!(wrapping.score_products, [25, 2_147_483_631]);
    assert_eq!(
        wrapping.result,
        [1, -24, 18, 2_147_483_623, -2_147_483_631, -2_147_483_627]
    );

    let malformed = reference_tensor_record(MALFORMED_TENSOR_RECORD);
    assert_eq!(malformed.first_products, [0; 6]);
    assert_eq!(malformed.raw, [0; 2]);
    assert_eq!(malformed.hidden, [0; 2]);
    assert_eq!(malformed.score_products, [0; 2]);
    assert_eq!(malformed.result, [0; 6]);
    assert_eq!(
        ORDINARY_TENSOR_RECORD,
        [2, 3, 1, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
        "ordinary source record changed while computing its by-value oracle"
    );
    assert_eq!(
        WRAPPING_TENSOR_RECORD,
        [
            2,
            3,
            1,
            2,
            -3,
            5,
            i32::MAX,
            4,
            -2,
            i32::MIN,
            -1,
            3,
            i32::MAX,
            i32::MAX,
            i32::MAX,
            -1,
            13,
        ],
        "wrapping source record changed while computing its by-value oracle"
    );

    let mut bad_header = ORDINARY_TENSOR_RECORD;
    bad_header[1] = 4;
    assert_eq!(
        reference_tensor_record(bad_header).result,
        [0; 6],
        "malformed header must reject before payload scoring"
    );
    let mut changed_payload = ORDINARY_TENSOR_RECORD;
    changed_payload[11] = 13;
    assert_ne!(
        reference_tensor_record(changed_payload).result,
        ordinary.result,
        "payload corruption must change the independent scoring oracle"
    );

    let tokens = try_tokenize_with_locations(TENSOR_RECORD_SCORING, None)
        .expect("tensor-record scoring product should lex");
    let ast = parse_with_locations(tokens).expect("tensor-record scoring product should parse");
    let raw_checked = IrGenerator::new()
        .try_generate_ir(ast.clone())
        .expect("independent checked-IR verification should admit the raw product AST");
    let mut analyzer = SemanticAnalyzer::new();
    let (_, analyzed) = analyzer
        .analyze(ast)
        .expect("tensor-record scoring product should pass semantic analysis");
    let analyzed_checked = IrGenerator::new()
        .try_generate_ir(analyzed)
        .expect("independent checked-IR verification should admit the analyzed product");
    let exact_array = |count| LogicalType::Array {
        element: Box::new(LogicalType::Int),
        count,
    };
    for checked in [&raw_checked, &analyzed_checked] {
        let metadata = checked.metadata();
        let matvec = &metadata.functions["matvec_2x3"];
        assert_eq!(
            matvec.signature.parameters,
            vec![
                ("matrix".to_string(), exact_array(6)),
                ("vector".to_string(), exact_array(3)),
            ]
        );
        assert_eq!(matvec.signature.result, exact_array(2));
        let affine = &metadata.functions["affine_2_to_1"];
        assert_eq!(
            affine.signature.parameters,
            vec![
                ("values".to_string(), exact_array(2)),
                ("weights".to_string(), exact_array(2)),
                ("bias".to_string(), LogicalType::Int),
            ]
        );
        assert_eq!(affine.signature.result, LogicalType::Int);
        let decode = &metadata.functions["decode_and_score"];
        assert_eq!(
            decode.signature.parameters,
            vec![("record".to_string(), exact_array(17))]
        );
        assert_eq!(decode.signature.result, exact_array(6));
        let compare = &metadata.functions["records_equal"];
        assert_eq!(
            compare.signature.parameters,
            vec![
                ("left".to_string(), exact_array(17)),
                ("right".to_string(), exact_array(17)),
            ]
        );
        assert_eq!(compare.signature.result, LogicalType::Int);
        let checked_debug = format!("{checked:#?}");
        for marker in [
            "CheckedMutableOwnedPlaceAlloca",
            "CheckedOwnedPlaceAssignment",
            "count: 17",
            "count: 6",
            "count: 3",
            "count: 2",
            "count: 1",
        ] {
            assert!(
                checked_debug.contains(marker),
                "checked record-to-score IR omitted `{marker}`:\n{checked_debug}"
            );
        }
    }

    check_program(TENSOR_RECORD_SCORING, exact_options())
        .expect("tensor-record scoring product should pass selected-profile checking");
    let first = compile_program(TENSOR_RECORD_SCORING, exact_options())
        .expect("tensor-record scoring product should compile without production changes");
    let second = compile_program(TENSOR_RECORD_SCORING, exact_options())
        .expect("tensor-record scoring product should compile deterministically");
    assert_eq!(
        first, second,
        "tensor-record scoring LLVM must be deterministic"
    );

    let matvec_signature =
        "define [2 x i32] @matvec_2x3([6 x i32] %aero.arg.matrix, [3 x i32] %aero.arg.vector)";
    let affine_signature = "define i32 @affine_2_to_1([2 x i32] %aero.arg.values, [2 x i32] %aero.arg.weights, i32 %aero.arg.bias)";
    let decode_signature = "define [6 x i32] @decode_and_score([17 x i32] %aero.arg.record)";
    let compare_signature =
        "define i32 @records_equal([17 x i32] %aero.arg.left, [17 x i32] %aero.arg.right)";
    for anchor in [
        matvec_signature,
        affine_signature,
        decode_signature,
        compare_signature,
        "ret [2 x i32]",
        "ret [6 x i32]",
        "declare void @llvm.trap()",
    ] {
        assert!(
            first.contains(anchor),
            "scoring LLVM omitted `{anchor}`:\n{first}"
        );
    }
    assert_eq!(
        occurrences(&first, "call [6 x i32] @decode_and_score([17 x i32]"),
        3,
        "main must score ordinary, wrapping, and malformed-header records"
    );
    assert_eq!(
        occurrences(&first, "call i32 @records_equal([17 x i32]"),
        2,
        "main must reread all 17 lanes of both valid source records"
    );

    let decode = llvm_function_body(&first, decode_signature);
    assert_eq!(
        occurrences(decode, "call [2 x i32] @matvec_2x3([6 x i32]"),
        1,
        "decoder must compose exactly one accepted matvec"
    );
    assert_eq!(
        occurrences(decode, "call i32 @affine_2_to_1([2 x i32]"),
        1,
        "decoder must compose exactly one genuinely second affine stage"
    );

    let decode_blocks = llvm_blocks(decode);
    let record_base = exact_argument_array_base(decode, "[17 x i32]", "record");
    let record_accesses = dynamic_i32_accesses(decode, "[17 x i32]");
    assert_eq!(
        record_accesses.len(),
        6,
        "record decoder needs header plus five exact payload source loops"
    );
    let mut decoded_accesses = Vec::new();
    for aggregate in ["[3 x i32]", "[6 x i32]", "[2 x i32]", "[1 x i32]"] {
        decoded_accesses.extend(dynamic_i32_accesses(decode, aggregate));
    }
    let record_source = |offset: i32| {
        let matches = record_accesses
            .iter()
            .filter(|access| {
                let rhs = ssa_rhs(decode, &access.index);
                if offset == 0 {
                    rhs.starts_with("load i32, i32* ") && rhs.ends_with(", align 4")
                } else {
                    rhs.strip_prefix(&format!("add i32 {offset}, "))
                        .is_some_and(|raw| {
                            ssa_rhs(decode, raw).starts_with("load i32, i32* ")
                                && ssa_rhs(decode, raw).ends_with(", align 4")
                        })
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(
            matches.len(),
            1,
            "record offset {offset} must identify exactly one decode source"
        );
        (*matches[0]).clone()
    };
    let decode_specs = [
        ("header", "[3 x i32]", 3, 0),
        ("input", "[3 x i32]", 3, 3),
        ("first_weights", "[6 x i32]", 6, 6),
        ("first_bias", "[2 x i32]", 2, 12),
        ("score_weights", "[2 x i32]", 2, 14),
        ("score_bias", "[1 x i32]", 1, 16),
    ];
    let decode_bindings = decode_specs
        .into_iter()
        .map(|(name, aggregate, width, offset)| {
            assert_decode_loop_binding(
                decode,
                &decode_blocks,
                &record_base,
                name,
                aggregate,
                width,
                offset,
                &record_source(offset),
                &decoded_accesses,
            )
        })
        .collect::<Vec<_>>();
    let mut destination_bases = decode_bindings
        .iter()
        .map(|binding| binding.destination.base.as_str())
        .collect::<Vec<_>>();
    destination_bases.sort_unstable();
    destination_bases.dedup();
    assert_eq!(
        destination_bases.len(),
        6,
        "all six decoded fields must own distinct destination locals"
    );
    assert_tensor_header_gate(
        decode,
        &decode_blocks,
        &decode_bindings[0],
        &decode_bindings[1..],
    );

    let input_value =
        exact_aggregate_load(decode, "[3 x i32]", &decode_bindings[1].destination.base);
    let first_weights_value =
        exact_aggregate_load(decode, "[6 x i32]", &decode_bindings[2].destination.base);
    let matvec_call_rhs = format!(
        "call [2 x i32] @matvec_2x3([6 x i32] {first_weights_value}, [3 x i32] {input_value})"
    );
    let matvec_result = ssa_value_for_rhs(decode, &matvec_call_rhs);
    let matvec_result_store_prefix = format!("store [2 x i32] {matvec_result}, [2 x i32]* ");
    let matvec_result_bases = decode
        .lines()
        .filter_map(|line| line.trim().strip_prefix(&matvec_result_store_prefix))
        .filter_map(|line| line.strip_suffix(", align 8"))
        .collect::<Vec<_>>();
    assert_eq!(matvec_result_bases.len(), 1);

    let two_accesses = dynamic_i32_accesses(decode, "[2 x i32]");
    let first_bias_reads = two_accesses
        .iter()
        .filter(|access| {
            access.base == decode_bindings[3].destination.base && access.consumer == "load i32"
        })
        .collect::<Vec<_>>();
    assert_eq!(
        first_bias_reads.len(),
        1,
        "first-bias decoded local must feed one guarded hidden read"
    );
    let raw_reads = two_accesses
        .iter()
        .filter(|access| access.base == matvec_result_bases[0] && access.consumer == "load i32")
        .collect::<Vec<_>>();
    assert_eq!(
        raw_reads.len(),
        1,
        "accepted matvec result must feed one guarded hidden read"
    );
    let hidden_sum = ssa_value_for_rhs(
        decode,
        &format!(
            "add i32 {}, {}",
            raw_reads[0].value, first_bias_reads[0].value
        ),
    );
    let hidden_destinations = two_accesses
        .iter()
        .filter(|access| access.consumer == "store i32" && access.value == hidden_sum)
        .collect::<Vec<_>>();
    assert_eq!(
        hidden_destinations.len(),
        1,
        "hidden add must feed one guarded hidden destination store"
    );
    for access in [raw_reads[0], first_bias_reads[0]] {
        assert_identity_linked_guard_consumer(decode, &access.index, 2, "[2 x i32]", "load i32");
    }
    assert_identity_linked_guard_consumer(
        decode,
        &hidden_destinations[0].index,
        2,
        "[2 x i32]",
        "store i32",
    );
    assert_eq!(
        loaded_i32_pointer(decode, &raw_reads[0].index),
        loaded_i32_pointer(decode, &first_bias_reads[0].index)
    );
    assert_eq!(
        loaded_i32_pointer(decode, &raw_reads[0].index),
        loaded_i32_pointer(decode, &hidden_destinations[0].index),
        "hidden read/add/store chain must preserve the exact loop-index identity"
    );
    let hidden_value = exact_aggregate_load(decode, "[2 x i32]", &hidden_destinations[0].base);
    let score_weights_value =
        exact_aggregate_load(decode, "[2 x i32]", &decode_bindings[4].destination.base);
    let score_bias_value =
        exact_static_i32_load(decode, "[1 x i32]", &decode_bindings[5].destination.base, 0);
    let affine_call_rhs = format!(
        "call i32 @affine_2_to_1([2 x i32] {hidden_value}, [2 x i32] {score_weights_value}, i32 {score_bias_value})"
    );
    let affine_score = ssa_value_for_rhs(decode, &affine_call_rhs);
    let score_slots = decode
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix(&format!("store i32 {affine_score}, i32* "))
                .and_then(|rhs| rhs.strip_suffix(", align 4"))
        })
        .collect::<Vec<_>>();
    assert_eq!(score_slots.len(), 1);
    let final_score = ssa_value_for_rhs(
        decode,
        &format!("load i32, i32* {}, align 4", score_slots[0]),
    );
    assert_result_array_contract(
        decode,
        &decode_blocks,
        &decode_bindings[0],
        matvec_result_bases[0],
        &hidden_destinations[0].base,
        &final_score,
    );

    let affine = llvm_function_body(&first, affine_signature);
    assert_affine_accumulator_contract(affine);

    let matvec = llvm_function_body(&first, matvec_signature);
    let matvec_matrix = dynamic_i32_accesses(matvec, "[6 x i32]");
    assert_identity_linked_dynamic_accesses(matvec, "[6 x i32]", 6, &["load i32"]);
    assert_identity_linked_dynamic_accesses(matvec, "[3 x i32]", 3, &["load i32"]);
    assert_identity_linked_dynamic_accesses(matvec, "[2 x i32]", 2, &["store i32"]);
    let row_multiply = matvec
        .lines()
        .filter(|line| line.contains(" = mul i32 ") && line.trim_end().ends_with(", 3"))
        .collect::<Vec<_>>();
    assert_eq!(row_multiply.len(), 1);
    let row_offset = ssa_definition(row_multiply[0]);
    let linear = matvec
        .lines()
        .filter(|line| line.contains(&format!(" = add i32 {row_offset}, ")))
        .collect::<Vec<_>>();
    assert_eq!(linear.len(), 1);
    assert_eq!(
        matvec_matrix[0].index,
        ssa_definition(linear[0]),
        "row-times-three plus column must feed the guarded matrix read"
    );

    let compare = llvm_function_body(&first, compare_signature);
    assert_records_equal_contract(compare);
    let main = llvm_function_body(&first, "define i32 @main()");
    assert_main_source_preservation(main);

    for forbidden in [
        "double",
        "fptosi",
        "sitofp",
        " nsw ",
        " nuw ",
        "<2 x i32>",
        "<3 x i32>",
        "<6 x i32>",
        "<17 x i32>",
        "[17 x [",
        "extractelement",
        "insertelement",
    ] {
        assert!(
            !first.contains(forbidden),
            "tensor-record product leaked forbidden representation `{forbidden}`:\n{first}"
        );
    }

    let workspace = TestWorkspace::new("tensor-record-scoring-public-routes");
    let source = workspace.path("tensor_record_scoring.aero");
    fs::write(&source, TENSOR_RECORD_SCORING).expect("write tensor-record scoring source");
    check_file(&source, exact_options())
        .expect("file library check should admit tensor-record scoring");
    let file_llvm = compile_file(&source, exact_options())
        .expect("file library compile should admit tensor-record scoring");
    assert_eq!(file_llvm, first, "source and file scorer LLVM diverged");
    for command in ["check", "build"] {
        let llvm = workspace.path("tensor_record_scoring.ll");
        let arguments = if command == "build" {
            vec![
                Path::new(command),
                &source,
                Path::new("-o"),
                &llvm,
                Path::new("--require-llvm-verifier"),
                Path::new("--language-profile"),
                Path::new("exact-i32-array-v0"),
            ]
        } else {
            vec![
                Path::new(command),
                &source,
                Path::new("--language-profile"),
                Path::new("exact-i32-array-v0"),
            ]
        };
        let output = run_cli(&workspace, &arguments);
        assert!(
            output.status.success(),
            "public {command} rejected tensor-record scoring:\n{}",
            combined_output(&output)
        );
        if command == "build" {
            let public = fs::read_to_string(&llvm).expect("public scorer LLVM artifact");
            assert_eq!(
                llvm_body_without_public_route_headers(&public),
                llvm_body_without_public_route_headers(&first),
                "public and library scorer LLVM bodies diverged"
            );
        }
    }
    let output = run_cli(
        &workspace,
        &[
            Path::new("run"),
            &source,
            Path::new("--language-profile"),
            Path::new("exact-i32-array-v0"),
        ],
    );
    assert_eq!(
        output.status.code(),
        Some(91),
        "public tensor-record scoring run diverged:\n{}",
        combined_output(&output)
    );
    assert_eq!(
        occurrences(&combined_output(&output), "Exit code: 91"),
        1,
        "public scorer route must report the exact sentinel once"
    );
}

#[test]
fn exact_array_kernel_and_wrapping_edges_match_independent_i32_oracles() {
    assert_eq!(reference_kernel(), 2035);

    let values = [i32::MAX, 1, 1_073_741_824, 2];
    let wrapped_add = values[0].wrapping_add(values[1]);
    let wrapped_sub = wrapped_add.wrapping_sub(values[1]);
    let wrapped_mul = values[2].wrapping_mul(values[3]);
    let wrapped_neg = wrapped_add.wrapping_neg();
    assert_eq!(
        if wrapped_add < 0 && wrapped_sub > 0 && wrapped_mul < 0 && wrapped_neg < 0 {
            93
        } else {
            1
        },
        93
    );

    for source in [FIXED_INT_ARRAY_PROGRAM, FIXED_INT_ARRAY_WRAPPING_EDGES] {
        let llvm = compile_program(source, exact_options())
            .expect("exact fixed-array arithmetic specimen should compile");
        for forbidden in ["double", "fptosi", "sitofp", " nsw ", " nuw "] {
            assert!(
                !llvm.contains(forbidden),
                "wrapping proof leaked `{forbidden}`"
            );
        }
    }

    let wrapping_llvm = compile_program(FIXED_INT_ARRAY_WRAPPING_EDGES, exact_options())
        .expect("constant-index wrapping specimen should compile");
    assert!(
        wrapping_llvm.contains("icmp slt i32"),
        "wrapping control flow must retain its ordinary signed comparison"
    );
    assert_dynamic_guard_sequences(&wrapping_llvm, "[4 x i32]", 0);
}

#[test]
fn exact_profile_rejects_every_neighboring_array_family_before_checked_ir() {
    let cases = [
        (
            "repeat array",
            "fn main() -> int { let values: [int; 2] = [1; 2]; return 0; }",
            "array bindings without direct literal initializers",
        ),
        (
            "empty array",
            "fn main() -> int { let values: [int; 0] = []; return 0; }",
            "binding annotation types",
        ),
        (
            "array count above the signed i32 profile boundary",
            "fn take(values: [int; 2147483648]) -> int { return 0; } fn main() -> int { return 0; }",
            "function parameter types",
        ),
        (
            "nested array",
            "fn main() -> int { let values: [[int; 1]; 1] = [[1]]; return 0; }",
            "binding annotation types",
        ),
        (
            "non-int element",
            "fn main() -> int { let values: [bool; 1] = [true]; return 0; }",
            "binding annotation types",
        ),
        (
            "char element",
            "fn main() -> int { let values: [char; 1] = ['a']; return 0; }",
            "binding annotation types",
        ),
        (
            "user-defined element",
            "fn take(values: [Widget; 1]) -> int { return 0; } fn main() -> int { return 0; }",
            "function parameter types",
        ),
        (
            "array write",
            "fn main() -> int { let values: [int; 1] = [1]; values[0] = 2; return 0; }",
            "projected assignment targets rooted in immutable exact-array bindings",
        ),
        (
            "array comparison",
            "fn main() -> int { let values: [int; 1] = [1]; if values == values { return 1; } return 0; }",
            "array identifiers outside direct call transport or index reads",
        ),
        (
            "wrong literal count",
            "fn main() -> int { let values: [int; 1] = [1, 2]; return 0; }",
            "array literal counts that differ from their annotations",
        ),
        (
            "wrong transport count",
            "fn take(values: [int; 2]) -> int { return values[0]; } fn main() -> int { let values: [int; 1] = [1]; return take(values); }",
            "array call arguments with mismatched counts",
        ),
        (
            "out-of-range lane literal",
            "fn main() -> int { let values: [int; 1] = [2147483648]; return 0; }",
            "array elements other than exact signed i32 literals",
        ),
    ];

    for (name, source, expected) in cases {
        let error = match check_program(source, exact_options()) {
            Ok(()) => panic!("{name} unexpectedly entered exact checked IR"),
            Err(error) => error,
        };
        assert!(
            error.starts_with("Language Profile Error: exact-i32-array-v0 rejects "),
            "{name} escaped pre-semantic profile admission: {error}"
        );
        assert!(
            error.contains(expected),
            "{name}: wrong diagnostic: {error}"
        );
        assert!(
            !error.contains("Semantic Analysis Error") && !error.contains("IR Generation Error")
        );
    }
}

#[test]
fn exact_profile_preserves_compile_time_bounds_rejection_and_runtime_guards() {
    for (label, index) in [("negative", "-1"), ("equal", "2"), ("above", "3")] {
        let source = format!(
            "fn main() -> int {{ let values: [int; 2] = [10, 20]; return values[{index}]; }}"
        );
        let error = compile_program(&source, exact_options())
            .expect_err("constant out-of-bounds exact array index must reject");
        assert!(error.contains("outside 0..2"), "{label}: {error}");
    }

    for (label, source) in [
        ("negative read", NEGATIVE_RUNTIME_INDEX),
        ("equal-to-count read", EQUAL_TO_COUNT_RUNTIME_INDEX),
    ] {
        check_program(source, exact_options())
            .unwrap_or_else(|error| panic!("{label} runtime trap specimen should check: {error}"));
        let llvm = compile_program(source, exact_options())
            .expect("runtime bounds-failure specimen should lower to a guard");
        assert_dynamic_guard_sequences(&llvm, "[2 x i32]", 1);
        assert_guarded_dynamic_array_reads_and_writes(&llvm, "[2 x i32]", 1, 0);
    }

    for (label, source) in [
        ("negative write", NEGATIVE_RUNTIME_WRITE_INDEX),
        ("equal-to-count write", EQUAL_TO_COUNT_RUNTIME_WRITE_INDEX),
    ] {
        check_program(source, exact_options())
            .unwrap_or_else(|error| panic!("{label} runtime trap specimen should check: {error}"));
        let llvm = compile_program(source, exact_options()).unwrap_or_else(|error| {
            panic!("{label} runtime trap specimen should compile: {error}")
        });
        assert_dynamic_guard_sequences(&llvm, "[2 x i32]", 1);
        assert_guarded_dynamic_array_reads_and_writes(&llvm, "[2 x i32]", 0, 1);
        assert_single_dynamic_array_store_value(&llvm, "[2 x i32]", 9);
    }
}

#[test]
fn exact_i32_array_system_gate_is_anchored_on_linux_and_windows() {
    let workflow = include_str!("../../../.github/workflows/rust.yml");
    let linux = workflow_named_step(
        workflow,
        "Test exact i32 fixed-array CPU profile at O0 and O2",
    );
    let windows = workflow_named_step(
        workflow,
        "Test exact i32 fixed-array CPU profile on Windows at O0 and O2",
    );
    let expected_bounds = [
        "negative_index.aero",
        "equal_to_count_index.aero",
        "negative_write_index.aero",
        "equal_to_count_write_index.aero",
    ];
    assert_eq!(linux_bounds_loop_names(linux), expected_bounds.to_vec());
    assert_eq!(windows_bounds_loop_names(windows), expected_bounds.to_vec());

    let required_product_evidence: [(&str, &str, &[&str]); 2] = [
        (
            "Linux",
            linux,
            &[
                "matvec:flat_matvec.aero:91:yes",
                "if [ \"${name}\" = matvec ]; then",
                "matvec_llvm=\"$(awk '",
                "matvec_identity_pattern='(?ms)^",
                "matvec_identity_count=\"$(grep -Pzo -- \"${matvec_identity_pattern}\"",
                "test \"${matvec_identity_count}\" -eq 1",
                "matvec_guard_pattern='(?m)^",
                "matvec_guard_count=\"$(grep -Pzo -- \"${matvec_guard_pattern}\"",
                "test \"${matvec_guard_count}\" -eq 3",
            ],
        ),
        (
            "Windows",
            windows,
            &[
                "[pscustomobject]@{ Name = \"matvec\"; File = \"flat_matvec.aero\"; Expected = 91; Dynamic = $true }",
                "if ($specimen.Name -ceq \"matvec\") {",
                "$matvecFunctionPattern = '(?ms)^define",
                "$matvecFunctionMatches = [regex]::Matches($llvmText, $matvecFunctionPattern)",
                "$matvecFunctionMatches.Count -ne 1",
                "$matvecText = $matvecFunctionMatches[0].Value",
                "$matvecIdentityPattern = '(?ms)^",
                "$matvecIdentityMatches = [regex]::Matches($matvecText, $matvecIdentityPattern)",
                "$matvecIdentityMatches.Count -ne 1",
                "$matvecGuardPattern = '(?m)^",
                "$matvecGuardMatches = [regex]::Matches($matvecText, $matvecGuardPattern)",
                "$matvecGuardMatches.Count -ne 3",
            ],
        ),
    ];
    let mut missing_product_evidence = Vec::new();
    for (os, step, anchors) in required_product_evidence {
        for &anchor in anchors {
            if !step.contains(anchor) {
                missing_product_evidence.push(format!("{os}: {anchor}"));
            }
        }
    }
    for (os, step) in [("Linux", linux), ("Windows", windows)] {
        for anchor in [
            "define [2 x i32] @matvec_2x3([6 x i32] %aero.arg.matrix, [3 x i32] %aero.arg.vector)",
            "call [2 x i32] @matvec_2x3([6 x i32]",
            "(?<matvec_row_times_three>%reg[0-9]+) = mul i32 \\k<matvec_row>, 3",
            "(?<matvec_linear>%reg[0-9]+) = add i32 \\k<matvec_row_times_three>, \\k<matvec_column>",
            "icmp slt i32 \\k<matvec_linear>, 6",
            "sext i32 \\k<matvec_linear> to i64",
            "load i32, i32\\* \\k<matvec_matrix_target>, align 4",
            "load i32, i32\\* \\k<matvec_vector_target>, align 4",
            "store i32 [^,\\r\\n]+, i32\\* \\k<matvec_output_target>, align 4",
            "(?<matvec_guard_bound>[236])",
            "\\[\\k<matvec_guard_bound> x i32\\]",
        ] {
            if !step.contains(anchor) {
                missing_product_evidence.push(format!("{os}: {anchor}"));
            }
        }
    }
    for (os, step) in [("Linux", linux), ("Windows", windows)] {
        for anchor in [
            "examples/fixed_int_array_v0/",
            "wrapping_edges.aero",
            "runtime_fail/",
            "--language-profile exact-i32-array-v0",
            "--require-llvm-verifier",
            "define [8 x i32] @increment_each_lane([8 x i32] %aero.arg.values)",
            "define [8 x i32] @forward_array([8 x i32] %aero.arg.values)",
            "call [8 x i32] @increment_each_lane([8 x i32]",
            "call [8 x i32] @forward_array([8 x i32]",
            "ret [8 x i32]",
        ] {
            assert!(step.contains(anchor), "{os} exact step omitted `{anchor}`");
        }
        for bounds_name in expected_bounds {
            assert_eq!(
                occurrences(step, bounds_name),
                1,
                "{os} exact step must run `{bounds_name}` exactly once"
            );
        }
        for guard_identity in [
            r"\k<index>, [0-9]+",
            r"\k<lower>, \k<upper>",
            r"br i1 \k<inbounds>",
            r"%aero\.bounds\.trap\.\k<place>",
            r"sext i32 \k<index> to i64",
            r"\k<aggregate>\* %ptr",
            r"i64 \k<extended>\r?$",
        ] {
            assert!(
                step.contains(guard_identity),
                "{os} exact step lost guard identity link `{guard_identity}`"
            );
        }
        assert!(
            !step.contains("icmp sge i32|icmp slt i32|sext i32|llvm\\.trap"),
            "{os} exact step must not mistake unlinked IR fragments for a bounds proof"
        );
    }

    for (anchor, expected) in [
        ("opt-22 -passes=verify", 2),
        ("llc-22 -verify-machineinstrs", 2),
        ("clang-22 -O0", 1),
        ("clang-22 -O2", 1),
        (
            "test \"$(printf '%s\\n' \"${public_output}\" | grep -Fxc \"Exit code: ${expected}\")\" -eq 1",
            1,
        ),
        ("if [ \"${name}\" = kernel ]; then", 1),
        (
            "kernel_mutation_pattern='(?ms)^  (?<kernel_lower>%reg[0-9]+) = icmp sge i32",
            1,
        ),
        (
            "(?<kernel_target>%ptr[0-9]+) = getelementptr inbounds \\[8 x i32\\]",
            1,
        ),
        ("i32\\* \\k<kernel_target>, align 4", 1),
        (
            r"(?:(?!^\}).)*?^  store i32 [^,\r\n]+, i32\* \k<kernel_target>, align 4\r?$",
            1,
        ),
        (
            "kernel_mutation_count=\"$(grep -Pzo -- \"${kernel_mutation_pattern}\" \"${llvm}\" | tr -cd '\\000' | wc -c)\"",
            1,
        ),
        ("test \"${kernel_mutation_count}\" -eq 1", 1),
        (
            "guard_block_pattern='(?m)^  (?<lower>%reg[0-9]+) = icmp sge i32",
            1,
        ),
        (
            "guard_count=\"$(grep -Pzo -- \"${guard_block_pattern}\" \"${llvm}\" | tr -cd '\\000' | wc -c)\"",
            1,
        ),
        ("test \"${guard_count}\" -eq 4", 1),
        ("if [[ \"${bounds_case}\" == *_write_index ]]; then", 1),
        (
            "bounds_write_pattern='(?m)^  (?<bounds_write_lower>%reg[0-9]+) = icmp sge i32",
            1,
        ),
        (
            "(?<bounds_write_target>%ptr[0-9]+) = getelementptr inbounds \\[2 x i32\\]",
            1,
        ),
        ("store i32 9, i32\\* \\k<bounds_write_target>, align 4", 1),
        (
            "bounds_write_count=\"$(grep -Pzo -- \"${bounds_write_pattern}\" \"${bounds_llvm}\" | tr -cd '\\000' | wc -c)\"",
            1,
        ),
        ("test \"${bounds_write_count}\" -eq 1", 1),
    ] {
        assert_eq!(
            occurrences(linux, anchor),
            expected,
            "Linux exact step must contain `{anchor}` exactly {expected} time(s)"
        );
    }
    assert!(
        !linux.contains("test \"${guard_count}\" -eq 2"),
        "Linux exact step retained the former two-guard kernel contract"
    );
    assert!(
        !linux
            .contains("upper_line=\"$(grep -n -m1 -F 'icmp slt i32' \"${llvm}\" | cut -d: -f1)\""),
        "Linux exact step must identity-link its lower and upper guard predicates"
    );

    for (anchor, expected) in [
        ("& \"$llvmBin\\opt.exe\" -passes=verify", 2),
        ("& \"$llvmBin\\llc.exe\" -verify-machineinstrs", 2),
        ("& \"$llvmBin\\clang.exe\" -O0", 1),
        ("& \"$llvmBin\\clang.exe\" -O2", 1),
        ("if ($specimen.Name -ceq \"kernel\") {", 1),
        (
            "$kernelMutationPattern = '(?ms)^  (?<kernel_lower>%reg[0-9]+) = icmp sge i32",
            1,
        ),
        (
            "(?<kernel_target>%ptr[0-9]+) = getelementptr inbounds \\[8 x i32\\]",
            1,
        ),
        ("i32\\* \\k<kernel_target>, align 4", 1),
        (
            r"(?:(?!^\}).)*?^  store i32 [^,\r\n]+, i32\* \k<kernel_target>, align 4\r?$",
            1,
        ),
        (
            "$kernelMutationMatches = [regex]::Matches($llvmText, $kernelMutationPattern)",
            1,
        ),
        ("$kernelMutationMatches.Count -ne 1", 1),
        (
            "$guardPattern = '(?m)^  (?<lower>%reg[0-9]+) = icmp sge i32",
            1,
        ),
        (
            "$guardMatches = [regex]::Matches($llvmText, $guardPattern)",
            1,
        ),
        ("$guardMatches.Count -ne 4", 1),
        (
            "$boundsCase.EndsWith(\"_write_index\", [System.StringComparison]::Ordinal)",
            1,
        ),
        (
            "$boundsWritePattern = '(?m)^  (?<bounds_write_lower>%reg[0-9]+) = icmp sge i32",
            1,
        ),
        (
            "(?<bounds_write_target>%ptr[0-9]+) = getelementptr inbounds \\[2 x i32\\]",
            1,
        ),
        ("store i32 9, i32\\* \\k<bounds_write_target>, align 4", 1),
        (
            "$boundsWriteMatches = [regex]::Matches($boundsText, $boundsWritePattern)",
            1,
        ),
        ("$boundsWriteMatches.Count -ne 1", 1),
    ] {
        assert_eq!(
            occurrences(windows, anchor),
            expected,
            "Windows exact step must contain `{anchor}` exactly {expected} time(s)"
        );
    }
    assert!(
        !windows.contains("$guardMatches.Count -ne 2"),
        "Windows exact step retained the former two-guard kernel contract"
    );
    assert!(
        missing_product_evidence.is_empty(),
        "CAP-020 red: existing OS steps lack flat-matvec product evidence: {missing_product_evidence:#?}"
    );

    let required_scoring_evidence: [(&str, &str, &[&str]); 2] = [
        (
            "Linux",
            linux,
            &[
                "scorer:tensor_record_scoring.aero:91:yes",
                "if [ \"${name}\" = scorer ]; then",
                "scorer_second_llvm=\"${RUNNER_TEMP}/exact_i32_array_scorer.linux.second.ll\"",
                "build \"${source}\" -o \"${scorer_second_llvm}\" --require-llvm-verifier",
                "cmp -s \"${llvm}\" \"${scorer_second_llvm}\"",
                "llvm-as-22 \"${llvm}\" -o /dev/null",
                "scorer_decode_llvm=\"$(awk '",
                "scorer_affine_llvm=\"$(awk '",
                "scorer_matvec_llvm=\"$(awk '",
                "scorer_decode_pattern='(?ms)^",
                "scorer_decode_count=\"$(grep -Pzo -- \"${scorer_decode_pattern}\" <(printf '%s\\n' \"${scorer_decode_llvm}\")",
                "test \"${scorer_decode_count}\" -eq 1",
                "scorer_decode_chain_pattern='(?ms)^",
                "scorer_decode_chain_count=\"$(grep -Pzo -- \"${scorer_decode_chain_pattern}\" <(printf '%s\\n' \"${scorer_decode_llvm}\")",
                "test \"${scorer_decode_chain_count}\" -eq 6",
                "scorer_hidden_pattern='(?ms)^",
                "scorer_hidden_count=\"$(grep -Pzo -- \"${scorer_hidden_pattern}\" <(printf '%s\\n' \"${scorer_decode_llvm}\")",
                "test \"${scorer_hidden_count}\" -eq 1",
                "scorer_affine_pattern='(?ms)^",
                "scorer_affine_count=\"$(grep -Pzo -- \"${scorer_affine_pattern}\" <(printf '%s\\n' \"${scorer_affine_llvm}\")",
                "test \"${scorer_affine_count}\" -eq 1",
                "scorer_matvec_pattern='(?ms)^",
                "scorer_matvec_count=\"$(grep -Pzo -- \"${scorer_matvec_pattern}\" <(printf '%s\\n' \"${scorer_matvec_llvm}\")",
                "test \"${scorer_matvec_count}\" -eq 1",
                "scorer_guard_pattern='(?m)^",
                "scorer_guard_count=\"$(grep -Pzo -- \"${scorer_guard_pattern}\" \"${llvm}\"",
                "test \"${scorer_guard_count}\" -eq 15",
            ],
        ),
        (
            "Windows",
            windows,
            &[
                "[pscustomobject]@{ Name = \"scorer\"; File = \"tensor_record_scoring.aero\"; Expected = 91; Dynamic = $true }",
                "if ($specimen.Name -ceq \"scorer\") {",
                "$scorerSecondLlvm = Join-Path $env:RUNNER_TEMP \"exact_i32_array_scorer.windows.second.ll\"",
                "build $source -o $scorerSecondLlvm --require-llvm-verifier",
                "[System.Linq.Enumerable]::SequenceEqual([IO.File]::ReadAllBytes($llvm), [IO.File]::ReadAllBytes($scorerSecondLlvm))",
                "& \"$llvmBin\\llvm-as.exe\" $llvm -o $null",
                "$scorerDecodeFunctionPattern = '(?ms)^define",
                "$scorerDecodeFunctionMatches = [regex]::Matches($llvmText, $scorerDecodeFunctionPattern)",
                "$scorerDecodeFunctionMatches.Count -ne 1",
                "$scorerDecodeText = $scorerDecodeFunctionMatches[0].Value",
                "$scorerAffineFunctionPattern = '(?ms)^define",
                "$scorerAffineFunctionMatches = [regex]::Matches($llvmText, $scorerAffineFunctionPattern)",
                "$scorerAffineFunctionMatches.Count -ne 1",
                "$scorerAffineText = $scorerAffineFunctionMatches[0].Value",
                "$scorerMatvecFunctionPattern = '(?ms)^define",
                "$scorerMatvecFunctionMatches = [regex]::Matches($llvmText, $scorerMatvecFunctionPattern)",
                "$scorerMatvecFunctionMatches.Count -ne 1",
                "$scorerMatvecText = $scorerMatvecFunctionMatches[0].Value",
                "$scorerDecodeChainPattern = '(?ms)^",
                "$scorerDecodeChainMatches = [regex]::Matches($scorerDecodeText, $scorerDecodeChainPattern)",
                "$scorerDecodeChainMatches.Count -ne 6",
                "$scorerHiddenPattern = '(?ms)^",
                "$scorerHiddenMatches = [regex]::Matches($scorerDecodeText, $scorerHiddenPattern)",
                "$scorerHiddenMatches.Count -ne 1",
                "$scorerAffinePattern = '(?ms)^",
                "$scorerAffineMatches = [regex]::Matches($scorerAffineText, $scorerAffinePattern)",
                "$scorerAffineMatches.Count -ne 1",
                "$scorerMatvecPattern = '(?ms)^",
                "$scorerMatvecMatches = [regex]::Matches($scorerMatvecText, $scorerMatvecPattern)",
                "$scorerMatvecMatches.Count -ne 1",
                "$scorerGuardPattern = '(?m)^",
                "$scorerGuardMatches = [regex]::Matches($llvmText, $scorerGuardPattern)",
                "$scorerGuardMatches.Count -ne 15",
            ],
        ),
    ];
    let shared_scoring_anchors = [
        "define [6 x i32] @decode_and_score([17 x i32] %aero.arg.record)",
        "define i32 @affine_2_to_1([2 x i32] %aero.arg.values, [2 x i32] %aero.arg.weights, i32 %aero.arg.bias)",
        "call [2 x i32] @matvec_2x3([6 x i32]",
        "call i32 @affine_2_to_1([2 x i32]",
        "(?<scorer_decode_destination>%ptr[0-9]+) = getelementptr inbounds",
        "(?<scorer_record_index>%reg[0-9]+) = add i32",
        "icmp slt i32 \\k<scorer_record_index>, 17",
        "(?<scorer_record_target>%ptr[0-9]+) = getelementptr inbounds \\[17 x i32\\]",
        "(?<scorer_record_value>%reg[0-9]+) = load i32, i32\\* \\k<scorer_record_target>, align 4",
        "store i32 \\k<scorer_record_value>, i32\\* \\k<scorer_decode_destination>, align 4",
        "(?<scorer_hidden_raw>%reg[0-9]+) = load i32",
        "(?<scorer_hidden_bias>%reg[0-9]+) = load i32",
        "(?<scorer_hidden_sum>%reg[0-9]+) = add i32 \\k<scorer_hidden_raw>, \\k<scorer_hidden_bias>",
        "store i32 \\k<scorer_hidden_sum>",
        "(?<scorer_affine_value>%reg[0-9]+) = load i32",
        "(?<scorer_affine_weight>%reg[0-9]+) = load i32",
        "(?<scorer_affine_product>%reg[0-9]+) = mul i32 \\k<scorer_affine_value>, \\k<scorer_affine_weight>",
        "(?<scorer_affine_accumulator>%reg[0-9]+) = load i32",
        "add i32 \\k<scorer_affine_accumulator>, \\k<scorer_affine_product>",
        "(?<scorer_matvec_row>%reg[0-9]+) = load i32",
        "(?<scorer_matvec_column>%reg[0-9]+) = load i32",
        "mul i32 \\k<scorer_matvec_row>, 3",
        "icmp slt i32 \\k<scorer_matvec_column>, 3",
        "load i32, i32\\* \\k<scorer_record_target>, align 4",
        "ret [6 x i32]",
    ];
    let mut missing_scoring_evidence = Vec::new();
    for (os, step, anchors) in required_scoring_evidence {
        for &anchor in anchors {
            if !step.contains(anchor) {
                missing_scoring_evidence.push(format!("{os}: {anchor}"));
            }
        }
        for anchor in shared_scoring_anchors {
            if !step.contains(anchor) {
                missing_scoring_evidence.push(format!("{os}: {anchor}"));
            }
        }
    }
    assert!(
        missing_scoring_evidence.is_empty(),
        "CAP-021 intentional red: existing Linux and Windows exact-profile steps lack the tensor-record scorer descriptor and structural/native/public evidence: {missing_scoring_evidence:#?}"
    );
}
