use crate::ir::{CheckedIr, FunctionSignature, LogicalType, PlaceId, ResultId};
use crate::resolved_profile_shape::{
    ResolvedProfileProgram, ResolvedProfileResolution, ResolvedProfileShapeId,
};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ResolvedProfileAuthenticationSubject {
    Nominal {
        normalized: String,
    },
    FunctionParameter {
        function: String,
        index: usize,
        name: String,
    },
    FunctionResult {
        function: String,
    },
    MetadataResult {
        function: String,
        result: ResultId,
    },
    MetadataPlace {
        function: String,
        place: PlaceId,
        name: Option<String>,
    },
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolvedProfileAuthenticationCoverage {
    Authenticated(ResolvedProfileShapeId),
    ExplicitUnavailable(ResolvedProfileResolution),
    Uncovered,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedProfileAuthenticationObservation {
    pub(crate) subject: ResolvedProfileAuthenticationSubject,
    pub(crate) observed: LogicalType,
    pub(crate) coverage: ResolvedProfileAuthenticationCoverage,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AuthenticatedResolvedProfileProgram {
    pub(crate) program: ResolvedProfileProgram,
    pub(crate) coverage: Vec<ResolvedProfileAuthenticationObservation>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolvedProfileAuthenticationError {
    InvalidDescriptor {
        context: String,
    },
    DescriptorNominalMismatch {
        normalized: String,
        expected: Option<LogicalType>,
        observed: Option<LogicalType>,
    },
    FunctionSignatureMismatch {
        function: String,
        expected_parameters: Vec<(String, LogicalType)>,
        expected_result: Option<LogicalType>,
        observed: Option<FunctionSignature>,
    },
    MetadataNominalMismatch {
        normalized: String,
        expected: LogicalType,
        observed: LogicalType,
    },
}

#[allow(dead_code)]
pub(crate) fn authenticate_resolved_profile(
    descriptor: ResolvedProfileProgram,
    checked_ir: &CheckedIr,
) -> Result<AuthenticatedResolvedProfileProgram, ResolvedProfileAuthenticationError> {
    let _metadata = checked_ir.metadata();
    let _descriptor = descriptor;
    todo!("CAP-029 intentional mutation red: authentication is not implemented")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language_profile::ProfileTypeUse;
    use crate::resolved_profile_shape::{ResolvedProfileNominal, ResolvedProfileOrigin};
    use crate::{IrGenerator, SemanticAnalyzer, parse_with_locations, try_tokenize_with_locations};

    const AUTHENTICATION_FIXTURE: &str = r#"
struct Pair {
    count: int,
    ready: bool,
}

struct Unused {
    value: int,
}

struct Box<T> {
    value: T,
}

fn carry(pair: Pair, flag: bool) -> Pair {
    let mut current: Pair = pair;
    let inferred = Pair { count: pair.count, ready: flag };
    if flag {
        current = inferred;
    }
    return current;
}

fn boxed(value: int) -> Box<int> {
    return Box { value: value };
}

fn wrap(pair: Pair, valid: bool) -> Result<Pair, int> {
    if valid {
        return Ok(pair);
    }
    return Err(7);
}

fn score(value: Result<Pair, int>) -> int {
    return match value {
        Ok(pair) => pair.count,
        Err(code) => code,
    };
}

fn main() -> int {
    let pair: Pair = Pair { count: 3, ready: 1 < 2 };
    return score(wrap(carry(pair, 1 < 2), 1 < 2));
}
"#;

    fn fixture() -> (ResolvedProfileProgram, CheckedIr) {
        let tokens = try_tokenize_with_locations(AUTHENTICATION_FIXTURE, None)
            .expect("authentication fixture must lex");
        let ast = parse_with_locations(tokens).expect("authentication fixture must parse");
        let (_, analyzed_ast, descriptor) = SemanticAnalyzer::new()
            .analyze_with_resolved_profile(ast)
            .expect("authentication fixture must pass rich semantics");
        let checked_ir = IrGenerator::new()
            .try_generate_ir(analyzed_ast)
            .expect("authentication fixture must reach verified checked IR");
        (descriptor, checked_ir)
    }

    fn shape_resolution(
        program: &ResolvedProfileProgram,
        expected: &LogicalType,
    ) -> ResolvedProfileResolution {
        ResolvedProfileResolution::Resolved(ResolvedProfileShapeId(
            program
                .shapes
                .iter()
                .position(|shape| shape == expected)
                .unwrap_or_else(|| panic!("fixture omitted logical shape {expected:?}")),
        ))
    }

    fn pair_resolution(program: &ResolvedProfileProgram) -> ResolvedProfileResolution {
        program
            .nominals
            .iter()
            .find_map(|nominal| match nominal {
                ResolvedProfileNominal::Struct {
                    origin: ResolvedProfileOrigin::Source { normalized },
                    resolution,
                    ..
                } if normalized == "Pair" => Some(resolution.clone()),
                _ => None,
            })
            .expect("Pair nominal must be recorded")
    }

    fn rejected_after(
        mutate: impl Fn(&mut ResolvedProfileProgram),
    ) -> ResolvedProfileAuthenticationError {
        let (mut descriptor, checked_ir) = fixture();
        mutate(&mut descriptor);
        let first = authenticate_resolved_profile(descriptor.clone(), &checked_ir)
            .expect_err("mutated descriptor must fail authentication");
        let second = authenticate_resolved_profile(descriptor, &checked_ir)
            .expect_err("the same mutation must fail again");
        assert_eq!(first, second, "authentication errors must be deterministic");
        first
    }

    fn carry_uses(program: &ResolvedProfileProgram, role: ProfileTypeUse) -> Vec<usize> {
        program
            .uses
            .iter()
            .enumerate()
            .filter_map(|(index, usage)| {
                (usage.role == role
                    && matches!(
                        &usage.function,
                        Some(ResolvedProfileOrigin::Source { normalized })
                            if normalized == "carry"
                    ))
                .then_some(index)
            })
            .collect()
    }

    #[test]
    fn authentication_is_deterministic_and_never_promotes_uncovered_or_excluded_facts() {
        let (descriptor, checked_ir) = fixture();
        let first = authenticate_resolved_profile(descriptor.clone(), &checked_ir)
            .expect("baseline descriptor must authenticate");
        let second = authenticate_resolved_profile(descriptor.clone(), &checked_ir)
            .expect("baseline authentication must repeat");
        assert_eq!(first, second);
        assert!(first.program.nominals.iter().any(|nominal| matches!(
            nominal,
            ResolvedProfileNominal::Struct {
                origin: ResolvedProfileOrigin::Source { normalized },
                ..
            } if normalized == "Unused"
        )));
        assert!(first.coverage.iter().any(|observation| {
            observation.coverage == ResolvedProfileAuthenticationCoverage::Uncovered
                && matches!(
                    observation.observed,
                    LogicalType::Struct { ref name, .. } if name == "Pair"
                )
        }));
        assert!(first.coverage.iter().any(|observation| matches!(
            observation.coverage,
            ResolvedProfileAuthenticationCoverage::ExplicitUnavailable(
                ResolvedProfileResolution::Excluded(Some(_))
            )
        )));
        assert!(!first.coverage.iter().any(|observation| {
            matches!(
                observation.coverage,
                ResolvedProfileAuthenticationCoverage::Authenticated(_)
            ) && matches!(
                observation.subject,
                ResolvedProfileAuthenticationSubject::MetadataResult { .. }
            )
        }));

        let mut without_operations = descriptor;
        without_operations.operations.clear();
        authenticate_resolved_profile(without_operations, &checked_ir)
            .expect("operation occurrence counts are not authentication authority");
    }

    #[test]
    fn struct_schema_type_order_and_count_mutations_are_rejected() {
        let type_error = rejected_after(|program| {
            let bool_resolution = shape_resolution(program, &LogicalType::Bool);
            let fields = pair_fields_mut(program);
            fields[0].resolution = bool_resolution;
        });
        assert!(matches!(
            type_error,
            ResolvedProfileAuthenticationError::DescriptorNominalMismatch { .. }
        ));

        let order_error = rejected_after(|program| pair_fields_mut(program).swap(0, 1));
        assert!(matches!(
            order_error,
            ResolvedProfileAuthenticationError::DescriptorNominalMismatch { .. }
        ));

        let count_error = rejected_after(|program| {
            pair_fields_mut(program).pop();
        });
        assert!(matches!(
            count_error,
            ResolvedProfileAuthenticationError::DescriptorNominalMismatch { .. }
        ));
    }

    fn pair_fields_mut(
        program: &mut ResolvedProfileProgram,
    ) -> &mut Vec<crate::resolved_profile_shape::ResolvedProfileField> {
        program
            .nominals
            .iter_mut()
            .find_map(|nominal| match nominal {
                ResolvedProfileNominal::Struct {
                    origin: ResolvedProfileOrigin::Source { normalized },
                    fields,
                    ..
                } if normalized == "Pair" => Some(fields),
                _ => None,
            })
            .expect("Pair fields must be mutable in the fixture")
    }

    #[test]
    fn exact_result_identity_variant_order_and_payload_mutations_are_rejected() {
        let identity_error = rejected_after(|program| {
            let result = program
                .nominals
                .iter_mut()
                .find(|nominal| {
                    matches!(
                        nominal,
                        ResolvedProfileNominal::Enum {
                            origin: ResolvedProfileOrigin::BuiltinCarrier { source, .. },
                            ..
                        } if source == "Result<Pair, int>"
                    )
                })
                .expect("Result nominal must be present");
            let ResolvedProfileNominal::Enum {
                origin: ResolvedProfileOrigin::BuiltinCarrier { normalized, source },
                ..
            } = result
            else {
                unreachable!("matched Result nominal")
            };
            *normalized = source.clone();
        });
        assert!(matches!(
            identity_error,
            ResolvedProfileAuthenticationError::DescriptorNominalMismatch { .. }
        ));

        let order_error = rejected_after(|program| result_variants_mut(program).swap(0, 1));
        assert!(matches!(
            order_error,
            ResolvedProfileAuthenticationError::DescriptorNominalMismatch { .. }
        ));

        let payload_error = rejected_after(|program| {
            let int_resolution = shape_resolution(program, &LogicalType::Int);
            result_variants_mut(program)[0].payload = Some(int_resolution);
        });
        assert!(matches!(
            payload_error,
            ResolvedProfileAuthenticationError::DescriptorNominalMismatch { .. }
        ));

        let error_payload_error = rejected_after(|program| {
            let pair_resolution = pair_resolution(program);
            result_variants_mut(program)[1].payload = Some(pair_resolution);
        });
        assert!(matches!(
            error_payload_error,
            ResolvedProfileAuthenticationError::DescriptorNominalMismatch { .. }
        ));
    }

    fn result_variants_mut(
        program: &mut ResolvedProfileProgram,
    ) -> &mut Vec<crate::resolved_profile_shape::ResolvedProfileVariant> {
        program
            .nominals
            .iter_mut()
            .find_map(|nominal| match nominal {
                ResolvedProfileNominal::Enum {
                    origin: ResolvedProfileOrigin::BuiltinCarrier { source, .. },
                    variants,
                    ..
                } if source == "Result<Pair, int>" => Some(variants),
                _ => None,
            })
            .expect("Result variants must be mutable in the fixture")
    }

    #[test]
    fn function_parameter_name_order_type_and_explicit_result_mutations_are_rejected() {
        let name_error = rejected_after(|program| {
            let parameter = carry_uses(program, ProfileTypeUse::Parameter)[0];
            program.uses[parameter].name = Some("changed".to_string());
        });
        assert!(matches!(
            name_error,
            ResolvedProfileAuthenticationError::FunctionSignatureMismatch { .. }
        ));

        let order_error = rejected_after(|program| {
            let parameters = carry_uses(program, ProfileTypeUse::Parameter);
            program.uses.swap(parameters[0], parameters[1]);
        });
        assert!(matches!(
            order_error,
            ResolvedProfileAuthenticationError::FunctionSignatureMismatch { .. }
        ));

        let type_error = rejected_after(|program| {
            let bool_resolution = shape_resolution(program, &LogicalType::Bool);
            let parameter = carry_uses(program, ProfileTypeUse::Parameter)[0];
            program.uses[parameter].resolution = bool_resolution;
        });
        assert!(matches!(
            type_error,
            ResolvedProfileAuthenticationError::FunctionSignatureMismatch { .. }
        ));

        let result_error = rejected_after(|program| {
            let int_resolution = shape_resolution(program, &LogicalType::Int);
            let result = carry_uses(program, ProfileTypeUse::Result)
                .into_iter()
                .next()
                .expect("carry has an explicit result use");
            program.uses[result].resolution = int_resolution;
        });
        assert!(matches!(
            result_error,
            ResolvedProfileAuthenticationError::FunctionSignatureMismatch { .. }
        ));
    }
}
