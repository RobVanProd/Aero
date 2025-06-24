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
                        tokens.push(format!("float_literal:{}", num_str));
                    } else {
                        // Just a dot, not a float literal
                        tokens.push(".".to_string());
                    }
                } else {
                    tokens.push(".".to_string());
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
                    "return" => tokens.push("keyword:return".to_string()),
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


