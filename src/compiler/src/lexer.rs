pub fn tokenize(source: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            // Whitespace
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            // Operators and delimiters
            '+' => { tokens.push("+".to_string()); chars.next(); }
            '-' => { tokens.push("-".to_string()); chars.next(); }
            '*' => { tokens.push("*".to_string()); chars.next(); }
            '/' => { tokens.push("/".to_string()); chars.next(); }
            '%' => { tokens.push("%".to_string()); chars.next(); }
            '=' => { tokens.push("=".to_string()); chars.next(); }
            ';' => { tokens.push(";".to_string()); chars.next(); }
            '{' => { tokens.push("{".to_string()); chars.next(); }
            '}' => { tokens.push("}".to_string()); chars.next(); }
            '(' => { tokens.push("(".to_string()); chars.next(); }
            ')' => { tokens.push(")".to_string()); chars.next(); }
            // Integer and Float Literals
            '0'..='9' => {
                let mut num_str = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() || d == '.' {
                        num_str.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if num_str.contains('.') {
                    tokens.push(format!("float_literal:{}", num_str));
                } else {
                    tokens.push(format!("integer_literal:{}", num_str));
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
                match ident_str.as_str() {
                    "let" => tokens.push("keyword:let".to_string()),
                    "fn" => tokens.push("keyword:fn".to_string()),
                    _ => tokens.push(format!("identifier:{}", ident_str)),
                }
            }
            _ => {
                // Handle unexpected characters or errors
                eprintln!("Unexpected character: {}", c);
                chars.next();
            }
        }
    }
    tokens
}


