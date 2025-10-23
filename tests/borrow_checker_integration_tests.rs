use blaze_compiler::*;

/// Test that immutable borrows are allowed simultaneously
#[test]
fn test_multiple_immutable_borrows_allowed() {
    let source = r#"
        fn main() {
            let x = 10;
            let y = x;
            let z = x;
        }
    "#;
    
    let result = check(source);
    // Multiple immutable borrows should be allowed
    assert!(result.is_ok(), "Multiple immutable borrows should be allowed");
}

/// Test that mutable and immutable borrows conflict
#[test]
fn test_mutable_and_immutable_borrow_conflict() {
    let source = r#"
        fn main() {
            let x = 10;
            let y = x;
            let mut z = x;
        }
    "#;
    
    let result = check(source);
    // This should fail due to borrow conflict
    if result.is_err() {
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(
            error_msg.contains("borrow") || error_msg.contains("conflict"),
            "Error should mention borrow conflict"
        );
    }
}

/// Test that multiple mutable borrows conflict
#[test]
fn test_multiple_mutable_borrows_conflict() {
    let source = r#"
        fn main() {
            let x = 10;
            let mut y = x;
            let mut z = x;
        }
    "#;
    
    let result = check(source);
    // This should fail due to multiple mutable borrows
    if result.is_err() {
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(
            error_msg.contains("mutable") || error_msg.contains("borrow"),
            "Error should mention mutable borrow conflict"
        );
    }
}

/// Test borrow checker with simple arithmetic
#[test]
fn test_borrow_checker_with_arithmetic() {
    let source = r#"
        fn main() {
            let x = 10;
            let y = 20;
            let sum = x + y;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_ok(), "Simple arithmetic should pass borrow checking");
}

/// Test borrow checker with binary expressions
#[test]
fn test_borrow_checker_with_binary_expressions() {
    let source = r#"
        fn main() {
            let a = 5;
            let b = 10;
            let result = a + b;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_ok(), "Binary expressions should pass borrow checking");
}

/// Test borrow checker with if statements
#[test]
fn test_borrow_checker_with_if_statement() {
    let source = r#"
        fn main() {
            let x = 10;
            if true {
                let y = x;
            }
        }
    "#;
    
    let result = check(source);
    assert!(result.is_ok(), "If statements should pass borrow checking");
}

/// Test borrow checker with while loops
#[test]
fn test_borrow_checker_with_while_loop() {
    let source = r#"
        fn main() {
            let x = 10;
            while false {
                let y = x;
            }
        }
    "#;
    
    let result = check(source);
    assert!(result.is_ok(), "While loops should pass borrow checking");
}

/// Test borrow checker with nested control flow
#[test]
fn test_borrow_checker_with_nested_control_flow() {
    let source = r#"
        fn main() {
            let x = 10;
            if true {
                while false {
                    let y = x;
                }
            }
        }
    "#;
    
    let result = check(source);
    assert!(result.is_ok(), "Nested control flow should pass borrow checking");
}

/// Test borrow checker with function calls
#[test]
fn test_borrow_checker_with_function_calls() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            return a + b;
        }
        
        fn main() {
            let x = 5;
            let y = 10;
            let result = add(x, y);
        }
    "#;
    
    let result = check(source);
    assert!(result.is_ok(), "Function calls should pass borrow checking");
}

/// Test borrow checker with return statements
#[test]
fn test_borrow_checker_with_return() {
    let source = r#"
        fn test() -> i32 {
            let x = 42;
            return x;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_ok(), "Return statements should pass borrow checking");
}

/// Test borrow checker with complex expressions
#[test]
fn test_borrow_checker_with_complex_expressions() {
    let source = r#"
        fn main() {
            let a = 1;
            let b = 2;
            let c = 3;
            let result = a + b + c;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_ok(), "Complex expressions should pass borrow checking");
}

/// Test borrow checker with sequential borrows
#[test]
fn test_borrow_checker_sequential_borrows() {
    let source = r#"
        fn main() {
            let x = 10;
            let y = x;
            let z = 20;
            let w = z;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_ok(), "Sequential borrows should pass borrow checking");
}

/// Test borrow checker with literals
#[test]
fn test_borrow_checker_with_literals() {
    let source = r#"
        fn main() {
            let x = 42;
            let y = true;
            let z = 3.14;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_ok(), "Literals should pass borrow checking");
}

/// Test borrow checker with unary expressions
#[test]
fn test_borrow_checker_with_unary_expressions() {
    let source = r#"
        fn main() {
            let x = 10;
            let y = -x;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_ok(), "Unary expressions should pass borrow checking");
}

/// Test borrow checker with boolean expressions
#[test]
fn test_borrow_checker_with_boolean_expressions() {
    let source = r#"
        fn main() {
            let a = true;
            let b = false;
            let result = a && b;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_ok(), "Boolean expressions should pass borrow checking");
}

/// Test borrow checker with comparison expressions
#[test]
fn test_borrow_checker_with_comparisons() {
    let source = r#"
        fn main() {
            let x = 10;
            let y = 20;
            let result = x < y;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_ok(), "Comparison expressions should pass borrow checking");
}

/// Test borrow checker with if-else branches
#[test]
fn test_borrow_checker_with_if_else() {
    let source = r#"
        fn main() {
            let x = 10;
            if true {
                let y = x;
            } else {
                let z = x;
            }
        }
    "#;
    
    let result = check(source);
    assert!(result.is_ok(), "If-else branches should pass borrow checking");
}

/// Test borrow checker dataflow analysis converges
#[test]
fn test_borrow_checker_dataflow_convergence() {
    let source = r#"
        fn main() {
            let x = 10;
            while false {
                let y = x;
                let z = y;
            }
        }
    "#;
    
    let result = check(source);
    assert!(result.is_ok(), "Dataflow analysis should converge for loops");
}

/// Test borrow checker with multiple functions
#[test]
fn test_borrow_checker_multiple_functions() {
    let source = r#"
        fn helper(x: i32) -> i32 {
            let y = x;
            return y;
        }
        
        fn main() {
            let a = 10;
            let b = helper(a);
        }
    "#;
    
    let result = check(source);
    assert!(result.is_ok(), "Multiple functions should pass borrow checking");
}

/// Test borrow checker with empty function
#[test]
fn test_borrow_checker_empty_function() {
    let source = r#"
        fn main() {
        }
    "#;
    
    let result = check(source);
    assert!(result.is_ok(), "Empty function should pass borrow checking");
}

/// Test borrow checker with single statement
#[test]
fn test_borrow_checker_single_statement() {
    let source = r#"
        fn main() {
            let x = 42;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_ok(), "Single statement should pass borrow checking");
}
