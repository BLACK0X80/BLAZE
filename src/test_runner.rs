// Simple test runner that can be called from main
use crate::lexer::{lex, TokenType};
use crate::parser::{parse, Program};

pub fn run_tests() {
    println!("Running Blaze compiler tests...");
    
    // Test 1: Lexer
    println!("Test 1: Lexer");
    let source = "let x = 42;";
    match lex(source) {
        Ok(tokens) => {
            assert!(!tokens.is_empty());
            assert_eq!(tokens[0].token_type, TokenType::Let);
            println!("✓ Lexer test passed");
        }
        Err(e) => {
            println!("✗ Lexer test failed: {}", e);
            return;
        }
    }
    
    // Test 2: Parser
    println!("Test 2: Parser");
    let source = "let x = 42;";
    match lex(source) {
        Ok(tokens) => {
            match parse(tokens) {
                Ok(ast) => {
                    match ast {
                        Program::Statements(statements) => {
                            assert_eq!(statements.len(), 1);
                            println!("✓ Parser test passed");
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Parser test failed: {}", e);
                    return;
                }
            }
        }
        Err(e) => {
            println!("✗ Parser test failed: {}", e);
            return;
        }
    }
    
    // Test 3: Semantic Check
    println!("Test 3: Semantic Check");
    let source = "let x = 42;";
    match lex(source) {
        Ok(tokens) => {
            match parse(tokens) {
                Ok(ast) => {
                    match crate::semantic::check(&ast) {
                        Ok(_) => {
                            println!("✓ Semantic check test passed");
                        }
                        Err(e) => {
                            println!("✗ Semantic check test failed: {}", e);
                            return;
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Semantic check test failed: {}", e);
                    return;
                }
            }
        }
        Err(e) => {
            println!("✗ Semantic check test failed: {}", e);
            return;
        }
    }
    
    println!("All tests passed! 🎉");
}
