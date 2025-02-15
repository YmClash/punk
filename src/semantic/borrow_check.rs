// use std::collections::{HashMap, HashSet};
//
// pub struct BorrowChecker {
//     // Pour chaque variable, garde une trace des emprunts actifs
//     borrows: HashMap<String, Vec<BorrowInfo>>,
//     lifetimes:Vec<LifetimeScope>,
//     move_values:HashSet<String>,
// }
//
//
// // Implémentation des règles
// impl BorrowChecker {
//     fn check_borrow(&mut self, var: &str, is_mut: bool) -> Result<(), Semantoc>;
//     fn check_move(&mut self, var: &str) -> Result<(), Error>;
//     fn validate_lifetime(&mut self, scope: &Scope) -> Result<(), Error>;
// }
//
//
//
//
// // fn validate_borrowing(ast: &AST) -> Result<(), Error> {
// //     // Vérifier les règles d'emprunt
// //     // - Une seule référence mutable à la fois
// //     // - Plusieurs références immutables possibles
// //     // - Pas de références après un move
// //     todo!()
// // }