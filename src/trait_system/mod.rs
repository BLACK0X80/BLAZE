use std::collections::HashMap;
use crate::parser::Type;

pub struct TraitRegistry {
    traits: HashMap<String, TraitDefinition>,
    implementations: HashMap<(String, Type), TraitImpl>,
}

#[derive(Debug, Clone)]
pub struct TraitDefinition {
    pub name: String,
    pub methods: Vec<TraitMethod>,
    pub associated_types: Vec<String>,
    pub super_traits: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TraitMethod {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Option<Type>,
    pub default_impl: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TraitImpl {
    pub trait_name: String,
    pub for_type: Type,
    pub methods: HashMap<String, String>,
}

impl TraitRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            traits: HashMap::new(),
            implementations: HashMap::new(),
        };
        
        registry.register_builtin_traits();
        registry
    }
    
    fn register_builtin_traits(&mut self) {
        self.register_trait(TraitDefinition {
            name: "Clone".to_string(),
            methods: vec![
                TraitMethod {
                    name: "clone".to_string(),
                    params: vec![("self".to_string(), Type::Custom("Self".to_string()))],
                    return_type: Some(Type::Custom("Self".to_string())),
                    default_impl: None,
                }
            ],
            associated_types: vec![],
            super_traits: vec![],
        });
        
        self.register_trait(TraitDefinition {
            name: "Copy".to_string(),
            methods: vec![],
            associated_types: vec![],
            super_traits: vec!["Clone".to_string()],
        });
        
        self.register_trait(TraitDefinition {
            name: "Debug".to_string(),
            methods: vec![
                TraitMethod {
                    name: "fmt".to_string(),
                    params: vec![
                        ("self".to_string(), Type::Custom("Self".to_string())),
                        ("f".to_string(), Type::Custom("Formatter".to_string())),
                    ],
                    return_type: Some(Type::Custom("Result".to_string())),
                    default_impl: None,
                }
            ],
            associated_types: vec![],
            super_traits: vec![],
        });
        
        self.register_trait(TraitDefinition {
            name: "Display".to_string(),
            methods: vec![
                TraitMethod {
                    name: "fmt".to_string(),
                    params: vec![
                        ("self".to_string(), Type::Custom("Self".to_string())),
                        ("f".to_string(), Type::Custom("Formatter".to_string())),
                    ],
                    return_type: Some(Type::Custom("Result".to_string())),
                    default_impl: None,
                }
            ],
            associated_types: vec![],
            super_traits: vec![],
        });
        
        self.register_trait(TraitDefinition {
            name: "Iterator".to_string(),
            methods: vec![
                TraitMethod {
                    name: "next".to_string(),
                    params: vec![("self".to_string(), Type::Custom("Self".to_string()))],
                    return_type: Some(Type::Custom("Option<Item>".to_string())),
                    default_impl: None,
                }
            ],
            associated_types: vec!["Item".to_string()],
            super_traits: vec![],
        });
        
        self.register_trait(TraitDefinition {
            name: "Ord".to_string(),
            methods: vec![
                TraitMethod {
                    name: "cmp".to_string(),
                    params: vec![
                        ("self".to_string(), Type::Custom("Self".to_string())),
                        ("other".to_string(), Type::Custom("Self".to_string())),
                    ],
                    return_type: Some(Type::Custom("Ordering".to_string())),
                    default_impl: None,
                }
            ],
            associated_types: vec![],
            super_traits: vec!["PartialOrd".to_string(), "Eq".to_string()],
        });
        
        self.register_trait(TraitDefinition {
            name: "PartialOrd".to_string(),
            methods: vec![
                TraitMethod {
                    name: "partial_cmp".to_string(),
                    params: vec![
                        ("self".to_string(), Type::Custom("Self".to_string())),
                        ("other".to_string(), Type::Custom("Self".to_string())),
                    ],
                    return_type: Some(Type::Custom("Option<Ordering>".to_string())),
                    default_impl: None,
                }
            ],
            associated_types: vec![],
            super_traits: vec!["PartialEq".to_string()],
        });
        
        self.register_trait(TraitDefinition {
            name: "Eq".to_string(),
            methods: vec![],
            associated_types: vec![],
            super_traits: vec!["PartialEq".to_string()],
        });
        
        self.register_trait(TraitDefinition {
            name: "PartialEq".to_string(),
            methods: vec![
                TraitMethod {
                    name: "eq".to_string(),
                    params: vec![
                        ("self".to_string(), Type::Custom("Self".to_string())),
                        ("other".to_string(), Type::Custom("Self".to_string())),
                    ],
                    return_type: Some(Type::Bool),
                    default_impl: None,
                }
            ],
            associated_types: vec![],
            super_traits: vec![],
        });
    }
    
    pub fn register_trait(&mut self, trait_def: TraitDefinition) {
        self.traits.insert(trait_def.name.clone(), trait_def);
    }
    
    pub fn register_impl(&mut self, impl_def: TraitImpl) {
        let key = (impl_def.trait_name.clone(), impl_def.for_type.clone());
        self.implementations.insert(key, impl_def);
    }
    
    pub fn get_trait(&self, name: &str) -> Option<&TraitDefinition> {
        self.traits.get(name)
    }
    
    pub fn get_impl(&self, trait_name: &str, for_type: &Type) -> Option<&TraitImpl> {
        self.implementations.get(&(trait_name.to_string(), for_type.clone()))
    }
    
    pub fn has_impl(&self, trait_name: &str, for_type: &Type) -> bool {
        self.get_impl(trait_name, for_type).is_some()
    }
    
    pub fn check_bounds(&self, type_param: &Type, bounds: &[String]) -> Result<(), String> {
        for bound in bounds {
            if !self.has_impl(bound, type_param) {
                return Err(format!(
                    "Type {:?} does not implement trait '{}'",
                    type_param, bound
                ));
            }
        }
        Ok(())
    }
    
    pub fn get_method(&self, trait_name: &str, method_name: &str) -> Option<&TraitMethod> {
        self.traits
            .get(trait_name)?
            .methods
            .iter()
            .find(|m| m.name == method_name)
    }
    
    pub fn resolve_method_call(
        &self,
        receiver_type: &Type,
        method_name: &str,
    ) -> Option<(String, &TraitMethod)> {
        for (trait_name, trait_def) in &self.traits {
            if let Some(method) = trait_def.methods.iter().find(|m| m.name == method_name) {
                if self.has_impl(trait_name, receiver_type) {
                    return Some((trait_name.clone(), method));
                }
            }
        }
        None
    }
}

impl Default for TraitRegistry {
    fn default() -> Self {
        Self::new()
    }
}
