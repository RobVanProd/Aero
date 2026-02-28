// Test for Vec implementation - Task 11.1
// This test verifies the built-in Vec<T> collection type

use std::collections::HashMap;

// Include the compiler modules
mod src {
    pub mod compiler {
        pub mod src {
            pub mod ir;
            pub mod code_generator;
            pub mod stdlib;
        }
    }
}

use src::compiler::src::ir::{Function, Inst, Value};
use src::compiler::src::code_generator::CodeGenerator;
use src::compiler::src::stdlib::{VecType, CollectionLibrary};

fn main() {
    println!("Testing Vec<T> implementation...");
    
    // Test Vec type creation
    let vec_type = VecType::new("i32".to_string());
    println!("✓ Vec<i32> type created");
    
    // Test Vec methods
    let methods = vec!["new", "push", "pop", "len", "capacity", "is_empty", "clear", "get"];
    for method in &methods {
        if vec_type.methods.contains_key(*method) {
            println!("✓ Vec::{} method available", method);
        } else {
            println!("✗ Vec::{} method missing", method);
        }
    }
    
    // Test Vec method IR generation
    let new_instructions = vec_type.generate_method_call("new", &[]);
    if !new_instructions.is_empty() {
        println!("✓ Vec::new() IR generation works");
    } else {
        println!("✗ Vec::new() IR generation failed");
    }
    
    let push_instructions = vec_type.generate_method_call("push", &[Value::Reg(1), Value::ImmInt(42)]);
    if !push_instructions.is_empty() {
        println!("✓ Vec::push() IR generation works");
    } else {
        println!("✗ Vec::push() IR generation failed");
    }
    
    let pop_instructions = vec_type.generate_method_call("pop", &[Value::Reg(1)]);
    if !pop_instructions.is_empty() {
        println!("✓ Vec::pop() IR generation works");
    } else {
        println!("✗ Vec::pop() IR generation failed");
    }
    
    let len_instructions = vec_type.generate_method_call("len", &[Value::Reg(1)]);
    if !len_instructions.is_empty() {
        println!("✓ Vec::len() IR generation works");
    } else {
        println!("✗ Vec::len() IR generation failed");
    }
    
    // Test vec! macro
    let vec_macro = CollectionLibrary::generate_vec_macro(
        vec![Value::ImmInt(1), Value::ImmInt(2), Value::ImmInt(3)],
        "i32".to_string()
    );
    if !vec_macro.is_empty() {
        println!("✓ vec![] macro IR generation works");
    } else {
        println!("✗ vec![] macro IR generation failed");
    }
    
    // Test Vec iteration
    let for_loop = CollectionLibrary::generate_for_loop(
        Value::Reg(1),
        "item".to_string(),
        vec![
            Inst::Print {
                format_string: "Item: {}".to_string(),
                arguments: vec![Value::Reg(47)],
            }
        ]
    );
    if !for_loop.is_empty() {
        println!("✓ Vec iteration IR generation works");
    } else {
        println!("✗ Vec iteration IR generation failed");
    }
    
    // Test collection library
    let mut library = CollectionLibrary::new();
    library.register_vec_type("i32".to_string());
    library.register_vec_type("f64".to_string());
    library.register_vec_type("String".to_string());
    
    if library.get_vec_type("i32").is_some() {
        println!("✓ Vec<i32> registered in library");
    } else {
        println!("✗ Vec<i32> not found in library");
    }
    
    if library.get_vec_type("f64").is_some() {
        println!("✓ Vec<f64> registered in library");
    } else {
        println!("✗ Vec<f64> not found in library");
    }
    
    // Test Vec with code generator
    let mut code_gen = CodeGenerator::new();
    
    let function = Function {
        name: "test_vec".to_string(),
        body: vec![
            // Create new Vec
            Inst::VecAlloca {
                result: Value::Reg(1),
                element_type: "i32".to_string(),
            },
            // Initialize with elements
            Inst::VecInit {
                result: Value::Reg(2),
                element_type: "i32".to_string(),
                elements: vec![
                    Value::ImmInt(1),
                    Value::ImmInt(2),
                    Value::ImmInt(3),
                ],
            },
            // Push element
            Inst::VecPush {
                vec_ptr: Value::Reg(2),
                value: Value::ImmInt(4),
            },
            // Get length
            Inst::VecLength {
                result: Value::Reg(3),
                vec_ptr: Value::Reg(2),
            },
            // Access element
            Inst::VecAccess {
                result: Value::Reg(4),
                vec_ptr: Value::Reg(2),
                index: Value::ImmInt(0),
            },
            // Pop element
            Inst::VecPop {
                result: Value::Reg(5),
                vec_ptr: Value::Reg(2),
            },
            Inst::Return(Value::ImmInt(0)),
        ],
        next_reg: 6,
    };
    
    let mut functions = HashMap::new();
    functions.insert("test_vec".to_string(), function);
    
    let llvm_ir = code_gen.generate_code(functions);
    
    // Check for Vec-related LLVM IR
    let mut checks_passed = 0;
    let total_checks = 6;
    
    if llvm_ir.contains("alloca { double*, i64, i64 }") {
        println!("✓ Vec structure allocation in LLVM IR");
        checks_passed += 1;
    } else {
        println!("✗ Vec structure allocation missing in LLVM IR");
    }
    
    if llvm_ir.contains("call i8* @malloc") {
        println!("✓ Vec memory allocation in LLVM IR");
        checks_passed += 1;
    } else {
        println!("✗ Vec memory allocation missing in LLVM IR");
    }
    
    if llvm_ir.contains("add i64") {
        println!("✓ Vec length increment in LLVM IR");
        checks_passed += 1;
    } else {
        println!("✗ Vec length increment missing in LLVM IR");
    }
    
    if llvm_ir.contains("sub i64") {
        println!("✓ Vec length decrement in LLVM IR");
        checks_passed += 1;
    } else {
        println!("✗ Vec length decrement missing in LLVM IR");
    }
    
    if llvm_ir.contains("getelementptr inbounds double") {
        println!("✓ Vec element access in LLVM IR");
        checks_passed += 1;
    } else {
        println!("✗ Vec element access missing in LLVM IR");
    }
    
    if llvm_ir.contains("sitofp i64") {
        println!("✓ Vec length conversion in LLVM IR");
        checks_passed += 1;
    } else {
        println!("✗ Vec length conversion missing in LLVM IR");
    }
    
    println!("\nVec Implementation Test Results: {}/{} checks passed", checks_passed, total_checks);
    
    if checks_passed == total_checks {
        println!("🎉 All Vec<T> implementation tests passed!");
        println!("Task 11.1 - Create Vec implementation: COMPLETE");
    } else {
        println!("⚠️  Some Vec<T> features need work");
    }
}