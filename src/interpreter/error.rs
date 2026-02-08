// src/interpreter/error.rs

use std::fmt;
use std::fmt::Display;
use crate::parser::parser_error::ParserError;

#[derive(Debug, Clone)]
pub enum RuntimeError {
    TypeError(String),
    UndefinedVariable(String),
    DivisionByZero,
    IndexOutOfBounds,
    ArityMismatch { expected: usize, got: usize },
    NotCallable(String),
    InvalidAssignment,
    IOError(String),
    ConversionError(String),
    EmptyArray,
    Unimplemented(String),
    SyntaxError(ParserError),
    UnsupportedOperator(String),

}


impl Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RuntimeError::TypeError(msg) => write!(f, "Type Error: {}", msg),
            RuntimeError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            RuntimeError::DivisionByZero => write!(f, "Division by zero"),
            RuntimeError::IndexOutOfBounds => write!(f, "Index out of bounds"),
            RuntimeError::ArityMismatch { expected, got } => {
                write!(f, "Expected {} arguments, got {}", expected, got)
            },
            RuntimeError::NotCallable(typ) => write!(f, "{} is not callable", typ),
            RuntimeError::InvalidAssignment => write!(f, "Invalid assignment target"),
            RuntimeError::IOError(msg) => write!(f, "IO Error: {}", msg),
            RuntimeError::ConversionError(msg) => write!(f, "Conversion Error: {}", msg),
            RuntimeError::EmptyArray => write!(f, "Cannot pop from empty array"),
            RuntimeError::Unimplemented(feature) => write!(f, "Unimplemented: {}", feature),
            RuntimeError::SyntaxError(err) => write!(f, "Syntax Error: {}", err),
            RuntimeError::UnsupportedOperator(op) => write!(f, "Unsupported operator: {}", op),
        }
    }
}