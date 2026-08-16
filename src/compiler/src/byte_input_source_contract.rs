use crate::ast::{Expression, Type};
use crate::builtin_carrier_contract::private_result_int_int_name;

pub(crate) const STDIN_READ_BYTE: &str = "stdin_read_byte";

pub(crate) fn is_reserved_byte_input_intrinsic(name: &str) -> bool {
    name == STDIN_READ_BYTE
}

/// Classify only the closed R2 source call. Type, source-context, and checked-IR
/// admission remain independently owned by semantics and IR generation.
pub(crate) fn classify_byte_input_intrinsic_call(
    name: &str,
    arguments: &[Expression],
) -> Result<bool, String> {
    if !is_reserved_byte_input_intrinsic(name) {
        return Ok(false);
    }
    if !arguments.is_empty() {
        return Err(format!(
            "byte-input intrinsic `{STDIN_READ_BYTE}` requires exactly zero arguments"
        ));
    }
    Ok(true)
}

pub(crate) fn result_context_diagnostic() -> String {
    format!(
        "byte-input intrinsic `{STDIN_READ_BYTE}` requires an explicit `Result<int, int>` context"
    )
}

pub(crate) fn is_direct_byte_input_result_initializer(
    expression: &Expression,
    annotation: Option<&Type>,
) -> bool {
    matches!(
        expression,
        Expression::FunctionCall { name, arguments }
            if is_reserved_byte_input_intrinsic(name)
                && arguments.is_empty()
                && matches!(annotation, Some(Type::Named(result)) if result == &private_result_int_int_name())
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code_generator::try_generate_code_with_profile;
    use crate::ir::{CheckedIr, Inst, LogicalType, RawIr, ResultId, Value};
    use crate::ir_generator::IrGenerator;
    use crate::ir_verifier::{IrVerificationErrorKind, verify_ir};
    use crate::language_profile::{LanguageProfile, validate_resolved_language_profile};
    use crate::lexer::try_tokenize_with_locations;
    use crate::parser::parse_with_locations;
    use crate::semantic_analyzer::SemanticAnalyzer;

    const SOURCE: &str = r#"
fn main() -> int {
    let read: Result<int, int> = stdin_read_byte();
    return match read {
        Ok(value) => value,
        Err(code) => 0 - code,
    };
}
"#;

    fn parsed() -> Vec<crate::ast::AstNode> {
        let tokens = try_tokenize_with_locations(SOURCE, None).expect("R2 source lexes");
        parse_with_locations(tokens).expect("R2 source parses")
    }

    fn function_body<'a>(checked: &'a CheckedIr, name: &str) -> &'a [Inst] {
        checked
            .raw()
            .values()
            .flat_map(|function| function.body.iter())
            .find_map(|instruction| match instruction {
                Inst::FunctionDef {
                    name: candidate,
                    body,
                    ..
                }
                | Inst::CheckedFunctionDef {
                    name: candidate,
                    body,
                    ..
                } if candidate == name => Some(body.as_slice()),
                _ => None,
            })
            .unwrap_or_else(|| panic!("checked IR omitted function `{name}`"))
    }

    fn function_body_mut<'a>(raw: &'a mut RawIr, name: &str) -> &'a mut Vec<Inst> {
        raw.values_mut()
            .flat_map(|function| function.body.iter_mut())
            .find_map(|instruction| match instruction {
                Inst::FunctionDef {
                    name: candidate,
                    body,
                    ..
                }
                | Inst::CheckedFunctionDef {
                    name: candidate,
                    body,
                    ..
                } if candidate == name => Some(body),
                _ => None,
            })
            .unwrap_or_else(|| panic!("raw IR omitted function `{name}`"))
    }

    #[test]
    fn exact_stdin_read_call_is_closed() {
        assert!(
            classify_byte_input_intrinsic_call(STDIN_READ_BYTE, &[])
                .expect("exact zero-argument call")
        );
        assert!(
            classify_byte_input_intrinsic_call(STDIN_READ_BYTE, &[Expression::IntegerLiteral(1)])
                .is_err()
        );
        assert!(!classify_byte_input_intrinsic_call("ordinary", &[]).expect("ordinary call"));
    }

    #[test]
    fn source_mode_emits_one_typed_checked_read_and_mode_off_rejects_it() {
        let ordinary_error = SemanticAnalyzer::new()
            .analyze(parsed())
            .expect_err("ordinary semantics must not acquire R2 input");
        assert!(
            ordinary_error.contains("Function `stdin_read_byte` is not defined"),
            "ordinary semantics stopped at the wrong boundary: {ordinary_error}"
        );

        let (_, ast, resolved) = SemanticAnalyzer::new_with_byte_input_source()
            .analyze_with_resolved_profile(parsed())
            .expect("R2 source passes semantic analysis");
        validate_resolved_language_profile(&resolved, LanguageProfile::ExactI32ByteInputV0)
            .expect("R2 source passes resolved profile admission");
        assert!(
            IrGenerator::new().try_generate_ir(ast.clone()).is_err(),
            "ordinary IR generation must not acquire R2 input"
        );
        let checked = IrGenerator::new_with_byte_input_source()
            .try_generate_ir(ast)
            .expect("R2 source reaches verified IR");
        let reads = function_body(&checked, "main")
            .iter()
            .filter_map(|instruction| match instruction {
                Inst::CheckedStdinReadByte { result } => Some(result),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(reads.len(), 1, "R2 source must emit one checked read site");
        let Value::Reg(read_id) = reads[0] else {
            panic!("R2 checked read did not define a result identifier")
        };
        assert_eq!(
            checked.metadata().functions["main"].results[&ResultId(*read_id)],
            LogicalType::Int,
            "verifier metadata did not publish the R2 raw status as logical Int"
        );

        let backend_error =
            try_generate_code_with_profile(checked, LanguageProfile::ExactI32ByteBufferV0)
                .expect_err("the earlier exact profile must reject R2 checked IR")
                .to_string();
        assert!(
            backend_error.contains("checked stdin reads require exact-i32-byte-input-v0"),
            "wrong-profile backend stopped at the wrong boundary: {backend_error}"
        );
    }

    #[test]
    fn verifier_rejects_duplicate_and_nonidentifier_read_results() {
        let (_, ast, _) = SemanticAnalyzer::new_with_byte_input_source()
            .analyze_with_resolved_profile(parsed())
            .expect("R2 corruption fixture passes semantics");
        let checked = IrGenerator::new_with_byte_input_source()
            .try_generate_ir(ast)
            .expect("R2 corruption fixture reaches checked IR");

        let mut duplicate = checked.raw().clone();
        let body = function_body_mut(&mut duplicate, "main");
        let index = body
            .iter()
            .position(|instruction| matches!(instruction, Inst::CheckedStdinReadByte { .. }))
            .expect("R2 corruption fixture omitted checked read");
        let repeated = body[index].clone();
        let Inst::CheckedStdinReadByte {
            result: Value::Reg(id),
        } = &repeated
        else {
            panic!("R2 corruption fixture read did not define a register")
        };
        let duplicate_id = ResultId(*id);
        body.insert(index + 1, repeated);
        assert_eq!(
            verify_ir(duplicate)
                .expect_err("duplicate R2 result definition must fail")
                .kind,
            IrVerificationErrorKind::DuplicateResultDefinition(duplicate_id)
        );

        let mut nonidentifier = checked.raw().clone();
        let read = function_body_mut(&mut nonidentifier, "main")
            .iter_mut()
            .find(|instruction| matches!(instruction, Inst::CheckedStdinReadByte { .. }))
            .expect("R2 corruption fixture omitted mutable checked read");
        let Inst::CheckedStdinReadByte { result } = read else {
            unreachable!()
        };
        *result = Value::ImmInt(0);
        assert_eq!(
            verify_ir(nonidentifier)
                .expect_err("nonidentifier R2 result must fail")
                .kind,
            IrVerificationErrorKind::ExpectedResultIdentifier("instruction")
        );
    }
}
