
#[allow(dead_code)]
use crate::lexer::lex::{SyntaxMode, Token};

use crate::parser::ast::{ ArrayRest, AssociatedType, ASTNode,  CompoundOperator, Expression,  GenericType,   ImportKeyword, Literal,  MatchArm, MatchStatement, ModuleImportStatement, Operator, Parameter, Pattern,  RangePattern, ReturnStatement,  SpecificImportStatement, Statement, Type, TypeBound,UnaryOperation, UnaryOperator,};

use crate::parser::parser_error::ParserErrorType::{ ExpectIdentifier, ExpectedTypeAnnotation,  InvalidTypeAnnotation,  UnexpectedEOF, UnexpectedEndOfInput,  UnexpectedToken, ExpectedParameterName,MultipleRestPatterns, ExpectedUseOrImport,  ExpectedCommaOrCloseBrace, };
use crate::parser::parser_error::{ParserError, ParserErrorType, Position};
use crate::tok::{Delimiters, Keywords, Operators, TokenType};
use crate::parser::inference::{TypeContext};

pub struct Parser {
    pub(crate) tokens: Vec<Token>, // liste des tokens genere par le lexer
    pub(crate) current: usize,     // index du token actuel
    pub(crate) syntax_mode: SyntaxMode,
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

    pub fn parse_program(&mut self) -> Result<ASTNode, ParserError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            let statement = match self.parse_statement() {
                Ok(stmt) => stmt,
                Err(e) => {
                    log::debug!("Erreur de parsing : {:?}", e);
                    // On applique la synchronisation
                    self.synchronize()?;
                    // On peut continuer à la prochaine itération
                    continue;
                }
            };
            statements.push(statement);
        }
        Ok(ASTNode::Program(statements))
    }



    /// fonction pour aider le parsing des blocs
    #[allow(dead_code)]
    fn get_syntax_mode(&self) ->SyntaxMode{
        self.syntax_mode
    }

    pub fn parse_block(&mut self) -> Result<Vec<ASTNode>, ParserError> {
        match self.syntax_mode{
            SyntaxMode::Indentation => self.parse_indented_block(),
            SyntaxMode::Braces => self.parse_braced_block(),

        }
    }

    #[allow(dead_code)]
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
        log::debug!("Parsing indented block");
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
        log::debug!("Parsing braced block");
        self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?;
        let mut statements = Vec::new();

        while !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE), TokenType::EOF]) {
            let stmt = self.parse_statement()?;

            statements.push(stmt);
            // je vais ajoute un code qui  m'aiderai  a  parse le  body de parse_declaration_body
            if self.check(&[TokenType::DELIMITER(Delimiters::COMMA)]){
                    self.consume(TokenType::DELIMITER(Delimiters::COMMA))?;
            }else if !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]) {
                return Err(ParserError::new(ExpectedCommaOrCloseBrace, self.current_position()));
            }
        }
        self.consume(TokenType::DELIMITER(Delimiters::RCURBRACE))?;

        Ok(statements)
    }

    #[allow(dead_code)]
    fn begin_block(&mut self) {
        // self.indent_level.push(self.current_token().unwrap().indent);
        todo!()
    }

    #[allow(dead_code)]
    fn end_block(&mut self) {
        // self.indent_level.pop();
        todo!()
    }

    fn parse_labeled_statement(&mut self) -> Result<Option<ASTNode>, ParserError> {
        if let Some(label_name) = self.check_for_label()? {
            // Après avoir consommé le label, on vérifie quelle instruction suit
            if self.check(&[TokenType::KEYWORD(Keywords::LOOP)]) {
                return self.parse_loop_statement().map(Some);
            } else {
                // Vous pouvez étendre ici pour d'autres instructions qui peuvent être labellisées
                return Err(ParserError::new(UnexpectedToken, self.current_position()));
            }
        }

        // Pas de label, retourner None
        Ok(None)
    }

    pub(crate) fn parse_function_parameters(&mut self) -> Result<Vec<Parameter>, ParserError> {
        log::debug!("Début du parsing des paramètres de fonction");
        let mut parameters = Vec::new();

        if self.check(&[TokenType::DELIMITER(Delimiters::RPAR)]){
            // pas de paramètres
            return Ok(parameters);
        }

        if !self.match_token(&[TokenType::DELIMITER(Delimiters::RPAR)]) {
            loop {
                //let name = self.consume_parameter_name()?;
                let name = self.consume_identifier()?;
                log::debug!("Nom du paramètre parsé : {}", name);
                self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
                let param_type = self.parse_type()?;
                log::debug!("Type du paramètre parsé : {:?}", param_type);

                parameters.push(Parameter { name, parameter_type: param_type });

                if self.match_token(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
                    continue;
                } else if self.check(&[TokenType::DELIMITER(Delimiters::RPAR)]) {
                    break;
                }else {
                    log::debug!("Erreur lors du parsing des paramètres, token actuel : {:?}", self.current_token());
                    return Err(ParserError::new(ExpectedParameterName, self.current_position()));
                }
            }
        }
        log::debug!("Paramètres parsés : {:?}", parameters);
        Ok(parameters)
    }

    pub fn parse_function_body(&mut self) -> Result<Vec<ASTNode>, ParserError> {
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

    pub fn parse_body_block(&mut self) -> Result<Vec<ASTNode>,ParserError>{
        log::debug!("Début du parsing du corps");
        self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?;
        let mut statements = Vec::new();
        while !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]) && !self.is_at_end() {
            let stmt = self.parse_statement()?;
            statements.push(stmt);
        }
        self.consume(TokenType::DELIMITER(Delimiters::RCURBRACE))?;
        log::debug!("Fin du parsing du corps OK!!!!!!!!!!!!");
        Ok(statements)
    }

    pub fn parse_block_expression(&mut self) -> Result<Vec<ASTNode>,ParserError>{
        log::debug!("Debut du parsing de du bloc de L'expression LAMBDA");
        self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?;

        let mut body = Vec::new();
        while !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]){
            let statement = self.parse_statement()?;
            body.push(statement);
        }
        self.consume(TokenType::DELIMITER(Delimiters::RCURBRACE))?;
        log::debug!("Fin du parsing du bloc de l'expression LAMBDA OK!!!!!!!!!!!");
        Ok(body)

    }


    pub fn parse_associated_type(&mut self) -> Result<AssociatedType, ParserError> {
        // Consommer le mot-clé `type`
        self.consume(TokenType::KEYWORD(Keywords::TYPE))?;

        // Lire le nom du type associé
        let name = self.consume_identifier()?;

        // Vérifier si des bounds (contraintes de type) sont spécifiées
        let type_bound = if self.check(&[TokenType::DELIMITER(Delimiters::COLON)]) {
            self.advance();
            // self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
            Some(self.parse_type_bounds()?)
            // Some(self.parse_type()?)
        } else {
            None
        };

        //ici on verifie aussi le where clause
        let where_clause = self.parse_where_clauses()?;

        // Consommer le point-virgule `;`
        // self. Consume(TokenType::DELIMITER(Delimiters::SEMICOLON))?;
        self.consume_seperator();

        Ok(AssociatedType { name, type_bound ,where_clause})
    }

    pub fn parse_type_bounds(&mut self) -> Result<Vec<TypeBound>, ParserError> {
        let mut type_bounds = Vec::new();

        let bound = self.consume_identifier()?;
        // type_bounds.push(Type::Trait(bound));
        type_bounds.push(TypeBound::TraitBound(bound));

        while self.check(&[TokenType::OPERATOR(Operators::PLUS)]){
            self.advance();
            let bound = self.consume_identifier()?;
            type_bounds.push(TypeBound::TraitBound(bound));

        }

        Ok(type_bounds)
    }


    /// fonction pour parser les types
    pub fn parse_type(&mut self) -> Result<Type, ParserError> {
        let token = self
            .current_token()
            .ok_or_else(|| ParserError::new(ExpectedTypeAnnotation, self.current_position()))?;

        // eprintln!("parse_type: current token = {:?}", token);
        log::debug!("Parsing type: {:?}", token);

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
            TokenType::IDENTIFIER { name } => {
                let base_name = name.clone();
                self.advance();
                // let base_name = name.clone();

                // Vérifie s'il y a des paramètres génériques
                if self.check(&[TokenType::OPERATOR(Operators::LESS)]) {
                    // Parse les paramètres génériques
                    let mut type_params = Vec::new();
                    self.consume(TokenType::OPERATOR(Operators::LESS))?;

                    loop {
                        type_params.push(self.parse_type()?);

                        if self.check(&[TokenType::OPERATOR(Operators::GREATER)]) {
                            self.advance();
                            break;
                        } else if self.check(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
                            self.advance();
                        } else {
                            return Err(ParserError::new(InvalidTypeAnnotation, self.current_position()));
                        }
                    }

                    Ok(Type::Generic(GenericType {
                        base: base_name,
                        type_parameters: type_params,
                    }))
                } else {
                    Ok(Type::Named(base_name))
                }
            }
            _ => {
                log::debug!("Unexpected token: {:?}", token);
                // Si le token actuel n'est pas un type valide, renvoyer une erreur
                Err(ParserError::new(
                    InvalidTypeAnnotation,
                    self.current_position(),
                ))
            }
        }
    }


    // fn parse_type_cast(&mut self,expr: Expression) -> Result<Expression, ParserError> {
    //
    // }


    pub fn parse_match_statement(&mut self) -> Result<ASTNode, ParserError> {
        log::debug!("Début du parsing de l'instruction match");
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

        log::debug!("Fin du parsing de l'instruction match OK!!!!!!!!!!!!!!");
        Ok(ASTNode::Statement(Statement::MatchStatement(MatchStatement{
            expression: match_expr,
            arms,
        })))

    }

    #[allow(dead_code)]
    fn is_end_of_match(&self) -> bool {
        match self.syntax_mode {
            SyntaxMode::Indentation => self.check(&[TokenType::DEDENT]),
            SyntaxMode::Braces => self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]),
        }
    }

    #[allow(dead_code)]
    fn is_block_expression(&self,node: &ASTNode) -> bool {
        matches!(node,
            ASTNode::Statement(Statement::IfStatement(_)) |
            ASTNode::Statement(Statement::LoopStatement(_)) |
            ASTNode::Statement(Statement::WhileStatement(_))|
            ASTNode::Expression(Expression::LambdaExpression(_))
        )
    }


    pub fn parse_indented_arm_body(&mut self) -> Result<Vec<ASTNode>, ParserError> {
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

    pub fn parse_braced_arm_body(&mut self) -> Result<Vec<ASTNode>, ParserError> {
        self.consume(TokenType::OPERATOR(Operators::FATARROW))?;

        let body = if self.check(&[TokenType::DELIMITER(Delimiters::LCURBRACE)]) {
            // Corps avec bloc
            self.parse_unified_block()?
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

    pub fn parse_guard(&mut self) -> Result<Option<Box<Expression>>, ParserError> {
        if self.match_token(&[TokenType::KEYWORD(Keywords::IF)]){
            let condition = self.parse_expression(0)?;
            Ok(Some(Box::new(condition)))
        }else {
            Ok(None)
        }
    }
    pub fn parse_match_arm(&mut self) -> Result<MatchArm, ParserError> {
        log::debug!("Début du parsing du bras de match");
        let pattern = self.parse_pattern_complex()?;

        let guard = self.parse_guard()?;

        let body= if self.syntax_mode == SyntaxMode::Indentation{
            self.parse_indented_arm_body()?
        }else {
            self.parse_braced_arm_body()?
        };
        log::debug!("Fin du parsing du bras de match OK!!!!!!!!!!!!!!!");
        Ok(MatchArm{
            pattern,
            guard,
            body,
        })
    }

    pub fn parse_pattern_complex(&mut self) -> Result<Pattern, ParserError>{
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

    pub fn parse_tuple_pattern(&mut self) -> Result<Pattern, ParserError> {
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
        log::debug!("Fin du parsing du tuple pattern OK!!!!!!!!!!!!!!!");
        Ok(Pattern::Tuple(patterns))
    }

    //feature pour plus tard
    pub fn parse_tuple_rest_pattern(&mut self) -> Result<Pattern, ParserError> {
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

    pub fn parse_array_pattern(&mut self) -> Result<Pattern, ParserError> {
        log::debug!("Début du parsing du pattern de tableau Array");
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
        log::debug!("Fin du parsing du pattern de tableau Array OK!!!!!!!!!!!!!!!");
        Ok(Pattern::Array(patterns))

    }

    //feature pour plus tard
    pub fn parse_array_rest_pattern(&mut self) -> Result<Pattern, ParserError> {
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

    pub fn parse_range_pattern(&mut self) -> Result<Pattern, ParserError> {
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


    pub fn parse_pattern(&mut self) -> Result<Pattern, ParserError> {
        log::debug!("Début du parsing du pattern");


        if self.match_token(&[TokenType::OPERATOR(Operators::UNDERSCORE)]) {
            // Pattern par défaut '_'
            Ok(Pattern::Wildcard)
        } else if let Some(token) = self.current_token() {
            match &token.token_type {
                TokenType::IDENTIFIER { name } => {
                    if *name == "_" {
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


    pub fn parse_return_statement(&mut self) -> Result<ASTNode, ParserError> {
        log::debug!("Début du parsing de l'instruction de retour");
        self.consume(TokenType::KEYWORD(Keywords::RETURN))?;
        let value = if !self.match_token(&[TokenType::NEWLINE, TokenType::DEDENT, TokenType::EOF]) {
            Some(self.parse_expression(0)?)
        } else {
            None
        };
        log::debug!("Valeur de retour parsée : {:?}", value);
        log::debug!("Fin du parsing de l'instruction de retour OK!!!!!!!!!!!!!!");
        Ok(ASTNode::Statement(Statement::ReturnStatement(ReturnStatement{
            value,
        })))

    }


    /// fonction pour la gestion des emprunts
    /// J'ai deja  implementé la gestion des emprunts dans parse_unary_expression()

    pub fn parse_borrow(&mut self) -> Result<Expression, ParserError> {
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

    pub fn parse_module_import_statement(&mut self) -> Result<ASTNode, ParserError> {
        log::debug!("Début du parsing de l'instruction d'import de module Import/Use");

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
            log::debug!("Fin du parsing de l'instruction d'import de module Import/Use OK!!!!!!!!!!!!!!");
            Ok(ASTNode::Statement(Statement::ModuleImportStatement(ModuleImportStatement{
                keyword,
                module_path,
                alias,
            })))
        }


    }

    pub fn parse_module_path(&mut self) -> Result<Vec<String>, ParserError> {
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

    pub fn parse_specific_import(&mut self, keyword: ImportKeyword, module_path: Vec<String>) -> Result<ASTNode, ParserError>{
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

        log::debug!("Fin du parsing de L'importation Specifique OK!!!!!!!!!!!!!!");
        Ok(ASTNode::Statement(Statement::SpecificImportStatement(SpecificImportStatement{
            keyword,
            module_path,
            alias: None,
            imports: import_list,
        })))

    }

    // fonction utilitaire pour aide au parsing
    #[allow(dead_code)]
    fn is_operator(&self,_token_type: &TokenType) -> bool {
        todo!()
    }


    pub fn if_single_quote(&self,s:&str) -> bool {
        if s.starts_with('\'') && s.ends_with('\'')  && s.len() == 3 {
            true
        } else {
            false
        }

    }

    pub fn parse_inference_type(&mut self,explicit_type:&Type, infer:&Expression) -> Result<Type, ParserError> {
        let mut type_context = TypeContext::new();

        let inferred_type = type_context.infer_expression(infer)
            .map_err(|_msg| ParserError::new(
                ParserErrorType::TypeInferenceError,
                self.current_position()
            ))?;

        match explicit_type {
            Type::Infer => Ok(inferred_type),
            explicit if *explicit == *&inferred_type => Ok(explicit.clone()),
            _explicit => Err(ParserError::new(
                ParserErrorType::TypeInferenceError,
                self.current_position(),
            )),
        }
    }


    pub fn get_operator_precedence(&self, operator: &Operator) -> u8 {
        match operator {
            Operator::Multiplication | Operator::Division | Operator::Modulo => 5,
            Operator::Addition | Operator::Subtraction => 4,
            Operator::LessThan | Operator::GreaterThan | Operator::LessThanOrEqual | Operator::GreaterThanOrEqual => 3,
            Operator::Range | Operator::RangeInclusive => 3,
            Operator::Equal | Operator::NotEqual => 2,
            Operator::And => 1,
            //Operator::Or => 0,
            _ => 0,
        }
    }


    pub fn get_compound_operator(&self,op:&Operators) -> Option<CompoundOperator>{
        match op {
            Operators::PLUSEQUAL => Some(CompoundOperator::AddAssign),
            Operators::MINEQUAL => Some(CompoundOperator::SubAssign),
            Operators::STAREQUAL => Some(CompoundOperator::MulAssign),
            Operators::SLASHEQUAL => Some(CompoundOperator::DivAssign),
            Operators::PERCENTEQUAL => Some(CompoundOperator::ModAssign),
            _ => None,
        }
    }


    pub fn peek_operator(&self) -> Option<Operator> {
        let token = self.current_token()?;
        log::debug!("Token: {:?}", token);
        match &token.token_type {
            TokenType::OPERATOR(op) => {
                match op {
                    Operators::PLUS => Some(Operator::Addition),
                    Operators::MINUS => Some(Operator::Subtraction),
                    Operators::STAR => Some(Operator::Multiplication),
                    Operators::SLASH => Some(Operator::Division),
                    Operators::PERCENT => Some(Operator::Modulo),
                    Operators::LESS => Some(Operator::LessThan),
                    Operators::GREATER => Some(Operator::GreaterThan),
                    Operators::LESSEQUAL => Some(Operator::LessThanOrEqual),
                    Operators::GREATEREQUAL => Some(Operator::GreaterThanOrEqual),
                    Operators::EQUAL => Some(Operator::Equal),
                    Operators::EQEQUAL => Some(Operator::EqualEqual),
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






    ///  Fonctions de Vérification et de Correspondance des Tokens





    /// fonctontion  pour aider a comsume les tokens


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
        log::debug!("");
        log::debug!("---------------- Token Error Context--by-YmC ----------");
        if let Some(prev) = prev_token {
            log::debug!("Previous Token: {:?}", prev);
        }
        if let Some(current) = current_token {
            log::debug!("Current Token: {:?}", current);
        }
        if let Some(next) = next_token {
            log::debug!("Next Token: {:?}", next);
        }
        log::debug!("----------------------------------------------------------");
        log::debug!("");
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


    // fonction pour checke  le lifetime
    pub fn check_lifetime_token(&mut self) ->bool{
        if let Some(token) = &self.current_token(){
            match &token.token_type {
                TokenType::IDENTIFIER {name} => name.starts_with('\''),
                _ => false,
            }
        }else { false }
    }

}

////////////////////////////////PyRust////Dev////by YmC///////////////////////////////////
/////////////////////////////////fin de mon  parse/////////////////////////////////////*/