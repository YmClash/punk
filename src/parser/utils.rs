// src/parser/utils.rs
// Utilitaires de base pour la navigation et manipulation des tokens

use crate::lexer::lex::Token;
use crate::parser::parser::Parser;
use crate::parser::parser_error::{ParserError, ParserErrorType, Position};
use crate::tok::TokenType;

impl Parser {
    /// Avance au prochain token
    pub fn advance(&mut self) {
        if self.current < self.tokens.len() {
            self.current += 1;
        }
    }

    /// Retourne le token actuel sans avancer
    pub fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    /// Retourne le prochain token sans avancer
    pub fn peek_token(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    /// Retourne le token suivant le token actuel
    pub fn peek_next_token(&self) -> Option<&Token> {
        self.tokens.get(self.current + 1)
    }

    /// Vérifie si on a atteint la fin des tokens
    pub fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || 
        self.current_token()
            .map(|t| matches!(t.token_type, TokenType::EOF))
            .unwrap_or(true)
    }

    /// Vérifie si le token actuel correspond à l'un des types donnés
    pub fn check(&self, expected: &[TokenType]) -> bool {
        if let Some(token) = self.current_token() {
            expected.contains(&token.token_type)
        } else {
            false
        }
    }

    /// Vérifie et consomme si le token actuel correspond à l'un des types donnés
    pub fn match_token(&mut self, token_types: &[TokenType]) -> bool {
        if self.check(token_types) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consomme le token attendu ou retourne une erreur
    pub fn consume(&mut self, expected: TokenType) -> Result<(), ParserError> {
        if self.check(&[expected.clone()]) {
            self.advance();
            Ok(())
        } else {
            let found = self.current_token()
                .map(|t| format!("{:?}", t.token_type))
                .unwrap_or_else(|| "EOF".to_string());
            
            Err(ParserError::new(
                ParserErrorType::UnexpectedToken,
                self.current_position(),
            ))
        }
    }

    /// Consomme un identifiant et retourne sa valeur
    pub fn consume_identifier(&mut self) -> Result<String, ParserError> {
        if let Some(token) = self.current_token() {
            if let TokenType::IDENTIFIER { name, .. } = &token.token_type {
                let name = name.clone();
                self.advance();
                Ok(name)
            } else {
                Err(ParserError::new(
                    ParserErrorType::ExpectIdentifier,
                    self.current_position(),
                ))
            }
        } else {
            Err(ParserError::new(
                ParserErrorType::UnexpectedEOF,
                self.current_position(),
            ))
        }
    }

    /// Retourne la position actuelle dans le flux de tokens
    pub fn current_position(&self) -> Position {
        Position {
            index: self.current,
        }
    }

    /// Retourne le token précédent
    pub fn previous_token(&self) -> Option<&Token> {
        if self.current > 0 {
            self.tokens.get(self.current - 1)
        } else {
            None
        }
    }

    /// Sauvegarde la position actuelle
    pub fn save_position(&self) -> usize {
        self.current
    }

    /// Restaure une position sauvegardée
    pub fn restore_position(&mut self, position: usize) {
        self.current = position;
    }

    /// Avance jusqu'à trouver un des tokens spécifiés
    pub fn advance_until(&mut self, token_types: &[TokenType]) {
        while !self.is_at_end() && !self.check(token_types) {
            self.advance();
        }
    }

    /// Compte le nombre de tokens jusqu'au prochain token du type spécifié
    pub fn distance_to(&self, token_type: &TokenType) -> Option<usize> {
        let mut distance = 0;
        let mut index = self.current;
        
        while index < self.tokens.len() {
            if let Some(token) = self.tokens.get(index) {
                if std::mem::discriminant(&token.token_type) == std::mem::discriminant(token_type) {
                    return Some(distance);
                }
            }
            distance += 1;
            index += 1;
        }
        
        None
    }

    /// Vérifie si on peut parser en toute sécurité (au moins N tokens disponibles)
    pub fn can_parse(&self, required_tokens: usize) -> bool {
        self.current + required_tokens <= self.tokens.len()
    }

    /// Obtient une tranche de tokens pour lookahead
    pub fn peek_range(&self, count: usize) -> Vec<&Token> {
        let start = self.current;
        let end = std::cmp::min(start + count, self.tokens.len());
        self.tokens[start..end].iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex::{Lexer, SyntaxMode};
    use crate::tok::{Keywords, Delimiters};

    fn create_parser(code: &str) -> Parser {
        let mut lexer = Lexer::new(code, SyntaxMode::Braces);
        let tokens = lexer.tokenize();
        Parser::new(tokens, SyntaxMode::Braces)
    }

    #[test]
    fn test_advance_and_current() {
        let mut parser = create_parser("let x = 5;");
        
        assert!(parser.check(&[TokenType::KEYWORD(Keywords::LET)]));
        parser.advance();
        
        // Maintenant on devrait être sur 'x'
        if let Some(token) = parser.current_token() {
            assert!(matches!(token.token_type, TokenType::IDENTIFIER { .. }));
        }
    }

    #[test]
    fn test_consume_identifier() {
        let mut parser = create_parser("myVariable");
        let result = parser.consume_identifier();
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "myVariable");
    }

    #[test]
    fn test_match_token() {
        let mut parser = create_parser("let x");
        
        assert!(parser.match_token(&[TokenType::KEYWORD(Keywords::LET)]));
        assert!(!parser.match_token(&[TokenType::KEYWORD(Keywords::LET)])); // Already consumed
    }

    #[test]
    fn test_is_at_end() {
        let mut parser = create_parser("x");
        
        assert!(!parser.is_at_end());
        parser.advance();
        parser.advance(); // Advance past the identifier and EOF
        assert!(parser.is_at_end());
    }

    #[test]
    fn test_save_restore_position() {
        let mut parser = create_parser("let x = 5");
        
        let saved = parser.save_position();
        parser.advance();
        parser.advance();
        
        parser.restore_position(saved);
        assert!(parser.check(&[TokenType::KEYWORD(Keywords::LET)]));
    }
}