use compiler::accelerator::AcceleratorBackend;
use compiler::graph_compiler::{
    GraphCompilationConfig, apply_advanced_graph_compilation_with_config,
};
use compiler::quantization::{QuantizationConfig, QuantizationMode, apply_quantization_interface};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn repository_file(path: &str) -> String {
    let full_path = repository_root().join(path);
    fs::read_to_string(&full_path)
        .unwrap_or_else(|error| panic!("could not read {}: {error}", full_path.display()))
}

fn run_aero(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aero"))
        .args(args)
        .output()
        .expect("run Aero CLI")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn without_graph_scope_telemetry(output: &str) -> String {
    output
        .split_inclusive('\n')
        .filter(|line| {
            !matches!(
                *line,
                "; aero.graph_compilation.execution_scope=internal-scalar-helper\n"
                    | "; aero.graph_compilation.device_execution=false\n"
            )
        })
        .collect()
}

fn without_quant_scope_telemetry(output: &str) -> String {
    output
        .split_inclusive('\n')
        .filter(|line| {
            !matches!(
                *line,
                "; aero.quantization.execution_scope=scalar-double-helper\n"
                    | "; aero.quantization.device_execution=false\n"
            )
        })
        .collect()
}

fn assert_exactly_one_line(output: &str, expected: &str) {
    assert_eq!(
        output.lines().filter(|line| *line == expected).count(),
        1,
        "expected exactly one {expected:?} telemetry line"
    );
}

fn expected_quant_runtime_output(
    mode: QuantizationMode,
    expected_backend: &str,
    expected_mode: &str,
    gpu_arch: Option<&str>,
    op: &str,
) -> String {
    let (scale, max_mag) = match mode {
        QuantizationMode::Int8 => (127.0, None),
        QuantizationMode::Fp8E4M3 => (448.0, Some(448.0)),
        QuantizationMode::Fp8E5M2 => (57344.0, Some(57344.0)),
    };
    let inv_scale = 1.0 / scale;
    let helper = format!(
        "aero_{}_{}_{}",
        expected_backend,
        expected_mode.replace('-', "_"),
        op
    );
    let mut expected = format!(
        "; aero.quantization.mode={}\n; aero.quantization.backend={}\n",
        expected_mode, expected_backend
    );
    if let Some(gpu_arch) = gpu_arch {
        expected.push_str(&format!("; aero.quantization.gpu_arch={gpu_arch}\n"));
    }
    expected.push_str(&format!(
        "; aero.quantization.per_channel=true\n; aero.quantization.runtime_lowering=true\n; aero.quantization.calibration.scale={scale:.8}\n; aero.quantization.calibration.samples=4\ndefine i32 @main() {{\nentry:\n  ; aero.quantize candidate=1 helper={helper} scale={scale:.8}\n  %0 = call double @{helper}(double %a, double %b)\n  ret i32 0\n}}\n\ndefine internal i32 @aero_quant_clamp_i32(i32 %v, i32 %lo, i32 %hi) {{\nentry:\n  %lt = icmp slt i32 %v, %lo\n  %gt = icmp sgt i32 %v, %hi\n  %lowered = select i1 %lt, i32 %lo, i32 %v\n  %clamped = select i1 %gt, i32 %hi, i32 %lowered\n  ret i32 %clamped\n}}\n"
    ));
    if max_mag.is_some() {
        expected.push_str(
            "define internal double @aero_quant_clamp_f64(double %v, double %lo, double %hi) {\nentry:\n  %lt = fcmp olt double %v, %lo\n  %gt = fcmp ogt double %v, %hi\n  %lowered = select i1 %lt, double %lo, double %v\n  %clamped = select i1 %gt, double %hi, double %lowered\n  ret double %clamped\n}\n",
        );
    }
    expected.push('\n');
    match max_mag {
        None => expected.push_str(&format!(
            "; aero.quantization.runtime helper for mode=int8\ndefine internal double @{helper}(double %a, double %b) {{\nentry:\n  %qa = fmul double %a, {scale:.12}\n  %qb = fmul double %b, {scale:.12}\n  %ia = fptosi double %qa to i32\n  %ib = fptosi double %qb to i32\n  %ca = call i32 @aero_quant_clamp_i32(i32 %ia, i32 -127, i32 127)\n  %cb = call i32 @aero_quant_clamp_i32(i32 %ib, i32 -127, i32 127)\n  %fca = sitofp i32 %ca to double\n  %fcb = sitofp i32 %cb to double\n  %qres = {op} double %fca, %fcb\n  %ires = fptosi double %qres to i32\n  %cres = call i32 @aero_quant_clamp_i32(i32 %ires, i32 -127, i32 127)\n  %fq = sitofp i32 %cres to double\n  %dres = fmul double %fq, {inv_scale:.12}\n  ret double %dres\n}}\n"
        )),
        Some(max_mag) => expected.push_str(&format!(
            "; aero.quantization.runtime helper for mode={}\ndefine internal double @{helper}(double %a, double %b) {{\nentry:\n  %qa = fmul double %a, {scale:.12}\n  %qb = fmul double %b, {scale:.12}\n  %ca = call double @aero_quant_clamp_f64(double %qa, double -{max_mag:.12}, double {max_mag:.12})\n  %cb = call double @aero_quant_clamp_f64(double %qb, double -{max_mag:.12}, double {max_mag:.12})\n  %qres = {op} double %ca, %cb\n  %cres = call double @aero_quant_clamp_f64(double %qres, double -{max_mag:.12}, double {max_mag:.12})\n  %dres = fmul double %cres, {inv_scale:.12}\n  ret double %dres\n}}\n",
            expected_mode
        )),
    }
    expected
}

#[test]
fn formal_spec_remains_visibly_design_only() {
    let specification = repository_file("docs/language/aero_formal_language_specification.md");
    assert!(specification.contains("Design target — not current implementation evidence."));
    assert!(specification.contains("not a conformance or stability claim"));
}

#[test]
fn immutable_claims_keep_gguf_external_and_aero_gpu_claims_blocked() {
    let claims: Value = serde_json::from_str(&repository_file("claim-verification/claims.json"))
        .expect("parse immutable claim index");
    let claims = claims["claims"].as_array().expect("claims array");
    let gguf = claims
        .iter()
        .find(|claim| claim["id"] == "aero_public_master_gguf_llama_cli_20260528")
        .expect("external GGUF claim");
    assert_eq!(gguf["status"], "verified_reference_only");
    assert!(
        gguf["claim"]
            .as_str()
            .expect("GGUF claim text")
            .contains("not an Aero-vs-llama.cpp-vs-PyTorch comparison")
    );
    let gpu = claims
        .iter()
        .find(|claim| claim["id"] == "aero_public_master_gpu_claims_20260528")
        .expect("blocked Aero GPU claim");
    assert_eq!(gpu["status"], "blocked");
}

#[test]
fn cli_help_separates_cpu_execution_from_accelerator_stages() {
    let output = run_aero(&["--help"]);
    assert!(output.status.success());
    let help = stdout(&output);
    assert!(help.contains("CPU is the only current process-execution target."));
    assert!(help.contains(
        "ROCm run probes temporary object emission but has no HIP link or device launch."
    ));
    assert!(help.contains("CUDA run has no object, link, or device-launch path."));
    assert!(help.contains(
        "graph-opt: verified textual internal scalar-helper rewrite; device_execution=false"
    ));
    assert!(help.contains("quantize: scalar-double helper rewrite; no real FP8, per-channel execution, numerical proof, or device execution"));
    assert!(!help.contains("cpu|rocm|cuda|gpu"));
    assert!(!help.contains("run --target rocm"));
    assert!(!help.contains("Apply graph compilation and executable kernel fusion"));
    assert!(!help.contains("Apply calibrated INT8/FP8 lowering interface"));
}

#[test]
fn current_readme_build_and_tutorial_claim_only_observed_stages() {
    let readme = repository_file("README.md");
    let build = repository_file("BUILD.md");
    let tutorial = repository_file("tutorials/01-getting-started.md");
    for (label, text) in [
        ("README", readme.as_str()),
        ("BUILD", build.as_str()),
        ("tutorial", tutorial.as_str()),
    ] {
        assert!(
            text.contains("CPU is the only current process-execution target"),
            "{label} omits CPU execution boundary"
        );
        assert!(
            text.contains("ROCm") && text.contains("temporary object"),
            "{label} omits ROCm object-only stage"
        );
        assert!(
            text.contains("internal scalar helpers") && text.contains("device execution"),
            "{label} omits graph stage boundary"
        );
        assert!(
            text.contains("scalar-`double` helpers")
                && text.contains("no real FP8")
                && text.contains("numerical"),
            "{label} omits quantization boundary"
        );
    }
    for phrase in [
        "aero run --target rocm",
        "aero run --backend rocm",
        "aero run --target gpu",
        "Hardware-calibrated INT8/FP8 lowering",
        "executable fused-kernel backend generation",
    ] {
        assert!(!readme.contains(phrase), "README retains {phrase:?}");
    }
    for phrase in [
        "cpu|rocm|cuda|gpu",
        "compile and run an Aero program",
        "executable fused-kernel lowering",
        "hardware-calibrated INT8/FP8 lowering",
    ] {
        assert!(!build.contains(phrase), "BUILD retains {phrase:?}");
    }
    for phrase in [
        "aero run --target rocm",
        "executable kernel fusion",
        "backend-aware fused kernels",
        "calibrated quantization lowering",
        "calibrated quantization helpers",
    ] {
        assert!(!tutorial.contains(phrase), "tutorial retains {phrase:?}");
    }
}

#[test]
fn graph_transform_emits_non_device_scope_telemetry() {
    let input = "define i32 @main() {\nentry:\n  %0 = fadd double %a, %b\n  %1 = fmul double %0, %c\n  ret i32 0\n}\n";
    let config = GraphCompilationConfig {
        backend: AcceleratorBackend::Rocm,
        gpu_arch: Some("gfx1101".to_string()),
        executable_fusion: true,
    };
    let (output, report) = apply_advanced_graph_compilation_with_config(input, &config);
    assert_eq!(
        without_graph_scope_telemetry(&output),
        "; aero.graph_compilation=enabled\n; aero.graph_compilation.backend=rocm\n; aero.graph_compilation.gpu_arch=gfx1101\n; aero.graph_compilation.executable_fusion=true\ndefine i32 @main() {\nentry:\n  ; aero.fused_kernel id=1 backend=rocm ops=2 range=3..4 executable=true helper=@aero_fused_rocm_kernel_1\n  %1 = call double @aero_fused_rocm_kernel_1(double %a, double %b, double %c)\n  ret i32 0\n}\n\ndefine internal double @aero_fused_rocm_kernel_1(double %arg0, double %arg1, double %arg2) {\nentry:\n  %r0 = fadd double %arg0, %arg1\n  %r1 = fmul double %r0, %arg2\n  ret double %r1\n}\n\n"
    );
    let alternate_input = "define i32 @main() {\nentry:\n  %0 = fsub double %a, %b\n  %1 = fdiv double %0, %c\n  ret i32 0\n}\n";
    let (alternate_output, _) =
        apply_advanced_graph_compilation_with_config(alternate_input, &config);
    assert_eq!(
        without_graph_scope_telemetry(&alternate_output),
        "; aero.graph_compilation=enabled\n; aero.graph_compilation.backend=rocm\n; aero.graph_compilation.gpu_arch=gfx1101\n; aero.graph_compilation.executable_fusion=true\ndefine i32 @main() {\nentry:\n  ; aero.fused_kernel id=1 backend=rocm ops=2 range=3..4 executable=true helper=@aero_fused_rocm_kernel_1\n  %1 = call double @aero_fused_rocm_kernel_1(double %a, double %b, double %c)\n  ret i32 0\n}\n\ndefine internal double @aero_fused_rocm_kernel_1(double %arg0, double %arg1, double %arg2) {\nentry:\n  %r0 = fsub double %arg0, %arg1\n  %r1 = fdiv double %r0, %arg2\n  ret double %r1\n}\n\n"
    );
    assert_exactly_one_line(
        &alternate_output,
        "; aero.graph_compilation.execution_scope=internal-scalar-helper",
    );
    assert_exactly_one_line(
        &alternate_output,
        "; aero.graph_compilation.device_execution=false",
    );
    assert_exactly_one_line(
        &output,
        "; aero.graph_compilation.execution_scope=internal-scalar-helper",
    );
    assert_exactly_one_line(&output, "; aero.graph_compilation.device_execution=false");
    assert_eq!(report.fused_kernel_count, 1);
    assert_eq!(report.executable_kernel_count, 1);
    assert_eq!(report.backend, "rocm");
    assert_eq!(report.gpu_arch.as_deref(), Some("gfx1101"));
    assert_eq!(report.total_fused_ops, 2);
    assert_eq!(report.skipped_chains, 0);
    assert_eq!(report.kernels.len(), 1);
    let kernel = &report.kernels[0];
    assert_eq!(kernel.id, 1);
    assert_eq!(kernel.backend, "rocm");
    assert_eq!(kernel.gpu_arch.as_deref(), Some("gfx1101"));
    assert_eq!(kernel.op_count, 2);
    assert_eq!((kernel.start_line, kernel.end_line), (3, 4));
    assert!(kernel.executable);
    assert_eq!(
        kernel.helper_name.as_deref(),
        Some("aero_fused_rocm_kernel_1")
    );
    assert_eq!(kernel.fallback_reason, None);
    assert_eq!(
        serde_json::to_value(&report).expect("serialize graph report"),
        json!({
            "backend": "rocm",
            "gpu_arch": "gfx1101",
            "fused_kernel_count": 1,
            "executable_kernel_count": 1,
            "total_fused_ops": 2,
            "skipped_chains": 0,
            "kernels": [{
                "id": 1,
                "backend": "rocm",
                "gpu_arch": "gfx1101",
                "op_count": 2,
                "start_line": 3,
                "end_line": 4,
                "executable": true,
                "helper_name": "aero_fused_rocm_kernel_1",
                "fallback_reason": null
            }]
        })
    );
}

#[test]
fn quantization_telemetry_and_notes_deny_unimplemented_capabilities() {
    let input = "define i32 @main() {\nentry:\n  %0 = fadd double %a, %b\n  ret i32 0\n}\n";
    let cases = [
        (
            AcceleratorBackend::Rocm,
            QuantizationMode::Fp8E4M3,
            "rocm",
            "fp8-e4m3",
            Some("gfx1101"),
            "ROCm fp8-e4m3 label selects scalar-double helpers for gfx1101; no FP8 representation, per-channel execution, or device execution is implemented.",
        ),
        (
            AcceleratorBackend::Cuda,
            QuantizationMode::Fp8E5M2,
            "cuda",
            "fp8-e5m2",
            Some("sm_89"),
            "CUDA fp8-e5m2 label selects scalar-double helpers for sm_89; no FP8 representation, per-channel execution, or device execution is implemented.",
        ),
        (
            AcceleratorBackend::Cpu,
            QuantizationMode::Int8,
            "cpu",
            "int8",
            None,
            "INT8 label selects scalar-double clamp-and-dequantize helpers; per-channel execution and device execution are not implemented.",
        ),
        (
            AcceleratorBackend::Cpu,
            QuantizationMode::Fp8E5M2,
            "cpu",
            "fp8-e5m2",
            None,
            "CPU fp8-e5m2 label selects scalar-double helpers; no FP8 representation, per-channel execution, or device execution is implemented.",
        ),
    ];
    for &(backend, mode, expected_backend, expected_mode, gpu_arch, _) in &cases {
        let mut config = QuantizationConfig::new(mode);
        config.backend = backend;
        config.gpu_arch = gpu_arch.map(str::to_string);
        config.per_channel = true;
        let (output, _) = apply_quantization_interface(input, &config);
        assert_eq!(
            without_quant_scope_telemetry(&output),
            expected_quant_runtime_output(mode, expected_backend, expected_mode, gpu_arch, "fadd")
        );
    }
    for op in ["fsub", "fmul", "fdiv"] {
        let op_input =
            format!("define i32 @main() {{\nentry:\n  %0 = {op} double %a, %b\n  ret i32 0\n}}\n");
        let mut config = QuantizationConfig::new(QuantizationMode::Int8);
        config.per_channel = true;
        let (output, _) = apply_quantization_interface(&op_input, &config);
        assert_eq!(
            without_quant_scope_telemetry(&output),
            expected_quant_runtime_output(QuantizationMode::Int8, "cpu", "int8", None, op)
        );
        assert_exactly_one_line(
            &output,
            "; aero.quantization.execution_scope=scalar-double-helper",
        );
        assert_exactly_one_line(&output, "; aero.quantization.device_execution=false");
    }
    let mut body_only_annotation = QuantizationConfig::new(QuantizationMode::Int8);
    body_only_annotation.enable_runtime_lowering = false;
    let (body_only_annotation_output, _) =
        apply_quantization_interface(input, &body_only_annotation);
    assert_eq!(
        without_quant_scope_telemetry(&body_only_annotation_output),
        "; aero.quantization.mode=int8\n; aero.quantization.backend=cpu\n; aero.quantization.per_channel=false\n; aero.quantization.runtime_lowering=false\n; aero.quantization.calibration.scale=127.00000000\n; aero.quantization.calibration.samples=4\ndefine i32 @main() {\nentry:\n  ; aero.quantize candidate=1 mode=int8 backend=cpu\n  %0 = fadd double %a, %b\n  ret i32 0\n}\n"
    );
    let no_candidate_input = "define i32 @main() {\nentry:\n  ret i32 0\n}\n";
    let body_only_no_candidate = QuantizationConfig::new(QuantizationMode::Int8);
    let (body_only_no_candidate_output, _) =
        apply_quantization_interface(no_candidate_input, &body_only_no_candidate);
    assert_eq!(
        without_quant_scope_telemetry(&body_only_no_candidate_output),
        "; aero.quantization.mode=int8\n; aero.quantization.backend=cpu\n; aero.quantization.per_channel=false\n; aero.quantization.runtime_lowering=true\n; aero.quantization.calibration.scale=127.00000000\n; aero.quantization.calibration.samples=4\ndefine i32 @main() {\nentry:\n  ret i32 0\n}\n"
    );

    for (backend, mode, expected_backend, expected_mode, gpu_arch, expected_note) in cases {
        let mut config = QuantizationConfig::new(mode);
        config.backend = backend;
        config.gpu_arch = gpu_arch.map(str::to_string);
        config.per_channel = true;
        let (output, report) = apply_quantization_interface(input, &config);
        assert_eq!(
            without_quant_scope_telemetry(&output),
            expected_quant_runtime_output(mode, expected_backend, expected_mode, gpu_arch, "fadd")
        );
        assert_exactly_one_line(
            &output,
            "; aero.quantization.execution_scope=scalar-double-helper",
        );
        assert_exactly_one_line(&output, "; aero.quantization.device_execution=false");
        assert_eq!(report.mode, expected_mode);
        assert_eq!(report.backend, expected_backend);
        assert_eq!(report.gpu_arch.as_deref(), gpu_arch);
        assert_eq!(report.candidate_ops, 1);
        assert_eq!(report.lowered_ops, 1);
        assert_eq!(report.helper_count, 1);
        assert!(report.per_channel);
        assert_eq!(report.calibration_samples, 4);
        assert_eq!(
            report.notes,
            vec![
                expected_note,
                "Scalar-double helper rewriting was applied; numerical correctness is not established.",
            ]
        );
        assert_eq!(
            serde_json::to_value(&report).expect("serialize quantization report"),
            json!({
                "mode": expected_mode,
                "backend": expected_backend,
                "gpu_arch": gpu_arch,
                "candidate_ops": 1,
                "lowered_ops": 1,
                "helper_count": 1,
                "per_channel": true,
                "calibration_samples": 4,
                "notes": [
                    expected_note,
                    "Scalar-double helper rewriting was applied; numerical correctness is not established."
                ]
            })
        );
    }

    let mut annotation_only = QuantizationConfig::new(QuantizationMode::Int8);
    annotation_only.enable_runtime_lowering = false;
    let (annotation_output, annotation_report) =
        apply_quantization_interface(input, &annotation_only);
    assert_eq!(
        without_quant_scope_telemetry(&annotation_output),
        "; aero.quantization.mode=int8\n; aero.quantization.backend=cpu\n; aero.quantization.per_channel=false\n; aero.quantization.runtime_lowering=false\n; aero.quantization.calibration.scale=127.00000000\n; aero.quantization.calibration.samples=4\ndefine i32 @main() {\nentry:\n  ; aero.quantize candidate=1 mode=int8 backend=cpu\n  %0 = fadd double %a, %b\n  ret i32 0\n}\n"
    );
    assert_exactly_one_line(
        &annotation_output,
        "; aero.quantization.execution_scope=scalar-double-helper",
    );
    assert_exactly_one_line(
        &annotation_output,
        "; aero.quantization.device_execution=false",
    );
    assert_eq!(annotation_report.mode, "int8");
    assert_eq!(annotation_report.backend, "cpu");
    assert_eq!(annotation_report.gpu_arch, None);
    assert_eq!(annotation_report.candidate_ops, 1);
    assert_eq!(annotation_report.lowered_ops, 0);
    assert_eq!(annotation_report.helper_count, 0);
    assert!(!annotation_report.per_channel);
    assert_eq!(annotation_report.calibration_samples, 4);
    assert_eq!(
        annotation_report.notes,
        vec![
            "INT8 label selects scalar-double clamp-and-dequantize helpers; per-channel execution and device execution are not implemented.",
            "Quantization annotations were emitted; no helper rewriting or device execution was performed.",
        ]
    );
    assert_eq!(
        serde_json::to_value(&annotation_report).expect("serialize annotation report"),
        json!({
            "mode": "int8",
            "backend": "cpu",
            "gpu_arch": null,
            "candidate_ops": 1,
            "lowered_ops": 0,
            "helper_count": 0,
            "per_channel": false,
            "calibration_samples": 4,
            "notes": [
                "INT8 label selects scalar-double clamp-and-dequantize helpers; per-channel execution and device execution are not implemented.",
                "Quantization annotations were emitted; no helper rewriting or device execution was performed."
            ]
        })
    );

    let no_candidate = QuantizationConfig::new(QuantizationMode::Int8);
    let (no_candidate_output, no_candidate_report) =
        apply_quantization_interface(no_candidate_input, &no_candidate);
    assert_eq!(
        without_quant_scope_telemetry(&no_candidate_output),
        "; aero.quantization.mode=int8\n; aero.quantization.backend=cpu\n; aero.quantization.per_channel=false\n; aero.quantization.runtime_lowering=true\n; aero.quantization.calibration.scale=127.00000000\n; aero.quantization.calibration.samples=4\ndefine i32 @main() {\nentry:\n  ret i32 0\n}\n"
    );
    assert_exactly_one_line(
        &no_candidate_output,
        "; aero.quantization.execution_scope=scalar-double-helper",
    );
    assert_exactly_one_line(
        &no_candidate_output,
        "; aero.quantization.device_execution=false",
    );
    assert_eq!(no_candidate_report.mode, "int8");
    assert_eq!(no_candidate_report.backend, "cpu");
    assert_eq!(no_candidate_report.gpu_arch, None);
    assert_eq!(no_candidate_report.candidate_ops, 0);
    assert_eq!(no_candidate_report.lowered_ops, 0);
    assert_eq!(no_candidate_report.helper_count, 0);
    assert!(!no_candidate_report.per_channel);
    assert_eq!(no_candidate_report.calibration_samples, 4);
    assert_eq!(
        no_candidate_report.notes,
        vec![
            "INT8 label selects scalar-double clamp-and-dequantize helpers; per-channel execution and device execution are not implemented.",
            "Scalar-double helper rewriting found no eligible operation; numerical correctness is not established.",
            "No eligible floating-point arithmetic ops were found.",
        ]
    );
    assert_eq!(
        serde_json::to_value(&no_candidate_report).expect("serialize no-candidate report"),
        json!({
            "mode": "int8",
            "backend": "cpu",
            "gpu_arch": null,
            "candidate_ops": 0,
            "lowered_ops": 0,
            "helper_count": 0,
            "per_channel": false,
            "calibration_samples": 4,
            "notes": [
                "INT8 label selects scalar-double clamp-and-dequantize helpers; per-channel execution and device execution are not implemented.",
                "Scalar-double helper rewriting found no eligible operation; numerical correctness is not established.",
                "No eligible floating-point arithmetic ops were found."
            ]
        })
    );
}

#[test]
fn gguf_example_disables_the_nonexistent_aero_execution_route() {
    let config: Value = serde_json::from_str(&repository_file(
        "benchmarks/gguf/config.rx7800xt.example.json",
    ))
    .expect("parse GGUF example config");
    assert_eq!(
        config["variables"],
        json!({
            "aero_bin": "src/compiler/target/release/aero.exe",
            "llama_cli": "third_party/llama.cpp/build/bin/llama-cli.exe",
            "python_bin": "python",
            "pytorch_runner": "benchmarks/gguf/pytorch_reference_runner.py"
        })
    );
    let aero = config["backends"]
        .as_array()
        .expect("backends array")
        .iter()
        .find(|backend| backend["name"] == "aero_rocm")
        .expect("Aero ROCm entry");
    assert_eq!(aero["enabled"], false);
    assert_eq!(
        aero["disabled_reason"],
        "Aero has no GGUF inference program or ROCm HIP link/device-launch path; this template must not produce Aero performance data."
    );
    assert_eq!(
        aero,
        &json!({
            "name": "aero_rocm",
            "enabled": false,
            "disabled_reason": "Aero has no GGUF inference program or ROCm HIP link/device-launch path; this template must not produce Aero performance data.",
            "cwd": ".",
            "command": "\"{aero_bin}\" run --target rocm --gpu gfx1101 examples/gguf_inference.aero -- --model \"{model_path}\" --prompt \"{prompt}\" --max-tokens {max_tokens}",
            "metric_regexes": [
                r"tokens/sec\s*[:=]\s*([0-9]+(?:\.[0-9]+)?)",
                r"([0-9]+(?:\.[0-9]+)?)\s*tokens/s"
            ],
            "aux_metrics": {
                "prompt_eval_ms": [
                    r"prompt\s*eval\s*time\s*[:=].*?([0-9]+(?:\.[0-9]+)?)\s*ms",
                    r"prompt_eval_ms\s*[:=]\s*([0-9]+(?:\.[0-9]+)?)"
                ],
                "peak_vram_gb": [
                    r"peak\s*vram\s*[:=].*?([0-9]+(?:\.[0-9]+)?)\s*gb",
                    r"peak_vram_gb\s*[:=]\s*([0-9]+(?:\.[0-9]+)?)"
                ]
            }
        })
    );
    let llama = config["backends"]
        .as_array()
        .expect("backends array")
        .iter()
        .find(|backend| backend["name"] == "llama_cpp_rocm")
        .expect("llama.cpp ROCm entry");
    assert_eq!(llama["enabled"], true);
    assert_eq!(llama["cwd"], ".");
    assert_eq!(
        llama["command"],
        "\"{llama_cli}\" -m \"{model_path}\" -p \"{prompt}\" -n {max_tokens} -ngl 999 --no-warmup"
    );
    assert_eq!(
        llama,
        &json!({
            "name": "llama_cpp_rocm",
            "enabled": true,
            "cwd": ".",
            "command": "\"{llama_cli}\" -m \"{model_path}\" -p \"{prompt}\" -n {max_tokens} -ngl 999 --no-warmup",
            "metric_regexes": [
                r"([0-9]+(?:\.[0-9]+)?)\s*tokens per second",
                r"([0-9]+(?:\.[0-9]+)?)\s*t/s"
            ],
            "aux_metrics": {
                "prompt_eval_ms": [
                    r"prompt\s*eval\s*time\s*=\s*([0-9]+(?:\.[0-9]+)?)\s*ms"
                ]
            }
        })
    );
    let pytorch = config["backends"]
        .as_array()
        .expect("backends array")
        .iter()
        .find(|backend| backend["name"] == "pytorch_reference")
        .expect("PyTorch reference entry");
    assert_eq!(pytorch["enabled"], true);
    assert_eq!(pytorch["cwd"], ".");
    assert_eq!(
        pytorch["command"],
        "\"{python_bin}\" \"{pytorch_runner}\" --model \"{model_path}\" --prompt \"{prompt}\" --max-tokens {max_tokens}"
    );
    assert_eq!(
        pytorch,
        &json!({
            "name": "pytorch_reference",
            "enabled": true,
            "cwd": ".",
            "command": "\"{python_bin}\" \"{pytorch_runner}\" --model \"{model_path}\" --prompt \"{prompt}\" --max-tokens {max_tokens}",
            "metric_regexes": [
                r"tokens/sec\s*[:=]\s*([0-9]+(?:\.[0-9]+)?)",
                r"([0-9]+(?:\.[0-9]+)?)\s*tokens/s"
            ],
            "aux_metrics": {
                "prompt_eval_ms": [
                    r"prompt_eval_ms\s*[:=]\s*([0-9]+(?:\.[0-9]+)?)"
                ],
                "peak_vram_gb": [
                    r"peak_vram_gb\s*[:=]\s*([0-9]+(?:\.[0-9]+)?)"
                ]
            }
        })
    );
    let readme = repository_file("benchmarks/gguf/README.md");
    assert!(readme.contains(
        "The Aero ROCm entry is disabled and must not be treated as a runnable benchmark."
    ));
    assert!(readme.contains("llama.cpp and PyTorch are external reference backends"));
}
