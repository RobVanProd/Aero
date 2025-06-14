# Aero Language Specification - Type System Rules

## Introduction

This document formally specifies the static type system rules for the Aero programming language. It defines the built-in types, rules for type inference, and the syntax and behavior of generic type parameters and trait bounds. A strong static type system is crucial for Aero's goals of performance and reliability.

## Built-in Types

Aero provides a set of fundamental built-in types:

- `int`: Signed 64-bit integers.
- `float`: 64-bit floating-point numbers (IEEE 754 double-precision).
- `bool`: Boolean values, `true` or `false`.
- `string`: UTF-8 encoded immutable text sequences.
- `char`: Unicode scalar values.
- `unit`: Represents the absence of a value, similar to `void` in C/C++ or `()` in Rust. Used as the return type for functions that do not explicitly return a value.

## Type Inference

Aero supports local type inference, reducing the need for explicit type annotations. The compiler will infer types based on variable initialization, function return types, and usage context. However, explicit type annotations are required in certain cases, such as function parameters and public API definitions, to ensure clarity and stability.

### Rules for Type Inference:

1. **Variable Initialization**: If a variable is initialized with a literal or an expression whose type can be determined, the variable's type will be inferred from it.
   ```aero
   let x = 10; // x is inferred as int
   let y = 3.14; // y is inferred as float
   let z = "hello"; // z is inferred as string
   ```

2. **Function Return Types**: The return type of a function can often be inferred from the type of the expression in the `return` statement. However, explicit return type annotations are generally encouraged for public functions.

3. **Contextual Typing**: In expressions, the expected type can propagate downwards to infer the types of sub-expressions.

## Generic Type Parameters

Aero supports generic programming through type parameters, allowing the creation of flexible and reusable code that works with various types while maintaining type safety.

### Syntax:

Generic type parameters are declared within angle brackets (`<>`) after the name of a `struct`, `enum`, `fn`, or `trait`.

```aero
struct Option<T> {
    // ...
}

enum Result<T, E> {
    // ...
}

fn identity<T>(value: T) -> T {
    value
}
```

### Usage:

When using a generic type or function, concrete types are provided for the type parameters.

```aero
let opt_int: Option<int> = /* ... */ ;
let res_str_err: Result<string, MyError> = /* ... */ ;
let val = identity(123); // T is inferred as int
```

## Trait Bounds

Trait bounds are used to constrain generic type parameters, ensuring that only types implementing specific traits can be used. This enables polymorphism and allows generic functions to call methods defined by the bounded traits.

### Syntax:

Trait bounds are specified using the `trait_name` syntax after the type parameter, often with a `where` clause for multiple bounds or complex constraints.

```aero
fn print_and_add<T: Display + Add>(a: T, b: T) {
    // ...
}

struct Container<T> where T: Clone + Debug {
    // ...
}
```

### Rules for Trait Bounds:

1. **Compile-time Enforcement**: The compiler enforces trait bounds at compile time, ensuring that all operations on generic types are valid for the concrete types that will eventually replace them.

2. **Method Dispatch**: When a method is called on a generic type `T` with a trait bound `TraitA`, the compiler ensures that `T` implements `TraitA` and dispatches to the appropriate method implementation.

## Type System Error Conditions

The Aero compiler will detect and report the following type-related error conditions:

- **Type Mismatch**: Attempting to assign a value of one type to a variable of an incompatible type, or passing an argument of an incorrect type to a function.
- **Undefined Type/Variable**: Referencing a type or variable that has not been declared.
- **Incorrect Arity**: Calling a function with an incorrect number of arguments.
- **Unsatisfied Trait Bounds**: Using a generic type with a concrete type that does not satisfy its specified trait bounds.

## Future Considerations:

- More complex type inference scenarios (e.g., closures).
- Variance and subtyping rules.
- Advanced type system features (e.g., associated types, higher-kinded types).


