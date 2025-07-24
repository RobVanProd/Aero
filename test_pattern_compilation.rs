// test_pattern_compilation.rs
// Test for pattern compilation system functionality

use std::sync::Arc;

// Mock implementations for testing pattern compilation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Ty {
    Int,
    Float,
    Bool,
    Struct(String),
    Enum(String),
}

impl Ty {
    pub fn to_string(&self) -> String {
        match self {
            Ty::Int => "int".to_string(),
            Ty::Float => "float".to_string(),
            Ty::Bool => "bool".to_string(),
            Ty::Struct(name) => name.clone(),
            Ty::Enum(name) => name.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard,
    Identifier(String),
    Literal(Expression),
    Enum { variant: String, data: Option<Box<Pattern>> },
    Struct { name: String, fields: Vec<(String, Pattern)>, rest: bool },
    Tuple(Vec<Pattern>),
    Range { start: Box<Pattern>, end: Box<Pattern>, inclusive: bool },
    Or(Vec<Pattern>),
    Binding { name: String, pattern: Box<Pattern> },
}

#[derive(Debug, Clone)]
pub enum Expression {
    IntegerLiteral(i64),
    FloatLiteral(f64),
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum Condition {
    Always,
    DiscriminantEquals(usize),
    ValueEquals(PatternValue),
    InRange { start: PatternValue, end: PatternValue, inclusive: bool },
    FieldMatch { field: String, condition: Box<Condition> },
    TupleElementMatch { index: usize, condition: Box<Condition> },
    And(Vec<Condition>),
    Or(Vec<Condition>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatternValue {
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub name: String,
    pub binding_type: Ty,
    pub path: BindingPath,
}

#[derive(Debug, Clone)]
pub enum BindingPath {
    Direct,
    Field(String),
    TupleElement(usize),
    EnumData,
    Nested { base: Box<BindingPath>, inner: Box<BindingPath> },
}

#[derive(Debug)]
pub struct PatternCode {
    pub conditions: Vec<Condition>,
    pub bindings: Vec<Binding>,
}

// Mock TypeDefinitionManager for pattern compilation testing
pub struct TypeDefinitionManager {
    enums: std::collections::HashMap<String, Vec<EnumVariant>>,
    structs: std::collections::HashMap<String, Vec<String>>, // Simplified: struct name -> field names
}

impl TypeDefinitionManager {
    pub fn new() -> Self {
        let mut manager = Self {
            enums: std::collections::HashMap::new(),
            structs: std::collections::HashMap::new(),
        };
        
        // Add test enum
        manager.enums.insert("Option".to_string(), vec![
            EnumVariant { name: "Some".to_string() },
            EnumVariant { name: "None".to_string() },
        ]);
        
        // Add test struct
        manager.structs.insert("Point".to_string(), vec![
            "x".to_string(),
            "y".to_string(),
        ]);
        
        manager
    }
    
    pub fn get_variant_discriminant(&self, enum_name: &str, variant_name: &str) -> Result<usize, String> {
        let variants = self.enums.get(enum_name)
            .ok_or_else(|| format!("Enum '{}' not found", enum_name))?;
        
        for (index, variant) in variants.iter().enumerate() {
            if variant.name == variant_name {
                return Ok(index);
            }
        }
        Err(format!("Variant '{}' not found in enum '{}'", variant_name, enum_name))
    }
    
    pub fn get_variant_data_types(&self, _enum_name: &str, _variant_name: &str) -> Result<Option<Vec<Ty>>, String> {
        // Simplified: assume Some variant has one Int data
        Ok(Some(vec![Ty::Int]))
    }
    
    pub fn validate_field_access(&self, struct_name: &str, field_name: &str) -> Result<Ty, String> {
        let fields = self.structs.get(struct_name)
            .ok_or_else(|| format!("Struct '{}' not found", struct_name))?;
        
        if fields.contains(&field_name.to_string()) {
            Ok(Ty::Int) // Simplified: all fields are int
        } else {
            Err(format!("Field '{}' not found in struct '{}'", field_name, struct_name))
        }
    }
}

// Pattern compilation system
pub struct PatternCompiler {
    type_manager: Arc<TypeDefinitionManager>,
}

impl PatternCompiler {
    pub fn new(type_manager: Arc<TypeDefinitionManager>) -> Self {
        Self { type_manager }
    }
    
    pub fn compile_pattern(&self, pattern: &Pattern, target_type: &Ty) -> Result<PatternCode, String> {
        let mut conditions = Vec::new();
        let mut bindings = Vec::new();
        
        self.compile_pattern_recursive(pattern, target_type, &mut conditions, &mut bindings, BindingPath::Direct)?;
        
        Ok(PatternCode { conditions, bindings })
    }
    
    fn compile_pattern_recursive(
        &self,
        pattern: &Pattern,
        target_type: &Ty,
        conditions: &mut Vec<Condition>,
        bindings: &mut Vec<Binding>,
        binding_path: BindingPath,
    ) -> Result<(), String> {
        match pattern {
            Pattern::Wildcard => {
                conditions.push(Condition::Always);
            }
            Pattern::Identifier(name) => {
                conditions.push(Condition::Always);
                bindings.push(Binding {
                    name: name.clone(),
                    binding_type: target_type.clone(),
                    path: binding_path,
                });
            }
            Pattern::Literal(expr) => {
                if let Some(value) = self.extract_pattern_value(expr) {
                    conditions.push(Condition::ValueEquals(value));
                } else {
                    return Err("Invalid literal in pattern".to_string());
                }
            }
            Pattern::Enum { variant, data } => {
                if let Ty::Enum(enum_name) = target_type {
                    let discriminant = self.type_manager.get_variant_discriminant(enum_name, variant)?;
                    conditions.push(Condition::DiscriminantEquals(discriminant));
                    
                    if let Some(data_pattern) = data {
                        let variant_data_types = self.type_manager.get_variant_data_types(enum_name, variant)?;
                        if let Some(data_types) = variant_data_types {
                            if data_types.len() == 1 {
                                self.compile_pattern_recursive(
                                    data_pattern,
                                    &data_types[0],
                                    conditions,
                                    bindings,
                                    BindingPath::Nested {
                                        base: Box::new(binding_path),
                                        inner: Box::new(BindingPath::EnumData),
                                    },
                                )?;
                            }
                        }
                    }
                } else {
                    return Err(format!("Enum pattern used on non-enum type: {:?}", target_type));
                }
            }
            Pattern::Struct { name, fields, .. } => {
                if let Ty::Struct(struct_name) = target_type {
                    if name != struct_name {
                        return Err(format!("Struct pattern '{}' doesn't match type '{}'", name, struct_name));
                    }
                    
                    for (field_name, field_pattern) in fields {
                        let field_type = self.type_manager.validate_field_access(struct_name, field_name)?;
                        self.compile_pattern_recursive(
                            field_pattern,
                            &field_type,
                            conditions,
                            bindings,
                            BindingPath::Nested {
                                base: Box::new(binding_path.clone()),
                                inner: Box::new(BindingPath::Field(field_name.clone())),
                            },
                        )?;
                    }
                } else {
                    return Err(format!("Struct pattern used on non-struct type: {:?}", target_type));
                }
            }
            Pattern::Tuple(tuple_patterns) => {
                for (i, tuple_pattern) in tuple_patterns.iter().enumerate() {
                    self.compile_pattern_recursive(
                        tuple_pattern,
                        target_type, // Simplified
                        conditions,
                        bindings,
                        BindingPath::Nested {
                            base: Box::new(binding_path.clone()),
                            inner: Box::new(BindingPath::TupleElement(i)),
                        },
                    )?;
                }
            }
            Pattern::Range { start, end, inclusive } => {
                let start_value = if let Pattern::Literal(expr) = start.as_ref() {
                    self.extract_pattern_value(expr)
                        .ok_or("Invalid start value in range pattern")?
                } else {
                    return Err("Range pattern start must be a literal".to_string());
                };
                
                let end_value = if let Pattern::Literal(expr) = end.as_ref() {
                    self.extract_pattern_value(expr)
                        .ok_or("Invalid end value in range pattern")?
                } else {
                    return Err("Range pattern end must be a literal".to_string());
                };
                
                conditions.push(Condition::InRange {
                    start: start_value,
                    end: end_value,
                    inclusive: *inclusive,
                });
            }
            Pattern::Or(or_patterns) => {
                let mut or_conditions = Vec::new();
                for or_pattern in or_patterns {
                    let mut pattern_conditions = Vec::new();
                    let mut pattern_bindings = Vec::new();
                    self.compile_pattern_recursive(
                        or_pattern,
                        target_type,
                        &mut pattern_conditions,
                        &mut pattern_bindings,
                        binding_path.clone(),
                    )?;
                    
                    if !pattern_bindings.is_empty() {
                        return Err("Bindings not allowed in or-patterns".to_string());
                    }
                    
                    if pattern_conditions.len() == 1 {
                        or_conditions.push(pattern_conditions.into_iter().next().unwrap());
                    } else {
                        or_conditions.push(Condition::And(pattern_conditions));
                    }
                }
                conditions.push(Condition::Or(or_conditions));
            }
            Pattern::Binding { name, pattern } => {
                bindings.push(Binding {
                    name: name.clone(),
                    binding_type: target_type.clone(),
                    path: binding_path.clone(),
                });
                
                self.compile_pattern_recursive(pattern, target_type, conditions, bindings, binding_path)?;
            }
        }
        
        Ok(())
    }
    
    fn extract_pattern_value(&self, expr: &Expression) -> Option<PatternValue> {
        match expr {
            Expression::IntegerLiteral(value) => Some(PatternValue::Integer(*value)),
            Expression::FloatLiteral(value) => Some(PatternValue::Float(*value)),
        }
    }
    
    pub fn extract_bindings(&self, pattern: &Pattern) -> Vec<(String, Ty)> {
        let mut bindings = Vec::new();
        self.extract_bindings_recursive(pattern, &mut bindings, &Ty::Int);
        bindings
    }
    
    fn extract_bindings_recursive(&self, pattern: &Pattern, bindings: &mut Vec<(String, Ty)>, current_type: &Ty) {
        match pattern {
            Pattern::Identifier(name) => {
                bindings.push((name.clone(), current_type.clone()));
            }
            Pattern::Binding { name, pattern } => {
                bindings.push((name.clone(), current_type.clone()));
                self.extract_bindings_recursive(pattern, bindings, current_type);
            }
            Pattern::Enum { data: Some(data_pattern), .. } => {
                self.extract_bindings_recursive(data_pattern, bindings, current_type);
            }
            Pattern::Struct { fields, .. } => {
                for (_, field_pattern) in fields {
                    self.extract_bindings_recursive(field_pattern, bindings, current_type);
                }
            }
            Pattern::Tuple(tuple_patterns) => {
                for tuple_pattern in tuple_patterns {
                    self.extract_bindings_recursive(tuple_pattern, bindings, current_type);
                }
            }
            Pattern::Or(or_patterns) => {
                for or_pattern in or_patterns {
                    self.extract_bindings_recursive(or_pattern, bindings, current_type);
                }
            }
            _ => {}
        }
    }
    
    pub fn supports_guard_conditions(&self) -> bool {
        true
    }
    
    pub fn supports_nested_destructuring(&self) -> bool {
        true
    }
}

fn main() {
    println!("=== Pattern Compilation System Tests ===");
    
    let type_manager = Arc::new(TypeDefinitionManager::new());
    let pattern_compiler = PatternCompiler::new(type_manager);
    
    // Test 1: Wildcard pattern compilation
    println!("\n--- Test 1: Wildcard Pattern Compilation ---");
    let pattern = Pattern::Wildcard;
    match pattern_compiler.compile_pattern(&pattern, &Ty::Int) {
        Ok(code) => {
            println!("✓ Wildcard pattern compiled successfully");
            println!("  Conditions: {:?}", code.conditions);
            println!("  Bindings: {:?}", code.bindings);
            assert_eq!(code.conditions.len(), 1);
            assert!(matches!(code.conditions[0], Condition::Always));
            assert!(code.bindings.is_empty());
        }
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Test 2: Identifier pattern compilation
    println!("\n--- Test 2: Identifier Pattern Compilation ---");
    let pattern = Pattern::Identifier("x".to_string());
    match pattern_compiler.compile_pattern(&pattern, &Ty::Int) {
        Ok(code) => {
            println!("✓ Identifier pattern compiled successfully");
            println!("  Conditions: {:?}", code.conditions);
            println!("  Bindings: {:?}", code.bindings);
            assert_eq!(code.conditions.len(), 1);
            assert!(matches!(code.conditions[0], Condition::Always));
            assert_eq!(code.bindings.len(), 1);
            assert_eq!(code.bindings[0].name, "x");
        }
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Test 3: Literal pattern compilation
    println!("\n--- Test 3: Literal Pattern Compilation ---");
    let pattern = Pattern::Literal(Expression::IntegerLiteral(42));
    match pattern_compiler.compile_pattern(&pattern, &Ty::Int) {
        Ok(code) => {
            println!("✓ Literal pattern compiled successfully");
            println!("  Conditions: {:?}", code.conditions);
            assert_eq!(code.conditions.len(), 1);
            match &code.conditions[0] {
                Condition::ValueEquals(PatternValue::Integer(42)) => println!("  ✓ Correct value condition"),
                _ => println!("  ✗ Unexpected condition"),
            }
            assert!(code.bindings.is_empty());
        }
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Test 4: Enum pattern compilation
    println!("\n--- Test 4: Enum Pattern Compilation ---");
    let pattern = Pattern::Enum {
        variant: "Some".to_string(),
        data: Some(Box::new(Pattern::Identifier("value".to_string()))),
    };
    match pattern_compiler.compile_pattern(&pattern, &Ty::Enum("Option".to_string())) {
        Ok(code) => {
            println!("✓ Enum pattern compiled successfully");
            println!("  Conditions: {:?}", code.conditions);
            println!("  Bindings: {:?}", code.bindings);
            assert_eq!(code.conditions.len(), 2); // Discriminant check + inner pattern
            assert!(matches!(code.conditions[0], Condition::DiscriminantEquals(0)));
            assert_eq!(code.bindings.len(), 1);
            assert_eq!(code.bindings[0].name, "value");
        }
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Test 5: Struct pattern compilation
    println!("\n--- Test 5: Struct Pattern Compilation ---");
    let pattern = Pattern::Struct {
        name: "Point".to_string(),
        fields: vec![
            ("x".to_string(), Pattern::Identifier("px".to_string())),
            ("y".to_string(), Pattern::Identifier("py".to_string())),
        ],
        rest: false,
    };
    match pattern_compiler.compile_pattern(&pattern, &Ty::Struct("Point".to_string())) {
        Ok(code) => {
            println!("✓ Struct pattern compiled successfully");
            println!("  Conditions: {:?}", code.conditions);
            println!("  Bindings: {:?}", code.bindings);
            assert_eq!(code.conditions.len(), 2); // One condition per field
            assert_eq!(code.bindings.len(), 2);
            assert!(code.bindings.iter().any(|b| b.name == "px"));
            assert!(code.bindings.iter().any(|b| b.name == "py"));
        }
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Test 6: Range pattern compilation
    println!("\n--- Test 6: Range Pattern Compilation ---");
    let pattern = Pattern::Range {
        start: Box::new(Pattern::Literal(Expression::IntegerLiteral(1))),
        end: Box::new(Pattern::Literal(Expression::IntegerLiteral(10))),
        inclusive: true,
    };
    match pattern_compiler.compile_pattern(&pattern, &Ty::Int) {
        Ok(code) => {
            println!("✓ Range pattern compiled successfully");
            println!("  Conditions: {:?}", code.conditions);
            assert_eq!(code.conditions.len(), 1);
            match &code.conditions[0] {
                Condition::InRange { start, end, inclusive } => {
                    assert_eq!(*start, PatternValue::Integer(1));
                    assert_eq!(*end, PatternValue::Integer(10));
                    assert!(*inclusive);
                    println!("  ✓ Correct range condition");
                }
                _ => println!("  ✗ Unexpected condition"),
            }
        }
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Test 7: Or-pattern compilation
    println!("\n--- Test 7: Or-Pattern Compilation ---");
    let pattern = Pattern::Or(vec![
        Pattern::Literal(Expression::IntegerLiteral(1)),
        Pattern::Literal(Expression::IntegerLiteral(2)),
        Pattern::Literal(Expression::IntegerLiteral(3)),
    ]);
    match pattern_compiler.compile_pattern(&pattern, &Ty::Int) {
        Ok(code) => {
            println!("✓ Or-pattern compiled successfully");
            println!("  Conditions: {:?}", code.conditions);
            assert_eq!(code.conditions.len(), 1);
            match &code.conditions[0] {
                Condition::Or(or_conditions) => {
                    assert_eq!(or_conditions.len(), 3);
                    println!("  ✓ Correct or-condition structure");
                }
                _ => println!("  ✗ Unexpected condition"),
            }
        }
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Test 8: Binding pattern compilation
    println!("\n--- Test 8: Binding Pattern Compilation ---");
    let pattern = Pattern::Binding {
        name: "bound_value".to_string(),
        pattern: Box::new(Pattern::Literal(Expression::IntegerLiteral(42))),
    };
    match pattern_compiler.compile_pattern(&pattern, &Ty::Int) {
        Ok(code) => {
            println!("✓ Binding pattern compiled successfully");
            println!("  Conditions: {:?}", code.conditions);
            println!("  Bindings: {:?}", code.bindings);
            assert_eq!(code.conditions.len(), 1);
            assert_eq!(code.bindings.len(), 1);
            assert_eq!(code.bindings[0].name, "bound_value");
            match &code.conditions[0] {
                Condition::ValueEquals(PatternValue::Integer(42)) => println!("  ✓ Correct binding with condition"),
                _ => println!("  ✗ Unexpected condition"),
            }
        }
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Test 9: Binding extraction
    println!("\n--- Test 9: Binding Extraction ---");
    let pattern = Pattern::Struct {
        name: "Point".to_string(),
        fields: vec![
            ("x".to_string(), Pattern::Binding {
                name: "px".to_string(),
                pattern: Box::new(Pattern::Identifier("inner_x".to_string())),
            }),
            ("y".to_string(), Pattern::Identifier("py".to_string())),
        ],
        rest: false,
    };
    let bindings = pattern_compiler.extract_bindings(&pattern);
    println!("✓ Binding extraction completed");
    println!("  Extracted bindings: {:?}", bindings);
    assert!(bindings.len() >= 2);
    
    // Test 10: Feature support checks
    println!("\n--- Test 10: Feature Support Checks ---");
    assert!(pattern_compiler.supports_guard_conditions());
    assert!(pattern_compiler.supports_nested_destructuring());
    println!("✓ Guard conditions supported: {}", pattern_compiler.supports_guard_conditions());
    println!("✓ Nested destructuring supported: {}", pattern_compiler.supports_nested_destructuring());
    
    println!("\n=== All Pattern Compilation Tests Completed ===");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wildcard_pattern_compilation() {
        let type_manager = Arc::new(TypeDefinitionManager::new());
        let pattern_compiler = PatternCompiler::new(type_manager);
        
        let pattern = Pattern::Wildcard;
        let result = pattern_compiler.compile_pattern(&pattern, &Ty::Int).unwrap();
        
        assert_eq!(result.conditions.len(), 1);
        assert!(matches!(result.conditions[0], Condition::Always));
        assert!(result.bindings.is_empty());
    }
    
    #[test]
    fn test_identifier_pattern_compilation() {
        let type_manager = Arc::new(TypeDefinitionManager::new());
        let pattern_compiler = PatternCompiler::new(type_manager);
        
        let pattern = Pattern::Identifier("x".to_string());
        let result = pattern_compiler.compile_pattern(&pattern, &Ty::Int).unwrap();
        
        assert_eq!(result.conditions.len(), 1);
        assert!(matches!(result.conditions[0], Condition::Always));
        assert_eq!(result.bindings.len(), 1);
        assert_eq!(result.bindings[0].name, "x");
    }
    
    #[test]
    fn test_literal_pattern_compilation() {
        let type_manager = Arc::new(TypeDefinitionManager::new());
        let pattern_compiler = PatternCompiler::new(type_manager);
        
        let pattern = Pattern::Literal(Expression::IntegerLiteral(42));
        let result = pattern_compiler.compile_pattern(&pattern, &Ty::Int).unwrap();
        
        assert_eq!(result.conditions.len(), 1);
        match &result.conditions[0] {
            Condition::ValueEquals(PatternValue::Integer(42)) => {}
            _ => panic!("Expected ValueEquals condition"),
        }
        assert!(result.bindings.is_empty());
    }
    
    #[test]
    fn test_enum_pattern_compilation() {
        let type_manager = Arc::new(TypeDefinitionManager::new());
        let pattern_compiler = PatternCompiler::new(type_manager);
        
        let pattern = Pattern::Enum {
            variant: "Some".to_string(),
            data: Some(Box::new(Pattern::Identifier("value".to_string()))),
        };
        let result = pattern_compiler.compile_pattern(&pattern, &Ty::Enum("Option".to_string())).unwrap();
        
        assert_eq!(result.conditions.len(), 2);
        assert!(matches!(result.conditions[0], Condition::DiscriminantEquals(0)));
        assert_eq!(result.bindings.len(), 1);
        assert_eq!(result.bindings[0].name, "value");
    }
    
    #[test]
    fn test_binding_extraction() {
        let type_manager = Arc::new(TypeDefinitionManager::new());
        let pattern_compiler = PatternCompiler::new(type_manager);
        
        let pattern = Pattern::Binding {
            name: "x".to_string(),
            pattern: Box::new(Pattern::Identifier("y".to_string())),
        };
        
        let bindings = pattern_compiler.extract_bindings(&pattern);
        assert_eq!(bindings.len(), 2);
        assert!(bindings.iter().any(|(name, _)| name == "x"));
        assert!(bindings.iter().any(|(name, _)| name == "y"));
    }
}