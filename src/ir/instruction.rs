use super::{IRType, Constant};

#[derive(Debug, Clone)]
pub enum Instruction {
    Alloca {
        result: String,
        ty: IRType,
    },
    Load {
        result: String,
        ty: IRType,
        ptr: String,
    },
    Store {
        ty: IRType,
        value: String,
        ptr: String,
    },
    Add {
        result: String,
        ty: IRType,
        left: String,
        right: String,
    },
    Sub {
        result: String,
        ty: IRType,
        left: String,
        right: String,
    },
    Mul {
        result: String,
        ty: IRType,
        left: String,
        right: String,
    },
    Div {
        result: String,
        ty: IRType,
        left: String,
        right: String,
    },
    Rem {
        result: String,
        ty: IRType,
        left: String,
        right: String,
    },
    And {
        result: String,
        ty: IRType,
        left: String,
        right: String,
    },
    Or {
        result: String,
        ty: IRType,
        left: String,
        right: String,
    },
    Xor {
        result: String,
        ty: IRType,
        left: String,
        right: String,
    },
    Shl {
        result: String,
        ty: IRType,
        left: String,
        right: String,
    },
    Shr {
        result: String,
        ty: IRType,
        left: String,
        right: String,
    },
    ICmp {
        result: String,
        condition: ICmpCondition,
        ty: IRType,
        left: String,
        right: String,
    },
    FCmp {
        result: String,
        condition: FCmpCondition,
        ty: IRType,
        left: String,
        right: String,
    },
    Call {
        result: Option<String>,
        function: String,
        args: Vec<(IRType, String)>,
    },
    GetElementPtr {
        result: String,
        ty: IRType,
        ptr: String,
        indices: Vec<String>,
    },
    BitCast {
        result: String,
        value: String,
        from_ty: IRType,
        to_ty: IRType,
    },
    Trunc {
        result: String,
        value: String,
        from_ty: IRType,
        to_ty: IRType,
    },
    ZExt {
        result: String,
        value: String,
        from_ty: IRType,
        to_ty: IRType,
    },
    SExt {
        result: String,
        value: String,
        from_ty: IRType,
        to_ty: IRType,
    },
    FPTrunc {
        result: String,
        value: String,
        from_ty: IRType,
        to_ty: IRType,
    },
    FPExt {
        result: String,
        value: String,
        from_ty: IRType,
        to_ty: IRType,
    },
    FPToUI {
        result: String,
        value: String,
        from_ty: IRType,
        to_ty: IRType,
    },
    FPToSI {
        result: String,
        value: String,
        from_ty: IRType,
        to_ty: IRType,
    },
    UIToFP {
        result: String,
        value: String,
        from_ty: IRType,
        to_ty: IRType,
    },
    SIToFP {
        result: String,
        value: String,
        from_ty: IRType,
        to_ty: IRType,
    },
    PtrToInt {
        result: String,
        value: String,
        from_ty: IRType,
        to_ty: IRType,
    },
    IntToPtr {
        result: String,
        value: String,
        from_ty: IRType,
        to_ty: IRType,
    },
    Phi {
        result: String,
        ty: IRType,
        values: Vec<(String, String)>,
    },
    Select {
        result: String,
        condition: String,
        true_value: String,
        false_value: String,
        ty: IRType,
    },
    ExtractValue {
        result: String,
        aggregate: String,
        indices: Vec<u32>,
        ty: IRType,
    },
    InsertValue {
        result: String,
        aggregate: String,
        element: String,
        indices: Vec<u32>,
        ty: IRType,
    },
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Return {
        value: Option<(IRType, String)>,
    },
    Branch {
        dest: String,
    },
    ConditionalBranch {
        condition: String,
        true_dest: String,
        false_dest: String,
    },
    Switch {
        value: String,
        default_dest: String,
        cases: Vec<(Constant, String)>,
    },
    Unreachable,
}

#[derive(Debug, Clone)]
pub enum ICmpCondition {
    Eq,
    Ne,
    Ugt,
    Uge,
    Ult,
    Ule,
    Sgt,
    Sge,
    Slt,
    Sle,
}

#[derive(Debug, Clone)]
pub enum FCmpCondition {
    False,
    Oeq,
    Ogt,
    Oge,
    Olt,
    Ole,
    One,
    Ord,
    Ueq,
    Ugt,
    Uge,
    Ult,
    Ule,
    Une,
    Uno,
    True,
}

impl Instruction {
    pub fn get_result(&self) -> Option<&str> {
        match self {
            Instruction::Alloca { result, .. } |
            Instruction::Load { result, .. } |
            Instruction::Add { result, .. } |
            Instruction::Sub { result, .. } |
            Instruction::Mul { result, .. } |
            Instruction::Div { result, .. } |
            Instruction::Rem { result, .. } |
            Instruction::And { result, .. } |
            Instruction::Or { result, .. } |
            Instruction::Xor { result, .. } |
            Instruction::Shl { result, .. } |
            Instruction::Shr { result, .. } |
            Instruction::ICmp { result, .. } |
            Instruction::FCmp { result, .. } |
            Instruction::GetElementPtr { result, .. } |
            Instruction::BitCast { result, .. } |
            Instruction::Trunc { result, .. } |
            Instruction::ZExt { result, .. } |
            Instruction::SExt { result, .. } |
            Instruction::FPTrunc { result, .. } |
            Instruction::FPExt { result, .. } |
            Instruction::FPToUI { result, .. } |
            Instruction::FPToSI { result, .. } |
            Instruction::UIToFP { result, .. } |
            Instruction::SIToFP { result, .. } |
            Instruction::PtrToInt { result, .. } |
            Instruction::IntToPtr { result, .. } |
            Instruction::Phi { result, .. } |
            Instruction::Select { result, .. } |
            Instruction::ExtractValue { result, .. } |
            Instruction::InsertValue { result, .. } => Some(result),
            Instruction::Call { result, .. } => result.as_deref(),
            Instruction::Store { .. } => None,
        }
    }

    pub fn get_operands(&self) -> Vec<&str> {
        match self {
            Instruction::Load { ptr, .. } => vec![ptr],
            Instruction::Store { value, ptr, .. } => vec![value, ptr],
            Instruction::Add { left, right, .. } |
            Instruction::Sub { left, right, .. } |
            Instruction::Mul { left, right, .. } |
            Instruction::Div { left, right, .. } |
            Instruction::Rem { left, right, .. } |
            Instruction::And { left, right, .. } |
            Instruction::Or { left, right, .. } |
            Instruction::Xor { left, right, .. } |
            Instruction::Shl { left, right, .. } |
            Instruction::Shr { left, right, .. } |
            Instruction::ICmp { left, right, .. } |
            Instruction::FCmp { left, right, .. } => vec![left, right],
            Instruction::Call { args, .. } => args.iter().map(|(_, arg)| arg.as_str()).collect(),
            Instruction::GetElementPtr { ptr, indices, .. } => {
                let mut operands = vec![ptr.as_str()];
                operands.extend(indices.iter().map(|s| s.as_str()));
                operands
            }
            Instruction::BitCast { value, .. } |
            Instruction::Trunc { value, .. } |
            Instruction::ZExt { value, .. } |
            Instruction::SExt { value, .. } |
            Instruction::FPTrunc { value, .. } |
            Instruction::FPExt { value, .. } |
            Instruction::FPToUI { value, .. } |
            Instruction::FPToSI { value, .. } |
            Instruction::UIToFP { value, .. } |
            Instruction::SIToFP { value, .. } |
            Instruction::PtrToInt { value, .. } |
            Instruction::IntToPtr { value, .. } => vec![value],
            Instruction::Phi { values, .. } => values.iter().map(|(val, _)| val.as_str()).collect(),
            Instruction::Select { condition, true_value, false_value, .. } => {
                vec![condition, true_value, false_value]
            }
            Instruction::ExtractValue { aggregate, .. } => vec![aggregate],
            Instruction::InsertValue { aggregate, element, .. } => vec![aggregate, element],
            Instruction::Alloca { .. } => vec![],
        }
    }

    pub fn is_terminator(&self) -> bool {
        false
    }
}

impl Terminator {
    pub fn get_successors(&self) -> Vec<&str> {
        match self {
            Terminator::Branch { dest } => vec![dest],
            Terminator::ConditionalBranch { true_dest, false_dest, .. } => {
                vec![true_dest, false_dest]
            }
            Terminator::Switch { default_dest, cases, .. } => {
                let mut successors = vec![default_dest.as_str()];
                successors.extend(cases.iter().map(|(_, dest)| dest.as_str()));
                successors
            }
            Terminator::Return { .. } | Terminator::Unreachable => vec![],
        }
    }

    pub fn get_operands(&self) -> Vec<&str> {
        match self {
            Terminator::Return { value: Some((_, val)) } => vec![val],
            Terminator::ConditionalBranch { condition, .. } => vec![condition],
            Terminator::Switch { value, .. } => vec![value],
            _ => vec![],
        }
    }
}

