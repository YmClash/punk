pub mod lexer;
//mod parser;
mod codegen;
pub mod parser;
pub mod semantic;
pub mod compiler;
mod utils;

//mod ast;
pub use crate::lexer::lex::SyntaxMode;
pub use lexer::lex::Lexer;
pub use lexer::lexer_error;
pub use lexer::tok;
