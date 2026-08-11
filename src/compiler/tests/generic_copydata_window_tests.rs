use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};

const GENERIC_WINDOW_PRODUCT: &str = r#"
struct Window<T> {
    values: [T; 3],
}

fn window_get<T>(window: Window<T>, index: int) -> T {
    window.values[index]
}

fn window_set<T>(window: Window<T>, index: int, value: T) -> Window<T> {
    let mut updated: Window<T> = window;
    updated.values[index] = value;
    updated
}

fn main() -> int {
    let seed: Window<int> = Window { values: [10, 20, 30] };
    let updated = window_set(seed, 1, 21);
    window_get(updated, 1)
}
"#;

fn parsed(source: &str) -> Vec<compiler::ast::AstNode> {
    let tokens = try_tokenize_with_locations(source, None).expect("fixture must lex");
    parse_with_locations(tokens).expect("fixture must parse")
}

#[test]
fn generic_fixed_window_algorithms_execute_end_to_end() {
    let parsed = parsed(GENERIC_WINDOW_PRODUCT);
    let (_, analyzed) = SemanticAnalyzer::new()
        .analyze(parsed.clone())
        .expect("generic fixed-window algorithms must pass semantics");
    let checked = IrGenerator::new()
        .try_generate_ir(parsed)
        .expect("raw checked admission must share generic-window specialization");
    let llvm = CodeGenerator::new()
        .try_generate_code(checked)
        .expect("independently verified generic-window IR must lower");
    let public = compile_program(GENERIC_WINDOW_PRODUCT, CompilerOptions::default())
        .expect("generic fixed-window algorithms must compile publicly");

    assert!(!analyzed.is_empty());
    assert_eq!(llvm, public, "semantic and raw checked routes drifted");
    assert!(
        public.contains("aero.generic.window_get<Window<int>,int>")
            && public.contains("aero.generic.window_set<Window<int>,int>"),
        "generic-window specializations were not source-readable:\n{public}"
    );
}
