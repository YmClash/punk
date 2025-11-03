// tests/semantic_optimization_test.rs
// Tests pour les optimisations sémantiques

use punk::semantic::optimizations::constant_folding::{
    ConstantFolder, ConstantValue
};
use punk::semantic::optimizations::dead_code_elimination::DeadCodeEliminator;
use punk::semantic::context::CompilationContext;
use punk::parser::ast::{
    ASTNode, Expression, Literal, BinaryOperation, UnaryOperation, 
    Operator, UnaryOperator
};
use std::rc::Rc;
use std::cell::RefCell;
use num_bigint::BigInt;

#[cfg(test)]
mod optimization_tests {
    use super::*;

    fn create_test_context() -> Rc<RefCell<CompilationContext>> {
        Rc::new(RefCell::new(CompilationContext::new()))
    }

    // ========== Tests de Constant Folding ==========

    #[test]
    fn test_constant_value_from_literal() {
        // Test conversion depuis Literal
        let int_lit = Literal::Integer { value: BigInt::from(42) };
        let const_val = ConstantValue::from_literal(&int_lit).unwrap();
        assert_eq!(const_val, ConstantValue::Integer(42));

        let float_lit = Literal::Float { value: 3.14 };
        let const_val = ConstantValue::from_literal(&float_lit).unwrap();
        assert_eq!(const_val, ConstantValue::Float(3.14));

        let bool_lit = Literal::Boolean(true);
        let const_val = ConstantValue::from_literal(&bool_lit).unwrap();
        assert_eq!(const_val, ConstantValue::Boolean(true));

        let string_lit = Literal::String("hello".to_string());
        let const_val = ConstantValue::from_literal(&string_lit).unwrap();
        assert_eq!(const_val, ConstantValue::String("hello".to_string()));

        let char_lit = Literal::Char('a');
        let const_val = ConstantValue::from_literal(&char_lit).unwrap();
        assert_eq!(const_val, ConstantValue::Char('a'));
    }

    #[test]
    fn test_constant_value_to_literal() {
        // Test conversion vers Literal
        let const_val = ConstantValue::Integer(42);
        let lit = const_val.to_literal();
        match lit {
            Literal::Integer { value } => assert_eq!(value, BigInt::from(42)),
            _ => panic!("Expected Integer literal"),
        }

        let const_val = ConstantValue::Float(3.14);
        let lit = const_val.to_literal();
        match lit {
            Literal::Float { value } => assert_eq!(value, 3.14),
            _ => panic!("Expected Float literal"),
        }

        let const_val = ConstantValue::Boolean(false);
        let lit = const_val.to_literal();
        match lit {
            Literal::Boolean(b) => assert!(!b),
            _ => panic!("Expected Boolean literal"),
        }

        let const_val = ConstantValue::String("test".to_string());
        let lit = const_val.to_literal();
        match lit {
            Literal::String(s) => assert_eq!(s, "test"),
            _ => panic!("Expected String literal"),
        }

        let const_val = ConstantValue::Char('z');
        let lit = const_val.to_literal();
        match lit {
            Literal::Char(c) => assert_eq!(c, 'z'),
            _ => panic!("Expected Char literal"),
        }
    }

    #[test]
    fn test_constant_folder_creation() {
        let context = create_test_context();
        let folder = ConstantFolder::new(context);
        
        // Vérifier l'état initial
        assert_eq!(folder.stats.expressions_folded, 0);
        assert_eq!(folder.stats.arithmetic_operations, 0);
        assert_eq!(folder.stats.boolean_operations, 0);
        assert_eq!(folder.stats.string_concatenations, 0);
        assert_eq!(folder.stats.comparisons_folded, 0);
    }

    #[test]
    fn test_fold_arithmetic_addition() {
        let context = create_test_context();
        let mut folder = ConstantFolder::new(context);
        
        // Créer l'expression: 2 + 3
        let expr = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(2) })),
            operator: Operator::Addition,
            right: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(3) })),
        });
        
        // Plier l'expression
        let folded = folder.fold_expression(&expr);
        assert!(folded.is_some());
        
        // Vérifier le résultat
        if let Some(Expression::Literal(Literal::Integer { value })) = folded {
            assert_eq!(value, BigInt::from(5));
        } else {
            panic!("Expected folded Integer literal");
        }
        
        // Vérifier les stats
        assert_eq!(folder.stats.expressions_folded, 1);
        assert_eq!(folder.stats.arithmetic_operations, 1);
    }

    #[test]
    fn test_fold_arithmetic_subtraction() {
        let context = create_test_context();
        let mut folder = ConstantFolder::new(context);
        
        // Créer l'expression: 10 - 4
        let expr = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(10) })),
            operator: Operator::Subtraction,
            right: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(4) })),
        });
        
        let folded = folder.fold_expression(&expr);
        assert!(folded.is_some());
        
        if let Some(Expression::Literal(Literal::Integer { value })) = folded {
            assert_eq!(value, BigInt::from(6));
        } else {
            panic!("Expected folded Integer literal");
        }
    }

    #[test]
    fn test_fold_arithmetic_multiplication() {
        let context = create_test_context();
        let mut folder = ConstantFolder::new(context);
        
        // Créer l'expression: 6 * 7
        let expr = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(6) })),
            operator: Operator::Multiplication,
            right: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(7) })),
        });
        
        let folded = folder.fold_expression(&expr);
        assert!(folded.is_some());
        
        if let Some(Expression::Literal(Literal::Integer { value })) = folded {
            assert_eq!(value, BigInt::from(42));
        } else {
            panic!("Expected folded Integer literal");
        }
    }

    #[test]
    fn test_fold_arithmetic_division() {
        let context = create_test_context();
        let mut folder = ConstantFolder::new(context);
        
        // Créer l'expression: 20 / 4
        let expr = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(20) })),
            operator: Operator::Division,
            right: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(4) })),
        });
        
        let folded = folder.fold_expression(&expr);
        assert!(folded.is_some());
        
        if let Some(Expression::Literal(Literal::Integer { value })) = folded {
            assert_eq!(value, BigInt::from(5));
        } else {
            panic!("Expected folded Integer literal");
        }
    }

    #[test]
    fn test_fold_division_by_zero() {
        let context = create_test_context();
        let mut folder = ConstantFolder::new(context);
        
        // Créer l'expression: 10 / 0
        let expr = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(10) })),
            operator: Operator::Division,
            right: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(0) })),
        });
        
        // La division par zéro ne devrait pas être pliée
        let folded = folder.fold_expression(&expr);
        assert!(folded.is_none());
    }

    #[test]
    fn test_fold_boolean_and() {
        let context = create_test_context();
        let mut folder = ConstantFolder::new(context);
        
        // true && true
        let expr = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(Expression::Literal(Literal::Boolean(true))),
            operator: Operator::And,
            right: Box::new(Expression::Literal(Literal::Boolean(true))),
        });
        
        let folded = folder.fold_expression(&expr);
        assert!(folded.is_some());
        
        if let Some(Expression::Literal(Literal::Boolean(value))) = folded {
            assert!(value);
        } else {
            panic!("Expected folded Boolean literal");
        }
        
        assert_eq!(folder.stats.boolean_operations, 1);
    }

    #[test]
    fn test_fold_boolean_or() {
        let context = create_test_context();
        let mut folder = ConstantFolder::new(context);
        
        // false || true
        let expr = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(Expression::Literal(Literal::Boolean(false))),
            operator: Operator::Or,
            right: Box::new(Expression::Literal(Literal::Boolean(true))),
        });
        
        let folded = folder.fold_expression(&expr);
        assert!(folded.is_some());
        
        if let Some(Expression::Literal(Literal::Boolean(value))) = folded {
            assert!(value);
        } else {
            panic!("Expected folded Boolean literal");
        }
    }

    #[test]
    fn test_fold_unary_negation() {
        let context = create_test_context();
        let mut folder = ConstantFolder::new(context);
        
        // -42
        let expr = Expression::UnaryOperation(UnaryOperation {
            operator: UnaryOperator::Negate,
            operand: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(42) })),
        });
        
        let folded = folder.fold_expression(&expr);
        assert!(folded.is_some());
        
        if let Some(Expression::Literal(Literal::Integer { value })) = folded {
            assert_eq!(value, BigInt::from(-42));
        } else {
            panic!("Expected folded Integer literal");
        }
    }

    #[test]
    fn test_fold_unary_logical_not() {
        let context = create_test_context();
        let mut folder = ConstantFolder::new(context);
        
        // !true
        let expr = Expression::UnaryOperation(UnaryOperation {
            operator: UnaryOperator::LogicalNot,
            operand: Box::new(Expression::Literal(Literal::Boolean(true))),
        });
        
        let folded = folder.fold_expression(&expr);
        assert!(folded.is_some());
        
        if let Some(Expression::Literal(Literal::Boolean(value))) = folded {
            assert!(!value);
        } else {
            panic!("Expected folded Boolean literal");
        }
    }

    #[test]
    fn test_fold_comparison_equal() {
        let context = create_test_context();
        let mut folder = ConstantFolder::new(context);
        
        // 5 == 5
        let expr = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(5) })),
            operator: Operator::Equal,
            right: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(5) })),
        });
        
        let folded = folder.fold_expression(&expr);
        assert!(folded.is_some());
        
        if let Some(Expression::Literal(Literal::Boolean(value))) = folded {
            assert!(value);
        } else {
            panic!("Expected folded Boolean literal");
        }
        
        assert_eq!(folder.stats.comparisons_folded, 1);
    }

    #[test]
    fn test_fold_comparison_less_than() {
        let context = create_test_context();
        let mut folder = ConstantFolder::new(context);
        
        // 3 < 7
        let expr = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(3) })),
            operator: Operator::LessThan,
            right: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(7) })),
        });
        
        let folded = folder.fold_expression(&expr);
        assert!(folded.is_some());
        
        if let Some(Expression::Literal(Literal::Boolean(value))) = folded {
            assert!(value);
        } else {
            panic!("Expected folded Boolean literal");
        }
    }

    #[test]
    fn test_fold_nested_expression() {
        let context = create_test_context();
        let mut folder = ConstantFolder::new(context);
        
        // (2 + 3) * 4
        let inner = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(2) })),
            operator: Operator::Addition,
            right: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(3) })),
        });
        
        let expr = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(inner),
            operator: Operator::Multiplication,
            right: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(4) })),
        });
        
        let folded = folder.fold_expression(&expr);
        assert!(folded.is_some());
        
        if let Some(Expression::Literal(Literal::Integer { value })) = folded {
            assert_eq!(value, BigInt::from(20));
        } else {
            panic!("Expected folded Integer literal");
        }
        
        // Les deux opérations devraient être comptées
        assert_eq!(folder.stats.arithmetic_operations, 2);
    }

    #[test]
    fn test_optimize_ast_node() {
        let context = create_test_context();
        let mut folder = ConstantFolder::new(context);
        
        // Créer un noeud avec une expression constante
        let node = ASTNode::Expression(Expression::BinaryOperation(BinaryOperation {
            left: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(10) })),
            operator: Operator::Addition,
            right: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(20) })),
        }));
        
        // Optimiser le noeud
        let mut nodes = vec![node];
        folder.optimize_ast(&mut nodes);
        
        // Vérifier que l'expression a été pliée
        if let ASTNode::Expression(Expression::Literal(Literal::Integer { value })) = &nodes[0] {
            assert_eq!(*value, BigInt::from(30));
        } else {
            panic!("Expected optimized expression to be a literal");
        }
    }

    // ========== Tests de Dead Code Elimination ==========

    #[test]
    fn test_dead_code_eliminator_creation() {
        let context = create_test_context();
        let eliminator = DeadCodeEliminator::new(context);
        
        // Vérifier l'état initial
        assert_eq!(eliminator.stats.dead_statements_removed, 0);
        assert_eq!(eliminator.stats.unreachable_code_removed, 0);
        assert_eq!(eliminator.stats.unused_variables_removed, 0);
        assert_eq!(eliminator.stats.unused_functions_removed, 0);
        assert_eq!(eliminator.stats.redundant_assignments_removed, 0);
        assert_eq!(eliminator.stats.empty_blocks_removed, 0);
    }

    #[test]
    fn test_optimize_empty_ast() {
        let context = create_test_context();
        let mut eliminator = DeadCodeEliminator::new(context);
        
        let mut ast = vec![];
        eliminator.optimize_ast(&mut ast);
        
        // L'AST vide devrait rester vide
        assert_eq!(ast.len(), 0);
    }

    #[test]
    fn test_optimize_single_expression() {
        let context = create_test_context();
        let mut eliminator = DeadCodeEliminator::new(context);
        
        let mut ast = vec![
            ASTNode::Expression(Expression::Literal(Literal::Integer { value: BigInt::from(42) }))
        ];
        
        let original_len = ast.len();
        eliminator.optimize_ast(&mut ast);
        
        // L'expression devrait rester
        assert_eq!(ast.len(), original_len);
    }

    #[test]
    fn test_dead_code_display_stats() {
        let context = create_test_context();
        let eliminator = DeadCodeEliminator::new(context);
        
        // Vérifier que display_stats ne panique pas
        eliminator.display_stats();
    }

    #[test]
    fn test_constant_folder_display_stats() {
        let context = create_test_context();
        let folder = ConstantFolder::new(context);
        
        // Vérifier que display_stats ne panique pas
        folder.display_stats();
    }

    #[test]
    fn test_fold_modulo() {
        let context = create_test_context();
        let mut folder = ConstantFolder::new(context);
        
        // 10 % 3
        let expr = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(10) })),
            operator: Operator::Modulo,
            right: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(3) })),
        });
        
        let folded = folder.fold_expression(&expr);
        assert!(folded.is_some());
        
        if let Some(Expression::Literal(Literal::Integer { value })) = folded {
            assert_eq!(value, BigInt::from(1));
        } else {
            panic!("Expected folded Integer literal");
        }
    }

    #[test]
    fn test_fold_float_operations() {
        let context = create_test_context();
        let mut folder = ConstantFolder::new(context);
        
        // 3.14 + 2.86
        let expr = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(Expression::Literal(Literal::Float { value: 3.14 })),
            operator: Operator::Addition,
            right: Box::new(Expression::Literal(Literal::Float { value: 2.86 })),
        });
        
        let folded = folder.fold_expression(&expr);
        assert!(folded.is_some());
        
        if let Some(Expression::Literal(Literal::Float { value })) = folded {
            assert!((value - 6.0).abs() < 0.001);
        } else {
            panic!("Expected folded Float literal");
        }
    }

    #[test]
    fn test_fold_comparison_not_equal() {
        let context = create_test_context();
        let mut folder = ConstantFolder::new(context);
        
        // 5 != 3
        let expr = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(5) })),
            operator: Operator::NotEqual,
            right: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(3) })),
        });
        
        let folded = folder.fold_expression(&expr);
        assert!(folded.is_some());
        
        if let Some(Expression::Literal(Literal::Boolean(value))) = folded {
            assert!(value);
        } else {
            panic!("Expected folded Boolean literal");
        }
    }

    #[test]
    fn test_fold_comparison_greater_than_or_equal() {
        let context = create_test_context();
        let mut folder = ConstantFolder::new(context);
        
        // 7 >= 7
        let expr = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(7) })),
            operator: Operator::GreaterThanOrEqual,
            right: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(7) })),
        });
        
        let folded = folder.fold_expression(&expr);
        assert!(folded.is_some());
        
        if let Some(Expression::Literal(Literal::Boolean(value))) = folded {
            assert!(value);
        } else {
            panic!("Expected folded Boolean literal");
        }
    }
}