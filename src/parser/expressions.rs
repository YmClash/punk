use crate::parser::ast::{ ArrayExpression, ArraySlice, Assignment, ASTNode, BinaryOperation, CompoundAssignment, ComprehensionFor, DestructuringAssignment, DictAccess, Expression, FunctionCall, IndexAccess, LambdaExpression, ListComprehension, Literal, MemberAccess, MethodCall, Operator, Parameter, Pattern, RangeExpression, Type, UnaryOperation, UnaryOperator};
use crate::parser::parser::Parser;
use crate::parser::parser_error::ParserError;
use crate::parser::parser_error::ParserErrorType::{ExpectedArrowOrBlock, ExpectedCloseParenthesis, ExpectedCommaOrClosingParenthesis, UnexpectedEndOfInput, UnexpectedToken};
use crate::tok::{Delimiters, Keywords, Operators, TokenType};

impl Parser {
    /// fonction pour parser les expressions

    pub fn parse_expression(&mut self,precedence:u8) -> Result<Expression, ParserError> {
        log::debug!("Début du parsing de l'expression");

        if self.check(&[TokenType::DELIMITER(Delimiters::LCURBRACE)]){
            return self.parse_dict_literal();
        }


        if self.is_list_comprehension() {
            return self.parse_list_comprehension();
        }

        if self.check(&[TokenType::DELIMITER(Delimiters::LSBRACKET)]) {
            // Si nous avons un token précédent et que c'est un '=', alors c'est une expression de tableau
            // Sinon, c'est une destructuration
            match self.previous_token() {
                Some(token) if token.token_type == TokenType::OPERATOR(Operators::EQUAL) => {
                    return self.parse_array_expression();
                },
                _ => {
                    return self.parse_destructuring_assignment();
                }
            }
        }


        //let mut left = self.parse_postfix_expression()?;
        let mut left = self.parse_unary_expression()?;


        if let Some(token) = self.current_token(){
            match &token.token_type {
                TokenType::OPERATOR(Operators::EQUAL) => {
                    self.advance();
                    let value = self.parse_expression(precedence)?;
                    return Ok(Expression::Assignment(Assignment{
                        target: Box::new(left),
                        value: Box::new(value),
                    }));
                }
                TokenType::OPERATOR(op) => {
                    if let Some(compound_op) = self.get_compound_operator(op){
                        self.advance();
                        let value = self.parse_expression(precedence)?;
                        return Ok(Expression::CompoundAssignment(CompoundAssignment{
                            target: Box::new(left),
                            operator: compound_op,
                            value: Box::new(value),
                        }));
                    }
                }
                _ => {}
            }
        }


        while let Some (operator) = self.peek_operator(){
            let operator_precedence = self.get_operator_precedence(&operator);
            if operator_precedence < precedence {
                break;
            }

            self.advance();
            let right = self.parse_expression(precedence +1)?;


            if let Operator::Range|Operator::RangeInclusive = operator{
                left = Expression::RangeExpression(RangeExpression{
                    left: Some(Box::new(left)),
                    operator,
                    right: Some(Box::new(right)),
                });
            }else {
                left = Expression::BinaryOperation(BinaryOperation{
                    left: Box::new(left),
                    operator,
                    right: Box::new(right),
                });
            }

        }

        log::debug!("Fin du parsing de l'expression ");

        Ok(left)

    }

    pub fn parse_expression_statement(&mut self) -> Result<ASTNode, ParserError> {
        log::debug!("Début du parsing de l'expression statement");
        let expr = self.parse_expression(0);
        log::debug!("Expression parsée : {:?}", expr);
        //self.consume(TokenType::DELIMITER(Delimiters::SEMICOLON))?;
        self.consume_seperator();
        log::debug!("Separateur consommé");
        Ok(ASTNode::Expression(expr?))

    }


    pub fn parse_postfix_expression(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.parse_primary_expression()?;

        while let Some(token) = self.current_token() {
            expr = match &token.token_type {
                TokenType::DELIMITER(Delimiters::LSBRACKET) => {
                    self.advance(); // Consume [

                    // Parse start
                    let start = if !self.check(&[TokenType::DELIMITER(Delimiters::COLON)]) {
                        Some(Box::new(self.parse_expression(0)?))
                    } else {
                        None
                    };

                    // Si on trouve .. ou :, c'est un slice
                    if self.check(&[TokenType::OPERATOR(Operators::DOTDOT)]) ||
                        self.check(&[TokenType::DELIMITER(Delimiters::COLON)]) {
                        self.advance(); // Consomme '..' ou ':'

                        // Parse end
                        let end = if !self.check(&[TokenType::DELIMITER(Delimiters::COLON)]) &&
                            !self.check(&[TokenType::DELIMITER(Delimiters::RSBRACKET)]) {
                            Some(Box::new(self.parse_expression(0)?))
                        } else {
                            None
                        };

                        // Parse step
                        let step = if self.check(&[TokenType::DELIMITER(Delimiters::COLON)]) {
                            self.advance(); // Consomme ':'
                            Some(Box::new(self.parse_expression(0)?))
                        } else {
                            None
                        };

                        self.consume(TokenType::DELIMITER(Delimiters::RSBRACKET))?;

                        // Ne pas créer de RangeExpression, traiter start comme une valeur normale
                        Expression::ArraySlice(ArraySlice {
                            array: Box::new(expr),
                            start,
                            end,
                            step
                        })
                    } else if let Some(start) = start {
                        // Simple index access
                        self.consume(TokenType::DELIMITER(Delimiters::RSBRACKET))?;
                        match &*start {
                            Expression::Literal(Literal::String(_)) => {
                                Expression::DictAccess(DictAccess {
                                    dict: Box::new(expr),
                                    key: start
                                })
                            },
                            _ => Expression::IndexAccess(IndexAccess {
                                array: Box::new(expr),
                                index: start
                            })
                        }
                    } else {
                        return Err(ParserError::new(UnexpectedToken, self.current_position()));
                    }
                },

                TokenType::DELIMITER(Delimiters::LPAR) => {
                    self.advance();
                    let arguments = self.parse_arguments_list()?;
                    // self.expect_token(&TokenType::DELIMITER(Delimiters::RPAR))?;
                    self.consume(TokenType::DELIMITER(Delimiters::RPAR))?;
                    Expression::FunctionCall(FunctionCall {
                        name: Box::new(expr),
                        arguments
                    })
                },
                TokenType::DELIMITER(Delimiters::DOT) => {
                    self.advance();
                    if let Some(TokenType::IDENTIFIER { name }) = self.current_token().map(|t| &t.token_type) {
                        let name = name.clone();
                        self.advance();
                        if self.check(&[TokenType::DELIMITER(Delimiters::LPAR)]) {
                            self.advance();
                            let arguments = self.parse_arguments_list()?;
                            // self.expect_token(&TokenType::DELIMITER(Delimiters::RPAR))?;
                            self.consume(TokenType::DELIMITER(Delimiters::RPAR))?;
                            Expression::MethodCall(MethodCall {
                                object: Box::new(expr),
                                method: name,
                                arguments
                            })
                        } else {
                            Expression::MemberAccess(MemberAccess {
                                object: Box::new(expr),
                                member: name
                            })
                        }
                    } else {
                        return Err(ParserError::new(UnexpectedToken, self.current_position()));
                    }
                },
                _ => break,
            };
        }

        Ok(expr)
    }


    pub fn parse_destructuring_assignment(&mut self) -> Result<Expression,ParserError>{
        log::debug!("Début du parsing de l'assignation destructuree[");
        self.consume(TokenType::DELIMITER(Delimiters::LSBRACKET))?;
        let mut targets = Vec::new();
        loop {
            let target = self.parse_expression(0)?;
            targets.push(target);
            if !self.match_token(&[TokenType::DELIMITER(Delimiters::COMMA)]){
                break;
            }
        }

        self.consume(TokenType::DELIMITER(Delimiters::RSBRACKET))?;


        if self.check(&[TokenType::OPERATOR(Operators::EQUAL)]){
            self.consume(TokenType::OPERATOR(Operators::EQUAL))?;
            let value = self.parse_expression(0)?;

            log::debug!("Fin du parsing de l'assignation destructuree OK!!!!");
            Ok(Expression::DestructuringAssignment(DestructuringAssignment {
                targets,
                value: Box::new(value),
            }))
        } else {
            // C'est un tableau littéral
            log::debug!("Fin du parsing d'un tableau");
            Ok(Expression::Array(ArrayExpression {
                elements: targets
            }))
        }
    }

    pub fn parse_unary_expression(&mut self) -> Result<Expression, ParserError> {
        log::debug!("Début du parsing de l'expression unaire");
        log::debug!("Début du parsing de l'expression unaire, current_token = {:?}", self.current_token());
        if let Some(token) = self.current_token(){
            match &token.token_type{
                //Gestion de la Negation (-)
                TokenType::OPERATOR(Operators::MINUS) => {
                    self.advance();
                    let right = self.parse_unary_expression()?;
                    return Ok(Expression::UnaryOperation(UnaryOperation{
                        operator: UnaryOperator::Negative,
                        operand: Box::new(right),
                    }));
                }
                // Gestion de la Negation  Logique (!)
                TokenType::OPERATOR(Operators::EXCLAMATION) => {
                    self.advance();
                    let right = self.parse_unary_expression()?;
                    return Ok(Expression::UnaryOperation(UnaryOperation{
                        operator: UnaryOperator::Not,
                        operand: Box::new(right),
                    }));
                }
                // Gestion de la Reference(Borrowing) (&)
                TokenType::OPERATOR(Operators::AMPER) => {
                    self.advance();
                    if self.check(&[TokenType::KEYWORD(Keywords::MUT)]){
                        self.advance();
                        let right = self.parse_unary_expression()?;
                        return Ok(Expression::UnaryOperation(UnaryOperation{
                            operator: UnaryOperator::ReferenceMutable,
                            operand: Box::new(right),
                        }));
                    }else{
                        let right = self.parse_unary_expression()?;
                        return Ok(Expression::UnaryOperation(UnaryOperation{
                            operator: UnaryOperator::Reference,
                            operand: Box::new(right),
                        }));
                    }
                }
                _ => self.parse_postfix_expression()
            }
        }else { Err(ParserError::new(UnexpectedEndOfInput, self.current_position())) }

    }


    pub fn parse_primary_expression(&mut self) -> Result<Expression, ParserError> {
        log::debug!("Début du parsing de l'expression primaire, current_token = {:?}", self.current_token());
        if let Some(token) = self.current_token() {
            let expr = match &token.token_type {
                TokenType::INTEGER { value } => {
                    let value = value.clone();
                    log::debug!("Valeur entière parsée : {}", value);
                    self.advance();
                    Expression::Literal(Literal::Integer { value })
                }
                TokenType::FLOAT { value } => {
                    let value = *value;
                    log::debug!("Valeur flottante parsée : {}", value);
                    self.advance();
                    Expression::Literal(Literal::Float { value })
                }

                TokenType::STRING { value,.. } => {
                    let value = value.clone();
                    if value.len() == 1 && self.if_single_quote(&value) {
                        self.advance();
                        Expression::Literal(Literal::Char(value.chars().next().unwrap()))
                    }else {
                        self.advance();
                        Expression::Literal(Literal::String(value))
                    }
                }

                TokenType::CHAR { value } => {
                    let value = *value;
                    log::debug!("Valeur de caractère parsée : {}", value);
                    self.advance();
                    Expression::Literal(Literal::Char(value))
                }

                TokenType::KEYWORD(Keywords::TRUE) => {
                    self.advance(); // Consomme le token
                    Expression::Literal(Literal::Boolean(true))
                }
                TokenType::KEYWORD(Keywords::FALSE) => {
                    self.advance(); // Consomme le token
                    Expression::Literal(Literal::Boolean(false))
                }
                // SELF   pour les methode d'instantiation dans class declaration
                TokenType::KEYWORD(Keywords::SELF) =>{
                    self.advance();
                    let name = "self".to_string();
                    Expression::Identifier(name)

                }

                TokenType::IDENTIFIER { name } => {
                    let name = name.clone();
                    self.advance();
                    Expression::Identifier(name)
                }
                TokenType::KEYWORD(Keywords::LAMBDA) => {
                    // self.advance();
                    self.parse_lambda_expression()?
                }

                TokenType::DELIMITER(Delimiters::LPAR) => {
                    self.advance();
                    let expr = self.parse_expression(0)?;
                    if let Some(token) = self.current_token() {
                        if matches!(token.token_type, TokenType::DELIMITER(Delimiters::RPAR)) {
                            self.advance();
                            expr
                        } else {
                            return Err(ParserError::new(
                                ExpectedCloseParenthesis,
                                self.current_position(),
                            ));
                        }
                    } else {
                        return Err(ParserError::new(
                            UnexpectedEndOfInput,
                            self.current_position(),
                        ));
                    }
                }
                _ => return Err(ParserError::new(UnexpectedToken, self.current_position())),
            };
            Ok(expr)
        } else {
            Err(ParserError::new(
                UnexpectedEndOfInput,
                self.current_position(),
            ))
        }

    }


    pub fn parse_lambda_expression(&mut self) -> Result<Expression, ParserError> {
        log::debug!("Début du parsing de l'expression lambda");
        self.consume(TokenType::KEYWORD(Keywords::LAMBDA))?;

        self.consume(TokenType::DELIMITER(Delimiters::LPAR))?;
        let parameters = self.parse_parameter_list()?;
        //let parameters = self.parse_function_parameters()?;
        self.consume(TokenType::DELIMITER(Delimiters::RPAR))?;

        let return_type = if self.match_token(&[TokenType::OPERATOR(Operators::RARROW)]) {
            self.parse_type()?
        } else {
            Type::Infer
        };

        let body = if self.match_token(&[TokenType::OPERATOR(Operators::FATARROW)]) {
            // Expression unique
            let expr = self.parse_expression(0)?;
            vec![ASTNode::Expression(expr)]
        } else if self.check(&[TokenType::DELIMITER(Delimiters::LCURBRACE)]) {
            // Bloc de code
            self.parse_block_expression()?
            //self.parse_body_block()?
            // self.parse_block()?
        } else {
            return Err(ParserError::new(ExpectedArrowOrBlock, self.current_position()));
        };

        Ok(Expression::LambdaExpression(LambdaExpression{
            parameters,
            return_type: Some(return_type),
            body,
        }))

    }

    /// fonction pour parser les parametres

    pub fn parse_arguments_list(&mut self) -> Result<Vec<Expression>, ParserError> {
        log::debug!("Début du parsing de la liste d'arguments");
        let mut arguments = Vec::new();
        if self.check(&[TokenType::DELIMITER(Delimiters::RPAR)]){
            return Ok(arguments);
        }
        loop {
            let argument = self.parse_expression(0);
            arguments.push(argument?);

            if !self.match_token(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
                break;
            }
        }
        log::debug!("Arguments liste parsés : {:?}", arguments);
        Ok(arguments)

    }

    pub fn parse_parameter_list(&mut self) -> Result<Vec<Parameter>, ParserError> {
        log::debug!("Début du parsing de la liste des paramètres");
        let mut parameters = Vec::new();

        if self.check(&[TokenType::DELIMITER(Delimiters::RPAR)]) {
            self.advance(); // Consomme ')'
            return Ok(parameters); // Pas de paramètres
        }

        loop {
            let param_name = self.consume_identifier()?;

            // Vérifier s'il y a un type spécifié
            let param_type = if self.match_token(&[TokenType::DELIMITER(Delimiters::COLON)]) {
                Some(self.parse_type()?)
            } else {
                None
            };

            parameters.push(Parameter {
                name: param_name,
                parameter_type: param_type.unwrap_or(Type::Infer),
            });

            // Si le prochain token est une virgule, continuer
            if self.match_token(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
                continue;
            } else if self.check(&[TokenType::DELIMITER(Delimiters::RPAR)]) {
                //self.advance(); // Consomme ')'
                break;
            } else {
                return Err(ParserError::new(ExpectedCommaOrClosingParenthesis, self.current_position()));
            }
        }

        Ok(parameters)
    }

    /// Parse un dictionnaire littéral
    pub fn parse_dict_literal(&mut self) -> Result<Expression, ParserError> {
        log::debug!("Parsing dict literal");
        self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?;
        
        let mut entries = Vec::new();
        
        while !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]) && !self.is_at_end() {
            // Parse la clé
            let key = self.parse_expression(0)?;
            self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
            // Parse la valeur
            let value = self.parse_expression(0)?;
            
            entries.push((key, value));
            
            if !self.match_token(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
                break;
            }
        }
        
        self.consume(TokenType::DELIMITER(Delimiters::RCURBRACE))?;
        
        // Retourner une expression Dict  
        // TODO: Implémenter le type Dictionary dans l'AST
        // Pour l'instant, on retourne une erreur
        Err(ParserError::new(
            UnexpectedToken,
            self.current_position()
        ))
    }
    
    /// Vérifie si c'est une list comprehension
    pub fn is_list_comprehension(&mut self) -> bool {
        // Regarde si on a un pattern de type: [expr for var in iter]
        let saved_pos = self.current;
        
        // Doit commencer par [
        if !self.check(&[TokenType::DELIMITER(Delimiters::LSBRACKET)]) {
            return false;
        }
        self.advance();
        
        // Skip l'expression initiale
        while !self.check(&[TokenType::KEYWORD(Keywords::FOR)]) && 
              !self.check(&[TokenType::DELIMITER(Delimiters::RSBRACKET)]) && 
              !self.is_at_end() {
            self.advance();
        }
        
        let result = self.check(&[TokenType::KEYWORD(Keywords::FOR)]);
        self.current = saved_pos;
        result
    }
    
    /// Parse une list comprehension
    pub fn parse_list_comprehension(&mut self) -> Result<Expression, ParserError> {
        log::debug!("Parsing list comprehension");
        self.consume(TokenType::DELIMITER(Delimiters::LSBRACKET))?;
        
        // Parse l'expression de transformation
        let expr = self.parse_expression(0)?;
        
        // Parse "for"
        self.consume(TokenType::KEYWORD(Keywords::FOR))?;
        
        // Parse la variable d'itération
        let var = self.consume_identifier()?;
        
        // Parse "in"
        self.consume(TokenType::KEYWORD(Keywords::IN))?;
        
        // Parse l'itérable
        let iterable = self.parse_expression(0)?;
        
        // Parse la condition optionnelle "if"
        let condition = if self.match_token(&[TokenType::KEYWORD(Keywords::IF)]) {
            Some(Box::new(self.parse_expression(0)?))
        } else {
            None
        };
        
        self.consume(TokenType::DELIMITER(Delimiters::RSBRACKET))?;
        
        // Retourner une expression ListComprehension
        // Créer un ComprehensionFor pour l'itérateur
        let comp_for = ComprehensionFor {
            pattern: Pattern::Identifier(var),
            iterator: iterable,
        };
        
        // Rassembler les conditions si présentes
        let conditions = if let Some(cond) = condition {
            vec![*cond]
        } else {
            Vec::new()
        };
        
        Ok(Expression::ListComprehension(ListComprehension {
            elements: Box::new(expr),
            iterators: vec![comp_for],
            conditions,
        }))
    }
    
    /// Parse une expression de tableau/liste
    pub fn parse_array_expression(&mut self) -> Result<Expression, ParserError> {
        log::debug!("Parsing array expression");
        
        // Si c'est une list comprehension
        if self.is_list_comprehension() {
            return self.parse_list_comprehension();
        }
        
        self.consume(TokenType::DELIMITER(Delimiters::LSBRACKET))?;
        
        let mut elements = Vec::new();
        
        while !self.check(&[TokenType::DELIMITER(Delimiters::RSBRACKET)]) && !self.is_at_end() {
            elements.push(self.parse_expression(0)?);
            
            if !self.match_token(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
                break;
            }
        }
        
        self.consume(TokenType::DELIMITER(Delimiters::RSBRACKET))?;
        
        Ok(Expression::Array(ArrayExpression {
            elements,
        }))
    }

}