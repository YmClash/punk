// src/parser/declarations/variables.rs
// Module pour le parsing des déclarations de variables et constantes

use crate::parser::ast::{
    ASTNode, Declaration, VariableDeclaration, ConstDeclaration, 
    Visibility, Mutability, Type
};
use crate::parser::parser::Parser;
use crate::parser::parser_error::{ParserError, ParserErrorType};
use crate::tok::{TokenType, Keywords, Delimiters, Operators};

impl Parser {
    /// Parse une déclaration de variable (let ou let mut)
    pub fn parse_variable_declaration(&mut self) -> Result<ASTNode, ParserError> {
        log::debug!("Début du parsing de la déclaration de variable");

        self.consume(TokenType::KEYWORD(Keywords::LET))?;

        let mutability = self.parse_mutability();

        let name = self.consume_identifier()?;
        log::debug!("Nom de la variable parsé : {}", name);

        let variable_type = if self.match_token(&[TokenType::DELIMITER(Delimiters::COLON)]) {
            self.parse_type()?
        } else {
            Type::Infer
        };

        log::debug!("Type de la variable parsé : {:?}", variable_type);

        log::debug!("Début de la valeur de la variable");
        self.consume(TokenType::OPERATOR(Operators::EQUAL))?;

        let value = self.parse_expression(0)?;

        // Inférer le type si nécessaire
        let final_type = self.parse_inference_type(&variable_type, &value)?;

        self.consume_seperator();
        log::debug!("Valeur de la variable parsée : {:?}", value);

        Ok(ASTNode::Declaration(Declaration::Variable(VariableDeclaration {
            name,
            variable_type: Some(final_type),
            value: Some(value),
            mutability,
        })))
    }

    /// Parse une déclaration de constante
    pub fn parse_const_declaration(&mut self, visibility: Visibility) -> Result<ASTNode, ParserError> {
        log::debug!("Début du parsing de la déclaration de constante");

        self.consume(TokenType::KEYWORD(Keywords::CONST))?;

        let name = self.consume_identifier()?;

        let variable_type = if self.match_token(&[TokenType::DELIMITER(Delimiters::COLON)]) {
            self.parse_type()?
        } else {
            Type::Infer
        };

        self.consume(TokenType::OPERATOR(Operators::EQUAL))?;
        let value = self.parse_expression(0)?;

        // Inférer le type si nécessaire
        let final_type = self.parse_inference_type(&variable_type, &value)?;

        self.consume_seperator();

        log::debug!("Valeur de la constante parsée : {:?}", value);

        Ok(ASTNode::Declaration(Declaration::Constante(ConstDeclaration {
            name,
            constant_type: Some(final_type),
            value,
            visibility,
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex::{Lexer, SyntaxMode};

    fn create_parser(code: &str) -> Parser {
        let mut lexer = Lexer::new(code, SyntaxMode::Braces);
        let tokens = lexer.tokenize();
        Parser::new(tokens, SyntaxMode::Braces)
    }

    #[test]
    fn test_parse_simple_variable() {
        let mut parser = create_parser("let x = 5;");
        let result = parser.parse_variable_declaration();
        assert!(result.is_ok());
        
        if let Ok(ASTNode::Declaration(Declaration::Variable(var))) = result {
            assert_eq!(var.name, "x");
            assert_eq!(var.mutability, Mutability::Immutable);
        } else {
            panic!("Expected variable declaration");
        }
    }

    #[test]
    fn test_parse_mutable_variable() {
        let mut parser = create_parser("let mut y: int = 10;");
        let result = parser.parse_variable_declaration();
        assert!(result.is_ok());
        
        if let Ok(ASTNode::Declaration(Declaration::Variable(var))) = result {
            assert_eq!(var.name, "y");
            assert_eq!(var.mutability, Mutability::Mutable);
        } else {
            panic!("Expected mutable variable declaration");
        }
    }

    #[test]
    fn test_parse_const_declaration() {
        let mut parser = create_parser("const PI = 3.14159;");
        let result = parser.parse_const_declaration(Visibility::Public);
        assert!(result.is_ok());
        
        if let Ok(ASTNode::Declaration(Declaration::Constante(const_decl))) = result {
            assert_eq!(const_decl.name, "PI");
            assert_eq!(const_decl.visibility, Visibility::Public);
        } else {
            panic!("Expected const declaration");
        }
    }

    #[test]
    fn test_parse_typed_variable() {
        let mut parser = create_parser("let name: str = \"PunkLang\";");
        let result = parser.parse_variable_declaration();
        assert!(result.is_ok());
        
        if let Ok(ASTNode::Declaration(Declaration::Variable(var))) = result {
            assert_eq!(var.name, "name");
            assert!(var.variable_type.is_some());
        } else {
            panic!("Expected typed variable declaration");
        }
    }
}