#![allow(dead_code)]

use crate::ast::*;

#[cfg(test)]
mod tests {
    use super::*;

    // Generic and Collection AST Node Tests
    
    #[test]
    fn test_generic_struct_definition() {
        let generic_struct = Statement::Struct {
            name: "Container".to_string(),
            generics: vec!["T".to_string()],
            fields: vec![
                StructField {
                    name: "value".to_string(),
                    field_type: Type::Named("T".to_string()),
                    visibility: Visibility::Public,
                },
            ],
            is_tuple: false,
        };

        match generic_struct {
            Statement::Struct { name, generics, fields, is_tuple } => {
                assert_eq!(name, "Container");
                assert_eq!(generics.len(), 1);
                assert_eq!(generics[0], "T");
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].name, "value");
                assert_eq!(fields[0].field_type, Type::Named("T".to_string()));
                assert!(!is_tuple);
            }
            _ => panic!("Expected Struct statement"),
        }
    }

    #[test]
    fn test_generic_enum_definition() {
        let generic_enum = Statement::Enum {
            name: "Result".to_string(),
            generics: vec!["T".to_string(), "E".to_string()],
            variants: vec![
                EnumVariant {
                    name: "Ok".to_string(),
                    data: Some(EnumVariantData::Tuple(vec![Type::Named("T".to_string())])),
                },
                EnumVariant {
                    name: "Err".to_string(),
                    data: Some(EnumVariantData::Tuple(vec![Type::Named("E".to_string())])),
                },
            ],
        };

        match generic_enum {
            Statement::Enum { name, generics, variants } => {
                assert_eq!(name, "Result");
                assert_eq!(generics.len(), 2);
                assert_eq!(generics[0], "T");
                assert_eq!(generics[1], "E");
                assert_eq!(variants.len(), 2);
                assert_eq!(variants[0].name, "Ok");
                assert_eq!(variants[1].name, "Err");
            }
            _ => panic!("Expected Enum statement"),
        }
    }

    #[test]
    fn test_array_literal() {
        let array_literal = Expression::ArrayLiteral {
            elements: vec![
                Expression::IntegerLiteral(1),
                Expression::IntegerLiteral(2),
                Expression::IntegerLiteral(3),
                Expression::IntegerLiteral(4),
                Expression::IntegerLiteral(5),
            ],
        };

        match array_literal {
            Expression::ArrayLiteral { elements } => {
                assert_eq!(elements.len(), 5);
                for (i, element) in elements.iter().enumerate() {
                    match element {
                        Expression::IntegerLiteral(value) => {
                            assert_eq!(*value, (i + 1) as i64);
                        }
                        _ => panic!("Expected IntegerLiteral in array"),
                    }
                }
            }
            _ => panic!("Expected ArrayLiteral expression"),
        }
    }

    #[test]
    fn test_array_access() {
        let array_access = Expression::ArrayAccess {
            array: Box::new(Expression::Identifier("numbers".to_string())),
            index: Box::new(Expression::IntegerLiteral(0)),
        };

        match array_access {
            Expression::ArrayAccess { array, index } => {
                match array.as_ref() {
                    Expression::Identifier(name) => assert_eq!(name, "numbers"),
                    _ => panic!("Expected Identifier for array"),
                }
                match index.as_ref() {
                    Expression::IntegerLiteral(value) => assert_eq!(*value, 0),
                    _ => panic!("Expected IntegerLiteral for index"),
                }
            }
            _ => panic!("Expected ArrayAccess expression"),
        }
    }

    #[test]
    fn test_vec_macro() {
        let vec_macro = Expression::VecMacro {
            elements: vec![
                Expression::IntegerLiteral(10),
                Expression::IntegerLiteral(20),
                Expression::IntegerLiteral(30),
            ],
        };

        match vec_macro {
            Expression::VecMacro { elements } => {
                assert_eq!(elements.len(), 3);
                let expected_values = [10, 20, 30];
                for (i, element) in elements.iter().enumerate() {
                    match element {
                        Expression::IntegerLiteral(value) => {
                            assert_eq!(*value, expected_values[i]);
                        }
                        _ => panic!("Expected IntegerLiteral in vec macro"),
                    }
                }
            }
            _ => panic!("Expected VecMacro expression"),
        }
    }

    #[test]
    fn test_generic_types() {
        // Test Generic type
        let generic_type = Type::Generic {
            name: "Vec".to_string(),
            type_args: vec![Type::Named("i32".to_string())],
        };

        match generic_type {
            Type::Generic { name, type_args } => {
                assert_eq!(name, "Vec");
                assert_eq!(type_args.len(), 1);
                assert_eq!(type_args[0], Type::Named("i32".to_string()));
            }
            _ => panic!("Expected Generic type"),
        }

        // Test Array type
        let array_type = Type::Array {
            element_type: Box::new(Type::Named("f64".to_string())),
            size: Some(10),
        };

        match array_type {
            Type::Array { element_type, size } => {
                assert_eq!(*element_type, Type::Named("f64".to_string()));
                assert_eq!(size, Some(10));
            }
            _ => panic!("Expected Array type"),
        }

        // Test Slice type
        let slice_type = Type::Slice {
            element_type: Box::new(Type::Named("u8".to_string())),
        };

        match slice_type {
            Type::Slice { element_type } => {
                assert_eq!(*element_type, Type::Named("u8".to_string()));
            }
            _ => panic!("Expected Slice type"),
        }

        // Test Vec type
        let vec_type = Type::Vec {
            element_type: Box::new(Type::Named("String".to_string())),
        };

        match vec_type {
            Type::Vec { element_type } => {
                assert_eq!(*element_type, Type::Named("String".to_string()));
            }
            _ => panic!("Expected Vec type"),
        }

        // Test HashMap type
        let hashmap_type = Type::HashMap {
            key_type: Box::new(Type::Named("String".to_string())),
            value_type: Box::new(Type::Named("i32".to_string())),
        };

        match hashmap_type {
            Type::HashMap { key_type, value_type } => {
                assert_eq!(*key_type, Type::Named("String".to_string()));
                assert_eq!(*value_type, Type::Named("i32".to_string()));
            }
            _ => panic!("Expected HashMap type"),
        }
    }

    #[test]
    fn test_nested_generic_types() {
        // Test Vec<Vec<i32>>
        let nested_vec_type = Type::Vec {
            element_type: Box::new(Type::Vec {
                element_type: Box::new(Type::Named("i32".to_string())),
            }),
        };

        match nested_vec_type {
            Type::Vec { element_type } => {
                match element_type.as_ref() {
                    Type::Vec { element_type: inner_element_type } => {
                        assert_eq!(**inner_element_type, Type::Named("i32".to_string()));
                    }
                    _ => panic!("Expected nested Vec type"),
                }
            }
            _ => panic!("Expected Vec type"),
        }

        // Test HashMap<String, Vec<i32>>
        let complex_hashmap_type = Type::HashMap {
            key_type: Box::new(Type::Named("String".to_string())),
            value_type: Box::new(Type::Vec {
                element_type: Box::new(Type::Named("i32".to_string())),
            }),
        };

        match complex_hashmap_type {
            Type::HashMap { key_type, value_type } => {
                assert_eq!(*key_type, Type::Named("String".to_string()));
                match value_type.as_ref() {
                    Type::Vec { element_type } => {
                        assert_eq!(**element_type, Type::Named("i32".to_string()));
                    }
                    _ => panic!("Expected Vec type for HashMap value"),
                }
            }
            _ => panic!("Expected HashMap type"),
        }
    }

    #[test]
    fn test_complex_generic_expressions() {
        // Test nested array access with method call
        let complex_expr = Expression::MethodCall {
            object: Box::new(Expression::ArrayAccess {
                array: Box::new(Expression::Identifier("containers".to_string())),
                index: Box::new(Expression::IntegerLiteral(0)),
            }),
            method: "get_value".to_string(),
            arguments: vec![],
        };

        match complex_expr {
            Expression::MethodCall { object, method, arguments } => {
                assert_eq!(method, "get_value");
                assert_eq!(arguments.len(), 0);
                match object.as_ref() {
                    Expression::ArrayAccess { array, index } => {
                        match array.as_ref() {
                            Expression::Identifier(name) => assert_eq!(name, "containers"),
                            _ => panic!("Expected Identifier for array"),
                        }
                        match index.as_ref() {
                            Expression::IntegerLiteral(value) => assert_eq!(*value, 0),
                            _ => panic!("Expected IntegerLiteral for index"),
                        }
                    }
                    _ => panic!("Expected ArrayAccess for object"),
                }
            }
            _ => panic!("Expected MethodCall expression"),
        }
    }

    #[test]
    fn test_method_call_on_collections() {
        let method_call = Expression::MethodCall {
            object: Box::new(Expression::Identifier("vec".to_string())),
            method: "push".to_string(),
            arguments: vec![Expression::IntegerLiteral(42)],
        };

        match method_call {
            Expression::MethodCall { object, method, arguments } => {
                match object.as_ref() {
                    Expression::Identifier(name) => assert_eq!(name, "vec"),
                    _ => panic!("Expected Identifier for object"),
                }
                assert_eq!(method, "push");
                assert_eq!(arguments.len(), 1);
                match &arguments[0] {
                    Expression::IntegerLiteral(value) => assert_eq!(*value, 42),
                    _ => panic!("Expected IntegerLiteral for argument"),
                }
            }
            _ => panic!("Expected MethodCall expression"),
        }
    }

    #[test]
    fn test_format_macro() {
        let format_macro = Expression::FormatMacro {
            format_string: "Hello, {}! You have {} messages.".to_string(),
            arguments: vec![
                Expression::Identifier("name".to_string()),
                Expression::IntegerLiteral(5),
            ],
        };

        match format_macro {
            Expression::FormatMacro { format_string, arguments } => {
                assert_eq!(format_string, "Hello, {}! You have {} messages.");
                assert_eq!(arguments.len(), 2);
                match &arguments[0] {
                    Expression::Identifier(name) => assert_eq!(name, "name"),
                    _ => panic!("Expected Identifier for first argument"),
                }
                match &arguments[1] {
                    Expression::IntegerLiteral(value) => assert_eq!(*value, 5),
                    _ => panic!("Expected IntegerLiteral for second argument"),
                }
            }
            _ => panic!("Expected FormatMacro expression"),
        }
    }

    #[test]
    fn test_reference_types() {
        // Test mutable reference
        let mut_ref_type = Type::Reference {
            mutable: true,
            inner_type: Box::new(Type::Named("String".to_string())),
        };

        match mut_ref_type {
            Type::Reference { mutable, inner_type } => {
                assert!(mutable);
                assert_eq!(*inner_type, Type::Named("String".to_string()));
            }
            _ => panic!("Expected Reference type"),
        }

        // Test immutable reference to Vec
        let ref_type = Type::Reference {
            mutable: false,
            inner_type: Box::new(Type::Vec {
                element_type: Box::new(Type::Named("i32".to_string())),
            }),
        };

        match ref_type {
            Type::Reference { mutable, inner_type } => {
                assert!(!mutable);
                match inner_type.as_ref() {
                    Type::Vec { element_type } => {
                        assert_eq!(**element_type, Type::Named("i32".to_string()));
                    }
                    _ => panic!("Expected Vec type for reference"),
                }
            }
            _ => panic!("Expected Reference type"),
        }
    }

    #[test]
    fn test_generic_impl_block() {
        let generic_impl = Statement::Impl {
            generics: vec!["T".to_string()],
            type_name: "Container".to_string(),
            trait_name: None,
            methods: vec![
                Function {
                    name: "new".to_string(),
                    parameters: vec![
                        Parameter {
                            name: "value".to_string(),
                            param_type: Type::Named("T".to_string()),
                        }
                    ],
                    return_type: Some(Type::Generic {
                        name: "Container".to_string(),
                        type_args: vec![Type::Named("T".to_string())],
                    }),
                    body: Block {
                        statements: vec![],
                        expression: Some(Expression::StructLiteral {
                            name: "Container".to_string(),
                            fields: vec![
                                ("value".to_string(), Expression::Identifier("value".to_string()))
                            ],
                            base: None,
                        }),
                    },
                }
            ],
        };

        match generic_impl {
            Statement::Impl { generics, type_name, trait_name, methods } => {
                assert_eq!(generics.len(), 1);
                assert_eq!(generics[0], "T");
                assert_eq!(type_name, "Container");
                assert!(trait_name.is_none());
                assert_eq!(methods.len(), 1);
                assert_eq!(methods[0].name, "new");
                assert_eq!(methods[0].parameters.len(), 1);
                assert_eq!(methods[0].parameters[0].name, "value");
            }
            _ => panic!("Expected Impl statement"),
        }
    }

    #[test]
    fn test_array_type_with_size() {
        let array_type = Type::Array {
            element_type: Box::new(Type::Named("i32".to_string())),
            size: Some(100),
        };

        match array_type {
            Type::Array { element_type, size } => {
                assert_eq!(*element_type, Type::Named("i32".to_string()));
                assert_eq!(size, Some(100));
            }
            _ => panic!("Expected Array type"),
        }
    }

    #[test]
    fn test_array_type_without_size() {
        let array_type = Type::Array {
            element_type: Box::new(Type::Named("f64".to_string())),
            size: None,
        };

        match array_type {
            Type::Array { element_type, size } => {
                assert_eq!(*element_type, Type::Named("f64".to_string()));
                assert_eq!(size, None);
            }
            _ => panic!("Expected Array type"),
        }
    }

    #[test]
    fn test_complex_nested_collections() {
        // Test HashMap<String, Vec<Option<i32>>>
        let complex_type = Type::HashMap {
            key_type: Box::new(Type::Named("String".to_string())),
            value_type: Box::new(Type::Vec {
                element_type: Box::new(Type::Generic {
                    name: "Option".to_string(),
                    type_args: vec![Type::Named("i32".to_string())],
                }),
            }),
        };

        match complex_type {
            Type::HashMap { key_type, value_type } => {
                assert_eq!(*key_type, Type::Named("String".to_string()));
                match value_type.as_ref() {
                    Type::Vec { element_type } => {
                        match element_type.as_ref() {
                            Type::Generic { name, type_args } => {
                                assert_eq!(name, "Option");
                                assert_eq!(type_args.len(), 1);
                                assert_eq!(type_args[0], Type::Named("i32".to_string()));
                            }
                            _ => panic!("Expected Generic type for Vec element"),
                        }
                    }
                    _ => panic!("Expected Vec type for HashMap value"),
                }
            }
            _ => panic!("Expected HashMap type"),
        }
    }

    #[test]
    fn test_chained_method_calls() {
        // Test vec.iter().map().collect()
        let chained_call = Expression::MethodCall {
            object: Box::new(Expression::MethodCall {
                object: Box::new(Expression::MethodCall {
                    object: Box::new(Expression::Identifier("vec".to_string())),
                    method: "iter".to_string(),
                    arguments: vec![],
                }),
                method: "map".to_string(),
                arguments: vec![Expression::Identifier("closure".to_string())],
            }),
            method: "collect".to_string(),
            arguments: vec![],
        };

        match chained_call {
            Expression::MethodCall { object, method, arguments } => {
                assert_eq!(method, "collect");
                assert_eq!(arguments.len(), 0);
                
                match object.as_ref() {
                    Expression::MethodCall { object: inner_object, method: inner_method, arguments: inner_args } => {
                        assert_eq!(inner_method, "map");
                        assert_eq!(inner_args.len(), 1);
                        
                        match inner_object.as_ref() {
                            Expression::MethodCall { object: innermost_object, method: innermost_method, arguments: innermost_args } => {
                                assert_eq!(innermost_method, "iter");
                                assert_eq!(innermost_args.len(), 0);
                                
                                match innermost_object.as_ref() {
                                    Expression::Identifier(name) => assert_eq!(name, "vec"),
                                    _ => panic!("Expected Identifier for innermost object"),
                                }
                            }
                            _ => panic!("Expected MethodCall for inner object"),
                        }
                    }
                    _ => panic!("Expected MethodCall for object"),
                }
            }
            _ => panic!("Expected MethodCall expression"),
        }
    }
}