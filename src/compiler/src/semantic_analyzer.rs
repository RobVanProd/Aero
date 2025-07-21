use std::collections::HashMap;
use crate::ast::{AstNode, Expression, Statement, Parameter, Type, Block};
use crate::types::{Ty, infer_binary_type};

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub ptr_name: String,
    pub ty: Ty, // Changed from String to Ty
    pub initialized: bool,
}

#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub name: String,
    pub var_type: Ty,
    pub mutable: bool,
    pub initialized: bool,
    pub scope_level: u32,
    pub ptr_name: String,
}

#[derive(Debug, Clone)]
pub struct ScopeManager {
    scopes: Vec<HashMap<String, VariableInfo>>,
    current_function: Option<String>,
    in_loop: u32,
    scope_level: u32,
    next_ptr_id: u32,
}

impl ScopeManager {
    pub fn new() -> Self {
        ScopeManager {
            scopes: vec![HashMap::new()], // Start with global scope
            current_function: None,
            in_loop: 0,
            scope_level: 0,
            next_ptr_id: 0,
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.scope_level += 1;
    }

    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
            self.scope_level -= 1;
        }
    }

    pub fn enter_function(&mut self, name: String) {
        self.current_function = Some(name);
        self.enter_scope(); // Functions create their own scope
    }

    pub fn exit_function(&mut self) {
        self.current_function = None;
        self.exit_scope(); // Exit function scope
    }

    pub fn enter_loop(&mut self) {
        self.in_loop += 1;
        self.enter_scope(); // Loops create their own scope
    }

    pub fn exit_loop(&mut self) {
        if self.in_loop > 0 {
            self.in_loop -= 1;
            self.exit_scope(); // Exit loop scope
        }
    }

    pub fn define_variable(&mut self, name: String, var_type: Ty, mutable: bool, initialized: bool) -> Result<String, String> {
        // Check if variable already exists in current scope (no shadowing within same scope)
        if let Some(current_scope) = self.scopes.last() {
            if current_scope.contains_key(&name) {
                return Err(format!("Error: Variable `{}` is already defined in this scope.", name));
            }
        }

        let ptr_name = self.fresh_ptr_name();
        let var_info = VariableInfo {
            name: name.clone(),
            var_type,
            mutable,
            initialized,
            scope_level: self.scope_level,
            ptr_name: ptr_name.clone(),
        };

        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name, var_info);
        }

        Ok(ptr_name)
    }

    pub fn get_variable(&self, name: &str) -> Option<&VariableInfo> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter().rev() {
            if let Some(var_info) = scope.get(name) {
                return Some(var_info);
            }
        }
        None
    }

    pub fn update_variable_initialization(&mut self, name: &str, initialized: bool) -> Result<(), String> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var_info) = scope.get_mut(name) {
                var_info.initialized = initialized;
                return Ok(());
            }
        }
        Err(format!("Error: Variable `{}` not found.", name))
    }

    pub fn can_break_continue(&self) -> bool {
        self.in_loop > 0
    }

    pub fn is_in_function(&self) -> bool {
        self.current_function.is_some()
    }

    pub fn get_current_function(&self) -> Option<&String> {
        self.current_function.as_ref()
    }

    pub fn check_mutability(&self, name: &str) -> Result<bool, String> {
        if let Some(var_info) = self.get_variable(name) {
            Ok(var_info.mutable)
        } else {
            Err(format!("Error: Variable `{}` not found.", name))
        }
    }

    pub fn get_scope_level(&self) -> u32 {
        self.scope_level
    }

    pub fn get_loop_depth(&self) -> u32 {
        self.in_loop
    }

    fn fresh_ptr_name(&mut self) -> String {
        let ptr_name = format!("ptr{}", self.next_ptr_id);
        self.next_ptr_id += 1;
        ptr_name
    }

    // Helper method to check if a variable is shadowing another variable
    pub fn is_shadowing(&self, name: &str) -> bool {
        let mut found_count = 0;
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(name) {
                found_count += 1;
                if found_count > 1 {
                    return true;
                }
            }
        }
        false
    }

    // Get all variables in current scope (for debugging/testing)
    pub fn get_current_scope_variables(&self) -> Vec<&String> {
        if let Some(current_scope) = self.scopes.last() {
            current_scope.keys().collect()
        } else {
            vec![]
        }
    }

    // Get all variables across all scopes (for debugging/testing)
    pub fn get_all_variables(&self) -> Vec<&VariableInfo> {
        let mut all_vars = Vec::new();
        for scope in &self.scopes {
            for var_info in scope.values() {
                all_vars.push(var_info);
            }
        }
        all_vars
    }

    // Check if variable exists in current scope
    pub fn variable_exists_in_current_scope(&self, name: &str) -> bool {
        if let Some(current_scope) = self.scopes.last() {
            current_scope.contains_key(name)
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Ty,
    pub defined_at: Option<String>, // Source location placeholder
}

pub struct FunctionTable {
    functions: HashMap<String, FunctionInfo>,
}

impl FunctionTable {
    pub fn new() -> Self {
        FunctionTable {
            functions: HashMap::new(),
        }
    }

    pub fn define_function(&mut self, info: FunctionInfo) -> Result<(), String> {
        if self.functions.contains_key(&info.name) {
            return Err(format!("Error: Function `{}` is already defined.", info.name));
        }
        self.functions.insert(info.name.clone(), info);
        Ok(())
    }

    pub fn get_function(&self, name: &str) -> Option<&FunctionInfo> {
        self.functions.get(name)
    }

    pub fn validate_call(&self, name: &str, args: &[Ty]) -> Result<Ty, String> {
        if let Some(func_info) = self.functions.get(name) {
            // Check arity (number of arguments)
            if args.len() != func_info.parameters.len() {
                return Err(format!(
                    "Error: Function `{}` expects {} arguments, but {} were provided.",
                    name,
                    func_info.parameters.len(),
                    args.len()
                ));
            }

            // Check argument types
            for (i, (param, arg_type)) in func_info.parameters.iter().zip(args.iter()).enumerate() {
                let expected_type = self.type_from_ast_type(&param.param_type)?;
                if *arg_type != expected_type {
                    return Err(format!(
                        "Error: Function `{}` parameter {} expects type `{}`, but `{}` was provided.",
                        name,
                        i + 1,
                        expected_type.to_string(),
                        arg_type.to_string()
                    ));
                }
            }

            Ok(func_info.return_type.clone())
        } else {
            Err(format!("Error: Undefined function `{}`.", name))
        }
    }

    fn type_from_ast_type(&self, ast_type: &Type) -> Result<Ty, String> {
        match ast_type.name.as_str() {
            "i32" | "int" => Ok(Ty::Int),
            "f64" | "float" => Ok(Ty::Float),
            "bool" => Ok(Ty::Bool),
            _ => Err(format!("Error: Unknown type `{}`.", ast_type.name)),
        }
    }

    pub fn list_functions(&self) -> Vec<&String> {
        self.functions.keys().collect()
    }
}

pub struct SemanticAnalyzer {
    scope_manager: ScopeManager,
    function_table: FunctionTable,
    // Keep old symbol_table for backward compatibility during transition
    symbol_table: HashMap<String, VarInfo>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer { 
            scope_manager: ScopeManager::new(),
            function_table: FunctionTable::new(),
            symbol_table: HashMap::new(),
        }
    }

    pub fn get_scope_manager(&self) -> &ScopeManager {
        &self.scope_manager
    }

    pub fn get_scope_manager_mut(&mut self) -> &mut ScopeManager {
        &mut self.scope_manager
    }

    pub fn analyze(&mut self, mut ast: Vec<AstNode>) -> Result<(String, Vec<AstNode>), String> {
        // First pass: type inference and validation
        for node in &mut ast {
            match node {
                AstNode::Statement(stmt) => {
                    match stmt {
                        Statement::Let { name, value } => {
                            if self.symbol_table.contains_key(name) {
                                return Err(format!("Error: Variable `{}` already declared.", name));
                            }
                            
                            // Infer and validate the expression type
                            let inferred_type = self.infer_and_validate_expression(value)?;
                            let ptr_name = self.fresh_ptr_name();
                            let var_info = VarInfo {
                                ptr_name: ptr_name.clone(),
                                ty: inferred_type.clone(),
                                initialized: true, // Let statements always initialize
                            };
                            self.symbol_table.insert(name.clone(), var_info);
                            println!("Declared variable: {} with type {} (ptr: {})", name, inferred_type.to_string(), ptr_name);
                        }
                        Statement::Return(expr) => {
                            // Check if the expression uses any variables and validate they are initialized
                            self.check_expression_initialization(expr)?;
                            self.infer_and_validate_expression(expr)?;
                            println!("Return statement with expression of type inferred.");
                        }
                        Statement::Function { .. } => {
                            // TODO: Implement function definition semantic analysis
                            // This will be implemented in task 6.1
                            println!("Function definition found - semantic analysis not yet implemented");
                        }
                        Statement::If { condition, then_block, else_block } => {
                            // Validate condition expression
                            self.check_expression_initialization(condition)?;
                            let condition_type = self.infer_and_validate_expression(condition)?;
                            
                            // Condition must be boolean
                            if condition_type != Ty::Bool {
                                return Err(format!("Error: If condition must be boolean, found: {}", condition_type.to_string()));
                            }

                            // Analyze then block in its own scope
                            self.scope_manager.enter_scope();
                            self.analyze_block(then_block)?;
                            self.scope_manager.exit_scope();

                            // Analyze else block if present
                            if let Some(else_stmt) = else_block {
                                self.scope_manager.enter_scope();
                                self.analyze_statement(else_stmt)?;
                                self.scope_manager.exit_scope();
                            }
                        }
                        Statement::While { condition, body } => {
                            // Validate condition expression
                            self.check_expression_initialization(condition)?;
                            let condition_type = self.infer_and_validate_expression(condition)?;
                            
                            // Condition must be boolean
                            if condition_type != Ty::Bool {
                                return Err(format!("Error: While condition must be boolean, found: {}", condition_type.to_string()));
                            }

                            // Enter loop context for break/continue validation
                            self.scope_manager.enter_loop();
                            self.analyze_block(body)?;
                            self.scope_manager.exit_loop();
                        }
                        Statement::For { variable, iterable, body } => {
                            // Validate iterable expression
                            self.check_expression_initialization(iterable)?;
                            let _iterable_type = self.infer_and_validate_expression(iterable)?;
                            
                            // TODO: Add proper range/iterator type checking when implemented
                            // For now, accept any type as iterable (placeholder)

                            // Enter loop context and define loop variable
                            self.scope_manager.enter_loop();
                            
                            // Define the loop variable in the loop scope
                            // TODO: Infer proper type from iterable when range types are implemented
                            self.scope_manager.define_variable(
                                variable.clone(),
                                Ty::Int, // Placeholder - should be inferred from iterable
                                false,   // Loop variables are immutable by default
                                true,    // Loop variables are always initialized
                            )?;

                            self.analyze_block(body)?;
                            self.scope_manager.exit_loop();
                        }
                        Statement::Loop { body } => {
                            // Enter loop context for break/continue validation
                            self.scope_manager.enter_loop();
                            self.analyze_block(body)?;
                            self.scope_manager.exit_loop();
                        }
                        Statement::Break => {
                            // Validate break is inside a loop
                            if !self.scope_manager.can_break_continue() {
                                return Err("Error: Break statement outside of loop.".to_string());
                            }
                            
                            // TODO: Implement unreachable code detection after break
                            // Any statements after break in the same block should be flagged as unreachable
                        }
                        Statement::Continue => {
                            // Validate continue is inside a loop
                            if !self.scope_manager.can_break_continue() {
                                return Err("Error: Continue statement outside of loop.".to_string());
                            }
                            
                            // TODO: Implement unreachable code detection after continue
                            // Any statements after continue in the same block should be flagged as unreachable
                        }
                    }
                }
                AstNode::Expression(_) => {
                    // Top-level expressions are not handled yet
                    return Err("Error: Top-level expressions not supported yet.".to_string());
                }
            }
        }
        Ok(("Semantic analysis completed successfully. No borrow checker violations detected.".to_string(), ast))
    }

    fn check_expression_initialization(&self, expr: &Expression) -> Result<(), String> {
        match expr {
            Expression::Number(_) | Expression::Float(_) => Ok(()),
            Expression::Identifier(name) => {
                if let Some(var_info) = self.symbol_table.get(name) {
                    if !var_info.initialized {
                        Err(format!("Error: Use of uninitialized variable `{}`.", name))
                    } else {
                        Ok(())
                    }
                } else {
                    Err(format!("Error: Use of undeclared variable `{}`.", name))
                }
            }
            Expression::Binary { lhs, rhs, .. } => {
                self.check_expression_initialization(lhs)?;
                self.check_expression_initialization(rhs)?;
                Ok(())
            }
            Expression::FunctionCall { arguments, .. } => {
                // Check initialization of all function call arguments
                for arg in arguments {
                    self.check_expression_initialization(arg)?;
                }
                Ok(())
            }
            Expression::Print { arguments, .. } => {
                // Check initialization of all print arguments
                for arg in arguments {
                    self.check_expression_initialization(arg)?;
                }
                Ok(())
            }
            Expression::Println { arguments, .. } => {
                // Check initialization of all println arguments
                for arg in arguments {
                    self.check_expression_initialization(arg)?;
                }
                Ok(())
            }
            Expression::Comparison { left, right, .. } => {
                self.check_expression_initialization(left)?;
                self.check_expression_initialization(right)?;
                Ok(())
            }
            Expression::Logical { left, right, .. } => {
                self.check_expression_initialization(left)?;
                self.check_expression_initialization(right)?;
                Ok(())
            }
            Expression::Unary { operand, .. } => {
                self.check_expression_initialization(operand)?;
                Ok(())
            }
        }
    }

    fn infer_and_validate_expression(&self, expr: &mut Expression) -> Result<Ty, String> {
        match expr {
            Expression::Number(_) => Ok(Ty::Int),
            Expression::Float(_) => Ok(Ty::Float),
            Expression::Identifier(name) => {
                if let Some(var_info) = self.symbol_table.get(name) {
                    if !var_info.initialized {
                        Err(format!("Error: Use of uninitialized variable `{}`.", name))
                    } else {
                        Ok(var_info.ty.clone())
                    }
                } else {
                    Err(format!("Error: Use of undeclared variable `{}`.", name))
                }
            }
            Expression::Binary { op, lhs, rhs, ty } => {
                let lhs_type = self.infer_and_validate_expression(lhs)?;
                let rhs_type = self.infer_and_validate_expression(rhs)?;

                // Infer the result type using type promotion rules
                let result_type = infer_binary_type(op, &lhs_type, &rhs_type)?;
                
                // Store the result type in the AST node
                *ty = Some(result_type.clone());
                
                Ok(result_type)
            }
            Expression::FunctionCall { .. } => {
                // TODO: Implement function call type inference
                // This will be implemented in task 6.1
                // For now, assume function calls return int
                Ok(Ty::Int)
            }
            Expression::Print { format_string, arguments } => {
                // Validate format string and arguments
                self.validate_format_string_and_args(format_string, arguments)?;
                
                // Validate all arguments
                for arg in arguments {
                    self.infer_and_validate_expression(arg)?;
                }
                // Print expressions don't return a value (unit type)
                Ok(Ty::Int) // Placeholder - should be unit type when unit type is implemented
            }
            Expression::Println { format_string, arguments } => {
                // Validate format string and arguments
                self.validate_format_string_and_args(format_string, arguments)?;
                
                // Validate all arguments
                for arg in arguments {
                    self.infer_and_validate_expression(arg)?;
                }
                // Println expressions don't return a value (unit type)
                Ok(Ty::Int) // Placeholder - should be unit type when unit type is implemented
            }
            Expression::Comparison { op, left, right } => {
                let left_type = self.infer_and_validate_expression(left)?;
                let right_type = self.infer_and_validate_expression(right)?;
                
                // Validate that comparison operands are compatible
                self.validate_comparison_operands(op, &left_type, &right_type)?;
                
                // Comparison operations always return boolean
                Ok(Ty::Bool)
            }
            Expression::Logical { op, left, right } => {
                let left_type = self.infer_and_validate_expression(left)?;
                let right_type = self.infer_and_validate_expression(right)?;
                
                // Validate that logical operands are boolean
                self.validate_logical_operands(op, &left_type, &right_type)?;
                
                // Logical operations return boolean
                Ok(Ty::Bool)
            }
            Expression::Unary { operand, op } => {
                let operand_type = self.infer_and_validate_expression(operand)?;
                
                // Validate unary operation and return appropriate type
                self.validate_unary_operation(op, &operand_type)
            }
        }
    }

    pub fn get_var_info(&self, name: &str) -> Option<&VarInfo> {
        self.symbol_table.get(name)
    }

    pub fn get_function_table(&self) -> &FunctionTable {
        &self.function_table
    }

    pub fn get_function_table_mut(&mut self) -> &mut FunctionTable {
        &mut self.function_table
    }

    pub fn define_function(&mut self, name: String, parameters: Vec<Parameter>, return_type: Option<Type>) -> Result<(), String> {
        let return_ty = if let Some(ret_type) = return_type {
            self.function_table.type_from_ast_type(&ret_type)?
        } else {
            Ty::Int // Default return type - should be unit type in the future
        };

        let func_info = FunctionInfo {
            name: name.clone(),
            parameters,
            return_type: return_ty,
            defined_at: None, // TODO: Add source location tracking
        };

        self.function_table.define_function(func_info)
    }

    pub fn validate_function_call(&self, name: &str, arg_types: &[Ty]) -> Result<Ty, String> {
        self.function_table.validate_call(name, arg_types)
    }

    // Keep for backward compatibility during transition
    fn fresh_ptr_name(&mut self) -> String {
        self.scope_manager.fresh_ptr_name()
    }

    /// Validate format string and arguments for print/println macros
    fn validate_format_string_and_args(&self, format_string: &str, arguments: &[Expression]) -> Result<(), String> {
        // Count format placeholders in the format string
        let placeholder_count = format_string.matches("{}").count();
        
        // Check that the number of placeholders matches the number of arguments
        if placeholder_count != arguments.len() {
            return Err(format!(
                "Error: Format string has {} placeholders but {} arguments were provided.",
                placeholder_count,
                arguments.len()
            ));
        }

        // Validate that all arguments are of printable types
        for (i, arg) in arguments.iter().enumerate() {
            let arg_type = self.infer_and_validate_expression_immutable(arg)?;
            if !self.is_printable_type(&arg_type) {
                return Err(format!(
                    "Error: Argument {} of type `{}` is not printable.",
                    i + 1,
                    arg_type.to_string()
                ));
            }
        }

        Ok(())
    }

    /// Check if a type can be printed
    fn is_printable_type(&self, ty: &Ty) -> bool {
        match ty {
            Ty::Int | Ty::Float | Ty::Bool => true,
            // Add more types as they are implemented
        }
    }

    /// Validate comparison operands
    fn validate_comparison_operands(&self, op: &crate::ast::ComparisonOp, left_type: &Ty, right_type: &Ty) -> Result<(), String> {
        use crate::ast::ComparisonOp;
        
        match (left_type, right_type) {
            // Same types are always comparable
            (Ty::Int, Ty::Int) | (Ty::Float, Ty::Float) => Ok(()),
            
            // Int and Float can be compared (with promotion)
            (Ty::Int, Ty::Float) | (Ty::Float, Ty::Int) => Ok(()),
            
            // Bool can only be compared with == and != and only with other bools
            (Ty::Bool, Ty::Bool) => {
                match op {
                    ComparisonOp::Equal | ComparisonOp::NotEqual => Ok(()),
                    _ => Err(format!(
                        "Error: Boolean values can only be compared using `==` or `!=`, not `{:?}`.",
                        op
                    )),
                }
            }
            
            // Bool cannot be compared with other types
            (Ty::Bool, _) | (_, Ty::Bool) => {
                Err(format!(
                    "Error: Cannot compare `{}` with `{}` using `{:?}`.",
                    left_type.to_string(),
                    right_type.to_string(),
                    op
                ))
            }
        }
    }

    /// Validate logical operands
    fn validate_logical_operands(&self, op: &crate::ast::LogicalOp, left_type: &Ty, right_type: &Ty) -> Result<(), String> {
        use crate::ast::LogicalOp;
        
        // Logical operations require both operands to be boolean
        if *left_type != Ty::Bool {
            return Err(format!(
                "Error: Left operand of `{:?}` must be boolean, found `{}`.",
                op,
                left_type.to_string()
            ));
        }
        
        if *right_type != Ty::Bool {
            return Err(format!(
                "Error: Right operand of `{:?}` must be boolean, found `{}`.",
                op,
                right_type.to_string()
            ));
        }
        
        Ok(())
    }

    /// Validate unary operations
    fn validate_unary_operation(&self, op: &crate::ast::UnaryOp, operand_type: &Ty) -> Result<Ty, String> {
        use crate::ast::UnaryOp;
        
        match op {
            UnaryOp::Not => {
                // Logical not requires boolean operand
                if *operand_type != Ty::Bool {
                    return Err(format!(
                        "Error: Logical not `!` requires boolean operand, found `{}`.",
                        operand_type.to_string()
                    ));
                }
                Ok(Ty::Bool)
            }
            UnaryOp::Minus => {
                // Unary minus requires numeric operand
                match operand_type {
                    Ty::Int | Ty::Float => Ok(operand_type.clone()),
                    _ => Err(format!(
                        "Error: Unary minus `-` requires numeric operand, found `{}`.",
                        operand_type.to_string()
                    )),
                }
            }
        }
    }

    /// Check mutability for variable assignments (for future use)
    pub fn check_variable_mutability(&self, name: &str) -> Result<(), String> {
        if let Some(var_info) = self.scope_manager.get_variable(name) {
            if !var_info.mutable {
                return Err(format!(
                    "Error: Cannot assign to immutable variable `{}`.",
                    name
                ));
            }
            Ok(())
        } else {
            Err(format!("Error: Variable `{}` not found.", name))
        }
    }

    /// Analyze a block of statements
    fn analyze_block(&mut self, block: &Block) -> Result<(), String> {
        // Analyze all statements in the block
        for stmt in &block.statements {
            self.analyze_statement(stmt)?;
        }

        // Analyze the optional expression at the end of the block
        if let Some(expr) = &block.expression {
            self.check_expression_initialization(expr)?;
            self.infer_and_validate_expression_immutable(expr)?;
        }

        Ok(())
    }

    /// Analyze a single statement
    fn analyze_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::Let { name, value } => {
                // Check if variable already exists in current scope
                if self.scope_manager.variable_exists_in_current_scope(name) {
                    return Err(format!("Error: Variable `{}` is already defined in this scope.", name));
                }

                // Infer and validate the expression type
                let inferred_type = self.infer_and_validate_expression_immutable(value)?;
                
                // Add variable to current scope
                self.scope_manager.define_variable(
                    name.clone(),
                    inferred_type.clone(),
                    false, // TODO: Handle mutability when `mut` is implemented
                    true,  // Let statements always initialize
                )?;

                println!("Declared variable: {} with type {}", name, inferred_type.to_string());
                Ok(())
            }
            Statement::Return(expr) => {
                // Check if we're in a function
                if !self.scope_manager.is_in_function() {
                    return Err("Error: Return statement outside of function.".to_string());
                }

                // Validate the return expression
                self.check_expression_initialization(expr)?;
                let return_type = self.infer_and_validate_expression_immutable(expr)?;

                // TODO: Validate that return type matches function's declared return type
                println!("Return statement with expression of type: {}", return_type.to_string());
                Ok(())
            }
            Statement::Function { .. } => {
                // TODO: Function definition analysis will be implemented in task 6.1
                println!("Function definition found - semantic analysis not yet implemented");
                Ok(())
            }
            Statement::If { condition, then_block, else_block } => {
                // Validate condition expression
                self.check_expression_initialization(condition)?;
                let condition_type = self.infer_and_validate_expression_immutable(condition)?;
                
                // Condition must be boolean
                if condition_type != Ty::Bool {
                    return Err(format!("Error: If condition must be boolean, found: {}", condition_type.to_string()));
                }

                // Analyze then block in its own scope
                self.scope_manager.enter_scope();
                self.analyze_block(then_block)?;
                self.scope_manager.exit_scope();

                // Analyze else block if present
                if let Some(else_stmt) = else_block {
                    self.scope_manager.enter_scope();
                    self.analyze_statement(else_stmt)?;
                    self.scope_manager.exit_scope();
                }

                Ok(())
            }
            Statement::While { condition, body } => {
                // Validate condition expression
                self.check_expression_initialization(condition)?;
                let condition_type = self.infer_and_validate_expression_immutable(condition)?;
                
                // Condition must be boolean
                if condition_type != Ty::Bool {
                    return Err(format!("Error: While condition must be boolean, found: {}", condition_type.to_string()));
                }

                // Enter loop context for break/continue validation
                self.scope_manager.enter_loop();
                self.analyze_block(body)?;
                self.scope_manager.exit_loop();

                Ok(())
            }
            Statement::For { variable, iterable, body } => {
                // Validate iterable expression
                self.check_expression_initialization(iterable)?;
                let _iterable_type = self.infer_and_validate_expression_immutable(iterable)?;
                
                // TODO: Add proper range/iterator type checking when implemented
                // For now, accept any type as iterable (placeholder)

                // Enter loop context and define loop variable
                self.scope_manager.enter_loop();
                
                // Define the loop variable in the loop scope
                // TODO: Infer proper type from iterable when range types are implemented
                self.scope_manager.define_variable(
                    variable.clone(),
                    Ty::Int, // Placeholder - should be inferred from iterable
                    false,   // Loop variables are immutable by default
                    true,    // Loop variables are always initialized
                )?;

                self.analyze_block(body)?;
                self.scope_manager.exit_loop();

                Ok(())
            }
            Statement::Loop { body } => {
                // Enter loop context for break/continue validation
                self.scope_manager.enter_loop();
                self.analyze_block(body)?;
                self.scope_manager.exit_loop();

                Ok(())
            }
            Statement::Break => {
                // Validate break is inside a loop
                if !self.scope_manager.can_break_continue() {
                    return Err("Error: Break statement outside of loop.".to_string());
                }
                
                // TODO: Implement unreachable code detection after break
                // Any statements after break in the same block should be flagged as unreachable
                
                Ok(())
            }
            Statement::Continue => {
                // Validate continue is inside a loop
                if !self.scope_manager.can_break_continue() {
                    return Err("Error: Continue statement outside of loop.".to_string());
                }
                
                // TODO: Implement unreachable code detection after continue
                // Any statements after continue in the same block should be flagged as unreachable
                
                Ok(())
            }
        }
    }

    /// Immutable version of infer_and_validate_expression for use in new analysis methods
    fn infer_and_validate_expression_immutable(&self, expr: &Expression) -> Result<Ty, String> {
        match expr {
            Expression::Number(_) => Ok(Ty::Int),
            Expression::Float(_) => Ok(Ty::Float),
            Expression::Identifier(name) => {
                // First check new scope manager
                if let Some(var_info) = self.scope_manager.get_variable(name) {
                    if !var_info.initialized {
                        Err(format!("Error: Use of uninitialized variable `{}`.", name))
                    } else {
                        Ok(var_info.var_type.clone())
                    }
                }
                // Fall back to old symbol table for backward compatibility
                else if let Some(var_info) = self.symbol_table.get(name) {
                    if !var_info.initialized {
                        Err(format!("Error: Use of uninitialized variable `{}`.", name))
                    } else {
                        Ok(var_info.ty.clone())
                    }
                } else {
                    Err(format!("Error: Use of undeclared variable `{}`.", name))
                }
            }
            Expression::Binary { op, lhs, rhs, .. } => {
                let lhs_type = self.infer_and_validate_expression_immutable(lhs)?;
                let rhs_type = self.infer_and_validate_expression_immutable(rhs)?;

                // Infer the result type using type promotion rules
                infer_binary_type(op, &lhs_type, &rhs_type)
            }
            Expression::FunctionCall { arguments, .. } => {
                // TODO: Implement function call validation in task 6.1
                // Validate all arguments
                for arg in arguments {
                    self.infer_and_validate_expression_immutable(arg)?;
                }
                // For now, assume function calls return int
                Ok(Ty::Int)
            }
            Expression::Print { format_string, arguments } => {
                // Validate format string and arguments
                self.validate_format_string_and_args(format_string, arguments)?;
                
                // Validate all arguments
                for arg in arguments {
                    self.infer_and_validate_expression_immutable(arg)?;
                }
                // Print expressions don't return a value (unit type)
                Ok(Ty::Int) // Placeholder - should be unit type when unit type is implemented
            }
            Expression::Println { format_string, arguments } => {
                // Validate format string and arguments
                self.validate_format_string_and_args(format_string, arguments)?;
                
                // Validate all arguments
                for arg in arguments {
                    self.infer_and_validate_expression_immutable(arg)?;
                }
                // Println expressions don't return a value (unit type)
                Ok(Ty::Int) // Placeholder - should be unit type when unit type is implemented
            }
            Expression::Comparison { op, left, right } => {
                let left_type = self.infer_and_validate_expression_immutable(left)?;
                let right_type = self.infer_and_validate_expression_immutable(right)?;
                
                // Validate that comparison operands are compatible
                self.validate_comparison_operands(op, &left_type, &right_type)?;
                
                // Comparison operations always return boolean
                Ok(Ty::Bool)
            }
            Expression::Logical { op, left, right } => {
                let left_type = self.infer_and_validate_expression_immutable(left)?;
                let right_type = self.infer_and_validate_expression_immutable(right)?;
                
                // Validate that logical operands are boolean
                self.validate_logical_operands(op, &left_type, &right_type)?;
                
                // Logical operations return boolean
                Ok(Ty::Bool)
            }
            Expression::Unary { operand, op } => {
                let operand_type = self.infer_and_validate_expression_immutable(operand)?;
                
                // Validate unary operation and return appropriate type
                self.validate_unary_operation(op, &operand_type)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Parameter, Type};

    #[test]
    fn test_function_table_creation() {
        let table = FunctionTable::new();
        assert_eq!(table.list_functions().len(), 0);
    }

    #[test]
    fn test_function_definition() {
        let mut table = FunctionTable::new();
        
        let param1 = Parameter {
            name: "a".to_string(),
            param_type: Type { name: "i32".to_string() },
        };
        let param2 = Parameter {
            name: "b".to_string(),
            param_type: Type { name: "i32".to_string() },
        };

        let func_info = FunctionInfo {
            name: "add".to_string(),
            parameters: vec![param1, param2],
            return_type: Ty::Int,
            defined_at: None,
        };

        let result = table.define_function(func_info);
        assert!(result.is_ok());
        assert_eq!(table.list_functions().len(), 1);
        assert!(table.list_functions().contains(&&"add".to_string()));
    }

    #[test]
    fn test_function_redefinition_error() {
        let mut table = FunctionTable::new();
        
        let func_info1 = FunctionInfo {
            name: "test".to_string(),
            parameters: vec![],
            return_type: Ty::Int,
            defined_at: None,
        };

        let func_info2 = FunctionInfo {
            name: "test".to_string(),
            parameters: vec![],
            return_type: Ty::Float,
            defined_at: None,
        };

        assert!(table.define_function(func_info1).is_ok());
        let result = table.define_function(func_info2);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already defined"));
    }

    #[test]
    fn test_function_lookup() {
        let mut table = FunctionTable::new();
        
        let param = Parameter {
            name: "x".to_string(),
            param_type: Type { name: "f64".to_string() },
        };

        let func_info = FunctionInfo {
            name: "sqrt".to_string(),
            parameters: vec![param],
            return_type: Ty::Float,
            defined_at: None,
        };

        table.define_function(func_info).unwrap();

        let found = table.get_function("sqrt");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "sqrt");
        assert_eq!(found.unwrap().parameters.len(), 1);
        assert_eq!(found.unwrap().return_type, Ty::Float);

        let not_found = table.get_function("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_function_call_validation_success() {
        let mut table = FunctionTable::new();
        
        let param1 = Parameter {
            name: "a".to_string(),
            param_type: Type { name: "i32".to_string() },
        };
        let param2 = Parameter {
            name: "b".to_string(),
            param_type: Type { name: "i32".to_string() },
        };

        let func_info = FunctionInfo {
            name: "add".to_string(),
            parameters: vec![param1, param2],
            return_type: Ty::Int,
            defined_at: None,
        };

        table.define_function(func_info).unwrap();

        let result = table.validate_call("add", &[Ty::Int, Ty::Int]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Ty::Int);
    }

    #[test]
    fn test_function_call_validation_arity_error() {
        let mut table = FunctionTable::new();
        
        let func_info = FunctionInfo {
            name: "test".to_string(),
            parameters: vec![],
            return_type: Ty::Int,
            defined_at: None,
        };

        table.define_function(func_info).unwrap();

        let result = table.validate_call("test", &[Ty::Int]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expects 0 arguments"));
    }

    #[test]
    fn test_function_call_validation_type_error() {
        let mut table = FunctionTable::new();
        
        let param = Parameter {
            name: "x".to_string(),
            param_type: Type { name: "i32".to_string() },
        };

        let func_info = FunctionInfo {
            name: "test".to_string(),
            parameters: vec![param],
            return_type: Ty::Int,
            defined_at: None,
        };

        table.define_function(func_info).unwrap();

        let result = table.validate_call("test", &[Ty::Float]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expects type"));
    }

    // ScopeManager Tests
    #[test]
    fn test_scope_manager_creation() {
        let scope_manager = ScopeManager::new();
        assert_eq!(scope_manager.get_scope_level(), 0);
        assert_eq!(scope_manager.get_loop_depth(), 0);
        assert!(!scope_manager.is_in_function());
        assert!(scope_manager.can_break_continue() == false);
    }

    #[test]
    fn test_scope_manager_variable_definition() {
        let mut scope_manager = ScopeManager::new();
        
        let result = scope_manager.define_variable(
            "x".to_string(),
            Ty::Int,
            false,
            true
        );
        
        assert!(result.is_ok());
        let ptr_name = result.unwrap();
        assert!(ptr_name.starts_with("ptr"));
        
        let var_info = scope_manager.get_variable("x");
        assert!(var_info.is_some());
        assert_eq!(var_info.unwrap().name, "x");
        assert_eq!(var_info.unwrap().var_type, Ty::Int);
        assert!(!var_info.unwrap().mutable);
        assert!(var_info.unwrap().initialized);
        assert_eq!(var_info.unwrap().scope_level, 0);
    }

    #[test]
    fn test_scope_manager_variable_redefinition_error() {
        let mut scope_manager = ScopeManager::new();
        
        scope_manager.define_variable("x".to_string(), Ty::Int, false, true).unwrap();
        
        let result = scope_manager.define_variable("x".to_string(), Ty::Float, true, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already defined in this scope"));
    }

    #[test]
    fn test_scope_manager_nested_scopes() {
        let mut scope_manager = ScopeManager::new();
        
        // Define variable in global scope
        scope_manager.define_variable("x".to_string(), Ty::Int, false, true).unwrap();
        assert_eq!(scope_manager.get_scope_level(), 0);
        
        // Enter new scope
        scope_manager.enter_scope();
        assert_eq!(scope_manager.get_scope_level(), 1);
        
        // Variable from outer scope should be accessible
        let var_info = scope_manager.get_variable("x");
        assert!(var_info.is_some());
        assert_eq!(var_info.unwrap().scope_level, 0);
        
        // Define new variable in inner scope
        scope_manager.define_variable("y".to_string(), Ty::Float, true, true).unwrap();
        
        let y_info = scope_manager.get_variable("y");
        assert!(y_info.is_some());
        assert_eq!(y_info.unwrap().scope_level, 1);
        
        // Exit scope
        scope_manager.exit_scope();
        assert_eq!(scope_manager.get_scope_level(), 0);
        
        // Variable from inner scope should no longer be accessible
        let y_info_after = scope_manager.get_variable("y");
        assert!(y_info_after.is_none());
        
        // Variable from outer scope should still be accessible
        let x_info_after = scope_manager.get_variable("x");
        assert!(x_info_after.is_some());
    }

    #[test]
    fn test_scope_manager_variable_shadowing() {
        let mut scope_manager = ScopeManager::new();
        
        // Define variable in global scope
        scope_manager.define_variable("x".to_string(), Ty::Int, false, true).unwrap();
        
        // Enter new scope
        scope_manager.enter_scope();
        
        // Shadow the variable with same name but different type
        scope_manager.define_variable("x".to_string(), Ty::Float, true, false).unwrap();
        
        // Should find the shadowed variable (inner scope)
        let var_info = scope_manager.get_variable("x");
        assert!(var_info.is_some());
        assert_eq!(var_info.unwrap().var_type, Ty::Float);
        assert!(var_info.unwrap().mutable);
        assert!(!var_info.unwrap().initialized);
        assert_eq!(var_info.unwrap().scope_level, 1);
        
        // Check that shadowing is detected
        assert!(scope_manager.is_shadowing("x"));
        
        // Exit scope
        scope_manager.exit_scope();
        
        // Should find the original variable again
        let var_info_after = scope_manager.get_variable("x");
        assert!(var_info_after.is_some());
        assert_eq!(var_info_after.unwrap().var_type, Ty::Int);
        assert!(!var_info_after.unwrap().mutable);
        assert!(var_info_after.unwrap().initialized);
        assert_eq!(var_info_after.unwrap().scope_level, 0);
        
        // Shadowing should no longer be detected
        assert!(!scope_manager.is_shadowing("x"));
    }

    #[test]
    fn test_scope_manager_function_scope() {
        let mut scope_manager = ScopeManager::new();
        
        assert!(!scope_manager.is_in_function());
        assert!(scope_manager.get_current_function().is_none());
        
        // Enter function
        scope_manager.enter_function("test_func".to_string());
        assert!(scope_manager.is_in_function());
        assert_eq!(scope_manager.get_current_function().unwrap(), "test_func");
        assert_eq!(scope_manager.get_scope_level(), 1); // Function creates new scope
        
        // Define variable in function scope
        scope_manager.define_variable("param".to_string(), Ty::Int, false, true).unwrap();
        
        let var_info = scope_manager.get_variable("param");
        assert!(var_info.is_some());
        assert_eq!(var_info.unwrap().scope_level, 1);
        
        // Exit function
        scope_manager.exit_function();
        assert!(!scope_manager.is_in_function());
        assert!(scope_manager.get_current_function().is_none());
        assert_eq!(scope_manager.get_scope_level(), 0);
        
        // Function variable should no longer be accessible
        let var_info_after = scope_manager.get_variable("param");
        assert!(var_info_after.is_none());
    }

    #[test]
    fn test_scope_manager_loop_context() {
        let mut scope_manager = ScopeManager::new();
        
        assert_eq!(scope_manager.get_loop_depth(), 0);
        assert!(!scope_manager.can_break_continue());
        
        // Enter loop
        scope_manager.enter_loop();
        assert_eq!(scope_manager.get_loop_depth(), 1);
        assert!(scope_manager.can_break_continue());
        assert_eq!(scope_manager.get_scope_level(), 1); // Loop creates new scope
        
        // Enter nested loop
        scope_manager.enter_loop();
        assert_eq!(scope_manager.get_loop_depth(), 2);
        assert!(scope_manager.can_break_continue());
        assert_eq!(scope_manager.get_scope_level(), 2);
        
        // Exit inner loop
        scope_manager.exit_loop();
        assert_eq!(scope_manager.get_loop_depth(), 1);
        assert!(scope_manager.can_break_continue());
        assert_eq!(scope_manager.get_scope_level(), 1);
        
        // Exit outer loop
        scope_manager.exit_loop();
        assert_eq!(scope_manager.get_loop_depth(), 0);
        assert!(!scope_manager.can_break_continue());
        assert_eq!(scope_manager.get_scope_level(), 0);
    }

    #[test]
    fn test_scope_manager_mutability_checking() {
        let mut scope_manager = ScopeManager::new();
        
        // Define mutable variable
        scope_manager.define_variable("mut_var".to_string(), Ty::Int, true, true).unwrap();
        
        // Define immutable variable
        scope_manager.define_variable("immut_var".to_string(), Ty::Int, false, true).unwrap();
        
        // Check mutability
        let mut_result = scope_manager.check_mutability("mut_var");
        assert!(mut_result.is_ok());
        assert!(mut_result.unwrap());
        
        let immut_result = scope_manager.check_mutability("immut_var");
        assert!(immut_result.is_ok());
        assert!(!immut_result.unwrap());
        
        // Check non-existent variable
        let nonexistent_result = scope_manager.check_mutability("nonexistent");
        assert!(nonexistent_result.is_err());
        assert!(nonexistent_result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_scope_manager_initialization_tracking() {
        let mut scope_manager = ScopeManager::new();
        
        // Define uninitialized variable
        scope_manager.define_variable("uninit_var".to_string(), Ty::Int, true, false).unwrap();
        
        let var_info = scope_manager.get_variable("uninit_var");
        assert!(var_info.is_some());
        assert!(!var_info.unwrap().initialized);
        
        // Update initialization status
        let update_result = scope_manager.update_variable_initialization("uninit_var", true);
        assert!(update_result.is_ok());
        
        let var_info_after = scope_manager.get_variable("uninit_var");
        assert!(var_info_after.is_some());
        assert!(var_info_after.unwrap().initialized);
        
        // Try to update non-existent variable
        let nonexistent_update = scope_manager.update_variable_initialization("nonexistent", true);
        assert!(nonexistent_update.is_err());
        assert!(nonexistent_update.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_scope_manager_complex_nesting() {
        let mut scope_manager = ScopeManager::new();
        
        // Global scope
        scope_manager.define_variable("global_var".to_string(), Ty::Int, false, true).unwrap();
        
        // Enter function
        scope_manager.enter_function("main".to_string());
        scope_manager.define_variable("func_var".to_string(), Ty::Float, false, true).unwrap();
        
        // Enter loop within function
        scope_manager.enter_loop();
        scope_manager.define_variable("loop_var".to_string(), Ty::Bool, true, true).unwrap();
        
        // All variables should be accessible
        assert!(scope_manager.get_variable("global_var").is_some());
        assert!(scope_manager.get_variable("func_var").is_some());
        assert!(scope_manager.get_variable("loop_var").is_some());
        
        assert!(scope_manager.is_in_function());
        assert!(scope_manager.can_break_continue());
        assert_eq!(scope_manager.get_scope_level(), 2);
        assert_eq!(scope_manager.get_loop_depth(), 1);
        
        // Exit loop
        scope_manager.exit_loop();
        assert!(scope_manager.get_variable("global_var").is_some());
        assert!(scope_manager.get_variable("func_var").is_some());
        assert!(scope_manager.get_variable("loop_var").is_none()); // Loop variable gone
        
        assert!(scope_manager.is_in_function());
        assert!(!scope_manager.can_break_continue());
        assert_eq!(scope_manager.get_scope_level(), 1);
        assert_eq!(scope_manager.get_loop_depth(), 0);
        
        // Exit function
        scope_manager.exit_function();
        assert!(scope_manager.get_variable("global_var").is_some());
        assert!(scope_manager.get_variable("func_var").is_none()); // Function variable gone
        assert!(scope_manager.get_variable("loop_var").is_none());
        
        assert!(!scope_manager.is_in_function());
        assert!(!scope_manager.can_break_continue());
        assert_eq!(scope_manager.get_scope_level(), 0);
        assert_eq!(scope_manager.get_loop_depth(), 0);
    }

    #[test]
    fn test_scope_manager_debugging_methods() {
        let mut scope_manager = ScopeManager::new();
        
        // Define variables in global scope
        scope_manager.define_variable("x".to_string(), Ty::Int, false, true).unwrap();
        scope_manager.define_variable("y".to_string(), Ty::Float, true, false).unwrap();
        
        // Check current scope variables
        let current_vars = scope_manager.get_current_scope_variables();
        assert_eq!(current_vars.len(), 2);
        assert!(current_vars.contains(&&"x".to_string()));
        assert!(current_vars.contains(&&"y".to_string()));
        
        // Check all variables
        let all_vars = scope_manager.get_all_variables();
        assert_eq!(all_vars.len(), 2);
        
        // Enter new scope and add variable
        scope_manager.enter_scope();
        scope_manager.define_variable("z".to_string(), Ty::Bool, false, true).unwrap();
        
        // Current scope should only have new variable
        let current_vars_inner = scope_manager.get_current_scope_variables();
        assert_eq!(current_vars_inner.len(), 1);
        assert!(current_vars_inner.contains(&&"z".to_string()));
        
        // All variables should include all scopes
        let all_vars_inner = scope_manager.get_all_variables();
        assert_eq!(all_vars_inner.len(), 3);
    }

    // Tests for I/O and Enhanced Type Validation (Task 6.3)

    #[test]
    fn test_format_string_validation_success() {
        let analyzer = SemanticAnalyzer::new();
        
        // Test valid format string with matching arguments
        let args = vec![
            Expression::Number(42),
            Expression::Float(3.14),
        ];
        
        let result = analyzer.validate_format_string_and_args("Value: {}, Pi: {}", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_string_validation_no_placeholders() {
        let analyzer = SemanticAnalyzer::new();
        
        // Test format string with no placeholders and no arguments
        let args = vec![];
        let result = analyzer.validate_format_string_and_args("Hello, World!", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_string_validation_mismatch_too_many_args() {
        let analyzer = SemanticAnalyzer::new();
        
        // Test format string with fewer placeholders than arguments
        let args = vec![
            Expression::Number(42),
            Expression::Number(24),
        ];
        
        let result = analyzer.validate_format_string_and_args("Value: {}", &args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("1 placeholders but 2 arguments"));
    }

    #[test]
    fn test_format_string_validation_mismatch_too_few_args() {
        let analyzer = SemanticAnalyzer::new();
        
        // Test format string with more placeholders than arguments
        let args = vec![Expression::Number(42)];
        
        let result = analyzer.validate_format_string_and_args("Value: {}, Other: {}", &args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("2 placeholders but 1 arguments"));
    }

    #[test]
    fn test_printable_type_validation() {
        let analyzer = SemanticAnalyzer::new();
        
        // Test that basic types are printable
        assert!(analyzer.is_printable_type(&Ty::Int));
        assert!(analyzer.is_printable_type(&Ty::Float));
        assert!(analyzer.is_printable_type(&Ty::Bool));
    }

    #[test]
    fn test_comparison_operands_validation_success() {
        let analyzer = SemanticAnalyzer::new();
        
        // Test valid comparisons
        assert!(analyzer.validate_comparison_operands(&crate::ast::ComparisonOp::Equal, &Ty::Int, &Ty::Int).is_ok());
        assert!(analyzer.validate_comparison_operands(&crate::ast::ComparisonOp::LessThan, &Ty::Float, &Ty::Float).is_ok());
        assert!(analyzer.validate_comparison_operands(&crate::ast::ComparisonOp::Equal, &Ty::Bool, &Ty::Bool).is_ok());
        
        // Test int/float promotion
        assert!(analyzer.validate_comparison_operands(&crate::ast::ComparisonOp::GreaterThan, &Ty::Int, &Ty::Float).is_ok());
        assert!(analyzer.validate_comparison_operands(&crate::ast::ComparisonOp::LessEqual, &Ty::Float, &Ty::Int).is_ok());
    }

    #[test]
    fn test_comparison_operands_validation_bool_restrictions() {
        let analyzer = SemanticAnalyzer::new();
        
        // Test that bool can only use == and !=
        assert!(analyzer.validate_comparison_operands(&crate::ast::ComparisonOp::Equal, &Ty::Bool, &Ty::Bool).is_ok());
        assert!(analyzer.validate_comparison_operands(&crate::ast::ComparisonOp::NotEqual, &Ty::Bool, &Ty::Bool).is_ok());
        
        // Test that bool cannot use ordering operators
        assert!(analyzer.validate_comparison_operands(&crate::ast::ComparisonOp::LessThan, &Ty::Bool, &Ty::Bool).is_err());
        assert!(analyzer.validate_comparison_operands(&crate::ast::ComparisonOp::GreaterThan, &Ty::Bool, &Ty::Bool).is_err());
        assert!(analyzer.validate_comparison_operands(&crate::ast::ComparisonOp::LessEqual, &Ty::Bool, &Ty::Bool).is_err());
        assert!(analyzer.validate_comparison_operands(&crate::ast::ComparisonOp::GreaterEqual, &Ty::Bool, &Ty::Bool).is_err());
    }

    #[test]
    fn test_comparison_operands_validation_type_mismatch() {
        let analyzer = SemanticAnalyzer::new();
        
        // Test invalid type combinations
        let result = analyzer.validate_comparison_operands(&crate::ast::ComparisonOp::Equal, &Ty::Bool, &Ty::Int);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot compare"));
        
        let result2 = analyzer.validate_comparison_operands(&crate::ast::ComparisonOp::LessThan, &Ty::Int, &Ty::Bool);
        assert!(result2.is_err());
        assert!(result2.unwrap_err().contains("Cannot compare"));
    }

    #[test]
    fn test_logical_operands_validation_success() {
        let analyzer = SemanticAnalyzer::new();
        
        // Test valid logical operations
        assert!(analyzer.validate_logical_operands(&crate::ast::LogicalOp::And, &Ty::Bool, &Ty::Bool).is_ok());
        assert!(analyzer.validate_logical_operands(&crate::ast::LogicalOp::Or, &Ty::Bool, &Ty::Bool).is_ok());
    }

    #[test]
    fn test_logical_operands_validation_type_errors() {
        let analyzer = SemanticAnalyzer::new();
        
        // Test invalid left operand
        let result = analyzer.validate_logical_operands(&crate::ast::LogicalOp::And, &Ty::Int, &Ty::Bool);
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("Left operand"));
        assert!(error_msg.contains("must be boolean"));
        
        // Test invalid right operand
        let result2 = analyzer.validate_logical_operands(&crate::ast::LogicalOp::Or, &Ty::Bool, &Ty::Float);
        assert!(result2.is_err());
        let error_msg2 = result2.unwrap_err();
        assert!(error_msg2.contains("Right operand"));
        assert!(error_msg2.contains("must be boolean"));
        
        // Test both operands invalid
        let result3 = analyzer.validate_logical_operands(&crate::ast::LogicalOp::And, &Ty::Int, &Ty::Float);
        assert!(result3.is_err());
        assert!(result3.unwrap_err().contains("Left operand"));
    }

    #[test]
    fn test_unary_operation_validation_logical_not() {
        let analyzer = SemanticAnalyzer::new();
        
        // Test valid logical not
        let result = analyzer.validate_unary_operation(&crate::ast::UnaryOp::Not, &Ty::Bool);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Ty::Bool);
        
        // Test invalid logical not
        let result2 = analyzer.validate_unary_operation(&crate::ast::UnaryOp::Not, &Ty::Int);
        assert!(result2.is_err());
        assert!(result2.unwrap_err().contains("requires boolean operand"));
        
        let result3 = analyzer.validate_unary_operation(&crate::ast::UnaryOp::Not, &Ty::Float);
        assert!(result3.is_err());
        assert!(result3.unwrap_err().contains("requires boolean operand"));
    }

    #[test]
    fn test_unary_operation_validation_minus() {
        let analyzer = SemanticAnalyzer::new();
        
        // Test valid unary minus
        let result = analyzer.validate_unary_operation(&crate::ast::UnaryOp::Minus, &Ty::Int);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Ty::Int);
        
        let result2 = analyzer.validate_unary_operation(&crate::ast::UnaryOp::Minus, &Ty::Float);
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), Ty::Float);
        
        // Test invalid unary minus
        let result3 = analyzer.validate_unary_operation(&crate::ast::UnaryOp::Minus, &Ty::Bool);
        assert!(result3.is_err());
        assert!(result3.unwrap_err().contains("requires numeric operand"));
    }

    #[test]
    fn test_variable_mutability_checking() {
        let mut analyzer = SemanticAnalyzer::new();
        
        // Define mutable variable
        analyzer.scope_manager.define_variable("mut_var".to_string(), Ty::Int, true, true).unwrap();
        
        // Define immutable variable
        analyzer.scope_manager.define_variable("immut_var".to_string(), Ty::Int, false, true).unwrap();
        
        // Test mutability checking
        let result = analyzer.check_variable_mutability("mut_var");
        assert!(result.is_ok());
        
        let result2 = analyzer.check_variable_mutability("immut_var");
        assert!(result2.is_err());
        assert!(result2.unwrap_err().contains("Cannot assign to immutable variable"));
        
        // Test non-existent variable
        let result3 = analyzer.check_variable_mutability("nonexistent");
        assert!(result3.is_err());
        assert!(result3.unwrap_err().contains("not found"));
    }
}


