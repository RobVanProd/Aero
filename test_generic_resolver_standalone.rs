// test_generic_resolver_standalone.rs - Standalone test for Generic Resolver

// This is a standalone test that doesn't depend on the full compiler infrastructure
// It tests the generic resolver functionality in isolation

use std::collections::HashMap;

// Minimal AST types for testing
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Named(String),
    Generic {
        name: String,
        type_args: Vec<Type>,
    },
    Array {
        element_type: Box<Type>,
        size: Option<usize>,
    },
    Vec {
        element_type: Box<Type>,
    },
    Reference {
        mutable: bool,
        inner_type: Box<Type>,
    },
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub field_type: Type,
    pub visibility: Visibility,
}

#[derive(Debug, Clone)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub data: Option<EnumVariantData>,
}

#[derive(Debug, Clone)]
pub enum EnumVariantData {
    Tuple(Vec<Type>),
    Struct(Vec<StructField>),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
}

// Minimal types for testing
#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Int,
    Float,
    Bool,
    Struct(String),
    Enum(String),
}

impl Ty {
    pub fn from_string(s: &str) -> Option<Ty> {
        match s {
            "int" => Some(Ty::Int),
            "float" => Some(Ty::Float),
            "bool" => Some(Ty::Bool),
            "u8" => Some(Ty::Int),
            "u16" => Some(Ty::Int),
            "u32" => Some(Ty::Int),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryLayout {
    pub size: usize,
    pub alignment: usize,
    pub field_offsets: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct StructDefinition {
    pub name: String,
    pub generics: Vec<String>,
    pub fields: Vec<StructField>,
    pub is_tuple: bool,
    pub layout: MemoryLayout,
}

#[derive(Debug, Clone)]
pub struct EnumDefinition {
    pub name: String,
    pub generics: Vec<String>,
    pub variants: Vec<EnumVariant>,
    pub discriminant_type: Ty,
}

// Generic Resolver implementation (simplified for testing)
#[derive(Debug, Clone)]
pub struct GenericInstance {
    pub base_name: String,
    pub type_args: Vec<Type>,
    pub instantiated_name: String,
}

#[derive(Debug, Clone)]
pub enum GenericDefinition {
    Struct {
        name: String,
        generics: Vec<String>,
        fields: Vec<StructField>,
        is_tuple: bool,
    },
    Enum {
        name: String,
        generics: Vec<String>,
        variants: Vec<EnumVariant>,
    },
    Function {
        name: String,
        generics: Vec<String>,
        function: Function,
    },
}

#[derive(Debug, Clone)]
pub enum ConcreteDefinition {
    Struct(StructDefinition),
    Enum(EnumDefinition),
    Function(Function),
}

#[derive(Debug, Clone)]
pub struct GenericConstraint {
    pub type_param: String,
    pub trait_bounds: Vec<String>,
}

pub struct GenericResolver {
    instantiations: HashMap<String, Vec<GenericInstance>>,
    generic_definitions: HashMap<String, GenericDefinition>,
    constraints: HashMap<String, Vec<GenericConstraint>>,
}

impl GenericResolver {
    pub fn new() -> Self {
        Self {
            instantiations: HashMap::new(),
            generic_definitions: HashMap::new(),
            constraints: HashMap::new(),
        }
    }

    pub fn register_generic_struct(&mut self, name: String, generics: Vec<String>, fields: Vec<StructField>, is_tuple: bool) -> Result<(), String> {
        if self.generic_definitions.contains_key(&name) {
            return Err(format!("Generic definition '{}' already exists", name));
        }

        let definition = GenericDefinition::Struct {
            name: name.clone(),
            generics,
            fields,
            is_tuple,
        };

        self.generic_definitions.insert(name, definition);
        Ok(())
    }

    pub fn register_generic_enum(&mut self, name: String, generics: Vec<String>, variants: Vec<EnumVariant>) -> Result<(), String> {
        if self.generic_definitions.contains_key(&name) {
            return Err(format!("Generic definition '{}' already exists", name));
        }

        let definition = GenericDefinition::Enum {
            name: name.clone(),
            generics,
            variants,
        };

        self.generic_definitions.insert(name, definition);
        Ok(())
    }

    pub fn instantiate_generic(&mut self, base_name: &str, type_args: &[Type]) -> Result<String, String> {
        let generic_def = self.generic_definitions.get(base_name)
            .ok_or_else(|| format!("Generic definition '{}' not found", base_name))?;

        let expected_count = match generic_def {
            GenericDefinition::Struct { generics, .. } => generics.len(),
            GenericDefinition::Enum { generics, .. } => generics.len(),
            GenericDefinition::Function { generics, .. } => generics.len(),
        };

        if type_args.len() != expected_count {
            return Err(format!(
                "Generic '{}' expects {} type arguments, but {} were provided",
                base_name, expected_count, type_args.len()
            ));
        }

        let instantiated_name = self.generate_instantiated_name(base_name, type_args);

        if let Some(instances) = self.instantiations.get(base_name) {
            for instance in instances {
                if instance.type_args == type_args {
                    return Ok(instance.instantiated_name.clone());
                }
            }
        }

        let instance = GenericInstance {
            base_name: base_name.to_string(),
            type_args: type_args.to_vec(),
            instantiated_name: instantiated_name.clone(),
        };

        self.instantiations.entry(base_name.to_string()).or_insert_with(Vec::new).push(instance);
        Ok(instantiated_name)
    }

    pub fn monomorphize(&self, base_name: &str, type_args: &[Type]) -> Result<ConcreteDefinition, String> {
        let generic_def = self.generic_definitions.get(base_name)
            .ok_or_else(|| format!("Generic definition '{}' not found", base_name))?;

        match generic_def {
            GenericDefinition::Struct { name, generics, fields, is_tuple } => {
                let type_map = self.create_type_substitution_map(generics, type_args)?;
                let concrete_fields = self.substitute_struct_fields(fields, &type_map)?;
                let concrete_name = self.generate_instantiated_name(name, type_args);
                
                let struct_def = StructDefinition {
                    name: concrete_name,
                    generics: vec![],
                    fields: concrete_fields,
                    is_tuple: *is_tuple,
                    layout: MemoryLayout {
                        size: 8,
                        alignment: 4,
                        field_offsets: vec![0],
                    },
                };
                
                Ok(ConcreteDefinition::Struct(struct_def))
            }
            GenericDefinition::Enum { name, generics, variants } => {
                let type_map = self.create_type_substitution_map(generics, type_args)?;
                let concrete_variants = self.substitute_enum_variants(variants, &type_map)?;
                let concrete_name = self.generate_instantiated_name(name, type_args);
                
                let enum_def = EnumDefinition {
                    name: concrete_name,
                    generics: vec![],
                    variants: concrete_variants,
                    discriminant_type: Ty::Int,
                };
                
                Ok(ConcreteDefinition::Enum(enum_def))
            }
            GenericDefinition::Function { function, .. } => {
                Ok(ConcreteDefinition::Function(function.clone()))
            }
        }
    }

    pub fn get_instantiations(&self, base_name: &str) -> Vec<&GenericInstance> {
        self.instantiations.get(base_name).map(|instances| instances.iter().collect()).unwrap_or_default()
    }

    pub fn is_instantiated(&self, base_name: &str, type_args: &[Type]) -> bool {
        if let Some(instances) = self.instantiations.get(base_name) {
            instances.iter().any(|instance| instance.type_args == type_args)
        } else {
            false
        }
    }

    pub fn get_instantiated_name(&self, base_name: &str, type_args: &[Type]) -> Option<String> {
        if let Some(instances) = self.instantiations.get(base_name) {
            instances.iter()
                .find(|instance| instance.type_args == type_args)
                .map(|instance| instance.instantiated_name.clone())
        } else {
            None
        }
    }

    fn generate_instantiated_name(&self, base_name: &str, type_args: &[Type]) -> String {
        let mut name = base_name.to_string();
        name.push('_');
        
        for (i, arg) in type_args.iter().enumerate() {
            if i > 0 {
                name.push('_');
            }
            name.push_str(&self.type_to_string(arg));
        }
        
        name
    }

    fn type_to_string(&self, ty: &Type) -> String {
        match ty {
            Type::Named(name) => name.clone(),
            Type::Generic { name, type_args } => {
                let mut result = name.clone();
                if !type_args.is_empty() {
                    result.push('_');
                    for (i, arg) in type_args.iter().enumerate() {
                        if i > 0 {
                            result.push('_');
                        }
                        result.push_str(&self.type_to_string(arg));
                    }
                }
                result
            }
            Type::Array { element_type, size } => {
                format!("Array_{}_{}",
                    self.type_to_string(element_type),
                    size.map(|s| s.to_string()).unwrap_or_else(|| "dyn".to_string())
                )
            }
            Type::Vec { element_type } => {
                format!("Vec_{}", self.type_to_string(element_type))
            }
            Type::Reference { mutable, inner_type } => {
                format!("{}Ref_{}", 
                    if *mutable { "Mut" } else { "" },
                    self.type_to_string(inner_type)
                )
            }
        }
    }

    fn create_type_substitution_map(&self, generics: &[String], type_args: &[Type]) -> Result<HashMap<String, Type>, String> {
        if generics.len() != type_args.len() {
            return Err(format!(
                "Generic parameter count mismatch: expected {}, got {}",
                generics.len(), type_args.len()
            ));
        }

        let mut map = HashMap::new();
        for (generic, concrete) in generics.iter().zip(type_args.iter()) {
            map.insert(generic.clone(), concrete.clone());
        }
        Ok(map)
    }

    fn substitute_struct_fields(&self, fields: &[StructField], type_map: &HashMap<String, Type>) -> Result<Vec<StructField>, String> {
        let mut concrete_fields = Vec::new();
        
        for field in fields {
            let concrete_type = self.substitute_type(&field.field_type, type_map)?;
            concrete_fields.push(StructField {
                name: field.name.clone(),
                field_type: concrete_type,
                visibility: field.visibility.clone(),
            });
        }
        
        Ok(concrete_fields)
    }

    fn substitute_enum_variants(&self, variants: &[EnumVariant], type_map: &HashMap<String, Type>) -> Result<Vec<EnumVariant>, String> {
        let mut concrete_variants = Vec::new();
        
        for variant in variants {
            let concrete_data = match &variant.data {
                None => None,
                Some(EnumVariantData::Tuple(types)) => {
                    let mut concrete_types = Vec::new();
                    for ty in types {
                        concrete_types.push(self.substitute_type(ty, type_map)?);
                    }
                    Some(EnumVariantData::Tuple(concrete_types))
                }
                Some(EnumVariantData::Struct(fields)) => {
                    let concrete_fields = self.substitute_struct_fields(fields, type_map)?;
                    Some(EnumVariantData::Struct(concrete_fields))
                }
            };
            
            concrete_variants.push(EnumVariant {
                name: variant.name.clone(),
                data: concrete_data,
            });
        }
        
        Ok(concrete_variants)
    }

    fn substitute_type(&self, ty: &Type, type_map: &HashMap<String, Type>) -> Result<Type, String> {
        match ty {
            Type::Named(name) => {
                if let Some(concrete_type) = type_map.get(name) {
                    Ok(concrete_type.clone())
                } else {
                    Ok(ty.clone())
                }
            }
            Type::Generic { name, type_args } => {
                let mut concrete_args = Vec::new();
                for arg in type_args {
                    concrete_args.push(self.substitute_type(arg, type_map)?);
                }
                
                if let Some(concrete_type) = type_map.get(name) {
                    Ok(concrete_type.clone())
                } else {
                    Ok(Type::Generic {
                        name: name.clone(),
                        type_args: concrete_args,
                    })
                }
            }
            Type::Array { element_type, size } => {
                let concrete_element = self.substitute_type(element_type, type_map)?;
                Ok(Type::Array {
                    element_type: Box::new(concrete_element),
                    size: *size,
                })
            }
            Type::Vec { element_type } => {
                let concrete_element = self.substitute_type(element_type, type_map)?;
                Ok(Type::Vec {
                    element_type: Box::new(concrete_element),
                })
            }
            Type::Reference { mutable, inner_type } => {
                let concrete_inner = self.substitute_type(inner_type, type_map)?;
                Ok(Type::Reference {
                    mutable: *mutable,
                    inner_type: Box::new(concrete_inner),
                })
            }
        }
    }
}

// Test functions
fn test_basic_generic_struct() {
    println!("Testing basic generic struct instantiation...");
    
    let mut resolver = GenericResolver::new();
    
    // Register a generic struct Container<T>
    let fields = vec![
        StructField {
            name: "value".to_string(),
            field_type: Type::Named("T".to_string()),
            visibility: Visibility::Public,
        },
    ];
    
    let result = resolver.register_generic_struct(
        "Container".to_string(),
        vec!["T".to_string()],
        fields,
        false
    );
    
    assert!(result.is_ok(), "Failed to register generic struct");
    
    // Instantiate Container<i32>
    let type_args = vec![Type::Named("i32".to_string())];
    let instantiated_name = resolver.instantiate_generic("Container", &type_args);
    
    assert!(instantiated_name.is_ok(), "Failed to instantiate generic struct");
    assert_eq!(instantiated_name.unwrap(), "Container_i32");
    
    // Check if instantiation is cached
    let cached_name = resolver.get_instantiated_name("Container", &type_args);
    assert!(cached_name.is_some());
    assert_eq!(cached_name.unwrap(), "Container_i32");
    
    println!("✓ Basic generic struct instantiation works correctly");
}

fn test_generic_enum() {
    println!("Testing generic enum instantiation...");
    
    let mut resolver = GenericResolver::new();
    
    // Register a generic enum Option<T>
    let variants = vec![
        EnumVariant {
            name: "Some".to_string(),
            data: Some(EnumVariantData::Tuple(vec![Type::Named("T".to_string())])),
        },
        EnumVariant {
            name: "None".to_string(),
            data: None,
        },
    ];
    
    let result = resolver.register_generic_enum(
        "Option".to_string(),
        vec!["T".to_string()],
        variants
    );
    
    assert!(result.is_ok(), "Failed to register generic enum");
    
    // Instantiate Option<String>
    let type_args = vec![Type::Named("String".to_string())];
    let instantiated_name = resolver.instantiate_generic("Option", &type_args);
    
    assert!(instantiated_name.is_ok(), "Failed to instantiate generic enum");
    assert_eq!(instantiated_name.unwrap(), "Option_String");
    
    println!("✓ Generic enum instantiation works correctly");
}

fn test_multiple_type_parameters() {
    println!("Testing multiple type parameters...");
    
    let mut resolver = GenericResolver::new();
    
    // Register a generic struct Pair<T, U>
    let fields = vec![
        StructField {
            name: "first".to_string(),
            field_type: Type::Named("T".to_string()),
            visibility: Visibility::Public,
        },
        StructField {
            name: "second".to_string(),
            field_type: Type::Named("U".to_string()),
            visibility: Visibility::Public,
        },
    ];
    
    let result = resolver.register_generic_struct(
        "Pair".to_string(),
        vec!["T".to_string(), "U".to_string()],
        fields,
        false
    );
    
    assert!(result.is_ok(), "Failed to register generic struct with multiple parameters");
    
    // Instantiate Pair<i32, String>
    let type_args = vec![
        Type::Named("i32".to_string()),
        Type::Named("String".to_string())
    ];
    let instantiated_name = resolver.instantiate_generic("Pair", &type_args);
    
    assert!(instantiated_name.is_ok(), "Failed to instantiate generic struct with multiple parameters");
    assert_eq!(instantiated_name.unwrap(), "Pair_i32_String");
    
    println!("✓ Multiple type parameters work correctly");
}

fn test_monomorphization() {
    println!("Testing monomorphization...");
    
    let mut resolver = GenericResolver::new();
    
    // Register a generic struct
    let fields = vec![
        StructField {
            name: "value".to_string(),
            field_type: Type::Named("T".to_string()),
            visibility: Visibility::Public,
        },
    ];
    
    resolver.register_generic_struct(
        "Container".to_string(),
        vec!["T".to_string()],
        fields,
        false
    ).unwrap();
    
    // Monomorphize Container<i32>
    let type_args = vec![Type::Named("i32".to_string())];
    let concrete_def = resolver.monomorphize("Container", &type_args);
    
    assert!(concrete_def.is_ok(), "Failed to monomorphize generic struct");
    
    match concrete_def.unwrap() {
        ConcreteDefinition::Struct(struct_def) => {
            assert_eq!(struct_def.name, "Container_i32");
            assert_eq!(struct_def.generics.len(), 0); // No generics in concrete definition
            assert_eq!(struct_def.fields.len(), 1);
            assert_eq!(struct_def.fields[0].field_type, Type::Named("i32".to_string()));
        }
        _ => panic!("Expected concrete struct definition"),
    }
    
    println!("✓ Monomorphization works correctly");
}

fn test_error_cases() {
    println!("Testing error cases...");
    
    let mut resolver = GenericResolver::new();
    
    // Test 1: Wrong number of type arguments
    let fields = vec![
        StructField {
            name: "value".to_string(),
            field_type: Type::Named("T".to_string()),
            visibility: Visibility::Public,
        },
    ];
    
    resolver.register_generic_struct(
        "Container".to_string(),
        vec!["T".to_string()],
        fields,
        false
    ).unwrap();
    
    // Try to instantiate with wrong number of arguments
    let wrong_args = vec![
        Type::Named("i32".to_string()),
        Type::Named("String".to_string()),
    ];
    let result = resolver.instantiate_generic("Container", &wrong_args);
    
    assert!(result.is_err(), "Should fail with wrong number of type arguments");
    assert!(result.unwrap_err().contains("expects 1 type arguments, but 2 were provided"));
    
    // Test 2: Undefined generic
    let result = resolver.instantiate_generic("UndefinedType", &[Type::Named("i32".to_string())]);
    assert!(result.is_err(), "Should fail with undefined generic");
    assert!(result.unwrap_err().contains("not found"));
    
    println!("✓ Error cases handled correctly");
}

fn main() {
    println!("=== Generic Resolver Standalone Tests ===");
    
    test_basic_generic_struct();
    test_generic_enum();
    test_multiple_type_parameters();
    test_monomorphization();
    test_error_cases();
    
    println!("=== All Generic Resolver Tests Passed! ===");
}