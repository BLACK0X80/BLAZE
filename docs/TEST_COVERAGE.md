# Test Coverage Guide

This document describes how to measure and improve test coverage for the BLAZE Compiler.

## Overview

Test coverage measures how much of the codebase is executed during testing. The goal is to achieve 90%+ coverage to ensure the compiler is well-tested and reliable.

## Coverage Tools

We support two coverage tools:

### 1. cargo-tarpaulin (Recommended for Linux)

**Installation:**
```bash
cargo install cargo-tarpaulin
```

**Usage:**
```bash
# Run coverage analysis
cargo tarpaulin --out Html --out Xml --output-dir target/coverage

# Or use the provided script
./scripts/coverage.sh  # Linux/macOS
.\scripts\coverage.ps1  # Windows
```

**Pros:**
- Easy to use
- Fast
- Good HTML reports

**Cons:**
- Linux only (doesn't work on Windows/macOS)

### 2. grcov (Cross-platform)

**Installation:**
```bash
cargo install grcov
```

**Usage:**
```bash
# Set environment variables
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="target/coverage/cargo-test-%p-%m.profraw"

# Run tests
cargo test

# Generate coverage report
grcov . \
    --binary-path ./target/debug/deps/ \
    --source-dir . \
    --output-type html \
    --branch \
    --ignore-not-existing \
    --output-path target/coverage/html
```

**Pros:**
- Cross-platform (Windows, Linux, macOS)
- Detailed branch coverage
- Multiple output formats

**Cons:**
- More complex setup
- Requires LLVM tools

## Running Coverage Analysis

### Using Scripts (Recommended)

We provide scripts that automatically detect and use the available coverage tool:

**Linux/macOS:**
```bash
chmod +x scripts/coverage.sh
./scripts/coverage.sh
```

**Windows:**
```powershell
.\scripts\coverage.ps1
```

### Manual Execution

**With tarpaulin:**
```bash
cargo tarpaulin \
    --out Html \
    --out Xml \
    --output-dir target/coverage \
    --exclude-files "tests/*" \
    --exclude-files "benches/*" \
    --exclude-files "examples/*" \
    --timeout 300 \
    --verbose
```

**With grcov:**
```bash
# Clean previous data
rm -rf target/coverage
mkdir -p target/coverage

# Set environment
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="target/coverage/cargo-test-%p-%m.profraw"

# Run tests
cargo test

# Generate report
grcov . \
    --binary-path ./target/debug/deps/ \
    --source-dir . \
    --output-type html \
    --branch \
    --ignore-not-existing \
    --ignore "tests/*" \
    --ignore "benches/*" \
    --ignore "examples/*" \
    --output-path target/coverage/html
```

## Coverage Reports

After running coverage analysis, reports are generated in the `target/coverage` directory:

- **HTML Report**: `target/coverage/tarpaulin-report.html` or `target/coverage/html/index.html`
- **XML Report**: `target/coverage/cobertura.xml` (tarpaulin only)

Open the HTML report in your browser to see:
- Overall coverage percentage
- Coverage by file
- Line-by-line coverage visualization
- Uncovered code paths

## Current Test Coverage

### Test Files

1. **Unit Tests**
   - `tests/lexer_tests.rs` - Lexer functionality
   - `tests/parser_tests.rs` - Parser functionality
   - `tests/diagnostics_tests.rs` - Diagnostic system
   - `tests/suggestions_tests.rs` - Error suggestions
   - `tests/runtime_tests.rs` - Runtime allocator and intrinsics

2. **Integration Tests**
   - `tests/integration_tests.rs` - Basic integration tests
   - `tests/comprehensive_tests.rs` - Comprehensive feature tests
   - `tests/end_to_end_tests.rs` - Complete compilation pipeline tests
   - `tests/codegen_integration_tests.rs` - Code generation tests

3. **Performance Tests**
   - `tests/performance_tests.rs` - Performance benchmarks

4. **Error Handling Tests**
   - `tests/error_handling_tests.rs` - Error detection and reporting

5. **Example Program Tests**
   - `tests/example_programs_test.rs` - Tests for example programs

### Coverage by Module

Target coverage for each module:

| Module | Target | Description |
|--------|--------|-------------|
| `lexer` | 95%+ | Tokenization and lexical analysis |
| `parser` | 95%+ | Syntax analysis and AST construction |
| `semantic` | 90%+ | Type checking, borrow checking, symbol resolution |
| `ir` | 90%+ | IR generation, optimization, validation |
| `codegen` | 85%+ | LLVM backend, register allocation, linking |
| `runtime` | 90%+ | Memory allocator, intrinsics, panic handling |
| `stdlib` | 90%+ | Standard library collections |
| `error` | 95%+ | Error reporting and diagnostics |
| `utils` | 90%+ | Utility functions and helpers |

## Improving Coverage

### 1. Identify Untested Code

Run coverage analysis and look for:
- Red lines (uncovered code)
- Low coverage percentages
- Entire functions without tests

### 2. Add Missing Tests

For each untested code path:

1. **Understand the code**: Read the function/module to understand what it does
2. **Write test cases**: Create tests that exercise the untested paths
3. **Verify coverage**: Run coverage again to confirm improvement

### 3. Focus on Critical Paths

Prioritize testing:
- Error handling paths
- Edge cases and boundary conditions
- Complex logic and algorithms
- Public APIs

### 4. Test Categories

Ensure you have tests for:

**Positive Tests** (should succeed):
```rust
#[test]
fn test_valid_program() {
    let source = "fn main() { let x = 42; }";
    assert!(compile(source).is_ok());
}
```

**Negative Tests** (should fail):
```rust
#[test]
fn test_invalid_syntax() {
    let source = "fn main() { let x = ; }";
    assert!(compile(source).is_err());
}
```

**Edge Cases**:
```rust
#[test]
fn test_empty_program() {
    let source = "";
    assert!(compile(source).is_err());
}
```

**Integration Tests**:
```rust
#[test]
fn test_complete_pipeline() {
    let source = "fn main() { let x = 42; }";
    let output = PathBuf::from("target/test_output");
    assert!(compile_to_executable(source, output, 0).is_ok());
}
```

## Coverage Goals

### Short-term (Current Sprint)
- [ ] Achieve 80%+ overall coverage
- [ ] 90%+ coverage for lexer and parser
- [ ] 85%+ coverage for semantic analysis
- [ ] 80%+ coverage for code generation

### Medium-term (Next Sprint)
- [ ] Achieve 85%+ overall coverage
- [ ] 95%+ coverage for error handling
- [ ] 90%+ coverage for IR generation
- [ ] Add property-based tests

### Long-term (Future)
- [ ] Achieve 90%+ overall coverage
- [ ] 95%+ coverage for all critical modules
- [ ] Automated coverage tracking in CI/CD
- [ ] Coverage regression prevention

## Best Practices

### 1. Write Tests First (TDD)
When adding new features:
1. Write tests for the feature
2. Implement the feature
3. Verify tests pass
4. Check coverage

### 2. Test Public APIs
Focus on testing public interfaces:
```rust
// Good: Test public API
#[test]
fn test_compile_function() {
    let result = compile("fn main() {}");
    assert!(result.is_ok());
}

// Less important: Test internal implementation details
```

### 3. Test Error Paths
Ensure error handling is tested:
```rust
#[test]
fn test_error_handling() {
    let result = compile("invalid syntax");
    assert!(result.is_err());
    
    if let Err(e) = result {
        // Verify error message is helpful
        assert!(!format!("{:?}", e).is_empty());
    }
}
```

### 4. Use Test Fixtures
Create reusable test data:
```rust
fn valid_program() -> &'static str {
    "fn main() { let x = 42; }"
}

#[test]
fn test_compilation() {
    assert!(compile(valid_program()).is_ok());
}
```

### 5. Avoid Flaky Tests
- Don't rely on timing
- Don't depend on external resources
- Use deterministic test data
- Clean up test artifacts

## Continuous Integration

### GitHub Actions

Add coverage to CI pipeline:

```yaml
name: Coverage

on: [push, pull_request]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      - name: Run coverage
        run: cargo tarpaulin --out Xml
      - name: Upload coverage
        uses: codecov/codecov-action@v2
```

### Coverage Badges

Add coverage badge to README:

```markdown
[![Coverage](https://codecov.io/gh/username/blaze-compiler/branch/main/graph/badge.svg)](https://codecov.io/gh/username/blaze-compiler)
```

## Troubleshooting

### Issue: Coverage tool not found

**Solution**: Install the coverage tool:
```bash
cargo install cargo-tarpaulin  # or
cargo install grcov
```

### Issue: Tests fail during coverage

**Solution**: Run tests normally first to ensure they pass:
```bash
cargo test
```

### Issue: Low coverage despite many tests

**Solution**: 
1. Check if tests are actually running the code
2. Look for dead code or unreachable paths
3. Add tests for error paths and edge cases

### Issue: Coverage report not generated

**Solution**:
1. Check for errors in the output
2. Ensure output directory exists
3. Try running with `--verbose` flag

## Resources

- [cargo-tarpaulin documentation](https://github.com/xd009642/tarpaulin)
- [grcov documentation](https://github.com/mozilla/grcov)
- [Rust testing guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Code coverage best practices](https://martinfowler.com/bliki/TestCoverage.html)
