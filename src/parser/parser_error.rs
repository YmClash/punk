
use std::fmt;
#[allow(dead_code)]
use std::fmt::{Display, Formatter};

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub struct Position {
    pub index: usize,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub struct ParserError {
    pub error: ParserErrorType,
    pub message: String,
    pub position: Position,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum ParserErrorType {
    UnexpectedToken, //{ expected: TokenType, found: TokenType },
    UnexpectedEOF,
    IndentationError,
    BraceError,
    InvalidAssignmentTarget,
    ExpectedExpression,
    InvalidVariableDeclaration,
    InvalidFunctionDeclaration,
    InvalidTypeAnnotation,
    ExpectVariableName,
    ExpectOperatorEqual,
    ExpectValue,
    ExpectColon,

    ExpectedTypeAnnotation,
    ExpectParameterName,
    ExpectFunctionName,
    ExpectIdentifier,

    ExpectedType,
    ExpectedDeclaration,

    ExpectedOpenParenthesis,
    ExpectedCloseParenthesis,
    ExpectedCommaOrCloseBrace,
    ExpectedStructField,
    ExpectedArrowOrBlock,
    ExpectedCommaOrClosingParenthesis,


    UnexpectedIndentation,
    UnexpectedEndOfInput,

    ExpectedUseOrImport,
    ExpectedAlias,
    ExpectedRangeOperator,
    ExpectedLifetime,



    ExpectedParameterName,
    ExpectedSelfParameter,
    MultipleConstructors,
    UnexpectedParameterName,
    MismatchedParametersAndAttributes,
    MultipleRestPatterns,

    InvalidConstructorReturn,
    InvalidConstructorParameter,
    InvalidConstructorName,
    InvalidSelfParameter,


    MissingType,
    MissingParameter,

    //TypeInferenceNotSupported,/
    TypeInferenceError,






}

// #[allow(dead_code)]
// #[derive(Debug,PartialEq,Clone)]
// struct TokExpected{
//     expected: TokenType,
//     found: TokenType,
// }

// implement de la position
#[allow(dead_code)]
impl Position {
    fn new() -> Self {
        Position { index: 0 }
    }
    fn advance(&mut self, ch: char) {
        self.index += ch.len_utf8();
    }
    fn move_left(&mut self) {
        self.index -= 1;
    }
}

//implement de l'affichage de la position

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Position index {}", self.index)
    }
}

//implement de l'affichage de l'erreur

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ParserError: {} at {}", self.message, self.position)
    }
}

//implement de l'affichage du type d'erreur du parseur

impl Display for ParserErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParserErrorType::UnexpectedToken => write!(f, "UnexpectedToken"),
            // ParserErrorType::UnexpectedToken{expected,found} => write!(f, "Expected Toke {:?}, but Found: {:?}", expected, found),
            ParserErrorType::UnexpectedEOF => write!(f, "UnexpectedEOF"),
            ParserErrorType::IndentationError => write!(f, "IndentationError"),
            ParserErrorType::BraceError => write!(f, "BraceError"),
            ParserErrorType::InvalidAssignmentTarget => write!(f, "InvalidAssignmentTarget"),
            ParserErrorType::ExpectedExpression => write!(f, "ExpectedExpression"),
            ParserErrorType::InvalidVariableDeclaration => write!(f, "InvalidVariableDeclaration"),
            ParserErrorType::InvalidTypeAnnotation => write!(f, "InvalidTypeAnnotation"),
            ParserErrorType::ExpectVariableName => write!(f, "ExpectVariableName"),
            ParserErrorType::ExpectOperatorEqual => write!(f, "ExpectOperatorEqual"),
            ParserErrorType::ExpectValue => write!(f, "ExpectValue"),
            ParserErrorType::ExpectColon => write!(f, "ExpectColon"),

            ParserErrorType::UnexpectedParameterName => write!(f, "UnexpectedParameterName"),

            ParserErrorType::ExpectedTypeAnnotation => write!(f, "ExpectedTypeAnnotation"),
            ParserErrorType::ExpectIdentifier => write!(f, "ExpectIdentifier"),
            ParserErrorType::ExpectedType => write!(f, "ExpectedType"),


            ParserErrorType::UnexpectedIndentation => write!(f, "UnexpectedIndentation"),
            ParserErrorType::ExpectedParameterName => write!(f, "ExpectedParameterName"),
            ParserErrorType::ExpectedArrowOrBlock => write!(f, "ExpectedArrowOrBlock"),


            ParserErrorType::ExpectedSelfParameter => write!(f, "ExpectedSelfParameter"),

            ParserErrorType::ExpectedDeclaration => write!(f, "ExpectedDeclaration"),

            ParserErrorType::ExpectedOpenParenthesis => write!(f, "ExpectedOpenParenthesis"),
            ParserErrorType::ExpectedCloseParenthesis => write!(f, "ExpectedCloseParenthesis"),
            ParserErrorType::UnexpectedEndOfInput => write!(f, "UnexpectedEndOfInput"),
            ParserErrorType::ExpectParameterName => write!(f, "ExpectParameterName"),
            ParserErrorType::ExpectFunctionName => write!(f, "ExpectFunctionName"),
            ParserErrorType::ExpectedCommaOrCloseBrace => write!(f, "ExpectedCommaOrCloseBrace"),
            ParserErrorType::ExpectedStructField => write!(f, "ExpectedStructField"),

            ParserErrorType::ExpectedCommaOrClosingParenthesis => write!(f, "ExpectedCommaOrClosingParenthesis"),

            ParserErrorType::ExpectedUseOrImport => write!(f, "ExpectedUseOrImport"),
            ParserErrorType::ExpectedAlias => write!(f, "ExpectedAlias"),
            ParserErrorType::ExpectedRangeOperator => write!(f, "ExpectedRangeOperator"),
            ParserErrorType::ExpectedLifetime => write!(f, "ExpectedLifetime"),


            ParserErrorType::MultipleConstructors => write!(f, "MultipleConstructors"),
            ParserErrorType::MismatchedParametersAndAttributes => write!(f, "MismatchedParametersAndAttributes"),
            ParserErrorType::MultipleRestPatterns => write!(f, "MultipleRestPatterns"),


            ParserErrorType::InvalidFunctionDeclaration => write!(f, "InvalidFunctionDeclaration"),

            ParserErrorType::InvalidConstructorReturn => write!(f, "InvalidConstructorReturn"),
            ParserErrorType::InvalidConstructorParameter => write!(f, "InvalidConstructorParameter"),
            ParserErrorType::InvalidConstructorName => write!(f, "InvalidConstructorName"),
            ParserErrorType::InvalidSelfParameter => write!(f, "InvalidSelfParameter"),

            ParserErrorType::MissingType => write!(f, "MissingType"),
            ParserErrorType::MissingParameter => write!(f, "MissingParameter"),


            ParserErrorType::TypeInferenceError => write!(f, "TypeInferenceError"),




        }
    }
}

//implementation du message d'erreur du parseur
impl ParserError {
    pub fn new(error: ParserErrorType, position: Position) -> Self {
        let message = match &error {
            ParserErrorType::UnexpectedToken => "Unexpected token".to_string(),
            // ParserErrorType::UnexpectedToken { expected, found } =>
            //     format!("Expected {:?}, but found {:?}", expected, found),
            ParserErrorType::UnexpectedEOF => "Unexpected end of file".to_string(),
            ParserErrorType::IndentationError => "Indentation error".to_string(),
            ParserErrorType::BraceError => "Mismatched braces".to_string(),
            ParserErrorType::InvalidAssignmentTarget => "Invalid assignment target".to_string(),
            ParserErrorType::ExpectedExpression => "Expected expression".to_string(),
            //ParserErrorType::InvalidExpression => "Invalid expression".to_string(),
            ParserErrorType::InvalidVariableDeclaration => {
                "Invalid variable declaration".to_string()
            }
            ParserErrorType::InvalidTypeAnnotation => "Invalid type annotation".to_string(),
            ParserErrorType::ExpectVariableName => "Expect variable name".to_string(),
            ParserErrorType::ExpectOperatorEqual => "Expect operator equal".to_string(),
            ParserErrorType::ExpectValue => "Expect value".to_string(),
            ParserErrorType::ExpectColon => "Expect colon".to_string(),
            ParserErrorType::ExpectedTypeAnnotation => "Expected type annotation".to_string(),
            ParserErrorType::ExpectedType => "Expected type".to_string(),

            ParserErrorType::UnexpectedIndentation => "Unexpected indentation".to_string(),
            ParserErrorType::ExpectedOpenParenthesis => "Expected open parenthesis".to_string(),
            ParserErrorType::ExpectedCloseParenthesis => "Expected close parenthesis".to_string(),
            ParserErrorType::InvalidFunctionDeclaration => {
                "Invalid function declaration".to_string()
            }
            ParserErrorType::ExpectParameterName => "Expect parameter name".to_string(),
            ParserErrorType::ExpectFunctionName => "Expect function name".to_string(),
            ParserErrorType::ExpectIdentifier => "Expect identifier".to_string(),
            ParserErrorType::ExpectedCommaOrCloseBrace => "Expected comma or close brace".to_string(),
            ParserErrorType::ExpectedStructField => "Expected struct field".to_string(),

            ParserErrorType::ExpectedCommaOrClosingParenthesis => "Expected comma or closing parenthesis".to_string(),

            ParserErrorType::ExpectedUseOrImport => "Expected use or import".to_string(),
            ParserErrorType::ExpectedAlias => "Expected alias after 'as '".to_string(),

            ParserErrorType::ExpectedRangeOperator => "Expected range operator".to_string(),
            ParserErrorType::ExpectedLifetime => "Expected lifetime".to_string(),


            ParserErrorType::ExpectedDeclaration => "Expected declaration".to_string(),
            ParserErrorType::ExpectedParameterName => "Expected parameter name".to_string(),

            ParserErrorType::ExpectedSelfParameter => "Expected self parameter".to_string(),

            ParserErrorType::ExpectedArrowOrBlock => "Expected arrow or block".to_string(),

            ParserErrorType::MultipleConstructors => "Multiple constructors".to_string(),
            ParserErrorType::MismatchedParametersAndAttributes => "Mismatched parameters and attributes".to_string(),
            ParserErrorType::MultipleRestPatterns => "Multiple rest patterns".to_string(),


            ParserErrorType::InvalidConstructorReturn => "Invalid constructor return".to_string(),
            ParserErrorType::InvalidConstructorParameter => "Invalid constructor parameter".to_string(),
            ParserErrorType::InvalidConstructorName => "Invalid constructor name".to_string(),
            ParserErrorType::InvalidSelfParameter => "Invalid self parameter".to_string(),



            ParserErrorType::UnexpectedParameterName => "Unexpected parameter name".to_string(),
            ParserErrorType::UnexpectedEndOfInput => "Unexpected end of input".to_string(),


            ParserErrorType::MissingType => "Missing type".to_string(),
            ParserErrorType::MissingParameter => "Missing parameter".to_string(),

            ParserErrorType::TypeInferenceError => "Type inference error".to_string(),


        };

        ParserError {
            error,
            message,
            position,
        }
    }
}