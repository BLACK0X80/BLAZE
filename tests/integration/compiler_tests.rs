use blaze::*;
use std::fs;

#[test]
fn test_basic_compilation() {
    let source = r#"
        fn main() {
            let x = 42;
            println("Hello, World!");
        }
    "#;
    
    let mut compiler = Compiler::new();
    let result = compiler.compile_string(source);
    
    assert!(result.is_ok(), "Basic compilation should succeed");
}

#[test]
fn test_type_checking() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            return a + b;
        }
        
        fn main() {
            let result = add(10, 20);
            assert_eq!(result, 30);
        }
    "#;
    
    let mut compiler = Compiler::new();
    let result = compiler.compile_string(source);
    
    assert!(result.is_ok(), "Type checking should pass");
}

#[test]
fn test_borrow_checker() {
    let source = r#"
        fn main() {
            let x = String::from("hello");
            let y = &x;
            let z = &x;
            
            println("{}", y);
            println("{}", z);
        }
    "#;
    
    let mut compiler = Compiler::new();
    let result = compiler.compile_string(source);
    
    assert!(result.is_ok(), "Multiple immutable borrows should be allowed");
}

#[test]
fn test_borrow_checker_error() {
    let source = r#"
        fn main() {
            let mut x = String::from("hello");
            let y = &mut x;
            let z = &mut x;
        }
    "#;
    
    let mut compiler = Compiler::new();
    let result = compiler.compile_string(source);
    
    assert!(result.is_err(), "Multiple mutable borrows should fail");
}

#[test]
fn test_lifetime_inference() {
    let source = r#"
        fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
            if x.len() > y.len() {
                return x;
            } else {
                return y;
            }
        }
        
        fn main() {
            let s1 = String::from("hello");
            let s2 = String::from("world!");
            let result = longest(&s1, &s2);
            println("{}", result);
        }
    "#;
    
    let mut compiler = Compiler::new();
    let result = compiler.compile_string(source);
    
    assert!(result.is_ok(), "Lifetime inference should work");
}

#[test]
fn test_generic_functions() {
    let source = r#"
        fn max<T>(a: T, b: T) -> T
        where T: Ord
        {
            if a > b {
                return a;
            } else {
                return b;
            }
        }
        
        fn main() {
            let m1 = max(10, 20);
            let m2 = max(3.14, 2.71);
            
            assert_eq!(m1, 20);
        }
    "#;
    
    let mut compiler = Compiler::new();
    let result = compiler.compile_string(source);
    
    assert!(result.is_ok(), "Generic functions should compile");
}

#[test]
fn test_trait_implementation() {
    let source = r#"
        trait Drawable {
            fn draw(&self);
        }
        
        struct Circle {
            radius: f64,
        }
        
        impl Drawable for Circle {
            fn draw(&self) {
                println("Drawing circle with radius {}", self.radius);
            }
        }
        
        fn main() {
            let circle = Circle { radius: 5.0 };
            circle.draw();
        }
    "#;
    
    let mut compiler = Compiler::new();
    let result = compiler.compile_string(source);
    
    assert!(result.is_ok(), "Trait implementation should work");
}

#[test]
fn test_pattern_matching() {
    let source = r#"
        enum Option<T> {
            Some(T),
            None,
        }
        
        fn main() {
            let x = Option::Some(42);
            
            match x {
                Option::Some(value) => println("Got value: {}", value),
                Option::None => println("Got nothing"),
            }
        }
    "#;
    
    let mut compiler = Compiler::new();
    let result = compiler.compile_string(source);
    
    assert!(result.is_ok(), "Pattern matching should compile");
}

#[test]
fn test_async_await() {
    let source = r#"
        async fn fetch_data() -> String {
            return String::from("data");
        }
        
        async fn main() {
            let data = fetch_data().await;
            println("Fetched: {}", data);
        }
    "#;
    
    let mut compiler = Compiler::new();
    let result = compiler.compile_string(source);
    
    assert!(result.is_ok(), "Async/await should compile");
}

#[test]
fn test_optimization_level_0() {
    let source = r#"
        fn main() {
            let x = 1 + 2 + 3;
            println("{}", x);
        }
    "#;
    
    let mut compiler = Compiler::new();
    compiler.set_optimization_level(0);
    let result = compiler.compile_string(source);
    
    assert!(result.is_ok(), "Compilation with -O0 should succeed");
}

#[test]
fn test_optimization_level_3() {
    let source = r#"
        fn fibonacci(n: i32) -> i32 {
            if n <= 1 {
                return n;
            }
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
        
        fn main() {
            let result = fibonacci(10);
            println("{}", result);
        }
    "#;
    
    let mut compiler = Compiler::new();
    compiler.set_optimization_level(3);
    let result = compiler.compile_string(source);
    
    assert!(result.is_ok(), "Compilation with -O3 should succeed");
}

#[test]
fn test_const_evaluation() {
    let source = r#"
        const MAX_SIZE: i32 = 100;
        const DOUBLE_MAX: i32 = MAX_SIZE * 2;
        
        fn main() {
            println("Max: {}", DOUBLE_MAX);
        }
    "#;
    
    let mut compiler = Compiler::new();
    let result = compiler.compile_string(source);
    
    assert!(result.is_ok(), "Const evaluation should work");
}

#[test]
fn test_macro_expansion() {
    let source = r#"
        macro_rules! println {
            ($fmt:expr) => {
                __builtin_println($fmt);
            };
            ($fmt:expr, $($arg:expr),*) => {
                __builtin_println($fmt, $($arg),*);
            };
        }
        
        fn main() {
            println!("Hello, {}!", "World");
        }
    "#;
    
    let mut compiler = Compiler::new();
    let result = compiler.compile_string(source);
    
    assert!(result.is_ok(), "Macro expansion should work");
}

#[test]
fn test_error_recovery() {
    let source = r#"
        fn main() {
            let x = 
            let y = 20;
        }
    "#;
    
    let mut compiler = Compiler::new();
    let result = compiler.compile_string(source);
    
    assert!(result.is_err(), "Syntax errors should be caught");
    
    if let Err(e) = result {
        assert!(e.to_string().contains("syntax"), "Error should mention syntax");
    }
}

#[test]
fn test_incremental_compilation() {
    let source1 = r#"
        fn helper() -> i32 {
            return 42;
        }
    "#;
    
    let source2 = r#"
        fn main() {
            let x = helper();
            println("{}", x);
        }
    "#;
    
    let mut compiler = Compiler::new();
    compiler.enable_incremental();
    
    let result1 = compiler.compile_string(source1);
    assert!(result1.is_ok());
    
    let result2 = compiler.compile_string(source2);
    assert!(result2.is_ok());
}

#[test]
fn test_cross_compilation_wasm() {
    let source = r#"
        fn main() {
            println("Hello from WASM!");
        }
    "#;
    
    let mut compiler = Compiler::new();
    compiler.set_target("wasm32-unknown-unknown");
    let result = compiler.compile_string(source);
    
    assert!(result.is_ok(), "Cross-compilation to WASM should work");
}
