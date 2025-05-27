

// src/semantic/types/tests.rs

use punk::semantic::types::type_system::TypeSystem;

use punk::parser::ast;
use punk::semantic::types::type_system::{Type, TypeId, TypeKind};


use punk::semantic::symbols::{SymbolId, ScopeId, SourceLocation};
use punk::semantic::borrow_checker::{BorrowChecker, BorrowKind};

use punk::semantic::symbol_table::SymbolTable;
use punk::semantic::symbols::{ScopeKind, SymbolKind};


use punk::parser::ast::{ASTNode, Expression, ReturnStatement};
use punk::semantic::analyser::SemanticAnalyzer;
use punk::parser::ast::{Declaration, Literal, Mutability, VariableDeclaration};

use punk::parser::ast::Type as ASTType;

#[cfg(test)]
mod tests {

    use super::*;



    #[test]
    fn test_type_creation() {
        let mut type_system = TypeSystem::new();
        let type_registry = &mut type_system.type_registry;

        // Tester la création des types primitifs
        assert!(type_registry.get_type(type_registry.type_int).is_some());
        assert!(type_registry.get_type(type_registry.type_float).is_some());
        assert!(type_registry.get_type(type_registry.type_bool).is_some());

        // Tester la création d'un type array
        let array_type_id = type_registry.create_array_type(type_registry.type_int, Some(5));
        let array_type = type_registry.get_type(array_type_id).unwrap();

        match &array_type.kind {
            TypeKind::Array(elem_type, size) => {
                assert_eq!(elem_type.id, type_registry.type_int);
                assert_eq!(*size, Some(5));
            },
            _ => panic!("Expected Array type"),
        }
    }

    #[test]
    fn test_type_compatibility() {
        let mut type_system = TypeSystem::new();
        let type_registry = &mut type_system.type_registry;

        // Récupérer les types
        let int_type = type_registry.get_type(type_registry.type_int).unwrap();
        let float_type = type_registry.get_type(type_registry.type_float).unwrap();
        let bool_type = type_registry.get_type(type_registry.type_bool).unwrap();

        // Tester la compatibilité
        assert!(int_type.is_compatible_with(int_type));  // int est compatible avec int
        assert!(int_type.is_compatible_with(float_type)); // int est compatible avec float (conversion implicite)
        assert!(!int_type.is_compatible_with(bool_type)); // int n'est pas compatible avec bool
    }

    #[test]
    fn test_ast_type_conversion() {
        let mut type_system = TypeSystem::new();
        let type_registry = &mut type_system.type_registry;

        // Créer un type AST
        let ast_int = ast::Type::Int;
        let ast_array = ast::Type::Array(Box::new(ast::Type::Int));

        // Convertir les types AST
        let int_type_id = type_registry.convert_ast_type(&ast_int);
        let array_type_id = type_registry.convert_ast_type(&ast_array);

        // Vérifier la conversion
        assert_eq!(int_type_id, type_registry.type_int);

        let array_type = type_registry.get_type(array_type_id).unwrap();
        match &array_type.kind {
            TypeKind::Array(elem_type, _) => {
                assert_eq!(elem_type.id, type_registry.type_int);
            },
            _ => panic!("Expected Array type"),
        }
    }

    #[test]
    fn test_type_inference() {
        let mut type_system = TypeSystem::new();

        // Créer des types pour le test d'unification
        let int_type = type_system.type_registry.get_type(type_system.type_registry.type_int).unwrap().clone();

        // Créer une variable de type
        let type_var = type_system.create_type_variable(Some("T".to_string()));
        let infer_type = Type::new(
            TypeId(100), // ID arbitraire
            TypeKind::Infer(type_var.clone())
        );

        // Unifier la variable avec un int
        let result = type_system.unify(&infer_type, &int_type).unwrap();

        // Vérifier que la variable a été correctement instanciée
        assert_eq!(result.kind, int_type.kind);

        // Vérifier que la variable est résolue
        let resolved = type_system.resolve_type_variable(&type_var);
        assert!(resolved.is_some());
        if let Some(resolved_type) = resolved {
            assert_eq!(resolved_type.kind, int_type.kind);
        }
    }
}


// src/semantic/symbols/tests.rs


    #[test]
    fn test_symbol_declaration() {
        let mut symbol_table = SymbolTable::new();

        // Créer un symbole
        let location = SourceLocation {
            file: "test.rs".to_string(),
            line: 1,
            column: 1,
        };

        let symbol_id = symbol_table.declare_symbol(
            "test_var".to_string(),
            SymbolKind::Variable,
            location.clone()
        ).unwrap();

        // Vérifier que le symbole a été créé
        let symbol = symbol_table.get_symbol(symbol_id).unwrap();
        assert_eq!(symbol.name, "test_var");
        assert_eq!(symbol.kind, SymbolKind::Variable);
    }

    #[test]
    fn test_symbol_lookup() {
        let mut symbol_table = SymbolTable::new();

        // Créer un symbole
        let location = SourceLocation {
            file: "test.rs".to_string(),
            line: 1,
            column: 1,
        };

        symbol_table.declare_symbol(
            "test_var".to_string(),
            SymbolKind::Variable,
            location.clone()
        ).unwrap();

        // Rechercher le symbole
        let found_id = symbol_table.lookup_symbol("test_var").unwrap();
        let found_symbol = symbol_table.get_symbol(found_id).unwrap();

        assert_eq!(found_symbol.name, "test_var");
        assert_eq!(found_symbol.kind, SymbolKind::Variable);
    }

    #[test]
    fn test_scope_management() {
        let mut symbol_table = SymbolTable::new();

        // Créer un symbole dans le scope global
        let location = SourceLocation {
            file: "test.rs".to_string(),
            line: 1,
            column: 1,
        };

        symbol_table.declare_symbol(
            "global_var".to_string(),
            SymbolKind::Variable,
            location.clone()
        ).unwrap();

        // Entrer dans un nouveau scope
        let function_scope = symbol_table.enter_scope(ScopeKind::Function);

        // Créer un symbole dans le scope de fonction
        symbol_table.declare_symbol(
            "local_var".to_string(),
            SymbolKind::Variable,
            location.clone()
        ).unwrap();

        // Rechercher les symboles
        assert!(symbol_table.lookup_symbol("global_var").is_ok()); // Visible depuis le scope de fonction
        assert!(symbol_table.lookup_symbol("local_var").is_ok());  // Visible dans le scope actuel

        // Sortir du scope de fonction
        symbol_table.exit_scope().unwrap();

        // Vérifier la visibilité
        assert!(symbol_table.lookup_symbol("global_var").is_ok());     // Toujours visible
        assert!(symbol_table.lookup_symbol("local_var").is_err());     // Plus visible

        // Rechercher dans un scope spécifique
        assert!(symbol_table.lookup_symbol_in_scope("local_var", function_scope).is_ok());
    }

    #[test]
    fn test_symbol_with_type() {
        let mut symbol_table = SymbolTable::new();

        let location = SourceLocation {
            file: "test.rs".to_string(),
            line: 1,
            column: 1,
        };

        // Déclarer un symbole avec un type
        let type_id = symbol_table.type_system.type_registry.type_int;

        let symbol_id = symbol_table.declare_symbol_with_type(
            "typed_var".to_string(),
            SymbolKind::Variable,
            type_id,
            location.clone(),
            false  // Non mutable
        ).unwrap();

        // Vérifier le type
        let symbol_type_id = symbol_table.get_symbol_type_id(symbol_id).unwrap().unwrap();
        assert_eq!(symbol_type_id, type_id);
    }


    #[test]
    fn test_variable_declaration_analysis() {
        let mut analyzer = SemanticAnalyzer::new();

        // Créer une déclaration de variable simple
        let var_decl = VariableDeclaration {
            name: "x".to_string(),
            variable_type: Some(ASTType::Int),
            value: Some(Expression::Literal(Literal::Integer { value: 42.into()})),
            mutability: Mutability::Immutable
        };

        let ast_node = ASTNode::Declaration(Declaration::Variable(var_decl));

        // Analyser l'AST
        assert!(analyzer.analyze(&[ast_node]).is_ok());

        // Vérifier que le symbole a été créé avec le bon type
        let symbol_id = analyzer.symbol_table.lookup_symbol("x").unwrap();
        let symbol_type = analyzer.symbol_table.get_symbol_type(symbol_id).unwrap().unwrap();

        match symbol_type.kind {
            TypeKind::Int => {} // OK
            _ => panic!("Expected Int type, got {:?}", symbol_type.kind),
        }
    }

    #[test]
    fn test_type_compatibility_check() {
        let mut analyzer = SemanticAnalyzer::new();

        // Créer une déclaration de variable avec un type explicite et une valeur incompatible
        let var_decl = VariableDeclaration {
            name: "x".to_string(),
            variable_type: Some(ASTType::Int),
            value: Some(Expression::Literal(Literal::Boolean(true))),
            mutability: Mutability::Immutable
        };

        let ast_node = ASTNode::Declaration(Declaration::Variable(var_decl));

        // L'analyse devrait échouer à cause de l'incompatibilité de types
        assert!(analyzer.analyze(&[ast_node]).is_err());
    }

    #[test]
    fn test_expression_type_inference() {
        let mut analyzer = SemanticAnalyzer::new();

        // Créer une expression pour le test
        let expr = Expression::BinaryOperation(
            *Box::new(
                ast::BinaryOperation {
                    left: Box::new(Expression::Literal(Literal::Integer { value: 5.into() })),
                    operator: ast::Operator::Addition,
                    right: Box::new(Expression::Literal(Literal::Integer { value: 3.into()}))
                }
            )
        );

        // Analyser l'expression
        let type_id = analyzer.analyze_expression(&expr).unwrap();
        let type_obj = analyzer.type_checker.type_system.type_registry.get_type(type_id).unwrap();

        // Le résultat de 5 + 3 devrait être de type Int
        match type_obj.kind {
            TypeKind::Int => {} // OK
            _ => panic!("Expected Int type, got {:?}", type_obj.kind),
        }
    }

    // #[test]
    // fn test_function_declaration_analysis() {
    //     let mut analyzer = SemanticAnalyzer::new();
    //
    //     // Créer une déclaration de fonction simple
    //     let func_decl = ast::FunctionDeclaration {
    //         name: "add".to_string(),
    //         parameters: vec![
    //             ast::Parameter {
    //                 name: "a".to_string(),
    //                 parameter_type: ASTType::Int,
    //             },
    //             ast::Parameter {
    //                 name: "b".to_string(),
    //                 parameter_type: ASTType::Int,
    //             }
    //         ],
    //         return_type: Some(ASTType::Int),
    //         body: vec![ReturnStatement(Some(
    //                 Expression::BinaryOperation(
    //                     *Box::new(
    //                         ast::BinaryOperation {
    //                             left: Box::new(Expression::Identifier("a".to_string())),
    //                             operator: ast::Operator::Addition,
    //                             right: Box::new(Expression::Identifier("b".to_string()))
    //                         }
    //                     )
    //                 )
    //             ))
    //         ],
    //         visibility: ast::Visibility::Public,
    //     };
    //
    //     let ast_node = ASTNode::Declaration(Declaration::Function(func_decl));
    //
    //     // Analyser l'AST
    //     assert!(analyzer.analyze(&[ast_node]).is_ok());
    //
    //     // Vérifier que le symbole de fonction a été créé
    //     let func_id = analyzer.symbol_table.lookup_symbol("add").unwrap();
    //     let func_symbol = analyzer.symbol_table.get_symbol(func_id).unwrap();
    //
    //     assert_eq!(func_symbol.kind, SymbolKind::Function);
    // }





// src/semantic/borrow_checker/tests.rs

// #[cfg(test)]


    // use super::*;
    #[test]
    fn test_basic_borrows() {
        let mut borrow_checker = BorrowChecker::new();

        let symbol_id = SymbolId(1);
        let scope_id = ScopeId(1);
        let location = SourceLocation {
            file: "test.rs".to_string(),
            line: 1,
            column: 1,
        };

        // Enregistrer un emprunt de lecture
        assert!(borrow_checker.register_borrow(
            symbol_id,
            BorrowKind::Read,
            location.clone(),
            scope_id,
            None
        ).is_ok());

        // Un autre emprunt de lecture devrait être OK
        assert!(borrow_checker.register_borrow(
            symbol_id,
            BorrowKind::Read,
            location.clone(),
            scope_id,
            None
        ).is_ok());

        // Un emprunt d'écriture devrait échouer à cause des emprunts de lecture existants
        assert!(borrow_checker.register_borrow(
            symbol_id,
            BorrowKind::Write,
            location.clone(),
            scope_id,
            None
        ).is_err());
    }

    #[test]
    fn test_mutable_borrows() {
        let mut borrow_checker = BorrowChecker::new();

        let symbol_id = SymbolId(1);
        let scope_id = ScopeId(1);
        let location = SourceLocation {
            file: "test.rs".to_string(),
            line: 1,
            column: 1,
        };

        // Enregistrer un emprunt mutable
        assert!(borrow_checker.register_borrow(
            symbol_id,
            BorrowKind::Mutable,
            location.clone(),
            scope_id,
            None
        ).is_ok());

        // Un autre emprunt mutable devrait échouer
        assert!(borrow_checker.register_borrow(
            symbol_id,
            BorrowKind::Mutable,
            location.clone(),
            scope_id,
            None
        ).is_err());

        // Un emprunt de lecture devrait également échouer
        assert!(borrow_checker.register_borrow(
            symbol_id,
            BorrowKind::Read,
            location.clone(),
            scope_id,
            None
        ).is_err());
    }

    #[test]
    fn test_scope_release() {
        let mut borrow_checker = BorrowChecker::new();

        let symbol_id = SymbolId(1);
        let scope_id = ScopeId(1);
        let child_scope_id = ScopeId(2);
        let location = SourceLocation {
            file: "test.rs".to_string(),
            line: 1,
            column: 1,
        };

        // Enregistrer un emprunt dans le scope enfant
        assert!(borrow_checker.register_borrow(
            symbol_id,
            BorrowKind::Mutable,
            location.clone(),
            child_scope_id,
            None
        ).is_ok());

        // Libérer les emprunts du scope enfant
        borrow_checker.release_borrows_for_scope(child_scope_id);

        // Maintenant un nouvel emprunt devrait être possible
        assert!(borrow_checker.register_borrow(
            symbol_id,
            BorrowKind::Mutable,
            location.clone(),
            scope_id,
            None
        ).is_ok());
    }

    #[test]
    fn test_initialization_tracking() {
        let mut borrow_checker = BorrowChecker::new();

        let symbol_id = SymbolId(1);

        // Au départ, le symbole n'est pas initialisé
        assert!(!borrow_checker.is_initialized(symbol_id));

        // Marquer le symbole comme initialisé
        borrow_checker.mark_initialized(symbol_id);

        // Maintenant il devrait être initialisé
        assert!(borrow_checker.is_initialized(symbol_id));
    }
