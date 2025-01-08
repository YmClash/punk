
use num_bigint::BigInt;

use pyrust::lexer::lex::Lexer;
use pyrust::lexer::tok::{Delimiters, Keywords, Operators, StringKind, TokenType};
use pyrust::lexer_error::{LexerError, LexerErrorType, Position};

#[cfg(test)]
mod tests {
    //use pyrust::lex::SyntaxMode;
    use super::*;
    use pyrust::lexer::lex::SyntaxMode;

    // Test pour les nombres
    #[test]
    fn test_lex_number() {
        let mut lexer = Lexer::new("123 3.14", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::INTEGER {
                value: BigInt::from(123)
            })
        );
        assert_eq!(lexer.get_token(), Some(TokenType::FLOAT { value: 3.14 }));
    }

    #[test]
    fn test_lex_number_2() {
        let mut lexer = Lexer::new("123 3.14", SyntaxMode::Indentation);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::INTEGER {
                value: BigInt::from(123)
            })
        );
        assert_eq!(lexer.get_token(), Some(TokenType::FLOAT { value: 3.14 }));
    }

    // Test pour les chaînes avec des séquences d'échappement
    #[test]
    fn test_lex_string_with_escapes() {
        let mut lexer = Lexer::new(r#""Hello, \"world\"""#, SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::STRING {
                value: r#"Hello, "world""#.to_string(),
                kind: StringKind::NORMAL
            })
        );
    }

    #[test]
    fn test_lex_string_with_escapes_2() {
        let mut lexer = Lexer::new(r#""Hello, \"world\"""#, SyntaxMode::Indentation);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::STRING {
                value: r#"Hello, "world""#.to_string(),
                kind: StringKind::NORMAL
            })
        );
    }

    // Test pour les identifiants et les mots-clés
    #[test]
    fn test_lex_identifier_and_keyword() {
        let mut lexer = Lexer::new("variable if", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "variable".to_string()
            })
        );
        assert_eq!(lexer.get_token(), Some(TokenType::KEYWORD(Keywords::IF)));
    }
    #[test]
    fn test_lex_identifier_and_keyword_2() {
        let mut lexer = Lexer::new("variable if", SyntaxMode::Indentation);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "variable".to_string()
            })
        );
        assert_eq!(lexer.get_token(), Some(TokenType::KEYWORD(Keywords::IF)));
    }
    // Test pour les commentaires multi-lignes
    #[test]
    fn test_lex_multi_line_comment() {
        let mut lexer = Lexer::new("/* comment */ code", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::COMMENT(" comment ".to_string()))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "code".to_string()
            })
        );
    }

    #[test]
    fn test_lex_multi_line_comment_2() {
        let mut lexer = Lexer::new("/* comment */ code", SyntaxMode::Indentation);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::COMMENT(" comment ".to_string()))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "code".to_string()
            })
        );
    }

    // Test pour les commentaires avec du code à l'intérieur
    #[test]
    fn test_lex_comment_with_code_inside() {
        let mut lexer = Lexer::new(
            "/* this is not code: let x = 42; */ actual_code",
            SyntaxMode::Braces,
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::COMMENT(
                " this is not code: let x = 42; ".to_string()
            ))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "actual_code".to_string()
            })
        );
    }

    #[test]
    fn test_lex_comment_with_code_inside_2() {
        let mut lexer = Lexer::new(
            "/* this is not code: let x = 42; */ actual_code",
            SyntaxMode::Indentation,
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::COMMENT(
                " this is not code: let x = 42; ".to_string()
            ))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "actual_code".to_string()
            })
        );
    }

    // Test pour les commentaires sur une seule ligne
    #[test]
    fn test_lex_comment() {
        let mut lexer = Lexer::new("# This is a comment\ncode", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::COMMENT(" This is a comment".to_string()))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "code".to_string()
            })
        );
    }

    #[test]
    fn test_lex_comment_2() {
        let mut lexer = Lexer::new("# This is a comment\ncode", SyntaxMode::Indentation);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::COMMENT(" This is a comment".to_string()))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "code".to_string()
            })
        );
    }
    // Test pour les chaînes multi-lignes
    #[test]
    fn test_lex_multiline_string() {
        let mut lexer = Lexer::new(
            r#""This is a \
                                       multi-line string""#,
            SyntaxMode::Braces,
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::STRING {
                value: "This is a multi-line string".to_string(),
                kind: StringKind::NORMAL
            })
        );
    }
    #[test]
    fn test_lex_multiline_string_2() {
        let mut lexer = Lexer::new(
            r#""This is a \
                                       multi-line string""#,
            SyntaxMode::Indentation,
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::STRING {
                value: "This is a multi-line string".to_string(),
                kind: StringKind::NORMAL
            })
        );
    }

    // Test pour les opérateurs complexes
    #[test]
    fn test_lex_complex_operator() {
        let mut lexer = Lexer::new("a += 1 && b == c || d != e", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "a".to_string()
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::PLUSEQUAL))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::INTEGER {
                value: BigInt::from(1)
            })
        );
        assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::AND)));
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "b".to_string()
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::EQEQUAL))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "c".to_string()
            })
        );
        assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::OR)));
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "d".to_string()
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::NOTEQUAL))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "e".to_string()
            })
        );
    }

    #[test]
    fn test_lex_complex_operator_2() {
        let mut lexer = Lexer::new("a += 1 && b == c || d != e", SyntaxMode::Indentation);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "a".to_string()
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::PLUSEQUAL))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::INTEGER {
                value: BigInt::from(1)
            })
        );
        assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::AND)));
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "b".to_string()
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::EQEQUAL))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "c".to_string()
            })
        );
        assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::OR)));
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "d".to_string()
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::NOTEQUAL))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "e".to_string()
            })
        );
    }

    // Test pour les délimiteurs
    #[test]
    fn test_lex_delimiters() {
        let mut lexer = Lexer::new("( { [ ] } )", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::LPAR))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::LCURBRACE))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::LSBRACKET))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::RSBRACKET))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::RCURBRACE))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::RPAR))
        );
    }

    #[test]
    fn test_lex_delimiters_2() {
        let mut lexer = Lexer::new("( { [ ] } )", SyntaxMode::Indentation);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::LPAR))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::LCURBRACE))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::LSBRACKET))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::RSBRACKET))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::RCURBRACE))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::RPAR))
        );
    }

    // Test pour les délimiteurs imbriqués
    #[test]
    fn test_lex_nested_delimiters() {
        let mut lexer = Lexer::new("{[(())]}", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::LCURBRACE))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::LSBRACKET))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::LPAR))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::LPAR))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::RPAR))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::RPAR))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::RSBRACKET))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::RCURBRACE))
        );
    }

    #[test]
    fn test_lex_nested_delimiters_2() {
        let mut lexer = Lexer::new("{[(())]}", SyntaxMode::Indentation);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::LCURBRACE))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::LSBRACKET))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::LPAR))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::LPAR))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::RPAR))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::RPAR))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::RSBRACKET))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::RCURBRACE))
        );
    }

    // Test pour les caractères inattendus
    #[test]
    fn test_lex_unexpected_character() {
        let mut lexer = Lexer::new("@ $", SyntaxMode::Braces);
        assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::AT)));
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::ERROR(LexerError::invalid_token(
                "$",
                Position { line: 1, column: 4 }
            )))
        );
    }

    #[test]
    fn test_lex_unexpected_character_2() {
        let mut lexer = Lexer::new("@ $", SyntaxMode::Indentation);
        assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::AT)));
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::ERROR(LexerError::invalid_token(
                "$",
                Position { line: 1, column: 4 }
            )))
        );
    }

    // Test pour un mélange d'opérateurs et de caractères inconnus
    #[test]
    fn test_mixed_operators_and_unknown() {
        let mut lexer = Lexer::new("@$", SyntaxMode::Braces);
        assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::AT)));
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::ERROR(LexerError::invalid_token(
                "$",
                Position { line: 1, column: 3 }
            )))
        );
    }

    // Test pour l'opérateur @ seul
    #[test]
    fn test_arobase() {
        let mut lexer = Lexer::new("@", SyntaxMode::Braces);
        assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::AT)));
    }

    // Test pour un mélange d'opérateurs et de délimiteurs

    #[test]
    fn test_lex_mixed_input() {
        let mut lexer = Lexer::new(r#"let sum = a + b * (c - d) / e;"#, SyntaxMode::Braces);
        assert_eq!(lexer.get_token(), Some(TokenType::KEYWORD(Keywords::LET)));
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "sum".to_string()
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::EQUAL))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "a".to_string()
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::PLUS))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "b".to_string()
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::STAR))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::LPAR))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "c".to_string()
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::MINUS))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "d".to_string()
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::RPAR))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::SLASH))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "e".to_string()
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::SEMICOLON))
        );
    }

    ///////////////////////////////////////fonction  pour test//////////////////////////////////////////
    fn assert_tokens(input: &str, expected_tokens: Vec<TokenType>, syntax_mode: SyntaxMode) {
        let mut lexer = Lexer::new(input, syntax_mode);
        let tokens: Vec<TokenType> = lexer.tokenize().into_iter().map(|t| t.token_type).collect();
        assert_eq!(tokens, expected_tokens);
    }
    //////////////////////////////////////////////////////////////////////////////

    // Test pour des tokens de base avec des commentaires et des chaînes multi-lignes
    #[test]
    fn test_basic_tokens_braces_mode() {
        let input = r#"
        fn main() {
            let x = 5;
            if (x > 3) {
                println!("Hello, world!");
            }
        }
        "#;

        let expected_tokens = vec![
            // TokenType::NEWLINE,
            TokenType::KEYWORD(Keywords::FN),
            TokenType::IDENTIFIER {
                name: "main".to_string(),
            },
            TokenType::DELIMITER(Delimiters::LPAR),
            TokenType::DELIMITER(Delimiters::RPAR),
            TokenType::DELIMITER(Delimiters::LCURBRACE),
            // TokenType::NEWLINE,
            TokenType::KEYWORD(Keywords::LET),
            TokenType::IDENTIFIER {
                name: "x".to_string(),
            },
            TokenType::OPERATOR(Operators::EQUAL),
            TokenType::INTEGER {
                value: BigInt::from(5),
            },
            TokenType::DELIMITER(Delimiters::SEMICOLON),
            // TokenType::NEWLINE,
            TokenType::KEYWORD(Keywords::IF),
            TokenType::DELIMITER(Delimiters::LPAR),
            TokenType::IDENTIFIER {
                name: "x".to_string(),
            },
            TokenType::OPERATOR(Operators::GREATER),
            TokenType::INTEGER {
                value: BigInt::from(3),
            },
            TokenType::DELIMITER(Delimiters::RPAR),
            TokenType::DELIMITER(Delimiters::LCURBRACE),
            // TokenType::NEWLINE,
            TokenType::IDENTIFIER {
                name: "println".to_string(),
            },
            TokenType::OPERATOR(Operators::EXCLAMATION),
            TokenType::DELIMITER(Delimiters::LPAR),
            TokenType::STRING {
                value: "Hello, world!".to_string(),
                kind: StringKind::NORMAL,
            },
            TokenType::DELIMITER(Delimiters::RPAR),
            TokenType::DELIMITER(Delimiters::SEMICOLON),
            // TokenType::NEWLINE,
            TokenType::DELIMITER(Delimiters::RCURBRACE),
            // TokenType::NEWLINE,
            TokenType::DELIMITER(Delimiters::RCURBRACE),
            // TokenType::NEWLINE,
            TokenType::EOF,
        ];

        assert_tokens(input, expected_tokens, SyntaxMode::Braces);
    }

    #[test]
    fn test_basic_tokens_indentation_mode() {
        let input = r#"
def main():
    x = 5
    if x > 3:
        print("Hello, world!")
        "#;

        let expected_tokens = vec![
            TokenType::NEWLINE,
            TokenType::KEYWORD(Keywords::DEF),
            TokenType::IDENTIFIER {
                name: "main".to_string(),
            },
            TokenType::DELIMITER(Delimiters::LPAR),
            TokenType::DELIMITER(Delimiters::RPAR),
            TokenType::DELIMITER(Delimiters::COLON),
            TokenType::NEWLINE,
            TokenType::INDENT,
            TokenType::IDENTIFIER {
                name: "x".to_string(),
            },
            TokenType::OPERATOR(Operators::EQUAL),
            TokenType::INTEGER {
                value: BigInt::from(5),
            },
            TokenType::NEWLINE,
            TokenType::KEYWORD(Keywords::IF),
            TokenType::IDENTIFIER {
                name: "x".to_string(),
            },
            TokenType::OPERATOR(Operators::GREATER),
            TokenType::INTEGER {
                value: BigInt::from(3),
            },
            TokenType::DELIMITER(Delimiters::COLON),
            TokenType::NEWLINE,
            TokenType::INDENT,
            TokenType::IDENTIFIER {
                name: "print".to_string(),
            },
            TokenType::DELIMITER(Delimiters::LPAR),
            TokenType::STRING {
                value: "Hello, world!".to_string(),
                kind: StringKind::NORMAL,
            },
            TokenType::DELIMITER(Delimiters::RPAR),
            TokenType::NEWLINE,
            TokenType::DEDENT,
            TokenType::DEDENT,
            TokenType::EOF,
        ];

        assert_tokens(input, expected_tokens, SyntaxMode::Indentation);
    }

    #[test]
    fn test_mixed_tokens() {
        let input = r#"x = 3.14 + 2 * (5 - 1) # This is a comment"#;

        let expected_tokens = vec![
            TokenType::IDENTIFIER {
                name: "x".to_string(),
            },
            TokenType::OPERATOR(Operators::EQUAL),
            TokenType::FLOAT { value: 3.14 },
            TokenType::OPERATOR(Operators::PLUS),
            TokenType::INTEGER {
                value: BigInt::from(2),
            },
            TokenType::OPERATOR(Operators::STAR),
            TokenType::DELIMITER(Delimiters::LPAR),
            TokenType::INTEGER {
                value: BigInt::from(5),
            },
            TokenType::OPERATOR(Operators::MINUS),
            TokenType::INTEGER {
                value: BigInt::from(1),
            },
            TokenType::DELIMITER(Delimiters::RPAR),
            TokenType::COMMENT(" This is a comment".to_string()),
            TokenType::EOF,
        ];

        assert_tokens(input, expected_tokens, SyntaxMode::Braces);
        // assert_tokens(input, expected_tokens, SyntaxMode::Indentation);
    }

    #[test]
    fn test_mixed_tokens_2() {
        let input = r#"x = 3.14 + 2 * (5 - 1) # This is a comment"#;

        let expected_tokens = vec![
            TokenType::IDENTIFIER {
                name: "x".to_string(),
            },
            TokenType::OPERATOR(Operators::EQUAL),
            TokenType::FLOAT { value: 3.14 },
            TokenType::OPERATOR(Operators::PLUS),
            TokenType::INTEGER {
                value: BigInt::from(2),
            },
            TokenType::OPERATOR(Operators::STAR),
            TokenType::DELIMITER(Delimiters::LPAR),
            TokenType::INTEGER {
                value: BigInt::from(5),
            },
            TokenType::OPERATOR(Operators::MINUS),
            TokenType::INTEGER {
                value: BigInt::from(1),
            },
            TokenType::DELIMITER(Delimiters::RPAR),
            TokenType::COMMENT(" This is a comment".to_string()),
            TokenType::EOF,
        ];

        //assert_tokens(input, expected_tokens, SyntaxMode::Braces);
        assert_tokens(input, expected_tokens, SyntaxMode::Indentation);
    }

    // Test pour les chaînes vides
    #[test]
    fn test_empty_string() {
        let mut lexer = Lexer::new(r#""""#, SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::STRING {
                value: "".to_string(),
                kind: StringKind::NORMAL
            })
        );
    }

    // Test pour les nombres négatifs
    #[test]
    fn test_negative_numbers() {
        let mut lexer = Lexer::new("-42 -3.14", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::MINUS))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::INTEGER {
                value: BigInt::from(42)
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::MINUS))
        );
        assert_eq!(lexer.get_token(), Some(TokenType::FLOAT { value: 3.14 }));
    }

    // Test pour les identifiants avec des underscores
    #[test]
    fn test_identifiers_with_underscores() {
        let mut lexer = Lexer::new("my_variable _private", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "my_variable".to_string()
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "_private".to_string()
            })
        );
    }

    // Test pour les opérateurs composés
    #[test]
    fn test_compound_operators() {
        let mut lexer = Lexer::new("++ -- ** //", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::PLUSEQUAL))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::MINEQUAL))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::DOUBLESTAR))
        );
        assert_eq!(lexer.get_token(), Some(TokenType::COMMENT("".to_string())));
    }

    //Test pour la gestion des espaces et des sauts de ligne
    // #[test]
    // fn test_whitespace_handling() {
    //     let mut lexer = Lexer::new("let   x\n=\t42", SyntaxMode::Braces);
    //     assert_eq!(lexer.get_token(), Some(TokenType::KEYWORD(Keywords::LET)));
    //     assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "x".to_string() }));
    //     assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::EQUAL)));
    //     assert_eq!(lexer.get_token(), Some(TokenType::INTEGER { value: BigInt::from(42) }));
    // }

    // Test pour les caractères d'échappement dans les chaînes
    #[test]
    fn test_string_escape_sequences() {
        let mut lexer = Lexer::new(r#""Hello\nWorld\t\"Escaped\"""#, SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::STRING {
                value: "Hello\nWorld\t\"Escaped\"".to_string(),
                kind: StringKind::NORMAL
            })
        );
    }

    // Test pour les chaînes multi-lignes complexes
    // #[test]
    // fn test_multiline_strings() {
    //     let input = r#"
    //     "This is a simple string"
    //     "This is a string \
    //      that spans multiple \
    //      lines"
    //     "This string has
    //      actual newlines"
    //     "This string has \n escaped newlines"
    //     "Mixed \
    //      newlines \n and \
    //      continuations"
    // "#;
    //
    //     let mut lexer = Lexer::new(input, SyntaxMode::Braces);
    //
    //     assert_eq!(
    //         lexer.get_token(),
    //         Some(TokenType::STRING {
    //             value: "This is a simple string".to_string(),
    //             kind: StringKind::NORMAL
    //         })
    //     );
    //
    //     assert_eq!(
    //         lexer.get_token(),
    //         Some(TokenType::STRING {
    //             value: "This is a string that spans multiple lines".to_string(),
    //             kind: StringKind::NORMAL
    //         })
    //     );
    //
    //     assert_eq!(
    //         lexer.get_token(),
    //         Some(TokenType::STRING {
    //             value: "This string has\n         actual newlines".to_string(),
    //             kind: StringKind::NORMAL
    //         })
    //     );
    //
    //     assert_eq!(
    //         lexer.get_token(),
    //         Some(TokenType::STRING {
    //             value: "This string has \n escaped newlines".to_string(),
    //             kind: StringKind::NORMAL
    //         })
    //     );
    //
    //     assert_eq!(
    //         lexer.get_token(),
    //         Some(TokenType::STRING {
    //             value: "Mixed newlines \n and continuations".to_string(),
    //             kind: StringKind::NORMAL
    //         })
    //     );
    //
    //     assert_eq!(lexer.get_token(), Some(TokenType::EOF));
    // }

    // Test pour les commentaires et les commentaires de documentation
    #[test]
    fn test_comments_and_doc_comments() {
        let mut lexer = Lexer::new(
            "// This is a regular comment\n/// This is a doc comment\n/* Multi-line comment */",
            SyntaxMode::Braces,
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::COMMENT(" This is a regular comment".to_string()))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DOCSTRING(" This is a doc comment".to_string()))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::COMMENT(" Multi-line comment ".to_string()))
        );
    }

    // #[test]
    // fn test_invalid_identifier() {
    //     let mut lexer = Lexer::new("var\$", SyntaxMode::Braces);
    //     assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "var".to_string() }));
    //
    //     if let Some(TokenType::ERROR(error)) = lexer.get_token() {
    //         assert_eq!(error.error, LexerErrorType::InvalidToken("\$".to_string()));
    //     } else {
    //         panic!("Expected an ERROR token for '\$'");
    //     }
    //
    //     assert_eq!(lexer.get_token(), Some(TokenType::EOF));
    // }

    #[test]
    fn test_invalid_escape_sequence() {
        let mut lexer = Lexer::new(r#""This is a bad escape: \q""#, SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::STRING {
                value: "This is a bad escape: q".to_string(),
                kind: StringKind::NORMAL
            })
        );
    }

    #[test]
    fn test_complex_expression() {
        let mut lexer = Lexer::new("a + (b - (c * d) / e) && f || g", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "a".to_string()
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::PLUS))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::LPAR))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "b".to_string()
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::MINUS))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::LPAR))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "c".to_string()
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::STAR))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "d".to_string()
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::RPAR))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::SLASH))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "e".to_string()
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::DELIMITER(Delimiters::RPAR))
        );
        assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::AND)));
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "f".to_string()
            })
        );
        assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::OR)));
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "g".to_string()
            })
        );
    }

    #[test]
    fn test_reserved_keyword_as_identifier() {
        let mut lexer = Lexer::new("fn fn", SyntaxMode::Braces);
        assert_eq!(lexer.get_token(), Some(TokenType::KEYWORD(Keywords::FN)));
        assert_eq!(lexer.get_token(), Some(TokenType::KEYWORD(Keywords::FN)));
    }

    #[test]
    fn test_large_numbers() {
        let mut lexer = Lexer::new("123456789012345678901234567890", SyntaxMode::Braces);
        if let Some(TokenType::ERROR(error)) = lexer.get_token() {
            assert_eq!(
                error.error,
                LexerErrorType::InvalidInteger("123456789012345678901234567890".to_string())
            );
        } else {
            panic!("Expected an ERROR token for very large integer");
        }
    }

    #[test]
    fn test_string_with_keywords_and_operators() {
        let mut lexer = Lexer::new(r#""if else let + -""#, SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::STRING {
                value: "if else let + -".to_string(),
                kind: StringKind::NORMAL
            })
        );
    }

    #[test]
    fn test_lex_hex_number() {
        let mut lexer = Lexer::new("0x1A3F 0Xb4", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::HEXADECIMAL { value: 0x1A3F })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::HEXADECIMAL { value: 0xB4 })
        );
    }

    #[test]
    fn test_invalid_hex_number() {
        let mut lexer = Lexer::new("0xGHI", SyntaxMode::Braces);
        if let Some(TokenType::ERROR(error)) = lexer.get_token() {
            assert_eq!(
                error.error,
                LexerErrorType::InvalidHexadecimal("0x".to_string())
            );
        } else {
            panic!("Expected an ERROR token for invalid hexadecimal");
        }
    }

    #[test]
    fn test_valid_hexadecimal() {
        let mut lexer = Lexer::new("0x1A3F", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::HEXADECIMAL { value: 0x1A3F })
        );
    }

    #[test]
    fn test_valid_hexadecimal_uppercase() {
        let mut lexer = Lexer::new("0X1A3F", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::HEXADECIMAL { value: 0x1A3F })
        );
    }

    #[test]
    fn test_valid_hexadecimal_mixed_case() {
        let mut lexer = Lexer::new("0xaBcD", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::HEXADECIMAL { value: 0xABCD })
        );
    }

    #[test]
    fn test_valid_hexadecimal_mixed_case_2() {
        let mut lexer = Lexer::new("0x1A3F 0Xb4", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::HEXADECIMAL { value: 0x1A3F })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::HEXADECIMAL { value: 0xB4 })
        );
    }

    #[test]
    fn test_valid_hexadecimal_zero() {
        let mut lexer = Lexer::new("0x0", SyntaxMode::Braces);
        assert_eq!(lexer.get_token(), Some(TokenType::HEXADECIMAL { value: 0 }));
    }

    #[test]
    fn test_valid_hexadecimal_max() {
        let mut lexer = Lexer::new("0xFFFFFFFFFFFFFFFF", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::HEXADECIMAL {
                value: 0xFFFFFFFFFFFFFFFF
            })
        );
    }

    #[test]
    fn test_hexadecimal_followed_by_identifier() {
        let mut lexer = Lexer::new("0xABC def", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::HEXADECIMAL { value: 0xABC })
        );
        assert_eq!(lexer.get_token(), Some(TokenType::KEYWORD(Keywords::DEF)));
    }

    #[test]
    fn test_multiple_hexadecimals() {
        let mut lexer = Lexer::new("0x123 0x456 0x789", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::HEXADECIMAL { value: 0x123 })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::HEXADECIMAL { value: 0x456 })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::HEXADECIMAL { value: 0x789 })
        );
    }

    /// Test pour vérifier la tokenisation des opérateurs complexes.
    #[test]
    fn test_lex_complex_operator_3() {
        let mut lexer = Lexer::new("a += 1 && b == c || d != e", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "a".to_string()
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::PLUSEQUAL))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::INTEGER {
                value: BigInt::from(1)
            })
        );
        assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::AND)));
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "b".to_string()
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::EQEQUAL))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "c".to_string()
            })
        );
        assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::OR)));
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "d".to_string()
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::NOTEQUAL))
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "e".to_string()
            })
        );
    }

    ////////////////////////////////////////////////ERROR HANDLING TEST ///////////////////////////////////////////////////////////////////////////

    #[test]
    fn test_invalid_number() {
        let mut lexer = Lexer::new("123a", SyntaxMode::Braces);

        // Le premier token devrait être un entier valide
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::INTEGER {
                value: BigInt::from(123)
            })
        );

        // Le deuxième token devrait être un identifiant
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "a".to_string()
            })
        );

        // Il ne devrait plus y avoir de tokens
        assert_eq!(lexer.get_token(), Some(TokenType::EOF));
    }

    //Test pour un float invalide a  revoir
    // #[test]
    // fn test_invalid_float() {
    //     let mut lexer = Lexer::new("3.14.15", SyntaxMode::Braces);
    //     assert_eq!(
    //         lexer.get_token(),
    //         Some(TokenType::ERROR(LexerError::invalid_float(
    //             "3.14.15",
    //             Position { line: 1, column: 8 }
    //         )))
    //     );
    // }

    // Test pour une chaîne non terminée
    #[test]
    fn test_unterminated_string() {
        let mut lexer = Lexer::new("\"Hello, world", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::ERROR(LexerError::unterminated_string(
                Position {
                    line: 1,
                    column: 14
                }
            )))
        );
    }

    // Test pour un commentaire non terminé
    #[test]
    fn test_unterminated_comment() {
        let mut lexer = Lexer::new("/* This is a comment", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::ERROR(LexerError::unterminated_comment(
                Position {
                    line: 1,
                    column: 21
                }
            )))
        );
    }

    //Test pour un hexadécimal invalide
    #[test]
    fn test_invalid_hexadecimal() {
        let mut lexer = Lexer::new("0xGHI", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::ERROR(LexerError::new(
                LexerErrorType::InvalidHexadecimal("0x".to_string()),
                "Invalid hexadecimal: 0x".to_string(),
                Position { line: 1, column: 3 }
            )))
        );
    }

    //Test pour un hexadécimal valide suivi d'un identifiant
    #[test]
    fn test_hexadecimal_followed_by_identifier_2() {
        let mut lexer = Lexer::new("0x1FAb identifier", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::HEXADECIMAL { value: 0x1FAb })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "identifier".to_string()
            })
        );
    }

    // Test pour une chaîne avec un saut de ligne non échappé
    #[test]
    fn test_multiline_string() {
        let mut lexer = Lexer::new("\"Hello\nWorld\"", SyntaxMode::Braces);
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::STRING {
                value: "Hello\nWorld".to_string(),
                kind: StringKind::NORMAL
            })
        );
    }

    // Test pour un commentaire multi-ligne non terminé
    #[test]
    fn test_unterminated_multiline_comment() {
        let mut lexer = Lexer::new(
            "/* This is a multi-line comment that doesn't end",
            SyntaxMode::Braces,
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::ERROR(LexerError::unterminated_comment(
                Position {
                    line: 1,
                    column: 49
                }
            )))
        );
    }

    #[test]
    fn test_unknown_operator() {
        let mut lexer = Lexer::new("a $$ b", SyntaxMode::Braces);

        // Le premier token devrait être un identifiant
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "a".to_string()
            })
        );

        // Les deux prochains tokens devraient être des erreurs pour des opérateurs inconnus
        for _ in 0..2 {
            if let Some(TokenType::ERROR(error)) = lexer.get_token() {
                assert_eq!(error.error, LexerErrorType::InvalidToken("$".to_string()));
                // Nous ne vérifions pas la position exacte ici
            } else {
                panic!("Expected an ERROR token for unknown operator");
            }
        }

        // Le quatrième token devrait être un autre identifiant
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "b".to_string()
            })
        );

        // Assurons-nous qu'il n'y a plus de tokens
        assert_eq!(lexer.get_token(), Some(TokenType::EOF));
    }

    //     #[test]
    //     fn test_indentation_with_empty_lines() {
    //         let source = r#"
    // def func():
    //     if True:
    //
    //         print("Hello")
    //
    //     print("World")
    //         "#;
    //
    //         let mut lexer = Lexer::new(source, SyntaxMode::Indentation);
    //         let tokens = lexer.tokenize();
    //
    //         // Vérifiez que les lignes vides n'affectent pas l'indentation
    //         let indent_dedent_sequence: Vec<TokenType> = tokens.iter()
    //             .filter(|t| matches!(t.token_type, TokenType::INDENT | TokenType::DEDENT))
    //             .map(|t| t.token_type.clone())
    //             .collect();
    //
    //         assert_eq!(
    //             indent_dedent_sequence,
    //             vec![TokenType::INDENT, TokenType::INDENT, TokenType::DEDENT, TokenType::DEDENT]
    //         );
    //     }

    ////// TESTS AVANCÉS //////

    // mod advanced_tests {
    //     use super::*;
    //
    // Tests de performance
    // #[test]
    // fn test_lexer_performance_large_input() {
    //     let single_line = "let x = 42;\n";
    //     let num_lines = 100000;
    //
    //     let start = Instant::now();
    //     let mut token_count = 0;
    //
    //     for _ in 0..num_lines {
    //         let mut lexer = Lexer::new(single_line, SyntaxMode::Braces);
    //         while let Some(token) = lexer.get_token() {
    //             token_count += 1;
    //             if matches!(token, TokenType::EOF) {
    //                 break;
    //             }
    //         }
    //     }
    //
    //     let duration = start.elapsed();
    //
    //     println!("Tokenizing {} lines took: {:?}", num_lines, duration);
    //     println!("Total tokens processed: {}", token_count);
    //     println!("Tokens per second: {}", token_count as f64 / duration.as_secs_f64());
    //
    //     // Ajuster le seuil en fonction de la performance actuelle
    //     assert!(duration.as_secs() < 15, "Tokenizing took too long");
    //     assert_eq!(token_count, num_lines * 6); // 6 tokens per line (let, x, =, 42, ;, EOF)
    // }

    #[test]
    fn test_lexer_robustness_very_long_identifier() {
        let very_long_identifier = "a".repeat(10000);
        let mut lexer = Lexer::new(&very_long_identifier, SyntaxMode::Braces);
        if let Some(TokenType::IDENTIFIER { name }) = lexer.get_token() {
            assert_eq!(name, very_long_identifier);
        } else {
            panic!("Expected an IDENTIFIER token for very long identifier");
        }
        assert_eq!(lexer.get_token(), Some(TokenType::EOF));
    }

    #[test]
    fn test_lexer_robustness_very_long_number() {
        let very_long_number = "9".repeat(1000);
        let mut lexer = Lexer::new(&very_long_number, SyntaxMode::Braces);
        if let Some(TokenType::ERROR(error)) = lexer.get_token() {
            let expected_error = LexerError::invalid_integer(
                &very_long_number,
                Position {
                    line: 1,
                    column: 1001,
                },
            );
            assert_eq!(error.error, expected_error.error);
            assert_eq!(error.message, expected_error.message);
            assert_eq!(error.position, expected_error.position);
        } else {
            panic!("Expected an ERROR token for very long number");
        }
    }

    #[test]
    fn test_lexer_robustness_deeply_nested_expressions() {
        let deeply_nested = "(".repeat(1000) + &"42".to_string() + &")".repeat(1000);
        let mut lexer = Lexer::new(&deeply_nested, SyntaxMode::Indentation);

        for _ in 0..1000 {
            assert_eq!(
                lexer.get_token(),
                Some(TokenType::DELIMITER(Delimiters::LPAR))
            );
        }
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::INTEGER {
                value: BigInt::from(42)
            })
        );
        for _ in 0..1000 {
            assert_eq!(
                lexer.get_token(),
                Some(TokenType::DELIMITER(Delimiters::RPAR))
            );
        }
        assert_eq!(lexer.get_token(), Some(TokenType::EOF));
    }

    #[test]
    fn test_lexer_robustness_malformed_input() {
        let malformed_input = "let x = @#$%^&*;";
        let mut lexer = Lexer::new(malformed_input, SyntaxMode::Braces);

        assert_eq!(lexer.get_token(), Some(TokenType::KEYWORD(Keywords::LET)));
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::IDENTIFIER {
                name: "x".to_string()
            })
        );
        assert_eq!(
            lexer.get_token(),
            Some(TokenType::OPERATOR(Operators::EQUAL))
        );
        assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::AT)));

        // Vérifie si le reste est traité comme un commentaire
        if let Some(TokenType::COMMENT(comment)) = lexer.get_token() {
            assert_eq!(comment, "$%^&*;");
        } else {
            panic!("Expected a COMMENT token for '#$%^&*;'");
        }

        // Vérifie qu'il n'y a plus de tokens
        assert_eq!(lexer.get_token(), Some(TokenType::EOF));
    }
}












// #[test]
// fn test_indentation_mode() {
//     let code = r#"
// def function():
//     if condition:
//         print("Indented")
//     else:
//         print("Also indented")
//         if nested:
//             print("Nested indentation")
//     print("Back to first level")
// print("No indentation")
// "#;
//     let mut lexer = Lexer::new(code, SyntaxMode::Indentation);
//     let tokens = lexer.tokenize();
//
//     // Vérifions d'abord la présence des tokens INDENT et DEDENT
//     let indent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::INDENT)).count();
//     let dedent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::DEDENT)).count();
//
//     assert_eq!(indent_count, 3, "Nombre incorrect de tokens INDENT");
//     assert_eq!(dedent_count, 3, "Nombre incorrect de tokens DEDENT");
//
//     // Vérifions ensuite quelques tokens clés pour s'assurer que la structure générale est correcte
//     let key_tokens = tokens.iter().filter(|t| matches!(t.token_type,
//             TokenType::IDENTIFIER { .. } |
//             TokenType::KEYWORD(..) |
//             TokenType::INDENT |
//             TokenType::DEDENT
//         )).collect::<Vec<_>>();
//
//     let expected_key_sequence = vec![
//         TokenType::IDENTIFIER { name: "def".to_string() },
//         TokenType::IDENTIFIER { name: "function".to_string() },
//         TokenType::INDENT,
//         TokenType::KEYWORD(Keywords::IF),
//         TokenType::INDENT,
//         TokenType::IDENTIFIER { name: "print".to_string() },
//         TokenType::DEDENT,
//         TokenType::KEYWORD(Keywords::ELSE),
//         TokenType::INDENT,
//         TokenType::IDENTIFIER { name: "print".to_string() },
//         TokenType::KEYWORD(Keywords::IF),
//         TokenType::INDENT,
//         TokenType::IDENTIFIER { name: "print".to_string() },
//         TokenType::DEDENT,
//         TokenType::DEDENT,
//         TokenType::IDENTIFIER { name: "print".to_string() },
//         TokenType::DEDENT,
//         TokenType::IDENTIFIER { name: "print".to_string() },
//     ];
//
//     for (i, (token, expected)) in key_tokens.iter().zip(expected_key_sequence.iter()).enumerate() {
//         assert_eq!(&token.token_type, expected, "Mismatch at key token {}", i);
//     }
// }
//
// #[test]
// fn test_invalid_indentation() {
//     let code = r#"
// def function():
//     if condition:
//         print("Correct indentation")
//        print("Invalid indentation")
// "#;
//     let mut lexer = Lexer::new(code, SyntaxMode::Indentation);
//     let tokens = lexer.tokenize();
//
//     // Vérifions que le lexer a généré le bon nombre de tokens
//     assert!(tokens.len() > 0, "Le lexer n'a généré aucun token");
//
//     // Vérifions que le dernier token n'est pas une erreur (car le lexer pourrait ne pas détecter cette erreur)
//     assert!(!matches!(tokens.last().unwrap().token_type, TokenType::ERROR(_)),
//             "Le lexer a généré une erreur inattendue");
//
//     // Vérifions que l'indentation est correctement gérée
//     let indent_count = tokens.iter().filter(|t| matches!(t.token_type, TokenType::INDENT)).count();
//     assert_eq!(indent_count, 2, "Nombre incorrect de tokens INDENT");
// }
//

// use pyrust::lexer::lex::Lexer;
// use pyrust::lexer::tok::{TokenType, Keywords, Operators, Delimiters, StringKind};
// use pyrust::error::{LexerError,Position};
// use num_bigint::BigInt;
// use std::time::Instant;
//
//
// #[cfg(test)]
// mod tests {
//     use pyrust::error::{ LexerErrorType};
//     use super::*;
//
//     // Test pour les nombres
//     #[test]
//     fn test_lex_number() {
//         let mut lexer = Lexer::new("123 3.14");
//         assert_eq!(lexer.get_token(), Some(TokenType::INTEGER { value: BigInt::from(123) }));
//         assert_eq!(lexer.get_token(), Some(TokenType::FLOAT { value: 3.14 }));
//     }
//
//     // Test pour les chaînes avec des séquences d'échappement
//     #[test]
//     fn test_lex_string_with_escapes() {
//         let mut lexer = Lexer::new(r#""Hello, \"world\"""#);
//         assert_eq!(
//             lexer.get_token(),
//             Some(TokenType::STRING {
//                 value: r#"Hello, "world""#.to_string(),
//                 kind: StringKind::NORMAL
//             })
//         );
//     }
//
//     // Test pour les identifiants et les mots-clés
//     #[test]
//     fn test_lex_identifier_and_keyword() {
//         let mut lexer = Lexer::new("variable if");
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "variable".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::KEYWORD(Keywords::IF)));
//     }
//
//     // Test pour les commentaires multi-lignes
//     #[test]
//     fn test_lex_multi_line_comment() {
//         let mut lexer = Lexer::new("/* comment */ code");
//         assert_eq!(lexer.get_token(), Some(TokenType::COMMENT(" comment ".to_string())));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "code".to_string() }));
//     }
//
//     // Test pour les commentaires avec du code à l'intérieur
//     #[test]
//     fn test_lex_comment_with_code_inside() {
//         let mut lexer = Lexer::new("/* this is not code: let x = 42; */ actual_code");
//         assert_eq!(lexer.get_token(), Some(TokenType::COMMENT(" this is not code: let x = 42; ".to_string())));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "actual_code".to_string() }));
//     }
//
//     // Test pour les commentaires sur une seule ligne
//     #[test]
//     fn test_lex_comment() {
//         let mut lexer = Lexer::new("# This is a comment\ncode");
//         assert_eq!(lexer.get_token(), Some(TokenType::COMMENT(" This is a comment".to_string())));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "code".to_string() }));
//     }
//
//     // Test pour les chaînes multi-lignes
//     #[test]
//     fn test_lex_multiline_string() {
//         let mut lexer = Lexer::new(r#""This is a \
//                                        multi-line string""#);
//         assert_eq!(
//             lexer.get_token(),
//             Some(TokenType::STRING {
//                 value: "This is a multi-line string".to_string(),
//                 kind: StringKind::NORMAL
//             })
//         );
//     }
//
//     // Test pour les opérateurs complexes
//     #[test]
//     fn test_lex_complex_operator() {
//         let mut lexer = Lexer::new("a += 1 && b == c || d != e");
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "a".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::PLUSEQUAL)));
//         assert_eq!(lexer.get_token(), Some(TokenType::INTEGER { value: BigInt::from(1) }));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::AND)));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "b".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::EQEQUAL)));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "c".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::OR)));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "d".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::NOTEQUAL)));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "e".to_string() }));
//     }
//
//     // Test pour les délimiteurs
//     #[test]
//     fn test_lex_delimiters() {
//         let mut lexer = Lexer::new("( { [ ] } )");
//         assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::LPAR)));
//         assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::LCURBRACE)));
//         assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::LSBRACKET)));
//         assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::RSBRACKET)));
//         assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::RCURBRACE)));
//         assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::RPAR)));
//     }
//
//     // Test pour les délimiteurs imbriqués
//     #[test]
//     fn test_lex_nested_delimiters() {
//         let mut lexer = Lexer::new("{[(())]}");
//         assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::LCURBRACE)));
//         assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::LSBRACKET)));
//         assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::LPAR)));
//         assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::LPAR)));
//         assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::RPAR)));
//         assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::RPAR)));
//         assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::RSBRACKET)));
//         assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::RCURBRACE)));
//     }
//
//     // Test pour les caractères inattendus
//     #[test]
//     fn test_lex_unexpected_character() {
//         let mut lexer = Lexer::new("@ $");
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::AT)));
//         assert_eq!(lexer.get_token(), Some(TokenType::ERROR(LexerError::invalid_token("$", Position { line: 1, column: 4 }))));
//     }
//
//     // Test pour un mélange d'opérateurs et de caractères inconnus
//     #[test]
//     fn test_mixed_operators_and_unknown() {
//         let mut lexer = Lexer::new("@$");
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::AT)));
//         assert_eq!(lexer.get_token(), Some(TokenType::ERROR(LexerError::invalid_token("$", Position { line: 1, column: 3 }))));
//     }
//
//     // Test pour l'opérateur @ seul
//     #[test]
//     fn test_arobase() {
//         let mut lexer = Lexer::new("@");
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::AT)));
//     }
//
//     // Test pour un mélange d'opérateurs et de délimiteurs
//     #[test]
//     fn test_lex_mixed_input() {
//         let mut lexer = Lexer::new(r#"let sum = a + b * (c - d) / e;"#);
//         assert_eq!(lexer.get_token(), Some(TokenType::KEYWORD(Keywords::LET)));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "sum".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::EQUAL)));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "a".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::PLUS)));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "b".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::STAR)));
//         assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::LPAR)));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "c".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::MINUS)));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "d".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::RPAR)));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::SLASH)));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "e".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::SEMICOLON)));
//     }
//
//     // Test pour des tokens de base avec des commentaires et des chaînes multi-lignes
//     #[test]
//     fn test_basic_tokens() {
//         let input = r#"
//             let x = 42;
//             let y = 3.14;
//             if (x > y) {
//                 print("x is greater");
//             } else {
//                 print("y is greater or equal");
//             }
//
//             // This is a comment
//             /* This is a
//                multi-line comment */
//         "#;
//
//         let mut lexer = Lexer::new(input);
//         let tokens = lexer.tokenize();
//
//         let expected_tokens = vec![
//             TokenType::KEYWORD(Keywords::LET),
//             TokenType::IDENTIFIER { name: "x".to_string() },
//             TokenType::OPERATOR(Operators::EQUAL),
//             TokenType::INTEGER { value: BigInt::from(42) },
//             TokenType::DELIMITER(Delimiters::SEMICOLON),
//             TokenType::KEYWORD(Keywords::LET),
//             TokenType::IDENTIFIER { name: "y".to_string() },
//             TokenType::OPERATOR(Operators::EQUAL),
//             TokenType::FLOAT { value: 3.14 },
//             TokenType::DELIMITER(Delimiters::SEMICOLON),
//             TokenType::KEYWORD(Keywords::IF),
//             TokenType::DELIMITER(Delimiters::LPAR),
//             TokenType::IDENTIFIER { name: "x".to_string() },
//             TokenType::OPERATOR(Operators::GREATER),
//             TokenType::IDENTIFIER { name: "y".to_string() },
//             TokenType::DELIMITER(Delimiters::RPAR),
//             TokenType::DELIMITER(Delimiters::LCURBRACE),
//             TokenType::IDENTIFIER { name: "print".to_string() },
//             TokenType::DELIMITER(Delimiters::LPAR),
//             TokenType::STRING { value: "x is greater".to_string(), kind: StringKind::NORMAL },
//             TokenType::DELIMITER(Delimiters::RPAR),
//             TokenType::DELIMITER(Delimiters::SEMICOLON),
//             TokenType::DELIMITER(Delimiters::RCURBRACE),
//             TokenType::KEYWORD(Keywords::ELSE),
//             TokenType::DELIMITER(Delimiters::LCURBRACE),
//             TokenType::IDENTIFIER { name: "print".to_string() },
//             TokenType::DELIMITER(Delimiters::LPAR),
//             TokenType::STRING { value: "y is greater or equal".to_string(), kind: StringKind::NORMAL },
//             TokenType::DELIMITER(Delimiters::RPAR),
//             TokenType::DELIMITER(Delimiters::SEMICOLON),
//             TokenType::DELIMITER(Delimiters::RCURBRACE),
//             TokenType::COMMENT(" This is a comment".to_string()),
//             TokenType::COMMENT(" This is a\n               multi-line comment ".to_string()),
//             TokenType::EOF,
//         ];
//
//         assert_eq!(tokens.len(), expected_tokens.len(), "Le nombre de tokens ne correspond pas");
//
//         for (token, expected_type) in tokens.iter().zip(expected_tokens.iter()) {
//             assert_eq!(&token.token_type, expected_type, "Type de token incorrect pour '{}'", token.text);
//         }
//     }
//
//     // Test pour les chaînes vides
//     #[test]
//     fn test_empty_string() {
//         let mut lexer = Lexer::new(r#""""#);
//         assert_eq!(
//             lexer.get_token(),
//             Some(TokenType::STRING {
//                 value: "".to_string(),
//                 kind: StringKind::NORMAL
//             })
//         );
//     }
//
//     // Test pour les nombres négatifs
//     #[test]
//     fn test_negative_numbers() {
//         let mut lexer = Lexer::new("-42 -3.14");
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::MINUS)));
//         assert_eq!(lexer.get_token(), Some(TokenType::INTEGER { value: BigInt::from(42) }));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::MINUS)));
//         assert_eq!(lexer.get_token(), Some(TokenType::FLOAT { value: 3.14 }));
//     }
//
//     // Test pour les identifiants avec des underscores
//     #[test]
//     fn test_identifiers_with_underscores() {
//         let mut lexer = Lexer::new("my_variable _private");
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "my_variable".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "_private".to_string() }));
//     }
//
//     // Test pour les opérateurs composés
//     #[test]
//     fn test_compound_operators() {
//         let mut lexer = Lexer::new("++ -- ** //");
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::PLUSEQUAL)));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::MINEQUAL)));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::DOUBLESTAR)));
//         assert_eq!(lexer.get_token(), Some(TokenType::COMMENT("".to_string())));
//     }
//
//     // Test pour la gestion des espaces et des sauts de ligne
//     #[test]
//     fn test_whitespace_handling() {
//         let mut lexer = Lexer::new("let   x\n=\t42");
//         assert_eq!(lexer.get_token(), Some(TokenType::KEYWORD(Keywords::LET)));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "x".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::EQUAL)));
//         assert_eq!(lexer.get_token(), Some(TokenType::INTEGER { value: BigInt::from(42) }));
//     }
//
//     // Test pour les caractères d'échappement dans les chaînes
//     #[test]
//     fn test_string_escape_sequences() {
//         let mut lexer = Lexer::new(r#""Hello\nWorld\t\"Escaped\"""#);
//         assert_eq!(
//             lexer.get_token(),
//             Some(TokenType::STRING {
//                 value: "Hello\nWorld\t\"Escaped\"".to_string(),
//                 kind: StringKind::NORMAL
//             })
//         );
//     }
//
//     // Test pour les chaînes multi-lignes complexes
//     #[test]
//     fn test_multiline_strings() {
//         let input = r#"
//         "This is a simple string"
//         "This is a string \
//          that spans multiple \
//          lines"
//         "This string has
//          actual newlines"
//         "This string has \n escaped newlines"
//         "Mixed \
//          newlines \n and \
//          continuations"
//     "#;
//
//         let mut lexer = Lexer::new(input);
//
//         assert_eq!(
//             lexer.get_token(),
//             Some(TokenType::STRING {
//                 value: "This is a simple string".to_string(),
//                 kind: StringKind::NORMAL
//             })
//         );
//
//         assert_eq!(
//             lexer.get_token(),
//             Some(TokenType::STRING {
//                 value: "This is a string that spans multiple lines".to_string(),
//                 kind: StringKind::NORMAL
//             })
//         );
//
//         assert_eq!(
//             lexer.get_token(),
//             Some(TokenType::STRING {
//                 value: "This string has\n         actual newlines".to_string(),
//                 kind: StringKind::NORMAL
//             })
//         );
//
//         assert_eq!(
//             lexer.get_token(),
//             Some(TokenType::STRING {
//                 value: "This string has \n escaped newlines".to_string(),
//                 kind: StringKind::NORMAL
//             })
//         );
//
//         assert_eq!(
//             lexer.get_token(),
//             Some(TokenType::STRING {
//                 value: "Mixed newlines \n and continuations".to_string(),
//                 kind: StringKind::NORMAL
//             })
//         );
//
//         assert_eq!(lexer.get_token(), Some(TokenType::EOF));
//     }
//
//     // Test pour les commentaires et les commentaires de documentation
//     #[test]
//     fn test_comments_and_doc_comments() {
//         let mut lexer = Lexer::new("// This is a regular comment\n/// This is a doc comment\n/* Multi-line comment */");
//         assert_eq!(lexer.get_token(), Some(TokenType::COMMENT(" This is a regular comment".to_string())));
//         assert_eq!(lexer.get_token(), Some(TokenType::DOCSTRING(" This is a doc comment".to_string())));
//         assert_eq!(lexer.get_token(), Some(TokenType::COMMENT(" Multi-line comment ".to_string())));
//     }
//
//     #[test]
//     fn test_invalid_identifier() {
//         let mut lexer = Lexer::new("var$");
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "var".to_string() }));
//
//         if let Some(TokenType::ERROR(error)) = lexer.get_token() {
//             assert_eq!(error.error, LexerErrorType::InvalidToken("$".to_string()));
//         } else {
//             panic!("Expected an ERROR token for '$'");
//         }
//
//         assert_eq!(lexer.get_token(), Some(TokenType::EOF));
//     }
//
//
//     #[test]
//     fn test_invalid_escape_sequence() {
//         let mut lexer = Lexer::new(r#""This is a bad escape: \q""#);
//         assert_eq!(lexer.get_token(), Some(TokenType::STRING { value: "This is a bad escape: q".to_string(), kind: StringKind::NORMAL }));
//     }
//
//     #[test]
//     fn test_complex_expression() {
//         let mut lexer = Lexer::new("a + (b - (c * d) / e) && f || g");
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "a".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::PLUS)));
//         assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::LPAR)));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "b".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::MINUS)));
//         assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::LPAR)));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "c".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::STAR)));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "d".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::RPAR)));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::SLASH)));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "e".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::RPAR)));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::AND)));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "f".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::OR)));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "g".to_string() }));
//     }
//
//     #[test]
//     fn test_reserved_keyword_as_identifier() {
//         let mut lexer = Lexer::new("fn fn");
//         assert_eq!(lexer.get_token(), Some(TokenType::KEYWORD(Keywords::FN)));
//         assert_eq!(lexer.get_token(), Some(TokenType::KEYWORD(Keywords::FN)));
//     }
//
//     #[test]
//     fn test_large_numbers() {
//         let mut lexer = Lexer::new("123456789012345678901234567890");
//         if let Some(TokenType::ERROR(error)) = lexer.get_token() {
//             assert_eq!(error.error, LexerErrorType::InvalidInteger("123456789012345678901234567890".to_string()));
//         } else {
//             panic!("Expected an ERROR token for very large integer");
//         }
//     }
//
//     #[test]
//     fn test_string_with_keywords_and_operators() {
//         let mut lexer = Lexer::new(r#""if else let + -""#);
//         assert_eq!(lexer.get_token(), Some(TokenType::STRING { value: "if else let + -".to_string(), kind: StringKind::NORMAL }));
//     }
//
//     #[test]
//     fn test_lex_hex_number() {
//         let mut lexer = Lexer::new("0x1A3F 0Xb4");
//         assert_eq!(lexer.get_token(), Some(TokenType::HEXADECIMAL { value: 0x1A3F }));
//         assert_eq!(lexer.get_token(), Some(TokenType::HEXADECIMAL { value: 0xB4 }));
//     }
//
//     #[test]
//     fn test_invalid_hex_number() {
//         let mut lexer = Lexer::new("0xGHI");
//         if let Some(TokenType::ERROR(error)) = lexer.get_token() {
//             assert_eq!(error.error, LexerErrorType::InvalidHexadecimal("0x".to_string()));
//         } else {
//             panic!("Expected an ERROR token for invalid hexadecimal");
//         }
//     }
//
//     #[test]
//     fn test_valid_hexadecimal() {
//         let mut lexer = Lexer::new("0x1A3F");
//         assert_eq!(lexer.get_token(), Some(TokenType::HEXADECIMAL { value: 0x1A3F }));
//     }
//
//     #[test]
//     fn test_valid_hexadecimal_uppercase(){
//         let mut lexer = Lexer::new("0X1A3F");
//         assert_eq!(lexer.get_token(), Some(TokenType::HEXADECIMAL { value: 0x1A3F }));
//
//     }
//
//     #[test]
//     fn test_valid_hexadecimal_mixed_case() {
//         let mut lexer = Lexer::new("0xaBcD");
//         assert_eq!(lexer.get_token(), Some(TokenType::HEXADECIMAL { value: 0xABCD }));
//     }
//
//     #[test]
//     fn test_valid_hexadecimal_mixed_case_2(){
//         let mut lexer = Lexer::new("0x1A3F 0Xb4");
//         assert_eq!(lexer.get_token(), Some(TokenType::HEXADECIMAL { value: 0x1A3F }));
//         assert_eq!(lexer.get_token(), Some(TokenType::HEXADECIMAL { value: 0xB4 }));
//     }
//
//     #[test]
//     fn test_valid_hexadecimal_zero() {
//         let mut lexer = Lexer::new("0x0");
//         assert_eq!(lexer.get_token(), Some(TokenType::HEXADECIMAL { value: 0 }));
//     }
//
//     #[test]
//     fn test_valid_hexadecimal_max() {
//         let mut lexer = Lexer::new("0xFFFFFFFFFFFFFFFF");
//         assert_eq!(lexer.get_token(), Some(TokenType::HEXADECIMAL { value: 0xFFFFFFFFFFFFFFFF }));
//     }
//
//
//
//     #[test]
//     fn test_hexadecimal_followed_by_identifier() {
//         let mut lexer = Lexer::new("0xABC def");
//         assert_eq!(lexer.get_token(), Some(TokenType::HEXADECIMAL { value: 0xABC }));
//         assert_eq!(lexer.get_token(), Some(TokenType::KEYWORD(Keywords::DEF)));
//     }
//
//     #[test]
//     fn test_multiple_hexadecimals() {
//         let mut lexer = Lexer::new("0x123 0x456 0x789");
//         assert_eq!(lexer.get_token(), Some(TokenType::HEXADECIMAL { value: 0x123 }));
//         assert_eq!(lexer.get_token(), Some(TokenType::HEXADECIMAL { value: 0x456 }));
//         assert_eq!(lexer.get_token(), Some(TokenType::HEXADECIMAL { value: 0x789 }));
//     }
//
//
//
//     /// Test pour vérifier la tokenisation des opérateurs complexes.
//     #[test]
//     fn test_lex_complex_operator_3() {
//         let mut lexer = Lexer::new("a += 1 && b == c || d != e");
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "a".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::PLUSEQUAL)));
//         assert_eq!(lexer.get_token(), Some(TokenType::INTEGER { value: BigInt::from(1) }));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::AND)));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "b".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::EQEQUAL)));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "c".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::OR)));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "d".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::NOTEQUAL)));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "e".to_string() }));
//     }
//
//
// ///////////////////////////////////////////////////ERROR HANDLING TEST ///////////////////////////////////////////////////////////////////////////
// #[test]
// fn test_invalid_number() {
//     let mut lexer = Lexer::new("123a");
//
//     // Le premier token devrait être un entier valide
//     assert_eq!(lexer.get_token(), Some(TokenType::INTEGER { value: BigInt::from(123) }));
//
//     // Le deuxième token devrait être un identifiant
//     assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "a".to_string() }));
//
//     // Il ne devrait plus y avoir de tokens
//     assert_eq!(lexer.get_token(), Some(TokenType::EOF));
// }
//
//
//
//     //Test pour un float invalide
//     #[test]
//     fn test_invalid_float() {
//         let mut lexer = Lexer::new("3.14.15");
//         assert_eq!(lexer.get_token(), Some(TokenType::ERROR(LexerError::invalid_float("3.14.15", Position { line: 1, column: 8 }))));
//     }
//
//     // Test pour une chaîne non terminée
//     #[test]
//     fn test_unterminated_string() {
//         let mut lexer = Lexer::new("\"Hello, world");
//         assert_eq!(
//             lexer.get_token(),
//             Some(TokenType::ERROR(LexerError::unterminated_string(Position { line: 1, column: 14 })))
//         );
//     }
//
//     // Test pour un commentaire non terminé
//     #[test]
//     fn test_unterminated_comment() {
//         let mut lexer = Lexer::new("/* This is a comment");
//         assert_eq!(
//             lexer.get_token(),
//             Some(TokenType::ERROR(LexerError::unterminated_comment(Position { line: 1, column: 21 })))
//         );
//     }
//
//     //Test pour un hexadécimal invalide
//     #[test]
//     fn test_invalid_hexadecimal() {
//         let mut lexer = Lexer::new("0xGHI");
//         assert_eq!(
//             lexer.get_token(),
//             Some(TokenType::ERROR(LexerError::new(
//                 LexerErrorType::InvalidHexadecimal("0x".to_string()),
//                 "Invalid hexadecimal: 0x".to_string(),
//                 Position { line: 1, column: 3 }
//             )))
//         );
//     }
//
//     //Test pour un hexadécimal valide suivi d'un identifiant
//     #[test]
//     fn test_hexadecimal_followed_by_identifier_2() {
//         let mut lexer = Lexer::new("0x1FAb identifier");
//         assert_eq!(lexer.get_token(), Some(TokenType::HEXADECIMAL { value: 0x1FAb }));
//         assert_eq!(
//             lexer.get_token(),
//             Some(TokenType::IDENTIFIER { name: "identifier".to_string() })
//         );
//     }
//
//     // Test pour une chaîne avec un saut de ligne non échappé
//     #[test]
//     fn test_multiline_string() {
//         let mut lexer = Lexer::new("\"Hello\nWorld\"");
//         assert_eq!(
//             lexer.get_token(),
//             Some(TokenType::STRING {
//                 value: "Hello\nWorld".to_string(),
//                 kind: StringKind::NORMAL
//             })
//         );
//     }
//
//     // Test pour un commentaire multi-ligne non terminé
//     #[test]
//     fn test_unterminated_multiline_comment() {
//         let mut lexer = Lexer::new("/* This is a multi-line comment that doesn't end");
//         assert_eq!(
//             lexer.get_token(),
//             Some(TokenType::ERROR(LexerError::unterminated_comment(Position { line: 1, column: 49 })))
//         );
//     }
//
//     #[test]
//     fn test_unknown_operator() {
//         let mut lexer = Lexer::new("a $$ b");
//
//
//         // Le premier token devrait être un identifiant
//         assert_eq!(
//             lexer.get_token(),
//             Some(TokenType::IDENTIFIER { name: "a".to_string() })
//         );
//
//         // Les deux prochains tokens devraient être des erreurs pour des opérateurs inconnus
//         for _ in 0..2 {
//             if let Some(TokenType::ERROR(error)) = lexer.get_token() {
//                 assert_eq!(error.error, LexerErrorType::InvalidToken("$".to_string()));
//                 // Nous ne vérifions pas la position exacte ici
//             } else {
//                 panic!("Expected an ERROR token for unknown operator");
//             }
//         }
//
//         // Le quatrième token devrait être un autre identifiant
//         assert_eq!(
//             lexer.get_token(),
//             Some(TokenType::IDENTIFIER { name: "b".to_string() })
//         );
//
//         // Assurons-nous qu'il n'y a plus de tokens
//         assert_eq!(lexer.get_token(), Some(TokenType::EOF));
//     }
//
//
//
// }
//
//
// ////// TESTS AVANCÉS //////
//
// mod advanced_tests {
//     use super::*;
//
//     // Tests de performance
//     #[test]
//     fn test_lexer_performance_large_input() {
//         let single_line = "let x = 42;\n";
//         let num_lines = 100000;
//
//         let start = Instant::now();
//         let mut token_count = 0;
//
//         for _ in 0..num_lines {
//             let mut lexer = Lexer::new(single_line);
//             while let Some(token) = lexer.get_token() {
//                 token_count += 1;
//                 if matches!(token, TokenType::EOF) {
//                     break;
//                 }
//             }
//         }
//
//         let duration = start.elapsed();
//
//         println!("Tokenizing {} lines took: {:?}", num_lines, duration);
//         println!("Total tokens processed: {}", token_count);
//         println!("Tokens per second: {}", token_count as f64 / duration.as_secs_f64());
//
//         // Ajuster le seuil en fonction de la performance actuelle
//         assert!(duration.as_secs() < 15, "Tokenizing took too long");
//         assert_eq!(token_count, num_lines * 6); // 6 tokens per line (let, x, =, 42, ;, EOF)
//     }
//
//
//
//     #[test]
//     fn test_lexer_robustness_very_long_identifier() {
//         let very_long_identifier = "a".repeat(10000);
//         let mut lexer = Lexer::new(&very_long_identifier);
//         if let Some(TokenType::IDENTIFIER { name }) = lexer.get_token() {
//             assert_eq!(name, very_long_identifier);
//         } else {
//             panic!("Expected an IDENTIFIER token for very long identifier");
//         }
//         assert_eq!(lexer.get_token(), Some(TokenType::EOF));
//     }
//
//     #[test]
//     fn test_lexer_robustness_very_long_number() {
//         let very_long_number = "9".repeat(1000);
//         let mut lexer = Lexer::new(&very_long_number);
//         if let Some(TokenType::ERROR(error)) = lexer.get_token() {
//             let expected_error = LexerError::invalid_integer(&very_long_number, Position { line: 1, column: 1001 });
//             assert_eq!(error.error, expected_error.error);
//             assert_eq!(error.message, expected_error.message);
//             assert_eq!(error.position, expected_error.position);
//         } else {
//             panic!("Expected an ERROR token for very long number");
//         }
//     }
//
//
//
//     #[test]
//     fn test_lexer_robustness_deeply_nested_expressions() {
//         let deeply_nested = "(".repeat(1000) + &"42".to_string() + &")".repeat(1000);
//         let mut lexer = Lexer::new(&deeply_nested);
//
//         for _ in 0..1000 {
//             assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::LPAR)));
//         }
//         assert_eq!(lexer.get_token(), Some(TokenType::INTEGER { value: BigInt::from(42) }));
//         for _ in 0..1000 {
//             assert_eq!(lexer.get_token(), Some(TokenType::DELIMITER(Delimiters::RPAR)));
//         }
//         assert_eq!(lexer.get_token(), Some(TokenType::EOF));
//     }
//
//
//     #[test]
//     fn test_lexer_robustness_malformed_input() {
//         let malformed_input = "let x = @#$%^&*;";
//         let mut lexer = Lexer::new(malformed_input);
//
//         assert_eq!(lexer.get_token(), Some(TokenType::KEYWORD(Keywords::LET)));
//         assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "x".to_string() }));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::EQUAL)));
//         assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::AT)));
//
//         // Vérifie si le reste est traité comme un commentaire
//         if let Some(TokenType::COMMENT(comment)) = lexer.get_token() {
//             assert_eq!(comment, "$%^&*;");
//         } else {
//             panic!("Expected a COMMENT token for '#$%^&*;'");
//         }
//
//         // Vérifie qu'il n'y a plus de tokens
//         assert_eq!(lexer.get_token(), Some(TokenType::EOF));
//     }
//
//     // #[test]
//     // fn test_lexer_robustness_malformed_input_2() {
//     //     let malformed_input = "let x = @#$%^&*;";
//     //     let mut lexer = Lexer::new(malformed_input);
//     //
//     //     assert_eq!(lexer.get_token(), Some(TokenType::KEYWORD(Keywords::LET)));
//     //     assert_eq!(lexer.get_token(), Some(TokenType::IDENTIFIER { name: "x".to_string() }));
//     //     assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::EQUAL)));
//     //     assert_eq!(lexer.get_token(), Some(TokenType::OPERATOR(Operators::AT)));
//     //
//     //     // Vérifie le traitement des caractères non reconnus
//     //     let invalid_chars = "#$%^&*;";
//     //     for (i, ch) in invalid_chars.chars().enumerate() {
//     //         if let Some(TokenType::ERROR(error)) = lexer.get_token() {
//     //             let expected_error = LexerError::invalid_token(&ch.to_string(), Position { line: 1, column: 9 + i });
//     //             assert_eq!(error.error, expected_error.error);
//     //             assert_eq!(error.message, expected_error.message);
//     //             assert_eq!(error.position, expected_error.position);
//     //         } else {
//     //             panic!("Expected an ERROR token for '{}'", ch);
//     //         }
//     //     }
//     //
//     //     // Vérifie qu'il n'y a plus de tokens
//     //     assert_eq!(lexer.get_token(), Some(TokenType::EOF));
//     // }
//
// }

// //by YmC




