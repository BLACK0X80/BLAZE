# BLAZE Quick Start Guide

Get started with BLAZE in 5 minutes.

## Prerequisites

- Rust 1.70+
- LLVM 15
- C/C++ compiler

## Installation

### 1. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Install LLVM 15

**Ubuntu:**
```bash
wget https://apt.llvm.org/llvm.sh
chmod +x llvm.sh
sudo ./llvm.sh 15
export LLVM_SYS_150_PREFIX=/usr/lib/llvm-15
```

**macOS:**
```bash
brew install llvm@15
export LLVM_SYS_150_PREFIX=$(brew --prefix llvm@15)
```

**Windows:**
```powershell
choco install llvm --version=15.0.7
$env:LLVM_SYS_150_PREFIX = "C:\Program Files\LLVM"
```

### 3. Build BLAZE
```bash
git clone https://github.com/yourusername/blaze.git
cd blaze
cargo build --release
```

## Your First Program

### Create a file `hello.blz`:
```blaze
fn main() {
    println("Hello, BLAZE! 🔥");
}
```

### Check syntax:
```bash
blaze check hello.blz
```

### Build and run:
```bash
blaze build hello.blz -o hello
./hello
```

### Or run directly:
```bash
blaze run hello.blz
```

## Basic Examples

### Variables
```blaze
fn main() {
    let x: i32 = 42;
    let mut y = 10;
    y = 20;
    println(x, y);
}
```

### Functions
```blaze
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn main() {
    let result = add(5, 7);
    println(result);
}
```

### Structs
```blaze
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let p = Point { x: 10, y: 20 };
    println(p.x, p.y);
}
```

### Pattern Matching
```blaze
enum Option<T> {
    Some(T),
    None,
}

fn main() {
    let value = Option::Some(42);
    match value {
        Option::Some(x) => println(x),
        Option::None => println("No value"),
    }
}
```

## Command Reference

### Check for errors:
```bash
blaze check file.blz
```

### Build executable:
```bash
blaze build file.blz -o output
```

### Build with optimization:
```bash
blaze build file.blz -O2 -o output
```

### Run directly:
```bash
blaze run file.blz
```

### View IR:
```bash
blaze build file.blz --emit-ir
```

### View assembly:
```bash
blaze build file.blz --emit-asm
```

### Create new project:
```bash
blaze init my_project
```

## Optimization Levels

- `-O0`: No optimization (fast compilation)
- `-O1`: Basic optimization
- `-O2`: Moderate optimization (recommended)
- `-O3`: Aggressive optimization

## Common Issues

### LLVM Not Found
```bash
export LLVM_SYS_150_PREFIX=/usr/lib/llvm-15
```

### Linker Errors
**Ubuntu:**
```bash
sudo apt-get install build-essential
```

**macOS:**
```bash
xcode-select --install
```

## Next Steps

1. Explore examples in `examples/`
2. Read `INSTALLATION.md` for detailed setup
3. Check `CONTRIBUTING.md` to contribute
4. Try building larger projects

## Getting Help

- Check error messages for suggestions
- Review examples for common patterns
- Open GitHub issue for bugs
- Join discussions for questions

## Resources

- Examples: `examples/`
- Documentation: `README.md`
- Installation: `INSTALLATION.md`
- Contributing: `CONTRIBUTING.md`

Happy coding with BLAZE! 🔥
