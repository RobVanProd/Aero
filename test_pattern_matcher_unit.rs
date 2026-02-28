// test_pattern_matcher_unit.rs
// Unit test for pattern matcher functionality

use std::sync::Arc;

// Mock implementations for testing
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
    Or(Vec<Pattern>),
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

// Mock TypeDefinitionManager
pub struct TypeDefinitionManager {
    enums: std::collections::HashMap<String, Vec<EnumVariant>>,
}

impl TypeDefinitionManager {
    pub fn new() -> Self {
        let mut manager = Self {
            enums: std::collections::HashMap::new(),
        };
        
        // Add a test enum
        manager.enums.insert("Color".to_string(), vec![
            EnumVariant { name: "Red".to_string() },
            EnumVariant { name: "Green".to_string() },
            EnumVariant { name: "Blue".to_string() },
        ]);
        
        manager
    }
    
    pub fn get_enum_variants(&self, enum_name: &str) -> Result<&[EnumVariant], String> {
        self.enums.get(enum_name)
            .map(|v| v.as_slice())
            .ok_or_else(|| format!("Enum '{}' not found", enum_name))
    }
    
    pub fn get_variant_discriminant(&self, enum_name: &str, variant_name: &str) -> Result<usize, String> {
        let variants = self.get_enum_variants(enum_name)?;
        for (index, variant) in variants.iter().enumerate() {
            if variant.name == variant_name {
                return Ok(index);
            }
        }
        Err(format!("Variant '{}' not found in enum '{}'", variant_name, enum_name))
    }
    
    pub fn get_variant_data_types(&self, _enum_name: &str, _variant_name: &str) -> Result<Option<Vec<Ty>>, String> {
        Ok(None) // Simplified for testing
    }
    
    pub fn validate_field_access(&self, _struct_name: &str, _field_name: &str) -> Result<Ty, String> {
        Ok(Ty::Int) // Simplified for testing
    }
}

// Simplified PatternMatcher for testing
pub struct PatternMatcher {
    type_manager: Arc<TypeDefinitionManager>,
}

#[derive(Debug)]
pub enum ExhaustivenessResult {
    Exhaustive,
    Missing(Vec<String>),
    Unreachable(Vec<usize>),
}

impl PatternMatcher {
    pub fn new(type_manager: Arc<TypeDefinitionManager>) -> Self {
        Self { type_manager }
    }
    
    pub fn check_exhaustiveness(&self, patterns: &[Pattern], match_type: &Ty) -> Result<ExhaustivenessResult, String> {
        match match_type {
            Ty::Enum(enum_name) => self.check_enum_exhaustiveness(patterns, enum_name),
            Ty::Bool => self.check_bool_exhaustiveness(patterns),
            _ => {
                if patterns.iter().any(|p| matches!(p, Pattern::Wildcard | Pattern::Identifier(_))) {
                    Ok(ExhaustivenessResult::Exhaustive)
                } else {
                    Ok(ExhaustivenessResult::Missing(vec!["_ (wildcard)".to_string()]))
                }
            }
        }
    }
    
    fn check_enum_exhaustiveness(&self, patterns: &[Pattern], enum_name: &str) -> Result<ExhaustivenessResult, String> {
        let variants = self.type_manager.get_enum_variants(enum_name)?;
        let mut covered_variants = std::collections::HashSet::new();
        let mut has_wildcard = false;
        let mut unreachable_patterns = Vec::new();
        
        for (pattern_index, pattern) in patterns.iter().enumerate() {
            match pattern {
                Pattern::Enum { variant, .. } => {
                    if covered_variants.contains(variant) || has_wildcard {
                        unreachable_patterns.push(pattern_index);
                    } else {
                        if !variants.iter().any(|v| &v.name == variant) {
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
                        for i in (pattern_index + 1)..patterns.len() {
                            unreachable_patterns.push(i);
                        }
                        break;
                    }
                }
                Pattern::Or(or_patterns) => {
                    for or_pattern in or_patterns {
                        if let Pattern::Enum { variant, .. } = or_pattern {
                            if !covered_variants.contains(variant) && !has_wildcard {
                                if !variants.iter().any(|v| &v.name == variant) {
                                    return Err(format!("Unknown variant '{}' for enum '{}'", variant, enum_name));
                                }
                                covered_variants.insert(variant.clone());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        
        if has_wildcard || covered_variants.len() == variants.len() {
            if unreachable_patterns.is_empty() {
                Ok(ExhaustivenessResult::Exhaustive)
            } else {
                Ok(ExhaustivenessResult::Unreachable(unreachable_patterns))
            }
        } else {
            let missing_variants: Vec<String> = variants
                .iter()
                .filter(|v| !covered_variants.contains(&v.name))
                .map(|v| format!("{}::{}", enum_name, v.name))
                .collect();
            
            Ok(ExhaustivenessResult::Missing(missing_variants))
        }
    }
    
    fn check_bool_exhaustiveness(&self, patterns: &[Pattern]) -> Result<ExhaustivenessResult, String> {
        let mut has_true = false;
        let mut has_false = false;
        let mut has_wildcard = false;
        
        for pattern in patterns {
            match pattern {
                Pattern::Literal(Expression::IntegerLiteral(1)) => has_true = true, // Simplified
                Pattern::Literal(Expression::IntegerLiteral(0)) => has_false = true, // Simplified
                Pattern::Wildcard | Pattern::Identifier(_) => has_wildcard = true,
                _ => {}
            }
        }
        
        if has_wildcard || (has_true && has_false) {
            Ok(ExhaustivenessResult::Exhaustive)
        } else {
            let mut missing = Vec::new();
            if !has_true { missing.push("true".to_string()); }
            if !has_false { missing.push("false".to_string()); }
            Ok(ExhaustivenessResult::Missing(missing))
        }
    }
    
    pub fn is_irrefutable(&self, pattern: &Pattern) -> bool {
        matches!(pattern, Pattern::Wildcard | Pattern::Identifier(_))
    }
}

fn main() {
    println!("=== Pattern Matcher Unit Tests ===");
    
    let type_manager = Arc::new(TypeDefinitionManager::new());
    let pattern_matcher = PatternMatcher::new(type_manager);
    
    // Test 1: Complete enum exhaustiveness
    println!("\n--- Test 1: Complete Enum Exhaustiveness ---");
    let patterns = vec![
        Pattern::Enum { variant: "Red".to_string(), data: None },
        Pattern::Enum { variant: "Green".to_string(), data: None },
        Pattern::Enum { variant: "Blue".to_string(), data: None },
    ];
    
    match pattern_matcher.check_exhaustiveness(&patterns, &Ty::Enum("Color".to_string())) {
        Ok(ExhaustivenessResult::Exhaustive) => println!("✓ Complete enum patterns are exhaustive"),
        Ok(result) => println!("✗ Unexpected result: {:?}", result),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Test 2: Missing enum pattern
    println!("\n--- Test 2: Missing Enum Pattern ---");
    let patterns = vec![
        Pattern::Enum { variant: "Red".to_string(), data: None },
        Pattern::Enum { variant: "Green".to_string(), data: None },
    ];
    
    match pattern_matcher.check_exhaustiveness(&patterns, &Ty::Enum("Color".to_string())) {
        Ok(ExhaustivenessResult::Missing(missing)) => {
            println!("✓ Missing patterns detected: {:?}", missing);
            assert!(missing.contains(&"Color::Blue".to_string()));
        }
        Ok(result) => println!("✗ Unexpected result: {:?}", result),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Test 3: Wildcard makes patterns exhaustive
    println!("\n--- Test 3: Wildcard Pattern ---");
    let patterns = vec![
        Pattern::Enum { variant: "Red".to_string(), data: None },
        Pattern::Wildcard,
    ];
    
    match pattern_matcher.check_exhaustiveness(&patterns, &Ty::Enum("Color".to_string())) {
        Ok(ExhaustivenessResult::Exhaustive) => println!("✓ Wildcard makes patterns exhaustive"),
        Ok(result) => println!("✗ Unexpected result: {:?}", result),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Test 4: Unreachable pattern detection
    println!("\n--- Test 4: Unreachable Pattern Detection ---");
    let patterns = vec![
        Pattern::Enum { variant: "Red".to_string(), data: None },
        Pattern::Wildcard,
        Pattern::Enum { variant: "Green".to_string(), data: None }, // Unreachable
    ];
    
    match pattern_matcher.check_exhaustiveness(&patterns, &Ty::Enum("Color".to_string())) {
        Ok(ExhaustivenessResult::Unreachable(unreachable)) => {
            println!("✓ Unreachable patterns detected at positions: {:?}", unreachable);
            assert!(unreachable.contains(&2));
        }
        Ok(result) => println!("✗ Unexpected result: {:?}", result),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Test 5: Or-patterns
    println!("\n--- Test 5: Or-Patterns ---");
    let patterns = vec![
        Pattern::Or(vec![
            Pattern::Enum { variant: "Red".to_string(), data: None },
            Pattern::Enum { variant: "Green".to_string(), data: None },
        ]),
        Pattern::Enum { variant: "Blue".to_string(), data: None },
    ];
    
    match pattern_matcher.check_exhaustiveness(&patterns, &Ty::Enum("Color".to_string())) {
        Ok(ExhaustivenessResult::Exhaustive) => println!("✓ Or-patterns work correctly"),
        Ok(result) => println!("✗ Unexpected result: {:?}", result),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    // Test 6: Irrefutable patterns
    println!("\n--- Test 6: Irrefutable Patterns ---");
    assert!(pattern_matcher.is_irrefutable(&Pattern::Wildcard));
    assert!(pattern_matcher.is_irrefutable(&Pattern::Identifier("x".to_string())));
    assert!(!pattern_matcher.is_irrefutable(&Pattern::Enum { variant: "Red".to_string(), data: None }));
    println!("✓ Irrefutable pattern detection works correctly");
    
    // Test 7: Non-enum types
    println!("\n--- Test 7: Non-Enum Types ---");
    let patterns = vec![
        Pattern::Literal(Expression::IntegerLiteral(42)),
    ];
    
    match pattern_matcher.check_exhaustiveness(&patterns, &Ty::Int) {
        Ok(ExhaustivenessResult::Missing(missing)) => {
            println!("✓ Non-enum types require wildcard: {:?}", missing);
            assert!(missing.contains(&"_ (wildcard)".to_string()));
        }
        Ok(result) => println!("✗ Unexpected result: {:?}", result),
        Err(e) => println!("✗ Error: {}", e),
    }
    
    println!("\n=== All Pattern Matcher Unit Tests Completed ===");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_complete_enum_exhaustiveness() {
        let type_manager = Arc::new(TypeDefinitionManager::new());
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
    fn test_missing_enum_pattern() {
        let type_manager = Arc::new(TypeDefinitionManager::new());
        let pattern_matcher = PatternMatcher::new(type_manager);
        
        let patterns = vec![
            Pattern::Enum { variant: "Red".to_string(), data: None },
            Pattern::Enum { variant: "Green".to_string(), data: None },
        ];
        
        let result = pattern_matcher.check_exhaustiveness(&patterns, &Ty::Enum("Color".to_string())).unwrap();
        match result {
            ExhaustivenessResult::Missing(missing) => {
                assert!(missing.contains(&"Color::Blue".to_string()));
            }
            _ => panic!("Expected missing patterns"),
        }
    }
    
    #[test]
    fn test_wildcard_exhaustiveness() {
        let type_manager = Arc::new(TypeDefinitionManager::new());
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
        let type_manager = Arc::new(TypeDefinitionManager::new());
        let pattern_matcher = PatternMatcher::new(type_manager);
        
        let patterns = vec![
            Pattern::Enum { variant: "Red".to_string(), data: None },
            Pattern::Wildcard,
            Pattern::Enum { variant: "Green".to_string(), data: None }, // Unreachable
        ];
        
        let result = pattern_matcher.check_exhaustiveness(&patterns, &Ty::Enum("Color".to_string())).unwrap();
        match result {
            ExhaustivenessResult::Unreachable(unreachable) => {
                assert!(unreachable.contains(&2));
            }
            _ => panic!("Expected unreachable patterns"),
        }
    }
    
    #[test]
    fn test_irrefutable_patterns() {
        let type_manager = Arc::new(TypeDefinitionManager::new());
        let pattern_matcher = PatternMatcher::new(type_manager);
        
        assert!(pattern_matcher.is_irrefutable(&Pattern::Wildcard));
        assert!(pattern_matcher.is_irrefutable(&Pattern::Identifier("x".to_string())));
        assert!(!pattern_matcher.is_irrefutable(&Pattern::Enum { variant: "Red".to_string(), data: None }));
    }
}