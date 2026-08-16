use crate::ast::{AstNode, Expression, Statement, Type};
use crate::binding_annotation::{
    BindingAnnotationDisposition, BindingContractKind, classify_binding_annotation,
    is_legacy_numeric_array_annotation, is_statically_empty_fixed_array,
};
use crate::builtin_carrier_contract::{normalize_builtin_carriers, private_result_int_int_name};
use crate::byte_buffer_source_contract::{
    BYTES_NEW, ByteBufferIntrinsic, byte_buffer_type_declaration_diagnostic,
    classify_byte_buffer_intrinsic_call, contains_byte_buffer_annotation,
    is_byte_buffer_annotation, is_reserved_byte_buffer_intrinsic, result_context_diagnostic,
};
use crate::byte_input_source_contract::{
    STDIN_READ_BYTE, classify_byte_input_intrinsic_call, is_direct_byte_input_result_initializer,
    is_reserved_byte_input_intrinsic,
    result_context_diagnostic as byte_input_result_context_diagnostic,
};
use crate::closure_contract::unsupported_closure_diagnostic;
use crate::const_contract::normalize_primitive_consts;
use crate::enum_match_contract::{
    EnumError, EnumExecutionContext, EnumFunctionContract, EnumRegistry,
};
use crate::function_call_contract::{
    FunctionCallDisposition, FunctionCallFacts, FunctionCallParameter, FunctionCallTarget,
    FunctionCallUse, classify_function_call, unsupported_function_call_diagnostic,
};
use crate::generic_function_contract::valid_generic_aware_function_symbol;
use crate::ir::{EnumSchema, Function, Inst, LogicalType, PlaceId, Value};
use crate::ir_verifier::PlaceTypeHints;
use crate::local_reference::{
    LocalReferenceDisposition, LocalReferenceSourceFacts, MutableReferenceAssignmentDisposition,
    MutableReferenceAssignmentFacts, ReferenceCallDisposition, ReferenceCallSourceMode,
    ReferenceFunctionContract, ReferenceFunctionDisposition, ReferencePointeeContext,
    classify_enum_match_dereference, classify_local_borrow_with_enums, classify_local_dereference,
    classify_local_reference_annotation_with_enums,
    classify_mutable_reference_assignment_with_enums, classify_mutable_reference_binding,
    classify_reference_call_with_enums, classify_reference_function_with_enums,
    classify_reference_pointee_type, reference_call_source_modes,
    validate_enum_reference_match_result,
};
use crate::method_call_contract::{
    IntrinsicMethodDisposition, IntrinsicMethodLowering, IntrinsicMethodPhase,
    classify_intrinsic_method,
};
use crate::ownership_flow::{
    ConditionalOwnershipArm, LOOP_OWNERSHIP_FIXED_POINT_LIMIT, LoopControlSnapshots,
    LoopOwnershipDisposition, LoopOwnershipEdge, LoopOwnershipEdgeKind, LoopOwnershipKind,
    OwnershipFlowDisposition, block_reaches_merge, classify_conditional_ownership,
    classify_loop_ownership, classify_owned_consumption_paths,
    live_mutable_owner_immutable_enum_loan_edge_diagnostic, maybe_moved_diagnostic,
    statement_reaches_merge,
};
use crate::primitive_contract::PrimitiveKind;
use crate::scalar_assignment::{
    CopyProjectionIndex, CopyProjectionStep, OwnedPlaceAssignmentDisposition,
    OwnedPlaceAssignmentTargetFacts, ProjectedCopyDataAssignmentDisposition,
    ProjectedCopyDataPlaceContract, ProjectedCopyDataPlaceDisposition, ProjectedCopyDataPlaceUse,
    classify_owned_place_assignment, classify_projected_copydata_assignment,
    classify_projected_copydata_assignment_after_admission,
    classify_projected_copydata_place_after_admission,
    projected_copydata_assignment_array_selectors, projected_copydata_place_array_selectors,
    resolve_owned_place_logical_type,
};
use crate::specialization_contract::normalize_copydata_specializations;
use crate::static_string_equality::{
    StaticStringEqualityDisposition, classify_static_string_equality,
};
use crate::struct_contract::{
    CopyArrayIndexDisposition, CopyFunctionContract, StructContractError, StructExecutionContext,
    StructRegistry,
};
use crate::tuple_contract::{
    CopyTupleContract, TupleBindingValidationError, TupleContractDisposition,
    TupleExecutionContext, classify_copy_tuple_elements, classify_tuple_projection,
    validate_tuple_binding,
};
use crate::types::{OwnershipState, Ty, needs_promotion};
use crate::use_import_contract::unsupported_name_import_diagnostic;
use std::collections::{BTreeMap, HashMap};
use std::fmt;

#[derive(Debug)]
pub enum IrGenerationError {
    Admission(String),
    Verification(crate::ir_verifier::IrVerificationError),
}

impl fmt::Display for IrGenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Admission(message) => formatter.write_str(message),
            Self::Verification(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for IrGenerationError {}

impl From<crate::ir_verifier::IrVerificationError> for IrGenerationError {
    fn from(error: crate::ir_verifier::IrVerificationError) -> Self {
        Self::Verification(error)
    }
}

#[derive(Clone)]
struct AdmissionBinding {
    ty: Ty,
    mutable: bool,
    initialized: bool,
    ownership: OwnershipState,
    callable: bool,
    static_string: Option<String>,
}

type AdmissionLoopControl = LoopControlSnapshots<HashMap<String, AdmissionBinding>>;

struct AdmissionTopLevelFunction {
    result: Ty,
    arity: Option<usize>,
    parameter_types: Option<Vec<Ty>>,
}

struct AdmissionProgram {
    functions: HashMap<String, AdmissionTopLevelFunction>,
    enum_functions: HashMap<String, EnumFunctionContract>,
    reference_functions: HashMap<String, ReferenceFunctionContract>,
    structs: StructRegistry,
    enums: EnumRegistry,
    byte_buffer_source_enabled: bool,
    byte_input_source_enabled: bool,
}

#[derive(Clone)]
struct GeneratedByteBufferOwner {
    name: String,
    place: Value,
    live: bool,
}

const STRUCT_ADMISSION_BINDING: &str = "\0aero.checked.struct.context";

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExpressionUse {
    Value,
    Binding,
    ByteInputResultBinding,
    PrintArgument,
    Discarded,
    MatchArm,
}

enum LogicalLoweringTask {
    Evaluate(Expression),
    Combine(crate::ast::LogicalOp),
}

#[derive(Clone)]
struct GeneratedScopeSnapshot {
    bindings: HashMap<String, (Value, Ty)>,
    immutable_owned_enum_places: HashMap<String, Value>,
    mutable_owned_enum_places: HashMap<String, Value>,
    mutable_owner_immutable_enum_reference_sources: HashMap<u32, (Value, EnumSchema)>,
}

#[derive(Clone, Copy)]
enum StatementLoopKind {
    While,
    For,
    Loop,
}

struct StatementLoopLabels {
    header: String,
    body: Option<String>,
    continue_target: String,
    exit: String,
}

#[derive(Clone)]
struct ProjectedCallReferenceSource {
    root: Value,
    source: Value,
    root_type: LogicalType,
    pointee: LogicalType,
    mutable: bool,
}

pub struct IrGenerator {
    functions: HashMap<String, Function>,
    #[allow(dead_code)]
    current_function_name: String,
    next_reg: u32,
    next_ptr: u32,
    symbol_table: HashMap<String, (Value, Ty)>, // Track both pointer and type
    mutable_reference_sources: HashMap<u32, Value>,
    projected_call_reference_sources: HashMap<u32, ProjectedCallReferenceSource>,
    mutable_owner_immutable_enum_reference_sources: HashMap<u32, (Value, EnumSchema)>,
    immutable_owned_enum_places: HashMap<String, Value>,
    mutable_owned_enum_places: HashMap<String, Value>,
    function_return_types: HashMap<String, Ty>,
    copy_function_contracts: HashMap<String, CopyFunctionContract>,
    enum_function_contracts: HashMap<String, EnumFunctionContract>,
    reference_function_contracts: HashMap<String, ReferenceFunctionContract>,
    loop_label_stack: Vec<(String, String)>, // Stack of (continue_target, loop_exit) labels
    checked_mode: bool,
    checked_place_hints: PlaceTypeHints,
    struct_registry: StructRegistry,
    enum_registry: EnumRegistry,
    byte_buffer_source_enabled: bool,
    byte_input_source_enabled: bool,
    generated_byte_buffer_owners: Vec<GeneratedByteBufferOwner>,
}

impl IrGenerator {
    pub fn new() -> Self {
        IrGenerator {
            functions: HashMap::new(),
            current_function_name: String::new(),
            next_reg: 0,
            next_ptr: 0,
            symbol_table: HashMap::new(),
            mutable_reference_sources: HashMap::new(),
            projected_call_reference_sources: HashMap::new(),
            mutable_owner_immutable_enum_reference_sources: HashMap::new(),
            immutable_owned_enum_places: HashMap::new(),
            mutable_owned_enum_places: HashMap::new(),
            function_return_types: HashMap::new(),
            copy_function_contracts: HashMap::new(),
            enum_function_contracts: HashMap::new(),
            reference_function_contracts: HashMap::new(),
            loop_label_stack: Vec::new(),
            checked_mode: false,
            checked_place_hints: BTreeMap::new(),
            struct_registry: StructRegistry::default(),
            enum_registry: EnumRegistry::default(),
            byte_buffer_source_enabled: false,
            byte_input_source_enabled: false,
            generated_byte_buffer_owners: Vec::new(),
        }
    }

    pub(crate) fn new_with_byte_buffer_source() -> Self {
        let mut generator = Self::new();
        generator.byte_buffer_source_enabled = true;
        generator
    }

    pub(crate) fn new_with_byte_input_source() -> Self {
        let mut generator = Self::new_with_byte_buffer_source();
        generator.byte_input_source_enabled = true;
        generator
    }
}

impl Default for IrGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl IrGenerator {
    fn fresh_control_label(&mut self, prefix: &str) -> String {
        let label = format!("{prefix}_{}", self.next_reg);
        self.next_reg += 1;
        label
    }

    fn statement_loop_labels(&mut self, kind: StatementLoopKind) -> StatementLoopLabels {
        let prefix = match kind {
            StatementLoopKind::While => "while",
            StatementLoopKind::For => "for",
            StatementLoopKind::Loop => "loop",
        };
        let header = self.fresh_control_label(&format!("{prefix}_start"));
        let body = (!matches!(kind, StatementLoopKind::Loop))
            .then(|| self.fresh_control_label(&format!("{prefix}_body")));
        let continue_target = if matches!(kind, StatementLoopKind::For) {
            self.fresh_control_label("for_continue")
        } else {
            header.clone()
        };
        let exit = self.fresh_control_label(&format!("{prefix}_end"));
        StatementLoopLabels {
            header,
            body,
            continue_target,
            exit,
        }
    }

    pub fn try_generate_ir(
        &mut self,
        ast: Vec<AstNode>,
    ) -> Result<crate::ir::CheckedIr, IrGenerationError> {
        let ast = normalize_primitive_consts(ast).map_err(IrGenerationError::Admission)?;
        let ast = normalize_copydata_specializations(ast).map_err(IrGenerationError::Admission)?;
        let ast = normalize_builtin_carriers(ast).map_err(IrGenerationError::Admission)?;
        Self::validate_checked_ast(
            &ast,
            self.byte_buffer_source_enabled,
            self.byte_input_source_enabled,
        )?;
        self.struct_registry = StructRegistry::from_top_level_ast(&ast);
        self.enum_registry = EnumRegistry::from_top_level_ast(&ast, &self.struct_registry);
        self.functions.clear();
        self.current_function_name.clear();
        self.next_reg = 0;
        self.next_ptr = 0;
        self.symbol_table.clear();
        self.mutable_reference_sources.clear();
        self.projected_call_reference_sources.clear();
        self.mutable_owner_immutable_enum_reference_sources.clear();
        self.immutable_owned_enum_places.clear();
        self.mutable_owned_enum_places.clear();
        self.function_return_types.clear();
        self.copy_function_contracts.clear();
        self.enum_function_contracts.clear();
        self.reference_function_contracts.clear();
        self.loop_label_stack.clear();
        self.checked_place_hints.clear();
        self.generated_byte_buffer_owners.clear();
        self.checked_mode = true;
        let mut functions = self.generate_ir(ast);
        self.checked_mode = false;
        Self::ensure_checked_main_terminator(&mut functions);
        Self::normalize_checked_place_ids(&mut functions, &mut self.checked_place_hints);
        crate::ir_verifier::verify_ir_with_place_hints(functions, &self.checked_place_hints)
            .map_err(Into::into)
    }

    pub fn generate_ir(&mut self, ast: Vec<AstNode>) -> HashMap<String, Function> {
        let ast = normalize_primitive_consts(ast)
            .unwrap_or_else(|error| panic!("Primitive const normalization failed: {error}"));
        self.function_return_types.clear();
        self.copy_function_contracts.clear();
        self.enum_function_contracts.clear();
        self.reference_function_contracts.clear();
        for node in &ast {
            if let AstNode::Statement(Statement::Function {
                name,
                parameters,
                return_type,
                type_params,
                ..
            }) = node
            {
                let reference_contract = if self.checked_mode {
                    match classify_reference_function_with_enums(
                        name,
                        parameters,
                        return_type.as_ref(),
                        type_params,
                        &self.struct_registry,
                        &self.enum_registry,
                    ) {
                        ReferenceFunctionDisposition::Supported(contract) => Some(contract),
                        ReferenceFunctionDisposition::ExplicitlyRejected(diagnostic) => {
                            panic!(
                                "checked admission accepted rejected reference signature: {diagnostic}"
                            )
                        }
                        ReferenceFunctionDisposition::Preserved => None,
                    }
                } else {
                    None
                };
                let enum_contract = if self.checked_mode && reference_contract.is_none() {
                    self.enum_registry
                        .resolve_function_contract(
                            name,
                            parameters,
                            return_type.as_ref(),
                            type_params,
                            |annotation| {
                                self.struct_registry
                                    .resolve_copy_annotation(annotation)
                                    .map(|contract| (contract.ty, contract.logical_type))
                            },
                        )
                        .expect("checked admission resolved unit-enum function contracts")
                } else {
                    None
                };
                let copy_contract = if self.checked_mode
                    && reference_contract.is_none()
                    && enum_contract.is_none()
                    && type_params.is_empty()
                {
                    self.struct_registry
                        .resolve_copy_function_contract(
                            name,
                            parameters,
                            return_type.as_ref(),
                            type_params,
                        )
                        .expect("checked admission resolved struct Copy function contracts")
                } else {
                    None
                };
                if let Some(contract) = reference_contract {
                    self.function_return_types
                        .insert(name.clone(), contract.result.ty.clone());
                    self.reference_function_contracts
                        .insert(name.clone(), contract);
                } else if let Some(contract) = enum_contract {
                    self.function_return_types
                        .insert(name.clone(), contract.result.ty.clone());
                    self.enum_function_contracts.insert(name.clone(), contract);
                } else if let Some(contract) = copy_contract {
                    self.function_return_types
                        .insert(name.clone(), contract.result.ty.clone());
                    self.copy_function_contracts.insert(name.clone(), contract);
                } else if type_params.is_empty()
                    && parameters.iter().all(|parameter| {
                        Self::numeric_contract_type(&parameter.param_type).is_some()
                    })
                {
                    let contract_return = match return_type {
                        Some(ty) => Self::numeric_contract_type(ty),
                        None => Some(Ty::Void),
                    };
                    if let Some(return_type) = contract_return {
                        self.function_return_types.insert(name.clone(), return_type);
                    }
                }
            }
        }

        let mut main_function = Function {
            name: "main".to_string(),
            body: Vec::new(),
            next_reg: 0,
            next_ptr: 0,
        };

        for node in ast {
            match node {
                AstNode::Statement(stmt) => self.generate_statement_ir(stmt, &mut main_function),
                AstNode::Expression(_) => {
                    eprintln!(
                        "Warning: Top-level expressions are not yet handled in IR generation."
                    );
                }
            }
        }

        main_function.next_reg = self.next_reg;
        main_function.next_ptr = self.next_ptr;
        self.functions.insert("main".to_string(), main_function);
        self.functions.clone()
    }

    fn numeric_contract_type(ty: &Type) -> Option<Ty> {
        match ty {
            Type::Named(name) => PrimitiveKind::from_source_name(name).map(PrimitiveKind::ty),
            _ => None,
        }
    }

    fn build_function_call(&mut self, name: String, arguments: Vec<Value>) -> (Inst, Value, Ty) {
        let function_name = self.resolve_callable_name(&name);
        let return_type = self.function_return_types.get(&function_name).cloned();

        if !self.checked_mode {
            return self.build_quarantined_unchecked_function_call(
                function_name,
                arguments,
                return_type,
            );
        }

        let target = match return_type {
            Some(
                result @ (Ty::Void
                | Ty::Int
                | Ty::Float
                | Ty::Bool
                | Ty::Char
                | Ty::Struct(_)
                | Ty::Array(_, _)
                | Ty::Tuple(_)
                | Ty::Enum(_)),
            ) => FunctionCallTarget::Admitted {
                parameters: None,
                result,
            },
            Some(_) => FunctionCallTarget::DeclaredUnadmitted,
            None => FunctionCallTarget::Missing,
        };
        let return_type = match classify_function_call(FunctionCallFacts {
            name: function_name.clone(),
            target,
            arguments: Vec::new(),
            use_context: FunctionCallUse::Discarded,
        }) {
            FunctionCallDisposition::Supported(contract) => contract.result,
            FunctionCallDisposition::ExplicitlyRejected(diagnostic)
            | FunctionCallDisposition::PreservedContext(diagnostic) => {
                unreachable!(
                    "checked function-call lowering escaped independent admission: {diagnostic}"
                )
            }
        };

        match return_type {
            Ty::Void => (
                Inst::Call {
                    function: function_name,
                    arguments,
                    result: None,
                },
                Value::ImmInt(0),
                Ty::Void,
            ),
            return_type @ (Ty::Int
            | Ty::Float
            | Ty::Bool
            | Ty::Char
            | Ty::Struct(_)
            | Ty::Array(_, _)
            | Ty::Tuple(_)
            | Ty::Enum(_)) => {
                let result_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                (
                    Inst::Call {
                        function: function_name,
                        arguments,
                        result: Some(result_reg.clone()),
                    },
                    result_reg,
                    return_type,
                )
            }
            _ => unreachable!("checked function-call result escaped admitted lowering"),
        }
    }

    fn build_quarantined_unchecked_function_call(
        &mut self,
        function_name: String,
        arguments: Vec<Value>,
        return_type: Option<Ty>,
    ) -> (Inst, Value, Ty) {
        // Compatibility-only unchecked generation predates checked admission. Its
        // scalar fallback is quarantined here and is unreachable from try_generate_ir.
        match return_type {
            Some(Ty::Void) => (
                Inst::Call {
                    function: function_name,
                    arguments,
                    result: None,
                },
                Value::ImmInt(0),
                Ty::Void,
            ),
            Some(
                return_type @ (Ty::Int
                | Ty::Float
                | Ty::Bool
                | Ty::Struct(_)
                | Ty::Array(_, _)
                | Ty::Tuple(_)
                | Ty::Enum(_)),
            ) => {
                let result_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                (
                    Inst::Call {
                        function: function_name,
                        arguments,
                        result: Some(result_reg.clone()),
                    },
                    result_reg,
                    return_type,
                )
            }
            _ => {
                let result_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                (
                    Inst::Call {
                        function: function_name,
                        arguments,
                        result: Some(result_reg.clone()),
                    },
                    result_reg,
                    Ty::Int,
                )
            }
        }
    }

    fn stores_value_directly(ty: &Ty) -> bool {
        matches!(
            ty,
            Ty::String
                | Ty::Array(_, _)
                | Ty::Struct(_)
                | Ty::Tuple(_)
                | Ty::Enum(_)
                | Ty::Reference(_, _)
                | Ty::Vec(_)
                | Ty::Fn(_)
                | Ty::ByteBuffer
        )
    }

    fn admitted_reference_pointee_logical_type(
        &self,
        ty: &Ty,
        context: ReferencePointeeContext,
    ) -> LogicalType {
        classify_reference_pointee_type(ty, context, &self.struct_registry, &self.enum_registry)
            .unwrap_or_else(|message| {
                unreachable!(
                    "checked reference-pointee admission escaped classification: {message}"
                )
            })
            .logical_type
    }

    fn admitted_owned_place_logical_type(&self, ty: &Ty) -> LogicalType {
        resolve_owned_place_logical_type(ty, &self.struct_registry, &self.enum_registry)
            .unwrap_or_else(|message| {
                unreachable!("checked owned-place admission escaped classification: {message}")
            })
    }

    fn is_mutable_owned_enum_place(&self, name: &str, storage: &Value, ty: &Ty) -> bool {
        matches!(ty, Ty::Enum(_))
            && self
                .mutable_owned_enum_places
                .get(name)
                .is_some_and(|place| place == storage)
    }

    fn is_immutable_owned_enum_place(&self, name: &str, storage: &Value, ty: &Ty) -> bool {
        matches!(ty, Ty::Enum(_))
            && self
                .immutable_owned_enum_places
                .get(name)
                .is_some_and(|place| place == storage)
    }

    fn admission_local_reference_source_facts(
        expression: &Expression,
        bindings: &HashMap<String, AdmissionBinding>,
        inside_admitted_function: bool,
    ) -> Option<LocalReferenceSourceFacts> {
        let Expression::Identifier(name) = expression else {
            return None;
        };
        bindings.get(name).map(|binding| LocalReferenceSourceFacts {
            ty: binding.ty.clone(),
            mutable: binding.mutable,
            initialized: binding.initialized,
            local: inside_admitted_function,
            ownership: binding.ownership.clone(),
        })
    }

    fn validate_byte_buffer_owner(
        bindings: &HashMap<String, AdmissionBinding>,
        name: &str,
        require_mutable: bool,
    ) -> Result<(), IrGenerationError> {
        let binding = bindings.get(name).ok_or_else(|| {
            IrGenerationError::Admission(format!(
                "byte-buffer intrinsic owner `{name}` is not a live local binding"
            ))
        })?;
        if binding.ty != Ty::ByteBuffer || !binding.initialized {
            return Err(IrGenerationError::Admission(format!(
                "byte-buffer intrinsic owner `{name}` is not a live ByteBuffer"
            )));
        }
        match binding.ownership {
            OwnershipState::Owned => {}
            OwnershipState::Moved => {
                return Err(IrGenerationError::Admission(format!(
                    "use of moved ByteBuffer owner `{name}` in checked IR"
                )));
            }
            OwnershipState::MaybeMoved => {
                return Err(IrGenerationError::Admission(maybe_moved_diagnostic(name)));
            }
            _ => {
                return Err(IrGenerationError::Admission(format!(
                    "byte-buffer intrinsic owner `{name}` is already borrowed"
                )));
            }
        }
        if require_mutable && !binding.mutable {
            return Err(IrGenerationError::Admission(format!(
                "byte-buffer intrinsic `bytes_push` requires mutable owner `{name}`"
            )));
        }
        Ok(())
    }

    fn validate_byte_buffer_intrinsic(
        name: &str,
        arguments: &[Expression],
        bindings: &HashMap<String, AdmissionBinding>,
        program: &AdmissionProgram,
        inside_impl: bool,
        admit_static_string_equality: bool,
    ) -> Result<Option<Ty>, IrGenerationError> {
        if !program.byte_buffer_source_enabled {
            return Ok(None);
        }
        let Some(call) = classify_byte_buffer_intrinsic_call(name, arguments)
            .map_err(IrGenerationError::Admission)?
        else {
            return Ok(None);
        };
        if call.intrinsic == ByteBufferIntrinsic::New {
            return Err(IrGenerationError::Admission(
                "byte-buffer intrinsic `bytes_new` must directly initialize an explicit ByteBuffer binding"
                    .to_string(),
            ));
        }
        let owner = call
            .owner
            .expect("non-constructor byte-buffer intrinsic retains an owner");
        Self::validate_byte_buffer_owner(
            bindings,
            owner,
            call.intrinsic == ByteBufferIntrinsic::Push,
        )?;
        if let Some(scalar) = call.scalar {
            let actual = Self::validate_expression(
                scalar,
                bindings,
                program,
                ExpressionUse::Value,
                inside_impl,
                admit_static_string_equality,
            )?;
            if actual != Ty::Int {
                return Err(IrGenerationError::Admission(format!(
                    "byte-buffer intrinsic `{name}` scalar argument has type {actual}, expected int"
                )));
            }
        }
        Ok(Some(match call.intrinsic {
            ByteBufferIntrinsic::Push | ByteBufferIntrinsic::Get => {
                let result = private_result_int_int_name();
                program
                    .enums
                    .owned_place_logical_type(&result)
                    .map_err(|_| IrGenerationError::Admission(result_context_diagnostic(name)))?;
                Ty::Enum(result)
            }
            ByteBufferIntrinsic::Length | ByteBufferIntrinsic::Capacity => Ty::Int,
            ByteBufferIntrinsic::New => unreachable!("constructor handled above"),
        }))
    }

    fn validate_byte_input_intrinsic(
        name: &str,
        arguments: &[Expression],
        program: &AdmissionProgram,
        expression_use: ExpressionUse,
        inside_impl: bool,
    ) -> Result<Option<Ty>, IrGenerationError> {
        if !program.byte_input_source_enabled
            || !classify_byte_input_intrinsic_call(name, arguments)
                .map_err(IrGenerationError::Admission)?
        {
            return Ok(None);
        }
        if inside_impl {
            return Err(IrGenerationError::Admission(format!(
                "byte-input intrinsic `{STDIN_READ_BYTE}` requires a direct nongeneric source function body"
            )));
        }
        if expression_use != ExpressionUse::ByteInputResultBinding {
            return Err(IrGenerationError::Admission(
                byte_input_result_context_diagnostic(),
            ));
        }
        let result = private_result_int_int_name();
        program
            .enums
            .owned_place_logical_type(&result)
            .map_err(|_| IrGenerationError::Admission(byte_input_result_context_diagnostic()))?;
        Ok(Some(Ty::Enum(result)))
    }

    fn validate_byte_buffer_binding(
        name: &str,
        mutable: bool,
        type_annotation: Option<&Type>,
        value: Option<&Expression>,
        bindings: &mut HashMap<String, AdmissionBinding>,
        program: &AdmissionProgram,
        direct_byte_buffer_owner_scope: bool,
    ) -> Result<bool, IrGenerationError> {
        if !program.byte_buffer_source_enabled {
            return Ok(false);
        }
        let byte_buffer_initializer = value.is_some_and(|value| match value {
            Expression::FunctionCall { name, .. } => name == BYTES_NEW,
            Expression::Identifier(source) => bindings
                .get(source)
                .is_some_and(|binding| binding.ty == Ty::ByteBuffer),
            _ => false,
        });
        let byte_buffer_annotation = type_annotation.is_some_and(contains_byte_buffer_annotation);
        if !byte_buffer_annotation && !byte_buffer_initializer {
            return Ok(false);
        }
        if !direct_byte_buffer_owner_scope {
            return Err(IrGenerationError::Admission(
                "ByteBuffer owners may be declared or moved only in a direct nongeneric function body outside control-flow topology"
                    .to_string(),
            ));
        }
        if !type_annotation.is_some_and(is_byte_buffer_annotation) {
            return Err(IrGenerationError::Admission(format!(
                "byte-buffer owner `{name}` requires the explicit `ByteBuffer` annotation"
            )));
        }
        let value = value.ok_or_else(|| {
            IrGenerationError::Admission(format!(
                "byte-buffer owner `{name}` must be initialized at declaration"
            ))
        })?;
        match value {
            Expression::FunctionCall {
                name: intrinsic,
                arguments,
            } => {
                let Some(call) = classify_byte_buffer_intrinsic_call(intrinsic, arguments)
                    .map_err(IrGenerationError::Admission)?
                else {
                    return Err(IrGenerationError::Admission(format!(
                        "byte-buffer owner `{name}` requires `bytes_new()` or a direct live owner move"
                    )));
                };
                if call.intrinsic != ByteBufferIntrinsic::New {
                    return Err(IrGenerationError::Admission(format!(
                        "byte-buffer owner `{name}` requires `bytes_new()` or a direct live owner move"
                    )));
                }
            }
            Expression::Identifier(source) => {
                Self::validate_byte_buffer_owner(bindings, source, false)?;
                bindings
                    .get_mut(source)
                    .expect("validated ByteBuffer move source remains in scope")
                    .ownership = OwnershipState::Moved;
            }
            _ => {
                return Err(IrGenerationError::Admission(format!(
                    "byte-buffer owner `{name}` requires `bytes_new()` or a direct live owner move"
                )));
            }
        }
        bindings.insert(
            name.to_string(),
            AdmissionBinding {
                ty: Ty::ByteBuffer,
                mutable,
                initialized: true,
                ownership: OwnershipState::Owned,
                callable: false,
                static_string: None,
            },
        );
        Ok(true)
    }

    fn validate_checked_ast(
        ast: &[AstNode],
        byte_buffer_source_enabled: bool,
        byte_input_source_enabled: bool,
    ) -> Result<(), IrGenerationError> {
        if byte_buffer_source_enabled {
            for node in ast {
                if let AstNode::Statement(statement) = node
                    && let Some(diagnostic) = byte_buffer_type_declaration_diagnostic(statement)
                {
                    return Err(IrGenerationError::Admission(diagnostic));
                }
            }
        }
        let mut program: HashMap<String, AdmissionTopLevelFunction> = HashMap::new();
        let mut enum_functions = HashMap::new();
        let mut reference_functions = HashMap::new();
        let structs = StructRegistry::from_top_level_ast(ast);
        let enums = EnumRegistry::from_top_level_ast(ast, &structs);
        for node in ast {
            if let AstNode::Statement(Statement::Function {
                name,
                parameters,
                return_type,
                type_params,
                ..
            }) = node
            {
                if byte_buffer_source_enabled {
                    if is_reserved_byte_buffer_intrinsic(name) {
                        return Err(IrGenerationError::Admission(format!(
                            "byte-buffer intrinsic name `{name}` is reserved by exact-i32-byte-buffer-v0"
                        )));
                    }
                    if parameters
                        .iter()
                        .any(|parameter| contains_byte_buffer_annotation(&parameter.param_type))
                        || return_type
                            .as_ref()
                            .is_some_and(contains_byte_buffer_annotation)
                    {
                        return Err(IrGenerationError::Admission(format!(
                            "function `{name}` cannot transport ByteBuffer in a parameter or result"
                        )));
                    }
                }
                if byte_input_source_enabled && is_reserved_byte_input_intrinsic(name) {
                    return Err(IrGenerationError::Admission(format!(
                        "byte-input intrinsic name `{name}` is reserved by exact-i32-byte-input-v0"
                    )));
                }
                let reference_contract = match classify_reference_function_with_enums(
                    name,
                    parameters,
                    return_type.as_ref(),
                    type_params,
                    &structs,
                    &enums,
                ) {
                    ReferenceFunctionDisposition::Supported(contract) => Some(contract),
                    ReferenceFunctionDisposition::ExplicitlyRejected(diagnostic) => {
                        return Err(IrGenerationError::Admission(diagnostic));
                    }
                    ReferenceFunctionDisposition::Preserved => None,
                };
                let enum_contract = if reference_contract.is_none() {
                    enums
                        .resolve_function_contract(
                            name,
                            parameters,
                            return_type.as_ref(),
                            type_params,
                            |annotation| {
                                structs
                                    .resolve_copy_annotation(annotation)
                                    .map(|contract| (contract.ty, contract.logical_type))
                            },
                        )
                        .map_err(|error| IrGenerationError::Admission(error.diagnostic()))?
                } else {
                    None
                };
                let copy_contract = if reference_contract.is_none()
                    && enum_contract.is_none()
                    && type_params.is_empty()
                {
                    match structs.resolve_copy_function_contract(
                        name,
                        parameters,
                        return_type.as_ref(),
                        type_params,
                    ) {
                        Ok(contract) => contract,
                        Err(StructContractError::PreserveExistingBehavior) => None,
                        Err(error) => {
                            return Err(IrGenerationError::Admission(error.diagnostic()));
                        }
                    }
                } else {
                    None
                };
                let (result, arity, parameter_types) = if let Some(contract) = reference_contract {
                    reference_functions.insert(name.clone(), contract.clone());
                    let parameter_types = contract
                        .parameters
                        .iter()
                        .map(|(_, parameter)| parameter.ty.clone())
                        .collect::<Vec<_>>();
                    (
                        contract.result.ty,
                        Some(parameter_types.len()),
                        Some(parameter_types),
                    )
                } else if let Some(contract) = enum_contract {
                    enum_functions.insert(name.clone(), contract.clone());
                    let parameter_types = contract.parameter_types();
                    (
                        contract.result.ty,
                        Some(parameter_types.len()),
                        Some(parameter_types),
                    )
                } else if let Some(contract) = copy_contract {
                    let parameter_types = contract
                        .parameters
                        .iter()
                        .map(|(_, parameter)| parameter.ty.clone())
                        .collect::<Vec<_>>();
                    (
                        contract.result.ty,
                        Some(parameter_types.len()),
                        Some(parameter_types),
                    )
                } else {
                    let result = return_type
                        .as_ref()
                        .map(Self::admission_type)
                        .unwrap_or(Ty::Void);
                    let arity = Self::admitted_top_level_arity(
                        name,
                        parameters,
                        return_type.as_ref(),
                        type_params,
                    );
                    let parameter_types = arity.map(|_| {
                        parameters
                            .iter()
                            .map(|parameter| Self::admission_type(&parameter.param_type))
                            .collect()
                    });
                    (result, arity, parameter_types)
                };
                if let Some(existing) = program.get_mut(name) {
                    existing.result = result;
                    existing.arity = None;
                    existing.parameter_types = None;
                } else {
                    program.insert(
                        name.clone(),
                        AdmissionTopLevelFunction {
                            result,
                            arity,
                            parameter_types,
                        },
                    );
                }
            }
        }

        let program = AdmissionProgram {
            functions: program,
            enum_functions,
            reference_functions,
            structs,
            enums,
            byte_buffer_source_enabled,
            byte_input_source_enabled,
        };
        let mut bindings = HashMap::new();
        let mut loop_controls = Vec::<AdmissionLoopControl>::new();
        for node in ast {
            match node {
                AstNode::Statement(statement) => Self::validate_statement(
                    statement,
                    &mut bindings,
                    &program,
                    false,
                    false,
                    false,
                    true,
                    false,
                    &mut loop_controls,
                )?,
                AstNode::Expression(_) => {
                    return Err(IrGenerationError::Admission(
                        "top-level expressions are not admitted in checked IR".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn admitted_symbol(name: &str) -> bool {
        let mut characters = name.chars();
        characters
            .next()
            .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
            && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
    }

    fn admitted_top_level_arity(
        name: &str,
        parameters: &[crate::ast::Parameter],
        return_type: Option<&Type>,
        type_params: &[String],
    ) -> Option<usize> {
        if matches!(name, "main" | "printf")
            || !valid_generic_aware_function_symbol(name, Self::admitted_symbol)
            || !type_params.is_empty()
            || return_type.is_some_and(|return_type| {
                !matches!(
                    Self::admission_type(return_type),
                    Ty::Int | Ty::Float | Ty::Bool | Ty::Char
                )
            })
        {
            return None;
        }

        let mut parameter_names = std::collections::HashSet::new();
        for parameter in parameters {
            if !Self::admitted_symbol(&parameter.name)
                || !parameter_names.insert(parameter.name.as_str())
                || !matches!(
                    Self::admission_type(&parameter.param_type),
                    Ty::Int | Ty::Float | Ty::Bool | Ty::Char
                )
            {
                return None;
            }
        }
        Some(parameters.len())
    }

    fn validate_block(
        block: &crate::ast::Block,
        bindings: &mut HashMap<String, AdmissionBinding>,
        program: &AdmissionProgram,
        inside_loop: bool,
        inside_impl: bool,
        inside_generic_impl: bool,
        direct_byte_buffer_owner_scope: bool,
        loop_controls: &mut Vec<AdmissionLoopControl>,
    ) -> Result<(), IrGenerationError> {
        for statement in &block.statements {
            Self::validate_statement(
                statement,
                bindings,
                program,
                inside_loop,
                inside_impl,
                inside_generic_impl,
                false,
                direct_byte_buffer_owner_scope,
                loop_controls,
            )?;
        }
        if let Some(expression) = &block.expression {
            Self::validate_expression(
                expression,
                bindings,
                program,
                ExpressionUse::Value,
                inside_impl,
                !inside_impl,
            )?;
            Self::apply_enum_expression_ownership(expression, bindings, program, inside_loop)?;
        }
        Ok(())
    }

    fn apply_conditional_admission_join(
        bindings: &mut HashMap<String, AdmissionBinding>,
        entry: &HashMap<String, AdmissionBinding>,
        then_state: &HashMap<String, AdmissionBinding>,
        then_reaches_merge: bool,
        else_state: &HashMap<String, AdmissionBinding>,
        else_reaches_merge: bool,
        inside_loop: bool,
    ) -> Result<(), IrGenerationError> {
        let mut joined = entry.clone();
        let mut names = entry.keys().cloned().collect::<Vec<_>>();
        names.sort();
        for name in names {
            let entry_binding = &entry[&name];
            let then_binding = &then_state[&name];
            let else_binding = &else_state[&name];
            match classify_conditional_ownership(
                &name,
                &entry_binding.ty,
                &entry_binding.ownership,
                &[
                    ConditionalOwnershipArm {
                        state: then_binding.ownership.clone(),
                        reaches_merge: then_reaches_merge,
                    },
                    ConditionalOwnershipArm {
                        state: else_binding.ownership.clone(),
                        reaches_merge: else_reaches_merge,
                    },
                ],
                inside_loop,
            ) {
                OwnershipFlowDisposition::Joined(Some(state)) => {
                    joined
                        .get_mut(&name)
                        .expect("entry admission binding remains available")
                        .ownership = state;
                }
                OwnershipFlowDisposition::Joined(None)
                | OwnershipFlowDisposition::PreserveExistingBehavior => {}
                OwnershipFlowDisposition::ExplicitlyRejected(message) => {
                    return Err(IrGenerationError::Admission(message));
                }
            }
        }
        *bindings = joined;
        Ok(())
    }

    fn summarize_loop_admission(
        initial_header: &HashMap<String, AdmissionBinding>,
        kind: LoopOwnershipKind,
        edges: &[(LoopOwnershipEdgeKind, HashMap<String, AdmissionBinding>)],
    ) -> Result<
        (
            HashMap<String, AdmissionBinding>,
            Option<HashMap<String, AdmissionBinding>>,
        ),
        IrGenerationError,
    > {
        let mut header = initial_header.clone();
        let mut exit = initial_header.clone();
        let has_exit = kind != LoopOwnershipKind::Loop
            || edges
                .iter()
                .any(|(edge, _)| *edge == LoopOwnershipEdgeKind::Break);
        let mut names = initial_header.keys().cloned().collect::<Vec<_>>();
        names.sort();
        for name in names {
            let initial_binding = &initial_header[&name];
            let owner_edges = edges
                .iter()
                .map(|(kind, snapshot)| LoopOwnershipEdge {
                    kind: *kind,
                    state: snapshot[&name].ownership.clone(),
                })
                .collect::<Vec<_>>();
            match classify_loop_ownership(
                &name,
                &initial_binding.ty,
                kind,
                &initial_binding.ownership,
                &owner_edges,
            ) {
                LoopOwnershipDisposition::FixedPoint(summary) => {
                    header
                        .get_mut(&name)
                        .expect("initial admission binding remains in loop header")
                        .ownership = summary.header;
                    if let Some(state) = summary.exit {
                        exit.get_mut(&name)
                            .expect("initial admission binding remains at loop exit")
                            .ownership = state;
                    }
                }
                LoopOwnershipDisposition::PreserveExistingBehavior => {}
                LoopOwnershipDisposition::ExplicitlyRejected(message) => {
                    return Err(IrGenerationError::Admission(message));
                }
            }
        }
        Ok((header, has_exit.then_some(exit)))
    }

    fn admission_ownership_matches(
        left: &HashMap<String, AdmissionBinding>,
        right: &HashMap<String, AdmissionBinding>,
    ) -> bool {
        left.len() == right.len()
            && left.iter().all(|(name, binding)| {
                right
                    .get(name)
                    .is_some_and(|other| binding.ownership == other.ownership)
            })
    }

    fn admission_direct_owned_enum_result_type(
        name: &str,
        bindings: &HashMap<String, AdmissionBinding>,
    ) -> Option<Ty> {
        let binding = bindings.get(name)?;
        (bindings.contains_key(STRUCT_ADMISSION_BINDING)
            && binding.initialized
            && binding.ownership == OwnershipState::Owned
            && matches!(binding.ty, Ty::Enum(_)))
        .then(|| binding.ty.clone())
    }

    fn owned_match_consumption_paths(
        expression: &Expression,
        bindings: &HashMap<String, AdmissionBinding>,
        program: &AdmissionProgram,
    ) -> Option<Vec<Vec<String>>> {
        match expression {
            Expression::Match { arms, .. } => program
                .enums
                .resolve_owned_match_result(
                    arms,
                    &|name| Self::admission_direct_owned_enum_result_type(name, bindings),
                    &|name| {
                        program
                            .enum_functions
                            .get(name)
                            .map(|contract| contract.result.ty.clone())
                    },
                )
                .ok()
                .map(|result| result.consumption_paths),
            Expression::FunctionCall { arguments, .. } => {
                let mut combined = vec![Vec::new()];
                let mut found = false;
                for argument in arguments {
                    let Some(paths) =
                        Self::owned_match_consumption_paths(argument, bindings, program)
                    else {
                        continue;
                    };
                    found = true;
                    combined = combined
                        .into_iter()
                        .flat_map(|prefix| {
                            paths.iter().map(move |path| {
                                let mut combined = prefix.clone();
                                combined.extend(path.iter().cloned());
                                combined
                            })
                        })
                        .collect();
                }
                found.then_some(combined)
            }
            _ => None,
        }
    }

    fn apply_enum_expression_ownership(
        expression: &Expression,
        bindings: &mut HashMap<String, AdmissionBinding>,
        program: &AdmissionProgram,
        inside_loop: bool,
    ) -> Result<(), IrGenerationError> {
        let consumed = program
            .enums
            .consumed_owned_values(
                expression,
                |name| bindings.get(name).map(|binding| binding.ty.clone()),
                |name| {
                    program
                        .enum_functions
                        .get(name)
                        .map(EnumFunctionContract::parameter_types)
                },
            )
            .map_err(|error| IrGenerationError::Admission(error.diagnostic()))?;
        let Some(mut paths) = Self::owned_match_consumption_paths(expression, bindings, program)
        else {
            for name in consumed {
                let binding = bindings.get(&name).ok_or_else(|| {
                    IrGenerationError::Admission(format!(
                        "checked IR has no binding for consumed enum owner `{name}`"
                    ))
                })?;
                let ty = binding.ty.clone();
                let entry = binding.ownership.clone();
                let paths = vec![vec![name.clone()]];
                match classify_owned_consumption_paths(&name, &ty, &entry, &paths, inside_loop) {
                    OwnershipFlowDisposition::Joined(Some(state)) => {
                        bindings
                            .get_mut(&name)
                            .expect("consumed enum owner remains admitted")
                            .ownership = state;
                    }
                    OwnershipFlowDisposition::Joined(None)
                    | OwnershipFlowDisposition::PreserveExistingBehavior => {}
                    OwnershipFlowDisposition::ExplicitlyRejected(message) => {
                        return Err(IrGenerationError::Admission(message));
                    }
                }
            }
            return Ok(());
        };
        for path in &mut paths {
            path.extend(consumed.iter().cloned());
        }
        let mut names = paths
            .iter()
            .flat_map(|path| path.iter().cloned())
            .collect::<Vec<_>>();
        names.sort();
        names.dedup();
        for name in names {
            let binding = bindings.get(&name).ok_or_else(|| {
                IrGenerationError::Admission(format!(
                    "checked IR has no binding for consumed enum owner `{name}`"
                ))
            })?;
            let ty = binding.ty.clone();
            let entry = binding.ownership.clone();
            match classify_owned_consumption_paths(&name, &ty, &entry, &paths, inside_loop) {
                OwnershipFlowDisposition::Joined(Some(state)) => {
                    bindings
                        .get_mut(&name)
                        .expect("consumed enum owner remains admitted")
                        .ownership = state;
                }
                OwnershipFlowDisposition::Joined(None)
                | OwnershipFlowDisposition::PreserveExistingBehavior => {}
                OwnershipFlowDisposition::ExplicitlyRejected(message) => {
                    return Err(IrGenerationError::Admission(message));
                }
            }
        }
        Ok(())
    }

    fn validate_statement(
        statement: &Statement,
        bindings: &mut HashMap<String, AdmissionBinding>,
        program: &AdmissionProgram,
        inside_loop: bool,
        inside_impl: bool,
        inside_generic_impl: bool,
        is_top_level: bool,
        direct_byte_buffer_owner_scope: bool,
        loop_controls: &mut Vec<AdmissionLoopControl>,
    ) -> Result<(), IrGenerationError> {
        match statement {
            Statement::Const { .. } => {
                unreachable!("primitive constants are normalized before checked admission")
            }
            Statement::Let {
                name,
                mutable,
                type_annotation,
                value,
            } => {
                let byte_input_result_initializer = value.as_ref().is_some_and(|value| {
                    is_direct_byte_input_result_initializer(value, type_annotation.as_ref())
                });
                if program.byte_input_source_enabled
                    && value.as_ref().is_some_and(|value| {
                        matches!(value, Expression::FunctionCall { name, arguments }
                            if is_reserved_byte_input_intrinsic(name) && arguments.is_empty())
                    })
                    && !byte_input_result_initializer
                {
                    return Err(IrGenerationError::Admission(
                        byte_input_result_context_diagnostic(),
                    ));
                }
                if Self::validate_byte_buffer_binding(
                    name,
                    *mutable,
                    type_annotation.as_ref(),
                    value.as_ref(),
                    bindings,
                    program,
                    direct_byte_buffer_owner_scope,
                )? {
                    return Ok(());
                }
                let disposition = type_annotation.as_ref().map_or(
                    BindingAnnotationDisposition::PreservedQuarantinedTopology,
                    |annotation| classify_binding_annotation(annotation, value.is_some()),
                );
                if value.is_none()
                    && let Some(kind) = disposition.rejected_topology()
                {
                    return Err(IrGenerationError::Admission(format!(
                        "checked IR binding `{}` uses an unsupported {} for an uninitialized binding",
                        name,
                        kind.topology()
                    )));
                }
                if let Some(value) = value {
                    let static_string = if type_annotation.is_none()
                        || disposition.supported_contract() == Some(BindingContractKind::String)
                    {
                        Self::static_string_value(value, bindings)
                    } else {
                        None
                    };
                    let ty = if inside_impl || inside_generic_impl {
                        Self::validate_expression(
                            value,
                            bindings,
                            program,
                            if byte_input_result_initializer {
                                ExpressionUse::ByteInputResultBinding
                            } else {
                                ExpressionUse::Binding
                            },
                            inside_impl,
                            !inside_impl,
                        )?
                    } else if let Some(contract) = type_annotation
                        .as_ref()
                        .and_then(|annotation| program.structs.resolve_copy_annotation(annotation))
                        .filter(|contract| {
                            matches!(contract.ty, Ty::Array(_, 0))
                                && matches!(value, Expression::ArrayLiteral(elements) if elements.is_empty())
                        })
                    {
                        contract.ty
                    } else {
                        Self::validate_expression(
                            value,
                            bindings,
                            program,
                            if byte_input_result_initializer {
                                ExpressionUse::ByteInputResultBinding
                            } else {
                                ExpressionUse::Binding
                            },
                            inside_impl,
                            !inside_impl,
                        )?
                    };
                    if matches!(ty, Ty::Void) {
                        return Err(IrGenerationError::Admission(
                            "Void expressions cannot be stored in a binding".to_string(),
                        ));
                    }
                    if disposition.defers_to_tuple_contract() {
                        match validate_tuple_binding(
                            type_annotation.as_ref(),
                            &ty,
                            *mutable,
                            &program.structs,
                        ) {
                            Ok(()) => {}
                            Err(TupleBindingValidationError::Explicit(message)) => {
                                return Err(IrGenerationError::Admission(message));
                            }
                            Err(TupleBindingValidationError::PreserveInitializedDirectAnnotationRejection) => {
                                return Err(IrGenerationError::Admission(format!(
                                    "checked IR binding `{name}` uses an unsupported tuple type annotation for an initialized binding"
                                )));
                            }
                        }
                    }
                    if !is_top_level
                        && !inside_impl
                        && !inside_generic_impl
                        && type_annotation.as_ref().is_some_and(|annotation| {
                            !is_legacy_numeric_array_annotation(annotation)
                                && matches!(annotation, Type::Array(_, _))
                        })
                    {
                        program
                            .structs
                            .validate_copy_array_binding(type_annotation.as_ref(), &ty)
                            .map_err(|error| IrGenerationError::Admission(error.diagnostic()))?;
                    }
                    program
                        .structs
                        .validate_direct_binding_initializer(value, &ty)
                        .map_err(|error| IrGenerationError::Admission(error.diagnostic()))?;
                    if let Ty::Struct(struct_name) = &ty {
                        program
                            .structs
                            .validate_binding_annotation(struct_name, type_annotation.as_ref())
                            .map_err(|error| IrGenerationError::Admission(error.diagnostic()))?;
                    }
                    if let Ty::Enum(enum_name) = &ty {
                        program
                            .enums
                            .validate_binding(enum_name, type_annotation.as_ref())
                            .map_err(|error| IrGenerationError::Admission(error.diagnostic()))?;
                    }
                    let reference_annotation = if matches!(ty, Ty::Reference(_, _)) {
                        type_annotation.as_ref().map_or(
                            LocalReferenceDisposition::Preserved,
                            |annotation| {
                                classify_local_reference_annotation_with_enums(
                                    annotation,
                                    true,
                                    &program.structs,
                                    &program.enums,
                                )
                            },
                        )
                    } else {
                        LocalReferenceDisposition::Preserved
                    };
                    if let Some(kind) = disposition.rejected_topology()
                        && !matches!(
                            &reference_annotation,
                            LocalReferenceDisposition::Supported(_)
                        )
                    {
                        return Err(IrGenerationError::Admission(format!(
                            "checked IR binding `{}` uses an unsupported {} for an initialized binding",
                            name,
                            kind.topology()
                        )));
                    }
                    if let LocalReferenceDisposition::ExplicitlyRejected(message) =
                        reference_annotation
                    {
                        return Err(IrGenerationError::Admission(message));
                    }
                    if !inside_generic_impl && let Some(contract) = disposition.supported_contract()
                    {
                        let expected = contract.ty();
                        if ty != expected {
                            return Err(IrGenerationError::Admission(format!(
                                "checked IR binding `{}` type annotation mismatch: expected {}, actual {}",
                                name, expected, ty
                            )));
                        }
                    }
                    if let LocalReferenceDisposition::Supported(contract) = reference_annotation {
                        let expected = contract.reference_type();
                        if ty != expected {
                            return Err(IrGenerationError::Admission(format!(
                                "checked IR binding `{}` type annotation mismatch: expected {}, actual {}",
                                name, expected, ty
                            )));
                        }
                    }
                    classify_mutable_reference_binding(value, &ty)
                        .map_err(IrGenerationError::Admission)?;
                    if *mutable && matches!(ty, Ty::String) {
                        return Err(IrGenerationError::Admission(
                            "mutable string bindings are not admitted; checked strings are immutable compile-time aliases"
                                .to_string(),
                        ));
                    }
                    if let Expression::Borrow { expr, mutable } = value
                        && let Expression::Identifier(source) = expr.as_ref()
                    {
                        let source = bindings
                            .get_mut(source)
                            .expect("shared reference classifier resolved source binding");
                        source.ownership = if *mutable {
                            OwnershipState::MutablyBorrowed
                        } else {
                            match source.ownership.clone() {
                                OwnershipState::Owned => OwnershipState::ImmutablyBorrowed(1),
                                OwnershipState::ImmutablyBorrowed(count) => {
                                    OwnershipState::ImmutablyBorrowed(count + 1)
                                }
                                OwnershipState::MaybeMoved => {
                                    unreachable!(
                                        "shared reference classifier rejected maybe-moved source"
                                    )
                                }
                                _ => unreachable!(
                                    "shared reference classifier rejected conflicting borrow"
                                ),
                            }
                        };
                    }
                    Self::apply_enum_expression_ownership(value, bindings, program, inside_loop)?;
                    if let Expression::Identifier(source) = value
                        && matches!(
                            bindings.get(source).map(|binding| &binding.ty),
                            Some(Ty::Enum(_))
                        )
                    {
                        bindings
                            .get_mut(source)
                            .expect("checked enum binding move source remains in scope")
                            .ownership = OwnershipState::Moved;
                    }
                    bindings.insert(
                        name.clone(),
                        AdmissionBinding {
                            mutable: *mutable,
                            initialized: true,
                            ownership: OwnershipState::Owned,
                            callable: matches!(ty, Ty::Fn(_)),
                            static_string,
                            ty,
                        },
                    );
                }
            }
            Statement::Assignment { target, value } => {
                if let Expression::Identifier(name) = target
                    && bindings
                        .get(name)
                        .is_some_and(|binding| binding.ty == Ty::ByteBuffer)
                {
                    return Err(IrGenerationError::Admission(format!(
                        "ByteBuffer owner `{name}` may only be initialized or moved by direct binding"
                    )));
                }
                let mut array_selector_types = Vec::new();
                if let Some(selectors) = projected_copydata_assignment_array_selectors(target)
                    .map_err(IrGenerationError::Admission)?
                {
                    for selector in selectors {
                        array_selector_types.push(Self::validate_expression(
                            selector,
                            bindings,
                            program,
                            ExpressionUse::Value,
                            inside_impl,
                            !inside_impl,
                        )?);
                        Self::apply_enum_expression_ownership(
                            selector,
                            bindings,
                            program,
                            inside_loop,
                        )?;
                    }
                }
                let rhs = Self::validate_expression(
                    value,
                    bindings,
                    program,
                    ExpressionUse::Value,
                    inside_impl,
                    !inside_impl,
                )?;
                Self::apply_enum_expression_ownership(value, bindings, program, inside_loop)?;
                let inside_admitted_function = bindings.contains_key(STRUCT_ADMISSION_BINDING)
                    && !inside_impl
                    && !inside_generic_impl;
                let mutable_reference_facts = if let Expression::Deref(reference) = target
                    && let Expression::Identifier(name) = reference.as_ref()
                {
                    bindings
                        .get(name)
                        .map(|binding| MutableReferenceAssignmentFacts {
                            ty: binding.ty.clone(),
                            initialized: binding.initialized,
                            local: inside_admitted_function,
                            ownership: binding.ownership.clone(),
                        })
                } else {
                    None
                };
                match classify_mutable_reference_assignment_with_enums(
                    target,
                    mutable_reference_facts.as_ref(),
                    &rhs,
                    inside_admitted_function,
                    &program.structs,
                    &program.enums,
                ) {
                    MutableReferenceAssignmentDisposition::Supported(_) => return Ok(()),
                    MutableReferenceAssignmentDisposition::ExplicitlyRejected(message) => {
                        return Err(IrGenerationError::Admission(message));
                    }
                    MutableReferenceAssignmentDisposition::Preserved => {}
                }
                match classify_projected_copydata_assignment(
                    target,
                    &rhs,
                    &array_selector_types,
                    inside_admitted_function,
                    &program.structs,
                    |name| {
                        bindings
                            .get(name)
                            .map(|binding| OwnedPlaceAssignmentTargetFacts {
                                ty: binding.ty.clone(),
                                mutable: binding.mutable,
                                initialized: binding.initialized,
                                local: inside_admitted_function,
                                ownership: binding.ownership.clone(),
                            })
                    },
                ) {
                    ProjectedCopyDataAssignmentDisposition::Supported(_) => return Ok(()),
                    ProjectedCopyDataAssignmentDisposition::ExplicitlyRejected(message) => {
                        return Err(IrGenerationError::Admission(message));
                    }
                    ProjectedCopyDataAssignmentDisposition::PreserveExistingBehavior => {}
                }
                let facts = if let Expression::Identifier(name) = target {
                    bindings
                        .get(name)
                        .map(|binding| OwnedPlaceAssignmentTargetFacts {
                            ty: binding.ty.clone(),
                            mutable: binding.mutable,
                            initialized: binding.initialized,
                            local: inside_admitted_function,
                            ownership: binding.ownership.clone(),
                        })
                } else {
                    None
                };
                match classify_owned_place_assignment(
                    Some(target),
                    facts.as_ref(),
                    Some(value),
                    &rhs,
                    inside_admitted_function,
                    inside_loop,
                    &program.structs,
                    &program.enums,
                ) {
                    OwnedPlaceAssignmentDisposition::Supported(contract) => {
                        if let Some(source) = contract.moved_source {
                            bindings
                                .get_mut(&source)
                                .expect("shared owned-place classifier resolved source binding")
                                .ownership = OwnershipState::Moved;
                        }
                        bindings
                            .get_mut(&contract.name)
                            .expect("shared owned-place classifier resolved target binding")
                            .ownership = contract.transition.resulting_ownership();
                    }
                    OwnedPlaceAssignmentDisposition::ExplicitlyRejected(message) => {
                        return Err(IrGenerationError::Admission(message));
                    }
                    OwnedPlaceAssignmentDisposition::PreserveExistingBehavior => {
                        unreachable!("explicit assignment must receive a classifier disposition")
                    }
                }
            }
            Statement::Return(expression) => {
                if let Some(expression) = expression {
                    Self::validate_expression(
                        expression,
                        bindings,
                        program,
                        ExpressionUse::Value,
                        inside_impl,
                        !inside_impl,
                    )?;
                    Self::apply_enum_expression_ownership(
                        expression,
                        bindings,
                        program,
                        inside_loop,
                    )?;
                }
            }
            Statement::Expression(expression) => {
                Self::validate_expression(
                    expression,
                    bindings,
                    program,
                    ExpressionUse::Discarded,
                    inside_impl,
                    !inside_impl,
                )?;
                Self::apply_enum_expression_ownership(expression, bindings, program, inside_loop)?;
            }
            Statement::Block(block) => {
                let mut nested = bindings.clone();
                Self::validate_block(
                    block,
                    &mut nested,
                    program,
                    inside_loop,
                    inside_impl,
                    inside_generic_impl,
                    false,
                    loop_controls,
                )?;
            }
            Statement::Loop { body } => {
                let initial_header = bindings.clone();
                let mut header = initial_header.clone();
                let mut converged = false;
                let mut final_exit = None;
                for _ in 0..LOOP_OWNERSHIP_FIXED_POINT_LIMIT {
                    let mut nested = header.clone();
                    loop_controls.push(AdmissionLoopControl::default());
                    let body_result = Self::validate_block(
                        body,
                        &mut nested,
                        program,
                        true,
                        inside_impl,
                        inside_generic_impl,
                        false,
                        loop_controls,
                    );
                    let control = loop_controls
                        .pop()
                        .expect("loop admission control frame was pushed");
                    body_result?;
                    let mut edges = Vec::new();
                    if block_reaches_merge(body, true) {
                        edges.push((LoopOwnershipEdgeKind::Fallthrough, nested));
                    }
                    edges.extend(
                        control
                            .continues
                            .into_iter()
                            .map(|state| (LoopOwnershipEdgeKind::Continue, state)),
                    );
                    edges.extend(
                        control
                            .breaks
                            .into_iter()
                            .map(|state| (LoopOwnershipEdgeKind::Break, state)),
                    );
                    let (next_header, exit) = Self::summarize_loop_admission(
                        &initial_header,
                        LoopOwnershipKind::Loop,
                        &edges,
                    )?;
                    if Self::admission_ownership_matches(&header, &next_header) {
                        converged = true;
                        final_exit = exit;
                        break;
                    }
                    header = next_header;
                }
                if !converged {
                    return Err(IrGenerationError::Admission(
                        "direct enum loop ownership did not converge within the finite fixed-point bound"
                            .to_string(),
                    ));
                }
                *bindings = final_exit.unwrap_or(initial_header);
            }
            Statement::Function {
                name,
                parameters,
                return_type,
                type_params,
                body,
                ..
            } => {
                let reference_contract = match classify_reference_function_with_enums(
                    name,
                    parameters,
                    return_type.as_ref(),
                    type_params,
                    &program.structs,
                    &program.enums,
                ) {
                    ReferenceFunctionDisposition::Supported(contract) => Some(contract),
                    ReferenceFunctionDisposition::ExplicitlyRejected(diagnostic) => {
                        return Err(IrGenerationError::Admission(diagnostic));
                    }
                    ReferenceFunctionDisposition::Preserved => None,
                };
                if reference_contract.is_none() && !type_params.is_empty() {
                    return Err(IrGenerationError::Admission(
                        "generic function IR is not admitted in CORE-010".to_string(),
                    ));
                }
                let mut function_bindings = HashMap::new();
                if is_top_level && !inside_impl {
                    function_bindings.insert(
                        STRUCT_ADMISSION_BINDING.to_string(),
                        AdmissionBinding {
                            ty: Ty::Void,
                            mutable: false,
                            initialized: true,
                            ownership: OwnershipState::Owned,
                            callable: false,
                            static_string: None,
                        },
                    );
                }
                let enum_contract = if reference_contract.is_none() {
                    program
                        .enums
                        .resolve_function_contract(
                            name,
                            parameters,
                            return_type.as_ref(),
                            type_params,
                            |annotation| {
                                program
                                    .structs
                                    .resolve_copy_annotation(annotation)
                                    .map(|contract| (contract.ty, contract.logical_type))
                            },
                        )
                        .map_err(|error| IrGenerationError::Admission(error.diagnostic()))?
                } else {
                    None
                };
                let copy_contract = if reference_contract.is_none() && enum_contract.is_none() {
                    match program.structs.resolve_copy_function_contract(
                        name,
                        parameters,
                        return_type.as_ref(),
                        type_params,
                    ) {
                        Ok(contract) => contract,
                        Err(StructContractError::PreserveExistingBehavior) => None,
                        Err(error) => {
                            return Err(IrGenerationError::Admission(error.diagnostic()));
                        }
                    }
                } else {
                    None
                };
                for (index, parameter) in parameters.iter().enumerate() {
                    let parameter_ty = if let Some(contract) = &reference_contract {
                        contract.parameters[index].1.ty.clone()
                    } else if let Some(contract) = &enum_contract {
                        contract.parameters[index].1.ty.clone()
                    } else {
                        copy_contract.as_ref().map_or_else(
                            || Self::admission_type(&parameter.param_type),
                            |contract| contract.parameters[index].1.ty.clone(),
                        )
                    };
                    if reference_contract.is_none()
                        && enum_contract.is_none()
                        && copy_contract.is_none()
                        && !matches!(parameter_ty, Ty::Int | Ty::Float | Ty::Bool | Ty::Char)
                    {
                        return Err(IrGenerationError::Admission(format!(
                            "function parameter `{}` is not an admitted scalar type",
                            parameter.name
                        )));
                    }
                    function_bindings.insert(
                        parameter.name.clone(),
                        AdmissionBinding {
                            ty: parameter_ty,
                            mutable: false,
                            initialized: true,
                            ownership: OwnershipState::Owned,
                            callable: false,
                            static_string: None,
                        },
                    );
                }
                if reference_contract.is_none()
                    && enum_contract.is_none()
                    && copy_contract.is_none()
                    && return_type.as_ref().is_some_and(|return_type| {
                        !matches!(
                            Self::admission_type(return_type),
                            Ty::Int | Ty::Float | Ty::Bool | Ty::Char
                        )
                    })
                {
                    return Err(IrGenerationError::Admission(
                        "function return type is not an admitted scalar or Void type".to_string(),
                    ));
                }
                Self::validate_block(
                    body,
                    &mut function_bindings,
                    program,
                    false,
                    inside_impl,
                    inside_generic_impl,
                    program.byte_buffer_source_enabled
                        && is_top_level
                        && !inside_impl
                        && !inside_generic_impl
                        && type_params.is_empty(),
                    loop_controls,
                )?;
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                Self::validate_expression(
                    condition,
                    bindings,
                    program,
                    ExpressionUse::Value,
                    inside_impl,
                    !inside_impl,
                )?;
                Self::apply_enum_expression_ownership(condition, bindings, program, inside_loop)?;
                let entry_bindings = bindings.clone();
                let mut then_bindings = entry_bindings.clone();
                Self::validate_block(
                    then_block,
                    &mut then_bindings,
                    program,
                    inside_loop,
                    inside_impl,
                    inside_generic_impl,
                    false,
                    loop_controls,
                )?;
                let else_bindings = if let Some(else_statement) = else_block {
                    let mut else_bindings = entry_bindings.clone();
                    Self::validate_statement(
                        else_statement,
                        &mut else_bindings,
                        program,
                        inside_loop,
                        inside_impl,
                        inside_generic_impl,
                        false,
                        false,
                        loop_controls,
                    )?;
                    else_bindings
                } else {
                    entry_bindings.clone()
                };
                Self::apply_conditional_admission_join(
                    bindings,
                    &entry_bindings,
                    &then_bindings,
                    block_reaches_merge(then_block, inside_loop),
                    &else_bindings,
                    else_block
                        .as_deref()
                        .is_none_or(|statement| statement_reaches_merge(statement, inside_loop)),
                    inside_loop,
                )?;
            }
            Statement::While { condition, body } => {
                let initial_header = bindings.clone();
                let mut header = initial_header.clone();
                let mut converged = false;
                let mut final_exit = None;
                for _ in 0..LOOP_OWNERSHIP_FIXED_POINT_LIMIT {
                    let mut condition_bindings = header.clone();
                    Self::validate_expression(
                        condition,
                        &mut condition_bindings,
                        program,
                        ExpressionUse::Value,
                        inside_impl,
                        !inside_impl,
                    )?;
                    Self::apply_enum_expression_ownership(
                        condition,
                        &mut condition_bindings,
                        program,
                        true,
                    )?;
                    let mut nested = condition_bindings.clone();
                    loop_controls.push(AdmissionLoopControl::default());
                    let body_result = Self::validate_block(
                        body,
                        &mut nested,
                        program,
                        true,
                        inside_impl,
                        inside_generic_impl,
                        false,
                        loop_controls,
                    );
                    let control = loop_controls
                        .pop()
                        .expect("while admission control frame was pushed");
                    body_result?;
                    let mut edges = vec![(LoopOwnershipEdgeKind::Condition, condition_bindings)];
                    if block_reaches_merge(body, true) {
                        edges.push((LoopOwnershipEdgeKind::Fallthrough, nested));
                    }
                    edges.extend(
                        control
                            .continues
                            .into_iter()
                            .map(|state| (LoopOwnershipEdgeKind::Continue, state)),
                    );
                    edges.extend(
                        control
                            .breaks
                            .into_iter()
                            .map(|state| (LoopOwnershipEdgeKind::Break, state)),
                    );
                    let (next_header, exit) = Self::summarize_loop_admission(
                        &initial_header,
                        LoopOwnershipKind::While,
                        &edges,
                    )?;
                    if Self::admission_ownership_matches(&header, &next_header) {
                        converged = true;
                        final_exit = exit;
                        break;
                    }
                    header = next_header;
                }
                if !converged {
                    return Err(IrGenerationError::Admission(
                        "direct enum while-loop ownership did not converge within the finite fixed-point bound"
                            .to_string(),
                    ));
                }
                *bindings = final_exit.unwrap_or(initial_header);
            }
            Statement::For {
                variable,
                iterable,
                body,
            } => {
                let iterable_ty = Self::validate_expression(
                    iterable,
                    bindings,
                    program,
                    ExpressionUse::Value,
                    inside_impl,
                    !inside_impl,
                )?;
                Self::apply_enum_expression_ownership(iterable, bindings, program, true)?;
                let initial_header = bindings.clone();
                let element_ty = match iterable_ty {
                    Ty::Array(element, _) | Ty::Vec(element) => *element,
                    _ => {
                        return Err(IrGenerationError::Admission(
                            "for-loop iteration requires an admitted array/Vec".to_string(),
                        ));
                    }
                };
                let mut header = initial_header.clone();
                let mut converged = false;
                let mut final_exit = None;
                for _ in 0..LOOP_OWNERSHIP_FIXED_POINT_LIMIT {
                    let mut nested = header.clone();
                    let shadowed_loop_binding = nested.insert(
                        variable.clone(),
                        AdmissionBinding {
                            ty: element_ty.clone(),
                            mutable: false,
                            initialized: true,
                            ownership: OwnershipState::Owned,
                            callable: false,
                            static_string: None,
                        },
                    );
                    loop_controls.push(AdmissionLoopControl::default());
                    let body_result = Self::validate_block(
                        body,
                        &mut nested,
                        program,
                        true,
                        inside_impl,
                        inside_generic_impl,
                        false,
                        loop_controls,
                    );
                    let control = loop_controls
                        .pop()
                        .expect("for admission control frame was pushed");
                    body_result?;
                    let restore_loop_binding = |state: &mut HashMap<String, AdmissionBinding>| {
                        if let Some(shadowed) = &shadowed_loop_binding {
                            state.insert(variable.clone(), shadowed.clone());
                        } else {
                            state.remove(variable);
                        }
                    };
                    restore_loop_binding(&mut nested);
                    let mut edges = vec![(LoopOwnershipEdgeKind::Iterable, initial_header.clone())];
                    if block_reaches_merge(body, true) {
                        edges.push((LoopOwnershipEdgeKind::Fallthrough, nested));
                    }
                    edges.extend(control.continues.into_iter().map(|mut state| {
                        restore_loop_binding(&mut state);
                        (LoopOwnershipEdgeKind::Continue, state)
                    }));
                    edges.extend(control.breaks.into_iter().map(|mut state| {
                        restore_loop_binding(&mut state);
                        (LoopOwnershipEdgeKind::Break, state)
                    }));
                    let (next_header, exit) = Self::summarize_loop_admission(
                        &initial_header,
                        LoopOwnershipKind::For,
                        &edges,
                    )?;
                    if Self::admission_ownership_matches(&header, &next_header) {
                        converged = true;
                        final_exit = exit;
                        break;
                    }
                    header = next_header;
                }
                if !converged {
                    return Err(IrGenerationError::Admission(
                        "direct enum for-loop ownership did not converge within the finite fixed-point bound"
                            .to_string(),
                    ));
                }
                *bindings = final_exit.unwrap_or(initial_header);
            }
            Statement::ImplBlock {
                methods,
                type_params,
                ..
            } => {
                let methods_are_generic = inside_generic_impl || !type_params.is_empty();
                for method in methods {
                    Self::validate_statement(
                        method,
                        bindings,
                        program,
                        false,
                        true,
                        methods_are_generic,
                        false,
                        false,
                        loop_controls,
                    )?;
                }
            }
            // Trait declarations remain syntax-only in this prototype. The semantic
            // pass recursively diagnoses unsupported expressions in default bodies;
            // checked lowering must not activate runtime name binding for them.
            Statement::TraitDef { .. } => {}
            Statement::Break => {
                if !inside_loop {
                    return Err(IrGenerationError::Admission(
                        "break and continue are only admitted inside loops".to_string(),
                    ));
                }
                for (name, binding) in bindings.iter() {
                    if let Some(message) = live_mutable_owner_immutable_enum_loan_edge_diagnostic(
                        name,
                        &binding.ty,
                        binding.mutable,
                        &binding.ownership,
                        LoopOwnershipEdgeKind::Break,
                    ) {
                        return Err(IrGenerationError::Admission(message));
                    }
                }
                loop_controls
                    .last_mut()
                    .expect("inside_loop requires an admission control frame")
                    .breaks
                    .push(bindings.clone());
            }
            Statement::Continue => {
                if !inside_loop {
                    return Err(IrGenerationError::Admission(
                        "break and continue are only admitted inside loops".to_string(),
                    ));
                }
                for (name, binding) in bindings.iter() {
                    if let Some(message) = live_mutable_owner_immutable_enum_loan_edge_diagnostic(
                        name,
                        &binding.ty,
                        binding.mutable,
                        &binding.ownership,
                        LoopOwnershipEdgeKind::Continue,
                    ) {
                        return Err(IrGenerationError::Admission(message));
                    }
                }
                loop_controls
                    .last_mut()
                    .expect("inside_loop requires an admission control frame")
                    .continues
                    .push(bindings.clone());
            }
            Statement::StructDef { .. } | Statement::EnumDef { .. } | Statement::ModDecl { .. } => {
            }
            Statement::UseImport {
                syntax, location, ..
            } => {
                return Err(IrGenerationError::Admission(
                    unsupported_name_import_diagnostic(*syntax, location),
                ));
            }
        }
        Ok(())
    }

    #[inline(never)]
    fn validate_function_call_expression(
        name: &str,
        arguments: &[Expression],
        bindings: &HashMap<String, AdmissionBinding>,
        program: &AdmissionProgram,
        expression_use: ExpressionUse,
        inside_impl: bool,
        admit_static_string_equality: bool,
    ) -> Result<Ty, IrGenerationError> {
        let admission_error = |message: &str| IrGenerationError::Admission(message.to_string());
        let inside_admitted_function =
            bindings.contains_key(STRUCT_ADMISSION_BINDING) && !inside_impl;
        let reference_call = if let Some(contract) = program.reference_functions.get(name) {
            classify_reference_call_with_enums(
                contract,
                arguments,
                |subject| {
                    Self::admission_local_reference_source_facts(
                        subject,
                        bindings,
                        inside_admitted_function,
                    )
                },
                |selector| {
                    Self::validate_expression(
                        selector,
                        bindings,
                        program,
                        ExpressionUse::Value,
                        inside_impl,
                        admit_static_string_equality,
                    )
                    .map_err(|error| error.to_string())
                },
                &program.structs,
                &program.enums,
            )
        } else {
            ReferenceCallDisposition::Preserved
        };
        let mut argument_types = Vec::with_capacity(arguments.len());
        match reference_call {
            ReferenceCallDisposition::Supported(contract) => {
                for (index, argument) in arguments.iter().enumerate() {
                    if let Some(reference_type) = contract.reference_type(index) {
                        argument_types.push(reference_type);
                    } else {
                        argument_types.push(Self::validate_expression(
                            argument,
                            bindings,
                            program,
                            ExpressionUse::Value,
                            inside_impl,
                            admit_static_string_equality,
                        )?);
                    }
                }
            }
            ReferenceCallDisposition::ExplicitlyRejected(message) => {
                return Err(admission_error(&message));
            }
            ReferenceCallDisposition::Preserved => {
                for argument in arguments {
                    argument_types.push(Self::validate_expression(
                        argument,
                        bindings,
                        program,
                        ExpressionUse::Value,
                        inside_impl,
                        admit_static_string_equality,
                    )?);
                }
            }
        }
        if let Some(binding) = bindings.get(name)
            && binding.callable
        {
            let result = match &binding.ty {
                Ty::Fn(signature) => Self::callable_result_type(signature)?,
                _ => return Err(admission_error("callable binding lost its signature")),
            };
            return Self::classified_call_result(classify_function_call(FunctionCallFacts {
                name: name.to_string(),
                target: FunctionCallTarget::Callable { result },
                arguments: argument_types,
                use_context: if matches!(
                    expression_use,
                    ExpressionUse::Discarded | ExpressionUse::MatchArm
                ) {
                    FunctionCallUse::Discarded
                } else {
                    FunctionCallUse::Value
                },
            }));
        }
        let target = if let Some(function) = program.functions.get(name) {
            match (&function.parameter_types, function.arity) {
                (Some(parameter_types), Some(_)) => FunctionCallTarget::Admitted {
                    parameters: Some(
                        parameter_types
                            .iter()
                            .cloned()
                            .map(|ty| FunctionCallParameter { name: None, ty })
                            .collect(),
                    ),
                    result: function.result.clone(),
                },
                _ => FunctionCallTarget::DeclaredUnadmitted,
            }
        } else if matches!(name, "Some" | "Ok") {
            FunctionCallTarget::PreservedContext {
                diagnostic: unsupported_function_call_diagnostic(
                    name,
                    "enum and Option/Result construction is not admitted in checked IR",
                ),
            }
        } else {
            FunctionCallTarget::Missing
        };
        Self::classified_call_result(classify_function_call(FunctionCallFacts {
            name: name.to_string(),
            target,
            arguments: argument_types,
            use_context: if matches!(
                expression_use,
                ExpressionUse::Discarded | ExpressionUse::MatchArm
            ) {
                FunctionCallUse::Discarded
            } else {
                FunctionCallUse::Value
            },
        }))
    }

    fn validate_logical_expression_iterative(
        left: &Expression,
        right: &Expression,
        bindings: &HashMap<String, AdmissionBinding>,
        program: &AdmissionProgram,
        inside_impl: bool,
        admit_static_string_equality: bool,
    ) -> Result<Ty, IrGenerationError> {
        let mut pending = vec![right, left];
        while let Some(operand) = pending.pop() {
            match operand {
                Expression::Logical { left, right, .. } => {
                    pending.push(right);
                    pending.push(left);
                }
                operand => {
                    Self::validate_expression(
                        operand,
                        bindings,
                        program,
                        ExpressionUse::Value,
                        inside_impl,
                        admit_static_string_equality,
                    )?;
                }
            }
        }
        Ok(Ty::Bool)
    }

    fn validate_expression(
        expression: &Expression,
        bindings: &HashMap<String, AdmissionBinding>,
        program: &AdmissionProgram,
        expression_use: ExpressionUse,
        inside_impl: bool,
        admit_static_string_equality: bool,
    ) -> Result<Ty, IrGenerationError> {
        let admission_error = |message: &str| IrGenerationError::Admission(message.to_string());
        match expression {
            Expression::IntegerLiteral(value) => {
                i32::try_from(*value).map_err(|_| {
                    admission_error("integer literal is outside the admitted i32 range")
                })?;
                Ok(Ty::Int)
            }
            Expression::FloatLiteral(_) => Ok(Ty::Float),
            Expression::CharacterLiteral(_) => Ok(Ty::Char),
            Expression::StringLiteral(_) => Ok(Ty::String),
            Expression::Identifier(name) => {
                let binding = bindings.get(name).ok_or_else(|| {
                    admission_error(&format!("checked IR has no binding for `{name}`"))
                })?;
                if binding.callable {
                    return Err(admission_error(
                        "closure aliases may only be used as direct callees",
                    ));
                }
                if binding.ownership == OwnershipState::MutablyBorrowed {
                    return Err(admission_error(&format!(
                        "cannot read `{name}` while it is mutably borrowed"
                    )));
                }
                if binding.ownership == OwnershipState::Moved {
                    return Err(admission_error(&format!(
                        "use of moved value `{name}` in checked IR"
                    )));
                }
                if binding.ownership == OwnershipState::MaybeMoved {
                    return Err(IrGenerationError::Admission(maybe_moved_diagnostic(name)));
                }
                if binding.ty == Ty::ByteBuffer {
                    return Err(admission_error(
                        "ByteBuffer owners may only be moved by direct binding or used by byte-buffer intrinsics",
                    ));
                }
                Ok(binding.ty.clone())
            }
            Expression::Binary {
                op,
                left,
                right,
                ty,
            } => {
                if matches!(op, crate::ast::BinaryOp::Modulo) {
                    return Err(admission_error("modulo is not admitted in checked IR"));
                }
                let left_ty = Self::validate_expression(
                    left,
                    bindings,
                    program,
                    ExpressionUse::Value,
                    inside_impl,
                    admit_static_string_equality,
                )?;
                let right_ty = Self::validate_expression(
                    right,
                    bindings,
                    program,
                    ExpressionUse::Value,
                    inside_impl,
                    admit_static_string_equality,
                )?;
                let derived_ty = if matches!(left_ty, Ty::Float)
                    && matches!(right_ty, Ty::Int | Ty::Float)
                    || matches!(right_ty, Ty::Float) && matches!(left_ty, Ty::Int | Ty::Float)
                {
                    Ty::Float
                } else if matches!(left_ty, Ty::Int) && matches!(right_ty, Ty::Int) {
                    Ty::Int
                } else {
                    return Err(admission_error(
                        "binary expression is not an admitted scalar",
                    ));
                };
                if let Some(actual_ty) = ty
                    && actual_ty != &derived_ty
                {
                    return Err(admission_error(&format!(
                        "binary result metadata mismatch: expected {}, actual {}",
                        derived_ty, actual_ty
                    )));
                }
                if matches!(op, crate::ast::BinaryOp::Divide)
                    && matches!(derived_ty, Ty::Int)
                    && Self::constant_integer_value(right) == Some(0)
                {
                    return Err(admission_error("constant integer division by zero"));
                }
                Ok(derived_ty)
            }
            Expression::FunctionCall { name, arguments } => {
                if let Some(result) = Self::validate_byte_input_intrinsic(
                    name,
                    arguments,
                    program,
                    expression_use,
                    inside_impl,
                )? {
                    return Ok(result);
                }
                if let Some(result) = Self::validate_byte_buffer_intrinsic(
                    name,
                    arguments,
                    bindings,
                    program,
                    inside_impl,
                    admit_static_string_equality,
                )? {
                    return Ok(result);
                }
                Self::validate_function_call_expression(
                    name,
                    arguments,
                    bindings,
                    program,
                    expression_use,
                    inside_impl,
                    admit_static_string_equality,
                )
            }
            Expression::MethodCall {
                object,
                method,
                arguments,
            } => {
                let object_ty = Self::validate_expression(
                    object,
                    bindings,
                    program,
                    ExpressionUse::Value,
                    inside_impl,
                    admit_static_string_equality,
                )?;
                for argument in arguments {
                    Self::validate_expression(
                        argument,
                        bindings,
                        program,
                        ExpressionUse::Value,
                        inside_impl,
                        admit_static_string_equality,
                    )?;
                }
                let static_string = Self::static_string_value(object, bindings);
                let static_arguments = arguments
                    .iter()
                    .map(|argument| Self::static_string_value(argument, bindings))
                    .collect::<Vec<_>>();
                let static_argument_refs = static_arguments
                    .iter()
                    .map(|argument| argument.as_deref())
                    .collect::<Vec<_>>();
                match classify_intrinsic_method(
                    &object_ty,
                    method,
                    arguments.len(),
                    static_string.as_deref(),
                    &static_argument_refs,
                    &program.structs,
                    IntrinsicMethodPhase::Checked,
                    inside_impl,
                ) {
                    IntrinsicMethodDisposition::Supported { result, .. } => Ok(result),
                    IntrinsicMethodDisposition::ExplicitlyRejected(diagnostic)
                    | IntrinsicMethodDisposition::PreservedContext(diagnostic) => {
                        Err(admission_error(&diagnostic))
                    }
                }
            }
            Expression::Print { arguments, .. } | Expression::Println { arguments, .. } => {
                for argument in arguments {
                    let argument_ty = Self::validate_expression(
                        argument,
                        bindings,
                        program,
                        ExpressionUse::PrintArgument,
                        inside_impl,
                        admit_static_string_equality,
                    )?;
                    if !matches!(argument_ty, Ty::Int | Ty::Float | Ty::Bool | Ty::String) {
                        return Err(admission_error("print argument type is not admitted"));
                    }
                }
                if !matches!(
                    expression_use,
                    ExpressionUse::Discarded | ExpressionUse::MatchArm
                ) {
                    return Err(admission_error("Void expressions cannot be used as values"));
                }
                Ok(Ty::Void)
            }
            Expression::Comparison { op, left, right } => {
                let left_ty = Self::validate_expression(
                    left,
                    bindings,
                    program,
                    ExpressionUse::Value,
                    inside_impl,
                    admit_static_string_equality,
                )?;
                let right_ty = Self::validate_expression(
                    right,
                    bindings,
                    program,
                    ExpressionUse::Value,
                    inside_impl,
                    admit_static_string_equality,
                )?;
                if admit_static_string_equality {
                    let left_static = Self::static_string_value(left, bindings);
                    let right_static = Self::static_string_value(right, bindings);
                    if let StaticStringEqualityDisposition::StaticBool(_) =
                        classify_static_string_equality(
                            left_static.as_deref(),
                            op,
                            right_static.as_deref(),
                        )
                    {
                        return Ok(Ty::Bool);
                    }
                }
                let char_comparison = matches!(
                    (
                        PrimitiveKind::from_ty(&left_ty),
                        PrimitiveKind::from_ty(&right_ty)
                    ),
                    (Some(PrimitiveKind::Char), Some(PrimitiveKind::Char))
                ) && matches!(
                    op,
                    crate::ast::ComparisonOp::Equal | crate::ast::ComparisonOp::NotEqual
                );
                if !char_comparison
                    && !matches!(
                        (&left_ty, &right_ty),
                        (&Ty::Int, &Ty::Int)
                            | (&Ty::Float, &Ty::Float)
                            | (&Ty::Int, &Ty::Float)
                            | (&Ty::Float, &Ty::Int)
                            | (&Ty::Bool, &Ty::Bool)
                    )
                {
                    return Err(admission_error(
                        "comparison operand types are not admitted; expected numeric operands or Bool with Bool",
                    ));
                }
                Ok(Ty::Bool)
            }
            Expression::Logical { left, right, .. } => Self::validate_logical_expression_iterative(
                left,
                right,
                bindings,
                program,
                inside_impl,
                admit_static_string_equality,
            ),
            Expression::Unary { op, operand } => {
                if matches!(op, crate::ast::UnaryOp::Negate)
                    && let Expression::IntegerLiteral(value) = operand.as_ref()
                {
                    let negated = value.checked_neg().ok_or_else(|| {
                        admission_error("integer literal is outside the admitted i32 range")
                    })?;
                    i32::try_from(negated).map_err(|_| {
                        admission_error("integer literal is outside the admitted i32 range")
                    })?;
                    return Ok(Ty::Int);
                }
                let operand_ty = Self::validate_expression(
                    operand,
                    bindings,
                    program,
                    ExpressionUse::Value,
                    inside_impl,
                    admit_static_string_equality,
                )?;
                match op {
                    crate::ast::UnaryOp::Not => Ok(Ty::Bool),
                    crate::ast::UnaryOp::Negate => {
                        if let Some(value) = Self::constant_integer_value(expression) {
                            i32::try_from(value).map_err(|_| {
                                admission_error("integer literal is outside the admitted i32 range")
                            })?;
                        }
                        Ok(operand_ty)
                    }
                }
            }
            Expression::ArrayLiteral(elements) => {
                if elements.is_empty() {
                    return Err(admission_error(
                        "empty array literals have no admitted logical element type",
                    ));
                }
                let mut element_ty = Ty::Int;
                let mut remaining_types = Vec::with_capacity(elements.len().saturating_sub(1));
                for (index, element) in elements.iter().enumerate() {
                    let current = Self::validate_expression(
                        element,
                        bindings,
                        program,
                        ExpressionUse::Value,
                        inside_impl,
                        admit_static_string_equality,
                    )?;
                    if index == 0 {
                        if program.structs.resolve_copy_type(&current).is_none() {
                            return Err(admission_error(
                                "fixed arrays require recursively admitted Copy-data elements",
                            ));
                        }
                        element_ty = current;
                    } else if !inside_impl {
                        remaining_types.push(current);
                    }
                }
                if !inside_impl {
                    program
                        .structs
                        .validate_copy_array_elements(&element_ty, remaining_types)
                        .map_err(|error| admission_error(&error.diagnostic()))?;
                }
                Ok(Ty::Array(Box::new(element_ty), elements.len()))
            }
            Expression::ArrayRepeat { value, count } => {
                let element_ty = Self::validate_expression(
                    value,
                    bindings,
                    program,
                    ExpressionUse::Value,
                    inside_impl,
                    admit_static_string_equality,
                )?;
                if program.structs.resolve_copy_type(&element_ty).is_none() {
                    return Err(admission_error(
                        "fixed arrays require recursively admitted Copy-data elements",
                    ));
                }
                Ok(Ty::Array(Box::new(element_ty), *count))
            }
            Expression::IndexAccess { object, index } => {
                let object_ty = Self::validate_expression(
                    object,
                    bindings,
                    program,
                    ExpressionUse::Value,
                    inside_impl,
                    admit_static_string_equality,
                )?;
                let index_ty = Self::validate_expression(
                    index,
                    bindings,
                    program,
                    ExpressionUse::Value,
                    inside_impl,
                    admit_static_string_equality,
                )?;
                if !matches!(index_ty, Ty::Int) {
                    return Err(admission_error("array index must be Int"));
                }
                if is_statically_empty_fixed_array(&object_ty) {
                    return Err(admission_error(
                        "zero-length fixed arrays cannot be indexed in checked IR",
                    ));
                }
                match program.structs.classify_copy_array_index(&object_ty, index) {
                    CopyArrayIndexDisposition::PreserveExistingBehavior
                    | CopyArrayIndexDisposition::Accepted { .. } => {}
                    CopyArrayIndexDisposition::OutOfBounds { index, count } => {
                        return Err(admission_error(&format!(
                            "fixed Copy-data array index {index} is outside 0..{count}"
                        )));
                    }
                }
                match object_ty {
                    Ty::Array(element, _) => Ok(*element),
                    _ => Err(admission_error("indexing requires an admitted fixed array")),
                }
            }
            Expression::Closure { location, .. } => {
                Err(admission_error(&unsupported_closure_diagnostic(location)))
            }
            Expression::EnumVariant {
                enum_name,
                variant,
                data,
            } => {
                let actual = if let Some(fields) = data.as_deref() {
                    let mut types = Vec::with_capacity(fields.len());
                    for field in fields {
                        types.push(Self::validate_expression(
                            field,
                            bindings,
                            program,
                            ExpressionUse::Value,
                            inside_impl,
                            admit_static_string_equality,
                        )?);
                    }
                    Some(types)
                } else {
                    None
                };
                let context = if bindings.contains_key(STRUCT_ADMISSION_BINDING) {
                    EnumExecutionContext::AdmittedFunction
                } else {
                    EnumExecutionContext::PreservedContext
                };
                let resolved = program
                    .enums
                    .resolve_constructor(enum_name, variant, data.as_ref().map(Vec::len), context)
                    .map_err(|error| match error {
                        EnumError::PreserveExistingBehavior => {
                            admission_error("enum construction is not admitted in checked IR")
                        }
                        _ => admission_error(&error.diagnostic()),
                    })?;
                program
                    .enums
                    .validate_constructor_payload(&resolved, actual.as_deref())
                    .map_err(|error| admission_error(&error.diagnostic()))?;
                Ok(resolved.contract.ty())
            }
            Expression::Borrow { expr, mutable } => {
                let inside_admitted_function =
                    bindings.contains_key(STRUCT_ADMISSION_BINDING) && !inside_impl;
                let facts = Self::admission_local_reference_source_facts(
                    expr,
                    bindings,
                    inside_admitted_function,
                );
                if !matches!(expr.as_ref(), Expression::Identifier(_)) {
                    Self::validate_expression(
                        expr,
                        bindings,
                        program,
                        ExpressionUse::Value,
                        inside_impl,
                        admit_static_string_equality,
                    )?;
                }
                match classify_local_borrow_with_enums(
                    expr,
                    *mutable,
                    facts.as_ref(),
                    &program.structs,
                    &program.enums,
                ) {
                    LocalReferenceDisposition::Supported(contract) => Ok(contract.reference_type()),
                    LocalReferenceDisposition::ExplicitlyRejected(message) => {
                        Err(admission_error(&message))
                    }
                    LocalReferenceDisposition::Preserved => unreachable!(
                        "borrow expressions are fully classified by the local reference contract"
                    ),
                }
            }
            Expression::Deref(expr) => {
                let reference = Self::validate_expression(
                    expr,
                    bindings,
                    program,
                    ExpressionUse::Value,
                    inside_impl,
                    admit_static_string_equality,
                )?;
                match classify_local_dereference(&reference, &program.structs) {
                    LocalReferenceDisposition::Supported(contract) => Ok(contract.pointee),
                    LocalReferenceDisposition::ExplicitlyRejected(message) => {
                        Err(admission_error(&message))
                    }
                    LocalReferenceDisposition::Preserved => unreachable!(
                        "dereference expressions are fully classified by the local reference contract"
                    ),
                }
            }
            Expression::FieldAccess { object, field } => {
                let context = if bindings.contains_key(STRUCT_ADMISSION_BINDING) {
                    StructExecutionContext::AdmittedFunction
                } else {
                    StructExecutionContext::PreservedContext
                };
                let receiver = Self::validate_expression(
                    object,
                    bindings,
                    program,
                    ExpressionUse::Value,
                    inside_impl,
                    admit_static_string_equality,
                )
                .map_err(|_| {
                    // Field access is one aggregate operation. Preserve its established
                    // fail-closed boundary without enumerating receiver expression shapes;
                    // exact admitted receiver types are decided below by StructRegistry.
                    admission_error("aggregate expression is not admitted in checked IR")
                })?;
                let (_, _, field) = program
                    .structs
                    .resolve_field(&receiver, field, context)
                    .map_err(|error| match error {
                        StructContractError::PreserveExistingBehavior => {
                            admission_error("aggregate expression is not admitted in checked IR")
                        }
                        _ => IrGenerationError::Admission(error.diagnostic()),
                    })?;
                Ok(field.ty())
            }
            Expression::StructLiteral { name, fields } => {
                let context = if bindings.contains_key(STRUCT_ADMISSION_BINDING) {
                    StructExecutionContext::AdmittedFunction
                } else {
                    StructExecutionContext::PreservedContext
                };
                let resolved = match program.structs.resolve_construction(name, fields, context) {
                    Ok(resolved) => resolved,
                    Err(error) => {
                        for (_, value) in fields {
                            let _ = Self::validate_expression(
                                value,
                                bindings,
                                program,
                                ExpressionUse::Value,
                                inside_impl,
                                admit_static_string_equality,
                            )?;
                        }
                        return Err(match error {
                            StructContractError::PreserveExistingBehavior => admission_error(
                                "aggregate expression is not admitted in checked IR",
                            ),
                            _ => IrGenerationError::Admission(error.diagnostic()),
                        });
                    }
                };
                let mut actual_types = Vec::with_capacity(fields.len());
                for (source_index, (_, value)) in fields.iter().enumerate() {
                    let expected =
                        resolved.contract.fields[resolved.source_to_declaration[source_index]].ty();
                    let actual = if matches!(
                        (value, &expected),
                        (Expression::ArrayLiteral(elements), Ty::Array(_, 0)) if elements.is_empty()
                    ) {
                        expected
                    } else {
                        Self::validate_expression(
                            value,
                            bindings,
                            program,
                            ExpressionUse::Value,
                            inside_impl,
                            admit_static_string_equality,
                        )?
                    };
                    actual_types.push(actual);
                }
                program
                    .structs
                    .validate_construction_types(&resolved, &actual_types)
                    .map_err(|error| IrGenerationError::Admission(error.diagnostic()))?;
                Ok(Ty::Struct(resolved.contract.name))
            }
            Expression::Match { expr, arms } => {
                let scrutinee = if let Expression::Deref(reference) = expr.as_ref() {
                    let operand = Self::validate_expression(
                        reference,
                        bindings,
                        program,
                        ExpressionUse::Value,
                        inside_impl,
                        admit_static_string_equality,
                    )?;
                    match classify_enum_match_dereference(reference, &operand, &program.enums) {
                        LocalReferenceDisposition::Supported(contract) => contract.pointee,
                        LocalReferenceDisposition::ExplicitlyRejected(message) => {
                            return Err(admission_error(&message));
                        }
                        LocalReferenceDisposition::Preserved => Self::validate_expression(
                            expr,
                            bindings,
                            program,
                            ExpressionUse::Value,
                            inside_impl,
                            admit_static_string_equality,
                        )?,
                    }
                } else {
                    Self::validate_expression(
                        expr,
                        bindings,
                        program,
                        ExpressionUse::Value,
                        inside_impl,
                        admit_static_string_equality,
                    )?
                };
                let context = if bindings.contains_key(STRUCT_ADMISSION_BINDING) {
                    EnumExecutionContext::AdmittedFunction
                } else {
                    EnumExecutionContext::PreservedContext
                };
                let patterns = program
                    .enums
                    .resolve_match_patterns(&scrutinee, expr, arms, context)
                    .map_err(|error| admission_error(&error.diagnostic()))?;
                let mut result_types = Vec::with_capacity(arms.len());
                let mut arm_owned_consumptions = Vec::with_capacity(arms.len());
                for (arm, binding) in arms.iter().zip(patterns.payload_bindings.iter()) {
                    let mut arm_bindings = bindings.clone();
                    for binding in binding {
                        arm_bindings.insert(
                            binding.name.clone(),
                            AdmissionBinding {
                                ty: binding.ty.clone(),
                                mutable: false,
                                initialized: true,
                                ownership: OwnershipState::Owned,
                                callable: false,
                                static_string: None,
                            },
                        );
                    }
                    result_types.push(Self::validate_expression(
                        &arm.body,
                        &arm_bindings,
                        program,
                        ExpressionUse::MatchArm,
                        inside_impl,
                        admit_static_string_equality,
                    )?);
                    arm_owned_consumptions.push(
                        program
                            .enums
                            .consumed_owned_values(
                                &arm.body,
                                |name| arm_bindings.get(name).map(|binding| binding.ty.clone()),
                                |name| {
                                    program
                                        .enum_functions
                                        .get(name)
                                        .map(EnumFunctionContract::parameter_types)
                                },
                            )
                            .map_err(|error| admission_error(&error.diagnostic()))?,
                    );
                }
                let consumed = program
                    .enums
                    .consumed_owned_values(
                        expr,
                        |name| bindings.get(name).map(|binding| binding.ty.clone()),
                        |name| {
                            program
                                .enum_functions
                                .get(name)
                                .map(EnumFunctionContract::parameter_types)
                        },
                    )
                    .map_err(|error| admission_error(&error.diagnostic()))?;
                let result = program
                    .enums
                    .resolve_match_with_consumed(
                        &scrutinee,
                        expr,
                        arms,
                        &result_types,
                        &consumed,
                        &arm_owned_consumptions,
                        |ty| {
                            program
                                .structs
                                .resolve_copy_type(ty)
                                .map(|contract| contract.logical_type)
                        },
                        |name| Self::admission_direct_owned_enum_result_type(name, bindings),
                        |name| {
                            program
                                .enum_functions
                                .get(name)
                                .map(|contract| contract.result.ty.clone())
                        },
                        context,
                    )
                    .map(|resolved| resolved.result_contract.ty())
                    .map_err(|error| admission_error(&error.diagnostic()))?;
                validate_enum_reference_match_result(expr, &result, &program.structs)
                    .map_err(|message| admission_error(&message))?;
                Ok(result)
            }
            Expression::TupleLiteral(elements) => {
                let context = if bindings.contains_key(STRUCT_ADMISSION_BINDING) && !inside_impl {
                    TupleExecutionContext::AdmittedFunction
                } else {
                    TupleExecutionContext::PreservedContext
                };
                let mut element_types = Vec::with_capacity(elements.len());
                for element in elements {
                    element_types.push(Self::validate_expression(
                        element,
                        bindings,
                        program,
                        ExpressionUse::Value,
                        inside_impl,
                        admit_static_string_equality,
                    )?);
                }
                match classify_copy_tuple_elements(&element_types, &program.structs, context) {
                    TupleContractDisposition::Supported(contract) => Ok(contract.ty()),
                    TupleContractDisposition::ExplicitlyRejected(message) => {
                        Err(admission_error(&message))
                    }
                    TupleContractDisposition::Preserved => Err(admission_error(
                        "aggregate expression is not admitted in checked IR",
                    )),
                }
            }
            Expression::TupleIndex { object, index } => {
                let context = if bindings.contains_key(STRUCT_ADMISSION_BINDING) && !inside_impl {
                    TupleExecutionContext::AdmittedFunction
                } else {
                    TupleExecutionContext::PreservedContext
                };
                let receiver = Self::validate_expression(
                    object,
                    bindings,
                    program,
                    ExpressionUse::Value,
                    inside_impl,
                    admit_static_string_equality,
                )?;
                match classify_tuple_projection(&receiver, *index, &program.structs, context) {
                    TupleContractDisposition::Supported(contract) => Ok(contract.element),
                    TupleContractDisposition::ExplicitlyRejected(message) => {
                        Err(admission_error(&message))
                    }
                    TupleContractDisposition::Preserved => Err(admission_error(
                        "aggregate expression is not admitted in checked IR",
                    )),
                }
            }
        }
    }

    fn static_string_value(
        expression: &Expression,
        bindings: &HashMap<String, AdmissionBinding>,
    ) -> Option<String> {
        match expression {
            Expression::StringLiteral(value) => Some(value.clone()),
            Expression::Identifier(name) => bindings
                .get(name)
                .and_then(|binding| binding.static_string.clone()),
            _ => None,
        }
    }

    fn admission_type(ty: &Type) -> Ty {
        match ty {
            Type::Named(name) if PrimitiveKind::from_source_name(name).is_some() => {
                PrimitiveKind::from_source_name(name).unwrap().ty()
            }
            Type::Named(name) if matches!(name.as_str(), "string" | "String") => Ty::String,
            Type::Named(name) => Ty::Struct(name.clone()),
            Type::Array(element, count) => {
                Ty::Array(Box::new(Self::admission_type(element)), *count)
            }
            Type::Tuple(elements) => Ty::Tuple(elements.iter().map(Self::admission_type).collect()),
            Type::Reference(inner, mutable) => {
                Ty::Reference(Box::new(Self::admission_type(inner)), *mutable)
            }
            Type::Generic(name, arguments) if name == "Vec" && arguments.len() == 1 => {
                Ty::Vec(Box::new(Self::admission_type(&arguments[0])))
            }
            Type::Generic(name, _) => Ty::TypeParam(name.clone()),
        }
    }

    fn constant_integer_value(expression: &Expression) -> Option<i64> {
        match expression {
            Expression::IntegerLiteral(value) => Some(*value),
            Expression::Unary {
                op: crate::ast::UnaryOp::Negate,
                operand,
            } => Self::constant_integer_value(operand)?.checked_neg(),
            Expression::Binary {
                op, left, right, ..
            } => {
                let left = Self::constant_integer_value(left)?;
                let right = Self::constant_integer_value(right)?;
                match op {
                    crate::ast::BinaryOp::Add => left.checked_add(right),
                    crate::ast::BinaryOp::Subtract => left.checked_sub(right),
                    crate::ast::BinaryOp::Multiply => left.checked_mul(right),
                    crate::ast::BinaryOp::Divide => left.checked_div(right),
                    crate::ast::BinaryOp::Modulo => None,
                }
            }
            _ => None,
        }
    }

    fn callable_result_type(signature: &str) -> Result<Ty, IrGenerationError> {
        match signature.rsplit_once("->").map(|(_, result)| result) {
            Some("int") => Ok(Ty::Int),
            Some("float") => Ok(Ty::Float),
            Some("bool") => Ok(Ty::Bool),
            _ => Err(IrGenerationError::Admission(
                "callable signature is not an admitted scalar signature".to_string(),
            )),
        }
    }

    fn classified_call_result(
        disposition: FunctionCallDisposition,
    ) -> Result<Ty, IrGenerationError> {
        match disposition {
            FunctionCallDisposition::Supported(contract) => Ok(contract.result),
            FunctionCallDisposition::ExplicitlyRejected(diagnostic)
            | FunctionCallDisposition::PreservedContext(diagnostic) => {
                Err(IrGenerationError::Admission(diagnostic))
            }
        }
    }

    fn normalize_checked_place_ids(
        functions: &mut HashMap<String, Function>,
        place_hints: &mut PlaceTypeHints,
    ) {
        let mut reference_parameter_indices = HashMap::<String, Vec<usize>>::new();
        for function in functions.values() {
            Self::collect_reference_parameter_indices(
                &function.body,
                &mut reference_parameter_indices,
            );
        }
        for function in functions.values_mut() {
            Self::normalize_instruction_places(
                &mut function.body,
                &function.name,
                place_hints,
                &reference_parameter_indices,
            );
        }
    }

    fn collect_reference_parameter_indices(
        instructions: &[Inst],
        indices: &mut HashMap<String, Vec<usize>>,
    ) {
        for instruction in instructions {
            match instruction {
                Inst::CheckedFunctionDef {
                    name,
                    parameters,
                    body,
                    ..
                } => {
                    indices.insert(
                        name.clone(),
                        parameters
                            .iter()
                            .enumerate()
                            .filter_map(|(index, (_, ty))| {
                                matches!(
                                    ty,
                                    LogicalType::ImmutableReference { .. }
                                        | LogicalType::MutableReference { .. }
                                )
                                .then_some(index)
                            })
                            .collect(),
                    );
                    Self::collect_reference_parameter_indices(body, indices);
                }
                Inst::FunctionDef { body, .. } => {
                    Self::collect_reference_parameter_indices(body, indices)
                }
                _ => {}
            }
        }
    }

    fn ensure_checked_main_terminator(functions: &mut HashMap<String, Function>) {
        let main_has_emitted_definition = functions.values().any(|function| {
            function.body.iter().any(
                |instruction| matches!(instruction, Inst::FunctionDef { name, .. } if name == "main"),
            )
        });
        if main_has_emitted_definition {
            return;
        }
        if let Some(main) = functions.get_mut("main")
            && !main
                .body
                .last()
                .is_some_and(Self::instruction_terminates_block)
        {
            main.body.push(Inst::Return(Value::ImmInt(0)));
        }
    }

    fn normalize_instruction_places(
        instructions: &mut [Inst],
        function_name: &str,
        place_hints: &mut PlaceTypeHints,
        reference_parameter_indices: &HashMap<String, Vec<usize>>,
    ) {
        let mut maximum_result = None::<u32>;
        for instruction in instructions.iter() {
            Self::visit_result_definitions(instruction, &mut |register| {
                maximum_result =
                    Some(maximum_result.map_or(register, |maximum| maximum.max(register)));
            });
        }
        let mut next_place = maximum_result.map_or(0, |maximum| maximum.saturating_add(1));
        let mut places = HashMap::<u32, u32>::new();
        for instruction in instructions.iter() {
            match instruction {
                Inst::Alloca(Value::Reg(register), _)
                | Inst::CheckedMutableOwnedPlaceAlloca {
                    result: Value::Reg(register),
                    ..
                }
                | Inst::CheckedImmutableEnumOwnerPlaceAlloca {
                    result: Value::Reg(register),
                    ..
                }
                | Inst::CheckedMatchResultPlaceAlloca {
                    result: Value::Reg(register),
                    ..
                }
                | Inst::AllocaArray {
                    result: Value::Reg(register),
                    ..
                }
                | Inst::GetElementPtr {
                    result: Value::Reg(register),
                    ..
                }
                | Inst::CheckedCopyStructArrayAlloca {
                    result: Value::Reg(register),
                    ..
                }
                | Inst::CheckedCopyStructArrayElementPtr {
                    result: Value::Reg(register),
                    ..
                }
                | Inst::CheckedStructAlloca {
                    result: Value::Reg(register),
                    ..
                }
                | Inst::CheckedStructFieldPtr {
                    result: Value::Reg(register),
                    ..
                }
                | Inst::CheckedTupleAlloca {
                    result: Value::Reg(register),
                    ..
                }
                | Inst::CheckedTupleFieldPtr {
                    result: Value::Reg(register),
                    ..
                }
                | Inst::CheckedImmutableBorrow {
                    result: Value::Reg(register),
                    ..
                }
                | Inst::CheckedMutableBorrow {
                    result: Value::Reg(register),
                    ..
                }
                | Inst::CheckedProjectedBorrow {
                    result: Value::Reg(register),
                    ..
                }
                | Inst::CheckedByteBufferNew {
                    result: Value::Reg(register),
                    ..
                }
                | Inst::CheckedByteBufferMove {
                    result: Value::Reg(register),
                    ..
                }
                | Inst::CheckedByteBufferImmutableBorrow {
                    result: Value::Reg(register),
                    ..
                }
                | Inst::CheckedByteBufferMutableBorrow {
                    result: Value::Reg(register),
                    ..
                }
                | Inst::CheckedImmutableReferenceParameter {
                    result: Value::Reg(register),
                    ..
                }
                | Inst::CheckedMutableReferenceParameter {
                    result: Value::Reg(register),
                    ..
                } => {
                    places.entry(*register).or_insert_with(|| {
                        let place = next_place;
                        next_place = next_place.saturating_add(1);
                        place
                    });
                }
                _ => {}
            }
        }

        if let Some(hints) = place_hints.get_mut(function_name) {
            let remapped = std::mem::take(hints)
                .into_iter()
                .map(|(id, ty)| {
                    let normalized = places.get(&id.0).copied().unwrap_or(id.0);
                    (PlaceId(normalized), ty)
                })
                .collect();
            *hints = remapped;
        }

        for instruction in instructions {
            match instruction {
                Inst::Alloca(place, _) => Self::rewrite_place(place, &places),
                Inst::CheckedMutableOwnedPlaceAlloca { result, .. } => {
                    Self::rewrite_place(result, &places)
                }
                Inst::CheckedImmutableEnumOwnerPlaceAlloca { result, .. } => {
                    Self::rewrite_place(result, &places)
                }
                Inst::CheckedMatchResultPlaceAlloca { result, .. } => {
                    Self::rewrite_place(result, &places)
                }
                Inst::CheckedOwnedPlaceAssignment { target, .. } => {
                    Self::rewrite_place(target, &places)
                }
                Inst::AllocaArray { result, .. } => Self::rewrite_place(result, &places),
                Inst::GetElementPtr { result, base, .. } => {
                    Self::rewrite_place(result, &places);
                    Self::rewrite_place(base, &places);
                }
                Inst::CheckedCopyStructArrayAlloca { result, .. } => {
                    Self::rewrite_place(result, &places)
                }
                Inst::CheckedCopyStructArrayElementPtr { result, base, .. } => {
                    Self::rewrite_place(result, &places);
                    Self::rewrite_place(base, &places);
                }
                Inst::CheckedStructAlloca { result, .. } => Self::rewrite_place(result, &places),
                Inst::CheckedStructFieldPtr { result, base, .. } => {
                    Self::rewrite_place(result, &places);
                    Self::rewrite_place(base, &places);
                }
                Inst::CheckedTupleAlloca { result, .. } => Self::rewrite_place(result, &places),
                Inst::CheckedTupleFieldPtr { result, base, .. } => {
                    Self::rewrite_place(result, &places);
                    Self::rewrite_place(base, &places);
                }
                Inst::CheckedImmutableBorrow { result, source, .. } => {
                    Self::rewrite_place(result, &places);
                    Self::rewrite_place(source, &places);
                }
                Inst::CheckedImmutableEnumMatchRead { reference, .. }
                | Inst::CheckedMutableEnumMatchRead { reference, .. } => {
                    Self::rewrite_place(reference, &places)
                }
                Inst::CheckedMutableBorrow { result, source, .. } => {
                    Self::rewrite_place(result, &places);
                    Self::rewrite_place(source, &places);
                }
                Inst::CheckedProjectedBorrow {
                    result,
                    root,
                    source,
                    ..
                } => {
                    Self::rewrite_place(result, &places);
                    Self::rewrite_place(root, &places);
                    Self::rewrite_place(source, &places);
                }
                Inst::CheckedByteBufferNew { result, .. } => Self::rewrite_place(result, &places),
                Inst::CheckedByteBufferMove { result, source, .. }
                | Inst::CheckedByteBufferImmutableBorrow { result, source }
                | Inst::CheckedByteBufferMutableBorrow { result, source } => {
                    Self::rewrite_place(result, &places);
                    Self::rewrite_place(source, &places);
                }
                Inst::CheckedByteBufferImmutableBorrowEnd { reference, source }
                | Inst::CheckedByteBufferMutableBorrowEnd { reference, source } => {
                    Self::rewrite_place(reference, &places);
                    Self::rewrite_place(source, &places);
                }
                Inst::CheckedByteBufferPush { reference, .. }
                | Inst::CheckedByteBufferLength { reference, .. }
                | Inst::CheckedByteBufferCapacity { reference, .. }
                | Inst::CheckedByteBufferGet { reference, .. } => {
                    Self::rewrite_place(reference, &places)
                }
                Inst::CheckedByteBufferDrop { owner } => Self::rewrite_place(owner, &places),
                Inst::CheckedMutableDereferenceAssignment { target, .. } => {
                    Self::rewrite_place(target, &places);
                }
                Inst::CheckedMutableBorrowEnd {
                    reference, source, ..
                }
                | Inst::CheckedMutableOwnerImmutableEnumBorrowEnd {
                    reference, source, ..
                } => {
                    Self::rewrite_place(reference, &places);
                    Self::rewrite_place(source, &places);
                }
                Inst::CheckedProjectedBorrowEnd {
                    reference,
                    root,
                    source,
                    ..
                } => {
                    Self::rewrite_place(reference, &places);
                    Self::rewrite_place(root, &places);
                    Self::rewrite_place(source, &places);
                }
                Inst::CheckedImmutableReferenceParameter { result, .. }
                | Inst::CheckedMutableReferenceParameter { result, .. } => {
                    Self::rewrite_place(result, &places);
                }
                Inst::Store(place, _) => Self::rewrite_place(place, &places),
                Inst::Load(_, place) => Self::rewrite_place(place, &places),
                Inst::Call {
                    function,
                    arguments,
                    ..
                } => {
                    if let Some(indices) = reference_parameter_indices.get(function) {
                        for index in indices {
                            if let Some(argument) = arguments.get_mut(*index) {
                                Self::rewrite_place(argument, &places);
                            }
                        }
                    }
                }
                Inst::FunctionDef { name, body, .. }
                | Inst::CheckedFunctionDef { name, body, .. } => {
                    Self::normalize_instruction_places(
                        body,
                        name,
                        place_hints,
                        reference_parameter_indices,
                    )
                }
                _ => {}
            }
        }
    }

    fn rewrite_place(value: &mut Value, places: &HashMap<u32, u32>) {
        if let Value::Reg(register) = value
            && let Some(replacement) = places.get(register)
        {
            *register = *replacement;
        }
    }

    fn visit_result_definitions(instruction: &Inst, visitor: &mut impl FnMut(u32)) {
        let result = match instruction {
            Inst::Add(result, ..)
            | Inst::FAdd(result, ..)
            | Inst::Sub(result, ..)
            | Inst::FSub(result, ..)
            | Inst::Mul(result, ..)
            | Inst::FMul(result, ..)
            | Inst::Div(result, ..)
            | Inst::FDiv(result, ..)
            | Inst::Load(result, ..)
            | Inst::SIToFP(result, ..)
            | Inst::FPToSI(result, ..)
            | Inst::ICmp { result, .. }
            | Inst::FCmp { result, .. }
            | Inst::And { result, .. }
            | Inst::Or { result, .. }
            | Inst::Not { result, .. }
            | Inst::Neg { result, .. }
            | Inst::CheckedEnumParameter { result, .. }
            | Inst::CheckedEnumVariant { result, .. }
            | Inst::CheckedEnumVariantFields { result, .. }
            | Inst::CheckedEnumPayload { result, .. }
            | Inst::CheckedEnumField { result, .. }
            | Inst::CheckedImmutableEnumMatchRead { result, .. }
            | Inst::CheckedMutableEnumMatchRead { result, .. }
            | Inst::CheckedByteBufferPush { result, .. }
            | Inst::CheckedByteBufferLength { result, .. }
            | Inst::CheckedByteBufferCapacity { result, .. }
            | Inst::CheckedByteBufferGet { result, .. } => Some(result),
            Inst::Call {
                result: Some(result),
                ..
            } => Some(result),
            Inst::FunctionDef { body, .. } | Inst::CheckedFunctionDef { body, .. } => {
                for nested in body {
                    Self::visit_result_definitions(nested, visitor);
                }
                None
            }
            _ => None,
        };
        if let Some(Value::Reg(register)) = result {
            visitor(*register);
        }
    }

    fn instruction_terminates_block(inst: &Inst) -> bool {
        matches!(
            inst,
            Inst::Return(_)
                | Inst::Jump(_)
                | Inst::Branch { .. }
                | Inst::CheckedEnumDispatch { .. }
        )
    }

    fn generate_enum_match_ir(
        &mut self,
        scrutinee_expression: Expression,
        arms: Vec<crate::ast::MatchArm>,
        function: &mut Function,
    ) -> (Value, Ty) {
        let (scrutinee, scrutinee_ty) = if let Expression::Deref(reference) = &scrutinee_expression
        {
            let (place, operand) =
                self.generate_expression_ir(reference.as_ref().clone(), function);
            match classify_enum_match_dereference(reference, &operand, &self.enum_registry) {
                LocalReferenceDisposition::Supported(contract) => {
                    let mutable = contract.mutable;
                    let pointee = contract.pointee;
                    let LogicalType::Enum { name, variants } = contract.logical_pointee else {
                        unreachable!("checked enum-reference Match read has an enum schema")
                    };
                    let result = Value::Reg(self.next_reg);
                    self.next_reg += 1;
                    let schema = EnumSchema { name, variants };
                    function.body.push(if mutable {
                        Inst::CheckedMutableEnumMatchRead {
                            result: result.clone(),
                            reference: place,
                            schema,
                        }
                    } else {
                        Inst::CheckedImmutableEnumMatchRead {
                            result: result.clone(),
                            reference: place,
                            schema,
                        }
                    });
                    (result, pointee)
                }
                LocalReferenceDisposition::ExplicitlyRejected(message) => {
                    unreachable!("checked enum-reference Match admission escaped: {message}")
                }
                LocalReferenceDisposition::Preserved => {
                    self.generate_expression_ir(scrutinee_expression.clone(), function)
                }
            }
        } else {
            self.generate_expression_ir(scrutinee_expression.clone(), function)
        };
        let resolved = self
            .enum_registry
            .resolve_match_patterns(
                &scrutinee_ty,
                &scrutinee_expression,
                &arms,
                EnumExecutionContext::AdmittedFunction,
            )
            .expect("checked enum match was admitted");
        let result_place_id = self.next_ptr;
        let result_place = Value::Reg(result_place_id);
        self.next_ptr += 1;
        let result_place_position = function.body.len();
        function.body.push(Inst::CheckedMatchResultPlaceAlloca {
            result: result_place.clone(),
            result_type: LogicalType::Void,
            dispatch_schema: resolved.contract.schema.clone(),
        });

        let lowering_arms = resolved
            .arm_for_variant
            .iter()
            .map(|source_index| {
                (
                    arms[*source_index].clone(),
                    resolved.payload_bindings[*source_index].clone(),
                )
            })
            .collect::<Vec<_>>();
        let arm_labels = (0..lowering_arms.len())
            .map(|_| {
                let label = format!("match_arm_{}", self.next_reg);
                self.next_reg += 1;
                label
            })
            .collect::<Vec<_>>();
        let end_label = format!("match_end_{}", self.next_reg);
        self.next_reg += 1;
        function.body.push(Inst::CheckedEnumDispatch {
            value: scrutinee.clone(),
            schema: resolved.contract.schema.clone(),
            targets: arm_labels.clone(),
        });

        let mut result_ty = None;
        for ((arm, binding), label) in lowering_arms.into_iter().zip(arm_labels) {
            function.body.push(Inst::Label(label));
            let before_scope = self.scope_snapshot();
            for binding in binding {
                let payload = Value::Reg(self.next_reg);
                self.next_reg += 1;
                if resolved.contract.payload_types(binding.variant_index).len() == 1 {
                    function.body.push(Inst::CheckedEnumPayload {
                        result: payload.clone(),
                        value: scrutinee.clone(),
                        schema: resolved.contract.schema.clone(),
                        variant_index: binding.variant_index,
                    });
                } else {
                    function.body.push(Inst::CheckedEnumField {
                        result: payload.clone(),
                        value: scrutinee.clone(),
                        schema: resolved.contract.schema.clone(),
                        variant_index: binding.variant_index,
                        field_index: binding.field_index,
                    });
                }
                let place = if matches!(&binding.ty, Ty::Struct(_) | Ty::Array(_, _) | Ty::Tuple(_))
                {
                    self.store_copy_aggregate_value(payload, &binding.ty, function)
                } else {
                    let place = Value::Reg(self.next_ptr);
                    self.next_ptr += 1;
                    function.body.push(Inst::Alloca(
                        place.clone(),
                        format!("__enum_payload_{}", binding.name),
                    ));
                    function.body.push(Inst::Store(place.clone(), payload));
                    place
                };
                self.immutable_owned_enum_places.remove(&binding.name);
                self.mutable_owned_enum_places.remove(&binding.name);
                self.symbol_table.insert(binding.name, (place, binding.ty));
            }
            let (value, ty) = self.generate_expression_ir(arm.body, function);
            self.restore_bindings(&before_scope);
            if let Some(expected) = &result_ty {
                assert_eq!(expected, &ty, "checked match result type remains exact");
            } else {
                result_ty = Some(ty.clone());
            }
            if !matches!(ty, Ty::Void) {
                let value = self.load_copy_aggregate_value(value, &ty, function);
                function.body.push(Inst::CheckedOwnedPlaceAssignment {
                    target: result_place.clone(),
                    value,
                    ty: self.admitted_owned_place_logical_type(&ty),
                });
            }
            function.body.push(Inst::Jump(end_label.clone()));
        }
        function.body.push(Inst::Label(end_label));
        let result_ty = result_ty.expect("admitted enum has at least one arm");
        if matches!(result_ty, Ty::Void) {
            let removed = function.body.remove(result_place_position);
            assert!(
                matches!(removed, Inst::CheckedMatchResultPlaceAlloca { .. }),
                "checked Void Match removes only its unused result-place placeholder"
            );
            return (Value::ImmInt(0), Ty::Void);
        }
        let result_type = self.admitted_owned_place_logical_type(&result_ty);
        let Inst::CheckedMatchResultPlaceAlloca {
            result_type: stored_result_type,
            ..
        } = &mut function.body[result_place_position]
        else {
            unreachable!("checked Match result place remains at its recorded position")
        };
        *stored_result_type = result_type;
        let result = Value::Reg(self.next_reg);
        self.next_reg += 1;
        function.body.push(Inst::Load(result.clone(), result_place));
        let result = self.store_copy_aggregate_value(result, &result_ty, function);
        (result, result_ty)
    }

    fn scope_snapshot(&self) -> GeneratedScopeSnapshot {
        GeneratedScopeSnapshot {
            bindings: self.symbol_table.clone(),
            immutable_owned_enum_places: self.immutable_owned_enum_places.clone(),
            mutable_owned_enum_places: self.mutable_owned_enum_places.clone(),
            mutable_owner_immutable_enum_reference_sources: self
                .mutable_owner_immutable_enum_reference_sources
                .clone(),
        }
    }

    fn restore_bindings(&mut self, before_scope: &GeneratedScopeSnapshot) {
        self.symbol_table.clone_from(&before_scope.bindings);
        self.immutable_owned_enum_places
            .clone_from(&before_scope.immutable_owned_enum_places);
        self.mutable_owned_enum_places
            .clone_from(&before_scope.mutable_owned_enum_places);
        self.mutable_owner_immutable_enum_reference_sources
            .clone_from(&before_scope.mutable_owner_immutable_enum_reference_sources);
    }

    fn end_new_mutable_references(
        &mut self,
        before_scope: &HashMap<String, (Value, Ty)>,
        function: &mut Function,
    ) {
        let ended = self
            .symbol_table
            .iter()
            .filter_map(|(name, (reference, ty))| {
                let Ty::Reference(pointee, true) = ty else {
                    return None;
                };
                let existed = before_scope
                    .get(name)
                    .is_some_and(|(prior, prior_ty)| prior == reference && prior_ty == ty);
                if existed {
                    return None;
                }
                let Value::Reg(reference_id) = reference else {
                    unreachable!("checked mutable references use place identifiers")
                };
                let source = self
                    .mutable_reference_sources
                    .get(reference_id)
                    .expect("checked mutable reference retains direct source")
                    .clone();
                let pointee = self.admitted_reference_pointee_logical_type(
                    pointee,
                    ReferencePointeeContext::Mutable,
                );
                Some((*reference_id, reference.clone(), source, pointee))
            })
            .collect::<Vec<_>>();
        for (id, reference, source, pointee) in ended {
            function.body.push(Inst::CheckedMutableBorrowEnd {
                reference,
                source,
                pointee,
            });
            self.mutable_reference_sources.remove(&id);
        }
    }

    fn end_new_mutable_owner_immutable_enum_references(
        &mut self,
        before_scope: &HashMap<String, (Value, Ty)>,
        function: &mut Function,
    ) {
        let ended = self
            .symbol_table
            .iter()
            .filter_map(|(name, (reference, ty))| {
                let Ty::Reference(pointee, false) = ty else {
                    return None;
                };
                if !matches!(pointee.as_ref(), Ty::Enum(_)) {
                    return None;
                }
                let existed = before_scope
                    .get(name)
                    .is_some_and(|(prior, prior_ty)| prior == reference && prior_ty == ty);
                if existed {
                    return None;
                }
                let Value::Reg(reference_id) = reference else {
                    unreachable!("checked immutable enum references use place identifiers")
                };
                self.mutable_owner_immutable_enum_reference_sources
                    .get(reference_id)
                    .map(|(source, schema)| {
                        (
                            *reference_id,
                            reference.clone(),
                            source.clone(),
                            schema.clone(),
                        )
                    })
            })
            .collect::<Vec<_>>();
        for (id, reference, source, schema) in ended {
            function
                .body
                .push(Inst::CheckedMutableOwnerImmutableEnumBorrowEnd {
                    reference,
                    source,
                    schema,
                });
            self.mutable_owner_immutable_enum_reference_sources
                .remove(&id);
        }
    }

    fn end_new_lexical_references(
        &mut self,
        before_scope: &HashMap<String, (Value, Ty)>,
        function: &mut Function,
    ) {
        self.end_new_mutable_owner_immutable_enum_references(before_scope, function);
        self.end_new_mutable_references(before_scope, function);
    }

    fn end_all_active_mutable_owner_immutable_enum_references(&mut self, function: &mut Function) {
        let nonlocal_bindings = self
            .symbol_table
            .iter()
            .filter(|(_, (value, _))| {
                let Value::Reg(id) = value else {
                    return true;
                };
                !self
                    .mutable_owner_immutable_enum_reference_sources
                    .contains_key(id)
            })
            .map(|(name, binding)| (name.clone(), binding.clone()))
            .collect::<HashMap<_, _>>();
        self.end_new_mutable_owner_immutable_enum_references(&nonlocal_bindings, function);
    }

    fn generate_binding_expression_ir(
        &mut self,
        expression: Expression,
        type_annotation: Option<&Type>,
        current_function: &mut Function,
    ) -> (Value, Ty) {
        let typed_empty = self.checked_mode.then(|| {
            let Expression::ArrayLiteral(elements) = &expression else {
                return None;
            };
            if !elements.is_empty() {
                return None;
            }
            let contract = self
                .struct_registry
                .resolve_copy_annotation(type_annotation?)?;
            matches!(contract.ty, Ty::Array(_, 0)).then_some(contract.ty)
        });
        if let Some(Some(expected)) = typed_empty {
            let place = self.allocate_fixed_copy_array_place(&expected, current_function);
            return (place, expected);
        }
        self.generate_expression_ir(expression, current_function)
    }

    fn allocate_copy_struct_place(&mut self, ty: &Ty, function: &mut Function) -> Value {
        let contract = self
            .struct_registry
            .copy_struct_contract(ty)
            .expect("checked Copy-struct type has a shared contract");
        let place = Value::Reg(self.next_ptr);
        self.next_ptr += 1;
        function.body.push(Inst::CheckedStructAlloca {
            result: place.clone(),
            struct_name: contract.name,
            field_types: contract
                .fields
                .iter()
                .map(crate::struct_contract::StructFieldContract::logical_type)
                .collect(),
        });
        place
    }

    fn load_copy_struct_value(&mut self, place: Value, function: &mut Function) -> Value {
        let value = Value::Reg(self.next_reg);
        self.next_reg += 1;
        function.body.push(Inst::Load(value.clone(), place));
        value
    }

    fn store_copy_struct_value(&mut self, value: Value, ty: &Ty, function: &mut Function) -> Value {
        let place = self.allocate_copy_struct_place(ty, function);
        function.body.push(Inst::Store(place.clone(), value));
        place
    }

    fn copy_tuple_contract(&self, ty: &Ty) -> CopyTupleContract {
        let Ty::Tuple(elements) = ty else {
            unreachable!("Copy tuple lowering requires a tuple type")
        };
        match classify_copy_tuple_elements(
            elements,
            &self.struct_registry,
            TupleExecutionContext::AdmittedFunction,
        ) {
            TupleContractDisposition::Supported(contract) => contract,
            _ => unreachable!("checked admission retains a supported recursive Copy tuple"),
        }
    }

    fn allocate_copy_tuple_place(&mut self, ty: &Ty, function: &mut Function) -> Value {
        let contract = self.copy_tuple_contract(ty);
        let LogicalType::Tuple { elements } = contract.logical_type() else {
            unreachable!("Copy tuple contract has tuple logical type")
        };
        let place = Value::Reg(self.next_ptr);
        self.next_ptr += 1;
        function.body.push(Inst::CheckedTupleAlloca {
            result: place.clone(),
            element_types: elements,
        });
        place
    }

    fn copy_tuple_field_ptr(
        &mut self,
        base: Value,
        contract: &CopyTupleContract,
        index: usize,
        function: &mut Function,
    ) -> Value {
        let LogicalType::Tuple { elements } = contract.logical_type() else {
            unreachable!("Copy tuple contract has tuple logical type")
        };
        let field_type = elements[index].clone();
        let place = Value::Reg(self.next_ptr);
        self.next_ptr += 1;
        function.body.push(Inst::CheckedTupleFieldPtr {
            result: place.clone(),
            base,
            element_types: elements,
            field_index: index,
            field_type,
        });
        place
    }

    fn load_copy_tuple_value(&mut self, place: Value, function: &mut Function) -> Value {
        let value = Value::Reg(self.next_reg);
        self.next_reg += 1;
        function.body.push(Inst::Load(value.clone(), place));
        value
    }

    fn store_copy_tuple_value(&mut self, value: Value, ty: &Ty, function: &mut Function) -> Value {
        let place = self.allocate_copy_tuple_place(ty, function);
        function.body.push(Inst::Store(place.clone(), value));
        place
    }

    fn load_fixed_copy_array_value(&mut self, place: Value, function: &mut Function) -> Value {
        let value = Value::Reg(self.next_reg);
        self.next_reg += 1;
        function.body.push(Inst::Load(value.clone(), place));
        value
    }

    fn load_copy_aggregate_value(
        &mut self,
        place: Value,
        ty: &Ty,
        function: &mut Function,
    ) -> Value {
        match ty {
            Ty::Struct(_) => self.load_copy_struct_value(place, function),
            Ty::Array(_, _) => self.load_fixed_copy_array_value(place, function),
            Ty::Tuple(_) => self.load_copy_tuple_value(place, function),
            _ => place,
        }
    }

    fn allocate_fixed_copy_array_place(&mut self, ty: &Ty, function: &mut Function) -> Value {
        let contract = self
            .struct_registry
            .resolve_copy_type(ty)
            .expect("checked fixed Copy array has one recursive contract");
        let LogicalType::Array { element, count } = contract.logical_type else {
            unreachable!("fixed Copy-array allocation requires an array type")
        };
        let place = Value::Reg(self.next_ptr);
        self.next_ptr += 1;
        function.body.push(Inst::CheckedCopyStructArrayAlloca {
            result: place.clone(),
            element: *element,
            count,
        });
        place
    }

    fn fixed_copy_array_element_ptr(
        &mut self,
        base: Value,
        index: Value,
        ty: &Ty,
        function: &mut Function,
    ) -> Value {
        let contract = self
            .struct_registry
            .resolve_copy_type(ty)
            .expect("checked fixed Copy array has one recursive contract");
        let LogicalType::Array { element, count } = contract.logical_type else {
            unreachable!("fixed Copy-array projection requires an array type")
        };
        let place = Value::Reg(self.next_ptr);
        self.next_ptr += 1;
        function.body.push(Inst::CheckedCopyStructArrayElementPtr {
            result: place.clone(),
            base,
            index,
            element: *element,
            count,
        });
        place
    }

    fn generate_projected_copydata_place(
        &mut self,
        contract: &ProjectedCopyDataPlaceContract,
        array_selectors: &[Expression],
        function: &mut Function,
    ) -> (Value, Value) {
        let (root_place, root_type) = self
            .symbol_table
            .get(&contract.root_name)
            .expect("shared projected-place contract resolved its root")
            .clone();
        debug_assert_eq!(root_type, contract.root_type);
        let mut place = root_place.clone();
        for step in &contract.path {
            place = match step {
                CopyProjectionStep::StructField {
                    receiver,
                    field_index,
                    field,
                } => {
                    let result = Value::Reg(self.next_ptr);
                    self.next_ptr += 1;
                    function.body.push(Inst::CheckedStructFieldPtr {
                        result: result.clone(),
                        base: place,
                        struct_name: receiver.name.clone(),
                        field_index: *field_index as u32,
                        field_type: field.logical_type(),
                    });
                    result
                }
                CopyProjectionStep::TupleElement {
                    receiver,
                    index,
                    element,
                } => {
                    debug_assert_eq!(&receiver.elements[*index], element);
                    self.copy_tuple_field_ptr(place, receiver, *index, function)
                }
                CopyProjectionStep::ArrayElement {
                    receiver,
                    index,
                    element,
                } => {
                    debug_assert!(matches!(
                        receiver,
                        Ty::Array(actual, _) if actual.as_ref() == element
                    ));
                    let index = match index {
                        CopyProjectionIndex::Constant(index) => Value::ImmInt(*index as i64),
                        CopyProjectionIndex::Runtime { selector_ordinal } => {
                            let selector = array_selectors
                                .get(*selector_ordinal)
                                .expect("shared projected-place contract retained selector order")
                                .clone();
                            let (index, index_type) =
                                self.generate_expression_ir(selector, function);
                            debug_assert_eq!(index_type, Ty::Int);
                            index
                        }
                    };
                    self.fixed_copy_array_element_ptr(place, index, receiver, function)
                }
            };
        }
        (root_place, place)
    }

    fn store_fixed_copy_array_value(
        &mut self,
        value: Value,
        ty: &Ty,
        function: &mut Function,
    ) -> Value {
        let place = self.allocate_fixed_copy_array_place(ty, function);
        function.body.push(Inst::Store(place.clone(), value));
        place
    }

    fn store_copy_aggregate_value(
        &mut self,
        value: Value,
        ty: &Ty,
        function: &mut Function,
    ) -> Value {
        match ty {
            Ty::Struct(_) => self.store_copy_struct_value(value, ty, function),
            Ty::Array(_, _) => self.store_fixed_copy_array_value(value, ty, function),
            Ty::Tuple(_) => self.store_copy_tuple_value(value, ty, function),
            _ => value,
        }
    }

    fn allocate_copy_aggregate_place(&mut self, ty: &Ty, function: &mut Function) -> Value {
        match ty {
            Ty::Struct(_) => self.allocate_copy_struct_place(ty, function),
            Ty::Array(_, _) => self.allocate_fixed_copy_array_place(ty, function),
            Ty::Tuple(_) => self.allocate_copy_tuple_place(ty, function),
            _ => unreachable!("Copy aggregate allocation requires an aggregate CopyData type"),
        }
    }

    fn emit_live_byte_buffer_drops(&self, function: &mut Function) {
        for owner in self
            .generated_byte_buffer_owners
            .iter()
            .rev()
            .filter(|owner| owner.live)
        {
            function.body.push(Inst::CheckedByteBufferDrop {
                owner: owner.place.clone(),
            });
        }
    }

    fn generate_byte_buffer_binding(
        &mut self,
        name: &str,
        type_annotation: Option<&Type>,
        value: Option<&Expression>,
        function: &mut Function,
    ) -> bool {
        if !self.checked_mode
            || !self.byte_buffer_source_enabled
            || !type_annotation.is_some_and(is_byte_buffer_annotation)
        {
            return false;
        }
        let value = value.expect("checked ByteBuffer binding is initialized");
        let place = Value::Reg(self.next_ptr);
        self.next_ptr += 1;
        match value {
            Expression::FunctionCall {
                name: intrinsic,
                arguments,
            } => {
                let call = classify_byte_buffer_intrinsic_call(intrinsic, arguments)
                    .expect("checked ByteBuffer constructor has exact syntax")
                    .expect("checked ByteBuffer constructor is reserved");
                debug_assert_eq!(call.intrinsic, ByteBufferIntrinsic::New);
                function.body.push(Inst::CheckedByteBufferNew {
                    result: place.clone(),
                    name: name.to_string(),
                });
            }
            Expression::Identifier(source) => {
                let (source_place, source_ty) = self
                    .symbol_table
                    .get(source)
                    .expect("checked ByteBuffer move source exists")
                    .clone();
                debug_assert_eq!(source_ty, Ty::ByteBuffer);
                function.body.push(Inst::CheckedByteBufferMove {
                    result: place.clone(),
                    source: source_place,
                    name: name.to_string(),
                });
                self.generated_byte_buffer_owners
                    .iter_mut()
                    .rev()
                    .find(|owner| owner.name == *source && owner.live)
                    .expect("checked ByteBuffer move source is live")
                    .live = false;
            }
            _ => unreachable!("checked ByteBuffer binding has exact constructor or move syntax"),
        }
        self.symbol_table
            .insert(name.to_string(), (place.clone(), Ty::ByteBuffer));
        self.generated_byte_buffer_owners
            .push(GeneratedByteBufferOwner {
                name: name.to_string(),
                place,
                live: true,
            });
        true
    }

    fn generate_result_int_int_variant(
        &mut self,
        variant: &str,
        payload: Value,
        function: &mut Function,
    ) -> (Value, EnumSchema) {
        let enum_name = private_result_int_int_name();
        let resolved = self
            .enum_registry
            .resolve_constructor(
                &enum_name,
                variant,
                Some(1),
                EnumExecutionContext::AdmittedFunction,
            )
            .expect("normalized Result<int, int> constructor is admitted");
        let result = Value::Reg(self.next_reg);
        self.next_reg += 1;
        function.body.push(Inst::CheckedEnumVariant {
            result: result.clone(),
            schema: resolved.contract.schema.clone(),
            variant_index: resolved.variant_index,
            payload: Some(payload),
        });
        (result, resolved.contract.schema)
    }

    fn wrap_byte_buffer_status_result(
        &mut self,
        status: Value,
        function: &mut Function,
    ) -> (Value, Ty) {
        let initial_zero = Value::Reg(self.next_reg);
        self.next_reg += 1;
        function.body.push(Inst::Add(
            initial_zero.clone(),
            Value::ImmInt(0),
            Value::ImmInt(0),
        ));
        let (initial, schema) = self.generate_result_int_int_variant("Err", initial_zero, function);
        let result_place = Value::Reg(self.next_ptr);
        let result_name = format!("__aero_byte_buffer_result_{}", self.next_ptr);
        self.next_ptr += 1;
        let logical_type = LogicalType::Enum {
            name: schema.name.clone(),
            variants: schema.variants.clone(),
        };
        function.body.push(Inst::CheckedMutableOwnedPlaceAlloca {
            result: result_place.clone(),
            name: result_name,
            ty: logical_type.clone(),
        });
        function
            .body
            .push(Inst::Store(result_place.clone(), initial));

        let failed = Value::Reg(self.next_reg);
        self.next_reg += 1;
        function.body.push(Inst::ICmp {
            op: "slt".to_string(),
            result: failed.clone(),
            left: status.clone(),
            right: Value::ImmInt(0),
        });
        let error_label = self.fresh_control_label("byte_buffer_error");
        let success_label = self.fresh_control_label("byte_buffer_success");
        let join_label = self.fresh_control_label("byte_buffer_result");
        function.body.push(Inst::Branch {
            condition: failed,
            true_label: error_label.clone(),
            false_label: success_label.clone(),
        });

        function.body.push(Inst::Label(error_label));
        let error_code = Value::Reg(self.next_reg);
        self.next_reg += 1;
        function.body.push(Inst::Sub(
            error_code.clone(),
            Value::ImmInt(0),
            status.clone(),
        ));
        let (error_value, error_schema) =
            self.generate_result_int_int_variant("Err", error_code, function);
        debug_assert_eq!(error_schema, schema);
        function.body.push(Inst::CheckedOwnedPlaceAssignment {
            target: result_place.clone(),
            value: error_value,
            ty: logical_type.clone(),
        });
        function.body.push(Inst::Jump(join_label.clone()));

        function.body.push(Inst::Label(success_label));
        let (success_value, success_schema) =
            self.generate_result_int_int_variant("Ok", status, function);
        debug_assert_eq!(success_schema, schema);
        function.body.push(Inst::CheckedOwnedPlaceAssignment {
            target: result_place.clone(),
            value: success_value,
            ty: logical_type,
        });
        function.body.push(Inst::Jump(join_label.clone()));

        function.body.push(Inst::Label(join_label));
        let result = Value::Reg(self.next_reg);
        self.next_reg += 1;
        function.body.push(Inst::Load(result.clone(), result_place));
        (result, Ty::Enum(schema.name))
    }

    fn generate_byte_input_intrinsic(
        &mut self,
        name: &str,
        arguments: &[Expression],
        function: &mut Function,
    ) -> Option<(Value, Ty)> {
        if !self.checked_mode
            || !self.byte_input_source_enabled
            || !classify_byte_input_intrinsic_call(name, arguments)
                .expect("checked byte-input intrinsic has exact syntax")
        {
            return None;
        }
        let raw = Value::Reg(self.next_reg);
        self.next_reg += 1;
        function.body.push(Inst::CheckedStdinReadByte {
            result: raw.clone(),
        });
        Some(self.wrap_byte_buffer_status_result(raw, function))
    }

    fn generate_byte_buffer_intrinsic(
        &mut self,
        name: &str,
        arguments: &[Expression],
        function: &mut Function,
    ) -> Option<(Value, Ty)> {
        if !self.checked_mode || !self.byte_buffer_source_enabled {
            return None;
        }
        let call = classify_byte_buffer_intrinsic_call(name, arguments)
            .expect("checked byte-buffer intrinsic has exact syntax")?;
        debug_assert_ne!(call.intrinsic, ByteBufferIntrinsic::New);
        let owner = call
            .owner
            .expect("checked byte-buffer operation retains an owner");
        let (owner_place, owner_ty) = self
            .symbol_table
            .get(owner)
            .expect("checked byte-buffer owner exists")
            .clone();
        debug_assert_eq!(owner_ty, Ty::ByteBuffer);
        // Evaluate the scalar completely before opening the operation's immediate
        // loan. This keeps nested, already-validated integer calls from extending
        // or overlapping the outer resource loan.
        let scalar = call.scalar.map(|expression| {
            let (value, ty) = self.generate_expression_ir(expression.clone(), function);
            debug_assert_eq!(ty, Ty::Int);
            value
        });
        let reference = Value::Reg(self.next_ptr);
        self.next_ptr += 1;
        let mutable = call.intrinsic == ByteBufferIntrinsic::Push;
        if mutable {
            function.body.push(Inst::CheckedByteBufferMutableBorrow {
                result: reference.clone(),
                source: owner_place.clone(),
            });
        } else {
            function.body.push(Inst::CheckedByteBufferImmutableBorrow {
                result: reference.clone(),
                source: owner_place.clone(),
            });
        }

        let result = Value::Reg(self.next_reg);
        self.next_reg += 1;
        function.body.push(match call.intrinsic {
            ByteBufferIntrinsic::Push => Inst::CheckedByteBufferPush {
                result: result.clone(),
                reference: reference.clone(),
                byte: scalar.expect("push retains a byte"),
            },
            ByteBufferIntrinsic::Length => Inst::CheckedByteBufferLength {
                result: result.clone(),
                reference: reference.clone(),
            },
            ByteBufferIntrinsic::Capacity => Inst::CheckedByteBufferCapacity {
                result: result.clone(),
                reference: reference.clone(),
            },
            ByteBufferIntrinsic::Get => Inst::CheckedByteBufferGet {
                result: result.clone(),
                reference: reference.clone(),
                index: scalar.expect("get retains an index"),
            },
            ByteBufferIntrinsic::New => unreachable!("constructor lowers through binding"),
        });
        if mutable {
            function.body.push(Inst::CheckedByteBufferMutableBorrowEnd {
                reference,
                source: owner_place,
            });
        } else {
            function
                .body
                .push(Inst::CheckedByteBufferImmutableBorrowEnd {
                    reference,
                    source: owner_place,
                });
        }
        Some(match call.intrinsic {
            ByteBufferIntrinsic::Push | ByteBufferIntrinsic::Get => {
                self.wrap_byte_buffer_status_result(result, function)
            }
            ByteBufferIntrinsic::Length | ByteBufferIntrinsic::Capacity => (result, Ty::Int),
            ByteBufferIntrinsic::New => unreachable!("constructor lowers through binding"),
        })
    }

    fn generate_statement_ir(&mut self, stmt: Statement, current_function: &mut Function) {
        match stmt {
            Statement::Const { .. } => {
                unreachable!("primitive constants are normalized before IR lowering")
            }
            Statement::Let {
                name,
                mutable,
                type_annotation,
                value,
            } => {
                if self.generate_byte_buffer_binding(
                    &name,
                    type_annotation.as_ref(),
                    value.as_ref(),
                    current_function,
                ) {
                    return;
                }
                let binding_name = name.clone();
                let mutable_borrow_source = self
                    .checked_mode
                    .then(|| {
                        value.as_ref().and_then(|value| {
                            let Expression::Borrow {
                                expr,
                                mutable: true,
                            } = value
                            else {
                                return None;
                            };
                            let Expression::Identifier(source) = expr.as_ref() else {
                                return None;
                            };
                            self.symbol_table
                                .get(source)
                                .map(|(place, _)| place.clone())
                        })
                    })
                    .flatten();
                let copies_existing_aggregate =
                    matches!(value.as_ref(), Some(Expression::Identifier(_)));
                let (expr_value, expr_type) = if let Some(val) = value {
                    self.generate_binding_expression_ir(
                        val,
                        type_annotation.as_ref(),
                        current_function,
                    )
                } else {
                    (Value::ImmInt(0), Ty::Int)
                };

                let mutable_owned_place = self.checked_mode
                    && mutable
                    && resolve_owned_place_logical_type(
                        &expr_type,
                        &self.struct_registry,
                        &self.enum_registry,
                    )
                    .is_ok();
                self.immutable_owned_enum_places.remove(&name);
                self.mutable_owned_enum_places.remove(&name);
                if self.checked_mode && !mutable && matches!(expr_type, Ty::Enum(_)) {
                    let LogicalType::Enum {
                        name: enum_name,
                        variants,
                    } = self.admitted_owned_place_logical_type(&expr_type)
                    else {
                        unreachable!("checked immutable enum binding has an exact schema")
                    };
                    let storage = Value::Reg(self.next_ptr);
                    self.next_ptr += 1;
                    current_function
                        .body
                        .push(Inst::CheckedImmutableEnumOwnerPlaceAlloca {
                            result: storage.clone(),
                            name: name.clone(),
                            schema: EnumSchema {
                                name: enum_name,
                                variants,
                            },
                        });
                    current_function
                        .body
                        .push(Inst::Store(storage.clone(), expr_value));
                    self.immutable_owned_enum_places
                        .insert(name.clone(), storage.clone());
                    self.symbol_table.insert(name, (storage, expr_type));
                } else if mutable_owned_place {
                    let initial = if matches!(expr_type, Ty::Enum(_)) {
                        expr_value
                    } else {
                        self.load_copy_aggregate_value(expr_value, &expr_type, current_function)
                    };
                    let storage = Value::Reg(self.next_ptr);
                    self.next_ptr += 1;
                    let ty = self.admitted_owned_place_logical_type(&expr_type);
                    current_function
                        .body
                        .push(Inst::CheckedMutableOwnedPlaceAlloca {
                            result: storage.clone(),
                            name: name.clone(),
                            ty,
                        });
                    current_function
                        .body
                        .push(Inst::Store(storage.clone(), initial));
                    if matches!(expr_type, Ty::Enum(_)) {
                        self.mutable_owned_enum_places
                            .insert(name.clone(), storage.clone());
                    }
                    self.symbol_table.insert(name, (storage, expr_type));
                } else if matches!(&expr_type, Ty::Struct(_) | Ty::Array(_, _) | Ty::Tuple(_)) {
                    let storage = if copies_existing_aggregate {
                        let value = self.load_copy_aggregate_value(
                            expr_value,
                            &expr_type,
                            current_function,
                        );
                        self.store_copy_aggregate_value(value, &expr_type, current_function)
                    } else {
                        expr_value
                    };
                    self.symbol_table.insert(name, (storage, expr_type));
                } else if Self::stores_value_directly(&expr_type) {
                    // Keep string values as immediates for now; pointer-backed string variables
                    // and aggregate values are not fully modeled in the scalar slot pipeline yet.
                    self.symbol_table.insert(name, (expr_value, expr_type));
                } else {
                    // Allocate a stack slot for the variable
                    let ptr_reg = Value::Reg(self.next_ptr);
                    self.next_ptr += 1;
                    current_function
                        .body
                        .push(Inst::Alloca(ptr_reg.clone(), name.clone()));
                    self.symbol_table.insert(name, (ptr_reg.clone(), expr_type));

                    // Store the expression result into the allocated slot
                    current_function.body.push(Inst::Store(ptr_reg, expr_value));
                }
                if let Some(source) = mutable_borrow_source {
                    let (reference, reference_type) = self
                        .symbol_table
                        .get(&binding_name)
                        .expect("mutable reference binding was generated");
                    debug_assert!(matches!(reference_type, Ty::Reference(_, true)));
                    let Value::Reg(reference_id) = reference else {
                        unreachable!("checked mutable reference uses a place identifier")
                    };
                    self.mutable_reference_sources.insert(*reference_id, source);
                }
            }
            Statement::Assignment { target, value } => {
                if self.checked_mode {
                    let array_selectors = projected_copydata_assignment_array_selectors(&target)
                        .expect("checked admission resolved projected assignment topology");
                    let projected = classify_projected_copydata_assignment_after_admission(
                        &target,
                        true,
                        &self.struct_registry,
                        |name| {
                            self.symbol_table.get(name).map(|(_, ty)| {
                                OwnedPlaceAssignmentTargetFacts {
                                    ty: ty.clone(),
                                    mutable: true,
                                    initialized: true,
                                    local: true,
                                    ownership: OwnershipState::Owned,
                                }
                            })
                        },
                    );
                    if let ProjectedCopyDataAssignmentDisposition::Supported(contract) = projected {
                        let array_selectors = array_selectors
                            .expect("supported projected assignment retains its selectors");
                        let array_selectors =
                            array_selectors.into_iter().cloned().collect::<Vec<_>>();
                        let (_, target_place) = self.generate_projected_copydata_place(
                            &contract,
                            &array_selectors,
                            current_function,
                        );
                        let (assigned_value, assigned_type) =
                            self.generate_expression_ir(value, current_function);
                        debug_assert_eq!(assigned_type, contract.leaf_type);
                        let assigned_value = self.load_copy_aggregate_value(
                            assigned_value,
                            &contract.leaf_type,
                            current_function,
                        );
                        current_function
                            .body
                            .push(Inst::CheckedOwnedPlaceAssignment {
                                target: target_place,
                                value: assigned_value,
                                ty: contract.leaf_logical_type,
                            });
                        return;
                    }
                }
                let (assigned_value, assigned_type) =
                    self.generate_expression_ir(value, current_function);
                match target {
                    Expression::Identifier(name) => {
                        let (target_place, target_type) = self
                            .symbol_table
                            .get(&name)
                            .expect("checked admission resolves assignment targets")
                            .clone();
                        debug_assert_eq!(assigned_type, target_type);
                        if self.checked_mode {
                            let assigned_value = self.load_copy_aggregate_value(
                                assigned_value,
                                &assigned_type,
                                current_function,
                            );
                            current_function
                                .body
                                .push(Inst::CheckedOwnedPlaceAssignment {
                                    target: target_place,
                                    value: assigned_value,
                                    ty: self.admitted_owned_place_logical_type(&target_type),
                                });
                        } else {
                            current_function
                                .body
                                .push(Inst::Store(target_place, assigned_value));
                        }
                    }
                    Expression::Deref(_) if !self.checked_mode => {
                        // Raw IR is a compatibility surface and must not acquire the
                        // checked mutable-reference identities. The checked pipeline
                        // below is the sole admitted lowering path for this topology.
                    }
                    Expression::Deref(reference) => {
                        let Expression::Identifier(name) = reference.as_ref() else {
                            unreachable!(
                                "checked admission requires an identifier mutable reference"
                            )
                        };
                        let (target_place, target_type) = self
                            .symbol_table
                            .get(name)
                            .expect("checked admission resolves mutable reference target")
                            .clone();
                        let Ty::Reference(pointee, true) = target_type else {
                            unreachable!("checked admission requires a mutable reference target")
                        };
                        debug_assert_eq!(assigned_type, *pointee);
                        let assigned_value = if matches!(assigned_type, Ty::Enum(_)) {
                            assigned_value
                        } else {
                            self.load_copy_aggregate_value(
                                assigned_value,
                                &assigned_type,
                                current_function,
                            )
                        };
                        current_function
                            .body
                            .push(Inst::CheckedMutableDereferenceAssignment {
                                target: target_place,
                                value: assigned_value,
                                pointee: self.admitted_reference_pointee_logical_type(
                                    &pointee,
                                    ReferencePointeeContext::Mutable,
                                ),
                            });
                    }
                    _ => unreachable!("checked admission rejects assignment target topology"),
                }
            }
            Statement::Return(expr) => {
                let (mut return_value, return_type) = if let Some(val) = expr {
                    self.generate_expression_ir(val, current_function)
                } else {
                    (Value::ImmInt(0), Ty::Int)
                };
                return_value =
                    self.load_copy_aggregate_value(return_value, &return_type, current_function);
                if self.checked_mode
                    && current_function.name == "main"
                    && !self.function_return_types.contains_key("main")
                    && matches!(return_type, Ty::Float)
                {
                    let converted = Value::Reg(self.next_reg);
                    self.next_reg += 1;
                    current_function
                        .body
                        .push(Inst::FPToSI(converted.clone(), return_value));
                    return_value = converted;
                }
                if self.checked_mode {
                    self.end_all_active_mutable_owner_immutable_enum_references(current_function);
                    self.emit_live_byte_buffer_drops(current_function);
                }
                current_function.body.push(Inst::Return(return_value));
            }
            Statement::Function {
                name,
                parameters,
                return_type,
                body,
                ..
            } => {
                self.generate_function_definition_ir(
                    name,
                    parameters,
                    return_type,
                    body,
                    current_function,
                );
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.generate_if_statement_ir(condition, then_block, else_block, current_function);
            }
            Statement::While { condition, body } => {
                let scope_snapshot = self.scope_snapshot();
                self.generate_while_loop_ir(condition, body, current_function);
                self.restore_bindings(&scope_snapshot);
            }
            Statement::For {
                variable,
                iterable,
                body,
            } => {
                let scope_snapshot = self.scope_snapshot();
                self.generate_for_loop_ir(variable, iterable, body, current_function);
                self.restore_bindings(&scope_snapshot);
            }
            Statement::Loop { body } => {
                let scope_snapshot = self.scope_snapshot();
                self.generate_infinite_loop_ir(body, current_function);
                self.restore_bindings(&scope_snapshot);
            }
            Statement::Break => {
                self.generate_break_ir(current_function);
            }
            Statement::Continue => {
                self.generate_continue_ir(current_function);
            }
            Statement::Expression(expr) => {
                // Generate IR for standalone expressions
                self.generate_expression_ir(expr, current_function);
            }
            Statement::Block(block) => {
                let scope_snapshot = self.scope_snapshot();
                // Generate IR for block statements
                for stmt in block.statements {
                    self.generate_statement_ir(stmt, current_function);
                }
                if let Some(expr) = block.expression {
                    self.generate_expression_ir(expr, current_function);
                }
                if self.checked_mode
                    && !current_function
                        .body
                        .last()
                        .is_some_and(Self::instruction_terminates_block)
                {
                    self.end_new_lexical_references(&scope_snapshot.bindings, current_function);
                }
                self.restore_bindings(&scope_snapshot);
            }
            // Phase 4: struct/enum/impl definitions are processed at a higher level;
            // they don't generate body IR in the same way as executable statements.
            Statement::StructDef { .. }
            | Statement::EnumDef { .. }
            | Statement::ImplBlock { .. }
            | Statement::TraitDef { .. }
            | Statement::ModDecl { .. }
            | Statement::UseImport { .. } => {
                // Type/module definitions are registered in the semantic pass.
                // No runtime IR to generate.
            }
        }
    }

    #[inline(never)]
    fn generate_reference_function_call(
        &mut self,
        name: String,
        arguments: Vec<Expression>,
        mutable_calls: Vec<(usize, ReferenceCallSourceMode)>,
        function: &mut Function,
    ) -> (Value, Ty) {
        let mut pending_arguments = arguments.into_iter().map(Some).collect::<Vec<_>>();
        let mut argument_order = (0..pending_arguments.len())
            .filter(|index| {
                !mutable_calls
                    .iter()
                    .any(|(mutable_index, _)| mutable_index == index)
            })
            .collect::<Vec<_>>();
        argument_order.extend(mutable_calls.iter().map(|(index, _)| *index));
        let mut arg_values = vec![None; pending_arguments.len()];
        let mut temporary_mutable_borrows = Vec::new();
        let mut temporary_projected_borrows = Vec::new();
        let mut temporary_mutable_owner_immutable_enum_borrows = Vec::new();

        for index in argument_order {
            let mutable_source_mode = mutable_calls
                .iter()
                .find_map(|(mutable_index, mode)| (*mutable_index == index).then_some(*mode));
            let arg = pending_arguments[index]
                .take()
                .expect("each checked call argument is lowered exactly once");
            let direct_mutable_owner_immutable_enum_borrow =
                matches!(&arg, Expression::Borrow { mutable: false, .. });
            let direct_mutable_source = if mutable_source_mode
                == Some(ReferenceCallSourceMode::DirectOwnerBorrow)
                && let Expression::Borrow {
                    expr,
                    mutable: true,
                } = &arg
                && let Expression::Identifier(source) = expr.as_ref()
            {
                self.symbol_table
                    .get(source)
                    .map(|(place, ty)| (place.clone(), ty.clone()))
            } else {
                None
            };
            let (mut arg_value, arg_type) = self.generate_expression_ir(arg, function);
            if let Value::Reg(reference_id) = &arg_value
                && let Some(source) = self
                    .projected_call_reference_sources
                    .get(reference_id)
                    .cloned()
            {
                temporary_projected_borrows.push((*reference_id, arg_value.clone(), source));
            }
            if direct_mutable_owner_immutable_enum_borrow
                && let Value::Reg(reference_id) = &arg_value
                && let Some((source, schema)) = self
                    .mutable_owner_immutable_enum_reference_sources
                    .get(reference_id)
                    .cloned()
            {
                temporary_mutable_owner_immutable_enum_borrows.push((
                    *reference_id,
                    arg_value.clone(),
                    source,
                    schema,
                ));
            }
            if let Some((source, pointee)) = direct_mutable_source {
                let Ty::Reference(actual, true) = &arg_type else {
                    unreachable!("checked direct mutable call retains reference type")
                };
                debug_assert_eq!(actual.as_ref(), &pointee);
                temporary_mutable_borrows.push((
                    arg_value.clone(),
                    source,
                    self.admitted_reference_pointee_logical_type(
                        &pointee,
                        ReferencePointeeContext::Mutable,
                    ),
                ));
            } else if mutable_source_mode
                == Some(ReferenceCallSourceMode::MutableReferenceIdentifier)
            {
                let Ty::Reference(pointee, true) = &arg_type else {
                    unreachable!("checked mutable-reference identifier call retains reference type")
                };
                let parent = arg_value;
                let child = Value::Reg(self.next_ptr);
                self.next_ptr += 1;
                let logical_pointee = self.admitted_reference_pointee_logical_type(
                    pointee,
                    ReferencePointeeContext::Mutable,
                );
                function.body.push(Inst::CheckedMutableBorrow {
                    result: child.clone(),
                    source: parent.clone(),
                    pointee: logical_pointee.clone(),
                });
                temporary_mutable_borrows.push((child.clone(), parent, logical_pointee));
                arg_value = child;
            }
            let arg_value = self.load_copy_aggregate_value(arg_value, &arg_type, function);
            arg_values[index] = Some(arg_value);
        }

        let arg_values = arg_values
            .into_iter()
            .map(|argument| argument.expect("checked call argument retained its position"))
            .collect();
        let (call_inst, result, return_type) = self.build_function_call(name, arg_values);
        function.body.push(call_inst);
        for (reference, source, pointee) in temporary_mutable_borrows.into_iter().rev() {
            function.body.push(Inst::CheckedMutableBorrowEnd {
                reference,
                source,
                pointee,
            });
        }
        for (id, reference, source) in temporary_projected_borrows.into_iter().rev() {
            function.body.push(Inst::CheckedProjectedBorrowEnd {
                reference,
                root: source.root,
                source: source.source,
                root_type: source.root_type,
                pointee: source.pointee,
                mutable: source.mutable,
            });
            self.projected_call_reference_sources.remove(&id);
        }
        for (id, reference, source, schema) in temporary_mutable_owner_immutable_enum_borrows {
            function
                .body
                .push(Inst::CheckedMutableOwnerImmutableEnumBorrowEnd {
                    reference,
                    source,
                    schema,
                });
            self.mutable_owner_immutable_enum_reference_sources
                .remove(&id);
        }
        if matches!(&return_type, Ty::Struct(_) | Ty::Array(_, _) | Ty::Tuple(_)) {
            let place = self.store_copy_aggregate_value(result, &return_type, function);
            (place, return_type)
        } else {
            (result, return_type)
        }
    }

    fn generate_expression_ir(&mut self, expr: Expression, function: &mut Function) -> (Value, Ty) {
        match expr {
            Expression::IntegerLiteral(n) => (Value::ImmInt(n), Ty::Int),
            Expression::FloatLiteral(f) => (Value::ImmFloat(f), Ty::Float),
            Expression::CharacterLiteral(character) => (Value::ImmChar(character), Ty::Char),
            Expression::Identifier(name) => {
                let (storage, var_type) = self
                    .symbol_table
                    .get(&name)
                    .expect("Undeclared variable")
                    .clone();
                if self.is_immutable_owned_enum_place(&name, &storage, &var_type)
                    || self.is_mutable_owned_enum_place(&name, &storage, &var_type)
                {
                    let result = Value::Reg(self.next_reg);
                    self.next_reg += 1;
                    function.body.push(Inst::Load(result.clone(), storage));
                    return (result, var_type);
                }
                if Self::stores_value_directly(&var_type) {
                    return (storage, var_type);
                }
                let result_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function.body.push(Inst::Load(result_reg.clone(), storage));
                (result_reg, var_type)
            }
            Expression::Binary {
                op,
                left,
                right,
                ty,
            } => {
                let (lhs_val, lhs_type) = self.generate_expression_ir(*left, function);
                let (rhs_val, rhs_type) = self.generate_expression_ir(*right, function);

                // Prefer the result type from the AST (set by semantic analysis).
                //
                // Some transformation/compat codepaths may create `Expression::Binary` nodes
                // without a `ty` annotation. In that case, fall back to local inference
                // based on the operand types so we don't hard-panic during codegen.
                let result_type = ty.unwrap_or_else(|| match (&lhs_type, &rhs_type) {
                    (Ty::Float, _) | (_, Ty::Float) => Ty::Float,
                    (Ty::Int, Ty::Int) => Ty::Int,
                    (l, r) => panic!(
                        "Cannot infer binary op result type for op '{}' with operand types {:?} and {:?}",
                        op.as_str(),
                        l,
                        r
                    ),
                });

                // Handle type promotion if needed
                let (promoted_lhs, promoted_rhs) = self.handle_type_promotion(
                    lhs_val,
                    lhs_type,
                    rhs_val,
                    rhs_type,
                    &result_type,
                    function,
                );

                // Try constant folding first
                if let (Some(folded_value), Some(folded_type)) =
                    self.try_constant_fold(op.as_str(), &promoted_lhs, &promoted_rhs, &result_type)
                {
                    return (folded_value, folded_type);
                }

                let result_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;

                // Generate the appropriate instruction based on result type
                let inst = match (&result_type, op.as_str()) {
                    (Ty::Int, "+") => Inst::Add(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "+") => Inst::FAdd(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Int, "-") => Inst::Sub(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "-") => Inst::FSub(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Int, "*") => Inst::Mul(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "*") => Inst::FMul(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Int, "/") => Inst::Div(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "/") => Inst::FDiv(result_reg.clone(), promoted_lhs, promoted_rhs),
                    _ => panic!(
                        "Unsupported binary operation: {} for type {:?}",
                        op, result_type
                    ),
                };

                function.body.push(inst);
                (result_reg, result_type)
            }
            Expression::FunctionCall { name, arguments } => {
                if let Some(result) =
                    self.generate_byte_input_intrinsic(&name, &arguments, function)
                {
                    return result;
                }
                if let Some(result) =
                    self.generate_byte_buffer_intrinsic(&name, &arguments, function)
                {
                    return result;
                }
                let mutable_calls = self
                    .checked_mode
                    .then(|| self.reference_function_contracts.get(&name))
                    .flatten()
                    .map(|contract| reference_call_source_modes(contract, &arguments))
                    .filter(|calls| !calls.is_empty());
                if let Some(mutable_calls) = mutable_calls {
                    return self.generate_reference_function_call(
                        name,
                        arguments,
                        mutable_calls,
                        function,
                    );
                }
                // Calls without mutable parameters retain the ordinary lowering path.
                let mut arg_values = Vec::new();
                let mut temporary_projected_borrows = Vec::new();
                let mut temporary_mutable_owner_immutable_enum_borrows = Vec::new();
                for arg in arguments {
                    let direct_mutable_owner_immutable_enum_borrow =
                        matches!(&arg, Expression::Borrow { mutable: false, .. });
                    let (arg_value, arg_type) = self.generate_expression_ir(arg, function);
                    if let Value::Reg(reference_id) = &arg_value
                        && let Some(source) = self
                            .projected_call_reference_sources
                            .get(reference_id)
                            .cloned()
                    {
                        temporary_projected_borrows.push((
                            *reference_id,
                            arg_value.clone(),
                            source,
                        ));
                    }
                    if direct_mutable_owner_immutable_enum_borrow
                        && let Value::Reg(reference_id) = &arg_value
                        && let Some((source, schema)) = self
                            .mutable_owner_immutable_enum_reference_sources
                            .get(reference_id)
                            .cloned()
                    {
                        temporary_mutable_owner_immutable_enum_borrows.push((
                            *reference_id,
                            arg_value.clone(),
                            source,
                            schema,
                        ));
                    }
                    let arg_value = self.load_copy_aggregate_value(arg_value, &arg_type, function);
                    arg_values.push(arg_value);
                }
                let (call_inst, result, return_type) = self.build_function_call(name, arg_values);
                function.body.push(call_inst);
                for (id, reference, source) in temporary_projected_borrows.into_iter().rev() {
                    function.body.push(Inst::CheckedProjectedBorrowEnd {
                        reference,
                        root: source.root,
                        source: source.source,
                        root_type: source.root_type,
                        pointee: source.pointee,
                        mutable: source.mutable,
                    });
                    self.projected_call_reference_sources.remove(&id);
                }
                for (id, reference, source, schema) in
                    temporary_mutable_owner_immutable_enum_borrows
                {
                    function
                        .body
                        .push(Inst::CheckedMutableOwnerImmutableEnumBorrowEnd {
                            reference,
                            source,
                            schema,
                        });
                    self.mutable_owner_immutable_enum_reference_sources
                        .remove(&id);
                }
                if matches!(&return_type, Ty::Struct(_) | Ty::Array(_, _) | Ty::Tuple(_)) {
                    let place = self.store_copy_aggregate_value(result, &return_type, function);
                    (place, return_type)
                } else {
                    (result, return_type)
                }
            }
            Expression::Print {
                format_string,
                arguments,
            } => self.generate_print_ir(format_string, arguments, false, function),
            Expression::Println {
                format_string,
                arguments,
            } => self.generate_print_ir(format_string, arguments, true, function),
            Expression::Comparison { op, left, right } => {
                self.generate_comparison_ir(op, *left, *right, function)
            }
            Expression::Logical { op, left, right } => {
                self.generate_logical_ir(op, *left, *right, function)
            }
            Expression::Unary { op, operand } => self.generate_unary_ir(op, *operand, function),
            Expression::StringLiteral(s) => (Value::ImmString(s), Ty::String),
            Expression::MethodCall {
                object,
                method,
                arguments,
            } => {
                let (object_value, object_ty) = self.generate_expression_ir(*object, function);
                if self.checked_mode {
                    let static_string = match (&object_value, &object_ty) {
                        (Value::ImmString(value), Ty::String) => Some(value.as_str()),
                        _ => None,
                    };
                    let static_arguments = arguments
                        .iter()
                        .cloned()
                        .map(|argument| {
                            let (value, ty) = self.generate_expression_ir(argument, function);
                            match (value, ty) {
                                (Value::ImmString(value), Ty::String) => Some(value),
                                _ => None,
                            }
                        })
                        .collect::<Vec<_>>();
                    let static_argument_refs = static_arguments
                        .iter()
                        .map(|argument| argument.as_deref())
                        .collect::<Vec<_>>();
                    let disposition = classify_intrinsic_method(
                        &object_ty,
                        &method,
                        arguments.len(),
                        static_string,
                        &static_argument_refs,
                        &self.struct_registry,
                        IntrinsicMethodPhase::Checked,
                        false,
                    );
                    if let IntrinsicMethodDisposition::Supported { result, lowering } = disposition
                    {
                        return match lowering {
                            Some(IntrinsicMethodLowering::ConstantInt(value)) => {
                                (Value::ImmInt(i64::from(value)), result)
                            }
                            Some(IntrinsicMethodLowering::ConstantBool(value)) => {
                                let result_reg = Value::Reg(self.next_reg);
                                self.next_reg += 1;
                                function.body.push(Inst::ICmp {
                                    op: if value { "eq" } else { "ne" }.to_string(),
                                    result: result_reg.clone(),
                                    left: Value::ImmInt(0),
                                    right: Value::ImmInt(0),
                                });
                                (result_reg, result)
                            }
                            Some(IntrinsicMethodLowering::Receiver) => (object_value, result),
                            None => unreachable!(
                                "checked intrinsic method classification must include lowering"
                            ),
                        };
                    }
                    unreachable!("checked admission must reject unsupported intrinsic methods");
                }
                if method == "iter"
                    && arguments.is_empty()
                    && matches!(object_ty, Ty::Array(_, _) | Ty::Vec(_))
                {
                    // Minimal iterator protocol lowering: `.iter()` reuses the collection value.
                    (object_value, object_ty)
                } else {
                    // Quarantined legacy unchecked path. Checked IR admission cannot reach this
                    // compatibility placeholder.
                    (Value::ImmInt(0), Ty::Int)
                }
            }
            Expression::ArrayLiteral(elements) => {
                let count = elements.len();
                if self.checked_mode {
                    let Some(first) = elements.first().cloned() else {
                        unreachable!("checked empty arrays require an exact annotation")
                    };
                    let (first_value, element_ty) = self.generate_expression_ir(first, function);
                    let array_ty = Ty::Array(Box::new(element_ty.clone()), count);
                    let array = self.allocate_fixed_copy_array_place(&array_ty, function);
                    let first_value =
                        self.load_copy_aggregate_value(first_value, &element_ty, function);
                    let first_element = self.fixed_copy_array_element_ptr(
                        array.clone(),
                        Value::ImmInt(0),
                        &array_ty,
                        function,
                    );
                    function.body.push(Inst::Store(first_element, first_value));
                    for (index, expression) in elements.into_iter().skip(1).enumerate() {
                        let (value, actual) = self.generate_expression_ir(expression, function);
                        debug_assert_eq!(actual, element_ty);
                        let value = self.load_copy_aggregate_value(value, &actual, function);
                        let element = self.fixed_copy_array_element_ptr(
                            array.clone(),
                            Value::ImmInt((index + 1) as i64),
                            &array_ty,
                            function,
                        );
                        function.body.push(Inst::Store(element, value));
                    }
                    return (array, array_ty);
                }
                let arr_id = self.next_ptr;
                let arr_ptr = Value::Reg(arr_id);
                self.next_ptr += 1;
                // Determine element type from first element
                let elem_type = if count > 0 {
                    let (first_val, first_ty) =
                        self.generate_expression_ir(elements[0].clone(), function);
                    function.body.push(Inst::AllocaArray {
                        result: arr_ptr.clone(),
                        elem_type: "double".to_string(),
                        count,
                    });
                    // Store first element
                    let elem_ptr = Value::Reg(self.next_ptr);
                    self.next_ptr += 1;
                    function.body.push(Inst::GetElementPtr {
                        result: elem_ptr.clone(),
                        base: arr_ptr.clone(),
                        index: Value::ImmInt(0),
                        elem_type: format!("[{} x double]", count),
                    });
                    function.body.push(Inst::Store(elem_ptr, first_val));
                    // Store remaining elements
                    for (i, elem) in elements.into_iter().skip(1).enumerate() {
                        let (val, _) = self.generate_expression_ir(elem, function);
                        let ep = Value::Reg(self.next_ptr);
                        self.next_ptr += 1;
                        function.body.push(Inst::GetElementPtr {
                            result: ep.clone(),
                            base: arr_ptr.clone(),
                            index: Value::ImmInt((i + 1) as i64),
                            elem_type: format!("[{} x double]", count),
                        });
                        function.body.push(Inst::Store(ep, val));
                    }
                    first_ty
                } else {
                    function.body.push(Inst::AllocaArray {
                        result: arr_ptr.clone(),
                        elem_type: "double".to_string(),
                        count: 0,
                    });
                    Ty::Int
                };
                (arr_ptr, Ty::Array(Box::new(elem_type), count))
            }
            Expression::ArrayRepeat { value, count } => {
                let (val, elem_ty) = self.generate_expression_ir(*value, function);
                if self.checked_mode {
                    let array_ty = Ty::Array(Box::new(elem_ty.clone()), count);
                    let copied_value = self.load_copy_aggregate_value(val, &elem_ty, function);
                    let array = self.allocate_fixed_copy_array_place(&array_ty, function);
                    for index in 0..count {
                        let element = self.fixed_copy_array_element_ptr(
                            array.clone(),
                            Value::ImmInt(index as i64),
                            &array_ty,
                            function,
                        );
                        function
                            .body
                            .push(Inst::Store(element, copied_value.clone()));
                    }
                    return (array, array_ty);
                }
                let arr_id = self.next_ptr;
                let arr_ptr = Value::Reg(arr_id);
                self.next_ptr += 1;
                function.body.push(Inst::AllocaArray {
                    result: arr_ptr.clone(),
                    elem_type: "double".to_string(),
                    count,
                });
                for i in 0..count {
                    let ep = Value::Reg(self.next_ptr);
                    self.next_ptr += 1;
                    function.body.push(Inst::GetElementPtr {
                        result: ep.clone(),
                        base: arr_ptr.clone(),
                        index: Value::ImmInt(i as i64),
                        elem_type: format!("[{} x double]", count),
                    });
                    function.body.push(Inst::Store(ep, val.clone()));
                }
                (arr_ptr, Ty::Array(Box::new(elem_ty), count))
            }
            Expression::IndexAccess { object, index } => {
                let (arr_val, arr_ty) = self.generate_expression_ir(*object, function);
                let (idx_val, _) = self.generate_expression_ir(*index, function);
                if self.checked_mode {
                    let Ty::Array(element_ty, _) = &arr_ty else {
                        unreachable!("checked array index receiver was admitted")
                    };
                    let element_ty = element_ty.as_ref().clone();
                    let element =
                        self.fixed_copy_array_element_ptr(arr_val, idx_val, &arr_ty, function);
                    let value = Value::Reg(self.next_reg);
                    self.next_reg += 1;
                    function.body.push(Inst::Load(value.clone(), element));
                    let value = self.store_copy_aggregate_value(value, &element_ty, function);
                    return (value, element_ty);
                }
                let (elem_ty, gep_elem_type) = match &arr_ty {
                    Ty::Array(et, len) => (*et.clone(), format!("[{} x double]", len)),
                    _ => (Ty::Int, "double".to_string()),
                };
                let elem_ptr = Value::Reg(self.next_ptr);
                self.next_ptr += 1;
                function.body.push(Inst::GetElementPtr {
                    result: elem_ptr.clone(),
                    base: arr_val,
                    index: idx_val,
                    elem_type: gep_elem_type,
                });
                let result = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function.body.push(Inst::Load(result.clone(), elem_ptr));
                (result, elem_ty)
            }
            Expression::Borrow { expr, mutable } if self.checked_mode => {
                if !matches!(expr.as_ref(), Expression::Identifier(_)) {
                    let array_selectors = projected_copydata_place_array_selectors(&expr)
                        .expect("checked projected call loan retained a valid place")
                        .expect("checked non-identifier call loan is projected")
                        .into_iter()
                        .cloned()
                        .collect::<Vec<_>>();
                    let use_context = if mutable {
                        ProjectedCopyDataPlaceUse::MutableCallLoan
                    } else {
                        ProjectedCopyDataPlaceUse::ImmutableCallLoan
                    };
                    let projected = classify_projected_copydata_place_after_admission(
                        &expr,
                        None,
                        true,
                        &self.struct_registry,
                        use_context,
                        |root| {
                            self.symbol_table.get(root).map(|(_, ty)| {
                                OwnedPlaceAssignmentTargetFacts {
                                    ty: ty.clone(),
                                    mutable: true,
                                    initialized: true,
                                    local: true,
                                    ownership: OwnershipState::Owned,
                                }
                            })
                        },
                    );
                    let ProjectedCopyDataPlaceDisposition::Supported(contract) = projected else {
                        unreachable!(
                            "checked projected call-loan admission escaped its shared classifier"
                        )
                    };
                    let (root, source) = self.generate_projected_copydata_place(
                        &contract,
                        &array_selectors,
                        function,
                    );
                    let result = Value::Reg(self.next_ptr);
                    self.next_ptr += 1;
                    function.body.push(Inst::CheckedProjectedBorrow {
                        result: result.clone(),
                        root: root.clone(),
                        source: source.clone(),
                        root_type: contract.root_logical_type.clone(),
                        pointee: contract.leaf_logical_type.clone(),
                        mutable,
                    });
                    let Value::Reg(reference_id) = result else {
                        unreachable!("checked projected borrow uses a place identifier")
                    };
                    self.projected_call_reference_sources.insert(
                        reference_id,
                        ProjectedCallReferenceSource {
                            root,
                            source,
                            root_type: contract.root_logical_type,
                            pointee: contract.leaf_logical_type,
                            mutable,
                        },
                    );
                    return (
                        Value::Reg(reference_id),
                        Ty::Reference(Box::new(contract.leaf_type), mutable),
                    );
                }
                let Expression::Identifier(name) = expr.as_ref() else {
                    unreachable!("checked direct reference admission retained an identifier")
                };
                let (source, pointee) = self
                    .symbol_table
                    .get(name)
                    .expect("checked borrowed binding exists")
                    .clone();
                let context = if mutable {
                    ReferencePointeeContext::Mutable
                } else {
                    ReferencePointeeContext::Immutable
                };
                let pointee_contract =
                    self.admitted_reference_pointee_logical_type(&pointee, context);
                let result = Value::Reg(self.next_ptr);
                self.next_ptr += 1;
                let instruction = if mutable {
                    Inst::CheckedMutableBorrow {
                        result: result.clone(),
                        source: source.clone(),
                        pointee: pointee_contract.clone(),
                    }
                } else {
                    Inst::CheckedImmutableBorrow {
                        result: result.clone(),
                        source: source.clone(),
                        pointee: pointee_contract.clone(),
                    }
                };
                function.body.push(instruction);
                if !mutable
                    && matches!(pointee, Ty::Enum(_))
                    && self.mutable_owned_enum_places.get(name) == Some(&source)
                {
                    let LogicalType::Enum { name, variants } = pointee_contract else {
                        unreachable!("checked immutable enum borrow retains an exact schema")
                    };
                    let Value::Reg(reference_id) = &result else {
                        unreachable!("checked immutable enum borrow uses a place identifier")
                    };
                    self.mutable_owner_immutable_enum_reference_sources
                        .insert(*reference_id, (source, EnumSchema { name, variants }));
                }
                (result, Ty::Reference(Box::new(pointee), mutable))
            }
            Expression::Deref(reference) if self.checked_mode => {
                let (place, reference_type) = self.generate_expression_ir(*reference, function);
                let Ty::Reference(pointee, _) = reference_type else {
                    unreachable!("checked dereference admission requires a reference")
                };
                let pointee = *pointee;
                if matches!(pointee, Ty::Struct(_) | Ty::Array(_, _) | Ty::Tuple(_)) {
                    let value = self.load_copy_aggregate_value(place, &pointee, function);
                    let copied = self.store_copy_aggregate_value(value, &pointee, function);
                    return (copied, pointee);
                }
                let result = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function.body.push(Inst::Load(result.clone(), place));
                (result, pointee)
            }
            Expression::EnumVariant {
                enum_name,
                variant,
                data,
            } if self.checked_mode => {
                let source_arity = data.as_ref().map(Vec::len);
                let mut payload = Vec::new();
                if let Some(fields) = data {
                    payload.reserve(fields.len());
                    for field in fields {
                        let (value, ty) = self.generate_expression_ir(field, function);
                        let value = self.load_copy_aggregate_value(value, &ty, function);
                        payload.push((value, ty));
                    }
                }
                let resolved = self
                    .enum_registry
                    .resolve_constructor(
                        &enum_name,
                        &variant,
                        source_arity,
                        EnumExecutionContext::AdmittedFunction,
                    )
                    .expect("checked enum constructor was admitted");
                let payload_types = payload.iter().map(|(_, ty)| ty.clone()).collect::<Vec<_>>();
                self.enum_registry
                    .validate_constructor_payload(
                        &resolved,
                        source_arity.map(|_| payload_types.as_slice()),
                    )
                    .expect("checked enum payload type was admitted");
                let result = Value::Reg(self.next_reg);
                self.next_reg += 1;
                if payload.len() <= 1 {
                    function.body.push(Inst::CheckedEnumVariant {
                        result: result.clone(),
                        schema: resolved.contract.schema.clone(),
                        variant_index: resolved.variant_index,
                        payload: payload.into_iter().next().map(|(value, _)| value),
                    });
                } else {
                    function.body.push(Inst::CheckedEnumVariantFields {
                        result: result.clone(),
                        schema: resolved.contract.schema.clone(),
                        variant_index: resolved.variant_index,
                        fields: payload.into_iter().map(|(value, _)| value).collect(),
                    });
                }
                (result, resolved.contract.ty())
            }
            Expression::Match { expr, arms } if self.checked_mode => {
                self.generate_enum_match_ir(*expr, arms, function)
            }
            Expression::FieldAccess { object, field } if self.checked_mode => {
                let (base, receiver) = self.generate_expression_ir(*object, function);
                let (contract, field_index, field_contract) = self
                    .struct_registry
                    .resolve_field(&receiver, &field, StructExecutionContext::AdmittedFunction)
                    .expect("checked struct field access was admitted");
                let field_ptr = Value::Reg(self.next_ptr);
                self.next_ptr += 1;
                function.body.push(Inst::CheckedStructFieldPtr {
                    result: field_ptr.clone(),
                    base,
                    struct_name: contract.name,
                    field_index: field_index as u32,
                    field_type: field_contract.logical_type(),
                });
                let result = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function.body.push(Inst::Load(result.clone(), field_ptr));
                let field_type = field_contract.ty();
                let result = self.store_copy_aggregate_value(result, &field_type, function);
                (result, field_type)
            }
            Expression::StructLiteral { name, fields } if self.checked_mode => {
                let resolved = self
                    .struct_registry
                    .resolve_construction(&name, &fields, StructExecutionContext::AdmittedFunction)
                    .expect("checked struct construction was admitted");
                let mut values = Vec::with_capacity(fields.len());
                for (source_index, (_, expression)) in fields.into_iter().enumerate() {
                    let declaration_index = resolved.source_to_declaration[source_index];
                    let expected = resolved.contract.fields[declaration_index].ty();
                    let (value, actual) = if matches!(
                        (&expression, &expected),
                        (Expression::ArrayLiteral(elements), Ty::Array(_, 0)) if elements.is_empty()
                    ) {
                        (
                            self.allocate_fixed_copy_array_place(&expected, function),
                            expected.clone(),
                        )
                    } else {
                        self.generate_expression_ir(expression, function)
                    };
                    values.push(self.load_copy_aggregate_value(value, &actual, function));
                }
                let base = Value::Reg(self.next_ptr);
                self.next_ptr += 1;
                function.body.push(Inst::CheckedStructAlloca {
                    result: base.clone(),
                    struct_name: resolved.contract.name.clone(),
                    field_types: resolved
                        .contract
                        .fields
                        .iter()
                        .map(crate::struct_contract::StructFieldContract::logical_type)
                        .collect(),
                });
                for (value, declaration_index) in
                    values.into_iter().zip(resolved.source_to_declaration)
                {
                    let declaration_field = &resolved.contract.fields[declaration_index];
                    let field_ptr = Value::Reg(self.next_ptr);
                    self.next_ptr += 1;
                    function.body.push(Inst::CheckedStructFieldPtr {
                        result: field_ptr.clone(),
                        base: base.clone(),
                        struct_name: resolved.contract.name.clone(),
                        field_index: declaration_index as u32,
                        field_type: declaration_field.logical_type(),
                    });
                    function.body.push(Inst::Store(field_ptr, value));
                }
                (base, Ty::Struct(resolved.contract.name))
            }
            Expression::TupleLiteral(elements) if self.checked_mode => {
                let mut values = Vec::with_capacity(elements.len());
                let mut element_types = Vec::with_capacity(elements.len());
                for element in elements {
                    let (value, ty) = self.generate_expression_ir(element, function);
                    values.push(self.load_copy_aggregate_value(value, &ty, function));
                    element_types.push(ty);
                }
                let contract = match classify_copy_tuple_elements(
                    &element_types,
                    &self.struct_registry,
                    TupleExecutionContext::AdmittedFunction,
                ) {
                    TupleContractDisposition::Supported(contract) => contract,
                    _ => unreachable!("checked tuple literal was admitted"),
                };
                let tuple_ty = contract.ty();
                let base = self.allocate_copy_tuple_place(&tuple_ty, function);
                for (index, value) in values.into_iter().enumerate() {
                    let field = self.copy_tuple_field_ptr(base.clone(), &contract, index, function);
                    function.body.push(Inst::Store(field, value));
                }
                (base, tuple_ty)
            }
            Expression::TupleIndex { object, index } if self.checked_mode => {
                let (base, receiver) = self.generate_expression_ir(*object, function);
                let projection = match classify_tuple_projection(
                    &receiver,
                    index,
                    &self.struct_registry,
                    TupleExecutionContext::AdmittedFunction,
                ) {
                    TupleContractDisposition::Supported(contract) => contract,
                    _ => unreachable!("checked tuple projection was admitted"),
                };
                let field =
                    self.copy_tuple_field_ptr(base, &projection.tuple, projection.index, function);
                let result = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function.body.push(Inst::Load(result.clone(), field));
                let element = projection.element;
                let result = self.store_copy_aggregate_value(result, &element, function);
                (result, element)
            }
            Expression::FieldAccess { .. }
            | Expression::TupleLiteral(_)
            | Expression::TupleIndex { .. }
            | Expression::StructLiteral { .. }
            | Expression::EnumVariant { .. }
            | Expression::Match { .. }
            | Expression::Borrow { .. }
            | Expression::Deref(_) => {
                // Stub: these will be implemented as remaining Phase 4/5 tasks progress
                (Value::ImmInt(0), Ty::Int)
            }
            Expression::Closure { .. } => Self::quarantine_closure_expression(),
        }
    }

    fn handle_type_promotion(
        &mut self,
        lhs_val: Value,
        lhs_type: Ty,
        rhs_val: Value,
        rhs_type: Ty,
        target_type: &Ty,
        function: &mut Function,
    ) -> (Value, Value) {
        let promoted_lhs = if needs_promotion(&lhs_type, target_type) {
            let promoted_reg = Value::Reg(self.next_reg);
            self.next_reg += 1;
            function
                .body
                .push(Inst::SIToFP(promoted_reg.clone(), lhs_val));
            promoted_reg
        } else {
            lhs_val
        };

        let promoted_rhs = if needs_promotion(&rhs_type, target_type) {
            let promoted_reg = Value::Reg(self.next_reg);
            self.next_reg += 1;
            function
                .body
                .push(Inst::SIToFP(promoted_reg.clone(), rhs_val));
            promoted_reg
        } else {
            rhs_val
        };

        (promoted_lhs, promoted_rhs)
    }

    fn try_constant_fold(
        &self,
        op: &str,
        lhs: &Value,
        rhs: &Value,
        result_type: &Ty,
    ) -> (Option<Value>, Option<Ty>) {
        match (lhs, rhs, result_type) {
            (Value::ImmInt(l), Value::ImmInt(r), Ty::Int) => {
                if !self.checked_mode {
                    let result = match op {
                        "+" => l + r,
                        "-" => l - r,
                        "*" => l * r,
                        "/" => l / r,
                        _ => return (None, None),
                    };
                    return (Some(Value::ImmInt(result)), Some(Ty::Int));
                }
                let result = match op {
                    "+" => l.checked_add(*r),
                    "-" => l.checked_sub(*r),
                    "*" => l.checked_mul(*r),
                    "/" => l.checked_div(*r),
                    _ => return (None, None),
                };
                match result.filter(|value| i32::try_from(*value).is_ok()) {
                    Some(result) => (Some(Value::ImmInt(result)), Some(Ty::Int)),
                    None => (None, None),
                }
            }
            (Value::ImmFloat(l), Value::ImmFloat(r), Ty::Float) => {
                let result = match op {
                    "+" => l + r,
                    "-" => l - r,
                    "*" => l * r,
                    "/" => l / r,
                    _ => return (None, None),
                };
                (Some(Value::ImmFloat(result)), Some(Ty::Float))
            }
            _ => (None, None),
        }
    }

    fn generate_function_definition_ir(
        &mut self,
        name: String,
        parameters: Vec<crate::ast::Parameter>,
        return_type: Option<Type>,
        body: crate::ast::Block,
        current_function: &mut Function,
    ) {
        // Save current state
        let saved_symbol_table = self.symbol_table.clone();
        let saved_mutable_reference_sources = self.mutable_reference_sources.clone();
        let saved_projected_call_reference_sources = self.projected_call_reference_sources.clone();
        let saved_mutable_owner_immutable_enum_reference_sources =
            self.mutable_owner_immutable_enum_reference_sources.clone();
        let saved_immutable_owned_enum_places = self.immutable_owned_enum_places.clone();
        let saved_mutable_owned_enum_places = self.mutable_owned_enum_places.clone();
        let saved_generated_byte_buffer_owners = self.generated_byte_buffer_owners.clone();
        let saved_next_reg = self.next_reg;
        let saved_next_ptr = self.next_ptr;

        // Reset for function generation
        self.symbol_table.clear();
        self.mutable_reference_sources.clear();
        self.projected_call_reference_sources.clear();
        self.mutable_owner_immutable_enum_reference_sources.clear();
        self.immutable_owned_enum_places.clear();
        self.mutable_owned_enum_places.clear();
        self.generated_byte_buffer_owners.clear();
        self.next_reg = 0;
        self.next_ptr = 0;

        let copy_contract = self.copy_function_contracts.get(&name).cloned();
        let enum_contract = self.enum_function_contracts.get(&name).cloned();
        let reference_contract = self.reference_function_contracts.get(&name).cloned();

        // Create parameter names and types for IR
        let eligible_contract = self.function_return_types.contains_key(&name);
        let param_names: Vec<(String, String)> = parameters
            .iter()
            .map(|p| {
                (
                    p.name.clone(),
                    if eligible_contract {
                        self.ast_type_to_ir_name(&p.param_type)
                    } else {
                        match &p.param_type {
                            Type::Named(name) => name.clone(),
                            Type::Array(_, _) => "array".to_string(),
                            Type::Tuple(_) => "tuple".to_string(),
                            Type::Reference(_, mutable) => {
                                if *mutable {
                                    "&mut".to_string()
                                } else {
                                    "&".to_string()
                                }
                            }
                            Type::Generic(name, _) => name.clone(),
                        }
                    },
                )
            })
            .collect();

        // Set up parameter variables in symbol table
        for (index, param) in parameters.iter().enumerate() {
            let param_type = if let Some(contract) = &reference_contract {
                contract.parameters[index].1.ty.clone()
            } else if let Some(contract) = &enum_contract {
                contract.parameters[index].1.ty.clone()
            } else {
                copy_contract.as_ref().map_or_else(
                    || self.ast_type_to_ty(&param.param_type),
                    |contract| contract.parameters[index].1.ty.clone(),
                )
            };
            let storage = if matches!(param_type, Ty::Enum(_)) {
                let result = Value::Reg(self.next_reg);
                self.next_reg += 1;
                result
            } else {
                let place = Value::Reg(self.next_ptr);
                self.next_ptr += 1;
                place
            };

            self.symbol_table
                .insert(param.name.clone(), (storage, param_type));
        }

        // Generate function body IR
        let mut function_ir = Function {
            name: name.clone(),
            body: Vec::new(),
            next_reg: 0,
            next_ptr: 0,
        };

        // Allocate parameters
        for (index, param) in parameters.iter().enumerate() {
            let (storage, parameter_ty) = self.symbol_table.get(&param.name).unwrap().clone();
            if let Some(contract) = &reference_contract
                && let LogicalType::ImmutableReference { pointee } =
                    &contract.parameters[index].1.logical_type
            {
                function_ir
                    .body
                    .push(Inst::CheckedImmutableReferenceParameter {
                        result: storage,
                        parameter: param.name.clone(),
                        pointee: pointee.as_ref().clone(),
                    });
                debug_assert!(matches!(parameter_ty, Ty::Reference(_, false)));
            } else if let Some(contract) = &reference_contract
                && let LogicalType::MutableReference { pointee } =
                    &contract.parameters[index].1.logical_type
            {
                function_ir
                    .body
                    .push(Inst::CheckedMutableReferenceParameter {
                        result: storage,
                        parameter: param.name.clone(),
                        pointee: pointee.as_ref().clone(),
                    });
                debug_assert!(matches!(parameter_ty, Ty::Reference(_, true)));
            } else if let Some(contract) = &enum_contract
                && let LogicalType::Enum {
                    name: enum_name,
                    variants,
                } = &contract.parameters[index].1.logical_type
            {
                function_ir.body.push(Inst::CheckedEnumParameter {
                    result: storage,
                    parameter: param.name.clone(),
                    schema: EnumSchema {
                        name: enum_name.clone(),
                        variants: variants.clone(),
                    },
                });
                debug_assert!(matches!(parameter_ty, Ty::Enum(_)));
            } else {
                function_ir
                    .body
                    .push(Inst::Alloca(storage, param.name.clone()));
            }
        }

        // Generate statements
        for stmt in body.statements {
            self.generate_statement_ir(stmt, &mut function_ir);
        }

        // Handle block expression (implicit return) or default return when needed.
        if let Some(expr) = body.expression {
            let (mut return_value, return_ty) = self.generate_expression_ir(expr, &mut function_ir);
            return_value =
                self.load_copy_aggregate_value(return_value, &return_ty, &mut function_ir);
            if self.checked_mode {
                self.end_all_active_mutable_owner_immutable_enum_references(&mut function_ir);
                self.emit_live_byte_buffer_drops(&mut function_ir);
            }
            function_ir.body.push(Inst::Return(return_value));
        } else if !function_ir
            .body
            .last()
            .is_some_and(Self::instruction_terminates_block)
        {
            // If no explicit return exists, emit a default scalar return.
            // `None` return type is lowered as `void` in codegen.
            if self.checked_mode {
                self.end_all_active_mutable_owner_immutable_enum_references(&mut function_ir);
                self.emit_live_byte_buffer_drops(&mut function_ir);
            }
            function_ir.body.push(Inst::Return(Value::ImmInt(0)));
        }

        // Create a schema-carrying checked definition only for an admitted Copy or
        // enum transport contract; legacy/raw definitions keep their old shape.
        let func_def = if let Some(contract) = reference_contract {
            Inst::CheckedFunctionDef {
                name: name.clone(),
                parameters: contract
                    .parameters
                    .into_iter()
                    .map(|(parameter, contract)| (parameter, contract.logical_type))
                    .collect(),
                result: contract.result.logical_type,
                body: function_ir.body.clone(),
            }
        } else if let Some(contract) = enum_contract {
            Inst::CheckedFunctionDef {
                name: name.clone(),
                parameters: contract
                    .parameters
                    .into_iter()
                    .map(|(parameter, contract)| (parameter, contract.logical_type))
                    .collect(),
                result: contract.result.logical_type,
                body: function_ir.body.clone(),
            }
        } else if let Some(contract) = copy_contract {
            Inst::CheckedFunctionDef {
                name: name.clone(),
                parameters: contract
                    .parameters
                    .into_iter()
                    .map(|(parameter, contract)| (parameter, contract.logical_type))
                    .collect(),
                result: contract.result.logical_type,
                body: function_ir.body.clone(),
            }
        } else {
            let ir_return_type = return_type.as_ref().map(|ty| self.ast_type_to_ir_name(ty));
            Inst::FunctionDef {
                name: name.clone(),
                parameters: param_names,
                return_type: ir_return_type,
                body: function_ir.body.clone(),
            }
        };

        // Add function definition to current function (main)
        current_function.body.push(func_def);

        // Create and store function in functions map
        let function = Function {
            name: name.clone(),
            body: vec![], // The actual body is in the FunctionDef instruction
            next_reg: self.next_reg,
            next_ptr: self.next_ptr,
        };
        self.functions.insert(name, function);

        // Restore state
        self.symbol_table = saved_symbol_table;
        self.mutable_reference_sources = saved_mutable_reference_sources;
        self.projected_call_reference_sources = saved_projected_call_reference_sources;
        self.mutable_owner_immutable_enum_reference_sources =
            saved_mutable_owner_immutable_enum_reference_sources;
        self.immutable_owned_enum_places = saved_immutable_owned_enum_places;
        self.mutable_owned_enum_places = saved_mutable_owned_enum_places;
        self.generated_byte_buffer_owners = saved_generated_byte_buffer_owners;
        self.next_reg = saved_next_reg;
        self.next_ptr = saved_next_ptr;
    }

    fn generate_statement_ir_for_function(
        &mut self,
        stmt: Statement,
        function_body: &mut Vec<Inst>,
    ) {
        match stmt {
            Statement::Let {
                name,
                mutable: _,
                type_annotation: _,
                value,
            } => {
                let (expr_value, expr_type) = if let Some(val) = value {
                    self.generate_expression_ir_for_function(val, function_body)
                } else {
                    (Value::ImmInt(0), Ty::Int)
                };

                if Self::stores_value_directly(&expr_type) {
                    self.symbol_table.insert(name, (expr_value, expr_type));
                } else {
                    // Allocate a stack slot for the variable
                    let ptr_reg = Value::Reg(self.next_ptr);
                    self.next_ptr += 1;
                    function_body.push(Inst::Alloca(ptr_reg.clone(), name.clone()));
                    self.symbol_table.insert(name, (ptr_reg.clone(), expr_type));

                    // Store the expression result into the allocated slot
                    function_body.push(Inst::Store(ptr_reg, expr_value));
                }
            }
            Statement::Return(expr) => {
                let (return_value, _) = if let Some(val) = expr {
                    self.generate_expression_ir_for_function(val, function_body)
                } else {
                    (Value::ImmInt(0), Ty::Int)
                };
                function_body.push(Inst::Return(return_value));
            }
            Statement::Function { .. } => {
                // Nested functions not supported yet
                println!("Warning: Nested function definitions are not supported");
            }
            _ => {
                // Other statements not implemented yet
                println!("Warning: Statement type not yet implemented in function body");
            }
        }
    }

    fn generate_expression_ir_for_function(
        &mut self,
        expr: Expression,
        function_body: &mut Vec<Inst>,
    ) -> (Value, Ty) {
        match expr {
            Expression::IntegerLiteral(n) => (Value::ImmInt(n), Ty::Int),
            Expression::FloatLiteral(f) => (Value::ImmFloat(f), Ty::Float),
            Expression::CharacterLiteral(character) => (Value::ImmChar(character), Ty::Char),
            Expression::Identifier(name) => {
                let (storage, var_type) = self
                    .symbol_table
                    .get(&name)
                    .expect("Undeclared variable")
                    .clone();
                if self.is_mutable_owned_enum_place(&name, &storage, &var_type) {
                    let result = Value::Reg(self.next_reg);
                    self.next_reg += 1;
                    function_body.push(Inst::Load(result.clone(), storage));
                    return (result, var_type);
                }
                if Self::stores_value_directly(&var_type) {
                    return (storage, var_type);
                }
                let result_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function_body.push(Inst::Load(result_reg.clone(), storage));
                (result_reg, var_type)
            }
            Expression::Binary {
                op,
                left,
                right,
                ty,
            } => {
                let (lhs_val, lhs_type) =
                    self.generate_expression_ir_for_function(*left, function_body);
                let (rhs_val, rhs_type) =
                    self.generate_expression_ir_for_function(*right, function_body);

                // Prefer semantic type annotation when present, but don't require it.
                // Some front-end paths still produce untyped binary nodes.
                let result_type = ty.unwrap_or_else(|| match (&lhs_type, &rhs_type) {
                    (Ty::Float, _) | (_, Ty::Float) => Ty::Float,
                    (Ty::Int, Ty::Int) => Ty::Int,
                    (l, r) => panic!(
                        "Cannot infer binary op result type for operand types {:?} and {:?}",
                        l, r
                    ),
                });

                // Handle type promotion if needed
                let (promoted_lhs, promoted_rhs) = self.handle_type_promotion_for_function(
                    lhs_val,
                    lhs_type,
                    rhs_val,
                    rhs_type,
                    &result_type,
                    function_body,
                );

                // Try constant folding first
                if let (Some(folded_value), Some(folded_type)) =
                    self.try_constant_fold(op.as_str(), &promoted_lhs, &promoted_rhs, &result_type)
                {
                    return (folded_value, folded_type);
                }

                let result_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;

                // Generate the appropriate instruction based on result type
                let inst = match (&result_type, op.as_str()) {
                    (Ty::Int, "+") => Inst::Add(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "+") => Inst::FAdd(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Int, "-") => Inst::Sub(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "-") => Inst::FSub(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Int, "*") => Inst::Mul(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "*") => Inst::FMul(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Int, "/") => Inst::Div(result_reg.clone(), promoted_lhs, promoted_rhs),
                    (Ty::Float, "/") => Inst::FDiv(result_reg.clone(), promoted_lhs, promoted_rhs),
                    _ => panic!(
                        "Unsupported binary operation: {} for type {:?}",
                        op, result_type
                    ),
                };

                function_body.push(inst);
                (result_reg, result_type)
            }
            Expression::FunctionCall { name, arguments } => {
                self.generate_function_call_ir(name, arguments, function_body)
            }
            Expression::Print {
                format_string,
                arguments,
            } => {
                self.generate_print_ir_for_function(format_string, arguments, false, function_body)
            }
            Expression::Println {
                format_string,
                arguments,
            } => self.generate_print_ir_for_function(format_string, arguments, true, function_body),
            Expression::Comparison { op, left, right } => {
                self.generate_comparison_ir_for_function(op, *left, *right, function_body)
            }
            Expression::Logical { op, left, right } => {
                self.generate_logical_ir_for_function(op, *left, *right, function_body)
            }
            Expression::Unary { op, operand } => {
                self.generate_unary_ir_for_function(op, *operand, function_body)
            }
            // Phase 4 stubs for function-level IR
            Expression::StringLiteral(s) => (Value::ImmString(s), Ty::String),
            Expression::MethodCall {
                object,
                method,
                arguments,
            } => {
                let (object_value, object_ty) =
                    self.generate_expression_ir_for_function(*object, function_body);
                if method == "iter"
                    && arguments.is_empty()
                    && matches!(object_ty, Ty::Array(_, _) | Ty::Vec(_))
                {
                    (object_value, object_ty)
                } else {
                    // Quarantined legacy function-level stub. Checked generation uses the
                    // shared intrinsic-method classifier in `generate_expression_ir`.
                    (Value::ImmInt(0), Ty::Int)
                }
            }
            Expression::ArrayLiteral(_)
            | Expression::ArrayRepeat { .. }
            | Expression::IndexAccess { .. }
            | Expression::FieldAccess { .. }
            | Expression::TupleLiteral(_)
            | Expression::TupleIndex { .. }
            | Expression::StructLiteral { .. }
            | Expression::EnumVariant { .. }
            | Expression::Match { .. }
            | Expression::Borrow { .. }
            | Expression::Deref(_) => (Value::ImmInt(0), Ty::Int),
            Expression::Closure { .. } => Self::quarantine_closure_expression(),
        }
    }

    fn handle_type_promotion_for_function(
        &mut self,
        lhs_val: Value,
        lhs_type: Ty,
        rhs_val: Value,
        rhs_type: Ty,
        target_type: &Ty,
        function_body: &mut Vec<Inst>,
    ) -> (Value, Value) {
        let promoted_lhs = if needs_promotion(&lhs_type, target_type) {
            let promoted_reg = Value::Reg(self.next_reg);
            self.next_reg += 1;
            function_body.push(Inst::SIToFP(promoted_reg.clone(), lhs_val));
            promoted_reg
        } else {
            lhs_val
        };

        let promoted_rhs = if needs_promotion(&rhs_type, target_type) {
            let promoted_reg = Value::Reg(self.next_reg);
            self.next_reg += 1;
            function_body.push(Inst::SIToFP(promoted_reg.clone(), rhs_val));
            promoted_reg
        } else {
            rhs_val
        };

        (promoted_lhs, promoted_rhs)
    }

    fn generate_function_call_ir(
        &mut self,
        name: String,
        arguments: Vec<Expression>,
        function_body: &mut Vec<Inst>,
    ) -> (Value, Ty) {
        // Generate IR for arguments
        let mut arg_values = Vec::new();
        for arg in arguments {
            let (arg_value, _) = self.generate_expression_ir_for_function(arg, function_body);
            arg_values.push(arg_value);
        }

        let (call_inst, result, return_type) = self.build_function_call(name, arg_values);
        function_body.push(call_inst);
        (result, return_type)
    }

    // Control flow IR generation methods
    fn generate_if_statement_ir(
        &mut self,
        condition: Expression,
        then_block: crate::ast::Block,
        else_block: Option<Box<Statement>>,
        current_function: &mut Function,
    ) {
        // Generate condition evaluation
        let (cond_value, _) = self.generate_expression_ir(condition, current_function);

        // Generate unique labels
        let then_label = format!("if_then_{}", self.next_reg);
        self.next_reg += 1;
        let else_label = format!("if_else_{}", self.next_reg);
        self.next_reg += 1;
        let end_label = format!("if_end_{}", self.next_reg);
        self.next_reg += 1;

        // Branch based on condition
        current_function.body.push(Inst::Branch {
            condition: cond_value,
            true_label: then_label.clone(),
            false_label: else_label.clone(),
        });

        let scope_snapshot = self.scope_snapshot();

        // Generate then block
        current_function.body.push(Inst::Label(then_label));
        for stmt in then_block.statements {
            self.generate_statement_ir(stmt, current_function);
        }
        if let Some(expr) = then_block.expression {
            self.generate_expression_ir(expr, current_function);
        }
        let then_terminates = current_function
            .body
            .last()
            .is_some_and(Self::instruction_terminates_block);
        if !then_terminates {
            if self.checked_mode {
                self.end_new_lexical_references(&scope_snapshot.bindings, current_function);
            }
            current_function.body.push(Inst::Jump(end_label.clone()));
        }
        self.restore_bindings(&scope_snapshot);

        // Generate else block
        current_function.body.push(Inst::Label(else_label));
        if let Some(else_stmt) = else_block {
            self.generate_statement_ir(*else_stmt, current_function);
        }
        let else_terminates = current_function
            .body
            .last()
            .is_some_and(Self::instruction_terminates_block);
        if !else_terminates {
            current_function.body.push(Inst::Jump(end_label.clone()));
        }
        self.restore_bindings(&scope_snapshot);

        // A merge block is only reachable when at least one arm falls through.
        if !then_terminates || !else_terminates {
            current_function.body.push(Inst::Label(end_label));
        }
    }

    fn generate_while_loop_ir(
        &mut self,
        condition: Expression,
        body: crate::ast::Block,
        current_function: &mut Function,
    ) {
        let labels = self.statement_loop_labels(StatementLoopKind::While);
        let loop_start = labels.header;
        let loop_body = labels.body.expect("while loop has a body label");
        let loop_end = labels.exit;

        // Push loop labels onto stack for break/continue
        self.loop_label_stack
            .push((labels.continue_target, loop_end.clone()));

        // Jump to loop start
        current_function.body.push(Inst::Jump(loop_start.clone()));

        // Loop start - evaluate condition
        current_function.body.push(Inst::Label(loop_start.clone()));
        let (cond_value, _) = self.generate_expression_ir(condition, current_function);
        current_function.body.push(Inst::Branch {
            condition: cond_value,
            true_label: loop_body.clone(),
            false_label: loop_end.clone(),
        });

        // Loop body
        current_function.body.push(Inst::Label(loop_body));
        for stmt in body.statements {
            self.generate_statement_ir(stmt, current_function);
            if current_function
                .body
                .last()
                .is_some_and(Self::instruction_terminates_block)
            {
                break;
            }
        }
        if let Some(expr) = body.expression {
            self.generate_expression_ir(expr, current_function);
        }
        if !current_function
            .body
            .last()
            .is_some_and(Self::instruction_terminates_block)
        {
            current_function.body.push(Inst::Jump(loop_start));
        }

        // Pop loop labels
        self.loop_label_stack.pop();

        // Loop end
        current_function.body.push(Inst::Label(loop_end));
    }

    fn generate_for_loop_ir(
        &mut self,
        variable: String,
        iterable: Expression,
        body: crate::ast::Block,
        current_function: &mut Function,
    ) {
        let (iter_value, iter_type) = self.generate_expression_ir(iterable, current_function);
        match iter_type {
            Ty::Array(elem_ty, len) => {
                self.generate_array_for_loop_ir(
                    variable,
                    iter_value,
                    *elem_ty,
                    len,
                    body,
                    current_function,
                );
            }
            other => {
                // Preserve the legacy numeric lowering behavior for non-array iterables.
                self.generate_legacy_for_loop_ir(
                    variable,
                    iter_value,
                    other,
                    body,
                    current_function,
                );
            }
        }
    }

    fn generate_array_for_loop_ir(
        &mut self,
        variable: String,
        array_ptr: Value,
        element_ty: Ty,
        array_len: usize,
        body: crate::ast::Block,
        current_function: &mut Function,
    ) {
        let labels = self.statement_loop_labels(StatementLoopKind::For);
        let loop_start = labels.header;
        let loop_body = labels.body.expect("for loop has a body label");
        let loop_continue = labels.continue_target;
        let loop_end = labels.exit;

        self.loop_label_stack
            .push((loop_continue.clone(), loop_end.clone()));

        // User-visible loop variable slot (updated each iteration with current element).
        let array_ty = Ty::Array(Box::new(element_ty.clone()), array_len);
        let copy_element = self
            .struct_registry
            .resolve_copy_type(&element_ty)
            .is_some();
        let loop_var_ptr = if matches!(element_ty, Ty::Struct(_) | Ty::Array(_, _) | Ty::Tuple(_)) {
            self.allocate_copy_aggregate_place(&element_ty, current_function)
        } else {
            let place = Value::Reg(self.next_ptr);
            self.next_ptr += 1;
            current_function
                .body
                .push(Inst::Alloca(place.clone(), variable.clone()));
            let initial_element = PrimitiveKind::from_ty(&element_ty)
                .map(PrimitiveKind::raw_zero_value)
                .unwrap_or(Value::ImmInt(0));
            current_function
                .body
                .push(Inst::Store(place.clone(), initial_element));
            place
        };
        self.symbol_table
            .insert(variable.clone(), (loop_var_ptr.clone(), element_ty.clone()));

        // Internal iteration index.
        let index_ptr = Value::Reg(self.next_ptr);
        self.next_ptr += 1;
        current_function.body.push(Inst::Alloca(
            index_ptr.clone(),
            format!("__for_idx_{}", variable),
        ));
        current_function
            .body
            .push(Inst::Store(index_ptr.clone(), Value::ImmInt(0)));

        current_function.body.push(Inst::Jump(loop_start.clone()));

        // Header: idx < len
        current_function.body.push(Inst::Label(loop_start.clone()));
        let index_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        current_function
            .body
            .push(Inst::Load(index_reg.clone(), index_ptr.clone()));

        let cond_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        current_function.body.push(Inst::ICmp {
            op: "slt".to_string(),
            result: cond_reg.clone(),
            left: index_reg.clone(),
            right: Value::ImmInt(array_len as i64),
        });
        current_function.body.push(Inst::Branch {
            condition: cond_reg,
            true_label: loop_body.clone(),
            false_label: loop_end.clone(),
        });

        // Body: load element at idx, assign loop variable, execute body, idx += 1.
        current_function.body.push(Inst::Label(loop_body));
        let body_start = current_function.body.len();
        let elem_ptr = if copy_element {
            self.fixed_copy_array_element_ptr(
                array_ptr.clone(),
                index_reg.clone(),
                &array_ty,
                current_function,
            )
        } else {
            let place = Value::Reg(self.next_ptr);
            self.next_ptr += 1;
            current_function.body.push(Inst::GetElementPtr {
                result: place.clone(),
                base: array_ptr.clone(),
                index: index_reg.clone(),
                elem_type: format!("[{} x double]", array_len),
            });
            place
        };
        let elem_val = Value::Reg(self.next_reg);
        self.next_reg += 1;
        current_function
            .body
            .push(Inst::Load(elem_val.clone(), elem_ptr));
        current_function
            .body
            .push(Inst::Store(loop_var_ptr, elem_val));

        for stmt in body.statements {
            self.generate_statement_ir(stmt, current_function);
            if current_function
                .body
                .last()
                .is_some_and(Self::instruction_terminates_block)
            {
                break;
            }
        }
        if let Some(expr) = body.expression {
            self.generate_expression_ir(expr, current_function);
        }

        self.generate_for_iteration_tail(
            body_start,
            loop_continue,
            loop_start,
            index_ptr,
            index_reg,
            current_function,
        );

        self.loop_label_stack.pop();
        current_function.body.push(Inst::Label(loop_end));
    }

    fn generate_legacy_for_loop_ir(
        &mut self,
        variable: String,
        start_value: Value,
        var_type: Ty,
        body: crate::ast::Block,
        current_function: &mut Function,
    ) {
        let labels = self.statement_loop_labels(StatementLoopKind::For);
        let loop_start = labels.header;
        let loop_body = labels.body.expect("for loop has a body label");
        let loop_continue = labels.continue_target;
        let loop_end = labels.exit;

        self.loop_label_stack
            .push((loop_continue.clone(), loop_end.clone()));

        let var_ptr = Value::Reg(self.next_ptr);
        self.next_ptr += 1;
        current_function
            .body
            .push(Inst::Alloca(var_ptr.clone(), variable.clone()));
        current_function
            .body
            .push(Inst::Store(var_ptr.clone(), start_value));
        self.symbol_table
            .insert(variable.clone(), (var_ptr.clone(), var_type));

        current_function.body.push(Inst::Jump(loop_start.clone()));

        current_function.body.push(Inst::Label(loop_start.clone()));
        let loop_var_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        current_function
            .body
            .push(Inst::Load(loop_var_reg.clone(), var_ptr.clone()));

        let cond_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;
        current_function.body.push(Inst::ICmp {
            op: "slt".to_string(),
            result: cond_reg.clone(),
            left: loop_var_reg.clone(),
            right: Value::ImmInt(10),
        });
        current_function.body.push(Inst::Branch {
            condition: cond_reg,
            true_label: loop_body.clone(),
            false_label: loop_end.clone(),
        });

        current_function.body.push(Inst::Label(loop_body));
        let body_start = current_function.body.len();
        for stmt in body.statements {
            self.generate_statement_ir(stmt, current_function);
            if current_function
                .body
                .last()
                .is_some_and(Self::instruction_terminates_block)
            {
                break;
            }
        }
        if let Some(expr) = body.expression {
            self.generate_expression_ir(expr, current_function);
        }

        self.generate_for_iteration_tail(
            body_start,
            loop_continue,
            loop_start,
            var_ptr,
            loop_var_reg,
            current_function,
        );

        self.loop_label_stack.pop();
        current_function.body.push(Inst::Label(loop_end));
    }

    fn generate_infinite_loop_ir(
        &mut self,
        body: crate::ast::Block,
        current_function: &mut Function,
    ) {
        let labels = self.statement_loop_labels(StatementLoopKind::Loop);
        let loop_start = labels.header;
        let loop_end = labels.exit;

        // Push loop labels onto stack for break/continue
        self.loop_label_stack
            .push((labels.continue_target, loop_end.clone()));

        // Jump to loop start
        current_function.body.push(Inst::Jump(loop_start.clone()));

        // Loop start
        current_function.body.push(Inst::Label(loop_start.clone()));

        // Loop body
        for stmt in body.statements {
            self.generate_statement_ir(stmt, current_function);
            if current_function
                .body
                .last()
                .is_some_and(Self::instruction_terminates_block)
            {
                break;
            }
        }
        if let Some(expr) = body.expression {
            self.generate_expression_ir(expr, current_function);
        }

        // Jump back to start (infinite loop)
        if !current_function
            .body
            .last()
            .is_some_and(Self::instruction_terminates_block)
        {
            current_function.body.push(Inst::Jump(loop_start));
        }

        // Pop loop labels
        self.loop_label_stack.pop();

        // Loop end (reachable via break)
        current_function.body.push(Inst::Label(loop_end));
    }

    fn generate_for_iteration_tail(
        &mut self,
        body_start: usize,
        continue_target: String,
        header: String,
        index_place: Value,
        current_index: Value,
        current_function: &mut Function,
    ) {
        let falls_through = !current_function
            .body
            .last()
            .is_some_and(Self::instruction_terminates_block);
        if falls_through {
            current_function
                .body
                .push(Inst::Jump(continue_target.clone()));
        }
        let has_explicit_continue = current_function.body[body_start..].iter().any(
            |instruction| matches!(instruction, Inst::Jump(target) if target == &continue_target),
        );
        if !falls_through && !has_explicit_continue {
            return;
        }

        current_function.body.push(Inst::Label(continue_target));
        let next_index = Value::Reg(self.next_reg);
        self.next_reg += 1;
        current_function.body.push(Inst::Add(
            next_index.clone(),
            current_index,
            Value::ImmInt(1),
        ));
        current_function
            .body
            .push(Inst::Store(index_place, next_index));
        current_function.body.push(Inst::Jump(header));
    }

    fn generate_break_ir(&mut self, current_function: &mut Function) {
        if let Some((_loop_start, loop_end)) = self.loop_label_stack.last() {
            let break_label = loop_end.clone();
            current_function.body.push(Inst::Jump(break_label));
        } else {
            panic!("Break statement outside of loop");
        }
    }

    fn generate_continue_ir(&mut self, current_function: &mut Function) {
        if let Some((loop_start, _loop_end)) = self.loop_label_stack.last() {
            let continue_label = loop_start.clone();
            current_function.body.push(Inst::Jump(continue_label));
        } else {
            panic!("Continue statement outside of loop");
        }
    }

    fn ast_type_to_ty(&self, ty: &Type) -> Ty {
        match ty {
            Type::Named(name) => PrimitiveKind::from_source_name(name)
                .map(PrimitiveKind::ty)
                .unwrap_or_else(|| {
                    if name == "String" {
                        Ty::String
                    } else {
                        Ty::Struct(name.clone())
                    }
                }),
            Type::Array(elem, size) => Ty::Array(Box::new(self.ast_type_to_ty(elem)), *size),
            Type::Tuple(types) => Ty::Tuple(types.iter().map(|t| self.ast_type_to_ty(t)).collect()),
            Type::Reference(inner, mutable) => {
                Ty::Reference(Box::new(self.ast_type_to_ty(inner)), *mutable)
            }
            Type::Generic(name, _) => Ty::TypeParam(name.clone()),
        }
    }

    fn ast_type_to_ir_name(&self, ty: &Type) -> String {
        match self.ast_type_to_ty(ty) {
            Ty::Int => "i32".to_string(),
            Ty::Float => "f64".to_string(),
            Ty::Bool => "bool".to_string(),
            Ty::Char => "char".to_string(),
            Ty::String => "String".to_string(),
            Ty::ByteBuffer => "ByteBuffer".to_string(),
            Ty::Void => "void".to_string(),
            Ty::Array(_, _) => "array".to_string(),
            Ty::Tuple(_) => "tuple".to_string(),
            Ty::Struct(name) => name,
            Ty::Enum(name) => name,
            Ty::Reference(_, mutable) => {
                if mutable {
                    "&mut".to_string()
                } else {
                    "&".to_string()
                }
            }
            Ty::TypeParam(name) => name,
            Ty::Option(_) => "Option".to_string(),
            Ty::Result(_, _) => "Result".to_string(),
            Ty::Vec(_) => "Vec".to_string(),
            Ty::HashMap(_, _) => "HashMap".to_string(),
            Ty::Fn(name) => name,
        }
    }

    fn resolve_callable_name(&self, name: &str) -> String {
        if let Some((_, Ty::Fn(target))) = self.symbol_table.get(name) {
            return target.clone();
        }
        name.to_string()
    }

    fn quarantine_closure_expression() -> (Value, Ty) {
        // Deprecated unchecked generation keeps the parsed node inert. It must not
        // synthesize a callable type, signature, environment, layout, or symbol.
        (Value::ImmInt(0), Ty::Void)
    }

    // I/O and enhanced expression IR generation methods
    fn generate_print_ir(
        &mut self,
        format_string: String,
        arguments: Vec<Expression>,
        newline: bool,
        function: &mut Function,
    ) -> (Value, Ty) {
        // Generate IR for arguments
        let mut arg_values = Vec::new();
        for arg in arguments {
            let (arg_value, _) = self.generate_expression_ir(arg, function);
            arg_values.push(arg_value);
        }

        // Modify format string to add newline if needed
        let final_format = if newline {
            format!("{}\n", format_string)
        } else {
            format_string
        };

        // Create print instruction
        let print_inst = Inst::Print {
            format_string: final_format,
            arguments: arg_values,
        };

        function.body.push(print_inst);

        // The placeholder is never a value: semantic and checked admission classify
        // print expressions as Void, including discarded exhaustive Match arms.
        (Value::ImmInt(0), Ty::Void)
    }

    fn generate_comparison_ir(
        &mut self,
        op: crate::ast::ComparisonOp,
        left: Expression,
        right: Expression,
        function: &mut Function,
    ) -> (Value, Ty) {
        let (left_val, left_type) = self.generate_expression_ir(left, function);
        let (right_val, right_type) = self.generate_expression_ir(right, function);

        if self.checked_mode {
            let left_static = match (&left_val, &left_type) {
                (Value::ImmString(value), Ty::String) => Some(value.as_str()),
                _ => None,
            };
            let right_static = match (&right_val, &right_type) {
                (Value::ImmString(value), Ty::String) => Some(value.as_str()),
                _ => None,
            };
            if let StaticStringEqualityDisposition::StaticBool(value) =
                classify_static_string_equality(left_static, &op, right_static)
            {
                let result_reg = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function.body.push(Inst::ICmp {
                    op: if value { "eq" } else { "ne" }.to_string(),
                    result: result_reg.clone(),
                    left: Value::ImmInt(0),
                    right: Value::ImmInt(0),
                });
                return (result_reg, Ty::Bool);
            }
        }

        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;

        // Convert ComparisonOp to string for IR instruction
        let op_str = match op {
            crate::ast::ComparisonOp::Equal => "eq",
            crate::ast::ComparisonOp::NotEqual => "ne",
            crate::ast::ComparisonOp::LessThan => "slt",
            crate::ast::ComparisonOp::GreaterThan => "sgt",
            crate::ast::ComparisonOp::LessEqual => "sle",
            crate::ast::ComparisonOp::GreaterEqual => "sge",
        };

        // Generate appropriate comparison instruction based on operand types
        let inst = match (&left_type, &right_type) {
            (Ty::Int, Ty::Int) => Inst::ICmp {
                op: op_str.to_string(),
                result: result_reg.clone(),
                left: left_val,
                right: right_val,
            },
            (Ty::Float, Ty::Float) => {
                // Convert integer comparison ops to float comparison ops
                let float_op = match op_str {
                    "eq" => "oeq",
                    "ne" => "one",
                    "slt" => "olt",
                    "sgt" => "ogt",
                    "sle" => "ole",
                    "sge" => "oge",
                    _ => op_str,
                };
                Inst::FCmp {
                    op: float_op.to_string(),
                    result: result_reg.clone(),
                    left: left_val,
                    right: right_val,
                }
            }
            (Ty::Int, Ty::Float) => {
                // Promote left operand to float
                let promoted_left = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function
                    .body
                    .push(Inst::SIToFP(promoted_left.clone(), left_val));

                let float_op = match op_str {
                    "eq" => "oeq",
                    "ne" => "one",
                    "slt" => "olt",
                    "sgt" => "ogt",
                    "sle" => "ole",
                    "sge" => "oge",
                    _ => op_str,
                };
                Inst::FCmp {
                    op: float_op.to_string(),
                    result: result_reg.clone(),
                    left: promoted_left,
                    right: right_val,
                }
            }
            (Ty::Float, Ty::Int) => {
                // Promote right operand to float
                let promoted_right = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function
                    .body
                    .push(Inst::SIToFP(promoted_right.clone(), right_val));

                let float_op = match op_str {
                    "eq" => "oeq",
                    "ne" => "one",
                    "slt" => "olt",
                    "sgt" => "ogt",
                    "sle" => "ole",
                    "sge" => "oge",
                    _ => op_str,
                };
                Inst::FCmp {
                    op: float_op.to_string(),
                    result: result_reg.clone(),
                    left: left_val,
                    right: promoted_right,
                }
            }
            (Ty::Bool, Ty::Bool) => Inst::ICmp {
                op: op_str.to_string(),
                result: result_reg.clone(),
                left: left_val,
                right: right_val,
            },
            (Ty::Char, Ty::Char) => Inst::ICmp {
                op: op_str.to_string(),
                result: result_reg.clone(),
                left: left_val,
                right: right_val,
            },
            _ => panic!(
                "Unsupported comparison between {:?} and {:?}",
                left_type, right_type
            ),
        };

        function.body.push(inst);
        (result_reg, Ty::Bool)
    }

    fn generate_logical_ir(
        &mut self,
        op: crate::ast::LogicalOp,
        left: Expression,
        right: Expression,
        function: &mut Function,
    ) -> (Value, Ty) {
        self.generate_logical_ir_iterative(op, left, right, function)
    }

    fn generate_logical_ir_iterative(
        &mut self,
        op: crate::ast::LogicalOp,
        left: Expression,
        right: Expression,
        function: &mut Function,
    ) -> (Value, Ty) {
        let mut pending = vec![
            LogicalLoweringTask::Combine(op),
            LogicalLoweringTask::Evaluate(right),
            LogicalLoweringTask::Evaluate(left),
        ];
        let mut values = Vec::new();

        while let Some(task) = pending.pop() {
            match task {
                LogicalLoweringTask::Evaluate(Expression::Logical { op, left, right }) => {
                    pending.push(LogicalLoweringTask::Combine(op));
                    pending.push(LogicalLoweringTask::Evaluate(*right));
                    pending.push(LogicalLoweringTask::Evaluate(*left));
                }
                LogicalLoweringTask::Evaluate(expression) => {
                    let (value, _) = self.generate_expression_ir(expression, function);
                    values.push(value);
                }
                LogicalLoweringTask::Combine(op) => {
                    let right = values
                        .pop()
                        .expect("logical lowering must produce a right operand");
                    let left = values
                        .pop()
                        .expect("logical lowering must produce a left operand");
                    let result = Value::Reg(self.next_reg);
                    self.next_reg += 1;
                    function.body.push(match op {
                        crate::ast::LogicalOp::And => Inst::And {
                            result: result.clone(),
                            left,
                            right,
                        },
                        crate::ast::LogicalOp::Or => Inst::Or {
                            result: result.clone(),
                            left,
                            right,
                        },
                    });
                    values.push(result);
                }
            }
        }

        let result = values
            .pop()
            .expect("logical lowering must produce one result");
        debug_assert!(values.is_empty(), "logical lowering left extra values");
        (result, Ty::Bool)
    }

    fn generate_unary_ir(
        &mut self,
        op: crate::ast::UnaryOp,
        operand: Expression,
        function: &mut Function,
    ) -> (Value, Ty) {
        if self.checked_mode
            && matches!(op, crate::ast::UnaryOp::Negate)
            && let Expression::IntegerLiteral(value) = &operand
        {
            return (
                Value::ImmInt(
                    value
                        .checked_neg()
                        .expect("checked admission validated the negated integer literal"),
                ),
                Ty::Int,
            );
        }
        let (operand_val, operand_type) = self.generate_expression_ir(operand, function);

        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;

        let (inst, result_type) = match op {
            crate::ast::UnaryOp::Not => (
                Inst::Not {
                    result: result_reg.clone(),
                    operand: operand_val,
                },
                Ty::Bool,
            ),
            crate::ast::UnaryOp::Negate => (
                Inst::Neg {
                    result: result_reg.clone(),
                    operand: operand_val,
                },
                operand_type,
            ),
        };

        function.body.push(inst);
        (result_reg, result_type)
    }

    // Function-level I/O and enhanced expression IR generation methods
    fn generate_print_ir_for_function(
        &mut self,
        format_string: String,
        arguments: Vec<Expression>,
        newline: bool,
        function_body: &mut Vec<Inst>,
    ) -> (Value, Ty) {
        // Generate IR for arguments
        let mut arg_values = Vec::new();
        for arg in arguments {
            let (arg_value, _) = self.generate_expression_ir_for_function(arg, function_body);
            arg_values.push(arg_value);
        }

        // Modify format string to add newline if needed
        let final_format = if newline {
            format!("{}\n", format_string)
        } else {
            format_string
        };

        // Create print instruction
        let print_inst = Inst::Print {
            format_string: final_format,
            arguments: arg_values,
        };

        function_body.push(print_inst);

        // Print operations return unit type (represented as 0 for now)
        (Value::ImmInt(0), Ty::Int)
    }

    fn generate_comparison_ir_for_function(
        &mut self,
        op: crate::ast::ComparisonOp,
        left: Expression,
        right: Expression,
        function_body: &mut Vec<Inst>,
    ) -> (Value, Ty) {
        let (left_val, left_type) = self.generate_expression_ir_for_function(left, function_body);
        let (right_val, right_type) =
            self.generate_expression_ir_for_function(right, function_body);

        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;

        // Convert ComparisonOp to string for IR instruction
        let op_str = match op {
            crate::ast::ComparisonOp::Equal => "eq",
            crate::ast::ComparisonOp::NotEqual => "ne",
            crate::ast::ComparisonOp::LessThan => "slt",
            crate::ast::ComparisonOp::GreaterThan => "sgt",
            crate::ast::ComparisonOp::LessEqual => "sle",
            crate::ast::ComparisonOp::GreaterEqual => "sge",
        };

        // Generate appropriate comparison instruction based on operand types
        let inst = match (&left_type, &right_type) {
            (Ty::Int, Ty::Int) => Inst::ICmp {
                op: op_str.to_string(),
                result: result_reg.clone(),
                left: left_val,
                right: right_val,
            },
            (Ty::Float, Ty::Float) => {
                // Convert integer comparison ops to float comparison ops
                let float_op = match op_str {
                    "eq" => "oeq",
                    "ne" => "one",
                    "slt" => "olt",
                    "sgt" => "ogt",
                    "sle" => "ole",
                    "sge" => "oge",
                    _ => op_str,
                };
                Inst::FCmp {
                    op: float_op.to_string(),
                    result: result_reg.clone(),
                    left: left_val,
                    right: right_val,
                }
            }
            (Ty::Int, Ty::Float) => {
                // Promote left operand to float
                let promoted_left = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function_body.push(Inst::SIToFP(promoted_left.clone(), left_val));

                let float_op = match op_str {
                    "eq" => "oeq",
                    "ne" => "one",
                    "slt" => "olt",
                    "sgt" => "ogt",
                    "sle" => "ole",
                    "sge" => "oge",
                    _ => op_str,
                };
                Inst::FCmp {
                    op: float_op.to_string(),
                    result: result_reg.clone(),
                    left: promoted_left,
                    right: right_val,
                }
            }
            (Ty::Float, Ty::Int) => {
                // Promote right operand to float
                let promoted_right = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function_body.push(Inst::SIToFP(promoted_right.clone(), right_val));

                let float_op = match op_str {
                    "eq" => "oeq",
                    "ne" => "one",
                    "slt" => "olt",
                    "sgt" => "ogt",
                    "sle" => "ole",
                    "sge" => "oge",
                    _ => op_str,
                };
                Inst::FCmp {
                    op: float_op.to_string(),
                    result: result_reg.clone(),
                    left: left_val,
                    right: promoted_right,
                }
            }
            (Ty::Bool, Ty::Bool) => Inst::ICmp {
                op: op_str.to_string(),
                result: result_reg.clone(),
                left: left_val,
                right: right_val,
            },
            _ => panic!(
                "Unsupported comparison between {:?} and {:?}",
                left_type, right_type
            ),
        };

        function_body.push(inst);
        (result_reg, Ty::Bool)
    }

    fn generate_logical_ir_for_function(
        &mut self,
        op: crate::ast::LogicalOp,
        left: Expression,
        right: Expression,
        function_body: &mut Vec<Inst>,
    ) -> (Value, Ty) {
        let (left_val, _) = self.generate_expression_ir_for_function(left, function_body);
        let (right_val, _) = self.generate_expression_ir_for_function(right, function_body);

        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;

        let inst = match op {
            crate::ast::LogicalOp::And => Inst::And {
                result: result_reg.clone(),
                left: left_val,
                right: right_val,
            },
            crate::ast::LogicalOp::Or => Inst::Or {
                result: result_reg.clone(),
                left: left_val,
                right: right_val,
            },
        };

        function_body.push(inst);
        (result_reg, Ty::Bool)
    }

    fn generate_unary_ir_for_function(
        &mut self,
        op: crate::ast::UnaryOp,
        operand: Expression,
        function_body: &mut Vec<Inst>,
    ) -> (Value, Ty) {
        if self.checked_mode
            && matches!(op, crate::ast::UnaryOp::Negate)
            && let Expression::IntegerLiteral(value) = &operand
        {
            return (
                Value::ImmInt(
                    value
                        .checked_neg()
                        .expect("checked admission validated the negated integer literal"),
                ),
                Ty::Int,
            );
        }
        let (operand_val, operand_type) =
            self.generate_expression_ir_for_function(operand, function_body);

        let result_reg = Value::Reg(self.next_reg);
        self.next_reg += 1;

        let (inst, result_type) = match op {
            crate::ast::UnaryOp::Not => (
                Inst::Not {
                    result: result_reg.clone(),
                    operand: operand_val,
                },
                Ty::Bool,
            ),
            crate::ast::UnaryOp::Negate => (
                Inst::Neg {
                    result: result_reg.clone(),
                    operand: operand_val,
                },
                operand_type,
            ),
        };

        function_body.push(inst);
        (result_reg, result_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{AstNode, BinaryOp, Block, Expression, Parameter, Statement, Type};
    use crate::errors::SourceLocation;
    use crate::types::Ty;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    #[test]
    fn generates_main_function() {
        let mut ir_gen = IrGenerator::new();
        let ir = ir_gen.generate_ir(vec![]);
        assert!(ir.contains_key("main"));
    }

    #[test]
    fn checked_direct_ast_rejects_unlowerable_nodes_without_unwinding() {
        let cases = vec![
            (
                "typed modulo",
                vec![AstNode::Statement(Statement::Let {
                    name: "remainder".to_string(),
                    mutable: false,
                    type_annotation: None,
                    value: Some(Expression::Binary {
                        op: BinaryOp::Modulo,
                        left: Box::new(Expression::IntegerLiteral(5)),
                        right: Box::new(Expression::IntegerLiteral(2)),
                        ty: Some(Ty::Int),
                    }),
                })],
            ),
            (
                "top-level expression",
                vec![AstNode::Expression(Expression::IntegerLiteral(1))],
            ),
            (
                "break outside loop",
                vec![AstNode::Statement(Statement::Break)],
            ),
            (
                "continue outside loop",
                vec![AstNode::Statement(Statement::Continue)],
            ),
        ];

        for (name, ast) in cases {
            let outcome =
                catch_unwind(AssertUnwindSafe(|| IrGenerator::new().try_generate_ir(ast)));
            let result = outcome.unwrap_or_else(|_| panic!("{name}: checked API unwound"));
            assert!(result.is_err(), "{name}: checked API published partial IR");
        }
    }

    #[test]
    fn let_with_integer_emits_alloca_and_store() {
        let mut ir_gen = IrGenerator::new();
        let ast = vec![AstNode::Statement(Statement::Let {
            name: "x".to_string(),
            mutable: false,
            type_annotation: None,
            value: Some(Expression::IntegerLiteral(1)),
        })];

        let ir = ir_gen.generate_ir(ast);
        let main = &ir["main"].body;

        assert!(
            main.iter()
                .any(|i| matches!(i, crate::ir::Inst::Alloca(_, n) if n == "x"))
        );
        assert!(
            main.iter()
                .any(|i| matches!(i, crate::ir::Inst::Store(_, crate::ir::Value::ImmInt(1))))
        );
    }

    #[test]
    fn binary_expression_requires_type_annotation_in_ast() {
        let mut ir_gen = IrGenerator::new();
        let expr = Expression::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expression::IntegerLiteral(1)),
            right: Box::new(Expression::IntegerLiteral(2)),
            ty: Some(Ty::Int),
        };

        let (val, ty) = ir_gen.generate_expression_ir(
            expr,
            &mut crate::ir::Function {
                name: "main".to_string(),
                body: vec![],
                next_reg: 0,
                next_ptr: 0,
            },
        );
        assert_eq!(ty, Ty::Int);
        // should be immediate foldable
        assert!(matches!(
            val,
            crate::ir::Value::ImmInt(3) | crate::ir::Value::Reg(_)
        ));
    }

    #[test]
    fn unchecked_closure_is_quarantined_without_a_function_symbol() {
        let mut ir_gen = IrGenerator::new();
        let ast = vec![AstNode::Statement(Statement::Let {
            name: "add".to_string(),
            mutable: false,
            type_annotation: None,
            value: Some(Expression::Closure {
                params: vec![
                    Parameter {
                        name: "x".to_string(),
                        param_type: Type::Named("i32".to_string()),
                    },
                    Parameter {
                        name: "y".to_string(),
                        param_type: Type::Named("i32".to_string()),
                    },
                ],
                body: Box::new(Expression::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expression::Identifier("x".to_string())),
                    right: Box::new(Expression::Identifier("y".to_string())),
                    ty: Some(Ty::Int),
                }),
                location: SourceLocation::new(1, 1),
            }),
        })];

        let ir = ir_gen.generate_ir(ast);
        let main = &ir["main"].body;

        assert!(main.iter().all(|inst| !matches!(
            inst,
            crate::ir::Inst::Call { function, .. } if function.starts_with("__closure_")
        )));
        assert!(
            ir.keys().all(|name| !name.starts_with("__closure_")),
            "unchecked closure lowering manufactured a closure symbol: {:?}",
            ir.keys().collect::<Vec<_>>()
        );
    }

    #[test]
    fn for_loop_over_array_emits_indexed_iteration() {
        let mut ir_gen = IrGenerator::new();
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "values".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::ArrayLiteral(vec![
                    Expression::IntegerLiteral(1),
                    Expression::IntegerLiteral(2),
                    Expression::IntegerLiteral(3),
                ])),
            }),
            AstNode::Statement(Statement::For {
                variable: "v".to_string(),
                iterable: Expression::Identifier("values".to_string()),
                body: Block {
                    statements: vec![],
                    expression: None,
                },
            }),
        ];

        let ir = ir_gen.generate_ir(ast);
        let main = &ir["main"].body;

        assert!(
            main.iter()
                .any(|inst| matches!(inst, crate::ir::Inst::Alloca(_, name) if name == "v"))
        );
        assert!(
            main.iter()
                .any(|inst| matches!(inst, crate::ir::Inst::GetElementPtr { .. }))
        );
        assert!(
            main.iter()
                .any(|inst| matches!(inst, crate::ir::Inst::ICmp { op, .. } if op == "slt"))
        );
    }

    #[test]
    fn print_argument_keeps_string_immediate() {
        let mut ir_gen = IrGenerator::new();
        let ast = vec![
            AstNode::Statement(Statement::Let {
                name: "name".to_string(),
                mutable: false,
                type_annotation: None,
                value: Some(Expression::StringLiteral("Aero".to_string())),
            }),
            AstNode::Statement(Statement::Expression(Expression::Println {
                format_string: "{}".to_string(),
                arguments: vec![Expression::Identifier("name".to_string())],
            })),
        ];

        let ir = ir_gen.generate_ir(ast);
        let main = &ir["main"].body;

        assert!(main.iter().any(|inst| {
            matches!(
                inst,
                crate::ir::Inst::Print { arguments, .. }
                    if arguments.iter().any(|arg| matches!(arg, crate::ir::Value::ImmString(s) if s == "Aero"))
            )
        }));
    }
}
