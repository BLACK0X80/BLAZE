use blaze_compiler::lexer::{lex, TokenType};

#[test]
fn test_keywords() {
    let source = "let mut fn return if else while";
    let tokens = lex(source).unwrap();
    
    assert!(matches!(tokens[0].token_type, TokenType::Let));
    assert!(matches!(tokens[1].token_type, TokenType::Mut));
    assert!(matches!(tokens[2].token_type, TokenType::Fn));
    assert!(matches!(tokens[3].token_type, TokenType::Return));
    assert!(matches!(tokens[4].token_type, TokenType::If));
    assert!(matches!(tokens[5].token_type, TokenType::Else));
    assert!(matches!(tokens[6].token_type, TokenType::While));
}

#[test]
fn test_numbers() {
    let source = "42 3.14";
    let tokens = lex(source).unwrap();
    
    assert_eq!(tokens[0].token_type, TokenType::IntLit(42));
    assert_eq!(tokens[1].token_type, TokenType::FloatLit(3.14));
}

#[test]
fn test_strings() {
    let source = r#""hello world""#;
    let tokens = lex(source).unwrap();
    
    assert_eq!(tokens[0].token_type, TokenType::StringLit("hello world".to_string()));
}

#[test]
fn test_operators() {
    let source = "+ - * / == != < >";
    let tokens = lex(source).unwrap();
    
    assert!(matches!(tokens[0].token_type, TokenType::Plus));
    assert!(matches!(tokens[1].token_type, TokenType::Minus));
    assert!(matches!(tokens[2].token_type, TokenType::Star));
    assert!(matches!(tokens[3].token_type, TokenType::Slash));
    assert!(matches!(tokens[4].token_type, TokenType::EqualEqual));
    assert!(matches!(tokens[5].token_type, TokenType::BangEqual));
    assert!(matches!(tokens[6].token_type, TokenType::Less));
    assert!(matches!(tokens[7].token_type, TokenType::Greater));
}

#[test]
fn test_identifiers() {
    let source = "foo bar_123";
    let tokens = lex(source).unwrap();
    
    assert_eq!(tokens[0].token_type, TokenType::Ident("foo".to_string()));
    assert_eq!(tokens[1].token_type, TokenType::Ident("bar_123".to_string()));
}

#[test]
fn test_comments() {
    let source = "let x = 5; // comment\nlet y = 10;";
    let tokens = lex(source).unwrap();
    
    assert!(matches!(tokens[0].token_type, TokenType::Let));
    assert_eq!(tokens[1].token_type, TokenType::Ident("x".to_string()));
}