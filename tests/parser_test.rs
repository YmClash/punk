#[cfg(test)]
mod tests {
    use pyrust::parser::parser::Parser;
    use pyrust::{Lexer, SyntaxMode};
    use pyrust::parser::ast::ASTNode;
    use super::*;

    // Helper functions
    fn create_parser(source: &str, mode: SyntaxMode) -> Parser {
        let mut lexer = Lexer::new(source, mode);
        let tokens = lexer.tokenize();
        Parser::new(tokens, mode)
    }

    fn assert_ast_eq(actual: ASTNode, expected: ASTNode) {
        assert_eq!(format!("{:#?}", actual), format!("{:#?}", expected));
    }

    mod expression_tests {
        use super::*;

        #[test]
        fn test_literal_expressions_braces() {
            let test_cases = vec![
                ("42", "INTEGER"),
                ("3.14", "FLOAT"),
                ("\"hello\"", "STRING"),
                ("'c'", "CHAR"),
                ("true", "BOOLEAN"),
            ];

            for (input, expected_type) in test_cases {
                let mut parser = create_parser(input, SyntaxMode::Braces);
                let result = parser.parse_expression(0).unwrap();
                // Add assertions
            }
        }

        fn test_literal_expressions_indent() {
            let test_cases = vec![
                ("42", "INTEGER"),
                ("3.14", "FLOAT"),
                ("\"hello\"", "STRING"),
                ("'c'", "CHAR"),
                ("true", "BOOLEAN"),
            ];

            for (input, expected_type) in test_cases {
                let mut parser = create_parser(input, SyntaxMode::Indentation);
                let result = parser.parse_expression(0).unwrap();
                // Add assertions
            }
        }



        #[test]
        fn test_arithmetic_expressions() {
            let test_cases = vec![
                ("1 + 2", 3),
                ("3 * 4", 12),
                ("10 - 5", 5),
                ("20 / 5", 4),
                ("(2 + 3) * 4", 20),
            ];

            for (input, expected) in test_cases {
                let mut parser = create_parser(input, SyntaxMode::Braces);
                let result = parser.parse_expression(0).unwrap();
                // Add assertions
            }
        }

        #[test]
        fn test_complex_expressions() {
            let test_cases = vec![
                "foo.bar()",
                "array[index]",
                "obj.method().field[i]",
                "(a + b) * (c - d)",
            ];

            for input in test_cases {
                let mut parser = create_parser(input, SyntaxMode::Braces);
                assert!(parser.parse_expression(0).is_ok());
            }
        }

    }

    mod try_except_statement_tests{
        use super::*;

        #[test]
        fn test_try_except_braces() {
            let input = r#"
        try {
            risky_function();
        } except Error as e {
            handle_error(e);
        }
        "#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_try_statement();
            // assert!(result.is_ok());
        }

        #[test]
        fn test_try_except_indent() {
            let input = r#"
        try:
            risky_function()
        except Error as e:
            handle_error(e)
        "#;
            let mut parser = create_parser(input, SyntaxMode::Indentation);
            let result = parser.parse_try_statement();
            // assert!(result.is_ok());
        }

        #[test]
        fn test_try_except_finally() {
            let input = r#"
        try {
            risky_function();
        } except Error {
            handle_error();
        } finally {
            cleanup();
        }
        "#;
            let mut parser = create_parser(input, SyntaxMode::Braces);
            let result = parser.parse_try_statement();
            // assert!(result.is_ok());
        }
    }



    // mod declaration_tests {
    //     use pyrust::parser::ast::Visibility;
    //     use super::*;
    //
    //     #[test]
    //     fn test_variable_declarations() {
    //         let test_cases = vec![
    //             "let x = 42;",
    //             "let mut y: int = 10;",
    //             "let z: float = 3.14;",
    //             "let s = \"hello\";",
    //         ];
    //
    //         for input in test_cases {
    //             let mut parser = create_parser(input, SyntaxMode::Braces);
    //             assert!(parser.parse_variable_declaration().is_ok());
    //         }
    //     }
    //
    //     #[test]
    //     fn test_function_declarations() {
    //         let test_cases = vec![
    //             "fn foo() { }",
    //             "fn add(x: int, y: int) -> int { x + y }",
    //             "pub fn complex<T>(value: T) -> Option<T> { Some(value) }",
    //         ];
    //
    //         for input in test_cases {
    //             let mut parser = create_parser(input, SyntaxMode::Braces);
    //             assert!(parser.parse_function_declaration(Visibility::Public).is_ok());
    //         }
    //     }
    //
    //     #[test]
    //     fn test_type_declarations() {
    //         let test_cases = vec![
    //             r#"struct Point { x: int, y: int }"#,
    //             r#"enum Option { Some(T), None }"#,
    //             r#"trait Display { fn display(&self) -> str; }"#,
    //         ];
    //
    //         for input in test_cases {
    //             let mut parser = create_parser(input, SyntaxMode::Braces);
    //             // Test appropriate declaration type
    //         }
    //     }
    // }


    //
    // mod pattern_matching_tests {
    //     use super::*;
    //
    //     #[test]
    //     fn test_simple_patterns_braces() {
    //         let input = r#"
    //         match x {
    //             n if n > 0 => print("positive"),
    //             n if n < 0 => print("negative"),
    //             _ => print("zero")
    //         }
    //         "#;
    //         let mut parser = create_parser(input, SyntaxMode::Braces);
    //         let result = parser.parse_match_statement().unwrap();
    //         // Add assertions
    //     }
    //
    //     #[test]
    //     fn test_simple_patterns_indent() {
    //         let input = r#"
    //         match x:
    //             n if n > 0 => print("positive")
    //             n if n < 0 => print("negative")
    //             _ => print("zero")
    //         "#;
    //         let mut parser = create_parser(input, SyntaxMode::Indentation);
    //         let result = parser.parse_match_statement().unwrap();
    //         // Add assertions
    //     }
    //
    //     #[test]
    //     fn test_complex_patterns() {
    //         let input = r#"
    //         match value {
    //             Point{x, y} if x > 0 && y > 0 => "first quadrant",
    //             (x, y, z) => "tuple pattern",
    //             [head, ..tail] => "array pattern",
    //             _ => "wildcard"
    //         }
    //         "#;
    //         let mut parser = create_parser(input, SyntaxMode::Braces);
    //         let result = parser.parse_match_statement().unwrap();
    //         // Add assertions
    //     }
    // }
    //

    // mod control_flow_tests {
    //     use super::*;
    //
    //     #[test]
    //     fn test_if_statements() {
    //         let test_cases = vec![
    //             "if x > 0 { print(\"positive\"); }",
    //             r#"
    //             if x > 0 {
    //                 print("positive");
    //             } else if x < 0 {
    //                 print("negative");
    //             } else {
    //                 print("zero");
    //             }
    //             "#,
    //         ];
    //
    //         for input in test_cases {
    //             let mut parser = create_parser(input, SyntaxMode::Braces);
    //             assert!(parser.parse_if_statement().is_ok());
    //         }
    //     }
    //
    //     #[test]
    //     fn test_loops() {
    //         let test_cases = vec![
    //             "loop { break; }",
    //             "'outer: while x > 0 { x -= 1; }",
    //             "for i in 0..10 { print(i); }",
    //         ];
    //
    //         for input in test_cases {
    //             let mut parser = create_parser(input, SyntaxMode::Braces);
    //             // Test appropriate loop type
    //         }
    //     }
    // }
    //
    // mod error_tests {
    //     use super::*;
    //
    //     #[test]
    //     fn test_syntax_errors() {
    //         let test_cases = vec![
    //             "let;", // Missing identifier
    //             "let x;", // Missing initializer
    //             "fn {}", // Missing function name
    //             "match {}", // Missing match expression
    //         ];
    //
    //         for input in test_cases {
    //             let mut parser = create_parser(input, SyntaxMode::Braces);
    //             assert!(parser.parse_statement().is_err());
    //         }
    //     }
    //
    //     #[test]
    //     fn test_recovery() {
    //         let input = r#"
    //         let x = ; // Error
    //         let y = 42; // Should parse correctly
    //         "#;
    //         let mut parser = create_parser(input, SyntaxMode::Braces);
    //         // Test error recovery
    //     }
    // }

    // mod integration_tests {
    //     use pyrust::SyntaxMode;
    //     use super::*;
    //
    //     #[test]
    //     fn test_complete_file_braces() {
    //         let input = r#"
    //         fn main() {
    //             let x = 42;
    //             if x > 0 {
    //                 print("positive");
    //             }
    //
    //             match x {
    //                 n if n > 0 => print("positive"),
    //                 _ => print("other")
    //             }
    //         }
    //         "#;
    //         let mut parser = create_parser(input, SyntaxMode::Braces);
    //         assert!(parser.parse_program().is_ok());
    //     }
    //
    //     #[test]
    //     fn test_complete_file_indent() {
    //         let input = r#"
    //         fn main():
    //             let x = 42
    //             if x > 0:
    //                 print("positive")
    //
    //             match x:
    //                 n if n > 0 => print("positive")
    //                 _ => print("other")
    //         "#;
    //         let mut parser = create_parser(input, SyntaxMode::Indentation);
    //         assert!(parser.parse_program().is_ok());
    //     }
    // }
}