use crate::ast::{AstNode, Expression, Statement, Block, Parameter, ComparisonOp, LogicalOp, UnaryOp};
use crate::types::{Ty, infer_binary_type};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub name: String,
    pub ty: Ty,
    pub mutable: bool,
    pub initialized: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Ty,
    pub defined_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VariableInfoNew {
    pub name: String,
    pub var_type: Ty,
    pub mutable: bool,
    pub initialized: bool,
    pub scope_level: u32,
    pub ptr_name: String,
}

pub struct FunctionTable {
    functions: HashMap<String, FunctionInfo>,
}

impl FunctionTable {
    pub fn new() -> Self {
        Self {
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
        if let Some(func) = self.functions.get(name) {
            if func.parameters.len() != args.len() {
                return Err(format!(
                    "Error: Function `{}` expects {} arguments, but {} were provided.",
                    name,
                    func.parameters.len(),
                    args.len()
                ));
            }

            for (i, (param, arg_type)) in func.parameters.iter().zip(args.iter()).enumerate() {
                let expected_type = match &param.param_type {
                    crate::ast::Type::Named(type_name) => match type_name.as_str() {
                        "i32" => Ty::Int,
                        "f64" => Ty::Float,
                        "bool" => Ty::Bool,
                        _ => Ty::Int,
                    }
                };
                
                if expected_type != *arg_type {
                    return Err(format!(
                        "Error: Function `{}` expects type `{}` for argument {}, but `{}` was provided.",
                        name,
                        expected_type.to_string(),
                        i + 1,
                        arg_type.to_string()
                    ));
                }
            }

            Ok(func.return_type.clone())
        } else {
            Err(format!("Error: Function `{}` is not defined.", name))
        }
    }

    pub fn list_functions(&self) -> Vec<&String> {
        self.functions.keys().collect()
    }
}

pub struct ScopeManager {
    scopes: Vec<HashMap<String, VariableInfoNew>>,
    current_function: Option<String>,
    loop_depth: u32,
    next_ptr: u32,
}

impl ScopeManager {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()], // Start with global scope
            current_function: None,
            loop_depth: 0,
            next_ptr: 0,
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
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
        self.loop_depth += 1;
        self.enter_scope(); // Loops create their own scope
    }

    pub fn exit_loop(&mut self) {
        if self.loop_depth > 0 {
            self.loop_depth -= 1;
            self.exit_scope(); // Exit loop scope
        }
    }

    pub fn define_variable(&mut self, name: String, var_type: Ty, mutable: bool, initialized: bool) -> Result<String, String> {
        // Check if variable already exists in current scope
        if let Some(current_scope) = self.scopes.last() {
            if current_scope.contains_key(&name) {
                return Err(format!("Error: Variable `{}` is already defined in this scope.", name));
            }
        }

        // Generate unique pointer name
        let ptr_name = format!("ptr{}", self.next_ptr);
        self.next_ptr += 1;

        let var_info = VariableInfoNew {
            name: name.clone(),
            var_type,
            mutable,
            initialized,
            scope_level: (self.scopes.len() - 1) as u32,
            ptr_name: ptr_name.clone(),
        };

        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name, var_info);
        }

        Ok(ptr_name)
    }

    pub fn get_variable(&self, name: &str) -> Option<&VariableInfoNew> {
        // Search from innermost to outermost scope
        for scope in self.scopes.iter().rev() {
            if let Some(var_info) = scope.get(name) {
                return Some(var_info);
            }
        }
        None
    }

    pub fn variable_exists_in_current_scope(&self, name: &str) -> bool {
        if let Some(current_scope) = self.scopes.last() {
            current_scope.contains_key(name)
        } else {
            false
        }
    }

    pub fn is_in_function(&self) -> bool {
        self.current_function.is_some()
    }

    pub fn get_current_function(&self) -> Option<&String> {
        self.current_function.as_ref()
    }

    pub fn can_break_continue(&self) -> bool {
        self.loop_depth > 0
    }

    pub fn get_scope_level(&self) -> u32 {
        (self.scopes.len() - 1) as u32
    }

    pub fn get_loop_depth(&self) -> u32 {
        self.loop_depth
    }

    pub fn check_mutability(&self, name: &str) -> Result<bool, String> {
        if let Some(var_info) = self.get_variable(name) {
            Ok(var_info.mutable)
        } else {
            Err(format!("Error: Variable `{}` not found.", name))
        }
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

    pub fn is_shadowing(&self, name: &str) -> bool {
        let mut found_count = 0;
        for scope in &self.scopes {
            if scope.contains_key(name) {
                found_count += 1;
                if found_count > 1 {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_current_scope_variables(&self) -> Vec<&String> {
        if let Some(current_scope) = self.scopes.last() {
            current_scope.keys().collect()
        } else {
            vec![]
        }
    }

    pub fn get_all_variables(&self) -> Vec<&VariableInfoNew> {
        let mut all_vars = vec![];
        for scope in &self.scopes {
            for var_info in scope.values() {
                all_vars.push(var_info);
            }
        }
        all_vars
    }
}

pub struct SemanticAnalyzer {
    symbol_table: HashMap<String, VariableInfo>,
    function_table: FunctionTable,
    scope_manager: ScopeManager,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbol_table: HashMap::new(),
            function_table: FunctionTable::new(),
            scope_manager: ScopeManager::new(),
        }
    }

    pub fn analyze(&mut self, ast: Vec<AstNode>) -> Result<(String, Vec<AstNode>), String> {
        for node in &ast {
            match node {
                AstNode::Statement(stmt) => {
                    self.analyze_statement(stmt)?;
                }
                AstNode::Expression(expr) => {
                    self.check_expression_initialization(expr)?;
                    self.infer_and_validate_expression_immutable(expr)?;
                }
            }
        }
        Ok(("Semantic analysis completed successfully".to_string(), ast))
    }

    fn check_expression_initialization(&self, expr: &Expression) -> Result<(), String> {
        match expr {
            Expression::Identifier(name) => {
                if let Some(var_info) = self.scope_manager.get_variable(name) {
                    if !var_info.initialized {
                        return Err(format!("Error: Use of uninitialized variable `{}`.", name));
                    }
                } else if let Some(var_info) = self.symbol_table.get(name) {
                    if !var_info.initialized {
                        return Err(format!("Error: Use of uninitialized variable `{}`.", name));
                    }
                } else {
                    return Err(format!("Error: Use of undeclared variable `{}`.", name));
                }
            }
            Expression::Binary { left, right, .. } => {
                self.check_expression_initialization(left)?;
                self.check_expression_initialization(right)?;
            }
            Expression::FunctionCall { arguments, .. } => {
                for arg in arguments {
                    self.check_expression_initialization(arg)?;
                }
            }
            Expression::Print { arguments, .. } => {
                for arg in arguments {
                    self.check_expression_initialization(arg)?;
                }
            }
            Expression::Println { arguments, .. } => {
                for arg in arguments {
                    self.check_expression_initialization(arg)?;
                }
            }
            Expression::Comparison { left, right, .. } => {
                self.check_expression_initialization(left)?;
                self.check_expression_initialization(right)?;
            }
            Expression::Logical { left, right, .. } => {
                self.check_expression_initialization(left)?;
                self.check_expression_initialization(right)?;
            }
            Expression::Unary { operand, .. } => {
                self.check_expression_initialization(operand)?;
            }
            _ => {} // Literals don't need initialization checks
        }
        Ok(())
    }

    fn infer_and_validate_expression(&self, expr: &mut Expression) -> Result<Ty, String> {
        match expr {
            Expression::IntegerLiteral(_) => Ok(Ty::Int),
            Expression::FloatLiteral(_) => Ok(Ty::Float),
            Expression::Identifier(name) => {
                if let Some(var_info) = self.scope_manager.get_variable(name) {
                    if !var_info.initialized {
                        Err(format!("Error: Use of uninitialized variable `{}`.", name))
                    } else {
                        Ok(var_info.var_type.clone())
                    }
                } else if let Some(var_info) = self.symbol_table.get(name) {
                    if !var_info.initialized {
                        Err(format!("Error: Use of uninitialized variable `{}`.", name))
                    } else {
                        Ok(var_info.ty.clone())
                    }
                } else {
                    Err(format!("Error: Use of undeclared variable `{}`.", name))
                }
            }
            Expression::Binary { op, left, right, .. } => {
                let lhs_type = self.infer_and_validate_expression(left)?;
                let rhs_type = self.infer_and_validate_expression(right)?;
                infer_binary_type(op.as_str(), &lhs_type, &rhs_type)
            }
            Expression::FunctionCall { arguments, .. } => {
                for arg in arguments {
                    self.infer_and_validate_expression(arg)?;
                }
                Ok(Ty::Int)
            }
            Expression::Print { format_string, arguments } => {
                self.validate_format_string_and_args(format_string, arguments)?;
                Ok(Ty::Int)
            }
            Expression::Println { format_string, arguments } => {
                self.validate_format_string_and_args(format_string, arguments)?;
                Ok(Ty::Int)
            }
            Expression::Comparison { op, left, right } => {
                let left_type = self.infer_and_validate_expression(left)?;
                let right_type = self.infer_and_validate_expression(right)?;
                self.validate_comparison_operands(op, &left_type, &right_type)?;
                Ok(Ty::Bool)
            }
            Expression::Logical { op, left, right } => {
                let left_type = self.infer_and_validate_expression(left)?;
                let right_type = self.infer_and_validate_expression(right)?;
                self.validate_logical_operands(op, &left_type, &right_type)?;
                Ok(Ty::Bool)
            }
            Expression::Unary { operand, op } => {
                let operand_type = self.infer_and_validate_expression(operand)?;
                self.validate_unary_operation(op, &operand_type)
            }
        }
    }

    fn infer_and_validate_expression_immutable(&self, expr: &Expression) -> Result<Ty, String> {
        match expr {
            Expression::IntegerLiteral(_) => Ok(Ty::Int),
            Expression::FloatLiteral(_) => Ok(Ty::Float),
            Expression::Identifier(name) => {
                if let Some(var_info) = self.scope_manager.get_variable(name) {
                    if !var_info.initialized {
                        Err(format!("Error: Use of uninitialized variable `{}`.", name))
                    } else {
                        Ok(var_info.var_type.clone())
                    }
                } else if let Some(var_info) = self.symbol_table.get(name) {
                    if !var_info.initialized {
                        Err(format!("Error: Use of uninitialized variable `{}`.", name))
                    } else {
                        Ok(var_info.ty.clone())
                    }
                } else {
                    Err(format!("Error: Use of undeclared variable `{}`.", name))
                }
            }
            Expression::Binary { op, left, right, .. } => {
                let lhs_type = self.infer_and_validate_expression_immutable(left)?;
                let rhs_type = self.infer_and_validate_expression_immutable(right)?;
                infer_binary_type(op.as_str(), &lhs_type, &rhs_type)
            }
            Expression::FunctionCall { arguments, .. } => {
                for arg in arguments {
                    self.infer_and_validate_expression_immutable(arg)?;
                }
                Ok(Ty::Int)
            }
            Expression::Print { format_string, arguments } => {
                self.validate_format_string_and_args_immutable(format_string, arguments)?;
                Ok(Ty::Int)
            }
            Expression::Println { format_string, arguments } => {
                self.validate_format_string_and_args_immutable(format_string, arguments)?;
                Ok(Ty::Int)
            }
            Expression::Comparison { op, left, right } => {
                let left_type = self.infer_and_validate_expression_immutable(left)?;
                let right_type = self.infer_and_validate_expression_immutable(right)?;
                self.validate_comparison_operands(op, &left_type, &right_type)?;
                Ok(Ty::Bool)
            }
            Expression::Logical { op, left, right } => {
                let left_type = self.infer_and_validate_expression_immutable(left)?;
                let right_type = self.infer_and_validate_expression_immutable(right)?;
                self.validate_logical_operands(op, &left_type, &right_type)?;
                Ok(Ty::Bool)
            }
            Expression::Unary { operand, op } => {
                let operand_type = self.infer_and_validate_expression_immutable(operand)?;
                self.validate_unary_operation(op, &operand_type)
            }
        }
    }

    fn validate_format_string_and_args(&self, format_string: &str, arguments: &[Expression]) -> Result<(), String> {
        let placeholder_count = format_string.matches("{}").count();
        
        if placeholder_count != arguments.len() {
            return Err(format!(
                "Error: Format string has {} placeholders but {} arguments were provided.",
                placeholder_count,
                arguments.len()
            ));
        }

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

    fn validate_format_string_and_args_immutable(&self, format_string: &str, arguments: &[Expression]) -> Result<(), String> {
        let placeholder_count = format_string.matches("{}").count();
        
        if placeholder_count != arguments.len() {
            return Err(format!(
                "Error: Format string has {} placeholders but {} arguments were provided.",
                placeholder_count,
                arguments.len()
            ));
        }

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

    fn is_printable_type(&self, ty: &Ty) -> bool {
        matches!(ty, Ty::Int | Ty::Float | Ty::Bool)
    }

    fn validate_comparison_operands(&self, _op: &ComparisonOp, left_type: &Ty, right_type: &Ty) -> Result<(), String> {
        if left_type == right_type {
            Ok(())
        } else if (left_type == &Ty::Int && right_type == &Ty::Float) || 
                  (left_type == &Ty::Float && right_type == &Ty::Int) {
            Ok(()) // Allow int/float comparisons
        } else {
            Err(format!(
                "Error: Cannot compare types `{}` and `{}`.",
                left_type.to_string(),
                right_type.to_string()
            ))
        }
    }

    fn validate_logical_operands(&self, _op: &LogicalOp, left_type: &Ty, right_type: &Ty) -> Result<(), String> {
        if left_type != &Ty::Bool {
            return Err(format!("Error: Left operand of logical operation must be boolean, found: {}", left_type.to_string()));
        }
        if right_type != &Ty::Bool {
            return Err(format!("Error: Right operand of logical operation must be boolean, found: {}", right_type.to_string()));
        }
        Ok(())
    }

    fn validate_unary_operation(&self, op: &UnaryOp, operand_type: &Ty) -> Result<Ty, String> {
        match op {
            UnaryOp::Not => {
                if operand_type == &Ty::Bool {
                    Ok(Ty::Bool)
                } else {
                    Err(format!("Error: Logical NOT operator requires boolean operand, found: {}", operand_type.to_string()))
                }
            }
            UnaryOp::Negate => {
                if operand_type == &Ty::Int || operand_type == &Ty::Float {
                    Ok(operand_type.clone())
                } else {
                    Err(format!("Error: Unary minus operator requires numeric operand, found: {}", operand_type.to_string()))
                }
            }
        }
    }

    fn analyze_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::Let { name, mutable: _, type_annotation: _, value } => {
                if self.scope_manager.variable_exists_in_current_scope(name) {
                    return Err(format!("Error: Variable `{}` is already defined in this scope.", name));
                }

                let inferred_type = if let Some(val) = value { 
                    self.infer_and_validate_expression_immutable(val)? 
                } else { 
                    Ty::Int 
                };
                
                self.scope_manager.define_variable(
                    name.clone(),
                    inferred_type.clone(),
                    false,
                    value.is_some(),
                )?;

                // Also add to old symbol table for backward compatibility
                let var_info = VariableInfo {
                    name: name.clone(),
                    ty: inferred_type.clone(),
                    mutable: false,
                    initialized: value.is_some(),
                };
                self.symbol_table.insert(name.clone(), var_info);

                Ok(())
            }
            Statement::Return(expr) => {
                if let Some(val) = expr { 
                    self.check_expression_initialization(val)?; 
                    self.infer_and_validate_expression_immutable(val)?;
                }
                Ok(())
            }
            Statement::Function { .. } => {
                Ok(())
            }
            Statement::If { condition, then_block, else_block } => {
                self.check_expression_initialization(condition)?;
                let condition_type = self.infer_and_validate_expression_immutable(condition)?;
                
                if condition_type != Ty::Bool {
                    return Err(format!("Error: If condition must be boolean, found: {}", condition_type.to_string()));
                }

                self.scope_manager.enter_scope();
                self.analyze_block(then_block)?;
                self.scope_manager.exit_scope();

                if let Some(else_stmt) = else_block {
                    self.scope_manager.enter_scope();
                    self.analyze_statement(else_stmt)?;
                    self.scope_manager.exit_scope();
                }

                Ok(())
            }
            Statement::While { condition, body } => {
                self.check_expression_initialization(condition)?;
                let condition_type = self.infer_and_validate_expression_immutable(condition)?;
                
                if condition_type != Ty::Bool {
                    return Err(format!("Error: While condition must be boolean, found: {}", condition_type.to_string()));
                }

                self.scope_manager.enter_loop();
                self.analyze_block(body)?;
                self.scope_manager.exit_loop();

                Ok(())
            }
            Statement::For { variable, iterable, body } => {
                self.check_expression_initialization(iterable)?;
                let _iterable_type = self.infer_and_validate_expression_immutable(iterable)?;

                self.scope_manager.enter_loop();
                self.scope_manager.define_variable(
                    variable.clone(),
                    Ty::Int,
                    false,
                    true,
                )?;
                self.analyze_block(body)?;
                self.scope_manager.exit_loop();

                Ok(())
            }
            Statement::Loop { body } => {
                self.scope_manager.enter_loop();
                self.analyze_block(body)?;
                self.scope_manager.exit_loop();
                Ok(())
            }
            Statement::Break => {
                if !self.scope_manager.can_break_continue() {
                    return Err("Error: Break statement outside of loop.".to_string());
                }
                Ok(())
            }
            Statement::Continue => {
                if !self.scope_manager.can_break_continue() {
                    return Err("Error: Continue statement outside of loop.".to_string());
                }
                Ok(())
            }
            Statement::Expression(expr) => {
                self.check_expression_initialization(expr)?;
                self.infer_and_validate_expression_immutable(expr)?;
                Ok(())
            }
            Statement::Block(block) => {
                self.scope_manager.enter_scope();
                self.analyze_block(block)?;
                self.scope_manager.exit_scope();
                Ok(())
            }
        }
    }

    fn analyze_block(&mut self, block: &Block) -> Result<(), String> {
        for stmt in &block.statements {
            self.analyze_statement(stmt)?;
        }

        if let Some(expr) = &block.expression {
            self.check_expression_initialization(expr)?;
            self.infer_and_validate_expression_immutable(expr)?;
        }

        Ok(())
    }
}