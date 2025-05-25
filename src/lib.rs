pub mod lexer;
//mod parser;
mod codegen;
pub mod parser;
pub mod semantic;
mod utils;

//mod ast;
pub use crate::lexer::lex::SyntaxMode;
#[allow(dead_code)]
pub use lexer::lex::Lexer;
pub use lexer::lexer_error;
pub use lexer::tok;

//pub use parser::parser;
//pub use parser::ast;
//pub use parser::error;

//pub use parser::*;

//pub use ast::*;
