// src/parser/token_matcher.rs
// Module pour le matching avancé et l'analyse des tokens

use crate::parser::ast::{Visibility, Mutability};
use crate::parser::parser::Parser;
use crate::parser::parser_error::{ParserError, ParserErrorType};
use crate::tok::{TokenType, Keywords, Delimiters, Operators};
use crate::SyntaxMode;

impl Parser {
    /// Parse la visibilité (pub ou private par défaut)
    pub fn parse_visibility(&mut self) -> Option<Visibility> {
        if self.match_token(&[TokenType::KEYWORD(Keywords::PUB)]) {
            Some(Visibility::Public)
        } else {
            None
        }
    }

    /// Parse la mutabilité (mut ou immutable par défaut)
    pub fn parse_mutability(&mut self) -> Mutability {
        if self.match_token(&[TokenType::KEYWORD(Keywords::MUT)]) {
            Mutability::Mutable
        } else {
            Mutability::Immutable
        }
    }

    /// Vérifie et parse un label optionnel (pour les boucles)
    pub fn check_for_label(&mut self) -> Result<Option<String>, ParserError> {
        // Vérifie si on a pattern: identifier ':' suivi d'un keyword de boucle
        if let Some(current) = self.current_token() {
            if let TokenType::IDENTIFIER { name, .. } = &current.token_type {
                if let Some(next) = self.peek_next_token() {
                    if matches!(next.token_type, TokenType::DELIMITER(Delimiters::COLON)) {
                        // Vérifie que c'est bien un label de boucle
                        if let Some(third) = self.tokens.get(self.current + 2) {
                            if matches!(third.token_type, 
                                TokenType::KEYWORD(Keywords::LOOP) |
                                TokenType::KEYWORD(Keywords::WHILE) |
                                TokenType::KEYWORD(Keywords::FOR)
                            ) {
                                let label = name.clone();
                                self.advance(); // Consomme l'identifier
                                self.advance(); // Consomme le ':'
                                return Ok(Some(label));
                            }
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    /// Consomme un séparateur selon le mode syntaxique
    pub fn consume_seperator(&mut self) {
        match self.syntax_mode {
            SyntaxMode::Braces => {
                // En mode Braces, on attend un ';'
                if self.check(&[TokenType::DELIMITER(Delimiters::SEMICOLON)]) {
                    self.advance();
                }
                // Note: On ne retourne pas d'erreur si absent pour flexibilité
            }
            SyntaxMode::Indentation => {
                // En mode Indentation, on peut avoir un NEWLINE
                if self.check(&[TokenType::NEWLINE]) {
                    self.advance();
                }
            }
        }
    }

    /// Vérifie si le token actuel est un opérateur binaire
    pub fn is_binary_operator(&self) -> bool {
        if let Some(token) = self.current_token() {
            matches!(token.token_type,
                TokenType::OPERATOR(Operators::PLUS) |
                TokenType::OPERATOR(Operators::MINUS) |
                TokenType::OPERATOR(Operators::STAR) |
                TokenType::OPERATOR(Operators::SLASH) |
                TokenType::OPERATOR(Operators::PERCENT) |
                TokenType::OPERATOR(Operators::LESS) |
                TokenType::OPERATOR(Operators::GREATER) |
                TokenType::OPERATOR(Operators::LESSEQUAL) |
                TokenType::OPERATOR(Operators::GREATEREQUAL) |
                TokenType::OPERATOR(Operators::EQEQUAL) |
                TokenType::OPERATOR(Operators::NOTEQUAL) |
                TokenType::OPERATOR(Operators::AND) |
                TokenType::OPERATOR(Operators::OR) |
                TokenType::OPERATOR(Operators::EQUAL)
            )
        } else {
            false
        }
    }

    /// Vérifie si le token actuel est un opérateur unaire
    pub fn is_unary_operator(&self) -> bool {
        if let Some(token) = self.current_token() {
            matches!(token.token_type,
                TokenType::OPERATOR(Operators::MINUS) |
                TokenType::OPERATOR(Operators::EXCLAMATION) |
                TokenType::OPERATOR(Operators::TILDE) |

                TokenType::OPERATOR(Operators::PLUSEQUAL) |
                TokenType::OPERATOR(Operators::MINEQUAL) |
                TokenType::OPERATOR(Operators::STAR) |  // Déréférencement
                TokenType::OPERATOR(Operators::AMPER)      // Référence
            )
        } else {
            false
        }
    }

    /// Vérifie si le token actuel est un opérateur d'assignation composé
    pub fn is_compound_assignment(&self) -> bool {
        if let Some(token) = self.current_token() {
            matches!(token.token_type,
                TokenType::OPERATOR(Operators::PLUSEQUAL) |
                TokenType::OPERATOR(Operators::MINEQUAL) |
                TokenType::OPERATOR(Operators::STAREQUAL) |
                TokenType::OPERATOR(Operators::SLASHEQUAL) |
                TokenType::OPERATOR(Operators::PERCENTEQUAL)
            )
        } else {
            false
        }
    }

    /// Vérifie si le token actuel est un début de type
    pub fn is_type_start(&self) -> bool {
        if let Some(token) = self.current_token() {
            matches!(token.token_type,
                TokenType::KEYWORD(Keywords::INT) |
                TokenType::KEYWORD(Keywords::FLOAT) |
                TokenType::KEYWORD(Keywords::STR) |
                TokenType::KEYWORD(Keywords::BOOL) |
                TokenType::KEYWORD(Keywords::CHAR) |
                TokenType::KEYWORD(Keywords::NONE) |
                TokenType::IDENTIFIER { .. } |
                TokenType::DELIMITER(Delimiters::LSBRACKET) |  // Array type
                TokenType::DELIMITER(Delimiters::LPAR) |     // Tuple type
                TokenType::OPERATOR(Operators::AMPER) |          // Reference type
                TokenType::OPERATOR(Operators::STAR)           // Pointer type
            )
        } else {
            false
        }
    }

    /// Vérifie si le token actuel est un mot-clé de déclaration
    pub fn is_declaration_keyword(&self) -> bool {
        if let Some(token) = self.current_token() {
            matches!(token.token_type,
                TokenType::KEYWORD(Keywords::LET) |
                TokenType::KEYWORD(Keywords::CONST) |
                TokenType::KEYWORD(Keywords::FN) |
                TokenType::KEYWORD(Keywords::STRUCT) |
                TokenType::KEYWORD(Keywords::ENUM) |
                TokenType::KEYWORD(Keywords::CLASS) |
                TokenType::KEYWORD(Keywords::TRAIT) |
                TokenType::KEYWORD(Keywords::IMPL) |
                TokenType::KEYWORD(Keywords::TYPE) |
                TokenType::KEYWORD(Keywords::PUB)
            )
        } else {
            false
        }
    }

    /// Vérifie si le token actuel est un mot-clé de statement
    pub fn is_statement_keyword(&self) -> bool {
        if let Some(token) = self.current_token() {
            matches!(token.token_type,
                TokenType::KEYWORD(Keywords::IF) |
                TokenType::KEYWORD(Keywords::WHILE) |
                TokenType::KEYWORD(Keywords::FOR) |
                TokenType::KEYWORD(Keywords::LOOP) |
                TokenType::KEYWORD(Keywords::MATCH) |
                TokenType::KEYWORD(Keywords::RETURN) |
                TokenType::KEYWORD(Keywords::BREAK) |
                TokenType::KEYWORD(Keywords::CONTINUE) |
                TokenType::KEYWORD(Keywords::TRY)
            )
        } else {
            false
        }
    }

    /// Attend un token spécifique et retourne une erreur détaillée si absent
    pub fn expect_token(&mut self, expected: TokenType) -> Result<(), ParserError> {
        if self.check(&[expected.clone()]) {
            self.advance();
            Ok(())
        } else {
            let found = self.current_token()
                .map(|t| format!("{:?}", t.token_type))
                .unwrap_or_else(|| "EOF".to_string());
            
            let message = format!(
                "Expected {:?} but found {} in {} mode",
                expected,
                found,
                match self.syntax_mode {
                    SyntaxMode::Braces => "Braces",
                    SyntaxMode::Indentation => "Indentation",
                }
            );
            
            Err(ParserError {
                error: ParserErrorType::UnexpectedToken,
                message,
                position: self.current_position(),
            })
        }
    }

    /// Consomme un token optionnel s'il est présent
    pub fn optional_token(&mut self, token_type: TokenType) -> bool {
        if self.check(&[token_type]) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Vérifie si on peut commencer à parser un bloc selon le mode
    pub fn can_start_block(&self) -> bool {
        match self.syntax_mode {
            SyntaxMode::Braces => {
                self.check(&[TokenType::DELIMITER(Delimiters::LCURBRACE)])
            }
            SyntaxMode::Indentation => {
                self.check(&[TokenType::DELIMITER(Delimiters::COLON)])
            }
        }
    }

    /// Vérifie si on est à la fin d'un bloc selon le mode
    pub fn is_block_end(&self) -> bool {
        match self.syntax_mode {
            SyntaxMode::Braces => {
                self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)])
            }
            SyntaxMode::Indentation => {
                self.check(&[TokenType::DEDENT, TokenType::EOF])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex::{Lexer, SyntaxMode};

    fn create_parser(code: &str, mode: SyntaxMode) -> Parser {
        let mut lexer = Lexer::new(code, mode);
        let tokens = lexer.tokenize();
        Parser::new(tokens, mode)
    }

    #[test]
    fn test_parse_visibility() {
        let mut parser = create_parser("pub fn test()", SyntaxMode::Braces);
        let vis = parser.parse_visibility();
        assert_eq!(vis, Some(Visibility::Public));
        
        let mut parser2 = create_parser("fn test()", SyntaxMode::Braces);
        let vis2 = parser2.parse_visibility();
        assert_eq!(vis2, None);
    }

    #[test]
    fn test_parse_mutability() {
        let mut parser = create_parser("mut x", SyntaxMode::Braces);
        let mutability = parser.parse_mutability();
        assert_eq!(mutability, Mutability::Mutable);
        
        let mut parser2 = create_parser("x", SyntaxMode::Braces);
        let mutability2 = parser2.parse_mutability();
        assert_eq!(mutability2, Mutability::Immutable);
    }

    #[test]
    fn test_is_declaration_keyword() {
        let parser = create_parser("let x = 5", SyntaxMode::Braces);
        assert!(parser.is_declaration_keyword());
        
        let parser2 = create_parser("if x > 5", SyntaxMode::Braces);
        assert!(!parser2.is_declaration_keyword());
    }

    #[test]
    fn test_is_statement_keyword() {
        let parser = create_parser("if x > 5", SyntaxMode::Braces);
        assert!(parser.is_statement_keyword());
        
        let parser2 = create_parser("let x = 5", SyntaxMode::Braces);
        assert!(!parser2.is_statement_keyword());
    }

    #[test]
    fn test_can_start_block() {
        let parser_braces = create_parser("{ let x = 5; }", SyntaxMode::Braces);
        assert!(parser_braces.can_start_block());
        
        let parser_indent = create_parser(": let x = 5", SyntaxMode::Indentation);
        assert!(parser_indent.can_start_block());
    }
}