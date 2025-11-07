pub mod constant_folding;
pub mod dead_code_elimination;
pub mod inlining;
pub mod peephole;
pub mod aggressive_opts;

pub use constant_folding::*;
pub use dead_code_elimination::*;
pub use inlining::*;
pub use peephole::*;
pub use aggressive_opts::*;

use super::Module;
use anyhow::Result;

pub struct Optimizer {
    constant_folder: ConstantFolder,
    dead_code_eliminator: DeadCodeEliminator,
    inliner: Inliner,
    peephole_optimizer: PeepholeOptimizer,
    aggressive_optimizer: AggressiveOptimizer,
}

impl Optimizer {
    pub fn new() -> Self {
        Self {
            constant_folder: ConstantFolder::new(),
            dead_code_eliminator: DeadCodeEliminator::new(),
            inliner: Inliner::new(),
            peephole_optimizer: PeepholeOptimizer::new(),
            aggressive_optimizer: AggressiveOptimizer::new(),
        }
    }

    pub fn optimize(&mut self, module: &Module, opt_level: u8) -> Result<Module> {
        let mut optimized_module = module.clone();

        match opt_level {
            0 => {
                // No optimization
            }
            1 => {
                optimized_module = self.constant_folder.optimize(&optimized_module)?;
                optimized_module = self.dead_code_eliminator.optimize(&optimized_module)?;
            }
            2 => {
                optimized_module = self.constant_folder.optimize(&optimized_module)?;
                optimized_module = self.dead_code_eliminator.optimize(&optimized_module)?;
                optimized_module = self.peephole_optimizer.optimize(&optimized_module)?;
            }
            3 => {
                for _ in 0..3 {
                    optimized_module = self.constant_folder.optimize(&optimized_module)?;
                    optimized_module = self.dead_code_eliminator.optimize(&optimized_module)?;
                    optimized_module = self.inliner.optimize(&optimized_module)?;
                    optimized_module = self.peephole_optimizer.optimize(&optimized_module)?;
                    self.aggressive_optimizer.optimize(&mut optimized_module)?;
                }
            }
            _ => {
                return Err(anyhow::anyhow!("Invalid optimization level: {}", opt_level));
            }
        }

        Ok(optimized_module)
    }
}

pub trait OptimizationPass {
    fn optimize(&mut self, module: &Module) -> Result<Module>;
}

