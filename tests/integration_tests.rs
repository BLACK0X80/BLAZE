use blaze_compiler::compile;

#[test]
fn test_hello_program() {
    let source = r#"
        fn main() {
            let x: i32 = 5;
            let y: i32 = 10;
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_function_with_params() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            return a + b;
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_struct_definition() {
    let source = r#"
        struct Point {
            x: i32,
            y: i32,
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_complex_program() {
    let source = r#"
        fn fibonacci(n: i32) -> i32 {
            if n <= 1 {
                return n;
            }
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
        
        fn main() {
            let result = fibonacci(10);
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_ok());
}