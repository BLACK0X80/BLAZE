use crate::error::diagnostic::Diagnostic;

pub struct ErrorReporter {
    errors: Vec<Diagnostic>,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: std::io::Error) {
        self.errors.push(Diagnostic::new(error.to_string()));
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn print_all(&self) {
        for error in &self.errors {
            eprintln!("Error: {}", error.message);
        }
    }
}