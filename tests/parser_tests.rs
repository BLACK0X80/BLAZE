use blaze_compiler::{lex, parse};
use blaze_compiler::parser::{Statement, Expression, BinaryOp};

#[test]
fn test_parse_let_statement() {
    let source = "fn main() { let x: i32 = 5; }";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_parse_function() {
    let source = "fn add(a: i32, b: i32) -> i32 { return a + b; }";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_parse_arithmetic() {
    let source = "fn main() { let x = 2 + 3 * 4; }";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_parse_if_statement() {
    let source = "fn main() { if true { let x = 1; } else { let y = 2; } }";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_parse_while_loop() {
    let source = "fn main() { while true { let x = 1; } }";
    let tokens = lex(source).unwrap();
    let program = parse(tokens).unwrap();
    
    assert_eq!(program.items.len(), 1);
}