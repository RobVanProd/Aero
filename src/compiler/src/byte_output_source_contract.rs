use crate::ast::{Expression, Type};
use crate::builtin_carrier_contract::private_result_int_int_name;

pub(crate) const STDOUT_WRITE_BYTE: &str = "stdout_write_byte";

pub(crate) fn is_reserved_byte_output_intrinsic(name: &str) -> bool {
    name == STDOUT_WRITE_BYTE
}

/// Classify only the closed CAP-047 source call. Type, source-context, and
/// checked-IR admission remain independently owned by semantics and IR
/// generation.
pub(crate) fn classify_byte_output_intrinsic_call(
    name: &str,
    arguments: &[Expression],
) -> Result<bool, String> {
    if !is_reserved_byte_output_intrinsic(name) {
        return Ok(false);
    }
    if arguments.len() != 1 {
        return Err(format!(
            "byte-output intrinsic `{STDOUT_WRITE_BYTE}` requires exactly one argument"
        ));
    }
    Ok(true)
}

pub(crate) fn result_context_diagnostic() -> String {
    format!(
        "byte-output intrinsic `{STDOUT_WRITE_BYTE}` requires an explicit `Result<int, int>` context"
    )
}

pub(crate) fn is_direct_byte_output_result_initializer(
    expression: &Expression,
    annotation: Option<&Type>,
) -> bool {
    matches!(
        expression,
        Expression::FunctionCall { name, arguments }
            if is_reserved_byte_output_intrinsic(name)
                && arguments.len() == 1
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
    let written: Result<int, int> = stdout_write_byte(65);
    return match written {
        Ok(status) => status,
        Err(code) => 0 - code,
    };
}
"#;

    fn parsed() -> Vec<crate::ast::AstNode> {
        let tokens = try_tokenize_with_locations(SOURCE, None).expect("CAP-047 source lexes");
        parse_with_locations(tokens).expect("CAP-047 source parses")
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
    fn exact_stdout_write_call_is_closed() {
        assert!(
            classify_byte_output_intrinsic_call(
                STDOUT_WRITE_BYTE,
                &[Expression::IntegerLiteral(65)]
            )
            .expect("exact one-argument call")
        );
        assert!(classify_byte_output_intrinsic_call(STDOUT_WRITE_BYTE, &[]).is_err());
        assert!(
            classify_byte_output_intrinsic_call(
                STDOUT_WRITE_BYTE,
                &[Expression::IntegerLiteral(1), Expression::IntegerLiteral(2)]
            )
            .is_err()
        );
        assert!(
            !classify_byte_output_intrinsic_call("ordinary", &[Expression::IntegerLiteral(65)])
                .expect("ordinary call")
        );
    }

    #[test]
    fn source_mode_emits_one_typed_checked_write_and_wrong_mode_rejects_it() {
        let ordinary_error = SemanticAnalyzer::new()
            .analyze(parsed())
            .expect_err("ordinary semantics must not acquire CAP-047 output");
        assert!(
            ordinary_error.contains("Function `stdout_write_byte` is not defined"),
            "ordinary semantics stopped at the wrong boundary: {ordinary_error}"
        );

        let (_, ast, resolved) = SemanticAnalyzer::new_with_byte_io_source()
            .analyze_with_resolved_profile(parsed())
            .expect("CAP-047 source passes semantic analysis");
        validate_resolved_language_profile(&resolved, LanguageProfile::ExactI32ByteIoV0)
            .expect("CAP-047 source passes resolved profile admission");
        assert!(
            IrGenerator::new_with_byte_input_source()
                .try_generate_ir(ast.clone())
                .is_err(),
            "the input-only generator must not acquire output"
        );
        let checked = IrGenerator::new_with_byte_io_source()
            .try_generate_ir(ast)
            .expect("CAP-047 source reaches verified IR");
        let writes = function_body(&checked, "main")
            .iter()
            .filter_map(|instruction| match instruction {
                Inst::CheckedStdoutWriteByte { result, value } => Some((result, value)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(writes.len(), 1, "source must emit one checked write site");
        assert_eq!(writes[0].1, &Value::ImmInt(65));
        let Value::Reg(write_id) = writes[0].0 else {
            panic!("checked write did not define a result identifier")
        };
        assert_eq!(
            checked.metadata().functions["main"].results[&ResultId(*write_id)],
            LogicalType::Int
        );

        let backend_error =
            try_generate_code_with_profile(checked, LanguageProfile::ExactI32ByteInputV0)
                .expect_err("input-only backend must reject output checked IR")
                .to_string();
        assert!(
            backend_error.contains("checked stdout byte writes require exact-i32-byte-io-v0"),
            "wrong-profile backend stopped at the wrong boundary: {backend_error}"
        );
    }

    #[test]
    fn verifier_rejects_corrupt_output_result_and_operand() {
        let (_, ast, _) = SemanticAnalyzer::new_with_byte_io_source()
            .analyze_with_resolved_profile(parsed())
            .expect("CAP-047 corruption fixture passes semantics");
        let checked = IrGenerator::new_with_byte_io_source()
            .try_generate_ir(ast)
            .expect("CAP-047 corruption fixture reaches checked IR");

        let mut duplicate = checked.raw().clone();
        let body = function_body_mut(&mut duplicate, "main");
        let index = body
            .iter()
            .position(|instruction| matches!(instruction, Inst::CheckedStdoutWriteByte { .. }))
            .expect("corruption fixture omitted checked write");
        let repeated = body[index].clone();
        let Inst::CheckedStdoutWriteByte {
            result: Value::Reg(id),
            ..
        } = &repeated
        else {
            panic!("corruption fixture write did not define a register")
        };
        let duplicate_id = ResultId(*id);
        body.insert(index + 1, repeated);
        assert_eq!(
            verify_ir(duplicate)
                .expect_err("duplicate output result definition must fail")
                .kind,
            IrVerificationErrorKind::DuplicateResultDefinition(duplicate_id)
        );

        let mut nonidentifier = checked.raw().clone();
        let write = function_body_mut(&mut nonidentifier, "main")
            .iter_mut()
            .find(|instruction| matches!(instruction, Inst::CheckedStdoutWriteByte { .. }))
            .expect("corruption fixture omitted mutable checked write");
        let Inst::CheckedStdoutWriteByte { result, .. } = write else {
            unreachable!()
        };
        *result = Value::ImmInt(0);
        assert_eq!(
            verify_ir(nonidentifier)
                .expect_err("nonidentifier output result must fail")
                .kind,
            IrVerificationErrorKind::ExpectedResultIdentifier("instruction")
        );

        let mut wrong_operand = checked.raw().clone();
        let write = function_body_mut(&mut wrong_operand, "main")
            .iter_mut()
            .find(|instruction| matches!(instruction, Inst::CheckedStdoutWriteByte { .. }))
            .expect("corruption fixture omitted operand-bearing write");
        let Inst::CheckedStdoutWriteByte { value, .. } = write else {
            unreachable!()
        };
        *value = Value::ImmFloat(65.0);
        assert!(
            verify_ir(wrong_operand).is_err(),
            "non-Int output operand must fail checked verification"
        );
    }
}
