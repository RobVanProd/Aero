use crate::accelerator::AcceleratorBackend;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantizationMode {
    Int8,
    Fp8E4M3,
    Fp8E5M2,
}

impl QuantizationMode {
    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_ascii_lowercase().as_str() {
            "int8" => Some(Self::Int8),
            "fp8-e4m3" | "fp8e4m3" => Some(Self::Fp8E4M3),
            "fp8-e5m2" | "fp8e5m2" => Some(Self::Fp8E5M2),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Int8 => "int8",
            Self::Fp8E4M3 => "fp8-e4m3",
            Self::Fp8E5M2 => "fp8-e5m2",
        }
    }
}

#[derive(Debug, Clone)]
pub struct QuantizationConfig {
    pub mode: QuantizationMode,
    pub backend: AcceleratorBackend,
    pub per_channel: bool,
    pub calibration_profile: Option<CalibrationProfile>,
    pub calibration_source: Option<String>,
    pub enable_runtime_lowering: bool,
}

impl QuantizationConfig {
    pub fn new(mode: QuantizationMode) -> Self {
        Self {
            mode,
            backend: AcceleratorBackend::Cpu,
            per_channel: false,
            calibration_profile: None,
            calibration_source: None,
            enable_runtime_lowering: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationProfile {
    pub backend: String,
    pub mode: String,
    pub sample_count: usize,
    pub min_value: f64,
    pub max_value: f64,
    pub abs_max: f64,
    pub scale: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuantizationReport {
    pub mode: String,
    pub backend: String,
    pub candidate_ops: usize,
    pub lowered_ops: usize,
    pub helper_count: usize,
    pub per_channel: bool,
    pub calibration_samples: usize,
    pub notes: Vec<String>,
}

pub fn load_calibration_profile(
    calibration_file: &Path,
    mode: QuantizationMode,
    backend: AcceleratorBackend,
) -> Result<CalibrationProfile, String> {
    let text = fs::read_to_string(calibration_file).map_err(|err| {
        format!(
            "failed to read calibration file {}: {}",
            calibration_file.display(),
            err
        )
    })?;
    let samples = parse_calibration_samples(&text)?;
    if samples.is_empty() {
        return Err("calibration file contains no samples".to_string());
    }
    Ok(calibrate_from_samples(&samples, mode, backend))
}

pub fn calibrate_from_samples(
    samples: &[f64],
    mode: QuantizationMode,
    backend: AcceleratorBackend,
) -> CalibrationProfile {
    let min_value = samples.iter().copied().fold(f64::INFINITY, f64::min);
    let max_value = samples.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let abs_max = samples
        .iter()
        .copied()
        .map(f64::abs)
        .fold(0.0_f64, f64::max);

    let level = match mode {
        QuantizationMode::Int8 => 127.0,
        QuantizationMode::Fp8E4M3 => 448.0,
        QuantizationMode::Fp8E5M2 => 57344.0,
    };
    let scale = if abs_max <= f64::EPSILON {
        1.0
    } else {
        level / abs_max
    };

    CalibrationProfile {
        backend: backend.as_str().to_string(),
        mode: mode.as_str().to_string(),
        sample_count: samples.len(),
        min_value,
        max_value,
        abs_max,
        scale,
    }
}

pub fn apply_quantization_interface(
    llvm_ir: &str,
    config: &QuantizationConfig,
) -> (String, QuantizationReport) {
    let mut out = String::new();
    out.push_str(&format!(
        "; aero.quantization.mode={}\n",
        config.mode.as_str()
    ));
    out.push_str(&format!(
        "; aero.quantization.backend={}\n",
        config.backend.as_str()
    ));
    out.push_str(&format!(
        "; aero.quantization.per_channel={}\n",
        config.per_channel
    ));
    out.push_str(&format!(
        "; aero.quantization.runtime_lowering={}\n",
        config.enable_runtime_lowering
    ));

    let calibration_profile = config.calibration_profile.clone().unwrap_or_else(|| {
        calibrate_from_samples(&[0.25, 0.5, 1.0, -1.0], config.mode, config.backend)
    });
    out.push_str(&format!(
        "; aero.quantization.calibration.scale={:.8}\n",
        calibration_profile.scale
    ));
    out.push_str(&format!(
        "; aero.quantization.calibration.samples={}\n",
        calibration_profile.sample_count
    ));
    if let Some(source) = &config.calibration_source {
        out.push_str(&format!(
            "; aero.quantization.calibration.source={}\n",
            source
        ));
    }

    let mut helper_names = Vec::new();
    let mut helper_set = HashSet::new();
    let mut candidate_ops = 0usize;
    let mut lowered_ops = 0usize;

    for line in llvm_ir.lines() {
        if let Some(inst) = parse_float_binop_line(line) {
            candidate_ops += 1;
            if config.enable_runtime_lowering {
                let helper = helper_name(config.backend, config.mode, &inst.op);
                if helper_set.insert(helper.clone()) {
                    helper_names.push(helper.clone());
                }
                lowered_ops += 1;
                out.push_str(&format!(
                    "{}; aero.quantize candidate={} helper={} scale={:.8}\n",
                    inst.indent, candidate_ops, helper, calibration_profile.scale
                ));
                out.push_str(&format!(
                    "{}{} = call double @{}(double {}, double {})\n",
                    inst.indent, inst.result, helper, inst.lhs, inst.rhs
                ));
            } else {
                out.push_str(&format!(
                    "{}; aero.quantize candidate={} mode={} backend={}\n",
                    inst.indent,
                    candidate_ops,
                    config.mode.as_str(),
                    config.backend.as_str()
                ));
                out.push_str(line);
                out.push('\n');
            }
            continue;
        }

        out.push_str(line);
        out.push('\n');
    }

    if config.enable_runtime_lowering && !helper_names.is_empty() {
        out.push('\n');
        out.push_str(&emit_runtime_support(config.mode));
        out.push('\n');
        for helper in &helper_names {
            out.push_str(&emit_runtime_helper(
                helper,
                config.mode,
                calibration_profile.scale,
            ));
            out.push('\n');
        }
    }

    let mut notes = Vec::new();
    notes.push(backend_mode_note(config.backend, config.mode).to_string());
    if config.enable_runtime_lowering {
        notes.push("Executable quantization runtime helper lowering was applied.".to_string());
    } else {
        notes.push("Quantization annotations emitted without executable lowering.".to_string());
    }
    if candidate_ops == 0 {
        notes.push("No eligible floating-point arithmetic ops were found.".to_string());
    }

    (
        out,
        QuantizationReport {
            mode: config.mode.as_str().to_string(),
            backend: config.backend.as_str().to_string(),
            candidate_ops,
            lowered_ops,
            helper_count: helper_names.len(),
            per_channel: config.per_channel,
            calibration_samples: calibration_profile.sample_count,
            notes,
        },
    )
}

fn backend_mode_note(backend: AcceleratorBackend, mode: QuantizationMode) -> &'static str {
    match (backend, mode) {
        (AcceleratorBackend::Rocm, QuantizationMode::Fp8E4M3)
        | (AcceleratorBackend::Rocm, QuantizationMode::Fp8E5M2) => {
            "ROCm FP8 path enabled (hardware-assist friendly runtime helpers)."
        }
        (AcceleratorBackend::Cuda, QuantizationMode::Fp8E4M3)
        | (AcceleratorBackend::Cuda, QuantizationMode::Fp8E5M2) => {
            "CUDA FP8 path enabled (hardware-assist friendly runtime helpers)."
        }
        (_, QuantizationMode::Int8) => {
            "INT8 path enabled with calibrated clamp-and-dequantize runtime helpers."
        }
        _ => "FP8 is emulated for this backend via calibrated runtime helpers.",
    }
}

fn parse_calibration_samples(input: &str) -> Result<Vec<f64>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    if trimmed.starts_with('[') {
        let parsed: Vec<f64> = serde_json::from_str(trimmed)
            .map_err(|err| format!("failed to parse calibration JSON array: {}", err))?;
        return Ok(parsed);
    }

    let mut values = Vec::new();
    for (idx, raw) in trimmed.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let value = line
            .parse::<f64>()
            .map_err(|err| format!("invalid calibration value on line {}: {}", idx + 1, err))?;
        values.push(value);
    }
    Ok(values)
}

#[derive(Debug, Clone)]
struct FloatBinOpInst {
    indent: String,
    result: String,
    op: String,
    lhs: String,
    rhs: String,
}

fn parse_float_binop_line(line: &str) -> Option<FloatBinOpInst> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with(';') || !trimmed.contains(" = f") {
        return None;
    }

    let indent_len = line.len().saturating_sub(line.trim_start().len());
    let indent = line[..indent_len].to_string();

    let (result, rhs) = trimmed.split_once(" = ")?;
    let mut parts = rhs.split_whitespace();
    let op = parts.next()?.to_string();
    if op != "fadd" && op != "fsub" && op != "fmul" && op != "fdiv" {
        return None;
    }
    let ty = parts.next()?;
    if ty != "double" {
        return None;
    }

    let args = parts.collect::<Vec<_>>().join(" ");
    let (lhs, rhs_val) = args.split_once(',')?;
    Some(FloatBinOpInst {
        indent,
        result: result.to_string(),
        op,
        lhs: lhs.trim().to_string(),
        rhs: rhs_val.trim().to_string(),
    })
}

fn helper_name(backend: AcceleratorBackend, mode: QuantizationMode, op: &str) -> String {
    format!(
        "aero_{}_{}_{}",
        backend.as_str(),
        mode.as_str().replace('-', "_"),
        op
    )
}

fn emit_runtime_support(mode: QuantizationMode) -> String {
    let mut support = String::new();
    support.push_str(
        "define internal i32 @aero_quant_clamp_i32(i32 %v, i32 %lo, i32 %hi) {\nentry:\n  %lt = icmp slt i32 %v, %lo\n  %gt = icmp sgt i32 %v, %hi\n  %lowered = select i1 %lt, i32 %lo, i32 %v\n  %clamped = select i1 %gt, i32 %hi, i32 %lowered\n  ret i32 %clamped\n}\n",
    );
    if mode == QuantizationMode::Fp8E4M3 || mode == QuantizationMode::Fp8E5M2 {
        support.push_str(
            "define internal double @aero_quant_clamp_f64(double %v, double %lo, double %hi) {\nentry:\n  %lt = fcmp olt double %v, %lo\n  %gt = fcmp ogt double %v, %hi\n  %lowered = select i1 %lt, double %lo, double %v\n  %clamped = select i1 %gt, double %hi, double %lowered\n  ret double %clamped\n}\n",
        );
    }
    support
}

fn emit_runtime_helper(helper_name: &str, mode: QuantizationMode, scale: f64) -> String {
    let op = if helper_name.ends_with("fadd") {
        "fadd"
    } else if helper_name.ends_with("fsub") {
        "fsub"
    } else if helper_name.ends_with("fmul") {
        "fmul"
    } else {
        "fdiv"
    };
    let inv_scale = if scale.abs() <= f64::EPSILON {
        1.0
    } else {
        1.0 / scale
    };

    match mode {
        QuantizationMode::Int8 => format!(
            "; aero.quantization.runtime helper for mode=int8
define internal double @{helper_name}(double %a, double %b) {{
entry:
  %qa = fmul double %a, {scale:.12}
  %qb = fmul double %b, {scale:.12}
  %ia = fptosi double %qa to i32
  %ib = fptosi double %qb to i32
  %ca = call i32 @aero_quant_clamp_i32(i32 %ia, i32 -127, i32 127)
  %cb = call i32 @aero_quant_clamp_i32(i32 %ib, i32 -127, i32 127)
  %fca = sitofp i32 %ca to double
  %fcb = sitofp i32 %cb to double
  %qres = {op} double %fca, %fcb
  %ires = fptosi double %qres to i32
  %cres = call i32 @aero_quant_clamp_i32(i32 %ires, i32 -127, i32 127)
  %fq = sitofp i32 %cres to double
  %dres = fmul double %fq, {inv_scale:.12}
  ret double %dres
}}"
        ),
        QuantizationMode::Fp8E4M3 | QuantizationMode::Fp8E5M2 => {
            let max_mag = if mode == QuantizationMode::Fp8E4M3 {
                448.0
            } else {
                57344.0
            };
            let min_mag = -max_mag;
            format!(
                "; aero.quantization.runtime helper for mode={}
define internal double @{helper_name}(double %a, double %b) {{
entry:
  %qa = fmul double %a, {scale:.12}
  %qb = fmul double %b, {scale:.12}
  %ca = call double @aero_quant_clamp_f64(double %qa, double {min_mag:.12}, double {max_mag:.12})
  %cb = call double @aero_quant_clamp_f64(double %qb, double {min_mag:.12}, double {max_mag:.12})
  %qres = {op} double %ca, %cb
  %cres = call double @aero_quant_clamp_f64(double %qres, double {min_mag:.12}, double {max_mag:.12})
  %dres = fmul double %cres, {inv_scale:.12}
  ret double %dres
}}",
                mode.as_str()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantization_mode_parse_accepts_supported_variants() {
        assert_eq!(
            QuantizationMode::parse("int8"),
            Some(QuantizationMode::Int8)
        );
        assert_eq!(
            QuantizationMode::parse("fp8-e4m3"),
            Some(QuantizationMode::Fp8E4M3)
        );
        assert_eq!(
            QuantizationMode::parse("fp8e5m2"),
            Some(QuantizationMode::Fp8E5M2)
        );
        assert_eq!(QuantizationMode::parse("unknown"), None);
    }

    #[test]
    fn calibration_from_samples_is_stable() {
        let profile = calibrate_from_samples(
            &[0.5, -1.0, 2.0],
            QuantizationMode::Int8,
            AcceleratorBackend::Rocm,
        );
        assert_eq!(profile.backend, "rocm");
        assert_eq!(profile.mode, "int8");
        assert_eq!(profile.sample_count, 3);
        assert!(profile.scale > 0.0);
    }

    #[test]
    fn calibration_loader_supports_json_arrays() {
        let path = std::env::temp_dir().join("aero_quant_calibration.json");
        fs::write(&path, "[0.25, 0.5, -1.0, 2.0]").expect("should write calibration file");
        let profile =
            load_calibration_profile(&path, QuantizationMode::Fp8E4M3, AcceleratorBackend::Cuda)
                .expect("calibration should load");
        assert_eq!(profile.sample_count, 4);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn quantization_runtime_lowering_rewrites_float_ops_to_calls() {
        let llvm = "define i32 @main() {\nentry:\n  %0 = fadd double %a, %b\n  %1 = fmul double %0, %c\n  ret i32 0\n}\n";
        let mut config = QuantizationConfig::new(QuantizationMode::Int8);
        config.backend = AcceleratorBackend::Rocm;
        config.calibration_profile = Some(calibrate_from_samples(
            &[1.0, -1.0, 0.5],
            config.mode,
            config.backend,
        ));
        let (out, report) = apply_quantization_interface(llvm, &config);
        assert!(out.contains("; aero.quantization.backend=rocm"));
        assert!(out.contains("call double @aero_rocm_int8_fadd"));
        assert!(out.contains("call double @aero_rocm_int8_fmul"));
        assert!(out.contains("define internal i32 @aero_quant_clamp_i32"));
        assert_eq!(report.candidate_ops, 2);
        assert_eq!(report.lowered_ops, 2);
        assert_eq!(report.helper_count, 2);
    }

    #[test]
    fn quantization_can_run_annotation_only_mode() {
        let llvm = "define i32 @main() {\nentry:\n  %0 = fadd double %a, %b\n  ret i32 0\n}\n";
        let mut config = QuantizationConfig::new(QuantizationMode::Fp8E5M2);
        config.enable_runtime_lowering = false;
        let (out, report) = apply_quantization_interface(llvm, &config);
        assert!(out.contains("aero.quantize candidate=1 mode=fp8-e5m2"));
        assert_eq!(report.lowered_ops, 0);
    }
}
