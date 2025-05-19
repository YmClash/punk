// use std::any::TypeId;
// use std::collections::HashMap;
// use crate::parser::ast::{Assignment, BinaryOperation, Expression, Literal, Operator, UnaryOperation, UnaryOperator};
//
// use crate::semantic::semantic_error::SemanticErrorType::TypeError;
//
//
// #[derive(Debug, Clone, PartialEq)]
// pub enum Type {
//     // Types primitifs
//     Int,
//     Float,
//     Bool,
//     Char,
//     String,
//
//     // Conteneurs et structures
//     Array(Box<Type>, Option<usize>),  // Type, longueur optionnelle
//     Tuple(Vec<Type>),
//     Struct(StructTypeId),
//     Enum(EnumTypeId),
//
//     // Types pour les fonctions
//     Function(FunctionType),
//
//     // Types pour les références
//     Reference(Box<Type>, Mutability, Option<LifetimeId>),
//
//     // Type générique
//     Generic(String, Vec<TypeConstraint>),
//
//     // Type polymorphique (ex: auto, var, _)
//     Infer(TypeVarId),
//
//     // Type utilisateur
//     Named(String, Vec<Type>),  // Nom du type, arguments génériques
//
//     // Type d'une méthode associée à un self
//     SelfType,
//
//     // Type "unit" pour les fonctions sans retour
//     Unit,
//
//     // Type pour les valeurs impossibles (jamais atteintes)
//     Never,
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub enum Mutability {
//     Mutable,
//     Immutable,
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct FunctionType {
//     pub params: Vec<Type>,
//     pub return_type: Box<Type>,
//     pub lifetime_params: Vec<LifetimeId>,
//     pub type_params: Vec<TypeVarId>,
// }
//
// #[allow(dead_code)]
// #[derive(Debug, Clone, PartialEq)]
// pub enum TypeConstraint {
//     Equal(Type, Type),
//     Subtype(Type, Type),
//     Instance(Type, Vec<Type>),
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct StructTypeId {
//     pub id: TypeId,
//     pub name: String,
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct EnumTypeId {
//     pub id: TypeId,
//     pub name: String,
// }
//
// #[derive(Debug, Clone, PartialEq,Eq, Hash)]
// pub struct TypeVarId {
//     pub id: TypeId,
//     pub name: String,
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct LifetimeId {
//     pub id: TypeId,
//     pub name: String,
// }
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct TraitId {
//     pub id: TypeId,
//     pub name: String,
// }
//
//
// #[derive(Debug, Clone, PartialEq)]
// pub struct TypeDefinition {
//     pub id: TypeId,
//     pub name: String,
//     pub type_params: Vec<TypeVarId>,
//     pub lifetime_params: Vec<LifetimeId>,
//     pub fields: Vec<(String, Type)>,  // Nom du champ et son type
// }
//
//
// pub struct TypeSystem {
//     // Table des types nommés (structs, enums, etc.)
//     named_types: HashMap<String, TypeId>,
//
//     // Table des définitions de types
//     type_definitions: HashMap<TypeId, TypeDefinition>,
//
//     // Mappages pour l'inférence de types
//     type_variables: HashMap<TypeVarId, Option<Type>>,
//
//     // Contraintes de types
//     type_constraints: Vec<TypeConstraint>,
//
//     // Compteurs pour générer des identifiants uniques
//     next_type_id: u32,
//     next_type_var_id: u32,
//     next_lifetime_id: u32,
// }
//
//
// impl TypeSystem {
//     // Unifie deux types, en résolvant les variables de type
//     pub fn unify(&mut self, t1: &Type, t2: &Type) -> Result<Type, TypeError> {
//         match (t1, t2) {
//             // Cas de base: types identiques
//             (a, b) if a == b => Ok(a.clone()),
//
//             // Cas avec variable de type
//             (Type::Infer(id), other) | (other, Type::Infer(id)) => {
//                 self.unify_var(*id.clone(), other.clone())
//             },
//
//             // Cas avec types structurellement similaires
//             (Type::Array(elem1, size1), Type::Array(elem2, size2)) => {
//                 let unified_elem = self.unify(elem1, elem2)?;
//                 if size1 != size2 {
//                     return Err(TypeError::SizeMismatch(*size1, *size2));
//                 }
//                 Ok(Type::Array(Box::new(unified_elem), *size1))
//             },
//
//             // Autres cas similaires pour Tuple, Function, etc.
//
//             // Cas par défaut: types incompatibles
//             _ => Err(TypeError::TypeMismatch(t1.clone(), t2.clone())),
//         }
//     }
//
//     // Unifie une variable de type avec un type
//     fn unify_var(&mut self, id: TypeVarId, typ: Type) -> Result<Type, TypeError> {
//         // Vérifier si la variable a déjà une valeur
//         if let Some(Some(existing)) = self.type_variables.get(&id) {
//             return self.unify(existing, &typ);
//         }
//
//         // Vérifier l'occurrence de la variable dans le type (pour éviter les types récursifs)
//         if self.occurs_check(id.clone(), &typ) {
//             return Err(TypeError::RecursiveType);
//         }
//
//         // Assigner le type à la variable
//         self.type_variables.insert(id, Some(typ.clone()));
//         Ok(typ)
//     }
//
//     // Vérifie si une variable de type apparaît dans un type (occurs check)
//     fn occurs_check(&self, id: TypeVarId, typ: &Type) -> bool {
//         match typ {
//             Type::Infer(var_id) => *var_id == id,
//             Type::Array(elem, _) => self.occurs_check(id, elem),
//             Type::Tuple(types) => types.iter().any(|t| self.occurs_check(id, t)),
//             // Autres cas...
//             _ => false,
//         }
//     }
// }




//
//
//
// impl TypeChecker {
//     pub fn infer_expression(&mut self, expr: &Expression) -> Result<Type, String> {
//         match expr {
//             Expression::Literal(lit) => self.infer_literal(lit),
//             Expression::Identifier(name) => self.lookup_type(name),
//             Expression::BinaryOperation(binop) => self.infer_binary_op(binop),
//             Expression::Assignment(assign) => self.infer_assignment(assign),
//             Expression::UnaryOperation(unop) => self.infer_unary_op(unop),
//             _ => Ok(Type::Infer), // Pour les autres cas
//         }
//     }
//
//     fn infer_literal(&self, lit: &Literal) -> Result<crate::parser::ast::Type, String> {
//         match lit {
//             Literal::Integer { .. } => Ok(crate::parser::ast::Type::Int),
//             Literal::Float { .. } => Ok(crate::parser::ast::Type::Float),
//             // Literal::String(_) => Ok(Type::String),
//             Literal::String(s) => {
//                 // Si c'est un seul caractère entre guillemets simples
//                 if s.len() == 1 && s.starts_with('\'') && s.ends_with('\'') {
//                     Ok(crate::parser::ast::Type::Char)
//                 } else {
//                     Ok(crate::parser::ast::Type::String)
//                 }
//             }
//             Literal::Boolean(_) => Ok(crate::parser::ast::Type::Bool),
//             Literal::Char(_) => Ok(crate::parser::ast::Type::Char), // Ajout  l'inference de type pour les caractères
//             _ => Ok(crate::parser::ast::Type::Infer),
//         }
//     }
//
//     fn infer_binary_op(&mut self, binop: &BinaryOperation) -> Result<crate::parser::ast::Type, String> {
//         let left_type = self.infer_expression(&binop.left)?;
//         let right_type = self.infer_expression(&binop.right)?;
//
//         match binop.operator {
//             Operator::Addition | Operator::Substraction |
//             Operator::Multiplication | Operator::Division => {
//                 if left_type == crate::parser::ast::Type::Int && right_type == crate::parser::ast::Type::Int {
//                     Ok(crate::parser::ast::Type::Int)
//                 } else if left_type == crate::parser::ast::Type::Float || right_type == crate::parser::ast::Type::Float {
//                     Ok(crate::parser::ast::Type::Float)
//                 } else if left_type == crate::parser::ast::Type::Infer || right_type == crate::parser::ast::Type::Infer {
//                     Ok(crate::parser::ast::Type::Infer)
//                 } else {
//                     Err("Type mismatch in binary operation".to_string())
//                 }
//             },
//             Operator::Equal | Operator::NotEqual |
//             Operator::LessThan | Operator::GreaterThan |
//             Operator::LesshanOrEqual | Operator::GreaterThanOrEqual => {
//                 self.add_constraint(TypeConstraint::Equal(left_type, right_type));
//                 Ok(crate::parser::ast::Type::Bool)
//             },
//             _ => Ok(crate::parser::ast::Type::Infer),
//         }
//     }
//
//     fn infer_assignment(&mut self, assign: &Assignment) -> Result<crate::parser::ast::Type, String> {
//         let value_type = self.infer_expression(&assign.value)?;
//
//         match &*assign.target {
//             Expression::Identifier(name) => {
//                 self.type_vars.insert(name.clone(), value_type.clone());
//                 Ok(value_type)
//             },
//             _ => Err("Invalid assignment target".to_string())
//         }
//     }
//
//
//     fn infer_unary_op(&mut self, unop: &UnaryOperation) -> Result<crate::parser::ast::Type, String> {
//         let operand_type = self.infer_expression(&unop.operand)?;
//
//         match unop.operator {
//             UnaryOperator::Negative => {
//                 match operand_type {
//                     crate::parser::ast::Type::Int => Ok(crate::parser::ast::Type::Int),
//                     crate::parser::ast::Type::Float => Ok(crate::parser::ast::Type::Float),
//                     crate::parser::ast::Type::Infer => Ok(crate::parser::ast::Type::Infer),
//                     _ => Err("Operator '-' cannot be applied to this type".to_string())
//                 }
//             },
//             UnaryOperator::Not => {
//                 match operand_type {
//                     crate::parser::ast::Type::Bool => Ok(crate::parser::ast::Type::Bool),
//                     crate::parser::ast::Type::Infer => Ok(crate::parser::ast::Type::Infer),
//                     _ => Err("Operator '!' can only be applied to boolean types".to_string())
//                 }
//             },
//             UnaryOperator::Reference => Ok(crate::parser::ast::Type::Array(Box::new(operand_type))),
//             UnaryOperator::ReferenceMutable => Ok(crate::parser::ast::Type::Array(Box::new(operand_type))),
//             _ => todo!(),
//         }
//     }
//
//
//
//     pub fn check_assignment(&mut self, target: &Expression, value: &Expression) -> Result<(), TypeError> {
//         // 1. Vérifier que la cible est assignable (lvalue)
//         if !self.is_lvalue(target) {
//             return Err(TypeError::NotAssignable);
//         }
//
//         // 2. Vérifier que la cible est mutable
//         self.check_mutability(target)?;
//
//         // 3. Inférer les types
//         let target_type = self.infer_expression(target)?;
//         let value_type = self.infer_expression(value)?;
//
//         // 4. Vérifier la compatibilité des types
//         self.type_system.unify(&target_type, &value_type)
//             .map(|_| ())
//             .map_err(|_| TypeError::AssignmentMismatch(target_type, value_type))
//     }
//
//     fn is_lvalue(&self, expr: &Expression) -> bool {
//         match expr {
//             Expression::Identifier(_) => true,
//             Expression::Literal(_) => false,
//             Expression::BinaryOperation(_) => false,
//             Expression::UnaryOperation(_) => false,
//             Expression::Assignment(_) => false,
//             Expression::MethodCall(_) => false,
//
//             // Expression::Variable(_) => true,
//             // Expr::FieldAccess(_, _) => true,
//             // Expr::Index(_, _) => true,
//             // D'autres expressions qui peuvent être à gauche d'une affectation
//             _ => false,
//         }
//     }
//
//     fn check_mutability(&self, expr: &Expression) -> Result<(), TypeError> {
//         match expr {
//             Expression::Variable(name) => {
//                 let symbol_id = self.symbol_table.lookup_symbol(name)?;
//                 if !self.symbol_table.is_mutable(symbol_id)? {
//                     return Err(TypeError::ImmutableAssignment(name.clone()));
//                 }
//                 Ok(())
//             },
//             // Autres cas...
//             _ => Ok(()),
//         }
//     }
// }
//
