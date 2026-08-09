use crate::errors::{CompilerError, SourceLocation};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    IntegerLiteral(i64),
    FloatLiteral(f64),
    Identifier(String),

    // Keywords
    Let,
    Const,
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

    // Phase 7 keywords (v1.0.0 module system + closures)
    Mod,
    Use,
    Import,
    Pub,
    As,

    // String literal
    CharacterLiteral(char),
    StringLiteral(String),
    FStringLiteral(String), // f"hello {name}"

    // I/O Macros
    PrintMacro,   // print!
    PrintlnMacro, // println!
    VecMacro,     // vec!

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
    scan_with_locations(source, filename, LexMode::Recovery)
        .expect("recovery lexer must always produce a token stream")
}

#[allow(clippy::result_large_err)]
pub fn try_tokenize_with_locations(
    source: &str,
    filename: Option<String>,
) -> Result<Vec<LocatedToken>, CompilerError> {
    scan_with_locations(source, filename, LexMode::Strict)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexMode {
    Recovery,
    Strict,
}

#[allow(clippy::result_large_err)]
fn scan_with_locations(
    source: &str,
    filename: Option<String>,
    mode: LexMode,
) -> Result<Vec<LocatedToken>, CompilerError> {
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
            // Dot operator
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
                    let float_val = match num_str.parse::<f64>() {
                        Ok(value) if mode == LexMode::Strict && !value.is_finite() => {
                            return Err(CompilerError::InvalidNumber {
                                text: num_str,
                                location: make_location(token_start_line, token_start_column),
                            });
                        }
                        Ok(value) => value,
                        Err(_) if mode == LexMode::Strict => {
                            return Err(CompilerError::InvalidNumber {
                                text: num_str,
                                location: make_location(token_start_line, token_start_column),
                            });
                        }
                        Err(_) => 0.0,
                    };
                    tokens.push(LocatedToken::new(
                        Token::FloatLiteral(float_val),
                        make_location(token_start_line, token_start_column),
                    ));
                } else {
                    let int_val = match num_str.parse::<i64>() {
                        Ok(value) => value,
                        Err(_) if mode == LexMode::Strict => {
                            return Err(CompilerError::InvalidNumber {
                                text: num_str,
                                location: make_location(token_start_line, token_start_column),
                            });
                        }
                        Err(_) => 0,
                    };
                    tokens.push(LocatedToken::new(
                        Token::IntegerLiteral(int_val),
                        make_location(token_start_line, token_start_column),
                    ));
                }
            }
            // Character literals. A character is exactly one Unicode scalar value or
            // one of the language's closed escape set; recovery never fabricates a
            // token for malformed input.
            '\'' => {
                let opening_location = make_location(token_start_line, token_start_column);
                let opening = chars.next().unwrap();
                advance_position(opening, &mut line, &mut column);

                let value = match chars.peek().copied() {
                    Some('\n' | '\r' | '\'') | None => None,
                    Some('\\') => {
                        let slash = chars.next().unwrap();
                        advance_position(slash, &mut line, &mut column);
                        match chars.next() {
                            Some('n') => {
                                advance_position('n', &mut line, &mut column);
                                Some('\n')
                            }
                            Some('r') => {
                                advance_position('r', &mut line, &mut column);
                                Some('\r')
                            }
                            Some('t') => {
                                advance_position('t', &mut line, &mut column);
                                Some('\t')
                            }
                            Some('\\') => {
                                advance_position('\\', &mut line, &mut column);
                                Some('\\')
                            }
                            Some('\'') => {
                                advance_position('\'', &mut line, &mut column);
                                Some('\'')
                            }
                            Some('"') => {
                                advance_position('"', &mut line, &mut column);
                                Some('"')
                            }
                            Some('0') => {
                                advance_position('0', &mut line, &mut column);
                                Some('\0')
                            }
                            Some('x') => {
                                advance_position('x', &mut line, &mut column);
                                let mut digits = String::with_capacity(2);
                                for _ in 0..2 {
                                    match chars.peek().copied() {
                                        Some(digit) if digit.is_ascii_hexdigit() => {
                                            let digit = chars.next().unwrap();
                                            advance_position(digit, &mut line, &mut column);
                                            digits.push(digit);
                                        }
                                        _ => break,
                                    }
                                }
                                if digits.len() == 2 {
                                    u8::from_str_radix(&digits, 16).ok().map(char::from)
                                } else {
                                    None
                                }
                            }
                            Some(other) => {
                                advance_position(other, &mut line, &mut column);
                                None
                            }
                            None => None,
                        }
                    }
                    Some(_) => {
                        let raw = chars.next().unwrap();
                        advance_position(raw, &mut line, &mut column);
                        Some(raw)
                    }
                };

                let terminated = if value.is_some() && matches!(chars.peek(), Some('\'')) {
                    let closing = chars.next().unwrap();
                    advance_position(closing, &mut line, &mut column);
                    true
                } else {
                    false
                };

                if let (Some(value), true) = (value, terminated) {
                    tokens.push(LocatedToken::new(
                        Token::CharacterLiteral(value),
                        opening_location,
                    ));
                } else if mode == LexMode::Strict {
                    return Err(CompilerError::InvalidCharacterLiteral {
                        location: opening_location,
                    });
                } else {
                    eprintln!("Invalid character literal at {}", opening_location);
                    while let Some(next) = chars.peek().copied() {
                        if matches!(next, '\n' | '\r') {
                            break;
                        }
                        let next = chars.next().unwrap();
                        advance_position(next, &mut line, &mut column);
                        if next == '\'' {
                            break;
                        }
                    }
                }
            }
            // String literals
            '"' => {
                let ch = chars.next().unwrap(); // consume opening quote
                advance_position(ch, &mut line, &mut column);
                let mut string_content = String::new();
                let mut terminated = false;
                while let Some(&c) = chars.peek() {
                    if c == '"' {
                        let ch = chars.next().unwrap(); // consume closing quote
                        advance_position(ch, &mut line, &mut column);
                        terminated = true;
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
                if mode == LexMode::Strict && !terminated {
                    return Err(CompilerError::UnterminatedString {
                        location: make_location(token_start_line, token_start_column),
                    });
                }
                tokens.push(LocatedToken::new(
                    Token::StringLiteral(string_content),
                    make_location(token_start_line, token_start_column),
                ));
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

                // f"..." interpolation literal.
                // Keep the token as a single literal so parser can desugar placeholders.
                if ident_str == "f"
                    && let Some(&'"') = chars.peek()
                {
                    let quote = chars.next().unwrap(); // consume opening quote
                    advance_position(quote, &mut line, &mut column);
                    let mut string_content = String::new();
                    let mut terminated = false;
                    while let Some(&c) = chars.peek() {
                        if c == '"' {
                            let ch = chars.next().unwrap(); // consume closing quote
                            advance_position(ch, &mut line, &mut column);
                            terminated = true;
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
                    if mode == LexMode::Strict && !terminated {
                        return Err(CompilerError::UnterminatedString {
                            location: make_location(token_start_line, token_start_column),
                        });
                    }
                    tokens.push(LocatedToken::new(
                        Token::FStringLiteral(string_content),
                        make_location(token_start_line, token_start_column),
                    ));
                    continue;
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
                        "vec" => {
                            let ch = chars.next().unwrap(); // consume '!'
                            advance_position(ch, &mut line, &mut column);
                            Token::VecMacro
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
                        "const" => Token::Const,
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
                        "match" => Token::Match,
                        "struct" => Token::Struct,
                        "enum" => Token::Enum,
                        "impl" => Token::Impl,
                        "self" => Token::Self_,
                        "trait" => Token::Trait,
                        "where" => Token::Where,
                        "mod" => Token::Mod,
                        "use" => Token::Use,
                        "import" => Token::Import,
                        "pub" => Token::Pub,
                        "as" => Token::As,
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
                if mode == LexMode::Strict {
                    return Err(CompilerError::UnexpectedCharacter {
                        character: c,
                        location: make_location(token_start_line, token_start_column),
                    });
                }
                eprintln!("Unexpected character: {} at {}:{}", c, line, column);
                let ch = chars.next().unwrap();
                advance_position(ch, &mut line, &mut column);
            }
        }
    }

    tokens.push(LocatedToken::new(Token::Eof, make_location(line, column)));
    Ok(tokens)
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
    fn strict_and_recovery_lexers_agree_on_valid_source() {
        let source = "let integer = 42; let float = 1.25e+2; let text = \"ok\"; let formatted = f\"{integer}\";";
        let filename = Some("valid.aero".to_string());

        let strict = try_tokenize_with_locations(source, filename.clone())
            .expect("valid source must pass strict lexing");
        let recovery = tokenize_with_locations(source, filename);

        assert_eq!(strict, recovery);
    }

    #[test]
    fn strict_lexer_rejects_located_unexpected_character() {
        let error =
            try_tokenize_with_locations("let value = 1@;", Some("unexpected.aero".to_string()))
                .expect_err("unexpected character must fail strict lexing");

        match error {
            CompilerError::UnexpectedCharacter {
                character,
                location,
            } => {
                assert_eq!(character, '@');
                assert_eq!(
                    location,
                    SourceLocation::with_filename(1, 14, "unexpected.aero".to_string())
                );
            }
            other => panic!("unexpected strict lexer error: {other}"),
        }
    }

    #[test]
    fn strict_lexer_rejects_invalid_and_non_finite_numbers() {
        for (source, expected_text) in [
            ("let value = 9223372036854775808;", "9223372036854775808"),
            ("let value = 1e+;", "1e+"),
            ("let value = 1e9999;", "1e9999"),
        ] {
            let error = try_tokenize_with_locations(source, Some("number.aero".to_string()))
                .expect_err("invalid number must fail strict lexing");

            match error {
                CompilerError::InvalidNumber { text, location } => {
                    assert_eq!(text, expected_text);
                    assert_eq!(
                        location,
                        SourceLocation::with_filename(1, 13, "number.aero".to_string())
                    );
                }
                other => panic!("unexpected strict lexer error: {other}"),
            }
        }
    }

    #[test]
    fn strict_lexer_rejects_unterminated_ordinary_and_formatted_strings() {
        for source in ["let value = \"unterminated", "let value = f\"unterminated"] {
            let error = try_tokenize_with_locations(source, Some("string.aero".to_string()))
                .expect_err("unterminated string must fail strict lexing");

            match error {
                CompilerError::UnterminatedString { location } => {
                    assert_eq!(
                        location,
                        SourceLocation::with_filename(1, 13, "string.aero".to_string())
                    );
                }
                other => panic!("unexpected strict lexer error: {other}"),
            }
        }
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
    fn test_vec_macro_token() {
        let source = "let v = vec![1, 2, 3];";
        let tokens = tokenize(source);
        assert!(tokens.iter().any(|t| matches!(t, Token::VecMacro)));
    }

    #[test]
    fn test_f_string_token() {
        let source = r#"println!(f"hello {name}")"#;
        let tokens = tokenize(source);
        assert!(
            tokens
                .iter()
                .any(|t| matches!(t, Token::FStringLiteral(s) if s == "hello {name}"))
        );
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
