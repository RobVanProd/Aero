use crate::ast::{AstNode, BinaryOp, Block, Expression, Statement, Type};
use crate::ir::LogicalType;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

pub(crate) const STABLE_SCALAR_V0_NAME: &str = "stable-scalar-v0";
pub(crate) const EXACT_I32_ARRAY_V0_NAME: &str = "exact-i32-array-v0";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum LanguageProfile {
    #[default]
    Experimental,
    StableScalarV0,
    ExactI32ArrayV0,
}

impl LanguageProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Experimental => "experimental",
            Self::StableScalarV0 => STABLE_SCALAR_V0_NAME,
            Self::ExactI32ArrayV0 => EXACT_I32_ARRAY_V0_NAME,
        }
    }

    /// Whether verified logical `Int` values use the profile's exact i32 lane.
    pub(crate) fn uses_exact_i32_lane(self) -> bool {
        matches!(self, Self::StableScalarV0 | Self::ExactI32ArrayV0)
    }

    /// Whether this profile admits the exact, flat, nonempty i32-array shape.
    pub(crate) fn admits_exact_i32_array(self, logical_type: &LogicalType) -> bool {
        self == Self::ExactI32ArrayV0
            && matches!(
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
            _ => Err(format!(
                "unsupported language profile `{value}` (expected experimental|{STABLE_SCALAR_V0_NAME}|{EXACT_I32_ARRAY_V0_NAME})"
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
            profile == LanguageProfile::ExactI32ArrayV0 && usage != ProfileTypeUse::Result
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
        LanguageProfile::Experimental => Ok(()),
        LanguageProfile::StableScalarV0 | LanguageProfile::ExactI32ArrayV0 => {
            ProfileValidator::validate(ast, profile)
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
enum BindingShape {
    ScalarOrUnknown,
    ExactI32Array { count: usize },
}

struct ProfileValidator {
    profile: LanguageProfile,
    functions: BTreeSet<String>,
    function_parameter_shapes: BTreeMap<String, Vec<ProfileTypeShape>>,
    calls: BTreeMap<String, BTreeSet<String>>,
    binding_scopes: Vec<BTreeMap<String, BindingShape>>,
}

impl ProfileValidator {
    fn validate(ast: &[AstNode], profile: LanguageProfile) -> Result<(), String> {
        let mut validator = Self {
            profile,
            functions: BTreeSet::new(),
            function_parameter_shapes: BTreeMap::new(),
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
                    name, parameters, ..
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
                        binding_shape_for_type(&parameter.param_type),
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
                if let Some(annotation) = type_annotation {
                    self.validate_type(
                        annotation,
                        ProfileTypeUse::Binding,
                        "binding annotation types",
                    )?;
                }
                let Some(value) = value else {
                    return Err(self.error("uninitialized bindings"));
                };
                let shape = type_annotation
                    .as_ref()
                    .map(binding_shape_for_type)
                    .unwrap_or(BindingShape::ScalarOrUnknown);
                match shape {
                    BindingShape::ExactI32Array { count } => {
                        if *mutable {
                            return Err(self.error("mutable array bindings"));
                        }
                        self.validate_array_initializer(value, count)?;
                    }
                    BindingShape::ScalarOrUnknown => {
                        self.validate_expression(function, value)?;
                    }
                }
                self.binding_scopes
                    .last_mut()
                    .expect("validated function body retains a binding scope")
                    .insert(name.clone(), shape);
                Ok(())
            }
            Statement::Assignment { target, value } => {
                let Expression::Identifier(target_name) = target else {
                    return Err(self.error("projected or indirect assignment targets"));
                };
                if self.is_array_binding(target_name) {
                    return Err(self.error("array writes"));
                }
                self.validate_expression(function, value)
            }
            Statement::Return(value) => {
                if let Some(value) = value {
                    self.validate_expression(function, value)?;
                }
                Ok(())
            }
            Statement::Expression(Expression::FunctionCall { name, arguments }) => {
                self.validate_call(function, name, arguments)
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
        match expression {
            Expression::IntegerLiteral(value) => i32::try_from(*value)
                .map(|_| ())
                .map_err(|_| self.error("integer literals outside the signed i32 range")),
            Expression::Identifier(name) => {
                if self.is_array_binding(name) {
                    Err(self
                        .error("array identifiers outside direct call transport or index reads"))
                } else {
                    Ok(())
                }
            }
            Expression::Binary {
                op, left, right, ..
            } => {
                match op {
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply => {}
                    BinaryOp::Divide => return Err(self.error("division expressions")),
                    BinaryOp::Modulo => return Err(self.error("remainder expressions")),
                }
                self.validate_expression(function, left)?;
                self.validate_expression(function, right)
            }
            Expression::FunctionCall { name, arguments } => {
                self.validate_call(function, name, arguments)
            }
            Expression::Comparison { left, right, .. } => {
                self.validate_expression(function, left)?;
                self.validate_expression(function, right)
            }
            Expression::Logical { left, right, .. } => {
                if expression_contains_call(left) || expression_contains_call(right) {
                    return Err(self.error("function calls inside logical operands"));
                }
                self.validate_expression(function, left)?;
                self.validate_expression(function, right)
            }
            Expression::Unary { operand, .. } => self.validate_expression(function, operand),
            Expression::IndexAccess { object, index } => {
                if self.profile != LanguageProfile::ExactI32ArrayV0 {
                    return Err(self.error(expression_feature(expression)));
                }
                let Expression::Identifier(array_name) = object.as_ref() else {
                    return Err(self.error("projected array index objects"));
                };
                if !self.is_array_binding(array_name) {
                    return Err(self.error("index reads from non-array identifiers"));
                }
                self.validate_expression(function, index)
            }
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

    fn validate_call(
        &mut self,
        function: &str,
        callee: &str,
        arguments: &[Expression],
    ) -> Result<(), String> {
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
                    self.validate_array_transport(argument, *count)?;
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
        Ok(())
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

    fn validate_array_initializer(
        &self,
        initializer: &Expression,
        expected_count: usize,
    ) -> Result<(), String> {
        let Expression::ArrayLiteral(elements) = initializer else {
            return Err(self.error("array bindings without direct literal initializers"));
        };
        if elements.len() != expected_count {
            return Err(self.error("array literal counts that differ from their annotations"));
        }
        if !elements.iter().all(is_exact_signed_i32_literal) {
            return Err(self.error("array elements other than exact signed i32 literals"));
        }
        Ok(())
    }

    fn validate_array_transport(
        &self,
        argument: &Expression,
        expected_count: usize,
    ) -> Result<(), String> {
        let Expression::Identifier(name) = argument else {
            return Err(self.error("array call arguments other than direct identifiers"));
        };
        match self.binding_shape(name) {
            Some(BindingShape::ExactI32Array { count }) if count == expected_count => Ok(()),
            Some(BindingShape::ExactI32Array { .. }) => {
                Err(self.error("array call arguments with mismatched counts"))
            }
            Some(BindingShape::ScalarOrUnknown) | None => {
                Err(self.error("array call arguments from non-array identifiers"))
            }
        }
    }

    fn binding_shape(&self, name: &str) -> Option<BindingShape> {
        self.binding_scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn is_array_binding(&self, name: &str) -> bool {
        matches!(
            self.binding_shape(name),
            Some(BindingShape::ExactI32Array { .. })
        )
    }
}

fn binding_shape_for_type(ty: &Type) -> BindingShape {
    match classify_profile_ast_type(ty) {
        ProfileTypeShape::ExactI32Array { count } => BindingShape::ExactI32Array { count },
        ProfileTypeShape::Int | ProfileTypeShape::Bool | ProfileTypeShape::Unsupported => {
            BindingShape::ScalarOrUnknown
        }
    }
}

fn is_exact_signed_i32_literal(expression: &Expression) -> bool {
    match expression {
        Expression::IntegerLiteral(value) => i32::try_from(*value).is_ok(),
        Expression::Unary {
            op: crate::ast::UnaryOp::Negate,
            operand,
        } => matches!(
            operand.as_ref(),
            Expression::IntegerLiteral(value)
                if (0..=i64::from(i32::MAX) + 1).contains(value)
        ),
        _ => false,
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
    fn parses_only_the_three_named_profiles() {
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
        assert!(!LanguageProfile::Experimental.uses_exact_i32_lane());
        assert!(LanguageProfile::ExactI32ArrayV0.admits_exact_i32_array(&exact));
        assert!(!LanguageProfile::StableScalarV0.admits_exact_i32_array(&exact));
        assert!(!LanguageProfile::Experimental.admits_exact_i32_array(&exact));
    }

    #[test]
    fn shared_profile_role_policy_owns_array_transport_and_result_exclusion() {
        let exact_array = ProfileTypeShape::ExactI32Array { count: 8 };
        for usage in [
            ProfileTypeUse::Parameter,
            ProfileTypeUse::Binding,
            ProfileTypeUse::Value,
        ] {
            assert!(profile_type_shape_is_admitted(
                LanguageProfile::ExactI32ArrayV0,
                exact_array,
                usage
            ));
        }
        assert!(!profile_type_shape_is_admitted(
            LanguageProfile::ExactI32ArrayV0,
            exact_array,
            ProfileTypeUse::Result
        ));
        for profile in [
            LanguageProfile::Experimental,
            LanguageProfile::StableScalarV0,
        ] {
            for usage in [
                ProfileTypeUse::Parameter,
                ProfileTypeUse::Result,
                ProfileTypeUse::Binding,
                ProfileTypeUse::Value,
            ] {
                assert!(!profile_type_shape_is_admitted(profile, exact_array, usage));
            }
        }
        for usage in [
            ProfileTypeUse::Parameter,
            ProfileTypeUse::Result,
            ProfileTypeUse::Binding,
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
    }

    #[test]
    fn exact_i32_array_validator_accepts_the_complete_flat_array_class() {
        for source in [
            "fn read(values: [int; 2], index: int) -> int { return values[index]; } fn main() -> int { let values: [int; 2] = [-2147483648, 2147483647]; return read(values, 1); }",
            "fn read(values: [i32; 1]) -> int { return values[0]; } fn main() -> int { let values: [i32; 1] = [-0]; return read(values); }",
            "fn main() -> int { let values: [int; 1] = [7]; let mut index: int = 0; while index < 1 { let value: int = values[index + 0]; index = index + 1; } return values[0]; }",
        ] {
            validate_language_profile(&parsed(source), LanguageProfile::ExactI32ArrayV0)
                .unwrap_or_else(|error| panic!("exact flat-array source was rejected: {error}"));
        }
    }

    #[test]
    fn exact_i32_array_validator_rejects_every_neighboring_array_topology() {
        let rejected = [
            (
                "fn main() -> int { let values = [1]; return 0; }",
                "array expressions",
            ),
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
                "fn main() -> int { let values: [int; 1] = [1 + 2]; return 0; }",
                "array elements other than exact signed i32 literals",
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
                "fn source(values: [int; 1]) -> int { let copy: [int; 1] = values; return copy[0]; } fn main() -> int { let values: [int; 1] = [1]; return source(values); }",
                "array bindings without direct literal initializers",
            ),
            (
                "fn values() -> [int; 1] { let value: [int; 1] = [1]; return value; } fn main() -> int { return 0; }",
                "function result types",
            ),
            (
                "fn main() -> int { let values: [int; 1] = [1]; values = [2]; return 0; }",
                "array writes",
            ),
            (
                "fn main() -> int { let mut values: [int; 1] = [1]; return values[0]; }",
                "mutable array bindings",
            ),
            (
                "fn main() -> int { let values: [int; 1] = [1]; values[0] = 2; return 0; }",
                "projected or indirect assignment targets",
            ),
            (
                "fn main() -> int { let values: [int; 1] = [1]; let copy = values; return 0; }",
                "array identifiers outside direct call transport or index reads",
            ),
            (
                "fn take(values: [int; 1]) -> int { return values[0]; } fn main() -> int { return take([1]); }",
                "array call arguments other than direct identifiers",
            ),
            (
                "fn take(values: [int; 2]) -> int { return values[0]; } fn main() -> int { let values: [int; 1] = [1]; return take(values); }",
                "array call arguments with mismatched counts",
            ),
            (
                "fn main() -> int { let value: int = 1; return value[0]; }",
                "index reads from non-array identifiers",
            ),
            (
                "fn main() -> int { let values: [int; 1] = [1]; return values[0][0]; }",
                "projected array index objects",
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
