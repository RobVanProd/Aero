// src/compiler/src/ir_generator_struct_test.rs

#[cfg(test)]
mod tests {
    use super::super::ir_generator::{IrGenerator, StructDefinition};
    use super::super::ast::{AstNode, Statement, Expression, StructField, Type, Visibility};
    use super::super::ir::{Inst, Value, Function};
    use super::super::types::Ty;
    use std::collections::HashMap;

    #[test]
    fn test_struct_definition_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a simple struct: struct Point { x: i32, y: i32 }
        let fields = vec![
            StructField {
                name: "x".to_string(),
                field_type: Type::Named("i32".to_string()),
                visibility: Visibility::Public,
            },
            StructField {
                name: "y".to_string(),
                field_type: Type::Named("i32".to_string()),
                visibility: Visibility::Public,
            },
        ];
        
        let struct_stmt = Statement::Struct {
            name: "Point".to_string(),
            generics: vec![],
            fields,
            is_tuple: false,
        };
        
        let ast = vec![AstNode::Statement(struct_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        // Check that main function exists
        assert!(ir.contains_key("main"));
        let main_func = &ir["main"];
        
        // Check that main function contains a StructDef
        assert!(!main_func.body.is_empty());
        
        // Check that the first instruction is a StructDef
        match &main_func.body[0] {
            Inst::StructDef { name, fields, is_tuple } => {
                assert_eq!(name, "Point");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "x");
                assert_eq!(fields[0].1, "i32");
                assert_eq!(fields[1].0, "y");
                assert_eq!(fields[1].1, "i32");
                assert_eq!(*is_tuple, false);
            }
            _ => panic!("Expected StructDef instruction, got: {:?}", main_func.body[0]),
        }
    }

    #[test]
    fn test_tuple_struct_definition_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a tuple struct: struct Color(u8, u8, u8)
        let fields = vec![
            StructField {
                name: "0".to_string(), // Tuple fields use indices as names
                field_type: Type::Named("u8".to_string()),
                visibility: Visibility::Public,
            },
            StructField {
                name: "1".to_string(),
                field_type: Type::Named("u8".to_string()),
                visibility: Visibility::Public,
            },
            StructField {
                name: "2".to_string(),
                field_type: Type::Named("u8".to_string()),
                visibility: Visibility::Public,
            },
        ];
        
        let struct_stmt = Statement::Struct {
            name: "Color".to_string(),
            generics: vec![],
            fields,
            is_tuple: true,
        };
        
        let ast = vec![AstNode::Statement(struct_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        let main_func = &ir["main"];
        
        // Check that the instruction is a tuple StructDef
        match &main_func.body[0] {
            Inst::StructDef { name, fields, is_tuple } => {
                assert_eq!(name, "Color");
                assert_eq!(fields.len(), 3);
                assert_eq!(fields[0].0, "0");
                assert_eq!(fields[1].0, "1");
                assert_eq!(fields[2].0, "2");
                assert_eq!(*is_tuple, true);
            }
            _ => panic!("Expected StructDef instruction"),
        }
    }

    #[test]
    fn test_struct_literal_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // First define the struct
        let fields = vec![
            StructField {
                name: "x".to_string(),
                field_type: Type::Named("i32".to_string()),
                visibility: Visibility::Public,
            },
            StructField {
                name: "y".to_string(),
                field_type: Type::Named("i32".to_string()),
                visibility: Visibility::Public,
            },
        ];
        
        let struct_def = StructDefinition {
            name: "Point".to_string(),
            fields: vec![("x".to_string(), "i32".to_string()), ("y".to_string(), "i32".to_string())],
            field_indices: {
                let mut map = HashMap::new();
                map.insert("x".to_string(), 0);
                map.insert("y".to_string(), 1);
                map
            },
            is_tuple: false,
        };
        ir_gen.struct_definitions.insert("Point".to_string(), struct_def);
        
        // Create a struct literal: Point { x: 10, y: 20 }
        let struct_literal = Expression::StructLiteral {
            name: "Point".to_string(),
            fields: vec![
                ("x".to_string(), Expression::IntegerLiteral(10)),
                ("y".to_string(), Expression::IntegerLiteral(20)),
            ],
            base: None,
        };
        
        let mut function = Function {
            name: "test".to_string(),
            body: Vec::new(),
            next_reg: 0,
            next_ptr: 0,
        };
        
        let (result_val, result_type) = ir_gen.generate_expression_ir(struct_literal, &mut function);
        
        // Should return struct pointer and struct type
        assert!(matches!(result_val, Value::Reg(_)));
        assert_eq!(result_type, Ty::Struct("Point".to_string()));
        
        // Check that the function body contains the expected instructions
        assert_eq!(function.body.len(), 2);
        
        // First instruction should be StructAlloca
        match &function.body[0] {
            Inst::StructAlloca { result, struct_name } => {
                assert!(matches!(result, Value::Reg(_)));
                assert_eq!(struct_name, "Point");
            }
            _ => panic!("Expected StructAlloca instruction, got: {:?}", function.body[0]),
        }
        
        // Second instruction should be StructInit
        match &function.body[1] {
            Inst::StructInit { result, struct_name, field_values } => {
                assert!(matches!(result, Value::Reg(_)));
                assert_eq!(struct_name, "Point");
                assert_eq!(field_values.len(), 2);
                assert_eq!(field_values[0].0, "x");
                assert_eq!(field_values[0].1, Value::ImmInt(10));
                assert_eq!(field_values[1].0, "y");
                assert_eq!(field_values[1].1, Value::ImmInt(20));
            }
            _ => panic!("Expected StructInit instruction, got: {:?}", function.body[1]),
        }
    }

    #[test]
    fn test_field_access_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Set up struct definition
        let struct_def = StructDefinition {
            name: "Point".to_string(),
            fields: vec![("x".to_string(), "i32".to_string()), ("y".to_string(), "i32".to_string())],
            field_indices: {
                let mut map = HashMap::new();
                map.insert("x".to_string(), 0);
                map.insert("y".to_string(), 1);
                map
            },
            is_tuple: false,
        };
        ir_gen.struct_definitions.insert("Point".to_string(), struct_def);
        
        // Set up symbol table with a Point variable
        ir_gen.symbol_table.insert("point".to_string(), (Value::Reg(0), Ty::Struct("Point".to_string())));
        
        // Create field access: point.x
        let field_access = Expression::FieldAccess {
            object: Box::new(Expression::Identifier("point".to_string())),
            field: "x".to_string(),
        };
        
        let mut function = Function {
            name: "test".to_string(),
            body: Vec::new(),
            next_reg: 1,
            next_ptr: 1,
        };
        
        let (result_val, result_type) = ir_gen.generate_expression_ir(field_access, &mut function);
        
        // Should return register and int type
        assert!(matches!(result_val, Value::Reg(_)));
        assert_eq!(result_type, Ty::Int);
        
        // Check that the function body contains the expected instructions
        assert_eq!(function.body.len(), 2);
        
        // First instruction should be Load (for the identifier)
        match &function.body[0] {
            Inst::Load(result_reg, ptr_reg) => {
                assert!(matches!(result_reg, Value::Reg(_)));
                assert_eq!(*ptr_reg, Value::Reg(0));
            }
            _ => panic!("Expected Load instruction, got: {:?}", function.body[0]),
        }
        
        // Second instruction should be FieldAccess
        match &function.body[1] {
            Inst::FieldAccess { result, struct_ptr, field_name, field_index } => {
                assert!(matches!(result, Value::Reg(_)));
                assert!(matches!(struct_ptr, Value::Reg(_)));
                assert_eq!(field_name, "x");
                assert_eq!(*field_index, 0);
            }
            _ => panic!("Expected FieldAccess instruction, got: {:?}", function.body[1]),
        }
    }

    #[test]
    fn test_method_call_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Set up struct definition
        let struct_def = StructDefinition {
            name: "Point".to_string(),
            fields: vec![("x".to_string(), "i32".to_string()), ("y".to_string(), "i32".to_string())],
            field_indices: {
                let mut map = HashMap::new();
                map.insert("x".to_string(), 0);
                map.insert("y".to_string(), 1);
                map
            },
            is_tuple: false,
        };
        ir_gen.struct_definitions.insert("Point".to_string(), struct_def);
        
        // Set up symbol table with a Point variable
        ir_gen.symbol_table.insert("point".to_string(), (Value::Reg(0), Ty::Struct("Point".to_string())));
        
        // Create method call: point.distance(other_point)
        let method_call = Expression::MethodCall {
            object: Box::new(Expression::Identifier("point".to_string())),
            method: "distance".to_string(),
            arguments: vec![Expression::Identifier("other_point".to_string())],
        };
        
        // Set up other_point in symbol table
        ir_gen.symbol_table.insert("other_point".to_string(), (Value::Reg(1), Ty::Struct("Point".to_string())));
        
        let mut function = Function {
            name: "test".to_string(),
            body: Vec::new(),
            next_reg: 2,
            next_ptr: 2,
        };
        
        let (result_val, result_type) = ir_gen.generate_expression_ir(method_call, &mut function);
        
        // Should return register and struct type (method returns same type as object for now)
        assert!(matches!(result_val, Value::Reg(_)));
        assert_eq!(result_type, Ty::Struct("Point".to_string()));
        
        // Check that the function body contains the expected instructions
        assert_eq!(function.body.len(), 3);
        
        // First instruction should be Load (for the object identifier)
        match &function.body[0] {
            Inst::Load(result_reg, ptr_reg) => {
                assert!(matches!(result_reg, Value::Reg(_)));
                assert_eq!(*ptr_reg, Value::Reg(0));
            }
            _ => panic!("Expected Load instruction, got: {:?}", function.body[0]),
        }
        
        // Second instruction should be Load (for the argument)
        match &function.body[1] {
            Inst::Load(result_reg, ptr_reg) => {
                assert!(matches!(result_reg, Value::Reg(_)));
                assert_eq!(*ptr_reg, Value::Reg(1));
            }
            _ => panic!("Expected Load instruction, got: {:?}", function.body[1]),
        }
        
        // Third instruction should be Call (for the method)
        match &function.body[2] {
            Inst::Call { function: func_name, arguments, result } => {
                assert_eq!(func_name, "Point::distance");
                assert_eq!(arguments.len(), 2); // object + argument
                assert!(result.is_some());
            }
            _ => panic!("Expected Call instruction, got: {:?}", function.body[2]),
        }
    }

    #[test]
    fn test_struct_copy_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        let source_value = Value::Reg(0);
        let struct_name = "Point".to_string();
        
        let mut function = Function {
            name: "test".to_string(),
            body: Vec::new(),
            next_reg: 1,
            next_ptr: 1,
        };
        
        let result_val = ir_gen.generate_struct_copy_ir(source_value.clone(), struct_name.clone(), &mut function);
        
        // Should return a new register
        assert!(matches!(result_val, Value::Reg(_)));
        
        // Check that the function body contains StructCopy instruction
        assert_eq!(function.body.len(), 1);
        
        match &function.body[0] {
            Inst::StructCopy { result, source, struct_name: name } => {
                assert!(matches!(result, Value::Reg(_)));
                assert_eq!(*source, source_value);
                assert_eq!(name, &struct_name);
            }
            _ => panic!("Expected StructCopy instruction, got: {:?}", function.body[0]),
        }
    }

    #[test]
    fn test_field_type_resolution() {
        let ir_gen = IrGenerator::new();
        
        // Create a struct definition with different field types
        let struct_def = StructDefinition {
            name: "TestStruct".to_string(),
            fields: vec![
                ("int_field".to_string(), "i32".to_string()),
                ("float_field".to_string(), "f64".to_string()),
                ("bool_field".to_string(), "bool".to_string()),
                ("string_field".to_string(), "String".to_string()),
            ],
            field_indices: {
                let mut map = HashMap::new();
                map.insert("int_field".to_string(), 0);
                map.insert("float_field".to_string(), 1);
                map.insert("bool_field".to_string(), 2);
                map.insert("string_field".to_string(), 3);
                map
            },
            is_tuple: false,
        };
        
        // Test field type resolution
        assert_eq!(ir_gen.get_field_type(&struct_def, "int_field"), Ty::Int);
        assert_eq!(ir_gen.get_field_type(&struct_def, "float_field"), Ty::Float);
        assert_eq!(ir_gen.get_field_type(&struct_def, "bool_field"), Ty::Bool);
        assert_eq!(ir_gen.get_field_type(&struct_def, "string_field"), Ty::String);
    }

    #[test]
    fn test_string_to_ty_conversion() {
        let ir_gen = IrGenerator::new();
        
        // Test basic type conversions
        assert_eq!(ir_gen.string_to_ty("i32"), Ty::Int);
        assert_eq!(ir_gen.string_to_ty("int"), Ty::Int);
        assert_eq!(ir_gen.string_to_ty("f64"), Ty::Float);
        assert_eq!(ir_gen.string_to_ty("float"), Ty::Float);
        assert_eq!(ir_gen.string_to_ty("bool"), Ty::Bool);
        assert_eq!(ir_gen.string_to_ty("String"), Ty::String);
        
        // Test Vec type conversion
        assert_eq!(ir_gen.string_to_ty("Vec<i32>"), Ty::Vec(Box::new(Ty::Int)));
        assert_eq!(ir_gen.string_to_ty("Vec<f64>"), Ty::Vec(Box::new(Ty::Float)));
        
        // Test array type conversion
        assert_eq!(ir_gen.string_to_ty("[i32; 5]"), Ty::Array(Box::new(Ty::Int), Some(5)));
        assert_eq!(ir_gen.string_to_ty("[f64]"), Ty::Array(Box::new(Ty::Float), None));
        
        // Test reference type conversion
        assert_eq!(ir_gen.string_to_ty("&i32"), Ty::Reference(Box::new(Ty::Int)));
        assert_eq!(ir_gen.string_to_ty("&String"), Ty::Reference(Box::new(Ty::String)));
    }

    #[test]
    fn test_complex_struct_with_nested_types() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a struct with complex field types
        let fields = vec![
            StructField {
                name: "id".to_string(),
                field_type: Type::Named("i32".to_string()),
                visibility: Visibility::Public,
            },
            StructField {
                name: "values".to_string(),
                field_type: Type::Vec {
                    element_type: Box::new(Type::Named("f64".to_string())),
                },
                visibility: Visibility::Public,
            },
            StructField {
                name: "data".to_string(),
                field_type: Type::Array {
                    element_type: Box::new(Type::Named("i32".to_string())),
                    size: Some(10),
                },
                visibility: Visibility::Public,
            },
        ];
        
        let struct_stmt = Statement::Struct {
            name: "ComplexStruct".to_string(),
            generics: vec![],
            fields,
            is_tuple: false,
        };
        
        let ast = vec![AstNode::Statement(struct_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        let main_func = &ir["main"];
        
        // Check that the struct definition includes complex types
        match &main_func.body[0] {
            Inst::StructDef { name, fields, is_tuple } => {
                assert_eq!(name, "ComplexStruct");
                assert_eq!(fields.len(), 3);
                assert_eq!(fields[0].0, "id");
                assert_eq!(fields[0].1, "i32");
                assert_eq!(fields[1].0, "values");
                assert_eq!(fields[1].1, "Vec<f64>");
                assert_eq!(fields[2].0, "data");
                assert_eq!(fields[2].1, "[i32; 10]");
                assert_eq!(*is_tuple, false);
            }
            _ => panic!("Expected StructDef instruction"),
        }
    }
}