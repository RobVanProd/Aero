# Tutorial 3: Ownership and Borrowing in Aero

Welcome to the third tutorial in the Aero series! This is a crucial one, as we'll explore Aero's core memory management features: **Ownership** and **Borrowing**. These concepts are key to how Aero achieves memory safety (preventing bugs like dangling pointers or data races) without needing a garbage collector.

If you're coming from languages like C/C++ or languages with garbage collectors (like Java, Python, or JavaScript), this model might seem different, but it's designed to provide both safety and performance.

## What is Ownership?

In many programming languages, memory management is handled either manually (like in C/C++, which can be error-prone) or by a garbage collector (which can introduce overhead). Aero takes a different approach: memory is managed through a system of ownership with a set of rules that the compiler checks at compile time.

**Benefits of Ownership:**
-   **Memory Safety:** Automatically prevents common bugs like null pointer dereferences, dangling pointers (accessing memory that has been freed), and data races in concurrent code.
-   **No Garbage Collector:** Aero doesn't need a background process to clean up memory, which means more predictable performance and no garbage collection pauses.
-   **Efficient Resource Management:** The ownership system isn't just for memory; it can manage any resource that needs to be cleaned up (files, network sockets, etc.).

### The Three Rules of Ownership

Aero's ownership system follows three main rules:

1.  **Each value in Aero has a variable that’s its *owner*.**
2.  **There can only be one owner at a time.**
3.  **When the owner goes out of scope, the value will be *dropped* (deallocated/cleaned up).**

Let's break these down.

## Move Semantics: Transferring Ownership

When you assign a variable that owns data on the heap (like a `String`) to another variable, or pass it to a function, ownership is *moved*. The original variable is no longer considered the owner and cannot be used afterwards. This prevents accidental "double-free" errors where two variables might try to free the same memory.

### Example: Simple Move

```aero
fn main() {
    let s1 = String::from("hello"); // s1 owns the String data.
                                    // The String data is allocated on the heap.

    let s2 = s1; // Ownership of the String data is MOVED from s1 to s2.
                 // s1 is now invalidated and cannot be used.

    // If you try to use s1 here, the Aero compiler will give you an error:
    // io::println(s1); // Compile-time error: use of moved value `s1`

    io::println(s2); // This is fine, s2 is the current owner.
} // s2 goes out of scope. The String data it owns is dropped (memory is freed).
  // s1 does nothing on scope exit because it no longer owns the data.
```

### Example: Ownership and Functions

Passing a value to a function also moves ownership.

```aero
// This function takes ownership of the String passed to it.
fn takes_ownership(some_string: String) {
    io::println(some_string);
} // `some_string` goes out of scope here, and the String data it owns is dropped.

fn main() {
    let text = String::from("Aero is fun!");
    takes_ownership(text); // `text`'s ownership is moved into the `takes_ownership` function.

    // io::println(text); // Compile-time error: `text` was moved and is no longer valid here.
}
```
Similarly, when a function returns an owned value, ownership is transferred to the variable that receives the result.

### Exception: `Copy` Types

Some types, like basic integers (`i32`, `i64`), floating-point numbers (`f64`), booleans (`bool`), and characters (`char`), are considered `Copy` types. When you assign these types or pass them to functions, a bitwise copy of their data is made, and the original variable remains valid. Ownership transfer (move semantics) doesn't apply to them because they live entirely on the stack and are cheap to copy.

```aero
fn main() {
    let x = 5; // x is an i32, which is a Copy type.
    let y = x; // y gets a copy of x's value. x is still valid.

    io::println(x); // Prints 5
    io::println(y); // Prints 5

    print_integer(x);
    io::println(x); // x is still valid after being passed to the function.
}

fn print_integer(num: i32) {
    // io::println(num); // Assuming a way to print numbers
}
```

## Borrowing and References

Constantly transferring ownership can be cumbersome if you just want to use a value without taking ownership of it. This is where *borrowing* comes in. Borrowing allows you to create *references* to a value. A reference is like a pointer that allows you to access the data without owning it.

### Immutable References (`&T`)

You can create an immutable reference to a value using the `&` operator. Immutable references allow you to read the data but not change it.

```aero
fn calculate_length(s: &String) -> usize { // `s` is an immutable reference to a String
    s.len() // You can read the length
    // s.push_str(" world"); // This would be an error, cannot modify through an immutable reference
}

fn main() {
    let message = String::from("Hello Aero");

    // Pass an immutable reference to `calculate_length`.
    // `message` is borrowed, but `main` still owns it.
    let length = calculate_length(&message);

    // io::println(length); // Assuming print for usize
    io::println(message); // `message` is still valid here.
}
```
A key rule for immutable borrows: **You can have multiple immutable references to the same piece of data simultaneously.**

```aero
fn main() {
    let data = String::from("shared data");
    let r1 = &data;
    let r2 = &data;

    io::println(*r1); // Use * to dereference the reference and get the value
    io::println(*r2);
    // Both r1 and r2 can coexist and read `data`.
}
```

### Mutable References (`&mut T`)

If you need to modify the data, you can create a mutable reference using `&mut`.

```aero
fn append_text(text: &mut String, to_append: &str) {
    text.push_str(to_append); // We can modify `text` because it's a mutable reference.
}

fn main() {
    let mut my_string = String::from("Start"); // Variable must be `mut` to allow mutable borrows.

    io::println(my_string); // Prints "Start"

    append_text(&mut my_string, " and Finish");

    io::println(my_string); // Prints "Start and Finish"
}
```
A very important rule for mutable borrows: **You can only have one mutable reference to a particular piece of data in a particular scope.** This rule helps prevent *data races* at compile time. A data race can occur when:
1. Two or more pointers access the same data at the same time.
2. At least one of the pointers is being used to write to the data.
3. There’s no mechanism being used to synchronize access to the data.

Aero prevents this by ensuring that if you have a mutable reference (`&mut T`), no other references (mutable or immutable) to that data can exist at the same time.

### The Borrowing Rules Summarized

1.  **One mutable reference OR any number of immutable references:** In any given scope, for a particular piece of data, you can have:
    *   Exactly one mutable reference (`&mut T`), OR
    *   Any number of immutable references (`&T`).
    *   You cannot have both a mutable reference and any other reference (mutable or immutable) at the same time.
2.  **References must always be valid:** References must not outlive the data they point to. This prevents *dangling references*.

## The Borrow Checker

The Aero compiler has a component called the **borrow checker**. Its job is to analyze your code at compile time and ensure that all ownership and borrowing rules are strictly followed. If any rule is violated, the compiler will produce an error, and your code won't compile. This might seem strict at first, but it's what guarantees memory safety.

### Common Errors Caught by the Borrow Checker

1.  **Multiple Mutable Borrows in the Same Scope:**
    ```aero
    /*
    fn main() {
        let mut s = String::from("data");
        let r1 = &mut s;
        let r2 = &mut s; // Compile-time error! Cannot have a second mutable borrow.
        r1.push_str(" one");
        r2.push_str(" two");
        io::println(s);
    }
    */
    ```

2.  **Mutable and Immutable Borrows Simultaneously:**
    ```aero
    /*
    fn main() {
        let mut s = String::from("data");
        let r1 = &s;       // Immutable borrow
        let r2 = &mut s;   // Compile-time error! Cannot have a mutable borrow while an immutable one exists.
        // r2.push_str(" modified");
        // io::println(*r1);
    }
    */
    ```

3.  **Use After Move:** (This is an ownership error, but related as borrowing avoids moves)
    ```aero
    /*
    fn main() {
        let s1 = String::from("owned string");
        let s2 = s1; // s1 is moved
        // io::println(s1); // Compile-time error! s1 is no longer valid.
    }
    */
    ```

## Dangling References

A dangling reference is a reference that points to memory that has been deallocated or is no longer valid. Accessing data through a dangling reference can lead to crashes or undefined behavior. Aero's ownership and borrowing rules, especially lifetimes, prevent dangling references from ever occurring at runtime.

Consider this function that tries to return a reference to data that only exists inside the function:
```aero
/*
fn dangle() -> &String { // This function wants to return a reference to a String
    let s = String::from("temporary"); // s is created inside dangle
    &s // We try to return a reference to s
} // s goes out of scope here, and the String is dropped. Its memory is freed.
  // If Aero allowed this, the returned reference would be dangling!

fn main() {
    // let reference_to_nothing = dangle(); // This would be a compile-time error.
    // The borrow checker would report that `s` does not live long enough.
}
*/
```
Aero's compiler will prevent `dangle()` from compiling because the value `s` is local to `dangle` and will be dropped when `dangle` finishes. Returning a reference to it would mean the reference could be used after `s` is gone.

## Lifetimes: Ensuring References Stay Valid (Conceptual Introduction)

So, how does the borrow checker know if a reference will always be valid? It uses a concept called **lifetimes**. Lifetimes are scopes for which references are guaranteed to be valid.

-   For most cases, the compiler is smart enough to figure out lifetimes automatically. This is called **lifetime elision**. You won't see explicit lifetime syntax in much of the code you write initially.
-   When lifetimes could be ambiguous (often in functions that take multiple references or return references), Aero requires you to annotate them explicitly. This tells the compiler how the lifetimes of different references relate to each other.

We won't dive deep into explicit lifetime syntax in this core tutorial, as it's a more advanced topic. For now, understand that lifetimes are Aero's mechanism for ensuring that no reference outlives the data it points to.

A very brief glimpse (syntax might vary in Aero):
```aero
/*
// `get_longer` takes two string slices that live at least as long as lifetime 'a,
// and returns a string slice that also lives as long as 'a.
fn get_longer<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
*/
```
This ensures that the returned reference is valid as long as the shorter of the two input references is valid.

## Summary

Ownership and borrowing are foundational to Aero's promise of safety and performance.
-   **Ownership** means every value has one owner, and data is dropped when the owner goes out of scope.
-   **Move semantics** transfer ownership by default for heap-allocated data.
-   **`Copy` types** (like integers) are copied instead of moved.
-   **Borrowing** allows you to use data without taking ownership via references (`&T` for immutable, `&mut T` for mutable).
-   The **borrowing rules** (one mutable OR many immutable references; references must be valid) are enforced by the **borrow checker** at compile time.
-   These rules prevent **dangling references** and other memory safety bugs.
-   **Lifetimes** are the mechanism (often inferred) that ensures references are valid.

While these rules might take a little getting used to, they provide strong guarantees and eliminate many common bugs found in other languages. As you write more Aero code, the ownership and borrowing system will become more intuitive.

In the next tutorial, we'll look at more complex data structures like structs and enums, and see how ownership and borrowing apply to them.
