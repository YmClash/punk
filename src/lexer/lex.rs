use crate::lexer_error::{LexerError, LexerErrorType, Position};
use crate::tok::{Delimiters, Keywords, Operators, StringKind, TokenType};
use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;
//use crate::lexer::error::{LexerError, LexerErrorType, Position};

//#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SyntaxMode {
    Indentation, //Python Like Syntax mode
    Braces,      //Rust Like Syntax mode
}

/// Structure Token,
/// elle contient le text du token, le type du token, la ligne et la colonne
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub token_type: TokenType,
    pub(crate) line: usize,
    pub(crate) column: usize,
}

/// Implementation de la structure Token
#[allow(dead_code)]
impl Token {
    pub fn new(text: String, token_type: TokenType, line: usize, column: usize) -> Self {
        Token {
            text,
            token_type,
            line,
            column,
        }
    }
}

/// structure Lexer
#[allow(dead_code)]
pub struct Lexer<'a> {
    source: Peekable<Chars<'a>>,
    current_char: char,
    keywords: HashMap<String, Keywords>,
    operators: HashMap<String, Operators>,
    delimiters: HashMap<String, Delimiters>,
    current_line: usize,
    current_column: usize,
    current_token_text: String,
    syntax_mode: SyntaxMode,
    indent_level: Vec<usize>,
    at_line_start: bool,
    // nesting :usize
}

/// Implementation du lexer avec tous les methodes pour classer les tokens
#[allow(dead_code)]
impl<'a> Lexer<'a> {
    /// Creation d'une nouvelle instance de lexer
    pub fn new(code_source: &'a str, syntax_mode: SyntaxMode) -> Self {
        let lexer = Lexer {
            source: code_source.chars().peekable(),
            current_char: '\0',
            keywords: Self::keywords(),
            operators: Self::operators(),
            delimiters: Self::delimiters(),
            current_line: 1,
            current_column: 1,
            current_token_text: String::new(),
            syntax_mode,
            indent_level: vec![0],
            at_line_start: true,
            //  nesting: 0,
        };
        lexer
    }

    /// La Logique de la Methode pour compter l'indentation a ete trasferer dans la methode get_token()
    /// Methode pour compter l'indentation

    pub fn count_indentation(&mut self) -> usize {
        let mut count = 0;
        while let Some(&ch) = self.source.peek() {
            match ch {
                ' ' => count += 1,
                '\t' => count += 4, // ou une autre valeur selon votre convention
                _ => break,
            }
            self.advance();
        }
        count
    }

    /// Creation d'une hashmap pour les mots cles
    /// c'est plus facile de les stocker les mots cles dans une hashmap pour les retrouver plus facilement
    fn keywords() -> HashMap<String, Keywords> {
        let mut keywords = HashMap::new();
        keywords.insert("and".to_string(), Keywords::AND);
        keywords.insert("as".to_string(), Keywords::AS);
        keywords.insert("async".to_string(), Keywords::ASYNC);
        keywords.insert("await".to_string(), Keywords::AWAIT);
        keywords.insert("break".to_string(), Keywords::BREAK);
        keywords.insert("const".to_string(), Keywords::CONST);
        keywords.insert("continue".to_string(), Keywords::CONTINUE);
        keywords.insert("class".to_string(), Keywords::CLASS);
        keywords.insert("def".to_string(), Keywords::DEF);
        keywords.insert("del".to_string(), Keywords::DEL);
        keywords.insert("elif".to_string(), Keywords::ELIF);
        keywords.insert("else".to_string(), Keywords::ELSE);
        keywords.insert("enum".to_string(), Keywords::ENUM);
        keywords.insert("except".to_string(), Keywords::EXCEPT);
        keywords.insert("false".to_string(), Keywords::FALSE);
        keywords.insert("fn".to_string(), Keywords::FN);
        keywords.insert("for".to_string(), Keywords::FOR);
        keywords.insert("from".to_string(), Keywords::FROM);
        keywords.insert("if".to_string(), Keywords::IF);
        keywords.insert("impl".to_string(), Keywords::IMPL);
        keywords.insert("import".to_string(), Keywords::IMPORT);
        keywords.insert("in".to_string(), Keywords::IN);
        keywords.insert("is".to_string(), Keywords::IS);
        keywords.insert("lambda".to_string(), Keywords::LAMBDA);
        keywords.insert("let".to_string(), Keywords::LET);
        keywords.insert("loop".to_string(), Keywords::LOOP);
        keywords.insert("match".to_string(), Keywords::MATCH);
        keywords.insert("mod".to_string(), Keywords::MOD);
        keywords.insert("mut".to_string(), Keywords::MUT);
        keywords.insert("none".to_string(), Keywords::NONE);
        keywords.insert("not".to_string(), Keywords::NOT);
        keywords.insert("or".to_string(), Keywords::OR);
        keywords.insert("pub".to_string(), Keywords::PUB);
        keywords.insert("pass".to_string(), Keywords::PASS);
        keywords.insert("raise".to_string(), Keywords::RAISE);
        keywords.insert("return".to_string(), Keywords::RETURN);
        keywords.insert("self".to_string(), Keywords::SELF);
        keywords.insert("static".to_string(), Keywords::STATIC);
        keywords.insert("struct".to_string(), Keywords::STRUCT);
        keywords.insert("super".to_string(), Keywords::SUPER);
        keywords.insert("trait".to_string(), Keywords::TRAIT);
        keywords.insert("true".to_string(), Keywords::TRUE);
        keywords.insert("try".to_string(), Keywords::TRY);
        keywords.insert("type".to_string(), Keywords::TYPE);
        keywords.insert("typeof".to_string(), Keywords::TYPEOF);
        keywords.insert("use".to_string(), Keywords::USE);
        keywords.insert("with".to_string(), Keywords::WITH);
        keywords.insert("where".to_string(), Keywords::WHERE);
        keywords.insert("while".to_string(), Keywords::WHILE);
        keywords.insert("yield".to_string(), Keywords::YIELD);
        //TYPE KEYWORDS
        keywords.insert("int".to_string(), Keywords::INT);
        keywords.insert("float".to_string(), Keywords::FLOAT);
        keywords.insert("str".to_string(), Keywords::STR);
        keywords.insert("bool".to_string(), Keywords::BOOL);
        keywords.insert("char".to_string(), Keywords::CHAR);

        return keywords;
    }
    /// creation d'une hashmap pour les operateurs
    fn operators() -> HashMap<String, Operators> {
        let mut operators = HashMap::new();
        operators.insert("+".to_string(), Operators::PLUS);
        operators.insert("-".to_string(), Operators::MINUS);
        operators.insert("*".to_string(), Operators::STAR);
        operators.insert("/".to_string(), Operators::SLASH);
        operators.insert("%".to_string(), Operators::PERCENT);
        operators.insert("==".to_string(), Operators::EQEQUAL);
        operators.insert("!=".to_string(), Operators::NOTEQUAL);
        operators.insert("<".to_string(), Operators::LESS);
        operators.insert(">".to_string(), Operators::GREATER);
        operators.insert("<=".to_string(), Operators::LESSEQUAL);
        operators.insert(">=".to_string(), Operators::GREATEREQUAL);
        operators.insert("=".to_string(), Operators::EQUAL);
        operators.insert("++".to_string(), Operators::PLUSEQUAL);
        operators.insert("--".to_string(), Operators::MINEQUAL);
        operators.insert("**".to_string(), Operators::DOUBLESTAR);
        //operators.insert("//".to_string(), Operators::DOUBLESLASH);
        operators.insert("&&".to_string(), Operators::AND);
        operators.insert("||".to_string(), Operators::OR);
        operators.insert("!".to_string(), Operators::EXCLAMATION);
        operators.insert("&".to_string(), Operators::AMPER);
        operators.insert("|".to_string(), Operators::VBAR);
        operators.insert("^".to_string(), Operators::CIRCUMFLEX);
        operators.insert("<<".to_string(), Operators::LEFTSHIFT);
        operators.insert(">>".to_string(), Operators::RIGHTSHIFT);
        operators.insert("~".to_string(), Operators::TILDE);
        operators.insert("+=".to_string(), Operators::PLUSEQUAL);
        operators.insert("-=".to_string(), Operators::MINEQUAL);
        operators.insert("*=".to_string(), Operators::STAREQUAL);
        operators.insert("/=".to_string(), Operators::SLASHEQUAL);
        operators.insert("%=".to_string(), Operators::PERCENTEQUAL);
        operators.insert("&=".to_string(), Operators::AMPEREQUAL);
        operators.insert("|=".to_string(), Operators::VBAREQUAL);
        operators.insert("^=".to_string(), Operators::CIRCUMFLEXEQUAL);
        operators.insert("<<=".to_string(), Operators::LEFTSHIFTEQUAL);
        operators.insert(">>=".to_string(), Operators::RIGHTSHIFTEQUAL);
        operators.insert("**=".to_string(), Operators::DOUBLESTAREQUAL);
        operators.insert("//=".to_string(), Operators::DOUBLESLASHEQUAL);
        operators.insert("@".to_string(), Operators::AT);
        operators.insert("@=".to_string(), Operators::ATEQUAL);
        operators.insert("->".to_string(), Operators::RARROW);
        operators.insert(":=".to_string(), Operators::COLONEQUAL);
        operators.insert("*/".to_string(), Operators::STARSLASH);
        operators.insert("/*".to_string(), Operators::SLASHSTAR);
        operators.insert("#".to_string(), Operators::DIESE);
        operators.insert("?".to_string(), Operators::INTERROGATION);

        operators.insert("_".to_string(), Operators::UNDERSCORE);
        operators.insert("=>".to_string(), Operators::FATARROW);

        operators.insert("..".to_string(), Operators::DOTDOT);
        operators.insert("..=".to_string(), Operators::DOTDOTEQUAL);


        return operators;
    }

    /// Creation d'une hashmap pour les delimiters
    fn delimiters() -> HashMap<String, Delimiters> {
        let mut delimiters = HashMap::new();
        delimiters.insert("(".to_string(), Delimiters::LPAR);
        delimiters.insert(")".to_string(), Delimiters::RPAR);
        delimiters.insert("{".to_string(), Delimiters::LCURBRACE);
        delimiters.insert("}".to_string(), Delimiters::RCURBRACE);
        delimiters.insert("]".to_string(), Delimiters::RSBRACKET);
        delimiters.insert("[".to_string(), Delimiters::LSBRACKET);
        delimiters.insert(";".to_string(), Delimiters::SEMICOLON);
        delimiters.insert(":".to_string(), Delimiters::COLON);
        delimiters.insert(",".to_string(), Delimiters::COMMA);
        delimiters.insert(".".to_string(), Delimiters::DOT);
        delimiters.insert("...".to_string(), Delimiters::ELLIPSIS);
        delimiters.insert("::".to_string(), Delimiters::DOUBLECOLON);


        // delimiters.insert("..=".to_string(), Delimiters::DOTDOTEQUAL);
        // delimiters.insert("..".to_string(), Delimiters::DOTDOT);

        return delimiters;
    }

    /// Methode pour avancer d'un caractere
    #[allow(dead_code)]
    fn next_char(&mut self) -> Option<char> {
        let ch = self.source.next()?;
        self.current_char = ch;
        if ch == '\n' {
            self.current_line += 1;
            self.current_column = 1;
            self.at_line_start = true;
        } else {
            self.current_column += 1;
            self.at_line_start = false;
        }
        Some(ch)
    }

    /// Methode pour regarder le prochain caractere sans avancer
    #[allow(dead_code)]
    fn peek_char(&mut self) -> Option<char> {
        self.source.peek().copied()
    }
    /// Methode pour regarder le 2eme prochain caractere sans avancer
    #[allow(dead_code)]
    fn peek_next_char(&mut self) -> Option<char> {
        self.source.clone().nth(1)
    }

    fn peek_next(&mut self) -> Option<char> {
     let mut iter = self.source.clone();
        iter.next();
        iter.peek().copied()
    }

    /// C'est L'une de 2 methode principal avec fonction tokenize() pour obtenir le token
    /// son role c'est de sauter les espaces et examiner le prochain caractère
    /// Détermine le type de token en fonction de ce caractère.
    /// Appelle la méthode appropriée (comme lex_number(), lex_identifier_or_keyword(), etc.) pour analyser le token complet.
    /// Renvoie une Option<TokenType> représentant un seul token.

    /// methode pour obtenir le token

    pub fn get_token(&mut self) -> Option<TokenType> {
        //Gérer l'indentation au début d'une nouvelle ligne
        if self.at_line_start && self.syntax_mode == SyntaxMode::Indentation {
            self.at_line_start = false;
            let current_indent = self.count_indentation();
            let previous_indent = *self.indent_level.last().unwrap_or(&0);

            if current_indent > previous_indent {
                self.indent_level.push(current_indent);
                return Some(TokenType::INDENT);
            } else if current_indent < previous_indent {
                while current_indent < *self.indent_level.last().unwrap_or(&0) {
                    self.indent_level.pop();
                    return Some(TokenType::DEDENT);
                }
                if current_indent != *self.indent_level.last().unwrap_or(&0) {
                    return Some(TokenType::ERROR(LexerError::invalid_indentation(
                        Position {
                            line: self.current_line,
                            column: self.current_column,
                        },
                    )));
                }
            }
            // Si l'indentation est la même, on ne fait rien de spécial
        }

        self.skip_whitespace(); // Sauter les espaces et tabulations

        // Vérifier le prochain caractère
        match self.peek_char() {
            Some('\n') => {
                self.advance(); // Consomme le '\n'
                self.at_line_start = true;
            //    return Some(TokenType::NEWLINE);
                // retourn NEWLINE  seulement en  mode Indentation
                if self.syntax_mode == SyntaxMode::Indentation {
                    return Some(TokenType::NEWLINE);
                }else {
                    // en mode Brace, on ignore le newline et passe au token suivant
                    return self.get_token();
                }
            }


            Some('0'..='9') => Some(self.lex_number()),
            Some('a'..='z') | Some('A'..='Z') | Some('_') => Some(self.lex_identifier_or_keyword()),
            Some('"') | Some('\'') => Some(self.lex_string()),
            Some('#') => Some(self.lex_comment()),
            Some('/') => {
                if let Some(next_char) = self.peek_next_char() {
                    match next_char {
                        '/' | '*' => Some(self.lex_comment()),
                        _ => self.lex_operator(),
                    }
                } else {
                    self.lex_operator()
                }
            }
            Some(ch) if self.delimiters.contains_key(&ch.to_string()) => Some(self.lex_delimiter()),
            Some(ch) if !ch.is_alphanumeric() => self.lex_operator(),
            None => {
                ////////////à surveiller si c'est correct et pas redondant
                // Fin du fichier (EOF)
                if !self.indent_level.is_empty() && *self.indent_level.last().unwrap() > 0 {
                    self.indent_level.pop();
                    return Some(TokenType::DEDENT);
                } else {
                    return Some(TokenType::EOF);
                }
            }
            ////////////à surveiller si c'est correct et pas redondant

            //None => Some(TokenType::EOF),   //Ajouter nouvelement
            _ => Some(self.lex_unknown()),
        }
    }

    fn lex_number(&mut self) -> TokenType {
        self.current_token_text.clear();

        if self.peek_char() == Some('0') {
            if let Some(next_char) = self.peek_next() {
                if next_char == 'x' || next_char == 'X' {
                    let ch1 = self.advance(); // '0'
                    let ch2 = self.advance(); // 'x' ou 'X'
                    self.current_token_text.push(ch1);
                    self.current_token_text.push(ch2);
                    return self.lex_hexadecimal();
                }
            }
        }

        let mut number = String::new();
        let mut dot_count = 0;

        while let Some(&ch) = self.source.peek() {
            if ch.is_digit(10) {
                let digit = self.advance();
                number.push(digit);
                self.current_token_text.push(digit);
            } else if ch == '.' {
                // Vérifier si le '.' est suivi d'un chiffre pour un nombre flottant
                if dot_count == 0 {
                    if let Some(next_ch) = self.peek_next_char() {
                        if next_ch.is_digit(10) {
                            let dot = self.advance();
                            number.push(dot);
                            self.current_token_text.push(dot);
                            dot_count += 1;
                        } else {
                            // Le '.' n'est pas suivi d'un chiffre, il pourrait faire partie d'un opérateur
                            break;
                        }
                    } else {
                        // Fin de l'entrée après '.', ce qui est une erreur pour un float
                        break;
                    }
                } else {
                    // Deuxième point trouvé, c'est une erreur pour un float
                    break;
                }
            } else {
                break;
            }
        }



        // while let Some(&ch) = self.source.peek() {
        //     if ch.is_digit(10) {
        //         let digit = self.advance();
        //         number.push(digit);
        //         self.current_token_text.push(digit);
        //     } else if ch == '.' {
        //         if dot_count == 0 {
        //             let dot = self.advance();
        //             number.push(dot);
        //             self.current_token_text.push(dot);
        //             dot_count += 1;
        //         } else {
        //             // Deuxième point trouvé, c'est une erreur
        //             while let Some(&next_ch) = self.source.peek() {
        //                 if next_ch.is_digit(10) || next_ch == '.' {
        //                     let ch = self.advance();
        //                     number.push(ch);
        //                     self.current_token_text.push(ch);
        //                 } else {
        //                     break;
        //                 }
        //             }
        //
        //         }
        //     } else {
        //         break;
        //     }
        // }

        if number.is_empty() {
            return self.create_error(LexerErrorType::InvalidInteger(number));
        }

        if dot_count > 0 {
            match number.parse::<f64>() {
                Ok(value) => TokenType::FLOAT { value },
                Err(_) => self.create_error(LexerErrorType::InvalidFloat(number)),
            }
        } else {
            match number.parse::<i64>() {
                Ok(value) => TokenType::INTEGER {
                    value: value.into(),
                },
                Err(_) => self.create_error(LexerErrorType::InvalidInteger(number)),
            }
        }
    }

    // savoir si c'est un  hexdigit
    fn is_hex_digit(ch: char) -> bool {
        ch.is_digit(16)
    }

    fn lex_hexadecimal(&mut self) -> TokenType {
        let mut hex_number = self.current_token_text.clone(); // Déjà contient "0x" ou "0X"

        while let Some(&ch) = self.source.peek() {
            if Self::is_hex_digit(ch) {
                hex_number.push(self.advance());
            } else {
                break;
            }
        }

        self.current_token_text = hex_number.clone();

        if hex_number.len() == 2 {
            // Seulement "0x" ou "0X"
            self.create_error(LexerErrorType::InvalidHexadecimal(hex_number))
        } else {
            match u64::from_str_radix(&hex_number[2..], 16) {
                // Skip "0x"
                Ok(value) => TokenType::HEXADECIMAL { value },
                Err(_) => self.create_error(LexerErrorType::InvalidHexadecimal(hex_number)),
            }
        }
    }

    //fn lex_identifier(){}
    /// Methode pour les different types de token de Type Identifier ou Keyword
    fn lex_identifier_or_keyword(&mut self) -> TokenType {
        self.current_token_text.clear();
        while let Some(&ch) = self.source.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                let ch = self.advance();
                self.current_token_text.push(ch); // Ajoute le caractère à la chaîne de texte du token
            } else {
                break;
            }
        }

        if let Some(keyword) = self.keywords.get(&self.current_token_text) {
            TokenType::KEYWORD(keyword.clone()) // c'est un mot clé
        } else {
            TokenType::IDENTIFIER {
                name: self.current_token_text.clone(),
            } // sinon c'est un identifiant
        }
    }

    fn lex_string(&mut self) -> TokenType {
        self.current_token_text.clear();

        let quote = self.advance(); // Consomme le premier guillemet
        let mut value = String::new();
        let mut is_escaped = false;

        while let Some(&ch) = self.source.peek() {
            self.advance(); // Consomme le caractère actuel
            if is_escaped {
                match ch {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    '\'' => value.push('\''),
                    '\n' => {
                        // Ignorer le saut de ligne après un backslash
                        while let Some(&next_ch) = self.source.peek() {
                            if next_ch.is_whitespace() && next_ch != '\n' {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    _ => value.push(ch),
                }
                is_escaped = false;
            } else if ch == '\\' {
                is_escaped = true;
            }
                //But: 'C'->Char, "C"->String
            else if ch == '\'' && value.len() == 1 {        //Note !!!!!!
                // petit probleme  ici^  :Si  value.len() == 1 , c'est un caractère est parsé, mais ne
                //s'affiche pas et si value.len() == 3  le caractère est parsé et s'affiche

                //probleme resolu!!! la ligne suivante resoud le probleme


                self.current_token_text = value.clone();
                return TokenType::CHAR {
                    value: value.chars().next().unwrap()
                }
            }

            else if ch == quote {
                self.current_token_text = value.clone();
                return TokenType::STRING {
                    value,
                    kind: StringKind::NORMAL,
                };
            } else {
                value.push(ch);
            }
        }

        // Si nous sortons de la boucle sans avoir trouvé de guillemet fermant
        self.create_error(LexerErrorType::UnterminatedString)
    }

    fn lex_operator(&mut self) -> Option<TokenType> {
        self.current_token_text.clear();

        // Regardez les deux prochains caractères pour vérifier les opérateurs composés
        let first_char = self.advance();
        self.current_token_text.push(first_char);
        let mut op = self.current_token_text.clone();

        if let Some(&next_char) = self.source.peek() {
            op.push(next_char);
            if self.operators.contains_key(&op) {
                self.advance();
                self.current_token_text.push(next_char);
                return Some(TokenType::OPERATOR(self.operators[&op].clone()));
            }
        }

        // Si ce n'est pas un opérateur composé, vérifiez l'opérateur simple
        if let Some(operator) = self.operators.get(&self.current_token_text) {
            return Some(TokenType::OPERATOR(operator.clone()));
        }

        // Si l'opérateur n'est pas reconnu, retournez une erreur
        Some(TokenType::ERROR(LexerError::invalid_token(
            &self.current_token_text,
            Position {
                line: self.current_line,
                column: self.current_column,
            },
        )))
    }


    /// Methode pour les differents types de token de Type Operator
    // fn lex_operator(&mut self) -> Option<TokenType> {
    //     self.current_token_text.clear();
    //
    //     // Regardez les deux prochains caractères pour vérifier les opérateurs composés
    //     let first_char = self.advance();
    //     self.current_token_text.push(first_char);
    //     let mut op = self.current_token_text.clone();
    //
    //     // Traitement spécial pour les opérateurs de range
    //     if first_char == '.' {
    //         if let Some(&next_char) = self.source.peek() {
    //             if next_char == '.' {
    //                 // Consomme le deuxième '.'
    //                 self.advance();
    //                 self.current_token_text.push(next_char);
    //
    //                 // Regarde si le prochain caractère est '='
    //                 if let Some(&third_char) = self.source.peek() {
    //                     if third_char == '=' {
    //                         self.advance();
    //                         self.current_token_text.push(third_char);
    //                         return Some(TokenType::OPERATOR(Operators::DOTDOTEQUAL));
    //                     }
    //                 }
    //                 return Some(TokenType::OPERATOR(Operators::DOTDOT));
    //             } else {
    //                 // Si c'est juste un point simple
    //                 return Some(TokenType::DELIMITER(Delimiters::DOT));
    //             }
    //         }
    //         // Si c'est juste un point simple sans suite
    //         return Some(TokenType::DELIMITER(Delimiters::DOT));
    //     }
    //
    //     // Gestion des autres opérateurs composés
    //     if let Some(&next_char) = self.source.peek() {
    //         let mut combined = self.current_token_text.clone();
    //         combined.push(next_char);
    //
    //         if self.operators.contains_key(&combined) {
    //             self.advance();
    //             self.current_token_text = combined;
    //             return Some(TokenType::OPERATOR(self.operators[&self.current_token_text].clone()));
    //         }
    //     }
    //
    //
    //     // if let Some(&next_char) = self.source.peek() {
    //     //     let mut op = self.current_token_text.clone();
    //     //     op.push(next_char);
    //     //     if self.operators.contains_key(&op) {
    //     //         self.advance();
    //     //         self.current_token_text.push(next_char);
    //     //         return Some(TokenType::OPERATOR(self.operators[&op].clone()));
    //     //     }
    //     //     if op == ".." {
    //     //         self.advance();
    //     //         self.current_token_text.push(next_char);
    //     //
    //     //         if let Some(&third_char) = self.source.peek(){
    //     //             if third_char == '='{
    //     //                 self.advance();
    //     //                 op.push(third_char);
    //     //                 self.current_token_text = op;
    //     //                 return Some(TokenType::OPERATOR(Operators::DOTDOTEQUAL));
    //     //
    //     //             }
    //     //         }
    //     //         return Some(TokenType::OPERATOR(Operators::DOTDOT));
    //     //     }
    //     // }
    //
    //     // Si ce n'est pas un opérateur composé, vérifiez l'opérateur simple
    //     if let Some(operator) = self.operators.get(&self.current_token_text) {
    //         return Some(TokenType::OPERATOR(operator.clone()));
    //     }
    //
    //     // Si l'opérateur n'est pas reconnu, retournez une erreur
    //     Some(TokenType::ERROR(LexerError::invalid_token(
    //         &self.current_token_text,
    //         Position {
    //             line: self.current_line,
    //             column: self.current_column,
    //         },
    //     )))
    // }

    /// Methode pour les differents types de token de Type Delimiter
    fn lex_delimiter(&mut self) -> TokenType {

        self.current_token_text.clear();

        let first_char = self.advance();
        self.current_token_text.push(first_char);

        if let Some(&next_char) = self.source.peek(){
            let mut combined = self.current_token_text.clone();
            combined.push(next_char);

            // verifie pour "::"
            if combined == "::"{
                self.advance();
                self.current_token_text = combined;
                return TokenType::DELIMITER(Delimiters::DOUBLECOLON);
            }

            if first_char == '.' {
                if let Some(&next_char) = self.source.peek(){
                    if next_char == '.'{
                        self.advance();
                        self.current_token_text.push(next_char);

                        if let Some(&third_char) = self.source.peek(){
                            if third_char == '.'{
                                self.advance();
                                self.current_token_text.push(third_char);
                                return TokenType::DELIMITER(Delimiters::ELLIPSIS);
                            }else if third_char == '=' {
                                self.advance();
                                self.current_token_text.push(third_char);
                                return TokenType::OPERATOR(Operators::DOTDOTEQUAL);
                            }
                        }
                        return TokenType::OPERATOR(Operators::DOTDOT);
                    }else {
                        return TokenType::DELIMITER(Delimiters::DOT);
                    }
                }
            }



            // if first_char == '.' && next_char == '.'{
            //     self.advance();
            //     if let Some(&third_char) = self.source.peek(){
            //         if third_char == '.'{
            //             self.advance();
            //             self.current_token_text = "...".to_string();
            //             return TokenType::DELIMITER(Delimiters::ELLIPSIS);
            //         }
            //     }
            //     self.current_token_text = "..".to_string();
            //     return TokenType::DELIMITER(Delimiters::DOTDOT);
            //     //return TokenType::DELIMITER(Delimiters::DOT);
            // }

        }
        if let Some(delimiter) = self.delimiters.get(&self.current_token_text) {
            return TokenType::DELIMITER(delimiter.clone());
        } else {
            return TokenType::UNKNOWN;
        }

        // let ch = self.advance();
        // if let Some(delimiter) = self.delimiters.get(&ch.to_string()) {
        //     self.current_token_text = ch.to_string();
        //     TokenType::DELIMITER(delimiter.clone())
        // } else {
        //     TokenType::UNKNOWN
        // }
    }

    /// Methode pour les differents types de token de Type Comment # ou // ou /* */
    fn lex_comment(&mut self) -> TokenType {
        self.current_token_text.clear();
        let start_char = self.advance(); // Consomme le '/' ou le '#'
        let mut comment = String::new();

        match start_char {
            '#' => {
                // Commentaire en ligne commençant par '#'
                while let Some(ch) = self.next_char() {
                    if ch == '\n' {
                        break;
                    }
                    comment.push(ch);
                }
            }
            '/' => {
                if let Some(&next_char) = self.source.peek() {
                    return match next_char {
                        '/' => {
                            self.advance(); // Consomme le deuxième '/'
                            if self.peek_char() == Some('/') {
                                // C'est un commentaire de type `///`
                                self.advance(); // Consomme le troisième '/'
                                while let Some(ch) = self.next_char() {
                                    if ch == '\n' {
                                        break;
                                    }
                                    comment.push(ch);
                                }
                                TokenType::DOCSTRING(comment) // Retourne un DOCSTRING
                            } else {
                                // C'est un commentaire normal `//`
                                while let Some(ch) = self.next_char() {
                                    if ch == '\n' {
                                        break;
                                    }
                                    comment.push(ch);
                                }
                                TokenType::COMMENT(comment)
                            }
                        }
                        '*' => {
                            // Commentaire multi-lignes
                            self.advance(); // Consomme le '*'
                            let mut depth = 1;
                            while let Some(ch) = self.next_char() {
                                if ch == '*' && self.peek_char() == Some('/') {
                                    self.advance(); // Consomme le '/'
                                    depth -= 1;
                                    if depth == 0 {
                                        break;
                                    }
                                } else if ch == '/' && self.peek_char() == Some('*') {
                                    self.advance(); // Consomme le '*'
                                    depth += 1;
                                }
                                comment.push(ch);
                            }
                            if depth > 0 {
                                // Erreur : commentaire multi-lignes non terminé
                                self.create_error(LexerErrorType::UnterminatedComment)
                            } else {
                                TokenType::COMMENT(comment)
                            }
                        }
                        _ => {
                            // Ce n'est pas un commentaire, c'est probablement un opérateur de division
                            TokenType::OPERATOR(Operators::SLASH)
                        }
                    };
                }
            }
            _ => {
                // Ce cas ne devrait jamais se produire si la fonction est appelée correctement
                return TokenType::UNKNOWN;
            }
        }

        self.current_token_text = comment.clone();
        return TokenType::COMMENT(comment);
    }

    ////////////

    /// Methode pour avancer d'un character
    fn advance(&mut self) -> char {
        let ch = self.source.next().unwrap();
        if ch == '\n' {
            self.current_line += 1; // Incrémenter le numéro de ligne
            self.current_column = 1; // Réinitialiser le numéro de colonne
        } else {
            self.current_column += 1; // sinon incrementer le numero de colonne
        }
        ch
    }

    /// Methode pour sauter les espaces
    fn skip_whitespace(&mut self) {
        // amelioration pour qu'en  mode brace  le NEWLINE soit ignoré
        while let Some(&ch) = self.source.peek(){
            if ch.is_whitespace(){
                if ch == '\n' {
                    if self.syntax_mode == SyntaxMode::Braces{
                        self.advance();
                        self.at_line_start = true;
                    }else {
                        break;
                    }
                }else { self.advance();
                }
            }else {
                break;
            }

        }
        // while let Some(&ch) = self.source.peek() {
        //     if ch.is_whitespace() && ch != '\n' {
        //         self.advance();
        //     } else {
        //         break;
        //     }
        // }
    }

    /// C'est la deuxième methode principal avec get_token() pour obtenir les tokens
    /// Son role c'est de tokeniser le code source
    /// appel la methode get_token pour obtenir les tokens.
    /// Elle crée objet Token pour chaque TokenType retourné par get_token()
    /// elle retourne un vecteur de tokens Vec<Token>
    /// methode pour tokeniser le code source
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(token_type) = self.get_token() {
            let token = Token::new(
                self.current_token_text.clone(),
                token_type.clone(),
                self.current_line,
                self.current_column,
            );
            tokens.push(token);
            self.current_token_text.clear();
            if matches!(token_type, TokenType::EOF) {
                break;
            }
        }

        if self.syntax_mode == SyntaxMode::Indentation {
            while self.indent_level.len() > 1 {
                self.indent_level.pop();
                tokens.push(Token::new(
                    String::new(),
                    TokenType::DEDENT,
                    self.current_line,
                    self.current_column,
                ));
            }
        }

        return tokens;
    }

    /// methode pour les differents types de token de Type Unknown

    fn lex_unknown(&mut self) -> TokenType {
        let ch = self.advance();
        self.current_token_text = ch.to_string();
        TokenType::UNKNOWN
    }

    /// Methode pour creer un token de type erreur
    fn create_error(&self, error: LexerErrorType) -> TokenType {
        let position = Position {
            line: self.current_line,
            column: self.current_column,
        };
        TokenType::ERROR(LexerError::new(error.clone(), error.to_string(), position))
    }

    fn handle_newline(&mut self) -> TokenType {
        self.advance(); // Consomme le '\n'
        self.at_line_start = true;
        TokenType::NEWLINE
    }

    // fn is_operator_start(&self,ch:char) ->bool{
    //     match ch {
    //         '+' | '-' | '*' | '/' | '%' | '=' | '!' | '<' | '>' | '&' | '|' | '^' | '~' | '@' | ':' | '?' |'.'=> true,
    //         _ => false,
    //     }
    //
    // }
}

//by YmC



////////////////////////essai/////////////////////////////////////////////
pub fn lox(input: &str) -> Vec<Tok> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '+' => tokens.push(Tok::PLUS),
            '-' => tokens.push(Tok::MINUS),
            '*' => tokens.push(Tok::MUL),
            '/' => tokens.push(Tok::DIV),
            '0'..='9' => {
                let mut number = String::from(c);
                while let Some(&next_char) = chars.peek() {
                    if next_char.is_ascii_digit() {
                        number.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                tokens.push(Tok::NUMBER(number.parse().unwrap()));
            }

            _ => {}
        }
    }
    tokens
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum Tok {
    PLUS,
    MINUS,
    MUL,
    DIV,
    NUMBER(i64),
}
