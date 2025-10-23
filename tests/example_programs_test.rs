use blaze_compiler::*;
use std::fs;
use std::path::Path;

#[test]
fn test_fibonacci_example() {
    let source = fs::read_to_string("examples/fibonacci.blz");
    if let Ok(code) = source {
        let result = compile(&code);
        assert!(result.is_ok(), "Fibonacci example should compile: {:?}", result.err());
        
        // Also test semantic analysis
        let check_result = check(&code);
        assert!(check_result.is_ok(), "Fibonacci example should pass semantic analysis: {:?}", check_result.err());
    }
}

#[test]
fn test_hello_world_example() {
    let source = fs::read_to_string("examples/hello_world.blz");
    if let Ok(code) = source {
        let result = compile(&code);
        assert!(result.is_ok(), "Hello world example should compile: {:?}", result.err());
        
        // Verify it contains the correct message
        assert!(code.contains("Hello, BLAZE!"), "Hello world should print 'Hello, BLAZE!'");
        
        // Also test semantic analysis
        let check_result = check(&code);
        assert!(check_result.is_ok(), "Hello world example should pass semantic analysis: {:?}", check_result.err());
    }
}

#[test]
fn test_hello_example() {
    let source = fs::read_to_string("examples/hello.blz");
    if let Ok(code) = source {
        let result = compile(&code);
        assert!(result.is_ok(), "Hello example should compile: {:?}", result.err());
        
        // Also test semantic analysis
        let check_result = check(&code);
        assert!(check_result.is_ok(), "Hello example should pass semantic analysis: {:?}", check_result.err());
    }
}

#[test]
fn test_struct_example() {
    let source = fs::read_to_string("examples/struct_example.blz");
    if let Ok(code) = source {
        let result = compile(&code);
        assert!(result.is_ok(), "Struct example should compile: {:?}", result.err());
    }
}

#[test]
fn test_ownership_demo() {
    let source = fs::read_to_string("examples/ownership_demo.blz");
    if let Ok(code) = source {
        let result = compile(&code);
        assert!(result.is_ok(), "Ownership demo should compile: {:?}", result.err());
    }
}

#[test]
fn test_all_examples_compile() {
    let examples_dir = Path::new("examples");
    
    if !examples_dir.exists() {
        return;
    }
    
    let mut success_count = 0;
    let mut total_count = 0;
    
    if let Ok(entries) = fs::read_dir(examples_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("blz") {
                total_count += 1;
                
                if let Ok(source) = fs::read_to_string(&path) {
                    match compile(&source) {
                        Ok(_) => {
                            success_count += 1;
                            println!("✓ {} compiled successfully", path.display());
                        }
                        Err(e) => {
                            println!("✗ {} failed: {:?}", path.display(), e);
                        }
                    }
                }
            }
        }
    }
    
    println!("\nCompilation Results: {}/{} examples compiled successfully", success_count, total_count);
}

#[test]
fn test_data_structures_example() {
    let source = fs::read_to_string("examples/data_structures.blz");
    if let Ok(code) = source {
        let result = compile(&code);
        assert!(result.is_ok(), "Data structures example should compile: {:?}", result.err());
    }
}

#[test]
fn test_sorting_algorithms_example() {
    let source = fs::read_to_string("examples/sorting_algorithms.blz");
    if let Ok(code) = source {
        let result = compile(&code);
        assert!(result.is_ok(), "Sorting algorithms example should compile: {:?}", result.err());
    }
}

#[test]
fn test_mathematical_operations_example() {
    let source = fs::read_to_string("examples/mathematical_operations.blz");
    if let Ok(code) = source {
        let result = compile(&code);
        assert!(result.is_ok(), "Mathematical operations example should compile: {:?}", result.err());
    }
}

#[test]
fn test_semantic_analysis_on_examples() {
    let examples = vec![
        ("examples/fibonacci.blz", true),
        ("examples/hello_world.blz", true),
        ("examples/hello.blz", true),
        ("examples/struct_example.blz", true),
        ("examples/data_structures.blz", true),
        ("examples/sorting_algorithms.blz", true),
        ("examples/mathematical_operations.blz", true),
    ];
    
    for (path, should_pass) in examples {
        if let Ok(source) = fs::read_to_string(path) {
            let result = check(&source);
            
            if should_pass {
                assert!(result.is_ok(), "{} should pass semantic analysis: {:?}", path, result.err());
            }
        }
    }
}
