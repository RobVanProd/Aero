# Tutorial 2: Core Aero Language Features

Welcome to the second tutorial in the Aero series! Building on what you learned in "Getting Started," this tutorial dives into the core features of the Aero language. We'll cover variables, data types, operators, control flow, functions, and comments.

## Variables and Mutability

Variables are how we store data in Aero programs.

### Declaring Variables with `let`

You can declare variables using the `let` keyword. By default, variables in Aero are immutable, meaning once a value is assigned, it cannot be changed.

```aero
fn main() {
    let message = "Hello from Aero!"; // `message` is immutable
    io::println(message);

    // message = "New message"; // This would cause a compile-time error!
}
```

Aero uses type inference, meaning the compiler can often figure out the type of the variable automatically based on the value you assign it. In the example above, `message` is inferred to be of type `&str` (a string slice).

### Mutable Variables with `mut`

If you need a variable whose value can be changed, you can use the `mut` keyword.

```aero
fn main() {
    let mut count = 0; // `count` is mutable
    io::println(count); // Prints 0

    count = 1;
    io::println(count); // Prints 1
}
```
Here, `count` is inferred as an integer type (likely `i64`).

### Type Annotations

While Aero has good type inference, you can also explicitly specify the type of a variable using a colon `:` followed by the type name. This can be useful for clarity or when the compiler needs help.

```aero
fn main() {
    let score: i32 = 100;
    let pi: f64 = 3.14159;
    let name: String = String::from("Alice"); // An owned string

    io::println(score);
    io::println(pi);
    io::println(name);
}
```

## Basic Data Types

Aero comes with several built-in data types for common kinds of data.

*   **Integers**: Whole numbers.
    *   Example: `let age: i32 = 30; let big_number: i64 = 1_000_000_000;`
    *   Aero may support various sizes (e.g., `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`). `i64` is often the default for `int`.
*   **Floating-Point Numbers**: Numbers with a decimal point.
    *   Example: `let price: f64 = 19.99; let temperature: f32 = -2.5;`
    *   Aero may support `f32` (single-precision) and `f64` (double-precision). `f64` is often the default for `float`.
*   **Booleans**: Truth values.
    *   Example: `let is_active: bool = true; let is_admin: bool = false;`
    *   The two possible values are `true` and `false`.
*   **Characters**: Single Unicode scalar values.
    *   Example: `let initial: char = 'A'; let emoji: char = '🚀';`
    *   Character literals are enclosed in single quotes.
*   **Strings**: Sequences of characters. Aero has two main string types:
    *   `String`: An owned, growable, mutable string. It's stored on the heap.
        ```aero
        let mut name: String = String::from("Bob");
        name.push_str(" the Builder"); // Modifying the String
        io::println(name);
        ```
    *   `&str` (string slice or string literal): An immutable reference to a sequence of characters. String literals (like `"Hello"`) are `&str`.
        ```aero
        let greeting: &str = "Hello, Aero learner!";
        io::println(greeting);
        ```

## Operators

Aero supports various operators to perform common operations.

### Arithmetic Operators

*   `+` (addition)
*   `-` (subtraction)
*   `*` (multiplication)
*   `/` (division)
*   `%` (modulo/remainder)

```aero
fn main() {
    let a = 10;
    let b = 3;

    io::println(a + b); // 13
    io::println(a - b); // 7
    io::println(a * b); // 30
    io::println(a / b); // 3 (integer division truncates)
    io::println(a % b); // 1

    let c: f64 = 10.0;
    let d: f64 = 3.0;
    io::println(c / d); // Approximately 3.333...
}
```

### Comparison Operators

These operators compare two values and return a boolean (`true` or `false`).

*   `==` (equal to)
*   `!=` (not equal to)
*   `<` (less than)
*   `>` (greater than)
*   `<=` (less than or equal to)
*   `>=` (greater than or equal to)

```aero
fn main() {
    let x = 5;
    let y = 10;

    io::println(x == y); // false
    io::println(x != y); // true
    io::println(x < y);  // true
    io::println(x > y);  // false
    io::println(x <= 5); // true
    io::println(y >= 10); // true
}
```

### Logical Operators

These operators are used to combine boolean expressions.

*   `&&` (logical AND) - `true` if both operands are `true`.
*   `||` (logical OR) - `true` if at least one operand is `true`.
*   `!` (logical NOT) - inverts the boolean value.

```aero
fn main() {
    let is_sunny = true;
    let is_warm = false;

    io::println(is_sunny && is_warm); // false
    io::println(is_sunny || is_warm); // true
    io::println(!is_warm);          // true
}
```

### Assignment Operator

*   `=` (assignment) - assigns the value on the right to the variable on the left.
    ```aero
    let mut value = 10; // Initialization uses let
    value = 20;       // Assignment to a mutable variable
    io::println(value);
    ```
    Aero might also support compound assignment operators like `+=`, `-=`, `*=`, etc. (e.g., `value += 5;` is `value = value + 5;`).

## Control Flow

Control flow constructs allow you to run code conditionally or repeatedly.

### `if`/`else if`/`else` Expressions

`if` expressions allow you to branch your code based on conditions.

```aero
fn main() {
    let number = 7;

    if number < 5 {
        io::println("Condition was true: number is less than 5");
    } else if number < 10 {
        io::println("Condition was true: number is less than 10 but not less than 5");
    } else {
        io::println("Condition was false: number is 10 or greater");
    }

    // `if` is an expression, so you can use it in `let` statements:
    let result = if number % 2 == 0 {
        "even"
    } else {
        "odd"
    };
    io::println(result); // Prints "odd"
}
```
Note: All branches of an `if`/`else if`/`else` expression (when used to assign a value) must evaluate to the same type.

### `while` Loops

`while` loops execute a block of code as long as a condition remains `true`.

```aero
fn main() {
    let mut count = 0;
    while count < 3 {
        io::println(count);
        count = count + 1; // Or count += 1;
    }
    io::println("Loop finished!");
}
```
Output:
```
0
1
2
Loop finished!
```

### `loop` (Infinite Loop)

The `loop` keyword creates an infinite loop. You typically use `break` to exit a `loop`.

```aero
fn main() {
    let mut counter = 0;
    loop {
        io::println("Looping...");
        counter = counter + 1;
        if counter == 3 {
            break; // Exits the loop
        }
    }
    io::println("Exited the infinite loop.");
}
```

### `for` Loops (Conceptual)

`for` loops are used to iterate over a sequence of items, like elements in a collection or a range. The exact syntax for ranges or collection iteration will be detailed in specific RFCs or tutorials on collections.

A conceptual example:
```aero
/* Conceptual - exact syntax for ranges or collection iteration may vary
fn main() {
    // Example: Iterating over a range (if supported)
    for i in 0..3 { // Iterate from 0 up to (but not including) 3
        io::println(i);
    }
    // Output: 0, 1, 2

    // Example: Iterating over elements of a Vec (once Vec is introduced)
    // let my_vec = vec!["a", "b", "c"];
    // for item in my_vec {
    //     io::println(item);
    // }
}
*/
```
We'll explore `for` loops in more detail when we discuss collections.

## Functions

Functions are blocks of code that perform a specific task. We've already used the `main` function.

### Defining Functions

You define functions using the `fn` keyword.

```aero
// A simple function
fn greet() {
    io::println("Hello from a function!");
}

// Function with parameters
// Type annotations for parameters are mandatory.
fn greet_person(name: &str, age: i32) {
    io::print("Hello, ");
    io::print(name);
    io::print("! You are ");
    io::print(age); // Note: print does not automatically convert numbers to strings for display
                   // A proper Display trait implementation or string formatting would be needed here.
                   // For simplicity, we'll assume io::print can handle basic types for now,
                   // or we'd use string formatting if available.
    io::println(" years old.");
}

// Function with a return value
// The return type is specified after an arrow `->`.
fn add(a: i32, b: i32) -> i32 {
    a + b // In Aero, if the last expression in a function is not followed by a semicolon,
          // it is automatically returned. Alternatively, use the `return` keyword.
}

fn main() {
    greet(); // Calling the greet function

    greet_person("Alice", 30);
    greet_person("Bob", 25);

    let sum = add(5, 7);
    // io::println(sum); // Again, printing numbers might require specific formatting.
                       // Let's assume a way to print numbers for tutorial purposes:
    io::println("Sum is some_value"); // Placeholder for actual sum printing
                                      // A real std lib would have io::println_int(sum) or similar,
                                      // or string formatting like io::println(format!("Sum is {}", sum))
                                      // For now, let's imagine a basic print functionality.

    // To print the sum, one might need a to_string method or specific print function:
    // let sum_str = sum.to_string(); // If available
    // io::println(sum_str);
    // Or, if io::println is very flexible (like Rust's println! macro):
    // io::println("The sum of 5 and 7 is: {}", sum);
}
```
**Note on Printing Numbers:** Standard libraries often require explicit conversion of numbers to strings for printing with generic `print` functions, or they provide formatted printing utilities. For this tutorial, assume `io::println` can handle basic types or that appropriate formatting tools exist.

### Function Return Values

-   If a function returns a value, the type must be declared after `->`.
-   The last expression in the function body is implicitly returned if it doesn't have a semicolon.
-   You can also use the `return` keyword to return early from a function.

```aero
fn get_five() -> i32 {
    5 // Implicit return
}

fn explicit_return_add(a: i32, b: i32) -> i32 {
    return a + b; // Explicit return
}

fn main() {
    let five = get_five();
    // io::println(five); // Assuming print for i32
    let total = explicit_return_add(10, 2);
    // io::println(total); // Assuming print for i32
}
```

### Functions without Return Values (`unit`)

If a function doesn't explicitly return a value, it implicitly returns the `unit` type (represented as `()`).

```aero
fn log_message(message: &str) { // No return type specified, so it returns unit
    io::println(message);
}

fn main() {
    let result = log_message("Processing complete.");
    // `result` here would be of type `unit`
}
```

## Comments

Comments are notes in your source code that are ignored by the compiler. They are for human readers to understand the code better.

### Line Comments

Line comments start with `//` and go to the end of the line.

```aero
// This is a line comment.
let x = 10; // This comment is at the end of a line.
```

### Block Comments

Block comments start with `/*` and end with `*/`. They can span multiple lines.

```aero
/*
This is a block comment.
It can cover multiple lines
and is useful for longer explanations.
*/
fn main() {
    /* You can also use them mid-line, but it's less common: let y = /* ignored */ 5; */
    io::println("Block comments are useful!");
}
```
Block comments typically do not nest in Aero (similar to C or Rust).

## Summary

In this tutorial, you've learned about several core features of Aero:
-   **Variables**: `let` for immutable, `let mut` for mutable, with type inference and optional annotations.
-   **Basic Data Types**: Integers, floats, booleans, characters, and strings.
-   **Operators**: Arithmetic, comparison, logical, and assignment.
-   **Control Flow**: `if`/`else`, `while`, and `loop` / `break`. (`for` loops conceptually).
-   **Functions**: Definition (`fn`), parameters, return types, and calling.
-   **Comments**: `//` for line comments, `/* ... */` for block comments.

These building blocks are essential for writing more complex Aero programs. In the next tutorial, we'll start looking at Aero's powerful features for memory safety: Ownership and Borrowing.
