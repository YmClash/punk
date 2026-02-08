// src/parser/unified_block.rs
// Module unifié pour le parsing des blocs dans les deux modes syntaxiques

use crate::parser::ast::ASTNode;
use crate::parser::parser::Parser;
use crate::parser::parser_error::{ParserError, ParserErrorType};
use crate::tok::{Delimiters, TokenType};
use crate::SyntaxMode;

impl Parser {
    /// Parse un bloc de code unifié qui fonctionne dans les deux modes
    /// Cette méthode remplace parse_block(), parse_body_block() et parse_block_expression()
    pub fn parse_unified_block(&mut self) -> Result<Vec<ASTNode>, ParserError> {
        match self.syntax_mode {
            SyntaxMode::Indentation => self.parse_unified_indented_block(),
            SyntaxMode::Braces => self.parse_unified_braced_block(),
        }
    }

    /// Parse un bloc en mode indentation (Python-like)
    fn parse_unified_indented_block(&mut self) -> Result<Vec<ASTNode>, ParserError> {
        log::debug!("Parsing unified indented block");
        
        // Consommer le ':' qui précède le bloc indenté
        self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
        
        // Gérer le newline et l'indentation
        if self.check(&[TokenType::NEWLINE]) {
            self.consume(TokenType::NEWLINE)?;
        }
        
        // Vérifier et consommer INDENT
        if !self.check(&[TokenType::INDENT]) {
            // Cas spécial : bloc vide ou single-line statement
            if self.is_at_end() || self.check(&[TokenType::DEDENT, TokenType::EOF]) {
                return Ok(Vec::new());
            }
            
            // Single-line statement après ':'
            let stmt = self.parse_statement()?;
            return Ok(vec![stmt]);
        }
        
        self.consume(TokenType::INDENT)?;
        
        // Parser les statements du bloc
        let mut statements = Vec::new();
        while !self.check(&[TokenType::DEDENT, TokenType::EOF]) {
            // Skip les lignes vides
            if self.check(&[TokenType::NEWLINE]) {
                self.advance();
                continue;
            }
            
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            
            // Gérer les newlines entre statements
            if self.check(&[TokenType::NEWLINE]) && !self.check(&[TokenType::DEDENT, TokenType::EOF]) {
                self.advance();
            }
        }
        
        // Consommer DEDENT
        if self.check(&[TokenType::DEDENT]) {
            self.consume(TokenType::DEDENT)?;
        }
        log::debug!("Parsed indented block with {} statements", statements.len());
        Ok(statements)
    }

    /// Parse un bloc en mode accolades (C,Rust-like)
    fn parse_unified_braced_block(&mut self) -> Result<Vec<ASTNode>, ParserError> {
        log::debug!("Parsing unified braced block");
        
        // Consommer l'accolade ouvrante
        self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?;
        
        let mut statements = Vec::new();
        
        // Parser les statements jusqu'à l'accolade fermante
        while !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE), TokenType::EOF]) {
            // Skip les points-virgules ou virgules superflus
            while self.check(&[
                TokenType::DELIMITER(Delimiters::SEMICOLON),
                TokenType::DELIMITER(Delimiters::COMMA)
            ]) {
                self.advance();
            }
            
            // Vérifier si on a atteint la fin du bloc
            if self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE), TokenType::EOF]) {
                break;
            }
            
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            
            // Gérer les séparateurs optionnels
            // Note: Les virgules sont acceptées dans certains contextes (structs, enums)
            if self.check(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
                self.advance();
            }
        }
        
        // Consommer l'accolade fermante
        self.consume(TokenType::DELIMITER(Delimiters::RCURBRACE))?;

        log::debug!("Parsed braced block with {} statements", statements.len());
        
        Ok(statements)
    }

    /// Parse un bloc qui peut commencer soit par ':' (indentation) soit par '{' (braces)
    /// Utile pour les fonctions, classes, etc. qui doivent supporter les deux syntaxes
    pub fn parse_adaptive_block(&mut self) -> Result<Vec<ASTNode>, ParserError> {
        match self.syntax_mode {
            SyntaxMode::Indentation => {
                // En mode indentation, on attend ':'
                if !self.check(&[TokenType::DELIMITER(Delimiters::COLON)]) {
                    return Err(ParserError::new(
                        ParserErrorType::ExpectColon,
                        self.current_position(),
                    ));
                }
                self.parse_unified_block()
            }
            SyntaxMode::Braces => {
                // En mode braces, on attend '{'
                if !self.check(&[TokenType::DELIMITER(Delimiters::LCURBRACE)]) {
                    return Err(ParserError::new(
                        ParserErrorType::BraceError,
                        self.current_position(),
                    ));
                }
                self.parse_unified_block()
            }
        }
    }

    /// Helper pour vérifier si on peut parser un bloc
    pub fn can_parse_block(&self) -> bool {
        match self.syntax_mode {
            SyntaxMode::Indentation => {
                self.check(&[TokenType::DELIMITER(Delimiters::COLON)])
            }
            SyntaxMode::Braces => {
                self.check(&[TokenType::DELIMITER(Delimiters::LCURBRACE)])
            }
        }
    }

    /// Parse un bloc pour une expression lambda ou une closure
    /// Similaire à parse_unified_block mais avec des règles légèrement différentes
    pub fn parse_lambda_block(&mut self) -> Result<Vec<ASTNode>, ParserError> {
        log::debug!("Parsing lambda block");
        
        match self.syntax_mode {
            SyntaxMode::Braces => {
                // Les lambdas en mode braces utilisent toujours {}
                self.parse_unified_braced_block()
            }
            SyntaxMode::Indentation => {
                // Les lambdas en mode indentation peuvent être sur une ligne
                if self.check(&[TokenType::DELIMITER(Delimiters::COLON)]) {
                    self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
                    
                    // Vérifier si c'est une lambda single-line
                    if !self.check(&[TokenType::NEWLINE]) {
                        // Expression simple sur la même ligne
                        let expr = self.parse_expression(0)?;
                        return Ok(vec![ASTNode::Expression(expr)]);
                    }
                    
                    // Sinon, parser comme un bloc normal
                    self.parse_unified_indented_block()
                } else {
                    // Lambda sans ':' - juste une expression
                    let expr = self.parse_expression(0)?;
                    Ok(vec![ASTNode::Expression(expr)])
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex::Lexer;

    #[test]
    fn test_unified_block_braces() {
        let code = "{ let x = 5; let y = 10; }";
        let mut lexer = Lexer::new(code, SyntaxMode::Braces);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens, SyntaxMode::Braces);
        
        let result = parser.parse_unified_block();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_unified_block_indentation() {
        let code = ":\n    let x = 5\n    let y = 10";
        let mut lexer = Lexer::new(code, SyntaxMode::Indentation);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens, SyntaxMode::Indentation);
        
        let result = parser.parse_unified_block();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_empty_block_braces() {
        let code = "{}";
        let mut lexer = Lexer::new(code, SyntaxMode::Braces);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens, SyntaxMode::Braces);
        
        let result = parser.parse_unified_block();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_single_line_indentation() {
        let code = ": return 42";
        let mut lexer = Lexer::new(code, SyntaxMode::Indentation);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens, SyntaxMode::Indentation);
        
        let result = parser.parse_unified_block();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }
}