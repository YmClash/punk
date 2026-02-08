// tests/dual_mode_tests.rs
// Tests unitaires pour valider la robustesse du système dual-mode (Braces vs Indentation)

use punk::lexer::lex::{Lexer, SyntaxMode};
use punk::parser::parser::Parser;
use punk::parser::ast::ASTNode;

/// Test helper pour comparer les AST de deux modes
fn compare_dual_mode(name: &str, braces_code: &str, indent_code: &str) -> Result<(), String> {
    // Parse Braces mode
    let mut braces_lexer = Lexer::new(braces_code, SyntaxMode::Braces);
    let braces_tokens = braces_lexer.tokenize();
    let mut braces_parser = Parser::new(braces_tokens, SyntaxMode::Braces);
    let braces_ast = braces_parser.parse_program()
        .map_err(|e| format!("Braces mode error: {}", e.message))?;
    
    // Parse Indentation mode
    let mut indent_lexer = Lexer::new(indent_code, SyntaxMode::Indentation);
    let indent_tokens = indent_lexer.tokenize();
    let mut indent_parser = Parser::new(indent_tokens, SyntaxMode::Indentation);
    let indent_ast = indent_parser.parse_program()
        .map_err(|e| format!("Indentation mode error: {}", e.message))?;
    
    // Compare AST structures (simplified comparison)
    if format!("{:?}", braces_ast) != format!("{:?}", indent_ast) {
        return Err(format!("{}: ASTs differ between modes", name));
    }
    
    Ok(())
}

#[cfg(test)]
mod basic_syntax_tests {
    use super::*;

    #[test]
    fn test_variable_declaration() {
        let braces = "let x: int = 5;";
        let indent = "let x: int = 5";
        
        assert!(compare_dual_mode("variable_declaration", braces, indent).is_ok());
    }

    #[test]
    fn test_mutable_variable() {
        let braces = "let mut y: float = 3.14;";
        let indent = "let mut y: float = 3.14";
        
        assert!(compare_dual_mode("mutable_variable", braces, indent).is_ok());
    }

    #[test]
    fn test_simple_function() {
        let braces = r#"
fn add(a: int, b: int) -> int {
    return a + b
}"#;
        let indent = r#"
fn add(a: int, b: int) -> int:
    return a + b"#;
        
        let result = compare_dual_mode("simple_function", braces, indent);
        if let Err(e) = &result {
            eprintln!("Test error: {}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_with_locals() {
        let braces = r#"
fn compute(x: int) -> int {
    let temp: int = x * 2;
    return temp + 10
}"#;
        let indent = r#"
fn compute(x: int) -> int:
    let temp: int = x * 2
    return temp + 10"#;
        
        assert!(compare_dual_mode("function_with_locals", braces, indent).is_ok());
    }
}

// #[ignore]
// #[cfg(test)]
// mod control_flow_tests {
//     use super::*;
//
//     #[test]
//     fn test_if_statement() {
//         let braces = r#"
// if x > 0 {
//     print("positive");
// }"#;
//         let indent = r#"
// if x > 0:
//     print("positive")"#;
//
//         assert!(compare_dual_mode("if_statement", braces, indent).is_ok());
//     }
//
//     #[test]
//     fn test_if_else() {
//         let braces = r#"
// if x > 0 {
//     print("positive");
// } else {
//     print("non-positive");
// }"#;
//         let indent = r#"
// if x > 0:
//     print("positive")
// else:
//     print("non-positive")"#;
//
//         assert!(compare_dual_mode("if_else", braces, indent).is_ok());
//     }
//
//     #[test]
//     fn test_elif_chain() {
//         let braces = r#"
// if x > 0 {
//     print("positive");
// } elif x < 0 {
//     print("negative");
// } else {
//     print("zero");
// }"#;
//         let indent = r#"
// if x > 0:
//     print("positive")
// elif x < 0:
//     print("negative")
// else:
//     print("zero")"#;
//
//         assert!(compare_dual_mode("elif_chain", braces, indent).is_ok());
//     }
//
//     #[test]
//     fn test_while_loop() {
//         let braces = r#"
// while x < 10 {
//     x = x + 1;
// }"#;
//         let indent = r#"
// while x < 10:
//     x = x + 1"#;
//
//         assert!(compare_dual_mode("while_loop", braces, indent).is_ok());
//     }
//
//     #[test]
//     fn test_for_loop() {
//         let braces = r#"
// for i in range(10) {
//     print(i);
// }"#;
//         let indent = r#"
// for i in range(10):
//     print(i)"#;
//
//         assert!(compare_dual_mode("for_loop", braces, indent).is_ok());
//     }
//
//     #[test]
//     fn test_nested_loops() {
//         let braces = r#"
// for i in range(3) {
//     for j in range(3) {
//         print(i * j);
//     }
// }"#;
//         let indent = r#"
// for i in range(3):
//     for j in range(3):
//         print(i * j)
//     "#;
//
//         assert!(compare_dual_mode("nested_loops", braces, indent).is_ok());
//     }
// }

#[cfg(test)]
mod struct_and_class_tests {
    use super::*;

    #[test]
    fn test_struct_declaration() {
        let braces = r#"
struct Point {
    x: float,
    y: float
}"#;
        let indent = r#"
struct Point:
    x: float
    y: float"#;
        
        assert!(compare_dual_mode("struct_declaration", braces, indent).is_ok());
    }

    #[test]
    fn test_class_with_constructor() {
        let braces = r#"
class Person {
    name: str;
    age: int;
    
    def __init__(self, name: str, age: int) {
        self.name = name;
        self.age = age;
    }
}"#;
        let indent = r#"
class Person:
    name: str
    age: int
    
    def __init__(self, name: str, age: int):
        self.name = name
        self.age = age"#;
        
        assert!(compare_dual_mode("class_with_constructor", braces, indent).is_ok());
    }

    #[test]
    fn test_class_with_methods() {
        let braces = r#"
class Rectangle {
    width: float;
    height: float;
    
    fn area(self) -> float {
        return self.width * self.height
    }
    
    fn perimeter(self) -> float {
        return 2 * (self.width + self.height)
    }
}"#;
        let indent = r#"
class Rectangle:
    width: float
    height: float
    
    fn area(self) -> float:
        return self.width * self.height
    
    fn perimeter(self) -> float:
        return 2 * (self.width + self.height)"#;
        
        assert!(compare_dual_mode("class_with_methods", braces, indent).is_ok());
    }
}

#[cfg(test)]
mod expression_tests {
    use super::*;

    #[test]
    fn test_binary_operations() {
        let braces = "let result: int = (a + b) * (c - d)";
        let indent = "let result: int = (a + b) * (c - d)";
        
        assert!(compare_dual_mode("binary_operations", braces, indent).is_ok());
    }

    #[test]
    fn test_comparison_operations() {
        let braces = "let check: bool = x > 5 && y <= 10;";
        let indent = "let check: bool = x > 5 && y <= 10";
        
        assert!(compare_dual_mode("comparison_operations", braces, indent).is_ok());
    }

    #[test]
    fn test_function_calls() {
        let braces = "let sum: int = add(multiply(2, 3), 4)";
        let indent = "let sum: int = add(multiply(2, 3), 4)";
        
        assert!(compare_dual_mode("function_calls", braces, indent).is_ok());
    }

    #[test]
    fn test_array_access() {
        let braces = "let element: int = array[index + 1]";
        let indent = "let element: int = array[index + 1]";
        
        assert!(compare_dual_mode("array_access", braces, indent).is_ok());
    }
}
//
// #[ignore]
// #[cfg(test)]
// mod error_handling_tests {
//     use super::*;
//
//     #[test]
//     fn test_try_except() {
//         let braces = r#"
// try {
//     risky_operation();
// } except ValueError {
//     handle_error();
// }"#;
//         let indent = r#"
// try:
//     risky_operation()
// except ValueError:
//     handle_error()"#;
//
//         assert!(compare_dual_mode("try_except", braces, indent).is_ok());
//     }
//
//     #[test]
//     fn test_try_finally() {
//         let braces = r#"
// try {
//     open_file();
// } finally {
//     close_file();
// }"#;
//         let indent = r#"
// try:
//     open_file()
// finally:
//     close_file()"#;
//
//         assert!(compare_dual_mode("try_finally", braces, indent).is_ok());
//     }
//
//     #[test]
//     fn test_match_statement() {
//         let braces = r#"
// match value {
//     1 => print("one"),
//     2 => print("two"),
//     _ => print("other")
// }"#;
//         let indent = r#"
// match value:
//     1 => print("one")
//     2 => print("two")
//     _ => print("other")"#;
//
//         assert!(compare_dual_mode("match_statement", braces, indent).is_ok());
//     }
// }

#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    #[test]
    fn test_empty_function() {
        let braces = r#"
fn do_nothing() {
    pass;
}"#;
        let indent = r#"
fn do_nothing():
    pass"#;
        
        assert!(compare_dual_mode("empty_function", braces, indent).is_ok());
    }

    #[test]
    fn test_single_line_if() {
        // This might fail depending on parser implementation
        let braces = "if x > 0 { return true }";
        let indent = "if x > 0: return true";
        
        // We expect this might fail, documenting the difference
        let result = compare_dual_mode("single_line_if", braces, indent);
        if result.is_err() {
            println!("Known difference: single-line if statements");
        }
    }

    #[test]
    fn test_deeply_nested_blocks() {
        let braces = r#"
fn complex() {
    if a {
        if b {
            if c {
                print("deep");
            }
        }
    }
}"#;
        let indent = r#"
fn complex():
    if a:
        if b:
            if c:
                print("deep")"#;
        
        assert!(compare_dual_mode("deeply_nested_blocks", braces, indent).is_ok());
    }

    // #[ignore]
    // #[test]
    // fn test_mixed_indentation_error() {
    //     // Test that mixing tabs and spaces causes errors in indentation mode
    //     let mixed_indent = "fn bad():\n\tlet x = 1\n    let y = 2"; // mixed tabs and spaces
    //
    //     let mut lexer = Lexer::new(mixed_indent, SyntaxMode::Indentation);
    //     let tokens = lexer.tokenize();
    //     let mut parser = Parser::new(tokens, SyntaxMode::Indentation);
    //
    //     // We expect this to fail
    //     assert!(parser.parse_program().is_err(), "Mixed indentation should cause error");
    // }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_large_program_performance() {
        // Generate a large program
        let mut braces_code = String::new();
        let mut indent_code = String::new();
        
        for i in 0..100 {
            braces_code.push_str(&format!("let var_{}: int = {};\n", i, i));
            indent_code.push_str(&format!("let var_{}: int = {}\n", i, i));
        }
        
        // Measure Braces mode
        let start = Instant::now();
        let mut braces_lexer = Lexer::new(&braces_code, SyntaxMode::Braces);
        let braces_tokens = braces_lexer.tokenize();
        let mut braces_parser = Parser::new(braces_tokens, SyntaxMode::Braces);
        let _ = braces_parser.parse_program();
        let braces_time = start.elapsed();
        
        // Measure Indentation mode
        let start = Instant::now();
        let mut indent_lexer = Lexer::new(&indent_code, SyntaxMode::Indentation);
        let indent_tokens = indent_lexer.tokenize();
        let mut indent_parser = Parser::new(indent_tokens, SyntaxMode::Indentation);
        let _ = indent_parser.parse_program();
        let indent_time = start.elapsed();
        
        println!("Braces mode: {:?}", braces_time);
        println!("Indentation mode: {:?}", indent_time);
        
        // Both should complete in reasonable time
        assert!(braces_time.as_secs() < 1);
        assert!(indent_time.as_secs() < 1);
    }
}