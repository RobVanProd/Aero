use crate::ast::{AstNode, Block, Expression, Parameter, Pattern, Statement, Type, UnaryOp};
use crate::copydata_trait_dispatch::{TraitDispatchPlan, is_trait_call_marker};
use crate::generic_struct_contract::{
    canonical_copydata_type_matches_logical, parse_canonical_copydata_type_list,
    private_generic_struct_source_name,
};
use crate::ir::LogicalType;
use crate::primitive_contract::PrimitiveKind;
use crate::struct_contract::StructRegistry;
use crate::types::Ty;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const PRIVATE_GENERIC_FUNCTION_PREFIX: &str = "__aero$generic_function$";

#[derive(Debug, Clone)]
struct GenericFunctionTemplate {
    name: String,
    parameters: Vec<Parameter>,
    result: Option<Type>,
    body: Block,
    type_parameters: Vec<String>,
}

#[derive(Debug, Clone)]
struct FunctionSignature {
    parameters: Vec<Type>,
    result: Option<Type>,
}

#[derive(Debug, Clone)]
enum IdentityTypeRole {
    Generic(usize),
    Concrete(Type),
}

#[derive(Debug, Clone, Default)]
struct TypeScopes {
    scopes: Vec<BTreeMap<String, Option<Type>>>,
}

impl TypeScopes {
    fn with_globals(globals: &BTreeMap<String, Type>) -> Self {
        Self {
            scopes: vec![
                globals
                    .iter()
                    .map(|(name, ty)| (name.clone(), Some(ty.clone())))
                    .collect(),
            ],
        }
    }

    fn push(&mut self) {
        self.scopes.push(BTreeMap::new());
    }

    fn pop(&mut self) {
        self.scopes
            .pop()
            .expect("generic-function type scopes remain balanced");
    }

    fn insert(&mut self, name: String, ty: Option<Type>) {
        self.scopes
            .last_mut()
            .expect("generic-function type scopes are nonempty")
            .insert(name, ty);
    }

    fn get(&self, name: &str) -> Option<Type> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned().flatten())
    }
}

#[derive(Debug, Clone, Default)]
struct ParametricScopes {
    scopes: Vec<BTreeMap<String, Option<String>>>,
}

impl ParametricScopes {
    fn new() -> Self {
        Self {
            scopes: vec![BTreeMap::new()],
        }
    }

    fn push(&mut self) {
        self.scopes.push(BTreeMap::new());
    }

    fn pop(&mut self) {
        self.scopes
            .pop()
            .expect("generic-function parametric scopes remain balanced");
    }

    fn insert(&mut self, name: String, parameter: Option<String>) {
        self.scopes
            .last_mut()
            .expect("generic-function parametric scopes are nonempty")
            .insert(name, parameter);
    }

    fn get(&self, name: &str) -> Option<String> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned().flatten())
    }
}

struct GenericFunctionNormalizer {
    templates: BTreeMap<String, GenericFunctionTemplate>,
    signatures: BTreeMap<String, Option<FunctionSignature>>,
    struct_fields: BTreeMap<String, Option<Vec<(String, Type)>>>,
    globals: BTreeMap<String, Type>,
    registry: StructRegistry,
    specializations: BTreeMap<String, AstNode>,
    trait_dispatch: TraitDispatchPlan,
}

pub(crate) fn normalize_generic_copydata_functions(
    ast: Vec<AstNode>,
) -> Result<Vec<AstNode>, String> {
    let initial_registry = StructRegistry::from_top_level_ast(&ast);
    let trait_dispatch = TraitDispatchPlan::from_ast(&ast, &initial_registry)?;
    let ast = trait_dispatch.lower_active_declarations(ast);
    let generic_names = ast
        .iter()
        .filter_map(|node| match node {
            AstNode::Statement(Statement::Function {
                name, type_params, ..
            }) if !type_params.is_empty() => Some(name.clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let has_private = ast.iter().any(|node| {
        matches!(
            node,
            AstNode::Statement(Statement::Function { name, .. })
                if name.starts_with(PRIVATE_GENERIC_FUNCTION_PREFIX)
        )
    });

    if generic_names.is_empty() {
        if has_private {
            let registry = StructRegistry::from_top_level_ast(&ast);
            validate_existing_private_functions(&ast, &registry)?;
        }
        return Ok(ast);
    }
    if has_private {
        return Err(
            "generic-function templates cannot be mixed with pre-specialized private functions"
                .to_string(),
        );
    }

    let mut counts = BTreeMap::new();
    for node in &ast {
        if let AstNode::Statement(Statement::Function { name, .. }) = node {
            *counts.entry(name.clone()).or_insert(0usize) += 1;
        }
    }

    let registry = StructRegistry::from_top_level_ast(&ast);
    let mut templates = BTreeMap::new();
    let mut retained = Vec::with_capacity(ast.len());
    for node in ast {
        match node {
            AstNode::Statement(Statement::Function {
                name,
                parameters,
                return_type,
                body,
                type_params,
                trait_bounds,
            }) if !type_params.is_empty() => {
                if counts.get(&name).copied() != Some(1) {
                    return Err(format!("duplicate generic function definition `{name}`"));
                }
                if templates.contains_key(&name) {
                    return Err(format!("duplicate generic function definition `{name}`"));
                }
                let mut template = GenericFunctionTemplate {
                    name: name.clone(),
                    parameters: parameters.clone(),
                    result: return_type.clone(),
                    body: body.clone(),
                    type_parameters: type_params.clone(),
                };
                trait_dispatch.elaborate_generic_template(
                    &name,
                    &parameters,
                    &type_params,
                    &trait_bounds,
                    &mut template.body,
                )?;
                if validate_template(&template, &generic_names, &registry)? {
                    templates.insert(name, template);
                } else {
                    retained.push(AstNode::Statement(Statement::Function {
                        name,
                        parameters,
                        return_type,
                        body,
                        type_params,
                        trait_bounds,
                    }));
                }
            }
            other => retained.push(other),
        }
    }

    let mut normalizer = GenericFunctionNormalizer {
        templates,
        signatures: BTreeMap::new(),
        struct_fields: BTreeMap::new(),
        globals: BTreeMap::new(),
        registry,
        specializations: BTreeMap::new(),
        trait_dispatch,
    };
    normalizer.prepare_context(&retained)?;
    normalizer.rewrite_top_level(&mut retained)?;

    let mut normalized = normalizer.specializations.into_values().collect::<Vec<_>>();
    normalized.extend(retained);
    Ok(normalized)
}

pub(crate) fn private_generic_function_source_name(name: &str) -> Option<String> {
    let (canonical, _, _) = decode_private_payload(name)?;
    valid_canonical_function_name(&canonical).then_some(canonical)
}

pub(crate) fn private_generic_function_llvm_symbol(name: &str) -> Option<String> {
    private_generic_function_source_name(name)
        .map(|canonical| format!("\"aero.generic.{canonical}\""))
}

pub(crate) fn valid_generic_aware_function_symbol(
    name: &str,
    valid_source_symbol: fn(&str) -> bool,
) -> bool {
    if name.starts_with(PRIVATE_GENERIC_FUNCTION_PREFIX) {
        private_generic_function_source_name(name).is_some()
    } else {
        crate::copydata_trait_dispatch::valid_trait_aware_function_symbol(name, valid_source_symbol)
    }
}

pub(crate) fn valid_private_generic_function_signature(
    name: &str,
    parameters: &[LogicalType],
    result: &LogicalType,
) -> bool {
    if name.starts_with(crate::copydata_trait_dispatch::PRIVATE_TRAIT_IMPL_PREFIX) {
        return crate::copydata_trait_dispatch::valid_private_trait_impl_signature(
            name, parameters, result,
        );
    }
    if !name.starts_with(PRIVATE_GENERIC_FUNCTION_PREFIX) {
        return true;
    }
    let Some((canonical, encoded_contract, encoded_signature)) = decode_private_payload(name)
    else {
        return false;
    };
    let Some(arguments) = canonical_function_arguments(&canonical) else {
        return false;
    };
    let Some((parameter_roles, result_role)) = decode_identity_contract(&encoded_contract) else {
        return false;
    };
    parameter_roles.len() == parameters.len()
        && parameter_roles
            .iter()
            .zip(parameters)
            .all(|(role, actual)| identity_role_matches(role, &arguments, actual))
        && match (&result_role, result) {
            (None, LogicalType::Void) => true,
            (Some(role), actual) if *actual != LogicalType::Void => {
                identity_role_matches(role, &arguments, actual)
            }
            _ => false,
        }
        && encoded_signature == logical_signature_key(parameters, result)
}

pub(crate) fn has_complete_direct_type_parameter_inference(
    type_parameters: &[String],
    parameters: &[Parameter],
) -> bool {
    let declared = type_parameters.iter().cloned().collect::<BTreeSet<_>>();
    if declared.len() != type_parameters.len() {
        return false;
    }
    let inferred = parameters
        .iter()
        .filter_map(|parameter| direct_type_parameter(&parameter.param_type, &declared))
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    inferred == declared
}

impl GenericFunctionNormalizer {
    fn prepare_context(&mut self, ast: &[AstNode]) -> Result<(), String> {
        for node in ast {
            let AstNode::Statement(statement) = node else {
                continue;
            };
            match statement {
                Statement::Const {
                    name,
                    type_annotation,
                    ..
                } => {
                    self.globals.insert(name.clone(), type_annotation.clone());
                }
                Statement::Function {
                    name,
                    parameters,
                    return_type,
                    type_params,
                    ..
                } if type_params.is_empty() => {
                    let signature = FunctionSignature {
                        parameters: parameters
                            .iter()
                            .map(|parameter| parameter.param_type.clone())
                            .collect(),
                        result: return_type.clone(),
                    };
                    match self.signatures.entry(name.clone()) {
                        std::collections::btree_map::Entry::Vacant(entry) => {
                            entry.insert(Some(signature));
                        }
                        std::collections::btree_map::Entry::Occupied(mut entry) => {
                            entry.insert(None);
                        }
                    }
                }
                Statement::StructDef {
                    name,
                    fields,
                    type_params,
                } if type_params.is_empty() => {
                    let fields = fields
                        .iter()
                        .map(|field| (field.name.clone(), field.field_type.clone()))
                        .collect::<Vec<_>>();
                    match self.struct_fields.entry(name.clone()) {
                        std::collections::btree_map::Entry::Vacant(entry) => {
                            entry.insert(Some(fields));
                        }
                        std::collections::btree_map::Entry::Occupied(mut entry) => {
                            entry.insert(None);
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn rewrite_top_level(&mut self, ast: &mut [AstNode]) -> Result<(), String> {
        for node in ast {
            match node {
                AstNode::Statement(Statement::Function {
                    parameters,
                    return_type,
                    body,
                    type_params,
                    ..
                }) if type_params.is_empty() => {
                    let mut scopes = TypeScopes::with_globals(&self.globals);
                    for parameter in parameters {
                        scopes.insert(parameter.name.clone(), Some(parameter.param_type.clone()));
                    }
                    self.rewrite_block(body, &mut scopes, return_type.as_ref())?;
                }
                AstNode::Statement(Statement::Function { type_params, .. })
                    if !type_params.is_empty() => {}
                AstNode::Statement(statement) => {
                    let mut scopes = TypeScopes::with_globals(&self.globals);
                    self.rewrite_statement(statement, &mut scopes, None)?;
                }
                AstNode::Expression(expression) => {
                    let mut scopes = TypeScopes::with_globals(&self.globals);
                    self.rewrite_expression(expression, &mut scopes)?;
                }
            }
        }
        Ok(())
    }

    fn rewrite_block(
        &mut self,
        block: &mut Block,
        scopes: &mut TypeScopes,
        result: Option<&Type>,
    ) -> Result<(), String> {
        scopes.push();
        for statement in &mut block.statements {
            self.rewrite_statement(statement, scopes, result)?;
        }
        if let Some(expression) = &mut block.expression {
            self.rewrite_expression(expression, scopes)?;
        }
        scopes.pop();
        Ok(())
    }

    fn rewrite_statement(
        &mut self,
        statement: &mut Statement,
        scopes: &mut TypeScopes,
        result: Option<&Type>,
    ) -> Result<(), String> {
        match statement {
            Statement::Const {
                name,
                type_annotation,
                value,
                ..
            } => {
                self.rewrite_expression(value, scopes)?;
                scopes.insert(name.clone(), Some(type_annotation.clone()));
                Ok(())
            }
            Statement::Let {
                name,
                type_annotation,
                value,
                ..
            } => {
                if let Some(value) = value {
                    self.rewrite_expression(value, scopes)?;
                }
                let inferred = value
                    .as_ref()
                    .and_then(|value| self.expression_type(value, scopes));
                scopes.insert(name.clone(), type_annotation.clone().or(inferred));
                Ok(())
            }
            Statement::Assignment { target, value } => {
                self.rewrite_expression(target, scopes)?;
                self.rewrite_expression(value, scopes)
            }
            Statement::Return(value) => {
                if let Some(value) = value {
                    self.rewrite_expression(value, scopes)?;
                    if let Some(expected) = result {
                        if let Some(actual) = self.expression_type(value, scopes) {
                            if !self.types_equivalent(expected, &actual) {
                                return Err(format!(
                                    "generic-function specialization return type mismatch: expected {}, actual {}",
                                    display_source_type(expected)?,
                                    display_source_type(&actual)?
                                ));
                            }
                        }
                    }
                }
                Ok(())
            }
            Statement::Expression(expression) => self.rewrite_expression(expression, scopes),
            Statement::Block(block) | Statement::Loop { body: block } => {
                self.rewrite_block(block, scopes, result)
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.rewrite_expression(condition, scopes)?;
                self.rewrite_block(then_block, scopes, result)?;
                if let Some(otherwise) = else_block {
                    scopes.push();
                    self.rewrite_statement(otherwise, scopes, result)?;
                    scopes.pop();
                }
                Ok(())
            }
            Statement::While { condition, body } => {
                self.rewrite_expression(condition, scopes)?;
                self.rewrite_block(body, scopes, result)
            }
            Statement::For {
                variable,
                iterable,
                body,
            } => {
                self.rewrite_expression(iterable, scopes)?;
                let element = self
                    .expression_type(iterable, scopes)
                    .and_then(|ty| match ty {
                        Type::Array(element, _) => Some(*element),
                        _ => None,
                    });
                scopes.push();
                scopes.insert(variable.clone(), element);
                self.rewrite_block(body, scopes, result)?;
                scopes.pop();
                Ok(())
            }
            Statement::Function {
                parameters,
                return_type,
                body,
                type_params,
                ..
            } => {
                if !type_params.is_empty() {
                    return Err("nested generic functions are not admitted in CAP-005".to_string());
                }
                let mut function_scopes = TypeScopes::with_globals(&self.globals);
                for parameter in parameters {
                    function_scopes
                        .insert(parameter.name.clone(), Some(parameter.param_type.clone()));
                }
                self.rewrite_block(body, &mut function_scopes, return_type.as_ref())
            }
            Statement::ImplBlock { methods, .. } => {
                for method in methods {
                    self.rewrite_statement(method, scopes, None)?;
                }
                Ok(())
            }
            Statement::TraitDef { methods, .. } => {
                for method in methods {
                    if let Some(body) = &mut method.body {
                        let mut method_scopes = TypeScopes::with_globals(&self.globals);
                        for parameter in &method.parameters {
                            method_scopes
                                .insert(parameter.name.clone(), Some(parameter.param_type.clone()));
                        }
                        self.rewrite_block(body, &mut method_scopes, method.return_type.as_ref())?;
                    }
                }
                Ok(())
            }
            Statement::Break
            | Statement::Continue
            | Statement::StructDef { .. }
            | Statement::EnumDef { .. }
            | Statement::ModDecl { .. }
            | Statement::UseImport { .. } => Ok(()),
        }
    }

    fn rewrite_expression(
        &mut self,
        expression: &mut Expression,
        scopes: &mut TypeScopes,
    ) -> Result<(), String> {
        match expression {
            Expression::FunctionCall { name, arguments } => {
                for argument in arguments.iter_mut() {
                    self.rewrite_expression(argument, scopes)?;
                }
                if self.templates.contains_key(name) {
                    let source_name = name.clone();
                    let private = self.specialize_call(&source_name, arguments, scopes)?;
                    *name = private;
                }
                Ok(())
            }
            Expression::Binary { left, right, .. }
            | Expression::Comparison { left, right, .. }
            | Expression::Logical { left, right, .. } => {
                self.rewrite_expression(left, scopes)?;
                self.rewrite_expression(right, scopes)
            }
            Expression::MethodCall {
                object, arguments, ..
            } => {
                self.rewrite_expression(object, scopes)?;
                for argument in arguments {
                    self.rewrite_expression(argument, scopes)?;
                }
                Ok(())
            }
            Expression::Print { arguments, .. } | Expression::Println { arguments, .. } => {
                for argument in arguments {
                    self.rewrite_expression(argument, scopes)?;
                }
                Ok(())
            }
            Expression::Unary { operand, .. } => self.rewrite_expression(operand, scopes),
            Expression::ArrayLiteral(elements) | Expression::TupleLiteral(elements) => {
                for element in elements {
                    self.rewrite_expression(element, scopes)?;
                }
                Ok(())
            }
            Expression::ArrayRepeat { value, .. } => self.rewrite_expression(value, scopes),
            Expression::IndexAccess { object, index } => {
                self.rewrite_expression(object, scopes)?;
                self.rewrite_expression(index, scopes)
            }
            Expression::FieldAccess { object, .. } | Expression::TupleIndex { object, .. } => {
                self.rewrite_expression(object, scopes)
            }
            Expression::StructLiteral { fields, .. } => {
                for (_, value) in fields {
                    self.rewrite_expression(value, scopes)?;
                }
                Ok(())
            }
            Expression::EnumVariant { data, .. } => {
                if let Some(values) = data {
                    for value in values {
                        self.rewrite_expression(value, scopes)?;
                    }
                }
                Ok(())
            }
            Expression::Match { expr, arms } => {
                self.rewrite_expression(expr, scopes)?;
                for arm in arms {
                    self.rewrite_expression(&mut arm.body, scopes)?;
                }
                Ok(())
            }
            Expression::Borrow { expr, .. } | Expression::Deref(expr) => {
                self.rewrite_expression(expr, scopes)
            }
            Expression::Closure { body, .. } => self.rewrite_expression(body, scopes),
            Expression::IntegerLiteral(_)
            | Expression::FloatLiteral(_)
            | Expression::CharacterLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::Identifier(_) => Ok(()),
        }
    }

    fn specialize_call(
        &mut self,
        source_name: &str,
        arguments: &[Expression],
        scopes: &TypeScopes,
    ) -> Result<String, String> {
        let template = self
            .templates
            .get(source_name)
            .cloned()
            .expect("generic call lookup is guarded by template membership");
        if arguments.len() != template.parameters.len() {
            return Err(format!(
                "generic function `{source_name}` expects {} argument(s), actual {}",
                template.parameters.len(),
                arguments.len()
            ));
        }

        let actual_types = arguments
            .iter()
            .map(|argument| {
                self.expression_type(argument, scopes).ok_or_else(|| {
                    format!(
                        "generic function call `{source_name}` requires exact CopyData argument types"
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let declared = template
            .type_parameters
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut substitutions = BTreeMap::new();

        for (parameter, actual) in template.parameters.iter().zip(&actual_types) {
            if let Some(type_parameter) = direct_type_parameter(&parameter.param_type, &declared) {
                let concrete = self.canonical_copy_type(actual).ok_or_else(|| {
                    format!(
                        "generic function `{source_name}` requires recursive finite CopyData arguments"
                    )
                })?;
                if let Some(existing) = substitutions.get(type_parameter) {
                    if !self.types_equivalent(existing, &concrete) {
                        return Err(format!(
                            "generic function `{source_name}` inferred conflicting types for `{type_parameter}`: {} and {}",
                            display_source_type(existing)
                                .unwrap_or_else(|_| "<invalid>".to_string()),
                            display_source_type(&concrete)
                                .unwrap_or_else(|_| "<invalid>".to_string())
                        ));
                    }
                } else {
                    substitutions.insert(type_parameter.to_string(), concrete);
                }
            } else if !self.types_equivalent(&parameter.param_type, actual) {
                return Err(format!(
                    "generic function `{source_name}` argument for `{}` requires {}, actual {}",
                    parameter.name,
                    display_source_type(&parameter.param_type)?,
                    display_source_type(actual)?
                ));
            }
        }

        for type_parameter in &template.type_parameters {
            if !substitutions.contains_key(type_parameter) {
                return Err(format!(
                    "generic function `{source_name}` cannot infer type parameter `{type_parameter}` from arguments"
                ));
            }
        }

        let parameters = template
            .parameters
            .iter()
            .map(|parameter| {
                Ok(Parameter {
                    name: parameter.name.clone(),
                    param_type: substitute_type(&parameter.param_type, &substitutions)?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let result = template
            .result
            .as_ref()
            .map(|result| substitute_type(result, &substitutions))
            .transpose()?;
        let mut body = template.body.clone();
        substitute_block(&mut body, &substitutions)?;
        self.trait_dispatch
            .finalize_specialization(source_name, &mut body, &substitutions)?;

        let concrete_arguments = template
            .type_parameters
            .iter()
            .map(|parameter| {
                substitutions
                    .get(parameter)
                    .expect("all generic-function substitutions were proven")
            })
            .collect::<Vec<_>>();
        let canonical = format!(
            "{source_name}<{}>",
            concrete_arguments
                .iter()
                .map(|ty| display_source_type(ty))
                .collect::<Result<Vec<_>, _>>()?
                .join(",")
        );
        let parameter_logical = parameters
            .iter()
            .map(|parameter| self.copy_logical_type(&parameter.param_type))
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| {
                format!(
                    "generic function `{source_name}` specialization has a non-CopyData parameter"
                )
            })?;
        let result_logical = match &result {
            Some(result) => self.copy_logical_type(result).ok_or_else(|| {
                format!("generic function `{source_name}` specialization has a non-CopyData result")
            })?,
            None => LogicalType::Void,
        };
        let signature_key = logical_signature_key(&parameter_logical, &result_logical);
        let contract_key = identity_contract_key(&template, &declared)?;
        let private_name = private_name_for(&canonical, &contract_key, &signature_key);
        let signature = FunctionSignature {
            parameters: parameters
                .iter()
                .map(|parameter| parameter.param_type.clone())
                .collect(),
            result: result.clone(),
        };

        if let Some(existing) = self.signatures.get(&private_name) {
            let Some(existing) = existing else {
                return Err("generic-function private identity collision".to_string());
            };
            if !signatures_equal(existing, &signature, &self.registry) {
                return Err("generic-function private identity collision".to_string());
            }
        } else {
            self.signatures
                .insert(private_name.clone(), Some(signature));
        }
        self.specializations
            .entry(private_name.clone())
            .or_insert_with(|| {
                AstNode::Statement(Statement::Function {
                    name: private_name.clone(),
                    parameters,
                    return_type: result,
                    body,
                    type_params: Vec::new(),
                    trait_bounds: Vec::new(),
                })
            });
        Ok(private_name)
    }

    fn expression_type(&self, expression: &Expression, scopes: &TypeScopes) -> Option<Type> {
        match expression {
            Expression::IntegerLiteral(_) => Some(Type::Named("int".to_string())),
            Expression::FloatLiteral(_) => Some(Type::Named("float".to_string())),
            Expression::CharacterLiteral(_) => Some(Type::Named("char".to_string())),
            Expression::StringLiteral(_) => Some(Type::Named("String".to_string())),
            Expression::Identifier(name) => scopes.get(name),
            Expression::Comparison { .. } | Expression::Logical { .. } => {
                Some(Type::Named("bool".to_string()))
            }
            Expression::Binary {
                left, right, ty, ..
            } => ty.as_ref().and_then(ty_to_type).or_else(|| {
                let left = self.expression_type(left, scopes)?;
                let right = self.expression_type(right, scopes)?;
                if self.types_equivalent(&left, &right)
                    && self
                        .copy_logical_type(&left)
                        .is_some_and(|logical| logical.is_numeric())
                {
                    Some(left)
                } else {
                    None
                }
            }),
            Expression::Unary { op, operand } => match op {
                UnaryOp::Not => Some(Type::Named("bool".to_string())),
                UnaryOp::Negate => self.expression_type(operand, scopes),
            },
            Expression::FunctionCall { name, .. } if is_trait_call_marker(name) => {
                self.trait_dispatch.marker_result_type(name)
            }
            Expression::FunctionCall { name, .. } => self
                .signatures
                .get(name)
                .and_then(Option::as_ref)
                .and_then(|signature| signature.result.clone()),
            Expression::MethodCall { method, .. } if matches!(method.as_str(), "len" | "chars") => {
                Some(Type::Named("int".to_string()))
            }
            Expression::MethodCall { method, .. }
                if matches!(method.as_str(), "is_empty" | "contains") =>
            {
                Some(Type::Named("bool".to_string()))
            }
            Expression::ArrayLiteral(elements) if !elements.is_empty() => self
                .expression_type(&elements[0], scopes)
                .map(|element| Type::Array(Box::new(element), elements.len())),
            Expression::ArrayRepeat { value, count } => self
                .expression_type(value, scopes)
                .map(|element| Type::Array(Box::new(element), *count)),
            Expression::IndexAccess { object, .. } => self
                .expression_type(object, scopes)
                .and_then(|ty| match ty {
                    Type::Array(element, _) => Some(*element),
                    _ => None,
                }),
            Expression::FieldAccess { object, field } => self
                .expression_type(object, scopes)
                .and_then(|ty| self.field_type(&ty, field)),
            Expression::TupleLiteral(elements) if elements.len() >= 2 => elements
                .iter()
                .map(|element| self.expression_type(element, scopes))
                .collect::<Option<Vec<_>>>()
                .map(Type::Tuple),
            Expression::TupleIndex { object, index } => self
                .expression_type(object, scopes)
                .and_then(|ty| match ty {
                    Type::Tuple(elements) => elements.get(*index).cloned(),
                    _ => None,
                }),
            Expression::StructLiteral { name, .. } => Some(Type::Named(name.clone())),
            Expression::Match { arms, .. } if !arms.is_empty() => {
                let first = self.expression_type(&arms[0].body, scopes)?;
                arms.iter()
                    .skip(1)
                    .all(|arm| {
                        self.expression_type(&arm.body, scopes)
                            .is_some_and(|actual| self.types_equivalent(&first, &actual))
                    })
                    .then_some(first)
            }
            Expression::Borrow { expr, mutable } => self
                .expression_type(expr, scopes)
                .map(|ty| Type::Reference(Box::new(ty), *mutable)),
            Expression::Deref(expr) => self.expression_type(expr, scopes).and_then(|ty| match ty {
                Type::Reference(inner, _) => Some(*inner),
                _ => None,
            }),
            Expression::Print { .. }
            | Expression::Println { .. }
            | Expression::EnumVariant { .. }
            | Expression::Closure { .. }
            | Expression::ArrayLiteral(_)
            | Expression::TupleLiteral(_)
            | Expression::MethodCall { .. }
            | Expression::Match { .. } => None,
        }
    }

    fn field_type(&self, ty: &Type, field: &str) -> Option<Type> {
        let Type::Named(name) = ty else {
            return None;
        };
        self.struct_fields
            .get(name)
            .and_then(Option::as_ref)
            .and_then(|fields| fields.iter().find(|(candidate, _)| candidate == field))
            .map(|(_, ty)| ty.clone())
    }

    fn copy_logical_type(&self, ty: &Type) -> Option<LogicalType> {
        self.registry
            .resolve_copy_annotation(ty)
            .map(|contract| contract.logical_type)
    }

    fn canonical_copy_type(&self, ty: &Type) -> Option<Type> {
        self.registry
            .resolve_copy_annotation(ty)
            .and_then(|contract| ty_to_type(&contract.ty))
    }

    fn types_equivalent(&self, left: &Type, right: &Type) -> bool {
        match (self.copy_logical_type(left), self.copy_logical_type(right)) {
            (Some(left), Some(right)) => left == right,
            _ => types_equal(left, right),
        }
    }
}

fn validate_template(
    template: &GenericFunctionTemplate,
    generic_names: &BTreeSet<String>,
    registry: &StructRegistry,
) -> Result<bool, String> {
    if !valid_source_symbol(&template.name) || matches!(template.name.as_str(), "main" | "printf") {
        return Err(format!(
            "generic function `{}` has an invalid or reserved name",
            template.name
        ));
    }
    let mut declared = BTreeSet::new();
    for parameter in &template.type_parameters {
        if !valid_source_symbol(parameter) || !declared.insert(parameter.clone()) {
            return Err(format!(
                "generic function `{}` has duplicate or invalid type parameter `{parameter}`",
                template.name
            ));
        }
    }
    let mut parameter_names = BTreeSet::new();
    for parameter in &template.parameters {
        if !valid_source_symbol(&parameter.name) || !parameter_names.insert(parameter.name.clone())
        {
            return Err(format!(
                "generic function `{}` has duplicate or invalid parameter `{}`",
                template.name, parameter.name
            ));
        }
        if direct_type_parameter(&parameter.param_type, &declared).is_none() {
            if type_mentions_parameters(&parameter.param_type, &declared) {
                return Ok(false);
            }
            if registry
                .resolve_copy_annotation(&parameter.param_type)
                .is_none()
            {
                return Ok(false);
            }
        }
    }
    if !has_complete_direct_type_parameter_inference(
        &template.type_parameters,
        &template.parameters,
    ) {
        return Ok(false);
    }
    if let Some(result) = &template.result {
        if direct_type_parameter(result, &declared).is_none() {
            if type_mentions_parameters(result, &declared) {
                return Ok(false);
            }
            if registry.resolve_copy_annotation(result).is_none() {
                return Ok(false);
            }
        }
    }

    validate_parametric_body(template, generic_names)?;
    Ok(true)
}

fn validate_parametric_body(
    template: &GenericFunctionTemplate,
    generic_names: &BTreeSet<String>,
) -> Result<(), String> {
    let declared = template
        .type_parameters
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let result_parameter = template
        .result
        .as_ref()
        .and_then(|result| direct_type_parameter(result, &declared))
        .map(str::to_string);
    let mut scopes = ParametricScopes::new();
    for parameter in &template.parameters {
        scopes.insert(
            parameter.name.clone(),
            direct_type_parameter(&parameter.param_type, &declared).map(str::to_string),
        );
    }
    validate_parametric_block(
        &template.name,
        &template.body,
        &mut scopes,
        result_parameter.as_deref(),
        &declared,
        generic_names,
    )
}

fn validate_parametric_block(
    function: &str,
    block: &Block,
    scopes: &mut ParametricScopes,
    result_parameter: Option<&str>,
    declared: &BTreeSet<String>,
    generic_names: &BTreeSet<String>,
) -> Result<(), String> {
    scopes.push();
    for statement in &block.statements {
        validate_parametric_statement(
            function,
            statement,
            scopes,
            result_parameter,
            declared,
            generic_names,
        )?;
    }
    if let Some(expression) = &block.expression {
        validate_parametric_return_expression(function, expression, scopes, result_parameter)?;
        reject_generic_calls(function, expression, generic_names)?;
    }
    scopes.pop();
    Ok(())
}

fn validate_parametric_statement(
    function: &str,
    statement: &Statement,
    scopes: &mut ParametricScopes,
    result_parameter: Option<&str>,
    declared: &BTreeSet<String>,
    generic_names: &BTreeSet<String>,
) -> Result<(), String> {
    match statement {
        Statement::Let {
            name,
            type_annotation,
            value,
            ..
        } => {
            let parameter = type_annotation
                .as_ref()
                .and_then(|ty| direct_type_parameter(ty, declared));
            if let Some(parameter) = parameter {
                let Some(value) = value else {
                    return Err(parametric_body_diagnostic(function));
                };
                if direct_parametric_identifier(value, scopes).as_deref() != Some(parameter) {
                    return Err(parametric_body_diagnostic(function));
                }
                scopes.insert(name.clone(), Some(parameter.to_string()));
            } else {
                if value
                    .as_ref()
                    .is_some_and(|value| expression_mentions_parametric(value, scopes))
                {
                    return Err(parametric_body_diagnostic(function));
                }
                scopes.insert(name.clone(), None);
            }
            if let Some(value) = value {
                reject_generic_calls(function, value, generic_names)?;
            }
            Ok(())
        }
        Statement::Assignment { target, value } => {
            if let Expression::Identifier(name) = target {
                if let Some(parameter) = scopes.get(name) {
                    if direct_parametric_identifier(value, scopes).as_deref()
                        != Some(parameter.as_str())
                    {
                        return Err(parametric_body_diagnostic(function));
                    }
                    reject_generic_calls(function, value, generic_names)?;
                    return Ok(());
                }
            }
            if expression_mentions_parametric(target, scopes)
                || expression_mentions_parametric(value, scopes)
            {
                return Err(parametric_body_diagnostic(function));
            }
            reject_generic_calls(function, target, generic_names)?;
            reject_generic_calls(function, value, generic_names)
        }
        Statement::Return(value) => {
            if let Some(value) = value {
                validate_parametric_return_expression(function, value, scopes, result_parameter)?;
                reject_generic_calls(function, value, generic_names)?;
            } else if result_parameter.is_some() {
                return Err(parametric_body_diagnostic(function));
            }
            Ok(())
        }
        Statement::Expression(expression) => {
            if expression_mentions_parametric(expression, scopes) {
                return Err(parametric_body_diagnostic(function));
            }
            reject_generic_calls(function, expression, generic_names)
        }
        Statement::Block(block) | Statement::Loop { body: block } => validate_parametric_block(
            function,
            block,
            scopes,
            result_parameter,
            declared,
            generic_names,
        ),
        Statement::If {
            condition,
            then_block,
            else_block,
        } => {
            if expression_mentions_parametric(condition, scopes) {
                return Err(parametric_body_diagnostic(function));
            }
            reject_generic_calls(function, condition, generic_names)?;
            let mut branch = scopes.clone();
            validate_parametric_block(
                function,
                then_block,
                &mut branch,
                result_parameter,
                declared,
                generic_names,
            )?;
            if let Some(otherwise) = else_block {
                let mut branch = scopes.clone();
                validate_parametric_statement(
                    function,
                    otherwise,
                    &mut branch,
                    result_parameter,
                    declared,
                    generic_names,
                )?;
            }
            Ok(())
        }
        Statement::While { condition, body } => {
            if expression_mentions_parametric(condition, scopes) {
                return Err(parametric_body_diagnostic(function));
            }
            reject_generic_calls(function, condition, generic_names)?;
            let mut loop_scopes = scopes.clone();
            validate_parametric_block(
                function,
                body,
                &mut loop_scopes,
                result_parameter,
                declared,
                generic_names,
            )
        }
        Statement::For {
            variable,
            iterable,
            body,
        } => {
            if expression_mentions_parametric(iterable, scopes) {
                return Err(parametric_body_diagnostic(function));
            }
            reject_generic_calls(function, iterable, generic_names)?;
            let mut loop_scopes = scopes.clone();
            loop_scopes.push();
            loop_scopes.insert(variable.clone(), None);
            validate_parametric_block(
                function,
                body,
                &mut loop_scopes,
                result_parameter,
                declared,
                generic_names,
            )?;
            loop_scopes.pop();
            Ok(())
        }
        Statement::Const { value, .. } => {
            if expression_mentions_parametric(value, scopes) {
                return Err(parametric_body_diagnostic(function));
            }
            reject_generic_calls(function, value, generic_names)
        }
        Statement::Break | Statement::Continue => Ok(()),
        Statement::Function { .. }
        | Statement::StructDef { .. }
        | Statement::EnumDef { .. }
        | Statement::ImplBlock { .. }
        | Statement::TraitDef { .. }
        | Statement::ModDecl { .. }
        | Statement::UseImport { .. } => Err(format!(
            "generic function `{function}` contains a declaration topology not admitted in CAP-005"
        )),
    }
}

fn validate_parametric_return_expression(
    function: &str,
    expression: &Expression,
    scopes: &ParametricScopes,
    result_parameter: Option<&str>,
) -> Result<(), String> {
    if expression_mentions_parametric(expression, scopes)
        && direct_parametric_identifier(expression, scopes).as_deref() != result_parameter
    {
        return Err(parametric_body_diagnostic(function));
    }
    Ok(())
}

fn direct_parametric_identifier(
    expression: &Expression,
    scopes: &ParametricScopes,
) -> Option<String> {
    match expression {
        Expression::Identifier(name) => scopes.get(name),
        _ => None,
    }
}

fn expression_mentions_parametric(expression: &Expression, scopes: &ParametricScopes) -> bool {
    match expression {
        Expression::Identifier(name) => scopes.get(name).is_some(),
        Expression::Binary { left, right, .. }
        | Expression::Comparison { left, right, .. }
        | Expression::Logical { left, right, .. }
        | Expression::IndexAccess {
            object: left,
            index: right,
        } => {
            expression_mentions_parametric(left, scopes)
                || expression_mentions_parametric(right, scopes)
        }
        Expression::FunctionCall { name, arguments } if is_trait_call_marker(name) => arguments
            .iter()
            .skip(1)
            .any(|argument| expression_mentions_parametric(argument, scopes)),
        Expression::FunctionCall { arguments, .. }
        | Expression::Print { arguments, .. }
        | Expression::Println { arguments, .. } => arguments
            .iter()
            .any(|argument| expression_mentions_parametric(argument, scopes)),
        Expression::MethodCall {
            object, arguments, ..
        } => {
            expression_mentions_parametric(object, scopes)
                || arguments
                    .iter()
                    .any(|argument| expression_mentions_parametric(argument, scopes))
        }
        Expression::Unary { operand, .. }
        | Expression::FieldAccess {
            object: operand, ..
        }
        | Expression::TupleIndex {
            object: operand, ..
        }
        | Expression::Borrow { expr: operand, .. }
        | Expression::Deref(operand) => expression_mentions_parametric(operand, scopes),
        Expression::ArrayLiteral(elements) | Expression::TupleLiteral(elements) => elements
            .iter()
            .any(|element| expression_mentions_parametric(element, scopes)),
        Expression::ArrayRepeat { value, .. } => expression_mentions_parametric(value, scopes),
        Expression::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expression_mentions_parametric(value, scopes)),
        Expression::EnumVariant { data, .. } => data.as_ref().is_some_and(|values| {
            values
                .iter()
                .any(|value| expression_mentions_parametric(value, scopes))
        }),
        Expression::Match { expr, arms } => {
            expression_mentions_parametric(expr, scopes)
                || arms
                    .iter()
                    .any(|arm| expression_mentions_parametric(&arm.body, scopes))
        }
        Expression::Closure { body, .. } => expression_mentions_parametric(body, scopes),
        Expression::IntegerLiteral(_)
        | Expression::FloatLiteral(_)
        | Expression::CharacterLiteral(_)
        | Expression::StringLiteral(_) => false,
    }
}

fn reject_generic_calls(
    function: &str,
    expression: &Expression,
    generic_names: &BTreeSet<String>,
) -> Result<(), String> {
    match expression {
        Expression::FunctionCall { name, arguments } => {
            if generic_names.contains(name) {
                return Err(format!(
                    "generic function `{function}` calls generic function `{name}`; generic-to-generic calls are not admitted in CAP-005"
                ));
            }
            for argument in arguments {
                reject_generic_calls(function, argument, generic_names)?;
            }
        }
        Expression::Binary { left, right, .. }
        | Expression::Comparison { left, right, .. }
        | Expression::Logical { left, right, .. }
        | Expression::IndexAccess {
            object: left,
            index: right,
        } => {
            reject_generic_calls(function, left, generic_names)?;
            reject_generic_calls(function, right, generic_names)?;
        }
        Expression::MethodCall {
            object, arguments, ..
        } => {
            reject_generic_calls(function, object, generic_names)?;
            for argument in arguments {
                reject_generic_calls(function, argument, generic_names)?;
            }
        }
        Expression::Print { arguments, .. } | Expression::Println { arguments, .. } => {
            for argument in arguments {
                reject_generic_calls(function, argument, generic_names)?;
            }
        }
        Expression::Unary { operand, .. }
        | Expression::FieldAccess {
            object: operand, ..
        }
        | Expression::TupleIndex {
            object: operand, ..
        }
        | Expression::Borrow { expr: operand, .. }
        | Expression::Deref(operand) => reject_generic_calls(function, operand, generic_names)?,
        Expression::ArrayLiteral(elements) | Expression::TupleLiteral(elements) => {
            for element in elements {
                reject_generic_calls(function, element, generic_names)?;
            }
        }
        Expression::ArrayRepeat { value, .. } => {
            reject_generic_calls(function, value, generic_names)?
        }
        Expression::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                reject_generic_calls(function, value, generic_names)?;
            }
        }
        Expression::EnumVariant { data, .. } => {
            if let Some(values) = data {
                for value in values {
                    reject_generic_calls(function, value, generic_names)?;
                }
            }
        }
        Expression::Match { expr, arms } => {
            reject_generic_calls(function, expr, generic_names)?;
            for arm in arms {
                reject_generic_calls(function, &arm.body, generic_names)?;
            }
        }
        Expression::Closure { body, .. } => reject_generic_calls(function, body, generic_names)?,
        Expression::IntegerLiteral(_)
        | Expression::FloatLiteral(_)
        | Expression::CharacterLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::Identifier(_) => {}
    }
    Ok(())
}

fn parametric_body_diagnostic(function: &str) -> String {
    format!(
        "generic function `{function}` uses a type-parameter value outside CAP-005 whole-value transport"
    )
}

fn substitute_block(
    block: &mut Block,
    substitutions: &BTreeMap<String, Type>,
) -> Result<(), String> {
    for statement in &mut block.statements {
        substitute_statement(statement, substitutions)?;
    }
    if let Some(expression) = &mut block.expression {
        substitute_expression_types(expression, substitutions)?;
    }
    Ok(())
}

fn substitute_statement(
    statement: &mut Statement,
    substitutions: &BTreeMap<String, Type>,
) -> Result<(), String> {
    match statement {
        Statement::Const {
            type_annotation,
            value,
            ..
        } => {
            *type_annotation = substitute_type(type_annotation, substitutions)?;
            substitute_expression_types(value, substitutions)
        }
        Statement::Let {
            type_annotation,
            value,
            ..
        } => {
            if let Some(annotation) = type_annotation {
                *annotation = substitute_type(annotation, substitutions)?;
            }
            if let Some(value) = value {
                substitute_expression_types(value, substitutions)?;
            }
            Ok(())
        }
        Statement::Assignment { target, value } => {
            substitute_expression_types(target, substitutions)?;
            substitute_expression_types(value, substitutions)
        }
        Statement::Return(value) => {
            if let Some(value) = value {
                substitute_expression_types(value, substitutions)?;
            }
            Ok(())
        }
        Statement::Expression(expression) => substitute_expression_types(expression, substitutions),
        Statement::Block(block) | Statement::Loop { body: block } => {
            substitute_block(block, substitutions)
        }
        Statement::If {
            condition,
            then_block,
            else_block,
        } => {
            substitute_expression_types(condition, substitutions)?;
            substitute_block(then_block, substitutions)?;
            if let Some(otherwise) = else_block {
                substitute_statement(otherwise, substitutions)?;
            }
            Ok(())
        }
        Statement::While { condition, body } => {
            substitute_expression_types(condition, substitutions)?;
            substitute_block(body, substitutions)
        }
        Statement::For { iterable, body, .. } => {
            substitute_expression_types(iterable, substitutions)?;
            substitute_block(body, substitutions)
        }
        Statement::Function { .. }
        | Statement::StructDef { .. }
        | Statement::EnumDef { .. }
        | Statement::ImplBlock { .. }
        | Statement::TraitDef { .. }
        | Statement::Break
        | Statement::Continue
        | Statement::ModDecl { .. }
        | Statement::UseImport { .. } => Ok(()),
    }
}

fn substitute_expression_types(
    expression: &mut Expression,
    substitutions: &BTreeMap<String, Type>,
) -> Result<(), String> {
    match expression {
        Expression::Closure { params, body, .. } => {
            for parameter in params {
                parameter.param_type = substitute_type(&parameter.param_type, substitutions)?;
            }
            substitute_expression_types(body, substitutions)
        }
        Expression::Binary { left, right, .. }
        | Expression::Comparison { left, right, .. }
        | Expression::Logical { left, right, .. }
        | Expression::IndexAccess {
            object: left,
            index: right,
        } => {
            substitute_expression_types(left, substitutions)?;
            substitute_expression_types(right, substitutions)
        }
        Expression::FunctionCall { arguments, .. }
        | Expression::Print { arguments, .. }
        | Expression::Println { arguments, .. } => {
            for argument in arguments {
                substitute_expression_types(argument, substitutions)?;
            }
            Ok(())
        }
        Expression::MethodCall {
            object, arguments, ..
        } => {
            substitute_expression_types(object, substitutions)?;
            for argument in arguments {
                substitute_expression_types(argument, substitutions)?;
            }
            Ok(())
        }
        Expression::Unary { operand, .. }
        | Expression::FieldAccess {
            object: operand, ..
        }
        | Expression::TupleIndex {
            object: operand, ..
        }
        | Expression::Borrow { expr: operand, .. }
        | Expression::Deref(operand) => substitute_expression_types(operand, substitutions),
        Expression::ArrayLiteral(elements) | Expression::TupleLiteral(elements) => {
            for element in elements {
                substitute_expression_types(element, substitutions)?;
            }
            Ok(())
        }
        Expression::ArrayRepeat { value, .. } => substitute_expression_types(value, substitutions),
        Expression::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                substitute_expression_types(value, substitutions)?;
            }
            Ok(())
        }
        Expression::EnumVariant { data, .. } => {
            if let Some(values) = data {
                for value in values {
                    substitute_expression_types(value, substitutions)?;
                }
            }
            Ok(())
        }
        Expression::Match { expr, arms } => {
            substitute_expression_types(expr, substitutions)?;
            for arm in arms {
                substitute_pattern(&mut arm.pattern, substitutions)?;
                substitute_expression_types(&mut arm.body, substitutions)?;
            }
            Ok(())
        }
        Expression::IntegerLiteral(_)
        | Expression::FloatLiteral(_)
        | Expression::CharacterLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::Identifier(_) => Ok(()),
    }
}

fn substitute_pattern(
    pattern: &mut Pattern,
    substitutions: &BTreeMap<String, Type>,
) -> Result<(), String> {
    match pattern {
        Pattern::Tuple(patterns) => {
            for pattern in patterns {
                substitute_pattern(pattern, substitutions)?;
            }
        }
        Pattern::Struct { fields, .. } => {
            for (_, pattern) in fields {
                substitute_pattern(pattern, substitutions)?;
            }
        }
        Pattern::Enum { data, .. } => {
            if let Some(patterns) = data {
                for pattern in patterns {
                    substitute_pattern(pattern, substitutions)?;
                }
            }
        }
        Pattern::Wildcard | Pattern::Literal(_) | Pattern::Identifier(_) => {}
    }
    Ok(())
}

fn substitute_type(ty: &Type, substitutions: &BTreeMap<String, Type>) -> Result<Type, String> {
    match ty {
        Type::Named(name) => Ok(substitutions
            .get(name)
            .cloned()
            .unwrap_or_else(|| Type::Named(name.clone()))),
        Type::Array(element, count) => Ok(Type::Array(
            Box::new(substitute_type(element, substitutions)?),
            *count,
        )),
        Type::Tuple(elements) => Ok(Type::Tuple(
            elements
                .iter()
                .map(|element| substitute_type(element, substitutions))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        Type::Reference(element, mutable) => Ok(Type::Reference(
            Box::new(substitute_type(element, substitutions)?),
            *mutable,
        )),
        Type::Generic(name, arguments) => Ok(Type::Generic(
            name.clone(),
            arguments
                .iter()
                .map(|argument| substitute_type(argument, substitutions))
                .collect::<Result<Vec<_>, _>>()?,
        )),
    }
}

fn direct_type_parameter<'a>(ty: &'a Type, declared: &BTreeSet<String>) -> Option<&'a str> {
    match ty {
        Type::Named(name) if declared.contains(name) => Some(name),
        _ => None,
    }
}

fn type_mentions_parameters(ty: &Type, declared: &BTreeSet<String>) -> bool {
    match ty {
        Type::Named(name) => declared.contains(name),
        Type::Array(element, _) | Type::Reference(element, _) => {
            type_mentions_parameters(element, declared)
        }
        Type::Tuple(elements) | Type::Generic(_, elements) => elements
            .iter()
            .any(|element| type_mentions_parameters(element, declared)),
    }
}

fn validate_existing_private_functions(
    ast: &[AstNode],
    registry: &StructRegistry,
) -> Result<(), String> {
    for node in ast {
        let AstNode::Statement(Statement::Function {
            name,
            parameters,
            return_type,
            type_params,
            trait_bounds,
            ..
        }) = node
        else {
            continue;
        };
        if !name.starts_with(PRIVATE_GENERIC_FUNCTION_PREFIX) {
            continue;
        }
        if !type_params.is_empty() || !trait_bounds.is_empty() {
            return Err(format!(
                "invalid private generic-function definition `{name}`"
            ));
        }
        let parameter_logical = parameters
            .iter()
            .map(|parameter| {
                registry
                    .resolve_copy_annotation(&parameter.param_type)
                    .map(|contract| contract.logical_type)
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| format!("invalid private generic-function signature `{name}`"))?;
        let result_logical = match return_type {
            Some(result) => registry
                .resolve_copy_annotation(result)
                .map(|contract| contract.logical_type)
                .ok_or_else(|| format!("invalid private generic-function result `{name}`"))?,
            None => LogicalType::Void,
        };
        if !valid_private_generic_function_signature(name, &parameter_logical, &result_logical) {
            return Err(format!(
                "private generic-function identity `{name}` does not match its exact signature"
            ));
        }
    }
    Ok(())
}

fn signatures_equal(
    left: &FunctionSignature,
    right: &FunctionSignature,
    registry: &StructRegistry,
) -> bool {
    left.parameters.len() == right.parameters.len()
        && left
            .parameters
            .iter()
            .zip(&right.parameters)
            .all(|(left, right)| {
                match (
                    registry.resolve_copy_annotation(left),
                    registry.resolve_copy_annotation(right),
                ) {
                    (Some(left), Some(right)) => left.logical_type == right.logical_type,
                    _ => types_equal(left, right),
                }
            })
        && match (&left.result, &right.result) {
            (None, None) => true,
            (Some(left), Some(right)) => match (
                registry.resolve_copy_annotation(left),
                registry.resolve_copy_annotation(right),
            ) {
                (Some(left), Some(right)) => left.logical_type == right.logical_type,
                _ => types_equal(left, right),
            },
            _ => false,
        }
}

fn ty_to_type(ty: &Ty) -> Option<Type> {
    match ty {
        Ty::Int => Some(Type::Named("int".to_string())),
        Ty::Float => Some(Type::Named("float".to_string())),
        Ty::Bool => Some(Type::Named("bool".to_string())),
        Ty::Char => Some(Type::Named("char".to_string())),
        Ty::Array(element, count) => Some(Type::Array(Box::new(ty_to_type(element)?), *count)),
        Ty::Tuple(elements) => elements
            .iter()
            .map(ty_to_type)
            .collect::<Option<Vec<_>>>()
            .map(Type::Tuple),
        Ty::Struct(name) => Some(Type::Named(name.clone())),
        Ty::String
        | Ty::Enum(_)
        | Ty::Void
        | Ty::Reference(_, _)
        | Ty::TypeParam(_)
        | Ty::Option(_)
        | Ty::Result(_, _)
        | Ty::Vec(_)
        | Ty::HashMap(_, _)
        | Ty::Fn(_) => None,
    }
}

fn display_source_type(ty: &Type) -> Result<String, String> {
    match ty {
        Type::Named(name) => {
            if let Some(primitive) = PrimitiveKind::from_source_name(name) {
                Ok(match primitive {
                    PrimitiveKind::Int => "int".to_string(),
                    PrimitiveKind::Float => "float".to_string(),
                    PrimitiveKind::Bool => "bool".to_string(),
                    PrimitiveKind::Char => "char".to_string(),
                })
            } else if let Some(source) = private_generic_struct_source_name(name) {
                Ok(source)
            } else if valid_source_symbol(name) {
                Ok(name.clone())
            } else {
                Err(format!("invalid CopyData type identity `{name}`"))
            }
        }
        Type::Array(element, count) => Ok(format!("[{};{count}]", display_source_type(element)?)),
        Type::Tuple(elements) if elements.len() >= 2 => Ok(format!(
            "({})",
            elements
                .iter()
                .map(display_source_type)
                .collect::<Result<Vec<_>, _>>()?
                .join(",")
        )),
        Type::Tuple(_) => {
            Err("generic-function CopyData tuples require arity at least two".to_string())
        }
        Type::Reference(_, _) | Type::Generic(_, _) => {
            Err("generic-function specialization type is not CopyData".to_string())
        }
    }
}

fn types_equal(left: &Type, right: &Type) -> bool {
    match (left, right) {
        (Type::Named(left), Type::Named(right)) => {
            match (
                PrimitiveKind::from_source_name(left),
                PrimitiveKind::from_source_name(right),
            ) {
                (Some(left), Some(right)) => left == right,
                _ => left == right,
            }
        }
        (Type::Array(left, left_count), Type::Array(right, right_count)) => {
            left_count == right_count && types_equal(left, right)
        }
        (Type::Tuple(left), Type::Tuple(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| types_equal(left, right))
        }
        (Type::Reference(left, left_mut), Type::Reference(right, right_mut)) => {
            left_mut == right_mut && types_equal(left, right)
        }
        (Type::Generic(left, left_args), Type::Generic(right, right_args)) => {
            left == right
                && left_args.len() == right_args.len()
                && left_args
                    .iter()
                    .zip(right_args)
                    .all(|(left, right)| types_equal(left, right))
        }
        _ => false,
    }
}

fn logical_signature_key(parameters: &[LogicalType], result: &LogicalType) -> String {
    format!(
        "({})->{}",
        parameters
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(","),
        result
    )
}

fn identity_contract_key(
    template: &GenericFunctionTemplate,
    declared: &BTreeSet<String>,
) -> Result<String, String> {
    let indexes = template
        .type_parameters
        .iter()
        .enumerate()
        .map(|(index, parameter)| (parameter.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let role = |ty: &Type| -> Result<String, String> {
        if let Some(parameter) = direct_type_parameter(ty, declared) {
            return Ok(format!(
                "g{}",
                indexes
                    .get(parameter)
                    .expect("declared type parameter has a stable index")
            ));
        }
        Ok(format!("c{}", encode_hex(&display_source_type(ty)?)))
    };
    let parameters = template
        .parameters
        .iter()
        .map(|parameter| role(&parameter.param_type))
        .collect::<Result<Vec<_>, _>>()?
        .join(".");
    let result = template
        .result
        .as_ref()
        .map(role)
        .transpose()?
        .unwrap_or_else(|| "v".to_string());
    Ok(format!("{parameters}>{result}"))
}

fn private_name_for(canonical: &str, contract: &str, signature: &str) -> String {
    let payload = format!("{canonical}|{contract}|{signature}");
    let encoded = encode_hex(&payload);
    format!("{PRIVATE_GENERIC_FUNCTION_PREFIX}{encoded}")
}

#[cfg(test)]
pub(crate) fn private_name_for_test(
    canonical: &str,
    contract: &str,
    parameters: &[LogicalType],
    result: &LogicalType,
) -> String {
    private_name_for(
        canonical,
        contract,
        &logical_signature_key(parameters, result),
    )
}

fn encode_hex(value: &str) -> String {
    value
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn decode_hex(encoded: &str) -> Option<String> {
    if encoded.is_empty()
        || encoded.len() % 2 != 0
        || !encoded.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return None;
    }
    let bytes = (0..encoded.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&encoded[index..index + 2], 16).ok())
        .collect::<Option<Vec<_>>>()?;
    String::from_utf8(bytes).ok()
}

fn decode_private_payload(name: &str) -> Option<(String, String, String)> {
    let encoded = name.strip_prefix(PRIVATE_GENERIC_FUNCTION_PREFIX)?;
    let payload = decode_hex(encoded)?;
    let mut parts = payload.split('|');
    let canonical = parts.next()?;
    let contract = parts.next()?;
    let signature = parts.next()?;
    if canonical.is_empty() || contract.is_empty() || signature.is_empty() || parts.next().is_some()
    {
        return None;
    }
    Some((
        canonical.to_string(),
        contract.to_string(),
        signature.to_string(),
    ))
}

fn canonical_function_arguments(canonical: &str) -> Option<Vec<Type>> {
    let Some((name, arguments)) = canonical.split_once('<') else {
        return None;
    };
    if !valid_source_symbol(name) || !arguments.ends_with('>') || arguments.len() <= 1 {
        return None;
    }
    parse_canonical_copydata_type_list(&arguments[..arguments.len() - 1])
}

fn valid_canonical_function_name(canonical: &str) -> bool {
    canonical_function_arguments(canonical).is_some()
}

fn decode_identity_contract(
    contract: &str,
) -> Option<(Vec<IdentityTypeRole>, Option<IdentityTypeRole>)> {
    let (parameters, result) = contract.split_once('>')?;
    if parameters.is_empty() || result.is_empty() || result.contains('>') {
        return None;
    }
    let parameters = parameters
        .split('.')
        .map(decode_identity_role)
        .collect::<Option<Vec<_>>>()?;
    let result = if result == "v" {
        None
    } else {
        Some(decode_identity_role(result)?)
    };
    Some((parameters, result))
}

fn decode_identity_role(encoded: &str) -> Option<IdentityTypeRole> {
    if let Some(index) = encoded.strip_prefix('g') {
        if index.is_empty() || (index.len() > 1 && index.starts_with('0')) {
            return None;
        }
        let parsed = index.parse::<usize>().ok()?;
        return (parsed.to_string() == index).then_some(IdentityTypeRole::Generic(parsed));
    }
    let source = decode_hex(encoded.strip_prefix('c')?)?;
    let mut types = parse_canonical_copydata_type_list(&source)?;
    (types.len() == 1).then(|| IdentityTypeRole::Concrete(types.remove(0)))
}

fn identity_role_matches(
    role: &IdentityTypeRole,
    arguments: &[Type],
    actual: &LogicalType,
) -> bool {
    let expected = match role {
        IdentityTypeRole::Generic(index) => arguments.get(*index),
        IdentityTypeRole::Concrete(expected) => Some(expected),
    };
    expected.is_some_and(|expected| canonical_copydata_type_matches_logical(expected, actual))
}

fn valid_source_symbol(name: &str) -> bool {
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn template_source() -> Vec<AstNode> {
        let source = "fn choose<T>(first: T, second: T, take_first: bool) -> T { if take_first { return first; } second } fn main() -> int { choose(1, 2, 1 < 2) }";
        let tokens = crate::lexer::try_tokenize_with_locations(source, None).expect("lex");
        crate::parser::parse_with_locations(tokens).expect("parse")
    }

    #[test]
    fn specialization_is_idempotent_and_signature_bound() {
        let first = normalize_generic_copydata_functions(template_source()).expect("normalize");
        let second = normalize_generic_copydata_functions(first.clone()).expect("renormalize");
        assert_eq!(format!("{first:?}"), format!("{second:?}"));

        let private = first
            .iter()
            .find_map(|node| match node {
                AstNode::Statement(Statement::Function {
                    name, parameters, ..
                }) if name.starts_with(PRIVATE_GENERIC_FUNCTION_PREFIX) => Some((name, parameters)),
                _ => None,
            })
            .expect("private specialization");
        assert_eq!(
            private_generic_function_source_name(private.0).as_deref(),
            Some("choose<int>")
        );
        assert!(valid_private_generic_function_signature(
            private.0,
            &[LogicalType::Int, LogicalType::Int, LogicalType::Bool],
            &LogicalType::Int
        ));
        assert!(!valid_private_generic_function_signature(
            private.0,
            &[LogicalType::Char, LogicalType::Int, LogicalType::Bool],
            &LogicalType::Int
        ));
    }

    #[test]
    fn parametric_operations_fail_closed_and_result_only_templates_remain_quarantined() {
        let operation =
            "fn add<T>(left: T, right: T) -> T { left + right } fn main() -> int { add(1, 2) }";
        let tokens = crate::lexer::try_tokenize_with_locations(operation, None).expect("lex");
        let ast = crate::parser::parse_with_locations(tokens).expect("parse");
        let error = normalize_generic_copydata_functions(ast).expect_err("must reject");
        assert!(
            error.contains("outside CAP-005 whole-value transport"),
            "{error}"
        );

        let result_only = "fn make<T>(value: int) -> T { value } fn main() -> int { make(1) }";
        let tokens = crate::lexer::try_tokenize_with_locations(result_only, None).expect("lex");
        let ast = crate::parser::parse_with_locations(tokens).expect("parse");
        let normalized = normalize_generic_copydata_functions(ast).expect("preserve quarantine");
        assert!(normalized.iter().any(|node| matches!(
            node,
            AstNode::Statement(Statement::Function { name, type_params, .. })
                if name == "make" && !type_params.is_empty()
        )));
    }
}
