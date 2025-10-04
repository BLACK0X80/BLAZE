            let s2 = s1;
            println("{}", s1);
        }
    "#;
    let tokens = lex(source).unwrap();
    let ast = parse(tokens).unwrap();
    let result = check(&ast);
    
    assert!(result.is_err());
}

#[test]
fn test_immutable_borrow() {
    let source = r#"
        fn main() {
            let s = String::from("hello");
            let r1 = &s;
            let r2 = &s;
            println("{} {}", r1, r2);
        }
    "#;
    let tokens = lex(source).unwrap();
    let ast = parse(tokens).unwrap();
    let result = check(&ast);
    
    assert!(result.is_ok());
}

#[test]
fn test_mutable_borrow_conflict() {
    let source = r#"
        fn main() {
            let mut s = String::from("hello");
            let r1 = &s;
            let r2 = &mut s;
            println("{}", r1);
        }
    "#;
    let tokens = lex(source).unwrap();
    let ast = parse(tokens).unwrap();
    let result = check(&ast);
    
    assert!(result.is_err());
}

#[test]
fn test_lifetime_inference() {
    let source = r#"
        fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
            if x.len() > y.len() {
                x
            } else {
                y
            }
        }
    "#;
    let tokens = lex(source).unwrap();
    let ast = parse(tokens).unwrap();
    let result = check(&ast);
    
    assert!(result.is_ok());
}
```

### tests/integration_tests.rs
```rust
use blaze_compiler::compile;
use std::path::PathBuf;
use std::fs;

#[test]
fn test_hello_world() {
    let source = r#"
        fn main() {
            println("Hello, World!");
        }
    "#;
    
    let temp_file = PathBuf::from("test_hello.blz");
    fs::write(&temp_file, source).unwrap();
    
    let result = compile(&temp_file, false, 0, None, None, false);
    
    fs::remove_file(&temp_file).ok();
    
    assert!(result.is_ok());
}

#[test]
fn test_fibonacci() {
    let source = r#"
        fn fib(n: i32) -> i32 {
            if n <= 1 {
                n
            } else {
                fib(n - 1) + fib(n - 2)
            }
        }
        
        fn main() {
            let result = fib(10);
            println("{}", result);
        }
    "#;
    
    let temp_file = PathBuf::from("test_fib.blz");
    fs::write(&temp_file, source).unwrap();
    
    let result = compile(&temp_file, false, 0, None, None, false);
    
    fs::remove_file(&temp_file).ok();
    
    assert!(result.is_ok());
}

#[test]
fn test_ownership_demo() {
    let source = r#"
        fn take_ownership(s: String) {
            println("{}", s);
        }
        
        fn borrow(s: &String) {
            println("{}", s);
        }
        
        fn main() {
            let s1 = String::from("hello");
            borrow(&s1);
            println("{}", s1);
            
            let s2 = String::from("world");
            take_ownership(s2);
        }
    "#;
    
    let temp_file = PathBuf::from("test_ownership.blz");
    fs::write(&temp_file, source).unwrap();
    
    let result = compile(&temp_file, false, 0, None, None, false);
    
    fs::remove_file(&temp_file).ok();
    
    assert!(result.is_ok());
}
```

## BENCHMARKS

### benches/lexer_bench.rs
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use blaze_compiler::lexer::lex;

fn lexer_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer");
    
    let small_source = "let x = 5;";
    let medium_source = r#"
        fn factorial(n: i32) -> i32 {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
    "#;
    let large_source = include_str!("../examples/large_program.blz");
    
    group.bench_with_input(
        BenchmarkId::new("small", small_source.len()),
        small_source,
        |b, s| b.iter(|| lex(black_box(s))),
    );
    
    group.bench_with_input(
        BenchmarkId::new("medium", medium_source.len()),
        medium_source,
        |b, s| b.iter(|| lex(black_box(s))),
    );
    
    group.bench_with_input(
        BenchmarkId::new("large", large_source.len()),
        large_source,
        |b, s| b.iter(|| lex(black_box(s))),
    );
    
    group.finish();
}

criterion_group!(benches, lexer_benchmark);
criterion_main!(benches);
```

### benches/parser_bench.rs
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use blaze_compiler::{lexer::lex, parser::parse};

fn parser_benchmark(c: &mut Criterion) {
    let source = r#"
        fn factorial(n: i32) -> i32 {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
        
        fn main() {
            let result = factorial(10);
            println("{}", result);
        }
    "#;
    
    let tokens = lex(source).unwrap();
    
    c.bench_function("parse_factorial", |b| {
        b.iter(|| parse(black_box(tokens.clone())))
    });
}

criterion_group!(benches, parser_benchmark);
criterion_main!(benches);
```

## DOCUMENTATION TEMPLATES

### README.md
```markdown
# Blaze Programming Language 🔥

A modern systems programming language with Rust-like safety guarantees and blazing fast performance.

## Features

- **Memory Safety**: Ownership and borrowing system prevents data races and memory leaks at compile time
- **Zero-Cost Abstractions**: High-level features with no runtime overhead
- **Strong Type System**: Advanced type inference and checking
- **Pattern Matching**: Exhaustive pattern matching for safer code
- **Generics**: Powerful generic programming with trait bounds
- **No Garbage Collector**: Predictable performance without GC pauses
- **LLVM Backend**: State-of-the-art optimizations and multi-platform support

## Installation

```bash
cargo install blaze
```

## Quick Start

Create a new file `hello.blz`:

```blaze
fn main() {
    println("Hello, Blaze!");
}
```

Compile and run:

```bash
blaze run hello.blz
```

## Language Syntax

### Variables
```blaze
let x: i32 = 5;
let mut y: i32 = 10;
y = 15;
```

### Functions
```blaze
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Ownership
```blaze
let s1 = String::from("hello");
let s2 = s1;
```

### Borrowing
```blaze
fn length(s: &String) -> usize {
    s.len()
}

let s = String::from("hello");
let len = length(&s);
```

### Structs
```blaze
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}
```

### Enums and Pattern Matching
```blaze
enum# BLAZE PROGRAMMING LANGUAGE - COMPLETE IMPLEMENTATION PROMPT

You are an expert systems programmer tasked with building a complete, production-ready programming language called **Blaze** (.blz file extension). This language must be heavily inspired by Rust with ownership, borrowing, and memory safety guarantees. This is a comprehensive, no-compromise implementation.

## CRITICAL REQUIREMENTS - READ CAREFULLY

1. ABSOLUTELY NO COMMENTS in any code files. Code must be self-documenting through clear naming.
2. Every feature must be fully functional, not placeholder or stub implementations.
3. Comprehensive test coverage for every component with edge cases.
4. Production-quality error messages with colors, line numbers, and helpful suggestions.
5. Complete standard library with common data structures and utilities.
6. Full optimization passes in the compiler backend.
7. Documentation in separate markdown files, never in code.
8. Performance benchmarks and profiling tools.

## PROJECT ARCHITECTURE

### Directory Structure
```
blaze/
├── Cargo.toml
├── README.md
├── LANGUAGE_SPEC.md
├── EXAMPLES.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── cli.rs
│   ├── lexer/
│   │   ├── mod.rs
│   │   ├── token.rs
│   │   └── scanner.rs
│   ├── parser/
│   │   ├── mod.rs
│   │   ├── ast.rs
│   │   ├── expr.rs
│   │   ├── stmt.rs
│   │   └── types.rs
│   ├── semantic/
│   │   ├── mod.rs
│   │   ├── symbol_table.rs
│   │   ├── type_checker.rs
│   │   ├── borrow_checker.rs
│   │   ├── lifetime_analyzer.rs
│   │   └── scope_resolver.rs
│   ├── ir/
│   │   ├── mod.rs
│   │   ├── builder.rs
│   │   ├── instruction.rs
│   │   ├── optimization/
│   │   │   ├── mod.rs
│   │   │   ├── constant_folding.rs
│   │   │   ├── dead_code_elimination.rs
│   │   │   ├── inlining.rs
│   │   │   └── peephole.rs
│   │   └── validation.rs
│   ├── codegen/
│   │   ├── mod.rs
│   │   ├── llvm_backend.rs
│   │   ├── register_allocator.rs
│   │   └── linker.rs
│   ├── runtime/
│   │   ├── mod.rs
│   │   ├── allocator.rs
│   │   ├── panic.rs
│   │   └── intrinsics.rs
│   ├── stdlib/
│   │   ├── mod.rs
│   │   ├── prelude.rs
│   │   ├── collections/
│   │   │   ├── mod.rs
│   │   │   ├── vec.rs
│   │   │   ├── hashmap.rs
│   │   │   ├── hashset.rs
│   │   │   └── linked_list.rs
│   │   ├── io/
│   │   │   ├── mod.rs
│   │   │   ├── stdio.rs
│   │   │   ├── file.rs
│   │   │   └── buffer.rs
│   │   ├── string/
│   │   │   ├── mod.rs
│   │   │   └── operations.rs
│   │   ├── math/
│   │   │   ├── mod.rs
│   │   │   └── ops.rs
│   │   └── sync/
│   │       ├── mod.rs
│   │       ├── mutex.rs
│   │       └── channel.rs
│   ├── error/
│   │   ├── mod.rs
│   │   ├── diagnostic.rs
│   │   ├── reporter.rs
│   │   └── recovery.rs
│   └── utils/
│       ├── mod.rs
│       ├── source_map.rs
│       └── span.rs
├── tests/
│   ├── lexer_tests.rs
│   ├── parser_tests.rs
│   ├── type_checker_tests.rs
│   ├── borrow_checker_tests.rs
│   ├── codegen_tests.rs
│   ├── integration_tests.rs
│   └── stdlib_tests.rs
├── benches/
│   ├── lexer_bench.rs
│   ├── parser_bench.rs
│   └── codegen_bench.rs
├── examples/
│   ├── hello_world.blz
│   ├── ownership_demo.blz
│   ├── borrowing_demo.blz
│   ├── structs_and_enums.blz
│   ├── pattern_matching.blz
│   ├── generics.blz
│   ├── traits.blz
│   ├── async_example.blz
│   ├── web_server.blz
│   └── game_engine.blz
└── docs/
    ├── getting_started.md
    ├── syntax_reference.md
    ├── ownership_guide.md
    ├── standard_library.md
    ├── compiler_internals.md
    └── contributing.md
```

## LANGUAGE SPECIFICATION - BLAZE SYNTAX

### Core Syntax Rules

#### Variables and Mutability
```
let x: i32 = 5;
let mut y: i32 = 10;
y = 15;

const MAX_SIZE: usize = 1000;
static GLOBAL_COUNT: i32 = 0;
```

#### Functions
```
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn no_return(x: i32) {
    println("Value: {}", x);
}

fn generic_function<T>(value: T) -> T {
    value
}

fn with_lifetime<'a>(s: &'a str) -> &'a str {
    s
}
```

#### Ownership and Borrowing
```
fn take_ownership(s: String) {
    println("{}", s);
}

fn borrow_immutable(s: &String) {
    println("{}", s);
}

fn borrow_mutable(s: &mut String) {
    s.push_str(" world");
}

fn main() {
    let s1 = String::from("hello");
    let s2 = s1;
    
    let s3 = String::from("hello");
    borrow_immutable(&s3);
    println("{}", s3);
    
    let mut s4 = String::from("hello");
    borrow_mutable(&mut s4);
    println("{}", s4);
}
```

#### Structs
```
struct Point {
    x: f64,
    y: f64,
}

struct GenericPoint<T> {
    x: T,
    y: T,
}

impl Point {
    fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }
    
    fn distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    
    fn translate(&mut self, dx: f64, dy: f64) {
        self.x += dx;
        self.y += dy;
    }
}

impl<T> GenericPoint<T> {
    fn new(x: T, y: T) -> GenericPoint<T> {
        GenericPoint { x, y }
    }
}
```

#### Enums and Pattern Matching
```
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}

enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}

fn process_message(msg: Message) {
    match msg {
        Message::Quit => println("Quit"),
        Message::Move { x, y } => println("Move to {}, {}", x, y),
        Message::Write(s) => println("Write: {}", s),
        Message::ChangeColor(r, g, b) => println("Color: {}, {}, {}", r, g, b),
    }
}

fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Result::Err(String::from("Division by zero"))
    } else {
        Result::Ok(a / b)
    }
}
```

#### Traits
```
trait Drawable {
    fn draw(&self);
}

trait Comparable {
    fn compare(&self, other: &Self) -> i32;
}

struct Circle {
    radius: f64,
}

impl Drawable for Circle {
    fn draw(&self) {
        println("Drawing circle with radius {}", self.radius);
    }
}

impl Comparable for Circle {
    fn compare(&self, other: &Circle) -> i32 {
        if self.radius < other.radius {
            -1
        } else if self.radius > other.radius {
            1
        } else {
            0
        }
    }
}

fn draw_shape<T: Drawable>(shape: &T) {
    shape.draw();
}
```

#### Control Flow
```
if x > 0 {
    println("Positive");
} else if x < 0 {
    println("Negative");
} else {
    println("Zero");
}

let result = if condition { value1 } else { value2 };

while i < 10 {
    i += 1;
}

for i in 0..10 {
    println("{}", i);
}

for item in collection {
    println("{}", item);
}

loop {
    if condition {
        break;
    }
}
```

#### Closures
```
let add = |a: i32, b: i32| -> i32 { a + b };
let result = add(5, 3);

let mut counter = 0;
let mut increment = || {
    counter += 1;
};

let numbers = vec![1, 2, 3, 4, 5];
let doubled: Vec<i32> = numbers.iter().map(|x| x * 2).collect();
let filtered: Vec<i32> = numbers.iter().filter(|x| *x % 2 == 0).collect();
```

#### Modules and Imports
```
mod geometry {
    pub struct Point {
        pub x: f64,
        pub y: f64,
    }
    
    pub fn distance(p1: &Point, p2: &Point) -> f64 {
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        (dx * dx + dy * dy).sqrt()
    }
}

use geometry::Point;
use geometry::distance;
```

#### Error Handling
```
fn read_file(path: &str) -> Result<String, String> {
    if path.is_empty() {
        return Result::Err(String::from("Empty path"));
    }
    Result::Ok(String::from("File content"))
}

fn main() {
    match read_file("test.txt") {
        Result::Ok(content) => println("{}", content),
        Result::Err(e) => println("Error: {}", e),
    }
    
    let content = read_file("test.txt")?;
}
```

#### Generics with Constraints
```
fn largest<T: Comparable>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item.compare(largest) > 0 {
            largest = item;
        }
    }
    largest
}
```

#### Lifetimes
```
fn longest<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.len() > s2.len() {
        s1
    } else {
        s2
    }
}

struct Reference<'a> {
    value: &'a str,
}

impl<'a> Reference<'a> {
    fn new(value: &'a str) -> Reference<'a> {
        Reference { value }
    }
}
```

#### Async/Await
```
async fn fetch_data(url: &str) -> Result<String, String> {
    Result::Ok(String::from("Data"))
}

async fn process() {
    let data = fetch_data("https://example.com").await;
    match data {
        Result::Ok(content) => println("{}", content),
        Result::Err(e) => println("Error: {}", e),
    }
}
```

## LEXER IMPLEMENTATION REQUIREMENTS

### Token Types
Implement all tokens for:
- Keywords: let, mut, fn, return, if, else, while, for, in, loop, break, continue, match, struct, enum, impl, trait, pub, mod, use, const, static, async, await, true, false, self, Self
- Types: i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64, bool, char, str, String
- Operators: +, -, *, /, %, =, ==, !=, <, <=, >, >=, &&, ||, !, &, &mut, |, ^, <<, >>, +=, -=, *=, /=, %=
- Delimiters: (, ), {, }, [, ], ;, :, ::, ,, ., .., ..=, ->, =>
- Literals: integers, floats, strings, chars, booleans
- Identifiers and comments

### Scanner Features
- Efficient buffered reading
- Line and column tracking
- Unicode support
- String interpolation parsing
- Multi-line string support
- Raw string literals
- Escape sequence handling
- Numeric literal parsing (binary, octal, hex, scientific notation)
- Whitespace and comment skipping
- Error recovery with helpful messages

## PARSER IMPLEMENTATION REQUIREMENTS

### AST Node Types
Complete AST with:
- Expressions: literals, identifiers, binary ops, unary ops, calls, indexing, field access, closures, match expressions, if expressions, block expressions
- Statements: let bindings, assignments, expressions, return, break, continue, while, for, loop
- Declarations: functions, structs, enums, impls, traits, modules, use statements, type aliases
- Types: primitive types, references, mutable references, slices, arrays, tuples, function types, generic types, trait bounds
- Patterns: literals, identifiers, wildcards, tuples, structs, enums, references, slices
- Generics: type parameters, lifetime parameters, where clauses, bounds

### Parsing Strategy
- Recursive descent parser
- Pratt parsing for expressions with proper precedence
- Error recovery with synchronization points
- Multiple error reporting (don't stop at first error)
- Span tracking for every node
- Lookahead when needed
- Left-factoring for efficiency

### Precedence Levels (lowest to highest)
1. Assignment (=, +=, -=, etc.)
2. Logical OR (||)
3. Logical AND (&&)
4. Equality (==, !=)
5. Comparison (<, <=, >, >=)
6. Bitwise OR (|)
7. Bitwise XOR (^)
8. Bitwise AND (&)
9. Shift (<<, >>)
10. Addition/Subtraction (+, -)
11. Multiplication/Division/Modulo (*, /, %)
12. Unary (!, -, &, &mut, *)
13. Call, Index, Field Access

## SEMANTIC ANALYSIS REQUIREMENTS

### Symbol Table
- Nested scopes with proper shadowing
- Separate namespaces for types, values, lifetimes
- Forward declarations support
- Module system with visibility rules
- Generic instantiation tracking
- Trait implementation registration

### Type Checker
Full type inference using Hindley-Milner with extensions:
- Unification algorithm
- Type variables and substitution
- Generalization and instantiation
- Subtyping for trait objects
- Variance checking for generic parameters
- Associated type resolution
- Method resolution with trait bounds
- Implicit conversions (limited)
- Type coercion rules
- Array and slice type checking
- Closure type inference
- Return type checking
- Pattern type checking

### Borrow Checker
Implement full Rust-style borrow checking:
- Ownership tracking for every value
- Move semantics with use-after-move detection
- Shared borrow rules (multiple immutable OR one mutable)
- Lifetime inference and checking
- Lifetime bounds on generics
- Non-lexical lifetimes (NLL)
- Two-phase borrows
- Interior mutability patterns
- Drop check
- Borrow scope calculation
- Conflict detection between borrows
- Mutable aliasing prevention

### Lifetime Analyzer
- Lifetime elision rules
- Lifetime parameter inference
- Lifetime subtyping
- Anonymous lifetimes
- Static lifetime special case
- Lifetime bounds checking
- Function signature lifetime validation

### Control Flow Analysis
- Reachability analysis
- Definite initialization checking
- Return path validation
- Break/continue target validation
- Unreachable code detection

## INTERMEDIATE REPRESENTATION

### IR Design
Create a low-level IR similar to LLVM IR:
- SSA form (Static Single Assignment)
- Basic blocks with control flow graph
- Type-annotated instructions
- Virtual registers (infinite)
- Explicit memory operations
- Function calling convention
- Phi nodes for control flow merging
- Explicit lifetime markers

### IR Instructions
- Arithmetic: add, sub, mul, div, rem, neg
- Bitwise: and, or, xor, not, shl, shr
- Comparison: eq, ne, lt, le, gt, ge
- Memory: alloca, load, store, gep (get element pointer)
- Control: br, condbr, switch, ret, unreachable
- Function: call, invoke, landingpad
- Conversion: trunc, zext, sext, fptrunc, fpext, fptoui, fptosi, uitofp, sitofp, ptrtoint, inttoptr, bitcast
- Aggregate: extractvalue, insertvalue
- Phi: phi nodes for SSA

### Optimization Passes
Implement comprehensive optimizations:

#### Function-Level Optimizations
- Constant folding and propagation
- Dead code elimination
- Common subexpression elimination
- Loop invariant code motion
- Induction variable simplification
- Loop unrolling
- Function inlining
- Tail call optimization
- Peephole optimizations
- Algebraic simplifications
- Strength reduction

#### Global Optimizations
- Global value numbering
- Sparse conditional constant propagation
- Dead store elimination
- Redundant load elimination
- Alias analysis
- Escape analysis
- Devirtualization

#### Control Flow Optimizations
- Branch elimination
- Jump threading
- Loop unswitching
- Basic block merging
- Unreachable code elimination

## CODE GENERATION - LLVM BACKEND

### LLVM Integration
Use inkwell crate for LLVM bindings:
- Module creation and management
- Function generation with correct calling convention
- Type mapping from Blaze to LLVM types
- Basic block generation
- Instruction emission
- PHI node insertion
- Exception handling setup
- Metadata generation for debugging
- Optimization pipeline configuration

### Code Generation Strategy
- Direct translation from IR to LLVM IR
- Register allocation handled by LLVM
- Calling convention adherence
- Stack frame management
- Exception table generation
- Debug information emission (DWARF)
- Profile-guided optimization support

### Target Support
- Multiple architectures: x86_64, ARM, AArch64
- Multiple platforms: Linux, macOS, Windows
- Cross-compilation support
- Target feature detection
- Architecture-specific optimizations

### Linker Integration
- Static linking
- Dynamic linking
- Incremental linking
- Link-time optimization (LTO)
- Symbol resolution
- Library path management
- Executable generation with proper format (ELF, Mach-O, PE)

## RUNTIME SYSTEM

### Memory Allocator
- Custom allocator implementation
- Jemalloc integration option
- Arena allocation for temporary objects
- Reference counting for shared ownership
- Drop flag optimization

### Panic Handler
- Panic message formatting
- Stack unwinding
- Backtrace generation
- Panic hook support
- Abort on panic option

### Intrinsics
Compiler intrinsics for:
- Atomic operations
- SIMD instructions
- Memory operations (memcpy, memset, etc.)
- Type information
- Size and alignment queries
- Overflow checking arithmetic

## STANDARD LIBRARY

### Core Types
```
struct String {
    data: *mut u8,
    len: usize,
    capacity: usize,
}

struct Vec<T> {
    data: *mut T,
    len: usize,
    capacity: usize,
}

struct HashMap<K, V> {
    buckets: Vec<Option<(K, V)>>,
    len: usize,
}

struct HashSet<T> {
    map: HashMap<T, ()>,
}

struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
    len: usize,
}
```

### Collections Module
- Vec: dynamic array with push, pop, insert, remove, sort, binary_search
- HashMap: hash table with insert, get, remove, contains_key, iter
- HashSet: set operations with insert, remove, contains, union, intersection
- LinkedList: doubly-linked list with push_front, push_back, pop_front, pop_back
- BTreeMap: ordered map
- BTreeSet: ordered set
- VecDeque: double-ended queue
- BinaryHeap: priority queue

### I/O Module
- File operations: open, read, write, seek, metadata
- Buffered I/O: BufferedReader, BufferedWriter
- Standard streams: stdin, stdout, stderr
- Path manipulation
- Directory operations
- Network I/O: TcpStream, TcpListener, UdpSocket

### String Module
- UTF-8 validation
- String operations: concat, split, trim, replace, contains, starts_with, ends_with
- Pattern matching
- Case conversion
- Formatting with interpolation

### Math Module
- Basic operations: abs, min, max, clamp
- Trigonometry: sin, cos, tan, asin, acos, atan
- Exponential: exp, log, log2, log10, pow, sqrt
- Rounding: floor, ceil, round, trunc
- Constants: PI, E, TAU

### Sync Module
- Mutex: mutual exclusion lock
- RwLock: reader-writer lock
- Atomic types: AtomicBool, AtomicI32, AtomicI64, AtomicUsize
- Channel: mpsc (multi-producer, single-consumer)
- Barrier: synchronization barrier
- Condvar: condition variable

## ERROR HANDLING AND DIAGNOSTICS

### Error Message Format
```
error[E0001]: cannot move out of borrowed content
  --> example.blz:5:13
   |
4  |     let s = String::from("hello");
   |         - move occurs because `s` has type `String`, which does not implement the `Copy` trait
5  |     let t = s;
   |             ^ value moved here
6  |     println("{}", s);
   |                   ^ value used here after move
   |
help: consider cloning the value if the performance cost is acceptable
   |
5  |     let t = s.clone();
   |              ++++++++
```

### Error Categories
- Lexical errors: invalid characters, unterminated strings
- Syntax errors: unexpected tokens, missing delimiters
- Type errors: type mismatch, undefined types, invalid operations
- Borrow errors: use after move, multiple mutable borrows, lifetime violations
- Name resolution errors: undefined variables, unresolved imports
- Trait errors: unimplemented traits, trait bound violations
- Visibility errors: private access violations

### Error Recovery
- Panic mode with synchronization tokens
- Phrase-level recovery
- Error production rules
- Multiple error reporting
- Suggestive error messages with code fixes

## TESTING REQUIREMENTS

### Unit Tests
Write comprehensive unit tests for:

#### Lexer Tests
- All token types
- String literals with escapes
- Numeric literals in all bases
- Comments (single-line, multi-line)
- Unicode identifiers
- Error cases: invalid characters, unterminated strings
- Edge cases: empty input, very long tokens

#### Parser Tests
- Every expression type
- Every statement type
- Operator precedence and associativity
- Complex nested structures
- Error recovery
- Edge cases: empty blocks, trailing commas

#### Type Checker Tests
- Basic type checking
- Generic type inference
- Trait resolution
- Method lookup
- Type errors with helpful messages
- Complex generic scenarios
- Associated types

#### Borrow Checker Tests
- Simple ownership transfer
- Shared borrowing
- Mutable borrowing
- Lifetime inference
- Common error patterns
- Multiple borrow conflicts
- Closure captures

#### Optimization Tests
- Constant folding correctness
- Dead code elimination
- Inlining decisions
- Loop optimizations
- Optimization level effects

### Integration Tests
Full programs that test:
- Hello world
- Fibonacci (recursive and iterative)
- Sorting algorithms
- Data structure implementations
- File I/O operations
- Network client/server
- Multi-threaded programs
- Generic programming
- Trait objects
- Complex ownership patterns

### Benchmarks
Performance benchmarks using criterion:
- Lexer speed on large files
- Parser speed on complex code
- Compilation time
- Generated code performance
- Memory usage
- Standard library operations

## CLI INTERFACE

Command-line interface with subcommands:

```
blaze build [OPTIONS] <SOURCE>
  --release              Build with optimizations
  --target <TARGET>      Target architecture
  --emit <TYPE>          Emit type (asm, llvm-ir, mir, exe)
  --out <PATH>           Output file path
  -O <LEVEL>             Optimization level (0, 1, 2, 3)
  --verbose              Verbose output
  --color <WHEN>         Colorize output (auto, always, never)

blaze run [OPTIONS] <SOURCE> [ARGS]
  --release              Run release build
  --                     Pass arguments to program

blaze test [OPTIONS] [TESTNAME]
  --                     Pass arguments to test

blaze check <SOURCE>
  Quick syntax and type check without codegen

blaze fmt <SOURCE>
  Format source code

blaze doc [OPTIONS]
  Generate documentation

blaze clean
  Remove build artifacts

blaze init <NAME>
  Create new project

blaze --version
  Print version

blaze --help
  Print help
```

## DOCUMENTATION REQUIREMENTS

### Getting Started Guide
- Installation instructions
- First program tutorial
- Basic syntax overview
- Ownership concepts explained
- Common patterns

### Syntax Reference
- Complete language grammar in BNF
- Every keyword explained
- Operator precedence table
- Example for every feature

### Ownership Guide
- Ownership rules detailed
- Borrowing patterns
- Lifetime annotations
- Common pitfalls and solutions
- Migration from GC languages

### Standard Library Docs
- Every type documented
- Every function documented
- Usage examples
- Performance characteristics
- Related functions

### Compiler Internals
- Architecture overview
- Each compilation phase explained
- IR specification
- Optimization passes documented
- Extension points

## EXAMPLE PROGRAMS

### Hello World
```
fn main() {
    println("Hello, Blaze!");
}
```

### Ownership Demo
```
fn main() {
    let s1 = String::from("hello");
    let s2 = s1;
    
    let s3 = String::from("world");
    let len = calculate_length(&s3);
    println("Length of '{}' is {}", s3, len);
    
    let mut s4 = String::from("hello");
    change(&mut s4);
    println("{}", s4);
}

fn calculate_length(s: &String) -> usize {
    s.len()
}

fn change(s: &mut String) {
    s.push_str(", world");
}
```

### Structs and Methods
```
struct Rectangle {
    width: f64,
    height: f64,
}

impl Rectangle {
    fn new(width: f64, height: f64) -> Rectangle {
        Rectangle { width, height }
    }
    
    fn area(&self) -> f64 {
        self.width * self.height
    }
    
    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
    
    fn square(size: f64) -> Rectangle {
        Rectangle {
            width: size,
            height: size,
        }
    }
}

fn main() {
    let rect1 = Rectangle::new(30.0, 50.0);
    let rect2 = Rectangle::new(10.0, 40.0);
    let rect3 = Rectangle::square(25.0);
    
    println("Area: {}", rect1.area());
    println("Can hold rect2: {}", rect1.can_hold(&rect2));
    println("Can hold rect3: {}", rect1.can_hold(&rect3));
}
```

### Enums and Pattern Matching
```
enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter,
}

fn value_in_cents(coin: Coin) -> i32 {
    match coin {
        Coin::Penny => 1,
        Coin::Nickel => 5,
        Coin::Dime => 10,
        Coin::Quarter => 25,
    }
}

enum Option<T> {
    Some(T),
    None,
}

fn plus_one(x: Option<i32>) -> Option<i32> {
    match x {
        Option::Some(i) => Option::Some(i + 1),
        Option::None => Option::None,
    }
}

fn main() {
    let penny = Coin::Penny;
    println("Value: {}", value_in_cents(penny));
    
    let five = Option::Some(5);
    let six = plus_one(five);
    let none = plus_one(Option::None);
}
```

### Generics and Traits
```
trait Summary {
    fn summarize(&self) -> String;
}

struct NewsArticle {
    headline: String,
    content: String,
}

impl Summary for NewsArticle {
    fn summarize(&self) -> String {
        format!("{}: {}", self.headline, self.content)
    }
}

struct Tweet {
    username: String,
    content: String,
}

impl Summary for Tweet {
    fn summarize(&self) -> String {
        format!("@{}: {}", self.username, self.content)
    }
}

fn notify<T: Summary>(item: &T) {
    println("Breaking news! {}", item.summarize());
}

struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    fn x(&self) -> &T {
        &self.x
    }
}

fn main() {
    let article = NewsArticle {
        headline: String::from("Big News"),
        content: String::from("Something happened"),
    };
    
    let tweet = Tweet {
        username: String::from("user123"),
        content: String::from("Hello world"),
    };
    
    notify(&article);
    notify(&tweet);
    
    let integer_point = Point { x: 5, y: 10 };
    let float_point = Point { x: 1.0, y: 4.0 };
}
```

### Collections and Iterators
```
fn main() {
    let mut vec = Vec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    
    for item in &vec {
        println("{}", item);
    }
    
    let doubled: Vec<i32> = vec.iter().map(|x| x * 2).collect();
    let sum: i32 = vec.iter().sum();
    
    let mut map = HashMap::new();
    map.insert(String::from("Blue"), 10);
    map.insert(String::from("Red"), 50);
    
    for (key, value) in &map {
        println("{}: {}", key, value);
    }
    
    let mut set = HashSet::new();
    set.insert(1);
    set.insert(2);
    set.insert(3);
    
    if set.contains(&2) {
        println("Set contains 2");
    }
}
```

### Error Handling with Result
```
use std::io;
use std::fs::File;

fn read_username_from_file(path: &str) -> Result<String, io::Error> {
    let file = File::open(path)?;
    let mut username = String::new();
    file.read_to_string(&mut username)?;
    Result::Ok(username)
}

fn main() {
    match read_username_from_file("username.txt") {
        Result::Ok(username) => println("Username: {}", username),
        Result::Err(e) => println("Error reading file: {}", e),
    }
}
```

### Closures and Functional Programming
```
fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    
    let sum: i32 = numbers.iter().sum();
    let product: i32 = numbers.iter().product();
    
    let evens: Vec<i32> = numbers.iter()
        .filter(|x| *x % 2 == 0)
        .map(|x| *x)
        .collect();
    
    let expensive_closure = |num: i32| -> i32 {
        println("Calculating...");
        num * 2
    };
    
    let result = expensive_closure(5);
    
    let mut counter = 0;
    let mut incrementer = || {
        counter += 1;
        counter
    };
    
    println("{}", incrementer());
    println("{}", incrementer());
}
```

### Multi-threading
```
use std::sync::{Mutex, Arc};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    
    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println("Result: {}", *counter.lock().unwrap());
}
```

### Async/Await
```
async fn fetch_data(url: &str) -> Result<String, String> {
    println("Fetching from {}", url);
    Result::Ok(String::from("Data from server"))
}

async fn process_data() {
    let data1 = fetch_data("https://api1.com").await;
    let data2 = fetch_data("https://api2.com").await;
    
    match (data1, data2) {
        (Result::Ok(d1), Result::Ok(d2)) => {
            println("Got: {} and {}", d1, d2);
        },
        _ => println("Error fetching data"),
    }
}

fn main() {
    let runtime = Runtime::new();
    runtime.block_on(process_data());
}
```

### Web Server Example
```
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    
    let response = "HTTP/1.1 200 OK\r\n\r\nHello from Blaze!";
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println("Server listening on port 8080");
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_client(stream);
                });
            },
            Err(e) => println("Error: {}", e),
        }
    }
}
```

### Game Engine Example
```
struct GameObject {
    position: Vec2,
    velocity: Vec2,
    sprite: String,
}

struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn new(x: f32, y: f32) -> Vec2 {
        Vec2 { x, y }
    }
    
    fn add(&self, other: &Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
    
    fn scale(&self, scalar: f32) -> Vec2 {
        Vec2::new(self.x * scalar, self.y * scalar)
    }
}

impl GameObject {
    fn new(x: f32, y: f32, sprite: String) -> GameObject {
        GameObject {
            position: Vec2::new(x, y),
            velocity: Vec2::new(0.0, 0.0),
            sprite,
        }
    }
    
    fn update(&mut self, delta_time: f32) {
        let displacement = self.velocity.scale(delta_time);
        self.position = self.position.add(&displacement);
    }
    
    fn apply_force(&mut self, force: Vec2) {
        self.velocity = self.velocity.add(&force);
    }
}

fn main() {
    let mut player = GameObject::new(100.0, 100.0, String::from("player.png"));
    player.apply_force(Vec2::new(10.0, 0.0));
    player.update(0.016);
    
    println("Player position: ({}, {})", player.position.x, player.position.y);
}
```

## CARGO.TOML CONFIGURATION

```toml
[package]
name = "blaze"
version = "0.1.0"
edition = "2021"
authors = ["Blaze Contributors"]
description = "A systems programming language with Rust-like safety guarantees"
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourusername/blaze"

[[bin]]
name = "blaze"
path = "src/main.rs"

[lib]
name = "blaze_compiler"
path = "src/lib.rs"

[dependencies]
inkwell = { version = "0.4", features = ["llvm17-0"] }
clap = { version = "4.5", features = ["derive", "color"] }
ariadne = "0.4"
codespan-reporting = "0.11"
logos = "0.14"
nom = "7.1"
colored = "2.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
anyhow = "1.0"
thiserror = "1.0"
walkdir = "2.5"
rayon = "1.10"
dashmap = "6.0"
parking_lot = "0.12"
once_cell = "1.19"
crossbeam = "0.8"
indexmap = "2.2"
petgraph = "0.6"
ahash = "0.8"
smallvec = "1.13"
arrayvec = "0.7"
bumpalo = "3.16"
regex = "1.10"
unicode-xid = "0.2"
unicode-normalization = "0.1"

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
proptest = "1.4"
quickcheck = "1.0"
pretty_assertions = "1.4"
insta = "1.39"
tempfile = "3.10"

[profile.dev]
opt-level = 0
debug = true
split-debuginfo = "unpacked"

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"

[profile.bench]
inherits = "release"

[[bench]]
name = "lexer_bench"
harness = false

[[bench]]
name = "parser_bench"
harness = false

[[bench]]
name = "codegen_bench"
harness = false
```

## IMPLEMENTATION PRIORITIES AND PHASES

### Phase 1: Core Compiler (Week 1-4)
1. Lexer with all tokens and error recovery
2. Parser for basic expressions, statements, functions
3. Basic AST with complete node types
4. Simple type checker (no generics yet)
5. Basic borrow checker (simple ownership rules)
6. IR generation from AST
7. Basic LLVM codegen
8. CLI with build, run, check commands
9. Comprehensive tests for all components

### Phase 2: Advanced Features (Week 5-8)
1. Generics with type parameters
2. Trait system with implementations
3. Pattern matching with exhaustiveness checking
4. Advanced borrow checking with NLL
5. Lifetime inference and checking
6. Module system with visibility
7. Optimization passes
8. Standard library core types (String, Vec, HashMap)
9. Error recovery and helpful diagnostics

### Phase 3: Standard Library (Week 9-12)
1. Collections: Vec, HashMap, HashSet, LinkedList, BTreeMap, BTreeSet
2. I/O: File, buffered I/O, stdin/stdout/stderr
3. String operations and formatting
4. Math functions
5. Iterators and functional programming
6. Smart pointers: Box, Rc, Arc
7. Synchronization primitives: Mutex, RwLock, Channel
8. Thread spawning and joining

### Phase 4: Advanced Compiler Features (Week 13-16)
1. Async/await transformation
2. Macro system (declarative and procedural)
3. Inline assembly
4. Foreign function interface (FFI)
5. Link-time optimization
6. Cross-compilation
7. Incremental compilation
8. Parallel compilation

### Phase 5: Tooling and Polish (Week 17-20)
1. Code formatter
2. Documentation generator
3. Language server protocol (LSP) implementation
4. Package manager
5. Build system improvements
6. Debugger integration (LLDB/GDB)
7. Profiler integration
8. Comprehensive benchmarks

## TESTING STRATEGY

### Unit Test Coverage Requirements
- Lexer: 100% line coverage, all token types, all error cases
- Parser: 100% coverage of grammar rules, error recovery paths
- Type checker: All type rules, all error messages, edge cases
- Borrow checker: All ownership rules, all borrow conflicts
- Codegen: Instruction correctness, optimization validity
- Standard library: Every public function, boundary conditions

### Test Organization
```
tests/
├── lexer/
│   ├── tokens.rs
│   ├── literals.rs
│   ├── operators.rs
│   ├── keywords.rs
│   ├── comments.rs
│   ├── errors.rs
│   └── unicode.rs
├── parser/
│   ├── expressions.rs
│   ├── statements.rs
│   ├── functions.rs
│   ├── structs.rs
│   ├── enums.rs
│   ├── traits.rs
│   ├── generics.rs
│   ├── patterns.rs
│   ├── precedence.rs
│   └── errors.rs
├── semantic/
│   ├── type_checking.rs
│   ├── type_inference.rs
│   ├── borrow_checking.rs
│   ├── lifetime_checking.rs
│   ├── trait_resolution.rs
│   ├── method_lookup.rs
│   └── errors.rs
├── codegen/
│   ├── basic_codegen.rs
│   ├── functions.rs
│   ├── structs.rs
│   ├── enums.rs
│   ├── traits.rs
│   ├── closures.rs
│   └── optimizations.rs
├── integration/
│   ├── hello_world.rs
│   ├── ownership.rs
│   ├── borrowing.rs
│   ├── generics.rs
│   ├── traits.rs
│   ├── collections.rs
│   ├── io.rs
│   ├── threads.rs
│   └── async.rs
└── stdlib/
    ├── collections_test.rs
    ├── io_test.rs
    ├── string_test.rs
    ├── math_test.rs
    └── sync_test.rs
```

### Property-Based Testing
Use proptest for:
- Parser round-tripping (parse -> pretty-print -> parse)
- Type checker soundness
- Optimization correctness (optimized code has same behavior)
- Collections invariants
- Borrow checker completeness

### Fuzzing
Implement fuzzing for:
- Lexer (arbitrary byte sequences)
- Parser (arbitrary token sequences)
- Type checker (arbitrary ASTs)
- Optimizer (arbitrary IR)

## PERFORMANCE REQUIREMENTS

### Compilation Speed Targets
- Lexing: > 50 MB/s
- Parsing: > 20 MB/s
- Type checking: < 1ms per 100 lines
- Full compilation: < 100ms for 1000 line program
- Incremental compilation: < 10ms for single function change

### Generated Code Performance
- Within 10% of equivalent Rust code
- Zero-cost abstractions (no runtime overhead)
- Optimal register allocation
- Effective inlining
- Dead code elimination
- Loop optimizations comparable to Clang

### Memory Usage
- Compiler peak memory: < 500 MB for 10k line program
- Memory safety without GC overhead
- Efficient stack usage
- Minimal heap allocations in hot paths

## ERROR MESSAGE QUALITY

### Error Message Requirements
Every error must have:
1. Error code (e.g., E0001)
2. Primary message explaining what went wrong
3. Source code snippet with highlighting
4. Line and column numbers
5. Secondary labels explaining context
6. Help message with suggested fix
7. Related documentation link

### Example Error Messages

#### Ownership Error
```
error[E0382]: use of moved value: `s`
  --> example.blz:5:20
   |
3  |     let s = String::from("hello");
   |         - move occurs because `s` has type `String`
4  |     let t = s;
   |             - value moved here
5  |     println("{}", s);
   |                    ^ value used here after move
   |
   = note: move occurs because `String` does not implement the `Copy` trait
help: consider cloning the value if you want to use it after the move
   |
4  |     let t = s.clone();
   |              ++++++++
```

#### Type Error
```
error[E0308]: mismatched types
  --> example.blz:3:24
   |
3  |     let result: i32 = "hello";
   |                 ---   ^^^^^^^ expected `i32`, found `&str`
   |                 |
   |                 expected due to this type
   |
help: try using a conversion method
   |
3  |     let result: i32 = "hello".parse().unwrap();
   |                              ++++++++++++++++
```

#### Borrow Error
```
error[E0502]: cannot borrow `x` as mutable because it is also borrowed as immutable
  --> example.blz:4:5
   |
3  |     let r1 = &x;
   |              -- immutable borrow occurs here
4  |     let r2 = &mut x;
   |              ^^^^^^ mutable borrow occurs here
5  |     println("{}", r1);
   |                   -- immutable borrow later used here
   |
   = help: consider restructuring your code to avoid simultaneous borrows
```

#### Lifetime Error
```
error[E0106]: missing lifetime specifier
  --> example.blz:1:36
   |
1  | fn longest(x: &str, y: &str) -> &str {
   |               ----     ----     ^ expected named lifetime parameter
   |
   = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `x` or `y`
help: consider introducing a named lifetime parameter
   |
1  | fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
   |           ++++     ++          ++          ++
```

## CODE QUALITY STANDARDS

### Rust Code Style
- Use rustfmt with default settings
- Follow Rust API guidelines
- Prefer iterators over loops
- Use type inference where clear
- Explicit error handling (no unwrap in production)
- Comprehensive documentation for public APIs
- Use derive macros where applicable
- Prefer match over if-let chains
- Use clippy and fix all warnings

### Performance Best Practices
- Avoid allocations in hot paths
- Use arena allocation for temporary data
- Batch operations when possible
- Parallel processing with rayon
- Use efficient data structures (BTreeMap for ordered, HashMap for unordered)
- Profile before optimizing
- Benchmark critical paths

### Safety Standards
- No unsafe code unless absolutely necessary
- Unsafe blocks must have safety comments
- Validate all inputs
- Check all array accesses
- Handle all error cases
- No panics in library code (use Result)

## OPTIMIZATION IMPLEMENTATION DETAILS

### Constant Folding
```rust
fn fold_constants(expr: Expr) -> Expr {
    match expr {
        Expr::Binary { op: BinOp::Add, left, right } => {
            match (fold_constants(*left), fold_constants(*right)) {
                (Expr::IntLit(a), Expr::IntLit(b)) => Expr::IntLit(a + b),
                (l, r) => Expr::Binary {
                    op: BinOp::Add,
                    left: Box::new(l),
                    right: Box::new(r),
                },
            }
        }
        _ => expr,
    }
}
```

### Dead Code Elimination
```rust
fn eliminate_dead_code(blocks: &mut Vec<BasicBlock>) {
    let mut reachable = HashSet::new();
    let mut worklist = vec![0];
    
    while let Some(block_id) = worklist.pop() {
        if reachable.insert(block_id) {
            for successor in blocks[block_id].successors() {
                worklist.push(successor);
            }
        }
    }
    
    blocks.retain(|block| reachable.contains(&block.id));
}
```

### Common Subexpression Elimination
```rust
fn eliminate_common_subexpressions(blocks: &mut Vec<BasicBlock>) {
    let mut available = HashMap::new();
    
    for block in blocks {
        for instr in &mut block.instructions {
            if let Some(existing) = available.get(&instr.expression()) {
                *instr = Instruction::Copy(*existing);
            } else {
                available.insert(instr.expression(), instr.destination());
            }
        }
    }
}
```

### Loop Invariant Code Motion
```rust
fn hoist_loop_invariants(cfg: &mut ControlFlowGraph) {
    for loop_header in cfg.loop_headers() {
        let loop_blocks = cfg.loop_blocks(loop_header);
        let mut invariants = Vec::new();
        
        for block in loop_blocks {
            for instr in &block.instructions {
                if is_loop_invariant(instr, &loop_blocks) {
                    invariants.push(instr.clone());
                }
            }
        }
        
        let preheader = cfg.create_preheader(loop_header);
        preheader.instructions.extend(invariants);
    }
}
```

### Inlining
```rust
fn inline_functions(module: &mut Module, threshold: usize) {
    let mut changed = true;
    
    while changed {
        changed = false;
        
        for func in &mut module.functions {
            for block in &mut func.blocks {
                for instr in &mut block.instructions {
                    if let Instruction::Call { callee, args } = instr {
                        let callee_func = module.get_function(callee);
                        if should_inline(callee_func, threshold) {
                            inline_call(block, instr, callee_func, args);
                            changed = true;
                        }
                    }
                }
            }
        }
    }
}
```

## BORROW CHECKER IMPLEMENTATION DETAILS

### Ownership Tracking
```rust
struct OwnershipState {
    variables: HashMap<VarId, OwnershipStatus>,
    borrows: Vec<Borrow>,
}

enum OwnershipStatus {
    Owned,
    Moved { to: VarId, at: Span },
    Borrowed { count: usize },
}

struct Borrow {
    variable: VarId,
    is_mutable: bool,
    lifetime: Lifetime,
    location: Span,
}

fn check_move(state: &mut OwnershipState, var: VarId, location: Span) -> Result<(), BorrowError> {
    match state.variables.get(&var) {
        Some(OwnershipStatus::Moved { to, at }) => {
            Err(BorrowError::UseAfterMove {
                var,
                moved_to: *to,
                moved_at: *at,
                used_at: location,
            })
        }
        Some(OwnershipStatus::Borrowed { .. }) => {
            Err(BorrowError::MoveWhileBorrowed {
                var,
                location,
            })
        }
        _ => {
            state.variables.insert(var, OwnershipStatus::Moved {
                to: var,
                at: location,
            });
            Ok(())
        }
    }
}
```

### Borrow Conflict Detection
```rust
fn check_borrow_conflict(
    existing: &[Borrow],
    new: &Borrow,
) -> Result<(), BorrowError> {
    for existing_borrow in existing {
        if existing_borrow.variable == new.variable {
            if new.is_mutable || existing_borrow.is_mutable {
                if lifetimes_overlap(&existing_borrow.lifetime, &new.lifetime) {
                    return Err(BorrowError::ConflictingBorrows {
                        first: existing_borrow.clone(),
                        second: new.clone(),
                    });
                }
            }
        }
    }
    Ok(())
}
```

### Lifetime Inference
```rust
struct LifetimeInferenceContext {
    constraints: Vec<LifetimeConstraint>,
    lifetimes: HashMap<LifetimeVar, Lifetime>,
}

enum LifetimeConstraint {
    Outlives { longer: LifetimeVar, shorter: LifetimeVar },
    Equal { left: LifetimeVar, right: LifetimeVar },
}

fn infer_lifetimes(func: &Function) -> Result<HashMap<LifetimeVar, Lifetime>, TypeError> {
    let mut ctx = LifetimeInferenceContext::new();
    
    for stmt in &func.body {
        collect_lifetime_constraints(&mut ctx, stmt);
    }
    
    solve_lifetime_constraints(&mut ctx)?;
    
    Ok(ctx.lifetimes)
}
```

## TYPE INFERENCE IMPLEMENTATION

### Hindley-Milner Type Inference
```rust
struct TypeInferenceContext {
    substitutions: HashMap<TypeVar, Type>,
    constraints: Vec<TypeConstraint>,
    next_type_var: usize,
}

enum TypeConstraint {
    Equal(Type, Type),
    HasTrait(Type, TraitId),
}

fn infer_types(expr: &Expr, ctx: &mut TypeInferenceContext) -> Result<Type, TypeError> {
    match expr {
        Expr::IntLit(_) => Ok(Type::I32),
        Expr::BoolLit(_) => Ok(Type::Bool),
        Expr::Var(name) => {
            ctx.lookup_variable(name)
                .ok_or_else(|| TypeError::UndefinedVariable(name.clone()))
        }
        Expr::Binary { op, left, right } => {
            let left_ty = infer_types(left, ctx)?;
            let right_ty = infer_types(right, ctx)?;
            
            ctx.constraints.push(TypeConstraint::Equal(left_ty.clone(), right_ty.clone()));
            
            match op {
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => Ok(left_ty),
                BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                    Ok(Type::Bool)
                }
                _ => Err(TypeError::InvalidBinaryOp),
            }
        }
        Expr::Call { func, args } => {
            let func_ty = infer_types(func, ctx)?;
            let arg_tys: Vec<Type> = args
                .iter()
                .map(|arg| infer_types(arg, ctx))
                .collect::<Result<_, _>>()?;
            
            match func_ty {
                Type::Function { params, ret } => {
                    if params.len() != arg_tys.len() {
                        return Err(TypeError::ArgCountMismatch);
                    }
                    
                    for (param_ty, arg_ty) in params.iter().zip(arg_tys.iter()) {
                        ctx.constraints.push(TypeConstraint::Equal(
                            param_ty.clone(),
                            arg_ty.clone(),
                        ));
                    }
                    
                    Ok(*ret)
                }
                _ => Err(TypeError::NotCallable),
            }
        }
        _ => todo!(),
    }
}

fn unify(t1: &Type, t2: &Type, ctx: &mut TypeInferenceContext) -> Result<(), TypeError> {
    match (t1, t2) {
        (Type::Var(v1), Type::Var(v2)) if v1 == v2 => Ok(()),
        (Type::Var(v), t) | (t, Type::Var(v)) => {
            if occurs_check(*v, t) {
                Err(TypeError::InfiniteType)
            } else {
                ctx.substitutions.insert(*v, t.clone());
                Ok(())
            }
        }
        (Type::I32, Type::I32) | (Type::Bool, Type::Bool) => Ok(()),
        (Type::Function { params: p1, ret: r1 }, Type::Function { params: p2, ret: r2 }) => {
            if p1.len() != p2.len() {
                return Err(TypeError::TypeMismatch);
            }
            
            for (param1, param2) in p1.iter().zip(p2.iter()) {
                unify(param1, param2, ctx)?;
            }
            
            unify(r1, r2, ctx)
        }
        _ => Err(TypeError::TypeMismatch),
    }
}
```

## LLVM CODE GENERATION

### Module and Function Setup
```rust
use inkwell::context::Context;
use inkwell::builder::Builder;
use inkwell::module::Module;
use inkwell::values::FunctionValue;

fn codegen_module(ast_module: &AstModule) -> Module {
    let context = Context::create();
    let module = context.create_module("blaze_module");
    let builder = context.create_builder();
    
    for function in &ast_module.functions {
        codegen_function(&context, &module, &builder, function);
    }
    
    module
}

fn codegen_function(
    context: &Context,
    module: &Module,
    builder: &Builder,
    function: &AstFunction,
) -> FunctionValue {
    let param_types: Vec<BasicMetadataTypeEnum> = function
        .params
        .iter()
        .map(|p| codegen_type(context, &p.ty).into())
        .collect();
    
    let fn_type = match &function.return_type {
        Some(ret_ty) => codegen_type(context, ret_ty).fn_type(&param_types, false),
        None => context.void_type().fn_type(&param_types, false),
    };
    
    let fn_value = module.add_function(&function.name, fn_type, None);
    let entry_block = context.append_basic_block(fn_value, "entry");
    builder.position_at_end(entry_block);
    
    let mut variables = HashMap::new();
    for (i, param) in function.params.iter().enumerate() {
        let param_value = fn_value.get_nth_param(i as u32).unwrap();
        param_value.set_name(&param.name);
        variables.insert(param.name.clone(), param_value);
    }
    
    for stmt in &function.body {
        codegen_statement(context, builder, &mut variables, stmt);
    }
    
    if !builder.get_insert_block().unwrap().get_terminator().is_some() {
        if function.return_type.is_none() {
            builder.build_return(None);
        }
    }
    
    fn_value
}
```

### Expression Code Generation
```rust
fn codegen_expr(
    context: &Context,
    builder: &Builder,
    variables: &HashMap<String, BasicValueEnum>,
    expr: &Expr,
) -> BasicValueEnum {
    match expr {
        Expr::IntLit(n) => context.i32_type().const_int(*n as u64, false).into(),
        Expr::BoolLit(b) => context.bool_type().const_int(if *b { 1 } else { 0 }, false).into(),
        Expr::Var(name) => *variables.get(name).expect("Undefined variable"),
        Expr::Binary { op, left, right } => {
            let lhs = codegen_expr(context, builder, variables, left).into_int_value();
            let rhs = codegen_expr(context, builder, variables, right).into_int_value();
            
            match op {
                BinOp::Add => builder.build_int_add(lhs, rhs, "addtmp").into(),
                BinOp::Sub => builder.build_int_sub(lhs, rhs, "subtmp").into(),
                BinOp::Mul => builder.build_int_mul(lhs, rhs, "multmp").into(),
                BinOp::Div => builder.build_int_signed_div(lhs, rhs, "divtmp").into(),
                BinOp::Eq => builder.build_int_compare(IntPredicate::EQ, lhs, rhs, "eqtmp").into(),
                BinOp::Ne => builder.build_int_compare(IntPredicate::NE, lhs, rhs, "netmp").into(),
                BinOp::Lt => builder.build_int_compare(IntPredicate::SLT, lhs, rhs, "lttmp").into(),
                BinOp::Le => builder.build_int_compare(IntPredicate::SLE, lhs, rhs, "letmp").into(),
                BinOp::Gt => builder.build_int_compare(IntPredicate::SGT, lhs, rhs, "gttmp").into(),
                BinOp::Ge => builder.build_int_compare(IntPredicate::SGE, lhs, rhs, "getmp").into(),
                _ => panic!("Unsupported binary operator"),
            }
        }
        Expr::Call { func, args } => {
            let func_name = if let Expr::Var(name) = &**func {
                name
            } else {
                panic!("Only direct function calls supported");
            };
            
            let function = builder
                .get_insert_block()
                .unwrap()
                .get_parent()
                .unwrap()
                .get_module()
                .get_function(func_name)
                .expect("Function not found");
            
            let arg_values: Vec<BasicMetadataValueEnum> = args
                .iter()
                .map(|arg| codegen_expr(context, builder, variables, arg).into())
                .collect();
            
            builder
                .build_call(function, &arg_values, "calltmp")
                .try_as_basic_value()
                .left()
                .unwrap()
        }
        Expr::If { condition, then_branch, else_branch } => {
            let cond_value = codegen_expr(context, builder, variables, condition)
                .into_int_value();
            
            let parent_fn = builder.get_insert_block().unwrap().get_parent().unwrap();
            
            let then_block = context.append_basic_block(parent_fn, "then");
            let else_block = context.append_basic_block(parent_fn, "else");
            let merge_block = context.append_basic_block(parent_fn, "ifcont");
            
            builder.build_conditional_branch(cond_value, then_block, else_block);
            
            builder.position_at_end(then_block);
            let then_value = codegen_expr(context, builder, variables, then_branch);
            builder.build_unconditional_branch(merge_block);
            let then_block = builder.get_insert_block().unwrap();
            
            builder.position_at_end(else_block);
            let else_value = codegen_expr(context, builder, variables, else_branch);
            builder.build_unconditional_branch(merge_block);
            let else_block = builder.get_insert_block().unwrap();
            
            builder.position_at_end(merge_block);
            let phi = builder.build_phi(then_value.get_type(), "iftmp");
            phi.add_incoming(&[(&then_value, then_block), (&else_value, else_block)]);
            phi.as_basic_value()
        }
        _ => panic!("Unsupported expression type"),
    }
}
```

### Statement Code Generation
```rust
fn codegen_statement(
    context: &Context,
    builder: &Builder,
    variables: &mut HashMap<String, BasicValueEnum>,
    stmt: &Statement,
) {
    match stmt {
        Statement::Let { name, value, .. } => {
            let value = codegen_expr(context, builder, variables, value);
            variables.insert(name.clone(), value);
        }
        Statement::Return { value } => {
            if let Some(expr) = value {
                let ret_value = codegen_expr(context, builder, variables, expr);
                builder.build_return(Some(&ret_value));
            } else {
                builder.build_return(None);
            }
        }
        Statement::Expr(expr) => {
            codegen_expr(context, builder, variables, expr);
        }
        Statement::While { condition, body } => {
            let parent_fn = builder.get_insert_block().unwrap().get_parent().unwrap();
            
            let loop_cond = context.append_basic_block(parent_fn, "loopcond");
            let loop_body = context.append_basic_block(parent_fn, "loopbody");
            let loop_end = context.append_basic_block(parent_fn, "loopend");
            
            builder.build_unconditional_branch(loop_cond);
            
            builder.position_at_end(loop_cond);
            let cond_value = codegen_expr(context, builder, variables, condition)
                .into_int_value();
            builder.build_conditional_branch(cond_value, loop_body, loop_end);
            
            builder.position_at_end(loop_body);
            for stmt in body {
                codegen_statement(context, builder, variables, stmt);
            }
            builder.build_unconditional_branch(loop_cond);
            
            builder.position_at_end(loop_end);
        }
        _ => panic!("Unsupported statement type"),
    }
}
```

### Type Mapping
```rust
fn codegen_type(context: &Context, ty: &Type) -> BasicTypeEnum {
    match ty {
        Type::I32 => context.i32_type().into(),
        Type::I64 => context.i64_type().into(),
        Type::F32 => context.f32_type().into(),
        Type::F64 => context.f64_type().into(),
        Type::Bool => context.bool_type().into(),
        Type::Pointer(inner) => codegen_type(context, inner)
            .ptr_type(AddressSpace::default())
            .into(),
        Type::Array(inner, size) => codegen_type(context, inner)
            .array_type(*size as u32)
            .into(),
        Type::Struct(fields) => {
            let field_types: Vec<BasicTypeEnum> = fields
                .iter()
                .map(|f| codegen_type(context, &f.ty))
                .collect();
            context.struct_type(&field_types, false).into()
        }
        _ => panic!("Unsupported type"),
    }
}
```

## STANDARD LIBRARY IMPLEMENTATION DETAILS

### Vec Implementation
```rust
pub struct Vec<T> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
}

impl<T> Vec<T> {
    pub fn new() -> Self {
        Vec {
            ptr: std::ptr::null_mut(),
            len: 0,
            capacity: 0,
        }
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        let ptr = if capacity == 0 {
            std::ptr::null_mut()
        } else {
            let layout = std::alloc::Layout::array::<T>(capacity).unwrap();
            unsafe { std::alloc::alloc(layout) as *mut T }
        };
        
        Vec {
            ptr,
            len: 0,
            capacity,
        }
    }
    
    pub fn push(&mut self, value: T) {
        if self.len == self.capacity {
            self.grow();
        }
        
        unsafe {
            std::ptr::write(self.ptr.add(self.len), value);
        }
        self.len += 1;
    }
    
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(std::ptr::read(self.ptr.add(self.len))) }
        }
    }
    
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe { Some(&*self.ptr.add(index)) }
        } else {
            None
        }
    }
    
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index < self.len {
            unsafe { Some(&mut *self.ptr.add(index)) }
        } else {
            None
        }
    }
    
    pub fn len(&self) -> usize {
        self.len
    }
    
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    
    fn grow(&mut self) {
        let new_capacity = if self.capacity == 0 {
            1
        } else {
            self.capacity * 2
        };
        
        let new_layout = std::alloc::Layout::array::<T>(new_capacity).unwrap();
        let new_ptr = unsafe { std::alloc::alloc(new_layout) as *mut T };
        
        if !self.ptr.is_null() {
            unsafe {
                std::ptr::copy_nonoverlapping(self.ptr, new_ptr, self.len);
                let old_layout = std::alloc::Layout::array::<T>(self.capacity).unwrap();
                std::alloc::dealloc(self.ptr as *mut u8, old_layout);
            }
        }
        
        self.ptr = new_ptr;
        self.capacity = new_capacity;
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            for i in 0..self.len {
                unsafe {
                    std::ptr::drop_in_place(self.ptr.add(i));
                }
            }
            
            let layout = std::alloc::Layout::array::<T>(self.capacity).unwrap();
            unsafe {
                std::alloc::dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

impl<T> Index<usize> for Vec<T> {
    type Output = T;
    
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("Index out of bounds")
    }
}

impl<T> IndexMut<usize> for Vec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("Index out of bounds")
    }
}
```

### HashMap Implementation
```rust
pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    len: usize,
}

impl<K: Hash + Eq, V> HashMap<K, V> {
    pub fn new() -> Self {
        Self::with_capacity(16)
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        let mut buckets = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buckets.push(Vec::new());
        }
        
        HashMap { buckets, len: 0 }
    }
    
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.len >= self.buckets.len() * 3 / 4 {
            self.resize();
        }
        
        let hash = self.hash(&key);
        let bucket = &mut self.buckets[hash % self.buckets.len()];
        
        for (k, v) in bucket.iter_mut() {
            if k == &key {
                return Some(std::mem::replace(v, value));
            }
        }
        
        bucket.push((key, value));
        self.len += 1;
        None
    }
    
    pub fn get(&self, key: &K) -> Option<&V> {
        let hash = self.hash(key);
        let bucket = &self.buckets[hash % self.buckets.len()];
        
        for (k, v) in bucket {
            if k == key {
                return Some(v);
            }
        }
        
        None
    }
    
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let hash = self.hash(key);
        let bucket = &mut self.buckets[hash % self.buckets.len()];
        
        for (k, v) in bucket {
            if k == key {
                return Some(v);
            }
        }
        
        None
    }
    
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let hash = self.hash(key);
        let bucket = &mut self.buckets[hash % self.buckets.len()];
        
        for i in 0..bucket.len() {
            if &bucket[i].0 == key {
                let (_, v) = bucket.swap_remove(i);
                self.len -= 1;
                return Some(v);
            }
        }
        
        None
    }
    
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }
    
    pub fn len(&self) -> usize {
        self.len
    }
    
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    
    fn hash(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize
    }
    
    fn resize(&mut self) {
        let new_capacity = self.buckets.len() * 2;
        let mut new_buckets = Vec::with_capacity(new_capacity);
        for _ in 0..new_capacity {
            new_buckets.push(Vec::new());
        }
        
        for bucket in &mut self.buckets {
            for (key, value) in bucket.drain(..) {
                let hash = self.hash(&key);
                new_buckets[hash % new_capacity].push((key, value));
            }
        }
        
        self.buckets = new_buckets;
    }
}
```

### String Implementation
```rust
pub struct String {
    bytes: Vec<u8>,
}

impl String {
    pub fn new() -> Self {
        String { bytes: Vec::new() }
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        String {
            bytes: Vec::with_capacity(capacity),
        }
    }
    
    pub fn from(s: &str) -> Self {
        String {
            bytes: s.as_bytes().to_vec(),
        }
    }
    
    pub fn push(&mut self, ch: char) {
        let mut buf = [0; 4];
        let s = ch.encode_utf8(&mut buf);
        self.bytes.extend_from_slice(s.as_bytes());
    }
    
    pub fn push_str(&mut self, s: &str) {
        self.bytes.extend_from_slice(s.as_bytes());
    }
    
    pub fn pop(&mut self) -> Option<char> {
        if self.bytes.is_empty() {
            return None;
        }
        
        let ch = self.chars().last()?;
        let ch_len = ch.len_utf8();
        self.bytes.truncate(self.bytes.len() - ch_len);
        Some(ch)
    }
    
    pub fn len(&self) -> usize {
        self.bytes.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
    
    pub fn clear(&mut self) {
        self.bytes.clear();
    }
    
    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.bytes) }
    }
    
    pub fn chars(&self) -> std::str::Chars {
        self.as_str().chars()
    }
    
    pub fn split(&self, delimiter: char) -> Vec<String> {
        self.as_str()
            .split(delimiter)
            .map(String::from)
            .collect()
    }
    
    pub fn trim(&self) -> String {
        String::from(self.as_str().trim())
    }
    
    pub fn to_lowercase(&self) -> String {
        String::from(&self.as_str().to_lowercase())
    }
    
    pub fn to_uppercase(&self) -> String {
        String::from(&self.as_str().to_uppercase())
    }
}
```

## COMPLETE FILE TEMPLATES

### src/main.rs
```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use blaze_compiler::{compile, run, check};

#[derive(Parser)]
#[command(name = "blaze")]
#[command(about = "Blaze programming language compiler", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build {
        #[arg(value_name = "FILE")]
        source: PathBuf,
        
        #[arg(short, long)]
        release: bool,
        
        #[arg(short = 'O', long, default_value = "2")]
        opt_level: u8,
        
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        #[arg(long)]
        emit: Option<String>,
        
        #[arg(short, long)]
        verbose: bool,
    },
    Run {
        #[arg(value_name = "FILE")]
        source: PathBuf,
        
        #[arg(short, long)]
        release: bool,
        
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    Check {
        #[arg(value_name = "FILE")]
        source: PathBuf,
    },
    Test {
        #[arg(value_name = "TESTNAME")]
        test_name: Option<String>,
    },
    Init {
        #[arg(value_name = "NAME")]
        name: String,
    },
    Clean,
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Build {
            source,
            release,
            opt_level,
            output,
            emit,
            verbose,
        } => {
            let result = compile(&source, release, opt_level, output, emit, verbose);
            if let Err(e) = result {
                eprintln!("Compilation failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Run { source, release, args } => {
            let result = run(&source, release, args);
            if let Err(e) = result {
                eprintln!("Execution failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Check { source } => {
            let result = check(&source);
            if let Err(e) = result {
                eprintln!("Check failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Test { test_name } => {
            println!("Running tests...");
        }
        Commands::Init { name } => {
            println!("Creating new project: {}", name);
        }
        Commands::Clean => {
            println!("Cleaning build artifacts...");
        }
    }
}
```

### src/lib.rs
```rust
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod ir;
pub mod codegen;
pub mod error;
pub mod runtime;
pub mod stdlib;
pub mod utils;

use std::path::Path;
use anyhow::Result;

pub fn compile(
    source: &Path,
    release: bool,
    opt_level: u8,
    output: Option<PathBuf>,
    emit: Option<String>,
    verbose: bool,
) -> Result<()> {
    let source_code = std::fs::read_to_string(source)?;
    
    if verbose {
        println!("Lexing...");
    }
    let tokens = lexer::lex(&source_code)?;
    
    if verbose {
        println!("Parsing...");
    }
    let ast = parser::parse(tokens)?;
    
    if verbose {
        println!("Type checking...");
    }
    semantic::check(&ast)?;
    
    if verbose {
        println!("Generating IR...");
    }
    let ir = ir::generate(&ast)?;
    
    if release || opt_level > 0 {
        if verbose {
            println!("Optimizing...");
        }
        ir::optimize(&ir, opt_level)?;
    }
    
    if verbose {
        println!("Generating code...");
    }
    codegen::generate(&ir, output, emit)?;
    
    Ok(())
}

pub fn run(source: &Path, release: bool, args: Vec<String>) -> Result<()> {
    compile(source, release, if release { 2 } else { 0 }, None, None, false)?;
    
    Ok(())
}

pub fn check(source: &Path) -> Result<()> {
    let source_code = std::fs::read_to_string(source)?;
    let tokens = lexer::lex(&source_code)?;
    let ast = parser::parse(tokens)?;
    semantic::check(&ast)?;
    
    println!("✓ No errors found");
    Ok(())
}
```

## COMPREHENSIVE TEST SUITE

### tests/lexer_tests.rs
```rust
use blaze_compiler::lexer::{lex, TokenType};

#[test]
fn test_keywords() {
    let source = "let mut fn return if else while for";
    let tokens = lex(source).unwrap();
    
    assert_eq!(tokens[0].token_type, TokenType::Let);
    assert_eq!(tokens[1].token_type, TokenType::Mut);
    assert_eq!(tokens[2].token_type, TokenType::Fn);
    assert_eq!(tokens[3].token_type, TokenType::Return);
    assert_eq!(tokens[4].token_type, TokenType::If);
    assert_eq!(tokens[5].token_type, TokenType::Else);
    assert_eq!(tokens[6].token_type, TokenType::While);
    assert_eq!(tokens[7].token_type, TokenType::For);
}

#[test]
fn test_integers() {
    let source = "0 123 456789 0xFF 0o77 0b1010";
    let tokens = lex(source).unwrap();
    
    assert_eq!(tokens[0].token_type, TokenType::IntLiteral(0));
    assert_eq!(tokens[1].token_type, TokenType::IntLiteral(123));
    assert_eq!(tokens[2].token_type, TokenType::IntLiteral(456789));
    assert_eq!(tokens[3].token_type, TokenType::IntLiteral(255));
    assert_eq!(tokens[4].token_type, TokenType::IntLiteral(63));
    assert_eq!(tokens[5].token_type, TokenType::IntLiteral(10));
}

#[test]
fn test_floats() {
    let source = "3.14 2.71828 1e10 1.5e-5";
    let tokens = lex(source).unwrap();
    
    assert_eq!(tokens[0].token_type, TokenType::FloatLiteral(3.14));
    assert_eq!(tokens[1].token_type, TokenType::FloatLiteral(2.71828));
}

#[test]
fn test_strings() {
    let source = r#""hello" "world\n" "tab\there""#;
    let tokens = lex(source).unwrap();
    
    assert_eq!(
        tokens[0].token_type,
        TokenType::StringLiteral("hello".to_string())
    );
    assert_eq!(
        tokens[1].token_type,
        TokenType::StringLiteral("world\n".to_string())
    );
}

#[test]
fn test_operators() {
    let source = "+ - * / % = == != < <= > >= && || !";
    let tokens = lex(source).unwrap();
    
    assert_eq!(tokens[0].token_type, TokenType::Plus);
    assert_eq!(tokens[1].token_type, TokenType::Minus);
    assert_eq!(tokens[2].token_type, TokenType::Star);
    assert_eq!(tokens[3].token_type, TokenType::Slash);
    assert_eq!(tokens[4].token_type, TokenType::Percent);
    assert_eq!(tokens[5].token_type, TokenType::Equal);
    assert_eq!(tokens[6].token_type, TokenType::EqualEqual);
    assert_eq!(tokens[7].token_type, TokenType::BangEqual);
}

#[test]
fn test_complex_expression() {
    let source = "let x = (a + b) * c / d;";
    let tokens = lex(source).unwrap();
    
    assert!(tokens.len() > 0);
    assert_eq!(tokens[0].token_type, TokenType::Let);
}

#[test]
fn test_error_unterminated_string() {
    let source = r#""unterminated"#;
    let result = lex(source);
    
    assert!(result.is_err());
}

#[test]
fn test_unicode_identifiers() {
    let source = "let متغير = 10;";
    let tokens = lex(source).unwrap();
    
    assert_eq!(tokens[0].token_type, TokenType::Let);
}
```

### tests/parser_tests.rs
```rust
use blaze_compiler::parser::{parse, Expr, Statement};
use blaze_compiler::lexer::lex;

#[test]
fn test_parse_let_statement() {
    let source = "let x: i32 = 5;";
    let tokens = lex(source).unwrap();
    let ast = parse(tokens).unwrap();
    
    assert_eq!(ast.statements.len(), 1);
}

#[test]
fn test_parse_function() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }
    "#;
    let tokens = lex(source).unwrap();
    let ast = parse(tokens).unwrap();
    
    assert_eq!(ast.functions.len(), 1);
    assert_eq!(ast.functions[0].params.len(), 2);
}

#[test]
fn test_parse_if_expression() {
    let source = r#"
        if x > 0 {
            println("positive");
        } else {
            println("negative");
        }
    "#;
    let tokens = lex(source).unwrap();
    let ast = parse(tokens).unwrap();
    
    assert!(ast.statements.len() > 0);
}

#[test]
fn test_parse_struct() {
    let source = r#"
        struct Point {
            x: f64,
            y: f64,
        }
    "#;
    let tokens = lex(source).unwrap();
    let ast = parse(tokens).unwrap();
    
    assert_eq!(ast.structs.len(), 1);
    assert_eq!(ast.structs[0].fields.len(), 2);
}

#[test]
fn test_operator_precedence() {
    let source = "let result = 2 + 3 * 4;";
    let tokens = lex(source).unwrap();
    let ast = parse(tokens).unwrap();
    
    assert_eq!(ast.statements.len(), 1);
}

#[test]
fn test_error_recovery() {
    let source = "let x = ;";
    let tokens = lex(source).unwrap();
    let result = parse(tokens);
    
    assert!(result.is_err());
}
```

### tests/type_checker_tests.rs
```rust
use blaze_compiler::{lexer::lex, parser::parse, semantic::check};

#[test]
fn test_simple_type_check() {
    let source = r#"
        fn main() {
            let x: i32 = 5;
            let y: i32 = x + 10;
        }
    "#;
    let tokens = lex(source).unwrap();
    let ast = parse(tokens).unwrap();
    let result = check(&ast);
    
    assert!(result.is_ok());
}

#[test]
fn test_type_mismatch() {
    let source = r#"
        fn main() {
            let x: i32 = "hello";
        }
    "#;
    let tokens = lex(source).unwrap();
    let ast = parse(tokens).unwrap();
    let result = check(&ast);
    
    assert!(result.is_err());
}

#[test]
fn test_function_return_type() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }
    "#;
    let tokens = lex(source).unwrap();
    let ast = parse(tokens).unwrap();
    let result = check(&ast);
    
    assert!(result.is_ok());
}

#[test]
fn test_generic_function() {
    let source = r#"
        fn identity<T>(x: T) -> T {
            x
        }
    "#;
    let tokens = lex(source).unwrap();
    let ast = parse(tokens).unwrap();
    let result = check(&ast);
    
    assert!(result.is_ok());
}
```

### tests/borrow_checker_tests.rs
```rust
use blaze_compiler::{lexer::lex, parser::parse, semantic::check};

#[test]
fn test_simple_move() {
    let source = r#"
        fn main() {
            let s1 = String::from("hello");
            let s2 = s1;
        }
    "#;
    let tokens = lex(source).unwrap();
    let ast = parse(tokens).unwrap();
    let result = check(&ast);
    
    assert!(result.is_ok());
}

#[test]
fn test_use_after_move() {
    let source = r#"
        fn main() {
            let s1 = String::from("hello");
            let s2
    