// src/parser/declarations/mod.rs
// Module principal pour les déclarations

pub mod variables;
pub mod functions;
pub mod structs;
pub mod classes;
pub mod traits;
pub mod enums;
pub mod impl_blocks;

// Ré-exporter les fonctions principales pour faciliter l'accès
pub use self::variables::*;
pub use self::functions::*;
pub use self::structs::*;
pub use self::classes::*;
pub use self::traits::*;
pub use self::enums::*;
pub use self::impl_blocks::*;