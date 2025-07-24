use crate::parser::Parser;
use crate::lexer::{Token, LocatedToken};
use crate::errors::SourceLocation;
use crate::ast::{Statement, Expression, Pattern, MatchArm, EnumVariant, EnumVariantData, Type, StructField, Visibility};

fn create_token(token: Token) -> LocatedToken {
    LocatedToken {
        token,
        location: SourceLocation {
            line: 1,
            column: 1,
            filename: "test".to_string(),
        },
    }
}

fn create_tokens(tokens: Vec<Token>) -> Vec<LocatedToken> {
    tokens.into_iter().map(create_token).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_enum() {
        let tokens = create_tokens(vec![
            Token::Enum,
            Token::Identifier("Color".to_string()),
            Token::LeftBrace,
            Token::Identifier("Red".to_string()),
            Token::Comma,
            Token::Identifier("Green".to_string()),
            Token::Comma,
            Token::Identifier("Blue".to_string()),
            Token::RightBrace,
            Token::Eof,
        ]);

        let mut parser = Parser::new(tokens);
        let result = parser.parse().unwrap();

        assert_eq!(result.len(), 1);
        match &result[0] {
            crate::ast::AstNode::Statement(Statement::Enum { name, generics, variants }) => {
                assert_eq!(name, "Color");
                assert_eq!(generics.len(), 0);
                assert_eq!(variants.len(), 3);
                assert_eq!(variants[0].name, "Red");
                assert_eq!(variants[1].name, "Green");
                assert_eq!(variants[2].name, "Blue");
                assert!(variants[0].data.is_none());
                assert!(variants[1].data.is_none());
                assert!(variants[2].data.is_none());
            }
            _ => panic!("Expected enum statement"),
        }
    }

    #[test]
    fn test_parse_generic_enum() {
        let tokens = create_tokens(vec![
            Token::Enum,
            Token::Identifier("Option".to_string()),
            Token::LeftAngle,
            Token::Identifier("T".to_string()),
            Token::RightAngle,
            Token::LeftBrace,
            Token::Identifier("Some".to_string()),
            Token::LeftParen,
            Token::Identifier("T".to_string()),
            Token::RightParen,
            Token::Comma,
            Token::Identifier("None".to_string()),
            Token::RightBrace,
            Token::Eof,
        ]);

        let mut parser = Parser::new(tokens);
        let result = parser.parse().unwrap();

        assert_eq!(result.len(), 1);
        match &result[0] {
            crate::ast::AstNode::Statement(Statement::Enum { name, generics, variants }) => {
                assert_eq!(name, "Option");
                assert_eq!(generics.len(), 1);
                assert_eq!(generics[0], "T");
                assert_eq!(variants.len(), 2);
                
                // Check Some variant
                assert_eq!(variants[0].name, "Some");
                match &variants[0].data {
                    Some(EnumVariantData::Tuple(types)) => {
                        assert_eq!(types.len(), 1);
                        assert_eq!(types[0], Type::Named("T".to_string()));
                    }
                    _ => panic!("Expected tuple variant data"),
                }
                
                // Check None variant
                assert_eq!(variants[1].name, "None");
                assert!(variants[1].data.is_none());
            }
            _ => panic!("Expected enum statement"),
        }
    }

    #[test]
    fn test_parse_enum_with_struct_variant() {
        let tokens = create_tokens(vec![
            Token::Enum,
            Token::Identifier("Shape".to_string()),
            Token::LeftBrace,
            Token::Identifier("Circle".to_string()),
            Token::LeftBrace,
            Token::Identifier("radius".to_string()),
            Token::Colon,
            Token::Identifier("f64".to_string()),
            Token::RightBrace,
            Token::Comma,
            Token::Identifier("Rectangle".to_string()),
            Token::LeftBrace,
            Token::Identifier("width".to_string()),
            Token::Colon,
            Token::Identifier("f64".to_string()),
            Token::Comma,
            Token::Identifier("height".to_string()),
            Token::Colon,
            Token::Identifier("f64".to_string()),
            Token::RightBrace,
            Token::RightBrace,
            Token::Eof,
        ]);

        let mut parser = Parser::new(tokens);
        let result = parser.parse().unwrap();

        assert_eq!(result.len(), 1);
        match &result[0] {
            crate::ast::AstNode::Statement(Statement::Enum { name, generics, variants }) => {
                assert_eq!(name, "Shape");
                assert_eq!(generics.len(), 0);
                assert_eq!(variants.len(), 2);
                
                // Check Circle variant
                assert_eq!(variants[0].name, "Circle");
                match &variants[0].data {
                    Some(EnumVariantData::Struct(fields)) => {
                        assert_eq!(fields.len(), 1);
                        assert_eq!(fields[0].name, "radius");
                        assert_eq!(fields[0].field_type, Type::Named("f64".to_string()));
                    }
                    _ => panic!("Expected struct variant data"),
                }
                
                // Check Rectangle variant
                assert_eq!(variants[1].name, "Rectangle");
                match &variants[1].data {
                    Some(EnumVariantData::Struct(fields)) => {
                        assert_eq!(fields.len(), 2);
                        assert_eq!(fields[0].name, "width");
                        assert_eq!(fields[1].name, "height");
                    }
                    _ => panic!("Expected struct variant data"),
                }
            }
            _ => panic!("Expected enum statement"),
        }
    }

    #[test]
    fn test_parse_simple_match() {
        let tokens = create_tokens(vec![
            Token::Match,
            Token::Identifier("color".to_string()),
            Token::LeftBrace,
            Token::Identifier("Red".to_string()),
            Token::FatArrow,
            Token::IntegerLiteral(1),
            Token::Comma,
            Token::Identifier("Green".to_string()),
            Token::FatArrow,
            Token::IntegerLiteral(2),
            Token::Comma,
            Token::Underscore,
            Token::FatArrow,
            Token::IntegerLiteral(0),
            Token::RightBrace,
            Token::Semicolon,
            Token::Eof,
        ]);

        let mut parser = Parser::new(tokens);
        let result = parser.parse().unwrap();

        assert_eq!(result.len(), 1);
        match &result[0] {
            crate::ast::AstNode::Statement(Statement::Expression(Expression::Match { expression, arms })) => {
                assert!(matches!(**expression, Expression::Identifier(_)));
                assert_eq!(arms.len(), 3);
                
                // Check first arm
                assert!(matches!(arms[0].pattern, Pattern::Identifier(_)));
                assert!(arms[0].guard.is_none());
                assert!(matches!(arms[0].body, Expression::IntegerLiteral(1)));
                
                // Check wildcard arm
                assert!(matches!(arms[2].pattern, Pattern::Wildcard));
                assert!(matches!(arms[2].body, Expression::IntegerLiteral(0)));
            }
            _ => panic!("Expected match expression statement"),
        }
    }

    #[test]
    fn test_parse_match_with_enum_patterns() {
        let tokens = create_tokens(vec![
            Token::Match,
            Token::Identifier("option".to_string()),
            Token::LeftBrace,
            Token::Identifier("Some".to_string()),
            Token::LeftParen,
            Token::Identifier("x".to_string()),
            Token::RightParen,
            Token::FatArrow,
            Token::Identifier("x".to_string()),
            Token::Comma,
            Token::Identifier("None".to_string()),
            Token::FatArrow,
            Token::IntegerLiteral(0),
            Token::RightBrace,
            Token::Semicolon,
            Token::Eof,
        ]);

        let mut parser = Parser::new(tokens);
        let result = parser.parse().unwrap();

        assert_eq!(result.len(), 1);
        match &result[0] {
            crate::ast::AstNode::Statement(Statement::Expression(Expression::Match { expression, arms })) => {
                assert!(matches!(**expression, Expression::Identifier(_)));
                assert_eq!(arms.len(), 2);
                
                // Check Some(x) pattern
                match &arms[0].pattern {
                    Pattern::Enum { variant, data } => {
                        assert_eq!(variant, "Some");
                        assert!(data.is_some());
                        match data.as_ref().unwrap().as_ref() {
                            Pattern::Identifier(name) => assert_eq!(name, "x"),
                            _ => panic!("Expected identifier pattern in Some"),
                        }
                    }
                    _ => panic!("Expected enum pattern"),
                }
                
                // Check None pattern
                match &arms[1].pattern {
                    Pattern::Identifier(name) => assert_eq!(name, "None"),
                    _ => panic!("Expected identifier pattern for None"),
                }
            }
            _ => panic!("Expected match expression statement"),
        }
    }

    #[test]
    fn test_parse_match_with_guard() {
        let tokens = create_tokens(vec![
            Token::Match,
            Token::Identifier("x".to_string()),
            Token::LeftBrace,
            Token::Identifier("n".to_string()),
            Token::Identifier("if".to_string()),
            Token::Identifier("n".to_string()),
            Token::GreaterThan,
            Token::IntegerLiteral(0),
            Token::FatArrow,
            Token::Identifier("n".to_string()),
            Token::Comma,
            Token::Underscore,
            Token::FatArrow,
            Token::IntegerLiteral(0),
            Token::RightBrace,
            Token::Semicolon,
            Token::Eof,
        ]);

        let mut parser = Parser::new(tokens);
        let result = parser.parse().unwrap();

        assert_eq!(result.len(), 1);
        match &result[0] {
            crate::ast::AstNode::Statement(Statement::Expression(Expression::Match { expression, arms })) => {
                assert!(matches!(**expression, Expression::Identifier(_)));
                assert_eq!(arms.len(), 2);
                
                // Check first arm with guard
                assert!(matches!(arms[0].pattern, Pattern::Identifier(_)));
                assert!(arms[0].guard.is_some());
                match &arms[0].guard {
                    Some(Expression::Comparison { op, .. }) => {
                        assert!(matches!(op, crate::ast::ComparisonOp::GreaterThan));
                    }
                    _ => panic!("Expected comparison guard"),
                }
                
                // Check wildcard arm without guard
                assert!(matches!(arms[1].pattern, Pattern::Wildcard));
                assert!(arms[1].guard.is_none());
            }
            _ => panic!("Expected match expression statement"),
        }
    }

    #[test]
    fn test_parse_struct_pattern() {
        let tokens = create_tokens(vec![
            Token::Match,
            Token::Identifier("point".to_string()),
            Token::LeftBrace,
            Token::Identifier("Point".to_string()),
            Token::LeftBrace,
            Token::Identifier("x".to_string()),
            Token::Comma,
            Token::Identifier("y".to_string()),
            Token::Colon,
            Token::Identifier("py".to_string()),
            Token::RightBrace,
            Token::FatArrow,
            Token::Identifier("x".to_string()),
            Token::RightBrace,
            Token::Semicolon,
            Token::Eof,
        ]);

        let mut parser = Parser::new(tokens);
        let result = parser.parse().unwrap();

        assert_eq!(result.len(), 1);
        match &result[0] {
            crate::ast::AstNode::Statement(Statement::Expression(Expression::Match { expression, arms })) => {
                assert!(matches!(**expression, Expression::Identifier(_)));
                assert_eq!(arms.len(), 1);
                
                // Check struct pattern
                match &arms[0].pattern {
                    Pattern::Struct { name, fields, rest } => {
                        assert_eq!(name, "Point");
                        assert_eq!(fields.len(), 2);
                        assert_eq!(fields[0].0, "x");
                        assert!(matches!(fields[0].1, Pattern::Identifier(_)));
                        assert_eq!(fields[1].0, "y");
                        assert!(matches!(fields[1].1, Pattern::Identifier(_)));
                        assert!(!rest);
                    }
                    _ => panic!("Expected struct pattern"),
                }
            }
            _ => panic!("Expected match expression statement"),
        }
    }

    #[test]
    fn test_parse_range_pattern() {
        let tokens = create_tokens(vec![
            Token::Match,
            Token::Identifier("x".to_string()),
            Token::LeftBrace,
            Token::IntegerLiteral(1),
            Token::DotDotEqual,
            Token::IntegerLiteral(10),
            Token::FatArrow,
            Token::Identifier("x".to_string()),
            Token::Comma,
            Token::Underscore,
            Token::FatArrow,
            Token::IntegerLiteral(0),
            Token::RightBrace,
            Token::Semicolon,
            Token::Eof,
        ]);

        let mut parser = Parser::new(tokens);
        let result = parser.parse().unwrap();

        assert_eq!(result.len(), 1);
        match &result[0] {
            crate::ast::AstNode::Statement(Statement::Expression(Expression::Match { expression, arms })) => {
                assert!(matches!(**expression, Expression::Identifier(_)));
                assert_eq!(arms.len(), 2);
                
                // Check range pattern
                match &arms[0].pattern {
                    Pattern::Range { start, end, inclusive } => {
                        assert!(matches!(start.as_ref(), Pattern::Literal(_)));
                        assert!(matches!(end.as_ref(), Pattern::Literal(_)));
                        assert!(*inclusive);
                    }
                    _ => panic!("Expected range pattern"),
                }
            }
            _ => panic!("Expected match expression statement"),
        }
    }

    #[test]
    fn test_parse_or_pattern() {
        let tokens = create_tokens(vec![
            Token::Match,
            Token::Identifier("color".to_string()),
            Token::LeftBrace,
            Token::Identifier("Red".to_string()),
            Token::Pipe,
            Token::Identifier("Green".to_string()),
            Token::Pipe,
            Token::Identifier("Blue".to_string()),
            Token::FatArrow,
            Token::IntegerLiteral(1),
            Token::RightBrace,
            Token::Semicolon,
            Token::Eof,
        ]);

        let mut parser = Parser::new(tokens);
        let result = parser.parse().unwrap();

        assert_eq!(result.len(), 1);
        match &result[0] {
            crate::ast::AstNode::Statement(Statement::Expression(Expression::Match { expression, arms })) => {
                assert!(matches!(**expression, Expression::Identifier(_)));
                assert_eq!(arms.len(), 1);
                
                // Check or pattern
                match &arms[0].pattern {
                    Pattern::Or(patterns) => {
                        assert_eq!(patterns.len(), 3);
                        for p in patterns {
                            assert!(matches!(p, Pattern::Identifier(_)));
                        }
                    }
                    _ => panic!("Expected or pattern"),
                }
            }
            _ => panic!("Expected match expression statement"),
        }
    }

    #[test]
    fn test_parse_binding_pattern() {
        let tokens = create_tokens(vec![
            Token::Match,
            Token::Identifier("x".to_string()),
            Token::LeftBrace,
            Token::Identifier("value".to_string()),
            Token::At,
            Token::IntegerLiteral(1),
            Token::DotDotEqual,
            Token::IntegerLiteral(10),
            Token::FatArrow,
            Token::Identifier("value".to_string()),
            Token::RightBrace,
            Token::Semicolon,
            Token::Eof,
        ]);

        let mut parser = Parser::new(tokens);
        let result = parser.parse().unwrap();

        assert_eq!(result.len(), 1);
        match &result[0] {
            crate::ast::AstNode::Statement(Statement::Expression(Expression::Match { expression, arms })) => {
                assert!(matches!(**expression, Expression::Identifier(_)));
                assert_eq!(arms.len(), 1);
                
                // Check binding pattern
                match &arms[0].pattern {
                    Pattern::Binding { name, pattern } => {
                        assert_eq!(name, "value");
                        assert!(matches!(pattern.as_ref(), Pattern::Range { .. }));
                    }
                    _ => panic!("Expected binding pattern"),
                }
            }
            _ => panic!("Expected match expression statement"),
        }
    }

    #[test]
    fn test_parse_tuple_pattern() {
        let tokens = create_tokens(vec![
            Token::Match,
            Token::Identifier("tuple".to_string()),
            Token::LeftBrace,
            Token::LeftParen,
            Token::Identifier("x".to_string()),
            Token::Comma,
            Token::Identifier("y".to_string()),
            Token::Comma,
            Token::Underscore,
            Token::RightParen,
            Token::FatArrow,
            Token::Identifier("x".to_string()),
            Token::RightBrace,
            Token::Semicolon,
            Token::Eof,
        ]);

        let mut parser = Parser::new(tokens);
        let result = parser.parse().unwrap();

        assert_eq!(result.len(), 1);
        match &result[0] {
            crate::ast::AstNode::Statement(Statement::Expression(Expression::Match { expression, arms })) => {
                assert!(matches!(**expression, Expression::Identifier(_)));
                assert_eq!(arms.len(), 1);
                
                // Check tuple pattern
                match &arms[0].pattern {
                    Pattern::Tuple(patterns) => {
                        assert_eq!(patterns.len(), 3);
                        assert!(matches!(patterns[0], Pattern::Identifier(_)));
                        assert!(matches!(patterns[1], Pattern::Identifier(_)));
                        assert!(matches!(patterns[2], Pattern::Wildcard));
                    }
                    _ => panic!("Expected tuple pattern"),
                }
            }
            _ => panic!("Expected match expression statement"),
        }
    }
}