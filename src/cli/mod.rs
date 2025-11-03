use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "blaze")]
#[command(author = "BLAZE Team")]
#[command(version = "0.1.0")]
#[command(about = "🔥 BLAZE - Modern systems programming language compiler")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Check {
        #[arg(value_name = "FILE")]
        input: PathBuf,
        
        #[arg(short, long)]
        verbose: bool,
    },
    
    Build {
        #[arg(value_name = "FILE")]
        input: PathBuf,
        
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
        
        #[arg(short = 'O', long, default_value = "0")]
        optimization: u8,
        
        #[arg(long)]
        emit_ir: bool,
        
        #[arg(long)]
        emit_asm: bool,
        
        #[arg(short, long)]
        verbose: bool,
        
        #[arg(short, long)]
        release: bool,
    },
    
    Run {
        #[arg(value_name = "FILE")]
        input: PathBuf,
        
        #[arg(last = true)]
        args: Vec<String>,
        
        #[arg(short = 'O', long, default_value = "0")]
        optimization: u8,
        
        #[arg(short, long)]
        verbose: bool,
    },
    
    Fmt {
        #[arg(value_name = "PATH")]
        path: PathBuf,
        
        #[arg(long)]
        check: bool,
    },
    
    Version {
        #[arg(long)]
        verbose: bool,
    },
    
    Init {
        #[arg(value_name = "NAME")]
        name: String,
        
        #[arg(long)]
        lib: bool,
    },
    
    Test {
        #[arg(long)]
        all: bool,
        
        #[arg(long)]
        nocapture: bool,
        
        #[arg(value_name = "FILTER")]
        filter: Option<String>,
    },
    
    Bench {
        #[arg(value_name = "BENCH_NAME")]
        bench_name: Option<String>,
        
        #[arg(long)]
        baseline: Option<String>,
    },
    
    Doc {
        #[arg(long)]
        open: bool,
        
        #[arg(long)]
        no_deps: bool,
    },
    
    Publish {
        #[arg(long)]
        token: Option<String>,
        
        #[arg(long)]
        dry_run: bool,
    },
    
    Add {
        #[arg(value_name = "PACKAGE")]
        package: String,
        
        #[arg(long)]
        version: Option<String>,
        
        #[arg(long)]
        dev: bool,
    },
    
    Remove {
        #[arg(value_name = "PACKAGE")]
        package: String,
    },
    
    Update {
        #[arg(value_name = "PACKAGE")]
        package: Option<String>,
    },
    
    Search {
        #[arg(value_name = "QUERY")]
        query: String,
        
        #[arg(long)]
        limit: Option<usize>,
    },
    
    Clean,
    
    Tree,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
