use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SourceMap {
    files: HashMap<String, SourceFile>,
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub name: String,
    pub content: String,
    pub line_starts: Vec<usize>,
}

impl SourceMap {
    pub fn new() -> Self {
        SourceMap {
            files: HashMap::new(),
        }
    }

    pub fn add_file(&mut self, name: String, content: String) {
        let line_starts = Self::compute_line_starts(&content);
        let file = SourceFile {
            name: name.clone(),
            content,
            line_starts,
        };
        self.files.insert(name, file);
    }

    pub fn get_file(&self, name: &str) -> Option<&SourceFile> {
        self.files.get(name)
    }

    pub fn get_line_column(&self, file_name: &str, offset: usize) -> Option<(usize, usize)> {
        let file = self.get_file(file_name)?;
        
        let line = file.line_starts.binary_search(&offset)
            .unwrap_or_else(|x| x.saturating_sub(1));
        
        let line_start = file.line_starts.get(line).copied().unwrap_or(0);
        let column = offset - line_start;
        
        Some((line + 1, column + 1))
    }

    pub fn get_line_text(&self, file_name: &str, line: usize) -> Option<&str> {
        let file = self.get_file(file_name)?;
        
        if line == 0 || line > file.line_starts.len() {
            return None;
        }
        
        let start = file.line_starts[line - 1];
        let end = file.line_starts.get(line).copied().unwrap_or(file.content.len());
        
        Some(&file.content[start..end])
    }

    fn compute_line_starts(content: &str) -> Vec<usize> {
        let mut starts = vec![0];
        
        for (i, ch) in content.char_indices() {
            if ch == '\n' {
                starts.push(i + 1);
            }
        }
        
        starts
    }
}

