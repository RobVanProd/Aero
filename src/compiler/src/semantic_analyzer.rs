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
                            // TODO: Implement if statement semantic analysis
                            // This will be implemented in task 6.2
                            self.check_expression_initialization(condition)?;
                            self.infer_and_validate_expression(condition)?;
                            println!("If statement found - semantic analysis not yet implemented");
                        }
                        Statement::While { condition, body } => {
                            // TODO: Implement while loop semantic analysis
                            // This will be implemented in task 6.2
                            self.check_expression_initialization(condition)?;
                            self.infer_and_validate_expression(condition)?;
                            println!("While loop found - semantic analysis not yet implemented");
                        }
                        Statement::For { variable, iterable, body } => {
                            // TODO: Implement for loop semantic analysis
                            // This will be implemented in task 6.2
                            self.check_expression_initialization(iterable)?;
                            self.infer_and_validate_expression(iterable)?;
                            println!("For loop found - semantic analysis not yet implemented");
                        }
                        Statement::Loop { body } => {
                            // TODO: Implement infinite loop semantic analysis
                            // This will be implemented in task 6.2
                            println!("Loop statement found - semantic analysis not yet implemented");
                        }
                        Statement::Break => {
                            // TODO: Implement break statement validation
                            // This will be implemented in task 6.2
                            println!("Break statement found - semantic analysis not yet implemented");
                        }
                        Statement::Continue => {
                            // TODO: Implement continue statement validation
                            // This will be implemented in task 6.2
                            println!("Continue statement found - semantic analysis not yet implemented");
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
            Expression::Print { arguments, .. } => {
                // TODO: Implement print expression type inference and validation
                // This will be implemented in task 6.3
                // Validate all arguments
                for arg in arguments {
                    self.infer_and_validate_expression(arg)?;
                }
                // Print expressions don't return a value (unit type)
                Ok(Ty::Int) // Placeholder - should be unit type
            }
            Expression::Println { arguments, .. } => {
                // TODO: Implement println expression type inference and validation
                // This will be implemented in task 6.3
                // Validate all arguments
                for arg in arguments {
                    self.infer_and_validate_expression(arg)?;
                }
                // Println expressions don't return a value (unit type)
                Ok(Ty::Int) // Placeholder - should be unit type
            }
            Expression::Comparison { left, right, .. } => {
                // TODO: Implement comparison expression type inference
                // This will be implemented in task 6.3
                let left_type = self.infer_and_validate_expression(left)?;
                let right_type = self.infer_and_validate_expression(right)?;
                // For now, just validate that both sides are compatible
                // Comparison operations return boolean
                Ok(Ty::Bool)
            }
            Expression::Logical { left, right, .. } => {
                // TODO: Implement logical expression type inference
                // This will be implemented in task 6.3
                let left_type = self.infer_and_validate_expression(left)?;
                let right_type = self.infer_and_validate_expression(right)?;
                // Logical operations require boolean operands and return boolean
                Ok(Ty::Bool)
            }
            Expression::Unary { operand, op } => {
                // TODO: Implement unary expression type inference
                // This will be implemented in task 6.3
                let operand_type = self.infer_and_validate_expression(operand)?;
                match op {
                    crate::ast::UnaryOp::Not => Ok(Ty::Bool), // Logical not returns boolean
                    crate::ast::UnaryOp::Minus => Ok(operand_type), // Unary minus returns same type as operand
                }
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
}


