use crate::parser::ast::{ASTNode, ExceptHandler, Statement, TryStatement};
use crate::parser::parser_error::{ParserError, ParserErrorType};
use crate::parser::parser;
use crate::parser::parser::Parser;
use crate::tok::{Delimiters, Keywords, TokenType};

impl Parser{

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

}
