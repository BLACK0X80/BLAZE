# BLAZE Compiler Architecture

This document describes the internal architecture of the BLAZE compiler, explaining how source code is transformed into executable binaries.

## Table of Contents

1. [Overview](#overview)
2. [Compilation Pipeline](#compilation-pipeline)
3. [Lexical Analysis](#lexical-analysis)
4. [Syntax Analysis](#syntax-analysis)
5. [Semantic Analysis](#semantic-analysis)
6. [Intermediate Representation](#intermediate-representation)
7. [Code Generation](#code-generation)
8. [Runtime System](#runtime-system)
9. [Design Decisions](#design-decisions)

## Overview

The BLAZE compiler is a multi-pass compiler that transforms source code through several intermediate representations before generating machine code. The architecture follows traditional compiler design principles with modern enhancements for memory safety.

### High-Level Architecture

```
┌─────────────┐
│ Source Code │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Lexer     │ ──► Tokens
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Parser    │ ──► AST
└──────┬──────┘
       │
       ▼
┌─────────────────────┐
│ Semantic Analyzer   │
│  ├─ Type Checker    │
│  ├─ Borrow Checker  │
│  └─ Lifetime Analyzer│
└──────┬──────────────┘
       │
       ▼
┌─────────────┐
│ IR Generator│ ──► IR Module (SSA)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ IR Optimizer│ ──► Optimized IR
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ LLVM Backend│ ──► Object File
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Linker    │ ──► Executable
└─────────────┘
```

## Compilation Pipeline

### Phase 1: Lexical Analysis

**Input**: Source code (text)  
**Output**: Token stream  
**Module**: `src/lexer/`

The lexer (scanner) reads the source code character by character and groups them into tokens. It handles:

- Keywords (`fn`, `let`, `if`, etc.)
- Identifiers (variable and function names)
- Literals (numbers, strings, booleans)
- Operators (`+`, `-`, `*`, `/`, etc.)
- Punctuation (`;`, `,`, `{`, `}`, etc.)
- Comments (single-line and multi-line)

**Key Components**:
- `Scanner`: Character-by-character processing
- `Token`: Token representation with type and value
- Error reporting for invalid characters

### Phase 2: Syntax Analysis

**Input**: Token stream  
**Output**: Abstract Syntax Tree (AST)  
**Module**: `src/parser/`

The parser consumes tokens and builds an AST representing the program structure. It uses recursive descent parsing with operator precedence climbing for expressions.

**Key Components**:
- `Parser`: Recursive descent parser
- `AST`: Tree representation of program structure
  - `Program`: Top-level node
  - `Function`: Function declarations
  - `Statement`: Statements (let, return, expression, etc.)
  - `Expression`: Expressions (binary ops, calls, literals, etc.)

**Grammar**:
```
Program     → Function*
Function    → 'fn' Identifier '(' Parameters ')' ('->' Type)? Block
Block       → '{' Statement* '}'
Statement   → LetStmt | ReturnStmt | ExprStmt | IfStmt | WhileStmt
Expression  → Primary | BinaryOp | UnaryOp | Call | FieldAccess
```

### Phase 3: Semantic Analysis

**Input**: AST  
**Output**: Validated AST with type information  
**Module**: `src/semantic/`

Semantic analysis ensures the program is semantically correct. It consists of several sub-phases:

#### 3.1 Symbol Resolution

**Module**: `src/semantic/symbol_table.rs`

Builds symbol tables for all scopes and resolves identifiers to their declarations.

- Tracks variables, functions, and types
- Handles nested scopes
- Detects undefined variables and duplicate definitions

#### 3.2 Scope Resolution

**Module**: `src/semantic/scope_resolver.rs`

Manages lexical scopes and variable visibility.

- Tracks scope hierarchy
- Resolves variable references
- Handles shadowing

#### 3.3 Type Checking

**Module**: `src/semantic/type_checker.rs`

Validates type correctness and performs type inference.

- Checks type compatibility in expressions
- Validates function call arguments
- Infers types for variables without explicit annotations
- Detects type mismatches

**Type System**:
- Primitive types: `i8`, `i16`, `i32`, `i64`, `i128`, `f32`, `f64`, `bool`, `String`
- Compound types: Arrays, structs
- Reference types: `&T`, `&mut T`

#### 3.4 Borrow Checking

**Module**: `src/semantic/borrow_checker.rs`

Enforces memory safety rules inspired by Rust's ownership system.

**Ownership Rules**:
1. Each value has a single owner
2. Values can be moved or borrowed
3. Borrows must not outlive the owner

**Borrow Rules**:
1. Either one mutable borrow OR any number of immutable borrows
2. No simultaneous mutable and immutable borrows
3. Borrows must be valid for their entire lifetime

**Implementation**:
- Builds Control Flow Graph (CFG) using `petgraph`
- Tracks loans (borrows) with unique IDs
- Performs dataflow analysis to propagate loan information
- Detects conflicts between loans

#### 3.5 Lifetime Analysis

**Module**: `src/semantic/lifetime_analyzer.rs`

Infers and validates reference lifetimes.

- Infers lifetimes for references
- Validates lifetime constraints
- Ensures references don't outlive their referents

### Phase 4: IR Generation

**Input**: Validated AST  
**Output**: IR Module in SSA form  
**Module**: `src/ir/`

The IR generator transforms the AST into an intermediate representation suitable for optimization and code generation.

**IR Structure**:
```
Module
  ├─ Functions
  │   ├─ Parameters
  │   ├─ Basic Blocks
  │   │   ├─ Instructions
  │   │   └─ Terminator
  │   └─ Return Type
  ├─ Global Variables
  └─ Type Definitions
```

**Key Features**:
- **SSA Form**: Each variable is assigned exactly once
- **Phi Nodes**: Merge values from different control flow paths
- **Basic Blocks**: Sequences of instructions with single entry and exit
- **Terminators**: Control flow instructions (branch, return)

**Instructions**:
- Arithmetic: `Add`, `Sub`, `Mul`, `Div`, `Mod`
- Comparison: `Eq`, `Ne`, `Lt`, `Le`, `Gt`, `Ge`
- Logical: `And`, `Or`, `Not`
- Memory: `Load`, `Store`, `Alloca`
- Control: `Call`, `Br`, `CondBr`, `Ret`

### Phase 5: IR Optimization

**Input**: IR Module  
**Output**: Optimized IR Module  
**Module**: `src/ir/optimization.rs`

Applies optimization passes to improve code quality.

**Optimization Passes**:
1. **Dead Code Elimination**: Removes unused code
2. **Constant Folding**: Evaluates constant expressions at compile time
3. **Constant Propagation**: Replaces variables with known constant values
4. **Copy Propagation**: Eliminates unnecessary copies
5. **Common Subexpression Elimination**: Reuses computed values

**Example**:
```rust
// Before optimization
let x = 2 + 3;
let y = 2 + 3;
let z = x + y;

// After optimization
let x = 5;
let y = 5;
let z = 10;
```

### Phase 6: Code Generation

**Input**: Optimized IR Module  
**Output**: Object file  
**Module**: `src/codegen/`

Code generation transforms IR into machine code using LLVM.

#### 6.1 Register Allocation

**Module**: `src/codegen/register_allocator.rs`

Assigns virtual registers to physical registers or stack slots.

- Uses graph coloring algorithm
- Handles register spilling when needed
- Optimizes register usage

#### 6.2 LLVM Backend

**Module**: `src/codegen/llvm_backend.rs`

Generates LLVM IR and compiles to object files.

**Process**:
1. Create LLVM context and module
2. Generate LLVM IR from BLAZE IR
3. Apply LLVM optimization passes
4. Emit object file for target platform

**LLVM Integration**:
- Uses `inkwell` crate for LLVM bindings
- Supports optimization levels 0-3
- Generates platform-specific code

#### 6.3 Linking

**Module**: `src/codegen/linker.rs`

Links object files with runtime libraries to create executables.

**Platform-Specific Linking**:
- **Windows**: Uses MSVC linker (`link.exe`)
- **Linux**: Uses GCC or Clang linker
- **macOS**: Uses Clang linker

**Runtime Libraries**:
- `libblaze_runtime`: BLAZE runtime support
- `libc`: Standard C library
- `libm`: Math library

### Phase 7: Runtime System

**Module**: `src/runtime/`

Provides runtime support for compiled programs.

#### 7.1 Memory Allocator

**Module**: `src/runtime/allocator.rs`

Efficient memory allocation with leak detection.

**Features**:
- Fast allocation and deallocation
- Allocation tracking for debugging
- Leak detection in debug builds
- Thread-safe operations

**API**:
```c
void* blaze_alloc(size_t size, size_t align);
void blaze_dealloc(void* ptr, size_t size, size_t align);
void* blaze_realloc(void* ptr, size_t old_size, size_t new_size, size_t align);
```

#### 7.2 Intrinsics

**Module**: `src/runtime/intrinsics.rs`

Built-in functions for common operations.

**Mathematical Functions**:
- `sqrt`, `pow`, `sin`, `cos`, `tan`
- `exp`, `log`, `abs`

**String Functions**:
- `strlen`, `strcmp`, `strcpy`

**Memory Functions**:
- `memcpy`, `memset`, `memmove`

#### 7.3 Panic Handling

**Module**: `src/runtime/panic.rs`

Handles runtime errors and panics.

**Features**:
- Prints error message with location
- Unwinds stack (basic implementation)
- Exits with error code

## Design Decisions

### Why Multi-Pass?

The compiler uses multiple passes for several reasons:

1. **Separation of Concerns**: Each phase has a clear responsibility
2. **Easier Testing**: Each phase can be tested independently
3. **Better Error Messages**: Errors are caught early with context
4. **Optimization Opportunities**: Multiple passes enable more optimizations

### Why SSA Form?

Static Single Assignment (SSA) form simplifies optimization:

1. **Dataflow Analysis**: Easier to track value flow
2. **Optimization**: Many optimizations are simpler in SSA
3. **Register Allocation**: Simplifies live range analysis

### Why LLVM?

LLVM provides several advantages:

1. **Production Quality**: Battle-tested code generation
2. **Optimization**: State-of-the-art optimization passes
3. **Platform Support**: Supports many target platforms
4. **Tooling**: Excellent debugging and profiling tools

### Why Rust?

The compiler is written in Rust for:

1. **Memory Safety**: Prevents bugs in the compiler itself
2. **Performance**: Comparable to C/C++
3. **Ecosystem**: Rich ecosystem of libraries
4. **Tooling**: Excellent build system and package manager

## Module Organization

```
src/
├── lexer/              # Lexical analysis
│   ├── mod.rs          # Lexer entry point
│   ├── scanner.rs      # Character scanning
│   └── token.rs        # Token definitions
│
├── parser/             # Syntax analysis
│   ├── mod.rs          # Parser entry point
│   └── ast.rs          # AST definitions
│
├── semantic/           # Semantic analysis
│   ├── mod.rs          # Semantic analyzer entry point
│   ├── type_checker.rs # Type checking
│   ├── borrow_checker.rs # Borrow checking
│   ├── lifetime_analyzer.rs # Lifetime analysis
│   ├── symbol_table.rs # Symbol table
│   └── scope_resolver.rs # Scope resolution
│
├── ir/                 # Intermediate representation
│   ├── mod.rs          # IR entry point
│   ├── builder.rs      # IR builder
│   ├── instruction.rs  # IR instructions
│   ├── optimization.rs # Optimization passes
│   └── validation.rs   # IR validation
│
├── codegen/            # Code generation
│   ├── mod.rs          # Codegen entry point
│   ├── llvm_backend.rs # LLVM integration
│   ├── register_allocator.rs # Register allocation
│   └── linker.rs       # Linking
│
├── runtime/            # Runtime support
│   ├── mod.rs          # Runtime entry point
│   ├── allocator.rs    # Memory allocator
│   ├── intrinsics.rs   # Built-in functions
│   └── panic.rs        # Panic handling
│
├── stdlib/             # Standard library
│   ├── mod.rs          # Stdlib entry point
│   └── collections/    # Collections (Vec, HashMap)
│
├── error/              # Error handling
│   ├── mod.rs          # Error types
│   ├── diagnostics.rs  # Diagnostic system
│   └── suggestions.rs  # Error suggestions
│
├── utils/              # Utilities
│   └── source_map.rs   # Source location tracking
│
├── lib.rs              # Library entry point
└── main.rs             # CLI entry point
```

## Performance Considerations

### Compilation Speed

- **Incremental Compilation**: Planned for future versions
- **Parallel Compilation**: Functions can be compiled in parallel (future)
- **Caching**: Symbol tables and type information are cached

### Runtime Performance

- **Optimization Levels**: Support for -O0 through -O3
- **LLVM Optimizations**: Leverage LLVM's optimization passes
- **Efficient Allocator**: Fast memory allocation with minimal overhead

### Memory Usage

- **Arena Allocation**: AST and IR use arena allocation (future)
- **Streaming**: Large files are processed in chunks (future)
- **Cleanup**: Intermediate data structures are freed after use

## Testing Strategy

### Unit Tests

Each module has comprehensive unit tests:

```bash
cargo test --lib
```

### Integration Tests

End-to-end tests in `tests/`:

```bash
cargo test --test end_to_end_tests
cargo test --test error_handling_tests
```

### Performance Tests

Benchmarks in `benches/`:

```bash
cargo bench
```

### Coverage

Measure test coverage:

```bash
./scripts/coverage.sh  # Linux/macOS
.\scripts\coverage.ps1  # Windows
```

## Future Enhancements

### Short-term

- [ ] Generic types and traits
- [ ] Pattern matching
- [ ] Closures and lambdas
- [ ] Module system

### Medium-term

- [ ] Incremental compilation
- [ ] Language Server Protocol (LSP)
- [ ] Debugger integration (DWARF)
- [ ] Package manager

### Long-term

- [ ] Async/await support
- [ ] Parallel compilation
- [ ] Cross-compilation
- [ ] JIT compilation mode
- [ ] WebAssembly backend

## Contributing

To contribute to the compiler:

1. **Understand the Architecture**: Read this document thoroughly
2. **Pick a Module**: Choose a module to work on
3. **Write Tests**: Add tests before implementing features
4. **Follow Style**: Use `cargo fmt` and `cargo clippy`
5. **Document**: Add rustdoc comments to public APIs

See the main README for contribution guidelines.

## Resources

### Books

- "Engineering a Compiler" by Cooper & Torczon
- "Modern Compiler Implementation in ML" by Appel
- "Compilers: Principles, Techniques, and Tools" (Dragon Book)

### Papers

- "Simple and Effective Type Check Removal through Lazy Basic Block Versioning"
- "A Graph-Free Approach to Data-Flow Analysis"
- "Linear Scan Register Allocation"

### Online Resources

- [LLVM Documentation](https://llvm.org/docs/)
- [Rust Compiler Development Guide](https://rustc-dev-guide.rust-lang.org/)
- [Crafting Interpreters](https://craftinginterpreters.com/)

## Glossary

- **AST**: Abstract Syntax Tree - tree representation of source code structure
- **CFG**: Control Flow Graph - graph representing program control flow
- **IR**: Intermediate Representation - internal code representation
- **SSA**: Static Single Assignment - form where each variable is assigned once
- **Phi Node**: Instruction that merges values from different control flow paths
- **Basic Block**: Sequence of instructions with single entry and exit
- **Terminator**: Instruction that ends a basic block (branch, return)
- **Loan**: A borrow in the borrow checker
- **Lifetime**: Duration for which a reference is valid
