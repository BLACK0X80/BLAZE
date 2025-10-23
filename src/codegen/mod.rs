//! Code generation module for the BLAZE compiler.
//!
//! This module orchestrates the complete code generation pipeline, from intermediate
//! representation (IR) to executable binaries. It includes:
//!
//! - LLVM backend integration for machine code generation
//! - Register allocation for efficient code
//! - Platform-specific linking
//!
//! # Examples
//!
//! ```rust,no_run
//! use blaze::codegen::CodeGenerator;
//! use blaze::ir::Module;
//! use std::path::PathBuf;
//!
//! let ir_module = Module::new("my_program");
//! let mut codegen = CodeGenerator::new();
//!
//! // Generate executable with optimization level 2
//! codegen.generate(&ir_module, PathBuf::from("output"), 2)
//!     .expect("Code generation failed");
//! ```

/// LLVM backend for machine code generation
pub mod llvm_backend;

/// Register allocation for efficient code generation
pub mod register_allocator;

/// Platform-specific linker integration
pub mod linker;

pub use llvm_backend::LLVMCodegen;
pub use register_allocator::RegisterAllocator;
pub use linker::Linker;

use crate::ir::Module;
use std::path::PathBuf;
use anyhow::{Result, Context as AnyhowContext};
use inkwell::context::Context;

/// Code generator that orchestrates the complete code generation pipeline.
///
/// The `CodeGenerator` coordinates register allocation, LLVM code generation,
/// and linking to produce executable binaries from IR modules.
pub struct CodeGenerator {
    
}

impl CodeGenerator {
    /// Creates a new code generator instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use blaze::codegen::CodeGenerator;
    ///
    /// let codegen = CodeGenerator::new();
    /// ```
    pub fn new() -> Self {
        Self {}
    }

    /// Generates an executable binary from an IR module.
    ///
    /// This method performs the complete code generation pipeline:
    /// 1. Register allocation for all functions
    /// 2. IR-level optimizations (if optimization level > 0)
    /// 3. LLVM code generation to object file
    /// 4. Platform-specific linking to executable
    ///
    /// # Arguments
    ///
    /// * `module` - The IR module to compile
    /// * `output` - Path where the executable should be written
    /// * `optimization_level` - Optimization level (0-3):
    ///   - 0: No optimization (fastest compilation)
    ///   - 1: Basic optimization
    ///   - 2: Moderate optimization (recommended)
    ///   - 3: Aggressive optimization (slowest compilation, best performance)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if any phase fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Register allocation fails
    /// - LLVM code generation fails
    /// - Linking fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use blaze::codegen::CodeGenerator;
    /// use blaze::ir::Module;
    /// use std::path::PathBuf;
    ///
    /// let ir_module = Module::new("my_program");
    /// let mut codegen = CodeGenerator::new();
    ///
    /// // Generate with moderate optimization
    /// codegen.generate(&ir_module, PathBuf::from("my_program"), 2)
    ///     .expect("Code generation failed");
    /// ```
    pub fn generate(&mut self, module: &Module, output: PathBuf, optimization_level: u8) -> Result<()> {
        // Create LLVM context at start of generate()
        let context = Context::create();
        
        // Perform register allocation
        let mut allocator = RegisterAllocator::new();
        for (idx, function) in module.functions.iter().enumerate() {
            let mut func_clone = function.clone();
            allocator.allocate(&mut func_clone)
                .context(format!("Register allocation failed for function '{}' (index {})", function.name, idx))?;
        }

        // Call LLVMCodegen to generate object file
        let object_file = self.generate_object(module, &context, &output, optimization_level)
            .context(format!("Code generation phase failed for module '{}'", module.name))?;

        // Call Linker to create executable
        self.link_executable(&[object_file], output.clone())
            .context(format!("Linking phase failed for output '{}'", output.display()))?;

        Ok(())
    }

    /// Generates an object file from an IR module.
    ///
    /// This internal method applies IR-level optimizations and uses the LLVM backend
    /// to generate a platform-specific object file.
    ///
    /// # Arguments
    ///
    /// * `module` - The IR module to compile
    /// * `context` - LLVM context for code generation
    /// * `output` - Base path for output (object file will have .o extension)
    /// * `optimization_level` - Optimization level (0-3)
    ///
    /// # Returns
    ///
    /// Returns the path to the generated object file on success.
    fn generate_object(&self, module: &Module, context: &Context, output: &PathBuf, optimization_level: u8) -> Result<PathBuf> {
        // Apply IR-level optimizations before code generation
        let mut optimized_module = module.clone();
        if optimization_level > 0 {
            crate::ir::optimization::optimize_module(&mut optimized_module);
        }

        // Create LLVM backend and pass optimization level
        let mut backend = LLVMCodegen::new(context, &optimized_module.name);
        backend.set_optimization_level(optimization_level);
        
        // Generate object file (LLVM optimization passes are configured inside)
        let object_file = output.with_extension("o");
        backend.generate(&optimized_module, Some(object_file.clone()), None)
            .context(format!("Failed to generate object file: {}", object_file.display()))?;

        Ok(object_file)
    }

    /// Links object files into an executable binary.
    ///
    /// This internal method uses the platform-specific linker to combine object files
    /// with runtime libraries and produce the final executable.
    ///
    /// # Arguments
    ///
    /// * `object_files` - Paths to object files to link
    /// * `output` - Path where the executable should be written
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful linking.
    fn link_executable(&self, object_files: &[PathBuf], output: PathBuf) -> Result<()> {
        
        let linker = Linker::new();
        linker.link_executable(object_files, output.clone())
            .context(format!("Failed to link executable: {}", output.display()))?;

        Ok(())
    }
}
