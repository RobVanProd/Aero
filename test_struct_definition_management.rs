// test_struct_definition_management.rs
// Standalone test for struct definition management

use std::collections::HashMap;

// Minimal AST types needed for testing
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Named(String),
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
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<String>, // Simplified for testing
    pub expression: Option<String>,
}

// Type system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Ty {
    Int,
    Float,
    Bool,
    Struct(String),
    Enum(String),
    Array(Box<Ty>, Option<usize>),
    Vec(Box<Ty>),
    Reference(Box<Ty>),
}

impl Ty {
    pub fn to_string(&self) -> String {
        match self {
            Ty::Int => "int".to_string(),
            Ty::Float => "float".to_string(),
            Ty::Bool => "bool".to_string(),
            Ty::Struct(name) => name.clone(),
            Ty::Enum(name) => name.clone(),
            Ty::Array(element_type, size) => {
                match size {
                    Some(s) => format!("[{}; {}]", element_type.to_string(), s),
                    None => format!("[{}]", element_type.to_string()),
                }
            }
            Ty::Vec(element_type) => format!("Vec<{}>", element_type.to_string()),
            Ty::Reference(inner_type) => format!("&{}", inner_type.to_string()),
        }
    }
    
    pub fn from_string(s: &str) -> Option<Ty> {
        match s {
            "int" => Some(Ty::Int),
            "float" => Some(Ty::Float),
            "bool" => Some(Ty::Bool),
            _ => None,
        }
    }
}

/// Memory layout information for data structures
#[derive(Debug, Clone)]
pub struct MemoryLayout {
    pub size: usize,
    pub alignment: usize,
    pub field_offsets: Vec<usize>,
}

/// Struct definition with type information and layout
#[derive(Debug, Clone)]
pub struct StructDefinition {
    pub name: String,
    pub generics: Vec<String>,
    pub fields: Vec<StructField>,
    pub is_tuple: bool,
    pub layout: MemoryLayout,
}

/// Implementation block for methods
#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub generics: Vec<String>,
    pub type_name: String,
    pub trait_name: Option<String>,
    pub methods: Vec<Function>,
}

/// Type Definition Manager - manages struct and enum definitions
pub struct TypeDefinitionManager {
    structs: HashMap<String, StructDefinition>,
    impls: HashMap<String, Vec<ImplBlock>>,
}

impl TypeDefinitionManager {
    /// Create a new Type Definition Manager
    pub fn new() -> Self {
        Self {
            structs: HashMap::new(),
            impls: HashMap::new(),
        }
    }

    /// Define a new struct type
    pub fn define_struct(&mut self, def: StructDefinition) -> Result<(), String> {
        let name = def.name.clone();
        
        // Check if struct already exists
        if self.structs.contains_key(&name) {
            return Err(format!("Struct '{}' is already defined", name));
        }
        
        // Validate struct definition
        self.validate_struct_definition(&def)?;
        
        // Store the struct definition
        self.structs.insert(name, def);
        Ok(())
    }

    /// Get a struct definition by name
    pub fn get_struct(&self, name: &str) -> Option<&StructDefinition> {
        self.structs.get(name)
    }

    /// Validate field access for a struct
    pub fn validate_field_access(&self, type_name: &str, field: &str) -> Result<Ty, String> {
        let struct_def = self.get_struct(type_name)
            .ok_or_else(|| format!("Undefined struct type: {}", type_name))?;

        // Find the field
        for struct_field in &struct_def.fields {
            if struct_field.name == field {
                // Convert AST Type to Ty
                return self.ast_type_to_ty(&struct_field.field_type);
            }
        }

        Err(format!("Field '{}' not found in struct '{}'", field, type_name))
    }

    /// Validate struct instantiation
    pub fn validate_struct_instantiation(&self, type_name: &str, provided_fields: &[(String, Ty)]) -> Result<(), String> {
        let struct_def = self.get_struct(type_name)
            .ok_or_else(|| format!("Undefined struct type: {}", type_name))?;

        // Check if all required fields are provided
        for struct_field in &struct_def.fields {
            let field_found = provided_fields.iter()
                .any(|(name, _)| name == &struct_field.name);
            
            if !field_found {
                return Err(format!("Missing field '{}' in struct '{}' instantiation", 
                    struct_field.name, type_name));
            }
        }

        // Check if provided fields exist and have correct types
        for (field_name, provided_type) in provided_fields {
            let mut field_found = false;
            
            for struct_field in &struct_def.fields {
                if struct_field.name == *field_name {
                    field_found = true;
                    let expected_type = self.ast_type_to_ty(&struct_field.field_type)?;
                    
                    if *provided_type != expected_type {
                        return Err(format!("Type mismatch for field '{}' in struct '{}': expected {}, got {}", 
                            field_name, type_name, expected_type.to_string(), provided_type.to_string()));
                    }
                    break;
                }
            }
            
            if !field_found {
                return Err(format!("Unknown field '{}' in struct '{}'", field_name, type_name));
            }
        }

        Ok(())
    }

    /// Add an implementation block for a type
    pub fn add_impl(&mut self, impl_block: ImplBlock) -> Result<(), String> {
        let type_name = impl_block.type_name.clone();
        
        // Validate that the type exists
        if !self.structs.contains_key(&type_name) {
            return Err(format!("Cannot implement methods for undefined type: {}", type_name));
        }
        
        // Add to implementations
        self.impls.entry(type_name).or_insert_with(Vec::new).push(impl_block);
        Ok(())
    }

    /// Get a method for a type
    pub fn get_method(&self, type_name: &str, method_name: &str) -> Option<&Function> {
        if let Some(impl_blocks) = self.impls.get(type_name) {
            for impl_block in impl_blocks {
                for method in &impl_block.methods {
                    if method.name == method_name {
                        return Some(method);
                    }
                }
            }
        }
        None
    }

    /// Get all methods for a type
    pub fn get_methods(&self, type_name: &str) -> Vec<&Function> {
        let mut methods = Vec::new();
        if let Some(impl_blocks) = self.impls.get(type_name) {
            for impl_block in impl_blocks {
                for method in &impl_block.methods {
                    methods.push(method);
                }
            }
        }
        methods
    }

    /// Validate a struct definition
    fn validate_struct_definition(&self, def: &StructDefinition) -> Result<(), String> {
        // Check for duplicate field names
        let mut field_names = std::collections::HashSet::new();
        for field in &def.fields {
            if !field_names.insert(&field.name) {
                return Err(format!("Duplicate field '{}' in struct '{}'", field.name, def.name));
            }
        }

        // Validate field types
        for field in &def.fields {
            self.validate_ast_type(&field.field_type)?;
        }

        Ok(())
    }

    /// Validate an AST type
    fn validate_ast_type(&self, ast_type: &Type) -> Result<(), String> {
        match ast_type {
            Type::Named(name) => {
                // Check if it's a primitive type or defined struct
                if Ty::from_string(name).is_none() && 
                   !self.structs.contains_key(name) {
                    return Err(format!("Undefined type: {}", name));
                }
            }
            Type::Array { element_type, size: _ } => {
                self.validate_ast_type(element_type)?;
            }
            Type::Vec { element_type } => {
                self.validate_ast_type(element_type)?;
            }
            Type::Reference { mutable: _, inner_type } => {
                self.validate_ast_type(inner_type)?;
            }
        }
        Ok(())
    }

    /// Convert AST Type to Ty
    fn ast_type_to_ty(&self, ast_type: &Type) -> Result<Ty, String> {
        match ast_type {
            Type::Named(name) => {
                if let Some(ty) = Ty::from_string(name) {
                    Ok(ty)
                } else if self.structs.contains_key(name) {
                    Ok(Ty::Struct(name.clone()))
                } else {
                    Err(format!("Unknown type: {}", name))
                }
            }
            Type::Array { element_type, size } => {
                let elem_ty = self.ast_type_to_ty(element_type)?;
                Ok(Ty::Array(Box::new(elem_ty), *size))
            }
            Type::Vec { element_type } => {
                let elem_ty = self.ast_type_to_ty(element_type)?;
                Ok(Ty::Vec(Box::new(elem_ty)))
            }
            Type::Reference { mutable: _, inner_type } => {
                let inner_ty = self.ast_type_to_ty(inner_type)?;
                Ok(Ty::Reference(Box::new(inner_ty)))
            }
        }
    }
}

fn create_test_memory_layout() -> MemoryLayout {
    MemoryLayout {
        size: 16,
        alignment: 8,
        field_offsets: vec![0, 8],
    }
}

fn main() {
    println!("Testing Struct Definition Management System");
    
    let mut manager = TypeDefinitionManager::new();
    
    // Test 1: Define a simple struct
    println!("\n=== Test 1: Define a simple struct ===");
    let point_struct = StructDefinition {
        name: "Point".to_string(),
        generics: vec![],
        fields: vec![
            StructField {
                name: "x".to_string(),
                field_type: Type::Named("int".to_string()),
                visibility: Visibility::Public,
            },
            StructField {
                name: "y".to_string(),
                field_type: Type::Named("int".to_string()),
                visibility: Visibility::Public,
            },
        ],
        is_tuple: false,
        layout: create_test_memory_layout(),
    };

    match manager.define_struct(point_struct) {
        Ok(()) => println!("✓ Successfully defined Point struct"),
        Err(e) => println!("✗ Failed to define Point struct: {}", e),
    }

    // Test 2: Validate field access
    println!("\n=== Test 2: Validate field access ===");
    match manager.validate_field_access("Point", "x") {
        Ok(ty) => println!("✓ Field 'x' has type: {}", ty.to_string()),
        Err(e) => println!("✗ Field access validation failed: {}", e),
    }

    match manager.validate_field_access("Point", "z") {
        Ok(ty) => println!("✗ Unexpected success for invalid field 'z': {}", ty.to_string()),
        Err(e) => println!("✓ Correctly rejected invalid field 'z': {}", e),
    }

    // Test 3: Validate struct instantiation
    println!("\n=== Test 3: Validate struct instantiation ===");
    let valid_fields = vec![
        ("x".to_string(), Ty::Int),
        ("y".to_string(), Ty::Int),
    ];

    match manager.validate_struct_instantiation("Point", &valid_fields) {
        Ok(()) => println!("✓ Valid struct instantiation accepted"),
        Err(e) => println!("✗ Valid struct instantiation rejected: {}", e),
    }

    let invalid_fields = vec![
        ("x".to_string(), Ty::Int),
        ("y".to_string(), Ty::Float), // Type mismatch
    ];

    match manager.validate_struct_instantiation("Point", &invalid_fields) {
        Ok(()) => println!("✗ Invalid struct instantiation incorrectly accepted"),
        Err(e) => println!("✓ Invalid struct instantiation correctly rejected: {}", e),
    }

    // Test 4: Add implementation block
    println!("\n=== Test 4: Add implementation block ===");
    let impl_block = ImplBlock {
        generics: vec![],
        type_name: "Point".to_string(),
        trait_name: None,
        methods: vec![
            Function {
                name: "new".to_string(),
                parameters: vec![
                    Parameter {
                        name: "x".to_string(),
                        param_type: Type::Named("int".to_string()),
                    },
                    Parameter {
                        name: "y".to_string(),
                        param_type: Type::Named("int".to_string()),
                    },
                ],
                return_type: Some(Type::Named("Point".to_string())),
                body: Block {
                    statements: vec![],
                    expression: None,
                },
            },
            Function {
                name: "distance".to_string(),
                parameters: vec![
                    Parameter {
                        name: "self".to_string(),
                        param_type: Type::Reference {
                            mutable: false,
                            inner_type: Box::new(Type::Named("Point".to_string())),
                        },
                    },
                    Parameter {
                        name: "other".to_string(),
                        param_type: Type::Reference {
                            mutable: false,
                            inner_type: Box::new(Type::Named("Point".to_string())),
                        },
                    },
                ],
                return_type: Some(Type::Named("float".to_string())),
                body: Block {
                    statements: vec![],
                    expression: None,
                },
            },
        ],
    };

    match manager.add_impl(impl_block) {
        Ok(()) => println!("✓ Successfully added implementation block"),
        Err(e) => println!("✗ Failed to add implementation block: {}", e),
    }

    // Test 5: Get method
    println!("\n=== Test 5: Get method ===");
    match manager.get_method("Point", "new") {
        Some(method) => println!("✓ Found method 'new' with {} parameters", method.parameters.len()),
        None => println!("✗ Method 'new' not found"),
    }

    match manager.get_method("Point", "nonexistent") {
        Some(_) => println!("✗ Unexpectedly found nonexistent method"),
        None => println!("✓ Correctly didn't find nonexistent method"),
    }

    // Test 6: Get all methods
    println!("\n=== Test 6: Get all methods ===");
    let methods = manager.get_methods("Point");
    println!("✓ Found {} methods for Point", methods.len());
    for method in methods {
        println!("  - {}", method.name);
    }

    // Test 7: Duplicate struct definition
    println!("\n=== Test 7: Duplicate struct definition ===");
    let duplicate_struct = StructDefinition {
        name: "Point".to_string(),
        generics: vec![],
        fields: vec![],
        is_tuple: false,
        layout: create_test_memory_layout(),
    };

    match manager.define_struct(duplicate_struct) {
        Ok(()) => println!("✗ Duplicate struct definition incorrectly accepted"),
        Err(e) => println!("✓ Duplicate struct definition correctly rejected: {}", e),
    }

    // Test 8: Tuple struct
    println!("\n=== Test 8: Tuple struct ===");
    let color_struct = StructDefinition {
        name: "Color".to_string(),
        generics: vec![],
        fields: vec![
            StructField {
                name: "0".to_string(),
                field_type: Type::Named("int".to_string()),
                visibility: Visibility::Public,
            },
            StructField {
                name: "1".to_string(),
                field_type: Type::Named("int".to_string()),
                visibility: Visibility::Public,
            },
            StructField {
                name: "2".to_string(),
                field_type: Type::Named("int".to_string()),
                visibility: Visibility::Public,
            },
        ],
        is_tuple: true,
        layout: create_test_memory_layout(),
    };

    match manager.define_struct(color_struct) {
        Ok(()) => {
            println!("✓ Successfully defined tuple struct Color");
            if let Some(color_def) = manager.get_struct("Color") {
                println!("  - is_tuple: {}", color_def.is_tuple);
                println!("  - fields: {}", color_def.fields.len());
            }
        },
        Err(e) => println!("✗ Failed to define tuple struct: {}", e),
    }

    println!("\n=== All tests completed ===");
}