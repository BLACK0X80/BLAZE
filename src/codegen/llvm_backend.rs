use crate::ir::{Module, IRFunction, BasicBlock, Instruction, Terminator, IRType, Constant};
use std::path::PathBuf;
use std::collections::HashMap;
use anyhow::Result;

pub struct LLVMCodegen {
    target_triple: String,
    optimization_level: u8,
}

impl LLVMCodegen {
    pub fn new() -> Self {
        Self {
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            optimization_level: 2,
        }
    }

    pub fn generate(
        &mut self,
        module: &Module,
        output: Option<PathBuf>,
        emit: Option<String>,
    ) -> Result<()> {
        let llvm_ir = self.generate_llvm_ir(module)?;
        
        match emit.as_deref() {
            Some("llvm-ir") => {
                let output_path = output.unwrap_or_else(|| PathBuf::from("output.ll"));
                std::fs::write(output_path, llvm_ir)?;
            }
            Some("asm") => {
                let output_path = output.unwrap_or_else(|| PathBuf::from("output.s"));
                let assembly = self.compile_to_assembly(&llvm_ir)?;
                std::fs::write(output_path, assembly)?;
            }
            Some("obj") => {
                let output_path = output.unwrap_or_else(|| PathBuf::from("output.o"));
                let object_code = self.compile_to_object(&llvm_ir)?;
                std::fs::write(output_path, object_code)?;
            }
            _ => {
                let output_path = output.unwrap_or_else(|| PathBuf::from("output"));
                let executable = self.compile_to_executable(&llvm_ir)?;
                std::fs::write(output_path, executable)?;
            }
        }

        Ok(())
    }

    fn generate_llvm_ir(&self, module: &Module) -> Result<String> {
        let mut ir = String::new();

        ir.push_str(&format!("target triple = \"{}\"\n\n", self.target_triple));

        for global in &module.globals {
            ir.push_str(&self.generate_global_variable(global));
            ir.push('\n');
        }

        for type_def in &module.types {
            ir.push_str(&self.generate_type_definition(type_def));
            ir.push('\n');
        }

        for function in &module.functions {
            ir.push_str(&self.generate_function(function)?);
            ir.push_str("\n\n");
        }

        Ok(ir)
    }

    fn generate_global_variable(&self, global: &crate::ir::GlobalVariable) -> String {
        let mut result = String::new();
        
        result.push_str(&format!("@{} = ", global.name));
        
        if global.is_constant {
            result.push_str("constant ");
        } else {
            result.push_str("global ");
        }
        
        result.push_str(&self.type_to_string(&global.ty));
        
        if let Some(initializer) = &global.initializer {
            result.push(' ');
            result.push_str(&self.constant_to_string(initializer));
        } else {
            result.push_str(" zeroinitializer");
        }
        
        result
    }

    fn generate_type_definition(&self, type_def: &crate::ir::TypeDefinition) -> String {
        format!("%{} = type {}", type_def.name, self.type_to_string(&type_def.ty))
    }

    fn generate_function(&self, function: &IRFunction) -> Result<String> {
        let mut result = String::new();
        
        result.push_str("define ");
        result.push_str(&self.type_to_string(&function.return_type));
        result.push_str(&format!(" @{}(", function.name));
        
        for (i, param) in function.params.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }
            result.push_str(&self.type_to_string(&param.ty));
            result.push_str(&format!(" %{}", param.name));
        }
        
        result.push_str(") {\n");
        
        for block in &function.blocks {
            result.push_str(&self.generate_basic_block(block)?);
        }
        
        result.push('}');
        Ok(result)
    }

    fn generate_basic_block(&self, block: &BasicBlock) -> Result<String> {
        let mut result = String::new();
        
        result.push_str(&format!("{}:\n", block.label));
        
        for instruction in &block.instructions {
            result.push_str("  ");
            result.push_str(&self.generate_instruction(instruction)?);
            result.push('\n');
        }
        
        result.push_str("  ");
        result.push_str(&self.generate_terminator(&block.terminator)?);
        result.push('\n');
        
        Ok(result)
    }

    fn generate_instruction(&self, instruction: &Instruction) -> Result<String> {
        match instruction {
            Instruction::Alloca { result, ty } => {
                Ok(format!("{} = alloca {}", result, self.type_to_string(ty)))
            }
            Instruction::Load { result, ty, ptr } => {
                Ok(format!("{} = load {}, {} {}", 
                    result, 
                    self.type_to_string(ty), 
                    self.type_to_string(&IRType::Pointer(Box::new(ty.clone()))),
                    ptr
                ))
            }
            Instruction::Store { ty, value, ptr } => {
                Ok(format!("store {} {}, {} {}", 
                    self.type_to_string(ty), 
                    value,
                    self.type_to_string(&IRType::Pointer(Box::new(ty.clone()))),
                    ptr
                ))
            }
            Instruction::Add { result, ty, left, right } => {
                Ok(format!("{} = add {} {}, {}", result, self.type_to_string(ty), left, right))
            }
            Instruction::Sub { result, ty, left, right } => {
                Ok(format!("{} = sub {} {}, {}", result, self.type_to_string(ty), left, right))
            }
            Instruction::Mul { result, ty, left, right } => {
                Ok(format!("{} = mul {} {}, {}", result, self.type_to_string(ty), left, right))
            }
            Instruction::Div { result, ty, left, right } => {
                Ok(format!("{} = sdiv {} {}, {}", result, self.type_to_string(ty), left, right))
            }
            Instruction::Rem { result, ty, left, right } => {
                Ok(format!("{} = srem {} {}, {}", result, self.type_to_string(ty), left, right))
            }
            Instruction::And { result, ty, left, right } => {
                Ok(format!("{} = and {} {}, {}", result, self.type_to_string(ty), left, right))
            }
            Instruction::Or { result, ty, left, right } => {
                Ok(format!("{} = or {} {}, {}", result, self.type_to_string(ty), left, right))
            }
            Instruction::Xor { result, ty, left, right } => {
                Ok(format!("{} = xor {} {}, {}", result, self.type_to_string(ty), left, right))
            }
            Instruction::Shl { result, ty, left, right } => {
                Ok(format!("{} = shl {} {}, {}", result, self.type_to_string(ty), left, right))
            }
            Instruction::Shr { result, ty, left, right } => {
                Ok(format!("{} = lshr {} {}, {}", result, self.type_to_string(ty), left, right))
            }
            Instruction::ICmp { result, condition, ty, left, right } => {
                let cond_str = match condition {
                    crate::ir::instruction::ICmpCondition::Eq => "eq",
                    crate::ir::instruction::ICmpCondition::Ne => "ne",
                    crate::ir::instruction::ICmpCondition::Slt => "slt",
                    crate::ir::instruction::ICmpCondition::Sle => "sle",
                    crate::ir::instruction::ICmpCondition::Sgt => "sgt",
                    crate::ir::instruction::ICmpCondition::Sge => "sge",
                    crate::ir::instruction::ICmpCondition::Ult => "ult",
                    crate::ir::instruction::ICmpCondition::Ule => "ule",
                    crate::ir::instruction::ICmpCondition::Ugt => "ugt",
                    crate::ir::instruction::ICmpCondition::Uge => "uge",
                };
                Ok(format!("{} = icmp {} {} {}, {}", result, cond_str, self.type_to_string(ty), left, right))
            }
            Instruction::Call { result, function, args } => {
                let mut call_str = String::new();
                
                if let Some(result_var) = result {
                    call_str.push_str(&format!("{} = ", result_var));
                }
                
                call_str.push_str(&format!("call {} @{}(", self.type_to_string(&IRType::I32), function));
                
                for (i, (ty, arg)) in args.iter().enumerate() {
                    if i > 0 {
                        call_str.push_str(", ");
                    }
                    call_str.push_str(&format!("{} {}", self.type_to_string(ty), arg));
                }
                
                call_str.push(')');
                Ok(call_str)
            }
            Instruction::BitCast { result, value, from_ty, to_ty } => {
                Ok(format!("{} = bitcast {} {} to {}", 
                    result, 
                    self.type_to_string(from_ty), 
                    value, 
                    self.type_to_string(to_ty)
                ))
            }
            Instruction::GetElementPtr { result, ty, ptr, indices } => {
                let mut gep_str = format!("{} = getelementptr {}, {} {}", 
                    result, 
                    self.type_to_string(ty), 
                    self.type_to_string(&IRType::Pointer(Box::new(ty.clone()))),
                    ptr
                );
                
                for index in indices {
                    gep_str.push_str(&format!(", i32 {}", index));
                }
                
                Ok(gep_str)
            }
            Instruction::Phi { result, ty, values } => {
                let mut phi_str = format!("{} = phi {} ", result, self.type_to_string(ty));
                
                for (i, (value, block)) in values.iter().enumerate() {
                    if i > 0 {
                        phi_str.push_str(", ");
                    }
                    phi_str.push_str(&format!("[ {}, %{} ]", value, block));
                }
                
                Ok(phi_str)
            }
            _ => Ok(format!("; Unimplemented instruction: {:?}", instruction)),
        }
    }

    fn generate_terminator(&self, terminator: &Terminator) -> Result<String> {
        match terminator {
            Terminator::Return { value } => {
                if let Some((ty, val)) = value {
                    Ok(format!("ret {} {}", self.type_to_string(ty), val))
                } else {
                    Ok("ret void".to_string())
                }
            }
            Terminator::Branch { dest } => {
                Ok(format!("br label %{}", dest))
            }
            Terminator::ConditionalBranch { condition, true_dest, false_dest } => {
                Ok(format!("br i1 {}, label %{}, label %{}", condition, true_dest, false_dest))
            }
            Terminator::Switch { value, default_dest, cases } => {
                let mut switch_str = format!("switch i32 {}, label %{} [", value, default_dest);
                
                for (constant, dest) in cases {
                    switch_str.push_str(&format!(" {} {}, label %{}", 
                        self.constant_to_string(constant), 
                        self.constant_to_string(constant),
                        dest
                    ));
                }
                
                switch_str.push_str(" ]");
                Ok(switch_str)
            }
            Terminator::Unreachable => {
                Ok("unreachable".to_string())
            }
        }
    }

    fn type_to_string(&self, ty: &IRType) -> String {
        match ty {
            IRType::Void => "void".to_string(),
            IRType::I1 => "i1".to_string(),
            IRType::I8 => "i8".to_string(),
            IRType::I16 => "i16".to_string(),
            IRType::I32 => "i32".to_string(),
            IRType::I64 => "i64".to_string(),
            IRType::I128 => "i128".to_string(),
            IRType::F32 => "float".to_string(),
            IRType::F64 => "double".to_string(),
            IRType::Pointer(inner) => format!("{}*", self.type_to_string(inner)),
            IRType::Array { element_type, size } => {
                format!("[{} x {}]", size, self.type_to_string(element_type))
            }
            IRType::Struct { fields } => {
                let field_types: Vec<String> = fields.iter()
                    .map(|f| self.type_to_string(f))
                    .collect();
                format!("{{ {} }}", field_types.join(", "))
            }
            IRType::Function { params, return_type } => {
                let param_types: Vec<String> = params.iter()
                    .map(|p| self.type_to_string(p))
                    .collect();
                format!("{} ({})", self.type_to_string(return_type), param_types.join(", "))
            }
        }
    }

    fn constant_to_string(&self, constant: &Constant) -> String {
        match constant {
            Constant::Integer { value, .. } => value.to_string(),
            Constant::Float { value, .. } => value.to_string(),
            Constant::String(s) => format!("c\"{}\\00\"", s),
            Constant::Boolean(b) => if *b { "1" } else { "0" }.to_string(),
            Constant::Null => "null".to_string(),
            Constant::Undefined => "undef".to_string(),
        }
    }

    fn compile_to_assembly(&self, llvm_ir: &str) -> Result<String> {
        Ok(format!("; Assembly for LLVM IR:\n{}", llvm_ir))
    }

    fn compile_to_object(&self, llvm_ir: &str) -> Result<Vec<u8>> {
        Ok(llvm_ir.as_bytes().to_vec())
    }

    fn compile_to_executable(&self, llvm_ir: &str) -> Result<Vec<u8>> {
        Ok(llvm_ir.as_bytes().to_vec())
    }
}

