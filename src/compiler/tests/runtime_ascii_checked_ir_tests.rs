use compiler::{
    CompilerOptions, LanguageProfile, LlvmVerificationMode, SemanticAnalyzer, check_file,
    check_program, compile_file, compile_program, parse_with_locations,
    prepare_checked_program_for_compiler_service, try_tokenize_with_locations, verify_llvm_module,
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
const PROFILE_NAME: &str = "exact-i32-byte-input-v0";
const PARSER_RELATIVE_PATH: &str = "../../examples/aero_frontend_v0/runtime_ascii_parser.aero";
const PRODUCT_RELATIVE_PATH: &str = "../../examples/aero_frontend_v0/runtime_ascii_semantics.aero";
const CHECKED_PRODUCT_RELATIVE_PATH: &str =
    "../../examples/aero_frontend_v0/runtime_ascii_checked_ir.aero";
const RUNTIME_RELATIVE_PATH: &str = "../../src/compiler/runtime/aero_runtime.c";
const TEST_RUNTIME_RELATIVE_PATH: &str = "../../src/compiler/runtime/aero_test_runtime.c";
const WORKFLOW_RELATIVE_PATH: &str = "../../.github/workflows/rust.yml";
const SELF_TEST_MARKER: &str = "// CAP-043 TRACKED SELF-TEST";
const INTENTIONAL_PRODUCT_RED: &str =
    "CAP-043 intentional product red: tracked runtime ASCII semantic facts are absent";
const CHECKED_SELF_TEST_MARKER: &str = "// CAP-044 TRACKED SELF-TEST";
const INTENTIONAL_CHECKED_PRODUCT_RED: &str =
    "CAP-044 intentional product red: tracked runtime ASCII checked IR is absent";

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NodeRecord {
    kind: i32,
    payload: i32,
    left_id: i32,
    right_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OriginRecord {
    node_id: i32,
    offset: i32,
    line: i32,
    column: i32,
    token_kind: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FrontendDiagnostic {
    status: i32,
    offset: i32,
    line: i32,
    column: i32,
    expected_code: i32,
    actual_kind: i32,
}

impl FrontendDiagnostic {
    const fn success() -> Self {
        Self {
            status: 0,
            offset: -1,
            line: 0,
            column: 0,
            expected_code: 0,
            actual_kind: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FrontendModel {
    input: Vec<u8>,
    names: Vec<NameRecord>,
    tokens: Vec<TokenRecord>,
    nodes: Vec<NodeRecord>,
    origins: Vec<OriginRecord>,
    diagnostic: FrontendDiagnostic,
    root: i32,
    checksum: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SymbolRecord {
    kind: i32,
    name_id: i32,
    function_node_id: i32,
    return_type: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FactRecord {
    node_id: i32,
    logical_type: i32,
    ownership: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SemanticDiagnostic {
    status: i32,
    node_id: i32,
    offset: i32,
    line: i32,
    column: i32,
    code: i32,
    expected_type: i32,
    actual_type: i32,
}

impl SemanticDiagnostic {
    const fn success() -> Self {
        Self {
            status: 0,
            node_id: 0,
            offset: -1,
            line: 0,
            column: 0,
            code: 0,
            expected_type: 0,
            actual_type: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticModel {
    origins: Vec<OriginRecord>,
    symbols: Vec<SymbolRecord>,
    facts: Vec<FactRecord>,
    diagnostic: SemanticDiagnostic,
    root_type: i32,
    checksum: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CheckedDiagnostic {
    status: i32,
    node_id: i32,
    offset: i32,
    line: i32,
    column: i32,
    code: i32,
    expected: i32,
    actual: i32,
}

impl CheckedDiagnostic {
    const fn success() -> Self {
        Self {
            status: 0,
            node_id: 0,
            offset: -1,
            line: 0,
            column: 0,
            code: 0,
            expected: 0,
            actual: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CheckedValueRecord {
    node_id: i32,
    operand_kind: i32,
    operand_payload: i32,
    sign: i32,
    magnitude_high: i32,
    magnitude_low: i32,
    evaluated: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CheckedInstructionRecord {
    instruction_id: i32,
    opcode: i32,
    result_id: i32,
    result_type: i32,
    left_kind: i32,
    left_payload: i32,
    right_kind: i32,
    right_payload: i32,
    origin_node_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CheckedResultRecord {
    result_id: i32,
    result_type: i32,
    definition_instruction_id: i32,
    origin_node_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CheckedModel {
    attempted: i32,
    values: Vec<CheckedValueRecord>,
    instructions: Vec<CheckedInstructionRecord>,
    results: Vec<CheckedResultRecord>,
    words: Vec<i32>,
    diagnostic: CheckedDiagnostic,
    root_kind: i32,
    root_payload: i32,
    root_type: i32,
    checksum: i32,
}

fn repository_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn options() -> CompilerOptions {
    CompilerOptions {
        language_profile: LanguageProfile::ExactI32ByteInputV0,
        ..CompilerOptions::default()
    }
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
            .join("cap043-runtime-semantics-tests");
        let root = parent.join(format!(
            "cap043-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create CAP-043 test workspace");
        Self { root }
    }

    fn write(&self, name: &str, contents: impl AsRef<[u8]>) -> PathBuf {
        let path = self.root.join(name);
        fs::write(&path, contents).expect("write CAP-043 artifact");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let valid = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("cap043-"));
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
        .expect("spawn CAP-043 child");
    child
        .stdin
        .take()
        .expect("CAP-043 child stdin")
        .write_all(input)
        .expect("write CAP-043 child stdin");
    child.wait_with_output().expect("wait for CAP-043 child")
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
    let output = command.output().expect("execute Clang for CAP-043");
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
) -> FrontendModel {
    FrontendModel {
        input: input.to_vec(),
        names,
        tokens,
        nodes: Vec::new(),
        origins: Vec::new(),
        diagnostic: FrontendDiagnostic {
            status,
            offset: i32::try_from(offset).expect("bounded offset"),
            line: i32::try_from(line).expect("bounded line"),
            column: i32::try_from(column).expect("bounded column"),
            expected_code: 0,
            actual_kind: 0,
        },
        root: 0,
        checksum: 0,
    }
}

fn reference_lex(source: &[u8]) -> FrontendModel {
    if source.len() > MAX_INPUT_BYTES {
        let retained = &source[..MAX_INPUT_BYTES];
        let mut line = 1;
        let mut column = 1;
        for byte in retained {
            advance(*byte, &mut line, &mut column);
        }
        return lex_failure(
            retained,
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
            while offset < input.len() && input[offset] != b'\n' {
                advance(input[offset], &mut line, &mut column);
                offset += 1;
            }
            continue;
        }
        if byte == b'/' && input.get(offset + 1) == Some(&b'*') {
            let start = offset;
            let start_line = line;
            let start_column = column;
            advance(input[offset], &mut line, &mut column);
            advance(input[offset + 1], &mut line, &mut column);
            offset += 2;
            let mut closed = false;
            while offset < input.len() {
                if input[offset] == b'*' && input.get(offset + 1) == Some(&b'/') {
                    advance(input[offset], &mut line, &mut column);
                    advance(input[offset + 1], &mut line, &mut column);
                    offset += 2;
                    closed = true;
                    break;
                }
                advance(input[offset], &mut line, &mut column);
                offset += 1;
            }
            if !closed {
                return lex_failure(&input, names, tokens, 4, start, start_line, start_column);
            }
            continue;
        }

        let start = offset;
        let start_line = line;
        let start_column = column;
        let mut length = 1;
        let mut name_id = 0;
        let kind = if is_identifier_start(byte) {
            while offset + length < input.len() && is_identifier_continue(input[offset + length]) {
                length += 1;
            }
            if length > MAX_IDENTIFIER_BYTES {
                return lex_failure(&input, names, tokens, 5, start, start_line, start_column);
            }
            let spelling = &input[start..start + length];
            if let Some(keyword) = keyword_kind(spelling) {
                keyword
            } else {
                if let Some(index) = names.iter().position(|record| {
                    input[record.start..record.start + record.length] == *spelling
                }) {
                    name_id = index + 1;
                } else {
                    if names.len() >= MAX_NAMES {
                        return lex_failure(
                            &input,
                            names,
                            tokens,
                            6,
                            start,
                            start_line,
                            start_column,
                        );
                    }
                    names.push(NameRecord { start, length });
                    name_id = names.len();
                }
                1
            }
        } else if byte.is_ascii_digit() {
            while offset + length < input.len() && input[offset + length].is_ascii_digit() {
                length += 1;
            }
            2
        } else if let Some(pair) = input
            .get(offset + 1)
            .and_then(|second| pair_kind(byte, *second))
        {
            length = 2;
            pair
        } else if let Some(single) = single_kind(byte) {
            single
        } else {
            return lex_failure(&input, names, tokens, 7, start, start_line, start_column);
        };

        if tokens.len() >= MAX_REAL_TOKENS {
            return lex_failure(&input, names, tokens, 9, start, start_line, start_column);
        }
        tokens.push(TokenRecord {
            kind,
            start,
            length,
            line: start_line,
            column: start_column,
            name_id,
        });
        for consumed in &input[offset..offset + length] {
            advance(*consumed, &mut line, &mut column);
        }
        offset += length;
    }
    tokens.push(TokenRecord {
        kind: 0,
        start: input.len(),
        length: 0,
        line,
        column,
        name_id: 0,
    });
    FrontendModel {
        input,
        names,
        tokens,
        nodes: Vec::new(),
        origins: Vec::new(),
        diagnostic: FrontendDiagnostic::success(),
        root: 0,
        checksum: 0,
    }
}

fn binary_precedence(kind: i32) -> i32 {
    match kind {
        34 => 1,
        33 => 2,
        26 | 28 => 3,
        29..=32 => 4,
        20 | 21 => 5,
        22..=24 => 6,
        _ => 0,
    }
}

fn binary_node_kind(kind: i32) -> i32 {
    match kind {
        22 => 5,
        23 => 6,
        24 => 7,
        20 => 8,
        21 => 9,
        29 => 10,
        30 => 11,
        31 => 12,
        32 => 13,
        26 => 14,
        28 => 15,
        33 => 16,
        34 => 17,
        _ => 0,
    }
}

struct ReferenceParser<'a> {
    input: &'a [u8],
    names: &'a [NameRecord],
    tokens: &'a [TokenRecord],
    index: usize,
    nodes: Vec<NodeRecord>,
    origins: Vec<OriginRecord>,
}

impl<'a> ReferenceParser<'a> {
    fn current(&self) -> TokenRecord {
        self.tokens[self.index]
    }

    fn failure(token: TokenRecord, status: i32, code: i32) -> FrontendDiagnostic {
        FrontendDiagnostic {
            status,
            offset: i32::try_from(token.start).expect("bounded token offset"),
            line: i32::try_from(token.line).expect("bounded token line"),
            column: i32::try_from(token.column).expect("bounded token column"),
            expected_code: code,
            actual_kind: token.kind,
        }
    }

    fn expect(&mut self, kind: i32) -> Result<TokenRecord, FrontendDiagnostic> {
        let token = self.current();
        if token.kind != kind {
            return Err(Self::failure(token, 10, kind));
        }
        self.index += 1;
        Ok(token)
    }

    fn append_node(
        &mut self,
        kind: i32,
        payload: i32,
        left_id: i32,
        right_id: i32,
        token: TokenRecord,
    ) -> Result<i32, FrontendDiagnostic> {
        if self.nodes.len() >= MAX_NODES {
            return Err(Self::failure(token, 14, 512));
        }
        let node_id = i32::try_from(self.nodes.len() + 1).expect("bounded node id");
        self.origins.push(OriginRecord {
            node_id,
            offset: i32::try_from(token.start).expect("bounded origin offset"),
            line: i32::try_from(token.line).expect("bounded origin line"),
            column: i32::try_from(token.column).expect("bounded origin column"),
            token_kind: token.kind,
        });
        self.nodes.push(NodeRecord {
            kind,
            payload,
            left_id,
            right_id,
        });
        Ok(node_id)
    }

    fn parse_primary(&mut self) -> Result<i32, FrontendDiagnostic> {
        let token = self.current();
        match token.kind {
            1 => {
                self.index += 1;
                self.append_node(
                    2,
                    i32::try_from(token.name_id).expect("bounded name id"),
                    0,
                    0,
                    token,
                )
            }
            2 => {
                let spelling = &self.input[token.start..token.start + token.length];
                let mut value = 0_i32;
                for digit in spelling {
                    value = value
                        .checked_mul(10)
                        .and_then(|current| current.checked_add(i32::from(*digit - b'0')))
                        .ok_or_else(|| Self::failure(token, 13, 103))?;
                }
                self.index += 1;
                self.append_node(1, value, 0, 0, token)
            }
            10 => {
                self.index += 1;
                let value = self.parse_binary(1)?;
                self.expect(11)?;
                Ok(value)
            }
            _ => Err(Self::failure(token, 11, 100)),
        }
    }

    fn parse_unary(&mut self) -> Result<i32, FrontendDiagnostic> {
        let token = self.current();
        if token.kind == 21 || token.kind == 27 {
            self.index += 1;
            let operand = self.parse_unary()?;
            let kind = if token.kind == 21 { 3 } else { 4 };
            self.append_node(kind, 0, operand, 0, token)
        } else {
            self.parse_primary()
        }
    }

    fn parse_binary(&mut self, minimum_precedence: i32) -> Result<i32, FrontendDiagnostic> {
        let mut left = self.parse_unary()?;
        loop {
            let operator = self.current();
            let precedence = binary_precedence(operator.kind);
            if precedence < minimum_precedence {
                break;
            }
            self.index += 1;
            let right = self.parse_binary(precedence + 1)?;
            left = self.append_node(binary_node_kind(operator.kind), 0, left, right, operator)?;
        }
        Ok(left)
    }

    fn parse_program(&mut self) -> Result<i32, FrontendDiagnostic> {
        let function_token = self.expect(3)?;
        let function_name = self.expect(1)?;
        self.expect(10)?;
        self.expect(11)?;
        self.expect(35)?;
        let return_type = self.expect(1)?;
        let name = self.names[return_type.name_id - 1];
        if &self.input[name.start..name.start + name.length] != b"int" {
            return Err(Self::failure(return_type, 12, 102));
        }
        self.expect(12)?;
        let return_token = self.expect(6)?;
        let expression = self.parse_binary(1)?;
        self.expect(18)?;
        self.expect(13)?;
        self.expect(0)?;
        let return_id = self.append_node(18, 0, expression, 0, return_token)?;
        self.append_node(
            19,
            i32::try_from(function_name.name_id).expect("bounded name id"),
            return_id,
            0,
            function_token,
        )
    }
}

fn checksum_step(checksum: i32, word: i32) -> i32 {
    let reduced = word.rem_euclid(1_000_003);
    (checksum * 31 + reduced).rem_euclid(1_000_003)
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
    let diagnostic = model.diagnostic;
    for word in [
        diagnostic.status,
        if diagnostic.offset < 0 {
            0
        } else {
            diagnostic.offset + 1
        },
        diagnostic.line,
        diagnostic.column,
        diagnostic.expected_code,
        diagnostic.actual_kind,
        i32::try_from(model.names.len()).expect("bounded names"),
        i32::try_from(model.tokens.len()).expect("bounded tokens"),
        i32::try_from(model.nodes.len()).expect("bounded nodes"),
        model.root,
    ] {
        checksum = checksum_step(checksum, word);
    }
    checksum
}

fn reference_frontend(source: &[u8]) -> FrontendModel {
    let mut model = reference_lex(source);
    if model.diagnostic.status == 0 {
        let mut parser = ReferenceParser {
            input: &model.input,
            names: &model.names,
            tokens: &model.tokens,
            index: 0,
            nodes: Vec::new(),
            origins: Vec::new(),
        };
        match parser.parse_program() {
            Ok(root) => model.root = root,
            Err(diagnostic) => model.diagnostic = diagnostic,
        }
        model.nodes = parser.nodes;
        model.origins = parser.origins;
    }
    model.checksum = frontend_checksum(&model);
    model
}

fn semantic_failure(
    origin: OriginRecord,
    status: i32,
    code: i32,
    expected_type: i32,
    actual_type: i32,
) -> SemanticDiagnostic {
    SemanticDiagnostic {
        status,
        node_id: origin.node_id,
        offset: origin.offset,
        line: origin.line,
        column: origin.column,
        code,
        expected_type,
        actual_type,
    }
}

fn semantic_checksum(model: &SemanticModel) -> i32 {
    let mut checksum = 17;
    for origin in &model.origins {
        for word in [
            origin.node_id,
            origin.offset,
            origin.line,
            origin.column,
            origin.token_kind,
        ] {
            checksum = checksum_step(checksum, word);
        }
    }
    checksum = checksum_step(checksum, 994);
    for symbol in &model.symbols {
        for word in [
            symbol.kind,
            symbol.name_id,
            symbol.function_node_id,
            symbol.return_type,
        ] {
            checksum = checksum_step(checksum, word);
        }
    }
    checksum = checksum_step(checksum, 995);
    for fact in &model.facts {
        for word in [fact.node_id, fact.logical_type, fact.ownership] {
            checksum = checksum_step(checksum, word);
        }
    }
    checksum = checksum_step(checksum, 996);
    let diagnostic = model.diagnostic;
    for word in [
        diagnostic.status,
        diagnostic.node_id,
        if diagnostic.offset < 0 {
            0
        } else {
            diagnostic.offset + 1
        },
        diagnostic.line,
        diagnostic.column,
        diagnostic.code,
        diagnostic.expected_type,
        diagnostic.actual_type,
        i32::try_from(model.origins.len()).expect("bounded origins"),
        i32::try_from(model.symbols.len()).expect("bounded symbols"),
        i32::try_from(model.facts.len()).expect("bounded facts"),
        model.root_type,
    ] {
        checksum = checksum_step(checksum, word);
    }
    checksum
}

fn reference_semantics(frontend: &FrontendModel) -> SemanticModel {
    let mut model = SemanticModel {
        origins: frontend.origins.clone(),
        symbols: Vec::new(),
        facts: Vec::new(),
        diagnostic: SemanticDiagnostic::success(),
        root_type: 0,
        checksum: 0,
    };
    if frontend.diagnostic.status != 0 {
        model.checksum = semantic_checksum(&model);
        return model;
    }
    let root_index = usize::try_from(frontend.root - 1).expect("positive root");
    let root = frontend.nodes[root_index];
    assert_eq!(root.kind, 19, "frontend root is a function");
    model.symbols.push(SymbolRecord {
        kind: 1,
        name_id: root.payload,
        function_node_id: frontend.root,
        return_type: 1,
    });

    for (index, node) in frontend.nodes.iter().enumerate() {
        if node.kind == 2 {
            let origin = frontend.origins[index];
            model.diagnostic = semantic_failure(origin, 17, 2, 0, 0);
            model.checksum = semantic_checksum(&model);
            return model;
        }
    }

    for (index, node) in frontend.nodes.iter().copied().enumerate() {
        let node_id = i32::try_from(index + 1).expect("bounded node id");
        let origin = frontend.origins[index];
        let child_type = |child_id: i32, facts: &[FactRecord]| -> Option<i32> {
            if child_id <= 0 {
                return None;
            }
            facts
                .get(usize::try_from(child_id - 1).ok()?)
                .filter(|fact| fact.node_id == child_id)
                .map(|fact| fact.logical_type)
        };
        let mut complete_type = 0;
        let mut ownership = 0;
        match node.kind {
            1 => {
                complete_type = 1;
                ownership = 1;
            }
            3 | 4 => {
                let actual = child_type(node.left_id, &model.facts).unwrap_or(0);
                let expected = if node.kind == 3 { 1 } else { 2 };
                if actual != expected {
                    model.diagnostic = semantic_failure(
                        origin,
                        if node.kind == 3 { 24 } else { 23 },
                        node.kind,
                        expected,
                        actual,
                    );
                    break;
                }
                complete_type = expected;
                ownership = 1;
            }
            5 | 6 | 8 | 9 => {
                let left = child_type(node.left_id, &model.facts).unwrap_or(0);
                let right = child_type(node.right_id, &model.facts).unwrap_or(0);
                let actual = if left != 1 { left } else { right };
                if left != 1 || right != 1 {
                    model.diagnostic = semantic_failure(origin, 19, node.kind, 1, actual);
                    break;
                }
                complete_type = 1;
                ownership = 1;
            }
            7 => {
                model.diagnostic = semantic_failure(origin, 18, 7, 0, 0);
                break;
            }
            10..=15 => {
                let left = child_type(node.left_id, &model.facts).unwrap_or(0);
                let right = child_type(node.right_id, &model.facts).unwrap_or(0);
                if left == 0 || right == 0 || left != right {
                    model.diagnostic = semantic_failure(origin, 20, node.kind, left, right);
                    break;
                }
                complete_type = 2;
                ownership = 1;
            }
            16 | 17 => {
                let left = child_type(node.left_id, &model.facts).unwrap_or(0);
                let right = child_type(node.right_id, &model.facts).unwrap_or(0);
                if left != 2 {
                    model.diagnostic = semantic_failure(origin, 21, node.kind, 2, left);
                    break;
                }
                if right != 2 {
                    model.diagnostic = semantic_failure(origin, 22, node.kind, 2, right);
                    break;
                }
                complete_type = 2;
                ownership = 1;
            }
            18 => {
                let actual = child_type(node.left_id, &model.facts).unwrap_or(0);
                if actual != 1 {
                    model.diagnostic = semantic_failure(origin, 25, 18, 1, actual);
                    break;
                }
            }
            19 => {
                if node_id != frontend.root
                    || node.payload <= 0
                    || node.left_id != node_id - 1
                    || child_type(node.left_id, &model.facts) != Some(0)
                {
                    model.diagnostic = semantic_failure(origin, 27, 3, 0, 0);
                    break;
                }
            }
            _ => {
                model.diagnostic = semantic_failure(origin, 27, 2, 0, 0);
                break;
            }
        }
        model.facts.push(FactRecord {
            node_id,
            logical_type: complete_type,
            ownership,
        });
    }
    if model.diagnostic.status == 0 {
        model.root_type = 1;
    }
    model.checksum = semantic_checksum(&model);
    model
}

fn checked_failure(origin: OriginRecord, status: i32, code: i32) -> CheckedDiagnostic {
    CheckedDiagnostic {
        status,
        node_id: origin.node_id,
        offset: origin.offset,
        line: origin.line,
        column: origin.column,
        code,
        expected: 0,
        actual: 0,
    }
}

fn signed_magnitude(value: i32) -> (i32, i32, i32) {
    if value >= 0 {
        return (0, value / 32_768, value % 32_768);
    }
    if value == i32::MIN {
        return (1, 65_536, 0);
    }
    let magnitude = -value;
    (1, magnitude / 32_768, magnitude % 32_768)
}

fn checked_checksum(semantic: &SemanticModel, model: &CheckedModel) -> i32 {
    let mut checksum = checksum_step(23, semantic.checksum);
    checksum = checksum_step(checksum, 997);
    for word in &model.words {
        checksum = checksum_step(checksum, *word);
    }
    checksum = checksum_step(checksum, 998);
    let diagnostic = model.diagnostic;
    for word in [
        diagnostic.status,
        diagnostic.node_id,
        if diagnostic.offset < 0 {
            0
        } else {
            diagnostic.offset + 1
        },
        diagnostic.line,
        diagnostic.column,
        diagnostic.code,
        diagnostic.expected,
        diagnostic.actual,
        model.attempted,
        i32::try_from(model.values.len()).expect("bounded checked values"),
        i32::try_from(model.instructions.len()).expect("bounded checked instructions"),
        i32::try_from(model.results.len()).expect("bounded checked results"),
        i32::try_from(model.words.len()).expect("bounded checked words"),
        model.root_kind,
        model.root_payload,
        model.root_type,
    ] {
        checksum = checksum_step(checksum, word);
    }
    checksum
}

fn reference_checked_ir(frontend: &FrontendModel, semantic: &SemanticModel) -> CheckedModel {
    let mut model = CheckedModel {
        attempted: 0,
        values: Vec::new(),
        instructions: Vec::new(),
        results: Vec::new(),
        words: Vec::new(),
        diagnostic: CheckedDiagnostic::success(),
        root_kind: 0,
        root_payload: 0,
        root_type: 0,
        checksum: 0,
    };
    if frontend.diagnostic.status != 0 || semantic.diagnostic.status != 0 {
        model.checksum = checked_checksum(semantic, &model);
        return model;
    }

    model.attempted = 1;
    assert_eq!(semantic.symbols.len(), 1, "M1A symbol authority");
    assert_eq!(
        semantic.facts.len(),
        frontend.nodes.len(),
        "M1A fact coverage"
    );
    assert_eq!(semantic.origins, frontend.origins, "M1A origin authority");
    assert!(frontend.nodes.len() >= 3, "literal, return, and function");
    let expression_count = frontend.nodes.len() - 2;

    for index in 0..expression_count {
        let node = frontend.nodes[index];
        let node_id = i32::try_from(index + 1).expect("bounded node ID");
        let origin = frontend.origins[index];
        let fact = semantic.facts[index];
        assert_eq!(origin.node_id, node_id);
        assert_eq!(
            fact,
            FactRecord {
                node_id,
                logical_type: 1,
                ownership: 1
            }
        );

        let child = |child_id: i32, values: &[CheckedValueRecord]| -> CheckedValueRecord {
            assert!(child_id > 0 && child_id < node_id, "postorder child");
            let value = values[usize::try_from(child_id - 1).expect("child index")];
            assert_eq!(value.node_id, child_id);
            value
        };

        let (operand_kind, operand_payload, evaluated) = match node.kind {
            1 => {
                assert!(node.payload >= 0);
                (1, node.payload, node.payload)
            }
            3 => {
                let operand = child(node.left_id, &model.values);
                let Some(evaluated) = operand.evaluated.checked_neg() else {
                    model.diagnostic = checked_failure(origin, 1, node.kind);
                    break;
                };
                let instruction_id =
                    i32::try_from(model.instructions.len() + 1).expect("instruction ID");
                let result_id = i32::try_from(model.results.len() + 1).expect("result ID");
                model.instructions.push(CheckedInstructionRecord {
                    instruction_id,
                    opcode: 5,
                    result_id,
                    result_type: 1,
                    left_kind: operand.operand_kind,
                    left_payload: operand.operand_payload,
                    right_kind: 0,
                    right_payload: 0,
                    origin_node_id: node_id,
                });
                model.results.push(CheckedResultRecord {
                    result_id,
                    result_type: 1,
                    definition_instruction_id: instruction_id,
                    origin_node_id: node_id,
                });
                (2, result_id, evaluated)
            }
            5 | 6 | 8 | 9 => {
                let left = child(node.left_id, &model.values);
                let right = child(node.right_id, &model.values);
                if node.kind == 6 && right.evaluated == 0 {
                    model.diagnostic = checked_failure(origin, 2, node.kind);
                    break;
                }
                let evaluated = match node.kind {
                    5 => left.evaluated.checked_mul(right.evaluated),
                    6 => left.evaluated.checked_div(right.evaluated),
                    8 => left.evaluated.checked_add(right.evaluated),
                    9 => left.evaluated.checked_sub(right.evaluated),
                    _ => unreachable!(),
                };
                let Some(evaluated) = evaluated else {
                    model.diagnostic = checked_failure(origin, 1, node.kind);
                    break;
                };
                let instruction_id =
                    i32::try_from(model.instructions.len() + 1).expect("instruction ID");
                let result_id = i32::try_from(model.results.len() + 1).expect("result ID");
                let opcode = match node.kind {
                    5 => 3,
                    6 => 4,
                    8 => 1,
                    9 => 2,
                    _ => unreachable!(),
                };
                model.instructions.push(CheckedInstructionRecord {
                    instruction_id,
                    opcode,
                    result_id,
                    result_type: 1,
                    left_kind: left.operand_kind,
                    left_payload: left.operand_payload,
                    right_kind: right.operand_kind,
                    right_payload: right.operand_payload,
                    origin_node_id: node_id,
                });
                model.results.push(CheckedResultRecord {
                    result_id,
                    result_type: 1,
                    definition_instruction_id: instruction_id,
                    origin_node_id: node_id,
                });
                (2, result_id, evaluated)
            }
            other => panic!("M1A success exposed unsupported M1B node kind {other}"),
        };
        let (sign, magnitude_high, magnitude_low) = signed_magnitude(evaluated);
        model.values.push(CheckedValueRecord {
            node_id,
            operand_kind,
            operand_payload,
            sign,
            magnitude_high,
            magnitude_low,
            evaluated,
        });
    }

    if model.diagnostic.status == 0 {
        assert_eq!(model.values.len(), expression_count);
        let return_id = i32::try_from(expression_count + 1).expect("return node ID");
        let function_id = return_id + 1;
        assert_eq!(function_id, frontend.root);
        assert_eq!(frontend.nodes[expression_count].kind, 18);
        assert_eq!(frontend.nodes[expression_count].left_id, return_id - 1);
        assert_eq!(frontend.nodes[expression_count + 1].kind, 19);
        let root = model.values[expression_count - 1];
        model.root_kind = root.operand_kind;
        model.root_payload = root.operand_payload;
        model.root_type = 1;
        model.instructions.push(CheckedInstructionRecord {
            instruction_id: i32::try_from(model.instructions.len() + 1)
                .expect("return instruction ID"),
            opcode: 6,
            result_id: 0,
            result_type: 0,
            left_kind: root.operand_kind,
            left_payload: root.operand_payload,
            right_kind: 0,
            right_payload: 0,
            origin_node_id: return_id,
        });

        let instruction_count = i32::try_from(model.instructions.len()).expect("instruction count");
        let result_count = i32::try_from(model.results.len()).expect("result count");
        let symbol = semantic.symbols[0];
        model.words.extend([
            1,
            1,
            1,
            instruction_count,
            result_count,
            1,
            model.root_kind,
            model.root_payload,
            model.root_type,
            1,
            1,
            symbol.name_id,
            symbol.function_node_id,
            0,
            1,
            1,
            1,
            instruction_count,
            2,
            1,
            1,
            1,
            0,
            1,
            instruction_count,
        ]);
        for instruction in &model.instructions {
            model.words.extend([
                3,
                instruction.instruction_id,
                instruction.opcode,
                instruction.result_id,
                instruction.result_type,
                instruction.left_kind,
                instruction.left_payload,
                instruction.right_kind,
                instruction.right_payload,
                instruction.origin_node_id,
                1,
            ]);
        }
        for result in &model.results {
            model.words.extend([
                4,
                1,
                result.result_id,
                result.result_type,
                result.definition_instruction_id,
                result.origin_node_id,
            ]);
        }
    }

    model.checksum = checked_checksum(semantic, &model);
    model
}

fn generated_program(
    kernel_prefix: &str,
    frontend: &FrontendModel,
    semantic: &SemanticModel,
    frontend_checksum_delta: i32,
    semantic_checksum_delta: i32,
) -> String {
    format!(
        "{}\n\nfn main() -> int {{\n    return run_runtime_ascii_semantics({}, {}, {}, {}, {}, {},\n        {}, {}, {}, {}, {},\n        {}, {}, {}, {}, {}, {}, {}, {},\n        {}, {}, {}, {}, {});\n}}\n",
        kernel_prefix.trim_end(),
        frontend.diagnostic.status,
        frontend.diagnostic.offset,
        frontend.diagnostic.line,
        frontend.diagnostic.column,
        frontend.diagnostic.expected_code,
        frontend.diagnostic.actual_kind,
        frontend.names.len(),
        frontend.tokens.len(),
        frontend.nodes.len(),
        frontend.root,
        frontend.checksum + frontend_checksum_delta,
        semantic.diagnostic.status,
        semantic.diagnostic.node_id,
        semantic.diagnostic.offset,
        semantic.diagnostic.line,
        semantic.diagnostic.column,
        semantic.diagnostic.code,
        semantic.diagnostic.expected_type,
        semantic.diagnostic.actual_type,
        semantic.origins.len(),
        semantic.symbols.len(),
        semantic.facts.len(),
        semantic.root_type,
        semantic.checksum + semantic_checksum_delta,
    )
}

fn compile_generated(program: &str) -> String {
    check_program(program, options()).expect("generated CAP-043 program checks");
    let llvm = compile_program(program, options()).expect("generated CAP-043 program compiles");
    verify_llvm_module(&llvm, LlvmVerificationMode::Required)
        .expect("generated CAP-043 LLVM verifies");
    llvm
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
    frontend: &FrontendModel,
    fail_after: u64,
) -> AllocationExpectation {
    let mut buffers = [BufferState::default(); 9];
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
    let mut seen_names = vec![false; frontend.names.len() + 1];
    if completed {
        for token in &frontend.tokens {
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

    // Exact parser event order for the allocation fixture `return 1+2;`.
    let parser_events = [
        (6usize, 20usize),
        (3, 16),
        (4, 8),
        (5, 20),
        (6, 20),
        (3, 16),
        (4, 8),
        (6, 20),
        (3, 16),
        (4, 8),
        (6, 20),
        (3, 16),
        (6, 20),
        (3, 16),
    ];
    if completed {
        for (buffer, bytes) in parser_events {
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
    if completed {
        completed = simulate_pushes(
            &mut buffers[7],
            16,
            fail_after,
            &mut successful_events,
            &mut allocations,
            &mut reallocations,
        );
    }
    if completed {
        completed = simulate_pushes(
            &mut buffers[8],
            frontend.nodes.len() * 12,
            fail_after,
            &mut successful_events,
            &mut allocations,
            &mut reallocations,
        );
    }
    AllocationExpectation {
        success: completed,
        allocations,
        reallocations,
        deallocations: buffers.iter().filter(|buffer| buffer.capacity != 0).count() as u64,
    }
}

fn allocation_harness(input: &[u8], frontend: &FrontendModel) -> String {
    let mut cases = String::new();
    for threshold in 0_u64..=48 {
        let expected = allocation_expectation(input, frontend, threshold);
        use std::fmt::Write as _;
        writeln!(
            cases,
            "    {{ UINT64_C({threshold}), {}, UINT64_C({}), UINT64_C({}), UINT64_C({}) }},",
            i32::from(expected.success),
            expected.allocations,
            expected.reallocations,
            expected.deallocations,
        )
        .expect("write CAP-043 allocation case");
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

static void reset_input(void) {{ input_index = 0; sticky_status = 0; }}

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

fn model_source(expression: &str) -> String {
    format!("fn score()->int{{return {expression};}}")
}

fn semantic_for_expression(expression: &str) -> SemanticModel {
    let source = model_source(expression);
    let frontend = reference_frontend(source.as_bytes());
    assert_eq!(
        frontend.diagnostic.status, 0,
        "semantic fixture failed frontend: {source}"
    );
    reference_semantics(&frontend)
}

fn checked_for_expression(expression: &str) -> CheckedModel {
    let source = model_source(expression);
    let frontend = reference_frontend(source.as_bytes());
    assert_eq!(
        frontend.diagnostic.status, 0,
        "checked fixture failed frontend: {source}"
    );
    let semantic = reference_semantics(&frontend);
    reference_checked_ir(&frontend, &semantic)
}

fn accepted_semantic_result(source: &str) -> Result<(), String> {
    let tokens = try_tokenize_with_locations(source, Some("m1a-overlap.aero".to_string()))
        .map_err(|error| error.to_string())?;
    let ast = parse_with_locations(tokens).map_err(|error| error.to_string())?;
    SemanticAnalyzer::new().analyze(ast).map(|_| ())
}

#[test]
fn independent_m1b_oracle_freezes_flat_ir_admission_ssa_and_checksum() {
    let frontend = reference_frontend(b"fn score()->int{return 1+2*3-4/2;}");
    let semantic = reference_semantics(&frontend);
    let checked = reference_checked_ir(&frontend, &semantic);

    assert_eq!(frontend.checksum, 586_661);
    assert_eq!(semantic.checksum, 827_574);
    assert_eq!(checked.attempted, 1);
    assert_eq!(checked.diagnostic, CheckedDiagnostic::success());
    assert_eq!(checked.values.len(), 9);
    assert_eq!(checked.instructions.len(), 5);
    assert_eq!(checked.results.len(), 4);
    assert_eq!(checked.words.len(), 104);
    assert_eq!(
        (checked.root_kind, checked.root_payload, checked.root_type),
        (2, 4, 1)
    );
    assert_eq!(checked.values.last().expect("root value").evaluated, 5);
    assert_eq!(
        checked.instructions,
        vec![
            CheckedInstructionRecord {
                instruction_id: 1,
                opcode: 3,
                result_id: 1,
                result_type: 1,
                left_kind: 1,
                left_payload: 2,
                right_kind: 1,
                right_payload: 3,
                origin_node_id: 4,
            },
            CheckedInstructionRecord {
                instruction_id: 2,
                opcode: 1,
                result_id: 2,
                result_type: 1,
                left_kind: 1,
                left_payload: 1,
                right_kind: 2,
                right_payload: 1,
                origin_node_id: 5,
            },
            CheckedInstructionRecord {
                instruction_id: 3,
                opcode: 4,
                result_id: 3,
                result_type: 1,
                left_kind: 1,
                left_payload: 4,
                right_kind: 1,
                right_payload: 2,
                origin_node_id: 8,
            },
            CheckedInstructionRecord {
                instruction_id: 4,
                opcode: 2,
                result_id: 4,
                result_type: 1,
                left_kind: 2,
                left_payload: 2,
                right_kind: 2,
                right_payload: 3,
                origin_node_id: 9,
            },
            CheckedInstructionRecord {
                instruction_id: 5,
                opcode: 6,
                result_id: 0,
                result_type: 0,
                left_kind: 2,
                left_payload: 4,
                right_kind: 0,
                right_payload: 0,
                origin_node_id: 10,
            },
        ]
    );
    assert_eq!(
        checked.checksum, 355_067,
        "freeze the red-first independent checksum"
    );

    for (expression, expected) in [
        ("2147483647", 2_147_483_647),
        ("-2147483647", -2_147_483_647),
        ("0-2147483647-1", i32::MIN),
        ("20/5+3*4-2", 14),
    ] {
        let model = checked_for_expression(expression);
        assert_eq!(
            model.diagnostic,
            CheckedDiagnostic::success(),
            "{expression}"
        );
        assert_eq!(
            model.values.last().expect("root").evaluated,
            expected,
            "{expression}"
        );
        assert!(!model.words.is_empty(), "{expression}");
    }

    for expression in [
        "2147483647+1",
        "(0-2147483647-1)-1",
        "-(0-2147483647-1)",
        "(0-2147483647-1)/(0-1)",
        "50000*50000",
    ] {
        let model = checked_for_expression(expression);
        assert_eq!(model.attempted, 1, "{expression}");
        assert_eq!(model.diagnostic.status, 1, "{expression}");
        assert!(model.words.is_empty(), "{expression}");
        assert_eq!(
            (model.root_kind, model.root_payload, model.root_type),
            (0, 0, 0)
        );
    }

    for expression in ["4/0", "4/(2-2)", "4/(-(2-2))"] {
        let model = checked_for_expression(expression);
        assert_eq!(model.attempted, 1, "{expression}");
        assert_eq!(model.diagnostic.status, 2, "{expression}");
        assert_eq!(model.diagnostic.code, 6, "{expression}");
        assert!(model.words.is_empty(), "{expression}");
    }

    let earlier_failure = checked_for_expression("1 < 2");
    assert_eq!(earlier_failure.attempted, 0);
    assert_eq!(earlier_failure.diagnostic, CheckedDiagnostic::success());
    assert!(earlier_failure.values.is_empty());
    assert!(earlier_failure.words.is_empty());
}

#[test]
fn accepted_rust_checked_ir_projection_overlaps_without_supplying_m1b_records() {
    for (source, expected) in [
        ("fn score()->int{return 7;}", 7),
        ("fn score()->int{return 1+2*3-4/2;}", 5),
        ("fn score()->int{return -5;}", -5),
        ("fn score()->int{return 100/5;}", 20),
    ] {
        let checked = prepare_checked_program_for_compiler_service(source, None, None)
            .unwrap_or_else(|error| {
                panic!("accepted Rust checked IR rejected `{source}`: {error}")
            });
        let debug = format!("{checked:?}");
        assert!(
            debug.contains(&format!("Return(ImmInt({expected}))")),
            "{debug}"
        );
        assert!(debug.contains("result: Int"), "{debug}");
        assert!(debug.contains("label: \"entry\""), "{debug}");
        assert!(debug.contains("successors: []"), "{debug}");
    }

    let division = prepare_checked_program_for_compiler_service(
        "fn score()->int{return 4/(2-2);}",
        None,
        None,
    )
    .expect_err("accepted checked admission must reject derived zero division");
    assert_eq!(
        division,
        "IR Generation Error: constant integer division by zero"
    );

    let overflow_source = "fn score()->int{return 2147483647+1;}";
    let accepted = prepare_checked_program_for_compiler_service(overflow_source, None, None)
        .expect("Rust checked IR retains the wider dynamic operation");
    assert!(format!("{accepted:?}").contains("Add(Reg(0), ImmInt(2147483647), ImmInt(1))"));
    assert_eq!(checked_for_expression("2147483647+1").diagnostic.status, 1);
}

#[test]
fn independent_oracle_freezes_origins_types_ownership_diagnostics_and_phase_order() {
    let accepted_f1b = reference_frontend(b"fn score()->int{return 1+2*3-4/2%2;}");
    assert_eq!(accepted_f1b.diagnostic.status, 0);
    assert_eq!(accepted_f1b.names.len(), 2);
    assert_eq!(accepted_f1b.tokens.len(), 22);
    assert_eq!(accepted_f1b.nodes.len(), 13);
    assert_eq!(accepted_f1b.origins.len(), 13);
    assert_eq!(accepted_f1b.root, 13);
    assert_eq!(accepted_f1b.checksum, 846_139);

    let source = b"fn score()->int{return 1+2*3-4/2;}";
    let frontend = reference_frontend(source);
    let semantic = reference_semantics(&frontend);
    assert_eq!(frontend.diagnostic, FrontendDiagnostic::success());
    assert_eq!(frontend.names.len(), 2);
    assert_eq!(frontend.tokens.len(), 20);
    assert_eq!(frontend.nodes.len(), 11);
    assert_eq!(frontend.origins.len(), 11);
    assert_eq!(frontend.root, 11);
    assert_eq!(semantic.diagnostic, SemanticDiagnostic::success());
    assert_eq!(semantic.root_type, 1);
    assert_eq!(semantic.symbols.len(), 1);
    assert_eq!(semantic.facts.len(), 11);
    assert_eq!(semantic.symbols[0].function_node_id, 11);
    assert_eq!(semantic.symbols[0].return_type, 1);
    assert_eq!(semantic.facts.last().map(|fact| fact.logical_type), Some(0));
    assert_eq!(semantic.facts.last().map(|fact| fact.ownership), Some(0));
    assert_eq!(semantic.origins[0].token_kind, 2);
    assert_eq!(semantic.origins[9].token_kind, 6);
    assert_eq!(semantic.origins[10].token_kind, 3);
    assert_eq!(frontend.checksum, 586_661);
    assert_eq!(semantic.checksum, 827_574);

    for (expression, status, code, expected, actual) in [
        ("1+(2<3)", 19, 8, 1, 2),
        ("1<(2<3)", 20, 10, 1, 2),
        ("1&&2", 21, 16, 2, 1),
        ("(1<2)&&3", 22, 16, 2, 1),
        ("!1", 23, 4, 2, 1),
        ("-(1<2)", 24, 3, 1, 2),
        ("1<2", 25, 18, 1, 2),
        ("1%2", 18, 7, 0, 0),
    ] {
        let actual_model = semantic_for_expression(expression);
        assert_eq!(actual_model.diagnostic.status, status, "{expression}");
        assert_eq!(actual_model.diagnostic.code, code, "{expression}");
        assert_eq!(
            actual_model.diagnostic.expected_type, expected,
            "{expression}"
        );
        assert_eq!(actual_model.diagnostic.actual_type, actual, "{expression}");
        assert_eq!(actual_model.root_type, 0, "{expression}");
    }

    let name_first = semantic_for_expression("1+(2<3)+missing");
    assert_eq!(name_first.diagnostic.status, 17);
    assert_eq!(name_first.diagnostic.code, 2);
    assert!(name_first.facts.is_empty());
    assert_eq!(name_first.symbols.len(), 1);

    let repeated = semantic_for_expression("first+second+first");
    assert_eq!(repeated.diagnostic.status, 17);
    let first_name = &reference_frontend(model_source("first+second+first").as_bytes()).names[2];
    assert_eq!(
        repeated.diagnostic.offset,
        i32::try_from(first_name.start).unwrap()
    );

    for expression in ["1", "-1", "1+2*3-4/2"] {
        let model = semantic_for_expression(expression);
        assert_eq!(model.diagnostic.status, 0, "{expression}");
        assert_eq!(model.root_type, 1, "{expression}");
        assert_eq!(model.facts.len(), model.origins.len(), "{expression}");
    }
    for expression in ["!(1<2)", "(1<2)&&(3==4)", "(1<2)<(3<4)"] {
        let model = semantic_for_expression(expression);
        assert_eq!(model.diagnostic.status, 25, "{expression}");
        assert_eq!(model.diagnostic.actual_type, 2, "{expression}");
        assert_eq!(model.facts.last().map(|fact| fact.logical_type), Some(2));
    }
}

#[test]
fn accepted_rust_semantics_overlap_without_supplying_m1a_facts() {
    for expression in ["1", "-1", "1+2*3-4/2"] {
        let source = model_source(expression);
        accepted_semantic_result(&source)
            .unwrap_or_else(|error| panic!("accepted semantic overlap `{expression}`: {error}"));
    }
    for expression in ["!(1<2)", "(1<2)<(3<4)"] {
        let error = accepted_semantic_result(&model_source(expression))
            .expect_err("Bool expression must reach the frozen Int return mismatch");
        assert!(error.contains("return type mismatch: expected int, actual bool"));
    }

    let name_precedence = accepted_semantic_result(&model_source("1+(2<3)+missing"))
        .expect_err("undeclared name must precede type mismatch");
    assert!(name_precedence.contains("Use of undeclared variable `missing`"));
    let modulo = accepted_semantic_result(&model_source("1%2"))
        .expect_err("accepted semantics reject modulo");
    assert_eq!(modulo, "Binary operator `%` is not supported.");
    let arithmetic = accepted_semantic_result(&model_source("1+(2<3)"))
        .expect_err("accepted semantics reject Int plus Bool");
    assert!(arithmetic.contains("Type mismatch in arithmetic operation `+`: int vs bool"));
    let returned_bool = accepted_semantic_result(&model_source("1<2"))
        .expect_err("accepted semantics reject Bool from Int function");
    assert!(returned_bool.contains("return type mismatch: expected int, actual bool"));
}

#[test]
fn accepted_f1b_product_remains_deterministic_and_byte_identical_to_its_oracle() {
    assert_eq!(PROFILE_NAME, "exact-i32-byte-input-v0");
    let product = fs::read_to_string(repository_path(PARSER_RELATIVE_PATH))
        .expect("read accepted CAP-042 parser product");
    assert_eq!(product.matches("fn run_runtime_ascii_parser(").count(), 1);
    assert!(product.contains("2, 22, 13, 13, 846139"));
    assert!(!product.contains("fn run_runtime_ascii_semantics("));
    check_program(&product, options()).expect("accepted CAP-042 product checks");
    let first = compile_program(&product, options()).expect("accepted CAP-042 product compiles");
    let second = compile_program(&product, options()).expect("accepted CAP-042 product recompiles");
    assert_eq!(
        first, second,
        "accepted CAP-042 LLVM changed or is nondeterministic"
    );
}

#[test]
fn tracked_runtime_ascii_semantics_is_the_only_intentional_red() {
    let product = fs::read_to_string(repository_path(PRODUCT_RELATIVE_PATH))
        .unwrap_or_else(|_| panic!("{INTENTIONAL_PRODUCT_RED}"));
    let (kernel_prefix, tracked_main) = product
        .split_once(SELF_TEST_MARKER)
        .expect("tracked product retains one semantic/self-test boundary");
    assert_eq!(product.matches(SELF_TEST_MARKER).count(), 1);
    assert_eq!(
        product.matches("fn run_runtime_ascii_semantics(").count(),
        1
    );
    for owner in [
        "source",
        "names",
        "tokens",
        "nodes",
        "values",
        "operators",
        "origins",
        "symbols",
        "facts",
    ] {
        assert!(
            product.contains(&format!("let mut {owner}: ByteBuffer = bytes_new();")),
            "tracked CAP-043 product omitted `{owner}`"
        );
    }
    assert!(tracked_main.contains("2, 20, 11, 11, 586661"));
    assert!(tracked_main.contains("11, 1, 11, 1, 827574"));
    for anchor in [
        "parser_append_target = 4",
        "origin_count = origin_count + 1",
        "semantic_status = 17",
        "semantic_status = 18",
        "semantic_status = 19",
        "semantic_status = 20",
        "semantic_status = 21",
        "semantic_status = 22",
        "semantic_status = 23",
        "semantic_status = 24",
        "semantic_status = 25",
        "semantic_status = 26",
        "semantic_status = 27",
        "bytes_len(&origins) != origin_count * 20",
        "bytes_len(&symbols) != 16",
        "bytes_len(&facts) != fact_count * 12",
    ] {
        assert!(
            product.contains(anchor),
            "tracked CAP-043 product omitted `{anchor}`"
        );
    }
    for forbidden in [
        "String",
        "Vec",
        "HashMap",
        "unsafe",
        "fn run_runtime_ascii_parser(",
    ] {
        assert!(
            !product.contains(forbidden),
            "tracked CAP-043 product contains `{forbidden}`"
        );
    }

    check_program(&product, options()).expect("tracked CAP-043 product checks");
    let first = compile_program(&product, options()).expect("tracked CAP-043 product compiles");
    let second = compile_program(&product, options()).expect("tracked CAP-043 product recompiles");
    assert_eq!(first, second, "tracked CAP-043 LLVM is nondeterministic");
    verify_llvm_module(&first, LlvmVerificationMode::Required)
        .expect("tracked CAP-043 LLVM verifies");
    for anchor in [
        "%aero.byte_buffer = type { ptr, i32, i32 }",
        "declare ptr @aero_alloc(i64)",
        "declare ptr @aero_realloc(ptr, i64, i64)",
        "declare void @aero_dealloc(ptr, i64)",
        "declare i32 @aero_stdin_read_byte()",
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
    let source_path = workspace.write("runtime_ascii_semantics.aero", &product);
    check_file(&source_path, options()).expect("tracked CAP-043 file checks");
    assert_eq!(
        compile_file(&source_path, options()).expect("tracked CAP-043 file compiles"),
        first,
        "tracked CAP-043 source/file LLVM diverged"
    );
    let canonical = b"fn score()->int{return 1+2*3-4/2;}";
    let frontend = reference_frontend(canonical);
    let semantic = reference_semantics(&frontend);
    assert_eq!(frontend.checksum, 586_661);
    assert_eq!(semantic.checksum, 827_574);
    let llvm_path = workspace.write("tracked.ll", &first);
    let runtime = repository_path(RUNTIME_RELATIVE_PATH);
    for optimization in ["-O0", "-O2"] {
        let executable = clang_link(
            "tracked",
            &workspace,
            &[llvm_path.as_path(), runtime.as_path()],
            optimization,
        );
        assert_silent_exit_91(
            &run_command_with_stdin(&mut Command::new(executable), canonical),
            &format!("tracked CAP-043 {optimization}"),
        );
    }

    let mut public = Command::new(env!("CARGO_BIN_EXE_aero"));
    public
        .args([
            "run",
            source_path.to_str().expect("public source path is UTF-8"),
            "--language-profile",
            PROFILE_NAME,
        ])
        .current_dir(&workspace.root);
    let public_output = run_command_with_stdin(&mut public, canonical);
    assert_eq!(public_output.status.code(), Some(91));
    let public_stdout = String::from_utf8_lossy(&public_output.stdout);
    assert_eq!(
        public_stdout
            .lines()
            .filter(|line| *line == "Exit code: 91")
            .count(),
        1
    );
    assert!(
        !public_stdout
            .lines()
            .any(|line| line.starts_with("Output:") || line.starts_with("Error output:"))
    );
    assert!(public_output.stderr.is_empty());

    for target in ["rocm", "cuda"] {
        let output_path = workspace.root.join(format!("{target}.ll"));
        let output = Command::new(env!("CARGO_BIN_EXE_aero"))
            .args([
                "build",
                source_path
                    .to_str()
                    .expect("accelerator source path is UTF-8"),
                "-o",
                output_path
                    .to_str()
                    .expect("accelerator output path is UTF-8"),
                "--target",
                target,
                "--language-profile",
                PROFILE_NAME,
            ])
            .current_dir(&workspace.root)
            .output()
            .expect("execute CAP-043 accelerator rejection");
        assert_eq!(output.status.code(), Some(2));
        assert!(
            !output_path.exists(),
            "{target} rejection created an artifact"
        );
    }

    assert_eq!(
        kernel_prefix.matches("let mut origins: ByteBuffer").count(),
        1
    );
}

#[test]
fn tracked_runtime_ascii_checked_ir_is_the_only_intentional_red() {
    let product = fs::read_to_string(repository_path(CHECKED_PRODUCT_RELATIVE_PATH))
        .unwrap_or_else(|_| panic!("{INTENTIONAL_CHECKED_PRODUCT_RED}"));
    assert_eq!(product.matches(CHECKED_SELF_TEST_MARKER).count(), 1);
}

#[test]
fn tracked_product_executes_every_semantic_status_from_independent_expectations() {
    let product = fs::read_to_string(repository_path(PRODUCT_RELATIVE_PATH))
        .expect("read tracked CAP-043 product");
    let (kernel_prefix, _) = product
        .split_once(SELF_TEST_MARKER)
        .expect("tracked CAP-043 self-test boundary");
    let runtime = repository_path(RUNTIME_RELATIVE_PATH);
    let cases = [
        ("success", "1+2*3-4/2"),
        ("undeclared", "1+(2<3)+missing"),
        ("modulo", "1%2"),
        ("arithmetic", "1+(2<3)"),
        ("comparison", "1<(2<3)"),
        ("logical-left", "1&&2"),
        ("logical-right", "(1<2)&&3"),
        ("logical-not", "!1"),
        ("negation", "-(1<2)"),
        ("return", "(1<2)<(3<4)"),
    ];
    for (label, expression) in cases {
        let input = model_source(expression).into_bytes();
        let frontend = reference_frontend(&input);
        let semantic = reference_semantics(&frontend);
        let program = generated_program(kernel_prefix, &frontend, &semantic, 0, 0);
        let llvm = compile_generated(&program);
        let workspace = TestWorkspace::new(label);
        let llvm_path = workspace.write("case.ll", llvm);
        let executable = clang_link(
            label,
            &workspace,
            &[llvm_path.as_path(), runtime.as_path()],
            "-O0",
        );
        assert_silent_exit_91(
            &run_command_with_stdin(&mut Command::new(executable), &input),
            &format!("CAP-043 semantic case {label}"),
        );
    }
}

#[test]
fn nine_owner_failures_and_semantic_mutations_never_leak_or_return_success() {
    let product = fs::read_to_string(repository_path(PRODUCT_RELATIVE_PATH))
        .expect("read tracked CAP-043 product");
    let (kernel_prefix, _) = product
        .split_once(SELF_TEST_MARKER)
        .expect("tracked CAP-043 self-test boundary");
    let input = model_source("1+2").into_bytes();
    let frontend = reference_frontend(&input);
    let semantic = reference_semantics(&frontend);
    assert_eq!(
        frontend.nodes.len(),
        5,
        "allocation fixture topology changed"
    );
    assert_eq!(semantic.facts.len(), 5, "allocation fixture facts changed");
    let program = generated_program(kernel_prefix, &frontend, &semantic, 0, 0);
    let llvm = compile_generated(&program);
    let renamed = llvm.replacen("define i32 @main()", "define i32 @aero_program_main()", 1);
    assert_ne!(renamed, llvm, "allocation product omitted main");
    let workspace = TestWorkspace::new("allocation");
    let llvm_path = workspace.write("program.ll", renamed);
    let harness_path = workspace.write("harness.c", allocation_harness(&input, &frontend));
    let test_runtime = repository_path(TEST_RUNTIME_RELATIVE_PATH);
    let executable = clang_link(
        "allocation",
        &workspace,
        &[
            llvm_path.as_path(),
            harness_path.as_path(),
            test_runtime.as_path(),
        ],
        "-O2",
    );
    assert_silent_exit_91(
        &Command::new(executable)
            .output()
            .expect("run CAP-043 allocation harness"),
        "CAP-043 nine-owner allocation harness",
    );

    let runtime = repository_path(RUNTIME_RELATIVE_PATH);
    let canonical = b"fn score()->int{return 1+2*3-4/2;}".to_vec();
    let canonical_frontend = reference_frontend(&canonical);
    let canonical_semantic = reference_semantics(&canonical_frontend);
    let wrong_checksum = generated_program(
        kernel_prefix,
        &canonical_frontend,
        &canonical_semantic,
        0,
        1,
    );
    let mutations = [
        ("semantic-checksum", wrong_checksum),
        (
            "origin",
            generated_program(
                &kernel_prefix.replace(
                    "parser_append_0 = node_count + 1;",
                    "parser_append_0 = node_count + 2;",
                ),
                &canonical_frontend,
                &canonical_semantic,
                0,
                0,
            ),
        ),
        (
            "symbol",
            generated_program(
                &kernel_prefix.replacen(
                    "semantic_append_word = function_payload;",
                    "semantic_append_word = function_payload + 1;",
                    1,
                ),
                &canonical_frontend,
                &canonical_semantic,
                0,
                0,
            ),
        ),
        (
            "fact",
            generated_program(
                &kernel_prefix.replacen(
                    "semantic_append_word = semantic_complete_type;",
                    "semantic_append_word = semantic_complete_type + 1;",
                    1,
                ),
                &canonical_frontend,
                &canonical_semantic,
                0,
                0,
            ),
        ),
    ];
    for (label, mutated) in mutations {
        let llvm = compile_generated(&mutated);
        let mutation = TestWorkspace::new(label);
        let llvm_path = mutation.write("mutation.ll", llvm);
        let executable = clang_link(
            label,
            &mutation,
            &[llvm_path.as_path(), runtime.as_path()],
            "-O2",
        );
        assert_ne!(
            run_command_with_stdin(&mut Command::new(executable), &canonical)
                .status
                .code(),
            Some(91),
            "CAP-043 {label} mutation returned success"
        );
    }
}

#[test]
fn protected_linux_and_windows_semantic_replays_are_frozen() {
    let workflow =
        fs::read_to_string(repository_path(WORKFLOW_RELATIVE_PATH)).expect("read Rust workflow");
    let linux = workflow_step(&workflow, "Test runtime ASCII semantics at O0 and O2");
    for anchor in [
        "runtime_ascii_semantics.aero",
        "fn score()->int{return 1+2*3-4/2;}",
        "runtime_ascii_semantics.linux.repeat.ll",
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
            "Linux CAP-043 step omitted `{anchor}`"
        );
    }
    let windows = workflow_step(
        &workflow,
        "Test runtime ASCII semantics on Windows at O0 and O2",
    );
    for anchor in [
        "runtime_ascii_semantics.aero",
        "fn score()->int{return 1+2*3-4/2;}",
        "runtime_ascii_semantics.windows.repeat.ll",
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
            "Windows CAP-043 step omitted `{anchor}`"
        );
    }
}
