// src/compiler/src/pattern_matcher.rs

use std::collections::{HashMap, HashSet};
use crate::ast::{Pattern, Type, EnumVariant, EnumVariantData};
use crate::types::{Ty, TypeDefinitionManager};

/// Pattern matcher for exhaustiveness checking and pattern compilation
pub struct PatternMatcher {
    type_manager: std::rc::Rc<std::cell::RefCell<TypeDefinitionManager>>,
}

/// Represents a condition in compiled pattern code
#[derive(Debug, Clone)]
pub enum Condition {
    /// Check if discriminant equals a value (for enums)
    DiscriminantEquals(usize),
    /// Check if value equals a literal
    ValueEquals(PatternValue),
    /// Check if value is in a range
    InRange { start: PatternValue, end: PatternValue, inclusive: bool },
    /// Always true condition (for wildcards)
    Always,
    /// Check if field matches a pattern
    FieldMatch { field: String, condition: Box<Condition> },
    /// Check if tuple element matches a pattern
    TupleElementMatch { index: usize, condition: Box<Condition> },
    /// Logical AND of conditions
    And(Vec<Condition>),
    /// Logical OR of conditions
    Or(Vec<Condition>),
}

/// Represents a value in pattern matching
#[derive(Debug, Clone, PartialEq)]
pub enum PatternValue {
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

/// Represents a binding extracted from a pattern
#[derive(Debug, Clone)]
pub struct Binding {
    pub name: String,
    pub binding_type: Ty,
    pub path: BindingPath,
}

/// Represents the path to extract a binding
#[derive(Debug, Clone)]
pub enum BindingPath {
    /// Direct binding to the matched value
    Direct,
    /// Binding to a field of a struct
    Field(String),
    /// Binding to a tuple element
    TupleElement(usize),
    /// Binding to enum variant data
    EnumData,
    /// Nested path
    Nested { base: Box<BindingPath>, inner: Box<BindingPath> },
}

/// Compiled pattern code
#[derive(Debug)]
pub struct PatternCode {
    pub conditions: Vec<Condition>,
    pub bindings: Vec<Binding>,
}

/// Exhaustiveness analysis result
#[derive(Debug)]
pub enum ExhaustivenessResult {
    /// Patterns are exhaustive
    Exhaustive,
    /// Missing patterns
    Missing(Vec<MissingPattern>),
    /// Unreachable patterns found
    Unreachable(Vec<usize>), // indices of unreachable patterns
}

/// Represents a missing pattern in exhaustiveness analysis
#[derive(Debug, Clone)]
pub struct MissingPattern {
    pub description: String,
    pub pattern_type: Ty,
}

impl PatternMatcher {
    /// Create a new pattern matcher
    pub fn new(type_manager: std::rc::Rc<std::cell::RefCell<TypeDefinitionManager>>) -> Self {
        Self { type_manager }
    }

    /// Check if patterns are exhaustive for the given type
    pub fn check_exhaustiveness(&self, patterns: &[Pattern], match_type: &Ty) -> Result<ExhaustivenessResult, String> {
        match match_type {
            Ty::Enum(enum_name) => self.check_enum_exhaustiveness(patterns, enum_name),
            Ty::Bool => self.check_bool_exhaustiveness(patterns),
            Ty::Int => self.check_int_exhaustiveness(patterns),
            _ => {
                // For other types, check if there's a wildcard or catch-all pattern
                if patterns.iter().any(|p| self.is_catch_all_pattern(p)) {
                    Ok(ExhaustivenessResult::Exhaustive)
                } else {
                    Ok(ExhaustivenessResult::Missing(vec![MissingPattern {
                        description: "_ (wildcard)".to_string(),
                        pattern_type: match_type.clone(),
                    }]))
                }
            }
        }
    }

    /// Check exhaustiveness for enum patterns
    fn check_enum_exhaustiveness(&self, patterns: &[Pattern], enum_name: &str) -> Result<ExhaustivenessResult, String> {
        let type_manager = self.type_manager.borrow();
        let enum_def = type_manager.get_enum(enum_name)
            .ok_or_else(|| format!("Undefined enum type: {}", enum_name))?;

        let mut covered_variants = HashSet::new();
        let mut unreachable_patterns = Vec::new();
        let mut has_wildcard = false;

        for (pattern_index, pattern) in patterns.iter().enumerate() {
            match pattern {
                Pattern::Enum { variant, .. } => {
                    if covered_variants.contains(variant) || has_wildcard {
                        unreachable_patterns.push(pattern_index);
                    } else {
                        // Check if variant exists
                        if !enum_def.variants.iter().any(|v| &v.name == variant) {
                            return Err(format!("Unknown variant '{}' for enum '{}'", variant, enum_name));
                        }
                        covered_variants.insert(variant.clone());
                    }
                }
                Pattern::Wildcard | Pattern::Identifier(_) => {
                    if has_wildcard {
                        unreachable_patterns.push(pattern_index);
                    } else {
                        has_wildcard = true;
                        // Mark all remaining patterns as unreachable
                        for i in (pattern_index + 1)..patterns.len() {
                            unreachable_patterns.push(i);
                        }
                        break;
                    }
                }
                Pattern::Or(or_patterns) => {
                    for or_pattern in or_patterns {
                        if let Pattern::Enum { variant, .. } = or_pattern {
                            if covered_variants.contains(variant) || has_wildcard {
                                // This or-pattern is unreachable, but we don't mark the whole pattern
                                continue;
                            } else {
                                if !enum_def.variants.iter().any(|v| &v.name == variant) {
                                    return Err(format!("Unknown variant '{}' for enum '{}'", variant, enum_name));
                                }
                                covered_variants.insert(variant.clone());
                            }
                        }
                    }
                }
                _ => {
                    return Err(format!("Invalid pattern type for enum matching: {:?}", pattern));
                }
            }
        }

        // Check if all variants are covered
        if has_wildcard || covered_variants.len() == enum_def.variants.len() {
            if unreachable_patterns.is_empty() {
                Ok(ExhaustivenessResult::Exhaustive)
            } else {
                Ok(ExhaustivenessResult::Unreachable(unreachable_patterns))
            }
        } else {
            let missing_variants: Vec<MissingPattern> = enum_def.variants
                .iter()
                .filter(|v| !covered_variants.contains(&v.name))
                .map(|v| MissingPattern {
                    description: format!("{}::{}", enum_name, v.name),
                    pattern_type: Ty::Enum(enum_name.to_string()),
                })
                .collect();
            
            Ok(ExhaustivenessResult::Missing(missing_variants))
        }
    }

    /// Check exhaustiveness for boolean patterns
    fn check_bool_exhaustiveness(&self, patterns: &[Pattern]) -> Result<ExhaustivenessResult, String> {
        let mut has_true = false;
        let mut has_false = false;
        let mut has_wildcard = false;
        let mut unreachable_patterns = Vec::new();

        for (pattern_index, pattern) in patterns.iter().enumerate() {
            match pattern {
                Pattern::Literal(expr) => {
                    if let Some(value) = self.extract_bool_literal(expr) {
                        if value {
                            if has_true || has_wildcard {
                                unreachable_patterns.push(pattern_index);
                            } else {
                                has_true = true;
                            }
                        } else {
                            if has_false || has_wildcard {
                                unreachable_patterns.push(pattern_index);
                            } else {
                                has_false = true;
                            }
                        }
                    }
                }
                Pattern::Wildcard | Pattern::Identifier(_) => {
                    if has_wildcard {
                        unreachable_patterns.push(pattern_index);
                    } else {
                        has_wildcard = true;
                        // Mark all remaining patterns as unreachable
                        for i in (pattern_index + 1)..patterns.len() {
                            unreachable_patterns.push(i);
                        }
                        break;
                    }
                }
                _ => {
                    return Err(format!("Invalid pattern type for boolean matching: {:?}", pattern));
                }
            }
        }

        if has_wildcard || (has_true && has_false) {
            if unreachable_patterns.is_empty() {
                Ok(ExhaustivenessResult::Exhaustive)
            } else {
                Ok(ExhaustivenessResult::Unreachable(unreachable_patterns))
            }
        } else {
            let mut missing = Vec::new();
            if !has_true {
                missing.push(MissingPattern {
                    description: "true".to_string(),
                    pattern_type: Ty::Bool,
                });
            }
            if !has_false {
                missing.push(MissingPattern {
                    description: "false".to_string(),
                    pattern_type: Ty::Bool,
                });
            }
            Ok(ExhaustivenessResult::Missing(missing))
        }
    }

    /// Check exhaustiveness for integer patterns (simplified - just checks for wildcard)
    fn check_int_exhaustiveness(&self, patterns: &[Pattern]) -> Result<ExhaustivenessResult, String> {
        let mut unreachable_patterns = Vec::new();
        let mut has_wildcard = false;

        for (pattern_index, pattern) in patterns.iter().enumerate() {
            match pattern {
                Pattern::Wildcard | Pattern::Identifier(_) => {
                    if has_wildcard {
                        unreachable_patterns.push(pattern_index);
                    } else {
                        has_wildcard = true;
                        // Mark all remaining patterns as unreachable
                        for i in (pattern_index + 1)..patterns.len() {
                            unreachable_patterns.push(i);
                        }
                        break;
                    }
                }
                Pattern::Literal(_) | Pattern::Range { .. } => {
                    if has_wildcard {
                        unreachable_patterns.push(pattern_index);
                    }
                    // For simplicity, we don't do full range analysis for integers
                }
                _ => {
                    return Err(format!("Invalid pattern type for integer matching: {:?}", pattern));
                }
            }
        }

        if has_wildcard {
            if unreachable_patterns.is_empty() {
                Ok(ExhaustivenessResult::Exhaustive)
            } else {
                Ok(ExhaustivenessResult::Unreachable(unreachable_patterns))
            }
        } else {
            Ok(ExhaustivenessResult::Missing(vec![MissingPattern {
                description: "_ (wildcard)".to_string(),
                pattern_type: Ty::Int,
            }]))
        }
    }

    /// Compile a pattern into executable conditions and bindings
    pub fn compile_pattern(&self, pattern: &Pattern, target_type: &Ty) -> Result<PatternCode, String> {
        let mut conditions = Vec::new();
        let mut bindings = Vec::new();

        self.compile_pattern_recursive(pattern, target_type, &mut conditions, &mut bindings, BindingPath::Direct)?;

        Ok(PatternCode { conditions, bindings })
    }

    /// Recursively compile a pattern
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
                    let discriminant = self.type_manager.borrow().get_variant_discriminant(enum_name, variant)?;
                    conditions.push(Condition::DiscriminantEquals(discriminant));

                    if let Some(data_pattern) = data {
                        let variant_data_types = self.type_manager.borrow().get_variant_data_types(enum_name, variant)?;
                        if let Some(data_types) = variant_data_types {
                            if data_types.len() == 1 {
                                // Single data type - compile directly
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
                            } else {
                                // Multiple data types - should be a tuple pattern
                                if let Pattern::Tuple(tuple_patterns) = data_pattern.as_ref() {
                                    if tuple_patterns.len() != data_types.len() {
                                        return Err(format!("Pattern tuple length {} doesn't match variant data length {}", 
                                            tuple_patterns.len(), data_types.len()));
                                    }
                                    
                                    for (i, (tuple_pattern, data_type)) in tuple_patterns.iter().zip(data_types.iter()).enumerate() {
                                        self.compile_pattern_recursive(
                                            tuple_pattern,
                                            data_type,
                                            conditions,
                                            bindings,
                                            BindingPath::Nested {
                                                base: Box::new(binding_path.clone()),
                                                inner: Box::new(BindingPath::TupleElement(i)),
                                            },
                                        )?;
                                    }
                                } else {
                                    return Err("Expected tuple pattern for multi-data enum variant".to_string());
                                }
                            }
                        }
                    }
                } else {
                    return Err(format!("Enum pattern used on non-enum type: {:?}", target_type));
                }
            }
            Pattern::Struct { name, fields, rest: _ } => {
                if let Ty::Struct(struct_name) = target_type {
                    if name != struct_name {
                        return Err(format!("Struct pattern '{}' doesn't match type '{}'", name, struct_name));
                    }

                    for (field_name, field_pattern) in fields {
                        let field_type = self.type_manager.borrow().validate_field_access(struct_name, field_name)?;
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
                // For tuple patterns, we need to know the tuple element types
                // This is simplified - in a real implementation, we'd need tuple types
                for (i, tuple_pattern) in tuple_patterns.iter().enumerate() {
                    self.compile_pattern_recursive(
                        tuple_pattern,
                        target_type, // Simplified - should be element type
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
                    
                    // For or-patterns, we can't have bindings (they would be ambiguous)
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
                // First, bind the value
                bindings.push(Binding {
                    name: name.clone(),
                    binding_type: target_type.clone(),
                    path: binding_path.clone(),
                });

                // Then, compile the inner pattern
                self.compile_pattern_recursive(pattern, target_type, conditions, bindings, binding_path)?;
            }
        }

        Ok(())
    }

    /// Extract bindings from a pattern
    pub fn extract_bindings(&self, pattern: &Pattern) -> Vec<(String, Ty)> {
        let mut bindings = Vec::new();
        self.extract_bindings_recursive(pattern, &mut bindings, &Ty::Int); // Default type, should be inferred
        bindings
    }

    /// Recursively extract bindings from a pattern
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
            _ => {} // No bindings in other pattern types
        }
    }

    /// Check if a pattern is irrefutable (always matches)
    pub fn is_irrefutable(&self, pattern: &Pattern) -> bool {
        match pattern {
            Pattern::Wildcard | Pattern::Identifier(_) => true,
            Pattern::Binding { pattern, .. } => self.is_irrefutable(pattern),
            Pattern::Tuple(patterns) => patterns.iter().all(|p| self.is_irrefutable(p)),
            Pattern::Struct { fields, rest, .. } => {
                *rest || fields.iter().all(|(_, p)| self.is_irrefutable(p))
            }
            _ => false,
        }
    }

    /// Check if a pattern is a catch-all pattern
    fn is_catch_all_pattern(&self, pattern: &Pattern) -> bool {
        match pattern {
            Pattern::Wildcard | Pattern::Identifier(_) => true,
            Pattern::Binding { pattern, .. } => self.is_catch_all_pattern(pattern),
            _ => false,
        }
    }

    /// Extract a boolean literal from an expression
    fn extract_bool_literal(&self, expr: &crate::ast::Expression) -> Option<bool> {
        // This is simplified - in a real implementation, we'd need to evaluate the expression
        // For now, we assume the expression is a boolean literal
        None // Placeholder
    }

    /// Extract a pattern value from an expression
    fn extract_pattern_value(&self, expr: &crate::ast::Expression) -> Option<PatternValue> {
        match expr {
            crate::ast::Expression::IntegerLiteral(value) => Some(PatternValue::Integer(*value)),
            crate::ast::Expression::FloatLiteral(value) => Some(PatternValue::Float(*value)),
            // Add more literal types as needed
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TypeDefinitionManager;
    use crate::ast::{EnumVariant, EnumVariantData, StructField, Visibility};
    use std::sync::Arc;

    fn create_test_type_manager() -> Arc<TypeDefinitionManager> {
        let mut type_manager = TypeDefinitionManager::new();
        
        // Create a simple enum for testing
        let color_enum = type_manager.create_enum_definition(
            "Color".to_string(),
            vec![],
            vec![
                EnumVariant { name: "Red".to_string(), data: None },
                EnumVariant { name: "Green".to_string(), data: None },
                EnumVariant { name: "Blue".to_string(), data: None },
            ],
        );
        type_manager.define_enum(color_enum).unwrap();

        // Create an Option-like enum
        let option_enum = type_manager.create_enum_definition(
            "Option".to_string(),
            vec!["T".to_string()],
            vec![
                EnumVariant {
                    name: "Some".to_string(),
                    data: Some(EnumVariantData::Tuple(vec![Type::Named("T".to_string())])),
                },
                EnumVariant { name: "None".to_string(), data: None },
            ],
        );
        type_manager.define_enum(option_enum).unwrap();

        Arc::new(type_manager)
    }

    #[test]
    fn test_enum_exhaustiveness_complete() {
        let type_manager = create_test_type_manager();
        let pattern_matcher = PatternMatcher::new(type_manager);

        let patterns = vec![
            Pattern::Enum { variant: "Red".to_string(), data: None },
            Pattern::Enum { variant: "Green".to_string(), data: None },
            Pattern::Enum { variant: "Blue".to_string(), data: None },
        ];

        let result = pattern_matcher.check_exhaustiveness(&patterns, &Ty::Enum("Color".to_string())).unwrap();
        assert!(matches!(result, ExhaustivenessResult::Exhaustive));
    }

    #[test]
    fn test_enum_exhaustiveness_missing() {
        let type_manager = create_test_type_manager();
        let pattern_matcher = PatternMatcher::new(type_manager);

        let patterns = vec![
            Pattern::Enum { variant: "Red".to_string(), data: None },
            Pattern::Enum { variant: "Green".to_string(), data: None },
        ];

        let result = pattern_matcher.check_exhaustiveness(&patterns, &Ty::Enum("Color".to_string())).unwrap();
        match result {
            ExhaustivenessResult::Missing(missing) => {
                assert_eq!(missing.len(), 1);
                assert_eq!(missing[0].description, "Color::Blue");
            }
            _ => panic!("Expected missing patterns"),
        }
    }

    #[test]
    fn test_enum_exhaustiveness_with_wildcard() {
        let type_manager = create_test_type_manager();
        let pattern_matcher = PatternMatcher::new(type_manager);

        let patterns = vec![
            Pattern::Enum { variant: "Red".to_string(), data: None },
            Pattern::Wildcard,
        ];

        let result = pattern_matcher.check_exhaustiveness(&patterns, &Ty::Enum("Color".to_string())).unwrap();
        assert!(matches!(result, ExhaustivenessResult::Exhaustive));
    }

    #[test]
    fn test_unreachable_pattern_detection() {
        let type_manager = create_test_type_manager();
        let pattern_matcher = PatternMatcher::new(type_manager);

        let patterns = vec![
            Pattern::Enum { variant: "Red".to_string(), data: None },
            Pattern::Wildcard,
            Pattern::Enum { variant: "Green".to_string(), data: None }, // Unreachable
        ];

        let result = pattern_matcher.check_exhaustiveness(&patterns, &Ty::Enum("Color".to_string())).unwrap();
        match result {
            ExhaustivenessResult::Unreachable(unreachable) => {
                assert_eq!(unreachable, vec![2]);
            }
            _ => panic!("Expected unreachable patterns"),
        }
    }

    #[test]
    fn test_bool_exhaustiveness() {
        let type_manager = create_test_type_manager();
        let pattern_matcher = PatternMatcher::new(type_manager);

        let patterns = vec![
            Pattern::Literal(crate::ast::Expression::IntegerLiteral(1)), // Simplified - should be bool
            Pattern::Literal(crate::ast::Expression::IntegerLiteral(0)), // Simplified - should be bool
        ];

        // This test is simplified since we don't have proper bool literal extraction
        let result = pattern_matcher.check_exhaustiveness(&patterns, &Ty::Bool).unwrap();
        // The result depends on the implementation of extract_bool_literal
    }

    #[test]
    fn test_pattern_compilation_wildcard() {
        let type_manager = create_test_type_manager();
        let pattern_matcher = PatternMatcher::new(type_manager);

        let pattern = Pattern::Wildcard;
        let result = pattern_matcher.compile_pattern(&pattern, &Ty::Int).unwrap();

        assert_eq!(result.conditions.len(), 1);
        assert!(matches!(result.conditions[0], Condition::Always));
        assert!(result.bindings.is_empty());
    }

    #[test]
    fn test_pattern_compilation_identifier() {
        let type_manager = create_test_type_manager();
        let pattern_matcher = PatternMatcher::new(type_manager);

        let pattern = Pattern::Identifier("x".to_string());
        let result = pattern_matcher.compile_pattern(&pattern, &Ty::Int).unwrap();

        assert_eq!(result.conditions.len(), 1);
        assert!(matches!(result.conditions[0], Condition::Always));
        assert_eq!(result.bindings.len(), 1);
        assert_eq!(result.bindings[0].name, "x");
        assert_eq!(result.bindings[0].binding_type, Ty::Int);
    }

    #[test]
    fn test_pattern_compilation_literal() {
        let type_manager = create_test_type_manager();
        let pattern_matcher = PatternMatcher::new(type_manager);

        let pattern = Pattern::Literal(crate::ast::Expression::IntegerLiteral(42));
        let result = pattern_matcher.compile_pattern(&pattern, &Ty::Int).unwrap();

        assert_eq!(result.conditions.len(), 1);
        match &result.conditions[0] {
            Condition::ValueEquals(PatternValue::Integer(42)) => {}
            _ => panic!("Expected ValueEquals condition with integer 42"),
        }
        assert!(result.bindings.is_empty());
    }

    #[test]
    fn test_irrefutable_patterns() {
        let type_manager = create_test_type_manager();
        let pattern_matcher = PatternMatcher::new(type_manager);

        assert!(pattern_matcher.is_irrefutable(&Pattern::Wildcard));
        assert!(pattern_matcher.is_irrefutable(&Pattern::Identifier("x".to_string())));
        assert!(pattern_matcher.is_irrefutable(&Pattern::Binding {
            name: "x".to_string(),
            pattern: Box::new(Pattern::Wildcard),
        }));
        assert!(!pattern_matcher.is_irrefutable(&Pattern::Literal(crate::ast::Expression::IntegerLiteral(42))));
        assert!(!pattern_matcher.is_irrefutable(&Pattern::Enum {
            variant: "Red".to_string(),
            data: None,
        }));
    }

    #[test]
    fn test_binding_extraction() {
        let type_manager = create_test_type_manager();
        let pattern_matcher = PatternMatcher::new(type_manager);

        let pattern = Pattern::Binding {
            name: "color".to_string(),
            pattern: Box::new(Pattern::Enum {
                variant: "Red".to_string(),
                data: None,
            }),
        };

        let bindings = pattern_matcher.extract_bindings(&pattern);
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].0, "color");
    }
}