// src/semantic/borrow_checker.rs

use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::cell::RefCell;
use crate::semantic::semantic_error::{SemanticError, SemanticErrorType, Position};
use crate::semantic::symbols::{SymbolId, ScopeId, SourceLocation};
use crate::semantic::symbol_table::SymbolTable;
use crate::semantic::flow::control_flow_graph::{ControlFlowGraph, BlockId, CFGInstruction};
use crate::semantic::context::CompilationContext;

/// Types de références possibles pour une variable
#[derive(Debug, Clone, PartialEq)]
pub enum BorrowKind {
    /// Référence immutable (&T)
    Immutable,

    /// Référence mutable (&mut T)
    Mutable,

    /// Emprunt de la valeur (move)
    Move,

    /// Utilisation simple (lecture)
    Read,

    /// Modification de la valeur
    Write,
}

/// Représente une référence à un symbole
#[derive(Debug, Clone)]
pub struct BorrowInfo {
    /// L'ID du symbole emprunté
    pub symbol_id: SymbolId,

    /// Le type d'emprunt
    pub kind: BorrowKind,

    /// Où l'emprunt a lieu
    pub location: SourceLocation,

    /// Scope dans lequel l'emprunt est actif
    pub scope_id: ScopeId,

    /// Durée de vie estimée (optionnel)
    pub lifetime: Option<String>,
}

/// Erreurs liées au borrow checking
#[derive(Debug, Clone)]
pub enum BorrowErrorKind {
    /// Emprunt mutable alors qu'il existe des emprunts immutables
    MutableBorrowWithImmutableBorrows {
        symbol_id: SymbolId,
        immutable_locations: Vec<SourceLocation>,
        mutable_location: SourceLocation,
    },

    /// Plusieurs emprunts mutables simultanés
    MultipleMutableBorrows {
        symbol_id: SymbolId,
        locations: Vec<SourceLocation>,
    },

    /// Utilisation après un move
    UseAfterMove {
        symbol_id: SymbolId,
        move_location: SourceLocation,
        use_location: SourceLocation,
    },

    /// Modification d'une variable immutable
    ModifyImmutableVariable {
        symbol_id: SymbolId,
        location: SourceLocation,
    },

    /// Variable non initialisée
    UninitializedVariable {
        symbol_id: SymbolId,
        use_location: SourceLocation,
    },

    /// Lifetime invalide
    InvalidLifetime {
        symbol_id: SymbolId,
        location: SourceLocation,
        message: String,
    },
}

/// Convertit une erreur de borrow en erreur sémantique
fn borrow_error_to_semantic(error: BorrowErrorKind, position: Position) -> SemanticError {
    // Créer une erreur sémantique de type approprié
    // Dans une implémentation réelle, on pourrait ajouter un type spécifique
    // comme SemanticErrorType::BorrowError(...)
    match error {
        BorrowErrorKind::MutableBorrowWithImmutableBorrows { .. } => {
            SemanticError::new(
                SemanticErrorType::TypeError(
                    crate::semantic::semantic_error::TypeError::TypeMismatch(
                        "Cannot borrow mutably while immutable borrows exist".to_string()
                    )
                ),
                "Mutable borrow conflict".to_string(),
                position
            )
        },
        BorrowErrorKind::MultipleMutableBorrows { .. } => {
            SemanticError::new(
                SemanticErrorType::TypeError(
                    crate::semantic::semantic_error::TypeError::TypeMismatch(
                        "Cannot have multiple mutable borrows".to_string()
                    )
                ),
                "Multiple mutable borrows".to_string(),
                position
            )
        },
        BorrowErrorKind::UseAfterMove { .. } => {
            SemanticError::new(
                SemanticErrorType::TypeError(
                    crate::semantic::semantic_error::TypeError::TypeMismatch(
                        "Use after move".to_string()
                    )
                ),
                "Value used after it was moved".to_string(),
                position
            )
        },
        BorrowErrorKind::ModifyImmutableVariable { .. } => {
            SemanticError::new(
                SemanticErrorType::TypeError(
                    crate::semantic::semantic_error::TypeError::InvalidType(
                        "Cannot modify immutable variable".to_string()
                    )
                ),
                "Attempt to modify immutable value".to_string(),
                position
            )
        },
        BorrowErrorKind::UninitializedVariable { .. } => {
            SemanticError::new(
                SemanticErrorType::TypeError(
                    crate::semantic::semantic_error::TypeError::InvalidType(
                        "Variable not initialized".to_string()
                    )
                ),
                "Use of uninitialized variable".to_string(),
                position
            )
        },
        BorrowErrorKind::InvalidLifetime { message, .. } => {
            SemanticError::new(
                SemanticErrorType::TypeError(
                    crate::semantic::semantic_error::TypeError::InvalidTypeParameter(
                        message
                    )
                ),
                "Invalid lifetime".to_string(),
                position
            )
        },
    }
}

/// Gère l'état des emprunts et la mutabilité avec intégration CFG
#[derive(Debug, Clone)]
pub struct BorrowChecker {
    /// Borrows actifs par symbole
    pub active_borrows: HashMap<SymbolId, Vec<BorrowInfo>>,

    /// Variables initialisées
    pub initialized_variables: HashSet<SymbolId>,

    /// Variables moved
    pub moved_variables: HashMap<SymbolId, SourceLocation>,

    /// Historique des borrows (pour debug et reporting)
    pub borrow_history: Vec<BorrowInfo>,
    
    /// Control Flow Graph pour l'analyse basée sur le flux
    cfg: Option<Rc<RefCell<ControlFlowGraph>>>,
    
    /// État des emprunts par bloc du CFG
    block_borrow_states: HashMap<BlockId, BlockBorrowState>,
    
    /// Contexte de compilation partagé
    context: Option<Rc<RefCell<CompilationContext>>>,
}

/// État des emprunts dans un bloc du CFG
#[derive(Debug, Clone)]
pub struct BlockBorrowState {
    /// Emprunts actifs à l'entrée du bloc
    pub entry_borrows: HashMap<SymbolId, Vec<BorrowInfo>>,
    
    /// Emprunts actifs à la sortie du bloc
    pub exit_borrows: HashMap<SymbolId, Vec<BorrowInfo>>,
    
    /// Variables initialisées dans ce bloc
    pub initialized: HashSet<SymbolId>,
    
    /// Variables moved dans ce bloc
    pub moved: HashMap<SymbolId, SourceLocation>,
}

impl BorrowChecker {
    /// Crée un nouveau borrow checker
    pub fn new() -> Self {
        BorrowChecker {
            active_borrows: HashMap::new(),
            initialized_variables: HashSet::new(),
            moved_variables: HashMap::new(),
            borrow_history: Vec::new(),
            cfg: None,
            block_borrow_states: HashMap::new(),
            context: None,
        }
    }
    
    /// Associe un CFG au borrow checker
    pub fn with_cfg(mut self, cfg: Rc<RefCell<ControlFlowGraph>>) -> Self {
        self.cfg = Some(cfg);
        self
    }
    
    /// Associe un contexte de compilation
    pub fn with_context(mut self, context: Rc<RefCell<CompilationContext>>) -> Self {
        self.context = Some(context);
        self
    }

    /// Vérifie si une variable est initialisée
    pub fn is_initialized(&self, symbol_id: SymbolId) -> bool {
        self.initialized_variables.contains(&symbol_id)
    }

    /// Marque une variable comme initialisée
    pub fn mark_initialized(&mut self, symbol_id: SymbolId) {
        self.initialized_variables.insert(symbol_id);
    }

    /// Enregistre un emprunt ou une utilisation
    pub fn register_borrow(
        &mut self,
        symbol_id: SymbolId,
        kind: BorrowKind,
        location: SourceLocation,
        scope_id: ScopeId,
        lifetime: Option<String>,
    ) -> Result<(), BorrowErrorKind> {
        // Vérifier si la variable a été moved
        if let Some(move_loc) = self.moved_variables.get(&symbol_id) {
            return Err(BorrowErrorKind::UseAfterMove {
                symbol_id,
                move_location: move_loc.clone(),
                use_location: location.clone(),
            });
        }

        // Vérifier l'initialisation pour les lectures
        if matches!(kind, BorrowKind::Read | BorrowKind::Immutable | BorrowKind::Mutable)
            && !self.is_initialized(symbol_id) {
            return Err(BorrowErrorKind::UninitializedVariable {
                symbol_id,
                use_location: location.clone(),
            });
        }

        // Gérer le move
        if matches!(kind, BorrowKind::Move) {
            self.moved_variables.insert(symbol_id, location.clone());
            // Supprimer les emprunts actifs car la valeur est moved
            self.active_borrows.remove(&symbol_id);
        }
        // Les opérations Read et Write ne créent pas d'emprunts persistants
        else if matches!(kind, BorrowKind::Read | BorrowKind::Write) {
            // Vérifier qu'il n'y a pas d'emprunts actifs qui bloquent
            if let Some(borrows) = self.active_borrows.get(&symbol_id) {
                for borrow in borrows {
                    match (&kind, &borrow.kind) {
                        (BorrowKind::Write, _) | (_, BorrowKind::Mutable) => {
                            // L'écriture est bloquée par n'importe quel emprunt
                            // Un emprunt mutable bloque tout
                            return Err(BorrowErrorKind::MutableBorrowWithImmutableBorrows {
                                symbol_id,
                                immutable_locations: vec![borrow.location.clone()],
                                mutable_location: location.clone(),
                            });
                        }
                        _ => {}
                    }
                }
            }
            
            // Si c'est une écriture, marquer comme initialisée
            if matches!(kind, BorrowKind::Write) {
                self.mark_initialized(symbol_id);
            }
            
            // Pas d'emprunt persistant pour Read/Write
            return Ok(());
        }
        // Gérer les emprunts persistants (Immutable et Mutable)
        else {
            let borrow_info = BorrowInfo {
                symbol_id,
                kind: kind.clone(),
                location: location.clone(),
                scope_id,
                lifetime,
            };

            // Vérifier les règles d'emprunt
            match kind {
                BorrowKind::Mutable | BorrowKind::Write => {
                    // Vérifier s'il y a des emprunts immutables actifs
                    if let Some(borrows) = self.active_borrows.get(&symbol_id) {
                        let immutable_borrows: Vec<_> = borrows.iter()
                            .filter(|b| matches!(b.kind, BorrowKind::Immutable | BorrowKind::Read))
                            .collect();

                        if !immutable_borrows.is_empty() {
                            return Err(BorrowErrorKind::MutableBorrowWithImmutableBorrows {
                                symbol_id,
                                immutable_locations: immutable_borrows.iter().map(|b| b.location.clone()).collect(),
                                mutable_location: location,
                            });
                        }

                        // Vérifier s'il y a déjà un emprunt mutable
                        let mutable_borrows: Vec<_> = borrows.iter()
                            .filter(|b| matches!(b.kind, BorrowKind::Mutable | BorrowKind::Write))
                            .collect();

                        if !mutable_borrows.is_empty() {
                            return Err(BorrowErrorKind::MultipleMutableBorrows {
                                symbol_id,
                                locations: mutable_borrows.iter().map(|b| b.location.clone()).collect(),
                            });
                        }
                    }
                },
                BorrowKind::Immutable | BorrowKind::Read => {
                    // Vérifier s'il y a des emprunts mutables actifs
                    if let Some(borrows) = self.active_borrows.get(&symbol_id) {
                        let mutable_borrows: Vec<_> = borrows.iter()
                            .filter(|b| matches!(b.kind, BorrowKind::Mutable | BorrowKind::Write))
                            .collect();

                        if !mutable_borrows.is_empty() {
                            return Err(BorrowErrorKind::MutableBorrowWithImmutableBorrows {
                                symbol_id,
                                immutable_locations: vec![location.clone()],
                                mutable_location: mutable_borrows[0].location.clone(),
                            });
                        }
                    }
                },
                _ => {} // Les autres types peuvent coexister
            }

            // Ajouter l'emprunt aux emprunts actifs
            self.active_borrows.entry(symbol_id)
                .or_insert_with(Vec::new)
                .push(borrow_info.clone());

            // Enregistrer dans l'historique
            self.borrow_history.push(borrow_info);
        }

        Ok(())
    }

    /// Libère les emprunts d'un scope lorsqu'il est quitté
    pub fn release_borrows_for_scope(&mut self, scope_id: ScopeId) {
        for borrows in self.active_borrows.values_mut() {
            borrows.retain(|borrow| borrow.scope_id != scope_id);
        }

        // Nettoyer les entrées vides
        self.active_borrows.retain(|_, borrows| !borrows.is_empty());
    }

    /// Vérifie si un symbole a des emprunts actifs
    pub fn has_active_borrows(&self, symbol_id: SymbolId) -> bool {
        match self.active_borrows.get(&symbol_id) {
            Some(borrows) => !borrows.is_empty(),
            None => false,
        }
    }

    /// Vérifie si un symbole a été moved
    pub fn is_moved(&self, symbol_id: SymbolId) -> bool {
        self.moved_variables.contains_key(&symbol_id)
    }

    /// Obtient tous les emprunts actifs d'un symbole
    pub fn get_active_borrows(&self, symbol_id: SymbolId) -> Vec<&BorrowInfo> {
        match self.active_borrows.get(&symbol_id) {
            Some(borrows) => borrows.iter().collect(),
            None => Vec::new(),
        }
    }
    
    
    // ===== Méthodes d'analyse basées sur le CFG =====
    
    /// Analyse les emprunts en utilisant le Control Flow Graph
    pub fn analyze_with_cfg(&mut self) -> Result<(), Vec<BorrowErrorKind>> {
        if self.cfg.is_none() {
            // Pas de CFG, utiliser l'analyse classique
            return Ok(());
        }
        
        let mut errors = Vec::new();
        
        // D'abord, obtenir les informations nécessaires du CFG
        let (all_blocks, sorted_blocks) = {
            let cfg = self.cfg.as_ref().unwrap().borrow();
            (cfg.get_all_blocks(), cfg.topological_sort())
        };
        
        // Initialiser l'état pour chaque bloc
        for block_id in all_blocks {
            self.block_borrow_states.insert(block_id, BlockBorrowState {
                entry_borrows: HashMap::new(),
                exit_borrows: HashMap::new(),
                initialized: HashSet::new(),
                moved: HashMap::new(),
            });
        }
        
        // Analyser chaque bloc dans l'ordre topologique
        for block_id in sorted_blocks {
            // Obtenir les informations du bloc avant l'analyse
            let block_instructions = {
                let cfg = self.cfg.as_ref().unwrap().borrow();
                cfg.get_block(block_id).map(|block| block.instructions.clone())
            };
            
            // Analyser le bloc avec les informations extraites
            if let Some(instructions) = block_instructions {
                if let Err(block_errors) = self.analyze_block_instructions(block_id, &instructions) {
                    errors.extend(block_errors);
                }
            }
        }
        
        // Vérifier la cohérence entre les blocs
        {
            let cfg = self.cfg.as_ref().unwrap().borrow();
            if let Err(flow_errors) = self.verify_dataflow_consistency(&*cfg) {
                errors.extend(flow_errors);
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Analyse les instructions d'un bloc du CFG
    fn analyze_block_instructions(&mut self, block_id: BlockId, instructions: &[crate::semantic::flow::control_flow_graph::Instruction]) -> Result<(), Vec<BorrowErrorKind>> {
        let mut errors = Vec::new();
        
        // Récupérer l'état d'entrée (merge des prédécesseurs)
        let entry_state = {
            let cfg = self.cfg.as_ref().unwrap().borrow();
            self.compute_entry_state(&*cfg, block_id)
        };
        
        // Appliquer l'état d'entrée
        self.apply_block_state(&entry_state);
        
        // Analyser chaque instruction du bloc
        for instruction in instructions {
            // Convertir Instruction en CFGInstruction
            let cfg_instruction = CFGInstruction::Generic(instruction.clone());
            if let Err(err) = self.analyze_instruction(&cfg_instruction) {
                errors.push(err);
            }
        }
        
        // Sauvegarder l'état de sortie
        if let Some(state) = self.block_borrow_states.get_mut(&block_id) {
            state.exit_borrows = self.active_borrows.clone();
            state.initialized = self.initialized_variables.clone();
            state.moved = self.moved_variables.clone();
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Calcule l'état d'entrée d'un bloc (merge des prédécesseurs)
    fn compute_entry_state(&self, cfg: &ControlFlowGraph, block_id: BlockId) -> BlockBorrowState {
        let predecessors = cfg.get_predecessors(block_id);
        
        if predecessors.is_empty() {
            // Bloc d'entrée : état initial vide
            return BlockBorrowState {
                entry_borrows: HashMap::new(),
                exit_borrows: HashMap::new(),
                initialized: HashSet::new(),
                moved: HashMap::new(),
            };
        }
        
        // Merger les états de sortie des prédécesseurs
        let mut merged_state = BlockBorrowState {
            entry_borrows: HashMap::new(),
            exit_borrows: HashMap::new(),
            initialized: HashSet::new(),
            moved: HashMap::new(),
        };
        
        // Pour les variables initialisées : intersection (doivent être initialisées dans tous les chemins)
        let mut first = true;
        for pred_id in &predecessors {
            if let Some(pred_state) = self.block_borrow_states.get(pred_id) {
                if first {
                    merged_state.initialized = pred_state.initialized.clone();
                    first = false;
                } else {
                    merged_state.initialized = merged_state.initialized
                        .intersection(&pred_state.initialized)
                        .cloned()
                        .collect();
                }
            }
        }
        
        // Pour les emprunts : union (tous les emprunts possibles)
        for pred_id in &predecessors {
            if let Some(pred_state) = self.block_borrow_states.get(pred_id) {
                for (symbol, borrows) in &pred_state.exit_borrows {
                    merged_state.entry_borrows
                        .entry(*symbol)
                        .or_insert_with(Vec::new)
                        .extend(borrows.clone());
                }
            }
        }
        
        // Pour les moves : union (si moved dans n'importe quel chemin)
        for pred_id in &predecessors {
            if let Some(pred_state) = self.block_borrow_states.get(pred_id) {
                merged_state.moved.extend(pred_state.moved.clone());
            }
        }
        
        merged_state
    }
    
    /// Applique un état de bloc au checker
    fn apply_block_state(&mut self, state: &BlockBorrowState) {
        self.active_borrows = state.entry_borrows.clone();
        self.initialized_variables = state.initialized.clone();
        self.moved_variables = state.moved.clone();
    }
    
    /// Analyse une instruction du CFG
    fn analyze_instruction(&mut self, instruction: &CFGInstruction) -> Result<(), BorrowErrorKind> {
        match instruction {
            CFGInstruction::Assign { target, value: _ } => {
                // Marquer la variable comme initialisée
                self.mark_initialized(*target);
                Ok(())
            }
            CFGInstruction::Read(symbol_id) => {
                // Vérifier que la variable est initialisée et non moved
                if !self.is_initialized(*symbol_id) {
                    return Err(BorrowErrorKind::UninitializedVariable {
                        symbol_id: *symbol_id,
                        use_location: SourceLocation::default(),
                    });
                }
                
                if let Some(move_loc) = self.moved_variables.get(symbol_id) {
                    return Err(BorrowErrorKind::UseAfterMove {
                        symbol_id: *symbol_id,
                        move_location: move_loc.clone(),
                        use_location: SourceLocation::default(),
                    });
                }
                
                Ok(())
            }
            CFGInstruction::Write(symbol_id) => {
                // Vérifier qu'il n'y a pas d'emprunts actifs
                if let Some(borrows) = self.active_borrows.get(symbol_id) {
                    if !borrows.is_empty() {
                        let locations: Vec<_> = borrows.iter().map(|b| b.location.clone()).collect();
                        return Err(BorrowErrorKind::MutableBorrowWithImmutableBorrows {
                            symbol_id: *symbol_id,
                            immutable_locations: locations,
                            mutable_location: SourceLocation::default(),
                        });
                    }
                }
                
                self.mark_initialized(*symbol_id);
                Ok(())
            }
            CFGInstruction::Borrow { target, source, is_mutable } => {
                // Enregistrer l'emprunt
                let kind = if *is_mutable {
                    BorrowKind::Mutable
                } else {
                    BorrowKind::Immutable
                };
                
                self.register_borrow(
                    *source,
                    kind,
                    SourceLocation::default(),
                    ScopeId(0), // À améliorer avec le vrai scope
                    None,
                )?;
                
                // Marquer la cible comme initialisée
                self.mark_initialized(*target);
                Ok(())
            }
            CFGInstruction::Move { target, source } => {
                // Enregistrer le move
                self.moved_variables.insert(*source, SourceLocation::default());
                self.active_borrows.remove(source);
                
                // Marquer la cible comme initialisée
                self.mark_initialized(*target);
                Ok(())
            }
            _ => Ok(()),
        }
    }
    
    /// Vérifie la cohérence du flux de données entre les blocs
    fn verify_dataflow_consistency(&self, cfg: &ControlFlowGraph) -> Result<(), Vec<BorrowErrorKind>> {
        let mut errors = Vec::new();
        
        // Vérifier que les emprunts sont valides à travers les edges du CFG
        for block_id in cfg.get_all_blocks() {
            let successors = cfg.get_successors(block_id);
            
            if let Some(block_state) = self.block_borrow_states.get(&block_id) {
                for succ_id in successors {
                    if let Some(succ_state) = self.block_borrow_states.get(&succ_id) {
                        // Vérifier que les emprunts actifs sont cohérents
                        for (symbol, borrows) in &block_state.exit_borrows {
                            if let Some(succ_borrows) = succ_state.entry_borrows.get(symbol) {
                                // Vérifier la compatibilité des emprunts
                                if !self.are_borrows_compatible(borrows, succ_borrows) {
                                    errors.push(BorrowErrorKind::InvalidLifetime {
                                        symbol_id: *symbol,
                                        location: SourceLocation::default(),
                                        message: "Incompatible borrows across control flow edge".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Vérifie si deux ensembles d'emprunts sont compatibles
    fn are_borrows_compatible(&self, borrows1: &[BorrowInfo], borrows2: &[BorrowInfo]) -> bool {
        // Règles de compatibilité :
        // - Plusieurs emprunts immutables sont OK
        // - Un seul emprunt mutable à la fois
        // - Pas de mélange immutable/mutable
        
        let has_mutable1 = borrows1.iter().any(|b| matches!(b.kind, BorrowKind::Mutable));
        let has_mutable2 = borrows2.iter().any(|b| matches!(b.kind, BorrowKind::Mutable));
        
        if has_mutable1 || has_mutable2 {
            // S'il y a un emprunt mutable, il doit être le seul
            borrows1.len() <= 1 && borrows2.len() <= 1
        } else {
            // Emprunts immutables uniquement : toujours compatibles
            true
        }
    }
}

/// Extension de SymbolTable pour intégrer le borrow checker
pub trait MutabilityManager {
    /// Vérifie si un symbole est mutable
    fn is_mutable(&self, symbol_id: SymbolId) -> Result<bool, SemanticError>;

    /// Marque un symbole comme initialisé
    fn mark_initialized(&mut self, symbol_id: SymbolId) -> Result<(), SemanticError>;

    /// Vérifie si un symbole est initialisé
    fn is_initialized(&self, symbol_id: SymbolId) -> Result<bool, SemanticError>;

    /// Enregistre une utilisation en lecture
    fn register_read(&mut self, symbol_id: SymbolId, location: SourceLocation) -> Result<(), SemanticError>;

    /// Enregistre une utilisation en écriture
    fn register_write(&mut self, symbol_id: SymbolId, location: SourceLocation) -> Result<(), SemanticError>;

    /// Enregistre un emprunt immutable
    fn register_immutable_borrow(&mut self, symbol_id: SymbolId, location: SourceLocation) -> Result<(), SemanticError>;

    /// Enregistre un emprunt mutable
    fn register_mutable_borrow(&mut self, symbol_id: SymbolId, location: SourceLocation) -> Result<(), SemanticError>;

    /// Enregistre un move
    fn register_move(&mut self, symbol_id: SymbolId, location: SourceLocation) -> Result<(), SemanticError>;
}


// Ajout des méthodes à SymbolTable pour le borrow checker
impl SymbolTable {
    // Mise à jour des méthodes existantes

    /// Initialise une table des symboles avec le borrow checker
    pub fn with_borrow_checker() -> Self {
        let mut table = SymbolTable::new();
        table.borrow_checker = BorrowChecker::new();
        table
    }

    /// Libère les emprunts d'un scope quand on le quitte
    fn exit_scope_and_release_borrows(&mut self) -> Result<ScopeId, SemanticError> {
        let result = self.exit_scope();

        if let Ok(scope_id) = result {
            // Libérer les emprunts du scope qu'on quitte
            self.borrow_checker.release_borrows_for_scope(scope_id);
            Ok(scope_id)
        } else {
            result
        }
    }

    /// Vérifie si les règles de mutabilité et d'emprunt sont respectées
    pub fn validate_borrows(&self) -> Result<(), Vec<SemanticError>> {
        // Une implémentation plus avancée vérifierait les erreurs de lifetime
        // et autres règles du borrow checker
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::symbols::{SymbolKind, ScopeKind};

    fn create_location(line: usize) -> SourceLocation {
        SourceLocation {
            file: "test.punk".to_string(),
            line,
            column: 1,
        }
    }

    // Note: ces tests sont juste des exemples et dépendent de l'implémentation complète de SymbolTable
    // avec le borrow checker intégré. Ils pourraient nécessiter des ajustements.

    #[test]
    fn test_mutable_variable() {
        let mut table = SymbolTable::with_borrow_checker();
        let location = create_location(1);

        // Déclarer une variable mutable
        let var_id = table.declare_symbol("x".to_string(), SymbolKind::Variable, location.clone()).unwrap();

        // Rendre la variable mutable
        if let Some(symbol) = table.symbols.get_mut(&var_id) {
            symbol.attributes.is_mutable = true;
        }

        // Écrire dans la variable devrait fonctionner
        assert!(table.register_write(var_id, create_location(2)).is_ok());

        // La variable est maintenant initialisée
        assert!(table.is_initialized(var_id).unwrap());

        // Lecture devrait fonctionner (pas d'emprunt persistant pour Write)
        assert!(table.register_read(var_id, create_location(3)).is_ok());
    }

    #[test]
    fn test_immutable_variable() {
        let mut table = SymbolTable::with_borrow_checker();
        let location = create_location(1);

        // Déclarer une variable immutable (par défaut)
        let var_id = table.declare_symbol("x".to_string(), SymbolKind::Variable, location.clone()).unwrap();

        // Initialiser la variable (en mode construction)
        table.mark_initialized(var_id).unwrap();

        // Lire devrait fonctionner
        assert!(table.register_read(var_id, create_location(2)).is_ok());

        // Écrire devrait échouer car immutable
        let write_result = table.register_write(var_id, create_location(3));
        assert!(write_result.is_err());
    }
}