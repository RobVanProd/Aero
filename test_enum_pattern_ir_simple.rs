// Simple test for enum and pattern matching IR generation - Task 10.2
// This test demonstrates basic enum operations in IR

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

fn main() {
    println!("Testing enum and pattern matching IR generation...");
    
    let mut code_gen = CodeGenerator::new();
    
    // Create IR for a simple enum usage with pattern matching
    let function = Function {
        name: "test_enum_pattern".to_string(),
        body: vec![
            // Define an Option enum
            Inst::EnumDef {
                name: "Option".to_string(),
                variants: vec![
                    ("None".to_string(), None),
                    ("Some".to_string(), Some(vec!["i32".to_string()])),
                ],
                discriminant_type: "i32".to_string(),
            },
            // Allocate an Option enum
            Inst::EnumAlloca {
                result: Value::Reg(1),
                enum_name: "Option".to_string(),
            },
            // Construct Some(42)
            Inst::EnumConstruct {
                result: Value::Reg(2),
                enum_name: "Option".to_string(),
                variant_name: "Some".to_string(),
                variant_index: 1,
                data_values: vec![Value::ImmInt(42)],
            },
            // Extract discriminant
            Inst::EnumDiscriminant {
                result: Value::Reg(3),
                enum_ptr: Value::Reg(2),
            },
            // Pattern check
            Inst::PatternCheck {
                result: Value::Reg(4),
                discriminant: Value::Reg(3),
                expected_variant: 1,
            },
            // Match expression
            Inst::Match {
                discriminant: Value::Reg(3),
                arms: vec![
                    MatchArm {
                        pattern_checks: vec![
                            PatternCheck {
                                check_type: PatternCheckType::VariantMatch,
                                target: Value::Reg(3),
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
                                target: Value::Reg(3),
                                expected: PatternValue::Variant(1),
                            }
                        ],
                        bindings: vec![("x".to_string(), Value::Reg(5))],
                        guard: None,
                        body_label: "some_case".to_string(),
                    },
                ],
                default_label: None,
            },
            // Extract data from Some variant
            Inst::EnumExtract {
                result: Value::Reg(6),
                enum_ptr: Value::Reg(2),
                variant_index: 1,
                data_index: 0,
            },
            // Return 0
            Inst::Return(Value::ImmInt(0)),
        ],
        next_reg: 7,
    };
    
    let mut functions = HashMap::new();
    functions.insert("test_enum_pattern".to_string(), function);
    
    let llvm_ir = code_gen.generate_code(functions);
    
    println!("Generated LLVM IR:\n{}", llvm_ir);
    
    // Check for key enum and pattern matching generation features
    let mut checks_passed = 0;
    let total_checks = 8;
    
    if llvm_ir.contains("%Option = type") {
        println!("✓ Enum type definition generated");
        checks_passed += 1;
    } else {
        println!("✗ Enum type definition missing");
    }
    
    if llvm_ir.contains("alloca %Option") {
        println!("✓ Enum allocation generated");
        checks_passed += 1;
    } else {
        println!("✗ Enum allocation missing");
    }
    
    if llvm_ir.contains("store i32 1") {
        println!("✓ Discriminant store generated");
        checks_passed += 1;
    } else {
        println!("✗ Discriminant store missing");
    }
    
    if llvm_ir.contains("load i32") {
        println!("✓ Discriminant load generated");
        checks_passed += 1;
    } else {
        println!("✗ Discriminant load missing");
    }
    
    if llvm_ir.contains("icmp eq i32") {
        println!("✓ Pattern check generated");
        checks_passed += 1;
    } else {
        println!("✗ Pattern check missing");
    }
    
    if llvm_ir.contains("switch i32") {
        println!("✓ Match expression switch generated");
        checks_passed += 1;
    } else {
        println!("✗ Match expression switch missing");
    }
    
    if llvm_ir.contains("none_case") && llvm_ir.contains("some_case") {
        println!("✓ Match arm labels generated");
        checks_passed += 1;
    } else {
        println!("✗ Match arm labels missing");
    }
    
    if llvm_ir.contains("getelementptr") {
        println!("✓ Enum field access generated");
        checks_passed += 1;
    } else {
        println!("✗ Enum field access missing");
    }
    
    println!("\nTest Results: {}/{} checks passed", checks_passed, total_checks);
    
    if checks_passed == total_checks {
        println!("🎉 All enum and pattern matching generation tests passed!");
    } else {
        println!("⚠️  Some enum and pattern matching features need work");
    }
}