#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{AstNode, Statement, Expression, Parameter, Block, Type};
    use crate::ir::{Inst, Value};
    use crate::types::Ty;

    // ===== PHASE 3 COMPREHENSIVE IR GENERATOR TESTS =====

    #[test]
    fn test_function_definition_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a simple function: fn add(a: i32, b: i32) -> i32 { a + b }
        let param1 = Parameter {
            name: "a".to_string(),
            param_type: Type { name: "i32".to_string() },
        };
        let param2 = Parameter {
            name: "b".to_string(),
            param_type: Type { name: "i32".to_string() },
        };
        
        let body = Block {
            statements: vec![],
            expression: Some(Expression::Binary {
                op: "+".to_string(),
                lhs: Box::new(Expression::Identifier("a".to_string())),
                rhs: Box::new(Expression::Identifier("b".to_string())),
                ty: Some(Ty::Int),
            }),
        };
        
        let func_stmt = Statement::Function {
            name: "add".to_string(),
            parameters: vec![param1, param2],
            return_type: Some(Type { name: "i32".to_string() }),
            body,
        };
        
        let ast = vec![AstNode::Statement(func_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        // Check that main function exists
        assert!(ir.contains_key("main"));
        let main_func = &ir["main"];
        
        // Check that main function contains a FunctionDef instruction
        assert_eq!(main_func.body.len(), 1);
        match &main_func.body[0] {
            Inst::FunctionDef { name, parameters, body } => {
                assert_eq!(name, "add");
                assert_eq!(parameters.len(), 2);
                assert_eq!(parameters[0], "a");
                assert_eq!(parameters[1], "b");
                
                // Check function body structure
                assert!(body.len() >= 4); // At least: 2 allocas, 1 add, 1 return
                
                // Check parameter allocations
                assert!(matches!(body[0], Inst::Alloca(Value::Reg(0), ref name) if name == "a"));
                assert!(matches!(body[1], Inst::Alloca(Value::Reg(1), ref name) if name == "b"));
                
                // Check return instruction exists
                assert!(body.iter().any(|inst| matches!(inst, Inst::Return(_))));
            }
            _ => panic!("Expected FunctionDef instruction"),
        }
        
        // Check that function is stored in functions map
        assert!(ir.contains_key("add"));
    }

    #[test]
    fn test_function_call_ir_generation() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a function call: add(5, 3)
        let func_call = Expression::FunctionCall {
            name: "add".to_string(),
            arguments: vec![
                Expression::IntegerLiteral(5),
                Expression::IntegerLiteral(3),
            ],
        };
        
        let let_stmt = Statement::Let {
            name: "result".to_string(),
            value: func_call,
        };
        
        let ast = vec![AstNode::Statement(let_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        let main_func = &ir["main"];
        
        // Should have: alloca for result, call instruction, store instruction
        assert!(main_func.body.len() >= 3);
        
        // Find the call instruction
        let call_inst = main_func.body.iter().find(|inst| matches!(inst, Inst::Call { .. }));
        assert!(call_inst.is_some());
        
        match call_inst.unwrap() {
            Inst::Call { function, arguments, result } => {
                assert_eq!(function, "add");
                assert_eq!(arguments.len(), 2);
                assert!(matches!(arguments[0], Value::ImmInt(5)));
                assert!(matches!(arguments[1], Value::ImmInt(3)));
                assert!(result.is_some());
            }
            _ => panic!("Expected Call instruction"),
        }
    }

    #[test]
    fn test_function_with_no_parameters() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a function: fn get_value() -> i32 { 42 }
        let body = Block {
            statements: vec![],
            expression: Some(Expression::IntegerLiteral(42)),
        };
        
        let func_stmt = Statement::Function {
            name: "get_value".to_string(),
            parameters: vec![],
            return_type: Some(Type { name: "i32".to_string() }),
            body,
        };
        
        let ast = vec![AstNode::Statement(func_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        let main_func = &ir["main"];
        match &main_func.body[0] {
            Inst::FunctionDef { name, parameters, body } => {
                assert_eq!(name, "get_value");
                assert_eq!(parameters.len(), 0);
                
                // Should have return instruction with immediate value
                assert!(body.iter().any(|inst| matches!(inst, Inst::Return(Value::ImmInt(42)))));
            }
            _ => panic!("Expected FunctionDef instruction"),
        }
    }

    #[test]
    fn test_function_with_return_statement() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a function: fn double(x: i32) -> i32 { return x * 2; }
        let param = Parameter {
            name: "x".to_string(),
            param_type: Type { name: "i32".to_string() },
        };
        
        let body = Block {
            statements: vec![
                Statement::Return(Expression::Binary {
                    op: "*".to_string(),
                    lhs: Box::new(Expression::Identifier("x".to_string())),
                    rhs: Box::new(Expression::IntegerLiteral(2)),
                    ty: Some(Ty::Int),
                }),
            ],
            expression: None,
        };
        
        let func_stmt = Statement::Function {
            name: "double".to_string(),
            parameters: vec![param],
            return_type: Some(Type { name: "i32".to_string() }),
            body,
        };
        
        let ast = vec![AstNode::Statement(func_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        let main_func = &ir["main"];
        match &main_func.body[0] {
            Inst::FunctionDef { name, parameters, body } => {
                assert_eq!(name, "double");
                assert_eq!(parameters.len(), 1);
                assert_eq!(parameters[0], "x");
                
                // Should have parameter alloca, multiplication, and return
                assert!(body.iter().any(|inst| matches!(inst, Inst::Alloca(Value::Reg(0), ref name) if name == "x")));
                assert!(body.iter().any(|inst| matches!(inst, Inst::Mul(_, _, _))));
                assert!(body.iter().any(|inst| matches!(inst, Inst::Return(_))));
            }
            _ => panic!("Expected FunctionDef instruction"),
        }
    }

    #[test]
    fn test_nested_function_calls() {
        let mut ir_gen = IrGenerator::new();
        
        // Create nested function calls: add(multiply(2, 3), 4)
        let inner_call = Expression::FunctionCall {
            name: "multiply".to_string(),
            arguments: vec![
                Expression::IntegerLiteral(2),
                Expression::IntegerLiteral(3),
            ],
        };
        
        let outer_call = Expression::FunctionCall {
            name: "add".to_string(),
            arguments: vec![
                inner_call,
                Expression::IntegerLiteral(4),
            ],
        };
        
        let let_stmt = Statement::Let {
            name: "result".to_string(),
            value: outer_call,
        };
        
        let ast = vec![AstNode::Statement(let_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        let main_func = &ir["main"];
        
        // Should have two call instructions
        let call_instructions: Vec<_> = main_func.body.iter()
            .filter(|inst| matches!(inst, Inst::Call { .. }))
            .collect();
        
        assert_eq!(call_instructions.len(), 2);
        
        // First call should be to multiply
        match call_instructions[0] {
            Inst::Call { function, arguments, .. } => {
                assert_eq!(function, "multiply");
                assert_eq!(arguments.len(), 2);
                assert!(matches!(arguments[0], Value::ImmInt(2)));
                assert!(matches!(arguments[1], Value::ImmInt(3)));
            }
            _ => panic!("Expected Call instruction"),
        }
        
        // Second call should be to add
        match call_instructions[1] {
            Inst::Call { function, arguments, .. } => {
                assert_eq!(function, "add");
                assert_eq!(arguments.len(), 2);
                // First argument should be a register (result of multiply call)
                assert!(matches!(arguments[0], Value::Reg(_)));
                assert!(matches!(arguments[1], Value::ImmInt(4)));
            }
            _ => panic!("Expected Call instruction"),
        }
    }

    #[test]
    fn test_function_with_local_variables() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a function with local variables:
        // fn calculate(x: i32) -> i32 {
        //     let temp = x + 1;
        //     temp * 2
        // }
        let param = Parameter {
            name: "x".to_string(),
            param_type: Type { name: "i32".to_string() },
        };
        
        let body = Block {
            statements: vec![
                Statement::Let {
                    name: "temp".to_string(),
                    value: Expression::Binary {
                        op: "+".to_string(),
                        lhs: Box::new(Expression::Identifier("x".to_string())),
                        rhs: Box::new(Expression::IntegerLiteral(1)),
                        ty: Some(Ty::Int),
                    },
                },
            ],
            expression: Some(Expression::Binary {
                op: "*".to_string(),
                lhs: Box::new(Expression::Identifier("temp".to_string())),
                rhs: Box::new(Expression::IntegerLiteral(2)),
                ty: Some(Ty::Int),
            }),
        };
        
        let func_stmt = Statement::Function {
            name: "calculate".to_string(),
            parameters: vec![param],
            return_type: Some(Type { name: "i32".to_string() }),
            body,
        };
        
        let ast = vec![AstNode::Statement(func_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        let main_func = &ir["main"];
        match &main_func.body[0] {
            Inst::FunctionDef { name, parameters, body } => {
                assert_eq!(name, "calculate");
                assert_eq!(parameters.len(), 1);
                
                // Should have allocas for both parameter and local variable
                let alloca_count = body.iter().filter(|inst| matches!(inst, Inst::Alloca(_, _))).count();
                assert_eq!(alloca_count, 2); // x and temp
                
                // Should have two add/multiply operations and a return
                assert!(body.iter().any(|inst| matches!(inst, Inst::Add(_, _, _))));
                assert!(body.iter().any(|inst| matches!(inst, Inst::Mul(_, _, _))));
                assert!(body.iter().any(|inst| matches!(inst, Inst::Return(_))));
            }
            _ => panic!("Expected FunctionDef instruction"),
        }
    }

    #[test]
    fn test_function_call_with_no_arguments() {
        let mut ir_gen = IrGenerator::new();
        
        // Create a function call with no arguments: get_value()
        let func_call = Expression::FunctionCall {
            name: "get_value".to_string(),
            arguments: vec![],
        };
        
        let let_stmt = Statement::Let {
            name: "result".to_string(),
            value: func_call,
        };
        
        let ast = vec![AstNode::Statement(let_stmt)];
        let ir = ir_gen.generate_ir(ast);
        
        let main_func = &ir["main"];
        
        // Find the call instruction
        let call_inst = main_func.body.iter().find(|inst| matches!(inst, Inst::Call { .. }));
        assert!(call_inst.is_some());
        
        match call_inst.unwrap() {
            Inst::Call { function, arguments, result } => {
                assert_eq!(function, "get_value");
                assert_eq!(arguments.len(), 0);
                assert!(result.is_some());
            }
            _ => panic!("Expected Call instruction"),
        }
    }
}