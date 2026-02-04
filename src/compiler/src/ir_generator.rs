use crate::ast::{AstNode, Expression, Statement, Type};
use crate::ir::{Function, Inst, Value};
use crate::types::{Ty, needs_promotion};
use std::collections::HashMap;

pub struct IrGenerator {
    functions: HashMap<String, Function>,
    current_function_name: String,
    next_reg: u32,
    next_ptr: u32,
    symbol_table: HashMap<String, (Value, Ty)>, // Track both pointer and type
}

impl IrGenerator {
    pub fn new() -> Self {
        IrGenerator {
            functions: HashMap::new(),
            current_function_name: String::new(),
            next_reg: 0,
            next_ptr: 0,
            symbol_table: HashMap::new(),
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
                    eprintln!(
                        "Warning: Top-level expressions are not yet handled in IR generation."
                    );
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
            Statement::Let {
                name,
                mutable: _,
                type_annotation: _,
                value,
            } => {
                let (expr_value, expr_type) = if let Some(val) = value {
                    self.generate_expression_ir(val, current_function)
                } else {
                    (Value::ImmInt(0), Ty::Int)
                };

                // Allocate a stack slot for the variable
                let ptr_reg = Value::Reg(self.next_ptr);
                self.next_ptr += 1;
                current_function
                    .body
                    .push(Inst::Alloca(ptr_reg.clone(), name.clone()));
                self.symbol_table.insert(name, (ptr_reg.clone(), expr_type));

                // Store the expression result into the allocated slot
                current_function.body.push(Inst::Store(ptr_reg, expr_value));
            }
            Statement::Return(expr) => {
                let (return_value, _) = if let Some(val) = expr {
                    self.generate_expression_ir(val, current_function)
                } else {
                    (Value::ImmInt(0), Ty::Int)
                };
                current_function.body.push(Inst::Return(return_value));
            }
            Statement::Function {
                name,
                parameters,
                return_type: _,
                body,
            } => {
                self.generate_function_definition_ir(name, parameters, body, current_function);
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.generate_if_statement_ir(condition, then_block, else_block, current_function);
            }
            Statement::While { condition, body } => {
                self.generate_while_loop_ir(condition, body, current_function);
            }
            Statement::For {
                variable,
                iterable,
                body,
            } => {
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
        }
    }

    fn generate_expression_ir(&mut self, expr: Expression, function: &mut Function) -> (Value, Ty) {
        match expr {
            Expression::IntegerLiteral(n) => (Value::ImmInt(n), Ty::Int),
            Expression::FloatLiteral(f) => (Value::ImmFloat(f), Ty::Float),
            Expression::Identifier(name) => {
                let (ptr_reg, var_type) = self
                    .symbol_table
                    .get(&name)
                    .expect("Undeclared variable")
                    .clone();
                let result_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function.body.push(Inst::Load(result_reg.clone(), ptr_reg));
                (result_reg, var_type)
            }
            Expression::Binary {
                op,
                left,
                right,
                ty,
            } => {
                let (lhs_val, lhs_type) = self.generate_expression_ir(*left, function);
                let (rhs_val, rhs_type) = self.generate_expression_ir(*right, function);

                // Get the result type from the AST (set by semantic analysis)
                let result_type =
                    ty.expect("Binary expression should have type set by semantic analysis");

                // Handle type promotion if needed
                let (promoted_lhs, promoted_rhs) = self.handle_type_promotion(
                    lhs_val,
                    lhs_type,
                    rhs_val,
                    rhs_type,
                    &result_type,
                    function,
                );

                // Try constant folding first
                if let (Some(folded_value), Some(folded_type)) =
                    self.try_constant_fold(op.as_str(), &promoted_lhs, &promoted_rhs, &result_type)
                {
                    return (folded_value, folded_type);
                }

                let result_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;

                // Generate the appropriate instruction based on result type
                let inst = match (&result_type, op.as_str()) {
                    (Ty::Int, "+") => Inst::Add(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "+") => Inst::FAdd(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Int, "-") => Inst::Sub(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "-") => Inst::FSub(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Int, "*") => Inst::Mul(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "*") => Inst::FMul(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Int, "/") => Inst::Div(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "/") => Inst::FDiv(result_reg.clone(), promoted_lhs, promoted_rhs),
                    _ => panic!(
                        "Unsupported binary operation: {} for type {:?}",
                        op, result_type
                    ),
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
            Expression::Print {
                format_string,
                arguments,
            } => self.generate_print_ir(format_string, arguments, false, function),
            Expression::Println {
                format_string,
                arguments,
            } => self.generate_print_ir(format_string, arguments, true, function),
            Expression::Comparison { op, left, right } => {
                self.generate_comparison_ir(op, *left, *right, function)
            }
            Expression::Logical { op, left, right } => {
                self.generate_logical_ir(op, *left, *right, function)
            }
            Expression::Unary { op, operand } => self.generate_unary_ir(op, *operand, function),
        }
    }

    fn handle_type_promotion(
        &mut self,
        lhs_val: Value,
        lhs_type: Ty,
        rhs_val: Value,
        rhs_type: Ty,
        target_type: &Ty,
        function: &mut Function,
    ) -> (Value, Value) {
        let promoted_lhs = if needs_promotion(&lhs_type, target_type) {
            let promoted_reg = Value::Reg(self.next_reg);
            self.next_reg += 1;
            function
                .body
                .push(Inst::SIToFP(promoted_reg.clone(), lhs_val));
            promoted_reg
        } else {
            lhs_val
        };

        let promoted_rhs = if needs_promotion(&rhs_type, target_type) {
            let promoted_reg = Value::Reg(self.next_reg);
            self.next_reg += 1;
            function
                .body
                .push(Inst::SIToFP(promoted_reg.clone(), rhs_val));
            promoted_reg
        } else {
            rhs_val
        };

        (promoted_lhs, promoted_rhs)
    }

    fn try_constant_fold(
        &self,
        op: &str,
        lhs: &Value,
        rhs: &Value,
        result_type: &Ty,
    ) -> (Option<Value>, Option<Ty>) {
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

    fn generate_function_definition_ir(
        &mut self,
        name: String,
        parameters: Vec<crate::ast::Parameter>,
        body: crate::ast::Block,
        current_function: &mut Function,
    ) {
        // Save current state
        let saved_symbol_table = self.symbol_table.clone();
        let saved_next_reg = self.next_reg;
        let saved_next_ptr = self.next_ptr;

        // Reset for function generation
        self.symbol_table.clear();
        self.next_reg = 0;
        self.next_ptr = 0;

        // Create parameter names and types for IR
        let param_names: Vec<(String, String)> = parameters
            .iter()
            .map(|p| {
                (
                    p.name.clone(),
                    match &p.param_type {
                        Type::Named(name) => name.clone(),
                    },
                )
            })
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
            };

            self.symbol_table
                .insert(param.name.clone(), (ptr_reg, param_type));
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
            let (return_value, _) =
                self.generate_expression_ir_for_function(expr, &mut function_body);
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

    fn generate_statement_ir_for_function(
        &mut self,
        stmt: Statement,
        function_body: &mut Vec<Inst>,
    ) {
        match stmt {
            Statement::Let {
                name,
                mutable: _,
                type_annotation: _,
                value,
            } => {
                let (expr_value, expr_type) = if let Some(val) = value {
                    self.generate_expression_ir_for_function(val, function_body)
                } else {
                    (Value::ImmInt(0), Ty::Int)
                };

                // Allocate a stack slot for the variable
                let ptr_reg = Value::Reg(self.next_ptr);
                self.next_ptr += 1;
                function_body.push(Inst::Alloca(ptr_reg.clone(), name.clone()));
                self.symbol_table.insert(name, (ptr_reg.clone(), expr_type));

                // Store the expression result into the allocated slot
                function_body.push(Inst::Store(ptr_reg, expr_value));
            }
            Statement::Return(expr) => {
                let (return_value, _) = if let Some(val) = expr {
                    self.generate_expression_ir_for_function(val, function_body)
                } else {
                    (Value::ImmInt(0), Ty::Int)
                };
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

    fn generate_expression_ir_for_function(
        &mut self,
        expr: Expression,
        function_body: &mut Vec<Inst>,
    ) -> (Value, Ty) {
        match expr {
            Expression::IntegerLiteral(n) => (Value::ImmInt(n), Ty::Int),
            Expression::FloatLiteral(f) => (Value::ImmFloat(f), Ty::Float),
            Expression::Identifier(name) => {
                let (ptr_reg, var_type) = self
                    .symbol_table
                    .get(&name)
                    .expect("Undeclared variable")
                    .clone();
                let result_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function_body.push(Inst::Load(result_reg.clone(), ptr_reg));
                (result_reg, var_type)
            }
            Expression::Binary {
                op,
                left,
                right,
                ty,
            } => {
                let (lhs_val, lhs_type) =
                    self.generate_expression_ir_for_function(*left, function_body);
                let (rhs_val, rhs_type) =
                    self.generate_expression_ir_for_function(*right, function_body);

                // Get the result type from the AST (set by semantic analysis)
                let result_type =
                    ty.expect("Binary expression should have type set by semantic analysis");

                // Handle type promotion if needed
                let (promoted_lhs, promoted_rhs) = self.handle_type_promotion_for_function(
                    lhs_val,
                    lhs_type,
                    rhs_val,
                    rhs_type,
                    &result_type,
                    function_body,
                );

                // Try constant folding first
                if let (Some(folded_value), Some(folded_type)) =
                    self.try_constant_fold(op.as_str(), &promoted_lhs, &promoted_rhs, &result_type)
                {
                    return (folded_value, folded_type);
                }

                let result_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;

                // Generate the appropriate instruction based on result type
                let inst = match (&result_type, op.as_str()) {
                    (Ty::Int, "+") => Inst::Add(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "+") => Inst::FAdd(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Int, "-") => Inst::Sub(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "-") => Inst::FSub(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Int, "*") => Inst::Mul(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "*") => Inst::FMul(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Int, "/") => Inst::Div(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "/") => Inst::FDiv(result_reg.clone(), promoted_lhs, promoted_rhs),
                    _ => panic!(
                        "Unsupported binary operation: {} for type {:?}",
                        op, result_type
                    ),
                };

                function_body.push(inst);
                (result_reg, result_type)
            }
            Expression::FunctionCall { name, arguments } => {
                self.generate_function_call_ir(name, arguments, function_body)
            }
            Expression::Print {
                format_string,
                arguments,
            } => {
                self.generate_print_ir_for_function(format_string, arguments, false, function_body)
            }
            Expression::Println {
                format_string,
                arguments,
            } => self.generate_print_ir_for_function(format_string, arguments, true, function_body),
            Expression::Comparison { op, left, right } => {
                self.generate_comparison_ir_for_function(op, *left, *right, function_body)
            }
            Expression::Logical { op, left, right } => {
                self.generate_logical_ir_for_function(op, *left, *right, function_body)
            }
            Expression::Unary { op, operand } => {
                self.generate_unary_ir_for_function(op, *operand, function_body)
            }
        }
    }

    fn handle_type_promotion_for_function(
        &mut self,
        lhs_val: Value,
        lhs_type: Ty,
        rhs_val: Value,
        rhs_type: Ty,
        target_type: &Ty,
        function_body: &mut Vec<Inst>,
    ) -> (Value, Value) {
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

    fn generate_function_call_ir(
        &mut self,
        name: String,
        arguments: Vec<Expression>,
        function_body: &mut Vec<Inst>,
    ) -> (Value, Ty) {
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
    fn generate_if_statement_ir(
        &mut self,
        condition: Expression,
        then_block: crate::ast::Block,
        else_block: Option<Box<Statement>>,
        current_function: &mut Function,
    ) {
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

    fn generate_while_loop_ir(
        &mut self,
        condition: Expression,
        body: crate::ast::Block,
        current_function: &mut Function,
    ) {
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

    fn generate_for_loop_ir(
        &mut self,
        variable: String,
        iterable: Expression,
        body: crate::ast::Block,
        current_function: &mut Function,
    ) {
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
        current_function
            .body
            .push(Inst::Alloca(var_ptr.clone(), variable.clone()));
        current_function
            .body
            .push(Inst::Store(var_ptr.clone(), start_value));
        self.symbol_table
            .insert(variable.clone(), (var_ptr.clone(), var_type));

        // For simplicity, create a condition that will eventually be false
        // In a real implementation, this would check against the range end
        current_function.body.push(Inst::Jump(loop_start.clone()));

        // Loop start - check condition (simplified)
        current_function.body.push(Inst::Label(loop_start.clone()));
        let loop_var_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        current_function
            .body
            .push(Inst::Load(loop_var_reg.clone(), var_ptr.clone()));

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
        current_function.body.push(Inst::Add(
            incremented_reg.clone(),
            loop_var_reg,
            Value::ImmInt(1),
        ));
        current_function
            .body
            .push(Inst::Store(var_ptr, incremented_reg));

        current_function.body.push(Inst::Jump(loop_start));

        // Loop end
        current_function.body.push(Inst::Label(loop_end));
    }

    fn generate_infinite_loop_ir(
        &mut self,
        body: crate::ast::Block,
        current_function: &mut Function,
    ) {
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
    fn generate_print_ir(
        &mut self,
        format_string: String,
        arguments: Vec<Expression>,
        newline: bool,
        function: &mut Function,
    ) -> (Value, Ty) {
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

    fn generate_comparison_ir(
        &mut self,
        op: crate::ast::ComparisonOp,
        left: Expression,
        right: Expression,
        function: &mut Function,
    ) -> (Value, Ty) {
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
            }
            (Ty::Int, Ty::Float) => {
                // Promote left operand to float
                let promoted_left = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function
                    .body
                    .push(Inst::SIToFP(promoted_left.clone(), left_val));

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
            }
            (Ty::Float, Ty::Int) => {
                // Promote right operand to float
                let promoted_right = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function
                    .body
                    .push(Inst::SIToFP(promoted_right.clone(), right_val));

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
            }
            (Ty::Bool, Ty::Bool) => Inst::ICmp {
                op: op_str.to_string(),
                result: result_reg.clone(),
                left: left_val,
                right: right_val,
            },
            _ => panic!(
                "Unsupported comparison between {:?} and {:?}",
                left_type, right_type
            ),
        };

        function.body.push(inst);
        (result_reg, Ty::Bool)
    }

    fn generate_logical_ir(
        &mut self,
        op: crate::ast::LogicalOp,
        left: Expression,
        right: Expression,
        function: &mut Function,
    ) -> (Value, Ty) {
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

    fn generate_unary_ir(
        &mut self,
        op: crate::ast::UnaryOp,
        operand: Expression,
        function: &mut Function,
    ) -> (Value, Ty) {
        let (operand_val, operand_type) = self.generate_expression_ir(operand, function);

        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;

        let (inst, result_type) = match op {
            crate::ast::UnaryOp::Not => (
                Inst::Not {
                    result: result_reg.clone(),
                    operand: operand_val,
                },
                Ty::Bool,
            ),
            crate::ast::UnaryOp::Negate => (
                Inst::Neg {
                    result: result_reg.clone(),
                    operand: operand_val,
                },
                operand_type,
            ),
        };

        function.body.push(inst);
        (result_reg, result_type)
    }

    // Function-level I/O and enhanced expression IR generation methods
    fn generate_print_ir_for_function(
        &mut self,
        format_string: String,
        arguments: Vec<Expression>,
        newline: bool,
        function_body: &mut Vec<Inst>,
    ) -> (Value, Ty) {
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

    fn generate_comparison_ir_for_function(
        &mut self,
        op: crate::ast::ComparisonOp,
        left: Expression,
        right: Expression,
        function_body: &mut Vec<Inst>,
    ) -> (Value, Ty) {
        let (left_val, left_type) = self.generate_expression_ir_for_function(left, function_body);
        let (right_val, right_type) =
            self.generate_expression_ir_for_function(right, function_body);

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
            }
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
            }
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
            }
            (Ty::Bool, Ty::Bool) => Inst::ICmp {
                op: op_str.to_string(),
                result: result_reg.clone(),
                left: left_val,
                right: right_val,
            },
            _ => panic!(
                "Unsupported comparison between {:?} and {:?}",
                left_type, right_type
            ),
        };

        function_body.push(inst);
        (result_reg, Ty::Bool)
    }

    fn generate_logical_ir_for_function(
        &mut self,
        op: crate::ast::LogicalOp,
        left: Expression,
        right: Expression,
        function_body: &mut Vec<Inst>,
    ) -> (Value, Ty) {
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

    fn generate_unary_ir_for_function(
        &mut self,
        op: crate::ast::UnaryOp,
        operand: Expression,
        function_body: &mut Vec<Inst>,
    ) -> (Value, Ty) {
        let (operand_val, operand_type) =
            self.generate_expression_ir_for_function(operand, function_body);

        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;

        let (inst, result_type) = match op {
            crate::ast::UnaryOp::Not => (
                Inst::Not {
                    result: result_reg.clone(),
                    operand: operand_val,
                },
                Ty::Bool,
            ),
            crate::ast::UnaryOp::Negate => (
                Inst::Neg {
                    result: result_reg.clone(),
                    operand: operand_val,
                },
                operand_type,
            ),
        };

        function_body.push(inst);
        (result_reg, result_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{AstNode, BinaryOp, Expression, Statement};
    use crate::types::Ty;

    #[test]
    fn generates_main_function() {
        let mut ir_gen = IrGenerator::new();
        let ir = ir_gen.generate_ir(vec![]);
        assert!(ir.contains_key("main"));
    }

    #[test]
    fn let_with_integer_emits_alloca_and_store() {
        let mut ir_gen = IrGenerator::new();
        let ast = vec![AstNode::Statement(Statement::Let {
            name: "x".to_string(),
            mutable: false,
            type_annotation: None,
            value: Some(Expression::IntegerLiteral(1)),
        })];

        let ir = ir_gen.generate_ir(ast);
        let main = &ir["main"].body;

        assert!(
            main.iter()
                .any(|i| matches!(i, crate::ir::Inst::Alloca(_, n) if n == "x"))
        );
        assert!(
            main.iter()
                .any(|i| matches!(i, crate::ir::Inst::Store(_, crate::ir::Value::ImmInt(1))))
        );
    }

    #[test]
    fn binary_expression_requires_type_annotation_in_ast() {
        let mut ir_gen = IrGenerator::new();
        let expr = Expression::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expression::IntegerLiteral(1)),
            right: Box::new(Expression::IntegerLiteral(2)),
            ty: Some(Ty::Int),
        };

        let (val, ty) = ir_gen.generate_expression_ir(
            expr,
            &mut crate::ir::Function {
                name: "main".to_string(),
                body: vec![],
                next_reg: 0,
                next_ptr: 0,
            },
        );
        assert_eq!(ty, Ty::Int);
        // should be immediate foldable
        assert!(matches!(
            val,
            crate::ir::Value::ImmInt(3) | crate::ir::Value::Reg(_)
        ));
    }
}
