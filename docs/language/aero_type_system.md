# Aero Language Specification - Type System Rules

## Introduction

This document formally specifies the static type system rules for the Aero programming language. It defines the built-in types, rules for type inference, and the syntax and behavior of generic type parameters and trait bounds. A strong static type system is crucial for Aero's goals of performance and reliability.

## Built-in Types

Aero provides a set of fundamental built-in types. These types are the basic building blocks for all other types in the language.

### Integer Types
Aero may offer a range of integer types to allow developers to choose the appropriate size and signedness for their data. For example:
-   **Signed Integers:** `i8`, `i16`, `i32`, `i64` (default for `int` if not otherwise specified), `i128`.
    *   `i64`: Represents signed integers from -(2<sup>63</sup>) to 2<sup>63</sup> - 1.
-   **Unsigned Integers:** `u8`, `u16`, `u32`, `u64`, `u128`.
    *   `u64`: Represents unsigned integers from 0 to 2<sup>64</sup> - 1.
-   **Architectural Integers:** `isize`, `usize` (pointer-sized signed/unsigned integers, their size depends on the target architecture, e.g., 32-bit or 64-bit).

If not specified, a generic `int` might default to `i64`. Integer literals can be type-annotated (e.g., `10u32`, `200i8`).

### Floating-Point Types
-   `f32`: Single-precision floating-point numbers (IEEE 754).
-   `f64`: Double-precision floating-point numbers (IEEE 754) (default for `float` if not otherwise specified).
    Example: `let pi: f32 = 3.14159; let gravity = 9.81f64;`

### Boolean Type
-   `bool`: Represents boolean values. Can only be `true` or `false`.
    Example: `let is_active = true; let has_permission = false;`

### Character Type
-   `char`: Represents a single Unicode scalar value (e.g., 'a', '🚀', '中'). Typically a 4-byte UTF-32 representation.
    Example: `let initial = 'J'; let emoji = '😊';`

### String Types
Aero distinguishes between owned strings and string slices (borrows):
-   `String`: An owned, heap-allocated, growable, mutable sequence of UTF-8 encoded characters. Similar to `std::string` in C++ or `String` in Rust.
    Example: `let mut greeting = String::from("Hello"); greeting.append(", Aero!");`
-   `&str` (string slice/reference): An immutable reference to a sequence of UTF-8 encoded characters, typically borrowed from a `String` or a string literal. String literals themselves are of type `&str`.
    Example: `let literal_slice = "immutable text"; let borrowed_slice = &greeting[0..5];`

### Unit Type
-   `unit` (often represented as `()`): Represents the absence of a specific value.
    *   It is used as the return type for functions that do not explicitly return a value.
    *   It can also be used as a placeholder in generic types or other situations where a type is needed but no actual data is stored.
    Example: `fn log_message(msg: &str) { print(msg); } // Implicitly returns unit ()`

## Type Inference

Aero employs a powerful type inference system, primarily local, to reduce the need for explicit type annotations, making code cleaner and more concise. The compiler deduces types based on how variables are initialized, the types of expressions, and function signatures.

### Rules and Examples:

1.  **Variable Initialization**: The type of a variable is inferred from the type of its initializer.
    ```aero
    let x = 10;         // x is inferred as i64 (default integer type)
    let y = 3.14;       // y is inferred as f64 (default float type)
    let name = "Aero";  // name is inferred as &str (from string literal)
    let mut message = String::from("Hello"); // message is inferred as String
    let is_done = false; // is_done is inferred as bool
    ```
    If ambiguity exists or a different specific type is desired, annotations can be used:
    ```aero
    let count: i32 = 50;
    let price = 19.99f32; // Suffix notation also guides inference or acts as annotation
    ```

2.  **Function Return Types**:
    *   **Explicit Annotation (Recommended for public APIs):**
        ```aero
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }
        ```
    *   **Inference from `return` statement (for local/private functions):**
        ```aero
        fn multiply(a: i32, b: i32) { // Return type i32 inferred
            return a * b;
        }
        // However, for clarity and stability, especially in public interfaces,
        // explicit return types are mandatory or strongly encouraged.
        // Recursive functions often require explicit return type annotations.
        ```

3.  **Contextual Typing and Propagation**: The expected type in a certain context can influence the type inference of expressions or parts of expressions.
    ```aero
    fn process_vector(data: Vec<i32>) { /* ... */ }

    fn main() {
        let numbers = vec![1, 2, 3]; // Type of `numbers` might be inferred as Vec<i32> if `vec![]` macro
                                     // can infer from usage or if it defaults, or if used with `process_vector`.
                                     // More typically, a collection literal might need annotation or RHS type:
        let explicit_numbers: Vec<i32> = vec![10, 20, 30];
        process_vector(explicit_numbers);
    }
    ```

4.  **Inference with Generics**: Type inference works effectively with generic types.
    ```aero
    struct Container<T> { value: T }
    impl<T> Container<T> {
        fn new(value: T) -> Container<T> { Container { value } }
    }

    fn main() {
        let int_container = Container::new(42); // Inferred as Container<i64> (or default int type)
        let str_container = Container::new("hello"); // Inferred as Container<&str>

        print(int_container.value);
        print(str_container.value);
    }
    ```
    Sometimes, you might need to provide type hints if the compiler cannot uniquely determine the generic type arguments, often called "turbofish" syntax if adopted from Rust:
    ```aero
    // let items = collect_into_vec(); // Might be ambiguous if `collect_into_vec` is generic.
    // let items = collect_into_vec::<String>(); // Using turbofish to specify T = String.
    ```

While type inference is powerful, explicit type annotations are crucial for:
-   Function signatures (parameters and return types), especially for public APIs.
-   Variables where the type is ambiguous or a specific type is required over the default.
-   Struct and enum field definitions.
-   Ensuring code clarity and maintainability, acting as documentation.

## Generic Type Parameters

Aero supports generic programming through type parameters, allowing the creation of flexible and reusable code that works with various types while maintaining type safety.

### Syntax:

Generic type parameters are declared within angle brackets (`<>`) after the name of a `struct`, `enum`, `fn`, or `trait`.

```aero
struct Option<T> {
    data: T,
}

// Generic Enum
enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Generic Function
fn process_item<T>(item: T) -> T {
    // Do something with item, assuming T supports needed operations (via trait bounds)
    print("Processing an item."); // Simplified example
    item
}

// Generic Trait (Conceptual)
trait Renderer<T> {
    fn render(&self, item: T);
}
```

### Usage and Instantiation:

When using a generic type or function, you can either specify the concrete types for the type parameters explicitly or let the compiler infer them where possible.

```aero
// Using a generic struct
let int_box = Box { data: 10 }; // T is inferred as i64 (or default int)
let string_box = Box { data: String::from("Aero") }; // T is inferred as String

// Using a generic enum
let success_result: Result<i32, String> = Result::Ok(100);
let error_result: Result<i32, String> = Result::Err(String::from("Something went wrong"));

// Using a generic function
let num = process_item(5); // T is inferred as i64
let text = process_item(String::from("test")); // T is inferred as String
```
Generics are fundamental to writing abstract and reusable code, forming the basis for many standard library features like collections and option/result types.

## Trait Bounds

Trait bounds are used to constrain generic type parameters, ensuring that only types implementing specific traits can be used. This enables polymorphism and allows generic functions to call methods defined by the bounded traits.

### Syntax:

Trait bounds are specified using a colon `:` after the type parameter, followed by the trait(s). For multiple traits, `+` is used. `where` clauses can be used for more complex or numerous bounds to improve readability.

```aero
// Simple trait bound
fn print_displayable<T: Display>(item: T) {
    item.display(); // Assumes Display trait provides a display() method
}

// Multiple trait bounds
fn compare_and_print<T: Display + PartialEq>(a: T, b: T) {
    if a == b { // Requires PartialEq for ==
        print_displayable(a); // Requires Display
    } else {
        print("Items are not equal.");
    }
}

// Using a `where` clause for clarity with multiple or complex bounds
struct DataProcessor<T, U>
    where T: DataSource + Clone,
          U: DataSink + Default
{
    source: T,
    sink: U,
}

impl<T, U> DataProcessor<T, U>
    where T: DataSource + Clone,
          U: DataSink + Default
{
    fn new(source: T) -> Self {
        DataProcessor { source, sink: U::default() }
    }

    fn process_all_data(&self) {
        let data_chunk = self.source.get_data();
        self.sink.write_data(data_chunk);
        // ... more complex logic
    }
}
```

### Polymorphism with Trait Bounds

Trait bounds are the primary mechanism for achieving polymorphism in Aero.
-   **Static Dispatch (Compile-Time Polymorphism):** When you use generics with trait bounds, the compiler generates specialized code for each concrete type used. This is often called "monomorphization." It results in highly efficient code because the exact method to call is known at compile time, allowing for inlining and other optimizations. The examples above use static dispatch.
-   **Dynamic Dispatch (Runtime Polymorphism):** Aero might also support trait objects (e.g., `&dyn MyTrait`) for dynamic dispatch. This allows you to have collections of different types that all implement the same trait, and the specific method to call is determined at runtime. Trait objects have a runtime cost (usually vtable lookups) but offer more flexibility in some scenarios. (This would be detailed further if trait objects are a confirmed feature).

### Rules for Trait Bounds:

1.  **Compile-Time Enforcement**: The compiler verifies at compile time that any concrete type used as a generic argument for `T` actually implements all traits specified in `T`'s bounds. If not, it's a compile-time error.
2.  **Method Availability**: Within the generic function or type definition, only methods (and associated types/constants) defined by the traits in the bounds (or universally available methods) can be called on instances of the generic type `T`. This ensures that the generic code is valid for any type that satisfies the bounds.

## Tuple Types

Tuple types are a way to group a fixed number of values of potentially different types into a single compound type.

-   **Syntax:** `(T1, T2, ..., Tn)`
-   **Usage:**
    *   Creating simple, heterogeneous fixed-size collections.
    *   Returning multiple values from a function.
-   **Accessing Elements:** Tuple elements are accessed by their index (e.g., `my_tuple.0`, `my_tuple.1`).

```aero
fn get_coordinates() -> (i32, i32, String) {
    (10, -5, String::from("Origin"))
}

fn main() {
    let point1: (i32, f64) = (100, 25.5);
    let x_coord = point1.0; // Accesses the first element (100)
    let y_coord = point1.1; // Accesses the second element (25.5)
    print("Point: ({}, {})", x_coord, y_coord);

    let (x, y, label) = get_coordinates(); // Destructuring a tuple
    print("Location: {} at ({}, {})", label, x, y);

    let unit_tuple: () = (); // The unit type is an empty tuple
}
```

## Array and Slice Types

### Arrays
Arrays are fixed-size collections of elements of the same type, stored contiguously in memory.
-   **Syntax:** `[T; N]`, where `T` is the type of the elements and `N` is the compile-time constant size.
-   **Usage:** Useful when you know the number of elements at compile time. Arrays are stack-allocated if their size is small enough or part of another struct.

```aero
fn main() {
    let numbers: [i32; 5] = [1, 2, 3, 4, 5]; // An array of 5 i32 integers
    let first_number = numbers[0];
    // numbers[5] = 6; // Compile-time or runtime error: index out of bounds

    let mut names: [&str; 3] = ["Alice", "Bob", "Charlie"];
    names[0] = "Alicia";
    print("First name: {}", names[0]);
}
```

### Slices
Slices provide a view into a contiguous sequence of elements, typically a portion of an array or a `String`. Slices do not own the data they point to; they are a borrowed view.
-   **Syntax:** `&[T]` for an immutable slice, `&mut [T]` for a mutable slice.
-   **Usage:** Passing sequences of data without copying, providing safe access to parts of arrays or other contiguous data structures.

```aero
fn print_slice(data: &[i32]) { // Takes an immutable slice of i32
    print("Slice contents: [");
    for item in data { // Iteration over slices is common
        print("{}, ", *item);
    }
    print("]");
}

fn main() {
    let arr: [i32; 4] = [10, 20, 30, 40];

    let slice1: &[i32] = &arr; // Slice covering the whole array
    let slice2: &[i32] = &arr[1..3]; // Slice from index 1 up to (but not including) 3 -> [20, 30]

    print_slice(slice1);
    print_slice(slice2);

    let my_string = String::from("Hello Aero");
    let string_slice: &str = &my_string[0..5]; // "Hello" (&str is a specialized slice)
    print("String slice: {}", string_slice);
}
```
Slices are fundamental for writing efficient and safe code that operates on sequences of data. The `&str` type is essentially a slice of `u8` bytes that are guaranteed to be valid UTF-8.

## Type Aliases

Type aliases allow you to give a new name to an existing type. This can be useful for reducing verbosity and improving readability, especially for complex or frequently used type signatures.
-   **Syntax:** `type NewName = ExistingType;`

```aero
type Age = u8;
type UserId = i64;
type Point = (i32, i32);
type StringMap<V> = HashMap<String, V>; // Generic type alias

fn display_user(id: UserId, user_age: Age) {
    print("User ID: {}, Age: {}", id, user_age);
}

fn main() {
    let user_id: UserId = 12345;
    let current_age: Age = 30;
    display_user(user_id, current_age);

    let p1: Point = (10, 20);
    print("Point: ({}, {})", p1.0, p1.1);

    // Assuming HashMap is defined
    // let mut user_scores: StringMap<i32> = HashMap::new();
    // user_scores.insert(String::from("player1"), 100);
}
```
Type aliases do not create new distinct types; they are simply synonyms. They are resolved at compile time and do not affect runtime performance.

## Interaction with Ownership and Borrowing

The type system is deeply intertwined with Aero's ownership and borrowing model to ensure memory safety.

1.  **Reference Types:** References are distinct types from the values they point to.
    *   `&T`: An immutable reference to a value of type `T`.
    *   `&mut T`: A mutable reference to a value of type `T`.
    The borrow checker ensures that these references are always valid and that aliasing rules (one mutable or many immutable references) are followed.

2.  **Lifetimes as Part of Reference Types:** References can also include lifetime parameters (e.g., `&'a T`, `&'a mut T`). Lifetimes specify the scope for which a reference is valid, preventing dangling references. The type system, along with the borrow checker, uses lifetime information to validate reference safety.

3.  **Ownership in Type Signatures:** Function signatures explicitly state whether they take ownership of, borrow, or mutably borrow their parameters.
    ```aero
    fn process_owned(data: String) { /* takes ownership */ }
    fn view_borrowed(data: &String) { /* takes an immutable borrow */ }
    fn modify_borrowed(data: &mut String) { /* takes a mutable borrow */ }
    ```
    The type system ensures that callers respect these ownership contracts. For instance, you cannot pass an immutable reference to a function expecting a mutable one, nor can you use a value after it has been moved into a function that takes ownership.

4.  **Data Structures:** The types of fields within structs or variants within enums dictate whether the structure owns its data or borrows it.
    *   `struct MyStruct { field: String }` // Owns `field`
    *   `struct MyView<'a> { field: &'a str }` // Borrows `field`
    The type system, in conjunction with lifetime rules, ensures that if a struct contains references, those references are valid for the lifetime of the struct instance.

The strictness of the type system, combined with ownership and borrowing rules, allows Aero to prevent many common programming errors (null pointers, use-after-free, data races) at compile time.

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


