// Standalone test for lexer with location tracking
use std::fmt;

/// Represents a location in source code
#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub filename: Option<String>,
}

impl SourceLocation {
    pub fn new(line: usize, column: usize) -> Self {
        SourceLocation {
            line,
            column,
            filename: None,
        }
    }

    pub fn with_filename(line: usize, column: usize, filename: String) -> Self {
        SourceLocation {
            line,
            column,
            filename: Some(filename),
        }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.filename {
            Some(filename) => write!(f, "{}:{}:{}", filename, self.line, self.column),
            None => write!(f, "{}:{}", self.line, self.column),
        }
    }
}

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

pub fn tokenize_with_locations(source: &str, filename: Option<String>) -> Vec<LocatedToken> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    let mut line = 1;
    let mut column = 1;

    // Helper function to create location
    let make_location = |line: usize, column: usize| {
        match &filename {
            Some(f) => SourceLocation::with_filename(line, column, f.clone()),
            None => SourceLocation::new(line, column),
        }
    };

    // Helper function to advance position tracking
    let mut advance_position = |c: char, line: &mut usize, column: &mut usize| {
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
            // Simple operators
            '+' => { 
                chars.next();
                advance_position(c, &mut line, &mut column);
                tokens.push(LocatedToken::new(Token::Plus, make_location(token_start_line, token_start_column)));
            }
            ';' => { 
                chars.next();
                advance_position(c, &mut line, &mut column);
                tokens.push(LocatedToken::new(Token::Semicolon, make_location(token_start_line, token_start_column)));
            }
            '=' => {
                let ch = chars.next().unwrap(); // consume '='
                advance_position(ch, &mut line, &mut column);
                if let Some(&'=') = chars.peek() {
                    let ch2 = chars.next().unwrap(); // consume second '='
                    advance_position(ch2, &mut line, &mut column);
                    tokens.push(LocatedToken::new(Token::Equal, make_location(token_start_line, token_start_column)));
                } else {
                    tokens.push(LocatedToken::new(Token::Assign, make_location(token_start_line, token_start_column)));
                }
            }
            // Integer literals
            '0'..='9' => {
                let mut num_str = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() {
                        let ch = chars.next().unwrap();
                        advance_position(ch, &mut line, &mut column);
                        num_str.push(ch);
                    } else {
                        break;
                    }
                }
                let int_val: i64 = num_str.parse().unwrap_or(0);
                tokens.push(LocatedToken::new(Token::IntegerLiteral(int_val), make_location(token_start_line, token_start_column)));
            }
            // Identifiers and Keywords
            'a'..='z' | 'A'..='Z' | '_' => {
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
                    tokens.push(LocatedToken::new(token, make_location(token_start_line, token_start_column)));
                }
            }
            _ => {
                // Handle unexpected characters or errors
                println!("Unexpected character: {} at {}:{}", c, line, column);
                let ch = chars.next().unwrap();
                advance_position(ch, &mut line, &mut column);
            }
        }
    }
    
    tokens.push(LocatedToken::new(Token::Eof, make_location(line, column)));
    tokens
}

fn main() {
    println!("Testing lexer with location tracking...");

    // Test simple tokenization with location tracking
    let source = "let x = 5;\nlet y = 10;";
    let tokens = tokenize_with_locations(source, Some("test.aero".to_string()));
    
    println!("Source code:");
    println!("{}", source);
    println!("\nTokens with locations:");
    
    for token in &tokens {
        println!("{:?} at {}", token.token, token.location);
    }
    
    // Verify specific locations
    assert_eq!(tokens[0].token, Token::Let);
    assert_eq!(tokens[0].location.line, 1);
    assert_eq!(tokens[0].location.column, 1);
    
    assert_eq!(tokens[1].token, Token::Identifier("x".to_string()));
    assert_eq!(tokens[1].location.line, 1);
    assert_eq!(tokens[1].location.column, 5);
    
    // Find second let on line 2
    let second_let = tokens.iter().find(|t| matches!(t.token, Token::Let) && t.location.line == 2).unwrap();
    assert_eq!(second_let.location.line, 2);
    assert_eq!(second_let.location.column, 1);
    
    println!("\nLocation tracking test passed!");
    
    // Test multiline with complex expressions
    let complex_source = "fn main() {\n    if x == 5 {\n        println!(\"Hello\");\n    }\n}";
    let complex_tokens = tokenize_with_locations(complex_source, Some("main.aero".to_string()));
    
    println!("\nComplex source:");
    println!("{}", complex_source);
    println!("\nComplex tokens:");
    
    for token in &complex_tokens {
        if !matches!(token.token, Token::Eof) {
            println!("{:?} at {}", token.token, token.location);
        }
    }
    
    // Find specific tokens and verify their locations
    let fn_token = complex_tokens.iter().find(|t| matches!(t.token, Token::Fn)).unwrap();
    assert_eq!(fn_token.location.line, 1);
    assert_eq!(fn_token.location.column, 1);
    
    let if_token = complex_tokens.iter().find(|t| matches!(t.token, Token::If)).unwrap();
    assert_eq!(if_token.location.line, 2);
    assert_eq!(if_token.location.column, 5);
    
    let println_token = complex_tokens.iter().find(|t| matches!(t.token, Token::PrintlnMacro)).unwrap();
    assert_eq!(println_token.location.line, 3);
    assert_eq!(println_token.location.column, 9);
    
    println!("\nAll lexer location tracking tests passed!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_location_tracking() {
        let source = "let x = 5;";
        let tokens = tokenize_with_locations(source, None);
        
        assert_eq!(tokens[0].token, Token::Let);
        assert_eq!(tokens[0].location.line, 1);
        assert_eq!(tokens[0].location.column, 1);
        
        assert_eq!(tokens[1].token, Token::Identifier("x".to_string()));
        assert_eq!(tokens[1].location.line, 1);
        assert_eq!(tokens[1].location.column, 5);
        
        assert_eq!(tokens[2].token, Token::Assign);
        assert_eq!(tokens[2].location.line, 1);
        assert_eq!(tokens[2].location.column, 7);
    }

    #[test]
    fn test_multiline_location_tracking() {
        let source = "let x = 5;\nlet y = 10;";
        let tokens = tokenize_with_locations(source, None);
        
        // Find the second let token
        let second_let = tokens.iter().find(|t| matches!(t.token, Token::Let) && t.location.line == 2).unwrap();
        assert_eq!(second_let.location.line, 2);
        assert_eq!(second_let.location.column, 1);
    }

    #[test]
    fn test_filename_tracking() {
        let source = "let x = 5;";
        let tokens = tokenize_with_locations(source, Some("test.aero".to_string()));
        
        assert_eq!(tokens[0].location.filename, Some("test.aero".to_string()));
        assert_eq!(format!("{}", tokens[0].location), "test.aero:1:1");
    }

    #[test]
    fn test_complex_expression_locations() {
        let source = "if x == 5 {\n    println!(\"test\");\n}";
        let tokens = tokenize_with_locations(source, None);
        
        let if_token = tokens.iter().find(|t| matches!(t.token, Token::If)).unwrap();
        assert_eq!(if_token.location.line, 1);
        assert_eq!(if_token.location.column, 1);
        
        let equal_token = tokens.iter().find(|t| matches!(t.token, Token::Equal)).unwrap();
        assert_eq!(equal_token.location.line, 1);
        assert_eq!(equal_token.location.column, 5);
        
        let println_token = tokens.iter().find(|t| matches!(t.token, Token::PrintlnMacro)).unwrap();
        assert_eq!(println_token.location.line, 2);
        assert_eq!(println_token.location.column, 5);
    }
}