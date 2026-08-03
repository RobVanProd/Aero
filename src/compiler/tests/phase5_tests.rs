use compiler::lexer;
use compiler::lexer::Token;
use compiler::parser;
use compiler::semantic_analyzer::SemanticAnalyzer;

fn strict_tokens(source: &str) -> Vec<compiler::lexer::Token> {
    lexer::try_tokenize_with_locations(source, Some("phase5_contract.aero".to_string()))
        .expect("selected Phase 5 syntax must lex strictly")
        .into_iter()
        .map(|located| located.token)
        .collect()
}

fn strict_parse(source: &str) -> Vec<compiler::ast::AstNode> {
    let tokens =
        lexer::try_tokenize_with_locations(source, Some("phase5_contract.aero".to_string()))
            .expect("selected Phase 5 syntax must lex strictly");
    parser::parse_with_locations(tokens).expect("selected Phase 5 syntax must parse strictly")
}

// =============================================================================
// Phase 5a: Ownership and Move Semantics
// =============================================================================
//
// The Aero ownership model requires:
// - Every value has exactly one owner
// - Assignment of non-Copy types transfers ownership (move)
// - Use after move is a compile-time error
// - Copy types (integers, bools, floats) are implicitly copied on assignment
// - Functions taking ownership of a parameter invalidate the caller's binding

#[test]
#[ignore = "quarantined: recovery-path shallow move diagnostic is not a full ownership model"]
fn test_semantic_use_after_move_simple() {
    // Assigning a non-Copy value to another variable moves it.
    // Using the original after the move should be a compile-time error.
    //
    // Expected: error about use of moved value `s1`
    let source = r#"
        let s1 = "hello";
        let s2 = s1;
        let s3 = s1;
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(result.is_err(), "Should detect use of moved value s1");
}

#[test]
#[ignore = "quarantined: positive Copy smoke does not establish ownership semantics"]
fn test_semantic_copy_types_not_moved() {
    // Integers are Copy types. Assigning them creates a copy,
    // and both the original and the copy remain valid.
    //
    // Expected: no error, all three variables are usable
    let source = r#"
        let x = 42;
        let y = x;
        let z = x + y;
    "#;
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
#[ignore = "quarantined: recovery-path call move diagnostic needs a frozen ownership model"]
fn test_semantic_move_into_function() {
    // Passing a non-Copy value to a function that takes ownership
    // invalidates the caller's binding.
    //
    // Expected: error on the line using s1 after take_ownership(s1)
    let source = r#"
        fn take_ownership(s: String) {
            return;
        }
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
#[ignore = "quarantined: initializer call does not prove move-return reownership"]
fn test_semantic_move_return_reownership() {
    // A function can return ownership. The returned value binds to a
    // new variable that is valid; the original variable remains moved.
    //
    // Expected: no error (s2 receives returned ownership, s3 moves from s2)
    let source = r#"
        fn take_and_return(s: String) -> String {
            return s;
        }
        fn main() {
            let s1 = "hello";
            let s2 = take_and_return(s1);
            let s3 = s2;
        }
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(result.is_ok(), "Returned ownership should make s2 valid");
}

#[test]
#[ignore = "quarantined: passes on unsupported struct construction before move semantics"]
fn test_semantic_move_in_struct_field() {
    // A struct containing a non-Copy field (String) is itself non-Copy.
    // Moving it invalidates the source variable.
    //
    // Expected: error on second use of p1 after move
    let source = r#"
        struct Person { name: String, age: i32 }
        fn main() {
            let p1 = Person { name: "Alice", age: 30 };
            let p2 = p1;
            let p3 = p1;
        }
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_err(),
        "Struct with non-Copy field should move, not copy"
    );
}

// =============================================================================
// Phase 5b: Borrowing and References
// =============================================================================
//
// Lexer additions needed: & as standalone token (Ampersand)
// AST types already exist:
//   Expression::Borrow { expr, mutable: bool }
//   Expression::Deref(Box<Expression>)
//   Type::Reference(Box<Type>, bool)  -- bool = is_mutable
//
// Borrow checker rules:
//   * One &mut OR any number of & at a time
//   * References must not outlive the data they point to

#[test]
fn test_strict_lexer_ampersand_token_stream() {
    // The lexer should recognize a single & as a borrow/reference operator,
    // distinct from && (logical and).
    let source = "let r = &x;";
    assert_eq!(
        strict_tokens(source),
        vec![
            Token::Let,
            Token::Identifier("r".to_string()),
            Token::Assign,
            Token::Ampersand,
            Token::Identifier("x".to_string()),
            Token::Semicolon,
            Token::Eof,
        ]
    );
    assert_eq!(
        strict_tokens("let both = left && right;"),
        vec![
            Token::Let,
            Token::Identifier("both".to_string()),
            Token::Assign,
            Token::Identifier("left".to_string()),
            Token::LogicalAnd,
            Token::Identifier("right".to_string()),
            Token::Semicolon,
            Token::Eof,
        ]
    );
}

#[test]
fn test_strict_lexer_ampersand_mut_token_stream() {
    // &mut should tokenize as Ampersand followed by the existing Mut keyword.
    let source = "let r = &mut x;";
    assert_eq!(
        strict_tokens(source),
        vec![
            Token::Let,
            Token::Identifier("r".to_string()),
            Token::Assign,
            Token::Ampersand,
            Token::Mut,
            Token::Identifier("x".to_string()),
            Token::Semicolon,
            Token::Eof,
        ]
    );
}

#[test]
fn test_strict_parse_immutable_reference_param_retained_shape() {
    // Function parameter typed as &String should parse as
    // Type::Reference(Box<Type::Named("String")>, false).
    let source = "fn calc_length(s: &String) -> i32 { return 0; }";
    let ast = strict_parse(source);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Function {
            parameters,
            ..
        }) => {
            assert_eq!(parameters.len(), 1);
            assert_eq!(parameters[0].name, "s");
            assert!(
                matches!(&parameters[0].param_type,
                    compiler::ast::Type::Reference(inner, false)
                    if matches!(inner.as_ref(), compiler::ast::Type::Named(n) if n == "String")
                ),
                "Parameter type should be &String (immutable reference)"
            );
        }
        _ => panic!("Expected function definition"),
    }
}

#[test]
fn test_strict_parse_mutable_reference_param_retained_shape() {
    // Function parameter typed as &mut String should parse as
    // Type::Reference(Box<Type::Named("String")>, true).
    let source = "fn append(s: &mut String) { return; }";
    let ast = strict_parse(source);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Function {
            parameters,
            ..
        }) => {
            assert_eq!(parameters.len(), 1);
            assert_eq!(parameters[0].name, "s");
            assert!(
                matches!(&parameters[0].param_type,
                    compiler::ast::Type::Reference(inner, true)
                    if matches!(inner.as_ref(), compiler::ast::Type::Named(n) if n == "String")
                ),
                "Parameter type should be &mut String (mutable reference)"
            );
        }
        _ => panic!("Expected function definition"),
    }
}

#[test]
fn test_strict_parse_borrow_expression_retained_shape() {
    // Taking a reference with & should parse as Expression::Borrow { mutable: false }.
    let source = "let r = &x;";
    let ast = strict_parse(source);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Let {
            name, value, ..
        }) => {
            assert_eq!(name, "r");
            assert!(matches!(
                value,
                Some(compiler::ast::Expression::Borrow {
                    mutable: false,
                    expr,
                }) if matches!(expr.as_ref(), compiler::ast::Expression::Identifier(identifier) if identifier == "x")
            ));
        }
        _ => panic!("Expected let statement with borrow"),
    }
}

#[test]
fn test_strict_parse_mutable_borrow_expression_retained_shape() {
    // Taking a mutable reference with &mut should parse as Expression::Borrow { mutable: true }.
    let source = "let r = &mut x;";
    let ast = strict_parse(source);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Let {
            name, value, ..
        }) => {
            assert_eq!(name, "r");
            assert!(matches!(
                value,
                Some(compiler::ast::Expression::Borrow {
                    mutable: true,
                    expr,
                }) if matches!(expr.as_ref(), compiler::ast::Expression::Identifier(identifier) if identifier == "x")
            ));
        }
        _ => panic!("Expected let statement with mutable borrow"),
    }
}

#[test]
fn test_strict_parse_deref_expression_retained_shape() {
    // The dereference operator * should parse as Expression::Deref.
    let source = "let val = *r;";
    let ast = strict_parse(source);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Let {
            name, value, ..
        }) => {
            assert_eq!(name, "val");
            assert!(matches!(
                value,
                Some(compiler::ast::Expression::Deref(expr))
                    if matches!(expr.as_ref(), compiler::ast::Expression::Identifier(identifier) if identifier == "r")
            ));
        }
        _ => panic!("Expected let statement with deref"),
    }
}

#[test]
#[ignore = "quarantined: positive shallow borrow smoke does not prove lifetime or provenance"]
fn test_semantic_multiple_immutable_borrows_ok() {
    // Multiple immutable borrows of the same value are allowed.
    //
    // Expected: no error
    let source = r#"
        let x = 42;
        let r1 = &x;
        let r2 = &x;
        let sum = *r1 + *r2;
    "#;
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
#[ignore = "quarantined: shallow conflict diagnostic is not a complete borrow checker"]
fn test_semantic_mutable_and_immutable_borrow_conflict() {
    // Cannot have an immutable borrow active while a mutable borrow exists.
    //
    // Expected: error about conflicting borrows
    let source = r#"
        let mut x = 42;
        let r1 = &mut x;
        let r2 = &x;
        let val = *r1;
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_err(),
        "Should reject simultaneous mutable and immutable borrows"
    );
}

#[test]
#[ignore = "quarantined: shallow conflict diagnostic is not a complete borrow checker"]
fn test_semantic_double_mutable_borrow_conflict() {
    // Cannot have two mutable borrows of the same value at the same time.
    //
    // Expected: error about multiple mutable borrows
    let source = r#"
        let mut x = 42;
        let r1 = &mut x;
        let r2 = &mut x;
        let val = *r1;
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_err(),
        "Should reject two simultaneous mutable borrows"
    );
}

#[test]
#[ignore = "quarantined: nested call borrow is not registered and x is Copy"]
fn test_semantic_borrow_keeps_value_alive() {
    // Borrowing does not move the value. The owner can still use it
    // after the reference is no longer needed.
    //
    // Expected: no error (x is borrowed, not moved)
    let source = r#"
        fn read_value(v: &i32) -> i32 {
            return *v;
        }
        fn main() {
            let x = 42;
            let y = read_value(&x);
            let z = x + y;
        }
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(result.is_ok(), "Borrowing should not move the value");
}

#[test]
#[ignore = "quarantined: unsupported assignment is dropped by compatibility parser recovery"]
fn test_semantic_cannot_mutate_through_immutable_ref() {
    // An immutable reference should not allow mutation of the underlying value.
    //
    // Expected: error about assignment through immutable reference
    let source = r#"
        let x = 42;
        let r = &x;
        *r = 100;
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_err(),
        "Should reject mutation through an immutable reference"
    );
}

// =============================================================================
// Phase 5c: Generics
// =============================================================================
//
// AST types already exist:
//   Statement::Function { type_params: Vec<String>, .. }
//   Statement::StructDef { type_params: Vec<String>, .. }
//   Statement::EnumDef { type_params: Vec<String>, .. }
//   Statement::ImplBlock { type_params: Vec<String>, .. }
//   Type::Generic(String, Vec<Type>)  -- e.g., Container<i32>
//
// The lexer already has < and > tokens (LessThan/GreaterThan) which the
// parser must disambiguate in type position vs expression position.

#[test]
fn test_strict_parse_generic_function_retained_shape() {
    // fn identity<T>(x: T) -> T { return x; }
    // Should parse with type_params = ["T"]
    let source = "fn identity<T>(x: T) -> T { return x; }";
    let ast = strict_parse(source);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Function {
            name,
            parameters,
            return_type,
            type_params,
            trait_bounds,
            ..
        }) => {
            assert_eq!(name, "identity");
            assert_eq!(type_params, &vec!["T".to_string()]);
            assert!(trait_bounds.is_empty());
            assert_eq!(parameters.len(), 1);
            assert_eq!(parameters[0].name, "x");
            assert!(matches!(&parameters[0].param_type, compiler::ast::Type::Named(n) if n == "T"));
            assert!(matches!(return_type, Some(compiler::ast::Type::Named(n)) if n == "T"));
        }
        _ => panic!("Expected generic function definition"),
    }
}

#[test]
fn test_strict_parse_generic_function_multiple_params_retained_shape() {
    // fn pair<A, B>(a: A, b: B) -> (A, B) { ... }
    // Should parse with type_params = ["A", "B"]
    let source = "fn pair<A, B>(a: A, b: B) -> (A, B) { return (a, b); }";
    let ast = strict_parse(source);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Function {
            name,
            parameters,
            type_params,
            return_type,
            ..
        }) => {
            assert_eq!(name, "pair");
            assert_eq!(type_params, &vec!["A".to_string(), "B".to_string()]);
            assert_eq!(parameters.len(), 2);
            assert_eq!(parameters[0].name, "a");
            assert!(
                matches!(&parameters[0].param_type, compiler::ast::Type::Named(name) if name == "A")
            );
            assert_eq!(parameters[1].name, "b");
            assert!(
                matches!(&parameters[1].param_type, compiler::ast::Type::Named(name) if name == "B")
            );
            assert!(matches!(
                return_type,
                Some(compiler::ast::Type::Tuple(types))
                    if matches!(types.as_slice(), [compiler::ast::Type::Named(a), compiler::ast::Type::Named(b)] if a == "A" && b == "B")
            ));
        }
        _ => panic!("Expected generic function definition"),
    }
}

#[test]
fn test_strict_parse_generic_struct_retained_shape() {
    // struct Container<T> { value: T }
    // Should parse with type_params = ["T"]
    let source = "struct Container<T> { value: T }";
    let ast = strict_parse(source);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::StructDef {
            name,
            fields,
            type_params,
        }) => {
            assert_eq!(name, "Container");
            assert_eq!(type_params, &vec!["T".to_string()]);
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].name, "value");
            assert!(matches!(&fields[0].field_type, compiler::ast::Type::Named(n) if n == "T"));
        }
        _ => panic!("Expected generic struct definition"),
    }
}

#[test]
fn test_strict_parse_generic_enum_payloads_retained_shape() {
    // enum Result<T, E> { Ok(T), Err(E) }
    // Should parse with type_params = ["T", "E"]
    let source = "enum Result<T, E> { Ok(T), Err(E) }";
    let ast = strict_parse(source);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::EnumDef {
            name,
            variants,
            type_params,
        }) => {
            assert_eq!(name, "Result");
            assert_eq!(type_params, &vec!["T".to_string(), "E".to_string()]);
            assert_eq!(variants.len(), 2);
            assert_eq!(variants[0].name, "Ok");
            assert_eq!(variants[1].name, "Err");
            assert!(matches!(
                &variants[0].kind,
                compiler::ast::VariantDeclKind::Tuple(types)
                    if matches!(types.as_slice(), [compiler::ast::Type::Named(name)] if name == "T")
            ));
            assert!(matches!(
                &variants[1].kind,
                compiler::ast::VariantDeclKind::Tuple(types)
                    if matches!(types.as_slice(), [compiler::ast::Type::Named(name)] if name == "E")
            ));
        }
        _ => panic!("Expected generic enum definition"),
    }
}

#[test]
fn test_strict_parse_generic_type_annotation_retained_shape() {
    // let x: Container<i32> = ...;
    // Type annotation should be Type::Generic("Container", [Type::Named("i32")])
    let source = "let x: Container<i32> = Container { value: 42 };";
    let ast = strict_parse(source);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Let {
            type_annotation,
            ..
        }) => {
            assert!(
                matches!(
                    type_annotation,
                    Some(compiler::ast::Type::Generic(name, params))
                    if name == "Container" && params.len() == 1
                       && matches!(&params[0], compiler::ast::Type::Named(n) if n == "i32")
                ),
                "Should parse as Type::Generic(\"Container\", [Named(\"i32\")])"
            );
        }
        _ => panic!("Expected let with generic type annotation"),
    }
}

#[test]
#[ignore = "quarantined: generic impl target arguments and declaration bounds are not retained"]
fn test_parse_generic_impl_block() {
    // impl<T> Container<T> { fn new(value: T) -> Container<T> { ... } }
    // ImplBlock should have type_params = ["T"]
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
            methods,
            type_params,
            trait_name,
        }) => {
            assert_eq!(type_name, "Container");
            assert_eq!(type_params, &vec!["T".to_string()]);
            assert!(
                trait_name.is_none(),
                "This is a plain impl, not impl Trait for"
            );
            assert_eq!(methods.len(), 1);
        }
        _ => panic!("Expected generic impl block"),
    }
}

// =============================================================================
// Phase 5d: Traits
// =============================================================================
//
// AST types already exist:
//   Statement::TraitDef { name, type_params, methods: Vec<TraitMethod> }
//   TraitMethod { name, parameters, return_type, body: Option<Block> }
//   Statement::ImplBlock { trait_name: Option<String>, .. }
//
// Lexer needs: `trait` and `where` as keyword tokens
// Parser needs: trait definition, impl Trait for Type, trait bounds on generics

#[test]
fn test_strict_lexer_trait_keyword_stream() {
    // "trait" should tokenize as a keyword, not an Identifier("trait").
    let source = "trait Display { }";
    assert_eq!(
        strict_tokens(source),
        vec![
            Token::Trait,
            Token::Identifier("Display".to_string()),
            Token::LeftBrace,
            Token::RightBrace,
            Token::Eof,
        ]
    );
}

#[test]
fn test_strict_lexer_where_keyword_stream() {
    // "where" should tokenize as a keyword, not an Identifier("where").
    let source = "where T";
    assert_eq!(
        strict_tokens(source),
        vec![Token::Where, Token::Identifier("T".to_string()), Token::Eof]
    );
}

#[test]
fn test_strict_parse_trait_definition_single_method_retained_shape() {
    // trait Printable { fn to_string(&self) -> String; }
    // Should parse as TraitDef with one method signature (no body)
    let source = r#"
        trait Printable {
            fn to_string(&self) -> String;
        }
    "#;
    let ast = strict_parse(source);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::TraitDef {
            name,
            methods,
            type_params,
        }) => {
            assert_eq!(name, "Printable");
            assert!(type_params.is_empty());
            assert_eq!(methods.len(), 1);
            assert_eq!(methods[0].name, "to_string");
            assert!(matches!(
                methods[0].parameters.as_slice(),
                [compiler::ast::Parameter {
                    name,
                    param_type: compiler::ast::Type::Reference(inner, false),
                }] if name == "self" && matches!(inner.as_ref(), compiler::ast::Type::Named(inner_name) if inner_name == "Self")
            ));
            assert!(
                methods[0].body.is_none(),
                "Trait method signatures have no body"
            );
            assert!(matches!(
                &methods[0].return_type,
                Some(compiler::ast::Type::Named(name)) if name == "String"
            ));
        }
        _ => panic!("Expected trait definition"),
    }
}

#[test]
fn test_strict_parse_trait_with_multiple_methods_retained_shape() {
    // trait Shape { fn area(&self) -> f64; fn perimeter(&self) -> f64; }
    let source = r#"
        trait Shape {
            fn area(&self) -> f64;
            fn perimeter(&self) -> f64;
            fn name(&self) -> String;
        }
    "#;
    let ast = strict_parse(source);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::TraitDef {
            name,
            methods,
            ..
        }) => {
            assert_eq!(name, "Shape");
            assert_eq!(methods.len(), 3);
            assert_eq!(methods[0].name, "area");
            assert_eq!(methods[1].name, "perimeter");
            assert_eq!(methods[2].name, "name");
            for method in methods {
                assert!(matches!(
                    method.parameters.as_slice(),
                    [compiler::ast::Parameter {
                        name,
                        param_type: compiler::ast::Type::Reference(inner, false),
                    }] if name == "self" && matches!(inner.as_ref(), compiler::ast::Type::Named(inner_name) if inner_name == "Self")
                ));
                assert!(method.body.is_none());
            }
            assert!(
                matches!(&methods[0].return_type, Some(compiler::ast::Type::Named(name)) if name == "f64")
            );
            assert!(
                matches!(&methods[1].return_type, Some(compiler::ast::Type::Named(name)) if name == "f64")
            );
            assert!(
                matches!(&methods[2].return_type, Some(compiler::ast::Type::Named(name)) if name == "String")
            );
        }
        _ => panic!("Expected trait definition"),
    }
}

#[test]
fn test_strict_parse_impl_trait_for_non_generic_type_retained_shape() {
    // impl Printable for Point { fn to_string(&self) -> String { ... } }
    // ImplBlock with trait_name = Some("Printable"), type_name = "Point"
    let source = r#"
        impl Printable for Point {
            fn to_string(&self) -> String {
                return "point";
            }
        }
    "#;
    let ast = strict_parse(source);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::ImplBlock {
            type_name,
            methods,
            trait_name,
            type_params,
        }) => {
            assert_eq!(type_name, "Point");
            assert_eq!(trait_name, &Some("Printable".to_string()));
            assert!(type_params.is_empty());
            assert_eq!(methods.len(), 1);
            match &methods[0] {
                compiler::ast::Statement::Function {
                    name,
                    parameters,
                    return_type: Some(compiler::ast::Type::Named(return_name)),
                    body,
                    ..
                } => {
                    assert_eq!(name, "to_string");
                    assert_eq!(return_name, "String");
                    assert!(matches!(
                        body.statements.as_slice(),
                        [compiler::ast::Statement::Return(Some(
                            compiler::ast::Expression::StringLiteral(value)
                        ))] if value == "point"
                    ));
                    assert!(body.expression.is_none());
                    assert!(matches!(parameters.as_slice(), [compiler::ast::Parameter {
                        name: self_name,
                        param_type: compiler::ast::Type::Reference(inner, false),
                    }] if self_name == "self" && matches!(inner.as_ref(), compiler::ast::Type::Named(inner_name) if inner_name == "Self")));
                }
                _ => panic!("Expected a retained to_string method"),
            }
        }
        _ => panic!("Expected impl Trait for Type block"),
    }
}

#[test]
fn test_strict_parse_single_trait_bound_retained_metadata() {
    // fn print_item<T: Display>(item: T) { ... }
    // The parser should record that T has a bound of Display.
    // This could be stored as part of type_params or a separate bounds map.
    let source = r#"
        fn print_item<T: Display>(item: T) {
            return;
        }
    "#;
    let ast = strict_parse(source);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Function {
            name,
            type_params,
            trait_bounds,
            ..
        }) => {
            assert_eq!(name, "print_item");
            assert_eq!(type_params, &vec!["T".to_string()]);
            assert_eq!(
                trait_bounds,
                &vec![("T".to_string(), vec!["Display".to_string()])]
            );
        }
        _ => panic!("Expected function with trait-bounded generic"),
    }
}

#[test]
fn test_strict_parse_multiple_trait_bounds_retained_metadata() {
    // fn clone_and_print<T: Display + Clone>(item: T) -> T { ... }
    // T should have bounds [Display, Clone]
    let source = r#"
        fn clone_and_print<T: Display + Clone>(item: T) -> T {
            return item;
        }
    "#;
    let ast = strict_parse(source);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Function {
            name,
            type_params,
            trait_bounds,
            ..
        }) => {
            assert_eq!(name, "clone_and_print");
            assert_eq!(type_params, &vec!["T".to_string()]);
            assert_eq!(
                trait_bounds,
                &vec![(
                    "T".to_string(),
                    vec!["Display".to_string(), "Clone".to_string()]
                )]
            );
        }
        _ => panic!("Expected function with multiple trait bounds"),
    }
}

#[test]
fn test_strict_parse_where_clause_retained_metadata() {
    // fn process<T, U>(source: T, sink: U) where T: Clone, U: Default { ... }
    let source = r#"
        fn process<T, U>(source: T, sink: U) where T: Clone, U: Default {
            return;
        }
    "#;
    let ast = strict_parse(source);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Function {
            name,
            parameters,
            type_params,
            trait_bounds,
            ..
        }) => {
            assert_eq!(name, "process");
            assert_eq!(parameters.len(), 2);
            assert_eq!(type_params.len(), 2);
            assert_eq!(parameters[0].name, "source");
            assert!(
                matches!(&parameters[0].param_type, compiler::ast::Type::Named(name) if name == "T")
            );
            assert_eq!(parameters[1].name, "sink");
            assert!(
                matches!(&parameters[1].param_type, compiler::ast::Type::Named(name) if name == "U")
            );
            assert_eq!(
                trait_bounds,
                &vec![
                    ("T".to_string(), vec!["Clone".to_string()]),
                    ("U".to_string(), vec!["Default".to_string()]),
                ]
            );
        }
        _ => panic!("Expected function with where clause"),
    }
}

#[test]
#[ignore = "quarantined: recovery-path shallow completeness diagnostic is not trait execution"]
fn test_semantic_trait_method_not_implemented() {
    // Implementing a trait but missing a required method should be a compile-time error.
    //
    // Expected: error about missing method 'perimeter' in impl Shape for Circle
    let source = r#"
        trait Shape {
            fn area(&self) -> f64;
            fn perimeter(&self) -> f64;
        }
        struct Circle { radius: f64 }
        impl Shape for Circle {
            fn area(&self) -> f64 {
                return 3.14;
            }
        }
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_err(),
        "Should detect missing trait method 'perimeter'"
    );
}

#[test]
#[ignore = "quarantined: passes on unsupported struct construction before bound checking"]
fn test_semantic_unsatisfied_trait_bound() {
    // Calling a function with a trait-bounded generic using a type that
    // does not implement the required trait should be a compile-time error.
    //
    // Expected: error that Opaque does not implement Display
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
        "Should detect that Opaque does not implement Display"
    );
}

// =============================================================================
// Phase 5 Integration: Combined feature tests
// =============================================================================

#[test]
fn test_strict_parse_generic_struct_reference_field_retained_shape() {
    // struct Ref<T> { value: &T }
    // Combines generic type params with reference types.
    let source = "struct Ref<T> { value: &T }";
    let ast = strict_parse(source);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::StructDef {
            name,
            fields,
            type_params,
        }) => {
            assert_eq!(name, "Ref");
            assert_eq!(type_params, &vec!["T".to_string()]);
            assert_eq!(fields.len(), 1);
            assert_eq!(fields[0].name, "value");
            assert!(
                matches!(&fields[0].field_type,
                    compiler::ast::Type::Reference(inner, false)
                    if matches!(inner.as_ref(), compiler::ast::Type::Named(n) if n == "T")
                ),
                "Field type should be &T"
            );
        }
        _ => panic!("Expected generic struct with reference field"),
    }
}

#[test]
#[ignore = "quarantined: passes on unsupported generic struct construction before ownership"]
fn test_semantic_generic_ownership_transfer() {
    // A generic container owning a non-Copy value should exhibit move semantics.
    //
    // Expected: error on use of b1 after move
    let source = r#"
        struct Box<T> { value: T }
        fn main() {
            let b1 = Box { value: "hello" };
            let b2 = b1;
            let b3 = b1;
        }
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_err(),
        "Generic container with non-Copy content should move"
    );
}

#[test]
#[ignore = "quarantined: unsupported struct and method paths precede receiver-borrow semantics"]
fn test_semantic_trait_method_borrows_self() {
    // A trait method with &self should not move the receiver.
    // Calling it multiple times on the same value should be valid.
    //
    // Expected: no error
    let source = r#"
        trait Describable {
            fn describe(&self) -> String;
        }
        struct Item { name: String }
        impl Describable for Item {
            fn describe(&self) -> String {
                return "item";
            }
        }
        fn main() {
            let item = Item { name: "widget" };
            let d1 = item.describe();
            let d2 = item.describe();
        }
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    let mut analyzer = SemanticAnalyzer::new();
    let result = analyzer.analyze(ast);
    assert!(
        result.is_ok(),
        "Calling &self methods should not move the receiver"
    );
}

#[test]
fn test_strict_parse_generic_function_bound_reference_retained_shape() {
    // fn display_ref<T: Display>(item: &T) { ... }
    // Combines trait bounds on generics with reference parameter types.
    let source = r#"
        fn display_ref<T: Display>(item: &T) {
            return;
        }
    "#;
    let ast = strict_parse(source);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Function {
            name,
            parameters,
            type_params,
            trait_bounds,
            ..
        }) => {
            assert_eq!(name, "display_ref");
            assert_eq!(type_params, &vec!["T".to_string()]);
            assert_eq!(
                trait_bounds,
                &vec![("T".to_string(), vec!["Display".to_string()])]
            );
            assert_eq!(parameters.len(), 1);
            assert_eq!(parameters[0].name, "item");
            assert!(matches!(
                &parameters[0].param_type,
                compiler::ast::Type::Reference(inner, false)
                    if matches!(inner.as_ref(), compiler::ast::Type::Named(name) if name == "T")
            ));
        }
        _ => panic!("Expected generic function with reference param"),
    }
}

#[test]
#[ignore = "quarantined: generic impl target arguments and declaration bounds are not retained"]
fn test_parse_impl_trait_for_generic_struct() {
    // impl<T: Display> Printable for Container<T> { ... }
    // Combines generic type params with trait bounds on an impl block.
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
