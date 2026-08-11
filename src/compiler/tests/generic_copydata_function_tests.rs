use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};

const EXECUTABLE_PRODUCT: &str = r#"
struct Reading<T> {
    value: T,
    valid: bool,
}

fn choose<T>(first: T, second: T, use_first: bool) -> T {
    if use_first {
        return first;
    }
    return second;
}

fn main() -> int {
    let scalar: int = choose(40, 41, 1 < 2);
    let marker: char = choose('a', 'b', 2 < 1);
    let first: Reading<int> = Reading { value: 1, valid: 1 < 2 };
    let second: Reading<int> = Reading { value: 2, valid: 2 < 3 };
    let selected: Reading<int> = choose(first, second, 3 < 4);
    if marker == 'b' && selected.valid {
        return scalar + selected.value;
    }
    return 0;
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
fn called_generic_copydata_function_reaches_semantics() {
    semantic(EXECUTABLE_PRODUCT)
        .expect("bound-free generic CopyData function calls must pass semantics");
}

#[test]
fn called_generic_copydata_function_reaches_checked_llvm_deterministically() {
    let checked = checked_without_semantics(EXECUTABLE_PRODUCT)
        .expect("raw checked admission must apply shared generic-function specialization");
    let raw_llvm = CodeGenerator::new()
        .try_generate_code(checked)
        .expect("independently verified generic-function IR must lower");
    let first = compile_program(EXECUTABLE_PRODUCT, CompilerOptions::default())
        .expect("called generic CopyData function must compile");
    let second = compile_program(EXECUTABLE_PRODUCT, CompilerOptions::default())
        .expect("repeat generic-function compilation must succeed");

    assert_eq!(first, second, "generic-function LLVM must be deterministic");
    assert_eq!(first, raw_llvm, "semantic and raw checked routes drifted");
    assert!(
        first.contains("aero.generic.choose<int>")
            && first.contains("aero.generic.choose<char>")
            && first.contains("aero.generic.choose<Reading<int>>"),
        "all concrete specializations must remain distinguishable:\n{first}"
    );
    assert!(
        !first.contains("__aero$generic_function$"),
        "private generic-function identities leaked into LLVM:\n{first}"
    );
}
