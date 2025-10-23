use blaze_compiler::*;
use std::time::Instant;

#[test]
fn test_lexer_performance() {
    let source = r#"
        fn main() {
            let x = 42;
            let y = x + 10;
            let z = y * 2;
        }
    "#.repeat(100);
    
    let start = Instant::now();
    let result = lex(&source);
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "Lexing should succeed");
    println!("Lexer processed {} chars in {:?}", source.len(), duration);
}

#[test]
fn test_parser_performance() {
    let source = r#"
        fn test() {
            let x = 1 + 2 + 3 + 4 + 5;
        }
    "#.repeat(50);
    
    let tokens = lex(&source).expect("Lexing failed");
    
    let start = Instant::now();
    let result = parse(tokens);
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "Parsing should succeed");
    println!("Parser processed program in {:?}", duration);
}

#[test]
fn test_compilation_performance() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            return a + b;
        }
        
        fn main() {
            let x = add(5, 10);
            let y = add(x, 20);
            let z = add(y, 30);
        }
    "#;
    
    let start = Instant::now();
    let result = compile(source);
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "Compilation should succeed");
    println!("Full compilation took {:?}", duration);
}

#[test]
fn test_vec_performance() {
    use blaze_compiler::stdlib::Vec;
    
    let start = Instant::now();
    let mut vec = Vec::new();
    
    for i in 0..10000 {
        vec.push(i);
    }
    
    let duration = start.elapsed();
    
    assert_eq!(vec.len(), 10000);
    println!("Vec push 10000 items in {:?}", duration);
}

#[test]
fn test_allocator_performance() {
    use blaze_compiler::runtime::{blaze_alloc, blaze_dealloc};
    
    let start = Instant::now();
    
    unsafe {
        for _ in 0..1000 {
            let ptr = blaze_alloc(1024, 8);
            if !ptr.is_null() {
                blaze_dealloc(ptr, 1024, 8);
            }
        }
    }
    
    let duration = start.elapsed();
    println!("1000 allocations/deallocations in {:?}", duration);
}

#[test]
fn test_semantic_analysis_performance() {
    let source = r#"
        fn compute(x: i32) -> i32 {
            let a = x + 1;
            let b = a * 2;
            let c = b - 3;
            return c;
        }
        
        fn main() {
            let result = compute(42);
        }
    "#;
    
    let start = Instant::now();
    let result = check(source);
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "Semantic analysis should succeed");
    println!("Semantic analysis took {:?}", duration);
}

#[test]
fn test_ir_generation_performance() {
    let source = r#"
        fn main() {
            let x = 1 + 2;
            let y = x * 3;
            let z = y - 4;
        }
    "#;
    
    let tokens = lex(source).expect("Lexing failed");
    let program = parse(tokens).expect("Parsing failed");
    
    let start = Instant::now();
    let result = ir::generate(&program);
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "IR generation should succeed");
    println!("IR generation took {:?}", duration);
}

#[test]
fn test_large_program_compilation() {
    let mut source = String::from("fn main() {\n");
    
    for i in 0..100 {
        source.push_str(&format!("    let x{} = {} + {};\n", i, i, i + 1));
    }
    
    source.push_str("}\n");
    
    let start = Instant::now();
    let result = compile(&source);
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "Large program should compile");
    println!("Large program (100 statements) compiled in {:?}", duration);
}

/// Benchmark compilation speed for various program sizes
#[test]
fn benchmark_compilation_speed_small() {
    let source = r#"
        fn main() {
            let x: i32 = 42;
            let y: i32 = x + 10;
        }
    "#;
    
    let start = Instant::now();
    for _ in 0..100 {
        let _ = compile(source);
    }
    let duration = start.elapsed();
    
    println!("Small program (2 statements): 100 compilations in {:?}", duration);
    println!("Average: {:?} per compilation", duration / 100);
}

#[test]
fn benchmark_compilation_speed_medium() {
    let mut source = String::from("fn main() {\n");
    for i in 0..50 {
        source.push_str(&format!("    let x{}: i32 = {} + {};\n", i, i, i + 1));
    }
    source.push_str("}\n");
    
    let start = Instant::now();
    for _ in 0..10 {
        let _ = compile(&source);
    }
    let duration = start.elapsed();
    
    println!("Medium program (50 statements): 10 compilations in {:?}", duration);
    println!("Average: {:?} per compilation", duration / 10);
}

#[test]
fn benchmark_compilation_speed_large() {
    let mut source = String::from("fn main() {\n");
    for i in 0..200 {
        source.push_str(&format!("    let x{}: i32 = {} + {};\n", i, i, i + 1));
    }
    source.push_str("}\n");
    
    let start = Instant::now();
    let result = compile(&source);
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "Large program should compile");
    println!("Large program (200 statements): compiled in {:?}", duration);
}

#[test]
fn benchmark_compilation_speed_with_functions() {
    let mut source = String::new();
    
    // Generate 20 functions
    for i in 0..20 {
        source.push_str(&format!(
            "fn func{}(a: i32, b: i32) -> i32 {{\n    return a + b + {};\n}}\n\n",
            i, i
        ));
    }
    
    source.push_str("fn main() {\n");
    for i in 0..20 {
        source.push_str(&format!("    let x{}: i32 = func{}({}, {});\n", i, i, i, i + 1));
    }
    source.push_str("}\n");
    
    let start = Instant::now();
    let result = compile(&source);
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "Program with functions should compile");
    println!("Program with 20 functions: compiled in {:?}", duration);
}

#[test]
fn benchmark_compilation_speed_with_control_flow() {
    let source = r#"
        fn main() {
            let x: i32 = 10;
            if x > 5 {
                let y: i32 = x * 2;
                if y > 15 {
                    let z: i32 = y + 5;
                }
            }
            
            let i: i32 = 0;
            while i < 10 {
                let j: i32 = i * 2;
                let i: i32 = i + 1;
            }
        }
    "#;
    
    let start = Instant::now();
    for _ in 0..50 {
        let _ = compile(source);
    }
    let duration = start.elapsed();
    
    println!("Program with control flow: 50 compilations in {:?}", duration);
    println!("Average: {:?} per compilation", duration / 50);
}

/// Benchmark memory allocator performance
#[test]
fn benchmark_allocator_small_allocations() {
    use blaze_compiler::runtime::{blaze_alloc, blaze_dealloc};
    
    let start = Instant::now();
    
    unsafe {
        for _ in 0..10000 {
            let ptr = blaze_alloc(64, 8);
            if !ptr.is_null() {
                blaze_dealloc(ptr, 64, 8);
            }
        }
    }
    
    let duration = start.elapsed();
    println!("10000 small allocations (64 bytes): {:?}", duration);
    println!("Average: {:?} per allocation", duration / 10000);
}

#[test]
fn benchmark_allocator_medium_allocations() {
    use blaze_compiler::runtime::{blaze_alloc, blaze_dealloc};
    
    let start = Instant::now();
    
    unsafe {
        for _ in 0..5000 {
            let ptr = blaze_alloc(1024, 8);
            if !ptr.is_null() {
                blaze_dealloc(ptr, 1024, 8);
            }
        }
    }
    
    let duration = start.elapsed();
    println!("5000 medium allocations (1KB): {:?}", duration);
    println!("Average: {:?} per allocation", duration / 5000);
}

#[test]
fn benchmark_allocator_large_allocations() {
    use blaze_compiler::runtime::{blaze_alloc, blaze_dealloc};
    
    let start = Instant::now();
    
    unsafe {
        for _ in 0..1000 {
            let ptr = blaze_alloc(1024 * 1024, 8);
            if !ptr.is_null() {
                blaze_dealloc(ptr, 1024 * 1024, 8);
            }
        }
    }
    
    let duration = start.elapsed();
    println!("1000 large allocations (1MB): {:?}", duration);
    println!("Average: {:?} per allocation", duration / 1000);
}

#[test]
fn benchmark_allocator_mixed_sizes() {
    use blaze_compiler::runtime::{blaze_alloc, blaze_dealloc};
    
    let sizes = vec![16, 32, 64, 128, 256, 512, 1024, 2048, 4096];
    let start = Instant::now();
    
    unsafe {
        for _ in 0..1000 {
            for &size in &sizes {
                let ptr = blaze_alloc(size, 8);
                if !ptr.is_null() {
                    blaze_dealloc(ptr, size, 8);
                }
            }
        }
    }
    
    let duration = start.elapsed();
    println!("9000 mixed size allocations: {:?}", duration);
    println!("Average: {:?} per allocation", duration / 9000);
}

/// Benchmark Vec performance
#[test]
fn benchmark_vec_push_performance() {
    use blaze_compiler::stdlib::Vec;
    
    let sizes = vec![100, 1000, 10000];
    
    for size in sizes {
        let start = Instant::now();
        let mut vec = Vec::new();
        
        for i in 0..size {
            vec.push(i);
        }
        
        let duration = start.elapsed();
        println!("Vec push {} items: {:?}", size, duration);
    }
}

#[test]
fn benchmark_vec_with_capacity() {
    use blaze_compiler::stdlib::Vec;
    
    let size = 10000;
    
    // Without capacity
    let start = Instant::now();
    let mut vec1 = Vec::new();
    for i in 0..size {
        vec1.push(i);
    }
    let duration1 = start.elapsed();
    
    // With capacity
    let start = Instant::now();
    let mut vec2 = Vec::with_capacity(size);
    for i in 0..size {
        vec2.push(i);
    }
    let duration2 = start.elapsed();
    
    println!("Vec without capacity: {:?}", duration1);
    println!("Vec with capacity: {:?}", duration2);
    println!("Speedup: {:.2}x", duration1.as_nanos() as f64 / duration2.as_nanos() as f64);
}

#[test]
fn benchmark_vec_iteration() {
    use blaze_compiler::stdlib::Vec;
    
    let mut vec = Vec::new();
    for i in 0..10000 {
        vec.push(i);
    }
    
    let start = Instant::now();
    let sum: i32 = vec.iter().sum();
    let duration = start.elapsed();
    
    assert_eq!(sum, (0..10000).sum());
    println!("Vec iteration over 10000 items: {:?}", duration);
}

/// Benchmark lexer performance with different input sizes
#[test]
fn benchmark_lexer_scaling() {
    let base_source = r#"
        fn test() {
            let x = 42;
            let y = x + 10;
        }
    "#;
    
    let sizes = vec![10, 50, 100, 200];
    
    for size in sizes {
        let source = base_source.repeat(size);
        let start = Instant::now();
        let result = lex(&source);
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        println!("Lexer with {}x repetition ({} chars): {:?}", 
                 size, source.len(), duration);
    }
}

/// Benchmark parser performance with different complexity
#[test]
fn benchmark_parser_scaling() {
    let sizes = vec![10, 25, 50, 100];
    
    for size in sizes {
        let mut source = String::from("fn main() {\n");
        for i in 0..size {
            source.push_str(&format!("    let x{}: i32 = {} + {};\n", i, i, i + 1));
        }
        source.push_str("}\n");
        
        let tokens = lex(&source).expect("Lexing failed");
        let start = Instant::now();
        let result = parse(tokens);
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        println!("Parser with {} statements: {:?}", size, duration);
    }
}

/// Benchmark semantic analysis performance
#[test]
fn benchmark_semantic_analysis_scaling() {
    let sizes = vec![10, 25, 50, 100];
    
    for size in sizes {
        let mut source = String::from("fn main() {\n");
        for i in 0..size {
            source.push_str(&format!("    let x{}: i32 = {} + {};\n", i, i, i + 1));
        }
        source.push_str("}\n");
        
        let start = Instant::now();
        let result = check(&source);
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        println!("Semantic analysis with {} statements: {:?}", size, duration);
    }
}

/// Benchmark IR generation performance
#[test]
fn benchmark_ir_generation_scaling() {
    let sizes = vec![10, 25, 50, 100];
    
    for size in sizes {
        let mut source = String::from("fn main() {\n");
        for i in 0..size {
            source.push_str(&format!("    let x{}: i32 = {} + {};\n", i, i, i + 1));
        }
        source.push_str("}\n");
        
        let tokens = lex(&source).expect("Lexing failed");
        let program = parse(tokens).expect("Parsing failed");
        
        let start = Instant::now();
        let result = ir::generate(&program);
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        println!("IR generation with {} statements: {:?}", size, duration);
    }
}
