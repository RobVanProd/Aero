use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};

const EXECUTABLE_PRODUCT: &str = r#"
struct Reading<T> {
    value: T,
    valid: bool,
}

fn make_reading(value: int, valid: bool) -> Reading<int> {
    Reading { value: value, valid: valid }
}

fn score_reading(reading: Reading<int>) -> int {
    if reading.valid {
        return reading.value;
    }
    return 0;
}

fn score_marker(marker: Reading<char>) -> int {
    if marker.valid && marker.value == 'a' {
        return 2;
    }
    return 0;
}

fn main() -> int {
    let mut reading: Reading<int> = make_reading(40, 1 < 2);
    reading = Reading { value: 41, valid: 2 < 3 };
    let marker: Reading<char> = Reading { value: 'a', valid: 3 < 4 };
    return score_reading(reading) + score_marker(marker);
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
fn explicit_generic_copydata_structs_reach_semantics() {
    semantic(EXECUTABLE_PRODUCT)
        .expect("explicit generic CopyData struct applications must pass semantics");
}

#[test]
fn explicit_generic_copydata_structs_reach_checked_llvm_deterministically() {
    let checked = checked_without_semantics(EXECUTABLE_PRODUCT)
        .expect("raw AST admission must apply the shared generic-struct elaboration");
    let raw_llvm = CodeGenerator::new()
        .try_generate_code(checked)
        .expect("independently verified generic-struct IR must lower");
    let first = compile_program(EXECUTABLE_PRODUCT, CompilerOptions::default())
        .expect("explicit generic CopyData struct program must compile");
    let second = compile_program(EXECUTABLE_PRODUCT, CompilerOptions::default())
        .expect("repeat generic CopyData struct compilation must succeed");

    assert_eq!(first, second, "generic-struct LLVM must be deterministic");
    assert_eq!(first, raw_llvm, "semantic and raw checked routes drifted");
    assert!(
        first.contains("define i32 @score_reading(")
            && first.contains("define i32 @score_marker("),
        "generic-struct consumers were not emitted:\n{first}"
    );
    assert!(
        !first.contains("__aero$generic_struct$"),
        "private generic-struct identities leaked into LLVM:\n{first}"
    );
}
