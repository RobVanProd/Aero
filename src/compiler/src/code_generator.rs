use crate::ir::{Function, Inst, Value};
use std::collections::HashMap;

type FunctionDef = (Vec<(String, String)>, Option<String>, Vec<Inst>);

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
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator {
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
                // Convert int to double for unified storage
                let f = *n as f64;
                format!("0x{:016X}", f.to_bits())
            }
            Value::ImmFloat(f) => {
                // Format float as hexadecimal for LLVM IR
                format!("0x{:016X}", f.to_bits())
            }
            Value::Reg(r) => format!("%reg{}", r),
            Value::ImmString(_) => {
                panic!("String value cannot be lowered as numeric LLVM `double`")
            }
        }
    }

    fn value_to_int_string(&self, value: &Value) -> String {
        match value {
            Value::ImmInt(n) => format!("{}", n),
            Value::ImmFloat(f) => format!("{}", *f as i64),
            Value::Reg(r) => format!("%reg{}", r),
            Value::ImmString(_) => panic!("String value cannot be lowered as LLVM integer"),
        }
    }

    fn value_to_i32_operand(&mut self, llvm_ir: &mut String, value: &Value) -> String {
        match value {
            Value::ImmInt(n) => n.to_string(),
            Value::ImmFloat(f) => (*f as i64).to_string(),
            Value::Reg(r) => {
                let tmp = self.fresh_reg();
                llvm_ir.push_str(&format!("  %{} = fptosi double %reg{} to i32\n", tmp, r));
                format!("%{}", tmp)
            }
            Value::ImmString(_) => panic!("String value cannot be lowered as i32 operand"),
        }
    }

    fn value_to_i64_operand(&mut self, llvm_ir: &mut String, value: &Value) -> String {
        match value {
            Value::ImmInt(n) => n.to_string(),
            Value::ImmFloat(f) => (*f as i64).to_string(),
            Value::Reg(r) => {
                let tmp = self.fresh_reg();
                llvm_ir.push_str(&format!("  %{} = fptosi double %reg{} to i64\n", tmp, r));
                format!("%{}", tmp)
            }
            Value::ImmString(_) => panic!("String value cannot be lowered as i64 operand"),
        }
    }

    fn type_to_llvm(&self, type_name: &str) -> &str {
        match type_name {
            "i32" => "i32",
            "i64" => "i64",
            "f32" => "float",
            "f64" => "double",
            "bool" => "i1",
            _ => "double", // Default fallback
        }
    }

    pub fn generate_code(&mut self, ir_functions: HashMap<String, Function>) -> String {
        let mut llvm_ir = String::new();
        llvm_ir.push_str("; ModuleID = \"aero_compiler\"\n");
        llvm_ir.push_str("source_filename = \"aero_compiler\"\n");
        llvm_ir.push_str("target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"\n");
        llvm_ir.push_str("target triple = \"x86_64-pc-linux-gnu\"\n\n");

        // Add printf declaration for I/O operations
        self.generate_printf_declaration(&mut llvm_ir);

        // First pass: collect function definitions from IR instructions
        let mut function_defs: HashMap<String, FunctionDef> = HashMap::new();

        for func in ir_functions.values() {
            for inst in &func.body {
                if let Inst::FunctionDef {
                    name,
                    parameters,
                    return_type,
                    body,
                } = inst
                {
                    function_defs.insert(
                        name.clone(),
                        (parameters.clone(), return_type.clone(), body.clone()),
                    );
                }
            }
        }

        // Generate function definitions
        for (func_name, func) in ir_functions {
            // Check if this function has a definition with parameters
            if let Some((parameters, return_type, body)) = function_defs.get(&func_name) {
                self.generate_function_definition(
                    &mut llvm_ir,
                    &func_name,
                    parameters,
                    return_type,
                    body,
                    func.next_reg,
                    &function_defs,
                );
            } else {
                // Legacy function without parameters (like main)
                llvm_ir.push_str(&format!("define i32 @{}() {{\nentry:\n", func_name));
                let empty_param_types: HashMap<String, String> = HashMap::new();
                self.generate_function_body(
                    &mut llvm_ir,
                    &func.body,
                    &empty_param_types,
                    "i32",
                    &function_defs,
                    func.next_reg,
                );
                llvm_ir.push_str("}\n\n");
            }
        }

        llvm_ir
    }

    fn generate_function_definition(
        &mut self,
        llvm_ir: &mut String,
        func_name: &str,
        parameters: &[(String, String)],
        return_type: &Option<String>,
        body: &[Inst],
        next_reg_seed: u32,
        function_defs: &HashMap<String, FunctionDef>,
    ) {
        // Generate function signature
        let return_llvm_type = if let Some(ret_type) = return_type {
            self.type_to_llvm(ret_type).to_string()
        } else {
            "void".to_string()
        };

        let mut param_str = String::new();
        for (i, (param_name, param_type)) in parameters.iter().enumerate() {
            if i > 0 {
                param_str.push_str(", ");
            }
            param_str.push_str(&format!(
                "{} %{}",
                self.type_to_llvm(param_type),
                param_name
            ));
        }

        llvm_ir.push_str(&format!(
            "define {} @{}({}) {{\nentry:\n",
            return_llvm_type, func_name, param_str
        ));

        let mut param_types = HashMap::new();
        for (param_name, param_type) in parameters {
            param_types.insert(param_name.clone(), param_type.clone());
        }

        self.generate_function_body(
            llvm_ir,
            body,
            &param_types,
            &return_llvm_type,
            function_defs,
            next_reg_seed,
        );
        llvm_ir.push_str("}\n\n");
    }

    fn generate_function_body(
        &mut self,
        llvm_ir: &mut String,
        instructions: &[Inst],
        param_types: &HashMap<String, String>,
        return_llvm_type: &str,
        function_defs: &HashMap<String, FunctionDef>,
        next_reg_seed: u32,
    ) {
        self.next_reg = next_reg_seed;

        for inst in instructions {
            match inst {
                Inst::Alloca(ptr_reg, name) => {
                    let ptr_id = match ptr_reg {
                        Value::Reg(r) => *r,
                        _ => panic!("Expected register for alloca"),
                    };
                    llvm_ir.push_str(&format!("  %ptr{} = alloca double, align 8\n", ptr_id));

                    if let Some(param_type) = param_types.get(name) {
                        match self.type_to_llvm(param_type) {
                            "double" => llvm_ir.push_str(&format!(
                                "  store double %{}, double* %ptr{}, align 8\n",
                                name, ptr_id
                            )),
                            "i32" => {
                                let tmp = self.fresh_reg();
                                llvm_ir.push_str(&format!(
                                    "  %{} = sitofp i32 %{} to double\n",
                                    tmp, name
                                ));
                                llvm_ir.push_str(&format!(
                                    "  store double %{}, double* %ptr{}, align 8\n",
                                    tmp, ptr_id
                                ));
                            }
                            "i64" => {
                                let tmp = self.fresh_reg();
                                llvm_ir.push_str(&format!(
                                    "  %{} = sitofp i64 %{} to double\n",
                                    tmp, name
                                ));
                                llvm_ir.push_str(&format!(
                                    "  store double %{}, double* %ptr{}, align 8\n",
                                    tmp, ptr_id
                                ));
                            }
                            "i1" => {
                                let tmp = self.fresh_reg();
                                llvm_ir.push_str(&format!(
                                    "  %{} = uitofp i1 %{} to double\n",
                                    tmp, name
                                ));
                                llvm_ir.push_str(&format!(
                                    "  store double %{}, double* %ptr{}, align 8\n",
                                    tmp, ptr_id
                                ));
                            }
                            _ => llvm_ir.push_str(&format!(
                                "  store double %{}, double* %ptr{}, align 8\n",
                                name, ptr_id
                            )),
                        }
                    }
                }
                Inst::Store(ptr_reg, value) => {
                    let val_str = self.value_to_string(value);
                    let ptr_str = match ptr_reg {
                        Value::Reg(r) => format!("ptr{}", r),
                        _ => panic!("Expected register for store pointer"),
                    };
                    llvm_ir.push_str(&format!(
                        "  store double {}, double* %{}, align 8\n",
                        val_str, ptr_str
                    ));
                }
                Inst::Load(result_reg, ptr_reg) => {
                    let result_str = match result_reg {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for load result"),
                    };
                    let ptr_str = match ptr_reg {
                        Value::Reg(r) => format!("ptr{}", r),
                        _ => panic!("Expected register for load pointer"),
                    };
                    llvm_ir.push_str(&format!(
                        "  %{} = load double, double* %{}, align 8\n",
                        result_str, ptr_str
                    ));
                }
                Inst::Add(result_reg, lhs, rhs) | Inst::FAdd(result_reg, lhs, rhs) => {
                    let result_str = match result_reg {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for add result"),
                    };
                    let lhs_str = self.value_to_string(lhs);
                    let rhs_str = self.value_to_string(rhs);
                    llvm_ir.push_str(&format!(
                        "  %{} = fadd double {}, {}\n",
                        result_str, lhs_str, rhs_str
                    ));
                }
                Inst::Sub(result_reg, lhs, rhs) | Inst::FSub(result_reg, lhs, rhs) => {
                    let result_str = match result_reg {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for sub result"),
                    };
                    let lhs_str = self.value_to_string(lhs);
                    let rhs_str = self.value_to_string(rhs);
                    llvm_ir.push_str(&format!(
                        "  %{} = fsub double {}, {}\n",
                        result_str, lhs_str, rhs_str
                    ));
                }
                Inst::Mul(result_reg, lhs, rhs) | Inst::FMul(result_reg, lhs, rhs) => {
                    let result_str = match result_reg {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for mul result"),
                    };
                    let lhs_str = self.value_to_string(lhs);
                    let rhs_str = self.value_to_string(rhs);
                    llvm_ir.push_str(&format!(
                        "  %{} = fmul double {}, {}\n",
                        result_str, lhs_str, rhs_str
                    ));
                }
                Inst::Div(result_reg, lhs, rhs) | Inst::FDiv(result_reg, lhs, rhs) => {
                    let result_str = match result_reg {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for div result"),
                    };
                    let lhs_str = self.value_to_string(lhs);
                    let rhs_str = self.value_to_string(rhs);
                    llvm_ir.push_str(&format!(
                        "  %{} = fdiv double {}, {}\n",
                        result_str, lhs_str, rhs_str
                    ));
                }
                Inst::FPToSI(result_reg, value) => {
                    let result_str = match result_reg {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for fptosi result"),
                    };
                    let val_str = self.value_to_string(value);
                    llvm_ir.push_str(&format!(
                        "  %{} = fptosi double {} to i64\n",
                        result_str, val_str
                    ));
                }
                Inst::Return(value) => self.emit_return(llvm_ir, value, return_llvm_type),
                Inst::SIToFP(result_reg, value) => {
                    let result_str = match result_reg {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for sitofp result"),
                    };
                    let val_str = self.value_to_string(value);
                    llvm_ir.push_str(&format!(
                        "  %{} = fadd double {}, 0x0000000000000000\n",
                        result_str, val_str
                    ));
                }
                Inst::FunctionDef { .. } => {}
                Inst::Call {
                    function,
                    arguments,
                    result,
                } => self.generate_function_call(llvm_ir, function, arguments, result, function_defs),
                Inst::Branch {
                    condition,
                    true_label,
                    false_label,
                } => self.generate_branch(llvm_ir, condition, true_label, false_label),
                Inst::Jump(label) => llvm_ir.push_str(&format!("  br label %{}\n", label)),
                Inst::Label(label) => llvm_ir.push_str(&format!("{}:\n", label)),
                Inst::ICmp {
                    op,
                    result,
                    left,
                    right,
                } => {
                    let result_str = match result {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for icmp result"),
                    };
                    let left_str = self.value_to_i32_operand(llvm_ir, left);
                    let right_str = self.value_to_i32_operand(llvm_ir, right);
                    llvm_ir.push_str(&format!(
                        "  %{} = icmp {} i32 {}, {}\n",
                        result_str, op, left_str, right_str
                    ));
                }
                Inst::FCmp {
                    op,
                    result,
                    left,
                    right,
                } => {
                    let result_str = match result {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for fcmp result"),
                    };
                    let left_str = self.value_to_string(left);
                    let right_str = self.value_to_string(right);
                    llvm_ir.push_str(&format!(
                        "  %{} = fcmp {} double {}, {}\n",
                        result_str, op, left_str, right_str
                    ));
                }
                Inst::Print {
                    format_string,
                    arguments,
                } => self.generate_print_call(llvm_ir, format_string, arguments, false),
                Inst::Println {
                    format_string,
                    arguments,
                } => self.generate_print_call(llvm_ir, format_string, arguments, true),
                Inst::And {
                    result,
                    left,
                    right,
                } => {
                    let result_str = match result {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for and result"),
                    };
                    let left_str = self.value_to_string(left);
                    let right_str = self.value_to_string(right);
                    llvm_ir.push_str(&format!(
                        "  %{} = and i1 {}, {}\n",
                        result_str, left_str, right_str
                    ));
                }
                Inst::Or {
                    result,
                    left,
                    right,
                } => {
                    let result_str = match result {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for or result"),
                    };
                    let left_str = self.value_to_string(left);
                    let right_str = self.value_to_string(right);
                    llvm_ir.push_str(&format!(
                        "  %{} = or i1 {}, {}\n",
                        result_str, left_str, right_str
                    ));
                }
                Inst::Not { result, operand } => {
                    let result_str = match result {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for not result"),
                    };
                    let operand_str = self.value_to_string(operand);
                    llvm_ir.push_str(&format!("  %{} = xor i1 {}, true\n", result_str, operand_str));
                }
                Inst::Neg { result, operand } => {
                    let result_str = match result {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for neg result"),
                    };
                    let operand_str = self.value_to_string(operand);
                    llvm_ir.push_str(&format!("  %{} = fsub double 0.0, {}\n", result_str, operand_str));
                }
                Inst::AllocaArray {
                    result,
                    elem_type,
                    count,
                } => {
                    let result_str = match result {
                        Value::Reg(r) => format!("ptr{}", r),
                        _ => panic!("Expected register for array alloca"),
                    };
                    llvm_ir.push_str(&format!(
                        "  %{} = alloca [{} x {}], align 8\n",
                        result_str, count, elem_type
                    ));
                }
                Inst::GetElementPtr {
                    result,
                    base,
                    index,
                    elem_type,
                } => {
                    let result_str = match result {
                        Value::Reg(r) => format!("ptr{}", r),
                        _ => panic!("Expected register for GEP result"),
                    };
                    let base_str = match base {
                        Value::Reg(r) => format!("ptr{}", r),
                        _ => panic!("Expected register for GEP base"),
                    };
                    let index_str = self.value_to_i64_operand(llvm_ir, index);
                    llvm_ir.push_str(&format!(
                        "  %{} = getelementptr inbounds {}, {}* %{}, i64 0, i64 {}\n",
                        result_str, elem_type, elem_type, base_str, index_str
                    ));
                }
                Inst::AllocaStruct {
                    result,
                    struct_type,
                } => {
                    let result_str = match result {
                        Value::Reg(r) => format!("ptr{}", r),
                        _ => panic!("Expected register for struct alloca"),
                    };
                    llvm_ir.push_str(&format!("  %{} = alloca %{}, align 8\n", result_str, struct_type));
                }
                Inst::GetFieldPtr {
                    result,
                    base,
                    field_index,
                    struct_type,
                } => {
                    let result_str = match result {
                        Value::Reg(r) => format!("ptr{}", r),
                        _ => panic!("Expected register for field GEP result"),
                    };
                    let base_str = match base {
                        Value::Reg(r) => format!("ptr{}", r),
                        _ => panic!("Expected register for field GEP base"),
                    };
                    llvm_ir.push_str(&format!(
                        "  %{} = getelementptr inbounds %{}, %{}* %{}, i32 0, i32 {}\n",
                        result_str, struct_type, struct_type, base_str, field_index
                    ));
                }
                _ => {}
            }
        }

        if !instructions.is_empty()
            && !instructions
                .iter()
                .any(|inst| matches!(inst, Inst::Return(_)))
        {
            match return_llvm_type {
                "void" => llvm_ir.push_str("  ret void\n"),
                "double" => llvm_ir.push_str("  ret double 0x0000000000000000\n"),
                "i1" => llvm_ir.push_str("  ret i1 false\n"),
                "i64" => llvm_ir.push_str("  ret i64 0\n"),
                _ => llvm_ir.push_str("  ret i32 0\n"),
            }
        }
    }

    fn generate_function_call(
        &mut self,
        llvm_ir: &mut String,
        function: &str,
        arguments: &[Value],
        result: &Option<Value>,
        function_defs: &HashMap<String, FunctionDef>,
    ) {
        let (param_defs, return_type) = if let Some((params, ret, _body)) = function_defs.get(function)
        {
            (params.clone(), ret.clone())
        } else {
            (Vec::new(), None)
        };

        let mut args = Vec::new();
        for (i, arg) in arguments.iter().enumerate() {
            let target_type = if let Some((_name, ty)) = param_defs.get(i) {
                self.type_to_llvm(ty).to_string()
            } else {
                "double".to_string()
            };
            let arg_val = self.cast_value_for_call_arg(llvm_ir, arg, &target_type);
            args.push(format!("{} {}", target_type, arg_val));
        }
        let args_str = args.join(", ");

        let return_llvm_type = if let Some(ret) = return_type {
            self.type_to_llvm(&ret).to_string()
        } else if result.is_some() {
            "double".to_string()
        } else {
            "void".to_string()
        };

        if let Some(result_reg) = result {
            let result_str = match result_reg {
                Value::Reg(r) => format!("reg{}", r),
                _ => panic!("Expected register for call result"),
            };

            match return_llvm_type.as_str() {
                "double" => llvm_ir.push_str(&format!(
                    "  %{} = call double @{}({})\n",
                    result_str, function, args_str
                )),
                "i32" => {
                    let call_reg = self.fresh_reg();
                    llvm_ir.push_str(&format!(
                        "  %{} = call i32 @{}({})\n",
                        call_reg, function, args_str
                    ));
                    llvm_ir.push_str(&format!(
                        "  %{} = sitofp i32 %{} to double\n",
                        result_str, call_reg
                    ));
                }
                "i64" => {
                    let call_reg = self.fresh_reg();
                    llvm_ir.push_str(&format!(
                        "  %{} = call i64 @{}({})\n",
                        call_reg, function, args_str
                    ));
                    llvm_ir.push_str(&format!(
                        "  %{} = sitofp i64 %{} to double\n",
                        result_str, call_reg
                    ));
                }
                "i1" => {
                    let call_reg = self.fresh_reg();
                    llvm_ir.push_str(&format!(
                        "  %{} = call i1 @{}({})\n",
                        call_reg, function, args_str
                    ));
                    llvm_ir.push_str(&format!(
                        "  %{} = uitofp i1 %{} to double\n",
                        result_str, call_reg
                    ));
                }
                "void" => {
                    llvm_ir.push_str(&format!("  call void @{}({})\n", function, args_str));
                    llvm_ir.push_str(&format!(
                        "  %{} = fadd double 0x0000000000000000, 0x0000000000000000\n",
                        result_str
                    ));
                }
                _ => llvm_ir.push_str(&format!(
                    "  %{} = call double @{}({})\n",
                    result_str, function, args_str
                )),
            }
        } else {
            llvm_ir.push_str(&format!(
                "  call {} @{}({})\n",
                return_llvm_type, function, args_str
            ));
        }
    }

    fn cast_value_for_call_arg(
        &mut self,
        llvm_ir: &mut String,
        value: &Value,
        target_type: &str,
    ) -> String {
        match target_type {
            "double" => self.value_to_string(value),
            "i32" => match value {
                Value::ImmInt(n) => n.to_string(),
                Value::ImmFloat(f) => (*f as i64).to_string(),
                Value::Reg(r) => {
                    let tmp = self.fresh_reg();
                    llvm_ir.push_str(&format!("  %{} = fptosi double %reg{} to i32\n", tmp, r));
                    format!("%{}", tmp)
                }
                Value::ImmString(_) => {
                    panic!("Cannot cast string argument to i32 in function call")
                }
            },
            "i64" => match value {
                Value::ImmInt(n) => n.to_string(),
                Value::ImmFloat(f) => (*f as i64).to_string(),
                Value::Reg(r) => {
                    let tmp = self.fresh_reg();
                    llvm_ir.push_str(&format!("  %{} = fptosi double %reg{} to i64\n", tmp, r));
                    format!("%{}", tmp)
                }
                Value::ImmString(_) => {
                    panic!("Cannot cast string argument to i64 in function call")
                }
            },
            "i1" => match value {
                Value::ImmInt(n) => {
                    if *n == 0 {
                        "false".to_string()
                    } else {
                        "true".to_string()
                    }
                }
                Value::ImmFloat(f) => {
                    if *f == 0.0 {
                        "false".to_string()
                    } else {
                        "true".to_string()
                    }
                }
                Value::Reg(r) => {
                    let tmp = self.fresh_reg();
                    llvm_ir.push_str(&format!(
                        "  %{} = fcmp one double %reg{}, 0x0000000000000000\n",
                        tmp, r
                    ));
                    format!("%{}", tmp)
                }
                Value::ImmString(_) => {
                    panic!("Cannot cast string argument to i1 in function call")
                }
            },
            _ => self.value_to_string(value),
        }
    }

    fn emit_return(&mut self, llvm_ir: &mut String, value: &Value, return_llvm_type: &str) {
        match return_llvm_type {
            "void" => llvm_ir.push_str("  ret void\n"),
            "double" => llvm_ir.push_str(&format!("  ret double {}\n", self.value_to_string(value))),
            "i64" => match value {
                Value::ImmInt(n) => llvm_ir.push_str(&format!("  ret i64 {}\n", n)),
                Value::ImmFloat(f) => llvm_ir.push_str(&format!("  ret i64 {}\n", *f as i64)),
                Value::Reg(r) => {
                    let tmp = self.fresh_reg();
                    llvm_ir.push_str(&format!("  %{} = fptosi double %reg{} to i64\n", tmp, r));
                    llvm_ir.push_str(&format!("  ret i64 %{}\n", tmp));
                }
                Value::ImmString(_) => panic!("Cannot return string value as i64"),
            },
            "i1" => match value {
                Value::ImmInt(n) => llvm_ir.push_str(&format!(
                    "  ret i1 {}\n",
                    if *n == 0 { "false" } else { "true" }
                )),
                Value::ImmFloat(f) => llvm_ir.push_str(&format!(
                    "  ret i1 {}\n",
                    if *f == 0.0 { "false" } else { "true" }
                )),
                Value::Reg(r) => {
                    let tmp = self.fresh_reg();
                    llvm_ir.push_str(&format!(
                        "  %{} = fcmp one double %reg{}, 0x0000000000000000\n",
                        tmp, r
                    ));
                    llvm_ir.push_str(&format!("  ret i1 %{}\n", tmp));
                }
                Value::ImmString(_) => panic!("Cannot return string value as i1"),
            },
            _ => match value {
                Value::ImmInt(n) => llvm_ir.push_str(&format!("  ret i32 {}\n", n)),
                Value::ImmFloat(f) => llvm_ir.push_str(&format!("  ret i32 {}\n", *f as i64)),
                Value::Reg(r) => {
                    let tmp = self.fresh_reg();
                    llvm_ir.push_str(&format!("  %{} = fptosi double %reg{} to i32\n", tmp, r));
                    llvm_ir.push_str(&format!("  ret i32 %{}\n", tmp));
                }
                Value::ImmString(_) => panic!("Cannot return string value as i32"),
            },
        }
    }
    fn generate_branch(
        &mut self,
        llvm_ir: &mut String,
        condition: &Value,
        true_label: &str,
        false_label: &str,
    ) {
        let cond_str = self.value_to_string(condition);

        // Check if condition is already a boolean (i1) or needs conversion
        match condition {
            Value::Reg(_) => {
                // Assume it's already a boolean from a comparison operation
                llvm_ir.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n",
                    cond_str, true_label, false_label
                ));
            }
            _ => {
                // Convert numeric value to boolean (non-zero is true)
                let bool_reg = self.fresh_reg();
                llvm_ir.push_str(&format!(
                    "  %{} = fcmp one double {}, 0x0000000000000000\n",
                    bool_reg, cond_str
                ));
                llvm_ir.push_str(&format!(
                    "  br i1 %{}, label %{}, label %{}\n",
                    bool_reg, true_label, false_label
                ));
            }
        }
    }

    #[allow(dead_code)]
    fn generate_phi_node(
        &mut self,
        llvm_ir: &mut String,
        result_reg: &str,
        incoming_values: &[(Value, String)],
    ) {
        // Generate phi node for variable updates in loops and control flow
        let mut phi_str = format!("  %{} = phi double ", result_reg);

        for (i, (value, label)) in incoming_values.iter().enumerate() {
            if i > 0 {
                phi_str.push_str(", ");
            }
            phi_str.push_str(&format!("[ {}, %{} ]", self.value_to_string(value), label));
        }

        phi_str.push('\n');
        llvm_ir.push_str(&phi_str);
    }

    #[allow(dead_code)]
    fn generate_loop_structure(
        &mut self,
        llvm_ir: &mut String,
        loop_header: &str,
        loop_body: &str,
        loop_exit: &str,
        condition: Option<&Value>,
    ) {
        // Generate basic loop structure with proper basic blocks

        // Jump to loop header
        llvm_ir.push_str(&format!("  br label %{}\n", loop_header));

        // Loop header block
        llvm_ir.push_str(&format!("{}:\n", loop_header));

        if let Some(cond) = condition {
            // Conditional loop (while/for)
            self.generate_branch(llvm_ir, cond, loop_body, loop_exit);
        } else {
            // Infinite loop
            llvm_ir.push_str(&format!("  br label %{}\n", loop_body));
        }

        // Loop body block
        llvm_ir.push_str(&format!("{}:\n", loop_body));
    }

    #[allow(dead_code)]
    fn generate_if_else_structure(
        &mut self,
        llvm_ir: &mut String,
        condition: &Value,
        then_label: &str,
        else_label: Option<&str>,
        merge_label: &str,
    ) {
        // Generate if-else structure with proper basic blocks
        let false_label = else_label.unwrap_or(merge_label);

        // Generate conditional branch
        self.generate_branch(llvm_ir, condition, then_label, false_label);

        // Then block
        llvm_ir.push_str(&format!("{}:\n", then_label));
    }

    fn generate_print_call(
        &mut self,
        llvm_ir: &mut String,
        format_string: &str,
        arguments: &[Value],
        is_println: bool,
    ) {
        // Convert Rust-style `{}` placeholders to `printf` specifiers from argument kinds.
        let processed_format = self.process_format_string_with_args(format_string, arguments);

        // Add newline for println
        let final_format = if is_println {
            format!("{}\n", processed_format)
        } else {
            processed_format
        };

        // Create format string as a local array (simplified approach)
        let format_len = final_format.len() + 1; // +1 for null terminator
        let format_const_reg = self.fresh_reg();

        // Allocate space for format string
        llvm_ir.push_str(&format!(
            "  %{} = alloca [{}  x i8], align 1\n",
            format_const_reg, format_len
        ));

        // Create the format string literal with proper escaping
        let escaped_format = self.escape_for_llvm(&final_format);
        llvm_ir.push_str(&format!(
            "  store [{}  x i8] c\"{}\\00\", [{}  x i8]* %{}, align 1\n",
            format_len, escaped_format, format_len, format_const_reg
        ));

        // Get pointer to format string
        let format_ptr = self.fresh_reg();
        llvm_ir.push_str(&format!(
            "  %{} = getelementptr inbounds [{}  x i8], [{}  x i8]* %{}, i64 0, i64 0\n",
            format_ptr, format_len, format_len, format_const_reg
        ));

        // Generate printf call
        let mut printf_args = format!("i8* %{}", format_ptr);

        for arg in arguments {
            match arg {
                Value::ImmString(s) => {
                    let arg_ptr = self.emit_stack_string_literal(llvm_ir, s);
                    printf_args.push_str(", i8* ");
                    printf_args.push_str(&arg_ptr);
                }
                _ => {
                    printf_args.push_str(", double ");
                    printf_args.push_str(&self.value_to_string(arg));
                }
            }
        }

        // Call printf
        llvm_ir.push_str(&format!("  call i32 @printf({})\n", printf_args));
    }

    fn escape_for_llvm(&self, input: &str) -> String {
        // Escape special characters for LLVM string literals
        input
            .replace("\\", "\\\\")
            .replace("\"", "\\\"")
            .replace("\n", "\\0A")
            .replace("\t", "\\09")
            .replace("\r", "\\0D")
    }

    fn process_format_string(&self, format_string: &str, arg_count: usize) -> String {
        // Keep legacy tests and helper calls by treating all placeholders as numeric.
        let numeric_args = vec![Value::ImmInt(0); arg_count];
        self.process_format_string_with_args(format_string, &numeric_args)
    }

    fn process_format_string_with_args(&self, format_string: &str, arguments: &[Value]) -> String {
        // Convert Rust-style {} placeholders to printf-style format specifiers.
        let mut result = String::new();
        let mut chars = format_string.chars().peekable();
        let mut placeholder_count = 0;

        while let Some(ch) = chars.next() {
            if ch == '{' {
                if let Some(&'}') = chars.peek() {
                    chars.next(); // consume '}'
                    if placeholder_count < arguments.len() {
                        let specifier = match arguments.get(placeholder_count) {
                            Some(Value::ImmString(_)) => "%s",
                            _ => "%g",
                        };
                        result.push_str(specifier);
                        placeholder_count += 1;
                    } else {
                        result.push_str("{}"); // Keep original if no corresponding argument
                    }
                } else {
                    result.push(ch);
                }
            } else if ch == '\\' {
                // Handle escape sequences
                if let Some(&next_ch) = chars.peek() {
                    match next_ch {
                        'n' => {
                            chars.next();
                            result.push_str("\\n");
                        }
                        't' => {
                            chars.next();
                            result.push_str("\\t");
                        }
                        'r' => {
                            chars.next();
                            result.push_str("\\r");
                        }
                        '\\' => {
                            chars.next();
                            result.push_str("\\\\");
                        }
                        '"' => {
                            chars.next();
                            result.push_str("\\\"");
                        }
                        _ => {
                            result.push(ch);
                        }
                    }
                } else {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    fn emit_stack_string_literal(&mut self, llvm_ir: &mut String, text: &str) -> String {
        let len = text.len() + 1; // +1 for null terminator
        let const_reg = self.fresh_reg();
        llvm_ir.push_str(&format!(
            "  %{} = alloca [{}  x i8], align 1\n",
            const_reg, len
        ));
        let escaped = self.escape_for_llvm(text);
        llvm_ir.push_str(&format!(
            "  store [{}  x i8] c\"{}\\00\", [{}  x i8]* %{}, align 1\n",
            len, escaped, len, const_reg
        ));
        let ptr_reg = self.fresh_reg();
        llvm_ir.push_str(&format!(
            "  %{} = getelementptr inbounds [{}  x i8], [{}  x i8]* %{}, i64 0, i64 0\n",
            ptr_reg, len, len, const_reg
        ));
        format!("%{}", ptr_reg)
    }

    fn generate_printf_declaration(&mut self, llvm_ir: &mut String) {
        // Generate printf declaration at module level
        llvm_ir.push_str("declare i32 @printf(i8*, ...)\n\n");
    }
}

// Legacy function for backward compatibility
pub fn generate_code(ir_functions: HashMap<String, Function>) -> String {
    let mut generator = CodeGenerator::new();
    generator.generate_code(ir_functions)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::approx_constant)]

    use super::*;
    use crate::ir::{Function, Inst, Value};
    use std::collections::HashMap;

    #[test]
    fn test_function_definition_generation() {
        let mut generator = CodeGenerator::new();

        // Create a simple function: fn add(a: i32, b: i32) -> i32 { return a + b; }
        let function = Function {
            name: "add".to_string(),
            body: vec![Inst::FunctionDef {
                name: "add".to_string(),
                parameters: vec![
                    ("a".to_string(), "i32".to_string()),
                    ("b".to_string(), "i32".to_string()),
                ],
                return_type: Some("i32".to_string()),
                body: vec![
                    Inst::Alloca(Value::Reg(0), "a".to_string()),
                    Inst::Alloca(Value::Reg(1), "b".to_string()),
                    Inst::Load(Value::Reg(2), Value::Reg(0)),
                    Inst::Load(Value::Reg(3), Value::Reg(1)),
                    Inst::Add(Value::Reg(4), Value::Reg(2), Value::Reg(3)),
                    Inst::Return(Value::Reg(4)),
                ],
            }],
            next_reg: 5,
            next_ptr: 2,
        };

        let mut functions = HashMap::new();
        functions.insert("add".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that function signature is correct
        assert!(llvm_ir.contains("define i32 @add(i32 %a, i32 %b)"));

        // Parameters are lowered to local double slots
        assert!(llvm_ir.contains("%ptr0 = alloca double"));
        assert!(llvm_ir.contains("%ptr1 = alloca double"));
        assert!(llvm_ir.contains("sitofp i32 %a to double"));
        assert!(llvm_ir.contains("sitofp i32 %b to double"));

        // Check that function has entry block
        assert!(llvm_ir.contains("entry:"));
    }

    #[test]
    fn test_function_call_generation() {
        let mut generator = CodeGenerator::new();

        // Create a function that calls another function
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Call {
                    function: "add".to_string(),
                    arguments: vec![Value::ImmInt(5), Value::ImmInt(3)],
                    result: Some(Value::Reg(0)),
                },
                Inst::Return(Value::Reg(0)),
            ],
            next_reg: 1,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that function call is generated
        assert!(llvm_ir.contains("call double @add"));
        assert!(llvm_ir.contains("double 0x4014000000000000")); // 5.0 in hex
        assert!(llvm_ir.contains("double 0x4008000000000000")); // 3.0 in hex
    }

    #[test]
    fn test_typed_function_call_uses_signature_and_converts_result() {
        let mut generator = CodeGenerator::new();

        let closure_like_function = Function {
            name: "__closure_0".to_string(),
            body: vec![Inst::FunctionDef {
                name: "__closure_0".to_string(),
                parameters: vec![
                    ("x".to_string(), "i32".to_string()),
                    ("y".to_string(), "i32".to_string()),
                ],
                return_type: Some("i32".to_string()),
                body: vec![
                    Inst::Alloca(Value::Reg(0), "x".to_string()),
                    Inst::Alloca(Value::Reg(1), "y".to_string()),
                    Inst::Load(Value::Reg(2), Value::Reg(0)),
                    Inst::Load(Value::Reg(3), Value::Reg(1)),
                    Inst::Add(Value::Reg(4), Value::Reg(2), Value::Reg(3)),
                    Inst::Return(Value::Reg(4)),
                ],
            }],
            next_reg: 5,
            next_ptr: 2,
        };

        let main = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Call {
                    function: "__closure_0".to_string(),
                    arguments: vec![Value::ImmInt(1), Value::ImmInt(2)],
                    result: Some(Value::Reg(0)),
                },
                Inst::Return(Value::Reg(0)),
            ],
            next_reg: 1,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("__closure_0".to_string(), closure_like_function);
        functions.insert("main".to_string(), main);

        let llvm_ir = generator.generate_code(functions);

        assert!(llvm_ir.contains("call i32 @__closure_0(i32 1, i32 2)"));
        assert!(llvm_ir.contains("sitofp i32 %"));
    }

    #[test]
    fn test_void_function_generation() {
        let mut generator = CodeGenerator::new();

        // Create a void function: fn print_hello() { }
        let function = Function {
            name: "print_hello".to_string(),
            body: vec![Inst::FunctionDef {
                name: "print_hello".to_string(),
                parameters: vec![],
                return_type: None,
                body: vec![Inst::Print {
                    format_string: "Hello, World!".to_string(),
                    arguments: vec![],
                }],
            }],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("print_hello".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that void function signature is correct
        assert!(llvm_ir.contains("define void @print_hello()"));

        // Check that print statement is generated with printf call
        assert!(llvm_ir.contains("call i32 @printf"));
    }

    #[test]
    fn test_print_generation() {
        let mut generator = CodeGenerator::new();

        // Create a function with print statement
        let function = Function {
            name: "main".to_string(),
            body: vec![Inst::Print {
                format_string: "Hello, World!".to_string(),
                arguments: vec![],
            }],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that printf declaration is present
        assert!(llvm_ir.contains("declare i32 @printf(i8*, ...)"));

        // Check that print call is generated
        assert!(llvm_ir.contains("call i32 @printf"));
        assert!(llvm_ir.contains("Hello, World!"));
    }

    #[test]
    fn test_println_generation() {
        let mut generator = CodeGenerator::new();

        // Create a function with println statement
        let function = Function {
            name: "main".to_string(),
            body: vec![Inst::Println {
                format_string: "Hello, World!".to_string(),
                arguments: vec![],
            }],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that printf declaration is present
        assert!(llvm_ir.contains("declare i32 @printf(i8*, ...)"));

        // Check that println call is generated with newline
        assert!(llvm_ir.contains("call i32 @printf"));
        assert!(llvm_ir.contains("Hello, World!\\0A"));
    }

    #[test]
    fn test_print_with_arguments() {
        let mut generator = CodeGenerator::new();

        // Create a function with print statement and arguments
        let function = Function {
            name: "main".to_string(),
            body: vec![Inst::Print {
                format_string: "Value: {}".to_string(),
                arguments: vec![Value::ImmInt(42)],
            }],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that format string is converted to printf style
        assert!(llvm_ir.contains("Value: %g"));

        // Check that argument is passed
        assert!(llvm_ir.contains("double 0x4045000000000000")); // 42.0 in hex
    }

    #[test]
    fn test_print_with_string_argument_uses_percent_s() {
        let mut generator = CodeGenerator::new();

        let function = Function {
            name: "main".to_string(),
            body: vec![Inst::Println {
                format_string: "Hello, {}".to_string(),
                arguments: vec![Value::ImmString("Aero".to_string())],
            }],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        assert!(llvm_ir.contains("Hello, %s\\0A"));
        assert!(llvm_ir.contains("call i32 @printf(i8*"));
        assert!(llvm_ir.contains(", i8* %"));
    }

    #[test]
    fn test_comparison_operations() {
        let mut generator = CodeGenerator::new();

        // Create a function with comparison operations
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::ICmp {
                    op: "eq".to_string(),
                    result: Value::Reg(0),
                    left: Value::ImmInt(5),
                    right: Value::ImmInt(5),
                },
                Inst::FCmp {
                    op: "olt".to_string(),
                    result: Value::Reg(1),
                    left: Value::ImmFloat(3.14),
                    right: Value::ImmFloat(4.0),
                },
            ],
            next_reg: 2,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that comparison operations are generated
        assert!(llvm_ir.contains("icmp eq i32"));
        assert!(llvm_ir.contains("fcmp olt double"));
    }

    #[test]
    fn test_logical_operations() {
        let mut generator = CodeGenerator::new();

        // Create a function with logical operations
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::And {
                    result: Value::Reg(0),
                    left: Value::Reg(1),
                    right: Value::Reg(2),
                },
                Inst::Or {
                    result: Value::Reg(3),
                    left: Value::Reg(4),
                    right: Value::Reg(5),
                },
                Inst::Not {
                    result: Value::Reg(6),
                    operand: Value::Reg(7),
                },
            ],
            next_reg: 8,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that logical operations are generated
        assert!(llvm_ir.contains("and i1"));
        assert!(llvm_ir.contains("or i1"));
        assert!(llvm_ir.contains("xor i1"));
    }

    #[test]
    fn test_unary_operations() {
        let mut generator = CodeGenerator::new();

        // Create a function with unary operations
        let function = Function {
            name: "main".to_string(),
            body: vec![Inst::Neg {
                result: Value::Reg(0),
                operand: Value::ImmFloat(5.0),
            }],
            next_reg: 1,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that negation operation is generated
        assert!(llvm_ir.contains("fsub double 0.0"));
    }

    #[test]
    fn test_format_string_processing() {
        let generator = CodeGenerator::new();

        // Test format string conversion
        let result = generator.process_format_string("Hello {}", 1);
        assert_eq!(result, "Hello %g");

        let result = generator.process_format_string("Values: {} and {}", 2);
        assert_eq!(result, "Values: %g and %g");

        let result = generator.process_format_string("No placeholders", 0);
        assert_eq!(result, "No placeholders");

        // Test with more placeholders than arguments
        let result = generator.process_format_string("Too many: {} {} {}", 1);
        assert_eq!(result, "Too many: %g {} {}");
    }

    #[test]
    fn test_escape_for_llvm() {
        let generator = CodeGenerator::new();

        // Test LLVM escaping
        let result = generator.escape_for_llvm("Hello\nWorld");
        assert_eq!(result, "Hello\\0AWorld");

        let result = generator.escape_for_llvm("Quote: \"test\"");
        assert_eq!(result, "Quote: \\\"test\\\"");

        let result = generator.escape_for_llvm("Tab\tSeparated");
        assert_eq!(result, "Tab\\09Separated");
    }

    #[test]
    fn test_complex_print_with_multiple_arguments() {
        let mut generator = CodeGenerator::new();

        // Create a function with complex print statement
        let function = Function {
            name: "main".to_string(),
            body: vec![Inst::Println {
                format_string: "Sum: {} + {} = {}".to_string(),
                arguments: vec![Value::ImmInt(5), Value::ImmInt(3), Value::ImmInt(8)],
            }],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that format string is converted correctly
        assert!(llvm_ir.contains("Sum: %g + %g = %g"));

        // Check that all arguments are passed
        assert!(llvm_ir.contains("double 0x4014000000000000")); // 5.0
        assert!(llvm_ir.contains("double 0x4008000000000000")); // 3.0
        assert!(llvm_ir.contains("double 0x4020000000000000")); // 8.0
    }

    #[test]
    fn test_type_to_llvm_conversion() {
        let generator = CodeGenerator::new();

        assert_eq!(generator.type_to_llvm("i32"), "i32");
        assert_eq!(generator.type_to_llvm("i64"), "i64");
        assert_eq!(generator.type_to_llvm("f32"), "float");
        assert_eq!(generator.type_to_llvm("f64"), "double");
        assert_eq!(generator.type_to_llvm("bool"), "i1");
        assert_eq!(generator.type_to_llvm("unknown"), "double"); // fallback
    }

    #[test]
    fn test_function_call_without_result() {
        let mut generator = CodeGenerator::new();

        // Create a function that calls a void function
        let function = Function {
            name: "main".to_string(),
            body: vec![Inst::Call {
                function: "print_hello".to_string(),
                arguments: vec![],
                result: None,
            }],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that void function call is generated
        assert!(llvm_ir.contains("call void @print_hello()"));
    }

    #[test]
    fn test_print_operation_generation() {
        let mut generator = CodeGenerator::new();

        // Create a function with print operation
        let function = Function {
            name: "test_print".to_string(),
            body: vec![Inst::Print {
                format_string: "Hello, {}!".to_string(),
                arguments: vec![Value::ImmInt(42)],
            }],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("test_print".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that printf call is generated
        assert!(llvm_ir.contains("call i32 @printf"));
        assert!(llvm_ir.contains("Hello, %g!")); // Format string should be processed
        assert!(llvm_ir.contains("getelementptr inbounds")); // String constant access
    }

    #[test]
    fn test_print_with_multiple_arguments() {
        let mut generator = CodeGenerator::new();

        // Create a function with print operation with multiple arguments
        let function = Function {
            name: "test_multi_print".to_string(),
            body: vec![Inst::Print {
                format_string: "Values: {}, {}, {}".to_string(),
                arguments: vec![Value::ImmInt(1), Value::ImmFloat(3.14), Value::Reg(5)],
            }],
            next_reg: 6,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("test_multi_print".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that printf call is generated with multiple arguments
        assert!(llvm_ir.contains("call i32 @printf"));
        assert!(llvm_ir.contains("Values: %g, %g, %g"));
        assert!(llvm_ir.contains("double 0x3FF0000000000000")); // 1.0 in hex
        assert!(llvm_ir.contains("double 0x40091EB851EB851F")); // 3.14 in hex
        assert!(llvm_ir.contains("double %reg5"));
    }

    #[test]
    fn test_println_vs_print_generation() {
        let mut generator = CodeGenerator::new();

        // Test print (without newline)
        let mut llvm_ir = String::new();
        generator.generate_print_call(&mut llvm_ir, "Hello", &[], false);
        assert!(llvm_ir.contains("Hello"));
        assert!(!llvm_ir.contains("\\n"));

        // Test println (with newline)
        let mut llvm_ir = String::new();
        generator.generate_print_call(&mut llvm_ir, "Hello", &[], true);
        assert!(llvm_ir.contains("Hello\\0A"));
    }

    #[test]
    fn test_enhanced_operations_generation() {
        let mut generator = CodeGenerator::new();

        // Create a comprehensive test with I/O, comparisons, logical, and unary operations
        let function = Function {
            name: "test_all_enhanced_ops".to_string(),
            body: vec![
                // Test comparison operations
                Inst::ICmp {
                    op: "eq".to_string(),
                    result: Value::Reg(0),
                    left: Value::ImmInt(5),
                    right: Value::ImmInt(5),
                },
                Inst::FCmp {
                    op: "ogt".to_string(),
                    result: Value::Reg(1),
                    left: Value::ImmFloat(3.14),
                    right: Value::ImmFloat(2.0),
                },
                // Test logical operations
                Inst::And {
                    result: Value::Reg(2),
                    left: Value::Reg(0),
                    right: Value::Reg(1),
                },
                Inst::Or {
                    result: Value::Reg(3),
                    left: Value::Reg(0),
                    right: Value::Reg(1),
                },
                Inst::Not {
                    result: Value::Reg(4),
                    operand: Value::Reg(0),
                },
                // Test unary operations
                Inst::Neg {
                    result: Value::Reg(5),
                    operand: Value::ImmFloat(-5.5),
                },
                // Test I/O operations
                Inst::Print {
                    format_string: "Results: {}, {}, {}".to_string(),
                    arguments: vec![Value::Reg(2), Value::Reg(3), Value::Reg(5)],
                },
                Inst::Println {
                    format_string: "Test completed!".to_string(),
                    arguments: vec![],
                },
                Inst::Return(Value::ImmInt(0)),
            ],
            next_reg: 6,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("test_all_enhanced_ops".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that all operations are generated
        assert!(llvm_ir.contains("icmp eq i32"));
        assert!(llvm_ir.contains("fcmp ogt double"));
        assert!(llvm_ir.contains("and i1"));
        assert!(llvm_ir.contains("or i1"));
        assert!(llvm_ir.contains("xor i1"));
        assert!(llvm_ir.contains("fsub double 0.0"));
        assert!(llvm_ir.contains("call i32 @printf"));
        assert!(llvm_ir.contains("Results: %g, %g, %g"));
        assert!(llvm_ir.contains("Test completed!\\0A"));
    }

    #[test]
    fn test_comprehensive_io_and_operations() {
        let mut generator = CodeGenerator::new();

        // Create a function with enhanced operations
        let function = Function {
            name: "test_enhanced_ops".to_string(),
            body: vec![
                // Comparison operations
                Inst::ICmp {
                    op: "eq".to_string(),
                    result: Value::Reg(0),
                    left: Value::ImmInt(5),
                    right: Value::ImmInt(5),
                },
                Inst::FCmp {
                    op: "ogt".to_string(),
                    result: Value::Reg(1),
                    left: Value::ImmFloat(3.14),
                    right: Value::ImmFloat(2.71),
                },
                // Logical operations
                Inst::And {
                    result: Value::Reg(2),
                    left: Value::Reg(0),
                    right: Value::Reg(1),
                },
                Inst::Or {
                    result: Value::Reg(3),
                    left: Value::Reg(0),
                    right: Value::Reg(1),
                },
                Inst::Not {
                    result: Value::Reg(4),
                    operand: Value::Reg(0),
                },
                // Unary operations
                Inst::Neg {
                    result: Value::Reg(5),
                    operand: Value::ImmFloat(42.0),
                },
            ],
            next_reg: 6,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("test_enhanced_ops".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that all operations are generated correctly
        assert!(llvm_ir.contains("icmp eq i32"));
        assert!(llvm_ir.contains("fcmp ogt double"));
        assert!(llvm_ir.contains("and i1"));
        assert!(llvm_ir.contains("or i1"));
        assert!(llvm_ir.contains("xor i1"));
        assert!(llvm_ir.contains("fsub double 0.0"));
    }

    #[test]
    fn test_escape_sequence_processing() {
        let generator = CodeGenerator::new();

        // Test various escape sequences
        let result =
            generator.process_format_string("Tab:\\t Newline:\\n Quote:\\\" Backslash:\\\\", 0);
        assert_eq!(result, "Tab:\\t Newline:\\n Quote:\\\" Backslash:\\\\");

        // Test carriage return
        let result = generator.process_format_string("CR:\\r", 0);
        assert_eq!(result, "CR:\\r");
    }

    #[test]
    fn test_print_with_no_arguments() {
        let mut generator = CodeGenerator::new();

        // Create a function with print operation with no arguments
        let function = Function {
            name: "test_no_args".to_string(),
            body: vec![Inst::Print {
                format_string: "Hello, World!".to_string(),
                arguments: vec![],
            }],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("test_no_args".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that printf call is generated with just format string
        assert!(llvm_ir.contains("call i32 @printf(i8*"));
        assert!(llvm_ir.contains("Hello, World!"));
    }
}

#[test]
fn test_legacy_function_without_definition() {
    let mut generator = CodeGenerator::new();

    // Create a legacy function without FunctionDef instruction (like main)
    let function = Function {
        name: "main".to_string(),
        body: vec![Inst::Return(Value::ImmInt(0))],
        next_reg: 0,
        next_ptr: 0,
    };

    let mut functions = HashMap::new();
    functions.insert("main".to_string(), function);

    let llvm_ir = generator.generate_code(functions);

    // Check that legacy function is handled correctly
    assert!(llvm_ir.contains("define i32 @main()"));
    assert!(llvm_ir.contains("entry:"));
    assert!(llvm_ir.contains("ret i32"));
}

#[test]
fn test_branch_generation() {
    let mut generator = CodeGenerator::new();

    // Create a function with conditional branch
    let function = Function {
        name: "test_branch".to_string(),
        body: vec![
            Inst::FCmp {
                op: "ogt".to_string(),
                result: Value::Reg(0),
                left: Value::ImmFloat(5.0),
                right: Value::ImmFloat(3.0),
            },
            Inst::Branch {
                condition: Value::Reg(0),
                true_label: "then_block".to_string(),
                false_label: "else_block".to_string(),
            },
            Inst::Label("then_block".to_string()),
            Inst::Return(Value::ImmInt(1)),
            Inst::Label("else_block".to_string()),
            Inst::Return(Value::ImmInt(0)),
        ],
        next_reg: 1,
        next_ptr: 0,
    };

    let mut functions = HashMap::new();
    functions.insert("test_branch".to_string(), function);

    let llvm_ir = generator.generate_code(functions);

    // Check that branch is generated correctly
    assert!(llvm_ir.contains("fcmp ogt double"));
    assert!(llvm_ir.contains("br i1 %reg0, label %then_block, label %else_block"));
    assert!(llvm_ir.contains("then_block:"));
    assert!(llvm_ir.contains("else_block:"));
}

#[test]
fn test_jump_and_label_generation() {
    let mut generator = CodeGenerator::new();

    // Create a function with unconditional jump
    let function = Function {
        name: "test_jump".to_string(),
        body: vec![
            Inst::Jump("target_label".to_string()),
            Inst::Label("target_label".to_string()),
            Inst::Return(Value::ImmInt(42)),
        ],
        next_reg: 0,
        next_ptr: 0,
    };

    let mut functions = HashMap::new();
    functions.insert("test_jump".to_string(), function);

    let llvm_ir = generator.generate_code(functions);

    // Check that jump and label are generated correctly
    assert!(llvm_ir.contains("br label %target_label"));
    assert!(llvm_ir.contains("target_label:"));
}

#[test]
fn test_comparison_operations() {
    let mut generator = CodeGenerator::new();

    // Create a function with various comparison operations
    let function = Function {
        name: "test_comparisons".to_string(),
        body: vec![
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(0),
                left: Value::ImmInt(5),
                right: Value::ImmInt(5),
            },
            Inst::FCmp {
                op: "olt".to_string(),
                result: Value::Reg(1),
                left: Value::ImmFloat(std::f64::consts::PI),
                right: Value::ImmFloat(std::f64::consts::E),
            },
            Inst::Return(Value::Reg(0)),
        ],
        next_reg: 2,
        next_ptr: 0,
    };

    let mut functions = HashMap::new();
    functions.insert("test_comparisons".to_string(), function);

    let llvm_ir = generator.generate_code(functions);

    // Check that comparisons are generated correctly
    assert!(llvm_ir.contains("icmp eq i32"));
    assert!(llvm_ir.contains("fcmp olt double"));
}

#[test]
fn test_logical_operations() {
    let mut generator = CodeGenerator::new();

    // Create a function with logical operations
    let function = Function {
        name: "test_logical".to_string(),
        body: vec![
            Inst::And {
                result: Value::Reg(0),
                left: Value::Reg(1),
                right: Value::Reg(2),
            },
            Inst::Or {
                result: Value::Reg(3),
                left: Value::Reg(4),
                right: Value::Reg(5),
            },
            Inst::Not {
                result: Value::Reg(6),
                operand: Value::Reg(7),
            },
            Inst::Return(Value::Reg(0)),
        ],
        next_reg: 8,
        next_ptr: 0,
    };

    let mut functions = HashMap::new();
    functions.insert("test_logical".to_string(), function);

    let llvm_ir = generator.generate_code(functions);

    // Check that logical operations are generated correctly
    assert!(llvm_ir.contains("and i1 %reg1, %reg2"));
    assert!(llvm_ir.contains("or i1 %reg4, %reg5"));
    assert!(llvm_ir.contains("xor i1 %reg7, true"));
}

#[test]
fn test_loop_structure_generation() {
    let mut generator = CodeGenerator::new();

    // Test the loop structure helper method
    let mut llvm_ir = String::new();
    let condition = Value::Reg(0);

    generator.generate_loop_structure(
        &mut llvm_ir,
        "loop_header",
        "loop_body",
        "loop_exit",
        Some(&condition),
    );

    // Check that loop structure is generated correctly
    assert!(llvm_ir.contains("br label %loop_header"));
    assert!(llvm_ir.contains("loop_header:"));
    assert!(llvm_ir.contains("loop_body:"));
    assert!(llvm_ir.contains("br i1 %reg0, label %loop_body, label %loop_exit"));
}

#[test]
fn test_infinite_loop_structure() {
    let mut generator = CodeGenerator::new();

    // Test infinite loop structure
    let mut llvm_ir = String::new();

    generator.generate_loop_structure(&mut llvm_ir, "loop_header", "loop_body", "loop_exit", None);

    // Check that infinite loop structure is generated correctly
    assert!(llvm_ir.contains("br label %loop_header"));
    assert!(llvm_ir.contains("loop_header:"));
    assert!(llvm_ir.contains("br label %loop_body"));
    assert!(llvm_ir.contains("loop_body:"));
}

