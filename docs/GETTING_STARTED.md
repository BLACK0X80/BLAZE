# Getting Started with BLAZE

Welcome to BLAZE! This guide will help you get up and running with the BLAZE compiler.

## Table of Contents

1. [Installation](#installation)
2. [Your First Program](#your-first-program)
3. [Basic Syntax](#basic-syntax)
4. [Compiling and Running](#compiling-and-running)
5. [Common Errors](#common-errors)
6. [Next Steps](#next-steps)

## Installation

### Prerequisites

Before installing BLAZE, ensure you have:

- **Rust**: Version 1.70.0 or later
  ```bash
  # Check your Rust version
  rustc --version
  
  # Install or update Rust
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **LLVM**: Version 15.0 (required for code generation)
  - See [LLVM_SETUP.md](../LLVM_SETUP.md) for detailed installation instructions

- **C Compiler**: 
  - Windows: MSVC (Visual Studio Build Tools)
  - Linux: GCC or Clang
  - macOS: Clang (comes with Xcode Command Line Tools)

### Building BLAZE

```bash
# Clone the repository
git clone https://github.com/yourusername/blaze.git
cd blaze

# Build the compiler
cargo build --release

# The compiler binary will be at target/release/blaze
# Optionally, install it to your PATH
cargo install --path .
```

### Verify Installation

```bash
# Check that BLAZE is installed
blaze --version

# Run the test suite
cargo test
```

## Your First Program

Let's write a simple "Hello, World!" program in BLAZE.

### Create a File

Create a new file called `hello.blz`:

```rust
fn main() {
    println("Hello, BLAZE!");
}
```

### Compile and Run

```bash
# Compile the program
blaze build hello.blz -o hello

# Run the executable
./hello  # Linux/macOS
hello.exe  # Windows
```

You should see:
```
Hello, BLAZE!
```

Congratulations! You've written and run your first BLAZE program.

## Basic Syntax

### Variables

BLAZE variables are immutable by default:

```rust
// Immutable variable
let x: i32 = 42;

// Mutable variable
let mut counter: i32 = 0;
counter = counter + 1;

// Type inference
let y = 3.14;  // Inferred as f64
let name = "Alice";  // Inferred as String
```

### Functions

Functions are declared with the `fn` keyword:

```rust
// Function with parameters and return type
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

// Expression-based return (no semicolon)
fn multiply(x: i32, y: i32) -> i32 {
    x * y
}

// Function with no return value
fn greet(name: String) {
    println("Hello, ");
    println(name);
}
```

### Control Flow

#### If Expressions

```rust
let x = 10;

if x > 0 {
    println("positive");
} else if x < 0 {
    println("negative");
} else {
    println("zero");
}
```

#### While Loops

```rust
let mut counter = 0;

while counter < 10 {
    println(counter);
    counter = counter + 1;
}
```

#### For Loops

```rust
// Range-based for loop
for i in 0..10 {
    println(i);
}
```

### Data Types

BLAZE supports several built-in types:

```rust
// Integers
let a: i8 = 127;
let b: i16 = 32767;
let c: i32 = 2147483647;
let d: i64 = 9223372036854775807;

// Floating point
let e: f32 = 3.14;
let f: f64 = 2.718281828;

// Boolean
let g: bool = true;
let h: bool = false;

// Strings
let i: String = "Hello, BLAZE!";
```

### Structs

Define custom data types with structs:

```rust
struct Point {
    x: f64,
    y: f64,
}

fn main() {
    let p = Point { x: 3.0, y: 4.0 };
    let distance = sqrt(p.x * p.x + p.y * p.y);
    println(distance);
}
```

## Compiling and Running

### Compiler Commands

BLAZE provides several commands:

#### Check Syntax

Validate your code without generating an executable:

```bash
blaze check myprogram.blz
```

This is fast and useful during development.

#### Build Executable

Compile your program to an executable:

```bash
# Basic compilation
blaze build myprogram.blz -o myprogram

# With optimization
blaze build myprogram.blz -o myprogram -O2
```

Optimization levels:
- `-O0`: No optimization (default, fastest compilation)
- `-O1`: Basic optimization
- `-O2`: Moderate optimization (recommended for release)
- `-O3`: Aggressive optimization (best performance)

#### Compile and Run

Compile and immediately run your program:

```bash
blaze run myprogram.blz
```

### Additional Options

```bash
# Emit intermediate representation
blaze build myprogram.blz --emit-ir

# Emit assembly code
blaze build myprogram.blz --emit-asm

# Disable colored output
blaze build myprogram.blz --no-color
```

## Common Errors

### Type Mismatch

```rust
let x: i32 = "hello";  // Error: expected i32, found String
```

**Fix**: Ensure the value matches the declared type:
```rust
let x: i32 = 42;
let y: String = "hello";
```

### Undefined Variable

```rust
fn main() {
    println(x);  // Error: cannot find value `x` in this scope
}
```

**Fix**: Declare the variable before using it:
```rust
fn main() {
    let x = 42;
    println(x);
}
```

### Borrow Checker Error

```rust
fn main() {
    let x = 42;
    let r1 = &x;
    let r2 = &mut x;  // Error: cannot borrow as mutable
}
```

**Fix**: Don't mix mutable and immutable borrows:
```rust
fn main() {
    let mut x = 42;
    let r1 = &x;
    // Use r1 before creating mutable borrow
    let r2 = &mut x;
}
```

### Missing Semicolon

```rust
fn main() {
    let x = 42
    println(x);  // Error: expected `;`
}
```

**Fix**: Add semicolons after statements:
```rust
fn main() {
    let x = 42;
    println(x);
}
```

## Next Steps

Now that you've learned the basics, here are some next steps:

1. **Explore Examples**: Check out the `examples/` directory for more complex programs
   - `fibonacci.blz` - Recursive functions
   - `sorting_algorithms.blz` - Algorithms and loops
   - `data_structures.blz` - Structs and arrays

2. **Read the Language Reference**: Learn about advanced features
   - See [LANGUAGE_REFERENCE.md](LANGUAGE_REFERENCE.md)

3. **Understand Memory Safety**: Learn about ownership and borrowing
   - See the ownership section in the language reference

4. **Learn the Compiler Architecture**: Understand how BLAZE works internally
   - See [COMPILER_ARCHITECTURE.md](COMPILER_ARCHITECTURE.md)

5. **Troubleshooting**: If you encounter issues
   - See [TROUBLESHOOTING.md](TROUBLESHOOTING.md)

## Getting Help

- **Documentation**: Check the `docs/` directory
- **Examples**: Look at the `examples/` directory
- **Issues**: Report bugs on GitHub Issues
- **Discussions**: Ask questions on GitHub Discussions

## Quick Reference

### Common Commands

```bash
# Check syntax
blaze check file.blz

# Compile
blaze build file.blz -o output

# Compile with optimization
blaze build file.blz -o output -O2

# Compile and run
blaze run file.blz

# Run tests
cargo test
```

### Basic Program Structure

```rust
// Import functions (if needed)

// Define structs
struct MyStruct {
    field1: i32,
    field2: String,
}

// Define functions
fn helper_function(x: i32) -> i32 {
    x * 2
}

// Main entry point
fn main() {
    let x = 42;
    let result = helper_function(x);
    println(result);
}
```

Happy coding with BLAZE! 🔥
