use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FusedKernel {
    pub id: usize,
    pub op_count: usize,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphCompilationReport {
    pub fused_kernel_count: usize,
    pub total_fused_ops: usize,
    pub kernels: Vec<FusedKernel>,
}

pub fn apply_advanced_graph_compilation(llvm_ir: &str) -> (String, GraphCompilationReport) {
    let mut out = String::new();
    out.push_str("; aero.graph_compilation=enabled\n");

    let mut kernels = Vec::new();
    let mut pending_chain: Vec<(usize, String)> = Vec::new();
    let mut next_kernel_id = 1usize;

    for (idx, line) in llvm_ir.lines().enumerate() {
        let line_no = idx + 1;
        if is_elementwise_candidate(line) {
            pending_chain.push((line_no, line.to_string()));
            continue;
        }

        flush_chain(
            &mut out,
            &mut pending_chain,
            &mut kernels,
            &mut next_kernel_id,
        );
        out.push_str(line);
        out.push('\n');
    }

    flush_chain(
        &mut out,
        &mut pending_chain,
        &mut kernels,
        &mut next_kernel_id,
    );

    let total_fused_ops = kernels.iter().map(|k| k.op_count).sum::<usize>();
    (
        out,
        GraphCompilationReport {
            fused_kernel_count: kernels.len(),
            total_fused_ops,
            kernels,
        },
    )
}

fn flush_chain(
    out: &mut String,
    chain: &mut Vec<(usize, String)>,
    kernels: &mut Vec<FusedKernel>,
    next_kernel_id: &mut usize,
) {
    if chain.is_empty() {
        return;
    }

    if chain.len() >= 2 {
        let id = *next_kernel_id;
        *next_kernel_id += 1;
        let start_line = chain[0].0;
        let end_line = chain[chain.len() - 1].0;
        out.push_str(&format!(
            "  ; aero.fused_kernel id={} ops={} range={}..{}\n",
            id,
            chain.len(),
            start_line,
            end_line
        ));
        kernels.push(FusedKernel {
            id,
            op_count: chain.len(),
            start_line,
            end_line,
        });
    }

    for (_, line) in chain.drain(..) {
        out.push_str(&line);
        out.push('\n');
    }
}

fn is_elementwise_candidate(line: &str) -> bool {
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
    fn graph_compilation_emits_fusion_metadata_for_arithmetic_chain() {
        let input = "define i32 @main() {\nentry:\n  %0 = fadd double %a, %b\n  %1 = fmul double %0, %c\n  ret i32 0\n}\n";
        let (out, report) = apply_advanced_graph_compilation(input);
        assert!(out.contains("; aero.graph_compilation=enabled"));
        assert!(out.contains("; aero.fused_kernel id=1 ops=2"));
        assert_eq!(report.fused_kernel_count, 1);
        assert_eq!(report.total_fused_ops, 2);
    }

    #[test]
    fn graph_compilation_skips_singleton_chains() {
        let input =
            "define i32 @main() {\nentry:\n  %0 = fadd double %a, %b\n  br label %done\n}\n";
        let (out, report) = apply_advanced_graph_compilation(input);
        assert!(!out.contains("; aero.fused_kernel"));
        assert_eq!(report.fused_kernel_count, 0);
        assert_eq!(report.total_fused_ops, 0);
    }
}
