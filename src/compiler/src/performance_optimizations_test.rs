#[cfg(test)]
mod performance_optimization_tests {
    use super::performance_optimizations::*;
    use crate::ir::{Value, Inst};

    #[test]
    fn test_function_call_optimizer_creation() {
        let optimizer = FunctionCallOptimizer::new();
        assert_eq!(optimizer.inline_threshold, 10);
        assert_eq!(optimizer.call_frequency.len(), 0);
        assert_eq!(optimizer.function_sizes.len(), 0);
    }

    #[test]
    fn test_function_call_frequency_tracking() {
        let mut optimizer = FunctionCallOptimizer::new();
        
        // Record multiple calls to the same function
        optimizer.record_function_call("test_func");
        optimizer.record_function_call("test_func");
        optimizer.record_function_call("test_func");
        
        assert_eq!(optimizer.call_frequency.get("test_func"), Some(&3));
    }

    #[test]
    fn test_function_inlining_decision() {
        let mut optimizer = FunctionCallOptimizer::new();
        
        // Small function should be inlined
        optimizer.record_function_size("small_func", 5);
        assert!(optimizer.should_inline_function("small_func"));
        
        // Large function should not be inlined initially
        optimizer.record_function_size("large_func", 20);
        assert!(!optimizer.should_inline_function("large_func"));
        
        // But frequently called large function should be inlined
        for _ in 0..6 {
            optimizer.record_function_call("large_func");
        }
        assert!(optimizer.should_inline_function("large_func"));
    }

    #[test]
    fn test_control_flow_optimizer_creation() {
        let optimizer = ControlFlowOptimizer::new();
        assert_eq!(optimizer.basic_block_cache.len(), 0);
        assert_eq!(optimizer.branch_optimization_cache.len(), 0);
    }

    #[test]
    fn test_branch_optimization_constant_folding() {
        let mut optimizer = ControlFlowOptimizer::new();
        
        // Test constant true condition
        let optimized = optimizer.optimize_branch_generation(
            &Value::ImmInt(1), 
            "true_label", 
            "false_label"
        );
        
        // Should generate direct jump to true label
        assert_eq!(optimized.len(), 1);
        match &optimized[0] {
            Inst::Jump(label) => assert_eq!(label, "true_label"),
            _ => panic!("Expected Jump instruction"),
        }
        
        // Test constant false condition
        let optimized = optimizer.optimize_branch_generation(
            &Value::ImmInt(0), 
            "true_label", 
            "false_label"
        );
        
        // Should generate direct jump to false label
        assert_eq!(optimized.len(), 1);
        match &optimized[0] {
            Inst::Jump(label) => assert_eq!(label, "false_label"),
            _ => panic!("Expected Jump instruction"),
        }
    }

    #[test]
    fn test_parser_optimizer_creation() {
        let optimizer = ParserOptimizer::new();
        assert_eq!(optimizer.expression_cache.len(), 0);
        assert_eq!(optimizer.statement_cache.len(), 0);
    }

    #[test]
    fn test_semantic_optimizer_creation() {
        let optimizer = SemanticOptimizer::new();
        assert_eq!(optimizer.symbol_table_cache.len(), 0);
        assert_eq!(optimizer.type_inference_cache.len(), 0);
        assert_eq!(optimizer.scope_analysis_cache.len(), 0);
    }

    #[test]
    fn test_compilation_cache_creation() {
        let cache = CompilationCache::new();
        assert_eq!(cache.ast_cache.len(), 0);
        assert_eq!(cache.ir_cache.len(), 0);
        assert_eq!(cache.llvm_cache.len(), 0);
        assert_eq!(cache.cache_hits, 0);
        assert_eq!(cache.cache_misses, 0);
    }

    #[test]
    fn test_compilation_cache_statistics() {
        let mut cache = CompilationCache::new();
        
        // Test cache miss
        let result = cache.get_cached_llvm("nonexistent");
        assert!(result.is_none());
        
        // Cache something
        cache.cache_llvm("test_hash".to_string(), "test_llvm_code".to_string());
        
        // Test cache hit
        let result = cache.get_cached_llvm("test_hash");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), "test_llvm_code");
        
        // Check statistics
        let (hits, misses, hit_rate) = cache.get_cache_stats();
        assert_eq!(hits, 1);
        assert_eq!(misses, 1);
        assert_eq!(hit_rate, 0.5);
    }

    #[test]
    fn test_performance_optimizer_creation() {
        let optimizer = PerformanceOptimizer::new();
        
        // Test that all sub-optimizers are created
        let report = optimizer.get_performance_report();
        assert!(report.contains("Function Call Optimization"));
        assert!(report.contains("Control Flow Optimization"));
        assert!(report.contains("Parser Optimization"));
        assert!(report.contains("Semantic Analysis Optimization"));
        assert!(report.contains("Compilation Cache"));
    }

    #[test]
    fn test_performance_metrics_collection() {
        let mut metrics = PerformanceMetrics::new();
        
        // Record some metrics
        metrics.record_function_call_time(0.001);
        metrics.record_function_call_time(0.002);
        metrics.record_control_flow_time(0.003);
        
        // Test averages
        assert_eq!(metrics.get_average_function_call_time(), 0.0015);
        assert_eq!(metrics.get_total_function_call_time(), 0.003);
        assert_eq!(metrics.function_call_times.len(), 2);
        assert_eq!(metrics.control_flow_times.len(), 1);
    }

    #[test]
    fn test_instruction_sequence_optimization() {
        let optimizer = ControlFlowOptimizer::new();
        
        // Create a sequence with redundant loads
        let instructions = vec![
            Inst::Load(Value::Reg(0), Value::Reg(100)),
            Inst::Load(Value::Reg(1), Value::Reg(100)), // Redundant load from same location
            Inst::Store(Value::Reg(100), Value::ImmInt(42)),
            Inst::Load(Value::Reg(2), Value::Reg(100)), // Should not be eliminated (after store)
        ];
        
        let optimized = optimizer.optimize_instruction_sequence(&instructions);
        
        // Should eliminate the redundant load but keep the one after store
        assert_eq!(optimized.len(), 3);
    }

    #[test]
    fn test_phi_node_optimization() {
        let optimizer = ControlFlowOptimizer::new();
        
        // Single incoming value should be eliminated
        let single_incoming = vec![(Value::ImmInt(42), "block1".to_string())];
        let result = optimizer.optimize_phi_node_generation("var1", &single_incoming);
        assert!(result.is_none());
        
        // Multiple incoming values should generate phi node
        let multiple_incoming = vec![
            (Value::ImmInt(42), "block1".to_string()),
            (Value::ImmInt(24), "block2".to_string()),
        ];
        let result = optimizer.optimize_phi_node_generation("var2", &multiple_incoming);
        assert!(result.is_some());
    }
}