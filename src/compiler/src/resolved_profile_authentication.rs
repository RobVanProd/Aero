use crate::ir::{CheckedIr, EnumVariantSchema, FunctionSignature, LogicalType, PlaceId, ResultId};
use crate::language_profile::ProfileTypeUse;
use crate::resolved_profile_shape::{
    ResolvedProfileNominal, ResolvedProfileOrigin, ResolvedProfileProgram,
    ResolvedProfileResolution, ResolvedProfileShapeId,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

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
        expected_parameters: Vec<(String, Option<LogicalType>)>,
        expected_result: Option<Option<LogicalType>>,
        observed: Option<FunctionSignature>,
    },
    MetadataNominalMismatch {
        normalized: String,
        expected: LogicalType,
        observed: LogicalType,
    },
}

impl fmt::Display for ResolvedProfileAuthenticationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "IR Verification Error: resolved profile authentication mismatch: {self:?}"
        )
    }
}

impl std::error::Error for ResolvedProfileAuthenticationError {}

#[derive(Debug, Clone)]
struct DescriptorNominal {
    resolution: ResolvedProfileResolution,
    logical: Option<LogicalType>,
    ambiguous: bool,
}

#[derive(Debug, Clone, Default)]
struct DescriptorFunction {
    parameters: Vec<(String, ResolvedProfileResolution)>,
    result: Option<ResolvedProfileResolution>,
    bindings: BTreeMap<String, Vec<(ProfileTypeUse, ResolvedProfileResolution)>>,
}

pub(crate) fn authenticate_resolved_profile(
    descriptor: ResolvedProfileProgram,
    checked_ir: &CheckedIr,
) -> Result<AuthenticatedResolvedProfileProgram, ResolvedProfileAuthenticationError> {
    let metadata = checked_ir.metadata();
    let descriptor_nominals = validate_descriptor_nominals(&descriptor)?;
    let descriptor_functions = collect_descriptor_functions(&descriptor)?;
    let mut coverage = Vec::new();
    let mut metadata_nominals = BTreeMap::<String, LogicalType>::new();
    let mut matched_functions = BTreeSet::<String>::new();

    for (function, observed) in &metadata.functions {
        collect_metadata_nominals(&observed.signature.result, &mut metadata_nominals)?;
        for (_, logical) in &observed.signature.parameters {
            collect_metadata_nominals(logical, &mut metadata_nominals)?;
        }
        for logical in observed.results.values() {
            collect_metadata_nominals(logical, &mut metadata_nominals)?;
        }
        for place in observed.places.values() {
            collect_metadata_nominals(&place.pointee, &mut metadata_nominals)?;
        }

        if let Some(expected) = descriptor_functions.get(function) {
            matched_functions.insert(function.clone());
            let expected_parameters = expected_parameters(&descriptor, expected)?;
            let expected_result = expected_result(&descriptor, expected)?;
            if !signature_matches(&expected_parameters, &expected_result, &observed.signature) {
                return Err(
                    ResolvedProfileAuthenticationError::FunctionSignatureMismatch {
                        function: function.clone(),
                        expected_parameters,
                        expected_result,
                        observed: Some(observed.signature.clone()),
                    },
                );
            }

            for (index, ((name, logical), (_, resolution))) in observed
                .signature
                .parameters
                .iter()
                .zip(&expected.parameters)
                .enumerate()
            {
                coverage.push(ResolvedProfileAuthenticationObservation {
                    subject: ResolvedProfileAuthenticationSubject::FunctionParameter {
                        function: function.clone(),
                        index,
                        name: name.clone(),
                    },
                    observed: logical.clone(),
                    coverage: coverage_for_resolution(
                        &descriptor,
                        resolution,
                        logical,
                        &format!("function `{function}` parameter {index}"),
                    )?,
                });
            }
            coverage.push(ResolvedProfileAuthenticationObservation {
                subject: ResolvedProfileAuthenticationSubject::FunctionResult {
                    function: function.clone(),
                },
                observed: observed.signature.result.clone(),
                coverage: match &expected.result {
                    Some(resolution) => coverage_for_resolution(
                        &descriptor,
                        resolution,
                        &observed.signature.result,
                        &format!("function `{function}` result"),
                    )?,
                    None => ResolvedProfileAuthenticationCoverage::Uncovered,
                },
            });
        } else {
            for (index, (name, logical)) in observed.signature.parameters.iter().enumerate() {
                coverage.push(ResolvedProfileAuthenticationObservation {
                    subject: ResolvedProfileAuthenticationSubject::FunctionParameter {
                        function: function.clone(),
                        index,
                        name: name.clone(),
                    },
                    observed: logical.clone(),
                    coverage: ResolvedProfileAuthenticationCoverage::Uncovered,
                });
            }
            coverage.push(ResolvedProfileAuthenticationObservation {
                subject: ResolvedProfileAuthenticationSubject::FunctionResult {
                    function: function.clone(),
                },
                observed: observed.signature.result.clone(),
                coverage: ResolvedProfileAuthenticationCoverage::Uncovered,
            });
        }

        for (result, logical) in &observed.results {
            coverage.push(ResolvedProfileAuthenticationObservation {
                subject: ResolvedProfileAuthenticationSubject::MetadataResult {
                    function: function.clone(),
                    result: *result,
                },
                observed: logical.clone(),
                coverage: ResolvedProfileAuthenticationCoverage::Uncovered,
            });
        }
        let mut named_place_counts = BTreeMap::<&str, usize>::new();
        for metadata in observed.places.values() {
            if let Some(name) = metadata.name.as_deref() {
                *named_place_counts.entry(name).or_default() += 1;
            }
        }
        for (place, metadata) in &observed.places {
            let place_coverage = metadata
                .name
                .as_deref()
                .and_then(|name| {
                    descriptor_functions.get(function).and_then(|expected| {
                        unique_mutable_binding(expected, name, &named_place_counts)
                    })
                })
                .map_or(
                    Ok(ResolvedProfileAuthenticationCoverage::Uncovered),
                    |resolution| local_place_coverage(&descriptor, resolution, &metadata.pointee),
                )?;
            coverage.push(ResolvedProfileAuthenticationObservation {
                subject: ResolvedProfileAuthenticationSubject::MetadataPlace {
                    function: function.clone(),
                    place: *place,
                    name: metadata.name.clone(),
                },
                observed: metadata.pointee.clone(),
                coverage: place_coverage,
            });
        }
    }

    for (function, expected) in &descriptor_functions {
        if matched_functions.contains(function) {
            continue;
        }
        return Err(
            ResolvedProfileAuthenticationError::FunctionSignatureMismatch {
                function: function.clone(),
                expected_parameters: expected_parameters(&descriptor, expected)?,
                expected_result: expected_result(&descriptor, expected)?,
                observed: None,
            },
        );
    }

    for (normalized, observed) in metadata_nominals {
        let nominal_coverage = match descriptor_nominals.get(&normalized) {
            Some(expected) if expected.ambiguous => {
                ResolvedProfileAuthenticationCoverage::Uncovered
            }
            Some(expected) => {
                if expected
                    .logical
                    .as_ref()
                    .is_some_and(|logical| logical != &observed)
                {
                    return Err(
                        ResolvedProfileAuthenticationError::MetadataNominalMismatch {
                            normalized,
                            expected: expected
                                .logical
                                .clone()
                                .expect("checked above that the expected shape is present"),
                            observed,
                        },
                    );
                }
                coverage_for_resolution(
                    &descriptor,
                    &expected.resolution,
                    &observed,
                    "metadata nominal",
                )?
            }
            None => ResolvedProfileAuthenticationCoverage::Uncovered,
        };
        coverage.push(ResolvedProfileAuthenticationObservation {
            subject: ResolvedProfileAuthenticationSubject::Nominal { normalized },
            observed,
            coverage: nominal_coverage,
        });
    }

    Ok(AuthenticatedResolvedProfileProgram {
        program: descriptor,
        coverage,
    })
}

fn validate_descriptor_nominals(
    descriptor: &ResolvedProfileProgram,
) -> Result<BTreeMap<String, DescriptorNominal>, ResolvedProfileAuthenticationError> {
    let mut nominals = BTreeMap::<String, DescriptorNominal>::new();
    for nominal in &descriptor.nominals {
        let (origin, resolution) = match nominal {
            ResolvedProfileNominal::Struct {
                origin, resolution, ..
            }
            | ResolvedProfileNominal::Enum {
                origin, resolution, ..
            } => (origin, resolution),
        };
        let normalized = nominal_identity(origin).ok_or_else(|| {
            ResolvedProfileAuthenticationError::InvalidDescriptor {
                context: format!("non-nominal origin {origin:?}"),
            }
        })?;
        let observed =
            logical_for_resolution(descriptor, resolution, &format!("nominal `{normalized}`"))?;
        let expected = match (&observed, nominal) {
            (None, _) => None,
            (Some(_), ResolvedProfileNominal::Struct { fields, .. }) => {
                let mut logical_fields = Vec::with_capacity(fields.len());
                for (index, field) in fields.iter().enumerate() {
                    let Some(logical) = logical_for_resolution(
                        descriptor,
                        &field.resolution,
                        &format!("nominal `{normalized}` field {index}"),
                    )?
                    else {
                        return Err(
                            ResolvedProfileAuthenticationError::DescriptorNominalMismatch {
                                normalized: normalized.to_string(),
                                expected: None,
                                observed,
                            },
                        );
                    };
                    logical_fields.push(logical);
                }
                Some(LogicalType::Struct {
                    name: normalized.to_string(),
                    fields: logical_fields,
                })
            }
            (Some(_), ResolvedProfileNominal::Enum { variants, .. }) => {
                let mut logical_variants = Vec::with_capacity(variants.len());
                for (index, variant) in variants.iter().enumerate() {
                    let payload = match &variant.payload {
                        Some(resolution) => {
                            let Some(logical) = logical_for_resolution(
                                descriptor,
                                resolution,
                                &format!("nominal `{normalized}` variant {index}"),
                            )?
                            else {
                                return Err(
                                    ResolvedProfileAuthenticationError::DescriptorNominalMismatch {
                                        normalized: normalized.to_string(),
                                        expected: None,
                                        observed,
                                    },
                                );
                            };
                            Some(logical)
                        }
                        None => None,
                    };
                    logical_variants.push(EnumVariantSchema {
                        name: variant.name.clone(),
                        payload,
                    });
                }
                Some(LogicalType::Enum {
                    name: normalized.to_string(),
                    variants: logical_variants,
                })
            }
        };

        if expected != observed {
            return Err(
                ResolvedProfileAuthenticationError::DescriptorNominalMismatch {
                    normalized: normalized.to_string(),
                    expected,
                    observed,
                },
            );
        }
        if let Some(existing) = nominals.get_mut(normalized) {
            if existing.logical.is_some() && observed.is_some() {
                return Err(ResolvedProfileAuthenticationError::InvalidDescriptor {
                    context: format!("duplicate concrete nominal identity `{normalized}`"),
                });
            }
            existing.ambiguous = true;
            if observed.is_some() {
                existing.resolution = resolution.clone();
                existing.logical = observed;
            }
            continue;
        }
        nominals.insert(
            normalized.to_string(),
            DescriptorNominal {
                resolution: resolution.clone(),
                logical: observed,
                ambiguous: false,
            },
        );
    }
    Ok(nominals)
}

fn collect_descriptor_functions(
    descriptor: &ResolvedProfileProgram,
) -> Result<BTreeMap<String, DescriptorFunction>, ResolvedProfileAuthenticationError> {
    let mut functions = BTreeMap::<String, DescriptorFunction>::new();
    for usage in &descriptor.uses {
        let Some(origin) = &usage.function else {
            continue;
        };
        let Some(function) = checked_function_identity(origin) else {
            continue;
        };
        match usage.role {
            ProfileTypeUse::Parameter => {
                let name = usage.name.clone().ok_or_else(|| {
                    ResolvedProfileAuthenticationError::InvalidDescriptor {
                        context: format!("function `{function}` parameter without a name"),
                    }
                })?;
                functions
                    .entry(function.to_string())
                    .or_default()
                    .parameters
                    .push((name, usage.resolution.clone()));
            }
            ProfileTypeUse::Result => {
                let entry = functions.entry(function.to_string()).or_default();
                if entry.result.replace(usage.resolution.clone()).is_some() {
                    return Err(ResolvedProfileAuthenticationError::InvalidDescriptor {
                        context: format!("function `{function}` has duplicate result roots"),
                    });
                }
            }
            ProfileTypeUse::Binding | ProfileTypeUse::MutableBinding => {
                let name = usage.name.clone().ok_or_else(|| {
                    ResolvedProfileAuthenticationError::InvalidDescriptor {
                        context: format!("function `{function}` binding without a name"),
                    }
                })?;
                functions
                    .entry(function.to_string())
                    .or_default()
                    .bindings
                    .entry(name)
                    .or_default()
                    .push((usage.role, usage.resolution.clone()));
            }
            ProfileTypeUse::OwnedAssignment | ProfileTypeUse::Value => {}
        }
    }
    Ok(functions)
}

fn unique_mutable_binding<'a>(
    function: &'a DescriptorFunction,
    name: &str,
    named_place_counts: &BTreeMap<&str, usize>,
) -> Option<&'a ResolvedProfileResolution> {
    if named_place_counts.get(name) != Some(&1)
        || function
            .parameters
            .iter()
            .any(|(parameter, _)| parameter == name)
    {
        return None;
    }
    let [(ProfileTypeUse::MutableBinding, resolution)] = function.bindings.get(name)?.as_slice()
    else {
        return None;
    };
    Some(resolution)
}

fn local_place_coverage(
    descriptor: &ResolvedProfileProgram,
    resolution: &ResolvedProfileResolution,
    observed: &LogicalType,
) -> Result<ResolvedProfileAuthenticationCoverage, ResolvedProfileAuthenticationError> {
    let Some(expected) = logical_for_resolution(descriptor, resolution, "local mutable binding")?
    else {
        return Ok(ResolvedProfileAuthenticationCoverage::Uncovered);
    };
    if &expected != observed {
        return Ok(ResolvedProfileAuthenticationCoverage::Uncovered);
    }
    Ok(match resolution {
        ResolvedProfileResolution::Resolved(id) => {
            ResolvedProfileAuthenticationCoverage::Authenticated(*id)
        }
        ResolvedProfileResolution::Excluded(Some(_)) => {
            ResolvedProfileAuthenticationCoverage::ExplicitUnavailable(resolution.clone())
        }
        ResolvedProfileResolution::Excluded(None) | ResolvedProfileResolution::Unresolved => {
            unreachable!("unavailable local resolutions returned before coverage")
        }
    })
}

fn expected_parameters(
    descriptor: &ResolvedProfileProgram,
    function: &DescriptorFunction,
) -> Result<Vec<(String, Option<LogicalType>)>, ResolvedProfileAuthenticationError> {
    function
        .parameters
        .iter()
        .enumerate()
        .map(|(index, (name, resolution))| {
            Ok((
                name.clone(),
                logical_for_resolution(
                    descriptor,
                    resolution,
                    &format!("function parameter {index}"),
                )?,
            ))
        })
        .collect()
}

fn expected_result(
    descriptor: &ResolvedProfileProgram,
    function: &DescriptorFunction,
) -> Result<Option<Option<LogicalType>>, ResolvedProfileAuthenticationError> {
    function
        .result
        .as_ref()
        .map(|resolution| logical_for_resolution(descriptor, resolution, "function result"))
        .transpose()
}

fn signature_matches(
    expected_parameters: &[(String, Option<LogicalType>)],
    expected_result: &Option<Option<LogicalType>>,
    observed: &FunctionSignature,
) -> bool {
    expected_parameters.len() == observed.parameters.len()
        && expected_parameters.iter().zip(&observed.parameters).all(
            |((expected_name, expected_type), (observed_name, observed_type))| {
                expected_name == observed_name
                    && expected_type
                        .as_ref()
                        .is_none_or(|logical| logical == observed_type)
            },
        )
        && expected_result
            .as_ref()
            .is_none_or(|expected| expected.as_ref().is_none_or(|ty| ty == &observed.result))
}

fn coverage_for_resolution(
    descriptor: &ResolvedProfileProgram,
    resolution: &ResolvedProfileResolution,
    observed: &LogicalType,
    context: &str,
) -> Result<ResolvedProfileAuthenticationCoverage, ResolvedProfileAuthenticationError> {
    match resolution {
        ResolvedProfileResolution::Resolved(id) => {
            require_shape(descriptor, *id, context, observed)?;
            Ok(ResolvedProfileAuthenticationCoverage::Authenticated(*id))
        }
        ResolvedProfileResolution::Excluded(Some(id)) => {
            require_shape(descriptor, *id, context, observed)?;
            Ok(ResolvedProfileAuthenticationCoverage::ExplicitUnavailable(
                resolution.clone(),
            ))
        }
        ResolvedProfileResolution::Excluded(None) | ResolvedProfileResolution::Unresolved => Ok(
            ResolvedProfileAuthenticationCoverage::ExplicitUnavailable(resolution.clone()),
        ),
    }
}

fn require_shape(
    descriptor: &ResolvedProfileProgram,
    id: ResolvedProfileShapeId,
    context: &str,
    observed: &LogicalType,
) -> Result<(), ResolvedProfileAuthenticationError> {
    let expected = descriptor.shapes.get(id.0).ok_or_else(|| {
        ResolvedProfileAuthenticationError::InvalidDescriptor {
            context: format!("{context} references missing shape {}", id.0),
        }
    })?;
    if expected != observed {
        return Err(ResolvedProfileAuthenticationError::InvalidDescriptor {
            context: format!(
                "{context} expected logical shape {expected:?}, observed {observed:?}"
            ),
        });
    }
    Ok(())
}

fn logical_for_resolution(
    descriptor: &ResolvedProfileProgram,
    resolution: &ResolvedProfileResolution,
    context: &str,
) -> Result<Option<LogicalType>, ResolvedProfileAuthenticationError> {
    match resolution {
        ResolvedProfileResolution::Resolved(id) | ResolvedProfileResolution::Excluded(Some(id)) => {
            descriptor
                .shapes
                .get(id.0)
                .cloned()
                .map(Some)
                .ok_or_else(|| ResolvedProfileAuthenticationError::InvalidDescriptor {
                    context: format!("{context} references missing shape {}", id.0),
                })
        }
        ResolvedProfileResolution::Excluded(None) | ResolvedProfileResolution::Unresolved => {
            Ok(None)
        }
    }
}

fn nominal_identity(origin: &ResolvedProfileOrigin) -> Option<&str> {
    match origin {
        ResolvedProfileOrigin::Source { normalized }
        | ResolvedProfileOrigin::SourceGenericStruct { normalized }
        | ResolvedProfileOrigin::SourceGenericEnum { normalized }
        | ResolvedProfileOrigin::GenericStruct { normalized, .. }
        | ResolvedProfileOrigin::GenericEnum { normalized, .. }
        | ResolvedProfileOrigin::BuiltinCarrier { normalized, .. }
        | ResolvedProfileOrigin::OpaquePrivate { normalized } => Some(normalized),
        ResolvedProfileOrigin::ImplMethod { .. }
        | ResolvedProfileOrigin::TraitMethod { .. }
        | ResolvedProfileOrigin::SourceGenericFunction { .. }
        | ResolvedProfileOrigin::GenericFunction { .. } => None,
    }
}

fn checked_function_identity(origin: &ResolvedProfileOrigin) -> Option<&str> {
    match origin {
        ResolvedProfileOrigin::Source { normalized }
        | ResolvedProfileOrigin::GenericFunction { normalized, .. }
        | ResolvedProfileOrigin::OpaquePrivate { normalized } => Some(normalized),
        ResolvedProfileOrigin::ImplMethod { .. }
        | ResolvedProfileOrigin::TraitMethod { .. }
        | ResolvedProfileOrigin::SourceGenericStruct { .. }
        | ResolvedProfileOrigin::SourceGenericEnum { .. }
        | ResolvedProfileOrigin::SourceGenericFunction { .. }
        | ResolvedProfileOrigin::GenericStruct { .. }
        | ResolvedProfileOrigin::GenericEnum { .. }
        | ResolvedProfileOrigin::BuiltinCarrier { .. } => None,
    }
}

fn collect_metadata_nominals(
    root: &LogicalType,
    nominals: &mut BTreeMap<String, LogicalType>,
) -> Result<(), ResolvedProfileAuthenticationError> {
    let mut pending = vec![root];
    while let Some(logical) = pending.pop() {
        match logical {
            LogicalType::Struct { name, fields } => {
                insert_metadata_nominal(name, logical, nominals)?;
                pending.extend(fields.iter().rev());
            }
            LogicalType::Enum { name, variants } => {
                insert_metadata_nominal(name, logical, nominals)?;
                pending.extend(
                    variants
                        .iter()
                        .rev()
                        .filter_map(|variant| variant.payload.as_ref()),
                );
            }
            LogicalType::ImmutableReference { pointee }
            | LogicalType::MutableReference { pointee }
            | LogicalType::Array {
                element: pointee, ..
            } => pending.push(pointee),
            LogicalType::Tuple { elements } | LogicalType::EnumFields { fields: elements } => {
                pending.extend(elements.iter().rev());
            }
            LogicalType::Int
            | LogicalType::Float
            | LogicalType::Bool
            | LogicalType::Char
            | LogicalType::Void
            | LogicalType::String => {}
        }
    }
    Ok(())
}

fn insert_metadata_nominal(
    normalized: &str,
    logical: &LogicalType,
    nominals: &mut BTreeMap<String, LogicalType>,
) -> Result<(), ResolvedProfileAuthenticationError> {
    if let Some(expected) = nominals.get(normalized) {
        if expected != logical {
            return Err(
                ResolvedProfileAuthenticationError::MetadataNominalMismatch {
                    normalized: normalized.to_string(),
                    expected: expected.clone(),
                    observed: logical.clone(),
                },
            );
        }
    } else {
        nominals.insert(normalized.to_string(), logical.clone());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language_profile::ProfileTypeUse;
    use crate::resolved_profile_shape::{ResolvedProfileNominal, ResolvedProfileOrigin};
    use crate::{IrGenerator, SemanticAnalyzer, parse_with_locations, try_tokenize_with_locations};

    const AUTHENTICATION_FIXTURE: &str = r#"
struct Leaf {
    value: int,
}

struct Pair {
    leaf: Leaf,
    ready: bool,
}

struct Unused {
    value: int,
}

struct Box<T> {
    value: T,
}

fn choose<T>(first: T, second: T, use_first: bool) -> T {
    if use_first {
        return first;
    }
    return second;
}

fn carry(pair: Pair, flag: bool) -> Pair {
    let mut current: Pair = pair;
    let mut inferred = Pair { leaf: pair.leaf, ready: flag };
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
        Ok(pair) => pair.leaf.value,
        Err(code) => code,
    };
}

fn main() -> int {
    let pair: Pair = Pair {
        leaf: Leaf { value: 3 },
        ready: 1 < 2,
    };
    let chosen = choose(1, 2, 1 < 2);
    return score(wrap(carry(pair, 1 < 2), 1 < 2)) + chosen;
}
"#;

    fn fixture() -> (ResolvedProfileProgram, CheckedIr) {
        fixture_for(AUTHENTICATION_FIXTURE)
    }

    fn fixture_for(source: &str) -> (ResolvedProfileProgram, CheckedIr) {
        let tokens =
            try_tokenize_with_locations(source, None).expect("authentication fixture must lex");
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
        let inferred_places = first
            .coverage
            .iter()
            .filter(|observation| {
                matches!(
                    &observation.subject,
                    ResolvedProfileAuthenticationSubject::MetadataPlace {
                        name: Some(name),
                        ..
                    } if name == "inferred"
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            inferred_places.len(),
            1,
            "the unannotated mutable aggregate must expose one named metadata place"
        );
        assert!(matches!(
            inferred_places[0].observed,
            LogicalType::Struct { ref name, .. } if name == "Pair"
        ));
        assert_eq!(
            inferred_places[0].coverage,
            ResolvedProfileAuthenticationCoverage::Uncovered
        );
        let current_places = first
            .coverage
            .iter()
            .filter(|observation| {
                matches!(
                    &observation.subject,
                    ResolvedProfileAuthenticationSubject::MetadataPlace {
                        function,
                        name: Some(name),
                        ..
                    } if function == "carry" && name == "current"
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            current_places.len(),
            1,
            "the unique explicit mutable binding must expose one named place"
        );
        assert_eq!(
            current_places[0].coverage,
            ResolvedProfileAuthenticationCoverage::Authenticated(
                match pair_resolution(&first.program) {
                    ResolvedProfileResolution::Resolved(id) => id,
                    resolution => panic!("Pair must remain resolved, got {resolution:?}"),
                }
            )
        );

        let generic_function = first
            .program
            .uses
            .iter()
            .find_map(|usage| match &usage.function {
                Some(ResolvedProfileOrigin::GenericFunction { normalized, source })
                    if source == "choose<int>" =>
                {
                    Some(normalized.clone())
                }
                _ => None,
            })
            .expect("the concrete generic function identity must be retained");
        let generic_signature = first
            .coverage
            .iter()
            .filter(|observation| match &observation.subject {
                ResolvedProfileAuthenticationSubject::FunctionParameter { function, .. }
                | ResolvedProfileAuthenticationSubject::FunctionResult { function } => {
                    function == &generic_function
                }
                _ => false,
            })
            .collect::<Vec<_>>();
        assert_eq!(generic_signature.len(), 4);
        assert!(generic_signature.iter().all(|observation| matches!(
            observation.coverage,
            ResolvedProfileAuthenticationCoverage::ExplicitUnavailable(
                ResolvedProfileResolution::Excluded(Some(_))
            )
        )));
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

        let mut without_operations = descriptor.clone();
        without_operations.operations.clear();
        authenticate_resolved_profile(without_operations, &checked_ir)
            .expect("operation occurrence counts are not authentication authority");
        let mut duplicate_operation = descriptor;
        let operation = duplicate_operation
            .operations
            .first()
            .expect("fixture must contain an observed operation")
            .clone();
        duplicate_operation.operations.push(operation);
        authenticate_resolved_profile(duplicate_operation, &checked_ir)
            .expect("duplicated operation counts are not authentication authority");

        for unavailable in [
            ResolvedProfileResolution::Excluded(None),
            ResolvedProfileResolution::Unresolved,
        ] {
            let (mut descriptor, checked_ir) = fixture();
            let parameter = carry_uses(&descriptor, ProfileTypeUse::Parameter)[0];
            descriptor.uses[parameter].resolution = unavailable.clone();
            let authenticated = authenticate_resolved_profile(descriptor, &checked_ir)
                .expect("an unavailable explicit fact must fail closed without rejection");
            assert!(authenticated.coverage.iter().any(|observation| {
                matches!(
                    &observation.subject,
                    ResolvedProfileAuthenticationSubject::FunctionParameter {
                        function,
                        index: 0,
                        ..
                    } if function == "carry"
                ) && observation.coverage
                    == ResolvedProfileAuthenticationCoverage::ExplicitUnavailable(
                        unavailable.clone(),
                    )
            }));
        }
        for unavailable in [
            ResolvedProfileResolution::Excluded(None),
            ResolvedProfileResolution::Unresolved,
        ] {
            let (mut descriptor, checked_ir) = fixture();
            let current = descriptor
                .uses
                .iter()
                .position(|usage| {
                    usage.role == ProfileTypeUse::MutableBinding
                        && usage.name.as_deref() == Some("current")
                        && matches!(
                            &usage.function,
                            Some(ResolvedProfileOrigin::Source { normalized })
                                if normalized == "carry"
                        )
                })
                .expect("carry/current mutable binding must be recorded");
            descriptor.uses[current].resolution = unavailable;
            let authenticated = authenticate_resolved_profile(descriptor, &checked_ir)
                .expect("a shape-less local fact must stay uncovered without rejection");
            assert!(authenticated.coverage.iter().any(|observation| {
                matches!(
                    &observation.subject,
                    ResolvedProfileAuthenticationSubject::MetadataPlace {
                        function,
                        name: Some(name),
                        ..
                    } if function == "carry" && name == "current"
                ) && observation.coverage == ResolvedProfileAuthenticationCoverage::Uncovered
            }));
        }
    }

    #[test]
    fn inferred_shadowed_and_parameter_colliding_local_places_are_uncovered() {
        let cases = [
            (
                r#"
struct Row { value: int }
fn main() -> int {
    { let item: Row = Row { value: 1 }; }
    { let item = 2; }
    return 0;
}
"#,
                vec![LogicalType::Int],
            ),
            (
                r#"
struct Row { value: int }
fn main() -> int {
    { let item: Row = Row { value: 1 }; }
    { let mut item = Row { value: 2 }; item = Row { value: 3 }; }
    return 0;
}
"#,
                vec![LogicalType::Struct {
                    name: "Row".to_string(),
                    fields: vec![LogicalType::Int],
                }],
            ),
            (
                r#"
struct Row { value: int }
fn inspect(item: Row) -> int {
    { let mut item: Row = Row { value: 2 }; item = Row { value: 3 }; }
    return item.value;
}
fn main() -> int {
    return inspect(Row { value: 1 });
}
"#,
                vec![
                    LogicalType::Struct {
                        name: "Row".to_string(),
                        fields: vec![LogicalType::Int],
                    },
                    LogicalType::Struct {
                        name: "Row".to_string(),
                        fields: vec![LogicalType::Int],
                    },
                ],
            ),
            (
                r#"
fn main() -> int {
    { const item: int = 1; }
    { let mut item = 2; item = 3; }
    return 0;
}
"#,
                vec![LogicalType::Int],
            ),
        ];
        for (source, expected) in cases {
            let (descriptor, checked_ir) = fixture_for(source);
            let authenticated = authenticate_resolved_profile(descriptor, &checked_ir)
                .expect("shadowing controls must remain accepted");
            let item_places = authenticated
                .coverage
                .iter()
                .filter(|observation| {
                    matches!(
                        &observation.subject,
                        ResolvedProfileAuthenticationSubject::MetadataPlace {
                            name: Some(name),
                            ..
                        } if name == "item"
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(
                item_places.len(),
                expected.len(),
                "each control must expose the exact verifier-metadata `item` places"
            );
            assert_eq!(
                item_places
                    .iter()
                    .map(|observation| observation.observed.clone())
                    .collect::<Vec<_>>(),
                expected
            );
            assert!(item_places.iter().all(|observation| {
                observation.coverage == ResolvedProfileAuthenticationCoverage::Uncovered
            }));
        }
    }

    #[test]
    fn recursive_nominals_reachable_only_through_local_metadata_are_authenticated() {
        let source = r#"
struct Leaf { value: int }
struct Local { leaf: Leaf }
fn main() -> int {
    let mut local = Local { leaf: Leaf { value: 4 } };
    local = Local { leaf: Leaf { value: 5 } };
    return local.leaf.value;
}
"#;
        let (descriptor, checked_ir) = fixture_for(source);
        let mut signature_nominals = BTreeMap::new();
        let mut local_nominals = BTreeMap::new();
        for metadata in checked_ir.metadata().functions.values() {
            collect_metadata_nominals(&metadata.signature.result, &mut signature_nominals)
                .expect("signature schema collection must remain valid");
            for (_, logical) in &metadata.signature.parameters {
                collect_metadata_nominals(logical, &mut signature_nominals)
                    .expect("parameter schema collection must remain valid");
            }
            for logical in metadata.results.values() {
                collect_metadata_nominals(logical, &mut local_nominals)
                    .expect("result schema collection must remain valid");
            }
            for place in metadata.places.values() {
                collect_metadata_nominals(&place.pointee, &mut local_nominals)
                    .expect("place schema collection must remain valid");
            }
        }
        for nominal in ["Local", "Leaf"] {
            assert!(
                !signature_nominals.contains_key(nominal),
                "{nominal} must not be reachable through a function signature"
            );
            assert!(
                local_nominals.contains_key(nominal),
                "{nominal} must be reachable through result/place metadata"
            );
        }

        let authenticated = authenticate_resolved_profile(descriptor, &checked_ir)
            .expect("local-only recursive metadata must authenticate");
        for nominal in ["Local", "Leaf"] {
            assert!(authenticated.coverage.iter().any(|observation| {
                matches!(
                    &observation.subject,
                    ResolvedProfileAuthenticationSubject::Nominal { normalized }
                        if normalized == nominal
                ) && matches!(
                    observation.coverage,
                    ResolvedProfileAuthenticationCoverage::Authenticated(_)
                )
            }));
        }
        let local_places = authenticated
            .coverage
            .iter()
            .filter(|observation| {
                matches!(
                    &observation.subject,
                    ResolvedProfileAuthenticationSubject::MetadataPlace {
                        name: Some(name),
                        ..
                    } if name == "local"
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(local_places.len(), 1);
        assert!(matches!(
            local_places[0].observed,
            LogicalType::Struct { ref name, .. } if name == "Local"
        ));
        assert_eq!(
            local_places[0].coverage,
            ResolvedProfileAuthenticationCoverage::Uncovered
        );
    }

    #[test]
    fn unused_ambiguous_nominal_declarations_remain_descriptor_only() {
        let source = r#"
struct Duplicate { value: int }
struct Duplicate { value: int }
struct Collision { value: int }
enum Collision { Empty }
fn main() -> int { return 0; }
"#;
        let (descriptor, checked_ir) = fixture_for(source);
        let duplicate_declarations = descriptor
            .nominals
            .iter()
            .filter(|nominal| {
                matches!(
                    nominal,
                    ResolvedProfileNominal::Struct {
                        origin: ResolvedProfileOrigin::Source { normalized },
                        resolution: ResolvedProfileResolution::Unresolved,
                        ..
                    } if normalized == "Duplicate"
                )
            })
            .count();
        assert_eq!(duplicate_declarations, 2);
        assert!(descriptor.nominals.iter().any(|nominal| matches!(
            nominal,
            ResolvedProfileNominal::Struct {
                origin: ResolvedProfileOrigin::Source { normalized },
                resolution: ResolvedProfileResolution::Resolved(_),
                ..
            } if normalized == "Collision"
        )));
        assert!(descriptor.nominals.iter().any(|nominal| matches!(
            nominal,
            ResolvedProfileNominal::Enum {
                origin: ResolvedProfileOrigin::Source { normalized },
                resolution: ResolvedProfileResolution::Unresolved,
                ..
            } if normalized == "Collision"
        )));

        let authenticated = authenticate_resolved_profile(descriptor, &checked_ir)
            .expect("unused ambiguous declarations must retain accepted behavior");
        for normalized in ["Duplicate", "Collision"] {
            assert!(!authenticated.coverage.iter().any(|observation| matches!(
                &observation.subject,
                ResolvedProfileAuthenticationSubject::Nominal {
                    normalized: observed
                } if observed == normalized
            )));
        }
    }

    #[test]
    fn opaque_trait_helpers_are_compared_by_exact_normalized_function_identity() {
        let source = r#"
struct Reading { value: int }
trait Score {
    fn score(&self) -> int;
}
impl Score for Reading {
    fn score(&self) -> int {
        return (*self).value;
    }
}
fn evaluate<T: Score>(reading: T) -> int {
    return reading.score();
}
fn main() -> int {
    return evaluate(Reading { value: 41 }) + 1;
}
"#;
        let (descriptor, checked_ir) = fixture_for(source);
        let opaque_functions = descriptor
            .uses
            .iter()
            .filter_map(|usage| match &usage.function {
                Some(ResolvedProfileOrigin::OpaquePrivate { normalized }) => {
                    Some(normalized.clone())
                }
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        assert!(
            !opaque_functions.is_empty(),
            "trait dispatch must emit an exact opaque helper identity"
        );

        let authenticated = authenticate_resolved_profile(descriptor, &checked_ir)
            .expect("exact opaque helper signatures must authenticate");
        for function in opaque_functions {
            let signature =
                authenticated
                    .coverage
                    .iter()
                    .filter(|observation| match &observation.subject {
                        ResolvedProfileAuthenticationSubject::FunctionParameter {
                            function: observed,
                            ..
                        }
                        | ResolvedProfileAuthenticationSubject::FunctionResult {
                            function: observed,
                        } => observed == &function,
                        _ => false,
                    })
                    .collect::<Vec<_>>();
            assert!(
                !signature.is_empty(),
                "opaque helper `{function}` must be represented in verifier metadata"
            );
            assert!(signature.iter().all(|observation| matches!(
                observation.coverage,
                ResolvedProfileAuthenticationCoverage::ExplicitUnavailable(_)
            )));
        }
    }

    #[test]
    fn verifier_metadata_schema_drift_is_rejected_without_raw_instruction_walking() {
        let (descriptor, checked_ir) = fixture();
        let mut metadata = checked_ir.metadata().clone();
        let carry = metadata
            .functions
            .get_mut("carry")
            .expect("carry metadata must be published");
        let (_, parameter) = carry
            .signature
            .parameters
            .first_mut()
            .expect("carry must have its Pair parameter");
        let LogicalType::Struct { fields, .. } = parameter else {
            panic!("carry parameter must retain the Pair schema")
        };
        fields[0] = LogicalType::Bool;
        let corrupted = CheckedIr::new(checked_ir.raw().clone(), metadata);
        let error = authenticate_resolved_profile(descriptor, &corrupted)
            .expect_err("same-name metadata schema drift must be rejected");
        assert!(matches!(
            error,
            ResolvedProfileAuthenticationError::FunctionSignatureMismatch { .. }
                | ResolvedProfileAuthenticationError::MetadataNominalMismatch { .. }
        ));
    }

    #[test]
    fn missing_shape_and_duplicate_nominal_identities_are_rejected() {
        let missing_shape = rejected_after(|program| {
            let parameter = carry_uses(program, ProfileTypeUse::Parameter)[0];
            program.uses[parameter].resolution =
                ResolvedProfileResolution::Resolved(ResolvedProfileShapeId(usize::MAX));
        });
        assert!(matches!(
            missing_shape,
            ResolvedProfileAuthenticationError::InvalidDescriptor { .. }
        ));

        let duplicate_nominal = rejected_after(|program| {
            let nominal = program
                .nominals
                .iter()
                .find(|nominal| {
                    matches!(
                        nominal,
                        ResolvedProfileNominal::Struct {
                            origin: ResolvedProfileOrigin::Source { normalized },
                            ..
                        } if normalized == "Pair"
                    )
                })
                .expect("Pair nominal must exist")
                .clone();
            program.nominals.push(nominal);
        });
        assert!(matches!(
            duplicate_nominal,
            ResolvedProfileAuthenticationError::InvalidDescriptor { .. }
        ));
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

        let nested_error = rejected_after(|program| {
            let bool_resolution = shape_resolution(program, &LogicalType::Bool);
            let fields = program
                .nominals
                .iter_mut()
                .find_map(|nominal| match nominal {
                    ResolvedProfileNominal::Struct {
                        origin: ResolvedProfileOrigin::Source { normalized },
                        fields,
                        ..
                    } if normalized == "Leaf" => Some(fields),
                    _ => None,
                })
                .expect("Leaf fields must be mutable in the fixture");
            fields[0].resolution = bool_resolution;
        });
        assert!(matches!(
            nested_error,
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
