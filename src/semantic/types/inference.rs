//src/semantic/types/inference.rs

use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::cell::RefCell;
use crate::parser::ast::{Expression, Statement, Declaration, ASTNode};
use crate::semantic::types::type_system::{Type, TypeId, TypeKind, TypeSystem, TypeVarId};
use crate::semantic::semantic_error::{SemanticError, SemanticErrorType, TypeError, Position};
use crate::semantic::context::CompilationContext;
use crate::semantic::symbols::SymbolId;

/// Contrainte de type pour l'inference
#[derive(Debug, Clone)]
pub enum TypeConstraint {
    /// Deux types doivent �tre egaux
    Equal(TypeId, TypeId),
    
    /// Un type doit avoir un champ specifique
    HasField(TypeId, String, TypeId),
    
    /// Un type doit �tre callable avec certains param�tres
    Callable(TypeId, Vec<TypeId>, TypeId),
    
    /// Un type doit impl�menter un trait
    Implements(TypeId, SymbolId),
    
    /// Un type doit �tre un sous-type d'un autre
    Subtype(TypeId, TypeId),
    
    /// Un type doit �tre num�rique
    Numeric(TypeId),
    
    /// Un type doit �tre comparable
    Comparable(TypeId),
}

/// Substitution de types (mapping TypeVarId -> TypeId)
#[derive(Debug, Clone)]
pub struct Substitution {
    mappings: HashMap<TypeVarId, TypeId>,
}

impl Substitution {
    pub fn new() -> Self {
        Substitution {
            mappings: HashMap::new(),
        }
    }
    
    /// Ajoute une substitution
    pub fn insert(&mut self, var: TypeVarId, typ: TypeId) {
        self.mappings.insert(var, typ);
    }
    
    /// Applique les substitutions � un type
    pub fn apply(&self, type_system: &TypeSystem, type_id: TypeId) -> TypeId {
        if let Some(typ) = type_system.get_type(type_id) {
            match &typ.kind {
                TypeKind::Infer(var_id) => {
                    // Si on a une substitution pour cette variable, l'appliquer
                    if let Some(&subst_type_id) = self.mappings.get(var_id) {
                        // Appliquer r�cursivement au cas o� le type substitu� contient aussi des variables
                        self.apply(type_system, subst_type_id)
                    } else {
                        type_id
                    }
                }
                _ => type_id
            }
        } else {
            type_id
        }
    }
    
    /// Compose deux substitutions
    pub fn compose(&self, other: &Substitution, type_system: &TypeSystem) -> Substitution {
        let mut result = Substitution::new();
        
        // Appliquer 'other' � toutes les valeurs de 'self'
        for (var, typ) in &self.mappings {
            result.insert(var.clone(), other.apply(type_system, *typ));
        }
        
        // Ajouter les mappings de 'other' qui ne sont pas dans 'self'
        for (var, typ) in &other.mappings {
            if !result.mappings.contains_key(var) {
                result.insert(var.clone(), *typ);
            }
        }
        
        result
    }
}

/// Moteur d'inférence de types
pub struct TypeInferenceEngine {
    pub(super) context: Rc<RefCell<CompilationContext>>,
    pub(super) constraints: Vec<TypeConstraint>,
    pub(super) substitution: Substitution,
    pub(super) next_type_var: u32,
}

impl TypeInferenceEngine {
    pub fn new(context: Rc<RefCell<CompilationContext>>) -> Self {
        TypeInferenceEngine {
            context,
            constraints: Vec::new(),
            substitution: Substitution::new(),
            next_type_var: 1000, // Commencer à 1000 pour éviter les conflits
        }
    }
    
    /// Crée une nouvelle variable de type
    pub fn fresh_type_var(&mut self, name: Option<String>) -> TypeId {
        let var_name = name.unwrap_or_else(|| format!("T{}", self.next_type_var));
        self.next_type_var += 1;
        
        let var_id = TypeVarId {
            id: TypeId(self.next_type_var),
            name: var_name,
        };
        
        let  ctx = self.context.borrow_mut();
        let mut type_system = ctx.types.borrow_mut();
        type_system.register_type(TypeKind::Infer(var_id))
    }
    
    /// Algorithme d'unification de Robinson
    pub fn unify(&mut self, t1: TypeId, t2: TypeId) -> Result<Substitution, SemanticError> {
        let ctx = self.context.borrow();
        let type_system = ctx.types.borrow();
        
        let type1 = type_system.get_type(t1).cloned();
        let type2 = type_system.get_type(t2).cloned();
        drop(type_system);
        drop(ctx);
        
        match (type1, type2) {
            (Some(typ1), Some(typ2)) => {
                self.unify_types(&typ1, &typ2)
            }
            _ => Err(create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeNotFound(
                    format!("Cannot unify types {:?} and {:?}", t1, t2)
                )),
                "Type not found during unification".to_string(),
                Position { index: 0 }
            ))
        }
    }
    
    /// Unifie deux types
    fn unify_types(&mut self, t1: &Type, t2: &Type) -> Result<Substitution, SemanticError> {
        match (&t1.kind, &t2.kind) {
            // Cas 1: Deux types identiques
            (k1, k2) if *k1 == *k2 => Ok(Substitution::new()),
            
            // Cas 2: Variable de type à gauche
            (TypeKind::Infer(var), _) => {
                if self.occurs_check(var, t2) {
                    Err(create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::InfiniteType(
                            format!("Infinite type: {} = {}", var.name, t2)
                        )),
                        "Occurs check failed".to_string(),
                        Position { index: 0 }
                    ))
                } else {
                    let mut subst = Substitution::new();
                    subst.insert(var.clone(), t2.id);
                    Ok(subst)
                }
            }
            
            // Cas 3: Variable de type à droite
            (_, TypeKind::Infer(var)) => {
                if self.occurs_check(var, t1) {
                    Err(create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::InfiniteType(
                            format!("Infinite type: {} = {}", var.name, t1)
                        )),
                        "Occurs check failed".to_string(),
                        Position { index: 0 }
                    ))
                } else {
                    let mut subst = Substitution::new();
                    subst.insert(var.clone(), t1.id);
                    Ok(subst)
                }
            }
            
            // Cas 4: Types structurellement similaires
            (TypeKind::Array(elem1, size1), TypeKind::Array(elem2, size2)) => {
                if *size1 != *size2 {
                    return Err(create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::TypeMismatch(
                            format!("Array sizes don't match: {:?} vs {:?}", size1, size2)
                        )),
                        "Array size mismatch".to_string(),
                        Position { index: 0 }
                    ));
                }
                self.unify_types(elem1, elem2)
            }
            
            (TypeKind::Tuple(types1), TypeKind::Tuple(types2)) => {
                if types1.len() != types2.len() {
                    return Err(create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::TypeMismatch(
                            format!("Tuple lengths don't match: {} vs {}", types1.len(), types2.len())
                        )),
                        "Tuple length mismatch".to_string(),
                        Position { index: 0 }
                    ));
                }
                
                let mut result = Substitution::new();
                for (t1, t2) in types1.iter().zip(types2.iter()) {
                    let subst = self.unify_types(t1, t2)?;
                    let ctx = self.context.borrow();
                    let type_system = ctx.types.borrow();
                    result = result.compose(&subst, &type_system);
                }
                Ok(result)
            }
            
            // Cas 5: Types incompatibles
            _ => Err(create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeMismatch(
                    format!("Cannot unify {} with {}", t1, t2)
                )),
                "Type mismatch".to_string(),
                Position { index: 0 }
            ))
        }
    }
    
    /// Vérifie si une variable de type apparaît dans un type (occurs check)
    fn occurs_check(&self, var: &TypeVarId, typ: &Type) -> bool {
        match &typ.kind {
            TypeKind::Infer(v) => *v == *var,
            TypeKind::Array(elem, _) => self.occurs_check(var, elem),
            TypeKind::Tuple(types) => types.iter().any(|t| self.occurs_check(var, t)),
            TypeKind::Function(func_type) => {
                func_type.params.iter().any(|t| self.occurs_check(var, t)) ||
                self.occurs_check(var, &func_type.return_type)
            }
            TypeKind::Reference(inner, _, _) => self.occurs_check(var, inner),
            _ => false
        }
    }
    
    /// Résout toutes les contraintes collectées
    pub fn solve_constraints(&mut self) -> Result<(), SemanticError> {
        // Traiter chaque contrainte
        while let Some(constraint) = self.constraints.pop() {
            match constraint {
                TypeConstraint::Equal(t1, t2) => {
                    let subst = self.unify(t1, t2)?;
                    let ctx = self.context.borrow();
                    let type_system = ctx.types.borrow();
                    self.substitution = self.substitution.compose(&subst, &type_system);
                }
                
                TypeConstraint::HasField(object_type, field_name, field_type) => {
                    // Pour l'instant, on suppose que le champ existe
                    // Dans une implémentation complète, il faudrait vérifier
                    // que le type a bien ce champ et unifier les types
                }
                
                TypeConstraint::Callable(func_type, arg_types, return_type) => {
                    // Vérifier que func_type est bien une fonction
                    let func_sig_info = {
                        let ctx = self.context.borrow();
                        let type_system = ctx.types.borrow();
                        
                        if let Some(typ) = type_system.get_type(func_type) {
                            if let TypeKind::Function(func_sig) = &typ.kind {
                                Some((func_sig.params.clone(), func_sig.return_type.clone()))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    };
                    
                    if let Some((params, ret_type)) = func_sig_info {
                        // Vérifier le nombre d'arguments
                        if params.len() != arg_types.len() {
                            return Err(create_semantic_error(
                                SemanticErrorType::TypeError(TypeError::TypeMismatch(
                                    format!("Function expects {} arguments, got {}", 
                                            params.len(), arg_types.len())
                                )),
                                "Argument count mismatch".to_string(),
                                Position { index: 0 }
                            ));
                        }
                        
                        // Unifier les types des arguments
                        for (param_type, arg_type) in params.iter().zip(arg_types.iter()) {
                            let subst = self.unify(param_type.id, *arg_type)?;
                            let ctx = self.context.borrow();
                            let type_system = ctx.types.borrow();
                            self.substitution = self.substitution.compose(&subst, &type_system);
                        }
                        
                        // Unifier le type de retour
                        let subst = self.unify(ret_type.id, return_type)?;
                        let ctx = self.context.borrow();
                        let type_system = ctx.types.borrow();
                        self.substitution = self.substitution.compose(&subst, &type_system);
                    }
                }
                
                TypeConstraint::Numeric(type_id) => {
                    // Vérifier que le type est numérique
                    let ctx = self.context.borrow();
                    let type_system = ctx.types.borrow();
                    
                    if let Some(typ) = type_system.get_type(type_id) {
                        match &typ.kind {
                            TypeKind::Int | TypeKind::Float => {
                                // OK, c'est un type numérique
                            }
                            TypeKind::Infer(_) => {
                                // Type non encore inféré, on laisse pour plus tard
                                self.constraints.insert(0, TypeConstraint::Numeric(type_id));
                            }
                            _ => {
                                return Err(create_semantic_error(
                                    SemanticErrorType::TypeError(TypeError::TypeMismatch(
                                        format!("Type {} is not numeric", typ)
                                    )),
                                    "Non-numeric type in numeric context".to_string(),
                                    Position { index: 0 }
                                ));
                            }
                        }
                    }
                }
                
                TypeConstraint::Comparable(type_id) => {
                    // Vérifier que le type est comparable
                    let ctx = self.context.borrow();
                    let type_system = ctx.types.borrow();
                    
                    if let Some(typ) = type_system.get_type(type_id) {
                        match &typ.kind {
                            TypeKind::Int | TypeKind::Float | TypeKind::Char | 
                            TypeKind::String | TypeKind::Bool => {
                                // OK, c'est un type comparable
                            }
                            TypeKind::Infer(_) => {
                                // Type non encore inféré, on laisse pour plus tard
                                self.constraints.insert(0, TypeConstraint::Comparable(type_id));
                            }
                            _ => {
                                return Err(create_semantic_error(
                                    SemanticErrorType::TypeError(TypeError::TypeMismatch(
                                        format!("Type {} is not comparable", typ)
                                    )),
                                    "Non-comparable type in comparison".to_string(),
                                    Position { index: 0 }
                                ));
                            }
                        }
                    }
                }
                
                _ => {
                    // Autres contraintes non encore implémentées
                }
            }
        }
        
        Ok(())
    }
    
    /// Applique les substitutions finales à un type
    pub fn finalize_type(&self, type_id: TypeId) -> TypeId {
        let ctx = self.context.borrow();
        let type_system = ctx.types.borrow();
        self.substitution.apply(&type_system, type_id)
    }
}

// Fonction helper pour créer une erreur sémantique
fn create_semantic_error(error_type: SemanticErrorType, message: String, position: Position) -> SemanticError {
    SemanticError::new(error_type, message, position)
}