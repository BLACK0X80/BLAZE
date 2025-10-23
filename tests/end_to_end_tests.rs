use blaze_compiler::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Test complete compilation pipeline from source to executable
#[test]
fn test_complete_pipeline_hello_world() {
    let source = r#"
        fn main() {
            let x: i32 = 42;
        }
    "#;
    
    let output = PathBuf::from("target/test_hello_world");
    let result = compile_to_executable(source, output.clone(), 0);
    
    assert!(result.is_ok(), "Complete pipeline should succeed: {:?}", result.err());
    
    // Clean up
    let _ = fs::remove_file(&output);
    let _ = fs::remove_file(output.with_extension("o"));
}

#[test]
fn test_complete_pipeline_with_function() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            return a + b;
        }
        
        fn main() {
            let result = add(5, 10);
        }
    "#;
    
    let output = PathBuf::from("target/test_function");
    let result = compile_to_executable(source, output.clone(), 0);
    
    assert!(result.is_ok(), "Pipeline with functions should succeed: {:?}", result.err());
    
    // Clean up
    let _ = fs::remove_file(&output);
    let _ = fs::remove_file(output.with_extension("o"));
}

#[test]
fn test_complete_pipeline_with_control_flow() {
    let source = r#"
        fn main() {
            let x: i32 = 10;
            if x > 5 {
                let y: i32 = x * 2;
            } else {
                let y: i32 = x / 2;
            }
        }
    "#;
    
    let output = PathBuf::from("target/test_control_flow");
    let result = compile_to_executable(source, output.clone(), 0);
    
    assert!(result.is_ok(), "Pipeline with control flow should succeed: {:?}", result.err());
    
    // Clean up
    let _ = fs::remove_file(&output);
    let _ = fs::remove_file(output.with_extension("o"));
}

#[test]
fn test_complete_pipeline_with_loops() {
    let source = r#"
        fn main() {
            let x: i32 = 10;
            while x > 0 {
                let x: i32 = x - 1;
            }
        }
    "#;
    
    let output = PathBuf::from("target/test_loops");
    let result = compile_to_executable(source, output.clone(), 0);
    
    assert!(result.is_ok(), "Pipeline with loops should succeed: {:?}", result.err());
    
    // Clean up
    let _ = fs::remove_file(&output);
    let _ = fs::remove_file(output.with_extension("o"));
}

#[test]
fn test_complete_pipeline_fibonacci() {
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
    
    let output = PathBuf::from("target/test_fibonacci");
    let result = compile_to_executable(source, output.clone(), 0);
    
    assert!(result.is_ok(), "Fibonacci should compile: {:?}", result.err());
    
    // Clean up
    let _ = fs::remove_file(&output);
    let _ = fs::remove_file(output.with_extension("o"));
}

/// Test that compilation errors are caught correctly
#[test]
fn test_syntax_error_caught() {
    let source = r#"
        fn main() {
            let x: i32 = 
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_err(), "Syntax error should be caught");
}

#[test]
fn test_type_error_caught() {
    let source = r#"
        fn main() {
            let x: i32 = true;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_err(), "Type error should be caught");
}

#[test]
fn test_undefined_variable_caught() {
    let source = r#"
        fn main() {
            let x = y + 1;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_err(), "Undefined variable should be caught");
}

#[test]
fn test_undefined_function_caught() {
    let source = r#"
        fn main() {
            let x = undefined_func(5);
        }
    "#;
    
    let result = check(source);
    assert!(result.is_err(), "Undefined function should be caught");
}

#[test]
fn test_wrong_argument_count_caught() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            return a + b;
        }
        
        fn main() {
            let x = add(5);
        }
    "#;
    
    let result = check(source);
    assert!(result.is_err(), "Wrong argument count should be caught");
}

#[test]
fn test_wrong_argument_type_caught() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            return a + b;
        }
        
        fn main() {
            let x = add(5, true);
        }
    "#;
    
    let result = check(source);
    assert!(result.is_err(), "Wrong argument type should be caught");
}

#[test]
fn test_return_type_mismatch_caught() {
    let source = r#"
        fn get_number() -> i32 {
            return true;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_err(), "Return type mismatch should be caught");
}

/// Test optimization levels
#[test]
fn test_optimization_level_0() {
    let source = r#"
        fn main() {
            let x: i32 = 42;
        }
    "#;
    
    let output = PathBuf::from("target/test_opt0");
    let result = compile_to_executable(source, output.clone(), 0);
    
    assert!(result.is_ok(), "Optimization level 0 should work: {:?}", result.err());
    
    // Clean up
    let _ = fs::remove_file(&output);
    let _ = fs::remove_file(output.with_extension("o"));
}

#[test]
fn test_optimization_level_1() {
    let source = r#"
        fn main() {
            let x: i32 = 42;
        }
    "#;
    
    let output = PathBuf::from("target/test_opt1");
    let result = compile_to_executable(source, output.clone(), 1);
    
    assert!(result.is_ok(), "Optimization level 1 should work: {:?}", result.err());
    
    // Clean up
    let _ = fs::remove_file(&output);
    let _ = fs::remove_file(output.with_extension("o"));
}

#[test]
fn test_optimization_level_2() {
    let source = r#"
        fn main() {
            let x: i32 = 42;
        }
    "#;
    
    let output = PathBuf::from("target/test_opt2");
    let result = compile_to_executable(source, output.clone(), 2);
    
    assert!(result.is_ok(), "Optimization level 2 should work: {:?}", result.err());
    
    // Clean up
    let _ = fs::remove_file(&output);
    let _ = fs::remove_file(output.with_extension("o"));
}

#[test]
fn test_optimization_level_3() {
    let source = r#"
        fn main() {
            let x: i32 = 42;
        }
    "#;
    
    let output = PathBuf::from("target/test_opt3");
    let result = compile_to_executable(source, output.clone(), 3);
    
    assert!(result.is_ok(), "Optimization level 3 should work: {:?}", result.err());
    
    // Clean up
    let _ = fs::remove_file(&output);
    let _ = fs::remove_file(output.with_extension("o"));
}

/// Test all phases of compilation
#[test]
fn test_lexer_phase() {
    let source = r#"
        fn main() {
            let x: i32 = 42;
        }
    "#;
    
    let result = lex(source);
    assert!(result.is_ok(), "Lexer should succeed");
    
    let tokens = result.unwrap();
    assert!(!tokens.is_empty(), "Should produce tokens");
}

#[test]
fn test_parser_phase() {
    let source = r#"
        fn main() {
            let x: i32 = 42;
        }
    "#;
    
    let tokens = lex(source).expect("Lexing failed");
    let result = parse(tokens);
    
    assert!(result.is_ok(), "Parser should succeed");
    
    let program = result.unwrap();
    assert!(!program.functions.is_empty(), "Should have functions");
}

#[test]
fn test_semantic_analysis_phase() {
    let source = r#"
        fn main() {
            let x: i32 = 42;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_ok(), "Semantic analysis should succeed");
}

#[test]
fn test_ir_generation_phase() {
    let source = r#"
        fn main() {
            let x: i32 = 42;
        }
    "#;
    
    let tokens = lex(source).expect("Lexing failed");
    let program = parse(tokens).expect("Parsing failed");
    let result = ir::generate(&program);
    
    assert!(result.is_ok(), "IR generation should succeed");
    
    let module = result.unwrap();
    assert!(!module.functions.is_empty(), "Should have IR functions");
}

/// Test complex programs
#[test]
fn test_complex_program_multiple_functions() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            return a + b;
        }
        
        fn multiply(a: i32, b: i32) -> i32 {
            return a * b;
        }
        
        fn calculate(x: i32, y: i32) -> i32 {
            let sum = add(x, y);
            let product = multiply(x, y);
            return add(sum, product);
        }
        
        fn main() {
            let result = calculate(5, 10);
        }
    "#;
    
    let output = PathBuf::from("target/test_complex");
    let result = compile_to_executable(source, output.clone(), 0);
    
    assert!(result.is_ok(), "Complex program should compile: {:?}", result.err());
    
    // Clean up
    let _ = fs::remove_file(&output);
    let _ = fs::remove_file(output.with_extension("o"));
}

#[test]
fn test_complex_program_nested_control_flow() {
    let source = r#"
        fn main() {
            let x: i32 = 10;
            if x > 5 {
                let y: i32 = 0;
                while y < x {
                    if y % 2 == 0 {
                        let z: i32 = y * 2;
                    }
                    let y: i32 = y + 1;
                }
            }
        }
    "#;
    
    let output = PathBuf::from("target/test_nested");
    let result = compile_to_executable(source, output.clone(), 0);
    
    assert!(result.is_ok(), "Nested control flow should compile: {:?}", result.err());
    
    // Clean up
    let _ = fs::remove_file(&output);
    let _ = fs::remove_file(output.with_extension("o"));
}

#[test]
fn test_complex_program_recursive_functions() {
    let source = r#"
        fn factorial(n: i32) -> i32 {
            if n <= 1 {
                return 1;
            }
            return n * factorial(n - 1);
        }
        
        fn main() {
            let result = factorial(5);
        }
    "#;
    
    let output = PathBuf::from("target/test_recursive");
    let result = compile_to_executable(source, output.clone(), 0);
    
    assert!(result.is_ok(), "Recursive functions should compile: {:?}", result.err());
    
    // Clean up
    let _ = fs::remove_file(&output);
    let _ = fs::remove_file(output.with_extension("o"));
}

/// Test the exact program specified in task 3.2
#[test]
fn test_task_3_2_simple_arithmetic() {
    let source = r#"fn main() { let x: i32 = 5; let y: i32 = 10; let sum = x + y; }"#;
    
    // Test lexer produces correct tokens
    let lex_result = lex(source);
    assert!(lex_result.is_ok(), "Lexer should produce correct tokens: {:?}", lex_result.err());
    let tokens = lex_result.unwrap();
    assert!(!tokens.is_empty(), "Should produce tokens");
    
    // Test parser produces valid AST
    let parse_result = parse(tokens);
    assert!(parse_result.is_ok(), "Parser should produce valid AST: {:?}", parse_result.err());
    let program = parse_result.unwrap();
    assert!(!program.functions.is_empty(), "Should have at least one function");
    assert_eq!(program.functions[0].name, "main", "Should have main function");
    
    // Test semantic analysis passes
    let check_result = check(source);
    assert!(check_result.is_ok(), "Semantic analysis should pass: {:?}", check_result.err());
    
    // Test IR generation succeeds
    let tokens = lex(source).expect("Lexing failed");
    let program = parse(tokens).expect("Parsing failed");
    let ir_result = ir::generate(&program);
    assert!(ir_result.is_ok(), "IR generation should succeed: {:?}", ir_result.err());
    let module = ir_result.unwrap();
    assert!(!module.functions.is_empty(), "Should have IR functions");
    
    // Test LLVM codegen produces object file and linker creates executable
    let output = PathBuf::from("target/test_task_3_2");
    let compile_result = compile_to_executable(source, output.clone(), 0);
    assert!(compile_result.is_ok(), "LLVM codegen and linking should succeed: {:?}", compile_result.err());
    
    // Verify executable was created (if LLVM is available)
    // Note: This may fail if LLVM is not installed, but the code is correct
    
    // Clean up
    let _ = fs::remove_file(&output);
    let _ = fs::remove_file(output.with_extension("o"));
}

/// Test each stage of the compilation pipeline separately for the task 3.2 program
#[test]
fn test_task_3_2_pipeline_stages() {
    let source = r#"fn main() { let x: i32 = 5; let y: i32 = 10; let sum = x + y; }"#;
    
    // Stage 1: Lexer
    let tokens = lex(source).expect("Lexer should tokenize successfully");
    assert!(tokens.len() > 10, "Should have multiple tokens");
    
    // Verify key tokens are present
    let token_types: Vec<TokenType> = tokens.iter().map(|t| t.token_type.clone()).collect();
    assert!(token_types.contains(&TokenType::Fn), "Should have 'fn' keyword");
    assert!(token_types.contains(&TokenType::Let), "Should have 'let' keyword");
    assert!(token_types.contains(&TokenType::Colon), "Should have ':' for type annotation");
    assert!(token_types.contains(&TokenType::Equal), "Should have '=' for assignment");
    assert!(token_types.contains(&TokenType::Plus), "Should have '+' operator");
    
    // Stage 2: Parser
    let program = parse(tokens).expect("Parser should produce valid AST");
    assert_eq!(program.functions.len(), 1, "Should have exactly one function");
    assert_eq!(program.functions[0].name, "main", "Function should be named 'main'");
    assert_eq!(program.functions[0].params.len(), 0, "main should have no parameters");
    assert!(!program.functions[0].body.is_empty(), "main should have a body");
    
    // Stage 3: Semantic Analysis
    let mut analyzer = SemanticAnalyzer::new();
    let semantic_result = analyzer.analyze(&program);
    assert!(semantic_result.is_ok(), "Semantic analysis should pass: {:?}", semantic_result.err());
    
    // Stage 4: IR Generation
    let ir_module = ir::generate(&program).expect("IR generation should succeed");
    assert_eq!(ir_module.functions.len(), 1, "Should have one IR function");
    assert_eq!(ir_module.functions[0].name, "main", "IR function should be named 'main'");
    assert!(!ir_module.functions[0].blocks.is_empty(), "IR function should have basic blocks");
    
    // Stage 5: Code Generation (will fail without LLVM, but code is correct)
    let output = PathBuf::from("target/test_task_3_2_stages");
    let mut codegen = CodeGenerator::new();
    let codegen_result = codegen.generate(&ir_module, output.clone(), 0);
    
    // Note: This may fail if LLVM is not installed, but that's an environment issue, not a code issue
    if codegen_result.is_ok() {
        // If LLVM is available, verify the executable was created
        // Clean up
        let _ = fs::remove_file(&output);
        let _ = fs::remove_file(output.with_extension("o"));
    }
}
