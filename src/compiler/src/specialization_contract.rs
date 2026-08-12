use crate::ast::{AstNode, Type};
use crate::ir::LogicalType;
use crate::primitive_contract::PrimitiveKind;

pub(crate) type PrivateTypeSource = fn(&str) -> Option<String>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SpecializationTypeKey {
    Int,
    Float,
    Bool,
    Char,
    Named(String),
    Array(Box<Self>, usize),
    Tuple(Vec<Self>),
    Reference(Box<Self>, bool),
    Application(String, Vec<Self>),
}

impl SpecializationTypeKey {
    pub(crate) fn from_type(ty: &Type) -> Self {
        match ty {
            Type::Named(name) => match PrimitiveKind::from_source_name(name) {
                Some(PrimitiveKind::Int) => Self::Int,
                Some(PrimitiveKind::Float) => Self::Float,
                Some(PrimitiveKind::Bool) => Self::Bool,
                Some(PrimitiveKind::Char) => Self::Char,
                None => Self::Named(name.clone()),
            },
            Type::Array(element, count) => Self::Array(Box::new(Self::from_type(element)), *count),
            Type::Tuple(elements) => Self::Tuple(elements.iter().map(Self::from_type).collect()),
            Type::Reference(element, mutable) => {
                Self::Reference(Box::new(Self::from_type(element)), *mutable)
            }
            Type::Generic(name, arguments) => Self::Application(
                name.clone(),
                arguments.iter().map(Self::from_type).collect(),
            ),
        }
    }

    pub(crate) fn into_canonical_type(self) -> Type {
        match self {
            Self::Int => Type::Named("int".to_string()),
            Self::Float => Type::Named("float".to_string()),
            Self::Bool => Type::Named("bool".to_string()),
            Self::Char => Type::Named("char".to_string()),
            Self::Named(name) => Type::Named(name),
            Self::Array(element, count) => {
                Type::Array(Box::new(element.into_canonical_type()), count)
            }
            Self::Tuple(elements) => Type::Tuple(
                elements
                    .into_iter()
                    .map(Self::into_canonical_type)
                    .collect(),
            ),
            Self::Reference(element, mutable) => {
                Type::Reference(Box::new(element.into_canonical_type()), mutable)
            }
            Self::Application(name, arguments) => Type::Generic(
                name,
                arguments
                    .into_iter()
                    .map(Self::into_canonical_type)
                    .collect(),
            ),
        }
    }
}

pub(crate) fn canonicalize_specialization_type(ty: &Type) -> Type {
    SpecializationTypeKey::from_type(ty).into_canonical_type()
}

pub(crate) fn specialization_types_equal(left: &Type, right: &Type) -> bool {
    SpecializationTypeKey::from_type(left) == SpecializationTypeKey::from_type(right)
}

pub(crate) fn canonical_copydata_source(
    ty: &Type,
    private_sources: &[PrivateTypeSource],
) -> Result<String, String> {
    match ty {
        Type::Named(name) => {
            if let Some(primitive) = PrimitiveKind::from_source_name(name) {
                return Ok(match primitive {
                    PrimitiveKind::Int => "int".to_string(),
                    PrimitiveKind::Float => "float".to_string(),
                    PrimitiveKind::Bool => "bool".to_string(),
                    PrimitiveKind::Char => "char".to_string(),
                });
            }
            for source in private_sources {
                if let Some(source) = source(name) {
                    return Ok(source.replace(", ", ","));
                }
            }
            if valid_source_symbol(name) {
                Ok(name.clone())
            } else {
                Err(format!("invalid CopyData specialization type '{name}'"))
            }
        }
        Type::Array(element, count) => Ok(format!(
            "[{};{count}]",
            canonical_copydata_source(element, private_sources)?
        )),
        Type::Tuple(elements) if elements.len() >= 2 => Ok(format!(
            "({})",
            elements
                .iter()
                .map(|element| canonical_copydata_source(element, private_sources))
                .collect::<Result<Vec<_>, _>>()?
                .join(",")
        )),
        Type::Tuple(_) => {
            Err("CopyData specialization tuples require arity at least two".to_string())
        }
        Type::Reference(_, _) => {
            Err("CopyData specialization identity cannot contain a reference".to_string())
        }
        Type::Generic(name, arguments) => {
            if !valid_source_symbol(name) || arguments.is_empty() {
                return Err("invalid CopyData specialization application".to_string());
            }
            Ok(format!(
                "{name}<{}>",
                arguments
                    .iter()
                    .map(|argument| canonical_copydata_source(argument, private_sources))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(",")
            ))
        }
    }
}

pub(crate) fn parse_canonical_copydata_type(source: &str) -> Option<Type> {
    let ty = CanonicalTypeParser::new(source).parse_complete().ok()?;
    (canonical_copydata_source(&ty, &[]).ok()?.as_str() == source).then_some(ty)
}

pub(crate) fn parse_canonical_copydata_type_list(source: &str) -> Option<Vec<Type>> {
    let types = CanonicalTypeParser::new(source).parse_type_list().ok()?;
    let canonical = types
        .iter()
        .map(|ty| canonical_copydata_source(ty, &[]))
        .collect::<Result<Vec<_>, _>>()
        .ok()?
        .join(",");
    (canonical == source).then_some(types)
}

pub(crate) fn parse_canonical_application(source: &str) -> Option<(String, Vec<Type>)> {
    let opening = source.find('<')?;
    if !source.ends_with('>') {
        return None;
    }
    let name = &source[..opening];
    if !valid_source_symbol(name) {
        return None;
    }
    let arguments = parse_canonical_copydata_type_list(&source[opening + 1..source.len() - 1])?;
    if arguments.is_empty() {
        return None;
    }
    Some((name.to_string(), arguments))
}

pub(crate) fn valid_source_symbol(name: &str) -> bool {
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

pub(crate) fn encode_hex(value: &str) -> String {
    value
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(crate) fn decode_canonical_hex(encoded: &str) -> Option<String> {
    if encoded.is_empty()
        || !encoded.len().is_multiple_of(2)
        || !encoded.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return None;
    }
    let bytes = (0..encoded.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&encoded[index..index + 2], 16).ok())
        .collect::<Option<Vec<_>>>()?;
    let decoded = String::from_utf8(bytes).ok()?;
    (encode_hex(&decoded) == encoded).then_some(decoded)
}

pub(crate) fn private_identity(prefix: &str, parts: &[&str]) -> String {
    debug_assert!(!parts.is_empty() && parts.iter().all(|part| !part.is_empty()));
    format!("{prefix}{}", encode_hex(&parts.join("|")))
}

pub(crate) fn decode_private_identity(
    prefix: &str,
    name: &str,
    part_count: usize,
) -> Option<Vec<String>> {
    let payload = decode_canonical_hex(name.strip_prefix(prefix)?)?;
    let parts = payload.split('|').map(str::to_string).collect::<Vec<_>>();
    (parts.len() == part_count && parts.iter().all(|part| !part.is_empty())).then_some(parts)
}

pub(crate) fn logical_signature_key(parameters: &[LogicalType], result: &LogicalType) -> String {
    format!(
        "({})->{result}",
        parameters
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(crate) fn normalize_copydata_specializations(
    ast: Vec<AstNode>,
) -> Result<Vec<AstNode>, String> {
    let ast = crate::generic_struct_contract::normalize_generic_copydata_structs(ast)?;
    let ast = crate::generic_enum_contract::normalize_generic_copydata_enums(ast)?;
    let ast = crate::generic_function_contract::normalize_generic_copydata_functions(ast)?;
    Ok(order_private_specializations(ast))
}

fn order_private_specializations(ast: Vec<AstNode>) -> Vec<AstNode> {
    let mut structs = Vec::new();
    let mut enums = Vec::new();
    let mut functions = Vec::new();
    let mut retained = Vec::new();
    for node in ast {
        match &node {
            AstNode::Statement(crate::ast::Statement::StructDef { name, .. })
                if name
                    .starts_with(crate::generic_struct_contract::PRIVATE_GENERIC_STRUCT_PREFIX) =>
            {
                structs.push(node);
            }
            AstNode::Statement(crate::ast::Statement::EnumDef { name, .. })
                if name.starts_with(crate::generic_enum_contract::PRIVATE_GENERIC_ENUM_PREFIX) =>
            {
                enums.push(node);
            }
            AstNode::Statement(crate::ast::Statement::Function { name, .. })
                if name.starts_with(
                    crate::generic_function_contract::PRIVATE_GENERIC_FUNCTION_PREFIX,
                ) || name
                    .starts_with(crate::copydata_trait_dispatch::PRIVATE_TRAIT_IMPL_PREFIX) =>
            {
                functions.push(node);
            }
            _ => retained.push(node),
        }
    }
    structs.sort_by(|left, right| {
        private_specialization_name(left).cmp(private_specialization_name(right))
    });
    enums.sort_by(|left, right| {
        private_specialization_name(left).cmp(private_specialization_name(right))
    });
    functions.sort_by(|left, right| {
        private_specialization_name(left).cmp(private_specialization_name(right))
    });
    structs.extend(enums);
    structs.extend(functions);
    structs.extend(retained);
    structs
}

fn private_specialization_name(node: &AstNode) -> &str {
    match node {
        AstNode::Statement(crate::ast::Statement::StructDef { name, .. })
        | AstNode::Statement(crate::ast::Statement::EnumDef { name, .. })
        | AstNode::Statement(crate::ast::Statement::Function { name, .. }) => name,
        _ => unreachable!("specialization order contains private declarations only"),
    }
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
            return Err("invalid canonical specialization type".to_string());
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
            return Err("invalid canonical specialization type list".to_string());
        }
        Ok(types)
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        match self.peek() {
            Some(b'[') => self.parse_array(),
            Some(b'(') => self.parse_tuple(),
            Some(_) => self.parse_named_or_generic(),
            None => Err("incomplete canonical specialization type".to_string()),
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
            return Err("canonical specialization tuples require arity two or greater".to_string());
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
            return Err("invalid canonical specialization name".to_string());
        }
        let name = std::str::from_utf8(&self.source[start..self.position])
            .map_err(|_| "invalid UTF-8 canonical specialization name".to_string())?
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
            return Err("missing canonical specialization array count".to_string());
        }
        std::str::from_utf8(&self.source[start..self.position])
            .map_err(|_| "invalid canonical specialization array count".to_string())?
            .parse()
            .map_err(|_| "invalid canonical specialization array count".to_string())
    }

    fn expect(&mut self, expected: u8) -> Result<(), String> {
        if self.peek() != Some(expected) {
            return Err("invalid canonical specialization type".to_string());
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

    #[test]
    fn canonical_keys_close_aliases_without_collapsing_distinct_types() {
        for (left, right) in [("int", "i32"), ("float", "f64")] {
            assert!(specialization_types_equal(
                &Type::Named(left.to_string()),
                &Type::Named(right.to_string())
            ));
        }
        assert!(!specialization_types_equal(
            &Type::Named("char".to_string()),
            &Type::Named("int".to_string())
        ));
        assert!(!specialization_types_equal(
            &Type::Named("Reading".to_string()),
            &Type::Named("reading".to_string())
        ));
    }

    #[test]
    fn canonical_keys_preserve_every_nonalias_shape_dimension() {
        let named = |name: &str| Type::Named(name.to_string());
        let array = |element: Type, count| Type::Array(Box::new(element), count);
        let tuple = |elements| Type::Tuple(elements);
        let application = |name: &str, arguments| Type::Generic(name.to_string(), arguments);
        for (left, right) in [
            (named("int"), named("char")),
            (named("int"), named("bool")),
            (named("Reading"), named("Sensor")),
            (array(named("int"), 2), array(named("int"), 3)),
            (
                tuple(vec![named("int"), named("char")]),
                tuple(vec![named("char"), named("int")]),
            ),
            (
                tuple(vec![named("int"), named("char")]),
                tuple(vec![named("int"), named("char"), named("bool")]),
            ),
            (
                application("Box", vec![named("int")]),
                application("Sample", vec![named("int")]),
            ),
            (
                application("Box", vec![named("int")]),
                application("Box", vec![named("char")]),
            ),
            (
                Type::Reference(Box::new(named("int")), false),
                Type::Reference(Box::new(named("int")), true),
            ),
        ] {
            assert!(
                !specialization_types_equal(&left, &right),
                "distinct specialization shapes collapsed: {left:?} and {right:?}"
            );
        }
    }

    #[test]
    fn canonicalization_is_recursive_and_preserves_shape() {
        let source = Type::Generic(
            "Box".to_string(),
            vec![Type::Tuple(vec![
                Type::Array(Box::new(Type::Named("i32".to_string())), 2),
                Type::Named("f64".to_string()),
                Type::Named("char".to_string()),
            ])],
        );
        let canonical = canonicalize_specialization_type(&source);
        assert_eq!(
            canonical_copydata_source(&canonical, &[]).expect("canonical source"),
            "Box<([int;2],float,char)>"
        );
        assert_ne!(
            SpecializationTypeKey::from_type(&Type::Array(
                Box::new(Type::Named("int".to_string())),
                2
            )),
            SpecializationTypeKey::from_type(&Type::Array(
                Box::new(Type::Named("int".to_string())),
                3
            ))
        );
    }

    #[test]
    fn canonical_parser_rejects_alias_and_malformed_spellings() {
        assert!(parse_canonical_copydata_type("Box<(int,[char;2])>").is_some());
        assert!(parse_canonical_copydata_type_list("int,[char;2]").is_some());
        for source in [
            "i32", "f64", "Box<i32>", "(int)", "[int;]", "Box<>", "Box<int", "int,",
        ] {
            assert!(
                parse_canonical_copydata_type(source).is_none(),
                "noncanonical source was accepted: {source}"
            );
        }
    }

    #[test]
    fn feature_framing_is_canonical_and_collision_free() {
        let left = private_identity("__struct$", &["Box<int>", "int"]);
        let right = private_identity("__enum$", &["Box<int>", "int"]);
        let changed_schema = private_identity("__struct$", &["Box<int>", "char"]);
        assert_ne!(left, right);
        assert_ne!(left, changed_schema);
        assert_eq!(
            decode_private_identity("__struct$", &left, 2),
            Some(vec!["Box<int>".to_string(), "int".to_string()])
        );
        assert!(decode_private_identity("__enum$", &left, 2).is_none());
        let uppercase_payload = format!(
            "__struct${}",
            left.strip_prefix("__struct$")
                .expect("test prefix")
                .to_uppercase()
        );
        assert!(decode_private_identity("__struct$", &uppercase_payload, 2).is_none());
        assert!(decode_private_identity("__struct$", "__struct$0", 2).is_none());
        assert!(decode_private_identity("__struct$", "__struct$zz", 2).is_none());
        assert!(
            decode_private_identity(
                "__struct$",
                &format!("__struct${}", encode_hex("Box<int>")),
                2
            )
            .is_none()
        );
        assert_ne!(
            logical_signature_key(&[LogicalType::Int], &LogicalType::Int),
            logical_signature_key(&[LogicalType::Char], &LogicalType::Int)
        );
        assert_ne!(
            logical_signature_key(&[LogicalType::Int], &LogicalType::Int),
            logical_signature_key(&[LogicalType::Int], &LogicalType::Char)
        );
    }

    #[test]
    fn shared_phase_plan_is_idempotent_for_composed_alias_specializations() {
        let source = r#"
struct Box<T> { value: T }
enum Sample<T> { Present(T), Missing }
fn identity<T>(value: T) -> T { value }
fn main() -> int {
    let boxed: Box<i32> = Box { value: 40 };
    let sample: Sample<f64> = Sample::Present(1.5);
    match sample {
        Sample::Present(value) => identity(boxed.value) + 2,
        Sample::Missing => 0,
    }
}
"#;
        let tokens =
            crate::lexer::try_tokenize_with_locations(source, None).expect("fixture must lex");
        let ast = crate::parser::parse_with_locations(tokens).expect("fixture must parse");
        let once = normalize_copydata_specializations(ast).expect("first shared phase plan");
        let twice =
            normalize_copydata_specializations(once.clone()).expect("idempotent shared phase plan");
        assert_eq!(format!("{once:?}"), format!("{twice:?}"));
        let rendered = format!("{once:?}");
        assert!(!rendered.contains("Named(\"i32\")"));
        assert!(!rendered.contains("Named(\"f64\")"));
    }

    #[test]
    fn shared_phase_plan_is_invariant_to_source_declaration_order() {
        let first = r#"
struct Box<T> { value: T }
enum Sample<T> { Present(T), Missing }
fn identity<T>(value: T) -> T { value }
fn main() -> int {
    let boxed: Box<i32> = Box { value: 40 };
    let sample: Sample<f64> = Sample::Present(1.5);
    match sample {
        Sample::Present(value) => identity(boxed.value) + 2,
        Sample::Missing => 0,
    }
}
"#;
        let permuted = r#"
fn identity<T>(value: T) -> T { value }
enum Sample<T> { Present(T), Missing }
struct Box<T> { value: T }
fn main() -> int {
    let boxed: Box<i32> = Box { value: 40 };
    let sample: Sample<f64> = Sample::Present(1.5);
    match sample {
        Sample::Present(value) => identity(boxed.value) + 2,
        Sample::Missing => 0,
    }
}
"#;
        let normalize = |source| {
            let tokens =
                crate::lexer::try_tokenize_with_locations(source, None).expect("fixture must lex");
            let ast = crate::parser::parse_with_locations(tokens).expect("fixture must parse");
            normalize_copydata_specializations(ast).expect("shared phase plan")
        };
        assert_eq!(
            format!("{:?}", normalize(first)),
            format!("{:?}", normalize(permuted))
        );
    }
}
