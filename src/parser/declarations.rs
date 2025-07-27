use crate::lexer::lex::Token;
use crate::parser::ast::{ArrayAccess, ArrayExpression, ArrayRepeatExpression, ASTNode, Attribute, ClassDeclaration, CompFor, ComprehensionFor, ConstDeclaration, Constructor, Declaration, DictAccess, DictComprehension, DictEntry, DictLiteral, EnumDeclaration, EnumVariant, Expression, Field, FunctionDeclaration, GenericType, ImplDeclaration, ListComprehension, MethodeDeclaration, Mutability, StructDeclaration, TraitDeclaration, TraitMethod, Type, VariableDeclaration, Visibility, WhereClause};
use crate::parser::ast::Declaration::Variable;
use crate::parser::parser::Parser;
use crate::parser::parser_error::ParserError;
use crate::parser::parser_error::ParserErrorType::{ExpectColon, MultipleConstructors, UnexpectedToken};
use crate::SyntaxMode;
use crate::tok::{Delimiters, Keywords, Operators, TokenType};


impl Parser{

    /// fonction pour parser les déclarations de variables
    /// Exemple: Brace Mode
    /// // let mut x: int = 5;
    /// // let y: float = 3.14;
    /// // let z = 42;
    /// // let a: bool = true;
    /// Exemple: Indentation Mode
    /// // let mut x: int = 5
    /// // let y: float = 3.14
    /// // let z = 42
    /// // let a: bool = true
    //dans ce parse_variable_declaration, il y a implementation de l'inference de type,
    //mais j'ai decide de gere  l'inference de type dans la partie SÉMANTIQUE
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

        //infere  le type si neccessaire

        // ici, on vas implementer la fonction parse_inference_type pour determiner le type de la variable
        let final_type = self.parse_inference_type(&variable_type,&value)?;


        self.consume_seperator();
        println!("Valeur de la variable parsée : {:?}", value);

        Ok(ASTNode::Declaration(Variable(VariableDeclaration {
            name,
            variable_type: Some(final_type),
            // variable_type: Some(variable_type),
            value: Some(value),
            mutability,
        })))

    }

    pub fn parse_const_declaration(&mut self, visibility: Visibility) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de la déclaration de constante");

        //let visibility = self.parse_visibility()?;

        self.consume(TokenType::KEYWORD(Keywords::CONST))?;

        // //test pour la synchronisation
        // self.consume(TokenType::KEYWORD(Keywords::CONST)).or_else(|e| {
        //     // On synchronise ici
        //     self.synchronize()?;
        //     Err(e)
        // })?;


        let name = self.consume_identifier()?;

        let variable_type = if self.match_token(&[TokenType::DELIMITER(Delimiters::COLON)]) {
            self.parse_type()?
        } else {
            Type::Infer
        };

        self.consume(TokenType::OPERATOR(Operators::EQUAL))?;
        let value = self.parse_expression(0)?;

        //transfer dan la fonction parse_inference_type

        //infere  le type si neccessaire
        let final_type = self.parse_inference_type(&variable_type,&value)?;

        self.consume_seperator();

        println!("la valeur de la constante parse : {:?}", value);

        Ok(ASTNode::Declaration(Declaration::Constante(ConstDeclaration{
            name,
            constant_type: Some(final_type),
            // constant_type: Some(variable_type),
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

        // let return_type = self.parse_inference_type(&return_type, &body.)?;
        // let return_type = self.parse_inference_type(&return_type, &Vec<ASTNode>)?;


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



    pub fn parse_enum_declaration(&mut self, visibility: Visibility) -> Result<ASTNode, ParserError> {
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

    pub fn parse_trait_declaration(&mut self, visibility: Visibility) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de la déclaration de trait");
        self.consume(TokenType::KEYWORD(Keywords::TRAIT))?;
        let name = self.consume_identifier()?;
        println!("Nom du trait parsé : {}", name);

        let generic_params = if self.check(&[TokenType::OPERATOR(Operators::LESS)]) {
            Some(self.parse_generic_parameters()?)
        } else {
            None
        };

        // Parse des supertraits optionnels - ne tente que si c'est vraiment un : pour les super traits
        let mut super_traits = Vec::new();
        if self.check(&[TokenType::DELIMITER(Delimiters::COLON)]) {
            let next_token = self.peek_next_token();
            if let Some(token) = next_token {
                if !matches!(token.token_type, TokenType::NEWLINE) &&
                    !matches!(token.token_type, TokenType::DELIMITER(Delimiters::LCURBRACE)) {
                    self.advance(); // Consomme le ':'
                    super_traits = self.parse_trait_bounds()?;
                }
            }
        }

        let mut methods = Vec::new();
        let mut associated_types = Vec::new();

        //Optionelement de where clause
        let where_clause = self.parse_where_clauses()?;

        match self.syntax_mode{
            SyntaxMode::Braces => {
                self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?;
                while !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]) && !self.is_at_end() {
                    if self.check(&[TokenType::KEYWORD(Keywords::FN)]) {
                        methods.push(self.parse_trait_methods()?);
                    } else if self.check(&[TokenType::KEYWORD(Keywords::TYPE)]) {
                        associated_types.push(self.parse_associated_type()?);
                    } else {
                        return Err(ParserError::new(UnexpectedToken, self.current_position()));
                    }
                }
                self.consume(TokenType::DELIMITER(Delimiters::RCURBRACE))?; // Consomme explicitement la '}'
            },
            SyntaxMode::Indentation => {
                self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
                self.consume(TokenType::NEWLINE)?;
                self.consume(TokenType::INDENT)?;
                while !self.check(&[TokenType::DEDENT]) && !self.is_at_end() {
                    if self.check(&[TokenType::KEYWORD(Keywords::FN)]) {
                        methods.push(self.parse_trait_methods()?);
                    } else if self.check(&[TokenType::KEYWORD(Keywords::TYPE)]) {
                        associated_types.push(self.parse_associated_type()?);
                    } else {
                        return Err(ParserError::new(UnexpectedToken, self.current_position()));
                    }
                }

            }
        }


        println!("Parsing des Trait OK!!!!!!!!!!!!!!!!!!!!!!");
        Ok(ASTNode::Declaration(Declaration::Trait(TraitDeclaration{
            name,
            generic_parameters: generic_params,
            methods,
            associated_types,
            visibility,
            where_clause,
            super_traits
        })))

    }


    pub fn parse_impl_declaration(&mut self, visibility: Visibility) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de la déclaration d'implémentation");
        self.consume(TokenType::KEYWORD(Keywords::IMPL))?;

        // Parse les paramètres génériques optionnels
        let generic_params = if self.check(&[TokenType::OPERATOR(Operators::LESS)]) {
            Some(self.parse_generic_parameters()?)
        } else {
            None
        };

        // Parse le type cible
        let trait_name = self.consume_identifier()?;

        // Vérifie s'il s'agit d'une implémentation de trait
        let target_type = if self.check(&[TokenType::KEYWORD(Keywords::FOR)]) {
            self.advance(); // Consomme 'for'
            // Parse le type cible qui peut être générique
            let base_type = self.consume_identifier()?;

            // Vérifie s'il y a des paramètres génériques pour le type cible
            if self.check(&[TokenType::OPERATOR(Operators::LESS)]) {
                self.advance(); // Consomme '<'
                let mut type_params = Vec::new();

                loop {
                    type_params.push(self.parse_type()?);
                    if self.check(&[TokenType::OPERATOR(Operators::GREATER)]) {
                        self.advance();
                        break;
                    } else if self.check(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
                        self.advance();
                    } else {
                        return Err(ParserError::new(UnexpectedToken, self.current_position()));
                    }
                }

                Type::Generic(GenericType {
                    base: base_type,
                    type_parameters: type_params,
                })
            } else {
                Type::Named(base_type)
            }
        } else {
            Type::Named(trait_name.clone())
        };

        // Parse where clause optionnelle
        let where_clause = self.parse_where_clauses()?;

        let mut methods = Vec::new();

        match self.syntax_mode {
            SyntaxMode::Braces => {
                self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?;
                while !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]) && !self.is_at_end() {
                    if self.check(&[TokenType::KEYWORD(Keywords::FN)]) {
                        methods.push(self.parse_impl_method()?);
                    } else {
                        return Err(ParserError::new(UnexpectedToken, self.current_position()));
                    }
                }
                self.consume(TokenType::DELIMITER(Delimiters::RCURBRACE))?;
            },
            SyntaxMode::Indentation => {
                self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
                self.consume(TokenType::NEWLINE)?;
                self.consume(TokenType::INDENT)?;

                while !self.check(&[TokenType::DEDENT]) && !self.is_at_end() {
                    if self.check(&[TokenType::KEYWORD(Keywords::FN)]) {
                        methods.push(self.parse_impl_method()?);
                    } else {
                        return Err(ParserError::new(UnexpectedToken, self.current_position()));
                    }
                }
                // self.consume(TokenType::NEWLINE)?;
                self.consume(TokenType::DEDENT)?;
            }
        }

        Ok(ASTNode::Declaration(Declaration::Impl(ImplDeclaration {
            target_type,
            trait_name: Some(trait_name),
            generic_parameters: generic_params,
            methods,
            where_clause,
            visibility,
        })))
    }

    pub fn parse_class_declaration(&mut self, visibility: Visibility) -> Result<ASTNode, ParserError> {
        println!("Début du parsing de la déclaration de classe");
        self.consume(TokenType::KEYWORD(Keywords::CLASS))?;

        let name = self.consume_identifier()?;

        println!("Nom de la classe parsé : {}", name);

        let parent_classes = self.parse_class_inheritance()?;

        match self.syntax_mode{
            SyntaxMode::Indentation => self.consume(TokenType::DELIMITER(Delimiters::COLON))?,
            // SyntaxMode::Braces => self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?,
            SyntaxMode::Braces => (),
        }

        let (attributes ,methods,constructor)= self.parse_class_body()?;

        println!("Fin du parsing de la classe OK!!!!!!!!!!!!!!!!!!!!!!");


        Ok(ASTNode::Declaration(Declaration::Class(ClassDeclaration{
            name,
            parent_classes,
            attributes,
            constructor,
            methods,
            visibility,

        })))

    }


    pub fn parse_class_inheritance(&mut self) -> Result<Vec<String>,ParserError>{
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
                            return Err(ParserError::new(MultipleConstructors, self.current_position()));
                        }
                        // parse un constructor
                        let ctor = self.parse_constructor_declaration()?;
                        constructor = Some(ctor);
                    } else if self.check(&[TokenType::KEYWORD(Keywords::FN)]) {
                        let method = self.parse_methode_declaration()?;
                        methods.push(method);
                    } else if self.check(&[TokenType::KEYWORD(Keywords::LET)]) {
                        let attribute = self.parse_attribute_declaration()?;
                        attributes.push(attribute);
                    } else {
                        return Err(ParserError::new(UnexpectedToken, self.current_position()));
                    }
                }
                self.consume(TokenType::DELIMITER(Delimiters::RCURBRACE))?;
            }
            SyntaxMode::Indentation => {
                self.consume(TokenType::NEWLINE)?;
                self.consume(TokenType::INDENT)?;
                while !self.check(&[TokenType::EOF, TokenType::DEDENT]) && !self.is_at_end() {
                    if self.check(&[TokenType::KEYWORD(Keywords::DEF)]) {
                        if constructor.is_some() {
                            return Err(ParserError::new(MultipleConstructors, self.current_position()));
                        }
                        let ctor = self.parse_constructor_declaration()?;
                        constructor = Some(ctor);
                    } else if self.check(&[TokenType::KEYWORD(Keywords::FN)]) {
                        let method = self.parse_methode_declaration()?;
                        methods.push(method);
                    } else if self.check(&[TokenType::KEYWORD(Keywords::LET)]) {
                        let attribute = self.parse_attribute_declaration()?;
                        attributes.push(attribute);
                    } else {
                        return Err(ParserError::new(UnexpectedToken, self.current_position()));
                    }
                }
                if !self.match_token(&[TokenType::DEDENT]){
                    self.consume(TokenType::DEDENT)?;
                }
            }
        }
        Ok((attributes, methods, constructor))
    }


    fn parse_constructor_declaration(&mut self) -> Result<Constructor,ParserError>{
        println!("Debut du parsing du constructeur");
        self.consume(TokenType::KEYWORD(Keywords::DEF))?;
        let constructor_name = self.consume_identifier()?;
        if constructor_name != "init"{
            return Err(ParserError::new(UnexpectedToken, self.current_position()));
        }
        self.consume(TokenType::DELIMITER(Delimiters::LPAR))?;
        let parameters = self.parse_function_parameters()?;
        self.consume(TokenType::DELIMITER(Delimiters::RPAR))?;

        // self.match_token(&[TokenType::OPERATOR(Operators::RARROW)]){
        //
        // };

        let body = self.parse_block()?;

        println!("Fin du parsing du constructeur OK!!!!!!!!!!!!!!!!!!!!!!");

        Ok(Constructor{
            name: constructor_name,
            parameters,
            body,
        })

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

    fn parse_trait_methods(&mut self) -> Result<TraitMethod, ParserError> {
        println!("Début du parsing de la signature de méthode de trait");
        self.consume(TokenType::KEYWORD(Keywords::FN))?;
        let name = self.consume_identifier()?;
        self.consume(TokenType::DELIMITER(Delimiters::LPAR))?;
        let parameters = self.parse_function_parameters()?;
        self.consume(TokenType::DELIMITER(Delimiters::RPAR))?;

        let return_type = if self.check(&[TokenType::OPERATOR(Operators::RARROW)]) {

            self.consume(TokenType::OPERATOR(Operators::RARROW))?;
            Some(self.parse_type()?)
        } else {
            None
        };

        self.consume_seperator();

        println!("Parsing de Trait Method OK!!!!!!!!!!!!!!!!!!!!!!!");

        Ok(TraitMethod {
            name,
            parameters,
            return_type,
        })
    }


    pub fn parse_where_clauses(&mut self) -> Result<Vec<WhereClause>,ParserError>{
        println!("Début du parsing des clauses where");

        // self.consume(TokenType::KEYWORD(Keywords::WHERE))?;

        let mut clauses = Vec::new();

        if self.check(&[TokenType::KEYWORD(Keywords::WHERE)]) /*&& !self.is_at_end()*/{
            self.consume(TokenType::KEYWORD(Keywords::WHERE))?;
            loop {
                let type_name = self.consume_identifier()?;
                self.consume(TokenType::DELIMITER(Delimiters::COLON))?;

                let bounds = self.parse_type_bounds()?;
                clauses.push(WhereClause{
                    type_name,
                    bounds,
                });
                if self.check(&[TokenType::DELIMITER(Delimiters::COMMA)]){
                    self.advance();
                }else { break; }
            }
        }
        println!("Parsing des clauses where OK!!!!!!!!!!!!!!!!!!!!!!!");
        Ok(clauses)

    }

    pub fn parse_methode_declaration(&mut self) -> Result<MethodeDeclaration,ParserError>{
        println!("Debut du parsing de la déclaration de méthode");
        // Pour la visibilite de methode dans une classe, je pense que
        // ça serait  mieux de laisse ceci à  "pub class".
        // Une classe publique  rend toutes ses methodes publiques aussi
        // pour let visibilite = self.parse_visibility()?;  pour  l'ast
        // on revoir

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



    /// fonction  pour parser la mutabilité et la visibilité
    pub fn parse_mutability(&mut self) -> Result<Mutability, ParserError> {
        if self.match_token(&[TokenType::KEYWORD(Keywords::MUT)]){
            Ok(Mutability::Mutable)
        } else {
            Ok(Mutability::Immutable)
        }
    }
    pub fn parse_visibility(&mut self) -> Result<Visibility, ParserError> {
        if self.match_token(&[TokenType::KEYWORD(Keywords::PUB)]){
            Ok(Visibility::Public)
        } else {
            Ok(Visibility::Private)
        }
    }

    ///fonction pour parser les champs de structure STRUCT

    pub fn parse_struct_fields(&mut self) -> Result<Vec<Field>, ParserError> {
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
    pub fn parse_struct_field(&mut self) -> Result<Field, ParserError> {
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

    pub fn parse_enum_variantes(&mut self) -> Result<Vec<EnumVariant>,ParserError>{
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

    pub fn parse_enum_variant_fields(&mut self) ->  Result<EnumVariant,ParserError>{
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





    // Declaration des Array


    pub fn parse_array_expression(&mut self) -> Result<Expression, ParserError> {
        println!("Début du parsing d'un tableau");

        // Consommer '['
        self.consume(TokenType::DELIMITER(Delimiters::LSBRACKET))?;

        let mut elements = Vec::new();

        // Vérifier si le tableau est vide
        if self.check(&[TokenType::DELIMITER(Delimiters::RSBRACKET)]) {
            self.advance();
            return Ok(Expression::Array(ArrayExpression { elements }));
        }

        // Parser le premier élément
        elements.push(self.parse_expression(0)?);

        // Vérifier si c'est une initialisation répétée [value; size]
        if self.check(&[TokenType::DELIMITER(Delimiters::SEMICOLON)]) {
            self.advance();
            let size = self.parse_expression(0)?;
            self.consume(TokenType::DELIMITER(Delimiters::RSBRACKET))?;
            return Ok(Expression::ArrayRepeat(ArrayRepeatExpression {
                value: Box::new(elements.remove(0)),
                size: Box::new(size),
            }));
        }
        // if self.check(&[TokenType::DELIMITER(Delimiters::COLON)]) {
        //     self.advance();
        //     let slice = self.parse_array_slice(elements.remove(0))?;
        //     return Ok(slice);
        //
        // }

        // Parser le reste des éléments
        while self.check(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
            self.advance();
            if self.check(&[TokenType::DELIMITER(Delimiters::RSBRACKET)]) {
                break;
            }
            elements.push(self.parse_expression(0)?);
        }

        // Consommer ']'
        self.consume(TokenType::DELIMITER(Delimiters::RSBRACKET))?;

        println!("Fin du parsing d'un tableau");
        Ok(Expression::Array(ArrayExpression { elements }))
    }



    // pub fn parse_array_slice(&mut self, array: Expression) -> Result<Expression, ParserError>{
    //
    //     let start = if !self.check(&[
    //         TokenType::OPERATOR(Operators::DOTDOT),
    //         TokenType::OPERATOR(Operators::DOTDOTEQUAL)
    //     ]) {
    //         Some(Box::new(self.parse_expression(0)?))
    //     } else {
    //         None
    //     };
    //
    //     let inclusive = if self.check(&[TokenType::OPERATOR(Operators::DOTDOTEQUAL)]) {
    //         self.consume(TokenType::OPERATOR(Operators::DOTDOTEQUAL))?;
    //         true
    //     } else {
    //         self.consume(TokenType::OPERATOR(Operators::DOTDOT))?;
    //         false
    //     };
    //
    //     let end = if !self.check(&[
    //         TokenType::DELIMITER(Delimiters::RSBRACKET),
    //         TokenType::DELIMITER(Delimiters::COLON)
    //     ]) {
    //         Some(Box::new(self.parse_expression(0)?))
    //     } else {
    //         None
    //     };
    //
    //     let step = if self.check(&[TokenType::DELIMITER(Delimiters::COLON)]) {
    //         self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
    //         Some(Box::new(self.parse_expression(0)?))
    //     } else {
    //         None
    //     };
    //
    //     self.consume(TokenType::DELIMITER(Delimiters::RSBRACKET))?;
    //
    //     println!("Fin du parsing d'une tranche de tableau OK!!!!!!!!!!!!!!!!!!!!!!!");
    //
    //     Ok(Expression::ArraySlice(ArraySlice{
    //         array: Box::new(array),
    //         start,
    //         end,
    //         step,
    //         inclusive,
    //     }))
    //
    // }
    // pub fn parse_slice(&mut self) -> Result<Expression, ParserError> {
    //     let index = self.parse_expression(0)?;
    //     self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
    //
    //     let end = if self.check(&[TokenType::DELIMITER(Delimiters::COLON)]) ||
    //         self.check(&[TokenType::DELIMITER(Delimiters::RSBRACKET)]) {
    //         None
    //     } else {
    //         Some(Box::new(self.parse_expression(0)?))
    //     };
    //
    //     let step = if self.check(&[TokenType::DELIMITER(Delimiters::COLON)]) {
    //         self.advance();
    //         Some(Box::new(self.parse_expression(0)?))
    //     } else {
    //         None
    //     };
    //
    //     self.consume(TokenType::DELIMITER(Delimiters::RSBRACKET))?;
    //
    //     Ok(Expression::Slice(Slice {
    //         start: Some(Box::new(index)),
    //         end,
    //         step
    //     }))
    // }









    pub fn parse_list_comprehension(&mut self) -> Result<Expression, ParserError> {
        println!("Début du parsing de list comprehension");

        // Consommer '['
        self.consume(TokenType::DELIMITER(Delimiters::LSBRACKET))?;

        // Parser l'expression à générer
        let elements = Box::new(self.parse_expression(0)?);

        // Doit être suivi par 'for'
        self.consume(TokenType::KEYWORD(Keywords::FOR))?;

        let mut iterators = Vec::new();
        let mut conditions = Vec::new();

        // Parser la première boucle for (obligatoire)
        iterators.push(self.parse_comprehension_for()?);

        // Parser les boucles for et conditions supplémentaires
        while !self.check(&[TokenType::DELIMITER(Delimiters::RSBRACKET)]) {
            match self.current_token() {
                Some(token) => match &token.token_type {
                    TokenType::KEYWORD(Keywords::FOR) => {
                        self.advance();
                        iterators.push(self.parse_comprehension_for()?);
                    }
                    TokenType::KEYWORD(Keywords::IF) => {
                        self.advance();
                        conditions.push(self.parse_expression(0)?);
                    }
                    _ => return Err(ParserError::new(UnexpectedToken, self.current_position())),
                },
                None => return Err(ParserError::new(UnexpectedToken, self.current_position())),
            }
        }

        // Consommer ']'
        self.consume(TokenType::DELIMITER(Delimiters::RSBRACKET))?;

        println!("Fin du parsing de list comprehension");

        Ok(Expression::ListComprehension(ListComprehension{
            elements,
            iterators,
            conditions,
        }))
    }

    pub fn parse_comprehension_for(&mut self) -> Result<ComprehensionFor, ParserError> {
        println!("Début du parsing de la boucle for de list comprehension");
        let pattern = self.parse_pattern()?;
        self.consume(TokenType::KEYWORD(Keywords::IN))?;
        let iterator = self.parse_expression(0)?;
        println!("Fin du parsing de la boucle for de list comprehension");
        Ok(ComprehensionFor {
            pattern,
            iterator })

    }


    pub fn is_list_comprehension(&mut self) -> Result<bool, ParserError> {
        // Sauvegarder la position actuelle
        let current = self.current;

        // Vérifier si c'est une list comprehension
        let mut is_comprehension = false;

        // Consommer '['
        if self.check(&[TokenType::DELIMITER(Delimiters::LSBRACKET)]) {
            self.advance();

            // Chercher un 'for' après une expression
            while let Some(token) = self.current_token() {
                if matches!(token.token_type, TokenType::KEYWORD(Keywords::FOR)) {
                    is_comprehension = true;
                    break;
                }
                if matches!(token.token_type, TokenType::DELIMITER(Delimiters::RSBRACKET)) {
                    break;
                }
                self.advance();
            }
        }

        // Restaurer la position
        self.current = current;

        Ok(is_comprehension)
    }

    pub fn parse_array_access(&mut self,array:Expression)  -> Result<Expression,ParserError>{
        self.consume(TokenType::DELIMITER(Delimiters::LSBRACKET))?;
        let index = self.parse_expression(0)?;
        self.consume(TokenType::DELIMITER(Delimiters::RSBRACKET))?;
        Ok(Expression::ArrayAccess(ArrayAccess {
            array: Box::new(array),
            index: Box::new(index),
        }))
    }

    pub fn parse_dict_access(&mut self,dict:Expression) -> Result<Expression,ParserError>{
        println!("Debut du parsing d'un accès à un dictionnaire");
        self.consume(TokenType::DELIMITER(Delimiters::LSBRACKET))?;
        let key = self.parse_expression(0)?;
        self.consume(TokenType::DELIMITER(Delimiters::RSBRACKET))?;
        Ok(Expression::DictAccess(DictAccess {
            dict: Box::new(dict),
            key: Box::new(key),
        }))
    }

    pub fn parse_dict_literal(&mut self) -> Result<Expression, ParserError> {
        println!("Debut du parsing d'un dictionnaire");
        self.consume(TokenType::DELIMITER(Delimiters::LCURBRACE))?;

        // verifie si c'est un dictionnaire vide
        if self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]){
            self.advance();
            println!("Fin du parsing du dictionnaire OK!!!!!!!!!!!!!!!!!!!!!!!");
            return Ok(Expression::DictLiteral(DictLiteral { entries: vec![] }));
        }


        // Parser la première paire clé-valeur
        let key = self.parse_expression(0)?;
        self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
        let value = self.parse_expression(0)?;

        // Vérifier si c'est une dict comprehension
        if self.check(&[TokenType::KEYWORD(Keywords::FOR)]) {
            self.advance(); // Consommer le 'for'
            return self.parse_dict_comprehension(key, value);
        }

        // Si ce n'est pas une dict comprehension, continuer avec un dictionnaire normal
        let mut entries = vec![DictEntry {
            key: Box::new(key),
            value: Box::new(value),
        }];

        while !self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]) {
            self.consume(TokenType::DELIMITER(Delimiters::COMMA))?;

            if self.check(&[TokenType::DELIMITER(Delimiters::RCURBRACE)]) {
                break;
            }

            let key = self.parse_expression(0)?;
            self.consume(TokenType::DELIMITER(Delimiters::COLON))?;
            let value = self.parse_expression(0)?;

            entries.push(DictEntry {
                key: Box::new(key),
                value: Box::new(value),
            });
        }

        self.consume(TokenType::DELIMITER(Delimiters::RCURBRACE))?;
        println!("Fin du parsing d'un dictionnaire OK!!!!!!!!!!!!!!!!!!!!!!!");

        Ok(Expression::DictLiteral(DictLiteral { entries }))
    }


    pub fn parse_dict_comprehension(&mut self, key_expr: Expression, value_expr: Expression) -> Result<Expression, ParserError> {
        println!("Début du parsing d'une dict comprehension");

        let mut iterators = Vec::new();
        let mut conditions = Vec::new();

        // Parse la partie itération
        loop {
            let mut targets = Vec::new();

            // Parser la première variable
            if let Some(Token { token_type: TokenType::IDENTIFIER { name }, .. }) = self.current_token() {
                targets.push(Expression::Identifier(name.clone()));
                self.advance();
            } else {
                return Err(ParserError::new(UnexpectedToken, self.current_position()));
            }

            // Parser d'autres variables si présentes
            while self.check(&[TokenType::DELIMITER(Delimiters::COMMA)]) {
                self.advance(); // Consomme la virgule
                if let Some(Token { token_type: TokenType::IDENTIFIER { name }, .. }) = self.current_token() {
                    targets.push(Expression::Identifier(name.clone()));
                    self.advance();
                } else {
                    return Err(ParserError::new(UnexpectedToken, self.current_position()));
                }
            }

            // Parse 'in' et l'itérable
            self.consume(TokenType::KEYWORD(Keywords::IN))?;
            let iterator = self.parse_expression(0)?;

            // Parse les conditions if
            if self.check(&[TokenType::KEYWORD(Keywords::IF)]) {
                self.advance(); // Consomme 'if'
                let condition = self.parse_expression(0)?;
                conditions.push(condition);
            }

            iterators.push(CompFor {
                targets,
                iterator: Box::new(iterator),
                conditions: vec![], // Les conditions sont maintenant stockées au niveau supérieur
            });

            // Vérifie s'il y a un autre 'for'
            if !self.check(&[TokenType::KEYWORD(Keywords::FOR)]) {
                break;
            }
            self.advance();
        }

        self.consume(TokenType::DELIMITER(Delimiters::RCURBRACE))?;

        println!("Fin du parsing de la dict comprehension");

        Ok(Expression::DictComprehension(DictComprehension {
            key_expr: Box::new(key_expr),
            value_expr: Box::new(value_expr),
            iterators,
            conditions,
        }))
    }







}


