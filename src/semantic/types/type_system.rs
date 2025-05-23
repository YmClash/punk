//src/semantic/types/type_system.rs

use std::collections::HashMap;
use std::fmt;
use crate::parser::ast::{Expression, Literal, Operator, UnaryOperator, Type as ASTType};
use crate::semantic::semantic_error::{Position, SemanticError, SemanticErrorType, TypeError};
use crate::semantic::semantic_error::SemanticErrorType::{SymbolError};
use crate::semantic::symbol_table::SymbolTable;
use crate::semantic::symbols::SymbolId;

/// Identifiant unique pour les types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

/// Types primitifs et composés du système
#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    // Types primitifs
    Int,
    Float,
    Bool,
    Char,
    String,

    // Conteneurs et structures
    Array(Box<Type>, Option<usize>),  // Type des éléments, longueur optionnelle
    Tuple(Vec<Type>),
    Struct(StructTypeId),
    Enum(EnumTypeId),

    // Types pour les fonctions
    Function(FunctionType),

    // Types pour les références
    Reference(Box<Type>, Mutability, Option<LifetimeId>),

    // Type générique
    Generic(String, Vec<TypeConstraint>),

    // Type polymorphique (ex: auto, var, _)
    Infer(TypeVarId),

    // Type utilisateur
    Named(String, Vec<Type>),  // Nom du type, arguments génériques

    // Trait comme contrainte
    TraitBound(SymbolId),

    // Type d'une méthode associée à un self
    SelfType,

    // Type "unit" pour les fonctions sans retour
    Unit,

    // Type pour les valeurs impossibles (jamais atteintes)
    Never,

    // Type pour représenter une erreur
    Error,
}

/// Structure d'un Type
#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub id: TypeId,
    pub kind: TypeKind,
    pub nullable: bool,
}

impl Type {
    /// Crée un nouveau type
    pub fn new(id: TypeId, kind: TypeKind) -> Self {
        Type {
            id,
            kind,
            nullable: false,
        }
    }

    /// Rend le type nullable (équivalent à Option<T>)
    pub fn as_nullable(mut self) -> Self {
        self.nullable = true;
        self
    }

    /// Vérifie si deux types sont compatibles
    pub fn is_compatible_with(&self, other: &Type) -> bool {
        match (&self.kind, &other.kind) {
            // Mêmes types primitifs
            (TypeKind::Int, TypeKind::Int) |
            (TypeKind::Float, TypeKind::Float) |
            (TypeKind::Bool, TypeKind::Bool) |
            (TypeKind::Char, TypeKind::Char) |
            (TypeKind::String, TypeKind::String) => true,

            // Conversion implicite int -> float
            (TypeKind::Int, TypeKind::Float) => true,

            // Array compatible si les éléments sont compatibles
            (TypeKind::Array(t1, _), TypeKind::Array(t2, _)) => t1.is_compatible_with(t2),

            // Tuple compatible si tous les éléments sont compatibles et même longueur
            (TypeKind::Tuple(t1), TypeKind::Tuple(t2)) => {
                if t1.len() != t2.len() {
                    return false;
                }
                t1.iter().zip(t2.iter()).all(|(a, b)| a.is_compatible_with(b))
            },

            // Références
            (
                TypeKind::Reference(i1, m1, _),
                TypeKind::Reference(i2, m2, _)
            ) => {
                // Une référence mutable peut être convertie en référence immutable
                // mais pas l'inverse
                if *m2 == Mutability::Mutable && *m1 == Mutability::Immutable {
                    return false;
                }
                i1.is_compatible_with(i2)
            },

            // Tout type est compatible avec lui-même
            _ => self.id == other.id,
        }
    }
}

/// Formatter pour afficher les types de façon lisible
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            TypeKind::Int => write!(f, "int"),
            TypeKind::Float => write!(f, "float"),
            TypeKind::Bool => write!(f, "bool"),
            TypeKind::Char => write!(f, "char"),
            TypeKind::String => write!(f, "str"),

            TypeKind::Array(elem_type, size) => {
                if let Some(size) = size {
                    write!(f, "[{}; {}]", elem_type, size)
                } else {
                    write!(f, "[{}]", elem_type)
                }
            },
            TypeKind::Tuple(types) => {
                write!(f, "(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            },

            TypeKind::Struct(struct_id) => write!(f, "struct({})", struct_id.name),
            TypeKind::Enum(enum_id) => write!(f, "enum({})", enum_id.name),

            TypeKind::Function(func_type) => {
                write!(f, "fn(")?;
                for (i, param) in func_type.params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", func_type.return_type)
            },

            TypeKind::Reference(inner, mutability, lifetime) => {
                match mutability {
                    Mutability::Mutable => write!(f, "&mut ")?,
                    Mutability::Immutable => write!(f, "&")?,
                }
                if let Some(lt) = lifetime {
                    write!(f, "{} ", lt.name)?;
                }
                write!(f, "{}", inner)
            },

            TypeKind::Generic(name, _) => write!(f, "{}", name),
            TypeKind::Named(name, args) => {
                write!(f, "{}", name)?;
                if !args.is_empty() {
                    write!(f, "<")?;
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", arg)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            },
            TypeKind::TraitBound(sym_id) => write!(f, "dyn {:?}", sym_id),
            TypeKind::SelfType => write!(f, "Self"),
            TypeKind::Infer(_) => write!(f, "_"),
            TypeKind::Unit => write!(f, "()"),
            TypeKind::Never => write!(f, "!"),
            TypeKind::Error => write!(f, "{{error}}"),
        }?;

        if self.nullable {
            write!(f, "?")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mutability {
    Mutable,
    Immutable,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    pub params: Vec<Type>,
    pub return_type: Box<Type>,
    pub lifetime_params: Vec<LifetimeId>,
    pub type_params: Vec<TypeVarId>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeConstraint {
    Equal(Type, Type),
    Subtype(Type, Type),
    Instance(Type, Vec<Type>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructTypeId {
    pub id: TypeId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumTypeId {
    pub id: TypeId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeVarId {
    pub id: TypeId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LifetimeId {
    pub id: TypeId,
    pub name: String,
}

/// Registre des types pour créer et suivre les types
#[derive(Clone, Debug)]
pub struct TypeRegistry {
    pub types: HashMap<TypeId, Type>,
    pub next_type_id: u32,

    // Types primitifs pré-définis
    pub type_int: TypeId,
    pub type_float: TypeId,
    pub type_bool: TypeId,
    pub type_char: TypeId,
    pub type_string: TypeId,
    pub type_unit: TypeId,
    pub type_error: TypeId,
}

impl TypeRegistry {
    /// Crée un nouveau registre avec les types primitifs pré-définis
    pub fn new() -> Self {
        let mut registry = TypeRegistry {
            types: HashMap::new(),
            next_type_id: 1,
            type_int: TypeId(0),
            type_float: TypeId(0),
            type_bool: TypeId(0),
            type_char: TypeId(0),
            type_string: TypeId(0),
            type_unit: TypeId(0),
            type_error: TypeId(0),
        };

        // Créer les types primitifs
        registry.type_int = registry.register_type(TypeKind::Int);
        registry.type_float = registry.register_type(TypeKind::Float);
        registry.type_bool = registry.register_type(TypeKind::Bool);
        registry.type_char = registry.register_type(TypeKind::Char);
        registry.type_string = registry.register_type(TypeKind::String);
        registry.type_unit = registry.register_type(TypeKind::Unit);
        registry.type_error = registry.register_type(TypeKind::Error);

        registry
    }

    /// Enregistre un nouveau type et retourne son ID
    pub fn register_type(&mut self, kind: TypeKind) -> TypeId {
        let id = TypeId(self.next_type_id);
        self.next_type_id += 1;

        let type_obj = Type::new(id, kind);
        self.types.insert(id, type_obj);

        id
    }

    /// Récupère un type par son ID
    pub fn get_type(&self, id: TypeId) -> Option<&Type> {
        self.types.get(&id)
    }

    /// Récupère un type par son ID de manière mutable
    pub fn get_type_mut(&mut self, id: TypeId) -> Option<&mut Type> {
        self.types.get_mut(&id)
    }

    /// Crée un type array
    pub fn create_array_type(&mut self, element_type_id: TypeId, size: Option<usize>) -> TypeId {
        if let Some(element_type) = self.get_type(element_type_id).cloned() {
            self.register_type(TypeKind::Array(Box::new(element_type), size))
        } else {
            self.type_error
        }
    }

    /// Crée un type tuple
    pub fn create_tuple_type(&mut self, element_type_ids: Vec<TypeId>) -> TypeId {
        let mut element_types = Vec::with_capacity(element_type_ids.len());

        for type_id in element_type_ids {
            if let Some(element_type) = self.get_type(type_id).cloned() {
                element_types.push(element_type);
            } else {
                return self.type_error;
            }
        }

        self.register_type(TypeKind::Tuple(element_types))
    }

    /// Crée un type fonction
    pub fn create_function_type(&mut self, param_type_ids: Vec<TypeId>, return_type_id: TypeId) -> TypeId {
        let mut param_types = Vec::with_capacity(param_type_ids.len());

        for type_id in param_type_ids {
            if let Some(param_type) = self.get_type(type_id).cloned() {
                param_types.push(param_type);
            } else {
                return self.type_error;
            }
        }

        if let Some(return_type) = self.get_type(return_type_id).cloned() {
            let func_type = FunctionType {
                params: param_types,
                return_type: Box::new(return_type),
                lifetime_params: Vec::new(),
                type_params: Vec::new(),
            };
            self.register_type(TypeKind::Function(func_type))
        } else {
            self.type_error
        }
    }

    /// Crée un type référence
    pub fn create_reference_type(&mut self, inner_type_id: TypeId, mutability: Mutability, lifetime: Option<LifetimeId>) -> TypeId {
        if let Some(inner_type) = self.get_type(inner_type_id).cloned() {
            self.register_type(TypeKind::Reference(
                Box::new(inner_type),
                mutability,
                lifetime
            ))
        } else {
            self.type_error
        }
    }

    /// Convertit un type AST en TypeId
    pub fn convert_ast_type(&mut self, ast_type: &ASTType) -> TypeId {
        match ast_type {
            // Cas simples (types primitifs)
            ASTType::Int => self.type_int,
            ASTType::Float => self.type_float,
            ASTType::Bool => self.type_bool,
            ASTType::Char => self.type_char,
            ASTType::String => self.type_string,

            // Cas qui nécessitent des clones pour éviter les problèmes d'emprunt
            ASTType::Array(elem_type) => {
                // Convertir d'abord le type d'élément
                let elem_type_id = self.convert_ast_type(elem_type);
                self.create_array_type(elem_type_id, None)
            },

            ASTType::Tuple(elem_types) => {
                // Convertir tous les types d'éléments d'abord
                let elem_type_ids: Vec<TypeId> = elem_types.iter()
                    .map(|t| self.convert_ast_type(t))
                    .collect();
                self.create_tuple_type(elem_type_ids)
            },

            ASTType::Generic(generic_type) => {
                // Faire une conversion en deux étapes
                let type_params: Vec<TypeId> = generic_type.type_parameters.iter()
                    .map(|t| self.convert_ast_type(t))
                    .collect();

                // Maintenant, créer les types
                let type_args: Vec<Type> = type_params.iter()
                    .filter_map(|&id| self.get_type(id).cloned())
                    .collect();

                self.register_type(TypeKind::Named(generic_type.base.clone(), type_args))
            },

            // Autres cas...
            ASTType::Reference(inner_type) => {
                let inner_type_id = self.convert_ast_type(inner_type);
                self.create_reference_type(inner_type_id, Mutability::Immutable, None)
            },

            ASTType::ReferenceMutable(inner_type) => {
                let inner_type_id = self.convert_ast_type(inner_type);
                self.create_reference_type(inner_type_id, Mutability::Mutable, None)
            },

            ASTType::Named(name) => {
                self.register_type(TypeKind::Named(name.clone(), Vec::new()))
            },

            ASTType::Custom(name) => {
                self.register_type(TypeKind::Named(name.clone(), Vec::new()))
            },

            ASTType::Infer => {
                let type_var = TypeVarId {
                    id: TypeId(self.next_type_id),
                    name: format!("T{}", self.next_type_id),
                };
                self.register_type(TypeKind::Infer(type_var))
            },

            ASTType::SelfType => {
                self.register_type(TypeKind::SelfType)
            },
        }
    }
}

/// Système d'unification pour l'inférence de types
#[derive(Debug, Clone)]
pub struct TypeSystem {
    pub type_registry: TypeRegistry,

    // Mappages pour l'inférence de types
    type_variables: HashMap<TypeVarId, Option<Type>>,

    // Contraintes de types
    type_constraints: Vec<TypeConstraint>,

    // Compteurs pour générer des identifiants uniques
    next_type_var_id: u32,
    next_lifetime_id: u32,
}

impl TypeSystem {
    pub fn new() -> Self {
        TypeSystem {
            type_registry: TypeRegistry::new(),
            type_variables: HashMap::new(),
            type_constraints: Vec::new(),
            next_type_var_id: 1,
            next_lifetime_id: 1,
        }
    }

    /// Unifie deux types, en résolvant les variables de type
    pub fn unify(&mut self, t1: &Type, t2: &Type) -> Result<Type, TypeError> {
        match (&t1.kind, &t2.kind) {
            // Cas de base: types identiques
            (a, b) if a == b => Ok(t1.clone()),

            // Cas avec variable de type
            (TypeKind::Infer(id), _) => {
                self.unify_var(id.clone(), t2.clone())
            },
            (_, TypeKind::Infer(id)) => {
                self.unify_var(id.clone(), t1.clone())
            },

            // Cas avec types structurellement similaires
            (TypeKind::Array(elem1, size1), TypeKind::Array(elem2, size2)) => {
                let unified_elem = self.unify(elem1, elem2)?;
                if size1 != size2 {
                    return Err(TypeError::TypeMismatch(format!(
                        "Array size mismatch: {:?} vs {:?}", size1, size2
                    )));
                }
                Ok(Type::new(
                    TypeId(0), // Sera assigné par le registry
                    TypeKind::Array(Box::new(unified_elem), *size1)
                ))
            },

            (TypeKind::Tuple(types1), TypeKind::Tuple(types2)) => {
                if types1.len() != types2.len() {
                    return Err(TypeError::TypeMismatch(format!(
                        "Tuple length mismatch: {} vs {}", types1.len(), types2.len()
                    )));
                }

                let mut unified_types = Vec::new();
                for (t1, t2) in types1.iter().zip(types2.iter()) {
                    unified_types.push(self.unify(t1, t2)?);
                }

                Ok(Type::new(
                    TypeId(0), // Sera assigné par le registry
                    TypeKind::Tuple(unified_types)
                ))
            },

            // Cas par défaut: types incompatibles
            _ => Err(TypeError::TypeMismatch(format!(
                "Cannot unify types {} and {}", t1, t2
            ))),
        }
    }

    /// Unifie une variable de type avec un type
    fn unify_var(&mut self, id: TypeVarId, typ: Type) -> Result<Type, TypeError> {
        // Vérifier si la variable a déjà une valeur
        let existing_type_option = self.type_variables.get(&id).cloned().flatten();

        if let Some(existing) = existing_type_option {
            // Cloner pour éviter les conflits d'emprunt
            let existing_clone = existing.clone();
            return self.unify(&existing_clone, &typ);
        }

        // Vérifier l'occurrence de la variable dans le type (pour éviter les types récursifs)
        if self.occurs_check(&id, &typ) {
            return Err(TypeError::TypeMismatch("Recursive type detected".to_string()));
        }

        // Assigner le type à la variable
        self.type_variables.insert(id, Some(typ.clone()));
        Ok(typ)
    }

    /// Vérifie si une variable de type apparaît dans un type (occurs check)
    fn occurs_check(&self, id: &TypeVarId, typ: &Type) -> bool {
        match &typ.kind {
            TypeKind::Infer(var_id) => *var_id == *id,
            TypeKind::Array(elem, _) => self.occurs_check(id, elem),
            TypeKind::Tuple(types) => types.iter().any(|t| self.occurs_check(id, t)),
            TypeKind::Reference(inner, _, _) => self.occurs_check(id, inner),
            TypeKind::Function(func_type) => {
                func_type.params.iter().any(|t| self.occurs_check(id, t)) ||
                    self.occurs_check(id, &func_type.return_type)
            },
            _ => false,
        }
    }

    /// Crée une nouvelle variable de type pour l'inférence
    pub fn create_type_variable(&mut self, name: Option<String>) -> TypeVarId {
        let id = TypeVarId {
            id: TypeId(self.next_type_var_id),
            name: name.unwrap_or_else(|| format!("T{}", self.next_type_var_id)),
        };
        self.next_type_var_id += 1;
        self.type_variables.insert(id.clone(), None);
        id
    }

    /// Résout une variable de type si elle a été instanciée
    pub fn resolve_type_variable(&self, id: &TypeVarId) -> Option<&Type> {
        self.type_variables.get(id).and_then(|opt| opt.as_ref())
    }
}








//*//*////////////////////////////////////////////////////////


// //src/semantic/types/type_system.rs
// // use std::any::TypeId;
// use std::collections::HashMap;
// use crate::parser::ast::{Assignment, BinaryOperation, Expression, Literal, Operator, UnaryOperation, UnaryOperator};
// use crate::semantic::semantic_error::{Position, SemanticError, SemanticErrorType};
// use crate::semantic::semantic_error::SemanticErrorType::{SymbolError, TypeError};
// use crate::semantic::symbol_table::SymbolTable;
// use crate::semantic::symbols::SymbolId;
//
//
//
// /// Identifiant unique pour les types
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub struct TypeId(pub u32);
// #[derive(Debug, Clone, PartialEq)]
// pub enum TypeKind {
//     // Types primitifs
//     Int,
//     Float,
//     Bool,
//     Char,
//     String,
//
//     // Conteneurs et structures
//     Array(Box<Type>, Option<usize>),  // Type, longueur optionnelle
//     Tuple(Vec<Type>),
//     Struct(StructTypeId),
//     Enum(EnumTypeId),
//
//     // Types pour les fonctions
//     Function(FunctionType),
//
//     // Types pour les références
//     Reference(Box<Type>, Mutability, Option<LifetimeId>),
//
//     // Type générique
//     Generic(String, Vec<TypeConstraint>),
//
//     // Type polymorphique (ex: auto, var, _)
//     Infer(TypeVarId),
//
//     // Type utilisateur
//     Named(String, Vec<Type>),  // Nom du type, arguments génériques
//
//     // Trait comme Contraintes
//     TraitBound(SymbolId),
//
//     // Type d'une méthode associée à un self
//     SelfType,
//
//     // Type "unit" pour les fonctions sans retour
//     Unit,
//
//     // Type pour les valeurs impossibles (jamais atteintes)
//     Never,
//
//     // Type pour représenter une erreur
//     Error
// }
//
// /// Structure d'un Type
// #[derive(Debug,Clone,PartialEq)]
// pub struct Type{
//     pub id: TypeId,
//     pub kind: TypeKind,
//     pub nullable: bool,
// }
//
//
//
// #[derive(Debug, Clone, PartialEq)]
// pub enum Mutability {
//     Mutable,
//     Immutable,
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct FunctionType {
//     pub params: Vec<Type>,
//     pub return_type: Box<Type>,
//     pub lifetime_params: Vec<LifetimeId>,
//     pub type_params: Vec<TypeVarId>,
// }
//
// #[allow(dead_code)]
// #[derive(Debug, Clone, PartialEq)]
// pub enum TypeConstraint {
//     Equal(Type, Type),
//     Subtype(Type, Type),
//     Instance(Type, Vec<Type>),
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct StructTypeId {
//     pub id: TypeId,
//     pub name: String,
//
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct EnumTypeId {
//     pub id: TypeId,
//     pub name: String,
// }
//
// #[derive(Debug, Clone, PartialEq,Eq, Hash)]
// pub struct TypeVarId {
//     pub id: TypeId,
//     pub name: String,
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct LifetimeId {
//     pub id: TypeId,
//     pub name: String,
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct TraitId {
//     pub id: TypeId,
//     pub name: String,
// }
//
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct TypeDefinition {
//     pub id: TypeId,
//     pub name: String,
//     pub kind: TypeKind,
//     pub type_params: Vec<TypeVarId>,
//     pub lifetime_params: Vec<LifetimeId>,
//     pub fields: Vec<(String, Type)>,  // Nom du champ et son type
// }
//
//
//
//
// pub struct TypeSystem {
//     // Table des types nommés (structs, enums, etc.)
//     named_types: HashMap<String, TypeId>,
//
//     // Table des définitions de types
//     type_definitions: HashMap<TypeId, TypeDefinition>,
//
//     // Mappages pour l'inférence de types
//     type_variables: HashMap<TypeVarId, Option<Type>>,
//
//     // Contraintes de types
//     type_constraints: Vec<TypeConstraint>,
//
//     // Compteurs pour générer des identifiants uniques
//     next_type_id: u32,
//     next_type_var_id: u32,
//     next_lifetime_id: u32,
// }
//
//
// impl TypeSystem {
//     // Unifie deux types, en résolvant les variables de type
//     pub fn unify(&mut self, t1: &Type, t2: &Type) -> Result<Type, TypeError> {
//         match (t1, t2) {
//             // Cas de base: types identiques
//             (a, b) if a == b => Ok(a.clone()),
//
//             // Cas avec variable de type
//             (TypeKind::Infer(id), other) | (other, TypeKind::Infer(id)) => {
//                 self.unify_var(*id.clone(), other.clone())
//             },
//
//             // Cas avec types structurellement similaires
//             (TypeKind::Array(elem1, size1), TypeKind::Array(elem2, size2)) => {
//                 let unified_elem = self.unify(elem1, elem2)?;
//                 if size1 != size2 {
//                     return Err(TypeError::SizeMismatch(*size1, *size2));
//                 }
//                 Ok(Type::Array(Box::new(unified_elem), *size1))
//             },
//
//             // Autres cas similaires pour Tuple, Function, etc.
//
//             // Cas par défaut: types incompatibles
//             _ => Err(TypeError::TypeMismatch(t1.clone(), t2.clone())),
//         }
//     }
//
//     // Unifie une variable de type avec un type
//     fn unify_var(&mut self, id: TypeVarId, typ: Type) -> Result<Type, TypeError> {
//         // Vérifier si la variable a déjà une valeur
//         if let Some(Some(existing)) = self.type_variables.get(&id) {
//             return self.unify(existing, &typ);
//         }
//
//         // Vérifier l'occurrence de la variable dans le type (pour éviter les types récursifs)
//         if self.occurs_check(id.clone(), &typ) {
//             return Err(TypeError::RecursiveType);
//         }
//
//         // Assigner le type à la variable
//         self.type_variables.insert(id, Some(typ.clone()));
//         Ok(typ)
//     }
//
//     // Vérifie si une variable de type apparaît dans un type (occurs check)
//     fn occurs_check(&self, id: TypeVarId, typ: &Type) -> bool {
//         match typ {
//             Type::Infer(var_id) => *var_id == id,
//             Type::Array(elem, _) => self.occurs_check(id, elem),
//             Type::Tuple(types) => types.iter().any(|t| self.occurs_check(id, t)),
//             // Autres cas...
//             _ => false,
//         }
//     }
// }
//
//
// /// Registre des types pour créer et suivre les types
// pub struct TypeRegistry {
//     types: HashMap<TypeId, Type>,
//     next_type_id: u32,
//
//     // Types primitifs pré-définis
//     pub type_int: TypeId,
//     pub type_float: TypeId,
//     pub type_bool: TypeId,
//     pub type_char: TypeId,
//     pub type_string: TypeId,
//     pub type_void: TypeId,
//     pub type_error: TypeId,
// }
//
// impl TypeRegistry {
//     /// Crée un nouveau registre avec les types primitifs pré-définis
//     pub fn new() -> Self {
//         let mut registry = TypeRegistry {
//             types: HashMap::new(),
//             next_type_id: 1,
//             type_int: TypeId(0),
//             type_float: TypeId(0),
//             type_bool: TypeId(0),
//             type_char: TypeId(0),
//             type_string: TypeId(0),
//             type_void: TypeId(0),
//             type_error: TypeId(0),
//         };
//
//         // Créer les types primitifs
//         registry.type_int = registry.register_type(TypeKind::Int);
//         registry.type_float = registry.register_type(TypeKind::Float);
//         registry.type_bool = registry.register_type(TypeKind::Bool);
//         registry.type_char = registry.register_type(TypeKind::Char);
//         registry.type_string = registry.register_type(TypeKind::String);
//         registry.type_void = registry.register_type(TypeKind::Tuple(vec![])); // Unit type ()
//         registry.type_error = registry.register_type(TypeKind::Error);
//
//         registry
//     }
//
//     /// Enregistre un nouveau type et retourne son ID
//     pub fn register_type(&mut self, kind: TypeKind) -> TypeId {
//         let id = TypeId(self.next_type_id);
//         self.next_type_id += 1;
//
//         let type_obj = Type::new(id, kind);
//         self.types.insert(id, type_obj);
//
//         id
//     }
//
//     /// Récupère un type par son ID
//     pub fn get_type(&self, id: TypeId) -> Option<&Type> {
//         self.types.get(&id)
//     }
//
//     /// Récupère un type par son ID de manière mutable
//     pub fn get_type_mut(&mut self, id: TypeId) -> Option<&mut Type> {
//         self.types.get_mut(&id)
//     }
//
//     /// Crée un type array
//     pub fn create_array_type(&mut self, element_type_id: TypeId) -> TypeId {
//         if let Some(element_type) = self.get_type(element_type_id).cloned() {
//             self.register_type(TypeKind::Array(Box::new(element_type)))
//         } else {
//             self.type_error
//         }
//     }
//
//     /// Crée un type tuple
//     pub fn create_tuple_type(&mut self, element_type_ids: Vec<TypeId>) -> TypeId {
//         let mut element_types = Vec::with_capacity(element_type_ids.len());
//
//         for type_id in element_type_ids {
//             if let Some(element_type) = self.get_type(type_id).cloned() {
//                 element_types.push(element_type);
//             } else {
//                 return self.type_error;
//             }
//         }
//
//         self.register_type(TypeKind::Tuple(element_types))
//     }
//
//     /// Crée un type fonction
//     pub fn create_function_type(&mut self, param_type_ids: Vec<TypeId>, return_type_id: TypeId) -> TypeId {
//         let mut param_types = Vec::with_capacity(param_type_ids.len());
//
//         for type_id in param_type_ids {
//             if let Some(param_type) = self.get_type(type_id).cloned() {
//                 param_types.push(param_type);
//             } else {
//                 return self.type_error;
//             }
//         }
//
//         if let Some(return_type) = self.get_type(return_type_id).cloned() {
//             self.register_type(TypeKind::Function {
//                 params: param_types,
//                 return_type: Box::new(return_type),
//             })
//         } else {
//             self.type_error
//         }
//     }
//
//     /// Crée un type référence
//     pub fn create_reference_type(&mut self, inner_type_id: TypeId, is_mutable: bool, lifetime: Option<String>) -> TypeId {
//         if let Some(inner_type) = self.get_type(inner_type_id).cloned() {
//             self.register_type(TypeKind::Reference {
//                 inner: Box::new(inner_type),
//                 is_mutable,
//                 lifetime,
//             })
//         } else {
//             self.type_error
//         }
//     }
// }
//
// impl SymbolTable{
//     /// Définit le type d'un symbole
//     pub fn set_symbol_type(&mut self, symbol_id: SymbolId, type_id: TypeId) -> Result<(), SemanticError> {
//         if let Some(symbol) = self.symbols.get_mut(&symbol_id) {
//             // Mettre à jour le type du symbole
//             symbol.attributes.inferred_type = Some(self.type_registry.get_type(type_id)
//                 .ok_or_else(||
//                     TypeError(TypeError::TypeNotFound(format!("{:?}", type_id))),
//                     "Type not found".to_string(),
//                     Position { index: 0 }
//                 ))?.clone();
//             Ok(())
//         } else {
//             Err(
//                 SymbolError::SymbolNotFound(format!("{:?}", symbol_id),
//                 Position { index: 0 }
//             ))
//         }
//     }
//
//     /// Récupère le type d'un symbole
//     pub fn get_symbol_type(&self, symbol_id: SymbolId) -> Result<Option<&Type>, SemanticError> {
//         let symbol = self.get_symbol(symbol_id)?;
//         Ok(symbol.attributes.inferred_type.as_ref())
//     }
//     //
// }
//
//
//
//
// //
// //
// //
// // impl TypeChecker {
// //     pub fn infer_expression(&mut self, expr: &Expression) -> Result<Type, String> {
// //         match expr {
// //             Expression::Literal(lit) => self.infer_literal(lit),
// //             Expression::Identifier(name) => self.lookup_type(name),
// //             Expression::BinaryOperation(binop) => self.infer_binary_op(binop),
// //             Expression::Assignment(assign) => self.infer_assignment(assign),
// //             Expression::UnaryOperation(unop) => self.infer_unary_op(unop),
// //             _ => Ok(Type::Infer), // Pour les autres cas
// //         }
// //     }
// //
// //     fn infer_literal(&self, lit: &Literal) -> Result<crate::parser::ast::Type, String> {
// //         match lit {
// //             Literal::Integer { .. } => Ok(crate::parser::ast::Type::Int),
// //             Literal::Float { .. } => Ok(crate::parser::ast::Type::Float),
// //             // Literal::String(_) => Ok(Type::String),
// //             Literal::String(s) => {
// //                 // Si c'est un seul caractère entre guillemets simples
// //                 if s.len() == 1 && s.starts_with('\'') && s.ends_with('\'') {
// //                     Ok(crate::parser::ast::Type::Char)
// //                 } else {
// //                     Ok(crate::parser::ast::Type::String)
// //                 }
// //             }
// //             Literal::Boolean(_) => Ok(crate::parser::ast::Type::Bool),
// //             Literal::Char(_) => Ok(crate::parser::ast::Type::Char), // Ajout  l'inference de type pour les caractères
// //             _ => Ok(crate::parser::ast::Type::Infer),
// //         }
// //     }
// //
// //     fn infer_binary_op(&mut self, binop: &BinaryOperation) -> Result<crate::parser::ast::Type, String> {
// //         let left_type = self.infer_expression(&binop.left)?;
// //         let right_type = self.infer_expression(&binop.right)?;
// //
// //         match binop.operator {
// //             Operator::Addition | Operator::Substraction |
// //             Operator::Multiplication | Operator::Division => {
// //                 if left_type == crate::parser::ast::Type::Int && right_type == crate::parser::ast::Type::Int {
// //                     Ok(crate::parser::ast::Type::Int)
// //                 } else if left_type == crate::parser::ast::Type::Float || right_type == crate::parser::ast::Type::Float {
// //                     Ok(crate::parser::ast::Type::Float)
// //                 } else if left_type == crate::parser::ast::Type::Infer || right_type == crate::parser::ast::Type::Infer {
// //                     Ok(crate::parser::ast::Type::Infer)
// //                 } else {
// //                     Err("Type mismatch in binary operation".to_string())
// //                 }
// //             },
// //             Operator::Equal | Operator::NotEqual |
// //             Operator::LessThan | Operator::GreaterThan |
// //             Operator::LesshanOrEqual | Operator::GreaterThanOrEqual => {
// //                 self.add_constraint(TypeConstraint::Equal(left_type, right_type));
// //                 Ok(crate::parser::ast::Type::Bool)
// //             },
// //             _ => Ok(crate::parser::ast::Type::Infer),
// //         }
// //     }
// //
// //     fn infer_assignment(&mut self, assign: &Assignment) -> Result<crate::parser::ast::Type, String> {
// //         let value_type = self.infer_expression(&assign.value)?;
// //
// //         match &*assign.target {
// //             Expression::Identifier(name) => {
// //                 self.type_vars.insert(name.clone(), value_type.clone());
// //                 Ok(value_type)
// //             },
// //             _ => Err("Invalid assignment target".to_string())
// //         }
// //     }
// //
// //
// //     fn infer_unary_op(&mut self, unop: &UnaryOperation) -> Result<crate::parser::ast::Type, String> {
// //         let operand_type = self.infer_expression(&unop.operand)?;
// //
// //         match unop.operator {
// //             UnaryOperator::Negative => {
// //                 match operand_type {
// //                     crate::parser::ast::Type::Int => Ok(crate::parser::ast::Type::Int),
// //                     crate::parser::ast::Type::Float => Ok(crate::parser::ast::Type::Float),
// //                     crate::parser::ast::Type::Infer => Ok(crate::parser::ast::Type::Infer),
// //                     _ => Err("Operator '-' cannot be applied to this type".to_string())
// //                 }
// //             },
// //             UnaryOperator::Not => {
// //                 match operand_type {
// //                     crate::parser::ast::Type::Bool => Ok(crate::parser::ast::Type::Bool),
// //                     crate::parser::ast::Type::Infer => Ok(crate::parser::ast::Type::Infer),
// //                     _ => Err("Operator '!' can only be applied to boolean types".to_string())
// //                 }
// //             },
// //             UnaryOperator::Reference => Ok(crate::parser::ast::Type::Array(Box::new(operand_type))),
// //             UnaryOperator::ReferenceMutable => Ok(crate::parser::ast::Type::Array(Box::new(operand_type))),
// //             _ => todo!(),
// //         }
// //     }
// //
// //
// //
// //     pub fn check_assignment(&mut self, target: &Expression, value: &Expression) -> Result<(), TypeError> {
// //         // 1. Vérifier que la cible est assignable (lvalue)
// //         if !self.is_lvalue(target) {
// //             return Err(TypeError::NotAssignable);
// //         }
// //
// //         // 2. Vérifier que la cible est mutable
// //         self.check_mutability(target)?;
// //
// //         // 3. Inférer les types
// //         let target_type = self.infer_expression(target)?;
// //         let value_type = self.infer_expression(value)?;
// //
// //         // 4. Vérifier la compatibilité des types
// //         self.type_system.unify(&target_type, &value_type)
// //             .map(|_| ())
// //             .map_err(|_| TypeError::AssignmentMismatch(target_type, value_type))
// //     }
// //
// //     fn is_lvalue(&self, expr: &Expression) -> bool {
// //         match expr {
// //             Expression::Identifier(_) => true,
// //             Expression::Literal(_) => false,
// //             Expression::BinaryOperation(_) => false,
// //             Expression::UnaryOperation(_) => false,
// //             Expression::Assignment(_) => false,
// //             Expression::MethodCall(_) => false,
// //
// //             // Expression::Variable(_) => true,
// //             // Expr::FieldAccess(_, _) => true,
// //             // Expr::Index(_, _) => true,
// //             // D'autres expressions qui peuvent être à gauche d'une affectation
// //             _ => false,
// //         }
// //     }
// //
// //     fn check_mutability(&self, expr: &Expression) -> Result<(), TypeError> {
// //         match expr {
// //             Expression::Variable(name) => {
// //                 let symbol_id = self.symbol_table.lookup_symbol(name)?;
// //                 if !self.symbol_table.is_mutable(symbol_id)? {
// //                     return Err(TypeError::ImmutableAssignment(name.clone()));
// //                 }
// //                 Ok(())
// //             },
// //             // Autres cas...
// //             _ => Ok(()),
// //         }
// //     }
// // }
// //
