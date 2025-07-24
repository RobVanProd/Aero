use crate::ast::{AstNode, Expression, Statement, Type, StructField, EnumVariant, EnumVariantData, Pattern, MatchArm};
use crate::ir::{Function, Inst, Value};
use crate::types::{Ty, needs_promotion};
use std::collections::HashMap;

pub struct IrGenerator {
    functions: HashMap<String, Function>,
    current_function_name: String,
    next_reg: u32,
    next_ptr: u32,
    symbol_table: HashMap<String, (Value, Ty)>, // Track both pointer and type
    struct_definitions: HashMap<String, StructDefinition>, // Track struct definitions
    enum_definitions: HashMap<String, EnumDefinition>, // Track enum definitions
}

#[derive(Debug, Clone)]
pub struct StructDefinition {
    pub name: String,
    pub fields: Vec<(String, String)>, // (field_name, field_type)
    pub field_indices: HashMap<String, usize>, // field_name -> index
    pub is_tuple: bool,
}

#[derive(Debug, Clone)]
pub struct EnumDefinition {
    pub name: String,
    pub variants: Vec<(String, Option<Vec<String>>)>, // (variant_name, optional_data_types)
    pub variant_indices: HashMap<String, usize>, // variant_name -> index
    pub discriminant_type: String,
}

impl IrGenerator {
    pub fn new() -> Self {
        IrGenerator {
            functions: HashMap::new(),
            current_function_name: String::new(),
            next_reg: 0,
            next_ptr: 0,
            symbol_table: HashMap::new(),
            struct_definitions: HashMap::new(),
            enum_definitions: HashMap::new(),
        }
    }

    pub fn generate_ir(&mut self, ast: Vec<AstNode>) -> HashMap<String, Function> {
        let mut main_function = Function {
            name: "main".to_string(),
            body: Vec::new(),
            next_reg: 0,
            next_ptr: 0,
        };

        for node in ast {
            match node {
                AstNode::Statement(stmt) => self.generate_statement_ir(stmt, &mut main_function),
                AstNode::Expression(_) => {
                    eprintln!("Warning: Top-level expressions are not yet handled in IR generation.");
                }
            }
        }
        
        main_function.next_reg = self.next_reg;
        main_function.next_ptr = self.next_ptr;
        self.functions.insert("main".to_string(), main_function);
        self.functions.clone()
    }

    fn generate_statement_ir(&mut self, stmt: Statement, current_function: &mut Function) {
        match stmt {
            Statement::Let { name, mutable: _, type_annotation: _, value } => {
                let (expr_value, expr_type) = if let Some(val) = value { self.generate_expression_ir(val, current_function) } else { (Value::ImmInt(0), Ty::Int) };

                // Allocate a stack slot for the variable
                let ptr_reg = Value::Reg(self.next_ptr);
                self.next_ptr += 1;
                current_function.body.push(Inst::Alloca(ptr_reg.clone(), name.clone()));
                self.symbol_table.insert(name, (ptr_reg.clone(), expr_type));

                // Store the expression result into the allocated slot
                current_function.body.push(Inst::Store(ptr_reg, expr_value));
            }
            Statement::Return(expr) => {
                let (return_value, _) = if let Some(val) = expr { self.generate_expression_ir(val, current_function) } else { (Value::ImmInt(0), Ty::Int) };
                current_function.body.push(Inst::Return(return_value));
            }
            Statement::Function { name, parameters, return_type: _, body } => {
                self.generate_function_definition_ir(name, parameters, body, current_function);
            }
            Statement::If { condition, then_block, else_block } => {
                self.generate_if_statement_ir(condition, then_block, else_block, current_function);
            }
            Statement::While { condition, body } => {
                self.generate_while_loop_ir(condition, body, current_function);
            }
            Statement::For { variable, iterable, body } => {
                self.generate_for_loop_ir(variable, iterable, body, current_function);
            }
            Statement::Loop { body } => {
                self.generate_infinite_loop_ir(body, current_function);
            }
            Statement::Break => {
                self.generate_break_ir(current_function);
            }
            Statement::Continue => {
                self.generate_continue_ir(current_function);
            }
            Statement::Expression(expr) => {
                // Generate IR for standalone expressions
                self.generate_expression_ir(expr, current_function);
            }
            Statement::Block(block) => {
                // Generate IR for block statements
                for stmt in block.statements {
                    self.generate_statement_ir(stmt, current_function);
                }
                if let Some(expr) = block.expression {
                    self.generate_expression_ir(expr, current_function);
                }
            }
            Statement::Struct { name, generics: _, fields, is_tuple } => {
                self.generate_struct_definition_ir(name, fields, is_tuple, current_function);
            }
            Statement::Enum { name, generics: _, variants } => {
                self.generate_enum_definition_ir(name, variants, current_function);
            }
            Statement::Impl { .. } => {
                // TODO: Implement impl block IR generation in Phase 4
            }
        }
    }

    fn generate_expression_ir(&mut self, expr: Expression, function: &mut Function) -> (Value, Ty) {
        match expr {
            Expression::IntegerLiteral(n) => (Value::ImmInt(n), Ty::Int),
            Expression::FloatLiteral(f) => (Value::ImmFloat(f), Ty::Float),
            Expression::Identifier(name) => {
                let (ptr_reg, var_type) = self.symbol_table.get(&name).expect("Undeclared variable").clone();
                let result_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function.body.push(Inst::Load(result_reg.clone(), ptr_reg));
                (result_reg, var_type)
            }
            Expression::Binary { op, left, right, ty } => {
                let (lhs_val, lhs_type) = self.generate_expression_ir(*left, function);
                let (rhs_val, rhs_type) = self.generate_expression_ir(*right, function);
                
                // Get the result type from the AST (set by semantic analysis)
                let result_type = ty.expect("Binary expression should have type set by semantic analysis");
                
                // Handle type promotion if needed
                let (promoted_lhs, promoted_rhs) = self.handle_type_promotion(
                    lhs_val, lhs_type, rhs_val, rhs_type, &result_type, function
                );

                // Try constant folding first
                if let (Some(folded_value), Some(folded_type)) = self.try_constant_fold(&format!("{:?}", op).to_lowercase(), &promoted_lhs, &promoted_rhs, &result_type) {
                    return (folded_value, folded_type);
                }

                let result_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                
                // Generate the appropriate instruction based on result type
                let inst = match (&result_type, format!("{:?}", op).to_lowercase().as_str()) {
                    (Ty::Int, "+") => Inst::Add(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "+") => Inst::FAdd(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Int, "-") => Inst::Sub(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "-") => Inst::FSub(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Int, "*") => Inst::Mul(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "*") => Inst::FMul(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Int, "/") => Inst::Div(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "/") => Inst::FDiv(result_reg.clone(), promoted_lhs, promoted_rhs),
                    _ => panic!("Unsupported binary operation: {} for type {:?}", op, result_type),
                };
                
                function.body.push(inst);
                (result_reg, result_type)
            }
            Expression::FunctionCall { name, arguments } => {
                // Generate IR for arguments
                let mut arg_values = Vec::new();
                for arg in arguments {
                    let (arg_value, _) = self.generate_expression_ir(arg, function);
                    arg_values.push(arg_value);
                }
                
                // Generate result register for function call
                let result_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                
                // Create function call instruction
                let call_inst = Inst::Call {
                    function: name,
                    arguments: arg_values,
                    result: Some(result_reg.clone()),
                };
                
                function.body.push(call_inst);
                
                // For now, assume function calls return int (this should be looked up from function table in semantic analysis)
                (result_reg, Ty::Int)
            }
            Expression::Print { format_string, arguments } => {
                self.generate_print_ir(format_string, arguments, false, function)
            }
            Expression::Println { format_string, arguments } => {
                self.generate_print_ir(format_string, arguments, true, function)
            }
            Expression::Comparison { op, left, right } => {
                self.generate_comparison_ir(op, *left, *right, function)
            }
            Expression::Logical { op, left, right } => {
                self.generate_logical_ir(op, *left, *right, function)
            }
            Expression::Unary { op, operand } => {
                self.generate_unary_ir(op, *operand, function)
            }
            Expression::StructLiteral { name, fields, base: _ } => {
                self.generate_struct_literal_ir(name, fields, function)
            }
            Expression::FieldAccess { object, field } => {
                self.generate_field_access_ir(*object, field, function)
            }
            Expression::Match { expression, arms } => {
                self.generate_match_expression_ir(*expression, arms, function)
            }
            Expression::MethodCall { object, method, arguments } => {
                self.generate_method_call_ir(*object, method, arguments, function)
            }
            Expression::ArrayLiteral { elements } => {
                self.generate_array_literal_ir(elements, function)
            }
            Expression::ArrayAccess { array, index } => {
                self.generate_array_access_ir(*array, *index, function)
            }
            Expression::VecMacro { elements } => {
                self.generate_vec_macro_ir(elements, function)
            }
            Expression::FormatMacro { .. } => {
                // TODO: Implement format macro IR generation in Phase 4
                (Value::ImmInt(0), Ty::Int)
            }
        }
    }

    fn handle_type_promotion(&mut self, lhs_val: Value, lhs_type: Ty, rhs_val: Value, rhs_type: Ty, target_type: &Ty, function: &mut Function) -> (Value, Value) {
        let promoted_lhs = if needs_promotion(&lhs_type, target_type) {
            let promoted_reg = Value::Reg(self.next_reg);
            self.next_reg += 1;
            function.body.push(Inst::SIToFP(promoted_reg.clone(), lhs_val));
            promoted_reg
        } else {
            lhs_val
        };

        let promoted_rhs = if needs_promotion(&rhs_type, target_type) {
            let promoted_reg = Value::Reg(self.next_reg);
            self.next_reg += 1;
            function.body.push(Inst::SIToFP(promoted_reg.clone(), rhs_val));
            promoted_reg
        } else {
            rhs_val
        };

        (promoted_lhs, promoted_rhs)
    }

    fn try_constant_fold(&self, op: &str, lhs: &Value, rhs: &Value, result_type: &Ty) -> (Option<Value>, Option<Ty>) {
        match (lhs, rhs, result_type) {
            (Value::ImmInt(l), Value::ImmInt(r), Ty::Int) => {
                let result = match op {
                    "+" => l + r,
                    "-" => l - r,
                    "*" => l * r,
                    "/" => l / r,
                    _ => return (None, None),
                };
                (Some(Value::ImmInt(result)), Some(Ty::Int))
            }
            (Value::ImmFloat(l), Value::ImmFloat(r), Ty::Float) => {
                let result = match op {
                    "+" => l + r,
                    "-" => l - r,
                    "*" => l * r,
                    "/" => l / r,
                    _ => return (None, None),
                };
                (Some(Value::ImmFloat(result)), Some(Ty::Float))
            }
            _ => (None, None),
        }
    }

    fn generate_function_definition_ir(&mut self, name: String, parameters: Vec<crate::ast::Parameter>, body: crate::ast::Block, current_function: &mut Function) {
        // Save current state
        let saved_symbol_table = self.symbol_table.clone();
        let saved_next_reg = self.next_reg;
        let saved_next_ptr = self.next_ptr;
        
        // Reset for function generation
        self.symbol_table.clear();
        self.next_reg = 0;
        self.next_ptr = 0;
        
        // Create parameter names and types for IR
        let param_names: Vec<(String, String)> = parameters.iter()
            .map(|p| (p.name.clone(), match &p.param_type {
                Type::Named(name) => name.clone(),
                // TODO: Implement proper type name generation for generic and collection types
                Type::Generic { .. } => "generic".to_string(), // Placeholder
                Type::Array { .. } => "array".to_string(), // Placeholder
                Type::Slice { .. } => "slice".to_string(), // Placeholder
                Type::Vec { .. } => "vec".to_string(), // Placeholder
                Type::HashMap { .. } => "hashmap".to_string(), // Placeholder
                Type::Reference { .. } => "reference".to_string(), // Placeholder
            }))
            .collect();
        
        // Set up parameter variables in symbol table
        for param in &parameters {
            let ptr_reg = Value::Reg(self.next_ptr);
            self.next_ptr += 1;
            
            // Convert AST Type to Ty
            let param_type = match &param.param_type {
                Type::Named(name) => match name.as_str() {
                    "i32" => Ty::Int,
                    "f64" => Ty::Float,
                    "bool" => Ty::Bool,
                    _ => Ty::Int, // Default fallback
                },
                // TODO: Implement proper type conversion for generic and collection types
                Type::Generic { .. } => Ty::Int, // Placeholder
                Type::Array { .. } => Ty::Int, // Placeholder
                Type::Slice { .. } => Ty::Int, // Placeholder
                Type::Vec { .. } => Ty::Int, // Placeholder
                Type::HashMap { .. } => Ty::Int, // Placeholder
                Type::Reference { .. } => Ty::Int, // Placeholder
            };
            
            self.symbol_table.insert(param.name.clone(), (ptr_reg, param_type));
        }
        
        // Generate function body IR
        let mut function_body = Vec::new();
        
        // Allocate parameters
        for param in &parameters {
            let (ptr_reg, _) = self.symbol_table.get(&param.name).unwrap().clone();
            function_body.push(Inst::Alloca(ptr_reg.clone(), param.name.clone()));
        }
        
        // Generate statements
        for stmt in body.statements {
            self.generate_statement_ir_for_function(stmt, &mut function_body);
        }
        
        // Handle block expression (implicit return)
        if let Some(expr) = body.expression {
            let (return_value, _) = self.generate_expression_ir_for_function(expr, &mut function_body);
            function_body.push(Inst::Return(return_value));
        } else {
            // If no explicit return, return unit/void (represented as 0 for now)
            function_body.push(Inst::Return(Value::ImmInt(0)));
        }
        
        // Create function definition instruction
        let func_def = Inst::FunctionDef {
            name: name.clone(),
            parameters: param_names,
            return_type: None, // TODO: Extract return type from AST
            body: function_body,
        };
        
        // Add function definition to current function (main)
        current_function.body.push(func_def);
        
        // Create and store function in functions map
        let function = Function {
            name: name.clone(),
            body: vec![], // The actual body is in the FunctionDef instruction
            next_reg: self.next_reg,
            next_ptr: self.next_ptr,
        };
        self.functions.insert(name, function);
        
        // Restore state
        self.symbol_table = saved_symbol_table;
        self.next_reg = saved_next_reg;
        self.next_ptr = saved_next_ptr;
    }
    
    fn generate_statement_ir_for_function(&mut self, stmt: Statement, function_body: &mut Vec<Inst>) {
        match stmt {
            Statement::Let { name, mutable: _, type_annotation: _, value } => {
                let (expr_value, expr_type) = if let Some(val) = value { self.generate_expression_ir_for_function(val, function_body) } else { (Value::ImmInt(0), Ty::Int) };

                // Allocate a stack slot for the variable
                let ptr_reg = Value::Reg(self.next_ptr);
                self.next_ptr += 1;
                function_body.push(Inst::Alloca(ptr_reg.clone(), name.clone()));
                self.symbol_table.insert(name, (ptr_reg.clone(), expr_type));

                // Store the expression result into the allocated slot
                function_body.push(Inst::Store(ptr_reg, expr_value));
            }
            Statement::Return(expr) => {
                let (return_value, _) = if let Some(val) = expr { self.generate_expression_ir_for_function(val, function_body) } else { (Value::ImmInt(0), Ty::Int) };
                function_body.push(Inst::Return(return_value));
            }
            Statement::Function { .. } => {
                // Nested functions not supported yet
                println!("Warning: Nested function definitions are not supported");
            }
            _ => {
                // Other statements not implemented yet
                println!("Warning: Statement type not yet implemented in function body");
            }
        }
    }
    
    fn generate_expression_ir_for_function(&mut self, expr: Expression, function_body: &mut Vec<Inst>) -> (Value, Ty) {
        match expr {
            Expression::IntegerLiteral(n) => (Value::ImmInt(n), Ty::Int),
            Expression::FloatLiteral(f) => (Value::ImmFloat(f), Ty::Float),
            Expression::Identifier(name) => {
                let (ptr_reg, var_type) = self.symbol_table.get(&name).expect("Undeclared variable").clone();
                let result_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function_body.push(Inst::Load(result_reg.clone(), ptr_reg));
                (result_reg, var_type)
            }
            Expression::Binary { op, left, right, ty } => {
                let (lhs_val, lhs_type) = self.generate_expression_ir_for_function(*left, function_body);
                let (rhs_val, rhs_type) = self.generate_expression_ir_for_function(*right, function_body);
                
                // Get the result type from the AST (set by semantic analysis)
                let result_type = ty.expect("Binary expression should have type set by semantic analysis");
                
                // Handle type promotion if needed
                let (promoted_lhs, promoted_rhs) = self.handle_type_promotion_for_function(
                    lhs_val, lhs_type, rhs_val, rhs_type, &result_type, function_body
                );

                // Try constant folding first
                if let (Some(folded_value), Some(folded_type)) = self.try_constant_fold(&format!("{:?}", op).to_lowercase(), &promoted_lhs, &promoted_rhs, &result_type) {
                    return (folded_value, folded_type);
                }

                let result_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                
                // Generate the appropriate instruction based on result type
                let inst = match (&result_type, format!("{:?}", op).to_lowercase().as_str()) {
                    (Ty::Int, "+") => Inst::Add(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "+") => Inst::FAdd(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Int, "-") => Inst::Sub(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "-") => Inst::FSub(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Int, "*") => Inst::Mul(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "*") => Inst::FMul(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Int, "/") => Inst::Div(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "/") => Inst::FDiv(result_reg.clone(), promoted_lhs, promoted_rhs),
                    _ => panic!("Unsupported binary operation: {} for type {:?}", op, result_type),
                };
                
                function_body.push(inst);
                (result_reg, result_type)
            }
            Expression::FunctionCall { name, arguments } => {
                self.generate_function_call_ir(name, arguments, function_body)
            }
            Expression::Print { format_string, arguments } => {
                self.generate_print_ir_for_function(format_string, arguments, false, function_body)
            }
            Expression::Println { format_string, arguments } => {
                self.generate_print_ir_for_function(format_string, arguments, true, function_body)
            }
            Expression::Comparison { op, left, right } => {
                self.generate_comparison_ir_for_function(op, *left, *right, function_body)
            }
            Expression::Logical { op, left, right } => {
                self.generate_logical_ir_for_function(op, *left, *right, function_body)
            }
            Expression::Unary { op, operand } => {
                self.generate_unary_ir_for_function(op, *operand, function_body)
            }
            Expression::StructLiteral { name, fields, base: _ } => {
                self.generate_struct_literal_ir_for_function(name, fields, function_body)
            }
            Expression::FieldAccess { object, field } => {
                self.generate_field_access_ir_for_function(*object, field, function_body)
            }
            Expression::Match { expression, arms } => {
                self.generate_match_expression_ir_for_function(*expression, arms, function_body)
            }
            Expression::MethodCall { object, method, arguments } => {
                self.generate_method_call_ir_for_function(*object, method, arguments, function_body)
            }
            Expression::ArrayLiteral { elements } => {
                self.generate_array_literal_ir_for_function(elements, function_body)
            }
            Expression::ArrayAccess { array, index } => {
                self.generate_array_access_ir_for_function(*array, *index, function_body)
            }
            Expression::VecMacro { elements } => {
                self.generate_vec_macro_ir_for_function(elements, function_body)
            }
            Expression::FormatMacro { .. } => {
                // TODO: Implement format macro IR generation in Phase 4
                (Value::ImmInt(0), Ty::Int)
            }
        }
    }
    
    fn handle_type_promotion_for_function(&mut self, lhs_val: Value, lhs_type: Ty, rhs_val: Value, rhs_type: Ty, target_type: &Ty, function_body: &mut Vec<Inst>) -> (Value, Value) {
        let promoted_lhs = if needs_promotion(&lhs_type, target_type) {
            let promoted_reg = Value::Reg(self.next_reg);
            self.next_reg += 1;
            function_body.push(Inst::SIToFP(promoted_reg.clone(), lhs_val));
            promoted_reg
        } else {
            lhs_val
        };

        let promoted_rhs = if needs_promotion(&rhs_type, target_type) {
            let promoted_reg = Value::Reg(self.next_reg);
            self.next_reg += 1;
            function_body.push(Inst::SIToFP(promoted_reg.clone(), rhs_val));
            promoted_reg
        } else {
            rhs_val
        };

        (promoted_lhs, promoted_rhs)
    }
    
    fn generate_function_call_ir(&mut self, name: String, arguments: Vec<Expression>, function_body: &mut Vec<Inst>) -> (Value, Ty) {
        // Generate IR for arguments
        let mut arg_values = Vec::new();
        for arg in arguments {
            let (arg_value, _) = self.generate_expression_ir_for_function(arg, function_body);
            arg_values.push(arg_value);
        }
        
        // Generate result register for function call
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        // Create function call instruction
        let call_inst = Inst::Call {
            function: name,
            arguments: arg_values,
            result: Some(result_reg.clone()),
        };
        
        function_body.push(call_inst);
        
        // For now, assume function calls return int (this should be looked up from function table in semantic analysis)
        (result_reg, Ty::Int)
    }

    // Control flow IR generation methods
    fn generate_if_statement_ir(&mut self, condition: Expression, then_block: crate::ast::Block, else_block: Option<Box<Statement>>, current_function: &mut Function) {
        // Generate condition evaluation
        let (cond_value, _) = self.generate_expression_ir(condition, current_function);
        
        // Generate unique labels
        let then_label = format!("if_then_{}", self.next_reg);
        self.next_reg += 1;
        let else_label = format!("if_else_{}", self.next_reg);
        self.next_reg += 1;
        let end_label = format!("if_end_{}", self.next_reg);
        self.next_reg += 1;
        
        // Branch based on condition
        current_function.body.push(Inst::Branch {
            condition: cond_value,
            true_label: then_label.clone(),
            false_label: else_label.clone(),
        });
        
        // Generate then block
        current_function.body.push(Inst::Label(then_label));
        for stmt in then_block.statements {
            self.generate_statement_ir(stmt, current_function);
        }
        if let Some(expr) = then_block.expression {
            self.generate_expression_ir(expr, current_function);
        }
        current_function.body.push(Inst::Jump(end_label.clone()));
        
        // Generate else block
        current_function.body.push(Inst::Label(else_label));
        if let Some(else_stmt) = else_block {
            self.generate_statement_ir(*else_stmt, current_function);
        }
        current_function.body.push(Inst::Jump(end_label.clone()));
        
        // End label
        current_function.body.push(Inst::Label(end_label));
    }
    
    fn generate_while_loop_ir(&mut self, condition: Expression, body: crate::ast::Block, current_function: &mut Function) {
        // Generate unique labels
        let loop_start = format!("while_start_{}", self.next_reg);
        self.next_reg += 1;
        let loop_body = format!("while_body_{}", self.next_reg);
        self.next_reg += 1;
        let loop_end = format!("while_end_{}", self.next_reg);
        self.next_reg += 1;
        
        // Jump to loop start
        current_function.body.push(Inst::Jump(loop_start.clone()));
        
        // Loop start - evaluate condition
        current_function.body.push(Inst::Label(loop_start.clone()));
        let (cond_value, _) = self.generate_expression_ir(condition, current_function);
        current_function.body.push(Inst::Branch {
            condition: cond_value,
            true_label: loop_body.clone(),
            false_label: loop_end.clone(),
        });
        
        // Loop body
        current_function.body.push(Inst::Label(loop_body));
        for stmt in body.statements {
            self.generate_statement_ir(stmt, current_function);
        }
        if let Some(expr) = body.expression {
            self.generate_expression_ir(expr, current_function);
        }
        current_function.body.push(Inst::Jump(loop_start));
        
        // Loop end
        current_function.body.push(Inst::Label(loop_end));
    }
    
    fn generate_for_loop_ir(&mut self, variable: String, iterable: Expression, body: crate::ast::Block, current_function: &mut Function) {
        // For now, implement a simple for loop assuming iterable is a range
        // This is a simplified implementation - a full implementation would need range support
        
        // Generate unique labels
        let loop_start = format!("for_start_{}", self.next_reg);
        self.next_reg += 1;
        let loop_body = format!("for_body_{}", self.next_reg);
        self.next_reg += 1;
        let loop_end = format!("for_end_{}", self.next_reg);
        self.next_reg += 1;
        
        // Initialize loop variable (simplified - assumes iterable evaluates to start value)
        let (start_value, var_type) = self.generate_expression_ir(iterable, current_function);
        let var_ptr = Value::Reg(self.next_ptr);
        self.next_ptr += 1;
        current_function.body.push(Inst::Alloca(var_ptr.clone(), variable.clone()));
        current_function.body.push(Inst::Store(var_ptr.clone(), start_value));
        self.symbol_table.insert(variable.clone(), (var_ptr.clone(), var_type));
        
        // For simplicity, create a condition that will eventually be false
        // In a real implementation, this would check against the range end
        current_function.body.push(Inst::Jump(loop_start.clone()));
        
        // Loop start - check condition (simplified)
        current_function.body.push(Inst::Label(loop_start.clone()));
        let loop_var_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        current_function.body.push(Inst::Load(loop_var_reg.clone(), var_ptr.clone()));
        
        // Simple condition: loop while var < 10 (this should be configurable)
        let limit_value = Value::ImmInt(10);
        let cond_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        // This would need a comparison instruction - for now use a placeholder
        current_function.body.push(Inst::Branch {
            condition: cond_reg,
            true_label: loop_body.clone(),
            false_label: loop_end.clone(),
        });
        
        // Loop body
        current_function.body.push(Inst::Label(loop_body));
        for stmt in body.statements {
            self.generate_statement_ir(stmt, current_function);
        }
        if let Some(expr) = body.expression {
            self.generate_expression_ir(expr, current_function);
        }
        
        // Increment loop variable
        let incremented_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        current_function.body.push(Inst::Add(incremented_reg.clone(), loop_var_reg, Value::ImmInt(1)));
        current_function.body.push(Inst::Store(var_ptr, incremented_reg));
        
        current_function.body.push(Inst::Jump(loop_start));
        
        // Loop end
        current_function.body.push(Inst::Label(loop_end));
    }
    
    fn generate_infinite_loop_ir(&mut self, body: crate::ast::Block, current_function: &mut Function) {
        // Generate unique labels
        let loop_start = format!("loop_start_{}", self.next_reg);
        self.next_reg += 1;
        
        // Jump to loop start
        current_function.body.push(Inst::Jump(loop_start.clone()));
        
        // Loop start
        current_function.body.push(Inst::Label(loop_start.clone()));
        
        // Loop body
        for stmt in body.statements {
            self.generate_statement_ir(stmt, current_function);
        }
        if let Some(expr) = body.expression {
            self.generate_expression_ir(expr, current_function);
        }
        
        // Jump back to start (infinite loop)
        current_function.body.push(Inst::Jump(loop_start));
    }
    
    fn generate_break_ir(&mut self, current_function: &mut Function) {
        // For now, generate a placeholder jump
        // In a real implementation, this would jump to the appropriate loop end label
        // This requires maintaining a stack of loop labels
        let break_label = "loop_end_placeholder".to_string();
        current_function.body.push(Inst::Jump(break_label));
    }
    
    fn generate_continue_ir(&mut self, current_function: &mut Function) {
        // For now, generate a placeholder jump
        // In a real implementation, this would jump to the appropriate loop start label
        // This requires maintaining a stack of loop labels
        let continue_label = "loop_start_placeholder".to_string();
        current_function.body.push(Inst::Jump(continue_label));
    }

    // I/O and enhanced expression IR generation methods
    fn generate_print_ir(&mut self, format_string: String, arguments: Vec<Expression>, newline: bool, function: &mut Function) -> (Value, Ty) {
        // Generate IR for arguments
        let mut arg_values = Vec::new();
        for arg in arguments {
            let (arg_value, _) = self.generate_expression_ir(arg, function);
            arg_values.push(arg_value);
        }
        
        // Modify format string to add newline if needed
        let final_format = if newline {
            format!("{}\n", format_string)
        } else {
            format_string
        };
        
        // Create print instruction
        let print_inst = Inst::Print {
            format_string: final_format,
            arguments: arg_values,
        };
        
        function.body.push(print_inst);
        
        // Print operations return unit type (represented as 0 for now)
        (Value::ImmInt(0), Ty::Int)
    }
    
    fn generate_comparison_ir(&mut self, op: crate::ast::ComparisonOp, left: Expression, right: Expression, function: &mut Function) -> (Value, Ty) {
        let (left_val, left_type) = self.generate_expression_ir(left, function);
        let (right_val, right_type) = self.generate_expression_ir(right, function);
        
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        // Convert ComparisonOp to string for IR instruction
        let op_str = match op {
            crate::ast::ComparisonOp::Equal => "eq",
            crate::ast::ComparisonOp::NotEqual => "ne",
            crate::ast::ComparisonOp::LessThan => "slt",
            crate::ast::ComparisonOp::GreaterThan => "sgt",
            crate::ast::ComparisonOp::LessEqual => "sle",
            crate::ast::ComparisonOp::GreaterEqual => "sge",
        };
        
        // Generate appropriate comparison instruction based on operand types
        let inst = match (&left_type, &right_type) {
            (Ty::Int, Ty::Int) => Inst::ICmp {
                op: op_str.to_string(),
                result: result_reg.clone(),
                left: left_val,
                right: right_val,
            },
            (Ty::Float, Ty::Float) => {
                // Convert integer comparison ops to float comparison ops
                let float_op = match op_str {
                    "eq" => "oeq",
                    "ne" => "one", 
                    "slt" => "olt",
                    "sgt" => "ogt",
                    "sle" => "ole",
                    "sge" => "oge",
                    _ => op_str,
                };
                Inst::FCmp {
                    op: float_op.to_string(),
                    result: result_reg.clone(),
                    left: left_val,
                    right: right_val,
                }
            },
            (Ty::Int, Ty::Float) => {
                // Promote left operand to float
                let promoted_left = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function.body.push(Inst::SIToFP(promoted_left.clone(), left_val));
                
                let float_op = match op_str {
                    "eq" => "oeq",
                    "ne" => "one",
                    "slt" => "olt", 
                    "sgt" => "ogt",
                    "sle" => "ole",
                    "sge" => "oge",
                    _ => op_str,
                };
                Inst::FCmp {
                    op: float_op.to_string(),
                    result: result_reg.clone(),
                    left: promoted_left,
                    right: right_val,
                }
            },
            (Ty::Float, Ty::Int) => {
                // Promote right operand to float
                let promoted_right = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function.body.push(Inst::SIToFP(promoted_right.clone(), right_val));
                
                let float_op = match op_str {
                    "eq" => "oeq",
                    "ne" => "one",
                    "slt" => "olt",
                    "sgt" => "ogt", 
                    "sle" => "ole",
                    "sge" => "oge",
                    _ => op_str,
                };
                Inst::FCmp {
                    op: float_op.to_string(),
                    result: result_reg.clone(),
                    left: left_val,
                    right: promoted_right,
                }
            },
            (Ty::Bool, Ty::Bool) => Inst::ICmp {
                op: op_str.to_string(),
                result: result_reg.clone(),
                left: left_val,
                right: right_val,
            },
            _ => panic!("Unsupported comparison between {:?} and {:?}", left_type, right_type),
        };
        
        function.body.push(inst);
        (result_reg, Ty::Bool)
    }
    
    fn generate_logical_ir(&mut self, op: crate::ast::LogicalOp, left: Expression, right: Expression, function: &mut Function) -> (Value, Ty) {
        let (left_val, _) = self.generate_expression_ir(left, function);
        let (right_val, _) = self.generate_expression_ir(right, function);
        
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        let inst = match op {
            crate::ast::LogicalOp::And => Inst::And {
                result: result_reg.clone(),
                left: left_val,
                right: right_val,
            },
            crate::ast::LogicalOp::Or => Inst::Or {
                result: result_reg.clone(),
                left: left_val,
                right: right_val,
            },
        };
        
        function.body.push(inst);
        (result_reg, Ty::Bool)
    }
    
    fn generate_unary_ir(&mut self, op: crate::ast::UnaryOp, operand: Expression, function: &mut Function) -> (Value, Ty) {
        let (operand_val, operand_type) = self.generate_expression_ir(operand, function);
        
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        let (inst, result_type) = match op {
            crate::ast::UnaryOp::Not => {
                (Inst::Not {
                    result: result_reg.clone(),
                    operand: operand_val,
                }, Ty::Bool)
            },
            crate::ast::UnaryOp::Negate => {
                (Inst::Neg {
                    result: result_reg.clone(),
                    operand: operand_val,
                }, operand_type)
            },
        };
        
        function.body.push(inst);
        (result_reg, result_type)
    }

    // Function-level I/O and enhanced expression IR generation methods
    fn generate_print_ir_for_function(&mut self, format_string: String, arguments: Vec<Expression>, newline: bool, function_body: &mut Vec<Inst>) -> (Value, Ty) {
        // Generate IR for arguments
        let mut arg_values = Vec::new();
        for arg in arguments {
            let (arg_value, _) = self.generate_expression_ir_for_function(arg, function_body);
            arg_values.push(arg_value);
        }
        
        // Modify format string to add newline if needed
        let final_format = if newline {
            format!("{}\n", format_string)
        } else {
            format_string
        };
        
        // Create print instruction
        let print_inst = Inst::Print {
            format_string: final_format,
            arguments: arg_values,
        };
        
        function_body.push(print_inst);
        
        // Print operations return unit type (represented as 0 for now)
        (Value::ImmInt(0), Ty::Int)
    }
    
    fn generate_comparison_ir_for_function(&mut self, op: crate::ast::ComparisonOp, left: Expression, right: Expression, function_body: &mut Vec<Inst>) -> (Value, Ty) {
        let (left_val, left_type) = self.generate_expression_ir_for_function(left, function_body);
        let (right_val, right_type) = self.generate_expression_ir_for_function(right, function_body);
        
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        // Convert ComparisonOp to string for IR instruction
        let op_str = match op {
            crate::ast::ComparisonOp::Equal => "eq",
            crate::ast::ComparisonOp::NotEqual => "ne",
            crate::ast::ComparisonOp::LessThan => "slt",
            crate::ast::ComparisonOp::GreaterThan => "sgt",
            crate::ast::ComparisonOp::LessEqual => "sle",
            crate::ast::ComparisonOp::GreaterEqual => "sge",
        };
        
        // Generate appropriate comparison instruction based on operand types
        let inst = match (&left_type, &right_type) {
            (Ty::Int, Ty::Int) => Inst::ICmp {
                op: op_str.to_string(),
                result: result_reg.clone(),
                left: left_val,
                right: right_val,
            },
            (Ty::Float, Ty::Float) => {
                // Convert integer comparison ops to float comparison ops
                let float_op = match op_str {
                    "eq" => "oeq",
                    "ne" => "one", 
                    "slt" => "olt",
                    "sgt" => "ogt",
                    "sle" => "ole",
                    "sge" => "oge",
                    _ => op_str,
                };
                Inst::FCmp {
                    op: float_op.to_string(),
                    result: result_reg.clone(),
                    left: left_val,
                    right: right_val,
                }
            },
            (Ty::Int, Ty::Float) => {
                // Promote left operand to float
                let promoted_left = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function_body.push(Inst::SIToFP(promoted_left.clone(), left_val));
                
                let float_op = match op_str {
                    "eq" => "oeq",
                    "ne" => "one",
                    "slt" => "olt", 
                    "sgt" => "ogt",
                    "sle" => "ole",
                    "sge" => "oge",
                    _ => op_str,
                };
                Inst::FCmp {
                    op: float_op.to_string(),
                    result: result_reg.clone(),
                    left: promoted_left,
                    right: right_val,
                }
            },
            (Ty::Float, Ty::Int) => {
                // Promote right operand to float
                let promoted_right = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function_body.push(Inst::SIToFP(promoted_right.clone(), right_val));
                
                let float_op = match op_str {
                    "eq" => "oeq",
                    "ne" => "one",
                    "slt" => "olt",
                    "sgt" => "ogt", 
                    "sle" => "ole",
                    "sge" => "oge",
                    _ => op_str,
                };
                Inst::FCmp {
                    op: float_op.to_string(),
                    result: result_reg.clone(),
                    left: left_val,
                    right: promoted_right,
                }
            },
            (Ty::Bool, Ty::Bool) => Inst::ICmp {
                op: op_str.to_string(),
                result: result_reg.clone(),
                left: left_val,
                right: right_val,
            },
            _ => panic!("Unsupported comparison between {:?} and {:?}", left_type, right_type),
        };
        
        function_body.push(inst);
        (result_reg, Ty::Bool)
    }
    
    fn generate_logical_ir_for_function(&mut self, op: crate::ast::LogicalOp, left: Expression, right: Expression, function_body: &mut Vec<Inst>) -> (Value, Ty) {
        let (left_val, _) = self.generate_expression_ir_for_function(left, function_body);
        let (right_val, _) = self.generate_expression_ir_for_function(right, function_body);
        
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        let inst = match op {
            crate::ast::LogicalOp::And => Inst::And {
                result: result_reg.clone(),
                left: left_val,
                right: right_val,
            },
            crate::ast::LogicalOp::Or => Inst::Or {
                result: result_reg.clone(),
                left: left_val,
                right: right_val,
            },
        };
        
        function_body.push(inst);
        (result_reg, Ty::Bool)
    }
    
    fn generate_unary_ir_for_function(&mut self, op: crate::ast::UnaryOp, operand: Expression, function_body: &mut Vec<Inst>) -> (Value, Ty) {
        let (operand_val, operand_type) = self.generate_expression_ir_for_function(operand, function_body);
        
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        let (inst, result_type) = match op {
            crate::ast::UnaryOp::Not => {
                (Inst::Not {
                    result: result_reg.clone(),
                    operand: operand_val,
                }, Ty::Bool)
            },
            crate::ast::UnaryOp::Negate => {
                (Inst::Neg {
                    result: result_reg.clone(),
                    operand: operand_val,
                }, operand_type)
            },
        };
        
        function_body.push(inst);
        (result_reg, result_type)
    }

    // Struct IR generation methods
    fn generate_struct_definition_ir(&mut self, name: String, fields: Vec<StructField>, is_tuple: bool, current_function: &mut Function) {
        // Convert AST StructField to IR field representation
        let ir_fields: Vec<(String, String)> = fields.iter().enumerate().map(|(i, field)| {
            let field_name = if is_tuple {
                i.to_string() // Use index as field name for tuple structs
            } else {
                field.name.clone()
            };
            let field_type = self.type_to_string(&field.field_type);
            (field_name, field_type)
        }).collect();
        
        // Create field index mapping
        let mut field_indices = HashMap::new();
        for (i, (field_name, _)) in ir_fields.iter().enumerate() {
            field_indices.insert(field_name.clone(), i);
        }
        
        // Store struct definition
        let struct_def = StructDefinition {
            name: name.clone(),
            fields: ir_fields.clone(),
            field_indices,
            is_tuple,
        };
        self.struct_definitions.insert(name.clone(), struct_def);
        
        // Generate struct definition IR instruction
        let struct_def_inst = Inst::StructDef {
            name,
            fields: ir_fields,
            is_tuple,
        };
        current_function.body.push(struct_def_inst);
    }
    
    fn generate_struct_literal_ir(&mut self, name: String, fields: Vec<(String, Expression)>, function: &mut Function) -> (Value, Ty) {
        // Get struct definition
        let struct_def = self.struct_definitions.get(&name)
            .expect(&format!("Undefined struct: {}", name))
            .clone();
        
        // Allocate memory for the struct
        let struct_ptr = Value::Reg(self.next_ptr);
        self.next_ptr += 1;
        function.body.push(Inst::StructAlloca {
            result: struct_ptr.clone(),
            struct_name: name.clone(),
        });
        
        // Generate field values and create field initialization list
        let mut field_values = Vec::new();
        for (field_name, field_expr) in fields {
            let (field_value, _) = self.generate_expression_ir(field_expr, function);
            field_values.push((field_name, field_value));
        }
        
        // Initialize struct with field values
        function.body.push(Inst::StructInit {
            result: struct_ptr.clone(),
            struct_name: name.clone(),
            field_values,
        });
        
        // Return struct pointer and custom struct type
        (struct_ptr, Ty::Struct(name))
    }
    
    fn generate_field_access_ir(&mut self, object: Expression, field: String, function: &mut Function) -> (Value, Ty) {
        // Generate IR for the object expression
        let (object_value, object_type) = self.generate_expression_ir(object, function);
        
        // Extract struct name from type
        let struct_name = match object_type {
            Ty::Struct(name) => name,
            _ => panic!("Field access on non-struct type: {:?}", object_type),
        };
        
        // Get struct definition and field index
        let struct_def = self.struct_definitions.get(&struct_name)
            .expect(&format!("Undefined struct: {}", struct_name))
            .clone();
        
        let field_index = *struct_def.field_indices.get(&field)
            .expect(&format!("Field '{}' not found in struct '{}'", field, struct_name));
        
        // Generate field access instruction
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        function.body.push(Inst::FieldAccess {
            result: result_reg.clone(),
            struct_ptr: object_value,
            field_name: field.clone(),
            field_index,
        });
        
        // Look up actual field type from struct definition
        let field_type = self.get_field_type(&struct_def, &field);
        
        (result_reg, field_type)
    }
    
    fn generate_struct_copy_ir(&mut self, source: Value, struct_name: String, function: &mut Function) -> Value {
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        function.body.push(Inst::StructCopy {
            result: result_reg.clone(),
            source,
            struct_name,
        });
        
        result_reg
    }

    // Struct IR generation methods for function bodies
    fn generate_struct_literal_ir_for_function(&mut self, name: String, fields: Vec<(String, Expression)>, function_body: &mut Vec<Inst>) -> (Value, Ty) {
        // Get struct definition
        let struct_def = self.struct_definitions.get(&name)
            .expect(&format!("Undefined struct: {}", name))
            .clone();
        
        // Allocate memory for the struct
        let struct_ptr = Value::Reg(self.next_ptr);
        self.next_ptr += 1;
        function_body.push(Inst::StructAlloca {
            result: struct_ptr.clone(),
            struct_name: name.clone(),
        });
        
        // Generate field values and create field initialization list
        let mut field_values = Vec::new();
        for (field_name, field_expr) in fields {
            let (field_value, _) = self.generate_expression_ir_for_function(field_expr, function_body);
            field_values.push((field_name, field_value));
        }
        
        // Initialize struct with field values
        function_body.push(Inst::StructInit {
            result: struct_ptr.clone(),
            struct_name: name.clone(),
            field_values,
        });
        
        // Return struct pointer and custom struct type
        (struct_ptr, Ty::Struct(name))
    }
    
    fn generate_field_access_ir_for_function(&mut self, object: Expression, field: String, function_body: &mut Vec<Inst>) -> (Value, Ty) {
        // Generate IR for the object expression
        let (object_value, object_type) = self.generate_expression_ir_for_function(object, function_body);
        
        // Extract struct name from type
        let struct_name = match object_type {
            Ty::Struct(name) => name,
            _ => panic!("Field access on non-struct type: {:?}", object_type),
        };
        
        // Get struct definition and field index
        let struct_def = self.struct_definitions.get(&struct_name)
            .expect(&format!("Undefined struct: {}", struct_name))
            .clone();
        
        let field_index = *struct_def.field_indices.get(&field)
            .expect(&format!("Field '{}' not found in struct '{}'", field, struct_name));
        
        // Generate field access instruction
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        function_body.push(Inst::FieldAccess {
            result: result_reg.clone(),
            struct_ptr: object_value,
            field_name: field.clone(),
            field_index,
        });
        
        // Look up actual field type from struct definition
        let field_type = self.get_field_type(&struct_def, &field);
        
        (result_reg, field_type)
    }

    // Helper method to get field type from struct definition
    fn get_field_type(&self, struct_def: &StructDefinition, field_name: &str) -> Ty {
        // Find the field in the struct definition
        for (name, type_str) in &struct_def.fields {
            if name == field_name {
                // Convert type string back to Ty
                return self.string_to_ty(type_str);
            }
        }
        
        // Fallback to Int if field not found (should not happen if semantic analysis is correct)
        Ty::Int
    }
    
    // Helper method to convert type string back to Ty
    fn string_to_ty(&self, type_str: &str) -> Ty {
        match type_str {
            "i32" | "int" => Ty::Int,
            "f64" | "float" => Ty::Float,
            "bool" => Ty::Bool,
            "String" => Ty::String,
            s if s.starts_with("Vec<") => {
                // Parse Vec<T> type
                let inner = &s[4..s.len()-1];
                Ty::Vec(Box::new(self.string_to_ty(inner)))
            }
            s if s.starts_with("[") && s.contains(";") => {
                // Parse [T; N] array type
                let parts: Vec<&str> = s[1..s.len()-1].split(";").collect();
                if parts.len() == 2 {
                    let element_type = self.string_to_ty(parts[0].trim());
                    if let Ok(size) = parts[1].trim().parse::<usize>() {
                        return Ty::Array(Box::new(element_type), Some(size));
                    }
                }
                Ty::Int // Fallback
            }
            s if s.starts_with("[") && s.ends_with("]") => {
                // Parse [T] slice type
                let inner = &s[1..s.len()-1];
                Ty::Array(Box::new(self.string_to_ty(inner)), None)
            }
            s if s.starts_with("&") => {
                // Parse reference type
                let inner = &s[1..];
                Ty::Reference(Box::new(self.string_to_ty(inner)))
            }
            _ => {
                // Check if it's a struct type
                if self.struct_definitions.contains_key(type_str) {
                    Ty::Struct(type_str.to_string())
                } else {
                    // Default fallback
                    Ty::Int
                }
            }
        }
    }

    // Method call IR generation methods
    fn generate_method_call_ir(&mut self, object: Expression, method: String, arguments: Vec<Expression>, function: &mut Function) -> (Value, Ty) {
        // Generate IR for the object expression
        let (object_value, object_type) = self.generate_expression_ir(object, function);
        
        // Generate IR for arguments
        let mut arg_values = Vec::new();
        for arg in arguments {
            let (arg_value, _) = self.generate_expression_ir(arg, function);
            arg_values.push(arg_value);
        }
        
        // Create method call - for now, treat as function call with object as first argument
        let mut all_args = vec![object_value];
        all_args.extend(arg_values);
        
        // Generate result register for method call
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        // Create method call instruction (using function call for now)
        let method_name = match &object_type {
            Ty::Struct(struct_name) => format!("{}::{}", struct_name, method),
            _ => format!("{}::{}", object_type.to_string(), method),
        };
        
        let call_inst = Inst::Call {
            function: method_name,
            arguments: all_args,
            result: Some(result_reg.clone()),
        };
        
        function.body.push(call_inst);
        
        // For now, assume method calls return the same type as the object
        (result_reg, object_type)
    }
    
    fn generate_method_call_ir_for_function(&mut self, object: Expression, method: String, arguments: Vec<Expression>, function_body: &mut Vec<Inst>) -> (Value, Ty) {
        // Generate IR for the object expression
        let (object_value, object_type) = self.generate_expression_ir_for_function(object, function_body);
        
        // Generate IR for arguments
        let mut arg_values = Vec::new();
        for arg in arguments {
            let (arg_value, _) = self.generate_expression_ir_for_function(arg, function_body);
            arg_values.push(arg_value);
        }
        
        // Create method call - for now, treat as function call with object as first argument
        let mut all_args = vec![object_value];
        all_args.extend(arg_values);
        
        // Generate result register for method call
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        // Create method call instruction (using function call for now)
        let method_name = match &object_type {
            Ty::Struct(struct_name) => format!("{}::{}", struct_name, method),
            _ => format!("{}::{}", object_type.to_string(), method),
        };
        
        let call_inst = Inst::Call {
            function: method_name,
            arguments: all_args,
            result: Some(result_reg.clone()),
        };
        
        function_body.push(call_inst);
        
        // For now, assume method calls return the same type as the object
        (result_reg, object_type)
    }

    // Enhanced struct IR generation with field modification support
    fn generate_field_store_ir(&mut self, object: Expression, field: String, value: Expression, function: &mut Function) -> (Value, Ty) {
        // Generate IR for the object expression (should be a pointer)
        let (object_value, object_type) = self.generate_expression_ir(object, function);
        
        // Generate IR for the value to store
        let (value_reg, value_type) = self.generate_expression_ir(value, function);
        
        // Extract struct name from type
        let struct_name = match object_type {
            Ty::Struct(name) => name,
            _ => panic!("Field store on non-struct type: {:?}", object_type),
        };
        
        // Get struct definition and field index
        let struct_def = self.struct_definitions.get(&struct_name)
            .expect(&format!("Undefined struct: {}", struct_name))
            .clone();
        
        let field_index = *struct_def.field_indices.get(&field)
            .expect(&format!("Field '{}' not found in struct '{}'", field, struct_name));
        
        // Generate field store instruction
        function.body.push(Inst::FieldStore {
            struct_ptr: object_value.clone(),
            field_name: field,
            field_index,
            value: value_reg,
        });
        
        // Return the stored value and its type
        (value_reg, value_type)
    }

    // Struct copy and move semantics
    fn generate_struct_move_ir(&mut self, source: Value, struct_name: String, function: &mut Function) -> Value {
        // For now, implement move as copy (proper move semantics would invalidate source)
        self.generate_struct_copy_ir(source, struct_name, function)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{AstNode, Statement, Expression, Parameter, Block, Type};
    use crate::ir::{Inst, Value};
    use crate::types::Ty;

    #[test]
    fn test_function_definition_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a simple function: fn add(a: i32, b: i32) -> i32 { a + b }
        let param1 = Parameter {
            name: "a".to_string(),
            param_type: Type { name: "i32".to_string() },
        };
        let param2 = Parameter {
            name: "b".to_string(),
            param_type: Type { name: "i32".to_string() },
        };
        
        let body = Block {
            statements: vec![],
            expression: Some(Expression::Binary {
                op: "+".to_string(),
                lhs: Box::new(Expression::Identifier("a".to_string())),
                rhs: Box::new(Expression::Identifier("b".to_string())),
                ty: Some(Ty::Int),
            }),
        };
        
        let func_stmt = Statement::Function {
            name: "add".to_string(),
            parameters: vec![param1, param2],
            return_type: Some(Type { name: "i32".to_string() }),
            body,
        };
        
        let ast = vec![AstNode::Statement(func_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        // Check that main function exists
        assert!(ir.contains_key("main"));
        let main_func = &ir["main"];
        
        // Check that main function contains a FunctionDef
        assert!(!main_func.body.is_empty());
        
        // Check that the first instruction is a FunctionDef
        match &main_func.body[0] {
            Inst::FunctionDef { name, parameters, return_type, body } => {
                assert_eq!(name, "add");
                assert_eq!(parameters.len(), 2);
                assert_eq!(parameters[0].0, "a");
                assert_eq!(parameters[1].0, "b");
                assert!(return_type.is_none());
                assert!(!body.is_empty());
            }
            _ => panic!("Expected FunctionDef instruction"),
        }
    }

    #[test]
    fn test_print_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a print expression: print!("Hello, {}!", name)
        let print_expr = Expression::Print {
            format_string: "Hello, {}!".to_string(),
            arguments: vec![Expression::Identifier("name".to_string())],
        };
        
        // Set up symbol table for the identifier
        ir_gen.symbol_table.insert("name".to_string(), (Value::Reg(0), Ty::Int));
        
        let mut function = Function {
            name: "test".to_string(),
            body: Vec::new(),
            next_reg: 1,
            next_ptr: 1,
        };
        
        let (result_val, result_type) = ir_gen.generate_expression_ir(print_expr, &mut function);
        
        // Print should return unit type (represented as 0)
        assert_eq!(result_val, Value::ImmInt(0));
        assert_eq!(result_type, Ty::Int);
        
        // Check that the function body contains Load and Print instructions
        assert_eq!(function.body.len(), 2);
        
        // First instruction should be Load for the identifier
        match &function.body[0] {
            Inst::Load(result_reg, ptr_reg) => {
                assert_eq!(*result_reg, Value::Reg(0));
                assert_eq!(*ptr_reg, Value::Reg(0));
            }
            _ => panic!("Expected Load instruction"),
        }
        
        // Second instruction should be Print
        match &function.body[1] {
            Inst::Print { format_string, arguments } => {
                assert_eq!(format_string, "Hello, {}!");
                assert_eq!(arguments.len(), 1);
                assert_eq!(arguments[0], Value::Reg(0));
            }
            _ => panic!("Expected Print instruction"),
        }
    }

    #[test]
    fn test_println_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a println expression: println!("Value: {}", 42)
        let println_expr = Expression::Println {
            format_string: "Value: {}".to_string(),
            arguments: vec![Expression::IntegerLiteral(42)],
        };
        
        let mut function = Function {
            name: "test".to_string(),
            body: Vec::new(),
            next_reg: 0,
            next_ptr: 0,
        };
        
        let (result_val, result_type) = ir_gen.generate_expression_ir(println_expr, &mut function);
        
        // Println should return unit type (represented as 0)
        assert_eq!(result_val, Value::ImmInt(0));
        assert_eq!(result_type, Ty::Int);
        
        // Check that the function body contains Print instruction with newline
        assert_eq!(function.body.len(), 1);
        
        match &function.body[0] {
            Inst::Print { format_string, arguments } => {
                assert_eq!(format_string, "Value: {}\n");
                assert_eq!(arguments.len(), 1);
                assert_eq!(arguments[0], Value::ImmInt(42));
            }
            _ => panic!("Expected Print instruction"),
        }
    }

    #[test]
    fn test_comparison_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a comparison expression: x == 5
        let comparison_expr = Expression::Comparison {
            op: crate::ast::ComparisonOp::Equal,
            left: Box::new(Expression::Identifier("x".to_string())),
            right: Box::new(Expression::IntegerLiteral(5)),
        };
        
        // Set up symbol table for the identifier
        ir_gen.symbol_table.insert("x".to_string(), (Value::Reg(0), Ty::Int));
        
        let mut function = Function {
            name: "test".to_string(),
            body: Vec::new(),
            next_reg: 1,
            next_ptr: 1,
        };
        
        let (result_val, result_type) = ir_gen.generate_expression_ir(comparison_expr, &mut function);
        
        // Comparison should return boolean type
        assert_eq!(result_type, Ty::Bool);
        assert_eq!(result_val, Value::Reg(1));
        
        // Check that the function body contains Load and ICmp instructions
        assert_eq!(function.body.len(), 2);
        
        // First instruction should be Load for the identifier
        match &function.body[0] {
            Inst::Load(result_reg, ptr_reg) => {
                assert_eq!(*result_reg, Value::Reg(0));
                assert_eq!(*ptr_reg, Value::Reg(0));
            }
            _ => panic!("Expected Load instruction"),
        }
        
        // Second instruction should be ICmp
        match &function.body[1] {
            Inst::ICmp { op, result, left, right } => {
                assert_eq!(op, "eq");
                assert_eq!(*result, Value::Reg(1));
                assert_eq!(*left, Value::Reg(0));
                assert_eq!(*right, Value::ImmInt(5));
            }
            _ => panic!("Expected ICmp instruction"),
        }
    }

    #[test]
    fn test_float_comparison_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a float comparison expression: x > 3.14
        let comparison_expr = Expression::Comparison {
            op: crate::ast::ComparisonOp::GreaterThan,
            left: Box::new(Expression::Identifier("x".to_string())),
            right: Box::new(Expression::FloatLiteral(3.14)),
        };
        
        // Set up symbol table for the identifier (float type)
        ir_gen.symbol_table.insert("x".to_string(), (Value::Reg(0), Ty::Float));
        
        let mut function = Function {
            name: "test".to_string(),
            body: Vec::new(),
            next_reg: 1,
            next_ptr: 1,
        };
        
        let (result_val, result_type) = ir_gen.generate_expression_ir(comparison_expr, &mut function);
        
        // Comparison should return boolean type
        assert_eq!(result_type, Ty::Bool);
        assert_eq!(result_val, Value::Reg(2));
        
        // Check that the function body contains Load and FCmp instructions
        assert_eq!(function.body.len(), 2);
        
        // Second instruction should be FCmp
        match &function.body[1] {
            Inst::FCmp { op, result, left, right } => {
                assert_eq!(op, "ogt");
                assert_eq!(*result, Value::Reg(2));
                assert_eq!(*left, Value::Reg(1));
                assert_eq!(*right, Value::ImmFloat(3.14));
            }
            _ => panic!("Expected FCmp instruction"),
        }
    }

    #[test]
    fn test_mixed_type_comparison_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a mixed type comparison: int_var < 3.5
        let comparison_expr = Expression::Comparison {
            op: crate::ast::ComparisonOp::LessThan,
            left: Box::new(Expression::Identifier("int_var".to_string())),
            right: Box::new(Expression::FloatLiteral(3.5)),
        };
        
        // Set up symbol table for the identifier (int type)
        ir_gen.symbol_table.insert("int_var".to_string(), (Value::Reg(0), Ty::Int));
        
        let mut function = Function {
            name: "test".to_string(),
            body: Vec::new(),
            next_reg: 1,
            next_ptr: 1,
        };
        
        let (result_val, result_type) = ir_gen.generate_expression_ir(comparison_expr, &mut function);
        
        // Comparison should return boolean type
        assert_eq!(result_type, Ty::Bool);
        assert_eq!(result_val, Value::Reg(3));
        
        // Check that the function body contains Load, SIToFP, and FCmp instructions
        assert_eq!(function.body.len(), 3);
        
        // Second instruction should be SIToFP (type promotion)
        match &function.body[1] {
            Inst::SIToFP(result_reg, int_val) => {
                assert_eq!(*result_reg, Value::Reg(2));
                assert_eq!(*int_val, Value::Reg(1));
            }
            _ => panic!("Expected SIToFP instruction"),
        }
        
        // Third instruction should be FCmp
        match &function.body[2] {
            Inst::FCmp { op, result, left, right } => {
                assert_eq!(op, "olt");
                assert_eq!(*result, Value::Reg(3));
                assert_eq!(*left, Value::Reg(2)); // Promoted value
                assert_eq!(*right, Value::ImmFloat(3.5));
            }
            _ => panic!("Expected FCmp instruction"),
        }
    }

    #[test]
    fn test_logical_and_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a logical AND expression: flag1 && flag2
        let logical_expr = Expression::Logical {
            op: crate::ast::LogicalOp::And,
            left: Box::new(Expression::Identifier("flag1".to_string())),
            right: Box::new(Expression::Identifier("flag2".to_string())),
        };
        
        // Set up symbol table for the identifiers
        ir_gen.symbol_table.insert("flag1".to_string(), (Value::Reg(0), Ty::Bool));
        ir_gen.symbol_table.insert("flag2".to_string(), (Value::Reg(1), Ty::Bool));
        
        let mut function = Function {
            name: "test".to_string(),
            body: Vec::new(),
            next_reg: 2,
            next_ptr: 2,
        };
        
        let (result_val, result_type) = ir_gen.generate_expression_ir(logical_expr, &mut function);
        
        // Logical operation should return boolean type
        assert_eq!(result_type, Ty::Bool);
        assert_eq!(result_val, Value::Reg(2));
        
        // Check that the function body contains Load, Load, and And instructions
        assert_eq!(function.body.len(), 3);
        
        // Third instruction should be And
        match &function.body[2] {
            Inst::And { result, left, right } => {
                assert_eq!(*result, Value::Reg(2));
                assert_eq!(*left, Value::Reg(0));
                assert_eq!(*right, Value::Reg(1));
            }
            _ => panic!("Expected And instruction"),
        }
    }

    #[test]
    fn test_logical_or_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a logical OR expression: flag1 || flag2
        let logical_expr = Expression::Logical {
            op: crate::ast::LogicalOp::Or,
            left: Box::new(Expression::Identifier("flag1".to_string())),
            right: Box::new(Expression::Identifier("flag2".to_string())),
        };
        
        // Set up symbol table for the identifiers
        ir_gen.symbol_table.insert("flag1".to_string(), (Value::Reg(0), Ty::Bool));
        ir_gen.symbol_table.insert("flag2".to_string(), (Value::Reg(1), Ty::Bool));
        
        let mut function = Function {
            name: "test".to_string(),
            body: Vec::new(),
            next_reg: 2,
            next_ptr: 2,
        };
        
        let (result_val, result_type) = ir_gen.generate_expression_ir(logical_expr, &mut function);
        
        // Logical operation should return boolean type
        assert_eq!(result_type, Ty::Bool);
        assert_eq!(result_val, Value::Reg(2));
        
        // Check that the function body contains Load, Load, and Or instructions
        assert_eq!(function.body.len(), 3);
        
        // Third instruction should be Or
        match &function.body[2] {
            Inst::Or { result, left, right } => {
                assert_eq!(*result, Value::Reg(2));
                assert_eq!(*left, Value::Reg(0));
                assert_eq!(*right, Value::Reg(1));
            }
            _ => panic!("Expected Or instruction"),
        }
    }

    #[test]
    fn test_unary_not_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a unary NOT expression: !flag
        let unary_expr = Expression::Unary {
            op: crate::ast::UnaryOp::Not,
            operand: Box::new(Expression::Identifier("flag".to_string())),
        };
        
        // Set up symbol table for the identifier
        ir_gen.symbol_table.insert("flag".to_string(), (Value::Reg(0), Ty::Bool));
        
        let mut function = Function {
            name: "test".to_string(),
            body: Vec::new(),
            next_reg: 1,
            next_ptr: 1,
        };
        
        let (result_val, result_type) = ir_gen.generate_expression_ir(unary_expr, &mut function);
        
        // Unary NOT should return boolean type
        assert_eq!(result_type, Ty::Bool);
        assert_eq!(result_val, Value::Reg(1));
        
        // Check that the function body contains Load and Not instructions
        assert_eq!(function.body.len(), 2);
        
        // Second instruction should be Not
        match &function.body[1] {
            Inst::Not { result, operand } => {
                assert_eq!(*result, Value::Reg(1));
                assert_eq!(*operand, Value::Reg(0));
            }
            _ => panic!("Expected Not instruction"),
        }
    }

    #[test]
    fn test_unary_minus_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a unary minus expression: -x
        let unary_expr = Expression::Unary {
            op: crate::ast::UnaryOp::Negate,
            operand: Box::new(Expression::Identifier("x".to_string())),
        };
        
        // Set up symbol table for the identifier
        ir_gen.symbol_table.insert("x".to_string(), (Value::Reg(0), Ty::Int));
        
        let mut function = Function {
            name: "test".to_string(),
            body: Vec::new(),
            next_reg: 1,
            next_ptr: 1,
        };
        
        let (result_val, result_type) = ir_gen.generate_expression_ir(unary_expr, &mut function);
        
        // Unary minus should preserve the operand type
        assert_eq!(result_type, Ty::Int);
        assert_eq!(result_val, Value::Reg(1));
        
        // Check that the function body contains Load and Neg instructions
        assert_eq!(function.body.len(), 2);
        
        // Second instruction should be Neg
        match &function.body[1] {
            Inst::Neg { result, operand } => {
                assert_eq!(*result, Value::Reg(1));
                assert_eq!(*operand, Value::Reg(0));
            }
            _ => panic!("Expected Neg instruction"),
        }
    }

    #[test]
    fn test_complex_expression_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a complex expression: !(x > 5 && y < 10)
        let complex_expr = Expression::Unary {
            op: crate::ast::UnaryOp::Not,
            operand: Box::new(Expression::Logical {
                op: crate::ast::LogicalOp::And,
                left: Box::new(Expression::Comparison {
                    op: crate::ast::ComparisonOp::GreaterThan,
                    left: Box::new(Expression::Identifier("x".to_string())),
                    right: Box::new(Expression::IntegerLiteral(5)),
                }),
                right: Box::new(Expression::Comparison {
                    op: crate::ast::ComparisonOp::LessThan,
                    left: Box::new(Expression::Identifier("y".to_string())),
                    right: Box::new(Expression::IntegerLiteral(10)),
                }),
            }),
        };
        
        // Set up symbol table for the identifiers
        ir_gen.symbol_table.insert("x".to_string(), (Value::Reg(0), Ty::Int));
        ir_gen.symbol_table.insert("y".to_string(), (Value::Reg(1), Ty::Int));
        
        let mut function = Function {
            name: "test".to_string(),
            body: Vec::new(),
            next_reg: 2,
            next_ptr: 2,
        };
        
        let (result_val, result_type) = ir_gen.generate_expression_ir(complex_expr, &mut function);
        
        // Final result should be boolean
        assert_eq!(result_type, Ty::Bool);
        
        // Check that we have the expected number of instructions
        // Load x, ICmp x > 5, Load y, ICmp y < 10, And, Not
        assert_eq!(function.body.len(), 6);
        
        // Verify the final Not instruction
        match &function.body[5] {
            Inst::Not { result, operand: _ } => {
                assert_eq!(*result, result_val);
            }
            _ => panic!("Expected final Not instruction"),
        }
    }

    #[test]
    fn test_print_with_multiple_arguments() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a print expression with multiple arguments: print!("{} + {} = {}", a, b, sum)
        let print_expr = Expression::Print {
            format_string: "{} + {} = {}".to_string(),
            arguments: vec![
                Expression::Identifier("a".to_string()),
                Expression::Identifier("b".to_string()),
                Expression::Identifier("sum".to_string()),
            ],
        };
        
        // Set up symbol table for the identifiers
        ir_gen.symbol_table.insert("a".to_string(), (Value::Reg(0), Ty::Int));
        ir_gen.symbol_table.insert("b".to_string(), (Value::Reg(1), Ty::Int));
        ir_gen.symbol_table.insert("sum".to_string(), (Value::Reg(2), Ty::Int));
        
        let mut function = Function {
            name: "test".to_string(),
            body: Vec::new(),
            next_reg: 3,
            next_ptr: 3,
        };
        
        let (result_val, result_type) = ir_gen.generate_expression_ir(print_expr, &mut function);
        
        // Print should return unit type
        assert_eq!(result_val, Value::ImmInt(0));
        assert_eq!(result_type, Ty::Int);
        
        // Check that we have Load instructions for each argument plus Print
        assert_eq!(function.body.len(), 4);
        
        // Final instruction should be Print with 3 arguments
        match &function.body[3] {
            Inst::Print { format_string, arguments } => {
                assert_eq!(format_string, "{} + {} = {}");
                assert_eq!(arguments.len(), 3);
                assert_eq!(arguments[0], Value::Reg(0));
                assert_eq!(arguments[1], Value::Reg(1));
                assert_eq!(arguments[2], Value::Reg(2));
            }
            _ => panic!("Expected Print instruction"),
        }
    }

    #[test]
    fn test_function_call_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a function call: add(5, 3)
        let func_call = Expression::FunctionCall {
            name: "add".to_string(),
            arguments: vec![
                Expression::IntegerLiteral(5),
                Expression::IntegerLiteral(3),
            ],
        };
        
        let let_stmt = Statement::Let {
            name: "result".to_string(),
            value: func_call,
        };
        
        let ast = vec![AstNode::Statement(let_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        let main_func = &ir["main"];
        
        // Should have: alloca for result, call instruction, store instruction
        assert!(main_func.body.len() >= 3);
        
        // Find the call instruction
        let call_inst = main_func.body.iter().find(|inst| matches!(inst, Inst::Call { .. }));
        assert!(call_inst.is_some());
        
        match call_inst.unwrap() {
            Inst::Call { function, arguments, result } => {
                assert_eq!(function, "add");
                assert_eq!(arguments.len(), 2);
                assert!(matches!(arguments[0], Value::ImmInt(5)));
                assert!(matches!(arguments[1], Value::ImmInt(3)));
                assert!(result.is_some());
            }
            _ => panic!("Expected Call instruction"),
        }
    }

    #[test]
    fn test_function_with_no_parameters() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a function: fn get_value() -> i32 { 42 }
        let body = Block {
            statements: vec![],
            expression: Some(Expression::IntegerLiteral(42)),
        };
        
        let func_stmt = Statement::Function {
            name: "get_value".to_string(),
            parameters: vec![],
            return_type: Some(Type { name: "i32".to_string() }),
            body,
        };
        
        let ast = vec![AstNode::Statement(func_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        let main_func = &ir["main"];
        match &main_func.body[0] {
            Inst::FunctionDef { name, parameters, return_type, body } => {
                assert_eq!(name, "get_value");
                assert_eq!(parameters.len(), 0);
                assert!(return_type.is_none());
                
                // Should have return instruction with immediate value
                assert!(body.iter().any(|inst| matches!(inst, Inst::Return(Value::ImmInt(42)))));
            }
            _ => panic!("Expected FunctionDef instruction"),
        }
    }

    #[test]
    fn test_function_with_return_statement() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a function: fn double(x: i32) -> i32 { return x * 2; }
        let param = Parameter {
            name: "x".to_string(),
            param_type: Type { name: "i32".to_string() },
        };
        
        let body = Block {
            statements: vec![
                Statement::Return(Expression::Binary {
                    op: "*".to_string(),
                    lhs: Box::new(Expression::Identifier("x".to_string())),
                    rhs: Box::new(Expression::IntegerLiteral(2)),
                    ty: Some(Ty::Int),
                }),
            ],
            expression: None,
        };
        
        let func_stmt = Statement::Function {
            name: "double".to_string(),
            parameters: vec![param],
            return_type: Some(Type { name: "i32".to_string() }),
            body,
        };
        
        let ast = vec![AstNode::Statement(func_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        let main_func = &ir["main"];
        match &main_func.body[0] {
            Inst::FunctionDef { name, parameters, return_type, body } => {
                assert_eq!(name, "double");
                assert_eq!(parameters.len(), 1);
                assert_eq!(parameters[0].0, "x");
                assert!(return_type.is_none());
                
                // Should have parameter alloca, multiplication, and return
                assert!(body.iter().any(|inst| matches!(inst, Inst::Alloca(Value::Reg(0), name) if name == "x")));
                assert!(body.iter().any(|inst| matches!(inst, Inst::Mul(_, _, _))));
                assert!(body.iter().any(|inst| matches!(inst, Inst::Return(_))));
            }
            _ => panic!("Expected FunctionDef instruction"),
        }
    }

    #[test]
    fn test_nested_function_calls() {
        let mut ir_gen = IrGenerator::new();
        
        // Create nested function calls: add(multiply(2, 3), 4)
        let inner_call = Expression::FunctionCall {
            name: "multiply".to_string(),
            arguments: vec![
                Expression::IntegerLiteral(2),
                Expression::IntegerLiteral(3),
            ],
        };
        
        let outer_call = Expression::FunctionCall {
            name: "add".to_string(),
            arguments: vec![
                inner_call,
                Expression::IntegerLiteral(4),
            ],
        };
        
        let let_stmt = Statement::Let {
            name: "result".to_string(),
            value: outer_call,
        };
        
        let ast = vec![AstNode::Statement(let_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        let main_func = &ir["main"];
        
        // Should have two call instructions
        let call_instructions: Vec<_> = main_func.body.iter()
            .filter(|inst| matches!(inst, Inst::Call { .. }))
            .collect();
        
        assert_eq!(call_instructions.len(), 2);
        
        // First call should be to multiply
        match call_instructions[0] {
            Inst::Call { function, arguments, .. } => {
                assert_eq!(function, "multiply");
                assert_eq!(arguments.len(), 2);
                assert!(matches!(arguments[0], Value::ImmInt(2)));
                assert!(matches!(arguments[1], Value::ImmInt(3)));
            }
            _ => panic!("Expected Call instruction"),
        }
        
        // Second call should be to add
        match call_instructions[1] {
            Inst::Call { function, arguments, .. } => {
                assert_eq!(function, "add");
                assert_eq!(arguments.len(), 2);
                // First argument should be a register (result of multiply call)
                assert!(matches!(arguments[0], Value::Reg(_)));
                assert!(matches!(arguments[1], Value::ImmInt(4)));
            }
            _ => panic!("Expected Call instruction"),
        }
    }

    #[test]
    fn test_function_with_local_variables() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a function with local variables:
        // fn calculate(x: i32) -> i32 {
        //     let temp = x + 1;
        //     temp * 2
        // }
        let param = Parameter {
            name: "x".to_string(),
            param_type: Type { name: "i32".to_string() },
        };
        
        let body = Block {
            statements: vec![
                Statement::Let {
                    name: "temp".to_string(),
                    value: Expression::Binary {
                        op: "+".to_string(),
                        lhs: Box::new(Expression::Identifier("x".to_string())),
                        rhs: Box::new(Expression::IntegerLiteral(1)),
                        ty: Some(Ty::Int),
                    },
                },
            ],
            expression: Some(Expression::Binary {
                op: "*".to_string(),
                lhs: Box::new(Expression::Identifier("temp".to_string())),
                rhs: Box::new(Expression::IntegerLiteral(2)),
                ty: Some(Ty::Int),
            }),
        };
        
        let func_stmt = Statement::Function {
            name: "calculate".to_string(),
            parameters: vec![param],
            return_type: Some(Type { name: "i32".to_string() }),
            body,
        };
        
        let ast = vec![AstNode::Statement(func_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        let main_func = &ir["main"];
        match &main_func.body[0] {
            Inst::FunctionDef { name, parameters, return_type, body } => {
                assert_eq!(name, "calculate");
                assert_eq!(parameters.len(), 1);
                assert!(return_type.is_none());
                
                // Should have allocas for both parameter and local variable
                let alloca_count = body.iter().filter(|inst| matches!(inst, Inst::Alloca(_, _))).count();
                assert_eq!(alloca_count, 2); // x and temp
                
                // Should have two add/multiply operations and a return
                assert!(body.iter().any(|inst| matches!(inst, Inst::Add(_, _, _))));
                assert!(body.iter().any(|inst| matches!(inst, Inst::Mul(_, _, _))));
                assert!(body.iter().any(|inst| matches!(inst, Inst::Return(_))));
            }
            _ => panic!("Expected FunctionDef instruction"),
        }
    }

    #[test]
    fn test_function_call_with_no_arguments() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a function call with no arguments: get_value()
        let func_call = Expression::FunctionCall {
            name: "get_value".to_string(),
            arguments: vec![],
        };
        
        let let_stmt = Statement::Let {
            name: "result".to_string(),
            value: func_call,
        };
        
        let ast = vec![AstNode::Statement(let_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        let main_func = &ir["main"];
        
        // Find the call instruction
        let call_inst = main_func.body.iter().find(|inst| matches!(inst, Inst::Call { .. }));
        assert!(call_inst.is_some());
        
        match call_inst.unwrap() {
            Inst::Call { function, arguments, result } => {
                assert_eq!(function, "get_value");
                assert_eq!(arguments.len(), 0);
                assert!(result.is_some());
            }
            _ => panic!("Expected Call instruction"),
        }
    }

    // Control flow tests
    #[test]
    fn test_if_statement_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create an if statement: let x = 7; if x > 5 { let y = 10; }
        let x_decl = Statement::Let {
            name: "x".to_string(),
            value: Expression::IntegerLiteral(7),
        };
        
        let condition = Expression::Binary {
            op: ">".to_string(),
            lhs: Box::new(Expression::Identifier("x".to_string())),
            rhs: Box::new(Expression::IntegerLiteral(5)),
            ty: Some(Ty::Bool),
        };
        
        let then_block = Block {
            statements: vec![
                Statement::Let {
                    name: "y".to_string(),
                    value: Expression::IntegerLiteral(10),
                },
            ],
            expression: None,
        };
        
        let if_stmt = Statement::If {
            condition,
            then_block,
            else_block: None,
        };
        
        let ast = vec![AstNode::Statement(x_decl), AstNode::Statement(if_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        let main_func = &ir["main"];
        
        // Should have branch, labels, and jump instructions
        let branch_count = main_func.body.iter().filter(|inst| matches!(inst, Inst::Branch { .. })).count();
        let label_count = main_func.body.iter().filter(|inst| matches!(inst, Inst::Label(_))).count();
        let jump_count = main_func.body.iter().filter(|inst| matches!(inst, Inst::Jump(_))).count();
        
        assert_eq!(branch_count, 1);
        assert!(label_count >= 2); // then and end labels
        assert!(jump_count >= 1); // jump to end
    }

    #[test]
    fn test_if_else_statement_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create an if-else statement: let x = 7; if x > 5 { let y = 10; } else { let z = 20; }
        let x_decl = Statement::Let {
            name: "x".to_string(),
            value: Expression::IntegerLiteral(7),
        };
        
        let condition = Expression::Binary {
            op: ">".to_string(),
            lhs: Box::new(Expression::Identifier("x".to_string())),
            rhs: Box::new(Expression::IntegerLiteral(5)),
            ty: Some(Ty::Bool),
        };
        
        let then_block = Block {
            statements: vec![
                Statement::Let {
                    name: "y".to_string(),
                    value: Expression::IntegerLiteral(10),
                },
            ],
            expression: None,
        };
        
        let else_block = Some(Box::new(Statement::Let {
            name: "z".to_string(),
            value: Expression::IntegerLiteral(20),
        }));
        
        let if_stmt = Statement::If {
            condition,
            then_block,
            else_block,
        };
        
        let ast = vec![AstNode::Statement(x_decl), AstNode::Statement(if_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        let main_func = &ir["main"];
        
        // Should have branch, labels, and jump instructions
        let branch_count = main_func.body.iter().filter(|inst| matches!(inst, Inst::Branch { .. })).count();
        let label_count = main_func.body.iter().filter(|inst| matches!(inst, Inst::Label(_))).count();
        let jump_count = main_func.body.iter().filter(|inst| matches!(inst, Inst::Jump(_))).count();
        
        assert_eq!(branch_count, 1);
        assert!(label_count >= 3); // then, else, and end labels
        assert!(jump_count >= 2); // jumps from then and else to end
    }

    #[test]
    fn test_while_loop_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a while loop: let i = 0; while i < 10 { i = i + 1; }
        let i_decl = Statement::Let {
            name: "i".to_string(),
            value: Expression::IntegerLiteral(0),
        };
        
        let condition = Expression::Binary {
            op: "<".to_string(),
            lhs: Box::new(Expression::Identifier("i".to_string())),
            rhs: Box::new(Expression::IntegerLiteral(10)),
            ty: Some(Ty::Bool),
        };
        
        let body = Block {
            statements: vec![
                Statement::Let {
                    name: "i".to_string(),
                    value: Expression::Binary {
                        op: "+".to_string(),
                        lhs: Box::new(Expression::Identifier("i".to_string())),
                        rhs: Box::new(Expression::IntegerLiteral(1)),
                        ty: Some(Ty::Int),
                    },
                },
            ],
            expression: None,
        };
        
        let while_stmt = Statement::While { condition, body };
        
        let ast = vec![AstNode::Statement(i_decl), AstNode::Statement(while_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        let main_func = &ir["main"];
        
        // Should have branch, labels, and jump instructions for loop
        let branch_count = main_func.body.iter().filter(|inst| matches!(inst, Inst::Branch { .. })).count();
        let label_count = main_func.body.iter().filter(|inst| matches!(inst, Inst::Label(_))).count();
        let jump_count = main_func.body.iter().filter(|inst| matches!(inst, Inst::Jump(_))).count();
        
        assert_eq!(branch_count, 1); // condition check
        assert!(label_count >= 3); // start, body, end labels
        assert!(jump_count >= 2); // initial jump to start, jump back to start
    }

    #[test]
    fn test_for_loop_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a for loop: for i in 0..5 { println(i); }
        let iterable = Expression::Binary {
            op: "..".to_string(),
            lhs: Box::new(Expression::IntegerLiteral(0)),
            rhs: Box::new(Expression::IntegerLiteral(5)),
            ty: Some(Ty::Int),
        };
        
        let body = Block {
            statements: vec![
                Statement::Let {
                    name: "temp".to_string(),
                    value: Expression::Identifier("i".to_string()),
                },
            ],
            expression: None,
        };
        
        let for_stmt = Statement::For {
            variable: "i".to_string(),
            iterable,
            body,
        };
        
        let ast = vec![AstNode::Statement(for_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        let main_func = &ir["main"];
        
        // Should have variable allocation, branch, labels, and jump instructions
        let alloca_count = main_func.body.iter().filter(|inst| matches!(inst, Inst::Alloca(_, _))).count();
        let branch_count = main_func.body.iter().filter(|inst| matches!(inst, Inst::Branch { .. })).count();
        let label_count = main_func.body.iter().filter(|inst| matches!(inst, Inst::Label(_))).count();
        let jump_count = main_func.body.iter().filter(|inst| matches!(inst, Inst::Jump(_))).count();
        
        assert!(alloca_count >= 1); // loop variable allocation
        assert_eq!(branch_count, 1); // condition check
        assert!(label_count >= 3); // start, body, end labels
        assert!(jump_count >= 2); // initial jump to start, jump back to start
    }

    #[test]
    fn test_infinite_loop_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create an infinite loop: loop { x = x + 1; }
        let body = Block {
            statements: vec![
                Statement::Let {
                    name: "x".to_string(),
                    value: Expression::Binary {
                        op: "+".to_string(),
                        lhs: Box::new(Expression::Identifier("x".to_string())),
                        rhs: Box::new(Expression::IntegerLiteral(1)),
                        ty: Some(Ty::Int),
                    },
                },
            ],
            expression: None,
        };
        
        let loop_stmt = Statement::Loop { body };
        
        let ast = vec![AstNode::Statement(loop_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        let main_func = &ir["main"];
        
        // Should have labels and jump instructions for infinite loop
        let label_count = main_func.body.iter().filter(|inst| matches!(inst, Inst::Label(_))).count();
        let jump_count = main_func.body.iter().filter(|inst| matches!(inst, Inst::Jump(_))).count();
        
        assert!(label_count >= 1); // loop start label
        assert!(jump_count >= 2); // initial jump to start, jump back to start
    }

    #[test]
    fn test_break_statement_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a break statement
        let break_stmt = Statement::Break;
        
        let ast = vec![AstNode::Statement(break_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        let main_func = &ir["main"];
        
        // Should have a jump instruction for break
        let jump_count = main_func.body.iter().filter(|inst| matches!(inst, Inst::Jump(_))).count();
        assert_eq!(jump_count, 1);
        
        // Check that it's jumping to a break label
        let jump_inst = main_func.body.iter().find(|inst| matches!(inst, Inst::Jump(_)));
        assert!(jump_inst.is_some());
        match jump_inst.unwrap() {
            Inst::Jump(label) => {
                assert!(label.contains("loop_end") || label.contains("placeholder"));
            }
            _ => panic!("Expected Jump instruction"),
        }
    }

    #[test]
    fn test_continue_statement_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a continue statement
        let continue_stmt = Statement::Continue;
        
        let ast = vec![AstNode::Statement(continue_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        let main_func = &ir["main"];
        
        // Should have a jump instruction for continue
        let jump_count = main_func.body.iter().filter(|inst| matches!(inst, Inst::Jump(_))).count();
        assert_eq!(jump_count, 1);
        
        // Check that it's jumping to a continue label
        let jump_inst = main_func.body.iter().find(|inst| matches!(inst, Inst::Jump(_)));
        assert!(jump_inst.is_some());
        match jump_inst.unwrap() {
            Inst::Jump(label) => {
                assert!(label.contains("loop_start") || label.contains("placeholder"));
            }
            _ => panic!("Expected Jump instruction"),
        }
    }

    #[test]
    fn test_nested_control_flow_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create nested control flow: if x > 0 { while y < 10 { y = y + 1; } }
        let while_condition = Expression::Binary {
            op: "<".to_string(),
            lhs: Box::new(Expression::Identifier("y".to_string())),
            rhs: Box::new(Expression::IntegerLiteral(10)),
            ty: Some(Ty::Bool),
        };
        
        let while_body = Block {
            statements: vec![
                Statement::Let {
                    name: "y".to_string(),
                    value: Expression::Binary {
                        op: "+".to_string(),
                        lhs: Box::new(Expression::Identifier("y".to_string())),
                        rhs: Box::new(Expression::IntegerLiteral(1)),
                        ty: Some(Ty::Int),
                    },
                },
            ],
            expression: None,
        };
        
        let while_stmt = Statement::While {
            condition: while_condition,
            body: while_body,
        };
        
        let if_condition = Expression::Binary {
            op: ">".to_string(),
            lhs: Box::new(Expression::Identifier("x".to_string())),
            rhs: Box::new(Expression::IntegerLiteral(0)),
            ty: Some(Ty::Bool),
        };
        
        let if_then_block = Block {
            statements: vec![while_stmt],
            expression: None,
        };
        
        let if_stmt = Statement::If {
            condition: if_condition,
            then_block: if_then_block,
            else_block: None,
        };
        
        let ast = vec![AstNode::Statement(if_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        let main_func = &ir["main"];
        
        // Should have multiple branches, labels, and jumps for nested control flow
        let branch_count = main_func.body.iter().filter(|inst| matches!(inst, Inst::Branch { .. })).count();
        let label_count = main_func.body.iter().filter(|inst| matches!(inst, Inst::Label(_))).count();
        let jump_count = main_func.body.iter().filter(|inst| matches!(inst, Inst::Jump(_))).count();
        
        assert!(branch_count >= 2); // if condition + while condition
        assert!(label_count >= 5); // if then/else/end + while start/body/end
        assert!(jump_count >= 3); // if jumps + while jumps
    }

    // Struct IR generation methods
    
    fn generate_struct_definition_ir(&mut self, name: String, fields: Vec<StructField>, is_tuple: bool, current_function: &mut Function) {
        // Convert AST StructField to IR field representation
        let ir_fields: Vec<(String, String)> = fields.iter().enumerate().map(|(i, field)| {
            let field_name = if is_tuple {
                i.to_string() // Use index as field name for tuple structs
            } else {
                field.name.clone()
            };
            let field_type = self.type_to_string(&field.field_type);
            (field_name, field_type)
        }).collect();
        
        // Create field index mapping
        let mut field_indices = HashMap::new();
        for (i, (field_name, _)) in ir_fields.iter().enumerate() {
            field_indices.insert(field_name.clone(), i);
        }
        
        // Store struct definition
        let struct_def = StructDefinition {
            name: name.clone(),
            fields: ir_fields.clone(),
            field_indices,
            is_tuple,
        };
        self.struct_definitions.insert(name.clone(), struct_def);
        
        // Generate struct definition IR instruction
        let struct_def_inst = Inst::StructDef {
            name,
            fields: ir_fields,
            is_tuple,
        };
        current_function.body.push(struct_def_inst);
    }
    
    fn generate_struct_literal_ir(&mut self, name: String, fields: Vec<(String, Expression)>, function: &mut Function) -> (Value, Ty) {
        // Get struct definition
        let struct_def = self.struct_definitions.get(&name)
            .expect(&format!("Undefined struct: {}", name))
            .clone();
        
        // Allocate memory for the struct
        let struct_ptr = Value::Reg(self.next_ptr);
        self.next_ptr += 1;
        function.body.push(Inst::StructAlloca {
            result: struct_ptr.clone(),
            struct_name: name.clone(),
        });
        
        // Generate field values and create field initialization list
        let mut field_values = Vec::new();
        for (field_name, field_expr) in fields {
            let (field_value, _) = self.generate_expression_ir(field_expr, function);
            field_values.push((field_name, field_value));
        }
        
        // Initialize struct with field values
        function.body.push(Inst::StructInit {
            result: struct_ptr.clone(),
            struct_name: name.clone(),
            field_values,
        });
        
        // Return struct pointer and custom struct type
        (struct_ptr, Ty::Struct(name))
    }
    
    fn generate_field_access_ir(&mut self, object: Expression, field: String, function: &mut Function) -> (Value, Ty) {
        // Generate IR for the object expression
        let (object_value, object_type) = self.generate_expression_ir(object, function);
        
        // Extract struct name from type
        let struct_name = match object_type {
            Ty::Struct(name) => name,
            _ => panic!("Field access on non-struct type: {:?}", object_type),
        };
        
        // Get struct definition and field index
        let struct_def = self.struct_definitions.get(&struct_name)
            .expect(&format!("Undefined struct: {}", struct_name))
            .clone();
        
        let field_index = *struct_def.field_indices.get(&field)
            .expect(&format!("Field '{}' not found in struct '{}'", field, struct_name));
        
        // Generate field access instruction
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        function.body.push(Inst::FieldAccess {
            result: result_reg.clone(),
            struct_ptr: object_value,
            field_name: field.clone(),
            field_index,
        });
        
        // Look up actual field type from struct definition
        let field_type = self.get_field_type(&struct_def, &field);
        
        (result_reg, field_type)
    }
    
    fn generate_struct_copy_ir(&mut self, source: Value, struct_name: String, function: &mut Function) -> Value {
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        function.body.push(Inst::StructCopy {
            result: result_reg.clone(),
            source,
            struct_name,
        });
        
        result_reg
    }
    
    fn type_to_string(&self, ty: &Type) -> String {
        match ty {
            Type::Named(name) => name.clone(),
            Type::Generic { name, .. } => name.clone(),
            Type::Array { element_type, size } => {
                if let Some(s) = size {
                    format!("[{}; {}]", self.type_to_string(element_type), s)
                } else {
                    format!("[{}]", self.type_to_string(element_type))
                }
            }
            Type::Slice { element_type } => format!("&[{}]", self.type_to_string(element_type)),
            Type::Vec { element_type } => format!("Vec<{}>", self.type_to_string(element_type)),
            Type::HashMap { key_type, value_type } => {
                format!("HashMap<{}, {}>", self.type_to_string(key_type), self.type_to_string(value_type))
            }
            Type::Reference { mutable, inner_type } => {
                if *mutable {
                    format!("&mut {}", self.type_to_string(inner_type))
                } else {
                    format!("&{}", self.type_to_string(inner_type))
                }
            }
        }
    }
    
    // Struct IR generation methods for function bodies
    
    fn generate_struct_literal_ir_for_function(&mut self, name: String, fields: Vec<(String, Expression)>, function_body: &mut Vec<Inst>) -> (Value, Ty) {
        // Get struct definition
        let struct_def = self.struct_definitions.get(&name)
            .expect(&format!("Undefined struct: {}", name))
            .clone();
        
        // Allocate memory for the struct
        let struct_ptr = Value::Reg(self.next_ptr);
        self.next_ptr += 1;
        function_body.push(Inst::StructAlloca {
            result: struct_ptr.clone(),
            struct_name: name.clone(),
        });
        
        // Generate field values and create field initialization list
        let mut field_values = Vec::new();
        for (field_name, field_expr) in fields {
            let (field_value, _) = self.generate_expression_ir_for_function(field_expr, function_body);
            field_values.push((field_name, field_value));
        }
        
        // Initialize struct with field values
        function_body.push(Inst::StructInit {
            result: struct_ptr.clone(),
            struct_name: name.clone(),
            field_values,
        });
        
        // Return struct pointer and custom struct type
        (struct_ptr, Ty::Struct(name))
    }
    
    fn generate_field_access_ir_for_function(&mut self, object: Expression, field: String, function_body: &mut Vec<Inst>) -> (Value, Ty) {
        // Generate IR for the object expression
        let (object_value, object_type) = self.generate_expression_ir_for_function(object, function_body);
        
        // Extract struct name from type
        let struct_name = match object_type {
            Ty::Struct(name) => name,
            _ => panic!("Field access on non-struct type: {:?}", object_type),
        };
        
        // Get struct definition and field index
        let struct_def = self.struct_definitions.get(&struct_name)
            .expect(&format!("Undefined struct: {}", struct_name))
            .clone();
        
        let field_index = *struct_def.field_indices.get(&field)
            .expect(&format!("Field '{}' not found in struct '{}'", field, struct_name));
        
        // Generate field access instruction
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        function_body.push(Inst::FieldAccess {
            result: result_reg.clone(),
            struct_ptr: object_value,
            field_name: field.clone(),
            field_index,
        });
        
        // Look up actual field type from struct definition
        let field_type = self.get_field_type(&struct_def, &field);
        
        (result_reg, field_type)
    }

    // Helper method to get field type from struct definition
    fn get_field_type(&self, struct_def: &StructDefinition, field_name: &str) -> Ty {
        // Find the field in the struct definition
        for (name, type_str) in &struct_def.fields {
            if name == field_name {
                // Convert type string back to Ty
                return self.string_to_ty(type_str);
            }
        }
        
        // Fallback to Int if field not found (should not happen if semantic analysis is correct)
        Ty::Int
    }
    
    // Helper method to convert type string back to Ty
    fn string_to_ty(&self, type_str: &str) -> Ty {
        match type_str {
            "i32" | "int" => Ty::Int,
            "f64" | "float" => Ty::Float,
            "bool" => Ty::Bool,
            "String" => Ty::String,
            s if s.starts_with("Vec<") => {
                // Parse Vec<T> type
                let inner = &s[4..s.len()-1];
                Ty::Vec(Box::new(self.string_to_ty(inner)))
            }
            s if s.starts_with("[") && s.contains(";") => {
                // Parse [T; N] array type
                let parts: Vec<&str> = s[1..s.len()-1].split(";").collect();
                if parts.len() == 2 {
                    let element_type = self.string_to_ty(parts[0].trim());
                    if let Ok(size) = parts[1].trim().parse::<usize>() {
                        return Ty::Array(Box::new(element_type), Some(size));
                    }
                }
                Ty::Int // Fallback
            }
            s if s.starts_with("[") && s.ends_with("]") => {
                // Parse [T] slice type
                let inner = &s[1..s.len()-1];
                Ty::Array(Box::new(self.string_to_ty(inner)), None)
            }
            s if s.starts_with("&") => {
                // Parse reference type
                let inner = &s[1..];
                Ty::Reference(Box::new(self.string_to_ty(inner)))
            }
            _ => {
                // Check if it's a struct type
                if self.struct_definitions.contains_key(type_str) {
                    Ty::Struct(type_str.to_string())
                } else {
                    // Default fallback
                    Ty::Int
                }
            }
        }
    }
    
    fn type_to_string(&self, ty: &Type) -> String {
        match ty {
            Type::Named(name) => name.clone(),
            Type::Generic { name, .. } => name.clone(),
            Type::Array { element_type, size } => {
                if let Some(s) = size {
                    format!("[{}; {}]", self.type_to_string(element_type), s)
                } else {
                    format!("[{}]", self.type_to_string(element_type))
                }
            }
            Type::Slice { element_type } => format!("&[{}]", self.type_to_string(element_type)),
            Type::Vec { element_type } => format!("Vec<{}>", self.type_to_string(element_type)),
            Type::HashMap { key_type, value_type } => {
                format!("HashMap<{}, {}>", self.type_to_string(key_type), self.type_to_string(value_type))
            }
            Type::Reference { mutable, inner_type } => {
                if *mutable {
                    format!("&mut {}", self.type_to_string(inner_type))
                } else {
                    format!("&{}", self.type_to_string(inner_type))
                }
            }
        }
    }

    // Method call IR generation methods
    fn generate_method_call_ir(&mut self, object: Expression, method: String, arguments: Vec<Expression>, function: &mut Function) -> (Value, Ty) {
        // Generate IR for the object expression
        let (object_value, object_type) = self.generate_expression_ir(object, function);
        
        // Generate IR for arguments
        let mut arg_values = Vec::new();
        for arg in arguments {
            let (arg_value, _) = self.generate_expression_ir(arg, function);
            arg_values.push(arg_value);
        }
        
        // Create method call - for now, treat as function call with object as first argument
        let mut all_args = vec![object_value];
        all_args.extend(arg_values);
        
        // Generate result register for method call
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        // Create method call instruction (using function call for now)
        let method_name = match &object_type {
            Ty::Struct(struct_name) => format!("{}::{}", struct_name, method),
            _ => format!("{}::{}", object_type.to_string(), method),
        };
        
        let call_inst = Inst::Call {
            function: method_name,
            arguments: all_args,
            result: Some(result_reg.clone()),
        };
        
        function.body.push(call_inst);
        
        // For now, assume method calls return the same type as the object
        (result_reg, object_type)
    }
    
    fn generate_method_call_ir_for_function(&mut self, object: Expression, method: String, arguments: Vec<Expression>, function_body: &mut Vec<Inst>) -> (Value, Ty) {
        // Generate IR for the object expression
        let (object_value, object_type) = self.generate_expression_ir_for_function(object, function_body);
        
        // Generate IR for arguments
        let mut arg_values = Vec::new();
        for arg in arguments {
            let (arg_value, _) = self.generate_expression_ir_for_function(arg, function_body);
            arg_values.push(arg_value);
        }
        
        // Create method call - for now, treat as function call with object as first argument
        let mut all_args = vec![object_value];
        all_args.extend(arg_values);
        
        // Generate result register for method call
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        // Create method call instruction (using function call for now)
        let method_name = match &object_type {
            Ty::Struct(struct_name) => format!("{}::{}", struct_name, method),
            _ => format!("{}::{}", object_type.to_string(), method),
        };
        
        let call_inst = Inst::Call {
            function: method_name,
            arguments: all_args,
            result: Some(result_reg.clone()),
        };
        
        function_body.push(call_inst);
        
        // For now, assume method calls return the same type as the object
        (result_reg, object_type)
    }

    // Enhanced struct IR generation with field modification support
    fn generate_field_store_ir(&mut self, object: Expression, field: String, value: Expression, function: &mut Function) -> (Value, Ty) {
        // Generate IR for the object expression (should be a pointer)
        let (object_value, object_type) = self.generate_expression_ir(object, function);
        
        // Generate IR for the value to store
        let (value_reg, value_type) = self.generate_expression_ir(value, function);
        
        // Extract struct name from type
        let struct_name = match object_type {
            Ty::Struct(name) => name,
            _ => panic!("Field store on non-struct type: {:?}", object_type),
        };
        
        // Get struct definition and field index
        let struct_def = self.struct_definitions.get(&struct_name)
            .expect(&format!("Undefined struct: {}", struct_name))
            .clone();
        
        let field_index = *struct_def.field_indices.get(&field)
            .expect(&format!("Field '{}' not found in struct '{}'", field, struct_name));
        
        // Generate field store instruction
        function.body.push(Inst::FieldStore {
            struct_ptr: object_value.clone(),
            field_name: field,
            field_index,
            value: value_reg,
        });
        
        // Return the stored value and its type
        (value_reg, value_type)
    }

    // Struct copy and move semantics
    fn generate_struct_move_ir(&mut self, source: Value, struct_name: String, function: &mut Function) -> Value {
        // For now, implement move as copy (proper move semantics would invalidate source)
        self.generate_struct_copy_ir(source, struct_name, function)
    }

    // Enum definition IR generation
    fn generate_enum_definition_ir(&mut self, name: String, variants: Vec<EnumVariant>, current_function: &mut Function) {
        // Convert enum variants to IR format
        let ir_variants: Vec<(String, Option<Vec<String>>)> = variants.iter().map(|variant| {
            let data_types = match &variant.data {
                Some(EnumVariantData::Tuple(types)) => {
                    Some(types.iter().map(|t| self.type_to_string(t)).collect())
                }
                Some(EnumVariantData::Struct(fields)) => {
                    Some(fields.iter().map(|f| self.type_to_string(&f.field_type)).collect())
                }
                None => None,
            };
            (variant.name.clone(), data_types)
        }).collect();

        // Create variant index mapping
        let mut variant_indices = HashMap::new();
        for (index, variant) in variants.iter().enumerate() {
            variant_indices.insert(variant.name.clone(), index);
        }

        // Store enum definition
        let enum_def = EnumDefinition {
            name: name.clone(),
            variants: ir_variants.clone(),
            variant_indices,
            discriminant_type: "i32".to_string(), // Use i32 for discriminant
        };
        self.enum_definitions.insert(name.clone(), enum_def);

        // Generate enum definition instruction
        current_function.body.push(Inst::EnumDef {
            name,
            variants: ir_variants,
            discriminant_type: "i32".to_string(),
        });
    }

    // Match expression IR generation
    fn generate_match_expression_ir(&mut self, expression: Expression, arms: Vec<MatchArm>, function: &mut Function) -> (Value, Ty) {
        // Generate IR for the match expression (discriminant)
        let (discriminant_value, discriminant_type) = self.generate_expression_ir(expression, function);

        // Generate unique labels for match arms and end
        let mut arm_labels = Vec::new();
        let mut arm_body_labels = Vec::new();
        for i in 0..arms.len() {
            arm_labels.push(format!("match_arm_{}_{}", i, self.next_reg));
            arm_body_labels.push(format!("match_body_{}_{}", i, self.next_reg));
            self.next_reg += 1;
        }
        let end_label = format!("match_end_{}", self.next_reg);
        self.next_reg += 1;

        // Result register for the match expression
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;

        // Generate pattern matching logic
        match discriminant_type {
            Ty::Enum(enum_name) => {
                self.generate_enum_match_ir(discriminant_value, enum_name, arms, arm_labels, arm_body_labels, end_label.clone(), result_reg.clone(), function);
            }
            _ => {
                // For non-enum types, generate simple pattern matching
                self.generate_simple_match_ir(discriminant_value, discriminant_type, arms, arm_labels, arm_body_labels, end_label.clone(), result_reg.clone(), function);
            }
        }

        // End label
        function.body.push(Inst::Label(end_label));

        // Return result (assuming int type for now, should be inferred from arms)
        (result_reg, Ty::Int)
    }

    // Generate enum-specific match IR
    fn generate_enum_match_ir(&mut self, discriminant: Value, enum_name: String, arms: Vec<MatchArm>, arm_labels: Vec<String>, arm_body_labels: Vec<String>, end_label: String, result_reg: Value, function: &mut Function) {
        // Get enum definition
        let enum_def = self.enum_definitions.get(&enum_name)
            .expect(&format!("Undefined enum: {}", enum_name))
            .clone();

        // Extract discriminant from enum value
        let disc_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        function.body.push(Inst::EnumDiscriminant {
            result: disc_reg.clone(),
            enum_ptr: discriminant.clone(),
        });

        // Generate switch instruction for efficient pattern matching
        let mut switch_cases = Vec::new();
        let mut default_label = None;

        for (i, arm) in arms.iter().enumerate() {
            match &arm.pattern {
                Pattern::Enum { variant, .. } => {
                    if let Some(&variant_index) = enum_def.variant_indices.get(variant) {
                        switch_cases.push((variant_index as i64, arm_labels[i].clone()));
                    }
                }
                Pattern::Wildcard => {
                    default_label = Some(arm_labels[i].clone());
                }
                _ => {
                    // For other patterns, use the first arm label as default
                    if default_label.is_none() {
                        default_label = Some(arm_labels[i].clone());
                    }
                }
            }
        }

        // Generate switch instruction
        function.body.push(Inst::Switch {
            discriminant: disc_reg,
            cases: switch_cases,
            default_label: default_label.unwrap_or_else(|| format!("match_default_{}", self.next_reg)),
        });

        // Generate code for each arm
        for (i, arm) in arms.iter().enumerate() {
            // Arm label
            function.body.push(Inst::Label(arm_labels[i].clone()));

            // Generate pattern binding extraction
            self.generate_pattern_bindings(&arm.pattern, discriminant.clone(), enum_name.clone(), function);

            // Generate guard condition if present
            if let Some(guard) = &arm.guard {
                let (guard_value, _) = self.generate_expression_ir(guard.clone(), function);
                let guard_end_label = format!("guard_end_{}_{}", i, self.next_reg);
                self.next_reg += 1;
                
                function.body.push(Inst::Branch {
                    condition: guard_value,
                    true_label: arm_body_labels[i].clone(),
                    false_label: guard_end_label.clone(),
                });
                
                // If guard fails, continue to next arm or end
                function.body.push(Inst::Label(guard_end_label));
                let next_label = if i + 1 < arms.len() {
                    arm_labels[i + 1].clone()
                } else {
                    end_label.clone()
                };
                function.body.push(Inst::Jump(next_label));
            } else {
                // No guard, jump directly to body
                function.body.push(Inst::Jump(arm_body_labels[i].clone()));
            }

            // Arm body label
            function.body.push(Inst::Label(arm_body_labels[i].clone()));

            // Generate arm body
            let (arm_result, _) = self.generate_expression_ir(arm.body.clone(), function);
            
            // Store result in match result register
            let temp_ptr = Value::Reg(self.next_ptr);
            self.next_ptr += 1;
            function.body.push(Inst::Alloca(temp_ptr.clone(), format!("match_result_{}", i)));
            function.body.push(Inst::Store(temp_ptr.clone(), arm_result));
            function.body.push(Inst::Load(result_reg.clone(), temp_ptr));

            // Jump to end
            function.body.push(Inst::Jump(end_label.clone()));
        }
    }

    // Generate simple pattern matching for non-enum types
    fn generate_simple_match_ir(&mut self, discriminant: Value, discriminant_type: Ty, arms: Vec<MatchArm>, arm_labels: Vec<String>, arm_body_labels: Vec<String>, end_label: String, result_reg: Value, function: &mut Function) {
        // For simple types, generate if-else chain
        for (i, arm) in arms.iter().enumerate() {
            function.body.push(Inst::Label(arm_labels[i].clone()));

            // Generate pattern check
            let pattern_matches = self.generate_pattern_check(&arm.pattern, discriminant.clone(), discriminant_type.clone(), function);

            // Generate guard condition if present
            let condition = if let Some(guard) = &arm.guard {
                let (guard_value, _) = self.generate_expression_ir(guard.clone(), function);
                // Combine pattern match and guard with AND
                let combined_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function.body.push(Inst::And {
                    result: combined_reg.clone(),
                    left: pattern_matches,
                    right: guard_value,
                });
                combined_reg
            } else {
                pattern_matches
            };

            // Branch based on condition
            let next_label = if i + 1 < arms.len() {
                arm_labels[i + 1].clone()
            } else {
                end_label.clone()
            };

            function.body.push(Inst::Branch {
                condition,
                true_label: arm_body_labels[i].clone(),
                false_label: next_label,
            });

            // Arm body
            function.body.push(Inst::Label(arm_body_labels[i].clone()));
            let (arm_result, _) = self.generate_expression_ir(arm.body.clone(), function);
            
            // Store result
            let temp_ptr = Value::Reg(self.next_ptr);
            self.next_ptr += 1;
            function.body.push(Inst::Alloca(temp_ptr.clone(), format!("match_result_{}", i)));
            function.body.push(Inst::Store(temp_ptr.clone(), arm_result));
            function.body.push(Inst::Load(result_reg.clone(), temp_ptr));

            function.body.push(Inst::Jump(end_label.clone()));
        }
    }

    // Generate pattern bindings for enum variants
    fn generate_pattern_bindings(&mut self, pattern: &Pattern, discriminant: Value, enum_name: String, function: &mut Function) {
        match pattern {
            Pattern::Enum { variant, data } => {
                if let Some(data_pattern) = data {
                    // Get variant index
                    let enum_def = self.enum_definitions.get(&enum_name).unwrap();
                    let variant_index = *enum_def.variant_indices.get(variant).unwrap();

                    // Extract variant data and bind to pattern
                    self.generate_variant_data_bindings(data_pattern, discriminant, variant_index, function);
                }
            }
            Pattern::Binding { name, pattern } => {
                // Bind the discriminant to the name
                let ptr_reg = Value::Reg(self.next_ptr);
                self.next_ptr += 1;
                function.body.push(Inst::Alloca(ptr_reg.clone(), name.clone()));
                function.body.push(Inst::Store(ptr_reg.clone(), discriminant.clone()));
                self.symbol_table.insert(name.clone(), (ptr_reg, Ty::Int)); // TODO: Use proper type

                // Process nested pattern
                self.generate_pattern_bindings(pattern, discriminant, enum_name, function);
            }
            _ => {
                // Other patterns don't create bindings in this context
            }
        }
    }

    // Generate variant data bindings
    fn generate_variant_data_bindings(&mut self, pattern: &Pattern, enum_value: Value, variant_index: usize, function: &mut Function) {
        match pattern {
            Pattern::Identifier(name) => {
                // Extract single data value from variant
                let data_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function.body.push(Inst::EnumExtract {
                    result: data_reg.clone(),
                    enum_ptr: enum_value,
                    variant_index,
                    data_index: 0,
                });

                // Bind to variable
                let ptr_reg = Value::Reg(self.next_ptr);
                self.next_ptr += 1;
                function.body.push(Inst::Alloca(ptr_reg.clone(), name.clone()));
                function.body.push(Inst::Store(ptr_reg.clone(), data_reg));
                self.symbol_table.insert(name.clone(), (ptr_reg, Ty::Int)); // TODO: Use proper type
            }
            Pattern::Tuple(patterns) => {
                // Extract multiple data values from variant
                for (i, sub_pattern) in patterns.iter().enumerate() {
                    let data_reg = Value::Reg(self.next_reg);
                    self.next_reg += 1;
                    function.body.push(Inst::EnumExtract {
                        result: data_reg.clone(),
                        enum_ptr: enum_value.clone(),
                        variant_index,
                        data_index: i,
                    });

                    // Recursively handle sub-pattern
                    if let Pattern::Identifier(name) = sub_pattern {
                        let ptr_reg = Value::Reg(self.next_ptr);
                        self.next_ptr += 1;
                        function.body.push(Inst::Alloca(ptr_reg.clone(), name.clone()));
                        function.body.push(Inst::Store(ptr_reg.clone(), data_reg));
                        self.symbol_table.insert(name.clone(), (ptr_reg, Ty::Int)); // TODO: Use proper type
                    }
                }
            }
            _ => {
                // Other patterns not implemented yet
            }
        }
    }

    // Generate pattern check for simple patterns
    fn generate_pattern_check(&mut self, pattern: &Pattern, value: Value, value_type: Ty, function: &mut Function) -> Value {
        match pattern {
            Pattern::Literal(expr) => {
                // Generate literal value and compare
                let (literal_value, _) = self.generate_expression_ir(expr.clone(), function);
                let result_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;

                match value_type {
                    Ty::Int => {
                        function.body.push(Inst::ICmp {
                            op: "eq".to_string(),
                            result: result_reg.clone(),
                            left: value,
                            right: literal_value,
                        });
                    }
                    Ty::Float => {
                        function.body.push(Inst::FCmp {
                            op: "oeq".to_string(),
                            result: result_reg.clone(),
                            left: value,
                            right: literal_value,
                        });
                    }
                    _ => {
                        // Default to integer comparison
                        function.body.push(Inst::ICmp {
                            op: "eq".to_string(),
                            result: result_reg.clone(),
                            left: value,
                            right: literal_value,
                        });
                    }
                }

                result_reg
            }
            Pattern::Range { start, end, inclusive } => {
                // Generate range check
                let (start_value, _) = if let Pattern::Literal(expr) = start.as_ref() {
                    self.generate_expression_ir(expr.clone(), function)
                } else {
                    (Value::ImmInt(0), Ty::Int) // Default
                };

                let (end_value, _) = if let Pattern::Literal(expr) = end.as_ref() {
                    self.generate_expression_ir(expr.clone(), function)
                } else {
                    (Value::ImmInt(100), Ty::Int) // Default
                };

                // Generate start <= value
                let start_check_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function.body.push(Inst::ICmp {
                    op: "sle".to_string(),
                    result: start_check_reg.clone(),
                    left: start_value,
                    right: value.clone(),
                });

                // Generate value <= end (or value < end for exclusive)
                let end_check_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                let end_op = if *inclusive { "sle" } else { "slt" };
                function.body.push(Inst::ICmp {
                    op: end_op.to_string(),
                    result: end_check_reg.clone(),
                    left: value,
                    right: end_value,
                });

                // Combine with AND
                let result_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function.body.push(Inst::And {
                    result: result_reg.clone(),
                    left: start_check_reg,
                    right: end_check_reg,
                });

                result_reg
            }
            Pattern::Wildcard => {
                // Wildcard always matches
                Value::ImmInt(1) // True
            }
            Pattern::Identifier(_) => {
                // Identifier patterns always match (they bind the value)
                Value::ImmInt(1) // True
            }
            _ => {
                // Other patterns not implemented yet, default to false
                Value::ImmInt(0) // False
            }
        }
    }

    // Helper method to convert AST Type to string
    fn type_to_string(&self, ty: &Type) -> String {
        match ty {
            Type::Named(name) => name.clone(),
            Type::Generic { name, .. } => name.clone(),
            Type::Array { element_type, size } => {
                if let Some(s) = size {
                    format!("[{}; {}]", self.type_to_string(element_type), s)
                } else {
                    format!("[{}]", self.type_to_string(element_type))
                }
            }
            Type::Slice { element_type } => {
                format!("&[{}]", self.type_to_string(element_type))
            }
            Type::Vec { element_type } => {
                format!("Vec<{}>", self.type_to_string(element_type))
            }
            Type::HashMap { key_type, value_type } => {
                format!("HashMap<{}, {}>", self.type_to_string(key_type), self.type_to_string(value_type))
            }
            Type::Reference { mutable, inner_type } => {
                if *mutable {
                    format!("&mut {}", self.type_to_string(inner_type))
                } else {
                    format!("&{}", self.type_to_string(inner_type))
                }
            }
        }
    }

    // Match expression IR generation for function bodies
    fn generate_match_expression_ir_for_function(&mut self, expression: Expression, arms: Vec<crate::ast::MatchArm>, function_body: &mut Vec<Inst>) -> (Value, Ty) {
        // Generate IR for the match expression (discriminant)
        let (discriminant_value, discriminant_type) = self.generate_expression_ir_for_function(expression, function_body);

        // Generate unique labels for match arms and end
        let mut arm_labels = Vec::new();
        let mut arm_body_labels = Vec::new();
        for i in 0..arms.len() {
            arm_labels.push(format!("match_arm_{}_{}", i, self.next_reg));
            arm_body_labels.push(format!("match_body_{}_{}", i, self.next_reg));
            self.next_reg += 1;
        }
        let end_label = format!("match_end_{}", self.next_reg);
        self.next_reg += 1;

        // Result register for the match expression
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;

        // Generate pattern matching logic
        match discriminant_type {
            Ty::Enum(enum_name) => {
                self.generate_enum_match_ir_for_function(discriminant_value, enum_name, arms, arm_labels, arm_body_labels, end_label.clone(), result_reg.clone(), function_body);
            }
            _ => {
                // For non-enum types, generate simple pattern matching
                self.generate_simple_match_ir_for_function(discriminant_value, discriminant_type, arms, arm_labels, arm_body_labels, end_label.clone(), result_reg.clone(), function_body);
            }
        }

        // End label
        function_body.push(Inst::Label(end_label));

        // Return result (assuming int type for now, should be inferred from arms)
        (result_reg, Ty::Int)
    }

    // Generate enum-specific match IR for function bodies
    fn generate_enum_match_ir_for_function(&mut self, discriminant: Value, enum_name: String, arms: Vec<crate::ast::MatchArm>, arm_labels: Vec<String>, arm_body_labels: Vec<String>, end_label: String, result_reg: Value, function_body: &mut Vec<Inst>) {
        // Get enum definition
        let enum_def = self.enum_definitions.get(&enum_name)
            .expect(&format!("Undefined enum: {}", enum_name))
            .clone();

        // Extract discriminant from enum value
        let disc_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        function_body.push(Inst::EnumDiscriminant {
            result: disc_reg.clone(),
            enum_ptr: discriminant.clone(),
        });

        // Generate switch instruction for efficient pattern matching
        let mut switch_cases = Vec::new();
        let mut default_label = None;

        for (i, arm) in arms.iter().enumerate() {
            match &arm.pattern {
                crate::ast::Pattern::Enum { variant, .. } => {
                    if let Some(&variant_index) = enum_def.variant_indices.get(variant) {
                        switch_cases.push((variant_index as i64, arm_labels[i].clone()));
                    }
                }
                crate::ast::Pattern::Wildcard => {
                    default_label = Some(arm_labels[i].clone());
                }
                _ => {
                    // For other patterns, use the first arm label as default
                    if default_label.is_none() {
                        default_label = Some(arm_labels[i].clone());
                    }
                }
            }
        }

        // Generate switch instruction
        function_body.push(Inst::Switch {
            discriminant: disc_reg,
            cases: switch_cases,
            default_label: default_label.unwrap_or_else(|| format!("match_default_{}", self.next_reg)),
        });

        // Generate code for each arm
        for (i, arm) in arms.iter().enumerate() {
            // Arm label
            function_body.push(Inst::Label(arm_labels[i].clone()));

            // Generate pattern binding extraction
            self.generate_pattern_bindings_for_function(&arm.pattern, discriminant.clone(), enum_name.clone(), function_body);

            // Generate guard condition if present
            if let Some(guard) = &arm.guard {
                let (guard_value, _) = self.generate_expression_ir_for_function(guard.clone(), function_body);
                let guard_end_label = format!("guard_end_{}_{}", i, self.next_reg);
                self.next_reg += 1;
                
                function_body.push(Inst::Branch {
                    condition: guard_value,
                    true_label: arm_body_labels[i].clone(),
                    false_label: guard_end_label.clone(),
                });
                
                // If guard fails, continue to next arm or end
                function_body.push(Inst::Label(guard_end_label));
                let next_label = if i + 1 < arms.len() {
                    arm_labels[i + 1].clone()
                } else {
                    end_label.clone()
                };
                function_body.push(Inst::Jump(next_label));
            } else {
                // No guard, jump directly to body
                function_body.push(Inst::Jump(arm_body_labels[i].clone()));
            }

            // Arm body label
            function_body.push(Inst::Label(arm_body_labels[i].clone()));

            // Generate arm body
            let (arm_result, _) = self.generate_expression_ir_for_function(arm.body.clone(), function_body);
            
            // Store result in match result register
            let temp_ptr = Value::Reg(self.next_ptr);
            self.next_ptr += 1;
            function_body.push(Inst::Alloca(temp_ptr.clone(), format!("match_result_{}", i)));
            function_body.push(Inst::Store(temp_ptr.clone(), arm_result));
            function_body.push(Inst::Load(result_reg.clone(), temp_ptr));

            // Jump to end
            function_body.push(Inst::Jump(end_label.clone()));
        }
    }

    // Generate simple pattern matching for non-enum types in function bodies
    fn generate_simple_match_ir_for_function(&mut self, discriminant: Value, discriminant_type: Ty, arms: Vec<crate::ast::MatchArm>, arm_labels: Vec<String>, arm_body_labels: Vec<String>, end_label: String, result_reg: Value, function_body: &mut Vec<Inst>) {
        // For simple types, generate if-else chain
        for (i, arm) in arms.iter().enumerate() {
            function_body.push(Inst::Label(arm_labels[i].clone()));

            // Generate pattern check
            let pattern_matches = self.generate_pattern_check_for_function(&arm.pattern, discriminant.clone(), discriminant_type.clone(), function_body);

            // Generate guard condition if present
            let condition = if let Some(guard) = &arm.guard {
                let (guard_value, _) = self.generate_expression_ir_for_function(guard.clone(), function_body);
                // Combine pattern match and guard with AND
                let combined_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function_body.push(Inst::And {
                    result: combined_reg.clone(),
                    left: pattern_matches,
                    right: guard_value,
                });
                combined_reg
            } else {
                pattern_matches
            };

            // Branch based on condition
            let next_label = if i + 1 < arms.len() {
                arm_labels[i + 1].clone()
            } else {
                end_label.clone()
            };

            function_body.push(Inst::Branch {
                condition,
                true_label: arm_body_labels[i].clone(),
                false_label: next_label,
            });

            // Arm body
            function_body.push(Inst::Label(arm_body_labels[i].clone()));
            let (arm_result, _) = self.generate_expression_ir_for_function(arm.body.clone(), function_body);
            
            // Store result
            let temp_ptr = Value::Reg(self.next_ptr);
            self.next_ptr += 1;
            function_body.push(Inst::Alloca(temp_ptr.clone(), format!("match_result_{}", i)));
            function_body.push(Inst::Store(temp_ptr.clone(), arm_result));
            function_body.push(Inst::Load(result_reg.clone(), temp_ptr));

            function_body.push(Inst::Jump(end_label.clone()));
        }
    }

    // Generate pattern bindings for enum variants in function bodies
    fn generate_pattern_bindings_for_function(&mut self, pattern: &crate::ast::Pattern, discriminant: Value, enum_name: String, function_body: &mut Vec<Inst>) {
        match pattern {
            crate::ast::Pattern::Enum { variant, data } => {
                if let Some(data_pattern) = data {
                    // Get variant index
                    let enum_def = self.enum_definitions.get(&enum_name).unwrap();
                    let variant_index = *enum_def.variant_indices.get(variant).unwrap();

                    // Extract variant data and bind to pattern
                    self.generate_variant_data_bindings_for_function(data_pattern, discriminant, variant_index, function_body);
                }
            }
            crate::ast::Pattern::Binding { name, pattern } => {
                // Bind the discriminant to the name
                let ptr_reg = Value::Reg(self.next_ptr);
                self.next_ptr += 1;
                function_body.push(Inst::Alloca(ptr_reg.clone(), name.clone()));
                function_body.push(Inst::Store(ptr_reg.clone(), discriminant.clone()));
                self.symbol_table.insert(name.clone(), (ptr_reg, Ty::Int)); // TODO: Use proper type

                // Process nested pattern
                self.generate_pattern_bindings_for_function(pattern, discriminant, enum_name, function_body);
            }
            _ => {
                // Other patterns don't create bindings in this context
            }
        }
    }

    // Generate variant data bindings for function bodies
    fn generate_variant_data_bindings_for_function(&mut self, pattern: &crate::ast::Pattern, enum_value: Value, variant_index: usize, function_body: &mut Vec<Inst>) {
        match pattern {
            crate::ast::Pattern::Identifier(name) => {
                // Extract single data value from variant
                let data_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function_body.push(Inst::EnumExtract {
                    result: data_reg.clone(),
                    enum_ptr: enum_value,
                    variant_index,
                    data_index: 0,
                });

                // Bind to variable
                let ptr_reg = Value::Reg(self.next_ptr);
                self.next_ptr += 1;
                function_body.push(Inst::Alloca(ptr_reg.clone(), name.clone()));
                function_body.push(Inst::Store(ptr_reg.clone(), data_reg));
                self.symbol_table.insert(name.clone(), (ptr_reg, Ty::Int)); // TODO: Use proper type
            }
            crate::ast::Pattern::Tuple(patterns) => {
                // Extract multiple data values from variant
                for (i, sub_pattern) in patterns.iter().enumerate() {
                    let data_reg = Value::Reg(self.next_reg);
                    self.next_reg += 1;
                    function_body.push(Inst::EnumExtract {
                        result: data_reg.clone(),
                        enum_ptr: enum_value.clone(),
                        variant_index,
                        data_index: i,
                    });

                    // Recursively handle sub-pattern
                    if let crate::ast::Pattern::Identifier(name) = sub_pattern {
                        let ptr_reg = Value::Reg(self.next_ptr);
                        self.next_ptr += 1;
                        function_body.push(Inst::Alloca(ptr_reg.clone(), name.clone()));
                        function_body.push(Inst::Store(ptr_reg.clone(), data_reg));
                        self.symbol_table.insert(name.clone(), (ptr_reg, Ty::Int)); // TODO: Use proper type
                    }
                }
            }
            _ => {
                // Other patterns not implemented yet
            }
        }
    }

    // Generate pattern check for simple patterns in function bodies
    fn generate_pattern_check_for_function(&mut self, pattern: &crate::ast::Pattern, value: Value, value_type: Ty, function_body: &mut Vec<Inst>) -> Value {
        match pattern {
            crate::ast::Pattern::Literal(expr) => {
                // Generate literal value and compare
                let (literal_value, _) = self.generate_expression_ir_for_function(expr.clone(), function_body);
                let result_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;

                match value_type {
                    Ty::Int => {
                        function_body.push(Inst::ICmp {
                            op: "eq".to_string(),
                            result: result_reg.clone(),
                            left: value,
                            right: literal_value,
                        });
                    }
                    Ty::Float => {
                        function_body.push(Inst::FCmp {
                            op: "oeq".to_string(),
                            result: result_reg.clone(),
                            left: value,
                            right: literal_value,
                        });
                    }
                    _ => {
                        // Default to integer comparison
                        function_body.push(Inst::ICmp {
                            op: "eq".to_string(),
                            result: result_reg.clone(),
                            left: value,
                            right: literal_value,
                        });
                    }
                }

                result_reg
            }
            crate::ast::Pattern::Range { start, end, inclusive } => {
                // Generate range check
                let (start_value, _) = if let crate::ast::Pattern::Literal(expr) = start.as_ref() {
                    self.generate_expression_ir_for_function(expr.clone(), function_body)
                } else {
                    (Value::ImmInt(0), Ty::Int) // Default
                };

                let (end_value, _) = if let crate::ast::Pattern::Literal(expr) = end.as_ref() {
                    self.generate_expression_ir_for_function(expr.clone(), function_body)
                } else {
                    (Value::ImmInt(100), Ty::Int) // Default
                };

                // Generate start <= value
                let start_check_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function_body.push(Inst::ICmp {
                    op: "sle".to_string(),
                    result: start_check_reg.clone(),
                    left: start_value,
                    right: value.clone(),
                });

                // Generate value <= end (or value < end for exclusive)
                let end_check_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                let end_op = if *inclusive { "sle" } else { "slt" };
                function_body.push(Inst::ICmp {
                    op: end_op.to_string(),
                    result: end_check_reg.clone(),
                    left: value,
                    right: end_value,
                });

                // Combine with AND
                let result_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function_body.push(Inst::And {
                    result: result_reg.clone(),
                    left: start_check_reg,
                    right: end_check_reg,
                });

                result_reg
            }
            crate::ast::Pattern::Wildcard => {
                // Wildcard always matches
                Value::ImmInt(1) // True
            }
            crate::ast::Pattern::Identifier(_) => {
                // Identifier patterns always match (they bind the value)
                Value::ImmInt(1) // True
            }
            _ => {
                // Other patterns not implemented yet, default to false
                Value::ImmInt(0) // False
            }
        }
    }

    // Array literal IR generation
    fn generate_array_literal_ir(&mut self, elements: Vec<Expression>, function: &mut Function) -> (Value, Ty) {
        // Generate IR for all elements
        let mut element_values = Vec::new();
        let mut element_type = Ty::Int; // Default type
        
        for (i, element) in elements.iter().enumerate() {
            let (element_value, elem_type) = self.generate_expression_ir(element.clone(), function);
            element_values.push(element_value);
            
            // Use the first element's type as the array element type
            if i == 0 {
                element_type = elem_type;
            }
        }
        
        // Allocate array
        let array_ptr = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        let size_value = Value::ImmInt(elements.len() as i64);
        function.body.push(Inst::ArrayAlloca {
            result: array_ptr.clone(),
            element_type: element_type.to_string(),
            size: size_value,
        });
        
        // Initialize array with elements
        function.body.push(Inst::ArrayInit {
            result: array_ptr.clone(),
            element_type: element_type.to_string(),
            elements: element_values,
        });
        
        (array_ptr, Ty::Array(Box::new(element_type), Some(elements.len())))
    }
    
    // Array access IR generation with bounds checking
    fn generate_array_access_ir(&mut self, array: Expression, index: Expression, function: &mut Function) -> (Value, Ty) {
        // Generate IR for array and index expressions
        let (array_value, array_type) = self.generate_expression_ir(array, function);
        let (index_value, _) = self.generate_expression_ir(index, function);
        
        // Extract element type from array type
        let element_type = match array_type {
            Ty::Array(elem_type, _) => *elem_type,
            _ => Ty::Int, // Default fallback
        };
        
        // Generate bounds check
        let success_label = format!("bounds_check_success_{}", self.next_reg);
        let failure_label = format!("bounds_check_failure_{}", self.next_reg);
        self.next_reg += 1;
        
        function.body.push(Inst::BoundsCheck {
            array_ptr: array_value.clone(),
            index: index_value.clone(),
            success_label: success_label.clone(),
            failure_label: failure_label.clone(),
        });
        
        // Failure label - could generate runtime error
        function.body.push(Inst::Label(failure_label));
        // For now, just return 0 on bounds failure
        let error_result = Value::ImmInt(0);
        let end_label = format!("array_access_end_{}", self.next_reg);
        self.next_reg += 1;
        function.body.push(Inst::Jump(end_label.clone()));
        
        // Success label - perform actual access
        function.body.push(Inst::Label(success_label));
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        function.body.push(Inst::ArrayAccess {
            result: result_reg.clone(),
            array_ptr: array_value,
            index: index_value,
        });
        
        function.body.push(Inst::Label(end_label));
        
        (result_reg, element_type)
    }
    
    // Vec macro IR generation
    fn generate_vec_macro_ir(&mut self, elements: Vec<Expression>, function: &mut Function) -> (Value, Ty) {
        // Generate IR for all elements
        let mut element_values = Vec::new();
        let mut element_type = Ty::Int; // Default type
        
        for (i, element) in elements.iter().enumerate() {
            let (element_value, elem_type) = self.generate_expression_ir(element.clone(), function);
            element_values.push(element_value);
            
            // Use the first element's type as the Vec element type
            if i == 0 {
                element_type = elem_type;
            }
        }
        
        // Allocate Vec
        let vec_ptr = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        function.body.push(Inst::VecAlloca {
            result: vec_ptr.clone(),
            element_type: element_type.to_string(),
        });
        
        // Initialize Vec with elements
        function.body.push(Inst::VecInit {
            result: vec_ptr.clone(),
            element_type: element_type.to_string(),
            elements: element_values,
        });
        
        (vec_ptr, Ty::Vec(Box::new(element_type)))
    }
    
    // Generic type instantiation IR generation
    fn generate_generic_instantiation_ir(&mut self, base_type: String, type_args: Vec<String>, function: &mut Function) -> (Value, Ty) {
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        let instantiated_name = format!("{}<{}>", base_type, type_args.join(", "));
        
        function.body.push(Inst::GenericInstantiate {
            result: result_reg.clone(),
            base_type,
            type_args,
            instantiated_name: instantiated_name.clone(),
        });
        
        // For now, return a generic type representation
        (result_reg, Ty::Int) // TODO: Implement proper generic type representation
    }
    
    // Collection method call IR generation
    fn generate_collection_method_ir(&mut self, object: Expression, method: String, arguments: Vec<Expression>, function: &mut Function) -> (Value, Ty) {
        let (object_value, object_type) = self.generate_expression_ir(object, function);
        
        // Generate IR for arguments
        let mut arg_values = Vec::new();
        for arg in arguments {
            let (arg_value, _) = self.generate_expression_ir(arg, function);
            arg_values.push(arg_value);
        }
        
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        match (&object_type, method.as_str()) {
            (Ty::Vec(_), "push") => {
                if let Some(value) = arg_values.first() {
                    function.body.push(Inst::VecPush {
                        vec_ptr: object_value,
                        value: value.clone(),
                    });
                }
                (Value::ImmInt(0), Ty::Int) // push returns unit
            }
            (Ty::Vec(_), "pop") => {
                function.body.push(Inst::VecPop {
                    result: result_reg.clone(),
                    vec_ptr: object_value,
                });
                (result_reg, Ty::Int) // TODO: Return Option<T>
            }
            (Ty::Vec(_), "len") => {
                function.body.push(Inst::VecLength {
                    result: result_reg.clone(),
                    vec_ptr: object_value,
                });
                (result_reg, Ty::Int)
            }
            (Ty::Vec(_), "capacity") => {
                function.body.push(Inst::VecCapacity {
                    result: result_reg.clone(),
                    vec_ptr: object_value,
                });
                (result_reg, Ty::Int)
            }
            (Ty::Array(elem_type, _), "len") => {
                function.body.push(Inst::ArrayLength {
                    result: result_reg.clone(),
                    array_ptr: object_value,
                });
                (result_reg, Ty::Int)
            }
            _ => {
                // Generic method call
                function.body.push(Inst::GenericMethodCall {
                    result: Some(result_reg.clone()),
                    object: object_value,
                    method,
                    type_args: vec![], // TODO: Extract type arguments
                    arguments: arg_values,
                });
                (result_reg, Ty::Int) // Default return type
            }
        }
    }
    
    // Array literal IR generation for function bodies
    fn generate_array_literal_ir_for_function(&mut self, elements: Vec<Expression>, function_body: &mut Vec<Inst>) -> (Value, Ty) {
        // Generate IR for all elements
        let mut element_values = Vec::new();
        let mut element_type = Ty::Int; // Default type
        
        for (i, element) in elements.iter().enumerate() {
            let (element_value, elem_type) = self.generate_expression_ir_for_function(element.clone(), function_body);
            element_values.push(element_value);
            
            // Use the first element's type as the array element type
            if i == 0 {
                element_type = elem_type;
            }
        }
        
        // Allocate array
        let array_ptr = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        let size_value = Value::ImmInt(elements.len() as i64);
        function_body.push(Inst::ArrayAlloca {
            result: array_ptr.clone(),
            element_type: element_type.to_string(),
            size: size_value,
        });
        
        // Initialize array with elements
        function_body.push(Inst::ArrayInit {
            result: array_ptr.clone(),
            element_type: element_type.to_string(),
            elements: element_values,
        });
        
        (array_ptr, Ty::Array(Box::new(element_type), Some(elements.len())))
    }
    
    // Array access IR generation for function bodies
    fn generate_array_access_ir_for_function(&mut self, array: Expression, index: Expression, function_body: &mut Vec<Inst>) -> (Value, Ty) {
        // Generate IR for array and index expressions
        let (array_value, array_type) = self.generate_expression_ir_for_function(array, function_body);
        let (index_value, _) = self.generate_expression_ir_for_function(index, function_body);
        
        // Extract element type from array type
        let element_type = match array_type {
            Ty::Array(elem_type, _) => *elem_type,
            _ => Ty::Int, // Default fallback
        };
        
        // Generate bounds check
        let success_label = format!("bounds_check_success_{}", self.next_reg);
        let failure_label = format!("bounds_check_failure_{}", self.next_reg);
        self.next_reg += 1;
        
        function_body.push(Inst::BoundsCheck {
            array_ptr: array_value.clone(),
            index: index_value.clone(),
            success_label: success_label.clone(),
            failure_label: failure_label.clone(),
        });
        
        // Failure label - could generate runtime error
        function_body.push(Inst::Label(failure_label));
        let end_label = format!("array_access_end_{}", self.next_reg);
        self.next_reg += 1;
        function_body.push(Inst::Jump(end_label.clone()));
        
        // Success label - perform actual access
        function_body.push(Inst::Label(success_label));
        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        function_body.push(Inst::ArrayAccess {
            result: result_reg.clone(),
            array_ptr: array_value,
            index: index_value,
        });
        
        function_body.push(Inst::Label(end_label));
        
        (result_reg, element_type)
    }
    
    // Vec macro IR generation for function bodies
    fn generate_vec_macro_ir_for_function(&mut self, elements: Vec<Expression>, function_body: &mut Vec<Inst>) -> (Value, Ty) {
        // Generate IR for all elements
        let mut element_values = Vec::new();
        let mut element_type = Ty::Int; // Default type
        
        for (i, element) in elements.iter().enumerate() {
            let (element_value, elem_type) = self.generate_expression_ir_for_function(element.clone(), function_body);
            element_values.push(element_value);
            
            // Use the first element's type as the Vec element type
            if i == 0 {
                element_type = elem_type;
            }
        }
        
        // Allocate Vec
        let vec_ptr = Value::Reg(self.next_reg);
        self.next_reg += 1;
        
        function_body.push(Inst::VecAlloca {
            result: vec_ptr.clone(),
            element_type: element_type.to_string(),
        });
        
        // Initialize Vec with elements
        function_body.push(Inst::VecInit {
            result: vec_ptr.clone(),
            element_type: element_type.to_string(),
            elements: element_values,
        });
        
        (vec_ptr, Ty::Vec(Box::new(element_type)))
    }
}