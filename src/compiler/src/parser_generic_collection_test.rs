use super::*;
use crate::lexer::tokenize_with_locations;
use crate::ast::*;

#[test]
fn test_generic_struct_definition() {
    let source = "struct Container<T> { value: T }";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    
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
        }
        _ => panic!("Expected struct definition"),
    }
}

#[test]
fn test_generic_struct_multiple_parameters() {
    let source = "struct Pair<T, U> { first: T, second: U }";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    
    assert_eq!(result.len(), 1);
    match &result[0] {
        AstNode::Statement(Statement::Struct { name, generics, fields, is_tuple }) => {
            assert_eq!(name, "Pair");
            assert_eq!(generics.len(), 2);
            assert_eq!(generics[0], "T");
            assert_eq!(generics[1], "U");
            assert!(!is_tuple);
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].name, "first");
            assert_eq!(fields[0].field_type, Type::Named("T".to_string()));
            assert_eq!(fields[1].name, "second");
            assert_eq!(fields[1].field_type, Type::Named("U".to_string()));
        }
        _ => panic!("Expected struct definition"),
    }
}

#[test]
fn test_generic_enum_definition() {
    let source = "enum Option<T> { Some(T), None }";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    
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
        }
        _ => panic!("Expected enum definition"),
    }
}

#[test]
fn test_generic_impl_block() {
    let source = "impl<T> Container<T> { fn new(value: T) -> Container<T> { Container { value } } }";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    
    assert_eq!(result.len(), 1);
    match &result[0] {
        AstNode::Statement(Statement::Impl { generics, type_name, trait_name, methods }) => {
            assert_eq!(generics.len(), 1);
            assert_eq!(generics[0], "T");
            assert_eq!(type_name, "Container<T>");
            assert!(trait_name.is_none());
            assert_eq!(methods.len(), 1);
            assert_eq!(methods[0].name, "new");
        }
        _ => panic!("Expected impl block"),
    }
}

#[test]
fn test_vec_type_parsing() {
    let source = "let v: Vec<i32> = vec![1, 2, 3];";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    
    assert_eq!(result.len(), 1);
    match &result[0] {
        AstNode::Statement(Statement::Let { name, type_annotation, value, .. }) => {
            assert_eq!(name, "v");
            
            // Check type annotation
            match type_annotation {
                Some(Type::Vec { element_type }) => {
                    assert_eq!(**element_type, Type::Named("i32".to_string()));
                }
                _ => panic!("Expected Vec type annotation"),
            }
            
            // Check vec! macro value
            match value {
                Some(Expression::VecMacro { elements }) => {
                    assert_eq!(elements.len(), 3);
                    assert!(matches!(elements[0], Expression::IntegerLiteral(1)));
                    assert!(matches!(elements[1], Expression::IntegerLiteral(2)));
                    assert!(matches!(elements[2], Expression::IntegerLiteral(3)));
                }
                _ => panic!("Expected vec! macro"),
            }
        }
        _ => panic!("Expected let statement"),
    }
}

#[test]
fn test_hashmap_type_parsing() {
    let source = "let map: HashMap<String, i32>;";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    
    assert_eq!(result.len(), 1);
    match &result[0] {
        AstNode::Statement(Statement::Let { name, type_annotation, .. }) => {
            assert_eq!(name, "map");
            
            // Check type annotation
            match type_annotation {
                Some(Type::HashMap { key_type, value_type }) => {
                    assert_eq!(**key_type, Type::Named("String".to_string()));
                    assert_eq!(**value_type, Type::Named("i32".to_string()));
                }
                _ => panic!("Expected HashMap type annotation"),
            }
        }
        _ => panic!("Expected let statement"),
    }
}

#[test]
fn test_array_type_parsing() {
    let source = "let arr: [i32; 5];";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    
    assert_eq!(result.len(), 1);
    match &result[0] {
        AstNode::Statement(Statement::Let { name, type_annotation, .. }) => {
            assert_eq!(name, "arr");
            
            // Check type annotation
            match type_annotation {
                Some(Type::Array { element_type, size }) => {
                    assert_eq!(**element_type, Type::Named("i32".to_string()));
                    assert_eq!(*size, Some(5));
                }
                _ => panic!("Expected Array type annotation"),
            }
        }
        _ => panic!("Expected let statement"),
    }
}

#[test]
fn test_slice_type_parsing() {
    let source = "let slice: [i32];";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    
    assert_eq!(result.len(), 1);
    match &result[0] {
        AstNode::Statement(Statement::Let { name, type_annotation, .. }) => {
            assert_eq!(name, "slice");
            
            // Check type annotation
            match type_annotation {
                Some(Type::Slice { element_type }) => {
                    assert_eq!(**element_type, Type::Named("i32".to_string()));
                }
                _ => panic!("Expected Slice type annotation"),
            }
        }
        _ => panic!("Expected let statement"),
    }
}

#[test]
fn test_method_call_parsing() {
    let source = "let result = vec.push(42);";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    
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
                }
                _ => panic!("Expected method call"),
            }
        }
        _ => panic!("Expected let statement"),
    }
}

#[test]
fn test_array_access_parsing() {
    let source = "let item = arr[0];";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    
    assert_eq!(result.len(), 1);
    match &result[0] {
        AstNode::Statement(Statement::Let { name, value, .. }) => {
            assert_eq!(name, "item");
            
            // Check array access
            match value {
                Some(Expression::ArrayAccess { array, index }) => {
                    assert!(matches!(**array, Expression::Identifier(_)));
                    assert!(matches!(**index, Expression::IntegerLiteral(0)));
                }
                _ => panic!("Expected array access"),
            }
        }
        _ => panic!("Expected let statement"),
    }
}

#[test]
fn test_format_macro_parsing() {
    let source = r#"let msg = format!("Hello {}", name);"#;
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    
    assert_eq!(result.len(), 1);
    match &result[0] {
        AstNode::Statement(Statement::Let { name, value, .. }) => {
            assert_eq!(name, "msg");
            
            // Check format! macro
            match value {
                Some(Expression::FormatMacro { format_string, arguments }) => {
                    assert_eq!(format_string, r#""Hello {}""#);
                    assert_eq!(arguments.len(), 1);
                    assert!(matches!(arguments[0], Expression::Identifier(_)));
                }
                _ => panic!("Expected format! macro"),
            }
        }
        _ => panic!("Expected let statement"),
    }
}

#[test]
fn test_nested_generic_types() {
    let source = "let nested: Vec<Vec<i32>>;";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    
    assert_eq!(result.len(), 1);
    match &result[0] {
        AstNode::Statement(Statement::Let { name, type_annotation, .. }) => {
            assert_eq!(name, "nested");
            
            // Check nested generic type
            match type_annotation {
                Some(Type::Vec { element_type }) => {
                    match element_type.as_ref() {
                        Type::Vec { element_type: inner_type } => {
                            assert_eq!(**inner_type, Type::Named("i32".to_string()));
                        }
                        _ => panic!("Expected nested Vec type"),
                    }
                }
                _ => panic!("Expected Vec type annotation"),
            }
        }
        _ => panic!("Expected let statement"),
    }
}

#[test]
fn test_complex_generic_struct() {
    let source = "struct Node<T> { value: T, children: Vec<Node<T>> }";
    let tokens = tokenize_with_locations(source, None);
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    
    assert_eq!(result.len(), 1);
    match &result[0] {
        AstNode::Statement(Statement::Struct { name, generics, fields, .. }) => {
            assert_eq!(name, "Node");
            assert_eq!(generics.len(), 1);
            assert_eq!(generics[0], "T");
            assert_eq!(fields.len(), 2);
            
            // Check value field
            assert_eq!(fields[0].name, "value");
            assert_eq!(fields[0].field_type, Type::Named("T".to_string()));
            
            // Check children field
            assert_eq!(fields[1].name, "children");
            match &fields[1].field_type {
                Type::Vec { element_type } => {
                    match element_type.as_ref() {
                        Type::Generic { name, type_args } => {
                            assert_eq!(name, "Node");
                            assert_eq!(type_args.len(), 1);
                            assert_eq!(type_args[0], Type::Named("T".to_string()));
                        }
                        _ => panic!("Expected generic Node type"),
                    }
                }
                _ => panic!("Expected Vec type"),
            }
        }
        _ => panic!("Expected struct definition"),
    }
}