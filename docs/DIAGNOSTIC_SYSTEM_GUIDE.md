# BLAZE Diagnostic System - Developer Guide

## Quick Start

### Creating a Simple Error

```rust
use blaze::error::{Diagnostic, DiagnosticEmitter};

let mut emitter = DiagnosticEmitter::new();
let diag = Diagnostic::error("undefined variable `x`");
println!("{}", emitter.emit(&diag));
```

### Creating an Error with Location

```rust
use blaze::error::{DiagnosticBuilder};
use blaze::utils::Span;

let diag = DiagnosticBuilder::error("type mismatch")
    .with_location("main.blz", Span::new(10, 15, 2, 5))
    .with_label("expected i32, found f64")
    .build();
```

### Adding Suggestions

```rust
let diag = DiagnosticBuilder::error("cannot find function `printl`")
    .with_suggestion("did you mean `println`?")
    .build();
```

### Using "Did You Mean" Helper

```rust
use blaze::error::did_you_mean;

let candidates = vec!["println", "print", "format"];
if let Some(suggestion) = did_you_mean("printl", &candidates) {
    // suggestion = "did you mean `println`?"
}
```

## Collecting Multiple Errors

```rust
use blaze::error::{DiagnosticCollector, Diagnostic};

let mut collector = DiagnosticCollector::new();

// Add errors during compilation
collector.add(Diagnostic::error("error 1"));
collector.add(Diagnostic::warning("warning 1"));
collector.add(Diagnostic::error("error 2"));

// Check if there were errors
if collector.has_errors() {
    // Emit all diagnostics
    let mut emitter = DiagnosticEmitter::new();
    println!("{}", collector.emit_all(&mut emitter));
}
```

## Showing Source Context

```rust
use blaze::utils::SourceMap;

// Create source map and add files
let mut source_map = SourceMap::new();
source_map.add_file("main.blz".to_string(), source_code.to_string());

// Create emitter with source map
let mut emitter = DiagnosticEmitter::new()
    .with_source_map(source_map);

// Errors will now show source context
let diag = Diagnostic::error("type mismatch")
    .with_location("main.blz", span);
println!("{}", emitter.emit(&diag));
```

## Type Conversion Suggestions

```rust
use blaze::error::suggest_type_conversion;

if let Some(suggestion) = suggest_type_conversion("i32", "f64") {
    // suggestion = "try converting with `as f64`"
}
```

## Borrow Error Suggestions

```rust
use blaze::error::suggest_borrow_fix;

if let Some(suggestion) = suggest_borrow_fix("multiple_mutable_borrows") {
    // suggestion = "consider using separate scopes..."
}
```

## Syntax Error Suggestions

```rust
use blaze::error::suggest_syntax_fix;

if let Some(suggestion) = suggest_syntax_fix("missing_semicolon") {
    // suggestion = "add a semicolon `;` at the end of the statement"
}
```

## Example Code Patterns

```rust
use blaze::error::example_for_pattern;

if let Some(example) = example_for_pattern("function_definition") {
    // example = "fn function_name(param: Type) -> ReturnType { ... }"
}
```

## Customizing the Emitter

```rust
let emitter = DiagnosticEmitter::new()
    .with_colors(false)           // Disable colors
    .with_max_errors(20)          // Allow up to 20 errors
    .with_source_map(source_map); // Add source context
```

## Best Practices

### 1. Use Error Codes
```rust
let diag = DiagnosticBuilder::error("type mismatch")
    .with_code("E0308")  // Makes errors searchable
    .build();
```

### 2. Provide Context with Notes
```rust
let diag = DiagnosticBuilder::error("cannot borrow as mutable")
    .with_note("previous borrow occurs here")
    .with_note("mutable borrow occurs here")
    .build();
```

### 3. Always Suggest Fixes
```rust
let diag = DiagnosticBuilder::error("undefined variable")
    .with_suggestion("did you mean `count`?")
    .build();
```

### 4. Use Labels for Clarity
```rust
let diag = DiagnosticBuilder::error("type mismatch")
    .with_location("main.blz", span)
    .with_label("expected i32, found f64")  // Shows next to caret
    .build();
```

### 5. Collect Errors During Compilation
```rust
// Don't stop at first error
let mut collector = DiagnosticCollector::new();

for item in items {
    if let Err(e) = check_item(item) {
        collector.add(e);
        // Continue checking other items
    }
}

// Report all errors at once
if collector.has_errors() {
    return Err(collector);
}
```

## Common Patterns

### Pattern 1: Type Error with Conversion Suggestion
```rust
let diag = DiagnosticBuilder::error(format!(
    "mismatched types: expected `{}`, found `{}`",
    expected, found
))
.with_code("E0308")
.with_location(file, span)
.with_label(format!("expected {}", expected));

if let Some(suggestion) = suggest_type_conversion(found, expected) {
    diag = diag.with_suggestion(suggestion);
}
```

### Pattern 2: Undefined Variable with "Did You Mean"
```rust
let candidates: Vec<&str> = symbol_table.keys().collect();
let mut diag = DiagnosticBuilder::error(format!(
    "cannot find variable `{}` in this scope",
    name
))
.with_code("E0425")
.with_location(file, span);

if let Some(suggestion) = did_you_mean(name, &candidates) {
    diag = diag.with_suggestion(suggestion);
}
```

### Pattern 3: Borrow Error with Context
```rust
let diag = DiagnosticBuilder::error("cannot borrow as mutable")
    .with_code("E0502")
    .with_location(file, mutable_borrow_span)
    .with_label("mutable borrow occurs here")
    .with_note(format!(
        "immutable borrow occurs at {}:{}",
        immutable_borrow_line, immutable_borrow_col
    ));

if let Some(suggestion) = suggest_borrow_fix("mutable_and_immutable_borrow") {
    diag = diag.with_suggestion(suggestion);
}
```

## Error Code Conventions

- **E0xxx**: Lexer and parser errors
- **E1xxx**: Type errors
- **E2xxx**: Borrow checker errors
- **E3xxx**: Semantic errors
- **E4xxx**: Code generation errors
- **W0xxx**: Warnings

## Testing Your Diagnostics

```rust
#[test]
fn test_my_diagnostic() {
    let mut emitter = DiagnosticEmitter::new().with_colors(false);
    
    let diag = create_my_diagnostic();
    let output = emitter.emit(&diag);
    
    assert!(output.contains("expected error message"));
    assert!(output.contains("expected suggestion"));
}
```

## Integration with Compiler Phases

### Lexer
```rust
if invalid_token {
    return Err(Diagnostic::error("invalid token")
        .with_code("E0001")
        .with_location(file, span));
}
```

### Parser
```rust
if unexpected_token {
    let mut diag = DiagnosticBuilder::error("unexpected token")
        .with_code("E0002")
        .with_location(file, span);
    
    if let Some(suggestion) = suggest_syntax_fix("missing_semicolon") {
        diag = diag.with_suggestion(suggestion);
    }
    
    return Err(diag.build());
}
```

### Type Checker
```rust
if type_mismatch {
    let mut diag = DiagnosticBuilder::error("type mismatch")
        .with_code("E1001")
        .with_location(file, span)
        .with_label(format!("expected {}, found {}", expected, found));
    
    if let Some(suggestion) = suggest_type_conversion(found, expected) {
        diag = diag.with_suggestion(suggestion);
    }
    
    collector.add(diag.build());
}
```

### Borrow Checker
```rust
if borrow_conflict {
    let diag = DiagnosticBuilder::error("cannot borrow as mutable")
        .with_code("E2001")
        .with_location(file, span)
        .with_note("previous borrow occurs here")
        .with_suggestion(suggest_borrow_fix("multiple_mutable_borrows").unwrap())
        .build();
    
    collector.add(diag);
}
```
