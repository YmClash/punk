// src/semantic/diagnostics/diagnostic_engine.rs

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::semantic::semantic_error::{SemanticError, Position};
use crate::semantic::context::CompilationContext;
use super::diagnostic_level::DiagnosticLevel;
use super::suggestion_engine::{Suggestion, SuggestionEngine};
use super::error_formatter::ErrorFormatter;

/// Représente un diagnostic complet avec toutes les informations
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Niveau de sévérité
    pub level: DiagnosticLevel,
    /// Code d'erreur unique (ex: E0001)
    pub code: String,
    /// Message principal
    pub message: String,
    /// Position dans le code source
    pub position: SourcePosition,
    /// Suggestions de correction
    pub suggestions: Vec<Suggestion>,
    /// Notes supplémentaires
    pub notes: Vec<String>,
    /// Diagnostics liés
    pub related: Vec<RelatedDiagnostic>,
    /// Tags pour catégorisation
    pub tags: HashSet<String>,
}

/// Position dans le code source avec contexte
#[derive(Debug, Clone)]
pub struct SourcePosition {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub length: usize,
    /// Ligne de code complète
    pub line_content: Option<String>,
    /// Lignes de contexte avant
    pub context_before: Vec<String>,
    /// Lignes de contexte après
    pub context_after: Vec<String>,
}

/// Diagnostic lié à un diagnostic principal
#[derive(Debug, Clone)]
pub struct RelatedDiagnostic {
    pub position: SourcePosition,
    pub message: String,
    pub level: DiagnosticLevel,
}

/// Moteur de diagnostics principal
pub struct DiagnosticEngine {
    context: Rc<RefCell<CompilationContext>>,
    /// Tous les diagnostics collectés
    diagnostics: Vec<Diagnostic>,
    /// Moteur de suggestions
    suggestion_engine: SuggestionEngine,
    /// Formateur d'erreurs
    formatter: ErrorFormatter,
    /// Compteurs par niveau
    counts: HashMap<DiagnosticLevel, usize>,
    /// Configuration
    config: DiagnosticConfig,
    /// Cache du code source
    source_cache: HashMap<PathBuf, Vec<String>>,
}

/// Configuration du moteur de diagnostics
#[derive(Debug, Clone)]
pub struct DiagnosticConfig {
    /// Nombre maximum d'erreurs avant arrêt
    pub max_errors: usize,
    /// Afficher les suggestions
    pub show_suggestions: bool,
    /// Afficher le contexte du code
    pub show_context: bool,
    /// Nombre de lignes de contexte
    pub context_lines: usize,
    /// Utiliser les couleurs ANSI
    pub use_colors: bool,
    /// Niveau minimum à afficher
    pub min_level: DiagnosticLevel,
    /// Afficher les codes d'erreur
    pub show_error_codes: bool,
}

impl Default for DiagnosticConfig {
    fn default() -> Self {
        DiagnosticConfig {
            max_errors: 100,
            show_suggestions: true,
            show_context: true,
            context_lines: 2,
            use_colors: true,
            min_level: DiagnosticLevel::Info,
            show_error_codes: true,
        }
    }
}

impl DiagnosticEngine {
    pub fn new(context: Rc<RefCell<CompilationContext>>) -> Self {
        DiagnosticEngine {
            context: context.clone(),
            diagnostics: Vec::new(),
            suggestion_engine: SuggestionEngine::new(context.clone()),
            formatter: ErrorFormatter::new(),
            counts: HashMap::new(),
            config: DiagnosticConfig::default(),
            source_cache: HashMap::new(),
        }
    }
    
    /// Configure le moteur
    pub fn with_config(mut self, config: DiagnosticConfig) -> Self {
        let use_colors = config.use_colors;
        self.config = config;
        self.formatter.set_use_colors(use_colors);
        self
    }
    
    /// Ajoute un diagnostic depuis une erreur sémantique
    pub fn add_from_error(&mut self, error: &SemanticError) {
        let diagnostic = self.create_diagnostic_from_error(error);
        self.add_diagnostic(diagnostic);
    }
    
    /// Crée un diagnostic depuis une erreur sémantique
    fn create_diagnostic_from_error(&mut self, error: &SemanticError) -> Diagnostic {
        let code = self.get_error_code(error);
        let position = self.get_source_position(&error.position);
        let suggestions = self.suggestion_engine.get_suggestions(error);
        let notes = self.get_error_notes(error);
        let related = self.get_related_diagnostics(error);
        
        Diagnostic {
            level: DiagnosticLevel::Error,
            code,
            message: error.message.clone(),
            position,
            suggestions,
            notes,
            related,
            tags: self.get_error_tags(error),
        }
    }
    
    /// Ajoute un avertissement
    pub fn add_warning(&mut self, message: String, position: Position) {
        let diagnostic = Diagnostic {
            level: DiagnosticLevel::Warning,
            code: self.generate_warning_code(),
            message,
            position: self.get_source_position(&position),
            suggestions: Vec::new(),
            notes: Vec::new(),
            related: Vec::new(),
            tags: HashSet::new(),
        };
        self.add_diagnostic(diagnostic);
    }
    
    /// Ajoute une information
    pub fn add_info(&mut self, message: String, position: Position) {
        let diagnostic = Diagnostic {
            level: DiagnosticLevel::Info,
            code: String::new(),
            message,
            position: self.get_source_position(&position),
            suggestions: Vec::new(),
            notes: Vec::new(),
            related: Vec::new(),
            tags: HashSet::new(),
        };
        self.add_diagnostic(diagnostic);
    }
    
    /// Ajoute un diagnostic
    pub fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        // Incrémenter le compteur
        *self.counts.entry(diagnostic.level).or_insert(0) += 1;
        
        // Vérifier la limite d'erreurs
        if diagnostic.level == DiagnosticLevel::Error && 
           self.counts.get(&DiagnosticLevel::Error).unwrap_or(&0) > &self.config.max_errors {
            return;
        }
        
        // Filtrer par niveau minimum
        if diagnostic.level < self.config.min_level {
            return;
        }
        
        self.diagnostics.push(diagnostic);
    }
    
    /// Obtient la position source avec contexte
    fn get_source_position(&mut self, position: &Position) -> SourcePosition {
        let file = PathBuf::from("current_file.pk"); // À améliorer avec le vrai nom de fichier
        
        // Charger le fichier en cache si nécessaire
        if !self.source_cache.contains_key(&file) {
            // Pour l'instant, utiliser des lignes factices
            let lines = vec![
                "let x = 10".to_string(),
                "let y = 20".to_string(),
                "let z = x + y".to_string(),
            ];
            self.source_cache.insert(file.clone(), lines);
        }
        
        let lines = self.source_cache.get(&file).unwrap();
        let line_idx = (position.index / 100).min(lines.len().saturating_sub(1)); // Protection contre dépassement
        let column = position.index % 100;
        
        let line_content = lines.get(line_idx).cloned();
        
        let context_before = if line_idx > 0 && self.config.context_lines > 0 {
            let start = line_idx.saturating_sub(self.config.context_lines);
            lines[start..line_idx].to_vec()
        } else {
            Vec::new()
        };
        
        let context_after = if line_idx < lines.len() - 1 && self.config.context_lines > 0 {
            let end = (line_idx + 1 + self.config.context_lines).min(lines.len());
            lines[line_idx + 1..end].to_vec()
        } else {
            Vec::new()
        };
        
        SourcePosition {
            file,
            line: line_idx + 1,
            column: column + 1,
            length: 1,
            line_content,
            context_before,
            context_after,
        }
    }
    
    /// Génère un code d'erreur
    fn get_error_code(&self, error: &SemanticError) -> String {
        use crate::semantic::semantic_error::SemanticErrorType::*;
        
        let code = match &error.error {
            TypeError(_) => "E0001",
            SymbolError(_) => "E0002",
            // _ => "E0000",
        };
        
        code.to_string()
    }
    
    /// Génère un code d'avertissement
    fn generate_warning_code(&self) -> String {
        let count = self.counts.get(&DiagnosticLevel::Warning).unwrap_or(&0);
        format!("W{:04}", count + 1)
    }
    
    /// Obtient les notes pour une erreur
    fn get_error_notes(&self, error: &SemanticError) -> Vec<String> {
        let mut notes = Vec::new();
        
        use crate::semantic::semantic_error::SemanticErrorType::*;
        match &error.error {
            TypeError(type_error) => {
                use crate::semantic::semantic_error::TypeError::*;
                match type_error {
                    TypeMismatch(msg) => {
                        notes.push(format!("Type mismatch: {}", msg));
                    }
                    _ => {}
                }
            }
            SymbolError(symbol_error) => {
                use crate::semantic::semantic_error::SymbolError::*;
                match symbol_error {
                    UnusedSymbol(name) => {
                        notes.push(format!("Symbol '{}' is declared but not used", name));
                        notes.push("Consider removing it or prefixing with underscore".to_string());
                    }
                    _ => {}
                }
            }
            // _ => {}
        }
        
        notes
    }
    
    /// Obtient les diagnostics liés
    fn get_related_diagnostics(&self, _error: &SemanticError) -> Vec<RelatedDiagnostic> {
        // Pour l'instant, pas de diagnostics liés
        Vec::new()
    }
    
    /// Obtient les tags pour une erreur
    fn get_error_tags(&self, error: &SemanticError) -> HashSet<String> {
        let mut tags = HashSet::new();
        
        use crate::semantic::semantic_error::SemanticErrorType::*;
        match &error.error {
            TypeError(_) => {
                tags.insert("type-system".to_string());
            }
            SymbolError(_) => {
                tags.insert("symbol-resolution".to_string());
            }
            // _ => {}
        }
        
        tags
    }
    
    /// Affiche tous les diagnostics
    pub fn display_all(&self) {
        if self.diagnostics.is_empty() {
            return;
        }
        
        // Grouper par fichier
        let mut by_file: HashMap<PathBuf, Vec<&Diagnostic>> = HashMap::new();
        for diagnostic in &self.diagnostics {
            by_file.entry(diagnostic.position.file.clone())
                   .or_insert_with(Vec::new)
                   .push(diagnostic);
        }
        
        // Afficher par fichier
        for (file, diagnostics) in by_file {
            println!("\n=== {} ===", file.display());
            
            // Trier par position
            let mut sorted = diagnostics;
            sorted.sort_by_key(|d| (d.position.line, d.position.column));
            
            for diagnostic in sorted {
                self.formatter.format_diagnostic(diagnostic, &self.config);
            }
        }
        
        // Afficher le résumé
        self.display_summary();
    }
    
    /// Affiche le résumé des diagnostics
    pub fn display_summary(&self) {
        println!("\n=== Diagnostic Summary ===");
        
        for (level, count) in &self.counts {
            if *count > 0 {
                println!("{}: {}", level, count);
            }
        }
        
        let error_count = self.counts.get(&DiagnosticLevel::Error).unwrap_or(&0);
        let warning_count = self.counts.get(&DiagnosticLevel::Warning).unwrap_or(&0);
        
        if *error_count == 0 && *warning_count == 0 {
            println!("✅ No errors or warnings!");
        } else if *error_count == 0 {
            println!("⚠️  {} warning(s) found", warning_count);
        } else {
            println!("❌ {} error(s) and {} warning(s) found", error_count, warning_count);
        }
    }
    
    /// Vérifie s'il y a des erreurs
    pub fn has_errors(&self) -> bool {
        self.counts.get(&DiagnosticLevel::Error).unwrap_or(&0) > &0
    }
    
    /// Obtient le nombre d'erreurs
    pub fn error_count(&self) -> usize {
        *self.counts.get(&DiagnosticLevel::Error).unwrap_or(&0)
    }
    
    /// Obtient le nombre d'avertissements
    pub fn warning_count(&self) -> usize {
        *self.counts.get(&DiagnosticLevel::Warning).unwrap_or(&0)
    }
    
    /// Obtient tous les diagnostics
    pub fn get_diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
    
    /// Efface tous les diagnostics
    pub fn clear(&mut self) {
        self.diagnostics.clear();
        self.counts.clear();
    }
}