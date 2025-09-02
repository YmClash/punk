// tests/semantic_borrow_test.rs
// Tests pour le Borrow Checker basés sur l'API réelle

use punk::semantic::borrow_checker::{
    BorrowChecker, BorrowKind, BorrowErrorKind
};
use punk::semantic::symbols::{SymbolId, ScopeId, SourceLocation};
use punk::semantic::flow::control_flow_graph::ControlFlowGraph;
use std::rc::Rc;
use std::cell::RefCell;

#[cfg(test)]
mod borrow_tests {
    use super::*;

    fn create_test_location(line: u32) -> SourceLocation {
        SourceLocation {
            file: "test.rs".to_string(),
            line: line as usize,
            column: 1,
        }
    }

    #[test]
    fn test_borrow_checker_creation() {
        let checker = BorrowChecker::new();
        
        // Vérifier l'état initial
        assert!(checker.active_borrows.is_empty());
        assert!(checker.initialized_variables.is_empty());
        assert!(checker.moved_variables.is_empty());
        assert!(checker.borrow_history.is_empty());
    }

    #[test]
    fn test_initialization_tracking() {
        let mut checker = BorrowChecker::new();
        let symbol_id = SymbolId(1);
        
        // Au départ, non initialisé
        assert!(!checker.is_initialized(symbol_id));
        
        // Marquer comme initialisé
        checker.mark_initialized(symbol_id);
        
        // Vérifier l'initialisation
        assert!(checker.is_initialized(symbol_id));
    }

    #[test]
    fn test_basic_immutable_borrow() {
        let mut checker = BorrowChecker::new();
        let symbol_id = SymbolId(1);
        let scope_id = ScopeId(1);
        let location = create_test_location(1);
        
        // Initialiser la variable d'abord
        checker.mark_initialized(symbol_id);
        
        // Enregistrer un emprunt immutable
        let result = checker.register_borrow(
            symbol_id,
            BorrowKind::Immutable,
            location.clone(),
            scope_id,
            None
        );
        
        assert!(result.is_ok());
        assert!(checker.has_active_borrows(symbol_id));
        
        // Un deuxième emprunt immutable devrait être OK
        let result2 = checker.register_borrow(
            symbol_id,
            BorrowKind::Immutable,
            create_test_location(2),
            scope_id,
            None
        );
        
        assert!(result2.is_ok());
        assert_eq!(checker.get_active_borrows(symbol_id).len(), 2);
    }

    #[test]
    fn test_mutable_borrow_exclusive() {
        let mut checker = BorrowChecker::new();
        let symbol_id = SymbolId(1);
        let scope_id = ScopeId(1);
        
        // Initialiser la variable
        checker.mark_initialized(symbol_id);
        
        // Enregistrer un emprunt mutable
        let result = checker.register_borrow(
            symbol_id,
            BorrowKind::Mutable,
            create_test_location(1),
            scope_id,
            None
        );
        
        assert!(result.is_ok());
        
        // Un deuxième emprunt mutable devrait échouer
        let result2 = checker.register_borrow(
            symbol_id,
            BorrowKind::Mutable,
            create_test_location(2),
            scope_id,
            None
        );
        
        assert!(result2.is_err());
        match result2.unwrap_err() {
            BorrowErrorKind::MultipleMutableBorrows { .. } => {},
            _ => panic!("Expected MultipleMutableBorrows error"),
        }
    }

    #[test]
    fn test_immutable_then_mutable_conflict() {
        let mut checker = BorrowChecker::new();
        let symbol_id = SymbolId(1);
        let scope_id = ScopeId(1);
        
        checker.mark_initialized(symbol_id);
        
        // D'abord un emprunt immutable
        checker.register_borrow(
            symbol_id,
            BorrowKind::Immutable,
            create_test_location(1),
            scope_id,
            None
        ).unwrap();
        
        // Puis essayer un emprunt mutable - devrait échouer
        let result = checker.register_borrow(
            symbol_id,
            BorrowKind::Mutable,
            create_test_location(2),
            scope_id,
            None
        );
        
        assert!(result.is_err());
        match result.unwrap_err() {
            BorrowErrorKind::MutableBorrowWithImmutableBorrows { .. } => {},
            _ => panic!("Expected MutableBorrowWithImmutableBorrows error"),
        }
    }

    #[test]
    fn test_move_semantics() {
        let mut checker = BorrowChecker::new();
        let symbol_id = SymbolId(1);
        let scope_id = ScopeId(1);
        
        checker.mark_initialized(symbol_id);
        
        // Effectuer un move
        let result = checker.register_borrow(
            symbol_id,
            BorrowKind::Move,
            create_test_location(1),
            scope_id,
            None
        );
        
        assert!(result.is_ok());
        assert!(checker.is_moved(symbol_id));
        
        // Essayer d'utiliser après le move - devrait échouer
        let result2 = checker.register_borrow(
            symbol_id,
            BorrowKind::Read,
            create_test_location(2),
            scope_id,
            None
        );
        
        assert!(result2.is_err());
        match result2.unwrap_err() {
            BorrowErrorKind::UseAfterMove { .. } => {},
            _ => panic!("Expected UseAfterMove error"),
        }
    }

    #[test]
    fn test_uninitialized_use() {
        let mut checker = BorrowChecker::new();
        let symbol_id = SymbolId(1);
        let scope_id = ScopeId(1);
        
        // Essayer de lire une variable non initialisée
        let result = checker.register_borrow(
            symbol_id,
            BorrowKind::Read,
            create_test_location(1),
            scope_id,
            None
        );
        
        assert!(result.is_err());
        match result.unwrap_err() {
            BorrowErrorKind::UninitializedVariable { .. } => {},
            _ => panic!("Expected UninitializedVariable error"),
        }
    }

    #[test]
    fn test_write_initializes() {
        let mut checker = BorrowChecker::new();
        let symbol_id = SymbolId(1);
        let scope_id = ScopeId(1);
        
        // Une écriture devrait initialiser la variable
        let result = checker.register_borrow(
            symbol_id,
            BorrowKind::Write,
            create_test_location(1),
            scope_id,
            None
        );
        
        assert!(result.is_ok());
        assert!(checker.is_initialized(symbol_id));
        
        // Maintenant on peut lire
        let result2 = checker.register_borrow(
            symbol_id,
            BorrowKind::Read,
            create_test_location(2),
            scope_id,
            None
        );
        
        assert!(result2.is_ok());
    }

    #[test]
    fn test_scope_release() {
        let mut checker = BorrowChecker::new();
        let symbol_id = SymbolId(1);
        let scope1 = ScopeId(1);
        let scope2 = ScopeId(2);
        
        checker.mark_initialized(symbol_id);
        
        // Emprunts dans différents scopes
        checker.register_borrow(
            symbol_id,
            BorrowKind::Immutable,
            create_test_location(1),
            scope1,
            None
        ).unwrap();
        
        checker.register_borrow(
            symbol_id,
            BorrowKind::Immutable,
            create_test_location(2),
            scope2,
            None
        ).unwrap();
        
        assert_eq!(checker.get_active_borrows(symbol_id).len(), 2);
        
        // Libérer les emprunts du scope2
        checker.release_borrows_for_scope(scope2);
        
        assert_eq!(checker.get_active_borrows(symbol_id).len(), 1);
        
        // Maintenant on peut faire un emprunt mutable
        let result = checker.register_borrow(
            symbol_id,
            BorrowKind::Mutable,
            create_test_location(3),
            scope2,
            None
        );
        
        // Devrait toujours échouer car scope1 a encore un emprunt
        assert!(result.is_err());
        
        // Libérer scope1
        checker.release_borrows_for_scope(scope1);
        
        // Maintenant ça devrait marcher
        let result2 = checker.register_borrow(
            symbol_id,
            BorrowKind::Mutable,
            create_test_location(4),
            scope2,
            None
        );
        
        assert!(result2.is_ok());
    }

    #[test]
    fn test_borrow_history() {
        let mut checker = BorrowChecker::new();
        let symbol_id = SymbolId(1);
        let scope_id = ScopeId(1);
        
        checker.mark_initialized(symbol_id);
        
        // Plusieurs emprunts
        checker.register_borrow(
            symbol_id,
            BorrowKind::Read,
            create_test_location(1),
            scope_id,
            Some("'a".to_string())
        ).unwrap();
        
        checker.register_borrow(
            symbol_id,
            BorrowKind::Read,
            create_test_location(2),
            scope_id,
            Some("'b".to_string())
        ).unwrap();
        
        // Vérifier l'historique
        assert_eq!(checker.borrow_history.len(), 2);
        assert_eq!(checker.borrow_history[0].lifetime, Some("'a".to_string()));
        assert_eq!(checker.borrow_history[1].lifetime, Some("'b".to_string()));
    }

    #[test]
    fn test_with_cfg() {
        let mut checker = BorrowChecker::new();
        let cfg = Rc::new(RefCell::new(ControlFlowGraph::new()));
        
        // Associer le CFG
        checker = checker.with_cfg(cfg.clone());
        
        // Créer quelques blocs dans le CFG
        {
            let mut cfg_mut = cfg.borrow_mut();
            cfg_mut.create_block();
            cfg_mut.create_block();
        }
        
        // Analyser avec le CFG
        let result = checker.analyze_with_cfg();
        assert!(result.is_ok());
    }

    #[test]
    fn test_block_borrow_state() {
        let mut checker = BorrowChecker::new();
        let cfg = Rc::new(RefCell::new(ControlFlowGraph::new()));
        
        checker = checker.with_cfg(cfg.clone());
        
        let _block_id = {
            let mut cfg_mut = cfg.borrow_mut();
            cfg_mut.create_block()
        };
        
        // Analyser pour initialiser les états de bloc
        // Les états de bloc sont internes au checker, on vérifie juste que l'analyse fonctionne
        let result = checker.analyze_with_cfg();
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_reads() {
        let mut checker = BorrowChecker::new();
        let symbol_id = SymbolId(1);
        let scope_id = ScopeId(1);
        
        checker.mark_initialized(symbol_id);
        
        // Plusieurs lectures simultanées devraient être OK
        for i in 0..5 {
            let result = checker.register_borrow(
                symbol_id,
                BorrowKind::Read,
                create_test_location(i),
                scope_id,
                None
            );
            assert!(result.is_ok());
        }
        
        assert_eq!(checker.get_active_borrows(symbol_id).len(), 5);
    }

    #[test]
    fn test_write_with_active_reads() {
        let mut checker = BorrowChecker::new();
        let symbol_id = SymbolId(1);
        let scope_id = ScopeId(1);
        
        checker.mark_initialized(symbol_id);
        
        // D'abord des lectures
        checker.register_borrow(
            symbol_id,
            BorrowKind::Read,
            create_test_location(1),
            scope_id,
            None
        ).unwrap();
        
        // Puis essayer une écriture - devrait échouer
        let result = checker.register_borrow(
            symbol_id,
            BorrowKind::Write,
            create_test_location(2),
            scope_id,
            None
        );
        
        assert!(result.is_err());
        match result.unwrap_err() {
            BorrowErrorKind::MutableBorrowWithImmutableBorrows { .. } => {},
            _ => panic!("Expected MutableBorrowWithImmutableBorrows error"),
        }
    }

    #[test]
    fn test_move_clears_borrows() {
        let mut checker = BorrowChecker::new();
        let symbol_id = SymbolId(1);
        let scope_id = ScopeId(1);
        
        checker.mark_initialized(symbol_id);
        
        // D'abord quelques emprunts
        checker.register_borrow(
            symbol_id,
            BorrowKind::Read,
            create_test_location(1),
            scope_id,
            None
        ).unwrap();
        
        assert!(checker.has_active_borrows(symbol_id));
        
        // Un move devrait effacer les emprunts
        checker.register_borrow(
            symbol_id,
            BorrowKind::Move,
            create_test_location(2),
            scope_id,
            None
        ).unwrap();
        
        assert!(!checker.has_active_borrows(symbol_id));
        assert!(checker.is_moved(symbol_id));
    }
}