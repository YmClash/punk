//src/semantic/context.rs

use std::rc::Rc;
use std::cell::RefCell;
use crate::semantic::types::type_system::TypeSystem;
use crate::semantic::symbol_table::SymbolTable;
use crate::semantic::semantic_error::{SemanticError, SemanticErrorType, Position};

/// Contexte de compilation partagé entre les différentes phases d'analyse
/// Utilise Rc<RefCell> pour permettre le partage mutable entre composants
#[derive(Clone, Debug)]
pub struct CompilationContext {
    /// Système de types partagé
    pub types: Rc<RefCell<TypeSystem>>,
    
    /// Table des symboles partagée
    pub symbols: Rc<RefCell<SymbolTable>>,
}

impl CompilationContext {
    /// Crée un nouveau contexte de compilation
    pub fn new() -> Self {
        CompilationContext {
            types: Rc::new(RefCell::new(TypeSystem::new())),
            symbols: Rc::new(RefCell::new(SymbolTable::new())),
        }
    }
    
    /// Crée un contexte avec des instances existantes
    pub fn from_existing(types: TypeSystem, symbols: SymbolTable) -> Self {
        CompilationContext {
            types: Rc::new(RefCell::new(types)),
            symbols: Rc::new(RefCell::new(symbols)),
        }
    }
    
    /// Réinitialise le contexte pour une nouvelle compilation
    pub fn reset(&mut self) {
        self.types.borrow_mut().clear();
        self.symbols.borrow_mut().clear();
    }
    
    /// Vérifie la cohérence du contexte
    pub fn validate(&self) -> Result<(), Vec<SemanticError>> {
        let mut errors = Vec::new();
        
        // Vérification de la cohérence des types
        let types = self.types.borrow();
        if let Err(type_err) = types.validate() {
            // Convertir TypeError en SemanticError
            errors.push(SemanticError::new(
                SemanticErrorType::TypeError(type_err),
                "Type system validation error".to_string(),
                Position { index: 0 }
            ));
        }
        
        // Vérification de la cohérence des symboles
        let symbols = self.symbols.borrow();
        let symbol_errors = symbols.get_all_errors();
        errors.extend(symbol_errors);
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Extrait les statistiques du contexte
    pub fn get_statistics(&self) -> ContextStatistics {
        ContextStatistics {
            type_count: self.types.borrow().type_count(),
            symbol_count: self.symbols.borrow().symbol_count(),
            scope_count: self.symbols.borrow().scope_count(),
        }
    }
}

impl Default for CompilationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistiques du contexte de compilation
#[derive(Debug, Clone)]
pub struct ContextStatistics {
    pub type_count: usize,
    pub symbol_count: usize,
    pub scope_count: usize,
}

impl std::fmt::Display for ContextStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Context Statistics: {} types, {} symbols, {} scopes",
            self.type_count, self.symbol_count, self.scope_count
        )
    }
}

/// Builder pour le contexte de compilation
pub struct ContextBuilder {
    types: Option<TypeSystem>,
    symbols: Option<SymbolTable>,
}

impl ContextBuilder {
    pub fn new() -> Self {
        ContextBuilder {
            types: None,
            symbols: None,
        }
    }
    
    pub fn with_types(mut self, types: TypeSystem) -> Self {
        self.types = Some(types);
        self
    }
    
    pub fn with_symbols(mut self, symbols: SymbolTable) -> Self {
        self.symbols = Some(symbols);
        self
    }
    
    pub fn build(self) -> CompilationContext {
        CompilationContext {
            types: Rc::new(RefCell::new(self.types.unwrap_or_else(TypeSystem::new))),
            symbols: Rc::new(RefCell::new(self.symbols.unwrap_or_else(SymbolTable::new))),
        }
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_context_creation() {
        let context = CompilationContext::new();
        let stats = context.get_statistics();
        assert!(stats.type_count > 0); // Les types primitifs sont créés par défaut
        assert_eq!(stats.symbol_count, 0);
        assert_eq!(stats.scope_count, 1); // Scope global
    }
    
    #[test]
    fn test_context_builder() {
        let context = ContextBuilder::new()
            .with_types(TypeSystem::new())
            .with_symbols(SymbolTable::new())
            .build();
        
        assert!(context.validate().is_ok());
    }
    
    #[test]
    fn test_context_sharing() {
        let context = CompilationContext::new();
        let context_clone = context.clone();
        
        // Les deux contextes partagent les mêmes données
        context.symbols.borrow_mut().symbol_count();
        assert_eq!(
            context.get_statistics().symbol_count,
            context_clone.get_statistics().symbol_count
        );
    }
}