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

const GENERIC_WINDOW_TYPE_MATRIX: &str = r#"
struct Point { value: int }
struct Box<T> { value: T }
struct Window<T> { values: [T; 3] }
struct Split<T, U> { left: [T; 2], right: [U; 2] }
struct Tagged<T> { values: [T; 2], valid: bool }

fn window_get<T>(window: Window<T>, index: int) -> T {
    window.values[index]
}

fn split_left<T, U>(window: Split<T, U>, index: int) -> T {
    window.left[index]
}

fn tagged_valid<T>(tagged: Tagged<T>) -> bool {
    tagged.valid
}

fn window_get_with_side<T, U>(window: Window<T>, index: int, side: U) -> T {
    let copy: U = side;
    window.values[index]
}

fn main() -> int {
    let integers: Window<int> = Window { values: [10, 20, 30] };
    let markers: Window<char> = Window { values: ['a', 'b', 'c'] };
    let points: Window<Point> = Window {
        values: [Point { value: 1 }, Point { value: 2 }, Point { value: 3 }],
    };
    let tuples: Window<(int, char)> = Window {
        values: [(4, 'd'), (5, 'e'), (6, 'f')],
    };
    let arrays: Window<[int; 2]> = Window { values: [[7, 8], [9, 10], [11, 12]] };
    let boxes: Window<Box<int>> = Window {
        values: [Box { value: 13 }, Box { value: 14 }, Box { value: 15 }],
    };
    let split: Split<int, char> = Split { left: [16, 17], right: ['x', 'y'] };
    let tagged: Tagged<int> = Tagged { values: [18, 19], valid: 1 < 2 };
    if window_get(markers, 1) == 'b' && tagged_valid(tagged) {
        return window_get(integers, 1)
            + window_get(points, 1).value
            + window_get(tuples, 1).0
            + window_get(arrays, 1)[0]
            + window_get(boxes, 1).value
            + split_left(split, 1)
            + window_get_with_side(integers, 0, 'q');
    }
    0
}
"#;

fn parsed(source: &str) -> Vec<compiler::ast::AstNode> {
    let tokens = try_tokenize_with_locations(source, None).expect("fixture must lex");
    parse_with_locations(tokens).expect("fixture must parse")
}

fn assert_shared_rejection(source: &str, diagnostic: &str) {
    let parsed = parsed(source);
    let semantic = SemanticAnalyzer::new()
        .analyze(parsed.clone())
        .expect_err("excluded generic-container source must fail semantics");
    let checked = IrGenerator::new()
        .try_generate_ir(parsed)
        .expect_err("excluded generic-container source must fail raw checked admission");
    let public = compile_program(source, CompilerOptions::default())
        .expect_err("excluded generic-container source must fail public compilation");
    for (route, error) in [
        ("semantic", semantic),
        ("checked", checked.to_string()),
        ("public", public),
    ] {
        assert!(
            error.contains(diagnostic),
            "{route} rejection `{error}` did not contain `{diagnostic}`"
        );
    }
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
        public.contains("aero.generic.window_get<int>")
            && public.contains("aero.generic.window_set<int>"),
        "generic-window specializations were not source-readable:\n{public}"
    );
}

#[test]
fn generic_container_inference_composes_across_recursive_copydata_types() {
    let llvm = compile_program(GENERIC_WINDOW_TYPE_MATRIX, CompilerOptions::default())
        .expect("the complete recursive-CopyData element matrix must compile");
    for specialization in [
        "aero.generic.window_get<int>",
        "aero.generic.window_get<char>",
        "aero.generic.window_get<Point>",
        "aero.generic.window_get<(int,char)>",
        "aero.generic.window_get<[int;2]>",
        "aero.generic.window_get<Box<int>>",
        "aero.generic.split_left<int,char>",
        "aero.generic.tagged_valid<int>",
        "aero.generic.window_get_with_side<int,char>",
    ] {
        assert!(
            llvm.contains(specialization),
            "missing source-readable specialization `{specialization}`:\n{llvm}"
        );
    }
}

#[test]
fn excluded_generic_container_signatures_fail_closed_in_both_trusted_routes() {
    let prefix = "struct Pair<T, U> { first: T, second: U } ";
    for (label, source) in [
        (
            "partially concrete application",
            "fn bad<T>(value: Pair<T, int>) -> T { value.first } fn main() -> int { 0 }",
        ),
        (
            "reordered application",
            "fn bad<T, U>(value: Pair<U, T>) -> T { value.second } fn main() -> int { 0 }",
        ),
        (
            "repeated application",
            "fn bad<T, U>(value: Pair<T, T>) -> T { value.first } fn main() -> int { 0 }",
        ),
        (
            "result-only inference",
            "fn bad<T>(value: int) -> Pair<T, T> { value } fn main() -> int { 0 }",
        ),
        (
            "reference application",
            "fn bad<T, U>(value: Pair<&T, U>) -> U { value.second } fn main() -> int { 0 }",
        ),
    ] {
        let source = format!("{prefix}{source}");
        let parsed = parsed(&source);
        assert!(!parsed.is_empty(), "{label} fixture must parse");
        assert_shared_rejection(
            &source,
            "cannot transport an explicit generic CopyData struct",
        );
    }
}

#[test]
fn excluded_parametric_container_uses_share_one_classifier_boundary() {
    let cases = [
        (
            "arithmetic",
            "fn bad<T>(window: Window<T>, index: int, value: T) -> T { window.values[index] + value }",
        ),
        (
            "comparison",
            "fn bad<T>(window: Window<T>, index: int, value: T) -> bool { window.values[index] == value }",
        ),
        (
            "arbitrary call argument",
            "fn consume(value: int) -> int { value } fn bad<T>(window: Window<T>, index: int) -> int { consume(window.values[index]) }",
        ),
        (
            "borrowing",
            "fn bad<T>(window: Window<T>) -> T { let borrowed = &window; window.values[0] }",
        ),
        (
            "container construction",
            "fn bad<T>(value: T) -> Window<T> { Window { values: [value, value, value] } }",
        ),
        (
            "immutable local write",
            "fn bad<T>(window: Window<T>, value: T) -> Window<T> { let copy: Window<T> = window; copy.values[0] = value; copy }",
        ),
        (
            "parameter write",
            "fn bad<T>(window: Window<T>, value: T) -> Window<T> { window.values[0] = value; window }",
        ),
        (
            "mismatched leaf write",
            "fn bad<T>(window: Window<T>) -> Window<T> { let mut copy: Window<T> = window; copy.values[0] = 1; copy }",
        ),
        (
            "generic-to-generic call",
            "fn get<T>(window: Window<T>) -> T { window.values[0] } fn bad<T>(window: Window<T>) -> T { get(window) }",
        ),
    ];
    for (label, declarations) in cases {
        let source = format!(
            "struct Window<T> {{ values: [T; 3] }} {declarations} fn main() -> int {{ 0 }}"
        );
        let parsed = parsed(&source);
        assert!(!parsed.is_empty(), "{label} fixture must parse");
        assert_shared_rejection(&source, "generic function `");
    }
}
