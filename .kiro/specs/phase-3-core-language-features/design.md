# Phase 3: Core Language Features - Design Document

## Overview

This design document outlines the implementation approach for Phase 3 of the Aero programming language, focusing on functions, control flow, basic I/O, and enhanced language features. The design builds upon the existing Phase 2 MVP compiler architecture while adding significant new capabilities.

## Architecture

### High-Level Architecture Changes

```
Source Code (.aero)
    ↓
Enhanced Lexer (new tokens: fn, if, while, for, loop, break, continue, print!, println!)
    ↓
Enhanced Parser (function definitions, control flow, I/O macros)
    ↓
Enhanced AST (Function, If, While, For, Loop, Break, Continue, Print nodes)
    ↓
Enhanced Semantic Analyzer (function table, scope management, control flow validation)
    ↓
Enhanced IR Generator (function calls, control flow, I/O operations)
    ↓
Enhanced Code Generator (LLVM function definitions, branches, loops, printf calls)
    ↓
Native Executable
```

### New Components

1. **Function Table**: Manages function definitions and signatures
2. **Scope Manager**: Handles nested scopes and variable shadowing
3. **Control Flow Analyzer**: Validates break/continue usage and unreachable code
4. **I/O Intrinsics**: Built-in print functions with format string support

## Components and Interfaces

### 1. Enhanced Lexer

**New Tokens:**
```rust
pub enum Token {
    // Existing tokens...
    
    // Function keywords
    Fn,
    Arrow,          // ->
    
    // Control flow keywords
    If,
    Else,
    While,
    For,
    In,
    Loop,
    Break,
    Continue,
    
    // I/O macros
    Print,          // print!
    Println,        // println!
    
    // Additional operators
    EqualEqual,     // ==
    NotEqual,       // !=
    LessEqual,      // <=
    GreaterEqual,   // >=
    AndAnd,         // &&
    OrOr,           // ||
    Bang,           // !
    
    // Mutability
    Mut,
    
    // Delimiters
    LeftBrace,      // {
    RightBrace,     // }
}
```

**Interface:**
```rust
impl Lexer {
    pub fn tokenize(source: &str) -> Vec<Token>;
    fn scan_keyword(word: &str) -> Option<Token>;
    fn scan_operator(chars: &mut PeekableChars) -> Option<Token>;
}
```

### 2. Enhanced AST

**New AST Nodes:**
```rust
#[derive(Debug, Clone)]
pub enum Statement {
    // Existing...
    Let { name: String, mutable: bool, type_annotation: Option<Type>, value: Option<Expression> },
    Expression(Expression),
    
    // New statements
    Function {
        name: String,
        parameters: Vec<Parameter>,
        return_type: Option<Type>,
        body: Block,
    },
    If {
        condition: Expression,
        then_block: Block,
        else_block: Option<Box<Statement>>,
    },
    While {
        condition: Expression,
        body: Block,
    },
    For {
        variable: String,
        iterable: Expression,
        body: Block,
    },
    Loop {
        body: Block,
    },
    Break,
    Continue,
    Return(Option<Expression>),
}

#[derive(Debug, Clone)]
pub enum Expression {
    // Existing...
    
    // New expressions
    FunctionCall {
        name: String,
        arguments: Vec<Expression>,
    },
    Print {
        format_string: String,
        arguments: Vec<Expression>,
    },
    Println {
        format_string: String,
        arguments: Vec<Expression>,
    },
    Comparison {
        op: ComparisonOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Logical {
        op: LogicalOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub expression: Option<Expression>,
}
```

### 3. Enhanced Semantic Analyzer

**Function Table:**
```rust
pub struct FunctionInfo {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
    pub defined_at: SourceLocation,
}

pub struct FunctionTable {
    functions: HashMap<String, FunctionInfo>,
}

impl FunctionTable {
    pub fn define_function(&mut self, info: FunctionInfo) -> Result<(), String>;
    pub fn get_function(&self, name: &str) -> Option<&FunctionInfo>;
    pub fn validate_call(&self, name: &str, args: &[Type]) -> Result<Type, String>;
}
```

**Scope Manager:**
```rust
pub struct ScopeManager {
    scopes: Vec<HashMap<String, VariableInfo>>,
    current_function: Option<String>,
    in_loop: u32,
}

impl ScopeManager {
    pub fn enter_scope(&mut self);
    pub fn exit_scope(&mut self);
    pub fn enter_function(&mut self, name: String);
    pub fn exit_function(&mut self);
    pub fn enter_loop(&mut self);
    pub fn exit_loop(&mut self);
    pub fn define_variable(&mut self, name: String, info: VariableInfo) -> Result<(), String>;
    pub fn get_variable(&self, name: &str) -> Option<&VariableInfo>;
    pub fn can_break_continue(&self) -> bool;
}
```

### 4. Enhanced IR

**New IR Instructions:**
```rust
#[derive(Debug, Clone)]
pub enum Instruction {
    // Existing...
    
    // Function operations
    FunctionDef {
        name: String,
        parameters: Vec<String>,
        body: Vec<Instruction>,
    },
    Call {
        function: String,
        arguments: Vec<Value>,
        result: Option<String>,
    },
    Return(Option<Value>),
    
    // Control flow
    Branch {
        condition: Value,
        true_label: String,
        false_label: String,
    },
    Jump(String),
    Label(String),
    
    // I/O operations
    Print {
        format: String,
        arguments: Vec<Value>,
    },
    
    // Comparisons
    Compare {
        op: ComparisonOp,
        left: Value,
        right: Value,
        result: String,
    },
}
```

### 5. Enhanced Code Generator

**LLVM Function Generation:**
```rust
impl CodeGenerator {
    fn generate_function(&mut self, name: &str, params: &[String], body: &[Instruction]) -> String;
    fn generate_function_call(&mut self, name: &str, args: &[Value]) -> String;
    fn generate_branch(&mut self, condition: &Value, true_label: &str, false_label: &str) -> String;
    fn generate_print(&mut self, format: &str, args: &[Value]) -> String;
    fn generate_comparison(&mut self, op: ComparisonOp, left: &Value, right: &Value) -> String;
}
```

## Data Models

### Function Signature Model
```rust
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub parameters: Vec<(String, Type)>,
    pub return_type: Type,
    pub is_builtin: bool,
}
```

### Control Flow Context
```rust
#[derive(Debug, Clone)]
pub struct ControlFlowContext {
    pub in_function: Option<String>,
    pub loop_depth: u32,
    pub break_labels: Vec<String>,
    pub continue_labels: Vec<String>,
}
```

### Variable Information
```rust
#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub name: String,
    pub var_type: Type,
    pub mutable: bool,
    pub initialized: bool,
    pub scope_level: u32,
}
```

## Error Handling

### Error Types
```rust
#[derive(Debug)]
pub enum CompilerError {
    // Existing errors...
    
    // Function errors
    FunctionRedefinition { name: String, location: SourceLocation },
    UndefinedFunction { name: String, location: SourceLocation },
    ArityMismatch { expected: usize, actual: usize, location: SourceLocation },
    ParameterTypeMismatch { expected: Type, actual: Type, location: SourceLocation },
    ReturnTypeMismatch { expected: Type, actual: Type, location: SourceLocation },
    
    // Control flow errors
    BreakOutsideLoop { location: SourceLocation },
    ContinueOutsideLoop { location: SourceLocation },
    UnreachableCode { location: SourceLocation },
    
    // Variable errors
    ImmutableAssignment { name: String, location: SourceLocation },
    VariableShadowing { name: String, location: SourceLocation },
    
    // I/O errors
    InvalidFormatString { format: String, location: SourceLocation },
    FormatArgumentMismatch { expected: usize, actual: usize, location: SourceLocation },
}
```

### Error Recovery Strategy
1. **Function-level recovery**: Skip to next function definition on function parsing errors
2. **Statement-level recovery**: Skip to next statement on control flow errors
3. **Expression-level recovery**: Use error expressions for invalid expressions
4. **Scope recovery**: Properly unwind scopes on errors

## Testing Strategy

### Unit Tests
1. **Lexer Tests**: New token recognition, keyword vs identifier disambiguation
2. **Parser Tests**: Function definitions, control flow parsing, I/O macro parsing
3. **Semantic Tests**: Function resolution, scope management, type checking
4. **IR Tests**: Function call generation, control flow IR, I/O operations
5. **Code Generation Tests**: LLVM function definitions, branch generation

### Integration Tests
1. **Simple Functions**: Parameter passing, return values, recursion
2. **Control Flow**: Nested if/else, various loop types, break/continue
3. **I/O Operations**: Print formatting, multiple arguments, type formatting
4. **Scoping**: Variable shadowing, function-local variables, nested scopes
5. **Error Cases**: All error conditions with expected error messages

### End-to-End Tests
```aero
// Test: fibonacci.aero
fn fib(n: i32) -> i32 {
    if n <= 1 {
        return n;
    } else {
        return fib(n - 1) + fib(n - 2);
    }
}

fn main() {
    let result = fib(10);
    println!("Fibonacci(10) = {}", result);
}
```

```aero
// Test: loops.aero
fn main() {
    let mut i = 0;
    while i < 5 {
        println!("Count: {}", i);
        i = i + 1;
    }
    
    for j in 0..3 {
        println!("Loop: {}", j);
    }
}
```

### Performance Tests
1. **Function Call Overhead**: Measure call/return performance
2. **Loop Performance**: Compare loop types for performance
3. **I/O Performance**: Measure print operation overhead
4. **Compilation Speed**: Track compilation time for larger programs

## Implementation Phases

### Phase 3.1: Function Definitions and Calls
- Lexer support for `fn`, `->`, parameter lists
- Parser support for function definitions
- AST nodes for functions and calls
- Basic semantic analysis for functions
- IR generation for function definitions and calls
- LLVM code generation for functions

### Phase 3.2: Control Flow
- Lexer support for control flow keywords
- Parser support for if/else, loops
- AST nodes for control flow constructs
- Semantic analysis for control flow validation
- IR generation for branches and loops
- LLVM code generation for control flow

### Phase 3.3: Basic I/O
- Lexer support for print macros
- Parser support for format strings
- AST nodes for I/O operations
- Semantic analysis for format string validation
- IR generation for I/O operations
- LLVM code generation with printf integration

### Phase 3.4: Enhanced Variables and Types
- Lexer support for `mut`, comparison operators
- Parser support for mutability and new operators
- Enhanced semantic analysis for mutability
- IR generation for new operations
- LLVM code generation for comparisons and assignments

### Phase 3.5: Error Reporting and Polish
- Enhanced error messages with source locations
- Error recovery improvements
- Comprehensive testing
- Documentation updates
- Performance optimizations