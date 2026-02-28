// Test for LLVM struct generation - Task 10.1
// This test verifies that the code generator can properly generate LLVM IR for struct operations

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

#[test]
fn test_struct_definition_llvm_generation() {
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
    
    // Verify that the LLVM IR contains struct type definition
    assert!(llvm_ir.contains("%Point = type"));
    assert!(llvm_ir.contains("double")); // Should contain field types
    
    println!("Generated LLVM IR:\n{}", llvm_ir);
}

#[test]
fn test_struct_allocation_llvm_generation() {
    let mut code_gen = CodeGenerator::new();
    
    // Create struct allocation IR
    let struct_alloca = Inst::StructAlloca {
        result: Value::Reg(1),
        struct_name: "Point".to_string(),
    };
    
    let function = Function {
        name: "test_alloca".to_string(),
        body: vec![struct_alloca],
        next_reg: 2,
    };
    
    let mut functions = HashMap::new();
    functions.insert("test_alloca".to_string(), function);
    
    let llvm_ir = code_gen.generate_code(functions);
    
    // Verify that the LLVM IR contains struct allocation
    assert!(llvm_ir.contains("alloca"));
    assert!(llvm_ir.contains("%Point"));
    
    println!("Generated LLVM IR:\n{}", llvm_ir);
}

#[test]
fn test_struct_field_access_llvm_generation() {
    let mut code_gen = CodeGenerator::new();
    
    // Create field access IR
    let field_access = Inst::FieldAccess {
        result: Value::Reg(2),
        struct_ptr: Value::Reg(1),
        field_name: "x".to_string(),
        field_index: 0,
    };
    
    let function = Function {
        name: "test_field_access".to_string(),
        body: vec![field_access],
        next_reg: 3,
    };
    
    let mut functions = HashMap::new();
    functions.insert("test_field_access".to_string(), function);
    
    let llvm_ir = code_gen.generate_code(functions);
    
    // Verify that the LLVM IR contains getelementptr for field access
    assert!(llvm_ir.contains("getelementptr"));
    
    println!("Generated LLVM IR:\n{}", llvm_ir);
}

#[test]
fn test_struct_initialization_llvm_generation() {
    let mut code_gen = CodeGenerator::new();
    
    // Create struct initialization IR
    let struct_init = Inst::StructInit {
        result: Value::Reg(1),
        struct_name: "Point".to_string(),
        field_values: vec![
            ("x".to_string(), Value::ImmFloat(10.0)),
            ("y".to_string(), Value::ImmFloat(20.0)),
        ],
    };
    
    let function = Function {
        name: "test_init".to_string(),
        body: vec![struct_init],
        next_reg: 2,
    };
    
    let mut functions = HashMap::new();
    functions.insert("test_init".to_string(), function);
    
    let llvm_ir = code_gen.generate_code(functions);
    
    // Verify that the LLVM IR contains struct initialization
    assert!(llvm_ir.contains("store"));
    
    println!("Generated LLVM IR:\n{}", llvm_ir);
}

fn main() {
    test_struct_definition_llvm_generation();
    test_struct_allocation_llvm_generation();
    test_struct_field_access_llvm_generation();
    test_struct_initialization_llvm_generation();
    println!("All LLVM struct generation tests passed!");
}