use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};

const EXECUTABLE_PRODUCT: &str = r#"
struct Reading<T> {
    value: T,
    valid: bool,
}

enum Sample<T> {
    Present(T),
    Missing,
}

fn make_reading() -> Reading<int> {
    Reading { value: 1, valid: 1 < 2 }
}

fn score_number(value: Sample<int>) -> int {
    match value {
        Sample::Present(number) => number,
        Sample::Missing => 0,
    }
}

fn score_marker(value: Sample<char>) -> int {
    match value {
        Sample::Present(marker) => 2,
        Sample::Missing => 0,
    }
}

fn score_reading(value: Sample<Reading<int>>) -> int {
    match value {
        Sample::Present(reading) => reading.value,
        Sample::Missing => 0,
    }
}

fn main() -> int {
    let number: Sample<int> = Sample::Present(40);
    let marker: Sample<char> = Sample::Present('a');
    let reading: Sample<Reading<int>> = Sample::Present(make_reading());
    return score_number(number) + score_marker(marker) + score_reading(reading);
}
"#;

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
fn explicit_generic_copydata_enums_reach_both_trusted_routes() {
    semantic(EXECUTABLE_PRODUCT)
        .expect("explicit generic CopyData enum applications must pass semantics");
    let checked = checked_without_semantics(EXECUTABLE_PRODUCT)
        .expect("raw checked admission must apply shared generic-enum specialization");
    let raw_llvm = CodeGenerator::new()
        .try_generate_code(checked)
        .expect("independently verified generic-enum IR must lower");
    let public_llvm = compile_program(EXECUTABLE_PRODUCT, CompilerOptions::default())
        .expect("explicit generic CopyData enum applications must compile");

    assert_eq!(raw_llvm, public_llvm, "trusted generic-enum routes drifted");
    for identity in ["Sample<int>", "Sample<char>", "Sample<Reading<int>>"] {
        assert!(
            public_llvm.contains(identity),
            "specialized enum identity {identity:?} is not source-readable:\n{public_llvm}"
        );
    }
    assert!(
        !public_llvm.contains("__aero$generic_enum$"),
        "private generic-enum identity leaked into LLVM:\n{public_llvm}"
    );
}

#[test]
fn generic_enum_bound_metadata_is_not_silently_discarded() {
    let ast = parsed("enum Sample<T: Comparable> { Present(T), Missing }");
    let retained = format!("{ast:#?}");
    assert!(
        retained.contains("Comparable"),
        "generic-enum bound metadata was discarded before the fail-closed authority:\n{retained}"
    );
}
