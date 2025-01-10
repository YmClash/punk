// pub struct BorrowChecker {
//     // Pour chaque variable, garde une trace des emprunts actifs
//     borrows: HashMap<String, Vec<BorrowInfo>>,
// }
//
// struct BorrowInfo {
//     kind: BorrowKind,
//     scope_id: usize,
//     location: Position,
// }
//
// enum BorrowKind {
//     Shared,
//     Mutable,
// }
//
// impl BorrowChecker {
//     fn check_borrow(&mut self, var_name: &str, borrow_kind: BorrowKind,
//                     scope_id: usize) -> Result<(), String> {
//         match borrow_kind {
//             BorrowKind::Mutable => {
//                 // Vérifie qu'il n'y a pas d'autres emprunts
//                 if !self.borrows.get(var_name).unwrap_or(&vec![]).is_empty() {
//                     return Err("Cannot borrow as mutable when already borrowed".into());
//                 }
//             }
//             BorrowKind::Shared => {
//                 // Vérifie qu'il n'y a pas d'emprunt mutable
//                 if self.borrows.get(var_name)
//                     .unwrap_or(&vec![])
//                     .iter()
//                     .any(|b| matches!(b.kind, BorrowKind::Mutable)) {
//                     return Err("Cannot borrow as immutable when mutably borrowed".into());
//                 }
//             }
//         }
//         // Ajoute le nouvel emprunt
//         self.borrows.entry(var_name.to_string())
//             .or_default()
//             .push(BorrowInfo {
//                 kind: borrow_kind,
//                 scope_id,
//                 location: Position { index: 0 }, // À adapter
//             });
//         Ok(())
//     }
// }