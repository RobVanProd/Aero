use crate::errors::SourceLocation;

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
    Match,

    // Data structure keywords
    Struct,
    Enum,
    Impl,
    Self_,

    // Phase 5 keywords
    Trait,
    Where,

    // String literal
    StringLiteral(String),

    // I/O Macros
    PrintMacro,   // print!
    PrintlnMacro, // println!

    // Operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Assign,
    Arrow, // ->

    // Comparison operators
    Equal,        // ==
    NotEqual,     // !=
    LessThan,     // <
    GreaterThan,  // >
    LessEqual,    // <=
    GreaterEqual, // >=

    // Logical operators
    LogicalAnd, // &&
    LogicalOr,  // ||
    LogicalNot, // !

    // Delimiters
    Semicolon,
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Dot,
    Colon,
    DoubleColon, // ::
    Comma,
    FatArrow,   // =>
    Underscore, // _ (wildcard pattern)
    Ampersand,  // & (borrow / reference)
    Pipe,       // | (single pipe, for closures/patterns)

    // End of file
    Eof,
}

/// Token with location information
#[derive(Debug, Clone, PartialEq)]
pub struct LocatedToken {
    pub token: Token,
    pub location: SourceLocation,
}

impl LocatedToken {
    pub fn new(token: Token, location: SourceLocation) -> Self {
        LocatedToken { token, location }
    }
}

pub fn tokenize(source: &str) -> Vec<Token> {
    let located_tokens = tokenize_with_locations(source, None);
    located_tokens.into_iter().map(|lt| lt.token).collect()
}

pub fn tokenize_with_locations(source: &str, filename: Option<String>) -> Vec<LocatedToken> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    let mut line = 1;
    let mut column = 1;

    // Helper function to create location
    let make_location = |line: usize, column: usize| match &filename {
        Some(f) => SourceLocation::with_filename(line, column, f.clone()),
        None => SourceLocation::new(line, column),
    };

    // Helper function to advance position tracking
    let advance_position = |c: char, line: &mut usize, column: &mut usize| {
        if c == '\n' {
            *line += 1;
            *column = 1;
        } else {
            *column += 1;
        }
    };

    while let Some(&c) = chars.peek() {
        let token_start_line = line;
        let token_start_column = column;

        match c {
            // Whitespace
            ' ' | '\t' | '\n' | '\r' => {
                let ch = chars.next().unwrap();
                advance_position(ch, &mut line, &mut column);
            }
            // Operators and delimiters
            '+' => {
                chars.next();
                advance_position(c, &mut line, &mut column);
                tokens.push(LocatedToken::new(
                    Token::Plus,
                    make_location(token_start_line, token_start_column),
                ));
            }
            '*' => {
                chars.next();
                advance_position(c, &mut line, &mut column);
                tokens.push(LocatedToken::new(
                    Token::Multiply,
                    make_location(token_start_line, token_start_column),
                ));
            }
            '/' => {
                let ch = chars.next().unwrap(); // consume first '/'
                advance_position(ch, &mut line, &mut column);
                if let Some(&'/') = chars.peek() {
                    // Line comment - consume until end of line
                    let ch2 = chars.next().unwrap(); // consume second '/'
                    advance_position(ch2, &mut line, &mut column);
                    while let Some(&c) = chars.peek() {
                        if c == '\n' || c == '\r' {
                            break;
                        }
                        let ch = chars.next().unwrap();
                        advance_position(ch, &mut line, &mut column);
                    }
                } else {
                    tokens.push(LocatedToken::new(
                        Token::Divide,
                        make_location(token_start_line, token_start_column),
                    ));
                }
            }
            '%' => {
                chars.next();
                advance_position(c, &mut line, &mut column);
                tokens.push(LocatedToken::new(
                    Token::Modulo,
                    make_location(token_start_line, token_start_column),
                ));
            }
            ';' => {
                chars.next();
                advance_position(c, &mut line, &mut column);
                tokens.push(LocatedToken::new(
                    Token::Semicolon,
                    make_location(token_start_line, token_start_column),
                ));
            }
            '{' => {
                chars.next();
                advance_position(c, &mut line, &mut column);
                tokens.push(LocatedToken::new(
                    Token::LeftBrace,
                    make_location(token_start_line, token_start_column),
                ));
            }
            '}' => {
                chars.next();
                advance_position(c, &mut line, &mut column);
                tokens.push(LocatedToken::new(
                    Token::RightBrace,
                    make_location(token_start_line, token_start_column),
                ));
            }
            '(' => {
                chars.next();
                advance_position(c, &mut line, &mut column);
                tokens.push(LocatedToken::new(
                    Token::LeftParen,
                    make_location(token_start_line, token_start_column),
                ));
            }
            ')' => {
                chars.next();
                advance_position(c, &mut line, &mut column);
                tokens.push(LocatedToken::new(
                    Token::RightParen,
                    make_location(token_start_line, token_start_column),
                ));
            }
            '[' => {
                chars.next();
                advance_position(c, &mut line, &mut column);
                tokens.push(LocatedToken::new(
                    Token::LeftBracket,
                    make_location(token_start_line, token_start_column),
                ));
            }
            ']' => {
                chars.next();
                advance_position(c, &mut line, &mut column);
                tokens.push(LocatedToken::new(
                    Token::RightBracket,
                    make_location(token_start_line, token_start_column),
                ));
            }
            ':' => {
                let ch = chars.next().unwrap();
                advance_position(ch, &mut line, &mut column);
                if let Some(&':') = chars.peek() {
                    let ch2 = chars.next().unwrap();
                    advance_position(ch2, &mut line, &mut column);
                    tokens.push(LocatedToken::new(
                        Token::DoubleColon,
                        make_location(token_start_line, token_start_column),
                    ));
                } else {
                    tokens.push(LocatedToken::new(
                        Token::Colon,
                        make_location(token_start_line, token_start_column),
                    ));
                }
            }
            ',' => {
                chars.next();
                advance_position(c, &mut line, &mut column);
                tokens.push(LocatedToken::new(
                    Token::Comma,
                    make_location(token_start_line, token_start_column),
                ));
            }
            // At operator for pattern matching
            '@' => { 
                chars.next();
                advance_position(c, &mut line, &mut column);
                tokens.push(LocatedToken::new(Token::At, make_location(token_start_line, token_start_column)));
            }
            // Question mark operator for error propagation
            '?' => { 
                chars.next();
                advance_position(c, &mut line, &mut column);
                tokens.push(LocatedToken::new(Token::Question, make_location(token_start_line, token_start_column)));
            }
            // Handle minus and arrow (->)
            '-' => {
                let ch = chars.next().unwrap(); // consume '-'
                advance_position(ch, &mut line, &mut column);
                if let Some(&'>') = chars.peek() {
                    let ch2 = chars.next().unwrap(); // consume '>'
                    advance_position(ch2, &mut line, &mut column);
                    tokens.push(LocatedToken::new(
                        Token::Arrow,
                        make_location(token_start_line, token_start_column),
                    ));
                } else {
                    tokens.push(LocatedToken::new(
                        Token::Minus,
                        make_location(token_start_line, token_start_column),
                    ));
                }
            }
            // Handle assignment, equality, and fat arrow
            '=' => {
                let ch = chars.next().unwrap(); // consume '='
                advance_position(ch, &mut line, &mut column);
                if let Some(&'=') = chars.peek() {
                    let ch2 = chars.next().unwrap(); // consume second '='
                    advance_position(ch2, &mut line, &mut column);
                    tokens.push(LocatedToken::new(
                        Token::Equal,
                        make_location(token_start_line, token_start_column),
                    ));
                } else if let Some(&'>') = chars.peek() {
                    let ch2 = chars.next().unwrap(); // consume '>'
                    advance_position(ch2, &mut line, &mut column);
                    tokens.push(LocatedToken::new(
                        Token::FatArrow,
                        make_location(token_start_line, token_start_column),
                    ));
                } else {
                    tokens.push(LocatedToken::new(
                        Token::Assign,
                        make_location(token_start_line, token_start_column),
                    ));
                }
            }
            // Handle not equal and logical not
            '!' => {
                let ch = chars.next().unwrap(); // consume '!'
                advance_position(ch, &mut line, &mut column);
                if let Some(&'=') = chars.peek() {
                    let ch2 = chars.next().unwrap(); // consume '='
                    advance_position(ch2, &mut line, &mut column);
                    tokens.push(LocatedToken::new(
                        Token::NotEqual,
                        make_location(token_start_line, token_start_column),
                    ));
                } else {
                    tokens.push(LocatedToken::new(
                        Token::LogicalNot,
                        make_location(token_start_line, token_start_column),
                    ));
                }
            }
            // Handle less than and less equal
            '<' => {
                let ch = chars.next().unwrap(); // consume '<'
                advance_position(ch, &mut line, &mut column);
                if let Some(&'=') = chars.peek() {
                    let ch2 = chars.next().unwrap(); // consume '='
                    advance_position(ch2, &mut line, &mut column);
                    tokens.push(LocatedToken::new(
                        Token::LessEqual,
                        make_location(token_start_line, token_start_column),
                    ));
                } else {
                    tokens.push(LocatedToken::new(
                        Token::LessThan,
                        make_location(token_start_line, token_start_column),
                    ));
                }
            }
            // Handle greater than and greater equal
            '>' => {
                let ch = chars.next().unwrap(); // consume '>'
                advance_position(ch, &mut line, &mut column);
                if let Some(&'=') = chars.peek() {
                    let ch2 = chars.next().unwrap(); // consume '='
                    advance_position(ch2, &mut line, &mut column);
                    tokens.push(LocatedToken::new(
                        Token::GreaterEqual,
                        make_location(token_start_line, token_start_column),
                    ));
                } else {
                    tokens.push(LocatedToken::new(
                        Token::GreaterThan,
                        make_location(token_start_line, token_start_column),
                    ));
                }
            }
            // Handle & (reference/borrow) and && (logical and)
            '&' => {
                let ch = chars.next().unwrap(); // consume '&'
                advance_position(ch, &mut line, &mut column);
                if let Some(&'&') = chars.peek() {
                    let ch2 = chars.next().unwrap(); // consume second '&'
                    advance_position(ch2, &mut line, &mut column);
                    tokens.push(LocatedToken::new(
                        Token::LogicalAnd,
                        make_location(token_start_line, token_start_column),
                    ));
                } else {
                    tokens.push(LocatedToken::new(
                        Token::Ampersand,
                        make_location(token_start_line, token_start_column),
                    ));
                }
            }
            // Handle | (single pipe) and || (logical or)
            '|' => {
                let ch = chars.next().unwrap(); // consume '|'
                advance_position(ch, &mut line, &mut column);
                if let Some(&'|') = chars.peek() {
                    let ch2 = chars.next().unwrap(); // consume second '|'
                    advance_position(ch2, &mut line, &mut column);
                    tokens.push(LocatedToken::new(
                        Token::LogicalOr,
                        make_location(token_start_line, token_start_column),
                    ));
                } else {
                    tokens.push(LocatedToken::new(
                        Token::Pipe,
                        make_location(token_start_line, token_start_column),
                    ));
                }
            }
            // Dot operator and range operators
            '.' => {
                let ch = chars.next().unwrap(); // consume the '.'
                advance_position(ch, &mut line, &mut column);
                tokens.push(LocatedToken::new(
                    Token::Dot,
                    make_location(token_start_line, token_start_column),
                ));
            }
            // Integer and Float Literals
            '0'..='9' => {
                let mut num_str = String::new();
                let mut has_dot = false;
                let mut has_exponent = false;

                // Collect digits and decimal point
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() {
                        let ch = chars.next().unwrap();
                        advance_position(ch, &mut line, &mut column);
                        num_str.push(ch);
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
                                let ch = chars.next().unwrap();
                                advance_position(ch, &mut line, &mut column);
                                num_str.push(ch);
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
                        let ch = chars.next().unwrap();
                        advance_position(ch, &mut line, &mut column);
                        num_str.push(ch);
                        // Handle optional sign after exponent
                        if let Some(&sign) = chars.peek()
                            && (sign == '+' || sign == '-')
                        {
                            let ch = chars.next().unwrap();
                            advance_position(ch, &mut line, &mut column);
                            num_str.push(ch);
                        }
                    } else {
                        break;
                    }
                }

                if has_dot || has_exponent {
                    let float_val: f64 = num_str.parse().unwrap_or(0.0);
                    tokens.push(LocatedToken::new(
                        Token::FloatLiteral(float_val),
                        make_location(token_start_line, token_start_column),
                    ));
                } else {
                    let int_val: i64 = num_str.parse().unwrap_or(0);
                    tokens.push(LocatedToken::new(
                        Token::IntegerLiteral(int_val),
                        make_location(token_start_line, token_start_column),
                    ));
                }
            }
            // String literals
            '"' => {
                let ch = chars.next().unwrap(); // consume opening quote
                advance_position(ch, &mut line, &mut column);
                let mut string_content = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '"' {
                        let ch = chars.next().unwrap(); // consume closing quote
                        advance_position(ch, &mut line, &mut column);
                        break;
                    } else if c == '\\' {
                        // Handle escape sequences
                        let _ch = chars.next().unwrap(); // consume backslash
                        advance_position(_ch, &mut line, &mut column);
                        if let Some(&escaped) = chars.peek() {
                            let ch = chars.next().unwrap(); // consume escaped char
                            advance_position(ch, &mut line, &mut column);
                            match escaped {
                                'n' => string_content.push('\n'),
                                't' => string_content.push('\t'),
                                'r' => string_content.push('\r'),
                                '\\' => string_content.push('\\'),
                                '"' => string_content.push('"'),
                                '0' => string_content.push('\0'),
                                _ => {
                                    string_content.push('\\');
                                    string_content.push(escaped);
                                }
                            }
                        }
                    } else {
                        let ch = chars.next().unwrap();
                        advance_position(ch, &mut line, &mut column);
                        string_content.push(ch);
                    }
                }
                tokens.push(LocatedToken::new(
                    Token::StringLiteral(string_content),
                    make_location(token_start_line, token_start_column),
                ));
            }
            // Handle underscore (could be wildcard pattern or identifier)
            '_' => {
                let ch = chars.next().unwrap(); // consume '_'
                advance_position(ch, &mut line, &mut column);
                
                // Check if this is a standalone underscore (wildcard pattern)
                if let Some(&next_char) = chars.peek() {
                    if next_char.is_ascii_alphanumeric() || next_char == '_' {
                        // This is part of an identifier, put the '_' back and handle as identifier
                        let mut ident_str = String::from("_");
                        while let Some(&d) = chars.peek() {
                            if d.is_ascii_alphanumeric() || d == '_' {
                                let ch = chars.next().unwrap();
                                advance_position(ch, &mut line, &mut column);
                                ident_str.push(ch);
                            } else {
                                break;
                            }
                        }
                        
                        // Check for I/O macros (identifiers followed by !)
                        if let Some(&'!') = chars.peek() {
                            let token = match ident_str.as_str() {
                                "print" => {
                                    let ch = chars.next().unwrap(); // consume '!'
                                    advance_position(ch, &mut line, &mut column);
                                    Token::PrintMacro
                                }
                                "println" => {
                                    let ch = chars.next().unwrap(); // consume '!'
                                    advance_position(ch, &mut line, &mut column);
                                    Token::PrintlnMacro
                                }
                                _ => Token::Identifier(ident_str), // Regular identifier, don't consume '!'
                            };
                            tokens.push(LocatedToken::new(token, make_location(token_start_line, token_start_column)));
                        } else {
                            // Regular keywords and identifiers
                            let token = match ident_str.as_str() {
                                "let" => Token::Let,
                                "fn" => Token::Fn,
                                "return" => Token::Return,
                                "mut" => Token::Mut,
                                "struct" => Token::Struct,
                                "enum" => Token::Enum,
                                "impl" => Token::Impl,
                                "if" => Token::If,
                                "else" => Token::Else,
                                "while" => Token::While,
                                "for" => Token::For,
                                "in" => Token::In,
                                "loop" => Token::Loop,
                                "break" => Token::Break,
                                "continue" => Token::Continue,
                                "match" => Token::Match,
                                _ => Token::Identifier(ident_str),
                            };
                            tokens.push(LocatedToken::new(token, make_location(token_start_line, token_start_column)));
                        }
                    } else {
                        // Standalone underscore - wildcard pattern
                        tokens.push(LocatedToken::new(Token::Underscore, make_location(token_start_line, token_start_column)));
                    }
                } else {
                    // End of input, standalone underscore
                    tokens.push(LocatedToken::new(Token::Underscore, make_location(token_start_line, token_start_column)));
                }
            }
            // Identifiers and Keywords
            'a'..='z' | 'A'..='Z' => {
                let mut ident_str = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_alphanumeric() || d == '_' {
                        let ch = chars.next().unwrap();
                        advance_position(ch, &mut line, &mut column);
                        ident_str.push(ch);
                    } else {
                        break;
                    }
                }

                // Check for I/O macros (identifiers followed by !)
                if let Some(&'!') = chars.peek() {
                    let token = match ident_str.as_str() {
                        "print" => {
                            let ch = chars.next().unwrap(); // consume '!'
                            advance_position(ch, &mut line, &mut column);
                            Token::PrintMacro
                        }
                        "println" => {
                            let ch = chars.next().unwrap(); // consume '!'
                            advance_position(ch, &mut line, &mut column);
                            Token::PrintlnMacro
                        }
                        "format" => {
                            let ch = chars.next().unwrap(); // consume '!'
                            advance_position(ch, &mut line, &mut column);
                            Token::Format
                        }
                        _ => Token::Identifier(ident_str), // Regular identifier, don't consume '!'
                    };
                    tokens.push(LocatedToken::new(
                        token,
                        make_location(token_start_line, token_start_column),
                    ));
                } else {
                    // Regular keywords and identifiers
                    let token = match ident_str.as_str() {
                        "let" => Token::Let,
                        "fn" => Token::Fn,
                        "return" => Token::Return,
                        "mut" => Token::Mut,
                        "struct" => Token::Struct,
                        "enum" => Token::Enum,
                        "impl" => Token::Impl,
                        "if" => Token::If,
                        "else" => Token::Else,
                        "while" => Token::While,
                        "for" => Token::For,
                        "in" => Token::In,
                        "loop" => Token::Loop,
                        "break" => Token::Break,
                        "continue" => Token::Continue,
                        "match" => Token::Match,
                        "struct" => Token::Struct,
                        "enum" => Token::Enum,
                        "impl" => Token::Impl,
                        "self" => Token::Self_,
                        "trait" => Token::Trait,
                        "where" => Token::Where,
                        "_" => Token::Underscore,
                        _ => Token::Identifier(ident_str),
                    };
                    tokens.push(LocatedToken::new(
                        token,
                        make_location(token_start_line, token_start_column),
                    ));
                }
            }
            _ => {
                // Handle unexpected characters or errors
                eprintln!("Unexpected character: {} at {}:{}", c, line, column);
                let ch = chars.next().unwrap();
                advance_position(ch, &mut line, &mut column);
            }
        }
    }

    tokens.push(LocatedToken::new(Token::Eof, make_location(line, column)));
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_tracking() {
        let source = "let x = 5;\nlet y = 10;";
        let tokens = tokenize_with_locations(source, None);

        assert_eq!(tokens[0].token, Token::Let);
        assert_eq!(tokens[0].location.line, 1);
        assert_eq!(tokens[0].location.column, 1);

        assert_eq!(tokens[1].token, Token::Identifier("x".to_string()));
        assert_eq!(tokens[1].location.line, 1);
        assert_eq!(tokens[1].location.column, 5);

        assert_eq!(tokens[5].token, Token::Let); // Second let on line 2
        assert_eq!(tokens[5].location.line, 2);
        assert_eq!(tokens[5].location.column, 1);
    }

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
    fn test_io_macros() {
        let source = r#"print!("Hello") println!("World")"#;
        let tokens = tokenize(source);

        assert_eq!(tokens[0], Token::PrintMacro);
        assert_eq!(tokens[1], Token::LeftParen);
        assert_eq!(tokens[2], Token::StringLiteral("Hello".to_string()));
        assert_eq!(tokens[3], Token::RightParen);
        assert_eq!(tokens[4], Token::PrintlnMacro);
        assert_eq!(tokens[5], Token::LeftParen);
        assert_eq!(tokens[6], Token::StringLiteral("World".to_string()));
        assert_eq!(tokens[7], Token::RightParen);
        assert_eq!(tokens[8], Token::Eof);
    }

    #[test]
    fn test_brackets_and_new_keywords() {
        let source = "let arr: [i32; 3] = [1, 2, 3]; struct enum match impl";
        let tokens = tokenize(source);

        assert_eq!(tokens[0], Token::Let);
        assert_eq!(tokens[3], Token::LeftBracket);
        assert_eq!(tokens[7], Token::RightBracket);
        assert!(tokens.iter().any(|t| *t == Token::Struct));
        assert!(tokens.iter().any(|t| *t == Token::Enum));
        assert!(tokens.iter().any(|t| *t == Token::Match));
        assert!(tokens.iter().any(|t| *t == Token::Impl));
    }
}
