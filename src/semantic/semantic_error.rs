//src/semantic/semantic_error.rs


use std::fmt;
use std::fmt::Display;



#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub struct Position {
    pub index: usize,
    // pub line: usize,
    // pub column: usize,
}
#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub struct Span{
    pub start: Position,
    pub end: Position,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub struct SemanticErrorWithSpan {
    pub error: SemanticError,
    pub span: Span,
}

#[derive(Debug, PartialEq, Clone)]
pub struct  SemanticError {
    pub error : SemanticErrorType,
    pub message: String,
    pub position : Position,
    // Type errors
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolError {
    SymbolNotFound(String),
    SymbolAlreadyDeclared(String),
    InvalidVisibility(String),
    InvalidScope,
    ImportError(String),

}
#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    TypeMismatch(String),
    InvalidType(String),
    UndefinedType(String),
    TypeNotFound(String),
    InvalidTypeParameter(String),
}


#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum  SemanticErrorType{
    SymbolError(SymbolError),
    TypeError(TypeError),

    //todo!();
}

/// Implementation de Possition
impl Position {
    fn new() -> Self {
        Position { index: 0 }
    }
    fn advance(&mut self, ch:char) {
        self.index += ch.len_utf8();
    }
    fn move_left(&mut self){
        self.index -= 1;
    }
}

/// Implementation de l'affichage de Position
impl Display for Position{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Position index: {}", self.index)
    }
}

/// Implementation de l'affichage de type d'erreur semantique

impl Display for SemanticErrorType{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // SemanticErrorType::SymbolError(err) => write!(f, "Symbol Error: {:?}", err),
            // SemanticErrorType::TypeError(err) => write!(f, "Type Error: {:?}", err),
            SemanticErrorType::SymbolError(SymbolError::SymbolNotFound(name)) => {
                write!(f, "Symbol Error: Symbol '{}' not found", name)
            }
            SemanticErrorType::SymbolError(SymbolError::SymbolAlreadyDeclared(name)) => {
                write!(f, "Symbol Error: Symbol '{}' already declared", name)
            }
            SemanticErrorType::SymbolError(SymbolError::InvalidVisibility(name)) => {
                write!(f, "Symbol Error: Invalid visibility for symbol '{}'", name)
            }
            SemanticErrorType::SymbolError(SymbolError::InvalidScope) => {
                write!(f, "Symbol Error: Invalid scope")
            }
            SemanticErrorType::SymbolError(SymbolError::ImportError(name)) => {
                write!(f, "Symbol Error: Import error for symbol '{}'", name)
            }
            SemanticErrorType::TypeError(TypeError::TypeMismatch(name)) => {
                write!(f, "Type Error: Type mismatch for symbol '{}'", name)
            }
            SemanticErrorType::TypeError(TypeError::InvalidType(name)) => {
                write!(f, "Type Error: Invalid type for symbol '{}'", name)
            }
            SemanticErrorType::TypeError(TypeError::UndefinedType(name)) => {
                write!(f, "Type Error: Undefined type for symbol '{}'", name)
            }
            SemanticErrorType::TypeError(TypeError::TypeNotFound(name)) => {
                write!(f, "Type Error: Type not found for symbol '{}'", name)
            }
            SemanticErrorType::TypeError(TypeError::InvalidTypeParameter(name)) => {
                write!(f, "Type Error: Invalid type parameter for symbol '{}'", name)
            }

        }
    }
}


/// implementatique du message l'erreur semantique

impl SemanticError{
    pub fn new(error: SemanticErrorType, message:String, position: Position) -> Self{
        let message = match &error {
            // SemanticErrorType::SymbolError(err) => format!("Symbol Error: {:?} - {}", err, message),
            // SemanticErrorType::TypeError(err) => format!("Type Error: {:?} - {}", err, message),
            SemanticErrorType::SymbolError(SymbolError::SymbolNotFound(name)) => {
                format!("Symbol '{}' not found", name)
            }
            SemanticErrorType::SymbolError(SymbolError::SymbolAlreadyDeclared(name)) => {
                format!("Symbol '{}' already declared", name)
            }
            SemanticErrorType::SymbolError(SymbolError::InvalidVisibility(name)) => {
                format!("Invalid visibility for symbol '{}'", name)
            }
            SemanticErrorType::SymbolError(SymbolError::InvalidScope) => {
                format!("Invalid scope")
            }
            SemanticErrorType::SymbolError(SymbolError::ImportError(name)) => {
                format!("Import error for symbol '{}'", name)
            }
            SemanticErrorType::TypeError(TypeError::TypeMismatch(name)) => {
                format!("Type mismatch for symbol '{}'", name)
            }
            SemanticErrorType::TypeError(TypeError::InvalidType(name)) => {
                format!("Invalid type for symbol '{}'", name)
            }
            SemanticErrorType::TypeError(TypeError::UndefinedType(name)) => {
                format!("Undefined type for symbol '{}'", name)
            }
            SemanticErrorType::TypeError(TypeError::TypeNotFound(name)) => {
                format!("Type not found for symbol '{}'", name)
            }
            SemanticErrorType::TypeError(TypeError::InvalidTypeParameter(name)) => {
                format!("Invalid type parameter for symbol '{}'", name)
            }

        };


        SemanticError {
            error,
            message,
            position,
        }
    }
}


/// Test unitaire isole
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position() {
        let mut pos = Position::new();
        pos.advance('a');
        assert_eq!(pos.index, 1);
        pos.advance('b');
        assert_eq!(pos.index, 2);
        pos.move_left();
        assert_eq!(pos.index, 1);
    }
}





