# Changelog

All notable changes to the BLAZE compiler.

## [Unreleased]

### Added
- Complete optimizer module with multi-pass optimization
- CLI interface with clap integration
- Comprehensive examples (fibonacci, structs, ownership, pattern matching, generics)
- Test utilities and infrastructure
- Rich error handling with suggestions
- BorrowError and LifetimeError variants
- Installation guide with platform-specific instructions
- Contribution guidelines

### Enhanced
- Lexer with bitwise operators (^, <<, >>)
- Parser with comprehensive AST nodes
- Error messages with source snippets and suggestions
- Type system with additional variants

### Fixed
- Missing TokenTypes for operators
- Parser type mismatches
- AST compatibility issues
- Error Display implementations

### Changed
- Removed documentation comments from production code
- Streamlined public API
- Improved code organization

## [0.1.0] - Initial Release

### Added
- Basic lexer and parser
- Semantic analysis (type checking, borrow checking)
- IR generation
- LLVM backend integration
- Runtime system
- Standard library foundation
- Basic error handling

### Features
- Memory safety through ownership
- Borrow checker
- Lifetime analysis
- Type inference
- Pattern matching
- Generics support

### Supported Platforms
- Windows 10+
- macOS 10.15+
- Ubuntu 20.04+

### Requirements
- Rust 1.70+
- LLVM 15
- C/C++ compiler
