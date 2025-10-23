// Standalone tests for the diagnostics system
// These tests verify the diagnostic formatting, source highlighting, and error collection

#[cfg(test)]
mod diagnostics_tests {
    use blaze::error::{Diagnostic, DiagnosticBuilder, DiagnosticEmitter, DiagnosticCollector, Severity};
    use blaze::utils::{Span, SourceMap};

    #[test]
    fn test_basic_diagnostic_creation() {
        let diag = Diagnostic::error("test error message");
        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.message, "test error message");
        assert!(diag.location.is_none());
        assert!(diag.suggestions.is_empty());
    }

    #[test]
    fn test_diagnostic_builder_fluent_api() {
        let diag = DiagnosticBuilder::error("type mismatch")
            .with_code("E0308")
            .with_location("example.blz", Span::new(10, 15, 2, 5))
            .with_label("expected i32, found f64")
            .with_suggestion("try using `as i32` to convert")
            .with_note("implicit conversions are not allowed")
            .build();

        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.message, "type mismatch");
        assert_eq!(diag.code, Some("E0308".to_string()));
        assert!(diag.location.is_some());
        assert_eq!(diag.suggestions.len(), 1);
        assert_eq!(diag.notes.len(), 1);
    }

    #[test]
    fn test_diagnostic_emitter_basic_formatting() {
        let mut emitter = DiagnosticEmitter::new().with_colors(false);
        
        let diag = Diagnostic::error("undefined variable `x`")
            .with_code("E0425");

        let output = emitter.emit(&diag);
        assert!(output.contains("error[E0425]: undefined variable `x`"));
    }

    #[test]
    fn test_diagnostic_with_source_context() {
        let mut source_map = SourceMap::new();
        source_map.add_file(
            "test.blz".to_string(),
            "fn main() {\n    let x = 5;\n    let y = x + \"hello\";\n}\n".to_string()
        );

        let mut emitter = DiagnosticEmitter::new()
            .with_source_map(source_map)
            .with_colors(false);

        let diag = Diagnostic::error("type mismatch: cannot add i32 and &str")
            .with_location("test.blz", Span::new(40, 47, 3, 17))
            .with_label("expected i32, found &str");

        let output = emitter.emit(&diag);
        
        // Verify the output contains the source line
        assert!(output.contains("let y = x + \"hello\""));
        // Verify it contains the caret markers
        assert!(output.contains("^"));
        // Verify it contains the error message
        assert!(output.contains("type mismatch"));
    }

    #[test]
    fn test_diagnostic_with_multiple_suggestions() {
        let diag = DiagnosticBuilder::error("cannot borrow `x` as mutable")
            .with_code("E0502")
            .with_suggestion("consider using a different variable")
            .with_suggestion("or restructure your code to avoid the conflict")
            .with_note("`x` is already borrowed as immutable")
            .build();

        assert_eq!(diag.suggestions.len(), 2);
        assert_eq!(diag.notes.len(), 1);
    }

    #[test]
    fn test_diagnostic_collector_accumulation() {
        let mut collector = DiagnosticCollector::new();
        
        collector.add(Diagnostic::error("first error"));
        collector.add(Diagnostic::warning("first warning"));
        collector.add(Diagnostic::error("second error"));
        collector.add(Diagnostic::warning("second warning"));

        assert_eq!(collector.error_count(), 2);
        assert_eq!(collector.warning_count(), 2);
        assert!(collector.has_errors());
        assert_eq!(collector.diagnostics().len(), 4);
    }

    #[test]
    fn test_diagnostic_collector_max_errors() {
        let mut collector = DiagnosticCollector::new().with_max_errors(3);
        
        // Add more errors than the limit
        for i in 0..10 {
            collector.add(Diagnostic::error(format!("error {}", i)));
        }

        assert_eq!(collector.error_count(), 10);
        // Should have limited the stored diagnostics
        assert!(collector.diagnostics().len() <= 4); // 3 errors + 1 suppression note
    }

    #[test]
    fn test_diagnostic_collector_emit_summary() {
        let mut collector = DiagnosticCollector::new();
        collector.add(Diagnostic::error("error 1"));
        collector.add(Diagnostic::error("error 2"));
        collector.add(Diagnostic::warning("warning 1"));

        let mut emitter = DiagnosticEmitter::new().with_colors(false);
        let output = collector.emit_all(&mut emitter);

        // Verify summary is included
        assert!(output.contains("2 errors"));
        assert!(output.contains("1 warning"));
    }

    #[test]
    fn test_diagnostic_collector_clear() {
        let mut collector = DiagnosticCollector::new();
        collector.add(Diagnostic::error("error"));
        collector.add(Diagnostic::warning("warning"));
        
        assert_eq!(collector.error_count(), 1);
        assert_eq!(collector.warning_count(), 1);
        
        collector.clear();
        
        assert_eq!(collector.error_count(), 0);
        assert_eq!(collector.warning_count(), 0);
        assert!(collector.diagnostics().is_empty());
    }

    #[test]
    fn test_severity_levels() {
        let error = Diagnostic::error("error");
        let warning = Diagnostic::warning("warning");
        let note = Diagnostic::note("note");

        assert_eq!(error.severity, Severity::Error);
        assert_eq!(warning.severity, Severity::Warning);
        assert_eq!(note.severity, Severity::Note);
    }

    #[test]
    fn test_diagnostic_with_replacement_suggestion() {
        let span = Span::new(10, 15, 2, 5);
        let diag = DiagnosticBuilder::error("incorrect syntax")
            .with_replacement("use `let` instead of `var`", "let x = 5", span)
            .build();

        assert_eq!(diag.suggestions.len(), 1);
        assert!(diag.suggestions[0].replacement.is_some());
        assert_eq!(diag.suggestions[0].replacement.as_ref().unwrap(), "let x = 5");
    }

    #[test]
    fn test_emitter_error_counting() {
        let mut emitter = DiagnosticEmitter::new().with_colors(false);
        
        assert_eq!(emitter.error_count(), 0);
        assert!(!emitter.has_errors());
        
        emitter.emit(&Diagnostic::error("error 1"));
        assert_eq!(emitter.error_count(), 1);
        assert!(emitter.has_errors());
        
        emitter.emit(&Diagnostic::warning("warning 1"));
        assert_eq!(emitter.error_count(), 1); // Warnings don't count as errors
        
        emitter.reset();
        assert_eq!(emitter.error_count(), 0);
    }

    #[test]
    fn test_diagnostic_location_formatting() {
        let mut emitter = DiagnosticEmitter::new().with_colors(false);
        
        let diag = Diagnostic::error("test error")
            .with_location("main.blz", Span::new(0, 5, 10, 15));

        let output = emitter.emit(&diag);
        assert!(output.contains("main.blz:10:15"));
    }

    #[test]
    fn test_multiple_notes() {
        let diag = DiagnosticBuilder::error("complex error")
            .with_note("first note explaining context")
            .with_note("second note with additional info")
            .with_note("third note with a suggestion")
            .build();

        assert_eq!(diag.notes.len(), 3);
    }

    #[test]
    fn test_emitter_max_errors_limit() {
        let mut emitter = DiagnosticEmitter::new()
            .with_colors(false)
            .with_max_errors(2);
        
        let mut output = String::new();
        
        // Emit 3 errors, but only 2 should be fully emitted
        for i in 0..3 {
            let diag = Diagnostic::error(format!("error {}", i));
            let result = emitter.emit(&diag);
            output.push_str(&result);
        }
        
        // The third error should produce empty output
        assert_eq!(emitter.error_count(), 3);
    }
}
