use crate::parser::ast::{ASTNode, BreakStatement, ContinueStatement, ElifStatement, ExceptHandler, ForStatement, IfStatement, LoopStatement, Statement, TryStatement, Visibility, WhileStatement};
use crate::parser::parser_error::{ParserError, ParserErrorType};
use crate::parser::parser::Parser;
use crate::tok::{Delimiters, Keywords, TokenType};

impl Parser{


    /// fonction pour le gestion de structure de controle
    pub fn parse_if_statement(&mut self) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de l'instruction if");
        self.consume(TokenType::KEYWORD(Keywords::IF))?;
        let condition = self.parse_expression(0)?;
        let then_block = self.parse_block()?;

        let mut elif_branches = Vec::new();
        while self.check(&[TokenType::KEYWORD(Keywords::ELIF)]) {
            self.consume(TokenType::KEYWORD(Keywords::ELIF))?;
            let elif_condition = self.parse_expression(0)?;
            let elif_then_block = self.parse_block()?;
            elif_branches.push(ElifStatement {
                condition: elif_condition,
                block: elif_then_block,
            });
        }

        let else_block = if self.check(&[TokenType::KEYWORD(Keywords::ELSE)]) {
            self.consume(TokenType::KEYWORD(Keywords::ELSE))?;
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(ASTNode::Statement(Statement::IfStatement(IfStatement {
            condition,
            then_block,
            elif_block: elif_branches,
            else_block,
        })))
    }

    pub fn parse_while_statement(&mut self) -> Result<ASTNode, ParserError> {
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

    pub fn parse_loop_statement(&mut self) -> Result<ASTNode, ParserError> {
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

    pub fn parse_for_statement(&mut self) -> Result<ASTNode, ParserError> {
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

    pub fn parse_break_statement(&mut self) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de l'instruction break");
        self.consume(TokenType::KEYWORD(Keywords::BREAK))?;
        let label = self.check_for_label()?;
        self.consume_seperator();
        println!("Fin du parsing de l'instruction break OK!!!!!!!!!!!!!!!");
        Ok(ASTNode::Statement(Statement::BreakStatement(BreakStatement{
            label
        })))
    }

    pub fn parse_continue_statement(&mut self) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de l'instruction continue");
        self.consume(TokenType::KEYWORD(Keywords::CONTINUE))?;
        let label = self.check_for_label()?;
        self.consume_seperator();
        println!("Fin du parsing de l'instruction continue OK!!!!!!!!!!!!!!!");
        Ok(ASTNode::Statement(Statement::ContinueStatement(ContinueStatement{
            label
        })))
    }



    ///fonction pour le parsing des blocs de code Try/Except/Finally
    pub fn parse_try_statement(&mut self) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de l'instruction try");

        // Consommer le 'try'
        self.consume(TokenType::KEYWORD(Keywords::TRY))?;

        // Parser le bloc try selon le mode syntaxique
        let try_body = self.parse_block()?;

        let mut handlers = Vec::new();
        let mut finally_body = None;

        // Parser les blocs except
        while self.check(&[TokenType::KEYWORD(Keywords::EXCEPT)]) {
            let handler = self.parse_except_handler()?;
            handlers.push(handler);
        }

        // Parser le bloc finally optionnel
        if self.match_token(&[TokenType::KEYWORD(Keywords::FINALLY)]) {
            finally_body = Some(self.parse_block()?);
        }

        if handlers.is_empty() && finally_body.is_none() {
            return Err(ParserError::new(
                ParserErrorType::MissingExceptHandler,
                self.current_position(),
            ));
        }

        Ok(ASTNode::Statement(Statement::TryStatement(TryStatement {
            body: try_body,
            handlers,
            finally_body,
        })))
    }


    pub fn parse_except_handler(&mut self) -> Result<ExceptHandler,ParserError>{
        println!("Début du parsing de l'except handler");

        self.consume(TokenType::KEYWORD(Keywords::EXCEPT))?;

        let exception_type = if !self.check(&[TokenType::DELIMITER(Delimiters::COLON)]) {
            Some(self.parse_expression(0)?)
        } else {
            None
        };

        let name = if self.match_token(&[TokenType::KEYWORD(Keywords::AS)]) {
            Some(self.consume_identifier()?)
        } else {
            None
        };

        let body = self.parse_block()?;

        println!("Fin du parsing de l'exception handler");

        Ok(ExceptHandler {
            exception_type,
            name,
            body,
        })

    }


    ///fonction principal pour  le parsing des statements

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
        }else if self.check(&[TokenType::KEYWORD(Keywords::LOOP)]){
            self.parse_loop_statement()
        }else if self.match_token(&[TokenType::KEYWORD(Keywords::IMPORT),TokenType::KEYWORD(Keywords::USE)]){
            self.parse_module_import_statement()
        }else if self.check(&[TokenType::KEYWORD(Keywords::RETURN)]) {
            self.parse_return_statement()
        }else if self.check(&[TokenType::KEYWORD(Keywords::IF)]){
            self.parse_if_statement()
        }

        // else if self.check(&[TokenType::KEYWORD(Keywords::ELIF)]) {
        //     self.parse_if_statement()
        // }

        // else if self.check(&[TokenType::DEDENT]) {
        //     // Si on rencontre un DEDENT, le consommer et continuer
        //     self.advance();
        //     return self.parse_statement();
        // }




        else if self.check(&[TokenType::KEYWORD(Keywords::WHILE)]) {
            self.parse_while_statement()
        }else if self.check(&[TokenType::KEYWORD(Keywords::FOR)]) {
            self.parse_for_statement()
        }else if self.check(&[TokenType::KEYWORD(Keywords::MATCH)]) {
            self.parse_match_statement()
        }else if self.check(&[TokenType::KEYWORD(Keywords::TRY)]) {
            self.parse_try_statement()

            // }else if self.check(&[TokenType::KEYWORD(Keywords::WHERE)]){
            //     self.parse_where_clauses()

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

}
