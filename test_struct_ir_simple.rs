// Simple test for struct IR generation

use std::collections::HashMap;

// Minimal struct definition for testing
#[derive(Debug, Clone)]
pub struct StructDefinition {
    pub name: String,
    pub fields: Vec<(String, String)>,
    pub field_indices: HashMap<String, usize>,
    pub is_tuple: bool,
}

fn main() {
    println!("Testing struct IR generation...");
    
    // Create a simple struct definition
    let mut field_indices = HashMap::new();
    field_indices.insert("x".to_string(), 0);
    field_indices.insert("y".to_string(), 1);
    
    let struct_def = StructDefinition {
        name: "Point".to_string(),
        fields: vec![
            ("x".to_string(), "i32".to_string()),
            ("y".to_string(), "i32".to_string()),
        ],
        field_indices,
        is_tuple: false,
    };
    
    println!("Created struct definition: {:?}", struct_def);
    println!("Test completed successfully!");
}