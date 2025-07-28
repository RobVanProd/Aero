// Built-in Collections Library for Task 11 and Error Handling for Task 12
// This module implements Vec, array operations, string operations, Result, and Option types

use std::collections::HashMap;
use crate::ir::{Function, Inst, Value};

/// Built-in Vec<T> implementation
pub struct VecType {
    pub element_type: String,
    pub methods: HashMap<String, VecMethod>,
}

#[derive(Debug, Clone)]
pub enum VecMethod {
    New,
    Push,
    Pop,
    Len,
    Capacity,
    IsEmpty,
    Clear,
    Get,
    Insert,
    Remove,
    Contains,
    Iter,
}

impl VecType {
    pub fn new(element_type: String) -> Self {
        let mut methods = HashMap::new();
        
        // Add all Vec methods
        methods.insert("new".to_string(), VecMethod::New);
        methods.insert("push".to_string(), VecMethod::Push);
        methods.insert("pop".to_string(), VecMethod::Pop);
        methods.insert("len".to_string(), VecMethod::Len);
        methods.insert("capacity".to_string(), VecMethod::Capacity);
        methods.insert("is_empty".to_string(), VecMethod::IsEmpty);
        methods.insert("clear".to_string(), VecMethod::Clear);
        methods.insert("get".to_string(), VecMethod::Get);
        methods.insert("insert".to_string(), VecMethod::Insert);
        methods.insert("remove".to_string(), VecMethod::Remove);
        methods.insert("contains".to_string(), VecMethod::Contains);
        methods.insert("iter".to_string(), VecMethod::Iter);
        
        VecType {
            element_type,
            methods,
        }
    }
    
    /// Generate LLVM IR for Vec method call
    pub fn generate_method_call(&self, method: &str, args: &[Value]) -> Vec<Inst> {
        match self.methods.get(method) {
            Some(VecMethod::New) => self.generate_vec_new(),
            Some(VecMethod::Push) => self.generate_vec_push(args),
            Some(VecMethod::Pop) => self.generate_vec_pop(args),
            Some(VecMethod::Len) => self.generate_vec_len(args),
            Some(VecMethod::Capacity) => self.generate_vec_capacity(args),
            Some(VecMethod::IsEmpty) => self.generate_vec_is_empty(args),
            Some(VecMethod::Clear) => self.generate_vec_clear(args),
            Some(VecMethod::Get) => self.generate_vec_get(args),
            Some(VecMethod::Insert) => self.generate_vec_insert(args),
            Some(VecMethod::Remove) => self.generate_vec_remove(args),
            Some(VecMethod::Contains) => self.generate_vec_contains(args),
            Some(VecMethod::Iter) => self.generate_vec_iter(args),
            None => panic!("Unknown Vec method: {}", method),
        }
    }
    
    fn generate_vec_new(&self) -> Vec<Inst> {
        vec![
            Inst::VecAlloca {
                result: Value::Reg(1),
                element_type: self.element_type.clone(),
            }
        ]
    }
    
    fn generate_vec_push(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Vec::push requires 2 arguments (self, value)");
        }
        vec![
            Inst::VecPush {
                vec_ptr: args[0].clone(),
                value: args[1].clone(),
            }
        ]
    }
    
    fn generate_vec_pop(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 1 {
            panic!("Vec::pop requires 1 argument (self)");
        }
        vec![
            Inst::VecPop {
                result: Value::Reg(2),
                vec_ptr: args[0].clone(),
            }
        ]
    }
    
    fn generate_vec_len(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 1 {
            panic!("Vec::len requires 1 argument (self)");
        }
        vec![
            Inst::VecLength {
                result: Value::Reg(3),
                vec_ptr: args[0].clone(),
            }
        ]
    }
    
    fn generate_vec_capacity(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 1 {
            panic!("Vec::capacity requires 1 argument (self)");
        }
        vec![
            Inst::VecCapacity {
                result: Value::Reg(4),
                vec_ptr: args[0].clone(),
            }
        ]
    }
    
    fn generate_vec_is_empty(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 1 {
            panic!("Vec::is_empty requires 1 argument (self)");
        }
        vec![
            Inst::VecLength {
                result: Value::Reg(5),
                vec_ptr: args[0].clone(),
            },
            Inst::FCmp {
                op: "oeq".to_string(),
                result: Value::Reg(6),
                left: Value::Reg(5),
                right: Value::ImmFloat(0.0),
            }
        ]
    }
    
    fn generate_vec_clear(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 1 {
            panic!("Vec::clear requires 1 argument (self)");
        }
        // Set length to 0
        vec![
            Inst::Store(Value::Reg(100), Value::ImmFloat(0.0)), // Simplified - should access length field
        ]
    }
    
    fn generate_vec_get(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Vec::get requires 2 arguments (self, index)");
        }
        vec![
            Inst::VecAccess {
                result: Value::Reg(7),
                vec_ptr: args[0].clone(),
                index: args[1].clone(),
            }
        ]
    }
    
    fn generate_vec_insert(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 3 {
            panic!("Vec::insert requires 3 arguments (self, index, value)");
        }
        // Simplified implementation - should shift elements
        vec![
            Inst::VecPush {
                vec_ptr: args[0].clone(),
                value: args[2].clone(),
            }
        ]
    }
    
    fn generate_vec_remove(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Vec::remove requires 2 arguments (self, index)");
        }
        // Simplified implementation - should shift elements
        vec![
            Inst::VecPop {
                result: Value::Reg(8),
                vec_ptr: args[0].clone(),
            }
        ]
    }
    
    fn generate_vec_contains(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Vec::contains requires 2 arguments (self, value)");
        }
        // Simplified implementation - should iterate and compare
        vec![
            Inst::FCmp {
                op: "oeq".to_string(),
                result: Value::Reg(9),
                left: args[1].clone(),
                right: Value::ImmFloat(0.0),
            }
        ]
    }
    
    fn generate_vec_iter(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 1 {
            panic!("Vec::iter requires 1 argument (self)");
        }
        // Return iterator (simplified)
        vec![
            Inst::VecLength {
                result: Value::Reg(10),
                vec_ptr: args[0].clone(),
            }
        ]
    }
}

/// Built-in Array operations
pub struct ArrayOps;

impl ArrayOps {
    /// Generate array slice operation
    pub fn generate_slice(array: Value, start: Value, end: Value) -> Vec<Inst> {
        vec![
            // Convert indices to i64
            Inst::FPToSI(Value::Reg(11), start),
            Inst::FPToSI(Value::Reg(12), end),
            // Create slice (simplified - should create slice struct)
            Inst::ArrayAccess {
                result: Value::Reg(13),
                array_ptr: array,
                index: Value::Reg(11),
            }
        ]
    }
    
    /// Generate array iteration
    pub fn generate_iter(array: Value) -> Vec<Inst> {
        vec![
            Inst::ArrayLength {
                result: Value::Reg(14),
                array_ptr: array,
            },
            // Create iterator (simplified)
            Inst::Alloca(Value::Reg(15), "iterator".to_string()),
        ]
    }
    
    /// Generate array method calls
    pub fn generate_method_call(method: &str, args: &[Value]) -> Vec<Inst> {
        match method {
            "len" => vec![
                Inst::ArrayLength {
                    result: Value::Reg(16),
                    array_ptr: args[0].clone(),
                }
            ],
            "is_empty" => vec![
                Inst::ArrayLength {
                    result: Value::Reg(17),
                    array_ptr: args[0].clone(),
                },
                Inst::FCmp {
                    op: "oeq".to_string(),
                    result: Value::Reg(18),
                    left: Value::Reg(17),
                    right: Value::ImmFloat(0.0),
                }
            ],
            "first" => vec![
                Inst::ArrayAccess {
                    result: Value::Reg(19),
                    array_ptr: args[0].clone(),
                    index: Value::ImmInt(0),
                }
            ],
            "last" => vec![
                Inst::ArrayLength {
                    result: Value::Reg(20),
                    array_ptr: args[0].clone(),
                },
                Inst::FSub(Value::Reg(21), Value::Reg(20), Value::ImmFloat(1.0)),
                Inst::ArrayAccess {
                    result: Value::Reg(22),
                    array_ptr: args[0].clone(),
                    index: Value::Reg(21),
                }
            ],
            "contains" => vec![
                // Simplified - should iterate and compare
                Inst::FCmp {
                    op: "oeq".to_string(),
                    result: Value::Reg(23),
                    left: args[1].clone(),
                    right: Value::ImmFloat(0.0),
                }
            ],
            _ => panic!("Unknown array method: {}", method),
        }
    }
}

/// Built-in String operations
pub struct StringOps;

impl StringOps {
    /// Generate string concatenation
    pub fn generate_concat(left: Value, right: Value) -> Vec<Inst> {
        vec![
            // Simplified string concatenation - should allocate new string
            Inst::Alloca(Value::Reg(24), "concat_result".to_string()),
            // Copy left string (simplified)
            Inst::Store(Value::Reg(24), left),
            // Append right string (simplified)
            Inst::Store(Value::Reg(25), right),
        ]
    }
    
    /// Generate string length
    pub fn generate_len(string: Value) -> Vec<Inst> {
        vec![
            // String length (simplified - should access string metadata)
            Inst::Alloca(Value::Reg(26), "string_len".to_string()),
            Inst::Store(Value::Reg(26), Value::ImmFloat(10.0)), // Placeholder length
        ]
    }
    
    /// Generate string slicing
    pub fn generate_slice(string: Value, start: Value, end: Value) -> Vec<Inst> {
        vec![
            // String slicing with UTF-8 safety (simplified)
            Inst::FPToSI(Value::Reg(27), start),
            Inst::FPToSI(Value::Reg(28), end),
            Inst::Alloca(Value::Reg(29), "string_slice".to_string()),
        ]
    }
    
    /// Generate string comparison
    pub fn generate_eq(left: Value, right: Value) -> Vec<Inst> {
        vec![
            // String comparison (simplified - should compare byte by byte)
            Inst::FCmp {
                op: "oeq".to_string(),
                result: Value::Reg(30),
                left,
                right,
            }
        ]
    }
    
    /// Generate string method calls
    pub fn generate_method_call(method: &str, args: &[Value]) -> Vec<Inst> {
        match method {
            "len" => Self::generate_len(args[0].clone()),
            "is_empty" => vec![
                Inst::FCmp {
                    op: "oeq".to_string(),
                    result: Value::Reg(31),
                    left: args[0].clone(),
                    right: Value::ImmFloat(0.0),
                }
            ],
            "chars" => vec![
                // Return character iterator (simplified)
                Inst::Alloca(Value::Reg(32), "char_iter".to_string()),
            ],
            "contains" => vec![
                // String contains (simplified)
                Inst::FCmp {
                    op: "oeq".to_string(),
                    result: Value::Reg(33),
                    left: args[0].clone(),
                    right: args[1].clone(),
                }
            ],
            "starts_with" => vec![
                // String starts_with (simplified)
                Inst::FCmp {
                    op: "oeq".to_string(),
                    result: Value::Reg(34),
                    left: args[0].clone(),
                    right: args[1].clone(),
                }
            ],
            "ends_with" => vec![
                // String ends_with (simplified)
                Inst::FCmp {
                    op: "oeq".to_string(),
                    result: Value::Reg(35),
                    left: args[0].clone(),
                    right: args[1].clone(),
                }
            ],
            "to_uppercase" => vec![
                // String to_uppercase (simplified)
                Inst::Alloca(Value::Reg(36), "uppercase_string".to_string()),
            ],
            "to_lowercase" => vec![
                // String to_lowercase (simplified)
                Inst::Alloca(Value::Reg(37), "lowercase_string".to_string()),
            ],
            "trim" => vec![
                // String trim (simplified)
                Inst::Alloca(Value::Reg(38), "trimmed_string".to_string()),
            ],
            "split" => vec![
                // String split (simplified)
                Inst::Alloca(Value::Reg(39), "split_result".to_string()),
            ],
            "replace" => vec![
                // String replace (simplified)
                Inst::Alloca(Value::Reg(40), "replaced_string".to_string()),
            ],
            _ => panic!("Unknown string method: {}", method),
        }
    }
}

/// Collection library manager
pub struct CollectionLibrary {
    pub vec_types: HashMap<String, VecType>,
}

impl CollectionLibrary {
    pub fn new() -> Self {
        CollectionLibrary {
            vec_types: HashMap::new(),
        }
    }
    
    /// Register a new Vec type
    pub fn register_vec_type(&mut self, element_type: String) {
        let vec_type = VecType::new(element_type.clone());
        self.vec_types.insert(element_type, vec_type);
    }
    
    /// Get Vec type for element type
    pub fn get_vec_type(&self, element_type: &str) -> Option<&VecType> {
        self.vec_types.get(element_type)
    }
    
    /// Generate vec! macro
    pub fn generate_vec_macro(elements: Vec<Value>, element_type: String) -> Vec<Inst> {
        vec![
            Inst::VecInit {
                result: Value::Reg(41),
                element_type,
                elements,
            }
        ]
    }
    
    /// Generate for loop over collection
    pub fn generate_for_loop(collection: Value, loop_var: String, body: Vec<Inst>) -> Vec<Inst> {
        let mut instructions = vec![
            // Get collection length
            Inst::VecLength {
                result: Value::Reg(42),
                vec_ptr: collection.clone(),
            },
            // Initialize loop counter
            Inst::Alloca(Value::Reg(43), "loop_counter".to_string()),
            Inst::Store(Value::Reg(43), Value::ImmFloat(0.0)),
            // Loop header
            Inst::Label("loop_header".to_string()),
            // Check loop condition
            Inst::Load(Value::Reg(44), Value::Reg(43)),
            Inst::FCmp {
                op: "olt".to_string(),
                result: Value::Reg(45),
                left: Value::Reg(44),
                right: Value::Reg(42),
            },
            Inst::Branch {
                condition: Value::Reg(45),
                true_label: "loop_body".to_string(),
                false_label: "loop_exit".to_string(),
            },
            // Loop body
            Inst::Label("loop_body".to_string()),
            // Get current element
            Inst::VecAccess {
                result: Value::Reg(46),
                vec_ptr: collection,
                index: Value::Reg(44),
            },
            // Store in loop variable
            Inst::Alloca(Value::Reg(47), loop_var),
            Inst::Store(Value::Reg(47), Value::Reg(46)),
        ];
        
        // Add body instructions
        instructions.extend(body);
        
        // Add loop increment and jump
        instructions.extend(vec![
            // Increment counter
            Inst::FAdd(Value::Reg(48), Value::Reg(44), Value::ImmFloat(1.0)),
            Inst::Store(Value::Reg(43), Value::Reg(48)),
            // Jump back to header
            Inst::Jump("loop_header".to_string()),
            // Loop exit
            Inst::Label("loop_exit".to_string()),
        ]);
        
        instructions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vec_type_creation() {
        let vec_type = VecType::new("i32".to_string());
        assert_eq!(vec_type.element_type, "i32");
        assert!(vec_type.methods.contains_key("push"));
        assert!(vec_type.methods.contains_key("pop"));
        assert!(vec_type.methods.contains_key("len"));
    }
    
    #[test]
    fn test_vec_method_generation() {
        let vec_type = VecType::new("f64".to_string());
        let instructions = vec_type.generate_method_call("new", &[]);
        assert!(!instructions.is_empty());
    }
    
    #[test]
    fn test_array_operations() {
        let instructions = ArrayOps::generate_method_call("len", &[Value::Reg(1)]);
        assert!(!instructions.is_empty());
    }
    
    #[test]
    fn test_string_operations() {
        let instructions = StringOps::generate_method_call("len", &[Value::Reg(1)]);
        assert!(!instructions.is_empty());
    }
    
    #[test]
    fn test_collection_library() {
        let mut library = CollectionLibrary::new();
        library.register_vec_type("i32".to_string());
        assert!(library.get_vec_type("i32").is_some());
    }
}/
// Built-in Result<T, E> type for error handling - Task 12.1
#[derive(Debug, Clone)]
pub enum ResultType<T, E> {
    Ok(T),
    Err(E),
}

/// Built-in Result<T, E> implementation
pub struct ResultImpl {
    pub ok_type: String,
    pub err_type: String,
    pub methods: HashMap<String, ResultMethod>,
}

#[derive(Debug, Clone)]
pub enum ResultMethod {
    IsOk,
    IsErr,
    Ok,
    Err,
    Unwrap,
    UnwrapOr,
    UnwrapOrElse,
    Expect,
    Map,
    MapErr,
    And,
    AndThen,
    Or,
    OrElse,
}

impl ResultImpl {
    pub fn new(ok_type: String, err_type: String) -> Self {
        let mut methods = HashMap::new();
        
        // Add all Result methods
        methods.insert("is_ok".to_string(), ResultMethod::IsOk);
        methods.insert("is_err".to_string(), ResultMethod::IsErr);
        methods.insert("ok".to_string(), ResultMethod::Ok);
        methods.insert("err".to_string(), ResultMethod::Err);
        methods.insert("unwrap".to_string(), ResultMethod::Unwrap);
        methods.insert("unwrap_or".to_string(), ResultMethod::UnwrapOr);
        methods.insert("unwrap_or_else".to_string(), ResultMethod::UnwrapOrElse);
        methods.insert("expect".to_string(), ResultMethod::Expect);
        methods.insert("map".to_string(), ResultMethod::Map);
        methods.insert("map_err".to_string(), ResultMethod::MapErr);
        methods.insert("and".to_string(), ResultMethod::And);
        methods.insert("and_then".to_string(), ResultMethod::AndThen);
        methods.insert("or".to_string(), ResultMethod::Or);
        methods.insert("or_else".to_string(), ResultMethod::OrElse);
        
        ResultImpl {
            ok_type,
            err_type,
            methods,
        }
    }
    
    /// Generate LLVM IR for Result method call
    pub fn generate_method_call(&self, method: &str, args: &[Value]) -> Vec<Inst> {
        match self.methods.get(method) {
            Some(ResultMethod::IsOk) => self.generate_is_ok(args),
            Some(ResultMethod::IsErr) => self.generate_is_err(args),
            Some(ResultMethod::Ok) => self.generate_ok(args),
            Some(ResultMethod::Err) => self.generate_err(args),
            Some(ResultMethod::Unwrap) => self.generate_unwrap(args),
            Some(ResultMethod::UnwrapOr) => self.generate_unwrap_or(args),
            Some(ResultMethod::UnwrapOrElse) => self.generate_unwrap_or_else(args),
            Some(ResultMethod::Expect) => self.generate_expect(args),
            Some(ResultMethod::Map) => self.generate_map(args),
            Some(ResultMethod::MapErr) => self.generate_map_err(args),
            Some(ResultMethod::And) => self.generate_and(args),
            Some(ResultMethod::AndThen) => self.generate_and_then(args),
            Some(ResultMethod::Or) => self.generate_or(args),
            Some(ResultMethod::OrElse) => self.generate_or_else(args),
            None => panic!("Unknown Result method: {}", method),
        }
    }
    
    fn generate_is_ok(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 1 {
            panic!("Result::is_ok requires 1 argument (self)");
        }
        vec![
            // Check discriminant (0 = Ok, 1 = Err)
            Inst::EnumDiscriminant {
                result: Value::Reg(100),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(101),
                left: Value::Reg(100),
                right: Value::ImmInt(0),
            }
        ]
    }
    
    fn generate_is_err(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 1 {
            panic!("Result::is_err requires 1 argument (self)");
        }
        vec![
            // Check discriminant (0 = Ok, 1 = Err)
            Inst::EnumDiscriminant {
                result: Value::Reg(102),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(103),
                left: Value::Reg(102),
                right: Value::ImmInt(1),
            }
        ]
    }    

    fn generate_ok(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 1 {
            panic!("Result::ok requires 1 argument (self)");
        }
        vec![
            // Check if Ok variant and extract value
            Inst::EnumDiscriminant {
                result: Value::Reg(104),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(105),
                left: Value::Reg(104),
                right: Value::ImmInt(0),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(105),
                true_label: "extract_ok".to_string(),
                false_label: "return_none".to_string(),
            },
            Inst::Label("extract_ok".to_string()),
            Inst::EnumVariantData {
                result: Value::Reg(106),
                enum_ptr: args[0].clone(),
                variant_index: 0,
            },
            // Create Some(value)
            Inst::EnumConstruct {
                result: Value::Reg(107),
                enum_name: format!("Option<{}>", self.ok_type),
                variant_name: "Some".to_string(),
                variant_index: 1,
                data: vec![Value::Reg(106)],
            },
            Inst::Jump("end_ok".to_string()),
            Inst::Label("return_none".to_string()),
            // Create None
            Inst::EnumConstruct {
                result: Value::Reg(108),
                enum_name: format!("Option<{}>", self.ok_type),
                variant_name: "None".to_string(),
                variant_index: 0,
                data: vec![],
            },
            Inst::Label("end_ok".to_string()),
        ]
    }
    
    fn generate_err(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 1 {
            panic!("Result::err requires 1 argument (self)");
        }
        vec![
            // Check if Err variant and extract error
            Inst::EnumDiscriminant {
                result: Value::Reg(109),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(110),
                left: Value::Reg(109),
                right: Value::ImmInt(1),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(110),
                true_label: "extract_err".to_string(),
                false_label: "return_none_err".to_string(),
            },
            Inst::Label("extract_err".to_string()),
            Inst::EnumVariantData {
                result: Value::Reg(111),
                enum_ptr: args[0].clone(),
                variant_index: 1,
            },
            // Create Some(error)
            Inst::EnumConstruct {
                result: Value::Reg(112),
                enum_name: format!("Option<{}>", self.err_type),
                variant_name: "Some".to_string(),
                variant_index: 1,
                data: vec![Value::Reg(111)],
            },
            Inst::Jump("end_err".to_string()),
            Inst::Label("return_none_err".to_string()),
            // Create None
            Inst::EnumConstruct {
                result: Value::Reg(113),
                enum_name: format!("Option<{}>", self.err_type),
                variant_name: "None".to_string(),
                variant_index: 0,
                data: vec![],
            },
            Inst::Label("end_err".to_string()),
        ]
    }    

    fn generate_unwrap(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 1 {
            panic!("Result::unwrap requires 1 argument (self)");
        }
        vec![
            // Check if Ok variant
            Inst::EnumDiscriminant {
                result: Value::Reg(114),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(115),
                left: Value::Reg(114),
                right: Value::ImmInt(0),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(115),
                true_label: "unwrap_ok".to_string(),
                false_label: "panic_unwrap".to_string(),
            },
            Inst::Label("unwrap_ok".to_string()),
            Inst::EnumVariantData {
                result: Value::Reg(116),
                enum_ptr: args[0].clone(),
                variant_index: 0,
            },
            Inst::Jump("end_unwrap".to_string()),
            Inst::Label("panic_unwrap".to_string()),
            // Panic with error message
            Inst::Call {
                result: None,
                function: "panic".to_string(),
                args: vec![Value::ImmString("called `Result::unwrap()` on an `Err` value".to_string())],
            },
            Inst::Label("end_unwrap".to_string()),
        ]
    }
    
    fn generate_unwrap_or(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Result::unwrap_or requires 2 arguments (self, default)");
        }
        vec![
            // Check if Ok variant
            Inst::EnumDiscriminant {
                result: Value::Reg(117),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(118),
                left: Value::Reg(117),
                right: Value::ImmInt(0),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(118),
                true_label: "unwrap_or_ok".to_string(),
                false_label: "unwrap_or_default".to_string(),
            },
            Inst::Label("unwrap_or_ok".to_string()),
            Inst::EnumVariantData {
                result: Value::Reg(119),
                enum_ptr: args[0].clone(),
                variant_index: 0,
            },
            Inst::Jump("end_unwrap_or".to_string()),
            Inst::Label("unwrap_or_default".to_string()),
            // Return default value
            Inst::Alloca(Value::Reg(120), "default_value".to_string()),
            Inst::Store(Value::Reg(120), args[1].clone()),
            Inst::Label("end_unwrap_or".to_string()),
        ]
    }
    
    fn generate_unwrap_or_else(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Result::unwrap_or_else requires 2 arguments (self, closure)");
        }
        vec![
            // Check if Ok variant
            Inst::EnumDiscriminant {
                result: Value::Reg(121),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(122),
                left: Value::Reg(121),
                right: Value::ImmInt(0),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(122),
                true_label: "unwrap_or_else_ok".to_string(),
                false_label: "unwrap_or_else_closure".to_string(),
            },
            Inst::Label("unwrap_or_else_ok".to_string()),
            Inst::EnumVariantData {
                result: Value::Reg(123),
                enum_ptr: args[0].clone(),
                variant_index: 0,
            },
            Inst::Jump("end_unwrap_or_else".to_string()),
            Inst::Label("unwrap_or_else_closure".to_string()),
            // Extract error and call closure
            Inst::EnumVariantData {
                result: Value::Reg(124),
                enum_ptr: args[0].clone(),
                variant_index: 1,
            },
            Inst::Call {
                result: Some(Value::Reg(125)),
                function: "closure_call".to_string(),
                args: vec![args[1].clone(), Value::Reg(124)],
            },
            Inst::Label("end_unwrap_or_else".to_string()),
        ]
    }    
    
fn generate_expect(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Result::expect requires 2 arguments (self, message)");
        }
        vec![
            // Check if Ok variant
            Inst::EnumDiscriminant {
                result: Value::Reg(126),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(127),
                left: Value::Reg(126),
                right: Value::ImmInt(0),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(127),
                true_label: "expect_ok".to_string(),
                false_label: "panic_expect".to_string(),
            },
            Inst::Label("expect_ok".to_string()),
            Inst::EnumVariantData {
                result: Value::Reg(128),
                enum_ptr: args[0].clone(),
                variant_index: 0,
            },
            Inst::Jump("end_expect".to_string()),
            Inst::Label("panic_expect".to_string()),
            // Panic with custom message
            Inst::Call {
                result: None,
                function: "panic".to_string(),
                args: vec![args[1].clone()],
            },
            Inst::Label("end_expect".to_string()),
        ]
    }
    
    fn generate_map(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Result::map requires 2 arguments (self, closure)");
        }
        vec![
            // Check if Ok variant
            Inst::EnumDiscriminant {
                result: Value::Reg(129),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(130),
                left: Value::Reg(129),
                right: Value::ImmInt(0),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(130),
                true_label: "map_ok".to_string(),
                false_label: "map_err_passthrough".to_string(),
            },
            Inst::Label("map_ok".to_string()),
            // Extract Ok value and apply closure
            Inst::EnumVariantData {
                result: Value::Reg(131),
                enum_ptr: args[0].clone(),
                variant_index: 0,
            },
            Inst::Call {
                result: Some(Value::Reg(132)),
                function: "closure_call".to_string(),
                args: vec![args[1].clone(), Value::Reg(131)],
            },
            // Create new Ok with mapped value
            Inst::EnumConstruct {
                result: Value::Reg(133),
                enum_name: format!("Result<U, {}>", self.err_type),
                variant_name: "Ok".to_string(),
                variant_index: 0,
                data: vec![Value::Reg(132)],
            },
            Inst::Jump("end_map".to_string()),
            Inst::Label("map_err_passthrough".to_string()),
            // Pass through Err unchanged
            Inst::EnumVariantData {
                result: Value::Reg(134),
                enum_ptr: args[0].clone(),
                variant_index: 1,
            },
            Inst::EnumConstruct {
                result: Value::Reg(135),
                enum_name: format!("Result<U, {}>", self.err_type),
                variant_name: "Err".to_string(),
                variant_index: 1,
                data: vec![Value::Reg(134)],
            },
            Inst::Label("end_map".to_string()),
        ]
    }    
  
  fn generate_map_err(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Result::map_err requires 2 arguments (self, closure)");
        }
        vec![
            // Check if Err variant
            Inst::EnumDiscriminant {
                result: Value::Reg(136),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(137),
                left: Value::Reg(136),
                right: Value::ImmInt(1),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(137),
                true_label: "map_err_transform".to_string(),
                false_label: "map_err_ok_passthrough".to_string(),
            },
            Inst::Label("map_err_transform".to_string()),
            // Extract Err value and apply closure
            Inst::EnumVariantData {
                result: Value::Reg(138),
                enum_ptr: args[0].clone(),
                variant_index: 1,
            },
            Inst::Call {
                result: Some(Value::Reg(139)),
                function: "closure_call".to_string(),
                args: vec![args[1].clone(), Value::Reg(138)],
            },
            // Create new Err with mapped error
            Inst::EnumConstruct {
                result: Value::Reg(140),
                enum_name: format!("Result<{}, F>", self.ok_type),
                variant_name: "Err".to_string(),
                variant_index: 1,
                data: vec![Value::Reg(139)],
            },
            Inst::Jump("end_map_err".to_string()),
            Inst::Label("map_err_ok_passthrough".to_string()),
            // Pass through Ok unchanged
            Inst::EnumVariantData {
                result: Value::Reg(141),
                enum_ptr: args[0].clone(),
                variant_index: 0,
            },
            Inst::EnumConstruct {
                result: Value::Reg(142),
                enum_name: format!("Result<{}, F>", self.ok_type),
                variant_name: "Ok".to_string(),
                variant_index: 0,
                data: vec![Value::Reg(141)],
            },
            Inst::Label("end_map_err".to_string()),
        ]
    }
    
    fn generate_and(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Result::and requires 2 arguments (self, other)");
        }
        vec![
            // Check if self is Ok
            Inst::EnumDiscriminant {
                result: Value::Reg(143),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(144),
                left: Value::Reg(143),
                right: Value::ImmInt(0),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(144),
                true_label: "and_return_other".to_string(),
                false_label: "and_return_self_err".to_string(),
            },
            Inst::Label("and_return_other".to_string()),
            // Return other Result
            Inst::Alloca(Value::Reg(145), "and_result".to_string()),
            Inst::Store(Value::Reg(145), args[1].clone()),
            Inst::Jump("end_and".to_string()),
            Inst::Label("and_return_self_err".to_string()),
            // Return self (which is Err)
            Inst::Alloca(Value::Reg(146), "and_result".to_string()),
            Inst::Store(Value::Reg(146), args[0].clone()),
            Inst::Label("end_and".to_string()),
        ]
    }    
  
  fn generate_and_then(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Result::and_then requires 2 arguments (self, closure)");
        }
        vec![
            // Check if Ok variant
            Inst::EnumDiscriminant {
                result: Value::Reg(147),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(148),
                left: Value::Reg(147),
                right: Value::ImmInt(0),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(148),
                true_label: "and_then_ok".to_string(),
                false_label: "and_then_err_passthrough".to_string(),
            },
            Inst::Label("and_then_ok".to_string()),
            // Extract Ok value and call closure
            Inst::EnumVariantData {
                result: Value::Reg(149),
                enum_ptr: args[0].clone(),
                variant_index: 0,
            },
            Inst::Call {
                result: Some(Value::Reg(150)),
                function: "closure_call".to_string(),
                args: vec![args[1].clone(), Value::Reg(149)],
            },
            Inst::Jump("end_and_then".to_string()),
            Inst::Label("and_then_err_passthrough".to_string()),
            // Pass through Err unchanged
            Inst::EnumVariantData {
                result: Value::Reg(151),
                enum_ptr: args[0].clone(),
                variant_index: 1,
            },
            Inst::EnumConstruct {
                result: Value::Reg(152),
                enum_name: format!("Result<U, {}>", self.err_type),
                variant_name: "Err".to_string(),
                variant_index: 1,
                data: vec![Value::Reg(151)],
            },
            Inst::Label("end_and_then".to_string()),
        ]
    }
    
    fn generate_or(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Result::or requires 2 arguments (self, other)");
        }
        vec![
            // Check if self is Ok
            Inst::EnumDiscriminant {
                result: Value::Reg(153),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(154),
                left: Value::Reg(153),
                right: Value::ImmInt(0),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(154),
                true_label: "or_return_self".to_string(),
                false_label: "or_return_other".to_string(),
            },
            Inst::Label("or_return_self".to_string()),
            // Return self (which is Ok)
            Inst::Alloca(Value::Reg(155), "or_result".to_string()),
            Inst::Store(Value::Reg(155), args[0].clone()),
            Inst::Jump("end_or".to_string()),
            Inst::Label("or_return_other".to_string()),
            // Return other Result
            Inst::Alloca(Value::Reg(156), "or_result".to_string()),
            Inst::Store(Value::Reg(156), args[1].clone()),
            Inst::Label("end_or".to_string()),
        ]
    }
    
    fn generate_or_else(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Result::or_else requires 2 arguments (self, closure)");
        }
        vec![
            // Check if Ok variant
            Inst::EnumDiscriminant {
                result: Value::Reg(157),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(158),
                left: Value::Reg(157),
                right: Value::ImmInt(0),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(158),
                true_label: "or_else_ok_passthrough".to_string(),
                false_label: "or_else_err".to_string(),
            },
            Inst::Label("or_else_ok_passthrough".to_string()),
            // Pass through Ok unchanged
            Inst::EnumVariantData {
                result: Value::Reg(159),
                enum_ptr: args[0].clone(),
                variant_index: 0,
            },
            Inst::EnumConstruct {
                result: Value::Reg(160),
                enum_name: format!("Result<{}, F>", self.ok_type),
                variant_name: "Ok".to_string(),
                variant_index: 0,
                data: vec![Value::Reg(159)],
            },
            Inst::Jump("end_or_else".to_string()),
            Inst::Label("or_else_err".to_string()),
            // Extract Err value and call closure
            Inst::EnumVariantData {
                result: Value::Reg(161),
                enum_ptr: args[0].clone(),
                variant_index: 1,
            },
            Inst::Call {
                result: Some(Value::Reg(162)),
                function: "closure_call".to_string(),
                args: vec![args[1].clone(), Value::Reg(161)],
            },
            Inst::Label("end_or_else".to_string()),
        ]
    }
}///
 Built-in Option<T> type for nullable values - Task 12.1
#[derive(Debug, Clone)]
pub enum OptionType<T> {
    Some(T),
    None,
}

/// Built-in Option<T> implementation
pub struct OptionImpl {
    pub inner_type: String,
    pub methods: HashMap<String, OptionMethod>,
}

#[derive(Debug, Clone)]
pub enum OptionMethod {
    IsSome,
    IsNone,
    Unwrap,
    UnwrapOr,
    UnwrapOrElse,
    Expect,
    Map,
    MapOr,
    MapOrElse,
    And,
    AndThen,
    Or,
    OrElse,
    Filter,
    Take,
    Replace,
}

impl OptionImpl {
    pub fn new(inner_type: String) -> Self {
        let mut methods = HashMap::new();
        
        // Add all Option methods
        methods.insert("is_some".to_string(), OptionMethod::IsSome);
        methods.insert("is_none".to_string(), OptionMethod::IsNone);
        methods.insert("unwrap".to_string(), OptionMethod::Unwrap);
        methods.insert("unwrap_or".to_string(), OptionMethod::UnwrapOr);
        methods.insert("unwrap_or_else".to_string(), OptionMethod::UnwrapOrElse);
        methods.insert("expect".to_string(), OptionMethod::Expect);
        methods.insert("map".to_string(), OptionMethod::Map);
        methods.insert("map_or".to_string(), OptionMethod::MapOr);
        methods.insert("map_or_else".to_string(), OptionMethod::MapOrElse);
        methods.insert("and".to_string(), OptionMethod::And);
        methods.insert("and_then".to_string(), OptionMethod::AndThen);
        methods.insert("or".to_string(), OptionMethod::Or);
        methods.insert("or_else".to_string(), OptionMethod::OrElse);
        methods.insert("filter".to_string(), OptionMethod::Filter);
        methods.insert("take".to_string(), OptionMethod::Take);
        methods.insert("replace".to_string(), OptionMethod::Replace);
        
        OptionImpl {
            inner_type,
            methods,
        }
    }
    
    /// Generate LLVM IR for Option method call
    pub fn generate_method_call(&self, method: &str, args: &[Value]) -> Vec<Inst> {
        match self.methods.get(method) {
            Some(OptionMethod::IsSome) => self.generate_is_some(args),
            Some(OptionMethod::IsNone) => self.generate_is_none(args),
            Some(OptionMethod::Unwrap) => self.generate_unwrap(args),
            Some(OptionMethod::UnwrapOr) => self.generate_unwrap_or(args),
            Some(OptionMethod::UnwrapOrElse) => self.generate_unwrap_or_else(args),
            Some(OptionMethod::Expect) => self.generate_expect(args),
            Some(OptionMethod::Map) => self.generate_map(args),
            Some(OptionMethod::MapOr) => self.generate_map_or(args),
            Some(OptionMethod::MapOrElse) => self.generate_map_or_else(args),
            Some(OptionMethod::And) => self.generate_and(args),
            Some(OptionMethod::AndThen) => self.generate_and_then(args),
            Some(OptionMethod::Or) => self.generate_or(args),
            Some(OptionMethod::OrElse) => self.generate_or_else(args),
            Some(OptionMethod::Filter) => self.generate_filter(args),
            Some(OptionMethod::Take) => self.generate_take(args),
            Some(OptionMethod::Replace) => self.generate_replace(args),
            None => panic!("Unknown Option method: {}", method),
        }
    }    

    fn generate_is_some(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 1 {
            panic!("Option::is_some requires 1 argument (self)");
        }
        vec![
            // Check discriminant (0 = None, 1 = Some)
            Inst::EnumDiscriminant {
                result: Value::Reg(200),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(201),
                left: Value::Reg(200),
                right: Value::ImmInt(1),
            }
        ]
    }
    
    fn generate_is_none(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 1 {
            panic!("Option::is_none requires 1 argument (self)");
        }
        vec![
            // Check discriminant (0 = None, 1 = Some)
            Inst::EnumDiscriminant {
                result: Value::Reg(202),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(203),
                left: Value::Reg(202),
                right: Value::ImmInt(0),
            }
        ]
    }
    
    fn generate_unwrap(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 1 {
            panic!("Option::unwrap requires 1 argument (self)");
        }
        vec![
            // Check if Some variant
            Inst::EnumDiscriminant {
                result: Value::Reg(204),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(205),
                left: Value::Reg(204),
                right: Value::ImmInt(1),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(205),
                true_label: "unwrap_some".to_string(),
                false_label: "panic_unwrap_none".to_string(),
            },
            Inst::Label("unwrap_some".to_string()),
            Inst::EnumVariantData {
                result: Value::Reg(206),
                enum_ptr: args[0].clone(),
                variant_index: 1,
            },
            Inst::Jump("end_unwrap_option".to_string()),
            Inst::Label("panic_unwrap_none".to_string()),
            // Panic with error message
            Inst::Call {
                result: None,
                function: "panic".to_string(),
                args: vec![Value::ImmString("called `Option::unwrap()` on a `None` value".to_string())],
            },
            Inst::Label("end_unwrap_option".to_string()),
        ]
    }
    
    fn generate_unwrap_or(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Option::unwrap_or requires 2 arguments (self, default)");
        }
        vec![
            // Check if Some variant
            Inst::EnumDiscriminant {
                result: Value::Reg(207),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(208),
                left: Value::Reg(207),
                right: Value::ImmInt(1),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(208),
                true_label: "unwrap_or_some".to_string(),
                false_label: "unwrap_or_default".to_string(),
            },
            Inst::Label("unwrap_or_some".to_string()),
            Inst::EnumVariantData {
                result: Value::Reg(209),
                enum_ptr: args[0].clone(),
                variant_index: 1,
            },
            Inst::Jump("end_unwrap_or_option".to_string()),
            Inst::Label("unwrap_or_default".to_string()),
            // Return default value
            Inst::Alloca(Value::Reg(210), "default_value".to_string()),
            Inst::Store(Value::Reg(210), args[1].clone()),
            Inst::Label("end_unwrap_or_option".to_string()),
        ]
    }    
 
   fn generate_unwrap_or_else(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Option::unwrap_or_else requires 2 arguments (self, closure)");
        }
        vec![
            // Check if Some variant
            Inst::EnumDiscriminant {
                result: Value::Reg(211),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(212),
                left: Value::Reg(211),
                right: Value::ImmInt(1),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(212),
                true_label: "unwrap_or_else_some".to_string(),
                false_label: "unwrap_or_else_closure".to_string(),
            },
            Inst::Label("unwrap_or_else_some".to_string()),
            Inst::EnumVariantData {
                result: Value::Reg(213),
                enum_ptr: args[0].clone(),
                variant_index: 1,
            },
            Inst::Jump("end_unwrap_or_else_option".to_string()),
            Inst::Label("unwrap_or_else_closure".to_string()),
            // Call closure
            Inst::Call {
                result: Some(Value::Reg(214)),
                function: "closure_call".to_string(),
                args: vec![args[1].clone()],
            },
            Inst::Label("end_unwrap_or_else_option".to_string()),
        ]
    }
    
    fn generate_expect(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Option::expect requires 2 arguments (self, message)");
        }
        vec![
            // Check if Some variant
            Inst::EnumDiscriminant {
                result: Value::Reg(215),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(216),
                left: Value::Reg(215),
                right: Value::ImmInt(1),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(216),
                true_label: "expect_some".to_string(),
                false_label: "panic_expect_none".to_string(),
            },
            Inst::Label("expect_some".to_string()),
            Inst::EnumVariantData {
                result: Value::Reg(217),
                enum_ptr: args[0].clone(),
                variant_index: 1,
            },
            Inst::Jump("end_expect_option".to_string()),
            Inst::Label("panic_expect_none".to_string()),
            // Panic with custom message
            Inst::Call {
                result: None,
                function: "panic".to_string(),
                args: vec![args[1].clone()],
            },
            Inst::Label("end_expect_option".to_string()),
        ]
    }
    
    fn generate_map(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Option::map requires 2 arguments (self, closure)");
        }
        vec![
            // Check if Some variant
            Inst::EnumDiscriminant {
                result: Value::Reg(218),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(219),
                left: Value::Reg(218),
                right: Value::ImmInt(1),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(219),
                true_label: "map_some".to_string(),
                false_label: "map_none_passthrough".to_string(),
            },
            Inst::Label("map_some".to_string()),
            // Extract Some value and apply closure
            Inst::EnumVariantData {
                result: Value::Reg(220),
                enum_ptr: args[0].clone(),
                variant_index: 1,
            },
            Inst::Call {
                result: Some(Value::Reg(221)),
                function: "closure_call".to_string(),
                args: vec![args[1].clone(), Value::Reg(220)],
            },
            // Create new Some with mapped value
            Inst::EnumConstruct {
                result: Value::Reg(222),
                enum_name: "Option<U>".to_string(),
                variant_name: "Some".to_string(),
                variant_index: 1,
                data: vec![Value::Reg(221)],
            },
            Inst::Jump("end_map_option".to_string()),
            Inst::Label("map_none_passthrough".to_string()),
            // Return None
            Inst::EnumConstruct {
                result: Value::Reg(223),
                enum_name: "Option<U>".to_string(),
                variant_name: "None".to_string(),
                variant_index: 0,
                data: vec![],
            },
            Inst::Label("end_map_option".to_string()),
        ]
    }   
 
    fn generate_map_or(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 3 {
            panic!("Option::map_or requires 3 arguments (self, default, closure)");
        }
        vec![
            // Check if Some variant
            Inst::EnumDiscriminant {
                result: Value::Reg(224),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(225),
                left: Value::Reg(224),
                right: Value::ImmInt(1),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(225),
                true_label: "map_or_some".to_string(),
                false_label: "map_or_default".to_string(),
            },
            Inst::Label("map_or_some".to_string()),
            // Extract Some value and apply closure
            Inst::EnumVariantData {
                result: Value::Reg(226),
                enum_ptr: args[0].clone(),
                variant_index: 1,
            },
            Inst::Call {
                result: Some(Value::Reg(227)),
                function: "closure_call".to_string(),
                args: vec![args[2].clone(), Value::Reg(226)],
            },
            Inst::Jump("end_map_or_option".to_string()),
            Inst::Label("map_or_default".to_string()),
            // Return default value
            Inst::Alloca(Value::Reg(228), "map_or_default".to_string()),
            Inst::Store(Value::Reg(228), args[1].clone()),
            Inst::Label("end_map_or_option".to_string()),
        ]
    }
    
    fn generate_map_or_else(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 3 {
            panic!("Option::map_or_else requires 3 arguments (self, default_closure, map_closure)");
        }
        vec![
            // Check if Some variant
            Inst::EnumDiscriminant {
                result: Value::Reg(229),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(230),
                left: Value::Reg(229),
                right: Value::ImmInt(1),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(230),
                true_label: "map_or_else_some".to_string(),
                false_label: "map_or_else_default".to_string(),
            },
            Inst::Label("map_or_else_some".to_string()),
            // Extract Some value and apply map closure
            Inst::EnumVariantData {
                result: Value::Reg(231),
                enum_ptr: args[0].clone(),
                variant_index: 1,
            },
            Inst::Call {
                result: Some(Value::Reg(232)),
                function: "closure_call".to_string(),
                args: vec![args[2].clone(), Value::Reg(231)],
            },
            Inst::Jump("end_map_or_else_option".to_string()),
            Inst::Label("map_or_else_default".to_string()),
            // Call default closure
            Inst::Call {
                result: Some(Value::Reg(233)),
                function: "closure_call".to_string(),
                args: vec![args[1].clone()],
            },
            Inst::Label("end_map_or_else_option".to_string()),
        ]
    }
    
    fn generate_and(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Option::and requires 2 arguments (self, other)");
        }
        vec![
            // Check if self is Some
            Inst::EnumDiscriminant {
                result: Value::Reg(234),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(235),
                left: Value::Reg(234),
                right: Value::ImmInt(1),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(235),
                true_label: "and_return_other".to_string(),
                false_label: "and_return_none".to_string(),
            },
            Inst::Label("and_return_other".to_string()),
            // Return other Option
            Inst::Alloca(Value::Reg(236), "and_result".to_string()),
            Inst::Store(Value::Reg(236), args[1].clone()),
            Inst::Jump("end_and_option".to_string()),
            Inst::Label("and_return_none".to_string()),
            // Return None
            Inst::EnumConstruct {
                result: Value::Reg(237),
                enum_name: "Option<U>".to_string(),
                variant_name: "None".to_string(),
                variant_index: 0,
                data: vec![],
            },
            Inst::Label("end_and_option".to_string()),
        ]
    } 
   
    fn generate_and_then(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Option::and_then requires 2 arguments (self, closure)");
        }
        vec![
            // Check if Some variant
            Inst::EnumDiscriminant {
                result: Value::Reg(238),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(239),
                left: Value::Reg(238),
                right: Value::ImmInt(1),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(239),
                true_label: "and_then_some".to_string(),
                false_label: "and_then_none_passthrough".to_string(),
            },
            Inst::Label("and_then_some".to_string()),
            // Extract Some value and call closure
            Inst::EnumVariantData {
                result: Value::Reg(240),
                enum_ptr: args[0].clone(),
                variant_index: 1,
            },
            Inst::Call {
                result: Some(Value::Reg(241)),
                function: "closure_call".to_string(),
                args: vec![args[1].clone(), Value::Reg(240)],
            },
            Inst::Jump("end_and_then_option".to_string()),
            Inst::Label("and_then_none_passthrough".to_string()),
            // Return None
            Inst::EnumConstruct {
                result: Value::Reg(242),
                enum_name: "Option<U>".to_string(),
                variant_name: "None".to_string(),
                variant_index: 0,
                data: vec![],
            },
            Inst::Label("end_and_then_option".to_string()),
        ]
    }
    
    fn generate_or(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Option::or requires 2 arguments (self, other)");
        }
        vec![
            // Check if self is Some
            Inst::EnumDiscriminant {
                result: Value::Reg(243),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(244),
                left: Value::Reg(243),
                right: Value::ImmInt(1),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(244),
                true_label: "or_return_self".to_string(),
                false_label: "or_return_other".to_string(),
            },
            Inst::Label("or_return_self".to_string()),
            // Return self (which is Some)
            Inst::Alloca(Value::Reg(245), "or_result".to_string()),
            Inst::Store(Value::Reg(245), args[0].clone()),
            Inst::Jump("end_or_option".to_string()),
            Inst::Label("or_return_other".to_string()),
            // Return other Option
            Inst::Alloca(Value::Reg(246), "or_result".to_string()),
            Inst::Store(Value::Reg(246), args[1].clone()),
            Inst::Label("end_or_option".to_string()),
        ]
    }
    
    fn generate_or_else(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Option::or_else requires 2 arguments (self, closure)");
        }
        vec![
            // Check if Some variant
            Inst::EnumDiscriminant {
                result: Value::Reg(247),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(248),
                left: Value::Reg(247),
                right: Value::ImmInt(1),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(248),
                true_label: "or_else_some_passthrough".to_string(),
                false_label: "or_else_none".to_string(),
            },
            Inst::Label("or_else_some_passthrough".to_string()),
            // Pass through Some unchanged
            Inst::EnumVariantData {
                result: Value::Reg(249),
                enum_ptr: args[0].clone(),
                variant_index: 1,
            },
            Inst::EnumConstruct {
                result: Value::Reg(250),
                enum_name: format!("Option<{}>", self.inner_type),
                variant_name: "Some".to_string(),
                variant_index: 1,
                data: vec![Value::Reg(249)],
            },
            Inst::Jump("end_or_else_option".to_string()),
            Inst::Label("or_else_none".to_string()),
            // Call closure
            Inst::Call {
                result: Some(Value::Reg(251)),
                function: "closure_call".to_string(),
                args: vec![args[1].clone()],
            },
            Inst::Label("end_or_else_option".to_string()),
        ]
    }    

    fn generate_filter(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Option::filter requires 2 arguments (self, predicate)");
        }
        vec![
            // Check if Some variant
            Inst::EnumDiscriminant {
                result: Value::Reg(252),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(253),
                left: Value::Reg(252),
                right: Value::ImmInt(1),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(253),
                true_label: "filter_some".to_string(),
                false_label: "filter_none_passthrough".to_string(),
            },
            Inst::Label("filter_some".to_string()),
            // Extract Some value and test predicate
            Inst::EnumVariantData {
                result: Value::Reg(254),
                enum_ptr: args[0].clone(),
                variant_index: 1,
            },
            Inst::Call {
                result: Some(Value::Reg(255)),
                function: "closure_call".to_string(),
                args: vec![args[1].clone(), Value::Reg(254)],
            },
            // Branch based on predicate result
            Inst::Branch {
                condition: Value::Reg(255),
                true_label: "filter_keep".to_string(),
                false_label: "filter_discard".to_string(),
            },
            Inst::Label("filter_keep".to_string()),
            // Keep the Some value
            Inst::EnumConstruct {
                result: Value::Reg(256),
                enum_name: format!("Option<{}>", self.inner_type),
                variant_name: "Some".to_string(),
                variant_index: 1,
                data: vec![Value::Reg(254)],
            },
            Inst::Jump("end_filter_option".to_string()),
            Inst::Label("filter_discard".to_string()),
            // Return None
            Inst::EnumConstruct {
                result: Value::Reg(257),
                enum_name: format!("Option<{}>", self.inner_type),
                variant_name: "None".to_string(),
                variant_index: 0,
                data: vec![],
            },
            Inst::Jump("end_filter_option".to_string()),
            Inst::Label("filter_none_passthrough".to_string()),
            // Return None unchanged
            Inst::EnumConstruct {
                result: Value::Reg(258),
                enum_name: format!("Option<{}>", self.inner_type),
                variant_name: "None".to_string(),
                variant_index: 0,
                data: vec![],
            },
            Inst::Label("end_filter_option".to_string()),
        ]
    }
    
    fn generate_take(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 1 {
            panic!("Option::take requires 1 argument (self)");
        }
        vec![
            // Check if Some variant
            Inst::EnumDiscriminant {
                result: Value::Reg(259),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(260),
                left: Value::Reg(259),
                right: Value::ImmInt(1),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(260),
                true_label: "take_some".to_string(),
                false_label: "take_none".to_string(),
            },
            Inst::Label("take_some".to_string()),
            // Extract Some value
            Inst::EnumVariantData {
                result: Value::Reg(261),
                enum_ptr: args[0].clone(),
                variant_index: 1,
            },
            // Replace self with None
            Inst::EnumConstruct {
                result: Value::Reg(262),
                enum_name: format!("Option<{}>", self.inner_type),
                variant_name: "None".to_string(),
                variant_index: 0,
                data: vec![],
            },
            Inst::Store(args[0].clone(), Value::Reg(262)),
            // Return the taken value as Some
            Inst::EnumConstruct {
                result: Value::Reg(263),
                enum_name: format!("Option<{}>", self.inner_type),
                variant_name: "Some".to_string(),
                variant_index: 1,
                data: vec![Value::Reg(261)],
            },
            Inst::Jump("end_take_option".to_string()),
            Inst::Label("take_none".to_string()),
            // Return None
            Inst::EnumConstruct {
                result: Value::Reg(264),
                enum_name: format!("Option<{}>", self.inner_type),
                variant_name: "None".to_string(),
                variant_index: 0,
                data: vec![],
            },
            Inst::Label("end_take_option".to_string()),
        ]
    }
    
    fn generate_replace(&self, args: &[Value]) -> Vec<Inst> {
        if args.len() != 2 {
            panic!("Option::replace requires 2 arguments (self, value)");
        }
        vec![
            // Get current value (if any)
            Inst::EnumDiscriminant {
                result: Value::Reg(265),
                enum_ptr: args[0].clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(266),
                left: Value::Reg(265),
                right: Value::ImmInt(1),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(266),
                true_label: "replace_some".to_string(),
                false_label: "replace_none".to_string(),
            },
            Inst::Label("replace_some".to_string()),
            // Extract current Some value
            Inst::EnumVariantData {
                result: Value::Reg(267),
                enum_ptr: args[0].clone(),
                variant_index: 1,
            },
            // Create Some with old value to return
            Inst::EnumConstruct {
                result: Value::Reg(268),
                enum_name: format!("Option<{}>", self.inner_type),
                variant_name: "Some".to_string(),
                variant_index: 1,
                data: vec![Value::Reg(267)],
            },
            Inst::Jump("replace_update".to_string()),
            Inst::Label("replace_none".to_string()),
            // Create None to return
            Inst::EnumConstruct {
                result: Value::Reg(269),
                enum_name: format!("Option<{}>", self.inner_type),
                variant_name: "None".to_string(),
                variant_index: 0,
                data: vec![],
            },
            Inst::Label("replace_update".to_string()),
            // Replace self with new Some value
            Inst::EnumConstruct {
                result: Value::Reg(270),
                enum_name: format!("Option<{}>", self.inner_type),
                variant_name: "Some".to_string(),
                variant_index: 1,
                data: vec![args[1].clone()],
            },
            Inst::Store(args[0].clone(), Value::Reg(270)),
        ]
    }
}/// Er
ror handling library manager for Result and Option types - Task 12.1
pub struct ErrorHandlingLibrary {
    pub result_types: HashMap<String, ResultImpl>,
    pub option_types: HashMap<String, OptionImpl>,
}

impl ErrorHandlingLibrary {
    pub fn new() -> Self {
        ErrorHandlingLibrary {
            result_types: HashMap::new(),
            option_types: HashMap::new(),
        }
    }
    
    /// Register a new Result<T, E> type
    pub fn register_result_type(&mut self, ok_type: String, err_type: String) {
        let result_impl = ResultImpl::new(ok_type.clone(), err_type.clone());
        let type_key = format!("Result<{}, {}>", ok_type, err_type);
        self.result_types.insert(type_key, result_impl);
    }
    
    /// Register a new Option<T> type
    pub fn register_option_type(&mut self, inner_type: String) {
        let option_impl = OptionImpl::new(inner_type.clone());
        let type_key = format!("Option<{}>", inner_type);
        self.option_types.insert(type_key, option_impl);
    }
    
    /// Get Result type implementation
    pub fn get_result_type(&self, ok_type: &str, err_type: &str) -> Option<&ResultImpl> {
        let type_key = format!("Result<{}, {}>", ok_type, err_type);
        self.result_types.get(&type_key)
    }
    
    /// Get Option type implementation
    pub fn get_option_type(&self, inner_type: &str) -> Option<&OptionImpl> {
        let type_key = format!("Option<{}>", inner_type);
        self.option_types.get(&type_key)
    }
    
    /// Generate Result::Ok constructor
    pub fn generate_result_ok(ok_type: String, err_type: String, value: Value) -> Vec<Inst> {
        vec![
            Inst::EnumConstruct {
                result: Value::Reg(300),
                enum_name: format!("Result<{}, {}>", ok_type, err_type),
                variant_name: "Ok".to_string(),
                variant_index: 0,
                data: vec![value],
            }
        ]
    }
    
    /// Generate Result::Err constructor
    pub fn generate_result_err(ok_type: String, err_type: String, error: Value) -> Vec<Inst> {
        vec![
            Inst::EnumConstruct {
                result: Value::Reg(301),
                enum_name: format!("Result<{}, {}>", ok_type, err_type),
                variant_name: "Err".to_string(),
                variant_index: 1,
                data: vec![error],
            }
        ]
    }
    
    /// Generate Option::Some constructor
    pub fn generate_option_some(inner_type: String, value: Value) -> Vec<Inst> {
        vec![
            Inst::EnumConstruct {
                result: Value::Reg(302),
                enum_name: format!("Option<{}>", inner_type),
                variant_name: "Some".to_string(),
                variant_index: 1,
                data: vec![value],
            }
        ]
    }
    
    /// Generate Option::None constructor
    pub fn generate_option_none(inner_type: String) -> Vec<Inst> {
        vec![
            Inst::EnumConstruct {
                result: Value::Reg(303),
                enum_name: format!("Option<{}>", inner_type),
                variant_name: "None".to_string(),
                variant_index: 0,
                data: vec![],
            }
        ]
    }
    
    /// Generate ? operator for Result types
    pub fn generate_question_mark_operator(result_value: Value, ok_type: String, err_type: String) -> Vec<Inst> {
        vec![
            // Check if Result is Ok
            Inst::EnumDiscriminant {
                result: Value::Reg(304),
                enum_ptr: result_value.clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(305),
                left: Value::Reg(304),
                right: Value::ImmInt(0),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(305),
                true_label: "question_mark_ok".to_string(),
                false_label: "question_mark_err".to_string(),
            },
            Inst::Label("question_mark_ok".to_string()),
            // Extract Ok value and continue
            Inst::EnumVariantData {
                result: Value::Reg(306),
                enum_ptr: result_value.clone(),
                variant_index: 0,
            },
            Inst::Jump("question_mark_continue".to_string()),
            Inst::Label("question_mark_err".to_string()),
            // Extract Err value and early return
            Inst::EnumVariantData {
                result: Value::Reg(307),
                enum_ptr: result_value,
                variant_index: 1,
            },
            // Create new Err with same error type
            Inst::EnumConstruct {
                result: Value::Reg(308),
                enum_name: format!("Result<{}, {}>", ok_type, err_type),
                variant_name: "Err".to_string(),
                variant_index: 1,
                data: vec![Value::Reg(307)],
            },
            // Early return with error
            Inst::Return(Value::Reg(308)),
            Inst::Label("question_mark_continue".to_string()),
        ]
    }
    
    /// Generate try! macro (legacy ? operator)
    pub fn generate_try_macro(result_value: Value, ok_type: String, err_type: String) -> Vec<Inst> {
        // try! macro is equivalent to ? operator
        Self::generate_question_mark_operator(result_value, ok_type, err_type)
    }
    
    /// Generate pattern matching for Result
    pub fn generate_result_match(result_value: Value, ok_arm: Vec<Inst>, err_arm: Vec<Inst>) -> Vec<Inst> {
        let mut instructions = vec![
            // Check Result discriminant
            Inst::EnumDiscriminant {
                result: Value::Reg(309),
                enum_ptr: result_value.clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(310),
                left: Value::Reg(309),
                right: Value::ImmInt(0),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(310),
                true_label: "match_result_ok".to_string(),
                false_label: "match_result_err".to_string(),
            },
            Inst::Label("match_result_ok".to_string()),
            // Extract Ok value
            Inst::EnumVariantData {
                result: Value::Reg(311),
                enum_ptr: result_value.clone(),
                variant_index: 0,
            },
        ];
        
        // Add Ok arm instructions
        instructions.extend(ok_arm);
        instructions.push(Inst::Jump("match_result_end".to_string()));
        
        instructions.push(Inst::Label("match_result_err".to_string()));
        // Extract Err value
        instructions.push(Inst::EnumVariantData {
            result: Value::Reg(312),
            enum_ptr: result_value,
            variant_index: 1,
        });
        
        // Add Err arm instructions
        instructions.extend(err_arm);
        instructions.push(Inst::Label("match_result_end".to_string()));
        
        instructions
    }
    
    /// Generate pattern matching for Option
    pub fn generate_option_match(option_value: Value, some_arm: Vec<Inst>, none_arm: Vec<Inst>) -> Vec<Inst> {
        let mut instructions = vec![
            // Check Option discriminant
            Inst::EnumDiscriminant {
                result: Value::Reg(313),
                enum_ptr: option_value.clone(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(314),
                left: Value::Reg(313),
                right: Value::ImmInt(1),
            },
            // Branch based on discriminant
            Inst::Branch {
                condition: Value::Reg(314),
                true_label: "match_option_some".to_string(),
                false_label: "match_option_none".to_string(),
            },
            Inst::Label("match_option_some".to_string()),
            // Extract Some value
            Inst::EnumVariantData {
                result: Value::Reg(315),
                enum_ptr: option_value.clone(),
                variant_index: 1,
            },
        ];
        
        // Add Some arm instructions
        instructions.extend(some_arm);
        instructions.push(Inst::Jump("match_option_end".to_string()));
        
        instructions.push(Inst::Label("match_option_none".to_string()));
        
        // Add None arm instructions
        instructions.extend(none_arm);
        instructions.push(Inst::Label("match_option_end".to_string()));
        
        instructions
    }
}