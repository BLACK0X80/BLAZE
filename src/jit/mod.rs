use std::collections::HashMap;
use std::mem;

pub struct JITEngine {
    compiled_functions: HashMap<String, CompiledFunction>,
    memory_manager: MemoryManager,
}

struct CompiledFunction {
    code: Vec<u8>,
    entry_point: *const u8,
}

struct MemoryManager {
    allocations: Vec<MemoryBlock>,
}

struct MemoryBlock {
    ptr: *mut u8,
    size: usize,
}

impl JITEngine {
    pub fn new() -> Self {
        Self {
            compiled_functions: HashMap::new(),
            memory_manager: MemoryManager::new(),
        }
    }
    
    pub fn compile_and_execute(&mut self, name: String, code: Vec<u8>) -> Result<i64, String> {
        let function = self.compile(name.clone(), code)?;
        self.compiled_functions.insert(name.clone(), function);
        self.execute(&name)
    }
    
    pub fn compile(&mut self, name: String, code: Vec<u8>) -> Result<CompiledFunction, String> {
        let block = self.memory_manager.allocate(code.len())?;
        
        unsafe {
            std::ptr::copy_nonoverlapping(code.as_ptr(), block.ptr, code.len());
        }
        
        Ok(CompiledFunction {
            code,
            entry_point: block.ptr as *const u8,
        })
    }
    
    pub fn execute(&self, name: &str) -> Result<i64, String> {
        let function = self.compiled_functions
            .get(name)
            .ok_or_else(|| format!("Function '{}' not found", name))?;
        
        unsafe {
            let func: extern "C" fn() -> i64 = mem::transmute(function.entry_point);
            Ok(func())
        }
    }
    
    pub fn has_function(&self, name: &str) -> bool {
        self.compiled_functions.contains_key(name)
    }
    
    pub fn remove_function(&mut self, name: &str) -> bool {
        self.compiled_functions.remove(name).is_some()
    }
    
    pub fn clear(&mut self) {
        self.compiled_functions.clear();
        self.memory_manager.clear();
    }
}

impl MemoryManager {
    fn new() -> Self {
        Self {
            allocations: Vec::new(),
        }
    }
    
    #[cfg(unix)]
    fn allocate(&mut self, size: usize) -> Result<&MemoryBlock, String> {
        use libc::{mmap, mprotect, MAP_ANONYMOUS, MAP_PRIVATE, PROT_EXEC, PROT_READ, PROT_WRITE};
        
        let aligned_size = (size + 4095) & !4095;
        
        let ptr = unsafe {
            mmap(
                std::ptr::null_mut(),
                aligned_size,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        
        if ptr == libc::MAP_FAILED {
            return Err("Memory allocation failed".to_string());
        }
        
        unsafe {
            if mprotect(ptr, aligned_size, PROT_READ | PROT_EXEC) != 0 {
                return Err("Memory protection change failed".to_string());
            }
        }
        
        let block = MemoryBlock {
            ptr: ptr as *mut u8,
            size: aligned_size,
        };
        
        self.allocations.push(block);
        Ok(self.allocations.last().unwrap())
    }
    
    #[cfg(windows)]
    fn allocate(&mut self, size: usize) -> Result<&MemoryBlock, String> {
        use winapi::um::memoryapi::VirtualAlloc;
        use winapi::um::winnt::{MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE};
        
        let aligned_size = (size + 4095) & !4095;
        
        let ptr = unsafe {
            VirtualAlloc(
                std::ptr::null_mut(),
                aligned_size,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_EXECUTE_READWRITE,
            )
        };
        
        if ptr.is_null() {
            return Err("Memory allocation failed".to_string());
        }
        
        let block = MemoryBlock {
            ptr: ptr as *mut u8,
            size: aligned_size,
        };
        
        self.allocations.push(block);
        Ok(self.allocations.last().unwrap())
    }
    
    #[cfg(unix)]
    fn clear(&mut self) {
        use libc::munmap;
        
        for block in &self.allocations {
            unsafe {
                munmap(block.ptr as *mut libc::c_void, block.size);
            }
        }
        
        self.allocations.clear();
    }
    
    #[cfg(windows)]
    fn clear(&mut self) {
        use winapi::um::memoryapi::VirtualFree;
        use winapi::um::winnt::MEM_RELEASE;
        
        for block in &self.allocations {
            unsafe {
                VirtualFree(block.ptr as *mut winapi::ctypes::c_void, 0, MEM_RELEASE);
            }
        }
        
        self.allocations.clear();
    }
}

impl Drop for JITEngine {
    fn drop(&mut self) {
        self.clear();
    }
}

impl Default for JITEngine {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Send for JITEngine {}
unsafe impl Sync for JITEngine {}
