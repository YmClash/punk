// src/semantic/error_recovery.rs

use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::cell::RefCell;
use crate::semantic::semantic_error::{SemanticError, SemanticErrorType, Position};
use crate::semantic::context::CompilationContext;
use crate::semantic::types::type_system::TypeId;
use crate::semantic::symbols::{SymbolId, ScopeId};
use crate::parser::ast::{Expression, Statement, Declaration, ASTNode};

/// Stratégies de récupération d'erreur
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RecoveryStrategy {
    /// Ignorer l'erreur et continuer
    Skip,
    
    /// Insérer une valeur par défaut
    InsertDefault,
    
    /// Propager l'erreur au niveau supérieur
    Propagate,
    
    /// Essayer une alternative
    TryAlternative,
    
    /// Marquer comme invalide mais continuer
    MarkInvalid,
}

/// Point de récupération dans l'analyse
#[derive(Debug, Clone)]
pub struct RecoveryPoint {
    /// Position dans le code source
    pub position: Position,
    
    /// Type d'erreur à récupérer
    pub error_type: SemanticErrorType,
    
    /// Stratégie utilisée
    pub strategy: RecoveryStrategy,
    
    /// Context de récupération
    pub context: String,
}

/// Gestionnaire de récupération d'erreurs
pub struct ErrorRecovery {
    /// Contexte de compilation
    context: Rc<RefCell<CompilationContext>>,
    
    /// Erreurs collectées
    pub errors: Vec<SemanticError>,
    
    /// Points de récupération
    pub recovery_points: Vec<RecoveryPoint>,
    
    /// Symboles invalides (à ignorer dans les analyses ultérieures)
    pub invalid_symbols: HashSet<SymbolId>,
    
    /// Types invalides
    pub invalid_types: HashSet<TypeId>,
    
    /// Scopes avec erreurs
    pub problematic_scopes: HashSet<ScopeId>,
    
    /// Limite d'erreurs avant abandon
    pub error_limit: usize,
    
    /// Mode de récupération actif
    pub recovery_mode: bool,
}

impl ErrorRecovery {
    pub fn new(context: Rc<RefCell<CompilationContext>>) -> Self {
        ErrorRecovery {
            context,
            errors: Vec::new(),
            recovery_points: Vec::new(),
            invalid_symbols: HashSet::new(),
            invalid_types: HashSet::new(),
            problematic_scopes: HashSet::new(),
            error_limit: 100,
            recovery_mode: true,
        }
    }
    
    /// Enregistre une erreur et tente de récupérer
    pub fn record_and_recover(
        &mut self,
        error: SemanticError,
        strategy: RecoveryStrategy,
    ) -> Result<(), SemanticError> {
        // Enregistrer l'erreur
        self.errors.push(error.clone());
        
        // Vérifier la limite d'erreurs
        if self.errors.len() >= self.error_limit {
            return Err(SemanticError::new(
                SemanticErrorType::TypeError(
                    crate::semantic::semantic_error::TypeError::InvalidType(
                        format!("Too many errors ({}), compilation aborted", self.errors.len())
                    )
                ),
                "Error limit exceeded".to_string(),
                error.position.clone(),
            ));
        }
        
        // Si la récupération est désactivée, propager l'erreur
        if !self.recovery_mode {
            return Err(error);
        }
        
        // Créer un point de récupération
        let recovery_point = RecoveryPoint {
            position: error.position.clone(),
            error_type: error.error.clone(),
            strategy: strategy.clone(),
            context: error.message.clone(),
        };
        
        self.recovery_points.push(recovery_point);
        
        // Appliquer la stratégie de récupération
        match strategy {
            RecoveryStrategy::Skip => {
                // Simplement ignorer et continuer
                Ok(())
            }
            RecoveryStrategy::InsertDefault => {
                // Insérer une valeur par défaut selon le contexte
                self.insert_default_value(&error)
            }
            RecoveryStrategy::Propagate => {
                // Propager l'erreur mais continuer l'analyse
                Err(error)
            }
            RecoveryStrategy::TryAlternative => {
                // Essayer une approche alternative
                self.try_alternative_approach(&error)
            }
            RecoveryStrategy::MarkInvalid => {
                // Marquer les éléments comme invalides
                self.mark_as_invalid(&error);
                Ok(())
            }
        }
    }
    
    /// Insère une valeur par défaut pour continuer l'analyse
    fn insert_default_value(&mut self, error: &SemanticError) -> Result<(), SemanticError> {
        match &error.error {
            SemanticErrorType::TypeError(_) => {
                // Insérer un type par défaut (infer)
                let ctx = self.context.borrow();
                let mut type_system = ctx.types.borrow_mut();
                // Utiliser un compteur statique ou un ID basé sur le timestamp
                static mut TYPE_VAR_COUNTER: u32 = 1000;
                let type_var = unsafe {
                    TYPE_VAR_COUNTER += 1;
                    crate::semantic::types::type_system::TypeVarId {
                        id: crate::semantic::types::type_system::TypeId(TYPE_VAR_COUNTER),
                        name: format!("T{}", TYPE_VAR_COUNTER),
                    }
                };
                let _infer_type = type_system.register_type(
                    crate::semantic::types::type_system::TypeKind::Infer(type_var)
                );
                // Le type par défaut est maintenant disponible
                Ok(())
            }
            SemanticErrorType::SymbolError(_) => {
                // Créer un symbole temporaire
                Ok(())
            }
            _ => Ok(())
        }
    }
    
    /// Essaie une approche alternative
    fn try_alternative_approach(&mut self, error: &SemanticError) -> Result<(), SemanticError> {
        match &error.error {
            SemanticErrorType::TypeError(type_err) => {
                // Essayer la coercion de type
                self.try_type_coercion(type_err)
            }
            SemanticErrorType::SymbolError(sym_err) => {
                // Essayer de trouver un symbole similaire
                self.try_similar_symbol(sym_err)
            }
        }
    }
    
    /// Marque les éléments comme invalides
    fn mark_as_invalid(&mut self, error: &SemanticError) {
        match &error.error {
            SemanticErrorType::TypeError(_) => {
                // Marquer le type comme invalide
                // Note: nécessite d'extraire l'ID du type depuis l'erreur
            }
            SemanticErrorType::SymbolError(_) => {
                // Marquer le symbole comme invalide
                // Note: nécessite d'extraire l'ID du symbole depuis l'erreur
            }
        }
    }
    
    /// Essaie la coercion de type
    fn try_type_coercion(
        &mut self,
        _type_error: &crate::semantic::semantic_error::TypeError,
    ) -> Result<(), SemanticError> {
        // Implémenter la logique de coercion
        // Par exemple, int -> float, &T -> T, etc.
        Ok(())
    }
    
    /// Essaie de trouver un symbole similaire
    fn try_similar_symbol(
        &mut self,
        _symbol_error: &crate::semantic::semantic_error::SymbolError,
    ) -> Result<(), SemanticError> {
        // Implémenter la recherche de symboles similaires
        // Utiliser la distance de Levenshtein par exemple
        Ok(())
    }
    
    /// Vérifie si un symbole est invalide
    pub fn is_symbol_invalid(&self, symbol_id: SymbolId) -> bool {
        self.invalid_symbols.contains(&symbol_id)
    }
    
    /// Vérifie si un type est invalide
    pub fn is_type_invalid(&self, type_id: TypeId) -> bool {
        self.invalid_types.contains(&type_id)
    }
    
    /// Vérifie si un scope a des problèmes
    pub fn is_scope_problematic(&self, scope_id: ScopeId) -> bool {
        self.problematic_scopes.contains(&scope_id)
    }
    
    /// Récupère depuis une expression invalide
    pub fn recover_from_expression(
        &mut self,
        expr: &Expression,
        error: SemanticError,
    ) -> TypeId {
        // Enregistrer l'erreur
        let _ = self.record_and_recover(error, RecoveryStrategy::MarkInvalid);
        
        // Retourner un type par défaut
        let ctx = self.context.borrow();
        let mut type_system = ctx.types.borrow_mut();
        // Utiliser un compteur statique ou un ID basé sur le timestamp
        static mut TYPE_VAR_COUNTER_2: u32 = 2000;
        let type_var = unsafe {
            TYPE_VAR_COUNTER_2 += 1;
            crate::semantic::types::type_system::TypeVarId {
                id: crate::semantic::types::type_system::TypeId(TYPE_VAR_COUNTER_2),
                name: format!("T{}", TYPE_VAR_COUNTER_2),
            }
        };
        type_system.register_type(
            crate::semantic::types::type_system::TypeKind::Infer(type_var)
        )
    }
    
    /// Récupère depuis une déclaration invalide
    pub fn recover_from_declaration(
        &mut self,
        _decl: &Declaration,
        error: SemanticError,
    ) -> Result<(), SemanticError> {
        self.record_and_recover(error, RecoveryStrategy::Skip)
    }
    
    /// Récupère depuis un statement invalide
    pub fn recover_from_statement(
        &mut self,
        _stmt: &Statement,
        error: SemanticError,
    ) -> Result<(), SemanticError> {
        self.record_and_recover(error, RecoveryStrategy::Skip)
    }
    
    /// Génère un rapport des erreurs et récupérations
    pub fn generate_report(&self) -> ErrorRecoveryReport {
        ErrorRecoveryReport {
            total_errors: self.errors.len(),
            recovered_errors: self.recovery_points.len(),
            invalid_symbols: self.invalid_symbols.len(),
            invalid_types: self.invalid_types.len(),
            problematic_scopes: self.problematic_scopes.len(),
            recovery_strategies: self.count_strategies(),
            most_common_errors: self.analyze_error_patterns(),
        }
    }
    
    /// Compte les stratégies utilisées
    fn count_strategies(&self) -> HashMap<RecoveryStrategy, usize> {
        let mut counts = HashMap::new();
        for point in &self.recovery_points {
            *counts.entry(point.strategy.clone()).or_insert(0) += 1;
        }
        counts
    }
    
    /// Analyse les patterns d'erreurs
    fn analyze_error_patterns(&self) -> Vec<(String, usize)> {
        let mut patterns = HashMap::new();
        
        for error in &self.errors {
            let pattern = match &error.error {
                SemanticErrorType::TypeError(_) => "Type Error",
                SemanticErrorType::SymbolError(_) => "Symbol Error",
            };
            *patterns.entry(pattern.to_string()).or_insert(0) += 1;
        }
        
        let mut sorted: Vec<_> = patterns.into_iter().collect();
        sorted.sort_by_key(|&(_, count)| std::cmp::Reverse(count));
        sorted
    }
    
    /// Nettoie les ressources après récupération
    pub fn cleanup(&mut self) {
        // Nettoyer les symboles invalides qui ne sont plus référencés
        // Nettoyer les types invalides
        // Libérer la mémoire non nécessaire
    }
}

/// Rapport de récupération d'erreurs
#[derive(Debug)]
pub struct ErrorRecoveryReport {
    pub total_errors: usize,
    pub recovered_errors: usize,
    pub invalid_symbols: usize,
    pub invalid_types: usize,
    pub problematic_scopes: usize,
    pub recovery_strategies: HashMap<RecoveryStrategy, usize>,
    pub most_common_errors: Vec<(String, usize)>,
}

impl ErrorRecoveryReport {
    pub fn display(&self) {
        println!("=== Error Recovery Report ===");
        println!("Total errors: {}", self.total_errors);
        println!("Recovered errors: {}", self.recovered_errors);
        println!("Invalid symbols: {}", self.invalid_symbols);
        println!("Invalid types: {}", self.invalid_types);
        println!("Problematic scopes: {}", self.problematic_scopes);
        
        println!("\nRecovery Strategies Used:");
        for (strategy, count) in &self.recovery_strategies {
            println!("  {:?}: {}", strategy, count);
        }
        
        println!("\nMost Common Error Types:");
        for (pattern, count) in &self.most_common_errors {
            println!("  {}: {}", pattern, count);
        }
    }
}

/// Trait pour les éléments récupérables
pub trait Recoverable {
    /// Tente de récupérer depuis une erreur
    fn recover(&mut self, error: SemanticError) -> Result<(), SemanticError>;
    
    /// Vérifie si l'élément est dans un état valide
    fn is_valid(&self) -> bool;
    
    /// Réinitialise à un état sûr
    fn reset_to_safe_state(&mut self);
}

/// Gestionnaire de points de sauvegarde
pub struct CheckpointManager {
    checkpoints: Vec<AnalysisCheckpoint>,
    current_checkpoint: Option<usize>,
}

/// Point de sauvegarde de l'analyse
#[derive(Clone)]
struct AnalysisCheckpoint {
    position: Position,
    context_state: Rc<RefCell<CompilationContext>>,
    errors_count: usize,
}

impl CheckpointManager {
    pub fn new() -> Self {
        CheckpointManager {
            checkpoints: Vec::new(),
            current_checkpoint: None,
        }
    }
    
    /// Crée un nouveau point de sauvegarde
    pub fn create_checkpoint(
        &mut self,
        position: Position,
        context: Rc<RefCell<CompilationContext>>,
        errors_count: usize,
    ) {
        let checkpoint = AnalysisCheckpoint {
            position,
            context_state: context,
            errors_count,
        };
        
        self.checkpoints.push(checkpoint);
        self.current_checkpoint = Some(self.checkpoints.len() - 1);
    }
    
    /// Restaure au dernier point de sauvegarde
    pub fn restore_last_checkpoint(&mut self) -> Option<Rc<RefCell<CompilationContext>>> {
        if let Some(index) = self.current_checkpoint {
            if index > 0 {
                self.current_checkpoint = Some(index - 1);
                return Some(self.checkpoints[index - 1].context_state.clone());
            }
        }
        None
    }
    
    /// Nettoie les points de sauvegarde
    pub fn clear_checkpoints(&mut self) {
        self.checkpoints.clear();
        self.current_checkpoint = None;
    }
}