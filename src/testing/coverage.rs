use std::collections::{HashMap, HashSet};

pub struct CoverageAnalyzer {
    coverage_data: CoverageData,
    instrumented_points: HashMap<String, Vec<usize>>,
}

#[derive(Debug, Clone)]
pub struct CoverageData {
    pub line_coverage: HashMap<String, LineCoverage>,
    pub branch_coverage: HashMap<String, BranchCoverage>,
    pub function_coverage: HashMap<String, FunctionCoverage>,
}

#[derive(Debug, Clone)]
pub struct LineCoverage {
    pub total_lines: usize,
    pub covered_lines: HashSet<usize>,
}

#[derive(Debug, Clone)]
pub struct BranchCoverage {
    pub total_branches: usize,
    pub covered_branches: HashSet<(usize, bool)>,
}

#[derive(Debug, Clone)]
pub struct FunctionCoverage {
    pub total_functions: usize,
    pub executed_functions: HashSet<String>,
}

impl CoverageAnalyzer {
    pub fn new() -> Self {
        Self {
            coverage_data: CoverageData::new(),
            instrumented_points: HashMap::new(),
        }
    }
    
    pub fn instrument_code(&mut self, file: String, code: &str) -> String {
        let mut instrumented = String::new();
        let mut line_num = 0;
        
        for line in code.lines() {
            line_num += 1;
            
            instrumented.push_str(&format!("__coverage_hit(\"{}\", {});\n", file, line_num));
            instrumented.push_str(line);
            instrumented.push('\n');
            
            self.instrumented_points
                .entry(file.clone())
                .or_insert_with(Vec::new)
                .push(line_num);
        }
        
        instrumented
    }
    
    pub fn record_line_hit(&mut self, file: &str, line: usize) {
        self.coverage_data
            .line_coverage
            .entry(file.to_string())
            .or_insert_with(|| LineCoverage {
                total_lines: 0,
                covered_lines: HashSet::new(),
            })
            .covered_lines
            .insert(line);
    }
    
    pub fn record_branch_hit(&mut self, file: &str, branch: usize, taken: bool) {
        self.coverage_data
            .branch_coverage
            .entry(file.to_string())
            .or_insert_with(|| BranchCoverage {
                total_branches: 0,
                covered_branches: HashSet::new(),
            })
            .covered_branches
            .insert((branch, taken));
    }
    
    pub fn record_function_hit(&mut self, file: &str, function: String) {
        self.coverage_data
            .function_coverage
            .entry(file.to_string())
            .or_insert_with(|| FunctionCoverage {
                total_functions: 0,
                executed_functions: HashSet::new(),
            })
            .executed_functions
            .insert(function);
    }
    
    pub fn get_line_coverage_percentage(&self, file: &str) -> f64 {
        if let Some(coverage) = self.coverage_data.line_coverage.get(file) {
            if coverage.total_lines == 0 {
                return 0.0;
            }
            
            (coverage.covered_lines.len() as f64 / coverage.total_lines as f64) * 100.0
        } else {
            0.0
        }
    }
    
    pub fn get_branch_coverage_percentage(&self, file: &str) -> f64 {
        if let Some(coverage) = self.coverage_data.branch_coverage.get(file) {
            if coverage.total_branches == 0 {
                return 0.0;
            }
            
            (coverage.covered_branches.len() as f64 / (coverage.total_branches * 2) as f64) * 100.0
        } else {
            0.0
        }
    }
    
    pub fn get_function_coverage_percentage(&self, file: &str) -> f64 {
        if let Some(coverage) = self.coverage_data.function_coverage.get(file) {
            if coverage.total_functions == 0 {
                return 0.0;
            }
            
            (coverage.executed_functions.len() as f64 / coverage.total_functions as f64) * 100.0
        } else {
            0.0
        }
    }
    
    pub fn generate_html_report(&self, output_path: &str) -> Result<(), String> {
        let mut html = String::from("<html><head><title>Coverage Report</title></head><body>");
        html.push_str("<h1>Code Coverage Report</h1>");
        
        for (file, coverage) in &self.coverage_data.line_coverage {
            let percentage = self.get_line_coverage_percentage(file);
            
            html.push_str(&format!("<h2>{}</h2>", file));
            html.push_str(&format!("<p>Line Coverage: {:.2}%</p>", percentage));
            html.push_str(&format!("<p>Covered Lines: {} / {}</p>", 
                coverage.covered_lines.len(), coverage.total_lines));
        }
        
        html.push_str("</body></html>");
        
        std::fs::write(output_path, html)
            .map_err(|e| format!("Failed to write report: {}", e))
    }
    
    pub fn merge_coverage(&mut self, other: CoverageData) {
        for (file, coverage) in other.line_coverage {
            let entry = self.coverage_data.line_coverage
                .entry(file)
                .or_insert_with(|| LineCoverage {
                    total_lines: coverage.total_lines,
                    covered_lines: HashSet::new(),
                });
            
            entry.covered_lines.extend(coverage.covered_lines);
        }
    }
}

impl CoverageData {
    fn new() -> Self {
        Self {
            line_coverage: HashMap::new(),
            branch_coverage: HashMap::new(),
            function_coverage: HashMap::new(),
        }
    }
}

impl Default for CoverageAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
