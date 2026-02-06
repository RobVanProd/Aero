use compiler::lexer;
use compiler::parser;
use compiler::semantic_analyzer::SemanticAnalyzer;
use insta::assert_debug_snapshot;

// --- Phase 4: Data Structure Tests ---

#[test]
fn test_parse_array_literal() {
    let source = "let arr = [1, 2, 3];";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    // Verify it parsed as a Let with an ArrayLiteral value
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Let {
            name, value, ..
        }) => {
            assert_eq!(name, "arr");
            assert!(
                matches!(value, Some(compiler::ast::Expression::ArrayLiteral(elems)) if elems.len() == 3)
            );
        }
        _ => panic!("Expected let statement with array literal"),
    }
}

#[test]
fn test_parse_array_repeat() {
    let source = "let zeros = [0; 5];";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Let { value, .. }) => {
            assert!(matches!(
                value,
                Some(compiler::ast::Expression::ArrayRepeat { count: 5, .. })
            ));
        }
        _ => panic!("Expected let statement with array repeat"),
    }
}

#[test]
fn test_parse_index_access() {
    let source = "let x = arr[0];";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Let { value, .. }) => {
            assert!(matches!(
                value,
                Some(compiler::ast::Expression::IndexAccess { .. })
            ));
        }
        _ => panic!("Expected let statement with index access"),
    }
}

#[test]
fn test_parse_struct_def() {
    let source = "struct Point { x: i32, y: i32 }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::StructDef {
            name,
            fields,
            ..
        }) => {
            assert_eq!(name, "Point");
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].name, "x");
            assert_eq!(fields[1].name, "y");
        }
        _ => panic!("Expected struct definition"),
    }
}

#[test]
fn test_parse_enum_def() {
    let source = "enum Color { Red, Green, Blue }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::EnumDef {
            name,
            variants,
            ..
        }) => {
            assert_eq!(name, "Color");
            assert_eq!(variants.len(), 3);
            assert_eq!(variants[0].name, "Red");
            assert_eq!(variants[1].name, "Green");
            assert_eq!(variants[2].name, "Blue");
        }
        _ => panic!("Expected enum definition"),
    }
}

#[test]
fn test_parse_enum_with_data() {
    let source = "enum Shape { Circle(f64), Rect(f64, f64) }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::EnumDef {
            name,
            variants,
            ..
        }) => {
            assert_eq!(name, "Shape");
            assert_eq!(variants.len(), 2);
            assert!(
                matches!(&variants[0].kind, compiler::ast::VariantDeclKind::Tuple(types) if types.len() == 1)
            );
            assert!(
                matches!(&variants[1].kind, compiler::ast::VariantDeclKind::Tuple(types) if types.len() == 2)
            );
        }
        _ => panic!("Expected enum definition"),
    }
}

#[test]
fn test_parse_match_expression() {
    let source = "let result = match x { 1 => 10, 2 => 20, _ => 0 };";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Let { value, .. }) => {
            match value {
                Some(compiler::ast::Expression::Match { arms, .. }) => {
                    assert_eq!(arms.len(), 3);
                    assert!(matches!(&arms[2].pattern, compiler::ast::Pattern::Wildcard));
                }
                _ => panic!("Expected match expression"),
            }
        }
        _ => panic!("Expected let statement"),
    }
}

#[test]
fn test_parse_tuple_literal() {
    let source = "let t = (1, 2, 3);";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Let { value, .. }) => {
            assert!(
                matches!(value, Some(compiler::ast::Expression::TupleLiteral(elems)) if elems.len() == 3)
            );
        }
        _ => panic!("Expected let statement with tuple"),
    }
}

#[test]
fn test_parse_string_literal() {
    let source = r#"let s = "hello world";"#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Let { value, .. }) => {
            assert!(
                matches!(value, Some(compiler::ast::Expression::StringLiteral(s)) if s == "hello world")
            );
        }
        _ => panic!("Expected let statement with string"),
    }
}

#[test]
fn test_parse_field_access() {
    let source = "let x = point.x;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Let { value, .. }) => {
            match value {
                Some(compiler::ast::Expression::FieldAccess { field, .. }) => {
                    assert_eq!(field, "x");
                }
                _ => panic!("Expected field access"),
            }
        }
        _ => panic!("Expected let statement"),
    }
}

#[test]
fn test_parse_impl_block() {
    let source = "impl Point { fn new(x: i32, y: i32) -> Point { return x; } }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::ImplBlock {
            type_name,
            methods,
            ..
        }) => {
            assert_eq!(type_name, "Point");
            assert_eq!(methods.len(), 1);
        }
        _ => panic!("Expected impl block"),
    }
}

#[test]
fn test_parse_array_type_annotation() {
    let source = "let arr: [i32; 3] = [1, 2, 3];";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Let {
            type_annotation,
            ..
        }) => {
            assert!(matches!(
                type_annotation,
                Some(compiler::ast::Type::Array(_, 3))
            ));
        }
        _ => panic!("Expected let with array type"),
    }
}

#[test]
fn test_lexer_simple_let() {
    let source = "let x = 10;";
    let tokens = lexer::tokenize(source);
    assert_debug_snapshot!(tokens);
}

#[test]
fn test_lexer_binary_expression() {
    let source = "let y = x + 5.0;";
    let tokens = lexer::tokenize(source);
    assert_debug_snapshot!(tokens);
}

#[test]
fn test_parser_simple_let() {
    let source = "let x = 10;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_debug_snapshot!(ast);
}

#[test]
fn test_parser_binary_expression() {
    let source = "let y = x + 5.0;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_debug_snapshot!(ast);
}

#[test]
fn test_semantic_simple_let() {
    let source = "let x = 10;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert_debug_snapshot!(result);
}

#[test]
fn test_semantic_binary_expression() {
    let source = "let x = 10; let y = x + 5.0;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert_debug_snapshot!(result);
}

#[test]
fn test_semantic_redeclaration() {
    let source = "let x = 10; let x = 20;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert_debug_snapshot!(result);
}

#[test]
fn test_semantic_undeclared_variable() {
    let source = "let y = x + 10;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert_debug_snapshot!(result);
}

// --- Phase 5: Ownership & Borrowing Tests ---

#[test]
fn test_lexer_ampersand_token() {
    let source = "let r = &x;";
    let tokens = lexer::tokenize(source);
    assert!(
        tokens
            .iter()
            .any(|t| *t == compiler::lexer::Token::Ampersand)
    );
}

#[test]
fn test_lexer_trait_keyword() {
    let source = "trait Display { fn show(&self); }";
    let tokens = lexer::tokenize(source);
    assert!(tokens.iter().any(|t| *t == compiler::lexer::Token::Trait));
}

#[test]
fn test_parse_borrow_expression() {
    let source = "let r = &x;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Let { value, .. }) => {
            assert!(matches!(
                value,
                Some(compiler::ast::Expression::Borrow { mutable: false, .. })
            ));
        }
        _ => panic!("Expected let statement with borrow expression"),
    }
}

#[test]
fn test_parse_mut_borrow_expression() {
    let source = "let r = &mut x;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Let { value, .. }) => {
            assert!(matches!(
                value,
                Some(compiler::ast::Expression::Borrow { mutable: true, .. })
            ));
        }
        _ => panic!("Expected let statement with mutable borrow expression"),
    }
}

#[test]
fn test_parse_deref_expression() {
    let source = "let x = *r;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Let { value, .. }) => {
            assert!(matches!(value, Some(compiler::ast::Expression::Deref(_))));
        }
        _ => panic!("Expected let statement with deref expression"),
    }
}

#[test]
fn test_parse_reference_type_annotation() {
    let source = "let r: &i32 = &x;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Let {
            type_annotation,
            ..
        }) => {
            assert!(matches!(
                type_annotation,
                Some(compiler::ast::Type::Reference(_, false))
            ));
        }
        _ => panic!("Expected let with reference type"),
    }
}

#[test]
fn test_parse_mut_reference_type() {
    let source = "let r: &mut i32 = &mut x;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Let {
            type_annotation,
            ..
        }) => {
            assert!(matches!(
                type_annotation,
                Some(compiler::ast::Type::Reference(_, true))
            ));
        }
        _ => panic!("Expected let with mutable reference type"),
    }
}

#[test]
fn test_parse_trait_def() {
    let source = "trait Display { fn show(&self) -> i32; }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::TraitDef {
            name,
            methods,
            ..
        }) => {
            assert_eq!(name, "Display");
            assert_eq!(methods.len(), 1);
            assert_eq!(methods[0].name, "show");
        }
        _ => panic!("Expected trait definition"),
    }
}

#[test]
fn test_parse_impl_trait_for_type() {
    let source = "impl Display for Point { fn show(&self) -> i32 { return 0; } }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::ImplBlock {
            type_name,
            trait_name,
            methods,
            ..
        }) => {
            assert_eq!(type_name, "Point");
            assert_eq!(trait_name.as_deref(), Some("Display"));
            assert_eq!(methods.len(), 1);
        }
        _ => panic!("Expected impl Trait for Type block"),
    }
}

#[test]
fn test_parse_generic_function() {
    let source = "fn identity<T>(x: T) -> T { return x; }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Function {
            name,
            type_params,
            ..
        }) => {
            assert_eq!(name, "identity");
            assert_eq!(type_params, &vec!["T".to_string()]);
        }
        _ => panic!("Expected generic function"),
    }
}

#[test]
fn test_parse_generic_struct() {
    let source = "struct Container<T> { value: T }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::StructDef {
            name,
            type_params,
            fields,
            ..
        }) => {
            assert_eq!(name, "Container");
            assert_eq!(type_params, &vec!["T".to_string()]);
            assert_eq!(fields.len(), 1);
        }
        _ => panic!("Expected generic struct"),
    }
}

#[test]
fn test_semantic_copy_type_no_move() {
    // Integers are Copy types - should not trigger move
    let source = "let x = 10; let y = x; let z = x;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_ok(),
        "Copy types should not trigger use-after-move"
    );
}

#[test]
fn test_semantic_move_detection() {
    // String is a non-Copy type - second use should fail
    let source = r#"let s1 = "hello"; let s2 = s1; let s3 = s1;"#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_err(),
        "Use of moved String value should be an error"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("moved"),
        "Error should mention 'moved': {}",
        err
    );
}

#[test]
fn test_semantic_move_once_ok() {
    // Moving a value once is fine
    let source = r#"let s1 = "hello"; let s2 = s1;"#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(result.is_ok(), "Moving a value once should be ok");
}

// --- Phase 5b: Borrow Checker Tests ---

#[test]
fn test_semantic_immutable_borrow_ok() {
    // Multiple immutable borrows are fine
    let source = "let x = 10; let r1 = &x; let r2 = &x;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_ok(),
        "Multiple immutable borrows should be allowed"
    );
}

#[test]
fn test_semantic_mutable_borrow_requires_mut() {
    // Mutable borrow of non-mut variable should fail
    let source = "let x = 10; let r = &mut x;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_err(),
        "Mutable borrow of non-mut variable should fail"
    );
    let err = result.unwrap_err();
    assert!(err.contains("not declared as mutable"), "Error: {}", err);
}

#[test]
fn test_semantic_mutable_borrow_ok() {
    // Mutable borrow of mut variable is fine
    let source = "let mut x = 10; let r = &mut x;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_ok(),
        "Mutable borrow of mut variable should work: {:?}",
        result
    );
}

#[test]
fn test_semantic_double_mutable_borrow_fails() {
    // Two mutable borrows of same variable should fail
    let source = "let mut x = 10; let r1 = &mut x; let r2 = &mut x;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(result.is_err(), "Double mutable borrow should fail");
    let err = result.unwrap_err();
    assert!(err.contains("mutable more than once"), "Error: {}", err);
}

#[test]
fn test_semantic_mut_and_immut_borrow_conflict() {
    // Mutable borrow while immutably borrowed should fail
    let source = "let mut x = 10; let r1 = &x; let r2 = &mut x;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_err(),
        "Mutable borrow while immutably borrowed should fail"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("immutable"),
        "Error should mention immutable conflict: {}",
        err
    );
}

#[test]
fn test_semantic_immut_borrow_while_mut_borrowed_fails() {
    // Immutable borrow while mutably borrowed should fail
    let source = "let mut x = 10; let r1 = &mut x; let r2 = &x;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_err(),
        "Immutable borrow while mutably borrowed should fail"
    );
    let err = result.unwrap_err();
    assert!(
        err.contains("mutable"),
        "Error should mention mutable conflict: {}",
        err
    );
}

#[test]
fn test_semantic_borrow_of_moved_value_fails() {
    // Borrowing a moved value should fail
    let source = r#"let s1 = "hello"; let s2 = s1; let r = &s1;"#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(result.is_err(), "Borrowing a moved value should fail");
    let err = result.unwrap_err();
    assert!(err.contains("moved"), "Error: {}", err);
}

// --- Phase 5c: Generics Tests ---

#[test]
fn test_parse_generic_function_single_param() {
    let source = "fn identity<T>(x: T) -> T { return x; }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Function {
            name,
            type_params,
            parameters,
            return_type,
            ..
        }) => {
            assert_eq!(name, "identity");
            assert_eq!(type_params, &vec!["T".to_string()]);
            assert_eq!(parameters.len(), 1);
            assert!(matches!(&parameters[0].param_type, compiler::ast::Type::Named(n) if n == "T"));
            assert!(matches!(return_type, Some(compiler::ast::Type::Named(n)) if n == "T"));
        }
        _ => panic!("Expected generic function"),
    }
}

#[test]
fn test_parse_generic_function_multi_params() {
    let source = "fn pair<A, B>(a: A, b: B) -> (A, B) { return (a, b); }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Function {
            type_params,
            ..
        }) => {
            assert_eq!(type_params, &vec!["A".to_string(), "B".to_string()]);
        }
        _ => panic!("Expected generic function"),
    }
}

#[test]
fn test_parse_generic_struct_definition() {
    let source = "struct Container<T> { value: T }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::StructDef {
            name,
            type_params,
            fields,
            ..
        }) => {
            assert_eq!(name, "Container");
            assert_eq!(type_params, &vec!["T".to_string()]);
            assert_eq!(fields.len(), 1);
            assert!(matches!(&fields[0].field_type, compiler::ast::Type::Named(n) if n == "T"));
        }
        _ => panic!("Expected generic struct definition"),
    }
}

#[test]
fn test_parse_generic_enum_definition() {
    let source = "enum Result<T, E> { Ok(T), Err(E) }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::EnumDef {
            name,
            type_params,
            variants,
            ..
        }) => {
            assert_eq!(name, "Result");
            assert_eq!(type_params, &vec!["T".to_string(), "E".to_string()]);
            assert_eq!(variants.len(), 2);
        }
        _ => panic!("Expected generic enum"),
    }
}

#[test]
fn test_parse_generic_type_annotation() {
    let source = "let x: Container<i32> = Container { value: 42 };";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Let {
            type_annotation,
            ..
        }) => {
            assert!(matches!(
                type_annotation,
                Some(compiler::ast::Type::Generic(name, params))
                if name == "Container" && params.len() == 1
            ));
        }
        _ => panic!("Expected let with generic type annotation"),
    }
}

#[test]
fn test_parse_generic_impl_block() {
    let source = r#"
        impl<T> Container<T> {
            fn new(value: T) -> Container<T> {
                return value;
            }
        }
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::ImplBlock {
            type_name,
            type_params,
            trait_name,
            methods,
        }) => {
            assert_eq!(type_name, "Container");
            assert_eq!(type_params, &vec!["T".to_string()]);
            assert!(trait_name.is_none());
            assert_eq!(methods.len(), 1);
        }
        _ => panic!("Expected generic impl block"),
    }
}

#[test]
fn test_parse_trait_bounds_on_generic() {
    let source = "fn print_item<T: Display>(item: T) { return; }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Function {
            name,
            type_params,
            ..
        }) => {
            assert_eq!(name, "print_item");
            assert_eq!(type_params, &vec!["T".to_string()]);
        }
        _ => panic!("Expected function with trait-bounded generic"),
    }
}

#[test]
fn test_parse_multiple_trait_bounds() {
    let source = "fn foo<T: Clone + Display>(item: T) -> T { return item; }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Function {
            type_params,
            ..
        }) => {
            assert_eq!(type_params, &vec!["T".to_string()]);
        }
        _ => panic!("Expected function with multiple trait bounds"),
    }
}

#[test]
fn test_parse_where_clause() {
    let source = "fn process<T, U>(s: T, d: U) where T: Clone, U: Default { return; }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Function {
            type_params,
            parameters,
            ..
        }) => {
            assert_eq!(type_params.len(), 2);
            assert_eq!(parameters.len(), 2);
        }
        _ => panic!("Expected function with where clause"),
    }
}

#[test]
fn test_parse_impl_trait_for_generic_struct() {
    let source = r#"
        impl<T> Printable for Container<T> {
            fn to_string(&self) -> String {
                return "container";
            }
        }
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::ImplBlock {
            type_name,
            trait_name,
            type_params,
            methods,
        }) => {
            assert_eq!(type_name, "Container");
            assert_eq!(trait_name, &Some("Printable".to_string()));
            assert_eq!(type_params, &vec!["T".to_string()]);
            assert_eq!(methods.len(), 1);
        }
        _ => panic!("Expected impl Trait for generic struct"),
    }
}

#[test]
fn test_semantic_move_into_function() {
    // Passing a non-Copy value to a function should move it
    let source = r#"
        fn take_ownership(s: String) { return; }
        fn main() {
            let s1 = "hello";
            take_ownership(s1);
            let s2 = s1;
        }
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_err(),
        "Should detect use of moved value after function call"
    );
}

#[test]
fn test_semantic_struct_is_non_copy() {
    // Struct values should use move semantics
    let source = r#"
        struct Point { x: i32, y: i32 }
        fn main() {
            let p1 = Point { x: 1, y: 2 };
            let p2 = p1;
            let p3 = p1;
        }
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(result.is_err(), "Struct should use move semantics");
}

#[test]
fn test_semantic_generic_type_param_in_function() {
    // Generic function with T parameter should analyze without error
    let source = r#"
        fn identity<T>(x: T) -> T {
            return x;
        }
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_ok(),
        "Generic function should analyze successfully: {:?}",
        result
    );
}

// --- Phase 5d: Trait Enforcement Tests ---

#[test]
fn test_semantic_trait_missing_method() {
    // impl Trait for Type that's missing a required method should fail
    let source = r#"
        trait Shape {
            fn area(&self) -> f64;
            fn perimeter(&self) -> f64;
        }
        struct Circle { radius: f64 }
        impl Shape for Circle {
            fn area(&self) -> f64 { return 3.14; }
        }
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(result.is_err(), "Should detect missing trait method");
    let err = result.unwrap_err();
    assert!(
        err.contains("perimeter"),
        "Error should mention missing method: {}",
        err
    );
}

#[test]
fn test_semantic_trait_all_methods_implemented_ok() {
    // impl Trait for Type with all methods implemented should pass
    let source = r#"
        trait Shape {
            fn area(&self) -> f64;
        }
        struct Square { side: f64 }
        impl Shape for Square {
            fn area(&self) -> f64 { return 1.0; }
        }
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_ok(),
        "Complete trait impl should pass: {:?}",
        result
    );
}

#[test]
fn test_semantic_unsatisfied_trait_bound() {
    // Calling a function with trait-bounded generic using a type
    // that doesn't implement the trait should fail
    let source = r#"
        trait Display {
            fn display(&self) -> String;
        }
        fn print_item<T: Display>(item: T) {
            return;
        }
        struct Opaque { data: i32 }
        fn main() {
            let o = Opaque { data: 42 };
            print_item(o);
        }
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_err(),
        "Should detect unsatisfied trait bound: {:?}",
        result
    );
}

#[test]
fn test_semantic_trait_bound_satisfied_ok() {
    // Calling a function with trait-bounded generic using a type
    // that DOES implement the trait should pass
    let source = r#"
        trait Display {
            fn display(&self) -> String;
        }
        fn print_item<T: Display>(item: T) {
            return;
        }
        struct Widget { name: String }
        impl Display for Widget {
            fn display(&self) -> String { return "widget"; }
        }
        fn main() {
            let w = Widget { name: "foo" };
            print_item(w);
        }
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_ok(),
        "Satisfied trait bound should pass: {:?}",
        result
    );
}

#[test]
fn test_parse_trait_bounds_stored_in_ast() {
    // Verify trait bounds are stored in the Function AST node
    let source = "fn foo<T: Clone + Display>(item: T) { return; }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Function {
            trait_bounds,
            ..
        }) => {
            assert_eq!(trait_bounds.len(), 1);
            assert_eq!(trait_bounds[0].0, "T");
            assert_eq!(
                trait_bounds[0].1,
                vec!["Clone".to_string(), "Display".to_string()]
            );
        }
        _ => panic!("Expected function with trait bounds"),
    }
}
