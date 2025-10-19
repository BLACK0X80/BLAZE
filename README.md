<div align="center">

<img src="https://capsule-render.vercel.app/api?type=waving&color=gradient&customColorList=6,11,20&height=200&section=header&text=BLAZE&fontSize=90&fontColor=fff&animation=twinkling&fontAlignY=35" width="100%"/>

<br/>

<p align="center">
  <img src="https://readme-typing-svg.demolab.com?font=Fira+Code&weight=700&size=32&duration=3000&pause=1000&color=FF6B35&center=true&vCenter=true&multiline=true&width=800&height=120&lines=COMPILER+LEARNING+PROJECT;Educational+%7C+Rust-Based+%7C+Work+in+Progress" alt="Typing Animation" />
</p>

<br/>

[![BLAZE](https://img.shields.io/badge/BLAZE-v0.1.0-FF6B35?style=for-the-badge&logo=fire&logoColor=white)](https://github.com/BLACK0X80/blaze)
[![License](https://img.shields.io/badge/License-MIT-8B5CF6?style=for-the-badge&logo=balance-scale&logoColor=white)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/Status-Educational-10F5CC?style=for-the-badge&logo=book&logoColor=black)](https://github.com/BLACK0X80/blaze)

<br/>

### **Built with Rust • Learning Compiler Design • Frontend Implementation**

> ⚠️ **Educational Project**: This is a learning project exploring compiler construction. Only the lexer and parser are functional. Not intended for production use.

**BLAZE** is an educational compiler implementation that explores how programming languages work under the hood, with a focus on Rust-like syntax and modern compiler design principles.

[**Quick Start**](#quick-start) • [**Examples**](#examples) • [**Roadmap**](#roadmap)

</div>

---

## Table of Contents

<table>
<tr>
<td valign="top" width="33%">

### About
- [Project Goals](#project-goals)
- [Current Status](#current-status)
- [What Works](#what-works)
- [Installation](#installation)

</td>
<td valign="top" width="33%">

### Learn
- [Quick Start](#quick-start)
- [Language Syntax](#language-syntax)
- [Examples](#examples)
- [Architecture](#architecture)

</td>
<td valign="top" width="33%">

### Develop
- [Project Structure](#project-structure)
- [Testing](#testing)
- [Roadmap](#roadmap)
- [Contributing](#contributing)

</td>
</tr>
</table>

---

## Project Goals

<div align="center">

### **Learning Compiler Construction Through Practice**

</div>

This project aims to understand and implement the core concepts of compiler design by building a functional compiler frontend for a Rust-like language.

<table>
<tr>
<td align="center" width="33%">

### 📚 **Educational**

Learning by doing
<br/>
Exploring compiler theory
<br/>
Understanding language design
<br/>
**Hands-on experience**

</td>
<td align="center" width="33%">

### 🔧 **Practical**

Working lexer
<br/>
Functional parser
<br/>
AST generation
<br/>
**Real implementation**

</td>
<td align="center" width="33%">

### 🚀 **Progressive**

Step-by-step development
<br/>
Incremental features
<br/>
Clear documentation
<br/>
**Continuous learning**

</td>
</tr>
</table>

---

## Current Status

<div align="center">

### Implementation Progress

</div>

<table>
<tr>
<td width="50%" valign="top">

### ✅ Implemented

```rust
✓ Lexer (Tokenization)
  - Keywords and identifiers
  - Numeric literals (int, float)
  - String and char literals
  - Operators and punctuation
  - Comments (single/multi-line)
  - Basic error reporting

✓ Parser (AST Generation)
  - Function declarations
  - Variable declarations (let, let mut)
  - Basic expressions
  - Control flow (if/else, while)
  - Struct definitions
  - Operator precedence

✓ CLI Tool
  - blaze check (syntax validation)
  - blaze build (AST generation)
  - Error reporting
```

</td>
<td width="50%" valign="top">

### 🚧 In Progress / Not Working

```rust
⏳ Type System (Partial)
  - Basic structure exists
  - Type checking incomplete
  - No type inference yet
  
⏳ Semantic Analysis (Stub)
  - Symbol table framework
  - Scope resolution incomplete
  
⏳ IR Generation (Stub)
  - Module structure only
  - No actual IR generation
  
⏳ Code Generation (Not Started)
  - Placeholder files only
  - No LLVM integration
  
⏳ Standard Library (Minimal)
  - Basic stubs only
  - Not functional
```

</td>
</tr>
</table>

---

## What Works

<div align="center">

### Functional Components

| Component | Status | Description |
|:---------:|:------:|:------------|
| **Lexer** | ✅ Working | Tokenizes source code into tokens |
| **Parser** | ✅ Working | Generates Abstract Syntax Tree |
| **Error Reporting** | ✅ Basic | Shows syntax errors with line/column |
| **CLI Interface** | ✅ Working | Commands for checking and building |

### Test Results

| Test Suite | Tests | Status |
|:----------:|:-----:|:------:|
| Lexer Tests | 6 | ✅ Pass |
| Parser Tests | 5 | ✅ Pass |
| Integration Tests | 4 | ✅ Pass |
| **Total** | **15** | **✅ All Pass** |

</div>

---

## Installation

<div align="center">

### Prerequisites

| Requirement | Version |
|:-----------:|:-------:|
| Rust & Cargo | 1.70.0+ |
| Git | Any recent version |

</div>

<br/>

### Setup Instructions

```bash
# Clone the repository
git clone https://github.com/yourusername/blaze.git
cd blaze

# Build the project
cargo build --release

# Run tests to verify
cargo test

# Try the compiler
cargo run -- check examples/hello.blz
```

---

## Quick Start

<div align="center">

### Your First BLAZE Program

</div>

```bash
# Create a simple program
echo 'fn main() {
    let x: i32 = 5;
    let y: i32 = 10;
    let sum = x + y;
}' > hello.blz

# Check syntax (this works!)
cargo run -- check hello.blz

# View the AST
cargo run -- build hello.blz
```

<br/>

### Available Commands

<div align="center">

| Command | What It Does | Status |
|:-------:|:-------------|:------:|
| `check` | Validates syntax, reports errors | ✅ Works |
| `build` | Parses and shows AST | ✅ Works |
| `run` | Execute program | ❌ Not implemented |

</div>

---

## Language Syntax

### What's Currently Parseable

```rust
// ✅ Variables (parsing works)
let x: i32 = 42;
let mut counter = 0;
let name = "BLAZE";

// ✅ Functions (parsing works)
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

// ✅ Structs (parsing works)
struct Point {
    x: i32,
    y: i32,
}

// ✅ Control Flow (parsing works)
if x > 0 {
    let y = 1;
} else {
    let y = 2;
}

while x < 10 {
    x = x + 1;
}

// ✅ Expressions (parsing works)
let result = (a + b) * c;
let is_valid = x > 0 && y < 10;
```

### Current Limitations

```rust
// ❌ These don't work yet:

// No actual type checking
let x: i32 = "string";  // Parser accepts but doesn't catch error

// No code generation
// (Programs parse but don't compile to executables)

// No borrow checking
// (Ownership rules not enforced)

// Limited error messages
// (Basic syntax errors only)
```

---

## Examples

### Fibonacci (Syntax Only)

```rust
// This parses correctly but doesn't execute
fn fibonacci(n: i32) -> i32 {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fn main() {
    let result = fibonacci(10);
}
```

### Point Structure (Syntax Only)

```rust
// Parser generates correct AST for this
struct Point {
    x: f64,
    y: f64,
}

fn main() {
    let p = Point { x: 3.0, y: 4.0 };
}
```

---

## Architecture

<div align="center">

### **Compiler Pipeline Design**

*Current Implementation Status*

</div>

```mermaid
graph LR
    A[Source Code] --> B[Lexer ✅]
    B --> C[Parser ✅]
    C --> D[Type Checker 🚧]
    D --> E[Borrow Checker ⏳]
    E --> F[IR Generator ⏳]
    F --> G[Code Generation ❌]
    G --> H[Executable ❌]

    style A fill:#FF6B35,stroke:#333,stroke-width:3px,color:#fff
    style B fill:#32CD32,stroke:#333,stroke-width:3px,color:#fff
    style C fill:#32CD32,stroke:#333,stroke-width:3px,color:#fff
    style D fill:#FFD700,stroke:#333,stroke-width:3px,color:#000
    style E fill:#FFA500,stroke:#333,stroke-width:3px,color:#000
    style F fill:#FFA500,stroke:#333,stroke-width:3px,color:#000
    style G fill:#DC143C,stroke:#333,stroke-width:3px,color:#fff
    style H fill:#DC143C,stroke:#333,stroke-width:3px,color:#fff
```

<br/>

### Pipeline Implementation Status

<div align="center">

| Stage | Status | Implementation | Notes |
|:-----:|:------:|:-------------:|:------|
| **Lexer** | ✅ Complete | ~250 lines | Tokenization fully working |
| **Parser** | ✅ Complete | ~400 lines | AST generation functional |
| **Type Checker** | 🚧 Partial | ~200 lines | Structure exists, incomplete logic |
| **Borrow Checker** | ⏳ Stub | ~50 lines | Framework only, no checking |
| **IR Generation** | ⏳ Stub | ~30 lines | Module structure, no implementation |
| **Code Generation** | ❌ Not Started | ~10 lines | Placeholder files only |
| **Optimizer** | ❌ Not Started | ~0 lines | Not implemented |

</div>

<br/>

### What Each Stage Does (Planned Architecture)

<table>
<tr>
<td width="50%" valign="top">

### ✅ **Working Stages**

**1. Lexer (Tokenization)**
- Reads source code character by character
- Converts to tokens (keywords, operators, literals)
- Tracks line/column for error reporting
- **Status**: Fully functional

**2. Parser (AST Generation)**
- Consumes tokens from lexer
- Builds Abstract Syntax Tree
- Validates syntax rules
- **Status**: Fully functional

</td>
<td width="50%" valign="top">

### 🚧 **Planned Stages**

**3. Type Checker** *(Partial)*
- Validates type correctness
- Type inference
- **Status**: Structure exists, needs work

**4. Borrow Checker** *(Stub)*
- Memory safety validation
- **Status**: Framework only

**5. IR Generation** *(Stub)*
- Intermediate representation
- **Status**: Placeholder

**6. Code Generation** *(Not Started)*
- Machine code output
- **Status**: Not implemented

</td>
</tr>
</table>

---

## Project Structure

<div align="center">

### Codebase Organization

</div>

```
blaze/
├── src/
│   ├── lexer/              ✅ ~250 lines (Working)
│   │   ├── mod.rs          Entry point
│   │   ├── scanner.rs      Tokenization logic
│   │   └── token.rs        Token definitions
│   │
│   ├── parser/             ✅ ~400 lines (Working)
│   │   ├── mod.rs          Parser implementation
│   │   └── ast.rs          AST node definitions
│   │
│   ├── error/              ✅ ~80 lines (Basic)
│   │   └── mod.rs          Error types & reporting
│   │
│   ├── semantic/           🚧 ~200 lines (Partial)
│   │   └── mod.rs          Type checker (incomplete)
│   │
│   ├── ir/                 ⏳ ~30 lines (Stub)
│   │   └── mod.rs          IR generation (placeholder)
│   │
│   ├── codegen/            ❌ ~10 lines (Stub)
│   │   └── mod.rs          Code generation (not started)
│   │
│   └── main.rs             ✅ ~100 lines (CLI working)
│
├── examples/               ✅ 7 example files
│   ├── hello.blz           Basic hello world
│   ├── fibonacci.blz       Recursive function
│   ├── struct_example.blz  Struct definition
│   └── ...
│
├── tests/                  ✅ 15 tests passing
│   ├── lexer_tests.rs      6 tests
│   ├── parser_tests.rs     5 tests
│   └── integration_tests.rs 4 tests
│
├── Cargo.toml              Project configuration
├── README.md               This file
└── LICENSE                 MIT License

Total: ~1,300 lines of actual code
       (not counting stubs and placeholders)
```

---

## Testing

<div align="center">

### Current Test Coverage

</div>

```bash
# Run all tests
cargo test

Test Results:
✅ lexer_tests::test_keywords .......... passed
✅ lexer_tests::test_numbers ........... passed
✅ lexer_tests::test_strings ........... passed
✅ lexer_tests::test_operators ......... passed
✅ lexer_tests::test_identifiers ....... passed
✅ lexer_tests::test_comments .......... passed

✅ parser_tests::test_parse_let ........ passed
✅ parser_tests::test_parse_function ... passed
✅ parser_tests::test_parse_arithmetic . passed
✅ parser_tests::test_parse_if ......... passed
✅ parser_tests::test_parse_while ...... passed

✅ integration::test_hello_program ..... passed
✅ integration::test_function_params ... passed
✅ integration::test_struct_definition . passed
✅ integration::test_complex_program ... passed

Total: 15/15 tests passing ✅
```

### What's Tested

- ✅ Lexer tokenization of all language constructs
- ✅ Parser AST generation for valid syntax
- ✅ Basic error reporting for syntax errors
- ✅ Integration: Full lexer → parser pipeline

### What's NOT Tested

- ❌ Type checking (not fully implemented)
- ❌ Semantic analysis (stub only)
- ❌ Code generation (not started)
- ❌ Runtime behavior (no execution capability)

---

## Roadmap

<div align="center">

### Development Phases

</div>

### ✅ Phase 1: Frontend (Current - Completed)
- [x] Basic lexer implementation (~250 lines)
- [x] Recursive descent parser (~400 lines)
- [x] AST representation
- [x] Error reporting structure (~80 lines)
- [x] CLI interface (~100 lines)
- [x] Basic test suite (15 tests)

**Time Invested**: ~3 weeks of learning and development

### 🚧 Phase 2: Type System (Next - In Progress)
- [ ] Complete type checker implementation
- [ ] Type inference engine
- [ ] Basic generic type support
- [ ] Comprehensive type tests

**Estimated Time**: 4-6 weeks

### ⏳ Phase 3: Semantic Analysis (Future)
- [ ] Full symbol table implementation
- [ ] Scope resolution
- [ ] Name binding
- [ ] Semantic validation

**Estimated Time**: 3-4 weeks

### ⏳ Phase 4: Middle-end (Future)
- [ ] IR generation
- [ ] Basic optimizations
- [ ] Control flow analysis

**Estimated Time**: 6-8 weeks

### ⏳ Phase 5: Backend (Long-term Goal)
- [ ] LLVM integration
- [ ] Code generation
- [ ] Executable output

**Estimated Time**: 8-12 weeks

---

## Known Limitations

<div align="center">

### Current Constraints & Known Issues

</div>

1. **❌ No Code Generation**: Compiler only validates syntax, cannot produce executables
2. **❌ Incomplete Type Checking**: Type errors often not caught (structure exists but logic incomplete)
3. **❌ No Execution**: Cannot run compiled programs
4. **⚠️ Limited Error Messages**: Only basic syntax errors with line/column numbers
5. **⚠️ No Standard Library**: Stdlib files are empty stubs for future implementation
6. **⚠️ No Optimization**: No optimization passes implemented
7. **⚠️ Single-threaded**: No parallel compilation support
8. **⚠️ Memory Leaks Possible**: Parser doesn't handle all edge cases properly
9. **⚠️ Limited Unicode**: Basic UTF-8 support only

---

## Learning Resources

<div align="center">

### Built While Learning From

</div>

This project was developed while studying:

<table>
<tr>
<td width="50%">

**Books & Courses**
- "Crafting Interpreters" by Robert Nystrom
- "Engineering a Compiler" (2nd Edition)
- "The Rust Programming Language" Book
- Stanford CS143 (Compilers)

</td>
<td width="50%">

**Online Resources**
- LLVM Tutorial
- Rust compiler source code
- Various compiler blog posts
- PL research papers

</td>
</tr>
</table>

### Recommended Reading Order

1. [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust basics first
2. [Crafting Interpreters](https://craftinginterpreters.com/) - Best intro to compilers
3. [LLVM Tutorial](https://llvm.org/docs/tutorial/) - For backend (future phase)

---

## Contributing

<div align="center">

### Help Make This Better for Learning!

</div>

This is primarily a learning project, but contributions that enhance the educational value are welcome!

### How to Contribute

**1. Educational Improvements** ⭐ Most Welcome!
   - Add code comments explaining concepts
   - Write tutorials or guides
   - Create example programs
   - Improve documentation

**2. Bug Fixes** 🐛
   - Fix parsing bugs
   - Improve error messages
   - Add test cases

**3. Feature Suggestions** 💡
   - Ideas for next learning phases
   - Architecture improvements
   - Best practices feedback

### Contribution Process

```bash
# Fork and clone
git clone https://github.com/yourusername/blaze.git
cd blaze

# Create branch
git checkout -b educational/better-comments

# Make changes
# ... add comments, examples, docs ...

# Test
cargo test
cargo fmt
cargo clippy

# Submit PR with clear description
git push origin educational/better-comments
```

---

## FAQ

<div align="center">

### Frequently Asked Questions

</div>

**Q: Can I use this in production?**
<br/>
A: **No.** This is an educational project. It only parses code; it doesn't compile or execute anything.

**Q: Will it ever be production-ready?**
<br/>
A: This is primarily a learning tool. The goal is understanding compilers, not building a production language. However, it could evolve over time.

**Q: What actually works right now?**
<br/>
A: The lexer and parser work completely. You can check syntax and view the AST. That's it. No type checking, no execution.

**Q: How can I learn from this?**
<br/>
A: Read the code (it's ~1,300 lines), run the examples, modify the parser, add tests, and experiment! Start with `src/lexer/` then `src/parser/`.

**Q: Why are there so many stub files?**
<br/>
A: They represent planned future work and help visualize the complete compiler architecture, even though they're not implemented yet.

**Q: Can I help develop this?**
<br/>
A: Yes! Especially with documentation, examples, and educational improvements. See [Contributing](#contributing).

**Q: How long did this take to build?**
<br/>
A: About 3 weeks of part-time learning and coding for the working parts (lexer, parser, CLI).

---

## License

<div align="center">

MIT License - See [LICENSE](LICENSE) for details

**Free to use for learning and educational purposes**

</div>

---

## Acknowledgments

<div align="center">

### Built While Learning From

| Resource | What I Learned |
|:--------:|:--------------|
| **Rust Language** | Systems programming concepts and safe memory management |
| **Crafting Interpreters** | Parser implementation patterns and AST design |
| **LLVM Project** | Compiler backend architecture (for future phases) |
| **Compiler Books** | Theory and algorithms |
| **Open Source Compilers** | Real-world implementation patterns |

</div>

### Special Thanks

- **Robert Nystrom** for "Crafting Interpreters" - the best compiler book for beginners
- **The Rust Team** for creating an amazing language to learn systems programming
- **LLVM Developers** for the architecture inspiration
- **Stack Overflow & Reddit** for answering countless questions
- **Everyone who writes compiler tutorials** - you made this possible!

---

<div align="center">

## Try It Out & Learn!

```bash
# Clone the repo
git clone https://github.com/yourusername/blaze.git
cd blaze

# Build and test
cargo build
cargo test

# Try parsing an example
cargo run -- check examples/fibonacci.blz

# View the AST output
cargo run -- build examples/hello.blz
```

<br/>

[![View Code](https://img.shields.io/badge/View%20Code-FF6B35?style=for-the-badge&logo=github&logoColor=white)](https://github.com/yourusername/blaze)
[![Read Examples](https://img.shields.io/badge/Examples-10F5CC?style=for-the-badge&logo=book&logoColor=black)](https://github.com/yourusername/blaze/tree/main/examples)
[![Run Tests](https://img.shields.io/badge/Tests-32CD32?style=for-the-badge&logo=check&logoColor=white)](https://github.com/yourusername/blaze)

<br/><br/>

### An Educational Journey into Compiler Construction

*Learning by building • Understanding by doing • Growing through practice*

**~ 1,300 lines of educational Rust code**
<br/>
**15 tests • 7 examples • Full frontend implementation**

**Built with curiosity by BLACK • 2025**

<br/>

<img src="https://capsule-render.vercel.app/api?type=waving&color=gradient&customColorList=6,11,20&height=120&section=footer" width="100%"/>

</div>
