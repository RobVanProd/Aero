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
        assert_eq!(ingested.status, 0, "ingestion must succeed first");
        let mut stopped = ingested.clone();
        stopped.signature_grammar = true;
        stopped.parameters = Vec::new();
        stopped.nodes = Vec::new();
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

        // One leading identifier operand becomes a name-reference node; the
        // frozen closing sequence then requires `;`.
        if ingested.tokens[index][0] == 1 {
            stopped.nodes.push([2, ingested.tokens[index][5], 0, 0]);
            index += 1;
        }
        take!(18); // ;
        panic!("CAP-050 requires this input to stop inside the parse phase");
    }

    /// Where the parser stops once CAP-050 / H1B-1 admits the signature grammar,
    /// projected out of [`signature_parser_stop`].
    pub fn signature_grammar_stop(ingested: &Ingestion, source: &[u8]) -> SignatureStop {
        let stopped = signature_parser_stop(ingested, source);
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
    pub fn unattempted_semantic_checksum() -> i32 {
        let mut checksum = 17;
        for word in [994, 995, 996] {
            checksum = checksum_step(checksum, word);
        }
        for _ in 0..12 {
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
        let semantic = unattempted_semantic_checksum();
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
            // semantic group - never entered
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
    derived.replace(tail_anchor, "        1, 0, 0, -1, 144, 506643, 0);")
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

    let mut stopped = oracle::first_parser_stop(&ingested, &source);
    stopped.signature_grammar = true;
    // The first construct outside the frozen `fn NAME ( ) -> int { return`
    // skeleton is the `result` parameter of `fn result_value(...)`.
    assert_eq!(stopped.status, 10);
    assert_eq!(stopped.error_offset, 16);
    assert_eq!(stopped.error_line, 1);
    assert_eq!(stopped.error_column, 17);
    assert_eq!(stopped.diagnostic_code, 11);
    assert_eq!(stopped.diagnostic_actual, 1);
    assert_eq!(&source[16..22], b"result");

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
            "CAP-049 self-ingestion diverged from the independent oracle at {optimization}"
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
/// Red-first: each probe asserts what the accepted parser does today - it stops
/// at the first parameter name expecting `)` - as real product evidence, and
/// separately states the CAP-050 target derived by the oracle. Nothing here is
/// graded against Aero output.
///
/// The probes link at `-O0` only. `-O0`/`-O2` equivalence for this product is
/// established by the canonical-module and self-ingestion tests.
const SIGNATURE_PROBES: &[(&str, &[u8], i32, i32, i32, &str, usize)] = &[
    // label, source, status, diagnostic code, diagnostic actual, token text, parameters
    (
        "one-int",
        b"fn f(a: int) -> q { return 1; }",
        12,
        102,
        1,
        "q",
        1,
    ),
    (
        "one-result",
        b"fn f(r: Result<int, int>) -> q { return 1; }",
        12,
        102,
        1,
        "q",
        1,
    ),
    (
        "two-int",
        b"fn f(a: int, b: int) -> q { return 1; }",
        12,
        102,
        1,
        "q",
        2,
    ),
    (
        "missing-colon",
        b"fn f(a int) -> int { return 1; }",
        10,
        17,
        1,
        "int",
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
    ),
    (
        "unknown-type",
        b"fn f(a: byte) -> int { return 1; }",
        12,
        102,
        1,
        "byte",
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
    ),
    (
        "missing-paren",
        b"fn f(a: int -> int { return 1; }",
        10,
        11,
        35,
        "->",
        1,
    ),
    (
        "malformed-result",
        b"fn f(r: Result<int>) -> int { return 1; }",
        10,
        16,
        31,
        ">",
        0,
    ),
];

#[test]
fn focused_signature_probes_pin_todays_parser_and_derive_the_checkpoint_targets() {
    for (label, source, status, code, actual, text, parameters) in SIGNATURE_PROBES {
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

        // Today: the accepted parser rejects the first parameter name because
        // the frozen skeleton expects `)` immediately after `(`.
        let mut today = oracle::first_parser_stop(&ingested, source);
        today.signature_grammar = true;
        assert_eq!(today.status, 10, "probe `{label}` today");
        assert_eq!(today.diagnostic_code, 11, "probe `{label}` today");
        assert_eq!(today.diagnostic_actual, 1, "probe `{label}` today");
        assert_eq!(today.error_offset, 5, "probe `{label}` today");
        assert_eq!(
            run_expectation("signature-probe", compiled_h1a(), source, &today, "-O0"),
            91,
            "probe `{label}` diverged from the accepted parser boundary"
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
        assert!(
            target.nodes.is_empty(),
            "probe `{label}` must stop before the body"
        );
    }
}

/// CAP-050 / H1B-1 target, derived here before the parser changes.
///
/// This is not a claim that the parser admits signatures today - the test above
/// proves it still stops at the CAP-049 boundary. It freezes, from the canonical
/// token stream alone, exactly where the next checkpoint must stop, so the
/// implementation cannot be graded against its own output.
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
