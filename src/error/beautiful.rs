use colored::*;
use std::fmt;

pub struct BeautifulError {
    pub error_code: String,
    pub message: String,
    pub file_path: String,
    pub line: usize,
    pub column: usize,
    pub source_line: String,
    pub help: Option<String>,
    pub suggestion: Option<Suggestion>,
}

pub struct Suggestion {
    pub message: String,
    pub replacement: String,
}

impl BeautifulError {
    pub fn new(
        error_code: impl Into<String>,
        message: impl Into<String>,
        file_path: impl Into<String>,
        line: usize,
        column: usize,
        source_line: impl Into<String>,
    ) -> Self {
        Self {
            error_code: error_code.into(),
            message: message.into(),
            file_path: file_path.into(),
            line,
            column,
            source_line: source_line.into(),
            help: None,
            suggestion: None,
        }
    }
    
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
    
    pub fn with_suggestion(mut self, message: impl Into<String>, replacement: impl Into<String>) -> Self {
        self.suggestion = Some(Suggestion {
            message: message.into(),
            replacement: replacement.into(),
        });
        self
    }
}

impl fmt::Display for BeautifulError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}{} {}", 
            "error".red().bold(),
            format!("[{}]", self.error_code).white().bold(),
            self.message.white().bold()
        )?;
        
        writeln!(f, "  {} {}:{}:{}",
            "-->".blue().bold(),
            self.file_path,
            self.line,
            self.column
        )?;
        
        writeln!(f, "   {}", "|".blue().bold())?;
        
        writeln!(f, "{:>3} {} {}",
            self.line.to_string().blue().bold(),
            "|".blue().bold(),
            self.source_line
        )?;
        
        let spaces = " ".repeat(self.column);
        writeln!(f, "   {} {}{}",
            "|".blue().bold(),
            spaces,
            "^".repeat(self.source_line.len().min(10)).red().bold()
        )?;
        
        if let Some(help) = &self.help {
            writeln!(f)?;
            writeln!(f, "   {} {}",
                "help:".cyan().bold(),
                help
            )?;
        }
        
        if let Some(suggestion) = &self.suggestion {
            writeln!(f)?;
            writeln!(f, "   {} {}",
                "suggestion:".green().bold(),
                suggestion.message
            )?;
            writeln!(f)?;
            writeln!(f, "{:>3} {} {}",
                self.line.to_string().blue().bold(),
                "|".blue().bold(),
                suggestion.replacement.green()
            )?;
        }
        
        Ok(())
    }
}

pub fn type_mismatch_error(
    file_path: &str,
    line: usize,
    column: usize,
    source_line: &str,
    expected: &str,
    found: &str,
) -> BeautifulError {
    BeautifulError::new(
        "E0308",
        format!("mismatched types: expected `{}`, found `{}`", expected, found),
        file_path,
        line,
        column,
        source_line,
    )
    .with_help(format!("consider converting the value to `{}`", expected))
}

pub fn undefined_variable_error(
    file_path: &str,
    line: usize,
    column: usize,
    source_line: &str,
    var_name: &str,
) -> BeautifulError {
    let mut error = BeautifulError::new(
        "E0425",
        format!("cannot find value `{}` in this scope", var_name),
        file_path,
        line,
        column,
        source_line,
    );
    
    if let Some(suggestion) = find_similar_variable(var_name) {
        error = error.with_suggestion(
            format!("did you mean `{}`?", suggestion),
            source_line.replace(var_name, &suggestion),
        );
    }
    
    error
}

pub fn borrow_checker_error(
    file_path: &str,
    line: usize,
    column: usize,
    source_line: &str,
    message: &str,
) -> BeautifulError {
    BeautifulError::new(
        "E0502",
        message,
        file_path,
        line,
        column,
        source_line,
    )
    .with_help("consider using a reference or cloning the value")
}

pub fn lifetime_error(
    file_path: &str,
    line: usize,
    column: usize,
    source_line: &str,
    message: &str,
) -> BeautifulError {
    BeautifulError::new(
        "E0597",
        message,
        file_path,
        line,
        column,
        source_line,
    )
    .with_help("add explicit lifetime annotations")
}

pub fn pattern_not_exhaustive_error(
    file_path: &str,
    line: usize,
    column: usize,
    source_line: &str,
    missing_patterns: &[String],
) -> BeautifulError {
    BeautifulError::new(
        "E0004",
        "non-exhaustive patterns",
        file_path,
        line,
        column,
        source_line,
    )
    .with_help(format!(
        "missing patterns: {}",
        missing_patterns.join(", ")
    ))
}

pub fn trait_not_implemented_error(
    file_path: &str,
    line: usize,
    column: usize,
    source_line: &str,
    trait_name: &str,
    type_name: &str,
) -> BeautifulError {
    BeautifulError::new(
        "E0277",
        format!("the trait `{}` is not implemented for `{}`", trait_name, type_name),
        file_path,
        line,
        column,
        source_line,
    )
    .with_help(format!("consider implementing `{}` for `{}`", trait_name, type_name))
}

pub fn async_await_error(
    file_path: &str,
    line: usize,
    column: usize,
    source_line: &str,
) -> BeautifulError {
    BeautifulError::new(
        "E0728",
        "`await` is only allowed inside `async` functions and blocks",
        file_path,
        line,
        column,
        source_line,
    )
    .with_help("consider making the enclosing function `async`")
    .with_suggestion(
        "add `async` keyword",
        source_line.replace("fn ", "async fn "),
    )
}

fn find_similar_variable(name: &str) -> Option<String> {
    let known_variables = vec![
        "self", "Self", "super", "crate",
        "Some", "None", "Ok", "Err",
        "String", "Vec", "Option", "Result",
    ];
    
    for var in known_variables {
        if levenshtein_distance(name, var) <= 2 {
            return Some(var.to_string());
        }
    }
    
    None
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let len_a = a.len();
    let len_b = b.len();
    
    if len_a == 0 { return len_b; }
    if len_b == 0 { return len_a; }
    
    let mut matrix = vec![vec![0; len_b + 1]; len_a + 1];
    
    for i in 0..=len_a {
        matrix[i][0] = i;
    }
    for j in 0..=len_b {
        matrix[0][j] = j;
    }
    
    for (i, ca) in a.chars().enumerate() {
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            matrix[i + 1][j + 1] = std::cmp::min(
                std::cmp::min(
                    matrix[i][j + 1] + 1,
                    matrix[i + 1][j] + 1
                ),
                matrix[i][j] + cost
            );
        }
    }
    
    matrix[len_a][len_b]
}

pub struct MultiError {
    errors: Vec<BeautifulError>,
}

impl MultiError {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }
    
    pub fn add(&mut self, error: BeautifulError) {
        self.errors.push(error);
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
    
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
}

impl fmt::Display for MultiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in &self.errors {
            writeln!(f, "{}", error)?;
            writeln!(f)?;
        }
        
        if self.errors.len() > 1 {
            writeln!(f, "{}: compilation failed due to {} errors",
                "error".red().bold(),
                self.errors.len()
            )?;
        }
        
        Ok(())
    }
}

impl Default for MultiError {
    fn default() -> Self {
        Self::new()
    }
}
