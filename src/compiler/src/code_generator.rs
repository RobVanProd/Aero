use std::collections::HashMap;
use crate::ir::{Function, Inst, Value};

pub fn generate_code(ir_functions: HashMap<String, Function>) -> String {
    let mut llvm_ir = String::new();
    llvm_ir.push_str("; ModuleID = \"aero_compiler\"\n");
    llvm_ir.push_str("source_filename = \"aero_compiler\"\n");
    llvm_ir.push_str("target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"\n");
    llvm_ir.push_str("target triple = \"x86_64-pc-linux-gnu\"\n\n");

    for (func_name, func) in ir_functions {
        llvm_ir.push_str(&format!("define i32 @{}() {{\nentry:\n", func_name)); // Changed return type to i32

        let mut var_map = HashMap::new(); // Maps variable names to their alloca\'d pointer registers

        for inst in &func.body { // Iterate over reference to avoid moving func.body
            match inst {
                Inst::Alloca(ptr_reg, name) => {
                    llvm_ir.push_str(&format!("  %ptr{} = alloca i64, align 8\n", ptr_reg));
                    var_map.insert(name.clone(), ptr_reg.clone());
                }
                Inst::Store(ptr_reg, value) => {
                    let val_str = match value {
                        Value::ImmInt(n) => n.to_string(),
                        Value::ImmFloat(f) => f.to_string(),
                        Value::Reg(r) => format!("%reg{}", r),
                    };
                    llvm_ir.push_str(&format!("  store i64 {}, i64* %ptr{}, align 8\n", val_str, ptr_reg));
                }
                Inst::Load(result_reg, ptr_reg) => {
                    llvm_ir.push_str(&format!("  %reg{} = load i64, i64* %ptr{}, align 8\n", result_reg, ptr_reg));
                }
                Inst::Add(result_reg, lhs, rhs) => {
                    let lhs_str = match lhs {
                        Value::ImmInt(n) => n.to_string(),
                        Value::ImmFloat(f) => f.to_string(),
                        Value::Reg(r) => format!("%reg{}", r),
                    };
                    let rhs_str = match rhs {
                        Value::ImmInt(n) => n.to_string(),
                        Value::ImmFloat(f) => f.to_string(),
                        Value::Reg(r) => format!("%reg{}", r),
                    };
                    llvm_ir.push_str(&format!("  %reg{} = add i64 {}, {}\n", result_reg, lhs_str, rhs_str));
                }
                Inst::Sub(result_reg, lhs, rhs) => {
                    let lhs_str = match lhs {
                        Value::ImmInt(n) => n.to_string(),
                        Value::ImmFloat(f) => f.to_string(),
                        Value::Reg(r) => format!("%reg{}", r),
                    };
                    let rhs_str = match rhs {
                        Value::ImmInt(n) => n.to_string(),
                        Value::ImmFloat(f) => f.to_string(),
                        Value::Reg(r) => format!("%reg{}", r),
                    };
                    llvm_ir.push_str(&format!("  %reg{} = sub i64 {}, {}\n", result_reg, lhs_str, rhs_str));
                }
                Inst::Mul(result_reg, lhs, rhs) => {
                    let lhs_str = match lhs {
                        Value::ImmInt(n) => n.to_string(),
                        Value::ImmFloat(f) => f.to_string(),
                        Value::Reg(r) => format!("%reg{}", r),
                    };
                    let rhs_str = match rhs {
                        Value::ImmInt(n) => n.to_string(),
                        Value::ImmFloat(f) => f.to_string(),
                        Value::Reg(r) => format!("%reg{}", r),
                    };
                    llvm_ir.push_str(&format!("  %reg{} = mul i64 {}, {}\n", result_reg, lhs_str, rhs_str));
                }
                Inst::Div(result_reg, lhs, rhs) => {
                    let lhs_str = match lhs {
                        Value::ImmInt(n) => n.to_string(),
                        Value::ImmFloat(f) => f.to_string(),
                        Value::Reg(r) => format!("%reg{}", r),
                    };
                    let rhs_str = match rhs {
                        Value::ImmInt(n) => n.to_string(),
                        Value::ImmFloat(f) => f.to_string(),
                        Value::Reg(r) => format!("%reg{}", r),
                    };
                    llvm_ir.push_str(&format!("  %reg{} = sdiv i64 {}, {}\n", result_reg, lhs_str, rhs_str));
                }
                Inst::Return(value) => {
                    let val_str = match value {
                        Value::ImmInt(n) => n.to_string(),
                        Value::ImmFloat(f) => f.to_string(),
                        Value::Reg(r) => format!("%reg{}", r),
                    };
                    // Truncate to i32 if necessary
                    let trunc_reg = func.next_reg; // Use a new register for the truncated value
                    llvm_ir.push_str(&format!("  %reg{} = trunc i64 {} to i32\n", trunc_reg, val_str)); // Added 'to' keyword
                    llvm_ir.push_str(&format!("  ret i32 %reg{}\n", trunc_reg));
                }
            }
        }
        // If no explicit return, return 0
        if !func.body.is_empty() && !func.body.iter().any(|inst| matches!(inst, Inst::Return(_))) { // Check if body is not empty before iterating
            llvm_ir.push_str("  ret i32 0\n");
        }
        llvm_ir.push_str("}\n\n");
    }

    llvm_ir
}


