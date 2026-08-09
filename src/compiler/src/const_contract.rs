use crate::ast::{
    AstNode, BinaryOp, Block, ComparisonOp, Expression, LogicalOp, MatchArm, Pattern, Statement,
    TraitMethod, Type, UnaryOp,
};
use crate::errors::SourceLocation;
use crate::primitive_contract::PrimitiveKind;
use crate::types::Ty;
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum ConstValue {
    Int(i32),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),
}

impl ConstValue {
    fn ty(&self) -> Ty {
        match self {
            Self::Int(_) => Ty::Int,
            Self::Float(_) => Ty::Float,
            Self::Bool(_) => Ty::Bool,
            Self::Char(_) => Ty::Char,
            Self::String(_) => Ty::String,
        }
    }

    fn expression(&self) -> Expression {
        match self {
            Self::Int(value) => Expression::IntegerLiteral(i64::from(*value)),
            Self::Float(value) => Expression::FloatLiteral(*value),
            Self::Bool(value) => Expression::Comparison {
                op: if *value {
                    ComparisonOp::Equal
                } else {
                    ComparisonOp::NotEqual
                },
                left: Box::new(Expression::IntegerLiteral(0)),
                right: Box::new(Expression::IntegerLiteral(0)),
            },
            Self::Char(value) => Expression::CharacterLiteral(*value),
            Self::String(value) => Expression::StringLiteral(value.clone()),
        }
    }
}

#[derive(Debug, Clone)]
enum LexicalBinding {
    Const(ConstValue),
    Runtime,
}

#[derive(Debug, Clone)]
struct ConstEnvironment {
    scopes: Vec<HashMap<String, LexicalBinding>>,
}

impl ConstEnvironment {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes
            .pop()
            .expect("primitive-const scopes are balanced");
    }

    fn current_contains(&self, name: &str) -> bool {
        self.scopes
            .last()
            .expect("primitive-const environment has a root scope")
            .contains_key(name)
    }

    fn define(&mut self, name: String, binding: LexicalBinding) {
        self.scopes
            .last_mut()
            .expect("primitive-const environment has a root scope")
            .insert(name, binding);
    }

    fn lookup(&self, name: &str) -> Option<&LexicalBinding> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name))
    }

    fn constant(&self, name: &str) -> Option<&ConstValue> {
        match self.lookup(name) {
            Some(LexicalBinding::Const(value)) => Some(value),
            Some(LexicalBinding::Runtime) | None => None,
        }
    }
}

pub(crate) fn normalize_primitive_consts(ast: Vec<AstNode>) -> Result<Vec<AstNode>, String> {
    let mut environment = ConstEnvironment::new();
    let mut normalized = Vec::with_capacity(ast.len());
    for node in ast {
        match node {
            AstNode::Statement(statement) => {
                if let Some(statement) = normalize_statement(statement, &mut environment)? {
                    normalized.push(AstNode::Statement(statement));
                }
            }
            AstNode::Expression(mut expression) => {
                normalize_expression(&mut expression, &mut environment)?;
                normalized.push(AstNode::Expression(expression));
            }
        }
    }
    Ok(normalized)
}

fn normalize_statement(
    mut statement: Statement,
    environment: &mut ConstEnvironment,
) -> Result<Option<Statement>, String> {
    match &mut statement {
        Statement::Const {
            name,
            type_annotation,
            value,
            location,
        } => {
            let name = name.clone();
            if matches!(name.as_str(), "true" | "false") {
                return Err(const_diagnostic(
                    &name,
                    location,
                    "Boolean literal names cannot be declared as constants",
                ));
            }
            if environment.current_contains(&name) {
                return Err(const_diagnostic(
                    &name,
                    location,
                    "the name is already defined in this lexical scope",
                ));
            }
            let expected = annotation_type(type_annotation)
                .map_err(|message| const_diagnostic(&name, location, &message))?;
            let evaluated = evaluate_const_expression(value, environment)
                .map_err(|message| const_diagnostic(&name, location, &message))?;
            let actual = evaluated.ty();
            if actual != expected {
                return Err(const_diagnostic(
                    &name,
                    location,
                    &format!("type annotation mismatch: expected {expected}, evaluated {actual}"),
                ));
            }
            environment.define(name, LexicalBinding::Const(evaluated));
            return Ok(None);
        }
        Statement::Let { name, value, .. } => {
            if let Some(value) = value {
                normalize_expression(value, environment)?;
            }
            reject_shadowed_constant(name, environment)?;
            environment.define(name.clone(), LexicalBinding::Runtime);
        }
        Statement::Assignment { target, value } => {
            if let Expression::Identifier(name) = target
                && environment.constant(name).is_some()
            {
                return Err(format!(
                    "Error: Cannot assign to primitive constant `{name}`."
                ));
            }
            normalize_expression(target, environment)?;
            normalize_expression(value, environment)?;
        }
        Statement::Return(value) => {
            if let Some(value) = value {
                normalize_expression(value, environment)?;
            }
        }
        Statement::Expression(expression) => normalize_expression(expression, environment)?,
        Statement::Block(block) => normalize_block(block, environment, &[])?,
        Statement::Function {
            parameters, body, ..
        } => {
            let runtime_names = parameters
                .iter()
                .map(|parameter| parameter.name.clone())
                .collect::<Vec<_>>();
            normalize_block(body, environment, &runtime_names)?;
        }
        Statement::If {
            condition,
            then_block,
            else_block,
        } => {
            normalize_expression(condition, environment)?;
            normalize_block(then_block, environment, &[])?;
            if let Some(else_statement) = else_block {
                normalize_nested_statement(else_statement, environment)?;
            }
        }
        Statement::While { condition, body } => {
            normalize_expression(condition, environment)?;
            normalize_block(body, environment, &[])?;
        }
        Statement::For {
            variable,
            iterable,
            body,
        } => {
            normalize_expression(iterable, environment)?;
            normalize_block(body, environment, std::slice::from_ref(variable))?;
        }
        Statement::Loop { body } => normalize_block(body, environment, &[])?,
        Statement::ImplBlock { methods, .. } => {
            normalize_statement_list(methods, environment)?;
        }
        Statement::TraitDef { methods, .. } => {
            for method in methods {
                normalize_trait_method(method, environment)?;
            }
        }
        Statement::Break
        | Statement::Continue
        | Statement::StructDef { .. }
        | Statement::EnumDef { .. }
        | Statement::ModDecl { .. }
        | Statement::UseImport { .. } => {}
    }
    Ok(Some(statement))
}

fn normalize_nested_statement(
    statement: &mut Box<Statement>,
    environment: &mut ConstEnvironment,
) -> Result<(), String> {
    let owned = std::mem::replace(statement.as_mut(), Statement::Break);
    let normalized = normalize_statement(owned, environment)?
        .ok_or_else(|| "Error: A const declaration cannot replace an else branch.".to_string())?;
    **statement = normalized;
    Ok(())
}

fn normalize_statement_list(
    statements: &mut Vec<Statement>,
    environment: &mut ConstEnvironment,
) -> Result<(), String> {
    let mut normalized = Vec::with_capacity(statements.len());
    for statement in std::mem::take(statements) {
        if let Some(statement) = normalize_statement(statement, environment)? {
            normalized.push(statement);
        }
    }
    *statements = normalized;
    Ok(())
}

fn normalize_block(
    block: &mut Block,
    environment: &mut ConstEnvironment,
    runtime_names: &[String],
) -> Result<(), String> {
    environment.push_scope();
    for name in runtime_names {
        environment.define(name.clone(), LexicalBinding::Runtime);
    }
    let result = (|| {
        normalize_statement_list(&mut block.statements, environment)?;
        if let Some(expression) = &mut block.expression {
            normalize_expression(expression, environment)?;
        }
        Ok(())
    })();
    environment.pop_scope();
    result
}

fn normalize_trait_method(
    method: &mut TraitMethod,
    environment: &mut ConstEnvironment,
) -> Result<(), String> {
    if let Some(body) = &mut method.body {
        let runtime_names = method
            .parameters
            .iter()
            .map(|parameter| parameter.name.clone())
            .collect::<Vec<_>>();
        normalize_block(body, environment, &runtime_names)?;
    }
    Ok(())
}

fn reject_shadowed_constant(name: &str, environment: &ConstEnvironment) -> Result<(), String> {
    if matches!(
        environment.scopes.last().and_then(|scope| scope.get(name)),
        Some(LexicalBinding::Const(_))
    ) {
        return Err(format!(
            "Error: Variable `{name}` is already defined as a primitive constant in this scope."
        ));
    }
    Ok(())
}

fn normalize_expression(
    expression: &mut Expression,
    environment: &mut ConstEnvironment,
) -> Result<(), String> {
    match expression {
        Expression::Identifier(name) => {
            if let Some(value) = environment.constant(name) {
                *expression = value.expression();
            }
        }
        Expression::Binary { left, right, .. }
        | Expression::Comparison { left, right, .. }
        | Expression::Logical { left, right, .. } => {
            normalize_expression(left, environment)?;
            normalize_expression(right, environment)?;
        }
        Expression::FunctionCall { arguments, .. }
        | Expression::Print { arguments, .. }
        | Expression::Println { arguments, .. } => {
            for argument in arguments {
                normalize_expression(argument, environment)?;
            }
        }
        Expression::MethodCall {
            object, arguments, ..
        } => {
            normalize_expression(object, environment)?;
            for argument in arguments {
                normalize_expression(argument, environment)?;
            }
        }
        Expression::Unary { operand, .. }
        | Expression::Borrow { expr: operand, .. }
        | Expression::Deref(operand) => normalize_expression(operand, environment)?,
        Expression::ArrayLiteral(elements) | Expression::TupleLiteral(elements) => {
            for element in elements {
                normalize_expression(element, environment)?;
            }
        }
        Expression::ArrayRepeat { value, .. } => normalize_expression(value, environment)?,
        Expression::IndexAccess { object, index } => {
            normalize_expression(object, environment)?;
            normalize_expression(index, environment)?;
        }
        Expression::FieldAccess { object, .. } | Expression::TupleIndex { object, .. } => {
            normalize_expression(object, environment)?;
        }
        Expression::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                normalize_expression(value, environment)?;
            }
        }
        Expression::EnumVariant { data, .. } => {
            if let Some(fields) = data {
                for field in fields {
                    normalize_expression(field, environment)?;
                }
            }
        }
        Expression::Match { expr, arms } => {
            normalize_expression(expr, environment)?;
            for arm in arms {
                normalize_match_arm(arm, environment)?;
            }
        }
        Expression::Closure { params, body, .. } => {
            environment.push_scope();
            for parameter in params {
                environment.define(parameter.name.clone(), LexicalBinding::Runtime);
            }
            let result = normalize_expression(body, environment);
            environment.pop_scope();
            result?;
        }
        Expression::IntegerLiteral(_)
        | Expression::FloatLiteral(_)
        | Expression::CharacterLiteral(_)
        | Expression::StringLiteral(_) => {}
    }
    Ok(())
}

fn normalize_match_arm(
    arm: &mut MatchArm,
    environment: &mut ConstEnvironment,
) -> Result<(), String> {
    environment.push_scope();
    define_pattern_bindings(&arm.pattern, environment);
    let result = normalize_expression(&mut arm.body, environment);
    environment.pop_scope();
    result
}

fn define_pattern_bindings(pattern: &Pattern, environment: &mut ConstEnvironment) {
    match pattern {
        Pattern::Identifier(name) => {
            environment.define(name.clone(), LexicalBinding::Runtime);
        }
        Pattern::Tuple(patterns) => {
            for pattern in patterns {
                define_pattern_bindings(pattern, environment);
            }
        }
        Pattern::Struct { fields, .. } => {
            for (_, pattern) in fields {
                define_pattern_bindings(pattern, environment);
            }
        }
        Pattern::Enum { data, .. } => {
            if let Some(patterns) = data {
                for pattern in patterns {
                    define_pattern_bindings(pattern, environment);
                }
            }
        }
        Pattern::Wildcard | Pattern::Literal(_) => {}
    }
}

fn annotation_type(annotation: &Type) -> Result<Ty, String> {
    match annotation {
        Type::Named(name) => PrimitiveKind::from_source_name(name)
            .map(PrimitiveKind::ty)
            .or_else(|| (name == "String").then_some(Ty::String))
            .ok_or_else(|| {
                format!(
                    "unsupported annotation `{name}`; primitive constants require int/i32, float/f64, bool, char, or String"
                )
            }),
        _ => Err(
            "unsupported composite annotation; primitive constants require int/i32, float/f64, bool, char, or String"
                .to_string(),
        ),
    }
}

fn evaluate_const_expression(
    expression: &Expression,
    environment: &ConstEnvironment,
) -> Result<ConstValue, String> {
    match expression {
        Expression::IntegerLiteral(value) => i32::try_from(*value)
            .map(ConstValue::Int)
            .map_err(|_| "integer literal is outside the admitted i32 range".to_string()),
        Expression::FloatLiteral(value) if value.is_finite() => Ok(ConstValue::Float(*value)),
        Expression::FloatLiteral(_) => Err("floating-point literal is not finite".to_string()),
        Expression::CharacterLiteral(value) => Ok(ConstValue::Char(*value)),
        Expression::StringLiteral(value) => Ok(ConstValue::String(value.clone())),
        Expression::Identifier(name) if name == "true" => Ok(ConstValue::Bool(true)),
        Expression::Identifier(name) if name == "false" => Ok(ConstValue::Bool(false)),
        Expression::Identifier(name) => match environment.lookup(name) {
            Some(LexicalBinding::Const(value)) => Ok(value.clone()),
            Some(LexicalBinding::Runtime) => {
                Err(format!("initializer depends on runtime binding `{name}`"))
            }
            None => Err(format!(
                "initializer refers to unknown or not-yet-declared constant `{name}`"
            )),
        },
        Expression::Binary {
            op, left, right, ..
        } => evaluate_binary(op, left, right, environment),
        Expression::Comparison { op, left, right } => {
            let left = evaluate_const_expression(left, environment)?;
            let right = evaluate_const_expression(right, environment)?;
            evaluate_comparison(op, left, right)
        }
        Expression::Logical { op, left, right } => {
            let ConstValue::Bool(left) = evaluate_const_expression(left, environment)? else {
                return Err("logical operator requires Bool operands".to_string());
            };
            match (op, left) {
                (LogicalOp::And, false) => Ok(ConstValue::Bool(false)),
                (LogicalOp::Or, true) => Ok(ConstValue::Bool(true)),
                _ => {
                    let ConstValue::Bool(right) = evaluate_const_expression(right, environment)?
                    else {
                        return Err("logical operator requires Bool operands".to_string());
                    };
                    Ok(ConstValue::Bool(right))
                }
            }
        }
        Expression::Unary { op, operand } => {
            let operand = evaluate_const_expression(operand, environment)?;
            match (op, operand) {
                (UnaryOp::Not, ConstValue::Bool(value)) => Ok(ConstValue::Bool(!value)),
                (UnaryOp::Negate, ConstValue::Int(value)) => {
                    value.checked_neg().map(ConstValue::Int).ok_or_else(|| {
                        "integer negation overflowed the admitted i32 range".to_string()
                    })
                }
                (UnaryOp::Negate, ConstValue::Float(value)) => finite_float(-value),
                (UnaryOp::Not, _) => Err("logical not requires a Bool operand".to_string()),
                (UnaryOp::Negate, _) => Err("unary minus requires a numeric operand".to_string()),
            }
        }
        Expression::FunctionCall { .. } => excluded("function call"),
        Expression::MethodCall { .. } => excluded("method call"),
        Expression::Print { .. } | Expression::Println { .. } => excluded("I/O macro call"),
        Expression::ArrayLiteral(_) | Expression::ArrayRepeat { .. } => {
            excluded("array expression")
        }
        Expression::IndexAccess { .. } => excluded("index expression"),
        Expression::FieldAccess { .. } => excluded("field access"),
        Expression::TupleLiteral(_) | Expression::TupleIndex { .. } => excluded("tuple expression"),
        Expression::StructLiteral { .. } => excluded("struct expression"),
        Expression::EnumVariant { .. } => excluded("enum expression"),
        Expression::Match { .. } => excluded("match expression"),
        Expression::Borrow { .. } => excluded("borrow expression"),
        Expression::Deref(_) => excluded("dereference expression"),
        Expression::Closure { .. } => excluded("closure expression"),
    }
}

fn evaluate_binary(
    op: &BinaryOp,
    left: &Expression,
    right: &Expression,
    environment: &ConstEnvironment,
) -> Result<ConstValue, String> {
    if matches!(op, BinaryOp::Modulo) {
        return Err(
            "operator `%` is not admitted by the current primitive runtime contract".to_string(),
        );
    }
    let left = evaluate_const_expression(left, environment)?;
    let right = evaluate_const_expression(right, environment)?;
    match (left, right) {
        (ConstValue::Int(left), ConstValue::Int(right)) => {
            let result = match op {
                BinaryOp::Add => left.checked_add(right),
                BinaryOp::Subtract => left.checked_sub(right),
                BinaryOp::Multiply => left.checked_mul(right),
                BinaryOp::Divide if right == 0 => {
                    return Err("integer division by zero".to_string());
                }
                BinaryOp::Divide => left.checked_div(right),
                BinaryOp::Modulo => unreachable!("rejected above"),
            };
            result
                .map(ConstValue::Int)
                .ok_or_else(|| format!("integer operator `{op}` overflowed the admitted i32 range"))
        }
        (ConstValue::Float(left), ConstValue::Float(right)) => {
            evaluate_float_binary(op, left, right)
        }
        (ConstValue::Int(left), ConstValue::Float(right)) => {
            evaluate_float_binary(op, f64::from(left), right)
        }
        (ConstValue::Float(left), ConstValue::Int(right)) => {
            evaluate_float_binary(op, left, f64::from(right))
        }
        (left, right) => Err(format!(
            "operator `{op}` requires numeric operands, found {} and {}",
            left.ty(),
            right.ty()
        )),
    }
}

fn evaluate_float_binary(op: &BinaryOp, left: f64, right: f64) -> Result<ConstValue, String> {
    if matches!(op, BinaryOp::Divide) && right == 0.0 {
        return Err("floating-point division by zero".to_string());
    }
    let value = match op {
        BinaryOp::Add => left + right,
        BinaryOp::Subtract => left - right,
        BinaryOp::Multiply => left * right,
        BinaryOp::Divide => left / right,
        BinaryOp::Modulo => unreachable!("rejected above"),
    };
    finite_float(value)
}

fn finite_float(value: f64) -> Result<ConstValue, String> {
    value
        .is_finite()
        .then_some(ConstValue::Float(value))
        .ok_or_else(|| "floating-point evaluation produced a non-finite result".to_string())
}

fn evaluate_comparison(
    op: &ComparisonOp,
    left: ConstValue,
    right: ConstValue,
) -> Result<ConstValue, String> {
    let result = match (left, right) {
        (ConstValue::Int(left), ConstValue::Int(right)) => compare_ordered(op, left, right),
        (ConstValue::Float(left), ConstValue::Float(right)) => compare_ordered(op, left, right),
        (ConstValue::Int(left), ConstValue::Float(right)) => {
            compare_ordered(op, f64::from(left), right)
        }
        (ConstValue::Float(left), ConstValue::Int(right)) => {
            compare_ordered(op, left, f64::from(right))
        }
        (ConstValue::Bool(left), ConstValue::Bool(right)) => compare_equality(op, left, right)?,
        (ConstValue::Char(left), ConstValue::Char(right)) => compare_equality(op, left, right)?,
        (ConstValue::String(left), ConstValue::String(right)) => compare_equality(op, left, right)?,
        (left, right) => {
            return Err(format!(
                "comparison requires compatible primitive operands, found {} and {}",
                left.ty(),
                right.ty()
            ));
        }
    };
    Ok(ConstValue::Bool(result))
}

fn compare_ordered<T: PartialEq + PartialOrd>(op: &ComparisonOp, left: T, right: T) -> bool {
    match op {
        ComparisonOp::Equal => left == right,
        ComparisonOp::NotEqual => left != right,
        ComparisonOp::LessThan => left < right,
        ComparisonOp::GreaterThan => left > right,
        ComparisonOp::LessEqual => left <= right,
        ComparisonOp::GreaterEqual => left >= right,
    }
}

fn compare_equality<T: PartialEq>(op: &ComparisonOp, left: T, right: T) -> Result<bool, String> {
    match op {
        ComparisonOp::Equal => Ok(left == right),
        ComparisonOp::NotEqual => Ok(left != right),
        _ => Err("ordered comparison is admitted only for numeric constants".to_string()),
    }
}

fn excluded(topology: &str) -> Result<ConstValue, String> {
    Err(format!(
        "initializer uses unsupported {topology}; primitive constants require a closed primitive expression"
    ))
}

fn const_diagnostic(name: &str, location: &SourceLocation, detail: &str) -> String {
    format!("Error: Primitive constant `{name}` at {location}: {detail}.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_integer_evaluation_rejects_overflow() {
        let expression = Expression::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expression::IntegerLiteral(i64::from(i32::MAX))),
            right: Box::new(Expression::IntegerLiteral(1)),
            ty: None,
        };
        let error = evaluate_const_expression(&expression, &ConstEnvironment::new())
            .expect_err("i32 overflow must fail closed");
        assert!(error.contains("overflowed the admitted i32 range"));
    }

    #[test]
    fn logical_evaluation_short_circuits() {
        let division_by_zero = Expression::Comparison {
            op: ComparisonOp::Equal,
            left: Box::new(Expression::Binary {
                op: BinaryOp::Divide,
                left: Box::new(Expression::IntegerLiteral(1)),
                right: Box::new(Expression::IntegerLiteral(0)),
                ty: None,
            }),
            right: Box::new(Expression::IntegerLiteral(0)),
        };
        let expression = Expression::Logical {
            op: LogicalOp::And,
            left: Box::new(Expression::Identifier("false".to_string())),
            right: Box::new(division_by_zero),
        };
        assert!(matches!(
            evaluate_const_expression(&expression, &ConstEnvironment::new()),
            Ok(ConstValue::Bool(false))
        ));
    }
}
