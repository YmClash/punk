#[allow(dead_code)]
use crate::lexer::lex::{SyntaxMode, Token};

use crate::parser::ast::{ArrayRest, Assignment, ASTNode, Attribute, BinaryOperation, Block, BlockSyntax, Body, BreakStatement, ClassDeclaration, ClassMember, CompoundAssignment, CompoundOperator, ConstDeclaration, Constructor, ContinueStatement, Declaration, DestructuringAssignment, EnumDeclaration, EnumVariant, Expression, Field, ForStatement, Function, FunctionCall, FunctionDeclaration, FunctionSignature, GenericType, Identifier, IfStatement, ImportItem, ImportKeyword, IndexAccess, LambdaExpression, Literal, LoopStatement, MatchArm, MatchStatement, MemberAccess, MethodCall, MethodeDeclaration, ModuleImportStatement, Mutability, Operator, Parameter, Parameters, Pattern, RangeExpression, RangePattern, ReturnStatement, SpecificImportStatement, Statement, StructDeclaration, TraitDeclaration, Type, TypeCast, UnaryOperation, UnaryOperator, VariableDeclaration, Visibility, WhileStatement};

use crate::parser::parser_error::ParserErrorType::{ExpectColon, ExpectFunctionName, ExpectIdentifier, ExpectOperatorEqual, ExpectParameterName, ExpectValue, ExpectVariableName, ExpectedCloseParenthesis, ExpectedOpenParenthesis, ExpectedTypeAnnotation, InvalidFunctionDeclaration, InvalidTypeAnnotation, InvalidVariableDeclaration, UnexpectedEOF, UnexpectedEndOfInput, UnexpectedIndentation, UnexpectedToken, ExpectedParameterName, InvalidAssignmentTarget, ExpectedDeclaration, ExpectedArrowOrBlock, ExpectedCommaOrClosingParenthesis, MultipleRestPatterns, ExpectedUseOrImport, ExpectedAlias, ExpectedRangeOperator};
use crate::parser::parser_error::{ParserError, ParserErrorType, Position};
use crate::tok::{Delimiters, Keywords, Operators, TokenType};

use num_bigint::BigInt;
use crate::parser::ast::Declaration::Variable;
//use crate::tok::TokenType::EOF;
//////////////////////Debut///////////////////////////

pub struct Parser {
    tokens: Vec<Token>, // liste des tokens genere par le lexer
    current: usize,     // index du token actuel
    syntax_mode: SyntaxMode,
    indent_level: Vec<usize>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, syntax_mode: SyntaxMode) -> Self {
        Parser {
            tokens,
            current: 0,
            syntax_mode,
            indent_level: vec![0],
        }
    }

    fn parse_program(&mut self) -> Result<ASTNode, ParserError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        Ok(ASTNode::Program(statements))
    }


    pub fn current_position(&self) -> Position {
        Position {
            index: self.current,
        }
    }

    /// fonction pour aider le parsing des blocs

    fn get_syntax_mode(&self) ->SyntaxMode{
        self.syntax_mode
    }

    fn parse_block(&mut self) -> Result<Vec<ASTNode>, ParserError> {
        match self.syntax_mode{
            SyntaxMode::Indentation => self.parse_indented_block(),
            SyntaxMode::Braces => self.parse_braced_block(),

        }
    }
    fn current_indent_level(&self) -> usize {
        if let Some(TokenType::INDENT) = self.current_token().map(|t| &t.token_type) {
            self.get_current_indent_level()
        } else {
            0
        }
    }
    pub fn get_current_indent_level(&self) -> usize {
        *self.indent_level.last().unwrap_or(&0)
    }

    fn parse_indented_block(&mut self) -> Result<Vec<ASTNode>, ParserError> {
        println!("Parsing indented block");
        self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
        self.consume(TokenType::NEWLINE)?;
        self.consume(TokenType::INDENT)?;

        let mut statements = Vec::new();
        while !self.check(&[TokenType::DEDENT, TokenType::EOF]) {
            let stmt = self.parse_statement()?;
            //self.consume(TokenType::NEWLINE)?;
            statements.push(stmt);
        }
        self.consume(TokenType::DEDENT)?;

        Ok(statements)
    }

    fn parse_braced_block(&mut self) -> Result<Vec<ASTNode>, ParserError> {
        println!("Parsing braced block");
        self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?;
        let mut statements = Vec::new();

        while !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE), TokenType::EOF]) {
            let stmt = self.parse_statement()?;

            // if !self.is_block_expression(&stmt) && !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]) {
            //     self.consume(TokenType::DELIMITER(Delimiters::SEMICOLON))?;
            // }
            //self.consume(TokenType::DELIMITER(Delimiters::SEMICOLON))?;

            statements.push(stmt);
        }
        self.consume(TokenType::DELIMITER(Delimiters::RCURBRACE))?;

        // Ok(ASTNode::Block(Block{
        //     statements,
        //     syntax_mode:BlockSyntax::Indentation,
        //     }))

        Ok(statements)
    }


    fn begin_block(&mut self) {
        // self.indent_level.push(self.current_token().unwrap().indent);
        todo!()
    }

    fn end_block(&mut self) {
        // self.indent_level.pop();
        todo!()
    }

    // fn parse_labeled_statement(&mut self) -> Result<Option<ASTNode>, ParserError> {
    //     if let Some(label_name) = self.check_for_label()? {
    //         // Après avoir consommé le label, on vérifie quelle instruction suit
    //         if self.check(&[TokenType::KEYWORD(Keywords::LOOP)]) {
    //             return self.parse_loop_statement().map(Some);
    //         } else {
    //             // Vous pouvez étendre ici pour d'autres instructions qui peuvent être labellisées
    //             return Err(ParserError::new(UnexpectedToken, self.current_position()));
    //         }
    //     }
    //
    //     // Pas de label, retourner None
    //     Ok(None)
    // }

    // fn parse_labeled_statement(&mut self) -> Result<ASTNode, ParserError> {
    //     if let Some(current) = self.peek_token() {
    //         if let Some(next) = self.peek_next_token() {
    //             if matches!(current.token_type, TokenType::IDENTIFIER { .. }) &&
    //                 matches!(next.token_type, TokenType::DELIMITER(Delimiters::COLON)) {
    //                 // Si le token suivant est 'loop', c'est un label de boucle
    //                 if let Some(third) = self.tokens.get(self.current + 2) {
    //                     if matches!(third.token_type, TokenType::KEYWORD(Keywords::LOOP)) {
    //                         return self.parse_loop_statement();
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //
    //
    // }



    pub fn parse_statement(&mut self) -> Result<ASTNode, ParserError> {
        let visibility = self.parse_visibility();

        // Cas particulier : pour la gestion de label dans les statements
        // l'utilisation de label est optionnelle et est de souhaite restreinte a etre utilise que pour les boucles


        if let Some(current) = self.peek_token() {
            if let Some(next) = self.peek_next_token() {
                if matches!(current.token_type, TokenType::IDENTIFIER { .. }) &&
                    matches!(next.token_type, TokenType::DELIMITER(Delimiters::COLON)) {
                    // Si le token suivant est 'loop', c'est un label de boucle
                    if let Some(third) = self.tokens.get(self.current + 2) {
                        if matches!(third.token_type, TokenType::KEYWORD(Keywords::LOOP)) {
                            return self.parse_loop_statement();
                        }
                    }
                }
            }
        }

        // if let Some(stmt) = self.parse_labeled_statement()? {
        //     return Ok(stmt);
        // }


        if self.check(&[TokenType::KEYWORD(Keywords::LET)]){
            self.parse_variable_declaration()
        }else if self.check(&[TokenType::KEYWORD(Keywords::FN)]) {
            let visibility = visibility.unwrap_or(Visibility::Private);
            self.parse_function_declaration(visibility)
        }else if self.check(&[TokenType::KEYWORD(Keywords::CONST)]){
            let visibility = visibility.unwrap_or(Visibility::Private);
            self.parse_const_declaration(visibility)
        }else if self.check(&[TokenType::KEYWORD(Keywords::STRUCT)]){
            let visibility = visibility.unwrap_or(Visibility::Private);
            self.parse_struct_declaration(visibility)
        }else if self.check(&[TokenType::KEYWORD(Keywords::ENUM)]){
            let visibility = visibility.unwrap_or(Visibility::Private);
            self.parse_enum_declaration(visibility)
        }else if self.check(&[TokenType::KEYWORD(Keywords::CLASS)]) {
            let visibility = visibility.unwrap_or(Visibility::Private);
            self.parse_class_declaration(visibility)
        }else if self.check(&[TokenType::KEYWORD(Keywords::TRAIT)]) {
            let visibility = visibility.unwrap_or(Visibility::Private);
            self.parse_trait_declaration(visibility)
        }else if self.check(&[TokenType::KEYWORD(Keywords::IMPL)]) {
            let visibility = visibility.unwrap_or(Visibility::Private);
            self.parse_impl_declaration(visibility)
        // }else if self.check(&[TokenType::KEYWORD(Keywords::LOOP)]){
        //     self.parse_loop_statement()
        }else if self.match_token(&[TokenType::KEYWORD(Keywords::IMPORT),TokenType::KEYWORD(Keywords::USE)]){
            self.parse_module_import_statement()
        }else if self.check(&[TokenType::KEYWORD(Keywords::RETURN)]) {
            self.parse_return_statement()
        }else if self.check(&[TokenType::KEYWORD(Keywords::IF)]){
            self.parse_if_statement()
        }else if self.check(&[TokenType::KEYWORD(Keywords::WHILE)]) {
            self.parse_while_statement()
        }else if self.check(&[TokenType::KEYWORD(Keywords::FOR)]) {
            self.parse_for_statement()
        }else if self.check(&[TokenType::KEYWORD(Keywords::MATCH)]) {
            self.parse_match_statement()
        }else if self.match_token(&[TokenType::KEYWORD(Keywords::BREAK)]){
            self.consume_seperator();
            Ok(ASTNode::Statement(Statement::Break))
            //self.parse_break_statement()

        }else if self.match_token(&[TokenType::KEYWORD(Keywords::CONTINUE)]){
            self.consume_seperator();
            Ok(ASTNode::Statement(Statement::Continue))
            //self.parse_continue_statement()
        }else {
            self.parse_expression_statement()
        }

    }

    /// fonction pour parser les expressions

    pub fn parse_expression(&mut self,precedence:u8) -> Result<Expression, ParserError> {
        println!("Début du parsing de l'expression");
        //verifier si c'est destructuration
        if self.check(&[TokenType::DELIMITER(Delimiters::LSBRACKET)]){
            return self.parse_destructuring_assignment();
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







            // left = Expression::BinaryOperation(BinaryOperation{
            //     left: Box::new(left),
            //     operator,
            //     right: Box::new(right),
            // });

        }

        println!("Fin du parsing de l'expression ");

        Ok(left)

    }

    pub fn parse_expression_statement(&mut self) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de l'expression statement");
        let expr = self.parse_expression(0);
        println!("Expression parsée : {:?}", expr);
        //self.consume(TokenType::DELIMITER(Delimiters::SEMICOLON))?;
        self.consume_seperator();
        println!("Separateur consommé");
        Ok(ASTNode::Expression(expr?))

    }

    fn parse_postfix_expression(&mut self) -> Result<Expression, ParserError> {
        let mut expr = self.parse_primary_expression()?;

        loop {
            if self.check(&[TokenType::DELIMITER(Delimiters::DOT)]){
                self.advance();
                let member_name = self.consume_identifier()?;

                if self.check(&[TokenType::DELIMITER(Delimiters::LPAR)]){
                    // Appel de méthode
                    self.advance();
                    let arguments = self.parse_arguments_list()?;
                    self.consume(TokenType::DELIMITER(Delimiters::RPAR))?;
                    println!("Arguments parsés : {:?}", arguments);
                    expr = Expression::MethodCall(MethodCall{
                        object: Box::new(expr),
                        method: member_name,
                        arguments,
                    });
                }else{
                    // Acces à un membre
                    println!("Nom du membre parsé : {}", member_name);
                    expr = Expression::MemberAccess(MemberAccess{
                        object: Box::new(expr),
                        member: member_name,
                    });
                }
            } else if self.check(&[TokenType::DELIMITER(Delimiters::LPAR)]) {
                // Appel de Fonction
                self.advance();
                let arguments = self.parse_arguments_list()?;
                self.consume(TokenType::DELIMITER(Delimiters::RPAR))?;
                println!("Arguments parsés : {:?}", arguments);
                expr = Expression::FunctionCall(FunctionCall{
                    name: Box::new(expr),
                    arguments,
                });
            } else if self.check(&[TokenType::DELIMITER(Delimiters::LSBRACKET)]) {
                //Acces à un élément d'un tableau par indice
                self.advance();
                let index = self.parse_expression(0)?;
                self.consume(TokenType::DELIMITER(Delimiters::RSBRACKET))?;
                println!("Index parsé : {:?}", index);
                expr = Expression::IndexAccess(IndexAccess{
                    array: Box::new(expr),
                    index: Box::new(index),
                });

            } else { break; }
        }
        Ok(expr)
    }

    fn parse_destructuring_assignment(&mut self) -> Result<Expression,ParserError>{
        println!("Début du parsing de l'assignation destructuree[");
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
        self.consume(TokenType::OPERATOR(Operators::EQUAL))?;
        let value = self.parse_expression(0)?;
        println!("Fin du parsing de l'assignation destructuree OK!!!!");
        Ok(Expression::DestructuringAssignment(DestructuringAssignment{
            targets,
            value: Box::new(value),
        }))
    }

    fn parse_unary_expression(&mut self) -> Result<Expression, ParserError> {
        println!("Début du parsing de l'expression unaire");
        println!("Début du parsing de l'expression unaire, current_token = {:?}", self.current_token());
        if let Some(token) = self.current_token(){
            match &token.token_type{
                TokenType::OPERATOR(Operators::MINUS) => {
                    self.advance();
                    let right = self.parse_unary_expression()?;
                    return Ok(Expression::UnaryOperation(UnaryOperation{
                        operator: UnaryOperator::Negative,
                        operand: Box::new(right),
                    }));
                }
                TokenType::OPERATOR(Operators::EXCLAMATION) => {
                    self.advance();
                    let right = self.parse_unary_expression()?;
                    return Ok(Expression::UnaryOperation(UnaryOperation{
                        operator: UnaryOperator::Not,
                        operand: Box::new(right),
                    }));
                }
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


    fn parse_primary_expression(&mut self) -> Result<Expression, ParserError> {
        println!("Début du parsing de l'expression primaire, current_token = {:?}", self.current_token());
        if let Some(token) = self.current_token() {
            let expr = match &token.token_type {
                TokenType::INTEGER { value } => {
                    let value = value.clone();
                    println!("Valeur entière parsée : {}", value);
                    self.advance();
                    Expression::Literal(Literal::Integer { value })
                }
                TokenType::FLOAT { value } => {
                    let value = *value;
                    println!("Valeur flottante parsée : {}", value);
                    self.advance();
                    Expression::Literal(Literal::Float { value })
                }
                TokenType::STRING { value, .. } => {
                    let value = value.clone();
                    println!("Valeur de chaîne parsée : {}", value);
                    self.advance();
                    Expression::Literal(Literal::String(value))
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


    fn parse_lambda_expression(&mut self) -> Result<Expression, ParserError> {
        println!("Début du parsing de l'expression lambda");
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
            //self.parse_block_expression()?
            //self.parse_body_block()?
            self.parse_block()?
        } else {
            return Err(ParserError::new(ExpectedArrowOrBlock, self.current_position()));
        };



        Ok(Expression::LambdaExpression(LambdaExpression{
            parameters,
            return_type: Some(return_type),
            body,
        }))

    }


    // fn parse_range_expression(&mut self) -> Result<Expression, ParserError> {
    //     println!("Début du parsing de l'expression de Range");
    //     let left = if !self.match_token(&[
    //         TokenType::OPERATOR(Operators::DOTDOT),
    //         TokenType::OPERATOR(Operators::DOTDOTEQUAL),
    //     ]) {
    //         Some(Box::new(self.parse_term()?))
    //     } else {
    //         None
    //     };
    //
    //     // Vérifie et consomme l'opérateur de plage
    //     let operator = if self.match_token(&[
    //         TokenType::OPERATOR(Operators::DOTDOT),
    //         TokenType::OPERATOR(Operators::DOTDOTEQUAL),
    //     ]) {
    //         let token = self.advance_token(); // Consomme le token
    //         token.token_type
    //     } else {
    //         return Err(ParserError::new(ExpectedRangeOperator, self.current_position()));
    //     };
    //
    //     // Parse l'expression de droite si elle existe
    //     let right = if !self.check(TokenType::DELIMITER(Delimiters::SEMICOLON)) && !self.check(TokenType::EOF) {
    //         Some(Box::new(self.parse_term()?))
    //     } else {
    //         None
    //     };
    //
    //     Ok(Expression::RangeExpression(RangeExpression {
    //         left,
    //         operator,
    //         right,
    //     }))
    // }


    /// fonction pour parser les parametres

    fn parse_arguments_list(&mut self) -> Result<Vec<Expression>, ParserError> {
        println!("Début du parsing de la liste d'arguments");
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
        println!("Arguments liste parsés : {:?}", arguments);
        Ok(arguments)

    }

    fn parse_parameter_list(&mut self) -> Result<Vec<Parameter>, ParserError> {
        println!("Début du parsing de la liste des paramètres");
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

    fn parse_function_parameters(&mut self) -> Result<Vec<Parameter>, ParserError> {
        println!("Début du parsing des paramètres de fonction");
        let mut parameters = Vec::new();

        if self.check(&[TokenType::DELIMITER(Delimiters::RPAR)]){
            // pas de paramètres
            return Ok(parameters);
        }

        if !self.match_token(&[TokenType::DELIMITER(Delimiters::RPAR)]) {
            loop {
                //let name = self.consume_parameter_name()?;
                let name = self.consume_identifier()?;
                println!("Nom du paramètre parsé : {}", name);
                self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
                let param_type = self.parse_type()?;
                println!("Type du paramètre parsé : {:?}", param_type);

                parameters.push(Parameter { name, parameter_type: param_type });

                if self.match_token(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
                    continue;
                } else if self.check(&[TokenType::DELIMITER(Delimiters::RPAR)]) {
                    break;
                }else {
                    println!("Erreur lors du parsing des paramètres, token actuel : {:?}", self.current_token());
                    return Err(ParserError::new(ExpectedParameterName, self.current_position()));
                }
            }
        }
        println!("Paramètres parsés : {:?}", parameters);
        Ok(parameters)
    }

    fn parse_function_body(&mut self) -> Result<Vec<ASTNode>, ParserError> {
        let mut body = Vec::new();

        match self.syntax_mode {
            SyntaxMode::Braces => {
                self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?;
                while !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]) {
                    let statement = self.parse_statement()?;
                    body.push(statement);
                }
                self.consume(TokenType::DELIMITER(Delimiters::RCURBRACE))?;
            }
            SyntaxMode::Indentation => {
                // Consommer le NEWLINE initial
                self.consume(TokenType::NEWLINE)?;
                self.consume(TokenType::INDENT)?;

                while !self.check(&[TokenType::EOF, TokenType::DEDENT]) {
                    let statement = self.parse_statement()?;
                    body.push(statement);
                }
                //consommer le DEDENT final s'il existe
                if !self.check(&[TokenType::DEDENT]) {
                    self.consume(TokenType::DEDENT)?;
                }
            }
        }

        Ok(body)
    }

    fn parse_body_block(&mut self) -> Result<Vec<ASTNode>,ParserError>{
        println!("Début du parsing du corps");
        self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?;
        let mut statements = Vec::new();
        while !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]) && !self.is_at_end() {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
        }
        self.consume(TokenType::DELIMITER(Delimiters::RCURBRACE))?;
        println!("Fin du parsing du corps OK!!!!!!!!!!!!");
        Ok(statements)
    }

    fn parse_block_expression(&mut self) -> Result<Vec<ASTNode>,ParserError>{
        println!("Debut du parsing de du bloc de L'expression LAMBDA");
        self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?;

        let mut body = Vec::new();
        while !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]){
            let statement = self.parse_statement()?;
            body.push(statement);
        }
        self.consume(TokenType::DELIMITER(Delimiters::RCURBRACE))?;
        println!("Fin du parsing du bloc de l'expression LAMBDA OK!!!!!!!!!!!");
        Ok(body)

    }


    /// fonction pour parser les declarations
    // fonction tranfere a parse_statement()

    // pub fn parse_declaration(&mut self) -> Result<ASTNode, ParserError> {
    //     let visibility = self.parse_visibility()?;
    //
    //     if self.check(&[TokenType::KEYWORD(Keywords::LET)]) {
    //         self.parse_variable_declaration()
    //     } else if self.check(&[TokenType::KEYWORD(Keywords::CONST)]) {
    //         self.parse_const_declaration(visibility)
    //     } else if self.check(&[TokenType::KEYWORD(Keywords::FN)]) {
    //         self.parse_function_declaration(visibility)
    //     } else if self.check(&[TokenType::KEYWORD(Keywords::STRUCT)]) {
    //         self.parse_struct_declaration(visibility)
    //     } else if self.check(&[TokenType::KEYWORD(Keywords::ENUM)]) {
    //         self.parse_enum_declaration(visibility)
    //     } else if self.check(&[TokenType::KEYWORD(Keywords::TRAIT)]) {
    //         self.parse_trait_declaration(visibility)
    //     } else if self.check(&[TokenType::KEYWORD(Keywords::CLASS)]) {
    //         self.parse_class_declaration(visibility)
    //     } else if self.check(&[TokenType::KEYWORD(Keywords::IMPL)]) {
    //         self.parse_impl_declaration()
    //     } else {
    //         Err(ParserError::new(ExpectedDeclaration, self.current_position()))
    //     }
    // }


    /// fonction pour parser les déclarations de variables
    /// Exemple: Brace Mode
    /// // let mut x: int = 5;
    /// // let y: float = 3.14;
    /// // let z = 42;
    /// // let a:bool = true;
    /// Exemple: Indentation Mode
    /// // let mut x: int = 5
    /// // let y: float = 3.14
    /// // let z = 42
    /// // let a:bool = true


    pub fn parse_variable_declaration(&mut self) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de la déclaration de variable");
        self.consume(TokenType::KEYWORD(Keywords::LET))?;

        let mutability = self.parse_mutability()?;

        let  name = self.consume_identifier()?;
        println!("Nom de la variable parsé : {}", name);
        let variable_type = if self.match_token(&[TokenType::DELIMITER(Delimiters::COLON)]) {
            self.parse_type()?

        } else {
            Type::Infer
        };
        println!("Type de la variable parsé : {:?}", variable_type);

        println!("Debut de la valeur de la variable");
        self.consume(TokenType::OPERATOR(Operators::EQUAL))?;

        let value = self.parse_expression(0)?;


        self.consume_seperator();
        println!("Valeur de la variable parsée : {:?}", value);

        Ok(ASTNode::Declaration(Declaration::Variable(VariableDeclaration{
            name,
            variable_type: Some(variable_type),
            value: Some(value),
            mutability,
        })))

    }


    pub fn parse_const_declaration(&mut self, visibility: Visibility) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de la déclaration de constante");

        //let visibility = self.parse_visibility()?;

        self.consume(TokenType::KEYWORD(Keywords::CONST))?;

        let name = self.consume_identifier()?;
        let variable_type = if self.match_token(&[TokenType::DELIMITER(Delimiters::COLON)]) {
            self.parse_type()?
        } else {
            Type::Infer
        };
        self.consume(TokenType::OPERATOR(Operators::EQUAL))?;
        let value = self.parse_expression(0)?;
        self.consume_seperator();

        println!("la valeur de la constante parse : {:?}", value);

        Ok(ASTNode::Declaration(Declaration::Constante(ConstDeclaration{
            name,
            constant_type: Some(variable_type),
            value,
            visibility,
        })))

    }

    pub fn parse_function_declaration(&mut self, visibility: Visibility) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de la déclaration de fonction");
        self.consume(TokenType::KEYWORD(Keywords::FN))?;
        let name = self.consume_identifier()?;
        println!("Nom de la fonction parsé : {}", name);

        self.consume(TokenType::DELIMITER(Delimiters::LPAR))?;

        let parameters = self.parse_function_parameters()?;

        self.consume(TokenType::DELIMITER(Delimiters::RPAR))?;

        let return_type = if self.match_token(&[TokenType::OPERATOR(Operators::RARROW)]) {
            self.parse_type()?
        } else {
            Type::Infer // Ou un type par défaut
        };

        if self.syntax_mode == SyntaxMode::Indentation{
            self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
        }

        let body = self.parse_function_body()?;

        // self.consume_seperator();  plus de ; apres une fonction

        Ok(ASTNode::Declaration(Declaration::Function(FunctionDeclaration {
            name,
            parameters,
            return_type: Some(return_type),
            body,
            visibility,
        })))
    }

    pub fn parse_struct_declaration(&mut self, visibility: Visibility) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de la déclaration de structure");


        self.consume(TokenType::KEYWORD(Keywords::STRUCT))?;
        let name = self.consume_identifier()?;
        println!("Nom de la structure parsé : {}", name);

        // // on vas implementer le type generique si on as un <
        // let generic_type = if self.match_token(&[TokenType::OPERATOR(Operators::LESS)]){
        //     self.parse_gen_type_param()?;
        // }else {
        //     vec![]
        //
        // };



        self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?;

        let fields = self.parse_struct_fields()?;
        self.consume(TokenType::DELIMITER(Delimiters::RCURBRACE))?;

        // self.consume_seperator();

        Ok(ASTNode::Declaration(Declaration::Structure(StructDeclaration{
            name,
            // generic_type,
            fields,
            visibility,
        })))

    }

    fn parse_enum_declaration(&mut self, visibility: Visibility) -> Result<ASTNode, ParserError> {
        println!("Debut du parsing de la déclaration d'énumération");
        self.consume(TokenType::KEYWORD(Keywords::ENUM))?;
        let name = self.consume_identifier()?;
        println!("Nom de l'énumération parsé : {}", name);
        self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?;
        let variantes = self.parse_enum_variantes()?;
        self.consume(TokenType::DELIMITER(Delimiters::RCURBRACE))?;

        // self.consume_seperator();

        println!("Variantes d'énumération parsées OK!!!!!!!!!!!!!!!!!!!!!!");
        Ok(ASTNode::Declaration(Declaration::Enum(EnumDeclaration{
            name,
            variantes,
            visibility,
        })))

    }

    fn parse_trait_declaration(&mut self, visibility: Visibility) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de la déclaration de trait");
        self.consume(TokenType::KEYWORD(Keywords::TRAIT))?;
        let name = self.consume_identifier()?;
        println!("Nom du trait parsé : {}", name);

        todo!()
    }

    fn parse_impl_declaration(&mut self,visibility: Visibility) -> Result<ASTNode, ParserError> {
        todo!()
    }

    fn parse_class_declaration(&mut self, visibility: Visibility) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de la déclaration de classe");
        self.consume(TokenType::KEYWORD(Keywords::CLASS))?;

        let name = self.consume_identifier()?;
        println!("Nom de la classe parsé : {}", name);
        let parent_classes = self.parse_class_inheritance()?;

        match self.syntax_mode{
            SyntaxMode::Indentation => self.consume(TokenType::DELIMITER(Delimiters::COLON))?,
            SyntaxMode::Braces => self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?,
        }

        let (attributes ,methods)= self.parse_class_body()?;

        println!("Attributs de la classe parsés : {:?} et {:?}", attributes,methods);

        Ok(ASTNode::Declaration(Declaration::Class(ClassDeclaration{
            name,
            parent_classes,
            attributes,
            methods,
            visibility,

        })))

    }
    fn parse_class_inheritance(&mut self) -> Result<Vec<String>,ParserError>{
        let mut parent_classes = Vec::new();
        if self.check(&[TokenType::DELIMITER(Delimiters::LPAR)]){
            self.consume(TokenType::DELIMITER(Delimiters::LPAR))?;
            loop {
                let parent = self.consume_identifier()?;
                parent_classes.push(parent.clone());
                if !self.match_token(&[TokenType::DELIMITER(Delimiters::COMMA)]){
                    break;
                }
            }
            self.consume(TokenType::DELIMITER(Delimiters::RPAR))?;
        }
        println!("Classes parentes parsées : {:?}", parent_classes);
        Ok(parent_classes)
    }

    pub fn parse_class_body(&mut self) -> Result<(Vec<Attribute>,Vec<MethodeDeclaration>), ParserError>{
        let mut attributes = Vec::new();
        let mut methods = Vec::new();

        match self.syntax_mode{
            SyntaxMode::Braces => {
                // self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?;
                while !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]) && !self.is_at_end(){
                    if self.check(&[TokenType::KEYWORD(Keywords::LET)]){
                        let attribute = self.parse_attribute_declaration()?;
                        attributes.push(attribute);
                    }else if self.check(&[TokenType::KEYWORD(Keywords::FN)]){
                        let method = self.parse_methode_declaration()?;
                        methods.push(method);
                    }else {
                        return Err(ParserError::new(UnexpectedToken, self.current_position()));
                    }
                }
                self.consume(TokenType::DELIMITER(Delimiters::RCURBRACE))?;
            }
            SyntaxMode::Indentation => {
                self.consume(TokenType::NEWLINE)?;
                self.consume(TokenType::INDENT)?;
                while !self.check(&[TokenType::EOF, TokenType::DEDENT]) && !self.is_at_end(){
                    if self.check(&[TokenType::KEYWORD(Keywords::FN)]){
                        let method = self.parse_methode_declaration()?;
                        methods.push(method);
                    }else if self.check(&[TokenType::KEYWORD(Keywords::LET)]){
                        let attribute = self.parse_attribute_declaration()?;
                        attributes.push(attribute);
                    }else {
                        return Err(ParserError::new(UnexpectedToken, self.current_position()));
                    }
                }
                if !self.check(&[TokenType::DEDENT]){
                    self.consume(TokenType::DEDENT)?;
                }
            }
        }
        Ok((attributes,methods))

    }


    fn parse_attribute_declaration(&mut self) -> Result<Attribute, ParserError> {
        println!("Début du parsing de la déclaration de méthode");
        let visibility = self.parse_visibility()?;
        self.consume(TokenType::KEYWORD(Keywords::LET))?;
        let mutability = self.parse_mutability()?;

        let name = self.consume_identifier()?;
        self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
        let attribute_type = self.parse_type()?;
        self.consume_seperator();
        println!("Parsing de la déclaration de méthode OK!!!!!!!!!!!!!!!!!!!!!!!");

        Ok(Attribute{
            name,
            attr_type: attribute_type,
            // value: Some(value),
            visibility,
            mutability
        })

    }
    fn parse_methode_declaration(&mut self) -> Result<MethodeDeclaration,ParserError>{
        println!("Debut du parsing de la déclaration de méthode");
        let visibility = self.parse_visibility()?;

        self.consume(TokenType::KEYWORD(Keywords::FN))?;

        let name = self.consume_identifier()?;
        self.consume(TokenType::DELIMITER(Delimiters::LPAR))?;
        let parameters = self.parse_function_parameters()?;
        self.consume(TokenType::DELIMITER(Delimiters::RPAR))?;

        let return_type = if self.match_token(&[TokenType::OPERATOR(Operators::RARROW)]){
            self.parse_type()?
        }else { Type::Infer };
        if self.syntax_mode == SyntaxMode::Indentation{
            self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
        }
        let body = self.parse_function_body()?;
        self.consume_seperator();
        println!("Fin du parsing de la déclaration de méthode OK!!!!!!!!!!!!!!!!!!!!!!!");

        Ok(MethodeDeclaration{
            name,
            parameters,
            return_type: Some(return_type),
            body,
            visibility,
        })
    }



    /// fonction pour parser les types
    fn parse_type(&mut self) -> Result<Type, ParserError> {
        let token = self
            .current_token()
            .ok_or_else(|| ParserError::new(ExpectedTypeAnnotation, self.current_position()))?;

        println!("Parsing type: {:?}", token);

        match &token.token_type {
            TokenType::KEYWORD(Keywords::INT) => {
                self.advance(); // Consomme le token `int`
                Ok(Type::Int)
            }
            TokenType::KEYWORD(Keywords::FLOAT) => {
                self.advance(); // Consomme le token `float`
                Ok(Type::Float)
            }
            TokenType::KEYWORD(Keywords::BOOL) => {
                self.advance(); // Consomme le token `bool`
                Ok(Type::Bool)
            }
            TokenType::KEYWORD(Keywords::STR) => {
                self.advance(); // Consomme le token `string`
                Ok(Type::String)
            }
            TokenType::KEYWORD(Keywords::CHAR) => {
                self.advance(); // Consomme le token `char`
                Ok(Type::Char)
            }
            _ => {
                println!("Unexpected token: {:?}", token);
                // Si le token actuel n'est pas un type valide, renvoyer une erreur
                Err(ParserError::new(
                    InvalidTypeAnnotation,
                    self.current_position(),
                ))
            }
        }
    }

    fn parse_type_cast(&mut self,expr: Expression) -> Result<Expression, ParserError> {
        todo!()
    }

    // fn parse_gen_type_param(&mut self) -> Result<Vec<String>,ParserError> {
    //     let mut params = Vec::new();
    //
    //     loop {
    //         let param_type = self.consume_identifier()?;
    //         params.push(param_type);
    //
    //         if self.match_token(&[TokenType::DELIMITER(Delimiters::COMMA)]){
    //             continue;
    //         }else { break; }
    //     }
    //     Ok(params)
    // }
    //
    //
    // fn parse_generic_type(&mut self) -> Result<GenericType, ParserError> {
    //     let base_name = self.consume_identifier()?;
    //
    //     if self.match_token(&[TokenType::OPERATOR(Operators::LESS)]){
    //         let type_parameters = self.parse_type_parameters()?;
    //         self.consume(TokenType::OPERATOR(Operators::GREATER))?;
    //
    //         Ok(Type::Generic(GenericType{
    //             base: base_name,
    //             type_parameters,
    //         }))
    //     }
    //
    // }

    /// fonction  pour parser la mutabilité et la visibilité
    fn parse_mutability(&mut self) -> Result<Mutability, ParserError> {
        if self.match_token(&[TokenType::KEYWORD(Keywords::MUT)]){
            Ok(Mutability::Mutable)
        } else {
            Ok(Mutability::Immutable)
        }
    }
    fn parse_visibility(&mut self) -> Result<Visibility, ParserError> {
        if self.match_token(&[TokenType::KEYWORD(Keywords::PUB)]){
            Ok(Visibility::Public)
        } else {
            Ok(Visibility::Private)
        }
    }

    ///fonction pour parser les champs de structure STRUCT

    fn parse_struct_fields(&mut self) -> Result<Vec<Field>, ParserError> {
        println!("Début du parsing des champs de structure");
        let mut fields = Vec::new();

        if self.match_token(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]){
            return Ok(fields)
        }
        // ici  on  gere au cas ou on as  une structure vide
        if self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]){
            return Ok(fields)
        }
        loop {
            let field = self.parse_struct_field()?;
            fields.push(field);
            if self.match_token(&[TokenType::DELIMITER(Delimiters::COMMA)]){
                // ne pas exiger de NEWLINE après la virgule en mode indentation
                let _ = self.match_token(&[TokenType::NEWLINE]);
                // if self.syntax_mode == SyntaxMode::Indentation{
                //     self.consume(TokenType::NEWLINE)?;
                // }
                //continue;
            } else if self.match_token(&[TokenType::NEWLINE])  && self.syntax_mode==SyntaxMode::Indentation{

            } else if self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]){
                break;
            } else {
                return Err(ParserError::new(ExpectColon,self.current_position()))
            }
        }
        println!("Champs de structure parsés : {:?}", fields);
        Ok(fields)

    }
    fn parse_struct_field(&mut self) -> Result<Field, ParserError> {
        let visibility = self.parse_visibility()?;
        println!("Visibilité du champ parsée : {:?}", visibility);
        let name = self.consume_identifier()?;
        println!("Nom du champ parsé : {}", name);
        self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
        let field_type = self.parse_type()?;
        println!("Type du champ parsé : {:?}", field_type);
        Ok(Field{
            name,
            field_type,
            visibility

        })

    }

    fn parse_enum_variantes(&mut self) -> Result<Vec<EnumVariant>,ParserError>{
        println!("Début du parsing des variantes d'énumération");
        let mut variantes = Vec::new();
        if self.match_token(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]){
            return Ok(variantes)
        }
        if self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]){
            return Ok(variantes)
        }
        loop{
            let variante = self.parse_enum_variant_fields()?;
            variantes.push(variante);
            if self.match_token(&[TokenType::DELIMITER(Delimiters::COMMA)]){

                let _ = self.match_token(&[TokenType::NEWLINE]);

            }else if self.match_token(&[TokenType::NEWLINE]) && self.syntax_mode == SyntaxMode::Indentation{

            }else if self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]){
                break;
            } else {
                return Err(ParserError::new(ExpectColon,self.current_position()))
            }
        }
        println!("Variantes d'énumération parsées : {:?}", variantes);
        Ok(variantes)
    }

    fn parse_enum_variant_fields(&mut self) ->  Result<EnumVariant,ParserError>{
        let visibility = self.parse_visibility()?;
        println!("Visibilité de la variante parsée : {:?}", visibility);
        let name = self.consume_identifier()?;
        println!("Nom de la variante parsée : {}", name);
        self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
        let variante_type = self.parse_type()?;
        println!("Type de la variante parsée : {:?}", variante_type);
        Ok(EnumVariant{
            name,
            variante_type,
            visibility
        })

    }


    /// fonction pour le gestion de structure de controle
    fn parse_if_statement(&mut self) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de l'instruction if");

        self.consume(TokenType::KEYWORD(Keywords::IF))?;
        let condition = self.parse_expression(0)?;
        //let then_block = self.parse_body_block()?;; // block_expression
        let then_block = self.parse_block()?;

        let else_block = if self.check(&[TokenType::KEYWORD(Keywords::ELIF)]){
            self.advance();
            let elif_statement = self.parse_if_statement()?;
            Some(vec![elif_statement])
        }else if self.match_token(&[TokenType::KEYWORD(Keywords::ELSE)]){
            //Some(self.parse_body_block()?)
            Some(self.parse_block()?)
        }else { None };

        println!("Fin du parsing de l'instruction if");
        Ok(ASTNode::Statement(Statement::IfStatement(IfStatement{
            condition,
            then_block,
            else_block,
        })))

    }
    fn parse_while_statement(&mut self) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de l'instruction while");

        self.consume(TokenType::KEYWORD(Keywords::WHILE))?;

        let condition = self.parse_expression(0)?;
        let body = self.parse_body_block()?;
        println!("Fin du parsing de l'instruction while OK!!!!!!!!!!!!!!");
        Ok(ASTNode::Statement(Statement::WhileStatement(WhileStatement{
            condition,
            body,
        })))

    }

    fn parse_loop_statement(&mut self) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de l'instruction loop");

        // ajoute de label optional pour la boucle pour
        let label = self.check_for_label()?;

        self.consume(TokenType::KEYWORD(Keywords::LOOP))?;
        let body = self.parse_block()?;
        println!("Fin du parsing de l'instruction loop OK!!!!!!!!!!!!!!");
        Ok(ASTNode::Statement(Statement::LoopStatement(LoopStatement{
            label,
            body,
        })))
    }


    fn parse_for_statement(&mut self) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de l'instruction for");

        self.consume(TokenType::KEYWORD(Keywords::FOR))?;

        let iterator = self.consume_identifier()?;
        self.consume(TokenType::KEYWORD(Keywords::IN))?;
        let iterable = self.parse_expression(0)?;
        let body = self.parse_body_block()?;
        println!("Fin du parsing de l'instruction for OK!!!!!!!!!!!!!!!");
        Ok(ASTNode::Statement(Statement::ForStatement(ForStatement{
            iterator,
            iterable,
            body
        })))

    }


    fn parse_break_statement(&mut self) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de l'instruction break");
        self.consume(TokenType::KEYWORD(Keywords::BREAK))?;
        let label = self.check_for_label()?;
        self.consume_seperator();
        println!("Fin du parsing de l'instruction break OK!!!!!!!!!!!!!!!");
        Ok(ASTNode::Statement(Statement::BreakStatement(BreakStatement{
            label
        })))
    }

    fn parse_continue_statement(&mut self) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de l'instruction continue");
        self.consume(TokenType::KEYWORD(Keywords::CONTINUE))?;
        let label = self.check_for_label()?;
        self.consume_seperator();
        println!("Fin du parsing de l'instruction continue OK!!!!!!!!!!!!!!!");
        Ok(ASTNode::Statement(Statement::ContinueStatement(ContinueStatement{
            label
        })))
    }


    pub fn parse_match_statement(&mut self) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de l'instruction match");
        self.consume(TokenType::KEYWORD(Keywords::MATCH))?;
        let match_expr = self.parse_expression(0)?;

        let mut arms = Vec::new();
        if self.syntax_mode == SyntaxMode::Indentation {
            self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
            self.consume(TokenType::NEWLINE)?;
            self.consume(TokenType::INDENT)?;

            // En mode indentation, on continue jusqu'au DEDENT
            while !self.check(&[TokenType::DEDENT]) && !self.is_at_end() {
                let arm = self.parse_match_arm()?;
                arms.push(arm);
            }

            self.consume(TokenType::DEDENT)?;
        } else {
            self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?;

            while !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]) && !self.is_at_end() {
                let arm = self.parse_match_arm()?;
                arms.push(arm);
            }

            self.consume(TokenType::DELIMITER(Delimiters::RCURBRACE))?;
        }

        println!("Fin du parsing de l'instruction match OK!!!!!!!!!!!!!!");
        Ok(ASTNode::Statement(Statement::MatchStatement(MatchStatement{
            expression: match_expr,
            arms,
        })))

    }
    fn is_end_of_match(&self) -> bool {
        match self.syntax_mode {
            SyntaxMode::Indentation => self.check(&[TokenType::DEDENT]),
            SyntaxMode::Braces => self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]),
        }
    }

    fn is_block_expression(&self,node: &ASTNode) -> bool {
        matches!(node,
            ASTNode::Statement(Statement::IfStatement(_)) |
            ASTNode::Statement(Statement::LoopStatement(_)) |
            ASTNode::Statement(Statement::WhileStatement(_))|
            ASTNode::Expression(Expression::LambdaExpression(_))
        )
    }



    fn parse_indented_arm_body(&mut self) -> Result<Vec<ASTNode>, ParserError> {
        // On vérifie si on utilise => ou : pour ce bras
        let uses_arrow = self.check(&[TokenType::OPERATOR(Operators::FATARROW)]);

        if uses_arrow {
            // Style avec =>
            self.consume(TokenType::OPERATOR(Operators::FATARROW))?;
            let expr = self.parse_expression(0)?;
            self.consume(TokenType::NEWLINE)?;
            Ok(vec![ASTNode::Expression(expr)])
        } else {
            // Style avec :
            self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
            self.consume(TokenType::NEWLINE)?;
            self.consume(TokenType::INDENT)?;

            let mut body = Vec::new();
            while !self.check(&[TokenType::DEDENT]) && !self.is_at_end() {
                let expr = self.parse_expression(0)?;
                body.push(ASTNode::Expression(expr));
                self.consume(TokenType::NEWLINE)?;
            }

            self.consume(TokenType::DEDENT)?;
            Ok(body)
        }
    }


    fn parse_braced_arm_body(&mut self) -> Result<Vec<ASTNode>, ParserError> {
        self.consume(TokenType::OPERATOR(Operators::FATARROW))?;

        let body = if self.check(&[TokenType::DELIMITER(Delimiters::LCURBRACE)]) {
            // Corps avec bloc
            self.parse_body_block()?
            //self.parse_block()? a test  plus tard
        } else {
            // Expression simple
            let expr = self.parse_expression(0)?;
            vec![ASTNode::Expression(expr)]
        };

        // Consomme la virgule si ce n'est pas le dernier bras
        if !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]) {
            self.consume(TokenType::DELIMITER(Delimiters::COMMA))?;
        }
        Ok(body)
    }

    fn parse_guard(&mut self) -> Result<Option<Box<Expression>>, ParserError> {
        if self.match_token(&[TokenType::KEYWORD(Keywords::IF)]){
            let condition = self.parse_expression(0)?;
            Ok(Some(Box::new(condition)))
        }else {
            Ok(None)
        }
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm, ParserError> {
        println!("Début du parsing du bras de match");
        let pattern = self.parse_pattern_complex()?;

        let guard = self.parse_guard()?;

        let body= if self.syntax_mode == SyntaxMode::Indentation{
            self.parse_indented_arm_body()?
        }else {
            self.parse_braced_arm_body()?
        };
        println!("Fin du parsing du bras de match OK!!!!!!!!!!!!!!!");
        Ok(MatchArm{
            pattern,
            guard,
            body,
        })
    }

    fn parse_pattern_complex(&mut self) -> Result<Pattern, ParserError>{
        if self.check(&[TokenType::DELIMITER(Delimiters::DOT)]){
            self.consume(TokenType::DELIMITER(Delimiters::DOT))?;
            self.consume(TokenType::DELIMITER(Delimiters::DOT))?;
            Ok(Pattern::Rest)
        }else if self.check(&[TokenType::DELIMITER(Delimiters::LPAR)]){
            self.parse_tuple_pattern()
        }else if self.check(&[TokenType::DELIMITER(Delimiters::LSBRACKET)]){
            self.parse_array_pattern()
        }else { self.parse_pattern() }

    }

    fn parse_tuple_pattern(&mut self) -> Result<Pattern, ParserError> {
        self.consume(TokenType::DELIMITER(Delimiters::LPAR))?;
        let mut patterns = Vec::new();
        if !self.check(&[TokenType::DELIMITER(Delimiters::RPAR)]){
            loop {
                let pattern = self.parse_pattern_complex()?;
                patterns.push(pattern);
                if self.check(&[TokenType::DELIMITER(Delimiters::COMMA)]){
                    self.consume(TokenType::DELIMITER(Delimiters::COMMA))?;
                }else { break; }
            }
        }
        self.consume(TokenType::DELIMITER(Delimiters::RPAR))?;
        println!("Fin du parsing du tuple pattern OK!!!!!!!!!!!!!!!");
        Ok(Pattern::Tuple(patterns))
    }

    //feature pour plus tard
    fn parse_tuple_rest_pattern(&mut self) -> Result<Pattern, ParserError> {
        self.consume(TokenType::DELIMITER(Delimiters::LPAR))?;
        let mut patterns = Vec::new();
        let mut has_rest = false;

        while !self.check(&[TokenType::DELIMITER(Delimiters::RPAR)]) {
            if self.check(&[TokenType::DELIMITER(Delimiters::DOT)]) {
                if has_rest {
                    return Err(ParserError::new(MultipleRestPatterns, self.current_position()));
                }
                self.consume(TokenType::DELIMITER(Delimiters::DOT))?;
                self.consume(TokenType::DELIMITER(Delimiters::DOT))?;
                has_rest = true;
            } else {
                patterns.push(self.parse_pattern()?);
            }

            if self.check(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
                self.consume(TokenType::DELIMITER(Delimiters::COMMA))?;
            } else {
                break;
            }
        }

        self.consume(TokenType::DELIMITER(Delimiters::RPAR))?;
        Ok(Pattern::TupleRest(patterns))
    }

    fn parse_array_pattern(&mut self) -> Result<Pattern, ParserError> {
        println!("Début du parsing du pattern de tableau Array");
        self.consume(TokenType::DELIMITER(Delimiters::LSBRACKET))?;
        let mut patterns = Vec::new();
        if !self.check(&[TokenType::DELIMITER(Delimiters::RSBRACKET)]){
            loop {
                let pattern = self.parse_pattern_complex()?;
                patterns.push(pattern);
                if self.check(&[TokenType::DELIMITER(Delimiters::COMMA)]){
                    self.consume(TokenType::DELIMITER(Delimiters::COMMA))?;
                }else { break; }
            }
        }
        self.consume(TokenType::DELIMITER(Delimiters::RSBRACKET))?;
        println!("Fin du parsing du pattern de tableau Array OK!!!!!!!!!!!!!!!");
        Ok(Pattern::Array(patterns))

    }

    //feature pour plus tard
    fn parse_array_rest_pattern(&mut self) -> Result<Pattern, ParserError> {
        self.consume(TokenType::DELIMITER(Delimiters::LSBRACKET))?;
        let mut before = Vec::new();
        let mut after = Vec::new();
        let mut has_rest = false;

        while !self.check(&[TokenType::DELIMITER(Delimiters::RSBRACKET)]) {
            if self.check(&[TokenType::DELIMITER(Delimiters::DOT)]) {
                if has_rest {
                    return Err(ParserError::new(MultipleRestPatterns, self.current_position()));
                }
                self.consume(TokenType::DELIMITER(Delimiters::DOT))?;
                self.consume(TokenType::DELIMITER(Delimiters::DOT))?;
                has_rest = true;
            } else {
                let pattern = self.parse_pattern()?;
                if has_rest {
                    after.push(pattern);
                } else {
                    before.push(pattern);
                }
            }

            if self.check(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
                self.consume(TokenType::DELIMITER(Delimiters::COMMA))?;
            } else {
                break;
            }
        }

        self.consume(TokenType::DELIMITER(Delimiters::RSBRACKET))?;
        Ok(Pattern::ArrayRest(ArrayRest{
            before,
            after,
        }))
    }

    fn parse_range_pattern(&mut self) -> Result<Pattern, ParserError> {
        let start = if !self.check(&[TokenType::DELIMITER(Delimiters::DOT)]) {
            Some(Box::new(self.parse_expression(0)?))
        } else {
            None
        };

        // Consomme le premier point ; Update avec nouveau  Token DOT
        self.consume(TokenType::DELIMITER(Delimiters::DOT))?;
        self.consume(TokenType::DELIMITER(Delimiters::DOT))?;

        // L'expression finale si elle existe
        let end = if !self.check(&[TokenType::OPERATOR(Operators::FATARROW)]) &&
            !self.check(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
            Some(Box::new(self.parse_expression(0)?))
        } else {
            None
        };

        Ok(Pattern::RangePattern(RangePattern{
            start,
            end,
            inclusive: false  // Par défaut, on utilise la range exclusive
        }))
    }

    fn is_start_of_range(&self) -> bool {
        todo!()

    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParserError> {
        println!("Début du parsing du pattern");


        if self.match_token(&[TokenType::OPERATOR(Operators::UNDERSCORE)]) {
            // Pattern par défaut '_'
            Ok(Pattern::Wildcard)
        } else if let Some(token) = self.current_token() {
            match &token.token_type {
                TokenType::IDENTIFIER { name } => {
                    if name == "_" {
                        self.advance();
                        Ok(Pattern::Wildcard)
                    } else {
                        let identifier = name.clone();
                        self.advance();
                        Ok(Pattern::Identifier(identifier))
                    }
                },
                TokenType::INTEGER { value } => {
                    let int_value = value.clone(); // Clonez la valeur ici
                    self.advance(); // Consomme l'entier
                    Ok(Pattern::Literal(Literal::Integer { value: int_value }))
                },
                TokenType::FLOAT { value } => {
                    let float_value = *value;
                    self.advance(); // Consomme le flottant
                    Ok(Pattern::Literal(Literal::Float { value: float_value }))
                },
                TokenType::STRING { value, kind: _ } => {
                    let string_value = value.clone();
                    self.advance(); // Consomme la chaîne
                    Ok(Pattern::Literal(Literal::String(string_value)))
                }
                TokenType::KEYWORD(Keywords::TRUE) => {
                    self.advance(); // Consomme le mot-clé 'true'
                    Ok(Pattern::Literal(Literal::Boolean(true)))
                },
                TokenType::KEYWORD(Keywords::FALSE) => {
                    self.advance(); // Consomme le mot-clé 'false'
                    Ok(Pattern::Literal(Literal::Boolean(false)))
                },


                _ => Err(ParserError::new(UnexpectedToken, self.current_position())),
            }
        } else {
            Err(ParserError::new(UnexpectedEndOfInput, self.current_position()))
        }
    }






    fn parse_return_statement(&mut self) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de l'instruction de retour");
        self.consume(TokenType::KEYWORD(Keywords::RETURN))?;
        let value = if !self.match_token(&[TokenType::NEWLINE, TokenType::DEDENT, TokenType::EOF]) {
            Some(self.parse_expression(0)?)
        } else {
            None
        };
        println!("Valeur de retour parsée : {:?}", value);
        println!("Fin du parsing de l'instruction de retour OK!!!!!!!!!!!!!!");
        Ok(ASTNode::Statement(Statement::ReturnStatement(ReturnStatement{
            value,
        })))

    }


    /// fonction pour la gestion des emprunts
    fn parse_borrow(&mut self) -> Result<Expression, ParserError> {
        if self.match_token(&[TokenType::OPERATOR(Operators::AMPER)]){
            let mutable = self.match_token(&[TokenType::KEYWORD(Keywords::MUT)]);
            let expression = self.parse_expression(0)?;
            Ok(Expression::UnaryOperation(UnaryOperation{
                operator: if mutable { UnaryOperator::ReferenceMutable} else {UnaryOperator::Reference},
                operand: Box::new(expression),
            }))
        } else {
            self.parse_primary_expression()
        }

    }

    fn parse_module_import_statement(&mut self) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de l'instruction d'import de module Import/Use");

        let keyword_token = self.previous_token();
        let keyword = match keyword_token.unwrap().token_type {
            TokenType::KEYWORD(Keywords::USE) => ImportKeyword::Use,
            TokenType::KEYWORD(Keywords::IMPORT) => ImportKeyword::Import,
            _ => return Err(ParserError::new(ExpectedUseOrImport, self.current_position())),
        };

        // parse le chemin du module path
        let module_path = self.parse_module_path()?;

        if self.match_token(&[TokenType::DELIMITER(Delimiters::DOUBLECOLON)]){
            self.parse_specific_import(keyword, module_path)
        }else {
            let alias = if self.match_token(&[TokenType::KEYWORD(Keywords::AS)]) {
                let name = self.consume_identifier()?;
                Some(name)
            } else {
                None
            };

            self.consume_seperator();
            println!("Fin du parsing de l'instruction d'import de module Import/Use OK!!!!!!!!!!!!!!");
            Ok(ASTNode::Statement(Statement::ModuleImportStatement(ModuleImportStatement{
                keyword,
                module_path,
                alias,
            })))
        }


    }


    fn parse_module_path(&mut self) -> Result<Vec<String>, ParserError> {
        let mut path = Vec::new();
        loop {

            let name = self.consume_identifier()?;
            path.push(name);

            if self.match_token(&[TokenType::DELIMITER(Delimiters::DOT)]) {
                continue;
            } else {
                break;
            }
        }
        Ok(path)
    }

    fn parse_specific_import(&mut self, keyword: ImportKeyword, module_path: Vec<String>) -> Result<ASTNode, ParserError>{
        self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?;

        // parser la liste des element importés
        let mut import_list = Vec::new();
        loop {
            let name = self.consume_identifier()?;

            let alias = if self.match_token(&[TokenType::KEYWORD(Keywords::AS)]) {
                let alias = self.consume_identifier()?;
                Some(alias)
            } else {
                None
            };
            import_list.push((name, alias));

            // verifier si la liste continue ou pas
            if self.match_token(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
                continue;
            } else {
                break;
            }
        }

        self.consume(TokenType::DELIMITER(Delimiters::RCURBRACE))?;
        self.consume_seperator();

        println!("Fin du parsing de L'importation Specifique OK!!!!!!!!!!!!!!");
        Ok(ASTNode::Statement(Statement::SpecificImportStatement(SpecificImportStatement{
            keyword,
            module_path,
            alias: None,
            imports: import_list,
        })))

    }





    // fn parse_annotation(&mut self) -> Result<Attribute, ParserError> {
    //     todo!()
    // }






    // fonction utilitaire pour aide au parsing

    fn is_operator(&self,token_type: &TokenType) -> bool {

        todo!()
    }

    fn get_operator_precedence(&self, operator: &Operator) -> u8 {
        match operator {
            Operator::Multiplication | Operator::Division | Operator::Modulo => 5,
            Operator::Addition | Operator::Substraction => 4,
            Operator::LessThan | Operator::GreaterThan | Operator::LesshanOrEqual | Operator::GreaterThanOrEqual => 3,
            Operator::Range | Operator::RangeInclusive => 3,
            Operator::Equal | Operator::NotEqual => 2,
            Operator::And => 1,
            //Operator::Or => 0,
            _ => 0,
        }
    }



    fn get_compound_operator(&self,op:&Operators) -> Option<CompoundOperator>{
        match op {
            Operators::PLUSEQUAL => Some(CompoundOperator::AddAssign),
            Operators::MINEQUAL => Some(CompoundOperator::SubAssign),
            Operators::STAREQUAL => Some(CompoundOperator::MulAssign),
            Operators::SLASHEQUAL => Some(CompoundOperator::DivAssign),
            Operators::PERCENTEQUAL => Some(CompoundOperator::ModAssign),
            _ => None,
        }
    }


    fn peek_operator(&self) -> Option<Operator> {
        let token = self.current_token()?;
        println!("Token: {:?}", token);
        match &token.token_type {
            TokenType::OPERATOR(op) => {
                match op {
                    Operators::PLUS => Some(Operator::Addition),
                    Operators::MINUS => Some(Operator::Substraction),
                    Operators::STAR => Some(Operator::Multiplication),
                    Operators::SLASH => Some(Operator::Division),
                    Operators::PERCENT => Some(Operator::Modulo),
                    Operators::LESS => Some(Operator::LessThan),
                    Operators::GREATER => Some(Operator::GreaterThan),
                    Operators::LESSEQUAL => Some(Operator::LesshanOrEqual),
                    Operators::GREATEREQUAL => Some(Operator::GreaterThanOrEqual),
                    Operators::EQEQUAL => Some(Operator::Equal),
                    Operators::NOTEQUAL => Some(Operator::NotEqual),
                    Operators::AND => Some(Operator::And),
                    Operators::OR => Some(Operator::Or),
                    Operators::DOTDOT => Some(Operator::Range),
                    Operators::DOTDOTEQUAL => Some(Operator::RangeInclusive),
                    _ => None,
                }
            }
            _ => None,
        }

    }

    /// fonction pour la gestion des


    fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }
    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous_token()
    }

    fn peek_token(&self) -> Option<&Token>{
        self.tokens.get(self.current)
    }
    fn peek_next_token(&self) -> Option<&Token>{
        self.tokens.get(self.current + 1)

    }

    fn previous_token(&self) -> Option<&Token> {
        if self.current > 0 {
            // &self.tokens(self.current - 1)
            Some(&self.tokens[self.current - 1])
        } else { None }
    }

    pub fn is_at_end(&self) -> bool{
        self.current >= self.tokens.len() || self.current_token().map_or(true, |t| t.token_type == TokenType::EOF)

    }

    ///  Fonctions de Vérification et de Correspondance des Tokens

    fn match_token(&mut self,expected:&[TokenType]) -> bool {
        if self.check(expected){
            self.advance();
            return true
        } else {
            false
        }
    }

    fn check(&self,expected:&[TokenType]) -> bool {
        if let Some(token) = self.current_token(){
            expected.contains(&token.token_type)
        } else {
            false
        }
    }

    fn consume(&mut self, expected: TokenType) -> Result<(), ParserError> {
        if let Some(token) = self.current_token() {
            if token.token_type == expected {
                println!("Consommation du token {:?}", token);
                //self.print_surrounding_tokens();
                self.advance();
                Ok(())
            } else {
                println!("PyRust:!!!!!!!!!!!!!!!!!!!! Erreur: token attendu {:?}, token actuel {:?}", expected, token);
                Err(ParserError::new(UnexpectedToken, self.current_position()))
            }
        } else {
            //self.print_surrounding_tokens();
            println!("PyRust:!!!!!!!!!!!!!!!!: Erreur: fin de l'entrée inattendue");
            Err(ParserError::new(UnexpectedEndOfInput, self.current_position()))
        }
    }

    // pub fn consume(&mut self, expected: TokenType) -> Result<Token, ParserError> {
    //     // on clone le token actuel pour ne pas avoir de problem avec le borrow checker
    //     let current_token = self.current_token().cloned().ok_or_else(|| {
    //         self.print_surrounding_tokens(); // Affiche les tokens autour de l'erreur
    //         ParserError::new(UnexpectedEOF, self.current_position())
    //     })?;
    //
    //     if current_token.token_type == expected {
    //         self.advance(); // Avance au prochain token
    //         Ok(current_token.clone()) // Renvoie le token consommé
    //     } else {
    //         self.print_surrounding_tokens(); // Affiche les tokens autour de l'erreur
    //         Err(ParserError::new(UnexpectedToken, self.current_position()))
    //     }
    // }

    /// fonctontion  pour aider a comsume les tokens

    fn consume_identifier(&mut self) -> Result<String, ParserError> {
        let current_token = self.current_token().ok_or_else(|| ParserError::new(UnexpectedEOF,self.current_position()))?;
        if let TokenType::IDENTIFIER {name:_} = &current_token.token_type{
            let name = current_token.text.clone();
            self.advance();
            Ok(name)
        } else { Err(ParserError::new(ExpectIdentifier,self.current_position())) }

    }

    /// Fonction pour afficher les tokens autour de l'erreur
    pub fn create_error_with_context(&self, error_type: ParserErrorType) -> ParserError {
        self.print_surrounding_tokens();
        ParserError::new(
            error_type,
            Position {
                index: self.current,
            },
        )
    }

    fn print_surrounding_tokens(&self) {
        let prev_token = if self.current > 0 {
            Some(&self.tokens[self.current - 1])
        } else {
            None
        };
        let current_token = self.current_token();
        let next_token = if self.current + 1 < self.tokens.len() {
            Some(&self.tokens[self.current + 1])
        } else {
            None
        };
        println!("");
        println!("---------------- Token Error Context--by-YmC ----------");
        if let Some(prev) = prev_token {
            println!("Previous Token: {:?}", prev);
        }
        if let Some(current) = current_token {
            println!("Current Token: {:?}", current);
        }
        if let Some(next) = next_token {
            println!("Next Token: {:?}", next);
        }
        println!("----------------------------------------------------------");
        println!("");
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if let Some(previous) = self.previous_token() {
                if previous.token_type == TokenType::DELIMITER(Delimiters::SEMICOLON) {
                    return;
                }
            }

            match self.current_token().map(|t| &t.token_type) {
                Some(TokenType::KEYWORD(Keywords::STRUCT)) |
                Some(TokenType::KEYWORD(Keywords::FN)) |
                Some(TokenType::KEYWORD(Keywords::LET)) |
                Some(TokenType::KEYWORD(Keywords::CONST)) |
                Some(TokenType::KEYWORD(Keywords::PUB)) => return,
                _ => { self.advance(); }
            }
        }
    }

    fn consume_seperator(&mut self)  {
        println!("Mode de syntaxe : {:?}", self.syntax_mode);
        match self.syntax_mode{
            SyntaxMode::Indentation =>{
                // ordre logique de verification EOF -> DEDENT -> NEWLINE
                println!("Indentation Mode");
                if self.check(&[TokenType::EOF]){
                    let _ = self.consume(TokenType::EOF);
                }else if self.check(&[TokenType::DEDENT]){
                    let _ = self.consume(TokenType::DEDENT);
                }else {
                    let _ = self.consume(TokenType::NEWLINE) ;
                }
            }
            SyntaxMode::Braces =>{
                println!("Braces Mode");
                if self.check(&[TokenType::DELIMITER(Delimiters::SEMICOLON)]) || self.check(&[TokenType::EOF]){
                    let _  = self.consume(TokenType::DELIMITER(Delimiters::SEMICOLON));
                }
            }
        }
    }

    /// fonction pour verifier la sequence de tokens a utiliser plus tard
    pub fn check_sequence(&self, tokens: &[TokenType]) -> bool {
        for (i, token_type) in tokens.iter().enumerate() {
            if self.current + i >= self.tokens.len() || self.tokens[self.current + i].token_type != *token_type {
                return false;
            }
        }
        true
    }

    fn check_for_label(&mut self) -> Result<Option<String>, ParserError> {
        // Vérifie si le token actuel est un identifiant
        if let Some(current) = self.peek_token() {
            if let Some(next) = self.peek_next_token() {
                // Vérifie si c'est un label (identifiant suivi de ':')
                match (&current.token_type, &next.token_type) {
                    (
                        TokenType::IDENTIFIER { name },
                        TokenType::DELIMITER(Delimiters::COLON)
                    ) => {
                        // Clone le nom avant d'avancer
                        let label_name = name.clone();

                        // Consomme l'identifiant et le ':'
                        self.advance(); // Consomme l'identifiant
                        self.advance(); // Consomme le ':'

                        return Ok(Some(label_name));
                    }
                    _ => return Ok(None)
                }
            }
        }
        Ok(None)
    }

    // methode utilitaire pour consommer un token s'il est present
    // sans generé d'erreur s'il n'est pas present
    // fn consume_if(&mut self,expected:TokenType) -> bool{
    //     if self.match_token(&[expected.clone()]){
    //         self.advance();
    //         true
    //     }else { false }
    // }
    //
    //





    // pub fn parse_declarations(&mut self) -> Result<Vec<ASTNode>, ParserError> {
    //     let mut declarations = Vec::new();
    //
    //     while !self.is_at_end() {
    //         match self.parse_declaration() {
    //             Ok(decl) => {
    //                 declarations.push(decl);
    //             },
    //             Err(e) => {
    //                 println!("Erreur lors du parsing : {:?}", e);
    //                 self.synchronize();
    //                 continue;
    //             }
    //         }
    //     }
    //
    //     Ok(declarations)
    //





}

//by YmC






///////////////////////fin essai//////////////////////////////

// ////////////////////fin de mon  parse/////////////////////// */