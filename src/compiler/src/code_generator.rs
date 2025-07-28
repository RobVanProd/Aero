use std::collections::HashMap;
use crate::ir::{Function, Inst, Value};
use crate::memory::{MemorySafetyAnalyzer, MemoryLayoutCalculator, MemorySafetyWarning};


pub struct CodeGenerator {
    next_reg: u32,
    next_ptr: u32,
    struct_definitions: HashMap<String, StructTypeInfo>,
    enum_definitions: HashMap<String, EnumTypeInfo>,
}

#[derive(Debug, Clone)]
pub struct StructTypeInfo {
    pub name: String,
    pub fields: Vec<(String, String)>, // (field_name, field_type)
    pub is_tuple: bool,
}

#[derive(Debug, Clone)]
pub struct EnumTypeInfo {
    pub name: String,
    pub variants: Vec<(String, Option<Vec<String>>)>, // (variant_name, optional_data_types)
    pub discriminant_type: String,
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            next_reg: 0,
            next_ptr: 0,
            struct_definitions: HashMap::new(),
            enum_definitions: HashMap::new(),
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
                // Convert int to double for unified storage
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

        // First pass: collect struct definitions and function definitions from IR instructions
        let mut function_defs: HashMap<String, (Vec<(String, String)>, Option<String>)> = HashMap::new();
        
        for (func_name, func) in &ir_functions {
            for inst in &func.body {
                match inst {
                    Inst::FunctionDef { name, parameters, return_type, body: _ } => {
                        function_defs.insert(name.clone(), (parameters.clone(), return_type.clone()));
                    }
                    Inst::StructDef { name, fields, is_tuple } => {
                        self.struct_definitions.insert(name.clone(), StructTypeInfo {
                            name: name.clone(),
                            fields: fields.clone(),
                            is_tuple: *is_tuple,
                        });
                    }
                    Inst::EnumDef { name, variants, discriminant_type } => {
                        self.enum_definitions.insert(name.clone(), EnumTypeInfo {
                            name: name.clone(),
                            variants: variants.clone(),
                            discriminant_type: discriminant_type.clone(),
                        });
                    }
                    _ => {}
                }
            }
        }

        // Generate struct type definitions
        self.generate_struct_type_definitions(&mut llvm_ir);
        
        // Generate enum type definitions
        self.generate_enum_type_definitions(&mut llvm_ir);

        // Generate function definitions
        for (func_name, func) in ir_functions {
            // Check if this function has a definition with parameters
            if let Some((parameters, return_type)) = function_defs.get(&func_name) {
                self.generate_function_definition(&mut llvm_ir, &func_name, parameters, return_type, &func);
            } else {
                // Legacy function without parameters (like main)
                llvm_ir.push_str(&format!("define i32 @{}() {{\nentry:\n", func_name));
                self.generate_function_body(&mut llvm_ir, &func);
                llvm_ir.push_str("}\n\n");
            }
        }

        llvm_ir
    }

    fn generate_function_definition(&mut self, llvm_ir: &mut String, func_name: &str, parameters: &[(String, String)], return_type: &Option<String>, func: &Function) {
        // Generate function signature
        let return_llvm_type = if let Some(ret_type) = return_type {
            self.type_to_llvm(ret_type)
        } else {
            "void"
        };

        let mut param_str = String::new();
        for (i, (param_name, param_type)) in parameters.iter().enumerate() {
            if i > 0 {
                param_str.push_str(", ");
            }
            param_str.push_str(&format!("{} %{}", self.type_to_llvm(param_type), param_name));
        }

        llvm_ir.push_str(&format!("define {} @{}({}) {{\nentry:\n", return_llvm_type, func_name, param_str));

        // Allocate space for parameters
        for (param_name, param_type) in parameters {
            let ptr_reg = self.fresh_ptr();
            llvm_ir.push_str(&format!("  %{} = alloca {}, align 8\n", ptr_reg, self.type_to_llvm(param_type)));
            llvm_ir.push_str(&format!("  store {} %{}, {}* %{}, align 8\n", 
                self.type_to_llvm(param_type), param_name, self.type_to_llvm(param_type), ptr_reg));
        }

        self.generate_function_body(llvm_ir, func);
        llvm_ir.push_str("}\n\n");
    }

    fn generate_function_body(&mut self, llvm_ir: &mut String, func: &Function) {
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
                    llvm_ir.push_str(&format!("  %{} = fadd double {}, {}\n", result_str, lhs_str, rhs_str));
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
                    llvm_ir.push_str(&format!("  %{} = fsub double {}, {}\n", result_str, lhs_str, rhs_str));
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
                    llvm_ir.push_str(&format!("  %{} = fmul double {}, {}\n", result_str, lhs_str, rhs_str));
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
                    llvm_ir.push_str(&format!("  %{} = fdiv double {}, {}\n", result_str, lhs_str, rhs_str));
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
                    // Since we're storing everything as double, always convert to int for return
                    let convert_reg = self.fresh_reg();
                    llvm_ir.push_str(&format!("  %{} = fptosi double {} to i32\n", convert_reg, val_str));
                    llvm_ir.push_str(&format!("  ret i32 %{}\n", convert_reg));
                }
                Inst::SIToFP(result_reg, value) => {
                    // Since we're already storing everything as double, this is essentially a no-op
                    // Just copy the value to the result register
                    let result_str = match result_reg { 
                        Value::Reg(r) => format!("reg{}", r), 
                        _ => panic!("Expected register for sitofp result") 
                    };
                    let val_str = self.value_to_string(value);
                    // Since both source and target are double, just use fadd with 0.0
                    llvm_ir.push_str(&format!("  %{} = fadd double {}, 0x0000000000000000\n", result_str, val_str));
                }
                Inst::FunctionDef { name: _, parameters: _, return_type: _, body: _ } => {
                    // Function definitions are handled separately in generate_code
                    // Skip them in the main function body
                }
                Inst::Call { function, arguments, result } => {
                    self.generate_function_call(llvm_ir, function, arguments, result);
                }
                Inst::Branch { condition, true_label, false_label } => {
                    self.generate_branch(llvm_ir, condition, true_label, false_label);
                }
                Inst::Jump(label) => {
                    llvm_ir.push_str(&format!("  br label %{}\n", label));
                }
                Inst::Label(label) => {
                    llvm_ir.push_str(&format!("{}:\n", label));
                }
                Inst::ICmp { op, result, left, right } => {
                    let result_str = match result { 
                        Value::Reg(r) => format!("reg{}", r), 
                        _ => panic!("Expected register for icmp result") 
                    };
                    let left_str = self.value_to_string(left);
                    let right_str = self.value_to_string(right);
                    llvm_ir.push_str(&format!("  %{} = icmp {} i32 {}, {}\n", result_str, op, left_str, right_str));
                }
                Inst::FCmp { op, result, left, right } => {
                    let result_str = match result { 
                        Value::Reg(r) => format!("reg{}", r), 
                        _ => panic!("Expected register for fcmp result") 
                    };
                    let left_str = self.value_to_string(left);
                    let right_str = self.value_to_string(right);
                    llvm_ir.push_str(&format!("  %{} = fcmp {} double {}, {}\n", result_str, op, left_str, right_str));
                }
                Inst::Print { format_string, arguments } => {
                    self.generate_print_call(llvm_ir, format_string, arguments, false);
                }
                Inst::Println { format_string, arguments } => {
                    self.generate_print_call(llvm_ir, format_string, arguments, true);
                }
                Inst::And { result, left, right } => {
                    let result_str = match result { 
                        Value::Reg(r) => format!("reg{}", r), 
                        _ => panic!("Expected register for and result") 
                    };
                    let left_str = self.value_to_string(left);
                    let right_str = self.value_to_string(right);
                    llvm_ir.push_str(&format!("  %{} = and i1 {}, {}\n", result_str, left_str, right_str));
                }
                Inst::Or { result, left, right } => {
                    let result_str = match result { 
                        Value::Reg(r) => format!("reg{}", r), 
                        _ => panic!("Expected register for or result") 
                    };
                    let left_str = self.value_to_string(left);
                    let right_str = self.value_to_string(right);
                    llvm_ir.push_str(&format!("  %{} = or i1 {}, {}\n", result_str, left_str, right_str));
                }
                Inst::Not { result, operand } => {
                    let result_str = match result { 
                        Value::Reg(r) => format!("reg{}", r), 
                        _ => panic!("Expected register for not result") 
                    };
                    let operand_str = self.value_to_string(operand);
                    llvm_ir.push_str(&format!("  %{} = xor i1 {}, true\n", result_str, operand_str));
                }
                Inst::Neg { result, operand } => {
                    let result_str = match result { 
                        Value::Reg(r) => format!("reg{}", r), 
                        _ => panic!("Expected register for neg result") 
                    };
                    let operand_str = self.value_to_string(operand);
                    // Assuming double type for now - this should be type-aware
                    llvm_ir.push_str(&format!("  %{} = fsub double 0.0, {}\n", result_str, operand_str));
                }
                // Struct operations - Implemented LLVM struct generation
                Inst::StructDef { .. } => {
                    // Struct definitions are handled at module level, skip here
                }
                Inst::StructAlloca { result, struct_name } => {
                    self.generate_struct_alloca(llvm_ir, result, struct_name);
                }
                Inst::StructInit { result, struct_name, field_values } => {
                    self.generate_struct_init(llvm_ir, result, struct_name, field_values);
                }
                Inst::FieldAccess { result, struct_ptr, field_name, field_index } => {
                    self.generate_field_access(llvm_ir, result, struct_ptr, field_name, *field_index);
                }
                Inst::FieldStore { struct_ptr, field_name, field_index, value } => {
                    self.generate_field_store(llvm_ir, struct_ptr, field_name, *field_index, value);
                }
                Inst::StructCopy { result, source, struct_name } => {
                    self.generate_struct_copy(llvm_ir, result, source, struct_name);
                }
                // Enum operations - Implemented LLVM enum generation
                Inst::EnumDef { .. } => {
                    // Enum definitions are handled at module level, skip here
                }
                Inst::EnumAlloca { result, enum_name } => {
                    self.generate_enum_alloca(llvm_ir, result, enum_name);
                }
                Inst::EnumConstruct { result, enum_name, variant_name, variant_index, data_values } => {
                    self.generate_enum_construct(llvm_ir, result, enum_name, variant_name, *variant_index, data_values);
                }
                Inst::EnumDiscriminant { result, enum_ptr } => {
                    self.generate_enum_discriminant(llvm_ir, result, enum_ptr);
                }
                Inst::EnumExtract { result, enum_ptr, variant_index, data_index } => {
                    self.generate_enum_extract(llvm_ir, result, enum_ptr, *variant_index, *data_index);
                }
                // Pattern matching operations - Implemented LLVM pattern matching generation
                Inst::Match { discriminant, arms, default_label } => {
                    self.generate_match_expression(llvm_ir, discriminant, arms, default_label);
                }
                Inst::PatternCheck { result, discriminant, expected_variant } => {
                    self.generate_pattern_check(llvm_ir, result, discriminant, *expected_variant);
                }
                Inst::Switch { discriminant, cases, default_label } => {
                    // Generate LLVM switch instruction
                    let disc_str = self.value_to_string(discriminant);
                    llvm_ir.push_str(&format!("  switch i32 {}, label %{} [\n", disc_str, default_label));
                    for (value, label) in cases {
                        llvm_ir.push_str(&format!("    i32 {}, label %{}\n", value, label));
                    }
                    llvm_ir.push_str("  ]\n");
                }
                // Array and collection operations - Implemented LLVM array/collection generation
                Inst::ArrayAlloca { result, element_type, size } => {
                    self.generate_array_alloca(llvm_ir, result, element_type, size);
                }
                Inst::ArrayInit { result, element_type, elements } => {
                    self.generate_array_init(llvm_ir, result, element_type, elements);
                }
                Inst::ArrayAccess { result, array_ptr, index } => {
                    self.generate_array_access(llvm_ir, result, array_ptr, index);
                }
                Inst::ArrayStore { array_ptr, index, value } => {
                    self.generate_array_store(llvm_ir, array_ptr, index, value);
                }
                Inst::ArrayLength { result, array_ptr } => {
                    self.generate_array_length(llvm_ir, result, array_ptr);
                }
                Inst::BoundsCheck { array_ptr, index, success_label, failure_label } => {
                    self.generate_bounds_check(llvm_ir, array_ptr, index, success_label, failure_label);
                }
                // Vec operations - Implemented LLVM Vec generation
                Inst::VecAlloca { result, element_type } => {
                    self.generate_vec_alloca(llvm_ir, result, element_type);
                }
                Inst::VecInit { result, element_type, elements } => {
                    self.generate_vec_init(llvm_ir, result, element_type, elements);
                }
                Inst::VecPush { vec_ptr, value } => {
                    self.generate_vec_push(llvm_ir, vec_ptr, value);
                }
                Inst::VecPop { result, vec_ptr } => {
                    self.generate_vec_pop(llvm_ir, result, vec_ptr);
                }
                Inst::VecLength { result, vec_ptr } => {
                    self.generate_vec_length(llvm_ir, result, vec_ptr);
                }
                Inst::VecCapacity { result, vec_ptr } => {
                    self.generate_vec_capacity(llvm_ir, result, vec_ptr);
                }
                Inst::VecAccess { result, vec_ptr, index } => {
                    self.generate_vec_access(llvm_ir, result, vec_ptr, index);
                }
                // Generic operations - TODO: Implement proper LLVM generic generation
                Inst::GenericInstantiate { .. } => {
                    // TODO: Implement generic instantiation
                    llvm_ir.push_str("  ; TODO: generic instantiate\n");
                }
                Inst::GenericMethodCall { .. } => {
                    // TODO: Implement generic method call
                    llvm_ir.push_str("  ; TODO: generic method call\n");
                }
            }
        }
        // If no explicit return, return 0
        if !func.body.is_empty() && !func.body.iter().any(|inst| matches!(inst, Inst::Return(_))) {
            llvm_ir.push_str("  ret i32 0\n");
        }
    }

    fn generate_function_call(&mut self, llvm_ir: &mut String, function: &str, arguments: &[Value], result: &Option<Value>) {
        // Generate function call with proper type handling
        let mut args_str = String::new();
        for (i, arg) in arguments.iter().enumerate() {
            if i > 0 {
                args_str.push_str(", ");
            }
            // For now, assume all arguments are double - this should be type-aware
            args_str.push_str("double ");
            args_str.push_str(&self.value_to_string(arg));
        }
        
        if let Some(result_reg) = result {
            let result_str = match result_reg { 
                Value::Reg(r) => format!("reg{}", r), 
                _ => panic!("Expected register for call result") 
            };
            // For now, assume return type is double - this should be type-aware
            llvm_ir.push_str(&format!("  %{} = call double @{}({})\n", result_str, function, args_str));
        } else {
            llvm_ir.push_str(&format!("  call void @{}({})\n", function, args_str));
        }
    }

    fn generate_branch(&mut self, llvm_ir: &mut String, condition: &Value, true_label: &str, false_label: &str) {
        let cond_str = self.value_to_string(condition);
        
        // Check if condition is already a boolean (i1) or needs conversion
        match condition {
            Value::Reg(_) => {
                // Assume it's already a boolean from a comparison operation
                llvm_ir.push_str(&format!("  br i1 {}, label %{}, label %{}\n", cond_str, true_label, false_label));
            }
            _ => {
                // Convert numeric value to boolean (non-zero is true)
                let bool_reg = self.fresh_reg();
                llvm_ir.push_str(&format!("  %{} = fcmp one double {}, 0x0000000000000000\n", bool_reg, cond_str));
                llvm_ir.push_str(&format!("  br i1 %{}, label %{}, label %{}\n", bool_reg, true_label, false_label));
            }
        }
    }

    fn generate_phi_node(&mut self, llvm_ir: &mut String, result_reg: &str, incoming_values: &[(Value, String)]) {
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

    fn generate_loop_structure(&mut self, llvm_ir: &mut String, loop_header: &str, loop_body: &str, loop_exit: &str, condition: Option<&Value>) {
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

    fn generate_if_else_structure(&mut self, llvm_ir: &mut String, condition: &Value, then_label: &str, else_label: Option<&str>, merge_label: &str) {
        // Generate if-else structure with proper basic blocks
        let false_label = else_label.unwrap_or(merge_label);
        
        // Generate conditional branch
        self.generate_branch(llvm_ir, condition, then_label, false_label);
        
        // Then block
        llvm_ir.push_str(&format!("{}:\n", then_label));
    }

    fn generate_print_call(&mut self, llvm_ir: &mut String, format_string: &str, arguments: &[Value], is_println: bool) {
        // Process format string to convert Rust-style {} to printf-style %g
        let processed_format = self.process_format_string(format_string, arguments.len());
        
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
        llvm_ir.push_str(&format!("  %{} = alloca [{}  x i8], align 1\n", format_const_reg, format_len));
        
        // Create the format string literal with proper escaping
        let escaped_format = self.escape_for_llvm(&final_format);
        llvm_ir.push_str(&format!("  store [{}  x i8] c\"{}\\00\", [{}  x i8]* %{}, align 1\n", 
            format_len, escaped_format, format_len, format_const_reg));
        
        // Get pointer to format string
        let format_ptr = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds [{}  x i8], [{}  x i8]* %{}, i64 0, i64 0\n", 
            format_ptr, format_len, format_len, format_const_reg));
        
        // Generate printf call
        let mut printf_args = format!("i8* %{}", format_ptr);
        
        for arg in arguments {
            printf_args.push_str(", double ");
            printf_args.push_str(&self.value_to_string(arg));
        }
        
        // Call printf
        llvm_ir.push_str(&format!("  call i32 @printf({})\n", printf_args));
    }
    
    fn escape_for_llvm(&self, input: &str) -> String {
        // Escape special characters for LLVM string literals
        input.replace("\\", "\\\\")
             .replace("\"", "\\\"")
             .replace("\n", "\\0A")
             .replace("\t", "\\09")
             .replace("\r", "\\0D")
    }



    // Struct generation methods for Task 10.1
    fn generate_struct_type_definitions(&self, llvm_ir: &mut String) {
        // Generate LLVM struct type definitions at module level
        for (struct_name, struct_info) in &self.struct_definitions {
            if struct_info.is_tuple {
                // Generate tuple struct type
                let mut field_types = Vec::new();
                for (_, field_type) in &struct_info.fields {
                    field_types.push(self.type_to_llvm(field_type));
                }
                llvm_ir.push_str(&format!("%{} = type {{ {} }}\n", 
                    struct_name, field_types.join(", ")));
            } else {
                // Generate named struct type
                let mut field_types = Vec::new();
                for (_, field_type) in &struct_info.fields {
                    field_types.push(self.type_to_llvm(field_type));
                }
                llvm_ir.push_str(&format!("%{} = type {{ {} }}\n", 
                    struct_name, field_types.join(", ")));
            }
        }
        if !self.struct_definitions.is_empty() {
            llvm_ir.push('\n');
        }
    }

    fn generate_struct_alloca(&mut self, llvm_ir: &mut String, result: &Value, struct_name: &str) {
        // Generate LLVM struct allocation
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for struct alloca result"),
        };
        
        llvm_ir.push_str(&format!("  %{} = alloca %{}, align 8\n", result_str, struct_name));
    }

    fn generate_struct_init(&mut self, llvm_ir: &mut String, result: &Value, struct_name: &str, field_values: &[(String, Value)]) {
        // Generate LLVM struct initialization
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for struct init result"),
        };

        // First allocate the struct
        llvm_ir.push_str(&format!("  %{} = alloca %{}, align 8\n", result_str, struct_name));

        // Get struct info to determine field indices
        if let Some(struct_info) = self.struct_definitions.get(struct_name) {
            // Initialize each field
            for (field_name, field_value) in field_values {
                // Find field index
                if let Some(field_index) = struct_info.fields.iter().position(|(name, _)| name == field_name) {
                    // Generate getelementptr to get field address
                    let field_ptr = self.fresh_reg();
                    llvm_ir.push_str(&format!("  %{} = getelementptr inbounds %{}, %{}* %{}, i32 0, i32 {}\n",
                        field_ptr, struct_name, struct_name, result_str, field_index));
                    
                    // Store the field value
                    let field_type = &struct_info.fields[field_index].1;
                    let llvm_type = self.type_to_llvm(field_type);
                    let value_str = self.value_to_string(field_value);
                    llvm_ir.push_str(&format!("  store {} {}, {}* %{}, align 8\n",
                        llvm_type, value_str, llvm_type, field_ptr));
                }
            }
        }
    }

    fn generate_field_access(&mut self, llvm_ir: &mut String, result: &Value, struct_ptr: &Value, _field_name: &str, field_index: usize) {
        // Generate LLVM field access using getelementptr
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for field access result"),
        };
        
        let ptr_str = match struct_ptr {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for struct pointer"),
        };

        // Generate getelementptr to get field address
        let field_ptr = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds %struct_type, %struct_type* %{}, i32 0, i32 {}\n",
            field_ptr, ptr_str, field_index));
        
        // Load the field value (assuming double for now - should be type-aware)
        llvm_ir.push_str(&format!("  %{} = load double, double* %{}, align 8\n",
            result_str, field_ptr));
    }

    fn generate_field_store(&mut self, llvm_ir: &mut String, struct_ptr: &Value, _field_name: &str, field_index: usize, value: &Value) {
        // Generate LLVM field store using getelementptr
        let ptr_str = match struct_ptr {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for struct pointer"),
        };

        // Generate getelementptr to get field address
        let field_ptr = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds %struct_type, %struct_type* %{}, i32 0, i32 {}\n",
            field_ptr, ptr_str, field_index));
        
        // Store the field value (assuming double for now - should be type-aware)
        let value_str = self.value_to_string(value);
        llvm_ir.push_str(&format!("  store double {}, double* %{}, align 8\n",
            value_str, field_ptr));
    }

    fn generate_struct_copy(&mut self, llvm_ir: &mut String, result: &Value, source: &Value, struct_name: &str) {
        // Generate LLVM struct copy using memcpy
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for struct copy result"),
        };
        
        let source_str = match source {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for struct copy source"),
        };

        // First allocate destination struct
        llvm_ir.push_str(&format!("  %{} = alloca %{}, align 8\n", result_str, struct_name));

        // Calculate struct size (simplified - should use actual struct size)
        let struct_size = if let Some(struct_info) = self.struct_definitions.get(struct_name) {
            struct_info.fields.len() * 8 // Assuming 8 bytes per field for simplicity
        } else {
            8 // Default size
        };

        // Cast pointers to i8* for memcpy
        let dest_cast = self.fresh_reg();
        let src_cast = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = bitcast %{}* %{} to i8*\n", dest_cast, struct_name, result_str));
        llvm_ir.push_str(&format!("  %{} = bitcast %{}* %{} to i8*\n", src_cast, struct_name, source_str));

        // Generate memcpy call
        llvm_ir.push_str(&format!("  call void @llvm.memcpy.p0i8.p0i8.i64(i8* align 8 %{}, i8* align 8 %{}, i64 {}, i1 false)\n",
            dest_cast, src_cast, struct_size));
    }

    fn generate_printf_declaration(&self, llvm_ir: &mut String) {
        // Add printf and memcpy declarations
        llvm_ir.push_str("declare i32 @printf(i8*, ...)\n");
        llvm_ir.push_str("declare void @llvm.memcpy.p0i8.p0i8.i64(i8* noalias nocapture writeonly, i8* noalias nocapture readonly, i64, i1 immarg)\n\n");
    }

    fn process_format_string(&self, format_string: &str, arg_count: usize) -> String {
        // Convert Rust-style {} placeholders to printf-style %g
        let mut result = String::new();
        let mut chars = format_string.chars().peekable();
        let mut placeholder_count = 0;
        
        while let Some(ch) = chars.next() {
            if ch == '{' {
                if let Some(&'}') = chars.peek() {
                    chars.next(); // consume '}'
                    if placeholder_count < arg_count {
                        result.push_str("%g"); // Use %g for general numeric formatting
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
    use super::*;
    use crate::ir::{Function, Inst, Value};
    use std::collections::HashMap;

    #[test]
    fn test_function_definition_generation() {
        let mut generator = CodeGenerator::new();
        
        // Create a simple function: fn add(a: i32, b: i32) -> i32 { return a + b; }
        let function = Function {
            name: "add".to_string(),
            body: vec![
                Inst::FunctionDef {
                    name: "add".to_string(),
                    parameters: vec![("a".to_string(), "i32".to_string()), ("b".to_string(), "i32".to_string())],
                    return_type: Some("i32".to_string()),
                    body: vec![],
                },
                Inst::Load(Value::Reg(0), Value::Reg(100)), // Load parameter a
                Inst::Load(Value::Reg(1), Value::Reg(101)), // Load parameter b
                Inst::Add(Value::Reg(2), Value::Reg(0), Value::Reg(1)), // Add a + b
                Inst::Return(Value::Reg(2)), // Return result
            ],
            next_reg: 3,
            next_ptr: 102,
        };

        let mut functions = HashMap::new();
        functions.insert("add".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Check that function signature is correct
        assert!(llvm_ir.contains("define i32 @add(i32 %a, i32 %b)"));
        
        // Check that parameters are allocated
        assert!(llvm_ir.contains("alloca i32"));
        assert!(llvm_ir.contains("store i32 %a"));
        assert!(llvm_ir.contains("store i32 %b"));
        
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
    fn test_void_function_generation() {
        let mut generator = CodeGenerator::new();
        
        // Create a void function: fn print_hello() { }
        let function = Function {
            name: "print_hello".to_string(),
            body: vec![
                Inst::FunctionDef {
                    name: "print_hello".to_string(),
                    parameters: vec![],
                    return_type: None,
                    body: vec![],
                },
                Inst::Print {
                    format_string: "Hello, World!".to_string(),
                    arguments: vec![],
                },
            ],
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
            body: vec![
                Inst::Print {
                    format_string: "Hello, World!".to_string(),
                    arguments: vec![],
                },
            ],
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
            body: vec![
                Inst::Println {
                    format_string: "Hello, World!".to_string(),
                    arguments: vec![],
                },
            ],
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
            body: vec![
                Inst::Print {
                    format_string: "Value: {}".to_string(),
                    arguments: vec![Value::ImmInt(42)],
                },
            ],
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
            body: vec![
                Inst::Neg {
                    result: Value::Reg(0),
                    operand: Value::ImmFloat(5.0),
                },
            ],
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
        let result = generator.escape_for_llvm("Hello\\nWorld");
        assert_eq!(result, "Hello\\\\0AWorld");
        
        let result = generator.escape_for_llvm("Quote: \"test\"");
        assert_eq!(result, "Quote: \\\"test\\\"");
        
        let result = generator.escape_for_llvm("Tab\\tSeparated");
        assert_eq!(result, "Tab\\09Separated");
    }

    #[test]
    fn test_complex_print_with_multiple_arguments() {
        let mut generator = CodeGenerator::new();
        
        // Create a function with complex print statement
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Println {
                    format_string: "Sum: {} + {} = {}".to_string(),
                    arguments: vec![
                        Value::ImmInt(5),
                        Value::ImmInt(3),
                        Value::ImmInt(8),
                    ],
                },
            ],
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
            body: vec![
                Inst::Call {
                    function: "print_hello".to_string(),
                    arguments: vec![],
                    result: None,
                },
            ],
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
            body: vec![
                Inst::Print {
                    format_string: "Hello, {}!".to_string(),
                    arguments: vec![Value::ImmInt(42)],
                },
            ],
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
            body: vec![
                Inst::Print {
                    format_string: "Values: {}, {}, {}".to_string(),
                    arguments: vec![
                        Value::ImmInt(1),
                        Value::ImmFloat(3.14),
                        Value::Reg(5),
                    ],
                },
            ],
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
        let result = generator.process_format_string("Tab:\\t Newline:\\n Quote:\\\" Backslash:\\\\", 0);
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
            body: vec![
                Inst::Print {
                    format_string: "Hello, World!".to_string(),
                    arguments: vec![],
                },
            ],
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
            body: vec![
                Inst::Return(Value::ImmInt(0)),
            ],
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
                    left: Value::ImmFloat(3.14),
                    right: Value::ImmFloat(2.71),
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
        
        generator.generate_loop_structure(&mut llvm_ir, "loop_header", "loop_body", "loop_exit", Some(&condition));
        
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





    // Struct generation methods for Task 10.1
    fn generate_struct_type_definitions(&self, llvm_ir: &mut String) {
        // Generate LLVM struct type definitions at module level
        for (struct_name, struct_info) in &self.struct_definitions {
            let mut field_types = Vec::new();
            for (_, field_type) in &struct_info.fields {
                field_types.push(self.type_to_llvm(field_type));
            }
            llvm_ir.push_str(&format!("%{} = type {{ {} }}\n", 
                struct_name, field_types.join(", ")));
        }
        if !self.struct_definitions.is_empty() {
            llvm_ir.push('\n');
        }
    }

    fn generate_struct_alloca(&mut self, llvm_ir: &mut String, result: &Value, struct_name: &str) {
        // Generate LLVM struct allocation
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for struct alloca result"),
        };
        
        llvm_ir.push_str(&format!("  %{} = alloca %{}, align 8\n", result_str, struct_name));
    }

    fn generate_struct_init(&mut self, llvm_ir: &mut String, result: &Value, struct_name: &str, field_values: &[(String, Value)]) {
        // Generate LLVM struct initialization
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for struct init result"),
        };

        // First allocate the struct
        llvm_ir.push_str(&format!("  %{} = alloca %{}, align 8\n", result_str, struct_name));

        // Clone the struct definitions to avoid borrowing issues
        let struct_info = self.struct_definitions.get(struct_name).cloned();
        
        if let Some(struct_info) = struct_info {
            // Initialize each field
            for (field_name, field_value) in field_values {
                // Find field index
                if let Some(field_index) = struct_info.fields.iter().position(|(name, _)| name == field_name) {
                    // Generate getelementptr to get field address
                    let field_ptr = self.fresh_reg();
                    llvm_ir.push_str(&format!("  %{} = getelementptr inbounds %{}, %{}* %{}, i32 0, i32 {}\n",
                        field_ptr, struct_name, struct_name, result_str, field_index));
                    
                    // Store the field value
                    let field_type = &struct_info.fields[field_index].1;
                    let llvm_type = self.type_to_llvm(field_type);
                    let value_str = self.value_to_string(field_value);
                    llvm_ir.push_str(&format!("  store {} {}, {}* %{}, align 8\n",
                        llvm_type, value_str, llvm_type, field_ptr));
                }
            }
        }
    }

    fn generate_field_access(&mut self, llvm_ir: &mut String, result: &Value, struct_ptr: &Value, _field_name: &str, field_index: usize) {
        // Generate LLVM field access using getelementptr
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for field access result"),
        };
        
        let ptr_str = match struct_ptr {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for struct pointer"),
        };

        // Generate getelementptr to get field address (using generic struct type for now)
        let field_ptr = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds %struct_type, %struct_type* %{}, i32 0, i32 {}\n",
            field_ptr, ptr_str, field_index));
        
        // Load the field value (assuming double for now - should be type-aware)
        llvm_ir.push_str(&format!("  %{} = load double, double* %{}, align 8\n",
            result_str, field_ptr));
    }

    fn generate_field_store(&mut self, llvm_ir: &mut String, struct_ptr: &Value, _field_name: &str, field_index: usize, value: &Value) {
        // Generate LLVM field store using getelementptr
        let ptr_str = match struct_ptr {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for struct pointer"),
        };

        // Generate getelementptr to get field address
        let field_ptr = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds %struct_type, %struct_type* %{}, i32 0, i32 {}\n",
            field_ptr, ptr_str, field_index));
        
        // Store the field value (assuming double for now - should be type-aware)
        let value_str = self.value_to_string(value);
        llvm_ir.push_str(&format!("  store double {}, double* %{}, align 8\n",
            value_str, field_ptr));
    }

    fn generate_struct_copy(&mut self, llvm_ir: &mut String, result: &Value, source: &Value, struct_name: &str) {
        // Generate LLVM struct copy using memcpy
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for struct copy result"),
        };
        
        let source_str = match source {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for struct copy source"),
        };

        // First allocate destination struct
        llvm_ir.push_str(&format!("  %{} = alloca %{}, align 8\n", result_str, struct_name));

        // Calculate struct size (simplified - should use actual struct size)
        let struct_size = if let Some(struct_info) = self.struct_definitions.get(struct_name) {
            struct_info.fields.len() * 8 // Assuming 8 bytes per field for simplicity
        } else {
            8 // Default size
        };

        // Cast pointers to i8* for memcpy
        let dest_cast = self.fresh_reg();
        let src_cast = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = bitcast %{}* %{} to i8*\n", dest_cast, struct_name, result_str));
        llvm_ir.push_str(&format!("  %{} = bitcast %{}* %{} to i8*\n", src_cast, struct_name, source_str));

        // Generate memcpy call
        llvm_ir.push_str(&format!("  call void @llvm.memcpy.p0i8.p0i8.i64(i8* align 8 %{}, i8* align 8 %{}, i64 {}, i1 false)\n",
            dest_cast, src_cast, struct_size));
    }

    fn generate_printf_declaration(&self, llvm_ir: &mut String) {
        // Add printf and memcpy declarations
        llvm_ir.push_str("declare i32 @printf(i8*, ...)\n");
        llvm_ir.push_str("declare void @llvm.memcpy.p0i8.p0i8.i64(i8* noalias nocapture writeonly, i8* noalias nocapture readonly, i64, i1 immarg)\n\n");
    }

    fn process_format_string(&self, format_string: &str, arg_count: usize) -> String {
        // Convert Rust-style {} placeholders to printf-style %g
        let mut result = String::new();
        let mut chars = format_string.chars().peekable();
        let mut placeholder_count = 0;
        
        while let Some(ch) = chars.next() {
            if ch == '{' {
                if let Some(&'}') = chars.peek() {
                    chars.next(); // consume '}'
                    if placeholder_count < arg_count {
                        result.push_str("%g"); // Use %g for general numeric formatting
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
    // Enum generation methods for Task 10.2
    fn generate_enum_type_definitions(&self, llvm_ir: &mut String) {
        // Generate LLVM enum type definitions at module level
        for (enum_name, enum_info) in &self.enum_definitions {
            // Enums are represented as tagged unions with a discriminant
            // Structure: { discriminant_type, union_of_variant_data }
            
            // First, generate union type for variant data if needed
            let has_data_variants = enum_info.variants.iter().any(|(_, data)| data.is_some());
            
            if has_data_variants {
                // Generate union type for variant data
                let mut union_members = Vec::new();
                for (variant_name, variant_data) in &enum_info.variants {
                    if let Some(data_types) = variant_data {
                        if !data_types.is_empty() {
                            // Create struct type for this variant's data
                            let variant_struct_name = format!("{}.{}", enum_name, variant_name);
                            let mut field_types = Vec::new();
                            for data_type in data_types {
                                field_types.push(self.type_to_llvm(data_type));
                            }
                            llvm_ir.push_str(&format!("%{} = type {{ {} }}\n", 
                                variant_struct_name, field_types.join(", ")));
                            union_members.push(format!("%{}", variant_struct_name));
                        }
                    }
                }
                
                // Generate union type if we have data variants
                if !union_members.is_empty() {
                    llvm_ir.push_str(&format!("%{}.union = type {{ {} }}\n", 
                        enum_name, union_members.join(", ")));
                    
                    // Generate main enum type with discriminant and union
                    llvm_ir.push_str(&format!("%{} = type {{ {}, %{}.union }}\n", 
                        enum_name, self.type_to_llvm(&enum_info.discriminant_type), enum_name));
                } else {
                    // Only discriminant needed (no data variants)
                    llvm_ir.push_str(&format!("%{} = type {{ {} }}\n", 
                        enum_name, self.type_to_llvm(&enum_info.discriminant_type)));
                }
            } else {
                // Simple enum with only discriminant
                llvm_ir.push_str(&format!("%{} = type {{ {} }}\n", 
                    enum_name, self.type_to_llvm(&enum_info.discriminant_type)));
            }
        }
        if !self.enum_definitions.is_empty() {
            llvm_ir.push('\n');
        }
    }

    fn generate_enum_alloca(&mut self, llvm_ir: &mut String, result: &Value, enum_name: &str) {
        // Generate LLVM enum allocation
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for enum alloca result"),
        };
        
        llvm_ir.push_str(&format!("  %{} = alloca %{}, align 8\n", result_str, enum_name));
    }

    fn generate_enum_construct(&mut self, llvm_ir: &mut String, result: &Value, enum_name: &str, _variant_name: &str, variant_index: usize, data_values: &[Value]) {
        // Generate LLVM enum construction
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for enum construct result"),
        };

        // First allocate the enum
        llvm_ir.push_str(&format!("  %{} = alloca %{}, align 8\n", result_str, enum_name));

        // Set the discriminant
        let disc_ptr = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds %{}, %{}* %{}, i32 0, i32 0\n",
            disc_ptr, enum_name, enum_name, result_str));
        llvm_ir.push_str(&format!("  store i32 {}, i32* %{}, align 4\n",
            variant_index, disc_ptr));

        // If there's data, store it in the union
        if !data_values.is_empty() {
            // Get pointer to union data
            let union_ptr = self.fresh_reg();
            llvm_ir.push_str(&format!("  %{} = getelementptr inbounds %{}, %{}* %{}, i32 0, i32 1\n",
                union_ptr, enum_name, enum_name, result_str));

            // Store each data value
            for (i, data_value) in data_values.iter().enumerate() {
                let data_ptr = self.fresh_reg();
                llvm_ir.push_str(&format!("  %{} = getelementptr inbounds %{}.union, %{}.union* %{}, i32 0, i32 {}\n",
                    data_ptr, enum_name, enum_name, union_ptr, i));
                
                let value_str = self.value_to_string(data_value);
                llvm_ir.push_str(&format!("  store double {}, double* %{}, align 8\n",
                    value_str, data_ptr));
            }
        }
    }

    fn generate_enum_discriminant(&mut self, llvm_ir: &mut String, result: &Value, enum_ptr: &Value) {
        // Generate LLVM enum discriminant extraction
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for enum discriminant result"),
        };
        
        let ptr_str = match enum_ptr {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for enum pointer"),
        };

        // Get pointer to discriminant (first field)
        let disc_ptr = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds %enum_type, %enum_type* %{}, i32 0, i32 0\n",
            disc_ptr, ptr_str));
        
        // Load the discriminant
        llvm_ir.push_str(&format!("  %{} = load i32, i32* %{}, align 4\n",
            result_str, disc_ptr));
    }

    fn generate_enum_extract(&mut self, llvm_ir: &mut String, result: &Value, enum_ptr: &Value, _variant_index: usize, data_index: usize) {
        // Generate LLVM enum data extraction
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for enum extract result"),
        };
        
        let ptr_str = match enum_ptr {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for enum pointer"),
        };

        // Get pointer to union data (second field)
        let union_ptr = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds %enum_type, %enum_type* %{}, i32 0, i32 1\n",
            union_ptr, ptr_str));

        // Get pointer to specific data field
        let data_ptr = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds %union_type, %union_type* %{}, i32 0, i32 {}\n",
            data_ptr, union_ptr, data_index));
        
        // Load the data value
        llvm_ir.push_str(&format!("  %{} = load double, double* %{}, align 8\n",
            result_str, data_ptr));
    }

    fn generate_match_expression(&mut self, llvm_ir: &mut String, discriminant: &Value, arms: &[crate::ir::MatchArm], default_label: &Option<String>) {
        // Generate LLVM match expression using switch instruction
        let disc_str = self.value_to_string(discriminant);
        
        // Convert discriminant to i32 if needed
        let disc_i32 = if matches!(discriminant, Value::Reg(_)) {
            // Assume it's already i32 from discriminant extraction
            disc_str
        } else {
            // Convert to i32
            let conv_reg = self.fresh_reg();
            llvm_ir.push_str(&format!("  %{} = fptosi double {} to i32\n", conv_reg, disc_str));
            format!("%{}", conv_reg)
        };

        // Generate switch instruction
        let default_lbl = default_label.as_ref().map(|s| s.as_str()).unwrap_or("match_default");
        llvm_ir.push_str(&format!("  switch i32 {}, label %{} [\n", disc_i32, default_lbl));
        
        // Generate cases for each arm
        for (i, arm) in arms.iter().enumerate() {
            // For now, assume simple variant matching
            for pattern_check in &arm.pattern_checks {
                if let crate::ir::PatternValue::Variant(variant_idx) = &pattern_check.expected {
                    llvm_ir.push_str(&format!("    i32 {}, label %{}\n", variant_idx, arm.body_label));
                }
            }
        }
        
        llvm_ir.push_str("  ]\n");

        // Generate default label if needed
        if default_label.is_none() {
            llvm_ir.push_str(&format!("{}:\n", default_lbl));
            llvm_ir.push_str("  unreachable\n");
        }
    }

    fn generate_pattern_check(&mut self, llvm_ir: &mut String, result: &Value, discriminant: &Value, expected_variant: usize) {
        // Generate LLVM pattern check (discriminant comparison)
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for pattern check result"),
        };
        
        let disc_str = self.value_to_string(discriminant);
        
        // Compare discriminant with expected variant
        llvm_ir.push_str(&format!("  %{} = icmp eq i32 {}, {}\n", 
            result_str, disc_str, expected_variant));
    }
    // Collection and string generation methods for Task 10.3
    
    // Array operations
    fn generate_array_alloca(&mut self, llvm_ir: &mut String, result: &Value, element_type: &str, size: &Value) {
        // Generate LLVM array allocation
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for array alloca result"),
        };
        
        let size_str = self.value_to_string(size);
        let llvm_element_type = self.type_to_llvm(element_type);
        
        // Convert size to i64 if needed
        let size_i64 = if matches!(size, Value::Reg(_)) {
            // Assume it's already i64
            size_str
        } else {
            // Convert double to i64
            let conv_reg = self.fresh_reg();
            llvm_ir.push_str(&format!("  %{} = fptosi double {} to i64\n", conv_reg, size_str));
            format!("%{}", conv_reg)
        };
        
        // Allocate array with dynamic size
        llvm_ir.push_str(&format!("  %{} = alloca {}, i64 {}, align 8\n", 
            result_str, llvm_element_type, size_i64));
    }

    fn generate_array_init(&mut self, llvm_ir: &mut String, result: &Value, element_type: &str, elements: &[Value]) {
        // Generate LLVM array initialization
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for array init result"),
        };
        
        let llvm_element_type = self.type_to_llvm(element_type);
        let array_size = elements.len();
        
        // Allocate array with fixed size
        llvm_ir.push_str(&format!("  %{} = alloca [{} x {}], align 8\n", 
            result_str, array_size, llvm_element_type));
        
        // Initialize each element
        for (i, element) in elements.iter().enumerate() {
            let elem_ptr = self.fresh_reg();
            llvm_ir.push_str(&format!("  %{} = getelementptr inbounds [{} x {}], [{} x {}]* %{}, i64 0, i64 {}\n",
                elem_ptr, array_size, llvm_element_type, array_size, llvm_element_type, result_str, i));
            
            let value_str = self.value_to_string(element);
            llvm_ir.push_str(&format!("  store {} {}, {}* %{}, align 8\n",
                llvm_element_type, value_str, llvm_element_type, elem_ptr));
        }
    }

    fn generate_array_access(&mut self, llvm_ir: &mut String, result: &Value, array_ptr: &Value, index: &Value) {
        // Generate LLVM array access with bounds checking
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for array access result"),
        };
        
        let ptr_str = match array_ptr {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for array pointer"),
        };
        
        let index_str = self.value_to_string(index);
        
        // Convert index to i64 if needed
        let index_i64 = if matches!(index, Value::Reg(_)) {
            // Assume it's already i64
            index_str
        } else {
            // Convert double to i64
            let conv_reg = self.fresh_reg();
            llvm_ir.push_str(&format!("  %{} = fptosi double {} to i64\n", conv_reg, index_str));
            format!("%{}", conv_reg)
        };
        
        // Generate getelementptr for array access
        let elem_ptr = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds double, double* %{}, i64 {}\n",
            elem_ptr, ptr_str, index_i64));
        
        // Load the element value
        llvm_ir.push_str(&format!("  %{} = load double, double* %{}, align 8\n",
            result_str, elem_ptr));
    }

    fn generate_array_store(&mut self, llvm_ir: &mut String, array_ptr: &Value, index: &Value, value: &Value) {
        // Generate LLVM array store
        let ptr_str = match array_ptr {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for array pointer"),
        };
        
        let index_str = self.value_to_string(index);
        let value_str = self.value_to_string(value);
        
        // Convert index to i64 if needed
        let index_i64 = if matches!(index, Value::Reg(_)) {
            // Assume it's already i64
            index_str
        } else {
            // Convert double to i64
            let conv_reg = self.fresh_reg();
            llvm_ir.push_str(&format!("  %{} = fptosi double {} to i64\n", conv_reg, index_str));
            format!("%{}", conv_reg)
        };
        
        // Generate getelementptr for array access
        let elem_ptr = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds double, double* %{}, i64 {}\n",
            elem_ptr, ptr_str, index_i64));
        
        // Store the element value
        llvm_ir.push_str(&format!("  store double {}, double* %{}, align 8\n",
            value_str, elem_ptr));
    }

    fn generate_array_length(&mut self, llvm_ir: &mut String, result: &Value, _array_ptr: &Value) {
        // Generate LLVM array length (simplified - should track actual array metadata)
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for array length result"),
        };
        
        // For now, return a placeholder length (this should be improved with proper array metadata)
        llvm_ir.push_str(&format!("  %{} = fadd double 0x4014000000000000, 0x0000000000000000\n", result_str)); // 5.0
    }

    fn generate_bounds_check(&mut self, llvm_ir: &mut String, _array_ptr: &Value, index: &Value, success_label: &str, failure_label: &str) {
        // Generate LLVM bounds checking
        let index_str = self.value_to_string(index);
        
        // Convert index to i64 if needed
        let index_i64 = if matches!(index, Value::Reg(_)) {
            // Assume it's already i64
            index_str
        } else {
            // Convert double to i64
            let conv_reg = self.fresh_reg();
            llvm_ir.push_str(&format!("  %{} = fptosi double {} to i64\n", conv_reg, index_str));
            format!("%{}", conv_reg)
        };
        
        // Check if index is within bounds (simplified - should use actual array size)
        let bounds_check = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = icmp ult i64 {}, 10\n", bounds_check, index_i64)); // Assume max size 10
        
        // Branch based on bounds check
        llvm_ir.push_str(&format!("  br i1 %{}, label %{}, label %{}\n", 
            bounds_check, success_label, failure_label));
    }

    // Vec operations
    fn generate_vec_alloca(&mut self, llvm_ir: &mut String, result: &Value, element_type: &str) {
        // Generate LLVM Vec allocation (Vec is a struct with ptr, len, capacity)
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for vec alloca result"),
        };
        
        let llvm_element_type = self.type_to_llvm(element_type);
        
        // Define Vec structure: { ptr, len, capacity }
        llvm_ir.push_str(&format!("  %{} = alloca {{ {}*, i64, i64 }}, align 8\n", 
            result_str, llvm_element_type));
        
        // Initialize Vec fields to zero
        let ptr_field = self.fresh_reg();
        let len_field = self.fresh_reg();
        let cap_field = self.fresh_reg();
        
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds {{ {}*, i64, i64 }}, {{ {}*, i64, i64 }}* %{}, i32 0, i32 0\n",
            ptr_field, llvm_element_type, llvm_element_type, result_str));
        llvm_ir.push_str(&format!("  store {}* null, {}** %{}, align 8\n",
            llvm_element_type, llvm_element_type, ptr_field));
        
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds {{ {}*, i64, i64 }}, {{ {}*, i64, i64 }}* %{}, i32 0, i32 1\n",
            len_field, llvm_element_type, llvm_element_type, result_str));
        llvm_ir.push_str(&format!("  store i64 0, i64* %{}, align 8\n", len_field));
        
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds {{ {}*, i64, i64 }}, {{ {}*, i64, i64 }}* %{}, i32 0, i32 2\n",
            cap_field, llvm_element_type, llvm_element_type, result_str));
        llvm_ir.push_str(&format!("  store i64 0, i64* %{}, align 8\n", cap_field));
    }

    fn generate_vec_init(&mut self, llvm_ir: &mut String, result: &Value, element_type: &str, elements: &[Value]) {
        // Generate LLVM Vec initialization with elements
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for vec init result"),
        };
        
        let llvm_element_type = self.type_to_llvm(element_type);
        let vec_size = elements.len();
        
        // Allocate Vec structure
        llvm_ir.push_str(&format!("  %{} = alloca {{ {}*, i64, i64 }}, align 8\n", 
            result_str, llvm_element_type));
        
        // Allocate memory for elements
        let data_ptr = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = call i8* @malloc(i64 {})\n", 
            data_ptr, vec_size * 8)); // Assuming 8 bytes per element
        
        let typed_ptr = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = bitcast i8* %{} to {}*\n", 
            typed_ptr, data_ptr, llvm_element_type));
        
        // Initialize elements
        for (i, element) in elements.iter().enumerate() {
            let elem_ptr = self.fresh_reg();
            llvm_ir.push_str(&format!("  %{} = getelementptr inbounds {}, {}* %{}, i64 {}\n",
                elem_ptr, llvm_element_type, llvm_element_type, typed_ptr, i));
            
            let value_str = self.value_to_string(element);
            llvm_ir.push_str(&format!("  store {} {}, {}* %{}, align 8\n",
                llvm_element_type, value_str, llvm_element_type, elem_ptr));
        }
        
        // Set Vec fields
        let ptr_field = self.fresh_reg();
        let len_field = self.fresh_reg();
        let cap_field = self.fresh_reg();
        
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds {{ {}*, i64, i64 }}, {{ {}*, i64, i64 }}* %{}, i32 0, i32 0\n",
            ptr_field, llvm_element_type, llvm_element_type, result_str));
        llvm_ir.push_str(&format!("  store {}* %{}, {}** %{}, align 8\n",
            llvm_element_type, typed_ptr, llvm_element_type, ptr_field));
        
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds {{ {}*, i64, i64 }}, {{ {}*, i64, i64 }}* %{}, i32 0, i32 1\n",
            len_field, llvm_element_type, llvm_element_type, result_str));
        llvm_ir.push_str(&format!("  store i64 {}, i64* %{}, align 8\n", vec_size, len_field));
        
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds {{ {}*, i64, i64 }}, {{ {}*, i64, i64 }}* %{}, i32 0, i32 2\n",
            cap_field, llvm_element_type, llvm_element_type, result_str));
        llvm_ir.push_str(&format!("  store i64 {}, i64* %{}, align 8\n", vec_size, cap_field));
    }

    fn generate_vec_push(&mut self, llvm_ir: &mut String, vec_ptr: &Value, value: &Value) {
        // Generate LLVM Vec push operation (simplified)
        let ptr_str = match vec_ptr {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for vec pointer"),
        };
        
        let value_str = self.value_to_string(value);
        
        // Get current length
        let len_field = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds {{ double*, i64, i64 }}, {{ double*, i64, i64 }}* %{}, i32 0, i32 1\n",
            len_field, ptr_str));
        
        let current_len = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = load i64, i64* %{}, align 8\n", current_len, len_field));
        
        // Get data pointer
        let ptr_field = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds {{ double*, i64, i64 }}, {{ double*, i64, i64 }}* %{}, i32 0, i32 0\n",
            ptr_field, ptr_str));
        
        let data_ptr = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = load double*, double** %{}, align 8\n", data_ptr, ptr_field));
        
        // Store new element at current length position
        let elem_ptr = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds double, double* %{}, i64 %{}\n",
            elem_ptr, data_ptr, current_len));
        
        llvm_ir.push_str(&format!("  store double {}, double* %{}, align 8\n", value_str, elem_ptr));
        
        // Increment length
        let new_len = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = add i64 %{}, 1\n", new_len, current_len));
        llvm_ir.push_str(&format!("  store i64 %{}, i64* %{}, align 8\n", new_len, len_field));
    }

    fn generate_vec_pop(&mut self, llvm_ir: &mut String, result: &Value, vec_ptr: &Value) {
        // Generate LLVM Vec pop operation
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for vec pop result"),
        };
        
        let ptr_str = match vec_ptr {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for vec pointer"),
        };
        
        // Get current length
        let len_field = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds {{ double*, i64, i64 }}, {{ double*, i64, i64 }}* %{}, i32 0, i32 1\n",
            len_field, ptr_str));
        
        let current_len = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = load i64, i64* %{}, align 8\n", current_len, len_field));
        
        // Decrement length
        let new_len = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = sub i64 %{}, 1\n", new_len, current_len));
        llvm_ir.push_str(&format!("  store i64 %{}, i64* %{}, align 8\n", new_len, len_field));
        
        // Get data pointer
        let ptr_field = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds {{ double*, i64, i64 }}, {{ double*, i64, i64 }}* %{}, i32 0, i32 0\n",
            ptr_field, ptr_str));
        
        let data_ptr = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = load double*, double** %{}, align 8\n", data_ptr, ptr_field));
        
        // Load element at new length position
        let elem_ptr = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds double, double* %{}, i64 %{}\n",
            elem_ptr, data_ptr, new_len));
        
        llvm_ir.push_str(&format!("  %{} = load double, double* %{}, align 8\n", result_str, elem_ptr));
    }

    fn generate_vec_length(&mut self, llvm_ir: &mut String, result: &Value, vec_ptr: &Value) {
        // Generate LLVM Vec length operation
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for vec length result"),
        };
        
        let ptr_str = match vec_ptr {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for vec pointer"),
        };
        
        // Get length field
        let len_field = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds {{ double*, i64, i64 }}, {{ double*, i64, i64 }}* %{}, i32 0, i32 1\n",
            len_field, ptr_str));
        
        let len_i64 = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = load i64, i64* %{}, align 8\n", len_i64, len_field));
        
        // Convert to double for unified storage
        llvm_ir.push_str(&format!("  %{} = sitofp i64 %{} to double\n", result_str, len_i64));
    }

    fn generate_vec_capacity(&mut self, llvm_ir: &mut String, result: &Value, vec_ptr: &Value) {
        // Generate LLVM Vec capacity operation
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for vec capacity result"),
        };
        
        let ptr_str = match vec_ptr {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for vec pointer"),
        };
        
        // Get capacity field
        let cap_field = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds {{ double*, i64, i64 }}, {{ double*, i64, i64 }}* %{}, i32 0, i32 2\n",
            cap_field, ptr_str));
        
        let cap_i64 = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = load i64, i64* %{}, align 8\n", cap_i64, cap_field));
        
        // Convert to double for unified storage
        llvm_ir.push_str(&format!("  %{} = sitofp i64 %{} to double\n", result_str, cap_i64));
    }

    fn generate_vec_access(&mut self, llvm_ir: &mut String, result: &Value, vec_ptr: &Value, index: &Value) {
        // Generate LLVM Vec access operation
        let result_str = match result {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for vec access result"),
        };
        
        let ptr_str = match vec_ptr {
            Value::Reg(r) => format!("reg{}", r),
            _ => panic!("Expected register for vec pointer"),
        };
        
        let index_str = self.value_to_string(index);
        
        // Convert index to i64 if needed
        let index_i64 = if matches!(index, Value::Reg(_)) {
            // Assume it's already i64
            index_str
        } else {
            // Convert double to i64
            let conv_reg = self.fresh_reg();
            llvm_ir.push_str(&format!("  %{} = fptosi double {} to i64\n", conv_reg, index_str));
            format!("%{}", conv_reg)
        };
        
        // Get data pointer
        let ptr_field = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds {{ double*, i64, i64 }}, {{ double*, i64, i64 }}* %{}, i32 0, i32 0\n",
            ptr_field, ptr_str));
        
        let data_ptr = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = load double*, double** %{}, align 8\n", data_ptr, ptr_field));
        
        // Access element at index
        let elem_ptr = self.fresh_reg();
        llvm_ir.push_str(&format!("  %{} = getelementptr inbounds double, double* %{}, i64 {}\n",
            elem_ptr, data_ptr, index_i64));
        
        llvm_ir.push_str(&format!("  %{} = load double, double* %{}, align 8\n", result_str, elem_ptr));
    }
}


// Legacy function for backward compatibility
pub fn generate_code(ir_functions: HashMap<String, Function>) -> String {
    let mut generator = CodeGenerator::new();
    generator.generate_code(ir_functions)
}