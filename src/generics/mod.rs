use std::collections::HashMap;
use crate::parser::Type;

pub struct GenericResolver {
    type_parameters: HashMap<String, TypeParameter>,
    monomorphizations: HashMap<String, Vec<MonomorphizedInstance>>,
}

#[derive(Debug, Clone)]
pub struct TypeParameter {
    pub name: String,
    pub bounds: Vec<String>,
    pub default: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct MonomorphizedInstance {
    pub generic_name: String,
    pub type_args: Vec<Type>,
    pub concrete_name: String,
}

impl GenericResolver {
    pub fn new() -> Self {
        Self {
            type_parameters: HashMap::new(),
            monomorphizations: HashMap::new(),
        }
    }
    
    pub fn register_type_parameter(&mut self, param: TypeParameter) {
        self.type_parameters.insert(param.name.clone(), param);
    }
    
    pub fn resolve_generic_type(&self, name: &str, args: &[Type]) -> Result<Type, String> {
        if args.is_empty() {
            return Ok(Type::Custom(name.to_string()));
        }
        
        let concrete_name = format!("{}<{}>", name, 
            args.iter()
                .map(|t| format!("{:?}", t))
                .collect::<Vec<_>>()
                .join(", ")
        );
        
        Ok(Type::Custom(concrete_name))
    }
    
    pub fn monomorphize(
        &mut self,
        generic_name: &str,
        type_args: Vec<Type>,
    ) -> Result<String, String> {
        let concrete_name = self.generate_concrete_name(generic_name, &type_args);
        
        let instance = MonomorphizedInstance {
            generic_name: generic_name.to_string(),
            type_args: type_args.clone(),
            concrete_name: concrete_name.clone(),
        };
        
        self.monomorphizations
            .entry(generic_name.to_string())
            .or_insert_with(Vec::new)
            .push(instance);
        
        Ok(concrete_name)
    }
    
    fn generate_concrete_name(&self, generic_name: &str, type_args: &[Type]) -> String {
        let args_str = type_args
            .iter()
            .map(|t| self.type_to_mangled_string(t))
            .collect::<Vec<_>>()
            .join("_");
        
        format!("{}_{}", generic_name, args_str)
    }
    
    fn type_to_mangled_string(&self, ty: &Type) -> String {
        match ty {
            Type::I32 => "i32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Char => "char".to_string(),
            Type::String => "String".to_string(),
            Type::Custom(name) => name.replace('<', "_").replace('>', "_").replace(',', "_"),
            _ => "unknown".to_string(),
        }
    }
    
    pub fn substitute_type_params(
        &self,
        ty: &Type,
        substitutions: &HashMap<String, Type>,
    ) -> Type {
        match ty {
            Type::Custom(name) => {
                substitutions.get(name).cloned().unwrap_or_else(|| ty.clone())
            }
            _ => ty.clone(),
        }
    }
    
    pub fn check_type_bounds(
        &self,
        ty: &Type,
        param_name: &str,
    ) -> Result<(), String> {
        if let Some(param) = self.type_parameters.get(param_name) {
            for bound in &param.bounds {
                if !self.type_implements_trait(ty, bound) {
                    return Err(format!(
                        "Type {:?} does not implement required trait '{}'",
                        ty, bound
                    ));
                }
            }
        }
        Ok(())
    }
    
    fn type_implements_trait(&self, _ty: &Type, _trait_name: &str) -> bool {
        true
    }
    
    pub fn get_monomorphizations(&self, generic_name: &str) -> Vec<&MonomorphizedInstance> {
        self.monomorphizations
            .get(generic_name)
            .map(|instances| instances.iter().collect())
            .unwrap_or_default()
    }
}

impl Default for GenericResolver {
    fn default() -> Self {
        Self::new()
    }
}

pub struct VarianceChecker {
    variances: HashMap<String, Variance>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Variance {
    Covariant,
    Contravariant,
    Invariant,
    Bivariant,
}

impl VarianceChecker {
    pub fn new() -> Self {
        Self {
            variances: HashMap::new(),
        }
    }
    
    pub fn compute_variance(&mut self, type_param: &str, usage_context: VarianceContext) -> Variance {
        match usage_context {
            VarianceContext::OutputPosition => Variance::Covariant,
            VarianceContext::InputPosition => Variance::Contravariant,
            VarianceContext::BothPositions => Variance::Invariant,
            VarianceContext::Unused => Variance::Bivariant,
        }
    }
    
    pub fn check_subtyping(&self, sub: &Type, sup: &Type, variance: Variance) -> bool {
        match variance {
            Variance::Covariant => self.is_subtype(sub, sup),
            Variance::Contravariant => self.is_subtype(sup, sub),
            Variance::Invariant => sub == sup,
            Variance::Bivariant => true,
        }
    }
    
    fn is_subtype(&self, _sub: &Type, _sup: &Type) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VarianceContext {
    OutputPosition,
    InputPosition,
    BothPositions,
    Unused,
}

impl Default for VarianceChecker {
    fn default() -> Self {
        Self::new()
    }
}
