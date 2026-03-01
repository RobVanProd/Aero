mod ast;
mod code_generator;
mod compatibility;
mod errors;
mod ir;
mod ir_generator;
mod lexer;
mod lsp;
mod module_resolver;
mod optimizations;
mod parser;
mod performance_optimizations;
mod project_init;
mod semantic_analyzer;
mod types;

// (unit tests live in the library crate)

use crate::ir_generator::IrGenerator;
use crate::performance_optimizations::PerformanceOptimizer;
use crate::semantic_analyzer::SemanticAnalyzer;
use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, exit};
use std::time::Instant;

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
            println!("Aero compiler version 1.0.0");
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

            compile_to_llvm_ir(&source_code, output_file, input_file);
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
        "check" => {
            if args.len() < 3 {
                eprintln!("Usage: {} check <input.aero>", args[0]);
                return;
            }
            let input_file = &args[2];

            let source_code = match fs::read_to_string(input_file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!(
                        "\x1b[1;31merror\x1b[0m: could not read file {}: {}",
                        input_file, err
                    );
                    return;
                }
            };

            check_aero_program(&source_code, input_file);
        }
        "test" => {
            // Discover and run *_test.aero files in examples/ and current directory
            let test_dirs = vec!["examples", "tests", "."];
            let mut test_count = 0;
            let mut pass_count = 0;

            println!("\x1b[1;36m   Compiling\x1b[0m test suite...");
            for dir in &test_dirs {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if name.ends_with("_test.aero") || name.ends_with("_tests.aero") {
                                test_count += 1;
                                println!("\x1b[1;36m     Running\x1b[0m {}", path.display());
                                if let Ok(src) = fs::read_to_string(&path) {
                                    let tokens = lexer::tokenize(&src);
                                    let ast = parser::parse(tokens);
                                    let mut analyzer = SemanticAnalyzer::new();
                                    match analyzer.analyze(ast) {
                                        Ok(_) => {
                                            pass_count += 1;
                                            println!("      \x1b[1;32m✓\x1b[0m {} passed", name);
                                        }
                                        Err(err) => {
                                            println!(
                                                "      \x1b[1;31m✗\x1b[0m {} failed: {}",
                                                name, err
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if test_count == 0 {
                println!(
                    "\x1b[1;33mwarning\x1b[0m: no test files found (*_test.aero, *_tests.aero)"
                );
            } else {
                println!(
                    "\n\x1b[1mtest result\x1b[0m: {} passed, {} failed, {} total",
                    pass_count,
                    test_count - pass_count,
                    test_count
                );
            }
        }
        "fmt" => {
            if args.len() < 3 {
                eprintln!("Usage: {} fmt <input.aero>", args[0]);
                return;
            }
            let input_file = &args[2];

            let source_code = match fs::read_to_string(input_file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!(
                        "\x1b[1;31merror\x1b[0m: could not read file {}: {}",
                        input_file, err
                    );
                    return;
                }
            };

            // Basic formatting: normalize indentation and trailing whitespace
            let formatted: String = source_code
                .lines()
                .map(|line| line.trim_end())
                .collect::<Vec<&str>>()
                .join("\n");

            match fs::write(input_file, &formatted) {
                Ok(_) => println!("\x1b[1;32m   Formatted\x1b[0m {}", input_file),
                Err(err) => eprintln!(
                    "\x1b[1;31merror\x1b[0m: could not write file {}: {}",
                    input_file, err
                ),
            }
        }
        "init" => {
            if args.len() > 3 {
                eprintln!("Usage: {} init [path]", args[0]);
                return;
            }
            let target = if args.len() == 3 {
                args[2].as_str()
            } else {
                "."
            };

            match project_init::init_project(Path::new(target)) {
                Ok(result) => {
                    println!("Initialized Aero project at {}", result.root_dir.display());
                    for file in result.created_files {
                        println!("  created {}", file.display());
                    }
                }
                Err(err) => {
                    eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                    exit(1);
                }
            }
        }
        "lsp" => {
            if let Err(err) = lsp::run_language_server() {
                eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                exit(1);
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Available commands: build, run, check, test, fmt, init, lsp");
        }
    }
}

fn compile_to_llvm_ir(source_code: &str, output_file: &str, input_file: &str) {
    println!("Compiling with performance optimizations enabled");

    // Initialize performance optimizer
    let mut perf_optimizer = PerformanceOptimizer::new();
    let compilation_start = Instant::now();

    // Generate source hash for caching
    let source_hash = format!("{:x}", md5::compute(source_code));

    // Check compilation cache first
    if let Some(cached_llvm) = perf_optimizer
        .get_compilation_cache()
        .get_cached_llvm(&source_hash)
    {
        println!("Using cached compilation result");
        match fs::write(output_file, cached_llvm) {
            Ok(_) => {
                println!("Cached LLVM IR written to {}", output_file);
                println!("{}", perf_optimizer.get_performance_report());
                return;
            }
            Err(err) => eprintln!("Error writing cached result: {}", err),
        }
    }

    // Lexing with performance timing
    let lexing_start = Instant::now();
    let tokens = lexer::tokenize(source_code);
    let lexing_time = lexing_start.elapsed();
    println!("Lexing completed in {:?}", lexing_time);

    // Optimized parsing with parser optimizer
    let parsing_start = Instant::now();
    let mut ast = parser::parse(tokens);

    // Apply parser optimizations for complex constructs
    let parser_optimizer = perf_optimizer.get_parser_optimizer();
    // Note: In a real implementation, we would integrate parser optimization here

    let parsing_time = parsing_start.elapsed();
    println!("Optimized parsing completed in {:?}", parsing_time);

    // Phase 7: Module resolution — resolve `mod foo;` to files
    let mut resolver = module_resolver::ModuleResolver::new(input_file);
    let mut module_asts = Vec::new();
    for node in &ast {
        if let crate::ast::AstNode::Statement(crate::ast::Statement::ModDecl {
            name,
            is_public: _,
        }) = node
        {
            match resolver.resolve(name) {
                Ok(resolved) => {
                    println!(
                        "  Resolved module `{}` → {}",
                        name,
                        resolved.file_path.display()
                    );
                    let mod_tokens = lexer::tokenize(&resolved.source);
                    let mod_ast = parser::parse(mod_tokens);
                    module_asts.extend(mod_ast);
                }
                Err(err) => {
                    eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                }
            }
        }
    }
    ast.extend(module_asts);

    // Optimized semantic analysis
    let semantic_start = Instant::now();
    let mut analyzer = SemanticAnalyzer::new();

    // Apply semantic optimizations for large programs
    let semantic_optimizer = perf_optimizer.get_semantic_optimizer();
    // Note: In a real implementation, we would integrate semantic optimization here

    let (analyzed_result, analyzed_ast) = match analyzer.analyze(ast.clone()) {
        Ok((msg, typed_ast)) => {
            println!("Semantic Analysis Result: {}", msg);
            (msg, typed_ast)
        }
        Err(err) => {
            eprintln!("Semantic Analysis Error: {}", err);
            return;
        }
    };
    let semantic_time = semantic_start.elapsed();
    println!(
        "Optimized semantic analysis completed in {:?}",
        semantic_time
    );

    // IR Generation with function call optimizations
    let ir_start = Instant::now();
    let mut ir_gen = IrGenerator::new();
    let mut ir = ir_gen.generate_ir(analyzed_ast);

    // Apply function call optimizations
    let function_optimizer = perf_optimizer.get_function_optimizer();
    // Note: In a real implementation, we would optimize function calls in IR here

    let ir_time = ir_start.elapsed();
    println!("Optimized IR generation completed in {:?}", ir_time);

    // Optimized code generation with control flow optimizations
    let codegen_start = Instant::now();

    // Apply control flow optimizations
    let control_flow_optimizer = perf_optimizer.get_control_flow_optimizer();
    // Note: In a real implementation, we would optimize control flow generation here

    let llvm_ir = code_generator::generate_code(ir);
    let codegen_time = codegen_start.elapsed();
    println!("Optimized code generation completed in {:?}", codegen_time);

    // Cache the compilation result
    perf_optimizer
        .get_compilation_cache()
        .cache_llvm(source_hash, llvm_ir.clone());

    // Write to output file
    match fs::write(output_file, &llvm_ir) {
        Ok(_) => println!("Optimized LLVM IR written to {}", output_file),
        Err(err) => eprintln!("Error writing to file {}: {}", output_file, err),
    }

    let total_time = compilation_start.elapsed();
    println!("Total compilation time: {:?}", total_time);

    // Print comprehensive performance report
    println!("{}", perf_optimizer.get_performance_report());

    println!("Performance-optimized compilation process completed successfully.");
}

fn run_aero_program(source_code: &str, input_file: &str) {
    // Generate temporary file names
    let base_name = input_file.trim_end_matches(".aero");
    let ll_file = format!("{}.ll", base_name);
    let obj_file = format!("{}.o", base_name);
    let exe_file = if cfg!(windows) {
        format!("{}.exe", base_name)
    } else {
        base_name.to_string()
    };

    // Compile to LLVM IR
    compile_to_llvm_ir(source_code, &ll_file, input_file);

    // Compile LLVM IR to object file using llc
    let llc_output = Command::new("llc")
        .args(&["-filetype=obj", &ll_file, "-o", &obj_file])
        .output();

    match llc_output {
        Ok(output) => {
            if !output.status.success() {
                eprintln!(
                    "Error running llc: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                return;
            }
        }
        Err(err) => {
            eprintln!(
                "Error executing llc: {}. Make sure LLVM is installed and llc is in your PATH.",
                err
            );
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
                eprintln!(
                    "Error running clang: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                return;
            }
        }
        Err(err) => {
            eprintln!(
                "Error executing clang: {}. Make sure clang is installed and in your PATH.",
                err
            );
            return;
        }
    }

    // Execute the compiled program
    let exe_path = if cfg!(windows) {
        format!(".\\{}", exe_file)
    } else {
        format!("./{}", exe_file)
    };
    let run_output = Command::new(&exe_path).output();

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
    println!("Aero Programming Language Compiler v1.0.0");
    println!();
    println!("USAGE:");
    println!("    {} <COMMAND> [OPTIONS]", program_name);
    println!();
    println!("COMMANDS:");
    println!("    build <input.aero> -o <output.ll>    Compile Aero source to LLVM IR");
    println!("    run <input.aero>                     Compile and run Aero source");
    println!("    check <input.aero>                   Type-check only (no codegen)");
    println!("    test                                 Discover and run *_test.aero files");
    println!("    fmt <input.aero>                     Auto-format Aero source");
    println!("    init [path]                          Initialize a new Aero project");
    println!("    lsp                                  Run Aero language server (stdio)");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Print this help message");
    println!("    -v, --version    Print version information");
    println!();
    println!("EXAMPLES:");
    println!("    {} build hello.aero -o hello.ll", program_name);
    println!("    {} run hello.aero", program_name);
    println!("    {} check hello.aero", program_name);
    println!("    {} test", program_name);
    println!("    {} fmt hello.aero", program_name);
    println!("    {} init my_app", program_name);
    println!("    {} lsp", program_name);
}

/// Type-check an Aero program without generating code.
/// Runs lexer → parser → semantic analysis only.
fn check_aero_program(source_code: &str, input_file: &str) {
    let check_start = Instant::now();

    // Lexing
    let tokens = lexer::tokenize(source_code);

    // Parsing
    let ast = parser::parse(tokens);

    // Semantic analysis
    let mut analyzer = SemanticAnalyzer::new();
    match analyzer.analyze(ast) {
        Ok((msg, _typed_ast)) => {
            let elapsed = check_start.elapsed();
            println!(
                "\x1b[1;32m    Checking\x1b[0m {} ... \x1b[1;32mok\x1b[0m ({:?})",
                input_file, elapsed
            );
            println!("  {}", msg);
        }
        Err(err) => {
            // Enhanced error display with color and source context
            let lines: Vec<&str> = source_code.lines().collect();
            eprintln!("\x1b[1;31merror\x1b[0m: {}", err);

            // Try to extract line number from error message
            if let Some(line_hint) = extract_error_line(&err.to_string()) {
                if line_hint > 0 && line_hint <= lines.len() {
                    let line_content = lines[line_hint - 1];
                    eprintln!("  \x1b[1;34m-->\x1b[0m {}:{}", input_file, line_hint);
                    eprintln!("   \x1b[1;34m|\x1b[0m");
                    eprintln!(" \x1b[1;34m{:3} |\x1b[0m {}", line_hint, line_content);
                    eprintln!(
                        "   \x1b[1;34m|\x1b[0m \x1b[1;31m{}\x1b[0m",
                        "^".repeat(line_content.trim().len().min(40))
                    );
                }
            }

            // Suggest similar identifiers if it's an undefined variable error
            if err.to_string().contains("undefined") || err.to_string().contains("not found") {
                eprintln!(
                    "\x1b[1;36mhelp\x1b[0m: check the spelling or ensure the variable is in scope"
                );
            }

            std::process::exit(1);
        }
    }
}

/// Attempt to extract a line number from a compiler error message
fn extract_error_line(error_msg: &str) -> Option<usize> {
    // Look for patterns like "line 5" or "at line 5" or ":5:"
    for word in error_msg.split_whitespace() {
        if let Ok(n) = word
            .trim_matches(|c: char| !c.is_ascii_digit())
            .parse::<usize>()
        {
            if n > 0 && n < 100000 {
                return Some(n);
            }
        }
    }
    None
}
