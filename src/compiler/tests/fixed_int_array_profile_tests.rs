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
}
