// tests/test_unified_blocks.rs
// Tests pour vérifier que le système de blocs unifié fonctionne correctement

use punk::lexer::lex::{Lexer, SyntaxMode};
use punk::parser::parser::Parser;
use punk::parser::ast::Visibility;

#[test]
fn test_while_loop_both_modes() {
    // Test Braces mode
    let braces_code = "while x < 10 { x = x + 1; y = y * 2; }";
    let mut braces_lexer = Lexer::new(braces_code, SyntaxMode::Braces);
    let braces_tokens = braces_lexer.tokenize();
    let mut braces_parser = Parser::new(braces_tokens, SyntaxMode::Braces);
    
    let braces_result = braces_parser.parse_while_statement();
    assert!(braces_result.is_ok(), "Braces while should parse: {:?}", braces_result.err());
    
    // Test Indentation mode
    let indent_code = "while x < 10:\n    x = x + 1\n    y = y * 2";
    let mut indent_lexer = Lexer::new(indent_code, SyntaxMode::Indentation);
    let indent_tokens = indent_lexer.tokenize();
    let mut indent_parser = Parser::new(indent_tokens, SyntaxMode::Indentation);
    
    let indent_result = indent_parser.parse_while_statement();
    assert!(indent_result.is_ok(), "Indent while should parse: {:?}", indent_result.err());
}

#[test]
fn test_for_loop_both_modes() {
    // Test Braces mode
    let braces_code = "for i in range(10) { print(i); sum = sum + i; }";
    let mut braces_lexer = Lexer::new(braces_code, SyntaxMode::Braces);
    let braces_tokens = braces_lexer.tokenize();
    let mut braces_parser = Parser::new(braces_tokens, SyntaxMode::Braces);
    
    let braces_result = braces_parser.parse_for_statement();
    assert!(braces_result.is_ok(), "Braces for should parse: {:?}", braces_result.err());
    
    // Test Indentation mode
    let indent_code = "for i in range(10):\n    print(i)\n    sum = sum + i";
    let mut indent_lexer = Lexer::new(indent_code, SyntaxMode::Indentation);
    let indent_tokens = indent_lexer.tokenize();
    let mut indent_parser = Parser::new(indent_tokens, SyntaxMode::Indentation);
    
    let indent_result = indent_parser.parse_for_statement();
    assert!(indent_result.is_ok(), "Indent for should parse: {:?}", indent_result.err());
}

#[test]
fn test_nested_blocks_both_modes() {
    // Test Braces mode with nested blocks
    let braces_code = r#"
while x < 10 {
    if x > 5 {
        print("big");
        x = x + 2;
    } else {
        print("small");
        x = x + 1;
    }
}"#;
    
    let mut braces_lexer = Lexer::new(braces_code, SyntaxMode::Braces);
    let braces_tokens = braces_lexer.tokenize();
    let mut braces_parser = Parser::new(braces_tokens, SyntaxMode::Braces);
    
    let braces_result = braces_parser.parse_while_statement();
    assert!(braces_result.is_ok(), "Nested braces should parse: {:?}", braces_result.err());
    
    // Test Indentation mode with nested blocks
    let indent_code = r#"
while x < 10:
    if x > 5:
        print("big")
        x = x + 2
    else:
        print("small")
        x = x + 1"#;
    
    let mut indent_lexer = Lexer::new(indent_code, SyntaxMode::Indentation);
    let indent_tokens = indent_lexer.tokenize();
    let mut indent_parser = Parser::new(indent_tokens, SyntaxMode::Indentation);
    
    let indent_result = indent_parser.parse_while_statement();
    assert!(indent_result.is_ok(), "Nested indent should parse: {:?}", indent_result.err());
}

#[test]
fn test_empty_block_both_modes() {
    // Test empty block in Braces mode
    let braces_code = "while true { }";
    let mut braces_lexer = Lexer::new(braces_code, SyntaxMode::Braces);
    let braces_tokens = braces_lexer.tokenize();
    let mut braces_parser = Parser::new(braces_tokens, SyntaxMode::Braces);
    
    let braces_result = braces_parser.parse_while_statement();
    assert!(braces_result.is_ok(), "Empty braces block should parse");
    
    // Test empty block in Indentation mode (using pass)
    let indent_code = "while true:\n    pass";
    let mut indent_lexer = Lexer::new(indent_code, SyntaxMode::Indentation);
    let indent_tokens = indent_lexer.tokenize();
    let mut indent_parser = Parser::new(indent_tokens, SyntaxMode::Indentation);
    
    let indent_result = indent_parser.parse_while_statement();
    assert!(indent_result.is_ok(), "Empty indent block should parse");
}

#[test]
fn test_single_statement_block() {
    // Test single statement in Braces mode
    let braces_code = "while x > 0 { x = x - 1; }";
    let mut braces_lexer = Lexer::new(braces_code, SyntaxMode::Braces);
    let braces_tokens = braces_lexer.tokenize();
    let mut braces_parser = Parser::new(braces_tokens, SyntaxMode::Braces);
    
    let braces_result = braces_parser.parse_while_statement();
    assert!(braces_result.is_ok(), "Single statement braces should parse");
    
    // Test single statement in Indentation mode
    let indent_code = "while x > 0:\n    x = x - 1";
    let mut indent_lexer = Lexer::new(indent_code, SyntaxMode::Indentation);
    let indent_tokens = indent_lexer.tokenize();
    let mut indent_parser = Parser::new(indent_tokens, SyntaxMode::Indentation);
    
    let indent_result = indent_parser.parse_while_statement();
    assert!(indent_result.is_ok(), "Single statement indent should parse");
}

#[test]
fn test_function_with_unified_block() {
    // Test function with Braces mode
    let braces_code = "fn calculate(x: int) -> int { let temp = x * 2; return temp + 10; }";
    let mut braces_lexer = Lexer::new(braces_code, SyntaxMode::Braces);
    let braces_tokens = braces_lexer.tokenize();
    let mut braces_parser = Parser::new(braces_tokens, SyntaxMode::Braces);
    
    let braces_result = braces_parser.parse_function_declaration(Visibility::Public);
    assert!(braces_result.is_ok(), "Function with braces should parse");
    
    // Test function with Indentation mode
    let indent_code = "fn calculate(x: int) -> int:\n    let temp = x * 2\n    return temp + 10";
    let mut indent_lexer = Lexer::new(indent_code, SyntaxMode::Indentation);
    let indent_tokens = indent_lexer.tokenize();
    let mut indent_parser = Parser::new(indent_tokens, SyntaxMode::Indentation);
    
    let indent_result = indent_parser.parse_function_declaration(Visibility::Public);
    assert!(indent_result.is_ok(), "Function with indent should parse");
}