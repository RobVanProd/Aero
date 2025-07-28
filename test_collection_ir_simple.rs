// Simple test for collection IR generation - Task 10.3
// This test demonstrates basic array and Vec operations in IR

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

use src::compiler::src::ir::{Function, Inst, Value};
use src::compiler::src::code_generator::CodeGenerator;

fn main() {
    println!("Testing collection IR generation...");
    
    let mut code_gen = CodeGenerator::new();
    
    // Create IR for array and Vec operations
    let function = Function {
        name: "test_collections".to_string(),
        body: vec![
            // Array operations
            Inst::ArrayInit {
                result: Value::Reg(1),
                element_type: "i32".to_string(),
                elements: vec![
                    Value::ImmInt(1),
                    Value::ImmInt(2),
                    Value::ImmInt(3),
                    Value::ImmInt(4),
                    Value::ImmInt(5),
                ],
            },
            // Array access
            Inst::ArrayAccess {
                result: Value::Reg(2),
                array_ptr: Value::Reg(1),
                index: Value::ImmInt(0),
            },
            // Array store
            Inst::ArrayStore {
                array_ptr: Value::Reg(1),
                index: Value::ImmInt(1),
                value: Value::ImmInt(42),
            },
            // Array length
            Inst::ArrayLength {
                result: Value::Reg(3),
                array_ptr: Value::Reg(1),
            },
            // Bounds check
            Inst::BoundsCheck {
                array_ptr: Value::Reg(1),
                index: Value::ImmInt(2),
                success_label: "bounds_ok".to_string(),
                failure_label: "bounds_fail".to_string(),
            },
            // Label for bounds check success
            Inst::Label("bounds_ok".to_string()),
            // Vec operations
            Inst::VecInit {
                result: Value::Reg(4),
                element_type: "f64".to_string(),
                elements: vec![
                    Value::ImmFloat(1.0),
                    Value::ImmFloat(2.0),
                    Value::ImmFloat(3.0),
                ],
            },
            // Vec push
            Inst::VecPush {
                vec_ptr: Value::Reg(4),
                value: Value::ImmFloat(4.0),
            },
            // Vec access
            Inst::VecAccess {
                result: Value::Reg(5),
                vec_ptr: Value::Reg(4),
                index: Value::ImmInt(0),
            },
            // Vec length
            Inst::VecLength {
                result: Value::Reg(6),
                vec_ptr: Value::Reg(4),
            },
            // Vec capacity
            Inst::VecCapacity {
                result: Value::Reg(7),
                vec_ptr: Value::Reg(4),
            },
            // Vec pop
            Inst::VecPop {
                result: Value::Reg(8),
                vec_ptr: Value::Reg(4),
            },
            // Return 0
            Inst::Return(Value::ImmInt(0)),
        ],
        next_reg: 9,
    };
    
    let mut functions = HashMap::new();
    functions.insert("test_collections".to_string(), function);
    
    let llvm_ir = code_gen.generate_code(functions);
    
    println!("Generated LLVM IR:\n{}", llvm_ir);
    
    // Check for key collection generation features
    let mut checks_passed = 0;
    let total_checks = 12;
    
    if llvm_ir.contains("alloca [5 x i32]") {
        println!("✓ Array allocation generated");
        checks_passed += 1;
    } else {
        println!("✗ Array allocation missing");
    }
    
    if llvm_ir.contains("getelementptr inbounds [5 x i32]") {
        println!("✓ Array element access generated");
        checks_passed += 1;
    } else {
        println!("✗ Array element access missing");
    }
    
    if llvm_ir.contains("store i32") {
        println!("✓ Array element store generated");
        checks_passed += 1;
    } else {
        println!("✗ Array element store missing");
    }
    
    if llvm_ir.contains("icmp ult i64") {
        println!("✓ Bounds checking generated");
        checks_passed += 1;
    } else {
        println!("✗ Bounds checking missing");
    }
    
    if llvm_ir.contains("bounds_ok") && llvm_ir.contains("bounds_fail") {
        println!("✓ Bounds check labels generated");
        checks_passed += 1;
    } else {
        println!("✗ Bounds check labels missing");
    }
    
    if llvm_ir.contains("alloca { double*, i64, i64 }") {
        println!("✓ Vec structure allocation generated");
        checks_passed += 1;
    } else {
        println!("✗ Vec structure allocation missing");
    }
    
    if llvm_ir.contains("call i8* @malloc") {
        println!("✓ Vec memory allocation generated");
        checks_passed += 1;
    } else {
        println!("✗ Vec memory allocation missing");
    }
    
    if llvm_ir.contains("store double*, double**") {
        println!("✓ Vec data pointer setup generated");
        checks_passed += 1;
    } else {
        println!("✗ Vec data pointer setup missing");
    }
    
    if llvm_ir.contains("add i64") {
        println!("✓ Vec length increment generated");
        checks_passed += 1;
    } else {
        println!("✗ Vec length increment missing");
    }
    
    if llvm_ir.contains("sub i64") {
        println!("✓ Vec length decrement generated");
        checks_passed += 1;
    } else {
        println!("✗ Vec length decrement missing");
    }
    
    if llvm_ir.contains("sitofp i64") {
        println!("✓ Vec length/capacity conversion generated");
        checks_passed += 1;
    } else {
        println!("✗ Vec length/capacity conversion missing");
    }
    
    if llvm_ir.contains("declare i8* @malloc") {
        println!("✓ Malloc declaration generated");
        checks_passed += 1;
    } else {
        println!("✗ Malloc declaration missing");
    }
    
    println!("\nTest Results: {}/{} checks passed", checks_passed, total_checks);
    
    if checks_passed == total_checks {
        println!("🎉 All collection generation tests passed!");
    } else {
        println!("⚠️  Some collection features need work");
    }
}