use crate::ir::{Module, IRFunction, IRType, Instruction, Terminator, ICmpCondition, FCmpCondition};
use anyhow::{Result, bail};
use inkwell::context::Context;
use inkwell::builder::Builder;
use inkwell::module::Module as LLVMModule;
use inkwell::values::{FunctionValue, BasicValueEnum, PointerValue};
use inkwell::types::{BasicTypeEnum, BasicMetadataTypeEnum};
use inkwell::basic_block::BasicBlock;
use inkwell::IntPredicate;
use inkwell::FloatPredicate;
use inkwell::targets::{Target, TargetMachine, RelocMode, CodeModel, FileType, InitializationConfig};
use inkwell::OptimizationLevel;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct LLVMCodegen<'ctx> {
    context: &'ctx Context,
    module: LLVMModule<'ctx>,
    builder: Builder<'ctx>,
    optimization_level: u8,
    values: HashMap<String, BasicValueEnum<'ctx>>,
    blocks: HashMap<String, BasicBlock<'ctx>>,
}

impl<'ctx> LLVMCodegen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        
        Self {
            context,
            module,
            builder,
            optimization_level: 2,
            values: HashMap::new(),
            blocks: HashMap::new(),
        }
    }

    pub fn set_optimization_level(&mut self, level: u8) {
        self.optimization_level = level.min(3);
    }

    pub fn generate(
        &mut self,
        ir_module: &Module,
        output: Option<PathBuf>,
        _emit: Option<String>,
    ) -> Result<()> {
        // Generate LLVM IR for all functions
        for function in &ir_module.functions {
            self.compile_function(function)?;
        }

        // Apply LLVM optimization passes if optimization level > 0
        if self.optimization_level > 0 {
            self.run_optimization_passes()?;
        }

        // Emit object file if output path is provided
        if let Some(output_path) = output {
            self.compile_to_object(output_path)?;
        }

        Ok(())
    }

    fn run_optimization_passes(&self) -> Result<()> {
        use inkwell::passes::{PassManager, PassManagerBuilder};

        // Create a pass manager for the module
        let pass_manager_builder = PassManagerBuilder::create();
        
        // Set optimization level
        let opt_level = match self.optimization_level {
            0 => 0,
            1 => 1,
            2 => 2,
            _ => 3,
        };
        pass_manager_builder.set_optimization_level(OptimizationLevel::from(opt_level));

        // Create function pass manager
        let fpm = PassManager::create(&self.module);
        pass_manager_builder.populate_function_pass_manager(&fpm);

        // Run passes on all functions
        for function in self.module.get_functions() {
            fpm.run_on(&function);
        }

        // Create module pass manager
        let mpm = PassManager::create(());
        pass_manager_builder.populate_module_pass_manager(&mpm);
        mpm.run_on(&self.module);

        Ok(())
    }

    fn compile_function(&mut self, function: &IRFunction) -> Result<FunctionValue<'ctx>> {
        // Clear state for new function
        self.values.clear();
        self.blocks.clear();

        // Create function signature
        let param_types: Vec<BasicMetadataTypeEnum> = function.params
            .iter()
            .map(|p| self.get_llvm_type(&p.ty).into())
            .collect();

        let return_type = self.get_llvm_type(&function.return_type);
        
        let fn_type = if matches!(function.return_type, IRType::Void) {
            self.context.void_type().fn_type(&param_types, false)
        } else {
            return_type.fn_type(&param_types, false)
        };

        let fn_value = self.module.add_function(&function.name, fn_type, None);

        if function.blocks.is_empty() {
            return Ok(fn_value);
        }

        // Pre-create all basic blocks before generating instructions
        for block in &function.blocks {
            let llvm_block = self.context.append_basic_block(fn_value, &block.label);
            self.blocks.insert(block.label.clone(), llvm_block);
        }

        // Position at the first block (entry block)
        if let Some(first_block) = function.blocks.first() {
            if let Some(entry_block) = self.blocks.get(&first_block.label) {
                self.builder.position_at_end(*entry_block);
            }
        }

        // Store function parameters as values
        for (i, param) in function.params.iter().enumerate() {
            if let Some(arg) = fn_value.get_nth_param(i as u32) {
                self.values.insert(param.name.clone(), arg);
            }
        }

        // Generate instructions for each block
        for block in &function.blocks {
            // Position builder at this block
            if let Some(llvm_block) = self.blocks.get(&block.label) {
                self.builder.position_at_end(*llvm_block);
            }

            // Compile all instructions in the block
            for instruction in &block.instructions {
                self.compile_instruction(instruction)?;
            }

            // Compile the terminator
            self.compile_terminator(&block.terminator, fn_value)?;
        }

        Ok(fn_value)
    }

    fn compile_instruction(&mut self, instruction: &Instruction) -> Result<()> {
        match instruction {
            Instruction::Alloca { result, ty } => {
                let llvm_type = self.get_llvm_type(ty);
                let alloca = self.builder.build_alloca(llvm_type, result)?;
                self.values.insert(result.clone(), alloca.into());
            }
            Instruction::Store { value, ptr } => {
                if let (Some(val), Some(BasicValueEnum::PointerValue(ptr_val))) = 
                    (self.get_value(value), self.values.get(ptr)) {
                    self.builder.build_store(*ptr_val, val)?;
                }
            }
            Instruction::Load { result, ptr } => {
                if let Some(BasicValueEnum::PointerValue(ptr_val)) = self.values.get(ptr) {
                    let loaded = self.builder.build_load(self.context.i32_type(), *ptr_val, result)?;
                    self.values.insert(result.clone(), loaded);
                }
            }
            Instruction::Add { result, left, right, ty } => {
                if let (Some(lhs), Some(rhs)) = (self.get_value(left), self.get_value(right)) {
                    let add_result = match (lhs, rhs) {
                        (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                            self.builder.build_int_add(l, r, result)?.into()
                        }
                        (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                            self.builder.build_float_add(l, r, result)?.into()
                        }
                        _ => bail!("Type mismatch in add instruction"),
                    };
                    self.values.insert(result.clone(), add_result);
                }
            }
            Instruction::Sub { result, left, right, ty } => {
                if let (Some(lhs), Some(rhs)) = (self.get_value(left), self.get_value(right)) {
                    let sub_result = match (lhs, rhs) {
                        (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                            self.builder.build_int_sub(l, r, result)?.into()
                        }
                        (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                            self.builder.build_float_sub(l, r, result)?.into()
                        }
                        _ => bail!("Type mismatch in sub instruction"),
                    };
                    self.values.insert(result.clone(), sub_result);
                }
            }
            Instruction::Mul { result, left, right, ty } => {
                if let (Some(lhs), Some(rhs)) = (self.get_value(left), self.get_value(right)) {
                    let mul_result = match (lhs, rhs) {
                        (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                            self.builder.build_int_mul(l, r, result)?.into()
                        }
                        (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                            self.builder.build_float_mul(l, r, result)?.into()
                        }
                        _ => bail!("Type mismatch in mul instruction"),
                    };
                    self.values.insert(result.clone(), mul_result);
                }
            }
            Instruction::Div { result, left, right, ty } => {
                if let (Some(lhs), Some(rhs)) = (self.get_value(left), self.get_value(right)) {
                    let div_result = match (lhs, rhs) {
                        (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                            self.builder.build_int_signed_div(l, r, result)?.into()
                        }
                        (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                            self.builder.build_float_div(l, r, result)?.into()
                        }
                        _ => bail!("Type mismatch in div instruction"),
                    };
                    self.values.insert(result.clone(), div_result);
                }
            }
            Instruction::ICmp { result, condition, left, right } => {
                if let (Some(BasicValueEnum::IntValue(lhs)), Some(BasicValueEnum::IntValue(rhs))) = 
                    (self.get_value(left), self.get_value(right)) {
                    let predicate = self.get_int_predicate(condition);
                    let cmp_result = self.builder.build_int_compare(predicate, lhs, rhs, result)?;
                    self.values.insert(result.clone(), cmp_result.into());
                }
            }
            Instruction::Call { result, func, args } => {
                if let Some(function) = self.module.get_function(func) {
                    let arg_values: Vec<BasicMetadataTypeEnum> = args.iter()
                        .filter_map(|arg| self.get_value(arg))
                        .map(|v| v.into())
                        .collect();
                    
                    let call_result = self.builder.build_call(function, &arg_values, "call")?;
                    
                    if let Some(res_name) = result {
                        if let Some(val) = call_result.try_as_basic_value().left() {
                            self.values.insert(res_name.clone(), val);
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn compile_terminator(&mut self, terminator: &Terminator, current_fn: FunctionValue<'ctx>) -> Result<()> {
        match terminator {
            Terminator::Ret { value: Some(val) } => {
                if let Some(ret_val) = self.get_value(val) {
                    self.builder.build_return(Some(&ret_val))?;
                } else {
                    bail!("Return value '{}' not found", val);
                }
            }
            Terminator::Ret { value: None } => {
                self.builder.build_return(None)?;
            }
            Terminator::Br { target } => {
                if let Some(target_block) = self.blocks.get(target) {
                    self.builder.build_unconditional_branch(*target_block)?;
                } else {
                    bail!("Branch target '{}' not found", target);
                }
            }
            Terminator::CondBr { condition, true_target, false_target } => {
                let cond_val = self.get_value(condition)
                    .ok_or_else(|| anyhow::anyhow!("Condition value '{}' not found", condition))?;
                
                let true_block = self.blocks.get(true_target)
                    .ok_or_else(|| anyhow::anyhow!("True branch target '{}' not found", true_target))?;
                
                let false_block = self.blocks.get(false_target)
                    .ok_or_else(|| anyhow::anyhow!("False branch target '{}' not found", false_target))?;
                
                if let BasicValueEnum::IntValue(cond_int) = cond_val {
                    self.builder.build_conditional_branch(cond_int, *true_block, *false_block)?;
                } else {
                    bail!("Condition must be an integer value");
                }
            }
            Terminator::Unreachable => {
                self.builder.build_unreachable()?;
            }
        }
        Ok(())
    }

    fn get_value(&self, name: &str) -> Option<BasicValueEnum<'ctx>> {
        if let Some(val) = self.values.get(name) {
            Some(*val)
        } else if let Ok(int_val) = name.parse::<i64>() {
            Some(self.context.i32_type().const_int(int_val as u64, true).into())
        } else if let Ok(float_val) = name.parse::<f64>() {
            Some(self.context.f64_type().const_float(float_val).into())
        } else {
            None
        }
    }

    fn get_llvm_type(&self, ty: &IRType) -> BasicTypeEnum<'ctx> {
        match ty {
            IRType::I1 => self.context.bool_type().into(),
            IRType::I8 => self.context.i8_type().into(),
            IRType::I16 => self.context.i16_type().into(),
            IRType::I32 => self.context.i32_type().into(),
            IRType::I64 => self.context.i64_type().into(),
            IRType::I128 => self.context.i128_type().into(),
            IRType::F32 => self.context.f32_type().into(),
            IRType::F64 => self.context.f64_type().into(),
            IRType::Pointer(inner) => {
                let inner_type = self.get_llvm_type(inner);
                inner_type.ptr_type(inkwell::AddressSpace::default()).into()
            }
            IRType::Array { element_type, size } => {
                let elem_type = self.get_llvm_type(element_type);
                elem_type.array_type(*size as u32).into()
            }
            IRType::Struct { fields } => {
                let field_types: Vec<BasicTypeEnum> = fields.iter()
                    .map(|f| self.get_llvm_type(f))
                    .collect();
                self.context.struct_type(&field_types, false).into()
            }
            _ => self.context.i32_type().into(),
        }
    }

    fn get_int_predicate(&self, condition: &ICmpCondition) -> IntPredicate {
        match condition {
            ICmpCondition::Eq => IntPredicate::EQ,
            ICmpCondition::Ne => IntPredicate::NE,
            ICmpCondition::Slt => IntPredicate::SLT,
            ICmpCondition::Sle => IntPredicate::SLE,
            ICmpCondition::Sgt => IntPredicate::SGT,
            ICmpCondition::Sge => IntPredicate::SGE,
            ICmpCondition::Ult => IntPredicate::ULT,
            ICmpCondition::Ule => IntPredicate::ULE,
            ICmpCondition::Ugt => IntPredicate::UGT,
            ICmpCondition::Uge => IntPredicate::UGE,
        }
    }

    fn compile_to_object(&self, output: PathBuf) -> Result<()> {
        // Initialize all LLVM targets
        Target::initialize_all(&InitializationConfig::default());
        
        // Get the target triple for the host machine
        let target_triple = TargetMachine::get_default_triple();
        
        // Create target from triple
        let target = Target::from_triple(&target_triple)
            .map_err(|e| anyhow::anyhow!("Failed to create target from triple '{}': {}", target_triple, e))?;
        
        // Map optimization level
        let opt_level = match self.optimization_level {
            0 => OptimizationLevel::None,
            1 => OptimizationLevel::Less,
            2 => OptimizationLevel::Default,
            _ => OptimizationLevel::Aggressive,
        };
        
        // Get CPU and features for the host
        let cpu = TargetMachine::get_host_cpu_name();
        let features = TargetMachine::get_host_cpu_features();
        
        // Create target machine with proper configuration
        let target_machine = target.create_target_machine(
            &target_triple,
            cpu.to_str().unwrap_or("generic"),
            features.to_str().unwrap_or(""),
            opt_level,
            RelocMode::PIC,  // Position Independent Code for better compatibility
            CodeModel::Default,
        ).ok_or_else(|| anyhow::anyhow!("Failed to create target machine for triple '{}'", target_triple))?;
        
        // Set the target data layout and triple on the module
        self.module.set_data_layout(&target_machine.get_target_data().get_data_layout());
        self.module.set_triple(&target_triple);
        
        // Write object file
        target_machine.write_to_file(&self.module, FileType::Object, &output)
            .map_err(|e| anyhow::anyhow!("Failed to write object file to '{}': {}", output.display(), e))?;
        
        Ok(())
    }
}

pub fn create_codegen_context() -> Context {
    Context::create()
}
