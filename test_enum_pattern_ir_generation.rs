// Test for LLVM enum and pattern matching generation - Task 10.2
// This test verifies that the code generator can properly generate LLVM IR for enum operations

use std::collections::HashMap;

// Include the compiler modules
mod src {
    pub mod compiler {
        pub mod src {
            pub mod ir;
            pub mod code_generator;
        }
    }
}

use src::compiler::src::ir::{Function, Inst, Value, MatchArm, PatternCheck, PatternCheckType, PatternValue};
use src::compiler::src::code_generator::CodeGenerator;

#[test]
fn test_enum_definition_llvm_generation() {
    let mut code_gen = CodeGenerator::new();
    
    // Create a simple enum definition IR
    let enum_def = Inst::EnumDef {
        name: "Option".to_string(),
        variants: vec![
            ("None".to_string(), None),
            ("Some".to_string(), Some(vec!["i32".to_string()])),
        ],
        discriminant_type: "i32".to_string(),
    };
    
    let function = Function {
        name: "test_enum".to_string(),
        body: vec![enum_def],
        next_reg: 1,
    };
    
    let mut functions = HashMap::new();
    functions.insert("test_enum".to_string(), function);
    
    let llvm_ir = code_gen.generate_code(functions);
    
    // Verify that the LLVM IR contains enum type definition
    assert!(llvm_ir.contains("%Option = type"));
    assert!(llvm_ir.contains("i32")); // Should contain discriminant type
    
    println!("Generated LLVM IR:\n{}", llvm_ir);
}

#[test]
fn test_enum_allocation_llvm_generation() {
    let mut code_gen = CodeGenerator::new();
    
    // Create enum allocation IR
    let enum_alloca = Inst::EnumAlloca {
        result: Value::Reg(1),
        enum_name: "Option".to_string(),
    };
    
    let function = Function {
        name: "test_alloca".to_string(),
        body: vec![enum_alloca],
        next_reg: 2,
    };
    
    let mut functions = HashMap::new();
    functions.insert("test_alloca".to_string(), function);
    
    let llvm_ir = code_gen.generate_code(functions);
    
    // Verify that the LLVM IR contains enum allocation
    assert!(llvm_ir.contains("alloca"));
    assert!(llvm_ir.contains("%Option"));
    
    println!("Generated LLVM IR:\n{}", llvm_ir);
}

#[test]
fn test_enum_construction_llvm_generation() {
    let mut code_gen = CodeGenerator::new();
    
    // Create enum construction IR
    let enum_construct = Inst::EnumConstruct {
        result: Value::Reg(1),
        enum_name: "Option".to_string(),
        variant_name: "Some".to_string(),
        variant_index: 1,
        data_values: vec![Value::ImmInt(42)],
    };
    
    let function = Function {
        name: "test_construct".to_string(),
        body: vec![enum_construct],
        next_reg: 2,
    };
    
    let mut functions = HashMap::new();
    functions.insert("test_construct".to_string(), function);
    
    let llvm_ir = code_gen.generate_code(functions);
    
    // Verify that the LLVM IR contains enum construction
    assert!(llvm_ir.contains("store i32 1")); // Discriminant store
    assert!(llvm_ir.contains("getelementptr"));
    
    println!("Generated LLVM IR:\n{}", llvm_ir);
}

#[test]
fn test_enum_discriminant_llvm_generation() {
    let mut code_gen = CodeGenerator::new();
    
    // Create discriminant extraction IR
    let enum_discriminant = Inst::EnumDiscriminant {
        result: Value::Reg(2),
        enum_ptr: Value::Reg(1),
    };
    
    let function = Function {
        name: "test_discriminant".to_string(),
        body: vec![enum_discriminant],
        next_reg: 3,
    };
    
    let mut functions = HashMap::new();
    functions.insert("test_discriminant".to_string(), function);
    
    let llvm_ir = code_gen.generate_code(functions);
    
    // Verify that the LLVM IR contains discriminant extraction
    assert!(llvm_ir.contains("getelementptr"));
    assert!(llvm_ir.contains("load i32"));
    
    println!("Generated LLVM IR:\n{}", llvm_ir);
}

#[test]
fn test_pattern_matching_llvm_generation() {
    let mut code_gen = CodeGenerator::new();
    
    // Create pattern matching IR
    let match_arms = vec![
        MatchArm {
            pattern_checks: vec![
                PatternCheck {
                    check_type: PatternCheckType::VariantMatch,
                    target: Value::Reg(1),
                    expected: PatternValue::Variant(0),
                }
            ],
            bindings: vec![],
            guard: None,
            body_label: "none_case".to_string(),
        },
        MatchArm {
            pattern_checks: vec![
                PatternCheck {
                    check_type: PatternCheckType::VariantMatch,
                    target: Value::Reg(1),
                    expected: PatternValue::Variant(1),
                }
            ],
            bindings: vec![("x".to_string(), Value::Reg(2))],
            guard: None,
            body_label: "some_case".to_string(),
        },
    ];
    
    let match_expr = Inst::Match {
        discriminant: Value::Reg(1),
        arms: match_arms,
        default_label: None,
    };
    
    let function = Function {
        name: "test_match".to_string(),
        body: vec![match_expr],
        next_reg: 3,
    };
    
    let mut functions = HashMap::new();
    functions.insert("test_match".to_string(), function);
    
    let llvm_ir = code_gen.generate_code(functions);
    
    // Verify that the LLVM IR contains switch statement
    assert!(llvm_ir.contains("switch i32"));
    assert!(llvm_ir.contains("none_case"));
    assert!(llvm_ir.contains("some_case"));
    
    println!("Generated LLVM IR:\n{}", llvm_ir);
}

#[test]
fn test_pattern_check_llvm_generation() {
    let mut code_gen = CodeGenerator::new();
    
    // Create pattern check IR
    let pattern_check = Inst::PatternCheck {
        result: Value::Reg(2),
        discriminant: Value::Reg(1),
        expected_variant: 1,
    };
    
    let function = Function {
        name: "test_pattern_check".to_string(),
        body: vec![pattern_check],
        next_reg: 3,
    };
    
    let mut functions = HashMap::new();
    functions.insert("test_pattern_check".to_string(), function);
    
    let llvm_ir = code_gen.generate_code(functions);
    
    // Verify that the LLVM IR contains comparison
    assert!(llvm_ir.contains("icmp eq i32"));
    
    println!("Generated LLVM IR:\n{}", llvm_ir);
}

fn main() {
    test_enum_definition_llvm_generation();
    test_enum_allocation_llvm_generation();
    test_enum_construction_llvm_generation();
    test_enum_discriminant_llvm_generation();
    test_pattern_matching_llvm_generation();
    test_pattern_check_llvm_generation();
    println!("All LLVM enum and pattern matching generation tests passed!");
}