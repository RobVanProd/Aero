use compiler::{
    CompilerOptions, LanguageProfile, LlvmVerificationMode, Token, check_file, check_program,
    compile_file, compile_program, try_tokenize_with_locations, verify_llvm_module,
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
const PROFILE_NAME: &str = "exact-i32-byte-input-v0";
const PRODUCT_RELATIVE_PATH: &str = "../../examples/aero_frontend_v0/runtime_ascii_lexer.aero";
const WORKFLOW_RELATIVE_PATH: &str = "../../.github/workflows/rust.yml";
const SELF_TEST_MARKER: &str = "// CAP-041 TRACKED SELF-TEST";
const INTENTIONAL_PRODUCT_RED: &str =
    "CAP-041 intentional product red: tracked runtime ASCII lexer is absent";

static NEXT_WORKSPACE_ID: AtomicU64 = AtomicU64::new(0);

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
            .join("cap041-runtime-lexer-tests");
        let root = parent.join(format!("{label}-{}-{nonce}-{serial}", std::process::id()));
        fs::create_dir_all(&root).expect("create CAP-041 test workspace");
        Self { root }
    }

    fn write(&self, name: &str, contents: impl AsRef<[u8]>) -> PathBuf {
        let path = self.root.join(name);
        fs::write(&path, contents).expect("write CAP-041 artifact");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let valid = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("cap041-"));
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
        .expect("spawn CAP-041 child");
    child
        .stdin
        .take()
        .expect("CAP-041 child stdin")
        .write_all(input)
        .expect("write CAP-041 child stdin");
    child.wait_with_output().expect("wait for CAP-041 child")
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
    let output = command.output().expect("execute Clang for CAP-041");
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

fn generated_program(kernel_prefix: &str, model: &LexModel, checksum_delta: i32) -> String {
    format!(
        "{}\n\nfn main() -> int {{\n    return run_runtime_ascii_lexer({}, {}, {}, {}, {}, {}, {});\n}}\n",
        kernel_prefix.trim_end(),
        model.status,
        model.error_offset,
        model.error_line,
        model.error_column,
        model.names.len(),
        model.tokens.len(),
        model.checksum + checksum_delta,
    )
}

fn compile_generated(program: &str) -> String {
    check_program(program, options()).expect("generated CAP-041 program checks");
    let first = compile_program(program, options()).expect("generated CAP-041 program compiles");
    let second = compile_program(program, options()).expect("generated CAP-041 program recompiles");
    assert_eq!(first, second, "generated CAP-041 LLVM is nondeterministic");
    verify_llvm_module(&first, LlvmVerificationMode::Required)
        .expect("generated CAP-041 LLVM verifies");
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
    model: &LexModel,
    fail_after: u64,
) -> AllocationExpectation {
    let mut buffers = [BufferState::default(); 3];
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
    AllocationExpectation {
        success: completed,
        allocations,
        reallocations,
        deallocations: buffers.iter().filter(|buffer| buffer.capacity != 0).count() as u64,
    }
}

fn allocation_harness(input: &[u8], model: &LexModel) -> String {
    let mut cases = String::new();
    for threshold in [0u64, 1, 2, 3, 4, 6, 9, 11, 12] {
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
        .expect("write CAP-041 allocation case");
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
    let product_path = repository_path(PRODUCT_RELATIVE_PATH);
    let product =
        fs::read_to_string(&product_path).unwrap_or_else(|_| panic!("{INTENTIONAL_PRODUCT_RED}"));
    let (kernel_prefix, tracked_main) = product
        .split_once(SELF_TEST_MARKER)
        .expect("tracked product retains one lexer/self-test boundary");

    assert_eq!(product.matches(SELF_TEST_MARKER).count(), 1);
    assert_eq!(product.matches("fn run_runtime_ascii_lexer(").count(), 1);
    assert!(tracked_main.contains("fn main() -> int"));
    assert!(tracked_main.contains("run_runtime_ascii_lexer(0, -1, 0, 0, 2, 12, 602295)"));
    for owner in ["source", "names", "tokens"] {
        assert!(
            product.contains(&format!("let mut {owner}: ByteBuffer = bytes_new();")),
            "tracked CAP-041 product omitted `{owner}`"
        );
    }
    for forbidden in [
        "String", "Vec", "HashMap", "print", "mod ", "use ", "unsafe",
    ] {
        assert!(
            !product.contains(forbidden),
            "tracked CAP-041 product contains `{forbidden}`"
        );
    }

    check_program(&product, options()).expect("tracked CAP-041 product checks");
    let first = compile_program(&product, options()).expect("tracked CAP-041 product compiles");
    let second = compile_program(&product, options()).expect("tracked CAP-041 product recompiles");
    assert_eq!(first, second, "tracked CAP-041 LLVM is nondeterministic");
    verify_llvm_module(&first, LlvmVerificationMode::Required)
        .expect("tracked CAP-041 LLVM verifies");
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
    let tracked_source = workspace.write("runtime_ascii_lexer.aero", &product);
    check_file(&tracked_source, options()).expect("tracked CAP-041 file checks");
    assert_eq!(
        compile_file(&tracked_source, options()).expect("tracked CAP-041 file compiles"),
        first,
        "tracked CAP-041 source/file LLVM diverged"
    );

    let canonical = b"fn f()->int{return f;}".to_vec();
    let canonical_model = reference_lex(&canonical);
    assert_eq!(canonical_model.checksum, 602_295);
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
            &format!("tracked CAP-041 {optimization}"),
        );
    }

    let located = b"let alpha\r\n\talpha/* x\ny */+12".to_vec();
    let unsupported = b"fn @".to_vec();
    let non_ascii = vec![b'\n', 0xff];
    let token_bound = "x ".repeat(MAX_REAL_TOKENS + 1).into_bytes();
    let oversized = vec![b' '; MAX_INPUT_BYTES + 1];
    for (label, input) in [
        ("empty", Vec::new()),
        ("located", located),
        ("unsupported", unsupported),
        ("non-ascii", non_ascii),
        ("token-bound", token_bound),
        ("input-bound", oversized),
    ] {
        let model = reference_lex(&input);
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
            &format!("generated CAP-041 {label}"),
        );
    }

    let wrong_program = generated_program(kernel_prefix, &canonical_model, 1);
    let wrong_llvm = compile_generated(&wrong_program);
    let wrong = TestWorkspace::new("wrong-oracle");
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
        "wrong independent expectation was accepted"
    );

    let corrupted_kernel = kernel_prefix.replacen("return 3;", "return 4;", 1);
    assert_ne!(
        corrupted_kernel, kernel_prefix,
        "corruption anchor was absent"
    );
    let corrupt_program = generated_program(&corrupted_kernel, &canonical_model, 0);
    let corrupt_llvm = compile_generated(&corrupt_program);
    let corrupt = TestWorkspace::new("corrupt-keyword");
    let corrupt_path = corrupt.write("corrupt.ll", corrupt_llvm);
    let corrupt_executable = clang_link(
        "corrupt",
        &corrupt,
        &[corrupt_path.as_path(), runtime.as_path()],
        "-O2",
    );
    assert_ne!(
        run_command_with_stdin(&mut Command::new(corrupt_executable), &canonical)
            .status
            .code(),
        Some(91),
        "keyword corruption was accepted"
    );

    let renamed = first.replacen("define i32 @main()", "define i32 @aero_program_main()", 1);
    assert_ne!(renamed, first, "tracked CAP-041 product omitted main");
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
            .expect("execute CAP-041 allocation harness"),
        "CAP-041 allocation/failure matrix",
    );

    let public = TestWorkspace::new("public-run");
    let public_source = public.write(
        "runtime_ascii_lexer.aero",
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
        "public CAP-041 runner failed"
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
        "public CAP-041 runner emitted stderr"
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
            .expect("execute CAP-041 accelerator rejection");
        assert_eq!(output.status.code(), Some(2));
        assert!(
            !output_path.exists(),
            "{target} rejection created an artifact"
        );
    }

    let workflow =
        fs::read_to_string(repository_path(WORKFLOW_RELATIVE_PATH)).expect("read Rust workflow");
    let linux = workflow_step(&workflow, "Test runtime ASCII lexer at O0 and O2");
    for anchor in [
        "runtime_ascii_lexer.aero",
        "exact-i32-byte-input-v0",
        "runtime_ascii_lexer.linux.repeat.ll",
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
            "Linux CAP-041 step omitted `{anchor}`"
        );
    }
    let windows = workflow_step(
        &workflow,
        "Test runtime ASCII lexer on Windows at O0 and O2",
    );
    for anchor in [
        "runtime_ascii_lexer.aero",
        "exact-i32-byte-input-v0",
        "runtime_ascii_lexer.windows.repeat.ll",
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
            "Windows CAP-041 step omitted `{anchor}`"
        );
    }
}
