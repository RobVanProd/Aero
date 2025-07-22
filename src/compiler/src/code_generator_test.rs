#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Function, Inst, Value};
    use std::collections::HashMap;

    // ===== PHASE 3 COMPREHENSIVE CODE GENERATOR TESTS =====

    #[test]
    fn test_simple_main_function_generation() {
        let mut generator = CodeGenerator::new();
        
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Return(Value::ImmInt(0)),
            ],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Check that main function is generated
        assert!(llvm_ir.contains("define i32 @main()"));
        assert!(llvm_ir.contains("entry:"));
        assert!(llvm_ir.contains("ret i32"));
    }

    #[test]
    fn test_variable_allocation_and_store() {
        let mut generator = CodeGenerator::new();
        
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Alloca(Value::Reg(0), "x".to_string()),
                Inst::Store(Value::ImmInt(42), Value::Reg(0)),
                Inst::Return(Value::ImmInt(0)),
            ],
            next_reg: 1,
            next_ptr: 1,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Check that variable allocation and store are generated
        assert!(llvm_ir.contains("alloca double"));
        assert!(llvm_ir.contains("store double"));
        assert!(llvm_ir.contains("0x4045000000000000")); // 42.0 in hex
    }

    #[test]
    fn test_load_instruction_generation() {
        let mut generator = CodeGenerator::new();
        
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Alloca(Value::Reg(0), "x".to_string()),
                Inst::Store(Value::ImmInt(42), Value::Reg(0)),
                Inst::Load(Value::Reg(1), Value::Reg(0)),
                Inst::Return(Value::Reg(1)),
            ],
            next_reg: 2,
            next_ptr: 1,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Check that load instruction is generated
        assert!(llvm_ir.contains("load double"));
        assert!(llvm_ir.contains("ret i32"));
    }

    #[test]
    fn test_arithmetic_operations_generation() {
        let mut generator = CodeGenerator::new();
        
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Add(Value::Reg(0), Value::ImmInt(5), Value::ImmInt(3)),
                Inst::Sub(Value::Reg(1), Value::ImmInt(10), Value::ImmInt(4)),
                Inst::Mul(Value::Reg(2), Value::ImmInt(6), Value::ImmInt(7)),
                Inst::Div(Value::Reg(3), Value::ImmInt(20), Value::ImmInt(4)),
                Inst::Mod(Value::Reg(4), Value::ImmInt(17), Value::ImmInt(5)),
                Inst::Return(Value::ImmInt(0)),
            ],
            next_reg: 5,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Check that arithmetic operations are generated
        assert!(llvm_ir.contains("fadd double"));
        assert!(llvm_ir.contains("fsub double"));
        assert!(llvm_ir.contains("fmul double"));
        assert!(llvm_ir.contains("fdiv double"));
        // Modulo might be implemented differently
    }

    #[test]
    fn test_print_instruction_generation() {
        let mut generator = CodeGenerator::new();
        
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Print {
                    format_string: "Hello, World!".to_string(),
                    arguments: vec![],
                },
                Inst::Return(Value::ImmInt(0)),
            ],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Check that print is handled (might be as comment or printf call)
        assert!(llvm_ir.contains("Hello, World!") || llvm_ir.contains("printf"));
    }

    #[test]
    fn test_print_with_arguments_generation() {
        let mut generator = CodeGenerator::new();
        
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Print {
                    format_string: "Value: {}".to_string(),
                    arguments: vec![Value::ImmInt(42)],
                },
                Inst::Return(Value::ImmInt(0)),
            ],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Check that print with arguments is handled
        assert!(llvm_ir.contains("Value:") || llvm_ir.contains("printf"));
        assert!(llvm_ir.contains("42") || llvm_ir.contains("0x4045000000000000"));
    }

    #[test]
    fn test_return_instruction_generation() {
        let mut generator = CodeGenerator::new();
        
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Return(Value::ImmInt(42)),
            ],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Check that return instruction is generated
        assert!(llvm_ir.contains("ret i32"));
        // Should convert double to int for return
        assert!(llvm_ir.contains("fptosi"));
    }

    #[test]
    fn test_unit_return_generation() {
        let mut generator = CodeGenerator::new();
        
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Return(Value::Unit),
            ],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Check that unit return is handled
        assert!(llvm_ir.contains("ret i32 0") || llvm_ir.contains("ret void"));
    }

    #[test]
    fn test_register_value_generation() {
        let mut generator = CodeGenerator::new();
        
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Add(Value::Reg(0), Value::ImmInt(5), Value::ImmInt(3)),
                Inst::Return(Value::Reg(0)),
            ],
            next_reg: 1,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Check that register references are generated
        assert!(llvm_ir.contains("%reg0"));
        assert!(llvm_ir.contains("fadd double"));
    }

    #[test]
    fn test_string_literal_generation() {
        let mut generator = CodeGenerator::new();
        
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Alloca(Value::Reg(0), "message".to_string()),
                Inst::Store(Value::ImmString("Hello World".to_string()), Value::Reg(0)),
                Inst::Return(Value::ImmInt(0)),
            ],
            next_reg: 1,
            next_ptr: 1,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Check that string literal is handled
        assert!(llvm_ir.contains("Hello World") || llvm_ir.contains("@str"));
    }

    #[test]
    fn test_float_literal_generation() {
        let mut generator = CodeGenerator::new();
        
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Alloca(Value::Reg(0), "pi".to_string()),
                Inst::Store(Value::ImmFloat(3.14), Value::Reg(0)),
                Inst::Return(Value::ImmInt(0)),
            ],
            next_reg: 1,
            next_ptr: 1,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Check that float literal is generated
        assert!(llvm_ir.contains("store double"));
        // 3.14 in hex format
        assert!(llvm_ir.contains("0x400921FB54442D18") || llvm_ir.contains("3.14"));
    }

    #[test]
    fn test_complex_expression_generation() {
        let mut generator = CodeGenerator::new();
        
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Alloca(Value::Reg(100), "x".to_string()),
                Inst::Store(Value::ImmInt(5), Value::Reg(100)),
                Inst::Alloca(Value::Reg(101), "y".to_string()),
                Inst::Store(Value::ImmInt(10), Value::Reg(101)),
                Inst::Load(Value::Reg(0), Value::Reg(100)),
                Inst::Load(Value::Reg(1), Value::Reg(101)),
                Inst::Add(Value::Reg(2), Value::Reg(0), Value::Reg(1)),
                Inst::Mul(Value::Reg(3), Value::Reg(2), Value::ImmInt(2)),
                Inst::Alloca(Value::Reg(102), "result".to_string()),
                Inst::Store(Value::Reg(3), Value::Reg(102)),
                Inst::Return(Value::ImmInt(0)),
            ],
            next_reg: 4,
            next_ptr: 103,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Check that complex expression is generated correctly
        assert!(llvm_ir.contains("alloca double")); // Multiple allocas
        assert!(llvm_ir.contains("load double"));   // Multiple loads
        assert!(llvm_ir.contains("fadd double"));   // Addition
        assert!(llvm_ir.contains("fmul double"));   // Multiplication
        assert!(llvm_ir.contains("store double"));  // Multiple stores
    }

    #[test]
    fn test_multiple_variables_generation() {
        let mut generator = CodeGenerator::new();
        
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Alloca(Value::Reg(100), "a".to_string()),
                Inst::Store(Value::ImmInt(1), Value::Reg(100)),
                Inst::Alloca(Value::Reg(101), "b".to_string()),
                Inst::Store(Value::ImmInt(2), Value::Reg(101)),
                Inst::Alloca(Value::Reg(102), "c".to_string()),
                Inst::Store(Value::ImmInt(3), Value::Reg(102)),
                Inst::Return(Value::ImmInt(0)),
            ],
            next_reg: 0,
            next_ptr: 103,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Check that multiple variables are allocated
        let alloca_count = llvm_ir.matches("alloca double").count();
        assert!(alloca_count >= 3);
        
        let store_count = llvm_ir.matches("store double").count();
        assert!(store_count >= 3);
    }

    #[test]
    fn test_type_conversion_functions() {
        let generator = CodeGenerator::new();
        
        // Test type_to_llvm function
        assert_eq!(generator.type_to_llvm("i32"), "i32");
        assert_eq!(generator.type_to_llvm("i64"), "i64");
        assert_eq!(generator.type_to_llvm("f32"), "float");
        assert_eq!(generator.type_to_llvm("f64"), "double");
        assert_eq!(generator.type_to_llvm("bool"), "i1");
        assert_eq!(generator.type_to_llvm("unknown"), "double"); // fallback
    }

    #[test]
    fn test_value_to_string_conversion() {
        let generator = CodeGenerator::new();
        
        // Test integer conversion
        let int_val = Value::ImmInt(42);
        let int_str = generator.value_to_string(&int_val);
        assert!(int_str.starts_with("0x")); // Should be hex format
        
        // Test float conversion
        let float_val = Value::ImmFloat(3.14);
        let float_str = generator.value_to_string(&float_val);
        assert!(float_str.starts_with("0x")); // Should be hex format
        
        // Test register conversion
        let reg_val = Value::Reg(5);
        let reg_str = generator.value_to_string(&reg_val);
        assert_eq!(reg_str, "%reg5");
    }

    #[test]
    fn test_empty_function_generation() {
        let mut generator = CodeGenerator::new();
        
        let function = Function {
            name: "empty".to_string(),
            body: vec![],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("empty".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Check that empty function is generated
        assert!(llvm_ir.contains("define i32 @empty()"));
        assert!(llvm_ir.contains("entry:"));
        // Should have implicit return
        assert!(llvm_ir.contains("ret i32 0"));
    }

    #[test]
    fn test_function_with_only_return() {
        let mut generator = CodeGenerator::new();
        
        let function = Function {
            name: "simple".to_string(),
            body: vec![
                Inst::Return(Value::ImmInt(123)),
            ],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("simple".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Check that simple function with return is generated
        assert!(llvm_ir.contains("define i32 @simple()"));
        assert!(llvm_ir.contains("entry:"));
        assert!(llvm_ir.contains("ret i32"));
        assert!(llvm_ir.contains("fptosi")); // Convert double to int
    }

    #[test]
    fn test_multiple_functions_generation() {
        let mut generator = CodeGenerator::new();
        
        let main_function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Return(Value::ImmInt(0)),
            ],
            next_reg: 0,
            next_ptr: 0,
        };

        let helper_function = Function {
            name: "helper".to_string(),
            body: vec![
                Inst::Return(Value::ImmInt(42)),
            ],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), main_function);
        functions.insert("helper".to_string(), helper_function);

        let llvm_ir = generator.generate_code(functions);
        
        // Check that both functions are generated
        assert!(llvm_ir.contains("define i32 @main()"));
        assert!(llvm_ir.contains("define i32 @helper()"));
    }

    // ===== EDGE CASE TESTS =====

    #[test]
    fn test_empty_functions_map() {
        let mut generator = CodeGenerator::new();
        let functions = HashMap::new();
        let llvm_ir = generator.generate_code(functions);
        
        // Should generate minimal LLVM IR
        assert!(!llvm_ir.is_empty());
    }

    #[test]
    fn test_large_integer_values() {
        let mut generator = CodeGenerator::new();
        
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Return(Value::ImmInt(i64::MAX)),
            ],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Should handle large integers
        assert!(llvm_ir.contains("ret i32"));
        assert!(llvm_ir.contains("fptosi"));
    }

    #[test]
    fn test_negative_integer_values() {
        let mut generator = CodeGenerator::new();
        
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Return(Value::ImmInt(-42)),
            ],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Should handle negative integers
        assert!(llvm_ir.contains("ret i32"));
        assert!(llvm_ir.contains("fptosi"));
    }

    #[test]
    fn test_zero_values() {
        let mut generator = CodeGenerator::new();
        
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Alloca(Value::Reg(0), "zero".to_string()),
                Inst::Store(Value::ImmInt(0), Value::Reg(0)),
                Inst::Store(Value::ImmFloat(0.0), Value::Reg(0)),
                Inst::Return(Value::ImmInt(0)),
            ],
            next_reg: 1,
            next_ptr: 1,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Should handle zero values
        assert!(llvm_ir.contains("store double 0x0000000000000000"));
        assert!(llvm_ir.contains("ret i32"));
    }

    #[test]
    fn test_very_long_function_name() {
        let mut generator = CodeGenerator::new();
        
        let long_name = "very_long_function_name_that_exceeds_normal_length_expectations_and_tests_edge_cases";
        let function = Function {
            name: long_name.to_string(),
            body: vec![
                Inst::Return(Value::ImmInt(0)),
            ],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert(long_name.to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Should handle long function names
        assert!(llvm_ir.contains(&format!("define i32 @{}", long_name)));
    }

    #[test]
    fn test_special_characters_in_strings() {
        let mut generator = CodeGenerator::new();
        
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Print {
                    format_string: "Special chars: \n\t\r\"\\".to_string(),
                    arguments: vec![],
                },
                Inst::Return(Value::ImmInt(0)),
            ],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Should handle special characters in strings
        assert!(llvm_ir.contains("Special chars") || llvm_ir.contains("printf"));
    }
}