use compiler::{
    CompilerOptions, LanguageProfile, SemanticAnalyzer, check_program, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::PathBuf;

const MAX_INPUT_BYTES: usize = 8_192;
const MAX_REAL_TOKENS: usize = 1_024;
const MAX_NAMES: usize = 1_024;
const MAX_IDENTIFIER_BYTES: usize = 63;
const MAX_NODES: usize = 512;
const PROFILE_NAME: &str = "exact-i32-byte-input-v0";
const PARSER_RELATIVE_PATH: &str = "../../examples/aero_frontend_v0/runtime_ascii_parser.aero";
const PRODUCT_RELATIVE_PATH: &str = "../../examples/aero_frontend_v0/runtime_ascii_semantics.aero";
const SELF_TEST_MARKER: &str = "// CAP-043 TRACKED SELF-TEST";
const INTENTIONAL_PRODUCT_RED: &str =
    "CAP-043 intentional product red: tracked runtime ASCII semantic facts are absent";

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

fn repository_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn options() -> CompilerOptions {
    CompilerOptions {
        language_profile: LanguageProfile::ExactI32ByteInputV0,
        ..CompilerOptions::default()
    }
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

fn accepted_semantic_result(source: &str) -> Result<(), String> {
    let tokens = try_tokenize_with_locations(source, Some("m1a-overlap.aero".to_string()))
        .map_err(|error| error.to_string())?;
    let ast = parse_with_locations(tokens).map_err(|error| error.to_string())?;
    SemanticAnalyzer::new().analyze(ast).map(|_| ())
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
    check_program(&product, options()).expect("tracked CAP-043 product checks");
}
