use crate::ast::{
    AstNode, Block, Expression, Pattern, Statement, Type, VariantDecl, VariantDeclKind,
};
use crate::generic_struct_contract::{
    canonical_copydata_type_matches_logical, display_source_type,
};
use crate::ir::{EnumVariantSchema, LogicalType};
use crate::specialization_contract::{
    canonicalize_specialization_type, decode_private_identity, parse_canonical_application,
    parse_canonical_copydata_type_list, private_identity, specialization_types_equal,
    valid_source_symbol,
};
use crate::struct_contract::StructRegistry;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const PRIVATE_GENERIC_ENUM_PREFIX: &str = "__aero$generic_enum$";

#[derive(Debug, Clone)]
struct GenericEnumDefinition {
    parameters: Vec<String>,
    variants: Vec<VariantDecl>,
}

#[derive(Debug, Clone)]
struct GenericEnumContract {
    source_name: String,
    canonical: String,
    private_name: String,
    variants: Vec<VariantDecl>,
}

impl GenericEnumContract {
    fn definition(&self) -> AstNode {
        AstNode::Statement(Statement::EnumDef {
            name: self.private_name.clone(),
            variants: self.variants.clone(),
            type_params: Vec::new(),
            trait_bounds: Vec::new(),
        })
    }

    fn variant_fields(&self, variant: &str) -> Option<Vec<Type>> {
        self.variants
            .iter()
            .find(|candidate| candidate.name == variant)
            .map(|variant| match &variant.kind {
                VariantDeclKind::Unit => Vec::new(),
                VariantDeclKind::Tuple(fields) => fields.clone(),
                VariantDeclKind::Struct(_) => {
                    unreachable!("CAP-006 contracts exclude named-field variants")
                }
            })
    }
}

#[derive(Debug, Clone)]
struct FunctionSignature {
    parameters: Vec<Type>,
    result: Option<Type>,
}

#[derive(Debug, Default)]
struct TypeScopes {
    scopes: Vec<BTreeMap<String, Option<Type>>>,
}

impl TypeScopes {
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
            .expect("generic-enum scope stack remains balanced");
    }

    fn insert(&mut self, name: String, ty: Option<Type>) {
        self.scopes
            .last_mut()
            .expect("generic-enum scope stack is nonempty")
            .insert(name, ty);
    }

    fn get(&self, name: &str) -> Option<Type> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned().flatten())
    }
}

#[derive(Debug)]
struct GenericEnumNormalizer {
    definitions: BTreeMap<String, GenericEnumDefinition>,
    contracts: BTreeMap<String, GenericEnumContract>,
    functions: BTreeMap<String, Option<FunctionSignature>>,
    enum_variants: BTreeMap<(String, String), Option<Vec<Type>>>,
    structs: StructRegistry,
}

impl GenericEnumNormalizer {
    fn new(structs: StructRegistry) -> Self {
        Self {
            definitions: BTreeMap::new(),
            contracts: BTreeMap::new(),
            functions: BTreeMap::new(),
            enum_variants: BTreeMap::new(),
            structs,
        }
    }
}

pub(crate) fn normalize_generic_copydata_enums(ast: Vec<AstNode>) -> Result<Vec<AstNode>, String> {
    let mut retained = Vec::with_capacity(ast.len());
    let mut existing_private = Vec::new();
    for node in ast {
        match &node {
            AstNode::Statement(Statement::EnumDef { name, .. })
                if name.starts_with(PRIVATE_GENERIC_ENUM_PREFIX) =>
            {
                existing_private.push(node);
            }
            _ => retained.push(node),
        }
    }

    let structs = StructRegistry::from_top_level_ast(&retained);
    let mut normalizer = GenericEnumNormalizer::new(structs);
    normalizer.collect_definitions(&retained)?;
    if normalizer.definitions.is_empty() && existing_private.is_empty() {
        return Ok(retained);
    }
    for definition in existing_private {
        normalizer.register_existing_private_definition(definition)?;
    }
    normalizer.normalize_annotations(&mut retained)?;
    normalizer.prepare_context(&retained)?;
    normalizer.normalize_top_level(&mut retained)?;

    retained.retain(|node| {
        !matches!(
            node,
            AstNode::Statement(Statement::EnumDef { name, type_params, .. })
                if !type_params.is_empty() && normalizer.definitions.contains_key(name)
        )
    });
    let mut normalized = normalizer
        .contracts
        .values()
        .map(GenericEnumContract::definition)
        .collect::<Vec<_>>();
    normalized.extend(retained);
    Ok(normalized)
}

impl GenericEnumNormalizer {
    fn collect_definitions(&mut self, ast: &[AstNode]) -> Result<(), String> {
        let mut top_level_names = BTreeMap::<String, usize>::new();
        for node in ast {
            match node {
                AstNode::Statement(Statement::StructDef { name, .. })
                | AstNode::Statement(Statement::EnumDef { name, .. }) => {
                    *top_level_names.entry(name.clone()).or_default() += 1;
                }
                _ => {}
            }
        }

        for node in ast {
            let AstNode::Statement(Statement::EnumDef {
                name,
                variants,
                type_params,
                trait_bounds,
            }) = node
            else {
                continue;
            };
            if name.starts_with(PRIVATE_GENERIC_ENUM_PREFIX) {
                return Err(format!(
                    "source enum name `{name}` uses Aero's reserved generic-enum identity"
                ));
            }
            if type_params.is_empty() {
                continue;
            }
            if top_level_names.get(name).copied() != Some(1) {
                return Err(format!("duplicate generic enum definition `{name}`"));
            }
            if !valid_source_symbol(name)
                || matches!(name.as_str(), "Option" | "Result")
                || variants.is_empty()
            {
                return Err(format!("generic enum `{name}` has an invalid definition"));
            }
            if !trait_bounds.is_empty() {
                return Err(format!(
                    "generic enum `{name}` trait bounds are not admitted in CAP-006"
                ));
            }

            let mut parameters = BTreeSet::new();
            for parameter in type_params {
                if !valid_source_symbol(parameter) || !parameters.insert(parameter.clone()) {
                    return Err(format!(
                        "generic enum `{name}` has duplicate or invalid type parameter `{parameter}`"
                    ));
                }
            }
            let mut variant_names = BTreeSet::new();
            let mut used_parameters = BTreeSet::new();
            for variant in variants {
                if !valid_source_symbol(&variant.name)
                    || !variant_names.insert(variant.name.clone())
                {
                    return Err(format!(
                        "generic enum `{name}` has duplicate or invalid variant `{}`",
                        variant.name
                    ));
                }
                match &variant.kind {
                    VariantDeclKind::Unit => {}
                    VariantDeclKind::Tuple(fields) if !fields.is_empty() => {
                        for field in fields {
                            self.validate_template_type(
                                field,
                                name,
                                &parameters,
                                &mut used_parameters,
                            )?;
                        }
                    }
                    VariantDeclKind::Tuple(_) => {
                        return Err(format!(
                            "generic enum `{name}` variant `{}` cannot use an empty positional field list",
                            variant.name
                        ));
                    }
                    VariantDeclKind::Struct(_) => {
                        return Err(format!(
                            "generic enum `{name}` named-field variants are not admitted in CAP-006"
                        ));
                    }
                }
            }
            if used_parameters != parameters {
                let unused = parameters
                    .difference(&used_parameters)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(format!(
                    "generic enum `{name}` has unused type parameter(s): {unused}"
                ));
            }
            self.definitions.insert(
                name.clone(),
                GenericEnumDefinition {
                    parameters: type_params.clone(),
                    variants: variants.clone(),
                },
            );
        }
        Ok(())
    }

    fn validate_template_type(
        &self,
        ty: &Type,
        enum_name: &str,
        parameters: &BTreeSet<String>,
        used: &mut BTreeSet<String>,
    ) -> Result<(), String> {
        match ty {
            Type::Named(name) if parameters.contains(name) => {
                used.insert(name.clone());
                Ok(())
            }
            Type::Named(name) if name == enum_name => Err(format!(
                "recursive generic enum `{enum_name}` is not admitted in CAP-006"
            )),
            Type::Named(_) if self.structs.resolve_copy_annotation(ty).is_some() => Ok(()),
            Type::Array(element, _) => {
                self.validate_template_type(element, enum_name, parameters, used)
            }
            Type::Tuple(elements) if elements.len() >= 2 => {
                for element in elements {
                    self.validate_template_type(element, enum_name, parameters, used)?;
                }
                Ok(())
            }
            Type::Tuple(_) => Err(format!(
                "generic enum `{enum_name}` requires tuple payloads with at least two elements"
            )),
            Type::Reference(_, _) => Err(format!(
                "generic enum `{enum_name}` payloads must be recursive finite CopyData"
            )),
            Type::Generic(_, _) => Err(format!(
                "nested generic applications in generic enum `{enum_name}` payloads are not admitted in CAP-006"
            )),
            Type::Named(_) => Err(format!(
                "generic enum `{enum_name}` payloads must be recursive finite CopyData"
            )),
        }
    }

    fn normalize_annotations(&mut self, ast: &mut [AstNode]) -> Result<(), String> {
        for node in ast {
            if let AstNode::Statement(statement) = node {
                self.normalize_statement_annotations(statement)?;
            }
        }
        Ok(())
    }

    fn normalize_statement_annotations(&mut self, statement: &mut Statement) -> Result<(), String> {
        match statement {
            Statement::Const {
                type_annotation, ..
            } => self.normalize_type(type_annotation, false, "constant annotations"),
            Statement::Let {
                type_annotation, ..
            } => {
                if let Some(annotation) = type_annotation {
                    self.normalize_type(annotation, true, "binding annotations")?;
                }
                Ok(())
            }
            Statement::Block(block)
            | Statement::Loop { body: block }
            | Statement::While { body: block, .. }
            | Statement::For { body: block, .. } => self.normalize_block_annotations(block),
            Statement::If {
                then_block,
                else_block,
                ..
            } => {
                self.normalize_block_annotations(then_block)?;
                if let Some(otherwise) = else_block {
                    self.normalize_statement_annotations(otherwise)?;
                }
                Ok(())
            }
            Statement::Function {
                name,
                parameters,
                return_type,
                body,
                type_params,
                ..
            } => {
                let mentions = parameters
                    .iter()
                    .any(|parameter| self.type_mentions_generic_enum(&parameter.param_type))
                    || return_type
                        .as_ref()
                        .is_some_and(|result| self.type_mentions_generic_enum(result));
                if mentions && !type_params.is_empty() {
                    return Err(format!(
                        "generic function `{name}` cannot transport an explicit generic enum in CAP-006"
                    ));
                }
                for parameter in parameters {
                    self.normalize_type(&mut parameter.param_type, true, "function parameters")?;
                }
                if let Some(result) = return_type {
                    self.normalize_type(result, true, "function results")?;
                }
                self.normalize_block_annotations(body)
            }
            Statement::StructDef {
                fields,
                type_params,
                ..
            } if type_params.is_empty() => {
                for field in fields {
                    self.normalize_type(&mut field.field_type, false, "struct fields")?;
                }
                Ok(())
            }
            Statement::StructDef { .. } => Ok(()),
            Statement::EnumDef {
                variants,
                type_params,
                ..
            } if type_params.is_empty() => {
                for variant in variants {
                    match &mut variant.kind {
                        VariantDeclKind::Unit => {}
                        VariantDeclKind::Tuple(fields) => {
                            for field in fields {
                                self.normalize_type(field, false, "enum payloads")?;
                            }
                        }
                        VariantDeclKind::Struct(fields) => {
                            for field in fields {
                                self.normalize_type(&mut field.field_type, false, "enum payloads")?;
                            }
                        }
                    }
                }
                Ok(())
            }
            Statement::EnumDef { .. } => Ok(()),
            Statement::ImplBlock { methods, .. } => {
                for method in methods {
                    if statement_mentions_generic_enum(method, &self.definitions, &self.contracts) {
                        return Err(
                            "generic enums are not admitted in impl methods in CAP-006".to_string()
                        );
                    }
                    self.normalize_statement_annotations(method)?;
                }
                Ok(())
            }
            Statement::TraitDef { methods, .. } => {
                for method in methods {
                    if method
                        .parameters
                        .iter()
                        .any(|parameter| self.type_mentions_generic_enum(&parameter.param_type))
                        || method
                            .return_type
                            .as_ref()
                            .is_some_and(|result| self.type_mentions_generic_enum(result))
                    {
                        return Err(
                            "generic enums are not admitted in trait signatures in CAP-006"
                                .to_string(),
                        );
                    }
                    if let Some(body) = &mut method.body {
                        self.normalize_block_annotations(body)?;
                    }
                }
                Ok(())
            }
            Statement::Assignment { .. }
            | Statement::Return(_)
            | Statement::Expression(_)
            | Statement::Break
            | Statement::Continue
            | Statement::ModDecl { .. }
            | Statement::UseImport { .. } => Ok(()),
        }
    }

    fn normalize_block_annotations(&mut self, block: &mut Block) -> Result<(), String> {
        for statement in &mut block.statements {
            self.normalize_statement_annotations(statement)?;
        }
        Ok(())
    }

    fn normalize_type(
        &mut self,
        ty: &mut Type,
        allow_direct: bool,
        context: &str,
    ) -> Result<(), String> {
        match ty {
            Type::Generic(name, arguments) if self.definitions.contains_key(name) => {
                if !allow_direct {
                    return Err(format!(
                        "generic enum applications are not admitted inside {context} in CAP-006"
                    ));
                }
                for argument in arguments.iter() {
                    if contains_reference(argument)
                        || self.type_mentions_generic_enum(argument)
                        || self.structs.resolve_copy_annotation(argument).is_none()
                    {
                        let application = display_application_diagnostic(name, arguments);
                        return Err(format!(
                            "generic enum application `{application}` is not recursive finite CopyData"
                        ));
                    }
                }
                let source_name = name.clone();
                let contract = self.build_contract(&source_name, arguments.clone(), context)?;
                let private_name = contract.private_name.clone();
                self.register_contract(contract)?;
                *ty = Type::Named(private_name);
                Ok(())
            }
            Type::Generic(_, arguments) => {
                for argument in arguments {
                    self.normalize_type(argument, false, "generic type applications")?;
                }
                Ok(())
            }
            Type::Named(name) if self.definitions.contains_key(name) => Err(format!(
                "generic enum `{name}` requires explicit type arguments in {context}"
            )),
            Type::Named(name) if name.starts_with(PRIVATE_GENERIC_ENUM_PREFIX) => {
                if !allow_direct {
                    return Err(format!(
                        "generic enum applications are not admitted inside {context} in CAP-006"
                    ));
                }
                if self.contracts.contains_key(name) {
                    Ok(())
                } else {
                    Err(format!(
                        "unknown private generic-enum identity `{name}` in {context}"
                    ))
                }
            }
            Type::Array(element, _) | Type::Reference(element, _) => {
                self.normalize_type(element, false, context)
            }
            Type::Tuple(elements) => {
                for element in elements {
                    self.normalize_type(element, false, context)?;
                }
                Ok(())
            }
            Type::Named(_) => Ok(()),
        }
    }

    fn build_contract(
        &self,
        source_name: &str,
        arguments: Vec<Type>,
        context: &str,
    ) -> Result<GenericEnumContract, String> {
        let definition = self
            .definitions
            .get(source_name)
            .ok_or_else(|| format!("unknown generic enum `{source_name}` in {context}"))?;
        if arguments.len() != definition.parameters.len() {
            return Err(format!(
                "generic enum `{source_name}` requires {} type argument(s), actual {}",
                definition.parameters.len(),
                arguments.len()
            ));
        }
        let arguments = arguments
            .iter()
            .map(canonicalize_specialization_type)
            .collect::<Vec<_>>();
        let substitutions = definition
            .parameters
            .iter()
            .cloned()
            .zip(arguments.iter().cloned())
            .collect::<BTreeMap<_, _>>();
        let variants = definition
            .variants
            .iter()
            .map(|variant| {
                let kind = match &variant.kind {
                    VariantDeclKind::Unit => VariantDeclKind::Unit,
                    VariantDeclKind::Tuple(fields) => VariantDeclKind::Tuple(
                        fields
                            .iter()
                            .map(|field| substitute_type(field, &substitutions))
                            .collect::<Result<Vec<_>, _>>()?,
                    ),
                    VariantDeclKind::Struct(_) => {
                        return Err(
                            "unsupported generic enum template escaped CAP-006 validation"
                                .to_string(),
                        );
                    }
                };
                Ok(VariantDecl {
                    name: variant.name.clone(),
                    kind,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        for variant in &variants {
            if let VariantDeclKind::Tuple(fields) = &variant.kind {
                for field in fields {
                    if self.structs.resolve_copy_annotation(field).is_none() {
                        let application = display_application(source_name, &arguments)?;
                        return Err(format!(
                            "generic enum application `{application}` is not recursive finite CopyData"
                        ));
                    }
                }
            }
        }
        let canonical = display_application(source_name, &arguments)?;
        Ok(GenericEnumContract {
            source_name: source_name.to_string(),
            private_name: private_name_for(&canonical, &variants)?,
            canonical,
            variants,
        })
    }

    fn register_contract(&mut self, contract: GenericEnumContract) -> Result<(), String> {
        if let Some(existing) = self.contracts.get(&contract.private_name) {
            if existing.canonical != contract.canonical
                || !variants_equal(&existing.variants, &contract.variants)
            {
                return Err("generic enum private identity collision".to_string());
            }
            return Ok(());
        }
        self.contracts
            .insert(contract.private_name.clone(), contract);
        Ok(())
    }

    fn register_existing_private_definition(&mut self, node: AstNode) -> Result<(), String> {
        let AstNode::Statement(Statement::EnumDef {
            name,
            variants,
            type_params,
            trait_bounds,
        }) = node
        else {
            unreachable!("private definition collection retains enum definitions only")
        };
        if !type_params.is_empty() || !trait_bounds.is_empty() {
            return Err(format!("invalid private generic-enum definition `{name}`"));
        }
        let (canonical, encoded_variants) = decode_private_payload(&name)
            .ok_or_else(|| format!("invalid private generic-enum definition `{name}`"))?;
        let (source_name, _arguments) = parse_canonical_application(&canonical)
            .ok_or_else(|| format!("invalid private generic-enum definition `{name}`"))?;
        let decoded = decode_schema(&encoded_variants)
            .ok_or_else(|| format!("invalid private generic-enum definition `{name}`"))?;
        if !decoded_schema_matches_variants(&decoded, &variants, &self.structs)
            || private_name_for(&canonical, &variants).ok().as_deref() != Some(name.as_str())
        {
            return Err(format!("invalid private generic-enum definition `{name}`"));
        }
        self.register_contract(GenericEnumContract {
            source_name,
            canonical,
            private_name: name,
            variants,
        })
    }

    fn prepare_context(&mut self, ast: &[AstNode]) -> Result<(), String> {
        for node in ast {
            match node {
                AstNode::Statement(Statement::Function {
                    name,
                    parameters,
                    return_type,
                    type_params,
                    ..
                }) if type_params.is_empty() => {
                    let signature = FunctionSignature {
                        parameters: parameters
                            .iter()
                            .map(|parameter| parameter.param_type.clone())
                            .collect(),
                        result: return_type.clone(),
                    };
                    match self.functions.entry(name.clone()) {
                        std::collections::btree_map::Entry::Vacant(entry) => {
                            entry.insert(Some(signature));
                        }
                        std::collections::btree_map::Entry::Occupied(mut entry) => {
                            entry.insert(None);
                        }
                    }
                }
                AstNode::Statement(Statement::EnumDef {
                    name,
                    variants,
                    type_params,
                    ..
                }) if type_params.is_empty() => {
                    register_variant_context(&mut self.enum_variants, name, variants);
                }
                _ => {}
            }
        }
        for contract in self.contracts.values() {
            register_variant_context(
                &mut self.enum_variants,
                &contract.private_name,
                &contract.variants,
            );
        }
        Ok(())
    }

    fn normalize_top_level(&self, ast: &mut [AstNode]) -> Result<(), String> {
        for node in ast {
            match node {
                AstNode::Statement(Statement::Function {
                    name,
                    parameters,
                    body,
                    return_type,
                    type_params,
                    ..
                }) => {
                    if !type_params.is_empty()
                        && block_mentions_source_generic_enum(body, &self.definitions)
                    {
                        return Err(format!(
                            "generic function `{name}` cannot construct or inspect a generic enum in CAP-006"
                        ));
                    }
                    let mut scopes = TypeScopes::new();
                    for parameter in parameters {
                        scopes.insert(parameter.name.clone(), Some(parameter.param_type.clone()));
                    }
                    self.normalize_block(body, &mut scopes, return_type.as_ref())?;
                }
                AstNode::Statement(statement) => {
                    let mut scopes = TypeScopes::new();
                    self.normalize_statement(statement, &mut scopes, None)?;
                }
                AstNode::Expression(expression) => {
                    let mut scopes = TypeScopes::new();
                    self.normalize_expression(expression, &mut scopes, None)?;
                }
            }
        }
        Ok(())
    }

    fn normalize_block(
        &self,
        block: &mut Block,
        scopes: &mut TypeScopes,
        result: Option<&Type>,
    ) -> Result<(), String> {
        scopes.push();
        for statement in &mut block.statements {
            self.normalize_statement(statement, scopes, result)?;
        }
        if let Some(expression) = &mut block.expression {
            self.normalize_expression(expression, scopes, result)?;
        }
        scopes.pop();
        Ok(())
    }

    fn normalize_statement(
        &self,
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
                self.normalize_expression(value, scopes, Some(type_annotation))?;
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
                    self.normalize_expression(value, scopes, type_annotation.as_ref())?;
                }
                let inferred = value
                    .as_ref()
                    .and_then(|value| self.expression_type(value, scopes));
                scopes.insert(name.clone(), type_annotation.clone().or(inferred));
                Ok(())
            }
            Statement::Assignment { target, value } => {
                let expected = self.expression_type(target, scopes);
                self.normalize_expression(target, scopes, None)?;
                self.normalize_expression(value, scopes, expected.as_ref())
            }
            Statement::Return(value) => {
                if let Some(value) = value {
                    self.normalize_expression(value, scopes, result)?;
                }
                Ok(())
            }
            Statement::Expression(expression) => {
                self.normalize_expression(expression, scopes, None)
            }
            Statement::Block(block) | Statement::Loop { body: block } => {
                self.normalize_block(block, scopes, result)
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.normalize_expression(condition, scopes, None)?;
                self.normalize_block(then_block, scopes, result)?;
                if let Some(otherwise) = else_block {
                    scopes.push();
                    self.normalize_statement(otherwise, scopes, result)?;
                    scopes.pop();
                }
                Ok(())
            }
            Statement::While { condition, body } => {
                self.normalize_expression(condition, scopes, None)?;
                self.normalize_block(body, scopes, result)
            }
            Statement::For {
                variable,
                iterable,
                body,
            } => {
                self.normalize_expression(iterable, scopes, None)?;
                let element = self
                    .expression_type(iterable, scopes)
                    .and_then(|ty| match ty {
                        Type::Array(element, _) => Some(*element),
                        _ => None,
                    });
                scopes.push();
                scopes.insert(variable.clone(), element);
                self.normalize_block(body, scopes, result)?;
                scopes.pop();
                Ok(())
            }
            Statement::ImplBlock { .. } | Statement::TraitDef { .. } => Ok(()),
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
        &self,
        expression: &mut Expression,
        scopes: &mut TypeScopes,
        expected: Option<&Type>,
    ) -> Result<(), String> {
        match expression {
            Expression::EnumVariant {
                enum_name,
                variant,
                data,
            } => {
                let expected_contract = expected.and_then(|ty| self.contract_for_type(ty));
                let source_definition = self.definitions.contains_key(enum_name);
                let private_contract = self.contracts.get(enum_name);
                let contract = if source_definition {
                    let Some(contract) = expected_contract else {
                        return Err(format!(
                            "generic enum constructor `{enum_name}::{variant}` requires an exact expected {enum_name}<...> type"
                        ));
                    };
                    if contract.source_name != *enum_name {
                        return Err(format!(
                            "generic enum constructor `{enum_name}::{variant}` does not match expected type {}",
                            contract.canonical
                        ));
                    }
                    *enum_name = contract.private_name.clone();
                    Some(contract)
                } else if let Some(contract) = private_contract {
                    if expected_contract
                        .is_some_and(|expected| expected.private_name != contract.private_name)
                    {
                        return Err(format!(
                            "generic enum constructor `{}` does not match expected specialization",
                            contract.canonical
                        ));
                    }
                    Some(contract)
                } else {
                    if let Some(expected) = expected_contract {
                        return Err(format!(
                            "enum constructor `{enum_name}::{variant}` does not match expected type {}",
                            expected.canonical
                        ));
                    }
                    None
                };

                let payloads = if let Some(contract) = contract {
                    let Some(fields) = contract.variant_fields(variant) else {
                        return Err(format!(
                            "generic enum `{}` has no variant `{variant}`",
                            contract.canonical
                        ));
                    };
                    match (fields.is_empty(), data.as_ref()) {
                        (true, None) => {}
                        (true, Some(_)) => {
                            return Err(format!(
                                "generic enum `{}` variant `{variant}` does not accept payload data",
                                contract.canonical
                            ));
                        }
                        (false, None) => {
                            return Err(format!(
                                "generic enum `{}` variant `{variant}` requires {} positional field(s)",
                                contract.canonical,
                                fields.len()
                            ));
                        }
                        (false, Some(values)) if values.len() != fields.len() => {
                            return Err(format!(
                                "generic enum `{}` variant `{variant}` requires {} positional field(s), actual {}",
                                contract.canonical,
                                fields.len(),
                                values.len()
                            ));
                        }
                        (false, Some(_)) => {}
                    }
                    Some(fields)
                } else {
                    self.enum_variants
                        .get(&(enum_name.clone(), variant.clone()))
                        .and_then(Clone::clone)
                };
                if let Some(values) = data {
                    for (index, value) in values.iter_mut().enumerate() {
                        self.normalize_expression(
                            value,
                            scopes,
                            payloads.as_ref().and_then(|fields| fields.get(index)),
                        )?;
                    }
                }
                Ok(())
            }
            Expression::FunctionCall { name, arguments } => {
                let signature = self.functions.get(name).and_then(Clone::clone);
                for (index, argument) in arguments.iter_mut().enumerate() {
                    self.normalize_expression(
                        argument,
                        scopes,
                        signature
                            .as_ref()
                            .and_then(|signature| signature.parameters.get(index)),
                    )?;
                }
                Ok(())
            }
            Expression::Match { expr, arms } => {
                self.normalize_expression(expr, scopes, None)?;
                let scrutinee = self.expression_type(expr, scopes);
                for arm in arms {
                    scopes.push();
                    if let Some(ty) = &scrutinee {
                        for (name, ty) in self.normalize_pattern(&mut arm.pattern, ty)? {
                            scopes.insert(name, Some(ty));
                        }
                    }
                    self.normalize_expression(&mut arm.body, scopes, expected)?;
                    scopes.pop();
                }
                Ok(())
            }
            Expression::StructLiteral { fields, .. } => {
                for (_, value) in fields {
                    self.normalize_expression(value, scopes, None)?;
                }
                Ok(())
            }
            Expression::ArrayLiteral(elements) => {
                let element = expected.and_then(|expected| match expected {
                    Type::Array(element, _) => Some(element.as_ref()),
                    _ => None,
                });
                for value in elements {
                    self.normalize_expression(value, scopes, element)?;
                }
                Ok(())
            }
            Expression::ArrayRepeat { value, .. } => {
                let element = expected.and_then(|expected| match expected {
                    Type::Array(element, _) => Some(element.as_ref()),
                    _ => None,
                });
                self.normalize_expression(value, scopes, element)
            }
            Expression::TupleLiteral(elements) => {
                let expected_elements = expected.and_then(|expected| match expected {
                    Type::Tuple(elements) => Some(elements.as_slice()),
                    _ => None,
                });
                for (index, value) in elements.iter_mut().enumerate() {
                    self.normalize_expression(
                        value,
                        scopes,
                        expected_elements.and_then(|elements| elements.get(index)),
                    )?;
                }
                Ok(())
            }
            Expression::Binary { left, right, .. }
            | Expression::Comparison { left, right, .. }
            | Expression::Logical { left, right, .. } => {
                self.normalize_expression(left, scopes, None)?;
                self.normalize_expression(right, scopes, None)
            }
            Expression::MethodCall {
                object, arguments, ..
            } => {
                self.normalize_expression(object, scopes, None)?;
                for argument in arguments {
                    self.normalize_expression(argument, scopes, None)?;
                }
                Ok(())
            }
            Expression::Print { arguments, .. } | Expression::Println { arguments, .. } => {
                for argument in arguments {
                    self.normalize_expression(argument, scopes, None)?;
                }
                Ok(())
            }
            Expression::Unary { operand, .. } => self.normalize_expression(operand, scopes, None),
            Expression::IndexAccess { object, index } => {
                self.normalize_expression(object, scopes, None)?;
                self.normalize_expression(index, scopes, None)
            }
            Expression::FieldAccess { object, .. } | Expression::TupleIndex { object, .. } => {
                self.normalize_expression(object, scopes, None)
            }
            Expression::Borrow { expr, .. } => {
                if self
                    .expression_type(expr, scopes)
                    .as_ref()
                    .and_then(|ty| self.contract_for_type(ty))
                    .is_some()
                {
                    return Err("generic enum references are not admitted in CAP-006".to_string());
                }
                self.normalize_expression(expr, scopes, None)
            }
            Expression::Deref(expr) => self.normalize_expression(expr, scopes, None),
            Expression::Closure { params, body, .. } => {
                if params
                    .iter()
                    .any(|parameter| self.type_mentions_generic_enum(&parameter.param_type))
                    || expression_mentions_source_generic_enum(body, &self.definitions)
                {
                    return Err(
                        "generic enums are not admitted in closure syntax in CAP-006".to_string(),
                    );
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

    fn normalize_pattern(
        &self,
        pattern: &mut Pattern,
        expected: &Type,
    ) -> Result<Vec<(String, Type)>, String> {
        match pattern {
            Pattern::Enum {
                enum_name,
                variant,
                data,
            } => {
                let expected_contract = self.contract_for_type(expected);
                let contract = if self.definitions.contains_key(enum_name) {
                    let Some(contract) = expected_contract else {
                        return Err(format!(
                            "generic enum pattern `{enum_name}::{variant}` requires an exact generic enum scrutinee"
                        ));
                    };
                    if contract.source_name != *enum_name {
                        return Err(format!(
                            "generic enum pattern `{enum_name}::{variant}` does not match scrutinee type {}",
                            contract.canonical
                        ));
                    }
                    *enum_name = contract.private_name.clone();
                    Some(contract)
                } else if let Some(contract) = self.contracts.get(enum_name) {
                    if expected_contract
                        .is_some_and(|expected| expected.private_name != contract.private_name)
                    {
                        return Err(format!(
                            "generic enum pattern `{}` does not match scrutinee specialization",
                            contract.canonical
                        ));
                    }
                    Some(contract)
                } else {
                    None
                };
                let Some(contract) = contract else {
                    return Ok(Vec::new());
                };
                let Some(fields) = contract.variant_fields(variant) else {
                    return Err(format!(
                        "generic enum `{}` has no variant `{variant}`",
                        contract.canonical
                    ));
                };
                let patterns = data.as_mut().map(Vec::as_mut_slice).unwrap_or_default();
                if patterns.len() != fields.len() {
                    return Err(format!(
                        "generic enum `{}` pattern `{variant}` requires {} positional field(s), actual {}",
                        contract.canonical,
                        fields.len(),
                        patterns.len()
                    ));
                }
                let mut bindings = Vec::new();
                for (pattern, field) in patterns.iter_mut().zip(fields) {
                    match pattern {
                        Pattern::Identifier(name) => bindings.push((name.clone(), field)),
                        _ => bindings.extend(self.normalize_pattern(pattern, &field)?),
                    }
                }
                Ok(bindings)
            }
            Pattern::Tuple(patterns) => {
                let Type::Tuple(elements) = expected else {
                    return Ok(Vec::new());
                };
                let mut bindings = Vec::new();
                for (pattern, element) in patterns.iter_mut().zip(elements) {
                    bindings.extend(self.normalize_pattern(pattern, element)?);
                }
                Ok(bindings)
            }
            Pattern::Struct { fields, .. } => {
                if fields.iter().any(|(_, pattern)| {
                    pattern_mentions_source_generic_enum(pattern, &self.definitions)
                }) {
                    return Err(
                        "nested generic enum patterns are not admitted in CAP-006".to_string()
                    );
                }
                Ok(Vec::new())
            }
            Pattern::Identifier(name) => Ok(vec![(name.clone(), expected.clone())]),
            Pattern::Wildcard | Pattern::Literal(_) => Ok(Vec::new()),
        }
    }

    fn expression_type(&self, expression: &Expression, scopes: &TypeScopes) -> Option<Type> {
        match expression {
            Expression::Identifier(name) => scopes.get(name),
            Expression::EnumVariant { enum_name, .. } if self.contracts.contains_key(enum_name) => {
                Some(Type::Named(enum_name.clone()))
            }
            Expression::FunctionCall { name, .. } => self
                .functions
                .get(name)
                .and_then(Option::as_ref)
                .and_then(|signature| signature.result.clone()),
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
            Expression::Deref(expr) => self.expression_type(expr, scopes).and_then(|ty| match ty {
                Type::Reference(inner, _) => Some(*inner),
                _ => None,
            }),
            _ => None,
        }
    }

    fn contract_for_type(&self, ty: &Type) -> Option<&GenericEnumContract> {
        let Type::Named(name) = ty else {
            return None;
        };
        self.contracts.get(name)
    }

    fn type_mentions_generic_enum(&self, ty: &Type) -> bool {
        type_mentions_generic_enum(ty, &self.definitions, &self.contracts)
    }
}

pub(crate) fn private_generic_enum_source_name(name: &str) -> Option<String> {
    let (canonical, schema) = decode_private_payload(name)?;
    let (source_name, arguments) = parse_canonical_application(&canonical)?;
    if display_application(&source_name, &arguments).ok()? != canonical {
        return None;
    }
    decode_schema(&schema)?;
    Some(canonical)
}

pub(crate) fn valid_generic_aware_enum_symbol(
    name: &str,
    valid_source_symbol: fn(&str) -> bool,
) -> bool {
    if name.starts_with(PRIVATE_GENERIC_ENUM_PREFIX) {
        private_generic_enum_source_name(name).is_some()
    } else {
        valid_source_symbol(name)
    }
}

pub(crate) fn valid_generic_enum_schema(name: &str, variants: &[EnumVariantSchema]) -> bool {
    if !name.starts_with(PRIVATE_GENERIC_ENUM_PREFIX) {
        return true;
    }
    let Some((canonical, encoded_schema)) = decode_private_payload(name) else {
        return false;
    };
    let Some((source_name, arguments)) = parse_canonical_application(&canonical) else {
        return false;
    };
    if display_application(&source_name, &arguments)
        .ok()
        .as_deref()
        != Some(canonical.as_str())
    {
        return false;
    }
    let Some(expected) = decode_schema(&encoded_schema) else {
        return false;
    };
    expected.len() == variants.len()
        && expected
            .iter()
            .zip(variants)
            .all(|((expected_name, expected_fields), actual)| {
                expected_name == &actual.name
                    && match (expected_fields.as_slice(), actual.payload.as_ref()) {
                        ([], None) => true,
                        ([expected], Some(actual)) => {
                            canonical_copydata_type_matches_logical(expected, actual)
                        }
                        (expected, Some(LogicalType::EnumFields { fields }))
                            if expected.len() >= 2 && expected.len() == fields.len() =>
                        {
                            expected.iter().zip(fields).all(|(expected, actual)| {
                                canonical_copydata_type_matches_logical(expected, actual)
                            })
                        }
                        _ => false,
                    }
            })
}

#[cfg(test)]
pub(crate) fn private_name_for_test(canonical: &str, variants: &[(&str, Vec<Type>)]) -> String {
    let variants = variants
        .iter()
        .map(|(name, fields)| VariantDecl {
            name: (*name).to_string(),
            kind: if fields.is_empty() {
                VariantDeclKind::Unit
            } else {
                VariantDeclKind::Tuple(fields.clone())
            },
        })
        .collect::<Vec<_>>();
    private_name_for(canonical, &variants).expect("test generic-enum schema is canonical")
}

fn register_variant_context(
    registry: &mut BTreeMap<(String, String), Option<Vec<Type>>>,
    enum_name: &str,
    variants: &[VariantDecl],
) {
    for variant in variants {
        let payload = match &variant.kind {
            VariantDeclKind::Unit => Vec::new(),
            VariantDeclKind::Tuple(fields) => fields.clone(),
            VariantDeclKind::Struct(fields) => fields
                .iter()
                .map(|field| field.field_type.clone())
                .collect(),
        };
        let key = (enum_name.to_string(), variant.name.clone());
        match registry.entry(key) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(Some(payload));
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                entry.insert(None);
            }
        }
    }
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
        Type::Reference(_, _) | Type::Generic(_, _) => {
            Err("unsupported generic enum template escaped CAP-006 validation".to_string())
        }
    }
}

fn display_application(name: &str, arguments: &[Type]) -> Result<String, String> {
    Ok(format!(
        "{name}<{}>",
        arguments
            .iter()
            .map(display_source_type)
            .collect::<Result<Vec<_>, _>>()?
            .join(",")
    ))
}

fn display_application_diagnostic(name: &str, arguments: &[Type]) -> String {
    format!(
        "{name}<{}>",
        arguments
            .iter()
            .map(display_type_diagnostic)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn display_type_diagnostic(ty: &Type) -> String {
    match ty {
        Type::Named(name) => {
            crate::generic_struct_contract::private_generic_struct_source_name(name)
                .or_else(|| crate::builtin_carrier_contract::private_carrier_source_name(name))
                .or_else(|| private_generic_enum_source_name(name))
                .unwrap_or_else(|| name.clone())
        }
        Type::Array(element, count) => {
            format!("[{};{count}]", display_type_diagnostic(element))
        }
        Type::Tuple(elements) => format!(
            "({})",
            elements
                .iter()
                .map(display_type_diagnostic)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Type::Reference(inner, mutable) => format!(
            "&{}{}",
            if *mutable { "mut " } else { "" },
            display_type_diagnostic(inner)
        ),
        Type::Generic(name, arguments) => display_application_diagnostic(name, arguments),
    }
}

fn private_name_for(canonical: &str, variants: &[VariantDecl]) -> Result<String, String> {
    let schema = encode_schema(variants)?;
    Ok(private_identity(
        PRIVATE_GENERIC_ENUM_PREFIX,
        &[canonical, &schema],
    ))
}

fn decode_private_payload(name: &str) -> Option<(String, String)> {
    let mut parts = decode_private_identity(PRIVATE_GENERIC_ENUM_PREFIX, name, 2)?.into_iter();
    Some((parts.next()?, parts.next()?))
}

fn encode_schema(variants: &[VariantDecl]) -> Result<String, String> {
    let mut encoded = format!("{}:", variants.len());
    for variant in variants {
        push_piece(&mut encoded, &variant.name);
        let fields = match &variant.kind {
            VariantDeclKind::Unit => Vec::new(),
            VariantDeclKind::Tuple(fields) => fields.clone(),
            VariantDeclKind::Struct(_) => {
                return Err("CAP-006 private schemas exclude named fields".to_string());
            }
        };
        encoded.push_str(&format!("{}:", fields.len()));
        for field in fields {
            push_piece(&mut encoded, &display_source_type(&field)?);
        }
    }
    Ok(encoded)
}

fn push_piece(target: &mut String, value: &str) {
    target.push_str(&format!("{}:{value}", value.len()));
}

fn decode_schema(source: &str) -> Option<Vec<(String, Vec<Type>)>> {
    let mut cursor = 0usize;
    let count = take_count(source, &mut cursor)?;
    let mut variants = Vec::with_capacity(count);
    for _ in 0..count {
        let name = take_piece(source, &mut cursor)?;
        if !valid_source_symbol(&name) {
            return None;
        }
        let field_count = take_count(source, &mut cursor)?;
        let mut fields = Vec::with_capacity(field_count);
        for _ in 0..field_count {
            let field = take_piece(source, &mut cursor)?;
            let parsed = parse_canonical_copydata_type_list(&field)?;
            let [field] = parsed.as_slice() else {
                return None;
            };
            fields.push(field.clone());
        }
        variants.push((name, fields));
    }
    (cursor == source.len()).then_some(variants)
}

fn take_count(source: &str, cursor: &mut usize) -> Option<usize> {
    let rest = source.get(*cursor..)?;
    let colon = rest.find(':')?;
    if colon == 0 || !rest[..colon].bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let count = rest[..colon].parse().ok()?;
    *cursor += colon + 1;
    Some(count)
}

fn take_piece(source: &str, cursor: &mut usize) -> Option<String> {
    let length = take_count(source, cursor)?;
    let end = cursor.checked_add(length)?;
    let value = source.get(*cursor..end)?.to_string();
    *cursor = end;
    Some(value)
}

fn variants_equal(left: &[VariantDecl], right: &[VariantDecl]) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            left.name == right.name
                && match (&left.kind, &right.kind) {
                    (VariantDeclKind::Unit, VariantDeclKind::Unit) => true,
                    (VariantDeclKind::Tuple(left), VariantDeclKind::Tuple(right)) => {
                        types_equal(left, right)
                    }
                    (VariantDeclKind::Struct(left), VariantDeclKind::Struct(right)) => {
                        left.len() == right.len()
                            && left.iter().zip(right).all(|(left, right)| {
                                left.name == right.name
                                    && type_equal(&left.field_type, &right.field_type)
                            })
                    }
                    _ => false,
                }
        })
}

fn decoded_schema_matches_variants(
    expected: &[(String, Vec<Type>)],
    actual: &[VariantDecl],
    structs: &StructRegistry,
) -> bool {
    expected.len() == actual.len()
        && expected
            .iter()
            .zip(actual)
            .all(|((expected_name, expected_fields), actual)| {
                if expected_name != &actual.name {
                    return false;
                }
                let actual_fields = match &actual.kind {
                    VariantDeclKind::Unit => Vec::new(),
                    VariantDeclKind::Tuple(fields) => fields.clone(),
                    VariantDeclKind::Struct(_) => return false,
                };
                expected_fields.len() == actual_fields.len()
                    && expected_fields
                        .iter()
                        .zip(actual_fields)
                        .all(|(expected, actual)| {
                            structs
                                .resolve_copy_annotation(&actual)
                                .is_some_and(|contract| {
                                    canonical_copydata_type_matches_logical(
                                        expected,
                                        &contract.logical_type,
                                    )
                                })
                        })
            })
}

fn types_equal(left: &[Type], right: &[Type]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| type_equal(left, right))
}

fn type_equal(left: &Type, right: &Type) -> bool {
    specialization_types_equal(left, right)
}

fn contains_reference(ty: &Type) -> bool {
    match ty {
        Type::Reference(_, _) => true,
        Type::Array(element, _) => contains_reference(element),
        Type::Tuple(elements) | Type::Generic(_, elements) => {
            elements.iter().any(contains_reference)
        }
        Type::Named(_) => false,
    }
}

fn type_mentions_generic_enum(
    ty: &Type,
    definitions: &BTreeMap<String, GenericEnumDefinition>,
    contracts: &BTreeMap<String, GenericEnumContract>,
) -> bool {
    match ty {
        Type::Named(name) => definitions.contains_key(name) || contracts.contains_key(name),
        Type::Array(element, _) | Type::Reference(element, _) => {
            type_mentions_generic_enum(element, definitions, contracts)
        }
        Type::Tuple(elements) => elements
            .iter()
            .any(|element| type_mentions_generic_enum(element, definitions, contracts)),
        Type::Generic(name, elements) => {
            definitions.contains_key(name)
                || elements
                    .iter()
                    .any(|element| type_mentions_generic_enum(element, definitions, contracts))
        }
    }
}

fn statement_mentions_generic_enum(
    statement: &Statement,
    definitions: &BTreeMap<String, GenericEnumDefinition>,
    contracts: &BTreeMap<String, GenericEnumContract>,
) -> bool {
    match statement {
        Statement::Function {
            parameters,
            return_type,
            body,
            ..
        } => {
            parameters.iter().any(|parameter| {
                type_mentions_generic_enum(&parameter.param_type, definitions, contracts)
            }) || return_type
                .as_ref()
                .is_some_and(|result| type_mentions_generic_enum(result, definitions, contracts))
                || block_mentions_source_generic_enum(body, definitions)
        }
        _ => false,
    }
}

fn block_mentions_source_generic_enum(
    block: &Block,
    definitions: &BTreeMap<String, GenericEnumDefinition>,
) -> bool {
    block
        .statements
        .iter()
        .any(|statement| statement_value_mentions_source_generic_enum(statement, definitions))
        || block.expression.as_ref().is_some_and(|expression| {
            expression_mentions_source_generic_enum(expression, definitions)
        })
}

fn statement_value_mentions_source_generic_enum(
    statement: &Statement,
    definitions: &BTreeMap<String, GenericEnumDefinition>,
) -> bool {
    match statement {
        Statement::Const { value, .. }
        | Statement::Let {
            value: Some(value), ..
        }
        | Statement::Return(Some(value))
        | Statement::Expression(value) => {
            expression_mentions_source_generic_enum(value, definitions)
        }
        Statement::Assignment { target, value } => {
            expression_mentions_source_generic_enum(target, definitions)
                || expression_mentions_source_generic_enum(value, definitions)
        }
        Statement::Block(block) | Statement::Loop { body: block } => {
            block_mentions_source_generic_enum(block, definitions)
        }
        Statement::If {
            condition,
            then_block,
            else_block,
        } => {
            expression_mentions_source_generic_enum(condition, definitions)
                || block_mentions_source_generic_enum(then_block, definitions)
                || else_block.as_ref().is_some_and(|otherwise| {
                    statement_value_mentions_source_generic_enum(otherwise, definitions)
                })
        }
        Statement::While { condition, body } => {
            expression_mentions_source_generic_enum(condition, definitions)
                || block_mentions_source_generic_enum(body, definitions)
        }
        Statement::For { iterable, body, .. } => {
            expression_mentions_source_generic_enum(iterable, definitions)
                || block_mentions_source_generic_enum(body, definitions)
        }
        Statement::Function { body, .. } => block_mentions_source_generic_enum(body, definitions),
        Statement::ImplBlock { methods, .. } => methods
            .iter()
            .any(|method| statement_value_mentions_source_generic_enum(method, definitions)),
        Statement::TraitDef { methods, .. } => methods.iter().any(|method| {
            method
                .body
                .as_ref()
                .is_some_and(|body| block_mentions_source_generic_enum(body, definitions))
        }),
        Statement::Let { value: None, .. }
        | Statement::Return(None)
        | Statement::Break
        | Statement::Continue
        | Statement::StructDef { .. }
        | Statement::EnumDef { .. }
        | Statement::ModDecl { .. }
        | Statement::UseImport { .. } => false,
    }
}

fn expression_mentions_source_generic_enum(
    expression: &Expression,
    definitions: &BTreeMap<String, GenericEnumDefinition>,
) -> bool {
    match expression {
        Expression::EnumVariant {
            enum_name, data, ..
        } => {
            definitions.contains_key(enum_name)
                || data.as_ref().is_some_and(|fields| {
                    fields
                        .iter()
                        .any(|field| expression_mentions_source_generic_enum(field, definitions))
                })
        }
        Expression::FunctionCall { arguments, .. }
        | Expression::Print { arguments, .. }
        | Expression::Println { arguments, .. }
        | Expression::ArrayLiteral(arguments)
        | Expression::TupleLiteral(arguments) => arguments
            .iter()
            .any(|argument| expression_mentions_source_generic_enum(argument, definitions)),
        Expression::MethodCall {
            object, arguments, ..
        } => {
            expression_mentions_source_generic_enum(object, definitions)
                || arguments
                    .iter()
                    .any(|argument| expression_mentions_source_generic_enum(argument, definitions))
        }
        Expression::Binary { left, right, .. }
        | Expression::Comparison { left, right, .. }
        | Expression::Logical { left, right, .. } => {
            expression_mentions_source_generic_enum(left, definitions)
                || expression_mentions_source_generic_enum(right, definitions)
        }
        Expression::Unary { operand, .. }
        | Expression::Borrow { expr: operand, .. }
        | Expression::Deref(operand)
        | Expression::ArrayRepeat { value: operand, .. }
        | Expression::Closure { body: operand, .. } => {
            expression_mentions_source_generic_enum(operand, definitions)
        }
        Expression::IndexAccess { object, index } => {
            expression_mentions_source_generic_enum(object, definitions)
                || expression_mentions_source_generic_enum(index, definitions)
        }
        Expression::FieldAccess { object, .. } | Expression::TupleIndex { object, .. } => {
            expression_mentions_source_generic_enum(object, definitions)
        }
        Expression::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expression_mentions_source_generic_enum(value, definitions)),
        Expression::Match { expr, arms } => {
            expression_mentions_source_generic_enum(expr, definitions)
                || arms.iter().any(|arm| {
                    pattern_mentions_source_generic_enum(&arm.pattern, definitions)
                        || expression_mentions_source_generic_enum(&arm.body, definitions)
                })
        }
        Expression::IntegerLiteral(_)
        | Expression::FloatLiteral(_)
        | Expression::CharacterLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::Identifier(_) => false,
    }
}

fn pattern_mentions_source_generic_enum(
    pattern: &Pattern,
    definitions: &BTreeMap<String, GenericEnumDefinition>,
) -> bool {
    match pattern {
        Pattern::Enum {
            enum_name, data, ..
        } => {
            definitions.contains_key(enum_name)
                || data.as_ref().is_some_and(|patterns| {
                    patterns
                        .iter()
                        .any(|pattern| pattern_mentions_source_generic_enum(pattern, definitions))
                })
        }
        Pattern::Tuple(patterns) => patterns
            .iter()
            .any(|pattern| pattern_mentions_source_generic_enum(pattern, definitions)),
        Pattern::Struct { fields, .. } => fields
            .iter()
            .any(|(_, pattern)| pattern_mentions_source_generic_enum(pattern, definitions)),
        Pattern::Wildcard | Pattern::Literal(_) | Pattern::Identifier(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(source: &str) -> Vec<AstNode> {
        let tokens = crate::lexer::try_tokenize_with_locations(source, None).expect("lex");
        crate::parser::parse_with_locations(tokens).expect("parse")
    }

    #[test]
    fn specialization_is_idempotent_and_removes_the_executable_template() {
        let source = "enum Sample<T> { Present(T), Missing } \
            fn score(value: Sample<int>) -> int { match value { Sample::Present(number) => number, Sample::Missing => 0 } } \
            fn main() -> int { let value: Sample<int> = Sample::Present(7); score(value) }";
        let once = normalize_generic_copydata_enums(parsed(source)).expect("specialize once");
        let twice = normalize_generic_copydata_enums(once.clone()).expect("specialize twice");
        assert_eq!(format!("{once:#?}"), format!("{twice:#?}"));

        let private = once
            .iter()
            .find_map(|node| match node {
                AstNode::Statement(Statement::EnumDef {
                    name, type_params, ..
                }) if type_params.is_empty() && name.starts_with(PRIVATE_GENERIC_ENUM_PREFIX) => {
                    Some(name)
                }
                _ => None,
            })
            .expect("one private specialization must be emitted");
        assert_eq!(
            private_generic_enum_source_name(private).as_deref(),
            Some("Sample<int>")
        );
        assert!(!once.iter().any(|node| matches!(
            node,
            AstNode::Statement(Statement::EnumDef { type_params, .. })
                if !type_params.is_empty()
        )));
    }

    #[test]
    fn private_identity_commits_to_the_complete_variant_schema() {
        let identity = private_name_for_test(
            "Sample<int>",
            &[
                ("Present", vec![Type::Named("int".to_string())]),
                ("Missing", Vec::new()),
            ],
        );
        let exact = vec![
            EnumVariantSchema {
                name: "Present".to_string(),
                payload: Some(LogicalType::Int),
            },
            EnumVariantSchema {
                name: "Missing".to_string(),
                payload: None,
            },
        ];
        assert_eq!(
            private_generic_enum_source_name(&identity).as_deref(),
            Some("Sample<int>")
        );
        assert!(valid_generic_enum_schema(&identity, &exact));

        let mut wrong_payload = exact.clone();
        wrong_payload[0].payload = Some(LogicalType::Char);
        assert!(!valid_generic_enum_schema(&identity, &wrong_payload));
        let mut wrong_variant = exact;
        wrong_variant[1].name = "Absent".to_string();
        assert!(!valid_generic_enum_schema(&identity, &wrong_variant));
        assert!(!valid_generic_enum_schema(&format!("{identity}00"), &[]));
    }
}
