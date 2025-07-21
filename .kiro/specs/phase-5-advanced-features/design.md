# Phase 5: Advanced Language Features - Design Document

## Overview

This design document outlines the implementation approach for Phase 5 of the Aero programming language, focusing on advanced features that will complete Aero's transformation into a full-featured systems programming language. Phase 5 introduces traits, advanced generics, modules, ownership/borrowing, memory management, and concurrency primitives.

## Architecture

### High-Level Architecture Changes

```
Source Code (.aero files in module hierarchy)
    ↓
Enhanced Lexer (trait, impl, mod, pub, use, unsafe, Box, Rc, Arc, async)
    ↓
Enhanced Parser (trait definitions, module declarations, lifetime syntax, unsafe blocks)
    ↓
Enhanced AST (Trait, Impl, Module, Lifetime, Unsafe, Concurrency nodes)
    ↓
Enhanced Semantic Analyzer (trait resolution, borrow checker, lifetime analysis, module resolution)
    ↓
Enhanced IR Generator (trait dispatch, ownership tracking, memory management, concurrency)
    ↓
Enhanced Code Generator (vtables, monomorphization, memory safety, thread safety)
    ↓
Optimized Native Executable with Advanced Features
```

### New Major Components

1. **Trait System**: Manages trait definitions, implementations, and dispatch
2. **Borrow Checker**: Enforces ownership and borrowing rules
3. **Lifetime Analyzer**: Tracks reference lifetimes and prevents dangling pointers
4. **Module Resolver**: Handles module hierarchy and visibility
5. **Memory Manager**: Manages heap allocation and smart pointers
6. **Concurrency Analyzer**: Ensures thread safety and prevents data races
7. **Macro Processor**: Handles macro expansion and hygiene
8. **Optimization Engine**: Performs advanced optimizations

## Components and Interfaces

### 1. Enhanced Lexer

**New Tokens:**
```rust
pub enum Token {
    // Existing tokens...
    
    // Trait system
    Trait,
    Impl,
    For,
    Dyn,
    Where,
    
    // Module system
    Mod,
    Pub,
    Use,
    As,
    Super,
    Crate,
    Self_,
    
    // Ownership and borrowing
    Ampersand,      // & (already exists, enhanced)
    Move,
    Copy,
    Clone,
    Drop,
    
    // Lifetimes
    Lifetime(String), // 'a, 'static, etc.
    
    // Memory management
    Box,
    Rc,
    Arc,
    Unsafe,
    
    // Concurrency
    Async,
    Await,
    Send,
    Sync,
    Mutex,
    Channel,
    
    // Macros
    MacroRules,     // macro_rules!
    Dollar,         // $
    
    // Advanced operators
    Turbofish,      // ::< for generic disambiguation
}
```

### 2. Enhanced AST

**New AST Nodes:**
```rust
#[derive(Debug, Clone)]
pub enum Statement {
    // Existing...
    
    // Trait system
    Trait {
        name: String,
        generics: Vec<GenericParam>,
        supertraits: Vec<TraitBound>,
        items: Vec<TraitItem>,
    },
    Impl {
        generics: Vec<GenericParam>,
        trait_ref: Option<TraitRef>,
        self_type: Type,
        items: Vec<ImplItem>,
        unsafe_: bool,
    },
    
    // Module system
    Module {
        name: String,
        items: Vec<Statement>,
        inline: bool,
    },
    Use {
        path: UsePath,
        alias: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub enum Expression {
    // Existing...
    
    // Advanced expressions
    Unsafe {
        block: Block,
    },
    Async {
        block: Block,
    },
    Await {
        expression: Box<Expression>,
    },
    Box {
        expression: Box<Expression>,
    },
    MacroCall {
        name: String,
        tokens: Vec<Token>,
    },
}

#[derive(Debug, Clone)]
pub struct TraitItem {
    pub name: String,
    pub kind: TraitItemKind,
    pub default_impl: Option<Block>,
}

#[derive(Debug, Clone)]
pub enum TraitItemKind {
    Method {
        signature: FunctionSignature,
    },
    AssociatedType {
        bounds: Vec<TraitBound>,
        default: Option<Type>,
    },
    AssociatedConst {
        type_: Type,
        default: Option<Expression>,
    },
}

#[derive(Debug, Clone)]
pub struct Lifetime {
    pub name: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum Type {
    // Existing...
    
    // Advanced types
    TraitObject {
        traits: Vec<TraitBound>,
        lifetime: Option<Lifetime>,
    },
    Reference {
        lifetime: Option<Lifetime>,
        mutable: bool,
        type_: Box<Type>,
    },
    RawPointer {
        mutable: bool,
        type_: Box<Type>,
    },
    Never, // !
}
```

### 3. Trait System

```rust
pub struct TraitSystem {
    traits: HashMap<String, TraitDefinition>,
    implementations: HashMap<(String, String), ImplDefinition>, // (trait, type)
    coherence_checker: CoherenceChecker,
}

#[derive(Debug, Clone)]
pub struct TraitDefinition {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub supertraits: Vec<TraitBound>,
    pub items: Vec<TraitItem>,
    pub object_safe: bool,
}

#[derive(Debug, Clone)]
pub struct ImplDefinition {
    pub trait_ref: Option<TraitRef>,
    pub self_type: Type,
    pub generics: Vec<GenericParam>,
    pub items: Vec<ImplItem>,
    pub where_clause: Vec<WherePredicate>,
}

impl TraitSystem {
    pub fn define_trait(&mut self, trait_def: TraitDefinition) -> Result<(), TraitError>;
    pub fn define_impl(&mut self, impl_def: ImplDefinition) -> Result<(), TraitError>;
    pub fn resolve_method(&self, receiver_type: &Type, method: &str) -> Result<MethodResolution, TraitError>;
    pub fn check_trait_bounds(&self, type_: &Type, bounds: &[TraitBound]) -> Result<(), TraitError>;
    pub fn create_vtable(&self, trait_ref: &TraitRef, concrete_type: &Type) -> Result<VTable, TraitError>;
}
```

### 4. Borrow Checker

```rust
pub struct BorrowChecker {
    loans: HashMap<NodeId, Loan>,
    move_data: MoveData,
    region_inference: RegionInference,
}

#[derive(Debug, Clone)]
pub struct Loan {
    pub kind: LoanKind,
    pub lifetime: Lifetime,
    pub borrowed_place: Place,
    pub location: Location,
}

#[derive(Debug, Clone)]
pub enum LoanKind {
    Shared,
    Mutable,
    Move,
}

#[derive(Debug, Clone)]
pub struct Place {
    pub base: PlaceBase,
    pub projections: Vec<Projection>,
}

impl BorrowChecker {
    pub fn check_function(&mut self, function: &Function) -> Result<(), BorrowError>;
    pub fn check_move(&mut self, place: &Place, location: Location) -> Result<(), BorrowError>;
    pub fn check_borrow(&mut self, place: &Place, kind: LoanKind, location: Location) -> Result<(), BorrowError>;
    pub fn check_assignment(&mut self, place: &Place, value: &Expression, location: Location) -> Result<(), BorrowError>;
}
```

### 5. Module System

```rust
pub struct ModuleSystem {
    modules: HashMap<ModulePath, Module>,
    imports: HashMap<ModulePath, Vec<Import>>,
    visibility_resolver: VisibilityResolver,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub path: ModulePath,
    pub items: HashMap<String, Item>,
    pub submodules: HashMap<String, ModulePath>,
    pub parent: Option<ModulePath>,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub path: UsePath,
    pub alias: Option<String>,
    pub visibility: Visibility,
}

impl ModuleSystem {
    pub fn define_module(&mut self, path: ModulePath, module: Module) -> Result<(), ModuleError>;
    pub fn resolve_path(&self, path: &Path, current_module: &ModulePath) -> Result<Item, ModuleError>;
    pub fn check_visibility(&self, item: &Item, access_location: &ModulePath) -> Result<(), ModuleError>;
    pub fn process_imports(&mut self, module: &ModulePath) -> Result<(), ModuleError>;
}
```

### 6. Memory Management

```rust
pub struct MemoryManager {
    allocators: HashMap<String, Box<dyn Allocator>>,
    smart_pointers: SmartPointerRegistry,
    drop_checker: DropChecker,
}

#[derive(Debug, Clone)]
pub enum SmartPointer {
    Box(Type),
    Rc(Type),
    Arc(Type),
    RefCell(Type),
    Mutex(Type),
}

impl MemoryManager {
    pub fn allocate_box(&self, type_: &Type, value: Value) -> Result<BoxValue, MemoryError>;
    pub fn create_rc(&self, type_: &Type, value: Value) -> Result<RcValue, MemoryError>;
    pub fn create_arc(&self, type_: &Type, value: Value) -> Result<ArcValue, MemoryError>;
    pub fn check_drop_order(&self, items: &[Item]) -> Result<(), MemoryError>;
}
```

## Data Models

### Trait Resolution

```rust
#[derive(Debug, Clone)]
pub struct MethodResolution {
    pub method: Method,
    pub dispatch_kind: DispatchKind,
    pub substitutions: HashMap<String, Type>,
}

#[derive(Debug, Clone)]
pub enum DispatchKind {
    Static,
    Dynamic(VTable),
}

#[derive(Debug, Clone)]
pub struct VTable {
    pub trait_ref: TraitRef,
    pub methods: Vec<MethodEntry>,
    pub drop_fn: Option<DropFn>,
}
```

### Ownership Tracking

```rust
#[derive(Debug, Clone)]
pub struct OwnershipInfo {
    pub owner: Option<Place>,
    pub borrows: Vec<Borrow>,
    pub moved: bool,
    pub drop_flag: bool,
}

#[derive(Debug, Clone)]
pub struct Borrow {
    pub kind: BorrowKind,
    pub lifetime: Lifetime,
    pub place: Place,
}
```

### Lifetime Analysis

```rust
#[derive(Debug, Clone)]
pub struct LifetimeConstraints {
    pub outlives: Vec<(Lifetime, Lifetime)>,
    pub bounds: Vec<(Type, Lifetime)>,
    pub inference_vars: Vec<LifetimeVar>,
}

#[derive(Debug, Clone)]
pub struct RegionInference {
    pub constraints: LifetimeConstraints,
    pub solutions: HashMap<LifetimeVar, Lifetime>,
}
```

## Error Handling

### Advanced Error Types

```rust
#[derive(Debug)]
pub enum CompilerError {
    // Existing errors...
    
    // Trait system errors
    TraitNotFound { name: String, location: SourceLocation },
    TraitNotImplemented { trait_name: String, type_name: String, location: SourceLocation },
    AmbiguousMethod { method: String, candidates: Vec<String>, location: SourceLocation },
    CoherenceViolation { trait_name: String, type_name: String, location: SourceLocation },
    ObjectSafetyViolation { trait_name: String, reason: String, location: SourceLocation },
    
    // Borrow checker errors
    UseAfterMove { place: String, move_location: SourceLocation, use_location: SourceLocation },
    BorrowConflict { kind: String, place: String, locations: Vec<SourceLocation> },
    LifetimeTooShort { lifetime: String, required: String, location: SourceLocation },
    DanglingReference { place: String, location: SourceLocation },
    
    // Module system errors
    ModuleNotFound { path: String, location: SourceLocation },
    PrivateAccess { item: String, module: String, location: SourceLocation },
    CircularDependency { modules: Vec<String>, location: SourceLocation },
    AmbiguousImport { name: String, sources: Vec<String>, location: SourceLocation },
    
    // Memory management errors
    AllocationFailure { type_name: String, size: usize, location: SourceLocation },
    DoubleFree { place: String, location: SourceLocation },
    MemoryLeak { places: Vec<String>, location: SourceLocation },
    
    // Concurrency errors
    DataRace { place: String, locations: Vec<SourceLocation> },
    DeadlockPotential { resources: Vec<String>, location: SourceLocation },
    SendSyncViolation { type_name: String, trait_name: String, location: SourceLocation },
}
```

## Testing Strategy

### Unit Tests

1. **Trait System Tests**: Trait definition, implementation, resolution, dispatch
2. **Borrow Checker Tests**: Move semantics, borrowing rules, lifetime validation
3. **Module System Tests**: Module resolution, visibility, import handling
4. **Memory Management Tests**: Smart pointer operations, drop checking
5. **Concurrency Tests**: Thread safety, data race detection

### Integration Tests

1. **Advanced Polymorphism**: Complex trait hierarchies and generic constraints
2. **Memory Safety**: Comprehensive ownership and borrowing scenarios
3. **Module Organization**: Large-scale code organization and visibility
4. **Performance**: Zero-cost abstractions and optimization validation
5. **Concurrency Safety**: Multi-threaded programs with shared data

### End-to-End Tests

```aero
// Test: advanced_traits.aero
trait Display {
    fn fmt(&self) -> String;
}

trait Debug: Display {
    fn debug_fmt(&self) -> String {
        format!("Debug: {}", self.fmt())
    }
}

struct Point {
    x: f64,
    y: f64,
}

impl Display for Point {
    fn fmt(&self) -> String {
        format!("({}, {})", self.x, self.y)
    }
}

impl Debug for Point {}

fn print_debug<T: Debug>(item: &T) {
    println!("{}", item.debug_fmt());
}

fn main() {
    let p = Point { x: 1.0, y: 2.0 };
    print_debug(&p);
}
```

```aero
// Test: ownership_borrowing.aero
struct Container<T> {
    data: Vec<T>,
}

impl<T> Container<T> {
    fn new() -> Self {
        Container { data: Vec::new() }
    }
    
    fn add(&mut self, item: T) {
        self.data.push(item);
    }
    
    fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }
    
    fn take(&mut self, index: usize) -> Option<T> {
        if index < self.data.len() {
            Some(self.data.remove(index))
        } else {
            None
        }
    }
}

fn main() {
    let mut container = Container::new();
    container.add(String::from("hello"));
    container.add(String::from("world"));
    
    if let Some(item) = container.get(0) {
        println!("First item: {}", item);
    }
    
    if let Some(taken) = container.take(0) {
        println!("Took: {}", taken);
    }
}
```

## Implementation Phases

### Phase 5.1: Basic Trait System
- Trait definition parsing and validation
- Simple trait implementations
- Static method dispatch
- Basic trait bounds

### Phase 5.2: Advanced Traits
- Associated types and constants
- Default implementations
- Trait objects and dynamic dispatch
- Coherence checking

### Phase 5.3: Ownership and Borrowing
- Move semantics implementation
- Basic borrow checking
- Reference lifetime tracking
- Simple lifetime inference

### Phase 5.4: Advanced Lifetimes
- Explicit lifetime parameters
- Lifetime bounds and constraints
- Higher-ranked trait bounds
- Complex lifetime inference

### Phase 5.5: Module System
- Module definition and hierarchy
- Visibility and access control
- Import and export mechanisms
- Path resolution

### Phase 5.6: Memory Management
- Smart pointer implementations (Box, Rc, Arc)
- Drop trait and destructors
- Unsafe code blocks
- Memory leak detection

### Phase 5.7: Basic Concurrency
- Thread spawning and management
- Send and Sync traits
- Basic synchronization primitives
- Data race detection

### Phase 5.8: Advanced Features
- Basic macro system
- Const evaluation
- Performance optimizations
- Advanced error handling

## Performance Considerations

### Zero-Cost Abstractions
- Trait method calls should compile to direct function calls when possible
- Generic instantiation should not introduce runtime overhead
- Smart pointers should optimize to raw pointers when safe

### Memory Efficiency
- Struct layout optimization
- Enum discriminant optimization
- String interning for common strings
- Compile-time constant folding

### Compilation Performance
- Incremental compilation support
- Parallel trait resolution
- Efficient generic instantiation caching
- Fast module dependency analysis