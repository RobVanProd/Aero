use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};

const MIXED_ALIAS_INT_STRUCT: &str = r#"
struct Box<T> { value: T }

fn take_int(value: Box<int>) -> int {
    value.value
}

fn main() -> int {
    let value: Box<i32> = Box { value: 42 };
    take_int(value)
}
"#;

const MIXED_ALIAS_FLOAT_STRUCT: &str = r#"
struct Box<T> { value: T }

fn positive(value: Box<float>) -> bool {
    value.value > 1.0
}

fn main() -> int {
    let value: Box<f64> = Box { value: 1.5 };
    if positive(value) {
        return 42;
    }
    0
}
"#;

const MIXED_ALIAS_WINDOW: &str = r#"
struct Window<T> { values: [T; 2] }

fn first<T>(window: Window<T>) -> T {
    window.values[0]
}

fn take_window(value: Window<int>) -> int {
    first(value)
}

fn main() -> int {
    let value: Window<i32> = Window { values: [42, 7] };
    take_window(value)
}
"#;

const MIXED_ALIAS_ENUM: &str = r#"
enum Sample<T> {
    Present(T),
    Missing,
}

fn score(value: Sample<int>) -> int {
    match value {
        Sample::Present(number) => number,
        Sample::Missing => 0,
    }
}

fn main() -> int {
    let value: Sample<i32> = Sample::Present(42);
    score(value)
}
"#;

const MIXED_ALIAS_TRAIT_SIGNATURE: &str = r#"
struct Reading { value: int }

trait Score {
    fn score(&self, add: int) -> int;
}

impl Score for Reading {
    fn score(&self, add: i32) -> i32 {
        (*self).value + add
    }
}

fn evaluate<T: Score>(value: T) -> int {
    value.score(2)
}

fn main() -> int {
    evaluate(Reading { value: 40 })
}
"#;

const CANONICAL_GENERIC_FUNCTION_CONTROL: &str = r#"
fn identity<T>(value: T) -> T {
    value
}

fn main() -> int {
    let first: i32 = identity(40);
    let second: int = identity(first);
    second + 2
}
"#;

fn parsed(source: &str) -> Vec<compiler::ast::AstNode> {
    let tokens = try_tokenize_with_locations(source, None).expect("fixture must lex");
    parse_with_locations(tokens).expect("fixture must parse")
}

fn assert_all_trusted_routes_compile(source: &str) -> String {
    let semantic = SemanticAnalyzer::new()
        .analyze(parsed(source))
        .map(|_| ())
        .map_err(|error| error.to_string());
    let raw = IrGenerator::new()
        .try_generate_ir(parsed(source))
        .map_err(|error| error.to_string())
        .and_then(|checked| {
            CodeGenerator::new()
                .try_generate_code(checked)
                .map_err(|error| error.to_string())
        });
    let public = compile_program(source, CompilerOptions::default());

    assert!(
        semantic.is_ok() && raw.is_ok() && public.is_ok(),
        "alias-equivalent source must reach every trusted route:\nsemantic={semantic:?}\nraw={raw:?}\npublic={public:?}"
    );

    let raw = raw.expect("raw checked route was proven successful");
    let public = public.expect("public route was proven successful");
    assert_eq!(raw, public, "semantic and raw checked routes drifted");
    public
}

#[test]
fn int_aliases_compose_through_generic_struct_transport() {
    let llvm = assert_all_trusted_routes_compile(MIXED_ALIAS_INT_STRUCT);
    assert!(
        llvm.contains("Box<int>"),
        "canonical identity is absent:\n{llvm}"
    );
    assert!(
        !llvm.contains("Box<i32>"),
        "int alias spelling split specialization identity:\n{llvm}"
    );
}

#[test]
fn float_aliases_compose_through_generic_struct_transport() {
    let llvm = assert_all_trusted_routes_compile(MIXED_ALIAS_FLOAT_STRUCT);
    assert!(
        llvm.contains("Box<float>"),
        "canonical identity is absent:\n{llvm}"
    );
    assert!(
        !llvm.contains("Box<f64>"),
        "float alias spelling split specialization identity:\n{llvm}"
    );
}

#[test]
fn primitive_aliases_compose_through_generic_window_algorithms() {
    let llvm = assert_all_trusted_routes_compile(MIXED_ALIAS_WINDOW);
    for identity in ["Window<int>", "aero.generic.first<int>"] {
        assert!(
            llvm.contains(identity),
            "canonical specialization identity {identity:?} is absent:\n{llvm}"
        );
    }
    for split_identity in ["Window<i32>", "first<i32>"] {
        assert!(
            !llvm.contains(split_identity),
            "alias spelling split specialization identity {split_identity:?}:\n{llvm}"
        );
    }
}

#[test]
fn primitive_aliases_compose_through_generic_enum_transport_and_match() {
    let llvm = assert_all_trusted_routes_compile(MIXED_ALIAS_ENUM);
    assert!(
        llvm.contains("Sample<int>"),
        "canonical enum specialization is absent:\n{llvm}"
    );
    assert!(
        !llvm.contains("Sample<i32>"),
        "alias spelling split enum specialization identity:\n{llvm}"
    );
}

#[test]
fn primitive_aliases_compose_between_trait_declarations_and_implementations() {
    let llvm = assert_all_trusted_routes_compile(MIXED_ALIAS_TRAIT_SIGNATURE);
    assert!(
        llvm.contains("aero.trait.Score.for.Reading.score"),
        "canonical trait helper is absent:\n{llvm}"
    );
}

#[test]
fn generic_functions_remain_the_existing_alias_canonicalization_control() {
    let llvm = assert_all_trusted_routes_compile(CANONICAL_GENERIC_FUNCTION_CONTROL);
    assert!(
        llvm.contains("aero.generic.identity<int>"),
        "canonical generic-function identity is absent:\n{llvm}"
    );
    assert!(
        !llvm.contains("aero.generic.identity<i32>"),
        "generic-function alias identity unexpectedly split:\n{llvm}"
    );
}
