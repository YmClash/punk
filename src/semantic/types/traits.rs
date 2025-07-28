// // src/semantic/traits.rs
//
// use std::collections::HashMap;
// use crate::semantic::symbols::{SymbolId, ScopeId, SourceLocation};
// use crate::semantic::types::{Type, TypeId, TypeKind, TypeRegistry};
// use crate::semantic::semantic_error::{SemanticError, TypeError, SemanticErrorType, Position};
// use crate::semantic::types::type_system::TypeId;
//
// /// Représente un trait (interface)
// #[derive(Debug, Clone)]
// pub struct Trait {
//     pub symbol_id: SymbolId,            // ID du symbole associé
//     pub name: String,                   // Nom du trait
//     pub methods: Vec<TraitMethod>,      // Méthodes du trait
//     pub associated_types: Vec<AssociatedType>,  // Types associés
//     pub super_traits: Vec<SymbolId>,    // Traits dont ce trait hérite
// }
//
// /// Méthode définie dans un trait
// #[derive(Debug, Clone)]
// pub struct TraitMethod {
//     pub name: String,                // Nom de la méthode
//     pub params: Vec<TraitParameter>, // Paramètres
//     pub return_type: TypeId,         // Type de retour
//     pub is_optional: bool,           // La méthode a-t-elle une implémentation par défaut?
// }
//
// /// Paramètre d'une méthode de trait
// #[derive(Debug, Clone)]
// pub struct TraitParameter {
//     pub name: String,  // Nom du paramètre
//     pub param_type: TypeId, // Type du paramètre
// }
//
// /// Type associé à un trait
// #[derive(Debug, Clone)]
// pub struct AssociatedType {
//     pub name: String,                // Nom du type associé
//     pub bounds: Vec<SymbolId>,       // Contraintes sur ce type
// }
//
// /// Implémentation d'un trait pour un type
// #[derive(Debug, Clone)]
// pub struct TraitImpl {
//     pub trait_id: SymbolId,          // Trait implémenté
//     pub for_type: TypeId,            // Type pour lequel le trait est implémenté
//     pub methods: Vec<ImplementedMethod>, // Méthodes implémentées
//     pub associated_types: HashMap<String, TypeId>, // Types associés implémentés
// }
//
// /// Méthode implémentée pour un trait
// #[derive(Debug, Clone)]
// pub struct ImplementedMethod {
//     pub name: String,                // Nom de la méthode
//     pub symbol_id: SymbolId,         // Symbole de la fonction implémentée
// }
//
// /// Gestionnaire des traits
// pub struct TraitManager {
//     /// Les traits déclarés, indexés par ID de symbole
//     traits: HashMap<SymbolId, Trait>,
//
//     /// Les implémentations de traits
//     trait_impls: Vec<TraitImpl>,
// }
//
// impl TraitManager {
//     /// Crée un nouveau gestionnaire de traits
//     pub fn new() -> Self {
//         TraitManager {
//             traits: HashMap::new(),
//             trait_impls: Vec::new(),
//         }
//     }
//
//     /// Enregistre un nouveau trait
//     pub fn register_trait(&mut self, trait_def: Trait) -> Result<(), SemanticError> {
//         // Vérifier si ce trait est déjà enregistré
//         if self.traits.contains_key(&trait_def.symbol_id) {
//             return Err(create_semantic_error(
//                 SemanticErrorType::SymbolError(
//                     crate::semantic::semantic_error::SymbolError::SymbolAlreadyDeclared(
//                         trait_def.name.clone()
//                     )
//                 ),
//                 "Trait already defined".to_string(),
//                 Position { index: 0 }
//             ));
//         }
//
//         self.traits.insert(trait_def.symbol_id, trait_def);
//         Ok(())
//     }
//
//     /// Enregistre une implémentation de trait
//     pub fn register_trait_impl(&mut self, impl_def: TraitImpl) -> Result<(), SemanticError> {
//         // Vérifier si le trait existe
//         if !self.traits.contains_key(&impl_def.trait_id) {
//             return Err(create_semantic_error(
//                 SemanticErrorType::SymbolError(
//                     crate::semantic::semantic_error::SymbolError::SymbolNotFound(
//                         format!("Trait with ID {:?}", impl_def.trait_id)
//                     )
//                 ),
//                 "Implementation of unknown trait".to_string(),
//                 Position { index: 0 }
//             ));
//         }
//
//         // Vérifier si cette implémentation n'est pas en conflit avec une autre
//         for existing_impl in &self.trait_impls {
//             if existing_impl.trait_id == impl_def.trait_id && existing_impl.for_type == impl_def.for_type {
//                 return Err(create_semantic_error(
//                     SemanticErrorType::TypeError(TypeError::TypeMismatch(
//                         format!("Trait already implemented for this type")
//                     )),
//                     "Duplicate trait implementation".to_string(),
//                     Position { index: 0 }
//                 ));
//             }
//         }
//
//         // Vérifier que toutes les méthodes requises sont implémentées
//         let trait_def = self.traits.get(&impl_def.trait_id).unwrap();
//
//         for trait_method in &trait_def.methods {
//             if !trait_method.is_optional {
//                 let method_implemented = impl_def.methods.iter()
//                     .any(|m| m.name == trait_method.name);
//
//                 if !method_implemented {
//                     return Err(create_semantic_error(
//                         SemanticErrorType::TypeError(TypeError::TypeMismatch(
//                             format!("Missing implementation for required method '{}'", trait_method.name)
//                         )),
//                         "Incomplete trait implementation".to_string(),
//                         Position { index: 0 }
//                     ));
//                 }
//             }
//         }
//
//         // Vérifier que tous les types associés requis sont implémentés
//         for assoc_type in &trait_def.associated_types {
//             if !impl_def.associated_types.contains_key(&assoc_type.name) {
//                 return Err(create_semantic_error(
//                     SemanticErrorType::TypeError(TypeError::TypeMismatch(
//                         format!("Missing implementation for associated type '{}'", assoc_type.name)
//                     )),
//                     "Incomplete trait implementation".to_string(),
//                     Position { index: 0 }
//                 ));
//             }
//         }
//
//         self.trait_impls.push(impl_def);
//         Ok(())
//     }
//
//     /// Vérifie si un type implémente un trait
//     pub fn type_implements_trait(&self, type_id: TypeId, trait_id: SymbolId) -> bool {
//         self.trait_impls.iter().any(|impl_def| {
//             impl_def.trait_id == trait_id && impl_def.for_type == type_id
//         })
//     }
//
//     /// Récupère la liste des traits implémentés par un type
//     pub fn get_traits_for_type(&self, type_id: TypeId) -> Vec<SymbolId> {
//         self.trait_impls.iter()
//             .filter(|impl_def| impl_def.for_type == type_id)
//             .map(|impl_def| impl_def.trait_id)
//             .collect()
//     }
//
//     /// Recherche une méthode de trait pour un type
//     pub fn lookup_trait_method(
//         &self,
//         type_id: TypeId,
//         trait_id: SymbolId,
//         method_name: &str
//     ) -> Option<SymbolId> {
//         for impl_def in &self.trait_impls {
//             if impl_def.trait_id == trait_id && impl_def.for_type == type_id {
//                 for method in &impl_def.methods {
//                     if method.name == method_name {
//                         return Some(method.symbol_id);
//                     }
//                 }
//             }
//         }
//         None
//     }
//
//     /// Récupère un trait par son ID
//     pub fn get_trait(&self, trait_id: SymbolId) -> Option<&Trait> {
//         self.traits.get(&trait_id)
//     }
//
//     /// Recherche une implémentation de trait
//     pub fn get_trait_impl(&self, trait_id: SymbolId, type_id: TypeId) -> Option<&TraitImpl> {
//         self.trait_impls.iter().find(|impl_def| {
//             impl_def.trait_id == trait_id && impl_def.for_type == type_id
//         })
//     }
// }
//
// // Fonction utilitaire pour créer une erreur sémantique
// fn create_semantic_error(error_type: SemanticErrorType, message: String, position: Position) -> SemanticError {
//     SemanticError::new(
//         error_type,
//         message,
//         position
//     )
// }