# BLAZE Language Reference

This document provides a comprehensive reference for the BLAZE programming language syntax and semantics.

## Table of Contents

1. [Lexical Structure](#lexical-structure)
2. [Types](#types)
3. [Variables](#variables)
4. [Functions](#functions)
5. [Expressions](#expressions)
6. [Statements](#statements)
7. [Control Flow](#control-flow)
8. [Structs](#structs)
9. [Ownership and Borrowing](#ownership-and-borrowing)
10. [Operators](#operators)

## Lexical Structure

### Keywords

BLAZE reserves the following keywords:

```
fn      let     mut     if      else    while   for     in
return  struct  enum    impl    trait   pub     use     mod
true    false   i8      i16     i32     i64     i128    u8
u16     u32     u64     u128    f32     f64     bool    String
```

### Identifiers

Identifiers must start with a letter or underscore, followed by any number of letters, digits, or underscores:

```rust
valid_identifier
_private
counter123
MyStruct
```

### Comments

```rust
// Single-line comment

/*
   Multi-line comment
   can span multiple lines
*/
```

### Literals

#### Integer Literals

```rust
42          // Decimal
0x2A        // Hexadecimal
0o52        // Octal
0b101010    // Binary
```

#### Floating-Point Literals

```rust
3.14
2.718
1.0e10
```

#### String Literals

```rust
"Hello, World!"
"Escape sequences: \n \t \\ \""
```

#### Boolean Literals

```rust
true
false
```

## Types

### Primitive Types

#### Integer Types

| Type | Size | Range |
|------|------|-------|
| `i8` | 8-bit | -128 to 127 |
| `i16` | 16-bit | -32,768 to 32,767 |
| `i32` | 32-bit | -2,147,483,648 to 2,147,483,647 |
| `i64` | 64-bit | -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807 |
| `i128` | 128-bit | Very large range |

#### Floating-Point Types

| Type | Size | Precision |
|------|------|-----------|
| `f32` | 32-bit | ~7 decimal digits |
| `f64` | 64-bit | ~15 decimal digits |

#### Boolean Type

```rust
let flag: bool = true;
let is_valid: bool = false;
```

#### String Type

```rust
let message: String = "Hello, BLAZE!";
```

### Compound Types

#### Arrays

Fixed-size collections of elements of the same type:

```rust
let numbers: [i32; 5] = [1, 2, 3, 4, 5];
let first = numbers[0];
```

#### Structs

Custom data types with named fields:

```rust
struct Point {
    x: f64,
    y: f64,
}

let p = Point { x: 3.0, y: 4.0 };
```

### Reference Types

#### Immutable References

```rust
let x = 42;
let r = &x;  // Immutable reference to x
```

#### Mutable References

```rust
let mut x = 42;
let r = &mut x;  // Mutable reference to x
*r = 43;  // Dereference and modify
```

## Variables

### Immutable Variables

Variables are immutable by default:

```rust
let x = 42;
// x = 43;  // Error: cannot assign twice to immutable variable
```

### Mutable Variables

Use `mut` to make a variable mutable:

```rust
let mut counter = 0;
counter = counter + 1;  // OK
```

### Type Annotations

Explicit type annotations:

```rust
let x: i32 = 42;
let y: f64 = 3.14;
let name: String = "Alice";
```

### Type Inference

The compiler can infer types:

```rust
let x = 42;        // Inferred as i32
let y = 3.14;      // Inferred as f64
let flag = true;   // Inferred as bool
```

### Shadowing

Variables can be shadowed in the same scope:

```rust
let x = 5;
let x = x + 1;  // Shadows previous x
let x = x * 2;  // Shadows again
```

## Functions

### Function Declaration

```rust
fn function_name(param1: Type1, param2: Type2) -> ReturnType {
    // Function body
    return value;
}
```

### Parameters

```rust
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}
```

### Return Values

#### Explicit Return

```rust
fn square(x: i32) -> i32 {
    return x * x;
}
```

#### Expression Return

The last expression is returned automatically (no semicolon):

```rust
fn square(x: i32) -> i32 {
    x * x  // No semicolon
}
```

### No Return Value

Functions without a return type return nothing:

```rust
fn print_message(msg: String) {
    println(msg);
}
```

### Main Function

Every program must have a `main` function:

```rust
fn main() {
    // Program entry point
}
```

## Expressions

### Arithmetic Expressions

```rust
let sum = a + b;
let difference = a - b;
let product = a * b;
let quotient = a / b;
let remainder = a % b;
```

### Comparison Expressions

```rust
let equal = a == b;
let not_equal = a != b;
let less = a < b;
let less_equal = a <= b;
let greater = a > b;
let greater_equal = a >= b;
```

### Logical Expressions

```rust
let and = a && b;
let or = a || b;
let not = !a;
```

### Function Calls

```rust
let result = function_name(arg1, arg2);
```

### Field Access

```rust
let x = point.x;
let y = point.y;
```

### Array Indexing

```rust
let first = array[0];
let second = array[1];
```

### Parenthesized Expressions

```rust
let result = (a + b) * c;
```

## Statements

### Let Statements

```rust
let x = 42;
let mut y = 10;
let z: i32 = 5;
```

### Expression Statements

```rust
function_call();
x + y;
```

### Assignment Statements

```rust
x = 42;
array[0] = 10;
point.x = 3.0;
```

### Return Statements

```rust
return value;
return;  // Return from void function
```

## Control Flow

### If Expressions

```rust
if condition {
    // Then block
} else if other_condition {
    // Else-if block
} else {
    // Else block
}
```

If expressions can return values:

```rust
let x = if condition { 10 } else { 20 };
```

### While Loops

```rust
while condition {
    // Loop body
}
```

Example:

```rust
let mut counter = 0;
while counter < 10 {
    println(counter);
    counter = counter + 1;
}
```

### For Loops

Range-based iteration:

```rust
for i in 0..10 {
    println(i);
}
```

### Break and Continue

```rust
while true {
    if condition {
        break;  // Exit loop
    }
    if other_condition {
        continue;  // Skip to next iteration
    }
}
```

## Structs

### Struct Definition

```rust
struct StructName {
    field1: Type1,
    field2: Type2,
}
```

### Struct Instantiation

```rust
let instance = StructName {
    field1: value1,
    field2: value2,
};
```

### Field Access

```rust
let value = instance.field1;
instance.field2 = new_value;
```

### Example

```rust
struct Rectangle {
    width: f64,
    height: f64,
}

fn area(rect: Rectangle) -> f64 {
    rect.width * rect.height
}

fn main() {
    let rect = Rectangle {
        width: 10.0,
        height: 20.0,
    };
    let a = area(rect);
    println(a);
}
```

## Ownership and Borrowing

BLAZE implements Rust-like ownership rules for memory safety.

### Ownership Rules

1. Each value has a single owner
2. When the owner goes out of scope, the value is dropped
3. Values can be moved or borrowed

### Move Semantics

```rust
let s1 = "hello";
let s2 = s1;  // s1 is moved to s2
// println(s1);  // Error: s1 was moved
```

### Borrowing

#### Immutable Borrows

Multiple immutable borrows are allowed:

```rust
let x = 42;
let r1 = &x;
let r2 = &x;  // OK: multiple immutable borrows
println(r1);
println(r2);
```

#### Mutable Borrows

Only one mutable borrow is allowed at a time:

```rust
let mut x = 42;
let r = &mut x;
*r = 43;
// let r2 = &mut x;  // Error: cannot borrow as mutable more than once
```

#### Borrow Rules

1. You can have either:
   - One mutable borrow, OR
   - Any number of immutable borrows
2. Borrows must not outlive the owner
3. Cannot have mutable and immutable borrows simultaneously

### Lifetimes

Lifetimes ensure references are valid:

```rust
fn longest(s1: &String, s2: &String) -> &String {
    if s1.len() > s2.len() {
        s1
    } else {
        s2
    }
}
```

The compiler infers lifetimes automatically in most cases.

## Operators

### Arithmetic Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `+` | Addition | `a + b` |
| `-` | Subtraction | `a - b` |
| `*` | Multiplication | `a * b` |
| `/` | Division | `a / b` |
| `%` | Remainder | `a % b` |

### Comparison Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `==` | Equal | `a == b` |
| `!=` | Not equal | `a != b` |
| `<` | Less than | `a < b` |
| `<=` | Less than or equal | `a <= b` |
| `>` | Greater than | `a > b` |
| `>=` | Greater than or equal | `a >= b` |

### Logical Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `&&` | Logical AND | `a && b` |
| `||` | Logical OR | `a || b` |
| `!` | Logical NOT | `!a` |

### Assignment Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `=` | Assignment | `x = 42` |

### Reference Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `&` | Immutable borrow | `&x` |
| `&mut` | Mutable borrow | `&mut x` |
| `*` | Dereference | `*r` |

### Operator Precedence

From highest to lowest:

1. Field access (`.`), array indexing (`[]`), function calls (`()`)
2. Unary operators (`!`, `-`, `*`, `&`)
3. Multiplication, division, remainder (`*`, `/`, `%`)
4. Addition, subtraction (`+`, `-`)
5. Comparison (`<`, `<=`, `>`, `>=`)
6. Equality (`==`, `!=`)
7. Logical AND (`&&`)
8. Logical OR (`||`)
9. Assignment (`=`)

Use parentheses to override precedence:

```rust
let result = (a + b) * c;  // Add first, then multiply
```

## Standard Library

### Built-in Functions

#### println

Print a value to standard output:

```rust
println("Hello, World!");
println(42);
println(3.14);
```

#### Mathematical Functions

```rust
let root = sqrt(16.0);      // Square root
let power = pow(2.0, 3.0);  // Power
let sine = sin(1.57);       // Sine
let cosine = cos(0.0);      // Cosine
```

### Collections

#### Vec

Dynamic array:

```rust
let mut v = Vec::new();
v.push(1);
v.push(2);
v.push(3);
let first = v.get(0);
let len = v.len();
```

#### HashMap

Key-value map:

```rust
let mut map = HashMap::new();
map.insert("key", 42);
let value = map.get("key");
```

## Best Practices

### Naming Conventions

- **Variables and functions**: `snake_case`
- **Types and structs**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE`

### Code Style

```rust
// Good: Clear and readable
fn calculate_area(width: f64, height: f64) -> f64 {
    width * height
}

// Good: Descriptive names
let user_count = 42;
let is_valid = true;

// Avoid: Single-letter names (except in loops)
let x = 42;  // What does x represent?
```

### Error Handling

Always handle potential errors:

```rust
// Check bounds before indexing
if index < array.len() {
    let value = array[index];
}

// Validate inputs
fn divide(a: i32, b: i32) -> i32 {
    if b == 0 {
        return 0;  // Handle division by zero
    }
    a / b
}
```

### Memory Safety

Follow ownership rules:

```rust
// Good: Clear ownership
fn process_data(data: String) {
    // Function owns data
}

// Good: Borrow when you don't need ownership
fn read_data(data: &String) {
    // Function borrows data
}
```

## Examples

### Fibonacci Sequence

```rust
fn fibonacci(n: i32) -> i32 {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fn main() {
    let result = fibonacci(10);
    println(result);
}
```

### Bubble Sort

```rust
fn bubble_sort(arr: &mut [i32]) {
    let n = arr.len();
    for i in 0..n {
        for j in 0..(n - i - 1) {
            if arr[j] > arr[j + 1] {
                let temp = arr[j];
                arr[j] = arr[j + 1];
                arr[j + 1] = temp;
            }
        }
    }
}
```

### Point Distance

```rust
struct Point {
    x: f64,
    y: f64,
}

fn distance(p1: &Point, p2: &Point) -> f64 {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    sqrt(dx * dx + dy * dy)
}

fn main() {
    let p1 = Point { x: 0.0, y: 0.0 };
    let p2 = Point { x: 3.0, y: 4.0 };
    let dist = distance(&p1, &p2);
    println(dist);  // 5.0
}
```

## Further Reading

- [Getting Started Guide](GETTING_STARTED.md) - Learn the basics
- [Compiler Architecture](COMPILER_ARCHITECTURE.md) - Understand how BLAZE works
- [Troubleshooting Guide](TROUBLESHOOTING.md) - Fix common issues
- [Examples Directory](../examples/) - More code examples
