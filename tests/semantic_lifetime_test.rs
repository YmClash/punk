// tests/semantic_lifetime_test.rs
// Tests pour le système d'inférence des lifetimes

use punk::semantic::lifetimes::inference::{
    LifetimeInference, LifetimeId, LifetimeConstraint, ScopeId
};
use punk::semantic::context::CompilationContext;
use punk::semantic::symbols::SymbolId;
use punk::semantic::flow::control_flow_graph::ControlFlowGraph;
use punk::parser::ast::{Expression, Literal, Borrow, BorrowType, Access, UnaryOperation, UnaryOperator, BinaryOperation, Operator, Assignment};
use std::rc::Rc;
use std::cell::RefCell;

#[cfg(test)]
mod lifetime_tests {
    use super::*;

    fn create_test_context() -> Rc<RefCell<CompilationContext>> {
        Rc::new(RefCell::new(CompilationContext::new()))
    }

    #[test]
    fn test_lifetime_inference_creation() {
        let context = create_test_context();
        let _inference = LifetimeInference::new(context);
        
        // On ne peut que vérifier que la création ne panique pas
        // Les champs sont privés
    }

    #[test]
    fn test_fresh_lifetime_creation() {
        let context = create_test_context();
        let mut inference = LifetimeInference::new(context);
        
        // Créer un lifetime sans nom
        let lifetime1 = inference.fresh_lifetime(None);
        assert_eq!(lifetime1.0, 1); // LifetimeId est un tuple struct public
        
        // Créer un lifetime avec nom
        let lifetime2 = inference.fresh_lifetime(Some("'a".to_string()));
        assert_eq!(lifetime2.0, 2);
        
        // Vérifier que les lifetimes sont différents
        assert_ne!(lifetime1, lifetime2);
    }

    #[test]
    fn test_scope_management() {
        let context = create_test_context();
        let mut inference = LifetimeInference::new(context);
        
        // Entrer dans un nouveau scope
        let scope1 = inference.enter_scope();
        assert_eq!(scope1.0, 1); // ScopeId est un tuple struct public
        
        // Entrer dans un autre scope
        let scope2 = inference.enter_scope();
        assert_eq!(scope2.0, 2);
        assert_ne!(scope1, scope2);
        
        // Sortir du scope
        inference.exit_scope();
        
        // On peut entrer dans un nouveau scope
        let scope3 = inference.enter_scope();
        assert_eq!(scope3.0, 3);
    }

    #[test]
    fn test_add_constraint_outlives() {
        let context = create_test_context();
        let mut inference = LifetimeInference::new(context);
        
        let lifetime_a = inference.fresh_lifetime(Some("'a".to_string()));
        let lifetime_b = inference.fresh_lifetime(Some("'b".to_string()));
        
        // Ajouter une contrainte 'a: 'b
        inference.add_constraint(LifetimeConstraint::Outlives(lifetime_a, lifetime_b));
        
        // On ne peut pas vérifier directement car les champs sont privés,
        // mais on peut vérifier que ça ne panique pas
    }

    #[test]
    fn test_add_constraint_equal() {
        let context = create_test_context();
        let mut inference = LifetimeInference::new(context);
        
        let lifetime_a = inference.fresh_lifetime(Some("'a".to_string()));
        let lifetime_b = inference.fresh_lifetime(Some("'b".to_string()));
        
        // Ajouter une contrainte 'a == 'b
        inference.add_constraint(LifetimeConstraint::Equal(lifetime_a, lifetime_b));
    }

    #[test]
    fn test_add_constraint_outlives_scope() {
        let context = create_test_context();
        let mut inference = LifetimeInference::new(context);
        
        let lifetime = inference.fresh_lifetime(Some("'a".to_string()));
        let scope = inference.enter_scope();
        
        // Ajouter une contrainte 'a outlives scope
        inference.add_constraint(LifetimeConstraint::OutlivesScope(lifetime, scope));
    }

    #[test]
    fn test_add_constraint_bound_to_symbol() {
        let context = create_test_context();
        let mut inference = LifetimeInference::new(context);
        
        let lifetime = inference.fresh_lifetime(Some("'a".to_string()));
        let symbol_id = SymbolId(42);
        
        // Ajouter une contrainte liant le lifetime à un symbole
        inference.add_constraint(LifetimeConstraint::BoundToSymbol(lifetime, symbol_id));
    }

    #[test]
    fn test_add_constraint_bound_to_expression() {
        let context = create_test_context();
        let mut inference = LifetimeInference::new(context);
        
        let lifetime = inference.fresh_lifetime(Some("'a".to_string()));
        
        // Ajouter une contrainte liant le lifetime à une expression
        inference.add_constraint(LifetimeConstraint::BoundToExpression(lifetime, 123));
    }

    #[test]
    fn test_infer_literal_lifetime() {
        let context = create_test_context();
        let mut inference = LifetimeInference::new(context);
        
        // Un literal devrait avoir un lifetime temporaire
        let expr = Expression::Literal(Literal::Integer { value: 42.into() });
        let lifetime = inference.infer_expression_lifetime(&expr).unwrap();
        
        // Vérifier qu'un lifetime a été créé
        assert!(lifetime.0 > 0);
    }

    #[test]
    fn test_infer_borrow_lifetime() {
        let context = create_test_context();
        let mut inference = LifetimeInference::new(context);
        
        // Créer une expression d'emprunt
        let inner_expr = Box::new(Expression::Literal(Literal::Integer { value: 42.into() }));
        let borrow_expr = Expression::Borrow(Borrow {
            borrowed_value: inner_expr,
            borrowed_type: BorrowType::Immutable,
            access: Access::Read,
        });
        
        let ref_lifetime = inference.infer_expression_lifetime(&borrow_expr).unwrap();
        
        // Vérifier qu'un lifetime a été créé
        assert!(ref_lifetime.0 > 0);
    }

    #[test]
    fn test_infer_binary_operation_lifetime() {
        let context = create_test_context();
        let mut inference = LifetimeInference::new(context);
        
        // Créer une opération binaire
        let left = Box::new(Expression::Literal(Literal::Integer { value: 1.into() }));
        let right = Box::new(Expression::Literal(Literal::Integer { value: 2.into() }));
        let binop = Expression::BinaryOperation(BinaryOperation {
            left,
            operator: Operator::Addition,
            right,
        });
        
        let result_lifetime = inference.infer_expression_lifetime(&binop).unwrap();
        
        // Vérifier qu'un lifetime a été créé
        assert!(result_lifetime.0 > 0);
    }

    #[test]
    fn test_infer_unary_operation_lifetime() {
        let context = create_test_context();
        let mut inference = LifetimeInference::new(context);
        
        // Créer une opération unaire
        let operand = Box::new(Expression::Literal(Literal::Integer { value: 42.into() }));
        let unop = Expression::UnaryOperation(UnaryOperation {
            operator: UnaryOperator::LogicalNot,
            operand,
        });
        
        let result_lifetime = inference.infer_expression_lifetime(&unop).unwrap();
        
        // Vérifier qu'un lifetime a été créé
        assert!(result_lifetime.0 > 0);
    }

    #[test]
    fn test_infer_assignment_lifetime() {
        let context = create_test_context();
        let mut inference = LifetimeInference::new(context);
        
        // Créer une assignation
        let target = Box::new(Expression::Identifier("x".to_string()));
        let value = Box::new(Expression::Literal(Literal::Integer { value: 42.into() }));
        let assign = Expression::Assignment(Assignment {
            target,
            value,
        });
        
        // Cela échouera car 'x' n'existe pas dans le contexte
        let result = inference.infer_expression_lifetime(&assign);
        assert!(result.is_err());
    }

    #[test]
    fn test_solve_constraints_simple() {
        let context = create_test_context();
        let mut inference = LifetimeInference::new(context);
        
        let lifetime_a = inference.fresh_lifetime(Some("'a".to_string()));
        let lifetime_b = inference.fresh_lifetime(Some("'b".to_string()));
        
        // Ajouter des contraintes simples
        inference.add_constraint(LifetimeConstraint::Outlives(lifetime_a, lifetime_b));
        
        // Résoudre les contraintes
        let result = inference.solve_constraints();
        assert!(result.is_ok());
    }

    #[test]
    fn test_solve_constraints_multiple() {
        let context = create_test_context();
        let mut inference = LifetimeInference::new(context);
        
        let lifetime_a = inference.fresh_lifetime(Some("'a".to_string()));
        let lifetime_b = inference.fresh_lifetime(Some("'b".to_string()));
        let lifetime_c = inference.fresh_lifetime(Some("'c".to_string()));
        
        // Ajouter plusieurs contraintes: 'a: 'b, 'b: 'c
        inference.add_constraint(LifetimeConstraint::Outlives(lifetime_a, lifetime_b));
        inference.add_constraint(LifetimeConstraint::Outlives(lifetime_b, lifetime_c));
        
        // Résoudre les contraintes
        let result = inference.solve_constraints();
        assert!(result.is_ok());
    }

    #[test]
    fn test_solve_constraints_with_equal() {
        let context = create_test_context();
        let mut inference = LifetimeInference::new(context);
        
        let lifetime_a = inference.fresh_lifetime(Some("'a".to_string()));
        let lifetime_b = inference.fresh_lifetime(Some("'b".to_string()));
        
        // Créer des contraintes avec égalité
        inference.add_constraint(LifetimeConstraint::Equal(lifetime_a, lifetime_b));
        
        // Résoudre devrait fonctionner
        let result = inference.solve_constraints();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_lifetimes() {
        let context = create_test_context();
        let mut inference = LifetimeInference::new(context);
        
        // Créer quelques lifetimes et contraintes
        let lifetime_a = inference.fresh_lifetime(Some("'a".to_string()));
        let lifetime_b = inference.fresh_lifetime(Some("'b".to_string()));
        
        inference.add_constraint(LifetimeConstraint::Outlives(lifetime_a, lifetime_b));
        
        // D'abord résoudre
        inference.solve_constraints().unwrap();
        
        // Puis valider
        let result = inference.validate_lifetimes();
        assert!(result.is_ok());
    }

    #[test]
    fn test_with_cfg() {
        let context = create_test_context();
        let mut inference = LifetimeInference::new(context);
        
        // Créer un CFG simple
        let cfg = ControlFlowGraph::new();
        
        // Associer le CFG
        inference.set_cfg(cfg);
        
        // Créer des lifetimes
        let lifetime = inference.fresh_lifetime(Some("'a".to_string()));
        assert!(lifetime.0 > 0);
    }

    #[test]
    fn test_lifetime_display() {
        let lifetime = LifetimeId(42);
        let display = format!("{}", lifetime);
        assert_eq!(display, "'_42");
    }

    #[test]
    fn test_multiple_scopes() {
        let context = create_test_context();
        let mut inference = LifetimeInference::new(context);
        
        // Entrer dans plusieurs scopes
        let scope1 = inference.enter_scope();
        let lifetime1 = inference.fresh_lifetime(Some("'a".to_string()));
        
        let scope2 = inference.enter_scope();
        let lifetime2 = inference.fresh_lifetime(Some("'b".to_string()));
        
        // Ajouter des contraintes entre scopes
        inference.add_constraint(LifetimeConstraint::OutlivesScope(lifetime1, scope2));
        inference.add_constraint(LifetimeConstraint::OutlivesScope(lifetime2, scope1));
        
        // Sortir des scopes
        inference.exit_scope();
        inference.exit_scope();
        
        // Résoudre devrait fonctionner
        let result = inference.solve_constraints();
        assert!(result.is_ok());
    }
}