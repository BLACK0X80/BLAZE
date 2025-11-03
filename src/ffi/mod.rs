use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double, c_int, c_void};

pub struct FFIRegistry {
    foreign_functions: HashMap<String, ForeignFunction>,
    type_mappings: HashMap<String, CType>,
}

#[derive(Debug, Clone)]
pub struct ForeignFunction {
    pub name: String,
    pub library: String,
    pub symbol: String,
    pub params: Vec<CType>,
    pub return_type: CType,
    pub calling_convention: CallingConvention,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CType {
    Void,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float,
    Double,
    Pointer(Box<CType>),
    Array(Box<CType>, usize),
    Struct(String),
    Function(Vec<CType>, Box<CType>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CallingConvention {
    C,
    Stdcall,
    Fastcall,
    Cdecl,
}

impl FFIRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            foreign_functions: HashMap::new(),
            type_mappings: HashMap::new(),
        };
        
        registry.register_standard_types();
        registry
    }
    
    fn register_standard_types(&mut self) {
        self.type_mappings.insert("i8".to_string(), CType::Int8);
        self.type_mappings.insert("i16".to_string(), CType::Int16);
        self.type_mappings.insert("i32".to_string(), CType::Int32);
        self.type_mappings.insert("i64".to_string(), CType::Int64);
        self.type_mappings.insert("u8".to_string(), CType::UInt8);
        self.type_mappings.insert("u16".to_string(), CType::UInt16);
        self.type_mappings.insert("u32".to_string(), CType::UInt32);
        self.type_mappings.insert("u64".to_string(), CType::UInt64);
        self.type_mappings.insert("f32".to_string(), CType::Float);
        self.type_mappings.insert("f64".to_string(), CType::Double);
    }
    
    pub fn register_foreign_function(&mut self, func: ForeignFunction) {
        self.foreign_functions.insert(func.name.clone(), func);
    }
    
    pub fn get_function(&self, name: &str) -> Option<&ForeignFunction> {
        self.foreign_functions.get(name)
    }
    
    pub fn map_type(&self, blaze_type: &str) -> Option<&CType> {
        self.type_mappings.get(blaze_type)
    }
    
    pub fn generate_wrapper(&self, func: &ForeignFunction) -> String {
        let mut wrapper = String::new();
        
        wrapper.push_str(&format!("extern \"{}\" fn {}(", 
            self.calling_convention_str(func.calling_convention),
            func.name
        ));
        
        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                wrapper.push_str(", ");
            }
            wrapper.push_str(&format!("arg{}: {}", i, self.type_to_rust_str(param)));
        }
        
        wrapper.push_str(") -> ");
        wrapper.push_str(&self.type_to_rust_str(&func.return_type));
        
        wrapper
    }
    
    fn calling_convention_str(&self, conv: CallingConvention) -> &str {
        match conv {
            CallingConvention::C | CallingConvention::Cdecl => "C",
            CallingConvention::Stdcall => "stdcall",
            CallingConvention::Fastcall => "fastcall",
        }
    }
    
    fn type_to_rust_str(&self, ty: &CType) -> String {
        match ty {
            CType::Void => "()".to_string(),
            CType::Int8 => "i8".to_string(),
            CType::Int16 => "i16".to_string(),
            CType::Int32 => "i32".to_string(),
            CType::Int64 => "i64".to_string(),
            CType::UInt8 => "u8".to_string(),
            CType::UInt16 => "u16".to_string(),
            CType::UInt32 => "u32".to_string(),
            CType::UInt64 => "u64".to_string(),
            CType::Float => "f32".to_string(),
            CType::Double => "f64".to_string(),
            CType::Pointer(inner) => format!("*mut {}", self.type_to_rust_str(inner)),
            CType::Array(inner, size) => format!("[{}; {}]", self.type_to_rust_str(inner), size),
            CType::Struct(name) => name.clone(),
            CType::Function(params, ret) => {
                let params_str = params.iter()
                    .map(|p| self.type_to_rust_str(p))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("fn({}) -> {}", params_str, self.type_to_rust_str(ret))
            }
        }
    }
}

impl Default for FFIRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CStringConverter;

impl CStringConverter {
    pub fn to_c_string(s: &str) -> Result<CString, String> {
        CString::new(s).map_err(|e| format!("Invalid C string: {}", e))
    }
    
    pub fn from_c_string(ptr: *const c_char) -> Result<String, String> {
        if ptr.is_null() {
            return Err("Null pointer".to_string());
        }
        
        unsafe {
            CStr::from_ptr(ptr)
                .to_str()
                .map(|s| s.to_string())
                .map_err(|e| format!("Invalid UTF-8: {}", e))
        }
    }
    
    pub fn to_raw_ptr(s: &str) -> Result<*mut c_char, String> {
        let c_string = Self::to_c_string(s)?;
        Ok(c_string.into_raw())
    }
    
    pub fn free_raw_ptr(ptr: *mut c_char) {
        if !ptr.is_null() {
            unsafe {
                let _ = CString::from_raw(ptr);
            }
        }
    }
}

pub struct LibraryLoader {
    loaded_libraries: HashMap<String, *mut c_void>,
}

impl LibraryLoader {
    pub fn new() -> Self {
        Self {
            loaded_libraries: HashMap::new(),
        }
    }
    
    #[cfg(unix)]
    pub fn load_library(&mut self, path: &str) -> Result<(), String> {
        use std::ffi::CString;
        use libc::{dlopen, RTLD_NOW};
        
        let c_path = CString::new(path)
            .map_err(|e| format!("Invalid path: {}", e))?;
        
        let handle = unsafe { dlopen(c_path.as_ptr(), RTLD_NOW) };
        
        if handle.is_null() {
            return Err(format!("Failed to load library: {}", path));
        }
        
        self.loaded_libraries.insert(path.to_string(), handle);
        Ok(())
    }
    
    #[cfg(windows)]
    pub fn load_library(&mut self, path: &str) -> Result<(), String> {
        use std::ffi::CString;
        use winapi::um::libloaderapi::LoadLibraryA;
        
        let c_path = CString::new(path)
            .map_err(|e| format!("Invalid path: {}", e))?;
        
        let handle = unsafe { LoadLibraryA(c_path.as_ptr()) };
        
        if handle.is_null() {
            return Err(format!("Failed to load library: {}", path));
        }
        
        self.loaded_libraries.insert(path.to_string(), handle as *mut c_void);
        Ok(())
    }
    
    #[cfg(unix)]
    pub fn get_symbol(&self, library: &str, symbol: &str) -> Result<*mut c_void, String> {
        use std::ffi::CString;
        use libc::dlsym;
        
        let handle = self.loaded_libraries.get(library)
            .ok_or_else(|| format!("Library not loaded: {}", library))?;
        
        let c_symbol = CString::new(symbol)
            .map_err(|e| format!("Invalid symbol: {}", e))?;
        
        let sym = unsafe { dlsym(*handle, c_symbol.as_ptr()) };
        
        if sym.is_null() {
            return Err(format!("Symbol not found: {}", symbol));
        }
        
        Ok(sym)
    }
    
    #[cfg(windows)]
    pub fn get_symbol(&self, library: &str, symbol: &str) -> Result<*mut c_void, String> {
        use std::ffi::CString;
        use winapi::um::libloaderapi::GetProcAddress;
        
        let handle = self.loaded_libraries.get(library)
            .ok_or_else(|| format!("Library not loaded: {}", library))?;
        
        let c_symbol = CString::new(symbol)
            .map_err(|e| format!("Invalid symbol: {}", e))?;
        
        let sym = unsafe { GetProcAddress(*handle as _, c_symbol.as_ptr()) };
        
        if sym.is_null() {
            return Err(format!("Symbol not found: {}", symbol));
        }
        
        Ok(sym as *mut c_void)
    }
}

impl Default for LibraryLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for LibraryLoader {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            use libc::dlclose;
            for handle in self.loaded_libraries.values() {
                unsafe { dlclose(*handle); }
            }
        }
        
        #[cfg(windows)]
        {
            use winapi::um::libloaderapi::FreeLibrary;
            for handle in self.loaded_libraries.values() {
                unsafe { FreeLibrary(*handle as _); }
            }
        }
    }
}
