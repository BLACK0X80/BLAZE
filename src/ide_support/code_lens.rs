use std::collections::HashMap;

pub struct CodeLensProvider {
    lenses: Vec<CodeLens>,
}

#[derive(Debug, Clone)]
pub struct CodeLens {
    pub range: Range,
    pub command: Command,
    pub data: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Range {
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub title: String,
    pub command: String,
    pub arguments: Vec<String>,
}

impl CodeLensProvider {
    pub fn new() -> Self {
        Self {
            lenses: Vec::new(),
        }
    }
    
    pub fn provide_code_lenses(&mut self, document: &str) -> Vec<CodeLens> {
        self.lenses.clear();
        
        self.add_run_lenses(document);
        self.add_test_lenses(document);
        self.add_benchmark_lenses(document);
        self.add_reference_lenses(document);
        
        self.lenses.clone()
    }
    
    fn add_run_lenses(&mut self, document: &str) {
        for (i, line) in document.lines().enumerate() {
            if line.contains("fn main()") {
                self.lenses.push(CodeLens {
                    range: Range {
                        start_line: i,
                        start_column: 0,
                        end_line: i,
                        end_column: line.len(),
                    },
                    command: Command {
                        title: "▶ Run".to_string(),
                        command: "blaze.run".to_string(),
                        arguments: vec![],
                    },
                    data: None,
                });
                
                self.lenses.push(CodeLens {
                    range: Range {
                        start_line: i,
                        start_column: 0,
                        end_line: i,
                        end_column: line.len(),
                    },
                    command: Command {
                        title: "🐛 Debug".to_string(),
                        command: "blaze.debug".to_string(),
                        arguments: vec![],
                    },
                    data: None,
                });
            }
        }
    }
    
    fn add_test_lenses(&mut self, document: &str) {
        for (i, line) in document.lines().enumerate() {
            if line.contains("#[test]") || line.contains("fn test_") {
                self.lenses.push(CodeLens {
                    range: Range {
                        start_line: i,
                        start_column: 0,
                        end_line: i,
                        end_column: line.len(),
                    },
                    command: Command {
                        title: "▶ Run Test".to_string(),
                        command: "blaze.test".to_string(),
                        arguments: vec![],
                    },
                    data: None,
                });
            }
        }
    }
    
    fn add_benchmark_lenses(&mut self, document: &str) {
        for (i, line) in document.lines().enumerate() {
            if line.contains("#[bench]") {
                self.lenses.push(CodeLens {
                    range: Range {
                        start_line: i,
                        start_column: 0,
                        end_line: i,
                        end_column: line.len(),
                    },
                    command: Command {
                        title: "⏱ Run Benchmark".to_string(),
                        command: "blaze.benchmark".to_string(),
                        arguments: vec![],
                    },
                    data: None,
                });
            }
        }
    }
    
    fn add_reference_lenses(&mut self, document: &str) {
        for (i, line) in document.lines().enumerate() {
            if line.contains("fn ") && !line.contains("fn main") {
                self.lenses.push(CodeLens {
                    range: Range {
                        start_line: i,
                        start_column: 0,
                        end_line: i,
                        end_column: line.len(),
                    },
                    command: Command {
                        title: "0 references".to_string(),
                        command: "blaze.showReferences".to_string(),
                        arguments: vec![],
                    },
                    data: Some("function".to_string()),
                });
            }
        }
    }
}

impl Default for CodeLensProvider {
    fn default() -> Self {
        Self::new()
    }
}
