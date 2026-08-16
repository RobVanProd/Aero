use compiler::{
    CompilerOptions, LanguageProfile, LlvmVerificationMode, Token, check_file, check_program,
    compile_file, compile_program, parse_with_locations, try_tokenize_with_locations,
    verify_llvm_module,
};
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

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
const WORKFLOW_RELATIVE_PATH: &str = "../../.github/workflows/rust.yml";
const SELF_TEST_MARKER: &str = "// CAP-042 TRACKED SELF-TEST";
const INTENTIONAL_PRODUCT_RED: &str =
    "CAP-042 intentional product red: tracked runtime ASCII parser is absent";

static NEXT_WORKSPACE_ID: AtomicU64 = AtomicU64::new(0);

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParserAllocation {
    Node,
    Value,
    Operator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FrontendModel {
    input: Vec<u8>,
    names: Vec<NameRecord>,
    tokens: Vec<TokenRecord>,
    nodes: Vec<NodeRecord>,
    parser_allocations: Vec<ParserAllocation>,
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
    allocations: Vec<ParserAllocation>,
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
        self.allocations.push(ParserAllocation::Node);
        if node.kind <= 17 {
            self.allocations.push(ParserAllocation::Value);
        }
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
            self.allocations.push(ParserAllocation::Operator);
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
                self.allocations.push(ParserAllocation::Operator);
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
                self.allocations.push(ParserAllocation::Operator);
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
    let (nodes, parser_allocations, diagnostic, root) = if lexed.status != 0 {
        (
            Vec::new(),
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
            allocations: Vec::new(),
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
        (parser.nodes, parser.allocations, diagnostic, root)
    };
    let mut model = FrontendModel {
        input: lexed.input,
        names: lexed.names,
        tokens: lexed.tokens,
        nodes,
        parser_allocations,
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

#[derive(Debug)]
struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let serial = NEXT_WORKSPACE_ID.fetch_add(1, Ordering::Relaxed);
        let parent = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| repository_path("../../target"))
            .join("cap042-runtime-parser-tests");
        let root = parent.join(format!("{label}-{}-{nonce}-{serial}", std::process::id()));
        fs::create_dir_all(&root).expect("create CAP-042 test workspace");
        Self { root }
    }

    fn write(&self, name: &str, contents: impl AsRef<[u8]>) -> PathBuf {
        let path = self.root.join(name);
        fs::write(&path, contents).expect("write CAP-042 artifact");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let valid = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("cap042-"));
        if valid {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn run_command_with_stdin(command: &mut Command, input: &[u8]) -> Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn CAP-042 child");
    child
        .stdin
        .take()
        .expect("CAP-042 child stdin")
        .write_all(input)
        .expect("write CAP-042 child stdin");
    child.wait_with_output().expect("wait for CAP-042 child")
}

fn clang_link(
    label: &str,
    workspace: &TestWorkspace,
    inputs: &[&Path],
    optimization: &str,
) -> PathBuf {
    let executable = workspace.root.join(if cfg!(windows) {
        format!("{label}-{optimization}.exe")
    } else {
        format!("{label}-{optimization}")
    });
    let mut command = Command::new("clang");
    command.args([
        "-std=c11",
        optimization,
        "-Wall",
        "-Wextra",
        "-Werror",
        "-Wno-override-module",
    ]);
    command.args(inputs).arg("-o").arg(&executable);
    let output = command.output().expect("execute Clang for CAP-042");
    assert!(
        output.status.success(),
        "link {label} {optimization} (stdout={:?}, stderr={:?})",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    executable
}

fn assert_silent_exit_91(output: &Output, label: &str) {
    assert_eq!(
        output.status.code(),
        Some(91),
        "{label} failed (stdout={:?}, stderr={:?})",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty(), "{label} emitted stdout");
    assert!(output.stderr.is_empty(), "{label} emitted stderr");
}

fn generated_program(kernel_prefix: &str, model: &FrontendModel, checksum_delta: i32) -> String {
    format!(
        "{}\n\nfn main() -> int {{\n    return run_runtime_ascii_parser({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {});\n}}\n",
        kernel_prefix.trim_end(),
        model.diagnostic.status,
        model.diagnostic.offset,
        model.diagnostic.line,
        model.diagnostic.column,
        model.diagnostic.expected_code,
        model.diagnostic.actual_kind,
        model.names.len(),
        model.tokens.len(),
        model.nodes.len(),
        model.root,
        model.checksum + checksum_delta,
    )
}

fn compile_generated(program: &str) -> String {
    check_program(program, options()).expect("generated CAP-042 program checks");
    let first = compile_program(program, options()).expect("generated CAP-042 program compiles");
    let second = compile_program(program, options()).expect("generated CAP-042 program recompiles");
    assert_eq!(first, second, "generated CAP-042 LLVM is nondeterministic");
    verify_llvm_module(&first, LlvmVerificationMode::Required)
        .expect("generated CAP-042 LLVM verifies");
    first
}

#[derive(Debug, Clone, Copy, Default)]
struct BufferState {
    length: usize,
    capacity: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AllocationExpectation {
    success: bool,
    allocations: u64,
    reallocations: u64,
    deallocations: u64,
}

fn simulate_pushes(
    state: &mut BufferState,
    count: usize,
    fail_after: u64,
    successful_events: &mut u64,
    allocations: &mut u64,
    reallocations: &mut u64,
) -> bool {
    for _ in 0..count {
        if state.length == state.capacity {
            if state.capacity == 0 {
                *allocations += 1;
            } else {
                *reallocations += 1;
            }
            if *successful_events >= fail_after {
                return false;
            }
            *successful_events += 1;
            state.capacity = if state.capacity == 0 {
                8
            } else {
                state.capacity * 2
            };
        }
        state.length += 1;
    }
    true
}

fn allocation_expectation(
    input: &[u8],
    model: &FrontendModel,
    fail_after: u64,
) -> AllocationExpectation {
    let mut buffers = [BufferState::default(); 6];
    let mut successful_events = 0;
    let mut allocations = 0;
    let mut reallocations = 0;
    let mut completed = simulate_pushes(
        &mut buffers[0],
        input.len(),
        fail_after,
        &mut successful_events,
        &mut allocations,
        &mut reallocations,
    );
    let mut seen_names = vec![false; model.names.len() + 1];
    if completed {
        for token in &model.tokens {
            if token.name_id != 0 && !seen_names[token.name_id] {
                completed = simulate_pushes(
                    &mut buffers[1],
                    8,
                    fail_after,
                    &mut successful_events,
                    &mut allocations,
                    &mut reallocations,
                );
                seen_names[token.name_id] = completed;
            }
            if completed {
                completed = simulate_pushes(
                    &mut buffers[2],
                    24,
                    fail_after,
                    &mut successful_events,
                    &mut allocations,
                    &mut reallocations,
                );
            }
            if !completed {
                break;
            }
        }
    }
    if completed {
        for event in &model.parser_allocations {
            let (buffer, bytes) = match event {
                ParserAllocation::Node => (3, 16),
                ParserAllocation::Value => (4, 8),
                ParserAllocation::Operator => (5, 20),
            };
            completed = simulate_pushes(
                &mut buffers[buffer],
                bytes,
                fail_after,
                &mut successful_events,
                &mut allocations,
                &mut reallocations,
            );
            if !completed {
                break;
            }
        }
    }
    AllocationExpectation {
        success: completed,
        allocations,
        reallocations,
        deallocations: buffers.iter().filter(|buffer| buffer.capacity != 0).count() as u64,
    }
}

fn allocation_harness(input: &[u8], model: &FrontendModel) -> String {
    let mut cases = String::new();
    for threshold in [0u64, 1, 3, 4, 6, 10, 13, 14, 15, 17, 20, 24, 28, 29, 30, 31] {
        let expected = allocation_expectation(input, model, threshold);
        use std::fmt::Write as _;
        writeln!(
            cases,
            "    {{ UINT64_C({threshold}), {}, UINT64_C({}), UINT64_C({}), UINT64_C({}) }},",
            i32::from(expected.success),
            expected.allocations,
            expected.reallocations,
            expected.deallocations,
        )
        .expect("write CAP-042 allocation case");
    }
    let input_bytes = input
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        r#"
#include <stddef.h>
#include <stdint.h>

extern int aero_program_main(void);
extern int32_t aero_test_reset(uint64_t fail_after_successes);
extern uint64_t aero_test_alloc_calls(void);
extern uint64_t aero_test_realloc_calls(void);
extern uint64_t aero_test_dealloc_calls(void);
extern uint64_t aero_test_live_allocations(void);
extern uint64_t aero_test_size_mismatch_calls(void);

static const uint8_t input_bytes[] = {{ {input_bytes} }};
static size_t input_index;
static int32_t sticky_status;

static void reset_input(void) {{
    input_index = 0;
    sticky_status = 0;
}}

int32_t aero_stdin_read_byte(void) {{
    if (sticky_status != 0) return sticky_status;
    if (input_index < sizeof(input_bytes)) return input_bytes[input_index++];
    sticky_status = -1;
    return sticky_status;
}}

struct Case {{
    uint64_t fail_after;
    int32_t success;
    uint64_t allocations;
    uint64_t reallocations;
    uint64_t deallocations;
}};

int main(void) {{
    const struct Case cases[] = {{
{cases}    }};
    for (size_t index = 0; index < sizeof(cases) / sizeof(cases[0]); ++index) {{
        const struct Case *test = &cases[index];
        if (aero_test_reset(test->fail_after) != 1) return 70;
        reset_input();
        int32_t result = aero_program_main();
        if ((result == 91) != test->success) return 71;
        if (aero_test_alloc_calls() != test->allocations) return 72;
        if (aero_test_realloc_calls() != test->reallocations) return 73;
        if (aero_test_dealloc_calls() != test->deallocations) return 74;
        if (aero_test_live_allocations() != 0) return 75;
        if (aero_test_size_mismatch_calls() != 0) return 76;
    }}
    return 91;
}}
"#,
    )
}

fn workflow_step<'a>(workflow: &'a str, name: &str) -> &'a str {
    let marker = format!("    - name: {name}\n");
    let start = workflow
        .find(&marker)
        .unwrap_or_else(|| panic!("workflow step `{name}` is absent"));
    let remainder = &workflow[start + marker.len()..];
    let end = remainder.find("\n    - name: ").unwrap_or(remainder.len());
    &remainder[..end]
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
fn accepted_rust_lexer_and_parser_overlap_without_supplying_f1b_nodes() {
    let source = "fn score()->int{return !(a+2*3<=b)||c!=d&&e%2==0;}";
    let expected = reference_frontend(source.as_bytes());
    assert_eq!(expected.diagnostic.status, 0);

    let located = try_tokenize_with_locations(source, Some("f1b-overlap.aero".to_string()))
        .expect("accepted overlap passes the strict Rust lexer");
    assert_eq!(located.len(), expected.tokens.len());
    for (actual, expected) in located.iter().zip(&expected.tokens) {
        assert_eq!(production_kind(&actual.token), Some(expected.kind));
        assert_eq!(actual.location.line, expected.line);
        assert_eq!(actual.location.column, expected.column);
    }
    let ast = parse_with_locations(located).expect("accepted overlap passes the Rust parser");
    assert_eq!(ast.len(), 1, "overlap contains one Rust function root");
    assert!(
        !expected.nodes.is_empty(),
        "independent F1B oracle emitted no nodes"
    );
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
fn tracked_runtime_ascii_parser_executes_independent_models_and_mutation_controls() {
    let product_path = repository_path(PRODUCT_RELATIVE_PATH);
    let product =
        fs::read_to_string(&product_path).unwrap_or_else(|_| panic!("{INTENTIONAL_PRODUCT_RED}"));
    let (kernel_prefix, tracked_main) = product
        .split_once(SELF_TEST_MARKER)
        .expect("tracked product retains one parser/self-test boundary");

    assert_eq!(product.matches(SELF_TEST_MARKER).count(), 1);
    assert_eq!(product.matches("fn run_runtime_ascii_parser(").count(), 1);
    assert!(tracked_main.contains("fn main() -> int"));
    assert!(tracked_main.contains("2, 22, 13, 13, 846139"));
    for owner in ["source", "names", "tokens", "nodes", "values", "operators"] {
        assert!(
            product.contains(&format!("let mut {owner}: ByteBuffer = bytes_new();")),
            "tracked CAP-042 product omitted `{owner}`"
        );
    }
    for anchor in [
        "one flat parser driver",
        "parser_cycle_state",
        "binary_precedence(current_kind)",
        "parser_append_target = 1",
        "parser_record_target = 2",
        "node_count * 16",
        "value_records * 8",
        "operator_records * 20",
    ] {
        assert!(
            product.contains(anchor),
            "tracked CAP-042 product omitted `{anchor}`"
        );
    }
    for forbidden in [
        "String",
        "Vec",
        "HashMap",
        "unsafe",
        "fn read_buffer_word(",
        "fn append_node_record(",
        "fn token_word(",
    ] {
        assert!(
            !product.contains(forbidden),
            "tracked CAP-042 product contains `{forbidden}`"
        );
    }

    check_program(&product, options()).expect("tracked CAP-042 product checks");
    let first = compile_program(&product, options()).expect("tracked CAP-042 product compiles");
    let second = compile_program(&product, options()).expect("tracked CAP-042 product recompiles");
    assert_eq!(first, second, "tracked CAP-042 LLVM is nondeterministic");
    verify_llvm_module(&first, LlvmVerificationMode::Required)
        .expect("tracked CAP-042 LLVM verifies");
    assert_eq!(
        first.matches("declare i32 @aero_stdin_read_byte()").count(),
        1
    );
    assert_eq!(first.matches("call i32 @aero_stdin_read_byte()").count(), 1);
    for anchor in [
        "%aero.byte_buffer = type { ptr, i32, i32 }",
        "declare ptr @aero_alloc(i64)",
        "declare ptr @aero_realloc(ptr, i64, i64)",
        "declare void @aero_dealloc(ptr, i64)",
    ] {
        assert!(first.contains(anchor), "tracked LLVM omitted `{anchor}`");
    }
    for forbidden in [
        "double", "fptosi", "sitofp", " nsw ", " nuw ", "@malloc", "@free",
    ] {
        assert!(
            !first.contains(forbidden),
            "tracked LLVM leaked `{forbidden}`"
        );
    }

    let workspace = TestWorkspace::new("tracked");
    let tracked_source = workspace.write("runtime_ascii_parser.aero", &product);
    check_file(&tracked_source, options()).expect("tracked CAP-042 file checks");
    assert_eq!(
        compile_file(&tracked_source, options()).expect("tracked CAP-042 file compiles"),
        first,
        "tracked CAP-042 source/file LLVM diverged"
    );

    let canonical = b"fn score()->int{return 1+2*3-4/2%2;}".to_vec();
    let canonical_model = reference_frontend(&canonical);
    assert_eq!(canonical_model.checksum, 846_139);
    let runtime = repository_path("../../src/compiler/runtime/aero_runtime.c");
    let llvm_path = workspace.write("tracked.ll", &first);
    for optimization in ["-O0", "-O2"] {
        let executable = clang_link(
            "tracked",
            &workspace,
            &[llvm_path.as_path(), runtime.as_path()],
            optimization,
        );
        assert_silent_exit_91(
            &run_command_with_stdin(&mut Command::new(executable), &canonical),
            &format!("tracked CAP-042 {optimization}"),
        );
    }

    let renamed = first.replacen("define i32 @main()", "define i32 @aero_program_main()", 1);
    assert_ne!(renamed, first, "tracked CAP-042 product omitted main");
    let allocation = TestWorkspace::new("allocation");
    let allocation_llvm = allocation.write("program.ll", renamed);
    let harness = allocation.write(
        "harness.c",
        allocation_harness(&canonical, &canonical_model),
    );
    let test_runtime = repository_path("../../src/compiler/runtime/aero_test_runtime.c");
    let executable = clang_link(
        "allocation",
        &allocation,
        &[
            allocation_llvm.as_path(),
            test_runtime.as_path(),
            harness.as_path(),
        ],
        "-O2",
    );
    assert_silent_exit_91(
        &Command::new(executable)
            .output()
            .expect("execute CAP-042 allocation harness"),
        "CAP-042 allocation/failure matrix",
    );

    let all_operators = b"fn f()->int{return a||b&&c==d!=e<f<=g>h>=i+j-k*l/m%n;}".to_vec();
    let unary_grouping = b"fn f()->int{return !-!-(a-(b-c));}".to_vec();
    let maximum_literal = b"fn f()->int{return 2147483647;}".to_vec();
    let wrong_type = b"fn f()->bool{return 0;}".to_vec();
    let overflow = b"fn f()->int{return 2147483648;}".to_vec();
    let empty_expression = b"fn f()->int{return ;}".to_vec();
    let missing_semicolon = b"fn f()->int{return 0}".to_vec();
    let mismatched_paren = b"fn f()->int{return (1;}".to_vec();
    let trailing_function = b"fn f()->int{return 0;}fn g()->int{return 1;}".to_vec();
    let lexical_failure = b"fn f()->int{return @;}".to_vec();
    let node_expression = std::iter::repeat_n("1", 257).collect::<Vec<_>>().join("+");
    let node_bound = format!("fn f()->int{{return {node_expression};}}").into_bytes();
    let stack_bound = format!("fn f()->int{{return {}1;}}", "(".repeat(513)).into_bytes();
    for (label, input) in [
        ("all-operators", all_operators),
        ("unary-grouping", unary_grouping),
        ("maximum-literal", maximum_literal),
        ("wrong-type", wrong_type),
        ("overflow", overflow),
        ("empty-expression", empty_expression.clone()),
        ("missing-semicolon", missing_semicolon),
        ("mismatched-paren", mismatched_paren),
        ("trailing-function", trailing_function),
        ("lexical-failure", lexical_failure),
        ("node-bound", node_bound),
        ("stack-bound", stack_bound),
    ] {
        let model = reference_frontend(&input);
        let program = generated_program(kernel_prefix, &model, 0);
        let llvm = compile_generated(&program);
        let case = TestWorkspace::new(label);
        let llvm_path = case.write("case.ll", llvm);
        let executable = clang_link(
            label,
            &case,
            &[llvm_path.as_path(), runtime.as_path()],
            "-O2",
        );
        assert_silent_exit_91(
            &run_command_with_stdin(&mut Command::new(executable), &input),
            &format!("generated CAP-042 {label}"),
        );
    }

    let wrong_program = generated_program(kernel_prefix, &canonical_model, 1);
    let wrong_llvm = compile_generated(&wrong_program);
    let wrong = TestWorkspace::new("wrong-checksum");
    let wrong_path = wrong.write("wrong.ll", wrong_llvm);
    let wrong_executable = clang_link(
        "wrong",
        &wrong,
        &[wrong_path.as_path(), runtime.as_path()],
        "-O2",
    );
    assert_ne!(
        run_command_with_stdin(&mut Command::new(wrong_executable), &canonical)
            .status
            .code(),
        Some(91),
        "wrong independent checksum was accepted"
    );

    let precedence = kernel_prefix.replacen(
        "if kind == 20 || kind == 21 {\n        return 5;\n    }",
        "if kind == 20 || kind == 21 {\n        return 6;\n    }",
        1,
    );
    let associativity = kernel_prefix.replacen(
        "top_precedence < current_precedence",
        "top_precedence <= current_precedence",
        1,
    );
    let payload = kernel_prefix.replacen(
        "parser_append_0 = 1;\n                    parser_append_1 = literal_value;",
        "parser_append_0 = 1;\n                    parser_append_1 = literal_value + 1;",
        1,
    );
    let child = kernel_prefix.replacen(
        "parser_append_2 = left_id;",
        "parser_append_2 = right_id;",
        1,
    );
    let root = kernel_prefix.replacen("root = node_count;", "root = node_count - 1;", 1);
    for (label, mutated) in [
        ("precedence", precedence),
        ("associativity", associativity),
        ("payload", payload),
        ("child", child),
        ("root", root),
    ] {
        assert_ne!(mutated, kernel_prefix, "{label} mutation anchor was absent");
        let program = generated_program(&mutated, &canonical_model, 0);
        let llvm = compile_generated(&program);
        let case = TestWorkspace::new(label);
        let llvm_path = case.write("mutation.ll", llvm);
        let executable = clang_link(
            label,
            &case,
            &[llvm_path.as_path(), runtime.as_path()],
            "-O2",
        );
        assert_ne!(
            run_command_with_stdin(&mut Command::new(executable), &canonical)
                .status
                .code(),
            Some(91),
            "{label} mutation was accepted"
        );
    }

    let invalid_model = reference_frontend(&empty_expression);
    let diagnostic = kernel_prefix.replacen("diagnostic_code = 100;", "diagnostic_code = 101;", 1);
    assert_ne!(
        diagnostic, kernel_prefix,
        "diagnostic mutation anchor was absent"
    );
    let diagnostic_program = generated_program(&diagnostic, &invalid_model, 0);
    let diagnostic_llvm = compile_generated(&diagnostic_program);
    let diagnostic_case = TestWorkspace::new("diagnostic");
    let diagnostic_path = diagnostic_case.write("diagnostic.ll", diagnostic_llvm);
    let diagnostic_executable = clang_link(
        "diagnostic",
        &diagnostic_case,
        &[diagnostic_path.as_path(), runtime.as_path()],
        "-O2",
    );
    assert_ne!(
        run_command_with_stdin(&mut Command::new(diagnostic_executable), &empty_expression,)
            .status
            .code(),
        Some(91),
        "diagnostic mutation was accepted"
    );

    let public = TestWorkspace::new("public-run");
    let public_source = public.write(
        "runtime_ascii_parser.aero",
        generated_program(kernel_prefix, &canonical_model, 0),
    );
    let mut run = Command::new(env!("CARGO_BIN_EXE_aero"));
    run.args([
        "run",
        public_source.to_str().expect("public source path is UTF-8"),
        "--language-profile",
        PROFILE_NAME,
    ])
    .current_dir(&public.root);
    let output = run_command_with_stdin(&mut run, &canonical);
    assert_eq!(
        output.status.code(),
        Some(91),
        "public CAP-042 runner failed"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        stdout
            .lines()
            .filter(|line| *line == "Exit code: 91")
            .count(),
        1
    );
    assert!(
        !stdout
            .lines()
            .any(|line| line.starts_with("Output:") || line.starts_with("Error output:"))
    );
    assert!(
        output.stderr.is_empty(),
        "public CAP-042 runner emitted stderr"
    );

    for target in ["rocm", "cuda"] {
        let output_path = public.root.join(format!("{target}.ll"));
        let output = Command::new(env!("CARGO_BIN_EXE_aero"))
            .args([
                "build",
                public_source.to_str().expect("public source path is UTF-8"),
                "-o",
                output_path.to_str().expect("output path is UTF-8"),
                "--target",
                target,
                "--language-profile",
                PROFILE_NAME,
            ])
            .current_dir(&public.root)
            .output()
            .expect("execute CAP-042 accelerator rejection");
        assert_eq!(output.status.code(), Some(2));
        assert!(
            !output_path.exists(),
            "{target} rejection created an artifact"
        );
    }

    let workflow =
        fs::read_to_string(repository_path(WORKFLOW_RELATIVE_PATH)).expect("read Rust workflow");
    let linux = workflow_step(&workflow, "Test runtime ASCII parser at O0 and O2");
    for anchor in [
        "runtime_ascii_parser.aero",
        "exact-i32-byte-input-v0",
        "runtime_ascii_parser.linux.repeat.ll",
        "cmp -s",
        "llvm-as-22",
        "opt-22",
        "llc-22",
        "-O0",
        "-O2",
        "Exit code: 91",
    ] {
        assert!(
            linux.contains(anchor),
            "Linux CAP-042 step omitted `{anchor}`"
        );
    }
    let windows = workflow_step(
        &workflow,
        "Test runtime ASCII parser on Windows at O0 and O2",
    );
    for anchor in [
        "runtime_ascii_parser.aero",
        "exact-i32-byte-input-v0",
        "runtime_ascii_parser.windows.repeat.ll",
        "SequenceEqual",
        "llvm-as.exe",
        "opt.exe",
        "llc.exe",
        "-O0",
        "-O2",
        "Exit code: 91",
    ] {
        assert!(
            windows.contains(anchor),
            "Windows CAP-042 step omitted `{anchor}`"
        );
    }
}
