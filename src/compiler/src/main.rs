mod doc_generator;
mod lsp;
mod profiler;
mod project_init;
mod runtime_link;

#[cfg(test)]
mod conformance_checked_ir_tests;
#[cfg(test)]
mod llvm_verifier_cache_tests;

#[cfg(test)]
static LLVM_VERIFIER_TEST_ENVIRONMENT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

use compiler::accelerator::AcceleratorBackend;
use compiler::gpu::{DeviceProfile, GpuDevice, default_gpu_arch};
use compiler::{
    CodeGenerationError, LIVE_REGISTRY_DISABLED_FOR_COMPILER_SERVICE, LanguageProfile,
    LlvmVerificationMode, PerformanceOptimizer, conformance, graph_compiler,
    guard_live_registry_transport_for_compiler_service,
    prepare_checked_program_with_module_observer,
    prepare_checked_program_with_module_observer_and_profile, quantization, registry,
    verify_llvm_module,
};
use runtime_link::compile_production_runtime;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use std::time::Instant;
use std::time::{SystemTime, UNIX_EPOCH};

fn render_code_generation_error(error: CodeGenerationError) -> String {
    match error {
        CodeGenerationError::IrVerification(error) => error.to_string(),
        other => format!("Code Generation Error: {other}"),
    }
}

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

fn parse_explicit_build_target(input: &str) -> Result<BuildTarget, String> {
    if input.trim().eq_ignore_ascii_case("gpu") {
        return Err(
            "target `gpu` is ambiguous and does not prove a usable device; choose cpu, rocm, or cuda explicitly"
                .to_string(),
        );
    }
    BuildTarget::parse(input).ok_or_else(|| {
        format!(
            "error: unsupported target `{}` (expected cpu|rocm|cuda)",
            input
        )
    })
}

#[derive(Debug, Clone)]
struct BuildConfig {
    target: BuildTarget,
    gpu_arch: Option<String>,
    require_llvm_verifier: bool,
    language_profile: LanguageProfile,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            target: BuildTarget::Cpu,
            gpu_arch: None,
            require_llvm_verifier: false,
            language_profile: LanguageProfile::Experimental,
        }
    }
}

#[derive(Debug, Clone)]
struct RunArtifactPaths {
    directory: PathBuf,
    ll_file: PathBuf,
    obj_file: PathBuf,
    runtime_source_file: PathBuf,
    runtime_obj_file: PathBuf,
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
    let runtime_source_file = run_dir.join("aero_runtime.c");
    let runtime_obj_file = run_dir.join("aero_runtime.o");
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
        runtime_source_file,
        runtime_obj_file,
        exe_file,
        gpu_obj_file,
    })
}

impl BuildConfig {
    fn validate_language_profile_target(&self) -> Result<(), String> {
        if self.language_profile != LanguageProfile::Experimental
            && (self.target != BuildTarget::Cpu || self.gpu_arch.is_some())
        {
            return Err(format!(
                "Language Profile Error: {} requires --target cpu without --gpu",
                self.language_profile.as_str()
            ));
        }
        Ok(())
    }

    fn llvm_verification_mode(&self) -> LlvmVerificationMode {
        if self.require_llvm_verifier || environment_requires_llvm_verifier() {
            LlvmVerificationMode::Required
        } else {
            LlvmVerificationMode::PreferExternal
        }
    }

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

#[derive(Debug)]
struct ConformanceCommandResult {
    exit_code: i32,
    stdout: String,
}

fn run_conformance_command_with_report(
    report: conformance::ConformanceReport,
    output_json: Option<&Path>,
) -> Result<ConformanceCommandResult, String> {
    let mut lines = vec![format!(
        "Conformance cases: {}/{} passed | Determinism checks: {}/{} passed",
        report.passed_cases,
        report.total_cases,
        report.passed_mechanized_checks,
        report.total_mechanized_checks
    )];
    for case in &report.case_results {
        lines.push(format!(
            "  [{}] {} - {}",
            if case.passed { "ok" } else { "fail" },
            case.name,
            case.details
        ));
    }
    for check in &report.mechanized_checks {
        lines.push(format!(
            "  [{}] {} - {}",
            if check.passed { "ok" } else { "fail" },
            check.name,
            check.details
        ));
    }

    if let Some(path) = output_json {
        let json = serde_json::to_string_pretty(&report)
            .map_err(|error| format!("failed to serialize conformance report: {error}"))?;
        fs::write(path, json)
            .map_err(|error| format!("could not write {}: {error}", path.display()))?;
        lines.push(format!("Wrote conformance report to {}", path.display()));
    }

    Ok(ConformanceCommandResult {
        exit_code: i32::from(report.failed_cases > 0 || report.failed_mechanized_checks > 0),
        stdout: lines.join("\n"),
    })
}

fn environment_requires_llvm_verifier() -> bool {
    env::var("AERO_REQUIRE_LLVM_VERIFIER")
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CliStatus {
    Success,
    OperationalFailure,
    InvocationFailure,
}

impl CliStatus {
    fn exit_code(self) -> i32 {
        match self {
            Self::Success => 0,
            Self::OperationalFailure => 1,
            Self::InvocationFailure => 2,
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let status = dispatch_cli(&args);
    if status != CliStatus::Success {
        exit(status.exit_code());
    }
}

fn dispatch_cli(args: &[String]) -> CliStatus {
    if args.len() < 2 {
        print_help(&args[0]);
        return CliStatus::InvocationFailure;
    }

    let command = &args[1];

    match command.as_str() {
        "--help" | "-h" => {
            print_help(&args[0]);
            return if args.len() == 2 {
                CliStatus::Success
            } else {
                CliStatus::InvocationFailure
            };
        }
        "--version" | "-v" => {
            println!("Aero compiler version {}", env!("CARGO_PKG_VERSION"));
            return if args.len() == 2 {
                CliStatus::Success
            } else {
                CliStatus::InvocationFailure
            };
        }
        "build" => {
            let (input_file, output_file, build_config) = match parse_build_args(&args) {
                Ok(parsed) => parsed,
                Err(usage) => {
                    eprintln!("{}", usage);
                    return CliStatus::InvocationFailure;
                }
            };
            apply_target_environment(&build_config);

            let source_code = match fs::read_to_string(&input_file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Error reading file {}: {}", input_file, err);
                    return CliStatus::OperationalFailure;
                }
            };

            if let Err(err) =
                compile_to_llvm_ir(&source_code, &output_file, &input_file, &build_config)
            {
                eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                return CliStatus::OperationalFailure;
            }
        }
        "run" => {
            let (input_file, build_config) = match parse_run_args(&args) {
                Ok(parsed) => parsed,
                Err(usage) => {
                    eprintln!("{}", usage);
                    return CliStatus::InvocationFailure;
                }
            };
            apply_target_environment(&build_config);

            let source_code = match fs::read_to_string(&input_file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Error reading file {}: {}", input_file, err);
                    return CliStatus::OperationalFailure;
                }
            };

            if let Err(err) = run_aero_program(&source_code, &input_file, &build_config) {
                eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                return CliStatus::OperationalFailure;
            }
        }
        "check" => {
            let (input_file, language_profile) = match parse_check_args(args) {
                Ok(parsed) => parsed,
                Err(usage) => {
                    eprintln!("{}", usage);
                    return CliStatus::InvocationFailure;
                }
            };

            let source_code = match fs::read_to_string(&input_file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!(
                        "\x1b[1;31merror\x1b[0m: could not read file {}: {}",
                        input_file, err
                    );
                    return CliStatus::OperationalFailure;
                }
            };

            if let Err(err) = check_aero_program(&source_code, &input_file, language_profile) {
                report_check_error(&source_code, &input_file, &err);
                return CliStatus::OperationalFailure;
            }
        }
        "test" => {
            if args.len() != 2 {
                eprintln!("Usage: {} test", args[0]);
                return CliStatus::InvocationFailure;
            }
            // Discover and validate Aero test sources without executing them.
            let test_dirs = vec!["examples", "tests", "."];
            let mut test_count = 0;
            let mut completed_count = 0;

            println!(
                "\x1b[1;36mAnalyzing\x1b[0m Aero test sources (canonical checked admission; no execution)..."
            );
            for dir in &test_dirs {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if name.ends_with("_test.aero") || name.ends_with("_tests.aero") {
                                test_count += 1;
                                println!("\x1b[1;36mAnalyzing\x1b[0m {}", path.display());
                                match fs::read_to_string(&path) {
                                    Ok(src) => {
                                        let filename = path.to_string_lossy().to_string();
                                        match prepare_checked_program_with_module_observer(
                                            &src,
                                            Some(filename.clone()),
                                            Some(&filename),
                                            |_, _| {},
                                        ) {
                                            Ok(_) => {
                                                completed_count += 1;
                                                println!(
                                                    "      \x1b[1;32m✓\x1b[0m {} analysis completed (not executed)",
                                                    name
                                                );
                                            }
                                            Err(err) => {
                                                println!(
                                                    "      \x1b[1;31m✗\x1b[0m {} analysis failed: {}",
                                                    name, err
                                                );
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        println!(
                                            "      \x1b[1;31m✗\x1b[0m {} analysis failed: could not read test: {}",
                                            name, err
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if test_count == 0 {
                println!(
                    "\x1b[1;33mwarning\x1b[0m: no Aero test source files found (*_test.aero, *_tests.aero); no tests were executed"
                );
            } else {
                let failure_count = test_count - completed_count;
                println!(
                    "\n\x1b[1manalysis result\x1b[0m: {} completed, {} failed, {} total; no tests were executed",
                    completed_count, failure_count, test_count
                );
                if failure_count > 0 {
                    return CliStatus::OperationalFailure;
                }
            }
        }
        "fmt" => {
            if args.len() != 3 {
                eprintln!("Usage: {} fmt <input.aero>", args[0]);
                return CliStatus::InvocationFailure;
            }
            let input_file = &args[2];

            let source_code = match fs::read_to_string(input_file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!(
                        "\x1b[1;31merror\x1b[0m: could not read file {}: {}",
                        input_file, err
                    );
                    return CliStatus::OperationalFailure;
                }
            };

            // Basic formatting: normalize indentation and trailing whitespace
            let formatted: String = source_code
                .lines()
                .map(|line| line.trim_end())
                .collect::<Vec<&str>>()
                .join("\n");

            if let Err(err) = fs::write(input_file, &formatted) {
                eprintln!(
                    "\x1b[1;31merror\x1b[0m: could not write file {}: {}",
                    input_file, err
                );
                return CliStatus::OperationalFailure;
            }
            println!("\x1b[1;32m   Formatted\x1b[0m {}", input_file);
        }
        "doc" => {
            if args.len() < 3 {
                eprintln!("Usage: {} doc <input.aero> [-o <output.md>]", args[0]);
                return CliStatus::InvocationFailure;
            }

            let input_file = &args[2];
            let output_file = if args.len() == 5 && args[3] == "-o" {
                args[4].clone()
            } else if args.len() == 3 {
                default_doc_output_path(input_file)
            } else {
                eprintln!("Usage: {} doc <input.aero> [-o <output.md>]", args[0]);
                return CliStatus::InvocationFailure;
            };

            let source_code = match fs::read_to_string(input_file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!(
                        "\x1b[1;31merror\x1b[0m: could not read file {}: {}",
                        input_file, err
                    );
                    return CliStatus::OperationalFailure;
                }
            };

            match doc_generator::generate_markdown(input_file, &source_code) {
                Ok(markdown) => {
                    if let Err(err) = fs::write(&output_file, markdown) {
                        eprintln!(
                            "\x1b[1;31merror\x1b[0m: could not write docs {}: {}",
                            output_file, err
                        );
                        return CliStatus::OperationalFailure;
                    }
                    println!("Generated documentation at {}", output_file);
                }
                Err(err) => {
                    eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                    return CliStatus::OperationalFailure;
                }
            }
        }
        "profile" => {
            if args.len() < 3 {
                eprintln!("Usage: {} profile <input.aero> [-o <trace.json>]", args[0]);
                return CliStatus::InvocationFailure;
            }

            let input_file = &args[2];
            let trace_output = if args.len() == 5 && args[3] == "-o" {
                Some(args[4].as_str())
            } else if args.len() == 3 {
                None
            } else {
                eprintln!("Usage: {} profile <input.aero> [-o <trace.json>]", args[0]);
                return CliStatus::InvocationFailure;
            };

            let source_code = match fs::read_to_string(input_file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!(
                        "\x1b[1;31merror\x1b[0m: could not read file {}: {}",
                        input_file, err
                    );
                    return CliStatus::OperationalFailure;
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
                                return CliStatus::OperationalFailure;
                            }
                        }
                    }
                }
                Err(err) => {
                    eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                    return CliStatus::OperationalFailure;
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
                return CliStatus::InvocationFailure;
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
                            return CliStatus::InvocationFailure;
                        }
                        output_file = Some(args[i + 1].clone());
                        i += 2;
                    }
                    "--backend" => {
                        if i + 1 >= args.len() {
                            eprintln!("{}", graph_usage);
                            return CliStatus::InvocationFailure;
                        }
                        let Some(parsed) = AcceleratorBackend::parse(&args[i + 1]) else {
                            eprintln!(
                                "\x1b[1;31merror\x1b[0m: unsupported backend `{}`",
                                args[i + 1]
                            );
                            return CliStatus::InvocationFailure;
                        };
                        backend = parsed;
                        i += 2;
                    }
                    "--gpu" => {
                        if i + 1 >= args.len() {
                            eprintln!("{}", graph_usage);
                            return CliStatus::InvocationFailure;
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
                        return CliStatus::InvocationFailure;
                    }
                }
            }
            let Some(output_file) = output_file else {
                eprintln!("{}", graph_usage);
                return CliStatus::InvocationFailure;
            };

            let input = match fs::read_to_string(input_file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!(
                        "\x1b[1;31merror\x1b[0m: could not read file {}: {}",
                        input_file, err
                    );
                    return CliStatus::OperationalFailure;
                }
            };
            match verify_llvm_module(&input, LlvmVerificationMode::Required) {
                Ok(status) => println!("LLVM input verification: {status}"),
                Err(error) => {
                    eprintln!("\x1b[1;31merror\x1b[0m: {error}");
                    return CliStatus::OperationalFailure;
                }
            }

            let config = graph_compiler::GraphCompilationConfig {
                backend,
                executable_fusion,
                gpu_arch,
            };
            let (optimized, report) =
                graph_compiler::apply_advanced_graph_compilation_with_config(&input, &config);
            match verify_llvm_module(&optimized, LlvmVerificationMode::Required) {
                Ok(status) => println!("LLVM output verification: {status}"),
                Err(error) => {
                    eprintln!("\x1b[1;31merror\x1b[0m: {error}");
                    return CliStatus::OperationalFailure;
                }
            }
            if let Err(err) = fs::write(&output_file, optimized) {
                eprintln!(
                    "\x1b[1;31merror\x1b[0m: could not write file {}: {}",
                    output_file, err
                );
                return CliStatus::OperationalFailure;
            }
            println!("Wrote graph-optimized IR to {}", output_file);
            let gpu_arch = report.gpu_arch.as_deref().unwrap_or("n/a");
            println!(
                "execution_scope=internal-scalar-helper | device_execution=false | backend: {} | gpu metadata: {} | fused chains: {} | rewritten helper chains: {} | skipped chains: {} | total fused ops: {}",
                report.backend,
                gpu_arch,
                report.fused_kernel_count,
                report.executable_kernel_count,
                report.skipped_chains,
                report.total_fused_ops
            );
        }
        "quantize" => {
            let quant_usage = format!(
                "Usage: {} quantize <input.ll> -o <output.ll> --mode <int8|fp8-e4m3|fp8-e5m2> [--backend <cpu|cuda|rocm>] [--gpu <arch>] [--calibration <samples.json|samples.txt>] [--per-channel] [--annotation-only]",
                args[0]
            );
            if args.len() < 7 {
                eprintln!("{}", quant_usage);
                return CliStatus::InvocationFailure;
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
                            return CliStatus::InvocationFailure;
                        }
                        output_file = Some(args[i + 1].clone());
                        i += 2;
                    }
                    "--mode" => {
                        if i + 1 >= args.len() {
                            eprintln!("{}", quant_usage);
                            return CliStatus::InvocationFailure;
                        }
                        mode = quantization::QuantizationMode::parse(&args[i + 1]);
                        if mode.is_none() {
                            eprintln!(
                                "\x1b[1;31merror\x1b[0m: unsupported quantization mode `{}`",
                                args[i + 1]
                            );
                            return CliStatus::InvocationFailure;
                        }
                        i += 2;
                    }
                    "--backend" => {
                        if i + 1 >= args.len() {
                            eprintln!("{}", quant_usage);
                            return CliStatus::InvocationFailure;
                        }
                        let Some(parsed) = AcceleratorBackend::parse(&args[i + 1]) else {
                            eprintln!(
                                "\x1b[1;31merror\x1b[0m: unsupported backend `{}`",
                                args[i + 1]
                            );
                            return CliStatus::InvocationFailure;
                        };
                        backend = parsed;
                        i += 2;
                    }
                    "--gpu" => {
                        if i + 1 >= args.len() {
                            eprintln!("{}", quant_usage);
                            return CliStatus::InvocationFailure;
                        }
                        gpu_arch = Some(args[i + 1].clone());
                        i += 2;
                    }
                    "--calibration" => {
                        if i + 1 >= args.len() {
                            eprintln!("{}", quant_usage);
                            return CliStatus::InvocationFailure;
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
                        return CliStatus::InvocationFailure;
                    }
                }
            }

            let Some(output_file) = output_file else {
                eprintln!("{}", quant_usage);
                return CliStatus::InvocationFailure;
            };
            let Some(mode) = mode else {
                eprintln!("{}", quant_usage);
                return CliStatus::InvocationFailure;
            };

            let input = match fs::read_to_string(input_file) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!(
                        "\x1b[1;31merror\x1b[0m: could not read file {}: {}",
                        input_file, err
                    );
                    return CliStatus::OperationalFailure;
                }
            };
            match verify_llvm_module(&input, LlvmVerificationMode::Required) {
                Ok(status) => println!("LLVM input verification: {status}"),
                Err(error) => {
                    eprintln!("\x1b[1;31merror\x1b[0m: {error}");
                    return CliStatus::OperationalFailure;
                }
            }

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
                        return CliStatus::OperationalFailure;
                    }
                }
            }

            let (quantized_ir, report) =
                quantization::apply_quantization_interface(&input, &config);
            match verify_llvm_module(&quantized_ir, LlvmVerificationMode::Required) {
                Ok(status) => println!("LLVM output verification: {status}"),
                Err(error) => {
                    eprintln!("\x1b[1;31merror\x1b[0m: {error}");
                    return CliStatus::OperationalFailure;
                }
            }
            if let Err(err) = fs::write(&output_file, quantized_ir) {
                eprintln!(
                    "\x1b[1;31merror\x1b[0m: could not write file {}: {}",
                    output_file, err
                );
                return CliStatus::OperationalFailure;
            }
            println!("Wrote quantization IR to {}", output_file);
            let gpu_arch = report.gpu_arch.as_deref().unwrap_or("n/a");
            println!(
                "execution_scope=scalar-double-helper | device_execution=false | mode label: {} | backend metadata: {} | gpu metadata: {} | candidates: {} | rewritten ops: {} | scalar helpers: {} | calibration samples: {}",
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
        "registry" => {
            if args.len() < 3 {
                print_registry_help(&args[0]);
                return CliStatus::InvocationFailure;
            }

            match args[2].as_str() {
                "search" => {
                    if args.len() < 4 {
                        eprintln!(
                            "Usage: {} registry search <query> [--index <index.json>] [--registry <url>] [--live] [--token <token>] [--token-file <path>]",
                            args[0]
                        );
                        return CliStatus::InvocationFailure;
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
                                    return CliStatus::InvocationFailure;
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
                                    return CliStatus::InvocationFailure;
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
                                    return CliStatus::InvocationFailure;
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
                                    return CliStatus::InvocationFailure;
                                }
                                token_file = Some(args[i + 1].clone());
                                i += 2;
                            }
                            _ => {
                                eprintln!(
                                    "Usage: {} registry search <query> [--index <index.json>] [--registry <url>] [--live] [--token <token>] [--token-file <path>]",
                                    args[0]
                                );
                                return CliStatus::InvocationFailure;
                            }
                        }
                    }

                    if live && let Err(err) = guard_live_registry_transport_for_compiler_service() {
                        eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                        return CliStatus::OperationalFailure;
                    }

                    let client = registry::RegistryClient::new(registry_url.as_deref());
                    println!("Registry: {}", client.base_url);

                    let auth = if live {
                        match registry::resolve_registry_auth(
                            token.as_deref(),
                            token_file.as_deref().map(Path::new),
                        ) {
                            Ok(auth) => auth,
                            Err(err) => {
                                eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                                return CliStatus::OperationalFailure;
                            }
                        }
                    } else {
                        None
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
                            return CliStatus::OperationalFailure;
                        }
                    }
                }
                "publish" => {
                    if args.len() < 4 {
                        eprintln!(
                            "Usage: {} registry publish <package-dir> [--registry <url>] [--token <token>] [--token-file <path>] [--dry-run]",
                            args[0]
                        );
                        return CliStatus::InvocationFailure;
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
                                    return CliStatus::InvocationFailure;
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
                                    return CliStatus::InvocationFailure;
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
                                    return CliStatus::InvocationFailure;
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
                                return CliStatus::InvocationFailure;
                            }
                        }
                    }

                    let client = registry::RegistryClient::new(registry_url.as_deref());

                    if dry_run {
                        match registry::build_publish_preview(&client, Path::new(package_dir)) {
                            Ok(preview) => {
                                println!("Registry publish preview:");
                                match serde_json::to_string_pretty(&preview) {
                                    Ok(json) => println!("{}", json),
                                    Err(err) => {
                                        eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                                        return CliStatus::OperationalFailure;
                                    }
                                }
                            }
                            Err(err) => {
                                eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                                return CliStatus::OperationalFailure;
                            }
                        }
                    } else {
                        if let Err(err) = guard_live_registry_transport_for_compiler_service() {
                            eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                            return CliStatus::OperationalFailure;
                        }
                        let auth = match registry::resolve_registry_auth(
                            token.as_deref(),
                            token_file.as_deref().map(Path::new),
                        ) {
                            Ok(auth) => auth,
                            Err(err) => {
                                eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                                return CliStatus::OperationalFailure;
                            }
                        };
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
                                        return CliStatus::OperationalFailure;
                                    }
                                }
                            }
                            Err(err) => {
                                eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                                return CliStatus::OperationalFailure;
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
                        return CliStatus::InvocationFailure;
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
                                    return CliStatus::InvocationFailure;
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
                                    return CliStatus::InvocationFailure;
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
                                    return CliStatus::InvocationFailure;
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
                                    return CliStatus::InvocationFailure;
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
                                    return CliStatus::InvocationFailure;
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
                                    return CliStatus::InvocationFailure;
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
                                return CliStatus::InvocationFailure;
                            }
                        }
                    }

                    let client = registry::RegistryClient::new(registry_url.as_deref());

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
                                return CliStatus::OperationalFailure;
                            }
                        }
                    } else {
                        if let Err(err) = guard_live_registry_transport_for_compiler_service() {
                            eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                            return CliStatus::OperationalFailure;
                        }
                        let auth = match registry::resolve_registry_auth(
                            token.as_deref(),
                            token_file.as_deref().map(Path::new),
                        ) {
                            Ok(auth) => auth,
                            Err(err) => {
                                eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                                return CliStatus::OperationalFailure;
                            }
                        };
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
                                        return CliStatus::OperationalFailure;
                                    }
                                }
                            }
                            Err(err) => {
                                eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                                return CliStatus::OperationalFailure;
                            }
                        }
                    }
                }
                "help" | "--help" | "-h" => {
                    print_registry_help(&args[0]);
                    if args.len() != 3 {
                        return CliStatus::InvocationFailure;
                    }
                }
                _ => {
                    print_registry_help(&args[0]);
                    return CliStatus::InvocationFailure;
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
                            return CliStatus::InvocationFailure;
                        }
                        output_json = Some(args[i + 1].clone());
                        i += 2;
                    }
                    _ => {
                        eprintln!("Usage: {} conformance [-o <report.json>]", args[0]);
                        return CliStatus::InvocationFailure;
                    }
                }
            }

            let report = conformance::run_conformance_suite();
            let command = match run_conformance_command_with_report(
                report,
                output_json.as_deref().map(Path::new),
            ) {
                Ok(command) => command,
                Err(error) => {
                    eprintln!("\x1b[1;31merror\x1b[0m: {error}");
                    return CliStatus::OperationalFailure;
                }
            };
            println!("{}", command.stdout);
            if command.exit_code != 0 {
                return CliStatus::OperationalFailure;
            }
        }
        "init" => {
            if args.len() > 3 {
                eprintln!("Usage: {} init [path]", args[0]);
                return CliStatus::InvocationFailure;
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
                    return CliStatus::OperationalFailure;
                }
            }
        }
        "lsp" => {
            if args.len() != 2 {
                eprintln!("Usage: {} lsp", args[0]);
                return CliStatus::InvocationFailure;
            }
            if let Err(err) = lsp::run_language_server() {
                eprintln!("\x1b[1;31merror\x1b[0m: {}", err);
                return CliStatus::OperationalFailure;
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!(
                "Available commands: build, run, check, test, fmt, doc, profile, graph-opt, quantize, registry, conformance, init, lsp"
            );
            return CliStatus::InvocationFailure;
        }
    }

    CliStatus::Success
}

fn parse_check_args(args: &[String]) -> Result<(String, LanguageProfile), String> {
    let usage = || {
        format!(
            "Usage: {} check <input.aero> [--language-profile <experimental|stable-scalar-v0|exact-i32-array-v0|exact-i32-record-result-v0>]",
            args.first().map(String::as_str).unwrap_or("aero")
        )
    };
    if args.len() < 3 {
        return Err(usage());
    }

    let mut input_file = None;
    let mut language_profile = LanguageProfile::Experimental;
    let mut i = 2usize;
    while i < args.len() {
        match args[i].as_str() {
            "--language-profile" => {
                if i + 1 >= args.len() {
                    return Err(usage());
                }
                language_profile = args[i + 1].parse()?;
                i += 2;
            }
            value if value.starts_with('-') => {
                return Err(format!("error: unknown option `{value}`\n{}", usage()));
            }
            value => {
                if let Some(existing) = &input_file {
                    return Err(format!(
                        "error: multiple input files provided (`{existing}` and `{value}`)\n{}",
                        usage()
                    ));
                }
                input_file = Some(value.to_string());
                i += 1;
            }
        }
    }

    input_file
        .map(|input_file| (input_file, language_profile))
        .ok_or_else(usage)
}

fn build_usage(program_name: &str) -> String {
    format!(
        "Usage: {program_name} build <input.aero> -o <output.ll> [--target <cpu|rocm|cuda>] [--gpu <arch>] [--require-llvm-verifier] [--language-profile <experimental|stable-scalar-v0|exact-i32-array-v0|exact-i32-record-result-v0>]"
    )
}

fn run_usage(program_name: &str) -> String {
    format!(
        "Usage: {program_name} run <input.aero> [--target <cpu|rocm|cuda>] [--gpu <arch>] [--language-profile <experimental|stable-scalar-v0|exact-i32-array-v0|exact-i32-record-result-v0>]"
    )
}

fn parse_build_args(args: &[String]) -> Result<(String, String, BuildConfig), String> {
    let usage = || build_usage(args.first().map(String::as_str).unwrap_or("aero"));
    if args.len() < 3 {
        return Err(usage());
    }

    let input_file = args[2].clone();
    let mut output_file: Option<String> = None;
    let mut config = BuildConfig::default();
    let mut i = 3usize;
    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                if i + 1 >= args.len() {
                    return Err(usage());
                }
                output_file = Some(args[i + 1].clone());
                i += 2;
            }
            "--target" | "--backend" => {
                if i + 1 >= args.len() {
                    return Err(usage());
                }
                config.target = parse_explicit_build_target(&args[i + 1])?;
                i += 2;
            }
            "--gpu" => {
                if i + 1 >= args.len() {
                    return Err(usage());
                }
                config.gpu_arch = Some(args[i + 1].clone());
                i += 2;
            }
            "--require-llvm-verifier" => {
                config.require_llvm_verifier = true;
                i += 1;
            }
            "--language-profile" => {
                if i + 1 >= args.len() {
                    return Err(usage());
                }
                config.language_profile = args[i + 1].parse()?;
                i += 2;
            }
            _ => {
                return Err(usage());
            }
        }
    }

    let Some(output_file) = output_file else {
        return Err(usage());
    };

    config.validate_language_profile_target()?;
    Ok((input_file, output_file, config))
}

fn parse_run_args(args: &[String]) -> Result<(String, BuildConfig), String> {
    let usage = || run_usage(args.first().map(String::as_str).unwrap_or("aero"));
    if args.len() < 3 {
        return Err(usage());
    }

    let mut input_file: Option<String> = None;
    let mut config = BuildConfig::default();

    let mut i = 2usize;
    while i < args.len() {
        match args[i].as_str() {
            "--target" | "--backend" => {
                if i + 1 >= args.len() {
                    return Err(usage());
                }
                config.target = parse_explicit_build_target(&args[i + 1])?;
                i += 2;
            }
            "--gpu" => {
                if i + 1 >= args.len() {
                    return Err(usage());
                }
                config.gpu_arch = Some(args[i + 1].clone());
                i += 2;
            }
            "--language-profile" => {
                if i + 1 >= args.len() {
                    return Err(usage());
                }
                config.language_profile = args[i + 1].parse()?;
                i += 2;
            }
            value if value.starts_with('-') => {
                return Err(format!("error: unknown option `{}`\n{}", value, usage()));
            }
            value => {
                if input_file.is_some() {
                    let existing = input_file.as_deref().unwrap_or("<unknown>");
                    return Err(format!(
                        "error: multiple input files provided (`{}` and `{}`)\n{}",
                        existing,
                        value,
                        usage()
                    ));
                }
                input_file = Some(value.to_string());
                i += 1;
            }
        }
    }

    let Some(input_file) = input_file else {
        return Err(usage());
    };

    config.validate_language_profile_target()?;
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
) -> Result<(), String> {
    let mut perf_optimizer = PerformanceOptimizer::new();
    compile_to_llvm_ir_with_optimizer(
        source_code,
        output_file,
        input_file,
        build_config,
        &mut perf_optimizer,
    )
}

fn push_compilation_cache_frame(bytes: &mut Vec<u8>, label: &str, payload: &[u8]) {
    bytes.extend_from_slice(label.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(&(payload.len() as u64).to_be_bytes());
    bytes.extend_from_slice(payload);
}

fn compilation_cache_key(
    source_code: &str,
    build_config: &BuildConfig,
    direct_module_cache_material: Option<&[u8]>,
) -> String {
    let Some(direct_module_cache_material) = direct_module_cache_material else {
        let profile_material = match build_config.language_profile {
            LanguageProfile::Experimental => String::new(),
            profile => format!("::language_profile={}", profile.as_str()),
        };
        return format!(
            "{:x}",
            md5::compute(format!(
                "{}::target={}::gpu={}{}",
                source_code,
                build_config.target.as_str(),
                build_config.gpu_arch_or_default(),
                profile_material
            ))
        );
    };

    let mut bytes = b"AERO_MODULE_CACHE_V1\0".to_vec();
    push_compilation_cache_frame(&mut bytes, "root", source_code.as_bytes());
    push_compilation_cache_frame(
        &mut bytes,
        "target",
        build_config.target.as_str().as_bytes(),
    );
    push_compilation_cache_frame(
        &mut bytes,
        "gpu",
        build_config.gpu_arch_or_default().as_bytes(),
    );
    if build_config.language_profile != LanguageProfile::Experimental {
        push_compilation_cache_frame(
            &mut bytes,
            "language-profile",
            build_config.language_profile.as_str().as_bytes(),
        );
    }
    bytes.extend_from_slice(direct_module_cache_material);

    format!("{:x}", md5::compute(bytes))
}

fn compile_to_llvm_ir_with_optimizer(
    source_code: &str,
    output_file: &str,
    input_file: &str,
    build_config: &BuildConfig,
    perf_optimizer: &mut PerformanceOptimizer,
) -> Result<(), String> {
    println!(
        "Compiling with performance optimizations enabled (target: {}, gpu: {})",
        build_config.target.as_str(),
        build_config.gpu_arch_or_default()
    );

    let compilation_start = Instant::now();
    let verification_mode = build_config.llvm_verification_mode();

    let checked_program = prepare_checked_program_with_module_observer_and_profile(
        source_code,
        Some(input_file.to_string()),
        Some(input_file),
        build_config.language_profile,
        |name, path| println!("  Resolved module `{name}` → {}", path.display()),
    )?;
    let pipeline_timings = checked_program.timings();
    println!("Lexing completed in {:?}", pipeline_timings.lexing);
    println!(
        "Optimized parsing completed in {:?}",
        pipeline_timings.parsing
    );

    // The canonical checked pipeline is mandatory before a verified cache lookup. A
    // cache hit may bypass only checked code generation and later transformations.
    let source_hash = compilation_cache_key(
        source_code,
        build_config,
        checked_program.direct_module_cache_material(),
    );
    if let Some(cached_llvm) = perf_optimizer
        .get_compilation_cache()
        .get_cached_llvm(&source_hash)
    {
        let status = verify_llvm_module(&cached_llvm, verification_mode)
            .map_err(|error| error.to_string())?;
        if status.external_verifier().is_some() {
            println!("Using cached compilation result");
            println!("LLVM verification: {status}");
            fs::write(output_file, cached_llvm)
                .map_err(|err| format!("Error writing cached result: {err}"))?;
            println!("Cached LLVM IR written to {}", output_file);
            println!("{}", perf_optimizer.get_performance_report());
            return Ok(());
        }
        println!("Cached result bypassed because external LLVM verification is unavailable");
    }

    println!(
        "Semantic Analysis Result: {}",
        checked_program.semantic_message()
    );
    println!(
        "Optimized semantic analysis completed in {:?}",
        pipeline_timings.semantics
    );
    println!(
        "Optimized IR generation completed in {:?}",
        pipeline_timings.checked_ir
    );

    // Optimized code generation with control flow optimizations
    let codegen_start = Instant::now();
    let llvm_ir = checked_program
        .try_generate_llvm()
        .map_err(render_code_generation_error)?;
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

    let verification_status =
        verify_llvm_module(&llvm_ir, verification_mode).map_err(|error| error.to_string())?;
    println!("LLVM verification: {verification_status}");

    // Cache only the exact final bytes that passed the selected verification policy.
    perf_optimizer
        .get_compilation_cache()
        .cache_llvm(source_hash, llvm_ir.clone());

    // Write to output file
    fs::write(output_file, &llvm_ir)
        .map_err(|err| format!("Error writing to file {}: {}", output_file, err))?;
    println!("Optimized LLVM IR written to {}", output_file);

    let total_time = compilation_start.elapsed();
    println!("Total compilation time: {:?}", total_time);

    // Print comprehensive performance report
    println!("{}", perf_optimizer.get_performance_report());

    println!("Performance-optimized compilation process completed successfully.");
    Ok(())
}

fn run_aero_program(
    source_code: &str,
    input_file: &str,
    build_config: &BuildConfig,
) -> Result<(), String> {
    let artifacts = create_run_artifact_paths(input_file, build_config)?;
    let result = run_aero_program_with_artifacts(source_code, input_file, build_config, &artifacts);
    let cleanup = fs::remove_dir_all(&artifacts.directory).map_err(|error| {
        format!(
            "failed to remove compile artifact directory {}: {}",
            artifacts.directory.display(),
            error
        )
    });

    match (result, cleanup) {
        (Ok(Some(exit_code)), Ok(())) => exit(exit_code),
        (Ok(None), Ok(())) => Ok(()),
        (Ok(_), Err(cleanup_error)) => Err(cleanup_error),
        (Err(error), Ok(())) => Err(error),
        (Err(error), Err(cleanup_error)) => Err(format!("{error}; {cleanup_error}")),
    }
}

fn run_aero_program_with_artifacts(
    source_code: &str,
    input_file: &str,
    build_config: &BuildConfig,
    artifacts: &RunArtifactPaths,
) -> Result<Option<i32>, String> {
    let ll_path = artifacts.ll_file.to_string_lossy().to_string();
    let obj_path = artifacts.obj_file.to_string_lossy().to_string();
    let runtime_obj_path = artifacts.runtime_obj_file.to_string_lossy().to_string();
    let exe_path = artifacts.exe_file.to_string_lossy().to_string();
    let gpu_obj_path = artifacts.gpu_obj_file.to_string_lossy().to_string();

    // Executing or producing native objects always requires an external LLVM 22 verifier.
    let mut verified_build_config = build_config.clone();
    verified_build_config.require_llvm_verifier = true;
    compile_to_llvm_ir(source_code, &ll_path, input_file, &verified_build_config)?;
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
            compile_production_runtime(
                &clang_bin,
                &artifacts.runtime_source_file,
                &artifacts.runtime_obj_file,
            )?;

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
                    .args([&obj_path, &runtime_obj_path, "-o", &exe_path])
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
                    .args([&ll_path, &runtime_obj_path, "-o", &exe_path])
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
            if exit_code == 0 {
                println!("Program executed successfully.");
            }
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

            // The wrapper removes every temporary artifact before mirroring the
            // executed program's exit code.
            return Ok(Some(exit_code));
        }
        BuildTarget::Rocm => {
            let llc_bin = find_llvm_tool("llc").ok_or_else(|| {
                format!(
                    "Error executing llc for ROCm target: program not found. Make sure LLVM is installed and llc is in your PATH. Temporary run artifacts will be removed."
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
                    if !artifacts.gpu_obj_file.is_file() {
                        return Err(
                            "ROCm object generation failed: llc reported success but did not create the requested regular object file."
                                .to_string(),
                        );
                    }
                    println!(
                        "ROCm object stage complete: llc produced a temporary file; no link or execution occurred."
                    );
                    return Err(
                        "ROCm run is unavailable: HIP link and device launch are not implemented; no program was executed."
                            .to_string(),
                    );
                }
                Err(err) => {
                    return Err(format!(
                        "Error executing llc for ROCm target ({}): {}. Temporary run artifacts will be removed.",
                        llc_bin, err
                    ));
                }
            }
        }
        BuildTarget::Cuda => {
            return Err(
                "CUDA run is unavailable: object generation, link, and device launch are not implemented; no program was executed. Use --target cpu for execution."
                    .to_string(),
            );
        }
    }
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
    println!(
        "Aero Programming Language Compiler v{}",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("USAGE:");
    println!("    {} <COMMAND> [OPTIONS]", program_name);
    println!();
    println!("COMMANDS:");
    println!(
        "    build <input.aero> -o <output.ll>    Compile Aero source to LLVM IR [--target <cpu|rocm|cuda>] [--gpu <arch>] [--language-profile <name>]"
    );
    println!(
        "    run <input.aero>                     Compile source; execution availability depends on target [--target <cpu|rocm|cuda>] [--gpu <arch>] [--language-profile <name>]"
    );
    println!(
        "    check <input.aero>                   Validate frontend and checked IR (no LLVM emission) [--language-profile <name>]"
    );
    println!(
        "    test                                 Discover and semantically analyze *_test.aero files (no execution)"
    );
    println!("    fmt <input.aero>                     Auto-format Aero source");
    println!("    doc <input.aero> [-o <output.md>]    Generate Markdown API docs from source");
    println!("    profile <input.aero> [-o <trace.json>] Profile compilation phases");
    println!(
        "    graph-opt <input.ll> -o <output.ll>  graph-opt: verified textual internal scalar-helper rewrite; device_execution=false [--backend <cpu|cuda|rocm>] [--gpu <arch>]"
    );
    println!("    quantize <input.ll> -o <output.ll> --mode <int8|fp8-e4m3|fp8-e5m2>");
    println!(
        "                                         quantize: scalar-double helper rewrite; no real FP8, per-channel execution, numerical proof, or device execution [--backend <cpu|cuda|rocm>] [--gpu <arch>]"
    );
    println!(
        "    registry <subcommand>                Search a local index or create network-free publish/install previews"
    );
    println!("    conformance [-o <report.json>]       Run deterministic regression checks");
    println!("    init [path]                          Initialize a new Aero project");
    println!("    lsp                                  Run Aero language server (stdio)");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Print this help message");
    println!("    -v, --version    Print version information");
    println!(
        "    --language-profile <experimental|stable-scalar-v0|exact-i32-array-v0|exact-i32-record-result-v0>  Select the compiler-enforced source profile"
    );
    println!();
    println!("EXECUTION BOUNDARIES:");
    println!("    CPU is the only current process-execution target.");
    println!("    ROCm run probes temporary object emission but has no HIP link or device launch.");
    println!("    CUDA run has no object, link, or device-launch path.");
    println!();
    println!("EXAMPLES:");
    println!("    {} build hello.aero -o hello.ll", program_name);
    println!(
        "    {} build hello.aero -o hello.rocm.ll --target rocm --gpu gfx1101",
        program_name
    );
    println!("    {} run hello.aero", program_name);
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
        "    {} registry search vision --index registry/index.json",
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
    println!(
        "    Local index search and publish/install --dry-run are credential-free and network-free."
    );
    println!(
        "    --live and publish/install without --dry-run fail: {}.",
        LIVE_REGISTRY_DISABLED_FOR_COMPILER_SERVICE
    );
}

/// Validate an Aero program without emitting LLVM or consulting external tools.
/// Runs lexer → parser → direct modules → semantics → checked IR/internal verification.
fn check_aero_program(
    source_code: &str,
    input_file: &str,
    language_profile: LanguageProfile,
) -> Result<(), String> {
    let check_start = Instant::now();
    let checked_program = prepare_checked_program_with_module_observer_and_profile(
        source_code,
        Some(input_file.to_string()),
        Some(input_file),
        language_profile,
        |_, _| {},
    )?;
    let elapsed = check_start.elapsed();
    println!(
        "\x1b[1;32m    Checking\x1b[0m {} ... \x1b[1;32mok\x1b[0m ({:?})",
        input_file, elapsed
    );
    println!("  {}", checked_program.semantic_message());
    Ok(())
}

fn report_check_error(source_code: &str, input_file: &str, error: &str) {
    // Enhanced error display with color and source context
    let lines: Vec<&str> = source_code.lines().collect();
    eprintln!("\x1b[1;31merror\x1b[0m: {}", error);

    // Try to extract line number from error message
    if let Some(line_hint) = extract_error_line(error) {
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
    if error.contains("undefined") || error.contains("not found") {
        eprintln!("\x1b[1;36mhelp\x1b[0m: check the spelling or ensure the variable is in scope");
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
    fn language_profile_option_is_shared_by_check_build_and_run_parsers() {
        for (name, expected) in [
            ("stable-scalar-v0", LanguageProfile::StableScalarV0),
            ("exact-i32-array-v0", LanguageProfile::ExactI32ArrayV0),
            (
                "exact-i32-record-result-v0",
                LanguageProfile::ExactI32RecordResultV0,
            ),
        ] {
            let check = vec![
                "aero".to_string(),
                "check".to_string(),
                "main.aero".to_string(),
                "--language-profile".to_string(),
                name.to_string(),
            ];
            let (check_input, check_profile) = parse_check_args(&check).expect("check args");
            assert_eq!(check_input, "main.aero");
            assert_eq!(check_profile, expected);

            let build = vec![
                "aero".to_string(),
                "build".to_string(),
                "main.aero".to_string(),
                "-o".to_string(),
                "main.ll".to_string(),
                "--language-profile".to_string(),
                name.to_string(),
            ];
            let (_, _, build_config) = parse_build_args(&build).expect("build args");
            assert_eq!(build_config.language_profile, expected);

            let run = vec![
                "aero".to_string(),
                "run".to_string(),
                "--language-profile".to_string(),
                name.to_string(),
                "main.aero".to_string(),
            ];
            let (run_input, run_config) = parse_run_args(&run).expect("run args");
            assert_eq!(run_input, "main.aero");
            assert_eq!(run_config.language_profile, expected);
        }
    }

    #[test]
    fn exact_profiles_have_distinct_cache_identity_without_changing_default_keys() {
        let experimental = BuildConfig::default();
        let stable = BuildConfig {
            language_profile: LanguageProfile::StableScalarV0,
            ..BuildConfig::default()
        };
        let exact_array = BuildConfig {
            language_profile: LanguageProfile::ExactI32ArrayV0,
            ..BuildConfig::default()
        };
        let exact_record_result = BuildConfig {
            language_profile: LanguageProfile::ExactI32RecordResultV0,
            ..BuildConfig::default()
        };
        let source = "fn main() -> int { return 0; }";

        for modules in [None, Some(b"module-frame".as_slice())] {
            let experimental_key = compilation_cache_key(source, &experimental, modules);
            let stable_key = compilation_cache_key(source, &stable, modules);
            let exact_array_key = compilation_cache_key(source, &exact_array, modules);
            let exact_record_result_key =
                compilation_cache_key(source, &exact_record_result, modules);
            assert_ne!(experimental_key, stable_key);
            assert_ne!(experimental_key, exact_array_key);
            assert_ne!(experimental_key, exact_record_result_key);
            assert_ne!(stable_key, exact_array_key);
            assert_ne!(stable_key, exact_record_result_key);
            assert_ne!(exact_array_key, exact_record_result_key);
        }
    }

    #[test]
    fn exact_profiles_reject_every_accelerator_selector_in_build_and_run() {
        let cases = [
            vec![
                "aero",
                "build",
                "main.aero",
                "-o",
                "main.ll",
                "--language-profile",
                "stable-scalar-v0",
                "--target",
                "rocm",
            ],
            vec![
                "aero",
                "build",
                "main.aero",
                "-o",
                "main.ll",
                "--language-profile",
                "stable-scalar-v0",
                "--gpu",
                "gfx1100",
            ],
            vec![
                "aero",
                "run",
                "main.aero",
                "--language-profile",
                "stable-scalar-v0",
                "--target",
                "cuda",
            ],
            vec![
                "aero",
                "run",
                "main.aero",
                "--language-profile",
                "stable-scalar-v0",
                "--gpu",
                "sm_90",
            ],
        ];

        for profile in [
            "stable-scalar-v0",
            "exact-i32-array-v0",
            "exact-i32-record-result-v0",
        ] {
            for mut arguments in cases.clone() {
                let profile_index = arguments
                    .iter()
                    .position(|argument| *argument == "stable-scalar-v0")
                    .expect("profile argument");
                arguments[profile_index] = profile;
                let arguments = arguments
                    .into_iter()
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                let error = if arguments[1] == "build" {
                    parse_build_args(&arguments).expect_err("exact build selector must fail")
                } else {
                    parse_run_args(&arguments).expect_err("exact run selector must fail")
                };
                assert_eq!(
                    error,
                    format!(
                        "Language Profile Error: {profile} requires --target cpu without --gpu"
                    )
                );
            }
        }
    }

    #[test]
    fn parse_run_args_rejects_ambiguous_gpu_target() {
        let args = vec![
            "aero".to_string(),
            "run".to_string(),
            "--target".to_string(),
            "gpu".to_string(),
            "examples/gguf_inference.aero".to_string(),
        ];
        let error = parse_run_args(&args).expect_err("run args must reject --target gpu");
        assert_eq!(
            error,
            "target `gpu` is ambiguous and does not prove a usable device; choose cpu, rocm, or cuda explicitly"
        );
    }

    #[test]
    fn retarget_llvm_module_switches_triple_for_rocm() {
        let input = "; ModuleID = \"a\"\nsource_filename = \"a\"\ntarget datalayout = \"old\"\ntarget triple = \"old\"\n\ndefine i32 @main() {\nentry:\n  ret i32 0\n}\n";
        let config = BuildConfig {
            target: BuildTarget::Rocm,
            gpu_arch: Some("gfx1101".to_string()),
            require_llvm_verifier: false,
            language_profile: LanguageProfile::Experimental,
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
            require_llvm_verifier: false,
            language_profile: LanguageProfile::Experimental,
        };
        let artifacts = create_run_artifact_paths("examples/hello.aero", &config)
            .expect("paths should be created");
        let dir = artifacts.directory.to_string_lossy();
        assert!(dir.contains("target"));
        assert!(dir.contains("aero-run"));
        assert!(artifacts.ll_file.to_string_lossy().ends_with(".ll"));
        assert!(
            artifacts
                .runtime_source_file
                .to_string_lossy()
                .ends_with("aero_runtime.c")
        );
        assert!(
            artifacts
                .runtime_obj_file
                .to_string_lossy()
                .ends_with("aero_runtime.o")
        );
        assert!(artifacts.gpu_obj_file.to_string_lossy().contains("gfx1101"));
        let _ = fs::remove_dir_all(artifacts.directory);
    }
}
