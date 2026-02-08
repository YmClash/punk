//src/semantic/type_checker.rs

use std::cell::RefCell;
use std::rc::Rc;
use crate::parser::ast::{Expression, Statement, Operator, UnaryOperator, Literal,
                         VariableDeclaration, FunctionDeclaration, ASTNode, Declaration};

use crate::semantic::types::type_system::{Type, TypeId, TypeKind, TypeSystem, Mutability};
use crate::semantic::semantic_error::{SemanticError, TypeError, SemanticErrorType, Position};
use crate::semantic::symbol_table::SymbolTable;
use crate::semantic::context::CompilationContext;

pub struct TypeChecker {
    context: Rc<RefCell<CompilationContext>>,
    // Keep direct references for performance in hot paths
    symbol_table_ref: Rc<RefCell<SymbolTable>>,
    type_system_ref: Rc<RefCell<TypeSystem>>,
}

impl TypeChecker {
    pub fn new(context: Rc<RefCell<CompilationContext>>) -> Self {
        let symbol_table_ref = context.borrow().symbols.clone();
        let type_system_ref = context.borrow().types.clone();
        
        TypeChecker {
            context,
            symbol_table_ref,
            type_system_ref,
        }
    }
    
    /// Crée un TypeChecker à partir de composants existants (pour compatibilité)
    pub fn from_components(symbol_table: SymbolTable, type_system: TypeSystem) -> Self {
        let context = Rc::new(RefCell::new(
            CompilationContext::from_existing(type_system, symbol_table)
        ));
        Self::new(context)
    }
    
    /// Méthode helper pour obtenir un type de manière safe
    fn get_type(&self, type_id: TypeId) -> Result<Type, SemanticError> {
        let type_system = self.type_system_ref.borrow();
        type_system.get_type(type_id)
            .ok_or_else(|| create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", type_id))),
                "Type not found".to_string(),
                Position { index: 0 }
            ))
            .map(|t| t.clone())
    }

    /// Vérifie et infère le type d'une expression
    pub fn check_expression(&mut self, expr: &Expression) -> Result<TypeId, SemanticError> {
        match expr {
            Expression::Literal(literal) => {
                self.check_literal(literal)
            },

            Expression::Identifier(name) => {
                // Rechercher l'identifiant dans la table des symboles
                let symbol_id = self.symbol_table_ref.borrow().lookup_symbol(name)?;

                // Récupérer le type associé au symbole
                if let Some(type_obj) = self.symbol_table_ref.borrow().get_symbol_type(symbol_id)? {
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
            Literal::Integer { .. } => Ok(self.type_system_ref.borrow().type_int),
            Literal::Float { .. } => Ok(self.type_system_ref.borrow().type_float),
            Literal::Boolean(_) => Ok(self.type_system_ref.borrow().type_bool),
            Literal::String(_) => Ok(self.type_system_ref.borrow().type_string),
            Literal::Char(_) => Ok(self.type_system_ref.borrow().type_char),
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
        let type_system = self.type_system_ref.borrow();
        let left_type = type_system.get_type(left_type_id)
            .ok_or_else(|| create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", left_type_id))),
                "Type not found".to_string(),
                Position { index: 0 }
            ))?.clone();

        let right_type = type_system.get_type(right_type_id)
            .ok_or_else(|| create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", right_type_id))),
                "Type not found".to_string(),
                Position { index: 0 }
            ))?.clone();
        drop(type_system); // Libérer l'emprunt avant d'utiliser les types

        // Vérifier la compatibilité des opérandes selon l'opérateur
        match operator {
            Operator::Addition | Operator::Subtraction |
            Operator::Multiplication | Operator::Division |
            Operator::Modulo => {
                // Opérations arithmétiques
                match (&left_type.kind, &right_type.kind) {
                    // (TypeKind::Infer, TypeKind::Infer) => Ok(right_type_id),
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
                if left_type.is_compatible_with(&right_type) || right_type.is_compatible_with(&left_type) {
                    Ok(self.type_system_ref.borrow().type_bool) // Résultat est toujours bool
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

            Operator::LessThan | Operator::LessThanOrEqual |
            Operator::GreaterThan | Operator::GreaterThanOrEqual => {
                // Opérations de comparaison (<, <=, >, >=)
                match (&left_type.kind, &right_type.kind) {
                    (TypeKind::Int, TypeKind::Int) |
                    (TypeKind::Float, TypeKind::Float) |
                    (TypeKind::Int, TypeKind::Float) |
                    (TypeKind::Float, TypeKind::Int) |
                    (TypeKind::Char, TypeKind::Char) => {
                        Ok(self.type_system_ref.borrow().type_bool) // Résultat est toujours bool
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
                    Ok(self.type_system_ref.borrow().type_bool) // bool op bool -> bool
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
                if right_type.is_compatible_with(&left_type) {
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
                if left_type.is_compatible_with(&right_type) || right_type.is_compatible_with(&left_type) {
                    // Créer un type range (pour l'instant, utilisons un type nommé)
                    let range_type_id = self.type_system_ref.borrow_mut().register_type(
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
        let type_system = self.type_system_ref.borrow();
        let operand_type = type_system.get_type(operand_type_id)
            .ok_or_else(|| create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", operand_type_id))),
                "Type not found".to_string(),
                Position { index: 0 }
            ))?.clone();
        drop(type_system);

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
                    TypeKind::Bool => Ok(self.type_system_ref.borrow().type_bool),
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
                let ref_type_id = self.type_system_ref.borrow_mut().create_reference_type(
                    operand_type_id,
                    Mutability::Immutable,
                    None
                );
                Ok(ref_type_id)
            },

            UnaryOperator::ReferenceMutable => {
                // Créer une référence mutable
                let ref_type_id = self.type_system_ref.borrow_mut().create_reference_type(
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
            let type_system = self.type_system_ref.borrow();
            let function_type = type_system.get_type(function_type_id)
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
        if !func_type_clone.is_variadic && arguments.len() != func_type_clone.params.len() {
            return Err(create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeMismatch(
                    format!("Function expects {} arguments, got {}",
                            func_type_clone.params.len(), arguments.len())
                )),
                "Incorrect number of arguments".to_string(),
                Position { index: 0 }
            ));
        }

        // si la fonction est variadique, vérifier qu'il y a au moins le nombre minimum d'arguments
        if func_type_clone.is_variadic && arguments.len() < func_type_clone.params.len(){
            return Err(create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeMismatch(
                    format!("Variadic function expects at least {} arguments, got {}",
                            func_type_clone.params.len(), arguments.len())
                )),
                "Insufficient number of arguments for variadic function".to_string(),
                Position { index: 0 }
            ))
        }

        // Vérifier les types des arguments
        for (i, (arg, param_type)) in arguments.iter().zip(func_type_clone.params.iter()).enumerate() {
            let arg_type_id = self.check_expression(arg)?;
            let type_system = self.type_system_ref.borrow();
            let arg_type = type_system.get_type(arg_type_id)
                .ok_or_else(|| create_semantic_error(
                    SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", arg_type_id))),
                    "Type not found".to_string(),
                    Position { index: 0 }
                ))?.clone();
            drop(type_system);

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

        // Pour les arguments variadiques, nous nous contentons de vérifier qu'ils sont valides
        // sans imposer de contrainte de type stricte pour l'instant

        if func_type_clone.is_variadic {
            for arg in arguments.iter().skip(func_type_clone.params.len()) {
                self.check_expression(arg)?; // S'assure que l'expression est valide
            }
        }

        // Retourner le type de retour de la fonction
        Ok(func_type_clone.return_type.id)
    }

    /// Vérifie un littéral array
    fn check_array_literal(&mut self, elements: &Vec<Expression>) -> Result<TypeId, SemanticError> {
        if elements.is_empty() {
            // Array vide - créer un array de type inféré
            let infer_type_var = self.type_system_ref.borrow_mut().create_type_variable(Some("ArrayElement".to_string()));
            let infer_type_id = self.type_system_ref.borrow_mut().register_type(TypeKind::Infer(infer_type_var));
            return Ok(self.type_system_ref.borrow_mut().create_array_type(infer_type_id, Some(0)));
        }

        // Vérifier le type du premier élément
        let first_element_type_id = self.check_expression(&elements[0])?;
        let type_system = self.type_system_ref.borrow();
        let first_element_type = type_system.get_type(first_element_type_id)
            .ok_or_else(|| create_semantic_error(
                SemanticErrorType::TypeError(TypeError::TypeNotFound(format!("{:?}", first_element_type_id))),
                "Type not found".to_string(),
                Position { index: 0 }
            ))?.clone();
        drop(type_system);

        // Vérifier que tous les éléments ont le même type
        for (i, element) in elements.iter().skip(1).enumerate() {
            let element_type_id = self.check_expression(element)?;
            let element_type = self.get_type(element_type_id)?;

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
        Ok(self.type_system_ref.borrow_mut().create_array_type(first_element_type_id, Some(elements.len())))
    }

    /// Vérifie un accès à un array
    fn check_array_access(
        &mut self,
        array: &Box<Expression>,
        index: &Box<Expression>
    ) -> Result<TypeId, SemanticError> {
        let array_type_id = self.check_expression(array)?;
        let index_type_id = self.check_expression(index)?;

        let array_type = self.get_type(array_type_id)?;

        let index_type = self.get_type(index_type_id)?;

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
        let object_type = self.get_type(object_type_id)?;

        // Pour l'instant, une implémentation simplifiée
        // Dans une vraie implémentation, il faudrait chercher le membre dans la structure
        match &object_type.kind {
            TypeKind::Struct(_) | TypeKind::Named(_, _) => {
                // Supposer que le membre existe et retourner un type inféré
                let member_type_var = self.type_system_ref.borrow_mut().create_type_variable(Some(format!("{}Member", member)));
                Ok(self.type_system_ref.borrow_mut().register_type(TypeKind::Infer(member_type_var)))
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

        let target_type = self.get_type(target_type_id)?;

        let value_type = self.get_type(value_type_id)?;

        // Vérifier la compatibilité des types
        if !value_type.is_compatible_with(&target_type) {
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
        let method_return_var = self.type_system_ref.borrow_mut().create_type_variable(Some(format!("{}Return", method)));
        Ok(self.type_system_ref.borrow_mut().register_type(TypeKind::Infer(method_return_var)))
    }

    /// Vérifie un cast de type
    fn check_type_cast(
        &mut self,
        expression: &Box<Expression>,
        target_type: &crate::parser::ast::Type
    ) -> Result<TypeId, SemanticError> {
        let expr_type_id = self.check_expression(expression)?;
        let target_type_id = self.type_system_ref.borrow_mut().convert_ast_type(target_type);

        let expr_type = self.get_type(expr_type_id)?;

        let target_type_obj = self.get_type(target_type_id)?;

        // Vérifier que le cast est valide
        match (&expr_type.kind, &target_type_obj.kind) {
            // Casts numériques autorisés
            (TypeKind::Int, TypeKind::Float) |
            (TypeKind::Float, TypeKind::Int) |
            (TypeKind::Int, TypeKind::Char) |
            (TypeKind::Char, TypeKind::Int) => Ok(target_type_id),

            // Cast vers le même type
            (a, b) if *a == *b => Ok(target_type_id),

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
                let declared_type_id = self.type_system_ref.borrow_mut().convert_ast_type(ast_type);
                let expr_type_id = self.check_expression(expr)?;

                // Vérifier la compatibilité des types
                let expr_type = self.get_type(expr_type_id)?;

                let declared_type = self.get_type(declared_type_id)?;

                if !expr_type.is_compatible_with(&declared_type) {
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
                self.type_system_ref.borrow_mut().convert_ast_type(ast_type)
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
            let param_type_id = self.type_system_ref.borrow_mut().convert_ast_type(&param.parameter_type);
            param_type_ids.push(param_type_id);
        }

        // Déterminer le type de retour
        let return_type_id = match &func_decl.return_type {
            Some(ast_type) => self.type_system_ref.borrow_mut().convert_ast_type(ast_type),
            None => self.type_system_ref.borrow().type_unit, // () par défaut
        };

        let is_variadic = func_decl.is_variadic;

        // Créer le type de la fonction
        let function_type_id = self.type_system_ref.borrow_mut().create_function_type(
            param_type_ids,
            return_type_id,
            is_variadic
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


