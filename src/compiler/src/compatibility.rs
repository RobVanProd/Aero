// Compatibility layer for Phase 3 optimizations
// This module provides backward compatibility shims and fixes for compilation errors

use crate::ast::{BinaryOp, Expression, Statement, Type, UnaryOp};
use crate::types::Ty;

/// Compatibility extensions for Expression enum
impl Expression {
    /// Create a Number expression (backward compatibility)
    pub fn number(value: i64) -> Self {
        Expression::IntegerLiteral(value)
    }

    /// Create a Float expression (backward compatibility)
    pub fn float(value: f64) -> Self {
        Expression::FloatLiteral(value)
    }

    /// Get lhs from Binary expression (backward compatibility)
    pub fn get_lhs(&self) -> Option<&Expression> {
        match self {
            Expression::Binary { left, .. } => Some(left),
            _ => None,
        }
    }

    /// Get rhs from Binary expression (backward compatibility)
    pub fn get_rhs(&self) -> Option<&Expression> {
        match self {
            Expression::Binary { right, .. } => Some(right),
            _ => None,
        }
    }

    /// Check if expression is a Number variant (backward compatibility)
    pub fn is_number(&self) -> bool {
        matches!(self, Expression::IntegerLiteral(_))
    }

    /// Check if expression is a Float variant (backward compatibility)
    pub fn is_float(&self) -> bool {
        matches!(self, Expression::FloatLiteral(_))
    }

    /// Extract number value (backward compatibility)
    pub fn as_number(&self) -> Option<i64> {
        match self {
            Expression::IntegerLiteral(n) => Some(*n),
            _ => None,
        }
    }

    /// Extract float value (backward compatibility)
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Expression::FloatLiteral(f) => Some(*f),
            _ => None,
        }
    }
}

/// Compatibility extensions for Statement enum
impl Statement {
    /// Create a Let statement with simplified parameters (backward compatibility)
    pub fn let_simple(name: String, value: Expression) -> Self {
        Statement::Let {
            name,
            mutable: false,
            type_annotation: None,
            value: Some(value),
        }
    }

    /// Check if statement is a Let variant
    pub fn is_let(&self) -> bool {
        matches!(self, Statement::Let { .. })
    }

    /// Extract let statement components (backward compatibility)
    pub fn as_let(&self) -> Option<(&String, &Option<Expression>)> {
        match self {
            Statement::Let { name, value, .. } => Some((name, value)),
            _ => None,
        }
    }
}

/// Compatibility extensions for BinaryOp enum
impl BinaryOp {
    /// Create BinaryOp from string (backward compatibility)
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "+" => Some(BinaryOp::Add),
            "-" => Some(BinaryOp::Subtract),
            "*" => Some(BinaryOp::Multiply),
            "/" => Some(BinaryOp::Divide),
            "%" => Some(BinaryOp::Modulo),
            _ => None,
        }
    }
}

/// Compatibility extensions for UnaryOp enum
impl UnaryOp {
    /// Add Minus variant support (backward compatibility)
    pub fn minus() -> Self {
        UnaryOp::Negate
    }

    /// Check if UnaryOp is Minus/Negate
    pub fn is_minus(&self) -> bool {
        matches!(self, UnaryOp::Negate)
    }
}

/// Compatibility extensions for Type enum
impl Type {
    /// Get type name (backward compatibility)
    pub fn name(&self) -> &String {
        match self {
            Type::Named(name) => name,
        }
    }

    /// Create a new named type
    pub fn new(name: String) -> Self {
        Type::Named(name)
    }
}

/// Display implementation for BinaryOp (fixes compilation error)

/// Helper functions for type inference compatibility
pub fn infer_binary_type_compat(op: &BinaryOp, lhs: &Ty, rhs: &Ty) -> Result<Ty, String> {
    crate::types::infer_binary_type(op.as_str(), lhs, rhs)
}

/// Helper function to create Binary expression with all required fields
pub fn create_binary_expression(op: BinaryOp, left: Expression, right: Expression) -> Expression {
    Expression::Binary {
        op,
        left: Box::new(left),
        right: Box::new(right),
        ty: None,
    }
}

/// Helper function to create Let statement with all required fields
pub fn create_let_statement(
    name: String,
    value: Option<Expression>,
    mutable: bool,
    type_annotation: Option<Type>,
) -> Statement {
    Statement::Let {
        name,
        mutable,
        type_annotation,
        value,
    }
}

/// Helper function to unwrap Option<Expression> safely
pub fn unwrap_expression_option<'a>(
    expr_opt: &'a Option<Expression>,
    context: &str,
) -> Result<&'a Expression, String> {
    expr_opt
        .as_ref()
        .ok_or_else(|| format!("Expected expression in {}", context))
}

/// Helper function to unwrap Option<Expression> mutably
pub fn unwrap_expression_option_mut<'a>(
    expr_opt: &'a mut Option<Expression>,
    context: &str,
) -> Result<&'a mut Expression, String> {
    expr_opt
        .as_mut()
        .ok_or_else(|| format!("Expected expression in {}", context))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_compatibility() {
        let num_expr = Expression::number(42);
        assert!(num_expr.is_number());
        assert_eq!(num_expr.as_number(), Some(42));

        let float_expr = Expression::float(3.14);
        assert!(float_expr.is_float());
        assert_eq!(float_expr.as_float(), Some(3.14));
    }

    #[test]
    fn test_binary_op_string_conversion() {
        let add_op = BinaryOp::Add;
        assert_eq!(add_op.as_str(), "+");
        assert_eq!(format!("{}", add_op), "+");

        let mul_op = BinaryOp::from_str("*").unwrap();
        assert_eq!(mul_op.as_str(), "*");
    }

    #[test]
    fn test_statement_compatibility() {
        let let_stmt = Statement::let_simple("x".to_string(), Expression::number(5));
        assert!(let_stmt.is_let());

        if let Some((name, value)) = let_stmt.as_let() {
            assert_eq!(name, "x");
            assert!(value.is_some());
        }
    }

    #[test]
    fn test_type_compatibility() {
        let int_type = Type::new("i32".to_string());
        assert_eq!(int_type.name(), "i32");
    }

    #[test]
    fn test_binary_expression_creation() {
        let left = Expression::number(5);
        let right = Expression::number(3);
        let binary = create_binary_expression(BinaryOp::Add, left, right);

        match binary {
            Expression::Binary {
                op,
                left,
                right,
                ty,
            } => {
                assert_eq!(op.as_str(), "+");
                assert!(left.is_number());
                assert!(right.is_number());
                assert!(ty.is_none());
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_unary_op_compatibility() {
        let minus_op = UnaryOp::minus();
        assert!(minus_op.is_minus());
    }
}
