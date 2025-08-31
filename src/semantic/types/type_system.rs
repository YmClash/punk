//src/semantic/types/type_system.rs

use std::collections::HashMap;
use std::{fmt,marker::Sized};
use crate::parser::ast::{Type as ASTType};
use crate::semantic::semantic_error::{TypeError};
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
    // Infer(TypeVarId),
    Infer(TypeVarId),  // Variable de type pour l'inférence

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

#[derive(Debug, Clone, Copy, PartialEq)]
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
    pub is_variadic: bool,
    pub lifetimes: Vec<LifetimeId>,  // Alias pour lifetime_params pour compatibilité
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LifetimeId {
    pub id: TypeId,
    pub name: String,
}

/// Information sur un lifetime
#[derive(Debug, Clone)]
pub struct LifetimeInfo {
    pub id: LifetimeId,
    pub bounds: Vec<LifetimeId>,  // Lifetimes qui doivent outlive ce lifetime
    pub is_static: bool,
}

// TypeRegistry a été unifié avec TypeSystem - voir la structure TypeSystem plus haut
// Toutes les fonctionnalités de TypeRegistry sont maintenant intégrées dans TypeSystem

/// Système unifié de gestion des types et d'inférence
/// Unifie TypeRegistry et TypeSystem en un seul composant cohérent
#[derive(Debug, Clone)]
pub struct TypeSystem {
    // ===== Registre des types (anciennement TypeRegistry) =====
    /// Tous les types enregistrés
    pub types: HashMap<TypeId, Type>,
    /// Prochain ID de type disponible
    next_type_id: u32,
    
    // Types primitifs pré-définis
    pub type_int: TypeId,
    pub type_float: TypeId,
    pub type_bool: TypeId,
    pub type_char: TypeId,
    pub type_string: TypeId,
    pub type_unit: TypeId,
    pub type_error: TypeId,
    pub type_never: TypeId,
    
    // ===== Système d'inférence =====
    /// Variables de type pour l'inférence
    type_variables: HashMap<TypeVarId, Option<Type>>,
    /// Contraintes de types collectées
    type_constraints: Vec<TypeConstraint>,
    /// Prochain ID de variable de type
    next_type_var_id: u32,
    
    // ===== Gestion des lifetimes =====
    /// Lifetimes enregistrées
    lifetimes: HashMap<LifetimeId, LifetimeInfo>,
    /// Prochain ID de lifetime
    next_lifetime_id: u32,
    
    // ===== Cache et optimisations =====
    /// Cache pour les types canoniques (utilise une représentation string car TypeKind n'implémente pas Hash)
    canonical_cache: HashMap<String, TypeId>,
    /// Cache pour les relations de sous-typage
    subtype_cache: HashMap<(TypeId, TypeId), bool>,
}

impl TypeSystem {
    pub fn new() -> Self {
        let mut system = TypeSystem {
            types: HashMap::new(),
            next_type_id: 1,
            type_int: TypeId(0),
            type_float: TypeId(0),
            type_bool: TypeId(0),
            type_char: TypeId(0),
            type_string: TypeId(0),
            type_unit: TypeId(0),
            type_error: TypeId(0),
            type_never: TypeId(0),
            type_variables: HashMap::new(),
            type_constraints: Vec::new(),
            next_type_var_id: 1,
            lifetimes: HashMap::new(),
            next_lifetime_id: 1,
            canonical_cache: HashMap::new(),
            subtype_cache: HashMap::new(),
        };
        
        // Initialiser les types primitifs
        system.type_int = system.register_type(TypeKind::Int);
        system.type_float = system.register_type(TypeKind::Float);
        system.type_bool = system.register_type(TypeKind::Bool);
        system.type_char = system.register_type(TypeKind::Char);
        system.type_string = system.register_type(TypeKind::String);
        system.type_unit = system.register_type(TypeKind::Unit);
        system.type_error = system.register_type(TypeKind::Error);
        system.type_never = system.register_type(TypeKind::Never);
        
        system
    }
    
    /// Réinitialise le système de types
    pub fn clear(&mut self) {
        // Sauvegarder puis restaurer les types primitifs
        let new_system = TypeSystem::new();
        *self = new_system;
    }
    
    /// Valide le système de types
    pub fn validate(&self) -> Result<(), TypeError> {
        // TODO: Implémenter la validation
        Ok(())
    }
    
    /// Compte le nombre de types enregistrés
    pub fn type_count(&self) -> usize {
        self.types.len()
    }
    
    /// Vérifie si deux types sont compatibles
    pub fn are_types_compatible(&self, t1: TypeId, t2: TypeId) -> bool {
        // Vérifier le cache d'abord
        if let Some(&result) = self.subtype_cache.get(&(t1, t2)) {
            return result;
        }
        
        if t1 == t2 {
            return true;
        }
        
        let type1 = self.get_type(t1);
        let type2 = self.get_type(t2);
        
        match (type1, type2) {
            (Some(t1), Some(t2)) => t1.is_compatible_with(t2),
            _ => false,
        }
    }

    /// Unifie deux types, en résolvant les variables de type
    pub fn unify(&mut self, t1: &Type, t2: &Type) -> Result<Type, TypeError> {
        match (&t1.kind, &t2.kind) {
            // Cas de base: types identiques
            (a, b) if *a == *b => Ok(t1.clone()),

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
                if *size1 != *size2 {
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
    
    // ===== Méthodes héritées de TypeRegistry =====
    
    /// Enregistre un nouveau type et retourne son ID
    pub fn register_type(&mut self, kind: TypeKind) -> TypeId {
        // Créer une clé string pour le cache
        let cache_key = format!("{:?}", kind);
        
        // Vérifier le cache canonique d'abord
        if let Some(&cached_id) = self.canonical_cache.get(&cache_key) {
            return cached_id;
        }
        
        let id = TypeId(self.next_type_id);
        self.next_type_id += 1;
        
        let type_obj = Type::new(id, kind);
        self.types.insert(id, type_obj);
        
        // Ajouter au cache canonique pour éviter les doublons
        self.canonical_cache.insert(cache_key, id);
        
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
            self.register_type(TypeKind::Function(FunctionType {
                params: param_types,
                return_type: Box::new(return_type),
                lifetime_params: Vec::new(),
                type_params: Vec::new(),
                is_variadic: false,
                lifetimes: Vec::new(),
            }))
        } else {
            self.type_error
        }
    }
    
    /// Crée un type référence
    pub fn create_reference_type(&mut self, inner_type_id: TypeId, mutability: Mutability, lifetime: Option<LifetimeId>) -> TypeId {
        if let Some(inner_type) = self.get_type(inner_type_id).cloned() {
            self.register_type(TypeKind::Reference(Box::new(inner_type), mutability, lifetime))
        } else {
            self.type_error
        }
    }
    
    /// Crée un type nommé (type défini par l'utilisateur)
    pub fn create_named_type(&mut self, name: String, type_args: Vec<TypeId>) -> TypeId {
        let mut args = Vec::with_capacity(type_args.len());
        
        for type_id in type_args {
            if let Some(arg_type) = self.get_type(type_id).cloned() {
                args.push(arg_type);
            } else {
                return self.type_error;
            }
        }
        
        self.register_type(TypeKind::Named(name, args))
    }
    
    /// Convertit un type AST en type du système  
    pub fn from_ast_type(&mut self, ast_type: &ASTType) -> TypeId {
        match ast_type {
            ASTType::Int => self.type_int,
            ASTType::Float => self.type_float,
            ASTType::Bool => self.type_bool,
            ASTType::Char => self.type_char,
            ASTType::String => self.type_string,
            // ASTType::Unit n'existe pas dans l'AST, utiliser un cas par défaut
            // ASTType::Unit => self.type_unit,
            ASTType::Array(element_type) => {
                let element_type_id = self.from_ast_type(element_type);
                self.create_array_type(element_type_id, None)  // Taille non spécifiée dans l'AST
            },
            ASTType::Tuple(types) => {
                let type_ids: Vec<TypeId> = types.iter().map(|t| self.from_ast_type(t)).collect();
                self.create_tuple_type(type_ids)
            },
            // ASTType::Function n'existe pas dans l'AST actuel
            // Pour gérer les types de fonction, il faudrait les ajouter à l'AST
            ASTType::Named(name) => {
                // Named n'a pas d'arguments de type dans l'AST actuel
                self.create_named_type(name.clone(), Vec::new())
            },
            ASTType::Reference(inner_type) => {
                let inner_id = self.from_ast_type(inner_type);
                self.create_reference_type(inner_id, Mutability::Immutable, None)
            },
            ASTType::ReferenceMutable(inner_type) => {
                let inner_id = self.from_ast_type(inner_type);
                self.create_reference_type(inner_id, Mutability::Mutable, None)
            },
            ASTType::Generic(generic_type) => {
                // Generic contient un GenericType avec base et type_parameters
                // Pour l'instant, créer un type nommé avec les paramètres
                let base_name = &generic_type.base;
                let type_var = self.create_type_variable(Some(base_name.clone()));
                self.register_type(TypeKind::Infer(type_var))
            },
            ASTType::SelfType => {
                self.register_type(TypeKind::SelfType)
            },
            ASTType::Custom(name) => {
                // Custom est un type défini par l'utilisateur
                self.create_named_type(name.clone(), Vec::new())
            },
            ASTType::Infer => {
                // Type à inférer
                let type_var = self.create_type_variable(None);
                self.register_type(TypeKind::Infer(type_var))
            },
        }
    }
    
    // ===== Gestion des lifetimes =====
    
    /// Crée un nouveau lifetime
    pub fn create_lifetime(&mut self, name: Option<String>) -> LifetimeId {
        let id = LifetimeId {
            id: TypeId(self.next_lifetime_id + 1000000), // Offset pour éviter les collisions avec les TypeId
            name: name.unwrap_or_else(|| format!("'l{}", self.next_lifetime_id)),
        };
        self.next_lifetime_id += 1;
        
        let info = LifetimeInfo {
            id: id.clone(),
            bounds: Vec::new(),
            is_static: false,
        };
        
        self.lifetimes.insert(id.clone(), info);
        id
    }
    
    /// Crée le lifetime 'static
    pub fn static_lifetime(&mut self) -> LifetimeId {
        let id = LifetimeId {
            id: TypeId(0),
            name: "'static".to_string(),
        };
        
        if !self.lifetimes.contains_key(&id) {
            let info = LifetimeInfo {
                id: id.clone(),
                bounds: Vec::new(),
                is_static: true,
            };
            self.lifetimes.insert(id.clone(), info);
        }
        
        id
    }
    
    /// Ajoute une contrainte de lifetime (l1 outlives l2)
    pub fn add_lifetime_bound(&mut self, l1: LifetimeId, l2: LifetimeId) {
        if let Some(info) = self.lifetimes.get_mut(&l1) {
            if !info.bounds.contains(&l2) {
                info.bounds.push(l2);
            }
        }
    }
    
    /// Invalide le cache de sous-typage
    pub fn invalidate_subtype_cache(&mut self) {
        self.subtype_cache.clear();
    }
    
    /// Alias pour from_ast_type pour compatibilité avec l'ancien TypeRegistry
    pub fn convert_ast_type(&mut self, ast_type: &ASTType) -> TypeId {
        self.from_ast_type(ast_type)
    }
}






#[cfg(test)]
mod tests {
    use super::*;

    // Tests pour TypeSystem unifié (anciennement TypeRegistry)
    #[test]
    fn test_type_system_creation() {
        let system = TypeSystem::new();

        // Vérifier que les types primitifs sont créés
        assert!(system.get_type(system.type_int).is_some());
        assert!(system.get_type(system.type_float).is_some());
        assert!(system.get_type(system.type_bool).is_some());
        assert!(system.get_type(system.type_char).is_some());
        assert!(system.get_type(system.type_string).is_some());
    }

    #[test]
    fn test_array_type_creation() {
        let mut system = TypeSystem::new();

        // Créer un type array d'entiers
        let array_type_id = system.create_array_type(system.type_int, Some(5));

        // Vérifier le type créé
        let array_type = system.get_type(array_type_id).unwrap();
        match &array_type.kind {
            TypeKind::Array(elem_type, size) => {
                assert_eq!(elem_type.id, system.type_int);
                assert_eq!(*size, Some(5));
            },
            _ => panic!("Expected array type"),
        }
    }

    #[test]
    fn test_tuple_type_creation() {
        let mut system = TypeSystem::new();

        // Créer un tuple (int, float)
        let tuple_type_id = system.create_tuple_type(vec![system.type_int, system.type_float]);

        // Vérifier le type créé
        let tuple_type = system.get_type(tuple_type_id).unwrap();
        match &tuple_type.kind {
            TypeKind::Tuple(types) => {
                assert_eq!(types.len(), 2);
                assert_eq!(types[0].id, system.type_int);
                assert_eq!(types[1].id, system.type_float);
            },
            _ => panic!("Expected tuple type"),
        }
    }

    // Tests pour TypeSystem
    #[test]
    fn test_type_system_unification() {
        let mut type_system = TypeSystem::new();

        // Créer deux types identiques
        let t1 = Type::new(TypeId(1), TypeKind::Int);
        let t2 = Type::new(TypeId(2), TypeKind::Int);

        // Vérifier l'unification
        let result = type_system.unify(&t1, &t2);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_variable_inference() {
        let mut type_system = TypeSystem::new();

        // Créer une variable de type
        let type_var_id = type_system.create_type_variable(Some("T".to_string()));
        let infer_type = Type::new(TypeId(1), TypeKind::Infer(type_var_id.clone()));

        // Unifier avec un type concret
        let concrete_type = Type::new(TypeId(2), TypeKind::Int);
        let result = type_system.unify(&infer_type, &concrete_type);

        assert!(result.is_ok());
        assert_eq!(type_system.resolve_type_variable(&type_var_id).unwrap().kind, TypeKind::Int);
    }

    #[test]
    fn test_reference_type_creation() {
        let mut system = TypeSystem::new();

        // Créer une référence mutable vers un int
        let ref_type_id = system.create_reference_type(
            system.type_int,
            Mutability::Mutable,
            None
        );

        // Vérifier le type créé
        let ref_type = system.get_type(ref_type_id).unwrap();
        match &ref_type.kind {
            TypeKind::Reference(inner, mutability, lifetime) => {
                assert_eq!(inner.id, system.type_int);
                assert_eq!(*mutability, Mutability::Mutable);
                assert!(lifetime.is_none());
            },
            _ => panic!("Expected reference type"),
        }
    }

    #[test]
    fn test_function_type_creation() {
        let mut system = TypeSystem::new();

        // Créer un type fonction (int, float) -> bool
        let func_type_id = system.create_function_type(
            vec![system.type_int, system.type_float],
            system.type_bool
        );

        // Vérifier le type créé
        let func_type = system.get_type(func_type_id).unwrap();
        match &func_type.kind {
            TypeKind::Function(ft) => {
                assert_eq!(ft.params.len(), 2);
                assert_eq!(ft.params[0].id, system.type_int);
                assert_eq!(ft.params[1].id, system.type_float);
                assert_eq!(ft.return_type.id, system.type_bool);
            },
            _ => panic!("Expected function type"),
        }
    }

    #[test]
    fn test_incompatible_type_unification() {
        let mut type_system = TypeSystem::new();

        let int_type = Type::new(TypeId(1), TypeKind::Int);
        let bool_type = Type::new(TypeId(2), TypeKind::Bool);

        // L'unification devrait échouer pour des types incompatibles
        let result = type_system.unify(&int_type, &bool_type);
        assert!(result.is_err());
    }
}

