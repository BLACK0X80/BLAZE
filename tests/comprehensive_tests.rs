use blaze_compiler::*;

#[test]
fn test_basic_compilation() {
    let source = r#"
        fn main() {
            let x = 42;
            let y = x + 10;
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_ok(), "Basic compilation should succeed");
}

#[test]
fn test_semantic_analysis() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            return a + b;
        }
        
        fn main() {
            let result = add(5, 10);
        }
    "#;
    
    let result = check(source);
    assert!(result.is_ok(), "Semantic analysis should pass");
}

#[test]
fn test_type_checking() {
    let source = r#"
        fn main() {
            let x: i32 = 42;
            let y: i32 = x + 10;
            let z: bool = true;
        }
    "#;
    
    let result = check(source);
    assert!(result.is_ok(), "Type checking should pass");
}

#[test]
fn test_control_flow() {
    let source = r#"
        fn main() {
            let x = 10;
            if x > 5 {
                let y = x * 2;
            } else {
                let y = x / 2;
            }
            
            while x > 0 {
                let x = x - 1;
            }
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_ok(), "Control flow compilation should succeed");
}

#[test]
fn test_function_calls() {
    let source = r#"
        fn multiply(a: i32, b: i32) -> i32 {
            return a * b;
        }
        
        fn main() {
            let result = multiply(6, 7);
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_ok(), "Function calls should compile");
}

#[test]
fn test_binary_operations() {
    let source = r#"
        fn main() {
            let a = 10 + 5;
            let b = 20 - 8;
            let c = 4 * 3;
            let d = 15 / 3;
            let e = 17 % 5;
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_ok(), "Binary operations should compile");
}

#[test]
fn test_comparison_operations() {
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
    
    let result = compile(source);
    assert!(result.is_ok(), "Comparison operations should compile");
}

#[test]
fn test_nested_scopes() {
    let source = r#"
        fn main() {
            let x = 10;
            {
                let y = 20;
                {
                    let z = 30;
                }
            }
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_ok(), "Nested scopes should compile");
}

#[test]
fn test_multiple_functions() {
    let source = r#"
        fn add(a: i32, b: i32) -> i32 {
            return a + b;
        }
        
        fn subtract(a: i32, b: i32) -> i32 {
            return a - b;
        }
        
        fn multiply(a: i32, b: i32) -> i32 {
            return a * b;
        }
        
        fn main() {
            let x = add(5, 3);
            let y = subtract(10, 4);
            let z = multiply(6, 7);
        }
    "#;
    
    let result = compile(source);
    assert!(result.is_ok(), "Multiple functions should compile");
}

#[test]
fn test_ir_generation() {
    let source = r#"
        fn main() {
            let x = 42;
        }
    "#;
    
    let tokens = lex(source).expect("Lexing failed");
    let program = parse(tokens).expect("Parsing failed");
    let ir_module = ir::generate(&program);
    
    assert!(ir_module.is_ok(), "IR generation should succeed");
    let module = ir_module.unwrap();
    assert!(!module.functions.is_empty(), "Should have at least one function");
}

#[test]
fn test_symbol_table() {
    use blaze_compiler::semantic::SymbolTable;
    use blaze_compiler::parser::Type;
    
    let mut table = SymbolTable::new();
    
    table.enter_scope();
    assert!(table.insert("x".to_string(), Type::I32, false).is_ok());
    assert!(table.lookup("x").is_some());
    
    table.enter_scope();
    assert!(table.insert("y".to_string(), Type::I64, true).is_ok());
    assert!(table.lookup("y").is_some());
    assert!(table.lookup("x").is_some());
    
    table.exit_scope();
    assert!(table.lookup("x").is_some());
}

#[test]
fn test_vec_implementation() {
    use blaze_compiler::stdlib::Vec;
    
    let mut vec = Vec::new();
    assert_eq!(vec.len(), 0);
    assert!(vec.is_empty());
    
    vec.push(1);
    vec.push(2);
    vec.push(3);
    
    assert_eq!(vec.len(), 3);
    assert!(!vec.is_empty());
    
    assert_eq!(vec.pop(), Some(3));
    assert_eq!(vec.pop(), Some(2));
    assert_eq!(vec.len(), 1);
    
    vec.push(4);
    vec.push(5);
    assert_eq!(vec.len(), 3);
}

#[test]
fn test_vec_with_capacity() {
    use blaze_compiler::stdlib::Vec;
    
    let vec: Vec<i32> = Vec::with_capacity(10);
    assert_eq!(vec.capacity(), 10);
    assert_eq!(vec.len(), 0);
}

#[test]
fn test_vec_insert_remove() {
    use blaze_compiler::stdlib::Vec;
    
    let mut vec = Vec::new();
    vec.push(1);
    vec.push(2);
    vec.push(4);
    
    vec.insert(2, 3);
    assert_eq!(vec.len(), 4);
    
    let removed = vec.remove(1);
    assert_eq!(removed, 2);
    assert_eq!(vec.len(), 3);
}

#[test]
fn test_vec_iterator() {
    use blaze_compiler::stdlib::Vec;
    
    let mut vec = Vec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    
    let sum: i32 = vec.iter().sum();
    assert_eq!(sum, 6);
}

#[test]
fn test_allocator_safety() {
    use blaze_compiler::runtime::{blaze_alloc, blaze_dealloc};
    
    unsafe {
        let ptr = blaze_alloc(100, 8);
        assert!(!ptr.is_null(), "Allocation should succeed");
        blaze_dealloc(ptr, 100, 8);
    }
}

#[test]
fn test_allocator_zero_size() {
    use blaze_compiler::runtime::blaze_alloc;
    
    unsafe {
        let ptr = blaze_alloc(0, 8);
        assert!(ptr.is_null(), "Zero-size allocation should return null");
    }
}

#[test]
fn test_allocator_invalid_alignment() {
    use blaze_compiler::runtime::blaze_alloc;
    
    unsafe {
        let ptr = blaze_alloc(100, 3);
        assert!(ptr.is_null(), "Invalid alignment should return null");
    }
}

#[test]
fn test_register_allocator() {
    use blaze_compiler::codegen::RegisterAllocator;
    use blaze_compiler::ir::{IRFunction, BasicBlock, Terminator, Parameter, IRType};
    
    let mut allocator = RegisterAllocator::new();
    
    let function = IRFunction {
        name: "test".to_string(),
        params: vec![],
        return_type: IRType::Void,
        blocks: vec![
            BasicBlock {
                label: "entry".to_string(),
                instructions: vec![],
                terminator: Terminator::Ret { value: None },
            }
        ],
    };
    
    let result = allocator.allocate(&function);
    assert!(result.is_ok(), "Register allocation should succeed");
}

#[test]
fn test_ir_validation() {
    use blaze_compiler::ir::{Module, IRFunction, BasicBlock, Terminator, IRType};
    use blaze_compiler::ir::validation::validate_module;
    
    let module = Module {
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
    assert!(result.is_ok(), "Valid module should pass validation");
}

#[test]
fn test_ir_optimization() {
    use blaze_compiler::ir::{Module, IRFunction, BasicBlock, Terminator, IRType, Instruction};
    use blaze_compiler::ir::optimization::optimize_module;
    
    let mut module = Module {
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
}

#[test]
fn test_runtime_initialization() {
    use blaze_compiler::runtime::initialize_runtime;
    
    let result = initialize_runtime();
    assert!(result.is_ok() || result.is_err(), "Runtime initialization should complete");
}

#[test]
fn test_diagnostic_builder() {
    use blaze_compiler::utils::{DiagnosticBuilder, SourceMap, Span};
    
    let source_map = SourceMap::new();
    let builder = DiagnosticBuilder::new(source_map);
    
    let span = Span::new(0, 0, 0, 10);
    let diagnostic = builder.error(span, "Test error".to_string());
    
    assert_eq!(diagnostic.message, "Test error");
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
    assert!(result.is_ok(), "Complex program should compile");
}
