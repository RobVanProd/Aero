use compiler::{CompilerOptions, compile_program};

const DEFAULT_SOURCE: &str = "let value = 1;";
const STRICT_LEX_FAILURE: &str = "let value = 1@;";
const EXPECTED_DEFAULT_LLVM: &str = concat!(
    "; ModuleID = \"aero_compiler\"\n",
    "source_filename = \"aero_compiler\"\n",
    "target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"\n",
    "target triple = \"x86_64-pc-linux-gnu\"\n",
    "\n",
    "declare i32 @printf(i8*, ...)\n",
    "\n",
    "define i32 @main() {\n",
    "entry:\n",
    "  %ptr0 = alloca double, align 8\n",
    "  store double 0x3FF0000000000000, double* %ptr0, align 8\n",
    "  ret i32 0\n",
    "}\n",
    "\n",
);
const EXPECTED_DEFAULT_PARSE_ERROR: &str =
    "Parse error: Error at 1:5: Expected variable name, found Assign";
const UNSUPPORTED_OPTIONS_ERROR: &str = "Unsupported CompilerOptions: only \
CompilerOptions::default() is supported; optimize, debug_info, and target behavior is not implemented";

#[test]
fn default_options_preserve_parent_llvm_and_parse_diagnostic_exactly() {
    let llvm = compile_program(DEFAULT_SOURCE, CompilerOptions::default())
        .expect("default parent source must compile");
    assert_eq!(llvm, EXPECTED_DEFAULT_LLVM);

    let parse_error = compile_program("let = ;", CompilerOptions::default())
        .expect_err("default malformed source must fail parsing");
    assert_eq!(parse_error, EXPECTED_DEFAULT_PARSE_ERROR);
}

#[test]
fn every_nondefault_option_fails_before_lexing_without_target_normalization() {
    let cases = [
        (
            "optimize valid",
            DEFAULT_SOURCE,
            CompilerOptions {
                optimize: true,
                ..CompilerOptions::default()
            },
        ),
        (
            "debug-info valid",
            DEFAULT_SOURCE,
            CompilerOptions {
                debug_info: true,
                ..CompilerOptions::default()
            },
        ),
        (
            "target valid",
            DEFAULT_SOURCE,
            CompilerOptions {
                target: "cpu".to_string(),
                ..CompilerOptions::default()
            },
        ),
        (
            "whitespace target valid",
            DEFAULT_SOURCE,
            CompilerOptions {
                target: "   ".to_string(),
                ..CompilerOptions::default()
            },
        ),
        (
            "combined valid",
            DEFAULT_SOURCE,
            CompilerOptions {
                optimize: true,
                debug_info: true,
                target: "x86_64-unknown-aero".to_string(),
            },
        ),
        (
            "optimize strict-lex failure",
            STRICT_LEX_FAILURE,
            CompilerOptions {
                optimize: true,
                ..CompilerOptions::default()
            },
        ),
        (
            "debug-info strict-lex failure",
            STRICT_LEX_FAILURE,
            CompilerOptions {
                debug_info: true,
                ..CompilerOptions::default()
            },
        ),
        (
            "target strict-lex failure",
            STRICT_LEX_FAILURE,
            CompilerOptions {
                target: "cpu".to_string(),
                ..CompilerOptions::default()
            },
        ),
        (
            "whitespace target strict-lex failure",
            STRICT_LEX_FAILURE,
            CompilerOptions {
                target: "   ".to_string(),
                ..CompilerOptions::default()
            },
        ),
        (
            "combined strict-lex failure",
            STRICT_LEX_FAILURE,
            CompilerOptions {
                optimize: true,
                debug_info: true,
                target: "x86_64-unknown-aero".to_string(),
            },
        ),
    ];

    let mut failures = Vec::new();
    for (label, source, options) in cases {
        match compile_program(source, options) {
            Err(error) if error == UNSUPPORTED_OPTIONS_ERROR => {}
            Err(error) => failures.push(format!(
                "{label}: expected `{UNSUPPORTED_OPTIONS_ERROR}`, got `{error}`"
            )),
            Ok(_) => failures.push(format!(
                "{label}: compilation succeeded instead of returning `{UNSUPPORTED_OPTIONS_ERROR}`"
            )),
        }
    }

    assert!(
        failures.is_empty(),
        "nondefault CompilerOptions contract violations:\n{}",
        failures.join("\n")
    );
}
