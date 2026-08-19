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

    /// CAP-055 / H1B-6. The five parse-group record ceilings, modelled here for
    /// the first time: through CAP-054 the oracle carried no record bound of
    /// any kind, so the product's `status = 14` and `status = 15` were never
    /// predicted by anything.
    ///
    /// These are deliberately *not* fields of [`Bounds`]. `Bounds` is the
    /// ingestion phase's policy and is threaded into `ingest`, which appends no
    /// record; folding parse-phase ceilings into it would make seventeen
    /// unrelated literals carry them. The contract said "extend `Bounds`"; this
    /// is the one place the implementation departs from it, and the departure
    /// is recorded in the ledger rather than smoothed.
    #[derive(Clone, Copy)]
    pub struct Caps {
        pub nodes: usize,
        pub values: usize,
        pub operators: usize,
        pub blocks: usize,
        pub calls: usize,
    }

    impl Caps {
        /// Every model through CAP-054, exactly. No check can fire, so the
        /// bounded parser is the unbounded parser by construction rather than
        /// by inspection.
        pub const UNBOUNDED: Caps = Caps {
            nodes: usize::MAX,
            values: usize::MAX,
            operators: usize::MAX,
            blocks: usize::MAX,
            calls: usize::MAX,
        };

        /// The product's own arrangement: one literal shared by all five.
        pub const fn uniform(bound: usize) -> Caps {
            Caps {
                nodes: bound,
                values: bound,
                operators: bound,
                blocks: bound,
                calls: bound,
            }
        }
    }

    /// A located stop. A grammar reject carries the offending token's own kind
    /// as `actual`; a capacity stop may not, because the product locates
    /// several of its exhaustion diagnostics at a pending operator
    /// (`compiler.aero:2622`, `diagnostic_actual = reduction_actual`) or at a
    /// held callee (`:2191`) rather than at the token being classified.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Reject {
        pub offset: i32,
        pub line: i32,
        pub column: i32,
        pub status: i32,
        pub code: i32,
        pub actual: i32,
    }

    impl Reject {
        /// Every grammar reject the models through CAP-054 produced, unchanged.
        fn token(record: &TokenRecord, status: i32, code: i32) -> Reject {
            Reject {
                offset: record[1],
                line: record[3],
                column: record[4],
                status,
                code,
                actual: record[0],
            }
        }

        /// A record-arena exhaustion. `origin` is the oracle's
        /// `[start, line, column, kind]`, and `diagnostic_code` carries the
        /// bound itself exactly as the product does.
        fn capacity(origin: [i32; 4], status: i32, bound: usize, actual: i32) -> Reject {
            Reject {
                offset: origin[0],
                line: origin[1],
                column: origin[2],
                status,
                code: i32::try_from(bound).expect("bounded record ceiling"),
                actual,
            }
        }
    }

    /// Mirrors the product's counters, which are never decremented: each counts
    /// total pushes over the whole parse, not live depth. A pop rewinds only a
    /// link, and the record arrays have an append path and no write-at-index
    /// path, so an abandoned record is never reused.
    ///
    /// The node arena is absent because it is append-only in the model too and
    /// `nodes.len()` already *is* the product's `node_count` - every accepted
    /// probe grades that equality through the expectation vector.
    #[derive(Default)]
    pub struct Counts {
        pub values: usize,
        pub operators: usize,
        pub blocks: usize,
        pub calls: usize,
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
        parser_stop(
            ingested,
            source,
            false,
            false,
            false,
            false,
            &Caps::UNBOUNDED,
        )
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
        parser_stop(
            ingested,
            source,
            true,
            false,
            false,
            false,
            &Caps::UNBOUNDED,
        )
    }

    /// Where the parser stops once CAP-052 / H1B-3 additionally admits the
    /// statement grammar: a body is `{` followed by one or more statements
    /// followed by `}`, and `;` terminates the return statement rather than
    /// closing the body.
    pub fn statement_parser_stop(ingested: &Ingestion, source: &[u8]) -> Ingestion {
        parser_stop(ingested, source, true, true, false, false, &Caps::UNBOUNDED)
    }

    /// Where the parser stops once CAP-053 / H1B-4 additionally admits the two
    /// control-flow forms `if EXPR BLOCK`, with any number of `else if EXPR
    /// BLOCK` arms and an optional final `else BLOCK`, and `while EXPR BLOCK`.
    ///
    /// A `BLOCK` is `{` followed by one or more statements followed by `}`, so
    /// the body rule CAP-052 froze for the function body now nests. Two rules
    /// move with it. A nested block's closing rule is `}` and nothing more,
    /// while the function body's remains `}` then end-of-input; and the
    /// requirement that a completed `return` exist moves from the block to the
    /// function, so a nested block may close without one. Within any block a
    /// `return` is the last statement and the only one, which is the rule
    /// CAP-052 froze and did not implement.
    ///
    /// Neither form creates a syntax node: an honest `if` node would have to
    /// reference its body, a statement sequence has no representation in the
    /// accepted arena, and a node carrying only its condition would assert at
    /// H1C that the conditional has no body. So the `1..=19` node-kind bound is
    /// untouched, and a nested `return` leaves its expression as an orphan.
    pub fn control_flow_parser_stop(ingested: &Ingestion, source: &[u8]) -> Ingestion {
        parser_stop(ingested, source, true, true, true, false, &Caps::UNBOUNDED)
    }

    /// Where the parser stops once CAP-054 / H1B-5 additionally admits call
    /// expressions and `&` / `&mut` operands - and, for the first time in H1B,
    /// *represents* what it admits.
    ///
    /// A call is `IDENT ( ARGS )` where the callee is an operand-position
    /// identifier immediately followed by `(`, and `ARGS` is empty or one or
    /// more arguments separated by `,` with no trailing `,`. An argument may
    /// begin with `&` or `& mut`, and may do so nowhere else, which is the
    /// measured shape: all 451 references in the canonical source are a whole
    /// call argument.
    ///
    /// Four node kinds are added, taking the node-kind bound to `1..=23`.
    /// Kind 20 is the call, carrying its callee as `payload` and its argument
    /// list as `left`; kind 21 is one argument-list cell, `left` the argument
    /// and `right` the next cell; kinds 22 and 23 are `&` and `&mut` over their
    /// operand. Every node under a call is reachable from the call node, so a
    /// call's whole subtree is as reachable as the call itself.
    pub fn call_parser_stop(ingested: &Ingestion, source: &[u8]) -> Ingestion {
        parser_stop(ingested, source, true, true, true, true, &Caps::UNBOUNDED)
    }

    /// CAP-055 / H1B-6. The same grammar as [`call_parser_stop`] under the five
    /// parse-group record ceilings, which nothing modelled before this
    /// checkpoint.
    ///
    /// This is *not* a new parser. `call_parser_stop` is this function at
    /// [`Caps::UNBOUNDED`], so the CAP-054 model survives as an instance of the
    /// CAP-055 one rather than as a copy that could drift, and the two must
    /// agree on every shape that stays under the bound. That equality is the
    /// product-free proof this checkpoint changed no grammar.
    pub fn capacity_parser_stop(ingested: &Ingestion, source: &[u8], caps: &Caps) -> Ingestion {
        parser_stop(ingested, source, true, true, true, true, caps)
    }

    /// The accepted expression grammar, modelled as the product's own
    /// shunting-yard so nodes append in the product's order.
    ///
    /// `values` holds node ids; the operator stack holds token kinds, with
    /// 103/104 for the prefix forms and 10 for an open parenthesis, each beside
    /// its located origin. The scan stops at the first token that cannot
    /// continue the expression, leaving `index` on it, and returns the root node
    /// id. An error carries the located token and the product's own
    /// `(status, diagnostic_code)` for it.
    fn parse_expression(
        ingested: &Ingestion,
        source: &[u8],
        index: &mut usize,
        nodes: &mut Vec<[i32; 4]>,
        origins: &mut Vec<[i32; 5]>,
        admit_calls: bool,
        caps: &Caps,
        counts: &mut Counts,
    ) -> Result<i32, Reject> {
        /// `compiler.aero` checks `node_count >= BOUND` *before* the append and
        /// locates the stop at the origin the node would have carried, so the
        /// check belongs here rather than at each call site.
        fn append(
            nodes: &mut Vec<[i32; 4]>,
            origins: &mut Vec<[i32; 5]>,
            caps: &Caps,
            kind: i32,
            payload: i32,
            left: i32,
            right: i32,
            origin: [i32; 4],
        ) -> Result<i32, Reject> {
            if nodes.len() >= caps.nodes {
                // `diagnostic_actual` is the origin's own token kind at every
                // reachable site. At the reduction site the product spells this
                // `reduction_actual`, which maps marker 103 to 21, 104 to 27
                // and 106/107 to 37 - exactly the kinds of the tokens that
                // pushed those markers, which is what the origin already holds.
                return Err(Reject::capacity(origin, 14, caps.nodes, origin[3]));
            }
            nodes.push([kind, payload, left, right]);
            let id = i32::try_from(nodes.len()).expect("bounded nodes");
            origins.push([id, origin[0], origin[1], origin[2], origin[3]]);
            Ok(id)
        }
        /// The product increments `node_count` and only then checks
        /// `value_records`, so a value push is guarded after its node append.
        fn push_value(
            values: &mut Vec<i32>,
            counts: &mut Counts,
            caps: &Caps,
            id: i32,
            origin: [i32; 4],
        ) -> Result<(), Reject> {
            if counts.values >= caps.values {
                return Err(Reject::capacity(origin, 15, caps.values, origin[3]));
            }
            counts.values += 1;
            values.push(id);
            Ok(())
        }
        fn push_operator(
            operators: &mut Vec<(i32, [i32; 4])>,
            counts: &mut Counts,
            caps: &Caps,
            marker: i32,
            origin: [i32; 4],
        ) -> Result<(), Reject> {
            if counts.operators >= caps.operators {
                return Err(Reject::capacity(origin, 15, caps.operators, origin[3]));
            }
            counts.operators += 1;
            operators.push((marker, origin));
            Ok(())
        }
        fn reduce_top(
            nodes: &mut Vec<[i32; 4]>,
            origins: &mut Vec<[i32; 5]>,
            values: &mut Vec<i32>,
            operators: &mut Vec<(i32, [i32; 4])>,
            caps: &Caps,
            counts: &mut Counts,
        ) -> Result<(), Reject> {
            let (marker, at) = operators.pop().expect("modelled operator stack");
            if marker == 103 || marker == 104 || marker == 106 || marker == 107 {
                let left = values.pop().expect("modelled operand stack");
                // 103 `-`, 104 `!`, 106 `&`, 107 `&mut`. A reference is a
                // prefix operator of the accepted shunting yard and nothing
                // else, so it reduces here beside the two that already were.
                let kind = match marker {
                    103 => 3,
                    104 => 4,
                    106 => 22,
                    _ => 23,
                };
                let id = append(nodes, origins, caps, kind, 0, left, 0, at)?;
                push_value(values, counts, caps, id, at)?;
            } else {
                let right = values.pop().expect("modelled operand stack");
                let left = values.pop().expect("modelled operand stack");
                let id = append(
                    nodes,
                    origins,
                    caps,
                    binary_node_kind(marker),
                    0,
                    left,
                    right,
                    at,
                )?;
                push_value(values, counts, caps, id, at)?;
            }
            Ok(())
        }

        /// Close the innermost call: turn the values above its base into a
        /// kind-21 chain, then append the kind-20 call node over it.
        ///
        /// The node arena is append-only and has no write-at-index path, so a
        /// cell cannot be back-patched to point at its successor and the chain
        /// must be built from its last element. Popping the value stack yields
        /// the arguments in reverse, which is exactly the order that needs, so
        /// the finished chain's head is the *first* argument.
        fn close_call(
            nodes: &mut Vec<[i32; 4]>,
            origins: &mut Vec<[i32; 5]>,
            values: &mut Vec<i32>,
            operators: &mut Vec<(i32, [i32; 4])>,
            calls: &mut Vec<(i32, usize)>,
            paren_depth: &mut i32,
            close: [i32; 4],
            caps: &Caps,
            counts: &mut Counts,
        ) -> Result<(), Reject> {
            let (callee, base) = calls.pop().expect("modelled call stack");
            let mut chain = 0i32;
            while values.len() > base {
                let argument = values.pop().expect("modelled operand stack");
                // `compiler.aero:3085` guards the argument cell at the closing
                // `)`, which is the origin the cell carries.
                chain = append(nodes, origins, caps, 21, 0, argument, chain, close)?;
            }
            let (marker, at) = operators.pop().expect("modelled operator stack");
            assert_eq!(marker, 105, "a call closes on its own marker");
            *paren_depth -= 1;
            // The callee is the call node's payload and never becomes a
            // name-reference node: `f` in `f(x)` is not a value read of `f`,
            // and the self-source has no first-class functions.
            //
            // `compiler.aero:3158` guards this node at `top_start`, the marker's
            // own origin, which is `at`.
            let id = append(nodes, origins, caps, 20, callee, chain, 0, at)?;
            push_value(values, counts, caps, id, at)?;
            Ok(())
        }

        let mut values: Vec<i32> = Vec::new();
        let mut operators: Vec<(i32, [i32; 4])> = Vec::new();
        // CAP-054 / H1B-5. One entry per open call: the callee's name id, and
        // the value-stack depth its arguments sit above. The base is how the
        // closing `)` knows how many arguments it has, without a counter that
        // an append-only record could not carry.
        let mut calls: Vec<(i32, usize)> = Vec::new();
        let mut paren_depth = 0i32;
        let mut expecting_operand = true;
        // Set by a call's `(` and by an argument-separating `,`, and cleared by
        // every other classified token. `&` is admissible only while it is set,
        // which is the whole of the measured restriction that a reference is
        // always a whole call argument.
        let mut arg_leading = false;
        loop {
            let record = ingested.tokens[*index];
            let origin = [record[1], record[3], record[4], record[0]];
            let leading = arg_leading;
            arg_leading = false;
            if expecting_operand {
                // `IDENT (` opens a call. The flat product has no lookahead and
                // holds the identifier for one iteration instead; the model
                // reaches the same result by looking at the next record, which
                // always exists because `record[0] == 1` excludes end of input.
                if admit_calls && record[0] == 1 && ingested.tokens[*index + 1][0] == 10 {
                    let open = ingested.tokens[*index + 1];
                    // `compiler.aero:2191` is one check over both stores -
                    // `call_records >= B || operator_records >= B` - located at
                    // the *held callee* with `diagnostic_actual` taken from the
                    // `(` that ended the hold. Under a uniform bound the two
                    // trip together; under a non-uniform one the model reports
                    // whichever ceiling was actually reached, and the call store
                    // is checked first because the product names it first.
                    if counts.calls >= caps.calls || counts.operators >= caps.operators {
                        let bound = if counts.calls >= caps.calls {
                            caps.calls
                        } else {
                            caps.operators
                        };
                        return Err(Reject::capacity(
                            [record[1], record[3], record[4], record[0]],
                            15,
                            bound,
                            open[0],
                        ));
                    }
                    counts.calls += 1;
                    counts.operators += 1;
                    calls.push((record[5], values.len()));
                    operators.push((105, [open[1], open[3], open[4], open[0]]));
                    paren_depth += 1;
                    *index += 2;
                    arg_leading = true;
                    continue;
                }
                if admit_calls && record[0] == 37 && leading {
                    let mut marker = 106;
                    *index += 1;
                    if ingested.tokens[*index][0] == 5 {
                        marker = 107;
                        *index += 1;
                    }
                    // `compiler.aero:2161`, located at the `&` with
                    // `diagnostic_actual = 37`, which is that token's own kind.
                    push_operator(&mut operators, counts, caps, marker, origin)?;
                    continue;
                }
                // A `)` is an operand-position token only as a zero-argument
                // call's close, which is `leading` with nothing yet pushed.
                // After a `,` the base no longer matches, so `f(a,)` is the
                // ordinary operand rejection.
                if admit_calls
                    && record[0] == 11
                    && leading
                    && calls.last().map(|call| call.1) == Some(values.len())
                {
                    close_call(
                        nodes,
                        origins,
                        &mut values,
                        &mut operators,
                        &mut calls,
                        &mut paren_depth,
                        origin,
                        caps,
                        counts,
                    )?;
                    expecting_operand = false;
                    *index += 1;
                    continue;
                }
                if record[0] == 1 {
                    let id = append(nodes, origins, caps, 2, record[5], 0, 0, origin)?;
                    push_value(&mut values, counts, caps, id, origin)?;
                } else if record[0] == 2 {
                    let from = usize::try_from(record[1]).expect("bounded start");
                    let to = from + usize::try_from(record[2]).expect("bounded length");
                    let mut literal = 0i32;
                    for byte in &source[from..to] {
                        literal = literal * 10 + i32::from(byte - b'0');
                    }
                    let id = append(nodes, origins, caps, 1, literal, 0, 0, origin)?;
                    push_value(&mut values, counts, caps, id, origin)?;
                } else if record[0] == 21 || record[0] == 27 || record[0] == 10 {
                    let marker = match record[0] {
                        21 => 103,
                        27 => 104,
                        _ => 10,
                    };
                    // `compiler.aero:2263`, located at the current token with
                    // its own kind. The grouping `(` is counted here and the
                    // check precedes the depth increment, so an exhausted
                    // operator store never leaves a half-opened group.
                    push_operator(&mut operators, counts, caps, marker, origin)?;
                    if marker == 10 {
                        paren_depth += 1;
                    }
                    *index += 1;
                    continue;
                } else {
                    return Err(Reject::token(&record, 11, 100));
                }
                expecting_operand = false;
                *index += 1;
                continue;
            }

            let precedence = binary_precedence(record[0]);
            if precedence > 0 {
                while let Some(&(marker, _)) = operators.last() {
                    if marker == 10 || marker == 105 {
                        break;
                    }
                    let top = if marker == 103 || marker == 104 || marker == 106 || marker == 107 {
                        7
                    } else {
                        binary_precedence(marker)
                    };
                    if top < precedence {
                        break;
                    }
                    reduce_top(nodes, origins, &mut values, &mut operators, caps, counts)?;
                }
                // `compiler.aero:2705`, located at the operator token itself.
                push_operator(&mut operators, counts, caps, record[0], origin)?;
                expecting_operand = true;
                *index += 1;
                continue;
            }
            // An argument separator reduces to the innermost marker and
            // requires it to be a call's. A `,` inside a grouping keeps the
            // accepted `diagnostic_code = 11`.
            if admit_calls && record[0] == 16 && paren_depth > 0 {
                loop {
                    let marker = operators.last().expect("modelled operator stack").0;
                    if marker == 10 {
                        return Err(Reject::token(&record, 10, 11));
                    }
                    if marker == 105 {
                        break;
                    }
                    reduce_top(nodes, origins, &mut values, &mut operators, caps, counts)?;
                }
                *index += 1;
                expecting_operand = true;
                arg_leading = true;
                continue;
            }
            if record[0] == 11 && paren_depth > 0 {
                loop {
                    let marker = operators.last().expect("modelled operator stack").0;
                    if marker == 10 {
                        operators.pop();
                        paren_depth -= 1;
                        break;
                    }
                    if marker == 105 {
                        close_call(
                            nodes,
                            origins,
                            &mut values,
                            &mut operators,
                            &mut calls,
                            &mut paren_depth,
                            origin,
                            caps,
                            counts,
                        )?;
                        break;
                    }
                    reduce_top(nodes, origins, &mut values, &mut operators, caps, counts)?;
                }
                *index += 1;
                continue;
            }
            if paren_depth > 0 {
                return Err(Reject::token(&record, 10, 11));
            }
            while !operators.is_empty() {
                reduce_top(nodes, origins, &mut values, &mut operators, caps, counts)?;
            }
            assert_eq!(values.len(), 1, "one expression reduces to one value");
            return Ok(values[0]);
        }
    }

    fn parser_stop(
        ingested: &Ingestion,
        source: &[u8],
        admit_match: bool,
        admit_statements: bool,
        admit_control_flow: bool,
        admit_calls: bool,
        caps: &Caps,
    ) -> Ingestion {
        assert_eq!(ingested.status, 0, "ingestion must succeed first");
        let mut counts = Counts::default();
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
        /// CAP-055. A stop that already knows its own location and
        /// `diagnostic_actual`, which a capacity stop does and a grammar reject
        /// does not.
        macro_rules! reject_located {
            ($reject:expr) => {{
                let reject: Reject = $reject;
                stopped.status = reject.status;
                stopped.error_offset = reject.offset;
                stopped.error_line = reject.line;
                stopped.error_column = reject.column;
                stopped.diagnostic_code = reject.code;
                stopped.diagnostic_actual = reject.actual;
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

        macro_rules! append {
            ($kind:expr, $payload:expr, $left:expr, $right:expr, $origin:expr) => {{
                let origin: [i32; 4] = $origin;
                if stopped.nodes.len() >= caps.nodes {
                    reject_located!(Reject::capacity(origin, 14, caps.nodes, origin[3]));
                }
                stopped.nodes.push([$kind, $payload, $left, $right]);
                let id = i32::try_from(stopped.nodes.len()).expect("bounded nodes");
                stopped
                    .origins
                    .push([id, origin[0], origin[1], origin[2], origin[3]]);
                id
            }};
        }

        // CAP-052 / H1B-3. The body is `{` followed by one or more statements
        // followed by `}`. The four admitted statement forms are
        // `let IDENT : int = EXPR ;`, `let mut IDENT : int = EXPR ;`,
        // `IDENT = EXPR ;`, and `return EXPR ;`. `;` is the return statement's
        // own terminator rather than a closing token, so the two entry points
        // CAP-051 created into the closing sequence collapse into one rule
        // inside the loop and the closing sequence shrinks to `}` then
        // end-of-input, entered once.
        //
        // A statement produces no syntax node, exactly as a CAP-050 parameter
        // does not, so the body's tree is still one return node over the last
        // completed return statement's expression. A binding's or an
        // assignment's initializer nodes are therefore orphans, joining the
        // four CAP-051 left.
        if admit_statements {
            // CAP-053 / H1B-4 adds the block stack the two control-flow forms
            // need. A record is pushed for every *nested* block and never for
            // the function body, so an empty stack means the function body and
            // its closing rule stays exactly where CAP-052 put it.
            //
            // A record holds the block's kind - 1 an `if` body, 2 a `while`
            // body, 3 an `else` body - and the enclosing block's statement
            // state, restored on pop. `block_state` is 0 before the block's
            // first statement, 1 once one has completed, and 2 once a `return`
            // has, which is the whole of the per-block return rule: a statement
            // opener at state 2 is rejected, and a `}` at state 0 is an empty
            // block.
            let mut body_root = 0i32;
            let mut blocks: Vec<(i32, i32)> = Vec::new();
            let mut block_state = 0i32;
            let mut else_admissible = false;
            loop {
                let leading = ingested.tokens[index];
                let else_opens = admit_control_flow && else_admissible && leading[0] == 8;
                else_admissible = false;
                // 1 `let`, 2 assignment, 3 `return`, 4 `if`, 5 `while`.
                let mut mode = 0i32;
                if else_opens {
                    index += 1; // else
                    let next = ingested.tokens[index];
                    if next[0] == 7 {
                        index += 1; // `if`, continuing the chain
                        mode = 4;
                    } else if next[0] == 12 {
                        index += 1; // `{`, opening the final `else` body
                        // `compiler.aero:1972`, located at the token that opens
                        // the block with that token's own kind.
                        if counts.blocks >= caps.blocks {
                            reject_located!(Reject::capacity(
                                [next[1], next[3], next[4], next[0]],
                                15,
                                caps.blocks,
                                next[0],
                            ));
                        }
                        counts.blocks += 1;
                        blocks.push((3, block_state));
                        block_state = 0;
                        continue;
                    } else {
                        reject!(&next, 10, 12);
                    }
                } else {
                    if leading[0] == 4 {
                        mode = 1;
                    } else if leading[0] == 1 {
                        mode = 2;
                    } else if leading[0] == 6 {
                        mode = 3;
                    } else if admit_control_flow && leading[0] == 7 {
                        mode = 4;
                    } else if admit_control_flow && leading[0] == 9 {
                        mode = 5;
                    }
                    if mode == 0 {
                        if blocks.is_empty() {
                            break;
                        }
                        // A nested block closes on `}` and nothing more, and
                        // carries no return requirement. Its one extra rule is
                        // that it may not be empty, reported the way CAP-052
                        // reports an empty function body.
                        let expected_close = if block_state == 0 { 6 } else { 13 };
                        if leading[0] != expected_close {
                            reject!(&leading, 10, expected_close);
                        }
                        index += 1; // }
                        let (kind, enclosing) = blocks.pop().expect("modelled block stack");
                        block_state = enclosing;
                        else_admissible = kind == 1;
                        continue;
                    }
                    if admit_control_flow && block_state == 2 {
                        // The rule CAP-052 froze and did not implement: after a
                        // return statement's `;` the only admissible token is
                        // that block's `}`.
                        reject!(&leading, 10, 13);
                    }
                    index += 1; // the statement's leading token
                    if mode == 1 {
                        if ingested.tokens[index][0] == 5 {
                            index += 1; // mut
                        }
                        take!(1); // the bound name
                        take!(17); // :
                        let ty = take!(1);
                        if text(ty) != b"int" {
                            reject!(ty, 12, 102);
                        }
                        take!(25); // =
                    } else if mode == 2 {
                        take!(25); // =
                    }
                }
                block_state = 1;

                // A return expression dispatches on its leading token exactly as
                // CAP-051 does; a binding's, an assignment's or a condition's
                // expression does not, so `match` there is an ordinary
                // identifier operand.
                let mut root = 0i32;
                let opening = ingested.tokens[index];
                if mode == 3 && opening[0] == 1 && text(&opening) == b"match" {
                    index += 1; // match
                    take!(1); // the scrutinee is exactly one identifier
                    take!(12); // {
                    for _ in 0..2 {
                        take!(1); // the pattern head
                        take!(10); // (
                        take!(1); // the bound identifier
                        take!(11); // )
                        take!(36); // =>
                        let arm = parse_expression(
                            ingested,
                            source,
                            &mut index,
                            &mut stopped.nodes,
                            &mut stopped.origins,
                            admit_calls,
                            caps,
                            &mut counts,
                        );
                        match arm {
                            Ok(value) => root = value,
                            Err(reject) => reject_located!(reject),
                        }
                        take!(16); // the arm's trailing `,`
                    }
                    take!(13); // the match construct's closing `}`
                } else {
                    let value = parse_expression(
                        ingested,
                        source,
                        &mut index,
                        &mut stopped.nodes,
                        &mut stopped.origins,
                        admit_calls,
                        caps,
                        &mut counts,
                    );
                    match value {
                        Ok(value) => root = value,
                        Err(reject) => reject_located!(reject),
                    }
                }
                // The statement's terminator is decided by what the expression
                // was for: `;` for a binding, an assignment or a return, and
                // `{` for a condition, whose block record is pushed as that `{`
                // is accepted.
                let terminator = if mode >= 4 { 12 } else { 18 };
                let opener = ingested.tokens[index];
                take!(terminator);
                if mode == 3 {
                    // `body_root` stays one register and the last write wins.
                    // Every block's `return` is its last statement and every
                    // function body ends in one, so the last `return` completed
                    // in token order within a function is always the function
                    // body's own. A nested `return` leaves an orphan.
                    body_root = root;
                    block_state = 2;
                }
                if mode >= 4 {
                    // `compiler.aero:1972`, located at the `{` that opens the
                    // block, which is the token just accepted.
                    if counts.blocks >= caps.blocks {
                        reject_located!(Reject::capacity(
                            [opener[1], opener[3], opener[4], opener[0]],
                            15,
                            caps.blocks,
                            opener[0],
                        ));
                    }
                    counts.blocks += 1;
                    blocks.push((if mode == 4 { 1 } else { 2 }, block_state));
                    block_state = 0;
                }
            }

            // The closing sequence: `}` then end-of-input, entered once. A body
            // that closes without a completed return statement has no root for
            // the function node, so `}` is rejected there with the statement
            // expectation instead - which is also how an empty body is rejected.
            // The function body's own closing rule is unchanged: `}` then
            // end-of-input, and the requirement that a `return` completed is
            // now the *function's* rather than any block's, so a nested block
            // that returned does not satisfy it.
            //
            // `body_root` is the last-write-wins register Decision 3 keeps, and
            // no probe observes it: the return node is appended only after
            // end-of-input is accepted, which no probe of this checkpoint
            // reaches. Its one relationship with the function's requirement is
            // asserted rather than assumed.
            assert!(
                block_state != 2 || body_root > 0,
                "a function body that returned always has a root"
            );
            //
            // The two spellings of that requirement differ only on inputs
            // CAP-053 rejects. `block_state == 2` says the *function body*
            // returned; `body_root > 0` says *some* block did. Under CAP-053
            // they coincide, because a statement can never follow a completed
            // `return` and no nested block can be the last thing a function
            // body does. Under CAP-052 they do not - its product admits a
            // statement after a return, which clears the block state - so the
            // CAP-052 model keeps the spelling that models its own product.
            let close = &ingested.tokens[index];
            let expected_close = if admit_control_flow {
                if block_state == 2 { 13 } else { 6 }
            } else if body_root > 0 {
                13
            } else {
                6
            };
            if close[0] != expected_close {
                reject!(close, 10, expected_close);
            }
            index += 1;
            let end = &ingested.tokens[index];
            if end[0] != 0 {
                reject!(end, 10, 0);
            }
            panic!("this checkpoint requires the input to stop inside the parse phase");
        }

        take!(6); // return

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

                // The arm body is the accepted expression grammar, modelled by
                // `parse_expression` as the product's own shunting-yard so nodes
                // append in the product's order.
                let arm = parse_expression(
                    ingested,
                    source,
                    &mut index,
                    &mut stopped.nodes,
                    &mut stopped.origins,
                    admit_calls,
                    caps,
                    &mut counts,
                );
                if let Err(reject) = arm {
                    reject_located!(reject);
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

    /// Where the parser stops once CAP-052 / H1B-3 admits the statement
    /// grammar, projected out of [`statement_parser_stop`].
    pub fn statement_grammar_stop(ingested: &Ingestion, source: &[u8]) -> SignatureStop {
        project(&statement_parser_stop(ingested, source))
    }

    /// Where the parser stops once CAP-053 / H1B-4 admits the two control-flow
    /// forms, projected out of [`control_flow_parser_stop`].
    pub fn control_flow_grammar_stop(ingested: &Ingestion, source: &[u8]) -> SignatureStop {
        project(&control_flow_parser_stop(ingested, source))
    }

    /// Where the parser stops once CAP-054 / H1B-5 admits and represents calls
    /// and references, projected out of [`call_parser_stop`].
    pub fn call_grammar_stop(ingested: &Ingestion, source: &[u8]) -> SignatureStop {
        project(&call_parser_stop(ingested, source))
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

/// CAP-052's statement grammar, inserted into the CAP-051 parser.
///
/// The Aero product and this reconstruction are patched from one shared
/// definition, so the admitted grammar and its byte-for-byte derivation cannot
/// drift apart.
const SKELETON_RETURN_STEP_ANCHOR: &str = r#"            if skeleton_step == 7 {
                expected_kind = 6;
            }
            // CAP-050 signature grammar."#;
const SKELETON_RETURN_STEP: &str = r#"            // CAP-052 dissolves the skeleton's fixed `return` step into the
            // statement loop, so the skeleton ends at the body's '{'.
            // CAP-050 signature grammar."#;
const STATEMENT_ENTRY_ANCHOR: &str = r#"            if parser_running == 1 && skeleton_step == 7 {
                return_start = current_start;
                return_line = current_line;
                return_column = current_column;
            }
            if parser_running == 1 {
                parse_index = parse_index + 1;
                if param_hold == 0 {
                    skeleton_step = skeleton_step + 1;
                }
                parser_state = 1;
                if skeleton_step == 8 {
                    parser_state = 40;
                }
            }"#;
const STATEMENT_ENTRY: &str = r#"            if parser_running == 1 {
                parse_index = parse_index + 1;
                if param_hold == 0 {
                    skeleton_step = skeleton_step + 1;
                }
                parser_state = 1;
                if skeleton_step == 7 {
                    parser_state = 44;
                }
            }"#;
const STATEMENT_REGISTERS_ANCHOR: &str = r#"    let mut match_b4: int = 0;"#;
const STATEMENT_REGISTERS: &str = r#"    let mut match_b4: int = 0;
    let mut statement_mode: int = 0;
    let mut stmt_step: int = 0;
    let mut stmt_cycle_step: int = 0;
    let mut stmt_expected: int = 0;
    let mut stmt_alternate: int = 0;
    let mut stmt_is_int: int = 0;
    let mut stmt_b0: int = 0;
    let mut stmt_b1: int = 0;
    let mut stmt_b2: int = 0;
    let mut body_root: int = 0;"#;
const STATEMENT_REQUESTS_ANCHOR: &str = r#"        if parser_cycle_state == 42 {
            parser_token_after = 43;
            parser_token_field = 0;
            parser_token_byte = 0;
            parser_token_word = 0;
            parser_state = 30;
        }"#;
const STATEMENT_REQUESTS: &str = r#"        if parser_cycle_state == 42 {
            parser_token_after = 43;
            parser_token_field = 0;
            parser_token_byte = 0;
            parser_token_word = 0;
            parser_state = 30;
        }
        // CAP-052 requests the leading token of each statement, one token per
        // step of a binding or an assignment, and the statement's own ';'.
        if parser_cycle_state == 44 {
            parser_token_after = 45;
            parser_token_field = 0;
            parser_token_byte = 0;
            parser_token_word = 0;
            parser_state = 30;
        }
        if parser_cycle_state == 46 {
            parser_token_after = 47;
            parser_token_field = 0;
            parser_token_byte = 0;
            parser_token_word = 0;
            parser_state = 30;
        }
        if parser_cycle_state == 48 {
            parser_token_after = 49;
            parser_token_field = 0;
            parser_token_byte = 0;
            parser_token_word = 0;
            parser_state = 30;
        }"#;
const STATEMENT_STATES_ANCHOR: &str =
    r#"        // CAP-051 decides the return expression on its leading token, before"#;
const STATEMENT_STATES: &str = r#"        // CAP-052 statement loop. A body is one or more statements followed by
        // '}'. Each statement is dispatched on its own leading token: 'let'
        // opens a binding, an identifier opens an assignment, and 'return'
        // opens a return statement. Any other token hands the same decoded
        // record to the closing sequence, which is entered from here and only
        // from here. A statement creates no node, so the arena vocabulary and
        // the '1..=19' node-kind bound are unchanged.
        if parser_cycle_state == 45 {
            if current_kind < 0 || current_start < 0 || current_length < 0
                || current_line <= 0 || current_column <= 0 {
                status = 16;
                parser_running = 0;
            }
            if parser_running == 1 {
                statement_mode = 0;
                if current_kind == 4 {
                    statement_mode = 1;
                    stmt_step = 0;
                }
                if current_kind == 1 {
                    statement_mode = 2;
                    stmt_step = 4;
                }
                if current_kind == 6 {
                    statement_mode = 3;
                    return_start = current_start;
                    return_line = current_line;
                    return_column = current_column;
                }
                if statement_mode == 0 {
                    closing_step = 0;
                    parser_state = 21;
                } else {
                    parse_index = parse_index + 1;
                    value_top = 0;
                    value_depth = 0;
                    expecting_operand = 1;
                    parser_state = 46;
                    if statement_mode == 3 {
                        parser_state = 40;
                    }
                }
            }
        }

        // One step of 'let [mut] IDENT : int =' or of 'IDENT ='. Step 0 admits
        // 'mut' as its alternate, step 3 requires the binding type to be exactly
        // 'int', and step 4 hands the initializer to the accepted expression
        // grammar. An assignment enters at step 4.
        if parser_cycle_state == 47 {
            stmt_cycle_step = stmt_step;
            stmt_expected = 1;
            stmt_alternate = 0;
            if stmt_cycle_step == 0 {
                stmt_alternate = 5;
            }
            if stmt_cycle_step == 2 {
                stmt_expected = 17;
            }
            if stmt_cycle_step == 4 {
                stmt_expected = 25;
            }
            if current_kind < 0 || current_start < 0 || current_length < 0
                || current_line <= 0 || current_column <= 0 {
                status = 16;
                parser_running = 0;
            }
            if parser_running == 1 && current_kind != stmt_expected
                && (stmt_alternate == 0 || current_kind != stmt_alternate) {
                status = 10;
                error_offset = current_start;
                error_line = current_line;
                error_column = current_column;
                diagnostic_code = stmt_expected;
                diagnostic_actual = current_kind;
                parser_running = 0;
            }
            if parser_running == 1 && stmt_cycle_step == 3 {
                stmt_is_int = 0;
                if current_length == 3 {
                    stmt_b0 = result_value(bytes_get(&source, current_start));
                    stmt_b1 = result_value(bytes_get(&source, current_start + 1));
                    stmt_b2 = result_value(bytes_get(&source, current_start + 2));
                    if stmt_b0 == 105 && stmt_b1 == 110 && stmt_b2 == 116 {
                        stmt_is_int = 1;
                    }
                }
                if stmt_is_int == 0 {
                    status = 12;
                    error_offset = current_start;
                    error_line = current_line;
                    error_column = current_column;
                    diagnostic_code = 102;
                    diagnostic_actual = current_kind;
                    parser_running = 0;
                }
            }
            if parser_running == 1 {
                parse_index = parse_index + 1;
                stmt_step = stmt_cycle_step + 1;
                if stmt_cycle_step == 0 && current_kind == 1 {
                    stmt_step = 2;
                }
                parser_state = 46;
                if stmt_cycle_step == 4 {
                    parser_state = 3;
                }
            }
        }

        // ';' terminates the statement rather than closing the body. It is the
        // one rule the ordinary return expression and the CAP-051 match
        // construct both return to, so their two entry points into the closing
        // sequence collapse into this one.
        if parser_cycle_state == 49 {
            if current_kind < 0 || current_start < 0 || current_line <= 0
                || current_column <= 0 {
                status = 16;
                parser_running = 0;
            }
            if parser_running == 1 && current_kind != 18 {
                status = 10;
                error_offset = current_start;
                error_line = current_line;
                error_column = current_column;
                diagnostic_code = 18;
                diagnostic_actual = current_kind;
                parser_running = 0;
            }
            if parser_running == 1 {
                parse_index = parse_index + 1;
                if statement_mode == 3 {
                    body_root = expression_root;
                }
                statement_mode = 0;
                parser_state = 44;
            }
        }

        // CAP-051 decides the return expression on its leading token, before"#;
const EXPRESSION_RETURN_ANCHOR: &str = r#"            if parser_running == 1 && match_active == 0 {
                closing_step = 0;
                parser_state = 20;
            }"#;
const EXPRESSION_RETURN: &str = r#"            if parser_running == 1 && match_active == 0 {
                parser_state = 48;
            }"#;
const MATCH_CLOSE_ANCHOR: &str = r#"                if match_cycle_step == 14 {
                    match_active = 0;
                    closing_step = 0;
                    parser_state = 20;
                }"#;
const MATCH_CLOSE: &str = r#"                if match_cycle_step == 14 {
                    match_active = 0;
                    parser_state = 48;
                }"#;
const CLOSING_SUFFIX_ANCHOR: &str = r#"        // Exact '; } EOF' suffix.
        if parser_cycle_state == 21 {
            expected_kind = 18;
            if closing_step == 1 {
                expected_kind = 13;
            }
            if closing_step == 2 {
                expected_kind = 0;
            }"#;
const CLOSING_SUFFIX: &str = r#"        // Exact '} EOF' suffix, entered once from the statement loop. A body
        // that closes without a completed return statement has no root for the
        // function node, so its '}' is rejected with the statement expectation
        // instead - which is also how an empty body is rejected.
        if parser_cycle_state == 21 {
            expected_kind = 13;
            if body_root <= 0 {
                expected_kind = 6;
            }
            if closing_step == 1 {
                expected_kind = 0;
            }"#;
const CLOSING_ADVANCE_ANCHOR: &str = r#"                closing_step = closing_step + 1;
                parser_state = 20;
                if closing_step == 3 {"#;
const CLOSING_ADVANCE: &str = r#"                closing_step = closing_step + 1;
                parser_state = 20;
                if closing_step == 2 {"#;
const BODY_ROOT_ANCHOR: &str = r#"                        pending_node_left = expression_root;"#;
const BODY_ROOT: &str = r#"                        pending_node_left = body_root;"#;

/// Apply the six frozen CAP-049 ingestion differences to the accepted B1C
/// source. `compiler.aero` must equal this byte for byte.
/// CAP-053 / H1B-4 admits the two control-flow forms. The Aero product and
/// this reconstruction are patched from one shared definition, so the admitted
/// grammar and its byte-for-byte derivation cannot drift apart.
const BLOCK_OWNER_ANCHOR: &str = r#"    let mut parameters: ByteBuffer = bytes_new();"#;
const BLOCK_OWNER: &str = r#"    let mut parameters: ByteBuffer = bytes_new();
    let mut blocks: ByteBuffer = bytes_new();"#;
const BLOCK_REGISTERS_ANCHOR: &str = r#"    let mut body_root: int = 0;"#;
const BLOCK_REGISTERS: &str = r#"    let mut body_root: int = 0;
    let mut block_state: int = 0;
    let mut block_top: int = 0;
    let mut block_depth: int = 0;
    let mut block_records: int = 0;
    let mut block_else: int = 0;
    let mut block_kind_push: int = 0;
    let mut block_kind_popped: int = 0;
    let mut block_state_saved: int = 0;
    let mut block_previous: int = 0;
    let mut stmt_is_else: int = 0;
    let mut stmt_close_expected: int = 0;
    let mut stmt_terminator: int = 0;"#;
const ELSE_REQUEST_ANCHOR: &str = r#"        if parser_cycle_state == 48 {
            parser_token_after = 49;
            parser_token_field = 0;
            parser_token_byte = 0;
            parser_token_word = 0;
            parser_state = 30;
        }"#;
const ELSE_REQUEST: &str = r#"        if parser_cycle_state == 48 {
            parser_token_after = 49;
            parser_token_field = 0;
            parser_token_byte = 0;
            parser_token_word = 0;
            parser_state = 30;
        }
        // CAP-053 requests the one token after an 'else'.
        if parser_cycle_state == 50 {
            parser_token_after = 51;
            parser_token_field = 0;
            parser_token_byte = 0;
            parser_token_word = 0;
            parser_state = 30;
        }"#;
const CONTROL_FLOW_DISPATCH_ANCHOR: &str = r#"            if parser_running == 1 {
                statement_mode = 0;
                if current_kind == 4 {
                    statement_mode = 1;
                    stmt_step = 0;
                }
                if current_kind == 1 {
                    statement_mode = 2;
                    stmt_step = 4;
                }
                if current_kind == 6 {
                    statement_mode = 3;
                    return_start = current_start;
                    return_line = current_line;
                    return_column = current_column;
                }
                if statement_mode == 0 {
                    closing_step = 0;
                    parser_state = 21;
                } else {
                    parse_index = parse_index + 1;
                    value_top = 0;
                    value_depth = 0;
                    expecting_operand = 1;
                    parser_state = 46;
                    if statement_mode == 3 {
                        parser_state = 40;
                    }
                }
            }
        }"#;
const CONTROL_FLOW_DISPATCH: &str = r#"            // CAP-053 admits 'else' here rather than as a statement: it is
            // the one token that may follow a nested 'if' body's '}', and
            // 'block_else' is set only by that pop.
            if parser_running == 1 {
                stmt_is_else = 0;
                if current_kind == 8 && block_else == 1 {
                    stmt_is_else = 1;
                }
                block_else = 0;
                if stmt_is_else == 1 {
                    parse_index = parse_index + 1;
                    parser_state = 50;
                }
            }
            if parser_running == 1 && stmt_is_else == 0 {
                statement_mode = 0;
                if current_kind == 4 {
                    statement_mode = 1;
                    stmt_step = 0;
                }
                if current_kind == 1 {
                    statement_mode = 2;
                    stmt_step = 4;
                }
                if current_kind == 6 {
                    statement_mode = 3;
                    return_start = current_start;
                    return_line = current_line;
                    return_column = current_column;
                }
                if current_kind == 7 {
                    statement_mode = 4;
                }
                if current_kind == 9 {
                    statement_mode = 5;
                }
                // The rule CAP-052 froze and did not implement: after a return
                // statement's ';' the only admissible token is this block's
                // '}'. 'block_state' is 0 before the block's first statement,
                // 1 once one has completed, and 2 once a 'return' has.
                if statement_mode > 0 && block_state == 2 {
                    status = 10;
                    error_offset = current_start;
                    error_line = current_line;
                    error_column = current_column;
                    diagnostic_code = 13;
                    diagnostic_actual = current_kind;
                    parser_running = 0;
                }
                if parser_running == 1 && statement_mode == 0 && block_top == 0 {
                    closing_step = 0;
                    parser_state = 21;
                }
                // A nested block closes on '}' and nothing more, and carries no
                // return requirement. Its one extra rule is that it may not be
                // empty, reported the way an empty function body is.
                if parser_running == 1 && statement_mode == 0 && block_top > 0 {
                    stmt_close_expected = 13;
                    if block_state == 0 {
                        stmt_close_expected = 6;
                    }
                    if current_kind != stmt_close_expected {
                        status = 10;
                        error_offset = current_start;
                        error_line = current_line;
                        error_column = current_column;
                        diagnostic_code = stmt_close_expected;
                        diagnostic_actual = current_kind;
                        parser_running = 0;
                    }
                    if parser_running == 1 {
                        parser_record_target = 3;
                        parser_record_width = 3;
                        parser_record_index = block_top;
                        parser_record_field = 0;
                        parser_record_byte = 0;
                        parser_record_word = 0;
                        parser_record_0 = 0;
                        parser_record_1 = 0;
                        parser_record_2 = 0;
                        parser_record_3 = 0;
                        parser_record_4 = 0;
                        parser_record_after = 54;
                        parser_state = 31;
                    }
                }
                if parser_running == 1 && statement_mode > 0 {
                    block_state = 1;
                    parse_index = parse_index + 1;
                    value_top = 0;
                    value_depth = 0;
                    expecting_operand = 1;
                    parser_state = 46;
                    if statement_mode == 3 {
                        parser_state = 40;
                    }
                    if statement_mode >= 4 {
                        parser_state = 3;
                    }
                }
            }
        }"#;
const STATEMENT_TERMINATOR_ANCHOR: &str = r#"        if parser_cycle_state == 49 {
            if current_kind < 0 || current_start < 0 || current_line <= 0
                || current_column <= 0 {
                status = 16;
                parser_running = 0;
            }
            if parser_running == 1 && current_kind != 18 {
                status = 10;
                error_offset = current_start;
                error_line = current_line;
                error_column = current_column;
                diagnostic_code = 18;
                diagnostic_actual = current_kind;
                parser_running = 0;
            }
            if parser_running == 1 {
                parse_index = parse_index + 1;
                if statement_mode == 3 {
                    body_root = expression_root;
                }
                statement_mode = 0;
                parser_state = 44;
            }
        }"#;
const STATEMENT_TERMINATOR: &str = r#"        if parser_cycle_state == 49 {
            stmt_terminator = 18;
            if statement_mode >= 4 {
                stmt_terminator = 12;
            }
            if current_kind < 0 || current_start < 0 || current_line <= 0
                || current_column <= 0 {
                status = 16;
                parser_running = 0;
            }
            if parser_running == 1 && current_kind != stmt_terminator {
                status = 10;
                error_offset = current_start;
                error_line = current_line;
                error_column = current_column;
                diagnostic_code = stmt_terminator;
                diagnostic_actual = current_kind;
                parser_running = 0;
            }
            if parser_running == 1 {
                parse_index = parse_index + 1;
                // 'body_root' stays one register and the last write wins. Every
                // block's 'return' is its last statement and every function
                // body ends in one, so the last 'return' completed in token
                // order within a function is always the function body's own.
                if statement_mode == 3 {
                    body_root = expression_root;
                    block_state = 2;
                }
                block_kind_push = 0;
                if statement_mode == 4 {
                    block_kind_push = 1;
                }
                if statement_mode == 5 {
                    block_kind_push = 2;
                }
                statement_mode = 0;
                parser_state = 44;
                if block_kind_push > 0 {
                    parser_state = 52;
                }
            }
        }

        // An 'else' continues the chain with another 'if', or opens the final
        // 'else' body. Nothing else is admissible after it.
        if parser_cycle_state == 51 {
            if current_kind < 0 || current_start < 0 || current_length < 0
                || current_line <= 0 || current_column <= 0 {
                status = 16;
                parser_running = 0;
            }
            if parser_running == 1 && current_kind != 12 && current_kind != 7 {
                status = 10;
                error_offset = current_start;
                error_line = current_line;
                error_column = current_column;
                diagnostic_code = 12;
                diagnostic_actual = current_kind;
                parser_running = 0;
            }
            if parser_running == 1 {
                parse_index = parse_index + 1;
                block_state = 1;
                if current_kind == 7 {
                    statement_mode = 4;
                    value_top = 0;
                    value_depth = 0;
                    expecting_operand = 1;
                    parser_state = 3;
                } else {
                    statement_mode = 0;
                    block_kind_push = 3;
                    parser_state = 52;
                }
            }
        }

        // Push one block record: the block's kind - 1 an 'if' body, 2 a 'while'
        // body, 3 an 'else' body - the enclosing block's statement state, and
        // the link to the enclosing record. Only a nested block pushes, so a
        // 'block_top' of zero is the function body and its closing rule is
        // unchanged. The store is a fourth monotonic parse-group counter and
        // carries the same bound and the same exhaustion diagnostic as the
        // value and operator stores.
        if parser_cycle_state == 52 {
            if block_records >= 512 {
                status = 15;
                error_offset = current_start;
                error_line = current_line;
                error_column = current_column;
                diagnostic_code = 512;
                diagnostic_actual = current_kind;
                parser_running = 0;
            }
            if parser_running == 1 {
                parser_append_target = 5;
                parser_append_width = 3;
                parser_append_0 = block_kind_push;
                parser_append_1 = block_state;
                parser_append_2 = block_top;
                parser_append_3 = 0;
                parser_append_4 = 0;
                parser_append_field = 0;
                parser_append_byte = 0;
                parser_append_after = 53;
                parser_append_offset = current_start;
                parser_append_line = current_line;
                parser_append_column = current_column;
                parser_state = 32;
            }
        }
        if parser_cycle_state == 53 {
            block_records = block_records + 1;
            block_top = block_records;
            block_depth = block_depth + 1;
            block_state = 0;
            parser_state = 44;
        }

        // The block record is decoded; pop it, restore the enclosing block's
        // statement state, and admit 'else' only after an 'if' body.
        if parser_cycle_state == 54 {
            block_kind_popped = parser_record_0;
            block_state_saved = parser_record_1;
            block_previous = parser_record_2;
            if block_kind_popped <= 0 || block_kind_popped > 3
                || block_state_saved < 0 || block_state_saved > 2 {
                status = 16;
                parser_running = 0;
            }
            if parser_running == 1 && (block_previous < 0 || block_depth <= 0
                || block_previous >= block_top) {
                status = 16;
                parser_running = 0;
            }
            if parser_running == 1 {
                block_top = block_previous;
                block_depth = block_depth - 1;
                block_state = block_state_saved;
                block_else = 0;
                if block_kind_popped == 1 {
                    block_else = 1;
                }
                parse_index = parse_index + 1;
                parser_state = 44;
            }
        }"#;
const FUNCTION_RETURN_REQUIREMENT_ANCHOR: &str = r#"            expected_kind = 13;
            if body_root <= 0 {
                expected_kind = 6;
            }"#;
const FUNCTION_RETURN_REQUIREMENT: &str = r#"            expected_kind = 13;
            if block_state != 2 {
                expected_kind = 6;
            }"#;
const BLOCK_RECORD_READ_ANCHOR: &str = r#"                if parser_record_target == 2 {
                    parser_read_byte_value = result_value(bytes_get(&operators,
                        parser_read_offset));
                }
                if parser_record_target != 1 && parser_record_target != 2 {
                    status = 16;
                    parser_running = 0;
                }"#;
const BLOCK_RECORD_READ: &str = r#"                if parser_record_target == 2 {
                    parser_read_byte_value = result_value(bytes_get(&operators,
                        parser_read_offset));
                }
                if parser_record_target == 3 {
                    parser_read_byte_value = result_value(bytes_get(&blocks,
                        parser_read_offset));
                }
                if parser_record_target != 1 && parser_record_target != 2
                    && parser_record_target != 3 {
                    status = 16;
                    parser_running = 0;
                }"#;
const BLOCK_RECORD_APPEND_ANCHOR: &str = r#"            if parser_running == 1 && parser_append_target == 4 {
                push_result = result_value(bytes_push(&mut origins,
                    parser_read_byte_value));
            }
            if parser_running == 1 && parser_append_target != 1
                && parser_append_target != 2 && parser_append_target != 3
                && parser_append_target != 4 {
                status = 16;
                parser_running = 0;
            }"#;
const BLOCK_RECORD_APPEND: &str = r#"            if parser_running == 1 && parser_append_target == 4 {
                push_result = result_value(bytes_push(&mut origins,
                    parser_read_byte_value));
            }
            if parser_running == 1 && parser_append_target == 5 {
                push_result = result_value(bytes_push(&mut blocks,
                    parser_read_byte_value));
            }
            if parser_running == 1 && parser_append_target != 1
                && parser_append_target != 2 && parser_append_target != 3
                && parser_append_target != 4 && parser_append_target != 5 {
                status = 16;
                parser_running = 0;
            }"#;
const BLOCK_STORAGE_INVARIANT_ANCHOR: &str = r#"    if bytes_len(&nodes) < node_count * 16 || bytes_len(&values) < value_records * 8
        || bytes_len(&operators) < operator_records * 20 {
        return 70;
    }
    if status == 0 && (bytes_len(&nodes) != node_count * 16
        || bytes_len(&values) != value_records * 8
        || bytes_len(&operators) != operator_records * 20) {
        return 70;
    }"#;
const BLOCK_STORAGE_INVARIANT: &str = r#"    if bytes_len(&nodes) < node_count * 16 || bytes_len(&values) < value_records * 8
        || bytes_len(&operators) < operator_records * 20
        || bytes_len(&blocks) < block_records * 12 {
        return 70;
    }
    if status == 0 && (bytes_len(&nodes) != node_count * 16
        || bytes_len(&values) != value_records * 8
        || bytes_len(&operators) != operator_records * 20
        || bytes_len(&blocks) != block_records * 12) {
        return 70;
    }"#;

// CAP-054 admits and represents call expressions and `&` / `&mut` operands:
// a fifth bounded parse-group arena for the open calls, an operand classifier
// that holds an identifier until it knows whether a `(` follows, an argument
// separator and a call close in the accepted shunting yard, and four node kinds
// that take the node-kind bound to `1..=23`. This is the first H1B checkpoint
// whose construct is represented rather than only admitted.
const CALL_OWNER_ANCHOR: &str = r#"    let mut blocks: ByteBuffer = bytes_new();"#;
const CALL_OWNER: &str = r#"    let mut blocks: ByteBuffer = bytes_new();
    let mut calls: ByteBuffer = bytes_new();"#;
const CALL_REGISTERS_ANCHOR: &str = r#"    let mut stmt_terminator: int = 0;"#;
const CALL_REGISTERS: &str = r#"    let mut stmt_terminator: int = 0;
    let mut expression_dispatch: int = 0;
    let mut held_active: int = 0;
    let mut held_name: int = 0;
    let mut held_start: int = 0;
    let mut held_line: int = 0;
    let mut held_column: int = 0;
    let mut ref_pending: int = 0;
    let mut ref_marker: int = 0;
    let mut ref_advance: int = 0;
    let mut ref_start: int = 0;
    let mut ref_line: int = 0;
    let mut ref_column: int = 0;
    let mut arg_leading: int = 0;
    let mut arg_open: int = 0;
    let mut call_top: int = 0;
    let mut call_base: int = 0;
    let mut call_records: int = 0;
    let mut call_callee: int = 0;
    let mut call_chain: int = 0;
    let mut call_argument: int = 0;"#;
const CALL_CLASSIFIER_ANCHOR: &str = r#"            if parser_running == 1 && expecting_operand == 1
                && (current_kind == 1 || current_kind == 2) {
                if current_kind == 1 {
                    literal_value = current_name_id;
                    reduced_kind = 2;
                    if literal_value <= 0 {
                        status = 16;
                        parser_running = 0;
                    }
                    if parser_running == 1 && node_count >= 512 {
                        status = 14;
                        error_offset = current_start;
                        error_line = current_line;
                        error_column = current_column;
                        diagnostic_code = 512;
                        diagnostic_actual = current_kind;
                        parser_running = 0;
                    }
                    if parser_running == 1 {
                        pending_node_kind = reduced_kind;
                        pending_node_payload = literal_value;
                        pending_node_left = 0;
                        pending_node_right = 0;
                        pending_node_after = 12;
                        pending_node_offset = current_start;
                        pending_node_line = current_line;
                        pending_node_column = current_column;
                        parser_append_target = 4;
                        parser_append_width = 5;
                        parser_append_0 = node_count + 1;
                        parser_append_1 = current_start;
                        parser_append_2 = current_line;
                        parser_append_3 = current_column;
                        parser_append_4 = current_kind;
                        parser_append_field = 0;
                        parser_append_byte = 0;
                        parser_append_after = 33;
                        parser_append_offset = current_start;
                        parser_append_line = current_line;
                        parser_append_column = current_column;
                        parser_state = 32;
                    }
                }
                if current_kind == 2 {
                    literal_value = 0;
                    decimal_index = 0;
                    reduced_kind = 1;
                    parser_state = 11;
                }
            }
            if parser_running == 1 && expecting_operand == 1
                && (current_kind == 21 || current_kind == 27 || current_kind == 10) {"#;
const CALL_CLASSIFIER: &str = r#"            // CAP-054 resolves two deferrals before the accepted classifier
            // runs. An identifier operand is *held* rather than appended,
            // because the flat parser has no lookahead and a callee must not
            // become a name-reference node; and a '&' is held for one iteration
            // to see whether 'mut' follows. 'expression_dispatch' records that
            // one of the new branches decided this token, so every accepted
            // branch below stays exactly where CAP-050 through CAP-053 put it.
            if parser_running == 1 {
                arg_open = arg_leading;
                arg_leading = 0;
                expression_dispatch = 0;
            }
            if parser_running == 1 && ref_pending == 1 {
                expression_dispatch = 1;
                ref_pending = 0;
                ref_marker = 106;
                ref_advance = 0;
                if current_kind == 5 {
                    ref_marker = 107;
                    ref_advance = 1;
                }
                if operator_records >= 512 {
                    status = 15;
                    error_offset = ref_start;
                    error_line = ref_line;
                    error_column = ref_column;
                    diagnostic_code = 512;
                    diagnostic_actual = 37;
                    parser_running = 0;
                }
                if parser_running == 1 {
                    parser_append_target = 3;
                    parser_append_width = 5;
                    parser_append_0 = ref_marker;
                    parser_append_1 = ref_start;
                    parser_append_2 = ref_line;
                    parser_append_3 = ref_column;
                    parser_append_4 = operator_top;
                    parser_append_field = 0;
                    parser_append_byte = 0;
                    parser_append_after = 27;
                    parser_append_offset = ref_start;
                    parser_append_line = ref_line;
                    parser_append_column = ref_column;
                    parser_state = 32;
                }
            }
            if parser_running == 1 && expression_dispatch == 0 && held_active == 1 {
                expression_dispatch = 1;
                parser_state = 24;
                if current_kind == 10 {
                    if call_records >= 512 || operator_records >= 512 {
                        status = 15;
                        error_offset = held_start;
                        error_line = held_line;
                        error_column = held_column;
                        diagnostic_code = 512;
                        diagnostic_actual = current_kind;
                        parser_running = 0;
                    }
                    if parser_running == 1 {
                        parser_append_target = 6;
                        parser_append_width = 3;
                        parser_append_0 = held_name;
                        parser_append_1 = call_base;
                        parser_append_2 = call_top;
                        parser_append_3 = 0;
                        parser_append_4 = 0;
                        parser_append_field = 0;
                        parser_append_byte = 0;
                        parser_append_after = 28;
                        parser_append_offset = current_start;
                        parser_append_line = current_line;
                        parser_append_column = current_column;
                        parser_state = 32;
                    }
                }
            }
            if parser_running == 1 && expression_dispatch == 0
                && expecting_operand == 1 && current_kind == 37 && arg_open == 1 {
                expression_dispatch = 1;
                ref_pending = 1;
                ref_start = current_start;
                ref_line = current_line;
                ref_column = current_column;
                parse_index = parse_index + 1;
                parser_state = 3;
            }
            if parser_running == 1 && expression_dispatch == 0
                && expecting_operand == 1 && current_kind == 11 && arg_open == 1
                && call_top > 0 && value_top == call_base {
                expression_dispatch = 1;
                call_chain = 0;
                reduction_mode = 2;
                parser_state = 5;
            }
            if parser_running == 1 && expression_dispatch == 0
                && expecting_operand == 1 && current_kind == 1 {
                expression_dispatch = 1;
                if current_name_id <= 0 {
                    status = 16;
                    parser_running = 0;
                }
                if parser_running == 1 {
                    held_active = 1;
                    held_name = current_name_id;
                    held_start = current_start;
                    held_line = current_line;
                    held_column = current_column;
                    expecting_operand = 0;
                    parse_index = parse_index + 1;
                    parser_state = 3;
                }
            }
            if parser_running == 1 && expression_dispatch == 0
                && expecting_operand == 1 && current_kind == 2 {
                literal_value = 0;
                decimal_index = 0;
                reduced_kind = 1;
                parser_state = 11;
            }
            if parser_running == 1 && expression_dispatch == 0 && expecting_operand == 1
                && (current_kind == 21 || current_kind == 27 || current_kind == 10) {"#;
const CALL_OPERATOR_POSITION_ANCHOR: &str = r#"            if parser_running == 1 && expecting_operand == 1
                && current_kind != 1 && current_kind != 2 && current_kind != 21
                && current_kind != 27 && current_kind != 10 {
                status = 11;
                error_offset = current_start;
                error_line = current_line;
                error_column = current_column;
                diagnostic_code = 100;
                diagnostic_actual = current_kind;
                parser_running = 0;
            }
            if parser_running == 1 && expecting_operand == 0 {
                current_precedence = binary_precedence(current_kind);
                reduction_mode = 0;
                if current_precedence > 0 {
                    reduction_mode = 1;
                }
                if current_kind == 11 && paren_depth > 0 {
                    reduction_mode = 2;
                }
                if current_precedence == 0 && current_kind != 11 && paren_depth > 0 {"#;
const CALL_OPERATOR_POSITION: &str = r#"            if parser_running == 1 && expression_dispatch == 0 && expecting_operand == 1
                && current_kind != 1 && current_kind != 2 && current_kind != 21
                && current_kind != 27 && current_kind != 10 {
                status = 11;
                error_offset = current_start;
                error_line = current_line;
                error_column = current_column;
                diagnostic_code = 100;
                diagnostic_actual = current_kind;
                parser_running = 0;
            }
            if parser_running == 1 && expression_dispatch == 0 && expecting_operand == 0 {
                current_precedence = binary_precedence(current_kind);
                reduction_mode = 0;
                if current_precedence > 0 {
                    reduction_mode = 1;
                }
                if current_kind == 11 && paren_depth > 0 {
                    reduction_mode = 2;
                    call_chain = 0;
                }
                if current_kind == 16 && paren_depth > 0 {
                    reduction_mode = 4;
                }
                if current_precedence == 0 && current_kind != 11 && current_kind != 16
                    && paren_depth > 0 {"#;
const CALL_REDUCE_WALK_ANCHOR: &str = r#"            if parser_running == 1 && top_kind == 10 {
                parser_state = 6;
            }
            if parser_running == 1 && top_kind != 10 {
                top_precedence = binary_precedence(top_kind);
                if top_kind == 103 || top_kind == 104 {
                    top_precedence = 7;
                }"#;
const CALL_REDUCE_WALK: &str = r#"            if parser_running == 1 && (top_kind == 10 || top_kind == 105) {
                parser_state = 6;
            }
            if parser_running == 1 && top_kind != 10 && top_kind != 105 {
                top_precedence = binary_precedence(top_kind);
                if top_kind == 103 || top_kind == 104 || top_kind == 106
                    || top_kind == 107 {
                    top_precedence = 7;
                }"#;
const CALL_UNARY_REDUCE_ANCHOR: &str = r#"                if top_kind == 103 || top_kind == 104 {
                    left_id = right_id;
                    right_id = 0;
                    reduced_kind = 3;
                    if top_kind == 104 {
                        reduced_kind = 4;
                    }
                    parser_state = 9;
                }
                if top_kind != 103 && top_kind != 104 {"#;
const CALL_UNARY_REDUCE: &str = r#"                if top_kind == 103 || top_kind == 104 || top_kind == 106
                    || top_kind == 107 {
                    left_id = right_id;
                    right_id = 0;
                    reduced_kind = 3;
                    if top_kind == 104 {
                        reduced_kind = 4;
                    }
                    if top_kind == 106 {
                        reduced_kind = 22;
                    }
                    if top_kind == 107 {
                        reduced_kind = 23;
                    }
                    parser_state = 9;
                }
                if top_kind != 103 && top_kind != 104 && top_kind != 106
                    && top_kind != 107 {"#;
const CALL_REDUCTION_ACTUAL_ANCHOR: &str = r#"            reduction_actual = top_kind;
            if top_kind == 103 {
                reduction_actual = 21;
            }
            if top_kind == 104 {
                reduction_actual = 27;
            }"#;
const CALL_REDUCTION_ACTUAL: &str = r#"            reduction_actual = top_kind;
            if top_kind == 103 {
                reduction_actual = 21;
            }
            if top_kind == 104 {
                reduction_actual = 27;
            }
            if top_kind == 106 || top_kind == 107 {
                reduction_actual = 37;
            }"#;
const CALL_CLOSE_DISPATCH_ANCHOR: &str = r#"            if parser_running == 1 && reduction_mode == 2 {
                if top_kind != 10 || top_previous < 0 || operator_depth <= 0
                    || paren_depth <= 0 {
                    status = 16;
                    parser_running = 0;
                }
                if parser_running == 1 {
                    operator_top = top_previous;
                    operator_depth = operator_depth - 1;
                    paren_depth = paren_depth - 1;
                    parse_index = parse_index + 1;
                    parser_state = 3;
                }
            }"#;
const CALL_CLOSE_DISPATCH: &str = r#"            if parser_running == 1 && reduction_mode == 2 {
                if (top_kind != 10 && top_kind != 105) || top_previous < 0
                    || operator_depth <= 0 || paren_depth <= 0 {
                    status = 16;
                    parser_running = 0;
                }
                if parser_running == 1 && top_kind == 105 {
                    parser_state = 34;
                }
                if parser_running == 1 && top_kind == 10 {
                    operator_top = top_previous;
                    operator_depth = operator_depth - 1;
                    paren_depth = paren_depth - 1;
                    parse_index = parse_index + 1;
                    parser_state = 3;
                }
            }
            // An argument separator reduces to the innermost marker and
            // requires it to be a call's. A ',' inside a grouping keeps the
            // accepted diagnostic.
            if parser_running == 1 && reduction_mode == 4 {
                if top_kind != 105 {
                    status = 10;
                    error_offset = current_start;
                    error_line = current_line;
                    error_column = current_column;
                    diagnostic_code = 11;
                    diagnostic_actual = current_kind;
                    parser_running = 0;
                }
                if parser_running == 1 {
                    parse_index = parse_index + 1;
                    expecting_operand = 1;
                    arg_leading = 1;
                    parser_state = 3;
                }
            }"#;
const CALL_STATES_ANCHOR: &str =
    r#"        // The origin is complete; append the frozen four-word node next."#;
const CALL_STATES: &str = r#"        // CAP-054. A held identifier turned out to be an ordinary operand,
        // so its kind-2 node is appended now, one iteration later than before
        // and in the same order, with the same payload and the same origin.
        if parser_cycle_state == 24 {
            if node_count >= 512 {
                status = 14;
                error_offset = held_start;
                error_line = held_line;
                error_column = held_column;
                diagnostic_code = 512;
                diagnostic_actual = 1;
                parser_running = 0;
            }
            if parser_running == 1 {
                pending_node_kind = 2;
                pending_node_payload = held_name;
                pending_node_left = 0;
                pending_node_right = 0;
                pending_node_after = 25;
                pending_node_offset = held_start;
                pending_node_line = held_line;
                pending_node_column = held_column;
                parser_append_target = 4;
                parser_append_width = 5;
                parser_append_0 = node_count + 1;
                parser_append_1 = held_start;
                parser_append_2 = held_line;
                parser_append_3 = held_column;
                parser_append_4 = 1;
                parser_append_field = 0;
                parser_append_byte = 0;
                parser_append_after = 33;
                parser_append_offset = held_start;
                parser_append_line = held_line;
                parser_append_column = held_column;
                parser_state = 32;
            }
        }
        if parser_cycle_state == 25 {
            node_count = node_count + 1;
            node_id = node_count;
            if value_records >= 512 {
                status = 15;
                error_offset = held_start;
                error_line = held_line;
                error_column = held_column;
                diagnostic_code = 512;
                diagnostic_actual = 1;
                parser_running = 0;
            }
            if parser_running == 1 {
                parser_append_target = 2;
                parser_append_width = 2;
                parser_append_0 = node_id;
                parser_append_1 = value_top;
                parser_append_2 = 0;
                parser_append_3 = 0;
                parser_append_4 = 0;
                parser_append_field = 0;
                parser_append_byte = 0;
                parser_append_after = 26;
                parser_append_offset = held_start;
                parser_append_line = held_line;
                parser_append_column = held_column;
                parser_state = 32;
            }
        }
        // The hold is discharged. The token that ended it has not been
        // consumed, so the classifier is re-entered on it rather than the
        // decoder.
        if parser_cycle_state == 26 {
            value_records = value_records + 1;
            value_top = value_records;
            value_depth = value_depth + 1;
            held_active = 0;
            parser_state = 4;
        }
        // A reference marker record is complete. When 'mut' was consumed the
        // operand has not been decoded yet; otherwise it already has been.
        if parser_cycle_state == 27 {
            operator_records = operator_records + 1;
            operator_top = operator_records;
            operator_depth = operator_depth + 1;
            parser_state = 4;
            if ref_advance == 1 {
                parse_index = parse_index + 1;
                parser_state = 3;
            }
        }
        // A call record is complete; push the call's operator marker at its
        // '(' so the reduce walk stops there and the call node is located
        // there.
        if parser_cycle_state == 28 {
            call_records = call_records + 1;
            call_top = call_records;
            call_base = value_top;
            held_active = 0;
            parser_append_target = 3;
            parser_append_width = 5;
            parser_append_0 = 105;
            parser_append_1 = current_start;
            parser_append_2 = current_line;
            parser_append_3 = current_column;
            parser_append_4 = operator_top;
            parser_append_field = 0;
            parser_append_byte = 0;
            parser_append_after = 29;
            parser_append_offset = current_start;
            parser_append_line = current_line;
            parser_append_column = current_column;
            parser_state = 32;
        }
        if parser_cycle_state == 29 {
            operator_records = operator_records + 1;
            operator_top = operator_records;
            operator_depth = operator_depth + 1;
            paren_depth = paren_depth + 1;
            expecting_operand = 1;
            arg_leading = 1;
            parse_index = parse_index + 1;
            parser_state = 3;
        }
        // Close a call. The node arena is append-only and has no write-at-index
        // path, so a cell cannot be back-patched to point at its successor and
        // the chain is built from its last element: popping the value stack to
        // the call's base yields the arguments in reverse, which is exactly the
        // order that needs, and the finished chain's head is the first
        // argument.
        if parser_cycle_state == 34 {
            if value_top < call_base || value_depth < 0 {
                status = 16;
                parser_running = 0;
            }
            if parser_running == 1 && value_top == call_base {
                parser_state = 37;
            }
            if parser_running == 1 && value_top != call_base {
                parser_record_target = 1;
                parser_record_width = 2;
                parser_record_index = value_top;
                parser_record_field = 0;
                parser_record_byte = 0;
                parser_record_word = 0;
                parser_record_0 = 0;
                parser_record_1 = 0;
                parser_record_2 = 0;
                parser_record_3 = 0;
                parser_record_4 = 0;
                parser_record_after = 35;
                parser_state = 31;
            }
        }
        if parser_cycle_state == 35 {
            call_argument = parser_record_0;
            value_previous = parser_record_1;
            if call_argument <= 0 || value_previous < 0 || value_depth <= 0 {
                status = 16;
                parser_running = 0;
            }
            if parser_running == 1 && node_count >= 512 {
                status = 14;
                error_offset = current_start;
                error_line = current_line;
                error_column = current_column;
                diagnostic_code = 512;
                diagnostic_actual = current_kind;
                parser_running = 0;
            }
            if parser_running == 1 {
                value_top = value_previous;
                value_depth = value_depth - 1;
                pending_node_kind = 21;
                pending_node_payload = 0;
                pending_node_left = call_argument;
                pending_node_right = call_chain;
                pending_node_after = 36;
                pending_node_offset = current_start;
                pending_node_line = current_line;
                pending_node_column = current_column;
                parser_append_target = 4;
                parser_append_width = 5;
                parser_append_0 = node_count + 1;
                parser_append_1 = current_start;
                parser_append_2 = current_line;
                parser_append_3 = current_column;
                parser_append_4 = 11;
                parser_append_field = 0;
                parser_append_byte = 0;
                parser_append_after = 33;
                parser_append_offset = current_start;
                parser_append_line = current_line;
                parser_append_column = current_column;
                parser_state = 32;
            }
        }
        if parser_cycle_state == 36 {
            node_count = node_count + 1;
            call_chain = node_count;
            parser_state = 34;
        }
        if parser_cycle_state == 37 {
            if call_top <= 0 {
                status = 16;
                parser_running = 0;
            }
            if parser_running == 1 {
                parser_record_target = 4;
                parser_record_width = 3;
                parser_record_index = call_top;
                parser_record_field = 0;
                parser_record_byte = 0;
                parser_record_word = 0;
                parser_record_0 = 0;
                parser_record_1 = 0;
                parser_record_2 = 0;
                parser_record_3 = 0;
                parser_record_4 = 0;
                parser_record_after = 38;
                parser_state = 31;
            }
        }
        // The callee is the call node's payload and never became a
        // name-reference node: 'f' in 'f(x)' is not a value read of 'f'.
        if parser_cycle_state == 38 {
            call_callee = parser_record_0;
            call_base = parser_record_1;
            call_top = parser_record_2;
            if call_callee <= 0 || call_callee > name_count || call_base < 0
                || call_top < 0 {
                status = 16;
                parser_running = 0;
            }
            if parser_running == 1 && node_count >= 512 {
                status = 14;
                error_offset = top_start;
                error_line = top_line;
                error_column = top_column;
                diagnostic_code = 512;
                diagnostic_actual = 10;
                parser_running = 0;
            }
            if parser_running == 1 {
                operator_top = top_previous;
                operator_depth = operator_depth - 1;
                paren_depth = paren_depth - 1;
                pending_node_kind = 20;
                pending_node_payload = call_callee;
                pending_node_left = call_chain;
                pending_node_right = 0;
                pending_node_after = 39;
                pending_node_offset = top_start;
                pending_node_line = top_line;
                pending_node_column = top_column;
                parser_append_target = 4;
                parser_append_width = 5;
                parser_append_0 = node_count + 1;
                parser_append_1 = top_start;
                parser_append_2 = top_line;
                parser_append_3 = top_column;
                parser_append_4 = 10;
                parser_append_field = 0;
                parser_append_byte = 0;
                parser_append_after = 33;
                parser_append_offset = top_start;
                parser_append_line = top_line;
                parser_append_column = top_column;
                parser_state = 32;
            }
        }
        if parser_cycle_state == 39 {
            node_count = node_count + 1;
            node_id = node_count;
            if value_records >= 512 {
                status = 15;
                error_offset = top_start;
                error_line = top_line;
                error_column = top_column;
                diagnostic_code = 512;
                diagnostic_actual = 10;
                parser_running = 0;
            }
            if parser_running == 1 {
                parser_append_target = 2;
                parser_append_width = 2;
                parser_append_0 = node_id;
                parser_append_1 = value_top;
                parser_append_2 = 0;
                parser_append_3 = 0;
                parser_append_4 = 0;
                parser_append_field = 0;
                parser_append_byte = 0;
                parser_append_after = 19;
                parser_append_offset = top_start;
                parser_append_line = top_line;
                parser_append_column = top_column;
                parser_state = 32;
            }
        }
        if parser_cycle_state == 19 {
            value_records = value_records + 1;
            value_top = value_records;
            value_depth = value_depth + 1;
            parse_index = parse_index + 1;
            expecting_operand = 0;
            parser_state = 3;
        }

        // The origin is complete; append the frozen four-word node next."#;
const CALL_RECORD_READ_ANCHOR: &str = r#"                if parser_record_target == 3 {
                    parser_read_byte_value = result_value(bytes_get(&blocks,
                        parser_read_offset));
                }
                if parser_record_target != 1 && parser_record_target != 2
                    && parser_record_target != 3 {
                    status = 16;
                    parser_running = 0;
                }"#;
const CALL_RECORD_READ: &str = r#"                if parser_record_target == 3 {
                    parser_read_byte_value = result_value(bytes_get(&blocks,
                        parser_read_offset));
                }
                if parser_record_target == 4 {
                    parser_read_byte_value = result_value(bytes_get(&calls,
                        parser_read_offset));
                }
                if parser_record_target != 1 && parser_record_target != 2
                    && parser_record_target != 3 && parser_record_target != 4 {
                    status = 16;
                    parser_running = 0;
                }"#;
const CALL_RECORD_APPEND_ANCHOR: &str = r#"            if parser_running == 1 && parser_append_target == 5 {
                push_result = result_value(bytes_push(&mut blocks,
                    parser_read_byte_value));
            }
            if parser_running == 1 && parser_append_target != 1
                && parser_append_target != 2 && parser_append_target != 3
                && parser_append_target != 4 && parser_append_target != 5 {
                status = 16;
                parser_running = 0;
            }"#;
const CALL_RECORD_APPEND: &str = r#"            if parser_running == 1 && parser_append_target == 5 {
                push_result = result_value(bytes_push(&mut blocks,
                    parser_read_byte_value));
            }
            if parser_running == 1 && parser_append_target == 6 {
                push_result = result_value(bytes_push(&mut calls,
                    parser_read_byte_value));
            }
            if parser_running == 1 && parser_append_target != 1
                && parser_append_target != 2 && parser_append_target != 3
                && parser_append_target != 4 && parser_append_target != 5
                && parser_append_target != 6 {
                status = 16;
                parser_running = 0;
            }"#;
const CALL_STORAGE_INVARIANT_ANCHOR: &str = r#"    if bytes_len(&nodes) < node_count * 16 || bytes_len(&values) < value_records * 8
        || bytes_len(&operators) < operator_records * 20
        || bytes_len(&blocks) < block_records * 12 {
        return 70;
    }
    if status == 0 && (bytes_len(&nodes) != node_count * 16
        || bytes_len(&values) != value_records * 8
        || bytes_len(&operators) != operator_records * 20
        || bytes_len(&blocks) != block_records * 12) {
        return 70;
    }"#;
const CALL_STORAGE_INVARIANT: &str = r#"    if bytes_len(&nodes) < node_count * 16 || bytes_len(&values) < value_records * 8
        || bytes_len(&operators) < operator_records * 20
        || bytes_len(&blocks) < block_records * 12
        || bytes_len(&calls) < call_records * 12 {
        return 70;
    }
    if status == 0 && (bytes_len(&nodes) != node_count * 16
        || bytes_len(&values) != value_records * 8
        || bytes_len(&operators) != operator_records * 20
        || bytes_len(&blocks) != block_records * 12
        || bytes_len(&calls) != call_records * 12) {
        return 70;
    }"#;
const CALL_NODE_KINDS_ANCHOR: &str = r#"        if validate_node_kind <= 0 || validate_node_kind > 19 {
            return 78;
        }"#;
const CALL_NODE_KINDS: &str = r#"        if validate_node_kind <= 0 || validate_node_kind > 23 {
            return 78;
        }"#;
const CALL_NODE_SHAPES_ANCHOR: &str = r#"        } else if validate_node_kind == 3 || validate_node_kind == 4
            || validate_node_kind == 18 {
            if validate_payload != 0 || validate_left <= 0
                || validate_left >= node_id || validate_right != 0 {
                return 78;
            }
        } else if validate_node_kind >= 5 && validate_node_kind <= 17 {
            if validate_payload != 0 || validate_left <= 0 || validate_right <= 0
                || validate_left >= node_id || validate_right >= node_id {
                return 78;
            }
        } else if validate_payload <= 0 || validate_payload > name_count"#;
const CALL_NODE_SHAPES: &str = r#"        } else if validate_node_kind == 3 || validate_node_kind == 4
            || validate_node_kind == 18 || validate_node_kind == 22
            || validate_node_kind == 23 {
            if validate_payload != 0 || validate_left <= 0
                || validate_left >= node_id || validate_right != 0 {
                return 78;
            }
        } else if validate_node_kind >= 5 && validate_node_kind <= 17 {
            if validate_payload != 0 || validate_left <= 0 || validate_right <= 0
                || validate_left >= node_id || validate_right >= node_id {
                return 78;
            }
        } else if validate_node_kind == 20 {
            if validate_payload <= 0 || validate_payload > name_count
                || validate_left < 0 || validate_left >= node_id
                || validate_right != 0 {
                return 78;
            }
        } else if validate_node_kind == 21 {
            if validate_payload != 0 || validate_left <= 0
                || validate_left >= node_id || validate_right < 0
                || validate_right >= node_id {
                return 78;
            }
        } else if validate_payload <= 0 || validate_payload > name_count"#;
const CALL_ORIGIN_BOUND_ANCHOR: &str = r#"                || origin_column <= 0 || origin_token_kind <= 0
                || origin_token_kind > 36 {"#;
const CALL_ORIGIN_BOUND: &str = r#"                || origin_column <= 0 || origin_token_kind <= 0
                || origin_token_kind > 37 {"#;
const CALL_ORIGIN_MAPPING_ANCHOR: &str = r#"                if origin_node_kind == 19 {
                    origin_expected_kind = 3;
                }"#;
const CALL_ORIGIN_MAPPING: &str = r#"                if origin_node_kind == 19 {
                    origin_expected_kind = 3;
                }
                if origin_node_kind == 20 {
                    origin_expected_kind = 10;
                }
                if origin_node_kind == 21 {
                    origin_expected_kind = 11;
                }
                if origin_node_kind == 22 {
                    origin_expected_kind = 37;
                }
                if origin_node_kind == 23 {
                    origin_expected_kind = 37;
                }"#;
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
    let derived = derived.replace(ARM_RETURN_ANCHOR, ARM_RETURN);

    // CAP-052 admits the statement grammar: the skeleton's fixed `return` step
    // is dissolved into a statement loop, `;` is demoted from a closing token to
    // the return statement's own terminator, and the closing sequence shrinks to
    // `}` then end-of-input with exactly one entry point. No statement produces
    // a syntax node, so the `1..=19` node-kind bound is untouched.
    assert_eq!(derived.matches(STATEMENT_REGISTERS_ANCHOR).count(), 1);
    let derived = derived.replace(STATEMENT_REGISTERS_ANCHOR, STATEMENT_REGISTERS);

    assert_eq!(derived.matches(SKELETON_RETURN_STEP_ANCHOR).count(), 1);
    let derived = derived.replace(SKELETON_RETURN_STEP_ANCHOR, SKELETON_RETURN_STEP);

    assert_eq!(derived.matches(STATEMENT_ENTRY_ANCHOR).count(), 1);
    let derived = derived.replace(STATEMENT_ENTRY_ANCHOR, STATEMENT_ENTRY);

    assert_eq!(derived.matches(STATEMENT_REQUESTS_ANCHOR).count(), 1);
    let derived = derived.replace(STATEMENT_REQUESTS_ANCHOR, STATEMENT_REQUESTS);

    assert_eq!(derived.matches(STATEMENT_STATES_ANCHOR).count(), 1);
    let derived = derived.replace(STATEMENT_STATES_ANCHOR, STATEMENT_STATES);

    assert_eq!(derived.matches(EXPRESSION_RETURN_ANCHOR).count(), 1);
    let derived = derived.replace(EXPRESSION_RETURN_ANCHOR, EXPRESSION_RETURN);

    assert_eq!(derived.matches(MATCH_CLOSE_ANCHOR).count(), 1);
    let derived = derived.replace(MATCH_CLOSE_ANCHOR, MATCH_CLOSE);

    assert_eq!(derived.matches(CLOSING_SUFFIX_ANCHOR).count(), 1);
    let derived = derived.replace(CLOSING_SUFFIX_ANCHOR, CLOSING_SUFFIX);

    assert_eq!(derived.matches(CLOSING_ADVANCE_ANCHOR).count(), 1);
    let derived = derived.replace(CLOSING_ADVANCE_ANCHOR, CLOSING_ADVANCE);

    assert_eq!(derived.matches(BODY_ROOT_ANCHOR).count(), 1);
    let derived = derived.replace(BODY_ROOT_ANCHOR, BODY_ROOT);

    // CAP-053 admits `if` / `else if` / `else` and `while` over the accepted
    // expression grammar: a block record store with its own bound and its own
    // exhaustion diagnostic, a statement dispatch that admits two more leading
    // tokens and rejects any statement after a completed `return`, a statement
    // terminator parameterized by what the expression was for, and the return
    // requirement moved from the block to the function. Neither form produces a
    // syntax node, so the `1..=19` node-kind bound is untouched.
    assert_eq!(derived.matches(BLOCK_OWNER_ANCHOR).count(), 1);
    let derived = derived.replace(BLOCK_OWNER_ANCHOR, BLOCK_OWNER);
    assert_eq!(derived.matches(BLOCK_REGISTERS_ANCHOR).count(), 1);
    let derived = derived.replace(BLOCK_REGISTERS_ANCHOR, BLOCK_REGISTERS);
    assert_eq!(derived.matches(ELSE_REQUEST_ANCHOR).count(), 1);
    let derived = derived.replace(ELSE_REQUEST_ANCHOR, ELSE_REQUEST);
    assert_eq!(derived.matches(CONTROL_FLOW_DISPATCH_ANCHOR).count(), 1);
    let derived = derived.replace(CONTROL_FLOW_DISPATCH_ANCHOR, CONTROL_FLOW_DISPATCH);
    assert_eq!(derived.matches(STATEMENT_TERMINATOR_ANCHOR).count(), 1);
    let derived = derived.replace(STATEMENT_TERMINATOR_ANCHOR, STATEMENT_TERMINATOR);
    assert_eq!(
        derived.matches(FUNCTION_RETURN_REQUIREMENT_ANCHOR).count(),
        1
    );
    let derived = derived.replace(
        FUNCTION_RETURN_REQUIREMENT_ANCHOR,
        FUNCTION_RETURN_REQUIREMENT,
    );
    assert_eq!(derived.matches(BLOCK_RECORD_READ_ANCHOR).count(), 1);
    let derived = derived.replace(BLOCK_RECORD_READ_ANCHOR, BLOCK_RECORD_READ);
    assert_eq!(derived.matches(BLOCK_RECORD_APPEND_ANCHOR).count(), 1);
    let derived = derived.replace(BLOCK_RECORD_APPEND_ANCHOR, BLOCK_RECORD_APPEND);
    assert_eq!(derived.matches(BLOCK_STORAGE_INVARIANT_ANCHOR).count(), 1);
    let derived = derived.replace(BLOCK_STORAGE_INVARIANT_ANCHOR, BLOCK_STORAGE_INVARIANT);

    assert_eq!(derived.matches(CALL_OWNER_ANCHOR).count(), 1);
    let derived = derived.replace(CALL_OWNER_ANCHOR, CALL_OWNER);
    assert_eq!(derived.matches(CALL_REGISTERS_ANCHOR).count(), 1);
    let derived = derived.replace(CALL_REGISTERS_ANCHOR, CALL_REGISTERS);
    assert_eq!(derived.matches(CALL_CLASSIFIER_ANCHOR).count(), 1);
    let derived = derived.replace(CALL_CLASSIFIER_ANCHOR, CALL_CLASSIFIER);
    assert_eq!(derived.matches(CALL_OPERATOR_POSITION_ANCHOR).count(), 1);
    let derived = derived.replace(CALL_OPERATOR_POSITION_ANCHOR, CALL_OPERATOR_POSITION);
    assert_eq!(derived.matches(CALL_REDUCE_WALK_ANCHOR).count(), 1);
    let derived = derived.replace(CALL_REDUCE_WALK_ANCHOR, CALL_REDUCE_WALK);
    assert_eq!(derived.matches(CALL_UNARY_REDUCE_ANCHOR).count(), 1);
    let derived = derived.replace(CALL_UNARY_REDUCE_ANCHOR, CALL_UNARY_REDUCE);
    assert_eq!(derived.matches(CALL_REDUCTION_ACTUAL_ANCHOR).count(), 2);
    let derived = derived.replace(CALL_REDUCTION_ACTUAL_ANCHOR, CALL_REDUCTION_ACTUAL);
    assert_eq!(derived.matches(CALL_CLOSE_DISPATCH_ANCHOR).count(), 1);
    let derived = derived.replace(CALL_CLOSE_DISPATCH_ANCHOR, CALL_CLOSE_DISPATCH);
    assert_eq!(derived.matches(CALL_STATES_ANCHOR).count(), 1);
    let derived = derived.replace(CALL_STATES_ANCHOR, CALL_STATES);
    assert_eq!(derived.matches(CALL_RECORD_READ_ANCHOR).count(), 1);
    let derived = derived.replace(CALL_RECORD_READ_ANCHOR, CALL_RECORD_READ);
    assert_eq!(derived.matches(CALL_RECORD_APPEND_ANCHOR).count(), 1);
    let derived = derived.replace(CALL_RECORD_APPEND_ANCHOR, CALL_RECORD_APPEND);
    assert_eq!(derived.matches(CALL_STORAGE_INVARIANT_ANCHOR).count(), 1);
    let derived = derived.replace(CALL_STORAGE_INVARIANT_ANCHOR, CALL_STORAGE_INVARIANT);
    assert_eq!(derived.matches(CALL_NODE_KINDS_ANCHOR).count(), 1);
    let derived = derived.replace(CALL_NODE_KINDS_ANCHOR, CALL_NODE_KINDS);
    assert_eq!(derived.matches(CALL_NODE_SHAPES_ANCHOR).count(), 1);
    let derived = derived.replace(CALL_NODE_SHAPES_ANCHOR, CALL_NODE_SHAPES);
    assert_eq!(derived.matches(CALL_ORIGIN_BOUND_ANCHOR).count(), 1);
    let derived = derived.replace(CALL_ORIGIN_BOUND_ANCHOR, CALL_ORIGIN_BOUND);
    assert_eq!(derived.matches(CALL_ORIGIN_MAPPING_ANCHOR).count(), 1);
    let derived = derived.replace(CALL_ORIGIN_MAPPING_ANCHOR, CALL_ORIGIN_MAPPING);

    // CAP-055 / H1B-6. The uniform parse-group capacity raise, applied last and
    // as one step rather than as thirty-two anchored fragments.
    //
    // It is expressed this way because that is what it is: one policy ceiling
    // shared by five stores, nine of whose sites the accepted B1C product
    // already carries verbatim and seven of which CAP-050 through CAP-054
    // added. Anchoring each site would make the anchors and their replacements
    // differ only in a number, and would silently accept a *missed* site as
    // "no difference" instead of failing. The counted transform cannot: it
    // asserts exactly how many sites it rewrote.
    raise_parse_record_bound(&derived)
}

/// Rewrite every parse-group record ceiling and every exhaustion
/// `diagnostic_code` from the accepted 512 to [`PARSE_RECORD_BOUND`].
///
/// The verifier's own `512` at `compiler.aero:5557` is untouched, because the
/// transform matches only lines that compare one of the five parse-group stores
/// and lines that are exactly an exhaustion `diagnostic_code` assignment.
/// `verified_function_node > 512` is neither.
fn raise_parse_record_bound(source: &str) -> String {
    let stores = [
        "block_records",
        "operator_records",
        "call_records",
        "node_count",
        "value_records",
    ];
    let bound = PARSE_RECORD_BOUND;
    let mut comparisons = 0usize;
    let mut compared = 0usize;
    let mut codes = 0usize;
    let mut raised: Vec<String> = Vec::new();
    for line in source.split('\n') {
        let hits = stores
            .iter()
            .filter(|store| line.contains(&format!("{store} >= 512")))
            .count();
        if hits > 0 {
            let mut rewritten = line.to_string();
            for store in stores {
                rewritten =
                    rewritten.replace(&format!("{store} >= 512"), &format!("{store} >= {bound}"));
            }
            comparisons += 1;
            compared += hits;
            raised.push(rewritten);
        } else if line.trim() == "diagnostic_code = 512;" {
            codes += 1;
            raised.push(line.replace("512", &bound.to_string()));
        } else {
            raised.push(line.to_string());
        }
    }
    // Sixteen conditions guarding seventeen compared stores, because
    // `compiler.aero:2191` guards the call store and the operator store in one
    // condition, and sixteen `diagnostic_code` assignments beside them.
    assert_eq!(comparisons, 16, "parse-group comparison conditions");
    assert_eq!(compared, 17, "parse-group stores compared");
    assert_eq!(codes, 16, "exhaustion diagnostic codes");
    raised.join("\n")
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

/// CAP-052 / H1B-3 focused statement probes.
///
/// This checkpoint cannot move the canonical self-ingestion stop: CAP-051
/// already parses function 1 completely, and admitting a second `fn` item is
/// excluded from every parser checkpoint. So these probes are the checkpoint's
/// entire forward evidence, and the canonical target below is a regression
/// guard rather than a progress measure.
///
/// The admitted statement forms are `let IDENT : int = EXPR ;`,
/// `let mut IDENT : int = EXPR ;`, `IDENT = EXPR ;`, and `return EXPR ;`, in a
/// body that is `{` followed by one or more statements followed by `}`. Every
/// expectation is stated here as an independent hand derivation from the frozen
/// contract and is separately derived by the oracle, so the two must agree.
const STATEMENT_PROBES: &[(&str, &[u8], i32, i32, i32, &str, usize, usize)] = &[
    // label, source, status, code, actual, token text, parameters, nodes
    // The CAP-051 shape, which the demoted `;` must not regress.
    (
        "stmt-return-only",
        b"fn f() -> int { return 1; } x",
        10,
        0,
        1,
        "x",
        0,
        1,
    ),
    (
        "stmt-let-before-return",
        b"fn f() -> int { let a: int = 1; return a; } x",
        10,
        0,
        1,
        "x",
        0,
        2,
    ),
    (
        "stmt-let-mut-before-return",
        b"fn f() -> int { let mut a: int = 1; return a; } x",
        10,
        0,
        1,
        "x",
        0,
        2,
    ),
    (
        "stmt-assignment-between",
        b"fn f() -> int { let mut a: int = 1; a = 2; return a; } x",
        10,
        0,
        1,
        "x",
        0,
        3,
    ),
    // The binding's initializer is the whole accepted expression grammar: the
    // three literals, then the product, then the sum, then `a`.
    (
        "stmt-let-expression",
        b"fn f() -> int { let a: int = 1+2*3; return a; } x",
        10,
        0,
        1,
        "x",
        0,
        6,
    ),
    // The two grammars compose: a binding, then the CAP-051 match construct as
    // the return statement's expression.
    (
        "stmt-match-return-composes",
        b"fn f(a: Result<int,int>) -> int { let b: int = 1; return match a { Ok(v) => v, Err(c) => c, }; } x",
        10,
        0,
        1,
        "x",
        1,
        3,
    ),
    // `match` is admitted in the return statement's expression only, so in a
    // binding's initializer it is an ordinary identifier operand and the
    // scrutinee that follows it ends the expression.
    (
        "stmt-match-not-in-initializer",
        b"fn f(a: Result<int,int>) -> int { let b: int = match a; return b; } x",
        10,
        18,
        1,
        "a",
        1,
        1,
    ),
    (
        "stmt-missing-type-annotation",
        b"fn f() -> int { let a = 1; return a; } x",
        10,
        17,
        25,
        "=",
        0,
        0,
    ),
    (
        "stmt-missing-initializer",
        b"fn f() -> int { let a: int; return a; } x",
        10,
        25,
        18,
        ";",
        0,
        0,
    ),
    (
        "stmt-bytebuffer-binding",
        b"fn f() -> int { let a: ByteBuffer = b; return a; } x",
        12,
        102,
        1,
        "ByteBuffer",
        0,
        0,
    ),
    (
        "stmt-result-binding",
        b"fn f() -> int { let a: Result<int,int> = b; return a; } x",
        12,
        102,
        1,
        "Result",
        0,
        0,
    ),
    (
        "stmt-mut-without-name",
        b"fn f() -> int { let mut: int = 1; return a; } x",
        10,
        1,
        17,
        ":",
        0,
        0,
    ),
    (
        "stmt-non-identifier-target",
        b"fn f() -> int { a[0] = 1; return a; } x",
        10,
        25,
        14,
        "[",
        0,
        0,
    ),
    (
        "stmt-missing-semicolon",
        b"fn f() -> int { return 1 } x",
        10,
        18,
        13,
        "}",
        0,
        1,
    ),
    (
        "stmt-empty-body",
        b"fn f() -> int { } x",
        10,
        6,
        13,
        "}",
        0,
        0,
    ),
    (
        "stmt-mut-without-let",
        b"fn f() -> int { mut a: int = 1; return a; } x",
        10,
        6,
        5,
        "mut",
        0,
        0,
    ),
    // A body that closes without a completed return statement has no root for
    // the function node, so its `}` is rejected with the statement expectation.
    (
        "stmt-body-without-return",
        b"fn f() -> int { let a: int = 1; } x",
        10,
        6,
        13,
        "}",
        0,
        1,
    ),
    (
        "stmt-non-statement-start",
        b"fn f() -> int { return 1; 2; } x",
        10,
        13,
        2,
        "2",
        0,
        1,
    ),
];

fn statement_probe_targets() -> Vec<oracle::Ingestion> {
    let mut targets = Vec::new();
    for (label, source, status, code, actual, text, parameters, nodes) in STATEMENT_PROBES {
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

        let target = oracle::statement_parser_stop(&ingested, source);
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

/// Every expectation in [`STATEMENT_PROBES`] is a hand derivation from the
/// frozen contract. This test touches no product: it only requires the oracle
/// to agree with all of them.
#[test]
fn every_statement_probe_expectation_is_derived_twice() {
    assert_eq!(statement_probe_targets().len(), STATEMENT_PROBES.len());
}

#[test]
fn focused_statement_probes_exercise_every_rule_of_the_admitted_grammar() {
    for ((label, source, _, _, _, _, _, _), target) in
        STATEMENT_PROBES.iter().zip(statement_probe_targets())
    {
        assert_eq!(
            run_expectation("statement-probe", compiled_h1a(), source, &target, "-O0"),
            91,
            "probe `{label}` diverged from the derived CAP-052 target"
        );
    }
}

/// CAP-052 / H1B-3 leaves the canonical self-ingestion stop exactly where
/// CAP-051 put it, and this is a regression guard rather than evidence of
/// forward progress.
///
/// Function 1 is already parsed completely, and admitting a second `fn` item is
/// excluded from every parser checkpoint, so no construct this checkpoint
/// admits is reachable in the canonical source at all. The statement model and
/// the CAP-051 model must therefore agree on it exactly, including the four
/// orphan arm-body nodes, whose count is asserted so that index-walking
/// chaining could not sweep them into a statement sequence unnoticed.
#[test]
fn the_statement_block_checkpoint_leaves_the_canonical_stop_unmoved() {
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

    let frozen = oracle::SignatureStop {
        status: 10,
        error_offset: 146,
        error_line: 8,
        error_column: 1,
        diagnostic_code: 0,
        diagnostic_actual: 3,
        node_count: 4,
        parameters: 1,
    };
    assert_eq!(
        oracle::match_grammar_stop(&ingested, &source),
        frozen,
        "CAP-051's frozen target moved"
    );
    assert_eq!(
        oracle::statement_grammar_stop(&ingested, &source),
        frozen,
        "CAP-052 must not move the canonical stop in either direction"
    );
    assert_eq!(&source[146..148], b"fn");

    // The whole node arena at the stop is still the four CAP-051 orphans.
    let stopped = oracle::statement_parser_stop(&ingested, &source);
    assert_eq!(stopped.nodes.len(), 4);
    assert_eq!(stopped.origins.len(), 4);

    for optimization in ["-O0", "-O2"] {
        assert_eq!(
            run_expectation(
                "statement-self-ingestion",
                compiled_h1a(),
                &source,
                &stopped,
                optimization
            ),
            91,
            "CAP-052 self-ingestion diverged from the independent oracle at {optimization}"
        );
    }
}

/// CAP-053 / H1B-4 focused control-flow probes.
///
/// This checkpoint cannot move the canonical self-ingestion stop either:
/// CAP-051 already parses function 1 completely, and a second `fn` item is
/// excluded from every parser checkpoint. So these probes are the checkpoint's
/// entire forward evidence, and the canonical target below stays a regression
/// guard.
///
/// The admitted forms are `if EXPR BLOCK`, with any number of `else if EXPR
/// BLOCK` arms and an optional final `else BLOCK`, and `while EXPR BLOCK`.
/// `EXPR` is the already-accepted expression grammar with no `match`, and
/// `BLOCK` is CAP-052's statement sequence with two differences: it closes on
/// `}` and nothing more, and it carries no return requirement, which has moved
/// from the block to the function. Within any block a `return` is the last
/// statement and the only one.
///
/// Every expectation is stated here as an independent hand derivation from the
/// frozen contract and is separately derived by the oracle, so the two must
/// agree.
const CONTROL_FLOW_PROBES: &[(&str, &[u8], i32, i32, i32, &str, usize, usize)] = &[
    // label, source, status, code, actual, token text, parameters, nodes
    // --- the two admitted forms, and the shapes they compose into ---
    (
        "cf-if-alone",
        b"fn f() -> int { if a { b = 1; } return 2; } x",
        10,
        0,
        1,
        "x",
        0,
        3,
    ),
    (
        "cf-if-else",
        b"fn f() -> int { if a { b = 1; } else { b = 2; } return 3; } x",
        10,
        0,
        1,
        "x",
        0,
        4,
    ),
    (
        "cf-if-else-if-else",
        b"fn f() -> int { if a { b = 1; } else if c { b = 2; } else { b = 3; } return 4; } x",
        10,
        0,
        1,
        "x",
        0,
        6,
    ),
    (
        "cf-while",
        b"fn f() -> int { while a { b = 1; } return 2; } x",
        10,
        0,
        1,
        "x",
        0,
        3,
    ),
    // Canonical function 2's shape: a `return` inside an `if` body, and a
    // further `return` after the `if`. The nested one leaves an orphan and the
    // last write to `body_root` wins.
    (
        "cf-return-in-if-then-return",
        b"fn f() -> int { if a { return 1; } return 2; } x",
        10,
        0,
        1,
        "x",
        0,
        3,
    ),
    (
        "cf-nesting-three-deep",
        b"fn f() -> int { if a { if b { if c { d = 1; } } } return 2; } x",
        10,
        0,
        1,
        "x",
        0,
        5,
    ),
    (
        "cf-binding-and-assignment-in-block",
        b"fn f() -> int { if a { let b: int = 1; b = 2; } return 3; } x",
        10,
        0,
        1,
        "x",
        0,
        4,
    ),
    (
        "cf-nested-if-in-else",
        b"fn f() -> int { if a { b = 1; } else { if c { b = 2; } } return 3; } x",
        10,
        0,
        1,
        "x",
        0,
        5,
    ),
    // A condition is the whole accepted expression grammar: the three literals,
    // then the product, then the sum.
    (
        "cf-condition-expression",
        b"fn f() -> int { if 1+2*3 { a = 4; } return 5; } x",
        10,
        0,
        1,
        "x",
        0,
        7,
    ),
    // The CAP-051 construct still composes: a `match` return inside an `if`
    // body, whose two arm bodies are orphaned exactly as before.
    (
        "cf-match-return-in-if",
        b"fn f(a: Result<int,int>) -> int { if b { return match a { Ok(v) => v, Err(c) => c, }; } return 1; } x",
        10,
        0,
        1,
        "x",
        1,
        4,
    ),
    // Canonical function 2, `is_identifier_start`, lifted verbatim. This is the
    // first real canonical function to become parseable since CAP-051. It does
    // not parse in situ, because the canonical run stops at the second `fn`
    // item first. Its condition is 10 operand leaves and 9 reductions, and each
    // of its two `return` statements adds one literal leaf.
    (
        "cf-canonical-function-2",
        b"fn is_identifier_start(value: int) -> int {\n    if value == 95 || (value >= 65 && value <= 90) || (value >= 97 && value <= 122) {\n        return 1;\n    }\n    return 0;\n} x",
        10,
        0,
        1,
        "x",
        1,
        21,
    ),
    // --- `else` must follow an `if` body's `}` and nothing else ---
    (
        "cf-else-without-if",
        b"fn f() -> int { else { a = 1; } return 2; } x",
        10,
        6,
        8,
        "else",
        0,
        0,
    ),
    (
        "cf-else-after-while",
        b"fn f() -> int { while a { b = 1; } else { b = 2; } return 3; } x",
        10,
        6,
        8,
        "else",
        0,
        2,
    ),
    (
        "cf-else-after-else",
        b"fn f() -> int { if a { b = 1; } else { b = 2; } else { b = 3; } return 4; } x",
        10,
        6,
        8,
        "else",
        0,
        3,
    ),
    (
        "cf-else-without-block",
        b"fn f() -> int { if a { b = 1; } else c { } return 2; } x",
        10,
        12,
        1,
        "c",
        0,
        2,
    ),
    // --- a block is never empty, and never closes with a statement open ---
    (
        "cf-empty-if-body",
        b"fn f() -> int { if a { } return 2; } x",
        10,
        6,
        13,
        "}",
        0,
        1,
    ),
    (
        "cf-empty-while-body",
        b"fn f() -> int { while a { } return 2; } x",
        10,
        6,
        13,
        "}",
        0,
        1,
    ),
    (
        "cf-non-statement-in-block",
        b"fn f() -> int { if a { b = 1; 2; } return 3; } x",
        10,
        13,
        2,
        "2",
        0,
        2,
    ),
    (
        "cf-block-without-close",
        b"fn f() -> int { if a { return 1;",
        10,
        13,
        0,
        "",
        0,
        2,
    ),
    // --- the condition ---
    (
        "cf-condition-opens-with-brace",
        b"fn f() -> int { if { a = 1; } return 2; } x",
        11,
        100,
        12,
        "{",
        0,
        0,
    ),
    // `match` is admitted in a return expression only, so in a condition it is
    // an ordinary identifier operand and the scrutinee that follows it ends the
    // expression where the block's `{` was required.
    (
        "cf-match-in-condition",
        b"fn f(a: Result<int,int>) -> int { if match a { b = 1; } return 2; } x",
        10,
        12,
        1,
        "a",
        1,
        1,
    ),
    (
        "cf-condition-without-block",
        b"fn f() -> int { if a return 1; } x",
        10,
        12,
        6,
        "return",
        0,
        1,
    ),
    // --- the per-block return rule CAP-052 froze and did not implement ---
    (
        "cf-statement-after-return",
        b"fn f() -> int { return 1; let a: int = 2; } x",
        10,
        13,
        4,
        "let",
        0,
        1,
    ),
    (
        "cf-second-return-in-block",
        b"fn f() -> int { return 1; return 2; } x",
        10,
        13,
        6,
        "return",
        0,
        1,
    ),
    // --- the return requirement is the function's, not any block's ---
    (
        "cf-function-without-return",
        b"fn f() -> int { if a { return 1; } } x",
        10,
        6,
        13,
        "}",
        0,
        2,
    ),
];

/// The byte span of canonical function 2 in `compiler.aero`, which the
/// `cf-canonical-function-2` probe must reproduce exactly. 146 is the frozen
/// canonical stop, and 315 is the byte after the function's closing `}` - the
/// newline that separates it from function 3 is not part of the function.
const CANONICAL_FUNCTION_2: (usize, usize) = (146, 315);

fn control_flow_probe_targets() -> Vec<oracle::Ingestion> {
    let mut targets = Vec::new();
    for (label, source, status, code, actual, text, parameters, nodes) in CONTROL_FLOW_PROBES {
        assert!(
            source.len() < 200,
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

        let target = oracle::control_flow_parser_stop(&ingested, source);
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

/// Every expectation in [`CONTROL_FLOW_PROBES`] is a hand derivation from the
/// frozen contract. This test touches no product: it only requires the oracle
/// to agree with all of them.
#[test]
fn every_control_flow_probe_expectation_is_derived_twice() {
    assert_eq!(
        control_flow_probe_targets().len(),
        CONTROL_FLOW_PROBES.len()
    );
}

/// The `cf-canonical-function-2` probe is a verbatim lift, not a paraphrase.
#[test]
fn the_canonical_function_2_probe_is_the_canonical_bytes() {
    let source = fs::read(repository_path(H1A_PRODUCT)).expect("read CAP-049 canonical source");
    let (from, to) = CANONICAL_FUNCTION_2;
    let probe = CONTROL_FLOW_PROBES
        .iter()
        .find(|(label, ..)| *label == "cf-canonical-function-2")
        .expect("the canonical function 2 probe is required coverage")
        .1;
    assert_eq!(&source[from..from + 2], b"fn");
    assert_eq!(
        &probe[..probe.len() - 2],
        &source[from..to],
        "the canonical function 2 probe must be the canonical bytes plus a trailing stop token"
    );
    assert_eq!(&probe[probe.len() - 2..], b" x");
}

#[test]
fn focused_control_flow_probes_exercise_every_rule_of_the_admitted_grammar() {
    for ((label, source, _, _, _, _, _, _), target) in
        CONTROL_FLOW_PROBES.iter().zip(control_flow_probe_targets())
    {
        assert_eq!(
            run_expectation("control-flow-probe", compiled_h1a(), source, &target, "-O0"),
            91,
            "probe `{label}` diverged from the derived CAP-053 target"
        );
    }
}

/// CAP-053 / H1B-4 leaves the canonical self-ingestion stop exactly where
/// CAP-051 put it, and this is a regression guard rather than evidence of
/// forward progress.
///
/// The canonical source's first function contains no control-flow token, so
/// the block stack is never pushed on the canonical run and the three earlier
/// models must agree with the control-flow model exactly, including the four
/// orphan arm-body nodes.
#[test]
fn the_control_flow_checkpoint_leaves_the_canonical_stop_unmoved() {
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

    let frozen = oracle::SignatureStop {
        status: 10,
        error_offset: 146,
        error_line: 8,
        error_column: 1,
        diagnostic_code: 0,
        diagnostic_actual: 3,
        node_count: 4,
        parameters: 1,
    };
    assert_eq!(
        oracle::statement_grammar_stop(&ingested, &source),
        frozen,
        "CAP-052's frozen target moved"
    );
    assert_eq!(
        oracle::control_flow_grammar_stop(&ingested, &source),
        frozen,
        "CAP-053 must not move the canonical stop in either direction"
    );
    assert_eq!(&source[146..148], b"fn");

    let stopped = oracle::control_flow_parser_stop(&ingested, &source);
    assert_eq!(stopped.nodes.len(), 4);
    assert_eq!(stopped.origins.len(), 4);

    for optimization in ["-O0", "-O2"] {
        assert_eq!(
            run_expectation(
                "control-flow-self-ingestion",
                compiled_h1a(),
                &source,
                &stopped,
                optimization
            ),
            91,
            "CAP-053 self-ingestion diverged from the independent oracle at {optimization}"
        );
    }
}

// ---------------------------------------------------------------------------
// CAP-054 / H1B-5: calls and references
// ---------------------------------------------------------------------------

/// Every expectation below is a hand derivation from the frozen CAP-054
/// contract, written before the oracle was consulted and before
/// `compiler.aero` was touched.
///
/// A probe's trailing `x` forces a stop at a predicted token, so the node count
/// is the arena at that stop: the kind-18 return node and the kind-19 function
/// node are appended only after end of input is accepted, which no probe
/// reaches.
///
/// Node accounting for a call: the callee produces **no** node, each argument
/// produces its own expression nodes, then one kind-21 cell per argument, then
/// one kind-20 call node. A reference produces one kind-22 or kind-23 node over
/// its operand.
const CALL_PROBES: &[(&str, &[u8], i32, i32, i32, &str, usize, usize)] = &[
    // label, source, status, code, actual, token text, parameters, nodes
    // --- the admitted call shapes ---
    (
        "call-zero-arguments",
        b"fn f() -> int { return g(); } x",
        10,
        0,
        1,
        "x",
        0,
        1,
    ),
    (
        "call-one-identifier-argument",
        b"fn f() -> int { return g(a); } x",
        10,
        0,
        1,
        "x",
        0,
        3,
    ),
    (
        "call-one-integer-argument",
        b"fn f() -> int { return g(1); } x",
        10,
        0,
        1,
        "x",
        0,
        3,
    ),
    (
        "call-two-arguments",
        b"fn f() -> int { return g(a, b); } x",
        10,
        0,
        1,
        "x",
        0,
        5,
    ),
    (
        "call-three-arguments",
        b"fn f() -> int { return g(a, b, c); } x",
        10,
        0,
        1,
        "x",
        0,
        7,
    ),
    (
        "call-nested",
        b"fn f() -> int { return g(h(a)); } x",
        10,
        0,
        1,
        "x",
        0,
        5,
    ),
    (
        "call-expression-argument",
        b"fn f() -> int { return g(a + 1); } x",
        10,
        0,
        1,
        "x",
        0,
        5,
    ),
    (
        "call-grouped-argument",
        b"fn f() -> int { return g((a)); } x",
        10,
        0,
        1,
        "x",
        0,
        3,
    ),
    (
        "call-prefix-argument",
        b"fn f() -> int { return g(-a); } x",
        10,
        0,
        1,
        "x",
        0,
        4,
    ),
    (
        "call-in-argument-with-operator",
        b"fn f() -> int { return g(h(a) + 1); } x",
        10,
        0,
        1,
        "x",
        0,
        7,
    ),
    (
        "call-two-calls-as-arguments",
        b"fn f() -> int { return g(h(a), k(b)); } x",
        10,
        0,
        1,
        "x",
        0,
        9,
    ),
    // --- a call in every position the source puts one ---
    (
        "call-in-if-condition",
        b"fn f() -> int { if g(a) { b = 1; } return 2; } x",
        10,
        0,
        1,
        "x",
        0,
        5,
    ),
    (
        "call-in-while-condition",
        b"fn f() -> int { while g(a) { b = 1; } return 2; } x",
        10,
        0,
        1,
        "x",
        0,
        5,
    ),
    (
        "call-in-binding",
        b"fn f() -> int { let a: int = g(b); return a; } x",
        10,
        0,
        1,
        "x",
        0,
        4,
    ),
    (
        "call-in-assignment",
        b"fn f() -> int { a = g(b); return c; } x",
        10,
        0,
        1,
        "x",
        0,
        4,
    ),
    (
        "call-as-binary-operand",
        b"fn f() -> int { return g(a) + 1; } x",
        10,
        0,
        1,
        "x",
        0,
        5,
    ),
    (
        "call-in-match-arm",
        b"fn f(r: Result<int,int>) -> int { return match r { Ok(v) => g(v), Err(c) => c, }; } x",
        10,
        0,
        1,
        "x",
        1,
        4,
    ),
    // --- the two reference forms ---
    (
        "call-reference-argument",
        b"fn f() -> int { return g(&a); } x",
        10,
        0,
        1,
        "x",
        0,
        4,
    ),
    (
        "call-mutable-reference-argument",
        b"fn f() -> int { return g(&mut a); } x",
        10,
        0,
        1,
        "x",
        0,
        4,
    ),
    (
        "call-reference-after-comma",
        b"fn f() -> int { return g(a, &b); } x",
        10,
        0,
        1,
        "x",
        0,
        6,
    ),
    (
        "call-mixed-reference-arguments",
        b"fn f() -> int { return g(&a, &mut b, c); } x",
        10,
        0,
        1,
        "x",
        0,
        9,
    ),
    // --- rejections: the argument list ---
    (
        "call-trailing-comma",
        b"fn f() -> int { return g(a,); } x",
        11,
        100,
        11,
        ")",
        0,
        1,
    ),
    (
        "call-leading-comma",
        b"fn f() -> int { return g(,a); } x",
        11,
        100,
        16,
        ",",
        0,
        0,
    ),
    (
        "call-empty-argument",
        b"fn f() -> int { return g(a,,b); } x",
        11,
        100,
        16,
        ",",
        0,
        1,
    ),
    (
        "call-missing-comma",
        b"fn f() -> int { return g(a b); } x",
        10,
        11,
        1,
        "b",
        0,
        1,
    ),
    (
        "call-unclosed",
        b"fn f() -> int { return g(a; } x",
        10,
        11,
        18,
        ";",
        0,
        1,
    ),
    (
        "call-extra-close-paren",
        b"fn f() -> int { return g(a)); } x",
        10,
        18,
        11,
        ")",
        0,
        3,
    ),
    (
        "call-close-paren-with-no-call",
        b"fn f() -> int { return ); } x",
        11,
        100,
        11,
        ")",
        0,
        0,
    ),
    (
        "call-comma-inside-grouping",
        b"fn f() -> int { return (a, b); } x",
        10,
        11,
        16,
        ",",
        0,
        1,
    ),
    // --- rejections: the callee must be a bare identifier ---
    (
        "call-on-grouped-callee",
        b"fn f() -> int { return (a)(b); } x",
        10,
        18,
        10,
        "(",
        0,
        1,
    ),
    (
        "call-on-integer-callee",
        b"fn f() -> int { return 1(a); } x",
        10,
        18,
        10,
        "(",
        0,
        1,
    ),
    (
        "call-on-call-result",
        b"fn f() -> int { return g(a)(b); } x",
        10,
        18,
        10,
        "(",
        0,
        3,
    ),
    (
        "call-match-in-argument",
        b"fn f() -> int { return g(match r); } x",
        10,
        11,
        1,
        "r",
        0,
        1,
    ),
    // --- rejections: a reference is a whole call argument and nothing else ---
    (
        "reference-in-return",
        b"fn f() -> int { return &a; } x",
        11,
        100,
        37,
        "&",
        0,
        0,
    ),
    (
        "reference-in-binding",
        b"fn f() -> int { let a: int = &b; return a; } x",
        11,
        100,
        37,
        "&",
        0,
        0,
    ),
    (
        "reference-in-condition",
        b"fn f() -> int { if &a { b = 1; } return 2; } x",
        11,
        100,
        37,
        "&",
        0,
        0,
    ),
    (
        "reference-after-binary-operator",
        b"fn f() -> int { return g(a + &b); } x",
        11,
        100,
        37,
        "&",
        0,
        1,
    ),
    (
        "reference-inside-grouping",
        b"fn f() -> int { return g((&a)); } x",
        11,
        100,
        37,
        "&",
        0,
        0,
    ),
    (
        "reference-not-first-in-argument",
        b"fn f() -> int { return g(&a, b + &c); } x",
        11,
        100,
        37,
        "&",
        0,
        3,
    ),
    (
        "reference-without-operand",
        b"fn f() -> int { return g(&); } x",
        11,
        100,
        11,
        ")",
        0,
        0,
    ),
    (
        "mutable-reference-without-operand",
        b"fn f() -> int { return g(& mut); } x",
        11,
        100,
        11,
        ")",
        0,
        0,
    ),
    // --- three canonical functions, lifted verbatim ---
    // `word_byte_1` is a nested call two deep, and its whole seven-node arena
    // is reachable from its root once the return and function nodes exist.
    (
        "call-canonical-word-byte-1",
        b"fn word_byte_1(value: int) -> int {\n    return word_byte_0(quotient_256(value));\n} x",
        10,
        0,
        1,
        "x",
        1,
        5,
    ),
    // `is_identifier_continue` puts a call in a condition.
    (
        "call-canonical-is-identifier-continue",
        b"fn is_identifier_continue(value: int) -> int {\n    if is_identifier_start(value) == 1 || (value >= 48 && value <= 57) {\n        return 1;\n    }\n    return 0;\n} x",
        10,
        0,
        1,
        "x",
        1,
        15,
    ),
];

/// The byte spans of the three canonical functions the CAP-054 probes lift
/// verbatim. Each range starts at its `fn` and ends at the byte after its
/// closing `}`; the newline that separates it from the next item is not part
/// of the function.
const CANONICAL_WORD_BYTE_1: (usize, usize) = (4539, 4621);
const CANONICAL_IS_IDENTIFIER_CONTINUE: (usize, usize) = (317, 476);

/// `main` is the last item in the canonical source, so its offset moves
/// whenever the parser above it changes. It is located by its own unique
/// opening line rather than frozen as a byte offset; the lift is still
/// verbatim, because the probe is asserted equal to the bytes found here.
fn canonical_main_span(source: &[u8]) -> (usize, usize) {
    let marker = b"fn main() -> int {";
    let occurrences = source
        .windows(marker.len())
        .filter(|window| *window == marker)
        .count();
    assert_eq!(occurrences, 1, "the canonical source declares one `main`");
    let from = source
        .windows(marker.len())
        .position(|window| window == marker)
        .expect("the canonical source declares `main`");
    assert_eq!(
        &source[source.len() - 2..],
        b"}
"
    );
    (from, source.len() - 1)
}

fn call_probe_targets() -> Vec<oracle::Ingestion> {
    let mut targets = Vec::new();
    for (label, source, status, code, actual, text, parameters, nodes) in CALL_PROBES {
        assert!(
            source.len() < 200,
            "probe `{label}` must stay a small complete program"
        );
        targets.push(graded_call_probe(
            label,
            source,
            *status,
            *code,
            *actual,
            text,
            *parameters,
            *nodes,
        ));
    }
    targets
}

#[allow(clippy::too_many_arguments)]
fn graded_call_probe(
    label: &str,
    source: &[u8],
    status: i32,
    code: i32,
    actual: i32,
    text: &str,
    parameters: usize,
    nodes: usize,
) -> oracle::Ingestion {
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

    let target = oracle::call_parser_stop(&ingested, source);
    let from = usize::try_from(target.error_offset).expect("bounded offset");
    assert_eq!(target.status, status, "probe `{label}` target status");
    assert_eq!(target.diagnostic_code, code, "probe `{label}` target code");
    assert_eq!(
        target.diagnostic_actual, actual,
        "probe `{label}` target actual"
    );
    assert_eq!(
        &source[from..from + text.len()],
        text.as_bytes(),
        "probe `{label}` target token"
    );
    assert_eq!(
        target.parameters.len(),
        parameters,
        "probe `{label}` target parameter count"
    );
    assert_eq!(
        target.nodes.len(),
        nodes,
        "probe `{label}` target node count"
    );
    assert_eq!(
        target.origins.len(),
        target.nodes.len(),
        "probe `{label}` must mirror every node with one origin"
    );
    target
}

/// `main` is the source's widest argument list at 68 arguments, and the only
/// canonical function whose complete arena becomes reachable from its root. It
/// is too large for the small-program assertion the other probes carry, so it
/// is graded on its own.
fn canonical_main_probe() -> (Vec<u8>, oracle::Ingestion) {
    let source = fs::read(repository_path(H1A_PRODUCT)).expect("read CAP-049 canonical source");
    let (from, to) = canonical_main_span(&source);
    let mut probe = source[from..to].to_vec();
    probe.extend_from_slice(b" x");
    let target = graded_call_probe(
        "call-canonical-main",
        &probe,
        10,
        0,
        1,
        "x",
        0,
        // 68 integer leaves, 7 prefix `-` nodes over 7 of them, 68 argument
        // cells and one call node.
        144,
    );
    (probe, target)
}

/// Every expectation in [`CALL_PROBES`] is a hand derivation from the frozen
/// CAP-054 contract. This test touches no product: it only requires the oracle
/// to agree with all of them.
#[test]
fn every_call_probe_expectation_is_derived_twice() {
    assert_eq!(call_probe_targets().len(), CALL_PROBES.len());
    canonical_main_probe();
}

/// The three canonical probes are verbatim lifts, not paraphrases.
#[test]
fn the_canonical_call_probes_are_the_canonical_bytes() {
    let source = fs::read(repository_path(H1A_PRODUCT)).expect("read CAP-049 canonical source");
    for (label, span) in [
        ("call-canonical-word-byte-1", CANONICAL_WORD_BYTE_1),
        (
            "call-canonical-is-identifier-continue",
            CANONICAL_IS_IDENTIFIER_CONTINUE,
        ),
    ] {
        let (from, to) = span;
        let probe = CALL_PROBES
            .iter()
            .find(|(name, ..)| *name == label)
            .unwrap_or_else(|| panic!("probe `{label}` is required coverage"))
            .1;
        assert_eq!(&source[from..from + 2], b"fn");
        assert_eq!(
            &probe[..probe.len() - 2],
            &source[from..to],
            "probe `{label}` must be the canonical bytes plus a trailing stop token"
        );
        assert_eq!(&probe[probe.len() - 2..], b" x");
    }
    let (probe, _) = canonical_main_probe();
    let (from, to) = canonical_main_span(&source);
    assert_eq!(&source[from..from + 2], b"fn");
    assert_eq!(&probe[..probe.len() - 2], &source[from..to]);
    assert_eq!(
        to,
        source.len() - 1,
        "`main` is the last item in the source"
    );
}

#[test]
fn focused_call_probes_exercise_every_rule_of_the_admitted_grammar() {
    for ((label, source, _, _, _, _, _, _), target) in CALL_PROBES.iter().zip(call_probe_targets())
    {
        assert_eq!(
            run_expectation("call-probe", compiled_h1a(), source, &target, "-O0"),
            91,
            "probe `{label}` diverged from the derived CAP-054 target"
        );
    }
    let (probe, target) = canonical_main_probe();
    assert_eq!(
        run_expectation("call-probe", compiled_h1a(), &probe, &target, "-O0"),
        91,
        "probe `call-canonical-main` diverged from the derived CAP-054 target"
    );
}

/// Shapes that **no** CAP-050, CAP-051, CAP-052 or CAP-053 probe covers, each
/// hand-derived under both the CAP-053 model and the CAP-054 model.
///
/// This table exists because of what CAP-053 found the hard way: its oracle
/// refactor silently changed the CAP-052 model for a shape no CAP-052 probe
/// covered, and all 41 inherited probes stayed green and hid it. A probe suite
/// passing is evidence about the probe suite, not about the extraction. So the
/// lock grades deliberately out-of-table shapes against the *previous*
/// checkpoint's model, and two of them - `a(b)` in a return expression and in a
/// condition - are shapes on which the two models must **disagree**, so that
/// the lock proves the CAP-053 model is still the CAP-053 model rather than
/// proving the two models have become the same.
const MODEL_LOCK_SHAPES: &[(
    &str,
    &[u8],
    (i32, i32, i32, &str, usize),
    (i32, i32, i32, &str, usize),
)] = &[
    // label, source, CAP-053 (status, code, actual, text, nodes), CAP-054 same
    (
        "lock-call-in-return",
        b"fn f() -> int { return a(b); } x",
        (10, 18, 10, "(", 1),
        (10, 0, 1, "x", 3),
    ),
    (
        "lock-call-in-condition",
        b"fn f() -> int { if a(b) { c = 1; } return 2; } x",
        (10, 12, 10, "(", 1),
        (10, 0, 1, "x", 5),
    ),
    (
        "lock-reference-in-return",
        b"fn f() -> int { return &a; } x",
        (11, 100, 37, "&", 0),
        (11, 100, 37, "&", 0),
    ),
    (
        "lock-two-identifiers",
        b"fn f() -> int { return a b; } x",
        (10, 18, 1, "b", 1),
        (10, 18, 1, "b", 1),
    ),
    (
        "lock-grouped-operand",
        b"fn f() -> int { return (a); } x",
        (10, 0, 1, "x", 1),
        (10, 0, 1, "x", 1),
    ),
    (
        "lock-prefix-operand",
        b"fn f() -> int { return -a; } x",
        (10, 0, 1, "x", 2),
        (10, 0, 1, "x", 2),
    ),
    (
        "lock-assignment-two-identifiers",
        b"fn f() -> int { a = b c; } x",
        (10, 18, 1, "c", 1),
        (10, 18, 1, "c", 1),
    ),
];

fn model_lock_targets() -> Vec<(&'static str, oracle::Ingestion, oracle::Ingestion)> {
    let mut graded = Vec::new();
    for (label, source, cap053, cap054) in MODEL_LOCK_SHAPES {
        let ingested = oracle::ingest(
            source,
            &oracle::Bounds {
                source: H1A_SOURCE_BOUND,
                token: H1A_TOKEN_BOUND,
                name: H1A_NAME_BOUND,
                ampersand: true,
            },
        );
        assert_eq!(ingested.status, 0, "lock `{label}` must lex completely");
        for (model, stopped) in [
            (cap053, oracle::control_flow_parser_stop(&ingested, source)),
            (cap054, oracle::call_parser_stop(&ingested, source)),
        ] {
            let (status, code, actual, text, nodes) = *model;
            let from = usize::try_from(stopped.error_offset).expect("bounded offset");
            assert_eq!(stopped.status, status, "lock `{label}` status");
            assert_eq!(stopped.diagnostic_code, code, "lock `{label}` code");
            assert_eq!(stopped.diagnostic_actual, actual, "lock `{label}` actual");
            assert_eq!(
                &source[from..from + text.len()],
                text.as_bytes(),
                "lock `{label}` token"
            );
            assert_eq!(stopped.nodes.len(), nodes, "lock `{label}` node count");
        }
        graded.push((
            *label,
            oracle::control_flow_parser_stop(&ingested, source),
            oracle::call_parser_stop(&ingested, source),
        ));
    }
    graded
}

/// The anti-fitting lock, product-free: neither model may drift on a shape no
/// probe table covers, and the two must still disagree where the checkpoint
/// says they do.
#[test]
fn neither_model_drifts_outside_the_probe_tables() {
    assert_eq!(model_lock_targets().len(), MODEL_LOCK_SHAPES.len());
    let disagreements = MODEL_LOCK_SHAPES
        .iter()
        .filter(|(_, _, cap053, cap054)| cap053 != cap054)
        .count();
    assert_eq!(
        disagreements, 2,
        "the lock must contain shapes the two models decide differently"
    );
}

/// The same out-of-table shapes, run against the real linked product.
#[test]
fn the_out_of_table_shapes_run_against_the_product() {
    for ((label, source, ..), (_, _cap053, target)) in
        MODEL_LOCK_SHAPES.iter().zip(model_lock_targets())
    {
        assert_eq!(
            run_expectation("model-lock", compiled_h1a(), source, &target, "-O0"),
            91,
            "out-of-table shape `{label}` diverged from the CAP-054 model"
        );
    }
}

/// CAP-054 / H1B-5 leaves the canonical self-ingestion stop exactly where
/// CAP-051 put it, and this is a regression guard rather than evidence of
/// forward progress.
///
/// The canonical source's first function contains no call and no reference, so
/// the call store is never pushed on the canonical run and all four earlier
/// models must agree with the call model exactly, including the four orphan
/// arm-body nodes.
#[test]
fn the_call_checkpoint_leaves_the_canonical_stop_unmoved() {
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

    let frozen = oracle::SignatureStop {
        status: 10,
        error_offset: 146,
        error_line: 8,
        error_column: 1,
        diagnostic_code: 0,
        diagnostic_actual: 3,
        node_count: 4,
        parameters: 1,
    };
    assert_eq!(
        oracle::control_flow_grammar_stop(&ingested, &source),
        frozen,
        "CAP-053's frozen target moved"
    );
    assert_eq!(
        oracle::call_grammar_stop(&ingested, &source),
        frozen,
        "CAP-054 must not move the canonical stop in either direction"
    );
    assert_eq!(&source[146..148], b"fn");

    let stopped = oracle::call_parser_stop(&ingested, &source);
    assert_eq!(stopped.nodes.len(), 4);
    assert_eq!(stopped.origins.len(), 4);

    for optimization in ["-O0", "-O2"] {
        assert_eq!(
            run_expectation(
                "call-self-ingestion",
                compiled_h1a(),
                &source,
                &stopped,
                optimization
            ),
            91,
            "CAP-054 self-ingestion diverged from the independent oracle at {optimization}"
        );
    }
}

// ---------------------------------------------------------------------------
// CAP-055 / H1B-6 - arena capacity
//
// The five parse-group record stores are `node_count`, `value_records`,
// `operator_records`, `block_records` and `call_records`. All five are never
// decremented, so each counts total pushes over the whole parse rather than
// live depth: a pop rewinds only a link, and the record arrays have an append
// path (`parser_append_target`) and no write-at-index path. "512 value records"
// was never a limit on expression complexity - the deepest the value stack gets
// on the entire canonical source is five.
//
// Every one of the five arrays is created by `bytes_new()` and grows by append,
// so all five bounds are policy ceilings and not preallocations. Raising them
// costs nothing until the capacity is used.
// ---------------------------------------------------------------------------

/// The single literal `compiler.aero` compares all five parse-group stores
/// against, and reports as `diagnostic_code` when one is exhausted.
///
/// The verifier's own `512` at `compiler.aero:5557` is deliberately *not* this
/// constant. It bounds `verified_function_node`, lives in the verifier group -
/// which `BOOTSTRAP_CONVERGENCE_READINESS.md:265-267` forbids H1B to widen -
/// and cannot bite inside H1B at all, because the verifier runs only on a
/// complete `status == 0` pipeline that no H1B checkpoint reaches. It is
/// recorded as debt for whichever checkpoint first drives the verifier over one
/// function.
const PARSE_RECORD_BOUND: usize = 65_536;

/// What the canonical source actually needs, measured for the 293,592-byte
/// tree CAP-054 left and recorded in `TASK_LEDGER.md` under "H1B-6
/// arena-capacity measurement" and at
/// `BOOTSTRAP_CONVERGENCE_READINESS.md:352-359`. This checkpoint consumes the
/// measurement and does not repeat it.
const CANONICAL_RECORD_REQUIREMENT: [(&str, usize); 5] = [
    ("node", 17_621),
    ("value", 15_842),
    ("operator", 6_030),
    ("block", 1_289),
    ("call", 1_120),
];

/// `fn f() -> int { return ` - every capacity probe shares this prefix, so a
/// located diagnostic's offset is derivable in one line.
const PROBE_PREFIX: &[u8] = b"fn f() -> int { return ";

/// A left-associative chain of `leaves` integer operands joined by `+`.
///
/// Equal precedence reduces eagerly, so the operator stack never gets deeper
/// than one and the arena holds `2k - 2` nodes after leaf *k* and `2k - 1`
/// after the reduction that follows it. `leaves` integers therefore produce
/// exactly `2 * leaves - 1` nodes, which is how a probe lands on a chosen side
/// of the bound to the record.
fn node_chain_probe(leaves: usize) -> Vec<u8> {
    let mut source = PROBE_PREFIX.to_vec();
    for index in 0..leaves {
        if index > 0 {
            source.push(b'+');
        }
        source.push(b'1');
    }
    source.extend_from_slice(b"; } x");
    source
}

/// `opens` grouping parentheses in operand position, each one operator record
/// and no node at all. The chain never closes, which is deliberate: the stop
/// happens on the way down, so the probe proves the operator store is exhausted
/// with an empty node arena rather than as a side effect of node pressure.
fn grouping_chain_probe(opens: usize) -> Vec<u8> {
    let mut source = PROBE_PREFIX.to_vec();
    source.extend(std::iter::repeat_n(b'(', opens));
    source.extend_from_slice(b"1; } x");
    source
}

/// `opens` nested calls, each one call record *and* one operator record, and no
/// node until a call closes - which this probe never lets happen.
fn call_chain_probe(opens: usize) -> Vec<u8> {
    let mut source = PROBE_PREFIX.to_vec();
    for _ in 0..opens {
        source.extend_from_slice(b"f(");
    }
    source.extend_from_slice(b"1; } x");
    source
}

fn capacity_ingest(source: &[u8]) -> oracle::Ingestion {
    let ingested = oracle::ingest(
        source,
        &oracle::Bounds {
            source: H1A_SOURCE_BOUND,
            token: H1A_TOKEN_BOUND,
            name: H1A_NAME_BOUND,
            ampersand: true,
        },
    );
    assert_eq!(
        ingested.status, 0,
        "a capacity probe must lex completely, so that what it proves is a \
         record ceiling and not the token or source ceiling"
    );
    ingested
}

/// The four capacity shapes, with the expectation hand-derived from the
/// accounting table and the check placement in `compiler.aero`, before any run.
///
/// `(label, source, status, code, actual, offset, nodes)`. `offset` is derived
/// arithmetically from `PROBE_PREFIX` rather than searched for, so a probe that
/// stopped at a plausible-looking wrong token would fail.
fn capacity_probe_table() -> Vec<(&'static str, Vec<u8>, i32, i32, i32, i32, usize)> {
    let bound = PARSE_RECORD_BOUND;
    let prefix = i32::try_from(PROBE_PREFIX.len()).expect("bounded prefix");
    let wide = i32::try_from(bound).expect("bounded record ceiling");
    vec![
        // The arena fills to `bound - 1` and the parse reaches the trailing
        // `x`, an ordinary grammar stop. This is the checkpoint's positive: at
        // 512 the identical shape cannot hold a hundredth of these records.
        (
            "node-under",
            node_chain_probe(bound / 2),
            10,
            0,
            1,
            0,
            bound - 1,
        ),
        // One more leaf makes the closing reduction the `bound + 1`-th append.
        // `compiler.aero:2622` locates it at the pending operator's own origin -
        // the last `+` in the source - and reports `reduction_actual`, which for
        // a binary operator is that token's kind, 20.
        (
            "node-over",
            node_chain_probe(bound / 2 + 1),
            14,
            wide,
            20,
            prefix + wide - 1,
            bound,
        ),
        // `compiler.aero:2263`. The `bound + 1`-th `(` is at `prefix + bound`,
        // and no node has been appended at all.
        (
            "operator-over",
            grouping_chain_probe(bound + 1),
            15,
            wide,
            10,
            prefix + wide,
            0,
        ),
        // `compiler.aero:2191`, one check over both the call store and the
        // operator store. A nested call pushes one of each, so under a uniform
        // bound they are exhausted on the same open. The stop is located at the
        // *held callee* and reports the `(` that ended the hold.
        (
            "call-over",
            call_chain_probe(bound + 1),
            15,
            wide,
            10,
            prefix + 2 * wide,
            0,
        ),
    ]
}

/// The capacity model must reproduce the CAP-054 model exactly on every shape
/// that stays under the bound, including the shapes no capacity probe covers.
///
/// This is the product-free proof that CAP-055 changed no grammar. It is not a
/// tautology even though `call_parser_stop` is `capacity_parser_stop` at
/// `Caps::UNBOUNDED`: a check placed one step early - before a reject the
/// grammar was going to raise anyway, or on a path the product does not guard -
/// would change an under-bound answer and this would catch it.
#[test]
fn the_capacity_model_agrees_with_the_call_model_under_the_bound() {
    let caps = oracle::Caps::uniform(PARSE_RECORD_BOUND);
    let mut compared = 0;
    for (label, source, ..) in MODEL_LOCK_SHAPES {
        let ingested = capacity_ingest(source);
        assert_eq!(
            oracle::expectation_vector(
                source,
                &oracle::capacity_parser_stop(&ingested, source, &caps)
            ),
            oracle::expectation_vector(source, &oracle::call_parser_stop(&ingested, source)),
            "capacity model drifted from CAP-054 on the out-of-table shape `{label}`"
        );
        compared += 1;
    }
    let under = node_chain_probe(PARSE_RECORD_BOUND / 2);
    let ingested = capacity_ingest(&under);
    assert_eq!(
        oracle::expectation_vector(
            &under,
            &oracle::capacity_parser_stop(&ingested, &under, &caps)
        ),
        oracle::expectation_vector(&under, &oracle::call_parser_stop(&ingested, &under)),
        "capacity model drifted from CAP-054 one record below the bound"
    );
    compared += 1;
    assert_eq!(compared, MODEL_LOCK_SHAPES.len() + 1);
}

/// The deliberate out-of-table grading, and the result that makes it worth
/// running.
///
/// CAP-054's model has no record ceiling of any kind, so on every over-bound
/// shape it does not merely get a detail wrong - it reports an ordinary grammar
/// stop where the product reports an exhausted arena. The accepted oracle was
/// blind to a whole class of product behaviour while twenty-six probes passed
/// green around it, because no probe in any table ever exceeded 512 records.
#[test]
fn the_call_model_is_blind_to_every_over_bound_shape() {
    let caps = oracle::Caps::uniform(PARSE_RECORD_BOUND);
    let mut divergences = 0;
    for (label, source, status, ..) in capacity_probe_table() {
        let ingested = capacity_ingest(&source);
        let bounded = oracle::capacity_parser_stop(&ingested, &source, &caps);
        let cap054 = oracle::call_parser_stop(&ingested, &source);
        assert!(
            cap054.status != 14 && cap054.status != 15,
            "CAP-054's model cannot report a capacity stop, yet `{label}` produced {}",
            cap054.status
        );
        if status == 14 || status == 15 {
            assert_ne!(
                oracle::expectation_vector(&source, &bounded),
                oracle::expectation_vector(&source, &cap054),
                "`{label}` must separate the two models"
            );
            divergences += 1;
        } else {
            assert_eq!(
                oracle::expectation_vector(&source, &bounded),
                oracle::expectation_vector(&source, &cap054),
                "`{label}` stays under the bound and must not separate them"
            );
        }
    }
    assert_eq!(divergences, 3, "three of the four shapes exceed a ceiling");
}

/// The hand-derived predictions, graded against the model before the product is
/// asked anything.
#[test]
fn every_capacity_probe_stops_where_this_checkpoint_predicted() {
    let caps = oracle::Caps::uniform(PARSE_RECORD_BOUND);
    for (label, source, status, code, actual, offset, nodes) in capacity_probe_table() {
        let ingested = capacity_ingest(&source);
        let stopped = oracle::capacity_parser_stop(&ingested, &source, &caps);
        assert_eq!(stopped.status, status, "`{label}` status");
        assert_eq!(stopped.diagnostic_code, code, "`{label}` diagnostic_code");
        assert_eq!(
            stopped.diagnostic_actual, actual,
            "`{label}` diagnostic_actual"
        );
        assert_eq!(stopped.nodes.len(), nodes, "`{label}` node records");
        if status != 10 {
            assert_eq!(stopped.error_offset, offset, "`{label}` located offset");
            assert_eq!(stopped.error_line, 1, "`{label}` located line");
            assert_eq!(stopped.error_column, offset + 1, "`{label}` located column");
        }
        assert_eq!(stopped.origins.len(), nodes, "`{label}` origin sidecar");
    }
}

/// The same four shapes against the real linked product. This is the raise:
/// `node-under` holds 65,535 node records, which is 128 times what the accepted
/// product admitted, and the three `-over` shapes fail closed on the new bound
/// with the located diagnostic intact rather than turning a capacity stop into
/// a silent success.
///
/// The full expectation vector is asserted, so a stop that lost its offset,
/// line or column, or that reported `status = 0`, would fail here.
#[test]
fn every_capacity_probe_agrees_with_the_product() {
    let caps = oracle::Caps::uniform(PARSE_RECORD_BOUND);
    for (label, source, ..) in capacity_probe_table() {
        let ingested = capacity_ingest(&source);
        let stopped = oracle::capacity_parser_stop(&ingested, &source, &caps);
        assert_eq!(
            run_expectation("capacity", compiled_h1a(), &source, &stopped, "-O0"),
            91,
            "capacity probe `{label}` diverged from the independent oracle"
        );
    }
}

/// Two of the five ceilings cannot be reached, and both facts are derived
/// rather than discovered by a probe that failed to build.
///
/// A value push is paired with a node append at all four of its sites, and
/// three node appends have no value push at all - the function node, the root
/// and the argument cell - so `value_records <= node_count` always. At each
/// value check the paired node increment has already happened and the node
/// check guarding the same path passed, so `value_records < B` whenever the
/// value check runs: at a uniform bound it can never fire. That was true at 512
/// too.
///
/// A block push costs at least 6.5 tokens, because an empty block is rejected
/// and the cheapest two-block shape is `if 1 { return 1; } else { return 1; }`
/// at 13 tokens. 65,537 pushes therefore need at least 425,990 tokens against
/// the frozen 262,144-token bound, which fires first. If a later checkpoint
/// raises the token bound past roughly 426,000, the block ceiling becomes
/// reachable and acquires a boundary no probe here covers.
///
/// Both paths are still live code, and this exercises them at a *non-uniform*
/// bound, which is the only arrangement that can reach either. This is
/// model-only evidence and is labelled model-only; it is not offered as product
/// evidence for those two ceilings.
#[test]
fn the_value_and_block_ceilings_are_live_paths_at_a_non_uniform_bound() {
    let unreachable_uniformly = oracle::Caps::uniform(PARSE_RECORD_BOUND);

    // Five operands need nine value pushes - five leaves and four reductions -
    // so a ceiling of four stops on the reduction that would be the fifth.
    let source = node_chain_probe(5);
    let ingested = capacity_ingest(&source);
    assert_eq!(
        oracle::capacity_parser_stop(&ingested, &source, &unreachable_uniformly).status,
        10,
        "at a uniform bound this shape is an ordinary grammar stop"
    );
    let value_pinched = oracle::Caps {
        values: 4,
        ..oracle::Caps::UNBOUNDED
    };
    let stopped = oracle::capacity_parser_stop(&ingested, &source, &value_pinched);
    assert_eq!(stopped.status, 15, "the value ceiling must fail closed");
    assert_eq!(stopped.diagnostic_code, 4, "and report the ceiling it hit");
    assert_eq!(stopped.error_line, 1);
    assert!(
        stopped.error_offset > 0 && stopped.error_column > 1,
        "a capacity stop must stay located"
    );

    // Three nested blocks, none of them empty, under a ceiling of two.
    let blocks = b"fn f() -> int { if 1 { a = 1; } else { a = 1; } if 1 { a = 1; } return 1; } x";
    let ingested = capacity_ingest(blocks);
    assert_eq!(
        oracle::capacity_parser_stop(&ingested, blocks, &unreachable_uniformly).status,
        10,
        "at a uniform bound this shape is an ordinary grammar stop"
    );
    let block_pinched = oracle::Caps {
        blocks: 2,
        ..oracle::Caps::UNBOUNDED
    };
    let stopped = oracle::capacity_parser_stop(&ingested, blocks, &block_pinched);
    assert_eq!(stopped.status, 15, "the block ceiling must fail closed");
    assert_eq!(stopped.diagnostic_code, 2, "and report the ceiling it hit");
    assert_eq!(
        stopped.diagnostic_actual, 12,
        "located on the opening brace"
    );
    assert_eq!(stopped.error_line, 1);
}

/// The raised bound admits what the canonical source measurably requires, with
/// margin, and the old one did not admit any of it.
///
/// This is arithmetic over a committed measurement rather than a fresh
/// measurement. It is a test so that a later source growth that overruns the
/// ceiling is caught here instead of at the module-shape gate, which is the
/// place `BOOTSTRAP_CONVERGENCE_READINESS.md:333-335` says capacity must never
/// be allowed to masquerade as a grammar failure.
#[test]
fn the_raised_bound_covers_the_measured_canonical_requirement() {
    for (arena, required) in CANONICAL_RECORD_REQUIREMENT {
        assert!(
            required > 512,
            "the {arena} store's requirement of {required} must exceed the bound \
             this checkpoint replaced, or the checkpoint has no reason to exist"
        );
        assert!(
            required < PARSE_RECORD_BOUND,
            "the {arena} store needs {required} records and the bound is \
             {PARSE_RECORD_BOUND}"
        );
    }
    // The upper projection of the same measurement, which is what the bound was
    // derived from.
    //
    // A correction to the measurement, left visible rather than restated. It
    // says 65,536 is 2.5x that; 65,536 / 26,332 is 2.4888, so 2.5x is a rounded
    // figure reported as exact. Nothing depends on the difference - the
    // recommendation stands and the bound is unchanged - but the claim that is
    // exactly true is the weaker one, that the bound is at least *twice* the
    // upper projection, which is the margin the derivation actually argues for:
    // a source that doubles still fits. The second assertion pins the
    // correction so the 2.5x claim cannot quietly return.
    const UPPER_PROJECTION: usize = 26_332;
    assert!(UPPER_PROJECTION * 2 <= PARSE_RECORD_BOUND);
    assert!(
        UPPER_PROJECTION * 5 / 2 > PARSE_RECORD_BOUND,
        "2.5x the upper projection is 65,830 and overstates the bound"
    );
}

/// CAP-055 / H1B-6 leaves the canonical self-ingestion stop exactly where
/// CAP-051 put it. A capacity raise must not move it; if it moves, something
/// other than capacity changed.
///
/// The canonical run stops four nodes in, so it never approaches any of the
/// five ceilings and the bounded model must agree with all five earlier ones.
#[test]
fn the_capacity_checkpoint_leaves_the_canonical_stop_unmoved() {
    let source = fs::read(repository_path(H1A_PRODUCT)).expect("read CAP-049 canonical source");
    let ingested = capacity_ingest(&source);

    let frozen = oracle::SignatureStop {
        status: 10,
        error_offset: 146,
        error_line: 8,
        error_column: 1,
        diagnostic_code: 0,
        diagnostic_actual: 3,
        node_count: 4,
        parameters: 1,
    };
    assert_eq!(
        oracle::call_grammar_stop(&ingested, &source),
        frozen,
        "the CAP-054 target moved"
    );
    let caps = oracle::Caps::uniform(PARSE_RECORD_BOUND);
    let stopped = oracle::capacity_parser_stop(&ingested, &source, &caps);
    assert_eq!(
        oracle::expectation_vector(&source, &stopped),
        oracle::expectation_vector(&source, &oracle::call_parser_stop(&ingested, &source)),
        "CAP-055 must not move the canonical stop in either direction"
    );
    assert_eq!(stopped.nodes.len(), 4);
    assert_eq!(stopped.origins.len(), 4);
    assert_eq!(&source[146..148], b"fn");

    for optimization in ["-O0", "-O2"] {
        assert_eq!(
            run_expectation(
                "capacity-self-ingestion",
                compiled_h1a(),
                &source,
                &stopped,
                optimization
            ),
            91,
            "CAP-055 self-ingestion diverged from the independent oracle at {optimization}"
        );
    }
}

/// The bound lives at exactly the parse-group sites and the verifier's own
/// ceiling is untouched. A future edit that raised the verifier's `512` because
/// it shares a literal with the others - the mistake the ledger explicitly
/// warns against - fails here.
///
/// The counts correct this checkpoint's own contract, which said "sixteen
/// parse-group sites - eight comparisons and eight `diagnostic_code`
/// assignments". The product carries **sixteen comparison conditions** - one of
/// which, `compiler.aero:2191`, tests two stores and so holds two occurrences -
/// and **sixteen `diagnostic_code` assignments`: 33 occurrences over 32 lines,
/// not 16. `65536` is not counted directly, because the source already uses it
/// 94 times as the 2^16 byte-packing multiplier and such a count would prove
/// nothing.
#[test]
fn the_raise_touched_the_parse_group_and_left_the_verifier_alone() {
    let source = fs::read_to_string(repository_path(H1A_PRODUCT)).expect("read canonical source");
    let stores = [
        "block_records",
        "operator_records",
        "call_records",
        "node_count",
        "value_records",
    ];
    let bound = PARSE_RECORD_BOUND;

    let comparisons = source
        .lines()
        .filter(|line| {
            stores
                .iter()
                .any(|store| line.contains(&format!("{store} >= {bound}")))
        })
        .count();
    assert_eq!(comparisons, 16, "sixteen parse-group comparison conditions");
    let occurrences: usize = source
        .lines()
        .map(|line| {
            stores
                .iter()
                .filter(|store| line.contains(&format!("{store} >= {bound}")))
                .count()
        })
        .sum();
    assert_eq!(
        occurrences, 17,
        "seventeen compared stores, because `compiler.aero:2191` guards the          call store and the operator store in one condition"
    );
    let codes = source
        .lines()
        .filter(|line| line.trim() == format!("diagnostic_code = {bound};"))
        .count();
    assert_eq!(codes, 16, "every exhaustion reports the bound it hit");

    let remaining: Vec<&str> = source
        .lines()
        .filter(|line| line.contains("512"))
        .map(|line| line.trim())
        .collect();
    assert_eq!(
        remaining,
        vec![
            "&& (verified_function_node < 3 || verified_function_node > 512) {",
            "verified_expected = 512;",
        ],
        "the only `512` left in the product must be the verifier's, which is          recorded as debt and is not H1B's to widen"
    );
}

/// Drive the block store past the bound this checkpoint replaced, through the
/// real product, rather than compare a number against it.
///
/// The block store is the one of the five that no `-over` probe can reach:
/// 65,537 pushes need at least 425,990 tokens against the frozen 262,144-token
/// bound, so the ceiling is unreachable and there is no boundary to exercise.
/// The raise is therefore proven from the other side, and this is the test that
/// makes the other four probes' evidence complete rather than partial.
///
/// The defect it rules out is specific: a guard raised without the storage
/// behind it. Structurally that cannot happen here - all five arenas are
/// `bytes_new()` buffers written by the same dispatch at
/// `compiler.aero:3372-3395`, and `bytes_push` doubles its capacity through
/// `aero_realloc` - but "the guard moved and the storage did not" is exactly
/// the shape of a defect that passes a gate and fails later, so it is measured
/// rather than argued. If the block array had a fixed 512-record allocation,
/// `bytes_push` would return negative at record 513 and `compiler.aero:3404`
/// would report `status = 8`, which is not the stop asserted here.
///
/// 1,300 blocks is chosen to exceed the canonical requirement of 1,289, so the
/// probe covers the capacity the self-source actually needs from this store.
#[test]
fn the_product_holds_more_block_records_than_the_replaced_bound() {
    const BLOCKS: usize = 1_300;
    assert!(
        BLOCKS > 512,
        "the probe must exceed the bound being replaced"
    );
    assert!(
        BLOCKS > 1_289,
        "and the canonical source's measured block requirement"
    );

    let mut source = b"fn f() -> int { ".to_vec();
    for _ in 0..BLOCKS / 2 {
        source.extend_from_slice(b"if 1 { a = 1; } else { a = 1; } ");
    }
    source.extend_from_slice(b"return 1; } x");
    let ingested = capacity_ingest(&source);
    let caps = oracle::Caps::uniform(PARSE_RECORD_BOUND);
    let stopped = oracle::capacity_parser_stop(&ingested, &source, &caps);
    assert_eq!(
        stopped.status, 10,
        "the shape must reach an ordinary grammar stop, not a capacity one"
    );

    // The record count is pinned by bracketing rather than asserted from the
    // model's own bookkeeping: a ceiling of exactly `BLOCKS` admits the parse
    // and one record less fails closed on the block store. Both cannot hold if
    // the shape pushes some other number of records.
    let exact = oracle::Caps {
        blocks: BLOCKS,
        ..oracle::Caps::UNBOUNDED
    };
    let short = oracle::Caps {
        blocks: BLOCKS - 1,
        ..oracle::Caps::UNBOUNDED
    };
    assert_eq!(
        oracle::capacity_parser_stop(&ingested, &source, &exact).status,
        10,
        "a ceiling of exactly {BLOCKS} must admit the parse"
    );
    let pinched = oracle::capacity_parser_stop(&ingested, &source, &short);
    assert_eq!(pinched.status, 15, "one record short must fail closed");
    assert_eq!(
        pinched.diagnostic_code,
        i32::try_from(BLOCKS - 1).expect("bounded ceiling")
    );

    // Reaching the predicted stop against the linked product means the
    // product's own block array actually grew to hold 1,300 records, and the
    // harness's allocation assertions mean it was freed cleanly afterwards.
    assert_eq!(
        run_expectation("block-storage", compiled_h1a(), &source, &stopped, "-O0"),
        91,
        "the product's block store did not hold {BLOCKS} records"
    );
}
