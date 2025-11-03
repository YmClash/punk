// src/semantic/diagnostics/suggestion_engine.rs

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::semantic::semantic_error::{SemanticError, SemanticErrorType};
use crate::semantic::context::CompilationContext;

/// Représente une suggestion de correction
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// Message décrivant la suggestion
    pub message: String,
    /// Code de remplacement suggéré
    pub replacement: Option<String>,
    /// Exemple d'utilisation
    pub example: Option<String>,
    /// Confidence de la suggestion (0.0 à 1.0)
    pub confidence: f32,
    /// Actions automatiques possibles
    pub actions: Vec<SuggestionAction>,
}

/// Action automatique suggérée
#[derive(Debug, Clone)]
pub enum SuggestionAction {
    /// Remplacer du texte
    Replace { start: usize, end: usize, text: String },
    /// Insérer du texte
    Insert { position: usize, text: String },
    /// Supprimer du texte
    Delete { start: usize, end: usize },
    /// Importer un module
    Import { module: String },
    /// Ajouter une annotation de type
    AddTypeAnnotation { variable: String, type_name: String },
}

/// Moteur de génération de suggestions
pub struct SuggestionEngine {
    context: Rc<RefCell<CompilationContext>>,
    /// Cache des suggestions par type d'erreur
    suggestion_cache: HashMap<String, Vec<Suggestion>>,
    /// Distance de Levenshtein maximale pour les suggestions
    max_edit_distance: usize,
}

impl SuggestionEngine {
    pub fn new(context: Rc<RefCell<CompilationContext>>) -> Self {
        SuggestionEngine {
            context,
            suggestion_cache: HashMap::new(),
            max_edit_distance: 3,
        }
    }
    
    /// Génère des suggestions pour une erreur
    pub fn get_suggestions(&mut self, error: &SemanticError) -> Vec<Suggestion> {
        use SemanticErrorType::*;
        
        match &error.error {
            TypeError(type_error) => self.get_type_error_suggestions(type_error),
            SymbolError(symbol_error) => self.get_symbol_error_suggestions(symbol_error),
            // _ => Vec::new(),
        }
    }
    
    /// Suggestions pour les erreurs de type
    fn get_type_error_suggestions(&self, error: &crate::semantic::semantic_error::TypeError) -> Vec<Suggestion> {
        use crate::semantic::semantic_error::TypeError::*;
        
        match error {
            TypeMismatch(msg) => {
                let mut suggestions = Vec::new();
                
                // Analyser le message pour extraire les types
                if msg.contains("expected") && msg.contains("found") {
                    suggestions.push(Suggestion {
                        message: "Consider adding an explicit type cast".to_string(),
                        replacement: Some("value as TargetType".to_string()),
                        example: Some("let x = 42 as f64;".to_string()),
                        confidence: 0.7,
                        actions: Vec::new(),
                    });
                }
                
                suggestions
            }
            
            _ => {
                // Cas par défaut pour autres erreurs de type
                vec![
                    Suggestion {
                        message: "Types are incompatible. Consider using a conversion function".to_string(),
                        replacement: None,
                        example: Some("value.into() or value.to_type()".to_string()),
                        confidence: 0.6,
                        actions: Vec::new(),
                    }
                ]
            }
        }
    }
    
    /// Suggestions pour les erreurs de symbole
    fn get_symbol_error_suggestions(&self, error: &crate::semantic::semantic_error::SymbolError) -> Vec<Suggestion> {
        use crate::semantic::semantic_error::SymbolError::*;
        
        match error {
            UnusedSymbol(name) => {
                vec![
                    Suggestion {
                        message: format!("Remove unused variable '{}'", name),
                        replacement: None,
                        example: None,
                        confidence: 0.9,
                        actions: vec![
                            SuggestionAction::Delete {
                                start: 0,
                                end: 0, // À calculer selon la position
                            }
                        ],
                    },
                    Suggestion {
                        message: format!("Prefix with underscore to silence warning: '_{}'", name),
                        replacement: Some(format!("_{}", name)),
                        example: Some(format!("let _{} = value;", name)),
                        confidence: 0.8,
                        actions: vec![
                            SuggestionAction::Replace {
                                start: 0,
                                end: name.len(),
                                text: format!("_{}", name),
                            }
                        ],
                    }
                ]
            }
            
            _ => Vec::new(),
        }
    }
    
    
    /// Trouve des types similaires
    fn find_similar_types(&self, type_name: &str) -> Vec<(String, usize)> {
        let mut similar = Vec::new();
        
        // Types intégrés
        let builtin_types = vec!["int", "float", "bool", "str", "char"];
        for builtin in builtin_types {
            let distance = self.levenshtein_distance(type_name, builtin);
            if distance <= self.max_edit_distance {
                similar.push((builtin.to_string(), distance));
            }
        }
        
        // Trier par distance
        similar.sort_by_key(|&(_, d)| d);
        similar.truncate(3); // Garder les 3 meilleurs
        
        similar
    }
    
    /// Trouve des symboles similaires
    fn find_similar_symbols(&self, symbol_name: &str) -> Vec<(String, usize)> {
        let mut similar = Vec::new();
        
        // Pour l'instant, utiliser des exemples statiques
        // Dans une vraie implémentation, parcourir la table des symboles
        let known_symbols = vec!["x", "y", "z", "result", "value", "index"];
        for known in known_symbols {
            let distance = self.levenshtein_distance(symbol_name, known);
            if distance <= self.max_edit_distance {
                similar.push((known.to_string(), distance));
            }
        }
        
        similar.sort_by_key(|&(_, d)| d);
        similar.truncate(3);
        
        similar
    }
    
    /// Calcule la distance de Levenshtein entre deux chaînes
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();
        
        if len1 == 0 { return len2; }
        if len2 == 0 { return len1; }
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }
        
        for (i, c1) in s1.chars().enumerate() {
            for (j, c2) in s2.chars().enumerate() {
                let cost = if c1 == c2 { 0 } else { 1 };
                matrix[i + 1][j + 1] = std::cmp::min(
                    std::cmp::min(
                        matrix[i][j + 1] + 1,     // Deletion
                        matrix[i + 1][j] + 1       // Insertion
                    ),
                    matrix[i][j] + cost            // Substitution
                );
            }
        }
        
        matrix[len1][len2]
    }
}