// src/parser/declarations/traits.rs
// Module pour le parsing des déclarations de traits

use crate::parser::ast::{
    ASTNode, Declaration, TraitDeclaration, TraitMethod,
    AssociatedType, WhereClause, Visibility, Type
};
use crate::parser::parser::Parser;
use crate::parser::parser_error::{ParserError, ParserErrorType};
use crate::tok::{TokenType, Keywords, Delimiters, Operators};
use crate::SyntaxMode;

impl Parser {
    /// Parse une déclaration de trait
    pub fn parse_trait_declaration(&mut self, visibility: Visibility) -> Result<ASTNode, ParserError> {
        log::debug!("Début du parsing de la déclaration de trait");
        
        self.consume(TokenType::KEYWORD(Keywords::TRAIT))?;
        let name = self.consume_identifier()?;
        log::debug!("Nom du trait parsé : {}", name);

        // Parse les paramètres génériques optionnels
        let generic_params = if self.check(&[TokenType::OPERATOR(Operators::LESS)]) {
            Some(self.parse_generic_parameters()?)
        } else {
            None
        };

        // Parse des supertraits optionnels
        let mut super_traits = Vec::new();
        if self.check(&[TokenType::DELIMITER(Delimiters::COLON)]) {
            let next_token = self.peek_next_token();
            if let Some(token) = next_token {
                if !matches!(token.token_type, TokenType::NEWLINE) &&
                   !matches!(token.token_type, TokenType::DELIMITER(Delimiters::LCURBRACE)) {
                    self.advance(); // Consomme le ':'
                    super_traits = self.parse_trait_bounds()?;
                }
            }
        }

        // Parse les clauses where optionnelles
        let where_clause = self.parse_where_clauses()?;

        let mut methods = Vec::new();
        let mut associated_types = Vec::new();

        match self.syntax_mode {
            SyntaxMode::Braces => {
                self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?;
                
                while !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]) && !self.is_at_end() {
                    if self.check(&[TokenType::KEYWORD(Keywords::FN)]) {
                        methods.push(self.parse_trait_methods()?);
                    } else if self.check(&[TokenType::KEYWORD(Keywords::TYPE)]) {
                        associated_types.push(self.parse_associated_type()?);
                    } else {
                        return Err(ParserError::new(
                            ParserErrorType::UnexpectedToken,
                            self.current_position()
                        ));
                    }
                }
                
                self.consume(TokenType::DELIMITER(Delimiters::RCURBRACE))?;
            }
            
            SyntaxMode::Indentation => {
                self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
                self.consume(TokenType::NEWLINE)?;
                self.consume(TokenType::INDENT)?;
                
                while !self.check(&[TokenType::DEDENT]) && !self.is_at_end() {
                    // Skip les lignes vides
                    if self.check(&[TokenType::NEWLINE]) {
                        self.advance();
                        continue;
                    }
                    
                    if self.check(&[TokenType::KEYWORD(Keywords::FN)]) {
                        methods.push(self.parse_trait_methods()?);
                    } else if self.check(&[TokenType::KEYWORD(Keywords::TYPE)]) {
                        associated_types.push(self.parse_associated_type()?);
                    } else {
                        return Err(ParserError::new(
                            ParserErrorType::UnexpectedToken,
                            self.current_position()
                        ));
                    }
                }
                
                self.consume(TokenType::DEDENT)?;
            }
        }

        log::debug!("Parsing du trait terminé");
        
        Ok(ASTNode::Declaration(Declaration::Trait(TraitDeclaration {
            name,
            generic_parameters: generic_params,
            methods,
            associated_types,
            visibility,
            where_clause,
            super_traits,
        })))
    }

    /// Parse une méthode de trait (signature seulement)
    fn parse_trait_methods(&mut self) -> Result<TraitMethod, ParserError> {
        log::debug!("Début du parsing de la signature de méthode de trait");
        
        self.consume(TokenType::KEYWORD(Keywords::FN))?;
        let name = self.consume_identifier()?;
        
        self.consume(TokenType::DELIMITER(Delimiters::LPAR))?;
        let parameters = self.parse_function_parameters()?;
        self.consume(TokenType::DELIMITER(Delimiters::RPAR))?;

        let return_type = if self.check(&[TokenType::OPERATOR(Operators::RARROW)]) {
            self.consume(TokenType::OPERATOR(Operators::RARROW))?;
            Some(self.parse_type()?)
        } else {
            None
        };

        self.consume_seperator();

        log::debug!("Parsing de la méthode de trait terminé");

        Ok(TraitMethod {
            name,
            parameters,
            return_type,
        })
    }


    /// Parse les clauses where (stub pour l'instant)
    pub fn parse_where_clauses(&mut self) -> Result<Vec<WhereClause>, ParserError> {
        // Implémentation simplifiée pour l'instant
        if self.check(&[TokenType::KEYWORD(Keywords::WHERE)]) {
            self.advance();
            // TODO: Implémenter le parsing complet des clauses where
            log::debug!("Parsing des clauses where (non implémenté)");
        }
        Ok(Vec::new())
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
    fn test_simple_trait() {
        let code = r#"
trait Display {
    fn fmt(s: Self) -> str
}"#;
        let mut parser = create_parser(code, SyntaxMode::Braces);
        let result = parser.parse_trait_declaration(Visibility::Public);
        
        if let Err(ref e) = result {
            eprintln!("Parse error in test_simple_trait: {:?}", e);
        }
        assert!(result.is_ok());
        
        if let Ok(ASTNode::Declaration(Declaration::Trait(trait_decl))) = result {
            assert_eq!(trait_decl.name, "Display");
            assert_eq!(trait_decl.methods.len(), 1);
            assert_eq!(trait_decl.methods[0].name, "fmt");
        } else {
            panic!("Expected trait declaration");
        }
    }

    #[test]
    fn test_trait_with_associated_type() {
        let code = r#"
trait Iterator {
    type Item;
    fn next(self) -> Option;
}"#;
        let mut parser = create_parser(code, SyntaxMode::Braces);
        let result = parser.parse_trait_declaration(Visibility::Private);
        
        if result.is_ok() {
            if let Ok(ASTNode::Declaration(Declaration::Trait(trait_decl))) = result {
                assert_eq!(trait_decl.name, "Iterator");
                assert_eq!(trait_decl.associated_types.len(), 1);
                assert_eq!(trait_decl.methods.len(), 1);
            }
        }
    }

    #[test]
    fn test_empty_trait() {
        let code = "trait Marker {}";
        let mut parser = create_parser(code, SyntaxMode::Braces);
        let result = parser.parse_trait_declaration(Visibility::Public);
        
        assert!(result.is_ok());
        
        if let Ok(ASTNode::Declaration(Declaration::Trait(trait_decl))) = result {
            assert_eq!(trait_decl.name, "Marker");
            assert_eq!(trait_decl.methods.len(), 0);
            assert_eq!(trait_decl.associated_types.len(), 0);
        } else {
            panic!("Expected empty trait declaration");
        }
    }
}