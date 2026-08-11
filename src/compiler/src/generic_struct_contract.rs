use crate::ast::{
    AstNode, Block, Expression, FieldDecl, Pattern, Statement, Type, VariantDeclKind,
};
use crate::builtin_carrier_contract::private_carrier_source_name;
use crate::ir::LogicalType;
use crate::primitive_contract::PrimitiveKind;
use crate::struct_contract::StructRegistry;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const PRIVATE_GENERIC_STRUCT_PREFIX: &str = "__aero$generic_struct$";

#[derive(Debug, Clone)]
struct GenericStructDefinition {
    parameters: Vec<String>,
    fields: Vec<FieldDecl>,
}

#[derive(Debug, Clone)]
struct GenericStructContract {
    source_name: String,
    canonical: String,
    private_name: String,
    fields: Vec<FieldDecl>,
}

impl GenericStructContract {
    fn definition(&self) -> AstNode {
        AstNode::Statement(Statement::StructDef {
            name: self.private_name.clone(),
            fields: self.fields.clone(),
            type_params: Vec::new(),
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
            .expect("generic-struct scope stack remains balanced");
    }

    fn insert(&mut self, name: String, ty: Option<Type>) {
        self.scopes
            .last_mut()
            .expect("generic-struct scope stack is nonempty")
            .insert(name, ty);
    }

    fn get(&self, name: &str) -> Option<Type> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).cloned().flatten())
    }
}

#[derive(Debug, Default)]
struct GenericStructNormalizer {
    definitions: BTreeMap<String, GenericStructDefinition>,
    contracts: BTreeMap<String, GenericStructContract>,
    functions: BTreeMap<String, Option<FunctionSignature>>,
    struct_fields: BTreeMap<String, Option<Vec<FieldDecl>>>,
    enum_variants: BTreeMap<(String, String), Option<Vec<Type>>>,
}

pub(crate) fn private_generic_struct_source_name(name: &str) -> Option<String> {
    let (source, _) = decode_private_payload(name)?;
    let parsed = CanonicalTypeParser::new(&source).parse_complete().ok()?;
    if display_source_type(&parsed).ok()? != source {
        return None;
    }
    match parsed {
        Type::Generic(_, ref arguments) if !arguments.is_empty() => Some(source),
        _ => None,
    }
}

pub(crate) fn valid_generic_struct_schema(name: &str, fields: &[LogicalType]) -> bool {
    if !name.starts_with(PRIVATE_GENERIC_STRUCT_PREFIX) {
        return true;
    }
    let Some((source, encoded_fields)) = decode_private_payload(name) else {
        return false;
    };
    if CanonicalTypeParser::new(&source)
        .parse_complete()
        .ok()
        .and_then(|ty| display_source_type(&ty).ok())
        .as_deref()
        != Some(source.as_str())
    {
        return false;
    }
    let Ok(expected_fields) = CanonicalTypeParser::new(&encoded_fields).parse_type_list() else {
        return false;
    };
    expected_fields.len() == fields.len()
        && expected_fields
            .iter()
            .zip(fields)
            .all(|(expected, actual)| annotation_matches_logical(expected, actual))
}

pub(crate) fn valid_generic_aware_struct_symbol(
    name: &str,
    valid_source_symbol: fn(&str) -> bool,
) -> bool {
    if name.starts_with(PRIVATE_GENERIC_STRUCT_PREFIX) {
        private_generic_struct_source_name(name).is_some()
    } else {
        valid_source_symbol(name)
    }
}

pub(crate) fn parse_canonical_copydata_type_list(source: &str) -> Option<Vec<Type>> {
    let types = CanonicalTypeParser::new(source).parse_type_list().ok()?;
    let canonical = types
        .iter()
        .map(display_source_type)
        .collect::<Result<Vec<_>, _>>()
        .ok()?
        .join(",");
    (canonical == source).then_some(types)
}

pub(crate) fn canonical_copydata_type_matches_logical(
    expected: &Type,
    actual: &LogicalType,
) -> bool {
    annotation_matches_logical(expected, actual)
}

pub(crate) fn normalize_generic_copydata_structs(
    ast: Vec<AstNode>,
) -> Result<Vec<AstNode>, String> {
    let mut retained = Vec::with_capacity(ast.len());
    let mut existing_private = Vec::new();
    for node in ast {
        match &node {
            AstNode::Statement(Statement::StructDef { name, .. })
                if name.starts_with(PRIVATE_GENERIC_STRUCT_PREFIX) =>
            {
                existing_private.push(node);
            }
            _ => retained.push(node),
        }
    }

    let mut normalizer = GenericStructNormalizer::default();
    normalizer.collect_definitions(&retained)?;
    if normalizer.definitions.is_empty() && existing_private.is_empty() {
        return Ok(retained);
    }
    for definition in existing_private {
        normalizer.register_existing_private_definition(definition)?;
    }
    normalizer.normalize_annotations(&mut retained)?;
    normalizer.validate_contracts(&retained)?;
    normalizer.prepare_context(&retained)?;
    normalizer.normalize_top_level(&mut retained)?;

    let mut normalized = normalizer
        .contracts
        .values()
        .map(GenericStructContract::definition)
        .collect::<Vec<_>>();
    normalized.extend(retained);
    Ok(normalized)
}

impl GenericStructNormalizer {
    fn collect_definitions(&mut self, ast: &[AstNode]) -> Result<(), String> {
        let mut definition_counts = BTreeMap::new();
        for node in ast {
            if let AstNode::Statement(Statement::StructDef { name, .. }) = node {
                *definition_counts.entry(name.clone()).or_insert(0usize) += 1;
            }
        }
        for node in ast {
            let AstNode::Statement(Statement::StructDef {
                name,
                fields,
                type_params,
            }) = node
            else {
                continue;
            };
            if name.starts_with(PRIVATE_GENERIC_STRUCT_PREFIX) {
                return Err(format!(
                    "source struct name `{name}` uses Aero's reserved generic-struct identity"
                ));
            }
            if type_params.is_empty() {
                continue;
            }
            if definition_counts.get(name).copied() != Some(1) {
                return Err(format!("duplicate generic struct definition `{name}`"));
            }
            if !valid_source_symbol(name) || fields.is_empty() {
                return Err(format!("generic struct `{name}` has an invalid definition"));
            }
            let mut parameters = BTreeSet::new();
            for parameter in type_params {
                if !valid_source_symbol(parameter) || !parameters.insert(parameter.clone()) {
                    return Err(format!(
                        "generic struct `{name}` has duplicate or invalid type parameter `{parameter}`"
                    ));
                }
            }
            let mut field_names = BTreeSet::new();
            let mut used_parameters = BTreeSet::new();
            for field in fields {
                if !valid_source_symbol(&field.name) || !field_names.insert(field.name.clone()) {
                    return Err(format!(
                        "generic struct `{name}` has duplicate or invalid field `{}`",
                        field.name
                    ));
                }
                validate_template_type(&field.field_type, name, &parameters, &mut used_parameters)?;
            }
            if used_parameters != parameters {
                let unused = parameters
                    .difference(&used_parameters)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(format!(
                    "generic struct `{name}` has unused type parameter(s): {unused}"
                ));
            }
            self.definitions.insert(
                name.clone(),
                GenericStructDefinition {
                    parameters: type_params.clone(),
                    fields: fields.clone(),
                },
            );
        }
        Ok(())
    }

    fn normalize_annotations(&mut self, ast: &mut [AstNode]) -> Result<(), String> {
        for node in ast {
            match node {
                AstNode::Statement(statement) => self.normalize_statement_annotations(statement)?,
                AstNode::Expression(_) => {}
            }
        }
        Ok(())
    }

    fn normalize_statement_annotations(&mut self, statement: &mut Statement) -> Result<(), String> {
        match statement {
            Statement::Const {
                type_annotation, ..
            } => self.normalize_type(type_annotation, "constant annotations"),
            Statement::Let {
                type_annotation, ..
            } => {
                if let Some(annotation) = type_annotation {
                    self.normalize_type(annotation, "binding annotations")?;
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
                trait_bounds,
                ..
            } => {
                let mentions = parameters.iter().any(|parameter| {
                    contains_source_generic_struct(&parameter.param_type, &self.definitions)
                }) || return_type.as_ref().is_some_and(|result| {
                    contains_source_generic_struct(result, &self.definitions)
                });
                if mentions
                    && !type_params.is_empty()
                    && (!trait_bounds.is_empty()
                        || !crate::generic_function_contract::has_complete_direct_type_parameter_inference(
                            type_params,
                            parameters,
                        ))
                {
                    return Err(format!(
                        "generic function `{name}` cannot transport an explicit generic CopyData struct in CAP-004"
                    ));
                }
                for parameter in parameters {
                    self.normalize_type(&mut parameter.param_type, "function parameters")?;
                }
                if let Some(result) = return_type {
                    self.normalize_type(result, "function results")?;
                }
                self.normalize_block_annotations(body)
            }
            Statement::StructDef {
                name,
                fields,
                type_params,
            } if type_params.is_empty() => {
                for field in fields {
                    self.normalize_type(&mut field.field_type, "struct fields")?;
                }
                if name.starts_with(PRIVATE_GENERIC_STRUCT_PREFIX) {
                    return Err(format!(
                        "source struct name `{name}` uses Aero's reserved generic-struct identity"
                    ));
                }
                Ok(())
            }
            Statement::StructDef { .. } => Ok(()),
            Statement::EnumDef {
                name,
                variants,
                type_params,
                ..
            } => {
                let mentions = variants.iter().any(|variant| match &variant.kind {
                    VariantDeclKind::Unit => false,
                    VariantDeclKind::Tuple(fields) => fields
                        .iter()
                        .any(|field| contains_source_generic_struct(field, &self.definitions)),
                    VariantDeclKind::Struct(fields) => fields.iter().any(|field| {
                        contains_source_generic_struct(&field.field_type, &self.definitions)
                    }),
                });
                if mentions && !type_params.is_empty() {
                    return Err(format!(
                        "generic enum `{name}` cannot store an explicit generic CopyData struct in CAP-004"
                    ));
                }
                for variant in variants {
                    match &mut variant.kind {
                        VariantDeclKind::Unit => {}
                        VariantDeclKind::Tuple(fields) => {
                            for field in fields {
                                self.normalize_type(field, "enum payloads")?;
                            }
                        }
                        VariantDeclKind::Struct(fields) => {
                            for field in fields {
                                self.normalize_type(&mut field.field_type, "enum payloads")?;
                            }
                        }
                    }
                }
                Ok(())
            }
            Statement::ImplBlock {
                type_name,
                methods,
                type_params,
                ..
            } => {
                if (self.definitions.contains_key(type_name) || !type_params.is_empty())
                    && methods
                        .iter()
                        .any(|method| statement_mentions_generic_struct(method, &self.definitions))
                {
                    return Err(
                        "generic CopyData structs are not admitted in impl blocks in CAP-004"
                            .to_string(),
                    );
                }
                for method in methods {
                    self.normalize_statement_annotations(method)?;
                }
                Ok(())
            }
            Statement::TraitDef { methods, .. } => {
                for method in methods {
                    if method.parameters.iter().any(|parameter| {
                        contains_source_generic_struct(&parameter.param_type, &self.definitions)
                    }) || method.return_type.as_ref().is_some_and(|result| {
                        contains_source_generic_struct(result, &self.definitions)
                    }) {
                        return Err(
                            "generic CopyData structs are not admitted in trait signatures in CAP-004"
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

    fn normalize_type(&mut self, ty: &mut Type, context: &str) -> Result<(), String> {
        match ty {
            Type::Array(element, _) | Type::Reference(element, _) => {
                self.normalize_type(element, context)
            }
            Type::Tuple(elements) => {
                for element in elements {
                    self.normalize_type(element, context)?;
                }
                Ok(())
            }
            Type::Generic(name, arguments) => {
                for argument in arguments.iter_mut() {
                    self.normalize_type(argument, context)?;
                }
                if !self.definitions.contains_key(name) {
                    return Ok(());
                }
                let source_name = name.clone();
                let contract = self.build_contract(&source_name, arguments.clone(), context)?;
                let private_name = contract.private_name.clone();
                self.register_contract(contract)?;
                *ty = Type::Named(private_name);
                Ok(())
            }
            Type::Named(name) if name.starts_with(PRIVATE_GENERIC_STRUCT_PREFIX) => {
                if self.contracts.contains_key(name) {
                    Ok(())
                } else {
                    Err(format!(
                        "unknown private generic-struct identity `{name}` in {context}"
                    ))
                }
            }
            Type::Named(_) => Ok(()),
        }
    }

    fn build_contract(
        &self,
        source_name: &str,
        arguments: Vec<Type>,
        context: &str,
    ) -> Result<GenericStructContract, String> {
        let definition = self
            .definitions
            .get(source_name)
            .ok_or_else(|| format!("unknown generic struct `{source_name}` in {context}"))?;
        if arguments.len() != definition.parameters.len() {
            return Err(format!(
                "generic struct `{source_name}` requires {} type argument(s), actual {}",
                definition.parameters.len(),
                arguments.len()
            ));
        }
        if arguments.iter().any(contains_reference) {
            return Err(format!(
                "generic struct `{source_name}` requires recursive finite CopyData type arguments"
            ));
        }
        let substitutions = definition
            .parameters
            .iter()
            .cloned()
            .zip(arguments.iter().cloned())
            .collect::<BTreeMap<_, _>>();
        let fields = definition
            .fields
            .iter()
            .map(|field| {
                Ok(FieldDecl {
                    name: field.name.clone(),
                    field_type: substitute_type(&field.field_type, &substitutions)?,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let canonical = format!(
            "{source_name}<{}>",
            arguments
                .iter()
                .map(display_source_type)
                .collect::<Result<Vec<_>, _>>()?
                .join(",")
        );
        Ok(GenericStructContract {
            source_name: source_name.to_string(),
            private_name: private_name_for(&canonical, &fields)?,
            canonical,
            fields,
        })
    }

    fn register_contract(&mut self, contract: GenericStructContract) -> Result<(), String> {
        if let Some(existing) = self.contracts.get(&contract.private_name) {
            if existing.canonical != contract.canonical
                || !fields_equal(&existing.fields, &contract.fields)
            {
                return Err("generic-struct private identity collision".to_string());
            }
            return Ok(());
        }
        self.contracts
            .insert(contract.private_name.clone(), contract);
        Ok(())
    }

    fn register_existing_private_definition(&mut self, node: AstNode) -> Result<(), String> {
        let AstNode::Statement(Statement::StructDef {
            name,
            fields,
            type_params,
        }) = node
        else {
            unreachable!("private definition collection retains struct definitions only")
        };
        if !type_params.is_empty() {
            return Err(format!(
                "invalid private generic-struct definition `{name}`"
            ));
        }
        let canonical = private_generic_struct_source_name(&name)
            .ok_or_else(|| format!("invalid private generic-struct identity `{name}`"))?;
        let Type::Generic(source_name, mut arguments) =
            CanonicalTypeParser::new(&canonical).parse_complete()?
        else {
            unreachable!("private identity decoder accepts generic applications only")
        };
        for argument in &mut arguments {
            self.normalize_type(argument, "private generic-struct identities")?;
        }
        let expected = self.build_contract(&source_name, arguments, "private definitions")?;
        if expected.private_name != name || !fields_equal(&expected.fields, &fields) {
            return Err(format!(
                "private generic-struct identity `{name}` does not match its exact schema"
            ));
        }
        self.register_contract(expected)
    }

    fn validate_contracts(&self, retained: &[AstNode]) -> Result<(), String> {
        let mut combined = self
            .contracts
            .values()
            .map(GenericStructContract::definition)
            .collect::<Vec<_>>();
        combined.extend_from_slice(retained);
        let registry = StructRegistry::from_top_level_ast(&combined);
        for contract in self.contracts.values() {
            if registry
                .resolve_copy_annotation(&Type::Named(contract.private_name.clone()))
                .is_none()
            {
                return Err(format!(
                    "generic struct application `{}` is not recursive finite CopyData",
                    contract.canonical
                ));
            }
        }
        Ok(())
    }

    fn prepare_context(&mut self, ast: &[AstNode]) -> Result<(), String> {
        for contract in self.contracts.values() {
            self.struct_fields
                .insert(contract.private_name.clone(), Some(contract.fields.clone()));
        }
        for node in ast {
            let AstNode::Statement(statement) = node else {
                continue;
            };
            match statement {
                Statement::Function {
                    name,
                    parameters,
                    return_type,
                    ..
                } => {
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
                Statement::StructDef {
                    name,
                    fields,
                    type_params,
                } if type_params.is_empty() => match self.struct_fields.entry(name.clone()) {
                    std::collections::btree_map::Entry::Vacant(entry) => {
                        entry.insert(Some(fields.clone()));
                    }
                    std::collections::btree_map::Entry::Occupied(mut entry) => {
                        entry.insert(None);
                    }
                },
                Statement::EnumDef {
                    name,
                    variants,
                    type_params,
                    ..
                } if type_params.is_empty() => {
                    for variant in variants {
                        let payload = match &variant.kind {
                            VariantDeclKind::Unit => Vec::new(),
                            VariantDeclKind::Tuple(fields) => fields.clone(),
                            VariantDeclKind::Struct(fields) => fields
                                .iter()
                                .map(|field| field.field_type.clone())
                                .collect(),
                        };
                        let key = (name.clone(), variant.name.clone());
                        match self.enum_variants.entry(key) {
                            std::collections::btree_map::Entry::Vacant(entry) => {
                                entry.insert(Some(payload));
                            }
                            std::collections::btree_map::Entry::Occupied(mut entry) => {
                                entry.insert(None);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn normalize_top_level(&self, ast: &mut [AstNode]) -> Result<(), String> {
        for node in ast {
            match node {
                AstNode::Statement(Statement::Function {
                    parameters,
                    body,
                    return_type,
                    ..
                }) => {
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
            Statement::ImplBlock { methods, .. } => {
                for method in methods {
                    self.normalize_statement(method, scopes, None)?;
                }
                Ok(())
            }
            Statement::TraitDef { methods, .. } => {
                for method in methods {
                    if let Some(body) = &mut method.body {
                        let mut method_scopes = TypeScopes::new();
                        for parameter in &method.parameters {
                            method_scopes
                                .insert(parameter.name.clone(), Some(parameter.param_type.clone()));
                        }
                        self.normalize_block(
                            body,
                            &mut method_scopes,
                            method.return_type.as_ref(),
                        )?;
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
        &self,
        expression: &mut Expression,
        scopes: &mut TypeScopes,
        expected: Option<&Type>,
    ) -> Result<(), String> {
        match expression {
            Expression::StructLiteral { name, fields } => {
                if self.definitions.contains_key(name) {
                    let Some(Type::Named(private_name)) = expected else {
                        return Err(format!(
                            "generic struct literal `{name}` requires an exact expected {name}<...> type"
                        ));
                    };
                    let Some(contract) = self.contracts.get(private_name) else {
                        return Err(format!(
                            "generic struct literal `{name}` requires an exact expected {name}<...> type"
                        ));
                    };
                    if contract.source_name != *name {
                        return Err(format!(
                            "generic struct literal `{name}` does not match expected type {}",
                            contract.canonical
                        ));
                    }
                    *name = private_name.clone();
                }
                let field_contracts = self.struct_fields.get(name).and_then(Clone::clone);
                for (field_name, value) in fields {
                    let field_type = field_contracts
                        .as_ref()
                        .into_iter()
                        .flatten()
                        .find(|field| field.name == *field_name)
                        .map(|field| &field.field_type);
                    self.normalize_expression(value, scopes, field_type)?;
                }
                Ok(())
            }
            Expression::FunctionCall { name, arguments } => {
                let signature = self.functions.get(name).and_then(Clone::clone);
                for (index, argument) in arguments.iter_mut().enumerate() {
                    let expected = signature
                        .as_ref()
                        .and_then(|signature| signature.parameters.get(index));
                    self.normalize_expression(argument, scopes, expected)?;
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
            Expression::EnumVariant {
                enum_name,
                variant,
                data,
            } => {
                if let Some(values) = data {
                    let payloads = self
                        .enum_variants
                        .get(&(enum_name.clone(), variant.clone()))
                        .and_then(Clone::clone);
                    for (index, value) in values.iter_mut().enumerate() {
                        self.normalize_expression(
                            value,
                            scopes,
                            payloads.as_ref().and_then(|payloads| payloads.get(index)),
                        )?;
                    }
                }
                Ok(())
            }
            Expression::Match { expr, arms } => {
                self.normalize_expression(expr, scopes, None)?;
                let scrutinee = self.expression_type(expr, scopes);
                for arm in arms {
                    if let Some(ty) = &scrutinee {
                        self.normalize_pattern(&mut arm.pattern, ty)?;
                    }
                    self.normalize_expression(&mut arm.body, scopes, expected)?;
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
            Expression::Borrow { expr, .. } => self.normalize_expression(expr, scopes, None),
            Expression::Deref(expr) => self.normalize_expression(expr, scopes, expected),
            Expression::Closure { .. } => Ok(()),
            Expression::IntegerLiteral(_)
            | Expression::FloatLiteral(_)
            | Expression::CharacterLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::Identifier(_) => Ok(()),
        }
    }

    fn normalize_pattern(&self, pattern: &mut Pattern, expected: &Type) -> Result<(), String> {
        match pattern {
            Pattern::Struct { name, fields } if self.definitions.contains_key(name) => {
                let Type::Named(private_name) = expected else {
                    return Err(format!(
                        "generic struct pattern `{name}` requires an exact generic struct scrutinee"
                    ));
                };
                let Some(contract) = self.contracts.get(private_name) else {
                    return Err(format!(
                        "generic struct pattern `{name}` requires an exact generic struct scrutinee"
                    ));
                };
                if contract.source_name != *name {
                    return Err(format!(
                        "generic struct pattern `{name}` does not match scrutinee type {}",
                        contract.canonical
                    ));
                }
                *name = private_name.clone();
                for (field_name, field_pattern) in fields {
                    if let Some(field) = contract
                        .fields
                        .iter()
                        .find(|field| field.name == *field_name)
                    {
                        self.normalize_pattern(field_pattern, &field.field_type)?;
                    }
                }
                Ok(())
            }
            Pattern::Struct { name, fields } => {
                if let Some(Some(contract)) = self.struct_fields.get(name) {
                    for (field_name, field_pattern) in fields {
                        if let Some(field) = contract.iter().find(|field| field.name == *field_name)
                        {
                            self.normalize_pattern(field_pattern, &field.field_type)?;
                        }
                    }
                }
                Ok(())
            }
            Pattern::Tuple(patterns) => {
                if let Type::Tuple(elements) = expected {
                    for (pattern, ty) in patterns.iter_mut().zip(elements) {
                        self.normalize_pattern(pattern, ty)?;
                    }
                }
                Ok(())
            }
            Pattern::Enum { data, .. } => {
                if let Some(patterns) = data {
                    for pattern in patterns {
                        self.normalize_pattern(pattern, expected)?;
                    }
                }
                Ok(())
            }
            Pattern::Wildcard | Pattern::Literal(_) | Pattern::Identifier(_) => Ok(()),
        }
    }

    fn expression_type(&self, expression: &Expression, scopes: &TypeScopes) -> Option<Type> {
        match expression {
            Expression::Identifier(name) => scopes.get(name),
            Expression::StructLiteral { name, .. } if self.struct_fields.contains_key(name) => {
                Some(Type::Named(name.clone()))
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
            Expression::Borrow { expr, mutable } => self
                .expression_type(expr, scopes)
                .map(|ty| Type::Reference(Box::new(ty), *mutable)),
            Expression::Deref(expr) => self.expression_type(expr, scopes).and_then(|ty| match ty {
                Type::Reference(inner, _) => Some(*inner),
                _ => None,
            }),
            _ => None,
        }
    }

    fn field_type(&self, ty: &Type, field: &str) -> Option<Type> {
        let Type::Named(name) = ty else {
            return None;
        };
        self.struct_fields
            .get(name)
            .and_then(Option::as_ref)
            .and_then(|fields| fields.iter().find(|candidate| candidate.name == field))
            .map(|field| field.field_type.clone())
    }
}

fn validate_template_type(
    ty: &Type,
    struct_name: &str,
    parameters: &BTreeSet<String>,
    used: &mut BTreeSet<String>,
) -> Result<(), String> {
    match ty {
        Type::Named(name) if parameters.contains(name) => {
            used.insert(name.clone());
            Ok(())
        }
        Type::Named(name) if name == struct_name => Err(format!(
            "recursive generic struct `{struct_name}` is not admitted in CAP-004"
        )),
        Type::Named(_) => Ok(()),
        Type::Array(element, _) => validate_template_type(element, struct_name, parameters, used),
        Type::Tuple(elements) if elements.len() >= 2 => {
            for element in elements {
                validate_template_type(element, struct_name, parameters, used)?;
            }
            Ok(())
        }
        Type::Tuple(_) => Err(format!(
            "generic struct `{struct_name}` requires tuple fields with at least two elements"
        )),
        Type::Reference(_, _) => Err(format!(
            "generic struct `{struct_name}` fields must be recursive finite CopyData"
        )),
        Type::Generic(_, _) => Err(format!(
            "nested generic applications in generic struct `{struct_name}` fields are not admitted in CAP-004"
        )),
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
            Err("unsupported generic struct template escaped CAP-004 validation".to_string())
        }
    }
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

fn contains_source_generic_struct(
    ty: &Type,
    definitions: &BTreeMap<String, GenericStructDefinition>,
) -> bool {
    match ty {
        Type::Generic(name, _) if definitions.contains_key(name) => true,
        Type::Array(element, _) | Type::Reference(element, _) => {
            contains_source_generic_struct(element, definitions)
        }
        Type::Tuple(elements) | Type::Generic(_, elements) => elements
            .iter()
            .any(|element| contains_source_generic_struct(element, definitions)),
        Type::Named(_) => false,
    }
}

fn statement_mentions_generic_struct(
    statement: &Statement,
    definitions: &BTreeMap<String, GenericStructDefinition>,
) -> bool {
    match statement {
        Statement::Function {
            parameters,
            return_type,
            ..
        } => {
            parameters
                .iter()
                .any(|parameter| contains_source_generic_struct(&parameter.param_type, definitions))
                || return_type
                    .as_ref()
                    .is_some_and(|ty| contains_source_generic_struct(ty, definitions))
        }
        _ => false,
    }
}

fn fields_equal(left: &[FieldDecl], right: &[FieldDecl]) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            left.name == right.name && types_equal(&left.field_type, &right.field_type)
        })
}

fn types_equal(left: &Type, right: &Type) -> bool {
    match (left, right) {
        (Type::Named(left), Type::Named(right)) => left == right,
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
        (Type::Generic(left_name, left), Type::Generic(right_name, right)) => {
            left_name == right_name
                && left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| types_equal(left, right))
        }
        (Type::Reference(left, left_mutable), Type::Reference(right, right_mutable)) => {
            left_mutable == right_mutable && types_equal(left, right)
        }
        _ => false,
    }
}

pub(crate) fn display_source_type(ty: &Type) -> Result<String, String> {
    match ty {
        Type::Named(name) => {
            if let Some(source) = private_generic_struct_source_name(name) {
                Ok(source)
            } else if let Some(source) = private_carrier_source_name(name) {
                Ok(source.replace(", ", ","))
            } else {
                Ok(name.clone())
            }
        }
        Type::Array(element, count) => Ok(format!("[{};{count}]", display_source_type(element)?)),
        Type::Tuple(elements) => Ok(format!(
            "({})",
            elements
                .iter()
                .map(display_source_type)
                .collect::<Result<Vec<_>, _>>()?
                .join(",")
        )),
        Type::Reference(_, _) => Err(
            "generic struct applications require recursive finite CopyData arguments".to_string(),
        ),
        Type::Generic(name, arguments) => Ok(format!(
            "{name}<{}>",
            arguments
                .iter()
                .map(display_source_type)
                .collect::<Result<Vec<_>, _>>()?
                .join(",")
        )),
    }
}

fn private_name_for(canonical: &str, fields: &[FieldDecl]) -> Result<String, String> {
    let schema = fields
        .iter()
        .map(|field| display_source_type(&field.field_type))
        .collect::<Result<Vec<_>, _>>()?
        .join(",");
    let payload = format!("{canonical}|{schema}");
    let encoded = payload
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(format!("{PRIVATE_GENERIC_STRUCT_PREFIX}{encoded}"))
}

#[cfg(test)]
pub(crate) fn private_name_for_test(canonical: &str, fields: &[Type]) -> String {
    let fields = fields
        .iter()
        .enumerate()
        .map(|(index, field_type)| FieldDecl {
            name: format!("field_{index}"),
            field_type: field_type.clone(),
        })
        .collect::<Vec<_>>();
    private_name_for(canonical, &fields).expect("test schema must be canonical CopyData")
}

fn decode_private_payload(name: &str) -> Option<(String, String)> {
    let encoded = name.strip_prefix(PRIVATE_GENERIC_STRUCT_PREFIX)?;
    if encoded.is_empty() || !encoded.len().is_multiple_of(2) {
        return None;
    }
    let mut bytes = Vec::with_capacity(encoded.len() / 2);
    for index in (0..encoded.len()).step_by(2) {
        bytes.push(u8::from_str_radix(&encoded[index..index + 2], 16).ok()?);
    }
    let payload = String::from_utf8(bytes).ok()?;
    let (source, schema) = payload.split_once('|')?;
    if source.is_empty() || schema.is_empty() || schema.contains('|') {
        return None;
    }
    Some((source.to_string(), schema.to_string()))
}

fn annotation_matches_logical(annotation: &Type, logical: &LogicalType) -> bool {
    if let Type::Named(name) = annotation {
        if let Some(primitive) = PrimitiveKind::from_source_name(name) {
            return PrimitiveKind::from_logical_type(logical) == Some(primitive);
        }
    }
    match (annotation, logical) {
        (Type::Named(expected), LogicalType::Struct { name, .. }) => {
            private_generic_struct_source_name(name)
                .as_deref()
                .unwrap_or(name)
                == expected
        }
        (Type::Generic(_, _), LogicalType::Struct { name, .. }) => {
            display_source_type(annotation).ok().as_deref()
                == private_generic_struct_source_name(name)
                    .as_deref()
                    .or(Some(name.as_str()))
        }
        (
            Type::Array(expected, expected_count),
            LogicalType::Array {
                element,
                count: actual_count,
            },
        ) => expected_count == actual_count && annotation_matches_logical(expected, element),
        (Type::Tuple(expected), LogicalType::Tuple { elements: actual }) => {
            expected.len() == actual.len()
                && expected
                    .iter()
                    .zip(actual)
                    .all(|(expected, actual)| annotation_matches_logical(expected, actual))
        }
        _ => false,
    }
}

fn valid_source_symbol(name: &str) -> bool {
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

struct CanonicalTypeParser<'a> {
    source: &'a [u8],
    position: usize,
}

impl<'a> CanonicalTypeParser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source: source.as_bytes(),
            position: 0,
        }
    }

    fn parse_complete(mut self) -> Result<Type, String> {
        let ty = self.parse_type()?;
        if self.position != self.source.len() {
            return Err("invalid canonical generic-struct type".to_string());
        }
        Ok(ty)
    }

    fn parse_type_list(mut self) -> Result<Vec<Type>, String> {
        let mut types = vec![self.parse_type()?];
        while self.peek() == Some(b',') {
            self.position += 1;
            types.push(self.parse_type()?);
        }
        if self.position != self.source.len() {
            return Err("invalid canonical generic-struct schema".to_string());
        }
        Ok(types)
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        match self.peek() {
            Some(b'[') => self.parse_array(),
            Some(b'(') => self.parse_tuple(),
            Some(_) => self.parse_named_or_generic(),
            None => Err("incomplete canonical generic-struct type".to_string()),
        }
    }

    fn parse_array(&mut self) -> Result<Type, String> {
        self.expect(b'[')?;
        let element = self.parse_type()?;
        self.expect(b';')?;
        let count = self.parse_digits()?;
        self.expect(b']')?;
        Ok(Type::Array(Box::new(element), count))
    }

    fn parse_tuple(&mut self) -> Result<Type, String> {
        self.expect(b'(')?;
        let mut elements = vec![self.parse_type()?];
        while self.peek() == Some(b',') {
            self.position += 1;
            elements.push(self.parse_type()?);
        }
        self.expect(b')')?;
        if elements.len() < 2 {
            return Err("canonical tuples require at least two elements".to_string());
        }
        Ok(Type::Tuple(elements))
    }

    fn parse_named_or_generic(&mut self) -> Result<Type, String> {
        let start = self.position;
        while self
            .peek()
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            self.position += 1;
        }
        if start == self.position {
            return Err("invalid canonical type name".to_string());
        }
        let name = std::str::from_utf8(&self.source[start..self.position])
            .map_err(|_| "invalid UTF-8 canonical type name".to_string())?
            .to_string();
        if self.peek() != Some(b'<') {
            return Ok(Type::Named(name));
        }
        self.position += 1;
        let mut arguments = vec![self.parse_type()?];
        while self.peek() == Some(b',') {
            self.position += 1;
            arguments.push(self.parse_type()?);
        }
        self.expect(b'>')?;
        Ok(Type::Generic(name, arguments))
    }

    fn parse_digits(&mut self) -> Result<usize, String> {
        let start = self.position;
        while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.position += 1;
        }
        if start == self.position {
            return Err("missing canonical array count".to_string());
        }
        std::str::from_utf8(&self.source[start..self.position])
            .map_err(|_| "invalid canonical array count".to_string())?
            .parse()
            .map_err(|_| "invalid canonical array count".to_string())
    }

    fn expect(&mut self, expected: u8) -> Result<(), String> {
        if self.peek() != Some(expected) {
            return Err("invalid canonical generic-struct type".to_string());
        }
        self.position += 1;
        Ok(())
    }

    fn peek(&self) -> Option<u8> {
        self.source.get(self.position).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading_definition() -> AstNode {
        AstNode::Statement(Statement::StructDef {
            name: "Reading".to_string(),
            fields: vec![
                FieldDecl {
                    name: "value".to_string(),
                    field_type: Type::Named("T".to_string()),
                },
                FieldDecl {
                    name: "valid".to_string(),
                    field_type: Type::Named("bool".to_string()),
                },
            ],
            type_params: vec!["T".to_string()],
        })
    }

    fn reading_application() -> AstNode {
        AstNode::Statement(Statement::Function {
            name: "read".to_string(),
            parameters: vec![],
            return_type: Some(Type::Generic(
                "Reading".to_string(),
                vec![Type::Named("int".to_string())],
            )),
            body: Block {
                statements: vec![Statement::Return(Some(Expression::StructLiteral {
                    name: "Reading".to_string(),
                    fields: vec![
                        ("value".to_string(), Expression::IntegerLiteral(7)),
                        (
                            "valid".to_string(),
                            Expression::Comparison {
                                op: crate::ast::ComparisonOp::LessThan,
                                left: Box::new(Expression::IntegerLiteral(1)),
                                right: Box::new(Expression::IntegerLiteral(2)),
                            },
                        ),
                    ],
                }))],
                expression: None,
            },
            type_params: vec![],
            trait_bounds: vec![],
        })
    }

    #[test]
    fn private_identity_round_trips_canonical_source_type() {
        let canonical = "Reading<(int,[char;2])>";
        let fields = vec![FieldDecl {
            name: "value".to_string(),
            field_type: Type::Tuple(vec![
                Type::Named("int".to_string()),
                Type::Array(Box::new(Type::Named("char".to_string())), 2),
            ]),
        }];
        let private = private_name_for(canonical, &fields).expect("canonical schema");
        assert_eq!(
            private_generic_struct_source_name(&private).as_deref(),
            Some(canonical)
        );
        assert!(valid_generic_aware_struct_symbol(
            &private,
            valid_source_symbol
        ));
    }

    #[test]
    fn private_identity_rejects_noncanonical_or_nongeneric_payloads() {
        assert!(private_generic_struct_source_name(PRIVATE_GENERIC_STRUCT_PREFIX).is_none());
        assert!(
            private_generic_struct_source_name("__aero$generic_struct$696e747c696e74").is_none()
        );
        assert!(private_generic_struct_source_name("__aero$generic_struct$zz").is_none());
    }

    #[test]
    fn normalization_is_idempotent_and_rejects_private_schema_corruption() {
        let first =
            normalize_generic_copydata_structs(vec![reading_definition(), reading_application()])
                .expect("first normalization");
        let second = normalize_generic_copydata_structs(first.clone())
            .expect("semantic output must be admitted idempotently");
        assert_eq!(format!("{first:?}"), format!("{second:?}"));

        let mut corrupt = first;
        let private = corrupt
            .iter_mut()
            .find_map(|node| match node {
                AstNode::Statement(Statement::StructDef {
                    name,
                    fields,
                    type_params,
                }) if name.starts_with(PRIVATE_GENERIC_STRUCT_PREFIX) => {
                    Some((name.clone(), fields, type_params))
                }
                _ => None,
            })
            .expect("normalized private definition");
        private.1[0].field_type = Type::Named("char".to_string());
        let error = normalize_generic_copydata_structs(corrupt)
            .expect_err("corrupted private schema must fail closed");
        assert!(
            error.contains("does not match its exact schema"),
            "unexpected corruption diagnostic: {error}"
        );
    }

    #[test]
    fn private_identity_commits_to_checked_logical_schema() {
        let fields = vec![
            FieldDecl {
                name: "value".to_string(),
                field_type: Type::Named("int".to_string()),
            },
            FieldDecl {
                name: "valid".to_string(),
                field_type: Type::Named("bool".to_string()),
            },
        ];
        let private = private_name_for("Reading<int>", &fields).expect("private identity");
        assert!(valid_generic_struct_schema(
            &private,
            &[LogicalType::Int, LogicalType::Bool]
        ));
        assert!(!valid_generic_struct_schema(
            &private,
            &[LogicalType::Char, LogicalType::Bool]
        ));
        assert!(!valid_generic_struct_schema(&private, &[LogicalType::Int]));
    }
}
