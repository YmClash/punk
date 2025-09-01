// src/parser/declarations/classes.rs
// Module pour le parsing des déclarations de classes

use crate::parser::ast::{
    ASTNode, Declaration, ClassDeclaration, Constructor, 
    Attribute, MethodeDeclaration, Visibility, Mutability, Type
};
use crate::parser::parser::Parser;
use crate::parser::parser_error::{ParserError, ParserErrorType};
use crate::tok::{TokenType, Keywords, Delimiters, Operators};
use crate::SyntaxMode;

impl Parser {
    /// Parse une déclaration de classe
    pub fn parse_class_declaration(&mut self, visibility: Visibility) -> Result<ASTNode, ParserError> {
        log::debug!("Début du parsing de la déclaration de classe");
        
        self.consume(TokenType::KEYWORD(Keywords::CLASS))?;
        let name = self.consume_identifier()?;
        log::debug!("Nom de la classe parsé : {}", name);

        let parent_classes = self.parse_class_inheritance()?;

        match self.syntax_mode {
            SyntaxMode::Indentation => self.consume(TokenType::DELIMITER(Delimiters::COLON))?,
            SyntaxMode::Braces => (),
        }

        let (attributes, methods, constructor) = self.parse_class_body()?;

        log::debug!("Fin du parsing de la classe");

        Ok(ASTNode::Declaration(Declaration::Class(ClassDeclaration {
            name,
            parent_classes,
            attributes,
            constructor,
            methods,
            visibility,
        })))
    }

    /// Parse l'héritage de classe (classes parentes)
    pub fn parse_class_inheritance(&mut self) -> Result<Vec<String>, ParserError> {
        let mut parent_classes = Vec::new();
        
        if self.check(&[TokenType::DELIMITER(Delimiters::LPAR)]) {
            self.consume(TokenType::DELIMITER(Delimiters::LPAR))?;
            
            loop {
                let parent = self.consume_identifier()?;
                parent_classes.push(parent);
                
                if !self.match_token(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
                    break;
                }
            }
            
            self.consume(TokenType::DELIMITER(Delimiters::RPAR))?;
        }
        
        log::debug!("Classes parentes parsées : {:?}", parent_classes);
        Ok(parent_classes)
    }

    /// Parse le corps d'une classe (attributs, méthodes, constructeur)
    pub fn parse_class_body(&mut self) -> Result<(Vec<Attribute>, Vec<MethodeDeclaration>, Option<Constructor>), ParserError> {
        let mut attributes = Vec::new();
        let mut methods = Vec::new();
        let mut constructor = None;

        match self.syntax_mode {
            SyntaxMode::Braces => {
                self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?;
                
                while !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]) && !self.is_at_end() {
                    if self.check(&[TokenType::KEYWORD(Keywords::DEF)]) {
                        if constructor.is_some() {
                            return Err(ParserError::new(
                                ParserErrorType::MultipleConstructors,
                                self.current_position()
                            ));
                        }
                        constructor = Some(self.parse_constructor_declaration()?);
                    } else if self.check(&[TokenType::KEYWORD(Keywords::FN)]) {
                        methods.push(self.parse_methode_declaration()?);
                    } else if self.check(&[TokenType::KEYWORD(Keywords::LET)]) {
                        attributes.push(self.parse_attribute_declaration()?);
                    } else if self.check(&[TokenType::KEYWORD(Keywords::PUB)]) {
                        // Gérer la visibilité publique
                        self.advance();
                        if self.check(&[TokenType::KEYWORD(Keywords::FN)]) {
                            let mut method = self.parse_methode_declaration()?;
                            method.visibility = Visibility::Public;
                            methods.push(method);
                        } else if self.check(&[TokenType::KEYWORD(Keywords::LET)]) {
                            let mut attr = self.parse_attribute_declaration()?;
                            attr.visibility = Visibility::Public;
                            attributes.push(attr);
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
                self.consume(TokenType::NEWLINE)?;
                self.consume(TokenType::INDENT)?;
                
                while !self.check(&[TokenType::EOF, TokenType::DEDENT]) && !self.is_at_end() {
                    // Skip empty lines
                    if self.check(&[TokenType::NEWLINE]) {
                        self.advance();
                        continue;
                    }
                    
                    if self.check(&[TokenType::KEYWORD(Keywords::DEF)]) {
                        if constructor.is_some() {
                            return Err(ParserError::new(
                                ParserErrorType::MultipleConstructors,
                                self.current_position()
                            ));
                        }
                        constructor = Some(self.parse_constructor_declaration()?);
                    } else if self.check(&[TokenType::KEYWORD(Keywords::FN)]) {
                        methods.push(self.parse_methode_declaration()?);
                    } else if self.check(&[TokenType::KEYWORD(Keywords::LET)]) {
                        attributes.push(self.parse_attribute_declaration()?);
                    } else if self.check(&[TokenType::KEYWORD(Keywords::PUB)]) {
                        self.advance();
                        if self.check(&[TokenType::KEYWORD(Keywords::FN)]) {
                            let mut method = self.parse_methode_declaration()?;
                            method.visibility = Visibility::Public;
                            methods.push(method);
                        } else if self.check(&[TokenType::KEYWORD(Keywords::LET)]) {
                            let mut attr = self.parse_attribute_declaration()?;
                            attr.visibility = Visibility::Public;
                            attributes.push(attr);
                        }
                    } else {
                        return Err(ParserError::new(
                            ParserErrorType::UnexpectedToken,
                            self.current_position()
                        ));
                    }
                }
                
                if self.check(&[TokenType::DEDENT]) {
                    self.consume(TokenType::DEDENT)?;
                }
            }
        }
        
        Ok((attributes, methods, constructor))
    }

    /// Parse une déclaration de constructeur
    fn parse_constructor_declaration(&mut self) -> Result<Constructor, ParserError> {
        log::debug!("Début du parsing du constructeur");
        
        self.consume(TokenType::KEYWORD(Keywords::DEF))?;
        let constructor_name = self.consume_identifier()?;
        
        // Python-style: __init__ ou simple init
        if constructor_name != "init" && constructor_name != "__init__" {
            return Err(ParserError::new(
                ParserErrorType::InvalidConstructorName,
                self.current_position()
            ));
        }
        
        self.consume(TokenType::DELIMITER(Delimiters::LPAR))?;
        let parameters = self.parse_function_parameters()?;
        self.consume(TokenType::DELIMITER(Delimiters::RPAR))?;

        // Parse le corps selon le mode
        let body = if self.syntax_mode == SyntaxMode::Indentation {
            self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
            self.parse_unified_block()?
        } else {
            self.parse_unified_block()?
        };

        log::debug!("Fin du parsing du constructeur");

        Ok(Constructor {
            name: constructor_name,
            parameters,
            body,
        })
    }

    /// Parse une déclaration d'attribut de classe
    fn parse_attribute_declaration(&mut self) -> Result<Attribute, ParserError> {
        log::debug!("Début du parsing de la déclaration d'attribut");
        
        let visibility = self.parse_visibility().unwrap_or(Visibility::Private);
        self.consume(TokenType::KEYWORD(Keywords::LET))?;
        let mutability = self.parse_mutability();

        let name = self.consume_identifier()?;
        self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
        let attribute_type = self.parse_type()?;
        
        self.consume_seperator();
        
        log::debug!("Parsing de la déclaration d'attribut terminé");

        Ok(Attribute {
            name,
            attr_type: attribute_type,
            visibility,
            mutability,
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
    fn test_simple_class() {
        let code = r#"
class Person {
    let name: str;
    let age: int;
}"#;
        let mut parser = create_parser(code, SyntaxMode::Braces);
        let result = parser.parse_class_declaration(Visibility::Public);
        
        assert!(result.is_ok());
        
        if let Ok(ASTNode::Declaration(Declaration::Class(class))) = result {
            assert_eq!(class.name, "Person");
            assert_eq!(class.attributes.len(), 2);
            assert_eq!(class.parent_classes.len(), 0);
        } else {
            panic!("Expected class declaration");
        }
    }

    #[test]
    fn test_class_with_inheritance() {
        let code = "class Dog(Animal, Pet) { let name: str; }";
        let mut parser = create_parser(code, SyntaxMode::Braces);
        let result = parser.parse_class_declaration(Visibility::Private);
        
        assert!(result.is_ok());
        
        if let Ok(ASTNode::Declaration(Declaration::Class(class))) = result {
            assert_eq!(class.name, "Dog");
            assert_eq!(class.parent_classes.len(), 2);
            assert_eq!(class.parent_classes[0], "Animal");
            assert_eq!(class.parent_classes[1], "Pet");
        } else {
            panic!("Expected class with inheritance");
        }
    }

    #[test]
    fn test_class_with_constructor() {
        let code = r#"
class Point {
    let x: float;
    let y: float;
    
    def init(self, x: float, y: float) {
        self.x = x;
        self.y = y;
    }
}"#;
        let mut parser = create_parser(code, SyntaxMode::Braces);
        let result = parser.parse_class_declaration(Visibility::Public);
        
        if result.is_ok() {
            if let Ok(ASTNode::Declaration(Declaration::Class(class))) = result {
                assert_eq!(class.name, "Point");
                assert!(class.constructor.is_some());
                
                if let Some(ctor) = class.constructor {
                    assert_eq!(ctor.name, "init");
                    assert_eq!(ctor.parameters.len(), 3); // self, x, y
                }
            }
        }
    }
}