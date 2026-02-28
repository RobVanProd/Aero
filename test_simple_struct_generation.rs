// Simple test for LLVM struct generation - Task 10.1
// This test verifies basic struct generation functionality

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
    println!("Testing basic struct generation...");
    
    let mut code_gen = CodeGenerator::new();
    
    // Create a simple struct definition IR
    let struct_def = Inst::StructDef {
        name: "Point".to_string(),
        fields: vec![
            ("x".to_string(), "f64".to_string()),
            ("y".to_string(), "f64".to_string()),
        ],
        is_tuple: false,
    };
    
    let function = Function {
        name: "test_struct".to_string(),
        body: vec![struct_def],
        next_reg: 1,
    };
    
    let mut functions = HashMap::new();
    functions.insert("test_struct".to_string(), function);
    
    let llvm_ir = code_gen.generate_code(functions);
    
    println!("Generated LLVM IR:\n{}", llvm_ir);
    
    // Basic checks
    if llvm_ir.contains("%Point = type") {
        println!("✓ Struct type definition found");
    } else {
        println!("✗ Struct type definition missing");
    }
    
    if llvm_ir.contains("double") {
        println!("✓ Field types found");
    } else {
        println!("✗ Field types missing");
    }
    
    println!("Basic struct generation test completed!");
}