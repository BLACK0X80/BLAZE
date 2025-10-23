use super::IRType;

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Alloca { result: String, ty: IRType },
    Store { value: String, ptr: String },
    Load { result: String, ptr: String },
    Const { result: String, ty: IRType, value: String },
    Add { result: String, left: String, right: String, ty: IRType },
    Sub { result: String, left: String, right: String, ty: IRType },
    Mul { result: String, left: String, right: String, ty: IRType },
    Div { result: String, left: String, right: String, ty: IRType },
    Mod { result: String, left: String, right: String, ty: IRType },
    ICmp { result: String, condition: ICmpCondition, left: String, right: String },
    FCmp { result: String, condition: FCmpCondition, left: String, right: String },
    Call { result: Option<String>, func: String, args: Vec<String> },
    GetElementPtr { result: String, ptr: String, indices: Vec<String> },
    BitCast { result: String, value: String, ty: IRType },
    Phi { result: String, ty: IRType, incoming: Vec<(String, String)> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Terminator {
    Ret { value: Option<String> },
    Br { target: String },
    CondBr { condition: String, true_target: String, false_target: String },
    Unreachable,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ICmpCondition {
    Eq,
    Ne,
    Slt,
    Sle,
    Sgt,
    Sge,
    Ult,
    Ule,
    Ugt,
    Uge,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FCmpCondition {
    Oeq,
    One,
    Olt,
    Ole,
    Ogt,
    Oge,
    Ueq,
    Une,
    Ult,
    Ule,
    Ugt,
    Uge,
}

impl Instruction {
    pub fn get_result(&self) -> Option<&str> {
        match self {
            Instruction::Alloca { result, .. } => Some(result),
            Instruction::Load { result, .. } => Some(result),
            Instruction::Const { result, .. } => Some(result),
            Instruction::Add { result, .. } => Some(result),
            Instruction::Sub { result, .. } => Some(result),
            Instruction::Mul { result, .. } => Some(result),
            Instruction::Div { result, .. } => Some(result),
            Instruction::Mod { result, .. } => Some(result),
            Instruction::ICmp { result, .. } => Some(result),
            Instruction::FCmp { result, .. } => Some(result),
            Instruction::Call { result, .. } => result.as_deref(),
            Instruction::GetElementPtr { result, .. } => Some(result),
            Instruction::BitCast { result, .. } => Some(result),
            Instruction::Phi { result, .. } => Some(result),
            _ => None,
        }
    }

    pub fn get_operands(&self) -> Vec<&str> {
        match self {
            Instruction::Store { value, ptr } => vec![value, ptr],
            Instruction::Load { ptr, .. } => vec![ptr],
            Instruction::Add { left, right, .. } => vec![left, right],
            Instruction::Sub { left, right, .. } => vec![left, right],
            Instruction::Mul { left, right, .. } => vec![left, right],
            Instruction::Div { left, right, .. } => vec![left, right],
            Instruction::Mod { left, right, .. } => vec![left, right],
            Instruction::ICmp { left, right, .. } => vec![left, right],
            Instruction::FCmp { left, right, .. } => vec![left, right],
            Instruction::Call { func, args, .. } => {
                let mut ops = vec![func.as_str()];
                ops.extend(args.iter().map(|s| s.as_str()));
                ops
            }
            Instruction::GetElementPtr { ptr, indices, .. } => {
                let mut ops = vec![ptr.as_str()];
                ops.extend(indices.iter().map(|s| s.as_str()));
                ops
            }
            Instruction::BitCast { value, .. } => vec![value],
            Instruction::Phi { incoming, .. } => {
                incoming.iter().map(|(val, _)| val.as_str()).collect()
            }
            _ => vec![],
        }
    }
}
