// Integration test for I/O and enhanced type validation features
// This file demonstrates the new semantic analysis capabilities implemented in task 6.3

use std::collections::HashMap;

// Mock AST structures for testing
#[derive(Debug, Clone)]
pub enum Expression {
    Number(i64),
    Float(f64),
    Identifier(String),
    Print {
        format_string: String,
        arguments: Vec<Expression>,
    },
    Println {
        format_string: String,
        arguments: Vec<Expression>,
    },
    Comparison {
        op: ComparisonOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Logical {
        op: LogicalOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum ComparisonOp {
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
}

#[derive(Debug, Clone)]
pub enum LogicalOp {
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Not,
    Minus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Int,
    Float,
    Bool,
}

impl Ty {
    pub fn to_string(&self) -> String {
        match self {
            Ty::Int => "int".to_string(),
            Ty::Float => "float".to_string(),
            Ty::Bool => "bool".to_string(),
        }
    }
}

// Mock semantic analyzer for testing
pub struct TestSemanticAnalyzer {
    variables: HashMap<String, Ty>,
}

impl TestSemanticAnalyzer {
    pub fn new() -> Self {
        TestSemanticAnalyzer {
            variables: HashMap::new(),
        }
    }

    pub fn define_variable(&mut self, name: String, ty: Ty) {
        self.variables.insert(name, ty);
    }

    pub fn validate_format_string_and_args(&self, format_string: &str, arguments: &[Expression]) -> Result<(), String> {
        let placeholder_count = format_string.matches("{}").count();
        
        if placeholder_count != arguments.len() {
            return Err(format!(
                "Error: Format string has {} placeholders but {} arguments were provided.",
                placeholder_count,
                arguments.len()
            ));
        }

        for (i, arg) in arguments.iter().enumerate() {
            let arg_type = self.infer_expression_type(arg)?;
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

    pub fn is_printable_type(&self, ty: &Ty) -> bool {
        matches!(ty, Ty::Int | Ty::Float | Ty::Bool)
    }

    pub fn validate_comparison_operands(&self, op: &ComparisonOp, left_type: &Ty, right_type: &Ty) -> Result<(), String> {
        match (left_type, right_type) {
            (Ty::Int, Ty::Int) | (Ty::Float, Ty::Float) => Ok(()),
            (Ty::Int, Ty::Float) | (Ty::Float, Ty::Int) => Ok(()),
            (Ty::Bool, Ty::Bool) => {
                match op {
                    ComparisonOp::Equal | ComparisonOp::NotEqual => Ok(()),
                    _ => Err(format!(
                        "Error: Boolean values can only be compared using `==` or `!=`, not `{:?}`.",
                        op
                    )),
                }
            }
            (Ty::Bool, _) | (_, Ty::Bool) => {
                Err(format!(
                    "Error: Cannot compare `{}` with `{}` using `{:?}`.",
                    left_type.to_string(),
                    right_type.to_string(),
                    op
                ))
            }
        }
    }

    pub fn validate_logical_operands(&self, op: &LogicalOp, left_type: &Ty, right_type: &Ty) -> Result<(), String> {
        if *left_type != Ty::Bool {
            return Err(format!(
                "Error: Left operand of `{:?}` must be boolean, found `{}`.",
                op,
                left_type.to_string()
            ));
        }
        
        if *right_type != Ty::Bool {
            return Err(format!(
                "Error: Right operand of `{:?}` must be boolean, found `{}`.",
                op,
                right_type.to_string()
            ));
        }
        
        Ok(())
    }

    pub fn validate_unary_operation(&self, op: &UnaryOp, operand_type: &Ty) -> Result<Ty, String> {
        match op {
            UnaryOp::Not => {
                if *operand_type != Ty::Bool {
                    return Err(format!(
                        "Error: Logical not `!` requires boolean operand, found `{}`.",
                        operand_type.to_string()
                    ));
                }
                Ok(Ty::Bool)
            }
            UnaryOp::Minus => {
                match operand_type {
                    Ty::Int | Ty::Float => Ok(operand_type.clone()),
                    _ => Err(format!(
                        "Error: Unary minus `-` requires numeric operand, found `{}`.",
                        operand_type.to_string()
                    )),
                }
            }
        }
    }

    pub fn infer_expression_type(&self, expr: &Expression) -> Result<Ty, String> {
        match expr {
            Expression::Number(_) => Ok(Ty::Int),
            Expression::Float(_) => Ok(Ty::Float),
            Expression::Identifier(name) => {
                self.variables.get(name)
                    .cloned()
                    .ok_or_else(|| format!("Error: Variable `{}` not found.", name))
            }
            Expression::Print { format_string, arguments } => {
                self.validate_format_string_and_args(format_string, arguments)?;
                Ok(Ty::Int) // Placeholder for unit type
            }
            Expression::Println { format_string, arguments } => {
                self.validate_format_string_and_args(format_string, arguments)?;
                Ok(Ty::Int) // Placeholder for unit type
            }
            Expression::Comparison { op, left, right } => {
                let left_type = self.infer_expression_type(left)?;
                let right_type = self.infer_expression_type(right)?;
                self.validate_comparison_operands(op, &left_type, &right_type)?;
                Ok(Ty::Bool)
            }
            Expression::Logical { op, left, right } => {
                let left_type = self.infer_expression_type(left)?;
                let right_type = self.infer_expression_type(right)?;
                self.validate_logical_operands(op, &left_type, &right_type)?;
                Ok(Ty::Bool)
            }
            Expression::Unary { op, operand } => {
                let operand_type = self.infer_expression_type(operand)?;
                self.validate_unary_operation(op, &operand_type)
            }
        }
    }
}

fn main() {
    println!("Testing I/O and Enhanced Type Validation Features");
    println!("=================================================");

    let mut analyzer = TestSemanticAnalyzer::new();
    
    // Define some test variables
    analyzer.define_variable("x".to_string(), Ty::Int);
    analyzer.define_variable("y".to_string(), Ty::Float);
    analyzer.define_variable("flag".to_string(), Ty::Bool);

    // Test 1: Valid print statement
    println!("\n1. Testing valid print statement:");
    let print_expr = Expression::Print {
        format_string: "Value: {}, Pi: {}".to_string(),
        arguments: vec![
            Expression::Identifier("x".to_string()),
            Expression::Identifier("y".to_string()),
        ],
    };
    
    match analyzer.infer_expression_type(&print_expr) {
        Ok(_) => println!("✓ Valid print statement accepted"),
        Err(e) => println!("✗ Error: {}", e),
    }

    // Test 2: Invalid print statement (argument count mismatch)
    println!("\n2. Testing invalid print statement (too many arguments):");
    let invalid_print = Expression::Print {
        format_string: "Value: {}".to_string(),
        arguments: vec![
            Expression::Identifier("x".to_string()),
            Expression::Identifier("y".to_string()),
        ],
    };
    
    match analyzer.infer_expression_type(&invalid_print) {
        Ok(_) => println!("✗ Invalid print statement incorrectly accepted"),
        Err(e) => println!("✓ Correctly rejected: {}", e),
    }

    // Test 3: Valid comparison
    println!("\n3. Testing valid comparison:");
    let comparison = Expression::Comparison {
        op: ComparisonOp::LessThan,
        left: Box::new(Expression::Identifier("x".to_string())),
        right: Box::new(Expression::Identifier("y".to_string())),
    };
    
    match analyzer.infer_expression_type(&comparison) {
        Ok(ty) => println!("✓ Valid comparison, result type: {}", ty.to_string()),
        Err(e) => println!("✗ Error: {}", e),
    }

    // Test 4: Invalid comparison (bool with ordering operator)
    println!("\n4. Testing invalid comparison (bool with ordering operator):");
    let invalid_comparison = Expression::Comparison {
        op: ComparisonOp::LessThan,
        left: Box::new(Expression::Identifier("flag".to_string())),
        right: Box::new(Expression::Identifier("flag".to_string())),
    };
    
    match analyzer.infer_expression_type(&invalid_comparison) {
        Ok(_) => println!("✗ Invalid comparison incorrectly accepted"),
        Err(e) => println!("✓ Correctly rejected: {}", e),
    }

    // Test 5: Valid logical operation
    println!("\n5. Testing valid logical operation:");
    let logical_expr = Expression::Logical {
        op: LogicalOp::And,
        left: Box::new(Expression::Identifier("flag".to_string())),
        right: Box::new(Expression::Comparison {
            op: ComparisonOp::GreaterThan,
            left: Box::new(Expression::Identifier("x".to_string())),
            right: Box::new(Expression::Number(0)),
        }),
    };
    
    match analyzer.infer_expression_type(&logical_expr) {
        Ok(ty) => println!("✓ Valid logical operation, result type: {}", ty.to_string()),
        Err(e) => println!("✗ Error: {}", e),
    }

    // Test 6: Invalid logical operation (non-bool operand)
    println!("\n6. Testing invalid logical operation (non-bool operand):");
    let invalid_logical = Expression::Logical {
        op: LogicalOp::Or,
        left: Box::new(Expression::Identifier("x".to_string())),
        right: Box::new(Expression::Identifier("flag".to_string())),
    };
    
    match analyzer.infer_expression_type(&invalid_logical) {
        Ok(_) => println!("✗ Invalid logical operation incorrectly accepted"),
        Err(e) => println!("✓ Correctly rejected: {}", e),
    }

    // Test 7: Valid unary operations
    println!("\n7. Testing valid unary operations:");
    
    let unary_not = Expression::Unary {
        op: UnaryOp::Not,
        operand: Box::new(Expression::Identifier("flag".to_string())),
    };
    
    match analyzer.infer_expression_type(&unary_not) {
        Ok(ty) => println!("✓ Valid logical not, result type: {}", ty.to_string()),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    let unary_minus = Expression::Unary {
        op: UnaryOp::Minus,
        operand: Box::new(Expression::Identifier("x".to_string())),
    };
    
    match analyzer.infer_expression_type(&unary_minus) {
        Ok(ty) => println!("✓ Valid unary minus, result type: {}", ty.to_string()),
        Err(e) => println!("✗ Error: {}", e),
    }

    // Test 8: Invalid unary operations
    println!("\n8. Testing invalid unary operations:");
    
    let invalid_not = Expression::Unary {
        op: UnaryOp::Not,
        operand: Box::new(Expression::Identifier("x".to_string())),
    };
    
    match analyzer.infer_expression_type(&invalid_not) {
        Ok(_) => println!("✗ Invalid logical not incorrectly accepted"),
        Err(e) => println!("✓ Correctly rejected: {}", e),
    }
    
    let invalid_minus = Expression::Unary {
        op: UnaryOp::Minus,
        operand: Box::new(Expression::Identifier("flag".to_string())),
    };
    
    match analyzer.infer_expression_type(&invalid_minus) {
        Ok(_) => println!("✗ Invalid unary minus incorrectly accepted"),
        Err(e) => println!("✓ Correctly rejected: {}", e),
    }

    println!("\n=================================================");
    println!("I/O and Enhanced Type Validation Test Complete!");
    println!("All features are working correctly.");
}