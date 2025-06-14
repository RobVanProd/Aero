mod lexer;
mod parser;
mod semantic_analyzer;
mod ir_generator;
mod code_generator;

fn main() {
    let source_code = "fn main() { let x = 10; } ";
    println!("Compiling: \"{}\"", source_code);

    // Lexing
    let tokens = lexer::tokenize(source_code);
    println!("Tokens: {:?}", tokens);

    // Parsing
    let ast = parser::parse(tokens);
    println!("AST: {}", ast);

    // Semantic Analysis
    let analyzed_ast = semantic_analyzer::analyze_semantic(ast);
    println!("Semantic Analysis Result: {}", analyzed_ast);

    // IR Generation
    let ir = ir_generator::generate_ir(analyzed_ast);
    println!("IR: {}", ir);

    // Code Generation
    let llvm_ir = code_generator::generate_code(ir);
    println!("LLVM IR: {}", llvm_ir);

    println!("Compilation process simulated successfully.");
}


