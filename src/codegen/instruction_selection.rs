use crate::ir::{Instruction as IRInstruction, Value, BinaryOp, UnaryOp};
use crate::codegen::register_alloc::Register;
use std::fmt;

#[derive(Debug, Clone)]
pub enum X86Instruction {
    Mov { dest: Operand, src: Operand },
    Add { dest: Operand, src: Operand },
    Sub { dest: Operand, src: Operand },
    Imul { dest: Operand, src: Operand },
    Idiv { operand: Operand },
    And { dest: Operand, src: Operand },
    Or { dest: Operand, src: Operand },
    Xor { dest: Operand, src: Operand },
    Shl { dest: Operand, amount: Operand },
    Shr { dest: Operand, amount: Operand },
    Neg { operand: Operand },
    Not { operand: Operand },
    Cmp { left: Operand, right: Operand },
    Test { left: Operand, right: Operand },
    Jmp { label: String },
    Je { label: String },
    Jne { label: String },
    Jl { label: String },
    Jle { label: String },
    Jg { label: String },
    Jge { label: String },
    Call { target: String },
    Ret,
    Push { operand: Operand },
    Pop { operand: Operand },
    Label { name: String },
    Lea { dest: Operand, src: Operand },
    Movsd { dest: Operand, src: Operand },
    Addsd { dest: Operand, src: Operand },
    Subsd { dest: Operand, src: Operand },
    Mulsd { dest: Operand, src: Operand },
    Divsd { dest: Operand, src: Operand },
}

#[derive(Debug, Clone)]
pub enum Operand {
    Register(Register),
    Immediate(i64),
    Memory { base: Register, offset: i32 },
    Label(String),
    Stack(i32),
}

pub struct InstructionSelector {
    next_label: usize,
}

impl InstructionSelector {
    pub fn new() -> Self {
        Self { next_label: 0 }
    }
    
    pub fn select(&mut self, ir_instr: &IRInstruction) -> Vec<X86Instruction> {
        match ir_instr {
            IRInstruction::Binary { op, left, right, result } => {
                self.select_binary(*op, left, right, result)
            }
            
            IRInstruction::Unary { op, operand, result } => {
                self.select_unary(*op, operand, result)
            }
            
            IRInstruction::Store { dest, value } => {
                self.select_store(dest, value)
            }
            
            IRInstruction::Load { dest, src } => {
                self.select_load(dest, src)
            }
            
            IRInstruction::Call { function, args, result } => {
                self.select_call(function, args, result.as_ref())
            }
            
            IRInstruction::Return { value } => {
                self.select_return(value.as_ref())
            }
            
            IRInstruction::Branch { condition, true_label, false_label } => {
                self.select_branch(condition, true_label, false_label)
            }
            
            IRInstruction::Jump { target } => {
                vec![X86Instruction::Jmp { label: target.clone() }]
            }
            
            IRInstruction::Label { name } => {
                vec![X86Instruction::Label { name: name.clone() }]
            }
            
            _ => Vec::new(),
        }
    }
    
    fn select_binary(&mut self, op: BinaryOp, left: &Value, right: &Value, result: &str) -> Vec<X86Instruction> {
        let mut instrs = Vec::new();
        let dest = Operand::Register(Register::RAX);
        
        instrs.push(X86Instruction::Mov {
            dest: dest.clone(),
            src: self.value_to_operand(left),
        });
        
        let src = self.value_to_operand(right);
        
        match op {
            BinaryOp::Add => instrs.push(X86Instruction::Add { dest: dest.clone(), src }),
            BinaryOp::Sub => instrs.push(X86Instruction::Sub { dest: dest.clone(), src }),
            BinaryOp::Mul => instrs.push(X86Instruction::Imul { dest: dest.clone(), src }),
            BinaryOp::Div => {
                instrs.push(X86Instruction::Mov { dest: Operand::Register(Register::RDX), src: Operand::Immediate(0) });
                instrs.push(X86Instruction::Idiv { operand: src });
            }
            BinaryOp::Mod => {
                instrs.push(X86Instruction::Mov { dest: Operand::Register(Register::RDX), src: Operand::Immediate(0) });
                instrs.push(X86Instruction::Idiv { operand: src });
                instrs.push(X86Instruction::Mov { dest: dest.clone(), src: Operand::Register(Register::RDX) });
            }
            BinaryOp::BitwiseAnd => instrs.push(X86Instruction::And { dest: dest.clone(), src }),
            BinaryOp::BitwiseOr => instrs.push(X86Instruction::Or { dest: dest.clone(), src }),
            BinaryOp::BitwiseXor => instrs.push(X86Instruction::Xor { dest: dest.clone(), src }),
            BinaryOp::LeftShift => instrs.push(X86Instruction::Shl { dest: dest.clone(), amount: src }),
            BinaryOp::RightShift => instrs.push(X86Instruction::Shr { dest: dest.clone(), amount: src }),
            
            BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                instrs.push(X86Instruction::Cmp { left: dest.clone(), right: src });
                let label_true = self.new_label("cmp_true");
                let label_end = self.new_label("cmp_end");
                
                let jump = match op {
                    BinaryOp::Eq => X86Instruction::Je { label: label_true.clone() },
                    BinaryOp::Ne => X86Instruction::Jne { label: label_true.clone() },
                    BinaryOp::Lt => X86Instruction::Jl { label: label_true.clone() },
                    BinaryOp::Le => X86Instruction::Jle { label: label_true.clone() },
                    BinaryOp::Gt => X86Instruction::Jg { label: label_true.clone() },
                    BinaryOp::Ge => X86Instruction::Jge { label: label_true.clone() },
                    _ => unreachable!(),
                };
                
                instrs.push(jump);
                instrs.push(X86Instruction::Mov { dest: dest.clone(), src: Operand::Immediate(0) });
                instrs.push(X86Instruction::Jmp { label: label_end.clone() });
                instrs.push(X86Instruction::Label { name: label_true });
                instrs.push(X86Instruction::Mov { dest: dest.clone(), src: Operand::Immediate(1) });
                instrs.push(X86Instruction::Label { name: label_end });
            }
            
            _ => {}
        }
        
        instrs.push(X86Instruction::Mov {
            dest: Operand::Label(result.to_string()),
            src: dest,
        });
        
        instrs
    }
    
    fn select_unary(&mut self, op: UnaryOp, operand: &Value, result: &str) -> Vec<X86Instruction> {
        let mut instrs = Vec::new();
        let dest = Operand::Register(Register::RAX);
        
        instrs.push(X86Instruction::Mov {
            dest: dest.clone(),
            src: self.value_to_operand(operand),
        });
        
        match op {
            UnaryOp::Neg => instrs.push(X86Instruction::Neg { operand: dest.clone() }),
            UnaryOp::Not => instrs.push(X86Instruction::Not { operand: dest.clone() }),
            _ => {}
        }
        
        instrs.push(X86Instruction::Mov {
            dest: Operand::Label(result.to_string()),
            src: dest,
        });
        
        instrs
    }
    
    fn select_store(&mut self, dest: &str, value: &Value) -> Vec<X86Instruction> {
        vec![X86Instruction::Mov {
            dest: Operand::Label(dest.to_string()),
            src: self.value_to_operand(value),
        }]
    }
    
    fn select_load(&mut self, dest: &str, src: &str) -> Vec<X86Instruction> {
        vec![X86Instruction::Mov {
            dest: Operand::Label(dest.to_string()),
            src: Operand::Label(src.to_string()),
        }]
    }
    
    fn select_call(&mut self, function: &str, args: &[Value], result: Option<&String>) -> Vec<X86Instruction> {
        let mut instrs = Vec::new();
        
        let arg_regs = [
            Register::RDI,
            Register::RSI,
            Register::RDX,
            Register::RCX,
            Register::R8,
            Register::R9,
        ];
        
        for (i, arg) in args.iter().take(6).enumerate() {
            instrs.push(X86Instruction::Mov {
                dest: Operand::Register(arg_regs[i]),
                src: self.value_to_operand(arg),
            });
        }
        
        for arg in args.iter().skip(6).rev() {
            instrs.push(X86Instruction::Push {
                operand: self.value_to_operand(arg),
            });
        }
        
        instrs.push(X86Instruction::Call {
            target: function.to_string(),
        });
        
        if let Some(result_var) = result {
            instrs.push(X86Instruction::Mov {
                dest: Operand::Label(result_var.clone()),
                src: Operand::Register(Register::RAX),
            });
        }
        
        instrs
    }
    
    fn select_return(&mut self, value: Option<&Value>) -> Vec<X86Instruction> {
        let mut instrs = Vec::new();
        
        if let Some(val) = value {
            instrs.push(X86Instruction::Mov {
                dest: Operand::Register(Register::RAX),
                src: self.value_to_operand(val),
            });
        }
        
        instrs.push(X86Instruction::Ret);
        instrs
    }
    
    fn select_branch(&mut self, condition: &Value, true_label: &str, false_label: &str) -> Vec<X86Instruction> {
        vec![
            X86Instruction::Test {
                left: self.value_to_operand(condition),
                right: self.value_to_operand(condition),
            },
            X86Instruction::Jne { label: true_label.to_string() },
            X86Instruction::Jmp { label: false_label.to_string() },
        ]
    }
    
    fn value_to_operand(&self, value: &Value) -> Operand {
        match value {
            Value::Const(c) => Operand::Immediate(c.to_i64()),
            Value::Variable(v) => Operand::Label(v.clone()),
            _ => Operand::Immediate(0),
        }
    }
    
    fn new_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.next_label);
        self.next_label += 1;
        label
    }
}

impl fmt::Display for X86Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            X86Instruction::Mov { dest, src } => write!(f, "mov {}, {}", dest, src),
            X86Instruction::Add { dest, src } => write!(f, "add {}, {}", dest, src),
            X86Instruction::Sub { dest, src } => write!(f, "sub {}, {}", dest, src),
            X86Instruction::Imul { dest, src } => write!(f, "imul {}, {}", dest, src),
            X86Instruction::Idiv { operand } => write!(f, "idiv {}", operand),
            X86Instruction::And { dest, src } => write!(f, "and {}, {}", dest, src),
            X86Instruction::Or { dest, src } => write!(f, "or {}, {}", dest, src),
            X86Instruction::Xor { dest, src } => write!(f, "xor {}, {}", dest, src),
            X86Instruction::Shl { dest, amount } => write!(f, "shl {}, {}", dest, amount),
            X86Instruction::Shr { dest, amount } => write!(f, "shr {}, {}", dest, amount),
            X86Instruction::Neg { operand } => write!(f, "neg {}", operand),
            X86Instruction::Not { operand } => write!(f, "not {}", operand),
            X86Instruction::Cmp { left, right } => write!(f, "cmp {}, {}", left, right),
            X86Instruction::Test { left, right } => write!(f, "test {}, {}", left, right),
            X86Instruction::Jmp { label } => write!(f, "jmp {}", label),
            X86Instruction::Je { label } => write!(f, "je {}", label),
            X86Instruction::Jne { label } => write!(f, "jne {}", label),
            X86Instruction::Jl { label } => write!(f, "jl {}", label),
            X86Instruction::Jle { label } => write!(f, "jle {}", label),
            X86Instruction::Jg { label } => write!(f, "jg {}", label),
            X86Instruction::Jge { label } => write!(f, "jge {}", label),
            X86Instruction::Call { target } => write!(f, "call {}", target),
            X86Instruction::Ret => write!(f, "ret"),
            X86Instruction::Push { operand } => write!(f, "push {}", operand),
            X86Instruction::Pop { operand } => write!(f, "pop {}", operand),
            X86Instruction::Label { name } => write!(f, "{}:", name),
            X86Instruction::Lea { dest, src } => write!(f, "lea {}, {}", dest, src),
            X86Instruction::Movsd { dest, src } => write!(f, "movsd {}, {}", dest, src),
            X86Instruction::Addsd { dest, src } => write!(f, "addsd {}, {}", dest, src),
            X86Instruction::Subsd { dest, src } => write!(f, "subsd {}, {}", dest, src),
            X86Instruction::Mulsd { dest, src } => write!(f, "mulsd {}, {}", dest, src),
            X86Instruction::Divsd { dest, src } => write!(f, "divsd {}, {}", dest, src),
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operand::Register(reg) => write!(f, "{}", reg.to_string()),
            Operand::Immediate(val) => write!(f, "${}", val),
            Operand::Memory { base, offset } => write!(f, "{}({})", offset, base.to_string()),
            Operand::Label(name) => write!(f, "{}", name),
            Operand::Stack(offset) => write!(f, "{}(%rbp)", offset),
        }
    }
}

impl Default for InstructionSelector {
    fn default() -> Self {
        Self::new()
    }
}

trait ToI64 {
    fn to_i64(&self) -> i64;
}

impl ToI64 for crate::ir::Constant {
    fn to_i64(&self) -> i64 {
        match self {
            crate::ir::Constant::Int(i) => *i,
            crate::ir::Constant::Float(f) => *f as i64,
            crate::ir::Constant::Bool(b) => if *b { 1 } else { 0 },
            _ => 0,
        }
    }
}
