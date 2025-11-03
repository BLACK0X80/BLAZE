use crate::parser::Program;

pub struct CodeFormatter {
    config: FormatterConfig,
    indent_level: usize,
}

#[derive(Debug, Clone)]
pub struct FormatterConfig {
    pub indent_size: usize,
    pub use_tabs: bool,
    pub max_line_length: usize,
    pub brace_style: BraceStyle,
    pub spaces_around_operators: bool,
    pub trailing_comma: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BraceStyle {
    SameLine,
    NextLine,
    NextLineIndented,
}

impl CodeFormatter {
    pub fn new(config: FormatterConfig) -> Self {
        Self {
            config,
            indent_level: 0,
        }
    }
    
    pub fn format_program(&mut self, program: &Program) -> String {
        let mut output = String::new();
        
        for item in &program.items {
            output.push_str(&self.format_item(item));
            output.push('\n');
        }
        
        output
    }
    
    fn format_item(&mut self, item: &crate::parser::Item) -> String {
        match item {
            crate::parser::Item::Function(func) => self.format_function(func),
            crate::parser::Item::Struct(s) => self.format_struct(s),
        }
    }
    
    fn format_function(&mut self, func: &crate::parser::Function) -> String {
        let mut output = String::new();
        
        output.push_str("fn ");
        output.push_str(&func.name);
        output.push('(');
        
        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                output.push_str(", ");
            }
            output.push_str(&param.name);
            output.push_str(": ");
            output.push_str(&format!("{:?}", param.ty));
        }
        
        output.push(')');
        
        if let Some(ret_type) = &func.return_type {
            output.push_str(" -> ");
            output.push_str(&format!("{:?}", ret_type));
        }
        
        output.push(' ');
        
        match self.config.brace_style {
            BraceStyle::SameLine => output.push('{'),
            BraceStyle::NextLine => {
                output.push('\n');
                output.push_str(&self.indent());
                output.push('{');
            }
            BraceStyle::NextLineIndented => {
                output.push('\n');
                self.indent_level += 1;
                output.push_str(&self.indent());
                output.push('{');
                self.indent_level -= 1;
            }
        }
        
        output.push('\n');
        
        self.indent_level += 1;
        for stmt in &func.body {
            output.push_str(&self.indent());
            output.push_str(&self.format_statement(stmt));
            output.push('\n');
        }
        self.indent_level -= 1;
        
        output.push_str(&self.indent());
        output.push('}');
        
        output
    }
    
    fn format_struct(&mut self, s: &crate::parser::Struct) -> String {
        let mut output = String::new();
        
        output.push_str("struct ");
        output.push_str(&s.name);
        output.push_str(" {\n");
        
        self.indent_level += 1;
        for field in &s.fields {
            output.push_str(&self.indent());
            output.push_str(&field.name);
            output.push_str(": ");
            output.push_str(&format!("{:?}", field.ty));
            output.push_str(",\n");
        }
        self.indent_level -= 1;
        
        output.push_str(&self.indent());
        output.push('}');
        
        output
    }
    
    fn format_statement(&mut self, stmt: &crate::parser::Statement) -> String {
        match stmt {
            crate::parser::Statement::Let { name, mutable, ty, value } => {
                let mut output = String::from("let ");
                if *mutable {
                    output.push_str("mut ");
                }
                output.push_str(name);
                
                if let Some(t) = ty {
                    output.push_str(": ");
                    output.push_str(&format!("{:?}", t));
                }
                
                if let Some(v) = value {
                    output.push_str(" = ");
                    output.push_str(&self.format_expression(v));
                }
                
                output.push(';');
                output
            }
            
            crate::parser::Statement::Return(expr) => {
                let mut output = String::from("return");
                if let Some(e) = expr {
                    output.push(' ');
                    output.push_str(&self.format_expression(e));
                }
                output.push(';');
                output
            }
            
            _ => String::from("/* statement */"),
        }
    }
    
    fn format_expression(&self, expr: &crate::parser::Expression) -> String {
        match expr {
            crate::parser::Expression::IntLit(n) => n.to_string(),
            crate::parser::Expression::StringLit(s) => format!("\"{}\"", s),
            crate::parser::Expression::BoolLit(b) => b.to_string(),
            crate::parser::Expression::Ident(name) | crate::parser::Expression::Identifier(name) => name.clone(),
            _ => String::from("expr"),
        }
    }
    
    fn indent(&self) -> String {
        if self.config.use_tabs {
            "\t".repeat(self.indent_level)
        } else {
            " ".repeat(self.indent_level * self.config.indent_size)
        }
    }
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            indent_size: 4,
            use_tabs: false,
            max_line_length: 100,
            brace_style: BraceStyle::SameLine,
            spaces_around_operators: true,
            trailing_comma: true,
        }
    }
}

impl Default for CodeFormatter {
    fn default() -> Self {
        Self::new(FormatterConfig::default())
    }
}
