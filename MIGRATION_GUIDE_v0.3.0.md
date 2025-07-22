# Migration Guide: Aero v0.2.0 → v0.3.0

This guide helps you migrate your Aero code from version 0.2.0 to 0.3.0, which introduces Phase 3 core language features.

## Overview of Changes

Version 0.3.0 introduces significant new language features while maintaining backward compatibility for basic arithmetic operations. The main additions are:

- Function definitions and calls
- Control flow statements (if/else, loops)
- I/O operations (print!/println!)
- Enhanced type system with new operators
- Variable mutability control
- Advanced scoping

## Breaking Changes

### 1. Print Statement Syntax

**Before (v0.2.0):**
```aero
// Print statements were not available
let result = 5 + 3;
// No way to output results
```

**After (v0.3.0):**
```aero
let result = 5 + 3;
println!("Result: {}", result);
print!("Value: ");
println!("{}", result);
```

**Migration:** Add `println!()` or `print!()` macro calls for output operations.

### 2. Function Definitions (New Feature)

**Before (v0.2.0):**
```aero
// Functions were not supported
let a = 5;
let b = 3;
let sum = a + b;
```

**After (v0.3.0):**
```aero
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn main() {
    let sum = add(5, 3);
    println!("Sum: {}", sum);
}
```

**Migration:** Wrap your main logic in a `main()` function and extract reusable code into separate functions.

### 3. Variable Mutability

**Before (v0.2.0):**
```aero
let x = 5;
x = 10;  // This worked in v0.2.0
```

**After (v0.3.0):**
```aero
let mut x = 5;  // Must explicitly declare as mutable
x = 10;         // Now this works

// Or use immutable variables
let y = 5;      // Cannot be reassigned
```

**Migration:** Add `mut` keyword to variables that need to be reassigned.

### 4. Enhanced Type System

**Before (v0.2.0):**
```aero
let a = 5;
let b = 3;
let result = a + b;  // Only basic arithmetic
```

**After (v0.3.0):**
```aero
let a = 5;
let b = 3;

// All these are now available:
let sum = a + b;
let equal = (a == b);
let greater = (a > b);
let logical_and = (a > 0) && (b > 0);
let logical_not = !(a == b);
```

**Migration:** No changes needed for existing arithmetic, but you can now use comparison and logical operators.

## New Features You Can Adopt

### 1. Control Flow

Add conditional logic and loops to your programs:

```aero
fn main() {
    let x = 10;
    
    // If/else statements
    if x > 5 {
        println!("x is greater than 5");
    } else {
        println!("x is 5 or less");
    }
    
    // While loops
    let mut i = 0;
    while i < 5 {
        println!("Count: {}", i);
        i = i + 1;
    }
    
    // For loops
    for j in 0..3 {
        println!("For loop: {}", j);
    }
    
    // Infinite loops with break/continue
    let mut counter = 0;
    loop {
        counter = counter + 1;
        if counter == 3 {
            continue;
        }
        if counter > 5 {
            break;
        }
        println!("Loop counter: {}", counter);
    }
}
```

### 2. Function Organization

Organize your code into reusable functions:

```aero
fn calculate_area(width: i32, height: i32) -> i32 {
    return width * height;
}

fn calculate_perimeter(width: i32, height: i32) -> i32 {
    return 2 * (width + height);
}

fn print_rectangle_info(width: i32, height: i32) {
    let area = calculate_area(width, height);
    let perimeter = calculate_perimeter(width, height);
    
    println!("Rectangle {}x{}", width, height);
    println!("Area: {}", area);
    println!("Perimeter: {}", perimeter);
}

fn main() {
    print_rectangle_info(5, 3);
    print_rectangle_info(10, 7);
}
```

### 3. Enhanced I/O

Add formatted output to your programs:

```aero
fn main() {
    let name = "Aero";
    let version = 3;
    let pi = 3.14159;
    
    // Simple output
    println!("Hello, World!");
    
    // Formatted output
    println!("Welcome to {} v0.{}!", name, version);
    println!("Pi is approximately {}", pi);
    
    // Multiple values
    let a = 15;
    let b = 4;
    println!("{} + {} = {}", a, b, a + b);
    println!("{} * {} = {}", a, b, a * b);
    println!("{} / {} = {}", a, b, a / b);
    println!("{} % {} = {}", a, b, a % b);
}
```

## Step-by-Step Migration Process

### Step 1: Update Your Build Environment

1. Update your Aero compiler to v0.3.0:
```bash
cd aero/src/compiler
git pull origin main
cargo build --release
```

2. Verify the new version:
```bash
aero --version  # Should show v0.3.0
```

### Step 2: Wrap Existing Code in main()

If you have existing v0.2.0 code like:
```aero
let x = 5;
let y = 10;
let result = x + y;
```

Wrap it in a main function:
```aero
fn main() {
    let x = 5;
    let y = 10;
    let result = x + y;
    println!("Result: {}", result);
}
```

### Step 3: Add Mutability Keywords

Review your code for variable reassignments and add `mut` where needed:

```aero
fn main() {
    let mut counter = 0;  // Add 'mut' for variables that change
    let max = 10;         // Keep immutable for constants
    
    while counter < max {
        counter = counter + 1;  // This requires 'mut'
        println!("Counter: {}", counter);
    }
}
```

### Step 4: Add Output Statements

Replace implicit output with explicit print statements:

```aero
fn main() {
    let calculation = 5 * 3 + 2;
    println!("Calculation result: {}", calculation);  // Add explicit output
}
```

### Step 5: Test Your Migrated Code

1. Compile your migrated code:
```bash
aero build your_program.aero -o your_program
```

2. Run and verify the output:
```bash
aero run your_program.aero
```

## Common Migration Issues

### Issue 1: Missing main() Function

**Error:** `No main function found`

**Solution:** Wrap your code in a `main()` function:
```aero
fn main() {
    // Your existing code here
}
```

### Issue 2: Immutable Variable Assignment

**Error:** `Cannot assign to immutable variable`

**Solution:** Add `mut` keyword:
```aero
let mut variable_name = initial_value;
```

### Issue 3: Missing Output

**Problem:** Your program compiles but produces no output

**Solution:** Add print statements:
```aero
println!("Your result: {}", result);
```

### Issue 4: Function Call Syntax

**Error:** `Unexpected token` when calling functions

**Solution:** Ensure proper function call syntax:
```aero
let result = function_name(arg1, arg2);  // Correct
// Not: let result = function_name arg1, arg2;  // Incorrect
```

## Example Migration

Here's a complete example showing before and after:

### Before (v0.2.0)
```aero
let principal = 1000;
let rate = 5;
let time = 2;
let interest = principal * rate * time / 100;
let amount = principal + interest;
```

### After (v0.3.0)
```aero
fn calculate_simple_interest(principal: i32, rate: i32, time: i32) -> i32 {
    return principal * rate * time / 100;
}

fn main() {
    let principal = 1000;
    let rate = 5;
    let time = 2;
    
    let interest = calculate_simple_interest(principal, rate, time);
    let amount = principal + interest;
    
    println!("Principal: {}", principal);
    println!("Rate: {}%", rate);
    println!("Time: {} years", time);
    println!("Interest: {}", interest);
    println!("Total Amount: {}", amount);
    
    // Demonstrate new control flow features
    if amount > 1000 {
        println!("Investment grew!");
    } else {
        println!("No growth");
    }
}
```

## Benefits of Migration

After migrating to v0.3.0, you'll gain access to:

1. **Better Code Organization:** Functions allow you to break code into reusable pieces
2. **Interactive Programs:** I/O operations let you create programs that communicate with users
3. **Complex Logic:** Control flow enables sophisticated program behavior
4. **Enhanced Safety:** Explicit mutability prevents accidental variable modifications
5. **Better Debugging:** Print statements help you understand program execution

## Getting Help

If you encounter issues during migration:

1. Check the [Troubleshooting Guide](TROUBLESHOOTING.md)
2. Review the [example programs](examples/) for reference
3. Run the test suite to ensure your environment is working:
   ```bash
   ./test_compiler.sh  # Linux/macOS
   test_compiler.bat   # Windows
   ```
4. Open an issue on GitHub if you find bugs or need assistance

## What's Next

After successfully migrating to v0.3.0, you can look forward to Phase 4 features:

- Data structures (arrays, structs, enums)
- Pattern matching
- Advanced type system features
- Module system

Stay tuned for the next migration guide when Phase 4 is released!