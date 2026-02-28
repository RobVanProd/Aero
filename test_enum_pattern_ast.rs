// Test file to verify enum and pattern AST nodes work correctly
// This is a standalone test to verify task 3.2 completion

use std::path::Path;

// Add the compiler source to the path
#[path = "src/compiler/src/ast.rs"]
mod ast;

use ast::*;

fn main() {
    println!("Testing Enum and Pattern AST Nodes...");
    
    // Test 1: Basic enum definition
    let color_enum = Statement::Enum {
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
    
    match color_enum {
        Statement::Enum { name, generics, variants } => {
            assert_eq!(name, "Color");
            assert_eq!(generics.len(), 0);
            assert_eq!(variants.len(), 3);
            println!("✓ Basic enum definition test passed");
        }
        _ => panic!("Expected Enum statement"),
    }
    
    // Test 2: Enum with tuple data
    let option_enum = Statement::Enum {
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
    
    match option_enum {
        Statement::Enum { name, generics, variants } => {
            assert_eq!(name, "Option");
            assert_eq!(generics.len(), 1);
            assert_eq!(generics[0], "T");
            assert_eq!(variants.len(), 2);
            println!("✓ Generic enum with tuple data test passed");
        }
        _ => panic!("Expected Enum statement"),
    }
    
    // Test 3: Pattern matching expression
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
    
    match match_expr {
        Expression::Match { expression, arms } => {
            assert_eq!(arms.len(), 3);
            println!("✓ Pattern matching expression test passed");
        }
        _ => panic!("Expected Match expression"),
    }
    
    // Test 4: Complex patterns
    let complex_pattern = Pattern::Enum {
        variant: "Some".to_string(),
        data: Some(Box::new(Pattern::Struct {
            name: "Point".to_string(),
            fields: vec![
                ("x".to_string(), Pattern::Identifier("px".to_string())),
                ("y".to_string(), Pattern::Wildcard),
            ],
            rest: true,
        })),
    };
    
    match complex_pattern {
        Pattern::Enum { variant, data } => {
            assert_eq!(variant, "Some");
            assert!(data.is_some());
            println!("✓ Complex nested pattern test passed");
        }
        _ => panic!("Expected Enum pattern"),
    }
    
    // Test 5: Pattern with guards
    let guarded_match = Expression::Match {
        expression: Box::new(Expression::Identifier("x".to_string())),
        arms: vec![
            MatchArm {
                pattern: Pattern::Identifier("n".to_string()),
                guard: Some(Expression::Comparison {
                    op: ComparisonOp::GreaterThan,
                    left: Box::new(Expression::Identifier("n".to_string())),
                    right: Box::new(Expression::IntegerLiteral(0)),
                }),
                body: Expression::Identifier("n".to_string()),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Expression::IntegerLiteral(0),
            },
        ],
    };
    
    match guarded_match {
        Expression::Match { expression, arms } => {
            assert_eq!(arms.len(), 2);
            assert!(arms[0].guard.is_some());
            println!("✓ Pattern with guards test passed");
        }
        _ => panic!("Expected Match expression"),
    }
    
    println!("\n🎉 All enum and pattern AST node tests passed!");
    println!("Task 3.2: Add enum definition and pattern AST nodes - COMPLETED");
}