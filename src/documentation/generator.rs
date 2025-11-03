use std::collections::HashMap;
use crate::parser::{Program, Function, Struct};

pub struct DocumentationGenerator {
    format: DocFormat,
    output_dir: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DocFormat {
    HTML,
    Markdown,
    JSON,
    PDF,
}

impl DocumentationGenerator {
    pub fn new(format: DocFormat, output_dir: String) -> Self {
        Self {
            format,
            output_dir,
        }
    }
    
    pub fn generate(&self, program: &Program) -> Result<String, String> {
        match self.format {
            DocFormat::HTML => self.generate_html(program),
            DocFormat::Markdown => self.generate_markdown(program),
            DocFormat::JSON => self.generate_json(program),
            DocFormat::PDF => self.generate_pdf(program),
        }
    }
    
    fn generate_html(&self, program: &Program) -> Result<String, String> {
        let mut html = String::from("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<title>API Documentation</title>\n");
        html.push_str("<style>\n");
        html.push_str("body { font-family: Arial, sans-serif; margin: 40px; }\n");
        html.push_str(".function { margin: 20px 0; padding: 15px; border: 1px solid #ddd; }\n");
        html.push_str(".signature { background: #f5f5f5; padding: 10px; font-family: monospace; }\n");
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");
        
        html.push_str("<h1>API Documentation</h1>\n");
        
        for item in &program.items {
            match item {
                crate::parser::Item::Function(func) => {
                    html.push_str(&self.format_function_html(func));
                }
                crate::parser::Item::Struct(s) => {
                    html.push_str(&self.format_struct_html(s));
                }
            }
        }
        
        html.push_str("</body>\n</html>");
        Ok(html)
    }
    
    fn format_function_html(&self, func: &Function) -> String {
        let mut html = String::from("<div class='function'>\n");
        html.push_str(&format!("<h2>{}</h2>\n", func.name));
        
        html.push_str("<div class='signature'>\n");
        html.push_str(&format!("fn {}(", func.name));
        
        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                html.push_str(", ");
            }
            html.push_str(&format!("{}: {:?}", param.name, param.ty));
        }
        
        html.push(')');
        
        if let Some(ret) = &func.return_type {
            html.push_str(&format!(" -> {:?}", ret));
        }
        
        html.push_str("</div>\n");
        html.push_str("</div>\n");
        
        html
    }
    
    fn format_struct_html(&self, s: &Struct) -> String {
        let mut html = String::from("<div class='struct'>\n");
        html.push_str(&format!("<h2>struct {}</h2>\n", s.name));
        
        html.push_str("<ul>\n");
        for field in &s.fields {
            html.push_str(&format!("<li>{}: {:?}</li>\n", field.name, field.ty));
        }
        html.push_str("</ul>\n");
        
        html.push_str("</div>\n");
        html
    }
    
    fn generate_markdown(&self, program: &Program) -> Result<String, String> {
        let mut md = String::from("# API Documentation\n\n");
        
        for item in &program.items {
            match item {
                crate::parser::Item::Function(func) => {
                    md.push_str(&self.format_function_markdown(func));
                }
                crate::parser::Item::Struct(s) => {
                    md.push_str(&self.format_struct_markdown(s));
                }
            }
        }
        
        Ok(md)
    }
    
    fn format_function_markdown(&self, func: &Function) -> String {
        let mut md = format!("## {}\n\n", func.name);
        
        md.push_str("```blaze\n");
        md.push_str(&format!("fn {}(", func.name));
        
        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                md.push_str(", ");
            }
            md.push_str(&format!("{}: {:?}", param.name, param.ty));
        }
        
        md.push(')');
        
        if let Some(ret) = &func.return_type {
            md.push_str(&format!(" -> {:?}", ret));
        }
        
        md.push_str("\n```\n\n");
        md
    }
    
    fn format_struct_markdown(&self, s: &Struct) -> String {
        let mut md = format!("## struct {}\n\n", s.name);
        
        md.push_str("### Fields\n\n");
        for field in &s.fields {
            md.push_str(&format!("- `{}: {:?}`\n", field.name, field.ty));
        }
        md.push('\n');
        
        md
    }
    
    fn generate_json(&self, program: &Program) -> Result<String, String> {
        let mut json = String::from("{\n  \"items\": [\n");
        
        for (i, item) in program.items.iter().enumerate() {
            if i > 0 {
                json.push_str(",\n");
            }
            
            match item {
                crate::parser::Item::Function(func) => {
                    json.push_str(&format!("    {{\n      \"type\": \"function\",\n      \"name\": \"{}\"\n    }}", func.name));
                }
                crate::parser::Item::Struct(s) => {
                    json.push_str(&format!("    {{\n      \"type\": \"struct\",\n      \"name\": \"{}\"\n    }}", s.name));
                }
            }
        }
        
        json.push_str("\n  ]\n}");
        Ok(json)
    }
    
    fn generate_pdf(&self, _program: &Program) -> Result<String, String> {
        Err("PDF generation not implemented".to_string())
    }
}

impl Default for DocumentationGenerator {
    fn default() -> Self {
        Self::new(DocFormat::HTML, "docs".to_string())
    }
}
