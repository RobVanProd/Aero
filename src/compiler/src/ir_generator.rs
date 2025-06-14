use crate::ast::{AstNode, Expression, Statement};
use crate::ir::{Function, Inst, Value};
use std::collections::HashMap;

pub struct IrGenerator {
    functions: HashMap<String, Function>,
    current_function_name: String,
    next_reg: u32,
    next_ptr: u32, // Added for unique pointer IDs
    symbol_table: HashMap<String, Value>, // To track allocated pointers
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
        // For now, assume a single implicit 'main' function
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
        self.functions.insert("main".to_string(), main_function);
        self.functions.clone()
    }

    fn generate_statement_ir(&mut self, stmt: Statement, current_function: &mut Function) {
        match stmt {
            Statement::Let { name, value } => {
                let expr_value = self.generate_expression_ir(value, current_function);

                // Allocate a stack slot for the variable
                let ptr_reg = Value::Reg(current_function.next_ptr);
                current_function.next_ptr += 1;
                current_function.body.push(Inst::Alloca(ptr_reg.clone(), name.clone()));
                self.symbol_table.insert(name, ptr_reg.clone());

                // Store the expression result into the allocated slot
                current_function.body.push(Inst::Store(ptr_reg, expr_value));
            }
            Statement::Return(expr) => {
                let return_value = self.generate_expression_ir(expr, current_function);
                current_function.body.push(Inst::Return(return_value));
            }
        }
    }

    fn generate_expression_ir(&mut self, expr: Expression, function: &mut Function) -> Value {
        match expr {
            Expression::Number(n) => Value::ImmInt(n),
            Expression::Float(f) => Value::ImmFloat(f),
            Expression::Identifier(name) => {
                let ptr_reg = self.symbol_table.get(&name).expect("Undeclared variable").clone();
                let result_reg = Value::Reg(function.next_reg);
                function.next_reg += 1;
                function.body.push(Inst::Load(result_reg.clone(), ptr_reg));
                result_reg
            }
            Expression::Binary { op, lhs, rhs } => {
                let lhs_val = self.generate_expression_ir(*lhs, function);
                let rhs_val = self.generate_expression_ir(*rhs, function);

                // Perform constant folding if both operands are immediate values
                match (lhs_val, rhs_val) {
                    (Value::ImmInt(l), Value::ImmInt(r)) => {
                        let result = match op.as_str() {
                            "+" => l + r,
                            "-" => l - r,
                            "*" => l * r,
                            "/" => l / r,
                            _ => panic!("Unsupported binary operator for integers: {}", op),
                        };
                        Value::ImmInt(result)
                    }
                    (Value::ImmFloat(l), Value::ImmFloat(r)) => {
                        let result = match op.as_str() {
                            "+" => l + r,
                            "-" => l - r,
                            "*" => l * r,
                            "/" => l / r,
                            _ => panic!("Unsupported binary operator for floats: {}", op),
                        };
                        Value::ImmFloat(result)
                    }
                    (lhs_val, rhs_val) => {
                        // If not constant, generate IR instructions
                        let result_reg = Value::Reg(function.next_reg);
                        function.next_reg += 1;

                        let inst = match op.as_str() {
                            "+" => Inst::Add(result_reg.clone(), lhs_val, rhs_val),
                            "-" => Inst::Sub(result_reg.clone(), lhs_val, rhs_val),
                            "*" => Inst::Mul(result_reg.clone(), lhs_val, rhs_val),
                            "/" => Inst::Div(result_reg.clone(), lhs_val, rhs_val),
                            _ => panic!("Unsupported binary operator: {}", op),
                        };
                        function.body.push(inst);
                        result_reg
                    }
                }
            }
        }
    }
}


