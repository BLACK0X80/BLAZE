use std::collections::HashMap;
use std::io::{self, Write};
use crate::lexer::lex;
use crate::parser::parse;

pub struct REPL {
    environment: HashMap<String, Value>,
    history: Vec<String>,
    multiline_buffer: String,
    prompt: String,
}

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Unit,
}

impl REPL {
    pub fn new() -> Self {
        Self {
            environment: HashMap::new(),
            history: Vec::new(),
            multiline_buffer: String::new(),
            prompt: "blaze> ".to_string(),
        }
    }
    
    pub fn run(&mut self) {
        println!("BLAZE REPL v0.1.0");
        println!("Type 'exit' to quit, 'help' for commands\n");
        
        loop {
            print!("{}", self.prompt);
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                break;
            }
            
            let input = input.trim();
            
            if input.is_empty() {
                continue;
            }
            
            if input == "exit" || input == "quit" {
                break;
            }
            
            if input == "help" {
                self.print_help();
                continue;
            }
            
            if input == "history" {
                self.print_history();
                continue;
            }
            
            if input == "clear" {
                self.environment.clear();
                println!("Environment cleared");
                continue;
            }
            
            self.history.push(input.to_string());
            
            match self.eval(input) {
                Ok(value) => println!("{:?}", value),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        
        println!("Goodbye!");
    }
    
    fn eval(&mut self, input: &str) -> Result<Value, String> {
        let tokens = lex(input).map_err(|e| format!("{:?}", e))?;
        
        let program = parse(tokens).map_err(|e| format!("{:?}", e))?;
        
        Ok(Value::Unit)
    }
    
    fn print_help(&self) {
        println!("Available commands:");
        println!("  help     - Show this help message");
        println!("  history  - Show command history");
        println!("  clear    - Clear environment");
        println!("  exit     - Exit REPL");
    }
    
    fn print_history(&self) {
        println!("Command history:");
        for (i, cmd) in self.history.iter().enumerate() {
            println!("  {}: {}", i + 1, cmd);
        }
    }
    
    pub fn set_variable(&mut self, name: String, value: Value) {
        self.environment.insert(name, value);
    }
    
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.environment.get(name)
    }
}

impl Default for REPL {
    fn default() -> Self {
        Self::new()
    }
}
