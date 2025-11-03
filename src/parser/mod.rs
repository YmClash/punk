pub mod ast;

pub mod parser_error;

pub mod parser;
mod inference;
mod synchronizer;
mod statements;
// L'ancien fichier monolithique declarations.rs
// mod declarations;
// Nouveau module modulaire
pub mod declarations;
mod expressions;
mod parameters;
mod unified_block;
mod error_context;
mod utils;
mod token_matcher;
