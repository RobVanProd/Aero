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
    
    // Operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Assign,
    Arrow,  // ->
    
    // Delimiters
    Semicolon,
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Dot,
    
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
            '/' => { tokens.push(Token::Divide); chars.next(); }
            '%' => { tokens.push(Token::Modulo); chars.next(); }
            ';' => { tokens.push(Token::Semicolon); chars.next(); }
            '{' => { tokens.push(Token::LeftBrace); chars.next(); }
            '}' => { tokens.push(Token::RightBrace); chars.next(); }
            '(' => { tokens.push(Token::LeftParen); chars.next(); }
            ')' => { tokens.push(Token::RightParen); chars.next(); }
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
            // Handle assignment
            '=' => { tokens.push(Token::Assign); chars.next(); }
            // Float literals starting with decimal point (e.g., .5)
            '.' => {
                chars.next(); // consume the '.'
                if let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() {
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
                        has_dot = true;
                        num_str.push(chars.next().unwrap());
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
                let token = match ident_str.as_str() {
                    "let" => Token::Let,
                    "fn" => Token::Fn,
                    "return" => Token::Return,
                    "mut" => Token::Mut,
                    _ => Token::Identifier(ident_str),
                };
                tokens.push(token);
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
}