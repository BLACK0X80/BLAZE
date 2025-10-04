use blaze_compiler::compile_file;
use std::env;
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: blaze <command> <file>");
        eprintln!("Commands:");
        eprintln!("  check <file>  - Check syntax");
        eprintln!("  build <file>  - Compile file");
        eprintln!("  run <file>    - Compile and run");
        process::exit(1);
    }
    
    let command = &args[1];
    
    match command.as_str() {
        "check" | "build" | "run" => {
            if args.len() < 3 {
                eprintln!("Error: Missing file argument");
                process::exit(1);
            }
            
            let file_path = Path::new(&args[2]);
            
            if !file_path.exists() {
                eprintln!("Error: File not found: {}", args[2]);
                process::exit(1);
            }
            
            match compile_file(file_path) {
                Ok(program) => {
                    println!("✓ Compilation successful!");
                    println!("\nAST:");
                    println!("{:#?}", program);
                    
                    if command == "run" {
                        println!("\nNote: Execution not yet implemented");
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Valid commands: check, build, run");
            process::exit(1);
        }
    }
}