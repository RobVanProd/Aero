use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};

const PROJECTED_CALL_LOAN_PRODUCT: &str = r#"
struct Cell {
    reading: int,
}

struct Telemetry {
    channels: [Cell; 2],
}

fn observe(reading: &int) -> int {
    *reading
}

fn adjust(reading: &mut int, delta: int) -> int {
    *reading = *reading + delta;
    *reading
}

fn main() -> int {
    let mut telemetry = Telemetry {
        channels: [Cell { reading: 10 }, Cell { reading: 20 }],
    };
    let index = 1;
    let before = observe(&telemetry.channels[index].reading);
    let after = adjust(&mut telemetry.channels[index].reading, 3);
    if before == 20 && after == 23 && telemetry.channels[1].reading == 23 {
        return 91;
    }
    1
}
"#;

const PROJECTED_CALL_LOAN_MATRIX: &str = r#"
struct Leaf { value: int, ready: bool }
struct State {
    leaves: [Leaf; 2],
    pair: (int, bool),
    grid: [[int; 2]; 2],
}

fn read_int(value: &int) -> int { *value }
fn read_two(left: &int, right: &int) -> int { *left + *right }
fn set_int(value: &mut int, replacement: int) -> int {
    *value = replacement;
    *value
}
fn set_two(left: &mut int, right: &mut int) -> int {
    *left = *left + 1;
    *right = *right + 2;
    *left + *right
}
fn read_and_set(observed: &int, changed: &mut int) -> int {
    *changed = *changed + *observed;
    *changed
}
fn replace_leaf(value: &mut Leaf, replacement: int) -> int {
    *value = Leaf { value: replacement, ready: replacement > 0 };
    (*value).value
}
fn replace_pair(value: &mut (int, bool), replacement: int) -> int {
    *value = (replacement, replacement > 0);
    (*value).0
}
fn replace_array(value: &mut [int; 2], replacement: int) -> int {
    *value = [replacement, replacement + 1];
    (*value)[1]
}
fn read_parameter(value: State, index: int) -> int {
    read_int(&value.leaves[index].value)
}

fn main() -> int {
    let mut left = State {
        leaves: [Leaf { value: 1, ready: 1 < 2 }, Leaf { value: 2, ready: 1 < 2 }],
        pair: (3, 1 < 2),
        grid: [[4, 5], [6, 7]],
    };
    let mut right = State {
        leaves: [Leaf { value: 8, ready: 1 < 2 }, Leaf { value: 9, ready: 1 < 2 }],
        pair: (10, 1 < 2),
        grid: [[11, 12], [13, 14]],
    };
    let index = 1;
    let shared = read_two(&left.leaves[0].value, &left.leaves[index].value);
    let changed = set_two(&mut left.leaves[index].value, &mut right.grid[0][index]);
    let mixed = read_and_set(&left.grid[1][0], &mut right.leaves[0].value);
    let leaf = replace_leaf(&mut left.leaves[0], 15);
    let pair = replace_pair(&mut left.pair, 16);
    let array = replace_array(&mut right.grid[index], 17);
    let scalar = set_int(&mut left.grid[index][0], 19);
    let observed = read_int(&right.grid[1][1]);
    let parameter_observed = read_parameter(right, 0);
    if shared == 3 && changed == 17 && mixed == 14 && leaf == 15
        && pair == 16 && array == 18 && scalar == 19 && observed == 18
        && parameter_observed == 14 && left.leaves[0].value == 15
        && right.leaves[0].value == 14 {
        return 92;
    }
    2
}
"#;

fn parsed() -> Vec<compiler::ast::AstNode> {
    let tokens = try_tokenize_with_locations(PROJECTED_CALL_LOAN_PRODUCT, None)
        .expect("projected call-loan product must lex");
    parse_with_locations(tokens).expect("projected call-loan product must parse")
}

fn parse_source(source: &str) -> Vec<compiler::ast::AstNode> {
    let tokens = try_tokenize_with_locations(source, None).expect("fixture must lex");
    parse_with_locations(tokens).expect("fixture must parse")
}

fn assert_shared_rejection(source: &str, diagnostic: &str) {
    let syntax = parse_source(source);
    let semantic = SemanticAnalyzer::new()
        .analyze(syntax.clone())
        .expect_err("excluded projected call-loan source must fail semantics");
    let checked = IrGenerator::new()
        .try_generate_ir(syntax)
        .expect_err("excluded projected call-loan source must fail raw checked admission")
        .to_string();
    let public = compile_program(source, CompilerOptions::default())
        .expect_err("excluded projected call-loan source must fail public compilation");
    for (route, error) in [
        ("semantic", semantic),
        ("raw checked", checked),
        ("public", public),
    ] {
        assert!(
            error.contains(diagnostic),
            "{route} rejection `{error}` did not contain `{diagnostic}`"
        );
    }
}

#[test]
fn projected_copydata_call_loans_execute_end_to_end() {
    let syntax = parsed();
    let mut failures = Vec::new();

    if let Err(error) = SemanticAnalyzer::new().analyze(syntax.clone()) {
        failures.push(format!(
            "semantic analysis rejected projected call loans: {error}"
        ));
    }
    if let Err(error) = IrGenerator::new().try_generate_ir(syntax) {
        failures.push(format!(
            "semantic-independent checked admission rejected projected call loans: {error}"
        ));
    }
    if let Err(error) = compile_program(PROJECTED_CALL_LOAN_PRODUCT, CompilerOptions::default()) {
        failures.push(format!(
            "public compilation rejected projected call loans: {error}"
        ));
    }

    assert!(
        failures.is_empty(),
        "CAP-012 projected call-loan failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn projected_call_loans_cover_the_recursive_copydata_path_and_signature_class() {
    let syntax = parse_source(PROJECTED_CALL_LOAN_MATRIX);
    let (_, analyzed) = SemanticAnalyzer::new()
        .analyze(syntax.clone())
        .expect("complete projected call-loan matrix must pass semantics");
    let checked = IrGenerator::new()
        .try_generate_ir(syntax)
        .expect("raw checked admission must accept the complete projected call-loan matrix");
    let direct = CodeGenerator::new()
        .try_generate_code(checked)
        .expect("independently verified projected call-loan IR must lower");
    let public = compile_program(PROJECTED_CALL_LOAN_MATRIX, CompilerOptions::default())
        .expect("complete projected call-loan matrix must compile publicly");
    assert!(!analyzed.is_empty());
    assert_eq!(direct, public, "semantic and raw checked routes drifted");
    assert!(
        public.matches("getelementptr inbounds").count() >= 20,
        "projected path lowering omitted typed addresses:\n{public}"
    );
}

#[test]
fn projected_call_loan_exclusions_fail_closed_in_every_trusted_route() {
    let cases = [
        (
            "immutable root mutable loan",
            "struct Row { value: int } fn set(value: &mut int) { *value = 2; } fn main() -> int { let row = Row { value: 1 }; set(&mut row.value); 0 }",
            "must be a mutable local owned binding",
        ),
        (
            "leaf mismatch",
            "struct Row { value: int } fn read(value: &bool) -> bool { *value } fn main() -> int { let row = Row { value: 1 }; if read(&row.value) { return 1; } 0 }",
            "place type mismatch",
        ),
        (
            "same-root mutable projections",
            "fn both(left: &mut int, right: &mut int) -> int { *left + *right } fn main() -> int { let mut pair = (1, 2); both(&mut pair.0, &mut pair.1) }",
            "pairwise-distinct source identities",
        ),
        (
            "mutable and immutable overlap",
            "fn mixed(changed: &mut int, seen: &int) -> int { *changed + *seen } fn main() -> int { let mut pair = (1, 2); mixed(&mut pair.0, &pair.1) }",
            "non-mutable arguments must be independent",
        ),
        (
            "side argument overlap",
            "struct Row { value: int } fn set(value: &mut int, amount: int) -> int { *value + amount } fn main() -> int { let mut row = Row { value: 1 }; set(&mut row.value, row.value) }",
            "non-mutable arguments must be independent",
        ),
        (
            "non-int runtime selector",
            "fn read(value: &int) -> int { *value } fn main() -> int { let values = [1, 2]; let index = 1 < 2; read(&values[index]) }",
            "array selector type mismatch",
        ),
        (
            "negative constant selector",
            "fn read(value: &int) -> int { *value } fn main() -> int { let values = [1, 2]; read(&values[-1]) }",
            "indexes require a nonnegative integer",
        ),
        (
            "upper constant selector",
            "fn read(value: &int) -> int { *value } fn main() -> int { let values = [1, 2]; read(&values[2]) }",
            "outside 0..2",
        ),
        (
            "temporary root",
            "struct Row { value: int } fn make() -> Row { Row { value: 1 } } fn read(value: &int) -> int { *value } fn main() -> int { read(&make().value) }",
            "requires a direct local identifier root",
        ),
        (
            "stored projected alias",
            "struct Row { value: int } fn main() -> int { let row = Row { value: 1 }; let alias = &row.value; *alias }",
            "requires an identifier place",
        ),
    ];

    for (label, source, diagnostic) in cases {
        let parsed = parse_source(source);
        assert!(!parsed.is_empty(), "{label} fixture must parse");
        assert_shared_rejection(source, diagnostic);
    }
}
