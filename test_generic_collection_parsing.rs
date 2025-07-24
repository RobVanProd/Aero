// Standalone test for generic and collection parsing functionality
use std::path::Path;

// Include the compiler modules
mod compiler {
    pub mod src {
        pub mod lexer;
        pub mod parser;
        pub mod ast;
        pub mod errors;
        pub mod types;
    }
}

use compiler::src::lexer::tokenize_with_locations;
use compiler::src::parser::Parser;
use compiler::src::ast::*;

fn main() {
    println!("Testing Generic and Collection Parsing Implementation");
    
    // Test 1: Generic struct definition
    test_generic_struct_definition();
    
    // Test 2: Generic enum definition
    test_generic_enum_definition();
    
    // Test 3: Vec type parsing
    test_vec_type_parsing();
    
    // Test 4: HashMap type parsing
    test_hashmap_type_parsing();
    
    // Test 5: Array type parsing
    test_array_type_parsing();
    
    // Test 6: Method call parsing
    test_method_call_parsing();
    
    println!("All tests completed!");
}

fn test_generic_struct_definition() {
    println!("Test 1: Generic struct definition");
    let source = "struct Container<T> { value: T }";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
    
    match parser.parse() {
        Ok(result) => {
            assert_eq!(result.len(), 1);
            match &result[0] {
                AstNode::Statement(Statement::Struct { name, generics, fields, is_tuple }) => {
                    assert_eq!(name, "Container");
                    assert_eq!(generics.len(), 1);
                    assert_eq!(generics[0], "T");
                    assert!(!is_tuple);
                    assert_eq!(fields.len(), 1);
                    assert_eq!(fields[0].name, "value");
                    assert_eq!(fields[0].field_type, Type::Named("T".to_string()));
                    println!("✓ Generic struct definition parsed correctly");
                }
                _ => panic!("Expected struct definition"),
            }
        }
        Err(e) => {
            println!("✗ Failed to parse generic struct: {}", e);
        }
    }
}

fn test_generic_enum_definition() {
    println!("Test 2: Generic enum definition");
    let source = "enum Option<T> { Some(T), None }";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
    
    match parser.parse() {
        Ok(result) => {
            assert_eq!(result.len(), 1);
            match &result[0] {
                AstNode::Statement(Statement::Enum { name, generics, variants }) => {
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
                    println!("✓ Generic enum definition parsed correctly");
                }
                _ => panic!("Expected enum definition"),
            }
        }
        Err(e) => {
            println!("✗ Failed to parse generic enum: {}", e);
        }
    }
}

fn test_vec_type_parsing() {
    println!("Test 3: Vec type parsing");
    let source = "let v: Vec<i32>;";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
    
    match parser.parse() {
        Ok(result) => {
            assert_eq!(result.len(), 1);
            match &result[0] {
                AstNode::Statement(Statement::Let { name, type_annotation, .. }) => {
                    assert_eq!(name, "v");
                    
                    // Check type annotation
                    match type_annotation {
                        Some(Type::Vec { element_type }) => {
                            assert_eq!(**element_type, Type::Named("i32".to_string()));
                            println!("✓ Vec type parsed correctly");
                        }
                        _ => panic!("Expected Vec type annotation"),
                    }
                }
                _ => panic!("Expected let statement"),
            }
        }
        Err(e) => {
            println!("✗ Failed to parse Vec type: {}", e);
        }
    }
}

fn test_hashmap_type_parsing() {
    println!("Test 4: HashMap type parsing");
    let source = "let map: HashMap<String, i32>;";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
    
    match parser.parse() {
        Ok(result) => {
            assert_eq!(result.len(), 1);
            match &result[0] {
                AstNode::Statement(Statement::Let { name, type_annotation, .. }) => {
                    assert_eq!(name, "map");
                    
                    // Check type annotation
                    match type_annotation {
                        Some(Type::HashMap { key_type, value_type }) => {
                            assert_eq!(**key_type, Type::Named("String".to_string()));
                            assert_eq!(**value_type, Type::Named("i32".to_string()));
                            println!("✓ HashMap type parsed correctly");
                        }
                        _ => panic!("Expected HashMap type annotation"),
                    }
                }
                _ => panic!("Expected let statement"),
            }
        }
        Err(e) => {
            println!("✗ Failed to parse HashMap type: {}", e);
        }
    }
}

fn test_array_type_parsing() {
    println!("Test 5: Array type parsing");
    let source = "let arr: [i32; 5];";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
    
    match parser.parse() {
        Ok(result) => {
            assert_eq!(result.len(), 1);
            match &result[0] {
                AstNode::Statement(Statement::Let { name, type_annotation, .. }) => {
                    assert_eq!(name, "arr");
                    
                    // Check type annotation
                    match type_annotation {
                        Some(Type::Array { element_type, size }) => {
                            assert_eq!(**element_type, Type::Named("i32".to_string()));
                            assert_eq!(*size, Some(5));
                            println!("✓ Array type parsed correctly");
                        }
                        _ => panic!("Expected Array type annotation"),
                    }
                }
                _ => panic!("Expected let statement"),
            }
        }
        Err(e) => {
            println!("✗ Failed to parse Array type: {}", e);
        }
    }
}

fn test_method_call_parsing() {
    println!("Test 6: Method call parsing");
    let source = "let result = vec.push(42);";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
    
    match parser.parse() {
        Ok(result) => {
            assert_eq!(result.len(), 1);
            match &result[0] {
                AstNode::Statement(Statement::Let { name, value, .. }) => {
                    assert_eq!(name, "result");
                    
                    // Check method call
                    match value {
                        Some(Expression::MethodCall { object, method, arguments }) => {
                            assert!(matches!(**object, Expression::Identifier(_)));
                            assert_eq!(method, "push");
                            assert_eq!(arguments.len(), 1);
                            assert!(matches!(arguments[0], Expression::IntegerLiteral(42)));
                            println!("✓ Method call parsed correctly");
                        }
                        _ => panic!("Expected method call"),
                    }
                }
                _ => panic!("Expected let statement"),
            }
        }
        Err(e) => {
            println!("✗ Failed to parse method call: {}", e);
        }
    }
}