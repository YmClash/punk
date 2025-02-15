use crate::parser::ast::{GenericParameter, ImplMethod, Parameter, SelfKind, Type, TypeBound, Visibility};
use crate::parser::parser::Parser;
use crate::parser::parser_error::{ParserError, ParserErrorType};
use crate::parser::parser_error::ParserErrorType::{ExpectedLifetime, InvalidConstructorName, MissingType};
use crate::tok::{Delimiters, Keywords, Operators, TokenType};

impl Parser{
    pub fn parse_generic_parameters(&mut self) -> Result<Vec<GenericParameter>, ParserError> {
        self.consume(TokenType::OPERATOR(Operators::LESS))?; // Consomme '<'
        let mut params = Vec::new();

        while !self.check(&[TokenType::OPERATOR(Operators::GREATER)]) {
            let name = self.consume_identifier()?;
            let mut bounds = Vec::new();

            // Parse les bounds du paramètre générique (T: Display + Clone)
            if self.check(&[TokenType::DELIMITER(Delimiters::COLON)]) {
                self.advance(); // Consomme ':'
                bounds = self.parse_trait_bounds()?;
            }

            params.push(GenericParameter { name, bounds });

            // Gère la virgule entre les paramètres
            if self.check(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
                self.advance();
            } else {
                break;
            }
        }

        self.consume(TokenType::OPERATOR(Operators::GREATER))?; // Consomme '>'
        Ok(params)
    }

    pub fn parse_trait_bounds(&mut self) -> Result<Vec<TypeBound>, ParserError> {
        let mut bounds = Vec::new();

        loop {
            let bound = if self.check_lifetime_token() {
                TypeBound::Lifetime(self.parse_lifetime()?)
            } else {
                TypeBound::TraitBound(self.consume_identifier()?)
            };

            bounds.push(bound);

            // Vérifie s'il y a d'autres bounds (séparés par +)
            if self.check(&[TokenType::OPERATOR(Operators::PLUS)]) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(bounds)
    }

    /// Parse un lifetime ('a, 'static, etc)

    fn parse_lifetime(&mut self) -> Result<String, ParserError> {
        if let Some(token) = self.current_token() {
            match &token.token_type {
                TokenType::IDENTIFIER { name } if name.starts_with('\'') => {
                    let lifetime_name = name.clone();
                    self.advance(); // Consomme le token
                    Ok(lifetime_name)
                },
                _ => Err(ParserError::new(ExpectedLifetime, self.current_position()
                )),
            }
        } else {
            Err(ParserError::new(ExpectedLifetime, self.current_position()
            ))
        }
    }

    pub fn parse_impl_method(&mut self) -> Result<ImplMethod, ParserError> {
        let visibility = self.parse_visibility().unwrap_or(Visibility::Private);

        // Vérifier si c'est un constructeur ou une méthode normale
        let (_is_constructor, name) = if self.check(&[TokenType::KEYWORD(Keywords::DEF)]) {
            self.advance();
            let name = self.consume_identifier()?;
            if name == "init" {
                (true, name)
            } else {
                return Err(ParserError::new(
                    InvalidConstructorName,
                    self.current_position()
                ));
            }

        } else {
            self.consume(TokenType::KEYWORD(Keywords::FN))?;
            let name = self.consume_identifier()?;
            (false, name)
        };

        // Parser les paramètres avec une meilleure gestion des erreurs
        self.consume(TokenType::DELIMITER(Delimiters::LPAR))?;
        let mut parameters = Vec::new();
        let mut self_param = None;

        if !self.check(&[TokenType::DELIMITER(Delimiters::RPAR)]) {
            // Gérer le paramètre self s'il existe
            if self.check(&[TokenType::KEYWORD(Keywords::SELF)]) {
                self_param = Some(self.parse_self_parameter()?);
                // self_param = Some(parse_paramer);
                // S'il y a une virgule après self, continuer avec les autres paramètres
                if self.check(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
                    self.advance();
                }
            }

            // Parser les autres paramètres
            while !self.check(&[TokenType::DELIMITER(Delimiters::RPAR)]) {
                let param = self.parse_parameter()?;
                parameters.push(param);

                if self.check(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.consume(TokenType::DELIMITER(Delimiters::RPAR))?;

        // Parser le type de retour
        let return_type = if self.check(&[TokenType::OPERATOR(Operators::RARROW)]) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        // Parser le corps de la méthode
        let body = self.parse_block()?;

        Ok(ImplMethod {
            name,
            self_param,
            parameters,
            return_type,
            visibility,
            body,
        })
    }

    #[allow(dead_code)]
    fn parse_constructor_parameters(&mut self) -> Result<Vec<Parameter>, ParserError> {
        let mut parameters = Vec::new();

        if !self.check(&[TokenType::DELIMITER(Delimiters::RPAR)]) {
            loop {
                let param = self.parse_parameter()?;
                parameters.push(param);

                if !self.check(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
                    break;
                }
                self.advance(); // Consomme la virgule
            }
        }

        self.consume(TokenType::DELIMITER(Delimiters::RPAR))?;
        Ok(parameters)
    }

    // plus tard   je  devrai unifie les deux fonctions parse_self_parameter et parse_method_parameters

    #[allow(dead_code)]
    fn parse_method_parameters(&mut self) -> Result<(Option<SelfKind>, Vec<Parameter>), ParserError> {
        self.consume(TokenType::DELIMITER(Delimiters::LPAR))?;
        let mut parameters = Vec::new();
        let mut self_param = None;

        if !self.check(&[TokenType::DELIMITER(Delimiters::RPAR)]) {
            // Vérifie si le premier paramètre est self
            if self.check(&[TokenType::KEYWORD(Keywords::SELF)]) {
                self.advance();
                self_param = Some(SelfKind::Value);
            } else if self.check(&[TokenType::OPERATOR(Operators::AMPER)]) { // &self
                self.advance();
                if self.check(&[TokenType::KEYWORD(Keywords::MUT)]) {
                    self.advance();
                    self.consume(TokenType::KEYWORD(Keywords::SELF))?;
                    self_param = Some(SelfKind::MutableReference);
                } else {
                    self.consume(TokenType::KEYWORD(Keywords::SELF))?;
                    self_param = Some(SelfKind::Reference);
                }
            }

            // S'il y a d'autres paramètres après self
            if self_param.is_some() && self.check(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
                self.advance();
            }

            // Parse les autres paramètres normaux
            while !self.check(&[TokenType::DELIMITER(Delimiters::RPAR)]) {
                let param = self.parse_parameter()?; // Utilise parse_parameter au lieu de parse_method_parameters
                parameters.push(param);

                if self.check(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.consume(TokenType::DELIMITER(Delimiters::RPAR))?;
        Ok((self_param, parameters))
    }

    fn parse_self_parameter(&mut self) -> Result<SelfKind, ParserError> {
        // Cas 1: self tout simple
        if self.check(&[TokenType::KEYWORD(Keywords::SELF)]) {
            self.advance(); // Consomme 'self'
            return Ok(SelfKind::Value);
        }

        // Cas 2: &self ou &mut self
        if self.check(&[TokenType::OPERATOR(Operators::AMPER)]) {
            self.advance(); // Consomme '&'

            if self.check(&[TokenType::KEYWORD(Keywords::MUT)]) {
                self.advance(); // Consomme 'mut'
                self.consume(TokenType::KEYWORD(Keywords::SELF))?;
                Ok(SelfKind::MutableReference)
            } else {
                self.consume(TokenType::KEYWORD(Keywords::SELF))?;
                Ok(SelfKind::Reference)
            }
        } else {
            Err(ParserError::new(
                ParserErrorType::InvalidSelfParameter,
                self.current_position(),
            ))
        }
    }

    fn parse_parameter(&mut self) -> Result<Parameter, ParserError> {
        println!("Début du parsing d'un paramètre");

        // 1. Parser le nom du paramètre
        let param_name = self.consume_identifier()?;

        // 2. Si on trouve un deux-points, on doit avoir un type qui suit
        if self.check(&[TokenType::DELIMITER(Delimiters::COLON)]) {
            self.advance(); // Consommer le ':'

            // Si on ne trouve pas de type après le ':', c'est une erreur
            if self.check(&[TokenType::DELIMITER(Delimiters::RPAR)]) {
                return Err(ParserError::new(
                    MissingType,
                    self.current_position()));
            }

            let param_type = self.parse_type()?;
            Ok(Parameter {
                name: param_name,
                parameter_type: param_type,
            })
        } else {
            // Si pas de ':', utiliser le type Infer
            Ok(Parameter {
                name: param_name,
                parameter_type: Type::Infer,
            })
        }
    }

    // fn parse_parameter(&mut self) -> Result<Parameter, ParserError> {
    //     let param_name = self.consume_identifier()?;
    //
    //     // Le type est obligatoire pour les paramètres
    //     self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
    //     let param_type = self.parse_type()?;
    //
    //     Ok(Parameter {
    //         name: param_name,
    //         parameter_type: param_type,
    //     })
    // }





}