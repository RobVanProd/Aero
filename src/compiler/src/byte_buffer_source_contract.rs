use crate::ast::{Expression, Statement, Type, VariantDeclKind};

pub(crate) const BYTE_BUFFER_SOURCE_TYPE: &str = "ByteBuffer";
pub(crate) const BYTES_NEW: &str = "bytes_new";
pub(crate) const BYTES_PUSH: &str = "bytes_push";
pub(crate) const BYTES_LENGTH: &str = "bytes_len";
pub(crate) const BYTES_CAPACITY: &str = "bytes_capacity";
pub(crate) const BYTES_GET: &str = "bytes_get";

pub(crate) const BYTE_BUFFER_INTRINSICS: [&str; 5] = [
    BYTES_NEW,
    BYTES_PUSH,
    BYTES_LENGTH,
    BYTES_CAPACITY,
    BYTES_GET,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ByteBufferIntrinsic {
    New,
    Push,
    Length,
    Capacity,
    Get,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ByteBufferIntrinsicCall<'a> {
    pub(crate) intrinsic: ByteBufferIntrinsic,
    pub(crate) owner: Option<&'a str>,
    pub(crate) scalar: Option<&'a Expression>,
}

pub(crate) fn is_byte_buffer_annotation(annotation: &Type) -> bool {
    matches!(annotation, Type::Named(name) if name == BYTE_BUFFER_SOURCE_TYPE)
}

pub(crate) fn contains_byte_buffer_annotation(annotation: &Type) -> bool {
    match annotation {
        Type::Named(name) => name == BYTE_BUFFER_SOURCE_TYPE,
        Type::Array(element, _) | Type::Reference(element, _) => {
            contains_byte_buffer_annotation(element)
        }
        Type::Tuple(elements) | Type::Generic(_, elements) => {
            elements.iter().any(contains_byte_buffer_annotation)
        }
    }
}

pub(crate) fn is_reserved_byte_buffer_intrinsic(name: &str) -> bool {
    BYTE_BUFFER_INTRINSICS.contains(&name)
}

pub(crate) fn is_reserved_byte_buffer_type(name: &str) -> bool {
    name == BYTE_BUFFER_SOURCE_TYPE
}

pub(crate) fn byte_buffer_type_declaration_diagnostic(statement: &Statement) -> Option<String> {
    match statement {
        Statement::StructDef { name, fields, .. } => {
            if is_reserved_byte_buffer_type(name) {
                Some(
                    "source type name `ByteBuffer` is reserved by exact-i32-byte-buffer-v0"
                        .to_string(),
                )
            } else if fields
                .iter()
                .any(|field| contains_byte_buffer_annotation(&field.field_type))
            {
                Some(format!(
                    "source struct `{name}` cannot contain a ByteBuffer field"
                ))
            } else {
                None
            }
        }
        Statement::EnumDef { name, variants, .. } => {
            if is_reserved_byte_buffer_type(name) {
                return Some(
                    "source type name `ByteBuffer` is reserved by exact-i32-byte-buffer-v0"
                        .to_string(),
                );
            }
            let contains = variants.iter().any(|variant| match &variant.kind {
                VariantDeclKind::Unit => false,
                VariantDeclKind::Tuple(fields) => {
                    fields.iter().any(contains_byte_buffer_annotation)
                }
                VariantDeclKind::Struct(fields) => fields
                    .iter()
                    .any(|field| contains_byte_buffer_annotation(&field.field_type)),
            });
            contains.then(|| format!("source enum `{name}` cannot contain a ByteBuffer payload"))
        }
        _ => None,
    }
}

fn diagnostic(name: &str, expected: &str) -> String {
    format!("byte-buffer intrinsic `{name}` requires exactly {expected}")
}

pub(crate) fn result_context_diagnostic(name: &str) -> String {
    format!("byte-buffer intrinsic `{name}` requires an explicit `Result<int, int>` context")
}

fn direct_owner_borrow<'a>(
    name: &str,
    argument: &'a Expression,
    mutable: bool,
) -> Result<&'a str, String> {
    let Expression::Borrow {
        expr,
        mutable: actual_mutable,
    } = argument
    else {
        return Err(diagnostic(
            name,
            if mutable {
                "an immediate `&mut ByteBuffer` identifier"
            } else {
                "an immediate `&ByteBuffer` identifier"
            },
        ));
    };
    let Expression::Identifier(owner) = expr.as_ref() else {
        return Err(diagnostic(
            name,
            if mutable {
                "an immediate `&mut ByteBuffer` identifier"
            } else {
                "an immediate `&ByteBuffer` identifier"
            },
        ));
    };
    if *actual_mutable != mutable {
        return Err(diagnostic(
            name,
            if mutable {
                "an immediate `&mut ByteBuffer` identifier"
            } else {
                "an immediate `&ByteBuffer` identifier"
            },
        ));
    }
    Ok(owner)
}

/// Classify only the closed R1C call syntax. Type, ownership, scope, and
/// lifecycle checks remain independently owned by semantics and checked IR.
pub(crate) fn classify_byte_buffer_intrinsic_call<'a>(
    name: &str,
    arguments: &'a [Expression],
) -> Result<Option<ByteBufferIntrinsicCall<'a>>, String> {
    let call = match name {
        BYTES_NEW => {
            if !arguments.is_empty() {
                return Err(diagnostic(BYTES_NEW, "zero arguments"));
            }
            ByteBufferIntrinsicCall {
                intrinsic: ByteBufferIntrinsic::New,
                owner: None,
                scalar: None,
            }
        }
        BYTES_PUSH => {
            let [owner, byte] = arguments else {
                return Err(diagnostic(BYTES_PUSH, "`&mut ByteBuffer, int` arguments"));
            };
            ByteBufferIntrinsicCall {
                intrinsic: ByteBufferIntrinsic::Push,
                owner: Some(direct_owner_borrow(BYTES_PUSH, owner, true)?),
                scalar: Some(byte),
            }
        }
        BYTES_LENGTH => {
            let [owner] = arguments else {
                return Err(diagnostic(BYTES_LENGTH, "one `&ByteBuffer` argument"));
            };
            ByteBufferIntrinsicCall {
                intrinsic: ByteBufferIntrinsic::Length,
                owner: Some(direct_owner_borrow(BYTES_LENGTH, owner, false)?),
                scalar: None,
            }
        }
        BYTES_CAPACITY => {
            let [owner] = arguments else {
                return Err(diagnostic(BYTES_CAPACITY, "one `&ByteBuffer` argument"));
            };
            ByteBufferIntrinsicCall {
                intrinsic: ByteBufferIntrinsic::Capacity,
                owner: Some(direct_owner_borrow(BYTES_CAPACITY, owner, false)?),
                scalar: None,
            }
        }
        BYTES_GET => {
            let [owner, index] = arguments else {
                return Err(diagnostic(BYTES_GET, "`&ByteBuffer, int` arguments"));
            };
            ByteBufferIntrinsicCall {
                intrinsic: ByteBufferIntrinsic::Get,
                owner: Some(direct_owner_borrow(BYTES_GET, owner, false)?),
                scalar: Some(index),
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(call))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{CheckedIr, Inst};
    use crate::ir_generator::IrGenerator;
    use crate::language_profile::{LanguageProfile, validate_resolved_language_profile};
    use crate::lexer::try_tokenize_with_locations;
    use crate::parser::parse_with_locations;
    use crate::semantic_analyzer::SemanticAnalyzer;

    fn identifier(name: &str) -> Expression {
        Expression::Identifier(name.to_string())
    }

    fn borrow(name: &str, mutable: bool) -> Expression {
        Expression::Borrow {
            expr: Box::new(identifier(name)),
            mutable,
        }
    }

    fn analyzed(
        source: &str,
    ) -> (
        Vec<crate::ast::AstNode>,
        crate::resolved_profile_shape::ResolvedProfileProgram,
    ) {
        let tokens = try_tokenize_with_locations(source, None).expect("R1C source lexes");
        let ast = parse_with_locations(tokens).expect("R1C source parses");
        let (_, ast, resolved) = SemanticAnalyzer::new_with_byte_buffer_source()
            .analyze_with_resolved_profile(ast)
            .expect("R1C source passes semantic ownership");
        validate_resolved_language_profile(&resolved, LanguageProfile::ExactI32ByteBufferV0)
            .expect("R1C source passes resolved profile admission");
        (ast, resolved)
    }

    fn checked(source: &str) -> CheckedIr {
        let (ast, _) = analyzed(source);
        IrGenerator::new_with_byte_buffer_source()
            .try_generate_ir(ast)
            .expect("R1C source reaches verified checked IR")
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

    #[test]
    fn exact_intrinsic_syntax_is_closed() {
        let push_arguments = [borrow("bytes", true), Expression::IntegerLiteral(7)];
        let push = classify_byte_buffer_intrinsic_call(BYTES_PUSH, &push_arguments)
            .expect("exact push syntax")
            .expect("reserved call");
        assert_eq!(push.intrinsic, ByteBufferIntrinsic::Push);
        assert_eq!(push.owner, Some("bytes"));
        assert!(matches!(push.scalar, Some(Expression::IntegerLiteral(7))));

        assert!(
            classify_byte_buffer_intrinsic_call(BYTES_LENGTH, &[borrow("bytes", false)])
                .expect("exact length syntax")
                .is_some()
        );
        assert!(
            classify_byte_buffer_intrinsic_call("ordinary", &[])
                .expect("ordinary call")
                .is_none()
        );
        assert!(is_reserved_byte_buffer_type("ByteBuffer"));
        assert!(!is_reserved_byte_buffer_type("ByteBuffer2"));
        assert!(
            classify_byte_buffer_intrinsic_call(BYTES_PUSH, &[borrow("bytes", false)]).is_err()
        );
        assert!(
            classify_byte_buffer_intrinsic_call(BYTES_GET, &[identifier("bytes"), identifier("i")])
                .is_err()
        );
    }

    #[test]
    fn source_mode_lowers_move_loans_results_and_reverse_cleanup_to_r1b_ir() {
        let checked = checked(
            r#"
fn main() -> int {
    let mut first: ByteBuffer = bytes_new();
    let pushed: Result<int, int> = bytes_push(&mut first, 91);
    let second: ByteBuffer = first;
    let found: Result<int, int> = bytes_get(&second, 0);
    if bytes_len(&second) == 1 && bytes_capacity(&second) == 8 {
        return 91;
    }
    return 1;
}
"#,
        );
        let body = function_body(&checked, "main");
        assert_eq!(
            body.iter()
                .filter(|instruction| matches!(instruction, Inst::CheckedByteBufferNew { .. }))
                .count(),
            1
        );
        assert_eq!(
            body.iter()
                .filter(|instruction| matches!(instruction, Inst::CheckedByteBufferMove { .. }))
                .count(),
            1
        );
        assert_eq!(
            body.iter()
                .filter(|instruction| matches!(instruction, Inst::CheckedByteBufferPush { .. }))
                .count(),
            1
        );
        assert_eq!(
            body.iter()
                .filter(|instruction| matches!(instruction, Inst::CheckedByteBufferGet { .. }))
                .count(),
            1
        );
        assert_eq!(
            body.iter()
                .filter(|instruction| {
                    matches!(instruction, Inst::CheckedByteBufferImmutableBorrow { .. })
                })
                .count(),
            3
        );
        assert_eq!(
            body.iter()
                .filter(|instruction| {
                    matches!(instruction, Inst::CheckedByteBufferMutableBorrow { .. })
                })
                .count(),
            1
        );
        assert!(body.iter().any(|instruction| {
            matches!(instruction, Inst::CheckedOwnedPlaceAssignment { .. })
        }));
        assert!(body.windows(2).all(|window| {
            !matches!(window[1], Inst::Return(_))
                || matches!(window[0], Inst::CheckedByteBufferDrop { .. })
        }));
        let (moved_from, moved_to) = body
            .iter()
            .find_map(|instruction| match instruction {
                Inst::CheckedByteBufferMove { result, source, .. } => {
                    Some((source.clone(), result.clone()))
                }
                _ => None,
            })
            .expect("fixture retains one owner move");
        for owner in body.iter().filter_map(|instruction| match instruction {
            Inst::CheckedByteBufferDrop { owner } => Some(owner),
            _ => None,
        }) {
            assert_eq!(owner, &moved_to, "cleanup must follow the moved identity");
            assert_ne!(owner, &moved_from, "cleanup must not drop the moved source");
        }
    }

    #[test]
    fn source_mode_closes_owners_on_early_and_fallthrough_exits() {
        let checked = checked(
            r#"
fn early(flag: bool) -> int {
    let bytes: ByteBuffer = bytes_new();
    if flag {
        return 91;
    }
    return 1;
}

fn tail() {
    let bytes: ByteBuffer = bytes_new();
}

fn main() -> int {
    tail();
    return early(1 < 2);
}
"#,
        );
        let early = function_body(&checked, "early");
        assert_eq!(
            early
                .iter()
                .filter(|instruction| matches!(instruction, Inst::CheckedByteBufferDrop { .. }))
                .count(),
            2
        );
        assert_eq!(
            early
                .iter()
                .filter(|instruction| matches!(instruction, Inst::Return(_)))
                .count(),
            2
        );
        let tail = function_body(&checked, "tail");
        assert!(matches!(
            tail.get(tail.len().saturating_sub(2)),
            Some(Inst::CheckedByteBufferDrop { .. })
        ));
        assert!(matches!(tail.last(), Some(Inst::Return(_))));
    }

    #[test]
    fn source_mode_drops_multiple_live_owners_in_reverse_declaration_order() {
        let checked = checked(
            r#"
fn main() -> int {
    let first: ByteBuffer = bytes_new();
    let second: ByteBuffer = bytes_new();
    return 91;
}
"#,
        );
        let body = function_body(&checked, "main");
        let created = body
            .iter()
            .filter_map(|instruction| match instruction {
                Inst::CheckedByteBufferNew { result, .. } => Some(result.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(created.len(), 2);
        let dropped = body
            .iter()
            .filter_map(|instruction| match instruction {
                Inst::CheckedByteBufferDrop { owner } => Some(owner.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            dropped,
            vec![created[1].clone(), created[0].clone()],
            "cleanup must use reverse declaration order"
        );
        assert!(matches!(body.last(), Some(Inst::Return(_))));
    }

    #[test]
    fn nested_integer_buffer_call_finishes_before_the_outer_mutable_loan() {
        let checked = checked(
            r#"
fn main() -> int {
    let mut bytes: ByteBuffer = bytes_new();
    let first: Result<int, int> = bytes_push(&mut bytes, 91);
    let second: Result<int, int> = bytes_push(&mut bytes, bytes_len(&bytes));
    return bytes_len(&bytes);
}
"#,
        );
        let body = function_body(&checked, "main");
        let second_push = body
            .iter()
            .enumerate()
            .filter_map(|(index, instruction)| {
                matches!(instruction, Inst::CheckedByteBufferPush { .. }).then_some(index)
            })
            .nth(1)
            .expect("fixture retains its second push");
        let nested_length = body[..second_push]
            .iter()
            .rposition(|instruction| matches!(instruction, Inst::CheckedByteBufferLength { .. }))
            .expect("second push scalar retains nested length");
        let nested_end = body[..second_push]
            .iter()
            .rposition(|instruction| {
                matches!(
                    instruction,
                    Inst::CheckedByteBufferImmutableBorrowEnd { .. }
                )
            })
            .expect("nested length ends its immutable loan");
        let outer_start = body[..second_push]
            .iter()
            .rposition(|instruction| {
                matches!(instruction, Inst::CheckedByteBufferMutableBorrow { .. })
            })
            .expect("second push opens its mutable loan");
        assert!(
            nested_length < nested_end && nested_end < outer_start && outer_start < second_push
        );
    }

    #[test]
    fn mode_off_direct_apis_cannot_reuse_source_mode_semantic_state() {
        let source = "fn main() -> int { let bytes: ByteBuffer = bytes_new(); return 91; }";
        let tokens = try_tokenize_with_locations(source, None).expect("source lexes");
        let raw = parse_with_locations(tokens).expect("source parses");
        assert!(
            SemanticAnalyzer::new().analyze(raw.clone()).is_err(),
            "public semantic API must stay mode-off"
        );
        let (analyzed, _) = analyzed(source);
        let error = IrGenerator::new()
            .try_generate_ir(analyzed)
            .expect_err("public checked-IR API must stay mode-off")
            .to_string();
        assert!(
            error.contains("bytes_new") || error.contains("ByteBuffer"),
            "mode-off checked admission reached the wrong boundary: {error}"
        );
    }

    #[test]
    fn checked_admission_independently_rejects_resource_escape_and_corruption() {
        let cases = [
            (
                "fn main() -> int { if 1 < 2 { let bytes: ByteBuffer = bytes_new(); } return 0; }",
                "outside control-flow topology",
            ),
            (
                "fn main() -> int { let first: ByteBuffer = bytes_new(); let second: ByteBuffer = first; return bytes_len(&first); }",
                "use of moved ByteBuffer owner `first` in checked IR",
            ),
            (
                "fn main() -> int { let bytes: ByteBuffer = bytes_new(); let pushed: Result<int, int> = bytes_push(&mut bytes, 1); return 0; }",
                "requires mutable owner `bytes`",
            ),
            (
                "fn main() -> int { let bytes: ByteBuffer = bytes_new(); return bytes_len(bytes); }",
                "requires exactly an immediate `&ByteBuffer` identifier",
            ),
            (
                "fn main() -> int { let mut bytes: ByteBuffer = bytes_new(); bytes_push(&mut bytes, 1); return 0; }",
                "requires an explicit `Result<int, int>` context",
            ),
            (
                "fn consume(bytes: ByteBuffer) -> int { return 0; } fn main() -> int { return 0; }",
                "cannot transport ByteBuffer in a parameter or result",
            ),
            (
                "struct ByteBuffer { value: int } fn main() -> int { return 0; }",
                "source type name `ByteBuffer` is reserved",
            ),
            (
                "struct Holder { bytes: ByteBuffer } fn main() -> int { return 0; }",
                "source struct `Holder` cannot contain a ByteBuffer field",
            ),
            (
                "fn main() -> int { let mut bytes: ByteBuffer = bytes_new(); bytes = bytes_new(); return 0; }",
                "may only be initialized or moved by direct binding",
            ),
        ];

        for (source, expected) in cases {
            let tokens = try_tokenize_with_locations(source, None).expect("invalid case lexes");
            let ast = parse_with_locations(tokens).expect("invalid case parses");
            let first = IrGenerator::new_with_byte_buffer_source()
                .try_generate_ir(ast.clone())
                .expect_err("checked admission must reject forged source AST")
                .to_string();
            let second = IrGenerator::new_with_byte_buffer_source()
                .try_generate_ir(ast)
                .expect_err("repeated checked admission must reject")
                .to_string();
            assert_eq!(first, second, "checked rejection was nondeterministic");
            assert!(
                first.contains(expected),
                "checked admission stopped at the wrong boundary: {first}"
            );
        }
    }
}
