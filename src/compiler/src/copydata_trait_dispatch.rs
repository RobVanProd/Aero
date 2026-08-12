use crate::ast::{AstNode, Block, Expression, Parameter, Statement, Type};
use crate::ir::LogicalType;
use crate::specialization_contract::{
    decode_private_identity, logical_signature_key, private_identity, specialization_types_equal,
    valid_source_symbol,
};
use crate::struct_contract::StructRegistry;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const PRIVATE_TRAIT_IMPL_PREFIX: &str = "__aero$trait_impl$";
const PRIVATE_TRAIT_CALL_PREFIX: &str = "__aero$trait_call$";

#[derive(Debug, Clone)]
struct TraitMethodContract {
    parameters: Vec<Parameter>,
    result: Option<Type>,
}

#[derive(Debug, Clone)]
struct TraitContract {
    methods: BTreeMap<String, TraitMethodContract>,
    method_order: Vec<String>,
}

#[derive(Debug, Clone)]
struct TraitImplContract {
    helper: String,
    function: AstNode,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TraitDispatchPlan {
    traits: BTreeMap<String, TraitContract>,
    implementations: BTreeMap<(String, String, String), TraitImplContract>,
    active_traits: BTreeSet<String>,
}

impl TraitDispatchPlan {
    pub(crate) fn from_ast(ast: &[AstNode], registry: &StructRegistry) -> Result<Self, String> {
        reject_source_private_symbols(ast)?;

        let active_traits = ast
            .iter()
            .filter_map(|node| match node {
                AstNode::Statement(Statement::Function {
                    type_params,
                    trait_bounds,
                    ..
                }) if !type_params.is_empty() && !trait_bounds.is_empty() => Some(trait_bounds),
                _ => None,
            })
            .flat_map(|bounds| bounds.iter())
            .flat_map(|(_, traits)| traits.iter().cloned())
            .collect::<BTreeSet<_>>();

        if active_traits.is_empty() {
            return Ok(Self::default());
        }

        for node in ast {
            match node {
                AstNode::Statement(Statement::TraitDef { name, .. })
                    if !active_traits.contains(name) =>
                {
                    return Err(format!(
                        "trait-bound static dispatch does not admit unused trait declaration `{name}`"
                    ));
                }
                AstNode::Statement(Statement::ImplBlock {
                    trait_name: None, ..
                }) => {
                    return Err(
                        "trait-bound static dispatch does not admit inherent impl blocks"
                            .to_string(),
                    );
                }
                AstNode::Statement(Statement::ImplBlock {
                    trait_name: Some(name),
                    ..
                }) if !active_traits.contains(name) => {
                    return Err(format!(
                        "trait-bound static dispatch does not admit impl for unbound trait `{name}`"
                    ));
                }
                _ => {}
            }
        }

        let mut trait_counts = BTreeMap::new();
        for node in ast {
            if let AstNode::Statement(Statement::TraitDef { name, .. }) = node {
                *trait_counts.entry(name.clone()).or_insert(0usize) += 1;
            }
        }

        let mut traits = BTreeMap::new();
        for node in ast {
            let AstNode::Statement(Statement::TraitDef {
                name,
                type_params,
                methods,
            }) = node
            else {
                continue;
            };
            if !active_traits.contains(name) {
                continue;
            }
            if trait_counts.get(name).copied() != Some(1) {
                return Err(format!(
                    "trait-bound static dispatch requires one unique trait definition `{name}`"
                ));
            }
            if !valid_source_symbol(name) || !type_params.is_empty() {
                return Err(format!(
                    "trait-bound static dispatch requires a unique nongeneric trait `{name}`"
                ));
            }
            if methods.is_empty() {
                return Err(format!(
                    "trait-bound static dispatch trait `{name}` must declare at least one required method"
                ));
            }

            let mut method_contracts = BTreeMap::new();
            let mut method_order = Vec::new();
            for method in methods {
                if method.body.is_some() {
                    return Err(format!(
                        "trait-bound static dispatch does not admit default method `{}::{}`",
                        name, method.name
                    ));
                }
                if !valid_source_symbol(&method.name) || method_contracts.contains_key(&method.name)
                {
                    return Err(format!(
                        "trait-bound static dispatch requires unique method names in trait `{name}`"
                    ));
                }
                validate_trait_method_signature(name, method, registry)?;
                method_contracts.insert(
                    method.name.clone(),
                    TraitMethodContract {
                        parameters: method.parameters.clone(),
                        result: method.return_type.clone(),
                    },
                );
                method_order.push(method.name.clone());
            }
            traits.insert(
                name.clone(),
                TraitContract {
                    methods: method_contracts,
                    method_order,
                },
            );
        }

        for name in &active_traits {
            if !traits.contains_key(name) {
                return Err(format!(
                    "trait-bound static dispatch references unknown trait `{name}`"
                ));
            }
        }

        let struct_targets = ast
            .iter()
            .filter_map(|node| match node {
                AstNode::Statement(Statement::StructDef {
                    name, type_params, ..
                }) if type_params.is_empty() => Some(name.clone()),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        let mut pair_counts = BTreeMap::new();
        for node in ast {
            if let AstNode::Statement(Statement::ImplBlock {
                type_name,
                trait_name: Some(trait_name),
                ..
            }) = node
            {
                if active_traits.contains(trait_name) {
                    *pair_counts
                        .entry((trait_name.clone(), type_name.clone()))
                        .or_insert(0usize) += 1;
                }
            }
        }

        let mut implementations = BTreeMap::new();
        for node in ast {
            let AstNode::Statement(Statement::ImplBlock {
                type_name,
                methods,
                type_params,
                trait_name: Some(trait_name),
            }) = node
            else {
                continue;
            };
            if !active_traits.contains(trait_name) {
                continue;
            }
            if pair_counts
                .get(&(trait_name.clone(), type_name.clone()))
                .copied()
                != Some(1)
            {
                return Err(format!(
                    "trait-bound static dispatch requires one unique `impl {trait_name} for {type_name}`"
                ));
            }
            if !type_params.is_empty()
                || !struct_targets.contains(type_name)
                || registry
                    .resolve_copy_annotation(&Type::Named(type_name.clone()))
                    .is_none()
            {
                return Err(format!(
                    "trait-bound static dispatch impl target `{type_name}` must be a unique nongeneric recursive CopyData struct"
                ));
            }
            let contract = traits
                .get(trait_name)
                .expect("active trait definitions were proven");
            validate_and_build_impl(
                trait_name,
                type_name,
                methods,
                contract,
                registry,
                &mut implementations,
            )?;
        }

        for trait_name in &active_traits {
            if !implementations
                .keys()
                .any(|(candidate, _, _)| candidate == trait_name)
            {
                return Err(format!(
                    "trait-bound static dispatch trait `{trait_name}` has no admitted concrete implementation"
                ));
            }
        }

        Ok(Self {
            traits,
            implementations,
            active_traits,
        })
    }

    pub(crate) fn is_active(&self) -> bool {
        !self.active_traits.is_empty()
    }

    pub(crate) fn lower_active_declarations(&self, ast: Vec<AstNode>) -> Vec<AstNode> {
        if !self.is_active() {
            return ast;
        }
        let mut lowered = ast
            .into_iter()
            .filter(|node| match node {
                AstNode::Statement(Statement::TraitDef { name, .. }) => {
                    !self.active_traits.contains(name)
                }
                AstNode::Statement(Statement::ImplBlock {
                    trait_name: Some(name),
                    ..
                }) => !self.active_traits.contains(name),
                _ => true,
            })
            .collect::<Vec<_>>();
        lowered.extend(
            self.implementations
                .values()
                .map(|contract| contract.function.clone()),
        );
        lowered
    }

    pub(crate) fn elaborate_generic_template(
        &self,
        function: &str,
        parameters: &[Parameter],
        type_parameters: &[String],
        trait_bounds: &[(String, Vec<String>)],
        body: &mut Block,
    ) -> Result<(), String> {
        if trait_bounds.is_empty() {
            return Ok(());
        }
        if !self.is_active() {
            return Err(format!(
                "generic function `{function}` has trait bounds without an active trait-dispatch contract"
            ));
        }

        let declared = type_parameters.iter().cloned().collect::<BTreeSet<_>>();
        if declared.len() != type_parameters.len() {
            return Err(format!(
                "generic function `{function}` has duplicate type parameters"
            ));
        }
        let mut bounds = BTreeMap::<String, BTreeSet<String>>::new();
        for (parameter, required) in trait_bounds {
            if !declared.contains(parameter) || required.is_empty() {
                return Err(format!(
                    "generic function `{function}` has an undeclared or empty trait bound for `{parameter}`"
                ));
            }
            let entry = bounds.entry(parameter.clone()).or_default();
            for trait_name in required {
                if !self.active_traits.contains(trait_name) || !entry.insert(trait_name.clone()) {
                    return Err(format!(
                        "generic function `{function}` has a duplicate or unknown bound `{parameter}: {trait_name}`"
                    ));
                }
            }
        }

        let direct_parameters = parameters
            .iter()
            .filter_map(|parameter| match &parameter.param_type {
                Type::Named(name) if bounds.contains_key(name) => {
                    Some((parameter.name.clone(), name.clone()))
                }
                _ => None,
            })
            .collect::<BTreeMap<_, _>>();
        for parameter in bounds.keys() {
            if !direct_parameters
                .values()
                .any(|candidate| candidate == parameter)
            {
                return Err(format!(
                    "generic function `{function}` cannot infer bounded type parameter `{parameter}` from a direct value parameter"
                ));
            }
        }

        let mut used = BTreeSet::new();
        elaborate_block(
            body,
            function,
            &direct_parameters,
            &bounds,
            &self.traits,
            &mut used,
        )?;
        for (parameter, traits) in &bounds {
            for trait_name in traits {
                if !used.contains(&(parameter.clone(), trait_name.clone())) {
                    return Err(format!(
                        "generic function `{function}` declares unused bound `{parameter}: {trait_name}`"
                    ));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn finalize_specialization(
        &self,
        function: &str,
        body: &mut Block,
        substitutions: &BTreeMap<String, Type>,
    ) -> Result<(), String> {
        finalize_block(body, function, substitutions, &self.implementations)
    }

    pub(crate) fn marker_result_type(&self, name: &str) -> Option<Type> {
        let marker = decode_call_marker(name)?;
        self.traits
            .get(&marker.trait_name)?
            .methods
            .get(&marker.method)?
            .result
            .clone()
    }
}

#[derive(Debug)]
struct CallMarker {
    trait_name: String,
    method: String,
    type_parameter: String,
}

fn validate_trait_method_signature(
    trait_name: &str,
    method: &crate::ast::TraitMethod,
    registry: &StructRegistry,
) -> Result<(), String> {
    let Some(receiver) = method.parameters.first() else {
        return Err(format!(
            "trait-bound static dispatch method `{trait_name}::{}` requires a leading immutable &self receiver",
            method.name
        ));
    };
    if receiver.name != "self"
        || !matches!(
            &receiver.param_type,
            Type::Reference(inner, false) if matches!(inner.as_ref(), Type::Named(name) if name == "Self")
        )
    {
        return Err(format!(
            "trait-bound static dispatch method `{trait_name}::{}` requires exactly one leading immutable &self receiver",
            method.name
        ));
    }
    let mut names = BTreeSet::new();
    names.insert("self".to_string());
    for parameter in method.parameters.iter().skip(1) {
        if !valid_source_symbol(&parameter.name)
            || !names.insert(parameter.name.clone())
            || registry
                .resolve_copy_annotation(&parameter.param_type)
                .is_none()
        {
            return Err(format!(
                "trait-bound static dispatch method `{trait_name}::{}` requires unique recursive CopyData parameters",
                method.name
            ));
        }
    }
    if method
        .return_type
        .as_ref()
        .is_some_and(|result| registry.resolve_copy_annotation(result).is_none())
    {
        return Err(format!(
            "trait-bound static dispatch method `{trait_name}::{}` requires a recursive CopyData or Void result",
            method.name
        ));
    }
    Ok(())
}

fn validate_and_build_impl(
    trait_name: &str,
    target: &str,
    methods: &[Statement],
    contract: &TraitContract,
    registry: &StructRegistry,
    implementations: &mut BTreeMap<(String, String, String), TraitImplContract>,
) -> Result<(), String> {
    let mut observed = BTreeMap::new();
    for (index, statement) in methods.iter().enumerate() {
        let Statement::Function {
            name,
            parameters,
            return_type,
            body,
            type_params,
            trait_bounds,
        } = statement
        else {
            return Err(format!(
                "impl {trait_name} for {target} contains a non-method item"
            ));
        };
        if !type_params.is_empty() || !trait_bounds.is_empty() || observed.contains_key(name) {
            return Err(format!(
                "impl {trait_name} for {target} requires unique nongeneric methods"
            ));
        }
        let Some(expected) = contract.methods.get(name) else {
            return Err(format!(
                "impl {trait_name} for {target} defines extra method `{name}`"
            ));
        };
        if contract.method_order.get(index) != Some(name) {
            return Err(format!(
                "impl {trait_name} for {target} method order does not match the trait declaration"
            ));
        }
        let concrete_parameters = expected
            .parameters
            .iter()
            .map(|parameter| Parameter {
                name: parameter.name.clone(),
                param_type: replace_self_type(&parameter.param_type, target),
            })
            .collect::<Vec<_>>();
        let concrete_result = expected
            .result
            .as_ref()
            .map(|result| replace_self_type(result, target));
        if parameters.len() != concrete_parameters.len()
            || parameters
                .iter()
                .zip(&concrete_parameters)
                .any(|(actual, expected)| {
                    actual.name != expected.name
                        || !same_type(
                            &replace_self_type(&actual.param_type, target),
                            &expected.param_type,
                        )
                })
            || !same_optional_type(
                return_type
                    .as_ref()
                    .map(|result| replace_self_type(result, target))
                    .as_ref(),
                concrete_result.as_ref(),
            )
        {
            return Err(format!(
                "impl method `{trait_name}::{name}` for `{target}` does not match its exact trait signature"
            ));
        }

        let parameter_logical = concrete_parameters
            .iter()
            .map(|parameter| type_to_logical(&parameter.param_type, registry))
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| {
                format!(
                    "impl method `{trait_name}::{name}` for `{target}` has an unsupported parameter type"
                )
            })?;
        let result_logical = concrete_result
            .as_ref()
            .and_then(|result| type_to_logical(result, registry))
            .unwrap_or(LogicalType::Void);
        let helper = private_impl_name(
            trait_name,
            target,
            name,
            &parameter_logical,
            &result_logical,
        );
        let mut concrete_body = body.clone();
        replace_self_types_in_block(&mut concrete_body, target);
        let function = AstNode::Statement(Statement::Function {
            name: helper.clone(),
            parameters: concrete_parameters,
            return_type: concrete_result,
            body: concrete_body,
            type_params: Vec::new(),
            trait_bounds: Vec::new(),
        });
        observed.insert(name.clone(), ());
        implementations.insert(
            (trait_name.to_string(), target.to_string(), name.clone()),
            TraitImplContract { helper, function },
        );
    }
    for method in contract.methods.keys() {
        if !observed.contains_key(method) {
            return Err(format!(
                "impl {trait_name} for {target} is missing required method `{method}`"
            ));
        }
    }
    Ok(())
}

fn type_to_logical(ty: &Type, registry: &StructRegistry) -> Option<LogicalType> {
    match ty {
        Type::Reference(inner, false) => Some(LogicalType::ImmutableReference {
            pointee: Box::new(registry.resolve_copy_annotation(inner)?.logical_type),
        }),
        _ => registry
            .resolve_copy_annotation(ty)
            .map(|contract| contract.logical_type),
    }
}

fn replace_self_type(ty: &Type, target: &str) -> Type {
    match ty {
        Type::Named(name) if name == "Self" => Type::Named(target.to_string()),
        Type::Array(element, count) => {
            Type::Array(Box::new(replace_self_type(element, target)), *count)
        }
        Type::Tuple(elements) => Type::Tuple(
            elements
                .iter()
                .map(|element| replace_self_type(element, target))
                .collect(),
        ),
        Type::Reference(inner, mutable) => {
            Type::Reference(Box::new(replace_self_type(inner, target)), *mutable)
        }
        Type::Generic(name, arguments) => Type::Generic(
            name.clone(),
            arguments
                .iter()
                .map(|argument| replace_self_type(argument, target))
                .collect(),
        ),
        Type::Named(_) => ty.clone(),
    }
}

fn same_optional_type(left: Option<&Type>, right: Option<&Type>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => same_type(left, right),
        _ => false,
    }
}

fn same_type(left: &Type, right: &Type) -> bool {
    specialization_types_equal(left, right)
}

fn replace_self_types_in_block(block: &mut Block, target: &str) {
    for statement in &mut block.statements {
        replace_self_types_in_statement(statement, target);
    }
    if let Some(expression) = &mut block.expression {
        replace_self_types_in_expression(expression, target);
    }
}

fn replace_self_types_in_statement(statement: &mut Statement, target: &str) {
    match statement {
        Statement::Let {
            type_annotation,
            value,
            ..
        } => {
            if let Some(ty) = type_annotation {
                *ty = replace_self_type(ty, target);
            }
            if let Some(value) = value {
                replace_self_types_in_expression(value, target);
            }
        }
        Statement::Assignment {
            target: place,
            value,
        } => {
            replace_self_types_in_expression(place, target);
            replace_self_types_in_expression(value, target);
        }
        Statement::Return(value) => {
            if let Some(value) = value {
                replace_self_types_in_expression(value, target);
            }
        }
        Statement::Expression(expression) => replace_self_types_in_expression(expression, target),
        Statement::Block(block) | Statement::Loop { body: block } => {
            replace_self_types_in_block(block, target)
        }
        Statement::If {
            condition,
            then_block,
            else_block,
        } => {
            replace_self_types_in_expression(condition, target);
            replace_self_types_in_block(then_block, target);
            if let Some(otherwise) = else_block {
                replace_self_types_in_statement(otherwise, target);
            }
        }
        Statement::While { condition, body } => {
            replace_self_types_in_expression(condition, target);
            replace_self_types_in_block(body, target);
        }
        Statement::For { iterable, body, .. } => {
            replace_self_types_in_expression(iterable, target);
            replace_self_types_in_block(body, target);
        }
        Statement::Const {
            type_annotation,
            value,
            ..
        } => {
            *type_annotation = replace_self_type(type_annotation, target);
            replace_self_types_in_expression(value, target);
        }
        Statement::Function { .. }
        | Statement::StructDef { .. }
        | Statement::EnumDef { .. }
        | Statement::ImplBlock { .. }
        | Statement::TraitDef { .. }
        | Statement::Break
        | Statement::Continue
        | Statement::ModDecl { .. }
        | Statement::UseImport { .. } => {}
    }
}

fn replace_self_types_in_expression(expression: &mut Expression, target: &str) {
    match expression {
        Expression::Closure { params, body, .. } => {
            for parameter in params {
                parameter.param_type = replace_self_type(&parameter.param_type, target);
            }
            replace_self_types_in_expression(body, target);
        }
        Expression::Binary { left, right, .. }
        | Expression::Comparison { left, right, .. }
        | Expression::Logical { left, right, .. }
        | Expression::IndexAccess {
            object: left,
            index: right,
        } => {
            replace_self_types_in_expression(left, target);
            replace_self_types_in_expression(right, target);
        }
        Expression::FunctionCall { arguments, .. }
        | Expression::Print { arguments, .. }
        | Expression::Println { arguments, .. } => {
            for argument in arguments {
                replace_self_types_in_expression(argument, target);
            }
        }
        Expression::MethodCall {
            object, arguments, ..
        } => {
            replace_self_types_in_expression(object, target);
            for argument in arguments {
                replace_self_types_in_expression(argument, target);
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
        | Expression::Deref(operand) => replace_self_types_in_expression(operand, target),
        Expression::ArrayLiteral(elements) | Expression::TupleLiteral(elements) => {
            for element in elements {
                replace_self_types_in_expression(element, target);
            }
        }
        Expression::ArrayRepeat { value, .. } => replace_self_types_in_expression(value, target),
        Expression::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                replace_self_types_in_expression(value, target);
            }
        }
        Expression::EnumVariant { data, .. } => {
            if let Some(values) = data {
                for value in values {
                    replace_self_types_in_expression(value, target);
                }
            }
        }
        Expression::Match { expr, arms } => {
            replace_self_types_in_expression(expr, target);
            for arm in arms {
                replace_self_types_in_expression(&mut arm.body, target);
            }
        }
        Expression::IntegerLiteral(_)
        | Expression::FloatLiteral(_)
        | Expression::CharacterLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::Identifier(_) => {}
    }
}

fn elaborate_block(
    block: &mut Block,
    function: &str,
    direct_parameters: &BTreeMap<String, String>,
    bounds: &BTreeMap<String, BTreeSet<String>>,
    traits: &BTreeMap<String, TraitContract>,
    used: &mut BTreeSet<(String, String)>,
) -> Result<(), String> {
    for statement in &mut block.statements {
        elaborate_statement(statement, function, direct_parameters, bounds, traits, used)?;
    }
    if let Some(expression) = &mut block.expression {
        elaborate_expression(
            expression,
            function,
            direct_parameters,
            bounds,
            traits,
            used,
        )?;
    }
    Ok(())
}

fn elaborate_statement(
    statement: &mut Statement,
    function: &str,
    direct_parameters: &BTreeMap<String, String>,
    bounds: &BTreeMap<String, BTreeSet<String>>,
    traits: &BTreeMap<String, TraitContract>,
    used: &mut BTreeSet<(String, String)>,
) -> Result<(), String> {
    let mut elaborate = |expression: &mut Expression| {
        elaborate_expression(
            expression,
            function,
            direct_parameters,
            bounds,
            traits,
            used,
        )
    };
    match statement {
        Statement::Let { value, .. } | Statement::Return(value) => {
            if let Some(value) = value {
                elaborate(value)?;
            }
        }
        Statement::Assignment { target, value } => {
            elaborate(target)?;
            elaborate(value)?;
        }
        Statement::Expression(expression) => elaborate(expression)?,
        Statement::Block(block) | Statement::Loop { body: block } => {
            elaborate_block(block, function, direct_parameters, bounds, traits, used)?
        }
        Statement::If {
            condition,
            then_block,
            else_block,
        } => {
            elaborate(condition)?;
            elaborate_block(
                then_block,
                function,
                direct_parameters,
                bounds,
                traits,
                used,
            )?;
            if let Some(otherwise) = else_block {
                elaborate_statement(otherwise, function, direct_parameters, bounds, traits, used)?;
            }
        }
        Statement::While { condition, body } => {
            elaborate(condition)?;
            elaborate_block(body, function, direct_parameters, bounds, traits, used)?;
        }
        Statement::For { iterable, body, .. } => {
            elaborate(iterable)?;
            elaborate_block(body, function, direct_parameters, bounds, traits, used)?;
        }
        Statement::Const { value, .. } => elaborate(value)?,
        Statement::Function { .. }
        | Statement::StructDef { .. }
        | Statement::EnumDef { .. }
        | Statement::ImplBlock { .. }
        | Statement::TraitDef { .. }
        | Statement::Break
        | Statement::Continue
        | Statement::ModDecl { .. }
        | Statement::UseImport { .. } => {}
    }
    Ok(())
}

fn elaborate_expression(
    expression: &mut Expression,
    function: &str,
    direct_parameters: &BTreeMap<String, String>,
    bounds: &BTreeMap<String, BTreeSet<String>>,
    traits: &BTreeMap<String, TraitContract>,
    used: &mut BTreeSet<(String, String)>,
) -> Result<(), String> {
    if let Expression::MethodCall {
        object,
        method,
        arguments,
    } = expression
    {
        let Expression::Identifier(receiver) = object.as_ref() else {
            return Err(format!(
                "generic function `{function}` trait dispatch requires a direct bounded-parameter receiver"
            ));
        };
        let Some(type_parameter) = direct_parameters.get(receiver) else {
            return Err(format!(
                "generic function `{function}` method `{method}` is not justified by a direct bounded parameter"
            ));
        };
        let candidates = bounds
            .get(type_parameter)
            .into_iter()
            .flatten()
            .filter(|trait_name| {
                traits
                    .get(*trait_name)
                    .is_some_and(|contract| contract.methods.contains_key(method))
            })
            .cloned()
            .collect::<Vec<_>>();
        if candidates.len() != 1 {
            return Err(format!(
                "generic function `{function}` method `{method}` is not uniquely supplied by bounds on `{type_parameter}`"
            ));
        }
        let trait_name = &candidates[0];
        let method_contract = traits
            .get(trait_name)
            .and_then(|contract| contract.methods.get(method))
            .expect("unique bound method was proven");
        if arguments.len() + 1 != method_contract.parameters.len() {
            return Err(format!(
                "trait method `{trait_name}::{method}` expects {} nonreceiver argument(s), actual {}",
                method_contract.parameters.len() - 1,
                arguments.len()
            ));
        }
        for argument in arguments.iter_mut() {
            elaborate_expression(argument, function, direct_parameters, bounds, traits, used)?;
        }
        let mut lowered_arguments = Vec::with_capacity(arguments.len() + 1);
        lowered_arguments.push(Expression::Borrow {
            expr: object.clone(),
            mutable: false,
        });
        lowered_arguments.append(arguments);
        *expression = Expression::FunctionCall {
            name: call_marker_name(trait_name, method, type_parameter),
            arguments: lowered_arguments,
        };
        used.insert((type_parameter.clone(), trait_name.clone()));
        return Ok(());
    }

    let mut elaborate = |expression: &mut Expression| {
        elaborate_expression(
            expression,
            function,
            direct_parameters,
            bounds,
            traits,
            used,
        )
    };
    match expression {
        Expression::Binary { left, right, .. }
        | Expression::Comparison { left, right, .. }
        | Expression::Logical { left, right, .. }
        | Expression::IndexAccess {
            object: left,
            index: right,
        } => {
            elaborate(left)?;
            elaborate(right)?;
        }
        Expression::FunctionCall { arguments, .. }
        | Expression::Print { arguments, .. }
        | Expression::Println { arguments, .. } => {
            for argument in arguments {
                elaborate(argument)?;
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
        | Expression::Deref(operand) => elaborate(operand)?,
        Expression::ArrayLiteral(elements) | Expression::TupleLiteral(elements) => {
            for element in elements {
                elaborate(element)?;
            }
        }
        Expression::ArrayRepeat { value, .. } => elaborate(value)?,
        Expression::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                elaborate(value)?;
            }
        }
        Expression::EnumVariant { data, .. } => {
            if let Some(values) = data {
                for value in values {
                    elaborate(value)?;
                }
            }
        }
        Expression::Match { expr, arms } => {
            elaborate(expr)?;
            for arm in arms {
                elaborate(&mut arm.body)?;
            }
        }
        Expression::Closure { body, .. } => elaborate(body)?,
        Expression::MethodCall { .. }
        | Expression::IntegerLiteral(_)
        | Expression::FloatLiteral(_)
        | Expression::CharacterLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::Identifier(_) => {}
    }
    Ok(())
}

fn finalize_block(
    block: &mut Block,
    function: &str,
    substitutions: &BTreeMap<String, Type>,
    implementations: &BTreeMap<(String, String, String), TraitImplContract>,
) -> Result<(), String> {
    for statement in &mut block.statements {
        finalize_statement(statement, function, substitutions, implementations)?;
    }
    if let Some(expression) = &mut block.expression {
        finalize_expression(expression, function, substitutions, implementations)?;
    }
    Ok(())
}

fn finalize_statement(
    statement: &mut Statement,
    function: &str,
    substitutions: &BTreeMap<String, Type>,
    implementations: &BTreeMap<(String, String, String), TraitImplContract>,
) -> Result<(), String> {
    let finalize = |expression: &mut Expression| {
        finalize_expression(expression, function, substitutions, implementations)
    };
    match statement {
        Statement::Let { value, .. } | Statement::Return(value) => {
            if let Some(value) = value {
                finalize(value)?;
            }
        }
        Statement::Assignment { target, value } => {
            finalize(target)?;
            finalize(value)?;
        }
        Statement::Expression(expression) => finalize(expression)?,
        Statement::Block(block) | Statement::Loop { body: block } => {
            finalize_block(block, function, substitutions, implementations)?
        }
        Statement::If {
            condition,
            then_block,
            else_block,
        } => {
            finalize(condition)?;
            finalize_block(then_block, function, substitutions, implementations)?;
            if let Some(otherwise) = else_block {
                finalize_statement(otherwise, function, substitutions, implementations)?;
            }
        }
        Statement::While { condition, body } => {
            finalize(condition)?;
            finalize_block(body, function, substitutions, implementations)?;
        }
        Statement::For { iterable, body, .. } => {
            finalize(iterable)?;
            finalize_block(body, function, substitutions, implementations)?;
        }
        Statement::Const { value, .. } => finalize(value)?,
        Statement::Function { .. }
        | Statement::StructDef { .. }
        | Statement::EnumDef { .. }
        | Statement::ImplBlock { .. }
        | Statement::TraitDef { .. }
        | Statement::Break
        | Statement::Continue
        | Statement::ModDecl { .. }
        | Statement::UseImport { .. } => {}
    }
    Ok(())
}

fn finalize_expression(
    expression: &mut Expression,
    function: &str,
    substitutions: &BTreeMap<String, Type>,
    implementations: &BTreeMap<(String, String, String), TraitImplContract>,
) -> Result<(), String> {
    if let Expression::FunctionCall { name, arguments } = expression {
        if let Some(marker) = decode_call_marker(name) {
            let target = substitutions
                .get(&marker.type_parameter)
                .and_then(|ty| match ty {
                    Type::Named(name) => Some(name),
                    _ => None,
                })
                .ok_or_else(|| {
                    format!(
                        "generic function `{function}` trait dispatch requires a concrete CopyData struct for `{}`",
                        marker.type_parameter
                    )
                })?;
            let implementation = implementations
                .get(&(marker.trait_name.clone(), target.clone(), marker.method.clone()))
                .ok_or_else(|| {
                    format!(
                        "type `{target}` does not implement trait `{}` required by generic function `{function}`",
                        marker.trait_name
                    )
                })?;
            *name = implementation.helper.clone();
        }
        for argument in arguments {
            finalize_expression(argument, function, substitutions, implementations)?;
        }
        return Ok(());
    }

    let finalize = |expression: &mut Expression| {
        finalize_expression(expression, function, substitutions, implementations)
    };
    match expression {
        Expression::Binary { left, right, .. }
        | Expression::Comparison { left, right, .. }
        | Expression::Logical { left, right, .. }
        | Expression::IndexAccess {
            object: left,
            index: right,
        } => {
            finalize(left)?;
            finalize(right)?;
        }
        Expression::MethodCall {
            object, arguments, ..
        } => {
            finalize(object)?;
            for argument in arguments {
                finalize(argument)?;
            }
        }
        Expression::Print { arguments, .. } | Expression::Println { arguments, .. } => {
            for argument in arguments {
                finalize(argument)?;
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
        | Expression::Deref(operand) => finalize(operand)?,
        Expression::ArrayLiteral(elements) | Expression::TupleLiteral(elements) => {
            for element in elements {
                finalize(element)?;
            }
        }
        Expression::ArrayRepeat { value, .. } => finalize(value)?,
        Expression::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                finalize(value)?;
            }
        }
        Expression::EnumVariant { data, .. } => {
            if let Some(values) = data {
                for value in values {
                    finalize(value)?;
                }
            }
        }
        Expression::Match { expr, arms } => {
            finalize(expr)?;
            for arm in arms {
                finalize(&mut arm.body)?;
            }
        }
        Expression::Closure { body, .. } => finalize(body)?,
        Expression::FunctionCall { .. }
        | Expression::IntegerLiteral(_)
        | Expression::FloatLiteral(_)
        | Expression::CharacterLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::Identifier(_) => {}
    }
    Ok(())
}

pub(crate) fn is_trait_call_marker(name: &str) -> bool {
    decode_call_marker(name).is_some()
}

fn call_marker_name(trait_name: &str, method: &str, type_parameter: &str) -> String {
    private_identity(
        PRIVATE_TRAIT_CALL_PREFIX,
        &[trait_name, method, type_parameter],
    )
}

fn decode_call_marker(name: &str) -> Option<CallMarker> {
    let mut parts = decode_private_identity(PRIVATE_TRAIT_CALL_PREFIX, name, 3)?.into_iter();
    let trait_name = parts.next()?;
    let method = parts.next()?;
    let type_parameter = parts.next()?;
    if !valid_source_symbol(&trait_name)
        || !valid_source_symbol(&method)
        || !valid_source_symbol(&type_parameter)
    {
        return None;
    }
    Some(CallMarker {
        trait_name,
        method,
        type_parameter,
    })
}

fn private_impl_name(
    trait_name: &str,
    target: &str,
    method: &str,
    parameters: &[LogicalType],
    result: &LogicalType,
) -> String {
    let signature = logical_signature_key(parameters, result);
    private_identity(
        PRIVATE_TRAIT_IMPL_PREFIX,
        &[trait_name, target, method, &signature],
    )
}

fn decode_impl_name(name: &str) -> Option<(String, String, String, String)> {
    let mut parts = decode_private_identity(PRIVATE_TRAIT_IMPL_PREFIX, name, 4)?.into_iter();
    let trait_name = parts.next()?;
    let target = parts.next()?;
    let method = parts.next()?;
    let signature = parts.next()?;
    if !valid_source_symbol(&trait_name)
        || !valid_source_symbol(&target)
        || !valid_source_symbol(&method)
        || signature.is_empty()
    {
        return None;
    }
    Some((trait_name, target, method, signature))
}

pub(crate) fn valid_private_trait_impl_signature(
    name: &str,
    parameters: &[LogicalType],
    result: &LogicalType,
) -> bool {
    if !name.starts_with(PRIVATE_TRAIT_IMPL_PREFIX) {
        return true;
    }
    decode_impl_name(name).is_some_and(|(_, target, _, encoded)| {
        let exact_receiver = matches!(
            parameters.first(),
            Some(LogicalType::ImmutableReference { pointee })
                if matches!(pointee.as_ref(), LogicalType::Struct { name, .. } if name == &target)
        );
        exact_receiver && encoded == logical_signature_key(parameters, result)
    })
}

pub(crate) fn valid_trait_aware_function_symbol(
    name: &str,
    valid_source: fn(&str) -> bool,
) -> bool {
    if name.starts_with(PRIVATE_TRAIT_IMPL_PREFIX) {
        decode_impl_name(name).is_some()
    } else if name.starts_with(PRIVATE_TRAIT_CALL_PREFIX) {
        false
    } else {
        valid_source(name)
    }
}

pub(crate) fn private_trait_impl_llvm_symbol(name: &str) -> Option<String> {
    decode_impl_name(name).map(|(trait_name, target, method, _)| {
        format!("\"aero.trait.{trait_name}.for.{target}.{method}\"")
    })
}

fn reject_source_private_symbols(ast: &[AstNode]) -> Result<(), String> {
    for node in ast {
        if let AstNode::Statement(Statement::Function { name, .. }) = node {
            if name.starts_with(PRIVATE_TRAIT_CALL_PREFIX) {
                return Err("trait-dispatch call markers are compiler-private".to_string());
            }
            if name.starts_with(PRIVATE_TRAIT_IMPL_PREFIX) && decode_impl_name(name).is_none() {
                return Err("invalid compiler-private trait implementation identity".to_string());
            }
        }
    }
    Ok(())
}
