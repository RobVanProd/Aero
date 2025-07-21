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