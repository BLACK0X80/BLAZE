use std::collections::HashMap;
use super::Span;

pub struct SourceMap {
    files: HashMap<String, SourceFile>,
}

pub struct SourceFile {
    pub name: String,
    pub content: String,
    pub line_starts: Vec<usize>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    pub fn add_file(&mut self, name: String, content: String) {
        let mut line_starts = vec![0];
        for (i, ch) in content.char_indices() {
            if ch == '\n' {
                line_starts.push(i + 1);
            }
        }

        self.files.insert(name.clone(), SourceFile {
            name,
            content,
            line_starts,
        });
    }

    pub fn get_file(&self, name: &str) -> Option<&SourceFile> {
        self.files.get(name)
    }
}
