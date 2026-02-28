use std::path::Path;

// Add the compiler crate to the path
fn main() {
    // Add the compiler source to the module path
    let compiler_path = Path::new("src/compiler/src");
    println!("Testing struct parsing functionality...");
    
    // Test basic struct parsing
    test_basic_struct_parsing();
    test_tuple_struct_parsing();
    test_generic_struct_parsing();
    test_struct_literal_parsing();
    test_field_access_parsing();
    
    println!("All struct parsing tests passed!");
}

fn test_basic_struct_parsing() {
    use compiler::lexer::tokenize_with_locations;
    use compiler::parser::Parser;
    use compiler::ast::{AstNode, Statement, Type, Visibility};
    
    let source = "struct Point { x: i32, y: i32 }";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
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
    
    println!("✓ Basic struct parsing test passed");
}

fn test_tuple_struct_parsing() {
    use compiler::lexer::tokenize_with_locations;
    use compiler::parser::Parser;
    use compiler::ast::{AstNode, Statement, Type, Visibility};
    
    let source = "struct Point(i32, i32);";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
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
    
    println!("✓ Tuple struct parsing test passed");
}

fn test_generic_struct_parsing() {
    use compiler::lexer::tokenize_with_locations;
    use compiler::parser::Parser;
    use compiler::ast::{AstNode, Statement, Type};
    
    let source = "struct Container<T> { value: T }";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
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
    
    println!("✓ Generic struct parsing test passed");
}

fn test_struct_literal_parsing() {
    use compiler::lexer::tokenize_with_locations;
    use compiler::parser::Parser;
    use compiler::ast::{AstNode, Statement, Expression};
    
    let source = "let p = Point { x: 10, y: 20 };";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
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
    
    println!("✓ Struct literal parsing test passed");
}

fn test_field_access_parsing() {
    use compiler::lexer::tokenize_with_locations;
    use compiler::parser::Parser;
    use compiler::ast::{AstNode, Statement, Expression};
    
    let source = "let x = point.x;";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
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
    
    println!("✓ Field access parsing test passed");
}