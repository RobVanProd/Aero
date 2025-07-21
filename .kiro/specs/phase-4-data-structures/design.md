# Phase 4: Data Structures & Advanced Types - Design Document

## Overview

This design document outlines the implementation approach for Phase 4 of the Aero programming language, focusing on data structures, advanced types, pattern matching, and enhanced string operations. This phase transforms Aero from a procedural language into a modern systems programming language with rich data modeling capabilities.

## Architecture

### High-Level Architecture Changes

```
Source Code (.aero)
    ↓
Enhanced Lexer (struct, enum, match, impl, Vec, String operations)
    ↓
Enhanced Parser (struct/enum definitions, pattern matching, generic syntax)
    ↓
Enhanced AST (Struct, Enum, Match, Pattern, Generic nodes)
    ↓
Enhanced Semantic Analyzer (type definitions, pattern exhaustiveness, generic resolution)
    ↓
Enhanced IR Generator (struct operations, pattern matching, generic instantiation)
    ↓
Enhanced Code Generator (LLVM struct types, switch statements, generic monomorphization)
    ↓
Native Executable with Rich Data Types
```

### New Components

1. **Type Definition Manager**: Handles struct and enum definitions
2. **Pattern Matcher**: Analyzes pattern completeness and generates match code
3. **Generic Resolver**: Handles generic type instantiation and monomorphization
4. **Memory Layout Calculator**: Determines optimal struct layouts
5. **Collection Library**: Built-in Vec, HashMap, and other collections

## Components and Interfaces

### 1. Enhanced Lexer

**New Tokens:**
```rust
pub enum Token {
    // Existing tokens...
    
    // Data structure keywords
    Struct,
    Enum,
    Impl,
    Match,
    
    // Pattern matching
    Underscore,     // _
    DotDot,         // ..
    DotDotEqual,    // ..=
    Pipe,           // |
    At,             // @
    
    // Generics
    LeftAngle,      // <
    RightAngle,     // >
    
    // Collections
    Vec,
    HashMap,
    
    // String operations
    Format,         // format!
    
    // Advanced operators
    Question,       // ?
    DoubleColon,    // ::
}
```

### 2. Enhanced AST

**New AST Nodes:**
```rust
#[derive(Debug, Clone)]
pub enum Statement {
    // Existing...
    
    // Data structure definitions
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
    Impl {
        generics: Vec<String>,
        type_name: String,
        trait_name: Option<String>,
        methods: Vec<Function>,
    },
}

#[derive(Debug, Clone)]
pub enum Expression {
    // Existing...
    
    // Data structure operations
    StructLiteral {
        name: String,
        fields: Vec<(String, Expression)>,
        base: Option<Box<Expression>>,
    },
    FieldAccess {
        object: Box<Expression>,
        field: String,
    },
    MethodCall {
        object: Box<Expression>,
        method: String,
        arguments: Vec<Expression>,
    },
    Match {
        expression: Box<Expression>,
        arms: Vec<MatchArm>,
    },
    ArrayLiteral {
        elements: Vec<Expression>,
    },
    ArrayAccess {
        array: Box<Expression>,
        index: Box<Expression>,
    },
    VecMacro {
        elements: Vec<Expression>,
    },
    FormatMacro {
        format_string: String,
        arguments: Vec<Expression>,
    },
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub field_type: Type,
    pub visibility: Visibility,
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
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub body: Expression,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard,
    Identifier(String),
    Literal(Literal),
    Tuple(Vec<Pattern>),
    Struct {
        name: String,
        fields: Vec<(String, Pattern)>,
        rest: bool,
    },
    Enum {
        variant: String,
        data: Option<Box<Pattern>>,
    },
    Range {
        start: Box<Pattern>,
        end: Box<Pattern>,
        inclusive: bool,
    },
    Or(Vec<Pattern>),
    Binding {
        name: String,
        pattern: Box<Pattern>,
    },
}
```

### 3. Type Definition Manager

```rust
pub struct TypeDefinitionManager {
    structs: HashMap<String, StructDefinition>,
    enums: HashMap<String, EnumDefinition>,
    impls: HashMap<String, Vec<ImplBlock>>,
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
    pub discriminant_type: Type,
}

impl TypeDefinitionManager {
    pub fn define_struct(&mut self, def: StructDefinition) -> Result<(), String>;
    pub fn define_enum(&mut self, def: EnumDefinition) -> Result<(), String>;
    pub fn get_struct(&self, name: &str) -> Option<&StructDefinition>;
    pub fn get_enum(&self, name: &str) -> Option<&EnumDefinition>;
    pub fn validate_field_access(&self, type_name: &str, field: &str) -> Result<Type, String>;
    pub fn get_method(&self, type_name: &str, method: &str) -> Option<&Function>;
}
```

### 4. Pattern Matcher

```rust
pub struct PatternMatcher {
    type_manager: Arc<TypeDefinitionManager>,
}

impl PatternMatcher {
    pub fn check_exhaustiveness(&self, patterns: &[Pattern], match_type: &Type) -> Result<(), String>;
    pub fn compile_pattern(&self, pattern: &Pattern, target_type: &Type) -> Result<PatternCode, String>;
    pub fn extract_bindings(&self, pattern: &Pattern) -> Vec<(String, Type)>;
    pub fn is_irrefutable(&self, pattern: &Pattern) -> bool;
}

#[derive(Debug)]
pub struct PatternCode {
    pub conditions: Vec<Condition>,
    pub bindings: Vec<Binding>,
}
```

### 5. Generic Resolver

```rust
pub struct GenericResolver {
    instantiations: HashMap<String, Vec<GenericInstance>>,
}

#[derive(Debug, Clone)]
pub struct GenericInstance {
    pub base_name: String,
    pub type_args: Vec<Type>,
    pub instantiated_name: String,
}

impl GenericResolver {
    pub fn instantiate_generic(&mut self, base: &str, args: &[Type]) -> Result<String, String>;
    pub fn resolve_generic_method(&self, type_name: &str, method: &str, args: &[Type]) -> Result<String, String>;
    pub fn monomorphize(&self, generic_def: &GenericDefinition, args: &[Type]) -> Result<ConcreteDefinition, String>;
}
```

### 6. Memory Layout Calculator

```rust
pub struct MemoryLayoutCalculator;

#[derive(Debug, Clone)]
pub struct MemoryLayout {
    pub size: usize,
    pub alignment: usize,
    pub field_offsets: Vec<usize>,
}

impl MemoryLayoutCalculator {
    pub fn calculate_struct_layout(&self, fields: &[StructField]) -> MemoryLayout;
    pub fn calculate_enum_layout(&self, variants: &[EnumVariant]) -> MemoryLayout;
    pub fn optimize_field_order(&self, fields: &[StructField]) -> Vec<usize>;
}
```

## Data Models

### Type System Extensions

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Existing primitive types...
    
    // Compound types
    Struct {
        name: String,
        generics: Vec<Type>,
    },
    Enum {
        name: String,
        generics: Vec<Type>,
    },
    Array {
        element_type: Box<Type>,
        size: Option<usize>,
    },
    Slice {
        element_type: Box<Type>,
    },
    Vec {
        element_type: Box<Type>,
    },
    HashMap {
        key_type: Box<Type>,
        value_type: Box<Type>,
    },
    
    // Generic types
    Generic(String),
    
    // Special types
    Never,  // ! type for diverging functions
}
```

### Collection Types

```rust
pub struct VecType {
    pub element_type: Type,
    pub capacity: Option<usize>,
}

pub struct HashMapType {
    pub key_type: Type,
    pub value_type: Type,
}

pub struct ArrayType {
    pub element_type: Type,
    pub size: usize,
}
```

## Error Handling

### Enhanced Error Types

```rust
#[derive(Debug)]
pub enum CompilerError {
    // Existing errors...
    
    // Data structure errors
    StructRedefinition { name: String, location: SourceLocation },
    UndefinedStruct { name: String, location: SourceLocation },
    FieldNotFound { struct_name: String, field: String, location: SourceLocation },
    FieldTypeMismatch { expected: Type, actual: Type, location: SourceLocation },
    
    // Enum errors
    EnumRedefinition { name: String, location: SourceLocation },
    UndefinedEnum { name: String, location: SourceLocation },
    VariantNotFound { enum_name: String, variant: String, location: SourceLocation },
    
    // Pattern matching errors
    NonExhaustiveMatch { missing_patterns: Vec<String>, location: SourceLocation },
    UnreachablePattern { pattern: String, location: SourceLocation },
    PatternTypeMismatch { expected: Type, actual: Type, location: SourceLocation },
    
    // Generic errors
    GenericArityMismatch { expected: usize, actual: usize, location: SourceLocation },
    GenericConstraintViolation { constraint: String, location: SourceLocation },
    
    // Collection errors
    IndexOutOfBounds { index: usize, size: usize, location: SourceLocation },
    InvalidSliceRange { start: usize, end: usize, location: SourceLocation },
}
```

## Testing Strategy

### Unit Tests

1. **Type Definition Tests**: Struct/enum parsing, validation, and storage
2. **Pattern Matching Tests**: Exhaustiveness checking, pattern compilation
3. **Generic Resolution Tests**: Type instantiation, monomorphization
4. **Memory Layout Tests**: Size calculation, alignment, optimization
5. **Collection Tests**: Array/Vec operations, bounds checking

### Integration Tests

1. **Data Structure Usage**: Complete struct/enum definition and usage
2. **Pattern Matching**: Complex match expressions with all pattern types
3. **Generic Programming**: Generic structs, enums, and functions
4. **Collection Operations**: Array/Vec manipulation, iteration
5. **String Processing**: Advanced string operations and formatting

### End-to-End Tests

```aero
// Test: data_structures.aero
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }
    
    fn distance(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

enum Shape {
    Circle { center: Point, radius: f64 },
    Rectangle { top_left: Point, bottom_right: Point },
}

fn area(shape: &Shape) -> f64 {
    match shape {
        Shape::Circle { radius, .. } => 3.14159 * radius * radius,
        Shape::Rectangle { top_left, bottom_right } => {
            let width = bottom_right.x - top_left.x;
            let height = bottom_right.y - top_left.y;
            width * height
        }
    }
}

fn main() {
    let circle = Shape::Circle {
        center: Point::new(0.0, 0.0),
        radius: 5.0,
    };
    
    let rect = Shape::Rectangle {
        top_left: Point::new(0.0, 0.0),
        bottom_right: Point::new(10.0, 5.0),
    };
    
    println!("Circle area: {}", area(&circle));
    println!("Rectangle area: {}", area(&rect));
}
```

```aero
// Test: collections.aero
fn main() {
    let mut numbers = vec![1, 2, 3, 4, 5];
    numbers.push(6);
    
    for (i, num) in numbers.iter().enumerate() {
        println!("numbers[{}] = {}", i, num);
    }
    
    let filtered: Vec<i32> = numbers
        .iter()
        .filter(|&x| x % 2 == 0)
        .map(|&x| x * 2)
        .collect();
    
    println!("Filtered and doubled: {:?}", filtered);
}
```

## Implementation Phases

### Phase 4.1: Basic Data Structures
- Struct definition parsing and validation
- Basic field access and modification
- Simple struct instantiation
- Memory layout calculation

### Phase 4.2: Enums and Pattern Matching
- Enum definition parsing
- Basic pattern matching with match expressions
- Pattern exhaustiveness checking
- Enum variant construction and access

### Phase 4.3: Advanced Patterns
- Complex pattern types (ranges, guards, bindings)
- Nested pattern destructuring
- Pattern compilation optimization
- Irrefutable patterns in let statements

### Phase 4.4: Generic Data Structures
- Generic struct and enum definitions
- Type parameter resolution
- Generic method implementations
- Monomorphization for code generation

### Phase 4.5: Collections and Arrays
- Fixed-size array support
- Dynamic Vec implementation
- Array/slice operations and bounds checking
- Collection iteration and methods

### Phase 4.6: Enhanced String Operations
- String concatenation and formatting
- String slicing with UTF-8 safety
- String comparison and manipulation methods
- format! macro implementation

### Phase 4.7: Method Implementations
- impl blocks for structs and enums
- Method resolution and dispatch
- Associated functions and methods
- Trait implementations (basic)

### Phase 4.8: Memory Management Integration
- Ownership rules for data structures
- Borrowing and references for struct fields
- Move semantics for large structures
- Drop implementations for cleanup