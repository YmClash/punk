// src/parser/declarations/impl_blocks.rs
// Module pour le parsing des blocs impl

use crate::parser::ast::{
    ASTNode, Declaration, ImplDeclaration, ImplMethod, SelfKind,
    Visibility, Type, GenericParameter, WhereClause, Parameter
};
use crate::parser::parser::Parser;
use crate::parser::parser_error::{ParserError, ParserErrorType};
use crate::tok::{TokenType, Keywords, Delimiters, Operators};
use crate::SyntaxMode;

impl Parser {
    /// Parse une déclaration impl
    pub fn parse_impl_declaration(&mut self, visibility: Visibility) -> Result<ASTNode, ParserError> {
        log::debug!("Début du parsing de la déclaration impl");
        
        self.consume(TokenType::KEYWORD(Keywords::IMPL))?;
        
        // Vérifier si c'est une implémentation de trait ou une implémentation simple
        // On regarde s'il y a un trait (format: impl Trait for Type)
        let mut trait_name = None;
        let target_type_str;
        
        // Parse le premier identifiant
        let first_identifier = self.consume_identifier()?;
        
        // Parse les paramètres génériques optionnels après le premier identifiant
        let generic_params = if self.check(&[TokenType::OPERATOR(Operators::LESS)]) {
            Some(self.parse_generic_parameters()?)
        } else {
            None
        };
        
        // Vérifier si c'est une implémentation de trait (mot-clé "for")
        if self.check(&[TokenType::KEYWORD(Keywords::FOR)]) {
            self.advance(); // Consomme "for"
            trait_name = Some(first_identifier);
            target_type_str = self.consume_identifier()?;
            log::debug!("Implémentation de trait détectée : {} for {}", trait_name.as_ref().unwrap(), target_type_str);
        } else {
            target_type_str = first_identifier;
            log::debug!("Implémentation simple détectée pour : {}", target_type_str);
        }
        
        // Parse les clauses where optionnelles
        let where_clause = self.parse_where_clauses()?;
        
        // Parse le corps de l'implémentation
        let mut methods = Vec::new();
        
        match self.syntax_mode {
            SyntaxMode::Braces => {
                self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?;
                
                while !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]) && !self.is_at_end() {
                    // Skip les newlines optionnels
                    if self.check(&[TokenType::NEWLINE]) {
                        self.advance();
                        continue;
                    }
                    
                    // Parse la visibilité (pour les méthodes impl)
                    let method_visibility = self.parse_visibility().unwrap_or(Visibility::Private);
                    
                    if self.check(&[TokenType::KEYWORD(Keywords::FN)]) {
                        let method = self.parse_impl_block_method(method_visibility)?;
                        
                        methods.push(method);
                    } else if self.check(&[TokenType::KEYWORD(Keywords::CONST)]) {
                        // TODO: Parse les constantes associées
                        log::debug!("Constante associée détectée (non implémentée)");
                        self.advance(); // Skip pour l'instant
                        // Consommer jusqu'au prochain ';' ou newline
                        while !self.check(&[TokenType::DELIMITER(Delimiters::SEMICOLON), TokenType::NEWLINE]) && !self.is_at_end() {
                            self.advance();
                        }
                        if self.check(&[TokenType::DELIMITER(Delimiters::SEMICOLON)]) {
                            self.advance();
                        }
                    } else if self.check(&[TokenType::KEYWORD(Keywords::TYPE)]) {
                        // TODO: Parse les types associés dans les impls de trait
                        log::debug!("Type associé détecté (non implémenté)");
                        self.advance(); // Skip pour l'instant
                        // Consommer jusqu'au prochain ';' ou newline
                        while !self.check(&[TokenType::DELIMITER(Delimiters::SEMICOLON), TokenType::NEWLINE]) && !self.is_at_end() {
                            self.advance();
                        }
                        if self.check(&[TokenType::DELIMITER(Delimiters::SEMICOLON)]) {
                            self.advance();
                        }
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
                    
                    // Parse la visibilité
                    let method_visibility = self.parse_visibility().unwrap_or(Visibility::Private);
                    
                    if self.check(&[TokenType::KEYWORD(Keywords::FN)]) {
                        let method = self.parse_impl_block_method(method_visibility)?;
                        
                        methods.push(method);
                    } else if self.check(&[TokenType::KEYWORD(Keywords::CONST)]) {
                        log::debug!("Constante associée détectée (non implémentée)");
                        self.advance(); // Skip pour l'instant
                        // Consommer jusqu'au prochain newline
                        while !self.check(&[TokenType::NEWLINE]) && !self.is_at_end() {
                            self.advance();
                        }
                    } else if self.check(&[TokenType::KEYWORD(Keywords::TYPE)]) {
                        log::debug!("Type associé détecté (non implémenté)");
                        self.advance(); // Skip pour l'instant
                        // Consommer jusqu'au prochain newline
                        while !self.check(&[TokenType::NEWLINE]) && !self.is_at_end() {
                            self.advance();
                        }
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
        
        log::debug!("Parsing de l'implémentation terminé");
        
        // Créer le nœud AST
        Ok(ASTNode::Declaration(Declaration::Impl(ImplDeclaration {
            trait_name,
            target_type: Type::Custom(target_type_str),
            generic_parameters: generic_params,
            methods,
            where_clause,
            visibility,
        })))
    }
    
    /// Parse une méthode dans un bloc impl
    fn parse_impl_block_method(&mut self, visibility: Visibility) -> Result<ImplMethod, ParserError> {
        log::debug!("Début du parsing de la méthode impl");
        
        self.consume(TokenType::KEYWORD(Keywords::FN))?;
        let name = self.consume_identifier()?;
        
        // Parse les paramètres génériques de la méthode
        let generic_params = if self.check(&[TokenType::OPERATOR(Operators::LESS)]) {
            Some(self.parse_generic_parameters()?)
        } else {
            None
        };
        
        self.consume(TokenType::DELIMITER(Delimiters::LPAR))?;
        
        // Parse self parameter si présent
        let self_param = if self.check(&[TokenType::KEYWORD(Keywords::SELF)]) {
            self.advance();
            Some(SelfKind::Value)
        } else if self.check(&[TokenType::OPERATOR(Operators::AMPER)]) {
            self.advance();
            if self.check(&[TokenType::KEYWORD(Keywords::MUT)]) {
                self.advance();
                self.consume(TokenType::KEYWORD(Keywords::SELF))?;
                Some(SelfKind::MutableReference)
            } else {
                self.consume(TokenType::KEYWORD(Keywords::SELF))?;
                Some(SelfKind::Reference)
            }
        } else {
            None
        };
        
        // Parse les autres paramètres
        let mut parameters = Vec::new();
        if self_param.is_some() && self.check(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
            self.advance();
            parameters = self.parse_function_parameters()?;
        } else if self_param.is_none() {
            parameters = self.parse_function_parameters()?;
        }
        
        self.consume(TokenType::DELIMITER(Delimiters::RPAR))?;
        
        let return_type = if self.check(&[TokenType::OPERATOR(Operators::RARROW)]) {
            self.consume(TokenType::OPERATOR(Operators::RARROW))?;
            Some(self.parse_type()?)
        } else {
            None
        };
        
        // Parse le corps de la méthode
        let body = self.parse_function_body()?;
        
        log::debug!("Parsing de la méthode impl terminé : {}", name);
        
        Ok(ImplMethod {
            name,
            self_param,
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
    fn test_simple_impl_block() {
        let code = r#"
impl Point {
    fn new(x: int, y: int) -> Point {
        return Point(x, y)
    }

    fn distance(&self) -> float {
        return 0.0
    }
}"#;
        let mut lexer = Lexer::new(code, SyntaxMode::Braces);
        let tokens = lexer.tokenize();
        // for (i, token) in tokens.iter().enumerate() {
        //     eprintln!("Token {}: {:?}", i, token);
        // }
        let mut parser = Parser::new(tokens, SyntaxMode::Braces);
        let result = parser.parse_impl_declaration(Visibility::Public);
        
        if let Err(ref e) = result {
            eprintln!("Parse error in test_simple_impl_block: {:?}", e);
        }
        assert!(result.is_ok());
        
        if let Ok(ASTNode::Declaration(Declaration::Impl(impl_decl))) = result {
            if let Type::Custom(type_name) = &impl_decl.target_type {
                assert_eq!(type_name, "Point");
            }
            assert_eq!(impl_decl.methods.len(), 2); // new et distance
        } else {
            panic!("Expected impl declaration");
        }
    }

    #[test]
    fn test_trait_impl() {
        let code = r#"
impl Display for Point {
    fn fmt(&self) -> str {
        return ""
    }
}"#;
        let mut parser = create_parser(code, SyntaxMode::Braces);
        let result = parser.parse_impl_declaration(Visibility::Private);
        
        assert!(result.is_ok());
        
        if let Ok(ASTNode::Declaration(Declaration::Impl(impl_decl))) = result {
            assert_eq!(impl_decl.trait_name, Some("Display".to_string()));
            if let Type::Custom(type_name) = &impl_decl.target_type {
                assert_eq!(type_name, "Point");
            }
            assert_eq!(impl_decl.methods.len(), 1);
            assert_eq!(impl_decl.methods[0].name, "fmt");
        } else {
            panic!("Expected impl declaration");
        }
    }

    #[test]
    fn test_impl_with_generics() {
        let code = r#"
impl<T> Vec<T> {
    fn push(&mut self, value: T) {
        return;
    }
}"#;
        let mut parser = create_parser(code, SyntaxMode::Braces);
        let result = parser.parse_impl_declaration(Visibility::Public);
        
        if let Ok(ASTNode::Declaration(Declaration::Impl(impl_decl))) = result {
            if let Type::Custom(type_name) = &impl_decl.target_type {
                assert_eq!(type_name, "Vec");
            }
            assert!(impl_decl.generic_parameters.is_some());
            if let Some(generics) = impl_decl.generic_parameters {
                assert_eq!(generics.len(), 1);
                assert_eq!(generics[0].name, "T");
            }
        }
    }
}