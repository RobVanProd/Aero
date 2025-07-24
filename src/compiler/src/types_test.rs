// src/compiler/src/types_test.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Type, StructField, Visibility, Function, Parameter, Block};

    fn create_test_memory_layout() -> MemoryLayout {
        MemoryLayout {
            size: 16,
            alignment: 8,
            field_offsets: vec![0, 8],
        }
    }

    #[test]
    fn test_type_definition_manager_new() {
        let manager = TypeDefinitionManager::new();
        assert!(manager.get_struct("Point").is_none());
    }

    #[test]
    fn test_define_struct_success() {
        let mut manager = TypeDefinitionManager::new();
        
        let struct_def = StructDefinition {
            name: "Point".to_string(),
            generics: vec![],
            fields: vec![
                StructField {
                    name: "x".to_string(),
                    field_type: Type::Named("int".to_string()),
                    visibility: Visibility::Public,
                },
                StructField {
                    name: "y".to_string(),
                    field_type: Type::Named("int".to_string()),
                    visibility: Visibility::Public,
                },
            ],
            is_tuple: false,
            layout: create_test_memory_layout(),
        };

        let result = manager.define_struct(struct_def);
        assert!(result.is_ok());
        assert!(manager.get_struct("Point").is_some());
    }

    #[test]
    fn test_define_struct_duplicate_error() {
        let mut manager = TypeDefinitionManager::new();
        
        let struct_def = StructDefinition {
            name: "Point".to_string(),
            generics: vec![],
            fields: vec![
                StructField {
                    name: "x".to_string(),
                    field_type: Type::Named("int".to_string()),
                    visibility: Visibility::Public,
                },
            ],
            is_tuple: false,
            layout: create_test_memory_layout(),
        };

        // First definition should succeed
        assert!(manager.define_struct(struct_def.clone()).is_ok());
        
        // Second definition should fail
        let result = manager.define_struct(struct_def);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already defined"));
    }

    #[test]
    fn test_define_struct_duplicate_fields_error() {
        let mut manager = TypeDefinitionManager::new();
        
        let struct_def = StructDefinition {
            name: "Point".to_string(),
            generics: vec![],
            fields: vec![
                StructField {
                    name: "x".to_string(),
                    field_type: Type::Named("int".to_string()),
                    visibility: Visibility::Public,
                },
                StructField {
                    name: "x".to_string(), // Duplicate field name
                    field_type: Type::Named("float".to_string()),
                    visibility: Visibility::Public,
                },
            ],
            is_tuple: false,
            layout: create_test_memory_layout(),
        };

        let result = manager.define_struct(struct_def);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate field"));
    }

    #[test]
    fn test_validate_field_access_success() {
        let mut manager = TypeDefinitionManager::new();
        
        let struct_def = StructDefinition {
            name: "Point".to_string(),
            generics: vec![],
            fields: vec![
                StructField {
                    name: "x".to_string(),
                    field_type: Type::Named("int".to_string()),
                    visibility: Visibility::Public,
                },
                StructField {
                    name: "y".to_string(),
                    field_type: Type::Named("float".to_string()),
                    visibility: Visibility::Public,
                },
            ],
            is_tuple: false,
            layout: create_test_memory_layout(),
        };

        manager.define_struct(struct_def).unwrap();

        let result = manager.validate_field_access("Point", "x");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Ty::Int);

        let result = manager.validate_field_access("Point", "y");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Ty::Float);
    }

    #[test]
    fn test_validate_field_access_undefined_struct() {
        let manager = TypeDefinitionManager::new();
        
        let result = manager.validate_field_access("UndefinedStruct", "x");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Undefined struct type"));
    }

    #[test]
    fn test_validate_field_access_undefined_field() {
        let mut manager = TypeDefinitionManager::new();
        
        let struct_def = StructDefinition {
            name: "Point".to_string(),
            generics: vec![],
            fields: vec![
                StructField {
                    name: "x".to_string(),
                    field_type: Type::Named("int".to_string()),
                    visibility: Visibility::Public,
                },
            ],
            is_tuple: false,
            layout: create_test_memory_layout(),
        };

        manager.define_struct(struct_def).unwrap();

        let result = manager.validate_field_access("Point", "z");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Field 'z' not found"));
    }

    #[test]
    fn test_validate_struct_instantiation_success() {
        let mut manager = TypeDefinitionManager::new();
        
        let struct_def = StructDefinition {
            name: "Point".to_string(),
            generics: vec![],
            fields: vec![
                StructField {
                    name: "x".to_string(),
                    field_type: Type::Named("int".to_string()),
                    visibility: Visibility::Public,
                },
                StructField {
                    name: "y".to_string(),
                    field_type: Type::Named("int".to_string()),
                    visibility: Visibility::Public,
                },
            ],
            is_tuple: false,
            layout: create_test_memory_layout(),
        };

        manager.define_struct(struct_def).unwrap();

        let provided_fields = vec![
            ("x".to_string(), Ty::Int),
            ("y".to_string(), Ty::Int),
        ];

        let result = manager.validate_struct_instantiation("Point", &provided_fields);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_struct_instantiation_missing_field() {
        let mut manager = TypeDefinitionManager::new();
        
        let struct_def = StructDefinition {
            name: "Point".to_string(),
            generics: vec![],
            fields: vec![
                StructField {
                    name: "x".to_string(),
                    field_type: Type::Named("int".to_string()),
                    visibility: Visibility::Public,
                },
                StructField {
                    name: "y".to_string(),
                    field_type: Type::Named("int".to_string()),
                    visibility: Visibility::Public,
                },
            ],
            is_tuple: false,
            layout: create_test_memory_layout(),
        };

        manager.define_struct(struct_def).unwrap();

        let provided_fields = vec![
            ("x".to_string(), Ty::Int),
            // Missing "y" field
        ];

        let result = manager.validate_struct_instantiation("Point", &provided_fields);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing field 'y'"));
    }

    #[test]
    fn test_validate_struct_instantiation_type_mismatch() {
        let mut manager = TypeDefinitionManager::new();
        
        let struct_def = StructDefinition {
            name: "Point".to_string(),
            generics: vec![],
            fields: vec![
                StructField {
                    name: "x".to_string(),
                    field_type: Type::Named("int".to_string()),
                    visibility: Visibility::Public,
                },
                StructField {
                    name: "y".to_string(),
                    field_type: Type::Named("int".to_string()),
                    visibility: Visibility::Public,
                },
            ],
            is_tuple: false,
            layout: create_test_memory_layout(),
        };

        manager.define_struct(struct_def).unwrap();

        let provided_fields = vec![
            ("x".to_string(), Ty::Int),
            ("y".to_string(), Ty::Float), // Type mismatch
        ];

        let result = manager.validate_struct_instantiation("Point", &provided_fields);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Type mismatch"));
    }

    #[test]
    fn test_validate_struct_instantiation_unknown_field() {
        let mut manager = TypeDefinitionManager::new();
        
        let struct_def = StructDefinition {
            name: "Point".to_string(),
            generics: vec![],
            fields: vec![
                StructField {
                    name: "x".to_string(),
                    field_type: Type::Named("int".to_string()),
                    visibility: Visibility::Public,
                },
            ],
            is_tuple: false,
            layout: create_test_memory_layout(),
        };

        manager.define_struct(struct_def).unwrap();

        let provided_fields = vec![
            ("x".to_string(), Ty::Int),
            ("z".to_string(), Ty::Int), // Unknown field
        ];

        let result = manager.validate_struct_instantiation("Point", &provided_fields);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown field 'z'"));
    }

    #[test]
    fn test_add_impl_success() {
        let mut manager = TypeDefinitionManager::new();
        
        // First define a struct
        let struct_def = StructDefinition {
            name: "Point".to_string(),
            generics: vec![],
            fields: vec![
                StructField {
                    name: "x".to_string(),
                    field_type: Type::Named("int".to_string()),
                    visibility: Visibility::Public,
                },
            ],
            is_tuple: false,
            layout: create_test_memory_layout(),
        };

        manager.define_struct(struct_def).unwrap();

        // Now add an implementation
        let impl_block = ImplBlock {
            generics: vec![],
            type_name: "Point".to_string(),
            trait_name: None,
            methods: vec![
                Function {
                    name: "new".to_string(),
                    parameters: vec![
                        Parameter {
                            name: "x".to_string(),
                            param_type: Type::Named("int".to_string()),
                        },
                    ],
                    return_type: Some(Type::Named("Point".to_string())),
                    body: Block {
                        statements: vec![],
                        expression: None,
                    },
                },
            ],
        };

        let result = manager.add_impl(impl_block);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_impl_undefined_type() {
        let mut manager = TypeDefinitionManager::new();
        
        let impl_block = ImplBlock {
            generics: vec![],
            type_name: "UndefinedType".to_string(),
            trait_name: None,
            methods: vec![],
        };

        let result = manager.add_impl(impl_block);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot implement methods for undefined type"));
    }

    #[test]
    fn test_get_method_success() {
        let mut manager = TypeDefinitionManager::new();
        
        // Define struct
        let struct_def = StructDefinition {
            name: "Point".to_string(),
            generics: vec![],
            fields: vec![],
            is_tuple: false,
            layout: create_test_memory_layout(),
        };

        manager.define_struct(struct_def).unwrap();

        // Add implementation with method
        let impl_block = ImplBlock {
            generics: vec![],
            type_name: "Point".to_string(),
            trait_name: None,
            methods: vec![
                Function {
                    name: "distance".to_string(),
                    parameters: vec![],
                    return_type: Some(Type::Named("float".to_string())),
                    body: Block {
                        statements: vec![],
                        expression: None,
                    },
                },
            ],
        };

        manager.add_impl(impl_block).unwrap();

        let method = manager.get_method("Point", "distance");
        assert!(method.is_some());
        assert_eq!(method.unwrap().name, "distance");
    }

    #[test]
    fn test_get_method_not_found() {
        let mut manager = TypeDefinitionManager::new();
        
        // Define struct
        let struct_def = StructDefinition {
            name: "Point".to_string(),
            generics: vec![],
            fields: vec![],
            is_tuple: false,
            layout: create_test_memory_layout(),
        };

        manager.define_struct(struct_def).unwrap();

        let method = manager.get_method("Point", "nonexistent");
        assert!(method.is_none());
    }

    #[test]
    fn test_get_methods() {
        let mut manager = TypeDefinitionManager::new();
        
        // Define struct
        let struct_def = StructDefinition {
            name: "Point".to_string(),
            generics: vec![],
            fields: vec![],
            is_tuple: false,
            layout: create_test_memory_layout(),
        };

        manager.define_struct(struct_def).unwrap();

        // Add implementation with multiple methods
        let impl_block = ImplBlock {
            generics: vec![],
            type_name: "Point".to_string(),
            trait_name: None,
            methods: vec![
                Function {
                    name: "new".to_string(),
                    parameters: vec![],
                    return_type: Some(Type::Named("Point".to_string())),
                    body: Block {
                        statements: vec![],
                        expression: None,
                    },
                },
                Function {
                    name: "distance".to_string(),
                    parameters: vec![],
                    return_type: Some(Type::Named("float".to_string())),
                    body: Block {
                        statements: vec![],
                        expression: None,
                    },
                },
            ],
        };

        manager.add_impl(impl_block).unwrap();

        let methods = manager.get_methods("Point");
        assert_eq!(methods.len(), 2);
        
        let method_names: Vec<&str> = methods.iter().map(|m| m.name.as_str()).collect();
        assert!(method_names.contains(&"new"));
        assert!(method_names.contains(&"distance"));
    }

    #[test]
    fn test_tuple_struct() {
        let mut manager = TypeDefinitionManager::new();
        
        let struct_def = StructDefinition {
            name: "Color".to_string(),
            generics: vec![],
            fields: vec![
                StructField {
                    name: "0".to_string(), // Tuple field names are indices
                    field_type: Type::Named("int".to_string()),
                    visibility: Visibility::Public,
                },
                StructField {
                    name: "1".to_string(),
                    field_type: Type::Named("int".to_string()),
                    visibility: Visibility::Public,
                },
                StructField {
                    name: "2".to_string(),
                    field_type: Type::Named("int".to_string()),
                    visibility: Visibility::Public,
                },
            ],
            is_tuple: true,
            layout: create_test_memory_layout(),
        };

        let result = manager.define_struct(struct_def);
        assert!(result.is_ok());
        
        let struct_def = manager.get_struct("Color").unwrap();
        assert!(struct_def.is_tuple);
        assert_eq!(struct_def.fields.len(), 3);
    }

    #[test]
    fn test_ty_to_string() {
        assert_eq!(Ty::Int.to_string(), "int");
        assert_eq!(Ty::Float.to_string(), "float");
        assert_eq!(Ty::Bool.to_string(), "bool");
        assert_eq!(Ty::Struct("Point".to_string()).to_string(), "Point");
        assert_eq!(Ty::Enum("Color".to_string()).to_string(), "Color");
        assert_eq!(Ty::Array(Box::new(Ty::Int), Some(5)).to_string(), "[int; 5]");
        assert_eq!(Ty::Array(Box::new(Ty::Int), None).to_string(), "[int]");
        assert_eq!(Ty::Vec(Box::new(Ty::Int)).to_string(), "Vec<int>");
        assert_eq!(Ty::Reference(Box::new(Ty::Int)).to_string(), "&int");
    }

    #[test]
    fn test_ast_type_to_ty_conversion() {
        let mut manager = TypeDefinitionManager::new();
        
        // Define a struct for testing
        let struct_def = StructDefinition {
            name: "Point".to_string(),
            generics: vec![],
            fields: vec![],
            is_tuple: false,
            layout: create_test_memory_layout(),
        };
        manager.define_struct(struct_def).unwrap();

        // Test primitive type conversion
        let ast_type = Type::Named("int".to_string());
        let ty = manager.ast_type_to_ty(&ast_type).unwrap();
        assert_eq!(ty, Ty::Int);

        // Test struct type conversion
        let ast_type = Type::Named("Point".to_string());
        let ty = manager.ast_type_to_ty(&ast_type).unwrap();
        assert_eq!(ty, Ty::Struct("Point".to_string()));

        // Test array type conversion
        let ast_type = Type::Array {
            element_type: Box::new(Type::Named("int".to_string())),
            size: Some(5),
        };
        let ty = manager.ast_type_to_ty(&ast_type).unwrap();
        assert_eq!(ty, Ty::Array(Box::new(Ty::Int), Some(5)));

        // Test Vec type conversion
        let ast_type = Type::Vec {
            element_type: Box::new(Type::Named("int".to_string())),
        };
        let ty = manager.ast_type_to_ty(&ast_type).unwrap();
        assert_eq!(ty, Ty::Vec(Box::new(Ty::Int)));

        // Test reference type conversion
        let ast_type = Type::Reference {
            mutable: false,
            inner_type: Box::new(Type::Named("int".to_string())),
        };
        let ty = manager.ast_type_to_ty(&ast_type).unwrap();
        assert_eq!(ty, Ty::Reference(Box::new(Ty::Int)));
    }

    #[test]
    fn test_ast_type_to_ty_unknown_type() {
        let manager = TypeDefinitionManager::new();
        
        let ast_type = Type::Named("UnknownType".to_string());
        let result = manager.ast_type_to_ty(&ast_type);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown type"));
    }
}