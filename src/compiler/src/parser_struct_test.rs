#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{tokenize_with_locations, Token, LocatedToken};
    use crate::ast::{AstNode, Statement, Expression, Type, StructField, Visibility};
    use crate::errors::SourceLocation;

    // Helper function to create a parser from source code
    fn create_parser(source: &str) -> Parser {
        let tokens = tokenize_with_locations(source, None);
        Parser::new(tokens)
    }

    // ===== STRUCT DEFINITION PARSING TESTS =====

    #[test]
    fn test_simple_struct_definition() {
        let source = "struct Point { x: i32, y: i32 }";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        assert_eq!(ast.len(), 1);
        match &ast[0] {
            AstNode::Statement(Statement::Struct { name, generics, fields, is_tuple }) => {
                assert_eq!(name, "Point");
                assert_eq!(generics.len(), 0);
                assert_eq!(fields.len(), 2);
                assert!(!is_tuple);
                
                // Check first field
                assert_eq!(fields[0].name, "x");
                assert_eq!(fields[0].field_type, Type::Named("i32".to_string()));
                assert!(matches!(fields[0].visibility, Visibility::Private));
                
                // Check second field
                assert_eq!(fields[1].name, "y");
                assert_eq!(fields[1].field_type, Type::Named("i32".to_string()));
                assert!(matches!(fields[1].visibility, Visibility::Private));
            }
            _ => panic!("Expected struct statement"),
        }
    }

    #[test]
    fn test_struct_with_public_fields() {
        let source = "struct Point { pub x: i32, pub y: i32 }";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Struct { fields, .. }) => {
                assert_eq!(fields.len(), 2);
                assert!(matches!(fields[0].visibility, Visibility::Public));
                assert!(matches!(fields[1].visibility, Visibility::Public));
            }
            _ => panic!("Expected struct statement"),
        }
    }

    #[test]
    fn test_struct_with_mixed_visibility() {
        let source = "struct Person { pub name: String, age: i32, pub email: String }";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Struct { fields, .. }) => {
                assert_eq!(fields.len(), 3);
                assert!(matches!(fields[0].visibility, Visibility::Public)); // name
                assert!(matches!(fields[1].visibility, Visibility::Private)); // age
                assert!(matches!(fields[2].visibility, Visibility::Public)); // email
            }
            _ => panic!("Expected struct statement"),
        }
    }

    #[test]
    fn test_struct_with_trailing_comma() {
        let source = "struct Point { x: i32, y: i32, }";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Struct { fields, .. }) => {
                assert_eq!(fields.len(), 2);
            }
            _ => panic!("Expected struct statement"),
        }
    }

    #[test]
    fn test_empty_struct() {
        let source = "struct Empty { }";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Struct { name, fields, .. }) => {
                assert_eq!(name, "Empty");
                assert_eq!(fields.len(), 0);
            }
            _ => panic!("Expected struct statement"),
        }
    }

    #[test]
    fn test_generic_struct_single_parameter() {
        let source = "struct Container<T> { value: T }";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Struct { name, generics, fields, .. }) => {
                assert_eq!(name, "Container");
                assert_eq!(generics.len(), 1);
                assert_eq!(generics[0], "T");
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].field_type, Type::Named("T".to_string()));
            }
            _ => panic!("Expected struct statement"),
        }
    }

    #[test]
    fn test_generic_struct_multiple_parameters() {
        let source = "struct Pair<T, U> { first: T, second: U }";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Struct { name, generics, fields, .. }) => {
                assert_eq!(name, "Pair");
                assert_eq!(generics.len(), 2);
                assert_eq!(generics[0], "T");
                assert_eq!(generics[1], "U");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].field_type, Type::Named("T".to_string()));
                assert_eq!(fields[1].field_type, Type::Named("U".to_string()));
            }
            _ => panic!("Expected struct statement"),
        }
    }

    #[test]
    fn test_struct_with_generic_field_types() {
        let source = "struct Container { items: Vec<i32>, map: HashMap<String, i32> }";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Struct { fields, .. }) => {
                assert_eq!(fields.len(), 2);
                
                // Check Vec<i32> field
                match &fields[0].field_type {
                    Type::Generic { name, type_args } => {
                        assert_eq!(name, "Vec");
                        assert_eq!(type_args.len(), 1);
                        assert_eq!(type_args[0], Type::Named("i32".to_string()));
                    }
                    _ => panic!("Expected generic type for Vec field"),
                }
                
                // Check HashMap<String, i32> field
                match &fields[1].field_type {
                    Type::Generic { name, type_args } => {
                        assert_eq!(name, "HashMap");
                        assert_eq!(type_args.len(), 2);
                        assert_eq!(type_args[0], Type::Named("String".to_string()));
                        assert_eq!(type_args[1], Type::Named("i32".to_string()));
                    }
                    _ => panic!("Expected generic type for HashMap field"),
                }
            }
            _ => panic!("Expected struct statement"),
        }
    }

    // ===== TUPLE STRUCT PARSING TESTS =====

    #[test]
    fn test_simple_tuple_struct() {
        let source = "struct Point(i32, i32);";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Struct { name, generics, fields, is_tuple }) => {
                assert_eq!(name, "Point");
                assert_eq!(generics.len(), 0);
                assert_eq!(fields.len(), 2);
                assert!(*is_tuple);
                
                // Check fields (indexed by position)
                assert_eq!(fields[0].name, "0");
                assert_eq!(fields[0].field_type, Type::Named("i32".to_string()));
                assert!(matches!(fields[0].visibility, Visibility::Public));
                
                assert_eq!(fields[1].name, "1");
                assert_eq!(fields[1].field_type, Type::Named("i32".to_string()));
                assert!(matches!(fields[1].visibility, Visibility::Public));
            }
            _ => panic!("Expected struct statement"),
        }
    }

    #[test]
    fn test_tuple_struct_with_public_fields() {
        let source = "struct Color(pub u8, pub u8, pub u8);";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Struct { fields, is_tuple, .. }) => {
                assert!(*is_tuple);
                assert_eq!(fields.len(), 3);
                for field in fields {
                    assert!(matches!(field.visibility, Visibility::Public));
                }
            }
            _ => panic!("Expected struct statement"),
        }
    }

    #[test]
    fn test_empty_tuple_struct() {
        let source = "struct Unit();";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Struct { name, fields, is_tuple, .. }) => {
                assert_eq!(name, "Unit");
                assert_eq!(fields.len(), 0);
                assert!(*is_tuple);
            }
            _ => panic!("Expected struct statement"),
        }
    }

    #[test]
    fn test_generic_tuple_struct() {
        let source = "struct Wrapper<T>(T);";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Struct { name, generics, fields, is_tuple }) => {
                assert_eq!(name, "Wrapper");
                assert_eq!(generics.len(), 1);
                assert_eq!(generics[0], "T");
                assert_eq!(fields.len(), 1);
                assert!(*is_tuple);
                assert_eq!(fields[0].field_type, Type::Named("T".to_string()));
            }
            _ => panic!("Expected struct statement"),
        }
    }

    // ===== STRUCT LITERAL PARSING TESTS =====

    #[test]
    fn test_simple_struct_literal() {
        let source = "let p = Point { x: 10, y: 20 };";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Let { value, .. }) => {
                match value.as_ref().unwrap() {
                    Expression::StructLiteral { name, fields, base } => {
                        assert_eq!(name, "Point");
                        assert_eq!(fields.len(), 2);
                        assert!(base.is_none());
                        
                        assert_eq!(fields[0].0, "x");
                        assert!(matches!(fields[0].1, Expression::IntegerLiteral(10)));
                        
                        assert_eq!(fields[1].0, "y");
                        assert!(matches!(fields[1].1, Expression::IntegerLiteral(20)));
                    }
                    _ => panic!("Expected struct literal"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_struct_literal_with_trailing_comma() {
        let source = "let p = Point { x: 10, y: 20, };";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Let { value, .. }) => {
                match value.as_ref().unwrap() {
                    Expression::StructLiteral { fields, .. } => {
                        assert_eq!(fields.len(), 2);
                    }
                    _ => panic!("Expected struct literal"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_struct_literal_with_update_syntax() {
        let source = "let p2 = Point { x: 5, ..p1 };";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Let { value, .. }) => {
                match value.as_ref().unwrap() {
                    Expression::StructLiteral { name, fields, base } => {
                        assert_eq!(name, "Point");
                        assert_eq!(fields.len(), 1);
                        assert_eq!(fields[0].0, "x");
                        
                        assert!(base.is_some());
                        match base.as_ref().unwrap().as_ref() {
                            Expression::Identifier(base_name) => {
                                assert_eq!(base_name, "p1");
                            }
                            _ => panic!("Expected identifier in base"),
                        }
                    }
                    _ => panic!("Expected struct literal"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_empty_struct_literal() {
        let source = "let e = Empty { };";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Let { value, .. }) => {
                match value.as_ref().unwrap() {
                    Expression::StructLiteral { name, fields, base } => {
                        assert_eq!(name, "Empty");
                        assert_eq!(fields.len(), 0);
                        assert!(base.is_none());
                    }
                    _ => panic!("Expected struct literal"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_struct_literal_with_expressions() {
        let source = "let p = Point { x: a + b, y: func_call() };";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Let { value, .. }) => {
                match value.as_ref().unwrap() {
                    Expression::StructLiteral { fields, .. } => {
                        assert_eq!(fields.len(), 2);
                        
                        // Check x field has binary expression
                        assert!(matches!(fields[0].1, Expression::Binary { .. }));
                        
                        // Check y field has function call
                        assert!(matches!(fields[1].1, Expression::FunctionCall { .. }));
                    }
                    _ => panic!("Expected struct literal"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    // ===== FIELD ACCESS PARSING TESTS =====

    #[test]
    fn test_simple_field_access() {
        let source = "let x = point.x;";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Let { value, .. }) => {
                match value.as_ref().unwrap() {
                    Expression::FieldAccess { object, field } => {
                        match object.as_ref() {
                            Expression::Identifier(obj_name) => {
                                assert_eq!(obj_name, "point");
                            }
                            _ => panic!("Expected identifier in field access object"),
                        }
                        assert_eq!(field, "x");
                    }
                    _ => panic!("Expected field access"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_chained_field_access() {
        let source = "let name = person.address.street;";
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        match &ast[0] {
            AstNode::Statement(Statement::Let { value, .. }) => {
                match value.as_ref().unwrap() {
                    Expression::FieldAccess { object, field } => {
                        assert_eq!(field, "street");
                        
                        // Check nested field access
                        match object.as_ref() {
                            Expression::FieldAccess { object: inner_obj, field: inner_field } => {
                                assert_eq!(inner_field, "address");
                                match inner_obj.as_ref() {
                                    Expression::Identifier(name) => {
                                        assert_eq!(name, "person");
                                    }
                                    _ => panic!("Expected identifier in nested field access"),
                                }
                            }
                            _ => panic!("Expected nested field access"),
                        }
                    }
                    _ => panic!("Expected field access"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    // ===== ERROR HANDLING TESTS =====

    #[test]
    fn test_struct_missing_name() {
        let source = "struct { x: i32 }";
        let mut parser = create_parser(source);
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_struct_missing_field_type() {
        let source = "struct Point { x: }";
        let mut parser = create_parser(source);
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_struct_missing_colon() {
        let source = "struct Point { x i32 }";
        let mut parser = create_parser(source);
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_struct_unclosed_brace() {
        let source = "struct Point { x: i32";
        let mut parser = create_parser(source);
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_tuple_struct_missing_semicolon() {
        let source = "struct Point(i32, i32)";
        let mut parser = create_parser(source);
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_generic_struct_unclosed_angle() {
        let source = "struct Container<T { value: T }";
        let mut parser = create_parser(source);
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_struct_literal_missing_colon() {
        let source = "let p = Point { x 10 };";
        let mut parser = create_parser(source);
        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_struct_literal_unclosed_brace() {
        let source = "let p = Point { x: 10";
        let mut parser = create_parser(source);
        let result = parser.parse();
        assert!(result.is_err());
    }

    // ===== INTEGRATION TESTS =====

    #[test]
    fn test_multiple_struct_definitions() {
        let source = r#"
            struct Point { x: i32, y: i32 }
            struct Color(u8, u8, u8);
            struct Person<T> { name: String, data: T }
        "#;
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        assert_eq!(ast.len(), 3);
        
        // Check all are struct statements
        for node in &ast {
            match node {
                AstNode::Statement(Statement::Struct { .. }) => {
                    // Expected
                }
                _ => panic!("Expected struct statement"),
            }
        }
    }

    #[test]
    fn test_struct_with_function() {
        let source = r#"
            struct Point { x: i32, y: i32 }
            
            fn create_point(x: i32, y: i32) -> Point {
                Point { x: x, y: y }
            }
        "#;
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        assert_eq!(ast.len(), 2);
        assert!(matches!(ast[0], AstNode::Statement(Statement::Struct { .. })));
        assert!(matches!(ast[1], AstNode::Statement(Statement::Function { .. })));
    }

    #[test]
    fn test_complex_struct_usage() {
        let source = r#"
            struct Point { x: i32, y: i32 }
            
            fn main() {
                let p1 = Point { x: 10, y: 20 };
                let p2 = Point { x: 5, ..p1 };
                let distance = p1.x + p2.y;
            }
        "#;
        let mut parser = create_parser(source);
        let ast = parser.parse().unwrap();
        
        assert_eq!(ast.len(), 2);
        
        // Check function body contains struct operations
        match &ast[1] {
            AstNode::Statement(Statement::Function { body, .. }) => {
                assert_eq!(body.statements.len(), 3);
                
                // Check struct literal
                match &body.statements[0] {
                    Statement::Let { value, .. } => {
                        assert!(matches!(value.as_ref().unwrap(), Expression::StructLiteral { .. }));
                    }
                    _ => panic!("Expected let with struct literal"),
                }
                
                // Check struct literal with update syntax
                match &body.statements[1] {
                    Statement::Let { value, .. } => {
                        match value.as_ref().unwrap() {
                            Expression::StructLiteral { base, .. } => {
                                assert!(base.is_some());
                            }
                            _ => panic!("Expected struct literal with base"),
                        }
                    }
                    _ => panic!("Expected let with struct literal"),
                }
                
                // Check field access in expression
                match &body.statements[2] {
                    Statement::Let { value, .. } => {
                        match value.as_ref().unwrap() {
                            Expression::Binary { left, right, .. } => {
                                assert!(matches!(left.as_ref(), Expression::FieldAccess { .. }));
                                assert!(matches!(right.as_ref(), Expression::FieldAccess { .. }));
                            }
                            _ => panic!("Expected binary expression with field access"),
                        }
                    }
                    _ => panic!("Expected let with binary expression"),
                }
            }
            _ => panic!("Expected function statement"),
        }
    }
}