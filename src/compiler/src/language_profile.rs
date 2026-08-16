use crate::ast::{AstNode, BinaryOp, Block, Expression, Statement, Type, UnaryOp};
use crate::builtin_carrier_contract::private_carrier_source_name;
use crate::byte_buffer_source_contract::{
    BYTES_CAPACITY, BYTES_GET, BYTES_LENGTH, BYTES_NEW, BYTES_PUSH,
    is_reserved_byte_buffer_intrinsic,
};
use crate::byte_input_source_contract::{STDIN_READ_BYTE, is_reserved_byte_input_intrinsic};
use crate::ir::LogicalType;
use crate::resolved_profile_shape::{
    ResolvedProfileAssignmentProjection, ResolvedProfileAssignmentRoot,
    ResolvedProfileBinaryOperator, ResolvedProfileCallArgumentKind, ResolvedProfileExpressionKind,
    ResolvedProfileNominal, ResolvedProfileOperation, ResolvedProfileOrigin,
    ResolvedProfilePatternKind, ResolvedProfileProgram, ResolvedProfileResolution,
    ResolvedProfileStatementKind, ResolvedProfileSurfaceContext, ResolvedProfileSurfaceObservation,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

pub(crate) const STABLE_SCALAR_V0_NAME: &str = "stable-scalar-v0";
pub(crate) const EXACT_I32_ARRAY_V0_NAME: &str = "exact-i32-array-v0";
pub(crate) const EXACT_I32_RECORD_RESULT_V0_NAME: &str = "exact-i32-record-result-v0";
pub(crate) const EXACT_I32_BYTE_BUFFER_V0_NAME: &str = "exact-i32-byte-buffer-v0";
pub(crate) const EXACT_I32_BYTE_INPUT_V0_NAME: &str = "exact-i32-byte-input-v0";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum LanguageProfile {
    #[default]
    Experimental,
    StableScalarV0,
    ExactI32ArrayV0,
    ExactI32RecordResultV0,
    ExactI32ByteBufferV0,
    ExactI32ByteInputV0,
}

impl LanguageProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Experimental => "experimental",
            Self::StableScalarV0 => STABLE_SCALAR_V0_NAME,
            Self::ExactI32ArrayV0 => EXACT_I32_ARRAY_V0_NAME,
            Self::ExactI32RecordResultV0 => EXACT_I32_RECORD_RESULT_V0_NAME,
            Self::ExactI32ByteBufferV0 => EXACT_I32_BYTE_BUFFER_V0_NAME,
            Self::ExactI32ByteInputV0 => EXACT_I32_BYTE_INPUT_V0_NAME,
        }
    }

    /// Whether verified logical `Int` values use the profile's exact i32 lane.
    pub(crate) fn uses_exact_i32_lane(self) -> bool {
        matches!(
            self,
            Self::StableScalarV0
                | Self::ExactI32ArrayV0
                | Self::ExactI32RecordResultV0
                | Self::ExactI32ByteBufferV0
                | Self::ExactI32ByteInputV0
        )
    }

    pub(crate) fn uses_exact_record_result_layout(self) -> bool {
        matches!(
            self,
            Self::ExactI32RecordResultV0 | Self::ExactI32ByteBufferV0 | Self::ExactI32ByteInputV0
        )
    }

    pub(crate) fn enables_byte_buffer_source(self) -> bool {
        matches!(self, Self::ExactI32ByteBufferV0 | Self::ExactI32ByteInputV0)
    }

    pub(crate) fn enables_byte_input_source(self) -> bool {
        self == Self::ExactI32ByteInputV0
    }

    /// Whether this profile admits the exact, flat, nonempty i32-array shape.
    pub(crate) fn admits_exact_i32_array(self, logical_type: &LogicalType) -> bool {
        matches!(
            self,
            Self::ExactI32ArrayV0
                | Self::ExactI32RecordResultV0
                | Self::ExactI32ByteBufferV0
                | Self::ExactI32ByteInputV0
        ) && matches!(
            classify_profile_logical_type(logical_type),
            ProfileTypeShape::ExactI32Array { .. }
        )
    }
}

impl fmt::Display for LanguageProfile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for LanguageProfile {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "experimental" => Ok(Self::Experimental),
            STABLE_SCALAR_V0_NAME => Ok(Self::StableScalarV0),
            EXACT_I32_ARRAY_V0_NAME => Ok(Self::ExactI32ArrayV0),
            EXACT_I32_RECORD_RESULT_V0_NAME => Ok(Self::ExactI32RecordResultV0),
            EXACT_I32_BYTE_BUFFER_V0_NAME => Ok(Self::ExactI32ByteBufferV0),
            EXACT_I32_BYTE_INPUT_V0_NAME => Ok(Self::ExactI32ByteInputV0),
            _ => Err(format!(
                "unsupported language profile `{value}` (expected experimental|{STABLE_SCALAR_V0_NAME}|{EXACT_I32_ARRAY_V0_NAME}|{EXACT_I32_RECORD_RESULT_V0_NAME}|{EXACT_I32_BYTE_BUFFER_V0_NAME}|{EXACT_I32_BYTE_INPUT_V0_NAME})"
            )),
        }
    }
}

/// Normalized source/checked-IR type shapes owned by the profile authority.
///
/// The backend consumes this classification instead of independently deciding
/// which array topologies qualify for the exact i32 physical lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProfileTypeShape {
    Int,
    Bool,
    ExactI32Array { count: usize },
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProfileTypeUse {
    Parameter,
    Result,
    Binding,
    MutableBinding,
    OwnedAssignment,
    Value,
}

pub(crate) fn profile_type_shape_is_admitted(
    profile: LanguageProfile,
    shape: ProfileTypeShape,
    usage: ProfileTypeUse,
) -> bool {
    match shape {
        ProfileTypeShape::Int | ProfileTypeShape::Bool => true,
        ProfileTypeShape::ExactI32Array { .. } => {
            matches!(
                profile,
                LanguageProfile::ExactI32ArrayV0
                    | LanguageProfile::ExactI32RecordResultV0
                    | LanguageProfile::ExactI32ByteBufferV0
                    | LanguageProfile::ExactI32ByteInputV0
            ) && usage != ProfileTypeUse::OwnedAssignment
        }
        ProfileTypeShape::Unsupported => false,
    }
}

trait ProfileTypeView: Sized {
    fn scalar_shape(&self) -> Option<ProfileTypeShape>;
    fn array_parts(&self) -> Option<(&Self, usize)>;
}

impl ProfileTypeView for Type {
    fn scalar_shape(&self) -> Option<ProfileTypeShape> {
        match self {
            Type::Named(name) if matches!(name.as_str(), "int" | "i32") => {
                Some(ProfileTypeShape::Int)
            }
            Type::Named(name) if name == "bool" => Some(ProfileTypeShape::Bool),
            _ => None,
        }
    }

    fn array_parts(&self) -> Option<(&Self, usize)> {
        match self {
            Type::Array(element, count) => Some((element, *count)),
            _ => None,
        }
    }
}

impl ProfileTypeView for LogicalType {
    fn scalar_shape(&self) -> Option<ProfileTypeShape> {
        match self {
            LogicalType::Int => Some(ProfileTypeShape::Int),
            LogicalType::Bool => Some(ProfileTypeShape::Bool),
            _ => None,
        }
    }

    fn array_parts(&self) -> Option<(&Self, usize)> {
        match self {
            LogicalType::Array { element, count } => Some((element, *count)),
            _ => None,
        }
    }
}

fn classify_profile_type<T: ProfileTypeView>(ty: &T) -> ProfileTypeShape {
    if let Some(scalar) = ty.scalar_shape() {
        return scalar;
    }
    if let Some((element, count)) = ty.array_parts()
        && (1..=i32::MAX as usize).contains(&count)
        && element.scalar_shape() == Some(ProfileTypeShape::Int)
    {
        return ProfileTypeShape::ExactI32Array { count };
    }
    ProfileTypeShape::Unsupported
}

fn classify_profile_ast_type(ty: &Type) -> ProfileTypeShape {
    classify_profile_type(ty)
}

pub(crate) fn classify_profile_logical_type(ty: &LogicalType) -> ProfileTypeShape {
    classify_profile_type(ty)
}

pub(crate) fn validate_language_profile(
    ast: &[AstNode],
    profile: LanguageProfile,
) -> Result<(), String> {
    match profile {
        LanguageProfile::Experimental
        | LanguageProfile::ExactI32RecordResultV0
        | LanguageProfile::ExactI32ByteBufferV0
        | LanguageProfile::ExactI32ByteInputV0 => Ok(()),
        LanguageProfile::StableScalarV0 | LanguageProfile::ExactI32ArrayV0 => {
            ProfileValidator::validate(ast, profile)
        }
    }
}

/// Applies the compiler-oriented profile only after semantic normalization has
/// produced the single resolved shape and surface authority. Existing profiles
/// retain their accepted pre-semantic validator and never enter this path.
pub(crate) fn validate_resolved_language_profile(
    program: &ResolvedProfileProgram,
    profile: LanguageProfile,
) -> Result<(), String> {
    match profile {
        LanguageProfile::ExactI32RecordResultV0 => {
            ExactRecordResultProfileValidator::validate(program)
        }
        LanguageProfile::ExactI32ByteBufferV0 => ExactByteBufferProfileValidator::validate(
            program,
            LanguageProfile::ExactI32ByteBufferV0,
        ),
        LanguageProfile::ExactI32ByteInputV0 => {
            ExactByteInputProfileValidator::validate(program)?;
            ExactByteBufferProfileValidator::validate(program, LanguageProfile::ExactI32ByteInputV0)
        }
        _ => Ok(()),
    }
}

struct ExactByteInputProfileValidator;

impl ExactByteInputProfileValidator {
    fn validate(program: &ResolvedProfileProgram) -> Result<(), String> {
        for observation in &program.surface {
            let ResolvedProfileSurfaceObservation::Expression {
                context,
                kind: ResolvedProfileExpressionKind::FunctionCall { name, arguments },
            } = observation
            else {
                continue;
            };
            if !is_reserved_byte_input_intrinsic(name) {
                continue;
            }
            if !matches!(
                context,
                ResolvedProfileSurfaceContext::Function(ResolvedProfileOrigin::Source { .. })
            ) {
                return Err(profile_named_error(
                    LanguageProfile::ExactI32ByteInputV0,
                    "byte-input intrinsic outside a direct source function",
                ));
            }
            if !arguments.is_empty() {
                return Err(profile_named_error(
                    LanguageProfile::ExactI32ByteInputV0,
                    &format!("byte-input intrinsic `{STDIN_READ_BYTE}` argument topology"),
                ));
            }
        }
        Ok(())
    }
}

struct ExactByteBufferProfileValidator;

impl ExactByteBufferProfileValidator {
    fn validate(program: &ResolvedProfileProgram, profile: LanguageProfile) -> Result<(), String> {
        let byte_shapes = Self::validate_shapes_and_uses(program, profile)?;
        let (expected_immutable_borrows, expected_mutable_borrows) =
            Self::validate_intrinsic_surface(program, profile)?;
        Self::validate_borrow_surface(
            program,
            expected_immutable_borrows,
            expected_mutable_borrows,
            profile,
        )?;

        let mut sanitized = program.clone();
        sanitized.uses.retain(|usage| {
            !Self::resolution_shape_id(&usage.resolution)
                .is_some_and(|id| byte_shapes.contains(&id))
        });
        sanitized.surface.retain(|observation| {
            !matches!(
                observation,
                ResolvedProfileSurfaceObservation::Expression {
                    kind: ResolvedProfileExpressionKind::Borrow { .. },
                    ..
                }
            )
        });
        ExactRecordResultProfileValidator::validate(&sanitized)
            .map_err(|error| error.replacen(EXACT_I32_RECORD_RESULT_V0_NAME, profile.as_str(), 1))
    }

    fn reject(profile: LanguageProfile, feature: impl AsRef<str>) -> String {
        profile_named_error(profile, feature.as_ref())
    }

    fn resolution_shape_id(
        resolution: &ResolvedProfileResolution,
    ) -> Option<crate::resolved_profile_shape::ResolvedProfileShapeId> {
        match resolution {
            ResolvedProfileResolution::Resolved(id)
            | ResolvedProfileResolution::Excluded(Some(id)) => Some(*id),
            ResolvedProfileResolution::Excluded(None) | ResolvedProfileResolution::Unresolved => {
                None
            }
        }
    }

    fn contains_byte_buffer(logical: &LogicalType) -> bool {
        match logical {
            LogicalType::ByteBuffer => true,
            LogicalType::ImmutableReference { pointee }
            | LogicalType::MutableReference { pointee }
            | LogicalType::Array {
                element: pointee, ..
            } => Self::contains_byte_buffer(pointee),
            LogicalType::Struct { fields, .. }
            | LogicalType::Tuple { elements: fields }
            | LogicalType::EnumFields { fields } => fields.iter().any(Self::contains_byte_buffer),
            LogicalType::Enum { variants, .. } => variants.iter().any(|variant| {
                variant
                    .payload
                    .as_ref()
                    .is_some_and(Self::contains_byte_buffer)
            }),
            LogicalType::Int
            | LogicalType::Float
            | LogicalType::Bool
            | LogicalType::Char
            | LogicalType::Void
            | LogicalType::String => false,
        }
    }

    fn validate_shapes_and_uses(
        program: &ResolvedProfileProgram,
        profile: LanguageProfile,
    ) -> Result<BTreeSet<crate::resolved_profile_shape::ResolvedProfileShapeId>, String> {
        let mut byte_shapes = BTreeSet::new();
        for (index, logical) in program.shapes.iter().enumerate() {
            if logical == &LogicalType::ByteBuffer {
                byte_shapes.insert(crate::resolved_profile_shape::ResolvedProfileShapeId(index));
            } else if Self::contains_byte_buffer(logical) {
                return Err(Self::reject(
                    profile,
                    format!("nested ByteBuffer logical type `{logical}`"),
                ));
            }
        }
        for usage in &program.uses {
            let Some(id) = Self::resolution_shape_id(&usage.resolution) else {
                continue;
            };
            if !byte_shapes.contains(&id) {
                continue;
            }
            if !matches!(usage.resolution, ResolvedProfileResolution::Resolved(_))
                || !matches!(usage.function, Some(ResolvedProfileOrigin::Source { .. }))
                || !matches!(
                    usage.role,
                    ProfileTypeUse::Binding
                        | ProfileTypeUse::MutableBinding
                        | ProfileTypeUse::Value
                )
            {
                return Err(Self::reject(
                    profile,
                    format!(
                        "{:?} ByteBuffer use outside a direct source function",
                        usage.role
                    ),
                ));
            }
        }
        Ok(byte_shapes)
    }

    fn validate_intrinsic_surface(
        program: &ResolvedProfileProgram,
        profile: LanguageProfile,
    ) -> Result<(usize, usize), String> {
        let mut immutable_borrows = 0_usize;
        let mut mutable_borrows = 0_usize;
        for observation in &program.surface {
            let ResolvedProfileSurfaceObservation::Expression {
                context,
                kind: ResolvedProfileExpressionKind::FunctionCall { name, arguments },
            } = observation
            else {
                continue;
            };
            if !is_reserved_byte_buffer_intrinsic(name) {
                continue;
            }
            if !matches!(
                context,
                ResolvedProfileSurfaceContext::Function(ResolvedProfileOrigin::Source { .. })
            ) {
                return Err(Self::reject(
                    profile,
                    format!("byte-buffer intrinsic `{name}` outside a source function"),
                ));
            }
            let expected = match name.as_str() {
                BYTES_NEW => &[][..],
                BYTES_PUSH => &[
                    ResolvedProfileCallArgumentKind::MutableBorrowIdentifier,
                    ResolvedProfileCallArgumentKind::Other,
                ][..],
                BYTES_LENGTH | BYTES_CAPACITY => {
                    &[ResolvedProfileCallArgumentKind::ImmutableBorrowIdentifier][..]
                }
                BYTES_GET => &[
                    ResolvedProfileCallArgumentKind::ImmutableBorrowIdentifier,
                    ResolvedProfileCallArgumentKind::Other,
                ][..],
                _ => unreachable!("reserved byte-buffer intrinsic is closed"),
            };
            if arguments.as_slice() != expected {
                return Err(Self::reject(
                    profile,
                    format!("byte-buffer intrinsic `{name}` argument topology"),
                ));
            }
            immutable_borrows += arguments
                .iter()
                .filter(|argument| {
                    **argument == ResolvedProfileCallArgumentKind::ImmutableBorrowIdentifier
                })
                .count();
            mutable_borrows += arguments
                .iter()
                .filter(|argument| {
                    **argument == ResolvedProfileCallArgumentKind::MutableBorrowIdentifier
                })
                .count();
        }
        Ok((immutable_borrows, mutable_borrows))
    }

    fn validate_borrow_surface(
        program: &ResolvedProfileProgram,
        expected_immutable: usize,
        expected_mutable: usize,
        profile: LanguageProfile,
    ) -> Result<(), String> {
        let mut immutable = 0_usize;
        let mut mutable = 0_usize;
        for observation in &program.surface {
            let ResolvedProfileSurfaceObservation::Expression {
                context,
                kind:
                    ResolvedProfileExpressionKind::Borrow {
                        mutable: is_mutable,
                    },
            } = observation
            else {
                continue;
            };
            if !matches!(
                context,
                ResolvedProfileSurfaceContext::Function(ResolvedProfileOrigin::Source { .. })
            ) {
                return Err(Self::reject(profile, "borrow outside a source function"));
            }
            if *is_mutable {
                mutable += 1;
            } else {
                immutable += 1;
            }
        }
        if immutable != expected_immutable || mutable != expected_mutable {
            return Err(Self::reject(
                profile,
                "borrow expression outside an immediate byte-buffer intrinsic argument",
            ));
        }
        Ok(())
    }
}

/// Shared logical policy used by post-semantic admission and the authenticated
/// backend guard. Source-origin eligibility is checked separately against the
/// resolved descriptor; this function owns only the closed logical topology.
pub(crate) fn exact_record_result_logical_type_is_admitted(logical: &LogicalType) -> bool {
    match logical {
        LogicalType::Enum { name, variants } => {
            private_carrier_source_name(name).is_some_and(|source| {
                source.starts_with("Result<")
                    && matches!(
                        variants.as_slice(),
                        [crate::ir::EnumVariantSchema {
                            name: ok,
                            payload: Some(ok_payload),
                        }, crate::ir::EnumVariantSchema {
                            name: error,
                            payload: Some(error_payload),
                        }] if ok == "Ok"
                            && error == "Err"
                            && exact_record_result_non_enum_shape(ok_payload, None)
                            && exact_record_result_non_enum_shape(error_payload, None)
                    )
            })
        }
        _ => exact_record_result_non_enum_shape(logical, None),
    }
}

fn exact_record_result_non_enum_shape(
    logical: &LogicalType,
    source_structs: Option<&BTreeSet<String>>,
) -> bool {
    match logical {
        LogicalType::Int | LogicalType::Bool => true,
        LogicalType::Array { element, count } => {
            (1..=i32::MAX as usize).contains(count) && **element == LogicalType::Int
        }
        LogicalType::Struct { name, fields } => {
            !fields.is_empty()
                && source_structs.is_none_or(|names| names.contains(name))
                && fields
                    .iter()
                    .all(|field| exact_record_result_non_enum_shape(field, source_structs))
        }
        LogicalType::Float
        | LogicalType::Char
        | LogicalType::Void
        | LogicalType::String
        | LogicalType::ByteBuffer
        | LogicalType::ImmutableReference { .. }
        | LogicalType::MutableReference { .. }
        | LogicalType::Tuple { .. }
        | LogicalType::EnumFields { .. }
        | LogicalType::Enum { .. } => false,
    }
}

struct ExactRecordResultProfileValidator<'a> {
    program: &'a ResolvedProfileProgram,
    source_structs: BTreeSet<String>,
    result_carriers: BTreeSet<String>,
}

impl<'a> ExactRecordResultProfileValidator<'a> {
    fn validate(program: &'a ResolvedProfileProgram) -> Result<(), String> {
        let mut validator = Self {
            program,
            source_structs: BTreeSet::new(),
            result_carriers: BTreeSet::new(),
        };
        validator.collect_nominal_identities()?;
        validator.validate_surface()?;
        validator.validate_nominals()?;
        validator.validate_uses()?;
        validator.validate_operations()
    }

    fn reject(feature: impl AsRef<str>) -> String {
        profile_named_error(LanguageProfile::ExactI32RecordResultV0, feature.as_ref())
    }

    fn collect_nominal_identities(&mut self) -> Result<(), String> {
        let mut identities = BTreeSet::new();
        for nominal in &self.program.nominals {
            let (normalized, eligible) = match nominal {
                ResolvedProfileNominal::Struct {
                    origin: ResolvedProfileOrigin::Source { normalized },
                    ..
                } => (normalized, Some(true)),
                ResolvedProfileNominal::Enum {
                    origin: ResolvedProfileOrigin::BuiltinCarrier { normalized, source },
                    ..
                } if source.starts_with("Result<")
                    && private_carrier_source_name(normalized).as_deref() == Some(source) =>
                {
                    (normalized, Some(false))
                }
                ResolvedProfileNominal::Struct { origin, .. }
                | ResolvedProfileNominal::Enum { origin, .. } => {
                    let normalized = Self::origin_label(origin);
                    if !identities.insert(normalized.clone()) {
                        return Err(Self::reject(format!(
                            "ambiguous nominal identity `{normalized}`"
                        )));
                    }
                    continue;
                }
            };
            if !identities.insert(normalized.clone()) {
                return Err(Self::reject(format!(
                    "ambiguous nominal identity `{normalized}`"
                )));
            }
            match eligible {
                Some(true) => {
                    self.source_structs.insert(normalized.clone());
                }
                Some(false) => {
                    self.result_carriers.insert(normalized.clone());
                }
                None => unreachable!("eligible nominal identities are classified above"),
            }
        }
        Ok(())
    }

    fn validate_surface(&self) -> Result<(), String> {
        for observation in &self.program.surface {
            match observation {
                ResolvedProfileSurfaceObservation::Statement { context, kind } => {
                    self.validate_statement_surface(context, kind)?
                }
                ResolvedProfileSurfaceObservation::Expression { context, kind } => {
                    self.require_source_function_context(context, "expression")?;
                    if !matches!(
                        kind,
                        ResolvedProfileExpressionKind::IntegerLiteral
                            | ResolvedProfileExpressionKind::Identifier
                            | ResolvedProfileExpressionKind::Binary(
                                ResolvedProfileBinaryOperator::Add
                                    | ResolvedProfileBinaryOperator::Subtract
                                    | ResolvedProfileBinaryOperator::Multiply
                            )
                            | ResolvedProfileExpressionKind::FunctionCall { .. }
                            | ResolvedProfileExpressionKind::Comparison(_)
                            | ResolvedProfileExpressionKind::Logical(_)
                            | ResolvedProfileExpressionKind::Unary(_)
                            | ResolvedProfileExpressionKind::ArrayLiteral
                            | ResolvedProfileExpressionKind::IndexAccess
                            | ResolvedProfileExpressionKind::FieldAccess
                            | ResolvedProfileExpressionKind::StructLiteral
                            | ResolvedProfileExpressionKind::EnumVariant {
                                parenthesized: true
                            }
                            | ResolvedProfileExpressionKind::Match
                    ) {
                        return Err(Self::reject(format!("surface expression `{kind:?}`")));
                    }
                }
                ResolvedProfileSurfaceObservation::Pattern { context, kind } => {
                    self.require_source_function_context(context, "Match pattern")?;
                    if !matches!(
                        kind,
                        ResolvedProfilePatternKind::Identifier
                            | ResolvedProfilePatternKind::Enum {
                                parenthesized: true
                            }
                    ) {
                        return Err(Self::reject(format!("Match pattern `{kind:?}`")));
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_statement_surface(
        &self,
        context: &ResolvedProfileSurfaceContext,
        kind: &ResolvedProfileStatementKind,
    ) -> Result<(), String> {
        match context {
            ResolvedProfileSurfaceContext::FileScope => {
                if matches!(
                    kind,
                    ResolvedProfileStatementKind::Function {
                        top_level: true,
                        generic: false,
                        trait_bounded: false,
                        ..
                    } | ResolvedProfileStatementKind::StructDefinition { generic: false }
                        | ResolvedProfileStatementKind::EnumDefinition {
                            generic: false,
                            trait_bounded: false,
                        }
                ) {
                    Ok(())
                } else {
                    Err(Self::reject(format!("file-scope statement `{kind:?}`")))
                }
            }
            ResolvedProfileSurfaceContext::Function(origin) => {
                if !matches!(origin, ResolvedProfileOrigin::Source { .. }) {
                    return Err(Self::reject(format!(
                        "non-source function context `{}`",
                        Self::origin_label(origin)
                    )));
                }
                let allowed = match kind {
                    ResolvedProfileStatementKind::Let {
                        annotated: true,
                        initialized: true,
                        ..
                    }
                    | ResolvedProfileStatementKind::Return { .. }
                    | ResolvedProfileStatementKind::Expression
                    | ResolvedProfileStatementKind::Block
                    | ResolvedProfileStatementKind::If { .. }
                    | ResolvedProfileStatementKind::While
                    | ResolvedProfileStatementKind::Loop
                    | ResolvedProfileStatementKind::Break
                    | ResolvedProfileStatementKind::Continue => true,
                    ResolvedProfileStatementKind::Assignment { target } => {
                        target.root == ResolvedProfileAssignmentRoot::Identifier
                            && (target.projections.is_empty()
                                || target.projections.as_slice()
                                    == [ResolvedProfileAssignmentProjection::Index])
                    }
                    _ => false,
                };
                if allowed {
                    Ok(())
                } else {
                    Err(Self::reject(format!("surface statement `{kind:?}`")))
                }
            }
        }
    }

    fn require_source_function_context(
        &self,
        context: &ResolvedProfileSurfaceContext,
        feature: &str,
    ) -> Result<(), String> {
        match context {
            ResolvedProfileSurfaceContext::Function(ResolvedProfileOrigin::Source { .. }) => Ok(()),
            ResolvedProfileSurfaceContext::FileScope => {
                Err(Self::reject(format!("file-scope {feature}")))
            }
            ResolvedProfileSurfaceContext::Function(origin) => Err(Self::reject(format!(
                "{feature} in non-source function context `{}`",
                Self::origin_label(origin)
            ))),
        }
    }

    fn validate_nominals(&self) -> Result<(), String> {
        for nominal in &self.program.nominals {
            match nominal {
                ResolvedProfileNominal::Struct {
                    origin: ResolvedProfileOrigin::Source { normalized },
                    resolution,
                    fields,
                } => {
                    let logical = self.resolved_shape(resolution, "record declaration")?;
                    let LogicalType::Struct {
                        name,
                        fields: logical_fields,
                    } = logical
                    else {
                        return Err(Self::reject(format!(
                            "record `{normalized}` has a non-record resolved shape"
                        )));
                    };
                    if name != normalized
                        || fields.is_empty()
                        || fields.len() != logical_fields.len()
                        || !exact_record_result_non_enum_shape(logical, Some(&self.source_structs))
                    {
                        return Err(Self::reject(format!("record `{normalized}` schema")));
                    }
                    let mut field_names = BTreeSet::new();
                    for (field, logical_field) in fields.iter().zip(logical_fields) {
                        if !field_names.insert(field.name.as_str())
                            || self.resolved_shape(&field.resolution, "record field")?
                                != logical_field
                        {
                            return Err(Self::reject(format!(
                                "record `{normalized}` field schema"
                            )));
                        }
                    }
                }
                ResolvedProfileNominal::Enum {
                    origin: ResolvedProfileOrigin::BuiltinCarrier { normalized, source },
                    resolution,
                    variants,
                } if source.starts_with("Result<") => {
                    let logical = self.resolved_shape(resolution, "Result declaration")?;
                    let LogicalType::Enum {
                        name,
                        variants: logical_variants,
                    } = logical
                    else {
                        return Err(Self::reject("Result declaration has non-enum shape"));
                    };
                    if name != normalized
                        || private_carrier_source_name(name).as_deref() != Some(source)
                        || !self.result_carriers.contains(name)
                        || !exact_record_result_logical_type_is_admitted(logical)
                        || variants.len() != 2
                        || variants.len() != logical_variants.len()
                    {
                        return Err(Self::reject(format!("Result `{source}` schema")));
                    }
                    for (variant, logical_variant) in variants.iter().zip(logical_variants) {
                        let Some(payload) = &variant.payload else {
                            return Err(Self::reject(format!("Result `{source}` unit variant")));
                        };
                        let Some(logical_payload) = &logical_variant.payload else {
                            return Err(Self::reject(format!("Result `{source}` unit schema")));
                        };
                        if variant.name != logical_variant.name
                            || self.resolved_shape(payload, "Result payload")? != logical_payload
                        {
                            return Err(Self::reject(format!("Result `{source}` payload schema")));
                        }
                    }
                }
                ResolvedProfileNominal::Struct { origin, .. }
                | ResolvedProfileNominal::Enum { origin, .. } => {
                    return Err(Self::reject(format!(
                        "nominal origin `{}`",
                        Self::origin_label(origin)
                    )));
                }
            }
        }
        Ok(())
    }

    fn validate_uses(&self) -> Result<(), String> {
        for usage in &self.program.uses {
            if let Some(origin) = &usage.function
                && !matches!(origin, ResolvedProfileOrigin::Source { .. })
            {
                return Err(Self::reject(format!(
                    "typed use in non-source function `{}`",
                    Self::origin_label(origin)
                )));
            }
            let logical = self.resolved_shape(&usage.resolution, "typed use")?;
            let admitted = if logical == &LogicalType::Void {
                usage.role == ProfileTypeUse::Result
            } else if usage.role == ProfileTypeUse::OwnedAssignment {
                matches!(logical, LogicalType::Int | LogicalType::Bool)
                    || matches!(logical, LogicalType::Enum { .. }) && self.admitted_shape(logical)
            } else {
                self.admitted_shape(logical)
            };
            if !admitted {
                return Err(Self::reject(format!(
                    "{:?} logical type `{logical}`",
                    usage.role
                )));
            }
        }
        Ok(())
    }

    fn validate_operations(&self) -> Result<(), String> {
        for operation in &self.program.operations {
            match operation {
                ResolvedProfileOperation::Declaration { origin, resolution } => {
                    let logical = self.resolved_shape(resolution, "nominal declaration")?;
                    if !self.origin_matches_logical(origin, logical) {
                        return Err(Self::reject(format!(
                            "declaration origin `{}`",
                            Self::origin_label(origin)
                        )));
                    }
                }
                ResolvedProfileOperation::StructConstruction {
                    function,
                    origin: ResolvedProfileOrigin::Source { normalized },
                    resolution,
                    source_to_declaration,
                } => {
                    Self::require_source_operation_function(function, "record construction")?;
                    let logical = self.resolved_shape(resolution, "record construction")?;
                    let LogicalType::Struct { name, fields } = logical else {
                        return Err(Self::reject("record construction shape"));
                    };
                    if name != normalized
                        || !self.source_structs.contains(name)
                        || !Self::complete_permutation(source_to_declaration, fields.len())
                    {
                        return Err(Self::reject(format!(
                            "record `{normalized}` construction mapping"
                        )));
                    }
                }
                ResolvedProfileOperation::EnumConstruction {
                    function,
                    origin: ResolvedProfileOrigin::BuiltinCarrier { normalized, source },
                    variant,
                    resolution,
                    variant_index,
                } if source.starts_with("Result<") => {
                    Self::require_source_operation_function(function, "Result construction")?;
                    let logical = self.resolved_shape(resolution, "Result construction")?;
                    let LogicalType::Enum { name, variants } = logical else {
                        return Err(Self::reject("Result construction shape"));
                    };
                    let expected_index = variants
                        .iter()
                        .position(|candidate| candidate.name == *variant);
                    if name != normalized
                        || !self.result_carriers.contains(name)
                        || expected_index != *variant_index
                        || !matches!(variant.as_str(), "Ok" | "Err")
                    {
                        return Err(Self::reject(format!(
                            "Result `{source}` constructor `{variant}`"
                        )));
                    }
                }
                ResolvedProfileOperation::ExhaustiveMatch {
                    function,
                    origin: Some(ResolvedProfileOrigin::BuiltinCarrier { normalized, source }),
                    resolution,
                    arm_for_variant,
                    result,
                } if source.starts_with("Result<") => {
                    Self::require_source_operation_function(function, "Result Match")?;
                    let logical = self.resolved_shape(resolution, "Result Match")?;
                    let LogicalType::Enum { name, variants } = logical else {
                        return Err(Self::reject("Result Match shape"));
                    };
                    if name != normalized
                        || !self.result_carriers.contains(name)
                        || variants.len() != 2
                        || !Self::complete_permutation(arm_for_variant, variants.len())
                    {
                        return Err(Self::reject(format!("Result `{source}` Match mapping")));
                    }
                    if let Some(result) = result {
                        let result = self.resolved_shape(result, "Match result")?;
                        if result != &LogicalType::Void && !self.admitted_shape(result) {
                            return Err(Self::reject(format!(
                                "Match result logical type `{result}`"
                            )));
                        }
                    }
                }
                ResolvedProfileOperation::StructConstruction { origin, .. }
                | ResolvedProfileOperation::EnumConstruction { origin, .. } => {
                    return Err(Self::reject(format!(
                        "construction origin `{}`",
                        Self::origin_label(origin)
                    )));
                }
                ResolvedProfileOperation::ExhaustiveMatch { origin, .. } => {
                    return Err(Self::reject(format!("Match origin `{origin:?}`")));
                }
            }
        }
        Ok(())
    }

    fn require_source_operation_function(
        function: &Option<ResolvedProfileOrigin>,
        feature: &str,
    ) -> Result<(), String> {
        if matches!(function, Some(ResolvedProfileOrigin::Source { .. })) {
            Ok(())
        } else {
            Err(Self::reject(format!(
                "{feature} outside an admitted source function"
            )))
        }
    }

    fn resolved_shape(
        &self,
        resolution: &ResolvedProfileResolution,
        context: &str,
    ) -> Result<&LogicalType, String> {
        let ResolvedProfileResolution::Resolved(id) = resolution else {
            return Err(Self::reject(format!("unavailable {context}")));
        };
        self.program
            .shapes
            .get(id.0)
            .ok_or_else(|| Self::reject(format!("invalid {context} shape identity")))
    }

    fn admitted_shape(&self, logical: &LogicalType) -> bool {
        match logical {
            LogicalType::Enum { name, .. } => {
                self.result_carriers.contains(name)
                    && exact_record_result_logical_type_is_admitted(logical)
            }
            _ => exact_record_result_non_enum_shape(logical, Some(&self.source_structs)),
        }
    }

    fn origin_matches_logical(
        &self,
        origin: &ResolvedProfileOrigin,
        logical: &LogicalType,
    ) -> bool {
        match (origin, logical) {
            (ResolvedProfileOrigin::Source { normalized }, LogicalType::Struct { name, .. }) => {
                normalized == name && self.source_structs.contains(name)
            }
            (
                ResolvedProfileOrigin::BuiltinCarrier { normalized, source },
                LogicalType::Enum { name, .. },
            ) => {
                normalized == name
                    && source.starts_with("Result<")
                    && self.result_carriers.contains(name)
                    && self.admitted_shape(logical)
            }
            _ => false,
        }
    }

    fn complete_permutation(mapping: &[usize], count: usize) -> bool {
        mapping.len() == count
            && mapping.iter().copied().collect::<BTreeSet<_>>()
                == (0..count).collect::<BTreeSet<_>>()
    }

    fn origin_label(origin: &ResolvedProfileOrigin) -> String {
        match origin {
            ResolvedProfileOrigin::Source { normalized }
            | ResolvedProfileOrigin::SourceGenericStruct { normalized }
            | ResolvedProfileOrigin::SourceGenericEnum { normalized }
            | ResolvedProfileOrigin::SourceGenericFunction { normalized }
            | ResolvedProfileOrigin::OpaquePrivate { normalized } => normalized.clone(),
            ResolvedProfileOrigin::GenericStruct { source, .. }
            | ResolvedProfileOrigin::GenericEnum { source, .. }
            | ResolvedProfileOrigin::GenericFunction { source, .. }
            | ResolvedProfileOrigin::BuiltinCarrier { source, .. } => source.clone(),
            ResolvedProfileOrigin::ImplMethod {
                type_name,
                trait_name,
                method,
            } => format!(
                "impl {}{}::{method}",
                trait_name
                    .as_ref()
                    .map(|name| format!("{name} for "))
                    .unwrap_or_default(),
                type_name
            ),
            ResolvedProfileOrigin::TraitMethod { trait_name, method } => {
                format!("trait {trait_name}::{method}")
            }
        }
    }
}

fn profile_error(feature: &str) -> String {
    format!("Language Profile Error: {STABLE_SCALAR_V0_NAME} rejects {feature}")
}

fn profile_named_error(profile: LanguageProfile, feature: &str) -> String {
    if profile == LanguageProfile::StableScalarV0 {
        profile_error(feature)
    } else {
        format!(
            "Language Profile Error: {} rejects {feature}",
            profile.as_str()
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProfileValueShape {
    Known(ProfileTypeShape),
    UnknownScalar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProfileBindingOrigin {
    Parameter,
    Local,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProfileBindingFact {
    shape: ProfileValueShape,
    mutable: bool,
    origin: ProfileBindingOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProfileValueContext {
    General,
    ExactArrayElement,
}

impl ProfileValueShape {
    fn known(shape: ProfileTypeShape) -> Self {
        Self::Known(shape)
    }

    fn exact_array(self) -> Option<usize> {
        match self {
            Self::Known(ProfileTypeShape::ExactI32Array { count }) => Some(count),
            Self::Known(ProfileTypeShape::Int | ProfileTypeShape::Bool)
            | Self::Known(ProfileTypeShape::Unsupported)
            | Self::UnknownScalar => None,
        }
    }
}

struct ProfileValidator {
    profile: LanguageProfile,
    functions: BTreeSet<String>,
    function_parameter_shapes: BTreeMap<String, Vec<ProfileTypeShape>>,
    function_result_shapes: BTreeMap<String, ProfileTypeShape>,
    calls: BTreeMap<String, BTreeSet<String>>,
    binding_scopes: Vec<BTreeMap<String, ProfileBindingFact>>,
}

impl ProfileValidator {
    fn validate(ast: &[AstNode], profile: LanguageProfile) -> Result<(), String> {
        let mut validator = Self {
            profile,
            functions: BTreeSet::new(),
            function_parameter_shapes: BTreeMap::new(),
            function_result_shapes: BTreeMap::new(),
            calls: BTreeMap::new(),
            binding_scopes: Vec::new(),
        };
        validator.collect_function_headers(ast)?;
        validator.validate_functions(ast)?;
        validator.reject_call_cycles()
    }

    fn error(&self, feature: &str) -> String {
        profile_named_error(self.profile, feature)
    }

    fn collect_function_headers(&mut self, ast: &[AstNode]) -> Result<(), String> {
        for node in ast {
            match node {
                AstNode::Statement(Statement::Function {
                    name,
                    parameters,
                    return_type,
                    ..
                }) => {
                    if !self.functions.insert(name.clone()) {
                        return Err(
                            self.error(&format!("duplicate function definitions (`{name}`)"))
                        );
                    }
                    self.calls.entry(name.clone()).or_default();
                    self.function_parameter_shapes.insert(
                        name.clone(),
                        parameters
                            .iter()
                            .map(|parameter| classify_profile_ast_type(&parameter.param_type))
                            .collect(),
                    );
                    self.function_result_shapes.insert(
                        name.clone(),
                        return_type
                            .as_ref()
                            .map(classify_profile_ast_type)
                            .unwrap_or(ProfileTypeShape::Unsupported),
                    );
                }
                AstNode::Statement(statement) => {
                    return Err(self.error(top_level_statement_feature(statement)));
                }
                AstNode::Expression(_) => {
                    return Err(self.error("top-level expressions"));
                }
            }
        }

        if !self.functions.contains("main") {
            return Err(self.error("programs without `fn main() -> int`"));
        }
        Ok(())
    }

    fn validate_functions(&mut self, ast: &[AstNode]) -> Result<(), String> {
        for node in ast {
            let AstNode::Statement(Statement::Function {
                name,
                parameters,
                return_type,
                body,
                type_params,
                trait_bounds,
            }) = node
            else {
                unreachable!("profile header collection admitted only functions")
            };

            if !type_params.is_empty() || !trait_bounds.is_empty() {
                return Err(self.error("generic functions or trait bounds"));
            }
            for parameter in parameters {
                self.validate_type(
                    &parameter.param_type,
                    ProfileTypeUse::Parameter,
                    "function parameter types",
                )?;
            }
            if let Some(return_type) = return_type {
                self.validate_type(return_type, ProfileTypeUse::Result, "function result types")?;
            }
            if name == "main"
                && (!parameters.is_empty()
                    || !matches!(return_type, Some(Type::Named(result)) if result == "int"))
            {
                return Err(self.error("entrypoints other than exact `fn main() -> int`"));
            }

            let parameter_scope = parameters
                .iter()
                .map(|parameter| {
                    (
                        parameter.name.clone(),
                        ProfileBindingFact {
                            shape: ProfileValueShape::known(classify_profile_ast_type(
                                &parameter.param_type,
                            )),
                            mutable: false,
                            origin: ProfileBindingOrigin::Parameter,
                        },
                    )
                })
                .collect();
            self.binding_scopes.push(parameter_scope);
            self.validate_block(name, body)?;
            self.binding_scopes.pop();
        }
        Ok(())
    }

    fn validate_block(&mut self, function: &str, block: &Block) -> Result<(), String> {
        self.binding_scopes.push(BTreeMap::new());
        let result = (|| {
            for statement in &block.statements {
                self.validate_statement(function, statement)?;
            }
            if block.expression.is_some() {
                return Err(self.error("implicit tail expressions"));
            }
            Ok(())
        })();
        self.binding_scopes.pop();
        result
    }

    fn validate_statement(&mut self, function: &str, statement: &Statement) -> Result<(), String> {
        match statement {
            Statement::Let {
                name,
                mutable,
                type_annotation,
                value,
            } => {
                let annotation_shape = if let Some(annotation) = type_annotation {
                    self.validate_type(
                        annotation,
                        if *mutable {
                            ProfileTypeUse::MutableBinding
                        } else {
                            ProfileTypeUse::Binding
                        },
                        "binding annotation types",
                    )?;
                    Some(classify_profile_ast_type(annotation))
                } else {
                    None
                };
                let Some(value) = value else {
                    return Err(self.error("uninitialized bindings"));
                };
                let value_shape = self.classify_value(function, value)?;
                let stored_shape = match annotation_shape {
                    Some(expected @ ProfileTypeShape::ExactI32Array { count }) => {
                        self.require_exact_array_shape(
                            value_shape,
                            count,
                            "array literal counts that differ from their annotations",
                        )?;
                        if *mutable {
                            self.validate_mutable_array_initializer(value)?;
                        }
                        ProfileValueShape::known(expected)
                    }
                    Some(expected @ (ProfileTypeShape::Int | ProfileTypeShape::Bool)) => {
                        if value_shape.exact_array().is_some() {
                            return Err(self.error("array values in scalar bindings"));
                        }
                        ProfileValueShape::known(expected)
                    }
                    Some(ProfileTypeShape::Unsupported) => {
                        unreachable!("validated binding annotations have admitted profile types")
                    }
                    None => {
                        if value_shape.exact_array().is_some() && *mutable {
                            self.validate_mutable_array_initializer(value)?;
                        }
                        value_shape
                    }
                };
                if self.profile != LanguageProfile::ExactI32ArrayV0
                    && stored_shape.exact_array().is_some()
                {
                    return Err(self.error("array expressions"));
                }
                self.binding_scopes
                    .last_mut()
                    .expect("validated function body retains a binding scope")
                    .insert(
                        name.clone(),
                        ProfileBindingFact {
                            shape: stored_shape,
                            mutable: *mutable,
                            origin: ProfileBindingOrigin::Local,
                        },
                    );
                Ok(())
            }
            Statement::Assignment { target, value } => {
                if let Expression::IndexAccess { object, index } = target {
                    if self.profile != LanguageProfile::ExactI32ArrayV0 {
                        return Err(self.error("projected or indirect assignment targets"));
                    }
                    let Expression::Identifier(target_name) = object.as_ref() else {
                        return Err(self.error("projected or indirect assignment targets"));
                    };
                    let Some(target_fact) = self.binding_fact(target_name) else {
                        return Err(
                            self.error("projected assignment targets rooted in unproved bindings")
                        );
                    };
                    if target_fact.shape.exact_array().is_none() {
                        return Err(
                            self.error("projected assignment targets rooted in non-array bindings")
                        );
                    }
                    if !target_fact.mutable || target_fact.origin != ProfileBindingOrigin::Local {
                        return Err(self.error(
                            "projected assignment targets rooted in immutable exact-array bindings",
                        ));
                    }
                    self.require_int_value(function, index, ProfileValueContext::General)?;
                    self.require_int_value(function, value, ProfileValueContext::General)?;
                    return Ok(());
                }

                let Expression::Identifier(target_name) = target else {
                    return Err(self.error("projected or indirect assignment targets"));
                };
                if self
                    .binding_shape(target_name)
                    .and_then(ProfileValueShape::exact_array)
                    .is_some()
                {
                    return Err(self.error("array writes"));
                }
                let value_shape = self.classify_value(function, value)?;
                if value_shape.exact_array().is_some() {
                    return Err(self.error("array values in scalar assignments"));
                }
                Ok(())
            }
            Statement::Return(value) => {
                if let Some(value) = value {
                    let expected = self
                        .function_result_shapes
                        .get(function)
                        .copied()
                        .unwrap_or(ProfileTypeShape::Unsupported);
                    let actual = self.classify_value(function, value)?;
                    if let ProfileTypeShape::ExactI32Array { count } = expected {
                        self.require_exact_array_shape(
                            actual,
                            count,
                            "array value source count mismatch",
                        )?;
                    } else if actual.exact_array().is_some() {
                        return Err(self.error("array values returned from scalar functions"));
                    }
                }
                Ok(())
            }
            Statement::Expression(Expression::FunctionCall { name, arguments }) => {
                let result = self.classify_call(function, name, arguments)?;
                if result.exact_array().is_some() {
                    return Err(self.error("effect-free array-result calls"));
                }
                Ok(())
            }
            Statement::Expression(expression) => {
                self.validate_expression(function, expression)?;
                Err(self.error("effect-free or non-call expression statements"))
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.validate_expression(function, condition)?;
                self.validate_block(function, then_block)?;
                if let Some(else_statement) = else_block {
                    match else_statement.as_ref() {
                        Statement::Block(block) => self.validate_block(function, block)?,
                        nested @ Statement::If { .. } => {
                            self.validate_statement(function, nested)?
                        }
                        other => return Err(self.error(statement_feature(other))),
                    }
                }
                Ok(())
            }
            Statement::While { condition, body } => {
                self.validate_expression(function, condition)?;
                self.validate_block(function, body)
            }
            Statement::Const { .. }
            | Statement::Block(_)
            | Statement::Function { .. }
            | Statement::For { .. }
            | Statement::Loop { .. }
            | Statement::Break
            | Statement::Continue
            | Statement::StructDef { .. }
            | Statement::EnumDef { .. }
            | Statement::ImplBlock { .. }
            | Statement::TraitDef { .. }
            | Statement::ModDecl { .. }
            | Statement::UseImport { .. } => Err(self.error(statement_feature(statement))),
        }
    }

    fn validate_expression(
        &mut self,
        function: &str,
        expression: &Expression,
    ) -> Result<(), String> {
        let shape = self.classify_value(function, expression)?;
        if shape.exact_array().is_some() {
            Err(self.error("array identifiers outside direct call transport or index reads"))
        } else {
            Ok(())
        }
    }

    fn classify_value(
        &mut self,
        function: &str,
        expression: &Expression,
    ) -> Result<ProfileValueShape, String> {
        if self.profile == LanguageProfile::StableScalarV0 {
            return self.classify_stable_scalar_value(function, expression);
        }
        self.classify_exact_value(function, expression, ProfileValueContext::General)
    }

    /// Preserve the accepted stable-scalar-v0 source policy byte-for-behavior.
    ///
    /// CAP-018 needs stronger value identity only for exact-array composition.
    /// Stable scalar programs continue to defer operand type equality to the
    /// semantic analyzer, exactly as they did before the array-value classifier.
    fn classify_stable_scalar_value(
        &mut self,
        function: &str,
        expression: &Expression,
    ) -> Result<ProfileValueShape, String> {
        match expression {
            Expression::IntegerLiteral(value) => i32::try_from(*value)
                .map(|_| ProfileValueShape::UnknownScalar)
                .map_err(|_| self.error("integer literals outside the signed i32 range")),
            Expression::Identifier(_) => Ok(ProfileValueShape::UnknownScalar),
            Expression::Binary {
                op, left, right, ..
            } => {
                match op {
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply => {}
                    BinaryOp::Divide => return Err(self.error("division expressions")),
                    BinaryOp::Modulo => return Err(self.error("remainder expressions")),
                }
                self.classify_stable_scalar_value(function, left)?;
                self.classify_stable_scalar_value(function, right)?;
                Ok(ProfileValueShape::UnknownScalar)
            }
            Expression::FunctionCall { name, arguments } => {
                self.classify_call(function, name, arguments)
            }
            Expression::Comparison { left, right, .. } => {
                self.classify_stable_scalar_value(function, left)?;
                self.classify_stable_scalar_value(function, right)?;
                Ok(ProfileValueShape::UnknownScalar)
            }
            Expression::Logical { left, right, .. } => {
                if expression_contains_call(left) || expression_contains_call(right) {
                    return Err(self.error("function calls inside logical operands"));
                }
                self.classify_stable_scalar_value(function, left)?;
                self.classify_stable_scalar_value(function, right)?;
                Ok(ProfileValueShape::UnknownScalar)
            }
            Expression::Unary { operand, .. } => {
                self.classify_stable_scalar_value(function, operand)?;
                Ok(ProfileValueShape::UnknownScalar)
            }
            Expression::IndexAccess { .. } => Err(self.error(expression_feature(expression))),
            Expression::FloatLiteral(_)
            | Expression::CharacterLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::MethodCall { .. }
            | Expression::Print { .. }
            | Expression::Println { .. }
            | Expression::ArrayLiteral(_)
            | Expression::ArrayRepeat { .. }
            | Expression::FieldAccess { .. }
            | Expression::TupleLiteral(_)
            | Expression::TupleIndex { .. }
            | Expression::StructLiteral { .. }
            | Expression::EnumVariant { .. }
            | Expression::Match { .. }
            | Expression::Borrow { .. }
            | Expression::Deref(_)
            | Expression::Closure { .. } => Err(self.error(expression_feature(expression))),
        }
    }

    fn classify_exact_value(
        &mut self,
        function: &str,
        expression: &Expression,
        context: ProfileValueContext,
    ) -> Result<ProfileValueShape, String> {
        match expression {
            Expression::IntegerLiteral(value) => i32::try_from(*value)
                .map(|_| ProfileValueShape::known(ProfileTypeShape::Int))
                .map_err(|_| self.error("integer literals outside the signed i32 range")),
            Expression::Identifier(name) => Ok(self
                .binding_shape(name)
                .unwrap_or(ProfileValueShape::UnknownScalar)),
            Expression::Binary {
                op, left, right, ..
            } => {
                match op {
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply => {}
                    BinaryOp::Divide => return Err(self.error("division expressions")),
                    BinaryOp::Modulo => return Err(self.error("remainder expressions")),
                }
                self.require_int_value(function, left, context)?;
                self.require_int_value(function, right, context)?;
                Ok(ProfileValueShape::known(ProfileTypeShape::Int))
            }
            Expression::FunctionCall { name, arguments } => {
                self.classify_call(function, name, arguments)
            }
            Expression::Comparison { left, right, .. } => {
                self.require_scalar_value(function, left)?;
                self.require_scalar_value(function, right)?;
                Ok(ProfileValueShape::known(ProfileTypeShape::Bool))
            }
            Expression::Logical { left, right, .. } => {
                if expression_contains_call(left) || expression_contains_call(right) {
                    return Err(self.error("function calls inside logical operands"));
                }
                self.require_bool_value(function, left)?;
                self.require_bool_value(function, right)?;
                Ok(ProfileValueShape::known(ProfileTypeShape::Bool))
            }
            Expression::Unary {
                op: UnaryOp::Negate,
                operand,
            } => {
                if context == ProfileValueContext::ExactArrayElement
                    && let Expression::IntegerLiteral(value) = operand.as_ref()
                    && (0..=i64::from(i32::MAX) + 1).contains(value)
                {
                    return Ok(ProfileValueShape::known(ProfileTypeShape::Int));
                }
                self.require_int_value(function, operand, context)?;
                Ok(ProfileValueShape::known(ProfileTypeShape::Int))
            }
            Expression::Unary {
                op: UnaryOp::Not,
                operand,
            } => {
                self.require_bool_value(function, operand)?;
                Ok(ProfileValueShape::known(ProfileTypeShape::Bool))
            }
            Expression::ArrayLiteral(elements) => {
                if self.profile != LanguageProfile::ExactI32ArrayV0 {
                    return Err(self.error("array expressions"));
                }
                let count = elements.len();
                if !(1..=i32::MAX as usize).contains(&count) {
                    return Err(self.error("array value source count outside the profile boundary"));
                }
                for element in elements {
                    let shape = self
                        .classify_exact_value(
                            function,
                            element,
                            ProfileValueContext::ExactArrayElement,
                        )
                        .map_err(|error| {
                            if error == self.error("integer literals outside the signed i32 range")
                            {
                                self.error("array elements other than exact signed i32 literals")
                            } else {
                                error
                            }
                        })?;
                    if shape != ProfileValueShape::known(ProfileTypeShape::Int) {
                        return Err(
                            self.error("array literal elements other than exact Int expressions")
                        );
                    }
                }
                Ok(ProfileValueShape::known(ProfileTypeShape::ExactI32Array {
                    count,
                }))
            }
            Expression::IndexAccess { object, index } => {
                if self.profile != LanguageProfile::ExactI32ArrayV0 {
                    return Err(self.error(expression_feature(expression)));
                }
                let object_shape = self.classify_value(function, object)?;
                if object_shape.exact_array().is_none() {
                    return Err(self.error("index reads from non-array values"));
                }
                self.require_int_value(function, index, ProfileValueContext::General)?;
                Ok(ProfileValueShape::known(ProfileTypeShape::Int))
            }
            Expression::ArrayRepeat { .. } if self.profile == LanguageProfile::ExactI32ArrayV0 => {
                Err(self.error("array bindings without direct literal initializers"))
            }
            Expression::FloatLiteral(_)
            | Expression::CharacterLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::MethodCall { .. }
            | Expression::Print { .. }
            | Expression::Println { .. }
            | Expression::ArrayRepeat { .. }
            | Expression::FieldAccess { .. }
            | Expression::TupleLiteral(_)
            | Expression::TupleIndex { .. }
            | Expression::StructLiteral { .. }
            | Expression::EnumVariant { .. }
            | Expression::Match { .. }
            | Expression::Borrow { .. }
            | Expression::Deref(_)
            | Expression::Closure { .. } => Err(self.error(expression_feature(expression))),
        }
    }

    fn classify_call(
        &mut self,
        function: &str,
        callee: &str,
        arguments: &[Expression],
    ) -> Result<ProfileValueShape, String> {
        let parameter_shapes = self
            .function_parameter_shapes
            .get(callee)
            .cloned()
            .unwrap_or_default();
        for (index, argument) in arguments.iter().enumerate() {
            match parameter_shapes.get(index) {
                Some(ProfileTypeShape::ExactI32Array { count })
                    if self.profile == LanguageProfile::ExactI32ArrayV0 =>
                {
                    let actual = self.classify_value(function, argument)?;
                    self.require_exact_array_shape(
                        actual,
                        *count,
                        "array call arguments with mismatched counts",
                    )?;
                }
                _ => self.validate_expression(function, argument)?,
            }
        }
        if self.functions.contains(callee) {
            self.calls
                .get_mut(function)
                .expect("validated function retains a call-graph node")
                .insert(callee.to_string());
        }
        let result = self
            .function_result_shapes
            .get(callee)
            .copied()
            .filter(|shape| *shape != ProfileTypeShape::Unsupported)
            .map(ProfileValueShape::known)
            .unwrap_or(ProfileValueShape::UnknownScalar);
        if self.profile == LanguageProfile::StableScalarV0 {
            Ok(ProfileValueShape::UnknownScalar)
        } else {
            Ok(result)
        }
    }

    fn require_scalar_value(
        &mut self,
        function: &str,
        expression: &Expression,
    ) -> Result<ProfileValueShape, String> {
        let shape = self.classify_value(function, expression)?;
        if shape.exact_array().is_some() {
            Err(self.error("array identifiers outside direct call transport or index reads"))
        } else {
            Ok(shape)
        }
    }

    fn require_int_value(
        &mut self,
        function: &str,
        expression: &Expression,
        context: ProfileValueContext,
    ) -> Result<(), String> {
        match self.classify_exact_value(function, expression, context)? {
            ProfileValueShape::Known(ProfileTypeShape::Int) => Ok(()),
            ProfileValueShape::Known(ProfileTypeShape::Bool) => {
                Err(self.error("non-Int values in exact integer expressions"))
            }
            ProfileValueShape::Known(ProfileTypeShape::ExactI32Array { .. }) => {
                Err(self.error("array values in exact integer expressions"))
            }
            ProfileValueShape::Known(ProfileTypeShape::Unsupported) => {
                Err(self.error("unsupported values in exact integer expressions"))
            }
            ProfileValueShape::UnknownScalar => {
                Err(self.error("unproved values in exact integer expressions"))
            }
        }
    }

    fn require_bool_value(
        &mut self,
        function: &str,
        expression: &Expression,
    ) -> Result<(), String> {
        match self.classify_exact_value(function, expression, ProfileValueContext::General)? {
            ProfileValueShape::Known(ProfileTypeShape::Bool) => Ok(()),
            ProfileValueShape::Known(ProfileTypeShape::Int) => {
                Err(self.error("non-Bool values in logical expressions"))
            }
            ProfileValueShape::Known(ProfileTypeShape::ExactI32Array { .. }) => {
                Err(self.error("array values in logical expressions"))
            }
            ProfileValueShape::Known(ProfileTypeShape::Unsupported) => {
                Err(self.error("unsupported values in logical expressions"))
            }
            ProfileValueShape::UnknownScalar => {
                Err(self.error("unproved values in logical expressions"))
            }
        }
    }

    fn require_exact_array_shape(
        &self,
        actual: ProfileValueShape,
        expected_count: usize,
        mismatch_feature: &str,
    ) -> Result<(), String> {
        match actual {
            ProfileValueShape::Known(ProfileTypeShape::ExactI32Array { count })
                if count == expected_count =>
            {
                Ok(())
            }
            ProfileValueShape::Known(ProfileTypeShape::ExactI32Array { .. }) => {
                Err(self.error(mismatch_feature))
            }
            ProfileValueShape::Known(ProfileTypeShape::Int | ProfileTypeShape::Bool)
            | ProfileValueShape::Known(ProfileTypeShape::Unsupported)
            | ProfileValueShape::UnknownScalar => {
                Err(self.error("array value source has non-array type"))
            }
        }
    }

    fn validate_mutable_array_initializer(&self, value: &Expression) -> Result<(), String> {
        match value {
            Expression::ArrayLiteral(_) | Expression::FunctionCall { .. } => Ok(()),
            Expression::Identifier(name) => match self.binding_fact(name) {
                Some(ProfileBindingFact {
                    shape: ProfileValueShape::Known(ProfileTypeShape::ExactI32Array { .. }),
                    mutable: false,
                    ..
                }) => Ok(()),
                Some(ProfileBindingFact {
                    shape: ProfileValueShape::Known(ProfileTypeShape::ExactI32Array { .. }),
                    mutable: true,
                    ..
                }) => Err(self.error("mutable exact-array values as initializer sources")),
                _ => Err(self.error("mutable array bindings with unproved initializer sources")),
            },
            _ => Err(self.error(
                "mutable array bindings without literal, immutable identifier, or function-call initializers",
            )),
        }
    }

    fn reject_call_cycles(&self) -> Result<(), String> {
        fn visit(
            name: &str,
            calls: &BTreeMap<String, BTreeSet<String>>,
            visiting: &mut BTreeSet<String>,
            visited: &mut BTreeSet<String>,
        ) -> bool {
            if visited.contains(name) {
                return false;
            }
            if !visiting.insert(name.to_string()) {
                return true;
            }
            if calls.get(name).is_some_and(|callees| {
                callees
                    .iter()
                    .any(|callee| visit(callee, calls, visiting, visited))
            }) {
                return true;
            }
            visiting.remove(name);
            visited.insert(name.to_string());
            false
        }

        let mut visiting = BTreeSet::new();
        let mut visited = BTreeSet::new();
        if self
            .calls
            .keys()
            .any(|name| visit(name, &self.calls, &mut visiting, &mut visited))
        {
            return Err(self.error("recursive function call cycles"));
        }
        Ok(())
    }

    fn validate_type(&self, ty: &Type, usage: ProfileTypeUse, context: &str) -> Result<(), String> {
        let stable_scalar =
            matches!(ty, Type::Named(name) if matches!(name.as_str(), "int" | "bool"));
        let exact_array = matches!(
            classify_profile_ast_type(ty),
            shape @ ProfileTypeShape::ExactI32Array { .. }
                if profile_type_shape_is_admitted(self.profile, shape, usage)
        );
        if stable_scalar || exact_array {
            Ok(())
        } else {
            Err(self.error(context))
        }
    }

    fn binding_shape(&self, name: &str) -> Option<ProfileValueShape> {
        self.binding_fact(name).map(|fact| fact.shape)
    }

    fn binding_fact(&self, name: &str) -> Option<ProfileBindingFact> {
        self.binding_scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }
}

fn expression_contains_call(expression: &Expression) -> bool {
    match expression {
        Expression::FunctionCall { .. } => true,
        Expression::Binary { left, right, .. }
        | Expression::Comparison { left, right, .. }
        | Expression::Logical { left, right, .. } => {
            expression_contains_call(left) || expression_contains_call(right)
        }
        Expression::Unary { operand, .. }
        | Expression::FieldAccess {
            object: operand, ..
        }
        | Expression::TupleIndex {
            object: operand, ..
        }
        | Expression::Borrow { expr: operand, .. }
        | Expression::Deref(operand)
        | Expression::Closure { body: operand, .. }
        | Expression::ArrayRepeat { value: operand, .. } => expression_contains_call(operand),
        Expression::MethodCall {
            object, arguments, ..
        } => expression_contains_call(object) || arguments.iter().any(expression_contains_call),
        Expression::Print { arguments, .. }
        | Expression::Println { arguments, .. }
        | Expression::ArrayLiteral(arguments)
        | Expression::TupleLiteral(arguments) => arguments.iter().any(expression_contains_call),
        Expression::IndexAccess { object, index } => {
            expression_contains_call(object) || expression_contains_call(index)
        }
        Expression::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, expression)| expression_contains_call(expression)),
        Expression::EnumVariant { data, .. } => data
            .as_ref()
            .is_some_and(|fields| fields.iter().any(expression_contains_call)),
        Expression::Match { expr, arms } => {
            expression_contains_call(expr)
                || arms.iter().any(|arm| expression_contains_call(&arm.body))
        }
        Expression::IntegerLiteral(_)
        | Expression::FloatLiteral(_)
        | Expression::CharacterLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::Identifier(_) => false,
    }
}

fn top_level_statement_feature(statement: &Statement) -> &'static str {
    match statement {
        Statement::Function { .. } => unreachable!("functions are admitted at top level"),
        Statement::StructDef { .. } => "struct definitions",
        Statement::EnumDef { .. } => "enum definitions",
        Statement::TraitDef { .. } => "trait definitions",
        Statement::ImplBlock { .. } => "impl blocks",
        Statement::ModDecl { .. } => "module declarations",
        Statement::UseImport { .. } => "import declarations",
        Statement::Const { .. } => "top-level constants",
        Statement::Let { .. }
        | Statement::Assignment { .. }
        | Statement::Return(_)
        | Statement::Expression(_)
        | Statement::Block(_)
        | Statement::If { .. }
        | Statement::While { .. }
        | Statement::For { .. }
        | Statement::Loop { .. }
        | Statement::Break
        | Statement::Continue => "top-level executable statements",
    }
}

fn statement_feature(statement: &Statement) -> &'static str {
    match statement {
        Statement::Const { .. } => "constant declarations",
        Statement::Let { .. } => "unsupported bindings",
        Statement::Assignment { .. } => "unsupported assignments",
        Statement::Return(_) => "unsupported returns",
        Statement::Expression(_) => "unsupported expression statements",
        Statement::Block(_) => "unsupported blocks",
        Statement::Function { .. } => "nested functions",
        Statement::If { .. } => "unsupported conditionals",
        Statement::While { .. } => "unsupported while loops",
        Statement::For { .. } => "for loops",
        Statement::Loop { .. } => "unconditional loop statements",
        Statement::Break => "break statements",
        Statement::Continue => "continue statements",
        Statement::StructDef { .. } => "struct definitions",
        Statement::EnumDef { .. } => "enum definitions",
        Statement::ImplBlock { .. } => "impl blocks",
        Statement::TraitDef { .. } => "trait definitions",
        Statement::ModDecl { .. } => "module declarations",
        Statement::UseImport { .. } => "import declarations",
    }
}

fn expression_feature(expression: &Expression) -> &'static str {
    match expression {
        Expression::IntegerLiteral(_) => "unsupported integer literals",
        Expression::FloatLiteral(_) => "float literals",
        Expression::CharacterLiteral(_) => "character literals",
        Expression::StringLiteral(_) => "String literals",
        Expression::Identifier(_) => "unsupported identifiers",
        Expression::Binary { .. } => "unsupported binary expressions",
        Expression::FunctionCall { .. } => "unsupported function calls",
        Expression::MethodCall { .. } => "method calls",
        Expression::Print { .. } | Expression::Println { .. } => "formatting/output intrinsics",
        Expression::Comparison { .. } => "unsupported comparisons",
        Expression::Logical { .. } => "unsupported logical expressions",
        Expression::Unary { .. } => "unsupported unary expressions",
        Expression::ArrayLiteral(_) | Expression::ArrayRepeat { .. } => "array expressions",
        Expression::IndexAccess { .. } => "index expressions",
        Expression::FieldAccess { .. } => "field-access expressions",
        Expression::TupleLiteral(_) | Expression::TupleIndex { .. } => "tuple expressions",
        Expression::StructLiteral { .. } => "struct value construction",
        Expression::EnumVariant { .. } => "enum value construction",
        Expression::Match { .. } => "Match expressions",
        Expression::Borrow { .. } | Expression::Deref(_) => "reference expressions",
        Expression::Closure { .. } => "closure expressions",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse_with_locations, try_tokenize_with_locations};

    fn parsed(source: &str) -> Vec<AstNode> {
        let tokens = try_tokenize_with_locations(source, None).expect("source should lex");
        parse_with_locations(tokens).expect("source should parse")
    }

    #[test]
    fn parses_only_the_five_named_profiles() {
        assert_eq!(
            "experimental".parse::<LanguageProfile>(),
            Ok(LanguageProfile::Experimental)
        );
        assert_eq!(
            STABLE_SCALAR_V0_NAME.parse::<LanguageProfile>(),
            Ok(LanguageProfile::StableScalarV0)
        );
        assert_eq!(
            EXACT_I32_ARRAY_V0_NAME.parse::<LanguageProfile>(),
            Ok(LanguageProfile::ExactI32ArrayV0)
        );
        assert_eq!(
            EXACT_I32_RECORD_RESULT_V0_NAME.parse::<LanguageProfile>(),
            Ok(LanguageProfile::ExactI32RecordResultV0)
        );
        assert_eq!(
            EXACT_I32_BYTE_BUFFER_V0_NAME.parse::<LanguageProfile>(),
            Ok(LanguageProfile::ExactI32ByteBufferV0)
        );
        assert_eq!(
            LanguageProfile::ExactI32ArrayV0.to_string(),
            EXACT_I32_ARRAY_V0_NAME
        );
        assert!("stable".parse::<LanguageProfile>().is_err());
    }

    #[test]
    fn shared_profile_type_classifier_owns_the_complete_exact_array_shape() {
        let ast_cases = [
            (Type::Named("int".to_string()), ProfileTypeShape::Int),
            (Type::Named("i32".to_string()), ProfileTypeShape::Int),
            (Type::Named("bool".to_string()), ProfileTypeShape::Bool),
            (
                Type::Array(Box::new(Type::Named("int".to_string())), 1),
                ProfileTypeShape::ExactI32Array { count: 1 },
            ),
            (
                Type::Array(
                    Box::new(Type::Named("i32".to_string())),
                    i32::MAX as usize + 1,
                ),
                ProfileTypeShape::Unsupported,
            ),
            (
                Type::Array(Box::new(Type::Named("int".to_string())), 0),
                ProfileTypeShape::Unsupported,
            ),
            (
                Type::Array(Box::new(Type::Named("bool".to_string())), 1),
                ProfileTypeShape::Unsupported,
            ),
            (
                Type::Array(
                    Box::new(Type::Array(Box::new(Type::Named("int".to_string())), 1)),
                    1,
                ),
                ProfileTypeShape::Unsupported,
            ),
            (
                Type::Tuple(vec![Type::Named("int".to_string())]),
                ProfileTypeShape::Unsupported,
            ),
            (
                Type::Reference(Box::new(Type::Named("int".to_string())), false),
                ProfileTypeShape::Unsupported,
            ),
            (
                Type::Generic("Box".to_string(), vec![Type::Named("int".to_string())]),
                ProfileTypeShape::Unsupported,
            ),
        ];
        for (ty, expected) in ast_cases {
            assert_eq!(classify_profile_ast_type(&ty), expected, "AST type {ty:?}");
        }

        let logical_cases = [
            (LogicalType::Int, ProfileTypeShape::Int),
            (LogicalType::Bool, ProfileTypeShape::Bool),
            (
                LogicalType::Array {
                    element: Box::new(LogicalType::Int),
                    count: 8,
                },
                ProfileTypeShape::ExactI32Array { count: 8 },
            ),
            (
                LogicalType::Array {
                    element: Box::new(LogicalType::Int),
                    count: 0,
                },
                ProfileTypeShape::Unsupported,
            ),
            (
                LogicalType::Array {
                    element: Box::new(LogicalType::Int),
                    count: i32::MAX as usize + 1,
                },
                ProfileTypeShape::Unsupported,
            ),
            (
                LogicalType::Array {
                    element: Box::new(LogicalType::Float),
                    count: 8,
                },
                ProfileTypeShape::Unsupported,
            ),
            (
                LogicalType::Array {
                    element: Box::new(LogicalType::Array {
                        element: Box::new(LogicalType::Int),
                        count: 1,
                    }),
                    count: 1,
                },
                ProfileTypeShape::Unsupported,
            ),
            (LogicalType::Float, ProfileTypeShape::Unsupported),
            (LogicalType::Char, ProfileTypeShape::Unsupported),
            (LogicalType::Void, ProfileTypeShape::Unsupported),
            (LogicalType::String, ProfileTypeShape::Unsupported),
        ];
        for (ty, expected) in logical_cases {
            assert_eq!(
                classify_profile_logical_type(&ty),
                expected,
                "logical type {ty:?}"
            );
        }

        let exact = LogicalType::Array {
            element: Box::new(LogicalType::Int),
            count: 8,
        };
        assert!(LanguageProfile::StableScalarV0.uses_exact_i32_lane());
        assert!(LanguageProfile::ExactI32ArrayV0.uses_exact_i32_lane());
        assert!(LanguageProfile::ExactI32RecordResultV0.uses_exact_i32_lane());
        assert!(LanguageProfile::ExactI32ByteBufferV0.uses_exact_i32_lane());
        assert!(!LanguageProfile::Experimental.uses_exact_i32_lane());
        assert!(LanguageProfile::ExactI32ArrayV0.admits_exact_i32_array(&exact));
        assert!(LanguageProfile::ExactI32RecordResultV0.admits_exact_i32_array(&exact));
        assert!(LanguageProfile::ExactI32ByteBufferV0.admits_exact_i32_array(&exact));
        assert!(!LanguageProfile::StableScalarV0.admits_exact_i32_array(&exact));
        assert!(!LanguageProfile::Experimental.admits_exact_i32_array(&exact));
    }

    #[test]
    fn shared_profile_role_policy_owns_array_transport_and_results() {
        let exact_array = ProfileTypeShape::ExactI32Array { count: 8 };
        for usage in [
            ProfileTypeUse::Parameter,
            ProfileTypeUse::Result,
            ProfileTypeUse::Binding,
            ProfileTypeUse::MutableBinding,
            ProfileTypeUse::Value,
        ] {
            assert!(profile_type_shape_is_admitted(
                LanguageProfile::ExactI32ArrayV0,
                exact_array,
                usage
            ));
        }
        for profile in [
            LanguageProfile::Experimental,
            LanguageProfile::StableScalarV0,
        ] {
            for usage in [
                ProfileTypeUse::Parameter,
                ProfileTypeUse::Result,
                ProfileTypeUse::Binding,
                ProfileTypeUse::MutableBinding,
                ProfileTypeUse::OwnedAssignment,
                ProfileTypeUse::Value,
            ] {
                assert!(!profile_type_shape_is_admitted(profile, exact_array, usage));
            }
        }
        for usage in [
            ProfileTypeUse::Parameter,
            ProfileTypeUse::Result,
            ProfileTypeUse::Binding,
            ProfileTypeUse::MutableBinding,
            ProfileTypeUse::OwnedAssignment,
            ProfileTypeUse::Value,
        ] {
            assert!(profile_type_shape_is_admitted(
                LanguageProfile::ExactI32ArrayV0,
                ProfileTypeShape::Int,
                usage
            ));
            assert!(profile_type_shape_is_admitted(
                LanguageProfile::ExactI32ArrayV0,
                ProfileTypeShape::Bool,
                usage
            ));
            assert!(!profile_type_shape_is_admitted(
                LanguageProfile::ExactI32ArrayV0,
                ProfileTypeShape::Unsupported,
                usage
            ));
        }
        assert!(!profile_type_shape_is_admitted(
            LanguageProfile::ExactI32ArrayV0,
            exact_array,
            ProfileTypeUse::OwnedAssignment,
        ));
        assert!(profile_type_shape_is_admitted(
            LanguageProfile::ExactI32RecordResultV0,
            exact_array,
            ProfileTypeUse::Value,
        ));
        assert!(!profile_type_shape_is_admitted(
            LanguageProfile::ExactI32RecordResultV0,
            exact_array,
            ProfileTypeUse::OwnedAssignment,
        ));
    }

    #[test]
    fn exact_i32_array_validator_accepts_the_complete_flat_array_class() {
        for source in [
            "fn read(values: [int; 2], index: int) -> int { return values[index]; } fn main() -> int { let values: [int; 2] = [-2147483648, 2147483647]; return read(values, 1); }",
            "fn read(values: [i32; 1]) -> int { return values[0]; } fn main() -> int { let values: [i32; 1] = [-0]; return read(values); }",
            "fn main() -> int { let values: [int; 1] = [7]; let mut index: int = 0; while index < 1 { let value: int = values[index + 0]; index = index + 1; } return values[0]; }",
            "fn forward(values: [int; 1]) -> [i32; 1] { return later(values); } fn main() -> int { let literal = [1]; let computed: [i32; 1] = [literal[0] + 1]; let annotated: [int; 1] = computed; let copied = annotated; let called = forward(copied); let taken: int = take([called[0] * 2]); let literal_index: int = [7][0]; if taken == 4 && literal_index == 7 { return 0; } return 1; } fn later(values: [i32; 1]) -> [int; 1] { return values; } fn take(values: [int; 1]) -> int { return values[0]; }",
            "fn seed() -> [int; 2] { return [3, 4]; } fn literal() -> [int; 2] { let mut output: [i32; 2] = [1, 2]; output[0] = 5; return output; } fn copied(source: [int; 2]) -> [i32; 2] { let mut output = source; let mut index: int = 0; while index < 2 { output[index] = source[index] + 1; index = index + 1; } return output; } fn called() -> [int; 2] { let mut output = seed(); output[1] = 6; return output; } fn main() -> int { return literal()[0] + copied(seed())[0] + called()[1]; }",
        ] {
            validate_language_profile(&parsed(source), LanguageProfile::ExactI32ArrayV0)
                .unwrap_or_else(|error| panic!("exact flat-array source was rejected: {error}"));
        }
    }

    #[test]
    fn exact_i32_array_validator_rejects_every_neighboring_array_topology() {
        let rejected = [
            (
                "fn main() -> int { let values: [int; 2] = [1; 2]; return 0; }",
                "array bindings without direct literal initializers",
            ),
            (
                "fn main() -> int { let values: [int; 0] = []; return 0; }",
                "binding annotation types",
            ),
            (
                "fn take(values: [int; 2147483648]) -> int { return 0; } fn main() -> int { return 0; }",
                "function parameter types",
            ),
            (
                "fn main() -> int { let values: [[int; 1]; 1] = [[1]]; return 0; }",
                "binding annotation types",
            ),
            (
                "fn main() -> int { let values: [bool; 1] = [true]; return 0; }",
                "binding annotation types",
            ),
            (
                "fn main() -> int { let values: [float; 1] = [1.0]; return 0; }",
                "binding annotation types",
            ),
            (
                "fn main() -> int { let values: [int; 1] = [1, 2]; return 0; }",
                "array literal counts that differ from their annotations",
            ),
            (
                "fn main() -> int { let values: [int; 1] = [2147483648]; return 0; }",
                "array elements other than exact signed i32 literals",
            ),
            (
                "fn main() -> int { let values: [int; 1] = [-2147483649]; return 0; }",
                "array elements other than exact signed i32 literals",
            ),
            (
                "fn main() -> int { let values: [int; 1] = [1]; values = [2]; return 0; }",
                "array writes",
            ),
            (
                "fn main() -> int { let values: [int; 1] = [1]; values[0] = 2; return 0; }",
                "projected assignment targets rooted in immutable exact-array bindings",
            ),
            (
                "fn main() -> int { let mut source: [int; 1] = [1]; let mut values: [i32; 1] = source; return values[0]; }",
                "mutable exact-array values as initializer sources",
            ),
            (
                "fn main() -> int { let mut values: [int; 1] = [1]; values = [2]; return values[0]; }",
                "array writes",
            ),
            (
                "fn take(values: [int; 2]) -> int { return values[0]; } fn main() -> int { let values: [int; 1] = [1]; return take(values); }",
                "array call arguments with mismatched counts",
            ),
            (
                "fn main() -> int { let value: int = 1; return value[0]; }",
                "index reads from non-array values",
            ),
            (
                "fn main() -> int { let values: [int; 1] = [1]; return values[0][0]; }",
                "index reads from non-array values",
            ),
            (
                "fn main() -> int { let values: [int; 1] = [1]; values.len(); return 0; }",
                "method calls",
            ),
            (
                "fn main() -> int { let values: [int; 1] = [1]; for value in values { return value; } return 0; }",
                "for loops",
            ),
        ];

        for (source, feature) in rejected {
            assert_eq!(
                validate_language_profile(&parsed(source), LanguageProfile::ExactI32ArrayV0),
                Err(profile_named_error(
                    LanguageProfile::ExactI32ArrayV0,
                    feature
                )),
                "source should reject as `{feature}`: {source}"
            );
        }
    }

    #[test]
    fn exact_i32_array_value_classifier_never_manufactures_int_identity() {
        let rejected = [
            (
                "fn bad(flag: bool) -> [int; 1] { return [flag]; } fn main() -> int { return 0; }",
                "array literal elements other than exact Int expressions",
            ),
            (
                "fn flag() -> bool { return 1 < 2; } fn bad() -> [int; 1] { return [flag()]; } fn main() -> int { return 0; }",
                "array literal elements other than exact Int expressions",
            ),
            (
                "fn read(values: [int; 1], flag: bool) -> int { return values[flag]; } fn main() -> int { return 0; }",
                "non-Int values in exact integer expressions",
            ),
            (
                "fn bad() -> [int; 1] { return [missing + 1]; } fn main() -> int { return 0; }",
                "unproved values in exact integer expressions",
            ),
        ];

        for (source, feature) in rejected {
            assert_eq!(
                validate_language_profile(&parsed(source), LanguageProfile::ExactI32ArrayV0),
                Err(profile_named_error(
                    LanguageProfile::ExactI32ArrayV0,
                    feature
                )),
                "source must fail closed without manufacturing Int: {source}"
            );
        }
    }

    #[test]
    fn exact_i32_array_validator_inherits_the_scalar_profile_exclusions() {
        let rejected = [
            (
                "fn id<T>(value: T) -> T { return value; } fn main() -> int { return 0; }",
                "generic functions or trait bounds",
            ),
            (
                "fn helper(value: &int) -> int { return 0; } fn main() -> int { return 0; }",
                "function parameter types",
            ),
            (
                "fn helper(value: (int, int)) -> int { return 0; } fn main() -> int { return 0; }",
                "function parameter types",
            ),
            (
                "fn helper(value: float) -> int { return 0; } fn main() -> int { return 0; }",
                "function parameter types",
            ),
            (
                "const LIMIT: int = 1; fn main() -> int { return 0; }",
                "top-level constants",
            ),
            (
                "struct Value { item: int } fn main() -> int { return 0; }",
                "struct definitions",
            ),
            (
                "enum Value { One } fn main() -> int { return 0; }",
                "enum definitions",
            ),
            (
                "trait Read { fn read(value: int) -> int; } fn main() -> int { return 0; }",
                "trait definitions",
            ),
            (
                "mod helper; fn main() -> int { return 0; }",
                "module declarations",
            ),
            (
                "use helper; fn main() -> int { return 0; }",
                "import declarations",
            ),
            ("fn main() -> int { return 4 / 2; }", "division expressions"),
            (
                "fn main() -> int { print!(\"{}\", 1); return 0; }",
                "formatting/output intrinsics",
            ),
            (
                "fn main() -> int { let value: int = 1; let reference = &value; return 0; }",
                "reference expressions",
            ),
            (
                "fn main() -> int { let closure = |value: int| value; return 0; }",
                "closure expressions",
            ),
            (
                "fn recurse(value: int) -> int { return recurse(value); } fn main() -> int { return recurse(0); }",
                "recursive function call cycles",
            ),
            (
                "fn recurse(values: [int; 1]) -> [int; 1] { return recurse(values); } fn main() -> int { return 0; }",
                "recursive function call cycles",
            ),
            (
                "fn left(values: [int; 1]) -> [int; 1] { return right(values); } fn right(values: [int; 1]) -> [int; 1] { return left(values); } fn main() -> int { return 0; }",
                "recursive function call cycles",
            ),
        ];

        for (source, feature) in rejected {
            assert_eq!(
                validate_language_profile(&parsed(source), LanguageProfile::ExactI32ArrayV0),
                Err(profile_named_error(
                    LanguageProfile::ExactI32ArrayV0,
                    feature
                )),
                "source should retain inherited exclusion `{feature}`: {source}"
            );
        }
    }

    #[test]
    fn stable_scalar_array_rejection_remains_byte_for_behavior() {
        let source = "fn main() -> int { let values: [int; 1] = [1]; return values[0]; }";
        assert_eq!(
            validate_language_profile(&parsed(source), LanguageProfile::StableScalarV0),
            Err(profile_error("binding annotation types"))
        );
    }

    #[test]
    fn stable_scalar_expression_policy_remains_byte_for_behavior() {
        validate_language_profile(
            &parsed("fn main() -> int { return (1 < 2) + 1; }"),
            LanguageProfile::StableScalarV0,
        )
        .expect("stable profile must continue to defer scalar type equality to semantics");
        assert_eq!(
            validate_language_profile(
                &parsed("fn main() -> int { return -2147483648; }"),
                LanguageProfile::StableScalarV0,
            ),
            Err(profile_error(
                "integer literals outside the signed i32 range"
            ))
        );
    }

    #[test]
    fn stable_scalar_validator_accepts_the_frozen_control_product() {
        let ast = parsed(
            "fn step(value: int, ready: bool) -> int { if ready && !(value < 0) { return -value + 3 * 2; } else { return value - 1; } } fn main() -> int { let mut value: int = 2; let ready: bool = value < 3 || value == 2; while value < 11 { value = step(value, ready); } return value; }",
        );
        validate_language_profile(&ast, LanguageProfile::StableScalarV0)
            .expect("frozen scalar product should be admitted");
    }

    #[test]
    fn stable_scalar_validator_rejects_direct_and_mutual_recursion() {
        for source in [
            "fn again(value: int) -> int { return again(value); } fn main() -> int { return again(1); }",
            "fn left(value: int) -> int { return right(value); } fn right(value: int) -> int { return left(value); } fn main() -> int { return left(1); }",
        ] {
            assert_eq!(
                validate_language_profile(&parsed(source), LanguageProfile::StableScalarV0),
                Err(profile_error("recursive function call cycles"))
            );
        }
    }

    #[test]
    fn stable_scalar_validator_rejects_the_ast_only_top_level_expression_variant() {
        let mut ast = parsed("fn main() -> int { return 0; }");
        ast.insert(0, AstNode::Expression(Expression::IntegerLiteral(1)));
        assert_eq!(
            validate_language_profile(&ast, LanguageProfile::StableScalarV0),
            Err(profile_error("top-level expressions"))
        );
    }
}
