# BLAZE Troubleshooting Guide

This guide helps you diagnose and fix common issues when using the BLAZE compiler.

## Table of Contents

1. [Installation Issues](#installation-issues)
2. [Compilation Errors](#compilation-errors)
3. [Runtime Errors](#runtime-errors)
4. [Performance Issues](#performance-issues)
5. [Platform-Specific Issues](#platform-specific-issues)

## Installation Issues

### LLVM Not Found

**Error**:
```
error: No suitable version of LLVM was found system-wide
```

**Solution**:

1. Install LLVM 15.0 for your platform (see [LLVM_SETUP.md](../LLVM_SETUP.md))

2. Set the `LLVM_SYS_150_PREFIX` environment variable:

   **Windows**:
   ```cmd
   set LLVM_SYS_150_PREFIX=C:\Program Files\LLVM
   ```

   **Linux/macOS**:
   ```bash
   export LLVM_SYS_150_PREFIX=/usr/lib/llvm-15
   ```

3. Rebuild the compiler:
   ```bash
   cargo clean
   cargo build --release
   ```

### Rust Version Too Old

**Error**:
```
error: package requires rustc 1.70.0 or newer
```

**Solution**:

Update Rust to the latest version:
```bash
rustup update stable
```

### Build Fails on Windows

**Error**:
```
error: linker `link.exe` not found
```

**Solution**:

Install Visual Studio Build Tools:
1. Download from https://visualstudio.microsoft.com/downloads/
2. Select "Desktop development with C++"
3. Restart your terminal
4. Rebuild: `cargo build --release`

### Build Fails on Linux

**Error**:
```
error: linking with `cc` failed
```

**Solution**:

Install build essentials:
```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# Fedora/RHEL
sudo dnf install gcc gcc-c++

# Arch
sudo pacman -S base-devel
```

## Compilation Errors

### Syntax Errors

#### Missing Semicolon

**Error**:
```
Parser error at 5:10: expected `;`
```

**Example**:
```rust
fn main() {
    let x = 42  // Missing semicolon
    println(x);
}
```

**Solution**:
Add semicolon after the statement:
```rust
fn main() {
    let x = 42;  // Fixed
    println(x);
}
```

#### Mismatched Braces

**Error**:
```
Parser error at 8:1: expected `}`, found end of file
```

**Example**:
```rust
fn main() {
    if x > 0 {
        println("positive");
    // Missing closing brace
}
```

**Solution**:
Ensure all braces are matched:
```rust
fn main() {
    if x > 0 {
        println("positive");
    }  // Fixed
}
```

### Type Errors

#### Type Mismatch

**Error**:
```
Type error at 3:17: mismatched types
  expected: i32
  found: bool
```

**Example**:
```rust
fn main() {
    let x: i32 = true;  // Wrong type
}
```

**Solution**:
Use the correct type:
```rust
fn main() {
    let x: i32 = 42;     // Fixed
    let y: bool = true;  // Or use bool
}
```

#### Function Argument Type Mismatch

**Error**:
```
Type error at 7:15: argument type mismatch
  expected: i32
  found: String
```

**Example**:
```rust
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn main() {
    let result = add(5, "10");  // Wrong type
}
```

**Solution**:
Pass the correct type:
```rust
fn main() {
    let result = add(5, 10);  // Fixed
}
```

#### Return Type Mismatch

**Error**:
```
Type error at 2:12: return type mismatch
  expected: i32
  found: bool
```

**Example**:
```rust
fn get_number() -> i32 {
    return true;  // Wrong return type
}
```

**Solution**:
Return the correct type:
```rust
fn get_number() -> i32 {
    return 42;  // Fixed
}
```

### Borrow Checker Errors

#### Multiple Mutable Borrows

**Error**:
```
Borrow error: cannot borrow `x` as mutable more than once at a time
```

**Example**:
```rust
fn main() {
    let mut x = 42;
    let r1 = &mut x;
    let r2 = &mut x;  // Error: second mutable borrow
    *r1 = 43;
}
```

**Solution**:
Use only one mutable borrow at a time:
```rust
fn main() {
    let mut x = 42;
    let r1 = &mut x;
    *r1 = 43;
    // r1 is no longer used, so we can create r2
    let r2 = &mut x;
    *r2 = 44;
}
```

#### Mutable and Immutable Borrow Conflict

**Error**:
```
Borrow error: cannot borrow `x` as mutable because it is also borrowed as immutable
```

**Example**:
```rust
fn main() {
    let mut x = 42;
    let r1 = &x;      // Immutable borrow
    let r2 = &mut x;  // Error: mutable borrow while immutable exists
    println(r1);
}
```

**Solution**:
Don't mix mutable and immutable borrows:
```rust
fn main() {
    let mut x = 42;
    let r1 = &x;
    println(r1);
    // r1 is no longer used
    let r2 = &mut x;  // Now OK
    *r2 = 43;
}
```

#### Use After Move

**Error**:
```
Borrow error: use of moved value: `s`
```

**Example**:
```rust
fn main() {
    let s = "hello";
    let s2 = s;      // s is moved to s2
    println(s);      // Error: s was moved
}
```

**Solution**:
Use a reference instead of moving:
```rust
fn main() {
    let s = "hello";
    let s2 = &s;     // Borrow instead of move
    println(s);      // OK: s is still valid
    println(s2);
}
```

### Undefined Variables/Functions

#### Undefined Variable

**Error**:
```
Semantic error: cannot find value `x` in this scope
```

**Example**:
```rust
fn main() {
    println(x);  // x is not defined
}
```

**Solution**:
Define the variable before using it:
```rust
fn main() {
    let x = 42;
    println(x);  // Fixed
}
```

#### Undefined Function

**Error**:
```
Semantic error: cannot find function `calculate` in this scope
```

**Example**:
```rust
fn main() {
    let result = calculate(5);  // Function not defined
}
```

**Solution**:
Define the function:
```rust
fn calculate(x: i32) -> i32 {
    return x * 2;
}

fn main() {
    let result = calculate(5);  // Fixed
}
```

## Runtime Errors

### Segmentation Fault

**Symptom**: Program crashes with "Segmentation fault"

**Common Causes**:
1. Array out of bounds access
2. Null pointer dereference
3. Stack overflow (infinite recursion)

**Solution**:

Check array bounds:
```rust
// Bad
let arr = [1, 2, 3];
let x = arr[10];  // Out of bounds

// Good
let arr = [1, 2, 3];
if index < arr.len() {
    let x = arr[index];
}
```

Avoid infinite recursion:
```rust
// Bad
fn infinite() {
    infinite();  // Stack overflow
}

// Good
fn factorial(n: i32) -> i32 {
    if n <= 1 {
        return 1;  // Base case
    }
    return n * factorial(n - 1);
}
```

### Memory Leaks

**Symptom**: Program uses increasing amounts of memory

**Diagnosis**:

Run with leak detection enabled (debug builds):
```bash
BLAZE_LEAK_CHECK=1 ./my_program
```

**Solution**:

Ensure all allocated memory is freed:
```rust
// The compiler handles this automatically in most cases
// Manual memory management is not exposed in BLAZE
```

### Division by Zero

**Symptom**: Program crashes or produces unexpected results

**Example**:
```rust
fn divide(a: i32, b: i32) -> i32 {
    return a / b;  // Crashes if b is 0
}
```

**Solution**:
Check for zero before dividing:
```rust
fn divide(a: i32, b: i32) -> i32 {
    if b == 0 {
        return 0;  // Or handle error appropriately
    }
    return a / b;
}
```

## Performance Issues

### Slow Compilation

**Symptom**: Compilation takes a long time

**Solutions**:

1. **Use debug builds during development**:
   ```bash
   blaze build program.blz -O0  # Fastest compilation
   ```

2. **Only use optimization for release builds**:
   ```bash
   blaze build program.blz -O2  # For release
   ```

3. **Check for large generated code**:
   - Avoid deeply nested loops
   - Reduce code duplication

### Slow Runtime Performance

**Symptom**: Program runs slower than expected

**Solutions**:

1. **Enable optimization**:
   ```bash
   blaze build program.blz -O3  # Maximum optimization
   ```

2. **Profile your code**:
   - Identify hot spots
   - Optimize critical sections

3. **Algorithm improvements**:
   - Use better algorithms (O(n log n) vs O(n²))
   - Reduce unnecessary allocations

4. **Avoid unnecessary copies**:
   ```rust
   // Bad: Copies data
   fn process(data: Vec<i32>) {
       // ...
   }

   // Good: Borrows data
   fn process(data: &Vec<i32>) {
       // ...
   }
   ```

### High Memory Usage

**Symptom**: Program uses too much memory

**Solutions**:

1. **Avoid unnecessary allocations**:
   ```rust
   // Bad: Creates many temporary strings
   for i in 0..1000 {
       let s = format("Item {}", i);
   }

   // Good: Reuse buffer
   let mut buffer = String::new();
   for i in 0..1000 {
       buffer.clear();
       // Use buffer
   }
   ```

2. **Use references instead of cloning**:
   ```rust
   // Bad: Clones data
   let copy = data.clone();

   // Good: Borrows data
   let reference = &data;
   ```

3. **Free resources when done**:
   ```rust
   // Resources are automatically freed when they go out of scope
   {
       let large_data = Vec::new();
       // Use large_data
   }  // large_data is freed here
   ```

## Platform-Specific Issues

### Windows Issues

#### MSVC Linker Not Found

**Error**:
```
error: linker `link.exe` not found
```

**Solution**:
Install Visual Studio Build Tools and ensure they're in your PATH.

#### Path Too Long

**Error**:
```
error: path too long
```

**Solution**:
1. Enable long paths in Windows:
   ```cmd
   reg add HKLM\SYSTEM\CurrentControlSet\Control\FileSystem /v LongPathsEnabled /t REG_DWORD /d 1
   ```
2. Or use shorter directory names

### Linux Issues

#### Missing Libraries

**Error**:
```
error while loading shared libraries: libLLVM-15.so
```

**Solution**:
Add LLVM library path to `LD_LIBRARY_PATH`:
```bash
export LD_LIBRARY_PATH=/usr/lib/llvm-15/lib:$LD_LIBRARY_PATH
```

#### Permission Denied

**Error**:
```
Permission denied (os error 13)
```

**Solution**:
Make the executable file executable:
```bash
chmod +x my_program
```

### macOS Issues

#### Code Signing

**Error**:
```
"my_program" cannot be opened because the developer cannot be verified
```

**Solution**:
1. Right-click the executable
2. Select "Open"
3. Click "Open" in the dialog

Or disable Gatekeeper temporarily:
```bash
sudo spctl --master-disable
```

#### Library Not Found

**Error**:
```
dyld: Library not loaded: @rpath/libLLVM.dylib
```

**Solution**:
Set `DYLD_LIBRARY_PATH`:
```bash
export DYLD_LIBRARY_PATH=/usr/local/opt/llvm@15/lib:$DYLD_LIBRARY_PATH
```

## Getting Help

If you can't resolve your issue:

1. **Check the documentation**:
   - [Getting Started Guide](GETTING_STARTED.md)
   - [Language Reference](LANGUAGE_REFERENCE.md)
   - [Compiler Architecture](COMPILER_ARCHITECTURE.md)

2. **Search existing issues**:
   - Check GitHub Issues for similar problems

3. **Create a minimal reproduction**:
   - Reduce your code to the smallest example that shows the problem

4. **Report the issue**:
   - Open a GitHub Issue with:
     - BLAZE version (`blaze --version`)
     - Operating system and version
     - Minimal reproduction code
     - Full error message
     - Steps to reproduce

5. **Ask for help**:
   - GitHub Discussions for questions
   - Include relevant context and what you've tried

## Debugging Tips

### Enable Verbose Output

```bash
blaze build program.blz --verbose
```

### Check Intermediate Representations

```bash
# View IR
blaze build program.blz --emit-ir

# View assembly
blaze build program.blz --emit-asm
```

### Use Debug Builds

Debug builds include more information:
```bash
cargo build  # Debug build
./target/debug/blaze check program.blz
```

### Add Print Statements

Add `println` statements to debug your code:
```rust
fn main() {
    let x = 42;
    println("x = ");
    println(x);
    
    let y = x * 2;
    println("y = ");
    println(y);
}
```

### Check Compiler Version

```bash
blaze --version
rustc --version
llvm-config --version
```

## Common Patterns

### Safe Array Access

```rust
fn get_element(arr: &[i32], index: usize) -> i32 {
    if index < arr.len() {
        return arr[index];
    }
    return 0;  // Default value
}
```

### Safe Division

```rust
fn safe_divide(a: i32, b: i32) -> i32 {
    if b == 0 {
        return 0;
    }
    return a / b;
}
```

### Resource Management

```rust
fn process_file(path: String) {
    // Resources are automatically cleaned up
    // when they go out of scope
    let data = read_file(path);
    // Process data
}  // data is freed here
```

## Prevention

### Best Practices

1. **Use type annotations**: Make types explicit for clarity
2. **Check bounds**: Always validate array indices
3. **Handle errors**: Check for error conditions
4. **Test thoroughly**: Write tests for edge cases
5. **Use the borrow checker**: Let it guide you to safe code
6. **Enable warnings**: Fix all compiler warnings
7. **Run tests**: Use `cargo test` regularly
8. **Use version control**: Commit working code frequently

### Code Review Checklist

- [ ] All variables are initialized before use
- [ ] Array accesses are bounds-checked
- [ ] Division operations check for zero
- [ ] Borrow checker rules are followed
- [ ] Types are correct and consistent
- [ ] Functions have clear return types
- [ ] Error cases are handled
- [ ] Code is tested

## Additional Resources

- [LLVM Setup Guide](../LLVM_SETUP.md)
- [Test Coverage Guide](TEST_COVERAGE.md)
- [Diagnostic System Guide](DIAGNOSTIC_SYSTEM_GUIDE.md)
- [GitHub Issues](https://github.com/yourusername/blaze/issues)
- [GitHub Discussions](https://github.com/yourusername/blaze/discussions)
