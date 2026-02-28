// src/compiler/src/types_enum_test.rs
// Comprehensive tests for enum definition management

use crate::types::{TypeDefinitionManager, EnumDefinition, Ty};
use crate::ast::{EnumVariant, EnumVariantData, Type, StructField, Visibility};

#[cfg(test)]
mod enum_definition_tests {
    use super::*;

    fn create_simple_color_enum() -> EnumDefinition {
        EnumDefinition {
            name: "Color".to_string(),
            generics: vec![],
            variants: vec![
                EnumVariant {
                    name: "Red".to_string(),
                    data: None,
                },
                EnumVariant {
                    name: "Green".to_string(),
                    data: None,
                },
                EnumVariant {
                    name: "Blue".to_string(),
                    data: None,
                },
            ],
            discriminant_type: Ty::Int,
        }
    }

    fn create_option_enum() -> EnumDefinition {
        EnumDefinition {
            name: "Option".to_string(),
            generics: vec!["T".to_string()],
            variants: vec![
                EnumVariant {
                    name: "Some".to_string(),
                    data: Some(EnumVariantData::Tuple(vec![Type::Named("T".to_string())])),
                },
                EnumVariant {
                    name: "None".to_string(),
                    data: None,
                },
            ],
            discriminant_type: Ty::Int,
        }
    }

    fn create_shape_enum() -> EnumDefinition {
        EnumDefinition {
            name: "Shape".to_string(),
            generics: vec![],
            variants: vec![
                EnumVariant {
                    name: "Circle".to_string(),
                    data: Some(EnumVariantData::Struct(vec![
                        StructField {
                            name: "radius".to_string(),
                            field_type: Type::Named("float".to_string()),
                            visibility: Visibility::Public,
                        },
                    ])),
                },
                EnumVariant {
                    name: "Rectangle".to_string(),
                    data: Some(EnumVariantData::Struct(vec![
                        StructField {
                            name: "width".to_string(),
                            field_type: Type::Named("float".to_string()),
                            visibility: Visibility::Public,
                        },
                        StructField {
                            name: "height".to_string(),
                            field_type: Type::Named("float".to_string()),
                            visibility: Visibility::Public,
                        },
                    ])),
                },
            ],
            discriminant_type: Ty::Int,
        }
    }

    #[test]
    fn test_define_simple_enum_success() {
        let mut manager = TypeDefinitionManager::new();
        let enum_def = create_simple_color_enum();

        let result = manager.define_enum(enum_def);
        assert!(result.is_ok(), "Failed to define simple enum: {:?}", result);
        
        let retrieved = manager.get_enum("Color");
        assert!(retrieved.is_some(), "Failed to retrieve defined enum");
        
        let enum_def = retrieved.unwrap();
        assert_eq!(enum_def.name, "Color");
        assert_eq!(enum_def.variants.len(), 3);
        assert_eq!(enum_def.variants[0].name, "Red");
        assert_eq!(enum_def.variants[1].name, "Green");
        assert_eq!(enum_def.variants[2].name, "Blue");
    }

    #[test]
    fn test_define_generic_enum_success() {
        let mut manager = TypeDefinitionManager::new();
        let enum_def = create_option_enum();

        let result = manager.define_enum(enum_def);
        assert!(result.is_ok(), "Failed to define generic enum: {:?}", result);
        
        let retrieved = manager.get_enum("Option");
        assert!(retrieved.is_some(), "Failed to retrieve defined generic enum");
        
        let enum_def = retrieved.unwrap();
        assert_eq!(enum_def.name, "Option");
        assert_eq!(enum_def.generics.len(), 1);
        assert_eq!(enum_def.generics[0], "T");
        assert_eq!(enum_def.variants.len(), 2);
        assert_eq!(enum_def.variants[0].name, "Some");
        assert_eq!(enum_def.variants[1].name, "None");
    }

    #[test]
    fn test_define_enum_with_struct_data() {
        let mut manager = TypeDefinitionManager::new();
        let enum_def = create_shape_enum();

        let result = manager.define_enum(enum_def);
        assert!(result.is_ok(), "Failed to define enum with struct data: {:?}", result);
        
        let retrieved = manager.get_enum("Shape");
        assert!(retrieved.is_some(), "Failed to retrieve defined enum with struct data");
        
        let enum_def = retrieved.unwrap();
        assert_eq!(enum_def.name, "Shape");
        assert_eq!(enum_def.variants.len(), 2);
        
        // Check Circle variant
        let circle_variant = &enum_def.variants[0];
        assert_eq!(circle_variant.name, "Circle");
        assert!(circle_variant.data.is_some());
        match &circle_variant.data {
            Some(EnumVariantData::Struct(fields)) => {
                assert_eq!(fields.len(), 1);
                assert_eq!(fields[0].name, "radius");
                assert_eq!(fields[0].field_type, Type::Named("float".to_string()));
            }
            _ => panic!("Expected struct data for Circle variant"),
        }
        
        // Check Rectangle variant
        let rect_variant = &enum_def.variants[1];
        assert_eq!(rect_variant.name, "Rectangle");
        assert!(rect_variant.data.is_some());
        match &rect_variant.data {
            Some(EnumVariantData::Struct(fields)) => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].name, "width");
                assert_eq!(fields[1].name, "height");
            }
            _ => panic!("Expected struct data for Rectangle variant"),
        }
    }

    #[test]
    fn test_define_enum_duplicate_error() {
        let mut manager = TypeDefinitionManager::new();
        let enum_def = create_simple_color_enum();

        // First definition should succeed
        let result = manager.define_enum(enum_def.clone());
        assert!(result.is_ok(), "First enum definition should succeed");

        // Second definition should fail
        let result = manager.define_enum(enum_def);
        assert!(result.is_err(), "Duplicate enum definition should fail");
        assert!(result.unwrap_err().contains("already defined"));
    }

    #[test]
    fn test_define_enum_conflicts_with_struct() {
        let mut manager = TypeDefinitionManager::new();
        
        // First define a struct with the same name
        use crate::types::{StructDefinition, MemoryLayout};
        let struct_def = StructDefinition {
            name: "Color".to_string(),
            generics: vec![],
            fields: vec![],
            is_tuple: false,
            layout: MemoryLayout {
                size: 0,
                alignment: 1,
                field_offsets: vec![],
            },
        };
        
        let result = manager.define_struct(struct_def);
        assert!(result.is_ok(), "Struct definition should succeed");

        // Now try to define an enum with the same name
        let enum_def = create_simple_color_enum();
        let result = manager.define_enum(enum_def);
        assert!(result.is_err(), "Enum definition should fail when struct exists");
        assert!(result.unwrap_err().contains("already defined as a struct"));
    }

    #[test]
    fn test_enum_variant_validation_unit_variant() {
        let mut manager = TypeDefinitionManager::new();
        let enum_def = create_simple_color_enum();
        manager.define_enum(enum_def).unwrap();

        // Test unit variant with no data
        let result = manager.validate_enum_variant_construction("Color", "Red", None);
        assert!(result.is_ok(), "Unit variant validation should succeed");

        // Test unit variant with data (should fail)
        let result = manager.validate_enum_variant_construction("Color", "Red", Some(&[Ty::Int]));
        assert!(result.is_err(), "Unit variant with data should fail");
        assert!(result.unwrap_err().contains("expects no data"));
    }

    #[test]
    fn test_enum_variant_validation_tuple_variant() {
        let mut manager = TypeDefinitionManager::new();
        let enum_def = create_option_enum();
        manager.define_enum(enum_def).unwrap();

        // Test Some variant with correct data
        let result = manager.validate_enum_variant_construction("Option", "Some", Some(&[Ty::Int]));
        assert!(result.is_ok(), "Some variant with data should succeed");

        // Test Some variant without data (should fail)
        let result = manager.validate_enum_variant_construction("Option", "Some", None);
        assert!(result.is_err(), "Some variant without data should fail");
        assert!(result.unwrap_err().contains("expects data"));

        // Test None variant without data
        let result = manager.validate_enum_variant_construction("Option", "None", None);
        assert!(result.is_ok(), "None variant without data should succeed");
    }

    #[test]
    fn test_enum_variant_validation_struct_variant() {
        let mut manager = TypeDefinitionManager::new();
        let enum_def = create_shape_enum();
        manager.define_enum(enum_def).unwrap();

        // Test Circle variant with correct data
        let result = manager.validate_enum_variant_construction("Shape", "Circle", Some(&[Ty::Float]));
        assert!(result.is_ok(), "Circle variant with correct data should succeed");

        // Test Rectangle variant with correct data
        let result = manager.validate_enum_variant_construction("Shape", "Rectangle", Some(&[Ty::Float, Ty::Float]));
        assert!(result.is_ok(), "Rectangle variant with correct data should succeed");

        // Test Circle variant with wrong number of fields
        let result = manager.validate_enum_variant_construction("Shape", "Circle", Some(&[Ty::Float, Ty::Float]));
        assert!(result.is_err(), "Circle variant with wrong field count should fail");

        // Test Rectangle variant with wrong types
        let result = manager.validate_enum_variant_construction("Shape", "Rectangle", Some(&[Ty::Int, Ty::Int]));
        assert!(result.is_err(), "Rectangle variant with wrong types should fail");
    }

    #[test]
    fn test_get_variant_discriminant() {
        let mut manager = TypeDefinitionManager::new();
        let enum_def = create_simple_color_enum();
        manager.define_enum(enum_def).unwrap();

        // Test discriminant values
        let red_discriminant = manager.get_variant_discriminant("Color", "Red");
        assert!(red_discriminant.is_ok());
        assert_eq!(red_discriminant.unwrap(), 0);

        let green_discriminant = manager.get_variant_discriminant("Color", "Green");
        assert!(green_discriminant.is_ok());
        assert_eq!(green_discriminant.unwrap(), 1);

        let blue_discriminant = manager.get_variant_discriminant("Color", "Blue");
        assert!(blue_discriminant.is_ok());
        assert_eq!(blue_discriminant.unwrap(), 2);

        // Test non-existent variant
        let result = manager.get_variant_discriminant("Color", "Yellow");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_get_enum_variants() {
        let mut manager = TypeDefinitionManager::new();
        let enum_def = create_simple_color_enum();
        manager.define_enum(enum_def).unwrap();

        let variants = manager.get_enum_variants("Color");
        assert!(variants.is_ok());
        
        let variants = variants.unwrap();
        assert_eq!(variants.len(), 3);
        assert_eq!(variants[0].name, "Red");
        assert_eq!(variants[1].name, "Green");
        assert_eq!(variants[2].name, "Blue");

        // Test non-existent enum
        let result = manager.get_enum_variants("NonExistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Undefined enum type"));
    }

    #[test]
    fn test_variant_has_data() {
        let mut manager = TypeDefinitionManager::new();
        let enum_def = create_option_enum();
        manager.define_enum(enum_def).unwrap();

        // Test Some variant (has data)
        let result = manager.variant_has_data("Option", "Some");
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Test None variant (no data)
        let result = manager.variant_has_data("Option", "None");
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // Test non-existent variant
        let result = manager.variant_has_data("Option", "Maybe");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_get_variant_data_types() {
        let mut manager = TypeDefinitionManager::new();
        let enum_def = create_shape_enum();
        manager.define_enum(enum_def).unwrap();

        // Test Circle variant data types
        let result = manager.get_variant_data_types("Shape", "Circle");
        assert!(result.is_ok());
        let data_types = result.unwrap();
        assert!(data_types.is_some());
        let types = data_types.unwrap();
        assert_eq!(types.len(), 1);
        assert_eq!(types[0], Ty::Float);

        // Test Rectangle variant data types
        let result = manager.get_variant_data_types("Shape", "Rectangle");
        assert!(result.is_ok());
        let data_types = result.unwrap();
        assert!(data_types.is_some());
        let types = data_types.unwrap();
        assert_eq!(types.len(), 2);
        assert_eq!(types[0], Ty::Float);
        assert_eq!(types[1], Ty::Float);

        // Test unit variant (Color enum)
        let color_enum = create_simple_color_enum();
        manager.define_enum(color_enum).unwrap();
        
        let result = manager.get_variant_data_types("Color", "Red");
        assert!(result.is_ok());
        let data_types = result.unwrap();
        assert!(data_types.is_none());
    }

    #[test]
    fn test_enum_validation_duplicate_variants() {
        let mut manager = TypeDefinitionManager::new();
        
        let enum_def = EnumDefinition {
            name: "BadEnum".to_string(),
            generics: vec![],
            variants: vec![
                EnumVariant {
                    name: "Variant1".to_string(),
                    data: None,
                },
                EnumVariant {
                    name: "Variant1".to_string(), // Duplicate name
                    data: None,
                },
            ],
            discriminant_type: Ty::Int,
        };

        let result = manager.define_enum(enum_def);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate variant"));
    }

    #[test]
    fn test_enum_validation_empty_variants() {
        let mut manager = TypeDefinitionManager::new();
        
        let enum_def = EnumDefinition {
            name: "EmptyEnum".to_string(),
            generics: vec![],
            variants: vec![], // No variants
            discriminant_type: Ty::Int,
        };

        let result = manager.define_enum(enum_def);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must have at least one variant"));
    }

    #[test]
    fn test_enum_validation_duplicate_struct_fields() {
        let mut manager = TypeDefinitionManager::new();
        
        let enum_def = EnumDefinition {
            name: "BadStructEnum".to_string(),
            generics: vec![],
            variants: vec![
                EnumVariant {
                    name: "BadVariant".to_string(),
                    data: Some(EnumVariantData::Struct(vec![
                        StructField {
                            name: "field1".to_string(),
                            field_type: Type::Named("int".to_string()),
                            visibility: Visibility::Public,
                        },
                        StructField {
                            name: "field1".to_string(), // Duplicate field name
                            field_type: Type::Named("float".to_string()),
                            visibility: Visibility::Public,
                        },
                    ])),
                },
            ],
            discriminant_type: Ty::Int,
        };

        let result = manager.define_enum(enum_def);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate field"));
    }

    #[test]
    fn test_enum_validation_invalid_field_types() {
        let mut manager = TypeDefinitionManager::new();
        
        let enum_def = EnumDefinition {
            name: "InvalidTypeEnum".to_string(),
            generics: vec![],
            variants: vec![
                EnumVariant {
                    name: "BadVariant".to_string(),
                    data: Some(EnumVariantData::Tuple(vec![
                        Type::Named("NonExistentType".to_string()), // Invalid type
                    ])),
                },
            ],
            discriminant_type: Ty::Int,
        };

        let result = manager.define_enum(enum_def);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Undefined type"));
    }

    #[test]
    fn test_enum_variant_construction_undefined_enum() {
        let manager = TypeDefinitionManager::new();
        
        let result = manager.validate_enum_variant_construction("NonExistent", "Variant", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Undefined enum type"));
    }

    #[test]
    fn test_enum_variant_construction_undefined_variant() {
        let mut manager = TypeDefinitionManager::new();
        let enum_def = create_simple_color_enum();
        manager.define_enum(enum_def).unwrap();

        let result = manager.validate_enum_variant_construction("Color", "Purple", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found in enum"));
    }

    #[test]
    fn test_enum_variant_type_mismatch() {
        let mut manager = TypeDefinitionManager::new();
        let enum_def = create_option_enum();
        manager.define_enum(enum_def).unwrap();

        // Try to construct Some with wrong type (expecting generic T, providing specific type)
        // This test assumes we're working with a concrete instantiation
        let result = manager.validate_enum_variant_construction("Option", "Some", Some(&[Ty::Bool]));
        // This should succeed since we can't validate generic constraints without instantiation
        assert!(result.is_ok());
    }
}