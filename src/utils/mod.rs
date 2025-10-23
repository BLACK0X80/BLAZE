pub mod span;
pub mod source_map;

pub use span::Span;
pub use source_map::SourceMap;

use std::fmt;

pub struct DiagnosticBuilder {
    source_map: SourceMap,
}

impl DiagnosticBuilder {
    pub fn new(source_map: SourceMap) -> Self {
        Self { source_map }
    }

    pub fn error(&self, span: Span, message: String) -> Diagnostic {
        Diagnostic {
            level: DiagnosticLevel::Error,
            span,
            message,
        }
    }

    pub fn warning(&self, span: Span, message: String) -> Diagnostic {
        Diagnostic {
            level: DiagnosticLevel::Warning,
            span,
            message,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub span: Span,
    pub message: String,
}
