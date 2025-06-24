use crate::ast::{AstNode, Expression, Statement};
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
            Statement::Let { name, value } => {
                let (expr_value, expr_type) = self.generate_expression_ir(value, current_function);

                // Allocate a stack slot for the variable
                let ptr_reg = Value::Reg(self.next_ptr);
                self.next_ptr += 1;
                current_function.body.push(Inst::Alloca(ptr_reg.clone(), name.clone()));
                self.symbol_table.insert(name, (ptr_reg.clone(), expr_type));

                // Store the expression result into the allocated slot
                current_function.body.push(Inst::Store(ptr_reg, expr_value));
            }
            Statement::Return(expr) => {
                let (return_value, _) = self.generate_expression_ir(expr, current_function);
                current_function.body.push(Inst::Return(return_value));
            }
        }
    }

    fn generate_expression_ir(&mut self, expr: Expression, function: &mut Function) -> (Value, Ty) {
        match expr {
            Expression::Number(n) => (Value::ImmInt(n), Ty::Int),
            Expression::Float(f) => (Value::ImmFloat(f), Ty::Float),
            Expression::Identifier(name) => {
                let (ptr_reg, var_type) = self.symbol_table.get(&name).expect("Undeclared variable").clone();
                let result_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function.body.push(Inst::Load(result_reg.clone(), ptr_reg));
                (result_reg, var_type)
            }
            Expression::Binary { op, lhs, rhs, ty } => {
                let (lhs_val, lhs_type) = self.generate_expression_ir(*lhs, function);
                let (rhs_val, rhs_type) = self.generate_expression_ir(*rhs, function);
                
                // Get the result type from the AST (set by semantic analysis)
                let result_type = ty.expect("Binary expression should have type set by semantic analysis");
                
                // Handle type promotion if needed
                let (promoted_lhs, promoted_rhs) = self.handle_type_promotion(
                    lhs_val, lhs_type, rhs_val, rhs_type, &result_type, function
                );

                // Try constant folding first
                if let (Some(folded_value), Some(folded_type)) = self.try_constant_fold(&op, &promoted_lhs, &promoted_rhs, &result_type) {
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
                    _ => panic!("Unsupported binary operation: {} for type {:?}", op, result_type),
                };
                
                function.body.push(inst);
                (result_reg, result_type)
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
}

