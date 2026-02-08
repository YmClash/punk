// tests/semantic_extended_test.rs
// Tests sémantiques étendus basés sur l'API RÉELLE

use punk::semantic::types::type_system::{TypeSystem, Type, TypeId, TypeKind, Mutability as TypeMutability};
use punk::parser::ast;
use punk::semantic::analyser::SemanticAnalyzer;
use punk::parser::ast::{Declaration, Expression, Literal, VariableDeclaration, Mutability, Type as ASTType};

#[cfg(test)]
mod extended_tests {
    use super::*;

    // ========== Tests du TypeSystem basés sur l'API réelle ==========
    
    #[test]
    fn test_type_system_primitives() {
        let type_system = TypeSystem::new();
        
        // Vérifier que les types primitifs sont créés
        assert!(type_system.get_type(type_system.type_int).is_some());
        assert!(type_system.get_type(type_system.type_float).is_some());
        assert!(type_system.get_type(type_system.type_bool).is_some());
        assert!(type_system.get_type(type_system.type_string).is_some());
        assert!(type_system.get_type(type_system.type_char).is_some());
        assert!(type_system.get_type(type_system.type_unit).is_some());
        assert!(type_system.get_type(type_system.type_never).is_some());
        assert!(type_system.get_type(type_system.type_error).is_some());
    }

    #[test]
    fn test_create_array_type() {
        let mut type_system = TypeSystem::new();
        
        // Créer [int; 5]
        let array_type_id = type_system.create_array_type(type_system.type_int, Some(5));
        let array_type = type_system.get_type(array_type_id).unwrap();
        
        match &array_type.kind {
            TypeKind::Array(elem_type, size) => {
                // elem_type est un Box<Type>
                assert_eq!(elem_type.id, type_system.type_int);
                assert_eq!(*size, Some(5));
            }
            _ => panic!("Expected Array type"),
        }
    }

    #[test]
    fn test_create_tuple_type() {
        let mut type_system = TypeSystem::new();
        
        // Créer (int, float, bool)
        let tuple_id = type_system.create_tuple_type(vec![
            type_system.type_int,
            type_system.type_float,
            type_system.type_bool,
        ]);
        
        let tuple_type = type_system.get_type(tuple_id).unwrap();
        
        match &tuple_type.kind {
            TypeKind::Tuple(types) => {
                assert_eq!(types.len(), 3);
                // types est Vec<Type>
                assert_eq!(types[0].id, type_system.type_int);
                assert_eq!(types[1].id, type_system.type_float);
                assert_eq!(types[2].id, type_system.type_bool);
            }
            _ => panic!("Expected Tuple type"),
        }
    }

    #[test]
    fn test_create_function_type() {
        let mut type_system = TypeSystem::new();
        
        // Créer fn(int, int) -> int
        let func_type_id = type_system.create_function_type(vec![type_system.type_int, type_system.type_int], type_system.type_int, false);
        
        let func_type = type_system.get_type(func_type_id).unwrap();
        
        match &func_type.kind {
            TypeKind::Function(func) => {
                assert_eq!(func.params.len(), 2);
                assert_eq!(func.params[0].id, type_system.type_int);
                assert_eq!(func.params[1].id, type_system.type_int);
                assert_eq!(func.return_type.id, type_system.type_int);
            }
            _ => panic!("Expected Function type"),
        }
    }

    #[test]
    fn test_create_reference_type() {
        let mut type_system = TypeSystem::new();
        
        // Créer &int
        let ref_type_id = type_system.create_reference_type(
            type_system.type_int,
            TypeMutability::Immutable,
            None, // Pas de lifetime spécifique
        );
        
        let ref_type = type_system.get_type(ref_type_id).unwrap();
        
        match &ref_type.kind {
            TypeKind::Reference(inner_type, mutability, _lifetime) => {
                assert_eq!(inner_type.id, type_system.type_int);
                assert_eq!(*mutability, TypeMutability::Immutable);
            }
            _ => panic!("Expected Reference type"),
        }
        
        // Créer &mut int
        let mut_ref_type_id = type_system.create_reference_type(
            type_system.type_int,
            TypeMutability::Mutable,
            None,
        );
        
        let mut_ref_type = type_system.get_type(mut_ref_type_id).unwrap();
        
        match &mut_ref_type.kind {
            TypeKind::Reference(inner_type, mutability, _lifetime) => {
                assert_eq!(inner_type.id, type_system.type_int);
                assert_eq!(*mutability, TypeMutability::Mutable);
            }
            _ => panic!("Expected mutable Reference type"),
        }
    }

    #[test]
    fn test_type_compatibility() {
        let type_system = TypeSystem::new();
        
        // Test compatibilité de base
        assert!(type_system.are_types_compatible(type_system.type_int, type_system.type_int));
        
        // int est compatible avec float (coercion implicite)
        assert!(type_system.are_types_compatible(type_system.type_int, type_system.type_float));
        
        // float n'est pas compatible avec int
        assert!(!type_system.are_types_compatible(type_system.type_float, type_system.type_int));
        
        // bool n'est pas compatible avec int
        assert!(!type_system.are_types_compatible(type_system.type_bool, type_system.type_int));
    }

    #[test]
    fn test_convert_ast_type() {
        let mut type_system = TypeSystem::new();
        
        // Convertir AST::Type::Int
        let ast_int = ast::Type::Int;
        let int_id = type_system.convert_ast_type(&ast_int);
        assert_eq!(int_id, type_system.type_int);
        
        // Convertir AST::Type::Array
        let ast_array = ast::Type::Array(Box::new(ast::Type::Float));
        let array_id = type_system.convert_ast_type(&ast_array);
        
        let array_type = type_system.get_type(array_id).unwrap();
        match &array_type.kind {
            TypeKind::Array(elem_type, _) => {
                assert_eq!(elem_type.id, type_system.type_float);
            }
            _ => panic!("Expected Array type"),
        }
    }

    #[test]
    fn test_type_inference_variable() {
        let mut type_system = TypeSystem::new();
        
        // Créer une variable de type pour l'inférence
        let type_var = type_system.create_type_variable(Some("T".to_string()));
        
        // Au début, la variable n'est pas résolue
        assert!(type_system.resolve_type_variable(&type_var).is_none());
        
        // Après unification avec int, elle devrait être résolue
        // Note: Ceci nécessiterait l'implémentation de unify, qui semble exister
        // mais nous ne pouvons pas facilement tester sans plus de contexte
    }

    #[test]
    fn test_semantic_analyzer_variable() {
        let mut analyzer = SemanticAnalyzer::new();
        
        // Créer une déclaration de variable: let x: int = 42
        let var_decl = VariableDeclaration {
            name: "x".to_string(),
            variable_type: Some(ASTType::Int),
            value: Some(Expression::Literal(Literal::Integer { value: 42.into() })),
            mutability: Mutability::Immutable,
        };
        
        let ast_node = ast::ASTNode::Declaration(Declaration::Variable(var_decl));
        
        // Analyser devrait fonctionner sans erreur
        let result = analyzer.analyze(&[ast_node]);
        assert!(result.is_ok(), "Variable declaration analysis failed: {:?}", result);
    }

    #[test]
    fn test_semantic_analyzer_type_mismatch() {
        let mut analyzer = SemanticAnalyzer::new();
        
        // Créer une déclaration avec type mismatch: let x: int = true
        let var_decl = VariableDeclaration {
            name: "y".to_string(),
            variable_type: Some(ASTType::Int),
            value: Some(Expression::Literal(Literal::Boolean(true))),
            mutability: Mutability::Immutable,
        };
        
        let ast_node = ast::ASTNode::Declaration(Declaration::Variable(var_decl));
        
        // L'analyse devrait échouer
        let result = analyzer.analyze(&[ast_node]);
        assert!(result.is_err(), "Type mismatch should have been detected");
    }

    #[test]
    fn test_lifetime_creation() {
        let mut type_system = TypeSystem::new();
        
        // Créer un lifetime
        let lifetime_a = type_system.create_lifetime(Some("'a".to_string()));
        
        // Le lifetime statique devrait toujours exister
        let static_lifetime = type_system.static_lifetime();
        
        // Ils devraient être différents
        assert_ne!(lifetime_a, static_lifetime);
    }

    #[test]
    fn test_type_count() {
        let mut type_system = TypeSystem::new();
        
        let initial_count = type_system.type_count();
        
        // Créer quelques types
        type_system.create_array_type(type_system.type_int, Some(10));
        type_system.create_tuple_type(vec![type_system.type_bool, type_system.type_float]);
        
        let new_count = type_system.type_count();
        
        // On devrait avoir créé 2 nouveaux types
        assert_eq!(new_count, initial_count + 2);
    }
}