use blaze_compiler::{compile, compile_file};
use std::path::Path;

#[test]
fn test_basic_compilation() {
    let source = r#"
        fn main() {
            let x: i32 = 42;
            let y = x + 10;
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_ok(), "Basic compilation should succeed");
}

#[test]
fn test_function_compilation() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            return a + b;
        }
        
        fn main() {
            let result = add(5, 3);
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_ok(), "Function compilation should succeed");
}

#[test]
fn test_struct_compilation() {
    let source = r#"
        struct Point {
            x: i32,
            y: i32,
        }
        
        fn main() {
            let p = Point { x: 10, y: 20 };
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_ok(), "Struct compilation should succeed");
}

#[test]
fn test_control_flow() {
    let source = r#"
        fn max(a: i32, b: i32) -> i32 {
            if a > b {
                return a;
            } else {
                return b;
            }
        }
        
        fn main() {
            let result = max(10, 20);
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_ok(), "Control flow compilation should succeed");
}

#[test]
fn test_loops() {
    let source = r#"
        fn sum_to_n(n: i32) -> i32 {
            let mut sum = 0;
            let mut i = 1;
            while i <= n {
                sum = sum + i;
                i = i + 1;
            }
            return sum;
        }
        
        fn main() {
            let result = sum_to_n(10);
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_ok(), "Loop compilation should succeed");
}

#[test]
fn test_arrays() {
    let source = r#"
        fn main() {
            let arr = [1, 2, 3, 4, 5];
            let first = arr[0];
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_ok(), "Array compilation should succeed");
}

#[test]
fn test_type_inference() {
    let source = r#"
        fn main() {
            let x = 42;
            let y = 3.14;
            let z = true;
            let s = "hello";
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_ok(), "Type inference should work");
}

#[test]
fn test_pattern_matching() {
    let source = r#"
        enum Option {
            Some(i32),
            None,
        }
        
        fn unwrap_or(opt: Option, default: i32) -> i32 {
            match opt {
                Option::Some(value) => value,
                Option::None => default,
            }
        }
        
        fn main() {
            let x = Option::Some(42);
            let result = unwrap_or(x, 0);
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_ok(), "Pattern matching should work");
}

#[test]
fn test_generic_functions() {
    let source = r#"
        fn identity<T>(value: T) -> T {
            return value;
        }
        
        fn main() {
            let x = identity(42);
            let y = identity(3.14);
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_ok(), "Generic functions should work");
}

#[test]
fn test_closures() {
    let source = r#"
        fn main() {
            let add_one = |x| x + 1;
            let result = add_one(5);
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_ok(), "Closures should work");
}

#[test]
fn test_references() {
    let source = r#"
        fn increment(x: &mut i32) {
            *x = *x + 1;
        }
        
        fn main() {
            let mut x = 5;
            increment(&mut x);
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_ok(), "References should work");
}

#[test]
fn test_traits() {
    let source = r#"
        trait Display {
            fn show(&self);
        }
        
        struct Point {
            x: i32,
            y: i32,
        }
        
        impl Display for Point {
            fn show(&self) {
                // Implementation
            }
        }
        
        fn main() {
            let p = Point { x: 1, y: 2 };
            p.show();
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_ok(), "Traits should work");
}

#[test]
fn test_error_handling() {
    let source = r#"
        fn main() {
            let x: i32 = "invalid";
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_err(), "Type error should be detected");
}

#[test]
fn test_undefined_variable() {
    let source = r#"
        fn main() {
            let y = undefined_var;
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_err(), "Undefined variable should be detected");
}

#[test]
fn test_example_files() {
    let examples = vec![
        "examples/complete_example.blaze",
        "examples/advanced_types.blaze",
    ];
    
    for example in examples {
        let path = Path::new(example);
        if path.exists() {
            let result = compile_file(path);
            assert!(result.is_ok(), "Example file {} should compile", example);
        }
    }
}
