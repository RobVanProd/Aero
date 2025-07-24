// src/compiler/src/generic_resolver_test.rs

#[cfg(test)]
mod tests {
    use super::super::generic_resolver::{GenericResolver, GenericInstance, GenericDefinition, ConcreteDefinition, GenericConstraint};
    use super::super::ast::{Type, StructField, Visibility, EnumVariant, EnumVariantData, Function, Parameter, Block};
    use super::super::types::{StructDefinition, EnumDefinition};

    #[test]
    fn test_generic_resolver_new() {
        let resolver = GenericResolver::new();
        assert!(resolver.get_instantiations("NonExistent").is_empty());
    }

    #[test]
    fn test_register_generic_struct() {
        let mut resolver = GenericResolver::new();
        
        let fields = vec![
            StructField {
                name: "value".to_string(),
                field_type: Type::Named("T".to_string()),
                visibility: Visibility::Public,
            },
        ];
        
        let result = resolver.register_generic_struct(
            "Container".to_string(),
            vec!["T".to_string()],
            fields,
            false
        );
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_generic_struct_duplicate() {
        let mut resolver = GenericResolver::new();
        
        let fields = vec![
            StructField {
                name: "value".to_string(),
                field_type: Type::Named("T".to_string()),
                visibility: Visibility::Public,
            },
        ];
        
        // First registration should succeed
        let result1 = resolver.register_generic_struct(
            "Container".to_string(),
            vec!["T".to_string()],
            fields.clone(),
            false
        );
        assert!(result1.is_ok());
        
        // Second registration should fail
        let result2 = resolver.register_generic_struct(
            "Container".to_string(),
            vec!["T".to_string()],
            fields,
            false
        );
        assert!(result2.is_err());
        assert!(result2.unwrap_err().contains("already exists"));
    }

    #[test]
    fn test_register_generic_enum() {
        let mut resolver = GenericResolver::new();
        
        let variants = vec![
            EnumVariant {
                name: "Some".to_string(),
                data: Some(EnumVariantData::Tuple(vec![Type::Named("T".to_string())])),
            },
            EnumVariant {
                name: "None".to_string(),
                data: None,
            },
        ];
        
        let result = resolver.register_generic_enum(
            "Option".to_string(),
            vec!["T".to_string()],
            variants
        );
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_generic_function() {
        let mut resolver = GenericResolver::new();
        
        let function = Function {
            name: "identity".to_string(),
            parameters: vec![
                Parameter {
                    name: "x".to_string(),
                    param_type: Type::Named("T".to_string()),
                },
            ],
            return_type: Some(Type::Named("T".to_string())),
            body: Block {
                statements: vec![],
                expression: None,
            },
        };
        
        let result = resolver.register_generic_function(
            "identity".to_string(),
            vec!["T".to_string()],
            function
        );
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_instantiate_generic_struct() {
        let mut resolver = GenericResolver::new();
        
        // Register a generic struct
        let fields = vec![
            StructField {
                name: "value".to_string(),
                field_type: Type::Named("T".to_string()),
                visibility: Visibility::Public,
            },
        ];
        
        resolver.register_generic_struct(
            "Container".to_string(),
            vec!["T".to_string()],
            fields,
            false
        ).unwrap();
        
        // Instantiate with i32
        let type_args = vec![Type::Named("i32".to_string())];
        let result = resolver.instantiate_generic("Container", &type_args);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Container_i32");
    }

    #[test]
    fn test_instantiate_generic_multiple_args() {
        let mut resolver = GenericResolver::new();
        
        // Register a generic struct with multiple type parameters
        let fields = vec![
            StructField {
                name: "first".to_string(),
                field_type: Type::Named("T".to_string()),
                visibility: Visibility::Public,
            },
            StructField {
                name: "second".to_string(),
                field_type: Type::Named("U".to_string()),
                visibility: Visibility::Public,
            },
        ];
        
        resolver.register_generic_struct(
            "Pair".to_string(),
            vec!["T".to_string(), "U".to_string()],
            fields,
            false
        ).unwrap();
        
        // Instantiate with i32 and String
        let type_args = vec![
            Type::Named("i32".to_string()),
            Type::Named("String".to_string())
        ];
        let result = resolver.instantiate_generic("Pair", &type_args);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Pair_i32_String");
    }

    #[test]
    fn test_instantiate_generic_wrong_arg_count() {
        let mut resolver = GenericResolver::new();
        
        // Register a generic struct with one type parameter
        let fields = vec![
            StructField {
                name: "value".to_string(),
                field_type: Type::Named("T".to_string()),
                visibility: Visibility::Public,
            },
        ];
        
        resolver.register_generic_struct(
            "Container".to_string(),
            vec!["T".to_string()],
            fields,
            false
        ).unwrap();
        
        // Try to instantiate with wrong number of arguments
        let type_args = vec![
            Type::Named("i32".to_string()),
            Type::Named("String".to_string())
        ];
        let result = resolver.instantiate_generic("Container", &type_args);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("expects 1 type arguments, but 2 were provided"));
    }

    #[test]
    fn test_instantiate_undefined_generic() {
        let mut resolver = GenericResolver::new();
        
        let type_args = vec![Type::Named("i32".to_string())];
        let result = resolver.instantiate_generic("UndefinedType", &type_args);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_instantiation_caching() {
        let mut resolver = GenericResolver::new();
        
        // Register a generic struct
        let fields = vec![
            StructField {
                name: "value".to_string(),
                field_type: Type::Named("T".to_string()),
                visibility: Visibility::Public,
            },
        ];
        
        resolver.register_generic_struct(
            "Container".to_string(),
            vec!["T".to_string()],
            fields,
            false
        ).unwrap();
        
        let type_args = vec![Type::Named("i32".to_string())];
        
        // First instantiation
        let result1 = resolver.instantiate_generic("Container", &type_args);
        assert!(result1.is_ok());
        
        // Second instantiation should return cached result
        let result2 = resolver.instantiate_generic("Container", &type_args);
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap(), result2.unwrap());
        
        // Check that it's actually cached
        assert!(resolver.is_instantiated("Container", &type_args));
        
        let cached_name = resolver.get_instantiated_name("Container", &type_args);
        assert!(cached_name.is_some());
        assert_eq!(cached_name.unwrap(), "Container_i32");
    }

    #[test]
    fn test_get_instantiations() {
        let mut resolver = GenericResolver::new();
        
        // Register a generic struct
        let fields = vec![
            StructField {
                name: "value".to_string(),
                field_type: Type::Named("T".to_string()),
                visibility: Visibility::Public,
            },
        ];
        
        resolver.register_generic_struct(
            "Container".to_string(),
            vec!["T".to_string()],
            fields,
            false
        ).unwrap();
        
        // Initially no instantiations
        assert!(resolver.get_instantiations("Container").is_empty());
        
        // Create some instantiations
        let type_args1 = vec![Type::Named("i32".to_string())];
        let type_args2 = vec![Type::Named("String".to_string())];
        
        resolver.instantiate_generic("Container", &type_args1).unwrap();
        resolver.instantiate_generic("Container", &type_args2).unwrap();
        
        // Should have two instantiations
        let instantiations = resolver.get_instantiations("Container");
        assert_eq!(instantiations.len(), 2);
    }

    #[test]
    fn test_resolve_generic_method() {
        let resolver = GenericResolver::new();
        
        // Test non-generic method resolution
        let type_args = vec![Type::Named("i32".to_string())];
        let result = resolver.resolve_generic_method("Container", "get", &type_args);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Container::get");
    }

    #[test]
    fn test_monomorphize_struct() {
        let mut resolver = GenericResolver::new();
        
        // Register a generic struct
        let fields = vec![
            StructField {
                name: "value".to_string(),
                field_type: Type::Named("T".to_string()),
                visibility: Visibility::Public,
            },
        ];
        
        resolver.register_generic_struct(
            "Container".to_string(),
            vec!["T".to_string()],
            fields,
            false
        ).unwrap();
        
        // Monomorphize with i32
        let type_args = vec![Type::Named("i32".to_string())];
        let result = resolver.monomorphize("Container", &type_args);
        
        assert!(result.is_ok());
        
        match result.unwrap() {
            ConcreteDefinition::Struct(struct_def) => {
                assert_eq!(struct_def.name, "Container_i32");
                assert_eq!(struct_def.generics.len(), 0);
                assert_eq!(struct_def.fields.len(), 1);
                assert_eq!(struct_def.fields[0].field_type, Type::Named("i32".to_string()));
            }
            _ => panic!("Expected concrete struct definition"),
        }
    }

    #[test]
    fn test_monomorphize_enum() {
        let mut resolver = GenericResolver::new();
        
        // Register a generic enum
        let variants = vec![
            EnumVariant {
                name: "Some".to_string(),
                data: Some(EnumVariantData::Tuple(vec![Type::Named("T".to_string())])),
            },
            EnumVariant {
                name: "None".to_string(),
                data: None,
            },
        ];
        
        resolver.register_generic_enum(
            "Option".to_string(),
            vec!["T".to_string()],
            variants
        ).unwrap();
        
        // Monomorphize with String
        let type_args = vec![Type::Named("String".to_string())];
        let result = resolver.monomorphize("Option", &type_args);
        
        assert!(result.is_ok());
        
        match result.unwrap() {
            ConcreteDefinition::Enum(enum_def) => {
                assert_eq!(enum_def.name, "Option_String");
                assert_eq!(enum_def.generics.len(), 0);
                assert_eq!(enum_def.variants.len(), 2);
                
                // Check Some variant
                match &enum_def.variants[0].data {
                    Some(EnumVariantData::Tuple(types)) => {
                        assert_eq!(types.len(), 1);
                        assert_eq!(types[0], Type::Named("String".to_string()));
                    }
                    _ => panic!("Expected tuple variant data"),
                }
            }
            _ => panic!("Expected concrete enum definition"),
        }
    }

    #[test]
    fn test_add_constraint() {
        let mut resolver = GenericResolver::new();
        
        let constraint = GenericConstraint {
            type_param: "T".to_string(),
            trait_bounds: vec!["Display".to_string()],
        };
        
        resolver.add_constraint("Container".to_string(), constraint);
        
        // The constraint should be stored (we can't easily test this without exposing internals)
        // But we can test that instantiation still works with constraints
        let fields = vec![
            StructField {
                name: "value".to_string(),
                field_type: Type::Named("T".to_string()),
                visibility: Visibility::Public,
            },
        ];
        
        resolver.register_generic_struct(
            "Container".to_string(),
            vec!["T".to_string()],
            fields,
            false
        ).unwrap();
        
        let type_args = vec![Type::Named("i32".to_string())];
        let result = resolver.instantiate_generic("Container", &type_args);
        
        // Should work with current placeholder constraint validation
        assert!(result.is_ok());
    }

    #[test]
    fn test_clear_instantiations() {
        let mut resolver = GenericResolver::new();
        
        // Register and instantiate a generic struct
        let fields = vec![
            StructField {
                name: "value".to_string(),
                field_type: Type::Named("T".to_string()),
                visibility: Visibility::Public,
            },
        ];
        
        resolver.register_generic_struct(
            "Container".to_string(),
            vec!["T".to_string()],
            fields,
            false
        ).unwrap();
        
        let type_args = vec![Type::Named("i32".to_string())];
        resolver.instantiate_generic("Container", &type_args).unwrap();
        
        // Should have instantiations
        assert!(!resolver.get_instantiations("Container").is_empty());
        
        // Clear instantiations
        resolver.clear_instantiations();
        
        // Should be empty now
        assert!(resolver.get_instantiations("Container").is_empty());
    }

    #[test]
    fn test_type_to_string_conversion() {
        let resolver = GenericResolver::new();
        
        // Test various type conversions (we can't directly test the private method,
        // but we can test it indirectly through instantiation names)
        let mut test_resolver = GenericResolver::new();
        
        let fields = vec![
            StructField {
                name: "value".to_string(),
                field_type: Type::Named("T".to_string()),
                visibility: Visibility::Public,
            },
        ];
        
        test_resolver.register_generic_struct(
            "Container".to_string(),
            vec!["T".to_string()],
            fields,
            false
        ).unwrap();
        
        // Test with different types
        let test_cases = vec![
            (vec![Type::Named("i32".to_string())], "Container_i32"),
            (vec![Type::Named("String".to_string())], "Container_String"),
            (vec![Type::Vec { element_type: Box::new(Type::Named("i32".to_string())) }], "Container_Vec_i32"),
        ];
        
        for (type_args, expected_name) in test_cases {
            let result = test_resolver.instantiate_generic("Container", &type_args);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), expected_name);
        }
    }

    #[test]
    fn test_nested_generic_types() {
        let mut resolver = GenericResolver::new();
        
        // Register a generic struct with nested generic field
        let fields = vec![
            StructField {
                name: "items".to_string(),
                field_type: Type::Vec { 
                    element_type: Box::new(Type::Named("T".to_string())) 
                },
                visibility: Visibility::Public,
            },
        ];
        
        resolver.register_generic_struct(
            "Container".to_string(),
            vec!["T".to_string()],
            fields,
            false
        ).unwrap();
        
        // Instantiate with i32
        let type_args = vec![Type::Named("i32".to_string())];
        let result = resolver.instantiate_generic("Container", &type_args);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Container_i32");
        
        // Test monomorphization to ensure nested types are substituted
        let concrete = resolver.monomorphize("Container", &type_args);
        assert!(concrete.is_ok());
        
        match concrete.unwrap() {
            ConcreteDefinition::Struct(struct_def) => {
                assert_eq!(struct_def.fields[0].field_type, Type::Vec { 
                    element_type: Box::new(Type::Named("i32".to_string())) 
                });
            }
            _ => panic!("Expected concrete struct definition"),
        }
    }

    #[test]
    fn test_resolve_generic_method_with_constraints() {
        let mut resolver = GenericResolver::new();
        
        // Register a generic method with constraints
        let function = Function {
            name: "display".to_string(),
            parameters: vec![
                Parameter {
                    name: "self".to_string(),
                    param_type: Type::Reference {
                        mutable: false,
                        inner_type: Box::new(Type::Named("Self".to_string())),
                    },
                },
                Parameter {
                    name: "value".to_string(),
                    param_type: Type::Named("T".to_string()),
                },
            ],
            return_type: None,
            body: Block {
                statements: vec![],
                expression: None,
            },
        };
        
        resolver.register_generic_function(
            "Container::display".to_string(),
            vec!["T".to_string()],
            function
        ).unwrap();
        
        // Add constraint that T must implement Display
        let constraint = GenericConstraint {
            type_param: "T".to_string(),
            trait_bounds: vec!["Display".to_string()],
        };
        resolver.add_constraint("Container::display".to_string(), constraint);
        
        // Test with type that implements Display
        let type_args = vec![Type::Named("i32".to_string())];
        let result = resolver.resolve_generic_method("Container", "display", &type_args);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Container::display_i32");
    }

    #[test]
    fn test_resolve_generic_method_constraint_violation() {
        let mut resolver = GenericResolver::new();
        
        // Register a generic method with constraints
        let function = Function {
            name: "display".to_string(),
            parameters: vec![
                Parameter {
                    name: "value".to_string(),
                    param_type: Type::Named("T".to_string()),
                },
            ],
            return_type: None,
            body: Block {
                statements: vec![],
                expression: None,
            },
        };
        
        resolver.register_generic_function(
            "Container::display".to_string(),
            vec!["T".to_string()],
            function
        ).unwrap();
        
        // Add constraint that T must implement Display
        let constraint = GenericConstraint {
            type_param: "T".to_string(),
            trait_bounds: vec!["Display".to_string()],
        };
        resolver.add_constraint("Container::display".to_string(), constraint);
        
        // Test with type that doesn't implement Display (using a custom type)
        let type_args = vec![Type::Named("CustomType".to_string())];
        let result = resolver.resolve_generic_method("Container", "display", &type_args);
        
        // Should fail due to constraint violation
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not implement required trait"));
    }

    #[test]
    fn test_resolve_method_on_generic_type() {
        let mut resolver = GenericResolver::new();
        
        // Register a generic method on a base type
        let function = Function {
            name: "get".to_string(),
            parameters: vec![
                Parameter {
                    name: "self".to_string(),
                    param_type: Type::Reference {
                        mutable: false,
                        inner_type: Box::new(Type::Named("Self".to_string())),
                    },
                },
            ],
            return_type: Some(Type::Named("T".to_string())),
            body: Block {
                statements: vec![],
                expression: None,
            },
        };
        
        resolver.register_generic_function(
            "Container::get".to_string(),
            vec!["T".to_string()],
            function
        ).unwrap();
        
        // Test method resolution on instantiated type
        let type_args = vec![Type::Named("i32".to_string())];
        let result = resolver.resolve_generic_method("Container_i32", "get", &type_args);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Container::get_i32");
    }

    #[test]
    fn test_infer_method_generics() {
        let mut resolver = GenericResolver::new();
        
        // Register a generic method that can have its types inferred
        let function = Function {
            name: "identity".to_string(),
            parameters: vec![
                Parameter {
                    name: "value".to_string(),
                    param_type: Type::Named("T".to_string()),
                },
            ],
            return_type: Some(Type::Named("T".to_string())),
            body: Block {
                statements: vec![],
                expression: None,
            },
        };
        
        resolver.register_generic_function(
            "Utils::identity".to_string(),
            vec!["T".to_string()],
            function
        ).unwrap();
        
        // Test type inference from argument types
        let arg_types = vec![Type::Named("String".to_string())];
        let result = resolver.infer_method_generics("Utils", "identity", &arg_types);
        
        assert!(result.is_ok());
        let inferred_types = result.unwrap();
        assert_eq!(inferred_types.len(), 1);
        assert_eq!(inferred_types[0], Type::Named("String".to_string()));
    }

    #[test]
    fn test_infer_method_generics_complex() {
        let mut resolver = GenericResolver::new();
        
        // Register a generic method with complex parameter types
        let function = Function {
            name: "map".to_string(),
            parameters: vec![
                Parameter {
                    name: "vec".to_string(),
                    param_type: Type::Vec {
                        element_type: Box::new(Type::Named("T".to_string())),
                    },
                },
                Parameter {
                    name: "func".to_string(),
                    param_type: Type::Named("Function".to_string()), // Simplified function type
                },
            ],
            return_type: Some(Type::Vec {
                element_type: Box::new(Type::Named("U".to_string())),
            }),
            body: Block {
                statements: vec![],
                expression: None,
            },
        };
        
        resolver.register_generic_function(
            "Utils::map".to_string(),
            vec!["T".to_string(), "U".to_string()],
            function
        ).unwrap();
        
        // Test type inference from Vec argument
        let arg_types = vec![
            Type::Vec {
                element_type: Box::new(Type::Named("i32".to_string())),
            },
            Type::Named("Function".to_string()),
        ];
        let result = resolver.infer_method_generics("Utils", "map", &arg_types);
        
        assert!(result.is_ok());
        let inferred_types = result.unwrap();
        assert_eq!(inferred_types.len(), 2);
        assert_eq!(inferred_types[0], Type::Named("i32".to_string()));
        // Note: U cannot be inferred from the given arguments, so this would fail in practice
        // This test demonstrates the inference mechanism for T
    }

    #[test]
    fn test_infer_method_generics_conflict() {
        let mut resolver = GenericResolver::new();
        
        // Register a generic method where the same type parameter appears multiple times
        let function = Function {
            name: "combine".to_string(),
            parameters: vec![
                Parameter {
                    name: "first".to_string(),
                    param_type: Type::Named("T".to_string()),
                },
                Parameter {
                    name: "second".to_string(),
                    param_type: Type::Named("T".to_string()),
                },
            ],
            return_type: Some(Type::Named("T".to_string())),
            body: Block {
                statements: vec![],
                expression: None,
            },
        };
        
        resolver.register_generic_function(
            "Utils::combine".to_string(),
            vec!["T".to_string()],
            function
        ).unwrap();
        
        // Test with conflicting argument types
        let arg_types = vec![
            Type::Named("i32".to_string()),
            Type::Named("String".to_string()),
        ];
        let result = resolver.infer_method_generics("Utils", "combine", &arg_types);
        
        // Should fail due to type inference conflict
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Type inference conflict"));
    }

    #[test]
    fn test_resolve_associated_type() {
        let resolver = GenericResolver::new();
        
        // Test basic associated type resolution
        let result = resolver.resolve_associated_type("Iterator", "Item");
        
        // Should fail since we don't have enough context
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot resolve associated type"));
    }

    #[test]
    fn test_resolve_associated_type_unknown() {
        let resolver = GenericResolver::new();
        
        // Test with unknown associated type
        let result = resolver.resolve_associated_type("SomeType", "UnknownAssoc");
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown associated type"));
    }

    #[test]
    fn test_extract_base_type_name() {
        let resolver = GenericResolver::new();
        
        // Test extracting base type from instantiated name
        assert_eq!(resolver.extract_base_type_name("Container_i32"), Some("Container"));
        assert_eq!(resolver.extract_base_type_name("Vec_String"), Some("Vec"));
        assert_eq!(resolver.extract_base_type_name("HashMap_String_i32"), Some("HashMap"));
        assert_eq!(resolver.extract_base_type_name("SimpleType"), None);
    }

    #[test]
    fn test_type_implements_trait() {
        let resolver = GenericResolver::new();
        
        // Test basic type trait implementations
        assert!(resolver.type_implements_trait(&Type::Named("i32".to_string()), "Display"));
        assert!(resolver.type_implements_trait(&Type::Named("String".to_string()), "Clone"));
        assert!(resolver.type_implements_trait(&Type::Vec { 
            element_type: Box::new(Type::Named("i32".to_string())) 
        }, "Debug"));
        
        // Test types that don't implement traits
        assert!(!resolver.type_implements_trait(&Type::Named("CustomType".to_string()), "Display"));
        assert!(!resolver.type_implements_trait(&Type::Named("i32".to_string()), "UnknownTrait"));
    }

    #[test]
    fn test_validate_trait_bounds() {
        let resolver = GenericResolver::new();
        
        // Test successful trait bound validation
        let concrete_type = Type::Named("i32".to_string());
        let trait_bounds = vec!["Display".to_string(), "Clone".to_string()];
        let result = resolver.validate_trait_bounds(&concrete_type, &trait_bounds);
        
        assert!(result.is_ok());
        
        // Test failed trait bound validation
        let concrete_type = Type::Named("CustomType".to_string());
        let trait_bounds = vec!["Display".to_string()];
        let result = resolver.validate_trait_bounds(&concrete_type, &trait_bounds);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not implement required trait"));
    }

    #[test]
    fn test_infer_from_parameters_mismatch() {
        let resolver = GenericResolver::new();
        
        let generics = vec!["T".to_string()];
        let params = vec![
            Parameter {
                name: "value".to_string(),
                param_type: Type::Named("T".to_string()),
            },
        ];
        let arg_types = vec![
            Type::Named("i32".to_string()),
            Type::Named("String".to_string()), // Extra argument
        ];
        
        let result = resolver.infer_from_parameters(&generics, &params, &arg_types);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Parameter count mismatch"));
    }

    #[test]
    fn test_infer_from_parameters_incomplete() {
        let resolver = GenericResolver::new();
        
        let generics = vec!["T".to_string(), "U".to_string()];
        let params = vec![
            Parameter {
                name: "value".to_string(),
                param_type: Type::Named("T".to_string()),
            },
        ];
        let arg_types = vec![Type::Named("i32".to_string())];
        
        let result = resolver.infer_from_parameters(&generics, &params, &arg_types);
        
        // Should fail because U cannot be inferred
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Could not infer type for generic parameter 'U'"));
    }
}