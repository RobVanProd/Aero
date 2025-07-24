use crate::ast::{AstNode, Expression, Statement, Block, Parameter, ComparisonOp, LogicalOp, UnaryOp, MatchArm, Pattern};
use crate::types::{Ty, infer_binary_type, TypeDefinitionManager};
use crate::pattern_matcher::{PatternMatcher, ExhaustivenessResult};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

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
                    },
                    // TODO: Implement proper type checking for generic and collection types
                    crate::ast::Type::Generic { .. } => Ty::Int, // Placeholder
                    crate::ast::Type::Array { .. } => Ty::Int, // Placeholder
                    crate::ast::Type::Slice { .. } => Ty::Int, // Placeholder
                    crate::ast::Type::Vec { .. } => Ty::Int, // Placeholder
                    crate::ast::Type::HashMap { .. } => Ty::Int, // Placeholder
                    crate::ast::Type::Reference { .. } => Ty::Int, // Placeholder
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
    type_manager: Rc<RefCell<TypeDefinitionManager>>,
    pattern_matcher: PatternMatcher,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let type_manager = Rc::new(RefCell::new(TypeDefinitionManager::new()));
        let pattern_matcher = PatternMatcher::new(type_manager.clone());
        
        Self {
            symbol_table: HashMap::new(),
            function_table: FunctionTable::new(),
            scope_manager: ScopeManager::new(),
            type_manager,
            pattern_matcher,
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
            Expression::Match { expression, arms } => {
                self.check_expression_initialization(expression)?;
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.check_expression_initialization(guard)?;
                    }
                    self.check_expression_initialization(&arm.body)?;
                }
            }
            Expression::StructLiteral { fields, base, .. } => {
                for (_, field_expr) in fields {
                    self.check_expression_initialization(field_expr)?;
                }
                if let Some(base_expr) = base {
                    self.check_expression_initialization(base_expr)?;
                }
            }
            Expression::FieldAccess { object, .. } => {
                self.check_expression_initialization(object)?;
            }
            Expression::MethodCall { object, arguments, .. } => {
                self.check_expression_initialization(object)?;
                for arg in arguments {
                    self.check_expression_initialization(arg)?;
                }
            }
            Expression::ArrayLiteral { elements } => {
                for element in elements {
                    self.check_expression_initialization(element)?;
                }
            }
            Expression::ArrayAccess { array, index } => {
                self.check_expression_initialization(array)?;
                self.check_expression_initialization(index)?;
            }
            Expression::VecMacro { elements } => {
                for element in elements {
                    self.check_expression_initialization(element)?;
                }
            }
            Expression::FormatMacro { arguments, .. } => {
                for arg in arguments {
                    self.check_expression_initialization(arg)?;
                }
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
            Expression::StructLiteral { name, fields, base } => {
                // Validate struct exists
                if self.type_manager.borrow().get_struct(name).is_none() {
                    return Err(format!("Undefined struct type: {}", name));
                }
                
                // Validate field types and collect them
                let mut field_types = Vec::new();
                for (field_name, field_expr) in fields {
                    let field_type = self.infer_and_validate_expression(field_expr)?;
                    field_types.push((field_name.clone(), field_type));
                }
                
                // If there's a base expression, validate it
                if let Some(base_expr) = base {
                    let base_type = self.infer_and_validate_expression(base_expr)?;
                    if base_type != Ty::Struct(name.clone()) {
                        return Err(format!("Base expression in struct literal must be of type {}, found: {}", 
                            name, base_type.to_string()));
                    }
                }
                
                // Validate struct instantiation
                self.type_manager.borrow().validate_struct_instantiation(name, &field_types)?;
                
                Ok(Ty::Struct(name.clone()))
            }
            Expression::FieldAccess { object, field } => {
                let object_type = self.infer_and_validate_expression(object)?;
                
                match object_type {
                    Ty::Struct(struct_name) => {
                        // Validate field access
                        let field_type = self.type_manager.borrow().validate_field_access(&struct_name, field)?;
                        Ok(field_type)
                    }
                    _ => Err(format!("Cannot access field '{}' on non-struct type: {}", 
                        field, object_type.to_string()))
                }
            }
            Expression::Match { expression, arms } => {
                self.analyze_match_expression(expression, arms)
            }
            Expression::MethodCall { object, method, arguments } => {
                let object_type = self.infer_and_validate_expression(object)?;
                
                // Validate arguments
                let mut arg_types = Vec::new();
                for arg in arguments {
                    let arg_type = self.infer_and_validate_expression(arg)?;
                    arg_types.push(arg_type);
                }
                
                match object_type {
                    Ty::Struct(struct_name) => {
                        // Look up method in type manager
                        if let Some(method_def) = self.type_manager.borrow().get_method(&struct_name, method) {
                            // Validate method call arguments
                            self.validate_method_call(&struct_name, method, &arg_types, method_def)?;
                            
                            // Return method return type
                            if let Some(return_type) = &method_def.return_type {
                                self.ast_type_to_ty(return_type)
                            } else {
                                Ok(Ty::Int) // Default return type
                            }
                        } else {
                            Err(format!("Method '{}' not found for struct '{}'", method, struct_name))
                        }
                    }
                    Ty::Enum(enum_name) => {
                        // Look up method in type manager
                        if let Some(method_def) = self.type_manager.borrow().get_method(&enum_name, method) {
                            // Validate method call arguments
                            self.validate_method_call(&enum_name, method, &arg_types, method_def)?;
                            
                            // Return method return type
                            if let Some(return_type) = &method_def.return_type {
                                self.ast_type_to_ty(return_type)
                            } else {
                                Ok(Ty::Int) // Default return type
                            }
                        } else {
                            Err(format!("Method '{}' not found for enum '{}'", method, enum_name))
                        }
                    }
                    Ty::Vec(element_type) => {
                        // Handle Vec method calls
                        self.validate_vec_method_call(method, &arg_types, &element_type)
                    }
                    Ty::Array(element_type, _) => {
                        // Handle array method calls
                        self.validate_array_method_call(method, &arg_types, &element_type)
                    }
                    Ty::String => {
                        // Handle String method calls
                        self.validate_string_method_call(method, &arg_types)
                    }
                    _ => Err(format!("Cannot call method '{}' on type: {}", method, object_type.to_string()))
                }
            }
            Expression::ArrayLiteral { elements } => {
                self.validate_array_literal_mutable(elements)
            }
            Expression::ArrayAccess { array, index } => {
                self.validate_array_access_mutable(array, index)
            }
            Expression::VecMacro { elements } => {
                self.validate_vec_macro_mutable(elements)
            }
            Expression::FormatMacro { format_string, arguments } => {
                self.validate_format_macro_mutable(format_string, arguments)
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
            Expression::StructLiteral { name, fields, base } => {
                // Validate struct exists
                if self.type_manager.borrow().get_struct(name).is_none() {
                    return Err(format!("Undefined struct type: {}", name));
                }
                
                // Validate field types and collect them
                let mut field_types = Vec::new();
                for (field_name, field_expr) in fields {
                    let field_type = self.infer_and_validate_expression_immutable(field_expr)?;
                    field_types.push((field_name.clone(), field_type));
                }
                
                // If there's a base expression, validate it
                if let Some(base_expr) = base {
                    let base_type = self.infer_and_validate_expression_immutable(base_expr)?;
                    if base_type != Ty::Struct(name.clone()) {
                        return Err(format!("Base expression in struct literal must be of type {}, found: {}", 
                            name, base_type.to_string()));
                    }
                }
                
                // Validate struct instantiation
                self.type_manager.borrow().validate_struct_instantiation(name, &field_types)?;
                
                Ok(Ty::Struct(name.clone()))
            }
            Expression::FieldAccess { object, field } => {
                let object_type = self.infer_and_validate_expression_immutable(object)?;
                
                match object_type {
                    Ty::Struct(struct_name) => {
                        // Validate field access
                        let field_type = self.type_manager.borrow().validate_field_access(&struct_name, field)?;
                        Ok(field_type)
                    }
                    _ => Err(format!("Cannot access field '{}' on non-struct type: {}", 
                        field, object_type.to_string()))
                }
            }
            Expression::Match { expression, arms } => {
                self.analyze_match_expression(expression, arms)
            }
            Expression::MethodCall { object, method, arguments } => {
                let object_type = self.infer_and_validate_expression_immutable(object)?;
                
                // Validate arguments
                let mut arg_types = Vec::new();
                for arg in arguments {
                    let arg_type = self.infer_and_validate_expression_immutable(arg)?;
                    arg_types.push(arg_type);
                }
                
                match object_type {
                    Ty::Struct(struct_name) => {
                        // Look up method in type manager
                        if let Some(method_def) = self.type_manager.borrow().get_method(&struct_name, method) {
                            // Validate method call arguments
                            self.validate_method_call(&struct_name, method, &arg_types, method_def)?;
                            
                            // Return method return type
                            if let Some(return_type) = &method_def.return_type {
                                self.ast_type_to_ty(return_type)
                            } else {
                                Ok(Ty::Int) // Default return type
                            }
                        } else {
                            Err(format!("Method '{}' not found for struct '{}'", method, struct_name))
                        }
                    }
                    Ty::Enum(enum_name) => {
                        // Look up method in type manager
                        if let Some(method_def) = self.type_manager.borrow().get_method(&enum_name, method) {
                            // Validate method call arguments
                            self.validate_method_call(&enum_name, method, &arg_types, method_def)?;
                            
                            // Return method return type
                            if let Some(return_type) = &method_def.return_type {
                                self.ast_type_to_ty(return_type)
                            } else {
                                Ok(Ty::Int) // Default return type
                            }
                        } else {
                            Err(format!("Method '{}' not found for enum '{}'", method, enum_name))
                        }
                    }
                    Ty::Vec(element_type) => {
                        // Handle Vec method calls
                        self.validate_vec_method_call(method, &arg_types, &element_type)
                    }
                    Ty::Array(element_type, _) => {
                        // Handle array method calls
                        self.validate_array_method_call(method, &arg_types, &element_type)
                    }
                    Ty::String => {
                        // Handle String method calls
                        self.validate_string_method_call(method, &arg_types)
                    }
                    _ => Err(format!("Cannot call method '{}' on type: {}", method, object_type.to_string()))
                }
            }
            Expression::ArrayLiteral { elements } => {
                self.validate_array_literal(elements)
            }
            Expression::ArrayAccess { array, index } => {
                self.validate_array_access(array, index)
            }
            Expression::VecMacro { elements } => {
                self.validate_vec_macro(elements)
            }
            Expression::FormatMacro { format_string, arguments } => {
                self.validate_format_macro(format_string, arguments)
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
            Statement::Struct { name, generics, fields, is_tuple } => {
                // Create struct definition
                let struct_def = self.type_manager.borrow().create_struct_definition(
                    name.clone(),
                    generics.clone(),
                    fields.clone(),
                    *is_tuple,
                );
                
                // Define the struct in the type manager
                self.type_manager.borrow_mut().define_struct(struct_def)?;
                
                Ok(())
            }
            Statement::Enum { name, generics, variants } => {
                // Create enum definition
                let enum_def = self.type_manager.borrow().create_enum_definition(
                    name.clone(),
                    generics.clone(),
                    variants.clone(),
                );
                
                // Define the enum in the type manager
                self.type_manager.borrow_mut().define_enum(enum_def)?;
                
                Ok(())
            }
            Statement::Impl { generics: _, type_name, trait_name, methods } => {
                // Validate that the type exists
                let type_manager = self.type_manager.borrow();
                if !type_manager.get_struct(type_name).is_some() && 
                   !type_manager.get_enum(type_name).is_some() {
                    return Err(format!("Cannot implement methods for undefined type: {}", type_name));
                }
                drop(type_manager); // Release the borrow
                
                // Validate each method
                for method in methods {
                    self.validate_function_definition(method)?;
                }
                
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

    /// Analyze a match expression for pattern exhaustiveness and type compatibility
    fn analyze_match_expression(&self, match_expr: &Expression, arms: &[MatchArm]) -> Result<Ty, String> {
        // First, infer the type of the match expression
        let match_type = self.infer_and_validate_expression_immutable(match_expr)?;
        
        // Extract patterns from match arms
        let patterns: Vec<Pattern> = arms.iter().map(|arm| arm.pattern.clone()).collect();
        
        // Check pattern exhaustiveness
        match self.pattern_matcher.check_exhaustiveness(&patterns, &match_type)? {
            ExhaustivenessResult::Exhaustive => {
                // Patterns are exhaustive, continue with type checking
            }
            ExhaustivenessResult::Missing(missing_patterns) => {
                let missing_descriptions: Vec<String> = missing_patterns
                    .iter()
                    .map(|mp| mp.description.clone())
                    .collect();
                return Err(format!(
                    "Non-exhaustive patterns in match expression. Missing patterns: {}",
                    missing_descriptions.join(", ")
                ));
            }
            ExhaustivenessResult::Unreachable(unreachable_indices) => {
                // For now, we'll just warn about unreachable patterns
                // In a full implementation, this would be a warning, not an error
                eprintln!(
                    "Warning: Unreachable patterns detected at positions: {:?}",
                    unreachable_indices
                );
            }
        }
        
        // Check that all match arms have compatible types
        let mut arm_types = Vec::new();
        for arm in arms {
            // Validate the pattern against the match type
            self.validate_pattern_type(&arm.pattern, &match_type)?;
            
            // Check guard condition if present
            if let Some(guard) = &arm.guard {
                let guard_type = self.infer_and_validate_expression_immutable(guard)?;
                if guard_type != Ty::Bool {
                    return Err(format!(
                        "Match guard must be boolean, found: {}",
                        guard_type.to_string()
                    ));
                }
            }
            
            // Check the arm body type
            let arm_type = self.infer_and_validate_expression_immutable(&arm.body)?;
            arm_types.push(arm_type);
        }
        
        // Ensure all arms have the same type
        if let Some(first_type) = arm_types.first() {
            for (i, arm_type) in arm_types.iter().enumerate().skip(1) {
                if arm_type != first_type {
                    return Err(format!(
                        "Match arms have incompatible types: arm 0 has type {}, arm {} has type {}",
                        first_type.to_string(),
                        i,
                        arm_type.to_string()
                    ));
                }
            }
            Ok(first_type.clone())
        } else {
            Err("Match expression must have at least one arm".to_string())
        }
    }
    
    /// Validate method call arguments against method definition
    fn validate_method_call(&self, type_name: &str, method_name: &str, arg_types: &[Ty], method_def: &crate::ast::Function) -> Result<(), String> {
        // Check argument count
        if method_def.parameters.len() != arg_types.len() {
            return Err(format!(
                "Method '{}' on type '{}' expects {} arguments, but {} were provided",
                method_name, type_name, method_def.parameters.len(), arg_types.len()
            ));
        }
        
        // Check argument types
        for (i, (param, provided_type)) in method_def.parameters.iter().zip(arg_types.iter()).enumerate() {
            let expected_type = self.ast_type_to_ty(&param.param_type)?;
            if *provided_type != expected_type {
                return Err(format!(
                    "Method '{}' on type '{}' expects argument {} to be of type {}, but {} was provided",
                    method_name, type_name, i + 1, expected_type.to_string(), provided_type.to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Convert AST Type to Ty for semantic analysis
    fn ast_type_to_ty(&self, ast_type: &crate::ast::Type) -> Result<Ty, String> {
        match ast_type {
            crate::ast::Type::Named(name) => {
                match name.as_str() {
                    "int" | "i32" => Ok(Ty::Int),
                    "float" | "f64" => Ok(Ty::Float),
                    "bool" => Ok(Ty::Bool),
                    "String" => Ok(Ty::String),
                    _ => {
                        // Check if it's a user-defined struct or enum
                        if self.type_manager.borrow().get_struct(name).is_some() {
                            Ok(Ty::Struct(name.clone()))
                        } else if self.type_manager.borrow().get_enum(name).is_some() {
                            Ok(Ty::Enum(name.clone()))
                        } else {
                            Err(format!("Unknown type: {}", name))
                        }
                    }
                }
            }
            crate::ast::Type::Array { element_type, size } => {
                let elem_ty = self.ast_type_to_ty(element_type)?;
                Ok(Ty::Array(Box::new(elem_ty), *size))
            }
            crate::ast::Type::Vec { element_type } => {
                let elem_ty = self.ast_type_to_ty(element_type)?;
                Ok(Ty::Vec(Box::new(elem_ty)))
            }
            crate::ast::Type::Reference { mutable: _, inner_type } => {
                let inner_ty = self.ast_type_to_ty(inner_type)?;
                Ok(Ty::Reference(Box::new(inner_ty)))
            }
            _ => Err(format!("Unsupported type conversion: {:?}", ast_type))
        }
    }
    
    /// Validate function definition
    fn validate_function_definition(&self, func: &crate::ast::Function) -> Result<(), String> {
        // Validate parameter types
        for param in &func.parameters {
            self.validate_ast_type(&param.param_type)?;
        }
        
        // Validate return type if present
        if let Some(return_type) = &func.return_type {
            self.validate_ast_type(return_type)?;
        }
        
        Ok(())
    }
    
    /// Validate an AST type exists
    fn validate_ast_type(&self, ast_type: &crate::ast::Type) -> Result<(), String> {
        match ast_type {
            crate::ast::Type::Named(name) => {
                match name.as_str() {
                    "int" | "i32" | "float" | "f64" | "bool" | "String" => Ok(()),
                    _ => {
                        if self.type_manager.borrow().get_struct(name).is_some() || 
                           self.type_manager.borrow().get_enum(name).is_some() {
                            Ok(())
                        } else {
                            Err(format!("Undefined type: {}", name))
                        }
                    }
                }
            }
            crate::ast::Type::Array { element_type, .. } => {
                self.validate_ast_type(element_type)
            }
            crate::ast::Type::Vec { element_type } => {
                self.validate_ast_type(element_type)
            }
            crate::ast::Type::Reference { inner_type, .. } => {
                self.validate_ast_type(inner_type)
            }
            _ => Ok(()) // Other types are assumed valid for now
        }
    }

    /// Validate that a pattern is compatible with the given type
    fn validate_pattern_type(&self, pattern: &Pattern, expected_type: &Ty) -> Result<(), String> {
        match pattern {
            Pattern::Wildcard | Pattern::Identifier(_) => {
                // Wildcard and identifier patterns match any type
                Ok(())
            }
            Pattern::Literal(expr) => {
                let literal_type = self.infer_and_validate_expression_immutable(expr)?;
                if literal_type != *expected_type {
                    return Err(format!(
                        "Pattern literal type {} doesn't match expected type {}",
                        literal_type.to_string(),
                        expected_type.to_string()
                    ));
                }
                Ok(())
            }
            Pattern::Enum { variant, data } => {
                if let Ty::Enum(enum_name) = expected_type {
                    // Check if the variant exists
                    if !self.type_manager.borrow().get_enum_variants(enum_name)
                        .map_err(|e| e.to_string())?
                        .iter()
                        .any(|v| &v.name == variant) {
                        return Err(format!(
                            "Unknown variant '{}' for enum '{}'",
                            variant, enum_name
                        ));
                    }
                    
                    // If the pattern has data, validate it
                    if let Some(data_pattern) = data {
                        let variant_data_types = self.type_manager.borrow()
                            .get_variant_data_types(enum_name, variant)
                            .map_err(|e| e.to_string())?;
                        
                        if let Some(data_types) = variant_data_types {
                            if data_types.len() == 1 {
                                self.validate_pattern_type(data_pattern, &data_types[0])?;
                            } else {
                                // Multiple data types - should be a tuple pattern
                                if let Pattern::Tuple(tuple_patterns) = data_pattern.as_ref() {
                                    if tuple_patterns.len() != data_types.len() {
                                        return Err(format!(
                                            "Pattern tuple length {} doesn't match variant data length {}",
                                            tuple_patterns.len(),
                                            data_types.len()
                                        ));
                                    }
                                    
                                    for (tuple_pattern, data_type) in tuple_patterns.iter().zip(data_types.iter()) {
                                        self.validate_pattern_type(tuple_pattern, data_type)?;
                                    }
                                } else {
                                    return Err("Expected tuple pattern for multi-data enum variant".to_string());
                                }
                            }
                        } else {
                            return Err(format!(
                                "Variant '{}' of enum '{}' doesn't have data, but pattern expects data",
                                variant, enum_name
                            ));
                        }
                    }
                    Ok(())
                } else {
                    Err(format!(
                        "Enum pattern used on non-enum type: {}",
                        expected_type.to_string()
                    ))
                }
            }
            Pattern::Struct { name, fields, .. } => {
                if let Ty::Struct(struct_name) = expected_type {
                    if name != struct_name {
                        return Err(format!(
                            "Struct pattern '{}' doesn't match type '{}'",
                            name, struct_name
                        ));
                    }
                    
                    // Validate each field pattern
                    for (field_name, field_pattern) in fields {
                        let field_type = self.type_manager.borrow()
                            .validate_field_access(struct_name, field_name)
                            .map_err(|e| e.to_string())?;
                        self.validate_pattern_type(field_pattern, &field_type)?;
                    }
                    Ok(())
                } else {
                    Err(format!(
                        "Struct pattern used on non-struct type: {}",
                        expected_type.to_string()
                    ))
                }
            }
            Pattern::Tuple(tuple_patterns) => {
                // For tuple patterns, we need to know the tuple element types
                // This is simplified - in a real implementation, we'd need tuple types
                for tuple_pattern in tuple_patterns {
                    self.validate_pattern_type(tuple_pattern, expected_type)?;
                }
                Ok(())
            }
            Pattern::Range { start, end, .. } => {
                // Validate that range bounds are compatible with the expected type
                if let Pattern::Literal(start_expr) = start.as_ref() {
                    let start_type = self.infer_and_validate_expression_immutable(start_expr)?;
                    if start_type != *expected_type {
                        return Err(format!(
                            "Range start type {} doesn't match expected type {}",
                            start_type.to_string(),
                            expected_type.to_string()
                        ));
                    }
                }
                
                if let Pattern::Literal(end_expr) = end.as_ref() {
                    let end_type = self.infer_and_validate_expression_immutable(end_expr)?;
                    if end_type != *expected_type {
                        return Err(format!(
                            "Range end type {} doesn't match expected type {}",
                            end_type.to_string(),
                            expected_type.to_string()
                        ));
                    }
                }
                Ok(())
            }
            Pattern::Or(or_patterns) => {
                // All patterns in an or-pattern must be compatible with the expected type
                for or_pattern in or_patterns {
                    self.validate_pattern_type(or_pattern, expected_type)?;
                }
                Ok(())
            }
            Pattern::Binding { pattern, .. } => {
                // Validate the inner pattern
                self.validate_pattern_type(pattern, expected_type)
            }
        }
    }

    // Collection and String Validation Methods

    /// Validate array literal type inference
    fn validate_array_literal(&self, elements: &[Expression]) -> Result<Ty, String> {
        if elements.is_empty() {
            return Err("Cannot infer type of empty array literal".to_string());
        }

        // Infer type from first element
        let first_type = self.infer_and_validate_expression_immutable(&elements[0])?;
        
        // Validate all elements have the same type
        for (i, element) in elements.iter().enumerate().skip(1) {
            let element_type = self.infer_and_validate_expression_immutable(element)?;
            if element_type != first_type {
                return Err(format!(
                    "Array literal element {} has type {}, but expected {}",
                    i, element_type.to_string(), first_type.to_string()
                ));
            }
        }

        Ok(Ty::Array(Box::new(first_type), Some(elements.len())))
    }

    /// Validate array literal type inference (mutable version)
    fn validate_array_literal_mutable(&self, elements: &mut [Expression]) -> Result<Ty, String> {
        if elements.is_empty() {
            return Err("Cannot infer type of empty array literal".to_string());
        }

        // Infer type from first element
        let first_type = self.infer_and_validate_expression(&mut elements[0])?;
        
        // Validate all elements have the same type
        for (i, element) in elements.iter_mut().enumerate().skip(1) {
            let element_type = self.infer_and_validate_expression(element)?;
            if element_type != first_type {
                return Err(format!(
                    "Array literal element {} has type {}, but expected {}",
                    i, element_type.to_string(), first_type.to_string()
                ));
            }
        }

        Ok(Ty::Array(Box::new(first_type), Some(elements.len())))
    }

    /// Validate array access with bounds checking
    fn validate_array_access(&self, array: &Expression, index: &Expression) -> Result<Ty, String> {
        let array_type = self.infer_and_validate_expression_immutable(array)?;
        let index_type = self.infer_and_validate_expression_immutable(index)?;

        // Index must be integer
        if index_type != Ty::Int {
            return Err(format!(
                "Array index must be integer, found: {}",
                index_type.to_string()
            ));
        }

        match array_type {
            Ty::Array(element_type, size) => {
                // Static bounds checking for literal indices
                if let Expression::IntegerLiteral(idx) = index {
                    if let Some(array_size) = size {
                        if *idx < 0 {
                            return Err(format!("Array index {} is negative", idx));
                        }
                        if *idx as usize >= array_size {
                            return Err(format!(
                                "Array index {} is out of bounds for array of size {}",
                                idx, array_size
                            ));
                        }
                    }
                }
                Ok(*element_type)
            }
            Ty::Vec(element_type) => {
                // Vec access - runtime bounds checking will be generated
                Ok(*element_type)
            }
            _ => Err(format!(
                "Cannot index into non-array type: {}",
                array_type.to_string()
            ))
        }
    }

    /// Validate array access with bounds checking (mutable version)
    fn validate_array_access_mutable(&self, array: &mut Expression, index: &mut Expression) -> Result<Ty, String> {
        let array_type = self.infer_and_validate_expression(array)?;
        let index_type = self.infer_and_validate_expression(index)?;

        // Index must be integer
        if index_type != Ty::Int {
            return Err(format!(
                "Array index must be integer, found: {}",
                index_type.to_string()
            ));
        }

        match array_type {
            Ty::Array(element_type, size) => {
                // Static bounds checking for literal indices
                if let Expression::IntegerLiteral(idx) = index {
                    if let Some(array_size) = size {
                        if *idx < 0 {
                            return Err(format!("Array index {} is negative", idx));
                        }
                        if *idx as usize >= array_size {
                            return Err(format!(
                                "Array index {} is out of bounds for array of size {}",
                                idx, array_size
                            ));
                        }
                    }
                }
                Ok(*element_type)
            }
            Ty::Vec(element_type) => {
                // Vec access - runtime bounds checking will be generated
                Ok(*element_type)
            }
            _ => Err(format!(
                "Cannot index into non-array type: {}",
                array_type.to_string()
            ))
        }
    }

    /// Validate Vec macro
    fn validate_vec_macro(&self, elements: &[Expression]) -> Result<Ty, String> {
        if elements.is_empty() {
            // Empty vec needs explicit type annotation
            return Err("Cannot infer type of empty vec! macro - use vec![T; 0] or provide explicit type".to_string());
        }

        // Infer type from first element
        let first_type = self.infer_and_validate_expression_immutable(&elements[0])?;
        
        // Validate all elements have the same type
        for (i, element) in elements.iter().enumerate().skip(1) {
            let element_type = self.infer_and_validate_expression_immutable(element)?;
            if element_type != first_type {
                return Err(format!(
                    "Vec macro element {} has type {}, but expected {}",
                    i, element_type.to_string(), first_type.to_string()
                ));
            }
        }

        Ok(Ty::Vec(Box::new(first_type)))
    }

    /// Validate Vec macro (mutable version)
    fn validate_vec_macro_mutable(&self, elements: &mut [Expression]) -> Result<Ty, String> {
        if elements.is_empty() {
            // Empty vec needs explicit type annotation
            return Err("Cannot infer type of empty vec! macro - use vec![T; 0] or provide explicit type".to_string());
        }

        // Infer type from first element
        let first_type = self.infer_and_validate_expression(&mut elements[0])?;
        
        // Validate all elements have the same type
        for (i, element) in elements.iter_mut().enumerate().skip(1) {
            let element_type = self.infer_and_validate_expression(element)?;
            if element_type != first_type {
                return Err(format!(
                    "Vec macro element {} has type {}, but expected {}",
                    i, element_type.to_string(), first_type.to_string()
                ));
            }
        }

        Ok(Ty::Vec(Box::new(first_type)))
    }

    /// Validate format macro
    fn validate_format_macro(&self, format_string: &str, arguments: &[Expression]) -> Result<Ty, String> {
        // Count placeholders in format string
        let placeholder_count = format_string.matches("{}").count();
        
        if placeholder_count != arguments.len() {
            return Err(format!(
                "Format string has {} placeholders but {} arguments were provided",
                placeholder_count, arguments.len()
            ));
        }

        // Validate all arguments are printable
        for (i, arg) in arguments.iter().enumerate() {
            let arg_type = self.infer_and_validate_expression_immutable(arg)?;
            if !self.is_printable_type(&arg_type) {
                return Err(format!(
                    "Argument {} of type {} is not printable in format! macro",
                    i + 1, arg_type.to_string()
                ));
            }
        }

        // format! macro returns String type
        Ok(Ty::from_string("String").unwrap_or(Ty::Int)) // Fallback to Int if String type not defined
    }

    /// Validate format macro (mutable version)
    fn validate_format_macro_mutable(&self, format_string: &str, arguments: &mut [Expression]) -> Result<Ty, String> {
        // Count placeholders in format string
        let placeholder_count = format_string.matches("{}").count();
        
        if placeholder_count != arguments.len() {
            return Err(format!(
                "Format string has {} placeholders but {} arguments were provided",
                placeholder_count, arguments.len()
            ));
        }

        // Validate all arguments are printable
        for (i, arg) in arguments.iter_mut().enumerate() {
            let arg_type = self.infer_and_validate_expression(arg)?;
            if !self.is_printable_type(&arg_type) {
                return Err(format!(
                    "Argument {} of type {} is not printable in format! macro",
                    i + 1, arg_type.to_string()
                ));
            }
        }

        // format! macro returns String type
        Ok(Ty::from_string("String").unwrap_or(Ty::Int)) // Fallback to Int if String type not defined
    }

    /// Validate Vec method calls
    fn validate_vec_method_call(&self, method: &str, arg_types: &[Ty], element_type: &Ty) -> Result<Ty, String> {
        match method {
            "push" => {
                if arg_types.len() != 1 {
                    return Err(format!("Vec::push expects 1 argument, got {}", arg_types.len()));
                }
                if arg_types[0] != *element_type {
                    return Err(format!(
                        "Vec::push expects argument of type {}, got {}",
                        element_type.to_string(), arg_types[0].to_string()
                    ));
                }
                Ok(Ty::Int) // push returns unit type (represented as Int for now)
            }
            "pop" => {
                if !arg_types.is_empty() {
                    return Err(format!("Vec::pop expects 0 arguments, got {}", arg_types.len()));
                }
                // pop returns Option<T> - for now return the element type
                Ok(element_type.clone())
            }
            "len" => {
                if !arg_types.is_empty() {
                    return Err(format!("Vec::len expects 0 arguments, got {}", arg_types.len()));
                }
                Ok(Ty::Int) // len returns usize (represented as Int)
            }
            "capacity" => {
                if !arg_types.is_empty() {
                    return Err(format!("Vec::capacity expects 0 arguments, got {}", arg_types.len()));
                }
                Ok(Ty::Int) // capacity returns usize (represented as Int)
            }
            "is_empty" => {
                if !arg_types.is_empty() {
                    return Err(format!("Vec::is_empty expects 0 arguments, got {}", arg_types.len()));
                }
                Ok(Ty::Bool)
            }
            "clear" => {
                if !arg_types.is_empty() {
                    return Err(format!("Vec::clear expects 0 arguments, got {}", arg_types.len()));
                }
                Ok(Ty::Int) // clear returns unit type
            }
            "get" => {
                if arg_types.len() != 1 {
                    return Err(format!("Vec::get expects 1 argument, got {}", arg_types.len()));
                }
                if arg_types[0] != Ty::Int {
                    return Err(format!(
                        "Vec::get expects index of type int, got {}",
                        arg_types[0].to_string()
                    ));
                }
                // get returns Option<&T> - for now return the element type
                Ok(element_type.clone())
            }
            _ => Err(format!("Unknown Vec method: {}", method))
        }
    }

    /// Validate array method calls
    fn validate_array_method_call(&self, method: &str, arg_types: &[Ty], element_type: &Ty) -> Result<Ty, String> {
        match method {
            "len" => {
                if !arg_types.is_empty() {
                    return Err(format!("Array::len expects 0 arguments, got {}", arg_types.len()));
                }
                Ok(Ty::Int) // len returns usize (represented as Int)
            }
            "is_empty" => {
                if !arg_types.is_empty() {
                    return Err(format!("Array::is_empty expects 0 arguments, got {}", arg_types.len()));
                }
                Ok(Ty::Bool)
            }
            "get" => {
                if arg_types.len() != 1 {
                    return Err(format!("Array::get expects 1 argument, got {}", arg_types.len()));
                }
                if arg_types[0] != Ty::Int {
                    return Err(format!(
                        "Array::get expects index of type int, got {}",
                        arg_types[0].to_string()
                    ));
                }
                // get returns Option<&T> - for now return the element type
                Ok(element_type.clone())
            }
            "first" => {
                if !arg_types.is_empty() {
                    return Err(format!("Array::first expects 0 arguments, got {}", arg_types.len()));
                }
                // first returns Option<&T> - for now return the element type
                Ok(element_type.clone())
            }
            "last" => {
                if !arg_types.is_empty() {
                    return Err(format!("Array::last expects 0 arguments, got {}", arg_types.len()));
                }
                // last returns Option<&T> - for now return the element type
                Ok(element_type.clone())
            }
            _ => Err(format!("Unknown array method: {}", method))
        }
    }

    /// Validate String method calls
    fn validate_string_method_call(&self, method: &str, arg_types: &[Ty]) -> Result<Ty, String> {
        match method {
            "len" => {
                if !arg_types.is_empty() {
                    return Err(format!("String::len expects 0 arguments, got {}", arg_types.len()));
                }
                Ok(Ty::Int) // len returns usize (represented as Int)
            }
            "is_empty" => {
                if !arg_types.is_empty() {
                    return Err(format!("String::is_empty expects 0 arguments, got {}", arg_types.len()));
                }
                Ok(Ty::Bool)
            }
            "chars" => {
                if !arg_types.is_empty() {
                    return Err(format!("String::chars expects 0 arguments, got {}", arg_types.len()));
                }
                // chars returns an iterator - for now return Int as placeholder
                Ok(Ty::Int)
            }
            "push" => {
                if arg_types.len() != 1 {
                    return Err(format!("String::push expects 1 argument, got {}", arg_types.len()));
                }
                // push expects a char - for now accept any type
                Ok(Ty::Int) // push returns unit type
            }
            "push_str" => {
                if arg_types.len() != 1 {
                    return Err(format!("String::push_str expects 1 argument, got {}", arg_types.len()));
                }
                if arg_types[0] != Ty::String {
                    return Err(format!(
                        "String::push_str expects argument of type String, got {}",
                        arg_types[0].to_string()
                    ));
                }
                Ok(Ty::Int) // push_str returns unit type
            }
            "pop" => {
                if !arg_types.is_empty() {
                    return Err(format!("String::pop expects 0 arguments, got {}", arg_types.len()));
                }
                // pop returns Option<char> - for now return Int as placeholder
                Ok(Ty::Int)
            }
            "clear" => {
                if !arg_types.is_empty() {
                    return Err(format!("String::clear expects 0 arguments, got {}", arg_types.len()));
                }
                Ok(Ty::Int) // clear returns unit type
            }
            "contains" => {
                if arg_types.len() != 1 {
                    return Err(format!("String::contains expects 1 argument, got {}", arg_types.len()));
                }
                if arg_types[0] != Ty::String {
                    return Err(format!(
                        "String::contains expects argument of type String, got {}",
                        arg_types[0].to_string()
                    ));
                }
                Ok(Ty::Bool)
            }
            "starts_with" => {
                if arg_types.len() != 1 {
                    return Err(format!("String::starts_with expects 1 argument, got {}", arg_types.len()));
                }
                if arg_types[0] != Ty::String {
                    return Err(format!(
                        "String::starts_with expects argument of type String, got {}",
                        arg_types[0].to_string()
                    ));
                }
                Ok(Ty::Bool)
            }
            "ends_with" => {
                if arg_types.len() != 1 {
                    return Err(format!("String::ends_with expects 1 argument, got {}", arg_types.len()));
                }
                if arg_types[0] != Ty::String {
                    return Err(format!(
                        "String::ends_with expects argument of type String, got {}",
                        arg_types[0].to_string()
                    ));
                }
                Ok(Ty::Bool)
            }
            "to_uppercase" => {
                if !arg_types.is_empty() {
                    return Err(format!("String::to_uppercase expects 0 arguments, got {}", arg_types.len()));
                }
                Ok(Ty::String)
            }
            "to_lowercase" => {
                if !arg_types.is_empty() {
                    return Err(format!("String::to_lowercase expects 0 arguments, got {}", arg_types.len()));
                }
                Ok(Ty::String)
            }
            "trim" => {
                if !arg_types.is_empty() {
                    return Err(format!("String::trim expects 0 arguments, got {}", arg_types.len()));
                }
                Ok(Ty::String) // trim returns &str, but we'll use String for simplicity
            }
            "replace" => {
                if arg_types.len() != 2 {
                    return Err(format!("String::replace expects 2 arguments, got {}", arg_types.len()));
                }
                if arg_types[0] != Ty::String || arg_types[1] != Ty::String {
                    return Err(format!(
                        "String::replace expects arguments of type (String, String), got ({}, {})",
                        arg_types[0].to_string(), arg_types[1].to_string()
                    ));
                }
                Ok(Ty::String)
            }
            "split" => {
                if arg_types.len() != 1 {
                    return Err(format!("String::split expects 1 argument, got {}", arg_types.len()));
                }
                if arg_types[0] != Ty::String {
                    return Err(format!(
                        "String::split expects argument of type String, got {}",
                        arg_types[0].to_string()
                    ));
                }
                // split returns an iterator - for now return Vec<String>
                Ok(Ty::Vec(Box::new(Ty::String)))
            }
            _ => Err(format!("Unknown String method: {}", method))
        }
    }

    /// Enhanced printable type checking for collections and strings
    fn is_printable_type(&self, ty: &Ty) -> bool {
        match ty {
            Ty::Int | Ty::Float | Ty::Bool | Ty::String => true,
            Ty::Array(element_type, _) => self.is_printable_type(element_type),
            Ty::Vec(element_type) => self.is_printable_type(element_type),
            Ty::Struct(_) | Ty::Enum(_) => {
                // For now, assume structs and enums are printable if they implement Display
                // In a full implementation, this would check for Display trait implementation
                true
            }
            Ty::Reference(inner_type) => self.is_printable_type(inner_type),
        }
    }
}