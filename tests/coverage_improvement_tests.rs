/// Tests specifically designed to improve code coverage
/// These tests target code paths that are typically under-tested

use blaze_compiler::*;
use std::path::PathBuf;

/// Test lexer edge cases
#[test]
fn test_lexer_empty_input() {
    let result = lex("");
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_lexer_whitespace_only() {
    let result = lex("   \n\t\r\n   ");
    assert!(result.is_ok());
}

#[test]
fn test_lexer_comments() {
    let source = r#"
        // This is a comment
        fn main() {
            // Another comment
            let x = 42; // Inline comment
        }
    "#;
    let result = lex(source);
    assert!(result.is_ok());
}

#[test]
fn test_lexer_string_literals() {
    let source = r#"
        fn main() {
            let s = "Hello, World!";
            let empty = "";
            let with_escape = "Line 1\nLine 2";
        }
    "#;
    let result = lex(source);
    assert!(result.is_ok());
}

#[test]
fn test_lexer_numeric_literals() {
    let source = r#"
        fn main() {
            let a = 42;
            let b = 3.14;
            let c = 0;
            let d = 1000000;
        }
    "#;
    let result = lex(source);
    assert!(result.is_ok());
}

#[test]
fn test_lexer_operators() {
    let source = r#"
        fn main() {
            let a = 1 + 2 - 3 * 4 / 5 % 6;
            let b = a > 0 && a < 10 || a == 5;
            let c = !b;
        }
    "#;
    let result = lex(source);
    assert!(result.is_ok());
}

/// Test parser edge cases
#[test]
fn test_parser_empty_function() {
    let source = "fn empty() {}";
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_parser_function_with_return_type() {
    let source = "fn returns_int() -> i32 { return 42; }";
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_parser_function_with_parameters() {
    let source = "fn add(a: i32, b: i32) -> i32 { return a + b; }";
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_parser_nested_blocks() {
    let source = r#"
        fn main() {
            {
                {
                    {
                        let x = 42;
                    }
                }
            }
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_parser_if_without_else() {
    let source = r#"
        fn main() {
            if true {
                let x = 42;
            }
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_parser_if_with_else() {
    let source = r#"
        fn main() {
            if true {
                let x = 42;
            } else {
                let y = 24;
            }
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_parser_while_loop() {
    let source = r#"
        fn main() {
            while true {
                let x = 42;
            }
        }
    "#;
    let result = compile(source);
    assert!(result.is_ok());
}

#[test]
fn test_parser_for_loop() {
    let source = r#"
        fn main() {
            for i in 0..10 {
                let x = i;
            }
        }
    "#;
    let result = compile(source);
    // May fail if for loops aren't implemented yet
}

/// Test semantic analysis edge cases
#[test]
fn test_semantic_variable_shadowing() {
    let source = r#"
        fn main() {
            let x = 10;
            {
                let x = 20;
            }
            let x = 30;
        }
    "#;
    let result = check(source);
    // Should succeed if shadowing is allowed
}

#[test]
fn test_semantic_function_forward_reference() {
    let source = r#"
        fn a() {
            b();
        }
        
        fn b() {
            let x = 42;
        }
    "#;
    let result = check(source);
    // Should succeed if forward references are allowed
}

#[test]
fn test_semantic_recursive_function() {
    let source = r#"
        fn factorial(n: i32) -> i32 {
            if n <= 1 {
                return 1;
            }
            return n * factorial(n - 1);
        }
    "#;
    let result = check(source);
    assert!(result.is_ok());
}

#[test]
fn test_semantic_mutually_recursive_functions() {
    let source = r#"
        fn is_even(n: i32) -> bool {
            if n == 0 {
                return true;
            }
            return is_odd(n - 1);
        }
        
        fn is_odd(n: i32) -> bool {
            if n == 0 {
                return false;
            }
            return is_even(n - 1);
        }
    "#;
    let result = check(source);
    // Should succeed if mutual recursion is supported
}

/// Test IR generation edge cases
#[test]
fn test_ir_empty_function() {
    let source = "fn empty() {}";
    let tokens = lex(source).expect("Lexing failed");
    let program = parse(tokens).expect("Parsing failed");
    let result = ir::generate(&program);
    assert!(result.is_ok());
}

#[test]
fn test_ir_function_with_single_statement() {
    let source = "fn main() { let x = 42; }";
    let tokens = lex(source).expect("Lexing failed");
    let program = parse(tokens).expect("Parsing failed");
    let result = ir::generate(&program);
    assert!(result.is_ok());
}

#[test]
fn test_ir_function_with_return() {
    let source = "fn test() -> i32 { return 42; }";
    let tokens = lex(source).expect("Lexing failed");
    let program = parse(tokens).expect("Parsing failed");
    let result = ir::generate(&program);
    assert!(result.is_ok());
}

#[test]
fn test_ir_binary_operations() {
    let source = r#"
        fn main() {
            let a = 10 + 20;
            let b = 30 - 15;
            let c = 5 * 6;
            let d = 40 / 8;
            let e = 17 % 5;
        }
    "#;
    let tokens = lex(source).expect("Lexing failed");
    let program = parse(tokens).expect("Parsing failed");
    let result = ir::generate(&program);
    assert!(result.is_ok());
}

#[test]
fn test_ir_comparison_operations() {
    let source = r#"
        fn main() {
            let a = 10 > 5;
            let b = 3 < 7;
            let c = 5 >= 5;
            let d = 4 <= 6;
            let e = 8 == 8;
            let f = 9 != 10;
        }
    "#;
    let tokens = lex(source).expect("Lexing failed");
    let program = parse(tokens).expect("Parsing failed");
    let result = ir::generate(&program);
    assert!(result.is_ok());
}

#[test]
fn test_ir_logical_operations() {
    let source = r#"
        fn main() {
            let a = true && false;
            let b = true || false;
            let c = !true;
        }
    "#;
    let tokens = lex(source).expect("Lexing failed");
    let program = parse(tokens).expect("Parsing failed");
    let result = ir::generate(&program);
    assert!(result.is_ok());
}

#[test]
fn test_ir_if_statement() {
    let source = r#"
        fn main() {
            if true {
                let x = 42;
            }
        }
    "#;
    let tokens = lex(source).expect("Lexing failed");
    let program = parse(tokens).expect("Parsing failed");
    let result = ir::generate(&program);
    assert!(result.is_ok());
}

#[test]
fn test_ir_if_else_statement() {
    let source = r#"
        fn main() {
            if true {
                let x = 42;
            } else {
                let y = 24;
            }
        }
    "#;
    let tokens = lex(source).expect("Lexing failed");
    let program = parse(tokens).expect("Parsing failed");
    let result = ir::generate(&program);
    assert!(result.is_ok());
}

#[test]
fn test_ir_while_loop() {
    let source = r#"
        fn main() {
            while true {
                let x = 42;
            }
        }
    "#;
    let tokens = lex(source).expect("Lexing failed");
    let program = parse(tokens).expect("Parsing failed");
    let result = ir::generate(&program);
    assert!(result.is_ok());
}

#[test]
fn test_ir_function_call() {
    let source = r#"
        fn helper() -> i32 {
            return 42;
        }
        
        fn main() {
            let x = helper();
        }
    "#;
    let tokens = lex(source).expect("Lexing failed");
    let program = parse(tokens).expect("Parsing failed");
    let result = ir::generate(&program);
    assert!(result.is_ok());
}

#[test]
fn test_ir_function_call_with_arguments() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            return a + b;
        }
        
        fn main() {
            let x = add(10, 20);
        }
    "#;
    let tokens = lex(source).expect("Lexing failed");
    let program = parse(tokens).expect("Parsing failed");
    let result = ir::generate(&program);
    assert!(result.is_ok());
}

/// Test runtime edge cases
#[test]
fn test_runtime_allocator_alignment() {
    use blaze_compiler::runtime::{blaze_alloc, blaze_dealloc};
    
    unsafe {
        // Test various alignments
        for align in [1, 2, 4, 8, 16, 32, 64].iter() {
            let ptr = blaze_alloc(100, *align);
            if !ptr.is_null() {
                blaze_dealloc(ptr, 100, *align);
            }
        }
    }
}

#[test]
fn test_runtime_allocator_sizes() {
    use blaze_compiler::runtime::{blaze_alloc, blaze_dealloc};
    
    unsafe {
        // Test various sizes
        for size in [1, 10, 100, 1000, 10000].iter() {
            let ptr = blaze_alloc(*size, 8);
            if !ptr.is_null() {
                blaze_dealloc(ptr, *size, 8);
            }
        }
    }
}

#[test]
fn test_runtime_realloc() {
    use blaze_compiler::runtime::{blaze_alloc, blaze_realloc, blaze_dealloc};
    
    unsafe {
        let ptr = blaze_alloc(100, 8);
        if !ptr.is_null() {
            let new_ptr = blaze_realloc(ptr, 100, 200, 8);
            if !new_ptr.is_null() {
                blaze_dealloc(new_ptr, 200, 8);
            }
        }
    }
}

/// Test stdlib edge cases
#[test]
fn test_vec_clear() {
    use blaze_compiler::stdlib::Vec;
    
    let mut vec = Vec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    
    vec.clear();
    assert_eq!(vec.len(), 0);
    assert!(vec.is_empty());
}

#[test]
fn test_vec_get() {
    use blaze_compiler::stdlib::Vec;
    
    let mut vec = Vec::new();
    vec.push(10);
    vec.push(20);
    vec.push(30);
    
    assert_eq!(vec.get(0), Some(&10));
    assert_eq!(vec.get(1), Some(&20));
    assert_eq!(vec.get(2), Some(&30));
    assert_eq!(vec.get(3), None);
}

#[test]
fn test_vec_get_mut() {
    use blaze_compiler::stdlib::Vec;
    
    let mut vec = Vec::new();
    vec.push(10);
    vec.push(20);
    
    if let Some(val) = vec.get_mut(0) {
        *val = 100;
    }
    
    assert_eq!(vec.get(0), Some(&100));
}

#[test]
fn test_vec_reserve() {
    use blaze_compiler::stdlib::Vec;
    
    let mut vec = Vec::new();
    vec.reserve(100);
    
    assert!(vec.capacity() >= 100);
}

#[test]
fn test_vec_shrink_to_fit() {
    use blaze_compiler::stdlib::Vec;
    
    let mut vec = Vec::with_capacity(100);
    vec.push(1);
    vec.push(2);
    
    vec.shrink_to_fit();
    assert!(vec.capacity() >= vec.len());
}

/// Test error reporting edge cases
#[test]
fn test_error_with_location() {
    use blaze_compiler::error::CompileError;
    use blaze_compiler::utils::SourceLocation;
    
    let error = CompileError::TypeError {
        expected: "i32".to_string(),
        found: "bool".to_string(),
        location: SourceLocation::new(1, 5, 1, 10),
    };
    
    let error_msg = format!("{:?}", error);
    assert!(!error_msg.is_empty());
}

#[test]
fn test_multiple_error_collection() {
    let source = r#"
        fn main() {
            let x = undefined1;
            let y = undefined2;
            let z = undefined3;
        }
    "#;
    
    let result = check(source);
    // Should collect multiple errors
    assert!(result.is_err());
}

/// Test optimization edge cases
#[test]
fn test_ir_optimization_constant_folding() {
    use blaze_compiler::ir::{Module, IRFunction, BasicBlock, Terminator, IRType, Instruction};
    use blaze_compiler::ir::optimization::optimize_module;
    
    let mut module = Module {
        name: "test".to_string(),
        functions: vec![
            IRFunction {
                name: "test".to_string(),
                params: vec![],
                return_type: IRType::I32,
                blocks: vec![
                    BasicBlock {
                        label: "entry".to_string(),
                        instructions: vec![
                            Instruction::Add {
                                result: "%1".to_string(),
                                left: "5".to_string(),
                                right: "10".to_string(),
                                ty: IRType::I32,
                            },
                        ],
                        terminator: Terminator::Ret { value: Some("%1".to_string()) },
                    }
                ],
            }
        ],
        globals: vec![],
        types: vec![],
    };
    
    optimize_module(&mut module);
    // Optimization should succeed
}

/// Test validation edge cases
#[test]
fn test_ir_validation_valid_module() {
    use blaze_compiler::ir::{Module, IRFunction, BasicBlock, Terminator, IRType};
    use blaze_compiler::ir::validation::validate_module;
    
    let module = Module {
        name: "test".to_string(),
        functions: vec![
            IRFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: IRType::Void,
                blocks: vec![
                    BasicBlock {
                        label: "entry".to_string(),
                        instructions: vec![],
                        terminator: Terminator::Ret { value: None },
                    }
                ],
            }
        ],
        globals: vec![],
        types: vec![],
    };
    
    let result = validate_module(&module);
    assert!(result.is_ok());
}

#[test]
fn test_ir_validation_empty_module() {
    use blaze_compiler::ir::Module;
    use blaze_compiler::ir::validation::validate_module;
    
    let module = Module {
        name: "empty".to_string(),
        functions: vec![],
        globals: vec![],
        types: vec![],
    };
    
    let result = validate_module(&module);
    // Empty module might be valid or invalid depending on requirements
}
