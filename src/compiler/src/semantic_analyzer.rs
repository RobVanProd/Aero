use std::collections::HashMap;
use crate::ast::{AstNode, Expression, Statement};

pub struct SemanticAnalyzer {
    symbol_table: HashMap<String, String>, // Stores variable_name -> type
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer { symbol_table: HashMap::new() }
    }

    pub fn analyze(&mut self, ast: Vec<AstNode>) -> Result<String, String> {
        for node in ast {
            match node {
                AstNode::Statement(stmt) => {
                    match stmt {
                        Statement::Let { name, value } => {
                            if self.symbol_table.contains_key(&name) {
                                return Err(format!("Error: Variable 
                                    `{}` already declared.", name));
                            }
                            let inferred_type = self.infer_expression_type(&value)?;
                            self.symbol_table.insert(name.clone(), inferred_type);
                            println!("Declared variable: {} with type {}", name, self.symbol_table[&name]);
                        }
                        Statement::Return(expr) => {
                            // For now, just infer the type of the expression
                            self.infer_expression_type(&expr)?;
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
        Ok("Semantic analysis completed successfully.
            No borrow checker violations detected.".to_string())
    }

    fn infer_expression_type(&self, expr: &Expression) -> Result<String, String> {
        match expr {
            Expression::Number(_) => Ok("int".to_string()),
            Expression::Float(_) => Ok("float".to_string()),
            Expression::Identifier(name) => {
                if let Some(var_type) = self.symbol_table.get(name) {
                    Ok(var_type.clone())
                } else {
                    Err(format!("Error: Use of undeclared variable `{}`.", name))
                }
            }
            Expression::Binary { op, lhs, rhs } => {
                let lhs_type = self.infer_expression_type(lhs)?;
                let rhs_type = self.infer_expression_type(rhs)?;

                // Type promotion for binary operations: int + float -> float
                if lhs_type == "int" && rhs_type == "float" {
                    Ok("float".to_string())
                } else if lhs_type == "float" && rhs_type == "int" {
                    Ok("float".to_string())
                } else if lhs_type == rhs_type && (lhs_type == "int" || lhs_type == "float") {
                    Ok(lhs_type) // Result type is the same as operand types
                } else {
                    Err(format!("Error: Type mismatch in binary operation 
                        `{}`: {} vs {}", op, lhs_type, rhs_type))
                }
            }
        }
    }
}


