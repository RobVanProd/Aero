# Aero Language Specification - Ownership and Borrowing Model

## Introduction

This document formally specifies Aero's ownership and borrowing model, a core design principle inspired by Rust that ensures memory safety and concurrency without a garbage collector. This model dictates how resources (primarily memory) are managed and accessed, preventing common programming errors such as null pointer dereferences, double-frees, and data races at compile time.

## Ownership

In Aero, every value has a single *owner*. The owner is a variable that binds to the value. When the owner goes out of scope, the value is *dropped*, and its resources are automatically deallocated. This deterministic destruction prevents memory leaks and ensures efficient resource management without needing a garbage collector.

"Going out of scope" refers to the point where a variable is no longer valid. For function parameters, this is at the end of the function. For local variables, it's typically at the end of the block (`{}`) in which they are declared.

### Rules of Ownership:

1.  **Each value in Aero has a variable that’s its owner.**
2.  **There can only be one owner at a time.** If a value is assigned to another variable, or passed to a function, ownership is *moved*.
3.  **When the owner goes out of scope, the value will be dropped.** This means its memory is deallocated, and any other cleanup logic (like closing files or releasing locks) associated with it is run.

### Ownership Transfer (Move Semantics)

Move semantics are the default for types that manage resources, such as heap-allocated strings or custom structs owning file handles. When such a value is assigned to another variable, passed to a function, or returned from a function, ownership is transferred. The original variable binding becomes invalid to prevent accidental use of a moved value (double-free protection).

**Example: Simple Move**
```aero
fn main() {
    let s1 = String::from("hello"); // s1 owns the String data
    let s2 = s1;                   // Ownership of the String data moves from s1 to s2.
                                   // s1 is no longer valid and cannot be used.
    // print(s1); // This would be a compile-time error: use of moved value s1
    print(s2);   // s2 is valid and owns the data.
} // s2 goes out of scope, the String data is dropped.
```

**Example: Ownership and Functions**
```aero
// This function takes ownership of the string passed to it.
fn take_ownership(some_string: String) {
    print(some_string);
} // `some_string` goes out of scope here, and the String data it owns is dropped.

// This function takes ownership of a string and returns ownership of a (possibly different) string.
fn take_and_return_ownership(a_string: String) -> String {
    print(a_string); // a_string is owned by this function now
    a_string // Ownership is returned to the caller
}

fn main() {
    let s1 = String::from("Aero");
    take_ownership(s1); // s1's ownership moves into the function.
    // print(s1); // Compile-time error: s1 was moved.

    let s2 = String::from("rocks");
    let s3 = take_and_return_ownership(s2); // s2 moves in, s3 takes ownership of the result.
    // print(s2); // Compile-time error: s2 was moved.
    print(s3);   // s3 is valid.
} // s3 goes out of scope and is dropped.
```

### Copy Types

Some types in Aero, typically simple scalar values that live entirely on the stack (like integers, booleans, characters, floating-point numbers, and tuples/arrays of Copy types), do not move. Instead, they are *copied* when assigned or passed to functions. This is because copying them is inexpensive, and move semantics would be unnecessarily restrictive.

A type can be a `Copy` type if all of its components are also `Copy` types. It generally means the type doesn't have any custom resource management logic on drop (no destructor).

```aero
fn main() {
    let x = 5;    // x is an i32, which is a Copy type
    let y = x;    // y gets a copy of the value of x. x is still valid.

    print(x);   // x is still valid and prints 5
    print(y);   // y also prints 5

    make_a_copy(x);
    print(x);   // x is still valid after being passed to the function.
}

fn make_a_copy(some_integer: i32) {
    print(some_integer);
} // some_integer goes out of scope, its copy is dropped.
```
If a type implements a special `Copy` trait (if Aero follows Rust's model), it will be copied instead of moved. Otherwise, types are moved.

## Borrowing

To allow access to a value without transferring ownership, Aero provides *borrowing*. This is achieved through *references*. A reference is like a pointer that allows you to access data owned by another variable, but it does not imply ownership. Borrowing is crucial for flexibility and efficiency, as it avoids unnecessary copying of data.

### Types of Borrows:

1.  **Immutable Borrows (`&T`)**:
    *   Allows read-only access to the data.
    *   Multiple immutable references to the same value can exist simultaneously.
    *   You can have any number of immutable borrows as long as no mutable borrow exists.

    ```aero
    fn calculate_length(s: &String) -> int { // s is an immutable reference to a String
        s.length()
    }

    fn main() {
        let s1 = String::from("Aero is cool");
        let len = calculate_length(&s1); // Pass an immutable reference to s1

        print("The string '{}' has length {}.", s1, len); // s1 is still valid and owned here.

        let r1 = &s1; // r1 is an immutable reference
        let r2 = &s1; // r2 is another immutable reference. This is allowed.
        print("r1 points to: {}", *r1); // Use * to dereference and get the value
        print("r2 points to: {}", *r2);
    } // s1 is dropped. r1 and r2 become invalid (but this is safe due to lifetime rules).
    ```

2.  **Mutable Borrows (`&mut T`)**:
    *   Allows read and write access to the data.
    *   Only one mutable reference to a value can exist in a particular scope.
    *   You cannot have any other references (immutable or mutable) to the same value while a mutable reference is active. This helps prevent data races at compile time.

    ```aero
    fn append_greeting(s: &mut String) { // s is a mutable reference to a String
        s.append(" greets you!");
    }

    fn main() {
        let mut s_mut = String::from("Aero"); // The variable must be mutable to allow mutable borrows

        let r_mut1 = &mut s_mut;
        // let r_mut2 = &mut s_mut; // Compile-time error: cannot borrow `s_mut` as mutable more than once at a time.
        // let r_immut = &s_mut;    // Compile-time error: cannot borrow `s_mut` as immutable because it is also borrowed as mutable.

        append_greeting(r_mut1); // Pass the mutable reference. r_mut1 itself can be used here.
                                 // If append_greeting took &mut s_mut directly: append_greeting(&mut s_mut);

        print(s_mut); // s_mut is valid, now "Aero greets you!"
                      // r_mut1 is no longer active here if its scope has ended or it's not used later.
    }
    ```
    Using a mutable reference:
    ```aero
    fn main() {
        let mut value = 10;
        let ref_to_value = &mut value;

        *ref_to_value += 5; // Dereference and modify the value

        print(value); // Prints 15
    }
    ```

### Rules of Borrowing (Enforced by the Borrow Checker):

The borrow checker in Aero enforces these rules at compile time. The primary goal is to prevent data races, which can occur when:
- Two or more pointers access the same data at the same time.
- At least one of the pointers is being used to write to the data.
- There’s no mechanism being used to synchronize access to the data.

Aero's rules are:

1.  **One Mutable Reference OR Any Number of Immutable References:**
    *   Within a given scope, you can have *either* one mutable reference (`&mut T`) to a particular piece of data, *or* any number of immutable references (`&T`). You cannot have both simultaneously.
    *   This is because mutable references imply that the data might change. If there were other references (even immutable ones) active at the same time, they might read inconsistent or incorrect data, or a write through the mutable reference could conflict with another write.
    *   The scope of a reference starts where it is introduced and continues until its last use. The concept of Non-Lexical Lifetimes (NLL), discussed later, refines how "last use" is determined, making the borrow checker more flexible.

    ```aero
    fn main() {
        let mut data = 10;

        let r1 = &data; // Immutable borrow - OK
        let r2 = &data; // Another immutable borrow - OK
        // let r_mut = &mut data; // COMPILE ERROR: Cannot borrow `data` as mutable because it is also borrowed as immutable by r1 and r2.
                                // This error would occur if r_mut was introduced while r1 and r2 were still considered live.
        print(*r1, *r2); // Last use of r1 and r2 for this example.

        // After r1 and r2 are no longer used (i.e., their lifetimes end), a mutable borrow is allowed.
        let r_mut_after = &mut data;
        *r_mut_after = 20;
        print(*r_mut_after); // Last use of r_mut_after.
    }
    ```

2.  **References Must Always Be Valid (No Dangling References):**
    *   A reference must not outlive the data it points to. The borrow checker ensures this primarily through lifetime analysis (discussed in the "Lifetimes" section).
    *   If Aero allowed references to exist longer than the data they refer to, this would lead to a "dangling reference" – a pointer to deallocated or invalid memory. Dereferencing a dangling reference can cause crashes, data corruption, or other undefined behavior. Aero prevents this at compile time.

### Common Error Conditions Prevented by the Borrow Checker:

The Aero compiler, through its borrow checker, will detect and report the following critical error conditions at compile time:

- **Multiple Mutable References**: Attempting to create more than one mutable reference to the same data at the same time.

  ```aero
  fn main() {
      let mut s = "hello";
      let r1 = &mut s;
      let r2 = &mut s; // Compile-time error: cannot borrow `s` as mutable more than once at a time
      print(r1);
  }
  ```

- **Mutable and Immutable References Simultaneously**: Attempting to create an immutable reference while a mutable reference to the same data is active, or vice-versa.

  ```aero
  fn main() {
      let mut s = "hello";
      let r1 = &mut s;
      let r2 = &s;    // Compile-time error: cannot borrow `s` as immutable because it is also borrowed as mutable
      print(r1);
  }

  fn main() {
      let mut s = "hello";
      let r1 = &s;
      let r2 = &mut s; // Compile-time error: cannot borrow `s` as mutable because it is also borrowed as immutable
      print(r1);
  }
  ```

- **Dangling References**: Attempting to create a reference that points to invalid memory, typically when the data being referred to has gone out of scope while the reference still exists.

  ```aero
  // Example of a function trying to return a dangling reference
  fn dangle() -> &int { // dangle returns a reference to an integer
      let x = 5;    // x is a new integer with value 5, local to dangle's scope
      &x            // We attempt to return a reference to x
  } // x goes out of scope here, and is dropped. Its memory is deallocated.
    // If this were allowed, the returned reference would point to invalid memory.

  fn main() {
      // let reference_to_nothing = dangle();
      // This line would cause a compile-time error: `x` does not live long enough.
      // Aero's borrow checker, through lifetime analysis, prevents this situation.
      // It determines that the lifetime of `x` is shorter than the lifetime expected for the returned reference.
  }
  ```
  Another scenario illustrating how dangling references are prevented:
  ```aero
  fn main() {
    let r: &int; // Declare a reference r

    { // Inner scope
        let x = 10; // x is created within this inner scope
        // r = &x;  // If this assignment were made here, r would point to x.
                    // However, x is about to go out of scope.
    } // x goes out of scope here and is dropped.

    // If `r = &x` was allowed in the inner scope, then using `r` here would be unsafe:
    // print(*r); // This would be dereferencing a dangling pointer.
                 // Aero's borrow checker will issue a compile-time error if `r = &x` is uncommented,
                 // stating that `x` does not live long enough for `r` to borrow it across scopes.
  }
  ```

- **Use After Move**: Attempting to use a variable after its ownership has been moved.

  ```aero
  fn main() {
      let s1 = "hello";
      let s2 = s1; // s1 is moved to s2
      print(s1); // Compile-time error: use of moved value `s1`
  }
  ```

## Lifetimes

Lifetimes are Aero's way of describing and reasoning about the scopes for which references are valid. They are a crucial part of how the borrow checker ensures that references do not outlive the data they point to, thus preventing dangling references at compile time.

Every reference has a lifetime, which is the scope for which that reference is guaranteed to be valid. Think of a lifetime as a contract that states a reference will only be used as long as the data it points to is also live.

In many cases, lifetimes are simple enough that the compiler can infer them automatically without needing explicit annotations from the programmer. This is known as **lifetime elision**. However, in more complex scenarios, especially with functions that take references as parameters or return references, you might need to use explicit lifetime annotations to help the compiler.

These annotations don't change how long any of the values live or exist. Rather, they describe the relationships of the lifetimes of multiple references to each other and to the data they point to. This allows the borrow checker to verify that all uses of references are safe. If the relationships cannot be satisfied, the code will not compile.

### Lifetime Elision Rules (Implicit Lifetimes):

Similar to Rust, Aero will have a set of lifetime elision rules that allow the compiler to infer lifetimes in common scenarios, reducing boilerplate.

### Explicit Lifetime Annotations:

When lifetime inference is ambiguous, explicit lifetime annotations are used, typically denoted with an apostrophe (`'`) followed by a lowercase name (e.g., `'a`).

```aero
fn longest<'a>(x: &'a string, y: &'a string) -> &'a string {
    if x.length() > y.length() {
        x
    } else {
        y
    }
}
```

## Non-Lexical Lifetimes (NLL) (Planned Feature)

Historically, some languages with borrow checking (like early versions of Rust) used purely *lexical lifetimes*. In a lexical system, the validity of a borrow is tied strictly to the lexical scope (the `{...}` block) where the reference variable is declared and is considered to last until the end of that scope. This can sometimes be overly restrictive, leading to valid and safe code being rejected because the borrow checker isn't precise enough about when a borrow is *actually* needed.

Aero plans to implement **Non-Lexical Lifetimes (NLL)**. NLL is a more advanced and precise way of determining how long a borrow is actually in use. Instead of a borrow lasting for the entire lexical scope of the reference variable, NLL understands that a borrow is only active from the point it's created until its last actual use in the code.

### Benefits of NLL:

1.  **More Ergonomic Code:** NLL allows more code patterns to compile without needing complex workarounds or explicit lifetime annotations. It makes the borrow checker feel more intuitive and less like an obstacle.
2.  **Reduced Need for Explicit Scopes:** Developers won't need to introduce as many artificial scopes (`{}`) just to satisfy the borrow checker when they know their code is safe.
3.  **Improved Precision:** The borrow checker can more accurately track the flow of borrows, leading to fewer false positives (i.e., rejecting safe code that doesn't violate memory safety). A borrow can end sooner, allowing other operations (like mutable borrows or modifications to the original data) to occur earlier.

### Example: Lexical vs. NLL

Consider the following scenario:

```aero
fn main() {
    let mut data = vec![1, 2, 3];
    let last_element_ref = &data[data.length() - 1]; // Immutable borrow of `data` starts here.

    // Under strict LEXICAL lifetimes:
    // `last_element_ref` (and thus the immutable borrow on `data`) would be considered
    // live until the end of its lexical scope (the end of the `main` function block).
    // Therefore, the following attempt to mutate `data` would be a compile-time error:
    // data.push(4); // ERROR: cannot borrow `data` as mutable because it is still borrowed as immutable by `last_element_ref`.

    print("Last element is: {}", *last_element_ref); // This is the last actual use of `last_element_ref`.

    // With Non-Lexical Lifetimes (NLL):
    // The borrow for `last_element_ref` is understood to end right after the `print!` statement above,
    // because `last_element_ref` is not used beyond that point.
    // So, this becomes valid under NLL:
    data.push(4); // OK with NLL! The immutable borrow ended, so a mutable borrow for `push` is allowed.
    print("Updated data: {:?}", data);
}
```
In a system with only lexical lifetimes, the `data.push(4)` line would be a compile-time error. With NLL, the compiler performs a more detailed analysis of the code's control flow and determines that the borrow for `last_element_ref` is no longer needed after it's printed. This allows the mutable borrow required by `data.push(4)` to proceed safely.

Aero's adoption of NLL aims to provide a smoother and more developer-friendly experience with its powerful ownership and borrowing system, making safe concurrency and memory management more accessible.

## Ownership and Borrowing with Data Structures

The principles of ownership, borrowing, moving, and copying apply consistently to more complex data structures like structs, enums, and collections.

### Structs

1.  **Owning Data:** Structs can own their data. When a struct owns its data (e.g., a field is a `String` rather than a `&str`), that data lives as long as the struct instance. When the struct instance goes out of scope, its owned data is dropped.

    ```aero
    struct Person {
        name: String,
        age: u32, // u32 is a Copy type
    }

    fn main() {
        let person = Person {
            name: String::from("Alice"),
            age: 30,
        };
        // `person` owns the String data for "Alice".
        // `age` is part of the struct directly (Copy type).

        let person2 = person; // Ownership of `name` (String) moves from `person` to `person2`.
                              // `age` is copied.
                              // `person.name` is no longer valid.
        // print(person.name); // Compile-time error: use of moved value (specifically person.name)
        print("Person 2 is {} aged {}", person2.name, person2.age);
    } // `person2` goes out of scope. `person2.name` (String) is dropped.
    ```

2.  **Borrowing Data (Structs with References):** Structs can also hold references to data owned by something else. In such cases, lifetimes must be used to ensure the referenced data outlives the struct instance holding the reference.

    ```aero
    struct Inspector<'a> { // Inspector holds a reference, so it needs a lifetime parameter.
        target_data: &'a String,
        id_number: u32,
    }

    fn main() {
        let important_data = String::from("Sensitive Information");
        let inspector_id = 101;

        let inspector_instance = Inspector {
            target_data: &important_data, // Borrows `important_data`
            id_number: inspector_id,
        };

        print("Inspector {} is looking at: {}", inspector_instance.id_number, inspector_instance.target_data);
        // `important_data` must outlive `inspector_instance`.
        // If `important_data` were dropped while `inspector_instance` was still live and holding a reference to it,
        // that reference would become dangling. Lifetimes prevent this.

    } // `inspector_instance` is dropped first, then `important_data` is dropped.
    ```

### Enums

Ownership rules also apply to data held within enum variants.

```aero
enum Message {
    Quit, // No data associated
    Update(String), // Owns a String
    Notify(&'a str), // Borrows a string slice (requires a lifetime 'a if part of a struct/generic context)
    ProcessId(u32), // Contains a Copy type
}

fn process_message(msg: Message) {
    match msg {
        Message::Quit => print("Quit message"),
        Message::Update(text) => {
            print("Update with owned text: {}", text);
            // `text` is owned by this scope now, will be dropped at the end of the match arm or function.
        },
        Message::Notify(slice) => {
            print("Notify with borrowed slice: {}", slice);
            // `slice` is a reference, data it points to is owned elsewhere.
        },
        Message::ProcessId(id) => print("Process ID: {}", id), // id is copied.
    }
}

fn main() {
    let owned_update = Message::Update(String::from("New data"));
    process_message(owned_update); // Ownership of the String inside `owned_update` moves into `process_message`.

    let borrowed_notification = Message::Notify("System alert");
    process_message(borrowed_notification); // The reference is copied, but the data it points to is not moved.
}
```

### Collections (Conceptual)

While specific collection types (like `Vec<T>`, `HashMap<K, V>`) will have their own detailed API documentation, they generally follow these ownership principles:
*   **Owned Elements:** Most collections will own their elements. For example, a `Vec<String>` owns all the `String` instances it contains. When the `Vec` is dropped, it will iterate through its elements and drop each one.
*   **Copying vs. Moving into Collections:** When you add an element to a collection, if the element's type is `Copy`, it will be copied into the collection. If it's a move type (like `String`), it will be moved into the collection, and you can no longer use the original variable.
*   **Borrowing from Collections:** Collections will provide methods to borrow elements (e.g., getting an element at an index typically returns a reference `&T` or `&mut T`). The borrow checker's rules apply to these references: you can have multiple immutable references or one mutable reference to elements within the collection. The collection itself is considered borrowed while references to its elements exist.

## Dropping Values and Resource Deallocation

When a value goes out of scope, Aero automatically calls a special function for that value's type, known as its *destructor* or `drop` function. This function allows the type to perform any necessary cleanup, such as deallocating memory, closing files, releasing locks, or decrementing reference counts.

### The `Drop` Trait (Conceptual)

Aero, similar to Rust, is expected to provide a `Drop` trait (or an equivalent mechanism). Types can implement this trait to define custom cleanup logic.

```aero
// Conceptual example
trait Drop {
    fn drop(&mut self); // This method is called when the value goes out of scope
}

struct MyFile {
    handle: FileHandle, // Represents an open file
    path: String,
}

impl Drop for MyFile {
    fn drop(&mut self) {
        // Custom cleanup logic: close the file handle
        print("Closing file: {}", self.path);
        // self.handle.close(); // Actual file closing operation
        // The memory for `self.path` (String) will be dropped automatically after this.
    }
}

fn main() {
    {
        let file_a = MyFile {
            handle: open_file_somehow("a.txt"),
            path: String::from("a.txt"),
        };
        // Do stuff with file_a
    } // `file_a` goes out of scope here. `MyFile::drop` is called, then its fields are dropped.

    let file_b = MyFile {
        handle: open_file_somehow("b.txt"),
        path: String::from("b.txt"),
    };
    // file_b will be dropped at the end of main.
}
```

### Order of Dropping

The order in which values are dropped is generally the reverse of the order in which they were declared within a given scope. For structs, fields are typically dropped in the order of their declaration before the struct's own `drop` logic (if any) is executed. This deterministic order is crucial for safe resource management, especially when one resource depends on another (e.g., ensuring data is flushed before a file handle is closed).

### Resource Management Beyond Memory

The drop mechanism is not just for memory. It's a general way to manage any resource that needs explicit deallocation or cleanup:
-   File handles
-   Network sockets
-   Database connections
-   Locks (like Mutex guards)
-   Pointers to resources managed by external C libraries

By ensuring every resource-owning type correctly implements its cleanup logic (often via `Drop`), Aero guarantees that these resources are released promptly and correctly when they are no longer needed, preventing leaks and other common programming errors.

## Conclusion

The ownership and borrowing model, enforced by the borrow checker and guided by lifetimes, is fundamental to Aero achieving memory safety and high performance without runtime overhead. By catching these critical errors at compile time, Aero aims to provide a robust and reliable development experience.


