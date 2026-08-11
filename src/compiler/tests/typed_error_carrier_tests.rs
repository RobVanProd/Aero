use compiler::{
    CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program, parse_with_locations,
    try_tokenize_with_locations,
};

const MISSING_CONTEXT: &str =
    "requires an exact expected Option<T> or Result<T, E> type; missing type arguments are never inferred by default";

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

fn main() -> int {
    let present: Option<int> = maybe(11, true);
    let absent: Option<int> = maybe(9, false);
    let success: Result<int, char> = validate(17, true);
    let failure: Result<int, char> = validate(19, false);
    return option_score(present)
        + option_score(absent)
        + result_score(success)
        + result_score(failure);
}
"#;

    let llvm = compile_program(source, CompilerOptions::default())
        .expect("explicit CopyData Option/Result program must compile");
    assert!(llvm.contains("define i32 @maybe("), "missing maybe function:\n{llvm}");
    assert!(
        llvm.contains("define i32 @validate("),
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
}
