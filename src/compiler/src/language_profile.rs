use crate::ast::{AstNode, BinaryOp, Block, Expression, Statement, Type};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

pub(crate) const STABLE_SCALAR_V0_NAME: &str = "stable-scalar-v0";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum LanguageProfile {
    #[default]
    Experimental,
    StableScalarV0,
}

impl LanguageProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Experimental => "experimental",
            Self::StableScalarV0 => STABLE_SCALAR_V0_NAME,
        }
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
            _ => Err(format!(
                "unsupported language profile `{value}` (expected experimental|{STABLE_SCALAR_V0_NAME})"
            )),
        }
    }
}

pub(crate) fn validate_language_profile(
    ast: &[AstNode],
    profile: LanguageProfile,
) -> Result<(), String> {
    match profile {
        LanguageProfile::Experimental => Ok(()),
        LanguageProfile::StableScalarV0 => StableScalarValidator::validate(ast),
    }
}

fn profile_error(feature: &str) -> String {
    format!("Language Profile Error: {STABLE_SCALAR_V0_NAME} rejects {feature}")
}

#[derive(Default)]
struct StableScalarValidator {
    functions: BTreeSet<String>,
    calls: BTreeMap<String, BTreeSet<String>>,
}

impl StableScalarValidator {
    fn validate(ast: &[AstNode]) -> Result<(), String> {
        let mut validator = Self::default();
        validator.collect_function_headers(ast)?;
        validator.validate_functions(ast)?;
        validator.reject_call_cycles()
    }

    fn collect_function_headers(&mut self, ast: &[AstNode]) -> Result<(), String> {
        for node in ast {
            match node {
                AstNode::Statement(Statement::Function { name, .. }) => {
                    if !self.functions.insert(name.clone()) {
                        return Err(profile_error(&format!(
                            "duplicate function definitions (`{name}`)"
                        )));
                    }
                    self.calls.entry(name.clone()).or_default();
                }
                AstNode::Statement(statement) => {
                    return Err(profile_error(top_level_statement_feature(statement)));
                }
                AstNode::Expression(_) => {
                    return Err(profile_error("top-level expressions"));
                }
            }
        }

        if !self.functions.contains("main") {
            return Err(profile_error("programs without `fn main() -> int`"));
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
                unreachable!("stable scalar header collection admitted only functions")
            };

            if !type_params.is_empty() || !trait_bounds.is_empty() {
                return Err(profile_error("generic functions or trait bounds"));
            }
            for parameter in parameters {
                validate_scalar_type(&parameter.param_type, "function parameter types")?;
            }
            if let Some(return_type) = return_type {
                validate_scalar_type(return_type, "function result types")?;
            }
            if name == "main"
                && (!parameters.is_empty()
                    || !matches!(return_type, Some(Type::Named(result)) if result == "int"))
            {
                return Err(profile_error(
                    "entrypoints other than exact `fn main() -> int`",
                ));
            }

            self.validate_block(name, body)?;
        }
        Ok(())
    }

    fn validate_block(&mut self, function: &str, block: &Block) -> Result<(), String> {
        for statement in &block.statements {
            self.validate_statement(function, statement)?;
        }
        if block.expression.is_some() {
            return Err(profile_error("implicit tail expressions"));
        }
        Ok(())
    }

    fn validate_statement(&mut self, function: &str, statement: &Statement) -> Result<(), String> {
        match statement {
            Statement::Let {
                type_annotation,
                value,
                ..
            } => {
                if let Some(annotation) = type_annotation {
                    validate_scalar_type(annotation, "binding annotation types")?;
                }
                let Some(value) = value else {
                    return Err(profile_error("uninitialized bindings"));
                };
                self.validate_expression(function, value)
            }
            Statement::Assignment { target, value } => {
                if !matches!(target, Expression::Identifier(_)) {
                    return Err(profile_error("projected or indirect assignment targets"));
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
                Err(profile_error(
                    "effect-free or non-call expression statements",
                ))
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
                        other => return Err(profile_error(statement_feature(other))),
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
            | Statement::UseImport { .. } => Err(profile_error(statement_feature(statement))),
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
                .map_err(|_| profile_error("integer literals outside the signed i32 range")),
            Expression::Identifier(_) => Ok(()),
            Expression::Binary {
                op, left, right, ..
            } => {
                match op {
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply => {}
                    BinaryOp::Divide => return Err(profile_error("division expressions")),
                    BinaryOp::Modulo => return Err(profile_error("remainder expressions")),
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
                    return Err(profile_error("function calls inside logical operands"));
                }
                self.validate_expression(function, left)?;
                self.validate_expression(function, right)
            }
            Expression::Unary { operand, .. } => self.validate_expression(function, operand),
            Expression::FloatLiteral(_)
            | Expression::CharacterLiteral(_)
            | Expression::StringLiteral(_)
            | Expression::MethodCall { .. }
            | Expression::Print { .. }
            | Expression::Println { .. }
            | Expression::ArrayLiteral(_)
            | Expression::ArrayRepeat { .. }
            | Expression::IndexAccess { .. }
            | Expression::FieldAccess { .. }
            | Expression::TupleLiteral(_)
            | Expression::TupleIndex { .. }
            | Expression::StructLiteral { .. }
            | Expression::EnumVariant { .. }
            | Expression::Match { .. }
            | Expression::Borrow { .. }
            | Expression::Deref(_)
            | Expression::Closure { .. } => Err(profile_error(expression_feature(expression))),
        }
    }

    fn validate_call(
        &mut self,
        function: &str,
        callee: &str,
        arguments: &[Expression],
    ) -> Result<(), String> {
        for argument in arguments {
            self.validate_expression(function, argument)?;
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
            return Err(profile_error("recursive function call cycles"));
        }
        Ok(())
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

fn validate_scalar_type(ty: &Type, context: &str) -> Result<(), String> {
    match ty {
        Type::Named(name) if matches!(name.as_str(), "int" | "bool") => Ok(()),
        Type::Named(_)
        | Type::Array(_, _)
        | Type::Tuple(_)
        | Type::Reference(_, _)
        | Type::Generic(_, _) => Err(profile_error(context)),
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
    fn parses_only_the_two_named_profiles() {
        assert_eq!(
            "experimental".parse::<LanguageProfile>(),
            Ok(LanguageProfile::Experimental)
        );
        assert_eq!(
            STABLE_SCALAR_V0_NAME.parse::<LanguageProfile>(),
            Ok(LanguageProfile::StableScalarV0)
        );
        assert!("stable".parse::<LanguageProfile>().is_err());
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
