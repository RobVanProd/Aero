# Tutorial 4: Data Structures in Aero - Structs, Enums, and More

In this tutorial, we'll explore how to create more complex data structures in Aero using `struct`s and `enum`s. We'll also get a first look at some basic collection types provided by the Aero standard library and have a conceptual introduction to `trait`s, which define shared behavior.

## Structs

A `struct` (short for structure) is a custom data type that lets you package together and name multiple related values that make up a meaningful group.

### Defining Structs with Named Fields

You define a struct using the `struct` keyword, followed by the name of the struct and a set of curly braces containing the names and types of its fields.

```aero
// Defining a struct to represent a User
struct User {
    username: String,
    email: String,
    sign_in_count: u64,
    active: bool,
}

// Defining a struct for a 2D point
struct Point {
    x: f64,
    y: f64,
}
```

### Creating Instances of Structs (Instantiation)

Once you've defined a struct, you can create instances of it.

```aero
fn main() {
    // Create an instance of the User struct
    let mut user1 = User {
        email: String::from("user@example.com"),
        username: String::from("aero_user123"),
        active: true,
        sign_in_count: 1,
    };

    // Create an instance of the Point struct
    let origin = Point { x: 0.0, y: 0.0 };
}
```
The order of fields doesn't have to match the definition when instantiating, but all fields must be specified unless default values are supported (a more advanced feature).

### Accessing and Modifying Struct Fields

You can access the fields of a struct instance using dot notation (`.`). If the struct instance is mutable (declared with `let mut`), you can also modify its fields.

```aero
fn main() {
    let mut user1 = User {
        email: String::from("user@example.com"),
        username: String::from("aero_user123"),
        active: true,
        sign_in_count: 1,
    };

    io::print("Username: ");
    io::println(user1.username); // Accessing username

    user1.email = String::from("new_email@example.com"); // Modifying email
    user1.active = false;

    io::print("New Email: ");
    io::println(user1.email);
    // io::println(user1.active); // Assuming a way to print booleans
}
```

### Tuple Structs (Conceptual)

Aero might also support tuple structs, which are like tuples but with a name. They have fields that are typed but not named explicitly.

```aero
/* Conceptual:
struct Color(i32, i32, i32); // RGB
struct Coordinates(f64, f64);

fn main() {
    let black = Color(0, 0, 0);
    let point = Coordinates(10.5, -3.2);

    // Accessing fields by index:
    // io::println(black.0); // Accesses the first field
    // io::println(point.1); // Accesses the second field
}
*/
```
Tuple structs are useful when the names of the fields would be redundant, but you still want a distinct type.

### Unit-Like Structs (Conceptual)

You can also define structs that don't have any fields at all. These are called unit-like structs because they behave similarly to the `unit` type `()`. They are useful when you need to implement a trait on some type but don’t have any data that you want to store in the type itself.

```aero
/* Conceptual:
struct AlwaysEqual; // A unit-like struct

fn main() {
    let subject = AlwaysEqual;
    // You might use `subject` when you need a type, for example,
    // to associate with a trait implementation.
}
*/
```

### Methods on Structs

Methods are functions that are associated with a struct (or enum, or trait). They are defined within an `impl` (implementation) block. The first parameter of a method is always `self`, `&self`, or `&mut self`, which represents the instance of the struct the method is being called on.

-   `&self`: Borrows the instance immutably (read-only).
-   `&mut self`: Borrows the instance mutably (read-write).
-   `self`: Takes ownership of the instance (consumes it).

```aero
struct Rectangle {
    width: u32,
    height: u32,
}

// Implementation block for Rectangle
impl Rectangle {
    // A method that calculates the area of the rectangle
    // It borrows the Rectangle instance immutably.
    fn area(&self) -> u32 {
        self.width * self.height
    }

    // A method that checks if another rectangle can fit inside this one
    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }

    // An "associated function" that creates a square Rectangle.
    // It doesn't take `self` because it's called on the type itself (e.g., Rectangle::square).
    // These are often used as constructors.
    fn square(size: u32) -> Rectangle {
        Rectangle { width: size, height: size }
    }

    // A method that scales the rectangle (mutable)
    fn scale(&mut self, factor: u32) {
        self.width = self.width * factor;
        self.height = self.height * factor;
    }
}

fn main() {
    let rect1 = Rectangle { width: 30, height: 50 };
    let rect2 = Rectangle { width: 10, height: 40 };
    let mut rect3 = Rectangle::square(20); // Call associated function

    // io::println(rect1.area()); // Assuming print for u32
    // io::println(rect1.can_hold(&rect2)); // Assuming print for bool

    rect3.scale(2);
    // io::println(rect3.area()); // Area should now be (20*2) * (20*2) = 1600
}
```

### Structs and Ownership

Structs own their data by default. If a struct field is an owned type (like `String` or `Vec<T>`), that data is part of the struct. When the struct instance goes out of scope, its owned data is also dropped.

```aero
struct Article {
    title: String,
    author: String,
    content: String,
} // When an Article instance is dropped, its String fields are also dropped.

// If a struct needs to store references, it would use lifetimes (see Tutorial 3):
/* Conceptual:
struct ArticleView<'a> {
    title: &'a str,
    excerpt: &'a str,
}
*/
```

## Enums (Enumerations)

Enums allow you to define a type by enumerating its possible *variants*.

### Defining Enums

```aero
enum Message {
    Quit,
    WriteText(String), // Variant with associated String data
    Move { x: i32, y: i32 }, // Variant with named associated data (like a mini-struct)
    ChangeColor(i32, i32, i32), // Variant with tuple-like associated data
}
```
Each name listed is a *variant* of the `Message` enum.

### Enum Variants with Associated Data

Enum variants can store data. For example:
-   `Message::WriteText` holds a `String`.
-   `Message::Move` holds an anonymous struct with `x` and `y` fields.
-   `Message::ChangeColor` holds three `i32` values.
-   `Message::Quit` has no associated data.

A very common enum is `Option<T>`, which represents an optional value (either some value or nothing). It's often part of the `core` library:
```aero
/* Conceptual:
enum Option<T> {
    Some(T), // Represents some value of type T
    None,    // Represents no value
}
*/
```
Another common one is `Result<T, E>` for error handling:
```aero
/* Conceptual:
enum Result<T, E> {
    Ok(T),   // Represents a successful result of type T
    Err(E),  // Represents an error of type E
}
*/
```

### Using Enums with `match`

The `match` keyword (if planned for Aero, similar to Rust) is a powerful control flow construct that works well with enums. It allows you to compare a value against a series of patterns and then execute code based on which pattern matches.

```aero
fn process_message(msg: Message) {
    match msg { // Assuming `match` syntax
        Message::Quit => {
            io::println("The Quit variant has no data to destructure.");
        }
        Message::WriteText(text) => { // `text` binds to the String value
            io::print("Message: ");
            io::println(text);
        }
        Message::Move { x, y } => { // Destructure the fields
            // io::println("Move in x direction {} and y direction {}", x, y); // Needs formatting
            io::println("Move command received.");
        }
        Message::ChangeColor(r, g, b) => { // Destructure the tuple values
            // io::println("Change color to red {}, green {}, and blue {}", r, g, b); // Needs formatting
            io::println("Color change command.");
        }
    }
}

fn main() {
    let m1 = Message::WriteText(String::from("Hello from enum!"));
    let m2 = Message::Move{ x: 10, y: -5 };
    let m3 = Message::Quit;

    process_message(m1);
    process_message(m2);
    process_message(m3);
}
```
If `match` is not yet available, you might use `if let` or methods on the enum to access data, though `match` is the most idiomatic way to handle enums.

## Basic Collections (from Standard Library)

Aero's standard library (as proposed in RFCs) will provide common data structures like vectors and hash maps. These are essential for many programs.

### `Vec<T>`: Dynamic Arrays

A `Vec<T>` (vector) is a resizable array that holds elements of type `T`. It's stored on the heap.

```aero
// Assuming Vec is available from std::collections or similar
// use std::collections::Vec; // Or however it's imported

fn main() {
    // Create a new, empty vector of i32 integers
    let mut numbers: Vec<i32> = Vec::new();

    // Add elements
    numbers.push(10);
    numbers.push(20);
    numbers.push(30);

    // Access elements by index (returns Option<&T> or panics/errors on out of bounds)
    let first = numbers.get(0); // Conceptual: first might be Option<&i32>
    match first {               // Assuming Option and match
        Some(value_ref) => { /* io::println(*value_ref) */ }
        None => { io::println("No element at index 0"); }
    }

    // A more direct access (might panic if index is out of bounds in some languages)
    // let second_val = numbers[1]; // Accessing by index (syntax may vary)
    // io::println(second_val);

    // Iterate over elements (conceptual `for` loop)
    /*
    for num_ref in numbers.iter() { // iter() usually returns an iterator over &T
        io::println(*num_ref);
    }
    */
    io::println("Vector operations demonstrated.");
}
```

### `HashMap<K, V>`: Hash Maps

A `HashMap<K, V>` stores key-value pairs. It uses a hash function to determine how to store and retrieve data, allowing for efficient lookups. `K` must be a hashable type.

```aero
// Assuming HashMap is available from std::collections or similar
// use std::collections::HashMap;

fn main() {
    let mut scores: HashMap<String, i32> = HashMap::new();

    // Insert key-value pairs
    scores.insert(String::from("Alice"), 100);
    scores.insert(String::from("Bob"), 95);

    // Retrieve a value by key (returns Option<&V>)
    let alice_score_option = scores.get(&String::from("Alice"));
    match alice_score_option { // Assuming Option and match
        Some(score_ref) => { /* io::println(*score_ref); */ }
        None => { io::println("Alice not found."); }
    }

    // Check if a key exists
    // let has_bob = scores.contains_key(&String::from("Bob")); // Assuming print for bool
    // io::println(has_bob);

    io::println("HashMap operations demonstrated.");
}
```
These collection types own their data. For example, `Vec<String>` owns all the strings it contains. When the collection is dropped, it also drops all its elements.

## Traits: Shared Behavior (Conceptual Introduction)

A `trait` defines a set of methods that a type can implement, enabling types to share common behaviors. It's similar to interfaces in other languages.

### Defining a Trait

```aero
// Trait definition
trait Summary {
    // Method signature: requires implementors to define this method
    fn summarize(&self) -> String; // Takes an immutable reference to self

    // Traits can also have default method implementations
    fn default_summary(&self) -> String {
        String::from("(Read more...)")
    }
}
```

### Implementing a Trait for a Type

You use an `impl TraitName for TypeName` block to implement a trait's methods for a specific type.

```aero
struct NewsArticle {
    headline: String,
    author: String,
    content: String,
}

impl Summary for NewsArticle {
    fn summarize(&self) -> String {
        // A simple summary: headline by author
        // String concatenation or formatting would be used here.
        // return format!("{} by {}", self.headline, self.author);
        return self.headline + " by " + self.author; // Assuming basic string concat for example
    }
}

struct Tweet {
    username: String,
    text: String,
    retweets: u32,
}

impl Summary for Tweet {
    fn summarize(&self) -> String {
        // return format!("@{}: {}", self.username, self.text);
        return "@" + self.username + ": " + self.text; // Basic concat
    }

    // We can override default methods too
    fn default_summary(&self) -> String {
        String::from("View tweet details...")
    }
}
```

### Using Traits

Once a trait is implemented, you can call its methods on instances of that type. Traits are also crucial for generics, allowing you to write functions that accept any type implementing a specific trait (see Tutorial 2 on Trait Bounds).

```aero
fn notify_summary(item: &dyn Summary) { // Takes any type that implements Summary (dynamic dispatch concept)
    io::print("Breaking News! Summary: ");
    io::println(item.summarize());
}

fn main() {
    let article = NewsArticle {
        headline: String::from("Aero Reaches Milestone"),
        author: String::from("A. Developer"),
        content: String::from("Aero is progressing well..."),
    };

    let tweet = Tweet {
        username: String::from("aerodev"),
        text: String::from("Just released a new tutorial! #aero"),
        retweets: 10,
    };

    io::println(article.summarize());
    io::println(tweet.summarize());
    io::println(article.default_summary());
    io::println(tweet.default_summary());

    // notify_summary(&article); // Using trait for polymorphism (conceptual)
    // notify_summary(&tweet);
}
```
This was a brief introduction. Traits are a powerful feature used extensively in Aero for abstraction and code reuse.

## Summary

This tutorial covered:
-   **Structs**: Named field, tuple, and unit-like structs; instantiation, field access, and methods via `impl`.
-   **Enums**: Defining variants, variants with associated data, and using `match` (conceptually) for control flow.
-   **Basic Collections**: A brief look at `Vec<T>` and `HashMap<K, V>` from the standard library.
-   **Traits**: A conceptual introduction to defining shared behavior with `trait` and `impl`.

These data structures and abstraction mechanisms are fundamental to building robust and maintainable applications in Aero. In upcoming tutorials, we'll see how these interact more deeply with other features like error handling, modules, and advanced generics.
