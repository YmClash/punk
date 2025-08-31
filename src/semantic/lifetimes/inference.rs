//src/semantic/lifetimes/inference.rs

use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::cell::RefCell;
use crate::parser::ast::{Expression, Statement, Declaration, ASTNode};
use crate::semantic::semantic_error::{SemanticError, SemanticErrorType, TypeError, Position};
use crate::semantic::context::CompilationContext;
use crate::semantic::symbols::SymbolId;
use crate::semantic::types::type_system::{TypeId, TypeKind};
use crate::semantic::flow::control_flow_graph::{ControlFlowGraph, BlockId};

/// Identifiant unique pour les lifetimes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LifetimeId(pub u32);

impl std::fmt::Display for LifetimeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'_{}", self.0)
    }
}

/// Représente une durée de vie (lifetime)
#[derive(Debug, Clone)]
pub struct Lifetime {
    pub id: LifetimeId,
    pub name: Option<String>,
    pub scope: ScopeId,
    pub constraints: Vec<LifetimeConstraint>,
}

/// Identifiant de scope pour les lifetimes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub u32);

/// Contraintes sur les lifetimes
#[derive(Debug, Clone)]
pub enum LifetimeConstraint {
    /// 'a: 'b signifie que 'a vit au moins aussi longtemps que 'b
    Outlives(LifetimeId, LifetimeId),
    
    /// 'a doit être égal à 'b
    Equal(LifetimeId, LifetimeId),
    
    /// 'a doit être au moins aussi long que le scope
    OutlivesScope(LifetimeId, ScopeId),
    
    /// Lifetime lié à une expression
    BoundToExpression(LifetimeId, usize), // usize est l'ID de l'expression
    
    /// Lifetime lié à un symbole
    BoundToSymbol(LifetimeId, SymbolId),
}

/// Région de code où un lifetime est valide
#[derive(Debug, Clone)]
pub struct LifetimeRegion {
    pub lifetime: LifetimeId,
    pub start_block: BlockId,
    pub end_block: BlockId,
    pub blocks: HashSet<BlockId>,
}

/// Moteur d'inférence des lifetimes
pub struct LifetimeInference {
    context: Rc<RefCell<CompilationContext>>,
    lifetimes: HashMap<LifetimeId, Lifetime>,
    constraints: Vec<LifetimeConstraint>,
    regions: HashMap<LifetimeId, LifetimeRegion>,
    next_lifetime_id: u32,
    next_scope_id: u32,
    current_scope: ScopeId,
    scope_stack: Vec<ScopeId>,
    cfg: Option<ControlFlowGraph>,
}

impl LifetimeInference {
    pub fn new(context: Rc<RefCell<CompilationContext>>) -> Self {
        LifetimeInference {
            context,
            lifetimes: HashMap::new(),
            constraints: Vec::new(),
            regions: HashMap::new(),
            next_lifetime_id: 1,
            next_scope_id: 1,
            current_scope: ScopeId(0), // Scope global
            scope_stack: vec![ScopeId(0)],
            cfg: None,
        }
    }
    
    /// Définit le CFG pour l'analyse
    pub fn set_cfg(&mut self, cfg: ControlFlowGraph) {
        self.cfg = Some(cfg);
    }
    
    /// Crée un nouveau lifetime
    pub fn fresh_lifetime(&mut self, name: Option<String>) -> LifetimeId {
        let id = LifetimeId(self.next_lifetime_id);
        self.next_lifetime_id += 1;
        
        let lifetime = Lifetime {
            id,
            name: name.clone().or_else(|| Some(format!("'_{}", id.0))),
            scope: self.current_scope,
            constraints: Vec::new(),
        };
        
        self.lifetimes.insert(id, lifetime);
        id
    }
    
    /// Entre dans un nouveau scope
    pub fn enter_scope(&mut self) -> ScopeId {
        let scope_id = ScopeId(self.next_scope_id);
        self.next_scope_id += 1;
        self.scope_stack.push(scope_id);
        self.current_scope = scope_id;
        scope_id
    }
    
    /// Sort du scope actuel
    pub fn exit_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop();
            self.current_scope = *self.scope_stack.last().unwrap();
        }
    }
    
    /// Ajoute une contrainte de lifetime
    pub fn add_constraint(&mut self, constraint: LifetimeConstraint) {
        self.constraints.push(constraint.clone());
        
        // Ajouter aussi à la lifetime concernée
        match &constraint {
            LifetimeConstraint::Outlives(a, _) |
            LifetimeConstraint::Equal(a, _) |
            LifetimeConstraint::OutlivesScope(a, _) |
            LifetimeConstraint::BoundToExpression(a, _) |
            LifetimeConstraint::BoundToSymbol(a, _) => {
                if let Some(lifetime) = self.lifetimes.get_mut(a) {
                    lifetime.constraints.push(constraint);
                }
            }
        }
    }
    
    /// Infère les lifetimes pour une expression
    pub fn infer_expression_lifetime(&mut self, expr: &Expression) -> Result<LifetimeId, SemanticError> {
        match expr {
            Expression::Identifier(name) => {
                // Chercher le symbole et retourner son lifetime
                let symbol_id_result = {
                    let ctx = self.context.borrow();
                    let symbols = ctx.symbols.borrow();
                    symbols.lookup_symbol(name).ok()
                };
                
                if let Some(symbol_id) = symbol_id_result {
                    let lifetime = self.fresh_lifetime(Some(format!("{}_lifetime", name)));
                    self.add_constraint(LifetimeConstraint::BoundToSymbol(lifetime, symbol_id));
                    Ok(lifetime)
                } else {
                    Err(create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::UndefinedVariable(name.clone())),
                        format!("Undefined variable: {}", name),
                        Position { index: 0 }
                    ))
                }
            }
            
            Expression::Borrow(borrow) => {
                // Créer un nouveau lifetime pour la référence
                let inner_lifetime = self.infer_expression_lifetime(&borrow.borrowed_value)?;
                let ref_lifetime = self.fresh_lifetime(Some("borrow".to_string()));
                
                // La référence doit vivre au moins aussi longtemps que ce qu'elle référence
                self.add_constraint(LifetimeConstraint::Outlives(inner_lifetime, ref_lifetime));
                
                Ok(ref_lifetime)
            }
            
            Expression::UnaryOperation(unop) => {
                // Pour le déréférencement et autres opérations unaires
                match unop.operator {
                    crate::parser::ast::UnaryOperator::Dereference => {
                        // Déréférencer retourne le lifetime de l'objet référencé
                        self.infer_expression_lifetime(&unop.operand)
                    }
                    _ => {
                        // Autres opérations unaires
                        self.infer_expression_lifetime(&unop.operand)
                    }
                }
            }
            
            Expression::BinaryOperation(binop) => {
                let left_lifetime = self.infer_expression_lifetime(&binop.left)?;
                let right_lifetime = self.infer_expression_lifetime(&binop.right)?;
                
                // Le résultat vit aussi longtemps que le plus court des opérandes
                let result_lifetime = self.fresh_lifetime(Some("binop_result".to_string()));
                self.add_constraint(LifetimeConstraint::Outlives(left_lifetime, result_lifetime));
                self.add_constraint(LifetimeConstraint::Outlives(right_lifetime, result_lifetime));
                
                Ok(result_lifetime)
            }
            
            Expression::FunctionCall(call) => {
                // Analyser les lifetimes des arguments
                let mut arg_lifetimes = Vec::new();
                for arg in &call.arguments {
                    arg_lifetimes.push(self.infer_expression_lifetime(arg)?);
                }
                
                // Le résultat a son propre lifetime
                let result_lifetime = self.fresh_lifetime(Some("call_result".to_string()));
                
                // Ajouter des contraintes basées sur la signature de la fonction
                // TODO: Analyser la signature de la fonction pour des contraintes plus précises
                
                Ok(result_lifetime)
            }
            
            Expression::Assignment(assign) => {
                let target_lifetime = self.infer_expression_lifetime(&assign.target)?;
                let value_lifetime = self.infer_expression_lifetime(&assign.value)?;
                
                // La valeur doit vivre au moins aussi longtemps que la cible
                self.add_constraint(LifetimeConstraint::Outlives(value_lifetime, target_lifetime));
                
                Ok(target_lifetime)
            }
            
            _ => {
                // Pour les autres expressions, créer un lifetime temporaire
                Ok(self.fresh_lifetime(Some("temp".to_string())))
            }
        }
    }
    
    /// Résout les contraintes de lifetime
    pub fn solve_constraints(&mut self) -> Result<(), SemanticError> {
        // Algorithme de point fixe pour propager les contraintes
        let mut changed = true;
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 100;
        
        while changed && iterations < MAX_ITERATIONS {
            changed = false;
            iterations += 1;
            
            // Parcourir toutes les contraintes
            for constraint in self.constraints.clone() {
                match constraint {
                    LifetimeConstraint::Outlives(longer, shorter) => {
                        // Propager les régions : longer doit contenir toutes les régions de shorter
                        if let (Some(longer_region), Some(shorter_region)) = 
                            (self.regions.get(&longer).cloned(), self.regions.get(&shorter).cloned()) {
                            
                            let mut new_blocks = longer_region.blocks.clone();
                            let old_size = new_blocks.len();
                            new_blocks.extend(&shorter_region.blocks);
                            
                            if new_blocks.len() > old_size {
                                changed = true;
                                self.regions.get_mut(&longer).unwrap().blocks = new_blocks;
                            }
                        }
                    }
                    
                    LifetimeConstraint::Equal(a, b) => {
                        // Les deux lifetimes doivent avoir exactement les mêmes régions
                        if let (Some(a_region), Some(b_region)) = 
                            (self.regions.get(&a).cloned(), self.regions.get(&b).cloned()) {
                            
                            let mut combined = a_region.blocks.clone();
                            combined.extend(&b_region.blocks);
                            
                            if combined.len() > a_region.blocks.len() {
                                changed = true;
                                self.regions.get_mut(&a).unwrap().blocks = combined.clone();
                            }
                            
                            if combined.len() > b_region.blocks.len() {
                                changed = true;
                                self.regions.get_mut(&b).unwrap().blocks = combined;
                            }
                        }
                    }
                    
                    _ => {
                        // Autres contraintes à implémenter
                    }
                }
            }
        }
        
        if iterations >= MAX_ITERATIONS {
            return Err(create_semantic_error(
                SemanticErrorType::TypeError(TypeError::InvalidType(
                    "Could not solve lifetime constraints".to_string()
                )),
                "Lifetime inference failed to converge".to_string(),
                Position { index: 0 }
            ));
        }
        
        Ok(())
    }
    
    /// Vérifie la validité des lifetimes
    pub fn validate_lifetimes(&self) -> Result<(), Vec<SemanticError>> {
        let mut errors = Vec::new();
        
        // Vérifier chaque contrainte
        for constraint in &self.constraints {
            match constraint {
                LifetimeConstraint::Outlives(longer, shorter) => {
                    if let (Some(longer_region), Some(shorter_region)) = 
                        (self.regions.get(longer), self.regions.get(shorter)) {
                        
                        // Vérifier que longer contient tous les blocks de shorter
                        if !shorter_region.blocks.is_subset(&longer_region.blocks) {
                            errors.push(create_semantic_error(
                                SemanticErrorType::TypeError(TypeError::InvalidType(
                                    format!("Lifetime {} does not outlive {}", longer, shorter)
                                )),
                                "Lifetime constraint violation".to_string(),
                                Position { index: 0 }
                            ));
                        }
                    }
                }
                
                LifetimeConstraint::Equal(a, b) => {
                    if let (Some(a_region), Some(b_region)) = 
                        (self.regions.get(a), self.regions.get(b)) {
                        
                        if a_region.blocks != b_region.blocks {
                            errors.push(create_semantic_error(
                                SemanticErrorType::TypeError(TypeError::InvalidType(
                                    format!("Lifetimes {} and {} are not equal", a, b)
                                )),
                                "Lifetime equality constraint violation".to_string(),
                                Position { index: 0 }
                            ));
                        }
                    }
                }
                
                _ => {
                    // Autres validations à implémenter
                }
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

// Fonction helper pour créer une erreur sémantique
fn create_semantic_error(error_type: SemanticErrorType, message: String, position: Position) -> SemanticError {
    SemanticError::new(error_type, message, position)
}