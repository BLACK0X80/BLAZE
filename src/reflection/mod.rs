use std::collections::HashMap;
use crate::parser::Type;

pub struct ReflectionSystem {
    type_info: HashMap<String, TypeInfo>,
    function_metadata: HashMap<String, FunctionMetadata>,
}

#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub name: String,
    pub size: usize,
    pub alignment: usize,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<String>,
    pub traits: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub field_type: Type,
    pub offset: usize,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Protected,
}

#[derive(Debug, Clone)]
pub struct FunctionMetadata {
    pub name: String,
    pub param_names: Vec<String>,
    pub param_types: Vec<Type>,
    pub return_type: Option<Type>,
    pub attributes: Vec<String>,
}

impl ReflectionSystem {
    pub fn new() -> Self {
        Self {
            type_info: HashMap::new(),
            function_metadata: HashMap::new(),
        }
    }
    
    pub fn register_type(&mut self, info: TypeInfo) {
        self.type_info.insert(info.name.clone(), info);
    }
    
    pub fn register_function(&mut self, metadata: FunctionMetadata) {
        self.function_metadata.insert(metadata.name.clone(), metadata);
    }
    
    pub fn get_type_info(&self, type_name: &str) -> Option<&TypeInfo> {
        self.type_info.get(type_name)
    }
    
    pub fn get_function_metadata(&self, func_name: &str) -> Option<&FunctionMetadata> {
        self.function_metadata.get(func_name)
    }
    
    pub fn get_field(&self, type_name: &str, field_name: &str) -> Option<&FieldInfo> {
        self.type_info
            .get(type_name)?
            .fields
            .iter()
            .find(|f| f.name == field_name)
    }
    
    pub fn list_methods(&self, type_name: &str) -> Vec<String> {
        self.type_info
            .get(type_name)
            .map(|info| info.methods.clone())
            .unwrap_or_default()
    }
    
    pub fn implements_trait(&self, type_name: &str, trait_name: &str) -> bool {
        self.type_info
            .get(type_name)
            .map(|info| info.traits.contains(&trait_name.to_string()))
            .unwrap_or(false)
    }
    
    pub fn get_all_types(&self) -> Vec<String> {
        self.type_info.keys().cloned().collect()
    }
}

impl Default for ReflectionSystem {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AttributeProcessor {
    attributes: HashMap<String, Vec<Attribute>>,
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<AttributeArg>,
}

#[derive(Debug, Clone)]
pub enum AttributeArg {
    String(String),
    Integer(i64),
    Boolean(bool),
    Named(String, Box<AttributeArg>),
}

impl AttributeProcessor {
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
        }
    }
    
    pub fn add_attribute(&mut self, target: String, attr: Attribute) {
        self.attributes
            .entry(target)
            .or_insert_with(Vec::new)
            .push(attr);
    }
    
    pub fn get_attributes(&self, target: &str) -> Vec<&Attribute> {
        self.attributes
            .get(target)
            .map(|attrs| attrs.iter().collect())
            .unwrap_or_default()
    }
    
    pub fn has_attribute(&self, target: &str, attr_name: &str) -> bool {
        self.attributes
            .get(target)
            .map(|attrs| attrs.iter().any(|a| a.name == attr_name))
            .unwrap_or(false)
    }
}

impl Default for AttributeProcessor {
    fn default() -> Self {
        Self::new()
    }
}
