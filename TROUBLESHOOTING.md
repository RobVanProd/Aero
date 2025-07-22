# Aero Troubleshooting Guide

This guide helps you resolve common issues when working with the Aero programming language, particularly with Phase 3 features including functions, control flow, I/O operations, and enhanced type checking.

## Installation Issues

### Compiler Not Found
**Problem:** `aero: command not found` or similar error.

**Solutions:**
1. Ensure `~/.cargo/bin` is in your PATH:
   ```bash
   echo $PATH | grep -q "$HOME/.cargo/bin" || echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   ```

2. Reinstall the compiler:
   ```bash
   cargo install --path src/compiler --force
   ```

3. Verify installation:
   ```bash
   aero --version
   ```

### LLVM Tools Missing
**Problem:** Compilation fails with LLVM-related errors.

**Solutions:**
- **Ubuntu/Debian:** `sudo apt install llvm clang`
- **macOS:** `brew install llvm`
- **Windows:** Download from [LLVM releases](https://releases.llvm.org/)

## Function-Related Issues

### Function Redefinition Error
**Problem:** `error: function 'name' is already defined`

**Solution:** Each function name must be unique within its scope. Rename one of the functions:
```aero
// ❌ Error: duplicate function names
fn calculate() -> i32 { return 5; }
fn calculate() -> i32 { return 10; }

// ✅ Correct: unique function names
fn calculate_basic() -> i32 { return 5; }
fn calculate_advanced() -> i32 { return 10; }
```

### Function Arity Mismatch
**Problem:** `error: function expects X arguments, found Y`

**Solution:** Ensure function calls match the parameter count:
```aero
fn add(a: i32, b: i32) -> i32 { return a + b; }

// ❌ Error: wrong number of arguments
let result = add(5);        // Too few
let result = add(5, 3, 2);  // Too many

// ✅ Correct: exact number of arguments
let result = add(5, 3);
```

### Return Type Mismatch
**Problem:** `error: expected return type 'i32', found 'bool'`

**Solution:** Ensure all return paths match the declared return type:
```aero
// ❌ Error: inconsistent return types
fn check_value(x: i32) -> i32 {
    if x > 0 {
        return true;  // Returns bool, not i32
    }
    return x;
}

// ✅ Correct: consistent return types
fn check_value(x: i32) -> i32 {
    if x > 0 {
        return 1;     // Returns i32
    }
    return 0;         // Returns i32
}
```

### Missing Return Statement
**Problem:** `error: function must return a value of type 'i32'`

**Solution:** Add explicit return or make the last expression return the value:
```aero
// ❌ Error: no return value
fn get_number() -> i32 {
    let x = 5;
    // Missing return
}

// ✅ Correct: explicit return
fn get_number() -> i32 {
    let x = 5;
    return x;
}

// ✅ Also correct: implicit return
fn get_number() -> i32 {
    let x = 5;
    x  // No semicolon = implicit return
}
```

## Control Flow Issues

### Break/Continue Outside Loop
**Problem:** `error: 'break' statement not in loop context`

**Solution:** Only use `break` and `continue` inside loops:
```aero
// ❌ Error: break outside loop
fn main() {
    if true {
        break;  // Not allowed here
    }
}

// ✅ Correct: break inside loop
fn main() {
    loop {
        if true {
            break;  // Allowed here
        }
    }
}
```

### Non-Boolean Condition
**Problem:** `error: expected 'bool', found 'i32'`

**Solution:** Ensure conditions evaluate to boolean values:
```aero
// ❌ Error: integer used as condition
let x = 5;
if x {  // x is i32, not bool
    println!("This won't work");
}

// ✅ Correct: boolean condition
let x = 5;
if x > 0 {  // x > 0 evaluates to bool
    println!("This works");
}
```

### Unreachable Code
**Problem:** `warning: unreachable code after return statement`

**Solution:** Remove or restructure code after return statements:
```aero
// ❌ Warning: unreachable code
fn example() -> i32 {
    return 5;
    let x = 10;  // This will never execute
    return x;
}

// ✅ Correct: restructured logic
fn example() -> i32 {
    let x = 10;
    if x > 5 {
        return 5;
    }
    return x;
}
```

## Variable and Mutability Issues

### Assignment to Immutable Variable
**Problem:** `error: cannot assign to immutable variable 'x'`

**Solution:** Declare variables as mutable when you need to modify them:
```aero
// ❌ Error: trying to modify immutable variable
let x = 5;
x = 10;  // Not allowed

// ✅ Correct: mutable variable
let mut x = 5;
x = 10;  // Allowed
```

### Variable Not Found
**Problem:** `error: cannot find value 'variable_name' in this scope`

**Solution:** Ensure variables are declared before use and are in scope:
```aero
// ❌ Error: variable not declared
fn main() {
    println!("{}", undefined_var);
}

// ✅ Correct: variable declared
fn main() {
    let defined_var = 42;
    println!("{}", defined_var);
}

// ❌ Error: variable out of scope
fn main() {
    {
        let inner_var = 42;
    }
    println!("{}", inner_var);  // inner_var not accessible here
}

// ✅ Correct: variable in scope
fn main() {
    let outer_var = 42;
    {
        println!("{}", outer_var);  // outer_var accessible here
    }
}
```

## I/O and Formatting Issues

### Format String Argument Mismatch
**Problem:** `error: format string expects 2 arguments, found 1`

**Solution:** Match the number of format placeholders with arguments:
```aero
// ❌ Error: mismatched arguments
println!("Values: {} and {}", 42);        // 2 placeholders, 1 argument
println!("Value: {}", 42, 24);            // 1 placeholder, 2 arguments

// ✅ Correct: matched arguments
println!("Values: {} and {}", 42, 24);    // 2 placeholders, 2 arguments
println!("Value: {}", 42);                // 1 placeholder, 1 argument
```

### Invalid Format String
**Problem:** `error: invalid format string`

**Solution:** Ensure format strings have properly matched braces:
```aero
// ❌ Error: unmatched braces
println!("Value: {", 42);     // Missing closing brace
println!("Value: }", 42);     // Missing opening brace

// ✅ Correct: matched braces
println!("Value: {}", 42);    // Properly matched braces
```

## Type System Issues

### Type Mismatch in Assignment
**Problem:** `error: expected 'i32', found '&str'`

**Solution:** Ensure assigned values match the expected type:
```aero
// ❌ Error: type mismatch
let x: i32 = "hello";  // String assigned to integer

// ✅ Correct: matching types
let x: i32 = 42;       // Integer assigned to integer
let y: &str = "hello"; // String assigned to string
```

### Incompatible Types in Operations
**Problem:** `error: cannot add 'i32' and '&str'`

**Solution:** Ensure operands are compatible or convert them:
```aero
// ❌ Error: incompatible types
let result = 5 + "hello";

// ✅ Correct: compatible types
let result = 5 + 3;           // Both integers
let result = "hello" + " world"; // Both strings (if supported)
```

### Comparison Type Mismatch
**Problem:** `error: cannot compare 'i32' and '&str'`

**Solution:** Only compare values of the same type:
```aero
// ❌ Error: comparing different types
if 42 == "42" {
    println!("This won't work");
}

// ✅ Correct: comparing same types
if 42 == 42 {
    println!("This works");
}
```

## Compilation Issues

### Syntax Errors
**Problem:** Various parsing errors

**Common Solutions:**
1. **Missing semicolons:** Add `;` after statements
2. **Unmatched braces:** Ensure `{` and `}` are balanced
3. **Unmatched parentheses:** Ensure `(` and `)` are balanced
4. **Invalid keywords:** Check spelling of keywords like `fn`, `let`, `mut`

### Semantic Analysis Errors
**Problem:** Code parses but fails semantic analysis

**Solutions:**
1. Check variable declarations and scoping
2. Verify function signatures match calls
3. Ensure type consistency throughout expressions
4. Validate control flow structure

## Performance Issues

### Slow Compilation
**Problem:** Compiler takes too long

**Solutions:**
1. Break large files into smaller modules
2. Reduce deeply nested expressions
3. Simplify complex function signatures
4. Check for infinite recursion in type definitions

### Large Executable Size
**Problem:** Generated executable is too large

**Solutions:**
1. Use release mode compilation (when available)
2. Remove unused functions and variables
3. Optimize recursive algorithms
4. Consider iterative alternatives to recursion

## Debugging Tips

### Enable Verbose Output
Use compiler flags for more detailed error information:
```bash
aero build --verbose program.aero
```

### Check Intermediate Representations
Examine generated IR for debugging:
```bash
aero build --emit-ir program.aero
```

### Use Simple Test Cases
Create minimal examples to isolate issues:
```aero
// Minimal function test
fn test() -> i32 {
    return 42;
}

fn main() {
    let result = test();
    println!("{}", result);
}
```

### Incremental Development
Build features step by step:
1. Start with basic variable declarations
2. Add simple functions
3. Introduce control flow
4. Add I/O operations
5. Combine features gradually

## Getting Help

### Community Resources
- Check the [GitHub Issues](https://github.com/RobVanProd/Aero/issues) for known problems
- Review example programs in the `examples/` directory
- Consult the language specification documents

### Reporting Bugs
When reporting issues, include:
1. Aero compiler version (`aero --version`)
2. Operating system and version
3. Complete error message
4. Minimal code example that reproduces the issue
5. Expected vs. actual behavior

### Best Practices for Avoiding Issues
1. **Start Simple:** Begin with basic programs and add complexity gradually
2. **Test Frequently:** Compile and test after each small change
3. **Read Error Messages:** Aero provides detailed error information
4. **Use Examples:** Reference working examples for syntax patterns
5. **Follow Conventions:** Use consistent naming and formatting
6. **Understand Scoping:** Be aware of variable and function scope rules
7. **Type Awareness:** Pay attention to type requirements and conversions

This troubleshooting guide covers the most common issues encountered when working with Aero's Phase 3 features. As the language evolves, this guide will be updated with new solutions and best practices.