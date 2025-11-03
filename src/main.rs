use blaze_compiler::cli::Cli;
use clap::Parser;

fn main() {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        blaze_compiler::cli::Commands::Check { input, verbose } => {
            println!("Checking: {:?}", input);
            if verbose {
                println!("Verbose mode enabled");
            }
        }
        blaze_compiler::cli::Commands::Build { input, output, optimization, verbose, .. } => {
            println!("Building: {:?}", input);
            if verbose {
                println!("Optimization level: {}", optimization);
                if let Some(out) = output {
                    println!("Output: {:?}", out);
                }
            }
        }
        blaze_compiler::cli::Commands::Run { input, args, .. } => {
            println!("Running: {:?}", input);
            if !args.is_empty() {
                println!("Arguments: {:?}", args);
            }
        }
        blaze_compiler::cli::Commands::Test { all, filter, .. } => {
            println!("Running tests...");
            if all {
                println!("Running all tests");
            }
            if let Some(f) = filter {
                println!("Filter: {}", f);
            }
        }
        blaze_compiler::cli::Commands::Fmt { path, check } => {
            println!("Formatting: {:?}", path);
            if check {
                println!("Check mode enabled");
            }
        }
        blaze_compiler::cli::Commands::Init { name, lib } => {
            println!("Initializing project: {}", name);
            if lib {
                println!("Creating library");
            } else {
                println!("Creating binary");
            }
        }
        blaze_compiler::cli::Commands::Version { verbose } => {
            println!("BLAZE compiler version 0.1.0");
            if verbose {
                println!("Built with: Rust {}", env!("CARGO_PKG_RUST_VERSION", "unknown"));
                println!("LLVM version: 15.0");
            }
        }
        _ => {
            println!("Command not yet fully implemented");
        }
    }
}