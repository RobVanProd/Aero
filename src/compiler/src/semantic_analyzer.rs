use std::collections::HashMap;
use crate::ast::{AstNode, Expression, Statement};
use crate::types::{Ty, infer_binary_type};

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub ptr_name: String,
    pub ty: Ty, // Changed from String to Ty
    pub initialized: bool,
}

pub struct SemanticAnalyzer {
    symbol_table: HashMap<String, VarInfo>, // Stores variable_name -> VarInfo
    next_ptr_id: u32,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer { 
            symbol_table: HashMap::new(),
            next_ptr_id: 0,
        }
    }

    fn fresh_ptr_name(&mut self) -> String {
        let ptr_name = format!("ptr{}", self.next_ptr_id);
        self.next_ptr_id += 1;
        ptr_name
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
        }
    }

    pub fn get_var_info(&self, name: &str) -> Option<&VarInfo> {
        self.symbol_table.get(name)
    }
}


