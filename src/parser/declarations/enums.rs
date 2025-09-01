// src/parser/declarations/enums.rs
// Module pour le parsing des déclarations d'énumérations

use crate::parser::ast::{
    ASTNode, Declaration, EnumDeclaration, EnumVariant,
    Visibility, Type
};
use crate::parser::parser::Parser;
use crate::parser::parser_error::{ParserError, ParserErrorType};
use crate::tok::{TokenType, Keywords, Delimiters};
use crate::SyntaxMode;

impl Parser {
    /// Parse une déclaration d'énumération
    pub fn parse_enum_declaration(&mut self, visibility: Visibility) -> Result<ASTNode, ParserError> {
        log::debug!("Début du parsing de la déclaration d'énumération");
        
        self.consume(TokenType::KEYWORD(Keywords::ENUM))?;
        let name = self.consume_identifier()?;
        log::debug!("Nom de l'énumération parsé : {}", name);
        
        match self.syntax_mode {
            SyntaxMode::Braces => {
                self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?;
                let variantes = self.parse_enum_variantes()?;
                self.consume(TokenType::DELIMITER(Delimiters::RCURBRACE))?;
                
                Ok(ASTNode::Declaration(Declaration::Enum(EnumDeclaration {
                    name,
                    variantes,
                    visibility,
                })))
            }
            SyntaxMode::Indentation => {
                self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
                self.consume(TokenType::NEWLINE)?;
                self.consume(TokenType::INDENT)?;
                
                let variantes = self.parse_enum_variantes_indented()?;
                
                self.consume(TokenType::DEDENT)?;
                
                Ok(ASTNode::Declaration(Declaration::Enum(EnumDeclaration {
                    name,
                    variantes,
                    visibility,
                })))
            }
        }
    }

    /// Parse les variantes d'une énumération en mode accolades
    pub fn parse_enum_variantes(&mut self) -> Result<Vec<EnumVariant>, ParserError> {
        log::debug!("Début du parsing des variantes d'énumération");
        
        let mut variantes = Vec::new();
        
        // Enum vide
        if self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]) {
            return Ok(variantes);
        }
        
        loop {
            let variante = self.parse_enum_variant_fields()?;
            variantes.push(variante);
            
            if self.match_token(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
                // Permettre une nouvelle ligne optionnelle après la virgule
                let _ = self.match_token(&[TokenType::NEWLINE]);
                
                // Si on trouve l'accolade fermante après la virgule, on termine
                if self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]) {
                    break;
                }
            } else if self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]) {
                break;
            } else {
                return Err(ParserError::new(
                    ParserErrorType::ExpectedCommaOrCloseBrace,
                    self.current_position()
                ));
            }
        }
        
        log::debug!("Variantes d'énumération parsées : {:?}", variantes);
        Ok(variantes)
    }

    /// Parse les variantes d'une énumération en mode indentation
    fn parse_enum_variantes_indented(&mut self) -> Result<Vec<EnumVariant>, ParserError> {
        log::debug!("Début du parsing des variantes d'énumération (mode indentation)");
        
        let mut variantes = Vec::new();
        
        while !self.check(&[TokenType::DEDENT]) && !self.is_at_end() {
            // Skip les lignes vides
            if self.check(&[TokenType::NEWLINE]) {
                self.advance();
                continue;
            }
            
            let variante = self.parse_enum_variant_fields()?;
            variantes.push(variante);
            
            // Consommer le newline si présent
            if self.check(&[TokenType::NEWLINE]) && !self.check(&[TokenType::DEDENT]) {
                self.advance();
            }
        }
        
        Ok(variantes)
    }

    /// Parse un champ de variante d'énumération
    pub fn parse_enum_variant_fields(&mut self) -> Result<EnumVariant, ParserError> {
        let visibility = self.parse_visibility().unwrap_or(Visibility::Private);
        log::debug!("Visibilité de la variante parsée : {:?}", visibility);
        
        let name = self.consume_identifier()?;
        log::debug!("Nom de la variante parsée : {}", name);
        
        // Les variantes peuvent avoir un type optionnel
        let variante_type = if self.match_token(&[TokenType::DELIMITER(Delimiters::COLON)]) {
            self.parse_type()?
        } else {
            // Type par défaut pour les variantes simples  
            Type::Custom("()".to_string()) // Utilise "()" pour représenter unit
        };
        
        log::debug!("Type de la variante parsée : {:?}", variante_type);
        
        Ok(EnumVariant {
            name,
            variante_type,
            visibility,
        })
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
    fn test_simple_enum() {
        let code = r#"
enum Color {
    Red,
    Green,
    Blue
}"#;
        let mut parser = create_parser(code, SyntaxMode::Braces);
        let result = parser.parse_enum_declaration(Visibility::Public);
        
        assert!(result.is_ok());
        
        if let Ok(ASTNode::Declaration(Declaration::Enum(enum_decl))) = result {
            assert_eq!(enum_decl.name, "Color");
            assert_eq!(enum_decl.variantes.len(), 3);
            assert_eq!(enum_decl.variantes[0].name, "Red");
            assert_eq!(enum_decl.variantes[1].name, "Green");
            assert_eq!(enum_decl.variantes[2].name, "Blue");
        } else {
            panic!("Expected enum declaration");
        }
    }

    #[test]
    fn test_enum_with_types() {
        let code = r#"
enum Option {
    Some: T,
    None
}"#;
        let mut parser = create_parser(code, SyntaxMode::Braces);
        let result = parser.parse_enum_declaration(Visibility::Private);
        
        if result.is_ok() {
            if let Ok(ASTNode::Declaration(Declaration::Enum(enum_decl))) = result {
                assert_eq!(enum_decl.name, "Option");
                assert_eq!(enum_decl.variantes.len(), 2);
                assert_eq!(enum_decl.variantes[0].name, "Some");
                assert_eq!(enum_decl.variantes[1].name, "None");
            }
        }
    }

    #[test]
    fn test_empty_enum() {
        let code = "enum Empty {}";
        let mut parser = create_parser(code, SyntaxMode::Braces);
        let result = parser.parse_enum_declaration(Visibility::Public);
        
        assert!(result.is_ok());
        
        if let Ok(ASTNode::Declaration(Declaration::Enum(enum_decl))) = result {
            assert_eq!(enum_decl.name, "Empty");
            assert_eq!(enum_decl.variantes.len(), 0);
        } else {
            panic!("Expected empty enum declaration");
        }
    }
}