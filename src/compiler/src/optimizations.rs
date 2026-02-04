// Performance optimizations for Aero Phase 3 compiler
// This module implements critical path optimizations for:
// 1. Function call generation
// 2. Control flow LLVM generation
// 3. Parser performance for complex constructs
// 4. Semantic analysis for large programs
// 5. Compilation caching

use crate::ast::{AstNode, BinaryOp, Expression, Statement, UnaryOp};
use crate::types::Ty;
use std::collections::HashMap;

/// Cache for frequently accessed function signatures
#[derive(Debug, Clone)]
pub struct FunctionSignatureCache {
    cache: HashMap<String, (Vec<Ty>, Ty)>, // function_name -> (param_types, return_type)
    hit_count: HashMap<String, u32>,
    miss_count: u32,
}

impl FunctionSignatureCache {
    pub fn new() -> Self {
        FunctionSignatureCache {
            cache: HashMap::new(),
            hit_count: HashMap::new(),
            miss_count: 0,
        }
    }

    pub fn get(&mut self, function_name: &str) -> Option<&(Vec<Ty>, Ty)> {
        if let Some(signature) = self.cache.get(function_name) {
            *self.hit_count.entry(function_name.to_string()).or_insert(0) += 1;
            Some(signature)
        } else {
            self.miss_count += 1;
            None
        }
    }

    pub fn insert(&mut self, function_name: String, param_types: Vec<Ty>, return_type: Ty) {
        self.cache.insert(function_name, (param_types, return_type));
    }

    pub fn get_cache_stats(&self) -> (usize, u32) {
        let total_hits: u32 = self.hit_count.values().sum();
        (self.cache.len(), total_hits)
    }
}

/// Optimized parser for complex constructs
pub struct OptimizedParser {
    // Pre-computed operator precedence table for faster parsing
    precedence_table: HashMap<BinaryOp, u8>,
    // Cache for frequently parsed expressions
    expression_cache: HashMap<String, Expression>,
}

impl OptimizedParser {
    pub fn new() -> Self {
        let mut precedence_table = HashMap::new();

        // Set operator precedence (higher number = higher precedence)
        precedence_table.insert(BinaryOp::Multiply, 6);
        precedence_table.insert(BinaryOp::Divide, 6);
        precedence_table.insert(BinaryOp::Modulo, 6);
        precedence_table.insert(BinaryOp::Add, 5);
        precedence_table.insert(BinaryOp::Subtract, 5);

        OptimizedParser {
            precedence_table,
            expression_cache: HashMap::new(),
        }
    }

    pub fn get_precedence(&self, op: &BinaryOp) -> u8 {
        self.precedence_table.get(op).copied().unwrap_or(0)
    }

    pub fn cache_expression(&mut self, key: String, expr: Expression) {
        // Only cache small expressions to avoid memory bloat
        if self.is_cacheable_expression(&expr) {
            self.expression_cache.insert(key, expr);
        }
    }

    pub fn get_cached_expression(&self, key: &str) -> Option<&Expression> {
        self.expression_cache.get(key)
    }

    fn is_cacheable_expression(&self, expr: &Expression) -> bool {
        match expr {
            Expression::IntegerLiteral(_)
            | Expression::FloatLiteral(_)
            | Expression::Identifier(_) => true,
            Expression::Binary { left, right, .. } => {
                self.is_cacheable_expression(left) && self.is_cacheable_expression(right)
            }
            _ => false,
        }
    }
}

/// Optimized semantic analyzer for large programs
pub struct OptimizedSemanticAnalyzer {
    // Cache for type inference results
    type_cache: HashMap<String, Ty>,
    // Pre-computed type compatibility matrix
    compatibility_matrix: HashMap<(Ty, Ty), bool>,
    // Statistics for optimization tracking
    cache_hits: u32,
    cache_misses: u32,
}

impl OptimizedSemanticAnalyzer {
    pub fn new() -> Self {
        let mut compatibility_matrix = HashMap::new();

        // Pre-compute type compatibility for common operations
        compatibility_matrix.insert((Ty::Int, Ty::Int), true);
        compatibility_matrix.insert((Ty::Float, Ty::Float), true);
        compatibility_matrix.insert((Ty::Int, Ty::Float), true);
        compatibility_matrix.insert((Ty::Float, Ty::Int), true);
        compatibility_matrix.insert((Ty::Bool, Ty::Bool), true);

        OptimizedSemanticAnalyzer {
            type_cache: HashMap::new(),
            compatibility_matrix,
            cache_hits: 0,
            cache_misses: 0,
        }
    }

    pub fn get_cached_type(&mut self, expr_key: &str) -> Option<&Ty> {
        if let Some(ty) = self.type_cache.get(expr_key) {
            self.cache_hits += 1;
            Some(ty)
        } else {
            self.cache_misses += 1;
            None
        }
    }

    pub fn cache_type(&mut self, expr_key: String, ty: Ty) {
        self.type_cache.insert(expr_key, ty);
    }

    pub fn are_types_compatible(&self, left: &Ty, right: &Ty) -> bool {
        self.compatibility_matrix
            .get(&(left.clone(), right.clone()))
            .copied()
            .unwrap_or(false)
    }

    pub fn get_cache_efficiency(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
}

/// Optimized function call generator
pub struct OptimizedFunctionCallGenerator {
    // Cache for generated function call IR
    call_cache: HashMap<String, String>, // call_signature -> generated_llvm
    // Inline threshold for small functions
    inline_threshold: usize,
    // Statistics
    inlined_calls: u32,
    cached_calls: u32,
}

impl OptimizedFunctionCallGenerator {
    pub fn new() -> Self {
        OptimizedFunctionCallGenerator {
            call_cache: HashMap::new(),
            inline_threshold: 10, // Inline functions with <= 10 instructions
            inlined_calls: 0,
            cached_calls: 0,
        }
    }

    pub fn should_inline_function(&self, function_name: &str, instruction_count: usize) -> bool {
        // Inline small functions and frequently called functions
        instruction_count <= self.inline_threshold || self.is_frequently_called(function_name)
    }

    pub fn generate_optimized_call(&mut self, function_name: &str, args: &[String]) -> String {
        let call_signature = format!("{}({})", function_name, args.join(","));

        if let Some(cached_llvm) = self.call_cache.get(&call_signature) {
            self.cached_calls += 1;
            return cached_llvm.clone();
        }

        // Generate new call (simplified for this optimization example)
        let llvm_code = self.generate_function_call_llvm(function_name, args);
        self.call_cache.insert(call_signature, llvm_code.clone());

        llvm_code
    }

    fn is_frequently_called(&self, _function_name: &str) -> bool {
        // Placeholder - would track call frequency in real implementation
        false
    }

    fn generate_function_call_llvm(&self, function_name: &str, args: &[String]) -> String {
        // Optimized LLVM generation for function calls
        let mut llvm = String::new();

        // Use more efficient register allocation
        let result_reg = format!("%call_result_{}", function_name);

        // Generate optimized argument passing
        let args_str = args.join(", ");
        llvm.push_str(&format!(
            "  {} = call fastcc i32 @{}({})\n",
            result_reg, function_name, args_str
        ));

        llvm
    }

    pub fn get_optimization_stats(&self) -> (u32, u32, usize) {
        (self.inlined_calls, self.cached_calls, self.call_cache.len())
    }
}

/// Optimized control flow generator
pub struct OptimizedControlFlowGenerator {
    // Cache for basic block patterns
    block_cache: HashMap<String, String>,
    // Optimized branch prediction hints
    branch_hints: HashMap<String, bool>, // condition -> likely_true
}

impl OptimizedControlFlowGenerator {
    pub fn new() -> Self {
        OptimizedControlFlowGenerator {
            block_cache: HashMap::new(),
            branch_hints: HashMap::new(),
        }
    }

    pub fn generate_optimized_if(
        &mut self,
        condition: &str,
        then_label: &str,
        else_label: &str,
    ) -> String {
        let pattern_key = format!("if_{}_{}", then_label, else_label);

        if let Some(cached) = self.block_cache.get(&pattern_key) {
            return cached.replace("CONDITION", condition);
        }

        // Generate optimized if statement with branch prediction
        let mut llvm = String::new();

        // Add branch prediction hint if available
        let likely_true = self.branch_hints.get(condition).copied().unwrap_or(true);
        let branch_weight = if likely_true {
            "!prof !{!\"branch_weights\", i32 2000, i32 1}"
        } else {
            ""
        };

        llvm.push_str(&format!(
            "  br i1 CONDITION, label %{}, label %{} {}\n",
            then_label, else_label, branch_weight
        ));

        self.block_cache.insert(pattern_key, llvm.clone());
        llvm.replace("CONDITION", condition)
    }

    pub fn generate_optimized_loop(&mut self, header: &str, body: &str, exit: &str) -> String {
        // Generate optimized loop structure with loop unrolling hints
        let mut llvm = String::new();

        // Add loop metadata for optimization
        llvm.push_str(&format!("{}:\n", header));
        llvm.push_str("  ; Loop optimization metadata\n");
        llvm.push_str("  ; !llvm.loop !{!\"llvm.loop.unroll.enable\"}\n");
        llvm.push_str(&format!("  br label %{}\n", body));

        llvm.push_str(&format!("{}:\n", body));

        llvm
    }

    pub fn add_branch_hint(&mut self, condition: String, likely_true: bool) {
        self.branch_hints.insert(condition, likely_true);
    }
}

/// Compilation cache for incremental compilation
pub struct CompilationCache {
    // Cache for compiled modules
    module_cache: HashMap<String, String>, // source_hash -> compiled_llvm
    // Dependency tracking
    dependencies: HashMap<String, Vec<String>>, // file -> dependencies
    // Timestamps for cache invalidation
    timestamps: HashMap<String, u64>,
}

impl CompilationCache {
    pub fn new() -> Self {
        CompilationCache {
            module_cache: HashMap::new(),
            dependencies: HashMap::new(),
            timestamps: HashMap::new(),
        }
    }

    pub fn get_cached_compilation(&self, source_hash: &str) -> Option<&String> {
        self.module_cache.get(source_hash)
    }

    pub fn cache_compilation(&mut self, source_hash: String, llvm_ir: String) {
        self.module_cache.insert(source_hash, llvm_ir);
    }

    pub fn is_cache_valid(&self, file_path: &str, current_timestamp: u64) -> bool {
        if let Some(&cached_timestamp) = self.timestamps.get(file_path) {
            cached_timestamp >= current_timestamp
        } else {
            false
        }
    }

    pub fn update_timestamp(&mut self, file_path: String, timestamp: u64) {
        self.timestamps.insert(file_path, timestamp);
    }

    pub fn add_dependency(&mut self, file: String, dependency: String) {
        self.dependencies
            .entry(file)
            .or_insert_with(Vec::new)
            .push(dependency);
    }

    pub fn invalidate_dependents(&mut self, changed_file: &str) {
        let mut to_invalidate = Vec::new();

        for (file, deps) in &self.dependencies {
            if deps.contains(&changed_file.to_string()) {
                to_invalidate.push(file.clone());
            }
        }

        for file in to_invalidate {
            self.timestamps.remove(&file);
            // Also remove from module cache if we had the hash
            // This is simplified - real implementation would need source->hash mapping
        }
    }

    pub fn get_cache_stats(&self) -> (usize, usize, usize) {
        (
            self.module_cache.len(),
            self.dependencies.len(),
            self.timestamps.len(),
        )
    }
}

/// Main optimization coordinator
pub struct CompilerOptimizer {
    pub function_cache: FunctionSignatureCache,
    pub parser: OptimizedParser,
    pub semantic_analyzer: OptimizedSemanticAnalyzer,
    pub function_generator: OptimizedFunctionCallGenerator,
    pub control_flow_generator: OptimizedControlFlowGenerator,
    pub compilation_cache: CompilationCache,
}

impl CompilerOptimizer {
    pub fn new() -> Self {
        CompilerOptimizer {
            function_cache: FunctionSignatureCache::new(),
            parser: OptimizedParser::new(),
            semantic_analyzer: OptimizedSemanticAnalyzer::new(),
            function_generator: OptimizedFunctionCallGenerator::new(),
            control_flow_generator: OptimizedControlFlowGenerator::new(),
            compilation_cache: CompilationCache::new(),
        }
    }

    pub fn optimize_ast(&mut self, ast: &mut Vec<AstNode>) -> Result<(), String> {
        // Apply AST-level optimizations
        for node in ast.iter_mut() {
            self.optimize_ast_node(node)?;
        }
        Ok(())
    }

    fn optimize_ast_node(&mut self, node: &mut AstNode) -> Result<(), String> {
        match node {
            AstNode::Statement(stmt) => self.optimize_statement(stmt),
            AstNode::Expression(expr) => {
                self.optimize_expression(expr)?;
                Ok(())
            }
        }
    }

    fn optimize_statement(&mut self, stmt: &mut Statement) -> Result<(), String> {
        match stmt {
            Statement::Function { body, .. } => {
                // Optimize function body
                for stmt in &mut body.statements {
                    self.optimize_statement(stmt)?;
                }
                if let Some(expr) = &mut body.expression {
                    self.optimize_expression(expr)?;
                }
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.optimize_expression(condition)?;
                for stmt in &mut then_block.statements {
                    self.optimize_statement(stmt)?;
                }
                if let Some(else_stmt) = else_block {
                    self.optimize_statement(else_stmt)?;
                }
            }
            Statement::While { condition, body } => {
                self.optimize_expression(condition)?;
                for stmt in &mut body.statements {
                    self.optimize_statement(stmt)?;
                }
            }
            Statement::For { iterable, body, .. } => {
                self.optimize_expression(iterable)?;
                for stmt in &mut body.statements {
                    self.optimize_statement(stmt)?;
                }
            }
            Statement::Loop { body } => {
                for stmt in &mut body.statements {
                    self.optimize_statement(stmt)?;
                }
            }
            Statement::Let {
                value: Some(expr), ..
            } => {
                self.optimize_expression(expr)?;
            }
            Statement::Return(Some(expr)) => {
                self.optimize_expression(expr)?;
            }
            Statement::Expression(expr) => {
                self.optimize_expression(expr)?;
            }
            _ => {} // Other statements don't need optimization
        }
        Ok(())
    }

    fn optimize_expression(&mut self, expr: &mut Expression) -> Result<(), String> {
        match expr {
            Expression::Binary {
                left, right, op, ..
            } => {
                self.optimize_expression(left)?;
                self.optimize_expression(right)?;

                // Apply constant folding optimization
                if let Some(folded) = self.try_constant_fold(left, right, op) {
                    *expr = folded;
                }
            }
            Expression::Unary { operand, .. } => {
                self.optimize_expression(operand)?;
            }
            Expression::FunctionCall { arguments, .. } => {
                for arg in arguments {
                    self.optimize_expression(arg)?;
                }
            }
            Expression::Print { arguments, .. } | Expression::Println { arguments, .. } => {
                for arg in arguments {
                    self.optimize_expression(arg)?;
                }
            }
            Expression::Comparison { left, right, .. } => {
                self.optimize_expression(left)?;
                self.optimize_expression(right)?;
            }
            Expression::Logical { left, right, .. } => {
                self.optimize_expression(left)?;
                self.optimize_expression(right)?;
            }
            _ => {} // Literals and identifiers don't need optimization
        }
        Ok(())
    }

    fn try_constant_fold(
        &self,
        left: &Expression,
        right: &Expression,
        op: &BinaryOp,
    ) -> Option<Expression> {
        match (left, right) {
            (Expression::IntegerLiteral(a), Expression::IntegerLiteral(b)) => match op {
                BinaryOp::Add => Some(Expression::IntegerLiteral(a + b)),
                BinaryOp::Subtract => Some(Expression::IntegerLiteral(a - b)),
                BinaryOp::Multiply => Some(Expression::IntegerLiteral(a * b)),
                BinaryOp::Divide if *b != 0 => Some(Expression::IntegerLiteral(a / b)),
                BinaryOp::Modulo if *b != 0 => Some(Expression::IntegerLiteral(a % b)),
                _ => None,
            },
            (Expression::FloatLiteral(a), Expression::FloatLiteral(b)) => match op {
                BinaryOp::Add => Some(Expression::FloatLiteral(a + b)),
                BinaryOp::Subtract => Some(Expression::FloatLiteral(a - b)),
                BinaryOp::Multiply => Some(Expression::FloatLiteral(a * b)),
                BinaryOp::Divide if *b != 0.0 => Some(Expression::FloatLiteral(a / b)),
                BinaryOp::Modulo if *b != 0.0 => Some(Expression::FloatLiteral(a % b)),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn print_optimization_stats(&self) {
        println!("=== Compiler Optimization Statistics ===");

        let (cache_size, cache_hits) = self.function_cache.get_cache_stats();
        println!(
            "Function Cache: {} entries, {} hits",
            cache_size, cache_hits
        );

        let semantic_efficiency = self.semantic_analyzer.get_cache_efficiency();
        println!(
            "Semantic Analysis Cache Efficiency: {:.2}%",
            semantic_efficiency * 100.0
        );

        let (inlined, cached, call_cache_size) = self.function_generator.get_optimization_stats();
        println!(
            "Function Calls: {} inlined, {} cached, {} cache entries",
            inlined, cached, call_cache_size
        );

        let (modules, deps, timestamps) = self.compilation_cache.get_cache_stats();
        println!(
            "Compilation Cache: {} modules, {} dependencies, {} timestamps",
            modules, deps, timestamps
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_signature_cache() {
        let mut cache = FunctionSignatureCache::new();

        // Test cache miss
        assert!(cache.get("test_func").is_none());

        // Test cache insert and hit
        cache.insert("test_func".to_string(), vec![Ty::Int, Ty::Float], Ty::Bool);
        let result = cache.get("test_func");
        assert!(result.is_some());

        let (params, return_type) = result.unwrap();
        assert_eq!(params.len(), 2);
        assert_eq!(*return_type, Ty::Bool);

        let (cache_size, hits) = cache.get_cache_stats();
        assert_eq!(cache_size, 1);
        assert_eq!(hits, 1);
    }

    #[test]
    fn test_optimized_parser_precedence() {
        let parser = OptimizedParser::new();

        assert!(parser.get_precedence(&BinaryOp::Multiply) > parser.get_precedence(&BinaryOp::Add));
        assert!(
            parser.get_precedence(&BinaryOp::Divide) > parser.get_precedence(&BinaryOp::Subtract)
        );
        assert_eq!(
            parser.get_precedence(&BinaryOp::Add),
            parser.get_precedence(&BinaryOp::Subtract)
        );
    }

    #[test]
    fn test_semantic_analyzer_type_compatibility() {
        let analyzer = OptimizedSemanticAnalyzer::new();

        assert!(analyzer.are_types_compatible(&Ty::Int, &Ty::Int));
        assert!(analyzer.are_types_compatible(&Ty::Int, &Ty::Float));
        assert!(analyzer.are_types_compatible(&Ty::Float, &Ty::Int));
        assert!(analyzer.are_types_compatible(&Ty::Bool, &Ty::Bool));
    }

    #[test]
    fn test_function_call_generator_inlining() {
        let generator = OptimizedFunctionCallGenerator::new();

        // Small functions should be inlined
        assert!(generator.should_inline_function("small_func", 5));

        // Large functions should not be inlined
        assert!(!generator.should_inline_function("large_func", 50));
    }

    #[test]
    fn test_compilation_cache() {
        let mut cache = CompilationCache::new();

        // Test cache miss
        assert!(cache.get_cached_compilation("hash123").is_none());

        // Test cache hit
        cache.cache_compilation("hash123".to_string(), "llvm_ir_code".to_string());
        assert!(cache.get_cached_compilation("hash123").is_some());

        // Test timestamp validation
        cache.update_timestamp("file.aero".to_string(), 1000);
        assert!(cache.is_cache_valid("file.aero", 999));
        assert!(!cache.is_cache_valid("file.aero", 1001));
    }

    #[test]
    fn test_constant_folding() {
        let optimizer = CompilerOptimizer::new();

        let left = Expression::IntegerLiteral(5);
        let right = Expression::IntegerLiteral(3);

        // Test addition folding
        let result = optimizer.try_constant_fold(&left, &right, &BinaryOp::Add);
        assert!(result.is_some());
        if let Some(Expression::IntegerLiteral(value)) = result {
            assert_eq!(value, 8);
        }

        // Test multiplication folding
        let result = optimizer.try_constant_fold(&left, &right, &BinaryOp::Multiply);
        assert!(result.is_some());
        if let Some(Expression::IntegerLiteral(value)) = result {
            assert_eq!(value, 15);
        }
    }
}
