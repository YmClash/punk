// src/parser/declarations/structs.rs
// Module pour le parsing des déclarations de structures

use crate::parser::ast::{
    ASTNode, Declaration, StructDeclaration, Field,
    Visibility, Type
};
use crate::parser::parser::Parser;
use crate::parser::parser_error::{ParserError, ParserErrorType};
use crate::tok::{TokenType, Keywords, Delimiters, Operators};
use crate::SyntaxMode;

impl Parser {
    /// Parse une déclaration de structure
    pub fn parse_struct_declaration(&mut self, visibility: Visibility) -> Result<ASTNode, ParserError> {
        log::debug!("Début du parsing de la déclaration de structure");

        self.consume(TokenType::KEYWORD(Keywords::STRUCT))?;
        let name = self.consume_identifier()?;
        log::debug!("Nom de la structure parsé : {}", name);

        // TODO: Implémenter les types génériques
        // let generic_types = if self.match_token(&[TokenType::OPERATOR(Operators::LESS)]) {
        //     self.parse_generic_type_params()?
        // } else {
        //     vec![]
        // };

        match self.syntax_mode {
            SyntaxMode::Braces => {
                self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?;
                let fields = self.parse_struct_fields()?;
                self.consume(TokenType::DELIMITER(Delimiters::RCURBRACE))?;
                
                self.consume_seperator();
                
                Ok(ASTNode::Declaration(Declaration::Structure(StructDeclaration {
                    name,
                    fields,
                    visibility,
                })))
            }
            SyntaxMode::Indentation => {
                self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
                self.consume(TokenType::NEWLINE)?;
                self.consume(TokenType::INDENT)?;
                
                let fields = self.parse_struct_fields_indented()?;
                
                self.consume(TokenType::DEDENT)?;
                
                Ok(ASTNode::Declaration(Declaration::Structure(StructDeclaration {
                    name,
                    fields,
                    visibility,
                })))
            }
        }
    }

    /// Parse les champs d'une structure en mode accolades
    pub fn parse_struct_fields(&mut self) -> Result<Vec<Field>, ParserError> {
        log::debug!("Début du parsing des champs de structure");
        
        let mut fields = Vec::new();
        
        while !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]) && !self.is_at_end() {
            let visibility = self.parse_visibility().unwrap_or(Visibility::Private);
            let field_name = self.consume_identifier()?;
            
            self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
            let field_type = self.parse_type()?;
            
            fields.push(Field {
                name: field_name,
                field_type,
                visibility,
            });
            
            // Gérer la virgule optionnelle
            if !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]) {
                if self.check(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
                    self.advance();
                } else if !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]) {
                    return Err(ParserError::new(
                        ParserErrorType::ExpectedCommaOrCloseBrace,
                        self.current_position()
                    ));
                }
            }
        }
        
        Ok(fields)
    }

    /// Parse les champs d'une structure en mode indentation
    fn parse_struct_fields_indented(&mut self) -> Result<Vec<Field>, ParserError> {
        log::debug!("Début du parsing des champs de structure (mode indentation)");
        
        let mut fields = Vec::new();
        
        while !self.check(&[TokenType::DEDENT]) && !self.is_at_end() {
            // Skip les lignes vides
            if self.check(&[TokenType::NEWLINE]) {
                self.advance();
                continue;
            }
            
            let visibility = self.parse_visibility().unwrap_or(Visibility::Private);
            let field_name = self.consume_identifier()?;
            
            self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
            let field_type = self.parse_type()?;
            
            fields.push(Field {
                name: field_name,
                field_type,
                visibility,
            });
            
            // Consommer le newline si présent
            if self.check(&[TokenType::NEWLINE]) && !self.check(&[TokenType::DEDENT]) {
                self.advance();
            }
        }
        
        Ok(fields)
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
    fn test_simple_struct() {
        let code = "struct Point { x: float, y: float }";
        let mut parser = create_parser(code, SyntaxMode::Braces);
        let result = parser.parse_struct_declaration(Visibility::Public);
        
        assert!(result.is_ok());
        
        if let Ok(ASTNode::Declaration(Declaration::Structure(struct_decl))) = result {
            assert_eq!(struct_decl.name, "Point");
            assert_eq!(struct_decl.fields.len(), 2);
            assert_eq!(struct_decl.fields[0].name, "x");
            assert_eq!(struct_decl.fields[1].name, "y");
        } else {
            panic!("Expected struct declaration");
        }
    }

    #[test]
    fn test_struct_with_visibility() {
        let code = "struct Person { pub name: str, age: int }";
        let mut parser = create_parser(code, SyntaxMode::Braces);
        let result = parser.parse_struct_declaration(Visibility::Public);
        
        assert!(result.is_ok());
        
        if let Ok(ASTNode::Declaration(Declaration::Structure(struct_decl))) = result {
            assert_eq!(struct_decl.name, "Person");
            assert_eq!(struct_decl.fields.len(), 2);
            // First field should be public
            assert_eq!(struct_decl.fields[0].visibility, Visibility::Public);
            // Second field should be private (default)
            assert_eq!(struct_decl.fields[1].visibility, Visibility::Private);
        } else {
            panic!("Expected struct declaration with visibility");
        }
    }

    #[test]
    fn test_empty_struct() {
        let code = "struct Empty {}";
        let mut parser = create_parser(code, SyntaxMode::Braces);
        let result = parser.parse_struct_declaration(Visibility::Private);
        
        assert!(result.is_ok());
        
        if let Ok(ASTNode::Declaration(Declaration::Structure(struct_decl))) = result {
            assert_eq!(struct_decl.name, "Empty");
            assert_eq!(struct_decl.fields.len(), 0);
        } else {
            panic!("Expected empty struct declaration");
        }
    }
}