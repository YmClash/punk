// src/parser/error_context.rs
// Module pour améliorer les messages d'erreur avec le contexte du mode syntaxique

use crate::parser::parser_error::{ParserError, ParserErrorType, Position};
use crate::SyntaxMode;
use crate::tok::TokenType;

/// Extension pour ParserError avec contexte de mode
pub struct ErrorContext {
    pub error_type: ParserErrorType,
    pub position: Position,
    pub syntax_mode: SyntaxMode,
    pub expected: Option<String>,
    pub found: Option<String>,
}

impl ErrorContext {
    /// Crée une nouvelle erreur avec contexte
    pub fn new(
        error_type: ParserErrorType,
        position: Position,
        syntax_mode: SyntaxMode,
    ) -> Self {
        ErrorContext {
            error_type,
            position,
            syntax_mode,
            expected: None,
            found: None,
        }
    }

    /// Ajoute ce qui était attendu
    pub fn with_expected(mut self, expected: String) -> Self {
        self.expected = Some(expected);
        self
    }

    /// Ajoute ce qui a été trouvé
    pub fn with_found(mut self, found: String) -> Self {
        self.found = Some(found);
        self
    }

    /// Convertit en ParserError avec message enrichi
    pub fn into_error(self) -> ParserError {
        let mut message = self.get_base_message();
        
        // Ajouter le contexte du mode
        message.push_str(&format!(" (in {} mode)", match self.syntax_mode {
            SyntaxMode::Braces => "Braces",
            SyntaxMode::Indentation => "Indentation",
        }));
        
        // Ajouter les détails expected/found
        if let Some(ref expected) = self.expected {
            message.push_str(&format!(". Expected: {}", expected));
        }
        
        if let Some(ref found) = self.found {
            message.push_str(&format!(", but found: {}", found));
        }
        
        // Ajouter des suggestions spécifiques au mode
        message.push_str(&self.get_mode_specific_hint());
        
        ParserError {
            error: self.error_type.clone(),
            message,
            position: self.position,
        }
    }

    /// Obtient le message de base pour le type d'erreur
    fn get_base_message(&self) -> String {
        match &self.error_type {
            ParserErrorType::UnexpectedToken => "Unexpected token".to_string(),
            ParserErrorType::UnexpectedEOF => "Unexpected end of file".to_string(),
            ParserErrorType::IndentationError => "Indentation error".to_string(),
            ParserErrorType::BraceError => "Brace mismatch error".to_string(),
            ParserErrorType::ExpectColon => "Expected ':' to start block".to_string(),
            ParserErrorType::ExpectedExpression => "Expected an expression".to_string(),
            _ => format!("{:?}", self.error_type),
        }
    }

    /// Obtient un indice spécifique au mode syntaxique
    fn get_mode_specific_hint(&self) -> String {
        match (&self.error_type, &self.syntax_mode) {
            (ParserErrorType::BraceError, SyntaxMode::Indentation) => {
                "\n  Hint: In Indentation mode, use ':' and proper indentation instead of braces".to_string()
            }
            (ParserErrorType::IndentationError, SyntaxMode::Braces) => {
                "\n  Hint: In Braces mode, use '{}' for blocks instead of indentation".to_string()
            }
            (ParserErrorType::ExpectColon, SyntaxMode::Braces) => {
                "\n  Hint: In Braces mode, use '{' to start a block, not ':'".to_string()
            }
            (ParserErrorType::ExpectedExpression, _) => {
                match self.syntax_mode {
                    SyntaxMode::Braces => "\n  Hint: Don't forget semicolons in Braces mode".to_string(),
                    SyntaxMode::Indentation => "\n  Hint: Check your indentation levels".to_string(),
                }
            }
            _ => String::new(),
        }
    }
}

/// Helper functions pour créer des erreurs contextualisées
impl crate::parser::parser::Parser {
    /// Crée une erreur avec le contexte du mode actuel
    pub fn error_with_context(
        &self,
        error_type: ParserErrorType,
    ) -> ParserError {
        ErrorContext::new(error_type, self.current_position(), self.syntax_mode)
            .into_error()
    }

    /// Crée une erreur "expected X but found Y" avec contexte
    pub fn error_expected_found(
        &self,
        expected: &str,
        found: Option<&TokenType>,
    ) -> ParserError {
        let mut ctx = ErrorContext::new(
            ParserErrorType::UnexpectedToken,
            self.current_position(),
            self.syntax_mode,
        );
        
        ctx = ctx.with_expected(expected.to_string());
        
        if let Some(token_type) = found {
            ctx = ctx.with_found(format!("{:?}", token_type));
        } else {
            ctx = ctx.with_found("end of file".to_string());
        }
        
        ctx.into_error()
    }

    /// Crée une erreur spécifique pour les blocs
    pub fn error_block_start(&self) -> ParserError {
        match self.syntax_mode {
            SyntaxMode::Braces => {
                self.error_expected_found("{", self.current_token().map(|t| &t.token_type))
            }
            SyntaxMode::Indentation => {
                self.error_expected_found(":", self.current_token().map(|t| &t.token_type))
            }
        }
    }

    /// Crée une erreur spécifique pour les séparateurs de statements
    pub fn error_statement_separator(&self) -> ParserError {
        match self.syntax_mode {
            SyntaxMode::Braces => {
                ErrorContext::new(
                    ParserErrorType::UnexpectedToken,
                    self.current_position(),
                    self.syntax_mode,
                )
                .with_expected("';' or '}'".to_string())
                .with_found(
                    self.current_token()
                        .map(|t| format!("{:?}", t.token_type))
                        .unwrap_or_else(|| "end of file".to_string())
                )
                .into_error()
            }
            SyntaxMode::Indentation => {
                ErrorContext::new(
                    ParserErrorType::IndentationError,
                    self.current_position(),
                    self.syntax_mode,
                )
                .with_expected("proper indentation or newline".to_string())
                .into_error()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_context_braces_mode() {
        let error = ErrorContext::new(
            ParserErrorType::BraceError,
            Position { index: 42 },
            SyntaxMode::Braces,
        )
        .with_expected("{".to_string())
        .with_found("if".to_string())
        .into_error();
        
        assert!(error.message.contains("Braces mode"));
        assert!(error.message.contains("Expected: {"));
        assert!(error.message.contains("found: if"));
    }
    
    #[test]
    fn test_error_context_indentation_mode() {
        let error = ErrorContext::new(
            ParserErrorType::IndentationError,
            Position { index: 10 },
            SyntaxMode::Indentation,
        )
        .into_error();
        
        assert!(error.message.contains("Indentation mode"));
        assert!(error.message.contains("Indentation error"));
    }
    
    #[test]
    fn test_mode_specific_hints() {
        let error = ErrorContext::new(
            ParserErrorType::BraceError,
            Position { index: 0 },
            SyntaxMode::Indentation,
        )
        .into_error();
        
        assert!(error.message.contains("use ':' and proper indentation"));
    }
}