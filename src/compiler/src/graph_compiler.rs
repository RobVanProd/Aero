use crate::accelerator::AcceleratorBackend;
use crate::gpu::default_gpu_arch;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct GraphCompilationConfig {
    pub backend: AcceleratorBackend,
    pub gpu_arch: Option<String>,
    pub executable_fusion: bool,
}

impl Default for GraphCompilationConfig {
    fn default() -> Self {
        Self {
            backend: AcceleratorBackend::Cpu,
            gpu_arch: None,
            executable_fusion: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FusedKernel {
    pub id: usize,
    pub backend: String,
    pub gpu_arch: Option<String>,
    pub op_count: usize,
    pub start_line: usize,
    pub end_line: usize,
    pub executable: bool,
    pub helper_name: Option<String>,
    pub fallback_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphCompilationReport {
    pub backend: String,
    pub gpu_arch: Option<String>,
    pub fused_kernel_count: usize,
    pub executable_kernel_count: usize,
    pub total_fused_ops: usize,
    pub skipped_chains: usize,
    pub kernels: Vec<FusedKernel>,
}

#[derive(Debug, Clone)]
struct FloatBinOpInst {
    indent: String,
    result: String,
    op: String,
    lhs: String,
    rhs: String,
}

#[derive(Debug, Clone)]
struct ChainInst {
    line_no: usize,
    source: String,
    op: FloatBinOpInst,
}

pub fn apply_advanced_graph_compilation(llvm_ir: &str) -> (String, GraphCompilationReport) {
    apply_advanced_graph_compilation_with_config(llvm_ir, &GraphCompilationConfig::default())
}

pub fn apply_advanced_graph_compilation_with_config(
    llvm_ir: &str,
    config: &GraphCompilationConfig,
) -> (String, GraphCompilationReport) {
    let gpu_arch = resolved_gpu_arch(config);

    let mut out = String::new();
    out.push_str("; aero.graph_compilation=enabled\n");
    out.push_str(&format!(
        "; aero.graph_compilation.backend={}\n",
        config.backend.as_str()
    ));
    if let Some(gpu_arch) = gpu_arch {
        out.push_str(&format!("; aero.graph_compilation.gpu_arch={}\n", gpu_arch));
    }
    out.push_str(&format!(
        "; aero.graph_compilation.executable_fusion={}\n",
        config.executable_fusion
    ));

    let lines = llvm_ir.lines().map(|l| l.to_string()).collect::<Vec<_>>();
    let mut kernels = Vec::new();
    let mut helper_defs = Vec::new();
    let mut pending_chain: Vec<ChainInst> = Vec::new();
    let mut next_kernel_id = 1usize;
    let mut skipped_chains = 0usize;

    for (idx, line) in lines.iter().enumerate() {
        let line_no = idx + 1;
        if let Some(op) = parse_float_binop_line(line) {
            pending_chain.push(ChainInst {
                line_no,
                source: line.clone(),
                op,
            });
            continue;
        }

        flush_chain(
            &mut out,
            &mut pending_chain,
            &lines[idx..],
            config,
            &mut kernels,
            &mut helper_defs,
            &mut next_kernel_id,
            &mut skipped_chains,
        );
        out.push_str(line);
        out.push('\n');
    }

    flush_chain(
        &mut out,
        &mut pending_chain,
        &[],
        config,
        &mut kernels,
        &mut helper_defs,
        &mut next_kernel_id,
        &mut skipped_chains,
    );

    if !helper_defs.is_empty() {
        out.push('\n');
        for helper in helper_defs {
            out.push_str(&helper);
            out.push('\n');
        }
    }

    let total_fused_ops = kernels.iter().map(|k| k.op_count).sum::<usize>();
    let executable_kernel_count = kernels.iter().filter(|k| k.executable).count();
    (
        out,
        GraphCompilationReport {
            backend: config.backend.as_str().to_string(),
            gpu_arch: gpu_arch.map(str::to_string),
            fused_kernel_count: kernels.len(),
            executable_kernel_count,
            total_fused_ops,
            skipped_chains,
            kernels,
        },
    )
}

fn flush_chain(
    out: &mut String,
    chain: &mut Vec<ChainInst>,
    future_lines: &[String],
    config: &GraphCompilationConfig,
    kernels: &mut Vec<FusedKernel>,
    helper_defs: &mut Vec<String>,
    next_kernel_id: &mut usize,
    skipped_chains: &mut usize,
) {
    if chain.is_empty() {
        return;
    }

    if chain.len() < 2 {
        for inst in chain.drain(..) {
            out.push_str(&inst.source);
            out.push('\n');
        }
        return;
    }

    let id = *next_kernel_id;
    *next_kernel_id += 1;
    let start_line = chain[0].line_no;
    let end_line = chain[chain.len() - 1].line_no;
    let op_count = chain.len();

    let intermediate_results = chain[..chain.len() - 1]
        .iter()
        .map(|inst| inst.op.result.clone())
        .collect::<Vec<_>>();

    let escaped = intermediate_results
        .iter()
        .find(|name| token_used_in_lines(future_lines, name));

    if !config.executable_fusion || escaped.is_some() {
        let reason = if !config.executable_fusion {
            "executable fusion disabled".to_string()
        } else {
            format!(
                "intermediate value {} escapes the chain",
                escaped.cloned().unwrap_or_else(|| "%tmp".to_string())
            )
        };

        out.push_str(&format!(
            "  ; aero.fused_kernel id={} backend={} ops={} range={}..{} executable=false reason=\"{}\"\n",
            id,
            config.backend.as_str(),
            op_count,
            start_line,
            end_line,
            reason
        ));
        for inst in chain.drain(..) {
            out.push_str(&inst.source);
            out.push('\n');
        }
        *skipped_chains += 1;
        kernels.push(FusedKernel {
            id,
            backend: config.backend.as_str().to_string(),
            gpu_arch: resolved_gpu_arch(config).map(str::to_string),
            op_count,
            start_line,
            end_line,
            executable: false,
            helper_name: None,
            fallback_reason: Some(reason),
        });
        return;
    }

    let helper_name = format!("aero_fused_{}_kernel_{}", config.backend.as_str(), id);
    let lowered = lower_chain_to_helper(chain.as_slice(), &helper_name);
    if let Some((call_line, helper_def)) = lowered {
        out.push_str(&format!(
            "  ; aero.fused_kernel id={} backend={} ops={} range={}..{} executable=true helper=@{}\n",
            id,
            config.backend.as_str(),
            op_count,
            start_line,
            end_line,
            helper_name
        ));
        out.push_str(&call_line);
        out.push('\n');
        helper_defs.push(helper_def);
        kernels.push(FusedKernel {
            id,
            backend: config.backend.as_str().to_string(),
            gpu_arch: resolved_gpu_arch(config).map(str::to_string),
            op_count,
            start_line,
            end_line,
            executable: true,
            helper_name: Some(helper_name),
            fallback_reason: None,
        });
        chain.clear();
        return;
    }

    let reason = "unable to lower chain to executable kernel".to_string();
    out.push_str(&format!(
        "  ; aero.fused_kernel id={} backend={} ops={} range={}..{} executable=false reason=\"{}\"\n",
            id,
            config.backend.as_str(),
            op_count,
            start_line,
            end_line,
            reason
    ));
    for inst in chain.drain(..) {
        out.push_str(&inst.source);
        out.push('\n');
    }
    *skipped_chains += 1;
    kernels.push(FusedKernel {
        id,
        backend: config.backend.as_str().to_string(),
        gpu_arch: resolved_gpu_arch(config).map(str::to_string),
        op_count,
        start_line,
        end_line,
        executable: false,
        helper_name: None,
        fallback_reason: Some(reason),
    });
}

fn lower_chain_to_helper(chain: &[ChainInst], helper_name: &str) -> Option<(String, String)> {
    let produced = chain
        .iter()
        .map(|inst| inst.op.result.clone())
        .collect::<HashSet<_>>();

    let mut external_inputs = Vec::new();
    let mut seen_inputs = HashSet::new();
    for inst in chain {
        for operand in [&inst.op.lhs, &inst.op.rhs] {
            if produced.contains(operand) || !looks_like_value_token(operand) {
                continue;
            }
            if seen_inputs.insert(operand.clone()) {
                external_inputs.push(operand.clone());
            }
        }
    }

    let mut input_to_param = HashMap::new();
    for (idx, token) in external_inputs.iter().enumerate() {
        input_to_param.insert(token.clone(), format!("%arg{}", idx));
    }

    let mut result_to_temp = HashMap::new();
    for (idx, inst) in chain.iter().enumerate() {
        result_to_temp.insert(inst.op.result.clone(), format!("%r{}", idx));
    }

    let indent = chain
        .first()
        .map(|c| c.op.indent.clone())
        .unwrap_or_default();
    let call_args = external_inputs
        .iter()
        .map(|token| format!("double {}", token))
        .collect::<Vec<_>>()
        .join(", ");
    let last_result = chain.last()?.op.result.clone();
    let call_line = if call_args.is_empty() {
        format!("{}{} = call double @{}()", indent, last_result, helper_name)
    } else {
        format!(
            "{}{} = call double @{}({})",
            indent, last_result, helper_name, call_args
        )
    };

    let helper_params = external_inputs
        .iter()
        .enumerate()
        .map(|(idx, _)| format!("double %arg{}", idx))
        .collect::<Vec<_>>()
        .join(", ");
    let mut helper = if helper_params.is_empty() {
        format!("define internal double @{}() {{\nentry:\n", helper_name)
    } else {
        format!(
            "define internal double @{}({}) {{\nentry:\n",
            helper_name, helper_params
        )
    };

    for (idx, inst) in chain.iter().enumerate() {
        let lhs = remap_operand(&inst.op.lhs, &input_to_param, &result_to_temp);
        let rhs = remap_operand(&inst.op.rhs, &input_to_param, &result_to_temp);
        helper.push_str(&format!(
            "  %r{} = {} double {}, {}\n",
            idx, inst.op.op, lhs, rhs
        ));
    }

    helper.push_str(&format!("  ret double %r{}\n", chain.len() - 1));
    helper.push_str("}\n");
    Some((call_line, helper))
}

fn remap_operand(
    operand: &str,
    input_to_param: &HashMap<String, String>,
    result_to_temp: &HashMap<String, String>,
) -> String {
    if let Some(mapped) = result_to_temp.get(operand) {
        return mapped.clone();
    }
    if let Some(mapped) = input_to_param.get(operand) {
        return mapped.clone();
    }
    operand.to_string()
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

fn token_used_in_lines(lines: &[String], token: &str) -> bool {
    lines.iter().any(|line| token_used_in_line(line, token))
}

fn token_used_in_line(line: &str, token: &str) -> bool {
    line.split(|c: char| {
        c.is_whitespace()
            || c == ','
            || c == '('
            || c == ')'
            || c == '['
            || c == ']'
            || c == '{'
            || c == '}'
    })
    .any(|part| part == token)
}

fn looks_like_value_token(token: &str) -> bool {
    token.starts_with('%') || token.starts_with('@')
}

fn resolved_gpu_arch(config: &GraphCompilationConfig) -> Option<&str> {
    match config.backend {
        AcceleratorBackend::Cpu => None,
        AcceleratorBackend::Rocm => config
            .gpu_arch
            .as_deref()
            .or(default_gpu_arch(AcceleratorBackend::Rocm)),
        AcceleratorBackend::Cuda => config
            .gpu_arch
            .as_deref()
            .or(default_gpu_arch(AcceleratorBackend::Cuda)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_compilation_emits_executable_fused_kernel() {
        let input = "define i32 @main() {\nentry:\n  %0 = fadd double %a, %b\n  %1 = fmul double %0, %c\n  ret i32 0\n}\n";
        let mut config = GraphCompilationConfig::default();
        config.backend = AcceleratorBackend::Rocm;

        let (out, report) = apply_advanced_graph_compilation_with_config(input, &config);
        assert!(out.contains("; aero.graph_compilation.backend=rocm"));
        assert!(out.contains("; aero.graph_compilation.gpu_arch=gfx1101"));
        assert!(out.contains("aero.fused_kernel id=1"));
        assert!(out.contains("executable=true"));
        assert!(out.contains("%1 = call double @aero_fused_rocm_kernel_1"));
        assert!(out.contains("define internal double @aero_fused_rocm_kernel_1"));
        assert_eq!(report.gpu_arch.as_deref(), Some("gfx1101"));
        assert_eq!(report.fused_kernel_count, 1);
        assert_eq!(report.executable_kernel_count, 1);
        assert_eq!(report.skipped_chains, 0);
    }

    #[test]
    fn graph_compilation_falls_back_when_intermediate_escapes() {
        let input = "define i32 @main() {\nentry:\n  %0 = fadd double %a, %b\n  %1 = fmul double %0, %c\n  br label %next\nnext:\n  %2 = fadd double %0, %d\n  ret i32 0\n}\n";
        let (out, report) = apply_advanced_graph_compilation(input);
        assert!(
            out.contains("executable=false reason=\"intermediate value %0 escapes the chain\"")
        );
        assert!(out.contains("%0 = fadd double %a, %b"));
        assert_eq!(report.fused_kernel_count, 1);
        assert_eq!(report.executable_kernel_count, 0);
        assert_eq!(report.skipped_chains, 1);
    }

    #[test]
    fn graph_compilation_respects_annotation_only_mode() {
        let input = "define i32 @main() {\nentry:\n  %0 = fadd double %a, %b\n  %1 = fmul double %0, %c\n  ret i32 0\n}\n";
        let config = GraphCompilationConfig {
            backend: AcceleratorBackend::Cpu,
            gpu_arch: None,
            executable_fusion: false,
        };
        let (out, report) = apply_advanced_graph_compilation_with_config(input, &config);
        assert!(out.contains("executable=false reason=\"executable fusion disabled\""));
        assert!(!out.contains("define internal double @aero_fused"));
        assert_eq!(report.executable_kernel_count, 0);
        assert_eq!(report.skipped_chains, 1);
    }
}
