use crate::diagnostics::{Span, SpanContext};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportLevel {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone)]
pub struct Report {
    pub level: ReportLevel,
    pub message: String,
    pub span: Option<Span>,
    pub notes: Vec<String>,
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub message: String,
    pub span: Span,
    pub replacement: Option<String>,
}

pub struct DiagnosticReporter {
    reports: Vec<Report>,
    max_errors: usize,
    error_count: usize,
    warning_count: usize,
}

impl DiagnosticReporter {
    pub fn new() -> Self {
        Self {
            reports: Vec::new(),
            max_errors: 100,
            error_count: 0,
            warning_count: 0,
        }
    }
    
    pub fn error(&mut self, message: String, span: Option<Span>) -> &mut Report {
        self.error_count += 1;
        self.add_report(Report {
            level: ReportLevel::Error,
            message,
            span,
            notes: Vec::new(),
            suggestions: Vec::new(),
        })
    }
    
    pub fn warning(&mut self, message: String, span: Option<Span>) -> &mut Report {
        self.warning_count += 1;
        self.add_report(Report {
            level: ReportLevel::Warning,
            message,
            span,
            notes: Vec::new(),
            suggestions: Vec::new(),
        })
    }
    
    pub fn info(&mut self, message: String, span: Option<Span>) -> &mut Report {
        self.add_report(Report {
            level: ReportLevel::Info,
            message,
            span,
            notes: Vec::new(),
            suggestions: Vec::new(),
        })
    }
    
    pub fn hint(&mut self, message: String, span: Option<Span>) -> &mut Report {
        self.add_report(Report {
            level: ReportLevel::Hint,
            message,
            span,
            notes: Vec::new(),
            suggestions: Vec::new(),
        })
    }
    
    fn add_report(&mut self, report: Report) -> &mut Report {
        self.reports.push(report);
        self.reports.last_mut().unwrap()
    }
    
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }
    
    pub fn error_count(&self) -> usize {
        self.error_count
    }
    
    pub fn warning_count(&self) -> usize {
        self.warning_count
    }
    
    pub fn should_abort(&self) -> bool {
        self.error_count >= self.max_errors
    }
    
    pub fn print_all(&self, context: &SpanContext) {
        for report in &self.reports {
            report.print(context);
        }
        
        self.print_summary();
    }
    
    pub fn print_summary(&self) {
        if self.error_count > 0 || self.warning_count > 0 {
            println!("\nCompilation finished with {} error(s) and {} warning(s)", 
                     self.error_count, self.warning_count);
        }
    }
    
    pub fn clear(&mut self) {
        self.reports.clear();
        self.error_count = 0;
        self.warning_count = 0;
    }
    
    pub fn get_reports(&self) -> &[Report] {
        &self.reports
    }
}

impl Report {
    pub fn with_note(mut self, note: String) -> Self {
        self.notes.push(note);
        self
    }
    
    pub fn with_suggestion(mut self, message: String, span: Span, replacement: Option<String>) -> Self {
        self.suggestions.push(Suggestion {
            message,
            span,
            replacement,
        });
        self
    }
    
    pub fn print(&self, context: &SpanContext) {
        let level_str = match self.level {
            ReportLevel::Error => "error",
            ReportLevel::Warning => "warning",
            ReportLevel::Info => "info",
            ReportLevel::Hint => "hint",
        };
        
        println!("\n{}: {}", level_str, self.message);
        
        if let Some(span) = self.span {
            println!("{}", context.format_span(&span));
        }
        
        for note in &self.notes {
            println!("note: {}", note);
        }
        
        for suggestion in &self.suggestions {
            println!("help: {}", suggestion.message);
            if let Some(ref replacement) = suggestion.replacement {
                println!("  suggestion: {}", replacement);
            }
        }
    }
}

impl Default for DiagnosticReporter {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ReportLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ReportLevel::Error => write!(f, "error"),
            ReportLevel::Warning => write!(f, "warning"),
            ReportLevel::Info => write!(f, "info"),
            ReportLevel::Hint => write!(f, "hint"),
        }
    }
}
