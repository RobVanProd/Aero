use crate::lexer::{tokenize_with_locations, Token, LocatedToken};
use crate::errors::SourceLocation;

#[cfg(test)]
mod tests {
    use super::*;

    // ===== PHASE 3 COMPREHENSIVE LEXER TESTS =====

    #[test]
    fn test_function_tokens() {
        let source = "fn add(x: i32, y: i32) -> i32 { return x + y; }";
        let tokens = tokenize_with_locations(source, None);
        
        let token_types: Vec<_> = tokens.iter().map(|t| &t.token).collect();
        
        assert!(token_types.contains(&&Token::Fn));
        assert!(token_types.contains(&&Token::Arrow));
        assert!(token_types.contains(&&Token::Return));
        assert!(token_types.contains(&&Token::LeftBrace));
        assert!(token_types.contains(&&Token::RightBrace));
        assert!(token_types.contains(&&Token::LeftParen));
        assert!(token_types.contains(&&Token::RightParen));
        assert!(token_types.contains(&&Token::Colon));
        assert!(token_types.contains(&&Token::Comma));
    }

    #[test]
    fn test_control_flow_tokens() {
        let source = "if x > 0 { while y < 10 { for i in range { loop { break; continue; } } } } else { }";
        let tokens = tokenize_with_locations(source, None);
        
        let token_types: Vec<_> = tokens.iter().map(|t| &t.token).collect();
        
        assert!(token_types.contains(&&Token::If));
        assert!(token_types.contains(&&Token::Else));
        assert!(token_types.contains(&&Token::While));
        assert!(token_types.contains(&&Token::For));
        assert!(token_types.contains(&&Token::In));
        assert!(token_types.contains(&&Token::Loop));
        assert!(token_types.contains(&&Token::Break));
        assert!(token_types.contains(&&Token::Continue));
    }

    #[test]
    fn test_io_macro_tokens() {
        let source = r#"print!("Hello"); println!("World {}", value);"#;
        let tokens = tokenize_with_locations(source, None);
        
        let token_types: Vec<_> = tokens.iter().map(|t| &t.token).collect();
        
        assert!(token_types.contains(&&Token::PrintMacro));
        assert!(token_types.contains(&&Token::PrintlnMacro));
    }

    #[test]
    fn test_comparison_operators() {
        let source = "x == y != z < a > b <= c >= d";
        let tokens = tokenize_with_locations(source, None);
        
        let token_types: Vec<_> = tokens.iter().map(|t| &t.token).collect();
        
        assert!(token_types.contains(&&Token::Equal));
        assert!(token_types.contains(&&Token::NotEqual));
        assert!(token_types.contains(&&Token::LessThan));
        assert!(token_types.contains(&&Token::GreaterThan));
        assert!(token_types.contains(&&Token::LessEqual));
        assert!(token_types.contains(&&Token::GreaterEqual));
    }

    #[test]
    fn test_logical_operators() {
        let source = "x && y || !z";
        let tokens = tokenize_with_locations(source, None);
        
        let token_types: Vec<_> = tokens.iter().map(|t| &t.token).collect();
        
        assert!(token_types.contains(&&Token::LogicalAnd));
        assert!(token_types.contains(&&Token::LogicalOr));
        assert!(token_types.contains(&&Token::LogicalNot));
    }

    #[test]
    fn test_mutability_tokens() {
        let source = "let mut x = 5; let y = 10;";
        let tokens = tokenize_with_locations(source, None);
        
        let token_types: Vec<_> = tokens.iter().map(|t| &t.token).collect();
        
        assert!(token_types.contains(&&Token::Let));
        assert!(token_types.contains(&&Token::Mut));
    }

    #[test]
    fn test_string_literals() {
        let source = r#""Hello World" "Format: {}" "Multi\nLine""#;
        let tokens = tokenize_with_locations(source, None);
        
        let string_tokens: Vec<_> = tokens.iter()
            .filter_map(|t| match &t.token {
                Token::StringLiteral(s) => Some(s),
                _ => None,
            })
            .collect();
        
        assert_eq!(string_tokens.len(), 3);
        assert_eq!(string_tokens[0], "Hello World");
        assert_eq!(string_tokens[1], "Format: {}");
        assert_eq!(string_tokens[2], "Multi\nLine");
    }

    #[test]
    fn test_numeric_literals() {
        let source = "42 3.14 0 -5 1.0e10";
        let tokens = tokenize_with_locations(source, None);
        
        let int_tokens: Vec<_> = tokens.iter()
            .filter_map(|t| match &t.token {
                Token::IntegerLiteral(n) => Some(*n),
                _ => None,
            })
            .collect();
        
        let float_tokens: Vec<_> = tokens.iter()
            .filter_map(|t| match &t.token {
                Token::FloatLiteral(f) => Some(*f),
                _ => None,
            })
            .collect();
        
        assert!(int_tokens.contains(&42));
        assert!(int_tokens.contains(&0));
        assert!(float_tokens.contains(&3.14));
    }

    #[test]
    fn test_identifier_vs_keyword_disambiguation() {
        let source = "fn function_name if_var while_loop for_each";
        let tokens = tokenize_with_locations(source, None);
        
        // fn should be keyword, function_name should be identifier
        assert_eq!(tokens[0].token, Token::Fn);
        assert!(matches!(tokens[1].token, Token::Identifier(ref s) if s == "function_name"));
        
        // if_var should be identifier, not if keyword
        assert!(matches!(tokens[2].token, Token::Identifier(ref s) if s == "if_var"));
        
        // while_loop should be identifier, not while keyword
        assert!(matches!(tokens[3].token, Token::Identifier(ref s) if s == "while_loop"));
        
        // for_each should be identifier, not for keyword
        assert!(matches!(tokens[4].token, Token::Identifier(ref s) if s == "for_each"));
    }

    #[test]
    fn test_multi_character_operators() {
        let source = "== != <= >= && || -> ::";
        let tokens = tokenize_with_locations(source, None);
        
        let expected_tokens = vec![
            Token::Equal,
            Token::NotEqual,
            Token::LessEqual,
            Token::GreaterEqual,
            Token::LogicalAnd,
            Token::LogicalOr,
            Token::Arrow,
        ];
        
        for (i, expected) in expected_tokens.iter().enumerate() {
            assert_eq!(&tokens[i].token, expected);
        }
    }

    #[test]
    fn test_whitespace_and_comments() {
        let source = "let x = 5; // This is a comment\n/* Multi-line\n   comment */ let y = 10;";
        let tokens = tokenize_with_locations(source, None);
        
        // Comments should be filtered out, only actual tokens remain
        let token_types: Vec<_> = tokens.iter().map(|t| &t.token).collect();
        
        assert!(token_types.contains(&&Token::Let));
        assert!(token_types.contains(&&Token::Identifier("x".to_string())));
        assert!(token_types.contains(&&Token::Assign));
        assert!(token_types.contains(&&Token::IntegerLiteral(5)));
        assert!(token_types.contains(&&Token::Semicolon));
        assert!(token_types.contains(&&Token::Identifier("y".to_string())));
        assert!(token_types.contains(&&Token::IntegerLiteral(10)));
        
        // Should not contain comment tokens
        assert!(!token_types.iter().any(|t| matches!(t, Token::Comment(_))));
    }

    #[test]
    fn test_complex_function_signature() {
        let source = "fn complex_func(a: i32, b: f64, c: bool) -> (i32, f64) {";
        let tokens = tokenize_with_locations(source, None);
        
        let expected_sequence = vec![
            Token::Fn,
            Token::Identifier("complex_func".to_string()),
            Token::LeftParen,
            Token::Identifier("a".to_string()),
            Token::Colon,
            Token::Identifier("i32".to_string()),
            Token::Comma,
            Token::Identifier("b".to_string()),
            Token::Colon,
            Token::Identifier("f64".to_string()),
            Token::Comma,
            Token::Identifier("c".to_string()),
            Token::Colon,
            Token::Identifier("bool".to_string()),
            Token::RightParen,
            Token::Arrow,
            Token::LeftParen,
            Token::Identifier("i32".to_string()),
            Token::Comma,
            Token::Identifier("f64".to_string()),
            Token::RightParen,
            Token::LeftBrace,
        ];
        
        for (i, expected) in expected_sequence.iter().enumerate() {
            assert_eq!(&tokens[i].token, expected, "Token mismatch at position {}", i);
        }
    }

    #[test]
    fn test_nested_control_flow() {
        let source = "if (x > 0) { while (y < 10) { if (z == 5) { break; } } }";
        let tokens = tokenize_with_locations(source, None);
        
        let control_flow_tokens: Vec<_> = tokens.iter()
            .filter(|t| matches!(t.token, 
                Token::If | Token::While | Token::Break | Token::Continue |
                Token::For | Token::Loop | Token::Else
            ))
            .map(|t| &t.token)
            .collect();
        
        assert_eq!(control_flow_tokens.len(), 4); // if, while, if, break
        assert_eq!(control_flow_tokens[0], &Token::If);
        assert_eq!(control_flow_tokens[1], &Token::While);
        assert_eq!(control_flow_tokens[2], &Token::If);
        assert_eq!(control_flow_tokens[3], &Token::Break);
    }

    #[test]
    fn test_print_macro_with_format_strings() {
        let source = r#"print!("Value: {}", x); println!("Debug: {} = {}", name, value);"#;
        let tokens = tokenize_with_locations(source, None);
        
        // Find print macros
        let print_tokens: Vec<_> = tokens.iter()
            .filter(|t| matches!(t.token, Token::PrintMacro | Token::PrintlnMacro))
            .collect();
        
        assert_eq!(print_tokens.len(), 2);
        assert_eq!(print_tokens[0].token, Token::PrintMacro);
        assert_eq!(print_tokens[1].token, Token::PrintlnMacro);
        
        // Check format strings are tokenized correctly
        let string_tokens: Vec<_> = tokens.iter()
            .filter_map(|t| match &t.token {
                Token::StringLiteral(s) => Some(s),
                _ => None,
            })
            .collect();
        
        assert!(string_tokens.contains(&&"Value: {}".to_string()));
        assert!(string_tokens.contains(&&"Debug: {} = {}".to_string()));
    }

    // ===== SOURCE LOCATION TRACKING TESTS =====

    #[test]
    fn test_source_location_tracking() {
        let source = "let x = 5;\nlet y = 10;";
        let tokens = tokenize_with_locations(source, Some("test.aero".to_string()));
        
        // Check first token (let)
        assert_eq!(tokens[0].token, Token::Let);
        assert_eq!(tokens[0].location.line, 1);
        assert_eq!(tokens[0].location.column, 1);
        assert_eq!(tokens[0].location.filename, Some("test.aero".to_string()));
        
        // Check identifier (x)
        assert_eq!(tokens[1].token, Token::Identifier("x".to_string()));
        assert_eq!(tokens[1].location.line, 1);
        assert_eq!(tokens[1].location.column, 5);
        
        // Check second let on line 2
        let second_let_index = tokens.iter().position(|t| matches!(t.token, Token::Let) && t.location.line == 2).unwrap();
        assert_eq!(tokens[second_let_index].location.line, 2);
        assert_eq!(tokens[second_let_index].location.column, 1);
    }

    #[test]
    fn test_multiline_location_tracking() {
        let source = "fn main() {\n    let x = 5;\n    return x;\n}";
        let tokens = tokenize_with_locations(source, None);
        
        // Find the return token
        let return_token = tokens.iter().find(|t| matches!(t.token, Token::Return)).unwrap();
        assert_eq!(return_token.location.line, 3);
        assert_eq!(return_token.location.column, 5);
    }

    #[test]
    fn test_error_location_display() {
        let location = SourceLocation::with_filename(10, 25, "example.aero".to_string());
        assert_eq!(format!("{}", location), "example.aero:10:25");
        
        let location_no_file = SourceLocation::new(5, 12);
        assert_eq!(format!("{}", location_no_file), "5:12");
    }

    #[test]
    fn test_complex_expression_locations() {
        let source = "if x >= 5 && y != 10 {\n    println!(\"test\");\n}";
        let tokens = tokenize_with_locations(source, None);
        
        // Find the >= operator
        let ge_token = tokens.iter().find(|t| matches!(t.token, Token::GreaterEqual)).unwrap();
        assert_eq!(ge_token.location.line, 1);
        assert_eq!(ge_token.location.column, 5);
        
        // Find the println! macro
        let println_token = tokens.iter().find(|t| matches!(t.token, Token::PrintlnMacro)).unwrap();
        assert_eq!(println_token.location.line, 2);
        assert_eq!(println_token.location.column, 5);
    }

    // ===== EDGE CASE TESTS =====

    #[test]
    fn test_empty_input() {
        let source = "";
        let tokens = tokenize_with_locations(source, None);
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn test_whitespace_only() {
        let source = "   \n\t  \r\n  ";
        let tokens = tokenize_with_locations(source, None);
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn test_single_character_tokens() {
        let source = "(){}[];,.:+-*/=%<>!&|";
        let tokens = tokenize_with_locations(source, None);
        
        // Should tokenize each character appropriately
        assert!(tokens.len() > 0);
        assert!(tokens.iter().any(|t| matches!(t.token, Token::LeftParen)));
        assert!(tokens.iter().any(|t| matches!(t.token, Token::RightParen)));
        assert!(tokens.iter().any(|t| matches!(t.token, Token::LeftBrace)));
        assert!(tokens.iter().any(|t| matches!(t.token, Token::RightBrace)));
    }

    #[test]
    fn test_number_edge_cases() {
        let source = "0 123 0.0 123.456 1e10 2.5e-3";
        let tokens = tokenize_with_locations(source, None);
        
        let int_count = tokens.iter().filter(|t| matches!(t.token, Token::IntegerLiteral(_))).count();
        let float_count = tokens.iter().filter(|t| matches!(t.token, Token::FloatLiteral(_))).count();
        
        assert!(int_count >= 2); // At least 0 and 123
        assert!(float_count >= 2); // At least some float literals
    }

    #[test]
    fn test_string_escape_sequences() {
        let source = r#""hello\nworld" "tab\there" "quote\"test""#;
        let tokens = tokenize_with_locations(source, None);
        
        let string_tokens: Vec<_> = tokens.iter()
            .filter_map(|t| match &t.token {
                Token::StringLiteral(s) => Some(s),
                _ => None,
            })
            .collect();
        
        assert_eq!(string_tokens.len(), 3);
        assert_eq!(string_tokens[0], "hello\nworld");
        assert_eq!(string_tokens[1], "tab\there");
        assert_eq!(string_tokens[2], "quote\"test");
    }
}