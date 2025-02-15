// fonction pour aider le parsing des erreurs
// il syncronise  le parsing apres une erreur  a implementer plus tard

use crate::parser::ast::{ImplMethod, Visibility};
use crate::parser::parser::Parser;
use crate::parser::parser_error::{ParserError, ParserErrorType};
use crate::SyntaxMode;
use crate::tok::{Delimiters, Keywords, TokenType};
impl Parser{

    pub fn synchronize(&mut self) -> Result<(), ParserError> {
        println!("Début de la synchronisation après erreur");

        let mut nesting_level: i32 = 0;

        fn is_declaration_start(token_type: &TokenType) -> bool {
            matches!(
            token_type,
            TokenType::KEYWORD(Keywords::FN) |
            TokenType::KEYWORD(Keywords::LET) |
            TokenType::KEYWORD(Keywords::CONST) |
            TokenType::KEYWORD(Keywords::STRUCT) |
            TokenType::KEYWORD(Keywords::ENUM) |
            TokenType::KEYWORD(Keywords::TRAIT) |
            TokenType::KEYWORD(Keywords::IMPL) |
            TokenType::KEYWORD(Keywords::CLASS)
        )
        }

        while !self.is_at_end() {
            // Gérer le niveau d'imbrication pour les blocs
            let current_token = self.current_token()
                .ok_or_else(|| ParserError::new(
                    ParserErrorType::UnexpectedEOF,
                    self.current_position()
                ))?;

            match &current_token.token_type {
                TokenType::DELIMITER(Delimiters::LCURBRACE) => {
                    nesting_level += 1;
                },
                TokenType::DELIMITER(Delimiters::RCURBRACE) => {
                    nesting_level = nesting_level.saturating_sub(1);
                    if nesting_level == 0 {
                        self.advance();
                        return Ok(());
                    }
                },
                TokenType::INDENT => {
                    if self.syntax_mode == SyntaxMode::Indentation {
                        nesting_level += 1;
                    }
                },
                TokenType::DEDENT => {
                    if self.syntax_mode == SyntaxMode::Indentation {
                        nesting_level = nesting_level.saturating_sub(1);
                        if nesting_level == 0 {
                            self.advance();
                            return Ok(());
                        }
                    }
                },
                _ => {}
            }

            // Si on est au niveau 0 et qu'on trouve un début de déclaration
            if nesting_level == 0 && is_declaration_start(&current_token.token_type) {
                return Ok(());
            }

            self.advance();
        }

        Ok(())
    }

    // Helper pour la récupération d'erreur dans les blocs spécifiques

    fn synchronize_block(&mut self) -> Result<(), ParserError> {
        let mut nesting = 1;

        while !self.is_at_end() {
            // Convertir l'Option en Result avec gestion d'erreur explicite
            let current_token = self.current_token()
                .ok_or_else(|| ParserError::new(
                    ParserErrorType::UnexpectedEOF,
                    self.current_position()
                ))?;

            match self.syntax_mode {
                SyntaxMode::Braces => {
                    match &current_token.token_type {
                        TokenType::DELIMITER(Delimiters::LCURBRACE) => nesting += 1,
                        TokenType::DELIMITER(Delimiters::RCURBRACE) => {
                            nesting -= 1;
                            if nesting == 0 {
                                self.advance();
                                return Ok(());
                            }
                        }
                        _ => {}
                    }
                }
                SyntaxMode::Indentation => {
                    match &current_token.token_type {
                        TokenType::INDENT => nesting += 1,
                        TokenType::DEDENT => {
                            nesting -= 1;
                            if nesting == 0 {
                                self.advance();
                                return Ok(());
                            }
                        }
                        _ => {}
                    }
                }
            }
            self.advance();
        }
        Ok(())
    }

    // Exemple d'utilisation dans une méthode de parsing
    fn parse_method_with_recovery(&mut self) -> Result<ImplMethod, ParserError> {
        let start_pos = self.current_position();
        match self.parse_impl_method() {
            Ok(method) => Ok(method),
            Err(e) => {
                println!("Erreur lors du parsing de la méthode : {:?}", e);
                self.synchronize()?;

                // Retourne une méthode "placeholder" pour continuer le parsing
                Ok(ImplMethod {
                    name: "error".to_string(),
                    self_param: None,
                    parameters: Vec::new(),
                    return_type: None,
                    visibility: Visibility::Private,
                    body: Vec::new(),
                })
            }
        }
    }



    fn sync_after_error(&mut self) {
        // Synchronisation après une erreur
        while let Some(token) = self.current_token() {
            match token.token_type {
                // Points de synchronisation
                TokenType::DELIMITER(Delimiters::SEMICOLON) |
                TokenType::DELIMITER(Delimiters::RCURBRACE) => {
                    self.advance();
                    break;
               }
                _ => {self.advance();}
                // _ => {self.advance();}
            }
        }
    }


}