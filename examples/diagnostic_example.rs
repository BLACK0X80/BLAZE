// Example demonstrating the diagnostic system
// This shows how to use the diagnostic system to report errors with source context

use blaze::error::{Diagnostic, DiagnosticBuilder, DiagnosticEmitter, DiagnosticCollector, Severity};
use blaze::error::{did_you_mean, suggest_type_conversion};
use blaze::utils::{Span, SourceMap};

fn main() {
    println!("=== BLAZE Diagnostic System Example ===\n");

    // Example 1: Basic error with code
    println!("Example 1: Basic Error");
    println!("----------------------");
    demonstrate_basic_error();
    println!();

    // Example 2: Error with source context
    println!("Example 2: Error with Source Context");
    println!("------------------------------------");
    demonstrate_source_context();
    println!();

    // Example 3: Error with suggestions
    println!("Example 3: Error with Suggestions");
    println!("---------------------------------");
    demonstrate_suggestions();
    println!();

    // Example 4: Multiple errors
    println!("Example 4: Multiple Errors");
    println!("-------------------------");
    demonstrate_multiple_errors();
    println!();

    // Example 5: Type mismatch with conversion suggestion
    println!("Example 5: Type Mismatch");
    println!("------------------------");
    demonstrate_type_mismatch();
    println!();
}

fn demonstrate_basic_error() {
    let mut emitter = DiagnosticEmitter::new().with_colors(false);
    
    let diag = DiagnosticBuilder::error("undefined variable `count`")
        .with_code("E0425")
        .with_note("this variable is not declared in the current scope")
        .build();

    print!("{}", emitter.emit(&diag));
}

fn demonstrate_source_context() {
    let source_code = r#"fn main() {
    let x = 5;
    let y = x + "hello";
    println("{}", y);
}
"#;

    let mut source_map = SourceMap::new();
    source_map.add_file("example.blz".to_string(), source_code.to_string());

    let mut emitter = DiagnosticEmitter::new()
        .with_source_map(source_map)
        .with_colors(false);

    let diag = DiagnosticBuilder::error("type mismatch: cannot add i32 and &str")
        .with_code("E0308")
        .with_location("example.blz", Span::new(32, 39, 3, 13))
        .with_label("expected i32, found &str")
        .with_note("the left operand has type `i32`")
        .with_note("the right operand has type `&str`")
        .build();

    print!("{}", emitter.emit(&diag));
}

fn demonstrate_suggestions() {
    let mut emitter = DiagnosticEmitter::new().with_colors(false);
    
    // Use the did_you_mean helper
    let candidates = vec!["println", "print", "format"];
    let suggestion = did_you_mean("printl", &candidates);

    let mut builder = DiagnosticBuilder::error("cannot find function `printl` in this scope")
        .with_code("E0425");

    if let Some(sug) = suggestion {
        builder = builder.with_suggestion(sug);
    }

    let diag = builder.build();
    print!("{}", emitter.emit(&diag));
}

fn demonstrate_multiple_errors() {
    let mut collector = DiagnosticCollector::new();

    // Add multiple errors
    collector.add(
        Diagnostic::error("undefined variable `x`")
            .with_code("E0425")
    );

    collector.add(
        Diagnostic::warning("unused variable `y`")
            .with_code("W0001")
            .with_suggestion("consider using `_y` if this is intentional")
    );

    collector.add(
        Diagnostic::error("type mismatch in function call")
            .with_code("E0308")
    );

    // Emit all diagnostics
    let mut emitter = DiagnosticEmitter::new().with_colors(false);
    print!("{}", collector.emit_all(&mut emitter));
}

fn demonstrate_type_mismatch() {
    let mut emitter = DiagnosticEmitter::new().with_colors(false);
    
    let from_type = "i32";
    let to_type = "f64";
    
    let mut builder = DiagnosticBuilder::error(format!(
        "mismatched types: expected `{}`, found `{}`",
        to_type, from_type
    ))
    .with_code("E0308");

    // Add type conversion suggestion if available
    if let Some(suggestion) = suggest_type_conversion(from_type, to_type) {
        builder = builder.with_suggestion(suggestion);
    }

    let diag = builder.build();
    print!("{}", emitter.emit(&diag));
}
