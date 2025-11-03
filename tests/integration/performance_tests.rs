use blaze::*;
use std::time::Instant;

#[test]
fn benchmark_compilation_speed() {
    let source = r#"
        fn factorial(n: i32) -> i32 {
            if n <= 1 {
                return 1;
            }
            return n * factorial(n - 1);
        }
        
        fn fibonacci(n: i32) -> i32 {
            if n <= 1 {
                return n;
            }
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
        
        fn main() {
            let f1 = factorial(10);
            let f2 = fibonacci(10);
            println("Results: {} {}", f1, f2);
        }
    "#;
    
    let start = Instant::now();
    
    let mut compiler = Compiler::new();
    let result = compiler.compile_string(source);
    
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    assert!(duration.as_millis() < 1000, "Compilation should be fast (< 1s)");
    
    println!("Compilation time: {:?}", duration);
}

#[test]
fn benchmark_optimization_passes() {
    let source = r#"
        fn compute(n: i32) -> i32 {
            let mut sum = 0;
            for i in 0..n {
                sum += i * 2;
            }
            return sum;
        }
        
        fn main() {
            let result = compute(1000);
            println("{}", result);
        }
    "#;
    
    for opt_level in 0..=3 {
        let start = Instant::now();
        
        let mut compiler = Compiler::new();
        compiler.set_optimization_level(opt_level);
        let result = compiler.compile_string(source);
        
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        println!("Optimization level {}: {:?}", opt_level, duration);
    }
}

#[test]
fn benchmark_large_file_compilation() {
    let mut source = String::from("fn main() {\n");
    
    for i in 0..1000 {
        source.push_str(&format!("    let var_{} = {};\n", i, i));
    }
    
    source.push_str("}\n");
    
    let start = Instant::now();
    
    let mut compiler = Compiler::new();
    let result = compiler.compile_string(&source);
    
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    assert!(duration.as_secs() < 5, "Large file compilation should be reasonable (< 5s)");
    
    println!("Large file (1000 vars) compilation: {:?}", duration);
}

#[test]
fn benchmark_incremental_rebuild() {
    let source1 = r#"
        fn helper() -> i32 { 42 }
    "#;
    
    let source2 = r#"
        fn main() {
            let x = helper();
            println("{}", x);
        }
    "#;
    
    let mut compiler = Compiler::new();
    compiler.enable_incremental();
    
    let start1 = Instant::now();
    compiler.compile_string(source1).unwrap();
    let duration1 = start1.elapsed();
    
    let start2 = Instant::now();
    compiler.compile_string(source2).unwrap();
    let duration2 = start2.elapsed();
    
    let start3 = Instant::now();
    compiler.compile_string(source2).unwrap();
    let duration3 = start3.elapsed();
    
    println!("First compilation: {:?}", duration1);
    println!("Second compilation: {:?}", duration2);
    println!("Rebuild (no changes): {:?}", duration3);
    
    assert!(duration3 < duration2, "Incremental rebuild should be faster");
}

#[test]
fn memory_usage_stress_test() {
    let mut large_source = String::from("struct Data {\n");
    
    for i in 0..100 {
        large_source.push_str(&format!("    field_{}: i32,\n", i));
    }
    
    large_source.push_str("}\n\nfn main() {\n");
    
    for i in 0..100 {
        large_source.push_str(&format!("    let obj_{} = Data {{ ", i));
        
        for j in 0..100 {
            if j > 0 {
                large_source.push_str(", ");
            }
            large_source.push_str(&format!("field_{}: {}", j, j));
        }
        
        large_source.push_str(" };\n");
    }
    
    large_source.push_str("}\n");
    
    let mut compiler = Compiler::new();
    let result = compiler.compile_string(&large_source);
    
    assert!(result.is_ok(), "Should handle large complex structures");
}

#[test]
fn parallel_compilation_benchmark() {
    let sources = vec![
        r#"fn func1() -> i32 { 1 }"#,
        r#"fn func2() -> i32 { 2 }"#,
        r#"fn func3() -> i32 { 3 }"#,
        r#"fn func4() -> i32 { 4 }"#,
    ];
    
    let start = Instant::now();
    
    let handles: Vec<_> = sources.into_iter().map(|source| {
        std::thread::spawn(move || {
            let mut compiler = Compiler::new();
            compiler.compile_string(source)
        })
    }).collect();
    
    for handle in handles {
        assert!(handle.join().unwrap().is_ok());
    }
    
    let duration = start.elapsed();
    
    println!("Parallel compilation (4 files): {:?}", duration);
}
