use crate::copy_data_layout::{CopyDataLayout, CopyDataLayoutPolicy, EnumStorageLayout};
use crate::ir::{
    CheckedIr, EnumSchema, Function, FunctionMetadata, Inst, IrMetadata, LogicalType, PlaceId,
    ResultId, Value,
};
use crate::ir_verifier::{IrVerificationError, verify_checked_ir};
use crate::language_profile::{
    LanguageProfile, ProfileTypeShape, ProfileTypeUse, classify_profile_logical_type,
    exact_record_result_logical_type_is_admitted, profile_type_shape_is_admitted,
    validate_resolved_language_profile,
};
use crate::primitive_contract::PrimitiveKind;
use crate::resolved_profile_authentication::{
    AuthenticatedResolvedProfileProgram, ResolvedProfileAuthenticationCoverage,
    ResolvedProfileAuthenticationObservation, ResolvedProfileAuthenticationSubject,
};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;

#[derive(Clone)]
enum FunctionDef {
    Legacy {
        parameters: Vec<(String, String)>,
        return_type: Option<String>,
        body: Vec<Inst>,
    },
    Checked {
        parameters: Vec<(String, LogicalType)>,
        result: LogicalType,
        body: Vec<Inst>,
    },
}

impl FunctionDef {
    fn body(&self) -> &[Inst] {
        match self {
            Self::Legacy { body, .. } | Self::Checked { body, .. } => body,
        }
    }
}

/// A failure at the checked LLVM-emission boundary.
#[derive(Debug)]
pub enum CodeGenerationError {
    /// The private raw IR failed mandatory verification before emission began.
    IrVerification(IrVerificationError),
    /// A verified program contained an instruction that this emitter does not admit.
    UnsupportedInstruction { instruction: &'static str },
    /// Verified logical metadata escaped the selected language profile's source class.
    LanguageProfileContract {
        profile: LanguageProfile,
        detail: String,
    },
}

impl fmt::Display for CodeGenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IrVerification(error) => error.fmt(formatter),
            Self::UnsupportedInstruction { instruction } => {
                write!(
                    formatter,
                    "unsupported instruction `{instruction}` in checked code generation"
                )
            }
            Self::LanguageProfileContract { profile, detail } => {
                write!(
                    formatter,
                    "language profile `{profile}` rejected checked IR: {detail}"
                )
            }
        }
    }
}

impl std::error::Error for CodeGenerationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IrVerification(error) => Some(error),
            Self::UnsupportedInstruction { .. } | Self::LanguageProfileContract { .. } => None,
        }
    }
}

impl From<IrVerificationError> for CodeGenerationError {
    fn from(error: IrVerificationError) -> Self {
        Self::IrVerification(error)
    }
}

pub struct CodeGenerator {
    next_reg: u64,
    next_ptr: u64,
    checked_metadata: Option<IrMetadata>,
    current_function: Option<String>,
    language_profile: LanguageProfile,
}

impl CodeGenerator {
    fn llvm_parameter_name(name: &str) -> String {
        format!("aero.arg.{name}")
    }

    fn llvm_function_symbol(name: &str) -> String {
        crate::copydata_trait_dispatch::private_trait_impl_llvm_symbol(name)
            .or_else(|| {
                crate::generic_function_contract::private_generic_function_llvm_symbol(name)
            })
            .unwrap_or_else(|| name.to_string())
    }

    fn struct_type_to_llvm(name: &str) -> String {
        match crate::generic_struct_contract::private_generic_struct_source_name(name) {
            Some(source_name) => format!("%\"aero.struct.{source_name}\""),
            None => format!("%aero.struct.{name}"),
        }
    }

    fn is_struct_llvm_type(llvm_type: &str) -> bool {
        llvm_type.starts_with("%aero.struct.") || llvm_type.starts_with("%\"aero.struct.")
    }

    fn logical_type_to_llvm(logical_type: &LogicalType) -> String {
        if let Some(primitive) = PrimitiveKind::from_logical_type(logical_type) {
            return primitive.scalar_llvm_type().to_string();
        }
        match logical_type {
            LogicalType::Int | LogicalType::Float | LogicalType::Bool | LogicalType::Char => {
                unreachable!("primitive logical types returned above")
            }
            LogicalType::Void => "void".to_string(),
            LogicalType::ImmutableReference { pointee }
            | LogicalType::MutableReference { pointee } => {
                format!("{}*", Self::reference_pointee_to_llvm(pointee))
            }
            LogicalType::Struct { name, .. } => Self::struct_type_to_llvm(name),
            LogicalType::Tuple { .. }
            | LogicalType::EnumFields { .. }
            | LogicalType::Array { .. } => Self::render_copy_data_layout(logical_type),
            LogicalType::Enum { name, variants } => Self::render_enum_storage_layout(&EnumSchema {
                name: name.clone(),
                variants: variants.clone(),
            }),
            LogicalType::String => {
                unreachable!("verified call signatures exclude String values")
            }
        }
    }

    fn render_copy_data_layout(logical_type: &LogicalType) -> String {
        CopyDataLayout::legacy(logical_type).llvm_type_with(&Self::struct_type_to_llvm)
    }

    /// Select the private physical CopyData representation paired with the checked
    /// program's source profile. The topology decision comes from language-profile
    /// authority; this backend owns only its physical rendering.
    fn profile_copy_data_type_to_llvm(&self, logical_type: &LogicalType) -> String {
        let policy = if self.language_profile == LanguageProfile::ExactI32RecordResultV0 {
            CopyDataLayoutPolicy::ExactI32
        } else {
            match classify_profile_logical_type(logical_type) {
                ProfileTypeShape::Int
                    if self.language_profile == LanguageProfile::ExactI32ArrayV0 =>
                {
                    CopyDataLayoutPolicy::ExactI32
                }
                ProfileTypeShape::ExactI32Array { .. }
                    if self.language_profile.admits_exact_i32_array(logical_type) =>
                {
                    CopyDataLayoutPolicy::ExactI32
                }
                _ => CopyDataLayoutPolicy::Legacy,
            }
        };
        CopyDataLayout::with_policy(logical_type, policy).llvm_type_with(&Self::struct_type_to_llvm)
    }

    fn profile_enum_storage_layout<'a>(&self, schema: &'a EnumSchema) -> EnumStorageLayout<'a> {
        if self.language_profile == LanguageProfile::ExactI32RecordResultV0 {
            EnumStorageLayout::with_policy(schema, CopyDataLayoutPolicy::ExactI32)
        } else {
            EnumStorageLayout::legacy(schema)
        }
    }

    /// Select the accepted physical storage for a verifier-authenticated
    /// checked place. Exact policy remains root-gated by profile authority;
    /// recursive aggregate leaves never select it through this helper.
    fn checked_place_copy_data_layout<'a>(
        &self,
        logical_type: &'a LogicalType,
    ) -> CopyDataLayout<'a> {
        let exact_root = self.language_profile == LanguageProfile::ExactI32RecordResultV0
            || self.language_profile.admits_exact_i32_array(logical_type)
            || self.uses_exact_i32_lane() && matches!(logical_type, LogicalType::Int);
        let policy = if exact_root {
            CopyDataLayoutPolicy::ExactI32
        } else {
            CopyDataLayoutPolicy::Legacy
        };
        CopyDataLayout::with_policy(logical_type, policy)
    }

    fn checked_place_storage(&self, logical_type: &LogicalType) -> (String, usize) {
        if let LogicalType::Enum { name, variants } = logical_type {
            let schema = EnumSchema {
                name: name.clone(),
                variants: variants.clone(),
            };
            return (
                self.profile_enum_storage_layout(&schema)
                    .enum_llvm_type_with(&Self::struct_type_to_llvm),
                8,
            );
        }
        let layout = self.checked_place_copy_data_layout(logical_type);
        let llvm_type = layout.llvm_type_with(&Self::struct_type_to_llvm);
        let alignment = if PrimitiveKind::from_logical_type(logical_type).is_some() {
            layout
                .alignment()
                .expect("primitive CopyData has a physical alignment")
        } else {
            // Preserve the accepted explicit aggregate alignment; CAP-026 does
            // not introduce or claim an aggregate ABI rule.
            8
        };
        (llvm_type, alignment)
    }

    fn profile_logical_type_to_llvm(&self, logical_type: &LogicalType) -> String {
        if self.language_profile == LanguageProfile::ExactI32RecordResultV0 {
            return match logical_type {
                LogicalType::Enum { name, variants } => self
                    .profile_enum_storage_layout(&EnumSchema {
                        name: name.clone(),
                        variants: variants.clone(),
                    })
                    .enum_llvm_type_with(&Self::struct_type_to_llvm),
                LogicalType::Int
                | LogicalType::Bool
                | LogicalType::Array { .. }
                | LogicalType::Struct { .. } => self.profile_copy_data_type_to_llvm(logical_type),
                _ => Self::logical_type_to_llvm(logical_type),
            };
        }
        if self.language_profile.admits_exact_i32_array(logical_type) {
            return self.profile_copy_data_type_to_llvm(logical_type);
        }
        Self::logical_type_to_llvm(logical_type)
    }

    fn render_enum_storage_layout(schema: &EnumSchema) -> String {
        EnumStorageLayout::legacy(schema).enum_llvm_type_with(&Self::struct_type_to_llvm)
    }

    fn reference_pointee_to_llvm(pointee: &LogicalType) -> String {
        match pointee {
            LogicalType::Enum { name, variants } => Self::render_enum_storage_layout(&EnumSchema {
                name: name.clone(),
                variants: variants.clone(),
            }),
            LogicalType::Int
            | LogicalType::Float
            | LogicalType::Bool
            | LogicalType::Char
            | LogicalType::Array { .. }
            | LogicalType::Struct { .. }
            | LogicalType::Tuple { .. }
            | LogicalType::EnumFields { .. } => Self::render_copy_data_layout(pointee),
            LogicalType::Void
            | LogicalType::String
            | LogicalType::ImmutableReference { .. }
            | LogicalType::MutableReference { .. } => Self::logical_type_to_llvm(pointee),
        }
    }

    fn collect_logical_struct_schema(
        logical_type: &LogicalType,
        schemas: &mut BTreeMap<String, Vec<LogicalType>>,
    ) {
        match logical_type {
            LogicalType::Array { element, .. }
            | LogicalType::ImmutableReference { pointee: element }
            | LogicalType::MutableReference { pointee: element } => {
                Self::collect_logical_struct_schema(element, schemas)
            }
            LogicalType::Struct { name, fields } => {
                if let Some(existing) = schemas.get(name) {
                    assert_eq!(existing, fields, "verified struct schema is stable");
                } else {
                    schemas.insert(name.clone(), fields.clone());
                }
                for field in fields {
                    Self::collect_logical_struct_schema(field, schemas);
                }
            }
            LogicalType::Tuple { elements } => {
                for element in elements {
                    Self::collect_logical_struct_schema(element, schemas);
                }
            }
            LogicalType::EnumFields { fields } => {
                for field in fields {
                    Self::collect_logical_struct_schema(field, schemas);
                }
            }
            LogicalType::Enum { variants, .. } => {
                for payload in variants
                    .iter()
                    .filter_map(|variant| variant.payload.as_ref())
                {
                    Self::collect_logical_struct_schema(payload, schemas);
                }
            }
            _ => {}
        }
    }

    fn collect_struct_schemas(
        instructions: &[Inst],
        schemas: &mut BTreeMap<String, Vec<LogicalType>>,
    ) {
        for instruction in instructions {
            match instruction {
                Inst::CheckedMutableOwnedPlaceAlloca { ty, .. }
                | Inst::CheckedOwnedPlaceAssignment { ty, .. } => {
                    Self::collect_logical_struct_schema(ty, schemas);
                }
                Inst::CheckedMatchResultPlaceAlloca { result_type, .. } => {
                    Self::collect_logical_struct_schema(result_type, schemas);
                }
                Inst::CheckedImmutableBorrow { pointee, .. }
                | Inst::CheckedMutableBorrow { pointee, .. }
                | Inst::CheckedMutableDereferenceAssignment { pointee, .. }
                | Inst::CheckedMutableBorrowEnd { pointee, .. }
                | Inst::CheckedImmutableReferenceParameter { pointee, .. }
                | Inst::CheckedMutableReferenceParameter { pointee, .. } => {
                    Self::collect_logical_struct_schema(pointee, schemas);
                }
                Inst::CheckedProjectedBorrow {
                    root_type, pointee, ..
                }
                | Inst::CheckedProjectedBorrowEnd {
                    root_type, pointee, ..
                } => {
                    Self::collect_logical_struct_schema(root_type, schemas);
                    Self::collect_logical_struct_schema(pointee, schemas);
                }
                Inst::CheckedMutableOwnerImmutableEnumBorrowEnd { schema, .. } => {
                    Self::collect_logical_struct_schema(&schema.logical_type(), schemas);
                }
                Inst::CheckedStructAlloca {
                    struct_name,
                    field_types,
                    ..
                } => {
                    Self::collect_logical_struct_schema(
                        &LogicalType::Struct {
                            name: struct_name.clone(),
                            fields: field_types.clone(),
                        },
                        schemas,
                    );
                }
                Inst::CheckedCopyStructArrayAlloca { element, .. } => {
                    Self::collect_logical_struct_schema(element, schemas);
                }
                Inst::CheckedCopyStructArrayElementPtr { element, .. }
                | Inst::CheckedStructFieldPtr {
                    field_type: element,
                    ..
                } => Self::collect_logical_struct_schema(element, schemas),
                Inst::CheckedTupleAlloca { element_types, .. }
                | Inst::CheckedTupleFieldPtr { element_types, .. } => {
                    for element in element_types {
                        Self::collect_logical_struct_schema(element, schemas);
                    }
                }
                Inst::FunctionDef { body, .. } => Self::collect_struct_schemas(body, schemas),
                Inst::CheckedFunctionDef {
                    parameters,
                    result,
                    body,
                    ..
                } => {
                    for (_, parameter) in parameters {
                        Self::collect_logical_struct_schema(parameter, schemas);
                    }
                    Self::collect_logical_struct_schema(result, schemas);
                    Self::collect_struct_schemas(body, schemas);
                }
                _ => {}
            }
        }
    }

    fn collect_metadata_struct_schemas(
        metadata: &IrMetadata,
        schemas: &mut BTreeMap<String, Vec<LogicalType>>,
    ) {
        for function in metadata.functions.values() {
            for (_, parameter) in &function.signature.parameters {
                Self::collect_logical_struct_schema(parameter, schemas);
            }
            Self::collect_logical_struct_schema(&function.signature.result, schemas);
            for result in function.results.values() {
                Self::collect_logical_struct_schema(result, schemas);
            }
            for place in function.places.values() {
                Self::collect_logical_struct_schema(&place.pointee, schemas);
            }
        }
    }

    fn collect_generic_enum_identities(
        logical_type: &LogicalType,
        identities: &mut BTreeSet<String>,
    ) {
        match logical_type {
            LogicalType::ImmutableReference { pointee }
            | LogicalType::MutableReference { pointee }
            | LogicalType::Array {
                element: pointee, ..
            } => Self::collect_generic_enum_identities(pointee, identities),
            LogicalType::Struct { fields, .. }
            | LogicalType::Tuple { elements: fields }
            | LogicalType::EnumFields { fields } => {
                for field in fields {
                    Self::collect_generic_enum_identities(field, identities);
                }
            }
            LogicalType::Enum { name, variants } => {
                if let Some(source) =
                    crate::generic_enum_contract::private_generic_enum_source_name(name)
                {
                    identities.insert(source);
                }
                for payload in variants
                    .iter()
                    .filter_map(|variant| variant.payload.as_ref())
                {
                    Self::collect_generic_enum_identities(payload, identities);
                }
            }
            LogicalType::Int
            | LogicalType::Float
            | LogicalType::Bool
            | LogicalType::Char
            | LogicalType::Void
            | LogicalType::String => {}
        }
    }

    fn collect_metadata_generic_enum_identities(
        metadata: &IrMetadata,
        identities: &mut BTreeSet<String>,
    ) {
        for function in metadata.functions.values() {
            for (_, parameter) in &function.signature.parameters {
                Self::collect_generic_enum_identities(parameter, identities);
            }
            Self::collect_generic_enum_identities(&function.signature.result, identities);
            for result in function.results.values() {
                Self::collect_generic_enum_identities(result, identities);
            }
            for place in function.places.values() {
                Self::collect_generic_enum_identities(&place.pointee, identities);
            }
        }
    }

    pub fn new() -> Self {
        CodeGenerator {
            next_reg: 0,
            next_ptr: 0,
            checked_metadata: None,
            current_function: None,
            language_profile: LanguageProfile::Experimental,
        }
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator {
    fn bump_seed_from_value(max_seed: &mut u64, value: &Value) {
        if let Value::Reg(r) = value {
            *max_seed = (*max_seed).max(u64::from(*r) + 1);
        }
    }

    fn infer_next_reg_seed(instructions: &[Inst]) -> u64 {
        let mut seed = 0u64;

        for inst in instructions {
            match inst {
                Inst::Add(result, lhs, rhs)
                | Inst::FAdd(result, lhs, rhs)
                | Inst::Sub(result, lhs, rhs)
                | Inst::FSub(result, lhs, rhs)
                | Inst::Mul(result, lhs, rhs)
                | Inst::FMul(result, lhs, rhs)
                | Inst::Div(result, lhs, rhs)
                | Inst::FDiv(result, lhs, rhs)
                | Inst::And {
                    result,
                    left: lhs,
                    right: rhs,
                }
                | Inst::Or {
                    result,
                    left: lhs,
                    right: rhs,
                } => {
                    Self::bump_seed_from_value(&mut seed, result);
                    Self::bump_seed_from_value(&mut seed, lhs);
                    Self::bump_seed_from_value(&mut seed, rhs);
                }
                Inst::Alloca(ptr, _)
                | Inst::CheckedMutableOwnedPlaceAlloca { result: ptr, .. }
                | Inst::CheckedImmutableEnumOwnerPlaceAlloca { result: ptr, .. }
                | Inst::CheckedMatchResultPlaceAlloca { result: ptr, .. }
                | Inst::AllocaArray { result: ptr, .. }
                | Inst::CheckedCopyStructArrayAlloca { result: ptr, .. } => {
                    Self::bump_seed_from_value(&mut seed, ptr);
                }
                Inst::CheckedImmutableBorrow { result, source, .. }
                | Inst::CheckedMutableBorrow { result, source, .. } => {
                    Self::bump_seed_from_value(&mut seed, result);
                    Self::bump_seed_from_value(&mut seed, source);
                }
                Inst::CheckedProjectedBorrow {
                    result,
                    root,
                    source,
                    ..
                } => {
                    Self::bump_seed_from_value(&mut seed, result);
                    Self::bump_seed_from_value(&mut seed, root);
                    Self::bump_seed_from_value(&mut seed, source);
                }
                Inst::CheckedImmutableEnumMatchRead {
                    result, reference, ..
                }
                | Inst::CheckedMutableEnumMatchRead {
                    result, reference, ..
                } => {
                    Self::bump_seed_from_value(&mut seed, result);
                    Self::bump_seed_from_value(&mut seed, reference);
                }
                Inst::CheckedMutableBorrowEnd {
                    reference, source, ..
                }
                | Inst::CheckedMutableOwnerImmutableEnumBorrowEnd {
                    reference, source, ..
                } => {
                    Self::bump_seed_from_value(&mut seed, reference);
                    Self::bump_seed_from_value(&mut seed, source);
                }
                Inst::CheckedProjectedBorrowEnd {
                    reference,
                    root,
                    source,
                    ..
                } => {
                    Self::bump_seed_from_value(&mut seed, reference);
                    Self::bump_seed_from_value(&mut seed, root);
                    Self::bump_seed_from_value(&mut seed, source);
                }
                Inst::CheckedImmutableReferenceParameter { result, .. }
                | Inst::CheckedMutableReferenceParameter { result, .. } => {
                    Self::bump_seed_from_value(&mut seed, result);
                }
                Inst::CheckedEnumParameter { result, .. } => {
                    Self::bump_seed_from_value(&mut seed, result);
                }
                Inst::CheckedEnumVariant {
                    result, payload, ..
                } => {
                    Self::bump_seed_from_value(&mut seed, result);
                    if let Some(payload) = payload {
                        Self::bump_seed_from_value(&mut seed, payload);
                    }
                }
                Inst::CheckedEnumVariantFields { result, fields, .. } => {
                    Self::bump_seed_from_value(&mut seed, result);
                    for field in fields {
                        Self::bump_seed_from_value(&mut seed, field);
                    }
                }
                Inst::CheckedEnumPayload { result, value, .. } => {
                    Self::bump_seed_from_value(&mut seed, result);
                    Self::bump_seed_from_value(&mut seed, value);
                }
                Inst::CheckedEnumField { result, value, .. } => {
                    Self::bump_seed_from_value(&mut seed, result);
                    Self::bump_seed_from_value(&mut seed, value);
                }
                Inst::CheckedEnumDispatch { value, .. } => {
                    Self::bump_seed_from_value(&mut seed, value);
                }
                Inst::Store(ptr, value)
                | Inst::CheckedOwnedPlaceAssignment {
                    target: ptr, value, ..
                }
                | Inst::CheckedMutableDereferenceAssignment {
                    target: ptr, value, ..
                }
                | Inst::Load(value, ptr)
                | Inst::SIToFP(value, ptr)
                | Inst::FPToSI(value, ptr) => {
                    Self::bump_seed_from_value(&mut seed, value);
                    Self::bump_seed_from_value(&mut seed, ptr);
                }
                Inst::Return(value)
                | Inst::Branch {
                    condition: value, ..
                } => {
                    Self::bump_seed_from_value(&mut seed, value);
                }
                Inst::Not { result, operand } | Inst::Neg { result, operand } => {
                    Self::bump_seed_from_value(&mut seed, result);
                    Self::bump_seed_from_value(&mut seed, operand);
                }
                Inst::Call {
                    arguments, result, ..
                } => {
                    for arg in arguments {
                        Self::bump_seed_from_value(&mut seed, arg);
                    }
                    if let Some(result) = result {
                        Self::bump_seed_from_value(&mut seed, result);
                    }
                }
                Inst::ICmp {
                    result,
                    left,
                    right,
                    ..
                }
                | Inst::FCmp {
                    result,
                    left,
                    right,
                    ..
                } => {
                    Self::bump_seed_from_value(&mut seed, result);
                    Self::bump_seed_from_value(&mut seed, left);
                    Self::bump_seed_from_value(&mut seed, right);
                }
                Inst::Print { arguments, .. } | Inst::Println { arguments, .. } => {
                    for arg in arguments {
                        Self::bump_seed_from_value(&mut seed, arg);
                    }
                }
                Inst::GetElementPtr {
                    result,
                    base,
                    index,
                    ..
                } => {
                    Self::bump_seed_from_value(&mut seed, result);
                    Self::bump_seed_from_value(&mut seed, base);
                    Self::bump_seed_from_value(&mut seed, index);
                }
                Inst::CheckedCopyStructArrayElementPtr {
                    result,
                    base,
                    index,
                    ..
                } => {
                    Self::bump_seed_from_value(&mut seed, result);
                    Self::bump_seed_from_value(&mut seed, base);
                    Self::bump_seed_from_value(&mut seed, index);
                }
                Inst::AllocaStruct { result, .. } | Inst::CheckedStructAlloca { result, .. } => {
                    Self::bump_seed_from_value(&mut seed, result);
                }
                Inst::CheckedTupleAlloca { result, .. } => {
                    Self::bump_seed_from_value(&mut seed, result);
                }
                Inst::GetFieldPtr { result, base, .. }
                | Inst::CheckedStructFieldPtr { result, base, .. }
                | Inst::CheckedTupleFieldPtr { result, base, .. } => {
                    Self::bump_seed_from_value(&mut seed, result);
                    Self::bump_seed_from_value(&mut seed, base);
                }
                Inst::VecAlloca { result, .. }
                | Inst::VecPop { result, .. }
                | Inst::VecLength { result, .. }
                | Inst::VecCapacity { result, .. } => {
                    Self::bump_seed_from_value(&mut seed, result);
                }
                Inst::VecPush { vec_ptr, value } => {
                    Self::bump_seed_from_value(&mut seed, vec_ptr);
                    Self::bump_seed_from_value(&mut seed, value);
                }
                Inst::VecAccess {
                    result,
                    vec_ptr,
                    index,
                } => {
                    Self::bump_seed_from_value(&mut seed, result);
                    Self::bump_seed_from_value(&mut seed, vec_ptr);
                    Self::bump_seed_from_value(&mut seed, index);
                }
                Inst::VecInit {
                    result, elements, ..
                } => {
                    Self::bump_seed_from_value(&mut seed, result);
                    for element in elements {
                        Self::bump_seed_from_value(&mut seed, element);
                    }
                }
                Inst::ArrayLength { result, array_ptr } => {
                    Self::bump_seed_from_value(&mut seed, result);
                    Self::bump_seed_from_value(&mut seed, array_ptr);
                }
                Inst::ArrayAccess {
                    result,
                    array_ptr,
                    index,
                } => {
                    Self::bump_seed_from_value(&mut seed, result);
                    Self::bump_seed_from_value(&mut seed, array_ptr);
                    Self::bump_seed_from_value(&mut seed, index);
                }
                Inst::EnumDiscriminant { result, enum_ptr }
                | Inst::EnumVariantData {
                    result, enum_ptr, ..
                } => {
                    Self::bump_seed_from_value(&mut seed, result);
                    Self::bump_seed_from_value(&mut seed, enum_ptr);
                }
                Inst::EnumConstruct { result, data, .. } => {
                    Self::bump_seed_from_value(&mut seed, result);
                    for value in data {
                        Self::bump_seed_from_value(&mut seed, value);
                    }
                }
                Inst::FunctionDef { body, .. } | Inst::CheckedFunctionDef { body, .. } => {
                    seed = seed.max(Self::infer_next_reg_seed(body));
                }
                Inst::Jump(_) | Inst::Label(_) => {}
            }
        }

        seed
    }

    fn fresh_reg(&mut self) -> String {
        let reg = format!("reg{}", self.next_reg);
        self.next_reg += 1;
        reg
    }

    fn fresh_ptr(&mut self) -> String {
        let ptr = format!("ptr{}", self.next_ptr);
        self.next_ptr += 1;
        ptr
    }

    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::ImmInt(n) => {
                // Convert int to double for unified storage
                let f = *n as f64;
                format!("0x{:016X}", f.to_bits())
            }
            Value::ImmFloat(f) => {
                // Format float as hexadecimal for LLVM IR
                format!("0x{:016X}", f.to_bits())
            }
            Value::ImmChar(_) => {
                panic!("Character value cannot be lowered as numeric LLVM `double`")
            }
            Value::Reg(r) => format!("%reg{}", r),
            Value::ImmString(_) => {
                panic!("String value cannot be lowered as numeric LLVM `double`")
            }
        }
    }

    fn value_to_int_string(&self, value: &Value) -> String {
        match value {
            Value::ImmInt(n) => format!("{}", n),
            Value::ImmFloat(f) => format!("{}", *f as i64),
            Value::ImmChar(character) => u32::from(*character).to_string(),
            Value::Reg(r) => format!("%reg{}", r),
            Value::ImmString(_) => panic!("String value cannot be lowered as LLVM integer"),
        }
    }

    fn value_to_i32_operand(&mut self, llvm_ir: &mut String, value: &Value) -> String {
        match value {
            Value::ImmInt(n) => n.to_string(),
            Value::ImmFloat(f) => (*f as i64).to_string(),
            Value::ImmChar(character) => u32::from(*character).to_string(),
            Value::Reg(r) => {
                if self.is_checked_char_result(value)
                    || (self.uses_exact_i32_lane() && self.is_checked_int_result(value))
                {
                    return format!("%reg{r}");
                }
                let tmp = self.fresh_reg();
                llvm_ir.push_str(&format!("  %{} = fptosi double %reg{} to i32\n", tmp, r));
                format!("%{}", tmp)
            }
            Value::ImmString(_) => panic!("String value cannot be lowered as i32 operand"),
        }
    }

    fn value_to_i64_operand(&mut self, llvm_ir: &mut String, value: &Value) -> String {
        match value {
            Value::ImmInt(n) => n.to_string(),
            Value::ImmFloat(f) => (*f as i64).to_string(),
            Value::ImmChar(character) => u32::from(*character).to_string(),
            Value::Reg(r) => {
                let tmp = self.fresh_reg();
                llvm_ir.push_str(&format!("  %{} = fptosi double %reg{} to i64\n", tmp, r));
                format!("%{}", tmp)
            }
            Value::ImmString(_) => panic!("String value cannot be lowered as i64 operand"),
        }
    }

    fn checked_copy_array_index_to_i64_operand(
        &mut self,
        llvm_ir: &mut String,
        index: &Value,
        count: usize,
        element_place: u32,
    ) -> String {
        let exact_i32_index = matches!(
            self.language_profile,
            LanguageProfile::ExactI32ArrayV0 | LanguageProfile::ExactI32RecordResultV0
        ) && self.is_checked_int_result(index);
        match index {
            Value::ImmInt(index) => index.to_string(),
            Value::Reg(index) if exact_i32_index => {
                let count = i32::try_from(count)
                    .expect("exact-i32-array-v0 count fits its signed i32 index domain");
                let nonnegative = self.fresh_reg();
                let below_count = self.fresh_reg();
                let in_bounds = self.fresh_reg();
                let safe_label = format!("aero.bounds.safe.{element_place}");
                let trap_label = format!("aero.bounds.trap.{element_place}");

                llvm_ir.push_str(&format!("  %{nonnegative} = icmp sge i32 %reg{index}, 0\n"));
                llvm_ir.push_str(&format!(
                    "  %{below_count} = icmp slt i32 %reg{index}, {count}\n"
                ));
                llvm_ir.push_str(&format!(
                    "  %{in_bounds} = and i1 %{nonnegative}, %{below_count}\n"
                ));
                llvm_ir.push_str(&format!(
                    "  br i1 %{in_bounds}, label %{safe_label}, label %{trap_label}\n"
                ));
                llvm_ir.push_str(&format!("{trap_label}:\n"));
                llvm_ir.push_str("  call void @llvm.trap()\n");
                llvm_ir.push_str("  unreachable\n");
                llvm_ir.push_str(&format!("{safe_label}:\n"));

                let converted = self.fresh_reg();
                llvm_ir.push_str(&format!("  %{converted} = sext i32 %reg{index} to i64\n"));
                format!("%{converted}")
            }
            Value::Reg(index) => {
                let nonnegative = self.fresh_reg();
                let below_count = self.fresh_reg();
                let in_bounds = self.fresh_reg();
                let safe_label = format!("aero.bounds.safe.{element_place}");
                let trap_label = format!("aero.bounds.trap.{element_place}");
                let count = format!("0x{:016X}", (count as f64).to_bits());

                llvm_ir.push_str(&format!(
                    "  %{nonnegative} = fcmp oge double %reg{index}, 0x0000000000000000\n"
                ));
                llvm_ir.push_str(&format!(
                    "  %{below_count} = fcmp olt double %reg{index}, {count}\n"
                ));
                llvm_ir.push_str(&format!(
                    "  %{in_bounds} = and i1 %{nonnegative}, %{below_count}\n"
                ));
                llvm_ir.push_str(&format!(
                    "  br i1 %{in_bounds}, label %{safe_label}, label %{trap_label}\n"
                ));
                llvm_ir.push_str(&format!("{trap_label}:\n"));
                llvm_ir.push_str("  call void @llvm.trap()\n");
                llvm_ir.push_str("  unreachable\n");
                llvm_ir.push_str(&format!("{safe_label}:\n"));

                let converted = self.fresh_reg();
                llvm_ir.push_str(&format!(
                    "  %{converted} = fptosi double %reg{index} to i64\n"
                ));
                format!("%{converted}")
            }
            Value::ImmFloat(_) | Value::ImmChar(_) | Value::ImmString(_) => {
                unreachable!("verified checked Copy-data array indexes are integers")
            }
        }
    }

    fn type_to_llvm(&self, type_name: &str) -> &str {
        match type_name {
            "int" | "i32" => "i32",
            "i64" => "i64",
            "f32" => "float",
            "float" | "f64" | "double" => "double",
            "bool" | "i1" => "i1",
            "char" => "i32",
            "void" => "void",
            _ => "double", // Default fallback
        }
    }

    fn current_function_metadata(&self) -> Option<&FunctionMetadata> {
        let function_name = self.current_function.as_ref()?;
        self.checked_metadata.as_ref()?.functions.get(function_name)
    }

    fn checked_result_type(&self, value: &Value) -> Option<&LogicalType> {
        let Value::Reg(register) = value else {
            return None;
        };
        self.current_function_metadata()?
            .results
            .get(&ResultId(*register))
    }

    fn checked_place_type(&self, value: &Value) -> Option<&LogicalType> {
        let Value::Reg(register) = value else {
            return None;
        };
        self.current_function_metadata()?
            .places
            .get(&PlaceId(*register))
            .map(|place| &place.pointee)
    }

    fn uses_exact_i32_lane(&self) -> bool {
        self.language_profile.uses_exact_i32_lane()
    }

    fn is_checked_int_result(&self, value: &Value) -> bool {
        matches!(self.checked_result_type(value), Some(LogicalType::Int))
    }

    fn stable_int_value_to_string(&self, value: &Value) -> String {
        match value {
            Value::ImmInt(value) => value.to_string(),
            Value::Reg(register) => format!("%reg{register}"),
            Value::ImmFloat(_) | Value::ImmChar(_) | Value::ImmString(_) => {
                unreachable!("stable-scalar-v0 integer operands retain logical Int identity")
            }
        }
    }

    fn is_checked_bool_result(&self, value: &Value) -> bool {
        matches!(self.checked_result_type(value), Some(LogicalType::Bool))
    }

    fn is_checked_enum_result(&self, value: &Value) -> bool {
        matches!(
            self.checked_result_type(value),
            Some(LogicalType::Enum { .. })
        )
    }

    fn is_checked_char_result(&self, value: &Value) -> bool {
        matches!(self.checked_result_type(value), Some(LogicalType::Char))
    }

    fn char_value_to_string(&self, value: &Value) -> String {
        match value {
            Value::ImmChar(character) => u32::from(*character).to_string(),
            Value::Reg(register) => format!("%reg{register}"),
            _ => panic!("verified character value has exact character identity"),
        }
    }

    fn copy_data_value_to_string(&self, ty: &LogicalType, value: &Value) -> String {
        if self.language_profile == LanguageProfile::ExactI32RecordResultV0
            && matches!(ty, LogicalType::Int)
        {
            return self.stable_int_value_to_string(value);
        }
        match PrimitiveKind::from_logical_type(ty) {
            Some(PrimitiveKind::Bool) => self.bool_value_to_string(value),
            Some(PrimitiveKind::Char) => self.char_value_to_string(value),
            _ => self.value_to_string(value),
        }
    }

    fn checked_place_value_to_string(&self, ty: &LogicalType, value: &Value) -> String {
        if self.uses_exact_i32_lane() && matches!(ty, LogicalType::Int) {
            self.stable_int_value_to_string(value)
        } else {
            self.copy_data_value_to_string(ty, value)
        }
    }

    fn bool_value_to_string(&self, value: &Value) -> String {
        match value {
            Value::ImmInt(value) => {
                if *value == 0 {
                    "false".to_string()
                } else {
                    "true".to_string()
                }
            }
            Value::ImmFloat(value) => {
                if *value == 0.0 {
                    "false".to_string()
                } else {
                    "true".to_string()
                }
            }
            Value::ImmChar(_) => panic!("Character value cannot be lowered as LLVM boolean"),
            Value::Reg(register) => format!("%reg{register}"),
            Value::ImmString(_) => {
                panic!("String value cannot be lowered as LLVM boolean")
            }
        }
    }

    fn exact_i32_profile_type(logical_type: &LogicalType, usage: ProfileTypeUse) -> bool {
        profile_type_shape_is_admitted(
            LanguageProfile::ExactI32ArrayV0,
            classify_profile_logical_type(logical_type),
            usage,
        )
    }

    fn collect_authenticated_nominals(
        logical: &LogicalType,
        nominals: &mut BTreeMap<ResolvedProfileAuthenticationSubject, LogicalType>,
    ) -> Result<(), String> {
        match logical {
            LogicalType::Struct { name, fields } => {
                let subject = ResolvedProfileAuthenticationSubject::Nominal {
                    normalized: name.clone(),
                };
                if let Some(existing) = nominals.insert(subject, logical.clone())
                    && existing != *logical
                {
                    return Err(format!(
                        "nominal `{name}` has conflicting verified logical schemas"
                    ));
                }
                for field in fields {
                    Self::collect_authenticated_nominals(field, nominals)?;
                }
            }
            LogicalType::Enum { name, variants } => {
                let subject = ResolvedProfileAuthenticationSubject::Nominal {
                    normalized: name.clone(),
                };
                if let Some(existing) = nominals.insert(subject, logical.clone())
                    && existing != *logical
                {
                    return Err(format!(
                        "nominal `{name}` has conflicting verified logical schemas"
                    ));
                }
                for payload in variants
                    .iter()
                    .filter_map(|variant| variant.payload.as_ref())
                {
                    Self::collect_authenticated_nominals(payload, nominals)?;
                }
            }
            LogicalType::Array { element, .. }
            | LogicalType::ImmutableReference { pointee: element }
            | LogicalType::MutableReference { pointee: element } => {
                Self::collect_authenticated_nominals(element, nominals)?;
            }
            LogicalType::Tuple { elements } => {
                for element in elements {
                    Self::collect_authenticated_nominals(element, nominals)?;
                }
            }
            LogicalType::EnumFields { fields } => {
                for field in fields {
                    Self::collect_authenticated_nominals(field, nominals)?;
                }
            }
            LogicalType::Int
            | LogicalType::Float
            | LogicalType::Bool
            | LogicalType::Char
            | LogicalType::Void
            | LogicalType::String => {}
        }
        Ok(())
    }

    fn authenticated_metadata_subjects(
        metadata: &IrMetadata,
    ) -> Result<BTreeMap<ResolvedProfileAuthenticationSubject, LogicalType>, String> {
        let mut subjects = BTreeMap::new();
        let mut nominals = BTreeMap::new();
        for (function_name, function) in &metadata.functions {
            for (index, (name, logical)) in function.signature.parameters.iter().enumerate() {
                subjects.insert(
                    ResolvedProfileAuthenticationSubject::FunctionParameter {
                        function: function_name.clone(),
                        index,
                        name: name.clone(),
                    },
                    logical.clone(),
                );
                Self::collect_authenticated_nominals(logical, &mut nominals)?;
            }
            subjects.insert(
                ResolvedProfileAuthenticationSubject::FunctionResult {
                    function: function_name.clone(),
                },
                function.signature.result.clone(),
            );
            Self::collect_authenticated_nominals(&function.signature.result, &mut nominals)?;
            for (result, logical) in &function.results {
                subjects.insert(
                    ResolvedProfileAuthenticationSubject::MetadataResult {
                        function: function_name.clone(),
                        result: *result,
                    },
                    logical.clone(),
                );
                Self::collect_authenticated_nominals(logical, &mut nominals)?;
            }
            for (place, metadata) in &function.places {
                subjects.insert(
                    ResolvedProfileAuthenticationSubject::MetadataPlace {
                        function: function_name.clone(),
                        place: *place,
                        name: metadata.name.clone(),
                    },
                    metadata.pointee.clone(),
                );
                Self::collect_authenticated_nominals(&metadata.pointee, &mut nominals)?;
            }
        }
        for (subject, logical) in nominals {
            subjects.insert(subject, logical);
        }
        Ok(subjects)
    }

    fn authentication_coverage_is_usable(
        observation: &ResolvedProfileAuthenticationObservation,
        authenticated: &AuthenticatedResolvedProfileProgram,
    ) -> bool {
        match &observation.coverage {
            ResolvedProfileAuthenticationCoverage::Authenticated(shape) => authenticated
                .program
                .shapes
                .get(shape.0)
                .is_some_and(|logical| logical == &observation.observed),
            ResolvedProfileAuthenticationCoverage::ExplicitUnavailable(_) => false,
            ResolvedProfileAuthenticationCoverage::Uncovered => {
                match (&observation.subject, &observation.observed) {
                    (
                        ResolvedProfileAuthenticationSubject::MetadataResult { .. }
                        | ResolvedProfileAuthenticationSubject::MetadataPlace { .. },
                        _,
                    ) => true,
                    (
                        ResolvedProfileAuthenticationSubject::FunctionResult { .. },
                        LogicalType::Void,
                    ) => true,
                    _ => false,
                }
            }
        }
    }

    fn ensure_exact_record_result_authentication(
        &self,
        metadata: &IrMetadata,
        authenticated: &AuthenticatedResolvedProfileProgram,
    ) -> Result<(), String> {
        validate_resolved_language_profile(
            &authenticated.program,
            LanguageProfile::ExactI32RecordResultV0,
        )
        .map_err(|error| format!("authenticated descriptor no longer admits: {error}"))?;

        let expected = Self::authenticated_metadata_subjects(metadata)?;
        let mut observed = BTreeMap::new();
        for observation in &authenticated.coverage {
            if observed
                .insert(
                    observation.subject.clone(),
                    (observation.observed.clone(), observation),
                )
                .is_some()
            {
                return Err(format!(
                    "authenticated coverage duplicates subject `{:?}`",
                    observation.subject
                ));
            }
        }
        if expected.len() != observed.len() {
            return Err(format!(
                "authenticated coverage count {} does not match re-verified metadata count {}",
                observed.len(),
                expected.len()
            ));
        }
        for (subject, expected_logical) in expected {
            let Some((observed_logical, observation)) = observed.get(&subject) else {
                return Err(format!(
                    "authenticated coverage omits re-verified subject `{subject:?}`"
                ));
            };
            if observed_logical != &expected_logical
                || !Self::authentication_coverage_is_usable(observation, authenticated)
            {
                return Err(format!(
                    "authenticated coverage does not authorize re-verified subject `{subject:?}`"
                ));
            }
        }
        Ok(())
    }

    fn ensure_exact_record_result_metadata(metadata: &IrMetadata) -> Result<(), String> {
        for (function_name, function) in &metadata.functions {
            for (parameter_name, logical) in &function.signature.parameters {
                if !exact_record_result_logical_type_is_admitted(logical) {
                    return Err(format!(
                        "function `{function_name}` parameter `{parameter_name}` has unsupported logical type `{logical}`"
                    ));
                }
            }
            if function.signature.result != LogicalType::Void
                && !exact_record_result_logical_type_is_admitted(&function.signature.result)
            {
                return Err(format!(
                    "function `{function_name}` has unsupported result type `{}`",
                    function.signature.result
                ));
            }
            for (result, logical) in &function.results {
                if !exact_record_result_logical_type_is_admitted(logical) {
                    return Err(format!(
                        "function `{function_name}` result {} has unsupported logical type `{logical}`",
                        result.0
                    ));
                }
            }
            for (place, metadata) in &function.places {
                if !exact_record_result_logical_type_is_admitted(&metadata.pointee) {
                    return Err(format!(
                        "function `{function_name}` place {} has unsupported logical type `{}`",
                        place.0, metadata.pointee
                    ));
                }
            }
        }
        Ok(())
    }

    fn unsupported_exact_record_result_instruction(instructions: &[Inst]) -> Option<&'static str> {
        instructions
            .iter()
            .find_map(|instruction| match instruction {
                Inst::FunctionDef { body, .. } | Inst::CheckedFunctionDef { body, .. } => {
                    Self::unsupported_exact_record_result_instruction(body)
                }
                Inst::FAdd(..) | Inst::FSub(..) | Inst::FMul(..) | Inst::FDiv(..) => {
                    Some("profile-excluded floating-point instruction")
                }
                Inst::Div(..) => Some("profile-excluded division instruction"),
                Inst::Print { .. } | Inst::Println { .. } => {
                    Some("profile-excluded output instruction")
                }
                Inst::AllocaArray { .. } => Some("legacy `alloca array` instruction"),
                Inst::GetElementPtr { .. } => Some("legacy `get element pointer` instruction"),
                Inst::VecAlloca { .. }
                | Inst::VecPush { .. }
                | Inst::VecPop { .. }
                | Inst::VecLength { .. }
                | Inst::VecCapacity { .. } => {
                    Some("profile-excluded dynamic collection instruction")
                }
                _ => None,
            })
    }

    fn ensure_language_profile_codegen_support(
        &self,
        metadata: &IrMetadata,
        ir_functions: &HashMap<String, Function>,
        authenticated: Option<&AuthenticatedResolvedProfileProgram>,
    ) -> Result<(), CodeGenerationError> {
        if self.language_profile == LanguageProfile::ExactI32RecordResultV0 {
            let reject = |detail: String| CodeGenerationError::LanguageProfileContract {
                profile: self.language_profile,
                detail,
            };
            let authenticated = authenticated.ok_or_else(|| {
                reject("missing verifier-authenticated resolved profile token".to_string())
            })?;
            Self::ensure_exact_record_result_metadata(metadata).map_err(&reject)?;
            self.ensure_exact_record_result_authentication(metadata, authenticated)
                .map_err(&reject)?;
            let mut function_names = ir_functions.keys().collect::<Vec<_>>();
            function_names.sort();
            for function_name in function_names {
                if let Some(instruction) = Self::unsupported_exact_record_result_instruction(
                    &ir_functions[function_name].body,
                ) {
                    return Err(reject(format!(
                        "function `{function_name}` contains {instruction}"
                    )));
                }
            }
            return Ok(());
        }
        if self.language_profile != LanguageProfile::ExactI32ArrayV0 {
            return Ok(());
        }

        let reject = |detail: String| CodeGenerationError::LanguageProfileContract {
            profile: self.language_profile,
            detail,
        };
        let mut raw_function_names = ir_functions.keys().collect::<Vec<_>>();
        raw_function_names.sort();
        for function_name in raw_function_names {
            let function = &ir_functions[function_name];
            if let Some(instruction) =
                Self::unsupported_exact_i32_profile_instruction(&function.body)
            {
                return Err(reject(format!(
                    "function `{function_name}` contains {instruction}"
                )));
            }
        }
        for (function_name, function) in &metadata.functions {
            for (parameter_name, parameter_type) in &function.signature.parameters {
                if !Self::exact_i32_profile_type(parameter_type, ProfileTypeUse::Parameter) {
                    return Err(reject(format!(
                        "function `{function_name}` parameter `{parameter_name}` has unsupported logical type `{parameter_type}`"
                    )));
                }
            }
            if !matches!(function.signature.result, LogicalType::Void)
                && !Self::exact_i32_profile_type(&function.signature.result, ProfileTypeUse::Result)
            {
                return Err(reject(format!(
                    "function `{function_name}` has unsupported result type `{}`",
                    function.signature.result
                )));
            }
            for (result, logical_type) in &function.results {
                if !Self::exact_i32_profile_type(logical_type, ProfileTypeUse::Value) {
                    return Err(reject(format!(
                        "function `{function_name}` result {} has unsupported logical type `{logical_type}`",
                        result.0
                    )));
                }
            }
            for (place, metadata) in &function.places {
                if !Self::exact_i32_profile_type(&metadata.pointee, ProfileTypeUse::Binding) {
                    return Err(reject(format!(
                        "function `{function_name}` place {} has unsupported logical type `{}`",
                        place.0, metadata.pointee
                    )));
                }
            }
        }
        Ok(())
    }

    fn unsupported_exact_i32_profile_instruction(instructions: &[Inst]) -> Option<&'static str> {
        instructions
            .iter()
            .find_map(|instruction| match instruction {
                Inst::Add(..)
                | Inst::Sub(..)
                | Inst::Mul(..)
                | Inst::Alloca(..)
                | Inst::Store(..)
                | Inst::Load(..)
                | Inst::Return(..)
                | Inst::Call { .. }
                | Inst::Branch { .. }
                | Inst::Jump(..)
                | Inst::Label(..)
                | Inst::ICmp { .. }
                | Inst::And { .. }
                | Inst::Or { .. }
                | Inst::Not { .. }
                | Inst::Neg { .. }
                | Inst::CheckedCopyStructArrayAlloca { .. }
                | Inst::CheckedCopyStructArrayElementPtr { .. } => None,
                Inst::CheckedMutableOwnedPlaceAlloca { ty, .. }
                    if Self::exact_i32_profile_type(ty, ProfileTypeUse::MutableBinding) =>
                {
                    None
                }
                Inst::CheckedOwnedPlaceAssignment { ty, .. }
                    if Self::exact_i32_profile_type(ty, ProfileTypeUse::OwnedAssignment) =>
                {
                    None
                }
                Inst::CheckedOwnedPlaceAssignment { ty, .. }
                    if matches!(
                        classify_profile_logical_type(ty),
                        ProfileTypeShape::ExactI32Array { .. }
                    ) =>
                {
                    Some("profile-excluded whole-array owned-place assignment")
                }
                Inst::FunctionDef { body, .. } | Inst::CheckedFunctionDef { body, .. } => {
                    Self::unsupported_exact_i32_profile_instruction(body)
                }
                Inst::AllocaArray { .. } => Some("legacy `alloca array` instruction"),
                Inst::GetElementPtr { .. } => Some("legacy `get element pointer` instruction"),
                Inst::Div(..) | Inst::FDiv(..) => Some("profile-excluded division instruction"),
                Inst::Print { .. } | Inst::Println { .. } => {
                    Some("profile-excluded output instruction")
                }
                _ => Some("instruction outside the exact i32 fixed-array profile"),
            })
    }

    fn ensure_checked_codegen_support(
        ir_functions: &HashMap<String, Function>,
    ) -> Result<(), CodeGenerationError> {
        for function in ir_functions.values() {
            Self::ensure_instruction_support(&function.body)?;
        }
        Ok(())
    }

    fn ensure_instruction_support(instructions: &[Inst]) -> Result<(), CodeGenerationError> {
        for instruction in instructions {
            match instruction {
                Inst::Add(..)
                | Inst::FAdd(..)
                | Inst::Sub(..)
                | Inst::FSub(..)
                | Inst::Mul(..)
                | Inst::FMul(..)
                | Inst::Div(..)
                | Inst::FDiv(..)
                | Inst::Alloca(..)
                | Inst::CheckedMutableOwnedPlaceAlloca { .. }
                | Inst::CheckedImmutableEnumOwnerPlaceAlloca { .. }
                | Inst::CheckedMatchResultPlaceAlloca { .. }
                | Inst::CheckedOwnedPlaceAssignment { .. }
                | Inst::CheckedMutableDereferenceAssignment { .. }
                | Inst::Store(..)
                | Inst::Load(..)
                | Inst::Return(..)
                | Inst::SIToFP(..)
                | Inst::FPToSI(..)
                | Inst::Call { .. }
                | Inst::Branch { .. }
                | Inst::Jump(..)
                | Inst::Label(..)
                | Inst::ICmp { .. }
                | Inst::FCmp { .. }
                | Inst::Print { .. }
                | Inst::Println { .. }
                | Inst::And { .. }
                | Inst::Or { .. }
                | Inst::Not { .. }
                | Inst::Neg { .. }
                | Inst::AllocaArray { .. }
                | Inst::GetElementPtr { .. }
                | Inst::CheckedCopyStructArrayAlloca { .. }
                | Inst::CheckedCopyStructArrayElementPtr { .. }
                | Inst::CheckedStructAlloca { .. }
                | Inst::CheckedStructFieldPtr { .. }
                | Inst::CheckedTupleAlloca { .. }
                | Inst::CheckedTupleFieldPtr { .. }
                | Inst::CheckedImmutableBorrow { .. }
                | Inst::CheckedImmutableEnumMatchRead { .. }
                | Inst::CheckedMutableEnumMatchRead { .. }
                | Inst::CheckedMutableBorrow { .. }
                | Inst::CheckedProjectedBorrow { .. }
                | Inst::CheckedMutableBorrowEnd { .. }
                | Inst::CheckedProjectedBorrowEnd { .. }
                | Inst::CheckedMutableOwnerImmutableEnumBorrowEnd { .. }
                | Inst::CheckedImmutableReferenceParameter { .. }
                | Inst::CheckedMutableReferenceParameter { .. }
                | Inst::CheckedEnumParameter { .. }
                | Inst::CheckedEnumVariant { .. }
                | Inst::CheckedEnumVariantFields { .. }
                | Inst::CheckedEnumPayload { .. }
                | Inst::CheckedEnumField { .. }
                | Inst::CheckedEnumDispatch { .. } => {}
                Inst::FunctionDef { body, .. } | Inst::CheckedFunctionDef { body, .. } => {
                    Self::ensure_instruction_support(body)?
                }
                Inst::AllocaStruct { .. } => {
                    return Err(CodeGenerationError::UnsupportedInstruction {
                        instruction: "alloca struct",
                    });
                }
                Inst::GetFieldPtr { .. } => {
                    return Err(CodeGenerationError::UnsupportedInstruction {
                        instruction: "field pointer",
                    });
                }
                Inst::VecAlloca { .. } => {
                    return Err(CodeGenerationError::UnsupportedInstruction {
                        instruction: "vec alloca",
                    });
                }
                Inst::VecPush { .. } => {
                    return Err(CodeGenerationError::UnsupportedInstruction {
                        instruction: "vec push",
                    });
                }
                Inst::VecPop { .. } => {
                    return Err(CodeGenerationError::UnsupportedInstruction {
                        instruction: "vec pop",
                    });
                }
                Inst::VecLength { .. } => {
                    return Err(CodeGenerationError::UnsupportedInstruction {
                        instruction: "vec length",
                    });
                }
                Inst::VecCapacity { .. } => {
                    return Err(CodeGenerationError::UnsupportedInstruction {
                        instruction: "vec capacity",
                    });
                }
                Inst::VecAccess { .. } => {
                    return Err(CodeGenerationError::UnsupportedInstruction {
                        instruction: "vec access",
                    });
                }
                Inst::VecInit { .. } => {
                    return Err(CodeGenerationError::UnsupportedInstruction {
                        instruction: "vec init",
                    });
                }
                Inst::ArrayLength { .. } => {
                    return Err(CodeGenerationError::UnsupportedInstruction {
                        instruction: "array length",
                    });
                }
                Inst::ArrayAccess { .. } => {
                    return Err(CodeGenerationError::UnsupportedInstruction {
                        instruction: "array access",
                    });
                }
                Inst::EnumDiscriminant { .. } => {
                    return Err(CodeGenerationError::UnsupportedInstruction {
                        instruction: "enum discriminant",
                    });
                }
                Inst::EnumVariantData { .. } => {
                    return Err(CodeGenerationError::UnsupportedInstruction {
                        instruction: "enum variant data",
                    });
                }
                Inst::EnumConstruct { .. } => {
                    return Err(CodeGenerationError::UnsupportedInstruction {
                        instruction: "enum construct",
                    });
                }
            }
        }
        Ok(())
    }

    fn contains_dynamic_checked_copy_array_index(instructions: &[Inst]) -> bool {
        instructions.iter().any(|instruction| match instruction {
            Inst::CheckedCopyStructArrayElementPtr { index, .. } => {
                !matches!(index, Value::ImmInt(_))
            }
            Inst::FunctionDef { body, .. } | Inst::CheckedFunctionDef { body, .. } => {
                Self::contains_dynamic_checked_copy_array_index(body)
            }
            _ => false,
        })
    }

    /// Verifies private IR and emits LLVM only after the complete program is admitted.
    pub fn try_generate_code<I>(&mut self, ir: I) -> Result<String, CodeGenerationError>
    where
        I: Into<CheckedIr>,
    {
        self.try_generate_code_with_authentication(ir, None)
    }

    fn try_generate_code_with_authentication<I>(
        &mut self,
        ir: I,
        authenticated: Option<&AuthenticatedResolvedProfileProgram>,
    ) -> Result<String, CodeGenerationError>
    where
        I: Into<CheckedIr>,
    {
        let checked_ir = ir.into();
        let metadata =
            verify_checked_ir(&checked_ir).map_err(CodeGenerationError::IrVerification)?;
        self.ensure_language_profile_codegen_support(&metadata, checked_ir.raw(), authenticated)?;
        Self::ensure_checked_codegen_support(checked_ir.raw())?;

        self.checked_metadata = Some(metadata);
        let llvm_ir = self.generate_checked_code(checked_ir.into_raw());
        self.checked_metadata = None;
        self.current_function = None;
        Ok(llvm_ir)
    }

    pub(crate) fn try_generate_code_with_profile<I>(
        &mut self,
        ir: I,
        language_profile: LanguageProfile,
    ) -> Result<String, CodeGenerationError>
    where
        I: Into<CheckedIr>,
    {
        self.language_profile = language_profile;
        let result = self.try_generate_code(ir);
        self.language_profile = LanguageProfile::Experimental;
        result
    }

    pub(crate) fn try_generate_code_with_authenticated_profile<I>(
        &mut self,
        ir: I,
        language_profile: LanguageProfile,
        authenticated: &AuthenticatedResolvedProfileProgram,
    ) -> Result<String, CodeGenerationError>
    where
        I: Into<CheckedIr>,
    {
        self.language_profile = language_profile;
        let result = self.try_generate_code_with_authentication(ir, Some(authenticated));
        self.language_profile = LanguageProfile::Experimental;
        result
    }

    fn generate_checked_code(&mut self, ir_functions: HashMap<String, Function>) -> String {
        let mut llvm_ir = String::new();
        llvm_ir.push_str("; ModuleID = \"aero_compiler\"\n");
        llvm_ir.push_str("source_filename = \"aero_compiler\"\n");
        llvm_ir.push_str("target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"\n");
        llvm_ir.push_str("target triple = \"x86_64-pc-linux-gnu\"\n\n");
        let requires_array_bounds_trap = ir_functions
            .values()
            .any(|function| Self::contains_dynamic_checked_copy_array_index(&function.body));
        let mut generic_enum_identities = BTreeSet::new();
        if let Some(metadata) = &self.checked_metadata {
            Self::collect_metadata_generic_enum_identities(metadata, &mut generic_enum_identities);
        }
        let has_generic_enum_identities = !generic_enum_identities.is_empty();
        for identity in generic_enum_identities {
            llvm_ir.push_str(&format!("; Aero generic enum: {identity}\n"));
        }
        if has_generic_enum_identities {
            llvm_ir.push('\n');
        }
        let mut struct_schemas = BTreeMap::new();
        if let Some(metadata) = &self.checked_metadata {
            Self::collect_metadata_struct_schemas(metadata, &mut struct_schemas);
        }
        for function in ir_functions.values() {
            Self::collect_struct_schemas(&function.body, &mut struct_schemas);
        }
        let has_struct_schemas = !struct_schemas.is_empty();
        for (name, fields) in struct_schemas {
            let fields = fields
                .iter()
                .map(|field| self.profile_copy_data_type_to_llvm(field))
                .collect::<Vec<_>>()
                .join(", ");
            let struct_type = Self::struct_type_to_llvm(&name);
            llvm_ir.push_str(&format!("{struct_type} = type {{ {fields} }}\n"));
        }
        if has_struct_schemas {
            llvm_ir.push('\n');
        }
        self.generate_printf_declaration(&mut llvm_ir);
        if requires_array_bounds_trap {
            llvm_ir.push_str("declare void @llvm.trap()\n\n");
        }

        let mut function_defs: HashMap<String, FunctionDef> = HashMap::new();
        for function in ir_functions.values() {
            for instruction in &function.body {
                match instruction {
                    Inst::FunctionDef {
                        name,
                        parameters,
                        return_type,
                        body,
                    } => {
                        function_defs.insert(
                            name.clone(),
                            FunctionDef::Legacy {
                                parameters: parameters.clone(),
                                return_type: return_type.clone(),
                                body: body.clone(),
                            },
                        );
                    }
                    Inst::CheckedFunctionDef {
                        name,
                        parameters,
                        result,
                        body,
                    } => {
                        function_defs.insert(
                            name.clone(),
                            FunctionDef::Checked {
                                parameters: parameters.clone(),
                                result: result.clone(),
                                body: body.clone(),
                            },
                        );
                    }
                    _ => {}
                }
            }
        }

        let mut ordered_functions = ir_functions.into_iter().collect::<Vec<_>>();
        ordered_functions.sort_by(|(left, _), (right, _)| left.cmp(right));

        for (function_name, function) in ordered_functions {
            self.current_function = Some(function_name.clone());
            if let Some(definition) = function_defs.get(&function_name) {
                self.generate_function_definition(
                    &mut llvm_ir,
                    &function_name,
                    definition,
                    function.next_reg,
                    &function_defs,
                );
            } else {
                let llvm_name = Self::llvm_function_symbol(&function_name);
                llvm_ir.push_str(&format!("define i32 @{llvm_name}() {{\nentry:\n"));
                let empty_param_types: HashMap<String, String> = HashMap::new();
                self.generate_function_body(
                    &mut llvm_ir,
                    &function.body,
                    &empty_param_types,
                    "i32",
                    &function_defs,
                    function.next_reg,
                );
                llvm_ir.push_str("}\n\n");
            }
        }

        llvm_ir
    }

    #[deprecated(note = "unchecked compatibility API; use CodeGenerator::try_generate_code")]
    pub fn generate_code(&mut self, ir_functions: HashMap<String, Function>) -> String {
        let mut llvm_ir = String::new();
        llvm_ir.push_str("; ModuleID = \"aero_compiler\"\n");
        llvm_ir.push_str("source_filename = \"aero_compiler\"\n");
        llvm_ir.push_str("target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128\"\n");
        llvm_ir.push_str("target triple = \"x86_64-pc-linux-gnu\"\n\n");

        // Add printf declaration for I/O operations
        self.generate_printf_declaration(&mut llvm_ir);

        // First pass: collect function definitions from IR instructions
        let mut function_defs: HashMap<String, FunctionDef> = HashMap::new();

        for func in ir_functions.values() {
            for inst in &func.body {
                if let Inst::FunctionDef {
                    name,
                    parameters,
                    return_type,
                    body,
                } = inst
                {
                    function_defs.insert(
                        name.clone(),
                        FunctionDef::Legacy {
                            parameters: parameters.clone(),
                            return_type: return_type.clone(),
                            body: body.clone(),
                        },
                    );
                }
            }
        }

        // Generate function definitions
        for (func_name, func) in ir_functions {
            // Check if this function has a definition with parameters
            if let Some(definition) = function_defs.get(&func_name) {
                self.generate_function_definition(
                    &mut llvm_ir,
                    &func_name,
                    definition,
                    func.next_reg,
                    &function_defs,
                );
            } else {
                // Legacy function without parameters (like main)
                let llvm_name = Self::llvm_function_symbol(&func_name);
                llvm_ir.push_str(&format!("define i32 @{llvm_name}() {{\nentry:\n"));
                let empty_param_types: HashMap<String, String> = HashMap::new();
                self.generate_function_body(
                    &mut llvm_ir,
                    &func.body,
                    &empty_param_types,
                    "i32",
                    &function_defs,
                    func.next_reg,
                );
                llvm_ir.push_str("}\n\n");
            }
        }

        llvm_ir
    }

    fn generate_function_definition(
        &mut self,
        llvm_ir: &mut String,
        func_name: &str,
        definition: &FunctionDef,
        next_reg_seed: u32,
        function_defs: &HashMap<String, FunctionDef>,
    ) {
        let (parameters, return_llvm_type) = match definition {
            FunctionDef::Legacy {
                parameters,
                return_type,
                ..
            } => (
                parameters
                    .iter()
                    .map(|(name, ty)| (name.clone(), self.type_to_llvm(ty).to_string()))
                    .collect::<Vec<_>>(),
                if let Some(ret_type) = return_type {
                    self.type_to_llvm(ret_type).to_string()
                } else if func_name == "main" {
                    "i32".to_string()
                } else {
                    "void".to_string()
                },
            ),
            FunctionDef::Checked {
                parameters, result, ..
            } => (
                parameters
                    .iter()
                    .map(|(name, ty)| (name.clone(), self.profile_logical_type_to_llvm(ty)))
                    .collect::<Vec<_>>(),
                self.profile_logical_type_to_llvm(result),
            ),
        };

        let mut param_str = String::new();
        for (i, (param_name, param_type)) in parameters.iter().enumerate() {
            if i > 0 {
                param_str.push_str(", ");
            }
            param_str.push_str(&format!(
                "{} %{}",
                param_type,
                Self::llvm_parameter_name(param_name)
            ));
        }

        let llvm_name = Self::llvm_function_symbol(func_name);
        llvm_ir.push_str(&format!(
            "define {} @{}({}) {{\nentry:\n",
            return_llvm_type, llvm_name, param_str
        ));

        let mut param_types = HashMap::new();
        for (param_name, param_type) in &parameters {
            param_types.insert(param_name.clone(), param_type.clone());
        }

        self.generate_function_body(
            llvm_ir,
            definition.body(),
            &param_types,
            &return_llvm_type,
            function_defs,
            next_reg_seed,
        );
        llvm_ir.push_str("}\n\n");
    }

    fn generate_function_body(
        &mut self,
        llvm_ir: &mut String,
        instructions: &[Inst],
        param_types: &HashMap<String, String>,
        return_llvm_type: &str,
        function_defs: &HashMap<String, FunctionDef>,
        next_reg_seed: u32,
    ) {
        self.next_reg = u64::from(next_reg_seed).max(Self::infer_next_reg_seed(instructions));
        let mut initialized_parameters = HashSet::new();

        for inst in instructions {
            match inst {
                Inst::CheckedMutableOwnedPlaceAlloca { result, ty, .. } => {
                    let Value::Reg(ptr_id) = result else {
                        panic!("Expected register for checked mutable owned-place alloca")
                    };
                    let (copy_type, align) = self.checked_place_storage(ty);
                    llvm_ir.push_str(&format!(
                        "  %ptr{ptr_id} = alloca {copy_type}, align {align}\n"
                    ));
                }
                Inst::CheckedImmutableEnumOwnerPlaceAlloca { result, schema, .. } => {
                    let Value::Reg(ptr_id) = result else {
                        panic!("Expected register for checked immutable enum owner alloca")
                    };
                    let enum_type = self
                        .profile_enum_storage_layout(schema)
                        .enum_llvm_type_with(&Self::struct_type_to_llvm);
                    llvm_ir.push_str(&format!("  %ptr{ptr_id} = alloca {enum_type}, align 8\n"));
                }
                Inst::CheckedMatchResultPlaceAlloca {
                    result,
                    result_type,
                    ..
                } => {
                    let Value::Reg(ptr_id) = result else {
                        panic!("Expected register for checked Match result-place alloca")
                    };
                    let (llvm_type, align) = self.checked_place_storage(result_type);
                    llvm_ir.push_str(&format!(
                        "  %ptr{ptr_id} = alloca {llvm_type}, align {align}\n"
                    ));
                }
                Inst::Alloca(ptr_reg, name) => {
                    let ptr_id = match ptr_reg {
                        Value::Reg(r) => *r,
                        _ => panic!("Expected register for alloca"),
                    };
                    let checked_storage = self
                        .checked_place_type(ptr_reg)
                        .map(|logical_type| self.checked_place_storage(logical_type));
                    let (storage_type, storage_align) = checked_storage
                        .as_ref()
                        .map(|(llvm_type, alignment)| (llvm_type.as_str(), *alignment))
                        .unwrap_or(("double", 8));
                    llvm_ir.push_str(&format!(
                        "  %ptr{ptr_id} = alloca {storage_type}, align {storage_align}\n"
                    ));

                    if let Some(param_type) = param_types
                        .get(name)
                        .filter(|_| initialized_parameters.insert(name.clone()))
                    {
                        let parameter = Self::llvm_parameter_name(name);
                        if checked_storage.is_some() && param_type == storage_type {
                            llvm_ir.push_str(&format!(
                                "  store {storage_type} %{parameter}, {storage_type}* %ptr{ptr_id}, align {storage_align}\n"
                            ));
                            continue;
                        }
                        match param_type.as_str() {
                            "double" => llvm_ir.push_str(&format!(
                                "  store {storage_type} %{parameter}, {storage_type}* %ptr{ptr_id}, align {storage_align}\n"
                            )),
                            "i32" => {
                                let tmp = self.fresh_reg();
                                llvm_ir.push_str(&format!(
                                    "  %{} = sitofp i32 %{} to double\n",
                                    tmp, parameter
                                ));
                                llvm_ir.push_str(&format!(
                                    "  store {storage_type} %{tmp}, {storage_type}* %ptr{ptr_id}, align {storage_align}\n"
                                ));
                            }
                            "i64" => {
                                let tmp = self.fresh_reg();
                                llvm_ir.push_str(&format!(
                                    "  %{} = sitofp i64 %{} to double\n",
                                    tmp, parameter
                                ));
                                llvm_ir.push_str(&format!(
                                    "  store {storage_type} %{tmp}, {storage_type}* %ptr{ptr_id}, align {storage_align}\n"
                                ));
                            }
                            "i1" => {
                                let tmp = self.fresh_reg();
                                llvm_ir.push_str(&format!(
                                    "  %{} = uitofp i1 %{} to double\n",
                                    tmp, parameter
                                ));
                                llvm_ir.push_str(&format!(
                                    "  store {storage_type} %{tmp}, {storage_type}* %ptr{ptr_id}, align {storage_align}\n"
                                ));
                            }
                            _ => llvm_ir.push_str(&format!(
                                "  store {storage_type} %{parameter}, {storage_type}* %ptr{ptr_id}, align {storage_align}\n"
                            )),
                        }
                    }
                }
                Inst::Store(ptr_reg, value) => {
                    if let Some(logical_type) = self.checked_place_type(ptr_reg).cloned() {
                        let Value::Reg(ptr_id) = ptr_reg else {
                            panic!("Expected register for checked store pointer")
                        };
                        let (storage_type, storage_align) =
                            self.checked_place_storage(&logical_type);
                        let stored_value = self.checked_place_value_to_string(&logical_type, value);
                        llvm_ir.push_str(&format!(
                            "  store {storage_type} {stored_value}, {storage_type}* %ptr{ptr_id}, align {storage_align}\n"
                        ));
                        continue;
                    }
                    let val_str = self.value_to_string(value);
                    let ptr_str = match ptr_reg {
                        Value::Reg(r) => format!("ptr{}", r),
                        _ => panic!("Expected register for store pointer"),
                    };
                    llvm_ir.push_str(&format!(
                        "  store double {}, double* %{}, align 8\n",
                        val_str, ptr_str
                    ));
                }
                Inst::CheckedOwnedPlaceAssignment { target, value, ty }
                | Inst::CheckedMutableDereferenceAssignment {
                    target,
                    value,
                    pointee: ty,
                } => {
                    let Value::Reg(ptr_id) = target else {
                        panic!("Expected register for checked owned-place assignment target")
                    };
                    let (storage_type, storage_align) = self.checked_place_storage(ty);
                    let stored_value = self.checked_place_value_to_string(ty, value);
                    llvm_ir.push_str(&format!(
                        "  store {storage_type} {stored_value}, {storage_type}* %ptr{ptr_id}, align {storage_align}\n"
                    ));
                }
                Inst::Load(result_reg, ptr_reg) => {
                    if let Some(logical_type) = self.checked_place_type(ptr_reg) {
                        let result_id = match result_reg {
                            Value::Reg(register) => *register,
                            _ => panic!("Expected register for load result"),
                        };
                        let ptr_id = match ptr_reg {
                            Value::Reg(register) => *register,
                            _ => panic!("Expected register for load pointer"),
                        };
                        let (storage_type, storage_align) =
                            self.checked_place_storage(logical_type);
                        llvm_ir.push_str(&format!(
                            "  %reg{result_id} = load {storage_type}, {storage_type}* %ptr{ptr_id}, align {storage_align}\n"
                        ));
                        continue;
                    }
                    let result_str = match result_reg {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for load result"),
                    };
                    let ptr_str = match ptr_reg {
                        Value::Reg(r) => format!("ptr{}", r),
                        _ => panic!("Expected register for load pointer"),
                    };
                    llvm_ir.push_str(&format!(
                        "  %{} = load double, double* %{}, align 8\n",
                        result_str, ptr_str
                    ));
                }
                Inst::Add(result_reg, lhs, rhs) if self.uses_exact_i32_lane() => {
                    let Value::Reg(result) = result_reg else {
                        panic!("Expected register for stable integer add result")
                    };
                    llvm_ir.push_str(&format!(
                        "  %reg{result} = add i32 {}, {}\n",
                        self.stable_int_value_to_string(lhs),
                        self.stable_int_value_to_string(rhs)
                    ));
                }
                Inst::Add(result_reg, lhs, rhs) | Inst::FAdd(result_reg, lhs, rhs) => {
                    let result_str = match result_reg {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for add result"),
                    };
                    let lhs_str = self.value_to_string(lhs);
                    let rhs_str = self.value_to_string(rhs);
                    llvm_ir.push_str(&format!(
                        "  %{} = fadd double {}, {}\n",
                        result_str, lhs_str, rhs_str
                    ));
                }
                Inst::Sub(result_reg, lhs, rhs) if self.uses_exact_i32_lane() => {
                    let Value::Reg(result) = result_reg else {
                        panic!("Expected register for stable integer subtract result")
                    };
                    llvm_ir.push_str(&format!(
                        "  %reg{result} = sub i32 {}, {}\n",
                        self.stable_int_value_to_string(lhs),
                        self.stable_int_value_to_string(rhs)
                    ));
                }
                Inst::Sub(result_reg, lhs, rhs) | Inst::FSub(result_reg, lhs, rhs) => {
                    let result_str = match result_reg {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for sub result"),
                    };
                    let lhs_str = self.value_to_string(lhs);
                    let rhs_str = self.value_to_string(rhs);
                    llvm_ir.push_str(&format!(
                        "  %{} = fsub double {}, {}\n",
                        result_str, lhs_str, rhs_str
                    ));
                }
                Inst::Mul(result_reg, lhs, rhs) if self.uses_exact_i32_lane() => {
                    let Value::Reg(result) = result_reg else {
                        panic!("Expected register for stable integer multiply result")
                    };
                    llvm_ir.push_str(&format!(
                        "  %reg{result} = mul i32 {}, {}\n",
                        self.stable_int_value_to_string(lhs),
                        self.stable_int_value_to_string(rhs)
                    ));
                }
                Inst::Mul(result_reg, lhs, rhs) | Inst::FMul(result_reg, lhs, rhs) => {
                    let result_str = match result_reg {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for mul result"),
                    };
                    let lhs_str = self.value_to_string(lhs);
                    let rhs_str = self.value_to_string(rhs);
                    llvm_ir.push_str(&format!(
                        "  %{} = fmul double {}, {}\n",
                        result_str, lhs_str, rhs_str
                    ));
                }
                Inst::Div(result_reg, lhs, rhs) | Inst::FDiv(result_reg, lhs, rhs) => {
                    let result_str = match result_reg {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for div result"),
                    };
                    let lhs_str = self.value_to_string(lhs);
                    let rhs_str = self.value_to_string(rhs);
                    llvm_ir.push_str(&format!(
                        "  %{} = fdiv double {}, {}\n",
                        result_str, lhs_str, rhs_str
                    ));
                }
                Inst::FPToSI(result_reg, value) => {
                    let result_str = match result_reg {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for fptosi result"),
                    };
                    let val_str = self.value_to_string(value);
                    if self.checked_metadata.is_some() {
                        let integer_result = self.fresh_reg();
                        llvm_ir.push_str(&format!(
                            "  %{integer_result} = fptosi double {val_str} to i32\n"
                        ));
                        llvm_ir.push_str(&format!(
                            "  %{result_str} = sitofp i32 %{integer_result} to double\n"
                        ));
                    } else {
                        llvm_ir.push_str(&format!(
                            "  %{} = fptosi double {} to i64\n",
                            result_str, val_str
                        ));
                    }
                }
                Inst::Return(value) => self.emit_return(llvm_ir, value, return_llvm_type),
                Inst::SIToFP(result_reg, value) => {
                    let result_str = match result_reg {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for sitofp result"),
                    };
                    let val_str = self.value_to_string(value);
                    llvm_ir.push_str(&format!(
                        "  %{} = fadd double {}, 0x0000000000000000\n",
                        result_str, val_str
                    ));
                }
                Inst::FunctionDef { .. } | Inst::CheckedFunctionDef { .. } => {}
                Inst::Call {
                    function,
                    arguments,
                    result,
                } => {
                    self.generate_function_call(llvm_ir, function, arguments, result, function_defs)
                }
                Inst::Branch {
                    condition,
                    true_label,
                    false_label,
                } => self.generate_branch(llvm_ir, condition, true_label, false_label),
                Inst::Jump(label) => llvm_ir.push_str(&format!("  br label %{}\n", label)),
                Inst::Label(label) => llvm_ir.push_str(&format!("{}:\n", label)),
                Inst::ICmp {
                    op,
                    result,
                    left,
                    right,
                } => {
                    let result_str = match result {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for icmp result"),
                    };
                    if self.is_checked_bool_result(left) {
                        let left_str = self.bool_value_to_string(left);
                        let right_str = self.bool_value_to_string(right);
                        llvm_ir.push_str(&format!(
                            "  %{} = icmp {} i1 {}, {}\n",
                            result_str, op, left_str, right_str
                        ));
                    } else {
                        let left_str = self.value_to_i32_operand(llvm_ir, left);
                        let right_str = self.value_to_i32_operand(llvm_ir, right);
                        llvm_ir.push_str(&format!(
                            "  %{} = icmp {} i32 {}, {}\n",
                            result_str, op, left_str, right_str
                        ));
                    }
                }
                Inst::FCmp {
                    op,
                    result,
                    left,
                    right,
                } => {
                    let result_str = match result {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for fcmp result"),
                    };
                    let left_str = self.value_to_string(left);
                    let right_str = self.value_to_string(right);
                    llvm_ir.push_str(&format!(
                        "  %{} = fcmp {} double {}, {}\n",
                        result_str, op, left_str, right_str
                    ));
                }
                Inst::Print {
                    format_string,
                    arguments,
                } => self.generate_print_call(llvm_ir, format_string, arguments, false),
                Inst::Println {
                    format_string,
                    arguments,
                } => self.generate_print_call(llvm_ir, format_string, arguments, true),
                Inst::And {
                    result,
                    left,
                    right,
                } => {
                    let result_str = match result {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for and result"),
                    };
                    let left_str = self.value_to_string(left);
                    let right_str = self.value_to_string(right);
                    llvm_ir.push_str(&format!(
                        "  %{} = and i1 {}, {}\n",
                        result_str, left_str, right_str
                    ));
                }
                Inst::Or {
                    result,
                    left,
                    right,
                } => {
                    let result_str = match result {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for or result"),
                    };
                    let left_str = self.value_to_string(left);
                    let right_str = self.value_to_string(right);
                    llvm_ir.push_str(&format!(
                        "  %{} = or i1 {}, {}\n",
                        result_str, left_str, right_str
                    ));
                }
                Inst::Not { result, operand } => {
                    let result_str = match result {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for not result"),
                    };
                    let operand_str = self.value_to_string(operand);
                    llvm_ir.push_str(&format!(
                        "  %{} = xor i1 {}, true\n",
                        result_str, operand_str
                    ));
                }
                Inst::Neg { result, operand } => {
                    let result_str = match result {
                        Value::Reg(r) => format!("reg{}", r),
                        _ => panic!("Expected register for neg result"),
                    };
                    if self.uses_exact_i32_lane() {
                        llvm_ir.push_str(&format!(
                            "  %{result_str} = sub i32 0, {}\n",
                            self.stable_int_value_to_string(operand)
                        ));
                        continue;
                    }
                    let operand_str = self.value_to_string(operand);
                    llvm_ir.push_str(&format!(
                        "  %{} = fsub double 0.0, {}\n",
                        result_str, operand_str
                    ));
                }
                Inst::AllocaArray {
                    result,
                    elem_type,
                    count,
                } => {
                    let result_str = match result {
                        Value::Reg(r) => format!("ptr{}", r),
                        _ => panic!("Expected register for array alloca"),
                    };
                    llvm_ir.push_str(&format!(
                        "  %{} = alloca [{} x {}], align 8\n",
                        result_str, count, elem_type
                    ));
                }
                Inst::GetElementPtr {
                    result,
                    base,
                    index,
                    elem_type,
                } => {
                    let result_str = match result {
                        Value::Reg(r) => format!("ptr{}", r),
                        _ => panic!("Expected register for GEP result"),
                    };
                    let base_str = match base {
                        Value::Reg(r) => format!("ptr{}", r),
                        _ => panic!("Expected register for GEP base"),
                    };
                    let index_str = self.value_to_i64_operand(llvm_ir, index);
                    llvm_ir.push_str(&format!(
                        "  %{} = getelementptr inbounds {}, {}* %{}, i64 0, i64 {}\n",
                        result_str, elem_type, elem_type, base_str, index_str
                    ));
                }
                Inst::CheckedCopyStructArrayAlloca {
                    result,
                    element,
                    count,
                } => {
                    let Value::Reg(result) = result else {
                        panic!("Expected register for checked Copy-data array alloca")
                    };
                    let aggregate = LogicalType::Array {
                        element: Box::new(element.clone()),
                        count: *count,
                    };
                    let aggregate = self.profile_copy_data_type_to_llvm(&aggregate);
                    llvm_ir.push_str(&format!("  %ptr{result} = alloca {aggregate}, align 8\n"));
                }
                Inst::CheckedCopyStructArrayElementPtr {
                    result,
                    base,
                    index,
                    element,
                    count,
                } => {
                    let Value::Reg(result) = result else {
                        panic!("Expected register for checked Copy-data array element")
                    };
                    let Value::Reg(base) = base else {
                        panic!("Expected register for checked Copy-data array base")
                    };
                    let aggregate = LogicalType::Array {
                        element: Box::new(element.clone()),
                        count: *count,
                    };
                    let aggregate = self.profile_copy_data_type_to_llvm(&aggregate);
                    let index = self
                        .checked_copy_array_index_to_i64_operand(llvm_ir, index, *count, *result);
                    llvm_ir.push_str(&format!(
                        "  %ptr{result} = getelementptr inbounds {aggregate}, {aggregate}* %ptr{base}, i64 0, i64 {index}\n"
                    ));
                }
                Inst::AllocaStruct {
                    result,
                    struct_type,
                } => {
                    let result_str = match result {
                        Value::Reg(r) => format!("ptr{}", r),
                        _ => panic!("Expected register for struct alloca"),
                    };
                    llvm_ir.push_str(&format!(
                        "  %{} = alloca %{}, align 8\n",
                        result_str, struct_type
                    ));
                }
                Inst::GetFieldPtr {
                    result,
                    base,
                    field_index,
                    struct_type,
                } => {
                    let result_str = match result {
                        Value::Reg(r) => format!("ptr{}", r),
                        _ => panic!("Expected register for field GEP result"),
                    };
                    let base_str = match base {
                        Value::Reg(r) => format!("ptr{}", r),
                        _ => panic!("Expected register for field GEP base"),
                    };
                    llvm_ir.push_str(&format!(
                        "  %{} = getelementptr inbounds %{}, %{}* %{}, i32 0, i32 {}\n",
                        result_str, struct_type, struct_type, base_str, field_index
                    ));
                }
                Inst::CheckedStructAlloca {
                    result,
                    struct_name,
                    ..
                } => {
                    let Value::Reg(result) = result else {
                        panic!("Expected register for checked struct alloca")
                    };
                    let struct_type = Self::struct_type_to_llvm(struct_name);
                    llvm_ir.push_str(&format!("  %ptr{result} = alloca {struct_type}, align 8\n"));
                }
                Inst::CheckedStructFieldPtr {
                    result,
                    base,
                    struct_name,
                    field_index,
                    ..
                } => {
                    let Value::Reg(result) = result else {
                        panic!("Expected register for checked struct field pointer")
                    };
                    let Value::Reg(base) = base else {
                        panic!("Expected register for checked struct base")
                    };
                    let struct_type = Self::struct_type_to_llvm(struct_name);
                    llvm_ir.push_str(&format!(
                        "  %ptr{result} = getelementptr inbounds {struct_type}, {struct_type}* %ptr{base}, i32 0, i32 {field_index}\n"
                    ));
                }
                Inst::CheckedTupleAlloca {
                    result,
                    element_types,
                } => {
                    let Value::Reg(result) = result else {
                        panic!("Expected register for checked tuple alloca")
                    };
                    let tuple_type = Self::logical_type_to_llvm(&LogicalType::Tuple {
                        elements: element_types.clone(),
                    });
                    llvm_ir.push_str(&format!("  %ptr{result} = alloca {tuple_type}, align 8\n"));
                }
                Inst::CheckedTupleFieldPtr {
                    result,
                    base,
                    element_types,
                    field_index,
                    ..
                } => {
                    let Value::Reg(result) = result else {
                        panic!("Expected register for checked tuple field pointer")
                    };
                    let Value::Reg(base) = base else {
                        panic!("Expected register for checked tuple base")
                    };
                    let tuple_type = Self::logical_type_to_llvm(&LogicalType::Tuple {
                        elements: element_types.clone(),
                    });
                    llvm_ir.push_str(&format!(
                        "  %ptr{result} = getelementptr inbounds {tuple_type}, {tuple_type}* %ptr{base}, i32 0, i32 {field_index}\n"
                    ));
                }
                Inst::CheckedImmutableBorrow {
                    result,
                    source,
                    pointee,
                }
                | Inst::CheckedMutableBorrow {
                    result,
                    source,
                    pointee,
                }
                | Inst::CheckedProjectedBorrow {
                    result,
                    source,
                    pointee,
                    ..
                } => {
                    let Value::Reg(result) = result else {
                        panic!("Expected register for checked scalar borrow result")
                    };
                    let Value::Reg(source) = source else {
                        panic!("Expected register for checked scalar borrow source")
                    };
                    let pointee = Self::reference_pointee_to_llvm(pointee);
                    llvm_ir.push_str(&format!(
                        "  %ptr{result} = getelementptr inbounds {pointee}, {pointee}* %ptr{source}, i64 0\n"
                    ));
                }
                Inst::CheckedImmutableEnumMatchRead {
                    result,
                    reference,
                    schema,
                }
                | Inst::CheckedMutableEnumMatchRead {
                    result,
                    reference,
                    schema,
                } => {
                    let Value::Reg(result) = result else {
                        panic!("Expected register for checked enum-reference Match read")
                    };
                    let Value::Reg(reference) = reference else {
                        panic!("Expected reference place for checked enum-reference Match read")
                    };
                    let enum_type = Self::render_enum_storage_layout(schema);
                    llvm_ir.push_str(&format!(
                        "  %reg{result} = load {enum_type}, {enum_type}* %ptr{reference}, align 8\n"
                    ));
                }
                Inst::CheckedImmutableReferenceParameter {
                    result,
                    parameter,
                    pointee,
                } => {
                    let Value::Reg(result) = result else {
                        panic!("Expected register for checked immutable reference parameter")
                    };
                    let pointee = Self::reference_pointee_to_llvm(pointee);
                    let parameter = Self::llvm_parameter_name(parameter);
                    llvm_ir.push_str(&format!(
                        "  %ptr{result} = getelementptr inbounds {pointee}, {pointee}* %{parameter}, i64 0\n"
                    ));
                }
                Inst::CheckedMutableReferenceParameter {
                    result,
                    parameter,
                    pointee,
                } => {
                    let Value::Reg(result) = result else {
                        panic!("Expected register for checked mutable reference parameter")
                    };
                    let pointee = Self::reference_pointee_to_llvm(pointee);
                    let parameter = Self::llvm_parameter_name(parameter);
                    llvm_ir.push_str(&format!(
                        "  %ptr{result} = getelementptr inbounds {pointee}, {pointee}* %{parameter}, i64 0\n"
                    ));
                }
                Inst::CheckedMutableBorrowEnd { .. }
                | Inst::CheckedProjectedBorrowEnd { .. }
                | Inst::CheckedMutableOwnerImmutableEnumBorrowEnd { .. } => {}
                Inst::CheckedEnumParameter {
                    result,
                    parameter,
                    schema,
                } => {
                    let Value::Reg(result) = result else {
                        panic!("Expected register for checked enum parameter")
                    };
                    let parameter = Self::llvm_parameter_name(parameter);
                    let layout = self.profile_enum_storage_layout(schema);
                    let tag_lane = layout.tag_lane();
                    let tag_type = layout
                        .lane_llvm_type(tag_lane, &Self::struct_type_to_llvm)
                        .expect("enum storage has a tag lane");
                    let tag_zero = layout
                        .lane_zero_value(tag_lane)
                        .expect("enum storage tag lane has a zero value");
                    if layout.is_unit() {
                        llvm_ir.push_str(&format!(
                            "  %reg{result} = add {tag_type} %{parameter}, {tag_zero}\n"
                        ));
                        continue;
                    }
                    let enum_type = layout.enum_llvm_type_with(&Self::struct_type_to_llvm);
                    if !layout.is_compact() {
                        let tag = self.fresh_reg();
                        llvm_ir.push_str(&format!(
                            "  %{tag} = extractvalue {enum_type} %{parameter}, {tag_lane}\n"
                        ));
                        llvm_ir.push_str(&format!(
                            "  %reg{result} = insertvalue {enum_type} %{parameter}, {tag_type} %{tag}, {tag_lane}\n"
                        ));
                        continue;
                    }
                    let tag = self.fresh_reg();
                    let numeric = self.fresh_reg();
                    let boolean = self.fresh_reg();
                    let with_tag = self.fresh_reg();
                    let with_numeric = self.fresh_reg();
                    let numeric_lane = layout
                        .compact_numeric_lane()
                        .expect("compact enum has a numeric storage lane");
                    let boolean_lane = layout
                        .compact_boolean_lane()
                        .expect("compact enum has a Boolean storage lane");
                    let numeric_type = layout
                        .lane_llvm_type(numeric_lane, &Self::struct_type_to_llvm)
                        .expect("compact enum has a numeric storage lane");
                    let boolean_type = layout
                        .lane_llvm_type(boolean_lane, &Self::struct_type_to_llvm)
                        .expect("compact enum has a Boolean storage lane");
                    llvm_ir.push_str(&format!(
                        "  %{tag} = extractvalue {enum_type} %{parameter}, {tag_lane}\n"
                    ));
                    llvm_ir.push_str(&format!(
                        "  %{numeric} = extractvalue {enum_type} %{parameter}, {numeric_lane}\n"
                    ));
                    llvm_ir.push_str(&format!(
                        "  %{boolean} = extractvalue {enum_type} %{parameter}, {boolean_lane}\n"
                    ));
                    llvm_ir.push_str(&format!(
                        "  %{with_tag} = insertvalue {enum_type} poison, {tag_type} %{tag}, {tag_lane}\n"
                    ));
                    llvm_ir.push_str(&format!(
                        "  %{with_numeric} = insertvalue {enum_type} %{with_tag}, {numeric_type} %{numeric}, {numeric_lane}\n"
                    ));
                    llvm_ir.push_str(&format!(
                        "  %reg{result} = insertvalue {enum_type} %{with_numeric}, {boolean_type} %{boolean}, {boolean_lane}\n"
                    ));
                }
                Inst::CheckedEnumVariant {
                    result,
                    schema,
                    variant_index,
                    payload,
                } => {
                    let Value::Reg(result) = result else {
                        panic!("Expected register for checked enum variant")
                    };
                    let layout = self.profile_enum_storage_layout(schema);
                    let tag_lane = layout.tag_lane();
                    let tag_type = layout
                        .lane_llvm_type(tag_lane, &Self::struct_type_to_llvm)
                        .expect("enum storage has a tag lane");
                    let tag_zero = layout
                        .lane_zero_value(tag_lane)
                        .expect("enum storage tag lane has a zero value");
                    if layout.is_unit() {
                        llvm_ir.push_str(&format!(
                            "  %reg{result} = add {tag_type} {tag_zero}, {variant_index}\n"
                        ));
                        continue;
                    }
                    let enum_type = layout.enum_llvm_type_with(&Self::struct_type_to_llvm);
                    if !layout.is_compact() {
                        let tagged = self.fresh_reg();
                        llvm_ir.push_str(&format!(
                            "  %{tagged} = insertvalue {enum_type} poison, {tag_type} {variant_index}, {tag_lane}\n"
                        ));
                        let payload_lanes = layout.payload_variants();
                        let mut aggregate = format!("%{tagged}");
                        for (position, (source_index, lane, payload_type)) in
                            payload_lanes.iter().copied().enumerate()
                        {
                            let payload_llvm = layout
                                .lane_llvm_type(lane, &Self::struct_type_to_llvm)
                                .expect("verified enum payload lane has a physical type");
                            let lane_value = if source_index == *variant_index {
                                let value = payload
                                    .as_ref()
                                    .expect("verified selected payload variant has a value");
                                self.copy_data_value_to_string(payload_type, value)
                            } else {
                                layout
                                    .lane_zero_value(lane)
                                    .expect("verified enum payload lane has a zero value")
                            };
                            let output = if position + 1 == payload_lanes.len() {
                                format!("reg{result}")
                            } else {
                                self.fresh_reg()
                            };
                            llvm_ir.push_str(&format!(
                                "  %{output} = insertvalue {enum_type} {aggregate}, {payload_llvm} {lane_value}, {lane}\n"
                            ));
                            aggregate = format!("%{output}");
                        }
                        continue;
                    }
                    let tagged = self.fresh_reg();
                    llvm_ir.push_str(&format!(
                        "  %{tagged} = insertvalue {enum_type} poison, {tag_type} {variant_index}, {tag_lane}\n"
                    ));
                    let payload_type = schema.variants[*variant_index].payload.as_ref();
                    let numeric = self.fresh_reg();
                    let numeric_lane = layout
                        .compact_numeric_lane()
                        .expect("compact enum has a numeric storage lane");
                    let numeric_type = layout
                        .lane_llvm_type(numeric_lane, &Self::struct_type_to_llvm)
                        .expect("compact enum has a numeric storage lane");
                    let numeric_zero = layout
                        .lane_zero_value(numeric_lane)
                        .expect("compact enum numeric lane has a zero value");
                    let numeric_value = match (payload_type, payload) {
                        (Some(LogicalType::Int | LogicalType::Float), Some(value)) => {
                            self.value_to_string(value)
                        }
                        _ => numeric_zero,
                    };
                    llvm_ir.push_str(&format!(
                        "  %{numeric} = insertvalue {enum_type} %{tagged}, {numeric_type} {numeric_value}, {numeric_lane}\n"
                    ));
                    let boolean_lane = layout
                        .compact_boolean_lane()
                        .expect("compact enum has a Boolean storage lane");
                    let boolean_type = layout
                        .lane_llvm_type(boolean_lane, &Self::struct_type_to_llvm)
                        .expect("compact enum has a Boolean storage lane");
                    let boolean_zero = layout
                        .lane_zero_value(boolean_lane)
                        .expect("compact enum Boolean lane has a zero value");
                    let bool_value = match (payload_type, payload) {
                        (Some(LogicalType::Bool), Some(value)) => self.bool_value_to_string(value),
                        _ => boolean_zero,
                    };
                    llvm_ir.push_str(&format!(
                        "  %reg{result} = insertvalue {enum_type} %{numeric}, {boolean_type} {bool_value}, {boolean_lane}\n"
                    ));
                }
                Inst::CheckedEnumVariantFields {
                    result,
                    schema,
                    variant_index,
                    fields,
                } => {
                    let Value::Reg(result) = result else {
                        panic!("Expected register for checked multi-field enum variant")
                    };
                    let LogicalType::EnumFields {
                        fields: field_types,
                    } = schema.variants[*variant_index]
                        .payload
                        .as_ref()
                        .expect("verified multi-field enum variant has a payload product")
                    else {
                        unreachable!("verified multi-field enum variant has a product schema")
                    };
                    let layout = self.profile_enum_storage_layout(schema);
                    let tag_lane = layout.tag_lane();
                    let tag_type = layout
                        .lane_llvm_type(tag_lane, &Self::struct_type_to_llvm)
                        .expect("enum storage has a tag lane");
                    let selected_lane = layout
                        .payload_lane(*variant_index)
                        .expect("verified multi-field enum variant has a lane");
                    let payload_type = layout
                        .lane_llvm_type(selected_lane, &Self::struct_type_to_llvm)
                        .expect("verified multi-field enum payload has a physical type");
                    let mut payload_value = "poison".to_string();
                    for (field_index, (field, field_type)) in
                        fields.iter().zip(field_types).enumerate()
                    {
                        let output = self.fresh_reg();
                        let field_llvm = self.profile_copy_data_type_to_llvm(field_type);
                        let field_value = self.copy_data_value_to_string(field_type, field);
                        llvm_ir.push_str(&format!(
                            "  %{output} = insertvalue {payload_type} {payload_value}, {field_llvm} {field_value}, {field_index}\n"
                        ));
                        payload_value = format!("%{output}");
                    }

                    let enum_type = layout.enum_llvm_type_with(&Self::struct_type_to_llvm);
                    let tagged = self.fresh_reg();
                    llvm_ir.push_str(&format!(
                        "  %{tagged} = insertvalue {enum_type} poison, {tag_type} {variant_index}, {tag_lane}\n"
                    ));
                    let payload_lanes = layout.payload_variants();
                    let mut aggregate = format!("%{tagged}");
                    for (position, (source_index, lane, _source_type)) in
                        payload_lanes.iter().copied().enumerate()
                    {
                        let source_llvm = layout
                            .lane_llvm_type(lane, &Self::struct_type_to_llvm)
                            .expect("verified enum payload lane has a physical type");
                        let lane_value = if source_index == *variant_index {
                            payload_value.clone()
                        } else {
                            layout
                                .lane_zero_value(lane)
                                .expect("verified enum payload lane has a zero value")
                        };
                        let output = if position + 1 == payload_lanes.len() {
                            format!("reg{result}")
                        } else {
                            self.fresh_reg()
                        };
                        llvm_ir.push_str(&format!(
                            "  %{output} = insertvalue {enum_type} {aggregate}, {source_llvm} {lane_value}, {lane}\n"
                        ));
                        aggregate = format!("%{output}");
                    }
                }
                Inst::CheckedEnumPayload {
                    result,
                    value,
                    schema,
                    variant_index,
                } => {
                    let Value::Reg(result) = result else {
                        panic!("Expected register for checked enum payload")
                    };
                    let Value::Reg(value) = value else {
                        panic!("Expected register for checked enum payload source")
                    };
                    let layout = self.profile_enum_storage_layout(schema);
                    let enum_type = layout.enum_llvm_type_with(&Self::struct_type_to_llvm);
                    let lane = layout
                        .payload_lane(*variant_index)
                        .expect("verified enum payload has a lane");
                    llvm_ir.push_str(&format!(
                        "  %reg{result} = extractvalue {enum_type} %reg{value}, {lane}\n"
                    ));
                }
                Inst::CheckedEnumField {
                    result,
                    value,
                    schema,
                    variant_index,
                    field_index,
                } => {
                    let Value::Reg(result) = result else {
                        panic!("Expected register for checked enum field")
                    };
                    let Value::Reg(value) = value else {
                        panic!("Expected register for checked enum field source")
                    };
                    let payload = schema.variants[*variant_index]
                        .payload
                        .as_ref()
                        .expect("verified multi-field enum variant has a payload product");
                    let LogicalType::EnumFields { fields } = payload else {
                        unreachable!("verified checked enum field uses a product schema")
                    };
                    fields
                        .get(*field_index)
                        .expect("verified checked enum field index is in bounds");
                    let layout = self.profile_enum_storage_layout(schema);
                    let enum_type = layout.enum_llvm_type_with(&Self::struct_type_to_llvm);
                    let lane = layout
                        .payload_lane(*variant_index)
                        .expect("verified multi-field enum variant has a lane");
                    let payload_type = layout
                        .lane_llvm_type(lane, &Self::struct_type_to_llvm)
                        .expect("verified multi-field enum payload has a physical type");
                    let extracted_payload = self.fresh_reg();
                    llvm_ir.push_str(&format!(
                        "  %{extracted_payload} = extractvalue {enum_type} %reg{value}, {lane}\n"
                    ));
                    llvm_ir.push_str(&format!(
                        "  %reg{result} = extractvalue {payload_type} %{extracted_payload}, {field_index}\n"
                    ));
                }
                Inst::CheckedEnumDispatch {
                    value,
                    schema,
                    targets,
                } => {
                    let Value::Reg(value) = value else {
                        panic!("Expected register for checked enum dispatch")
                    };
                    let first = targets
                        .first()
                        .expect("verified enum dispatch has a target");
                    let layout = self.profile_enum_storage_layout(schema);
                    let tag_lane = layout.tag_lane();
                    let tag_type = layout
                        .lane_llvm_type(tag_lane, &Self::struct_type_to_llvm)
                        .expect("enum storage has a tag lane");
                    let tag = if layout.is_unit() {
                        format!("%reg{value}")
                    } else {
                        let enum_type = layout.enum_llvm_type_with(&Self::struct_type_to_llvm);
                        let tag = self.fresh_reg();
                        llvm_ir.push_str(&format!(
                            "  %{tag} = extractvalue {enum_type} %reg{value}, {tag_lane}\n"
                        ));
                        format!("%{tag}")
                    };
                    llvm_ir.push_str(&format!("  switch {tag_type} {tag}, label %{first} [\n"));
                    for (index, target) in targets.iter().enumerate().skip(1) {
                        llvm_ir.push_str(&format!("    {tag_type} {index}, label %{target}\n"));
                    }
                    llvm_ir.push_str("  ]\n");
                }
                Inst::VecAlloca { .. }
                | Inst::VecPush { .. }
                | Inst::VecPop { .. }
                | Inst::VecLength { .. }
                | Inst::VecCapacity { .. }
                | Inst::VecAccess { .. }
                | Inst::VecInit { .. }
                | Inst::ArrayLength { .. }
                | Inst::ArrayAccess { .. }
                | Inst::EnumDiscriminant { .. }
                | Inst::EnumVariantData { .. }
                | Inst::EnumConstruct { .. } => {}
            }
        }

        if !instructions.is_empty()
            && !instructions.last().is_some_and(|instruction| {
                matches!(
                    instruction,
                    Inst::Return(_)
                        | Inst::Jump(_)
                        | Inst::Branch { .. }
                        | Inst::CheckedEnumDispatch { .. }
                )
            })
        {
            match return_llvm_type {
                "void" => llvm_ir.push_str("  ret void\n"),
                "double" => llvm_ir.push_str("  ret double 0x0000000000000000\n"),
                "i1" => llvm_ir.push_str("  ret i1 false\n"),
                "i64" => llvm_ir.push_str("  ret i64 0\n"),
                _ => llvm_ir.push_str("  ret i32 0\n"),
            }
        }
    }

    fn generate_function_call(
        &mut self,
        llvm_ir: &mut String,
        function: &str,
        arguments: &[Value],
        result: &Option<Value>,
        function_defs: &HashMap<String, FunctionDef>,
    ) {
        let checked_signature = self
            .checked_metadata
            .as_ref()
            .and_then(|metadata| metadata.functions.get(function))
            .map(|metadata| metadata.signature.clone());
        let (param_defs, return_type) = match function_defs.get(function) {
            Some(FunctionDef::Legacy {
                parameters,
                return_type,
                ..
            }) => (
                parameters
                    .iter()
                    .map(|(name, ty)| (name.clone(), self.type_to_llvm(ty).to_string()))
                    .collect(),
                return_type
                    .as_ref()
                    .map(|return_type| self.type_to_llvm(return_type).to_string()),
            ),
            Some(FunctionDef::Checked {
                parameters, result, ..
            }) => (
                parameters
                    .iter()
                    .map(|(name, ty)| (name.clone(), self.profile_logical_type_to_llvm(ty)))
                    .collect(),
                Some(self.profile_logical_type_to_llvm(result)),
            ),
            None => (Vec::new(), None),
        };

        let mut args = Vec::new();
        for (i, arg) in arguments.iter().enumerate() {
            let target_type = checked_signature
                .as_ref()
                .and_then(|signature| signature.parameters.get(i))
                .map(|(_, ty)| self.profile_logical_type_to_llvm(ty))
                .or_else(|| param_defs.get(i).map(|(_name, ty)| ty.clone()))
                .unwrap_or_else(|| "double".to_string());
            let arg_val = self.cast_value_for_call_arg(llvm_ir, arg, &target_type);
            args.push(format!("{} {}", target_type, arg_val));
        }
        let args_str = args.join(", ");

        let return_llvm_type = checked_signature
            .as_ref()
            .map(|signature| self.profile_logical_type_to_llvm(&signature.result))
            .unwrap_or_else(|| {
                if let Some(ret) = return_type {
                    ret
                } else if result.is_some() {
                    "double".to_string()
                } else {
                    "void".to_string()
                }
            });
        let llvm_function = Self::llvm_function_symbol(function);

        if let Some(result_reg) = result {
            let result_str = match result_reg {
                Value::Reg(r) => format!("reg{}", r),
                _ => panic!("Expected register for call result"),
            };

            match return_llvm_type.as_str() {
                "double" => llvm_ir.push_str(&format!(
                    "  %{} = call double @{}({})\n",
                    result_str, llvm_function, args_str
                )),
                "i32" => {
                    if self.is_checked_enum_result(result_reg)
                        || self.is_checked_char_result(result_reg)
                        || (self.uses_exact_i32_lane() && self.is_checked_int_result(result_reg))
                    {
                        llvm_ir.push_str(&format!(
                            "  %{} = call i32 @{}({})\n",
                            result_str, llvm_function, args_str
                        ));
                    } else {
                        let call_reg = self.fresh_reg();
                        llvm_ir.push_str(&format!(
                            "  %{} = call i32 @{}({})\n",
                            call_reg, llvm_function, args_str
                        ));
                        llvm_ir.push_str(&format!(
                            "  %{} = sitofp i32 %{} to double\n",
                            result_str, call_reg
                        ));
                    }
                }
                "i64" => {
                    let call_reg = self.fresh_reg();
                    llvm_ir.push_str(&format!(
                        "  %{} = call i64 @{}({})\n",
                        call_reg, llvm_function, args_str
                    ));
                    llvm_ir.push_str(&format!(
                        "  %{} = sitofp i64 %{} to double\n",
                        result_str, call_reg
                    ));
                }
                "i1" => {
                    if self.is_checked_bool_result(result_reg) {
                        llvm_ir.push_str(&format!(
                            "  %{} = call i1 @{}({})\n",
                            result_str, llvm_function, args_str
                        ));
                    } else {
                        let call_reg = self.fresh_reg();
                        llvm_ir.push_str(&format!(
                            "  %{} = call i1 @{}({})\n",
                            call_reg, llvm_function, args_str
                        ));
                        llvm_ir.push_str(&format!(
                            "  %{} = uitofp i1 %{} to double\n",
                            result_str, call_reg
                        ));
                    }
                }
                "void" => {
                    llvm_ir.push_str(&format!("  call void @{}({})\n", llvm_function, args_str));
                    llvm_ir.push_str(&format!(
                        "  %{} = fadd double 0x0000000000000000, 0x0000000000000000\n",
                        result_str
                    ));
                }
                struct_type if Self::is_struct_llvm_type(struct_type) => {
                    llvm_ir.push_str(&format!(
                        "  %{result_str} = call {struct_type} @{llvm_function}({args_str})\n"
                    ));
                }
                array_type if array_type.starts_with('[') => {
                    llvm_ir.push_str(&format!(
                        "  %{result_str} = call {array_type} @{llvm_function}({args_str})\n"
                    ));
                }
                aggregate_type if aggregate_type.starts_with('{') => {
                    llvm_ir.push_str(&format!(
                        "  %{result_str} = call {aggregate_type} @{llvm_function}({args_str})\n"
                    ));
                }
                _ => llvm_ir.push_str(&format!(
                    "  %{} = call double @{}({})\n",
                    result_str, llvm_function, args_str
                )),
            }
        } else {
            llvm_ir.push_str(&format!(
                "  call {} @{}({})\n",
                return_llvm_type, llvm_function, args_str
            ));
        }
    }

    fn cast_value_for_call_arg(
        &mut self,
        llvm_ir: &mut String,
        value: &Value,
        target_type: &str,
    ) -> String {
        match target_type {
            pointer_type if pointer_type.ends_with('*') => match value {
                Value::Reg(register) => format!("%ptr{register}"),
                _ => panic!("verified immutable reference arguments use place identifiers"),
            },
            "double" => self.value_to_string(value),
            "i32" => match value {
                Value::ImmInt(n) => n.to_string(),
                Value::ImmFloat(f) => (*f as i64).to_string(),
                Value::ImmChar(character) => u32::from(*character).to_string(),
                Value::Reg(r) => {
                    if self.is_checked_enum_result(value)
                        || self.is_checked_char_result(value)
                        || (self.uses_exact_i32_lane() && self.is_checked_int_result(value))
                    {
                        format!("%reg{}", r)
                    } else {
                        let tmp = self.fresh_reg();
                        llvm_ir.push_str(&format!("  %{} = fptosi double %reg{} to i32\n", tmp, r));
                        format!("%{}", tmp)
                    }
                }
                Value::ImmString(_) => {
                    panic!("Cannot cast string argument to i32 in function call")
                }
            },
            "i64" => match value {
                Value::ImmInt(n) => n.to_string(),
                Value::ImmFloat(f) => (*f as i64).to_string(),
                Value::ImmChar(_) => panic!("Character values do not use the i64 ABI lane"),
                Value::Reg(r) => {
                    let tmp = self.fresh_reg();
                    llvm_ir.push_str(&format!("  %{} = fptosi double %reg{} to i64\n", tmp, r));
                    format!("%{}", tmp)
                }
                Value::ImmString(_) => {
                    panic!("Cannot cast string argument to i64 in function call")
                }
            },
            "i1" => match value {
                Value::ImmInt(n) => {
                    if *n == 0 {
                        "false".to_string()
                    } else {
                        "true".to_string()
                    }
                }
                Value::ImmFloat(f) => {
                    if *f == 0.0 {
                        "false".to_string()
                    } else {
                        "true".to_string()
                    }
                }
                Value::ImmChar(_) => panic!("Character values do not use the i1 ABI lane"),
                Value::Reg(r) => {
                    if self.is_checked_bool_result(value) {
                        format!("%reg{}", r)
                    } else {
                        let tmp = self.fresh_reg();
                        llvm_ir.push_str(&format!(
                            "  %{} = fcmp one double %reg{}, 0x0000000000000000\n",
                            tmp, r
                        ));
                        format!("%{}", tmp)
                    }
                }
                Value::ImmString(_) => {
                    panic!("Cannot cast string argument to i1 in function call")
                }
            },
            _ => self.value_to_string(value),
        }
    }

    fn emit_return(&mut self, llvm_ir: &mut String, value: &Value, return_llvm_type: &str) {
        if Self::is_struct_llvm_type(return_llvm_type)
            || return_llvm_type.starts_with('[')
            || return_llvm_type.starts_with('{')
        {
            let Value::Reg(register) = value else {
                panic!("verified aggregate return must use an aggregate result register");
            };
            llvm_ir.push_str(&format!("  ret {return_llvm_type} %reg{register}\n"));
            return;
        }
        match return_llvm_type {
            "void" => llvm_ir.push_str("  ret void\n"),
            "double" => {
                llvm_ir.push_str(&format!("  ret double {}\n", self.value_to_string(value)))
            }
            "i64" => match value {
                Value::ImmInt(n) => llvm_ir.push_str(&format!("  ret i64 {}\n", n)),
                Value::ImmFloat(f) => llvm_ir.push_str(&format!("  ret i64 {}\n", *f as i64)),
                Value::ImmChar(_) => panic!("Character values do not use the i64 return lane"),
                Value::Reg(r) => {
                    let tmp = self.fresh_reg();
                    llvm_ir.push_str(&format!("  %{} = fptosi double %reg{} to i64\n", tmp, r));
                    llvm_ir.push_str(&format!("  ret i64 %{}\n", tmp));
                }
                Value::ImmString(_) => panic!("Cannot return string value as i64"),
            },
            "i1" => match value {
                Value::ImmInt(n) => llvm_ir.push_str(&format!(
                    "  ret i1 {}\n",
                    if *n == 0 { "false" } else { "true" }
                )),
                Value::ImmFloat(f) => llvm_ir.push_str(&format!(
                    "  ret i1 {}\n",
                    if *f == 0.0 { "false" } else { "true" }
                )),
                Value::ImmChar(_) => panic!("Character values do not use the i1 return lane"),
                Value::Reg(r) => {
                    if self.is_checked_bool_result(value) {
                        llvm_ir.push_str(&format!("  ret i1 %reg{}\n", r));
                    } else {
                        let tmp = self.fresh_reg();
                        llvm_ir.push_str(&format!(
                            "  %{} = fcmp one double %reg{}, 0x0000000000000000\n",
                            tmp, r
                        ));
                        llvm_ir.push_str(&format!("  ret i1 %{}\n", tmp));
                    }
                }
                Value::ImmString(_) => panic!("Cannot return string value as i1"),
            },
            _ => match value {
                Value::ImmInt(n) => llvm_ir.push_str(&format!("  ret i32 {}\n", n)),
                Value::ImmFloat(f) => llvm_ir.push_str(&format!("  ret i32 {}\n", *f as i64)),
                Value::ImmChar(character) => {
                    llvm_ir.push_str(&format!("  ret i32 {}\n", u32::from(*character)))
                }
                Value::Reg(r) => {
                    if self.is_checked_enum_result(value)
                        || self.is_checked_char_result(value)
                        || (self.uses_exact_i32_lane() && self.is_checked_int_result(value))
                    {
                        llvm_ir.push_str(&format!("  ret i32 %reg{}\n", r));
                    } else {
                        let tmp = self.fresh_reg();
                        llvm_ir.push_str(&format!("  %{} = fptosi double %reg{} to i32\n", tmp, r));
                        llvm_ir.push_str(&format!("  ret i32 %{}\n", tmp));
                    }
                }
                Value::ImmString(_) => panic!("Cannot return string value as i32"),
            },
        }
    }
    fn generate_branch(
        &mut self,
        llvm_ir: &mut String,
        condition: &Value,
        true_label: &str,
        false_label: &str,
    ) {
        let cond_str = self.value_to_string(condition);

        // Check if condition is already a boolean (i1) or needs conversion
        match condition {
            Value::Reg(_) => {
                // Assume it's already a boolean from a comparison operation
                llvm_ir.push_str(&format!(
                    "  br i1 {}, label %{}, label %{}\n",
                    cond_str, true_label, false_label
                ));
            }
            _ => {
                // Convert numeric value to boolean (non-zero is true)
                let bool_reg = self.fresh_reg();
                llvm_ir.push_str(&format!(
                    "  %{} = fcmp one double {}, 0x0000000000000000\n",
                    bool_reg, cond_str
                ));
                llvm_ir.push_str(&format!(
                    "  br i1 %{}, label %{}, label %{}\n",
                    bool_reg, true_label, false_label
                ));
            }
        }
    }

    #[allow(dead_code)]
    fn generate_phi_node(
        &mut self,
        llvm_ir: &mut String,
        result_reg: &str,
        incoming_values: &[(Value, String)],
    ) {
        // Generate phi node for variable updates in loops and control flow
        let mut phi_str = format!("  %{} = phi double ", result_reg);

        for (i, (value, label)) in incoming_values.iter().enumerate() {
            if i > 0 {
                phi_str.push_str(", ");
            }
            phi_str.push_str(&format!("[ {}, %{} ]", self.value_to_string(value), label));
        }

        phi_str.push('\n');
        llvm_ir.push_str(&phi_str);
    }

    #[allow(dead_code)]
    fn generate_loop_structure(
        &mut self,
        llvm_ir: &mut String,
        loop_header: &str,
        loop_body: &str,
        loop_exit: &str,
        condition: Option<&Value>,
    ) {
        // Generate basic loop structure with proper basic blocks

        // Jump to loop header
        llvm_ir.push_str(&format!("  br label %{}\n", loop_header));

        // Loop header block
        llvm_ir.push_str(&format!("{}:\n", loop_header));

        if let Some(cond) = condition {
            // Conditional loop (while/for)
            self.generate_branch(llvm_ir, cond, loop_body, loop_exit);
        } else {
            // Infinite loop
            llvm_ir.push_str(&format!("  br label %{}\n", loop_body));
        }

        // Loop body block
        llvm_ir.push_str(&format!("{}:\n", loop_body));
    }

    #[allow(dead_code)]
    fn generate_if_else_structure(
        &mut self,
        llvm_ir: &mut String,
        condition: &Value,
        then_label: &str,
        else_label: Option<&str>,
        merge_label: &str,
    ) {
        // Generate if-else structure with proper basic blocks
        let false_label = else_label.unwrap_or(merge_label);

        // Generate conditional branch
        self.generate_branch(llvm_ir, condition, then_label, false_label);

        // Then block
        llvm_ir.push_str(&format!("{}:\n", then_label));
    }

    fn generate_print_call(
        &mut self,
        llvm_ir: &mut String,
        format_string: &str,
        arguments: &[Value],
        is_println: bool,
    ) {
        // Convert Rust-style `{}` placeholders to `printf` specifiers from argument kinds.
        let processed_format = self.process_format_string_with_args(format_string, arguments);

        // Add newline for println
        let final_format = if is_println {
            format!("{}\n", processed_format)
        } else {
            processed_format
        };

        // Create format string as a local array (simplified approach)
        let format_len = final_format.len() + 1; // +1 for null terminator
        let format_const_reg = self.fresh_reg();

        // Allocate space for format string
        llvm_ir.push_str(&format!(
            "  %{} = alloca [{}  x i8], align 1\n",
            format_const_reg, format_len
        ));

        // Create the format string literal with proper escaping
        let escaped_format = self.escape_for_llvm(&final_format);
        llvm_ir.push_str(&format!(
            "  store [{}  x i8] c\"{}\\00\", [{}  x i8]* %{}, align 1\n",
            format_len, escaped_format, format_len, format_const_reg
        ));

        // Get pointer to format string
        let format_ptr = self.fresh_reg();
        llvm_ir.push_str(&format!(
            "  %{} = getelementptr inbounds [{}  x i8], [{}  x i8]* %{}, i64 0, i64 0\n",
            format_ptr, format_len, format_len, format_const_reg
        ));

        // Generate printf call
        let mut printf_args = format!("i8* %{}", format_ptr);

        for arg in arguments {
            match arg {
                Value::ImmString(s) => {
                    let arg_ptr = self.emit_stack_string_literal(llvm_ir, s);
                    printf_args.push_str(", i8* ");
                    printf_args.push_str(&arg_ptr);
                }
                _ => {
                    // Keep numeric varargs typed as LLVM doubles. The target backend
                    // owns ABI classification, including the Windows x64 requirement
                    // to duplicate variadic floating values into both XMM and general-
                    // purpose registers.
                    printf_args.push_str(", double ");
                    if self.is_checked_bool_result(arg) {
                        let converted = self.fresh_reg();
                        llvm_ir.push_str(&format!(
                            "  %{} = uitofp i1 {} to double\n",
                            converted,
                            self.bool_value_to_string(arg)
                        ));
                        printf_args.push_str(&format!("%{}", converted));
                    } else {
                        printf_args.push_str(&self.value_to_string(arg));
                    }
                }
            }
        }

        // Call printf
        llvm_ir.push_str(&format!("  call i32 (i8*, ...) @printf({})\n", printf_args));
    }

    fn escape_for_llvm(&self, input: &str) -> String {
        // Escape special characters for LLVM string literals
        input
            .replace("\\", "\\\\")
            .replace("\"", "\\\"")
            .replace("\n", "\\0A")
            .replace("\t", "\\09")
            .replace("\r", "\\0D")
    }

    fn process_format_string(&self, format_string: &str, arg_count: usize) -> String {
        // Keep legacy tests and helper calls by treating all placeholders as numeric.
        let numeric_args = vec![Value::ImmInt(0); arg_count];
        self.process_format_string_with_args(format_string, &numeric_args)
    }

    fn process_format_string_with_args(&self, format_string: &str, arguments: &[Value]) -> String {
        // Convert Rust-style {} placeholders to printf-style format specifiers.
        let mut result = String::new();
        let mut chars = format_string.chars().peekable();
        let mut placeholder_count = 0;

        while let Some(ch) = chars.next() {
            if ch == '{' {
                if let Some(&'}') = chars.peek() {
                    chars.next(); // consume '}'
                    if placeholder_count < arguments.len() {
                        let specifier = match arguments.get(placeholder_count) {
                            Some(Value::ImmString(_)) => "%s",
                            _ => "%g",
                        };
                        result.push_str(specifier);
                        placeholder_count += 1;
                    } else {
                        result.push_str("{}"); // Keep original if no corresponding argument
                    }
                } else {
                    result.push(ch);
                }
            } else if ch == '\\' {
                // Handle escape sequences
                if let Some(&next_ch) = chars.peek() {
                    match next_ch {
                        'n' => {
                            chars.next();
                            result.push_str("\\n");
                        }
                        't' => {
                            chars.next();
                            result.push_str("\\t");
                        }
                        'r' => {
                            chars.next();
                            result.push_str("\\r");
                        }
                        '\\' => {
                            chars.next();
                            result.push_str("\\\\");
                        }
                        '"' => {
                            chars.next();
                            result.push_str("\\\"");
                        }
                        _ => {
                            result.push(ch);
                        }
                    }
                } else {
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    fn emit_stack_string_literal(&mut self, llvm_ir: &mut String, text: &str) -> String {
        let len = text.len() + 1; // +1 for null terminator
        let const_reg = self.fresh_reg();
        llvm_ir.push_str(&format!(
            "  %{} = alloca [{}  x i8], align 1\n",
            const_reg, len
        ));
        let escaped = self.escape_for_llvm(text);
        llvm_ir.push_str(&format!(
            "  store [{}  x i8] c\"{}\\00\", [{}  x i8]* %{}, align 1\n",
            len, escaped, len, const_reg
        ));
        let ptr_reg = self.fresh_reg();
        llvm_ir.push_str(&format!(
            "  %{} = getelementptr inbounds [{}  x i8], [{}  x i8]* %{}, i64 0, i64 0\n",
            ptr_reg, len, len, const_reg
        ));
        format!("%{}", ptr_reg)
    }

    fn generate_printf_declaration(&mut self, llvm_ir: &mut String) {
        // Generate printf declaration at module level
        llvm_ir.push_str("declare i32 @printf(i8*, ...)\n\n");
    }
}

/// Verifies private IR and emits LLVM through the checked code-generation boundary.
pub fn try_generate_code<I>(ir: I) -> Result<String, CodeGenerationError>
where
    I: Into<CheckedIr>,
{
    CodeGenerator::new().try_generate_code(ir)
}

/// Crate-owned bridge for the physical lane selected by a validated language profile.
pub(crate) fn try_generate_code_with_profile<I>(
    ir: I,
    language_profile: LanguageProfile,
) -> Result<String, CodeGenerationError>
where
    I: Into<CheckedIr>,
{
    CodeGenerator::new().try_generate_code_with_profile(ir, language_profile)
}

/// Crate-owned canonical bridge for the profile whose backend lane is available
/// only after descriptor admission and verifier-backed authentication.
pub(crate) fn try_generate_code_with_authenticated_profile<I>(
    ir: I,
    language_profile: LanguageProfile,
    authenticated: AuthenticatedResolvedProfileProgram,
) -> Result<String, CodeGenerationError>
where
    I: Into<CheckedIr>,
{
    CodeGenerator::new().try_generate_code_with_authenticated_profile(
        ir,
        language_profile,
        &authenticated,
    )
}

/// Legacy unchecked function retained for backward compatibility.
#[deprecated(note = "unchecked compatibility API; use try_generate_code")]
#[allow(deprecated)]
pub fn generate_code(ir_functions: HashMap<String, Function>) -> String {
    let mut generator = CodeGenerator::new();
    generator.generate_code(ir_functions)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::approx_constant)]

    use super::*;
    use crate::ir::{BlockMetadata, Function, FunctionMetadata, FunctionSignature, Inst, Value};
    use crate::resolved_profile_shape::ResolvedProfileShapeId;
    use crate::semantic_analyzer::SemanticAnalyzer;
    use crate::{IrGenerator, parse_with_locations, try_tokenize_with_locations};
    use std::collections::{BTreeMap, HashMap};

    fn checked_ir_from_source(source: &str) -> CheckedIr {
        let tokens = try_tokenize_with_locations(source, None).expect("test source should lex");
        let ast = parse_with_locations(tokens).expect("test source should parse");
        IrGenerator::new()
            .try_generate_ir(ast)
            .expect("test source should enter verified checked IR")
    }

    fn authenticated_checked_ir_from_source(
        source: &str,
    ) -> (CheckedIr, AuthenticatedResolvedProfileProgram) {
        let tokens = try_tokenize_with_locations(source, None).expect("test source should lex");
        let ast = parse_with_locations(tokens).expect("test source should parse");
        let (_, analyzed, resolved) = SemanticAnalyzer::new()
            .analyze_with_resolved_profile(ast)
            .expect("test source should pass semantic finalization");
        validate_resolved_language_profile(&resolved, LanguageProfile::ExactI32RecordResultV0)
            .expect("test source should satisfy the resolved profile");
        let checked = IrGenerator::new()
            .try_generate_ir(analyzed)
            .expect("test source should enter verified checked IR");
        let authenticated = crate::resolved_profile_authentication::authenticate_resolved_profile(
            resolved, &checked,
        )
        .expect("test descriptor should authenticate against checked IR");
        (checked, authenticated)
    }

    #[test]
    fn exact_record_result_profile_requires_authentication_after_reverification() {
        let checked = checked_ir_from_source("fn main() -> int { return 91; }");
        let error = CodeGenerator::new()
            .try_generate_code_with_profile(checked, LanguageProfile::ExactI32RecordResultV0)
            .expect_err("direct checked codegen may not claim the authenticated profile");
        assert!(matches!(
            &error,
            CodeGenerationError::LanguageProfileContract {
                profile: LanguageProfile::ExactI32RecordResultV0,
                ..
            }
        ));
        assert!(
            error
                .to_string()
                .contains("missing verifier-authenticated resolved profile token")
        );

        let corrupt = HashMap::from([(
            "main".to_string(),
            Function {
                name: "main".to_string(),
                body: vec![Inst::Return(Value::Reg(99))],
                next_reg: 0,
                next_ptr: 0,
            },
        )]);
        let error = CodeGenerator::new()
            .try_generate_code_with_profile(corrupt, LanguageProfile::ExactI32RecordResultV0)
            .expect_err("corrupt IR must fail before the token guard");
        assert!(matches!(error, CodeGenerationError::IrVerification(_)));
    }

    #[test]
    fn exact_record_result_profile_rejects_authenticated_shape_identity_corruption() {
        let source = "fn identity(value: int) -> int { return value; } fn main() -> int { return identity(91); }";
        let (checked, authenticated) = authenticated_checked_ir_from_source(source);
        let llvm = CodeGenerator::new()
            .try_generate_code_with_authenticated_profile(
                checked.clone(),
                LanguageProfile::ExactI32RecordResultV0,
                &authenticated,
            )
            .expect("canonical authenticated scalar program should lower");
        assert!(llvm.contains("define i32 @identity(i32 %aero.arg.value)"));
        assert!(!llvm.contains("double"));

        let mut corrupt = authenticated;
        let coverage = corrupt
            .coverage
            .iter_mut()
            .find(|observation| {
                matches!(
                    observation.coverage,
                    ResolvedProfileAuthenticationCoverage::Authenticated(_)
                )
            })
            .expect("fixture should contain authenticated coverage");
        coverage.coverage = ResolvedProfileAuthenticationCoverage::Authenticated(
            ResolvedProfileShapeId(usize::MAX),
        );
        let first = CodeGenerator::new()
            .try_generate_code_with_authenticated_profile(
                checked.clone(),
                LanguageProfile::ExactI32RecordResultV0,
                &corrupt,
            )
            .expect_err("corrupt authenticated shape identity must fail");
        let second = CodeGenerator::new()
            .try_generate_code_with_authenticated_profile(
                checked,
                LanguageProfile::ExactI32RecordResultV0,
                &corrupt,
            )
            .expect_err("corrupt authenticated shape identity must fail deterministically");
        assert_eq!(first.to_string(), second.to_string());
        assert_eq!(format!("{first:?}"), format!("{second:?}"));
        assert!(matches!(
            &first,
            CodeGenerationError::LanguageProfileContract {
                profile: LanguageProfile::ExactI32RecordResultV0,
                ..
            }
        ));
        assert!(
            first
                .to_string()
                .contains("authenticated coverage does not authorize re-verified subject")
        );
    }

    #[test]
    fn recursive_copy_data_types_lower_to_exact_private_llvm_types() {
        let leaf = LogicalType::Struct {
            name: "Leaf".to_string(),
            fields: vec![
                LogicalType::Float,
                LogicalType::Array {
                    element: Box::new(LogicalType::Bool),
                    count: 0,
                },
            ],
        };
        let nested_tuple = LogicalType::Tuple {
            elements: vec![LogicalType::Bool, leaf.clone()],
        };
        let recursive = LogicalType::Array {
            element: Box::new(LogicalType::Tuple {
                elements: vec![
                    LogicalType::Int,
                    LogicalType::Array {
                        element: Box::new(nested_tuple),
                        count: 2,
                    },
                    leaf,
                ],
            }),
            count: 0,
        };

        assert_eq!(
            CodeGenerator::logical_type_to_llvm(&LogicalType::Int),
            "i32"
        );
        assert_eq!(
            CodeGenerator::logical_type_to_llvm(&recursive),
            "[0 x { double, [2 x { i1, %aero.struct.Leaf }], %aero.struct.Leaf }]"
        );
        assert_eq!(
            CodeGenerator::render_copy_data_layout(&recursive),
            "[0 x { double, [2 x { i1, %aero.struct.Leaf }], %aero.struct.Leaf }]"
        );
    }

    #[test]
    fn exact_i32_profile_maps_the_signed_index_count_boundary() {
        let mut generator = CodeGenerator::new();
        generator.language_profile = LanguageProfile::ExactI32ArrayV0;
        let admitted_count = i32::MAX as usize;
        let array = LogicalType::Array {
            element: Box::new(LogicalType::Int),
            count: admitted_count,
        };
        assert_eq!(
            generator.profile_copy_data_type_to_llvm(&array),
            format!("[{admitted_count} x i32]")
        );
        let too_large = LogicalType::Array {
            element: Box::new(LogicalType::Int),
            count: admitted_count + 1,
        };
        assert!(!LanguageProfile::ExactI32ArrayV0.admits_exact_i32_array(&too_large));
        assert_eq!(
            generator.profile_copy_data_type_to_llvm(&too_large),
            format!("[{} x double]", admitted_count + 1)
        );

        generator.current_function = Some("probe".to_string());
        generator.checked_metadata = Some(IrMetadata {
            functions: BTreeMap::from([(
                "probe".to_string(),
                FunctionMetadata {
                    signature: FunctionSignature {
                        parameters: Vec::new(),
                        result: LogicalType::Void,
                    },
                    results: BTreeMap::from([(ResultId(7), LogicalType::Int)]),
                    places: BTreeMap::new(),
                    blocks: vec![BlockMetadata {
                        label: "entry".to_string(),
                        reachable: true,
                        successors: Vec::new(),
                    }],
                },
            )]),
        });
        let mut llvm = String::new();
        let index = generator.checked_copy_array_index_to_i64_operand(
            &mut llvm,
            &Value::Reg(7),
            admitted_count,
            9,
        );
        assert!(llvm.contains("icmp sge i32 %reg7, 0"));
        assert!(llvm.contains("icmp slt i32 %reg7, 2147483647"));
        assert!(llvm.contains("sext i32 %reg7 to i64"));
        assert!(index.starts_with('%'));
    }

    #[test]
    fn exact_i32_profile_lowers_mutable_flat_array_production_through_the_copydata_mapper() {
        let checked = checked_ir_from_source(
            "fn produce() -> [int; 2] { let mut output: [i32; 2] = [1, 2]; output[0] = 3; return output; } fn main() -> int { return 0; }",
        );

        let llvm = CodeGenerator::new()
            .try_generate_code_with_profile(checked, LanguageProfile::ExactI32ArrayV0)
            .expect("exact mutable flat-array checked IR should reach LLVM");

        for anchor in [
            "define [2 x i32] @produce()",
            "alloca [2 x i32], align 8",
            "store [2 x i32]",
            "getelementptr inbounds [2 x i32]",
            "store i32 3",
            "load [2 x i32]",
            "ret [2 x i32]",
        ] {
            assert!(
                llvm.contains(anchor),
                "exact mutable-array LLVM omitted `{anchor}`:\n{llvm}"
            );
        }
        for forbidden in ["double", "fptosi", "sitofp"] {
            assert!(
                !llvm.contains(forbidden),
                "exact mutable-array LLVM leaked `{forbidden}`:\n{llvm}"
            );
        }
    }

    #[test]
    fn exact_i32_profile_rejects_verified_mutable_owner_topology_corruption() {
        let cases = [
            (
                "nested mutable array owner",
                "fn main() -> int { let mut output: [[int; 1]; 1] = [[1]]; return 0; }",
            ),
            (
                "non-Int mutable array owner",
                "fn main() -> int { let mut output: [bool; 2] = [1 < 2, 2 < 3]; return 0; }",
            ),
        ];

        for (label, source) in cases {
            let error = CodeGenerator::new()
                .try_generate_code_with_profile(
                    checked_ir_from_source(source),
                    LanguageProfile::ExactI32ArrayV0,
                )
                .expect_err("unsupported mutable owner topology must fail before emission");
            assert!(
                matches!(
                    &error,
                    CodeGenerationError::LanguageProfileContract {
                        profile: LanguageProfile::ExactI32ArrayV0,
                        ..
                    }
                ),
                "{label} was rejected at the wrong boundary: {error}"
            );
            assert!(
                error
                    .to_string()
                    .contains("instruction outside the exact i32 fixed-array profile"),
                "{label} escaped the mutable-owner topology guard: {error}"
            );
        }
    }

    #[test]
    fn exact_i32_profile_rejects_verified_legacy_array_instructions_before_emission() {
        let raw = HashMap::from([(
            "main".to_string(),
            Function {
                name: "main".to_string(),
                body: vec![
                    Inst::AllocaArray {
                        result: Value::Reg(0),
                        elem_type: "double".to_string(),
                        count: 2,
                    },
                    Inst::GetElementPtr {
                        result: Value::Reg(1),
                        base: Value::Reg(0),
                        index: Value::ImmInt(0),
                        elem_type: "[2 x double]".to_string(),
                    },
                    Inst::Store(Value::Reg(1), Value::ImmInt(7)),
                    Inst::Return(Value::ImmInt(0)),
                ],
                next_reg: 2,
                next_ptr: 2,
            },
        )]);

        let error = CodeGenerator::new()
            .try_generate_code_with_profile(raw, LanguageProfile::ExactI32ArrayV0)
            .expect_err("exact profile must reject the legacy double-array route");
        assert!(matches!(
            error,
            CodeGenerationError::LanguageProfileContract {
                profile: LanguageProfile::ExactI32ArrayV0,
                ..
            }
        ));
        assert!(error.to_string().contains("legacy `alloca array`"));
    }

    #[test]
    fn exact_i32_profile_rejects_verified_excluded_operations_and_array_roles() {
        let cases = [
            (
                "division",
                vec![
                    Inst::Div(Value::Reg(0), Value::ImmInt(4), Value::ImmInt(2)),
                    Inst::Return(Value::Reg(0)),
                ],
                "profile-excluded division instruction",
            ),
            (
                "whole-array owned-place assignment",
                vec![
                    Inst::CheckedCopyStructArrayAlloca {
                        result: Value::Reg(0),
                        element: LogicalType::Int,
                        count: 2,
                    },
                    Inst::CheckedCopyStructArrayElementPtr {
                        result: Value::Reg(1),
                        base: Value::Reg(0),
                        index: Value::ImmInt(0),
                        element: LogicalType::Int,
                        count: 2,
                    },
                    Inst::Store(Value::Reg(1), Value::ImmInt(7)),
                    Inst::CheckedCopyStructArrayElementPtr {
                        result: Value::Reg(2),
                        base: Value::Reg(0),
                        index: Value::ImmInt(1),
                        element: LogicalType::Int,
                        count: 2,
                    },
                    Inst::Store(Value::Reg(2), Value::ImmInt(8)),
                    Inst::Load(Value::Reg(3), Value::Reg(0)),
                    Inst::CheckedMutableOwnedPlaceAlloca {
                        result: Value::Reg(4),
                        name: "values".to_string(),
                        ty: LogicalType::Array {
                            element: Box::new(LogicalType::Int),
                            count: 2,
                        },
                    },
                    Inst::Store(Value::Reg(4), Value::Reg(3)),
                    Inst::CheckedOwnedPlaceAssignment {
                        target: Value::Reg(4),
                        value: Value::Reg(3),
                        ty: LogicalType::Array {
                            element: Box::new(LogicalType::Int),
                            count: 2,
                        },
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
                "profile-excluded whole-array owned-place assignment",
            ),
        ];

        for (name, body, expected) in cases {
            let raw = HashMap::from([(
                "main".to_string(),
                Function {
                    name: "main".to_string(),
                    body,
                    next_reg: 2,
                    next_ptr: 2,
                },
            )]);
            let error = CodeGenerator::new()
                .try_generate_code_with_profile(raw, LanguageProfile::ExactI32ArrayV0)
                .err()
                .unwrap_or_else(|| panic!("{name} unexpectedly reached exact LLVM"));
            assert!(
                matches!(
                    &error,
                    CodeGenerationError::LanguageProfileContract {
                        profile: LanguageProfile::ExactI32ArrayV0,
                        ..
                    }
                ),
                "{name} was rejected at the wrong boundary: {error}"
            );
            assert!(error.to_string().contains(expected), "{name}: {error}");
        }
    }

    #[test]
    fn stable_scalar_profile_does_not_claim_the_exact_copydata_array_lane() {
        let array = LogicalType::Array {
            element: Box::new(LogicalType::Int),
            count: 2,
        };
        let mut generator = CodeGenerator::new();
        generator.language_profile = LanguageProfile::StableScalarV0;
        assert_eq!(
            generator.profile_copy_data_type_to_llvm(&array),
            "[2 x double]"
        );
        assert_eq!(
            generator.profile_copy_data_type_to_llvm(&LogicalType::Int),
            "double"
        );
    }

    #[test]
    fn recursive_named_struct_schemas_are_collected_from_nested_checked_types() {
        let leaf = LogicalType::Struct {
            name: "Leaf".to_string(),
            fields: vec![LogicalType::Bool, LogicalType::Int],
        };
        let outer_fields = vec![
            LogicalType::Tuple {
                elements: vec![
                    LogicalType::Array {
                        element: Box::new(leaf.clone()),
                        count: 3,
                    },
                    LogicalType::Float,
                ],
            },
            LogicalType::Array {
                element: Box::new(LogicalType::Tuple {
                    elements: vec![LogicalType::Bool, leaf.clone()],
                }),
                count: 0,
            },
        ];
        let outer = LogicalType::Struct {
            name: "Outer".to_string(),
            fields: outer_fields.clone(),
        };
        let instructions = vec![Inst::CheckedTupleAlloca {
            result: Value::Reg(0),
            element_types: vec![outer, LogicalType::Int],
        }];
        let mut schemas = BTreeMap::new();

        CodeGenerator::collect_struct_schemas(&instructions, &mut schemas);

        assert_eq!(
            schemas.get("Leaf"),
            Some(&vec![LogicalType::Bool, LogicalType::Int])
        );
        assert_eq!(schemas.get("Outer"), Some(&outer_fields));
        assert_eq!(schemas.len(), 2);
    }

    #[test]
    fn test_function_definition_generation() {
        let mut generator = CodeGenerator::new();

        // Create a simple function: fn add(a: i32, b: i32) -> i32 { return a + b; }
        let function = Function {
            name: "add".to_string(),
            body: vec![Inst::FunctionDef {
                name: "add".to_string(),
                parameters: vec![
                    ("a".to_string(), "i32".to_string()),
                    ("b".to_string(), "i32".to_string()),
                ],
                return_type: Some("i32".to_string()),
                body: vec![
                    Inst::Alloca(Value::Reg(0), "a".to_string()),
                    Inst::Alloca(Value::Reg(1), "b".to_string()),
                    Inst::Load(Value::Reg(2), Value::Reg(0)),
                    Inst::Load(Value::Reg(3), Value::Reg(1)),
                    Inst::Add(Value::Reg(4), Value::Reg(2), Value::Reg(3)),
                    Inst::Return(Value::Reg(4)),
                ],
            }],
            next_reg: 5,
            next_ptr: 2,
        };

        let mut functions = HashMap::new();
        functions.insert("add".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that function signature is correct
        assert!(llvm_ir.contains("define i32 @add(i32 %aero.arg.a, i32 %aero.arg.b)"));

        // Parameters are lowered to local double slots
        assert!(llvm_ir.contains("%ptr0 = alloca double"));
        assert!(llvm_ir.contains("%ptr1 = alloca double"));
        assert!(llvm_ir.contains("sitofp i32 %aero.arg.a to double"));
        assert!(llvm_ir.contains("sitofp i32 %aero.arg.b to double"));

        // Check that function has entry block
        assert!(llvm_ir.contains("entry:"));
    }

    #[test]
    fn test_function_call_generation() {
        let mut generator = CodeGenerator::new();

        // Create a function that calls another function
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Call {
                    function: "add".to_string(),
                    arguments: vec![Value::ImmInt(5), Value::ImmInt(3)],
                    result: Some(Value::Reg(0)),
                },
                Inst::Return(Value::Reg(0)),
            ],
            next_reg: 1,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that function call is generated
        assert!(llvm_ir.contains("call double @add"));
        assert!(llvm_ir.contains("double 0x4014000000000000")); // 5.0 in hex
        assert!(llvm_ir.contains("double 0x4008000000000000")); // 3.0 in hex
    }

    #[test]
    fn test_typed_function_call_uses_signature_and_converts_result() {
        let mut generator = CodeGenerator::new();

        let typed_helper = Function {
            name: "typed_helper".to_string(),
            body: vec![Inst::FunctionDef {
                name: "typed_helper".to_string(),
                parameters: vec![
                    ("x".to_string(), "i32".to_string()),
                    ("y".to_string(), "i32".to_string()),
                ],
                return_type: Some("i32".to_string()),
                body: vec![
                    Inst::Alloca(Value::Reg(0), "x".to_string()),
                    Inst::Alloca(Value::Reg(1), "y".to_string()),
                    Inst::Load(Value::Reg(2), Value::Reg(0)),
                    Inst::Load(Value::Reg(3), Value::Reg(1)),
                    Inst::Add(Value::Reg(4), Value::Reg(2), Value::Reg(3)),
                    Inst::Return(Value::Reg(4)),
                ],
            }],
            next_reg: 5,
            next_ptr: 2,
        };

        let main = Function {
            name: "main".to_string(),
            body: vec![
                Inst::Call {
                    function: "typed_helper".to_string(),
                    arguments: vec![Value::ImmInt(1), Value::ImmInt(2)],
                    result: Some(Value::Reg(0)),
                },
                Inst::Return(Value::Reg(0)),
            ],
            next_reg: 1,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("typed_helper".to_string(), typed_helper);
        functions.insert("main".to_string(), main);

        let llvm_ir = generator.generate_code(functions);

        assert!(llvm_ir.contains("call i32 @typed_helper(i32 1, i32 2)"));
        assert!(llvm_ir.contains("sitofp i32 %"));
    }

    #[test]
    fn test_void_function_generation() {
        let mut generator = CodeGenerator::new();

        // Create a void function: fn print_hello() { }
        let function = Function {
            name: "print_hello".to_string(),
            body: vec![Inst::FunctionDef {
                name: "print_hello".to_string(),
                parameters: vec![],
                return_type: None,
                body: vec![Inst::Print {
                    format_string: "Hello, World!".to_string(),
                    arguments: vec![],
                }],
            }],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("print_hello".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that void function signature is correct
        assert!(llvm_ir.contains("define void @print_hello()"));

        // Check that print statement is generated with printf call
        assert!(llvm_ir.contains("call i32 (i8*, ...) @printf"));
    }

    #[test]
    fn test_print_generation() {
        let mut generator = CodeGenerator::new();

        // Create a function with print statement
        let function = Function {
            name: "main".to_string(),
            body: vec![Inst::Print {
                format_string: "Hello, World!".to_string(),
                arguments: vec![],
            }],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that printf declaration is present
        assert!(llvm_ir.contains("declare i32 @printf(i8*, ...)"));

        // Check that print call is generated
        assert!(llvm_ir.contains("call i32 (i8*, ...) @printf"));
        assert!(llvm_ir.contains("Hello, World!"));
    }

    #[test]
    fn test_println_generation() {
        let mut generator = CodeGenerator::new();

        // Create a function with println statement
        let function = Function {
            name: "main".to_string(),
            body: vec![Inst::Println {
                format_string: "Hello, World!".to_string(),
                arguments: vec![],
            }],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that printf declaration is present
        assert!(llvm_ir.contains("declare i32 @printf(i8*, ...)"));

        // Check that println call is generated with newline
        assert!(llvm_ir.contains("call i32 (i8*, ...) @printf"));
        assert!(llvm_ir.contains("Hello, World!\\0A"));
    }

    #[test]
    fn test_print_with_arguments() {
        let mut generator = CodeGenerator::new();

        // Create a function with print statement and arguments
        let function = Function {
            name: "main".to_string(),
            body: vec![Inst::Print {
                format_string: "Value: {}".to_string(),
                arguments: vec![Value::ImmInt(42)],
            }],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that format string is converted to printf style
        assert!(llvm_ir.contains("Value: %g"));

        // Check that argument is passed
        assert!(llvm_ir.contains("double 0x4045000000000000")); // 42.0 in hex
    }

    #[test]
    fn test_print_with_string_argument_uses_percent_s() {
        let mut generator = CodeGenerator::new();

        let function = Function {
            name: "main".to_string(),
            body: vec![Inst::Println {
                format_string: "Hello, {}".to_string(),
                arguments: vec![Value::ImmString("Aero".to_string())],
            }],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        assert!(llvm_ir.contains("Hello, %s\\0A"));
        assert!(llvm_ir.contains("call i32 (i8*, ...) @printf(i8*"));
        assert!(llvm_ir.contains(", i8* %"));
    }

    #[test]
    fn test_comparison_operations() {
        let mut generator = CodeGenerator::new();

        // Create a function with comparison operations
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::ICmp {
                    op: "eq".to_string(),
                    result: Value::Reg(0),
                    left: Value::ImmInt(5),
                    right: Value::ImmInt(5),
                },
                Inst::FCmp {
                    op: "olt".to_string(),
                    result: Value::Reg(1),
                    left: Value::ImmFloat(3.14),
                    right: Value::ImmFloat(4.0),
                },
            ],
            next_reg: 2,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that comparison operations are generated
        assert!(llvm_ir.contains("icmp eq i32"));
        assert!(llvm_ir.contains("fcmp olt double"));
    }

    #[test]
    fn test_logical_operations() {
        let mut generator = CodeGenerator::new();

        // Create a function with logical operations
        let function = Function {
            name: "main".to_string(),
            body: vec![
                Inst::And {
                    result: Value::Reg(0),
                    left: Value::Reg(1),
                    right: Value::Reg(2),
                },
                Inst::Or {
                    result: Value::Reg(3),
                    left: Value::Reg(4),
                    right: Value::Reg(5),
                },
                Inst::Not {
                    result: Value::Reg(6),
                    operand: Value::Reg(7),
                },
            ],
            next_reg: 8,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that logical operations are generated
        assert!(llvm_ir.contains("and i1"));
        assert!(llvm_ir.contains("or i1"));
        assert!(llvm_ir.contains("xor i1"));
    }

    #[test]
    fn test_unary_operations() {
        let mut generator = CodeGenerator::new();

        // Create a function with unary operations
        let function = Function {
            name: "main".to_string(),
            body: vec![Inst::Neg {
                result: Value::Reg(0),
                operand: Value::ImmFloat(5.0),
            }],
            next_reg: 1,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that negation operation is generated
        assert!(llvm_ir.contains("fsub double 0.0"));
    }

    #[test]
    fn test_format_string_processing() {
        let generator = CodeGenerator::new();

        // Test format string conversion
        let result = generator.process_format_string("Hello {}", 1);
        assert_eq!(result, "Hello %g");

        let result = generator.process_format_string("Values: {} and {}", 2);
        assert_eq!(result, "Values: %g and %g");

        let result = generator.process_format_string("No placeholders", 0);
        assert_eq!(result, "No placeholders");

        // Test with more placeholders than arguments
        let result = generator.process_format_string("Too many: {} {} {}", 1);
        assert_eq!(result, "Too many: %g {} {}");
    }

    #[test]
    fn test_escape_for_llvm() {
        let generator = CodeGenerator::new();

        // Test LLVM escaping
        let result = generator.escape_for_llvm("Hello\nWorld");
        assert_eq!(result, "Hello\\0AWorld");

        let result = generator.escape_for_llvm("Quote: \"test\"");
        assert_eq!(result, "Quote: \\\"test\\\"");

        let result = generator.escape_for_llvm("Tab\tSeparated");
        assert_eq!(result, "Tab\\09Separated");
    }

    #[test]
    fn test_complex_print_with_multiple_arguments() {
        let mut generator = CodeGenerator::new();

        // Create a function with complex print statement
        let function = Function {
            name: "main".to_string(),
            body: vec![Inst::Println {
                format_string: "Sum: {} + {} = {}".to_string(),
                arguments: vec![Value::ImmInt(5), Value::ImmInt(3), Value::ImmInt(8)],
            }],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that format string is converted correctly
        assert!(llvm_ir.contains("Sum: %g + %g = %g"));

        // Check that all arguments are passed
        assert!(llvm_ir.contains("double 0x4014000000000000")); // 5.0
        assert!(llvm_ir.contains("double 0x4008000000000000")); // 3.0
        assert!(llvm_ir.contains("double 0x4020000000000000")); // 8.0
    }

    #[test]
    fn test_type_to_llvm_conversion() {
        let generator = CodeGenerator::new();

        assert_eq!(generator.type_to_llvm("i32"), "i32");
        assert_eq!(generator.type_to_llvm("i64"), "i64");
        assert_eq!(generator.type_to_llvm("f32"), "float");
        assert_eq!(generator.type_to_llvm("f64"), "double");
        assert_eq!(generator.type_to_llvm("bool"), "i1");
        assert_eq!(generator.type_to_llvm("unknown"), "double"); // fallback
    }

    #[test]
    fn test_function_call_without_result() {
        let mut generator = CodeGenerator::new();

        // Create a function that calls a void function
        let function = Function {
            name: "main".to_string(),
            body: vec![Inst::Call {
                function: "print_hello".to_string(),
                arguments: vec![],
                result: None,
            }],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("main".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that void function call is generated
        assert!(llvm_ir.contains("call void @print_hello()"));
    }

    #[test]
    fn test_print_operation_generation() {
        let mut generator = CodeGenerator::new();

        // Create a function with print operation
        let function = Function {
            name: "test_print".to_string(),
            body: vec![Inst::Print {
                format_string: "Hello, {}!".to_string(),
                arguments: vec![Value::ImmInt(42)],
            }],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("test_print".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that printf call is generated
        assert!(llvm_ir.contains("call i32 (i8*, ...) @printf"));
        assert!(llvm_ir.contains("Hello, %g!")); // Format string should be processed
        assert!(llvm_ir.contains("getelementptr inbounds")); // String constant access
    }

    #[test]
    fn test_print_with_multiple_arguments() {
        let mut generator = CodeGenerator::new();

        // Create a function with print operation with multiple arguments
        let function = Function {
            name: "test_multi_print".to_string(),
            body: vec![Inst::Print {
                format_string: "Values: {}, {}, {}".to_string(),
                arguments: vec![Value::ImmInt(1), Value::ImmFloat(3.14), Value::Reg(5)],
            }],
            next_reg: 6,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("test_multi_print".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that printf call is generated with multiple arguments
        assert!(llvm_ir.contains("call i32 (i8*, ...) @printf"));
        assert!(llvm_ir.contains("Values: %g, %g, %g"));
        assert!(llvm_ir.contains("double 0x3FF0000000000000")); // 1.0 in hex
        assert!(llvm_ir.contains("double 0x40091EB851EB851F")); // 3.14 in hex
        assert!(llvm_ir.contains("double %reg5"));
    }

    #[test]
    fn test_println_vs_print_generation() {
        let mut generator = CodeGenerator::new();

        // Test print (without newline)
        let mut llvm_ir = String::new();
        generator.generate_print_call(&mut llvm_ir, "Hello", &[], false);
        assert!(llvm_ir.contains("Hello"));
        assert!(!llvm_ir.contains("\\n"));

        // Test println (with newline)
        let mut llvm_ir = String::new();
        generator.generate_print_call(&mut llvm_ir, "Hello", &[], true);
        assert!(llvm_ir.contains("Hello\\0A"));
    }

    #[test]
    fn test_enhanced_operations_generation() {
        let mut generator = CodeGenerator::new();

        // Create a comprehensive test with I/O, comparisons, logical, and unary operations
        let function = Function {
            name: "test_all_enhanced_ops".to_string(),
            body: vec![
                // Test comparison operations
                Inst::ICmp {
                    op: "eq".to_string(),
                    result: Value::Reg(0),
                    left: Value::ImmInt(5),
                    right: Value::ImmInt(5),
                },
                Inst::FCmp {
                    op: "ogt".to_string(),
                    result: Value::Reg(1),
                    left: Value::ImmFloat(3.14),
                    right: Value::ImmFloat(2.0),
                },
                // Test logical operations
                Inst::And {
                    result: Value::Reg(2),
                    left: Value::Reg(0),
                    right: Value::Reg(1),
                },
                Inst::Or {
                    result: Value::Reg(3),
                    left: Value::Reg(0),
                    right: Value::Reg(1),
                },
                Inst::Not {
                    result: Value::Reg(4),
                    operand: Value::Reg(0),
                },
                // Test unary operations
                Inst::Neg {
                    result: Value::Reg(5),
                    operand: Value::ImmFloat(-5.5),
                },
                // Test I/O operations
                Inst::Print {
                    format_string: "Results: {}, {}, {}".to_string(),
                    arguments: vec![Value::Reg(2), Value::Reg(3), Value::Reg(5)],
                },
                Inst::Println {
                    format_string: "Test completed!".to_string(),
                    arguments: vec![],
                },
                Inst::Return(Value::ImmInt(0)),
            ],
            next_reg: 6,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("test_all_enhanced_ops".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that all operations are generated
        assert!(llvm_ir.contains("icmp eq i32"));
        assert!(llvm_ir.contains("fcmp ogt double"));
        assert!(llvm_ir.contains("and i1"));
        assert!(llvm_ir.contains("or i1"));
        assert!(llvm_ir.contains("xor i1"));
        assert!(llvm_ir.contains("fsub double 0.0"));
        assert!(llvm_ir.contains("call i32 (i8*, ...) @printf"));
        assert!(llvm_ir.contains("Results: %g, %g, %g"));
        assert!(llvm_ir.contains("Test completed!\\0A"));
    }

    #[test]
    fn test_comprehensive_io_and_operations() {
        let mut generator = CodeGenerator::new();

        // Create a function with enhanced operations
        let function = Function {
            name: "test_enhanced_ops".to_string(),
            body: vec![
                // Comparison operations
                Inst::ICmp {
                    op: "eq".to_string(),
                    result: Value::Reg(0),
                    left: Value::ImmInt(5),
                    right: Value::ImmInt(5),
                },
                Inst::FCmp {
                    op: "ogt".to_string(),
                    result: Value::Reg(1),
                    left: Value::ImmFloat(3.14),
                    right: Value::ImmFloat(2.71),
                },
                // Logical operations
                Inst::And {
                    result: Value::Reg(2),
                    left: Value::Reg(0),
                    right: Value::Reg(1),
                },
                Inst::Or {
                    result: Value::Reg(3),
                    left: Value::Reg(0),
                    right: Value::Reg(1),
                },
                Inst::Not {
                    result: Value::Reg(4),
                    operand: Value::Reg(0),
                },
                // Unary operations
                Inst::Neg {
                    result: Value::Reg(5),
                    operand: Value::ImmFloat(42.0),
                },
            ],
            next_reg: 6,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("test_enhanced_ops".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that all operations are generated correctly
        assert!(llvm_ir.contains("icmp eq i32"));
        assert!(llvm_ir.contains("fcmp ogt double"));
        assert!(llvm_ir.contains("and i1"));
        assert!(llvm_ir.contains("or i1"));
        assert!(llvm_ir.contains("xor i1"));
        assert!(llvm_ir.contains("fsub double 0.0"));
    }

    #[test]
    fn test_escape_sequence_processing() {
        let generator = CodeGenerator::new();

        // Test various escape sequences
        let result =
            generator.process_format_string("Tab:\\t Newline:\\n Quote:\\\" Backslash:\\\\", 0);
        assert_eq!(result, "Tab:\\t Newline:\\n Quote:\\\" Backslash:\\\\");

        // Test carriage return
        let result = generator.process_format_string("CR:\\r", 0);
        assert_eq!(result, "CR:\\r");
    }

    #[test]
    fn test_print_with_no_arguments() {
        let mut generator = CodeGenerator::new();

        // Create a function with print operation with no arguments
        let function = Function {
            name: "test_no_args".to_string(),
            body: vec![Inst::Print {
                format_string: "Hello, World!".to_string(),
                arguments: vec![],
            }],
            next_reg: 0,
            next_ptr: 0,
        };

        let mut functions = HashMap::new();
        functions.insert("test_no_args".to_string(), function);

        let llvm_ir = generator.generate_code(functions);

        // Check that printf call is generated with just format string
        assert!(llvm_ir.contains("call i32 (i8*, ...) @printf(i8*"));
        assert!(llvm_ir.contains("Hello, World!"));
    }
}

#[test]
fn test_legacy_function_without_definition() {
    let mut generator = CodeGenerator::new();

    // Create a legacy function without FunctionDef instruction (like main)
    let function = Function {
        name: "main".to_string(),
        body: vec![Inst::Return(Value::ImmInt(0))],
        next_reg: 0,
        next_ptr: 0,
    };

    let mut functions = HashMap::new();
    functions.insert("main".to_string(), function);

    let llvm_ir = generator.generate_code(functions);

    // Check that legacy function is handled correctly
    assert!(llvm_ir.contains("define i32 @main()"));
    assert!(llvm_ir.contains("entry:"));
    assert!(llvm_ir.contains("ret i32"));
}

#[test]
fn test_branch_generation() {
    let mut generator = CodeGenerator::new();

    // Create a function with conditional branch
    let function = Function {
        name: "test_branch".to_string(),
        body: vec![
            Inst::FCmp {
                op: "ogt".to_string(),
                result: Value::Reg(0),
                left: Value::ImmFloat(5.0),
                right: Value::ImmFloat(3.0),
            },
            Inst::Branch {
                condition: Value::Reg(0),
                true_label: "then_block".to_string(),
                false_label: "else_block".to_string(),
            },
            Inst::Label("then_block".to_string()),
            Inst::Return(Value::ImmInt(1)),
            Inst::Label("else_block".to_string()),
            Inst::Return(Value::ImmInt(0)),
        ],
        next_reg: 1,
        next_ptr: 0,
    };

    let mut functions = HashMap::new();
    functions.insert("test_branch".to_string(), function);

    let llvm_ir = generator.generate_code(functions);

    // Check that branch is generated correctly
    assert!(llvm_ir.contains("fcmp ogt double"));
    assert!(llvm_ir.contains("br i1 %reg0, label %then_block, label %else_block"));
    assert!(llvm_ir.contains("then_block:"));
    assert!(llvm_ir.contains("else_block:"));
}

#[test]
fn test_jump_and_label_generation() {
    let mut generator = CodeGenerator::new();

    // Create a function with unconditional jump
    let function = Function {
        name: "test_jump".to_string(),
        body: vec![
            Inst::Jump("target_label".to_string()),
            Inst::Label("target_label".to_string()),
            Inst::Return(Value::ImmInt(42)),
        ],
        next_reg: 0,
        next_ptr: 0,
    };

    let mut functions = HashMap::new();
    functions.insert("test_jump".to_string(), function);

    let llvm_ir = generator.generate_code(functions);

    // Check that jump and label are generated correctly
    assert!(llvm_ir.contains("br label %target_label"));
    assert!(llvm_ir.contains("target_label:"));
}

#[test]
fn test_comparison_operations() {
    let mut generator = CodeGenerator::new();

    // Create a function with various comparison operations
    let function = Function {
        name: "test_comparisons".to_string(),
        body: vec![
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(0),
                left: Value::ImmInt(5),
                right: Value::ImmInt(5),
            },
            Inst::FCmp {
                op: "olt".to_string(),
                result: Value::Reg(1),
                left: Value::ImmFloat(std::f64::consts::PI),
                right: Value::ImmFloat(std::f64::consts::E),
            },
            Inst::Return(Value::Reg(0)),
        ],
        next_reg: 2,
        next_ptr: 0,
    };

    let mut functions = HashMap::new();
    functions.insert("test_comparisons".to_string(), function);

    let llvm_ir = generator.generate_code(functions);

    // Check that comparisons are generated correctly
    assert!(llvm_ir.contains("icmp eq i32"));
    assert!(llvm_ir.contains("fcmp olt double"));
}

#[test]
fn test_logical_operations() {
    let mut generator = CodeGenerator::new();

    // Create a function with logical operations
    let function = Function {
        name: "test_logical".to_string(),
        body: vec![
            Inst::And {
                result: Value::Reg(0),
                left: Value::Reg(1),
                right: Value::Reg(2),
            },
            Inst::Or {
                result: Value::Reg(3),
                left: Value::Reg(4),
                right: Value::Reg(5),
            },
            Inst::Not {
                result: Value::Reg(6),
                operand: Value::Reg(7),
            },
            Inst::Return(Value::Reg(0)),
        ],
        next_reg: 8,
        next_ptr: 0,
    };

    let mut functions = HashMap::new();
    functions.insert("test_logical".to_string(), function);

    let llvm_ir = generator.generate_code(functions);

    // Check that logical operations are generated correctly
    assert!(llvm_ir.contains("and i1 %reg1, %reg2"));
    assert!(llvm_ir.contains("or i1 %reg4, %reg5"));
    assert!(llvm_ir.contains("xor i1 %reg7, true"));
}

#[test]
fn test_loop_structure_generation() {
    let mut generator = CodeGenerator::new();

    // Test the loop structure helper method
    let mut llvm_ir = String::new();
    let condition = Value::Reg(0);

    generator.generate_loop_structure(
        &mut llvm_ir,
        "loop_header",
        "loop_body",
        "loop_exit",
        Some(&condition),
    );

    // Check that loop structure is generated correctly
    assert!(llvm_ir.contains("br label %loop_header"));
    assert!(llvm_ir.contains("loop_header:"));
    assert!(llvm_ir.contains("loop_body:"));
    assert!(llvm_ir.contains("br i1 %reg0, label %loop_body, label %loop_exit"));
}

#[test]
fn test_infinite_loop_structure() {
    let mut generator = CodeGenerator::new();

    // Test infinite loop structure
    let mut llvm_ir = String::new();

    generator.generate_loop_structure(&mut llvm_ir, "loop_header", "loop_body", "loop_exit", None);

    // Check that infinite loop structure is generated correctly
    assert!(llvm_ir.contains("br label %loop_header"));
    assert!(llvm_ir.contains("loop_header:"));
    assert!(llvm_ir.contains("br label %loop_body"));
    assert!(llvm_ir.contains("loop_body:"));
}
