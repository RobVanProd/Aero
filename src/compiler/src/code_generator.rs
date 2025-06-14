use std::collections::HashMap;
use crate::ir::{Function, Inst, Value};

pub fn generate_code(ir_functions: HashMap<String, Function>) -> String {
    let mut llvm_ir = String::new();
    llvm_ir.push_str("; ModuleID = \"aero_compiler\"\n");
    llvm_ir.push_str("source_filename = \"aero_compiler\"\n\n");

    for (func_name, func) in ir_functions {
        llvm_ir.push_str(&format!("define i64 @{}() {{\nentry:\n", func_name)); // Removed align 16 and fixed newline

        let mut var_map = HashMap::new();

        for inst in func.body {
            match inst {
                Inst::Store(name, value) => {
                    let val_str = match value {
                        Value::ImmInt(n) => n.to_string(),
                        Value::ImmFloat(f) => f.to_string(),
                        Value::Reg(r) => format!("%reg{}", r),
                        _ => panic!("Unsupported value type for store"),
                    };
                    llvm_ir.push_str(&format!("  %{} = alloca i64, align 8\n", name));
                    llvm_ir.push_str(&format!("  store i64 {}, i64* %{}, align 8\n", val_str, name));
                    var_map.insert(name.clone(), format!("%{}", name));
                }
                Inst::Load(reg, name) => {
                    let var_ptr = var_map.get(&name).expect("Undeclared variable");
                    llvm_ir.push_str(&format!("  %reg{} = load i64, i64* {}, align 8\n", reg, var_ptr));
                }
                Inst::Add(result_reg, lhs, rhs) => {
                    let lhs_str = match lhs {
                        Value::ImmInt(n) => n.to_string(),
                        Value::ImmFloat(f) => f.to_string(),
                        Value::Reg(r) => format!("%reg{}", r),
                        _ => panic!("Unsupported value type for add"),
                    };
                    let rhs_str = match rhs {
                        Value::ImmInt(n) => n.to_string(),
                        Value::ImmFloat(f) => f.to_string(),
                        Value::Reg(r) => format!("%reg{}", r),
                        _ => panic!("Unsupported value type for add"),
                    };
                    llvm_ir.push_str(&format!("  %reg{} = add i64 {}, {}\n", result_reg, lhs_str, rhs_str));
                }
                Inst::Sub(result_reg, lhs, rhs) => {
                    let lhs_str = match lhs {
                        Value::ImmInt(n) => n.to_string(),
                        Value::ImmFloat(f) => f.to_string(),
                        Value::Reg(r) => format!("%reg{}", r),
                        _ => panic!("Unsupported value type for sub"),
                    };
                    let rhs_str = match rhs {
                        Value::ImmInt(n) => n.to_string(),
                        Value::ImmFloat(f) => f.to_string(),
                        Value::Reg(r) => format!("%reg{}", r),
                        _ => panic!("Unsupported value type for sub"),
                    };
                    llvm_ir.push_str(&format!("  %reg{} = sub i64 {}, {}\n", result_reg, lhs_str, rhs_str));
                }
                Inst::Mul(result_reg, lhs, rhs) => {
                    let lhs_str = match lhs {
                        Value::ImmInt(n) => n.to_string(),
                        Value::ImmFloat(f) => f.to_string(),
                        Value::Reg(r) => format!("%reg{}", r),
                        _ => panic!("Unsupported value type for mul"),
                    };
                    let rhs_str = match rhs {
                        Value::ImmInt(n) => n.to_string(),
                        Value::ImmFloat(f) => f.to_string(),
                        Value::Reg(r) => format!("%reg{}", r),
                        _ => panic!("Unsupported value type for mul"),
                    };
                    llvm_ir.push_str(&format!("  %reg{} = mul i64 {}, {}\n", result_reg, lhs_str, rhs_str));
                }
                Inst::Div(result_reg, lhs, rhs) => {
                    let lhs_str = match lhs {
                        Value::ImmInt(n) => n.to_string(),
                        Value::ImmFloat(f) => f.to_string(),
                        Value::Reg(r) => format!("%reg{}", r),
                        _ => panic!("Unsupported value type for div"),
                    };
                    let rhs_str = match rhs {
                        Value::ImmInt(n) => n.to_string(),
                        Value::ImmFloat(f) => f.to_string(),
                        Value::Reg(r) => format!("%reg{}", r),
                        _ => panic!("Unsupported value type for div"),
                    };
                    llvm_ir.push_str(&format!("  %reg{} = sdiv i64 {}, {}\n", result_reg, lhs_str, rhs_str));
                }
                _ => panic!("Unsupported IR instruction"),
            }
        }
        llvm_ir.push_str("  ret i64 0\n"); // For simplicity, return 0 for now
        llvm_ir.push_str("}\n\n");
    }

    llvm_ir
}


