// Simple test for struct IR generation - Task 10.1
// This test demonstrates basic struct operations in IR

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
    println!("Testing struct IR generation...");
    
    let mut code_gen = CodeGenerator::new();
    
    // Create IR for a simple struct usage
    let function = Function {
        name: "test_struct".to_string(),
        body: vec![
            // Define a Point struct
            Inst::StructDef {
                name: "Point".to_string(),
                fields: vec![
                    ("x".to_string(), "f64".to_string()),
                    ("y".to_string(), "f64".to_string()),
                ],
                is_tuple: false,
            },
            // Allocate a Point struct
            Inst::StructAlloca {
                result: Value::Reg(1),
                struct_name: "Point".to_string(),
            },
            // Initialize the struct with values
            Inst::StructInit {
                result: Value::Reg(2),
                struct_name: "Point".to_string(),
                field_values: vec![
                    ("x".to_string(), Value::ImmFloat(10.0)),
                    ("y".to_string(), Value::ImmFloat(20.0)),
                ],
            },
            // Access a field
            Inst::FieldAccess {
                result: Value::Reg(3),
                struct_ptr: Value::Reg(2),
                field_name: "x".to_string(),
                field_index: 0,
            },
            // Store a value to a field
            Inst::FieldStore {
                struct_ptr: Value::Reg(2),
                field_name: "y".to_string(),
                field_index: 1,
                value: Value::ImmFloat(30.0),
            },
            // Return 0
            Inst::Return(Value::ImmInt(0)),
        ],
        next_reg: 4,
    };
    
    let mut functions = HashMap::new();
    functions.insert("test_struct".to_string(), function);
    
    let llvm_ir = code_gen.generate_code(functions);
    
    println!("Generated LLVM IR:\n{}", llvm_ir);
    
    // Check for key struct generation features
    let mut checks_passed = 0;
    let total_checks = 6;
    
    if llvm_ir.contains("%Point = type") {
        println!("✓ Struct type definition generated");
        checks_passed += 1;
    } else {
        println!("✗ Struct type definition missing");
    }
    
    if llvm_ir.contains("alloca %Point") {
        println!("✓ Struct allocation generated");
        checks_passed += 1;
    } else {
        println!("✗ Struct allocation missing");
    }
    
    if llvm_ir.contains("getelementptr") {
        println!("✓ Field access generated");
        checks_passed += 1;
    } else {
        println!("✗ Field access missing");
    }
    
    if llvm_ir.contains("store") {
        println!("✓ Field store generated");
        checks_passed += 1;
    } else {
        println!("✗ Field store missing");
    }
    
    if llvm_ir.contains("double") {
        println!("✓ Field types generated");
        checks_passed += 1;
    } else {
        println!("✗ Field types missing");
    }
    
    if llvm_ir.contains("memcpy") {
        println!("✓ Memcpy declaration generated");
        checks_passed += 1;
    } else {
        println!("✗ Memcpy declaration missing");
    }
    
    println!("\nTest Results: {}/{} checks passed", checks_passed, total_checks);
    
    if checks_passed == total_checks {
        println!("🎉 All struct generation tests passed!");
    } else {
        println!("⚠️  Some struct generation features need work");
    }
}