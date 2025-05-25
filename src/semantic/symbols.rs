//src/semantic/symbols.rs

use std::any::TypeId;
use std::collections::HashMap;


use crate::semantic::semantic_error::{Position, SymbolError};
use crate::semantic::types::type_system::Type;

/// Type pour les identifiants uniques des symboles
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub u32);

/// Type pour les identifiants uniques des scopes
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub u32);

/// Position dans le code source
#[allow(dead_code)]
#[derive(Debug, Clone )]
pub struct SourceLocation{
    pub file: String,
    pub line: usize,
    pub column: usize,
}

/// Type de symbole supporte
#[allow(dead_code)]
#[derive(Debug, Clone , PartialEq)]
pub enum SymbolKind {
    Variable,
    Function,
    Class,
    Type,
    Module,
    Trait,
    Struct,
    Enum,
    Constant,
    Macro,
    Generic,
}

/// Niveau de visibilite du symbole
#[allow(dead_code)]
#[derive(Debug, Clone , PartialEq)]
pub enum Visibility{
    Public,
    Private,
    // Protected,
    // Module,
}

/// Attribus additionnels pour les symboles

#[allow(dead_code)]
#[derive(Debug, Clone )]
pub struct SymbolAttrs {
    pub is_mutable: bool,
    pub is_initialized: bool,
    pub docstring: Option<String>,
    pub type_info: Option<TypeId>,
    pub inferred_type: Option<Type>,
    pub used: bool,

}

impl Default for SymbolAttrs {
    fn default() -> Self {
        SymbolAttrs {
            is_mutable: false,
            is_initialized: false,
            type_info: None,
            inferred_type: None,
            docstring: None,
            used: false,
        }
    }
}

/// Structure principale representant un symbole
#[allow(dead_code)]
#[derive(Debug, Clone )]
pub struct Symbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub scope_id: ScopeId,
    pub visibility: Visibility,
    pub attributes: SymbolAttrs,
    pub location: SourceLocation,
}

impl Symbol {
    // cree un nouveau symbole
    pub fn new(
        id: SymbolId,
        name: String,
        kind: SymbolKind,
        scope_id: ScopeId,
        location: SourceLocation,
    ) -> Self {
        Symbol {
            id,
            name,
            kind,
            scope_id,
            visibility: Visibility::Private, // Par défaut privé
            attributes: SymbolAttrs::default(),
            location,
        }
    }

    //verifie si le symbole est accessible depuis un scope donne
    pub fn is_accessible_from(&self,_from_scope: ScopeId) -> bool {

        //todo!()
        match self.visibility{
            Visibility::Public => true,
            _ => false,
        }
    }
}

/// Un scope est une collection de symboles
#[allow(dead_code)]
#[derive(Debug, Clone,PartialEq)]
pub enum ScopeKind {
    Global,
    Module,
    Function,
    Block,
    Loop,
    Trait,
    Struct,
    Implementation,
}


/// Structure  pour les Imports
#[allow(dead_code)]
#[derive(Debug, Clone )]
pub struct Import {
    pub source_module: String,
    pub imported_symbols: Vec<ImportedSymbol>,
    pub visibility: Visibility,
    pub is_glob: bool,
}

#[derive(Debug, Clone)]
pub struct ImportedSymbol {
    pub original_name: String,
    pub alias: Option<String>,
    pub symbol_id: Option<SymbolId>, // Sera résolu plus tard
}

/// Structure pour les symboles represantant un scope
#[allow(dead_code)]
#[derive(Debug, Clone )]
pub struct Scope{
    pub id: ScopeId,
    pub kind: ScopeKind,
    pub parent: Option<ScopeId>,
    pub children: Vec<ScopeId>,
    pub symbols: HashMap<String, SymbolId>,
    pub imports: Vec<Import>,
    pub level: usize,
}
impl Scope{
    pub fn new(id:ScopeId, kind:ScopeKind, parent:Option<ScopeId>, level:usize) -> Self{
        Scope{
            id,
            kind,
            parent,
            children: Vec::new(),
            symbols: HashMap::new(),
            imports: Vec::new(),
            level,
        }
    }

    //utilitaire

    /// Ajoute un symbole au scope
    pub fn add_symbol(&mut self, name: String, symbol_id: SymbolId) -> Result<(), SymbolError> {
        if self.symbols.contains_key(&name) {
            return Err(SymbolError::SymbolAlreadyDeclared(name));
        }
        self.symbols.insert(name, symbol_id);
        Ok(())
    }

    /// Recherche un symbole dans ce scope uniquement
    pub fn lookup_symbol(&self, name: &str) -> Option<SymbolId> {
        self.symbols.get(name).copied()
    }

    // ajouter un scope enfant
    pub fn add_child(&mut self, child_id: ScopeId) {
        self.children.push(child_id);
    }

    pub fn  current_position(&self) -> Position {
        Position{
            index: 0,

        }
    }

}


// Tests unitaires
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_creation() {
        let location = SourceLocation {
            file: "test.punk".to_string(),
            line: 1,
            column: 1,
        };

        let symbol = Symbol::new(
            SymbolId(1),
            "test_var".to_string(),
            SymbolKind::Variable,
            ScopeId(1),
            location,
        );

        assert_eq!(symbol.name, "test_var");
        assert_eq!(symbol.kind, SymbolKind::Variable);
        assert!(!symbol.attributes.is_mutable);
        assert!(!symbol.attributes.is_initialized);
    }

    #[test]
    fn test_scope_symbol_management() {
        let mut scope = Scope::new(ScopeId(1), ScopeKind::Block, Some(ScopeId(0)), 1);

        // Test d'ajout de symbole
        assert!(scope.add_symbol("x".to_string(), SymbolId(1)).is_ok());

        // Test de recherche de symbole
        assert_eq!(scope.lookup_symbol("x"), Some(SymbolId(1)));
        assert_eq!(scope.lookup_symbol("y"), None);

        // Test de symbole déjà déclaré
        assert!(scope.add_symbol("x".to_string(), SymbolId(2)).is_err());
    }

    #[test]
    fn test_scope_hierarchy() {
        let mut scope = Scope::new(ScopeId(1), ScopeKind::Function, Some(ScopeId(0)), 1);

        scope.add_child(ScopeId(2));
        scope.add_child(ScopeId(3));

        assert_eq!(scope.children.len(), 2);
        assert_eq!(scope.parent, Some(ScopeId(0)));
    }

}