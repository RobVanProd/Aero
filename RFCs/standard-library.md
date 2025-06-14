# RFC: Aero Standard Library - Core Modules and APIs

**RFC Number:** 0001
**Author:** Aero Core Team
**Status:** Draft
**Date:** {{YYYY-MM-DD}} <!-- Will be replaced by current date or a placeholder -->

## Abstract

This RFC proposes the initial structure and core APIs for the Aero Standard Library. The standard library aims to provide essential functionalities that are commonly needed by developers, promoting code reuse, efficiency, and a consistent development experience. This proposal focuses on fundamental modules: `core` (implicit), `io`, `collections`, `string`, and `math`.

## Motivation

A robust standard library is crucial for the usability and adoption of any programming language. It provides building blocks that simplify common tasks, ensures a certain level of quality and performance for these core utilities, and reduces the need for developers to rely on third-party libraries for basic operations.

## Proposed Structure

The Aero Standard Library will be organized into modules. Developers can import specific items from these modules. A `core` module containing the most fundamental types and traits (like `Option<T>`, `Result<T, E>`, `Drop`, `Display`, basic operators, etc.) is assumed to be implicitly available or auto-imported in most contexts.

The initially proposed top-level modules include:

-   `core`: (Implicit or auto-imported) Contains fundamental types, traits, and macros.
-   `io`: For input and output operations.
-   `collections`: For common data structures.
-   `string`: For string manipulation.
-   `math`: For mathematical functions and constants.
-   `os`: For interacting with the operating system (e.g., environment variables, process management - to be detailed in a future RFC).
-   `fs`: For file system operations (expanding on basic `io` file operations - to be detailed in a future RFC).

## Detailed Design

### Error Handling

Many standard library functions, especially in `io` and `collections` (e.g., when an operation might fail due to bounds or non-existent keys), will return a `Result<T, E>` enum to handle errors gracefully. `E` will typically be a specific error enum or struct relevant to the module (e.g., `io::Error`, `collections::AccessError`).

```aero
// Assumed to be in `core` or a prelude
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Example of a potential I/O Error type
enum IoError {
    NotFound,
    PermissionDenied,
    InvalidInput,
    Other(String), // For more dynamic error messages
}
```

---

### Module: `core` (Implicit / Auto-Imported)

This module is not explicitly imported but provides foundational elements.

**Key Data Structures & Enums:**

*   `Option<T>`: Represents an optional value.
    *   `Some(T)`: Contains a value.
    *   `None`: Represents no value.
*   `Result<T, E>`: Represents a value that could be an error.
    *   `Ok(T)`: Contains a success value.
    *   `Err(E)`: Contains an error value.

**Key Traits (Conceptual):**

*   `ToString`: For converting a value to a `String`.
*   `Display`: For user-facing string representation (used by `print` functions).
*   `Debug`: For developer-facing string representation.
*   `Clone`: For explicitly creating a deep copy of a value.
*   `Copy`: Marker trait for types that can be copied by simple bitwise copy.
*   `Drop`: For custom cleanup logic when a value goes out of scope.
*   Operator traits (e.g., `Add`, `Sub`, `Mul`, `Div`, `PartialEq`, `Ord`).

---

### Module: `io`

Provides input and output functionalities.

**Key Functions:**

1.  **`print(message: &dyn Display)`**
    *   **Description:** Prints a message to the standard output. The message must implement the `Display` trait. Does not automatically append a newline.
    *   **Parameters:**
        *   `message`: The value to print.
    *   **Return Type:** `unit`
    *   **Errors/Panics:** May panic if writing to stdout fails at a low level.

2.  **`println(message: &dyn Display)`**
    *   **Description:** Prints a message to the standard output, followed by a newline character.
    *   **Parameters:**
        *   `message`: The value to print.
    *   **Return Type:** `unit`
    *   **Errors/Panics:** Similar to `print`.

3.  **`eprint(message: &dyn Display)`**
    *   **Description:** Prints a message to the standard error. Does not automatically append a newline.
    *   **Parameters:**
        *   `message`: The value to print.
    *   **Return Type:** `unit`
    *   **Errors/Panics:** May panic if writing to stderr fails.

4.  **`eprintln(message: &dyn Display)`**
    *   **Description:** Prints a message to the standard error, followed by a newline character.
    *   **Parameters:**
        *   `message`: The value to print.
    *   **Return Type:** `unit`
    *   **Errors/Panics:** Similar to `eprint`.

5.  **`read_line() -> Result<String, IoError>`**
    *   **Description:** Reads a line of text from standard input until a newline character is encountered (or EOF). The newline character is included in the returned string.
    *   **Return Type:** `Result<String, IoError>`
        *   `Ok(String)`: The line read from input.
        *   `Err(IoError)`: If an error occurs during reading.
    *   **Errors/Panics:** None directly, errors are returned in `Result`.

**File I/O (High-Level Outline):**

A `File` struct and related functions will be provided.

*   **`File::open(path: &str) -> Result<File, IoError>`**: Opens a file in read-only mode.
*   **`File::create(path: &str) -> Result<File, IoError>`**: Creates a new file for writing, or truncates it if it exists.
*   **`file.read(buffer: &mut [u8]) -> Result<usize, IoError>`**: Reads bytes from the file into the buffer, returns number of bytes read.
*   **`file.write(buffer: &[u8]) -> Result<usize, IoError>`**: Writes bytes from the buffer to the file, returns number of bytes written.
*   **`file.close() -> Result<unit, IoError>`**: Closes the file. (Often handled by `Drop` trait automatically).
*   Other methods like `seek`, `flush`, reading to string, etc., will also be considered.

---

### Module: `collections`

Provides common generic data structures.

#### `Vec<T>` - Dynamic Array

A contiguous, growable array type.

**Key Methods:**

1.  **`Vec::new() -> Vec<T>`**
    *   **Description:** Creates a new, empty vector.
    *   **Return Type:** `Vec<T>`

2.  **`Vec::with_capacity(capacity: usize) -> Vec<T>`**
    *   **Description:** Creates a new, empty vector with at least the specified capacity.
    *   **Parameters:**
        *   `capacity`: The initial capacity.
    *   **Return Type:** `Vec<T>`

3.  **`vec.push(value: T)`**
    *   **Description:** Appends an element to the end of the vector. May reallocate if capacity is exceeded.
    *   **Parameters:**
        *   `value`: The value to append (ownership is moved).
    *   **Return Type:** `unit`

4.  **`vec.pop() -> Option<T>`**
    *   **Description:** Removes the last element from the vector and returns it, or `None` if the vector is empty.
    *   **Return Type:** `Option<T>`

5.  **`vec.get(index: usize) -> Option<&T>`**
    *   **Description:** Returns an immutable reference to the element at the given index, or `None` if the index is out of bounds.
    *   **Parameters:**
        *   `index`: The index of the element.
    *   **Return Type:** `Option<&T>`

6.  **`vec.get_mut(index: usize) -> Option<&mut T>`**
    *   **Description:** Returns a mutable reference to the element at the given index, or `None` if the index is out of bounds.
    *   **Parameters:**
        *   `index`: The index of the element.
    *   **Return Type:** `Option<&mut T>`

7.  **`vec.len() -> usize`**
    *   **Description:** Returns the number of elements in the vector.
    *   **Return Type:** `usize`

8.  **`vec.is_empty() -> bool`**
    *   **Description:** Returns `true` if the vector contains no elements.
    *   **Return Type:** `bool`

9.  **`vec.capacity() -> usize`**
    *   **Description:** Returns the number of elements the vector can hold without reallocating.
    *   **Return Type:** `usize`

10. **`vec.clear()`**
    *   **Description:** Removes all elements from the vector.
    *   **Return Type:** `unit`

(Other methods like `insert`, `remove`, `iter`, `iter_mut`, `contains`, etc., would be included.)

#### `HashMap<K, V>` - Hash Map

A collection of key-value pairs, where keys are unique and hashed for efficient lookup.
Requires `K` to implement `Hash` and `Eq` traits (from `core`).

**Key Methods (Outline):**

1.  **`HashMap::new() -> HashMap<K, V>`**
    *   **Description:** Creates a new, empty hash map.
    *   **Return Type:** `HashMap<K, V>`

2.  **`map.insert(key: K, value: V) -> Option<V>`**
    *   **Description:** Inserts a key-value pair into the map. If the key already existed, the old value is returned. Ownership of key and value is moved.
    *   **Return Type:** `Option<V>` (the old value if the key was present).

3.  **`map.get(key: &K) -> Option<&V>`**
    *   **Description:** Returns an immutable reference to the value corresponding to the key.
    *   **Parameters:**
        *   `key`: A reference to the key.
    *   **Return Type:** `Option<&V>`

4.  **`map.get_mut(key: &K) -> Option<&mut V>`**
    *   **Description:** Returns a mutable reference to the value corresponding to the key.
    *   **Parameters:**
        *   `key`: A reference to the key.
    *   **Return Type:** `Option<&mut V>`

5.  **`map.remove(key: &K) -> Option<V>`**
    *   **Description:** Removes a key from the map, returning the value at the key if the key was previously in the map.
    *   **Return Type:** `Option<V>`

6.  **`map.contains_key(key: &K) -> bool`**
    *   **Description:** Returns `true` if the map contains a value for the specified key.
    *   **Return Type:** `bool`

7.  **`map.len() -> usize`**
    *   **Description:** Returns the number of elements in the map.
    *   **Return Type:** `usize`

8.  **`map.is_empty() -> bool`**
    *   **Description:** Returns `true` if the map contains no elements.
    *   **Return Type:** `bool`

(Other methods like `keys`, `values`, `iter`, `iter_mut`, `clear`, etc., would be included.)

---

### Module: `string`

Provides operations for the owned `String` type and potentially for `&str` slices. Many `&str` methods might be directly available on string literals or `String` via deref coercions.

**`String` Type Methods:**

1.  **`String::new() -> String`**
    *   **Description:** Creates a new, empty string.
    *   **Return Type:** `String`

2.  **`String::from_str(s: &str) -> String`** (or `s.to_string()` via `ToString` trait)
    *   **Description:** Creates a new `String` from a string slice.
    *   **Parameters:**
        *   `s`: The string slice to copy from.
    *   **Return Type:** `String`

3.  **`string.len() -> usize`**
    *   **Description:** Returns the length of the string in bytes (not necessarily number of characters due to UTF-8).
    *   **Return Type:** `usize`

4.  **`string.is_empty() -> bool`**
    *   **Description:** Returns `true` if the string has a length of 0.
    *   **Return Type:** `bool`

5.  **`string.push_str(s: &str)`**
    *   **Description:** Appends a string slice to the end of this `String`.
    *   **Parameters:**
        *   `s`: The string slice to append.
    *   **Return Type:** `unit`

6.  **`string.push(ch: char)`**
    *   **Description:** Appends a single character to the end of this `String`.
    *   **Parameters:**
        *   `ch`: The character to append.
    *   **Return Type:** `unit`

7.  **`string.capacity() -> usize`**
    *   **Description:** Returns the allocated capacity of the string in bytes.
    *   **Return Type:** `usize`

8.  **`string.clear()`**
    *   **Description:** Empties the string, making its length 0.
    *   **Return Type:** `unit`

**Operations (may be on `String` or `&str` via traits):**

9.  **`s.contains(pattern: &str) -> bool`** (or `Pattern` trait for more flexibility)
    *   **Description:** Returns `true` if the string contains the given pattern.
    *   **Return Type:** `bool`

10. **`s.replace(from: &str, to: &str) -> String`**
    *   **Description:** Replaces all occurrences of a substring with another. Returns a new `String`.
    *   **Return Type:** `String`

11. **`s.split(delimiter: &str) -> Vec<&str>`** (Iterator-based return type preferred)
    *   **Description:** Splits the string by a delimiter, returning an iterator or collection of string slices.
    *   **Return Type:** An iterator type yielding `&str` (e.g., `SplitIterator<&str>`).

12. **`s.trim() -> &str`**
    *   **Description:** Returns a string slice with leading and trailing whitespace removed.
    *   **Return Type:** `&str`

(Many other methods like `starts_with`, `ends_with`, `to_lowercase`, `to_uppercase`, `chars` (iterator), `bytes` (iterator), etc., would be included.)

---

### Module: `math`

Provides common mathematical constants and functions. Functions typically operate on `float` types (e.g., `f64`) but may have generic versions or overloads for `f32` or even integer types where appropriate.

**Constants:**

*   **`PI: f64`**: The mathematical constant π (approx. 3.1415926535...).
*   **`E: f64`**: Euler's number e (approx. 2.7182818284...).
*   **`TAU: f64`**: The mathematical constant τ (2π, approx. 6.2831853071...).

**Functions (operating on `f64` by default, unless specified):**

1.  **`abs<T: SignedNumber>(x: T) -> T`** (or specific `f64::abs(f64) -> f64`, `i64::abs(i64) -> i64`)
    *   **Description:** Returns the absolute value of `x`.
    *   **Return Type:** `T` (same type as input)

2.  **`sqrt(x: f64) -> f64`**
    *   **Description:** Returns the square root of `x`.
    *   **Errors/Panics:** May return `NaN` or error if `x` is negative.

3.  **`pow(base: f64, exponent: f64) -> f64`** (or `powi` for integer exponents)
    *   **Description:** Returns `base` raised to the power of `exponent`.
    *   **Return Type:** `f64`

4.  **`sin(x: f64) -> f64`**: Computes the sine of `x` (in radians).
5.  **`cos(x: f64) -> f64`**: Computes the cosine of `x` (in radians).
6.  **`tan(x: f64) -> f64`**: Computes the tangent of `x` (in radians).
7.  **`asin(x: f64) -> f64`**: Computes the arcsine of `x`.
8.  **`acos(x: f64) -> f64`**: Computes the arccosine of `x`.
9.  **`atan(x: f64) -> f64`**: Computes the arctangent of `x`.
10. **`atan2(y: f64, x: f64) -> f64`**: Computes the arctangent of `y/x` using the signs of `y` and `x` to determine the quadrant.
11. **`ln(x: f64) -> f64`**: Computes the natural logarithm of `x`.
    *   **Errors/Panics:** Error if `x` is zero or negative.
12. **`log(base: f64, x: f64) -> f64`**: Computes the logarithm of `x` to the given `base`.
13. **`log10(x: f64) -> f64`**: Computes the base-10 logarithm of `x`.
14. **`exp(x: f64) -> f64`**: Computes e<sup>x</sup>.
15. **`floor(x: f64) -> f64`**: Returns the largest integer less than or equal to `x`.
16. **`ceil(x: f64) -> f64`**: Returns the smallest integer greater than or equal to `x`.
17. **`round(x: f64) -> f64`**: Returns `x` rounded to the nearest integer.
18. **`min<T: Ord>(a: T, b: T) -> T`**: Returns the minimum of `a` and `b`.
19. **`max<T: Ord>(a: T, b: T) -> T`**: Returns the maximum of `a` and `b`.

---

## Unresolved Questions

-   Specific error types for each module (e.g., `collections::Error`, `math::Error`).
-   Exact set of traits in the `core` module.
-   Async I/O strategy.
-   Concurrency primitives (threads, mutexes, channels).
-   Date and time functionalities.
-   Networking.
-   Detailed API for iterators for collections and strings.

## Future Possibilities

-   Modules for `fs` (advanced file system operations), `os` (operating system interactions), `net` (networking), `time`, `regex`, `json`, `crypto`, etc.
-   Macros for convenience (e.g., `vec![]`, `println!`).
-   Comprehensive testing utilities.

This RFC provides a foundational layer. Subsequent RFCs will detail additional modules and expand upon the APIs presented here.
