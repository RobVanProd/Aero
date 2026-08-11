use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};

const MISSING_CONTEXT: &str = "requires an exact expected Option<T> or Result<T, E> type; missing type arguments are never inferred by default";

fn parsed(source: &str) -> Vec<compiler::ast::AstNode> {
    let tokens = try_tokenize_with_locations(source, None).expect("fixture must lex");
    parse_with_locations(tokens).expect("fixture must parse")
}

fn semantic(source: &str) -> Result<Vec<compiler::ast::AstNode>, String> {
    SemanticAnalyzer::new()
        .analyze(parsed(source))
        .map(|(_, ast)| ast)
}

fn checked_without_semantics(source: &str) -> Result<compiler::CheckedIr, String> {
    IrGenerator::new()
        .try_generate_ir(parsed(source))
        .map_err(|error| error.to_string())
}

#[test]
fn missing_builtin_carrier_type_arguments_fail_closed_before_checked_ir() {
    let cases = [
        ("None", "fn main() { let value = None; }"),
        ("Ok", "fn main() { let value = Ok(7); }"),
        ("Err", "fn main() { let value = Err('e'); }"),
    ];

    let mut failures = Vec::new();
    for (constructor, source) in cases {
        match semantic(source) {
            Ok(_) => failures.push(format!(
                "{constructor}: semantic analysis fabricated a missing carrier type argument"
            )),
            Err(error) if error.contains(constructor) && error.contains(MISSING_CONTEXT) => {}
            Err(error) => failures.push(format!(
                "{constructor}: unexpected semantic diagnostic {error:?}"
            )),
        }

        match compile_program(source, CompilerOptions::default()) {
            Ok(llvm) => failures.push(format!(
                "{constructor}: public compilation unexpectedly succeeded:\n{llvm}"
            )),
            Err(error) if error.contains(constructor) && error.contains(MISSING_CONTEXT) => {}
            Err(error) => failures.push(format!(
                "{constructor}: unexpected public diagnostic {error:?}"
            )),
        }

        match checked_without_semantics(source) {
            Ok(ir) => failures.push(format!(
                "{constructor}: raw AST reached trusted checked IR: {ir:?}"
            )),
            Err(error) if error.contains(constructor) && error.contains(MISSING_CONTEXT) => {}
            Err(error) => failures.push(format!(
                "{constructor}: unexpected independent-admission diagnostic {error:?}"
            )),
        }
    }

    assert!(
        failures.is_empty(),
        "CAP-003 missing-context failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn explicitly_typed_option_and_result_execute_through_owned_enum_contracts() {
    let source = r#"
fn maybe(value: int, keep: bool) -> Option<int> {
    if keep {
        return Some(value);
    }
    return None;
}

fn validate(value: int, valid: bool) -> Result<int, char> {
    if valid {
        return Ok(value);
    }
    return Err('e');
}

struct Reading {
    value: int,
    valid: bool,
}

fn validate_reading(value: int, valid: bool) -> Result<Reading, char> {
    if valid {
        return Ok(Reading { value: value, valid: valid });
    }
    return Err('r');
}

fn option_score(value: Option<int>) -> int {
    return match value {
        Some(number) => number,
        None => 3,
    };
}

fn result_score(value: Result<int, char>) -> int {
    return match value {
        Ok(number) => number,
        Err(code) => 5,
    };
}

fn reading_score(value: Result<Reading, char>) -> int {
    return match value {
        Ok(reading) => reading.value,
        Err(code) => 7,
    };
}

fn main() -> int {
    let present: Option<int> = maybe(11, 1 < 2);
    let absent: Option<int> = maybe(9, 2 < 1);
    let success: Result<int, char> = validate(17, 3 < 4);
    let failure: Result<int, char> = validate(19, 4 < 3);
    let reading: Result<Reading, char> = validate_reading(23, 5 < 6);
    return option_score(present)
        + option_score(absent)
        + result_score(success)
        + result_score(failure)
        + reading_score(reading);
}
"#;

    let llvm = compile_program(source, CompilerOptions::default())
        .expect("explicit CopyData Option/Result program must compile");
    assert!(llvm.contains(" @maybe("), "missing maybe function:\n{llvm}");
    assert!(
        llvm.contains(" @validate("),
        "missing validate function:\n{llvm}"
    );
    assert!(
        llvm.contains("define i32 @option_score("),
        "missing Option consumer:\n{llvm}"
    );
    assert!(
        llvm.contains("define i32 @result_score("),
        "missing Result consumer:\n{llvm}"
    );
    assert!(
        llvm.contains("define i32 @reading_score("),
        "missing recursive CopyData Result consumer:\n{llvm}"
    );
    assert!(
        !llvm.contains("__aero$carrier$"),
        "private carrier identities leaked into public LLVM text:\n{llvm}"
    );
}

#[test]
fn exact_contexts_replacement_and_multiple_instantiations_share_one_contract() {
    let source = r#"
fn take_option(value: Option<int>) -> int {
    return match value { Some(number) => number, None => 0 };
}

fn take_character(value: Option<char>) -> int {
    return match value { Some(character) => 2, None => 1 };
}

fn take_result(value: Result<int, char>) -> int {
    return match value { Ok(number) => number, Err(code) => 4 };
}

fn make_result(valid: bool) -> Result<int, char> {
    if valid { return Ok(8); }
    Err('x')
}

fn main() -> int {
    let mut replaced: Option<int> = Some(3);
    replaced = None;
    let direct_argument = take_option(Some(5));
    let character: Option<char> = Some('a');
    let success: Result<int, char> = make_result(1 < 2);
    let failure: Result<int, char> = make_result(2 < 1);
    return take_option(replaced)
        + direct_argument
        + take_character(character)
        + take_result(success)
        + take_result(failure);
}
"#;

    let raw_checked = checked_without_semantics(source)
        .expect("raw AST admission must apply the same carrier normalization");
    let raw_llvm = CodeGenerator::new()
        .try_generate_code(raw_checked)
        .expect("independently checked carrier IR must lower");
    let first = compile_program(source, CompilerOptions::default())
        .expect("all exact carrier contexts must compile");
    let second = compile_program(source, CompilerOptions::default())
        .expect("deterministic repeat compilation must succeed");
    assert_eq!(first, second, "carrier LLVM must be deterministic");
    assert_eq!(first, raw_llvm, "semantic and raw checked routes drifted");
    assert!(
        first.contains(" @take_option(")
            && first.contains(" @take_character(")
            && first.contains(" @take_result("),
        "multiple concrete carrier functions were not emitted:\n{first}"
    );
}

#[test]
fn excluded_carrier_topologies_fail_before_trusted_llvm() {
    let cases = [
        (
            "context-free Some",
            "fn main() { let value = Some(7); }",
            "missing type arguments",
        ),
        (
            "wrong constructor family",
            "fn main() { let value: Option<int> = Ok(7); }",
            "belongs to Result",
        ),
        (
            "wrong constructor payload",
            "fn main() { let value: Option<int> = Some(1.5); }",
            "expected int, actual float",
        ),
        (
            "nested carrier argument",
            "fn main() { let value: Option<Option<int>> = None; }",
            "nested Option/Result",
        ),
        (
            "String carrier argument",
            "fn main() { let value: Result<int, String> = Err(\"bad\"); }",
            "not admitted recursive finite CopyData",
        ),
        (
            "carrier struct field",
            "struct Box { value: Option<int> } fn main() { 0 }",
            "not admitted inside struct fields",
        ),
        (
            "generic carrier function",
            "fn wrap<T>(value: int) -> Option<int> { return Some(value); } fn main() { 0 }",
            "generic function `wrap`",
        ),
        (
            "carrier borrowing",
            "fn main() { let value: Option<int> = Some(1); let reference = &value; }",
            "not admitted in borrowing",
        ),
        (
            "carrier aggregate storage",
            "fn main() { let value: Option<int> = Some(1); let stored = [value]; }",
            "not admitted in aggregate storage",
        ),
        (
            "carrier comparison",
            "fn main() { let left: Option<int> = Some(1); let right: Option<int> = None; let same = left == right; }",
            "not admitted in binary/comparison/logical operands",
        ),
        (
            "carrier output",
            "fn main() { let value: Result<int, char> = Ok(1); println!(\"{}\", value); }",
            "not admitted in formatted output",
        ),
        (
            "carrier process result",
            "fn main() -> Option<int> { return Some(1); }",
            "process entry `main`",
        ),
        (
            "carrier use after move",
            "fn take(value: Option<int>) -> int { return match value { Some(number) => number, None => 0 }; } fn main() { let value: Option<int> = Some(1); let score = take(value); let reused = value; }",
            "moved value `value`",
        ),
        (
            "incomplete carrier Match",
            "fn main() { let value: Option<int> = Some(1); let score = match value { Some(number) => number }; }",
            "cover every declared variant exactly once",
        ),
    ];

    let mut failures = Vec::new();
    for (label, source, expected) in cases {
        for (route, result) in [
            (
                "semantic",
                semantic(source).map(|_| "semantic success".to_string()),
            ),
            (
                "raw checked",
                checked_without_semantics(source).map(|_| "checked success".to_string()),
            ),
            (
                "public",
                compile_program(source, CompilerOptions::default())
                    .map(|_| "public success".to_string()),
            ),
        ] {
            match result {
                Ok(success) => failures.push(format!(
                    "{label} {route}: unexpectedly accepted ({success})"
                )),
                Err(error) if error.contains(expected) && !error.contains("__aero$carrier$") => {}
                Err(error) if error.contains("__aero$carrier$") => failures.push(format!(
                    "{label} {route}: private carrier identity leaked into source diagnostic {error:?}"
                )),
                Err(error) => failures.push(format!(
                    "{label} {route}: expected {expected:?}, got {error:?}"
                )),
            }
        }
    }

    assert!(
        failures.is_empty(),
        "CAP-003 excluded-topology failures:\n{}",
        failures.join("\n")
    );
}
