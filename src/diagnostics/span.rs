use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

#[derive(Debug, Clone)]
pub struct SpanContext {
    pub source: String,
    pub file_path: Option<String>,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
    
    pub fn single(pos: Position) -> Self {
        Self { start: pos, end: pos }
    }
    
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
    
    pub fn contains(&self, pos: Position) -> bool {
        self.start <= pos && pos <= self.end
    }
    
    pub fn overlaps(&self, other: &Span) -> bool {
        self.start <= other.end && other.start <= self.end
    }
    
    pub fn extract<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start.offset..self.end.offset]
    }
    
    pub fn line_range(&self) -> (usize, usize) {
        (self.start.line, self.end.line)
    }
    
    pub fn is_multiline(&self) -> bool {
        self.start.line != self.end.line
    }
}

impl Position {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self { line, column, offset }
    }
    
    pub fn zero() -> Self {
        Self { line: 1, column: 1, offset: 0 }
    }
    
    pub fn advance(&mut self, ch: char) {
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        self.offset += ch.len_utf8();
    }
}

impl SpanContext {
    pub fn new(source: String, file_path: Option<String>) -> Self {
        Self { source, file_path }
    }
    
    pub fn get_line(&self, line_number: usize) -> Option<&str> {
        self.source.lines().nth(line_number.saturating_sub(1))
    }
    
    pub fn get_line_range(&self, start_line: usize, end_line: usize) -> Vec<&str> {
        self.source
            .lines()
            .skip(start_line.saturating_sub(1))
            .take(end_line.saturating_sub(start_line) + 1)
            .collect()
    }
    
    pub fn format_span(&self, span: &Span) -> String {
        let mut output = String::new();
        
        if let Some(ref path) = self.file_path {
            output.push_str(&format!("{}:{}:{}\n", path, span.start.line, span.start.column));
        }
        
        let lines = self.get_line_range(span.start.line, span.end.line);
        
        for (i, line) in lines.iter().enumerate() {
            let line_num = span.start.line + i;
            output.push_str(&format!("{:5} | {}\n", line_num, line));
            
            if i == 0 && span.start.line == span.end.line {
                let padding = " ".repeat(8 + span.start.column - 1);
                let underline = "^".repeat((span.end.column - span.start.column).max(1));
                output.push_str(&format!("{}{}  \n", padding, underline));
            }
        }
        
        output
    }
    
    pub fn format_multiline_span(&self, span: &Span, message: &str) -> String {
        let mut output = String::new();
        
        if let Some(ref path) = self.file_path {
            output.push_str(&format!("\n{}:{}:{}\n", path, span.start.line, span.start.column));
        }
        
        output.push_str(&format!("\n{}\n\n", message));
        
        let context_before = 2;
        let context_after = 2;
        let start = span.start.line.saturating_sub(context_before);
        let end = (span.end.line + context_after).min(self.source.lines().count());
        
        for line_num in start..=end {
            if let Some(line_content) = self.get_line(line_num) {
                let in_span = line_num >= span.start.line && line_num <= span.end.line;
                let marker = if in_span { ">" } else { " " };
                
                output.push_str(&format!("{} {:5} | {}\n", marker, line_num, line_content));
                
                if line_num == span.start.line {
                    let padding = " ".repeat(9 + span.start.column - 1);
                    output.push_str(&format!("  {}^\n", padding));
                }
            }
        }
        
        output
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}-{}:{}", 
               self.start.line, self.start.column,
               self.end.line, self.end.column)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::zero()
    }
}
