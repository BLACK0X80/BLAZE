#[derive(Debug)]
pub struct Diagnostic {
    pub message: String,
}

impl Diagnostic {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}