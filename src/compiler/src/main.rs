mod accelerator;
mod ast;
mod code_generator;
mod compatibility;
mod conformance;
mod doc_generator;
mod errors;
mod gpu;
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
use gpu::{DeviceProfile, GpuDevice, default_gpu_arch};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use std::time::Instant;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BuildTarget {
    Cpu,
    Rocm,
    Cuda,
}

impl BuildTarget {
    fn parse(input: &str) -> Option<Self> {
        match input.trim().to_ascii_lowercase().as_str() {
            "cpu" | "host" => Some(Self::Cpu),
            "rocm" | "amd" => Some(Self::Rocm),
            "cuda" | "nvidia" => Some(Self::Cuda),
            "gpu" => {
                let detected = GpuDevice::auto_detect();
                Some(match detected.backend() {
                    AcceleratorBackend::Rocm => Self::Rocm,
                    AcceleratorBackend::Cuda => Self::Cuda,
                    AcceleratorBackend::Cpu => Self::Cpu,
                })
            }
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::Rocm => "rocm",
            Self::Cuda => "cuda",
        }
    }
}

#[derive(Debug, Clone)]
struct BuildConfig {
    target: BuildTarget,
    gpu_arch: Option<String>,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            target: BuildTarget::Cpu,
            gpu_arch: None,
        }
    }
}

#[derive(Debug, Clone)]
struct RunArtifactPaths {
    directory: PathBuf,
    ll_file: PathBuf,
    obj_file: PathBuf,
    exe_file: PathBuf,
    gpu_obj_file: PathBuf,
}

fn default_gpu_arch_for_backend(backend: AcceleratorBackend) -> Option<&'static str> {
    default_gpu_arch(backend)
}

fn backend_for_target(target: BuildTarget) -> AcceleratorBackend {
    match target {
        BuildTarget::Cpu => AcceleratorBackend::Cpu,
        BuildTarget::Rocm => AcceleratorBackend::Rocm,
        BuildTarget::Cuda => AcceleratorBackend::Cuda,
    }
}

fn apply_target_environment(build_config: &BuildConfig) {
    let backend = backend_for_target(build_config.target);
    let backend_name = backend.as_str();

    // SAFETY: this CLI is single-process and updates environment variables before
    // launching any child compilation commands.
    unsafe {
        env::set_var("AERO_ACCELERATOR", backend_name);
    }

    if backend == AcceleratorBackend::Rocm {
        let rocm_target = format!("rocm-{}", build_config.gpu_arch_or_default());
        // SAFETY: same rationale as above; this is process-local CLI configuration.
        unsafe {
            env::set_var("AERO_TARGET", rocm_target);
        }
    } else {
        // SAFETY: same rationale as above; this keeps stale ROCm target state from leaking.
        unsafe {
            env::remove_var("AERO_TARGET");
        }
    }
}

fn sanitize_artifact_stem(stem: &str) -> String {
    let mut out = String::new();
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "program".to_string()
    } else {
        out
    }
}

fn create_run_artifact_paths(
    input_file: &str,
    build_config: &BuildConfig,
) -> Result<RunArtifactPaths, String> {
    let stem = Path::new(input_file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("program");
    let safe_stem = sanitize_artifact_stem(stem);

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| format!("system clock error while creating run artifacts: {}", err))?
        .as_nanos();

    let run_dir = env::current_dir()
        .map_err(|err| format!("failed to get current directory: {}", err))?
        .join("target")
        .join("aero-run")
        .join(format!("{}-{}", safe_stem, nonce));
    fs::create_dir_all(&run_dir).map_err(|err| {
        format!(
            "failed to create run artifact directory {}: {}",
            run_dir.display(),
            err
        )
    })?;

    let ll_file = run_dir.join(format!("{}.ll", safe_stem));
    let obj_file = run_dir.join(format!("{}.o", safe_stem));
    let exe_name = if cfg!(windows) {
        format!("{}.exe", safe_stem)
    } else {
        safe_stem.clone()
    };
    let exe_file = run_dir.join(exe_name);
    let gpu_obj_file = run_dir.join(format!(
        "{}.{}.o",
        safe_stem,
        build_config.gpu_arch_or_default()
    ));

    Ok(RunArtifactPaths {
        directory: run_dir,
        ll_file,
        obj_file,
        exe_file,
        gpu_obj_file,
    })
}

impl BuildConfig {
    fn gpu_arch_or_default(&self) -> &str {
        if let Some(arch) = self.gpu_arch.as_deref() {
            return arch;
        }
        let backend = backend_for_target(self.target);
        default_gpu_arch_for_backend(backend).unwrap_or("x86_64")
    }

    fn llvm_target_triple(&self) -> &str {
        let backend = backend_for_target(self.target);
        let device = GpuDevice::new(backend, 0, self.gpu_arch.clone());
        device.target_triple()
    }

    fn llvm_data_layout(&self) -> &str {
        match self.target {
            BuildTarget::Cpu => {
                if cfg!(target_os = "windows") {
                    "e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128"
                } else {
                    "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
                }
            }
            BuildTarget::Rocm => {
                "e-p:64:64-p1:64:64-p2:32:32-p3:32:32-p4:64:64-p5:32:32-p6:32:32-p7:160:256:256:32-p8:128:128-p9:192:256:256:32-i64:64-v16:16-v24:32-v32:32-v48:64-v96:128-v192:256-v256:256-v512:512-v1024:1024-v2048:2048-n32:64"
            }
            BuildTarget::Cuda => "e-i64:64-v16:16-v32:32-n16:32:64",
        }
    }
}

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
            let (input_file, output_file, build_config) = match parse_build_args(&args) {
                Ok(parsed) => parsed,
                Err(usage) => {
                    eprintln!("{}", usage);
                    return;
                }
            };
            apply_target_environment(&build_config);

            let source_code = match fs::read_to_string(&input_file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Error reading file {}: {}", input_file, err);
                    return;
                }
            };

            compile_to_llvm_ir(&source_code, &output_file, &input_file, &build_config);
        }
        "run" => {
            let (input_file, build_config) = match parse_run_args(&args) {
                Ok(parsed) => parsed,
                Err(usage) => {
                    eprintln!("{}", usage);
                    return;
                }
            };
            apply_target_environment(&build_config);

            let source_code = match fs::read_to_string(&input_file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Error reading file {}: {}", input_file, err);
                    return;
                }
            };

            if let Err(err) = run_aero_program(&source_code, &input_file, &build_config) {
                eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                exit(1);
            }
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
            let graph_usage = format!(
                "Usage: {} graph-opt <input.ll> -o <output.ll> [--backend <cpu|cuda|rocm>] [--gpu <arch>] [--annotation-only]",
                args[0]
            );
            if args.len() < 5 {
                eprintln!("{}", graph_usage);
                return;
            }

            let input_file = &args[2];
            let mut output_file: Option<String> = None;
            let mut backend = AcceleratorBackend::Cpu;
            let mut gpu_arch: Option<String> = None;
            let mut executable_fusion = true;

            let mut i = 3usize;
            while i < args.len() {
                match args[i].as_str() {
                    "-o" => {
                        if i + 1 >= args.len() {
                            eprintln!("{}", graph_usage);
                            return;
                        }
                        output_file = Some(args[i + 1].clone());
                        i += 2;
                    }
                    "--backend" => {
                        if i + 1 >= args.len() {
                            eprintln!("{}", graph_usage);
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
                    "--gpu" => {
                        if i + 1 >= args.len() {
                            eprintln!("{}", graph_usage);
                            return;
                        }
                        gpu_arch = Some(args[i + 1].clone());
                        i += 2;
                    }
                    "--annotation-only" => {
                        executable_fusion = false;
                        i += 1;
                    }
                    _ => {
                        eprintln!("{}", graph_usage);
                        return;
                    }
                }
            }
            let Some(output_file) = output_file else {
                eprintln!("{}", graph_usage);
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
                gpu_arch,
            };
            let (optimized, report) =
                graph_compiler::apply_advanced_graph_compilation_with_config(&input, &config);
            match fs::write(&output_file, optimized) {
                Ok(_) => {
                    println!("Wrote graph-optimized IR to {}", output_file);
                    let gpu_arch = report.gpu_arch.as_deref().unwrap_or("n/a");
                    println!(
                        "Backend: {} | gpu: {} | fused kernels: {} | executable kernels: {} | skipped chains: {} | total fused ops: {}",
                        report.backend,
                        gpu_arch,
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
            let quant_usage = format!(
                "Usage: {} quantize <input.ll> -o <output.ll> --mode <int8|fp8-e4m3|fp8-e5m2> [--backend <cpu|cuda|rocm>] [--gpu <arch>] [--calibration <samples.json|samples.txt>] [--per-channel] [--annotation-only]",
                args[0]
            );
            if args.len() < 7 {
                eprintln!("{}", quant_usage);
                return;
            }

            let input_file = &args[2];
            let mut output_file: Option<String> = None;
            let mut mode: Option<quantization::QuantizationMode> = None;
            let mut backend = AcceleratorBackend::Cpu;
            let mut gpu_arch: Option<String> = None;
            let mut per_channel = false;
            let mut runtime_lowering = true;
            let mut calibration_file: Option<String> = None;

            let mut i = 3usize;
            while i < args.len() {
                match args[i].as_str() {
                    "-o" => {
                        if i + 1 >= args.len() {
                            eprintln!("{}", quant_usage);
                            return;
                        }
                        output_file = Some(args[i + 1].clone());
                        i += 2;
                    }
                    "--mode" => {
                        if i + 1 >= args.len() {
                            eprintln!("{}", quant_usage);
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
                            eprintln!("{}", quant_usage);
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
                    "--gpu" => {
                        if i + 1 >= args.len() {
                            eprintln!("{}", quant_usage);
                            return;
                        }
                        gpu_arch = Some(args[i + 1].clone());
                        i += 2;
                    }
                    "--calibration" => {
                        if i + 1 >= args.len() {
                            eprintln!("{}", quant_usage);
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
                        eprintln!("{}", quant_usage);
                        return;
                    }
                }
            }

            let Some(output_file) = output_file else {
                eprintln!("{}", quant_usage);
                return;
            };
            let Some(mode) = mode else {
                eprintln!("{}", quant_usage);
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
            config.gpu_arch = gpu_arch;
            config.per_channel = per_channel;
            config.enable_runtime_lowering = runtime_lowering;

            if let Some(calibration_file) = &calibration_file {
                match quantization::load_calibration_profile(
                    Path::new(calibration_file),
                    mode,
                    backend,
                    config.gpu_arch.as_deref(),
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
                    let gpu_arch = report.gpu_arch.as_deref().unwrap_or("n/a");
                    println!(
                        "Mode: {} | backend: {} | gpu: {} | candidates: {} | lowered: {} | helpers: {} | calibration samples: {}",
                        report.mode,
                        report.backend,
                        gpu_arch,
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

fn parse_build_args(args: &[String]) -> Result<(String, String, BuildConfig), String> {
    if args.len() < 3 {
        return Err(format!(
            "Usage: {} build <input.aero> -o <output.ll> [--target <cpu|rocm|cuda|gpu>] [--gpu <arch>]",
            args[0]
        ));
    }

    let input_file = args[2].clone();
    let mut output_file: Option<String> = None;
    let mut config = BuildConfig::default();
    let mut i = 3usize;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                if i + 1 >= args.len() {
                    return Err(format!(
                        "Usage: {} build <input.aero> -o <output.ll> [--target <cpu|rocm|cuda|gpu>] [--gpu <arch>]",
                        args[0]
                    ));
                }
                output_file = Some(args[i + 1].clone());
                i += 2;
            }
            "--target" | "--backend" => {
                if i + 1 >= args.len() {
                    return Err(format!(
                        "Usage: {} build <input.aero> -o <output.ll> [--target <cpu|rocm|cuda|gpu>] [--gpu <arch>]",
                        args[0]
                    ));
                }
                let Some(target) = BuildTarget::parse(&args[i + 1]) else {
                    return Err(format!(
                        "error: unsupported target `{}` (expected cpu|rocm|cuda|gpu)",
                        args[i + 1]
                    ));
                };
                config.target = target;
                i += 2;
            }
            "--gpu" => {
                if i + 1 >= args.len() {
                    return Err(format!(
                        "Usage: {} build <input.aero> -o <output.ll> [--target <cpu|rocm|cuda|gpu>] [--gpu <arch>]",
                        args[0]
                    ));
                }
                config.gpu_arch = Some(args[i + 1].clone());
                i += 2;
            }
            _ => {
                return Err(format!(
                    "Usage: {} build <input.aero> -o <output.ll> [--target <cpu|rocm|cuda|gpu>] [--gpu <arch>]",
                    args[0]
                ));
            }
        }
    }

    let Some(output_file) = output_file else {
        return Err(format!(
            "Usage: {} build <input.aero> -o <output.ll> [--target <cpu|rocm|cuda|gpu>] [--gpu <arch>]",
            args[0]
        ));
    };

    Ok((input_file, output_file, config))
}

fn parse_run_args(args: &[String]) -> Result<(String, BuildConfig), String> {
    if args.len() < 3 {
        return Err(format!(
            "Usage: {} run <input.aero> [--target <cpu|rocm|cuda|gpu>] [--gpu <arch>]\n       {} run --target rocm --gpu gfx1101 <input.aero>",
            args[0], args[0]
        ));
    }

    let mut input_file: Option<String> = None;
    let mut config = BuildConfig::default();

    let mut i = 2usize;
    while i < args.len() {
        match args[i].as_str() {
            "--target" | "--backend" => {
                if i + 1 >= args.len() {
                    return Err(format!(
                        "Usage: {} run <input.aero> [--target <cpu|rocm|cuda|gpu>] [--gpu <arch>]",
                        args[0]
                    ));
                }
                let Some(target) = BuildTarget::parse(&args[i + 1]) else {
                    return Err(format!(
                        "error: unsupported target `{}` (expected cpu|rocm|cuda|gpu)",
                        args[i + 1]
                    ));
                };
                config.target = target;
                i += 2;
            }
            "--gpu" => {
                if i + 1 >= args.len() {
                    return Err(format!(
                        "Usage: {} run <input.aero> [--target <cpu|rocm|cuda|gpu>] [--gpu <arch>]",
                        args[0]
                    ));
                }
                config.gpu_arch = Some(args[i + 1].clone());
                i += 2;
            }
            value if value.starts_with('-') => {
                return Err(format!(
                    "error: unknown option `{}`\nUsage: {} run <input.aero> [--target <cpu|rocm|cuda|gpu>] [--gpu <arch>]",
                    value, args[0]
                ));
            }
            value => {
                if input_file.is_some() {
                    let existing = input_file.as_deref().unwrap_or("<unknown>");
                    return Err(format!(
                        "error: multiple input files provided (`{}` and `{}`)\nUsage: {} run <input.aero> [--target <cpu|rocm|cuda|gpu>] [--gpu <arch>]",
                        existing, value, args[0]
                    ));
                }
                input_file = Some(value.to_string());
                i += 1;
            }
        }
    }

    let Some(input_file) = input_file else {
        return Err(format!(
            "Usage: {} run <input.aero> [--target <cpu|rocm|cuda|gpu>] [--gpu <arch>]\n       {} run --target rocm --gpu gfx1101 <input.aero>",
            args[0], args[0]
        ));
    };

    Ok((input_file, config))
}

fn retarget_llvm_module(llvm_ir: &str, build_config: &BuildConfig) -> String {
    let mut out = String::new();
    let mut inserted_target_header = false;

    for line in llvm_ir.lines() {
        if line.starts_with("target datalayout = ") || line.starts_with("target triple = ") {
            continue;
        }

        out.push_str(line);
        out.push('\n');
        if line.starts_with("source_filename = ") {
            out.push_str(&format!(
                "target datalayout = \"{}\"\n",
                build_config.llvm_data_layout()
            ));
            out.push_str(&format!(
                "target triple = \"{}\"\n",
                build_config.llvm_target_triple()
            ));
            inserted_target_header = true;
        }
    }

    if !inserted_target_header {
        out.push_str(&format!(
            "target datalayout = \"{}\"\n",
            build_config.llvm_data_layout()
        ));
        out.push_str(&format!(
            "target triple = \"{}\"\n",
            build_config.llvm_target_triple()
        ));
    }

    out
}

fn compile_to_llvm_ir(
    source_code: &str,
    output_file: &str,
    input_file: &str,
    build_config: &BuildConfig,
) {
    println!(
        "Compiling with performance optimizations enabled (target: {}, gpu: {})",
        build_config.target.as_str(),
        build_config.gpu_arch_or_default()
    );

    // Initialize performance optimizer
    let mut perf_optimizer = PerformanceOptimizer::new();
    let compilation_start = Instant::now();

    // Generate source hash for caching
    let source_hash = format!(
        "{:x}",
        md5::compute(format!(
            "{}::target={}::gpu={}",
            source_code,
            build_config.target.as_str(),
            build_config.gpu_arch_or_default()
        ))
    );

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
        AcceleratorBackend::from_env("AERO_ACCELERATOR").unwrap_or(match build_config.target {
            BuildTarget::Cpu => AcceleratorBackend::Cpu,
            BuildTarget::Rocm => AcceleratorBackend::Rocm,
            BuildTarget::Cuda => AcceleratorBackend::Cuda,
        });
    let graph_annotation_only = env::var("AERO_GRAPH_ANNOTATION_ONLY")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let graph_config = graph_compiler::GraphCompilationConfig {
        backend: graph_backend,
        executable_fusion: !graph_annotation_only,
        gpu_arch: build_config
            .gpu_arch
            .clone()
            .or_else(|| default_gpu_arch_for_backend(graph_backend).map(str::to_string)),
    };
    let (llvm_ir, graph_report) =
        graph_compiler::apply_advanced_graph_compilation_with_config(&llvm_ir, &graph_config);
    let llvm_ir = retarget_llvm_module(&llvm_ir, build_config);
    let graph_compile_time = graph_compile_start.elapsed();
    let codegen_time = codegen_start.elapsed();
    println!("Optimized code generation completed in {:?}", codegen_time);
    println!(
        "Advanced graph compilation completed in {:?} (backend: {}, gpu: {}, fused kernels: {}, executable: {}, total fused ops: {})",
        graph_compile_time,
        graph_report.backend,
        graph_report.gpu_arch.as_deref().unwrap_or("n/a"),
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

fn run_aero_program(
    source_code: &str,
    input_file: &str,
    build_config: &BuildConfig,
) -> Result<(), String> {
    let artifacts = create_run_artifact_paths(input_file, build_config)?;
    let ll_path = artifacts.ll_file.to_string_lossy().to_string();
    let obj_path = artifacts.obj_file.to_string_lossy().to_string();
    let exe_path = artifacts.exe_file.to_string_lossy().to_string();
    let gpu_obj_path = artifacts.gpu_obj_file.to_string_lossy().to_string();

    // Compile to LLVM IR first.
    compile_to_llvm_ir(source_code, &ll_path, input_file, build_config);
    if !artifacts.ll_file.exists() {
        return Err(format!(
            "compile step did not produce LLVM IR at {}",
            artifacts.ll_file.display()
        ));
    }

    match build_config.target {
        BuildTarget::Cpu => {
            let clang_bin = find_llvm_tool("clang").ok_or_else(|| {
                "Error executing clang: program not found. Make sure LLVM/clang is installed and in your PATH."
                    .to_string()
            })?;

            if let Some(llc_bin) = find_llvm_tool("llc") {
                let llc_output = Command::new(&llc_bin)
                    .args(["-filetype=obj", &ll_path, "-o", &obj_path])
                    .output()
                    .map_err(|err| format!("Error executing llc ({}): {}", llc_bin, err))?;

                if !llc_output.status.success() {
                    return Err(format!(
                        "Error running llc: {}",
                        String::from_utf8_lossy(&llc_output.stderr)
                    ));
                }

                let clang_output = Command::new(&clang_bin)
                    .args([&obj_path, "-o", &exe_path])
                    .output()
                    .map_err(|err| format!("Error executing clang ({}): {}", clang_bin, err))?;

                if !clang_output.status.success() {
                    return Err(format!(
                        "Error running clang: {}",
                        String::from_utf8_lossy(&clang_output.stderr)
                    ));
                }
            } else {
                // Fallback path: clang can compile textual LLVM IR directly.
                println!(
                    "llc not found in PATH. Falling back to direct clang LLVM IR compilation."
                );

                let clang_output = Command::new(&clang_bin)
                    .args([&ll_path, "-o", &exe_path])
                    .output()
                    .map_err(|err| format!("Error executing clang ({}): {}", clang_bin, err))?;

                if !clang_output.status.success() {
                    return Err(format!(
                        "Error running clang on LLVM IR fallback path: {}",
                        String::from_utf8_lossy(&clang_output.stderr)
                    ));
                }
            }

            let run_output = Command::new(&exe_path)
                .output()
                .map_err(|err| format!("Error executing compiled program: {}", err))?;

            let exit_code = run_output.status.code().unwrap_or(-1);
            println!("Program executed successfully.");
            println!("Exit code: {}", exit_code);

            if !run_output.stdout.is_empty() {
                println!("Output: {}", String::from_utf8_lossy(&run_output.stdout));
            }
            if !run_output.stderr.is_empty() {
                println!(
                    "Error output: {}",
                    String::from_utf8_lossy(&run_output.stderr)
                );
            }

            let _ = fs::remove_file(&artifacts.ll_file);
            let _ = fs::remove_file(&artifacts.obj_file);
            let _ = fs::remove_file(&artifacts.exe_file);
            let _ = fs::remove_dir(&artifacts.directory);

            // Mirror executed process exit code.
            exit(exit_code);
        }
        BuildTarget::Rocm => {
            let llc_bin = find_llvm_tool("llc").ok_or_else(|| {
                format!(
                    "Error executing llc for ROCm target: program not found. Make sure LLVM is installed and llc is in your PATH. LLVM IR remains at {}",
                    artifacts.ll_file.display()
                )
            })?;

            let device = GpuDevice::new(
                AcceleratorBackend::Rocm,
                0,
                Some(build_config.gpu_arch_or_default().to_string()),
            );
            let mut llc_args = device.llc_target_flags().unwrap_or_else(|| {
                vec![
                    "-march=amdgcn".to_string(),
                    format!("-mcpu={}", build_config.gpu_arch_or_default()),
                    "-mattr=+wavefrontsize64,+gfx11-insts".to_string(),
                ]
            });
            llc_args.push("-filetype=obj".to_string());
            llc_args.push(ll_path.clone());
            llc_args.push("-o".to_string());
            llc_args.push(gpu_obj_path.clone());

            let llc_output = Command::new(&llc_bin).args(&llc_args).output();

            match llc_output {
                Ok(output) => {
                    if !output.status.success() {
                        return Err(format!(
                            "Error running llc for ROCm target: {}",
                            String::from_utf8_lossy(&output.stderr)
                        ));
                    }
                    println!(
                        "ROCm object generated for {} at {}",
                        build_config.gpu_arch_or_default(),
                        artifacts.gpu_obj_file.display()
                    );
                    println!("IR artifact: {}", artifacts.ll_file.display());
                    println!(
                        "Runtime execution for ROCm kernels is staged for HIP launcher integration; use the generated object for now."
                    );
                }
                Err(err) => {
                    return Err(format!(
                        "Error executing llc for ROCm target ({}): {}. LLVM IR remains at {}",
                        llc_bin,
                        err,
                        artifacts.ll_file.display()
                    ));
                }
            }
        }
        BuildTarget::Cuda => {
            return Err(
                "CUDA run target is not implemented yet in this build. Use --target cpu or --target rocm."
                    .to_string(),
            );
        }
    }

    Ok(())
}

fn find_llvm_tool(tool: &str) -> Option<String> {
    if Command::new(tool).arg("--version").output().is_ok() {
        return Some(tool.to_string());
    }

    if cfg!(windows) {
        let exe_name = format!("{}.exe", tool);
        let mut candidates = vec![PathBuf::from(r"C:\Program Files\LLVM\bin").join(&exe_name)];

        // Local source-built LLVM fallback used by this repository.
        if let Ok(repo_root) = env::current_dir() {
            candidates.push(
                repo_root
                    .join("third_party")
                    .join("llvm-project")
                    .join("build-rocm-tools")
                    .join("Release")
                    .join("bin")
                    .join(&exe_name),
            );
        }

        for candidate in candidates {
            if candidate.exists() {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
    }

    None
}

fn print_help(program_name: &str) {
    println!("Aero Programming Language Compiler v1.0.0");
    println!();
    println!("USAGE:");
    println!("    {} <COMMAND> [OPTIONS]", program_name);
    println!();
    println!("COMMANDS:");
    println!(
        "    build <input.aero> -o <output.ll>    Compile Aero source to LLVM IR [--target <cpu|rocm|cuda|gpu>] [--gpu <arch>]"
    );
    println!(
        "    run <input.aero>                     Compile and run source [--target <cpu|rocm|cuda|gpu>] [--gpu <arch>]"
    );
    println!("    check <input.aero>                   Type-check only (no codegen)");
    println!("    test                                 Discover and run *_test.aero files");
    println!("    fmt <input.aero>                     Auto-format Aero source");
    println!("    doc <input.aero> [-o <output.md>]    Generate Markdown API docs from source");
    println!("    profile <input.aero> [-o <trace.json>] Profile compilation phases");
    println!(
        "    graph-opt <input.ll> -o <output.ll>  Apply graph compilation and executable kernel fusion [--backend <cpu|cuda|rocm>] [--gpu <arch>]"
    );
    println!("    quantize <input.ll> -o <output.ll> --mode <int8|fp8-e4m3|fp8-e5m2>");
    println!(
        "                                         Apply calibrated INT8/FP8 lowering interface [--backend <cpu|cuda|rocm>] [--gpu <arch>]"
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
    println!(
        "    {} build hello.aero -o hello.rocm.ll --target rocm --gpu gfx1101",
        program_name
    );
    println!("    {} run hello.aero", program_name);
    println!(
        "    {} run --target rocm --gpu gfx1101 examples/gguf_inference.aero",
        program_name
    );
    println!("    {} check hello.aero", program_name);
    println!("    {} test", program_name);
    println!("    {} fmt hello.aero", program_name);
    println!("    {} doc hello.aero -o hello.md", program_name);
    println!("    {} profile hello.aero -o trace.json", program_name);
    println!(
        "    {} graph-opt hello.ll -o hello.opt.ll --backend rocm --gpu gfx1101",
        program_name
    );
    println!(
        "    {} quantize hello.opt.ll -o hello.int8.ll --mode int8 --backend rocm --gpu gfx1101 --calibration calib.json",
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parse_build_args_accepts_rocm_target_and_gpu_arch() {
        let args = vec![
            "aero".to_string(),
            "build".to_string(),
            "main.aero".to_string(),
            "-o".to_string(),
            "main.ll".to_string(),
            "--target".to_string(),
            "rocm".to_string(),
            "--gpu".to_string(),
            "gfx1101".to_string(),
        ];
        let (input, output, config) = parse_build_args(&args).expect("build args should parse");
        assert_eq!(input, "main.aero");
        assert_eq!(output, "main.ll");
        assert_eq!(config.target, BuildTarget::Rocm);
        assert_eq!(config.gpu_arch.as_deref(), Some("gfx1101"));
    }

    #[test]
    fn parse_build_args_accepts_backend_alias() {
        let args = vec![
            "aero".to_string(),
            "build".to_string(),
            "main.aero".to_string(),
            "-o".to_string(),
            "main.ll".to_string(),
            "--backend".to_string(),
            "rocm".to_string(),
        ];
        let (_input, _output, config) =
            parse_build_args(&args).expect("build args should parse with --backend");
        assert_eq!(config.target, BuildTarget::Rocm);
    }

    #[test]
    fn parse_run_args_supports_option_first_style() {
        let args = vec![
            "aero".to_string(),
            "run".to_string(),
            "--target".to_string(),
            "rocm".to_string(),
            "--gpu".to_string(),
            "gfx1101".to_string(),
            "examples/gguf_inference.aero".to_string(),
        ];
        let (input, config) = parse_run_args(&args).expect("run args should parse");
        assert_eq!(input, "examples/gguf_inference.aero");
        assert_eq!(config.target, BuildTarget::Rocm);
        assert_eq!(config.gpu_arch.as_deref(), Some("gfx1101"));
    }

    #[test]
    fn parse_run_args_supports_backend_alias() {
        let args = vec![
            "aero".to_string(),
            "run".to_string(),
            "--backend".to_string(),
            "rocm".to_string(),
            "examples/gguf_inference.aero".to_string(),
        ];
        let (input, config) = parse_run_args(&args).expect("run args should parse with --backend");
        assert_eq!(input, "examples/gguf_inference.aero");
        assert_eq!(config.target, BuildTarget::Rocm);
    }

    #[test]
    fn parse_run_args_supports_gpu_auto_target() {
        let args = vec![
            "aero".to_string(),
            "run".to_string(),
            "--target".to_string(),
            "gpu".to_string(),
            "examples/gguf_inference.aero".to_string(),
        ];
        let (_input, config) =
            parse_run_args(&args).expect("run args should parse with --target gpu");
        assert!(matches!(
            config.target,
            BuildTarget::Cpu | BuildTarget::Rocm | BuildTarget::Cuda
        ));
    }

    #[test]
    fn retarget_llvm_module_switches_triple_for_rocm() {
        let input = "; ModuleID = \"a\"\nsource_filename = \"a\"\ntarget datalayout = \"old\"\ntarget triple = \"old\"\n\ndefine i32 @main() {\nentry:\n  ret i32 0\n}\n";
        let config = BuildConfig {
            target: BuildTarget::Rocm,
            gpu_arch: Some("gfx1101".to_string()),
        };
        let output = retarget_llvm_module(input, &config);
        assert!(output.contains("target triple = \"amdgcn-amd-amdhsa\""));
        assert!(!output.contains("target triple = \"old\""));
    }

    #[test]
    fn sanitize_artifact_stem_replaces_non_alphanumeric_chars() {
        assert_eq!(sanitize_artifact_stem("hello-world"), "hello-world");
        assert_eq!(
            sanitize_artifact_stem("hello world.aero"),
            "hello_world_aero"
        );
        assert_eq!(sanitize_artifact_stem(""), "program");
    }

    #[test]
    fn create_run_artifact_paths_writes_under_target_aero_run() {
        let config = BuildConfig {
            target: BuildTarget::Rocm,
            gpu_arch: Some("gfx1101".to_string()),
        };
        let artifacts = create_run_artifact_paths("examples/hello.aero", &config)
            .expect("paths should be created");
        let dir = artifacts.directory.to_string_lossy();
        assert!(dir.contains("target"));
        assert!(dir.contains("aero-run"));
        assert!(artifacts.ll_file.to_string_lossy().ends_with(".ll"));
        assert!(artifacts.gpu_obj_file.to_string_lossy().contains("gfx1101"));
        let _ = fs::remove_dir_all(artifacts.directory);
    }
}
