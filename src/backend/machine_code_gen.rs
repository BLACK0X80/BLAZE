use std::collections::HashMap;

pub struct MachineCodeGenerator {
    target_arch: TargetArchitecture,
    code_buffer: Vec<u8>,
    relocations: Vec<Relocation>,
    symbol_table: HashMap<String, usize>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TargetArchitecture {
    X86_64,
    ARM64,
    RISCV64,
}

#[derive(Debug, Clone)]
pub struct Relocation {
    pub offset: usize,
    pub symbol: String,
    pub reloc_type: RelocationType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RelocationType {
    Absolute,
    Relative,
    PLT,
    GOT,
}

impl MachineCodeGenerator {
    pub fn new(target: TargetArchitecture) -> Self {
        Self {
            target_arch: target,
            code_buffer: Vec::new(),
            relocations: Vec::new(),
            symbol_table: HashMap::new(),
        }
    }
    
    pub fn emit_function(&mut self, name: String, instructions: Vec<MachineInstruction>) {
        let start_offset = self.code_buffer.len();
        self.symbol_table.insert(name, start_offset);
        
        for instr in instructions {
            self.emit_instruction(&instr);
        }
    }
    
    fn emit_instruction(&mut self, instr: &MachineInstruction) {
        match self.target_arch {
            TargetArchitecture::X86_64 => self.emit_x86_64(instr),
            TargetArchitecture::ARM64 => self.emit_arm64(instr),
            TargetArchitecture::RISCV64 => self.emit_riscv64(instr),
        }
    }
    
    fn emit_x86_64(&mut self, instr: &MachineInstruction) {
        match instr {
            MachineInstruction::Move { dest, src } => {
                self.code_buffer.push(0x48);
                self.code_buffer.push(0x89);
                self.emit_modrm(dest, src);
            }
            
            MachineInstruction::Add { dest, src } => {
                self.code_buffer.push(0x48);
                self.code_buffer.push(0x01);
                self.emit_modrm(dest, src);
            }
            
            MachineInstruction::Sub { dest, src } => {
                self.code_buffer.push(0x48);
                self.code_buffer.push(0x29);
                self.emit_modrm(dest, src);
            }
            
            MachineInstruction::Mul { dest, src } => {
                self.code_buffer.push(0x48);
                self.code_buffer.push(0x0F);
                self.code_buffer.push(0xAF);
                self.emit_modrm(dest, src);
            }
            
            MachineInstruction::Call { target } => {
                self.code_buffer.push(0xE8);
                let offset = self.code_buffer.len();
                self.code_buffer.extend_from_slice(&[0, 0, 0, 0]);
                
                self.relocations.push(Relocation {
                    offset,
                    symbol: target.clone(),
                    reloc_type: RelocationType::Relative,
                });
            }
            
            MachineInstruction::Return => {
                self.code_buffer.push(0xC3);
            }
            
            _ => {}
        }
    }
    
    fn emit_arm64(&mut self, instr: &MachineInstruction) {
        match instr {
            MachineInstruction::Add { dest, src } => {
                let encoding = self.encode_arm64_add(dest, src);
                self.code_buffer.extend_from_slice(&encoding.to_le_bytes());
            }
            
            MachineInstruction::Sub { dest, src } => {
                let encoding = self.encode_arm64_sub(dest, src);
                self.code_buffer.extend_from_slice(&encoding.to_le_bytes());
            }
            
            MachineInstruction::Return => {
                self.code_buffer.extend_from_slice(&[0xC0, 0x03, 0x5F, 0xD6]);
            }
            
            _ => {}
        }
    }
    
    fn emit_riscv64(&mut self, instr: &MachineInstruction) {
        match instr {
            MachineInstruction::Add { dest, src } => {
                let encoding = self.encode_riscv64_add(dest, src);
                self.code_buffer.extend_from_slice(&encoding.to_le_bytes());
            }
            
            MachineInstruction::Return => {
                self.code_buffer.extend_from_slice(&[0x67, 0x80, 0x00, 0x00]);
            }
            
            _ => {}
        }
    }
    
    fn emit_modrm(&mut self, dest: &Operand, src: &Operand) {
        let (dest_reg, src_reg) = match (dest, src) {
            (Operand::Register(d), Operand::Register(s)) => (*d, *s),
            _ => (0, 0),
        };
        
        let modrm = 0xC0 | ((dest_reg & 7) << 3) | (src_reg & 7);
        self.code_buffer.push(modrm);
    }
    
    fn encode_arm64_add(&self, _dest: &Operand, _src: &Operand) -> u32 {
        0x8B000000
    }
    
    fn encode_arm64_sub(&self, _dest: &Operand, _src: &Operand) -> u32 {
        0xCB000000
    }
    
    fn encode_riscv64_add(&self, _dest: &Operand, _src: &Operand) -> u32 {
        0x00000033
    }
    
    pub fn finalize(&mut self) -> Vec<u8> {
        for reloc in &self.relocations {
            if let Some(&target_offset) = self.symbol_table.get(&reloc.symbol) {
                let current_offset = reloc.offset;
                let relative_offset = (target_offset as i64 - current_offset as i64 - 4) as i32;
                
                let bytes = relative_offset.to_le_bytes();
                self.code_buffer[reloc.offset..reloc.offset + 4].copy_from_slice(&bytes);
            }
        }
        
        self.code_buffer.clone()
    }
}

#[derive(Debug, Clone)]
pub enum MachineInstruction {
    Move { dest: Operand, src: Operand },
    Add { dest: Operand, src: Operand },
    Sub { dest: Operand, src: Operand },
    Mul { dest: Operand, src: Operand },
    Div { dest: Operand, src: Operand },
    Load { dest: Operand, addr: Operand },
    Store { src: Operand, addr: Operand },
    Call { target: String },
    Jump { target: String },
    ConditionalJump { condition: Condition, target: String },
    Return,
    Push { operand: Operand },
    Pop { operand: Operand },
}

#[derive(Debug, Clone)]
pub enum Operand {
    Register(u8),
    Immediate(i64),
    Memory { base: u8, offset: i32 },
    Label(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Condition {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

impl Default for MachineCodeGenerator {
    fn default() -> Self {
        Self::new(TargetArchitecture::X86_64)
    }
}
