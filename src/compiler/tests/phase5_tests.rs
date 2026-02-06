use compiler::lexer;
use compiler::parser;
use compiler::semantic_analyzer::SemanticAnalyzer;

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
#[ignore] // Phase 5a: semantic analyzer needs ownership tracking
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
#[ignore] // Phase 5a: semantic analyzer needs Copy type distinction
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
#[ignore] // Phase 5a: semantic analyzer needs ownership tracking across function calls
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
#[ignore] // Phase 5a: semantic analyzer needs ownership return tracking
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
#[ignore] // Phase 5a: semantic analyzer needs struct move semantics
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
#[ignore] // Phase 5b: lexer needs single & token
fn test_lexer_ampersand_token() {
    // The lexer should recognize a single & as a borrow/reference operator,
    // distinct from && (logical and).
    let source = "let r = &x;";
    let tokens = lexer::tokenize(source);
    // Expected tokens: Let, Identifier("r"), Assign, Ampersand, Identifier("x"), Semicolon, Eof
    // Currently single & prints an error. After Phase 5b, it should be a valid token.
    assert!(tokens.len() >= 6, "Should tokenize & as a standalone token");
}

#[test]
#[ignore] // Phase 5b: lexer needs single & token
fn test_lexer_ampersand_mut_tokens() {
    // &mut should tokenize as Ampersand followed by the existing Mut keyword.
    let source = "let r = &mut x;";
    let tokens = lexer::tokenize(source);
    // Expected: Let, Identifier("r"), Assign, Ampersand, Mut, Identifier("x"), Semicolon, Eof
    assert!(tokens.len() >= 7, "Should tokenize &mut as Ampersand + Mut");
}

#[test]
#[ignore] // Phase 5b: parser needs Type::Reference
fn test_parse_immutable_reference_param() {
    // Function parameter typed as &String should parse as
    // Type::Reference(Box<Type::Named("String")>, false).
    let source = "fn calc_length(s: &String) -> i32 { return 0; }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
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
#[ignore] // Phase 5b: parser needs Type::Reference with mutable flag
fn test_parse_mutable_reference_param() {
    // Function parameter typed as &mut String should parse as
    // Type::Reference(Box<Type::Named("String")>, true).
    let source = "fn append(s: &mut String) { return; }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
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
#[ignore] // Phase 5b: parser needs Expression::Borrow
fn test_parse_borrow_expression() {
    // Taking a reference with & should parse as Expression::Borrow { mutable: false }.
    let source = "let r = &x;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Let {
            name, value, ..
        }) => {
            assert_eq!(name, "r");
            assert!(
                matches!(
                    value,
                    Some(compiler::ast::Expression::Borrow { mutable: false, .. })
                ),
                "Should parse & as an immutable borrow expression"
            );
        }
        _ => panic!("Expected let statement with borrow"),
    }
}

#[test]
#[ignore] // Phase 5b: parser needs Expression::Borrow with mutable flag
fn test_parse_mutable_borrow_expression() {
    // Taking a mutable reference with &mut should parse as Expression::Borrow { mutable: true }.
    let source = "let r = &mut x;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Let {
            name, value, ..
        }) => {
            assert_eq!(name, "r");
            assert!(
                matches!(
                    value,
                    Some(compiler::ast::Expression::Borrow { mutable: true, .. })
                ),
                "Should parse &mut as a mutable borrow expression"
            );
        }
        _ => panic!("Expected let statement with mutable borrow"),
    }
}

#[test]
#[ignore] // Phase 5b: parser needs Expression::Deref
fn test_parse_deref_expression() {
    // The dereference operator * should parse as Expression::Deref.
    let source = "let val = *r;";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Let {
            name, value, ..
        }) => {
            assert_eq!(name, "val");
            assert!(
                matches!(value, Some(compiler::ast::Expression::Deref(_))),
                "Should parse *r as a Deref expression"
            );
        }
        _ => panic!("Expected let statement with deref"),
    }
}

#[test]
#[ignore] // Phase 5b: semantic analyzer needs borrow checker
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
#[ignore] // Phase 5b: semantic analyzer needs borrow checker
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
#[ignore] // Phase 5b: semantic analyzer needs borrow checker
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
#[ignore] // Phase 5b: semantic analyzer needs borrow checker
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
#[ignore] // Phase 5b: semantic analyzer needs borrow checker
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
#[ignore] // Phase 5c: parser needs generic function syntax
fn test_parse_generic_function() {
    // fn identity<T>(x: T) -> T { return x; }
    // Should parse with type_params = ["T"]
    let source = "fn identity<T>(x: T) -> T { return x; }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Function {
            name,
            parameters,
            return_type,
            type_params,
            ..
        }) => {
            assert_eq!(name, "identity");
            assert_eq!(type_params, &vec!["T".to_string()]);
            assert_eq!(parameters.len(), 1);
            assert_eq!(parameters[0].name, "x");
            assert!(matches!(&parameters[0].param_type, compiler::ast::Type::Named(n) if n == "T"));
            assert!(matches!(return_type, Some(compiler::ast::Type::Named(n)) if n == "T"));
        }
        _ => panic!("Expected generic function definition"),
    }
}

#[test]
#[ignore] // Phase 5c: parser needs generic function syntax
fn test_parse_generic_function_multiple_params() {
    // fn pair<A, B>(a: A, b: B) -> (A, B) { ... }
    // Should parse with type_params = ["A", "B"]
    let source = "fn pair<A, B>(a: A, b: B) -> (A, B) { return (a, b); }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
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
            assert!(
                matches!(return_type, Some(compiler::ast::Type::Tuple(types)) if types.len() == 2)
            );
        }
        _ => panic!("Expected generic function definition"),
    }
}

#[test]
#[ignore] // Phase 5c: parser needs generic struct syntax
fn test_parse_generic_struct() {
    // struct Container<T> { value: T }
    // Should parse with type_params = ["T"]
    let source = "struct Container<T> { value: T }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
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
#[ignore] // Phase 5c: parser needs generic enum syntax
fn test_parse_generic_enum() {
    // enum Result<T, E> { Ok(T), Err(E) }
    // Should parse with type_params = ["T", "E"]
    let source = "enum Result<T, E> { Ok(T), Err(E) }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
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
        }
        _ => panic!("Expected generic enum definition"),
    }
}

#[test]
#[ignore] // Phase 5c: parser needs Type::Generic in annotations
fn test_parse_generic_type_annotation() {
    // let x: Container<i32> = ...;
    // Type annotation should be Type::Generic("Container", [Type::Named("i32")])
    let source = "let x: Container<i32> = Container { value: 42 };";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
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
#[ignore] // Phase 5c: parser needs generic impl blocks
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
#[ignore] // Phase 5d: lexer needs trait keyword
fn test_lexer_trait_keyword() {
    // "trait" should tokenize as a keyword, not an Identifier("trait").
    let source = "trait Display { }";
    let tokens = lexer::tokenize(source);
    assert!(
        !matches!(&tokens[0], compiler::lexer::Token::Identifier(s) if s == "trait"),
        "trait should be a keyword, not an identifier"
    );
}

#[test]
#[ignore] // Phase 5d: lexer needs where keyword
fn test_lexer_where_keyword() {
    // "where" should tokenize as a keyword, not an Identifier("where").
    let source = "where T";
    let tokens = lexer::tokenize(source);
    assert!(
        !matches!(&tokens[0], compiler::lexer::Token::Identifier(s) if s == "where"),
        "where should be a keyword, not an identifier"
    );
}

#[test]
#[ignore] // Phase 5d: parser needs Statement::TraitDef
fn test_parse_trait_definition_single_method() {
    // trait Printable { fn to_string(&self) -> String; }
    // Should parse as TraitDef with one method signature (no body)
    let source = r#"
        trait Printable {
            fn to_string(&self) -> String;
        }
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
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
            assert!(
                methods[0].body.is_none(),
                "Trait method signatures have no body"
            );
            assert!(methods[0].return_type.is_some());
        }
        _ => panic!("Expected trait definition"),
    }
}

#[test]
#[ignore] // Phase 5d: parser needs Statement::TraitDef
fn test_parse_trait_with_multiple_methods() {
    // trait Shape { fn area(&self) -> f64; fn perimeter(&self) -> f64; }
    let source = r#"
        trait Shape {
            fn area(&self) -> f64;
            fn perimeter(&self) -> f64;
            fn name(&self) -> String;
        }
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
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
        }
        _ => panic!("Expected trait definition"),
    }
}

#[test]
#[ignore] // Phase 5d: parser needs impl Trait for Type syntax
fn test_parse_impl_trait_for_type() {
    // impl Printable for Point { fn to_string(&self) -> String { ... } }
    // ImplBlock with trait_name = Some("Printable"), type_name = "Point"
    let source = r#"
        impl Printable for Point {
            fn to_string(&self) -> String {
                return "point";
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
            trait_name,
            type_params,
        }) => {
            assert_eq!(type_name, "Point");
            assert_eq!(trait_name, &Some("Printable".to_string()));
            assert!(type_params.is_empty());
            assert_eq!(methods.len(), 1);
        }
        _ => panic!("Expected impl Trait for Type block"),
    }
}

#[test]
#[ignore] // Phase 5d: parser needs trait bound syntax on generics
fn test_parse_trait_bound_on_generic() {
    // fn print_item<T: Display>(item: T) { ... }
    // The parser should record that T has a bound of Display.
    // This could be stored as part of type_params or a separate bounds map.
    let source = r#"
        fn print_item<T: Display>(item: T) {
            return;
        }
    "#;
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
            // type_params should contain "T" (and its bound "Display" somewhere)
            assert!(!type_params.is_empty(), "Should have generic type params");
        }
        _ => panic!("Expected function with trait-bounded generic"),
    }
}

#[test]
#[ignore] // Phase 5d: parser needs multiple trait bounds with +
fn test_parse_multiple_trait_bounds() {
    // fn clone_and_print<T: Display + Clone>(item: T) -> T { ... }
    // T should have bounds [Display, Clone]
    let source = r#"
        fn clone_and_print<T: Display + Clone>(item: T) -> T {
            return item;
        }
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Function {
            name,
            type_params,
            ..
        }) => {
            assert_eq!(name, "clone_and_print");
            assert!(!type_params.is_empty());
        }
        _ => panic!("Expected function with multiple trait bounds"),
    }
}

#[test]
#[ignore] // Phase 5d: parser needs where clause syntax
fn test_parse_where_clause() {
    // fn process<T, U>(source: T, sink: U) where T: Clone, U: Default { ... }
    let source = r#"
        fn process<T, U>(source: T, sink: U) where T: Clone, U: Default {
            return;
        }
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Function {
            name,
            parameters,
            type_params,
            ..
        }) => {
            assert_eq!(name, "process");
            assert_eq!(parameters.len(), 2);
            assert_eq!(type_params.len(), 2);
        }
        _ => panic!("Expected function with where clause"),
    }
}

#[test]
#[ignore] // Phase 5d: semantic analyzer needs trait completeness checking
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
#[ignore] // Phase 5d: semantic analyzer needs trait bound verification
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
#[ignore] // Phase 5 integration: generics + borrowing
fn test_parse_generic_struct_with_reference_field() {
    // struct Ref<T> { value: &T }
    // Combines generic type params with reference types.
    let source = "struct Ref<T> { value: &T }";
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
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
#[ignore] // Phase 5 integration: ownership + generics
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
#[ignore] // Phase 5 integration: traits + borrowing
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
#[ignore] // Phase 5 integration: generics + traits + borrowing
fn test_parse_generic_function_with_trait_bound_and_ref() {
    // fn display_ref<T: Display>(item: &T) { ... }
    // Combines trait bounds on generics with reference parameter types.
    let source = r#"
        fn display_ref<T: Display>(item: &T) {
            return;
        }
    "#;
    let tokens = lexer::tokenize(source);
    let ast = parser::parse(tokens);
    assert_eq!(ast.len(), 1);
    match &ast[0] {
        compiler::ast::AstNode::Statement(compiler::ast::Statement::Function {
            name,
            parameters,
            type_params,
            ..
        }) => {
            assert_eq!(name, "display_ref");
            assert!(!type_params.is_empty());
            assert_eq!(parameters.len(), 1);
            assert!(
                matches!(
                    &parameters[0].param_type,
                    compiler::ast::Type::Reference(_, false)
                ),
                "Parameter should be a reference type"
            );
        }
        _ => panic!("Expected generic function with reference param"),
    }
}

#[test]
#[ignore] // Phase 5 integration: traits + generics in impl
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
