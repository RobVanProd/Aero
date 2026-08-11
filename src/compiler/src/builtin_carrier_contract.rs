use crate::ast::{
    AstNode, Block, Expression, Pattern, Statement, Type, VariantDecl, VariantDeclKind,
};
use crate::struct_contract::StructRegistry;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const PRIVATE_CARRIER_PREFIX: &str = "__aero$carrier$";

const MISSING_CONTEXT_SUFFIX: &str = "requires an exact expected Option<T> or Result<T, E> type; missing type arguments are never inferred by default";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CarrierFamily {
    Option,
    Result,
}

impl CarrierFamily {
    fn source_name(self) -> &'static str {
        match self {
            Self::Option => "Option",
            Self::Result => "Result",
        }
    }

    fn expected_arity(self) -> usize {
        match self {
            Self::Option => 1,
            Self::Result => 2,
        }
    }
}

#[derive(Debug, Clone)]
struct CarrierContract {
    family: CarrierFamily,
    private_name: String,
    canonical: String,
    arguments: Vec<Type>,
}

impl CarrierContract {
    fn definition(&self) -> AstNode {
        let variants = match self.family {
            CarrierFamily::Option => vec![
                VariantDecl {
                    name: "Some".to_string(),
                    kind: VariantDeclKind::Tuple(vec![self.arguments[0].clone()]),
                },
                VariantDecl {
                    name: "None".to_string(),
                    kind: VariantDeclKind::Unit,
                },
            ],
            CarrierFamily::Result => vec![
                VariantDecl {
                    name: "Ok".to_string(),
                    kind: VariantDeclKind::Tuple(vec![self.arguments[0].clone()]),
                },
                VariantDecl {
                    name: "Err".to_string(),
                    kind: VariantDeclKind::Tuple(vec![self.arguments[1].clone()]),
                },
            ],
        };
        AstNode::Statement(Statement::EnumDef {
            name: self.private_name.clone(),
            variants,
            type_params: Vec::new(),
        })
    }

    fn expected_payload<'a>(&'a self, variant: &str) -> Option<Option<&'a Type>> {
        match (self.family, variant) {
            (CarrierFamily::Option, "Some") => Some(Some(&self.arguments[0])),
            (CarrierFamily::Option, "None") => Some(None),
            (CarrierFamily::Result, "Ok") => Some(Some(&self.arguments[0])),
            (CarrierFamily::Result, "Err") => Some(Some(&self.arguments[1])),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct FunctionCarrierSignature {
    parameters: Vec<Option<String>>,
    result: Option<String>,
}

#[derive(Debug, Default)]
struct CarrierScopes {
    scopes: Vec<BTreeMap<String, Option<String>>>,
}

impl CarrierScopes {
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
            .expect("built-in carrier scope stack remains balanced");
    }

    fn insert(&mut self, name: String, carrier: Option<String>) {
        self.scopes
            .last_mut()
            .expect("built-in carrier scope stack is nonempty")
            .insert(name, carrier);
    }

    fn get(&self, name: &str) -> Option<&str> {
        for scope in self.scopes.iter().rev() {
            if let Some(carrier) = scope.get(name) {
                return carrier.as_deref();
            }
        }
        None
    }
}

struct BuiltinCarrierNormalizer {
    structs: StructRegistry,
    carriers: BTreeMap<String, CarrierContract>,
    functions: BTreeMap<String, Option<FunctionCarrierSignature>>,
}

pub(crate) fn missing_carrier_context_diagnostic(constructor: &str) -> String {
    format!("built-in carrier constructor `{constructor}` {MISSING_CONTEXT_SUFFIX}")
}

pub(crate) fn unnormalized_carrier_diagnostic(family: &str, variant: &str) -> String {
    format!(
        "built-in carrier constructor `{variant}` for `{family}` escaped the shared contextual normalization boundary"
    )
}

pub(crate) fn private_carrier_source_name(name: &str) -> Option<String> {
    let encoded = name.strip_prefix(PRIVATE_CARRIER_PREFIX)?;
    if encoded.is_empty() || !encoded.len().is_multiple_of(2) {
        return None;
    }
    let mut bytes = Vec::with_capacity(encoded.len() / 2);
    for index in (0..encoded.len()).step_by(2) {
        bytes.push(u8::from_str_radix(&encoded[index..index + 2], 16).ok()?);
    }
    let source = String::from_utf8(bytes).ok()?;
    if (source.starts_with("Option<") || source.starts_with("Result<")) && source.ends_with('>') {
        Some(source)
    } else {
        None
    }
}

pub(crate) fn valid_carrier_aware_enum_symbol(
    name: &str,
    valid_source_symbol: fn(&str) -> bool,
) -> bool {
    if name.starts_with(PRIVATE_CARRIER_PREFIX) {
        private_carrier_source_name(name).is_some()
    } else {
        valid_source_symbol(name)
    }
}

pub(crate) fn normalize_builtin_carriers(ast: Vec<AstNode>) -> Result<Vec<AstNode>, String> {
    let mut retained = Vec::with_capacity(ast.len());
    let mut private_definitions = Vec::new();
    for node in ast {
        match &node {
            AstNode::Statement(Statement::EnumDef { name, .. })
                if name.starts_with(PRIVATE_CARRIER_PREFIX) =>
            {
                private_definitions.push(node);
            }
            _ => retained.push(node),
        }
    }

    let structs = StructRegistry::from_top_level_ast(&retained);
    let mut normalizer = BuiltinCarrierNormalizer {
        structs,
        carriers: BTreeMap::new(),
        functions: BTreeMap::new(),
    };

    for definition in private_definitions {
        normalizer.register_existing_private_definition(definition)?;
    }
    normalizer.reject_reserved_source_declarations(&retained)?;
    normalizer.prepare_function_signatures(&mut retained)?;
    normalizer.normalize_top_level(&mut retained)?;

    let mut normalized = normalizer
        .carriers
        .values()
        .map(CarrierContract::definition)
        .collect::<Vec<_>>();
    normalized.extend(retained);
    Ok(normalized)
}

impl BuiltinCarrierNormalizer {
    fn reject_reserved_source_declarations(&self, ast: &[AstNode]) -> Result<(), String> {
        for node in ast {
            match node {
                AstNode::Statement(Statement::EnumDef { name, .. })
                    if matches!(name.as_str(), "Option" | "Result") =>
                {
                    return Err(format!(
                        "enum name `{name}` is reserved for Aero's built-in algebraic carrier"
                    ));
                }
                AstNode::Statement(Statement::StructDef { fields, .. }) => {
                    for field in fields {
                        self.reject_nested_carrier_annotation(&field.field_type, "struct fields")?;
                    }
                }
                AstNode::Statement(Statement::EnumDef { variants, .. }) => {
                    for variant in variants {
                        match &variant.kind {
                            VariantDeclKind::Tuple(fields) => {
                                for field in fields {
                                    self.reject_nested_carrier_annotation(
                                        field,
                                        "user-defined enum payloads",
                                    )?;
                                }
                            }
                            VariantDeclKind::Struct(fields) => {
                                for field in fields {
                                    self.reject_nested_carrier_annotation(
                                        &field.field_type,
                                        "user-defined enum payloads",
                                    )?;
                                }
                            }
                            VariantDeclKind::Unit => {}
                        }
                    }
                }
                AstNode::Statement(Statement::TraitDef { methods, .. }) => {
                    for method in methods {
                        for parameter in &method.parameters {
                            self.reject_nested_carrier_annotation(
                                &parameter.param_type,
                                "trait signatures",
                            )?;
                        }
                        if let Some(result) = &method.return_type {
                            self.reject_nested_carrier_annotation(result, "trait signatures")?;
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn prepare_function_signatures(&mut self, ast: &mut [AstNode]) -> Result<(), String> {
        let mut duplicates = BTreeSet::new();
        for node in ast {
            let AstNode::Statement(Statement::Function {
                name,
                parameters,
                return_type,
                type_params,
                ..
            }) = node
            else {
                continue;
            };

            let mut parameter_carriers = Vec::with_capacity(parameters.len());
            let mut mentions_carrier = false;
            for parameter in parameters {
                let carrier = self.normalize_direct_annotation(
                    &mut parameter.param_type,
                    "function parameters",
                )?;
                mentions_carrier |= carrier.is_some();
                parameter_carriers.push(carrier);
            }
            let result_carrier = match return_type {
                Some(result) => self.normalize_direct_annotation(result, "function results")?,
                None => None,
            };
            mentions_carrier |= result_carrier.is_some();
            if mentions_carrier && !type_params.is_empty() {
                return Err(format!(
                    "generic function `{name}` cannot use built-in Option/Result carriers in CAP-003"
                ));
            }
            if name == "main" && mentions_carrier {
                return Err(
                    "process entry `main` cannot transport a built-in Option/Result carrier"
                        .to_string(),
                );
            }

            let signature = FunctionCarrierSignature {
                parameters: parameter_carriers,
                result: result_carrier,
            };
            if self.functions.contains_key(name) {
                duplicates.insert(name.clone());
            } else {
                self.functions.insert(name.clone(), Some(signature));
            }
        }
        for duplicate in duplicates {
            self.functions.insert(duplicate, None);
        }
        Ok(())
    }

    fn normalize_top_level(&mut self, ast: &mut [AstNode]) -> Result<(), String> {
        for node in ast {
            match node {
                AstNode::Statement(Statement::Function {
                    name,
                    parameters,
                    body,
                    ..
                }) => {
                    let signature = self.functions.get(name).and_then(Clone::clone).unwrap_or(
                        FunctionCarrierSignature {
                            parameters: vec![None; parameters.len()],
                            result: None,
                        },
                    );
                    let mut scopes = CarrierScopes::new();
                    for (parameter, carrier) in parameters.iter().zip(signature.parameters.iter()) {
                        scopes.insert(parameter.name.clone(), carrier.clone());
                    }
                    self.normalize_block(body, &mut scopes, signature.result.as_deref())?;
                }
                AstNode::Statement(statement) => {
                    let mut scopes = CarrierScopes::new();
                    self.normalize_statement(statement, &mut scopes, None)?;
                }
                AstNode::Expression(expression) => {
                    let mut scopes = CarrierScopes::new();
                    self.normalize_expression(expression, &mut scopes, None)?;
                }
            }
        }
        Ok(())
    }

    fn normalize_block(
        &mut self,
        block: &mut Block,
        scopes: &mut CarrierScopes,
        return_carrier: Option<&str>,
    ) -> Result<(), String> {
        scopes.push();
        for statement in &mut block.statements {
            self.normalize_statement(statement, scopes, return_carrier)?;
        }
        if let Some(expression) = &mut block.expression {
            self.normalize_expression(expression, scopes, return_carrier)?;
        }
        scopes.pop();
        Ok(())
    }

    fn normalize_statement(
        &mut self,
        statement: &mut Statement,
        scopes: &mut CarrierScopes,
        return_carrier: Option<&str>,
    ) -> Result<(), String> {
        match statement {
            Statement::Const {
                type_annotation,
                value,
                ..
            } => {
                if contains_source_carrier(type_annotation)
                    || private_carrier_name(type_annotation).is_some()
                {
                    return Err(
                        "built-in Option/Result carriers are not admitted in constants".to_string(),
                    );
                }
                self.normalize_expression(value, scopes, None)
            }
            Statement::Let {
                name,
                type_annotation,
                value,
                ..
            } => {
                let expected = match type_annotation {
                    Some(annotation) => {
                        self.normalize_direct_annotation(annotation, "local bindings")?
                    }
                    None => None,
                };
                if let Some(value) = value {
                    self.normalize_expression(value, scopes, expected.as_deref())?;
                }
                let inferred = value
                    .as_ref()
                    .and_then(|value| self.expression_carrier(value, scopes));
                scopes.insert(name.clone(), expected.or(inferred));
                Ok(())
            }
            Statement::Assignment { target, value } => {
                let expected = match target {
                    Expression::Identifier(name) => scopes.get(name).map(str::to_string),
                    _ => None,
                };
                if expected.is_none() {
                    self.normalize_expression(target, scopes, None)?;
                }
                self.normalize_expression(value, scopes, expected.as_deref())
            }
            Statement::Return(value) => match value {
                Some(value) => self.normalize_expression(value, scopes, return_carrier),
                None => Ok(()),
            },
            Statement::Expression(expression) => {
                self.normalize_expression(expression, scopes, None)
            }
            Statement::Block(block) => self.normalize_block(block, scopes, return_carrier),
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.normalize_expression(condition, scopes, None)?;
                self.normalize_block(then_block, scopes, return_carrier)?;
                if let Some(otherwise) = else_block {
                    scopes.push();
                    self.normalize_statement(otherwise, scopes, return_carrier)?;
                    scopes.pop();
                }
                Ok(())
            }
            Statement::While { condition, body } => {
                self.normalize_expression(condition, scopes, None)?;
                self.normalize_block(body, scopes, return_carrier)
            }
            Statement::For {
                variable,
                iterable,
                body,
            } => {
                self.normalize_expression(iterable, scopes, None)?;
                scopes.push();
                scopes.insert(variable.clone(), None);
                self.normalize_block(body, scopes, return_carrier)?;
                scopes.pop();
                Ok(())
            }
            Statement::Loop { body } => self.normalize_block(body, scopes, return_carrier),
            Statement::ImplBlock { methods, .. } => {
                for method in methods {
                    if let Statement::Function {
                        parameters,
                        return_type,
                        body,
                        ..
                    } = method
                    {
                        if parameters
                            .iter()
                            .any(|parameter| contains_source_carrier(&parameter.param_type))
                            || return_type.as_ref().is_some_and(contains_source_carrier)
                        {
                            return Err(
                                "built-in Option/Result carriers are not admitted in impl methods"
                                    .to_string(),
                            );
                        }
                        let mut method_scopes = CarrierScopes::new();
                        for parameter in parameters {
                            method_scopes.insert(parameter.name.clone(), None);
                        }
                        self.normalize_block(body, &mut method_scopes, None)?;
                    }
                }
                Ok(())
            }
            Statement::TraitDef { methods, .. } => {
                for method in methods {
                    if let Some(body) = &mut method.body {
                        let mut method_scopes = CarrierScopes::new();
                        for parameter in &method.parameters {
                            method_scopes.insert(parameter.name.clone(), None);
                        }
                        self.normalize_block(body, &mut method_scopes, None)?;
                    }
                }
                Ok(())
            }
            Statement::Function { .. }
            | Statement::Break
            | Statement::Continue
            | Statement::StructDef { .. }
            | Statement::EnumDef { .. }
            | Statement::ModDecl { .. }
            | Statement::UseImport { .. } => Ok(()),
        }
    }

    fn normalize_expression(
        &mut self,
        expression: &mut Expression,
        scopes: &mut CarrierScopes,
        expected: Option<&str>,
    ) -> Result<(), String> {
        match expression {
            Expression::EnumVariant {
                enum_name,
                variant,
                data,
            } if matches!(enum_name.as_str(), "Option" | "Result")
                || self.carriers.contains_key(enum_name) =>
            {
                let constructor = variant.clone();
                let Some(expected) = expected else {
                    return Err(missing_carrier_context_diagnostic(&constructor));
                };
                let contract = self
                    .carriers
                    .get(expected)
                    .cloned()
                    .ok_or_else(|| missing_carrier_context_diagnostic(&constructor))?;
                if enum_name != contract.family.source_name() && enum_name != &contract.private_name
                {
                    if matches!(enum_name.as_str(), "Option" | "Result") {
                        return Err(format!(
                            "built-in constructor `{constructor}` belongs to {enum_name}, but the expected type is {}",
                            contract.canonical
                        ));
                    }
                    return Err(format!(
                        "built-in constructor `{constructor}` does not belong to the expected type {}",
                        contract.canonical
                    ));
                }
                let Some(payload) = contract.expected_payload(&constructor) else {
                    return Err(format!(
                        "unknown built-in {} constructor `{constructor}`",
                        contract.family.source_name()
                    ));
                };
                match (payload, data.as_mut()) {
                    (None, None) => {}
                    (None, Some(fields)) => {
                        return Err(format!(
                            "built-in constructor `{constructor}` accepts no payload, actual {}",
                            fields.len()
                        ));
                    }
                    (Some(_), Some(fields)) if fields.len() == 1 => {
                        self.normalize_expression(&mut fields[0], scopes, None)?;
                        if self.expression_carrier(&fields[0], scopes).is_some() {
                            return Err(format!(
                                "built-in constructor `{constructor}` payload must be recursive finite CopyData, not another carrier"
                            ));
                        }
                    }
                    (Some(_), Some(fields)) => {
                        return Err(format!(
                            "built-in constructor `{constructor}` requires one payload, actual {}",
                            fields.len()
                        ));
                    }
                    (Some(_), None) => {
                        return Err(format!(
                            "built-in constructor `{constructor}` requires one payload"
                        ));
                    }
                }
                *enum_name = contract.private_name;
                Ok(())
            }
            Expression::EnumVariant { data, .. } => {
                if let Some(fields) = data {
                    for field in fields {
                        self.normalize_expression(field, scopes, None)?;
                        self.reject_carrier_value(field, scopes, "user-defined enum payloads")?;
                    }
                }
                Ok(())
            }
            Expression::FunctionCall { name, arguments } => {
                let signature = self.functions.get(name).and_then(Clone::clone);
                for (index, argument) in arguments.iter_mut().enumerate() {
                    let expected = signature
                        .as_ref()
                        .and_then(|signature| signature.parameters.get(index))
                        .and_then(Option::as_deref);
                    self.normalize_expression(argument, scopes, expected)?;
                }
                Ok(())
            }
            Expression::Match { expr, arms } => {
                self.normalize_expression(expr, scopes, None)?;
                let carrier = self.expression_carrier(expr, scopes);
                if let Some(carrier) = carrier {
                    let contract = self
                        .carriers
                        .get(&carrier)
                        .cloned()
                        .expect("recognized private carrier has a contract");
                    for arm in arms {
                        self.normalize_carrier_pattern(&mut arm.pattern, &contract)?;
                        self.normalize_expression(&mut arm.body, scopes, None)?;
                    }
                } else {
                    for arm in arms {
                        if pattern_mentions_builtin_carrier(&arm.pattern) {
                            return Err(
                                "built-in Option/Result Match requires an exact concrete carrier scrutinee"
                                    .to_string(),
                            );
                        }
                        self.normalize_expression(&mut arm.body, scopes, None)?;
                    }
                }
                if expected.is_some() {
                    return Err(
                        "built-in Option/Result carriers are not admitted as Match result values"
                            .to_string(),
                    );
                }
                Ok(())
            }
            Expression::Binary { left, right, .. }
            | Expression::Comparison { left, right, .. }
            | Expression::Logical { left, right, .. } => {
                self.normalize_expression(left, scopes, None)?;
                self.normalize_expression(right, scopes, None)?;
                self.reject_carrier_value(left, scopes, "binary/comparison/logical operands")?;
                self.reject_carrier_value(right, scopes, "binary/comparison/logical operands")
            }
            Expression::MethodCall {
                object, arguments, ..
            } => {
                self.normalize_expression(object, scopes, None)?;
                self.reject_carrier_value(object, scopes, "method receivers")?;
                for argument in arguments {
                    self.normalize_expression(argument, scopes, None)?;
                    self.reject_carrier_value(argument, scopes, "method arguments")?;
                }
                Ok(())
            }
            Expression::Print { arguments, .. } | Expression::Println { arguments, .. } => {
                for argument in arguments {
                    self.normalize_expression(argument, scopes, None)?;
                    self.reject_carrier_value(argument, scopes, "formatted output")?;
                }
                Ok(())
            }
            Expression::Unary { operand, .. } => {
                self.normalize_expression(operand, scopes, None)?;
                self.reject_carrier_value(operand, scopes, "unary operands")
            }
            Expression::ArrayLiteral(elements) | Expression::TupleLiteral(elements) => {
                for element in elements {
                    self.normalize_expression(element, scopes, None)?;
                    self.reject_carrier_value(element, scopes, "aggregate storage")?;
                }
                Ok(())
            }
            Expression::ArrayRepeat { value, .. } => {
                self.normalize_expression(value, scopes, None)?;
                self.reject_carrier_value(value, scopes, "aggregate storage")
            }
            Expression::IndexAccess { object, index } => {
                self.normalize_expression(object, scopes, None)?;
                self.normalize_expression(index, scopes, None)?;
                self.reject_carrier_value(object, scopes, "carrier indexing")?;
                self.reject_carrier_value(index, scopes, "carrier indexes")
            }
            Expression::FieldAccess { object, .. } | Expression::TupleIndex { object, .. } => {
                self.normalize_expression(object, scopes, None)?;
                self.reject_carrier_value(object, scopes, "carrier projection")
            }
            Expression::StructLiteral { fields, .. } => {
                for (_, value) in fields {
                    self.normalize_expression(value, scopes, None)?;
                    self.reject_carrier_value(value, scopes, "struct storage")?;
                }
                Ok(())
            }
            Expression::Borrow { expr, .. } | Expression::Deref(expr) => {
                self.normalize_expression(expr, scopes, None)?;
                self.reject_carrier_value(expr, scopes, "borrowing or dereferencing")
            }
            Expression::Closure { .. } => Ok(()),
            Expression::IntegerLiteral(_)
            | Expression::FloatLiteral(_)
            | Expression::CharacterLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::Identifier(_) => Ok(()),
        }
    }

    fn normalize_carrier_pattern(
        &self,
        pattern: &mut Pattern,
        contract: &CarrierContract,
    ) -> Result<(), String> {
        match pattern {
            Pattern::Enum {
                enum_name,
                variant,
                data,
            } if matches!(enum_name.as_str(), "Option" | "Result") => {
                if enum_name != contract.family.source_name() {
                    return Err(format!(
                        "built-in Match arm `{enum_name}::{variant}` does not belong to {}",
                        contract.canonical
                    ));
                }
                if contract.expected_payload(variant).is_none() {
                    return Err(format!(
                        "unknown built-in {} Match variant `{variant}`",
                        contract.family.source_name()
                    ));
                }
                *enum_name = contract.private_name.clone();
                if let Some(patterns) = data {
                    for pattern in patterns {
                        if pattern_mentions_builtin_carrier(pattern) {
                            return Err(
                                "nested built-in Option/Result patterns are not admitted in CAP-003"
                                    .to_string(),
                            );
                        }
                    }
                }
                Ok(())
            }
            Pattern::Enum { enum_name, .. } if enum_name == &contract.private_name => Ok(()),
            Pattern::Enum {
                enum_name, variant, ..
            } => Err(format!(
                "Match arm `{enum_name}::{variant}` does not belong to {}",
                contract.canonical
            )),
            _ => Ok(()),
        }
    }

    fn reject_carrier_value(
        &self,
        expression: &Expression,
        scopes: &CarrierScopes,
        context: &str,
    ) -> Result<(), String> {
        if self.expression_carrier(expression, scopes).is_some() {
            return Err(format!(
                "built-in Option/Result carriers are not admitted in {context}"
            ));
        }
        Ok(())
    }

    fn expression_carrier(
        &self,
        expression: &Expression,
        scopes: &CarrierScopes,
    ) -> Option<String> {
        match expression {
            Expression::Identifier(name) => scopes.get(name).map(str::to_string),
            Expression::FunctionCall { name, .. } => self
                .functions
                .get(name)
                .and_then(Option::as_ref)
                .and_then(|signature| signature.result.clone()),
            Expression::EnumVariant { enum_name, .. } if self.carriers.contains_key(enum_name) => {
                Some(enum_name.clone())
            }
            _ => None,
        }
    }

    fn normalize_direct_annotation(
        &mut self,
        annotation: &mut Type,
        context: &str,
    ) -> Result<Option<String>, String> {
        if let Some(name) = private_carrier_name(annotation) {
            if self.carriers.contains_key(name) {
                return Ok(Some(name.to_string()));
            }
            return Err(format!(
                "unknown private built-in carrier identity `{name}` in {context}"
            ));
        }

        let Type::Generic(name, arguments) = annotation else {
            self.reject_nested_carrier_annotation(annotation, context)?;
            return Ok(None);
        };
        let family = match name.as_str() {
            "Option" => CarrierFamily::Option,
            "Result" => CarrierFamily::Result,
            _ => {
                self.reject_nested_carrier_annotation(annotation, context)?;
                return Ok(None);
            }
        };
        if arguments.len() != family.expected_arity() {
            return Err(format!(
                "{} requires {} type argument(s), actual {}",
                family.source_name(),
                family.expected_arity(),
                arguments.len()
            ));
        }
        if arguments.iter().any(contains_source_carrier) {
            return Err(
                "nested Option/Result carrier type arguments are not admitted in CAP-003"
                    .to_string(),
            );
        }
        let contract = self.build_contract(family, arguments.clone())?;
        let private_name = contract.private_name.clone();
        self.register_contract(contract)?;
        *annotation = Type::Named(private_name.clone());
        Ok(Some(private_name))
    }

    fn reject_nested_carrier_annotation(
        &self,
        annotation: &Type,
        context: &str,
    ) -> Result<(), String> {
        if contains_source_carrier(annotation) || contains_private_carrier(annotation) {
            return Err(format!(
                "built-in Option/Result carriers are not admitted inside {context}"
            ));
        }
        Ok(())
    }

    fn build_contract(
        &self,
        family: CarrierFamily,
        arguments: Vec<Type>,
    ) -> Result<CarrierContract, String> {
        let mut displays = Vec::with_capacity(arguments.len());
        for argument in &arguments {
            let Some(copy) = self.structs.resolve_copy_annotation(argument) else {
                return Err(format!(
                    "{} type argument `{}` is not admitted recursive finite CopyData",
                    family.source_name(),
                    display_annotation(argument)
                ));
            };
            displays.push(copy.ty.to_string());
        }
        let canonical = match family {
            CarrierFamily::Option => format!("Option<{}>", displays[0]),
            CarrierFamily::Result => format!("Result<{}, {}>", displays[0], displays[1]),
        };
        Ok(CarrierContract {
            family,
            private_name: private_name_for(&canonical),
            canonical,
            arguments,
        })
    }

    fn register_contract(&mut self, contract: CarrierContract) -> Result<(), String> {
        if let Some(existing) = self.carriers.get(&contract.private_name) {
            if existing.canonical != contract.canonical {
                return Err("built-in carrier private identity collision".to_string());
            }
            return Ok(());
        }
        self.carriers
            .insert(contract.private_name.clone(), contract);
        Ok(())
    }

    fn register_existing_private_definition(&mut self, node: AstNode) -> Result<(), String> {
        let AstNode::Statement(Statement::EnumDef {
            name,
            variants,
            type_params,
        }) = node
        else {
            unreachable!("private definition collection retains enum definitions only")
        };
        if !type_params.is_empty() {
            return Err(format!(
                "invalid private built-in carrier definition `{name}`"
            ));
        }
        let (family, arguments) = match variants.as_slice() {
            [
                VariantDecl {
                    name: some,
                    kind: VariantDeclKind::Tuple(some_fields),
                },
                VariantDecl {
                    name: none,
                    kind: VariantDeclKind::Unit,
                },
            ] if some == "Some" && none == "None" && some_fields.len() == 1 => {
                (CarrierFamily::Option, vec![some_fields[0].clone()])
            }
            [
                VariantDecl {
                    name: ok,
                    kind: VariantDeclKind::Tuple(ok_fields),
                },
                VariantDecl {
                    name: err,
                    kind: VariantDeclKind::Tuple(err_fields),
                },
            ] if ok == "Ok" && err == "Err" && ok_fields.len() == 1 && err_fields.len() == 1 => (
                CarrierFamily::Result,
                vec![ok_fields[0].clone(), err_fields[0].clone()],
            ),
            _ => {
                return Err(format!(
                    "invalid private built-in carrier definition `{name}`"
                ));
            }
        };
        let contract = self.build_contract(family, arguments)?;
        if contract.private_name != name {
            return Err(format!(
                "private built-in carrier identity `{name}` does not match its exact schema"
            ));
        }
        self.register_contract(contract)
    }
}

fn private_name_for(canonical: &str) -> String {
    let encoded = canonical
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("{PRIVATE_CARRIER_PREFIX}{encoded}")
}

fn private_carrier_name(annotation: &Type) -> Option<&str> {
    match annotation {
        Type::Named(name) if name.starts_with(PRIVATE_CARRIER_PREFIX) => Some(name),
        _ => None,
    }
}

fn contains_source_carrier(annotation: &Type) -> bool {
    match annotation {
        Type::Generic(name, _) if matches!(name.as_str(), "Option" | "Result") => true,
        Type::Array(element, _) | Type::Reference(element, _) => contains_source_carrier(element),
        Type::Tuple(elements) | Type::Generic(_, elements) => {
            elements.iter().any(contains_source_carrier)
        }
        Type::Named(_) => false,
    }
}

fn contains_private_carrier(annotation: &Type) -> bool {
    match annotation {
        Type::Named(name) => name.starts_with(PRIVATE_CARRIER_PREFIX),
        Type::Array(element, _) | Type::Reference(element, _) => contains_private_carrier(element),
        Type::Tuple(elements) | Type::Generic(_, elements) => {
            elements.iter().any(contains_private_carrier)
        }
    }
}

fn pattern_mentions_builtin_carrier(pattern: &Pattern) -> bool {
    match pattern {
        Pattern::Enum {
            enum_name, data, ..
        } => {
            matches!(enum_name.as_str(), "Option" | "Result")
                || data
                    .as_ref()
                    .is_some_and(|patterns| patterns.iter().any(pattern_mentions_builtin_carrier))
        }
        Pattern::Tuple(patterns) => patterns.iter().any(pattern_mentions_builtin_carrier),
        Pattern::Struct { fields, .. } => fields
            .iter()
            .any(|(_, pattern)| pattern_mentions_builtin_carrier(pattern)),
        Pattern::Wildcard | Pattern::Literal(_) | Pattern::Identifier(_) => false,
    }
}

fn display_annotation(annotation: &Type) -> String {
    match annotation {
        Type::Named(name) => name.clone(),
        Type::Array(element, count) => format!("[{}; {count}]", display_annotation(element)),
        Type::Tuple(elements) => format!(
            "({})",
            elements
                .iter()
                .map(display_annotation)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Type::Reference(inner, mutable) => format!(
            "&{}{}",
            if *mutable { "mut " } else { "" },
            display_annotation(inner)
        ),
        Type::Generic(name, arguments) => format!(
            "{}<{}>",
            name,
            arguments
                .iter()
                .map(display_annotation)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer::try_tokenize_with_locations, parser::parse_with_locations};

    fn parsed(source: &str) -> Vec<AstNode> {
        let tokens = try_tokenize_with_locations(source, None).expect("fixture lexes");
        parse_with_locations(tokens).expect("fixture parses")
    }

    fn private_names(ast: &[AstNode]) -> Vec<String> {
        ast.iter()
            .filter_map(|node| match node {
                AstNode::Statement(Statement::EnumDef { name, .. })
                    if name.starts_with(PRIVATE_CARRIER_PREFIX) =>
                {
                    Some(name.clone())
                }
                _ => None,
            })
            .collect()
    }

    #[test]
    fn concrete_identity_is_deterministic_distinct_and_source_decodable() {
        let normalized = normalize_builtin_carriers(parsed(
            "fn left(value: Option<int>) -> Option<int> { return value; } \
             fn right(value: Result<int, char>) -> Result<int, char> { return value; }",
        ))
        .expect("exact carriers normalize");
        let names = private_names(&normalized);
        assert_eq!(names.len(), 2);
        assert_ne!(names[0], names[1]);
        assert!(
            names
                .iter()
                .all(|name| valid_carrier_aware_enum_symbol(name, |_| false))
        );
        assert!(!valid_carrier_aware_enum_symbol(
            &format!("{}61", PRIVATE_CARRIER_PREFIX),
            |_| true
        ));
        let decoded = names
            .iter()
            .filter_map(|name| private_carrier_source_name(name))
            .collect::<BTreeSet<_>>();
        assert_eq!(
            decoded,
            BTreeSet::from(["Option<int>".to_string(), "Result<int, char>".to_string()])
        );
    }

    #[test]
    fn normalization_is_idempotent() {
        let once = normalize_builtin_carriers(parsed(
            "fn make() -> Option<int> { return Some(3); } fn main() { let value: Option<int> = make(); }",
        ))
        .expect("first normalization succeeds");
        let once_names = private_names(&once);
        let twice = normalize_builtin_carriers(once).expect("second normalization succeeds");
        assert_eq!(private_names(&twice), once_names);
    }

    #[test]
    fn corrupted_private_identity_or_schema_fails_closed() {
        let mut wrong_name = normalize_builtin_carriers(parsed(
            "fn make() -> Option<int> { return None; } fn main() { let value: Option<int> = make(); }",
        ))
        .expect("control normalizes");
        let AstNode::Statement(Statement::EnumDef { name, .. }) = &mut wrong_name[0] else {
            panic!("private definition must be first");
        };
        name.push_str("00");
        let error = normalize_builtin_carriers(wrong_name)
            .expect_err("wrong private identity must be rejected");
        assert!(error.contains("does not match its exact schema"), "{error}");

        let mut wrong_schema = normalize_builtin_carriers(parsed(
            "fn make() -> Result<int, char> { return Ok(2); } fn main() { let value: Result<int, char> = make(); }",
        ))
        .expect("control normalizes");
        let AstNode::Statement(Statement::EnumDef { variants, .. }) = &mut wrong_schema[0] else {
            panic!("private definition must be first");
        };
        let VariantDeclKind::Tuple(fields) = &mut variants[0].kind else {
            panic!("Result::Ok must have a tuple payload");
        };
        fields[0] = Type::Named("float".to_string());
        let error = normalize_builtin_carriers(wrong_schema)
            .expect_err("wrong private schema must be rejected");
        assert!(error.contains("does not match its exact schema"), "{error}");

        let mut missing_context =
            normalize_builtin_carriers(parsed("fn main() { let value: Option<int> = Some(3); }"))
                .expect("control normalizes");
        let AstNode::Statement(Statement::Function { body, .. }) = &mut missing_context[1] else {
            panic!("normalized function follows its private definition");
        };
        let Statement::Let {
            type_annotation, ..
        } = &mut body.statements[0]
        else {
            panic!("control starts with a carrier binding");
        };
        *type_annotation = None;
        let error = normalize_builtin_carriers(missing_context)
            .expect_err("pre-normalized private constructor cannot bypass context admission");
        assert!(error.contains(MISSING_CONTEXT_SUFFIX), "{error}");
    }

    #[test]
    fn source_cannot_spoof_reserved_carrier_families() {
        let error =
            normalize_builtin_carriers(parsed("enum Option { Some(int), None } fn main() { 0 }"))
                .expect_err("source Option declaration must be rejected");
        assert!(error.contains("reserved"), "{error}");
    }
}
