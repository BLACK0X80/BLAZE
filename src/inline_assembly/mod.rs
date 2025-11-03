use std::collections::HashMap;

pub struct InlineAssemblyParser {
    register_constraints: HashMap<String, RegisterConstraint>,
}

#[derive(Debug, Clone)]
pub struct InlineAssembly {
    pub template: String,
    pub inputs: Vec<AssemblyOperand>,
    pub outputs: Vec<AssemblyOperand>,
    pub clobbers: Vec<String>,
    pub options: Vec<AsmOption>,
    pub dialect: AsmDialect,
}

#[derive(Debug, Clone)]
pub struct AssemblyOperand {
    pub constraint: String,
    pub variable: String,
    pub register_class: Option<RegisterClass>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RegisterClass {
    General,
    Integer,
    Float,
    Vector,
    Any,
}

#[derive(Debug, Clone)]
pub enum RegisterConstraint {
    Input(String),
    Output(String),
    InputOutput(String),
    Clobber(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AsmDialect {
    ATT,
    Intel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AsmOption {
    Pure,
    NoMem,
    ReadOnly,
    PreserveSignedFlags,
    NoReturn,
    NoStack,
}

impl InlineAssemblyParser {
    pub fn new() -> Self {
        Self {
            register_constraints: HashMap::new(),
        }
    }
    
    pub fn parse_inline_asm(&mut self, asm_str: &str) -> Result<InlineAssembly, String> {
        let parts: Vec<&str> = asm_str.split(':').collect();
        
        if parts.is_empty() {
            return Err("Empty assembly string".to_string());
        }
        
        let template = parts[0].trim().to_string();
        let outputs = if parts.len() > 1 {
            self.parse_operands(parts[1])?
        } else {
            Vec::new()
        };
        
        let inputs = if parts.len() > 2 {
            self.parse_operands(parts[2])?
        } else {
            Vec::new()
        };
        
        let clobbers = if parts.len() > 3 {
            self.parse_clobbers(parts[3])?
        } else {
            Vec::new()
        };
        
        let options = if parts.len() > 4 {
            self.parse_options(parts[4])?
        } else {
            Vec::new()
        };
        
        Ok(InlineAssembly {
            template,
            inputs,
            outputs,
            clobbers,
            options,
            dialect: AsmDialect::ATT,
        })
    }
    
    fn parse_operands(&self, operands_str: &str) -> Result<Vec<AssemblyOperand>, String> {
        let mut operands = Vec::new();
        
        for operand in operands_str.split(',') {
            let operand = operand.trim();
            if operand.is_empty() {
                continue;
            }
            
            let parts: Vec<&str> = operand.split('(').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid operand format: {}", operand));
            }
            
            let constraint = parts[0].trim().trim_matches('"').to_string();
            let variable = parts[1].trim_end_matches(')').trim().to_string();
            
            operands.push(AssemblyOperand {
                constraint,
                variable,
                register_class: None,
            });
        }
        
        Ok(operands)
    }
    
    fn parse_clobbers(&self, clobbers_str: &str) -> Result<Vec<String>, String> {
        Ok(clobbers_str
            .split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }
    
    fn parse_options(&self, options_str: &str) -> Result<Vec<AsmOption>, String> {
        let mut options = Vec::new();
        
        for opt in options_str.split(',') {
            let opt = opt.trim();
            match opt {
                "pure" => options.push(AsmOption::Pure),
                "nomem" => options.push(AsmOption::NoMem),
                "readonly" => options.push(AsmOption::ReadOnly),
                "preserveflags" => options.push(AsmOption::PreserveSignedFlags),
                "noreturn" => options.push(AsmOption::NoReturn),
                "nostack" => options.push(AsmOption::NoStack),
                _ => {}
            }
        }
        
        Ok(options)
    }
    
    pub fn validate_constraints(&self, asm: &InlineAssembly) -> Result<(), String> {
        for output in &asm.outputs {
            if !self.is_valid_constraint(&output.constraint) {
                return Err(format!("Invalid output constraint: {}", output.constraint));
            }
        }
        
        for input in &asm.inputs {
            if !self.is_valid_constraint(&input.constraint) {
                return Err(format!("Invalid input constraint: {}", input.constraint));
            }
        }
        
        Ok(())
    }
    
    fn is_valid_constraint(&self, constraint: &str) -> bool {
        matches!(constraint, "r" | "m" | "i" | "a" | "b" | "c" | "d" | "x" | "y" | "z")
    }
}

impl Default for InlineAssemblyParser {
    fn default() -> Self {
        Self::new()
    }
}

pub struct X86AssemblyEmitter {
    instructions: Vec<String>,
}

impl X86AssemblyEmitter {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
        }
    }
    
    pub fn emit_asm(&mut self, asm: &InlineAssembly) -> String {
        let mut output = String::new();
        
        output.push_str("asm!(");
        output.push_str(&format!("\"{}\"", asm.template));
        
        if !asm.outputs.is_empty() {
            output.push_str(", out(");
            for (i, operand) in asm.outputs.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str(&operand.constraint);
                output.push_str(") ");
                output.push_str(&operand.variable);
            }
            output.push(')');
        }
        
        if !asm.inputs.is_empty() {
            output.push_str(", in(");
            for (i, operand) in asm.inputs.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str(&operand.constraint);
                output.push_str(") ");
                output.push_str(&operand.variable);
            }
            output.push(')');
        }
        
        if !asm.clobbers.is_empty() {
            output.push_str(", clobber_abi(");
            for (i, clobber) in asm.clobbers.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str(&format!("\"{}\"", clobber));
            }
            output.push(')');
        }
        
        output.push_str(");");
        output
    }
}

impl Default for X86AssemblyEmitter {
    fn default() -> Self {
        Self::new()
    }
}
