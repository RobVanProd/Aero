use compiler::lexer;
use compiler::parser;
use compiler::semantic_analyzer::SemanticAnalyzer;
use insta::assert_debug_snapshot;

#[test]
fn test_lexer_simple_let() {
    let source = "let x = 10;";
    let tokens = lexer::tokenize(source);
    assert_debug_snapshot!(tokens);
}

#[test]
fn test_lexer_binary_expression() {
    let source = "let y = x + 5.0;";
    let tokens = lexer::tokenize(source);
    assert_debug_snapshot!(tokens);
}

#[test]
fn test_parser_simple_let() {
    let source = "let x = 10;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_debug_snapshot!(ast);
}

#[test]
fn test_parser_binary_expression() {
    let source = "let y = x + 5.0;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_debug_snapshot!(ast);
}

#[test]
fn test_semantic_simple_let() {
    let source = "let x = 10;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert_debug_snapshot!(result);
}

#[test]
fn test_semantic_binary_expression() {
    let source = "let x = 10; let y = x + 5.0;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert_debug_snapshot!(result);
}

#[test]
fn test_semantic_redeclaration() {
    let source = "let x = 10; let x = 20;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert_debug_snapshot!(result);
}

#[test]
fn test_semantic_undeclared_variable() {
    let source = "let y = x + 10;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert_debug_snapshot!(result);
}
