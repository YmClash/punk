// src/semantic/symbols/borrow_checker.rs

use std::collections::{HashMap, HashSet};
use crate::semantic::semantic_error::{SemanticError, SemanticErrorType, Position};
use crate::semantic::symbols::{Symbol, SymbolId, ScopeId, SourceLocation};
use crate::semantic::symbol_table::SymbolTable;

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

/// Gère l'état des emprunts et la mutabilité
pub struct BorrowChecker {
    /// Borrows actifs par symbole
    active_borrows: HashMap<SymbolId, Vec<BorrowInfo>>,

    /// Variables initialisées
    initialized_variables: HashSet<SymbolId>,

    /// Variables moved
    moved_variables: HashMap<SymbolId, SourceLocation>,

    /// Historique des borrows (pour debug et reporting)
    borrow_history: Vec<BorrowInfo>,
}

impl BorrowChecker {
    /// Crée un nouveau borrow checker
    pub fn new() -> Self {
        BorrowChecker {
            active_borrows: HashMap::new(),
            initialized_variables: HashSet::new(),
            moved_variables: HashMap::new(),
            borrow_history: Vec::new(),
        }
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
        // Gérer les autres types d'emprunt
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
                _ => {} // Les emprunts immutables peuvent coexister
            }

            // Ajouter l'emprunt aux emprunts actifs
            self.active_borrows.entry(symbol_id)
                .or_insert_with(Vec::new)
                .push(borrow_info.clone());

            // Enregistrer dans l'historique
            self.borrow_history.push(borrow_info);

            // Si c'est une écriture, marquer comme initialisée
            if matches!(kind, BorrowKind::Write) {
                self.mark_initialized(symbol_id);
            }
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

/// Implémentation du MutabilityManager pour SymbolTable
// impl MutabilityManager for SymbolTable {
//     fn is_mutable(&self, symbol_id: SymbolId) -> Result<bool, SemanticError> {
//         match self.get_symbol(symbol_id) {
//             Ok(symbol) => Ok(symbol.attributes.is_mutable),
//             Err(e) => Err(e),
//         }
//     }
//
//     fn mark_initialized(&mut self, symbol_id: SymbolId) -> Result<(), SemanticError> {
//         if let Some(symbol) = self.symbols.get_mut(&symbol_id) {
//             symbol.attributes.is_initialized = true;
//             Ok(())
//         } else {
//             Err(SemanticError::new(
//                 SemanticErrorType::SymbolError(
//                     crate::semantic::semantic_error::SymbolError::SymbolNotFound(
//                         format!("{:?}", symbol_id)
//                     )
//                 ),
//                 "Symbol not found".to_string(),
//                 Position { index: 0 }
//             ))
//         }
//     }
//
//     fn is_initialized(&self, symbol_id: SymbolId) -> Result<bool, SemanticError> {
//         match self.get_symbol(symbol_id) {
//             Ok(symbol) => Ok(symbol.attributes.is_initialized),
//             Err(e) => Err(e),
//         }
//     }
//
//     fn register_read(&mut self, symbol_id: SymbolId, location: SourceLocation) -> Result<(), SemanticError> {
//         // Vérifier si la variable est initialisée
//         if !self.is_initialized(symbol_id)? {
//             return Err(borrow_error_to_semantic(
//                 BorrowErrorKind::UninitializedVariable {
//                     symbol_id,
//                     use_location: location,
//                 },
//                 Position { index: 0 }
//             ));
//         }
//
//         match self.borrow_checker.register_borrow(
//             symbol_id,
//             BorrowKind::Read,
//             location,
//             self.current_scope,
//             None,
//         ) {
//             Ok(()) => Ok(()),
//             Err(e) => Err(borrow_error_to_semantic(e, Position { index: 0 })),
//         }
//     }
//
//     fn register_write(&mut self, symbol_id: SymbolId, location: SourceLocation) -> Result<(), SemanticError> {
//         // Vérifier si la variable est mutable
//         if !self.is_mutable(symbol_id)? {
//             return Err(borrow_error_to_semantic(
//                 BorrowErrorKind::ModifyImmutableVariable {
//                     symbol_id,
//                     location,
//                 },
//                 Position { index: 0 }
//             ));
//         }
//
//         match self.borrow_checker.register_borrow(
//             symbol_id,
//             BorrowKind::Write,
//             location,
//             self.current_scope,
//             None,
//         ) {
//             Ok(()) => Ok(()),
//             Err(e) => Err(borrow_error_to_semantic(e, Position { index: 0 })),
//         }
//     }
//
//     fn register_immutable_borrow(&mut self, symbol_id: SymbolId, location: SourceLocation) -> Result<(), SemanticError> {
//         match self.borrow_checker.register_borrow(
//             symbol_id,
//             BorrowKind::Immutable,
//             location,
//             self.current_scope,
//             None,
//         ) {
//             Ok(()) => Ok(()),
//             Err(e) => Err(borrow_error_to_semantic(e, Position { index: 0 })),
//         }
//     }
//
//     fn register_mutable_borrow(&mut self, symbol_id: SymbolId, location: SourceLocation) -> Result<(), SemanticError> {
//         // Vérifier si la variable est mutable
//         if !self.is_mutable(symbol_id)? {
//             return Err(borrow_error_to_semantic(
//                 BorrowErrorKind::ModifyImmutableVariable {
//                     symbol_id,
//                     location,
//                 },
//                 Position { index: 0 }
//             ));
//         }
//
//         match self.borrow_checker.register_borrow(
//             symbol_id,
//             BorrowKind::Mutable,
//             location,
//             self.current_scope,
//             None,
//         ) {
//             Ok(()) => Ok(()),
//             Err(e) => Err(borrow_error_to_semantic(e, Position { index: 0 })),
//         }
//     }
//
//     fn register_move(&mut self, symbol_id: SymbolId, location: SourceLocation) -> Result<(), SemanticError> {
//         match self.borrow_checker.register_borrow(
//             symbol_id,
//             BorrowKind::Move,
//             location,
//             self.current_scope,
//             None,
//         ) {
//             Ok(()) => Ok(()),
//             Err(e) => Err(borrow_error_to_semantic(e, Position { index: 0 })),
//         }
//     }
// }

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

        // Lecture devrait fonctionner
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