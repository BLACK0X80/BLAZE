use std::collections::{HashMap, HashSet};

pub struct Debugger {
    breakpoints: HashMap<usize, Breakpoint>,
    watchpoints: HashMap<String, Watchpoint>,
    call_stack: Vec<StackFrame>,
    current_line: usize,
    state: DebuggerState,
}

#[derive(Debug, Clone)]
pub struct Breakpoint {
    pub line: usize,
    pub file: String,
    pub enabled: bool,
    pub condition: Option<String>,
    pub hit_count: usize,
}

#[derive(Debug, Clone)]
pub struct Watchpoint {
    pub variable: String,
    pub old_value: Option<String>,
    pub access_type: WatchpointType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WatchpointType {
    Read,
    Write,
    ReadWrite,
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function_name: String,
    pub file: String,
    pub line: usize,
    pub locals: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DebuggerState {
    Running,
    Paused,
    Stepping,
    Finished,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            breakpoints: HashMap::new(),
            watchpoints: HashMap::new(),
            call_stack: Vec::new(),
            current_line: 0,
            state: DebuggerState::Paused,
        }
    }
    
    pub fn add_breakpoint(&mut self, line: usize, file: String) -> usize {
        let id = self.breakpoints.len();
        let breakpoint = Breakpoint {
            line,
            file,
            enabled: true,
            condition: None,
            hit_count: 0,
        };
        self.breakpoints.insert(id, breakpoint);
        id
    }
    
    pub fn remove_breakpoint(&mut self, id: usize) -> bool {
        self.breakpoints.remove(&id).is_some()
    }
    
    pub fn enable_breakpoint(&mut self, id: usize, enabled: bool) -> Result<(), String> {
        self.breakpoints
            .get_mut(&id)
            .map(|bp| bp.enabled = enabled)
            .ok_or_else(|| "Breakpoint not found".to_string())
    }
    
    pub fn set_breakpoint_condition(&mut self, id: usize, condition: String) -> Result<(), String> {
        self.breakpoints
            .get_mut(&id)
            .map(|bp| bp.condition = Some(condition))
            .ok_or_else(|| "Breakpoint not found".to_string())
    }
    
    pub fn add_watchpoint(&mut self, variable: String, access_type: WatchpointType) {
        let watchpoint = Watchpoint {
            variable: variable.clone(),
            old_value: None,
            access_type,
        };
        self.watchpoints.insert(variable, watchpoint);
    }
    
    pub fn remove_watchpoint(&mut self, variable: &str) -> bool {
        self.watchpoints.remove(variable).is_some()
    }
    
    pub fn step_over(&mut self) {
        self.state = DebuggerState::Stepping;
        self.current_line += 1;
    }
    
    pub fn step_into(&mut self) {
        self.state = DebuggerState::Stepping;
        self.current_line += 1;
    }
    
    pub fn step_out(&mut self) {
        if !self.call_stack.is_empty() {
            self.call_stack.pop();
        }
        self.state = DebuggerState::Stepping;
    }
    
    pub fn continue_execution(&mut self) {
        self.state = DebuggerState::Running;
    }
    
    pub fn pause(&mut self) {
        self.state = DebuggerState::Paused;
    }
    
    pub fn push_frame(&mut self, frame: StackFrame) {
        self.call_stack.push(frame);
    }
    
    pub fn pop_frame(&mut self) -> Option<StackFrame> {
        self.call_stack.pop()
    }
    
    pub fn get_stack_trace(&self) -> Vec<String> {
        self.call_stack
            .iter()
            .map(|frame| {
                format!("{} at {}:{}", frame.function_name, frame.file, frame.line)
            })
            .collect()
    }
    
    pub fn get_local_variables(&self) -> HashMap<String, String> {
        self.call_stack
            .last()
            .map(|frame| frame.locals.clone())
            .unwrap_or_default()
    }
    
    pub fn evaluate_expression(&self, expr: &str) -> Result<String, String> {
        Ok(format!("Evaluated: {}", expr))
    }
    
    pub fn should_break_at(&mut self, line: usize, file: &str) -> bool {
        for breakpoint in self.breakpoints.values_mut() {
            if breakpoint.enabled && breakpoint.line == line && breakpoint.file == file {
                breakpoint.hit_count += 1;
                
                if let Some(ref condition) = breakpoint.condition {
                    return true;
                }
                
                return true;
            }
        }
        
        false
    }
    
    pub fn check_watchpoints(&mut self, variable: &str, new_value: &str) -> bool {
        if let Some(watchpoint) = self.watchpoints.get_mut(variable) {
            if matches!(watchpoint.access_type, WatchpointType::Write | WatchpointType::ReadWrite) {
                let changed = watchpoint.old_value.as_ref().map_or(true, |old| old != new_value);
                
                if changed {
                    watchpoint.old_value = Some(new_value.to_string());
                    return true;
                }
            }
        }
        
        false
    }
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DebugSymbols {
    line_to_address: HashMap<usize, usize>,
    address_to_line: HashMap<usize, usize>,
    variable_locations: HashMap<String, VariableLocation>,
}

#[derive(Debug, Clone)]
pub struct VariableLocation {
    pub name: String,
    pub scope_start: usize,
    pub scope_end: usize,
    pub location: StorageLocation,
}

#[derive(Debug, Clone)]
pub enum StorageLocation {
    Register(u8),
    Stack(i32),
    Global(usize),
}

impl DebugSymbols {
    pub fn new() -> Self {
        Self {
            line_to_address: HashMap::new(),
            address_to_line: HashMap::new(),
            variable_locations: HashMap::new(),
        }
    }
    
    pub fn add_line_mapping(&mut self, line: usize, address: usize) {
        self.line_to_address.insert(line, address);
        self.address_to_line.insert(address, line);
    }
    
    pub fn add_variable(&mut self, var: VariableLocation) {
        self.variable_locations.insert(var.name.clone(), var);
    }
    
    pub fn get_address(&self, line: usize) -> Option<usize> {
        self.line_to_address.get(&line).copied()
    }
    
    pub fn get_line(&self, address: usize) -> Option<usize> {
        self.address_to_line.get(&address).copied()
    }
    
    pub fn get_variable_location(&self, name: &str, line: usize) -> Option<&VariableLocation> {
        self.variable_locations.get(name).filter(|var| {
            line >= var.scope_start && line <= var.scope_end
        })
    }
}

impl Default for DebugSymbols {
    fn default() -> Self {
        Self::new()
    }
}
