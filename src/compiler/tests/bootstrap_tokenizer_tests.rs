use compiler::{CompilerOptions, LanguageProfile, check_program, compile_program};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const SOURCE_CAPACITY: usize = 64;
const TOKEN_CAPACITY: usize = 24;
const OUTPUT_LANES: usize = 3 + TOKEN_CAPACITY * 3;
const PRODUCT_RELATIVE_PATH: &str =
    "../../examples/fixed_int_array_v0/bootstrap_ascii_tokenizer.aero";
const WORKFLOW_RELATIVE_PATH: &str = "../../.github/workflows/rust.yml";
const SELF_TEST_MARKER: &str = "// CAP-031 TRACKED SELF-TEST";
const INTENTIONAL_PRODUCT_RED: &str =
    "CAP-031 intentional product red: tracked bounded ASCII token-span kernel is absent";

static NEXT_WORKSPACE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
struct Fixture {
    name: &'static str,
    source: Vec<i32>,
    source_len: i32,
}

impl Fixture {
    fn text(name: &'static str, source: &str) -> Self {
        Self {
            name,
            source: source.bytes().map(i32::from).collect(),
            source_len: i32::try_from(source.len()).expect("bounded fixture length"),
        }
    }

    fn lanes(&self) -> [i32; SOURCE_CAPACITY] {
        assert!(self.source.len() <= SOURCE_CAPACITY, "{}", self.name);
        let mut lanes = [0; SOURCE_CAPACITY];
        lanes[..self.source.len()].copy_from_slice(&self.source);
        lanes
    }
}

#[derive(Debug)]
struct TestWorkspace {
    root: PathBuf,
    temp_root: PathBuf,
}

impl TestWorkspace {
    fn new(test_name: &str) -> Self {
        let temp_root = std::env::temp_dir();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();
        let sequence = NEXT_WORKSPACE_ID.fetch_add(1, Ordering::Relaxed);
        let root = temp_root.join(format!(
            "aero-cap031-{test_name}-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create CAP-031 test workspace");
        Self { root, temp_root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let valid_name = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("aero-cap031-"));
        if valid_name && self.root.starts_with(&self.temp_root) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn exact_options() -> CompilerOptions {
    CompilerOptions {
        language_profile: LanguageProfile::ExactI32ArrayV0,
        ..CompilerOptions::default()
    }
}

fn repository_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn keyword_kind(bytes: &[i32]) -> i32 {
    match bytes {
        [102, 110] => 3,
        [108, 101, 116] => 4,
        [109, 117, 116] => 5,
        [114, 101, 116, 117, 114, 110] => 6,
        [105, 102] => 7,
        [101, 108, 115, 101] => 8,
        [119, 104, 105, 108, 101] => 9,
        _ => 1,
    }
}

fn encode_result(
    status: i32,
    error_offset: i32,
    completed: &[(i32, usize, usize)],
    eof_offset: Option<usize>,
) -> [i32; OUTPUT_LANES] {
    let mut output = [0; OUTPUT_LANES];
    output[0] = status;
    output[2] = error_offset;
    let mut records = completed.to_vec();
    if let Some(offset) = eof_offset {
        records.push((0, offset, 0));
    }
    assert!(records.len() <= TOKEN_CAPACITY);
    output[1] = i32::try_from(records.len()).expect("bounded token count");
    for (index, (kind, start, length)) in records.into_iter().enumerate() {
        let base = 3 + index * 3;
        output[base] = kind;
        output[base + 1] = i32::try_from(start).expect("bounded token start");
        output[base + 2] = i32::try_from(length).expect("bounded token length");
    }
    output
}

fn reference_tokenize_ascii_64(
    source: &[i32; SOURCE_CAPACITY],
    source_len: i32,
) -> [i32; OUTPUT_LANES] {
    if !(0..=SOURCE_CAPACITY as i32).contains(&source_len) {
        return encode_result(1, -1, &[], None);
    }

    let length = usize::try_from(source_len).expect("validated nonnegative length");
    let mut completed = Vec::new();
    let mut index = 0;
    while index < length {
        let byte = source[index];
        if !(0..=127).contains(&byte) {
            return encode_result(2, index as i32, &completed, None);
        }

        if matches!(byte, 9 | 10 | 13 | 32) {
            index += 1;
            continue;
        }

        if byte == 47 && index + 1 < length && source[index + 1] == 47 {
            index += 2;
            while index < length {
                let comment_byte = source[index];
                if !(0..=127).contains(&comment_byte) {
                    return encode_result(2, index as i32, &completed, None);
                }
                if comment_byte == 10 {
                    break;
                }
                index += 1;
            }
            continue;
        }

        if byte == 47 && index + 1 < length && source[index + 1] == 42 {
            let comment_start = index;
            index += 2;
            let mut closed = false;
            while index < length {
                let comment_byte = source[index];
                if !(0..=127).contains(&comment_byte) {
                    return encode_result(2, index as i32, &completed, None);
                }
                if comment_byte == 42 && index + 1 < length && source[index + 1] == 47 {
                    index += 2;
                    closed = true;
                    break;
                }
                index += 1;
            }
            if !closed {
                return encode_result(3, comment_start as i32, &completed, None);
            }
            continue;
        }

        let start = index;
        let kind = if byte == 95 || (65..=90).contains(&byte) || (97..=122).contains(&byte) {
            index += 1;
            while index < length {
                let next = source[index];
                if next == 95
                    || (65..=90).contains(&next)
                    || (97..=122).contains(&next)
                    || (48..=57).contains(&next)
                {
                    index += 1;
                } else {
                    break;
                }
            }
            keyword_kind(&source[start..index])
        } else if (48..=57).contains(&byte) {
            index += 1;
            while index < length && (48..=57).contains(&source[index]) {
                index += 1;
            }
            2
        } else {
            let next = (index + 1 < length).then(|| source[index + 1]);
            let (kind, width) = match (byte, next) {
                (61, Some(61)) => (26, 2),
                (61, Some(62)) => (36, 2),
                (33, Some(61)) => (28, 2),
                (60, Some(61)) => (30, 2),
                (62, Some(61)) => (32, 2),
                (38, Some(38)) => (33, 2),
                (124, Some(124)) => (34, 2),
                (45, Some(62)) => (35, 2),
                (40, _) => (10, 1),
                (41, _) => (11, 1),
                (123, _) => (12, 1),
                (125, _) => (13, 1),
                (91, _) => (14, 1),
                (93, _) => (15, 1),
                (44, _) => (16, 1),
                (58, _) => (17, 1),
                (59, _) => (18, 1),
                (46, _) => (19, 1),
                (43, _) => (20, 1),
                (45, _) => (21, 1),
                (42, _) => (22, 1),
                (47, _) => (23, 1),
                (37, _) => (24, 1),
                (61, _) => (25, 1),
                (33, _) => (27, 1),
                (60, _) => (29, 1),
                (62, _) => (31, 1),
                _ => return encode_result(3, index as i32, &completed, None),
            };
            index += width;
            kind
        };

        if completed.len() == TOKEN_CAPACITY - 1 {
            return encode_result(4, start as i32, &completed, None);
        }
        completed.push((kind, start, index - start));
    }

    encode_result(0, -1, &completed, Some(length))
}

fn fixture_matrix() -> Vec<Fixture> {
    let twenty_three = std::iter::repeat_n("a", 23).collect::<Vec<_>>().join(" ");
    let twenty_four = std::iter::repeat_n("a", 24).collect::<Vec<_>>().join(" ");
    vec![
        Fixture::text("empty", ""),
        Fixture::text("whitespace", " \t\r\n"),
        Fixture::text("keywords", "fn let mut return if else while"),
        Fixture::text("identifier-and-decimal", "_item a9 00123"),
        Fixture::text("delimiters", "(){}[],:;."),
        Fixture::text("operators", "+ - * / % = == ! != < <= > >= && || -> =>"),
        Fixture::text("comments", "a/* block */+b// tail"),
        Fixture::text("line-comment-newline", "a// x\nreturn"),
        Fixture {
            name: "ignored-inactive-tail",
            source: vec![102, 110, 200],
            source_len: 2,
        },
        Fixture {
            name: "negative-length",
            source: Vec::new(),
            source_len: -1,
        },
        Fixture {
            name: "length-above-capacity",
            source: Vec::new(),
            source_len: 65,
        },
        Fixture {
            name: "non-ascii-high",
            source: vec![102, 110, 32, 128],
            source_len: 4,
        },
        Fixture {
            name: "non-ascii-negative",
            source: vec![97, 32, -1],
            source_len: 3,
        },
        Fixture::text("unsupported-ascii", "let @"),
        Fixture::text("unterminated-block-comment", "a /* no end"),
        Fixture {
            name: "non-ascii-in-comment",
            source: vec![47, 42, 32, 200, 32, 42, 47],
            source_len: 7,
        },
        Fixture::text("twenty-three-tokens", &twenty_three),
        Fixture::text("twenty-fourth-token", &twenty_four),
    ]
}

fn fixture<'a>(fixtures: &'a [Fixture], name: &str) -> &'a Fixture {
    fixtures
        .iter()
        .find(|fixture| fixture.name == name)
        .unwrap_or_else(|| panic!("missing fixture {name}"))
}

fn array_literal(values: &[i32]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(i32::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn generated_oracle_program(kernel_prefix: &str, fixtures: &[Fixture]) -> String {
    let mut program = String::from(kernel_prefix.trim_end());
    program.push_str("\n\nfn main() -> int {\n");
    for (index, fixture) in fixtures.iter().enumerate() {
        let lanes = fixture.lanes();
        let expected = reference_tokenize_ascii_64(&lanes, fixture.source_len);
        writeln!(
            program,
            "    let source_{index}: [int; 64] = {};",
            array_literal(&lanes)
        )
        .expect("write generated source fixture");
        writeln!(
            program,
            "    let expected_{index}: [int; 75] = {};",
            array_literal(&expected)
        )
        .expect("write generated expected fixture");
        writeln!(
            program,
            "    let actual_{index}: [int; 75] = tokenize_ascii_64(source_{index}, {});",
            fixture.source_len
        )
        .expect("write generated tokenizer call");
        writeln!(
            program,
            "    if outputs_equal(actual_{index}, expected_{index}) == 0 {{ return {}; }}",
            10 + index
        )
        .expect("write generated vector comparison");
    }
    program.push_str("    return 91;\n}\n");
    program
}

fn run_cli(workspace: &TestWorkspace, arguments: &[String]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aero"))
        .current_dir(&workspace.root)
        .args(arguments)
        .output()
        .expect("run Aero CAP-031 public route")
}

fn visible_output(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
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
fn independent_reference_scanner_freezes_the_complete_bounded_contract() {
    let fixtures = fixture_matrix();
    let empty = reference_tokenize_ascii_64(&fixture(&fixtures, "empty").lanes(), 0);
    assert_eq!(&empty[..6], &[0, 1, -1, 0, 0, 0]);

    let keywords = fixture(&fixtures, "keywords");
    let keyword_output = reference_tokenize_ascii_64(&keywords.lanes(), keywords.source_len);
    let keyword_kinds = (0..keyword_output[1] as usize)
        .map(|index| keyword_output[3 + index * 3])
        .collect::<Vec<_>>();
    assert_eq!(keyword_kinds, vec![3, 4, 5, 6, 7, 8, 9, 0]);

    let mut observed_kinds = BTreeSet::new();
    for case in &fixtures {
        let output = reference_tokenize_ascii_64(&case.lanes(), case.source_len);
        if output[0] == 0 {
            for index in 0..output[1] as usize {
                observed_kinds.insert(output[3 + index * 3]);
            }
        }
    }
    assert_eq!(observed_kinds, (0..=36).collect());

    for (name, status, count, offset) in [
        ("negative-length", 1, 0, -1),
        ("length-above-capacity", 1, 0, -1),
        ("non-ascii-high", 2, 1, 3),
        ("non-ascii-negative", 2, 1, 2),
        ("unsupported-ascii", 3, 1, 4),
        ("unterminated-block-comment", 3, 1, 2),
        ("non-ascii-in-comment", 2, 0, 3),
        ("twenty-fourth-token", 4, 23, 46),
    ] {
        let case = fixture(&fixtures, name);
        let output = reference_tokenize_ascii_64(&case.lanes(), case.source_len);
        assert_eq!(&output[..3], &[status, count, offset], "{name}");
    }

    let boundary = fixture(&fixtures, "twenty-three-tokens");
    let boundary_output = reference_tokenize_ascii_64(&boundary.lanes(), boundary.source_len);
    assert_eq!(&boundary_output[..3], &[0, 24, -1]);
    assert_eq!(&boundary_output[72..75], &[0, 45, 0]);

    let ignored_tail = fixture(&fixtures, "ignored-inactive-tail");
    let ignored_output =
        reference_tokenize_ascii_64(&ignored_tail.lanes(), ignored_tail.source_len);
    assert_eq!(&ignored_output[..9], &[0, 2, -1, 3, 0, 2, 0, 2, 0]);
}

#[test]
fn tracked_kernel_matches_independent_vectors_and_cross_platform_gate() {
    let product_path = repository_path(PRODUCT_RELATIVE_PATH);
    assert!(product_path.is_file(), "{INTENTIONAL_PRODUCT_RED}");
    let product = fs::read_to_string(&product_path).expect("read tracked CAP-031 product");
    let (kernel_prefix, tracked_main) = product
        .split_once(SELF_TEST_MARKER)
        .expect("tracked product must retain the one kernel/self-test boundary");

    assert_eq!(product.matches(SELF_TEST_MARKER).count(), 1);
    assert_eq!(kernel_prefix.matches("fn tokenize_ascii_64(").count(), 1);
    assert!(
        kernel_prefix
            .contains("fn tokenize_ascii_64(source: [int; 64], source_len: int) -> [i32; 75]")
    );
    assert!(kernel_prefix.contains("fn outputs_equal(left: [int; 75], right: [int; 75]) -> int"));
    assert!(tracked_main.contains("fn main() -> int"));
    assert!(tracked_main.matches("tokenize_ascii_64(").count() >= 5);
    for forbidden in ["print", "String", "Vec", "mod ", "use "] {
        assert!(
            !product.contains(forbidden),
            "tracked bounded product contains forbidden `{forbidden}`"
        );
    }

    let workflow = fs::read_to_string(repository_path(WORKFLOW_RELATIVE_PATH))
        .expect("read Rust system workflow");
    let linux = workflow_step(
        &workflow,
        "Test exact i32 fixed-array CPU profile at O0 and O2",
    );
    let windows = workflow_step(
        &workflow,
        "Test exact i32 fixed-array CPU profile on Windows at O0 and O2",
    );
    for anchor in [
        "tokenizer:bootstrap_ascii_tokenizer.aero:91:yes",
        "if [ \"${name}\" = tokenizer ]; then",
        "exact_i32_array_tokenizer.linux.second.ll",
        "cmp -s \"${llvm}\" \"${tokenizer_second_llvm}\"",
        "llvm-as-22 \"${llvm}\"",
        "define [75 x i32] @tokenize_ascii_64([64 x i32] %aero.arg.source, i32 %aero.arg.source_len)",
        "tokenizer_guard_count",
    ] {
        assert!(
            linux.contains(anchor),
            "Linux CAP-031 gate lacks `{anchor}`"
        );
    }
    for anchor in [
        "Name = \"tokenizer\"; File = \"bootstrap_ascii_tokenizer.aero\"; Expected = 91; Dynamic = $true",
        "if ($specimen.Name -ceq \"tokenizer\") {",
        "exact_i32_array_tokenizer.windows.second.ll",
        "SequenceEqual([IO.File]::ReadAllBytes($llvm), [IO.File]::ReadAllBytes($tokenizerSecondLlvm))",
        "llvm-as.exe\" $llvm",
        "define [75 x i32] @tokenize_ascii_64([64 x i32] %aero.arg.source, i32 %aero.arg.source_len)",
        "$tokenizerGuardMatches",
    ] {
        assert!(
            windows.contains(anchor),
            "Windows CAP-031 gate lacks `{anchor}`"
        );
    }

    check_program(&product, exact_options()).expect("tracked CAP-031 product should check");
    let product_llvm =
        compile_program(&product, exact_options()).expect("tracked CAP-031 product should compile");
    assert!(product_llvm.contains(
        "define [75 x i32] @tokenize_ascii_64([64 x i32] %aero.arg.source, i32 %aero.arg.source_len)"
    ));

    let fixtures = fixture_matrix();
    let generated = generated_oracle_program(kernel_prefix, &fixtures);
    check_program(&generated, exact_options()).expect("generated oracle suite should check");
    let first = compile_program(&generated, exact_options())
        .expect("generated oracle suite should compile first time");
    let second = compile_program(&generated, exact_options())
        .expect("generated oracle suite should compile deterministically");
    assert_eq!(first, second, "CAP-031 generated LLVM is nondeterministic");
    for forbidden in [
        "double", "fptosi", "sitofp", " nsw ", " nuw ", " x i8]", "@malloc", "@free",
    ] {
        assert!(!first.contains(forbidden), "LLVM leaked `{forbidden}`");
    }

    let workspace = TestWorkspace::new("oracle");
    let generated_path = workspace.path("generated_oracle.aero");
    let first_llvm_path = workspace.path("generated.first.ll");
    let second_llvm_path = workspace.path("generated.second.ll");
    fs::write(&generated_path, generated).expect("write generated CAP-031 oracle suite");

    let check = run_cli(
        &workspace,
        &[
            "check".into(),
            generated_path.display().to_string(),
            "--language-profile".into(),
            "exact-i32-array-v0".into(),
        ],
    );
    assert!(check.status.success(), "{}", visible_output(&check));

    for llvm_path in [&first_llvm_path, &second_llvm_path] {
        let build = run_cli(
            &workspace,
            &[
                "build".into(),
                generated_path.display().to_string(),
                "-o".into(),
                llvm_path.display().to_string(),
                "--require-llvm-verifier".into(),
                "--language-profile".into(),
                "exact-i32-array-v0".into(),
            ],
        );
        assert!(build.status.success(), "{}", visible_output(&build));
    }
    assert_eq!(
        fs::read(&first_llvm_path).expect("read first verified LLVM"),
        fs::read(&second_llvm_path).expect("read second verified LLVM"),
        "public verified CAP-031 builds differ"
    );

    let generated_run = run_cli(
        &workspace,
        &[
            "run".into(),
            generated_path.display().to_string(),
            "--language-profile".into(),
            "exact-i32-array-v0".into(),
        ],
    );
    assert_eq!(
        generated_run.status.code(),
        Some(91),
        "{}",
        visible_output(&generated_run)
    );
    assert_eq!(
        visible_output(&generated_run)
            .matches("Exit code: 91")
            .count(),
        1
    );

    let tracked_run = run_cli(
        &workspace,
        &[
            "run".into(),
            product_path.display().to_string(),
            "--language-profile".into(),
            "exact-i32-array-v0".into(),
        ],
    );
    assert_eq!(
        tracked_run.status.code(),
        Some(91),
        "{}",
        visible_output(&tracked_run)
    );
    assert_eq!(
        visible_output(&tracked_run)
            .matches("Exit code: 91")
            .count(),
        1
    );
}

#[test]
fn task_generated_paths_are_routed_by_the_process_temp_contract() {
    let temp = std::env::temp_dir();
    assert!(Path::new(&temp).is_absolute());
}
