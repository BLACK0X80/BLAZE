use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{FunctionValue, BasicValueEnum, IntValue, FloatValue};
use inkwell::types::{BasicTypeEnum, IntType, FloatType};
use inkwell::IntPredicate;
use std::collections::HashMap;

pub struct LLVMCodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    variables: HashMap<String, BasicValueEnum<'ctx>>,
}

impl<'ctx> LLVMCodeGen<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        
        Self {
            context,
            module,
            builder,
            variables: HashMap::new(),
        }
    }
    
    pub fn create_function(&self, name: &str, param_types: &[BasicTypeEnum<'ctx>], return_type: Option<BasicTypeEnum<'ctx>>) -> FunctionValue<'ctx> {
        let fn_type = match return_type {
            Some(ret_ty) => ret_ty.fn_type(param_types, false),
            None => self.context.void_type().fn_type(param_types, false),
        };
        
        self.module.add_function(name, fn_type, None)
    }
    
    pub fn build_i32_constant(&self, value: i32) -> IntValue<'ctx> {
        self.context.i32_type().const_int(value as u64, false)
    }
    
    pub fn build_i64_constant(&self, value: i64) -> IntValue<'ctx> {
        self.context.i64_type().const_int(value as u64, false)
    }
    
    pub fn build_f64_constant(&self, value: f64) -> FloatValue<'ctx> {
        self.context.f64_type().const_float(value)
    }
    
    pub fn build_add(&self, lhs: IntValue<'ctx>, rhs: IntValue<'ctx>, name: &str) -> IntValue<'ctx> {
        self.builder.build_int_add(lhs, rhs, name).unwrap()
    }
    
    pub fn build_sub(&self, lhs: IntValue<'ctx>, rhs: IntValue<'ctx>, name: &str) -> IntValue<'ctx> {
        self.builder.build_int_sub(lhs, rhs, name).unwrap()
    }
    
    pub fn build_mul(&self, lhs: IntValue<'ctx>, rhs: IntValue<'ctx>, name: &str) -> IntValue<'ctx> {
        self.builder.build_int_mul(lhs, rhs, name).unwrap()
    }
    
    pub fn build_div(&self, lhs: IntValue<'ctx>, rhs: IntValue<'ctx>, name: &str) -> IntValue<'ctx> {
        self.builder.build_int_signed_div(lhs, rhs, name).unwrap()
    }
    
    pub fn build_rem(&self, lhs: IntValue<'ctx>, rhs: IntValue<'ctx>, name: &str) -> IntValue<'ctx> {
        self.builder.build_int_signed_rem(lhs, rhs, name).unwrap()
    }
    
    pub fn build_and(&self, lhs: IntValue<'ctx>, rhs: IntValue<'ctx>, name: &str) -> IntValue<'ctx> {
        self.builder.build_and(lhs, rhs, name).unwrap()
    }
    
    pub fn build_or(&self, lhs: IntValue<'ctx>, rhs: IntValue<'ctx>, name: &str) -> IntValue<'ctx> {
        self.builder.build_or(lhs, rhs, name).unwrap()
    }
    
    pub fn build_xor(&self, lhs: IntValue<'ctx>, rhs: IntValue<'ctx>, name: &str) -> IntValue<'ctx> {
        self.builder.build_xor(lhs, rhs, name).unwrap()
    }
    
    pub fn build_shl(&self, lhs: IntValue<'ctx>, rhs: IntValue<'ctx>, name: &str) -> IntValue<'ctx> {
        self.builder.build_left_shift(lhs, rhs, name).unwrap()
    }
    
    pub fn build_shr(&self, lhs: IntValue<'ctx>, rhs: IntValue<'ctx>, name: &str) -> IntValue<'ctx> {
        self.builder.build_right_shift(lhs, rhs, false, name).unwrap()
    }
    
    pub fn build_neg(&self, value: IntValue<'ctx>, name: &str) -> IntValue<'ctx> {
        self.builder.build_int_neg(value, name).unwrap()
    }
    
    pub fn build_not(&self, value: IntValue<'ctx>, name: &str) -> IntValue<'ctx> {
        self.builder.build_not(value, name).unwrap()
    }
    
    pub fn build_fadd(&self, lhs: FloatValue<'ctx>, rhs: FloatValue<'ctx>, name: &str) -> FloatValue<'ctx> {
        self.builder.build_float_add(lhs, rhs, name).unwrap()
    }
    
    pub fn build_fsub(&self, lhs: FloatValue<'ctx>, rhs: FloatValue<'ctx>, name: &str) -> FloatValue<'ctx> {
        self.builder.build_float_sub(lhs, rhs, name).unwrap()
    }
    
    pub fn build_fmul(&self, lhs: FloatValue<'ctx>, rhs: FloatValue<'ctx>, name: &str) -> FloatValue<'ctx> {
        self.builder.build_float_mul(lhs, rhs, name).unwrap()
    }
    
    pub fn build_fdiv(&self, lhs: FloatValue<'ctx>, rhs: FloatValue<'ctx>, name: &str) -> FloatValue<'ctx> {
        self.builder.build_float_div(lhs, rhs, name).unwrap()
    }
    
    pub fn build_icmp(&self, op: IntPredicate, lhs: IntValue<'ctx>, rhs: IntValue<'ctx>, name: &str) -> IntValue<'ctx> {
        self.builder.build_int_compare(op, lhs, rhs, name).unwrap()
    }
    
    pub fn build_return(&self, value: Option<&dyn inkwell::values::BasicValue<'ctx>>) {
        match value {
            Some(v) => { self.builder.build_return(Some(v)); },
            None => { self.builder.build_return(None); },
        }
    }
    
    pub fn build_alloca(&self, ty: BasicTypeEnum<'ctx>, name: &str) -> inkwell::values::PointerValue<'ctx> {
        self.builder.build_alloca(ty, name).unwrap()
    }
    
    pub fn build_store(&self, ptr: inkwell::values::PointerValue<'ctx>, value: BasicValueEnum<'ctx>) {
        self.builder.build_store(ptr, value).unwrap();
    }
    
    pub fn build_load(&self, ty: BasicTypeEnum<'ctx>, ptr: inkwell::values::PointerValue<'ctx>, name: &str) -> BasicValueEnum<'ctx> {
        self.builder.build_load(ty, ptr, name).unwrap()
    }
    
    pub fn build_call(&self, function: FunctionValue<'ctx>, args: &[BasicValueEnum<'ctx>], name: &str) -> Option<BasicValueEnum<'ctx>> {
        let args_meta: Vec<_> = args.iter().map(|v| (*v).into()).collect();
        let call_site = self.builder.build_call(function, &args_meta, name).unwrap();
        call_site.try_as_basic_value().left()
    }
    
    pub fn get_module(&self) -> &Module<'ctx> {
        &self.module
    }
    
    pub fn i32_type(&self) -> IntType<'ctx> {
        self.context.i32_type()
    }
    
    pub fn i64_type(&self) -> IntType<'ctx> {
        self.context.i64_type()
    }
    
    pub fn f64_type(&self) -> FloatType<'ctx> {
        self.context.f64_type()
    }
    
    pub fn bool_type(&self) -> IntType<'ctx> {
        self.context.bool_type()
    }
    
    pub fn store_variable(&mut self, name: String, value: BasicValueEnum<'ctx>) {
        self.variables.insert(name, value);
    }
    
    pub fn get_variable(&self, name: &str) -> Option<&BasicValueEnum<'ctx>> {
        self.variables.get(name)
    }
}
