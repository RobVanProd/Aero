# Aero Language Specification - Ownership and Borrowing Model

## Introduction

This document formally specifies Aero's ownership and borrowing model, a core design principle inspired by Rust that ensures memory safety and concurrency without a garbage collector. This model dictates how resources (primarily memory) are managed and accessed, preventing common programming errors such as null pointer dereferences, double-frees, and data races at compile time.

## Ownership

In Aero, every value has a single *owner*. When the owner goes out of scope, the value is dropped, and its resources are deallocated. This deterministic destruction prevents memory leaks and ensures efficient resource management.

### Rules of Ownership:

1. **Each value has a variable that's its owner.**
2. **There can only be one owner at a time.**
3. **When the owner goes out of scope, the value will be dropped.**

### Ownership Transfer (Move Semantics):

When a value is assigned to another variable or passed as an argument to a function, ownership is *moved*. The original owner can no longer access the value.

```aero
fn main() {
    let s1 = "hello"; // s1 owns "hello"
    let s2 = s1;      // Ownership of "hello" moves from s1 to s2. s1 is now invalid.
    // print(s1);    // Compile-time error: use of moved value
    print(s2);      // s2 is valid
}

fn take_ownership(s: string) {
    print(s);
} // s goes out of scope, "hello" is dropped

fn main() {
    let s3 = "world";
    take_ownership(s3); // Ownership of "world" moves to take_ownership
    // print(s3);    // Compile-time error: use of moved value
}
```

## Borrowing

To allow temporary access to a value without transferring ownership, Aero provides *borrowing*. This is achieved through references, which are pointers to a value that do not take ownership.

### Types of Borrows:

1. **Immutable Borrows (`&T`)**: Multiple immutable references to a value can exist simultaneously. These references allow reading the value but not modifying it.

   ```aero
   fn calculate_length(s: &string) -> int {
       s.length()
   }

   fn main() {
       let s = "example";
       let len = calculate_length(&s); // s is immutably borrowed
       print(s); // s is still valid
   }
   ```

2. **Mutable Borrows (`&mut T`)**: Only one mutable reference to a value can exist at any given time. This reference allows both reading and modifying the value.

   ```aero
   fn append_string(s: &mut string) {
       s.append(" world");
   }

   fn main() {
       let mut s = "hello";
       append_string(&mut s); // s is mutably borrowed
       print(s); // s is valid, now "hello world"
   }
   ```

### Rules of Borrowing (The 


Borrow Checker):

1. **At any given time, you can either have one mutable reference or any number of immutable references.**
2. **References must always be valid.**

### Error Conditions Detected by the Borrow Checker:

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
  fn dangle() -> &int {
      let x = 5; // x is created here
      &x         // We return a reference to x
  } // x goes out of scope here, and is dropped. The reference becomes dangling!

  fn main() {
      let reference_to_nothing = dangle(); // Compile-time error: `x` does not live long enough
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

Aero uses *lifetimes* to ensure that references are always valid. Lifetimes are a generic parameter that tells the compiler how long a reference is valid. In most cases, the compiler can infer lifetimes, but explicit lifetime annotations may be required for functions with multiple input lifetimes or when returning a reference.

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

## Conclusion

The ownership and borrowing model, enforced by the borrow checker and guided by lifetimes, is fundamental to Aero achieving memory safety and high performance without runtime overhead. By catching these critical errors at compile time, Aero aims to provide a robust and reliable development experience.


