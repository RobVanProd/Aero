#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    IntegerLiteral(i64),
    FloatLiteral(f64),
    Identifier(String),
    
    // Keywords
    Let,
    Fn,
    Return,
    Mut,
    
    // Control flow keywords
    If,
    Else,
    While,
    For,
    In,
    Loop,
    Break,
    Continue,
    
    // I/O Macros
    PrintMacro,    // print!
    PrintlnMacro,  // println!
    
    // Operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Assign,
    Arrow,  // ->
    
    // Comparison operators
    Equal,        // ==
    NotEqual,     // !=
    LessThan,     // <
    GreaterThan,  // >
    LessEqual,    // <=
    GreaterEqual, // >=
    
    // Logical operators
    LogicalAnd,   // &&
    LogicalOr,    // ||
    LogicalNot,   // !
    
    // Delimiters
    Semicolon,
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Dot,
    Colon,
    Comma,
    
    // End of file
    Eof,
}

pub fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            // Whitespace
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            // Operators and delimiters
            '+' => { tokens.push(Token::Plus); chars.next(); }
            '*' => { tokens.push(Token::Multiply); chars.next(); }
            '/' => {
                chars.next(); // consume first '/'
                if let Some(&'/') = chars.peek() {
                    // Line comment - consume until end of line
                    chars.next(); // consume second '/'
                    while let Some(&c) = chars.peek() {
                        if c == '\n' || c == '\r' {
                            break;
                        }
                        chars.next();
                    }
                } else {
                    tokens.push(Token::Divide);
                }
            }
            '%' => { tokens.push(Token::Modulo); chars.next(); }
            ';' => { tokens.push(Token::Semicolon); chars.next(); }
            '{' => { tokens.push(Token::LeftBrace); chars.next(); }
            '}' => { tokens.push(Token::RightBrace); chars.next(); }
            '(' => { tokens.push(Token::LeftParen); chars.next(); }
            ')' => { tokens.push(Token::RightParen); chars.next(); }
            ':' => { tokens.push(Token::Colon); chars.next(); }
            ',' => { tokens.push(Token::Comma); chars.next(); }
            // Handle minus and arrow (->)
            '-' => {
                chars.next(); // consume '-'
                if let Some(&'>') = chars.peek() {
                    chars.next(); // consume '>'
                    tokens.push(Token::Arrow);
                } else {
                    tokens.push(Token::Minus);
                }
            }
            // Handle assignment and equality
            '=' => {
                chars.next(); // consume '='
                if let Some(&'=') = chars.peek() {
                    chars.next(); // consume second '='
                    tokens.push(Token::Equal);
                } else {
                    tokens.push(Token::Assign);
                }
            }
            // Handle not equal and logical not
            '!' => {
                chars.next(); // consume '!'
                if let Some(&'=') = chars.peek() {
                    chars.next(); // consume '='
                    tokens.push(Token::NotEqual);
                } else {
                    tokens.push(Token::LogicalNot);
                }
            }
            // Handle less than and less equal
            '<' => {
                chars.next(); // consume '<'
                if let Some(&'=') = chars.peek() {
                    chars.next(); // consume '='
                    tokens.push(Token::LessEqual);
                } else {
                    tokens.push(Token::LessThan);
                }
            }
            // Handle greater than and greater equal
            '>' => {
                chars.next(); // consume '>'
                if let Some(&'=') = chars.peek() {
                    chars.next(); // consume '='
                    tokens.push(Token::GreaterEqual);
                } else {
                    tokens.push(Token::GreaterThan);
                }
            }
            // Handle logical and
            '&' => {
                chars.next(); // consume '&'
                if let Some(&'&') = chars.peek() {
                    chars.next(); // consume second '&'
                    tokens.push(Token::LogicalAnd);
                } else {
                    // Single & not supported yet, treat as unexpected
                    eprintln!("Unexpected character: &");
                }
            }
            // Handle logical or
            '|' => {
                chars.next(); // consume '|'
                if let Some(&'|') = chars.peek() {
                    chars.next(); // consume second '|'
                    tokens.push(Token::LogicalOr);
                } else {
                    // Single | not supported yet, treat as unexpected
                    eprintln!("Unexpected character: |");
                }
            }
            // Dot operator - handle carefully to avoid conflicts with range operator
            '.' => {
                chars.next(); // consume the '.'
                // Check if this is a range operator (..) or a float literal starting with .
                if let Some(&next_char) = chars.peek() {
                    if next_char == '.' {
                        // This is part of a range operator, just emit a single dot
                        tokens.push(Token::Dot);
                    } else if next_char.is_ascii_digit() {
                        // Check if the previous token was a Dot - if so, this is part of a range operator
                        let is_range_operator = if let Some(last_token) = tokens.last() {
                            matches!(last_token, Token::Dot)
                        } else {
                            false
                        };
                        
                        if is_range_operator {
                            // This is the second dot in a range operator followed by a number
                            // Just emit a dot and let the number be parsed separately
                            tokens.push(Token::Dot);
                        } else {
                            // This is a float literal like .5
                            let mut num_str = String::from("0.");
                            while let Some(&digit) = chars.peek() {
                                if digit.is_ascii_digit() {
                                    num_str.push(chars.next().unwrap());
                                } else {
                                    break;
                                }
                            }
                            // Handle scientific notation (e.g., .5e3)
                            if let Some(&e_char) = chars.peek() {
                                if e_char == 'e' || e_char == 'E' {
                                    num_str.push(chars.next().unwrap());
                                    if let Some(&sign) = chars.peek() {
                                        if sign == '+' || sign == '-' {
                                            num_str.push(chars.next().unwrap());
                                        }
                                    }
                                    while let Some(&digit) = chars.peek() {
                                        if digit.is_ascii_digit() {
                                            num_str.push(chars.next().unwrap());
                                        } else {
                                            break;
                                        }
                                    }
                                }
                            }
                            let float_val: f64 = num_str.parse().unwrap_or(0.0);
                            tokens.push(Token::FloatLiteral(float_val));
                        }
                    } else {
                        // Just a dot, not a float literal
                        tokens.push(Token::Dot);
                    }
                } else {
                    tokens.push(Token::Dot);
                }
            }
            // Integer and Float Literals
            '0'..='9' => {
                let mut num_str = String::new();
                let mut has_dot = false;
                let mut has_exponent = false;
                
                // Collect digits and decimal point
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() {
                        num_str.push(chars.next().unwrap());
                    } else if d == '.' && !has_dot && !has_exponent {
                        // Look ahead to see if this is a range operator (..) or a float
                        let mut lookahead = chars.clone();
                        lookahead.next(); // consume the '.'
                        if let Some(&next_char) = lookahead.peek() {
                            if next_char == '.' {
                                // This is a range operator, don't consume the dot
                                break;
                            } else if next_char.is_ascii_digit() {
                                // This is a float literal
                                has_dot = true;
                                num_str.push(chars.next().unwrap());
                            } else {
                                // Single dot followed by non-digit, don't consume
                                break;
                            }
                        } else {
                            // End of input, don't consume the dot
                            break;
                        }
                    } else if (d == 'e' || d == 'E') && !has_exponent {
                        has_exponent = true;
                        num_str.push(chars.next().unwrap());
                        // Handle optional sign after exponent
                        if let Some(&sign) = chars.peek() {
                            if sign == '+' || sign == '-' {
                                num_str.push(chars.next().unwrap());
                            }
                        }
                    } else {
                        break;
                    }
                }
                
                if has_dot || has_exponent {
                    let float_val: f64 = num_str.parse().unwrap_or(0.0);
                    tokens.push(Token::FloatLiteral(float_val));
                } else {
                    let int_val: i64 = num_str.parse().unwrap_or(0);
                    tokens.push(Token::IntegerLiteral(int_val));
                }
            }
            // String literals
            '"' => {
                chars.next(); // consume opening quote
                let mut string_content = String::from("\"");
                while let Some(&c) = chars.peek() {
                    if c == '"' {
                        string_content.push(chars.next().unwrap()); // consume closing quote
                        break;
                    } else if c == '\\' {
                        // Handle escape sequences
                        string_content.push(chars.next().unwrap()); // consume backslash
                        if let Some(&escaped) = chars.peek() {
                            string_content.push(chars.next().unwrap()); // consume escaped char
                        }
                    } else {
                        string_content.push(chars.next().unwrap());
                    }
                }
                tokens.push(Token::Identifier(string_content));
            }
            // Identifiers and Keywords
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident_str = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_alphanumeric() || d == '_' {
                        ident_str.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                
                // Check for I/O macros (identifiers followed by !)
                if let Some(&'!') = chars.peek() {
                    let token = match ident_str.as_str() {
                        "print" => {
                            chars.next(); // consume '!'
                            Token::PrintMacro
                        }
                        "println" => {
                            chars.next(); // consume '!'
                            Token::PrintlnMacro
                        }
                        _ => Token::Identifier(ident_str), // Regular identifier, don't consume '!'
                    };
                    tokens.push(token);
                } else {
                    // Regular keywords and identifiers
                    let token = match ident_str.as_str() {
                        "let" => Token::Let,
                        "fn" => Token::Fn,
                        "return" => Token::Return,
                        "mut" => Token::Mut,
                        "if" => Token::If,
                        "else" => Token::Else,
                        "while" => Token::While,
                        "for" => Token::For,
                        "in" => Token::In,
                        "loop" => Token::Loop,
                        "break" => Token::Break,
                        "continue" => Token::Continue,
                        _ => Token::Identifier(ident_str),
                    };
                    tokens.push(token);
                }
            }
            _ => {
                // Handle unexpected characters or errors
                eprintln!("Unexpected character: {}", c);
                chars.next();
            }
        }
    }
    
    tokens.push(Token::Eof);
    tokens
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_tokens() {
        let source = "fn main() -> i32 { let mut x = 5; }";
        let tokens = tokenize(source);
        
        assert_eq!(tokens[0], Token::Fn);
        assert_eq!(tokens[1], Token::Identifier("main".to_string()));
        assert_eq!(tokens[2], Token::LeftParen);
        assert_eq!(tokens[3], Token::RightParen);
        assert_eq!(tokens[4], Token::Arrow);
        assert_eq!(tokens[5], Token::Identifier("i32".to_string()));
        assert_eq!(tokens[6], Token::LeftBrace);
        assert_eq!(tokens[7], Token::Let);
        assert_eq!(tokens[8], Token::Mut);
        assert_eq!(tokens[9], Token::Identifier("x".to_string()));
        assert_eq!(tokens[10], Token::Assign);
        assert_eq!(tokens[11], Token::IntegerLiteral(5));
        assert_eq!(tokens[12], Token::Semicolon);
        assert_eq!(tokens[13], Token::RightBrace);
        assert_eq!(tokens[14], Token::Eof);
    }

    #[test]
    fn test_arrow_operator() {
        let source = "-> ->";
        let tokens = tokenize(source);
        
        assert_eq!(tokens[0], Token::Arrow);
        assert_eq!(tokens[1], Token::Arrow);
        assert_eq!(tokens[2], Token::Eof);
    }

    #[test]
    fn test_minus_vs_arrow() {
        let source = "- -> -";
        let tokens = tokenize(source);
        
        assert_eq!(tokens[0], Token::Minus);
        assert_eq!(tokens[1], Token::Arrow);
        assert_eq!(tokens[2], Token::Minus);
        assert_eq!(tokens[3], Token::Eof);
    }

    #[test]
    fn test_keywords() {
        let source = "let fn return mut";
        let tokens = tokenize(source);
        
        assert_eq!(tokens[0], Token::Let);
        assert_eq!(tokens[1], Token::Fn);
        assert_eq!(tokens[2], Token::Return);
        assert_eq!(tokens[3], Token::Mut);
        assert_eq!(tokens[4], Token::Eof);
    }

    #[test]
    fn test_control_flow_keywords() {
        let source = "if else while for in loop break continue";
        let tokens = tokenize(source);
        
        assert_eq!(tokens[0], Token::If);
        assert_eq!(tokens[1], Token::Else);
        assert_eq!(tokens[2], Token::While);
        assert_eq!(tokens[3], Token::For);
        assert_eq!(tokens[4], Token::In);
        assert_eq!(tokens[5], Token::Loop);
        assert_eq!(tokens[6], Token::Break);
        assert_eq!(tokens[7], Token::Continue);
        assert_eq!(tokens[8], Token::Eof);
    }

    #[test]
    fn test_if_else_statement() {
        let source = "if x > 5 { break; } else { continue; }";
        let tokens = tokenize(source);
        
        assert_eq!(tokens[0], Token::If);
        assert_eq!(tokens[1], Token::Identifier("x".to_string()));
        assert_eq!(tokens[2], Token::GreaterThan); // Now properly tokenized
        assert_eq!(tokens[3], Token::IntegerLiteral(5));
        assert_eq!(tokens[4], Token::LeftBrace);
        assert_eq!(tokens[5], Token::Break);
        assert_eq!(tokens[6], Token::Semicolon);
        assert_eq!(tokens[7], Token::RightBrace);
        assert_eq!(tokens[8], Token::Else);
        assert_eq!(tokens[9], Token::LeftBrace);
        assert_eq!(tokens[10], Token::Continue);
        assert_eq!(tokens[11], Token::Semicolon);
        assert_eq!(tokens[12], Token::RightBrace);
        assert_eq!(tokens[13], Token::Eof);
    }

    #[test]
    fn test_while_loop() {
        let source = "while i < 10 { i = i + 1; }";
        let tokens = tokenize(source);
        
        assert_eq!(tokens[0], Token::While);
        assert_eq!(tokens[1], Token::Identifier("i".to_string()));
        assert_eq!(tokens[2], Token::LessThan); // Now properly tokenized
        assert_eq!(tokens[3], Token::IntegerLiteral(10));
        assert_eq!(tokens[4], Token::LeftBrace);
        assert_eq!(tokens[5], Token::Identifier("i".to_string()));
        assert_eq!(tokens[6], Token::Assign);
        assert_eq!(tokens[7], Token::Identifier("i".to_string()));
        assert_eq!(tokens[8], Token::Plus);
        assert_eq!(tokens[9], Token::IntegerLiteral(1));
        assert_eq!(tokens[10], Token::Semicolon);
        assert_eq!(tokens[11], Token::RightBrace);
        assert_eq!(tokens[12], Token::Eof);
    }

    #[test]
    fn test_for_loop() {
        let source = "for i in 0..10 { }";
        let tokens = tokenize(source);
        
        assert_eq!(tokens[0], Token::For);
        assert_eq!(tokens[1], Token::Identifier("i".to_string()));
        assert_eq!(tokens[2], Token::In);
        assert_eq!(tokens[3], Token::IntegerLiteral(0));
        assert_eq!(tokens[4], Token::Dot);
        assert_eq!(tokens[5], Token::Dot);
        assert_eq!(tokens[6], Token::IntegerLiteral(10));
        assert_eq!(tokens[7], Token::LeftBrace);
        assert_eq!(tokens[8], Token::RightBrace);
        assert_eq!(tokens[9], Token::Eof);
    }

    #[test]
    fn test_infinite_loop() {
        let source = "loop { break; }";
        let tokens = tokenize(source);
        
        assert_eq!(tokens[0], Token::Loop);
        assert_eq!(tokens[1], Token::LeftBrace);
        assert_eq!(tokens[2], Token::Break);
        assert_eq!(tokens[3], Token::Semicolon);
        assert_eq!(tokens[4], Token::RightBrace);
        assert_eq!(tokens[5], Token::Eof);
    }

    #[test]
    fn test_keyword_vs_identifier() {
        // Test that keywords are properly distinguished from identifiers
        let source = "if_var else_func while_loop for_each in_range loop_count break_point continue_flag";
        let tokens = tokenize(source);
        
        assert_eq!(tokens[0], Token::Identifier("if_var".to_string()));
        assert_eq!(tokens[1], Token::Identifier("else_func".to_string()));
        assert_eq!(tokens[2], Token::Identifier("while_loop".to_string()));
        assert_eq!(tokens[3], Token::Identifier("for_each".to_string()));
        assert_eq!(tokens[4], Token::Identifier("in_range".to_string()));
        assert_eq!(tokens[5], Token::Identifier("loop_count".to_string()));
        assert_eq!(tokens[6], Token::Identifier("break_point".to_string()));
        assert_eq!(tokens[7], Token::Identifier("continue_flag".to_string()));
        assert_eq!(tokens[8], Token::Eof);
    }

    #[test]
    fn test_io_macros() {
        let source = r#"print!("Hello") println!("World")"#;
        let tokens = tokenize(source);
        
        assert_eq!(tokens[0], Token::PrintMacro);
        assert_eq!(tokens[1], Token::LeftParen);
        assert_eq!(tokens[2], Token::Identifier("\"Hello\"".to_string()));
        assert_eq!(tokens[3], Token::RightParen);
        assert_eq!(tokens[4], Token::PrintlnMacro);
        assert_eq!(tokens[5], Token::LeftParen);
        assert_eq!(tokens[6], Token::Identifier("\"World\"".to_string()));
        assert_eq!(tokens[7], Token::RightParen);
        assert_eq!(tokens[8], Token::Eof);
    }

    #[test]
    fn test_io_macros_vs_identifiers() {
        let source = "print println print! println!";
        let tokens = tokenize(source);
        
        assert_eq!(tokens[0], Token::Identifier("print".to_string()));
        assert_eq!(tokens[1], Token::Identifier("println".to_string()));
        assert_eq!(tokens[2], Token::PrintMacro);
        assert_eq!(tokens[3], Token::PrintlnMacro);
        assert_eq!(tokens[4], Token::Eof);
    }

    #[test]
    fn test_comparison_operators() {
        let source = "== != < > <= >=";
        let tokens = tokenize(source);
        
        assert_eq!(tokens[0], Token::Equal);
        assert_eq!(tokens[1], Token::NotEqual);
        assert_eq!(tokens[2], Token::LessThan);
        assert_eq!(tokens[3], Token::GreaterThan);
        assert_eq!(tokens[4], Token::LessEqual);
        assert_eq!(tokens[5], Token::GreaterEqual);
        assert_eq!(tokens[6], Token::Eof);
    }

    #[test]
    fn test_logical_operators() {
        let source = "&& || !";
        let tokens = tokenize(source);
        
        assert_eq!(tokens[0], Token::LogicalAnd);
        assert_eq!(tokens[1], Token::LogicalOr);
        assert_eq!(tokens[2], Token::LogicalNot);
        assert_eq!(tokens[3], Token::Eof);
    }

    #[test]
    fn test_assignment_vs_equality() {
        let source = "= == = ==";
        let tokens = tokenize(source);
        
        assert_eq!(tokens[0], Token::Assign);
        assert_eq!(tokens[1], Token::Equal);
        assert_eq!(tokens[2], Token::Assign);
        assert_eq!(tokens[3], Token::Equal);
        assert_eq!(tokens[4], Token::Eof);
    }

    #[test]
    fn test_not_vs_not_equal() {
        let source = "! != ! !=";
        let tokens = tokenize(source);
        
        assert_eq!(tokens[0], Token::LogicalNot);
        assert_eq!(tokens[1], Token::NotEqual);
        assert_eq!(tokens[2], Token::LogicalNot);
        assert_eq!(tokens[3], Token::NotEqual);
        assert_eq!(tokens[4], Token::Eof);
    }

    #[test]
    fn test_complex_expression_with_new_operators() {
        let source = "if x >= 5 && y != 10 || !flag { println!('Result: {}', x + y); }";
        let tokens = tokenize(source);
        
        assert_eq!(tokens[0], Token::If);
        assert_eq!(tokens[1], Token::Identifier("x".to_string()));
        assert_eq!(tokens[2], Token::GreaterEqual);
        assert_eq!(tokens[3], Token::IntegerLiteral(5));
        assert_eq!(tokens[4], Token::LogicalAnd);
        assert_eq!(tokens[5], Token::Identifier("y".to_string()));
        assert_eq!(tokens[6], Token::NotEqual);
        assert_eq!(tokens[7], Token::IntegerLiteral(10));
        assert_eq!(tokens[8], Token::LogicalOr);
        assert_eq!(tokens[9], Token::LogicalNot);
        assert_eq!(tokens[10], Token::Identifier("flag".to_string()));
        assert_eq!(tokens[11], Token::LeftBrace);
        assert_eq!(tokens[12], Token::PrintlnMacro);
        // ... rest of tokens
    }

    #[test]
    fn test_comments() {
        let source = "// This is a comment\nlet x = 5; // Another comment\n// Final comment";
        let tokens = tokenize(source);
        
        assert_eq!(tokens[0], Token::Let);
        assert_eq!(tokens[1], Token::Identifier("x".to_string()));
        assert_eq!(tokens[2], Token::Assign);
        assert_eq!(tokens[3], Token::IntegerLiteral(5));
        assert_eq!(tokens[4], Token::Semicolon);
        assert_eq!(tokens[5], Token::Eof);
    }

    #[test]
    fn test_divide_vs_comment() {
        let source = "x / y // comment\nz / w";
        let tokens = tokenize(source);
        
        assert_eq!(tokens[0], Token::Identifier("x".to_string()));
        assert_eq!(tokens[1], Token::Divide);
        assert_eq!(tokens[2], Token::Identifier("y".to_string()));
        assert_eq!(tokens[3], Token::Identifier("z".to_string()));
        assert_eq!(tokens[4], Token::Divide);
        assert_eq!(tokens[5], Token::Identifier("w".to_string()));
        assert_eq!(tokens[6], Token::Eof);
    }

    #[test]
    fn test_all_new_io_and_operators() {
        // Test all the new tokens required by task 2.3
        let source = "print! println! == != <= >= && || !";
        let tokens = tokenize(source);
        
        assert_eq!(tokens[0], Token::PrintMacro);
        assert_eq!(tokens[1], Token::PrintlnMacro);
        assert_eq!(tokens[2], Token::Equal);
        assert_eq!(tokens[3], Token::NotEqual);
        assert_eq!(tokens[4], Token::LessEqual);
        assert_eq!(tokens[5], Token::GreaterEqual);
        assert_eq!(tokens[6], Token::LogicalAnd);
        assert_eq!(tokens[7], Token::LogicalOr);
        assert_eq!(tokens[8], Token::LogicalNot);
        assert_eq!(tokens[9], Token::Eof);
    }

    #[test]
    fn test_complex_io_expression() {
        let source = r#"if x >= 5 && y != 10 || !flag { println!("Result: {}", x + y); }"#;
        let tokens = tokenize(source);
        
        assert_eq!(tokens[0], Token::If);
        assert_eq!(tokens[1], Token::Identifier("x".to_string()));
        assert_eq!(tokens[2], Token::GreaterEqual);
        assert_eq!(tokens[3], Token::IntegerLiteral(5));
        assert_eq!(tokens[4], Token::LogicalAnd);
        assert_eq!(tokens[5], Token::Identifier("y".to_string()));
        assert_eq!(tokens[6], Token::NotEqual);
        assert_eq!(tokens[7], Token::IntegerLiteral(10));
        assert_eq!(tokens[8], Token::LogicalOr);
        assert_eq!(tokens[9], Token::LogicalNot);
        assert_eq!(tokens[10], Token::Identifier("flag".to_string()));
        assert_eq!(tokens[11], Token::LeftBrace);
        assert_eq!(tokens[12], Token::PrintlnMacro);
        assert_eq!(tokens[13], Token::LeftParen);
        assert_eq!(tokens[14], Token::Identifier("\"Result: {}\"".to_string()));
        // Continue with more tokens...
        assert!(tokens.len() > 15);
        assert_eq!(tokens.last(), Some(&Token::Eof));
    }


}