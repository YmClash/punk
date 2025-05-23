//src/semantic/symbol_table.rs - Intégration avec le système de types

use std::collections::HashMap;
use crate::semantic::semantic_error::{SemanticError, SemanticErrorType, SymbolError, TypeError, Position};
use crate::semantic::symbols::{Scope, ScopeId, ScopeKind, SourceLocation, Symbol, SymbolId, SymbolKind};
use crate::semantic::borrow_checker::{BorrowChecker, BorrowKind, MutabilityManager};
use crate::semantic::types::type_system::{TypeId, TypeSystem, Type};

/// Structure Principale de la table des symboles avec système de types intégré
#[derive(Debug,Clone)]
pub struct SymbolTable {
     /// Scopes indexés par leur ID
     pub scopes: HashMap<ScopeId, Scope>,
     pub symbols: HashMap<SymbolId, Symbol>,
     pub current_scope: ScopeId,
     pub next_symbols_id: u32,
     pub next_scope_id: u32,

     /// Système de types intégré
     pub type_system: TypeSystem,

     /// Borrow checker
     pub borrow_checker: BorrowChecker,
}

impl SymbolTable {
     /// Crée une nouvelle table des symboles avec système de types
     pub fn new() -> Self {
          let mut table = SymbolTable {
               scopes: HashMap::new(),
               symbols: HashMap::new(),
               current_scope: ScopeId(0),
               next_symbols_id: 1,
               next_scope_id: 1,
               type_system: TypeSystem::new(),
               borrow_checker: BorrowChecker::new(),
          };

          // Crée le scope global
          let global_scope_id = ScopeId(0);
          let global_scope = Scope::new(global_scope_id, ScopeKind::Global, None, 0);
          table.scopes.insert(global_scope_id, global_scope);

          table
     }

     /// Récupère un symbole par son ID
     pub fn get_symbol(&self, id: SymbolId) -> Result<&Symbol, SemanticError> {
          self.symbols.get(&id)
              .ok_or_else(|| create_symbol_error(
                   SymbolError::SymbolNotFound(format!("{:?}", id)),
                   Position { index: 0 }
              ))
     }

     /// Récupère un symbole mutable par son ID
     pub fn get_symbol_mut(&mut self, id: SymbolId) -> Option<&mut Symbol> {
          self.symbols.get_mut(&id)
     }

     /// Génère un nouvel ID de symbole unique
     fn next_symbol_id(&mut self) -> SymbolId {
          let id = self.next_symbols_id;
          self.next_symbols_id += 1;
          SymbolId(id)
     }

     /// Génère un nouvel ID de scope unique
     fn next_scope_id(&mut self) -> ScopeId {
          let id = self.next_scope_id;
          self.next_scope_id += 1;
          ScopeId(id)
     }

     /// Entre dans un nouveau scope
     pub fn enter_scope(&mut self, kind: ScopeKind) -> ScopeId {
          let parent_id = self.current_scope;
          let parent_level = self.scopes.get(&parent_id).unwrap().level;

          let scope_id = self.next_scope_id();
          let scope = Scope::new(scope_id, kind, Some(parent_id), parent_level + 1);

          // Ajoute le nouveau scope comme enfant du scope parent
          if let Some(parent) = self.scopes.get_mut(&parent_id) {
               parent.add_child(scope_id);
          }

          self.scopes.insert(scope_id, scope);
          self.current_scope = scope_id;

          scope_id
     }

     /// Sort du scope actuel et revient au scope parent
     pub fn exit_scope(&mut self) -> Result<ScopeId, SemanticError> {
          let current = self.current_scope;
          let parent_id = match self.scopes.get(&current) {
               Some(scope) => scope.parent,
               None => return Err(create_symbol_error(
                    SymbolError::InvalidScope,
                    Position { index: 0 }
               )),
          };

          match parent_id {
               Some(id) => {
                    self.borrow_checker.release_borrows_for_scope(current);
                    self.current_scope = id;
                    Ok(id)
               },
               None => Err(create_symbol_error(
                    SymbolError::InvalidScope,
                    Position { index: 0 }
               )),
          }
     }

     /// Récupère le scope actuel
     pub fn get_current_scope(&self) -> Result<&Scope, SemanticError> {
          self.scopes.get(&self.current_scope)
              .ok_or_else(|| create_symbol_error(
                   SymbolError::InvalidScope,
                   Position { index: 0 }
              ))
     }

     /// Récupère un scope par son ID
     pub fn get_scope(&self, id: ScopeId) -> Result<&Scope, SemanticError> {
          self.scopes.get(&id)
              .ok_or_else(|| create_symbol_error(
                   SymbolError::InvalidScope,
                   Position { index: 0 }
              ))
     }

     /// Déclare un nouveau symbole dans le scope actuel
     pub fn declare_symbol(
          &mut self,
          name: String,
          kind: SymbolKind,
          location: SourceLocation
     ) -> Result<SymbolId, SemanticError> {
          // Vérifier si le symbole existe déjà dans le scope actuel
          if let Ok(scope) = self.get_current_scope() {
               if scope.lookup_symbol(&name).is_some() {
                    return Err(create_symbol_error(
                         SymbolError::SymbolAlreadyDeclared(name),
                         Position { index: 0 }
                    ));
               }
          }

          let symbol_id = self.next_symbol_id();
          let symbol = Symbol::new(
               symbol_id,
               name.clone(),
               kind,
               self.current_scope,
               location,
          );

          self.symbols.insert(symbol_id, symbol);

          // Ajouter le symbole au scope actuel
          if let Some(scope) = self.scopes.get_mut(&self.current_scope) {
               scope.add_symbol(name, symbol_id)
                   .map_err(|err| create_symbol_error(err, Position { index: 0 }))?;
          }

          Ok(symbol_id)
     }

     /// Déclare un symbole avec un type spécifique
     pub fn declare_symbol_with_type(
          &mut self,
          name: String,
          kind: SymbolKind,
          type_id: TypeId,
          location: SourceLocation,
          is_mutable: bool
     ) -> Result<SymbolId, SemanticError> {
          let symbol_id = self.declare_symbol(name, kind, location)?;

          // Associer le type au symbole
          self.set_symbol_type(symbol_id, type_id)?;

          // Définir la mutabilité si c'est une variable
          if let Some(symbol) = self.symbols.get_mut(&symbol_id) {
               symbol.attributes.is_mutable = is_mutable;
          }

          Ok(symbol_id)
     }

     /// Définit le type d'un symbole
     pub fn set_symbol_type(&mut self, symbol_id: SymbolId, type_id: TypeId) -> Result<(), SemanticError> {
          if let Some(symbol) = self.symbols.get_mut(&symbol_id) {
               // Récupérer le type depuis le système de types
               let type_obj = self.type_system.type_registry.get_type(type_id)
                   .ok_or_else(|| create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", type_id))),
                        "Type not found".to_string(),
                        Position { index: 0 }
                   ))?;

               // Mettre à jour le type du symbole
               symbol.attributes.inferred_type = Some(type_obj.clone());
               Ok(())
          } else {
               Err(create_symbol_error(
                    SymbolError::SymbolNotFound(format!("{:?}", symbol_id)),
                    Position { index: 0 }
               ))
          }
     }

     /// Récupère le type d'un symbole
     pub fn get_symbol_type(&self, symbol_id: SymbolId) -> Result<Option<&Type>, SemanticError> {
          let symbol = self.get_symbol(symbol_id)?;
          Ok(symbol.attributes.inferred_type.as_ref())
     }

     /// Récupère l'ID du type d'un symbole
     pub fn get_symbol_type_id(&self, symbol_id: SymbolId) -> Result<Option<TypeId>, SemanticError> {
          let symbol = self.get_symbol(symbol_id)?;
          Ok(symbol.attributes.inferred_type.as_ref().map(|t| t.id))
     }

     /// Recherche un symbole dans le scope actuel et ses parents
     pub fn lookup_symbol(&self, name: &str) -> Result<SymbolId, SemanticError> {
          let mut current_scope_id = self.current_scope;

          loop {
               // Vérifier dans le scope actuel
               if let Some(scope) = self.scopes.get(&current_scope_id) {
                    if let Some(symbol_id) = scope.lookup_symbol(name) {
                         return Ok(symbol_id);
                    }

                    // Remonter au scope parent
                    match scope.parent {
                         Some(parent_id) => {
                              current_scope_id = parent_id;
                         },
                         None => break,
                    }
               } else {
                    break;
               }
          }

          Err(create_symbol_error(
               SymbolError::SymbolNotFound(name.to_string()),
               Position { index: 0 }
          ))
     }

     /// Recherche un symbole dans un scope spécifique uniquement
     pub fn lookup_symbol_in_scope(&self, name: &str, scope_id: ScopeId) -> Result<SymbolId, SemanticError> {
          if let Some(scope) = self.scopes.get(&scope_id) {
               if let Some(symbol_id) = scope.lookup_symbol(name) {
                    return Ok(symbol_id);
               }
          }
          Err(create_symbol_error(
               SymbolError::SymbolNotFound(name.to_string()),
               Position { index: 0 }
          ))
     }

     /// Résout un chemin qualifié (par exemple, module::sous_module::nom)
     pub fn resolve_qualified_name(&self, path: &[String]) -> Result<SymbolId, SemanticError> {
          if path.is_empty() {
               return Err(create_symbol_error(
                    SymbolError::InvalidScope,
                    Position { index: 0 }
               ));
          }

          // Si le chemin a une seule partie, c'est un lookup simple
          if path.len() == 1 {
               return self.lookup_symbol(&path[0]);
          }

          // Sinon, on parcourt le chemin
          let mut current_scope_id = self.current_scope;
          let mut current_symbol_id = None;

          for (i, part) in path.iter().enumerate() {
               if i == 0 {
                    // Première partie du chemin
                    match self.lookup_symbol(part) {
                         Ok(symbol_id) => {
                              let symbol = self.get_symbol(symbol_id)?;

                              // Vérifier que c'est un module ou un type
                              match symbol.kind {
                                   SymbolKind::Module | SymbolKind::Class | SymbolKind::Trait | SymbolKind::Enum => {
                                        current_symbol_id = Some(symbol_id);

                                        // Pour un module, changer le scope courant
                                        if let SymbolKind::Module = symbol.kind {
                                             // Trouver le scope associé au module
                                             for (scope_id, scope) in &self.scopes {
                                                  if scope.symbols.get(part) == Some(&symbol_id) {
                                                       current_scope_id = *scope_id;
                                                       break;
                                                  }
                                             }
                                        }
                                   },
                                   _ => return Err(create_symbol_error(
                                        SymbolError::InvalidScope,
                                        Position { index: 0 }
                                   )),
                              }
                         },
                         Err(e) => return Err(e),
                    }
               } else {
                    // Parties suivantes du chemin
                    if let Some(scope) = self.scopes.get(&current_scope_id) {
                         match scope.lookup_symbol(part) {
                              Some(symbol_id) => {
                                   current_symbol_id = Some(symbol_id);

                                   // Mettre à jour le scope si nécessaire
                                   let symbol = self.get_symbol(symbol_id)?;
                                   if let SymbolKind::Module = symbol.kind {
                                        // Trouver le scope du module
                                        for (scope_id, scope) in &self.scopes {
                                             if scope.symbols.get(part) == Some(&symbol_id) {
                                                  current_scope_id = *scope_id;
                                                  break;
                                             }
                                        }
                                   }
                              },
                              None => return Err(create_symbol_error(
                                   SymbolError::SymbolNotFound(part.clone()),
                                   Position { index: 0 }
                              )),
                         }
                    } else {
                         return Err(create_symbol_error(
                              SymbolError::InvalidScope,
                              Position { index: 0 }
                         ));
                    }
               }
          }

          match current_symbol_id {
               Some(id) => Ok(id),
               None => Err(create_symbol_error(
                    SymbolError::SymbolNotFound("Path resolution failed".to_string()),
                    Position { index: 0 }
               )),
          }
     }

     /// Vérifie si un type peut être assigné à un autre
     pub fn can_assign(&self, from_type_id: TypeId, to_type_id: TypeId) -> bool {
          if let (Some(from_type), Some(to_type)) = (
               self.type_system.type_registry.get_type(from_type_id),
               self.type_system.type_registry.get_type(to_type_id)
          ) {
               from_type.is_compatible_with(to_type)
          } else {
               false
          }
     }

     /// Trouve le type commun entre deux types (pour les unions, etc.)
     pub fn find_common_type(&self, type1_id: TypeId, type2_id: TypeId) -> Option<TypeId> {
          if type1_id == type2_id {
               return Some(type1_id);
          }

          if let (Some(type1), Some(type2)) = (
               self.type_system.type_registry.get_type(type1_id),
               self.type_system.type_registry.get_type(type2_id)
          ) {
               // Règles de promotion de types
               match (&type1.kind, &type2.kind) {
                    // Int vers Float
                    (crate::semantic::types::type_system::TypeKind::Int, crate::semantic::types::type_system::TypeKind::Float) => Some(type2_id),
                    (crate::semantic::types::type_system::TypeKind::Float, crate::semantic::types::type_system::TypeKind::Int) => Some(type1_id),

                    // Pour d'autres cas, pas de type commun pour l'instant
                    _ => None,
               }
          } else {
               None
          }
     }

     /// Valide les références et les types
     pub fn validate_references(&self) -> Result<(), Vec<SemanticError>> {
          let mut errors = Vec::new();

          // Vérifier que tous les symboles utilisés ont des types valides
          for (symbol_id, symbol) in &self.symbols {
               if let Some(type_obj) = &symbol.attributes.inferred_type {
                    // Vérifier que le type existe dans le registre
                    if self.type_system.type_registry.get_type(type_obj.id).is_none() {
                         errors.push(create_semantic_error(
                              SemanticErrorType::TypeError(TypeError::TypeNotFound(
                                   format!("Type {:?} for symbol {:?}", type_obj.id, symbol_id)
                              )),
                              "Invalid type reference".to_string(),
                              Position { index: 0 }
                         ));
                    }
               }
          }

          if errors.is_empty() {
               Ok(())
          } else {
               Err(errors)
          }
     }

     /// Vérifie les symboles non utilisés
     pub fn check_unused_symbols(&self) -> Vec<SymbolId> {
          self.symbols.iter()
              .filter(|(_, symbol)| !symbol.attributes.used)
              .map(|(symbol_id, _)| *symbol_id)
              .collect()
     }

     /// Marque un symbole comme utilisé
     pub fn mark_symbol_used(&mut self, symbol_id: SymbolId) -> Result<(), SemanticError> {
          if let Some(symbol) = self.symbols.get_mut(&symbol_id) {
               symbol.attributes.used = true;
               Ok(())
          } else {
               Err(create_symbol_error(
                    SymbolError::SymbolNotFound(format!("{:?}", symbol_id)),
                    Position { index: 0 }
               ))
          }
     }

     /// Accès direct au système de types
     pub fn type_registry(&self) -> &crate::semantic::types::type_system::TypeRegistry {
          &self.type_system.type_registry
     }

     /// Accès mutable au système de types
     pub fn type_registry_mut(&mut self) -> &mut crate::semantic::types::type_system::TypeRegistry {
          &mut self.type_system.type_registry
     }

     /// Accès au système de types complet
     pub fn type_system(&self) -> &TypeSystem {
          &self.type_system
     }

     /// Accès mutable au système de types complet
     pub fn type_system_mut(&mut self) -> &mut TypeSystem {
          &mut self.type_system
     }
}

// Implémentation de MutabilityManager pour SymbolTable
impl MutabilityManager for SymbolTable {
     fn is_mutable(&self, symbol_id: SymbolId) -> Result<bool, SemanticError> {
          match self.get_symbol(symbol_id) {
               Ok(symbol) => Ok(symbol.attributes.is_mutable),
               Err(e) => Err(e),
          }
     }

     fn mark_initialized(&mut self, symbol_id: SymbolId) -> Result<(), SemanticError> {
          if let Some(symbol) = self.symbols.get_mut(&symbol_id) {
               symbol.attributes.is_initialized = true;
               // Également marquer dans le borrow checker
               self.borrow_checker.mark_initialized(symbol_id);
               Ok(())
          } else {
               Err(create_symbol_error(
                    SymbolError::SymbolNotFound(format!("{:?}", symbol_id)),
                    Position { index: 0 }
               ))
          }
     }

     fn is_initialized(&self, symbol_id: SymbolId) -> Result<bool, SemanticError> {
          // Vérifier d'abord dans le borrow checker
          if self.borrow_checker.is_initialized(symbol_id) {
               return Ok(true);
          }

          // Sinon vérifier dans le symbole
          match self.get_symbol(symbol_id) {
               Ok(symbol) => Ok(symbol.attributes.is_initialized),
               Err(e) => Err(e),
          }
     }

     fn register_read(&mut self, symbol_id: SymbolId, location: SourceLocation) -> Result<(), SemanticError> {
          // Vérifier si la variable existe
          self.get_symbol(symbol_id)?;

          // Marquer comme utilisé
          self.mark_symbol_used(symbol_id)?;

          // Vérifier si la variable est initialisée
          if !self.is_initialized(symbol_id)? {
               return Err(create_semantic_error(
                    SemanticErrorType::TypeError(TypeError::InvalidType(
                         format!("Variable {:?} is not initialized", symbol_id)
                    )),
                    "Use of uninitialized variable".to_string(),
                    Position { index: 0 }
               ));
          }

          // Enregistrer l'emprunt de lecture
          match self.borrow_checker.register_borrow(
               symbol_id,
               BorrowKind::Read,
               location,
               self.current_scope,
               None,
          ) {
               Ok(()) => Ok(()),
               Err(e) => {
                    // Convertir l'erreur de borrow en erreur sémantique
                    Err(create_semantic_error(
                         SemanticErrorType::TypeError(TypeError::TypeMismatch(
                              format!("Borrow checker error: {:?}", e)
                         )),
                         "Borrow checker error".to_string(),
                         Position { index: 0 }
                    ))
               }
          }
     }

     fn register_write(&mut self, symbol_id: SymbolId, location: SourceLocation) -> Result<(), SemanticError> {
          // Vérifier si la variable est mutable (clone les infos nécessaires)
          let is_mutable;
          let symbol_name;
          {
               let symbol = self.get_symbol(symbol_id)?;
               is_mutable = symbol.attributes.is_mutable;
               symbol_name = symbol.name.clone();
          }

          // Marquer comme utilisé
          self.mark_symbol_used(symbol_id)?;

          // Vérifier la mutabilité
          if !is_mutable {
               return Err(create_semantic_error(
                    SemanticErrorType::TypeError(TypeError::InvalidType(
                         format!("Cannot modify immutable variable {}", symbol_name)
                    )),
                    "Attempt to modify immutable value".to_string(),
                    Position { index: 0 }
               ));
          }

          // Enregistrer l'emprunt d'écriture
          match self.borrow_checker.register_borrow(
               symbol_id,
               BorrowKind::Write,
               location,
               self.current_scope,
               None,
          ) {
               Ok(()) => {
                    // Marquer comme initialisé
                    if let Some(symbol) = self.symbols.get_mut(&symbol_id) {
                         symbol.attributes.is_initialized = true;
                    }
                    Ok(())
               },
               Err(e) => {
                    // Convertir l'erreur de borrow en erreur sémantique
                    Err(create_semantic_error(
                         SemanticErrorType::TypeError(TypeError::TypeMismatch(
                              format!("Borrow checker error: {:?}", e)
                         )),
                         "Borrow checker error".to_string(),
                         Position { index: 0 }
                    ))
               }
          }
     }

     fn register_immutable_borrow(&mut self, symbol_id: SymbolId, location: SourceLocation) -> Result<(), SemanticError> {
          // Vérifier si la variable existe
          self.get_symbol(symbol_id)?;

          // Marquer comme utilisé
          self.mark_symbol_used(symbol_id)?;

          // Vérifier si la variable est initialisée
          if !self.is_initialized(symbol_id)? {
               return Err(create_semantic_error(
                    SemanticErrorType::TypeError(TypeError::InvalidType(
                         format!("Cannot borrow uninitialized variable {:?}", symbol_id)
                    )),
                    "Borrow of uninitialized variable".to_string(),
                    Position { index: 0 }
               ));
          }

          // Enregistrer l'emprunt immutable
          match self.borrow_checker.register_borrow(
               symbol_id,
               BorrowKind::Immutable,
               location,
               self.current_scope,
               None,
          ) {
               Ok(()) => Ok(()),
               Err(e) => {
                    // Convertir l'erreur de borrow en erreur sémantique
                    Err(create_semantic_error(
                         SemanticErrorType::TypeError(TypeError::TypeMismatch(
                              format!("Borrow checker error: {:?}", e)
                         )),
                         "Borrow checker error".to_string(),
                         Position { index: 0 }
                    ))
               }
          }
     }

     fn register_mutable_borrow(&mut self, symbol_id: SymbolId, location: SourceLocation) -> Result<(), SemanticError> {
          // Vérifier si la variable est mutable (clone les infos nécessaires)
          let is_mutable;
          let is_initialized;
          let symbol_name;
          {
               let symbol = self.get_symbol(symbol_id)?;
               is_mutable = symbol.attributes.is_mutable;
               is_initialized = symbol.attributes.is_initialized;
               symbol_name = symbol.name.clone();
          }

          // Marquer comme utilisé
          self.mark_symbol_used(symbol_id)?;

          // Vérifier si la variable est mutable
          if !is_mutable {
               return Err(create_semantic_error(
                    SemanticErrorType::TypeError(TypeError::InvalidType(
                         format!("Cannot mutably borrow immutable variable {}", symbol_name)
                    )),
                    "Mutable borrow of immutable value".to_string(),
                    Position { index: 0 }
               ));
          }

          // Vérifier si la variable est initialisée
          if !is_initialized && !self.borrow_checker.is_initialized(symbol_id) {
               return Err(create_semantic_error(
                    SemanticErrorType::TypeError(TypeError::InvalidType(
                         format!("Cannot borrow uninitialized variable {}", symbol_name)
                    )),
                    "Borrow of uninitialized variable".to_string(),
                    Position { index: 0 }
               ));
          }

          // Enregistrer l'emprunt mutable
          match self.borrow_checker.register_borrow(
               symbol_id,
               BorrowKind::Mutable,
               location,
               self.current_scope,
               None,
          ) {
               Ok(()) => Ok(()),
               Err(e) => {
                    // Convertir l'erreur de borrow en erreur sémantique
                    Err(create_semantic_error(
                         SemanticErrorType::TypeError(TypeError::TypeMismatch(
                              format!("Borrow checker error: {:?}", e)
                         )),
                         "Borrow checker error".to_string(),
                         Position { index: 0 }
                    ))
               }
          }
     }

     fn register_move(&mut self, symbol_id: SymbolId, location: SourceLocation) -> Result<(), SemanticError> {
          // Vérifier si la variable existe
          self.get_symbol(symbol_id)?;

          // Marquer comme utilisé
          self.mark_symbol_used(symbol_id)?;

          // Vérifier si la variable est initialisée
          if !self.is_initialized(symbol_id)? {
               return Err(create_semantic_error(
                    SemanticErrorType::TypeError(TypeError::InvalidType(
                         format!("Cannot move uninitialized variable {:?}", symbol_id)
                    )),
                    "Move of uninitialized variable".to_string(),
                    Position { index: 0 }
               ));
          }

          // Enregistrer le move
          match self.borrow_checker.register_borrow(
               symbol_id,
               BorrowKind::Move,
               location,
               self.current_scope,
               None,
          ) {
               Ok(()) => Ok(()),
               Err(e) => {
                    // Convertir l'erreur de borrow en erreur sémantique
                    Err(create_semantic_error(
                         SemanticErrorType::TypeError(TypeError::TypeMismatch(
                              format!("Borrow checker error: {:?}", e)
                         )),
                         "Borrow checker error".to_string(),
                         Position { index: 0 }
                    ))
               }
          }
     }
}

// Fonctions utilitaires pour créer des erreurs
fn create_symbol_error(error: SymbolError, position: Position) -> SemanticError {
     SemanticError::new(
          SemanticErrorType::SymbolError(error),
          "Symbol error".to_string(),
          position
     )
}

fn create_semantic_error(error_type: SemanticErrorType, message: String, position: Position) -> SemanticError {
     SemanticError::new(
          error_type,
          message,
          position
     )
}



//////////////////////////////////////////////////////////////////////////////////////////////
// // src/semantic/symbol_table.rs
//
// use std::collections::HashMap;
// use crate::semantic::semantic_error::{SemanticError, SemanticErrorType, SymbolError, Position};
// use crate::semantic::symbols::{Scope, ScopeId, ScopeKind, SourceLocation, Symbol, SymbolId, SymbolKind};
// use crate::semantic::borrow_checker::{BorrowChecker,BorrowKind,MutabilityManager};
// use crate::semantic::semantic_error::TypeError::{InvalidType, TypeMismatch};
// // use crate::semantic::types::type_system::{TypeId, TypeKind, TypeRegistry};
//
// // use crate::semantic::semantic_error::{SemanticError, SemanticErrorType, Position};
// // use crate::semantic::symbols::SymbolError;
//
// /// Structure Principale de la table des symboles
// #[allow(dead_code)]
// pub struct SymbolTable {
//      /// Scope Indexés par leur ID
//      pub scopes: HashMap<ScopeId, Scope>,
//      pub symbols: HashMap<SymbolId, Symbol>,
//      // pub type_registry: TypeRegistry,
//      pub current_scope: ScopeId,
//      pub next_symbols_id: u32,
//      pub next_scope_id: u32,
//
//      pub borrow_checker: BorrowChecker,
// }
//
// impl SymbolTable {
//      /// Crée une nouvelle table des symboles
//      pub fn new() -> Self {
//           let mut table = SymbolTable {
//                scopes: HashMap::new(),
//                symbols: HashMap::new(),
//                current_scope: ScopeId(0),
//                 // type_registry: TypeRegistry::new(),
//                next_symbols_id: 1,          // Commence à 1 pour éviter l'ID 0 nuls
//                next_scope_id: 1,           // Commence à 1 pour éviter l'ID 0 nuls
//                borrow_checker: BorrowChecker::new(),
//           };
//
//           // Crée le scope global
//           let global_scope_id = ScopeId(0);
//           let global_scope = Scope::new(global_scope_id, ScopeKind::Global, None, 0);
//           table.scopes.insert(global_scope_id, global_scope);
//
//           table
//      }
//
//      ///Fonction Utilitaire pour ajouter un symbole
//
//      /// Recupere un symbole par son ID
//      // pub fn get_symbol(&self, id: SymbolId) -> Result<&Symbol, SemanticError> {
//      //      self.symbols.get(&id)
//      //          .ok_or_else(|| create_symbol_error(
//      //               SymbolError::SymbolNotFound("Unknown symbol ID".to_string()),
//      //               Position { index: 0 } // Position par défaut
//      //          ))
//      // }
//      /// Recupere un symbole par son ID
//      pub fn get_symbol(&self, id: SymbolId) -> Result<&Symbol, SemanticError> {
//           self.symbols.get(&id)
//               .ok_or_else(|| create_symbol_error(
//                    SymbolError::SymbolNotFound(format!("{:?}", id)),
//                    Position { index: 0 } // Position par défaut
//               ))
//      }
//
//      /// Récupère un symbole mutable par son ID
//      pub fn get_symbol_mut(&mut self, id: SymbolId) -> Option<&mut Symbol> {
//           self.symbols.get_mut(&id)
//      }
//
//      /// genere un nouvelle ID de symbole unique
//      fn next_symbol_id(&mut self) -> SymbolId {
//           let id = self.next_symbols_id;
//           self.next_symbols_id += 1;
//           SymbolId(id)
//      }
//
//      ///genere un noubel ID de scope unique
//      fn next_scope_id(&mut self) -> ScopeId {
//           let id = self.next_scope_id;
//           self.next_scope_id += 1;
//           ScopeId(id)
//      }
//
//      /// Entre dans un nouveau scope
//      pub fn enter_scope(&mut self, kind: ScopeKind) -> ScopeId {
//           let parent_id = self.current_scope;
//           let parent_level = self.scopes.get(&parent_id).unwrap().level;
//
//           let scope_id = self.next_scope_id();
//           let scope = Scope::new(scope_id, kind, Some(parent_id), parent_level + 1);
//
//           // Ajoute le nouveau scope comme enfant du scope parent
//           if let Some(parent) = self.scopes.get_mut(&parent_id) {
//                parent.add_child(scope_id);
//           }
//
//           self.scopes.insert(scope_id, scope);
//           self.current_scope = scope_id;
//
//           scope_id
//      }
//
//      /// Sort du scope actuel et revient au scope parent
//      pub fn exit_scope(&mut self) -> Result<ScopeId, SemanticError> {
//           let current = self.current_scope;
//           let parent_id = match self.scopes.get(&current) {
//                Some(scope) => scope.parent,
//                None => return Err(create_symbol_error(
//                     SymbolError::InvalidScope,
//                     Position { index: 0 }
//                )),
//           };
//
//           match parent_id {
//                Some(id) => {
//                     self.borrow_checker.release_borrows_for_scope(current);
//                     self.current_scope = id;
//                     Ok(id)
//                },
//                None => Err(create_symbol_error(
//                     SymbolError::InvalidScope,
//                     Position { index: 0 }
//                )), // Ne peut pas sortir du scope global
//           }
//      }
//
//      /// Récupère le scope actuel
//      pub fn get_current_scope(&self) -> Result<&Scope, SemanticError> {
//           self.scopes.get(&self.current_scope)
//               .ok_or_else(|| create_symbol_error(
//                    SymbolError::InvalidScope,
//                    Position { index: 0 }
//               ))
//      }
//
//      /// Recupere un scope par son ID
//      pub fn get_scope(&self, id: ScopeId) -> Result<&Scope, SemanticError> {
//           self.scopes.get(&id)
//               .ok_or_else(|| create_symbol_error(
//                    SymbolError::InvalidScope,
//                    Position { index: 0 }
//               ))
//      }
//
//      /// Déclare un nouveau symbole dans le scope actuel
//      pub fn declare_symbol(
//           &mut self,
//           name: String,
//           kind: SymbolKind,
//           location: SourceLocation
//      ) -> Result<SymbolId, SemanticError> {
//           // Vérifier si le symbole existe déjà dans le scope actuel
//           if let Ok(scope) = self.get_current_scope() {
//                if scope.lookup_symbol(&name).is_some() {
//                     return Err(create_symbol_error(
//                          SymbolError::SymbolAlreadyDeclared(name),
//                          Position { index: 0 }
//                     ));
//                }
//           }
//
//           let symbol_id = self.next_symbol_id();
//           let symbol = Symbol::new(
//                symbol_id,
//                name.clone(),
//                kind,
//                self.current_scope,
//                location,
//           );
//
//           self.symbols.insert(symbol_id, symbol);
//
//           // Ajouter le symbole au scope actuel
//           if let Some(scope) = self.scopes.get_mut(&self.current_scope) {
//                scope.add_symbol(name, symbol_id)
//                    .map_err(|err| create_symbol_error(err, Position { index: 0 }))?;
//           }
//
//           Ok(symbol_id)
//      }
//
//      /// Recherche un symbole dans le scope actuel et ses parents
//      pub fn lookup_symbol(&self, name: &str) -> Result<SymbolId, SemanticError> {
//           let mut current_scope_id = self.current_scope;
//
//           loop {
//                // Vérifier dans le scope actuel
//                if let Some(scope) = self.scopes.get(&current_scope_id) {
//                     if let Some(symbol_id) = scope.lookup_symbol(name) {
//                          return Ok(symbol_id);
//                     }
//
//                     // Remonter au scope parent
//                     match scope.parent {
//                          Some(parent_id) => {
//                               current_scope_id = parent_id;
//                          },
//                          None => break, // Nous avons atteint le scope global
//                     }
//                } else {
//                     break; // Scope invalide
//                }
//           }
//
//           // Vérifier les imports (à implémenter)
//
//           Err(create_symbol_error(
//                SymbolError::SymbolNotFound(name.to_string()),
//                Position { index: 0 }
//           ))
//      }
//
//      /// Recherche un symbole dans un scope spécifique uniquement
//      pub fn lookup_symbol_in_scope(&self, name: &str, scope_id: ScopeId) -> Result<SymbolId, SemanticError> {
//           if let Some(scope) = self.scopes.get(&scope_id) {
//                if let Some(symbol_id) = scope.lookup_symbol(name) {
//                     return Ok(symbol_id);
//                }
//           }
//           Err(create_symbol_error(
//                SymbolError::SymbolNotFound(name.to_string()),
//                Position { index: 0 }
//           ))
//      }
//
//      /// Résout un chemin qualifié (par exemple, module::sous_module::nom)
//      pub fn resolve_qualified_name(&self, path: &[String]) -> Result<SymbolId, SemanticError> {
//           if path.is_empty() {
//                return Err(create_symbol_error(
//                     SymbolError::InvalidScope,
//                     Position { index: 0 }
//                ));
//           }
//
//           // Si le chemin a une seule partie, c'est un lookup simple
//           if path.len() == 1 {
//                return self.lookup_symbol(&path[0]);
//           }
//
//           // Sinon, on parcourt le chemin
//           let mut current_scope_id = self.current_scope;
//           let mut current_symbol_id = None;
//
//           for (i, part) in path.iter().enumerate() {
//                if i == 0 {
//                     // Première partie du chemin
//                     match self.lookup_symbol(part) {
//                          Ok(symbol_id) => {
//                               let symbol = self.get_symbol(symbol_id)?;
//
//                               // Vérifier que c'est un module ou un type
//                               match symbol.kind {
//                                    SymbolKind::Module | SymbolKind::Class | SymbolKind::Trait | SymbolKind::Enum => {
//                                         current_symbol_id = Some(symbol_id);
//
//                                         // Pour un module, changer le scope courant
//                                         if let SymbolKind::Module = symbol.kind {
//                                              // Trouver le scope associé au module
//                                              for (scope_id, scope) in &self.scopes {
//                                                   if scope.symbols.get(part) == Some(&symbol_id) {
//                                                        current_scope_id = *scope_id;
//                                                        break;
//                                                   }
//                                              }
//                                         }
//                                    },
//                                    _ => return Err(create_symbol_error(
//                                         SymbolError::InvalidScope,
//                                         Position { index: 0 }
//                                    )),
//                               }
//                          },
//                          Err(e) => return Err(e),
//                     }
//                } else {
//                     // Parties suivantes du chemin
//                     if let Some(scope) = self.scopes.get(&current_scope_id) {
//                          match scope.lookup_symbol(part) {
//                               Some(symbol_id) => {
//                                    current_symbol_id = Some(symbol_id);
//
//                                    // Mettre à jour le scope si nécessaire
//                                    let symbol = self.get_symbol(symbol_id)?;
//                                    if let SymbolKind::Module = symbol.kind {
//                                         // Trouver le scope du module
//                                         for (scope_id, scope) in &self.scopes {
//                                              if scope.symbols.get(part) == Some(&symbol_id) {
//                                                   current_scope_id = *scope_id;
//                                                   break;
//                                              }
//                                         }
//                                    }
//                               },
//                               None => return Err(create_symbol_error(
//                                    SymbolError::SymbolNotFound(part.clone()),
//                                    Position { index: 0 }
//                               )),
//                          }
//                     } else {
//                          return Err(create_symbol_error(
//                               SymbolError::InvalidScope,
//                               Position { index: 0 }
//                          ));
//                     }
//                }
//           }
//
//           match current_symbol_id {
//                Some(id) => Ok(id),
//                None => Err(create_symbol_error(
//                     SymbolError::SymbolNotFound("Path resolution failed".to_string()),
//                     Position { index: 0 }
//                )),
//           }
//      }
//
//      pub fn validate_references(&self) -> Result<(), Vec<SemanticError>> {
//           // TODO: Implémenter la validation des références
//           Ok(())
//      }
//
//      pub fn check_unused_symbols(&self) -> Vec<SymbolId> {
//           // TODO: Implémenter la vérification des symboles non inutilisés
//           Vec::new()
//      }
// }
//
//
// // Implémentation de MutabilityManager pour SymbolTable
// impl MutabilityManager for SymbolTable {
//      fn is_mutable(&self, symbol_id: SymbolId) -> Result<bool, SemanticError> {
//           match self.get_symbol(symbol_id) {
//                Ok(symbol) => Ok(symbol.attributes.is_mutable),
//                Err(e) => Err(e),
//           }
//      }
//
//      fn mark_initialized(&mut self, symbol_id: SymbolId) -> Result<(), SemanticError> {
//           if let Some(symbol) = self.symbols.get_mut(&symbol_id) {
//                symbol.attributes.is_initialized = true;
//                // Également marquer dans le borrow checker
//                self.borrow_checker.mark_initialized(symbol_id);
//                Ok(())
//           } else {
//                Err(create_symbol_error(
//                     SymbolError::SymbolNotFound(format!("{:?}", symbol_id)),
//                     Position { index: 0 }
//                ))
//           }
//      }
//
//      fn is_initialized(&self, symbol_id: SymbolId) -> Result<bool, SemanticError> {
//           // Vérifier d'abord dans le borrow checker
//           if self.borrow_checker.is_initialized(symbol_id) {
//                return Ok(true);
//           }
//
//           // Sinon vérifier dans le symbole
//           match self.get_symbol(symbol_id) {
//                Ok(symbol) => Ok(symbol.attributes.is_initialized),
//                Err(e) => Err(e),
//           }
//      }
//
//      fn register_read(&mut self, symbol_id: SymbolId, location: SourceLocation) -> Result<(), SemanticError> {
//           // Vérifier si la variable existe
//           self.get_symbol(symbol_id)?;
//
//           // Vérifier si la variable est initialisée
//           if !self.is_initialized(symbol_id)? {
//                return Err(create_semantic_error(
//                     SemanticErrorType::TypeError(InvalidType(
//                               format!("Variable {:?} is not initialized", symbol_id)
//                          )
//                     ),
//                     "Use of uninitialized variable".to_string(),
//                     Position { index: 0 }
//                ));
//           }
//
//           // Enregistrer l'emprunt de lecture
//           match self.borrow_checker.register_borrow(
//                symbol_id,
//                BorrowKind::Read,
//                location,
//                self.current_scope,
//                None,
//           ) {
//                Ok(()) => Ok(()),
//                Err(e) => {
//                     // Convertir l'erreur de borrow en erreur sémantique
//                     Err(create_semantic_error(
//                          SemanticErrorType::TypeError(TypeMismatch(
//                                    format!("Borrow checker error: {:?}", e)
//                               )
//                          ),
//                          "Borrow checker error".to_string(),
//                          Position { index: 0 }
//                     ))
//                }
//           }
//      }
//
//      fn register_write(&mut self, symbol_id: SymbolId, location: SourceLocation) -> Result<(), SemanticError> {
//           // Vérifier si la variable existe
//           let symbol = self.get_symbol(symbol_id)?;
//
//           // Vérifier si la variable est mutable
//           if !symbol.attributes.is_mutable {
//                return Err(create_semantic_error(
//                     SemanticErrorType::TypeError(InvalidType(
//                               format!("Cannot modify immutable variable {:?}", symbol_id)
//                          )
//                     ),
//                     "Attempt to modify immutable value".to_string(),
//                     Position { index: 0 }
//                ));
//           }
//
//           // Enregistrer l'emprunt d'écriture
//           match self.borrow_checker.register_borrow(
//                symbol_id,
//                BorrowKind::Write,
//                location,
//                self.current_scope,
//                None,
//           ) {
//                Ok(()) => {
//                     // Marquer comme initialisé
//                     if let Some(symbol) = self.symbols.get_mut(&symbol_id) {
//                          symbol.attributes.is_initialized = true;
//                     }
//                     Ok(())
//                },
//                Err(e) => {
//                     // Convertir l'erreur de borrow en erreur sémantique
//                     Err(create_semantic_error(
//                          SemanticErrorType::TypeError(
//                               crate::semantic::semantic_error::TypeError::TypeMismatch(
//                                    format!("Borrow checker error: {:?}", e)
//                               )
//                          ),
//                          "Borrow checker error".to_string(),
//                          Position { index: 0 }
//                     ))
//                }
//           }
//      }
//
//      fn register_immutable_borrow(&mut self, symbol_id: SymbolId, location: SourceLocation) -> Result<(), SemanticError> {
//           // Vérifier si la variable existe
//           self.get_symbol(symbol_id)?;
//
//           // Vérifier si la variable est initialisée
//           if !self.is_initialized(symbol_id)? {
//                return Err(create_semantic_error(
//                     SemanticErrorType::TypeError(InvalidType(
//                               format!("Cannot borrow uninitialized variable {:?}", symbol_id)
//                          )
//                     ),
//                     "Borrow of uninitialized variable".to_string(),
//                     Position { index: 0 }
//                ));
//           }
//
//           // Enregistrer l'emprunt immutable
//           match self.borrow_checker.register_borrow(
//                symbol_id,
//                BorrowKind::Immutable,
//                location,
//                self.current_scope,
//                None,
//           ) {
//                Ok(()) => Ok(()),
//                Err(e) => {
//                     // Convertir l'erreur de borrow en erreur sémantique
//                     Err(create_semantic_error(
//                          SemanticErrorType::TypeError(TypeMismatch(
//                                    format!("Borrow checker error: {:?}", e)
//                               )
//                          ),
//                          "Borrow checker error".to_string(),
//                          Position { index: 0 }
//                     ))
//                }
//           }
//      }
//
//      fn register_mutable_borrow(&mut self, symbol_id: SymbolId, location: SourceLocation) -> Result<(), SemanticError> {
//           // Vérifier si la variable existe
//           let symbol = self.get_symbol(symbol_id)?;
//
//           // Vérifier si la variable est mutable
//           if !symbol.attributes.is_mutable {
//                return Err(create_semantic_error(
//                     SemanticErrorType::TypeError(InvalidType(
//                               format!("Cannot mutably borrow immutable variable {:?}", symbol_id)
//                          )
//                     ),
//                     "Mutable borrow of immutable value".to_string(),
//                     Position { index: 0 }
//                ));
//           }
//
//           // Vérifier si la variable est initialisée
//           if !self.is_initialized(symbol_id)? {
//                return Err(create_semantic_error(
//                     SemanticErrorType::TypeError(InvalidType(
//                               format!("Cannot borrow uninitialized variable {:?}", symbol_id)
//                          )
//                     ),
//                     "Borrow of uninitialized variable".to_string(),
//                     Position { index: 0 }
//                ));
//           }
//
//           // Enregistrer l'emprunt mutable
//           match self.borrow_checker.register_borrow(
//                symbol_id,
//                BorrowKind::Mutable,
//                location,
//                self.current_scope,
//                None,
//           ) {
//                Ok(()) => Ok(()),
//                Err(e) => {
//                     // Convertir l'erreur de borrow en erreur sémantique
//                     Err(create_semantic_error(
//                          SemanticErrorType::TypeError(TypeMismatch(
//                                    format!("Borrow checker error: {:?}", e)
//                               )
//                          ),
//                          "Borrow checker error".to_string(),
//                          Position { index: 0 }
//                     ))
//                }
//           }
//      }
//
//      fn register_move(&mut self, symbol_id: SymbolId, location: SourceLocation) -> Result<(), SemanticError> {
//           // Vérifier si la variable existe
//           self.get_symbol(symbol_id)?;
//
//           // Vérifier si la variable est initialisée
//           if !self.is_initialized(symbol_id)? {
//                return Err(create_semantic_error(
//                     SemanticErrorType::TypeError(InvalidType(
//                               format!("Cannot move uninitialized variable {:?}", symbol_id)
//                          )
//                     ),
//                     "Move of uninitialized variable".to_string(),
//                     Position { index: 0 }
//                ));
//           }
//
//           // Enregistrer le move
//           match self.borrow_checker.register_borrow(
//                symbol_id,
//                BorrowKind::Move,
//                location,
//                self.current_scope,
//                None,
//           ) {
//                Ok(()) => Ok(()),
//                Err(e) => {
//                     // Convertir l'erreur de borrow en erreur sémantique
//                     Err(create_semantic_error(
//                          SemanticErrorType::TypeError(TypeMismatch(
//                                    format!("Borrow checker error: {:?}", e)
//                               )
//                          ),
//                          "Borrow checker error".to_string(),
//                          Position { index: 0 }
//                     ))
//                }
//           }
//      }
// }
//
// // Fonction utilitaire pour créer une erreur sémantique à partir d'une erreur de symbole
// fn create_symbol_error(error: SymbolError, position: Position) -> SemanticError {
//      SemanticError::new(
//           SemanticErrorType::SymbolError(error),
//           "Symbol error".to_string(),
//           position
//      )
// }
//
// // Fonction utilitaire pour créer une erreur sémantique générale
// fn create_semantic_error(error_type: SemanticErrorType, message: String, position: Position) -> SemanticError {
//      SemanticError::new(
//           error_type,
//           message,
//           position
//      )
// }


#[cfg(test)]
mod tests {
     use super::*;

     fn create_location() -> SourceLocation {
          SourceLocation {
               file: "test.punk".to_string(),
               line: 1,
               column: 1,
          }
     }

     #[test]
     fn test_symbol_table_creation() {
          let table = SymbolTable::new();
          assert_eq!(table.current_scope, ScopeId(0));
          assert_eq!(table.scopes.len(), 1);
     }

     #[test]
     fn test_enter_exit_scope() {
          let mut table = SymbolTable::new();

          // Entrer dans un nouveau scope
          let block_scope = table.enter_scope(ScopeKind::Block);
          assert_ne!(block_scope, ScopeId(0));
          assert_eq!(table.current_scope, block_scope);

          // Sortir du scope
          let result = table.exit_scope();
          assert!(result.is_ok());
          assert_eq!(result.unwrap(), ScopeId(0));
          assert_eq!(table.current_scope, ScopeId(0));
     }

     // #[test]
     // fn test_declare_lookup_symbol() {
     //      let mut table = SymbolTable::new();
     //      let location = create_location();
     //
     //      // Déclarer un symbole dans le scope global
     //      let var_id = table.declare_symbol("x".to_string(), SymbolKind::Variable, location.clone());
     //      assert!(var_id.is_ok());
     //
     //      // Rechercher le symbole
     //      let lookup_result = table.lookup_symbol("x");
     //      assert!(lookup_result.is_ok());
     //      assert_eq!(lookup_result.unwrap(), var_id.clone().unwrap());
     //
     //      // Entrer dans un nouveau scope
     //      let block_scope = table.enter_scope(ScopeKind::Block);
     //
     //      // Le symbole du parent est visible
     //      let lookup_result = table.lookup_symbol("x");
     //      assert!(lookup_result.is_ok());
     //
     //      // Déclarer un symbole avec le même nom dans le scope enfant (shadowing)
     //      let inner_var_id = table.declare_symbol("x".to_string(), SymbolKind::Variable, location.clone());
     //      assert!(inner_var_id.is_ok());
     //      assert_ne!(inner_var_id.unwrap(), var_id.unwrap());
     //
     //      // Le lookup trouve maintenant le symbole le plus proche (shadowed)
     //      let lookup_result = table.lookup_symbol("x");
     //      assert!(lookup_result.is_ok());
     //      assert_eq!(lookup_result.unwrap(), inner_var_id.unwrap());
     //
     //      // Sortir du scope
     //      let _ = table.exit_scope();
     //
     //      // Le lookup trouve maintenant le symbole original
     //      let lookup_result = table.lookup_symbol("x");
     //      assert!(lookup_result.is_ok());
     //      assert_eq!(lookup_result.unwrap(), var_id.unwrap());
     // }

     #[test]
     fn test_declare_lookup_symbol() {
          // Configuration initiale
          let mut table = SymbolTable::new();
          let location = create_location();

          // Test 1 : Déclaration dans le scope global
          let var_name = "x".to_string();
          let var_id = table.declare_symbol(
               var_name.clone(),
               SymbolKind::Variable,
               location.clone()
          ).expect("La déclaration dans le scope global devrait réussir");

          // Test 2 : Vérification de la recherche dans le scope global
          let lookup_id = table.lookup_symbol(&var_name)
              .expect("La recherche dans le scope global devrait réussir");
          assert_eq!(lookup_id, var_id, "L'ID du symbole trouvé devrait correspondre à l'ID déclaré");

          // Test 3 : Vérification de la visibilité dans un nouveau scope
          let block_scope = table.enter_scope(ScopeKind::Block);
          let parent_lookup = table.lookup_symbol(&var_name)
              .expect("Le symbole du parent devrait être visible dans le scope enfant");
          assert_eq!(parent_lookup, var_id, "Le symbole du parent devrait être trouvé");

          // Test 4 : Test du shadowing dans le scope enfant
          let inner_var_id = table.declare_symbol(
               var_name.clone(),
               SymbolKind::Variable,
               location.clone()
          ).expect("La déclaration dans le scope enfant devrait réussir");

          // Vérification que les IDs sont différents (shadowing)
          assert_ne!(
               inner_var_id, var_id,
               "Les IDs des variables avec le même nom dans différents scopes devraient être différents"
          );

          // Test 5 : Vérification que le lookup trouve la variable la plus proche
          let shadowed_lookup = table.lookup_symbol(&var_name)
              .expect("La recherche après shadowing devrait réussir");
          assert_eq!(
               shadowed_lookup, inner_var_id,
               "Le lookup devrait trouver la variable du scope actuel (shadowed)"
          );

          // Test 6 : Vérification après sortie du scope
          table.exit_scope().expect("La sortie du scope devrait réussir");
          let original_lookup = table.lookup_symbol(&var_name)
              .expect("La recherche après sortie du scope devrait réussir");
          assert_eq!(
               original_lookup, var_id,
               "Le lookup devrait retrouver la variable originale après sortie du scope"
          );
     }

     #[test]
     fn test_symbol_redeclaration() {
          let mut table = SymbolTable::new();
          let location = create_location();

          // Déclarer un symbole
          let var_id = table.declare_symbol("y".to_string(), SymbolKind::Variable, location.clone());
          assert!(var_id.is_ok());

          // Essayer de redéclarer le même symbole dans le même scope
          let redecl_result = table.declare_symbol("y".to_string(), SymbolKind::Function, location.clone());
          assert!(redecl_result.is_err());

          // Mais cela devrait fonctionner dans un scope différent
          let block_scope = table.enter_scope(ScopeKind::Block);
          let inner_decl = table.declare_symbol("y".to_string(), SymbolKind::Function, location.clone());
          assert!(inner_decl.is_ok());
     }

     #[test]
     fn test_nested_scopes() {
          let mut table = SymbolTable::new();
          let location = create_location();

          // Créer une structure de scope imbriquée
          // Global -> Module -> Function -> Block

          // Déclarer dans le scope global
          let global_var = table.declare_symbol("global".to_string(), SymbolKind::Variable, location.clone());
          assert!(global_var.is_ok());

          // Module scope
          let module_scope = table.enter_scope(ScopeKind::Module);
          let module_var = table.declare_symbol("module_var".to_string(), SymbolKind::Variable, location.clone());
          assert!(module_var.is_ok());

          // Function scope
          let function_scope = table.enter_scope(ScopeKind::Function);
          let function_var = table.declare_symbol("function_var".to_string(), SymbolKind::Variable, location.clone());
          assert!(function_var.is_ok());

          // Block scope
          let block_scope = table.enter_scope(ScopeKind::Block);
          let block_var = table.declare_symbol("block_var".to_string(), SymbolKind::Variable, location.clone());
          assert!(block_var.is_ok());

          // Vérifier la visibilité des symboles dans le scope le plus profond
          assert!(table.lookup_symbol("global").is_ok());
          assert!(table.lookup_symbol("module_var").is_ok());
          assert!(table.lookup_symbol("function_var").is_ok());
          assert!(table.lookup_symbol("block_var").is_ok());

          // Remonter au scope de fonction
          let _ = table.exit_scope();
          assert!(table.lookup_symbol("global").is_ok());
          assert!(table.lookup_symbol("module_var").is_ok());
          assert!(table.lookup_symbol("function_var").is_ok());
          assert!(table.lookup_symbol("block_var").is_err()); // Plus visible

          // Remonter au scope de module
          let _ = table.exit_scope();
          assert!(table.lookup_symbol("global").is_ok());
          assert!(table.lookup_symbol("module_var").is_ok());
          assert!(table.lookup_symbol("function_var").is_err()); // Plus visible

          // Remonter au scope global
          let _ = table.exit_scope();
          assert!(table.lookup_symbol("global").is_ok());
          assert!(table.lookup_symbol("module_var").is_err()); // Plus visible
     }

     #[test]
     fn test_mutability_and_initialization() {
          let mut table = SymbolTable::new();
          let location = create_location();

          // Déclarer une variable
          let var_id = table.declare_symbol("x".to_string(), SymbolKind::Variable, location.clone()).unwrap();

          // Par défaut, la variable n'est pas initialisée
          assert!(!table.is_initialized(var_id).unwrap());

          // La lecture devrait échouer (non initialisée)
          assert!(table.register_read(var_id, location.clone()).is_err());

          // Rendre la variable mutable
          if let Some(symbol) = table.symbols.get_mut(&var_id) {
               symbol.attributes.is_mutable = true;
          }

          // L'écriture devrait fonctionner et initialiser la variable
          assert!(table.register_write(var_id, location.clone()).is_ok());

          // Maintenant la variable est initialisée
          assert!(table.is_initialized(var_id).unwrap());

          // La lecture devrait fonctionner
          assert!(table.register_read(var_id, location.clone()).is_ok());
     }

     #[test]
     fn test_borrowing_rules() {
          let mut table = SymbolTable::new();
          let location = create_location();

          // Déclarer une variable mutable
          let var_id = table.declare_symbol("x".to_string(), SymbolKind::Variable, location.clone()).unwrap();
          if let Some(symbol) = table.symbols.get_mut(&var_id) {
               symbol.attributes.is_mutable = true;
          }

          // Initialiser la variable
          assert!(table.register_write(var_id, location.clone()).is_ok(),
                  "L'initialisation devrait réussir");

          // Un emprunt immutable devrait fonctionner
          assert!(table.register_immutable_borrow(var_id, location.clone()).is_ok(),
                  "Le premier emprunt immutable devrait réussir");

          // Un autre emprunt immutable devrait fonctionner
          assert!(table.register_immutable_borrow(var_id, location.clone()).is_ok(),
                  "Le deuxième emprunt immutable devrait réussir");

          // Un emprunt mutable devrait échouer (car il y a des emprunts immutables)
          assert!(table.register_mutable_borrow(var_id, location.clone()).is_err(),
                  "L'emprunt mutable devrait échouer quand il y a des emprunts immutables");

          // Libérer manuellement les emprunts du scope actuel
          table.borrow_checker.release_borrows_for_scope(table.current_scope);

          // Maintenant un emprunt mutable devrait fonctionner
          assert!(table.register_mutable_borrow(var_id, location.clone()).is_ok(),
                  "L'emprunt mutable devrait réussir après la libération des emprunts immutables");

          // Un autre emprunt mutable devrait échouer
          assert!(table.register_mutable_borrow(var_id, location.clone()).is_err(),
                  "Un deuxième emprunt mutable devrait échouer");

          // Libérer manuellement les emprunts du scope actuel à nouveau
          // table.borrow_checker.release_borrows_for_scope(table.current_scope);

          // Maintenant un emprunt immutable devrait fonctionner
          assert!(table.register_immutable_borrow(var_id, location.clone()).is_ok(),
                  "L'emprunt immutable devrait réussir après la libération de l'emprunt mutable");
     }

}