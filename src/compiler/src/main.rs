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
use std::process::{Command, exit};

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_help(&args[0]);
        return;
    }

    let command = &args[1];
    
    match command.as_str() {
        "--help" | "-h" => {
            print_help(&args[0]);
            return;
        }
        "--version" | "-v" => {
            println!("Aero compiler version 0.1.0");
            return;
        }
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
            
            run_aero_program(&source_code, input_file);
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

fn run_aero_program(source_code: &str, input_file: &str) {
    // Generate temporary file names
    let base_name = input_file.trim_end_matches(".aero");
    let ll_file = format!("{}.ll", base_name);
    let obj_file = format!("{}.o", base_name);
    let exe_file = if cfg!(windows) { format!("{}.exe", base_name) } else { base_name.to_string() };
    
    // Compile to LLVM IR
    compile_to_llvm_ir(source_code, &ll_file);
    
    // Compile LLVM IR to object file using llc
    let llc_output = Command::new("llc")
        .args(&["-filetype=obj", &ll_file, "-o", &obj_file])
        .output();
    
    match llc_output {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("Error running llc: {}", String::from_utf8_lossy(&output.stderr));
                return;
            }
        }
        Err(err) => {
            eprintln!("Error executing llc: {}. Make sure LLVM is installed and llc is in your PATH.", err);
            return;
        }
    }
    
    // Link object file to executable using clang
    let clang_output = Command::new("clang")
        .args(&[&obj_file, "-o", &exe_file])
        .output();
    
    match clang_output {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("Error running clang: {}", String::from_utf8_lossy(&output.stderr));
                return;
            }
        }
        Err(err) => {
            eprintln!("Error executing clang: {}. Make sure clang is installed and in your PATH.", err);
            return;
        }
    }
    
    // Execute the compiled program
    let exe_path = if cfg!(windows) { format!(".\\{}", exe_file) } else { format!("./{}", exe_file) };
    let run_output = Command::new(&exe_path)
        .output();
    
    match run_output {
        Ok(output) => {
            let exit_code = output.status.code().unwrap_or(-1);
            println!("Program executed successfully.");
            println!("Exit code: {}", exit_code);
            
            if !output.stdout.is_empty() {
                println!("Output: {}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                println!("Error output: {}", String::from_utf8_lossy(&output.stderr));
            }
            
            // Exit with the same code as the executed program
            exit(exit_code);
        }
        Err(err) => {
            eprintln!("Error executing compiled program: {}", err);
        }
    }
    
    // Clean up temporary files
    let _ = fs::remove_file(&ll_file);
    let _ = fs::remove_file(&obj_file);
    let _ = fs::remove_file(&exe_file);
}

fn print_help(program_name: &str) {
    println!("Aero Programming Language Compiler v0.1.0");
    println!();
    println!("USAGE:");
    println!("    {} <COMMAND> [OPTIONS]", program_name);
    println!();
    println!("COMMANDS:");
    println!("    build <input.aero> -o <output.ll>    Compile Aero source to LLVM IR");
    println!("    run <input.aero>                     Compile and run Aero source");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Print this help message");
    println!("    -v, --version    Print version information");
    println!();
    println!("EXAMPLES:");
    println!("    {} build hello.aero -o hello.ll", program_name);
    println!("    {} run hello.aero", program_name);
}


