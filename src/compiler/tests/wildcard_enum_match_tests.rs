use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};

const TERMINAL_WILDCARD: &str = r#"
enum Phase { Cold, Warm, Hot }

fn score(value: Phase) -> int {
    match value {
        Phase::Hot => 40,
        _ => 2,
    }
}

fn main() -> int {
    score(Phase::Warm)
}
"#;

const PAYLOAD_WILDCARD: &str = r#"
enum Outcome { Ready(int), Failed(char) }

fn score(value: Outcome) -> int {
    match value {
        Outcome::Ready(number) => number,
        Outcome::Failed(_) => 0,
    }
}

fn main() -> int {
    score(Outcome::Failed('e'))
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
fn terminal_and_payload_wildcards_reach_both_trusted_routes() {
    let mut failures = Vec::new();

    for (label, source) in [
        ("terminal wildcard", TERMINAL_WILDCARD),
        ("payload wildcard", PAYLOAD_WILDCARD),
    ] {
        if let Err(error) = semantic(source) {
            failures.push(format!("{label}: semantic analysis rejected parsed wildcard: {error}"));
        }

        match checked_without_semantics(source) {
            Err(error) => failures.push(format!(
                "{label}: independent checked admission rejected parsed wildcard: {error}"
            )),
            Ok(checked) => {
                if let Err(error) = CodeGenerator::new().try_generate_code(checked) {
                    failures.push(format!("{label}: trusted lowering rejected wildcard IR: {error}"));
                }
            }
        }

        if let Err(error) = compile_program(source, CompilerOptions::default()) {
            failures.push(format!("{label}: public compilation rejected wildcard: {error}"));
        }
    }

    assert!(
        failures.is_empty(),
        "CAP-008 wildcard enum Match red:\n{}",
        failures.join("\n\n")
    );
}
