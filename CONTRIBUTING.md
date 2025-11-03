# Contributing to BLAZE

Thank you for your interest in contributing! 🎉

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/blaze.git`
3. Create a branch: `git checkout -b feature/my-feature` 
4. Make your changes
5. Run tests: `cargo test` 
6. Commit: `git commit -m "Add my feature"` 
7. Push: `git push origin feature/my-feature` 
8. Open a Pull Request

## Development Setup

### Required Tools

- Rust 1.70+
- LLVM 15
- cargo-fmt: `rustup component add rustfmt` 
- cargo-clippy: `rustup component add clippy` 

### Code Style

Run before committing:
```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

## Testing

All new features must include tests:
```bash
cargo test
cargo test --test integration_tests
cargo test --doc
```

## Areas for Contribution

### 🐛 Bug Fixes
Check Issues labeled bug
Include test case that reproduces the bug

### ✨ New Features
Discuss in Discussions first
Update documentation
Add comprehensive tests

### 📚 Documentation
Improve README
Add examples
Write tutorials

### 🧪 Testing
Increase test coverage
Add edge case tests
Performance benchmarks

### 🎨 Code Quality
Refactoring
Performance improvements
Better error messages

## Pull Request Guidelines

### Title Format
- fix: Brief description for bug fixes
- feat: Brief description for new features
- docs: Brief description for documentation
- test: Brief description for tests
- refactor: Brief description for refactoring

### Description Template
```
## What does this PR do?
Brief description of changes

## Why is this needed?
Motivation and context

## How has this been tested?
Describe the tests

## Checklist
- [ ] Tests pass locally
- [ ] Code follows style guidelines
- [ ] Documentation updated
- [ ] CHANGELOG updated (for significant changes)
```

## Code Review Process

- Maintainer reviews within 2-3 days
- Address feedback
- Approved PRs are merged
- Changes included in next release

## Questions?

Feel free to ask in Discussions or open an issue!

Thank you for contributing! 🙏
