use std::collections::HashMap;

pub struct WASMGenerator {
    functions: Vec<WASMFunction>,
    globals: Vec<WASMGlobal>,
    memory: WASMMemory,
    exports: Vec<WASMExport>,
}

#[derive(Debug, Clone)]
pub struct WASMFunction {
    pub name: String,
    pub params: Vec<WASMType>,
    pub results: Vec<WASMType>,
    pub locals: Vec<WASMType>,
    pub instructions: Vec<WASMInstruction>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WASMType {
    I32,
    I64,
    F32,
    F64,
}

#[derive(Debug, Clone)]
pub enum WASMInstruction {
    LocalGet(u32),
    LocalSet(u32),
    LocalTee(u32),
    GlobalGet(u32),
    GlobalSet(u32),
    I32Const(i32),
    I64Const(i64),
    F32Const(f32),
    F64Const(f64),
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    Call(u32),
    Return,
    If,
    Else,
    End,
    Loop,
    Block,
    Br(u32),
    BrIf(u32),
}

#[derive(Debug, Clone)]
pub struct WASMGlobal {
    pub name: String,
    pub value_type: WASMType,
    pub mutable: bool,
    pub init_value: i64,
}

#[derive(Debug, Clone)]
pub struct WASMMemory {
    pub initial_pages: u32,
    pub maximum_pages: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct WASMExport {
    pub name: String,
    pub kind: WASMExportKind,
    pub index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WASMExportKind {
    Function,
    Global,
    Memory,
    Table,
}

impl WASMGenerator {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            globals: Vec::new(),
            memory: WASMMemory {
                initial_pages: 1,
                maximum_pages: None,
            },
            exports: Vec::new(),
        }
    }
    
    pub fn add_function(&mut self, func: WASMFunction) {
        self.functions.push(func);
    }
    
    pub fn add_global(&mut self, global: WASMGlobal) {
        self.globals.push(global);
    }
    
    pub fn export_function(&mut self, name: String, func_index: u32) {
        self.exports.push(WASMExport {
            name,
            kind: WASMExportKind::Function,
            index: func_index,
        });
    }
    
    pub fn generate(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        bytes.extend_from_slice(&[0x00, 0x61, 0x73, 0x6D]);
        bytes.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
        
        self.encode_type_section(&mut bytes);
        self.encode_function_section(&mut bytes);
        self.encode_memory_section(&mut bytes);
        self.encode_export_section(&mut bytes);
        self.encode_code_section(&mut bytes);
        
        bytes
    }
    
    fn encode_type_section(&self, bytes: &mut Vec<u8>) {
        if self.functions.is_empty() {
            return;
        }
        
        bytes.push(0x01);
        
        let mut section_bytes = Vec::new();
        self.encode_varuint(&mut section_bytes, self.functions.len() as u32);
        
        for func in &self.functions {
            section_bytes.push(0x60);
            
            self.encode_varuint(&mut section_bytes, func.params.len() as u32);
            for param in &func.params {
                section_bytes.push(self.type_to_byte(*param));
            }
            
            self.encode_varuint(&mut section_bytes, func.results.len() as u32);
            for result in &func.results {
                section_bytes.push(self.type_to_byte(*result));
            }
        }
        
        self.encode_varuint(bytes, section_bytes.len() as u32);
        bytes.extend_from_slice(&section_bytes);
    }
    
    fn encode_function_section(&self, bytes: &mut Vec<u8>) {
        if self.functions.is_empty() {
            return;
        }
        
        bytes.push(0x03);
        
        let mut section_bytes = Vec::new();
        self.encode_varuint(&mut section_bytes, self.functions.len() as u32);
        
        for i in 0..self.functions.len() {
            self.encode_varuint(&mut section_bytes, i as u32);
        }
        
        self.encode_varuint(bytes, section_bytes.len() as u32);
        bytes.extend_from_slice(&section_bytes);
    }
    
    fn encode_memory_section(&self, bytes: &mut Vec<u8>) {
        bytes.push(0x05);
        
        let mut section_bytes = Vec::new();
        self.encode_varuint(&mut section_bytes, 1);
        
        if let Some(max) = self.memory.maximum_pages {
            section_bytes.push(0x01);
            self.encode_varuint(&mut section_bytes, self.memory.initial_pages);
            self.encode_varuint(&mut section_bytes, max);
        } else {
            section_bytes.push(0x00);
            self.encode_varuint(&mut section_bytes, self.memory.initial_pages);
        }
        
        self.encode_varuint(bytes, section_bytes.len() as u32);
        bytes.extend_from_slice(&section_bytes);
    }
    
    fn encode_export_section(&self, bytes: &mut Vec<u8>) {
        if self.exports.is_empty() {
            return;
        }
        
        bytes.push(0x07);
        
        let mut section_bytes = Vec::new();
        self.encode_varuint(&mut section_bytes, self.exports.len() as u32);
        
        for export in &self.exports {
            self.encode_string(&mut section_bytes, &export.name);
            
            section_bytes.push(match export.kind {
                WASMExportKind::Function => 0x00,
                WASMExportKind::Table => 0x01,
                WASMExportKind::Memory => 0x02,
                WASMExportKind::Global => 0x03,
            });
            
            self.encode_varuint(&mut section_bytes, export.index);
        }
        
        self.encode_varuint(bytes, section_bytes.len() as u32);
        bytes.extend_from_slice(&section_bytes);
    }
    
    fn encode_code_section(&self, bytes: &mut Vec<u8>) {
        if self.functions.is_empty() {
            return;
        }
        
        bytes.push(0x0A);
        
        let mut section_bytes = Vec::new();
        self.encode_varuint(&mut section_bytes, self.functions.len() as u32);
        
        for func in &self.functions {
            let mut func_bytes = Vec::new();
            
            self.encode_varuint(&mut func_bytes, func.locals.len() as u32);
            for local in &func.locals {
                self.encode_varuint(&mut func_bytes, 1);
                func_bytes.push(self.type_to_byte(*local));
            }
            
            for instr in &func.instructions {
                self.encode_instruction(&mut func_bytes, instr);
            }
            
            func_bytes.push(0x0B);
            
            self.encode_varuint(&mut section_bytes, func_bytes.len() as u32);
            section_bytes.extend_from_slice(&func_bytes);
        }
        
        self.encode_varuint(bytes, section_bytes.len() as u32);
        bytes.extend_from_slice(&section_bytes);
    }
    
    fn encode_instruction(&self, bytes: &mut Vec<u8>, instr: &WASMInstruction) {
        match instr {
            WASMInstruction::LocalGet(idx) => {
                bytes.push(0x20);
                self.encode_varuint(bytes, *idx);
            }
            WASMInstruction::LocalSet(idx) => {
                bytes.push(0x21);
                self.encode_varuint(bytes, *idx);
            }
            WASMInstruction::I32Const(val) => {
                bytes.push(0x41);
                self.encode_varint(bytes, *val as i64);
            }
            WASMInstruction::I32Add => bytes.push(0x6A),
            WASMInstruction::I32Sub => bytes.push(0x6B),
            WASMInstruction::I32Mul => bytes.push(0x6C),
            WASMInstruction::Return => bytes.push(0x0F),
            WASMInstruction::End => bytes.push(0x0B),
            _ => {}
        }
    }
    
    fn type_to_byte(&self, ty: WASMType) -> u8 {
        match ty {
            WASMType::I32 => 0x7F,
            WASMType::I64 => 0x7E,
            WASMType::F32 => 0x7D,
            WASMType::F64 => 0x7C,
        }
    }
    
    fn encode_varuint(&self, bytes: &mut Vec<u8>, mut value: u32) {
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            
            if value != 0 {
                byte |= 0x80;
            }
            
            bytes.push(byte);
            
            if value == 0 {
                break;
            }
        }
    }
    
    fn encode_varint(&self, bytes: &mut Vec<u8>, mut value: i64) {
        loop {
            let mut byte = (value & 0x7F) as u8;
            value >>= 7;
            
            if (value == 0 && (byte & 0x40) == 0) || (value == -1 && (byte & 0x40) != 0) {
                bytes.push(byte);
                break;
            }
            
            byte |= 0x80;
            bytes.push(byte);
        }
    }
    
    fn encode_string(&self, bytes: &mut Vec<u8>, s: &str) {
        self.encode_varuint(bytes, s.len() as u32);
        bytes.extend_from_slice(s.as_bytes());
    }
}

impl Default for WASMGenerator {
    fn default() -> Self {
        Self::new()
    }
}
