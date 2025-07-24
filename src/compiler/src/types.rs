// src/compiler/src/types.rs

use std::collections::HashMap;
use crate::ast::{Type, StructField, Visibility, Function, EnumVariant, EnumVariantData};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Ty {
    Int,
    Float,
    Bool,
    String,
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
            Ty::String => "String".to_string(),
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
            "String" => Some(Ty::String),
            _ => None,
        }
    }
}

/// Type inference and promotion rules for binary operations
pub fn infer_binary_type(op: &str, lhs: &Ty, rhs: &Ty) -> Result<Ty, String> {
    match op {
        // Arithmetic operations
        "+" => {
            match (lhs, rhs) {
                (Ty::Int, Ty::Int) => Ok(Ty::Int),
                (Ty::Float, Ty::Float) => Ok(Ty::Float),
                (Ty::Int, Ty::Float) | (Ty::Float, Ty::Int) => Ok(Ty::Float), // promote to float
                (Ty::String, Ty::String) => Ok(Ty::String), // string concatenation
                (Ty::String, _) | (_, Ty::String) => Ok(Ty::String), // string concatenation with other types
                _ => Err(format!("Type mismatch in addition operation: {} vs {}", lhs.to_string(), rhs.to_string())),
            }
        }
        "-" | "*" | "/" | "%" => {
            match (lhs, rhs) {
                (Ty::Int, Ty::Int) => Ok(Ty::Int),
                (Ty::Float, Ty::Float) => Ok(Ty::Float),
                (Ty::Int, Ty::Float) | (Ty::Float, Ty::Int) => Ok(Ty::Float), // promote to float
                _ => Err(format!("Type mismatch in arithmetic operation `{}`: {} vs {}", op, lhs.to_string(), rhs.to_string())),
            }
        }
        // Comparison operations
        "==" | "!=" | "<" | ">" | "<=" | ">=" => {
            match (lhs, rhs) {
                (Ty::Int, Ty::Int) | (Ty::Float, Ty::Float) | (Ty::Bool, Ty::Bool) | (Ty::String, Ty::String) => Ok(Ty::Bool),
                (Ty::Int, Ty::Float) | (Ty::Float, Ty::Int) => Ok(Ty::Bool), // allow comparison with promotion
                _ => Err(format!("Type mismatch in comparison operation `{}`: {} vs {}", op, lhs.to_string(), rhs.to_string())),
            }
        }
        // Logical operations
        "&&" | "||" => {
            match (lhs, rhs) {
                (Ty::Bool, Ty::Bool) => Ok(Ty::Bool),
                _ => Err(format!("Logical operation `{}` requires boolean operands: {} vs {}", op, lhs.to_string(), rhs.to_string())),
            }
        }
        _ => Err(format!("Unknown binary operation: {}", op)),
    }
}

/// Check if a type promotion is needed from source to target
pub fn needs_promotion(from: &Ty, to: &Ty) -> bool {
    matches!((from, to), (Ty::Int, Ty::Float))
}

/// Memory layout information for data structures
#[derive(Debug, Clone)]
pub struct MemoryLayout {
    pub size: usize,
    pub alignment: usize,
    pub field_offsets: Vec<usize>,
}

/// Memory layout calculator for structs and enums
pub struct MemoryLayoutCalculator;

impl MemoryLayoutCalculator {
    /// Create a new memory layout calculator
    pub fn new() -> Self {
        Self
    }

    /// Calculate memory layout for a struct
    pub fn calculate_struct_layout(&self, fields: &[StructField]) -> MemoryLayout {
        if fields.is_empty() {
            return MemoryLayout {
                size: 0,
                alignment: 1,
                field_offsets: vec![],
            };
        }

        let mut field_offsets = Vec::new();
        let mut current_offset = 0;
        let mut max_alignment = 1;

        for field in fields {
            let field_size = self.get_type_size(&field.field_type);
            let field_alignment = self.get_type_alignment(&field.field_type);
            
            max_alignment = max_alignment.max(field_alignment);
            
            // Align current offset to field alignment
            current_offset = self.align_to(current_offset, field_alignment);
            field_offsets.push(current_offset);
            
            current_offset += field_size;
        }

        // Align total size to struct alignment
        let total_size = self.align_to(current_offset, max_alignment);

        MemoryLayout {
            size: total_size,
            alignment: max_alignment,
            field_offsets,
        }
    }

    /// Calculate memory layout for an enum
    pub fn calculate_enum_layout(&self, variants: &[EnumVariant]) -> MemoryLayout {
        if variants.is_empty() {
            return MemoryLayout {
                size: 1, // At least 1 byte for discriminant
                alignment: 1,
                field_offsets: vec![],
            };
        }

        let discriminant_size = self.get_discriminant_size(variants.len());
        let discriminant_alignment = discriminant_size;
        
        let mut max_variant_size = 0;
        let mut max_variant_alignment = discriminant_alignment;

        // Calculate the largest variant
        for variant in variants {
            let (variant_size, variant_alignment) = match &variant.data {
                None => (0, 1), // Unit variant
                Some(EnumVariantData::Tuple(types)) => {
                    self.calculate_tuple_layout(types)
                }
                Some(EnumVariantData::Struct(fields)) => {
                    let layout = self.calculate_struct_layout(fields);
                    (layout.size, layout.alignment)
                }
            };
            
            max_variant_size = max_variant_size.max(variant_size);
            max_variant_alignment = max_variant_alignment.max(variant_alignment);
        }

        // Enum layout: discriminant + padding + largest variant
        let data_offset = self.align_to(discriminant_size, max_variant_alignment);
        let total_size = self.align_to(data_offset + max_variant_size, max_variant_alignment);

        MemoryLayout {
            size: total_size,
            alignment: max_variant_alignment,
            field_offsets: vec![0, data_offset], // [discriminant_offset, data_offset]
        }
    }

    /// Optimize field order for better memory layout
    pub fn optimize_field_order(&self, fields: &[StructField]) -> Vec<usize> {
        if fields.is_empty() {
            return vec![];
        }

        // Create indices with alignment information
        let mut field_info: Vec<(usize, usize, usize)> = fields
            .iter()
            .enumerate()
            .map(|(i, field)| {
                let size = self.get_type_size(&field.field_type);
                let alignment = self.get_type_alignment(&field.field_type);
                (i, size, alignment)
            })
            .collect();

        // Sort by alignment (descending) then by size (descending)
        // This minimizes padding by placing larger, more aligned fields first
        field_info.sort_by(|a, b| {
            b.2.cmp(&a.2).then_with(|| b.1.cmp(&a.1))
        });

        // Return the optimized order as indices
        field_info.into_iter().map(|(i, _, _)| i).collect()
    }

    /// Calculate memory usage analysis for a type
    pub fn analyze_memory_usage(&self, type_name: &str, layout: &MemoryLayout, fields: &[StructField]) -> MemoryUsageReport {
        let mut padding_bytes = 0;
        let mut field_bytes = 0;

        if !fields.is_empty() {
            let mut current_offset = 0;
            
            for (i, field) in fields.iter().enumerate() {
                let field_size = self.get_type_size(&field.field_type);
                let field_alignment = self.get_type_alignment(&field.field_type);
                
                // Calculate padding before this field
                let aligned_offset = self.align_to(current_offset, field_alignment);
                padding_bytes += aligned_offset - current_offset;
                
                field_bytes += field_size;
                current_offset = aligned_offset + field_size;
            }
            
            // Calculate trailing padding
            let final_size = self.align_to(current_offset, layout.alignment);
            padding_bytes += final_size - current_offset;
        }

        let efficiency = if layout.size > 0 {
            (field_bytes as f64 / layout.size as f64) * 100.0
        } else {
            100.0
        };

        MemoryUsageReport {
            type_name: type_name.to_string(),
            total_size: layout.size,
            field_bytes,
            padding_bytes,
            alignment: layout.alignment,
            efficiency_percent: efficiency,
            suggestions: self.generate_optimization_suggestions(fields, layout),
        }
    }

    /// Get the size of a type in bytes
    fn get_type_size(&self, ast_type: &Type) -> usize {
        match ast_type {
            Type::Named(name) => match name.as_str() {
                "bool" => 1,
                "int" => 4,  // i32
                "float" => 4, // f32
                "String" => 24, // String has ptr, capacity, len (3 * 8 bytes)
                "i8" => 1,
                "i16" => 2,
                "i32" => 4,
                "i64" => 8,
                "u8" => 1,
                "u16" => 2,
                "u32" => 4,
                "u64" => 8,
                "f32" => 4,
                "f64" => 8,
                "char" => 4, // UTF-32
                "usize" | "isize" => 8, // 64-bit platform
                _ => 8, // Default for user-defined types (pointer size)
            },
            Type::Array { element_type, size } => {
                let element_size = self.get_type_size(element_type);
                element_size * size.unwrap_or(0)
            },
            Type::Vec { .. } => 24, // Vec has ptr, capacity, len (3 * 8 bytes)
            Type::HashMap { .. } => 48, // HashMap internal structure
            Type::Reference { .. } => 8, // Pointer size
            Type::Slice { .. } => 16, // Fat pointer (ptr + len)
            Type::Generic { .. } => 8, // Default to pointer size for generics
        }
    }

    /// Get the alignment requirement of a type in bytes
    fn get_type_alignment(&self, ast_type: &Type) -> usize {
        match ast_type {
            Type::Named(name) => match name.as_str() {
                "bool" => 1,
                "int" => 4,
                "float" => 4,
                "String" => 8, // String has pointer alignment
                "i8" | "u8" => 1,
                "i16" | "u16" => 2,
                "i32" | "u32" | "f32" => 4,
                "i64" | "u64" | "f64" => 8,
                "char" => 4,
                "usize" | "isize" => 8,
                _ => 8, // Default alignment for user-defined types
            },
            Type::Array { element_type, .. } => self.get_type_alignment(element_type),
            Type::Vec { .. } => 8, // Pointer alignment
            Type::HashMap { .. } => 8, // Pointer alignment
            Type::Reference { .. } => 8, // Pointer alignment
            Type::Slice { .. } => 8, // Pointer alignment
            Type::Generic { .. } => 8, // Default alignment for generics
        }
    }

    /// Calculate layout for tuple-like data
    fn calculate_tuple_layout(&self, types: &[Type]) -> (usize, usize) {
        if types.is_empty() {
            return (0, 1);
        }

        let mut current_offset = 0;
        let mut max_alignment = 1;

        for ast_type in types {
            let field_size = self.get_type_size(ast_type);
            let field_alignment = self.get_type_alignment(ast_type);
            
            max_alignment = max_alignment.max(field_alignment);
            current_offset = self.align_to(current_offset, field_alignment);
            current_offset += field_size;
        }

        let total_size = self.align_to(current_offset, max_alignment);
        (total_size, max_alignment)
    }

    /// Get the size needed for enum discriminant
    fn get_discriminant_size(&self, variant_count: usize) -> usize {
        if variant_count <= 256 {
            1 // u8
        } else if variant_count <= 65536 {
            2 // u16
        } else {
            4 // u32
        }
    }

    /// Align a value to the specified alignment
    fn align_to(&self, value: usize, alignment: usize) -> usize {
        (value + alignment - 1) & !(alignment - 1)
    }

    /// Generate optimization suggestions for memory layout
    fn generate_optimization_suggestions(&self, fields: &[StructField], layout: &MemoryLayout) -> Vec<String> {
        let mut suggestions = Vec::new();

        if fields.is_empty() {
            return suggestions;
        }

        // Check if field reordering would help
        let optimized_order = self.optimize_field_order(fields);
        let is_already_optimized = optimized_order.iter().enumerate().all(|(i, &idx)| i == idx);

        if !is_already_optimized {
            suggestions.push("Consider reordering fields by alignment and size to reduce padding".to_string());
        }

        // Check for excessive padding
        let field_bytes: usize = fields.iter()
            .map(|f| self.get_type_size(&f.field_type))
            .sum();
        let padding_bytes = layout.size - field_bytes;
        
        if padding_bytes > field_bytes / 2 {
            suggestions.push("High padding overhead detected. Consider using #[repr(packed)] if appropriate".to_string());
        }

        // Check for very large structs
        if layout.size > 1024 {
            suggestions.push("Large struct detected. Consider using Box<T> for heap allocation".to_string());
        }

        // Check for alignment issues
        if layout.alignment > 8 {
            suggestions.push("High alignment requirement. Verify if all fields need this alignment".to_string());
        }

        suggestions
    }
}

impl Default for MemoryLayoutCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory usage analysis report
#[derive(Debug, Clone)]
pub struct MemoryUsageReport {
    pub type_name: String,
    pub total_size: usize,
    pub field_bytes: usize,
    pub padding_bytes: usize,
    pub alignment: usize,
    pub efficiency_percent: f64,
    pub suggestions: Vec<String>,
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

/// Enum definition with variants and discriminant information
#[derive(Debug, Clone)]
pub struct EnumDefinition {
    pub name: String,
    pub generics: Vec<String>,
    pub variants: Vec<EnumVariant>,
    pub discriminant_type: Ty,
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
    enums: HashMap<String, EnumDefinition>,
    impls: HashMap<String, Vec<ImplBlock>>,
    layout_calculator: MemoryLayoutCalculator,
}

impl TypeDefinitionManager {
    /// Create a new Type Definition Manager
    pub fn new() -> Self {
        Self {
            structs: HashMap::new(),
            enums: HashMap::new(),
            impls: HashMap::new(),
            layout_calculator: MemoryLayoutCalculator::new(),
        }
    }

    /// Create a struct definition with calculated memory layout
    pub fn create_struct_definition(&self, name: String, generics: Vec<String>, fields: Vec<StructField>, is_tuple: bool) -> StructDefinition {
        let layout = self.layout_calculator.calculate_struct_layout(&fields);
        StructDefinition {
            name,
            generics,
            fields,
            is_tuple,
            layout,
        }
    }

    /// Create an enum definition with calculated memory layout
    pub fn create_enum_definition(&self, name: String, generics: Vec<String>, variants: Vec<EnumVariant>) -> EnumDefinition {
        let layout = self.layout_calculator.calculate_enum_layout(&variants);
        let discriminant_type = match self.layout_calculator.get_discriminant_size(variants.len()) {
            1 => Ty::from_string("u8").unwrap_or(Ty::Int),
            2 => Ty::from_string("u16").unwrap_or(Ty::Int),
            4 => Ty::from_string("u32").unwrap_or(Ty::Int),
            _ => Ty::Int,
        };
        
        EnumDefinition {
            name,
            generics,
            variants,
            discriminant_type,
        }
    }

    /// Get memory usage analysis for a struct
    pub fn analyze_struct_memory(&self, struct_name: &str) -> Result<MemoryUsageReport, String> {
        let struct_def = self.get_struct(struct_name)
            .ok_or_else(|| format!("Struct '{}' not found", struct_name))?;
        
        Ok(self.layout_calculator.analyze_memory_usage(
            struct_name,
            &struct_def.layout,
            &struct_def.fields
        ))
    }

    /// Get optimized field order for a struct
    pub fn get_optimized_field_order(&self, struct_name: &str) -> Result<Vec<usize>, String> {
        let struct_def = self.get_struct(struct_name)
            .ok_or_else(|| format!("Struct '{}' not found", struct_name))?;
        
        Ok(self.layout_calculator.optimize_field_order(&struct_def.fields))
    }

    /// Get memory layout for a struct
    pub fn get_struct_layout(&self, struct_name: &str) -> Result<&MemoryLayout, String> {
        let struct_def = self.get_struct(struct_name)
            .ok_or_else(|| format!("Struct '{}' not found", struct_name))?;
        
        Ok(&struct_def.layout)
    }

    /// Get memory layout for an enum
    pub fn get_enum_layout(&self, enum_name: &str) -> Result<MemoryLayout, String> {
        let enum_def = self.get_enum(enum_name)
            .ok_or_else(|| format!("Enum '{}' not found", enum_name))?;
        
        Ok(self.layout_calculator.calculate_enum_layout(&enum_def.variants))
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

    /// Define a new enum type
    pub fn define_enum(&mut self, def: EnumDefinition) -> Result<(), String> {
        let name = def.name.clone();
        
        // Check if enum already exists
        if self.enums.contains_key(&name) {
            return Err(format!("Enum '{}' is already defined", name));
        }
        
        // Check if a struct with the same name exists
        if self.structs.contains_key(&name) {
            return Err(format!("Type '{}' is already defined as a struct", name));
        }
        
        // Validate enum definition
        self.validate_enum_definition(&def)?;
        
        // Store the enum definition
        self.enums.insert(name, def);
        Ok(())
    }

    /// Get an enum definition by name
    pub fn get_enum(&self, name: &str) -> Option<&EnumDefinition> {
        self.enums.get(name)
    }

    /// Validate enum variant construction
    pub fn validate_enum_variant_construction(&self, enum_name: &str, variant_name: &str, provided_data: Option<&[Ty]>) -> Result<(), String> {
        let enum_def = self.get_enum(enum_name)
            .ok_or_else(|| format!("Undefined enum type: {}", enum_name))?;

        // Find the variant
        for variant in &enum_def.variants {
            if variant.name == variant_name {
                return self.validate_variant_data(&variant, provided_data, enum_name, variant_name);
            }
        }

        Err(format!("Variant '{}' not found in enum '{}'", variant_name, enum_name))
    }

    /// Get the discriminant value for an enum variant
    pub fn get_variant_discriminant(&self, enum_name: &str, variant_name: &str) -> Result<usize, String> {
        let enum_def = self.get_enum(enum_name)
            .ok_or_else(|| format!("Undefined enum type: {}", enum_name))?;

        // Find the variant and return its index as discriminant
        for (index, variant) in enum_def.variants.iter().enumerate() {
            if variant.name == variant_name {
                return Ok(index);
            }
        }

        Err(format!("Variant '{}' not found in enum '{}'", variant_name, enum_name))
    }

    /// Get all variants for an enum
    pub fn get_enum_variants(&self, enum_name: &str) -> Result<&[EnumVariant], String> {
        let enum_def = self.get_enum(enum_name)
            .ok_or_else(|| format!("Undefined enum type: {}", enum_name))?;

        Ok(&enum_def.variants)
    }

    /// Check if an enum variant has data
    pub fn variant_has_data(&self, enum_name: &str, variant_name: &str) -> Result<bool, String> {
        let enum_def = self.get_enum(enum_name)
            .ok_or_else(|| format!("Undefined enum type: {}", enum_name))?;

        for variant in &enum_def.variants {
            if variant.name == variant_name {
                return Ok(variant.data.is_some());
            }
        }

        Err(format!("Variant '{}' not found in enum '{}'", variant_name, enum_name))
    }

    /// Get the type of data for an enum variant
    pub fn get_variant_data_types(&self, enum_name: &str, variant_name: &str) -> Result<Option<Vec<Ty>>, String> {
        let enum_def = self.get_enum(enum_name)
            .ok_or_else(|| format!("Undefined enum type: {}", enum_name))?;

        for variant in &enum_def.variants {
            if variant.name == variant_name {
                match &variant.data {
                    None => return Ok(None),
                    Some(EnumVariantData::Tuple(types)) => {
                        let mut result_types = Vec::new();
                        for ast_type in types {
                            result_types.push(self.ast_type_to_ty(ast_type)?);
                        }
                        return Ok(Some(result_types));
                    }
                    Some(EnumVariantData::Struct(fields)) => {
                        let mut result_types = Vec::new();
                        for field in fields {
                            result_types.push(self.ast_type_to_ty(&field.field_type)?);
                        }
                        return Ok(Some(result_types));
                    }
                }
            }
        }

        Err(format!("Variant '{}' not found in enum '{}'", variant_name, enum_name))
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
        if !self.structs.contains_key(&type_name) && !self.enums.contains_key(&type_name) {
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

    /// Validate an enum definition
    fn validate_enum_definition(&self, def: &EnumDefinition) -> Result<(), String> {
        // Check for duplicate variant names
        let mut variant_names = std::collections::HashSet::new();
        for variant in &def.variants {
            if !variant_names.insert(&variant.name) {
                return Err(format!("Duplicate variant '{}' in enum '{}'", variant.name, def.name));
            }
        }

        // Validate variant data types
        for variant in &def.variants {
            if let Some(data) = &variant.data {
                match data {
                    EnumVariantData::Tuple(types) => {
                        for ast_type in types {
                            self.validate_ast_type(ast_type)?;
                        }
                    }
                    EnumVariantData::Struct(fields) => {
                        // Check for duplicate field names within the variant
                        let mut field_names = std::collections::HashSet::new();
                        for field in fields {
                            if !field_names.insert(&field.name) {
                                return Err(format!("Duplicate field '{}' in variant '{}' of enum '{}'", 
                                    field.name, variant.name, def.name));
                            }
                            self.validate_ast_type(&field.field_type)?;
                        }
                    }
                }
            }
        }

        // Ensure enum has at least one variant
        if def.variants.is_empty() {
            return Err(format!("Enum '{}' must have at least one variant", def.name));
        }

        Ok(())
    }

    /// Validate variant data against provided data
    fn validate_variant_data(&self, variant: &EnumVariant, provided_data: Option<&[Ty]>, enum_name: &str, variant_name: &str) -> Result<(), String> {
        match (&variant.data, provided_data) {
            (None, None) => Ok(()), // Unit variant with no data
            (None, Some(_)) => Err(format!("Variant '{}' of enum '{}' expects no data, but data was provided", variant_name, enum_name)),
            (Some(_), None) => Err(format!("Variant '{}' of enum '{}' expects data, but none was provided", variant_name, enum_name)),
            (Some(expected_data), Some(provided)) => {
                match expected_data {
                    EnumVariantData::Tuple(expected_types) => {
                        if expected_types.len() != provided.len() {
                            return Err(format!("Variant '{}' of enum '{}' expects {} data items, but {} were provided", 
                                variant_name, enum_name, expected_types.len(), provided.len()));
                        }
                        
                        for (i, (expected_ast_type, provided_type)) in expected_types.iter().zip(provided.iter()).enumerate() {
                            let expected_type = self.ast_type_to_ty(expected_ast_type)?;
                            if *provided_type != expected_type {
                                return Err(format!("Type mismatch for data item {} in variant '{}' of enum '{}': expected {}, got {}", 
                                    i, variant_name, enum_name, expected_type.to_string(), provided_type.to_string()));
                            }
                        }
                    }
                    EnumVariantData::Struct(expected_fields) => {
                        if expected_fields.len() != provided.len() {
                            return Err(format!("Variant '{}' of enum '{}' expects {} fields, but {} were provided", 
                                variant_name, enum_name, expected_fields.len(), provided.len()));
                        }
                        
                        for (i, (expected_field, provided_type)) in expected_fields.iter().zip(provided.iter()).enumerate() {
                            let expected_type = self.ast_type_to_ty(&expected_field.field_type)?;
                            if *provided_type != expected_type {
                                return Err(format!("Type mismatch for field '{}' in variant '{}' of enum '{}': expected {}, got {}", 
                                    expected_field.name, variant_name, enum_name, expected_type.to_string(), provided_type.to_string()));
                            }
                        }
                    }
                }
                Ok(())
            }
        }
    }

    /// Validate an AST type
    fn validate_ast_type(&self, ast_type: &Type) -> Result<(), String> {
        match ast_type {
            Type::Named(name) => {
                // Check if it's a primitive type or defined struct/enum
                if Ty::from_string(name).is_none() && 
                   !self.structs.contains_key(name) && 
                   !self.enums.contains_key(name) {
                    return Err(format!("Undefined type: {}", name));
                }
            }
            Type::Generic { name: _, type_args } => {
                // Validate generic type arguments
                for arg in type_args {
                    self.validate_ast_type(arg)?;
                }
            }
            Type::Array { element_type, size: _ } => {
                self.validate_ast_type(element_type)?;
            }
            Type::Slice { element_type } => {
                self.validate_ast_type(element_type)?;
            }
            Type::Vec { element_type } => {
                self.validate_ast_type(element_type)?;
            }
            Type::HashMap { key_type, value_type } => {
                self.validate_ast_type(key_type)?;
                self.validate_ast_type(value_type)?;
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
                } else if self.enums.contains_key(name) {
                    Ok(Ty::Enum(name.clone()))
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
            _ => Err(format!("Unsupported type conversion: {:?}", ast_type))
        }
    }
}

impl Default for TypeDefinitionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Type, StructField, Visibility, Function, Parameter, Block};

    fn create_test_memory_layout() -> MemoryLayout {
        MemoryLayout {
            size: 16,
            alignment: 8,
            field_offsets: vec![0, 8],
        }
    }

    #[test]
    fn test_type_definition_manager_new() {
        let manager = TypeDefinitionManager::new();
        assert!(manager.get_struct("Point").is_none());
    }

    #[test]
    fn test_define_struct_success() {
        let mut manager = TypeDefinitionManager::new();
        
        let struct_def = StructDefinition {
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

        let result = manager.define_struct(struct_def);
        assert!(result.is_ok());
        assert!(manager.get_struct("Point").is_some());
    }

    #[test]
    fn test_define_struct_duplicate_error() {
        let mut manager = TypeDefinitionManager::new();
        
        let struct_def = StructDefinition {
            name: "Point".to_string(),
            generics: vec![],
            fields: vec![
                StructField {
                    name: "x".to_string(),
                    field_type: Type::Named("int".to_string()),
                    visibility: Visibility::Public,
                },
            ],
            is_tuple: false,
            layout: create_test_memory_layout(),
        };

        // First definition should succeed
        assert!(manager.define_struct(struct_def.clone()).is_ok());
        
        // Second definition should fail
        let result = manager.define_struct(struct_def);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already defined"));
    }

    #[test]
    fn test_validate_field_access_success() {
        let mut manager = TypeDefinitionManager::new();
        
        let struct_def = StructDefinition {
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
                    field_type: Type::Named("float".to_string()),
                    visibility: Visibility::Public,
                },
            ],
            is_tuple: false,
            layout: create_test_memory_layout(),
        };

        manager.define_struct(struct_def).unwrap();

        let result = manager.validate_field_access("Point", "x");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Ty::Int);

        let result = manager.validate_field_access("Point", "y");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Ty::Float);
    }

    #[test]
    fn test_validate_struct_instantiation_success() {
        let mut manager = TypeDefinitionManager::new();
        
        let struct_def = StructDefinition {
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

        manager.define_struct(struct_def).unwrap();

        let provided_fields = vec![
            ("x".to_string(), Ty::Int),
            ("y".to_string(), Ty::Int),
        ];

        let result = manager.validate_struct_instantiation("Point", &provided_fields);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_impl_and_get_method() {
        let mut manager = TypeDefinitionManager::new();
        
        // First define a struct
        let struct_def = StructDefinition {
            name: "Point".to_string(),
            generics: vec![],
            fields: vec![
                StructField {
                    name: "x".to_string(),
                    field_type: Type::Named("int".to_string()),
                    visibility: Visibility::Public,
                },
            ],
            is_tuple: false,
            layout: create_test_memory_layout(),
        };

        manager.define_struct(struct_def).unwrap();

        // Now add an implementation
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
                    ],
                    return_type: Some(Type::Named("Point".to_string())),
                    body: Block {
                        statements: vec![],
                        expression: None,
                    },
                },
            ],
        };

        let result = manager.add_impl(impl_block);
        assert!(result.is_ok());

        let method = manager.get_method("Point", "new");
        assert!(method.is_some());
        assert_eq!(method.unwrap().name, "new");
    }

    #[test]
    fn test_ty_to_string() {
        assert_eq!(Ty::Int.to_string(), "int");
        assert_eq!(Ty::Float.to_string(), "float");
        assert_eq!(Ty::Bool.to_string(), "bool");
        assert_eq!(Ty::Struct("Point".to_string()).to_string(), "Point");
        assert_eq!(Ty::Enum("Color".to_string()).to_string(), "Color");
        assert_eq!(Ty::Array(Box::new(Ty::Int), Some(5)).to_string(), "[int; 5]");
        assert_eq!(Ty::Vec(Box::new(Ty::Int)).to_string(), "Vec<int>");
        assert_eq!(Ty::Reference(Box::new(Ty::Int)).to_string(), "&int");
    }

    // Memory Layout Calculator Tests
    #[test]
    fn test_memory_layout_calculator_new() {
        let calculator = MemoryLayoutCalculator::new();
        // Just verify it can be created
        assert_eq!(calculator.align_to(5, 4), 8);
    }

    #[test]
    fn test_calculate_struct_layout_empty() {
        let calculator = MemoryLayoutCalculator::new();
        let layout = calculator.calculate_struct_layout(&[]);
        
        assert_eq!(layout.size, 0);
        assert_eq!(layout.alignment, 1);
        assert_eq!(layout.field_offsets, vec![]);
    }

    #[test]
    fn test_calculate_struct_layout_simple() {
        let calculator = MemoryLayoutCalculator::new();
        let fields = vec![
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
        ];
        
        let layout = calculator.calculate_struct_layout(&fields);
        
        assert_eq!(layout.size, 8); // 4 + 4 = 8 bytes
        assert_eq!(layout.alignment, 4); // int alignment
        assert_eq!(layout.field_offsets, vec![0, 4]);
    }

    #[test]
    fn test_calculate_struct_layout_with_padding() {
        let calculator = MemoryLayoutCalculator::new();
        let fields = vec![
            StructField {
                name: "a".to_string(),
                field_type: Type::Named("bool".to_string()),
                visibility: Visibility::Public,
            },
            StructField {
                name: "b".to_string(),
                field_type: Type::Named("int".to_string()),
                visibility: Visibility::Public,
            },
        ];
        
        let layout = calculator.calculate_struct_layout(&fields);
        
        assert_eq!(layout.size, 8); // 1 + 3 padding + 4 = 8 bytes
        assert_eq!(layout.alignment, 4); // int alignment
        assert_eq!(layout.field_offsets, vec![0, 4]); // bool at 0, int at 4 (aligned)
    }

    #[test]
    fn test_calculate_enum_layout_empty() {
        let calculator = MemoryLayoutCalculator::new();
        let layout = calculator.calculate_enum_layout(&[]);
        
        assert_eq!(layout.size, 1); // At least 1 byte for discriminant
        assert_eq!(layout.alignment, 1);
    }

    #[test]
    fn test_calculate_enum_layout_unit_variants() {
        let calculator = MemoryLayoutCalculator::new();
        let variants = vec![
            EnumVariant {
                name: "Red".to_string(),
                data: None,
            },
            EnumVariant {
                name: "Green".to_string(),
                data: None,
            },
            EnumVariant {
                name: "Blue".to_string(),
                data: None,
            },
        ];
        
        let layout = calculator.calculate_enum_layout(&variants);
        
        assert_eq!(layout.size, 1); // Just discriminant, no data
        assert_eq!(layout.alignment, 1);
        assert_eq!(layout.field_offsets, vec![0, 1]); // [discriminant_offset, data_offset]
    }

    #[test]
    fn test_calculate_enum_layout_with_data() {
        let calculator = MemoryLayoutCalculator::new();
        let variants = vec![
            EnumVariant {
                name: "None".to_string(),
                data: None,
            },
            EnumVariant {
                name: "Some".to_string(),
                data: Some(EnumVariantData::Tuple(vec![Type::Named("int".to_string())])),
            },
        ];
        
        let layout = calculator.calculate_enum_layout(&variants);
        
        assert_eq!(layout.size, 8); // 1 byte discriminant + 3 padding + 4 bytes data
        assert_eq!(layout.alignment, 4); // int alignment
        assert_eq!(layout.field_offsets, vec![0, 4]); // [discriminant_offset, data_offset]
    }

    #[test]
    fn test_optimize_field_order_empty() {
        let calculator = MemoryLayoutCalculator::new();
        let order = calculator.optimize_field_order(&[]);
        assert_eq!(order, vec![]);
    }

    #[test]
    fn test_optimize_field_order_already_optimal() {
        let calculator = MemoryLayoutCalculator::new();
        let fields = vec![
            StructField {
                name: "a".to_string(),
                field_type: Type::Named("int".to_string()), // 4 bytes, 4 alignment
                visibility: Visibility::Public,
            },
            StructField {
                name: "b".to_string(),
                field_type: Type::Named("bool".to_string()), // 1 byte, 1 alignment
                visibility: Visibility::Public,
            },
        ];
        
        let order = calculator.optimize_field_order(&fields);
        assert_eq!(order, vec![0, 1]); // Already optimal (larger alignment first)
    }

    #[test]
    fn test_optimize_field_order_needs_reordering() {
        let calculator = MemoryLayoutCalculator::new();
        let fields = vec![
            StructField {
                name: "a".to_string(),
                field_type: Type::Named("bool".to_string()), // 1 byte, 1 alignment
                visibility: Visibility::Public,
            },
            StructField {
                name: "b".to_string(),
                field_type: Type::Named("int".to_string()), // 4 bytes, 4 alignment
                visibility: Visibility::Public,
            },
        ];
        
        let order = calculator.optimize_field_order(&fields);
        assert_eq!(order, vec![1, 0]); // Should reorder: int first, then bool
    }

    #[test]
    fn test_analyze_memory_usage() {
        let calculator = MemoryLayoutCalculator::new();
        let fields = vec![
            StructField {
                name: "a".to_string(),
                field_type: Type::Named("bool".to_string()),
                visibility: Visibility::Public,
            },
            StructField {
                name: "b".to_string(),
                field_type: Type::Named("int".to_string()),
                visibility: Visibility::Public,
            },
        ];
        
        let layout = calculator.calculate_struct_layout(&fields);
        let report = calculator.analyze_memory_usage("TestStruct", &layout, &fields);
        
        assert_eq!(report.type_name, "TestStruct");
        assert_eq!(report.total_size, 8);
        assert_eq!(report.field_bytes, 5); // 1 + 4
        assert_eq!(report.padding_bytes, 3); // 3 bytes padding
        assert_eq!(report.alignment, 4);
        assert_eq!(report.efficiency_percent, 62.5); // 5/8 * 100
        assert!(!report.suggestions.is_empty());
    }

    #[test]
    fn test_get_type_size() {
        let calculator = MemoryLayoutCalculator::new();
        
        assert_eq!(calculator.get_type_size(&Type::Named("bool".to_string())), 1);
        assert_eq!(calculator.get_type_size(&Type::Named("int".to_string())), 4);
        assert_eq!(calculator.get_type_size(&Type::Named("float".to_string())), 4);
        assert_eq!(calculator.get_type_size(&Type::Named("i64".to_string())), 8);
        assert_eq!(calculator.get_type_size(&Type::Named("f64".to_string())), 8);
        assert_eq!(calculator.get_type_size(&Type::Array { 
            element_type: Box::new(Type::Named("int".to_string())), 
            size: Some(5) 
        }), 20); // 4 * 5
        assert_eq!(calculator.get_type_size(&Type::Vec { 
            element_type: Box::new(Type::Named("int".to_string())) 
        }), 24); // Vec metadata
        assert_eq!(calculator.get_type_size(&Type::Reference { 
            mutable: false, 
            inner_type: Box::new(Type::Named("int".to_string())) 
        }), 8); // Pointer size
    }

    #[test]
    fn test_get_type_alignment() {
        let calculator = MemoryLayoutCalculator::new();
        
        assert_eq!(calculator.get_type_alignment(&Type::Named("bool".to_string())), 1);
        assert_eq!(calculator.get_type_alignment(&Type::Named("int".to_string())), 4);
        assert_eq!(calculator.get_type_alignment(&Type::Named("float".to_string())), 4);
        assert_eq!(calculator.get_type_alignment(&Type::Named("i64".to_string())), 8);
        assert_eq!(calculator.get_type_alignment(&Type::Named("f64".to_string())), 8);
        assert_eq!(calculator.get_type_alignment(&Type::Array { 
            element_type: Box::new(Type::Named("int".to_string())), 
            size: Some(5) 
        }), 4); // Element alignment
        assert_eq!(calculator.get_type_alignment(&Type::Vec { 
            element_type: Box::new(Type::Named("int".to_string())) 
        }), 8); // Pointer alignment
    }

    #[test]
    fn test_get_discriminant_size() {
        let calculator = MemoryLayoutCalculator::new();
        
        assert_eq!(calculator.get_discriminant_size(2), 1); // u8 for <= 256 variants
        assert_eq!(calculator.get_discriminant_size(256), 1); // u8 for <= 256 variants
        assert_eq!(calculator.get_discriminant_size(257), 2); // u16 for <= 65536 variants
        assert_eq!(calculator.get_discriminant_size(65536), 2); // u16 for <= 65536 variants
        assert_eq!(calculator.get_discriminant_size(65537), 4); // u32 for > 65536 variants
    }

    #[test]
    fn test_align_to() {
        let calculator = MemoryLayoutCalculator::new();
        
        assert_eq!(calculator.align_to(0, 4), 0);
        assert_eq!(calculator.align_to(1, 4), 4);
        assert_eq!(calculator.align_to(4, 4), 4);
        assert_eq!(calculator.align_to(5, 4), 8);
        assert_eq!(calculator.align_to(7, 8), 8);
        assert_eq!(calculator.align_to(9, 8), 16);
    }

    #[test]
    fn test_type_definition_manager_with_layout_calculator() {
        let mut manager = TypeDefinitionManager::new();
        
        // Test create_struct_definition
        let struct_def = manager.create_struct_definition(
            "Point".to_string(),
            vec![],
            vec![
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
            false,
        );
        
        assert_eq!(struct_def.name, "Point");
        assert_eq!(struct_def.layout.size, 8);
        assert_eq!(struct_def.layout.alignment, 4);
        assert_eq!(struct_def.layout.field_offsets, vec![0, 4]);
        
        // Define the struct and test memory analysis
        manager.define_struct(struct_def).unwrap();
        
        let report = manager.analyze_struct_memory("Point").unwrap();
        assert_eq!(report.type_name, "Point");
        assert_eq!(report.total_size, 8);
        assert_eq!(report.field_bytes, 8);
        assert_eq!(report.padding_bytes, 0);
        assert_eq!(report.efficiency_percent, 100.0);
    }

    #[test]
    fn test_type_definition_manager_enum_layout() {
        let mut manager = TypeDefinitionManager::new();
        
        // Test create_enum_definition
        let enum_def = manager.create_enum_definition(
            "Option".to_string(),
            vec!["T".to_string()],
            vec![
                EnumVariant {
                    name: "None".to_string(),
                    data: None,
                },
                EnumVariant {
                    name: "Some".to_string(),
                    data: Some(EnumVariantData::Tuple(vec![Type::Named("int".to_string())])),
                },
            ],
        );
        
        assert_eq!(enum_def.name, "Option");
        assert_eq!(enum_def.discriminant_type, Ty::from_string("u8").unwrap_or(Ty::Int));
        
        // Define the enum and test layout retrieval
        manager.define_enum(enum_def).unwrap();
        
        let layout = manager.get_enum_layout("Option").unwrap();
        assert_eq!(layout.size, 8); // 1 byte discriminant + padding + 4 bytes data
        assert_eq!(layout.alignment, 4);
    }

    #[test]
    fn test_get_optimized_field_order() {
        let mut manager = TypeDefinitionManager::new();
        
        let struct_def = manager.create_struct_definition(
            "BadLayout".to_string(),
            vec![],
            vec![
                StructField {
                    name: "a".to_string(),
                    field_type: Type::Named("bool".to_string()),
                    visibility: Visibility::Public,
                },
                StructField {
                    name: "b".to_string(),
                    field_type: Type::Named("i64".to_string()),
                    visibility: Visibility::Public,
                },
                StructField {
                    name: "c".to_string(),
                    field_type: Type::Named("bool".to_string()),
                    visibility: Visibility::Public,
                },
            ],
            false,
        );
        
        manager.define_struct(struct_def).unwrap();
        
        let optimized_order = manager.get_optimized_field_order("BadLayout").unwrap();
        // Should reorder to put i64 (8-byte alignment) first, then bools
        assert_eq!(optimized_order, vec![1, 0, 2]); // b, a, c
    }
}

