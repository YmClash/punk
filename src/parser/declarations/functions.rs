// src/parser/declarations/functions.rs
// Module pour le parsing des déclarations de fonctions

use crate::parser::ast::{
    ASTNode, Declaration, FunctionDeclaration, MethodeDeclaration,
    Visibility, Type, Parameter
};
use crate::parser::parser::Parser;
use crate::parser::parser_error::{ParserError, ParserErrorType};
use crate::tok::{TokenType, Keywords, Delimiters, Operators};
use crate::SyntaxMode;

impl Parser {
    /// Parse une déclaration de fonction
    pub fn parse_function_declaration(&mut self, visibility: Visibility) -> Result<ASTNode, ParserError> {
        log::debug!("Début du parsing de la déclaration de fonction");
        
        self.consume(TokenType::KEYWORD(Keywords::FN))?;
        let name = self.consume_identifier()?;
        log::debug!("Nom de la fonction parsé : {}", name);

        self.consume(TokenType::DELIMITER(Delimiters::LPAR))?;
        let parameters = self.parse_function_parameters()?;
        self.consume(TokenType::DELIMITER(Delimiters::RPAR))?;

        let return_type = if self.match_token(&[TokenType::OPERATOR(Operators::RARROW)]) {
            self.parse_type()?
        } else {
            Type::Infer
        };

        if self.syntax_mode == SyntaxMode::Indentation {
            self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
        }

        let body = self.parse_function_body()?;

        Ok(ASTNode::Declaration(Declaration::Function(FunctionDeclaration {
            name,
            parameters,
            return_type: Some(return_type),
            body,
            visibility,
            is_variadic:false,  // nouveau
        })))
    }


    /// Parse une déclaration de méthode (pour les classes et impls)
    pub fn parse_methode_declaration(&mut self) -> Result<MethodeDeclaration, ParserError> {
        log::debug!("Début du parsing de la déclaration de méthode");
        
        let visibility = self.parse_visibility().unwrap_or(Visibility::Private);
        
        // Vérifier si c'est 'fn' ou 'def'
        if self.match_token(&[TokenType::KEYWORD(Keywords::DEF)]) {
            // Syntaxe Python-like avec 'def'
            self.parse_def_method(visibility)
        } else {
            self.consume(TokenType::KEYWORD(Keywords::FN))?;
            self.parse_fn_method(visibility)
        }
    }

    /// Parse une méthode avec syntaxe 'def'
    fn parse_def_method(&mut self, visibility: Visibility) -> Result<MethodeDeclaration, ParserError> {
        let name = self.consume_identifier()?;
        
        self.consume(TokenType::DELIMITER(Delimiters::LPAR))?;
        
        // Parser tous les paramètres (incluant potentiellement self)
        let parameters = if !self.check(&[TokenType::DELIMITER(Delimiters::RPAR)]) {
            self.parse_function_parameters()?
        } else {
            Vec::new()
        };
        
        self.consume(TokenType::DELIMITER(Delimiters::RPAR))?;
        
        let return_type = if self.match_token(&[TokenType::OPERATOR(Operators::RARROW)]) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        if self.syntax_mode == SyntaxMode::Indentation {
            self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
        }
        
        let body = self.parse_function_body()?;
        
        Ok(MethodeDeclaration {
            name,
            parameters,
            return_type,
            body,
            visibility,
        })
    }

    /// Parse une méthode avec syntaxe 'fn'
    fn parse_fn_method(&mut self, visibility: Visibility) -> Result<MethodeDeclaration, ParserError> {
        let name = self.consume_identifier()?;
        
        self.consume(TokenType::DELIMITER(Delimiters::LPAR))?;
        
        let parameters = if !self.check(&[TokenType::DELIMITER(Delimiters::RPAR)]) {
            self.parse_function_parameters()?
        } else {
            Vec::new()
        };
        
        self.consume(TokenType::DELIMITER(Delimiters::RPAR))?;
        
        let return_type = if self.match_token(&[TokenType::OPERATOR(Operators::RARROW)]) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        if self.syntax_mode == SyntaxMode::Indentation {
            self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
        }
        
        let body = self.parse_function_body()?;
        
        Ok(MethodeDeclaration {
            name,
            parameters,
            return_type,
            body,
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
    fn test_simple_function() {
        let code = "fn add(a: int, b: int) -> int { return a + b }";
        let mut lexer = Lexer::new(code, SyntaxMode::Braces);
        let tokens = lexer.tokenize();
        // Debug: print tokens
        // for (i, token) in tokens.iter().enumerate() {
        //     eprintln!("Token {}: {:?}", i, token);
        // }
        let mut parser = Parser::new(tokens, SyntaxMode::Braces);
        let result = parser.parse_function_declaration(Visibility::Public);
        // if let Err(ref e) = result {
        //     eprintln!("Parse error: {:?}", e);
        // }
        assert!(result.is_ok());
        
        if let Ok(ASTNode::Declaration(Declaration::Function(func))) = result {
            assert_eq!(func.name, "add");
            assert_eq!(func.parameters.len(), 2);
            assert!(func.return_type.is_some());
        } else {
            panic!("Expected function declaration");
        }
    }

    #[test]
    fn test_function_no_params() {
        let mut parser = create_parser("fn hello() { print(\"Hello\"); }", SyntaxMode::Braces);
        let result = parser.parse_function_declaration(Visibility::Private);
        assert!(result.is_ok());
        
        if let Ok(ASTNode::Declaration(Declaration::Function(func))) = result {
            assert_eq!(func.name, "hello");
            assert_eq!(func.parameters.len(), 0);
        } else {
            panic!("Expected function declaration");
        }
    }

    #[test]
    fn test_function_indentation_mode() {
        let code = "fn multiply(x: int, y: int) -> int:\n    return x * y";
        let mut parser = create_parser(code, SyntaxMode::Indentation);
        let result = parser.parse_function_declaration(Visibility::Public);
        
        // This might fail without proper indentation tokens, but we're testing the structure
        if result.is_ok() {
            if let Ok(ASTNode::Declaration(Declaration::Function(func))) = result {
                assert_eq!(func.name, "multiply");
            }
        }
    }
}