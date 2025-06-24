mod ast;
mod lexer;
mod parser;
mod semantic_analyzer;
mod ir;
mod ir_generator;
mod code_generator;
mod types;

use crate::semantic_analyzer::SemanticAnalyzer;
use crate::ir_generator::IrGenerator;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <command> [options]", args[0]);
        eprintln!("Commands:");
        eprintln!("  build <input.aero> -o <output.ll>  Compile Aero source to LLVM IR");
        eprintln!("  run <input.aero>                   Compile and run Aero source");
        return;
    }

    let command = &args[1];
    
    match command.as_str() {
        "build" => {
            if args.len() < 5 || args[3] != "-o" {
                eprintln!("Usage: {} build <input.aero> -o <output.ll>", args[0]);
                return;
            }
            let input_file = &args[2];
            let output_file = &args[4];
            
            let source_code = match fs::read_to_string(input_file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Error reading file {}: {}", input_file, err);
                    return;
                }
            };
            
            compile_to_llvm_ir(&source_code, output_file);
        }
        "run" => {
            if args.len() < 3 {
                eprintln!("Usage: {} run <input.aero>", args[0]);
                return;
            }
            let input_file = &args[2];
            
            let source_code = match fs::read_to_string(input_file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Error reading file {}: {}", input_file, err);
                    return;
                }
            };
            
            // For now, just compile to a temporary file
            let temp_output = "temp_output.ll";
            compile_to_llvm_ir(&source_code, temp_output);
            println!("Compiled to {}", temp_output);
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Available commands: build, run");
        }
    }
}

fn compile_to_llvm_ir(source_code: &str, output_file: &str) {
    println!("Compiling: \"{}\"", source_code);

    // Lexing
    let tokens = lexer::tokenize(source_code);
    println!("Tokens: {:?}", tokens);

    // Parsing
    let ast = parser::parse(tokens);
    println!("AST: {:?}", ast);

    // Semantic Analysis
    let mut analyzer = SemanticAnalyzer::new();
    let (analyzed_result, analyzed_ast) = match analyzer.analyze(ast.clone()) {
        Ok((msg, typed_ast)) => {
            println!("Semantic Analysis Result: {}", msg);
            (msg, typed_ast)
        },
        Err(err) => {
            eprintln!("Semantic Analysis Error: {}", err);
            return;
        }
    };

    // IR Generation
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate_ir(analyzed_ast);
    println!("IR: {:?}", ir);

    // Code Generation
    let llvm_ir = code_generator::generate_code(ir);
    println!("LLVM IR:\n{}", llvm_ir);

    // Write to output file
    match fs::write(output_file, &llvm_ir) {
        Ok(_) => println!("LLVM IR written to {}", output_file),
        Err(err) => eprintln!("Error writing to file {}: {}", output_file, err),
    }

    println!("Compilation process completed successfully.");
}


