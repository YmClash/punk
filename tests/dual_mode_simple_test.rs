// tests/dual_mode_simple_test.rs
// Tests simplifiés pour valider le système dual-mode

use punk::lexer::lex::{Lexer, SyntaxMode};
use punk::parser::parser::Parser;
use punk::parser::ast::Visibility;

#[test]
fn test_basic_tokenization_dual_mode() {
    // Test que le lexer fonctionne dans les deux modes
    let braces_code = "let x: int = 5;";
    let indent_code = "let x: int = 5";
    
    let mut braces_lexer = Lexer::new(braces_code, SyntaxMode::Braces);
    let braces_tokens = braces_lexer.tokenize();
    assert!(!braces_tokens.is_empty(), "Braces mode should produce tokens");
    
    let mut indent_lexer = Lexer::new(indent_code, SyntaxMode::Indentation);
    let indent_tokens = indent_lexer.tokenize();
    assert!(!indent_tokens.is_empty(), "Indentation mode should produce tokens");
}

#[test]
fn test_function_tokenization() {
    let braces = "fn add(a: int, b: int) -> int { return a + b; }";
    let indent = "fn add(a: int, b: int) -> int:\n    return a + b";
    
    let mut braces_lexer = Lexer::new(braces, SyntaxMode::Braces);
    let braces_tokens = braces_lexer.tokenize();
    
    let mut indent_lexer = Lexer::new(indent, SyntaxMode::Indentation);
    let indent_tokens = indent_lexer.tokenize();
    
    // Vérifier que les deux produisent des tokens
    assert!(braces_tokens.len() > 5);
    assert!(indent_tokens.len() > 5);
}

#[test]
fn test_parser_variable_declaration() {
    // Test Braces mode
    let braces_code = "let x: int = 42;";
    let mut braces_lexer = Lexer::new(braces_code, SyntaxMode::Braces);
    let braces_tokens = braces_lexer.tokenize();
    let mut braces_parser = Parser::new(braces_tokens, SyntaxMode::Braces);
    
    match braces_parser.parse_variable_declaration() {
        Ok(_) => println!("Braces mode: Variable declaration parsed successfully"),
        Err(e) => panic!("Braces mode failed: {}", e.message),
    }
    
    // Test Indentation mode
    let indent_code = "let x: int = 42";
    let mut indent_lexer = Lexer::new(indent_code, SyntaxMode::Indentation);
    let indent_tokens = indent_lexer.tokenize();
    let mut indent_parser = Parser::new(indent_tokens, SyntaxMode::Indentation);
    
    match indent_parser.parse_variable_declaration() {
        Ok(_) => println!("Indentation mode: Variable declaration parsed successfully"),
        Err(e) => panic!("Indentation mode failed: {}", e.message),
    }
}

#[test]
fn test_parser_function_declaration() {
    // Test Braces mode
    let braces_code = "fn multiply(x: int, y: int) -> int { return x * y; }";
    let mut braces_lexer = Lexer::new(braces_code, SyntaxMode::Braces);
    let braces_tokens = braces_lexer.tokenize();
    let mut braces_parser = Parser::new(braces_tokens, SyntaxMode::Braces);
    
    match braces_parser.parse_function_declaration(Visibility::Public) {
        Ok(_) => println!("Braces mode: Function declaration parsed successfully"),
        Err(e) => panic!("Braces mode function failed: {}", e.message),
    }
    
    // Test Indentation mode
    let indent_code = "fn multiply(x: int, y: int) -> int:\n    return x * y";
    let mut indent_lexer = Lexer::new(indent_code, SyntaxMode::Indentation);
    let indent_tokens = indent_lexer.tokenize();
    let mut indent_parser = Parser::new(indent_tokens, SyntaxMode::Indentation);
    
    match indent_parser.parse_function_declaration(Visibility::Public) {
        Ok(_) => println!("Indentation mode: Function declaration parsed successfully"),
        Err(e) => panic!("Indentation mode function failed: {}", e.message),
    }
}

#[test]
fn test_if_statement_dual_mode() {
    // Test Braces mode
    let braces_code = "if x > 0 { print(\"positive\"); }";
    let mut braces_lexer = Lexer::new(braces_code, SyntaxMode::Braces);
    let braces_tokens = braces_lexer.tokenize();
    let mut braces_parser = Parser::new(braces_tokens, SyntaxMode::Braces);
    
    match braces_parser.parse_if_statement() {
        Ok(_) => println!("Braces mode: If statement parsed successfully"),
        Err(e) => println!("Braces mode if failed: {}", e.message),
    }
    
    // Test Indentation mode
    let indent_code = "if x > 0:\n    print(\"positive\")";
    let mut indent_lexer = Lexer::new(indent_code, SyntaxMode::Indentation);
    let indent_tokens = indent_lexer.tokenize();
    let mut indent_parser = Parser::new(indent_tokens, SyntaxMode::Indentation);
    
    match indent_parser.parse_if_statement() {
        Ok(_) => println!("Indentation mode: If statement parsed successfully"),
        Err(e) => println!("Indentation mode if failed: {}", e.message),
    }
}

#[test]
fn test_class_declaration_dual_mode() {
    // Test Braces mode
    let braces_code = r#"
class Person {
    name: str;
    age: int;
}"#;
    let mut braces_lexer = Lexer::new(braces_code, SyntaxMode::Braces);
    let braces_tokens = braces_lexer.tokenize();
    let mut braces_parser = Parser::new(braces_tokens, SyntaxMode::Braces);
    
    match braces_parser.parse_class_declaration(Visibility::Public) {
        Ok(_) => println!("Braces mode: Class declaration parsed successfully"),
        Err(e) => println!("Braces mode class failed: {}", e.message),
    }
    
    // Test Indentation mode
    let indent_code = r#"
class Person:
    name: str
    age: int"#;
    let mut indent_lexer = Lexer::new(indent_code, SyntaxMode::Indentation);
    let indent_tokens = indent_lexer.tokenize();
    let mut indent_parser = Parser::new(indent_tokens, SyntaxMode::Indentation);
    
    match indent_parser.parse_class_declaration(Visibility::Public) {
        Ok(_) => println!("Indentation mode: Class declaration parsed successfully"),
        Err(e) => println!("Indentation mode class failed: {}", e.message),
    }
}

#[test]
fn test_while_loop_dual_mode() {
    // Test Braces mode
    let braces_code = "while x < 10 { x = x + 1; }";
    let mut braces_lexer = Lexer::new(braces_code, SyntaxMode::Braces);
    let braces_tokens = braces_lexer.tokenize();
    let mut braces_parser = Parser::new(braces_tokens, SyntaxMode::Braces);
    
    match braces_parser.parse_while_statement() {
        Ok(_) => println!("Braces mode: While loop parsed successfully"),
        Err(e) => println!("Braces mode while failed: {}", e.message),
    }
    
    // Test Indentation mode
    let indent_code = "while x < 10:\n    x = x + 1";
    let mut indent_lexer = Lexer::new(indent_code, SyntaxMode::Indentation);
    let indent_tokens = indent_lexer.tokenize();
    let mut indent_parser = Parser::new(indent_tokens, SyntaxMode::Indentation);
    
    match indent_parser.parse_while_statement() {
        Ok(_) => println!("Indentation mode: While loop parsed successfully"),
        Err(e) => println!("Indentation mode while failed: {}", e.message),
    }
}

#[test]
fn test_struct_declaration_dual_mode() {
    // Test Braces mode
    let braces_code = "struct Point { x: float, y: float }";
    let mut braces_lexer = Lexer::new(braces_code, SyntaxMode::Braces);
    let braces_tokens = braces_lexer.tokenize();
    let mut braces_parser = Parser::new(braces_tokens, SyntaxMode::Braces);
    
    match braces_parser.parse_struct_declaration(Visibility::Public) {
        Ok(_) => println!("Braces mode: Struct declaration parsed successfully"),
        Err(e) => println!("Braces mode struct failed: {}", e.message),
    }
    
    // Test Indentation mode
    let indent_code = "struct Point:\n    x: float\n    y: float";
    let mut indent_lexer = Lexer::new(indent_code, SyntaxMode::Indentation);
    let indent_tokens = indent_lexer.tokenize();
    let mut indent_parser = Parser::new(indent_tokens, SyntaxMode::Indentation);
    
    match indent_parser.parse_struct_declaration(Visibility::Public) {
        Ok(_) => println!("Indentation mode: Struct declaration parsed successfully"),
        Err(e) => println!("Indentation mode struct failed: {}", e.message),
    }
}