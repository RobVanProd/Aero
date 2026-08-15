use crate::ast::{
    AstNode, BinaryOp, Block, ComparisonOp, Expression, ImportSyntax, LogicalOp, MatchArm, Pattern,
    Statement, Type, UnaryOp, VariantDecl, VariantDeclKind,
};
use crate::builtin_carrier_contract::private_carrier_source_name;
use crate::enum_match_contract::{EnumExecutionContext, EnumRegistry};
use crate::generic_enum_contract::{private_generic_enum_source_name, valid_generic_enum_schema};
use crate::generic_function_contract::private_generic_function_source_name;
use crate::generic_struct_contract::{
    private_generic_struct_source_name, valid_generic_struct_schema,
};
use crate::ir::{EnumVariantSchema, LogicalType};
use crate::language_profile::ProfileTypeUse;
use crate::struct_contract::{StructExecutionContext, StructRegistry};
use crate::types::Ty;
use std::collections::{BTreeMap, BTreeSet};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ResolvedProfileShapeId(pub(crate) usize);

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolvedProfileResolution {
    Resolved(ResolvedProfileShapeId),
    Excluded(Option<ResolvedProfileShapeId>),
    Unresolved,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolvedProfileOrigin {
    Source {
        normalized: String,
    },
    ImplMethod {
        type_name: String,
        trait_name: Option<String>,
        method: String,
    },
    TraitMethod {
        trait_name: String,
        method: String,
    },
    SourceGenericStruct {
        normalized: String,
    },
    SourceGenericEnum {
        normalized: String,
    },
    SourceGenericFunction {
        normalized: String,
    },
    GenericStruct {
        normalized: String,
        source: String,
    },
    GenericEnum {
        normalized: String,
        source: String,
    },
    GenericFunction {
        normalized: String,
        source: String,
    },
    BuiltinCarrier {
        normalized: String,
        source: String,
    },
    OpaquePrivate {
        normalized: String,
    },
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedProfileField {
    pub(crate) name: String,
    pub(crate) resolution: ResolvedProfileResolution,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedProfileVariant {
    pub(crate) name: String,
    pub(crate) payload: Option<ResolvedProfileResolution>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolvedProfileNominal {
    Struct {
        origin: ResolvedProfileOrigin,
        resolution: ResolvedProfileResolution,
        fields: Vec<ResolvedProfileField>,
    },
    Enum {
        origin: ResolvedProfileOrigin,
        resolution: ResolvedProfileResolution,
        variants: Vec<ResolvedProfileVariant>,
    },
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedProfileUse {
    pub(crate) role: ProfileTypeUse,
    pub(crate) function: Option<ResolvedProfileOrigin>,
    pub(crate) name: Option<String>,
    pub(crate) resolution: ResolvedProfileResolution,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolvedProfileOperation {
    Declaration {
        origin: ResolvedProfileOrigin,
        resolution: ResolvedProfileResolution,
    },
    StructConstruction {
        function: Option<ResolvedProfileOrigin>,
        origin: ResolvedProfileOrigin,
        resolution: ResolvedProfileResolution,
        source_to_declaration: Vec<usize>,
    },
    EnumConstruction {
        function: Option<ResolvedProfileOrigin>,
        origin: ResolvedProfileOrigin,
        variant: String,
        resolution: ResolvedProfileResolution,
        variant_index: Option<usize>,
    },
    ExhaustiveMatch {
        function: Option<ResolvedProfileOrigin>,
        origin: Option<ResolvedProfileOrigin>,
        resolution: ResolvedProfileResolution,
        arm_for_variant: Vec<usize>,
        result: Option<ResolvedProfileResolution>,
    },
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolvedProfileSurfaceContext {
    FileScope,
    Function(ResolvedProfileOrigin),
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolvedProfileSurfaceObservation {
    Statement {
        context: ResolvedProfileSurfaceContext,
        kind: ResolvedProfileStatementKind,
    },
    Expression {
        context: ResolvedProfileSurfaceContext,
        kind: ResolvedProfileExpressionKind,
    },
    Pattern {
        context: ResolvedProfileSurfaceContext,
        kind: ResolvedProfilePatternKind,
    },
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolvedProfileStatementKind {
    Const,
    Let {
        mutable: bool,
        annotated: bool,
        initialized: bool,
    },
    Assignment {
        target: ResolvedProfileAssignmentTarget,
    },
    Return {
        has_value: bool,
    },
    Expression,
    Block,
    Function {
        top_level: bool,
        generic: bool,
        trait_bounded: bool,
        explicit_result: bool,
    },
    If {
        has_else: bool,
    },
    While,
    For,
    Loop,
    Break,
    Continue,
    StructDefinition {
        generic: bool,
    },
    EnumDefinition {
        generic: bool,
        trait_bounded: bool,
    },
    ImplBlock {
        generic: bool,
        trait_impl: bool,
    },
    TraitDefinition {
        generic: bool,
    },
    ModuleDeclaration {
        public: bool,
    },
    UseImport {
        founding_syntax: bool,
        aliased: bool,
    },
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolvedProfileExpressionKind {
    IntegerLiteral,
    FloatLiteral,
    CharacterLiteral,
    StringLiteral,
    Identifier,
    Binary(ResolvedProfileBinaryOperator),
    FunctionCall,
    MethodCall,
    Print,
    Println,
    Comparison(ResolvedProfileComparisonOperator),
    Logical(ResolvedProfileLogicalOperator),
    Unary(ResolvedProfileUnaryOperator),
    ArrayLiteral,
    ArrayRepeat,
    IndexAccess,
    FieldAccess,
    TupleLiteral,
    TupleIndex,
    StructLiteral,
    EnumVariant { parenthesized: bool },
    Match,
    Borrow { mutable: bool },
    Dereference,
    Closure,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResolvedProfileBinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResolvedProfileComparisonOperator {
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResolvedProfileLogicalOperator {
    And,
    Or,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResolvedProfileUnaryOperator {
    Not,
    Negate,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolvedProfilePatternKind {
    Wildcard,
    Literal,
    Identifier,
    Tuple,
    Struct,
    Enum { parenthesized: bool },
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedProfileAssignmentTarget {
    pub(crate) root: ResolvedProfileAssignmentRoot,
    pub(crate) projections: Vec<ResolvedProfileAssignmentProjection>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResolvedProfileAssignmentRoot {
    Identifier,
    Other,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ResolvedProfileAssignmentProjection {
    Field,
    Index,
    Dereference,
}

/// Immutable logical facts produced after successful semantic analysis.
///
/// This is deliberately not a profile decision and owns no physical layout.
/// CAP-028 carries it out of band so a later task can add one source-policy
/// consumer without rebuilding semantic registries or re-inferring expressions.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedProfileProgram {
    pub(crate) shapes: Vec<LogicalType>,
    pub(crate) nominals: Vec<ResolvedProfileNominal>,
    pub(crate) uses: Vec<ResolvedProfileUse>,
    pub(crate) operations: Vec<ResolvedProfileOperation>,
    pub(crate) surface: Vec<ResolvedProfileSurfaceObservation>,
}

impl ResolvedProfileProgram {
    pub(crate) fn from_semantic_success<F>(
        ast: &[AstNode],
        structs: &StructRegistry,
        enums: &EnumRegistry,
        admitted_function: F,
    ) -> Self
    where
        F: Fn(&str) -> Option<(Vec<(String, Ty)>, Ty)>,
    {
        Builder::new(structs, enums, &admitted_function).build(ast)
    }
}

struct Builder<'a, F>
where
    F: Fn(&str) -> Option<(Vec<(String, Ty)>, Ty)>,
{
    structs: &'a StructRegistry,
    enums: &'a EnumRegistry,
    admitted_function: &'a F,
    shape_ids: BTreeMap<LogicalType, ResolvedProfileShapeId>,
    visiting_shapes: BTreeSet<LogicalType>,
    shapes: Vec<LogicalType>,
    nominals: Vec<ResolvedProfileNominal>,
    uses: Vec<ResolvedProfileUse>,
    operations: Vec<ResolvedProfileOperation>,
    surface: Vec<ResolvedProfileSurfaceObservation>,
    validated_carriers: BTreeSet<String>,
    binding_scopes: Vec<BTreeMap<String, Option<ResolvedProfileResolution>>>,
    function: Option<ResolvedProfileOrigin>,
    function_result: Option<ResolvedProfileResolution>,
    admitted_context: bool,
    profile_context: bool,
}

impl<'a, F> Builder<'a, F>
where
    F: Fn(&str) -> Option<(Vec<(String, Ty)>, Ty)>,
{
    fn new(structs: &'a StructRegistry, enums: &'a EnumRegistry, admitted_function: &'a F) -> Self {
        Self {
            structs,
            enums,
            admitted_function,
            shape_ids: BTreeMap::new(),
            visiting_shapes: BTreeSet::new(),
            shapes: Vec::new(),
            nominals: Vec::new(),
            uses: Vec::new(),
            operations: Vec::new(),
            surface: Vec::new(),
            validated_carriers: BTreeSet::new(),
            binding_scopes: vec![BTreeMap::new()],
            function: None,
            function_result: None,
            admitted_context: false,
            profile_context: false,
        }
    }

    fn build(mut self, ast: &[AstNode]) -> ResolvedProfileProgram {
        for node in ast {
            self.record_declaration(node);
            self.walk_node(node);
        }
        ResolvedProfileProgram {
            shapes: self.shapes,
            nominals: self.nominals,
            uses: self.uses,
            operations: self.operations,
            surface: self.surface,
        }
    }

    fn record_declaration(&mut self, node: &AstNode) {
        let AstNode::Statement(statement) = node else {
            return;
        };
        match statement {
            Statement::StructDef {
                name, type_params, ..
            } => {
                let origin = struct_origin(name, !type_params.is_empty());
                let (resolution, fields) = if !type_params.is_empty() {
                    (ResolvedProfileResolution::Excluded(None), Vec::new())
                } else if let Some(contract) =
                    self.structs.copy_struct_contract(&Ty::Struct(name.clone()))
                {
                    let logical = contract.logical_type();
                    let valid = valid_generic_struct_schema(
                        name,
                        match &logical {
                            LogicalType::Struct { fields, .. } => fields,
                            _ => unreachable!("a struct contract has struct logical identity"),
                        },
                    );
                    if valid {
                        let resolution = self.resolve_logical(logical);
                        let fields = contract
                            .fields
                            .into_iter()
                            .map(|field| {
                                let resolution = self.resolve_logical(field.logical_type());
                                ResolvedProfileField {
                                    name: field.name,
                                    resolution,
                                }
                            })
                            .collect();
                        (resolution, fields)
                    } else {
                        (ResolvedProfileResolution::Unresolved, Vec::new())
                    }
                } else {
                    (ResolvedProfileResolution::Unresolved, Vec::new())
                };
                self.operations.push(ResolvedProfileOperation::Declaration {
                    origin: origin.clone(),
                    resolution: resolution.clone(),
                });
                self.nominals.push(ResolvedProfileNominal::Struct {
                    origin,
                    resolution,
                    fields,
                });
            }
            Statement::EnumDef {
                name,
                variants,
                type_params,
                ..
            } => {
                let origin = enum_origin(name, !type_params.is_empty());
                let (resolution, resolved_variants) = if !type_params.is_empty() {
                    (ResolvedProfileResolution::Excluded(None), Vec::new())
                } else if let Ok(logical) = self.enums.owned_place_logical_type(name) {
                    let LogicalType::Enum {
                        variants: schema, ..
                    } = &logical
                    else {
                        unreachable!("an enum contract has enum logical identity")
                    };
                    if self.valid_enum_declaration(name, variants, schema) {
                        if private_carrier_source_name(name).is_some() {
                            self.validated_carriers.insert(name.clone());
                        }
                        let schema = schema.clone();
                        let resolution = self.resolve_logical(logical);
                        let resolved_variants = schema
                            .into_iter()
                            .map(|variant| ResolvedProfileVariant {
                                name: variant.name,
                                payload: variant
                                    .payload
                                    .map(|payload| self.resolve_logical(payload)),
                            })
                            .collect();
                        (resolution, resolved_variants)
                    } else {
                        (ResolvedProfileResolution::Unresolved, Vec::new())
                    }
                } else {
                    (ResolvedProfileResolution::Unresolved, Vec::new())
                };
                self.operations.push(ResolvedProfileOperation::Declaration {
                    origin: origin.clone(),
                    resolution: resolution.clone(),
                });
                self.nominals.push(ResolvedProfileNominal::Enum {
                    origin,
                    resolution,
                    variants: resolved_variants,
                });
            }
            _ => {}
        }
    }

    fn valid_enum_declaration(
        &self,
        name: &str,
        variants: &[VariantDecl],
        schema: &[EnumVariantSchema],
    ) -> bool {
        if private_generic_enum_source_name(name).is_some()
            && !valid_generic_enum_schema(name, schema)
        {
            return false;
        }
        let Some(source) = private_carrier_source_name(name) else {
            return true;
        };
        self.valid_carrier_declaration(&source, variants, schema)
    }

    fn valid_carrier_declaration(
        &self,
        source: &str,
        variants: &[VariantDecl],
        schema: &[EnumVariantSchema],
    ) -> bool {
        let resolved_payload = |variant: &VariantDecl| match &variant.kind {
            VariantDeclKind::Tuple(fields) if fields.len() == 1 => self
                .structs
                .resolve_copy_annotation(&fields[0])
                .map(|contract| (contract.ty, contract.logical_type)),
            _ => None,
        };
        match (variants, schema) {
            (
                [some, none],
                [
                    EnumVariantSchema {
                        name: schema_some,
                        payload: Some(schema_payload),
                    },
                    EnumVariantSchema {
                        name: schema_none,
                        payload: None,
                    },
                ],
            ) if source.starts_with("Option<")
                && some.name == "Some"
                && none.name == "None"
                && schema_some == "Some"
                && schema_none == "None" =>
            {
                resolved_payload(some).is_some_and(|(ty, logical)| {
                    logical == *schema_payload && source == format!("Option<{ty}>")
                })
            }
            (
                [ok, error],
                [
                    EnumVariantSchema {
                        name: schema_ok,
                        payload: Some(schema_ok_payload),
                    },
                    EnumVariantSchema {
                        name: schema_error,
                        payload: Some(schema_error_payload),
                    },
                ],
            ) if source.starts_with("Result<")
                && ok.name == "Ok"
                && error.name == "Err"
                && schema_ok == "Ok"
                && schema_error == "Err" =>
            {
                match (resolved_payload(ok), resolved_payload(error)) {
                    (Some((ok_ty, ok_logical)), Some((error_ty, error_logical))) => {
                        ok_logical == *schema_ok_payload
                            && error_logical == *schema_error_payload
                            && source == format!("Result<{ok_ty}, {error_ty}>")
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn walk_node(&mut self, node: &AstNode) {
        match node {
            AstNode::Statement(statement) => self.walk_statement(statement, true),
            AstNode::Expression(expression) => self.walk_expression(expression, None, false),
        }
    }

    fn surface_context(&self) -> ResolvedProfileSurfaceContext {
        self.function
            .clone()
            .map(ResolvedProfileSurfaceContext::Function)
            .unwrap_or(ResolvedProfileSurfaceContext::FileScope)
    }

    fn walk_statement(&mut self, statement: &Statement, is_top_level: bool) {
        let context = self.surface_context();
        self.surface
            .push(ResolvedProfileSurfaceObservation::Statement {
                context,
                kind: statement_kind(statement, is_top_level),
            });
        match statement {
            Statement::Const {
                name,
                type_annotation,
                value,
                ..
            } => {
                let resolution = self.resolve_annotation(type_annotation);
                let resolution = self.contextual_resolution(resolution);
                self.record_use(
                    ProfileTypeUse::Binding,
                    Some(name.clone()),
                    resolution.clone(),
                );
                self.walk_expression(value, Some(resolution.clone()), true);
                self.bind(name.clone(), Some(resolution));
            }
            Statement::Let {
                name,
                mutable,
                type_annotation,
                value,
            } => {
                let resolution = type_annotation
                    .as_ref()
                    .map(|annotation| self.resolve_annotation(annotation))
                    .map(|resolution| self.contextual_resolution(resolution));
                if let Some(resolution) = &resolution {
                    self.record_use(
                        if *mutable {
                            ProfileTypeUse::MutableBinding
                        } else {
                            ProfileTypeUse::Binding
                        },
                        Some(name.clone()),
                        resolution.clone(),
                    );
                }
                if let Some(value) = value {
                    self.walk_expression(value, resolution.clone(), resolution.is_some());
                }
                self.bind(name.clone(), resolution);
            }
            Statement::Assignment { target, value } => {
                let resolution = match target {
                    Expression::Identifier(name) => self.lookup_binding(name),
                    _ => None,
                };
                if let Some(resolution) = &resolution {
                    let name = match target {
                        Expression::Identifier(name) => Some(name.clone()),
                        _ => None,
                    };
                    self.record_use(ProfileTypeUse::OwnedAssignment, name, resolution.clone());
                }
                self.walk_expression(target, None, false);
                self.walk_expression(value, resolution.clone(), resolution.is_some());
            }
            Statement::Return(value) => {
                if let Some(value) = value {
                    let expected = self.function_result.clone();
                    self.walk_expression(value, expected.clone(), expected.is_some());
                }
            }
            Statement::Expression(expression) => self.walk_expression(expression, None, false),
            Statement::Block(block) => self.walk_block(block, None),
            Statement::Function {
                name,
                parameters,
                return_type,
                body,
                type_params,
                ..
            } => self.walk_function(
                name,
                parameters,
                return_type.as_ref(),
                type_params,
                body,
                is_top_level,
            ),
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.walk_expression(condition, None, false);
                self.walk_block(then_block, None);
                if let Some(else_block) = else_block {
                    self.walk_statement(else_block, false);
                }
            }
            Statement::While { condition, body } => {
                self.walk_expression(condition, None, false);
                self.walk_block(body, None);
            }
            Statement::For {
                variable,
                iterable,
                body,
            } => {
                self.walk_expression(iterable, None, false);
                self.push_scope();
                self.bind(variable.clone(), None);
                self.walk_block(body, None);
                self.pop_scope();
            }
            Statement::Loop { body } => self.walk_block(body, None),
            Statement::ImplBlock {
                type_name,
                methods,
                trait_name,
                ..
            } => {
                for method in methods {
                    if let Statement::Function {
                        name,
                        parameters,
                        return_type,
                        body,
                        type_params,
                        trait_bounds,
                    } = method
                    {
                        let context = self.surface_context();
                        self.surface
                            .push(ResolvedProfileSurfaceObservation::Statement {
                                context,
                                kind: ResolvedProfileStatementKind::Function {
                                    top_level: false,
                                    generic: !type_params.is_empty(),
                                    trait_bounded: !trait_bounds.is_empty(),
                                    explicit_result: return_type.is_some(),
                                },
                            });
                        self.walk_preserved_method(
                            ResolvedProfileOrigin::ImplMethod {
                                type_name: type_name.clone(),
                                trait_name: trait_name.clone(),
                                method: name.clone(),
                            },
                            parameters,
                            return_type.as_ref(),
                            Some(body),
                        );
                    }
                }
            }
            Statement::TraitDef { name, methods, .. } => {
                for method in methods {
                    let context = self.surface_context();
                    self.surface
                        .push(ResolvedProfileSurfaceObservation::Statement {
                            context,
                            kind: ResolvedProfileStatementKind::Function {
                                top_level: false,
                                generic: false,
                                trait_bounded: false,
                                explicit_result: method.return_type.is_some(),
                            },
                        });
                    self.walk_preserved_method(
                        ResolvedProfileOrigin::TraitMethod {
                            trait_name: name.clone(),
                            method: method.name.clone(),
                        },
                        &method.parameters,
                        method.return_type.as_ref(),
                        method.body.as_ref(),
                    );
                }
            }
            Statement::StructDef { .. }
            | Statement::EnumDef { .. }
            | Statement::Break
            | Statement::Continue
            | Statement::ModDecl { .. }
            | Statement::UseImport { .. } => {}
        }
    }

    fn walk_function(
        &mut self,
        name: &str,
        parameters: &[crate::ast::Parameter],
        return_type: Option<&Type>,
        type_params: &[String],
        body: &Block,
        is_top_level: bool,
    ) {
        let saved_function = self.function.clone();
        let saved_result = self.function_result.clone();
        let saved_context = self.admitted_context;
        let saved_profile_context = self.profile_context;
        let function = function_origin(name, !type_params.is_empty());
        let admitted = is_top_level
            .then(|| (self.admitted_function)(name))
            .flatten();
        let has_admitted_contract = admitted.is_some();
        self.function = Some(function.clone());
        self.admitted_context = is_top_level && type_params.is_empty();
        self.profile_context = self.admitted_context
            && has_admitted_contract
            && matches!(function, ResolvedProfileOrigin::Source { .. });
        self.push_scope();

        let (parameter_resolutions, result_resolution) = if let Some((resolved, result)) = admitted
        {
            let parameters = resolved
                .into_iter()
                .map(|(name, ty)| {
                    let resolution = self.resolve_ty(&ty);
                    (name, self.contextual_resolution(resolution))
                })
                .collect::<Vec<_>>();
            let result = self.resolve_ty(&result);
            (parameters, self.contextual_resolution(result))
        } else {
            let parameters = parameters
                .iter()
                .map(|parameter| {
                    (
                        parameter.name.clone(),
                        force_excluded(self.resolve_annotation(&parameter.param_type)),
                    )
                })
                .collect::<Vec<_>>();
            let result = force_excluded(match return_type {
                Some(annotation) => self.resolve_annotation(annotation),
                None => self.resolve_logical(LogicalType::Void),
            });
            (parameters, result)
        };

        for (parameter, resolution) in parameter_resolutions {
            self.record_use(
                ProfileTypeUse::Parameter,
                Some(parameter.clone()),
                resolution.clone(),
            );
            self.bind(parameter, Some(resolution));
        }
        let explicit_result = return_type.is_some().then_some(result_resolution);
        if let Some(resolution) = &explicit_result {
            self.record_use(ProfileTypeUse::Result, None, resolution.clone());
        }
        self.function_result = explicit_result.clone();
        self.walk_block(body, explicit_result);

        self.pop_scope();
        self.function = saved_function;
        self.function_result = saved_result;
        self.admitted_context = saved_context;
        self.profile_context = saved_profile_context;
    }

    fn walk_preserved_method(
        &mut self,
        origin: ResolvedProfileOrigin,
        parameters: &[crate::ast::Parameter],
        return_type: Option<&Type>,
        body: Option<&Block>,
    ) {
        let saved_function = self.function.clone();
        let saved_result = self.function_result.clone();
        let saved_context = self.admitted_context;
        let saved_profile_context = self.profile_context;
        self.function = Some(origin);
        self.function_result = None;
        self.admitted_context = false;
        self.profile_context = false;
        self.push_scope();

        for parameter in parameters {
            let resolution = force_excluded(self.resolve_annotation(&parameter.param_type));
            self.record_use(
                ProfileTypeUse::Parameter,
                Some(parameter.name.clone()),
                resolution.clone(),
            );
            self.bind(parameter.name.clone(), Some(resolution));
        }
        let result = return_type.map(|annotation| {
            let resolution = force_excluded(self.resolve_annotation(annotation));
            self.record_use(ProfileTypeUse::Result, None, resolution.clone());
            resolution
        });
        self.function_result = result.clone();
        if let Some(body) = body {
            self.walk_block(body, result);
        }

        self.pop_scope();
        self.function = saved_function;
        self.function_result = saved_result;
        self.admitted_context = saved_context;
        self.profile_context = saved_profile_context;
    }

    fn walk_block(&mut self, block: &Block, tail_expected: Option<ResolvedProfileResolution>) {
        self.push_scope();
        for statement in &block.statements {
            self.walk_statement(statement, false);
        }
        if let Some(expression) = &block.expression {
            self.walk_expression(expression, tail_expected.clone(), tail_expected.is_some());
        }
        self.pop_scope();
    }

    fn walk_expression(
        &mut self,
        expression: &Expression,
        expected: Option<ResolvedProfileResolution>,
        record_expected_value: bool,
    ) {
        let context = self.surface_context();
        self.surface
            .push(ResolvedProfileSurfaceObservation::Expression {
                context,
                kind: expression_kind(expression),
            });
        if record_expected_value {
            if let Some(expected) = &expected {
                self.record_use(ProfileTypeUse::Value, None, expected.clone());
            }
        }
        match expression {
            Expression::IntegerLiteral(_)
            | Expression::FloatLiteral(_)
            | Expression::CharacterLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::Identifier(_) => {}
            Expression::Binary { left, right, .. }
            | Expression::Comparison { left, right, .. }
            | Expression::Logical { left, right, .. } => {
                self.walk_expression(left, None, false);
                self.walk_expression(right, None, false);
            }
            Expression::FunctionCall { name, arguments } => {
                let contract = (self.admitted_function)(name);
                let source_callee = matches!(
                    function_origin(name, false),
                    ResolvedProfileOrigin::Source { .. }
                );
                let expected = contract
                    .map(|(parameters, _)| parameters)
                    .unwrap_or_default();
                let expected_count = expected.len();
                for (argument, (_, ty)) in arguments.iter().zip(expected) {
                    let resolution = self.resolve_ty(&ty);
                    let resolution = if self.profile_context && source_callee {
                        resolution
                    } else {
                        force_excluded(resolution)
                    };
                    self.walk_expression(argument, Some(resolution), true);
                }
                for argument in arguments.iter().skip(expected_count) {
                    self.walk_expression(argument, None, false);
                }
            }
            Expression::Print { arguments, .. } | Expression::Println { arguments, .. } => {
                for argument in arguments {
                    self.walk_expression(argument, None, false);
                }
            }
            Expression::MethodCall {
                object, arguments, ..
            } => {
                self.walk_expression(object, None, false);
                for argument in arguments {
                    self.walk_expression(argument, None, false);
                }
            }
            Expression::Unary { operand, .. }
            | Expression::Borrow { expr: operand, .. }
            | Expression::Deref(operand)
            | Expression::FieldAccess {
                object: operand, ..
            } => self.walk_expression(operand, None, false),
            Expression::ArrayLiteral(elements) | Expression::TupleLiteral(elements) => {
                for element in elements {
                    self.walk_expression(element, None, false);
                }
            }
            Expression::ArrayRepeat { value, .. } => {
                self.walk_expression(value, None, false);
            }
            Expression::IndexAccess { object, index } => {
                self.walk_expression(object, None, false);
                self.walk_expression(index, None, false);
            }
            Expression::TupleIndex { object, .. } => {
                self.walk_expression(object, None, false);
            }
            Expression::StructLiteral { name, fields } => {
                self.walk_struct_construction(name, fields, record_expected_value);
            }
            Expression::EnumVariant {
                enum_name,
                variant,
                data,
            } => self.walk_enum_construction(
                enum_name,
                variant,
                data.as_deref(),
                record_expected_value,
            ),
            Expression::Match { expr, arms } => {
                self.walk_match(expr, arms, expected);
            }
            Expression::Closure { body, .. } => {
                let saved = self.admitted_context;
                let saved_profile = self.profile_context;
                self.admitted_context = false;
                self.profile_context = false;
                self.walk_expression(body, None, false);
                self.admitted_context = saved;
                self.profile_context = saved_profile;
            }
        }
    }

    fn walk_struct_construction(
        &mut self,
        name: &str,
        fields: &[(String, Expression)],
        already_recorded_value: bool,
    ) {
        let origin = struct_origin(name, false);
        let resolved = self.structs.resolve_construction(
            name,
            fields,
            if self.admitted_context {
                StructExecutionContext::AdmittedFunction
            } else {
                StructExecutionContext::PreservedContext
            },
        );
        let (resolution, source_to_declaration, expected_fields) = match resolved {
            Ok(resolved) => {
                let resolution = self.resolve_logical(resolved.contract.logical_type());
                let resolution = self.contextual_resolution(resolution);
                let expected = resolved
                    .source_to_declaration
                    .iter()
                    .map(|index| {
                        let resolution =
                            self.resolve_logical(resolved.contract.fields[*index].logical_type());
                        self.contextual_resolution(resolution)
                    })
                    .collect::<Vec<_>>();
                (resolution, resolved.source_to_declaration, expected)
            }
            Err(_) => (
                ResolvedProfileResolution::Unresolved,
                Vec::new(),
                Vec::new(),
            ),
        };
        if !already_recorded_value {
            self.record_use(ProfileTypeUse::Value, None, resolution.clone());
        }
        self.operations
            .push(ResolvedProfileOperation::StructConstruction {
                function: self.function.clone(),
                origin,
                resolution,
                source_to_declaration,
            });
        let expected_count = expected_fields.len();
        for ((_, value), expected) in fields.iter().zip(expected_fields) {
            self.walk_expression(value, Some(expected), true);
        }
        for (_, value) in fields.iter().skip(expected_count) {
            self.walk_expression(value, None, false);
        }
    }

    fn walk_enum_construction(
        &mut self,
        enum_name: &str,
        variant: &str,
        data: Option<&[Expression]>,
        already_recorded_value: bool,
    ) {
        let origin = enum_origin(enum_name, false);
        let resolved = self.enums.resolve_constructor(
            enum_name,
            variant,
            data.map(<[Expression]>::len),
            if self.admitted_context {
                EnumExecutionContext::AdmittedFunction
            } else {
                EnumExecutionContext::PreservedContext
            },
        );
        let (resolution, variant_index, expected_payloads) = match resolved {
            Ok(resolved) => {
                let logical = resolved.contract.schema.logical_type();
                let resolution = self.resolve_logical(logical);
                let resolution = self.contextual_resolution(resolution);
                let payloads = resolved.contract.schema.variants[resolved.variant_index]
                    .payload
                    .as_ref()
                    .map(payload_logical_fields)
                    .unwrap_or_default()
                    .into_iter()
                    .map(|payload| {
                        let resolution = self.resolve_logical(payload);
                        self.contextual_resolution(resolution)
                    })
                    .collect();
                (resolution, Some(resolved.variant_index), payloads)
            }
            Err(_) => (ResolvedProfileResolution::Unresolved, None, Vec::new()),
        };
        if !already_recorded_value {
            self.record_use(ProfileTypeUse::Value, None, resolution.clone());
        }
        self.operations
            .push(ResolvedProfileOperation::EnumConstruction {
                function: self.function.clone(),
                origin,
                variant: variant.to_string(),
                resolution,
                variant_index,
            });
        let data = data.unwrap_or_default();
        for (value, expected) in data.iter().zip(&expected_payloads) {
            self.walk_expression(value, Some(expected.clone()), true);
        }
        for value in data.iter().skip(expected_payloads.len()) {
            self.walk_expression(value, None, false);
        }
    }

    fn walk_match(
        &mut self,
        scrutinee: &Expression,
        arms: &[MatchArm],
        result: Option<ResolvedProfileResolution>,
    ) {
        self.walk_expression(scrutinee, None, false);
        let identity =
            exact_match_identity(arms).or_else(|| self.exact_scrutinee_enum_identity(scrutinee));
        let resolved = identity.as_ref().and_then(|name| {
            self.enums
                .resolve_match_patterns(
                    &Ty::Enum(name.clone()),
                    scrutinee,
                    arms,
                    if self.admitted_context {
                        EnumExecutionContext::AdmittedFunction
                    } else {
                        EnumExecutionContext::PreservedContext
                    },
                )
                .ok()
        });
        let (origin, resolution, arm_for_variant) = match (identity, resolved) {
            (Some(name), Some(resolved)) => (
                Some(enum_origin(&name, false)),
                {
                    let resolution = self.resolve_logical(resolved.contract.schema.logical_type());
                    self.contextual_resolution(resolution)
                },
                resolved.arm_for_variant,
            ),
            (Some(name), None) => (
                Some(enum_origin(&name, false)),
                ResolvedProfileResolution::Unresolved,
                Vec::new(),
            ),
            (None, _) => (None, ResolvedProfileResolution::Unresolved, Vec::new()),
        };
        self.operations
            .push(ResolvedProfileOperation::ExhaustiveMatch {
                function: self.function.clone(),
                origin,
                resolution,
                arm_for_variant,
                result: result.clone(),
            });
        for arm in arms {
            self.push_scope();
            self.walk_pattern(&arm.pattern);
            bind_pattern_names(&arm.pattern, |name| self.bind(name, None));
            self.walk_expression(&arm.body, result.clone(), result.is_some());
            self.pop_scope();
        }
    }

    fn walk_pattern(&mut self, pattern: &Pattern) {
        let context = self.surface_context();
        self.surface
            .push(ResolvedProfileSurfaceObservation::Pattern {
                context,
                kind: pattern_kind(pattern),
            });
        match pattern {
            Pattern::Literal(expression) => self.walk_expression(expression, None, false),
            Pattern::Tuple(patterns) => {
                for pattern in patterns {
                    self.walk_pattern(pattern);
                }
            }
            Pattern::Struct { fields, .. } => {
                for (_, pattern) in fields {
                    self.walk_pattern(pattern);
                }
            }
            Pattern::Enum { data, .. } => {
                for pattern in data.as_deref().unwrap_or_default() {
                    self.walk_pattern(pattern);
                }
            }
            Pattern::Wildcard | Pattern::Identifier(_) => {}
        }
    }

    fn resolve_annotation(&mut self, annotation: &Type) -> ResolvedProfileResolution {
        if matches!(annotation, Type::Reference(_, _) | Type::Generic(_, _)) {
            return ResolvedProfileResolution::Excluded(None);
        }
        if let Ok(Some(contract)) = self.enums.reference_annotation_type(annotation) {
            return self.resolve_logical(contract.logical_type);
        }
        self.structs
            .resolve_copy_annotation(annotation)
            .map(|contract| self.resolve_logical(contract.logical_type))
            .unwrap_or(ResolvedProfileResolution::Unresolved)
    }

    fn resolve_ty(&mut self, ty: &Ty) -> ResolvedProfileResolution {
        if *ty == Ty::Void {
            return self.resolve_logical(LogicalType::Void);
        }
        if let Ok(Some(contract)) = self.enums.reference_pointee_type(ty) {
            return self.resolve_logical(contract.logical_type);
        }
        self.structs
            .resolve_copy_type(ty)
            .map(|contract| self.resolve_logical(contract.logical_type))
            .unwrap_or_else(|| match ty {
                Ty::Reference(_, _)
                | Ty::TypeParam(_)
                | Ty::Option(_)
                | Ty::Result(_, _)
                | Ty::Vec(_)
                | Ty::HashMap(_, _)
                | Ty::Fn(_) => ResolvedProfileResolution::Excluded(None),
                _ => ResolvedProfileResolution::Unresolved,
            })
    }

    fn resolve_logical(&mut self, logical: LogicalType) -> ResolvedProfileResolution {
        let candidate = match &logical {
            LogicalType::Enum { name, .. } if private_carrier_source_name(name).is_some() => {
                self.validated_carriers.contains(name) && candidate_shape(&logical)
            }
            _ => candidate_shape(&logical),
        };
        match self.intern_shape(logical) {
            Some(id) if candidate => ResolvedProfileResolution::Resolved(id),
            Some(id) => ResolvedProfileResolution::Excluded(Some(id)),
            None => ResolvedProfileResolution::Excluded(None),
        }
    }

    fn contextual_resolution(
        &self,
        resolution: ResolvedProfileResolution,
    ) -> ResolvedProfileResolution {
        if self.profile_context {
            resolution
        } else {
            force_excluded(resolution)
        }
    }

    fn exact_scrutinee_enum_identity(&self, scrutinee: &Expression) -> Option<String> {
        match scrutinee {
            Expression::Identifier(name) => self
                .lookup_binding(name)
                .as_ref()
                .and_then(|resolution| self.enum_name_for_resolution(resolution)),
            Expression::EnumVariant { enum_name, .. } => Some(enum_name.clone()),
            Expression::FunctionCall { name, .. } => {
                let (_, result) = (self.admitted_function)(name)?;
                match result {
                    Ty::Enum(name) => Some(name),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn enum_name_for_resolution(&self, resolution: &ResolvedProfileResolution) -> Option<String> {
        let id = match resolution {
            ResolvedProfileResolution::Resolved(id)
            | ResolvedProfileResolution::Excluded(Some(id)) => *id,
            ResolvedProfileResolution::Excluded(None) | ResolvedProfileResolution::Unresolved => {
                return None;
            }
        };
        match self.shapes.get(id.0) {
            Some(LogicalType::Enum { name, .. }) => Some(name.clone()),
            _ => None,
        }
    }

    fn intern_shape(&mut self, logical: LogicalType) -> Option<ResolvedProfileShapeId> {
        if let Some(id) = self.shape_ids.get(&logical) {
            return Some(*id);
        }
        if !self.visiting_shapes.insert(logical.clone()) {
            return None;
        }
        for child in logical_children(&logical) {
            if self.intern_shape(child.clone()).is_none() {
                self.visiting_shapes.remove(&logical);
                return None;
            }
        }
        self.visiting_shapes.remove(&logical);
        let id = ResolvedProfileShapeId(self.shapes.len());
        self.shapes.push(logical.clone());
        self.shape_ids.insert(logical, id);
        Some(id)
    }

    fn record_use(
        &mut self,
        role: ProfileTypeUse,
        name: Option<String>,
        resolution: ResolvedProfileResolution,
    ) {
        self.uses.push(ResolvedProfileUse {
            role,
            function: self.function.clone(),
            name,
            resolution,
        });
    }

    fn push_scope(&mut self) {
        self.binding_scopes.push(BTreeMap::new());
    }

    fn pop_scope(&mut self) {
        self.binding_scopes
            .pop()
            .expect("resolved profile binding scopes remain balanced");
    }

    fn bind(&mut self, name: String, resolution: Option<ResolvedProfileResolution>) {
        self.binding_scopes
            .last_mut()
            .expect("resolved profile binding scope is present")
            .insert(name, resolution);
    }

    fn lookup_binding(&self, name: &str) -> Option<ResolvedProfileResolution> {
        for scope in self.binding_scopes.iter().rev() {
            if let Some(resolution) = scope.get(name) {
                return resolution.clone();
            }
        }
        None
    }
}

fn statement_kind(statement: &Statement, is_top_level: bool) -> ResolvedProfileStatementKind {
    match statement {
        Statement::Const { .. } => ResolvedProfileStatementKind::Const,
        Statement::Let {
            mutable,
            type_annotation,
            value,
            ..
        } => ResolvedProfileStatementKind::Let {
            mutable: *mutable,
            annotated: type_annotation.is_some(),
            initialized: value.is_some(),
        },
        Statement::Assignment { target, .. } => ResolvedProfileStatementKind::Assignment {
            target: assignment_target(target),
        },
        Statement::Return(value) => ResolvedProfileStatementKind::Return {
            has_value: value.is_some(),
        },
        Statement::Expression(_) => ResolvedProfileStatementKind::Expression,
        Statement::Block(_) => ResolvedProfileStatementKind::Block,
        Statement::Function {
            return_type,
            type_params,
            trait_bounds,
            ..
        } => ResolvedProfileStatementKind::Function {
            top_level: is_top_level,
            generic: !type_params.is_empty(),
            trait_bounded: !trait_bounds.is_empty(),
            explicit_result: return_type.is_some(),
        },
        Statement::If { else_block, .. } => ResolvedProfileStatementKind::If {
            has_else: else_block.is_some(),
        },
        Statement::While { .. } => ResolvedProfileStatementKind::While,
        Statement::For { .. } => ResolvedProfileStatementKind::For,
        Statement::Loop { .. } => ResolvedProfileStatementKind::Loop,
        Statement::Break => ResolvedProfileStatementKind::Break,
        Statement::Continue => ResolvedProfileStatementKind::Continue,
        Statement::StructDef { type_params, .. } => {
            ResolvedProfileStatementKind::StructDefinition {
                generic: !type_params.is_empty(),
            }
        }
        Statement::EnumDef {
            type_params,
            trait_bounds,
            ..
        } => ResolvedProfileStatementKind::EnumDefinition {
            generic: !type_params.is_empty(),
            trait_bounded: !trait_bounds.is_empty(),
        },
        Statement::ImplBlock {
            type_params,
            trait_name,
            ..
        } => ResolvedProfileStatementKind::ImplBlock {
            generic: !type_params.is_empty(),
            trait_impl: trait_name.is_some(),
        },
        Statement::TraitDef { type_params, .. } => ResolvedProfileStatementKind::TraitDefinition {
            generic: !type_params.is_empty(),
        },
        Statement::ModDecl { is_public, .. } => {
            ResolvedProfileStatementKind::ModuleDeclaration { public: *is_public }
        }
        Statement::UseImport { syntax, alias, .. } => ResolvedProfileStatementKind::UseImport {
            founding_syntax: matches!(syntax, ImportSyntax::FoundingDottedImport),
            aliased: alias.is_some(),
        },
    }
}

fn expression_kind(expression: &Expression) -> ResolvedProfileExpressionKind {
    match expression {
        Expression::IntegerLiteral(_) => ResolvedProfileExpressionKind::IntegerLiteral,
        Expression::FloatLiteral(_) => ResolvedProfileExpressionKind::FloatLiteral,
        Expression::CharacterLiteral(_) => ResolvedProfileExpressionKind::CharacterLiteral,
        Expression::StringLiteral(_) => ResolvedProfileExpressionKind::StringLiteral,
        Expression::Identifier(_) => ResolvedProfileExpressionKind::Identifier,
        Expression::Binary { op, .. } => ResolvedProfileExpressionKind::Binary(binary_operator(op)),
        Expression::FunctionCall { .. } => ResolvedProfileExpressionKind::FunctionCall,
        Expression::MethodCall { .. } => ResolvedProfileExpressionKind::MethodCall,
        Expression::Print { .. } => ResolvedProfileExpressionKind::Print,
        Expression::Println { .. } => ResolvedProfileExpressionKind::Println,
        Expression::Comparison { op, .. } => {
            ResolvedProfileExpressionKind::Comparison(comparison_operator(op))
        }
        Expression::Logical { op, .. } => {
            ResolvedProfileExpressionKind::Logical(logical_operator(op))
        }
        Expression::Unary { op, .. } => ResolvedProfileExpressionKind::Unary(unary_operator(op)),
        Expression::ArrayLiteral(_) => ResolvedProfileExpressionKind::ArrayLiteral,
        Expression::ArrayRepeat { .. } => ResolvedProfileExpressionKind::ArrayRepeat,
        Expression::IndexAccess { .. } => ResolvedProfileExpressionKind::IndexAccess,
        Expression::FieldAccess { .. } => ResolvedProfileExpressionKind::FieldAccess,
        Expression::TupleLiteral(_) => ResolvedProfileExpressionKind::TupleLiteral,
        Expression::TupleIndex { .. } => ResolvedProfileExpressionKind::TupleIndex,
        Expression::StructLiteral { .. } => ResolvedProfileExpressionKind::StructLiteral,
        Expression::EnumVariant { data, .. } => ResolvedProfileExpressionKind::EnumVariant {
            parenthesized: data.is_some(),
        },
        Expression::Match { .. } => ResolvedProfileExpressionKind::Match,
        Expression::Borrow { mutable, .. } => {
            ResolvedProfileExpressionKind::Borrow { mutable: *mutable }
        }
        Expression::Deref(_) => ResolvedProfileExpressionKind::Dereference,
        Expression::Closure { .. } => ResolvedProfileExpressionKind::Closure,
    }
}

fn pattern_kind(pattern: &Pattern) -> ResolvedProfilePatternKind {
    match pattern {
        Pattern::Wildcard => ResolvedProfilePatternKind::Wildcard,
        Pattern::Literal(_) => ResolvedProfilePatternKind::Literal,
        Pattern::Identifier(_) => ResolvedProfilePatternKind::Identifier,
        Pattern::Tuple(_) => ResolvedProfilePatternKind::Tuple,
        Pattern::Struct { .. } => ResolvedProfilePatternKind::Struct,
        Pattern::Enum { data, .. } => ResolvedProfilePatternKind::Enum {
            parenthesized: data.is_some(),
        },
    }
}

fn assignment_target(expression: &Expression) -> ResolvedProfileAssignmentTarget {
    fn classify(
        expression: &Expression,
        projections: &mut Vec<ResolvedProfileAssignmentProjection>,
    ) -> ResolvedProfileAssignmentRoot {
        match expression {
            Expression::Identifier(_) => ResolvedProfileAssignmentRoot::Identifier,
            Expression::FieldAccess { object, .. } => {
                let root = classify(object, projections);
                projections.push(ResolvedProfileAssignmentProjection::Field);
                root
            }
            Expression::IndexAccess { object, .. } => {
                let root = classify(object, projections);
                projections.push(ResolvedProfileAssignmentProjection::Index);
                root
            }
            Expression::Deref(inner) => {
                let root = classify(inner, projections);
                projections.push(ResolvedProfileAssignmentProjection::Dereference);
                root
            }
            _ => ResolvedProfileAssignmentRoot::Other,
        }
    }

    let mut projections = Vec::new();
    let root = classify(expression, &mut projections);
    ResolvedProfileAssignmentTarget { root, projections }
}

fn binary_operator(operator: &BinaryOp) -> ResolvedProfileBinaryOperator {
    match operator {
        BinaryOp::Add => ResolvedProfileBinaryOperator::Add,
        BinaryOp::Subtract => ResolvedProfileBinaryOperator::Subtract,
        BinaryOp::Multiply => ResolvedProfileBinaryOperator::Multiply,
        BinaryOp::Divide => ResolvedProfileBinaryOperator::Divide,
        BinaryOp::Modulo => ResolvedProfileBinaryOperator::Modulo,
    }
}

fn comparison_operator(operator: &ComparisonOp) -> ResolvedProfileComparisonOperator {
    match operator {
        ComparisonOp::Equal => ResolvedProfileComparisonOperator::Equal,
        ComparisonOp::NotEqual => ResolvedProfileComparisonOperator::NotEqual,
        ComparisonOp::LessThan => ResolvedProfileComparisonOperator::LessThan,
        ComparisonOp::GreaterThan => ResolvedProfileComparisonOperator::GreaterThan,
        ComparisonOp::LessEqual => ResolvedProfileComparisonOperator::LessEqual,
        ComparisonOp::GreaterEqual => ResolvedProfileComparisonOperator::GreaterEqual,
    }
}

fn logical_operator(operator: &LogicalOp) -> ResolvedProfileLogicalOperator {
    match operator {
        LogicalOp::And => ResolvedProfileLogicalOperator::And,
        LogicalOp::Or => ResolvedProfileLogicalOperator::Or,
    }
}

fn unary_operator(operator: &UnaryOp) -> ResolvedProfileUnaryOperator {
    match operator {
        UnaryOp::Not => ResolvedProfileUnaryOperator::Not,
        UnaryOp::Negate => ResolvedProfileUnaryOperator::Negate,
    }
}

fn force_excluded(resolution: ResolvedProfileResolution) -> ResolvedProfileResolution {
    match resolution {
        ResolvedProfileResolution::Resolved(id) | ResolvedProfileResolution::Excluded(Some(id)) => {
            ResolvedProfileResolution::Excluded(Some(id))
        }
        ResolvedProfileResolution::Excluded(None) => ResolvedProfileResolution::Excluded(None),
        ResolvedProfileResolution::Unresolved => ResolvedProfileResolution::Unresolved,
    }
}

fn struct_origin(name: &str, source_generic: bool) -> ResolvedProfileOrigin {
    if let Some(source) = private_generic_struct_source_name(name) {
        ResolvedProfileOrigin::GenericStruct {
            normalized: name.to_string(),
            source,
        }
    } else if source_generic {
        ResolvedProfileOrigin::SourceGenericStruct {
            normalized: name.to_string(),
        }
    } else if name.starts_with("__aero$") {
        ResolvedProfileOrigin::OpaquePrivate {
            normalized: name.to_string(),
        }
    } else {
        ResolvedProfileOrigin::Source {
            normalized: name.to_string(),
        }
    }
}

fn enum_origin(name: &str, source_generic: bool) -> ResolvedProfileOrigin {
    if let Some(source) = private_carrier_source_name(name) {
        ResolvedProfileOrigin::BuiltinCarrier {
            normalized: name.to_string(),
            source,
        }
    } else if let Some(source) = private_generic_enum_source_name(name) {
        ResolvedProfileOrigin::GenericEnum {
            normalized: name.to_string(),
            source,
        }
    } else if source_generic {
        ResolvedProfileOrigin::SourceGenericEnum {
            normalized: name.to_string(),
        }
    } else if name.starts_with("__aero$") {
        ResolvedProfileOrigin::OpaquePrivate {
            normalized: name.to_string(),
        }
    } else {
        ResolvedProfileOrigin::Source {
            normalized: name.to_string(),
        }
    }
}

fn function_origin(name: &str, source_generic: bool) -> ResolvedProfileOrigin {
    if let Some(source) = private_generic_function_source_name(name) {
        ResolvedProfileOrigin::GenericFunction {
            normalized: name.to_string(),
            source,
        }
    } else if source_generic {
        ResolvedProfileOrigin::SourceGenericFunction {
            normalized: name.to_string(),
        }
    } else if name.starts_with("__aero$") {
        ResolvedProfileOrigin::OpaquePrivate {
            normalized: name.to_string(),
        }
    } else {
        ResolvedProfileOrigin::Source {
            normalized: name.to_string(),
        }
    }
}

fn candidate_shape(logical: &LogicalType) -> bool {
    match logical {
        LogicalType::Int | LogicalType::Bool | LogicalType::Void => true,
        LogicalType::Array { element, count } => {
            *count > 0 && *count <= i32::MAX as usize && **element == LogicalType::Int
        }
        LogicalType::Struct { .. } => candidate_non_carrier_shape(logical),
        LogicalType::Enum { name, variants } => {
            private_carrier_source_name(name).is_some_and(|source| {
                source.starts_with("Result<")
                    && matches!(
                        variants.as_slice(),
                        [EnumVariantSchema {
                            name: ok,
                            payload: Some(ok_payload),
                        }, EnumVariantSchema {
                            name: error,
                            payload: Some(error_payload),
                        }] if ok == "Ok"
                            && error == "Err"
                            && candidate_non_carrier_shape(ok_payload)
                            && candidate_non_carrier_shape(error_payload)
                    )
            })
        }
        LogicalType::Float
        | LogicalType::Char
        | LogicalType::String
        | LogicalType::ImmutableReference { .. }
        | LogicalType::MutableReference { .. }
        | LogicalType::Tuple { .. }
        | LogicalType::EnumFields { .. } => false,
    }
}

fn candidate_non_carrier_shape(logical: &LogicalType) -> bool {
    match logical {
        LogicalType::Int | LogicalType::Bool => true,
        LogicalType::Array { element, count } => {
            *count > 0 && *count <= i32::MAX as usize && **element == LogicalType::Int
        }
        LogicalType::Struct { name, fields } => {
            private_generic_struct_source_name(name).is_none()
                && fields.iter().all(candidate_non_carrier_shape)
        }
        LogicalType::Float
        | LogicalType::Char
        | LogicalType::Void
        | LogicalType::String
        | LogicalType::ImmutableReference { .. }
        | LogicalType::MutableReference { .. }
        | LogicalType::Tuple { .. }
        | LogicalType::EnumFields { .. }
        | LogicalType::Enum { .. } => false,
    }
}

fn logical_children(logical: &LogicalType) -> Vec<&LogicalType> {
    match logical {
        LogicalType::ImmutableReference { pointee }
        | LogicalType::MutableReference { pointee }
        | LogicalType::Array {
            element: pointee, ..
        } => vec![pointee],
        LogicalType::Struct { fields, .. } | LogicalType::EnumFields { fields } => {
            fields.iter().collect()
        }
        LogicalType::Tuple { elements } => elements.iter().collect(),
        LogicalType::Enum { variants, .. } => variants
            .iter()
            .filter_map(|variant| variant.payload.as_ref())
            .collect(),
        LogicalType::Int
        | LogicalType::Float
        | LogicalType::Bool
        | LogicalType::Char
        | LogicalType::Void
        | LogicalType::String => Vec::new(),
    }
}

fn payload_logical_fields(payload: &LogicalType) -> Vec<LogicalType> {
    match payload {
        LogicalType::EnumFields { fields } => fields.clone(),
        payload => vec![payload.clone()],
    }
}

fn exact_match_identity(arms: &[MatchArm]) -> Option<String> {
    let mut identity = None::<String>;
    for arm in arms {
        match &arm.pattern {
            Pattern::Enum { enum_name, .. } => match &identity {
                Some(expected) if expected != enum_name => return None,
                Some(_) => {}
                None => identity = Some(enum_name.clone()),
            },
            Pattern::Wildcard => {}
            _ => return None,
        }
    }
    identity
}

fn bind_pattern_names(pattern: &Pattern, mut bind: impl FnMut(String)) {
    fn visit(pattern: &Pattern, bind: &mut impl FnMut(String)) {
        match pattern {
            Pattern::Identifier(name) => bind(name.clone()),
            Pattern::Tuple(patterns) => {
                for pattern in patterns {
                    visit(pattern, bind);
                }
            }
            Pattern::Struct { fields, .. } => {
                for (_, pattern) in fields {
                    visit(pattern, bind);
                }
            }
            Pattern::Enum { data, .. } => {
                for pattern in data.iter().flatten() {
                    visit(pattern, bind);
                }
            }
            Pattern::Wildcard | Pattern::Literal(_) => {}
        }
    }
    visit(pattern, &mut bind);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IrGenerator, SemanticAnalyzer, parse_with_locations, try_tokenize_with_locations};

    const DESCRIPTOR_FIXTURE: &str = r#"
struct Leaf {
    value: int,
}

struct Pair {
    left: Leaf,
    right: Leaf,
}

struct Box<T> {
    value: T,
}

enum Sample<T> {
    Present(T),
    Missing,
}

fn choose<T>(first: T, second: T, use_first: bool) -> T {
    let mut selected: T = second;
    if use_first {
        selected = first;
    }
    selected
}

fn boxed(value: int) -> Box<int> {
    return Box { value: value };
}

fn sampled(value: int) -> Sample<int> {
    return Sample::Present(value);
}

fn array_id(value: [int; 2]) -> [int; 2] {
    return value;
}

fn make(input: Pair, valid: bool) -> Result<Pair, int> {
    let mut current: Pair = Pair {
        right: input.right,
        left: input.left,
    };
    let inferred = Pair {
        right: current.right,
        left: current.left,
    };
    let mut step: int = 0;
    while step < 1 {
        let loop_value = Pair {
            right: current.right,
            left: current.left,
        };
        current = loop_value;
        step = step + 1;
    }
    if valid {
        return Ok(inferred);
    }
    return Err(7);
}

fn score(value: Result<Pair, int>) -> int {
    return match value {
        Err(code) => code,
        Ok(pair) => pair.left.value + pair.right.value,
    };
}

fn wildcard_score(value: Result<Pair, int>) -> int {
    return match value {
        _ => 1,
    };
}

fn consume(value: Pair) -> int {
    return value.left.value;
}

fn forward(value: Pair) -> int {
    return consume(value);
}

fn main() -> int {
    let seed: Pair = Pair {
        left: Leaf { value: 2 },
        right: Leaf { value: 3 },
    };
    let result: Result<Pair, int> = make(seed, 1 < 2);
    let chosen: int = choose(4, 5, 2 < 3);
    return score(result) + chosen;
}
"#;

    fn parsed(source: &str) -> Vec<AstNode> {
        let tokens = try_tokenize_with_locations(source, None).expect("fixture must lex");
        parse_with_locations(tokens).expect("fixture must parse")
    }

    fn rich(
        analyzer: &mut SemanticAnalyzer,
        source: &str,
    ) -> (String, Vec<AstNode>, ResolvedProfileProgram) {
        analyzer
            .analyze_with_resolved_profile(parsed(source))
            .expect("fixture must pass rich semantic analysis")
    }

    fn logical_for<'a>(
        program: &'a ResolvedProfileProgram,
        resolution: &ResolvedProfileResolution,
    ) -> Option<&'a LogicalType> {
        let id = match resolution {
            ResolvedProfileResolution::Resolved(id)
            | ResolvedProfileResolution::Excluded(Some(id)) => *id,
            ResolvedProfileResolution::Excluded(None) | ResolvedProfileResolution::Unresolved => {
                return None;
            }
        };
        program.shapes.get(id.0)
    }

    fn file_statement(kind: ResolvedProfileStatementKind) -> ResolvedProfileSurfaceObservation {
        ResolvedProfileSurfaceObservation::Statement {
            context: ResolvedProfileSurfaceContext::FileScope,
            kind,
        }
    }

    fn file_expression(kind: ResolvedProfileExpressionKind) -> ResolvedProfileSurfaceObservation {
        ResolvedProfileSurfaceObservation::Expression {
            context: ResolvedProfileSurfaceContext::FileScope,
            kind,
        }
    }

    fn file_pattern(kind: ResolvedProfilePatternKind) -> ResolvedProfileSurfaceObservation {
        ResolvedProfileSurfaceObservation::Pattern {
            context: ResolvedProfileSurfaceContext::FileScope,
            kind,
        }
    }

    fn source_context(function: &str) -> ResolvedProfileSurfaceContext {
        ResolvedProfileSurfaceContext::Function(ResolvedProfileOrigin::Source {
            normalized: function.to_string(),
        })
    }

    fn source_statement(
        function: &str,
        kind: ResolvedProfileStatementKind,
    ) -> ResolvedProfileSurfaceObservation {
        ResolvedProfileSurfaceObservation::Statement {
            context: source_context(function),
            kind,
        }
    }

    fn source_expression(
        function: &str,
        kind: ResolvedProfileExpressionKind,
    ) -> ResolvedProfileSurfaceObservation {
        ResolvedProfileSurfaceObservation::Expression {
            context: source_context(function),
            kind,
        }
    }

    #[test]
    fn surface_projection_covers_every_statement_category() {
        let location = crate::errors::SourceLocation::new(1, 1);
        let int_type = Type::Named("int".to_string());
        let expression = Expression::IntegerLiteral(1);
        let block = Block {
            statements: Vec::new(),
            expression: None,
        };
        let cases = vec![
            (
                Statement::Const {
                    name: "value".to_string(),
                    type_annotation: int_type.clone(),
                    value: expression.clone(),
                    location: location.clone(),
                },
                true,
                ResolvedProfileStatementKind::Const,
            ),
            (
                Statement::Let {
                    name: "value".to_string(),
                    mutable: true,
                    type_annotation: Some(int_type.clone()),
                    value: None,
                },
                false,
                ResolvedProfileStatementKind::Let {
                    mutable: true,
                    annotated: true,
                    initialized: false,
                },
            ),
            (
                Statement::Assignment {
                    target: Expression::Identifier("value".to_string()),
                    value: expression.clone(),
                },
                false,
                ResolvedProfileStatementKind::Assignment {
                    target: ResolvedProfileAssignmentTarget {
                        root: ResolvedProfileAssignmentRoot::Identifier,
                        projections: Vec::new(),
                    },
                },
            ),
            (
                Statement::Return(None),
                false,
                ResolvedProfileStatementKind::Return { has_value: false },
            ),
            (
                Statement::Expression(expression.clone()),
                false,
                ResolvedProfileStatementKind::Expression,
            ),
            (
                Statement::Block(block.clone()),
                false,
                ResolvedProfileStatementKind::Block,
            ),
            (
                Statement::Function {
                    name: "function".to_string(),
                    parameters: Vec::new(),
                    return_type: Some(int_type.clone()),
                    body: block.clone(),
                    type_params: vec!["T".to_string()],
                    trait_bounds: vec![("T".to_string(), vec!["Copy".to_string()])],
                },
                false,
                ResolvedProfileStatementKind::Function {
                    top_level: false,
                    generic: true,
                    trait_bounded: true,
                    explicit_result: true,
                },
            ),
            (
                Statement::If {
                    condition: expression.clone(),
                    then_block: block.clone(),
                    else_block: Some(Box::new(Statement::Break)),
                },
                false,
                ResolvedProfileStatementKind::If { has_else: true },
            ),
            (
                Statement::While {
                    condition: expression.clone(),
                    body: block.clone(),
                },
                false,
                ResolvedProfileStatementKind::While,
            ),
            (
                Statement::For {
                    variable: "item".to_string(),
                    iterable: expression.clone(),
                    body: block.clone(),
                },
                false,
                ResolvedProfileStatementKind::For,
            ),
            (
                Statement::Loop {
                    body: block.clone(),
                },
                false,
                ResolvedProfileStatementKind::Loop,
            ),
            (Statement::Break, false, ResolvedProfileStatementKind::Break),
            (
                Statement::Continue,
                false,
                ResolvedProfileStatementKind::Continue,
            ),
            (
                Statement::StructDef {
                    name: "Box".to_string(),
                    fields: Vec::new(),
                    type_params: vec!["T".to_string()],
                },
                true,
                ResolvedProfileStatementKind::StructDefinition { generic: true },
            ),
            (
                Statement::EnumDef {
                    name: "Choice".to_string(),
                    variants: Vec::new(),
                    type_params: vec!["T".to_string()],
                    trait_bounds: vec![("T".to_string(), vec!["Copy".to_string()])],
                },
                true,
                ResolvedProfileStatementKind::EnumDefinition {
                    generic: true,
                    trait_bounded: true,
                },
            ),
            (
                Statement::ImplBlock {
                    type_name: "Box".to_string(),
                    methods: Vec::new(),
                    type_params: vec!["T".to_string()],
                    trait_name: Some("Copy".to_string()),
                },
                true,
                ResolvedProfileStatementKind::ImplBlock {
                    generic: true,
                    trait_impl: true,
                },
            ),
            (
                Statement::TraitDef {
                    name: "Contract".to_string(),
                    type_params: vec!["T".to_string()],
                    methods: Vec::new(),
                },
                true,
                ResolvedProfileStatementKind::TraitDefinition { generic: true },
            ),
            (
                Statement::ModDecl {
                    name: "helper".to_string(),
                    is_public: true,
                },
                true,
                ResolvedProfileStatementKind::ModuleDeclaration { public: true },
            ),
            (
                Statement::UseImport {
                    syntax: ImportSyntax::FoundingDottedImport,
                    path: vec!["helper".to_string(), "value".to_string()],
                    alias: Some("imported".to_string()),
                    location,
                },
                true,
                ResolvedProfileStatementKind::UseImport {
                    founding_syntax: true,
                    aliased: true,
                },
            ),
        ];

        for (statement, top_level, expected) in cases {
            assert_eq!(statement_kind(&statement, top_level), expected);
        }
    }

    #[test]
    fn surface_projection_covers_every_expression_operator_and_pattern_category() {
        let integer = || Expression::IntegerLiteral(1);
        for (operator, expected) in [
            (BinaryOp::Add, ResolvedProfileBinaryOperator::Add),
            (BinaryOp::Subtract, ResolvedProfileBinaryOperator::Subtract),
            (BinaryOp::Multiply, ResolvedProfileBinaryOperator::Multiply),
            (BinaryOp::Divide, ResolvedProfileBinaryOperator::Divide),
            (BinaryOp::Modulo, ResolvedProfileBinaryOperator::Modulo),
        ] {
            assert_eq!(
                expression_kind(&Expression::Binary {
                    op: operator,
                    left: Box::new(integer()),
                    right: Box::new(integer()),
                    ty: None,
                }),
                ResolvedProfileExpressionKind::Binary(expected)
            );
        }
        for (operator, expected) in [
            (
                ComparisonOp::Equal,
                ResolvedProfileComparisonOperator::Equal,
            ),
            (
                ComparisonOp::NotEqual,
                ResolvedProfileComparisonOperator::NotEqual,
            ),
            (
                ComparisonOp::LessThan,
                ResolvedProfileComparisonOperator::LessThan,
            ),
            (
                ComparisonOp::GreaterThan,
                ResolvedProfileComparisonOperator::GreaterThan,
            ),
            (
                ComparisonOp::LessEqual,
                ResolvedProfileComparisonOperator::LessEqual,
            ),
            (
                ComparisonOp::GreaterEqual,
                ResolvedProfileComparisonOperator::GreaterEqual,
            ),
        ] {
            assert_eq!(
                expression_kind(&Expression::Comparison {
                    op: operator,
                    left: Box::new(integer()),
                    right: Box::new(integer()),
                }),
                ResolvedProfileExpressionKind::Comparison(expected)
            );
        }
        for (operator, expected) in [
            (LogicalOp::And, ResolvedProfileLogicalOperator::And),
            (LogicalOp::Or, ResolvedProfileLogicalOperator::Or),
        ] {
            assert_eq!(
                expression_kind(&Expression::Logical {
                    op: operator,
                    left: Box::new(integer()),
                    right: Box::new(integer()),
                }),
                ResolvedProfileExpressionKind::Logical(expected)
            );
        }
        for (operator, expected) in [
            (UnaryOp::Not, ResolvedProfileUnaryOperator::Not),
            (UnaryOp::Negate, ResolvedProfileUnaryOperator::Negate),
        ] {
            assert_eq!(
                expression_kind(&Expression::Unary {
                    op: operator,
                    operand: Box::new(integer()),
                }),
                ResolvedProfileExpressionKind::Unary(expected)
            );
        }

        let expressions = vec![
            (integer(), ResolvedProfileExpressionKind::IntegerLiteral),
            (
                Expression::FloatLiteral(1.5),
                ResolvedProfileExpressionKind::FloatLiteral,
            ),
            (
                Expression::CharacterLiteral('a'),
                ResolvedProfileExpressionKind::CharacterLiteral,
            ),
            (
                Expression::StringLiteral("text".to_string()),
                ResolvedProfileExpressionKind::StringLiteral,
            ),
            (
                Expression::Identifier("value".to_string()),
                ResolvedProfileExpressionKind::Identifier,
            ),
            (
                Expression::FunctionCall {
                    name: "function".to_string(),
                    arguments: vec![integer()],
                },
                ResolvedProfileExpressionKind::FunctionCall,
            ),
            (
                Expression::MethodCall {
                    object: Box::new(integer()),
                    method: "method".to_string(),
                    arguments: vec![integer()],
                },
                ResolvedProfileExpressionKind::MethodCall,
            ),
            (
                Expression::Print {
                    format_string: "{}".to_string(),
                    arguments: vec![integer()],
                },
                ResolvedProfileExpressionKind::Print,
            ),
            (
                Expression::Println {
                    format_string: "{}".to_string(),
                    arguments: vec![integer()],
                },
                ResolvedProfileExpressionKind::Println,
            ),
            (
                Expression::ArrayLiteral(vec![integer()]),
                ResolvedProfileExpressionKind::ArrayLiteral,
            ),
            (
                Expression::ArrayRepeat {
                    value: Box::new(integer()),
                    count: 2,
                },
                ResolvedProfileExpressionKind::ArrayRepeat,
            ),
            (
                Expression::IndexAccess {
                    object: Box::new(integer()),
                    index: Box::new(integer()),
                },
                ResolvedProfileExpressionKind::IndexAccess,
            ),
            (
                Expression::FieldAccess {
                    object: Box::new(integer()),
                    field: "field".to_string(),
                },
                ResolvedProfileExpressionKind::FieldAccess,
            ),
            (
                Expression::TupleLiteral(vec![integer(), integer()]),
                ResolvedProfileExpressionKind::TupleLiteral,
            ),
            (
                Expression::TupleIndex {
                    object: Box::new(integer()),
                    index: 1,
                },
                ResolvedProfileExpressionKind::TupleIndex,
            ),
            (
                Expression::StructLiteral {
                    name: "Record".to_string(),
                    fields: vec![("field".to_string(), integer())],
                },
                ResolvedProfileExpressionKind::StructLiteral,
            ),
            (
                Expression::EnumVariant {
                    enum_name: "Choice".to_string(),
                    variant: "Some".to_string(),
                    data: Some(Vec::new()),
                },
                ResolvedProfileExpressionKind::EnumVariant {
                    parenthesized: true,
                },
            ),
            (
                Expression::Match {
                    expr: Box::new(integer()),
                    arms: Vec::new(),
                },
                ResolvedProfileExpressionKind::Match,
            ),
            (
                Expression::Borrow {
                    expr: Box::new(integer()),
                    mutable: false,
                },
                ResolvedProfileExpressionKind::Borrow { mutable: false },
            ),
            (
                Expression::Deref(Box::new(integer())),
                ResolvedProfileExpressionKind::Dereference,
            ),
            (
                Expression::Closure {
                    params: vec![crate::ast::Parameter {
                        name: "value".to_string(),
                        param_type: Type::Named("int".to_string()),
                    }],
                    body: Box::new(integer()),
                    location: crate::errors::SourceLocation::new(1, 1),
                },
                ResolvedProfileExpressionKind::Closure,
            ),
        ];
        for (expression, expected) in expressions {
            assert_eq!(expression_kind(&expression), expected);
        }

        let patterns = vec![
            (Pattern::Wildcard, ResolvedProfilePatternKind::Wildcard),
            (
                Pattern::Literal(integer()),
                ResolvedProfilePatternKind::Literal,
            ),
            (
                Pattern::Identifier("value".to_string()),
                ResolvedProfilePatternKind::Identifier,
            ),
            (
                Pattern::Tuple(Vec::new()),
                ResolvedProfilePatternKind::Tuple,
            ),
            (
                Pattern::Struct {
                    name: "Record".to_string(),
                    fields: Vec::new(),
                },
                ResolvedProfilePatternKind::Struct,
            ),
            (
                Pattern::Enum {
                    enum_name: "Choice".to_string(),
                    variant: "Some".to_string(),
                    data: Some(Vec::new()),
                },
                ResolvedProfilePatternKind::Enum {
                    parenthesized: true,
                },
            ),
            (
                Pattern::Enum {
                    enum_name: "Choice".to_string(),
                    variant: "Empty".to_string(),
                    data: None,
                },
                ResolvedProfilePatternKind::Enum {
                    parenthesized: false,
                },
            ),
        ];
        for (pattern, expected) in patterns {
            assert_eq!(pattern_kind(&pattern), expected);
        }

        let empty_ast = Vec::new();
        let structs = StructRegistry::from_top_level_ast(&empty_ast);
        let enums = EnumRegistry::from_top_level_ast(&empty_ast, &structs);
        let admitted_function = |_name: &str| None;
        let mut builder = Builder::new(&structs, &enums, &admitted_function);
        builder.walk_pattern(&Pattern::Tuple(vec![
            Pattern::Literal(integer()),
            Pattern::Struct {
                name: "Record".to_string(),
                fields: vec![("field".to_string(), Pattern::Literal(integer()))],
            },
        ]));
        assert_eq!(
            builder.surface,
            vec![
                file_pattern(ResolvedProfilePatternKind::Tuple),
                file_pattern(ResolvedProfilePatternKind::Literal),
                file_expression(ResolvedProfileExpressionKind::IntegerLiteral),
                file_pattern(ResolvedProfilePatternKind::Struct),
                file_pattern(ResolvedProfilePatternKind::Literal),
                file_expression(ResolvedProfileExpressionKind::IntegerLiteral),
            ]
        );

        let target = Expression::IndexAccess {
            object: Box::new(Expression::FieldAccess {
                object: Box::new(Expression::FunctionCall {
                    name: "make".to_string(),
                    arguments: Vec::new(),
                }),
                field: "values".to_string(),
            }),
            index: Box::new(integer()),
        };
        assert_eq!(
            assignment_target(&target),
            ResolvedProfileAssignmentTarget {
                root: ResolvedProfileAssignmentRoot::Other,
                projections: vec![
                    ResolvedProfileAssignmentProjection::Field,
                    ResolvedProfileAssignmentProjection::Index,
                ],
            }
        );
    }

    #[test]
    fn surface_witness_records_hidden_syntax_in_exact_preorder() {
        let source = r#"
fn main() -> int {
    let hidden = 1.0;
    print!("{}", hidden);
    println!("{}", hidden);
    return 0;
}
"#;
        let (_, _, program) = rich(&mut SemanticAnalyzer::new(), source);
        assert_eq!(
            program.surface,
            vec![
                file_statement(ResolvedProfileStatementKind::Function {
                    top_level: true,
                    generic: false,
                    trait_bounded: false,
                    explicit_result: true,
                }),
                source_statement(
                    "main",
                    ResolvedProfileStatementKind::Let {
                        mutable: false,
                        annotated: false,
                        initialized: true,
                    }
                ),
                source_expression("main", ResolvedProfileExpressionKind::FloatLiteral),
                source_statement("main", ResolvedProfileStatementKind::Expression),
                source_expression("main", ResolvedProfileExpressionKind::Print),
                source_expression("main", ResolvedProfileExpressionKind::Identifier),
                source_statement("main", ResolvedProfileStatementKind::Expression),
                source_expression("main", ResolvedProfileExpressionKind::Println),
                source_expression("main", ResolvedProfileExpressionKind::Identifier),
                source_statement(
                    "main",
                    ResolvedProfileStatementKind::Return { has_value: true },
                ),
                source_expression("main", ResolvedProfileExpressionKind::IntegerLiteral),
            ],
            "the single total walk must retain exact normalized preorder"
        );
        assert!(
            program
                .uses
                .iter()
                .all(|usage| usage.name.as_deref() != Some("hidden")),
            "surface observation must not infer an unannotated binding type"
        );
    }

    #[test]
    fn surface_context_distinguishes_file_scope_functions_and_restoration() {
        let source = r#"
1;

fn main() -> int {
    {
        let value: int = 2;
    }
    return 3;
}

4;
"#;
        let (_, _, program) = rich(&mut SemanticAnalyzer::new(), source);
        let file_integers = program
            .surface
            .iter()
            .filter(|observation| {
                matches!(
                    observation,
                    ResolvedProfileSurfaceObservation::Expression {
                        context: ResolvedProfileSurfaceContext::FileScope,
                        kind: ResolvedProfileExpressionKind::IntegerLiteral,
                    }
                )
            })
            .count();
        assert_eq!(
            file_integers, 2,
            "root expressions on both sides of a function must restore file scope"
        );
        assert!(program.surface.contains(&file_statement(
            ResolvedProfileStatementKind::Function {
                top_level: true,
                generic: false,
                trait_bounded: false,
                explicit_result: true,
            }
        )));
        for expected in [
            source_statement("main", ResolvedProfileStatementKind::Block),
            source_statement(
                "main",
                ResolvedProfileStatementKind::Let {
                    mutable: false,
                    annotated: true,
                    initialized: true,
                },
            ),
            source_statement(
                "main",
                ResolvedProfileStatementKind::Return { has_value: true },
            ),
        ] {
            assert!(
                program.surface.contains(&expected),
                "function body lost context for {expected:?}"
            );
        }
        assert_eq!(
            program
                .surface
                .iter()
                .filter(|observation| matches!(
                    observation,
                    ResolvedProfileSurfaceObservation::Expression {
                        context: ResolvedProfileSurfaceContext::Function(
                            ResolvedProfileOrigin::Source { normalized },
                        ),
                        kind: ResolvedProfileExpressionKind::IntegerLiteral,
                    } if normalized == "main"
                ))
                .count(),
            2,
            "nested block and return literals must retain the main function origin"
        );

        let nested = parsed(
            r#"
fn outer() -> int {
    fn inner() -> int {
        return 5;
    }
    return 6;
}
"#,
        );
        let structs = StructRegistry::from_top_level_ast(&nested);
        let enums = EnumRegistry::from_top_level_ast(&nested, &structs);
        let admitted_function = |_name: &str| None;
        let nested_program = Builder::new(&structs, &enums, &admitted_function).build(&nested);
        assert!(nested_program.surface.iter().any(|observation| matches!(
            observation,
            ResolvedProfileSurfaceObservation::Statement {
                context: ResolvedProfileSurfaceContext::Function(
                    ResolvedProfileOrigin::Source { normalized: outer },
                ),
                kind: ResolvedProfileStatementKind::Function { top_level: false, .. },
            } if outer == "outer"
        )));
        for function in ["inner", "outer"] {
            assert!(
                nested_program.surface.iter().any(|observation| matches!(
                    observation,
                    ResolvedProfileSurfaceObservation::Statement {
                        context: ResolvedProfileSurfaceContext::Function(
                            ResolvedProfileOrigin::Source { normalized },
                        ),
                        kind: ResolvedProfileStatementKind::Return { has_value: true },
                    } if normalized == function
                )),
                "nested function context did not restore `{function}`"
            );
        }

        let (_, _, generic) = rich(&mut SemanticAnalyzer::new(), DESCRIPTOR_FIXTURE);
        assert!(
            generic.surface.iter().any(|observation| matches!(
                observation,
                ResolvedProfileSurfaceObservation::Expression {
                    context: ResolvedProfileSurfaceContext::Function(
                        ResolvedProfileOrigin::GenericFunction { source, .. },
                    ),
                    ..
                } if source == "choose<int>"
            )),
            "normalized generic function body lost its exact private origin"
        );
        assert!(
            generic.surface.iter().any(|observation| matches!(
                observation,
                ResolvedProfileSurfaceObservation::Pattern {
                    context: ResolvedProfileSurfaceContext::Function(
                        ResolvedProfileOrigin::Source { normalized },
                    ),
                    kind: ResolvedProfilePatternKind::Enum { parenthesized: true },
                } if normalized == "score"
            )),
            "Match patterns lost their enclosing source function"
        );
    }

    #[test]
    fn surface_witness_retains_operators_patterns_and_assignment_topology() {
        let source = r#"
struct Frame {
    lanes: [int; 2],
}

fn main() -> int {
    let mut direct: [int; 2] = [1, 2];
    direct[0] = 8 / 2;
    direct[1] = 5 * 2 - 1;
    let mut frame: Frame = Frame { lanes: [4, 5] };
    frame.lanes[1] = direct[0] + 6;
    let mut scalar: int = 1;
    {
        let alias = &mut scalar;
        *alias = 2;
    }
    if !(direct[0] == 1) || (direct[1] < 3 && scalar > 0) {
        return frame.lanes[1];
    }
    return scalar;
}
"#;
        let (_, _, program) = rich(&mut SemanticAnalyzer::new(), source);
        let assignments = program
            .surface
            .iter()
            .filter_map(|observation| match observation {
                ResolvedProfileSurfaceObservation::Statement {
                    kind: ResolvedProfileStatementKind::Assignment { target },
                    ..
                } => Some(target),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(assignments.contains(&&ResolvedProfileAssignmentTarget {
            root: ResolvedProfileAssignmentRoot::Identifier,
            projections: vec![ResolvedProfileAssignmentProjection::Index],
        }));
        assert!(assignments.contains(&&ResolvedProfileAssignmentTarget {
            root: ResolvedProfileAssignmentRoot::Identifier,
            projections: vec![
                ResolvedProfileAssignmentProjection::Field,
                ResolvedProfileAssignmentProjection::Index,
            ],
        }));
        assert!(assignments.contains(&&ResolvedProfileAssignmentTarget {
            root: ResolvedProfileAssignmentRoot::Identifier,
            projections: vec![ResolvedProfileAssignmentProjection::Dereference],
        }));

        for operator in [
            ResolvedProfileExpressionKind::Binary(ResolvedProfileBinaryOperator::Add),
            ResolvedProfileExpressionKind::Binary(ResolvedProfileBinaryOperator::Subtract),
            ResolvedProfileExpressionKind::Binary(ResolvedProfileBinaryOperator::Multiply),
            ResolvedProfileExpressionKind::Binary(ResolvedProfileBinaryOperator::Divide),
            ResolvedProfileExpressionKind::Comparison(ResolvedProfileComparisonOperator::Equal),
            ResolvedProfileExpressionKind::Comparison(ResolvedProfileComparisonOperator::LessThan),
            ResolvedProfileExpressionKind::Comparison(
                ResolvedProfileComparisonOperator::GreaterThan,
            ),
            ResolvedProfileExpressionKind::Logical(ResolvedProfileLogicalOperator::And),
            ResolvedProfileExpressionKind::Logical(ResolvedProfileLogicalOperator::Or),
            ResolvedProfileExpressionKind::Unary(ResolvedProfileUnaryOperator::Not),
        ] {
            assert!(
                program
                    .surface
                    .contains(&source_expression("main", operator.clone())),
                "missing operator {operator:?}"
            );
        }
        assert!(program.surface.contains(&source_expression(
            "main",
            ResolvedProfileExpressionKind::Borrow { mutable: true },
        )));
        assert!(program.surface.contains(&source_expression(
            "main",
            ResolvedProfileExpressionKind::Dereference,
        )));

        let (_, _, descriptor) = rich(&mut SemanticAnalyzer::new(), DESCRIPTOR_FIXTURE);
        let pattern_preorder = descriptor
            .surface
            .iter()
            .filter_map(|observation| match observation {
                ResolvedProfileSurfaceObservation::Pattern { kind, .. } => Some(kind),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            pattern_preorder.windows(2).any(|window| matches!(
                window,
                [
                    ResolvedProfilePatternKind::Enum {
                        parenthesized: true,
                    },
                    ResolvedProfilePatternKind::Identifier,
                ]
            )),
            "enum payload patterns must be recorded before their identifier child"
        );
        for pattern in [
            ResolvedProfilePatternKind::Wildcard,
            ResolvedProfilePatternKind::Identifier,
            ResolvedProfilePatternKind::Enum {
                parenthesized: true,
            },
        ] {
            assert!(
                descriptor.surface.iter().any(|observation| matches!(
                    observation,
                    ResolvedProfileSurfaceObservation::Pattern { kind, .. } if kind == &pattern
                )),
                "missing pattern witness {pattern:?}"
            );
        }
    }

    #[test]
    fn surface_witness_retains_module_import_and_preserved_function_categories() {
        let source = r#"
struct Pair {
    left: int,
    right: int,
}

impl Pair {
    fn probe(value: Pair) -> Pair {
        return value;
    }
}

trait Contract {
    fn required(value: Pair) -> Pair;

    fn provided(value: Pair) -> Pair {
        return value;
    }
}

fn main() -> int {
    return 0;
}
"#;
        let (_, _, program) = rich(&mut SemanticAnalyzer::new(), source);
        assert_eq!(
            program
                .surface
                .iter()
                .filter(|observation| matches!(
                    observation,
                    ResolvedProfileSurfaceObservation::Statement {
                        kind: ResolvedProfileStatementKind::Function {
                            top_level: false,
                            generic: false,
                            trait_bounded: false,
                            explicit_result: true,
                        },
                        ..
                    }
                ))
                .count(),
            3,
            "impl, required, and default trait methods must retain function categories"
        );
        assert!(
            program.surface.iter().any(|observation| matches!(
                observation,
                ResolvedProfileSurfaceObservation::Expression {
                    context: ResolvedProfileSurfaceContext::Function(
                        ResolvedProfileOrigin::ImplMethod {
                            type_name,
                            trait_name: None,
                            method,
                        },
                    ),
                    kind: ResolvedProfileExpressionKind::Identifier,
                } if type_name == "Pair" && method == "probe"
            )),
            "impl body lost its container-qualified origin"
        );
        assert!(
            program.surface.iter().any(|observation| matches!(
                observation,
                ResolvedProfileSurfaceObservation::Expression {
                    context: ResolvedProfileSurfaceContext::Function(
                        ResolvedProfileOrigin::TraitMethod { trait_name, method },
                    ),
                    kind: ResolvedProfileExpressionKind::Identifier,
                } if trait_name == "Contract" && method == "provided"
            )),
            "trait default body lost its container-qualified origin"
        );
        assert!(
            !program.surface.iter().any(|observation| matches!(
                observation,
                ResolvedProfileSurfaceObservation::Expression {
                    context: ResolvedProfileSurfaceContext::Function(
                        ResolvedProfileOrigin::TraitMethod { trait_name, method },
                    ),
                    ..
                } if trait_name == "Contract" && method == "required"
            )),
            "bodyless trait method invented body observations"
        );
        assert_eq!(
            statement_kind(
                &Statement::ModDecl {
                    name: "helper".to_string(),
                    is_public: true,
                },
                true,
            ),
            ResolvedProfileStatementKind::ModuleDeclaration { public: true }
        );
        assert_eq!(
            statement_kind(
                &Statement::UseImport {
                    syntax: ImportSyntax::FoundingDottedImport,
                    path: vec!["helper".to_string(), "value".to_string()],
                    alias: None,
                    location: crate::errors::SourceLocation::new(1, 1),
                },
                true,
            ),
            ResolvedProfileStatementKind::UseImport {
                founding_syntax: true,
                aliased: false,
            }
        );
    }

    #[test]
    fn descriptor_retains_identity_order_roles_and_memoized_shapes() {
        let (_, _, program) = rich(&mut SemanticAnalyzer::new(), DESCRIPTOR_FIXTURE);

        let pair = program
            .nominals
            .iter()
            .find_map(|nominal| match nominal {
                ResolvedProfileNominal::Struct {
                    origin: ResolvedProfileOrigin::Source { normalized },
                    resolution,
                    fields,
                } if normalized == "Pair" => Some((resolution, fields)),
                _ => None,
            })
            .expect("Pair nominal must be observed");
        assert!(matches!(pair.0, ResolvedProfileResolution::Resolved(_)));
        let pair_resolution = pair.0.clone();
        assert_eq!(
            pair.1
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>(),
            ["left", "right"],
            "record declaration order drifted"
        );
        let [left, right] = pair.1.as_slice() else {
            panic!("Pair must retain exactly two fields")
        };
        assert_eq!(
            left.resolution, right.resolution,
            "repeated Leaf children must reuse one shape ID"
        );
        assert!(matches!(
            logical_for(&program, &left.resolution),
            Some(LogicalType::Struct { name, .. }) if name == "Leaf"
        ));

        let generic_box = program
            .nominals
            .iter()
            .find_map(|nominal| match nominal {
                ResolvedProfileNominal::Struct {
                    origin: ResolvedProfileOrigin::GenericStruct { normalized, source },
                    resolution: ResolvedProfileResolution::Excluded(Some(id)),
                    ..
                } if source == "Box<int>" => Some((normalized, *id)),
                _ => None,
            })
            .expect("normalized Box<int> nominal must be observed");
        assert_eq!(
            private_generic_struct_source_name(generic_box.0).as_deref(),
            Some("Box<int>")
        );
        assert!(matches!(
            program.shapes.get(generic_box.1.0),
            Some(LogicalType::Struct { name, .. }) if name == generic_box.0
        ));

        let generic_sample = program
            .nominals
            .iter()
            .find_map(|nominal| match nominal {
                ResolvedProfileNominal::Enum {
                    origin: ResolvedProfileOrigin::GenericEnum { normalized, source },
                    resolution: ResolvedProfileResolution::Excluded(Some(id)),
                    ..
                } if source == "Sample<int>" => Some((normalized, *id)),
                _ => None,
            })
            .expect("normalized Sample<int> nominal must be observed");
        assert_eq!(
            private_generic_enum_source_name(generic_sample.0).as_deref(),
            Some("Sample<int>")
        );
        assert!(matches!(
            program.shapes.get(generic_sample.1.0),
            Some(LogicalType::Enum { name, .. }) if name == generic_sample.0
        ));

        let result = program
            .nominals
            .iter()
            .find_map(|nominal| match nominal {
                ResolvedProfileNominal::Enum {
                    origin: ResolvedProfileOrigin::BuiltinCarrier { normalized, source },
                    resolution,
                    variants,
                } if source == "Result<Pair, int>" => Some((normalized, resolution, variants)),
                _ => None,
            })
            .expect("normalized Result<Pair, int> nominal must be observed");
        assert!(matches!(result.1, ResolvedProfileResolution::Resolved(_)));
        let result_resolution = result.1.clone();
        let result_logical = logical_for(&program, &result_resolution)
            .expect("resolved Result must have a logical shape")
            .clone();
        assert_eq!(
            private_carrier_source_name(result.0).as_deref(),
            Some("Result<Pair, int>")
        );
        assert!(matches!(
            &result_logical,
            LogicalType::Enum { name, .. } if name == result.0
        ));
        assert!(candidate_shape(&result_logical));
        assert!(
            !candidate_shape(&LogicalType::Struct {
                name: "NestedCarrier".to_string(),
                fields: vec![result_logical],
            }),
            "a nested carrier must never become a candidate record field"
        );
        assert_eq!(
            result
                .2
                .iter()
                .map(|variant| variant.name.as_str())
                .collect::<Vec<_>>(),
            ["Ok", "Err"],
            "Result variant order drifted"
        );
        let [ok, error] = result.2.as_slice() else {
            panic!("Result must retain exactly Ok and Err")
        };
        assert!(matches!(
            ok.payload
                .as_ref()
                .and_then(|payload| logical_for(&program, payload)),
            Some(LogicalType::Struct { name, .. }) if name == "Pair"
        ));
        assert_eq!(
            error
                .payload
                .as_ref()
                .and_then(|payload| logical_for(&program, payload)),
            Some(&LogicalType::Int)
        );
        let int_resolution = ResolvedProfileResolution::Resolved(ResolvedProfileShapeId(
            program
                .shapes
                .iter()
                .position(|shape| shape == &LogicalType::Int)
                .expect("Int shape must be interned"),
        ));
        let bool_resolution = ResolvedProfileResolution::Resolved(ResolvedProfileShapeId(
            program
                .shapes
                .iter()
                .position(|shape| shape == &LogicalType::Bool)
                .expect("Bool shape must be interned"),
        ));
        let array_resolution = ResolvedProfileResolution::Resolved(ResolvedProfileShapeId(
            program
                .shapes
                .iter()
                .position(|shape| {
                    matches!(
                        shape,
                        LogicalType::Array { element, count }
                            if **element == LogicalType::Int && *count == 2
                    )
                })
                .expect("flat [int; 2] shape must be interned once"),
        ));

        let roles = program
            .uses
            .iter()
            .map(|usage| usage.role)
            .collect::<Vec<_>>();
        for expected in [
            ProfileTypeUse::Parameter,
            ProfileTypeUse::Result,
            ProfileTypeUse::Binding,
            ProfileTypeUse::MutableBinding,
            ProfileTypeUse::OwnedAssignment,
            ProfileTypeUse::Value,
        ] {
            assert!(
                roles.contains(&expected),
                "missing transport role {expected:?}"
            );
        }
        let exact_use = |function: &str,
                         name: Option<&str>,
                         role: ProfileTypeUse,
                         resolution: &ResolvedProfileResolution| {
            program.uses.iter().any(|usage| {
                usage.role == role
                    && usage.name.as_deref() == name
                    && &usage.resolution == resolution
                    && matches!(
                        &usage.function,
                        Some(ResolvedProfileOrigin::Source { normalized })
                            if normalized == function
                    )
            })
        };
        assert!(exact_use(
            "make",
            Some("input"),
            ProfileTypeUse::Parameter,
            &pair_resolution
        ));
        assert!(exact_use(
            "array_id",
            Some("value"),
            ProfileTypeUse::Parameter,
            &array_resolution
        ));
        assert!(exact_use(
            "array_id",
            None,
            ProfileTypeUse::Result,
            &array_resolution
        ));
        assert!(exact_use(
            "make",
            None,
            ProfileTypeUse::Result,
            &result_resolution
        ));
        assert!(exact_use(
            "make",
            Some("current"),
            ProfileTypeUse::MutableBinding,
            &pair_resolution
        ));
        assert!(exact_use(
            "make",
            Some("current"),
            ProfileTypeUse::OwnedAssignment,
            &pair_resolution
        ));
        assert!(exact_use(
            "main",
            Some("seed"),
            ProfileTypeUse::Binding,
            &pair_resolution
        ));
        let forward_pair_values = program
            .uses
            .iter()
            .filter(|usage| {
                usage.role == ProfileTypeUse::Value
                    && usage.resolution == pair_resolution
                    && matches!(
                        &usage.function,
                        Some(ResolvedProfileOrigin::Source { normalized })
                            if normalized == "forward"
                    )
            })
            .count();
        assert_eq!(
            forward_pair_values, 1,
            "the constructor-free forward call must retain its exact Pair argument transport"
        );
        assert!(exact_use(
            "main",
            Some("result"),
            ProfileTypeUse::Binding,
            &result_resolution
        ));
        assert!(exact_use(
            "main",
            None,
            ProfileTypeUse::Value,
            &pair_resolution
        ));
        assert!(
            !program
                .uses
                .iter()
                .any(|usage| usage.name.as_deref() == Some("inferred")),
            "an unannotated aggregate binding became a resolved root"
        );

        let generic_uses = program
            .uses
            .iter()
            .filter(|usage| {
                matches!(
                    &usage.function,
                    Some(ResolvedProfileOrigin::GenericFunction { source, .. })
                        if source == "choose<int>"
                )
            })
            .collect::<Vec<_>>();
        assert!(
            !generic_uses.is_empty(),
            "normalized choose<int> roots were not observed"
        );
        let generic_names = generic_uses
            .iter()
            .filter_map(|usage| match &usage.function {
                Some(ResolvedProfileOrigin::GenericFunction { normalized, .. }) => {
                    Some(normalized.as_str())
                }
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(generic_names.len(), 1);
        assert_eq!(
            private_generic_function_source_name(generic_names.first().copied().unwrap())
                .as_deref(),
            Some("choose<int>")
        );
        assert!(
            generic_uses.iter().all(|usage| {
                matches!(
                    &usage.resolution,
                    ResolvedProfileResolution::Excluded(Some(id))
                        if program.shapes.get(id.0).is_some_and(|shape| {
                            matches!(shape, LogicalType::Int | LogicalType::Bool)
                        })
                )
            }),
            "source-facing generic function roots lost their exact excluded shape"
        );
        let generic_exact_use =
            |name: Option<&str>, role: ProfileTypeUse, expected: &ResolvedProfileResolution| {
                generic_uses.iter().any(|usage| {
                    usage.name.as_deref() == name
                        && usage.role == role
                        && usage.resolution == force_excluded(expected.clone())
                })
            };
        for name in ["first", "second"] {
            assert!(generic_exact_use(
                Some(name),
                ProfileTypeUse::Parameter,
                &int_resolution
            ));
        }
        assert!(generic_exact_use(
            Some("use_first"),
            ProfileTypeUse::Parameter,
            &bool_resolution
        ));
        assert!(generic_exact_use(
            None,
            ProfileTypeUse::Result,
            &int_resolution
        ));
        assert!(generic_exact_use(
            Some("selected"),
            ProfileTypeUse::MutableBinding,
            &int_resolution
        ));
        assert!(generic_exact_use(
            Some("selected"),
            ProfileTypeUse::OwnedAssignment,
            &int_resolution
        ));

        assert!(!candidate_shape(&LogicalType::Array {
            element: Box::new(LogicalType::Int),
            count: 0,
        }));
        assert!(!candidate_shape(&LogicalType::Array {
            element: Box::new(LogicalType::Bool),
            count: 2,
        }));
        assert!(!candidate_shape(&LogicalType::Array {
            element: Box::new(LogicalType::Array {
                element: Box::new(LogicalType::Int),
                count: 2,
            }),
            count: 2,
        }));

        assert!(program.operations.iter().any(|operation| matches!(
            operation,
            ResolvedProfileOperation::StructConstruction {
                function: Some(ResolvedProfileOrigin::Source { normalized: function }),
                origin: ResolvedProfileOrigin::Source { normalized },
                resolution,
                source_to_declaration,
                ..
            } if function == "make"
                && normalized == "Pair"
                && resolution == &pair_resolution
                && source_to_declaration == &[1, 0]
        )));
        assert!(program.operations.iter().any(|operation| matches!(
            operation,
            ResolvedProfileOperation::EnumConstruction {
                origin: ResolvedProfileOrigin::BuiltinCarrier { source, .. },
                variant,
                variant_index: Some(0),
                resolution,
                ..
            } if source == "Result<Pair, int>"
                && variant == "Ok"
                && resolution == &result_resolution
        )));
        assert!(program.operations.iter().any(|operation| matches!(
            operation,
            ResolvedProfileOperation::EnumConstruction {
                origin: ResolvedProfileOrigin::BuiltinCarrier { source, .. },
                variant,
                variant_index: Some(1),
                resolution,
                ..
            } if source == "Result<Pair, int>"
                && variant == "Err"
                && resolution == &result_resolution
        )));
        assert!(program.operations.iter().any(|operation| matches!(
            operation,
            ResolvedProfileOperation::ExhaustiveMatch {
                origin: Some(ResolvedProfileOrigin::BuiltinCarrier { source, .. }),
                arm_for_variant,
                resolution,
                result: Some(result),
                ..
            } if source == "Result<Pair, int>"
                && arm_for_variant == &[1, 0]
                && resolution == &result_resolution
                && result == &int_resolution
        )));
        assert!(program.operations.iter().any(|operation| matches!(
            operation,
            ResolvedProfileOperation::ExhaustiveMatch {
                function: Some(ResolvedProfileOrigin::Source { normalized }),
                origin: Some(ResolvedProfileOrigin::BuiltinCarrier { source, .. }),
                resolution,
                arm_for_variant,
                result: Some(result),
            } if normalized == "wildcard_score"
                && source == "Result<Pair, int>"
                && resolution == &result_resolution
                && arm_for_variant == &[0, 0]
                && result == &int_resolution
        )));
    }

    #[test]
    fn rich_analysis_is_deterministic_and_preserves_checked_ir() {
        let mut analyzer = SemanticAnalyzer::new();
        let (first_message, first_ast, first_program) = rich(&mut analyzer, DESCRIPTOR_FIXTURE);
        let (second_message, second_ast, second_program) = rich(&mut analyzer, DESCRIPTOR_FIXTURE);

        assert_eq!(first_message, second_message);
        assert_eq!(format!("{first_ast:?}"), format!("{second_ast:?}"));
        assert_eq!(first_program, second_program);
        assert_eq!(first_program.surface, second_program.surface);
        assert!(!first_program.surface.is_empty());
        assert_eq!(
            first_program.operations.len(),
            second_program.operations.len(),
            "descriptor observations accumulated across analyzer reuse"
        );
        let loop_pair_count = first_program
            .operations
            .iter()
            .filter(|operation| {
                matches!(
                    operation,
                    ResolvedProfileOperation::StructConstruction {
                        origin: ResolvedProfileOrigin::Source { normalized },
                        source_to_declaration,
                        ..
                    } if normalized == "Pair" && source_to_declaration == &[1, 0]
                )
            })
            .count();
        assert_eq!(
            loop_pair_count, 3,
            "each reversed Pair constructor, including the loop body, must be observed once"
        );

        let rich_checked = IrGenerator::new()
            .try_generate_ir(first_ast)
            .expect("rich normalized AST must reach checked IR");
        let (_, public_ast) = SemanticAnalyzer::new()
            .analyze(parsed(DESCRIPTOR_FIXTURE))
            .expect("public semantic route must remain compatible");
        let public_checked = IrGenerator::new()
            .try_generate_ir(public_ast)
            .expect("public normalized AST must reach checked IR");
        assert_eq!(
            rich_checked, public_checked,
            "rich semantic success changed checked IR"
        );
    }

    #[test]
    fn unresolved_declarations_fail_closed_without_a_new_diagnostic() {
        let source = r#"
struct Cycle {
    next: Cycle,
}

struct UnknownChild {
    child: Missing,
}

fn main() -> int {
    return 0;
}
"#;
        let (_, _, program) = rich(&mut SemanticAnalyzer::new(), source);
        for name in ["Cycle", "UnknownChild"] {
            let resolution = program
                .nominals
                .iter()
                .find_map(|nominal| match nominal {
                    ResolvedProfileNominal::Struct {
                        origin: ResolvedProfileOrigin::Source { normalized },
                        resolution,
                        ..
                    } if normalized == name => Some(resolution),
                    _ => None,
                })
                .unwrap_or_else(|| panic!("{name} declaration must remain observable"));
            assert_eq!(
                resolution,
                &ResolvedProfileResolution::Unresolved,
                "{name} must fail closed"
            );
            assert!(
                logical_for(&program, resolution).is_none(),
                "{name} silently fell back to a concrete shape"
            );
        }

        SemanticAnalyzer::new()
            .analyze(parsed(source))
            .expect("public semantics must retain declaration-only compatibility");
    }

    #[test]
    fn implicit_results_and_non_top_level_name_collisions_stay_excluded() {
        let source = r#"
struct Pair {
    left: int,
    right: int,
}

fn collide(value: Pair) -> Pair {
    return Pair { right: 2, left: 1 };
}

impl Pair {
    fn collide(value: Pair) -> Pair {
        let method_local: Pair = value;
        return method_local;
    }
}

trait Probe {
    fn required(value: Pair) -> Pair;

    fn collide(value: Pair) -> Pair {
        let trait_local: Pair = value;
        value = value;
        return value;
    }
}

fn implicit() {
}

fn main() -> int {
    let value = Pair { left: 1, right: 2 };
    return collide(value).left;
}
"#;
        let (_, _, program) = rich(&mut SemanticAnalyzer::new(), source);

        assert!(
            !program.uses.iter().any(|usage| {
                usage.role == ProfileTypeUse::Result
                    && matches!(
                        &usage.function,
                        Some(ResolvedProfileOrigin::Source { normalized })
                            if normalized == "implicit"
                    )
            }),
            "an implicit void function acquired an explicit Result root"
        );

        let pair_resolution = pair_resolution_for(&program);
        let excluded_pair = force_excluded(pair_resolution.clone());
        let top_level_parameter = program.uses.iter().find(|usage| {
            usage.role == ProfileTypeUse::Parameter
                && usage.name.as_deref() == Some("value")
                && matches!(
                    &usage.function,
                    Some(ResolvedProfileOrigin::Source { normalized })
                        if normalized == "collide"
                )
        });
        assert_eq!(
            top_level_parameter.map(|usage| &usage.resolution),
            Some(&pair_resolution),
            "the admitted top-level collide parameter must remain resolved"
        );

        let impl_origin = ResolvedProfileOrigin::ImplMethod {
            type_name: "Pair".to_string(),
            trait_name: None,
            method: "collide".to_string(),
        };
        let trait_origin = ResolvedProfileOrigin::TraitMethod {
            trait_name: "Probe".to_string(),
            method: "collide".to_string(),
        };
        for origin in [&impl_origin, &trait_origin] {
            for (role, name) in [
                (ProfileTypeUse::Parameter, Some("value")),
                (ProfileTypeUse::Result, None),
            ] {
                assert!(program.uses.iter().any(|usage| {
                    usage.role == role
                        && usage.name.as_deref() == name
                        && usage.function.as_ref() == Some(origin)
                        && usage.resolution == excluded_pair
                }));
            }
        }
        let required_origin = ResolvedProfileOrigin::TraitMethod {
            trait_name: "Probe".to_string(),
            method: "required".to_string(),
        };
        for (role, name) in [
            (ProfileTypeUse::Parameter, Some("value")),
            (ProfileTypeUse::Result, None),
        ] {
            assert!(program.uses.iter().any(|usage| {
                usage.role == role
                    && usage.name.as_deref() == name
                    && usage.function.as_ref() == Some(&required_origin)
                    && usage.resolution == excluded_pair
            }));
        }
        for (name, origin) in [
            ("method_local", &impl_origin),
            ("trait_local", &trait_origin),
        ] {
            let usage = program
                .uses
                .iter()
                .find(|usage| {
                    usage.role == ProfileTypeUse::Binding
                        && usage.name.as_deref() == Some(name)
                        && usage.function.as_ref() == Some(origin)
                })
                .unwrap_or_else(|| panic!("{name} preserved binding must be observed"));
            assert_eq!(
                usage.resolution, excluded_pair,
                "{name} must stay outside the profile context"
            );
        }
        assert!(program.uses.iter().any(|usage| {
            usage.role == ProfileTypeUse::OwnedAssignment
                && usage.name.as_deref() == Some("value")
                && usage.function.as_ref() == Some(&trait_origin)
                && usage.resolution == excluded_pair
        }));

        let reversed_pair_constructions = program
            .operations
            .iter()
            .filter_map(|operation| match operation {
                ResolvedProfileOperation::StructConstruction {
                    function:
                        Some(ResolvedProfileOrigin::Source {
                            normalized: function,
                        }),
                    origin: ResolvedProfileOrigin::Source { normalized },
                    resolution,
                    source_to_declaration,
                } if function == "collide"
                    && normalized == "Pair"
                    && source_to_declaration == &[1, 0] =>
                {
                    Some(resolution)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(reversed_pair_constructions, [&pair_resolution]);
    }

    fn pair_resolution_for(program: &ResolvedProfileProgram) -> ResolvedProfileResolution {
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
            .expect("Pair nominal must be present")
    }
}
