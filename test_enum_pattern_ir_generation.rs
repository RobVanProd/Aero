// Test for enum and pattern matching IR generation
use std::collections::HashMap;

// Import the compiler modules
mod src {
    pub mod compiler {
        pub mod src {
            pub mod ast;
            pub mod ir;
            pub mod ir_generator;
            pub mod types;
        }
    }
}

use src::compiler::src::ast::*;
use src::compiler::src::ir::*;
use src::compiler::src::ir_generator::*;
use src::compiler::src::types::*;

fn main() {
    println!("Testing enum and pattern matching IR generation...");

    // Test 1: Simple enum definition
    test_simple_enum_definition();
    
    // Test 2: Enum with data variants
    test_enum_with_data();
    
    // Test 3: Pattern matching with enum
    test_enum_pattern_matching();
    
    // Test 4: Pattern matching with guards
    test_pattern_matching_with_guards();
    
    // Test 5: Pattern matching with literal patterns
    test_literal_pattern_matching();

    println!("All enum and pattern matching IR generation tests completed!");
}

fn test_simple_enum_definition() {
    println!("\n=== Test 1: Simple Enum Definition ===");
    
    let mut ir_generator = IrGenerator::new();
    
    // Create a simple enum: enum Color { Red, Green, Blue }
    let enum_stmt = Statement::Enum {
        name: "Color".to_string(),
        generics: vec![],
        variants: vec![
            EnumVariant {
                name: "Red".to_string(),
                data: None,
            },
            EnumVariant {
                name: "Green".to_string(),
                data: None,
            },
            EnumVariant {
                name: "Blue".to_string(),
                data: None,
            },
        ],
    };
    
    let ast = vec![AstNode::Statement(enum_stmt)];
    let functions = ir_generator.generate_ir(ast);
    
    // Check that the main function contains the enum definition
    if let Some(main_func) = functions.get("main") {
        let has_enum_def = main_func.body.iter().any(|inst| {
            matches!(inst, Inst::EnumDef { name, .. } if name == "Color")
        });
        
        if has_enum_def {
            println!("✓ Simple enum definition IR generated successfully");
        } else {
            println!("✗ Simple enum definition IR generation failed");
        }
    } else {
        println!("✗ Main function not found");
    }
}

fn test_enum_with_data() {
    println!("\n=== Test 2: Enum with Data Variants ===");
    
    let mut ir_generator = IrGenerator::new();
    
    // Create enum Option<T> { Some(T), None }
    let enum_stmt = Statement::Enum {
        name: "Option".to_string(),
        generics: vec!["T".to_string()],
        variants: vec![
            EnumVariant {
                name: "Some".to_string(),
                data: Some(EnumVariantData::Tuple(vec![Type::Named("T".to_string())])),
            },
            EnumVariant {
                name: "None".to_string(),
                data: None,
            },
        ],
    };
    
    let ast = vec![AstNode::Statement(enum_stmt)];
    let functions = ir_generator.generate_ir(ast);
    
    // Check that the enum definition includes variant data
    if let Some(main_func) = functions.get("main") {
        let has_enum_def = main_func.body.iter().any(|inst| {
            if let Inst::EnumDef { name, variants, .. } = inst {
                name == "Option" && variants.len() == 2 && 
                variants[0].1.is_some() && variants[1].1.is_none()
            } else {
                false
            }
        });
        
        if has_enum_def {
            println!("✓ Enum with data variants IR generated successfully");
        } else {
            println!("✗ Enum with data variants IR generation failed");
        }
    } else {
        println!("✗ Main function not found");
    }
}

fn test_enum_pattern_matching() {
    println!("\n=== Test 3: Enum Pattern Matching ===");
    
    let mut ir_generator = IrGenerator::new();
    
    // First define the enum
    let enum_stmt = Statement::Enum {
        name: "Color".to_string(),
        generics: vec![],
        variants: vec![
            EnumVariant {
                name: "Red".to_string(),
                data: None,
            },
            EnumVariant {
                name: "Green".to_string(),
                data: None,
            },
            EnumVariant {
                name: "Blue".to_string(),
                data: None,
            },
        ],
    };
    
    // Create a match expression
    let match_expr = Expression::Match {
        expression: Box::new(Expression::Identifier("color".to_string())),
        arms: vec![
            MatchArm {
                pattern: Pattern::Enum {
                    variant: "Red".to_string(),
                    data: None,
                },
                guard: None,
                body: Expression::IntegerLiteral(1),
            },
            MatchArm {
                pattern: Pattern::Enum {
                    variant: "Green".to_string(),
                    data: None,
                },
                guard: None,
                body: Expression::IntegerLiteral(2),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Expression::IntegerLiteral(0),
            },
        ],
    };
    
    let expr_stmt = Statement::Expression(match_expr);
    
    let ast = vec![
        AstNode::Statement(enum_stmt),
        AstNode::Statement(expr_stmt),
    ];
    
    let functions = ir_generator.generate_ir(ast);
    
    // Check that the match expression generates switch instruction
    if let Some(main_func) = functions.get("main") {
        let has_switch = main_func.body.iter().any(|inst| {
            matches!(inst, Inst::Switch { .. })
        });
        
        if has_switch {
            println!("✓ Enum pattern matching IR generated successfully");
        } else {
            println!("✗ Enum pattern matching IR generation failed");
        }
    } else {
        println!("✗ Main function not found");
    }
}

fn test_pattern_matching_with_guards() {
    println!("\n=== Test 4: Pattern Matching with Guards ===");
    
    let mut ir_generator = IrGenerator::new();
    
    // Create a match expression with guard
    let match_expr = Expression::Match {
        expression: Box::new(Expression::Identifier("x".to_string())),
        arms: vec![
            MatchArm {
                pattern: Pattern::Identifier("n".to_string()),
                guard: Some(Expression::Comparison {
                    op: ComparisonOp::GreaterThan,
                    left: Box::new(Expression::Identifier("n".to_string())),
                    right: Box::new(Expression::IntegerLiteral(0)),
                }),
                body: Expression::IntegerLiteral(1),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Expression::IntegerLiteral(0),
            },
        ],
    };
    
    let expr_stmt = Statement::Expression(match_expr);
    let ast = vec![AstNode::Statement(expr_stmt)];
    
    let functions = ir_generator.generate_ir(ast);
    
    // Check that guard conditions generate branch instructions
    if let Some(main_func) = functions.get("main") {
        let has_branch = main_func.body.iter().any(|inst| {
            matches!(inst, Inst::Branch { .. })
        });
        
        if has_branch {
            println!("✓ Pattern matching with guards IR generated successfully");
        } else {
            println!("✗ Pattern matching with guards IR generation failed");
        }
    } else {
        println!("✗ Main function not found");
    }
}

fn test_literal_pattern_matching() {
    println!("\n=== Test 5: Literal Pattern Matching ===");
    
    let mut ir_generator = IrGenerator::new();
    
    // Create a match expression with literal patterns
    let match_expr = Expression::Match {
        expression: Box::new(Expression::Identifier("x".to_string())),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(Expression::IntegerLiteral(1)),
                guard: None,
                body: Expression::IntegerLiteral(10),
            },
            MatchArm {
                pattern: Pattern::Literal(Expression::IntegerLiteral(2)),
                guard: None,
                body: Expression::IntegerLiteral(20),
            },
            MatchArm {
                pattern: Pattern::Range {
                    start: Box::new(Pattern::Literal(Expression::IntegerLiteral(3))),
                    end: Box::new(Pattern::Literal(Expression::IntegerLiteral(5))),
                    inclusive: true,
                },
                guard: None,
                body: Expression::IntegerLiteral(30),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Expression::IntegerLiteral(0),
            },
        ],
    };
    
    let expr_stmt = Statement::Expression(match_expr);
    let ast = vec![AstNode::Statement(expr_stmt)];
    
    let functions = ir_generator.generate_ir(ast);
    
    // Check that literal patterns generate comparison instructions
    if let Some(main_func) = functions.get("main") {
        let has_icmp = main_func.body.iter().any(|inst| {
            matches!(inst, Inst::ICmp { .. })
        });
        
        if has_icmp {
            println!("✓ Literal pattern matching IR generated successfully");
        } else {
            println!("✗ Literal pattern matching IR generation failed");
        }
    } else {
        println!("✗ Main function not found");
    }
}