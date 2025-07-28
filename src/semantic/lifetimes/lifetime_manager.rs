// use std::collections::{HashMap, HashSet};
// use crate::semantic::symbols::{ScopeId, SourceLocation, SymbolId};
// use crate::semantic::types::type_system::LifetimeId;
//
// pub struct LifetimeManager {
//     // Table des lifetimes
//     lifetimes: HashMap<LifetimeId, Lifetime>,
//
//     // Contraintes sur les lifetimes
//     constraints: Vec<LifetimeConstraint>,
//
//     // Relations d'inclusion entre les scopes et les lifetimes
//     scope_lifetimes: HashMap<ScopeId, HashSet<LifetimeId>>,
//
//     // Lifetimes associées aux symboles
//     symbol_lifetimes: HashMap<SymbolId, HashSet<LifetimeId>>,
//
//     // Générateur d'ID pour les lifetimes
//     next_lifetime_id: u32,
// }
//
// impl LifetimeManager {
//     pub fn new() -> Self {
//         LifetimeManager {
//             lifetimes: HashMap::new(),
//             constraints: Vec::new(),
//             scope_lifetimes: HashMap::new(),
//             symbol_lifetimes: HashMap::new(),
//             next_lifetime_id: 1,
//         }
//     }
//
//     // Crée une nouvelle lifetime
//     pub fn create_lifetime(&mut self, name: Option<String>, location: SourceLocation) -> LifetimeId {
//         let id = LifetimeId(self.next_lifetime_id);
//         self.next_lifetime_id += 1;
//
//         let lifetime = Lifetime {
//             id,
//             name,
//             location,
//         };
//
//         self.lifetimes.insert(id, lifetime);
//
//         id
//     }
//
//     // Ajoute une contrainte sur les lifetimes
//     pub fn add_constraint(&mut self, relation: LifetimeRelation, location: SourceLocation) {
//         let constraint = LifetimeConstraint {
//             relation,
//             location,
//         };
//
//         self.constraints.push(constraint);
//     }
//
//     // Associe une lifetime à un scope
//     pub fn associate_lifetime_to_scope(&mut self, lifetime_id: LifetimeId, scope_id: ScopeId) {
//         self.scope_lifetimes.entry(scope_id)
//             .or_insert_with(HashSet::new)
//             .insert(lifetime_id);
//     }
//
//     // Associe une lifetime à un symbole
//     pub fn associate_lifetime_to_symbol(&mut self, lifetime_id: LifetimeId, symbol_id: SymbolId) {
//         self.symbol_lifetimes.entry(symbol_id)
//             .or_insert_with(HashSet::new)
//             .insert(lifetime_id);
//     }
//
//     // Obtient toutes les lifetimes associées à un symbole
//     pub fn get_symbol_lifetimes(&self, symbol_id: SymbolId) -> HashSet<LifetimeId> {
//         self.symbol_lifetimes.get(&symbol_id)
//             .cloned()
//             .unwrap_or_else(HashSet::new)
//     }
//
//     // Obtient toutes les lifetimes associées à un scope
//     pub fn get_scope_lifetimes(&self, scope_id: ScopeId) -> HashSet<LifetimeId> {
//         self.scope_lifetimes.get(&scope_id)
//             .cloned()
//             .unwrap_or_else(HashSet::new)
//     }
// }