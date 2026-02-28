use crate::ast::{
    AstNode, Block, ComparisonOp, Expression, LogicalOp, Parameter, Statement, UnaryOp,
};
use crate::types::{OwnershipState, Ty, infer_binary_type};
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
    pub ownership: OwnershipState,
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
}

impl Default for FunctionTable {
    fn default() -> Self {
        Self::new()
    }
}

impl FunctionTable {
    pub fn define_function(&mut self, info: FunctionInfo) -> Result<(), String> {
        if self.functions.contains_key(&info.name) {
            return Err(format!(
                "Error: Function `{}` is already defined.",
                info.name
            ));
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
                        "i32" | "int" => Ty::Int,
                        "f64" | "float" => Ty::Float,
                        "bool" => Ty::Bool,
                        "String" => Ty::String,
                        _ => Ty::Int,
                    },
                    crate::ast::Type::Array(_, _) | crate::ast::Type::Tuple(_) => Ty::Int,
                    crate::ast::Type::Reference(_, _) | crate::ast::Type::Generic(_, _) => Ty::Int,
                };

                if expected_type != *arg_type {
                    return Err(format!(
                        "Error: Function `{}` expects type `{}` for argument {}, but `{}` was provided.",
                        name,
                        expected_type,
                        i + 1,
                        arg_type
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
}

impl Default for ScopeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ScopeManager {
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

    pub fn define_variable(
        &mut self,
        name: String,
        var_type: Ty,
        mutable: bool,
        initialized: bool,
    ) -> Result<String, String> {
        // Check if variable already exists in current scope
        if let Some(current_scope) = self.scopes.last()
            && current_scope.contains_key(&name)
        {
            return Err(format!(
                "Error: Variable `{}` is already defined in this scope.",
                name
            ));
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
            ownership: OwnershipState::Owned,
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

    pub fn update_variable_initialization(
        &mut self,
        name: &str,
        initialized: bool,
    ) -> Result<(), String> {
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

    /// Mark a variable as moved. Returns error if already moved.
    pub fn mark_moved(&mut self, name: &str) -> Result<(), String> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var_info) = scope.get_mut(name) {
                if var_info.ownership == OwnershipState::Moved {
                    return Err(format!(
                        "Error: Use of moved value `{}`. Value was previously moved.",
                        name
                    ));
                }
                var_info.ownership = OwnershipState::Moved;
                return Ok(());
            }
        }
        Err(format!("Error: Variable `{}` not found.", name))
    }

    /// Check if a variable is in a valid (non-moved) state for use.
    pub fn check_not_moved(&self, name: &str) -> Result<(), String> {
        for scope in self.scopes.iter().rev() {
            if let Some(var_info) = scope.get(name) {
                if var_info.ownership == OwnershipState::Moved {
                    return Err(format!(
                        "Error: Use of moved value `{}`. Value was previously moved.",
                        name
                    ));
                }
                return Ok(());
            }
        }
        Ok(()) // Not found in scope manager, might be in old symbol table
    }

    /// Get the ownership state of a variable.
    pub fn get_ownership(&self, name: &str) -> Option<&OwnershipState> {
        for scope in self.scopes.iter().rev() {
            if let Some(var_info) = scope.get(name) {
                return Some(&var_info.ownership);
            }
        }
        None
    }

    /// Add an immutable borrow to a variable. Fails if mutably borrowed.
    pub fn add_immutable_borrow(&mut self, name: &str) -> Result<(), String> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var_info) = scope.get_mut(name) {
                match &var_info.ownership {
                    OwnershipState::Moved => {
                        return Err(format!(
                            "Error: Cannot borrow `{}` because it was moved.",
                            name
                        ));
                    }
                    OwnershipState::MutablyBorrowed => {
                        return Err(format!(
                            "Error: Cannot borrow `{}` as immutable because it is also borrowed as mutable.",
                            name
                        ));
                    }
                    OwnershipState::ImmutablyBorrowed(count) => {
                        var_info.ownership = OwnershipState::ImmutablyBorrowed(count + 1);
                        return Ok(());
                    }
                    OwnershipState::Owned => {
                        var_info.ownership = OwnershipState::ImmutablyBorrowed(1);
                        return Ok(());
                    }
                }
            }
        }
        Err(format!("Error: Variable `{}` not found.", name))
    }

    /// Add a mutable borrow to a variable. Fails if any borrows exist.
    pub fn add_mutable_borrow(&mut self, name: &str) -> Result<(), String> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var_info) = scope.get_mut(name) {
                match &var_info.ownership {
                    OwnershipState::Moved => {
                        return Err(format!(
                            "Error: Cannot borrow `{}` as mutable because it was moved.",
                            name
                        ));
                    }
                    OwnershipState::MutablyBorrowed => {
                        return Err(format!(
                            "Error: Cannot borrow `{}` as mutable more than once at a time.",
                            name
                        ));
                    }
                    OwnershipState::ImmutablyBorrowed(_) => {
                        return Err(format!(
                            "Error: Cannot borrow `{}` as mutable because it is also borrowed as immutable.",
                            name
                        ));
                    }
                    OwnershipState::Owned => {
                        if !var_info.mutable {
                            return Err(format!(
                                "Error: Cannot borrow `{}` as mutable because it is not declared as mutable.",
                                name
                            ));
                        }
                        var_info.ownership = OwnershipState::MutablyBorrowed;
                        return Ok(());
                    }
                }
            }
        }
        Err(format!("Error: Variable `{}` not found.", name))
    }
}

pub struct SemanticAnalyzer {
    symbol_table: HashMap<String, VariableInfo>,
    #[allow(dead_code)]
    function_table: FunctionTable,
    scope_manager: ScopeManager,
    /// Stack of active generic type parameter sets (e.g., ["T", "U"] for fn<T, U>)
    type_param_scopes: Vec<Vec<String>>,
    /// Trait registry: trait name -> list of required method names
    trait_registry: HashMap<String, Vec<String>>,
    /// Trait impl registry: type name -> list of implemented trait names
    trait_impls: HashMap<String, Vec<String>>,
    /// Function trait bounds: function name -> [(type_param, [trait_name])]
    function_bounds: HashMap<String, Vec<(String, Vec<String>)>>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let type_manager = Rc::new(RefCell::new(TypeDefinitionManager::new()));
        let pattern_matcher = PatternMatcher::new(type_manager.clone());
        
        Self {
            symbol_table: HashMap::new(),
            function_table: FunctionTable::new(),
            scope_manager: ScopeManager::new(),
            type_param_scopes: Vec::new(),
            trait_registry: HashMap::new(),
            trait_impls: HashMap::new(),
            function_bounds: HashMap::new(),
        }
    }

    /// Check if a name is an in-scope type parameter.
    fn is_type_param(&self, name: &str) -> bool {
        self.type_param_scopes
            .iter()
            .any(|scope| scope.iter().any(|p| p == name))
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticAnalyzer {
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
                // Phase 5: Check for use-after-move
                self.scope_manager.check_not_moved(name)?;

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

    #[allow(dead_code)]
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
            Expression::Binary {
                op, left, right, ..
            } => {
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
            Expression::Print {
                format_string,
                arguments,
            } => {
                self.validate_format_string_and_args(format_string, arguments)?;
                Ok(Ty::Int)
            }
            Expression::Println {
                format_string,
                arguments,
            } => {
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
            // Phase 4 expressions
            Expression::StringLiteral(_) => Ok(Ty::String),
            Expression::MethodCall { object, method, .. } => {
                let obj_ty = self.infer_and_validate_expression(object)?;
                // Phase 6: Option, Result, Vec, HashMap methods
                match &obj_ty {
                    Ty::Option(inner) => match method.as_str() {
                        "is_some" | "is_none" => Ok(Ty::Bool),
                        "unwrap" | "expect" | "unwrap_or" | "unwrap_or_else" => Ok(*inner.clone()),
                        _ => Ok(Ty::Int), // Unknown method
                    },
                    Ty::Result(ok_ty, err_ty) => match method.as_str() {
                        "is_ok" | "is_err" => Ok(Ty::Bool),
                        "unwrap" | "expect" | "unwrap_or" | "unwrap_or_else" => Ok(*ok_ty.clone()),
                        "unwrap_err" | "expect_err" => Ok(*err_ty.clone()),
                        _ => Ok(Ty::Int), // Unknown method
                    },
                    Ty::Vec(elem) => match method.as_str() {
                        "len" => Ok(Ty::Int),
                        "is_empty" => Ok(Ty::Bool),
                        "push" | "clear" => Ok(Ty::Void),
                        "pop" | "first" | "last" | "get" => Ok(Ty::Option(elem.clone())),
                        _ => Ok(Ty::Int), // Unknown method
                    },
                    Ty::HashMap(_, val) => match method.as_str() {
                        "len" => Ok(Ty::Int),
                        "is_empty" | "contains_key" => Ok(Ty::Bool),
                        "insert" | "remove" | "clear" => Ok(Ty::Void),
                        "get" => Ok(Ty::Option(val.clone())),
                        _ => Ok(Ty::Int), // Unknown method
                    },
                    Ty::String => match method.as_str() {
                        "len" => Ok(Ty::Int),
                        "is_empty" | "contains" | "starts_with" | "ends_with" => Ok(Ty::Bool),
                        "to_uppercase" | "to_lowercase" | "trim" | "trim_start" | "trim_end" => {
                            Ok(Ty::String)
                        }
                        "chars" => Ok(Ty::Vec(Box::new(Ty::Int))), // char as int
                        _ => Ok(Ty::Int),                          // Unknown method
                    },
                    _ => Ok(Ty::Int), // Other method calls - stub
                }
            }
            Expression::ArrayLiteral(elements) => {
                let elem_type = if let Some(first) = elements.first() {
                    self.infer_and_validate_expression(&mut first.clone())?
                } else {
                    Ty::Int
                };
                Ok(Ty::Array(Box::new(elem_type), elements.len()))
            }
            Expression::ArrayRepeat { value, count } => {
                let elem_type = self.infer_and_validate_expression(value)?;
                Ok(Ty::Array(Box::new(elem_type), *count))
            }
            Expression::IndexAccess { object, index } => {
                let obj_type = self.infer_and_validate_expression(object)?;
                self.infer_and_validate_expression(index)?;
                match obj_type {
                    Ty::Array(elem, _) => Ok(*elem),
                    _ => Err("Cannot index into non-array type".to_string()),
                }
            }
            Expression::FieldAccess { .. }
            | Expression::TupleLiteral(_)
            | Expression::TupleIndex { .. } => Ok(Ty::Int), // Stub
            Expression::StructLiteral { name, .. } => Ok(Ty::Struct(name.clone())),
            // Phase 6: Special handling for Option and Result constructors
            Expression::EnumVariant {
                enum_name,
                variant,
                data,
            } => {
                match enum_name.as_str() {
                    "Option" => {
                        match variant.as_str() {
                            "Some" => {
                                // Some(value) -> Option<typeof(value)>
                                if let Some(inner_expr) = data {
                                    let inner_ty = self
                                        .infer_and_validate_expression(&mut inner_expr.clone())?;
                                    Ok(Ty::Option(Box::new(inner_ty)))
                                } else {
                                    Err("Some variant requires a value".to_string())
                                }
                            }
                            "None" => {
                                // None -> Option<unknown>, type must be inferred from context
                                // For now, default to Option<Int> - proper inference would need context
                                Ok(Ty::Option(Box::new(Ty::Int)))
                            }
                            _ => Err(format!("Unknown Option variant: {}", variant)),
                        }
                    }
                    "Result" => {
                        match variant.as_str() {
                            "Ok" => {
                                // Ok(value) -> Result<typeof(value), String> (default error type)
                                if let Some(inner_expr) = data {
                                    let inner_ty = self
                                        .infer_and_validate_expression(&mut inner_expr.clone())?;
                                    Ok(Ty::Result(Box::new(inner_ty), Box::new(Ty::String)))
                                } else {
                                    Err("Ok variant requires a value".to_string())
                                }
                            }
                            "Err" => {
                                // Err(error) -> Result<Int, typeof(error)> (default ok type)
                                if let Some(inner_expr) = data {
                                    let inner_ty = self
                                        .infer_and_validate_expression(&mut inner_expr.clone())?;
                                    Ok(Ty::Result(Box::new(Ty::Int), Box::new(inner_ty)))
                                } else {
                                    Err("Err variant requires a value".to_string())
                                }
                            }
                            _ => Err(format!("Unknown Result variant: {}", variant)),
                        }
                    }
                    _ => Ok(Ty::Enum(enum_name.clone())),
                }
            }
            Expression::Match { .. } => Ok(Ty::Int), // Stub
            // Phase 5: Borrow and Deref
            Expression::Borrow { expr, mutable } => {
                let inner_ty = self.infer_and_validate_expression(expr)?;
                Ok(Ty::Reference(Box::new(inner_ty), *mutable))
            }
            Expression::Deref(expr) => {
                let inner_ty = self.infer_and_validate_expression(expr)?;
                match inner_ty {
                    Ty::Reference(inner, _) => Ok(*inner),
                    _ => Err("Cannot dereference non-reference type".to_string()),
                }
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
            Expression::Binary {
                op, left, right, ..
            } => {
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
            Expression::Print {
                format_string,
                arguments,
            } => {
                self.validate_format_string_and_args_immutable(format_string, arguments)?;
                Ok(Ty::Int)
            }
            Expression::Println {
                format_string,
                arguments,
            } => {
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
            // Phase 4 expressions
            Expression::StringLiteral(_) => Ok(Ty::String),
            Expression::MethodCall { object, method, .. } => {
                let obj_ty = self.infer_and_validate_expression_immutable(object)?;
                // Phase 6: Option, Result, Vec, HashMap methods
                match &obj_ty {
                    Ty::Option(inner) => match method.as_str() {
                        "is_some" | "is_none" => Ok(Ty::Bool),
                        "unwrap" | "expect" | "unwrap_or" | "unwrap_or_else" => Ok(*inner.clone()),
                        _ => Ok(Ty::Int), // Unknown method
                    },
                    Ty::Result(ok_ty, err_ty) => match method.as_str() {
                        "is_ok" | "is_err" => Ok(Ty::Bool),
                        "unwrap" | "expect" | "unwrap_or" | "unwrap_or_else" => Ok(*ok_ty.clone()),
                        "unwrap_err" | "expect_err" => Ok(*err_ty.clone()),
                        _ => Ok(Ty::Int), // Unknown method
                    },
                    Ty::Vec(elem) => match method.as_str() {
                        "len" => Ok(Ty::Int),
                        "is_empty" => Ok(Ty::Bool),
                        "push" | "clear" => Ok(Ty::Void),
                        "pop" | "first" | "last" | "get" => Ok(Ty::Option(elem.clone())),
                        _ => Ok(Ty::Int), // Unknown method
                    },
                    Ty::HashMap(_, val) => match method.as_str() {
                        "len" => Ok(Ty::Int),
                        "is_empty" | "contains_key" => Ok(Ty::Bool),
                        "insert" | "remove" | "clear" => Ok(Ty::Void),
                        "get" => Ok(Ty::Option(val.clone())),
                        _ => Ok(Ty::Int), // Unknown method
                    },
                    Ty::String => match method.as_str() {
                        "len" => Ok(Ty::Int),
                        "is_empty" | "contains" | "starts_with" | "ends_with" => Ok(Ty::Bool),
                        "to_uppercase" | "to_lowercase" | "trim" | "trim_start" | "trim_end" => {
                            Ok(Ty::String)
                        }
                        "chars" => Ok(Ty::Vec(Box::new(Ty::Int))), // char as int
                        _ => Ok(Ty::Int),                          // Unknown method
                    },
                    _ => Ok(Ty::Int), // Other method calls - stub
                }
            }
            Expression::ArrayLiteral(elements) => {
                let elem_type = if let Some(first) = elements.first() {
                    self.infer_and_validate_expression_immutable(first)?
                } else {
                    Ty::Int
                };
                Ok(Ty::Array(Box::new(elem_type), elements.len()))
            }
            Expression::ArrayRepeat { value, count } => {
                let elem_type = self.infer_and_validate_expression_immutable(value)?;
                Ok(Ty::Array(Box::new(elem_type), *count))
            }
            Expression::IndexAccess { object, index } => {
                let obj_type = self.infer_and_validate_expression_immutable(object)?;
                self.infer_and_validate_expression_immutable(index)?;
                match obj_type {
                    Ty::Array(elem, _) => Ok(*elem),
                    _ => Err("Cannot index into non-array type".to_string()),
                }
            }
            Expression::FieldAccess { .. }
            | Expression::TupleLiteral(_)
            | Expression::TupleIndex { .. } => Ok(Ty::Int), // Stub
            Expression::StructLiteral { name, .. } => Ok(Ty::Struct(name.clone())),
            // Phase 6: Special handling for Option and Result constructors
            Expression::EnumVariant {
                enum_name,
                variant,
                data,
            } => match enum_name.as_str() {
                "Option" => match variant.as_str() {
                    "Some" => {
                        if let Some(inner_expr) = data {
                            let inner_ty =
                                self.infer_and_validate_expression_immutable(inner_expr)?;
                            Ok(Ty::Option(Box::new(inner_ty)))
                        } else {
                            Err("Some variant requires a value".to_string())
                        }
                    }
                    "None" => Ok(Ty::Option(Box::new(Ty::Int))),
                    _ => Err(format!("Unknown Option variant: {}", variant)),
                },
                "Result" => match variant.as_str() {
                    "Ok" => {
                        if let Some(inner_expr) = data {
                            let inner_ty =
                                self.infer_and_validate_expression_immutable(inner_expr)?;
                            Ok(Ty::Result(Box::new(inner_ty), Box::new(Ty::String)))
                        } else {
                            Err("Ok variant requires a value".to_string())
                        }
                    }
                    "Err" => {
                        if let Some(inner_expr) = data {
                            let inner_ty =
                                self.infer_and_validate_expression_immutable(inner_expr)?;
                            Ok(Ty::Result(Box::new(Ty::Int), Box::new(inner_ty)))
                        } else {
                            Err("Err variant requires a value".to_string())
                        }
                    }
                    _ => Err(format!("Unknown Result variant: {}", variant)),
                },
                _ => Ok(Ty::Enum(enum_name.clone())),
            },
            Expression::Match { .. } => Ok(Ty::Int), // Stub
            // Phase 5: Borrow and Deref
            Expression::Borrow { expr, mutable } => {
                let inner_ty = self.infer_and_validate_expression_immutable(expr)?;
                Ok(Ty::Reference(Box::new(inner_ty), *mutable))
            }
            Expression::Deref(expr) => {
                let inner_ty = self.infer_and_validate_expression_immutable(expr)?;
                match inner_ty {
                    Ty::Reference(inner, _) => Ok(*inner),
                    _ => Err("Cannot dereference non-reference type".to_string()),
                }
            }
        }
    }

    #[allow(dead_code)]
    fn validate_format_string_and_args(
        &self,
        format_string: &str,
        arguments: &[Expression],
    ) -> Result<(), String> {
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
                    arg_type
                ));
            }
        }

        Ok(())
    }

    fn validate_format_string_and_args_immutable(
        &self,
        format_string: &str,
        arguments: &[Expression],
    ) -> Result<(), String> {
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
                    arg_type
                ));
            }
        }

        Ok(())
    }



    fn validate_comparison_operands(
        &self,
        _op: &ComparisonOp,
        left_type: &Ty,
        right_type: &Ty,
    ) -> Result<(), String> {
        if left_type == right_type
            || (left_type == &Ty::Int && right_type == &Ty::Float)
            || (left_type == &Ty::Float && right_type == &Ty::Int)
        {
            Ok(()) // Allow same-type comparisons and int/float comparisons
        } else {
            Err(format!(
                "Error: Cannot compare types `{}` and `{}`.",
                left_type, right_type
            ))
        }
    }

    fn validate_logical_operands(
        &self,
        _op: &LogicalOp,
        left_type: &Ty,
        right_type: &Ty,
    ) -> Result<(), String> {
        if left_type != &Ty::Bool {
            return Err(format!(
                "Error: Left operand of logical operation must be boolean, found: {}",
                left_type
            ));
        }
        if right_type != &Ty::Bool {
            return Err(format!(
                "Error: Right operand of logical operation must be boolean, found: {}",
                right_type
            ));
        }
        Ok(())
    }

    fn validate_unary_operation(&self, op: &UnaryOp, operand_type: &Ty) -> Result<Ty, String> {
        match op {
            UnaryOp::Not => {
                if operand_type == &Ty::Bool {
                    Ok(Ty::Bool)
                } else {
                    Err(format!(
                        "Error: Logical NOT operator requires boolean operand, found: {}",
                        operand_type
                    ))
                }
            }
            UnaryOp::Negate => {
                if operand_type == &Ty::Int || operand_type == &Ty::Float {
                    Ok(operand_type.clone())
                } else {
                    Err(format!(
                        "Error: Unary minus operator requires numeric operand, found: {}",
                        operand_type
                    ))
                }
            }
        }
    }

    fn analyze_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::Let {
                name,
                mutable,
                type_annotation: _,
                value,
            } => {
                if self.scope_manager.variable_exists_in_current_scope(name) {
                    return Err(format!(
                        "Error: Variable `{}` is already defined in this scope.",
                        name
                    ));
                }

                let inferred_type = if let Some(val) = value {
                    self.check_expression_initialization(val)?;
                    self.infer_and_validate_expression_immutable(val)?
                } else {
                    Ty::Int
                };

                // Phase 5: Track ownership transfers and borrows.
                if let Some(val_expr) = value {
                    match val_expr {
                        // Move semantics: let x = y (non-Copy type moves)
                        Expression::Identifier(source_name) => {
                            if !inferred_type.is_copy_type() {
                                self.scope_manager.mark_moved(source_name)?;
                            }
                        }
                        // Borrow checking: let r = &x or let r = &mut x
                        Expression::Borrow {
                            expr,
                            mutable: is_mut_borrow,
                        } => {
                            if let Expression::Identifier(source_name) = expr.as_ref() {
                                if *is_mut_borrow {
                                    self.scope_manager.add_mutable_borrow(source_name)?;
                                } else {
                                    self.scope_manager.add_immutable_borrow(source_name)?;
                                }
                            }
                        }
                        _ => {}
                    }
                }

                self.scope_manager.define_variable(
                    name.clone(),
                    inferred_type.clone(),
                    *mutable,
                    value.is_some(),
                )?;

                // Also add to old symbol table for backward compatibility
                let var_info = VariableInfo {
                    name: name.clone(),
                    ty: inferred_type.clone(),
                    mutable: *mutable,
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
            Statement::Function {
                name,
                parameters,
                body,
                return_type: _,
                type_params,
                trait_bounds,
            } => {
                // Phase 5: Register generic type parameters in scope
                if !type_params.is_empty() {
                    self.type_param_scopes.push(type_params.clone());
                }
                // Phase 5: Store trait bounds for this function
                if !trait_bounds.is_empty() {
                    self.function_bounds
                        .insert(name.clone(), trait_bounds.clone());
                }

                // Enter a new scope for the function body
                self.scope_manager.enter_function(name.clone());

                // Declare parameters as variables in the function scope
                for param in parameters {
                    let param_type = self.ast_type_to_ty(&param.param_type);
                    self.scope_manager.define_variable(
                        param.name.clone(),
                        param_type.clone(),
                        false,
                        true, // Parameters are always initialized
                    )?;
                    // Also add to old symbol table for backward compatibility
                    let var_info = VariableInfo {
                        name: param.name.clone(),
                        ty: param_type,
                        mutable: false,
                        initialized: true,
                    };
                    self.symbol_table.insert(param.name.clone(), var_info);
                }

                // Analyze each statement in the function body
                self.analyze_block(body)?;

                // Exit the function scope
                self.scope_manager.exit_function();

                // Pop generic type parameters
                if !type_params.is_empty() {
                    self.type_param_scopes.pop();
                }

                Ok(())
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.check_expression_initialization(condition)?;
                let condition_type = self.infer_and_validate_expression_immutable(condition)?;

                if condition_type != Ty::Bool {
                    return Err(format!(
                        "Error: If condition must be boolean, found: {}",
                        condition_type
                    ));
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
                    return Err(format!(
                        "Error: While condition must be boolean, found: {}",
                        condition_type
                    ));
                }

                self.scope_manager.enter_loop();
                self.analyze_block(body)?;
                self.scope_manager.exit_loop();

                Ok(())
            }
            Statement::For {
                variable,
                iterable,
                body,
            } => {
                self.check_expression_initialization(iterable)?;
                let _iterable_type = self.infer_and_validate_expression_immutable(iterable)?;

                self.scope_manager.enter_loop();
                self.scope_manager
                    .define_variable(variable.clone(), Ty::Int, false, true)?;
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
                // Phase 5: Track moves for non-Copy function call arguments
                self.track_expression_moves(expr)?;
                // Phase 5: Check trait bounds at function call sites
                self.check_trait_bounds_at_call(expr)?;
                Ok(())
            }
            Statement::Block(block) => {
                self.scope_manager.enter_scope();
                self.analyze_block(block)?;
                self.scope_manager.exit_scope();
                Ok(())
            }
            // Phase 4/5: type definitions
            Statement::StructDef { type_params, .. } | Statement::EnumDef { type_params, .. } => {
                if !type_params.is_empty() {
                    self.type_param_scopes.push(type_params.clone());
                }
                if !type_params.is_empty() {
                    self.type_param_scopes.pop();
                }
                Ok(())
            }
            Statement::TraitDef {
                name,
                type_params,
                methods,
            } => {
                // Register trait in registry with its required method names
                let required_methods: Vec<String> = methods
                    .iter()
                    .filter(|m| m.body.is_none()) // Only methods without default impl are required
                    .map(|m| m.name.clone())
                    .collect();
                self.trait_registry.insert(name.clone(), required_methods);
                if !type_params.is_empty() {
                    self.type_param_scopes.push(type_params.clone());
                }
                if !type_params.is_empty() {
                    self.type_param_scopes.pop();
                }
                Ok(())
            }
            Statement::ImplBlock {
                type_name,
                methods,
                type_params,
                trait_name,
            } => {
                if !type_params.is_empty() {
                    self.type_param_scopes.push(type_params.clone());
                }
                // Analyze method bodies
                for method in methods {
                    self.analyze_statement(method)?;
                }
                // Phase 5: Check trait completeness if this is an impl Trait for Type
                if let Some(trait_name) = trait_name {
                    // Register that this type implements this trait
                    self.trait_impls
                        .entry(type_name.clone())
                        .or_default()
                        .push(trait_name.clone());
                    // Check all required methods are implemented
                    if let Some(required_methods) = self.trait_registry.get(trait_name) {
                        let implemented: Vec<String> = methods
                            .iter()
                            .filter_map(|m| {
                                if let Statement::Function { name, .. } = m {
                                    Some(name.clone())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        for required in required_methods {
                            if !implemented.contains(required) {
                                if !type_params.is_empty() {
                                    self.type_param_scopes.pop();
                                }
                                return Err(format!(
                                    "Error: Method `{}` is required by trait `{}` but not implemented for `{}`.",
                                    required, trait_name, type_name
                                ));
                            }
                        }
                    }
                }
                if !type_params.is_empty() {
                    self.type_param_scopes.pop();
                }
                Ok(())
            }
        }
    }

    fn ast_type_to_ty(&self, ty: &crate::ast::Type) -> Ty {
        match ty {
            crate::ast::Type::Named(name) => match name.as_str() {
                "i32" | "int" => Ty::Int,
                "f64" | "float" => Ty::Float,
                "bool" => Ty::Bool,
                "String" => Ty::String,
                other => {
                    // Phase 5: Check if this is a generic type parameter
                    if self.is_type_param(other) {
                        Ty::TypeParam(other.to_string())
                    } else {
                        Ty::Struct(other.to_string())
                    }
                }
            },
            crate::ast::Type::Array(elem, size) => {
                Ty::Array(Box::new(self.ast_type_to_ty(elem)), *size)
            }
            crate::ast::Type::Tuple(types) => {
                Ty::Tuple(types.iter().map(|t| self.ast_type_to_ty(t)).collect())
            }
            crate::ast::Type::Reference(inner, mutable) => {
                Ty::Reference(Box::new(self.ast_type_to_ty(inner)), *mutable)
            }
            // Phase 6: Standard library types Option<T>, Result<T, E>, Vec<T>, HashMap<K, V>
            crate::ast::Type::Generic(name, type_args) => {
                match name.as_str() {
                    "Option" if type_args.len() == 1 => {
                        let inner_ty = self.ast_type_to_ty(&type_args[0]);
                        Ty::Option(Box::new(inner_ty))
                    }
                    "Result" if type_args.len() == 2 => {
                        let ok_ty = self.ast_type_to_ty(&type_args[0]);
                        let err_ty = self.ast_type_to_ty(&type_args[1]);
                        Ty::Result(Box::new(ok_ty), Box::new(err_ty))
                    }
                    "Vec" if type_args.len() == 1 => {
                        let elem_ty = self.ast_type_to_ty(&type_args[0]);
                        Ty::Vec(Box::new(elem_ty))
                    }
                    "HashMap" if type_args.len() == 2 => {
                        let key_ty = self.ast_type_to_ty(&type_args[0]);
                        let val_ty = self.ast_type_to_ty(&type_args[1]);
                        Ty::HashMap(Box::new(key_ty), Box::new(val_ty))
                    }
                    _ => {
                        // Other generic types - treat as type parameter for now
                        Ty::TypeParam(name.clone())
                    }
                }
            }
        }
    }

    /// Check trait bounds at function call sites.
    /// If a function has bounds like T: Display, verify the argument type implements Display.
    fn check_trait_bounds_at_call(&self, expr: &Expression) -> Result<(), String> {
        if let Expression::FunctionCall { name, arguments } = expr {
            if let Some(bounds) = self.function_bounds.get(name) {
                // For each bound, check that the corresponding argument's type
                // has the required trait implementation.
                // Simple heuristic: map type params to argument types by position.
                // Simple heuristic: map type params to argument types by position.
                for (param_name, required_traits) in bounds {
                    // Find which argument corresponds to this type param
                    // by looking at which param position uses this type param
                    if let Some((i, arg)) = arguments.iter().enumerate().next() {
                        // If the arg is at position i, and we know the function
                        // param at position i has type `param_name`, check bounds
                        let arg_type = self.infer_and_validate_expression_immutable(arg)?;
                        let type_name = match &arg_type {
                            Ty::Struct(name) => Some(name.clone()),
                            Ty::Enum(name) => Some(name.clone()),
                            _ => None,
                        };
                        if let Some(type_name) = type_name {
                            let impls = self.trait_impls.get(&type_name);
                            for required_trait in required_traits {
                                let has_impl = impls
                                    .map(|impls| impls.contains(required_trait))
                                    .unwrap_or(false);
                                if !has_impl {
                                    return Err(format!(
                                        "Error: Type `{}` does not implement trait `{}` required by `{}`.",
                                        type_name, required_trait, name
                                    ));
                                }
                            }
                        }
                        // Only check the first matching arg for simplicity
                        let _ = i;
                        let _ = param_name;
                    }
                }
            }
        }
        Ok(())
    }

    /// Track moves caused by non-Copy arguments in function calls and other expressions.
    fn track_expression_moves(&mut self, expr: &Expression) -> Result<(), String> {
        match expr {
            Expression::FunctionCall { arguments, .. } => {
                for arg in arguments {
                    if let Expression::Identifier(arg_name) = arg {
                        let arg_type = self.infer_and_validate_expression_immutable(arg)?;
                        if !arg_type.is_copy_type() {
                            self.scope_manager.mark_moved(arg_name)?;
                        }
                    }
                }
            }
            Expression::MethodCall { arguments, .. } => {
                for arg in arguments {
                    if let Expression::Identifier(arg_name) = arg {
                        let arg_type = self.infer_and_validate_expression_immutable(arg)?;
                        if !arg_type.is_copy_type() {
                            self.scope_manager.mark_moved(arg_name)?;
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
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
