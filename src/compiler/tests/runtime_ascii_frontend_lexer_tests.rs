use compiler::{Token, try_tokenize_with_locations};
use std::fs;
use std::path::PathBuf;

const MAX_INPUT_BYTES: usize = 8_192;
const MAX_REAL_TOKENS: usize = 1_024;
const MAX_NAMES: usize = 1_024;
const MAX_IDENTIFIER_BYTES: usize = 63;
const PRODUCT_RELATIVE_PATH: &str = "../../examples/aero_frontend_v0/runtime_ascii_lexer.aero";
const SELF_TEST_MARKER: &str = "// CAP-041 TRACKED SELF-TEST";
const INTENTIONAL_PRODUCT_RED: &str =
    "CAP-041 intentional product red: tracked runtime ASCII lexer is absent";

#[derive(Debug, Clone, PartialEq, Eq)]
struct NameRecord {
    start: usize,
    length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TokenRecord {
    kind: i32,
    start: usize,
    length: usize,
    line: usize,
    column: usize,
    name_id: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LexModel {
    input: Vec<u8>,
    names: Vec<NameRecord>,
    tokens: Vec<TokenRecord>,
    status: i32,
    error_offset: i32,
    error_line: i32,
    error_column: i32,
    checksum: i32,
}

fn is_identifier_start(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphabetic()
}

fn is_identifier_continue(byte: u8) -> bool {
    is_identifier_start(byte) || byte.is_ascii_digit()
}

fn keyword_kind(spelling: &[u8]) -> Option<i32> {
    match spelling {
        b"fn" => Some(3),
        b"let" => Some(4),
        b"mut" => Some(5),
        b"return" => Some(6),
        b"if" => Some(7),
        b"else" => Some(8),
        b"while" => Some(9),
        _ => None,
    }
}

fn pair_kind(first: u8, second: u8) -> Option<i32> {
    match (first, second) {
        (b'=', b'=') => Some(26),
        (b'!', b'=') => Some(28),
        (b'<', b'=') => Some(30),
        (b'>', b'=') => Some(32),
        (b'&', b'&') => Some(33),
        (b'|', b'|') => Some(34),
        (b'-', b'>') => Some(35),
        (b'=', b'>') => Some(36),
        _ => None,
    }
}

fn single_kind(byte: u8) -> Option<i32> {
    match byte {
        b'(' => Some(10),
        b')' => Some(11),
        b'{' => Some(12),
        b'}' => Some(13),
        b'[' => Some(14),
        b']' => Some(15),
        b',' => Some(16),
        b':' => Some(17),
        b';' => Some(18),
        b'.' => Some(19),
        b'+' => Some(20),
        b'-' => Some(21),
        b'*' => Some(22),
        b'/' => Some(23),
        b'%' => Some(24),
        b'=' => Some(25),
        b'!' => Some(27),
        b'<' => Some(29),
        b'>' => Some(31),
        _ => None,
    }
}

fn advance(byte: u8, line: &mut usize, column: &mut usize) {
    if byte == b'\n' {
        *line += 1;
        *column = 1;
    } else {
        *column += 1;
    }
}

fn checksum_step(checksum: i32, word: usize) -> i32 {
    let word = i64::try_from(word).expect("bounded checksum word");
    i32::try_from((i64::from(checksum) * 31 + word) % 1_000_003).expect("checksum remains bounded")
}

fn model_checksum(
    input: &[u8],
    names: &[NameRecord],
    tokens: &[TokenRecord],
    status: i32,
    error_offset: i32,
    error_line: i32,
    error_column: i32,
) -> i32 {
    let mut checksum = 17;
    for byte in input {
        checksum = checksum_step(checksum, usize::from(*byte));
    }
    checksum = checksum_step(checksum, 990);
    for name in names {
        checksum = checksum_step(checksum, name.start);
        checksum = checksum_step(checksum, name.length);
    }
    checksum = checksum_step(checksum, 991);
    for token in tokens {
        for word in [
            usize::try_from(token.kind).expect("nonnegative token kind"),
            token.start,
            token.length,
            token.line,
            token.column,
            token.name_id,
        ] {
            checksum = checksum_step(checksum, word);
        }
    }
    checksum = checksum_step(checksum, 992);
    for word in [
        usize::try_from(status).expect("nonnegative status"),
        usize::try_from(error_offset + 1).expect("encoded error offset"),
        usize::try_from(error_line).expect("nonnegative error line"),
        usize::try_from(error_column).expect("nonnegative error column"),
        names.len(),
        tokens.len(),
    ] {
        checksum = checksum_step(checksum, word);
    }
    checksum
}

fn finish_model(
    input: Vec<u8>,
    names: Vec<NameRecord>,
    tokens: Vec<TokenRecord>,
    status: i32,
    error_offset: i32,
    error_line: i32,
    error_column: i32,
) -> LexModel {
    let checksum = model_checksum(
        &input,
        &names,
        &tokens,
        status,
        error_offset,
        error_line,
        error_column,
    );
    LexModel {
        input,
        names,
        tokens,
        status,
        error_offset,
        error_line,
        error_column,
        checksum,
    }
}

fn fail_model(
    input: &[u8],
    names: Vec<NameRecord>,
    tokens: Vec<TokenRecord>,
    status: i32,
    offset: usize,
    line: usize,
    column: usize,
) -> LexModel {
    finish_model(
        input.to_vec(),
        names,
        tokens,
        status,
        i32::try_from(offset).expect("bounded error offset"),
        i32::try_from(line).expect("bounded error line"),
        i32::try_from(column).expect("bounded error column"),
    )
}

fn reference_lex(source: &[u8]) -> LexModel {
    if source.len() > MAX_INPUT_BYTES {
        let input = source[..MAX_INPUT_BYTES].to_vec();
        let mut line = 1;
        let mut column = 1;
        for byte in &input {
            advance(*byte, &mut line, &mut column);
        }
        return fail_model(
            &input,
            Vec::new(),
            Vec::new(),
            2,
            MAX_INPUT_BYTES,
            line,
            column,
        );
    }

    let input = source.to_vec();
    let mut names = Vec::<NameRecord>::new();
    let mut tokens = Vec::<TokenRecord>::new();
    let mut offset = 0;
    let mut line = 1;
    let mut column = 1;

    while offset < input.len() {
        let byte = input[offset];
        if !byte.is_ascii() {
            return fail_model(&input, names, tokens, 3, offset, line, column);
        }

        if matches!(byte, b' ' | b'\t' | b'\r' | b'\n') {
            advance(byte, &mut line, &mut column);
            offset += 1;
            continue;
        }

        if byte == b'/' && input.get(offset + 1) == Some(&b'/') {
            advance(b'/', &mut line, &mut column);
            advance(b'/', &mut line, &mut column);
            offset += 2;
            while offset < input.len() && !matches!(input[offset], b'\r' | b'\n') {
                if !input[offset].is_ascii() {
                    return fail_model(&input, names, tokens, 3, offset, line, column);
                }
                advance(input[offset], &mut line, &mut column);
                offset += 1;
            }
            continue;
        }

        if byte == b'/' && input.get(offset + 1) == Some(&b'*') {
            let opening_offset = offset;
            let opening_line = line;
            let opening_column = column;
            advance(b'/', &mut line, &mut column);
            advance(b'*', &mut line, &mut column);
            offset += 2;
            let mut closed = false;
            while offset < input.len() {
                if !input[offset].is_ascii() {
                    return fail_model(&input, names, tokens, 3, offset, line, column);
                }
                if input[offset] == b'*' && input.get(offset + 1) == Some(&b'/') {
                    advance(b'*', &mut line, &mut column);
                    advance(b'/', &mut line, &mut column);
                    offset += 2;
                    closed = true;
                    break;
                }
                advance(input[offset], &mut line, &mut column);
                offset += 1;
            }
            if !closed {
                return fail_model(
                    &input,
                    names,
                    tokens,
                    4,
                    opening_offset,
                    opening_line,
                    opening_column,
                );
            }
            continue;
        }

        let token_start = offset;
        let token_line = line;
        let token_column = column;
        let mut token_end;
        let kind;
        let candidate_name;

        if is_identifier_start(byte) {
            token_end = offset + 1;
            while token_end < input.len() && is_identifier_continue(input[token_end]) {
                token_end += 1;
            }
            let length = token_end - token_start;
            if length > MAX_IDENTIFIER_BYTES {
                return fail_model(
                    &input,
                    names,
                    tokens,
                    5,
                    token_start,
                    token_line,
                    token_column,
                );
            }
            kind = keyword_kind(&input[token_start..token_end]).unwrap_or(1);
            candidate_name = kind == 1;
        } else if byte.is_ascii_digit() {
            token_end = offset + 1;
            while token_end < input.len() && input[token_end].is_ascii_digit() {
                token_end += 1;
            }
            kind = 2;
            candidate_name = false;
        } else if let Some(second) = input.get(offset + 1).copied()
            && let Some(pair) = pair_kind(byte, second)
        {
            token_end = offset + 2;
            kind = pair;
            candidate_name = false;
        } else if let Some(single) = single_kind(byte) {
            token_end = offset + 1;
            kind = single;
            candidate_name = false;
        } else {
            return fail_model(
                &input,
                names,
                tokens,
                4,
                token_start,
                token_line,
                token_column,
            );
        }

        let mut name_id = 0;
        if candidate_name {
            if let Some((index, _)) = names.iter().enumerate().find(|(_, name)| {
                input[name.start..name.start + name.length] == input[token_start..token_end]
            }) {
                name_id = index + 1;
            } else {
                if names.len() == MAX_NAMES {
                    return fail_model(
                        &input,
                        names,
                        tokens,
                        7,
                        token_start,
                        token_line,
                        token_column,
                    );
                }
                names.push(NameRecord {
                    start: token_start,
                    length: token_end - token_start,
                });
                name_id = names.len();
            }
        }

        if tokens.len() == MAX_REAL_TOKENS {
            return fail_model(
                &input,
                names,
                tokens,
                6,
                token_start,
                token_line,
                token_column,
            );
        }

        tokens.push(TokenRecord {
            kind,
            start: token_start,
            length: token_end - token_start,
            line: token_line,
            column: token_column,
            name_id,
        });
        while offset < token_end {
            advance(input[offset], &mut line, &mut column);
            offset += 1;
        }
    }

    tokens.push(TokenRecord {
        kind: 0,
        start: input.len(),
        length: 0,
        line,
        column,
        name_id: 0,
    });
    finish_model(input, names, tokens, 0, -1, 0, 0)
}

fn production_kind(token: &Token) -> Option<i32> {
    match token {
        Token::Eof => Some(0),
        Token::Identifier(_) => Some(1),
        Token::IntegerLiteral(_) => Some(2),
        Token::Fn => Some(3),
        Token::Let => Some(4),
        Token::Mut => Some(5),
        Token::Return => Some(6),
        Token::If => Some(7),
        Token::Else => Some(8),
        Token::While => Some(9),
        Token::LeftParen => Some(10),
        Token::RightParen => Some(11),
        Token::LeftBrace => Some(12),
        Token::RightBrace => Some(13),
        Token::LeftBracket => Some(14),
        Token::RightBracket => Some(15),
        Token::Comma => Some(16),
        Token::Colon => Some(17),
        Token::Semicolon => Some(18),
        Token::Dot => Some(19),
        Token::Plus => Some(20),
        Token::Minus => Some(21),
        Token::Multiply => Some(22),
        Token::Divide => Some(23),
        Token::Modulo => Some(24),
        Token::Assign => Some(25),
        Token::Equal => Some(26),
        Token::LogicalNot => Some(27),
        Token::NotEqual => Some(28),
        Token::LessThan => Some(29),
        Token::LessEqual => Some(30),
        Token::GreaterThan => Some(31),
        Token::GreaterEqual => Some(32),
        Token::LogicalAnd => Some(33),
        Token::LogicalOr => Some(34),
        Token::Arrow => Some(35),
        Token::FatArrow => Some(36),
        _ => None,
    }
}

#[test]
fn independent_oracle_freezes_owned_records_limits_locations_and_checksum() {
    let canonical = reference_lex(b"fn f()->int{return f;}");
    assert_eq!(canonical.status, 0);
    assert_eq!(
        canonical.names,
        vec![
            NameRecord {
                start: 3,
                length: 1,
            },
            NameRecord {
                start: 8,
                length: 3,
            },
        ]
    );
    assert_eq!(
        canonical.tokens,
        vec![
            TokenRecord {
                kind: 3,
                start: 0,
                length: 2,
                line: 1,
                column: 1,
                name_id: 0
            },
            TokenRecord {
                kind: 1,
                start: 3,
                length: 1,
                line: 1,
                column: 4,
                name_id: 1
            },
            TokenRecord {
                kind: 10,
                start: 4,
                length: 1,
                line: 1,
                column: 5,
                name_id: 0
            },
            TokenRecord {
                kind: 11,
                start: 5,
                length: 1,
                line: 1,
                column: 6,
                name_id: 0
            },
            TokenRecord {
                kind: 35,
                start: 6,
                length: 2,
                line: 1,
                column: 7,
                name_id: 0
            },
            TokenRecord {
                kind: 1,
                start: 8,
                length: 3,
                line: 1,
                column: 9,
                name_id: 2
            },
            TokenRecord {
                kind: 12,
                start: 11,
                length: 1,
                line: 1,
                column: 12,
                name_id: 0
            },
            TokenRecord {
                kind: 6,
                start: 12,
                length: 6,
                line: 1,
                column: 13,
                name_id: 0
            },
            TokenRecord {
                kind: 1,
                start: 19,
                length: 1,
                line: 1,
                column: 20,
                name_id: 1
            },
            TokenRecord {
                kind: 18,
                start: 20,
                length: 1,
                line: 1,
                column: 21,
                name_id: 0
            },
            TokenRecord {
                kind: 13,
                start: 21,
                length: 1,
                line: 1,
                column: 22,
                name_id: 0
            },
            TokenRecord {
                kind: 0,
                start: 22,
                length: 0,
                line: 1,
                column: 23,
                name_id: 0
            },
        ]
    );
    assert_eq!(canonical.checksum, 602_295);

    let located = reference_lex(b"let alpha\r\n\talpha/* x\ny */+12");
    assert_eq!(located.status, 0);
    assert_eq!(located.names.len(), 1);
    assert_eq!(located.tokens.last().expect("EOF").kind, 0);
    assert_eq!(located.tokens[2].line, 2);
    assert_eq!(located.tokens[2].column, 2);
    assert_eq!(located.tokens[3].line, 3);
    assert_eq!(located.tokens[3].column, 5);

    let non_ascii = reference_lex(&[b'\n', 0xff]);
    assert_eq!(
        (
            non_ascii.status,
            non_ascii.error_offset,
            non_ascii.error_line,
            non_ascii.error_column
        ),
        (3, 1, 2, 1)
    );
    assert!(non_ascii.tokens.is_empty());

    let unsupported = reference_lex(b"fn @");
    assert_eq!((unsupported.status, unsupported.error_offset), (4, 3));
    assert_eq!(unsupported.tokens.len(), 1);
    assert!(!unsupported.tokens.iter().any(|token| token.kind == 0));

    let unterminated = reference_lex(b"let x /* open\n");
    assert_eq!(
        (
            unterminated.status,
            unterminated.error_offset,
            unterminated.error_line,
            unterminated.error_column
        ),
        (4, 6, 1, 7)
    );

    let long_name = reference_lex(&vec![b'a'; MAX_IDENTIFIER_BYTES + 1]);
    assert_eq!((long_name.status, long_name.error_offset), (5, 0));

    let repeated = "x ".repeat(MAX_REAL_TOKENS + 1);
    let token_bound = reference_lex(repeated.as_bytes());
    assert_eq!(
        (
            token_bound.status,
            token_bound.tokens.len(),
            token_bound.names.len()
        ),
        (6, MAX_REAL_TOKENS, 1)
    );

    let unique = (0..=MAX_NAMES)
        .map(|index| format!("n{index} "))
        .collect::<String>();
    let name_bound = reference_lex(unique.as_bytes());
    assert_eq!(
        (
            name_bound.status,
            name_bound.tokens.len(),
            name_bound.names.len()
        ),
        (7, MAX_REAL_TOKENS, MAX_NAMES)
    );

    let oversized = reference_lex(&vec![b' '; MAX_INPUT_BYTES + 1]);
    assert_eq!(
        (
            oversized.status,
            oversized.error_offset,
            oversized.tokens.len()
        ),
        (2, MAX_INPUT_BYTES as i32, 0)
    );
    assert_eq!(oversized.input.len(), MAX_INPUT_BYTES);
}

#[test]
fn accepted_rust_lexer_overlap_matches_independent_kind_and_location_oracle() {
    let source = concat!(
        "fn item(alpha: int)->int {\r\n",
        "\tlet mut value = 12 + 3-2*4/5%6; // line\n",
        "\tif value==1 && value!=2 || value<3 || value<=4 || value>5 || value>=6 {\n",
        "\t\twhile !value { return alpha=>value; }\n",
        "\t} else { return item.value[0], alpha; }\n",
        "}\n",
    );
    let expected = reference_lex(source.as_bytes());
    assert_eq!(expected.status, 0);

    let actual = try_tokenize_with_locations(source, Some("f1a.aero".to_string()))
        .expect("accepted overlap must pass the strict Rust lexer");
    assert_eq!(actual.len(), expected.tokens.len());
    for (actual, expected) in actual.iter().zip(&expected.tokens) {
        assert_eq!(production_kind(&actual.token), Some(expected.kind));
        assert_eq!(actual.location.line, expected.line);
        assert_eq!(actual.location.column, expected.column);
    }
}

#[test]
fn tracked_runtime_ascii_lexer_product_is_present_before_native_evidence() {
    let product_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(PRODUCT_RELATIVE_PATH);
    let product =
        fs::read_to_string(&product_path).unwrap_or_else(|_| panic!("{INTENTIONAL_PRODUCT_RED}"));
    assert!(
        product.contains("fn run_runtime_ascii_lexer("),
        "{INTENTIONAL_PRODUCT_RED}"
    );
    assert_eq!(product.matches(SELF_TEST_MARKER).count(), 1);
    assert!(product.contains("fn main() -> int"));
}
