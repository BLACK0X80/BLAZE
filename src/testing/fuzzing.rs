use std::collections::HashSet;
use std::time::{Duration, Instant};

pub struct FuzzingEngine {
    corpus: Vec<Vec<u8>>,
    coverage: HashSet<u64>,
    total_executions: u64,
    crashes: Vec<FuzzCrash>,
    config: FuzzConfig,
}

#[derive(Debug, Clone)]
pub struct FuzzConfig {
    pub max_iterations: u64,
    pub max_input_size: usize,
    pub timeout: Duration,
    pub mutation_strategies: Vec<MutationStrategy>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MutationStrategy {
    BitFlip,
    ByteFlip,
    ArithmeticMutation,
    InterestingValues,
    Havoc,
    Splice,
}

#[derive(Debug, Clone)]
pub struct FuzzCrash {
    pub input: Vec<u8>,
    pub error: String,
    pub stack_trace: Option<String>,
}

impl FuzzingEngine {
    pub fn new(config: FuzzConfig) -> Self {
        Self {
            corpus: Vec::new(),
            coverage: HashSet::new(),
            total_executions: 0,
            crashes: Vec::new(),
            config,
        }
    }
    
    pub fn add_seed(&mut self, seed: Vec<u8>) {
        self.corpus.push(seed);
    }
    
    pub fn fuzz<F>(&mut self, target: F) -> FuzzResult
    where
        F: Fn(&[u8]) -> Result<(), String>,
    {
        let start = Instant::now();
        let mut new_coverage = 0;
        
        while self.total_executions < self.config.max_iterations {
            if start.elapsed() > self.config.timeout {
                break;
            }
            
            let input = self.generate_input();
            
            let coverage_before = self.coverage.len();
            
            match target(&input) {
                Ok(_) => {
                    if self.coverage.len() > coverage_before {
                        new_coverage += 1;
                        self.corpus.push(input);
                    }
                }
                Err(e) => {
                    self.crashes.push(FuzzCrash {
                        input: input.clone(),
                        error: e,
                        stack_trace: None,
                    });
                }
            }
            
            self.total_executions += 1;
        }
        
        FuzzResult {
            total_executions: self.total_executions,
            crashes: self.crashes.len(),
            new_coverage,
            duration: start.elapsed(),
        }
    }
    
    fn generate_input(&mut self) -> Vec<u8> {
        if self.corpus.is_empty() {
            return self.random_input();
        }
        
        let seed_idx = self.total_executions as usize % self.corpus.len();
        let seed = &self.corpus[seed_idx].clone();
        
        let strategy_idx = self.total_executions as usize % self.config.mutation_strategies.len();
        let strategy = self.config.mutation_strategies[strategy_idx];
        
        self.mutate(seed, strategy)
    }
    
    fn random_input(&self) -> Vec<u8> {
        let size = (self.total_executions % self.config.max_input_size as u64) as usize;
        (0..size).map(|_| (self.total_executions % 256) as u8).collect()
    }
    
    fn mutate(&self, input: &[u8], strategy: MutationStrategy) -> Vec<u8> {
        let mut mutated = input.to_vec();
        
        match strategy {
            MutationStrategy::BitFlip => self.bit_flip(&mut mutated),
            MutationStrategy::ByteFlip => self.byte_flip(&mut mutated),
            MutationStrategy::ArithmeticMutation => self.arithmetic_mutation(&mut mutated),
            MutationStrategy::InterestingValues => self.interesting_values(&mut mutated),
            MutationStrategy::Havoc => self.havoc(&mut mutated),
            MutationStrategy::Splice => self.splice(&mut mutated),
        }
        
        mutated
    }
    
    fn bit_flip(&self, input: &mut [u8]) {
        if input.is_empty() {
            return;
        }
        
        let byte_idx = (self.total_executions as usize) % input.len();
        let bit_idx = (self.total_executions % 8) as u8;
        
        input[byte_idx] ^= 1 << bit_idx;
    }
    
    fn byte_flip(&self, input: &mut [u8]) {
        if input.is_empty() {
            return;
        }
        
        let idx = (self.total_executions as usize) % input.len();
        input[idx] ^= 0xFF;
    }
    
    fn arithmetic_mutation(&self, input: &mut [u8]) {
        if input.is_empty() {
            return;
        }
        
        let idx = (self.total_executions as usize) % input.len();
        let delta = ((self.total_executions % 35) as i8) - 16;
        
        input[idx] = input[idx].wrapping_add(delta as u8);
    }
    
    fn interesting_values(&self, input: &mut [u8]) {
        if input.is_empty() {
            return;
        }
        
        let interesting = [0, 1, 0xFF, 0x7F, 0x80];
        let idx = (self.total_executions as usize) % input.len();
        let val_idx = (self.total_executions as usize) % interesting.len();
        
        input[idx] = interesting[val_idx];
    }
    
    fn havoc(&self, input: &mut Vec<u8>) {
        let num_mutations = 1 + (self.total_executions % 16);
        
        for _ in 0..num_mutations {
            let mutation = (self.total_executions % 5) as usize;
            
            match mutation {
                0 => self.bit_flip(input),
                1 => self.byte_flip(input),
                2 => self.arithmetic_mutation(input),
                3 => {
                    if !input.is_empty() {
                        let idx = (self.total_executions as usize) % input.len();
                        input.remove(idx);
                    }
                }
                4 => {
                    let byte = (self.total_executions % 256) as u8;
                    input.push(byte);
                }
                _ => {}
            }
        }
    }
    
    fn splice(&self, input: &mut Vec<u8>) {
        if self.corpus.len() < 2 {
            return;
        }
        
        let other_idx = ((self.total_executions + 1) as usize) % self.corpus.len();
        let other = &self.corpus[other_idx];
        
        if !other.is_empty() {
            let split_point = (self.total_executions as usize) % input.len().max(1);
            input.truncate(split_point);
            input.extend_from_slice(other);
        }
    }
    
    pub fn minimize_crash(&self, crash: &FuzzCrash) -> Vec<u8> {
        let mut minimized = crash.input.clone();
        
        while minimized.len() > 1 {
            let half = minimized.len() / 2;
            let test = minimized[..half].to_vec();
            
            minimized = test;
        }
        
        minimized
    }
}

#[derive(Debug, Clone)]
pub struct FuzzResult {
    pub total_executions: u64,
    pub crashes: usize,
    pub new_coverage: usize,
    pub duration: Duration,
}

impl Default for FuzzConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100000,
            max_input_size: 4096,
            timeout: Duration::from_secs(3600),
            mutation_strategies: vec![
                MutationStrategy::BitFlip,
                MutationStrategy::ByteFlip,
                MutationStrategy::ArithmeticMutation,
                MutationStrategy::InterestingValues,
                MutationStrategy::Havoc,
            ],
        }
    }
}

impl Default for FuzzingEngine {
    fn default() -> Self {
        Self::new(FuzzConfig::default())
    }
}
