pub mod lexer;
//mod parser;
mod codegen;
pub mod parser;
pub mod semantic;
pub mod compiler;
pub mod interpreter;

pub mod repl;
pub mod utils;

//mod ast;
pub use crate::lexer::lex::SyntaxMode;
pub use lexer::lex::Lexer;
pub use lexer::lexer_error;
pub use lexer::tok;
