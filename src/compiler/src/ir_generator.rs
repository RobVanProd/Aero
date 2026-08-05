use crate::ast::{AstNode, Expression, Statement, Type};
use crate::binding_annotation::{
    BindingAnnotationDisposition, BindingContractKind, classify_binding_annotation,
    is_statically_empty_fixed_array, typed_empty_numeric_array_contract,
};
use crate::fixed_array_method::{FixedArrayLenDisposition, classify_fixed_array_len};
use crate::ir::{Function, Inst, LogicalType, PlaceId, Value};
use crate::ir_verifier::PlaceTypeHints;
use crate::static_string_equality::{
    StaticStringEqualityDisposition, classify_static_string_equality,
};
use crate::static_string_method::{StaticStringLenDisposition, classify_static_string_len};
use crate::static_string_predicate::{
    StaticStringPredicateDisposition, classify_static_string_predicate,
};
use crate::struct_contract::{
    CopyFunctionContract, CopyStructArrayIndexDisposition, StructContractError,
    StructExecutionContext, StructRegistry,
};
use crate::types::{Ty, needs_promotion};
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
    callable: bool,
    static_string: Option<String>,
}

struct AdmissionTopLevelFunction {
    result: Ty,
    arity: Option<usize>,
    parameter_types: Option<Vec<Ty>>,
}

struct AdmissionProgram {
    functions: HashMap<String, AdmissionTopLevelFunction>,
    structs: StructRegistry,
}

const STRUCT_ADMISSION_BINDING: &str = "\0aero.checked.struct.context";

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExpressionUse {
    Value,
    Binding,
    PrintArgument,
    Discarded,
}

pub struct IrGenerator {
    functions: HashMap<String, Function>,
    #[allow(dead_code)]
    current_function_name: String,
    next_reg: u32,
    next_ptr: u32,
    symbol_table: HashMap<String, (Value, Ty)>, // Track both pointer and type
    function_return_types: HashMap<String, Ty>,
    copy_function_contracts: HashMap<String, CopyFunctionContract>,
    loop_label_stack: Vec<(String, String)>, // Stack of (loop_start, loop_end) labels
    closure_count: u32,                      // Counter for unique closure names
    checked_mode: bool,
    checked_place_hints: PlaceTypeHints,
    struct_registry: StructRegistry,
}

impl IrGenerator {
    pub fn new() -> Self {
        IrGenerator {
            functions: HashMap::new(),
            current_function_name: String::new(),
            next_reg: 0,
            next_ptr: 0,
            symbol_table: HashMap::new(),
            function_return_types: HashMap::new(),
            copy_function_contracts: HashMap::new(),
            loop_label_stack: Vec::new(),
            closure_count: 0,
            checked_mode: false,
            checked_place_hints: BTreeMap::new(),
            struct_registry: StructRegistry::default(),
        }
    }
}

impl Default for IrGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl IrGenerator {
    pub fn try_generate_ir(
        &mut self,
        ast: Vec<AstNode>,
    ) -> Result<crate::ir::CheckedIr, IrGenerationError> {
        Self::validate_checked_ast(&ast)?;
        self.struct_registry = StructRegistry::from_top_level_ast(&ast);
        self.functions.clear();
        self.current_function_name.clear();
        self.next_reg = 0;
        self.next_ptr = 0;
        self.symbol_table.clear();
        self.function_return_types.clear();
        self.copy_function_contracts.clear();
        self.loop_label_stack.clear();
        self.closure_count = 0;
        self.checked_place_hints.clear();
        self.checked_mode = true;
        let mut functions = self.generate_ir(ast);
        self.checked_mode = false;
        Self::ensure_checked_main_terminator(&mut functions);
        Self::normalize_checked_place_ids(&mut functions, &mut self.checked_place_hints);
        crate::ir_verifier::verify_ir_with_place_hints(functions, &self.checked_place_hints)
            .map_err(Into::into)
    }

    pub fn generate_ir(&mut self, ast: Vec<AstNode>) -> HashMap<String, Function> {
        self.function_return_types.clear();
        self.copy_function_contracts.clear();
        for node in &ast {
            if let AstNode::Statement(Statement::Function {
                name,
                parameters,
                return_type,
                type_params,
                ..
            }) = node
                && type_params.is_empty()
            {
                let copy_contract = if self.checked_mode {
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
                if let Some(contract) = copy_contract {
                    self.function_return_types
                        .insert(name.clone(), contract.result.ty.clone());
                    self.copy_function_contracts.insert(name.clone(), contract);
                } else if parameters
                    .iter()
                    .all(|parameter| Self::numeric_contract_type(&parameter.param_type).is_some())
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
            Type::Named(name) if matches!(name.as_str(), "i32" | "int") => Some(Ty::Int),
            Type::Named(name) if matches!(name.as_str(), "f64" | "float") => Some(Ty::Float),
            Type::Named(name) if name == "bool" => Some(Ty::Bool),
            _ => None,
        }
    }

    fn build_function_call(&mut self, name: String, arguments: Vec<Value>) -> (Inst, Value, Ty) {
        let function_name = self.resolve_callable_name(&name);
        let return_type = self.function_return_types.get(&function_name).cloned();

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
                return_type @ (Ty::Int | Ty::Float | Ty::Bool | Ty::Struct(_) | Ty::Array(_, _)),
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
            Ty::String | Ty::Array(_, _) | Ty::Struct(_) | Ty::Vec(_) | Ty::Fn(_)
        )
    }

    fn validate_checked_ast(ast: &[AstNode]) -> Result<(), IrGenerationError> {
        let mut program: HashMap<String, AdmissionTopLevelFunction> = HashMap::new();
        let structs = StructRegistry::from_top_level_ast(ast);
        for node in ast {
            if let AstNode::Statement(Statement::Function {
                name,
                parameters,
                return_type,
                type_params,
                ..
            }) = node
            {
                let copy_contract = if type_params.is_empty() {
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
                let (result, arity, parameter_types) = if let Some(contract) = copy_contract {
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
            structs,
        };
        let mut bindings = HashMap::new();
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
            || !Self::admitted_symbol(name)
            || !type_params.is_empty()
            || return_type.is_some_and(|return_type| {
                !matches!(
                    Self::admission_type(return_type),
                    Ty::Int | Ty::Float | Ty::Bool
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
                    Ty::Int | Ty::Float | Ty::Bool
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
    ) -> Result<(), IrGenerationError> {
        match statement {
            Statement::Let {
                name,
                mutable,
                type_annotation,
                value,
            } => {
                let disposition = type_annotation.as_ref().map_or(
                    BindingAnnotationDisposition::PreserveExistingBehavior,
                    |annotation| classify_binding_annotation(annotation, value.is_some()),
                );
                if value.is_none()
                    && let BindingAnnotationDisposition::ExistingExplicitRejection(kind) =
                        disposition
                {
                    return Err(IrGenerationError::Admission(format!(
                        "checked IR binding `{}` uses an unsupported {} for an uninitialized binding",
                        name,
                        kind.topology()
                    )));
                }
                if let Some(value) = value {
                    let static_string = if type_annotation.is_none()
                        || matches!(
                            disposition,
                            BindingAnnotationDisposition::MatchesExistingContractShape(
                                BindingContractKind::String
                            )
                        ) {
                        Self::static_string_value(value, bindings)
                    } else {
                        None
                    };
                    let ty = if inside_impl || inside_generic_impl {
                        Self::validate_expression(
                            value,
                            bindings,
                            program,
                            ExpressionUse::Binding,
                            inside_impl,
                            !inside_impl,
                        )?
                    } else if let Some(contract) = type_annotation.as_ref().and_then(|annotation| {
                        program
                            .structs
                            .typed_empty_copy_struct_array_contract(annotation, value)
                    }) {
                        contract.ty()
                    } else if let Some(contract) = type_annotation.as_ref().and_then(|annotation| {
                        typed_empty_numeric_array_contract(annotation, value)
                    }) {
                        contract.ty()
                    } else {
                        Self::validate_expression(
                            value,
                            bindings,
                            program,
                            ExpressionUse::Binding,
                            inside_impl,
                            !inside_impl,
                        )?
                    };
                    if matches!(ty, Ty::Void) {
                        return Err(IrGenerationError::Admission(
                            "Void expressions cannot be stored in a binding".to_string(),
                        ));
                    }
                    program
                        .structs
                        .validate_copy_struct_array_binding(type_annotation.as_ref(), &ty)
                        .map_err(|error| IrGenerationError::Admission(error.diagnostic()))?;
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
                    if let BindingAnnotationDisposition::ExistingExplicitRejection(kind) =
                        disposition
                    {
                        return Err(IrGenerationError::Admission(format!(
                            "checked IR binding `{}` uses an unsupported {} for an initialized binding",
                            name,
                            kind.topology()
                        )));
                    }
                    if !inside_generic_impl
                        && let BindingAnnotationDisposition::MatchesExistingContractShape(contract) =
                            disposition
                    {
                        let expected = contract.ty();
                        if ty != expected {
                            return Err(IrGenerationError::Admission(format!(
                                "checked IR binding `{}` type annotation mismatch: expected {}, actual {}",
                                name, expected, ty
                            )));
                        }
                    }
                    if *mutable && matches!(ty, Ty::String) {
                        return Err(IrGenerationError::Admission(
                            "mutable string bindings are not admitted; checked strings are immutable compile-time aliases"
                                .to_string(),
                        ));
                    }
                    bindings.insert(
                        name.clone(),
                        AdmissionBinding {
                            callable: matches!(ty, Ty::Fn(_)),
                            static_string,
                            ty,
                        },
                    );
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
                )?;
            }
            Statement::Loop { body } => {
                let mut nested = bindings.clone();
                Self::validate_block(
                    body,
                    &mut nested,
                    program,
                    true,
                    inside_impl,
                    inside_generic_impl,
                )?;
            }
            Statement::Function {
                name,
                parameters,
                return_type,
                type_params,
                body,
                ..
            } => {
                if !type_params.is_empty() {
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
                            callable: false,
                            static_string: None,
                        },
                    );
                }
                let copy_contract = match program.structs.resolve_copy_function_contract(
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
                };
                for (index, parameter) in parameters.iter().enumerate() {
                    let parameter_ty = copy_contract.as_ref().map_or_else(
                        || Self::admission_type(&parameter.param_type),
                        |contract| contract.parameters[index].1.ty.clone(),
                    );
                    if copy_contract.is_none()
                        && !matches!(parameter_ty, Ty::Int | Ty::Float | Ty::Bool)
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
                            callable: false,
                            static_string: None,
                        },
                    );
                }
                if copy_contract.is_none()
                    && return_type.as_ref().is_some_and(|return_type| {
                        !matches!(
                            Self::admission_type(return_type),
                            Ty::Int | Ty::Float | Ty::Bool
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
                let mut then_bindings = bindings.clone();
                Self::validate_block(
                    then_block,
                    &mut then_bindings,
                    program,
                    inside_loop,
                    inside_impl,
                    inside_generic_impl,
                )?;
                if let Some(else_statement) = else_block {
                    let mut else_bindings = bindings.clone();
                    Self::validate_statement(
                        else_statement,
                        &mut else_bindings,
                        program,
                        inside_loop,
                        inside_impl,
                        inside_generic_impl,
                        false,
                    )?;
                }
            }
            Statement::While { condition, body } => {
                Self::validate_expression(
                    condition,
                    bindings,
                    program,
                    ExpressionUse::Value,
                    inside_impl,
                    !inside_impl,
                )?;
                let mut nested = bindings.clone();
                Self::validate_block(
                    body,
                    &mut nested,
                    program,
                    true,
                    inside_impl,
                    inside_generic_impl,
                )?;
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
                let mut nested = bindings.clone();
                let element_ty = match iterable_ty {
                    Ty::Array(element, _) | Ty::Vec(element) => *element,
                    _ => {
                        return Err(IrGenerationError::Admission(
                            "for-loop iteration requires an admitted array/Vec".to_string(),
                        ));
                    }
                };
                nested.insert(
                    variable.clone(),
                    AdmissionBinding {
                        ty: element_ty,
                        callable: false,
                        static_string: None,
                    },
                );
                Self::validate_block(
                    body,
                    &mut nested,
                    program,
                    true,
                    inside_impl,
                    inside_generic_impl,
                )?;
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
                    )?;
                }
            }
            // Trait declarations remain syntax-only in this prototype. The semantic
            // pass recursively diagnoses unsupported expressions in default bodies;
            // checked lowering must not activate runtime name binding for them.
            Statement::TraitDef { .. } => {}
            Statement::Break | Statement::Continue => {
                if !inside_loop {
                    return Err(IrGenerationError::Admission(
                        "break and continue are only admitted inside loops".to_string(),
                    ));
                }
            }
            Statement::StructDef { .. }
            | Statement::EnumDef { .. }
            | Statement::ModDecl { .. }
            | Statement::UseImport { .. } => {}
        }
        Ok(())
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
                let mut argument_types = Vec::with_capacity(arguments.len());
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
                if let Some(binding) = bindings.get(name) {
                    if binding.callable {
                        return match &binding.ty {
                            Ty::Fn(signature) => Self::callable_result_type(signature),
                            _ => Err(admission_error("callable binding lost its signature")),
                        };
                    }
                }
                if let Some(function) = program.functions.get(name) {
                    if function.result == Ty::Void && expression_use != ExpressionUse::Discarded {
                        return Err(admission_error(
                            "Void function calls cannot be used as values",
                        ));
                    }
                    if let Some(expected) = function.arity
                        && arguments.len() != expected
                    {
                        return Err(admission_error(&format!(
                            "call to `{name}` has {} arguments but its signature requires {expected}",
                            arguments.len()
                        )));
                    }
                    if let Some(expected_types) = &function.parameter_types {
                        for (index, (expected, actual)) in
                            expected_types.iter().zip(&argument_types).enumerate()
                        {
                            if expected != actual {
                                return Err(admission_error(&format!(
                                    "call to `{name}` argument {} type mismatch: expected {}, actual {}",
                                    index + 1,
                                    expected,
                                    actual
                                )));
                            }
                        }
                    }
                    return Ok(function.result.clone());
                }
                if !program.functions.contains_key(name) && matches!(name.as_str(), "Some" | "Ok") {
                    return Err(admission_error(
                        "enum and Option/Result construction is not admitted in checked IR",
                    ));
                }
                Ok(Ty::Int)
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
                if !inside_impl {
                    match classify_fixed_array_len(
                        &object_ty,
                        method,
                        arguments.len(),
                        &program.structs,
                    ) {
                        FixedArrayLenDisposition::StaticLength { .. } => return Ok(Ty::Int),
                        FixedArrayLenDisposition::WrongArity { kind, actual } => {
                            return Err(admission_error(&format!(
                                "{} .len() expects exactly 0 arguments, got {actual}",
                                kind.diagnostic_subject()
                            )));
                        }
                        FixedArrayLenDisposition::LengthOutsideIntRange { kind, count } => {
                            return Err(admission_error(&format!(
                                "{} .len() count {count} is outside the admitted i32 range",
                                kind.diagnostic_subject()
                            )));
                        }
                        FixedArrayLenDisposition::PreserveExistingBehavior => {}
                    }
                    let static_string = Self::static_string_value(object, bindings);
                    match classify_static_string_len(
                        static_string.as_deref(),
                        method,
                        arguments.len(),
                    ) {
                        StaticStringLenDisposition::StaticLength(_) => return Ok(Ty::Int),
                        StaticStringLenDisposition::WrongArity { actual } => {
                            return Err(admission_error(&format!(
                                "compile-time string .len() expects exactly 0 arguments, got {actual}"
                            )));
                        }
                        StaticStringLenDisposition::LengthOutsideIntRange { count } => {
                            return Err(admission_error(&format!(
                                "compile-time string .len() count {count} is outside the admitted i32 range"
                            )));
                        }
                        StaticStringLenDisposition::PreserveExistingBehavior => {}
                    }
                    let static_arguments = arguments
                        .iter()
                        .map(|argument| Self::static_string_value(argument, bindings))
                        .collect::<Vec<_>>();
                    let static_argument_refs = static_arguments
                        .iter()
                        .map(|argument| argument.as_deref())
                        .collect::<Vec<_>>();
                    match classify_static_string_predicate(
                        static_string.as_deref(),
                        method,
                        &static_argument_refs,
                    ) {
                        StaticStringPredicateDisposition::StaticBool(_) => return Ok(Ty::Bool),
                        StaticStringPredicateDisposition::WrongArity {
                            method,
                            expected,
                            actual,
                        } => {
                            let argument_label = if expected == 1 {
                                "argument"
                            } else {
                                "arguments"
                            };
                            return Err(admission_error(&format!(
                                "compile-time string .{method}() expects exactly {expected} {argument_label}, got {actual}"
                            )));
                        }
                        StaticStringPredicateDisposition::PreserveExistingBehavior => {}
                    }
                }
                if method == "iter"
                    && arguments.is_empty()
                    && matches!(object_ty, Ty::Array(_, _) | Ty::Vec(_))
                {
                    Ok(object_ty)
                } else {
                    Err(admission_error(
                        "method calls other than exact zero-argument array/Vec .iter() are not admitted",
                    ))
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
                if expression_use != ExpressionUse::Discarded {
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
                if !matches!(
                    (&left_ty, &right_ty),
                    (&Ty::Int, &Ty::Int)
                        | (&Ty::Float, &Ty::Float)
                        | (&Ty::Int, &Ty::Float)
                        | (&Ty::Float, &Ty::Int)
                        | (&Ty::Bool, &Ty::Bool)
                ) {
                    return Err(admission_error(
                        "comparison operand types are not admitted; expected numeric operands or Bool with Bool",
                    ));
                }
                Ok(Ty::Bool)
            }
            Expression::Logical { left, right, .. } => {
                Self::validate_expression(
                    left,
                    bindings,
                    program,
                    ExpressionUse::Value,
                    inside_impl,
                    admit_static_string_equality,
                )?;
                Self::validate_expression(
                    right,
                    bindings,
                    program,
                    ExpressionUse::Value,
                    inside_impl,
                    admit_static_string_equality,
                )?;
                Ok(Ty::Bool)
            }
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
                        if !matches!(current, Ty::Int | Ty::Float)
                            && !program.structs.is_copy_struct_ty(&current)
                        {
                            return Err(admission_error("only fixed numeric arrays are admitted"));
                        }
                        element_ty = current;
                    } else if !matches!(current, Ty::Int | Ty::Float)
                        && !program.structs.is_copy_struct_ty(&current)
                    {
                        return Err(admission_error("only fixed numeric arrays are admitted"));
                    } else if !inside_impl {
                        remaining_types.push(current);
                    }
                }
                if !inside_impl {
                    if program.structs.is_copy_struct_ty(&element_ty) {
                        program
                            .structs
                            .validate_copy_struct_array_elements(
                                &element_ty,
                                remaining_types.into_iter(),
                            )
                            .map_err(|error| IrGenerationError::Admission(error.diagnostic()))?;
                    } else if remaining_types.iter().any(|current| current != &element_ty) {
                        return Err(admission_error(
                            "fixed numeric arrays must have homogeneous element types in checked IR",
                        ));
                    }
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
                if !matches!(element_ty, Ty::Int | Ty::Float)
                    && !program.structs.is_copy_struct_ty(&element_ty)
                {
                    return Err(admission_error("only fixed numeric arrays are admitted"));
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
                match program
                    .structs
                    .classify_copy_struct_array_index(&object_ty, index)
                {
                    CopyStructArrayIndexDisposition::PreserveExistingBehavior
                    | CopyStructArrayIndexDisposition::Accepted { .. } => {}
                    CopyStructArrayIndexDisposition::NonConstant => {
                        return Err(admission_error(
                            "fixed Copy-struct array index must be a compile-time integer constant",
                        ));
                    }
                    CopyStructArrayIndexDisposition::OutOfBounds { index, count } => {
                        return Err(admission_error(&format!(
                            "fixed Copy-struct array index {index} is outside 0..{count}"
                        )));
                    }
                }
                match object_ty {
                    Ty::Array(element, _) => Ok(*element),
                    _ => Err(admission_error("indexing requires an admitted fixed array")),
                }
            }
            Expression::Closure { params, body } => {
                if expression_use != ExpressionUse::Binding {
                    return Err(admission_error(
                        "closures are admitted only as compile-time callable bindings",
                    ));
                }
                if Self::closure_body_needs_aggregate_lowering(body) {
                    return Err(admission_error(
                        "array construction, iteration, and indexing are not admitted in closure bodies",
                    ));
                }
                let mut closure_bindings = HashMap::new();
                let mut parameter_types = Vec::new();
                for parameter in params {
                    let parameter_ty = Self::admission_type(&parameter.param_type);
                    if !matches!(parameter_ty, Ty::Int | Ty::Float | Ty::Bool) {
                        return Err(admission_error(
                            "closure parameters must be admitted scalar types",
                        ));
                    }
                    parameter_types.push(parameter_ty.clone());
                    closure_bindings.insert(
                        parameter.name.clone(),
                        AdmissionBinding {
                            ty: parameter_ty,
                            callable: false,
                            static_string: None,
                        },
                    );
                }
                let result_ty = Self::validate_expression(
                    body,
                    &closure_bindings,
                    program,
                    ExpressionUse::Value,
                    inside_impl,
                    false,
                )?;
                if !matches!(result_ty, Ty::Int | Ty::Float | Ty::Bool) {
                    return Err(admission_error(
                        "closure results must be admitted scalar types",
                    ));
                }
                Ok(Ty::Fn(Self::encode_callable_signature(
                    &parameter_types,
                    &result_ty,
                )))
            }
            Expression::EnumVariant { data, .. } => {
                if let Some(data) = data {
                    let _ = Self::validate_expression(
                        data,
                        bindings,
                        program,
                        ExpressionUse::Value,
                        inside_impl,
                        admit_static_string_equality,
                    )?;
                }
                Err(admission_error(
                    "enum construction is not admitted in checked IR",
                ))
            }
            Expression::Borrow { expr, .. } | Expression::Deref(expr) => {
                let _ = Self::validate_expression(
                    expr,
                    bindings,
                    program,
                    ExpressionUse::Value,
                    inside_impl,
                    admit_static_string_equality,
                )?;
                Err(admission_error(
                    "borrow and dereference are not admitted in checked IR",
                ))
            }
            Expression::FieldAccess { object, field } => {
                let context = if bindings.contains_key(STRUCT_ADMISSION_BINDING) {
                    StructExecutionContext::AdmittedFunction
                } else {
                    StructExecutionContext::PreservedContext
                };
                let potential_struct_receiver = match object.as_ref() {
                    Expression::StructLiteral { name, fields } => program
                        .structs
                        .resolve_construction(name, fields, context)
                        .is_ok(),
                    Expression::Identifier(name) => bindings
                        .get(name)
                        .is_some_and(|binding| matches!(&binding.ty, Ty::Struct(_))),
                    Expression::FunctionCall { name, .. } => program
                        .functions
                        .get(name)
                        .is_some_and(|function| matches!(&function.result, Ty::Struct(_))),
                    // Exact receiver eligibility is resolved below after checked
                    // validation of the array and its shared index contract.
                    Expression::IndexAccess { .. } => true,
                    _ => false,
                };
                if !potential_struct_receiver {
                    return Err(admission_error(
                        "aggregate expression is not admitted in checked IR",
                    ));
                }
                let receiver = Self::validate_expression(
                    object,
                    bindings,
                    program,
                    ExpressionUse::Value,
                    inside_impl,
                    admit_static_string_equality,
                )?;
                let (_, _, field) = program
                    .structs
                    .resolve_field(&receiver, field, context)
                    .map_err(|error| match error {
                        StructContractError::PreserveExistingBehavior => {
                            admission_error("aggregate expression is not admitted in checked IR")
                        }
                        _ => IrGenerationError::Admission(error.diagnostic()),
                    })?;
                Ok(field.kind.ty())
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
                for (_, value) in fields {
                    actual_types.push(Self::validate_expression(
                        value,
                        bindings,
                        program,
                        ExpressionUse::Value,
                        inside_impl,
                        admit_static_string_equality,
                    )?);
                }
                program
                    .structs
                    .validate_construction_types(&resolved, &actual_types)
                    .map_err(|error| IrGenerationError::Admission(error.diagnostic()))?;
                Ok(Ty::Struct(resolved.contract.name))
            }
            Expression::TupleLiteral(_)
            | Expression::TupleIndex { .. }
            | Expression::Match { .. } => Err(admission_error(
                "aggregate expression is not admitted in checked IR",
            )),
        }
    }

    fn closure_body_needs_aggregate_lowering(expression: &Expression) -> bool {
        match expression {
            Expression::ArrayLiteral(_)
            | Expression::ArrayRepeat { .. }
            | Expression::IndexAccess { .. }
            | Expression::MethodCall { .. } => true,
            Expression::Binary { left, right, .. }
            | Expression::Comparison { left, right, .. }
            | Expression::Logical { left, right, .. } => {
                Self::closure_body_needs_aggregate_lowering(left)
                    || Self::closure_body_needs_aggregate_lowering(right)
            }
            Expression::FunctionCall { arguments, .. }
            | Expression::Print { arguments, .. }
            | Expression::Println { arguments, .. }
            | Expression::TupleLiteral(arguments) => arguments
                .iter()
                .any(Self::closure_body_needs_aggregate_lowering),
            Expression::Unary { operand, .. }
            | Expression::FieldAccess {
                object: operand, ..
            }
            | Expression::TupleIndex {
                object: operand, ..
            }
            | Expression::Borrow { expr: operand, .. }
            | Expression::Deref(operand)
            | Expression::Closure { body: operand, .. } => {
                Self::closure_body_needs_aggregate_lowering(operand)
            }
            Expression::StructLiteral { fields, .. } => fields
                .iter()
                .any(|(_, value)| Self::closure_body_needs_aggregate_lowering(value)),
            Expression::EnumVariant { data, .. } => data
                .as_deref()
                .is_some_and(Self::closure_body_needs_aggregate_lowering),
            Expression::Match { expr, arms } => {
                Self::closure_body_needs_aggregate_lowering(expr)
                    || arms
                        .iter()
                        .any(|arm| Self::closure_body_needs_aggregate_lowering(&arm.body))
            }
            Expression::IntegerLiteral(_)
            | Expression::FloatLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::Identifier(_) => false,
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
            Type::Named(name) if matches!(name.as_str(), "i32" | "int") => Ty::Int,
            Type::Named(name) if matches!(name.as_str(), "f64" | "float") => Ty::Float,
            Type::Named(name) if name == "bool" => Ty::Bool,
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

    fn encode_callable_signature(parameters: &[Ty], result: &Ty) -> String {
        let type_name = |ty: &Ty| match ty {
            Ty::Int => "int",
            Ty::Float => "float",
            Ty::Bool => "bool",
            _ => "unsupported",
        };
        format!(
            "({})->{}",
            parameters
                .iter()
                .map(type_name)
                .collect::<Vec<_>>()
                .join(","),
            type_name(result)
        )
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

    fn normalize_checked_place_ids(
        functions: &mut HashMap<String, Function>,
        place_hints: &mut PlaceTypeHints,
    ) {
        for function in functions.values_mut() {
            Self::normalize_instruction_places(&mut function.body, &function.name, place_hints);
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
                Inst::Store(place, _) => Self::rewrite_place(place, &places),
                Inst::Load(_, place) => Self::rewrite_place(place, &places),
                Inst::FunctionDef { name, body, .. }
                | Inst::CheckedFunctionDef { name, body, .. } => {
                    Self::normalize_instruction_places(body, name, place_hints)
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
            | Inst::Neg { result, .. } => Some(result),
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
        matches!(inst, Inst::Return(_) | Inst::Jump(_) | Inst::Branch { .. })
    }

    fn restore_bindings(&mut self, before_scope: &HashMap<String, (Value, Ty)>) {
        self.symbol_table.clone_from(before_scope);
    }

    fn generate_binding_expression_ir(
        &mut self,
        expression: Expression,
        type_annotation: Option<&Type>,
        current_function: &mut Function,
    ) -> (Value, Ty) {
        let typed_empty_copy_struct_array = if self.checked_mode {
            type_annotation.and_then(|annotation| {
                self.struct_registry
                    .typed_empty_copy_struct_array_contract(annotation, &expression)
            })
        } else {
            None
        };
        if let Some(contract) = typed_empty_copy_struct_array {
            let place = self.allocate_copy_struct_array_place(&contract, current_function);
            return (place, contract.ty());
        }
        let typed_empty_contract = if self.checked_mode {
            type_annotation
                .and_then(|annotation| typed_empty_numeric_array_contract(annotation, &expression))
        } else {
            None
        };
        let (value, inferred) = self.generate_expression_ir(expression, current_function);
        let Some(contract) = typed_empty_contract else {
            return (value, inferred);
        };

        let expected = contract.ty();
        let logical_element = match &expected {
            Ty::Array(element, 0) if matches!(element.as_ref(), Ty::Int) => LogicalType::Int,
            Ty::Array(element, 0) if matches!(element.as_ref(), Ty::Float) => LogicalType::Float,
            _ => unreachable!("typed empty predicate returns only zero-length numeric arrays"),
        };
        let Value::Reg(place) = &value else {
            unreachable!("empty array lowering must produce an allocation place");
        };
        self.checked_place_hints
            .entry(current_function.name.clone())
            .or_default()
            .insert(PlaceId(*place), logical_element);
        (value, expected)
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
                .map(|field| field.kind.logical_type())
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

    fn copy_struct_place(&mut self, source: Value, ty: &Ty, function: &mut Function) -> Value {
        let value = self.load_copy_struct_value(source, function);
        self.store_copy_struct_value(value, ty, function)
    }

    fn allocate_copy_struct_array_place(
        &mut self,
        contract: &crate::struct_contract::CopyStructArrayContract,
        function: &mut Function,
    ) -> Value {
        let place = Value::Reg(self.next_ptr);
        self.next_ptr += 1;
        let LogicalType::Array { element, .. } = contract.logical_type() else {
            unreachable!("fixed Copy-struct array contract has array logical type")
        };
        function.body.push(Inst::CheckedCopyStructArrayAlloca {
            result: place.clone(),
            element: *element,
            count: contract.count,
        });
        place
    }

    fn copy_struct_array_element_ptr(
        &mut self,
        base: Value,
        index: Value,
        contract: &crate::struct_contract::CopyStructArrayContract,
        function: &mut Function,
    ) -> Value {
        let place = Value::Reg(self.next_ptr);
        self.next_ptr += 1;
        function.body.push(Inst::CheckedCopyStructArrayElementPtr {
            result: place.clone(),
            base,
            index,
            element: contract.element.logical_type(),
            count: contract.count,
        });
        place
    }

    fn copy_struct_array_place(
        &mut self,
        source: Value,
        ty: &Ty,
        function: &mut Function,
    ) -> Value {
        let contract = self
            .struct_registry
            .copy_struct_array_contract(ty)
            .expect("checked fixed Copy-struct array has a shared contract");
        let target = self.allocate_copy_struct_array_place(&contract, function);
        for index in 0..contract.count {
            let source_element = self.copy_struct_array_element_ptr(
                source.clone(),
                Value::ImmInt(index as i64),
                &contract,
                function,
            );
            let value = self.load_copy_struct_value(source_element, function);
            let target_element = self.copy_struct_array_element_ptr(
                target.clone(),
                Value::ImmInt(index as i64),
                &contract,
                function,
            );
            function.body.push(Inst::Store(target_element, value));
        }
        target
    }

    fn load_fixed_copy_array_value(&mut self, place: Value, function: &mut Function) -> Value {
        let value = Value::Reg(self.next_reg);
        self.next_reg += 1;
        function.body.push(Inst::Load(value.clone(), place));
        value
    }

    fn allocate_fixed_copy_array_place(&mut self, ty: &Ty, function: &mut Function) -> Value {
        if let Some(contract) = self.struct_registry.copy_struct_array_contract(ty) {
            return self.allocate_copy_struct_array_place(&contract, function);
        }

        let Ty::Array(element, count) = ty else {
            unreachable!("fixed Copy-array allocation requires an array type")
        };
        let logical_element = match element.as_ref() {
            Ty::Int => LogicalType::Int,
            Ty::Float => LogicalType::Float,
            _ => unreachable!("shared transport contract admits only existing flat arrays"),
        };
        let place_id = self.next_ptr;
        let place = Value::Reg(place_id);
        self.next_ptr += 1;
        function.body.push(Inst::AllocaArray {
            result: place.clone(),
            elem_type: "double".to_string(),
            count: *count,
        });
        self.checked_place_hints
            .entry(function.name.clone())
            .or_default()
            .insert(PlaceId(place_id), logical_element);
        place
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

    fn generate_statement_ir(&mut self, stmt: Statement, current_function: &mut Function) {
        match stmt {
            Statement::Let {
                name,
                mutable: _,
                type_annotation,
                value,
            } => {
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

                if matches!(&expr_type, Ty::Struct(_)) {
                    let storage = if copies_existing_aggregate {
                        self.copy_struct_place(expr_value, &expr_type, current_function)
                    } else {
                        expr_value
                    };
                    self.symbol_table.insert(name, (storage, expr_type));
                } else if self
                    .struct_registry
                    .copy_struct_array_contract(&expr_type)
                    .is_some()
                {
                    let storage = if copies_existing_aggregate {
                        self.copy_struct_array_place(expr_value, &expr_type, current_function)
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
            }
            Statement::Return(expr) => {
                let (mut return_value, return_type) = if let Some(val) = expr {
                    self.generate_expression_ir(val, current_function)
                } else {
                    (Value::ImmInt(0), Ty::Int)
                };
                if matches!(&return_type, Ty::Struct(_)) {
                    return_value = self.load_copy_struct_value(return_value, current_function);
                } else if matches!(&return_type, Ty::Array(_, _)) {
                    return_value = self.load_fixed_copy_array_value(return_value, current_function);
                }
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
                let scope_snapshot = self.symbol_table.clone();
                self.generate_while_loop_ir(condition, body, current_function);
                self.restore_bindings(&scope_snapshot);
            }
            Statement::For {
                variable,
                iterable,
                body,
            } => {
                let scope_snapshot = self.symbol_table.clone();
                self.generate_for_loop_ir(variable, iterable, body, current_function);
                self.restore_bindings(&scope_snapshot);
            }
            Statement::Loop { body } => {
                let scope_snapshot = self.symbol_table.clone();
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
                let scope_snapshot = self.symbol_table.clone();
                // Generate IR for block statements
                for stmt in block.statements {
                    self.generate_statement_ir(stmt, current_function);
                }
                if let Some(expr) = block.expression {
                    self.generate_expression_ir(expr, current_function);
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

    fn generate_expression_ir(&mut self, expr: Expression, function: &mut Function) -> (Value, Ty) {
        match expr {
            Expression::IntegerLiteral(n) => (Value::ImmInt(n), Ty::Int),
            Expression::FloatLiteral(f) => (Value::ImmFloat(f), Ty::Float),
            Expression::Identifier(name) => {
                let (storage, var_type) = self
                    .symbol_table
                    .get(&name)
                    .expect("Undeclared variable")
                    .clone();
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
                // Generate IR for arguments
                let mut arg_values = Vec::new();
                for arg in arguments {
                    let (arg_value, arg_type) = self.generate_expression_ir(arg, function);
                    let arg_value = if matches!(&arg_type, Ty::Struct(_)) {
                        self.load_copy_struct_value(arg_value, function)
                    } else if matches!(&arg_type, Ty::Array(_, _)) {
                        self.load_fixed_copy_array_value(arg_value, function)
                    } else {
                        arg_value
                    };
                    arg_values.push(arg_value);
                }

                let (call_inst, result, return_type) = self.build_function_call(name, arg_values);
                function.body.push(call_inst);
                if matches!(&return_type, Ty::Struct(_)) {
                    let place = self.store_copy_struct_value(result, &return_type, function);
                    (place, return_type)
                } else if matches!(&return_type, Ty::Array(_, _)) {
                    let place = self.store_fixed_copy_array_value(result, &return_type, function);
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
                    match classify_fixed_array_len(
                        &object_ty,
                        &method,
                        arguments.len(),
                        &self.struct_registry,
                    ) {
                        FixedArrayLenDisposition::StaticLength { count, .. } => {
                            return (Value::ImmInt(i64::from(count)), Ty::Int);
                        }
                        _ => {}
                    }
                    let static_string = match (&object_value, &object_ty) {
                        (Value::ImmString(value), Ty::String) => Some(value.as_str()),
                        _ => None,
                    };
                    if let StaticStringLenDisposition::StaticLength(count) =
                        classify_static_string_len(static_string, &method, arguments.len())
                    {
                        return (Value::ImmInt(i64::from(count)), Ty::Int);
                    }
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
                    if let StaticStringPredicateDisposition::StaticBool(value) =
                        classify_static_string_predicate(
                            static_string,
                            &method,
                            &static_argument_refs,
                        )
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
                if method == "iter"
                    && arguments.is_empty()
                    && matches!(object_ty, Ty::Array(_, _) | Ty::Vec(_))
                {
                    // Minimal iterator protocol lowering: `.iter()` reuses the collection value.
                    (object_value, object_ty)
                } else {
                    // Method calls will be resolved to function calls as method lowering expands.
                    (Value::ImmInt(0), Ty::Int)
                }
            }
            Expression::ArrayLiteral(elements) => {
                let count = elements.len();
                let precomputed_first = (self.checked_mode && count > 0)
                    .then(|| self.generate_expression_ir(elements[0].clone(), function));
                if let Some((first_place, first_ty)) = precomputed_first.as_ref() {
                    if let Some(contract) = self
                        .struct_registry
                        .copy_struct_array_contract(&Ty::Array(Box::new(first_ty.clone()), count))
                    {
                        let array = self.allocate_copy_struct_array_place(&contract, function);
                        let first_value =
                            self.load_copy_struct_value(first_place.clone(), function);
                        let first_element = self.copy_struct_array_element_ptr(
                            array.clone(),
                            Value::ImmInt(0),
                            &contract,
                            function,
                        );
                        function.body.push(Inst::Store(first_element, first_value));
                        for (index, element) in elements.into_iter().skip(1).enumerate() {
                            let (place, _) = self.generate_expression_ir(element, function);
                            let value = self.load_copy_struct_value(place, function);
                            let element = self.copy_struct_array_element_ptr(
                                array.clone(),
                                Value::ImmInt((index + 1) as i64),
                                &contract,
                                function,
                            );
                            function.body.push(Inst::Store(element, value));
                        }
                        return (array, contract.ty());
                    }
                }
                let arr_ptr = Value::Reg(self.next_ptr);
                self.next_ptr += 1;
                // Determine element type from first element
                let elem_type = if count > 0 {
                    let (first_val, first_ty) = precomputed_first.unwrap_or_else(|| {
                        self.generate_expression_ir(elements[0].clone(), function)
                    });
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
                if self.checked_mode
                    && let Some(contract) = self
                        .struct_registry
                        .copy_struct_array_contract(&Ty::Array(Box::new(elem_ty.clone()), count))
                {
                    let copied_value = self.load_copy_struct_value(val, function);
                    let array = self.allocate_copy_struct_array_place(&contract, function);
                    for index in 0..count {
                        let element = self.copy_struct_array_element_ptr(
                            array.clone(),
                            Value::ImmInt(index as i64),
                            &contract,
                            function,
                        );
                        function
                            .body
                            .push(Inst::Store(element, copied_value.clone()));
                    }
                    return (array, contract.ty());
                }
                let arr_id = self.next_ptr;
                let arr_ptr = Value::Reg(arr_id);
                self.next_ptr += 1;
                function.body.push(Inst::AllocaArray {
                    result: arr_ptr.clone(),
                    elem_type: "double".to_string(),
                    count,
                });
                if self.checked_mode && count == 0 {
                    let logical_element = match &elem_ty {
                        Ty::Int => LogicalType::Int,
                        Ty::Float => LogicalType::Float,
                        _ => unreachable!("checked admission permits only fixed numeric arrays"),
                    };
                    self.checked_place_hints
                        .entry(function.name.clone())
                        .or_default()
                        .insert(PlaceId(arr_id), logical_element);
                }
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
                if self.checked_mode
                    && let Some(contract) = self.struct_registry.copy_struct_array_contract(&arr_ty)
                {
                    let element =
                        self.copy_struct_array_element_ptr(arr_val, idx_val, &contract, function);
                    let value = self.load_copy_struct_value(element, function);
                    let place = self.store_copy_struct_value(
                        value,
                        &Ty::Struct(contract.element.name.clone()),
                        function,
                    );
                    return (place, Ty::Struct(contract.element.name));
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
                    field_type: field_contract.kind.logical_type(),
                });
                let result = Value::Reg(self.next_reg);
                self.next_reg += 1;
                function.body.push(Inst::Load(result.clone(), field_ptr));
                (result, field_contract.kind.ty())
            }
            Expression::StructLiteral { name, fields } if self.checked_mode => {
                let resolved = self
                    .struct_registry
                    .resolve_construction(&name, &fields, StructExecutionContext::AdmittedFunction)
                    .expect("checked struct construction was admitted");
                let mut values = Vec::with_capacity(fields.len());
                for (_, expression) in fields {
                    values.push(self.generate_expression_ir(expression, function).0);
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
                        .map(|field| field.kind.logical_type())
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
                        field_type: declaration_field.kind.logical_type(),
                    });
                    function.body.push(Inst::Store(field_ptr, value));
                }
                (base, Ty::Struct(resolved.contract.name))
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
            Expression::Closure { params, body } => self.lower_closure_expression(params, *body),
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
        let saved_next_reg = self.next_reg;
        let saved_next_ptr = self.next_ptr;

        // Reset for function generation
        self.symbol_table.clear();
        self.next_reg = 0;
        self.next_ptr = 0;

        let copy_contract = self.copy_function_contracts.get(&name).cloned();

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
            let ptr_reg = Value::Reg(self.next_ptr);
            self.next_ptr += 1;

            // Convert AST Type to Ty
            let param_type = copy_contract.as_ref().map_or_else(
                || self.ast_type_to_ty(&param.param_type),
                |contract| contract.parameters[index].1.ty.clone(),
            );

            self.symbol_table
                .insert(param.name.clone(), (ptr_reg, param_type));
        }

        // Generate function body IR
        let mut function_ir = Function {
            name: name.clone(),
            body: Vec::new(),
            next_reg: 0,
            next_ptr: 0,
        };

        // Allocate parameters
        for param in &parameters {
            let (ptr_reg, _) = self.symbol_table.get(&param.name).unwrap().clone();
            function_ir
                .body
                .push(Inst::Alloca(ptr_reg.clone(), param.name.clone()));
        }

        // Generate statements
        for stmt in body.statements {
            self.generate_statement_ir(stmt, &mut function_ir);
        }

        // Handle block expression (implicit return) or default return when needed.
        if let Some(expr) = body.expression {
            let (mut return_value, return_ty) = self.generate_expression_ir(expr, &mut function_ir);
            if matches!(return_ty, Ty::Struct(_)) {
                return_value = self.load_copy_struct_value(return_value, &mut function_ir);
            } else if matches!(return_ty, Ty::Array(_, _)) {
                return_value = self.load_fixed_copy_array_value(return_value, &mut function_ir);
            }
            function_ir.body.push(Inst::Return(return_value));
        } else if !function_ir
            .body
            .last()
            .is_some_and(Self::instruction_terminates_block)
        {
            // If no explicit return exists, emit a default scalar return.
            // `None` return type is lowered as `void` in codegen.
            function_ir.body.push(Inst::Return(Value::ImmInt(0)));
        }

        // Create a schema-carrying checked definition only for the frozen struct
        // transport class; legacy/raw function definitions keep their old shape.
        let func_def = if let Some(contract) = copy_contract {
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
            Expression::Identifier(name) => {
                let (storage, var_type) = self
                    .symbol_table
                    .get(&name)
                    .expect("Undeclared variable")
                    .clone();
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
            Expression::Closure { params, body } => self.lower_closure_expression(params, *body),
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

        let scope_snapshot = self.symbol_table.clone();

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
        // Generate unique labels
        let loop_start = format!("while_start_{}", self.next_reg);
        self.next_reg += 1;
        let loop_body = format!("while_body_{}", self.next_reg);
        self.next_reg += 1;
        let loop_end = format!("while_end_{}", self.next_reg);
        self.next_reg += 1;

        // Push loop labels onto stack for break/continue
        self.loop_label_stack
            .push((loop_start.clone(), loop_end.clone()));

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
        let loop_start = format!("for_start_{}", self.next_reg);
        self.next_reg += 1;
        let loop_body = format!("for_body_{}", self.next_reg);
        self.next_reg += 1;
        let loop_end = format!("for_end_{}", self.next_reg);
        self.next_reg += 1;

        self.loop_label_stack
            .push((loop_start.clone(), loop_end.clone()));

        // User-visible loop variable slot (updated each iteration with current element).
        let copy_struct_contract = self.struct_registry.copy_struct_contract(&element_ty);
        let loop_var_ptr = if copy_struct_contract.is_some() {
            self.allocate_copy_struct_place(&element_ty, current_function)
        } else {
            let place = Value::Reg(self.next_ptr);
            self.next_ptr += 1;
            current_function
                .body
                .push(Inst::Alloca(place.clone(), variable.clone()));
            let initial_element = match &element_ty {
                Ty::Float => Value::ImmFloat(0.0),
                _ => Value::ImmInt(0),
            };
            current_function
                .body
                .push(Inst::Store(place.clone(), initial_element));
            place
        };
        self.symbol_table
            .insert(variable.clone(), (loop_var_ptr.clone(), element_ty));

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
        let elem_ptr = if let Some(element) = copy_struct_contract {
            let contract = crate::struct_contract::CopyStructArrayContract {
                element,
                count: array_len,
            };
            self.copy_struct_array_element_ptr(
                array_ptr.clone(),
                index_reg.clone(),
                &contract,
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

        if !current_function
            .body
            .last()
            .is_some_and(Self::instruction_terminates_block)
        {
            let next_index = Value::Reg(self.next_reg);
            self.next_reg += 1;
            current_function
                .body
                .push(Inst::Add(next_index.clone(), index_reg, Value::ImmInt(1)));
            current_function
                .body
                .push(Inst::Store(index_ptr, next_index));
            current_function.body.push(Inst::Jump(loop_start));
        }

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
        let loop_start = format!("for_start_{}", self.next_reg);
        self.next_reg += 1;
        let loop_body = format!("for_body_{}", self.next_reg);
        self.next_reg += 1;
        let loop_end = format!("for_end_{}", self.next_reg);
        self.next_reg += 1;

        self.loop_label_stack
            .push((loop_start.clone(), loop_end.clone()));

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
            let incremented_reg = Value::Reg(self.next_reg);
            self.next_reg += 1;
            current_function.body.push(Inst::Add(
                incremented_reg.clone(),
                loop_var_reg,
                Value::ImmInt(1),
            ));
            current_function
                .body
                .push(Inst::Store(var_ptr, incremented_reg));
            current_function.body.push(Inst::Jump(loop_start));
        }

        self.loop_label_stack.pop();
        current_function.body.push(Inst::Label(loop_end));
    }

    fn generate_infinite_loop_ir(
        &mut self,
        body: crate::ast::Block,
        current_function: &mut Function,
    ) {
        // Generate unique labels
        let loop_start = format!("loop_start_{}", self.next_reg);
        self.next_reg += 1;
        let loop_end = format!("loop_end_{}", self.next_reg);
        self.next_reg += 1;

        // Push loop labels onto stack for break/continue
        self.loop_label_stack
            .push((loop_start.clone(), loop_end.clone()));

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
            Type::Named(name) => match name.as_str() {
                "i32" | "int" => Ty::Int,
                "f64" | "float" => Ty::Float,
                "bool" => Ty::Bool,
                "String" => Ty::String,
                other => Ty::Struct(other.to_string()),
            },
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
            Ty::String => "String".to_string(),
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

    fn lower_closure_expression(
        &mut self,
        params: Vec<crate::ast::Parameter>,
        body: Expression,
    ) -> (Value, Ty) {
        let (closure_id, closure_name) = loop {
            let closure_id = self.closure_count;
            let candidate = format!("__closure_{closure_id}");
            self.closure_count += 1;
            if !self.function_return_types.contains_key(&candidate)
                && !self.functions.contains_key(&candidate)
            {
                break (i64::from(closure_id), candidate);
            }
        };

        let ir_params: Vec<(String, String)> = params
            .iter()
            .map(|p| {
                let ty_str = match &p.param_type {
                    Type::Named(n) => match n.as_str() {
                        "i32" | "int" => "i32".to_string(),
                        "f64" | "float" => "double".to_string(),
                        "bool" => "i1".to_string(),
                        _ => "i32".to_string(),
                    },
                    _ => "i32".to_string(),
                };
                (p.name.clone(), ty_str)
            })
            .collect();

        // Closures currently do not capture outer variables; compile them as plain
        // standalone functions with their own symbol table/register space.
        let saved_symbol_table = self.symbol_table.clone();
        let saved_next_reg = self.next_reg;
        let saved_next_ptr = self.next_ptr;

        self.symbol_table.clear();
        self.next_reg = 0;
        self.next_ptr = 0;

        let mut closure_body = Vec::new();
        for p in &params {
            let ptr = Value::Reg(self.next_ptr);
            self.next_ptr += 1;
            closure_body.push(Inst::Alloca(ptr.clone(), p.name.clone()));
            let ty = self.ast_type_to_ty(&p.param_type);
            self.symbol_table.insert(p.name.clone(), (ptr, ty));
        }

        let (body_val, body_ty) = self.generate_expression_ir_for_function(body, &mut closure_body);
        closure_body.push(Inst::Return(body_val));

        let return_type = match &body_ty {
            Ty::Int => Some("i32".to_string()),
            Ty::Float => Some("double".to_string()),
            Ty::Bool => Some("i1".to_string()),
            _ => Some("i32".to_string()),
        };
        self.function_return_types
            .insert(closure_name.clone(), body_ty.clone());

        let closure_fn = Function {
            name: closure_name.clone(),
            body: vec![Inst::FunctionDef {
                name: closure_name.clone(),
                parameters: ir_params,
                return_type,
                body: closure_body,
            }],
            next_reg: self.next_reg,
            next_ptr: self.next_ptr,
        };
        self.functions.insert(closure_name.clone(), closure_fn);

        self.symbol_table = saved_symbol_table;
        self.next_reg = saved_next_reg;
        self.next_ptr = saved_next_ptr;

        (Value::ImmInt(closure_id), Ty::Fn(closure_name))
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

        // Print operations return unit type (represented as 0 for now)
        (Value::ImmInt(0), Ty::Int)
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
        let (left_val, _) = self.generate_expression_ir(left, function);
        let (right_val, _) = self.generate_expression_ir(right, function);

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

        function.body.push(inst);
        (result_reg, Ty::Bool)
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
    fn closure_call_uses_generated_function_symbol() {
        let mut ir_gen = IrGenerator::new();
        let ast = vec![
            AstNode::Statement(Statement::Let {
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
                }),
            }),
            AstNode::Statement(Statement::Expression(Expression::FunctionCall {
                name: "add".to_string(),
                arguments: vec![Expression::IntegerLiteral(1), Expression::IntegerLiteral(2)],
            })),
        ];

        let ir = ir_gen.generate_ir(ast);
        let main = &ir["main"].body;

        assert!(
            main.iter()
                .any(|inst| matches!(inst, crate::ir::Inst::Call { function, .. } if function == "__closure_0"))
        );
        assert!(ir.contains_key("__closure_0"));

        let closure_func = &ir["__closure_0"];
        assert!(
            closure_func
                .body
                .iter()
                .any(|inst| matches!(inst, crate::ir::Inst::FunctionDef { body, .. } if body.iter().any(|i| matches!(i, crate::ir::Inst::Return(_)))))
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
