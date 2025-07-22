// Standalone lexer test - copy of lexer code for testing
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
       