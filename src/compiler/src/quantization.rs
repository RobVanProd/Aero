use serde::Serialize;

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
    pub per_channel: bool,
    pub calibration_samples: usize,
}

impl QuantizationConfig {
    pub fn new(mode: QuantizationMode) -> Self {
        Self {
            mode,
            per_channel: false,
            calibration_samples: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct QuantizationReport {
    pub mode: String,
    pub candidate_ops: usize,
    pub annotated_ops: usize,
    pub per_channel: bool,
    pub calibration_samples: usize,
    pub notes: Vec<String>,
}

pub fn apply_quantization_interface(
    llvm_ir: &str,
    config: &QuantizationConfig,
) -> (String, QuantizationReport) {
    let mut candidate_ops = 0usize;
    let mut annotated_ops = 0usize;

    let mut out = String::new();
    out.push_str(&format!(
        "; aero.quantization.mode={}\n",
        config.mode.as_str()
    ));
    out.push_str(&format!(
        "; aero.quantization.per_channel={}\n",
        config.per_channel
    ));
    out.push_str(&format!(
        "; aero.quantization.calibration_samples={}\n",
        config.calibration_samples
    ));

    for line in llvm_ir.lines() {
        if is_quantization_candidate(line) {
            candidate_ops += 1;
            annotated_ops += 1;
            out.push_str(&format!(
                "  ; aero.quantize candidate={} mode={}\n",
                candidate_ops,
                config.mode.as_str()
            ));
        }
        out.push_str(line);
        out.push('\n');
    }

    let mut notes = Vec::new();
    notes.push(
        "Interface pass only: emits quantization metadata without changing arithmetic semantics."
            .to_string(),
    );
    if candidate_ops == 0 {
        notes.push("No eligible floating-point arithmetic ops were found.".to_string());
    }

    (
        out,
        QuantizationReport {
            mode: config.mode.as_str().to_string(),
            candidate_ops,
            annotated_ops,
            per_channel: config.per_channel,
            calibration_samples: config.calibration_samples,
            notes,
        },
    )
}

fn is_quantization_candidate(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.contains(" fadd ")
        || trimmed.contains(" fsub ")
        || trimmed.contains(" fmul ")
        || trimmed.contains(" fdiv ")
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
    fn quantization_interface_marks_float_arithmetic_candidates() {
        let llvm = "define i32 @main() {\nentry:\n  %0 = fadd double %a, %b\n  %1 = fmul double %0, %c\n  ret i32 0\n}\n";
        let config = QuantizationConfig::new(QuantizationMode::Int8);
        let (out, report) = apply_quantization_interface(llvm, &config);
        assert!(out.contains("; aero.quantization.mode=int8"));
        assert!(out.contains("; aero.quantize candidate=1 mode=int8"));
        assert!(out.contains("; aero.quantize candidate=2 mode=int8"));
        assert_eq!(report.candidate_ops, 2);
        assert_eq!(report.annotated_ops, 2);
    }
}
