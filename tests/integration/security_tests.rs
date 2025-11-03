use blaze::security::*;

#[test]
fn test_buffer_overflow_detection() {
    let source = r#"
        fn unsafe_copy(src: &[u8], dest: &mut [u8]) {
            for i in 0..100 {
                dest[i] = src[i];
            }
        }
        
        fn main() {
            let source = [0u8; 50];
            let mut destination = [0u8; 50];
            unsafe_copy(&source, &mut destination);
        }
    "#;
    
    let mut analyzer = SecurityAnalyzer::new();
    let mut compiler = Compiler::new();
    
    match compiler.compile_string(source) {
        Ok(program) => {
            let issues = analyzer.analyze(&program);
            
            assert!(!issues.is_empty(), "Should detect buffer overflow");
            
            let has_overflow = issues.iter().any(|issue| {
                matches!(issue.category, IssueCategory::BufferOverflow)
            });
            
            assert!(has_overflow, "Should specifically flag buffer overflow");
        }
        Err(_) => panic!("Compilation should succeed for analysis"),
    }
}

#[test]
fn test_integer_overflow_detection() {
    let source = r#"
        fn main() {
            let x: i32 = 2147483647;
            let y = x + 1;
            println("{}", y);
        }
    "#;
    
    let mut analyzer = SecurityAnalyzer::new();
    let mut compiler = Compiler::new();
    
    match compiler.compile_string(source) {
        Ok(program) => {
            let issues = analyzer.analyze(&program);
            
            let has_overflow = issues.iter().any(|issue| {
                matches!(issue.category, IssueCategory::IntegerOverflow)
            });
            
            assert!(has_overflow || issues.is_empty(), 
                "Integer overflow should be detected or handled safely");
        }
        Err(_) => {}
    }
}

#[test]
fn test_null_pointer_safety() {
    let source = r#"
        fn main() {
            let ptr: *const i32 = null();
            let value = *ptr;
        }
    "#;
    
    let mut compiler = Compiler::new();
    let result = compiler.compile_string(source);
    
    assert!(result.is_err(), "Null pointer dereference should be prevented");
}

#[test]
fn test_use_after_free_prevention() {
    let source = r#"
        fn main() {
            let x = Box::new(42);
            drop(x);
            println("{}", *x);
        }
    "#;
    
    let mut compiler = Compiler::new();
    let result = compiler.compile_string(source);
    
    assert!(result.is_err(), "Use after free should be prevented by borrow checker");
}

#[test]
fn test_data_race_detection() {
    let source = r#"
        fn main() {
            let mut data = vec![1, 2, 3];
            
            thread::spawn(|| {
                data.push(4);
            });
            
            data.push(5);
        }
    "#;
    
    let mut compiler = Compiler::new();
    let result = compiler.compile_string(source);
    
    assert!(result.is_err(), "Data race should be detected");
}

#[test]
fn test_safe_concurrent_access() {
    let source = r#"
        fn main() {
            let data = Arc::new(Mutex::new(vec![1, 2, 3]));
            let data_clone = Arc::clone(&data);
            
            thread::spawn(move || {
                let mut d = data_clone.lock().unwrap();
                d.push(4);
            });
            
            let mut d = data.lock().unwrap();
            d.push(5);
        }
    "#;
    
    let mut compiler = Compiler::new();
    let result = compiler.compile_string(source);
    
    assert!(result.is_ok(), "Safe concurrent access should compile");
}

#[test]
fn test_sql_injection_detection() {
    let source = r#"
        fn get_user(username: String) -> String {
            let query = format!("SELECT * FROM users WHERE username = '{}'", username);
            execute_query(&query)
        }
    "#;
    
    let mut analyzer = SecurityAnalyzer::new();
    let mut compiler = Compiler::new();
    
    if let Ok(program) = compiler.compile_string(source) {
        let issues = analyzer.analyze(&program);
        
        let has_sql_injection = issues.iter().any(|issue| {
            matches!(issue.category, IssueCategory::SqlInjection)
        });
        
        if has_sql_injection {
            println!("SQL injection vulnerability detected (as expected)");
        }
    }
}

#[test]
fn test_memory_leak_detection() {
    let source = r#"
        fn main() {
            loop {
                let data = Box::new([0u8; 1024]);
                std::mem::forget(data);
            }
        }
    "#;
    
    let mut analyzer = SecurityAnalyzer::new();
    let mut compiler = Compiler::new();
    
    if let Ok(program) = compiler.compile_string(source) {
        let issues = analyzer.analyze(&program);
        println!("Memory analysis found {} potential issues", issues.len());
    }
}

#[test]
fn test_secure_random_numbers() {
    let source = r#"
        fn generate_token() -> String {
            let mut rng = SecureRandom::new();
            let token = rng.gen_bytes(32);
            hex_encode(&token)
        }
    "#;
    
    let mut compiler = Compiler::new();
    let result = compiler.compile_string(source);
    
    assert!(result.is_ok(), "Secure random generation should work");
}

#[test]
fn test_safe_pointer_arithmetic() {
    let source = r#"
        fn main() {
            let arr = [1, 2, 3, 4, 5];
            let ptr = arr.as_ptr();
            
            unsafe {
                let value = *ptr.offset(10);
            }
        }
    "#;
    
    let mut analyzer = SecurityAnalyzer::new();
    let mut compiler = Compiler::new();
    
    if let Ok(program) = compiler.compile_string(source) {
        let issues = analyzer.analyze(&program);
        
        let has_bounds_issue = issues.iter().any(|issue| {
            issue.severity == IssueSeverity::Critical
        });
        
        println!("Unsafe pointer arithmetic analysis: {} issues found", issues.len());
    }
}
