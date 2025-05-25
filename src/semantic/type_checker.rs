//src/semantic/type_checker.rs

use std::cell::RefCell;
use std::rc::Rc;
use crate::parser::ast::{Expression, Statement, Operator, UnaryOperator, Literal,
                         VariableDeclaration, FunctionDeclaration, ASTNode, Declaration};
use crate::semantic::symbols::{SymbolId, SourceLocation, SymbolKind};
use crate::semantic::types::type_system::{TypeId, TypeKind, TypeSystem, Mutability};
use crate::semantic::semantic_error::{SemanticError, TypeError, SemanticErrorType, Position};
use crate::semantic::symbol_table::SymbolTable;
use num_bigint::BigInt;

pub struct TypeChecker {
    pub symbol_table: SymbolTable,
    pub type_system: TypeSystem,
    // pub symbol_table: Rc<RefCell<SymbolTable>>,
    // pub type_system: Rc<RefCell<TypeSystem>>,

}

impl TypeChecker {
    pub fn new(symbol_table: SymbolTable) -> Self {
        TypeChecker {
            symbol_table,
            type_system: TypeSystem::new(),
        }
    }

    /// Vérifie et infère le type d'une expression
    pub fn check_expression(&mut self, expr: &Expression) -> Result<TypeId, SemanticError> {
        match expr {
            Expression::Literal(literal) => {
                self.check_literal(literal)
            },

            Expression::Identifier(name) => {
                // Rechercher l'identifiant dans la table des symboles
                let symbol_id = self.symbol_table.lookup_symbol(name)?;

                // Récupérer le type associé au symbole
                if let Some(type_obj) = self.symbol_table.get_symbol_type(symbol_id)? {
                    Ok(type_obj.id)
                } else {
                    Err(create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::UndefinedType(name.clone())),
                        "Variable used before its type is defined".to_string(),
                        Position { index: 0 }
                    ))
                }
            },

            Expression::BinaryOperation(bin_op) => {
                self.check_binary_expression(&bin_op.left, &bin_op.operator, &bin_op.right)
            },

            Expression::UnaryOperation(un_op) => {
                self.check_unary_expression(&un_op.operator, &un_op.operand)
            },

            Expression::FunctionCall(call) => {
                self.check_function_call(&call.name, &call.arguments)
            },

            Expression::Array(array_expr) => {
                self.check_array_literal(&array_expr.elements)
            },

            Expression::IndexAccess(access) => {
                self.check_array_access(&access.array, &access.index)
            },

            Expression::MemberAccess(access) => {
                self.check_member_access(&access.object, &access.member)
            },

            Expression::Assignment(assignment) => {
                self.check_assignment(&assignment.target, &assignment.value)
            },

            Expression::MethodCall(method_call) => {
                self.check_method_call(&method_call.object, &method_call.method, &method_call.arguments)
            },

            Expression::TypeCast(cast) => {
                self.check_type_cast(&cast.expression, &cast.target_type)
            },

            // Plus de cas selon votre AST...
            _ => {
                // Cas par défaut pour les expressions non gérées
                Err(create_semantic_error(
                    SemanticErrorType::TypeError(TypeError::InvalidType("Unsupported expression type".to_string())),
                    "Expression type not supported yet".to_string(),
                    Position { index: 0 }
                ))
            }
        }
    }

    /// Vérifie le type d'un littéral
    fn check_literal(&mut self, literal: &Literal) -> Result<TypeId, SemanticError> {
        match literal {
            Literal::Integer { .. } => Ok(self.type_system.type_registry.type_int),
            Literal::Float { .. } => Ok(self.type_system.type_registry.type_float),
            Literal::Boolean(_) => Ok(self.type_system.type_registry.type_bool),
            Literal::String(_) => Ok(self.type_system.type_registry.type_string),
            Literal::Char(_) => Ok(self.type_system.type_registry.type_char),
            Literal::Array(elements) => {
                self.check_array_literal(elements)
            },
        }
    }

    /// Vérifie les types d'une expression binaire
    fn check_binary_expression(
        &mut self,
        left: &Box<Expression>,
        operator: &Operator,
        right: &Box<Expression>
    ) -> Result<TypeId, SemanticError> {
        let left_type_id = self.check_expression(left)?;
        let right_type_id = self.check_expression(right)?;

        // Récupérer les objets Type
        let left_type = self.type_system.type_registry.get_type(left_type_id)
            .ok_or_else(|| create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", left_type_id))),
                "Type not found".to_string(),
                Position { index: 0 }
            ))?;

        let right_type = self.type_system.type_registry.get_type(right_type_id)
            .ok_or_else(|| create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", right_type_id))),
                "Type not found".to_string(),
                Position { index: 0 }
            ))?;

        // Vérifier la compatibilité des opérandes selon l'opérateur
        match operator {
            Operator::Addition | Operator::Substraction |
            Operator::Multiplication | Operator::Division |
            Operator::Modulo => {
                // Opérations arithmétiques
                match (&left_type.kind, &right_type.kind) {
                    (TypeKind::Int, TypeKind::Int) => Ok(left_type_id), // int op int -> int
                    (TypeKind::Float, TypeKind::Float) => Ok(left_type_id), // float op float -> float
                    (TypeKind::Int, TypeKind::Float) => Ok(right_type_id), // int op float -> float
                    (TypeKind::Float, TypeKind::Int) => Ok(left_type_id), // float op int -> float
                    (TypeKind::String, TypeKind::String) if *operator == Operator::Addition => {
                        // Concaténation de chaînes
                        Ok(left_type_id) // string + string -> string
                    },
                    _ => Err(create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::TypeMismatch(
                            format!("Cannot apply operator {:?} to types {} and {}",
                                    operator, left_type, right_type)
                        )),
                        "Incompatible types for binary operation".to_string(),
                        Position { index: 0 }
                    ))
                }
            },

            Operator::EqualEqual | Operator::NotEqual => {
                // Opérations d'égalité (==, !=) peuvent être appliquées à tous les types comparables
                if left_type.is_compatible_with(right_type) || right_type.is_compatible_with(left_type) {
                    Ok(self.type_system.type_registry.type_bool) // Résultat est toujours bool
                } else {
                    Err(create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::TypeMismatch(
                            format!("Cannot compare types {} and {}", left_type, right_type)
                        )),
                        "Incompatible types for comparison".to_string(),
                        Position { index: 0 }
                    ))
                }
            },

            Operator::LessThan | Operator::LesshanOrEqual |
            Operator::GreaterThan | Operator::GreaterThanOrEqual => {
                // Opérations de comparaison (<, <=, >, >=)
                match (&left_type.kind, &right_type.kind) {
                    (TypeKind::Int, TypeKind::Int) |
                    (TypeKind::Float, TypeKind::Float) |
                    (TypeKind::Int, TypeKind::Float) |
                    (TypeKind::Float, TypeKind::Int) |
                    (TypeKind::Char, TypeKind::Char) => {
                        Ok(self.type_system.type_registry.type_bool) // Résultat est toujours bool
                    },
                    _ => Err(create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::TypeMismatch(
                            format!("Cannot compare types {} and {} with operator {:?}",
                                    left_type, right_type, operator)
                        )),
                        "Incompatible types for comparison".to_string(),
                        Position { index: 0 }
                    ))
                }
            },

            Operator::And | Operator::Or => {
                // Opérations logiques (&&, ||)
                if left_type.kind == TypeKind::Bool && right_type.kind == TypeKind::Bool {
                    Ok(self.type_system.type_registry.type_bool) // bool op bool -> bool
                } else {
                    Err(create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::TypeMismatch(
                            format!("Logical operators require boolean operands, got {} and {}",
                                    left_type, right_type)
                        )),
                        "Logical operation requires boolean operands".to_string(),
                        Position { index: 0 }
                    ))
                }
            },

            // Assignment operator
            Operator::Equal => {
                // Pour l'assignation, vérifier que les types sont compatibles
                if right_type.is_compatible_with(left_type) {
                    Ok(left_type_id) // Retourner le type de la variable assignée
                } else {
                    Err(create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::TypeMismatch(
                            format!("Cannot assign value of type {} to variable of type {}",
                                    right_type, left_type)
                        )),
                        "Type mismatch in assignment".to_string(),
                        Position { index: 0 }
                    ))
                }
            },

            // Range operators
            Operator::Range | Operator::RangeInclusive => {
                // Les ranges nécessitent des types compatibles
                if left_type.is_compatible_with(right_type) || right_type.is_compatible_with(left_type) {
                    // Créer un type range (pour l'instant, utilisons un type nommé)
                    let range_type_id = self.type_system.type_registry.register_type(
                        TypeKind::Named("Range".to_string(), vec![left_type.clone()])
                    );
                    Ok(range_type_id)
                } else {
                    Err(create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::TypeMismatch(
                            format!("Range bounds must be of compatible types, got {} and {}",
                                    left_type, right_type)
                        )),
                        "Incompatible types for range".to_string(),
                        Position { index: 0 }
                    ))
                }
            },
        }
    }

    /// Vérifie les types d'une expression unaire
    fn check_unary_expression(
        &mut self,
        operator: &UnaryOperator,
        operand: &Box<Expression>
    ) -> Result<TypeId, SemanticError> {
        let operand_type_id = self.check_expression(operand)?;
        let operand_type = self.type_system.type_registry.get_type(operand_type_id)
            .ok_or_else(|| create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", operand_type_id))),
                "Type not found".to_string(),
                Position { index: 0 }
            ))?;

        match operator {
            UnaryOperator::Negate | UnaryOperator::Negative => {
                match &operand_type.kind {
                    TypeKind::Int | TypeKind::Float => Ok(operand_type_id),
                    _ => Err(create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::TypeMismatch(
                            format!("Cannot negate type {}", operand_type)
                        )),
                        "Invalid type for negation".to_string(),
                        Position { index: 0 }
                    ))
                }
            },

            UnaryOperator::Not | UnaryOperator::LogicalNot => {
                match &operand_type.kind {
                    TypeKind::Bool => Ok(self.type_system.type_registry.type_bool),
                    _ => Err(create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::TypeMismatch(
                            format!("Cannot apply logical not to type {}", operand_type)
                        )),
                        "Invalid type for logical not".to_string(),
                        Position { index: 0 }
                    ))
                }
            },

            UnaryOperator::Reference => {
                // Créer une référence immutable
                let ref_type_id = self.type_system.type_registry.create_reference_type(
                    operand_type_id,
                    Mutability::Immutable,
                    None
                );
                Ok(ref_type_id)
            },

            UnaryOperator::ReferenceMutable => {
                // Créer une référence mutable
                let ref_type_id = self.type_system.type_registry.create_reference_type(
                    operand_type_id,
                    Mutability::Mutable,
                    None
                );
                Ok(ref_type_id)
            },

            UnaryOperator::Dereference => {
                // Déréférencer une référence
                match &operand_type.kind {
                    TypeKind::Reference(inner_type, _, _) => Ok(inner_type.id),
                    _ => Err(create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::TypeMismatch(
                            format!("Cannot dereference non-reference type {}", operand_type)
                        )),
                        "Invalid type for dereference".to_string(),
                        Position { index: 0 }
                    ))
                }
            },

            UnaryOperator::Positive => {
                match &operand_type.kind {
                    TypeKind::Int | TypeKind::Float => Ok(operand_type_id),
                    _ => Err(create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::TypeMismatch(
                            format!("Cannot apply unary plus to type {}", operand_type)
                        )),
                        "Invalid type for unary plus".to_string(),
                        Position { index: 0 }
                    ))
                }
            },

            UnaryOperator::BitwiseNot => {
                match &operand_type.kind {
                    TypeKind::Int => Ok(operand_type_id),
                    _ => Err(create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::TypeMismatch(
                            format!("Cannot apply bitwise not to type {}", operand_type)
                        )),
                        "Invalid type for bitwise not".to_string(),
                        Position { index: 0 }
                    ))
                }
            },

            UnaryOperator::Increment | UnaryOperator::Decrement => {
                match &operand_type.kind {
                    TypeKind::Int => Ok(operand_type_id),
                    _ => Err(create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::TypeMismatch(
                            format!("Cannot apply increment/decrement to type {}", operand_type)
                        )),
                        "Invalid type for increment/decrement".to_string(),
                        Position { index: 0 }
                    ))
                }
            },
        }
    }

    /// Vérifie un appel de fonction
    fn check_function_call(
        &mut self,
        function: &Box<Expression>,
        arguments: &Vec<Expression>
    ) -> Result<TypeId, SemanticError> {
        let function_type_id = self.check_expression(function)?;

        // Clone le type de fonction pour éviter les problèmes d'emprunt
        let func_type_clone;
        {
            let function_type = self.type_system.type_registry.get_type(function_type_id)
                .ok_or_else(|| create_semantic_error(
                    SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", function_type_id))),
                    "Type not found".to_string(),
                    Position { index: 0 }
                ))?;

            match &function_type.kind {
                TypeKind::Function(func_type) => {
                    // Clone des informations nécessaires
                    func_type_clone = func_type.clone();
                },
                _ => {
                    return Err(create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::TypeMismatch(
                            format!("Cannot call non-function type {}", function_type)
                        )),
                        "Attempt to call non-function".to_string(),
                        Position { index: 0 }
                    ));
                }
            }
        }

        // Maintenant nous pouvons utiliser func_type_clone sans conflit d'emprunt

        // Vérifier le nombre d'arguments
        if arguments.len() != func_type_clone.params.len() {
            return Err(create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeMismatch(
                    format!("Function expects {} arguments, got {}",
                            func_type_clone.params.len(), arguments.len())
                )),
                "Incorrect number of arguments".to_string(),
                Position { index: 0 }
            ));
        }

        // Vérifier les types des arguments
        for (i, (arg, param_type)) in arguments.iter().zip(func_type_clone.params.iter()).enumerate() {
            let arg_type_id = self.check_expression(arg)?;
            let arg_type = self.type_system.type_registry.get_type(arg_type_id)
                .ok_or_else(|| create_semantic_error(
                    SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", arg_type_id))),
                    "Type not found".to_string(),
                    Position { index: 0 }
                ))?;

            if !arg_type.is_compatible_with(param_type) {
                return Err(create_semantic_error(
                    SemanticErrorType::TypeError(TypeError::TypeMismatch(
                        format!("Argument {} has type {}, expected {}",
                                i + 1, arg_type, param_type)
                    )),
                    "Type mismatch in function call".to_string(),
                    Position { index: 0 }
                ));
            }
        }

        // Retourner le type de retour de la fonction
        Ok(func_type_clone.return_type.id)
    }

    /// Vérifie un littéral array
    fn check_array_literal(&mut self, elements: &Vec<Expression>) -> Result<TypeId, SemanticError> {
        if elements.is_empty() {
            // Array vide - créer un array de type inféré
            let infer_type_var = self.type_system.create_type_variable(Some("ArrayElement".to_string()));
            let infer_type_id = self.type_system.type_registry.register_type(TypeKind::Infer(infer_type_var));
            return Ok(self.type_system.type_registry.create_array_type(infer_type_id, Some(0)));
        }

        // Vérifier le type du premier élément
        let first_element_type_id = self.check_expression(&elements[0])?;
        let first_element_type = self.type_system.type_registry.get_type(first_element_type_id)
            .ok_or_else(|| create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", first_element_type_id))),
                "Type not found".to_string(),
                Position { index: 0 }
            ))?.clone();

        // Vérifier que tous les éléments ont le même type
        for (i, element) in elements.iter().skip(1).enumerate() {
            let element_type_id = self.check_expression(element)?;
            let element_type = self.type_system.type_registry.get_type(element_type_id)
                .ok_or_else(|| create_semantic_error(
                    SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", element_type_id))),
                    "Type not found".to_string(),
                    Position { index: 0 }
                ))?;

            if !element_type.is_compatible_with(&first_element_type) {
                return Err(create_semantic_error(
                    SemanticErrorType::TypeError(TypeError::TypeMismatch(
                        format!("Array element {} has type {}, expected {}",
                                i + 2, element_type, first_element_type)
                    )),
                    "Inconsistent array element types".to_string(),
                    Position { index: 0 }
                ));
            }
        }

        // Créer le type array avec la taille
        Ok(self.type_system.type_registry.create_array_type(first_element_type_id, Some(elements.len())))
    }

    /// Vérifie un accès à un array
    fn check_array_access(
        &mut self,
        array: &Box<Expression>,
        index: &Box<Expression>
    ) -> Result<TypeId, SemanticError> {
        let array_type_id = self.check_expression(array)?;
        let index_type_id = self.check_expression(index)?;

        let array_type = self.type_system.type_registry.get_type(array_type_id)
            .ok_or_else(|| create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", array_type_id))),
                "Type not found".to_string(),
                Position { index: 0 }
            ))?;

        let index_type = self.type_system.type_registry.get_type(index_type_id)
            .ok_or_else(|| create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", index_type_id))),
                "Type not found".to_string(),
                Position { index: 0 }
            ))?;

        // Vérifier que l'index est un entier
        if index_type.kind != TypeKind::Int {
            return Err(create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeMismatch(
                    format!("Array index must be int, got {}", index_type)
                )),
                "Invalid array index type".to_string(),
                Position { index: 0 }
            ));
        }

        // Vérifier que l'expression est bien un array
        match &array_type.kind {
            TypeKind::Array(element_type, _) => Ok(element_type.id),
            _ => Err(create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeMismatch(
                    format!("Cannot index non-array type {}", array_type)
                )),
                "Invalid array access".to_string(),
                Position { index: 0 }
            ))
        }
    }

    /// Vérifie un accès à un membre
    fn check_member_access(
        &mut self,
        object: &Box<Expression>,
        member: &str
    ) -> Result<TypeId, SemanticError> {
        let object_type_id = self.check_expression(object)?;
        let object_type = self.type_system.type_registry.get_type(object_type_id)
            .ok_or_else(|| create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", object_type_id))),
                "Type not found".to_string(),
                Position { index: 0 }
            ))?;

        // Pour l'instant, une implémentation simplifiée
        // Dans une vraie implémentation, il faudrait chercher le membre dans la structure
        match &object_type.kind {
            TypeKind::Struct(_) | TypeKind::Named(_, _) => {
                // Supposer que le membre existe et retourner un type inféré
                let member_type_var = self.type_system.create_type_variable(Some(format!("{}Member", member)));
                Ok(self.type_system.type_registry.register_type(TypeKind::Infer(member_type_var)))
            },
            _ => Err(create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeMismatch(
                    format!("Cannot access member '{}' on type {}", member, object_type)
                )),
                "Invalid member access".to_string(),
                Position { index: 0 }
            ))
        }
    }

    /// Vérifie une assignation
    fn check_assignment(
        &mut self,
        target: &Box<Expression>,
        value: &Box<Expression>
    ) -> Result<TypeId, SemanticError> {
        let target_type_id = self.check_expression(target)?;
        let value_type_id = self.check_expression(value)?;

        let target_type = self.type_system.type_registry.get_type(target_type_id)
            .ok_or_else(|| create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", target_type_id))),
                "Type not found".to_string(),
                Position { index: 0 }
            ))?;

        let value_type = self.type_system.type_registry.get_type(value_type_id)
            .ok_or_else(|| create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", value_type_id))),
                "Type not found".to_string(),
                Position { index: 0 }
            ))?;

        // Vérifier la compatibilité des types
        if !value_type.is_compatible_with(target_type) {
            return Err(create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeMismatch(
                    format!("Cannot assign value of type {} to variable of type {}",
                            value_type, target_type)
                )),
                "Type mismatch in assignment".to_string(),
                Position { index: 0 }
            ));
        }

        // L'assignation retourne le type de la variable assignée
        Ok(target_type_id)
    }

    /// Vérifie un appel de méthode
    fn check_method_call(
        &mut self,
        object: &Box<Expression>,
        method: &str,
        arguments: &Vec<Expression>
    ) -> Result<TypeId, SemanticError> {
        let object_type_id = self.check_expression(object)?;

        // Pour l'instant, une implémentation simplifiée
        // Dans une vraie implémentation, il faudrait chercher la méthode dans le type

        // Vérifier les arguments
        for arg in arguments {
            self.check_expression(arg)?;
        }

        // Retourner un type inféré pour la méthode
        let method_return_var = self.type_system.create_type_variable(Some(format!("{}Return", method)));
        Ok(self.type_system.type_registry.register_type(TypeKind::Infer(method_return_var)))
    }

    /// Vérifie un cast de type
    fn check_type_cast(
        &mut self,
        expression: &Box<Expression>,
        target_type: &crate::parser::ast::Type
    ) -> Result<TypeId, SemanticError> {
        let expr_type_id = self.check_expression(expression)?;
        let target_type_id = self.type_system.type_registry.convert_ast_type(target_type);

        let expr_type = self.type_system.type_registry.get_type(expr_type_id)
            .ok_or_else(|| create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", expr_type_id))),
                "Type not found".to_string(),
                Position { index: 0 }
            ))?;

        let target_type_obj = self.type_system.type_registry.get_type(target_type_id)
            .ok_or_else(|| create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", target_type_id))),
                "Type not found".to_string(),
                Position { index: 0 }
            ))?;

        // Vérifier que le cast est valide
        match (&expr_type.kind, &target_type_obj.kind) {
            // Casts numériques autorisés
            (TypeKind::Int, TypeKind::Float) |
            (TypeKind::Float, TypeKind::Int) |
            (TypeKind::Int, TypeKind::Char) |
            (TypeKind::Char, TypeKind::Int) => Ok(target_type_id),

            // Cast vers le même type
            (a, b) if a == b => Ok(target_type_id),

            _ => Err(create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeMismatch(
                    format!("Cannot cast type {} to {}", expr_type, target_type_obj)
                )),
                "Invalid type cast".to_string(),
                Position { index: 0 }
            ))
        }
    }

    /// Vérifie une déclaration de variable
    pub fn check_variable_declaration(
        &mut self,
        var_decl: &VariableDeclaration
    ) -> Result<TypeId, SemanticError> {
        let inferred_type = match (&var_decl.variable_type, &var_decl.value) {
            (Some(ast_type), Some(expr)) => {
                // A la fois un type explicite et un initializer
                let declared_type_id = self.type_system.type_registry.convert_ast_type(ast_type);
                let expr_type_id = self.check_expression(expr)?;

                // Vérifier la compatibilité des types
                let expr_type = self.type_system.type_registry.get_type(expr_type_id)
                    .ok_or_else(|| create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", expr_type_id))),
                        "Type not found".to_string(),
                        Position { index: 0 }
                    ))?;

                let declared_type = self.type_system.type_registry.get_type(declared_type_id)
                    .ok_or_else(|| create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", declared_type_id))),
                        "Type not found".to_string(),
                        Position { index: 0 }
                    ))?;

                if !expr_type.is_compatible_with(declared_type) {
                    return Err(create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::TypeMismatch(
                            format!("Cannot assign type {} to variable of type {}",
                                    expr_type, declared_type)
                        )),
                        "Type mismatch in variable initialization".to_string(),
                        Position { index: 0 }
                    ));
                }

                declared_type_id
            },

            (Some(ast_type), None) => {
                // Type explicite sans initializer
                self.type_system.type_registry.convert_ast_type(ast_type)
            },

            (None, Some(expr)) => {
                // Initializer sans type explicite (inférence)
                self.check_expression(expr)?
            },

            (None, None) => {
                // Ni type explicite ni initializer - erreur
                return Err(create_semantic_error(
                    SemanticErrorType::TypeError(TypeError::UndefinedType(var_decl.name.clone())),
                    "Variable declaration without type or initializer".to_string(),
                    Position { index: 0 }
                ));
            }
        };

        Ok(inferred_type)
    }

    /// Vérifie une déclaration de fonction
    pub fn check_function_declaration(
        &mut self,
        func_decl: &FunctionDeclaration
    ) -> Result<TypeId, SemanticError> {
        // Collecter les types des paramètres
        let mut param_type_ids = Vec::new();
        for param in &func_decl.parameters {
            let param_type_id = self.type_system.type_registry.convert_ast_type(&param.parameter_type);
            param_type_ids.push(param_type_id);
        }

        // Déterminer le type de retour
        let return_type_id = match &func_decl.return_type {
            Some(ast_type) => self.type_system.type_registry.convert_ast_type(ast_type),
            None => self.type_system.type_registry.type_unit, // () par défaut
        };

        // Créer le type de la fonction
        let function_type_id = self.type_system.type_registry.create_function_type(
            param_type_ids,
            return_type_id
        );

        Ok(function_type_id)
    }

    /// Vérifie un statement
    pub fn check_statement(&mut self, stmt: &Statement) -> Result<(), SemanticError> {
        match stmt {
            Statement::DeclarationStatement(Declaration::Variable(var_decl)) => {
                let var_type_id = self.check_variable_declaration(var_decl)?;

                // Marquer la variable comme ayant ce type dans la table des symboles
                // (ceci nécessiterait une intégration avec la table des symboles)

                Ok(())
            },

            Statement::DeclarationStatement(Declaration::Function(func_decl)) => {
                let func_type_id = self.check_function_declaration(func_decl)?;

                // Analyser le corps de la fonction
                for stmt in &func_decl.body {
                    if let ASTNode::Statement(statement) = stmt {
                        self.check_statement(statement)?;
                    }
                }

                Ok(())
            },

            Statement::Expression(expr) => {
                self.check_expression(expr)?;
                Ok(())
            },

            Statement::Assignment(target, value) => {
                self.check_assignment(&Box::new(target.clone()), &Box::new(value.clone()))?;
                Ok(())
            },

            // Plus de types de statements...
            _ => {
                // Pour l'instant, accepter tous les autres statements
                Ok(())
            }
        }
    }
}

// Fonction utilitaire pour créer une erreur sémantique
fn create_semantic_error(error_type: SemanticErrorType, message: String, position: Position) -> SemanticError {
    SemanticError::new(
        error_type,
        message,
        position
    )
}







//////////////////////YMC////////////////////////////////




// // use crate::parser::ast::{Expression, Type};
//
// // src/semantic/type_checker.rs
//
// use crate::parser::ast::{Expression, Statement, Operator, UnaryOperator};
// use crate::semantic::symbols::{SymbolId, SourceLocation};
//
// use crate::semantic::types::{Type, TypeId, TypeKind, TypeRegistry};
// use crate::semantic::types::type_system::{TypeSystem};
// use crate::semantic::semantic_error::{SemanticError, TypeError, SemanticErrorType, Position};
// use crate::semantic::symbol_table::SymbolTable;
// use crate::semantic::types::type_system::{TypeId, TypeKind};
//
// pub struct TypeChecker {
//     pub symbol_table: SymbolTable,
// }
//
// impl TypeChecker {
//     pub fn new(symbol_table: SymbolTable) -> Self {
//         TypeChecker { symbol_table }
//     }
//
//     /// Vérifie et infère le type d'une expression
//     pub fn check_expression(&mut self, expr: &Expression) -> Result<TypeId, SemanticError> {
//         match expr {
//             Expression::IntLiteral(value) => {
//                 // Les litéraux entiers ont toujours le type int
//                 Ok(self.symbol_table.type_registry.type_int)
//             },
//
//             Expression::FloatLiteral(value) => {
//                 // Les litéraux flottants ont toujours le type float
//                 Ok(self.symbol_table.type_registry.type_float)
//             },
//
//             Expression::BoolLiteral(value) => {
//                 // Les litéraux booléens ont toujours le type bool
//                 Ok(self.symbol_table.type_registry.type_bool)
//             },
//
//             Expression::StringLiteral(value) => {
//                 // Les litéraux string ont toujours le type string
//                 Ok(self.symbol_table.type_registry.type_string)
//             },
//
//             Expression::CharLiteral(value) => {
//                 // Les litéraux caractères ont toujours le type char
//                 Ok(self.symbol_table.type_registry.type_char)
//             },
//
//             Expression::Identifier(name) => {
//                 // Rechercher l'identifiant dans la table des symboles
//                 let symbol_id = self.symbol_table.lookup_symbol(name)?;
//
//                 // Récupérer le type associé au symbole
//                 if let Some(type_obj) = self.symbol_table.get_symbol_type(symbol_id)? {
//                     Ok(type_obj.id)
//                 } else {
//                     Err(create_semantic_error(
//                         SemanticErrorType::TypeError(TypeError::UndefinedType(name.clone())),
//                         "Variable used before its type is defined".to_string(),
//                         Position { index: 0 }
//                     ))
//                 }
//             },
//
//             Expression::Binary { left, operator, right } => {
//                 self.check_binary_expression(left, operator, right)
//             },
//
//             Expression::Unary { operator, operand } => {
//                 self.check_unary_expression(operator, operand)
//             },
//
//             Expression::Call { function, arguments } => {
//                 self.check_function_call(function, arguments)
//             },
//
//             Expression::ArrayLiteral(elements) => {
//                 self.check_array_literal(elements)
//             },
//
//             Expression::ArrayAccess { array, index } => {
//                 self.check_array_access(array, index)
//             },
//
//             Expression::MemberAccess { object, member } => {
//                 self.check_member_access(object, member)
//             },
//
//             // Plus de cas selon votre AST...
//
//             _ => {
//                 // Cas par défaut pour les expressions non gérées
//                 Err(create_semantic_error(
//                     SemanticErrorType::TypeError(TypeError::InvalidType("Unsupported expression type".to_string())),
//                     "Expression type not supported yet".to_string(),
//                     Position { index: 0 }
//                 ))
//             }
//         }
//     }
//
//     /// Vérifie les types d'une expression binaire
//     fn check_binary_expression(
//         &mut self,
//         left: &Box<Expression>,
//         operator: &Operator,
//         right: &Box<Expression>
//     ) -> Result<TypeId, SemanticError> {
//         let left_type_id = self.check_expression(left)?;
//         let right_type_id = self.check_expression(right)?;
//
//         // Récupérer les objets Type
//         let left_type = self.symbol_table.type_registry.get_type(left_type_id)
//             .ok_or_else(|| create_semantic_error(
//                 SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", left_type_id))),
//                 "Type not found".to_string(),
//                 Position { index: 0 }
//             ))?;
//
//         let right_type = self.symbol_table.type_registry.get_type(right_type_id)
//             .ok_or_else(|| create_semantic_error(
//                 SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", right_type_id))),
//                 "Type not found".to_string(),
//                 Position { index: 0 }
//             ))?;
//
//         // Vérifier la compatibilité des opérandes selon l'opérateur
//         match operator {
//             Operator::Addition | Operator::Substraction |
//             Operator::Multiplication | Operator::Division |
//             Operator::Modulo => {
//                 // Opérations arithmétiques
//                 match (&left_type.kind, &right_type.kind) {
//                     (TypeKind::Int, TypeKind::Int) => Ok(left_type_id), // int op int -> int
//                     (TypeKind::Float, TypeKind::Float) => Ok(left_type_id), // float op float -> float
//                     (TypeKind::Int, TypeKind::Float) => Ok(right_type_id), // int op float -> float
//                     (TypeKind::Float, TypeKind::Int) => Ok(left_type_id), // float op int -> float
//                     (TypeKind::String, TypeKind::String) if *operator == BinaryOperator::Add => {
//                         // Concaténation de chaînes
//                         Ok(left_type_id) // string + string -> string
//                     },
//                     _ => Err(create_semantic_error(
//                         SemanticErrorType::TypeError(TypeError::TypeMismatch(
//                             format!("Cannot apply operator {:?} to types {} and {}",
//                                     operator, left_type, right_type)
//                         )),
//                         "Incompatible types for binary operation".to_string(),
//                         Position { index: 0 }
//                     ))
//                 }
//             },
//
//             Operator::Equal | Operator::NotEqual => {
//                 // Opérations d'égalité (==, !=) peuvent être appliquées à tous les types comparables
//                 if left_type.is_compatible_with(right_type) || right_type.is_compatible_with(left_type) {
//                     Ok(self.symbol_table.type_registry.type_bool) // Résultat est toujours bool
//                 } else {
//                     Err(create_semantic_error(
//                         SemanticErrorType::TypeError(TypeError::TypeMismatch(
//                             format!("Cannot compare types {} and {}", left_type, right_type)
//                         )),
//                         "Incompatible types for comparison".to_string(),
//                         Position { index: 0 }
//                     ))
//                 }
//             },
//
//             Operator::LessThan | Operator::LesshanOrEqual |
//             Operator::GreaterThan | Operator::GreaterThanOrEqual => {
//                 // Opérations de comparaison (<, <=, >, >=)
//                 match (&left_type.kind, &right_type.kind) {
//                     (TypeKind::Int, TypeKind::Int) |
//                     (TypeKind::Float, TypeKind::Float) |
//                     (TypeKind::Int, TypeKind::Float) |
//                     (TypeKind::Float, TypeKind::Int) |
//                     (TypeKind::Char, TypeKind::Char) => {
//                         Ok(self.symbol_table.type_registry.type_bool) // Résultat est toujours bool
//                     },
//                     _ => Err(create_semantic_error(
//                         SemanticErrorType::TypeError(TypeError::TypeMismatch(
//                             format!("Cannot compare types {} and {} with operator {:?}",
//                                     left_type, right_type, operator)
//                         )),
//                         "Incompatible types for comparison".to_string(),
//                         Position { index: 0 }
//                     ))
//                 }
//             },
//
//             Operator::And | Operator::Or => {
//                 // Opérations logiques (&&, ||)
//                 if left_type.kind == TypeKind::Bool && right_type.kind == TypeKind::Bool {
//                     Ok(self.symbol_table.type_registry.type_bool) // bool op bool -> bool
//                 } else {
//                     Err(create_semantic_error(
//                         SemanticErrorType::TypeError(TypeError::TypeMismatch(
//                             format!("Logical operators require boolean operands, got {} and {}",
//                                     left_type, right_type)
//                         )),
//                         "Logical operation requires boolean operands".to_string(),
//                         Position { index: 0 }
//                     ))
//                 }
//             },
//
//             // Plus d'opérateurs selon votre langage...
//         }
//     }
//
//     // Implémentez les autres méthodes check_* de façon similaire
//
//     /// Vérifie une déclaration de variable
//     pub fn check_variable_declaration(
//         &mut self,
//         name: &str,
//         initializer: Option<&Expression>,
//         explicit_type: Option<TypeId>
//     ) -> Result<TypeId, SemanticError> {
//         // Récupérer le symbole
//         let symbol_id = self.symbol_table.lookup_symbol(name)?;
//
//         let inferred_type = match (explicit_type, initializer) {
//             (Some(type_id), Some(expr)) => {
//                 // A la fois un type explicite et un initializer
//                 let expr_type_id = self.check_expression(expr)?;
//
//                 // Vérifier la compatibilité des types
//                 let expr_type = self.symbol_table.type_registry.get_type(expr_type_id)
//                     .ok_or_else(|| create_semantic_error(
//                         SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", expr_type_id))),
//                         "Type not found".to_string(),
//                         Position { index: 0 }
//                     ))?;
//
//                 let declared_type = self.symbol_table.type_registry.get_type(type_id)
//                     .ok_or_else(|| create_semantic_error(
//                         SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", type_id))),
//                         "Type not found".to_string(),
//                         Position { index: 0 }
//                     ))?;
//
//                 if !expr_type.is_compatible_with(declared_type) {
//                     return Err(create_semantic_error(
//                         SemanticErrorType::TypeError(TypeError::TypeMismatch(
//                             format!("Cannot assign type {} to variable of type {}",
//                                     expr_type, declared_type)
//                         )),
//                         "Type mismatch in variable initialization".to_string(),
//                         Position { index: 0 }
//                     ));
//                 }
//
//                 type_id
//             },
//
//             (Some(type_id), None) => {
//                 // Type explicite sans initializer
//                 type_id
//             },
//
//             (None, Some(expr)) => {
//                 // Initializer sans type explicite (inférence)
//                 self.check_expression(expr)?
//             },
//
//             (None, None) => {
//                 // Ni type explicite ni initializer - erreur ou type par défaut?
//                 return Err(create_semantic_error(
//                     SemanticErrorType::TypeError(TypeError::UndefinedType(name.to_string())),
//                     "Variable declaration without type or initializer".to_string(),
//                     Position { index: 0 }
//                 ));
//             }
//         };
//
//         // Mettre à jour le type du symbole
//         self.symbol_table.set_symbol_type(symbol_id, inferred_type)?;
//
//         Ok(inferred_type)
//     }
// }
//
// // Fonction utilitaire pour créer une erreur sémantique
// fn create_semantic_error(error_type: SemanticErrorType, message: String, position: Position) -> SemanticError {
//     SemanticError::new(
//         error_type,
//         message,
//         position
//     )
// }
//
//
//
//
//
// //
// // pub struct TypeChecker {
// //     //Environnement de typage
// //     type_env: TypeEnv,
// //     // context de trait
// //     trait_context: TraitContext,
// //
// //
// //     // // Le contexte de typage
// //     // type_context: TypeContext,
// //     // // Le contexte de portée
// //     // scope_context: ScopeContext,
// //     // // Le contexte des emprunts
// //     // borrow_checker: BorrowChecker,
// // }
// //
// //
// //
// //
// //
// // impl TypeChecker {
// //     fn check_assignment(&mut self, target: &Expression, value: &Expression) -> Result<Type, TypeError>;
// //     fn check_method_call(&mut self, object: &Expr, method: &str) -> Result<Type, TypeError>;
// // }
