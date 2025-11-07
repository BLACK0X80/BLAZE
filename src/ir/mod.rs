pub mod builder;
pub mod instruction;
pub mod optimization;
pub mod validation;
pub mod ssa;

pub use instruction::{Instruction, Terminator, ICmpCondition, FCmpCondition};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum IRType {
    Void,
    I1,
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
    Pointer(Box<IRType>),
    Array { element_type: Box<IRType>, size: u64 },
    Struct { fields: Vec<IRType> },
    Function { params: Vec<IRType>, return_type: Box<IRType> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    Integer { value: i64, ty: IRType },
    Float { value: f64, ty: IRType },
    String(String),
    Boolean(bool),
    Null,
    Undefined,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub functions: Vec<IRFunction>,
    pub globals: Vec<GlobalVariable>,
    pub types: Vec<TypeDefinition>,
}

impl Module {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            functions: Vec::new(),
            globals: Vec::new(),
            types: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IRFunction {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: IRType,
    pub blocks: Vec<BasicBlock>,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub label: String,
    pub instructions: Vec<Instruction>,
    pub terminator: Terminator,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub ty: IRType,
}

#[derive(Debug, Clone)]
pub struct GlobalVariable {
    pub name: String,
    pub ty: IRType,
    pub initializer: Option<Constant>,
    pub is_constant: bool,
}

#[derive(Debug, Clone)]
pub struct TypeDefinition {
    pub name: String,
    pub ty: IRType,
}

pub fn generate(program: &crate::parser::Program) -> anyhow::Result<Module> {
    let mut builder = builder::IRBuilder::new();
    builder.build_module(program)
}
