# 🤝 Contributing to BLAZE Compiler

Thank you for your interest in contributing to BLAZE! This document provides guidelines for contributing to the BLAZE Compiler project.

## 🚀 **Getting Started**

### Prerequisites

- Rust 1.70+ installed
- Git installed
- Basic understanding of compiler design
- Familiarity with Rust programming

### Setting Up Development Environment

1. **Fork the repository**

   ```bash
   git clone https://github.com/your-username/blaze.git
   cd blaze
   ```

2. **Build the project**

   ```bash
   cargo build
   cargo test
   ```

3. **Run examples**
   ```bash
   cargo run -- check examples/hello.blz
   ```

## 📋 **Development Guidelines**

### Code Style

- Follow Rust naming conventions
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Write comprehensive tests
- Document public APIs

### Commit Messages

Use clear, descriptive commit messages:

```
feat: add support for struct definitions
fix: resolve parser error with nested expressions
docs: update README with new examples
test: add integration tests for type checker
```

### Pull Request Process

1. **Create a feature branch**

   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**

   - Write code
   - Add tests
   - Update documentation

3. **Test your changes**

   ```bash
   cargo test
   cargo build --release
   ```

4. **Submit a pull request**
   - Clear description of changes
   - Reference any related issues
   - Include test results

## 🧪 **Testing**

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test lexer_tests
cargo test parser_tests
cargo test integration_tests

# Run benchmarks
cargo bench
```

### Adding Tests

- **Unit tests** - Test individual functions
- **Integration tests** - Test complete workflows
- **Example tests** - Test with real BLAZE code

### Test Structure

```rust
#[test]
fn test_feature_name() {
    // Arrange
    let input = "test input";

    // Act
    let result = function_under_test(input);

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_output);
}
```

## 🏗️ **Architecture Overview**

### Core Components

1. **Lexer** (`src/lexer/`)

   - Tokenizes source code
   - Handles keywords, operators, literals
   - Error reporting for invalid tokens

2. **Parser** (`src/parser/`)

   - Builds Abstract Syntax Tree (AST)
   - Handles grammar rules
   - Error recovery and reporting

3. **Semantic Analysis** (`src/semantic/`)

   - Type checking
   - Symbol table management
   - Scope resolution

4. **Code Generation** (`src/codegen/`)
   - IR generation
   - Machine code generation
   - Optimization passes

### Adding New Features

1. **Language Features**

   - Add tokens to `lexer/token.rs`
   - Update parser in `parser/mod.rs`
   - Add AST nodes in `parser/ast.rs`
   - Implement semantic analysis
   - Add code generation

2. **Tooling Features**
   - Add CLI commands in `main.rs`
   - Implement new compiler passes
   - Add debugging tools

## 🐛 **Bug Reports**

When reporting bugs, please include:

- **Description** - What happened?\*\*
- **Steps to Reproduce** - How to trigger the bug
- **Expected Behavior** - What should happen
- **Actual Behavior** - What actually happens
- **Environment** - OS, Rust version, etc.
- **Code Sample** - Minimal code that reproduces the issue

## 💡 **Feature Requests**

When requesting features:

- **Description** - What feature do you want?
- **Use Case** - Why is this feature needed?
- **Proposed Solution** - How should it work?
- **Alternatives** - Other ways to solve the problem

## 📚 **Documentation**

### Code Documentation

- Document all public APIs
- Include examples in doc comments
- Keep documentation up to date

### User Documentation

- Update README.md for new features
- Add examples to `examples/` directory
- Update setup instructions

## 🎯 **Areas for Contribution**

### High Priority

- **Type System** - Advanced type checking
- **Borrow Checker** - Memory safety analysis
- **Optimization** - Performance improvements
- **Error Messages** - Better error reporting

### Medium Priority

- **Standard Library** - Core library functions
- **Tooling** - Development tools
- **Documentation** - User guides
- **Examples** - More example programs

### Low Priority

- **IDE Support** - Language server
- **Debugging** - Debug information
- **Profiling** - Performance analysis
- **Cross-compilation** - Target other platforms

## 🏆 **Recognition**

Contributors will be recognized in:

- **README.md** - Contributor list
- **CHANGELOG.md** - Contribution history
- **Release Notes** - Feature acknowledgments

## 📞 **Getting Help**

- **GitHub Issues** - Bug reports and feature requests
- **Discussions** - General questions and ideas
- **Email** - black@blaze-lang.dev
- **Discord** - [BLAZE Community](https://discord.gg/blaze)

## 🙏 **Thank You**

Thank you for contributing to BLAZE! Your efforts help make BLAZE a better programming language for everyone.

---

**Developed with ❤️ by BLACK**

_"Building the future of programming together"_
