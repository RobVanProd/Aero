#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Function, Inst, Value};
    use std::collections::HashMap;

    #[test]
    fn test_function_definition_generation() {
        let mut generator = CodeGenerator::new();
        
        // Create a simple function: fn add(a: i32, b: i32) -> i32 { return a + b; }
        let mut function = Function {
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
        
        // Check that print statement is generated (as comment for now)
        assert!(llvm_ir.contains("; print: Hello, World!"));
    }

    #[test]
    fn test_function_with_multiple_parameters() {
        let mut generator = CodeGenerator::new();
        
        // Create function: fn calculate(x: f64, y: f64, z: i32) -> f64
        let function = Function {
            name: "calculate".to_string(),
            body: vec![
                Inst::FunctionDef {
                    name: "calculate".to_string(),
                    parameters: vec![
                        ("x".to_string(), "f64".to_string()),
                        ("y".to_string(), "f64".to_string()),
                        ("z".to_string(), "i32".to_string()),
                    ],
                    return_type: Some("f64".to_string()),
                    body: vec![],
                },
                Inst::Return(Value::ImmFloat(42.0)),
            ],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("calculate".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Check function signature with multiple parameters
        assert!(llvm_ir.contains("define double @calculate(double %x, double %y, i32 %z)"));
        
        // Check parameter allocations
        assert!(llvm_ir.contains("alloca double")); // for x and y
        assert!(llvm_ir.contains("alloca i32"));    // for z
        assert!(llvm_ir.contains("store double %x"));
        assert!(llvm_ir.contains("store double %y"));
        assert!(llvm_ir.contains("store i32 %z"));
    }

    #[test]
    fn test_recursive_function_call() {
        let mut generator = CodeGenerator::new();
        
        // Create a recursive function: fn factorial(n: i32) -> i32
        let function = Function {
            name: "factorial".to_string(),
            body: vec![
                Inst::FunctionDef {
                    name: "factorial".to_string(),
                    parameters: vec![("n".to_string(), "i32".to_string())],
                    return_type: Some("i32".to_string()),
                    body: vec![],
                },
                Inst::Load(Value::Reg(0), Value::Reg(100)), // Load n
                Inst::Call {
                    function: "factorial".to_string(),
                    arguments: vec![Value::Reg(0)],
                    result: Some(Value::Reg(1)),
                },
                Inst::Return(Value::Reg(1)),
            ],
            next_reg: 2,
            next_ptr: 101,
        };

        let mut functions = HashMap::new();
        functions.insert("factorial".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Check that recursive call is generated
        assert!(llvm_ir.contains("define i32 @factorial(i32 %n)"));
        assert!(llvm_ir.contains("call double @factorial"));
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
    fn test_function_with_return_statement() {
        let mut generator = CodeGenerator::new();
        
        // Create function that returns a value
        let function = Function {
            name: "get_value".to_string(),
            body: vec![
                Inst::FunctionDef {
                    name: "get_value".to_string(),
                    parameters: vec![],
                    return_type: Some("i32".to_string()),
                    body: vec![],
                },
                Inst::Return(Value::ImmInt(42)),
            ],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("get_value".to_string(), function);

        let llvm_ir = generator.generate_code(functions);
        
        // Check function signature and return
        assert!(llvm_ir.contains("define i32 @get_value()"));
        assert!(llvm_ir.contains("fptosi double"));
        assert!(llvm_ir.contains("ret i32"));
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
}