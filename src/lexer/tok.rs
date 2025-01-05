use crate::lexer_error::LexerError;
use num_bigint::BigInt;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    IDENTIFIER { name: String },
    INTEGER { value: BigInt },
    FLOAT { value: f64 },
    HEXADECIMAL { value: u64 },
    STRING { value: String, kind: StringKind },
    CHAR { value: char },
    EOF,
    NEWLINE,
    OPERATOR(Operators),
    KEYWORD(Keywords),
    DELIMITER(Delimiters),
    UNKNOWN,
    COMMENT(String),
    DOCSTRING(String),
    ERROR(LexerError),
    INDENT,
    DEDENT,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum Operators {
    PLUS,            // '+' PLUS / PLUS SIGN
    MINUS,           // '-' MOINS / MINUS SIGN
    STAR,            // '*' ETOILE / STAR
    SLASH,           // '/' SLASH / SLASH
    VBAR,            // '|' BARRE VERTICALE / VERTICAL BAR
    AMPER,           // '&' ET COMMERCIAL / AMPERSAND
    LESS,            // '<' INFERIEUR / LESS-THAN SIGN
    GREATER,         // '>' SUPERIEUR / GREATER-THAN SIGN
    EQUAL,           // '=' EGAL / EQUALS SIGN
    PERCENT,         // '%' POURCENTAGE / PERCENT
    EQEQUAL,         // '==' EGAL EGAL / EQUALS EQUALS
    NOTEQUAL,        // '!=' DIFFERENT / NOT EQUAL
    LESSEQUAL,       // '<=' INFERIEUR EGAL / LESS-THAN EQUAL
    FATARROW,        // '=>' IMPLIQUE / IMPLIES
    GREATEREQUAL,    // '>=' SUPERIEUR EGAL / GREATER-THAN EQUAL
    TILDE,           // '~' TILDE / TILDE
    CIRCUMFLEX,      // '^' CIRCONFLEXE / CIRCUMFLEX
    LEFTSHIFT,       // '<<' DECALAGE GAUCHE / LEFT SHIFT
    RIGHTSHIFT,      // '>>' DECALAGE DROITE / RIGHT SHIFT
    DOUBLESTAR,      // '**' DOUBLE ÉTOILE / DOUBLE STAR
    PLUSEQUAL,       // '+=' PLUS EGAL / PLUS EQUAL
    MINEQUAL,        // '-=' MOINS EGAL / MINUS EQUAL
    STAREQUAL,       // '*=' ETOILE EGAL / STAR EQUAL
    SLASHEQUAL,      // '/=' SLASH EGAL / SLASH EQUAL
    PERCENTEQUAL,    // '%=' POURCENTAGE EGAL / PERCENT EQUAL
    AMPEREQUAL,      // '&=' ET COMMERCIAL EGAL / AMPERSAND EQUAL
    VBAREQUAL,       // '|=' BARRE VERTICALE EGAL / VERTICAL BAR EQUAL
    CIRCUMFLEXEQUAL, // '^=' CIRCONFLEXE EGAL / CIRCUMFLEX EQUAL
    LEFTSHIFTEQUAL,  // '<<=' DECALAGE GAUCHE EGAL / LEFT SHIFT EQUAL
    RIGHTSHIFTEQUAL, // '>>=' DECALAGE DROITE EGAL / RIGHT SHIFT EQUAL
    DOUBLESTAREQUAL, // '**=' DOUBLE ETOILE EGAL / DOUBLE STAR EQUAL
    AND,             // '&&' ET ET / AND
    OR,              // '||' OU OU / OR

    //DOUBLESLASH, // '//' DOUBLE SLASH / DOUBLE SLASH
    DOUBLESLASHEQUAL, // '//=' DOUBLE SLASH EGAL / DOUBLE SLASH EQUAL
    AT,               // '@' AROBASE / AT
    ATEQUAL,          // '@=' AROBASE EGAL / AT EQUAL
    RARROW,           // '->' FLECHE DROITE / RIGHT ARROW
    //   ELLIPSIS, // '...' POINTS DE SUSPENSION / ELLIPSIS
    COLONEQUAL,    // ':=' DEUX POINT EGAL / COLON EQUAL
    STARSLASH,     // '*/' ETOILE SLASH / STAR SLASH
    SLASHSTAR,     // '/*' SLASH ETOILE / SLASH STAR
    DIESE,         // '#' DIESE / HASH
    EXCLAMATION,   // '!' POINT D'EXCLAMATION / EXCLAMATION POINT
    INTERROGATION, // '?'
    UNDERSCORE,    // '_'

    DOTDOT,      // '..' DEUX POINTS / DOUBLE DOT  // Range Operator
    DOTDOTEQUAL, // '..=' DEUX POINTS EGAL / DOUBLE DOT EQUAL RangeInclusive Operator
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum Keywords {
    AND,
    AS,
    ASYNC,
    AWAIT,
    BREAK,
    CONST,
    CLASS,
    CONTINUE,
    DEF,
    DEL,
    ELIF,
    ELSE,
    ENUM,
    EXCEPT,
    FALSE,
    FN,
    FOR,
    FROM,
    IF,
    IMPL,
    IMPORT,
    IN,
    IS,
    LAMBDA,
    LET,
    LOOP,
    MATCH,
    MOD,
    MUT,
    NONE,
    NOT,
    OR,
    PUB,
    PASS,
    RAISE,
    RETURN,
    SELF,
    STATIC,
    STRUCT,
    SUPER,
    TRAIT,
    TRUE,
    TRY,
    TYPE,
    TYPEOF,
    USE,
    WITH,
    WHERE,
    WHILE,
    YIELD,
    //TYPE KEYWORDS
    INT,
    FLOAT,
    STR,
    BOOL,
    CHAR,

}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum Delimiters {
    //DELIMITERS,
    LPAR,        // '(' PARANTHESE GAUCHE / LEFT PARENTHESIS
    RPAR,        // ')' PARANTHESE DROITE / RIGHT PARENTHESIS
    LSBRACKET,   // '[' CROCHET GAUCHE / LEFT SQUARE BRACKET
    RSBRACKET,   // ']' CROCHET DROIT / RIGHT SQUARE BRACKET
    COLON,       // ':' DEUX POINT / COLON
    COMMA,       // ',' VIRGULE / COMMA
    SEMICOLON,   // ';' POINT VIRGULE / SEMICOLON
    DOT,         // '.' POINT / DOT
    LCURBRACE,   // '{' ACCOLADE GAUCHE / LEFT CURLY BRACKET
    RCURBRACE,   // '}' ACCOLADE DROITE / RIGHT CURLY BRACKET
    ELLIPSIS,    // '...' POINTS DE SUSPENSION / ELLIPSIS
    DOUBLECOLON, // '::' DEUX POINTS / DOUBLE COLON
    //
    // DOTDOTEQUAL, // '..=' DEUX POINTS EGAL / DOUBLE DOT EQUAL
    // DOTDOT,      // '..' DEUX POINTS / DOUBLE DOT
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum StringKind {
    NORMAL,
    FORMATTED, // f-string implementation plus tard
    UNICODE,   // u-string implementation plus tard
}

//by YmC

/*

*/
