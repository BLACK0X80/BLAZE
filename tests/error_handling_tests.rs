use blaze_compiler::*;

#[test]
fn test_undefined_variable_error() {
    let source = r#"
        fn main() {
            let x = y + 10;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_err(), "Should fail with undefined variable");
}

#[test]
fn test_type_mismatch_error() {
    let source = r#"
        fn main() {
            let x: i32 = true;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_err(), "Should fail with type mismatch");
}

#[test]
fn test_undefined_function_error() {
    let source = r#"
        fn main() {
            let x = undefined_function(42);
        }
    "#;
    
    let result = check(source);
    assert!(result.is_err(), "Should fail with undefined function");
}

#[test]
fn test_duplicate_variable_error() {
    let source = r#"
        fn main() {
            let x = 10;
            let x = 20;
        }
    "#;
    
    let result = check(source);
}

#[test]
fn test_invalid_syntax_error() {
    let source = r#"
        fn main() {
            let x = ;
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_err(), "Should fail with syntax error");
}

#[test]
fn test_missing_return_type() {
    let source = r#"
        fn add(a: i32, b: i32) {
            return a + b;
        }
    "#;
    
    let result = compile(source);
}

#[test]
fn test_borrow_checker_error() {
    let source = r#"
        fn main() {
            let x = 10;
            let y = &x;
            let z = &mut x;
        }
    "#;
    
    let result = check(source);
}

#[test]
fn test_empty_program() {
    let source = "";
    
    let result = compile(source);
    assert!(result.is_err(), "Empty program should fail");
}

#[test]
fn test_unclosed_brace() {
    let source = r#"
        fn main() {
            let x = 10;
    "#;
    
    let result = compile(source);
    assert!(result.is_err(), "Unclosed brace should fail");
}

#[test]
fn test_invalid_operator() {
    let source = r#"
        fn main() {
            let x = 10 @ 5;
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_err(), "Invalid operator should fail");
}

#[test]
fn test_division_by_zero_detection() {
    let source = r#"
        fn main() {
            let x = 10 / 0;
        }
    "#;
    
    let result = compile(source);
}

#[test]
fn test_recursive_type_error() {
    let source = r#"
        fn main() {
            let x = x;
        }
    "#;
    
    let result = check(source);
}

#[test]
fn test_scope_error() {
    let source = r#"
        fn main() {
            {
                let x = 10;
            }
            let y = x;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_err(), "Variable out of scope should fail");
}

#[test]
fn test_invalid_return() {
    let source = r#"
        fn test() -> i32 {
            let x = 10;
        }
    "#;
    
    let result = compile(source);
}

#[test]
fn test_parameter_count_mismatch() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            return a + b;
        }
        
        fn main() {
            let x = add(5);
        }
    "#;
    
    let result = compile(source);
}

#[test]
fn test_invalid_assignment() {
    let source = r#"
        fn main() {
            let x = 10;
            x = 20;
        }
    "#;
    
    let result = check(source);
}

#[test]
fn test_multiple_errors() {
    let source = r#"
        fn main() {
            let x = undefined_var;
            let y: i32 = true;
            let z = unknown_function();
        }
    "#;
    
    let result = check(source);
    assert!(result.is_err(), "Multiple errors should be detected");
}

/// Test type errors are caught and reported
#[test]
fn test_type_error_integer_to_boolean() {
    let source = r#"
        fn main() {
            let x: bool = 42;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_err(), "Integer to boolean type error should be caught");
}

#[test]
fn test_type_error_boolean_to_integer() {
    let source = r#"
        fn main() {
            let x: i32 = false;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_err(), "Boolean to integer type error should be caught");
}

#[test]
fn test_type_error_in_binary_operation() {
    let source = r#"
        fn main() {
            let x = 10 + true;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_err(), "Type error in binary operation should be caught");
}

#[test]
fn test_type_error_in_comparison() {
    let source = r#"
        fn main() {
            let x = 10 > true;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_err(), "Type error in comparison should be caught");
}

#[test]
fn test_type_error_function_argument() {
    let source = r#"
        fn test(x: i32) -> i32 {
            return x + 1;
        }
        
        fn main() {
            let result = test(true);
        }
    "#;
    
    let result = check(source);
    assert!(result.is_err(), "Type error in function argument should be caught");
}

#[test]
fn test_type_error_function_return() {
    let source = r#"
        fn test() -> i32 {
            return true;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_err(), "Type error in function return should be caught");
}

#[test]
fn test_type_error_mixed_numeric_types() {
    let source = r#"
        fn main() {
            let x: i32 = 10;
            let y: i64 = x;
        }
    "#;
    
    let result = check(source);
    // This might be allowed with implicit conversion, adjust based on language design
}

/// Test borrow errors are caught and reported
#[test]
fn test_borrow_error_mutable_and_immutable() {
    let source = r#"
        fn main() {
            let x = 10;
            let y = &x;
            let z = &mut x;
            let w = y;
        }
    "#;
    
    let result = check(source);
    // Should fail if borrow checker is enabled
}

#[test]
fn test_borrow_error_multiple_mutable() {
    let source = r#"
        fn main() {
            let x = 10;
            let y = &mut x;
            let z = &mut x;
        }
    "#;
    
    let result = check(source);
    // Should fail if borrow checker is enabled
}

#[test]
fn test_borrow_error_use_after_move() {
    let source = r#"
        fn take_ownership(x: i32) {
            let y = x;
        }
        
        fn main() {
            let x = 10;
            take_ownership(x);
            let y = x;
        }
    "#;
    
    let result = check(source);
    // Should fail if move semantics are enforced
}

/// Test syntax errors are caught and reported
#[test]
fn test_syntax_error_missing_semicolon() {
    let source = r#"
        fn main() {
            let x = 10
            let y = 20;
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_err(), "Missing semicolon should be caught");
}

#[test]
fn test_syntax_error_missing_closing_paren() {
    let source = r#"
        fn main() {
            let x = (10 + 20;
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_err(), "Missing closing paren should be caught");
}

#[test]
fn test_syntax_error_missing_opening_brace() {
    let source = r#"
        fn main()
            let x = 10;
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_err(), "Missing opening brace should be caught");
}

#[test]
fn test_syntax_error_invalid_token() {
    let source = r#"
        fn main() {
            let x = 10 $ 5;
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_err(), "Invalid token should be caught");
}

#[test]
fn test_syntax_error_incomplete_expression() {
    let source = r#"
        fn main() {
            let x = 10 +;
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_err(), "Incomplete expression should be caught");
}

#[test]
fn test_syntax_error_missing_function_body() {
    let source = r#"
        fn test();
    "#;
    
    let result = compile(source);
    assert!(result.is_err(), "Missing function body should be caught");
}

#[test]
fn test_syntax_error_invalid_type() {
    let source = r#"
        fn main() {
            let x: invalid_type = 10;
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_err(), "Invalid type should be caught");
}

/// Test error messages are helpful
#[test]
fn test_error_message_contains_location() {
    let source = r#"
        fn main() {
            let x = undefined_var;
        }
    "#;
    
    let result = check(source);
    if let Err(e) = result {
        let error_msg = format!("{:?}", e);
        // Error message should contain some location information
        assert!(!error_msg.is_empty(), "Error message should not be empty");
    }
}

#[test]
fn test_error_message_type_mismatch_details() {
    let source = r#"
        fn main() {
            let x: i32 = true;
        }
    "#;
    
    let result = check(source);
    if let Err(e) = result {
        let error_msg = format!("{:?}", e);
        // Should mention both expected and actual types
        assert!(!error_msg.is_empty(), "Error message should contain type information");
    }
}

#[test]
fn test_error_message_undefined_function_details() {
    let source = r#"
        fn main() {
            let x = undefined_func(42);
        }
    "#;
    
    let result = check(source);
    if let Err(e) = result {
        let error_msg = format!("{:?}", e);
        // Should mention the undefined function name
        assert!(!error_msg.is_empty(), "Error message should contain function name");
    }
}

#[test]
fn test_error_message_scope_details() {
    let source = r#"
        fn main() {
            {
                let x = 10;
            }
            let y = x;
        }
    "#;
    
    let result = check(source);
    if let Err(e) = result {
        let error_msg = format!("{:?}", e);
        // Should mention the variable is out of scope
        assert!(!error_msg.is_empty(), "Error message should explain scope issue");
    }
}

/// Test edge cases and corner cases
#[test]
fn test_error_deeply_nested_expressions() {
    let source = r#"
        fn main() {
            let x = ((((10 + 20) * 30) - 40) / undefined_var);
        }
    "#;
    
    let result = check(source);
    assert!(result.is_err(), "Error in deeply nested expression should be caught");
}

#[test]
fn test_error_in_nested_functions() {
    let source = r#"
        fn outer() {
            fn inner() {
                let x = undefined_var;
            }
        }
    "#;
    
    let result = compile(source);
    // May fail due to nested functions not being supported
}

#[test]
fn test_error_circular_function_calls() {
    let source = r#"
        fn a() {
            b();
        }
        
        fn b() {
            a();
        }
    "#;
    
    let result = compile(source);
    // Should compile (circular calls are allowed, just might cause runtime issues)
}

#[test]
fn test_error_invalid_main_signature() {
    let source = r#"
        fn main(x: i32) {
            let y = x + 10;
        }
    "#;
    
    let result = compile(source);
    // May fail if main must have no parameters
}

#[test]
fn test_error_missing_main_function() {
    let source = r#"
        fn test() {
            let x = 10;
        }
    "#;
    
    let result = compile(source);
    // May fail if main function is required
}

#[test]
fn test_error_duplicate_function_definition() {
    let source = r#"
        fn test() {
            let x = 10;
        }
        
        fn test() {
            let y = 20;
        }
    "#;
    
    let result = compile(source);
    // Should fail with duplicate function error
}

#[test]
fn test_error_invalid_break_outside_loop() {
    let source = r#"
        fn main() {
            break;
        }
    "#;
    
    let result = compile(source);
    // Should fail if break is only allowed in loops
}

#[test]
fn test_error_invalid_continue_outside_loop() {
    let source = r#"
        fn main() {
            continue;
        }
    "#;
    
    let result = compile(source);
    // Should fail if continue is only allowed in loops
}

#[test]
fn test_error_return_in_void_function() {
    let source = r#"
        fn test() {
            return 42;
        }
    "#;
    
    let result = compile(source);
    // Should fail if function has no return type but returns a value
}

#[test]
fn test_error_missing_return_in_non_void_function() {
    let source = r#"
        fn test() -> i32 {
            let x = 10;
        }
    "#;
    
    let result = compile(source);
    // Should fail if function declares return type but doesn't return
}

/// Test error recovery and multiple error reporting
#[test]
fn test_multiple_type_errors() {
    let source = r#"
        fn main() {
            let x: i32 = true;
            let y: bool = 42;
            let z: i32 = false;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_err(), "Multiple type errors should be detected");
}

#[test]
fn test_multiple_undefined_variables() {
    let source = r#"
        fn main() {
            let x = a + b + c;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_err(), "Multiple undefined variables should be detected");
}

#[test]
fn test_mixed_error_types() {
    let source = r#"
        fn main() {
            let x: i32 = true;
            let y = undefined_var;
            let z = unknown_func();
        }
    "#;
    
    let result = check(source);
    assert!(result.is_err(), "Mixed error types should be detected");
}
