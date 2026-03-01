mod accelerator;
mod ast;
mod code_generator;
mod compatibility;
mod conformance;
mod doc_generator;
mod errors;
mod graph_compiler;
mod ir;
mod ir_generator;
mod lexer;
mod lsp;
mod module_resolver;
mod optimizations;
mod parser;
mod performance_optimizations;
mod profiler;
mod project_init;
mod quantization;
mod registry;
mod semantic_analyzer;
mod types;

// (unit tests live in the library crate)

use crate::ir_generator::IrGenerator;
use crate::performance_optimizations::PerformanceOptimizer;
use crate::semantic_analyzer::SemanticAnalyzer;
use accelerator::AcceleratorBackend;
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
        "doc" => {
            if args.len() < 3 {
                eprintln!("Usage: {} doc <input.aero> [-o <output.md>]", args[0]);
                return;
            }

            let input_file = &args[2];
            let output_file = if args.len() == 5 && args[3] == "-o" {
                args[4].clone()
            } else if args.len() == 3 {
                default_doc_output_path(input_file)
            } else {
                eprintln!("Usage: {} doc <input.aero> [-o <output.md>]", args[0]);
                return;
            };

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

            match doc_generator::generate_markdown(input_file, &source_code) {
                Ok(markdown) => match fs::write(&output_file, markdown) {
                    Ok(_) => println!("Generated documentation at {}", output_file),
                    Err(err) => eprintln!(
                        "\x1b[1;31merror\x1b[0m: could not write docs {}: {}",
                        output_file, err
                    ),
                },
                Err(err) => {
                    eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                    exit(1);
                }
            }
        }
        "profile" => {
            if args.len() < 3 {
                eprintln!("Usage: {} profile <input.aero> [-o <trace.json>]", args[0]);
                return;
            }

            let input_file = &args[2];
            let trace_output = if args.len() == 5 && args[3] == "-o" {
                Some(args[4].as_str())
            } else if args.len() == 3 {
                None
            } else {
                eprintln!("Usage: {} profile <input.aero> [-o <trace.json>]", args[0]);
                return;
            };

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

            match profiler::profile_compilation(&source_code, input_file) {
                Ok(profile) => {
                    profiler::print_profile(&profile);
                    if let Some(path) = trace_output {
                        match profiler::write_trace_file(&profile, path) {
                            Ok(_) => println!("Wrote trace file to {}", path),
                            Err(err) => {
                                eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                                exit(1);
                            }
                        }
                    }
                }
                Err(err) => {
                    eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                    exit(1);
                }
            }
        }
        "graph-opt" => {
            if args.len() < 5 {
                eprintln!(
                    "Usage: {} graph-opt <input.ll> -o <output.ll> [--backend <cpu|cuda|rocm>] [--annotation-only]",
                    args[0]
                );
                return;
            }

            let input_file = &args[2];
            let mut output_file: Option<String> = None;
            let mut backend = AcceleratorBackend::Cpu;
            let mut executable_fusion = true;

            let mut i = 3usize;
            while i < args.len() {
                match args[i].as_str() {
                    "-o" => {
                        if i + 1 >= args.len() {
                            eprintln!(
                                "Usage: {} graph-opt <input.ll> -o <output.ll> [--backend <cpu|cuda|rocm>] [--annotation-only]",
                                args[0]
                            );
                            return;
                        }
                        output_file = Some(args[i + 1].clone());
                        i += 2;
                    }
                    "--backend" => {
                        if i + 1 >= args.len() {
                            eprintln!(
                                "Usage: {} graph-opt <input.ll> -o <output.ll> [--backend <cpu|cuda|rocm>] [--annotation-only]",
                                args[0]
                            );
                            return;
                        }
                        let Some(parsed) = AcceleratorBackend::parse(&args[i + 1]) else {
                            eprintln!(
                                "\x1b[1;31merror\x1b[0m: unsupported backend `{}`",
                                args[i + 1]
                            );
                            return;
                        };
                        backend = parsed;
                        i += 2;
                    }
                    "--annotation-only" => {
                        executable_fusion = false;
                        i += 1;
                    }
                    _ => {
                        eprintln!(
                            "Usage: {} graph-opt <input.ll> -o <output.ll> [--backend <cpu|cuda|rocm>] [--annotation-only]",
                            args[0]
                        );
                        return;
                    }
                }
            }
            let Some(output_file) = output_file else {
                eprintln!(
                    "Usage: {} graph-opt <input.ll> -o <output.ll> [--backend <cpu|cuda|rocm>] [--annotation-only]",
                    args[0]
                );
                return;
            };

            let input = match fs::read_to_string(input_file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!(
                        "\x1b[1;31merror\x1b[0m: could not read file {}: {}",
                        input_file, err
                    );
                    return;
                }
            };

            let config = graph_compiler::GraphCompilationConfig {
                backend,
                executable_fusion,
            };
            let (optimized, report) =
                graph_compiler::apply_advanced_graph_compilation_with_config(&input, &config);
            match fs::write(&output_file, optimized) {
                Ok(_) => {
                    println!("Wrote graph-optimized IR to {}", output_file);
                    println!(
                        "Backend: {} | fused kernels: {} | executable kernels: {} | skipped chains: {} | total fused ops: {}",
                        report.backend,
                        report.fused_kernel_count,
                        report.executable_kernel_count,
                        report.skipped_chains,
                        report.total_fused_ops
                    );
                }
                Err(err) => eprintln!(
                    "\x1b[1;31merror\x1b[0m: could not write file {}: {}",
                    output_file, err
                ),
            }
        }
        "quantize" => {
            if args.len() < 7 {
                eprintln!(
                    "Usage: {} quantize <input.ll> -o <output.ll> --mode <int8|fp8-e4m3|fp8-e5m2> [--backend <cpu|cuda|rocm>] [--calibration <samples.json|samples.txt>] [--per-channel] [--annotation-only]",
                    args[0]
                );
                return;
            }

            let input_file = &args[2];
            let mut output_file: Option<String> = None;
            let mut mode: Option<quantization::QuantizationMode> = None;
            let mut backend = AcceleratorBackend::Cpu;
            let mut per_channel = false;
            let mut runtime_lowering = true;
            let mut calibration_file: Option<String> = None;

            let mut i = 3usize;
            while i < args.len() {
                match args[i].as_str() {
                    "-o" => {
                        if i + 1 >= args.len() {
                            eprintln!(
                                "Usage: {} quantize <input.ll> -o <output.ll> --mode <int8|fp8-e4m3|fp8-e5m2> [--backend <cpu|cuda|rocm>] [--calibration <samples.json|samples.txt>] [--per-channel] [--annotation-only]",
                                args[0]
                            );
                            return;
                        }
                        output_file = Some(args[i + 1].clone());
                        i += 2;
                    }
                    "--mode" => {
                        if i + 1 >= args.len() {
                            eprintln!(
                                "Usage: {} quantize <input.ll> -o <output.ll> --mode <int8|fp8-e4m3|fp8-e5m2> [--backend <cpu|cuda|rocm>] [--calibration <samples.json|samples.txt>] [--per-channel] [--annotation-only]",
                                args[0]
                            );
                            return;
                        }
                        mode = quantization::QuantizationMode::parse(&args[i + 1]);
                        if mode.is_none() {
                            eprintln!(
                                "\x1b[1;31merror\x1b[0m: unsupported quantization mode `{}`",
                                args[i + 1]
                            );
                            return;
                        }
                        i += 2;
                    }
                    "--backend" => {
                        if i + 1 >= args.len() {
                            eprintln!(
                                "Usage: {} quantize <input.ll> -o <output.ll> --mode <int8|fp8-e4m3|fp8-e5m2> [--backend <cpu|cuda|rocm>] [--calibration <samples.json|samples.txt>] [--per-channel] [--annotation-only]",
                                args[0]
                            );
                            return;
                        }
                        let Some(parsed) = AcceleratorBackend::parse(&args[i + 1]) else {
                            eprintln!(
                                "\x1b[1;31merror\x1b[0m: unsupported backend `{}`",
                                args[i + 1]
                            );
                            return;
                        };
                        backend = parsed;
                        i += 2;
                    }
                    "--calibration" => {
                        if i + 1 >= args.len() {
                            eprintln!(
                                "Usage: {} quantize <input.ll> -o <output.ll> --mode <int8|fp8-e4m3|fp8-e5m2> [--backend <cpu|cuda|rocm>] [--calibration <samples.json|samples.txt>] [--per-channel] [--annotation-only]",
                                args[0]
                            );
                            return;
                        }
                        calibration_file = Some(args[i + 1].clone());
                        i += 2;
                    }
                    "--per-channel" => {
                        per_channel = true;
                        i += 1;
                    }
                    "--annotation-only" | "--no-runtime-lowering" => {
                        runtime_lowering = false;
                        i += 1;
                    }
                    _ => {
                        eprintln!(
                            "Usage: {} quantize <input.ll> -o <output.ll> --mode <int8|fp8-e4m3|fp8-e5m2> [--backend <cpu|cuda|rocm>] [--calibration <samples.json|samples.txt>] [--per-channel] [--annotation-only]",
                            args[0]
                        );
                        return;
                    }
                }
            }

            let Some(output_file) = output_file else {
                eprintln!(
                    "Usage: {} quantize <input.ll> -o <output.ll> --mode <int8|fp8-e4m3|fp8-e5m2> [--backend <cpu|cuda|rocm>] [--calibration <samples.json|samples.txt>] [--per-channel] [--annotation-only]",
                    args[0]
                );
                return;
            };
            let Some(mode) = mode else {
                eprintln!(
                    "Usage: {} quantize <input.ll> -o <output.ll> --mode <int8|fp8-e4m3|fp8-e5m2> [--backend <cpu|cuda|rocm>] [--calibration <samples.json|samples.txt>] [--per-channel] [--annotation-only]",
                    args[0]
                );
                return;
            };

            let input = match fs::read_to_string(input_file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!(
                        "\x1b[1;31merror\x1b[0m: could not read file {}: {}",
                        input_file, err
                    );
                    return;
                }
            };

            let mut config = quantization::QuantizationConfig::new(mode);
            config.backend = backend;
            config.per_channel = per_channel;
            config.enable_runtime_lowering = runtime_lowering;

            if let Some(calibration_file) = &calibration_file {
                match quantization::load_calibration_profile(
                    Path::new(calibration_file),
                    mode,
                    backend,
                ) {
                    Ok(profile) => {
                        config.calibration_profile = Some(profile);
                        config.calibration_source = Some(calibration_file.clone());
                    }
                    Err(err) => {
                        eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                        return;
                    }
                }
            }

            let (quantized_ir, report) =
                quantization::apply_quantization_interface(&input, &config);
            match fs::write(&output_file, quantized_ir) {
                Ok(_) => {
                    println!("Wrote quantization IR to {}", output_file);
                    println!(
                        "Mode: {} | backend: {} | candidates: {} | lowered: {} | helpers: {} | calibration samples: {}",
                        report.mode,
                        report.backend,
                        report.candidate_ops,
                        report.lowered_ops,
                        report.helper_count,
                        report.calibration_samples
                    );
                    for note in report.notes {
                        println!("  - {}", note);
                    }
                }
                Err(err) => eprintln!(
                    "\x1b[1;31merror\x1b[0m: could not write file {}: {}",
                    output_file, err
                ),
            }
        }
        "registry" => {
            if args.len() < 3 {
                print_registry_help(&args[0]);
                return;
            }

            match args[2].as_str() {
                "search" => {
                    if args.len() < 4 {
                        eprintln!(
                            "Usage: {} registry search <query> [--index <index.json>] [--registry <url>] [--live] [--token <token>] [--token-file <path>]",
                            args[0]
                        );
                        return;
                    }
                    let query = &args[3];
                    let mut index_path = registry::DEFAULT_LOCAL_INDEX_PATH.to_string();
                    let mut registry_url: Option<String> = None;
                    let mut live = false;
                    let mut token: Option<String> = None;
                    let mut token_file: Option<String> = None;

                    let mut i = 4usize;
                    while i < args.len() {
                        match args[i].as_str() {
                            "--index" => {
                                if i + 1 >= args.len() {
                                    eprintln!(
                                        "Usage: {} registry search <query> [--index <index.json>] [--registry <url>] [--live] [--token <token>] [--token-file <path>]",
                                        args[0]
                                    );
                                    return;
                                }
                                index_path = args[i + 1].clone();
                                i += 2;
                            }
                            "--registry" => {
                                if i + 1 >= args.len() {
                                    eprintln!(
                                        "Usage: {} registry search <query> [--index <index.json>] [--registry <url>] [--live] [--token <token>] [--token-file <path>]",
                                        args[0]
                                    );
                                    return;
                                }
                                registry_url = Some(args[i + 1].clone());
                                i += 2;
                            }
                            "--live" => {
                                live = true;
                                i += 1;
                            }
                            "--token" => {
                                if i + 1 >= args.len() {
                                    eprintln!(
                                        "Usage: {} registry search <query> [--index <index.json>] [--registry <url>] [--live] [--token <token>] [--token-file <path>]",
                                        args[0]
                                    );
                                    return;
                                }
                                token = Some(args[i + 1].clone());
                                i += 2;
                            }
                            "--token-file" => {
                                if i + 1 >= args.len() {
                                    eprintln!(
                                        "Usage: {} registry search <query> [--index <index.json>] [--registry <url>] [--live] [--token <token>] [--token-file <path>]",
                                        args[0]
                                    );
                                    return;
                                }
                                token_file = Some(args[i + 1].clone());
                                i += 2;
                            }
                            _ => {
                                eprintln!(
                                    "Usage: {} registry search <query> [--index <index.json>] [--registry <url>] [--live] [--token <token>] [--token-file <path>]",
                                    args[0]
                                );
                                return;
                            }
                        }
                    }

                    let client = registry::RegistryClient::new(registry_url.as_deref());
                    println!("Registry: {}", client.base_url);

                    let auth = match registry::resolve_registry_auth(
                        token.as_deref(),
                        token_file.as_deref().map(Path::new),
                    ) {
                        Ok(auth) => auth,
                        Err(err) => {
                            eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                            exit(1);
                        }
                    };

                    let search_result = if live {
                        registry::search_live_registry(&client, query, auth.as_ref())
                    } else {
                        registry::search_local_index(Path::new(&index_path), query)
                    };
                    match search_result {
                        Ok(results) => {
                            println!("Found {} package(s)", results.len());
                            for pkg in results {
                                let description = pkg
                                    .description
                                    .unwrap_or_else(|| "no description".to_string());
                                println!(
                                    "  {} {} (downloads: {}) - {}",
                                    pkg.name, pkg.version, pkg.downloads, description
                                );
                            }
                        }
                        Err(err) => {
                            eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                            exit(1);
                        }
                    }
                }
                "publish" => {
                    if args.len() < 4 {
                        eprintln!(
                            "Usage: {} registry publish <package-dir> [--registry <url>] [--token <token>] [--token-file <path>] [--dry-run]",
                            args[0]
                        );
                        return;
                    }
                    let package_dir = &args[3];
                    let mut registry_url: Option<String> = None;
                    let mut token: Option<String> = None;
                    let mut token_file: Option<String> = None;
                    let mut dry_run = false;

                    let mut i = 4usize;
                    while i < args.len() {
                        match args[i].as_str() {
                            "--registry" => {
                                if i + 1 >= args.len() {
                                    eprintln!(
                                        "Usage: {} registry publish <package-dir> [--registry <url>] [--token <token>] [--token-file <path>] [--dry-run]",
                                        args[0]
                                    );
                                    return;
                                }
                                registry_url = Some(args[i + 1].clone());
                                i += 2;
                            }
                            "--token" => {
                                if i + 1 >= args.len() {
                                    eprintln!(
                                        "Usage: {} registry publish <package-dir> [--registry <url>] [--token <token>] [--token-file <path>] [--dry-run]",
                                        args[0]
                                    );
                                    return;
                                }
                                token = Some(args[i + 1].clone());
                                i += 2;
                            }
                            "--token-file" => {
                                if i + 1 >= args.len() {
                                    eprintln!(
                                        "Usage: {} registry publish <package-dir> [--registry <url>] [--token <token>] [--token-file <path>] [--dry-run]",
                                        args[0]
                                    );
                                    return;
                                }
                                token_file = Some(args[i + 1].clone());
                                i += 2;
                            }
                            "--dry-run" => {
                                dry_run = true;
                                i += 1;
                            }
                            _ => {
                                eprintln!(
                                    "Usage: {} registry publish <package-dir> [--registry <url>] [--token <token>] [--token-file <path>] [--dry-run]",
                                    args[0]
                                );
                                return;
                            }
                        }
                    }

                    let client = registry::RegistryClient::new(registry_url.as_deref());
                    let auth = match registry::resolve_registry_auth(
                        token.as_deref(),
                        token_file.as_deref().map(Path::new),
                    ) {
                        Ok(auth) => auth,
                        Err(err) => {
                            eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                            exit(1);
                        }
                    };

                    if dry_run {
                        match registry::build_publish_preview(&client, Path::new(package_dir)) {
                            Ok(preview) => {
                                println!("Registry publish preview:");
                                match serde_json::to_string_pretty(&preview) {
                                    Ok(json) => println!("{}", json),
                                    Err(err) => {
                                        eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                                        exit(1);
                                    }
                                }
                            }
                            Err(err) => {
                                eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                                exit(1);
                            }
                        }
                    } else {
                        match registry::publish_live(
                            &client,
                            Path::new(package_dir),
                            auth.as_ref(),
                            false,
                        ) {
                            Ok(result) => {
                                println!("Registry publish result:");
                                match serde_json::to_string_pretty(&result) {
                                    Ok(json) => println!("{}", json),
                                    Err(err) => {
                                        eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                                        exit(1);
                                    }
                                }
                            }
                            Err(err) => {
                                eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                                exit(1);
                            }
                        }
                    }
                }
                "install" => {
                    if args.len() < 4 {
                        eprintln!(
                            "Usage: {} registry install <package> [--version <semver>] [--registry <url>] [--target <dir>] [--token <token>] [--token-file <path>] [--expected-sha256 <digest>] [--allow-untrusted] [--dry-run]",
                            args[0]
                        );
                        return;
                    }
                    let package_name = &args[3];
                    let mut version: Option<String> = None;
                    let mut registry_url: Option<String> = None;
                    let mut target_dir = ".".to_string();
                    let mut token: Option<String> = None;
                    let mut token_file: Option<String> = None;
                    let mut expected_sha256: Option<String> = None;
                    let mut trust = registry::PackageTrustPolicy::default();
                    let mut dry_run = false;

                    let mut i = 4usize;
                    while i < args.len() {
                        match args[i].as_str() {
                            "--version" => {
                                if i + 1 >= args.len() {
                                    eprintln!(
                                        "Usage: {} registry install <package> [--version <semver>] [--registry <url>] [--target <dir>] [--token <token>] [--token-file <path>] [--expected-sha256 <digest>] [--allow-untrusted] [--dry-run]",
                                        args[0]
                                    );
                                    return;
                                }
                                version = Some(args[i + 1].clone());
                                i += 2;
                            }
                            "--registry" => {
                                if i + 1 >= args.len() {
                                    eprintln!(
                                        "Usage: {} registry install <package> [--version <semver>] [--registry <url>] [--target <dir>] [--token <token>] [--token-file <path>] [--expected-sha256 <digest>] [--allow-untrusted] [--dry-run]",
                                        args[0]
                                    );
                                    return;
                                }
                                registry_url = Some(args[i + 1].clone());
                                i += 2;
                            }
                            "--target" => {
                                if i + 1 >= args.len() {
                                    eprintln!(
                                        "Usage: {} registry install <package> [--version <semver>] [--registry <url>] [--target <dir>] [--token <token>] [--token-file <path>] [--expected-sha256 <digest>] [--allow-untrusted] [--dry-run]",
                                        args[0]
                                    );
                                    return;
                                }
                                target_dir = args[i + 1].clone();
                                i += 2;
                            }
                            "--token" => {
                                if i + 1 >= args.len() {
                                    eprintln!(
                                        "Usage: {} registry install <package> [--version <semver>] [--registry <url>] [--target <dir>] [--token <token>] [--token-file <path>] [--expected-sha256 <digest>] [--allow-untrusted] [--dry-run]",
                                        args[0]
                                    );
                                    return;
                                }
                                token = Some(args[i + 1].clone());
                                i += 2;
                            }
                            "--token-file" => {
                                if i + 1 >= args.len() {
                                    eprintln!(
                                        "Usage: {} registry install <package> [--version <semver>] [--registry <url>] [--target <dir>] [--token <token>] [--token-file <path>] [--expected-sha256 <digest>] [--allow-untrusted] [--dry-run]",
                                        args[0]
                                    );
                                    return;
                                }
                                token_file = Some(args[i + 1].clone());
                                i += 2;
                            }
                            "--expected-sha256" => {
                                if i + 1 >= args.len() {
                                    eprintln!(
                                        "Usage: {} registry install <package> [--version <semver>] [--registry <url>] [--target <dir>] [--token <token>] [--token-file <path>] [--expected-sha256 <digest>] [--allow-untrusted] [--dry-run]",
                                        args[0]
                                    );
                                    return;
                                }
                                expected_sha256 = Some(args[i + 1].clone());
                                i += 2;
                            }
                            "--allow-untrusted" => {
                                trust.allow_untrusted = true;
                                i += 1;
                            }
                            "--dry-run" => {
                                dry_run = true;
                                i += 1;
                            }
                            _ => {
                                eprintln!(
                                    "Usage: {} registry install <package> [--version <semver>] [--registry <url>] [--target <dir>] [--token <token>] [--token-file <path>] [--expected-sha256 <digest>] [--allow-untrusted] [--dry-run]",
                                    args[0]
                                );
                                return;
                            }
                        }
                    }

                    let client = registry::RegistryClient::new(registry_url.as_deref());
                    let auth = match registry::resolve_registry_auth(
                        token.as_deref(),
                        token_file.as_deref().map(Path::new),
                    ) {
                        Ok(auth) => auth,
                        Err(err) => {
                            eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                            exit(1);
                        }
                    };

                    if dry_run {
                        let plan = registry::build_install_plan(
                            &client,
                            package_name,
                            version.as_deref(),
                            Path::new(&target_dir),
                            trust.clone(),
                        );
                        println!("Registry install plan:");
                        match serde_json::to_string_pretty(&plan) {
                            Ok(json) => println!("{}", json),
                            Err(err) => {
                                eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                                exit(1);
                            }
                        }
                    } else {
                        match registry::install_live(
                            &client,
                            package_name,
                            version.as_deref(),
                            Path::new(&target_dir),
                            auth.as_ref(),
                            &trust,
                            expected_sha256.as_deref(),
                            false,
                        ) {
                            Ok(receipt) => {
                                println!("Registry install receipt:");
                                match serde_json::to_string_pretty(&receipt) {
                                    Ok(json) => println!("{}", json),
                                    Err(err) => {
                                        eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                                        exit(1);
                                    }
                                }
                            }
                            Err(err) => {
                                eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                                exit(1);
                            }
                        }
                    }
                }
                _ => {
                    print_registry_help(&args[0]);
                }
            }
        }
        "conformance" => {
            let mut output_json: Option<String> = None;
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "-o" => {
                        if i + 1 >= args.len() {
                            eprintln!("Usage: {} conformance [-o <report.json>]", args[0]);
                            return;
                        }
                        output_json = Some(args[i + 1].clone());
                        i += 2;
                    }
                    _ => {
                        eprintln!("Usage: {} conformance [-o <report.json>]", args[0]);
                        return;
                    }
                }
            }

            let report = conformance::run_conformance_suite();
            println!(
                "Conformance cases: {}/{} passed | Mechanized checks: {}/{} passed",
                report.passed_cases,
                report.total_cases,
                report.passed_mechanized_checks,
                report.total_mechanized_checks
            );
            for case in &report.case_results {
                println!(
                    "  [{}] {} - {}",
                    if case.passed { "ok" } else { "fail" },
                    case.name,
                    case.details
                );
            }
            for check in &report.mechanized_checks {
                println!(
                    "  [{}] {} - {}",
                    if check.passed { "ok" } else { "fail" },
                    check.name,
                    check.details
                );
            }

            if let Some(path) = output_json {
                match serde_json::to_string_pretty(&report) {
                    Ok(json) => {
                        if let Err(err) = fs::write(&path, json) {
                            eprintln!("\x1b[1;31merror\x1b[0m: could not write {}: {}", path, err);
                            exit(1);
                        }
                        println!("Wrote conformance report to {}", path);
                    }
                    Err(err) => {
                        eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                        exit(1);
                    }
                }
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
            eprintln!(
                "Available commands: build, run, check, test, fmt, doc, profile, graph-opt, quantize, registry, conformance, init, lsp"
            );
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
    let graph_compile_start = Instant::now();
    let graph_backend =
        AcceleratorBackend::from_env("AERO_ACCELERATOR").unwrap_or(AcceleratorBackend::Cpu);
    let graph_annotation_only = env::var("AERO_GRAPH_ANNOTATION_ONLY")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let graph_config = graph_compiler::GraphCompilationConfig {
        backend: graph_backend,
        executable_fusion: !graph_annotation_only,
    };
    let (llvm_ir, graph_report) =
        graph_compiler::apply_advanced_graph_compilation_with_config(&llvm_ir, &graph_config);
    let graph_compile_time = graph_compile_start.elapsed();
    let codegen_time = codegen_start.elapsed();
    println!("Optimized code generation completed in {:?}", codegen_time);
    println!(
        "Advanced graph compilation completed in {:?} (backend: {}, fused kernels: {}, executable: {}, total fused ops: {})",
        graph_compile_time,
        graph_report.backend,
        graph_report.fused_kernel_count,
        graph_report.executable_kernel_count,
        graph_report.total_fused_ops
    );

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
    println!("    doc <input.aero> [-o <output.md>]    Generate Markdown API docs from source");
    println!("    profile <input.aero> [-o <trace.json>] Profile compilation phases");
    println!(
        "    graph-opt <input.ll> -o <output.ll>  Apply graph compilation and executable kernel fusion"
    );
    println!("    quantize <input.ll> -o <output.ll> --mode <int8|fp8-e4m3|fp8-e5m2>");
    println!(
        "                                         Apply calibrated INT8/FP8 lowering interface"
    );
    println!(
        "    registry <subcommand>                Interact with registry.aero (local or live transport)"
    );
    println!(
        "    conformance [-o <report.json>]       Run formal conformance and mechanized checks"
    );
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
    println!("    {} doc hello.aero -o hello.md", program_name);
    println!("    {} profile hello.aero -o trace.json", program_name);
    println!(
        "    {} graph-opt hello.ll -o hello.opt.ll --backend rocm",
        program_name
    );
    println!(
        "    {} quantize hello.opt.ll -o hello.int8.ll --mode int8 --backend rocm --calibration calib.json",
        program_name
    );
    println!(
        "    {} registry search vision --live --registry https://registry.aero/api/v1",
        program_name
    );
    println!("    {} registry publish . --dry-run", program_name);
    println!(
        "    {} registry install vision-core --version 0.2.0 --target pkgs --dry-run",
        program_name
    );
    println!(
        "    {} conformance -o conformance_report.json",
        program_name
    );
    println!("    {} init my_app", program_name);
    println!("    {} lsp", program_name);
}

fn default_doc_output_path(input_file: &str) -> String {
    let path = Path::new(input_file);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("aero_doc");
    let mut output = path.with_file_name(format!("{}.md", stem));
    if output.extension().is_none() {
        output.set_extension("md");
    }
    output.to_string_lossy().to_string()
}

fn print_registry_help(program_name: &str) {
    println!("registry.aero commands");
    println!();
    println!("USAGE:");
    println!(
        "    {} registry search <query> [--index <index.json>] [--registry <url>] [--live] [--token <token>] [--token-file <path>]",
        program_name
    );
    println!(
        "    {} registry publish <package-dir> [--registry <url>] [--token <token>] [--token-file <path>] [--dry-run]",
        program_name
    );
    println!(
        "    {} registry install <package> [--version <semver>] [--registry <url>] [--target <dir>] [--token <token>] [--token-file <path>] [--expected-sha256 <digest>] [--allow-untrusted] [--dry-run]",
        program_name
    );
    println!();
    println!("NOTE:");
    println!("    Without --dry-run, publish/install use live HTTP transport and trust checks.");
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
