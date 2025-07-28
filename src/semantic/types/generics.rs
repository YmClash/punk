// // src/semantic/generics.rs
//
// use std::collections::{HashMap, HashSet};
// use crate::semantic::symbols::{SymbolId, ScopeId, SourceLocation};
// use crate::semantic::types::{Type, TypeId, TypeKind, TypeRegistry};
// use crate::semantic::traits::{TraitManager};
// use crate::semantic::semantic_error::{SemanticError, TypeError, SemanticErrorType, Position};
// use crate::semantic::types::type_system::{TypeId, TypeKind, TypeRegistry};
//
// /// Paramètre de type générique
// #[derive(Debug, Clone)]
// pub struct GenericParameter {
//     pub name: String,                // Nom du paramètre (T, U, etc.)
//     pub trait_bounds: Vec<SymbolId>, // Contraintes de traits (T: Display + Clone)
//     pub default_type: Option<TypeId>, // Type par défaut pour les paramètres optionnels
// }
//
// /// Contexte d'instanciation pour les types génériques
// #[derive(Debug, Clone)]
// pub struct GenericContext {
//     /// Map des paramètres de type vers leurs types concrets
//     type_arguments: HashMap<String, TypeId>,
//
//     /// Les paramètres de type non encore instanciés
//     unresolved_parameters: HashSet<String>,
// }
//
// impl GenericContext {
//     /// Crée un nouveau contexte vide
//     pub fn new() -> Self {
//         GenericContext {
//             type_arguments: HashMap::new(),
//             unresolved_parameters: HashSet::new(),
//         }
//     }
//
//     /// Ajoute un paramètre de type non résolu
//     pub fn add_parameter(&mut self, name: String) {
//         self.unresolved_parameters.insert(name);
//     }
//
//     /// Instancie un paramètre de type avec un type concret
//     pub fn set_type_argument(&mut self, param_name: String, type_id: TypeId) -> Result<(), SemanticError> {
//         if !self.unresolved_parameters.contains(&param_name) {
//             return Err(create_semantic_error(
//                 SemanticErrorType::TypeError(TypeError::InvalidTypeParameter(
//                     format!("Unknown type parameter '{}'", param_name)
//                 )),
//                 "Invalid type parameter".to_string(),
//                 Position { index: 0 }
//             ));
//         }
//
//         self.type_arguments.insert(param_name.clone(), type_id);
//         self.unresolved_parameters.remove(&param_name);
//         Ok(())
//     }
//
//     /// Récupère le type concret pour un paramètre de type
//     pub fn get_type_argument(&self, param_name: &str) -> Option<TypeId> {
//         self.type_arguments.get(param_name).copied()
//     }
//
//     /// Vérifie si tous les paramètres ont été instanciés
//     pub fn is_fully_instantiated(&self) -> bool {
//         self.unresolved_parameters.is_empty()
//     }
//
//     /// Vérifie si un paramètre a été instancié
//     pub fn is_instantiated(&self, param_name: &str) -> bool {
//         self.type_arguments.contains_key(param_name)
//     }
// }
//
// /// Gestionnaire des types génériques
// pub struct GenericManager {
//     pub type_registry: TypeRegistry,
//     pub trait_manager: TraitManager,
// }
//
// impl GenericManager {
//     /// Crée un nouveau gestionnaire de génériques
//     pub fn new(type_registry: TypeRegistry, trait_manager: TraitManager) -> Self {
//         GenericManager {
//             type_registry,
//             trait_manager,
//         }
//     }
//
//     /// Instancie un type générique avec des types concrets
//     pub fn instantiate_generic_type(
//         &mut self,
//         base_type_id: TypeId,
//         context: &GenericContext
//     ) -> Result<TypeId, SemanticError> {
//         let base_type = self.type_registry.get_type(base_type_id)
//             .ok_or_else(|| create_semantic_error(
//                 SemanticErrorType::TypeError(TypeError::TypeNotFound(
//                     format!("Type with ID {:?} not found", base_type_id)
//                 )),
//                 "Type not found".to_string(),
//                 Position { index: 0 }
//             ))?.clone();
//
//         match &base_type.kind {
//             TypeKind::TypeParameter(param_name) => {
//                 if let Some(concrete_type_id) = context.get_type_argument(param_name) {
//                     Ok(concrete_type_id)
//                 } else {
//                     Err(create_semantic_error(
//                         SemanticErrorType::TypeError(TypeError::InvalidTypeParameter(
//                             format!("Type parameter '{}' not instantiated", param_name)
//                         )),
//                         "Uninstantiated type parameter".to_string(),
//                         Position { index: 0 }
//                     ))
//                 }
//             },
//
//             TypeKind::Array(element_type) => {
//                 let concrete_element_type_id = self.instantiate_generic_type(element_type.id, context)?;
//                 Ok(self.type_registry.create_array_type(concrete_element_type_id))
//             },
//
//             TypeKind::Tuple(element_types) => {
//                 let mut concrete_element_type_ids = Vec::with_capacity(element_types.len());
//
//                 for element_type in element_types {
//                     let concrete_element_type_id = self.instantiate_generic_type(element_type.id, context)?;
//                     concrete_element_type_ids.push(concrete_element_type_id);
//                 }
//
//                 Ok(self.type_registry.create_tuple_type(concrete_element_type_ids))
//             },
//
//             TypeKind::Function { params, return_type } => {
//                 let mut concrete_param_type_ids = Vec::with_capacity(params.len());
//
//                 for param_type in params {
//                     let concrete_param_type_id = self.instantiate_generic_type(param_type.id, context)?;
//                     concrete_param_type_ids.push(concrete_param_type_id);
//                 }
//
//                 let concrete_return_type_id = self.instantiate_generic_type(return_type.id, context)?;
//
//                 Ok(self.type_registry.create_function_type(
//                     concrete_param_type_ids,
//                     concrete_return_type_id
//                 ))
//             },
//
//             TypeKind::Reference { inner, is_mutable, lifetime } => {
//                 let concrete_inner_type_id = self.instantiate_generic_type(inner.id, context)?;
//
//                 Ok(self.type_registry.create_reference_type(
//                     concrete_inner_type_id,
//                     *is_mutable,
//                     lifetime.clone()
//                 ))
//             },
//
//             // Les types primitifs ou les types déjà concrets sont retournés tels quels
//             _ => Ok(base_type_id),
//         }
//     }
//
//     /// Vérifie si un type satisfait une contrainte de trait
//     pub fn check_trait_bound(
//         &self,
//         type_id: TypeId,
//         trait_id: SymbolId
//     ) -> bool {
//         self.trait_manager.type_implements_trait(type_id, trait_id)
//     }
//
//     /// Vérifie si un type satisfait toutes les contraintes d'un paramètre générique
//     pub fn check_bounds(
//         &self,
//         type_id: TypeId,
//         bounds: &[SymbolId]
//     ) -> Result<(), SemanticError> {
//         for trait_id in bounds {
//             if !self.check_trait_bound(type_id, *trait_id) {
//                 let trait_def = self.trait_manager.get_trait(*trait_id)
//                     .ok_or_else(|| create_semantic_error(
//                         SemanticErrorType::SymbolError(
//                             crate::semantic::semantic_error::SymbolError::SymbolNotFound(
//                                 format!("Trait with ID {:?}", trait_id)
//                             )
//                         ),
//                         "Unknown trait".to_string(),
//                         Position { index: 0 }
//                     ))?;
//
//                 let type_obj = self.type_registry.get_type(type_id)
//                     .ok_or_else(|| create_semantic_error(
//                         SemanticErrorType::TypeError(TypeError::TypeNotFound(
//                             format!("Type with ID {:?}", type_id)
//                         )),
//                         "Type not found".to_string(),
//                         Position { index: 0 }
//                     ))?;
//
//                 return Err(create_semantic_error(
//                     SemanticErrorType::TypeError(TypeError::TypeMismatch(
//                         format!("Type {} does not satisfy trait bound {}", type_obj, trait_def.name)
//                     )),
//                     "Unsatisfied trait bound".to_string(),
//                     Position { index: 0 }
//                 ));
//             }
//         }
//
//         Ok(())
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