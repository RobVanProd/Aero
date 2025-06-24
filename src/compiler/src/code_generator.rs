use std::collections::HashMap;
use crate::ir::{Function, Inst, Value};

pub struct CodeGenerator {
    next_reg: u32,
    next_ptr: u32,
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            next_reg: 0,
            next_ptr: 0,
        }
    }

    fn fresh_reg(&mut self) -> String {
        let reg = format!("reg{}", self.next_reg);
        self.next_reg += 1;
        reg
    }

    fn fresh_ptr(&mut self) -> String {
        let ptr = format!("ptr{}", self.next_ptr);
        self.next_ptr += 1;
        ptr
    }

    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::ImmInt(n) => {
                // Convert int to double for storage
                let f = *n as f64;
                format!("0x{:016X}", f.to_bits())
            },
            Value::ImmFloat(f) => {
                // Format float as hexadecimal for LLVM IR
                format!("0x{:016X}", f.to_bits())
            },
            Value::Reg(r) => format!("%reg{}", r),
        }
    }

    pub fn generate_code(&mut self, ir_functions: HashMap<String, Function>) -> String {
        let mut llvm_ir = String::new();
        llvm_ir.push_str("; ModuleID = \"aero_compiler\"\n");
        llvm_ir.push_str("source_filename = \"aero_compiler\"\n");
        llvm_ir.push_str("target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"\n");
        llvm_ir.push_str("target triple = \"x86_64-pc-linux-gnu\"\n\n");

        for (func_name, func) in ir_functions {
            llvm_ir.push_str(&format!("define i32 @{}() {{\nentry:\n", func_name));

            let mut var_map = HashMap::new(); // Maps variable names to their alloca'd pointer registers
            
            // Initialize register counter to avoid conflicts with IR registers
            self.next_reg = func.next_reg;

            for inst in &func.body {
                match inst {
                    Inst::Alloca(ptr_reg, name) => {
                        llvm_ir.push_str(&format!("  %ptr{} = alloca double, align 8\n", 
                            match ptr_reg { Value::Reg(r) => *r, _ => panic!("Expected register for alloca") }));
                        var_map.insert(name.clone(), ptr_reg.clone());
                    }
                    Inst::Store(ptr_reg, value) => {
                        let val_str = self.value_to_string(value);
                        let ptr_str = match ptr_reg { 
                            Value::Reg(r) => format!("ptr{}", r), 
                            _ => panic!("Expected register for store pointer") 
                        };
                        llvm_ir.push_str(&format!("  store double {}, double* %{}, align 8\n", val_str, ptr_str));
                    }
                    Inst::Load(result_reg, ptr_reg) => {
                        let result_str = match result_reg { 
                            Value::Reg(r) => format!("reg{}", r), 
                            _ => panic!("Expected register for load result") 
                        };
                        let ptr_str = match ptr_reg { 
                            Value::Reg(r) => format!("ptr{}", r), 
                            _ => panic!("Expected register for load pointer") 
                        };
                        llvm_ir.push_str(&format!("  %{} = load double, double* %{}, align 8\n", result_str, ptr_str));
                    }
                    Inst::Add(result_reg, lhs, rhs) => {
                        let result_str = match result_reg { 
                            Value::Reg(r) => format!("reg{}", r), 
                            _ => panic!("Expected register for add result") 
                        };
                        let lhs_str = self.value_to_string(lhs);
                        let rhs_str = self.value_to_string(rhs);
                        llvm_ir.push_str(&format!("  %{} = add i64 {}, {}\n", result_str, lhs_str, rhs_str));
                    }
                    Inst::FAdd(result_reg, lhs, rhs) => {
                        let result_str = match result_reg { 
                            Value::Reg(r) => format!("reg{}", r), 
                            _ => panic!("Expected register for fadd result") 
                        };
                        let lhs_str = self.value_to_string(lhs);
                        let rhs_str = self.value_to_string(rhs);
                        llvm_ir.push_str(&format!("  %{} = fadd double {}, {}\n", result_str, lhs_str, rhs_str));
                    }
                    Inst::Sub(result_reg, lhs, rhs) => {
                        let result_str = match result_reg { 
                            Value::Reg(r) => format!("reg{}", r), 
                            _ => panic!("Expected register for sub result") 
                        };
                        let lhs_str = self.value_to_string(lhs);
                        let rhs_str = self.value_to_string(rhs);
                        llvm_ir.push_str(&format!("  %{} = sub i64 {}, {}\n", result_str, lhs_str, rhs_str));
                    }
                    Inst::FSub(result_reg, lhs, rhs) => {
                        let result_str = match result_reg { 
                            Value::Reg(r) => format!("reg{}", r), 
                            _ => panic!("Expected register for fsub result") 
                        };
                        let lhs_str = self.value_to_string(lhs);
                        let rhs_str = self.value_to_string(rhs);
                        llvm_ir.push_str(&format!("  %{} = fsub double {}, {}\n", result_str, lhs_str, rhs_str));
                    }
                    Inst::Mul(result_reg, lhs, rhs) => {
                        let result_str = match result_reg { 
                            Value::Reg(r) => format!("reg{}", r), 
                            _ => panic!("Expected register for mul result") 
                        };
                        let lhs_str = self.value_to_string(lhs);
                        let rhs_str = self.value_to_string(rhs);
                        llvm_ir.push_str(&format!("  %{} = mul i64 {}, {}\n", result_str, lhs_str, rhs_str));
                    }
                    Inst::FMul(result_reg, lhs, rhs) => {
                        let result_str = match result_reg { 
                            Value::Reg(r) => format!("reg{}", r), 
                            _ => panic!("Expected register for fmul result") 
                        };
                        let lhs_str = self.value_to_string(lhs);
                        let rhs_str = self.value_to_string(rhs);
                        llvm_ir.push_str(&format!("  %{} = fmul double {}, {}\n", result_str, lhs_str, rhs_str));
                    }
                    Inst::Div(result_reg, lhs, rhs) => {
                        let result_str = match result_reg { 
                            Value::Reg(r) => format!("reg{}", r), 
                            _ => panic!("Expected register for div result") 
                        };
                        let lhs_str = self.value_to_string(lhs);
                        let rhs_str = self.value_to_string(rhs);
                        llvm_ir.push_str(&format!("  %{} = sdiv i64 {}, {}\n", result_str, lhs_str, rhs_str));
                    }
                    Inst::FDiv(result_reg, lhs, rhs) => {
                        let result_str = match result_reg { 
                            Value::Reg(r) => format!("reg{}", r), 
                            _ => panic!("Expected register for fdiv result") 
                        };
                        let lhs_str = self.value_to_string(lhs);
                        let rhs_str = self.value_to_string(rhs);
                        llvm_ir.push_str(&format!("  %{} = fdiv double {}, {}\n", result_str, lhs_str, rhs_str));
                    }
                    Inst::FPToSI(result_reg, value) => {
                        let result_str = match result_reg { 
                            Value::Reg(r) => format!("reg{}", r), 
                            _ => panic!("Expected register for fptosi result") 
                        };
                        let val_str = self.value_to_string(value);
                        llvm_ir.push_str(&format!("  %{} = fptosi double {} to i64\n", result_str, val_str));
                    }
                    Inst::Return(value) => {
                        let val_str = self.value_to_string(value);
                        // Check if we need to convert float to int for return
                        match value {
                            Value::ImmFloat(_) | Value::Reg(_) => {
                                // For float values, convert to int first
                                let convert_reg = self.fresh_reg();
                                llvm_ir.push_str(&format!("  %{} = fptosi double {} to i32\n", convert_reg, val_str));
                                llvm_ir.push_str(&format!("  ret i32 %{}\n", convert_reg));
                            }
                            Value::ImmInt(_) => {
                                // For int values, truncate to i32
                                let trunc_reg = self.fresh_reg();
                                llvm_ir.push_str(&format!("  %{} = trunc i64 {} to i32\n", trunc_reg, val_str));
                                llvm_ir.push_str(&format!("  ret i32 %{}\n", trunc_reg));
                            }
                        }
                    }
                    Inst::SIToFP(result_reg, value) => {
                        let result_str = match result_reg { 
                            Value::Reg(r) => format!("reg{}", r), 
                            _ => panic!("Expected register for sitofp result") 
                        };
                        let val_str = match value {
                            Value::ImmInt(n) => n.to_string(), // Use integer format for sitofp source
                            Value::Reg(r) => format!("%reg{}", r),
                            _ => panic!("SIToFP expects integer input"),
                        };
                        llvm_ir.push_str(&format!("  %{} = sitofp i64 {} to double\n", result_str, val_str));
                    }
                }
            }
            // If no explicit return, return 0
            if !func.body.is_empty() && !func.body.iter().any(|inst| matches!(inst, Inst::Return(_))) {
                llvm_ir.push_str("  ret i32 0\n");
            }
            llvm_ir.push_str("}\n\n");
        }

        llvm_ir
    }
}

// Legacy function for backward compatibility
pub fn generate_code(ir_functions: HashMap<String, Function>) -> String {
    let mut generator = CodeGenerator::new();
    generator.generate_code(ir_functions)
}


