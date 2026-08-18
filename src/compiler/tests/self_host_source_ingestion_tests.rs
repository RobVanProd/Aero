//! CAP-049 / H1A - canonical self-host source ingestion.
//!
//! The accepted CAP-047/B1C compiler cannot read its own source: its stdin
//! ingestion loop stops at byte 8,192. This target freezes the first H1
//! prerequisite - the new canonical source
//! `examples/aero_self_host_v0/compiler.aero` must consume its own complete
//! byte, name, and token streams and then stop at one independently predicted
//! unsupported parser construct.
//!
//! Every expectation here is derived by the independent oracle in [`oracle`],
//! never by observing the Aero product.

use compiler::{
    CompilerOptions, LanguageProfile, LlvmVerificationMode, check_file, check_program,
    compile_file, compile_program, verify_llvm_module,
};
use std::fmt::Write as _;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const ACCEPTED_B1C_PRODUCT: &str =
    "../../examples/aero_frontend_v0/runtime_ascii_toolchain_driver.aero";
const H1A_PRODUCT: &str = "../../examples/aero_self_host_v0/compiler.aero";
const WORKFLOW: &str = "../../.github/workflows/rust.yml";
const INTENTIONAL_PRODUCT_RED: &str =
    "CAP-049 intentional product red: canonical self-host compiler source is absent";

const ACCEPTED_B1C_SOURCE_LENGTH: usize = 241_941;
const ACCEPTED_B1C_SOURCE_MD5: &str = "08a2fd5ec8c0093b56e05c2ae5608371";

/// The accepted B1C ingestion bound, and therefore the exact first self-input
/// failure of the accepted product.
const ACCEPTED_SOURCE_BOUND: usize = 8_192;
/// The three frozen CAP-049 ingestion bounds.
const H1A_SOURCE_BOUND: usize = 1_048_576;
const H1A_TOKEN_BOUND: usize = 262_144;
const H1A_NAME_BOUND: usize = 16_384;

const CANONICAL_LLVM: &str = concat!(
    "define i32 @aero_b1_entry() {\n",
    "entry:\n",
    "  %r1 = mul i32 2, 3\n",
    "  %r2 = add i32 1, %r1\n",
    "  %r3 = sdiv i32 4, 2\n",
    "  %r4 = sub i32 %r2, %r3\n",
    "  ret i32 %r4\n",
    "}\n",
);
const CANONICAL_INPUT: &[u8] = b"fn score()->int{return 1+2*3-4/2;}";

static NEXT_WORKSPACE_ID: AtomicU64 = AtomicU64::new(0);
static H1A_LLVM: OnceLock<String> = OnceLock::new();
static B1C_LLVM: OnceLock<String> = OnceLock::new();

// ---------------------------------------------------------------------------
// Independent oracle
// ---------------------------------------------------------------------------

/// A byte-for-byte independent model of the accepted runtime-ASCII lexer and of
/// the frozen parser skeleton, written from the frozen contract rather than from
/// observed Aero output.
mod oracle {
    /// The exact fold the Aero product uses for every checksum.
    pub fn checksum_step(checksum: i32, word: i32) -> i32 {
        i32::try_from((i64::from(checksum) * 31 + i64::from(word)) % 1_000_003)
            .expect("bounded CAP-049 checksum")
    }

    pub fn is_identifier_start(value: u8) -> bool {
        value == b'_' || value.is_ascii_uppercase() || value.is_ascii_lowercase()
    }

    pub fn is_identifier_continue(value: u8) -> bool {
        is_identifier_start(value) || value.is_ascii_digit()
    }

    /// `keyword_token_kind`: identifiers fall through to kind 1.
    pub fn keyword_token_kind(word: &[u8]) -> i32 {
        match word {
            b"fn" => 3,
            b"if" => 7,
            b"let" => 4,
            b"mut" => 5,
            b"else" => 8,
            b"while" => 9,
            b"return" => 6,
            _ => 1,
        }
    }

    /// `pair_token_kind`.
    pub fn pair_token_kind(first: u8, second: u8) -> i32 {
        match (first, second) {
            (b'=', b'=') => 26,
            (b'!', b'=') => 28,
            (b'<', b'=') => 30,
            (b'>', b'=') => 32,
            (b'&', b'&') => 33,
            (b'|', b'|') => 34,
            (b'-', b'>') => 35,
            (b'=', b'>') => 36,
            _ => 0,
        }
    }

    /// `single_token_kind`. `ampersand` selects the accepted B1C table (`false`,
    /// where a lone `&` has no kind) or the CAP-049 table (`true`, kind 37).
    pub fn single_token_kind(value: u8, ampersand: bool) -> i32 {
        match value {
            b'(' => 10,
            b')' => 11,
            b'{' => 12,
            b'}' => 13,
            b'[' => 14,
            b']' => 15,
            b',' => 16,
            b':' => 17,
            b';' => 18,
            b'.' => 19,
            b'+' => 20,
            b'-' => 21,
            b'*' => 22,
            b'/' => 23,
            b'%' => 24,
            b'=' => 25,
            b'!' => 27,
            b'<' => 29,
            b'>' => 31,
            b'&' if ampersand => 37,
            _ => 0,
        }
    }

    /// `binary_precedence`.
    pub fn binary_precedence(kind: i32) -> i32 {
        match kind {
            34 => 1,
            33 => 2,
            26 | 28 => 3,
            29 | 30 | 31 | 32 => 4,
            20 | 21 => 5,
            22 | 23 | 24 => 6,
            _ => 0,
        }
    }

    /// `binary_node_kind`.
    pub fn binary_node_kind(kind: i32) -> i32 {
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

    /// One located token record: kind, start, length, line, column, name id.
    pub type TokenRecord = [i32; 6];

    #[derive(Debug, Clone)]
    pub struct Ingestion {
        pub source_length: i32,
        /// Exactly how many stdin bytes the product consumes before it stops
        /// reading. A bounded stop still consumes the byte it rejects.
        pub consumed: i32,
        pub status: i32,
        pub error_offset: i32,
        pub error_line: i32,
        pub error_column: i32,
        pub diagnostic_code: i32,
        pub diagnostic_actual: i32,
        pub names: Vec<(i32, i32)>,
        pub tokens: Vec<TokenRecord>,
        /// Whether the product folds a CAP-050 parameter region into its parse
        /// checksum. The store-only bisection records zero parameters.
        pub signature_grammar: bool,
        /// Every admitted CAP-050 parameter as `(name id, type code)`, in
        /// declaration order. Type code 1 is `int` and 2 is `Result<int, int>`.
        pub parameters: Vec<(i32, i32)>,
        /// Every syntax node as `[kind, payload, left, right]`, in append order.
        pub nodes: Vec<[i32; 4]>,
        /// The parallel origin sidecar the parser appends beside every node, as
        /// `[node id, start, line, column, token kind]`. It is a parse-phase
        /// arena, but `origin_count` is compared and folded with the semantic
        /// group, so a stopped parse that produced a node still reports it.
        pub origins: Vec<[i32; 5]>,
    }

    pub struct Bounds {
        pub source: usize,
        pub token: usize,
        pub name: usize,
        pub ampersand: bool,
    }

    /// Model the stdin ingestion loop and the lexer. Returns the state the Aero
    /// product reaches before its parser runs.
    pub fn ingest(source: &[u8], bounds: &Bounds) -> Ingestion {
        // Stage 1: the bounded stdin ingestion loop.
        let mut line = 1i32;
        let mut column = 1i32;
        for (index, byte) in source.iter().enumerate() {
            if index >= bounds.source {
                return Ingestion {
                    source_length: i32::try_from(bounds.source).expect("bounded source"),
                    consumed: i32::try_from(index + 1).expect("bounded consumption"),
                    status: 2,
                    error_offset: i32::try_from(bounds.source).expect("bounded source"),
                    error_line: line,
                    error_column: column,
                    diagnostic_code: 0,
                    diagnostic_actual: 0,
                    names: Vec::new(),
                    tokens: Vec::new(),
                    signature_grammar: false,
                    parameters: Vec::new(),
                    nodes: Vec::new(),
                    origins: Vec::new(),
                };
            }
            if *byte == b'\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }
        let source_length = source.len();

        // Stage 2: the lexer.
        let mut offset = 0usize;
        let mut line = 1i32;
        let mut column = 1i32;
        let mut tokens: Vec<TokenRecord> = Vec::new();
        let mut names: Vec<(i32, i32)> = Vec::new();
        let mut interned: std::collections::HashMap<Vec<u8>, i32> =
            std::collections::HashMap::new();

        macro_rules! stop {
            ($status:expr, $start:expr, $line:expr, $column:expr) => {
                return Ingestion {
                    source_length: i32::try_from(source_length).expect("bounded source"),
                    consumed: i32::try_from(source_length).expect("bounded consumption"),
                    status: $status,
                    error_offset: i32::try_from($start).expect("bounded offset"),
                    error_line: $line,
                    error_column: $column,
                    diagnostic_code: 0,
                    diagnostic_actual: 0,
                    names,
                    tokens,
                    signature_grammar: false,
                    parameters: Vec::new(),
                    nodes: Vec::new(),
                    origins: Vec::new(),
                }
            };
        }

        while offset < source_length {
            let byte = source[offset];
            if byte > 127 {
                stop!(3, offset, line, column);
            }
            if matches!(byte, b' ' | b'\t' | b'\r' | b'\n') {
                if byte == b'\n' {
                    line += 1;
                    column = 1;
                } else {
                    column += 1;
                }
                offset += 1;
                continue;
            }
            let next = if offset + 1 < source_length {
                i32::from(source[offset + 1])
            } else {
                -1
            };

            if byte == b'/' && next == i32::from(b'/') {
                offset += 2;
                column += 2;
                while offset < source_length {
                    let inner = source[offset];
                    if inner > 127 {
                        stop!(3, offset, line, column);
                    }
                    if inner == b'\r' || inner == b'\n' {
                        break;
                    }
                    offset += 1;
                    column += 1;
                }
                continue;
            }
            if byte == b'/' && next == i32::from(b'*') {
                let (open_at, open_line, open_column) = (offset, line, column);
                offset += 2;
                column += 2;
                let mut closed = false;
                while offset < source_length && !closed {
                    let inner = source[offset];
                    if inner > 127 {
                        stop!(3, offset, line, column);
                    }
                    let following = if offset + 1 < source_length {
                        i32::from(source[offset + 1])
                    } else {
                        -1
                    };
                    if inner == b'*' && following == i32::from(b'/') {
                        offset += 2;
                        column += 2;
                        closed = true;
                    } else {
                        if inner == b'\n' {
                            line += 1;
                            column = 1;
                        } else {
                            column += 1;
                        }
                        offset += 1;
                    }
                }
                if !closed {
                    stop!(4, open_at, open_line, open_column);
                }
                continue;
            }

            let token_start = offset;
            let token_line = line;
            let token_column = column;
            let mut token_end = offset + 1;
            let kind;
            let mut name_id = 0i32;

            if is_identifier_start(byte) {
                while token_end < source_length && is_identifier_continue(source[token_end]) {
                    token_end += 1;
                }
                let token_length = token_end - token_start;
                if token_length > 63 {
                    stop!(5, token_start, token_line, token_column);
                }
                let word = &source[token_start..token_end];
                kind = keyword_token_kind(word);
                if kind == 1 {
                    let found = interned.get(word).copied().unwrap_or(0);
                    if found == 0 && names.len() >= bounds.name {
                        stop!(7, token_start, token_line, token_column);
                    }
                    if tokens.len() >= bounds.token {
                        stop!(6, token_start, token_line, token_column);
                    }
                    name_id = if found == 0 {
                        names.push((
                            i32::try_from(token_start).expect("bounded name start"),
                            i32::try_from(token_length).expect("bounded name length"),
                        ));
                        let assigned = i32::try_from(names.len()).expect("bounded name count");
                        interned.insert(word.to_vec(), assigned);
                        assigned
                    } else {
                        found
                    };
                } else if tokens.len() >= bounds.token {
                    stop!(6, token_start, token_line, token_column);
                }
            } else if byte.is_ascii_digit() {
                while token_end < source_length && source[token_end].is_ascii_digit() {
                    token_end += 1;
                }
                kind = 2;
                if tokens.len() >= bounds.token {
                    stop!(6, token_start, token_line, token_column);
                }
            } else {
                let pair = if next >= 0 {
                    pair_token_kind(byte, u8::try_from(next).expect("ascii next byte"))
                } else {
                    0
                };
                if pair > 0 {
                    kind = pair;
                    token_end = token_start + 2;
                } else {
                    let single = single_token_kind(byte, bounds.ampersand);
                    if single > 0 {
                        kind = single;
                        token_end = token_start + 1;
                    } else {
                        stop!(4, token_start, token_line, token_column);
                    }
                }
                if tokens.len() >= bounds.token {
                    stop!(6, token_start, token_line, token_column);
                }
            }

            let token_length = i32::try_from(token_end - token_start).expect("bounded token");
            tokens.push([
                kind,
                i32::try_from(token_start).expect("bounded token start"),
                token_length,
                token_line,
                token_column,
                name_id,
            ]);
            offset = token_end;
            column += token_length;
        }

        // The end-of-input record is appended without a capacity check.
        tokens.push([
            0,
            i32::try_from(source_length).expect("bounded source"),
            0,
            line,
            column,
            0,
        ]);
        Ingestion {
            source_length: i32::try_from(source_length).expect("bounded source"),
            consumed: i32::try_from(source_length).expect("bounded consumption"),
            status: 0,
            error_offset: -1,
            error_line: 0,
            error_column: 0,
            diagnostic_code: 0,
            diagnostic_actual: 0,
            names,
            tokens,
            signature_grammar: false,
            parameters: Vec::new(),
            nodes: Vec::new(),
            origins: Vec::new(),
        }
    }

    /// The frozen parser skeleton: `fn NAME ( ) -> int { return`.
    const SKELETON: [i32; 8] = [3, 1, 10, 11, 35, 1, 12, 6];

    /// Apply the frozen skeleton to an ingested token stream and return the
    /// first stop. Only the skeleton is modelled; H1A requires the self-source
    /// to stop inside it.
    pub fn first_parser_stop(ingested: &Ingestion, source: &[u8]) -> Ingestion {
        let mut stopped = ingested.clone();
        if ingested.status != 0 {
            return stopped;
        }
        for (step, expected) in SKELETON.iter().enumerate() {
            let record = ingested
                .tokens
                .get(step)
                .expect("frozen skeleton stays inside the token stream");
            let [kind, start, length, line, column, _] = *record;
            if kind != *expected {
                stopped.status = 10;
                stopped.error_offset = start;
                stopped.error_line = line;
                stopped.error_column = column;
                stopped.diagnostic_code = *expected;
                stopped.diagnostic_actual = kind;
                return stopped;
            }
            if step == 5 {
                let from = usize::try_from(start).expect("bounded start");
                if length != 3 || &source[from..from + 3] != b"int" {
                    stopped.status = 12;
                    stopped.error_offset = start;
                    stopped.error_line = line;
                    stopped.error_column = column;
                    stopped.diagnostic_code = 102;
                    stopped.diagnostic_actual = kind;
                    return stopped;
                }
            }
        }
        panic!("CAP-049 requires the self-source to stop inside the frozen skeleton");
    }

    /// Where the parser stops once CAP-050 / H1B-1 admits the signature grammar.
    #[derive(Debug, PartialEq, Eq)]
    pub struct SignatureStop {
        pub status: i32,
        pub error_offset: i32,
        pub error_line: i32,
        pub error_column: i32,
        pub diagnostic_code: i32,
        pub diagnostic_actual: i32,
        pub node_count: i32,
        pub parameters: i32,
    }

    /// Model the parser CAP-050 authorizes and report its first stop as a
    /// complete parse-phase state.
    ///
    /// The frozen signature grammar is `fn NAME ( params? ) -> int {` where a
    /// parameter is `IDENT : TYPE`, `TYPE` is the identifier `int` or the exact
    /// sequence `Result < int , int >`, and parameters are separated by `,`.
    /// After `return`, only enough of the accepted expression grammar is modelled
    /// to reach the frozen `; } EOF` closing sequence: a leading identifier
    /// reduces to one name-reference node, and the following token must be `;`.
    ///
    /// Parameters deliberately produce no syntax node, because the node arena is
    /// what the semantic, checked-IR, and verifier phases count.
    ///
    /// This is the single model. Both the canonical self-source target and the
    /// focused signature probes are graded against it.
    pub fn signature_parser_stop(ingested: &Ingestion, source: &[u8]) -> Ingestion {
        parser_stop(ingested, source, false)
    }

    /// Where the parser stops once CAP-051 / H1B-2 additionally admits one
    /// `match` construct in return-expression position.
    ///
    /// The admitted construct is
    /// `match IDENT { IDENT ( IDENT ) => EXPR , IDENT ( IDENT ) => EXPR , }`.
    /// The dispatch happens on the leading token of the return expression,
    /// before the operand reduction runs, so the identifier `match` never
    /// becomes a name-reference node in this position. Each arm body is the
    /// already-accepted expression grammar, modelled below as the product's own
    /// shunting-yard so nodes append in the product's order. The construct
    /// itself appends no node and needs no new node kind.
    pub fn match_parser_stop(ingested: &Ingestion, source: &[u8]) -> Ingestion {
        parser_stop(ingested, source, true)
    }

    fn parser_stop(ingested: &Ingestion, source: &[u8], admit_match: bool) -> Ingestion {
        assert_eq!(ingested.status, 0, "ingestion must succeed first");
        let mut stopped = ingested.clone();
        stopped.signature_grammar = true;
        stopped.parameters = Vec::new();
        stopped.nodes = Vec::new();
        stopped.origins = Vec::new();
        let mut index = 0usize;

        let text = |record: &TokenRecord| {
            let from = usize::try_from(record[1]).expect("bounded start");
            let to = from + usize::try_from(record[2]).expect("bounded length");
            &source[from..to]
        };
        macro_rules! reject {
            ($record:expr, $status:expr, $code:expr) => {{
                let record: &TokenRecord = $record;
                stopped.status = $status;
                stopped.error_offset = record[1];
                stopped.error_line = record[3];
                stopped.error_column = record[4];
                stopped.diagnostic_code = $code;
                stopped.diagnostic_actual = record[0];
                return stopped;
            }};
        }
        macro_rules! take {
            ($expected:expr) => {{
                let record = &ingested.tokens[index];
                if record[0] != $expected {
                    reject!(record, 10, $expected);
                }
                index += 1;
                record
            }};
        }

        take!(3); // fn
        take!(1); // function name
        take!(10); // (

        // Parameter list: either an immediate `)` or `IDENT : TYPE` repeated.
        if ingested.tokens[index][0] != 11 {
            loop {
                let name = take!(1); // parameter name
                take!(17); // :
                let ty = take!(1);
                let code = match text(ty) {
                    b"int" => 1,
                    b"Result" => {
                        take!(29); // <
                        let first = take!(1);
                        if text(first) != b"int" {
                            reject!(first, 12, 102);
                        }
                        take!(16); // ,
                        let second = take!(1);
                        if text(second) != b"int" {
                            reject!(second, 12, 102);
                        }
                        take!(31); // >
                        2
                    }
                    _ => reject!(ty, 12, 102),
                };
                stopped.parameters.push((name[5], code));
                if ingested.tokens[index][0] != 16 {
                    break;
                }
                index += 1; // ,
            }
        }
        take!(11); // )
        take!(35); // ->
        let result_type = take!(1);
        if text(result_type) != b"int" {
            reject!(result_type, 12, 102);
        }
        take!(12); // {
        take!(6); // return

        macro_rules! append {
            ($kind:expr, $payload:expr, $left:expr, $right:expr, $origin:expr) => {{
                let origin: [i32; 4] = $origin;
                stopped.nodes.push([$kind, $payload, $left, $right]);
                let id = i32::try_from(stopped.nodes.len()).expect("bounded nodes");
                stopped
                    .origins
                    .push([id, origin[0], origin[1], origin[2], origin[3]]);
                id
            }};
        }

        // CAP-051 decides on the leading token of the return expression, before
        // any operand reduction runs. `match` opens the admitted construct;
        // anything else falls through to CAP-050's operand path unchanged.
        let leading = ingested.tokens[index];
        if admit_match && leading[0] == 1 && text(&leading) == b"match" {
            index += 1; // match
            take!(1); // the scrutinee is exactly one identifier
            take!(12); // {
            for _ in 0..2 {
                take!(1); // the pattern head, matched as a bare identifier
                take!(10); // (
                take!(1); // the bound identifier
                take!(11); // )
                take!(36); // =>

                // The arm body: the accepted expression grammar, modelled as the
                // product's shunting-yard. `values` holds node ids; `operators`
                // holds token kinds, with 103/104 for the prefix forms and 10
                // for an open parenthesis, each beside its located origin.
                let mut values: Vec<i32> = Vec::new();
                let mut operators: Vec<(i32, [i32; 4])> = Vec::new();
                let mut paren_depth = 0i32;
                let mut expecting_operand = true;
                macro_rules! reduce_top {
                    () => {{
                        let (marker, at) = operators.pop().expect("modelled operator stack");
                        if marker == 103 || marker == 104 {
                            let left = values.pop().expect("modelled operand stack");
                            let kind = if marker == 103 { 3 } else { 4 };
                            values.push(append!(kind, 0, left, 0, at));
                        } else {
                            let right = values.pop().expect("modelled operand stack");
                            let left = values.pop().expect("modelled operand stack");
                            values.push(append!(binary_node_kind(marker), 0, left, right, at));
                        }
                    }};
                }
                loop {
                    let record = ingested.tokens[index];
                    let origin = [record[1], record[3], record[4], record[0]];
                    if expecting_operand {
                        if record[0] == 1 {
                            values.push(append!(2, record[5], 0, 0, origin));
                        } else if record[0] == 2 {
                            let from = usize::try_from(record[1]).expect("bounded start");
                            let to = from + usize::try_from(record[2]).expect("bounded length");
                            let mut literal = 0i32;
                            for byte in &source[from..to] {
                                literal = literal * 10 + i32::from(byte - b'0');
                            }
                            values.push(append!(1, literal, 0, 0, origin));
                        } else if record[0] == 21 || record[0] == 27 || record[0] == 10 {
                            let marker = match record[0] {
                                21 => 103,
                                27 => 104,
                                _ => 10,
                            };
                            if marker == 10 {
                                paren_depth += 1;
                            }
                            operators.push((marker, origin));
                            index += 1;
                            continue;
                        } else {
                            reject!(&record, 11, 100);
                        }
                        expecting_operand = false;
                        index += 1;
                        continue;
                    }

                    let precedence = binary_precedence(record[0]);
                    if precedence > 0 {
                        while let Some(&(marker, _)) = operators.last() {
                            if marker == 10 {
                                break;
                            }
                            let top = if marker == 103 || marker == 104 {
                                7
                            } else {
                                binary_precedence(marker)
                            };
                            if top < precedence {
                                break;
                            }
                            reduce_top!();
                        }
                        operators.push((record[0], origin));
                        expecting_operand = true;
                        index += 1;
                        continue;
                    }
                    if record[0] == 11 && paren_depth > 0 {
                        loop {
                            let marker = operators.last().expect("modelled operator stack").0;
                            if marker == 10 {
                                operators.pop();
                                break;
                            }
                            reduce_top!();
                        }
                        paren_depth -= 1;
                        index += 1;
                        continue;
                    }
                    if paren_depth > 0 {
                        reject!(&record, 10, 11);
                    }
                    while !operators.is_empty() {
                        reduce_top!();
                    }
                    assert_eq!(values.len(), 1, "one arm body reduces to one value");
                    break;
                }

                take!(16); // the arm's trailing `,`
            }
            take!(13); // the match construct's closing `}`
        } else if leading[0] == 1 {
            // CAP-050: one leading identifier operand becomes a name-reference
            // node.
            let record = ingested.tokens[index];
            append!(
                2,
                record[5],
                0,
                0,
                [record[1], record[3], record[4], record[0]]
            );
            index += 1;
        }

        // The frozen `; } EOF` closing sequence. The final step never advances
        // past the token it accepts, so it is spelled out rather than taken.
        take!(18); // ;
        take!(13); // }
        let end = &ingested.tokens[index];
        if end[0] != 0 {
            reject!(end, 10, 0);
        }
        panic!("this checkpoint requires the input to stop inside the parse phase");
    }

    /// Where the parser stops once CAP-051 / H1B-2 admits the match construct,
    /// projected out of [`match_parser_stop`].
    pub fn match_grammar_stop(ingested: &Ingestion, source: &[u8]) -> SignatureStop {
        project(&match_parser_stop(ingested, source))
    }

    /// Where the parser stops once CAP-050 / H1B-1 admits the signature grammar,
    /// projected out of [`signature_parser_stop`].
    pub fn signature_grammar_stop(ingested: &Ingestion, source: &[u8]) -> SignatureStop {
        project(&signature_parser_stop(ingested, source))
    }

    fn project(stopped: &Ingestion) -> SignatureStop {
        SignatureStop {
            status: stopped.status,
            error_offset: stopped.error_offset,
            error_line: stopped.error_line,
            error_column: stopped.error_column,
            diagnostic_code: stopped.diagnostic_code,
            diagnostic_actual: stopped.diagnostic_actual,
            node_count: i32::try_from(stopped.nodes.len()).expect("bounded nodes"),
            parameters: i32::try_from(stopped.parameters.len()).expect("bounded parameters"),
        }
    }

    /// The parse-group checksum over every source byte, name word, token word,
    /// node word, and located diagnostic field.
    pub fn parse_checksum(source: &[u8], stopped: &Ingestion) -> i32 {
        let mut checksum = 17;
        for byte in &source[..usize::try_from(stopped.source_length).expect("bounded source")] {
            checksum = checksum_step(checksum, i32::from(*byte));
        }
        checksum = checksum_step(checksum, 990);
        for (start, length) in &stopped.names {
            checksum = checksum_step(checksum, *start);
            checksum = checksum_step(checksum, *length);
        }
        checksum = checksum_step(checksum, 991);
        for record in &stopped.tokens {
            for word in record {
                checksum = checksum_step(checksum, *word);
            }
        }
        checksum = checksum_step(checksum, 992);
        // H1A produces no syntax node; CAP-050 reaches the body's first operand.
        for record in &stopped.nodes {
            for word in record {
                checksum = checksum_step(checksum, *word);
            }
        }
        checksum = checksum_step(checksum, 993);
        for word in [
            stopped.status,
            stopped.error_offset + 1,
            stopped.error_line,
            stopped.error_column,
            stopped.diagnostic_code,
            stopped.diagnostic_actual,
            i32::try_from(stopped.names.len()).expect("bounded names"),
            i32::try_from(stopped.tokens.len()).expect("bounded tokens"),
            i32::try_from(stopped.nodes.len()).expect("bounded nodes"),
            // A stopped parse never has a root.
            0,
        ] {
            checksum = checksum_step(checksum, word);
        }
        if stopped.signature_grammar {
            checksum = checksum_step(checksum, 989);
            for (name, code) in &stopped.parameters {
                checksum = checksum_step(checksum, *name);
                checksum = checksum_step(checksum, *code);
            }
            checksum = checksum_step(
                checksum,
                i32::try_from(stopped.parameters.len()).expect("bounded parameters"),
            );
        }
        checksum
    }

    /// The semantic-group checksum when the parser stopped first.
    ///
    /// The semantic phase is never entered, but the group still folds the
    /// parser's origin sidecar and reports `origin_count`, so a stopped parse
    /// that produced a node is not a group of zeros.
    pub fn unattempted_semantic_checksum(origins: &[[i32; 5]]) -> i32 {
        let mut checksum = 17;
        for record in origins {
            for word in record {
                checksum = checksum_step(checksum, *word);
            }
        }
        for word in [994, 995, 996] {
            checksum = checksum_step(checksum, word);
        }
        // status, node, offset + 1, line, column, code, expected, actual.
        for _ in 0..8 {
            checksum = checksum_step(checksum, 0);
        }
        checksum = checksum_step(
            checksum,
            i32::try_from(origins.len()).expect("bounded origins"),
        );
        // symbol_count, fact_count, semantic_root_type.
        for _ in 0..3 {
            checksum = checksum_step(checksum, 0);
        }
        checksum
    }

    /// The checked-IR-group checksum when nothing was attempted.
    pub fn unattempted_checked_checksum(semantic: i32) -> i32 {
        let mut checksum = 23;
        checksum = checksum_step(checksum, semantic);
        checksum = checksum_step(checksum, 997);
        checksum = checksum_step(checksum, 998);
        for _ in 0..16 {
            checksum = checksum_step(checksum, 0);
        }
        checksum
    }

    /// The verifier-group checksum when nothing was attempted.
    pub fn unattempted_verified_checksum() -> i32 {
        let mut checksum = 29;
        checksum = checksum_step(checksum, 995);
        for _ in 0..13 {
            checksum = checksum_step(checksum, 0);
        }
        checksum
    }

    /// The complete 67-value expectation vector `run_runtime_ascii_llvm_emitter`
    /// must match for a run that stops before the semantic phase.
    pub fn expectation_vector(source: &[u8], stopped: &Ingestion) -> Vec<i32> {
        let semantic = unattempted_semantic_checksum(&stopped.origins);
        let checked = unattempted_checked_checksum(semantic);
        let verified = unattempted_verified_checksum();
        let mut vector = vec![
            // parse group
            stopped.status,
            stopped.error_offset,
            stopped.error_line,
            stopped.error_column,
            stopped.diagnostic_code,
            stopped.diagnostic_actual,
            i32::try_from(stopped.names.len()).expect("bounded names"),
            i32::try_from(stopped.tokens.len()).expect("bounded tokens"),
            i32::try_from(stopped.nodes.len()).expect("bounded nodes"),
            0,
            parse_checksum(source, stopped),
            // semantic group - never entered, but it reports `origin_count`
            0,
            0,
            -1,
            0,
            0,
            0,
            0,
            0,
            i32::try_from(stopped.origins.len()).expect("bounded origins"),
            0,
            0,
            0,
            semantic,
            // checked-IR group - never attempted
            0,
            0,
            0,
            -1,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            checked,
            // no injected verifier fault
            -1,
            0,
            // verifier group - never attempted
            0,
            0,
            -1,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            verified,
            // emitter group - never attempted
            0,
            0,
            -1,
            0,
            0,
            0,
            // stdout driver group - never attempted
            0,
            0,
            0,
            -1,
            0,
            0,
        ];
        if stopped.signature_grammar {
            vector.push(i32::try_from(stopped.parameters.len()).expect("bounded parameters"));
        }
        vector
    }
}

// ---------------------------------------------------------------------------
// Workspace and tool plumbing
// ---------------------------------------------------------------------------

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
            .join("cap049-self-host-ingestion-tests");
        let root = parent.join(format!(
            "cap049-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create CAP-049 test workspace");
        let root = fs::canonicalize(root).expect("canonicalize CAP-049 test workspace");
        Self { root }
    }

    fn write(&self, name: &str, contents: impl AsRef<[u8]>) -> PathBuf {
        let path = self.root.join(name);
        fs::write(&path, contents).expect("write CAP-049 test artifact");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let valid = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("cap049-"));
        if valid {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn repository_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn io_options() -> CompilerOptions {
    CompilerOptions {
        language_profile: LanguageProfile::ExactI32ByteIoV0,
        ..CompilerOptions::default()
    }
}

fn read(relative: &str) -> Option<String> {
    fs::read_to_string(repository_path(relative)).ok()
}

fn accepted_b1c_source() -> String {
    fs::read_to_string(repository_path(ACCEPTED_B1C_PRODUCT)).expect("read accepted B1C product")
}

fn h1a_source() -> String {
    fs::read_to_string(repository_path(H1A_PRODUCT)).expect("read CAP-049 canonical source")
}

fn compiled_b1c() -> &'static String {
    B1C_LLVM.get_or_init(|| {
        compile_program(&accepted_b1c_source(), io_options())
            .expect("accepted B1C product compiles")
    })
}

fn compiled_h1a() -> &'static String {
    H1A_LLVM.get_or_init(|| {
        compile_program(&h1a_source(), io_options()).expect("CAP-049 canonical source compiles")
    })
}

fn clang_link(
    workspace: &TestWorkspace,
    label: &str,
    optimization: &str,
    inputs: &[&Path],
) -> PathBuf {
    let executable = workspace.root.join(if cfg!(windows) {
        format!("{label}-{optimization}.exe")
    } else {
        format!("{label}-{optimization}")
    });
    let output = Command::new(llvm_bin().join("clang"))
        .args([
            "-std=c11",
            optimization,
            "-Wall",
            "-Wextra",
            "-Werror",
            "-Wno-error=override-module",
        ])
        .args(inputs)
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("execute Clang for CAP-049 independent oracle");
    assert!(
        output.status.success(),
        "CAP-049 link failed at {optimization} (stdout={:?}, stderr={:?})",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    executable
}

fn run_command_with_stdin(command: &mut Command, input: &[u8]) -> Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn CAP-049 child");
    child
        .stdin
        .take()
        .expect("piped CAP-049 stdin")
        .write_all(input)
        .expect("write CAP-049 binary stdin");
    child.wait_with_output().expect("wait for CAP-049 child")
}

fn llvm_bin() -> PathBuf {
    for variable in ["AERO_LLVM_BIN", "LLVM_BIN"] {
        if let Some(path) = std::env::var_os(variable).map(PathBuf::from)
            && path
                .join(if cfg!(windows) { "clang.exe" } else { "clang" })
                .is_file()
        {
            return path;
        }
    }
    let path = std::env::var_os("PATH").expect("CAP-049 tests require PATH");
    std::env::split_paths(&path)
        .find(|directory| {
            directory
                .join(if cfg!(windows) { "clang.exe" } else { "clang" })
                .is_file()
        })
        .expect("CAP-049 tests require an explicit LLVM bin directory")
}

/// Rename the product entry point so an oracle harness can own `main`.
fn renamed_product(llvm: &str) -> String {
    let renamed = llvm.replacen("define i32 @main()", "define i32 @aero_product_main()", 1);
    assert_ne!(renamed, llvm, "product entry point was not renamed");
    renamed
}

/// Generate the C oracle harness. The exact tracked source bytes are compiled
/// into the harness, streamed into the product's stdin intrinsic, and the
/// compiler entry point is called with the exact expectation vector. Exit 91
/// means every one of the 67 values matched.
fn expectation_harness(expected: &[i32], source: &[u8], consumed: i32) -> String {
    let mut arguments = String::new();
    for (index, value) in expected.iter().enumerate() {
        if index % 6 == 0 {
            arguments.push_str("\n        ");
        }
        write!(arguments, "{value}, ").expect("format CAP-049 expectation");
    }
    let arguments = arguments.trim_end().trim_end_matches(',').to_string();

    let mut bytes = String::with_capacity(source.len() * 5);
    for (index, byte) in source.iter().enumerate() {
        if index % 20 == 0 {
            bytes.push_str("\n    ");
        }
        write!(bytes, "{byte},").expect("format CAP-049 source byte");
    }
    let bytes = bytes.trim_end_matches(',').to_string();

    format!(
        r#"
#include <stdint.h>

extern int32_t run_runtime_ascii_llvm_emitter({signature});
extern int32_t aero_test_reset(uint64_t fail_after_successes);
extern uint64_t aero_test_alloc_calls(void);
extern uint64_t aero_test_dealloc_calls(void);
extern uint64_t aero_test_live_allocations(void);
extern uint64_t aero_test_size_mismatch_calls(void);

static const unsigned char input[] = {{{bytes}
}};
static const long input_length = (long)(sizeof(input) / sizeof(input[0]));
static long input_index;
static int wrote_output;

int32_t aero_stdin_read_byte(void) {{
    if (input_index < input_length) return (int32_t)input[input_index++];
    return -1;
}}

int32_t aero_stdout_write_byte(int32_t value) {{
    (void)value;
    wrote_output = 1;
    return -1;
}}

int main(void) {{
    if (input_length != {length}) return 60;
    if (aero_test_reset(UINT64_MAX) != 1) return 61;

    int32_t result = run_runtime_ascii_llvm_emitter({arguments});

    if (wrote_output != 0) return 62;
    if (input_index != {consumed}) return 63;
    if (aero_test_live_allocations() != 0) return 64;
    if (aero_test_size_mismatch_calls() != 0) return 65;
    if (aero_test_alloc_calls() != aero_test_dealloc_calls()) return 66;
    if (aero_test_alloc_calls() == 0) return 67;
    return result;
}}
"#,
        signature = vec!["int32_t"; expected.len()].join(", "),
        arguments = arguments,
        bytes = bytes,
        length = source.len(),
        consumed = consumed,
    )
}

/// Link the product against the oracle harness and return the exit code.
fn run_expectation(
    label: &str,
    product_llvm: &str,
    source: &[u8],
    stopped: &oracle::Ingestion,
    optimization: &str,
) -> i32 {
    let expected = oracle::expectation_vector(source, stopped);
    let workspace = TestWorkspace::new(label);
    let llvm = workspace.write("product.ll", renamed_product(product_llvm));
    let harness = workspace.write(
        "expectation.c",
        expectation_harness(&expected, source, stopped.consumed),
    );
    let runtime = repository_path("../../src/compiler/runtime/aero_test_runtime.c");
    let executable = clang_link(
        &workspace,
        label,
        optimization,
        &[llvm.as_path(), runtime.as_path(), harness.as_path()],
    );
    let output = Command::new(executable)
        .output()
        .expect("run CAP-049 expectation harness");
    assert!(
        output.stdout.is_empty(),
        "CAP-049 harness wrote stdout: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
    output.status.code().expect("CAP-049 harness exit code")
}

/// The CAP-050a parameter-store validation and checksum region, inserted
/// after the parse group's `root` fold.
const PARAMETER_FOLD: &str = r#"    checksum = checksum_step(checksum, root);

    checksum = checksum_step(checksum, 989);
    let mut validate_parameter: int = 0;
    let mut validate_param_field: int = 0;
    let mut validate_param_name: int = 0;
    let mut validate_param_type: int = 0;
    if bytes_len(&parameters) != parameter_count * 8 {
        return 69;
    }
    while validate_parameter < parameter_count {
        validate_param_field = 0;
        while validate_param_field < 2 {
            word_offset = validate_parameter * 8 + validate_param_field * 4;
            byte_0 = result_value(bytes_get(&parameters, word_offset));
            byte_1 = result_value(bytes_get(&parameters, word_offset + 1));
            byte_2 = result_value(bytes_get(&parameters, word_offset + 2));
            byte_3 = result_value(bytes_get(&parameters, word_offset + 3));
            if byte_0 < 0 || byte_1 < 0 || byte_2 < 0 || byte_3 < 0 || byte_3 > 127 {
                return 69;
            }
            word = byte_0 + byte_1 * 256 + byte_2 * 65536 + byte_3 * 16777216;
            checksum = checksum_step(checksum, word);
            if validate_param_field == 0 {
                validate_param_name = word;
            } else {
                validate_param_type = word;
            }
            validate_param_field = validate_param_field + 1;
        }
        if validate_param_name <= 0 || validate_param_name > name_count
            || validate_param_type < 1 || validate_param_type > 2 {
            return 69;
        }
        validate_parameter = validate_parameter + 1;
    }
    checksum = checksum_step(checksum, parameter_count);

    if status != expected_status"#;

/// CAP-050's parameter sub-machine, inserted into the frozen skeleton block.
///
/// The Aero product and this reconstruction are patched from one shared
/// definition, so the admitted grammar and its byte-for-byte derivation cannot
/// drift apart.
const PARAM_REGISTERS_ANCHOR: &str = r#"    let mut parameter_count: int = 0;
"#;
const PARAM_REGISTERS: &str = r#"    let mut parameter_count: int = 0;
    let mut param_mode: int = 0;
    let mut param_cycle_mode: int = 0;
    let mut param_alternate: int = 0;
    let mut param_hold: int = 0;
    let mut param_store: int = 0;
    let mut param_name_id: int = 0;
    let mut param_type_code: int = 0;
    let mut param_is_int: int = 0;
    let mut param_is_result: int = 0;
    let mut param_failed: int = 0;
    let mut param_push: int = 0;
    let mut param_b0: int = 0;
    let mut param_b1: int = 0;
    let mut param_b2: int = 0;
    let mut param_b3: int = 0;
    let mut param_b4: int = 0;
    let mut param_b5: int = 0;
"#;
const SKELETON_TABLE_ANCHOR: &str = r#"            if skeleton_step == 7 {
                expected_kind = 6;
            }
            if current_kind < 0 || current_start < 0 || current_length < 0 {"#;
const SKELETON_TABLE: &str = r#"            if skeleton_step == 7 {
                expected_kind = 6;
            }
            // CAP-050 signature grammar. Between the '(' at step 2 and the '->'
            // at step 4 the parameter sub-machine owns step 3. Its mode is
            // latched once per token, exactly as parser_state is, so the flat
            // per-mode branches cannot cascade within one iteration.
            param_cycle_mode = param_mode;
            param_alternate = 0;
            param_hold = 0;
            param_store = 0;
            if skeleton_step == 3 {
                expected_kind = 1;
                if param_cycle_mode == 0 {
                    param_alternate = 11;
                }
                if param_cycle_mode == 1 {
                    expected_kind = 17;
                }
                if param_cycle_mode == 3 {
                    expected_kind = 29;
                }
                if param_cycle_mode == 5 {
                    expected_kind = 16;
                }
                if param_cycle_mode == 7 {
                    expected_kind = 31;
                }
                if param_cycle_mode == 8 {
                    expected_kind = 11;
                    param_alternate = 16;
                }
                param_hold = 1;
            }
            if current_kind < 0 || current_start < 0 || current_length < 0 {"#;
const SUB_MACHINE_ANCHOR: &str = r#"            if parser_running == 1 && current_kind != expected_kind {
                status = 10;
                error_offset = current_start;
                error_line = current_line;
                error_column = current_column;
                diagnostic_code = expected_kind;
                diagnostic_actual = current_kind;
                parser_running = 0;
            }
            int_matches = 1;"#;
const SUB_MACHINE: &str = r#"            if parser_running == 1 && current_kind != expected_kind
                && (param_alternate == 0 || current_kind != param_alternate) {
                status = 10;
                error_offset = current_start;
                error_line = current_line;
                error_column = current_column;
                diagnostic_code = expected_kind;
                diagnostic_actual = current_kind;
                parser_running = 0;
            }
            if parser_running == 1 && skeleton_step == 3 {
                if param_cycle_mode == 0 && current_kind == 11 {
                    param_hold = 0;
                    param_mode = 0;
                }
                if param_cycle_mode == 8 && current_kind == 11 {
                    param_hold = 0;
                    param_mode = 0;
                }
                if param_cycle_mode == 8 && current_kind == 16 {
                    param_mode = 9;
                }
                if (param_cycle_mode == 0 && current_kind == 1)
                    || param_cycle_mode == 9 {
                    param_name_id = current_name_id;
                    param_mode = 1;
                    if param_name_id <= 0 {
                        status = 16;
                        parser_running = 0;
                    }
                }
                if param_cycle_mode == 1 {
                    param_mode = 2;
                }
                if param_cycle_mode == 3 {
                    param_mode = 4;
                }
                if param_cycle_mode == 5 {
                    param_mode = 6;
                }
                if param_cycle_mode == 7 {
                    param_type_code = 2;
                    param_store = 1;
                    param_mode = 8;
                }
                if param_cycle_mode == 2 || param_cycle_mode == 4
                    || param_cycle_mode == 6 {
                    param_is_int = 0;
                    param_is_result = 0;
                    if current_length == 3 {
                        param_b0 = result_value(bytes_get(&source, current_start));
                        param_b1 = result_value(bytes_get(&source, current_start + 1));
                        param_b2 = result_value(bytes_get(&source, current_start + 2));
                        if param_b0 == 105 && param_b1 == 110 && param_b2 == 116 {
                            param_is_int = 1;
                        }
                    }
                    if param_cycle_mode == 2 && current_length == 6 {
                        param_b0 = result_value(bytes_get(&source, current_start));
                        param_b1 = result_value(bytes_get(&source, current_start + 1));
                        param_b2 = result_value(bytes_get(&source, current_start + 2));
                        param_b3 = result_value(bytes_get(&source, current_start + 3));
                        param_b4 = result_value(bytes_get(&source, current_start + 4));
                        param_b5 = result_value(bytes_get(&source, current_start + 5));
                        if param_b0 == 82 && param_b1 == 101 && param_b2 == 115
                            && param_b3 == 117 && param_b4 == 108
                            && param_b5 == 116 {
                            param_is_result = 1;
                        }
                    }
                    if param_is_int == 0 && param_is_result == 0 {
                        status = 12;
                        error_offset = current_start;
                        error_line = current_line;
                        error_column = current_column;
                        diagnostic_code = 102;
                        diagnostic_actual = current_kind;
                        parser_running = 0;
                    }
                    if parser_running == 1 && param_cycle_mode == 2
                        && param_is_int == 1 {
                        param_type_code = 1;
                        param_store = 1;
                        param_mode = 8;
                    }
                    if parser_running == 1 && param_cycle_mode == 2
                        && param_is_result == 1 {
                        param_mode = 3;
                    }
                    if parser_running == 1 && param_cycle_mode == 4 {
                        param_mode = 5;
                    }
                    if parser_running == 1 && param_cycle_mode == 6 {
                        param_mode = 7;
                    }
                }
                if parser_running == 1 && param_store == 1 {
                    param_failed = 0;
                    param_push = result_value(bytes_push(&mut parameters,
                        word_byte_0(param_name_id)));
                    if param_push < 0 {
                        param_failed = 1;
                    }
                    param_push = result_value(bytes_push(&mut parameters,
                        word_byte_1(param_name_id)));
                    if param_push < 0 {
                        param_failed = 1;
                    }
                    param_push = result_value(bytes_push(&mut parameters,
                        word_byte_2(param_name_id)));
                    if param_push < 0 {
                        param_failed = 1;
                    }
                    param_push = result_value(bytes_push(&mut parameters,
                        word_byte_3(param_name_id)));
                    if param_push < 0 {
                        param_failed = 1;
                    }
                    param_push = result_value(bytes_push(&mut parameters,
                        word_byte_0(param_type_code)));
                    if param_push < 0 {
                        param_failed = 1;
                    }
                    param_push = result_value(bytes_push(&mut parameters,
                        word_byte_1(param_type_code)));
                    if param_push < 0 {
                        param_failed = 1;
                    }
                    param_push = result_value(bytes_push(&mut parameters,
                        word_byte_2(param_type_code)));
                    if param_push < 0 {
                        param_failed = 1;
                    }
                    param_push = result_value(bytes_push(&mut parameters,
                        word_byte_3(param_type_code)));
                    if param_push < 0 {
                        param_failed = 1;
                    }
                    if param_failed == 1 {
                        status = 8;
                        error_offset = current_start;
                        error_line = current_line;
                        error_column = current_column;
                        parser_running = 0;
                    } else {
                        parameter_count = parameter_count + 1;
                    }
                }
            }
            int_matches = 1;"#;
const ADVANCE_ANCHOR: &str = r#"            if parser_running == 1 {
                parse_index = parse_index + 1;
                skeleton_step = skeleton_step + 1;
                parser_state = 1;
                if skeleton_step == 8 {
                    parser_state = 3;
                }
            }"#;
const HELD_ADVANCE: &str = r#"            if parser_running == 1 {
                parse_index = parse_index + 1;
                if param_hold == 0 {
                    skeleton_step = skeleton_step + 1;
                }
                parser_state = 1;
                if skeleton_step == 8 {
                    parser_state = 3;
                }
            }"#;

/// CAP-051's match construct, inserted into the CAP-050 parser.
///
/// The Aero product and this reconstruction are patched from one shared
/// definition, so the admitted grammar and its byte-for-byte derivation
/// cannot drift apart.
const MATCH_REGISTERS_ANCHOR: &str = r#"    let mut param_b5: int = 0;
"#;
const MATCH_REGISTERS: &str = r#"    let mut param_b5: int = 0;
    let mut match_active: int = 0;
    let mut match_step: int = 0;
    let mut match_cycle_step: int = 0;
    let mut match_expected: int = 0;
    let mut match_is_match: int = 0;
    let mut match_b0: int = 0;
    let mut match_b1: int = 0;
    let mut match_b2: int = 0;
    let mut match_b3: int = 0;
    let mut match_b4: int = 0;
"#;
const RETURN_DISPATCH_ANCHOR: &str = r#"                parser_state = 1;
                if skeleton_step == 8 {
                    parser_state = 3;
                }"#;
const RETURN_DISPATCH: &str = r#"                parser_state = 1;
                if skeleton_step == 8 {
                    parser_state = 40;
                }"#;
const MATCH_REQUESTS_ANCHOR: &str = r#"        if parser_cycle_state == 20 {
            parser_token_after = 21;
            parser_token_field = 0;
            parser_token_byte = 0;
            parser_token_word = 0;
            parser_state = 30;
        }"#;
const MATCH_REQUESTS: &str = r#"        if parser_cycle_state == 20 {
            parser_token_after = 21;
            parser_token_field = 0;
            parser_token_byte = 0;
            parser_token_word = 0;
            parser_state = 30;
        }
        // CAP-051 requests the leading token of the return expression, and then
        // one token per step of the admitted match construct.
        if parser_cycle_state == 40 {
            parser_token_after = 41;
            parser_token_field = 0;
            parser_token_byte = 0;
            parser_token_word = 0;
            parser_state = 30;
        }
        if parser_cycle_state == 42 {
            parser_token_after = 43;
            parser_token_field = 0;
            parser_token_byte = 0;
            parser_token_word = 0;
            parser_state = 30;
        }"#;
const MATCH_STATES_ANCHOR: &str = r#"        // Classify one expression token after its F1A record has been decoded.
        if parser_cycle_state == 4 {"#;
const MATCH_STATES: &str = r#"        // CAP-051 decides the return expression on its leading token, before
        // the operand classifier runs. The identifier 'match' opens the admitted
        // construct; every other token falls through to state 4 with the same
        // decoded record, so no node is appended for 'match' and the
        // append-only node arena never has to retract one.
        if parser_cycle_state == 41 {
            if current_kind < 0 || current_start < 0 || current_length < 0 {
                status = 16;
                parser_running = 0;
            }
            if parser_running == 1 && (current_line <= 0 || current_column <= 0
                || current_name_id < 0) {
                status = 16;
                parser_running = 0;
            }
            if parser_running == 1 {
                match_is_match = 0;
                if current_kind == 1 && current_length == 5 {
                    match_b0 = result_value(bytes_get(&source, current_start));
                    match_b1 = result_value(bytes_get(&source, current_start + 1));
                    match_b2 = result_value(bytes_get(&source, current_start + 2));
                    match_b3 = result_value(bytes_get(&source, current_start + 3));
                    match_b4 = result_value(bytes_get(&source, current_start + 4));
                    if match_b0 == 109 && match_b1 == 97 && match_b2 == 116
                        && match_b3 == 99 && match_b4 == 104 {
                        match_is_match = 1;
                    }
                }
                if match_is_match == 1 {
                    match_active = 1;
                    match_step = 0;
                    parse_index = parse_index + 1;
                    parser_state = 42;
                } else {
                    parser_state = 4;
                }
            }
        }

        // The one admitted match construct:
        // 'match IDENT { IDENT ( IDENT ) => EXPR , IDENT ( IDENT ) => EXPR , }'.
        // Every step is one exact token except 6 and 12, which hand the arm body
        // to the accepted expression grammar; state 18 returns here. The
        // construct itself appends no node and needs no new node kind.
        if parser_cycle_state == 43 {
            match_cycle_step = match_step;
            match_expected = 1;
            if match_cycle_step == 1 {
                match_expected = 12;
            }
            if match_cycle_step == 3 || match_cycle_step == 9 {
                match_expected = 10;
            }
            if match_cycle_step == 5 || match_cycle_step == 11 {
                match_expected = 11;
            }
            if match_cycle_step == 6 || match_cycle_step == 12 {
                match_expected = 36;
            }
            if match_cycle_step == 7 || match_cycle_step == 13 {
                match_expected = 16;
            }
            if match_cycle_step == 14 {
                match_expected = 13;
            }
            if current_kind < 0 || current_start < 0 || current_line <= 0
                || current_column <= 0 {
                status = 16;
                parser_running = 0;
            }
            if parser_running == 1 && current_kind != match_expected {
                status = 10;
                error_offset = current_start;
                error_line = current_line;
                error_column = current_column;
                diagnostic_code = match_expected;
                diagnostic_actual = current_kind;
                parser_running = 0;
            }
            if parser_running == 1 {
                parse_index = parse_index + 1;
                match_step = match_cycle_step + 1;
                parser_state = 42;
                if match_cycle_step == 6 || match_cycle_step == 12 {
                    expecting_operand = 1;
                    parser_state = 3;
                }
                if match_cycle_step == 14 {
                    match_active = 0;
                    closing_step = 0;
                    parser_state = 20;
                }
            }
        }

        // Classify one expression token after its F1A record has been decoded.
        if parser_cycle_state == 4 {"#;
const ARM_RETURN_ANCHOR: &str = r#"        if parser_cycle_state == 18 {
            expression_root = parser_record_0;
            value_previous = parser_record_1;
            if expression_root <= 0 || value_previous != 0 {
                status = 16;
                parser_running = 0;
            }
            if parser_running == 1 {
                closing_step = 0;
                parser_state = 20;
            }
        }"#;
const ARM_RETURN: &str = r#"        if parser_cycle_state == 18 {
            expression_root = parser_record_0;
            value_previous = parser_record_1;
            if expression_root <= 0 || value_previous != 0 {
                status = 16;
                parser_running = 0;
            }
            if parser_running == 1 && match_active == 1 {
                value_top = 0;
                value_depth = 0;
                parser_state = 42;
            }
            if parser_running == 1 && match_active == 0 {
                closing_step = 0;
                parser_state = 20;
            }
        }"#;

/// Apply the six frozen CAP-049 ingestion differences to the accepted B1C
/// source. `compiler.aero` must equal this byte for byte.
fn expected_h1a_source() -> String {
    let accepted = accepted_b1c_source();

    // 1. A lone `&` becomes token kind 37.
    let single_anchor = "    if value == 62 {\n        return 31;\n    }\n    return 0;\n}";
    assert_eq!(accepted.matches(single_anchor).count(), 1);
    let derived = accepted.replace(
        single_anchor,
        "    if value == 62 {\n        return 31;\n    }\n    if value == 38 {\n        return 37;\n    }\n    return 0;\n}",
    );

    // 2. The token-record validator admits the new kind.
    let kind_anchor = "if validate_kind < 0 || validate_kind > 36 || validate_start < previous_end";
    assert_eq!(derived.matches(kind_anchor).count(), 1);
    let derived = derived.replace(
        kind_anchor,
        "if validate_kind < 0 || validate_kind > 37 || validate_start < previous_end",
    );

    // 3. The stdin ingestion bound and its reported overflow offset.
    let bound_anchor = "            if bytes_len(&source) >= 8192 {";
    assert_eq!(derived.matches(bound_anchor).count(), 1);
    let derived = derived.replace(
        bound_anchor,
        &format!("            if bytes_len(&source) >= {H1A_SOURCE_BOUND} {{"),
    );
    let offset_anchor = "                error_offset = 8192;";
    assert_eq!(derived.matches(offset_anchor).count(), 1);
    let derived = derived.replace(
        offset_anchor,
        &format!("                error_offset = {H1A_SOURCE_BOUND};"),
    );

    // 4. The interned-name bound.
    assert_eq!(derived.matches("name_count >= 1024").count(), 1);
    let derived = derived.replace(
        "name_count >= 1024",
        &format!("name_count >= {H1A_NAME_BOUND}"),
    );

    // 5. The token-record bound at all four lexer sites.
    assert_eq!(derived.matches("token_count >= 1024").count(), 4);
    let derived = derived.replace(
        "token_count >= 1024",
        &format!("token_count >= {H1A_TOKEN_BOUND}"),
    );

    // 6. The located-token re-derivation carries its scan position forward.
    let rescan_anchor = "            return 75;\n        }\n        location_index = 0;\n        location_line = 1;\n        location_column = 1;\n        while location_index < validate_start {";
    assert_eq!(derived.matches(rescan_anchor).count(), 1);
    let derived = derived.replace(
        rescan_anchor,
        "            return 75;\n        }\n        while location_index < validate_start {",
    );

    // CAP-050a adds the parameter store: one owner, one counter, the 68th
    // expectation, the validated `989` checksum region, and the parse-group
    // comparison. No parser rule changes.
    let owner_anchor = "    let mut nodes: ByteBuffer = bytes_new();\n";
    assert_eq!(derived.matches(owner_anchor).count(), 1);
    let derived = derived.replace(
        owner_anchor,
        "    let mut nodes: ByteBuffer = bytes_new();\n    let mut parameters: ByteBuffer = bytes_new();\n",
    );

    let signature_anchor = "        expected_driven_checksum: int) -> int {";
    assert_eq!(derived.matches(signature_anchor).count(), 1);
    let derived = derived.replace(
        signature_anchor,
        "        expected_driven_checksum: int, expected_parameters: int) -> int {",
    );

    let guard_anchor = "        || expected_driven_checksum < 0 {";
    assert_eq!(derived.matches(guard_anchor).count(), 1);
    let derived = derived.replace(
        guard_anchor,
        "        || expected_driven_checksum < 0 || expected_parameters < 0 {",
    );

    let counter_anchor = "    let mut int_matches: int = 0;\n";
    assert_eq!(derived.matches(counter_anchor).count(), 1);
    let derived = derived.replace(
        counter_anchor,
        "    let mut int_matches: int = 0;\n    let mut parameter_count: int = 0;\n",
    );

    let fold_anchor =
        "    checksum = checksum_step(checksum, root);\n\n    if status != expected_status";
    assert_eq!(derived.matches(fold_anchor).count(), 1);
    let derived = derived.replace(fold_anchor, PARAMETER_FOLD);

    let compare_anchor = "        || node_count != expected_nodes || root != expected_root\n        || checksum != expected_checksum {";
    assert_eq!(derived.matches(compare_anchor).count(), 1);
    let derived = derived.replace(
        compare_anchor,
        "        || node_count != expected_nodes || root != expected_root\n        || parameter_count != expected_parameters\n        || checksum != expected_checksum {",
    );

    // The canonical program records no parameter, so its self-test vector moves
    // only by the new `989` region: step(step(586661, 989), 0) == 810191.
    let vector_anchor = "        2, 20, 11, 11, 586661,";
    assert_eq!(derived.matches(vector_anchor).count(), 1);
    let derived = derived.replace(vector_anchor, "        2, 20, 11, 11, 810191,");
    let tail_anchor = "        1, 0, 0, -1, 144, 506643);";
    assert_eq!(derived.matches(tail_anchor).count(), 1);
    let derived = derived.replace(tail_anchor, "        1, 0, 0, -1, 144, 506643, 0);");

    // CAP-050 adds the signature-grammar sub-machine on top of that proven
    // store: the latched mode with its scratch registers, the mode-driven
    // expected-kind table, the transition-and-append block, and the held
    // `skeleton_step` advance. Nothing else in the parser moves, and no syntax
    // node is created for a parameter.
    assert_eq!(derived.matches(PARAM_REGISTERS_ANCHOR).count(), 1);
    let derived = derived.replace(PARAM_REGISTERS_ANCHOR, PARAM_REGISTERS);

    assert_eq!(derived.matches(SKELETON_TABLE_ANCHOR).count(), 1);
    let derived = derived.replace(SKELETON_TABLE_ANCHOR, SKELETON_TABLE);

    assert_eq!(derived.matches(SUB_MACHINE_ANCHOR).count(), 1);
    let derived = derived.replace(SUB_MACHINE_ANCHOR, SUB_MACHINE);

    assert_eq!(derived.matches(ADVANCE_ANCHOR).count(), 1);
    let derived = derived.replace(ADVANCE_ANCHOR, HELD_ADVANCE);

    // CAP-051 admits one `match` construct in return-expression position: the
    // scratch registers, the leading-token dispatch, the two token requests,
    // the construct's own step table, and the arm-body return from the accepted
    // expression grammar. No node kind is added and no node is retracted.
    assert_eq!(derived.matches(MATCH_REGISTERS_ANCHOR).count(), 1);
    let derived = derived.replace(MATCH_REGISTERS_ANCHOR, MATCH_REGISTERS);

    assert_eq!(derived.matches(RETURN_DISPATCH_ANCHOR).count(), 1);
    let derived = derived.replace(RETURN_DISPATCH_ANCHOR, RETURN_DISPATCH);

    assert_eq!(derived.matches(MATCH_REQUESTS_ANCHOR).count(), 1);
    let derived = derived.replace(MATCH_REQUESTS_ANCHOR, MATCH_REQUESTS);

    assert_eq!(derived.matches(MATCH_STATES_ANCHOR).count(), 1);
    let derived = derived.replace(MATCH_STATES_ANCHOR, MATCH_STATES);

    assert_eq!(derived.matches(ARM_RETURN_ANCHOR).count(), 1);
    derived.replace(ARM_RETURN_ANCHOR, ARM_RETURN)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn accepted_b1c_product_is_unchanged_before_h1a() {
    let bytes = fs::read(repository_path(ACCEPTED_B1C_PRODUCT)).expect("read accepted B1C product");
    assert_eq!(bytes.len(), ACCEPTED_B1C_SOURCE_LENGTH);
    assert_eq!(
        format!("{:x}", md5::compute(&bytes)),
        ACCEPTED_B1C_SOURCE_MD5
    );
    assert!(
        !bytes.contains(&b'\r'),
        "accepted B1C product is not LF-only"
    );
    assert!(
        bytes.iter().all(|byte| *byte < 128),
        "accepted B1C product is not 7-bit ASCII"
    );

    let source = accepted_b1c_source();
    for marker in [
        "// CAP-045 B1A VERIFIER BEGIN",
        "// CAP-046 B1B LLVM EMITTER BEGIN",
        "// CAP-047 B1C STDOUT DRIVER BEGIN",
        "// CAP-047 TRACKED SELF-TEST",
    ] {
        assert_eq!(
            source.matches(marker).count(),
            1,
            "B1C marker {marker} drifted"
        );
    }
    check_program(&source, io_options()).expect("accepted B1C product still checks");
    verify_llvm_module(compiled_b1c(), LlvmVerificationMode::Required)
        .expect("accepted B1C LLVM still verifies");
}

#[test]
fn independent_oracle_predicts_the_accepted_eight_kilobyte_self_input_boundary() {
    let source = fs::read(repository_path(ACCEPTED_B1C_PRODUCT)).expect("read accepted B1C");
    let bounds = oracle::Bounds {
        source: ACCEPTED_SOURCE_BOUND,
        token: 1_024,
        name: 1_024,
        ampersand: false,
    };
    let stopped = oracle::ingest(&source, &bounds);
    assert_eq!(
        stopped.status, 2,
        "accepted B1C must stop at its source bound"
    );
    assert_eq!(
        stopped.error_offset,
        i32::try_from(ACCEPTED_SOURCE_BOUND).unwrap()
    );
    assert!(stopped.tokens.is_empty());
    assert!(stopped.names.is_empty());

    // Raising only the source bound reaches the next honest boundary: the lone
    // `&` of `bytes_len(&source)` has no token kind in the accepted table.
    let lexical = oracle::ingest(
        &source,
        &oracle::Bounds {
            source: usize::MAX,
            token: usize::MAX,
            name: usize::MAX,
            ampersand: false,
        },
    );
    assert_eq!(lexical.status, 4, "the next accepted boundary is lexical");
    let at = usize::try_from(lexical.error_offset).unwrap();
    assert_eq!(source[at], b'&');
    assert_ne!(source[at + 1], b'&');

    // Admitting that one lexical form completes the stream.
    let complete = oracle::ingest(
        &source,
        &oracle::Bounds {
            source: usize::MAX,
            token: usize::MAX,
            name: usize::MAX,
            ampersand: true,
        },
    );
    assert_eq!(complete.status, 0, "the accepted source lexes with `&`");
    assert!(
        complete.tokens.len() > 1_024,
        "the token bound is also real"
    );
    assert!(complete.names.len() < H1A_NAME_BOUND);
    assert!(complete.tokens.len() < H1A_TOKEN_BOUND);
}

#[test]
fn accepted_b1c_stops_at_the_eight_kilobyte_boundary_on_self_input() {
    let source = fs::read(repository_path(ACCEPTED_B1C_PRODUCT)).expect("read accepted B1C");
    let stopped = oracle::ingest(
        &source,
        &oracle::Bounds {
            source: ACCEPTED_SOURCE_BOUND,
            token: 1_024,
            name: 1_024,
            ampersand: false,
        },
    );
    assert_eq!(
        stopped.consumed,
        i32::try_from(ACCEPTED_SOURCE_BOUND + 1).unwrap(),
        "the bounded stop still consumes the byte it rejects"
    );
    assert_eq!(
        run_expectation("b1c-boundary", compiled_b1c(), &source, &stopped, "-O2"),
        91,
        "the accepted B1C product did not stop exactly at its 8,192-byte bound"
    );
}

#[test]
fn canonical_self_host_compiler_source_is_present() {
    let required = [
        (
            H1A_PRODUCT,
            &[
                "if value == 38 {",
                "return 37;",
                "validate_kind > 37",
                "// CAP-047 B1C STDOUT DRIVER BEGIN",
            ][..],
        ),
        (WORKFLOW, &["Test canonical self-host source ingestion"][..]),
    ];
    let ready = required.iter().all(|(relative, anchors)| {
        read(relative)
            .is_some_and(|contents| anchors.iter().all(|anchor| contents.contains(anchor)))
    });
    if !ready {
        panic!("{INTENTIONAL_PRODUCT_RED}");
    }
}

#[test]
fn canonical_self_host_source_is_a_copy_derived_successor() {
    let bytes = fs::read(repository_path(H1A_PRODUCT)).expect("read CAP-049 canonical source");
    assert!(!bytes.contains(&b'\r'), "canonical source is not LF-only");
    assert!(
        bytes.iter().all(|byte| *byte < 128),
        "canonical source is not 7-bit ASCII"
    );
    assert_eq!(
        h1a_source(),
        expected_h1a_source(),
        "canonical source differs from accepted B1C in more than the six frozen ways"
    );
    // Every accepted product section survives unchanged.
    let derived = h1a_source();
    for marker in [
        "// CAP-045 B1A VERIFIER BEGIN",
        "// CAP-045 B1A VERIFIER END",
        "// CAP-046 B1B LLVM EMITTER BEGIN",
        "// CAP-046 B1B LLVM EMITTER END",
        "// CAP-047 B1C STDOUT DRIVER BEGIN",
        "// CAP-047 B1C STDOUT DRIVER END",
    ] {
        assert_eq!(derived.matches(marker).count(), 1);
    }
    let section = |source: &str, begin: &str, end: &str| {
        source
            .split_once(begin)
            .and_then(|(_, suffix)| suffix.split_once(end).map(|(body, _)| body))
            .expect("isolate accepted section")
            .to_string()
    };
    let accepted = accepted_b1c_source();
    for (begin, end) in [
        (
            "// CAP-046 B1B LLVM EMITTER BEGIN",
            "// CAP-046 B1B LLVM EMITTER END",
        ),
        (
            "// CAP-047 B1C STDOUT DRIVER BEGIN",
            "// CAP-047 B1C STDOUT DRIVER END",
        ),
    ] {
        assert_eq!(
            section(&derived, begin, end),
            section(&accepted, begin, end),
            "CAP-049 changed the accepted {begin} section"
        );
    }
}

#[test]
fn canonical_self_host_source_compiles_deterministically() {
    let source = h1a_source();
    check_program(&source, io_options()).expect("CAP-049 canonical source checks");
    let first = compiled_h1a().clone();
    let second = compile_program(&source, io_options()).expect("CAP-049 source recompiles");
    assert_eq!(first, second, "CAP-049 LLVM became nondeterministic");
    verify_llvm_module(&first, LlvmVerificationMode::Required)
        .expect("CAP-049 canonical LLVM verifies");

    let workspace = TestWorkspace::new("file-parity");
    let path = workspace.write("compiler.aero", &source);
    check_file(&path, io_options()).expect("CAP-049 canonical file checks");
    assert_eq!(
        compile_file(&path, io_options()).expect("CAP-049 canonical file compiles"),
        first
    );
}

#[test]
fn canonical_self_host_source_preserves_the_accepted_canonical_module() {
    let workspace = TestWorkspace::new("canonical-module");
    let llvm = workspace.write("compiler.ll", compiled_h1a().as_bytes());
    let runtime = repository_path("../../src/compiler/runtime/aero_runtime.c");
    for optimization in ["-O0", "-O2"] {
        let executable = clang_link(
            &workspace,
            "canonical-module",
            optimization,
            &[llvm.as_path(), runtime.as_path()],
        );
        let output = run_command_with_stdin(&mut Command::new(executable), CANONICAL_INPUT);
        assert_eq!(
            output.status.code(),
            Some(91),
            "CAP-049 broke the accepted canonical program at {optimization}"
        );
        assert_eq!(output.stdout, CANONICAL_LLVM.as_bytes());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn canonical_self_host_source_ingests_itself_and_stops_at_the_predicted_construct() {
    let source = fs::read(repository_path(H1A_PRODUCT)).expect("read CAP-049 canonical source");
    let ingested = oracle::ingest(
        &source,
        &oracle::Bounds {
            source: H1A_SOURCE_BOUND,
            token: H1A_TOKEN_BOUND,
            name: H1A_NAME_BOUND,
            ampersand: true,
        },
    );
    assert_eq!(
        ingested.status, 0,
        "the canonical source must lex completely under the frozen bounds"
    );
    assert_eq!(
        ingested.source_length,
        i32::try_from(source.len()).unwrap(),
        "every source byte must be ingested"
    );

    // CAP-049 stopped at the `result` parameter of `fn result_value(...)`.
    // CAP-050 admitted that signature and then stopped one token past `match`,
    // because `match` was reduced to a name-reference operand. CAP-051
    // dispatches on `match` before that reduction runs, so function 1's body
    // completes and the frozen `; } EOF` closing sequence rejects the second
    // `fn` item.
    let superseded = oracle::first_parser_stop(&ingested, &source);
    assert_eq!(
        (superseded.error_offset, superseded.diagnostic_code),
        (16, 11)
    );
    assert_eq!(&source[16..22], b"result");

    let signature = oracle::signature_parser_stop(&ingested, &source);
    assert_eq!(
        (signature.error_offset, signature.diagnostic_code),
        (68, 18)
    );
    assert_eq!(&source[68..74], b"result");

    let stopped = oracle::match_parser_stop(&ingested, &source);
    assert_eq!(stopped.status, 10);
    assert_eq!(stopped.error_offset, 146);
    assert_eq!(stopped.error_line, 8);
    assert_eq!(stopped.error_column, 1);
    assert_eq!(stopped.diagnostic_code, 0);
    assert_eq!(stopped.diagnostic_actual, 3);
    assert_eq!(
        stopped.nodes.len(),
        4,
        "the two arm bodies: `value`, then `0`, `code` and their difference"
    );
    assert_eq!(stopped.origins.len(), stopped.nodes.len());
    assert_eq!(stopped.parameters.len(), 1);
    assert_eq!(
        stopped.parameters[0].1, 2,
        "the one parameter is `Result<int, int>`"
    );
    assert_eq!(&source[146..148], b"fn");

    for optimization in ["-O0", "-O2"] {
        assert_eq!(
            run_expectation(
                "self-ingestion",
                compiled_h1a(),
                &source,
                &stopped,
                optimization
            ),
            91,
            "CAP-051 self-ingestion diverged from the independent oracle at {optimization}"
        );
    }
}

/// CAP-050 / H1B-1 focused signature probes.
///
/// The canonical self-source is a single opaque pass/fail: the entry point
/// returns 91 or 80 and says nothing about which of the 68 values moved. These
/// probes are the smallest complete programs that exercise one signature-grammar
/// rule each, so a sub-machine defect localises to one rule instead of to the
/// whole checkpoint. Each stops inside the parse phase, so every downstream
/// group stays not-attempted and the expectation vector has the same shape as
/// the self-ingestion vector.
///
/// Every expectation is derived by the oracle from the token stream alone; the
/// superseded CAP-049 boundary is asserted alongside it so the checkpoint's
/// movement stays visible.
///
/// The probes link at `-O0` only. `-O0`/`-O2` equivalence for this product is
/// established by the canonical-module and self-ingestion tests.
const SIGNATURE_PROBES: &[(&str, &[u8], i32, i32, i32, &str, usize, usize)] = &[
    // label, source, status, code, actual, token text, parameters, nodes
    (
        "one-int",
        b"fn f(a: int) -> q { return 1; }",
        12,
        102,
        1,
        "q",
        1,
        0,
    ),
    (
        "one-result",
        b"fn f(r: Result<int, int>) -> q { return 1; }",
        12,
        102,
        1,
        "q",
        1,
        0,
    ),
    (
        "two-int",
        b"fn f(a: int, b: int) -> q { return 1; }",
        12,
        102,
        1,
        "q",
        2,
        0,
    ),
    (
        "missing-colon",
        b"fn f(a int) -> int { return 1; }",
        10,
        17,
        1,
        "int",
        0,
        0,
    ),
    (
        "missing-type",
        b"fn f(a: ) -> int { return 1; }",
        10,
        1,
        11,
        ")",
        0,
        0,
    ),
    (
        "unknown-type",
        b"fn f(a: byte) -> int { return 1; }",
        12,
        102,
        1,
        "byte",
        0,
        0,
    ),
    (
        "trailing-comma",
        b"fn f(a: int, ) -> int { return 1; }",
        10,
        1,
        11,
        ")",
        1,
        0,
    ),
    (
        "missing-paren",
        b"fn f(a: int -> int { return 1; }",
        10,
        11,
        35,
        "->",
        1,
        0,
    ),
    (
        "malformed-result",
        b"fn f(r: Result<int>) -> int { return 1; }",
        10,
        16,
        31,
        ">",
        0,
        0,
    ),
    // The self-input target shape at probe scale: one admitted parameter, one
    // name-reference node for the body's leading identifier, and the frozen
    // closing sequence rejecting the identifier that follows it.
    (
        "body-operand",
        b"fn f(a: int) -> int { return z x; }",
        10,
        18,
        1,
        "x",
        1,
        1,
    ),
];

#[test]
fn focused_signature_probes_exercise_every_rule_of_the_admitted_grammar() {
    for (label, source, status, code, actual, text, parameters, nodes) in SIGNATURE_PROBES {
        let ingested = oracle::ingest(
            source,
            &oracle::Bounds {
                source: H1A_SOURCE_BOUND,
                token: H1A_TOKEN_BOUND,
                name: H1A_NAME_BOUND,
                ampersand: true,
            },
        );
        assert_eq!(ingested.status, 0, "probe `{label}` must lex completely");

        // The superseded CAP-049 boundary: the frozen skeleton rejected the
        // first parameter name because it expected `)` immediately after `(`.
        let superseded = oracle::first_parser_stop(&ingested, source);
        assert_eq!(
            (superseded.error_offset, superseded.diagnostic_code),
            (5, 11),
            "probe `{label}` superseded boundary"
        );

        // The CAP-050 target for the same bytes, derived from the token stream.
        let target = oracle::signature_parser_stop(&ingested, source);
        let from = usize::try_from(target.error_offset).expect("bounded offset");
        assert_eq!(target.status, *status, "probe `{label}` target status");
        assert_eq!(target.diagnostic_code, *code, "probe `{label}` target code");
        assert_eq!(
            target.diagnostic_actual, *actual,
            "probe `{label}` target actual"
        );
        assert_eq!(
            &source[from..from + text.len()],
            text.as_bytes(),
            "probe `{label}` target token"
        );
        assert_eq!(
            target.parameters.len(),
            *parameters,
            "probe `{label}` target parameter count"
        );
        assert_eq!(
            target.nodes.len(),
            *nodes,
            "probe `{label}` target node count"
        );
        assert_eq!(
            run_expectation("signature-probe", compiled_h1a(), source, &target, "-O0"),
            91,
            "probe `{label}` diverged from the derived CAP-050 target"
        );
    }
}

/// CAP-050 / H1B-1 target, derived from the canonical token stream alone.
///
/// This target was frozen before the parser admitted a signature, so the
/// implementation could not be graded against its own output. It is retained
/// as the shortest statement of where the checkpoint must stop.
#[test]
fn the_signature_grammar_checkpoint_has_an_independently_derived_target() {
    let source = fs::read(repository_path(H1A_PRODUCT)).expect("read CAP-049 canonical source");
    let ingested = oracle::ingest(
        &source,
        &oracle::Bounds {
            source: H1A_SOURCE_BOUND,
            token: H1A_TOKEN_BOUND,
            name: H1A_NAME_BOUND,
            ampersand: true,
        },
    );
    assert_eq!(ingested.status, 0);

    // Today's frozen skeleton stops at the parameter list.
    let today = oracle::first_parser_stop(&ingested, &source);
    assert_eq!((today.error_offset, today.diagnostic_code), (16, 11));

    // With the signature grammar admitted, the first stop moves into the body:
    // `return match result {` reduces the `match` identifier to one
    // name-reference node, and the frozen `; } EOF` closing sequence rejects the
    // identifier `result` that follows it.
    let target = oracle::signature_grammar_stop(&ingested, &source);
    assert_eq!(
        target,
        oracle::SignatureStop {
            status: 10,
            error_offset: 68,
            error_line: 2,
            error_column: 18,
            diagnostic_code: 18,
            diagnostic_actual: 1,
            node_count: 1,
            parameters: 1,
        },
        "CAP-050's frozen target moved"
    );
    assert_eq!(&source[68..74], b"result");
    assert_eq!(&source[55..61], b"return");

    // The one parameter is `result: Result<int, int>`, the only non-`int`
    // parameter anywhere in the canonical source.
    assert_eq!(&source[16..22], b"result");
    assert_eq!(&source[24..40], b"Result<int, int>");
}

/// CAP-051 / H1B-2 focused match probes.
///
/// The canonical self-source is one opaque pass/fail. These probes are the
/// smallest complete programs that exercise one rule of the admitted match
/// construct each, so a defect localises to one rule instead of to the whole
/// checkpoint. Every expectation is derived by the oracle from the token stream
/// alone, and each is stated here as an independent hand derivation from the
/// frozen grammar so the two must agree.
///
/// The construct is `match IDENT { IDENT ( IDENT ) => EXPR , IDENT ( IDENT )
/// => EXPR , }` in return-expression position only. `match-not-leading` proves
/// the dispatch is position-scoped: away from that position `match` is still an
/// ordinary identifier operand.
const MATCH_PROBES: &[(&str, &[u8], i32, i32, i32, &str, usize, usize)] = &[
    // label, source, status, code, actual, token text, parameters, nodes
    (
        "match-accepted",
        b"fn f(a: int) -> int { return match a { Ok(v) => v, Err(c) => 0 - c, }; } x",
        10,
        0,
        1,
        "x",
        1,
        4,
    ),
    (
        "match-not-leading",
        b"fn f() -> int { return a match; }",
        10,
        18,
        1,
        "match",
        0,
        1,
    ),
    (
        "match-call-scrutinee",
        b"fn f() -> int { return match g(a) { Ok(v) => v, Err(c) => c, }; }",
        10,
        12,
        10,
        "(",
        0,
        0,
    ),
    (
        "match-missing-scrutinee",
        b"fn f() -> int { return match { Ok(v) => v, Err(c) => c, }; }",
        10,
        1,
        12,
        "{",
        0,
        0,
    ),
    (
        "match-literal-pattern",
        b"fn f() -> int { return match a { 1(v) => v, Err(c) => c, }; }",
        10,
        1,
        2,
        "1",
        0,
        0,
    ),
    (
        "match-wildcard-arm",
        b"fn f() -> int { return match a { _ => 1, Err(c) => c, }; }",
        10,
        10,
        36,
        "=>",
        0,
        0,
    ),
    (
        "match-nested-pattern",
        b"fn f() -> int { return match a { Ok(Ok(v)) => v, Err(c) => c, }; }",
        10,
        11,
        10,
        "(",
        0,
        0,
    ),
    (
        "match-guard",
        b"fn f() -> int { return match a { Ok(v) if v => v, Err(c) => c, }; }",
        10,
        36,
        7,
        "if",
        0,
        0,
    ),
    (
        "match-missing-arrow",
        b"fn f() -> int { return match a { Ok(v) v, Err(c) => c, }; }",
        10,
        36,
        1,
        "v",
        0,
        0,
    ),
    (
        "match-empty-arms",
        b"fn f() -> int { return match a { }; }",
        10,
        1,
        13,
        "}",
        0,
        0,
    ),
    (
        "match-single-arm",
        b"fn f() -> int { return match a { Ok(v) => v, }; }",
        10,
        1,
        13,
        "}",
        0,
        1,
    ),
    (
        "match-three-arms",
        b"fn f() -> int { return match a { Ok(v) => v, Err(c) => c, E(d) => d, }; }",
        10,
        13,
        1,
        "E",
        0,
        2,
    ),
    (
        "match-missing-trailing-comma",
        b"fn f() -> int { return match a { Ok(v) => v, Err(c) => c }; }",
        10,
        16,
        13,
        "}",
        0,
        2,
    ),
];

fn match_probe_targets() -> Vec<oracle::Ingestion> {
    let mut targets = Vec::new();
    for (label, source, status, code, actual, text, parameters, nodes) in MATCH_PROBES {
        assert!(
            source.len() < 100,
            "probe `{label}` must stay a small complete program"
        );
        let ingested = oracle::ingest(
            source,
            &oracle::Bounds {
                source: H1A_SOURCE_BOUND,
                token: H1A_TOKEN_BOUND,
                name: H1A_NAME_BOUND,
                ampersand: true,
            },
        );
        assert_eq!(ingested.status, 0, "probe `{label}` must lex completely");

        // No superseded CAP-049 boundary is asserted here: the parameterless
        // probes pass the whole frozen skeleton, so `first_parser_stop` has no
        // stop to report for them. The checkpoint's movement is asserted on the
        // canonical source instead.
        let target = oracle::match_parser_stop(&ingested, source);
        let from = usize::try_from(target.error_offset).expect("bounded offset");
        assert_eq!(target.status, *status, "probe `{label}` target status");
        assert_eq!(target.diagnostic_code, *code, "probe `{label}` target code");
        assert_eq!(
            target.diagnostic_actual, *actual,
            "probe `{label}` target actual"
        );
        assert_eq!(
            &source[from..from + text.len()],
            text.as_bytes(),
            "probe `{label}` target token"
        );
        assert_eq!(
            target.parameters.len(),
            *parameters,
            "probe `{label}` target parameter count"
        );
        assert_eq!(
            target.nodes.len(),
            *nodes,
            "probe `{label}` target node count"
        );
        assert_eq!(
            target.origins.len(),
            target.nodes.len(),
            "probe `{label}` must mirror every node with one origin"
        );
        targets.push(target);
    }
    targets
}

/// Every expectation in [`MATCH_PROBES`] is a hand derivation from the frozen
/// grammar. This test touches no product: it only requires the oracle to agree
/// with all of them.
#[test]
fn every_match_probe_expectation_is_derived_twice() {
    assert_eq!(match_probe_targets().len(), MATCH_PROBES.len());
}

#[test]
fn focused_match_probes_exercise_every_rule_of_the_admitted_construct() {
    for ((label, source, _, _, _, _, _, _), target) in
        MATCH_PROBES.iter().zip(match_probe_targets())
    {
        assert_eq!(
            run_expectation("match-probe", compiled_h1a(), source, &target, "-O0"),
            91,
            "probe `{label}` diverged from the derived CAP-051 target"
        );
    }
}

/// CAP-051 / H1B-2 target, derived from the canonical token stream alone.
///
/// Frozen before the parser dispatched on `match`, so the implementation cannot
/// be graded against its own output.
#[test]
fn the_match_return_checkpoint_has_an_independently_derived_target() {
    let source = fs::read(repository_path(H1A_PRODUCT)).expect("read CAP-049 canonical source");
    let ingested = oracle::ingest(
        &source,
        &oracle::Bounds {
            source: H1A_SOURCE_BOUND,
            token: H1A_TOKEN_BOUND,
            name: H1A_NAME_BOUND,
            ampersand: true,
        },
    );
    assert_eq!(ingested.status, 0);

    // CAP-050's accepted stop, retained so the movement stays visible.
    let superseded = oracle::signature_grammar_stop(&ingested, &source);
    assert_eq!(
        (superseded.error_offset, superseded.diagnostic_code),
        (68, 18)
    );

    // With `match` dispatched before the operand reduction, function 1's body
    // completes and the frozen `; } EOF` closing sequence rejects the second
    // `fn` item, which the canonical bytes place at offset 146, line 8,
    // column 1.
    let target = oracle::match_grammar_stop(&ingested, &source);
    assert_eq!(
        target,
        oracle::SignatureStop {
            status: 10,
            error_offset: 146,
            error_line: 8,
            error_column: 1,
            diagnostic_code: 0,
            diagnostic_actual: 3,
            node_count: 4,
            parameters: 1,
        },
        "CAP-051's frozen target moved"
    );

    // The measured construct: one scrutinee identifier, two arms, a trailing
    // comma on the last arm, and a second arm body the accepted binary
    // expression grammar already parses.
    assert_eq!(&source[62..67], b"match");
    assert_eq!(&source[68..74], b"result");
    assert_eq!(&source[85..94], b"Ok(value)");
    assert_eq!(&source[95..104], b"=> value,");
    assert_eq!(&source[113..122], b"Err(code)");
    assert_eq!(&source[123..135], b"=> 0 - code,");
    assert_eq!(&source[140..144], b"};\n}");
    assert_eq!(&source[146..148], b"fn");
}
