# BLAZE Examples

Production-ready examples demonstrating BLAZE language features.

## Running Examples

```bash
blaze check examples/02_fibonacci.blz
blaze build examples/02_fibonacci.blz -o fibonacci
./fibonacci

blaze run examples/02_fibonacci.blz
```

## Examples

### 01_hello.blz
Basic hello world program demonstrating:
- Function definition
- Print statements
- Entry point

### 02_fibonacci.blz
Fibonacci sequence implementations:
- Recursive algorithm
- Iterative algorithm
- Performance comparison
- Mutable variables
- While loops

### 03_structs.blz
Struct usage demonstrating:
- Struct definitions
- Field access
- Methods taking references
- Constructor functions
- Nested structs
- Mathematical operations

### 04_ownership.blz
Memory safety features:
- Taking ownership
- Immutable borrowing
- Mutable borrowing
- Ownership transfer
- Return value ownership
- Multiple immutable borrows

### 05_pattern_matching.blz
Pattern matching and enums:
- Enum definitions
- Generic enums
- Match expressions
- Result and Option types
- Error handling patterns

### 06_generics.blz
Generic programming:
- Generic functions
- Type parameters
- Trait bounds
- Generic structs
- Type inference

## Building All Examples

```bash
cd examples
for file in *.blz; do
    blaze build "$file" -o "${file%.blz}"
done
```

## Testing Examples

```bash
cd examples
for file in *.blz; do
    echo "Checking $file..."
    blaze check "$file"
done
```

## Optimization Levels

```bash
blaze build examples/02_fibonacci.blz -O0 -o fib_debug
blaze build examples/02_fibonacci.blz -O2 -o fib_optimized
blaze build examples/02_fibonacci.blz -O3 --release -o fib_release
```

## Output Options

```bash
blaze build examples/03_structs.blz --emit-ir
blaze build examples/03_structs.blz --emit-asm
```

## Common Patterns

### Error Handling
```blaze
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        return Result::Err(String::from("Division by zero"));
    }
    return Result::Ok(a / b);
}
```

### Ownership Transfer
```blaze
fn take_ownership(s: String) {
    println("Now I own: ", s);
}
```

### Borrowing
```blaze
fn borrow_immutable(s: &String) {
    println("Borrowed: ", s);
}

fn borrow_mutable(s: &mut String) {
    s.push_str(" modified");
}
```

### Pattern Matching
```blaze
match value {
    Option::Some(x) => println("Value: ", x),
    Option::None => println("No value"),
}
```

### Generics
```blaze
fn swap<T>(a: &mut T, b: &mut T) {
    let temp = *a;
    *a = *b;
    *b = temp;
}
```

## Performance Tips

1. Use optimization flags for production builds
2. Prefer iterative over recursive for performance-critical code
3. Use references to avoid unnecessary copies
4. Enable inlining for small functions with `-O2` or `-O3`

## Debugging

```bash
blaze build examples/02_fibonacci.blz -O0 --verbose
```

## Getting Help

- Check compiler errors for suggestions
- Review other examples for patterns
- Read INSTALLATION.md for setup issues
- Open GitHub issue for bugs
