//src/semantic/traits.rs

use crate::parser::ast::{Expression, Statement, Declaration, ASTNode};
use crate::semantic::semantic_error::SemanticError;
use crate::semantic::types::type_system::TypeId;
use crate::semantic::symbols::SymbolId;
use crate::semantic::context::CompilationContext;
use std::rc::Rc;
use std::cell::RefCell;

/// Trait pour les éléments qui peuvent être vérifiés au niveau des types
pub trait TypeCheckable {
    /// Vérifie et retourne le type de l'élément
    fn check_type(&self, context: &Rc<RefCell<CompilationContext>>) -> Result<TypeId, SemanticError>;
    
    /// Vérifie si l'élément est compatible avec un type donné
    fn is_compatible_with(&self, target_type: TypeId, context: &Rc<RefCell<CompilationContext>>) -> bool {
        match self.check_type(context) {
            Ok(type_id) => {
                let ctx = context.borrow();
                let type_system = ctx.types.borrow();
                type_system.are_types_compatible(type_id, target_type)
            }
            Err(_) => false,
        }
    }
}

/// Trait pour les éléments qui peuvent être résolus dans la table des symboles
pub trait Resolvable {
    /// Résout l'élément et retourne son ID de symbole
    fn resolve(&self, context: &Rc<RefCell<CompilationContext>>) -> Result<SymbolId, SemanticError>;
    
    /// Vérifie si l'élément peut être résolu
    fn can_resolve(&self, context: &Rc<RefCell<CompilationContext>>) -> bool {
        self.resolve(context).is_ok()
    }
}

/// Trait pour le pattern Visitor sur l'AST
pub trait Visitable {
    /// Accepte un visiteur
    fn accept(&self, visitor: &mut dyn AstVisitor) -> Result<(), SemanticError>;
}

/// Trait pour les visiteurs de l'AST
pub trait AstVisitor {
    /// Visite une expression
    fn visit_expression(&mut self, expr: &Expression) -> Result<(), SemanticError>;
    
    /// Visite un statement
    fn visit_statement(&mut self, stmt: &Statement) -> Result<(), SemanticError>;
    
    /// Visite une déclaration
    fn visit_declaration(&mut self, decl: &Declaration) -> Result<(), SemanticError>;
    
    /// Visite un noeud AST générique
    fn visit_node(&mut self, node: &ASTNode) -> Result<(), SemanticError> {
        match node {
            ASTNode::Expression(expr) => self.visit_expression(expr),
            ASTNode::Statement(stmt) => self.visit_statement(stmt),
            ASTNode::Declaration(decl) => self.visit_declaration(decl),
            ASTNode::Program(nodes) => {
                for node in nodes {
                    self.visit_node(node)?;
                }
                Ok(())
            }
            ASTNode::Error(_) => {
                // Ignorer les erreurs de parsing, elles ont déjà été rapportées
                Ok(())
            }
        }
    }
}

/// Trait pour les éléments qui peuvent être analysés sémantiquement
pub trait SemanticAnalyzable {
    /// Effectue l'analyse sémantique sur l'élément
    fn analyze(&self, context: &Rc<RefCell<CompilationContext>>) -> Result<(), SemanticError>;
    
    /// Collecte les erreurs sémantiques sans interrompre l'analyse
    fn collect_errors(&self, context: &Rc<RefCell<CompilationContext>>) -> Vec<SemanticError> {
        let mut errors = Vec::new();
        if let Err(e) = self.analyze(context) {
            errors.push(e);
        }
        errors
    }
}

/// Trait pour les éléments qui peuvent être évalués à la compilation
pub trait CompileTimeEvaluable {
    /// Type de la valeur évaluée
    type Value;
    
    /// Évalue l'expression à la compilation si possible
    fn evaluate_at_compile_time(&self, context: &Rc<RefCell<CompilationContext>>) -> Option<Self::Value>;
    
    /// Vérifie si l'expression peut être évaluée à la compilation
    fn is_compile_time_constant(&self, context: &Rc<RefCell<CompilationContext>>) -> bool {
        self.evaluate_at_compile_time(context).is_some()
    }
}

/// Trait pour les éléments qui peuvent être optimisés
pub trait Optimizable {
    /// Optimise l'élément et retourne une version optimisée
    fn optimize(&self, context: &Rc<RefCell<CompilationContext>>) -> Self
    where
        Self: Sized;
    
    /// Vérifie si l'élément peut être optimisé
    fn can_optimize(&self, context: &Rc<RefCell<CompilationContext>>) -> bool;
}

/// Implémentation par défaut pour Expression
impl TypeCheckable for Expression {
    fn check_type(&self, context: &Rc<RefCell<CompilationContext>>) -> Result<TypeId, SemanticError> {
        use crate::semantic::type_checker::TypeChecker;
        
        let mut checker = TypeChecker::new(context.clone());
        checker.check_expression(self)
    }
}

/// Implémentation par défaut pour les visitables
impl Visitable for Expression {
    fn accept(&self, visitor: &mut dyn AstVisitor) -> Result<(), SemanticError> {
        visitor.visit_expression(self)
    }
}

impl Visitable for Statement {
    fn accept(&self, visitor: &mut dyn AstVisitor) -> Result<(), SemanticError> {
        visitor.visit_statement(self)
    }
}

impl Visitable for Declaration {
    fn accept(&self, visitor: &mut dyn AstVisitor) -> Result<(), SemanticError> {
        visitor.visit_declaration(self)
    }
}

impl Visitable for ASTNode {
    fn accept(&self, visitor: &mut dyn AstVisitor) -> Result<(), SemanticError> {
        visitor.visit_node(self)
    }
}

/// Exemple de visiteur concret pour le comptage de noeuds
pub struct NodeCountVisitor {
    pub expression_count: usize,
    pub statement_count: usize,
    pub declaration_count: usize,
}

impl NodeCountVisitor {
    pub fn new() -> Self {
        NodeCountVisitor {
            expression_count: 0,
            statement_count: 0,
            declaration_count: 0,
        }
    }
}

impl AstVisitor for NodeCountVisitor {
    fn visit_expression(&mut self, _expr: &Expression) -> Result<(), SemanticError> {
        self.expression_count += 1;
        Ok(())
    }
    
    fn visit_statement(&mut self, _stmt: &Statement) -> Result<(), SemanticError> {
        self.statement_count += 1;
        Ok(())
    }
    
    fn visit_declaration(&mut self, _decl: &Declaration) -> Result<(), SemanticError> {
        self.declaration_count += 1;
        Ok(())
    }
}

/// Trait pour les éléments qui supportent le pattern matching
pub trait PatternMatchable {
    /// Vérifie si l'élément correspond au pattern
    fn matches_pattern(&self, pattern: &Expression, context: &Rc<RefCell<CompilationContext>>) -> bool;
    
    /// Extrait les bindings du pattern matching
    fn extract_bindings(&self, pattern: &Expression, context: &Rc<RefCell<CompilationContext>>) -> Vec<(String, TypeId)>;
}

/// Trait pour les éléments qui peuvent être sérialisés/désérialisés
pub trait Serializable {
    /// Sérialise l'élément en format JSON ou autre
    fn serialize(&self) -> String;
    
    /// Désérialise depuis une représentation string
    fn deserialize(data: &str) -> Result<Self, String>
    where
        Self: Sized;
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use super::*;
    use crate::parser::ast::{Literal, Expression};
    
    #[test]
    fn test_node_count_visitor() {
        let mut visitor = NodeCountVisitor::new();
        let expr = Expression::Literal(Literal::Integer { value: BigInt::from(42) });
        
        expr.accept(&mut visitor).unwrap();
        assert_eq!(visitor.expression_count, 1);
        assert_eq!(visitor.statement_count, 0);
        assert_eq!(visitor.declaration_count, 0);
    }
}