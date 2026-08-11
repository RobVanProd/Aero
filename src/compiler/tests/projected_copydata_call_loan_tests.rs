use compiler::{
    CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program, parse_with_locations,
    try_tokenize_with_locations,
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

fn parsed() -> Vec<compiler::ast::AstNode> {
    let tokens = try_tokenize_with_locations(PROJECTED_CALL_LOAN_PRODUCT, None)
        .expect("projected call-loan product must lex");
    parse_with_locations(tokens).expect("projected call-loan product must parse")
}

#[test]
fn projected_copydata_call_loans_execute_end_to_end() {
    let syntax = parsed();
    let mut failures = Vec::new();

    if let Err(error) = SemanticAnalyzer::new().analyze(syntax.clone()) {
        failures.push(format!("semantic analysis rejected projected call loans: {error}"));
    }
    if let Err(error) = IrGenerator::new().try_generate_ir(syntax) {
        failures.push(format!(
            "semantic-independent checked admission rejected projected call loans: {error}"
        ));
    }
    if let Err(error) = compile_program(PROJECTED_CALL_LOAN_PRODUCT, CompilerOptions::default()) {
        failures.push(format!("public compilation rejected projected call loans: {error}"));
    }

    assert!(
        failures.is_empty(),
        "CAP-012 projected call-loan failures:\n{}",
        failures.join("\n")
    );
}
