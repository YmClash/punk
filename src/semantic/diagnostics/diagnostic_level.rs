// src/semantic/diagnostics/diagnostic_level.rs

use std::fmt;

/// Niveau de sévérité d'un diagnostic
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticLevel {
    /// Erreur fatale qui empêche la compilation
    Error,
    /// Avertissement qui n'empêche pas la compilation
    Warning,
    /// Information utile pour le développeur
    Info,
    /// Conseil pour améliorer le code
    Hint,
    /// Note supplémentaire liée à un autre diagnostic
    Note,
}

impl DiagnosticLevel {
    /// Retourne le symbole associé au niveau
    pub fn symbol(&self) -> &str {
        match self {
            DiagnosticLevel::Error => "❌",
            DiagnosticLevel::Warning => "⚠️",
            DiagnosticLevel::Info => "ℹ️",
            DiagnosticLevel::Hint => "💡",
            DiagnosticLevel::Note => "📝",
        }
    }
    
    /// Retourne le code couleur ANSI
    pub fn color_code(&self) -> &str {
        match self {
            DiagnosticLevel::Error => "\x1b[31m",     // Rouge
            DiagnosticLevel::Warning => "\x1b[33m",   // Jaune
            DiagnosticLevel::Info => "\x1b[36m",      // Cyan
            DiagnosticLevel::Hint => "\x1b[32m",      // Vert
            DiagnosticLevel::Note => "\x1b[90m",      // Gris
        }
    }
    
    /// Retourne le préfixe textuel
    pub fn prefix(&self) -> &str {
        match self {
            DiagnosticLevel::Error => "error",
            DiagnosticLevel::Warning => "warning",
            DiagnosticLevel::Info => "info",
            DiagnosticLevel::Hint => "hint",
            DiagnosticLevel::Note => "note",
        }
    }
}

impl fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}\x1b[0m", self.color_code(), self.prefix())
    }
}