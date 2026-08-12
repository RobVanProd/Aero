use compiler::{
    CheckedIr, CodeGenerator, CompilerOptions, IrGenerator, LanguageProfile, LogicalType,
    SemanticAnalyzer, compile_file, compile_program, parse_with_locations,
    try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_DIRECTORY: &str = "examples/representative_telemetry";
const WORKFLOW: &str = ".github/workflows/rust.yml";

const MAIN_SOURCE: &str = r#"mod model;
mod policy;
mod records;

fn main() -> int {
    const BASE: int = 10 + 1;
    const EXPECTED: int = 91;

    let parsed_record: Result<int, char> = parse_record(['T', '=', '1', '7', ';', 'H', '=', '0', '8', ';']);
    let parsed_value = result_or(parsed_record, -100);
    let parser_contract = parsed_value == 42
        && result_or(parse_record(['T', '=', '0', '0', ';', 'H', '=', '0', '0', ';']), -1) == 0
        && result_or(parse_record(['T', '=', '9', '9', ';', 'H', '=', '9', '9', ';']), -1) == 297
        && result_has_error(parse_record(['X', '=', '1', '7', ';', 'H', '=', '0', '8', ';']), 'X')
        && result_has_error(parse_record(['T', ':', '1', '7', ';', 'H', '=', '0', '8', ';']), ':')
        && result_has_error(parse_record(['T', '=', 'x', '7', ';', 'H', '=', '0', '8', ';']), 'x')
        && result_has_error(parse_record(['T', '=', '1', 'x', ';', 'H', '=', '0', '8', ';']), 'x')
        && result_has_error(parse_record(['T', '=', '1', '7', ':', 'H', '=', '0', '8', ';']), ':')
        && result_has_error(parse_record(['T', '=', '1', '7', ';', 'J', '=', '0', '8', ';']), 'J')
        && result_has_error(parse_record(['T', '=', '1', '7', ';', 'H', ':', '0', '8', ';']), ':')
        && result_has_error(parse_record(['T', '=', '1', '7', ';', 'H', '=', 'x', '8', ';']), 'x')
        && result_has_error(parse_record(['T', '=', '1', '7', ';', 'H', '=', '0', 'x', ';']), 'x')
        && result_has_error(parse_record(['T', '=', '1', '7', ';', 'H', '=', '0', '8', ':']), ':')
        && result_has_error(parse_record(['X', '=', 'x', '7', ';', 'H', '=', '0', '8', ';']), 'X')
        && result_has_error(parse_record(['T', '=', 'x', 'y', ';', 'H', '=', '0', '8', ';']), 'x')
        && result_has_error(parse_record(['T', '=', '1', 'x', ':', 'H', '=', '0', '8', ';']), 'x');
    if !parser_contract { return 2; }

    let mut batch: Batch = make_batch();
    let calibration_seed: Window<i32> = Window { values: [0, 8, parsed_value] };
    let calibration: Window<int> = window_set(calibration_seed, 0, BASE);
    let mut sensor_index = 0;
    while sensor_index < 3 {
        batch.sensors[sensor_index].value = window_get(calibration, sensor_index);
        sensor_index = sensor_index + 1;
    }
    let metadata = replace_int(&mut batch.meta.0, 4);
    batch.meta.1 = 2 > 1;

    let first_sensor_index = 0;
    let observed = read_int(&batch.sensors[first_sensor_index].value);
    let observed_ref = &observed;
    let bias = read_policy_value(batch, 0);
    let mut total = batch.sensors[1].value + batch.sensors[2].value;
    total = add_bias(&mut total, observed_ref, bias);

    for weight in [1, 2, 3] {
        total = total + weight;
    }
    let mut step = 0;
    while step < 2 {
        total = total + step + 1;
        step = step + 1;
    }
    let delta_reading: Reading<int> = choose(make_reading(4, 3 < 4), make_reading(8, 4 < 3), 4 < 5);
    let marker_seed: Window<char> = Window { values: ['a', 'x', 'c'] };
    let marker_window: Window<char> = window_set(marker_seed, 1, 'b');
    let first_marker: Reading<char> = Reading { value: window_get(marker_window, 0), valid: 4 < 5 };
    let second_marker: Reading<char> = Reading { value: window_get(marker_window, 1), valid: 5 < 4 };
    let marker: Reading<char> = choose(first_marker, second_marker, 5 < 6);
    let marker_valid = marker.valid;
    let delta_sample: Sample<Reading<int>> = Sample::Present(delta_reading);
    let marker_sample: Sample<char> = Sample::Present(marker.value);
    let accepted_delta: Result<int, char> = validate_delta(sample_reading_value(delta_sample), sample_marker_is_a(marker_sample, marker_valid));
    let rejected_delta: Result<int, char> = validate_delta(8, 4 < 3);
    total = total + resolved_delta(accepted_delta) + resolved_delta(rejected_delta) + metadata - 4;
    if batch.meta.1 {
        total = total + 12;
    } else {
        total = 1;
    }

    let trusted = batch.sensors[first_sensor_index].trusted;
    let decision = classify(total, trusted);
    let score = decision_score(decision);
    println!("telemetry score: {}", score);
    if score == EXPECTED { return EXPECTED; }
    1
}
"#;

const MODEL_SOURCE: &str = r#"struct Sensor { value: int, trusted: bool }
struct Batch { sensors: [Sensor; 3], meta: (int, bool) }
struct Reading<T> { value: T, valid: bool }
struct Window<T> { values: [T; 3] }

enum Sample<T> {
    Present(T),
    Missing
}

enum Decision {
    Normal(int),
    Alert(int, bool)
}

fn choose<T>(first: T, second: T, take_first: bool) -> T {
    if take_first { return first; }
    second
}

fn window_get<T>(window: Window<T>, index: int) -> T {
    window.values[index]
}

fn window_set<T>(window: Window<T>, index: int, value: T) -> Window<T> {
    let mut updated: Window<T> = window;
    updated.values[index] = value;
    updated
}

fn make_sensor(value: int, trusted: bool) -> Sensor {
    Sensor { value: value, trusted: trusted }
}

fn make_reading(value: int, valid: bool) -> Reading<int> {
    Reading { value: value, valid: valid }
}

fn reading_value(reading: Reading<int>) -> int {
    reading.value
}

fn make_batch() -> Batch {
    let seed = make_sensor(0, 1 < 2);
    Batch {
        sensors: [seed, seed, seed],
        meta: (0, 1 > 2)
    }
}
"#;

const POLICY_SOURCE: &str = r#"trait PolicyValue {
    fn policy_value(&self, bias: int) -> int;
}

impl PolicyValue for Sensor {
    fn policy_value(&self, bias: i32) -> int {
        (*self).value + bias
    }
}

impl PolicyValue for Batch {
    fn policy_value(&self, bias: int) -> int {
        (*self).meta.0 + bias
    }
}

fn read_policy_value<T: PolicyValue>(value: T, bias: int) -> int {
    value.policy_value(bias)
}

fn add_bias(target: &mut int, observed: &int, bias: int) -> int {
    *target = *target + *observed + bias;
    *target
}

fn read_int(value: &int) -> int {
    *value
}

fn replace_int(target: &mut int, replacement: int) -> int {
    *target = replacement;
    *target
}

fn classify(total: int, trusted: bool) -> Decision {
    if trusted && total > 50 { return Decision::Alert(total, 1 < 2); }
    Decision::Normal(total)
}

fn add_urgent(urgent: bool, value: int) -> int {
    if urgent { return value + 1; }
    value
}

fn validate_delta(value: int, valid: bool) -> Result<int, char> {
    if valid { return Ok(value); }
    Err('e')
}

fn sample_reading_value(sample: Sample<Reading<int>>) -> int {
    match sample {
        Sample::Present(reading) => reading_value(reading),
        Sample::Missing => 0
    }
}

fn sample_marker_is_a(sample: Sample<char>, valid: bool) -> bool {
    match sample {
        Sample::Present(marker) => valid && marker == 'a',
        _ => 1 > 2
    }
}

fn resolved_delta(result: Result<int, char>) -> int {
    match result {
        Ok(value) => value,
        Err(_) => 0
    }
}

fn decision_score(decision: Decision) -> int {
    match decision {
        Decision::Normal(value) => value,
        Decision::Alert(value, urgent) => add_urgent(urgent, value)
    }
}
"#;

const RECORDS_SOURCE: &str = r#"fn decimal_digit(character: char) -> Result<int, char> {
    if character == '0' { return Ok(0); }
    if character == '1' { return Ok(1); }
    if character == '2' { return Ok(2); }
    if character == '3' { return Ok(3); }
    if character == '4' { return Ok(4); }
    if character == '5' { return Ok(5); }
    if character == '6' { return Ok(6); }
    if character == '7' { return Ok(7); }
    if character == '8' { return Ok(8); }
    if character == '9' { return Ok(9); }
    Err(character)
}

fn parse_record(record: [char; 10]) -> Result<int, char> {
    let mut index = 0;
    let first_marker = record[index];
    if first_marker != 'T' { return Err(first_marker); }

    index = index + 1;
    let first_equals = record[index];
    if first_equals != '=' { return Err(first_equals); }

    index = index + 1;
    let first_tens_character = record[index];
    let first_tens_result: Result<int, char> = decimal_digit(first_tens_character);
    let first_tens = match first_tens_result {
        Ok(digit) => digit,
        Err(character) => -1
    };
    if first_tens < 0 { return Err(first_tens_character); }

    index = index + 1;
    let first_ones_character = record[index];
    let first_ones_result: Result<int, char> = decimal_digit(first_ones_character);
    let first_ones = match first_ones_result {
        Ok(digit) => digit,
        Err(character) => -1
    };
    if first_ones < 0 { return Err(first_ones_character); }
    let temperature = first_tens * 10 + first_ones;

    index = index + 1;
    let first_separator = record[index];
    if first_separator != ';' { return Err(first_separator); }

    index = index + 1;
    let second_marker = record[index];
    if second_marker != 'H' { return Err(second_marker); }

    index = index + 1;
    let second_equals = record[index];
    if second_equals != '=' { return Err(second_equals); }

    index = index + 1;
    let second_tens_character = record[index];
    let second_tens_result: Result<int, char> = decimal_digit(second_tens_character);
    let second_tens = match second_tens_result {
        Ok(digit) => digit,
        Err(character) => -1
    };
    if second_tens < 0 { return Err(second_tens_character); }

    index = index + 1;
    let second_ones_character = record[index];
    let second_ones_result: Result<int, char> = decimal_digit(second_ones_character);
    let second_ones = match second_ones_result {
        Ok(digit) => digit,
        Err(character) => -1
    };
    if second_ones < 0 { return Err(second_ones_character); }
    let humidity = second_tens * 10 + second_ones;

    index = index + 1;
    let final_separator = record[index];
    if final_separator != ';' { return Err(final_separator); }

    Ok(temperature * 2 + humidity)
}

fn result_or(result: Result<int, char>, fallback: int) -> int {
    match result {
        Ok(value) => value,
        Err(character) => fallback
    }
}

fn result_has_error(result: Result<int, char>, expected: char) -> bool {
    match result {
        Ok(value) => 1 > 2,
        Err(character) => character == expected
    }
}
"#;

const NUMERIC_PRINT_ABI_SOURCE: &str = r#"fn main() -> int {
    let integer = 7;
    let floating = 2.5;
    let truth = integer > 3;
    print!("{} {} {}", integer, floating, truth);
    println!(" {}", 4);
    0
}
"#;

const IMMUTABLE_PROJECTED_UPDATE_SOURCE: &str = r#"struct Sensor { value: int, trusted: bool }

fn main() -> int {
    let sensor = Sensor { value: 1, trusted: 1 < 2 };
    sensor.value = 2;
    sensor.value
}
"#;

const UNKNOWN_PROJECTED_FIELD_SOURCE: &str = r#"struct Sensor { value: int, trusted: bool }

fn main() -> int {
    let mut sensor = Sensor { value: 1, trusted: 1 < 2 };
    sensor.missing = 2;
    sensor.value
}
"#;

const WRONG_POLICY_ARGUMENT_SOURCE: &str = r#"enum Decision { Normal(int), Alert(int, bool) }

fn classify(total: int, trusted: bool) -> Decision {
    if trusted { return Decision::Alert(total, trusted); }
    Decision::Normal(total)
}

fn main() -> int {
    let decision = classify(1, 2);
    match decision {
        Decision::Normal(value) => value,
        Decision::Alert(value, trusted) => value
    }
}
"#;

const UNSATISFIED_TRAIT_BOUND_SOURCE: &str = r#"struct Reading { value: int }
struct Other { value: int }

trait Score {
    fn score(&self) -> int;
}

impl Score for Reading {
    fn score(&self) -> int {
        (*self).value
    }
}

fn evaluate<T: Score>(value: T) -> int {
    value.score()
}

fn main() -> int {
    evaluate(Other { value: 7 })
}
"#;

const NEGATIVE_INDEX_SOURCE: &str = r#"fn main() -> int {
    let values = [10, 20];
    let zero = 0;
    let index = zero - 1;
    let selected = values[index];
    println!("unreachable negative index: {}", selected);
    0
}
"#;

const UPPER_BOUND_INDEX_SOURCE: &str = r#"fn main() -> int {
    let values = [10, 20];
    let count = 2;
    let index = count;
    let selected = values[index];
    println!("unreachable upper-bound index: {}", selected);
    0
}
"#;

const PARSER_NEGATIVE_INDEX_SOURCE: &str = r#"fn read_record_character(record: [char; 10], index: int) -> char {
    record[index]
}

fn main() -> int {
    let record: [char; 10] = ['T', '=', '1', '7', ';', 'H', '=', '0', '8', ';'];
    let zero = 0;
    let index = zero - 1;
    let selected = read_record_character(record, index);
    let selected_is_marker = selected == 'T';
    println!("unreachable parser negative index");
    if selected_is_marker { return 1; }
    0
}
"#;

const PARSER_EQUAL_TO_COUNT_INDEX_SOURCE: &str = r#"fn read_record_character(record: [char; 10], index: int) -> char {
    record[index]
}

fn main() -> int {
    let record: [char; 10] = ['T', '=', '1', '7', ';', 'H', '=', '0', '8', ';'];
    let count = 10;
    let selected = read_record_character(record, count);
    let selected_is_marker = selected == 'T';
    println!("unreachable parser equal-to-count index");
    if selected_is_marker { return 1; }
    0
}
"#;

const PROJECTED_NEGATIVE_INDEX_SOURCE: &str = r#"struct Reading { value: int }

fn observe(value: &int) -> int {
    *value
}

fn main() -> int {
    let values = [Reading { value: 10 }, Reading { value: 20 }];
    let zero = 0;
    let index = zero - 1;
    let selected = observe(&values[index].value);
    println!("unreachable projected negative index: {}", selected);
    0
}
"#;

const PROJECTED_UPPER_BOUND_INDEX_SOURCE: &str = r#"struct Reading { value: int }

fn replace(value: &mut int, replacement: int) -> int {
    *value = replacement;
    *value
}

fn main() -> int {
    let mut values = [Reading { value: 10 }, Reading { value: 20 }];
    let count = 2;
    let index = count;
    let selected = replace(&mut values[index].value, 41);
    println!("unreachable projected upper-bound index: {}", selected);
    0
}
"#;

const NEGATIVE_WRITE_INDEX_SOURCE: &str = r#"fn replacement() -> int {
    println!("unreachable assignment rhs");
    41
}

fn main() -> int {
    let mut values = [10, 20];
    let zero = 0;
    let index = zero - 1;
    values[index] = replacement();
    println!("unreachable negative write");
    0
}
"#;

const UPPER_BOUND_WRITE_INDEX_SOURCE: &str = r#"fn replacement() -> int {
    println!("unreachable assignment rhs");
    41
}

fn main() -> int {
    let mut values = [10, 20];
    let count = 2;
    let index = count;
    values[index] = replacement();
    println!("unreachable upper-bound write");
    0
}
"#;

const GENERIC_NEGATIVE_INDEX_SOURCE: &str = r#"struct Window<T> { values: [T; 2] }

fn window_get<T>(window: Window<T>, index: int) -> T {
    window.values[index]
}

fn main() -> int {
    let values: Window<int> = Window { values: [10, 20] };
    let zero = 0;
    let index = zero - 1;
    let selected = window_get(values, index);
    println!("unreachable generic negative index: {}", selected);
    0
}
"#;

const GENERIC_UPPER_BOUND_INDEX_SOURCE: &str = r#"struct Window<T> { values: [T; 2] }

fn window_get<T>(window: Window<T>, index: int) -> T {
    window.values[index]
}

fn main() -> int {
    let values: Window<int> = Window { values: [10, 20] };
    let count = 2;
    let index = count;
    let selected = window_get(values, index);
    println!("unreachable generic upper-bound index: {}", selected);
    0
}
"#;

const GENERIC_NEGATIVE_WRITE_INDEX_SOURCE: &str = r#"struct Window<T> { values: [T; 2] }

fn window_get<T>(window: Window<T>, index: int) -> T {
    window.values[index]
}

fn window_set<T>(window: Window<T>, index: int, value: T) -> Window<T> {
    let mut updated: Window<T> = window;
    updated.values[index] = value;
    updated
}

fn main() -> int {
    let values: Window<int> = Window { values: [10, 20] };
    let zero = 0;
    let index = zero - 1;
    let updated = window_set(values, index, 41);
    println!("unreachable generic negative write: {}", window_get(updated, 0));
    0
}
"#;

const GENERIC_UPPER_BOUND_WRITE_INDEX_SOURCE: &str = r#"struct Window<T> { values: [T; 2] }

fn window_get<T>(window: Window<T>, index: int) -> T {
    window.values[index]
}

fn window_set<T>(window: Window<T>, index: int, value: T) -> Window<T> {
    let mut updated: Window<T> = window;
    updated.values[index] = value;
    updated
}

fn main() -> int {
    let values: Window<int> = Window { values: [10, 20] };
    let count = 2;
    let index = count;
    let updated = window_set(values, index, 41);
    println!("unreachable generic upper-bound write: {}", window_get(updated, 0));
    0
}
"#;

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before Unix epoch")
            .as_nanos();
        let serial = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "aero-representative-telemetry-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create representative application workspace");
        Self { root }
    }

    fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create representative fixture directory");
        }
        fs::write(&path, contents).expect("write representative fixture");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let expected = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("aero-representative-telemetry-"));
        if self.root.starts_with(std::env::temp_dir()) && expected {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("compiler crate must be nested below repository root")
        .to_path_buf()
}

fn flattened_source() -> String {
    let root_without_modules = MAIN_SOURCE
        .strip_prefix("mod model;\nmod policy;\nmod records;\n\n")
        .expect("representative root keeps the frozen direct-module prefix");
    format!("{MODEL_SOURCE}\n{POLICY_SOURCE}\n{RECORDS_SOURCE}\n{root_without_modules}")
}

fn run_aero(workspace: &TestWorkspace, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aero"))
        .current_dir(&workspace.root)
        .args(arguments)
        .output()
        .expect("run Aero public CLI")
}

fn output_text(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn occurrences(text: &str, needle: &str) -> usize {
    text.match_indices(needle).count()
}

fn llvm_function_body<'a>(llvm: &'a str, signature: &str) -> Option<&'a str> {
    llvm.split(signature)
        .nth(1)
        .and_then(|rest| rest.split("\n}").next())
}

fn decimal_digit_failure(llvm: &str) -> Option<String> {
    let Some(body) = llvm_function_body(llvm, "define { i32, double, i32 } @decimal_digit") else {
        return Some("representative LLVM omitted the decimal_digit body".to_string());
    };

    let equality_lines = body
        .lines()
        .map(str::trim)
        .filter(|line| line.contains(" = icmp eq i32 "))
        .collect::<Vec<_>>();
    if equality_lines.len() != 10 {
        return Some(format!(
            "decimal_digit has {} equality classifiers, expected 10",
            equality_lines.len()
        ));
    }
    for ascii in 48..=57 {
        let suffix = format!(", {ascii}");
        let actual = equality_lines
            .iter()
            .filter(|line| line.ends_with(&suffix))
            .count();
        if actual != 1 {
            return Some(format!(
                "decimal_digit classified ASCII {ascii} {actual} times, expected once"
            ));
        }
    }
    for forbidden in [
        " sitofp ",
        " uitofp ",
        " fptosi ",
        " fptoui ",
        " zext ",
        " sext ",
        " trunc ",
        " bitcast ",
        " ptrtoint ",
        " inttoptr ",
    ] {
        if body.contains(forbidden) {
            return Some(format!(
                "decimal_digit introduced forbidden character conversion {forbidden:?}"
            ));
        }
    }
    None
}

fn parser_guard_failure(llvm: &str) -> Option<String> {
    let Some(body) = llvm_function_body(llvm, "define { i32, double, i32 } @parse_record") else {
        return Some("representative LLVM omitted the parse_record body".to_string());
    };

    for (anchor, expected) in [
        ("fcmp oge double", 10),
        ("fcmp olt double", 10),
        ("call void @llvm.trap()", 10),
        ("getelementptr inbounds [10 x i32]", 10),
    ] {
        let actual = occurrences(body, anchor);
        if actual != expected {
            return Some(format!(
                "parse_record has {actual} occurrences of {anchor:?}, expected {expected}"
            ));
        }
    }

    let lines = body.lines().map(str::trim).collect::<Vec<_>>();
    let lower_lines = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.contains(" = fcmp oge double "))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if lower_lines.len() != 10 {
        return Some(format!(
            "parse_record has {} lower-bound sequence starts, expected 10",
            lower_lines.len()
        ));
    }

    for (ordinal, lower_line) in lower_lines.into_iter().enumerate() {
        let sequence = lines.get(lower_line..lower_line + 10).unwrap_or(&[]);
        if sequence.len() != 10 {
            return Some(format!(
                "dynamic parser read {ordinal} has a truncated guard"
            ));
        }
        let Some((lower_result, lower_rhs)) = sequence[0].split_once(" = fcmp oge double ") else {
            return Some(format!(
                "dynamic parser read {ordinal} malformed its lower guard"
            ));
        };
        let Some((index_value, lower_bound)) = lower_rhs.split_once(',') else {
            return Some(format!(
                "dynamic parser read {ordinal} lost its lower operand"
            ));
        };
        if lower_bound.trim() != "0x0000000000000000" {
            return Some(format!(
                "dynamic parser read {ordinal} used lower bound {lower_bound:?}, expected zero"
            ));
        }
        let Some((upper_result, upper_rhs)) = sequence[1].split_once(" = fcmp olt double ") else {
            return Some(format!(
                "dynamic parser read {ordinal} malformed its upper guard"
            ));
        };
        let Some((upper_index, upper_bound)) = upper_rhs.split_once(',') else {
            return Some(format!(
                "dynamic parser read {ordinal} lost its upper operand"
            ));
        };
        if upper_index != index_value {
            return Some(format!(
                "dynamic parser read {ordinal} compared different lower/upper indexes"
            ));
        }
        if upper_bound.trim() != "0x4024000000000000" {
            return Some(format!(
                "dynamic parser read {ordinal} used upper bound {upper_bound:?}, expected ten"
            ));
        }
        let Some((conjunction, conjunction_rhs)) = sequence[2].split_once(" = and i1 ") else {
            return Some(format!(
                "dynamic parser read {ordinal} omitted its conjunction"
            ));
        };
        if conjunction_rhs != format!("{lower_result}, {upper_result}") {
            return Some(format!(
                "dynamic parser read {ordinal} did not combine its exact bound predicates"
            ));
        }
        let Some(branch_rhs) = sequence[3].strip_prefix("br i1 ") else {
            return Some(format!(
                "dynamic parser read {ordinal} omitted its guard branch"
            ));
        };
        let expected_branch_prefix = format!("{conjunction}, label %");
        let Some(labels) = branch_rhs.strip_prefix(&expected_branch_prefix) else {
            return Some(format!(
                "dynamic parser read {ordinal} did not branch on its exact conjunction"
            ));
        };
        let Some((safe_label, trap_label)) = labels.split_once(", label %") else {
            return Some(format!(
                "dynamic parser read {ordinal} malformed its branch labels"
            ));
        };
        let Some(place) = safe_label.strip_prefix("aero.bounds.safe.") else {
            return Some(format!(
                "dynamic parser read {ordinal} used noncanonical safe label {safe_label:?}"
            ));
        };
        if trap_label != format!("aero.bounds.trap.{place}") {
            return Some(format!(
                "dynamic parser read {ordinal} did not bind safe/trap labels to one place"
            ));
        }
        if sequence[4] != format!("{trap_label}:")
            || sequence[5] != "call void @llvm.trap()"
            || sequence[6] != "unreachable"
            || sequence[7] != format!("{safe_label}:")
        {
            return Some(format!(
                "dynamic parser read {ordinal} did not bind its trap/safe branch labels"
            ));
        }
        let Some((conversion, conversion_rhs)) = sequence[8].split_once(" = fptosi double ") else {
            return Some(format!(
                "dynamic parser read {ordinal} omitted its guarded index conversion"
            ));
        };
        if conversion_rhs != format!("{index_value} to i64") {
            return Some(format!(
                "dynamic parser read {ordinal} converted a different index value"
            ));
        }
        if !sequence[9].contains("getelementptr inbounds [10 x i32]")
            || !sequence[9].ends_with(&format!("i64 {conversion}"))
        {
            return Some(format!(
                "dynamic parser read {ordinal} did not feed its guarded index into the character GEP"
            ));
        }
    }
    None
}

fn parser_metadata_failure(checked: &CheckedIr) -> Option<String> {
    let result_schema_is_exact = |ty: &LogicalType| {
        let LogicalType::Enum { variants, .. } = ty else {
            return false;
        };
        variants.len() == 2
            && variants[0].name == "Ok"
            && variants[0].payload == Some(LogicalType::Int)
            && variants[1].name == "Err"
            && variants[1].payload == Some(LogicalType::Char)
    };
    let Some(digit) = checked.metadata().functions.get("decimal_digit") else {
        return Some("checked metadata omitted decimal_digit".to_string());
    };
    if !matches!(&digit.signature.parameters[..], [(_, LogicalType::Char)])
        || !result_schema_is_exact(&digit.signature.result)
    {
        return Some(format!(
            "decimal_digit checked signature lost char -> Result<int, char>: {:?}",
            digit.signature
        ));
    }
    let Some(function) = checked.metadata().functions.get("parse_record") else {
        return Some("checked metadata omitted parse_record".to_string());
    };
    match &function.signature.parameters[..] {
        [(_, LogicalType::Array { element, count })]
            if **element == LogicalType::Char && *count == 10 => {}
        actual => {
            return Some(format!(
                "parse_record checked parameter lost Array<Char; 10>: {actual:?}"
            ));
        }
    }
    if !result_schema_is_exact(&function.signature.result) {
        return Some(format!(
            "parse_record checked Result schema changed: {:?}",
            function.signature.result
        ));
    }
    None
}

fn contains_regular_file(path: &Path) -> bool {
    if path.is_file() {
        return true;
    }
    path.is_dir()
        && fs::read_dir(path)
            .expect("read artifact directory")
            .any(|entry| contains_regular_file(&entry.expect("read artifact entry").path()))
}

#[test]
fn representative_scalar_application_is_composed_and_portable() {
    let mut failures = Vec::new();
    let workspace = TestWorkspace::new();
    let root = workspace.write("main.aero", MAIN_SOURCE);
    workspace.write("model.aero", MODEL_SOURCE);
    workspace.write("policy.aero", POLICY_SOURCE);
    workspace.write("records.aero", RECORDS_SOURCE);

    let first_llvm = match compile_file(&root, CompilerOptions::default()) {
        Ok(llvm) => Some(llvm),
        Err(error) => {
            failures.push(format!(
                "representative direct-module application did not compile: {error}"
            ));
            None
        }
    };
    let second_llvm = compile_file(&root, CompilerOptions::default());
    if let (Some(first), Ok(second)) = (&first_llvm, second_llvm) {
        if first != &second {
            failures.push("representative file compilation was not deterministic".to_string());
        }
        for anchor in [
            "define i32 @main()",
            "define i32 @add_bias(",
            "define i32 @read_int(",
            "define i32 @replace_int(",
            "define i32 @decision_score(",
            "define i32 @reading_value(",
            "%\"aero.struct.Reading<int>\" = type",
            "%\"aero.struct.Reading<char>\" = type",
            "aero.generic.choose<Reading<int>>",
            "aero.generic.choose<Reading<char>>",
            "aero.generic.window_get<int>",
            "aero.generic.window_get<char>",
            "aero.generic.window_set<int>",
            "aero.generic.window_set<char>",
            "aero.trait.PolicyValue.for.Sensor.policy_value",
            "aero.trait.PolicyValue.for.Batch.policy_value",
            "; Aero generic enum: Sample<Reading<int>>",
            "; Aero generic enum: Sample<char>",
            "define i32 @sample_reading_value(",
            "define i1 @sample_marker_is_a(",
            "define { i32, double, i32 } @decimal_digit(i32 %aero.arg.character)",
            "define { i32, double, i32 } @parse_record([10 x i32] %aero.arg.record)",
            "call { i32, double, i32 } @parse_record([10 x i32]",
            "getelementptr inbounds [10 x i32]",
            "telemetry score: %g",
            "declare void @llvm.trap()",
            "fcmp oge double",
            "fcmp olt double",
        ] {
            if !first.contains(anchor) {
                failures.push(format!("representative LLVM omitted anchor {anchor:?}"));
            }
        }
        for forbidden in [
            "Window<i32>",
            "aero.generic.window_get<i32>",
            "aero.generic.window_set<i32>",
            "[10 x double]",
            "sitofp i32 %aero.arg.character",
        ] {
            if first.contains(forbidden) {
                failures.push(format!(
                    "representative LLVM retained split alias identity {forbidden:?}"
                ));
            }
        }
        if let Some(failure) = parser_guard_failure(first) {
            failures.push(failure);
        }
        if let Some(failure) = decimal_digit_failure(first) {
            failures.push(failure);
        }
    }

    let flattened = flattened_source();
    let ast = try_tokenize_with_locations(&flattened, Some("representative.aero".to_string()))
        .map_err(|error| error.to_string())
        .and_then(|tokens| parse_with_locations(tokens).map_err(|error| error.to_string()));
    match ast {
        Err(error) => failures.push(format!("representative flattened source rejected: {error}")),
        Ok(ast) => {
            let semantic_llvm = match SemanticAnalyzer::new().analyze(ast.clone()) {
                Err(error) => {
                    failures.push(format!(
                        "representative semantic analysis rejected: {error}"
                    ));
                    None
                }
                Ok((_, analyzed)) => match IrGenerator::new().try_generate_ir(analyzed) {
                    Err(error) => {
                        failures.push(format!(
                            "representative semantic checked admission rejected: {error}"
                        ));
                        None
                    }
                    Ok(checked) => {
                        if let Some(failure) = parser_metadata_failure(&checked) {
                            failures.push(format!(
                                "representative semantic checked metadata: {failure}"
                            ));
                        }
                        match CodeGenerator::new().try_generate_code(checked) {
                            Err(error) => {
                                failures.push(format!(
                                    "representative semantic verified codegen rejected: {error}"
                                ));
                                None
                            }
                            Ok(llvm) => Some(llvm),
                        }
                    }
                },
            };
            let raw_llvm = match IrGenerator::new().try_generate_ir(ast) {
                Err(error) => {
                    failures.push(format!(
                        "representative semantic-independent checked admission rejected: {error}"
                    ));
                    None
                }
                Ok(checked) => {
                    if let Some(failure) = parser_metadata_failure(&checked) {
                        failures.push(format!("representative raw checked metadata: {failure}"));
                    }
                    match CodeGenerator::new().try_generate_code(checked) {
                        Err(error) => {
                            failures.push(format!(
                                "representative independent verified codegen rejected: {error}"
                            ));
                            None
                        }
                        Ok(llvm) => Some(llvm),
                    }
                }
            };
            if let (Some(raw), Some(semantic)) = (&raw_llvm, &semantic_llvm) {
                if raw != semantic {
                    failures
                        .push("representative raw and semantic checked LLVM drifted".to_string());
                }
                if let Some(public) = &first_llvm
                    && semantic != public
                {
                    failures.push(
                        "representative flattened and direct-module LLVM drifted".to_string(),
                    );
                }
            }
        }
    }

    let check = run_aero(&workspace, &["check", "main.aero"]);
    if !check.status.success() {
        failures.push(format!(
            "representative public check failed: {}",
            output_text(&check)
        ));
    }
    let build = run_aero(
        &workspace,
        &["build", "main.aero", "-o", "representative.ll"],
    );
    if !build.status.success() {
        failures.push(format!(
            "representative public build failed: {}",
            output_text(&build)
        ));
    } else {
        let artifact = workspace.root.join("representative.ll");
        if !artifact.is_file() {
            failures.push(
                "representative public build omitted its requested LLVM artifact".to_string(),
            );
        } else {
            match fs::read_to_string(&artifact) {
                Err(error) => failures.push(format!(
                    "representative public LLVM artifact was unreadable: {error}"
                )),
                Ok(public_llvm) => {
                    if let Some(library_llvm) = &first_llvm {
                        for signature in [
                            "define { i32, double, i32 } @decimal_digit",
                            "define { i32, double, i32 } @parse_record",
                        ] {
                            let public_body = llvm_function_body(&public_llvm, signature);
                            let library_body = llvm_function_body(library_llvm, signature);
                            if public_body != library_body {
                                failures.push(format!(
                                    "representative public and library LLVM drifted in {signature}"
                                ));
                            }
                        }
                    }
                    if let Some(failure) = decimal_digit_failure(&public_llvm) {
                        failures.push(format!("representative public LLVM: {failure}"));
                    }
                    if let Some(failure) = parser_guard_failure(&public_llvm) {
                        failures.push(format!("representative public LLVM: {failure}"));
                    }
                }
            }
        }
    }
    let parser_profile_source = format!(
        "{RECORDS_SOURCE}\nfn main() -> int {{ let parsed: Result<int, char> = parse_record(['T', '=', '1', '7', ';', 'H', '=', '0', '8', ';']); result_or(parsed, 1) }}\n"
    );
    for (profile, name) in [
        (LanguageProfile::StableScalarV0, "stable-scalar-v0"),
        (LanguageProfile::ExactI32ArrayV0, "exact-i32-array-v0"),
    ] {
        let result = compile_program(
            &parser_profile_source,
            CompilerOptions {
                language_profile: profile,
                ..CompilerOptions::default()
            },
        );
        match result {
            Ok(_) => failures.push(format!(
                "representative parser unexpectedly entered profile {name}"
            )),
            Err(error)
                if error
                    == format!(
                        "Language Profile Error: {name} rejects function parameter types"
                    ) => {}
            Err(error) => failures.push(format!(
                "representative parser profile {name} produced unexpected diagnostic: {error}"
            )),
        }
    }

    let repository = repository_root();
    for (relative, expected) in [
        ("main.aero", MAIN_SOURCE),
        ("model.aero", MODEL_SOURCE),
        ("policy.aero", POLICY_SOURCE),
        ("records.aero", RECORDS_SOURCE),
        (
            "compile_fail/immutable_projected_update.aero",
            IMMUTABLE_PROJECTED_UPDATE_SOURCE,
        ),
        (
            "compile_fail/unknown_projected_field.aero",
            UNKNOWN_PROJECTED_FIELD_SOURCE,
        ),
        (
            "compile_fail/wrong_policy_argument.aero",
            WRONG_POLICY_ARGUMENT_SOURCE,
        ),
        (
            "compile_fail/unsatisfied_trait_bound.aero",
            UNSATISFIED_TRAIT_BOUND_SOURCE,
        ),
        ("runtime_fail/negative_index.aero", NEGATIVE_INDEX_SOURCE),
        (
            "runtime_fail/upper_bound_index.aero",
            UPPER_BOUND_INDEX_SOURCE,
        ),
        (
            "runtime_fail/parser_negative_index.aero",
            PARSER_NEGATIVE_INDEX_SOURCE,
        ),
        (
            "runtime_fail/parser_equal_to_count_index.aero",
            PARSER_EQUAL_TO_COUNT_INDEX_SOURCE,
        ),
        (
            "runtime_fail/projected_negative_index.aero",
            PROJECTED_NEGATIVE_INDEX_SOURCE,
        ),
        (
            "runtime_fail/projected_upper_bound_index.aero",
            PROJECTED_UPPER_BOUND_INDEX_SOURCE,
        ),
        (
            "runtime_fail/negative_write_index.aero",
            NEGATIVE_WRITE_INDEX_SOURCE,
        ),
        (
            "runtime_fail/upper_bound_write_index.aero",
            UPPER_BOUND_WRITE_INDEX_SOURCE,
        ),
        (
            "runtime_fail/generic_negative_index.aero",
            GENERIC_NEGATIVE_INDEX_SOURCE,
        ),
        (
            "runtime_fail/generic_upper_bound_index.aero",
            GENERIC_UPPER_BOUND_INDEX_SOURCE,
        ),
        (
            "runtime_fail/generic_negative_write_index.aero",
            GENERIC_NEGATIVE_WRITE_INDEX_SOURCE,
        ),
        (
            "runtime_fail/generic_upper_bound_write_index.aero",
            GENERIC_UPPER_BOUND_WRITE_INDEX_SOURCE,
        ),
    ] {
        let path = repository.join(EXAMPLE_DIRECTORY).join(relative);
        match fs::read_to_string(&path) {
            Ok(actual) if actual == expected => {}
            Ok(_) => failures.push(format!(
                "tracked representative source bytes differ at {}",
                path.display()
            )),
            Err(error) => failures.push(format!(
                "tracked representative source is unavailable at {}: {error}",
                path.display()
            )),
        }
    }

    let workflow_path = repository.join(WORKFLOW);
    match fs::read_to_string(&workflow_path) {
        Ok(workflow) => {
            for anchor in [
                "Test representative telemetry application at O0 and O2",
                "examples/representative_telemetry/main.aero",
                "representative_telemetry.o0",
                "representative_telemetry.o2",
                "telemetry score: 91",
                "; Aero generic enum: Sample<Reading<int>>",
                "; Aero generic enum: Sample<char>",
                "aero.trait.PolicyValue.for.Sensor.policy_value",
                "aero.trait.PolicyValue.for.Batch.policy_value",
                "aero.generic.window_get<int>",
                "aero.generic.window_get<char>",
                "aero.generic.window_set<int>",
                "aero.generic.window_set<char>",
                "define { i32, double, i32 } @parse_record([10 x i32]",
                "representative parser LLVM did not contain exactly ten guarded character reads",
                "representative parser retained a forbidden numeric character array lane",
                "representative telemetry test passed with exit code 91",
                "negative_index.aero",
                "upper_bound_index.aero",
                "parser_negative_index.aero",
                "parser_equal_to_count_index.aero",
                "negative_write_index.aero",
                "upper_bound_write_index.aero",
                "generic_negative_index.aero",
                "generic_upper_bound_index.aero",
                "generic_negative_write_index.aero",
                "generic_upper_bound_write_index.aero",
                "projected_negative_index.aero",
                "projected_upper_bound_index.aero",
                "runtime bounds failure corpus passed at O0 and O2",
                "Test representative telemetry application on Windows at O0 and O2",
            ] {
                if !workflow.contains(anchor) {
                    failures.push(format!(
                        "native workflow {} omitted anchor {anchor:?}",
                        workflow_path.display()
                    ));
                }
            }
            let shared_parser_anchors = [
                "define { i32, double, i32 } @parse_record([10 x i32]",
                "representative parser LLVM did not contain exactly ten guarded character reads",
                "representative parser retained a forbidden numeric character array lane",
                "parser_negative_index.aero",
                "parser_equal_to_count_index.aero",
            ];
            let linux_step = workflow
                .split("- name: Test representative telemetry application at O0 and O2")
                .nth(1)
                .and_then(|rest| rest.split("- name: Run tests").next());
            let windows_step = workflow
                .split("- name: Test representative telemetry application on Windows at O0 and O2")
                .nth(1)
                .and_then(|rest| rest.split("\n    - name:").next());
            for (lane, step) in [("Linux", linux_step), ("Windows", windows_step)] {
                let Some(step) = step else {
                    failures.push(format!(
                        "native workflow omitted the {lane} representative step body"
                    ));
                    continue;
                };
                for anchor in shared_parser_anchors {
                    if !step.contains(anchor) {
                        failures.push(format!(
                            "{lane} representative workflow omitted parser anchor {anchor:?}"
                        ));
                    }
                }
            }
            for anchor in shared_parser_anchors {
                if occurrences(&workflow, anchor) != 2 {
                    failures.push(format!(
                        "native workflow must bind parser anchor {anchor:?} once per OS lane"
                    ));
                }
            }
            if let Some(step) = linux_step {
                for anchor in [
                    r#"parser_llvm="$(awk '/^define .*@parse_record\(/ { capture=1 } capture { print } capture && /^}/ { exit }' "${representative_llvm}")"#,
                    r#"parser_lower_count="$(printf '%s\n' "${parser_llvm}" | grep -Fc 'fcmp oge double' || true)"#,
                    r#"parser_upper_count="$(printf '%s\n' "${parser_llvm}" | grep -Fc 'fcmp olt double' || true)"#,
                    r#"parser_trap_count="$(printf '%s\n' "${parser_llvm}" | grep -Fc 'call void @llvm.trap()' || true)"#,
                    r#"parser_gep_count="$(printf '%s\n' "${parser_llvm}" | grep -Fc 'getelementptr inbounds [10 x i32]' || true)"#,
                    r#"if test "${parser_lower_count}" -ne 10 \"#,
                    r#"|| test "${parser_upper_count}" -ne 10 \"#,
                    r#"|| test "${parser_trap_count}" -ne 10 \"#,
                    r#"|| test "${parser_gep_count}" -ne 10; then"#,
                    r#"if grep -Fq '[10 x double]' "${representative_llvm}" \"#,
                    r#"|| grep -Fq 'sitofp i32 %aero.arg.character' "${representative_llvm}"; then"#,
                ] {
                    if !step.contains(anchor) {
                        failures.push(format!(
                            "Linux representative workflow omitted parser predicate {anchor:?}"
                        ));
                    }
                }
            }
            if let Some(step) = windows_step {
                for anchor in [
                    "$parserMatch = [regex]::Match(",
                    r#"([regex]::Matches($parserText, [regex]::Escape("fcmp oge double"))).Count,"#,
                    r#"([regex]::Matches($parserText, [regex]::Escape("fcmp olt double"))).Count,"#,
                    r#"([regex]::Matches($parserText, [regex]::Escape("call void @llvm.trap()"))).Count,"#,
                    r#"([regex]::Matches($parserText, [regex]::Escape("getelementptr inbounds [10 x i32]"))).Count"#,
                    "if (@($parserCounts | Where-Object { $_ -ne 10 }).Count -ne 0) {",
                    r#"if ($representativeText.Contains("[10 x double]") -or $representativeText.Contains("sitofp i32 %aero.arg.character")) {"#,
                ] {
                    if !step.contains(anchor) {
                        failures.push(format!(
                            "Windows representative workflow omitted parser predicate {anchor:?}"
                        ));
                    }
                }
            }
        }
        Err(error) => failures.push(format!(
            "native workflow {} is unavailable: {error}",
            workflow_path.display()
        )),
    }

    assert!(
        failures.is_empty(),
        "M1-001 representative scalar application failures:\n{}",
        failures.join("\n\n")
    );
}

#[test]
fn representative_compile_fail_corpus_rejects_before_artifacts() {
    for (name, source, diagnostic) in [
        (
            "immutable_projected_update",
            IMMUTABLE_PROJECTED_UPDATE_SOURCE,
            "projected assignment root `sensor` must be a mutable local owned binding",
        ),
        (
            "unknown_projected_field",
            UNKNOWN_PROJECTED_FIELD_SOURCE,
            "struct `Sensor` has no field `missing`",
        ),
        (
            "wrong_policy_argument",
            WRONG_POLICY_ARGUMENT_SOURCE,
            "parameter `trusted` type mismatch: expected bool, actual int",
        ),
        (
            "unsatisfied_trait_bound",
            UNSATISFIED_TRAIT_BOUND_SOURCE,
            "type `Other` does not implement trait `Score` required by generic function `evaluate`",
        ),
    ] {
        let workspace = TestWorkspace::new();
        workspace.write("main.aero", source);

        for command in ["check", "build", "run"] {
            let output_name = format!("{name}-{command}.ll");
            let arguments = if command == "build" {
                vec![command, "main.aero", "-o", output_name.as_str()]
            } else {
                vec![command, "main.aero"]
            };
            let output = run_aero(&workspace, &arguments);
            let rendered = output_text(&output);
            assert_eq!(
                output.status.code(),
                Some(1),
                "{name} public {command} must reject with status 1: {rendered}"
            );
            assert!(
                rendered.contains(diagnostic),
                "{name} public {command} omitted deterministic diagnostic {diagnostic:?}: {rendered}"
            );
            assert!(
                !workspace.root.join(&output_name).exists(),
                "{name} public {command} emitted forbidden requested artifact {output_name}"
            );
            assert!(
                !contains_regular_file(&workspace.root.join("target")),
                "{name} public {command} emitted a native artifact after rejection"
            );
        }
    }
}

#[test]
fn admitted_numeric_print_arguments_keep_their_llvm_vararg_type() {
    let workspace = TestWorkspace::new();
    let root = workspace.write("numeric_print_abi.aero", NUMERIC_PRINT_ABI_SOURCE);
    let llvm = compile_file(&root, CompilerOptions::default())
        .expect("admitted numeric print ABI specimen must compile");
    let calls = llvm
        .lines()
        .filter(|line| line.contains("call i32") && line.contains("@printf("))
        .collect::<Vec<_>>();

    assert_eq!(calls.len(), 2, "expected print! and println! call sites");
    assert!(
        calls
            .iter()
            .all(|call| call.contains("call i32 (i8*, ...) @printf(")),
        "printf calls must retain their explicit variadic callee type: {calls:?}"
    );
    assert_eq!(
        calls[0].matches(", double ").count(),
        3,
        "all computed int/float/bool arguments must remain typed LLVM doubles: {}",
        calls[0]
    );
    assert_eq!(
        calls[1].matches(", double ").count(),
        1,
        "the immediate integer argument must remain a typed LLVM double: {}",
        calls[1]
    );
    assert!(
        calls.iter().all(|call| !call.contains(", i64 ")),
        "numeric printf arguments must not be rewritten as raw integer bits: {calls:?}"
    );
    assert!(
        !llvm.contains("bitcast double") || !llvm.contains("to i64"),
        "numeric print lowering must not manufacture i64 varargs from double values"
    );
}
