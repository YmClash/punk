// src/semantic/diagnostics/error_formatter.rs

use super::diagnostic_engine::{Diagnostic, DiagnosticConfig, SourcePosition};
use super::diagnostic_level::DiagnosticLevel;

/// Formateur pour afficher les diagnostics de manière lisible
pub struct ErrorFormatter {
    use_colors: bool,
}

impl ErrorFormatter {
    pub fn new() -> Self {
        ErrorFormatter {
            use_colors: true,
        }
    }
    
    pub fn set_use_colors(&mut self, use_colors: bool) {
        self.use_colors = use_colors;
    }
    
    /// Formate et affiche un diagnostic
    pub fn format_diagnostic(&self, diagnostic: &Diagnostic, config: &DiagnosticConfig) {
        // Ligne d'en-tête avec niveau et code
        self.format_header(diagnostic, config);
        
        // Position et contexte du code
        if config.show_context {
            self.format_source_context(&diagnostic.position, diagnostic.level);
        }
        
        // Suggestions
        if config.show_suggestions && !diagnostic.suggestions.is_empty() {
            self.format_suggestions(&diagnostic.suggestions);
        }
        
        // Notes
        if !diagnostic.notes.is_empty() {
            self.format_notes(&diagnostic.notes);
        }
        
        // Diagnostics liés
        if !diagnostic.related.is_empty() {
            self.format_related(&diagnostic.related);
        }
        
        println!(); // Ligne vide après chaque diagnostic
    }
    
    /// Formate l'en-tête du diagnostic
    fn format_header(&self, diagnostic: &Diagnostic, config: &DiagnosticConfig) {
        let level_str = if self.use_colors {
            format!("{}", diagnostic.level)
        } else {
            diagnostic.level.prefix().to_string()
        };
        
        let code_str = if config.show_error_codes && !diagnostic.code.is_empty() {
            format!("[{}]", diagnostic.code)
        } else {
            String::new()
        };
        
        println!("{}{}: {}", level_str, code_str, diagnostic.message);
    }
    
    /// Formate le contexte du code source
    fn format_source_context(&self, position: &SourcePosition, level: DiagnosticLevel) {
        let file_pos = format!("  --> {}:{}:{}", 
                               position.file.display(), 
                               position.line, 
                               position.column);
        
        if self.use_colors {
            println!("\x1b[36m{}\x1b[0m", file_pos);
        } else {
            println!("{}", file_pos);
        }
        
        // Afficher les lignes de contexte avant
        let start_line = position.line.saturating_sub(position.context_before.len());
        for (i, line) in position.context_before.iter().enumerate() {
            self.format_context_line(start_line + i, line, false);
        }
        
        // Afficher la ligne principale avec soulignement
        if let Some(ref line_content) = position.line_content {
            self.format_main_line(position.line, line_content, position.column, position.length, level);
        }
        
        // Afficher les lignes de contexte après
        for (i, line) in position.context_after.iter().enumerate() {
            self.format_context_line(position.line + i + 1, line, false);
        }
    }
    
    /// Formate une ligne de contexte
    fn format_context_line(&self, line_num: usize, content: &str, _highlight: bool) {
        let line_num_str = format!("{:4} |", line_num);
        
        if self.use_colors {
            println!("\x1b[90m{}\x1b[0m {}", line_num_str, content);
        } else {
            println!("{} {}", line_num_str, content);
        }
    }
    
    /// Formate la ligne principale avec soulignement
    fn format_main_line(&self, line_num: usize, content: &str, column: usize, length: usize, level: DiagnosticLevel) {
        let line_num_str = format!("{:4} |", line_num);
        
        // Afficher la ligne
        if self.use_colors {
            println!("\x1b[90m{}\x1b[0m {}", line_num_str, content);
        } else {
            println!("{} {}", line_num_str, content);
        }
        
        // Créer le soulignement
        let mut underline = String::new();
        underline.push_str("     | ");
        for _ in 1..column {
            underline.push(' ');
        }
        
        let underline_char = match level {
            DiagnosticLevel::Error => '^',
            DiagnosticLevel::Warning => '~',
            _ => '-',
        };
        
        for _ in 0..length.max(1) {
            underline.push(underline_char);
        }
        
        // Afficher le soulignement
        if self.use_colors {
            println!("{}{}\x1b[0m", level.color_code(), underline);
        } else {
            println!("{}", underline);
        }
    }
    
    /// Formate les suggestions
    fn format_suggestions(&self, suggestions: &[super::suggestion_engine::Suggestion]) {
        println!("     |");
        for suggestion in suggestions {
            let prefix = if self.use_colors {
                "\x1b[32m  help\x1b[0m"
            } else {
                "  help"
            };
            
            println!("{}: {}", prefix, suggestion.message);
            
            if let Some(ref replacement) = suggestion.replacement {
                println!("       {}", replacement);
            }
            
            if let Some(ref example) = suggestion.example {
                println!("       Example: {}", example);
            }
        }
    }
    
    /// Formate les notes
    fn format_notes(&self, notes: &[String]) {
        for note in notes {
            let prefix = if self.use_colors {
                "\x1b[90m  note\x1b[0m"
            } else {
                "  note"
            };
            
            println!("{}: {}", prefix, note);
        }
    }
    
    /// Formate les diagnostics liés
    fn format_related(&self, related: &[super::diagnostic_engine::RelatedDiagnostic]) {
        for rel in related {
            let prefix = if self.use_colors {
                format!("  {}", rel.level)
            } else {
                format!("  {}", rel.level.prefix())
            };
            
            println!("{}: {}", prefix, rel.message);
            println!("    --> {}:{}:{}", 
                    rel.position.file.display(),
                    rel.position.line,
                    rel.position.column);
        }
    }
}