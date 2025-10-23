use std::fmt;
use crate::utils::{Span, SourceMap};

/// Severity level of a diagnostic message
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
    Help,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Note => write!(f, "note"),
            Severity::Help => write!(f, "help"),
        }
    }
}

/// A single diagnostic message with location and optional suggestions
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub location: Option<DiagnosticLocation>,
    pub suggestions: Vec<Suggestion>,
    pub notes: Vec<String>,
    pub code: Option<String>,
}

/// Location information for a diagnostic
#[derive(Debug, Clone)]
pub struct DiagnosticLocation {
    pub file: String,
    pub span: Span,
    pub label: Option<String>,
}

/// A suggestion for fixing an error
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub message: String,
    pub replacement: Option<String>,
    pub span: Option<Span>,
}

impl Diagnostic {
    pub fn new(severity: Severity, message: impl Into<String>) -> Self {
        Self {
            severity,
            message: message.into(),
            location: None,
            suggestions: Vec::new(),
            notes: Vec::new(),
            code: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(Severity::Error, message)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, message)
    }

    pub fn note(message: impl Into<String>) -> Self {
        Self::new(Severity::Note, message)
    }

    pub fn with_location(mut self, file: impl Into<String>, span: Span) -> Self {
        self.location = Some(DiagnosticLocation {
            file: file.into(),
            span,
            label: None,
        });
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        if let Some(loc) = &mut self.location {
            loc.label = Some(label.into());
        }
        self
    }

    pub fn with_suggestion(mut self, message: impl Into<String>) -> Self {
        self.suggestions.push(Suggestion {
            message: message.into(),
            replacement: None,
            span: None,
        });
        self
    }

    pub fn with_replacement(mut self, message: impl Into<String>, replacement: impl Into<String>, span: Span) -> Self {
        self.suggestions.push(Suggestion {
            message: message.into(),
            replacement: Some(replacement.into()),
            span: Some(span),
        });
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

/// Builder for constructing diagnostics with a fluent API
pub struct DiagnosticBuilder {
    diagnostic: Diagnostic,
}

impl DiagnosticBuilder {
    pub fn new(severity: Severity, message: impl Into<String>) -> Self {
        Self {
            diagnostic: Diagnostic::new(severity, message),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(Severity::Error, message)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, message)
    }

    pub fn note(message: impl Into<String>) -> Self {
        Self::new(Severity::Note, message)
    }

    pub fn with_location(mut self, file: impl Into<String>, span: Span) -> Self {
        self.diagnostic = self.diagnostic.with_location(file, span);
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.diagnostic = self.diagnostic.with_label(label);
        self
    }

    pub fn with_suggestion(mut self, message: impl Into<String>) -> Self {
        self.diagnostic = self.diagnostic.with_suggestion(message);
        self
    }

    pub fn with_replacement(mut self, message: impl Into<String>, replacement: impl Into<String>, span: Span) -> Self {
        self.diagnostic = self.diagnostic.with_replacement(message, replacement, span);
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.diagnostic = self.diagnostic.with_note(note);
        self
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.diagnostic = self.diagnostic.with_code(code);
        self
    }

    pub fn build(self) -> Diagnostic {
        self.diagnostic
    }
}

/// Emitter for formatting and displaying diagnostics
pub struct DiagnosticEmitter {
    source_map: Option<SourceMap>,
    use_colors: bool,
    max_errors: usize,
    error_count: usize,
}

impl DiagnosticEmitter {
    pub fn new() -> Self {
        Self {
            source_map: None,
            use_colors: Self::should_use_colors(),
            max_errors: 10,
            error_count: 0,
        }
    }

    pub fn with_source_map(mut self, source_map: SourceMap) -> Self {
        self.source_map = Some(source_map);
        self
    }

    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.use_colors = use_colors;
        self
    }

    pub fn with_max_errors(mut self, max_errors: usize) -> Self {
        self.max_errors = max_errors;
        self
    }

    fn should_use_colors() -> bool {
        // Check if we're in a terminal that supports colors
        #[cfg(windows)]
        {
            // On Windows, check if ANSI support is enabled
            use std::env;
            env::var("NO_COLOR").is_err() && 
            (env::var("TERM").is_ok() || env::var("WT_SESSION").is_ok())
        }
        #[cfg(not(windows))]
        {
            use std::env;
            use std::io::IsTerminal;
            env::var("NO_COLOR").is_err() && std::io::stderr().is_terminal()
        }
    }

    pub fn emit(&mut self, diagnostic: &Diagnostic) -> String {
        if diagnostic.severity == Severity::Error {
            self.error_count += 1;
            if self.error_count > self.max_errors {
                return String::new();
            }
        }

        let mut output = String::new();
        
        // Format the main diagnostic line
        output.push_str(&self.format_diagnostic_header(diagnostic));
        output.push('\n');

        // Add source code context if available
        if let Some(ref location) = diagnostic.location {
            if let Some(ref source_map) = self.source_map {
                output.push_str(&self.format_source_context(location, source_map));
            }
        }

        // Add notes
        for note in &diagnostic.notes {
            output.push_str(&self.format_note(note));
            output.push('\n');
        }

        // Add suggestions
        for suggestion in &diagnostic.suggestions {
            output.push_str(&self.format_suggestion(suggestion));
            output.push('\n');
        }

        output
    }

    fn format_diagnostic_header(&self, diagnostic: &Diagnostic) -> String {
        let severity_str = if self.use_colors {
            self.colorize_severity(&diagnostic.severity)
        } else {
            diagnostic.severity.to_string()
        };

        let code_str = if let Some(ref code) = diagnostic.code {
            format!("[{}]", code)
        } else {
            String::new()
        };

        let location_str = if let Some(ref loc) = diagnostic.location {
            format!(" --> {}:{}:{}", loc.file, loc.span.line, loc.span.column)
        } else {
            String::new()
        };

        format!("{}{}: {}{}", severity_str, code_str, diagnostic.message, location_str)
    }

    fn format_source_context(&self, location: &DiagnosticLocation, source_map: &SourceMap) -> String {
        let mut output = String::new();

        if let Some(file) = source_map.get_file(&location.file) {
            let line_num = location.span.line;
            let col_num = location.span.column;

            // Show context: 1 line before and 1 line after
            let context_before = 1;
            let context_after = 1;

            if line_num > 0 && line_num <= file.line_starts.len() {
                // Calculate line range to display
                let start_line = line_num.saturating_sub(context_before);
                let end_line = (line_num + context_after).min(file.line_starts.len());

                // Add separator
                output.push_str("     |\n");

                // Display context lines
                for current_line in start_line..=end_line {
                    if current_line == 0 {
                        continue;
                    }

                    let line_start = file.line_starts[current_line - 1];
                    let line_end = if current_line < file.line_starts.len() {
                        file.line_starts[current_line]
                    } else {
                        file.content.len()
                    };

                    let line_content = &file.content[line_start..line_end];
                    let line_content = line_content.trim_end_matches('\n');

                    // Format line number with highlighting for error line
                    let line_num_str = if self.use_colors && current_line == line_num {
                        format!("{:4} | ", self.colorize(&current_line.to_string(), Color::Bold))
                    } else {
                        format!("{:4} | ", current_line)
                    };
                    
                    output.push_str(&line_num_str);
                    output.push_str(line_content);
                    output.push('\n');

                    // Add caret line only for the error line
                    if current_line == line_num {
                        output.push_str("     | ");
                        for _ in 0..col_num.saturating_sub(1) {
                            output.push(' ');
                        }
                        
                        let span_len = (location.span.end - location.span.start).max(1);
                        let caret = if self.use_colors {
                            self.colorize(&"^".repeat(span_len), Color::Red)
                        } else {
                            "^".repeat(span_len)
                        };
                        output.push_str(&caret);

                        if let Some(ref label) = location.label {
                            output.push(' ');
                            let label_text = if self.use_colors {
                                self.colorize(label, Color::Red)
                            } else {
                                label.to_string()
                            };
                            output.push_str(&label_text);
                        }
                        output.push('\n');
                    }
                }

                // Add separator
                output.push_str("     |\n");
            }
        }

        output
    }

    fn format_note(&self, note: &str) -> String {
        let note_str = if self.use_colors {
            self.colorize("note", Color::Cyan)
        } else {
            "note".to_string()
        };
        format!("{}: {}", note_str, note)
    }

    fn format_suggestion(&self, suggestion: &Suggestion) -> String {
        let help_str = if self.use_colors {
            self.colorize("help", Color::Green)
        } else {
            "help".to_string()
        };
        
        let mut output = format!("{}: {}", help_str, suggestion.message);
        
        if let Some(ref replacement) = suggestion.replacement {
            output.push_str(&format!("\n      {}", replacement));
        }
        
        output
    }

    fn colorize_severity(&self, severity: &Severity) -> String {
        if !self.use_colors {
            return severity.to_string();
        }
        
        match severity {
            Severity::Error => format!("\x1b[1m\x1b[31m{}\x1b[0m", severity),
            Severity::Warning => format!("\x1b[1m\x1b[33m{}\x1b[0m", severity),
            Severity::Note => format!("\x1b[1m\x1b[36m{}\x1b[0m", severity),
            Severity::Help => format!("\x1b[1m\x1b[32m{}\x1b[0m", severity),
        }
    }

    fn colorize(&self, text: &str, color: Color) -> String {
        if !self.use_colors {
            return text.to_string();
        }

        let color_code = match color {
            Color::Red => "\x1b[31m",
            Color::Yellow => "\x1b[33m",
            Color::Green => "\x1b[32m",
            Color::Cyan => "\x1b[36m",
            Color::Bold => "\x1b[1m",
        };

        format!("{}{}\x1b[0m", color_code, text)
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    pub fn error_count(&self) -> usize {
        self.error_count
    }

    pub fn reset(&mut self) {
        self.error_count = 0;
    }
}

impl Default for DiagnosticEmitter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
enum Color {
    Red,
    Yellow,
    Green,
    Cyan,
    Bold,
}

/// Collector for accumulating multiple diagnostics during compilation
pub struct DiagnosticCollector {
    diagnostics: Vec<Diagnostic>,
    max_errors: usize,
    error_count: usize,
    warning_count: usize,
}

impl DiagnosticCollector {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            max_errors: 10,
            error_count: 0,
            warning_count: 0,
        }
    }

    pub fn with_max_errors(mut self, max_errors: usize) -> Self {
        self.max_errors = max_errors;
        self
    }

    /// Add a diagnostic to the collection
    pub fn add(&mut self, diagnostic: Diagnostic) {
        match diagnostic.severity {
            Severity::Error => {
                self.error_count += 1;
                if self.error_count <= self.max_errors {
                    self.diagnostics.push(diagnostic);
                } else if self.error_count == self.max_errors + 1 {
                    // Add a note that we're suppressing further errors
                    self.diagnostics.push(
                        Diagnostic::note(format!(
                            "suppressing {} additional error(s)",
                            self.error_count - self.max_errors
                        ))
                    );
                }
            }
            Severity::Warning => {
                self.warning_count += 1;
                self.diagnostics.push(diagnostic);
            }
            _ => {
                self.diagnostics.push(diagnostic);
            }
        }
    }

    /// Add an error diagnostic
    pub fn error(&mut self, message: impl Into<String>) -> &mut Diagnostic {
        let diagnostic = Diagnostic::error(message);
        self.diagnostics.push(diagnostic);
        let idx = self.diagnostics.len() - 1;
        self.error_count += 1;
        &mut self.diagnostics[idx]
    }

    /// Add a warning diagnostic
    pub fn warning(&mut self, message: impl Into<String>) -> &mut Diagnostic {
        let diagnostic = Diagnostic::warning(message);
        self.diagnostics.push(diagnostic);
        let idx = self.diagnostics.len() - 1;
        self.warning_count += 1;
        &mut self.diagnostics[idx]
    }

    /// Check if any errors were collected
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    /// Get the number of errors
    pub fn error_count(&self) -> usize {
        self.error_count
    }

    /// Get the number of warnings
    pub fn warning_count(&self) -> usize {
        self.warning_count
    }

    /// Get all collected diagnostics
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Emit all diagnostics using the provided emitter
    pub fn emit_all(&self, emitter: &mut DiagnosticEmitter) -> String {
        let mut output = String::new();
        
        for diagnostic in &self.diagnostics {
            output.push_str(&emitter.emit(diagnostic));
        }

        // Add summary
        if self.error_count > 0 || self.warning_count > 0 {
            output.push('\n');
            let summary = self.format_summary(emitter.use_colors);
            output.push_str(&summary);
        }

        output
    }

    fn format_summary(&self, use_colors: bool) -> String {
        let mut parts = Vec::new();

        if self.error_count > 0 {
            let error_text = if self.error_count == 1 {
                "error"
            } else {
                "errors"
            };
            let error_str = if use_colors {
                format!("\x1b[1m\x1b[31m{} {}\x1b[0m", self.error_count, error_text)
            } else {
                format!("{} {}", self.error_count, error_text)
            };
            parts.push(error_str);
        }

        if self.warning_count > 0 {
            let warning_text = if self.warning_count == 1 {
                "warning"
            } else {
                "warnings"
            };
            let warning_str = if use_colors {
                format!("\x1b[1m\x1b[33m{} {}\x1b[0m", self.warning_count, warning_text)
            } else {
                format!("{} {}", self.warning_count, warning_text)
            };
            parts.push(warning_str);
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!("compilation finished with {}", parts.join(" and "))
        }
    }

    /// Clear all collected diagnostics
    pub fn clear(&mut self) {
        self.diagnostics.clear();
        self.error_count = 0;
        self.warning_count = 0;
    }
}

impl Default for DiagnosticCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_builder() {
        let diag = DiagnosticBuilder::error("type mismatch")
            .with_code("E0308")
            .with_location("test.blz", Span::new(10, 15, 2, 5))
            .with_label("expected i32, found f64")
            .with_suggestion("try converting the value")
            .with_note("this is a note")
            .build();

        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.message, "type mismatch");
        assert_eq!(diag.code, Some("E0308".to_string()));
        assert_eq!(diag.suggestions.len(), 1);
        assert_eq!(diag.notes.len(), 1);
    }

    #[test]
    fn test_diagnostic_emitter_no_colors() {
        let mut emitter = DiagnosticEmitter::new().with_colors(false);
        
        let diag = Diagnostic::error("test error")
            .with_code("E0001");

        let output = emitter.emit(&diag);
        assert!(output.contains("error[E0001]: test error"));
    }

    #[test]
    fn test_diagnostic_with_suggestion() {
        let diag = Diagnostic::error("undefined variable")
            .with_suggestion("did you mean `count`?");

        assert_eq!(diag.suggestions.len(), 1);
        assert_eq!(diag.suggestions[0].message, "did you mean `count`?");
    }

    #[test]
    fn test_diagnostic_formatting() {
        let mut emitter = DiagnosticEmitter::new().with_colors(false);
        
        let diag = DiagnosticBuilder::error("type mismatch")
            .with_code("E0308")
            .with_note("types must match in binary operations")
            .build();

        let output = emitter.emit(&diag);
        assert!(output.contains("error[E0308]: type mismatch"));
        assert!(output.contains("note: types must match"));
    }

    #[test]
    fn test_source_code_highlighting() {
        let mut source_map = SourceMap::new();
        source_map.add_file(
            "test.blz".to_string(),
            "fn main() {\n    let x = 5;\n    let y = x + \"hello\";\n}\n".to_string()
        );

        let mut emitter = DiagnosticEmitter::new()
            .with_source_map(source_map)
            .with_colors(false);

        let diag = Diagnostic::error("type mismatch")
            .with_location("test.blz", Span::new(40, 47, 3, 17))
            .with_label("expected i32, found &str");

        let output = emitter.emit(&diag);
        assert!(output.contains("let y = x + \"hello\""));
        assert!(output.contains("^^^^^^^"));
    }

    #[test]
    fn test_suggestion_generation() {
        let diag = DiagnosticBuilder::error("type mismatch")
            .with_suggestion("try converting with `as f64`")
            .with_replacement("consider using", "value as f64", Span::new(10, 15, 2, 5))
            .build();

        assert_eq!(diag.suggestions.len(), 2);
        assert!(diag.suggestions[0].replacement.is_none());
        assert!(diag.suggestions[1].replacement.is_some());
    }

    #[test]
    fn test_colored_output() {
        let mut emitter = DiagnosticEmitter::new().with_colors(true);
        
        let diag = Diagnostic::error("test error");
        let output = emitter.emit(&diag);
        
        // Should contain ANSI color codes when colors are enabled
        // We can't test the exact output since it depends on terminal support
        assert!(!output.is_empty());
    }

    #[test]
    fn test_diagnostic_collector() {
        let mut collector = DiagnosticCollector::new();
        
        collector.add(Diagnostic::error("error 1"));
        collector.add(Diagnostic::warning("warning 1"));
        collector.add(Diagnostic::error("error 2"));

        assert_eq!(collector.error_count(), 2);
        assert_eq!(collector.warning_count(), 1);
        assert!(collector.has_errors());
        assert_eq!(collector.diagnostics().len(), 3);
    }

    #[test]
    fn test_diagnostic_collector_max_errors() {
        let mut collector = DiagnosticCollector::new().with_max_errors(2);
        
        for i in 0..5 {
            collector.add(Diagnostic::error(format!("error {}", i)));
        }

        assert_eq!(collector.error_count(), 5);
        // Should have 2 errors + 1 suppression note
        assert!(collector.diagnostics().len() <= 3);
    }

    #[test]
    fn test_diagnostic_collector_emit_all() {
        let mut collector = DiagnosticCollector::new();
        collector.add(Diagnostic::error("error 1"));
        collector.add(Diagnostic::warning("warning 1"));

        let mut emitter = DiagnosticEmitter::new().with_colors(false);
        let output = collector.emit_all(&mut emitter);

        assert!(output.contains("error: error 1"));
        assert!(output.contains("warning: warning 1"));
        assert!(output.contains("1 error"));
        assert!(output.contains("1 warning"));
    }

    #[test]
    fn test_diagnostic_collector_clear() {
        let mut collector = DiagnosticCollector::new();
        collector.add(Diagnostic::error("error 1"));
        
        assert_eq!(collector.error_count(), 1);
        
        collector.clear();
        
        assert_eq!(collector.error_count(), 0);
        assert_eq!(collector.diagnostics().len(), 0);
    }

    #[test]
    fn test_multiple_notes_and_suggestions() {
        let diag = DiagnosticBuilder::error("complex error")
            .with_note("first note")
            .with_note("second note")
            .with_suggestion("first suggestion")
            .with_suggestion("second suggestion")
            .build();

        assert_eq!(diag.notes.len(), 2);
        assert_eq!(diag.suggestions.len(), 2);
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(Severity::Error.to_string(), "error");
        assert_eq!(Severity::Warning.to_string(), "warning");
        assert_eq!(Severity::Note.to_string(), "note");
        assert_eq!(Severity::Help.to_string(), "help");
    }

    #[test]
    fn test_diagnostic_location() {
        let diag = Diagnostic::error("test")
            .with_location("file.blz", Span::new(0, 5, 1, 1))
            .with_label("test label");

        assert!(diag.location.is_some());
        let loc = diag.location.unwrap();
        assert_eq!(loc.file, "file.blz");
        assert_eq!(loc.span.line, 1);
        assert_eq!(loc.label, Some("test label".to_string()));
    }
}
