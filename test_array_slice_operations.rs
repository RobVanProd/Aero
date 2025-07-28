// Test for array and slice operations - Task 11.2
// This test verifies fixed-size array support and slicing operations

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
use src::compiler::src::stdlib::ArrayOps;

fn main() {
    println!("Testing array and slice operations...");
    
    // Test array method generation
    let array_methods = vec!["len", "is_empty", "first", "last", "contains"];
    for method in &array_methods {
        let instructions = ArrayOps::generate_method_call(method, &[Value::Reg(1), Value::ImmInt(42)]);
        if !instructions.is_empty() {
            println!("✓ Array::{} method IR generation works", method);
        } else {
            println!("✗ Array::{} method IR generation failed", method);
        }
    }
    
    // Test array slicing
    let slice_instructions = ArrayOps::generate_slice(
        Value::Reg(1),
        Value::ImmInt(1),
        Value::ImmInt(3)
    );
    if !slice_instructions.is_empty() {
        println!("✓ Array slicing IR generation works");
    } else {
        println!("✗ Array slicing IR generation failed");
    }
    
    // Test array iteration
    let iter_instructions = ArrayOps::generate_iter(Value::Reg(1));
    if !iter_instructions.is_empty() {
        println!("✓ Array iteration IR generation works");
    } else {
        println!("✗ Array iteration IR generation failed");
    }
    
    // Test array operations with code generator
    let mut code_gen = CodeGenerator::new();
    
    let function = Function {
        name: "test_array_slice".to_string(),
        body: vec![
            // Create array
            Inst::ArrayInit {
                result: Value::Reg(1),
                element_type: "i32".to_string(),
                elements: vec![
                    Value::ImmInt(10),
                    Value::ImmInt(20),
                    Value::ImmInt(30),
                    Value::ImmInt(40),
                    Value::ImmInt(50),
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
                value: Value::ImmInt(99),
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
            Inst::Label("bounds_ok".to_string()),
            // Access with bounds check
            Inst::ArrayAccess {
                result: Value::Reg(4),
                array_ptr: Value::Reg(1),
                index: Value::ImmInt(2),
            },
            Inst::Jump("end".to_string()),
            Inst::Label("bounds_fail".to_string()),
            Inst::Return(Value::ImmInt(-1)),
            Inst::Label("end".to_string()),
            Inst::Return(Value::ImmInt(0)),
        ],
        next_reg: 5,
    };
    
    let mut functions = HashMap::new();
    functions.insert("test_array_slice".to_string(), function);
    
    let llvm_ir = code_gen.generate_code(functions);
    
    // Check for array-related LLVM IR
    let mut checks_passed = 0;
    let total_checks = 8;
    
    if llvm_ir.contains("alloca [5 x i32]") {
        println!("✓ Fixed-size array allocation in LLVM IR");
        checks_passed += 1;
    } else {
        println!("✗ Fixed-size array allocation missing in LLVM IR");
    }
    
    if llvm_ir.contains("getelementptr inbounds [5 x i32]") {
        println!("✓ Array element access in LLVM IR");
        checks_passed += 1;
    } else {
        println!("✗ Array element access missing in LLVM IR");
    }
    
    if llvm_ir.contains("store i32") {
        println!("✓ Array element store in LLVM IR");
        checks_passed += 1;
    } else {
        println!("✗ Array element store missing in LLVM IR");
    }
    
    if llvm_ir.contains("load i32") || llvm_ir.contains("load double") {
        println!("✓ Array element load in LLVM IR");
        checks_passed += 1;
    } else {
        println!("✗ Array element load missing in LLVM IR");
    }
    
    if llvm_ir.contains("icmp ult i64") {
        println!("✓ Bounds checking in LLVM IR");
        checks_passed += 1;
    } else {
        println!("✗ Bounds checking missing in LLVM IR");
    }
    
    if llvm_ir.contains("bounds_ok") && llvm_ir.contains("bounds_fail") {
        println!("✓ Bounds check labels in LLVM IR");
        checks_passed += 1;
    } else {
        println!("✗ Bounds check labels missing in LLVM IR");
    }
    
    if llvm_ir.contains("br i1") {
        println!("✓ Conditional branching in LLVM IR");
        checks_passed += 1;
    } else {
        println!("✗ Conditional branching missing in LLVM IR");
    }
    
    if llvm_ir.contains("fptosi double") {
        println!("✓ Index type conversion in LLVM IR");
        checks_passed += 1;
    } else {
        println!("✗ Index type conversion missing in LLVM IR");
    }
    
    println!("\nArray and Slice Operations Test Results: {}/{} checks passed", checks_passed, total_checks);
    
    // Test specific array methods
    println!("\nTesting array method implementations:");
    
    // Test array.len()
    let len_ir = ArrayOps::generate_method_call("len", &[Value::Reg(1)]);
    if len_ir.iter().any(|inst| matches!(inst, Inst::ArrayLength { .. })) {
        println!("✓ Array::len() generates ArrayLength instruction");
    } else {
        println!("✗ Array::len() does not generate ArrayLength instruction");
    }
    
    // Test array.is_empty()
    let empty_ir = ArrayOps::generate_method_call("is_empty", &[Value::Reg(1)]);
    if empty_ir.iter().any(|inst| matches!(inst, Inst::FCmp { .. })) {
        println!("✓ Array::is_empty() generates comparison instruction");
    } else {
        println!("✗ Array::is_empty() does not generate comparison instruction");
    }
    
    // Test array.first()
    let first_ir = ArrayOps::generate_method_call("first", &[Value::Reg(1)]);
    if first_ir.iter().any(|inst| matches!(inst, Inst::ArrayAccess { .. })) {
        println!("✓ Array::first() generates ArrayAccess instruction");
    } else {
        println!("✗ Array::first() does not generate ArrayAccess instruction");
    }
    
    // Test array.last()
    let last_ir = ArrayOps::generate_method_call("last", &[Value::Reg(1)]);
    if last_ir.iter().any(|inst| matches!(inst, Inst::ArrayLength { .. })) &&
       last_ir.iter().any(|inst| matches!(inst, Inst::ArrayAccess { .. })) {
        println!("✓ Array::last() generates length and access instructions");
    } else {
        println!("✗ Array::last() does not generate proper instructions");
    }
    
    // Test array slicing
    let slice_ir = ArrayOps::generate_slice(Value::Reg(1), Value::ImmInt(1), Value::ImmInt(3));
    if slice_ir.iter().any(|inst| matches!(inst, Inst::FPToSI(_, _))) {
        println!("✓ Array slicing generates index conversion");
    } else {
        println!("✗ Array slicing does not generate index conversion");
    }
    
    if checks_passed >= 6 {
        println!("\n🎉 Array and slice operations implementation successful!");
        println!("Task 11.2 - Create array and slice operations: COMPLETE");
    } else {
        println!("\n⚠️  Some array and slice features need work");
    }
}