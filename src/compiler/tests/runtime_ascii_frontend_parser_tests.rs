use compiler::{
    CompilerOptions, LanguageProfile, LlvmVerificationMode, check_program, compile_program,
    verify_llvm_module,
};
use std::fs;
use std::path::PathBuf;

const MAX_INPUT_BYTES: usize = 8_192;
const MAX_REAL_TOKENS: usize = 1_024;
const MAX_NAMES: usize = 1_024;
const MAX_IDENTIFIER_BYTES: usize = 63;
const MAX_NODES: usize = 512;
const MAX_PARSE_STACK: usize = 512;
const PROFILE_NAME: &str = "exact-i32-byte-input-v0";
const LEXER_RELATIVE_PATH: &str = "../../examples/aero_frontend_v0/runtime_ascii_lexer.aero";
const STORAGE_RELATIVE_PATH: &str =
    "../../examples/compiler_storage_v0/deterministic_compiler_storage.aero";
const PRODUCT_RELATIVE_PATH: &str = "../../examples/aero_frontend_v0/runtime_ascii_parser.aero";
const INTENTIONAL_PRODUCT_RED: &str =
    "CAP-042 intentional product red: tracked runtime ASCII parser is absent";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NameRecord {
    start: usize,
    length: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NodeRecord {
    kind: i32,
    payload: i32,
    left_id: i32,
    right_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Diagnostic {
    status: i32,
    offset: i32,
    line: i32,
    column: i32,
    expected_code: i32,
    actual_kind: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FrontendModel {
    input: Vec<u8>,
    names: Vec<NameRecord>,
    tokens: Vec<TokenRecord>,
    nodes: Vec<NodeRecord>,
    diagnostic: Diagnostic,
    root: i32,
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

fn lex_failure(
    input: &[u8],
    names: Vec<NameRecord>,
    tokens: Vec<TokenRecord>,
    status: i32,
    offset: usize,
    line: usize,
    column: usize,
) -> LexModel {
    LexModel {
        input: input.to_vec(),
        names,
        tokens,
        status,
        error_offset: i32::try_from(offset).expect("bounded error offset"),
        error_line: i32::try_from(line).expect("bounded error line"),
        error_column: i32::try_from(column).expect("bounded error column"),
    }
}

fn reference_lex(source: &[u8]) -> LexModel {
    if source.len() > MAX_INPUT_BYTES {
        let input = source[..MAX_INPUT_BYTES].to_vec();
        let mut line = 1;
        let mut column = 1;
        for byte in &input {
            advance(*byte, &mut line, &mut column);
        }
        return lex_failure(
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
            return lex_failure(&input, names, tokens, 3, offset, line, column);
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
                    return lex_failure(&input, names, tokens, 3, offset, line, column);
                }
                advance(input[offset], &mut line, &mut column);
                offset += 1;
            }
            continue;
        }
        if byte == b'/' && input.get(offset + 1) == Some(&b'*') {
            let opening = (offset, line, column);
            advance(b'/', &mut line, &mut column);
            advance(b'*', &mut line, &mut column);
            offset += 2;
            let mut closed = false;
            while offset < input.len() {
                if !input[offset].is_ascii() {
                    return lex_failure(&input, names, tokens, 3, offset, line, column);
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
                return lex_failure(&input, names, tokens, 4, opening.0, opening.1, opening.2);
            }
            continue;
        }

        let token_start = offset;
        let token_line = line;
        let token_column = column;
        let token_end;
        let kind;
        let candidate_name;
        if is_identifier_start(byte) {
            let mut end = offset + 1;
            while end < input.len() && is_identifier_continue(input[end]) {
                end += 1;
            }
            if end - token_start > MAX_IDENTIFIER_BYTES {
                return lex_failure(
                    &input,
                    names,
                    tokens,
                    5,
                    token_start,
                    token_line,
                    token_column,
                );
            }
            token_end = end;
            kind = keyword_kind(&input[token_start..token_end]).unwrap_or(1);
            candidate_name = kind == 1;
        } else if byte.is_ascii_digit() {
            let mut end = offset + 1;
            while end < input.len() && input[end].is_ascii_digit() {
                end += 1;
            }
            token_end = end;
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
            return lex_failure(
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
            if let Some(index) = names.iter().position(|name| {
                input[name.start..name.start + name.length] == input[token_start..token_end]
            }) {
                name_id = index + 1;
            } else {
                if names.len() == MAX_NAMES {
                    return lex_failure(
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
            return lex_failure(
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
    LexModel {
        input,
        names,
        tokens,
        status: 0,
        error_offset: -1,
        error_line: 0,
        error_column: 0,
    }
}

fn token_diagnostic(status: i32, token: TokenRecord, expected_code: i32) -> Diagnostic {
    Diagnostic {
        status,
        offset: i32::try_from(token.start).expect("bounded token offset"),
        line: i32::try_from(token.line).expect("bounded token line"),
        column: i32::try_from(token.column).expect("bounded token column"),
        expected_code,
        actual_kind: token.kind,
    }
}

struct ReferenceParser<'a> {
    lexed: &'a LexModel,
    index: usize,
    nodes: Vec<NodeRecord>,
    diagnostic: Option<Diagnostic>,
}

impl<'a> ReferenceParser<'a> {
    fn current(&self) -> TokenRecord {
        self.lexed.tokens[self.index.min(self.lexed.tokens.len() - 1)]
    }

    fn fail(&mut self, status: i32, token: TokenRecord, expected_code: i32) {
        if self.diagnostic.is_none() {
            self.diagnostic = Some(token_diagnostic(status, token, expected_code));
        }
    }

    fn expect(&mut self, kind: i32) -> Option<TokenRecord> {
        let token = self.current();
        if token.kind != kind {
            self.fail(10, token, kind);
            return None;
        }
        self.index += 1;
        Some(token)
    }

    fn append(&mut self, node: NodeRecord, origin: TokenRecord) -> Option<i32> {
        if self.nodes.len() == MAX_NODES {
            self.fail(
                14,
                origin,
                i32::try_from(MAX_NODES).expect("bounded node cap"),
            );
            return None;
        }
        self.nodes.push(node);
        Some(i32::try_from(self.nodes.len()).expect("bounded node id"))
    }

    fn binary(kind: i32) -> Option<(u8, i32)> {
        match kind {
            34 => Some((1, 17)),
            33 => Some((2, 16)),
            26 => Some((3, 14)),
            28 => Some((3, 15)),
            29 => Some((4, 10)),
            30 => Some((4, 11)),
            31 => Some((4, 12)),
            32 => Some((4, 13)),
            20 => Some((5, 8)),
            21 => Some((5, 9)),
            22 => Some((6, 5)),
            23 => Some((6, 6)),
            24 => Some((6, 7)),
            _ => None,
        }
    }

    fn parse_expression(&mut self, minimum_precedence: u8, depth: usize) -> Option<i32> {
        let mut left = self.parse_prefix(depth)?;
        loop {
            let operator = self.current();
            let Some((precedence, node_kind)) = Self::binary(operator.kind) else {
                break;
            };
            if precedence < minimum_precedence {
                break;
            }
            self.index += 1;
            let right = self.parse_expression(precedence + 1, depth)?;
            left = self.append(
                NodeRecord {
                    kind: node_kind,
                    payload: 0,
                    left_id: left,
                    right_id: right,
                },
                operator,
            )?;
        }
        Some(left)
    }

    fn parse_prefix(&mut self, depth: usize) -> Option<i32> {
        let token = self.current();
        if depth == MAX_PARSE_STACK {
            self.fail(
                15,
                token,
                i32::try_from(MAX_PARSE_STACK).expect("bounded stack cap"),
            );
            return None;
        }
        match token.kind {
            1 => {
                self.index += 1;
                self.append(
                    NodeRecord {
                        kind: 2,
                        payload: i32::try_from(token.name_id).expect("bounded name id"),
                        left_id: 0,
                        right_id: 0,
                    },
                    token,
                )
            }
            2 => {
                let mut value = 0i64;
                for byte in &self.lexed.input[token.start..token.start + token.length] {
                    value = value * 10 + i64::from(*byte - b'0');
                    if value > i64::from(i32::MAX) {
                        self.fail(13, token, 103);
                        return None;
                    }
                }
                self.index += 1;
                self.append(
                    NodeRecord {
                        kind: 1,
                        payload: i32::try_from(value).expect("checked i32 literal"),
                        left_id: 0,
                        right_id: 0,
                    },
                    token,
                )
            }
            21 | 27 => {
                self.index += 1;
                let child = self.parse_prefix(depth + 1)?;
                self.append(
                    NodeRecord {
                        kind: if token.kind == 21 { 3 } else { 4 },
                        payload: 0,
                        left_id: child,
                        right_id: 0,
                    },
                    token,
                )
            }
            10 => {
                self.index += 1;
                let child = self.parse_expression(1, depth + 1)?;
                self.expect(11)?;
                Some(child)
            }
            _ => {
                self.fail(11, token, 100);
                None
            }
        }
    }
}

fn checksum_step(checksum: i32, word: i32) -> i32 {
    assert!(word >= 0, "checksum words are nonnegative");
    i32::try_from((i64::from(checksum) * 31 + i64::from(word)) % 1_000_003)
        .expect("checksum remains bounded")
}

fn frontend_checksum(model: &FrontendModel) -> i32 {
    let mut checksum = 17;
    for byte in &model.input {
        checksum = checksum_step(checksum, i32::from(*byte));
    }
    checksum = checksum_step(checksum, 990);
    for name in &model.names {
        checksum = checksum_step(
            checksum,
            i32::try_from(name.start).expect("bounded name start"),
        );
        checksum = checksum_step(
            checksum,
            i32::try_from(name.length).expect("bounded name length"),
        );
    }
    checksum = checksum_step(checksum, 991);
    for token in &model.tokens {
        for word in [
            token.kind,
            i32::try_from(token.start).expect("bounded token start"),
            i32::try_from(token.length).expect("bounded token length"),
            i32::try_from(token.line).expect("bounded token line"),
            i32::try_from(token.column).expect("bounded token column"),
            i32::try_from(token.name_id).expect("bounded token name"),
        ] {
            checksum = checksum_step(checksum, word);
        }
    }
    checksum = checksum_step(checksum, 992);
    for node in &model.nodes {
        for word in [node.kind, node.payload, node.left_id, node.right_id] {
            checksum = checksum_step(checksum, word);
        }
    }
    checksum = checksum_step(checksum, 993);
    for word in [
        model.diagnostic.status,
        model.diagnostic.offset + 1,
        model.diagnostic.line,
        model.diagnostic.column,
        model.diagnostic.expected_code,
        model.diagnostic.actual_kind,
        i32::try_from(model.names.len()).expect("bounded name count"),
        i32::try_from(model.tokens.len()).expect("bounded token count"),
        i32::try_from(model.nodes.len()).expect("bounded node count"),
        model.root,
    ] {
        checksum = checksum_step(checksum, word);
    }
    checksum
}

fn reference_frontend(source: &[u8]) -> FrontendModel {
    let lexed = reference_lex(source);
    let (nodes, diagnostic, root) = if lexed.status != 0 {
        (
            Vec::new(),
            Diagnostic {
                status: lexed.status,
                offset: lexed.error_offset,
                line: lexed.error_line,
                column: lexed.error_column,
                expected_code: 0,
                actual_kind: 0,
            },
            0,
        )
    } else {
        let mut parser = ReferenceParser {
            lexed: &lexed,
            index: 0,
            nodes: Vec::new(),
            diagnostic: None,
        };
        let mut root = 0;
        let parsed = (|| {
            let function_token = parser.expect(3)?;
            let function_name = parser.expect(1)?;
            parser.expect(10)?;
            parser.expect(11)?;
            parser.expect(35)?;
            let return_type = parser.expect(1)?;
            if &lexed.input[return_type.start..return_type.start + return_type.length] != b"int" {
                parser.fail(12, return_type, 102);
                return None;
            }
            parser.expect(12)?;
            let return_token = parser.expect(6)?;
            let expression = parser.parse_expression(1, 0)?;
            parser.expect(18)?;
            parser.expect(13)?;
            parser.expect(0)?;
            let return_node = parser.append(
                NodeRecord {
                    kind: 18,
                    payload: 0,
                    left_id: expression,
                    right_id: 0,
                },
                return_token,
            )?;
            let function_node = parser.append(
                NodeRecord {
                    kind: 19,
                    payload: i32::try_from(function_name.name_id).expect("bounded function name"),
                    left_id: return_node,
                    right_id: 0,
                },
                function_token,
            )?;
            Some(function_node)
        })();
        if let Some(parsed_root) = parsed {
            root = parsed_root;
        }
        let diagnostic = parser.diagnostic.unwrap_or(Diagnostic {
            status: 0,
            offset: -1,
            line: 0,
            column: 0,
            expected_code: 0,
            actual_kind: 0,
        });
        (parser.nodes, diagnostic, root)
    };
    let mut model = FrontendModel {
        input: lexed.input,
        names: lexed.names,
        tokens: lexed.tokens,
        nodes,
        diagnostic,
        root,
        checksum: 0,
    };
    model.checksum = frontend_checksum(&model);
    model
}

fn options() -> CompilerOptions {
    CompilerOptions {
        language_profile: LanguageProfile::ExactI32ByteInputV0,
        ..CompilerOptions::default()
    }
}

fn repository_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

#[test]
fn independent_oracle_freezes_grammar_precedence_nodes_diagnostics_and_bounds() {
    let canonical = reference_frontend(b"fn score()->int{return 1+2*3-4/2%2;}");
    assert_eq!(canonical.diagnostic.status, 0);
    assert_eq!(canonical.names.len(), 2);
    assert_eq!(canonical.tokens.len(), 22);
    assert_eq!(canonical.root, 13);
    assert_eq!(canonical.checksum, 846_139);
    assert_eq!(
        canonical.nodes,
        vec![
            NodeRecord {
                kind: 1,
                payload: 1,
                left_id: 0,
                right_id: 0
            },
            NodeRecord {
                kind: 1,
                payload: 2,
                left_id: 0,
                right_id: 0
            },
            NodeRecord {
                kind: 1,
                payload: 3,
                left_id: 0,
                right_id: 0
            },
            NodeRecord {
                kind: 5,
                payload: 0,
                left_id: 2,
                right_id: 3
            },
            NodeRecord {
                kind: 8,
                payload: 0,
                left_id: 1,
                right_id: 4
            },
            NodeRecord {
                kind: 1,
                payload: 4,
                left_id: 0,
                right_id: 0
            },
            NodeRecord {
                kind: 1,
                payload: 2,
                left_id: 0,
                right_id: 0
            },
            NodeRecord {
                kind: 6,
                payload: 0,
                left_id: 6,
                right_id: 7
            },
            NodeRecord {
                kind: 1,
                payload: 2,
                left_id: 0,
                right_id: 0
            },
            NodeRecord {
                kind: 7,
                payload: 0,
                left_id: 8,
                right_id: 9
            },
            NodeRecord {
                kind: 9,
                payload: 0,
                left_id: 5,
                right_id: 10
            },
            NodeRecord {
                kind: 18,
                payload: 0,
                left_id: 11,
                right_id: 0
            },
            NodeRecord {
                kind: 19,
                payload: 1,
                left_id: 12,
                right_id: 0
            },
        ]
    );
    assert_eq!(canonical, reference_frontend(&canonical.input));

    let precedence = reference_frontend(b"fn f()->int{return a||b&&c==d<e+g*h;}");
    assert_eq!(precedence.diagnostic.status, 0);
    assert_eq!(
        precedence
            .nodes
            .iter()
            .map(|node| node.kind)
            .collect::<Vec<_>>(),
        vec![2, 2, 2, 2, 2, 2, 2, 5, 8, 10, 14, 16, 17, 18, 19]
    );

    let left = reference_frontend(b"fn f()->int{return a-b-c;}");
    assert_eq!(left.diagnostic.status, 0);
    assert_eq!(
        left.nodes[2],
        NodeRecord {
            kind: 9,
            payload: 0,
            left_id: 1,
            right_id: 2
        }
    );
    assert_eq!(
        left.nodes[4],
        NodeRecord {
            kind: 9,
            payload: 0,
            left_id: 3,
            right_id: 4
        }
    );
    let grouped = reference_frontend(b"fn f()->int{return a-(b-c);}");
    assert_eq!(grouped.diagnostic.status, 0);
    assert_eq!(
        grouped.nodes[3],
        NodeRecord {
            kind: 9,
            payload: 0,
            left_id: 2,
            right_id: 3
        }
    );
    assert_eq!(
        grouped.nodes[4],
        NodeRecord {
            kind: 9,
            payload: 0,
            left_id: 1,
            right_id: 4
        }
    );

    let unary = reference_frontend(b"fn f()->int{return !-!-x;}");
    assert_eq!(unary.diagnostic.status, 0);
    assert_eq!(
        unary.nodes.iter().map(|node| node.kind).collect::<Vec<_>>(),
        vec![2, 3, 4, 3, 4, 18, 19]
    );

    for (source, status, expected, actual) in [
        (b"fn f()->bool{return 0;}".as_slice(), 12, 102, 1),
        (b"fn f()->int{return 2147483648;}".as_slice(), 13, 103, 2),
        (b"fn f()->int{return ;}".as_slice(), 11, 100, 18),
        (b"fn f()->int{return 0}".as_slice(), 10, 18, 13),
        (b"fn f()->int{return (1;}".as_slice(), 10, 11, 18),
        (
            b"fn f()->int{return 0;}fn g()->int{return 1;}".as_slice(),
            10,
            0,
            3,
        ),
    ] {
        let model = reference_frontend(source);
        assert_eq!(model.diagnostic.status, status, "source={:?}", source);
        assert_eq!(
            model.diagnostic.expected_code, expected,
            "source={:?}",
            source
        );
        assert_eq!(model.diagnostic.actual_kind, actual, "source={:?}", source);
        assert_eq!(model, reference_frontend(source));
    }

    let lexical = reference_frontend(b"fn f()->int{return @;}");
    assert_eq!(lexical.diagnostic.status, 4);
    assert!(lexical.nodes.is_empty());

    let node_expression = std::iter::repeat_n("1", 257).collect::<Vec<_>>().join("+");
    let node_source = format!("fn f()->int{{return {node_expression};}}");
    let node_bound = reference_frontend(node_source.as_bytes());
    assert_eq!(node_bound.diagnostic.status, 14);
    assert_eq!(node_bound.nodes.len(), MAX_NODES);

    let stack_source = format!("fn f()->int{{return {}1;}}", "(".repeat(513));
    let stack_bound = reference_frontend(stack_source.as_bytes());
    assert_eq!(stack_bound.diagnostic.status, 15);
    assert!(stack_bound.nodes.is_empty());
}

#[test]
fn accepted_f1a_and_d1_products_remain_checked_deterministic_and_structurally_distinct() {
    assert_eq!(PROFILE_NAME, "exact-i32-byte-input-v0");
    let lexer = fs::read_to_string(repository_path(LEXER_RELATIVE_PATH))
        .expect("read accepted F1A product");
    let storage = fs::read_to_string(repository_path(STORAGE_RELATIVE_PATH))
        .expect("read accepted D1 product");
    assert_eq!(lexer.matches("fn run_runtime_ascii_lexer(").count(), 1);
    assert_eq!(
        lexer
            .matches("let mut tokens: ByteBuffer = bytes_new();")
            .count(),
        1
    );
    assert!(lexer.contains("token_count * 24"));
    assert!(!lexer.contains("fn run_runtime_ascii_parser("));
    assert_eq!(storage.matches("fn run_compiler_storage(").count(), 1);
    assert_eq!(
        storage
            .matches("let mut nodes: ByteBuffer = bytes_new();")
            .count(),
        1
    );
    assert!(storage.contains("node_count * 16"));

    for (label, product) in [("accepted F1A", lexer), ("accepted D1", storage)] {
        check_program(&product, options()).unwrap_or_else(|error| panic!("{label} check: {error}"));
        let first = compile_program(&product, options())
            .unwrap_or_else(|error| panic!("{label} compile: {error}"));
        let second = compile_program(&product, options())
            .unwrap_or_else(|error| panic!("{label} recompile: {error}"));
        assert_eq!(first, second, "{label} LLVM is nondeterministic");
        verify_llvm_module(&first, LlvmVerificationMode::Required)
            .unwrap_or_else(|error| panic!("{label} LLVM verification: {error}"));
    }
}

#[test]
fn tracked_runtime_ascii_parser_product_is_present_before_native_evidence() {
    let product = fs::read_to_string(repository_path(PRODUCT_RELATIVE_PATH))
        .unwrap_or_else(|_| panic!("{INTENTIONAL_PRODUCT_RED}"));
    assert_eq!(product.matches("fn run_runtime_ascii_parser(").count(), 1);
    assert!(product.contains("// CAP-042 TRACKED SELF-TEST"));
}
