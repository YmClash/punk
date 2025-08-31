//src/semantic/types/inference_impl.rs

use super::inference::*;
use crate::parser::ast::{Expression, Statement, Declaration, ASTNode, Literal, Operator};
use crate::semantic::semantic_error::{SemanticError, SemanticErrorType, TypeError, Position};
use crate::semantic::types::type_system::TypeId;

impl TypeInferenceEngine {
    /// Infère le type d'une expression et collecte les contraintes
    pub fn infer_expression(&mut self, expr: &Expression) -> Result<TypeId, SemanticError> {
        match expr {
            Expression::Literal(lit) => {
                let ctx = self.context.borrow();
                let type_system = ctx.types.borrow();
                
                Ok(match lit {
                    Literal::Integer { .. } => type_system.type_int,
                    Literal::Float { .. } => type_system.type_float,
                    Literal::String(_) => type_system.type_string,
                    Literal::Boolean(_) => type_system.type_bool,
                    Literal::Char(_) => type_system.type_char,
                    _ => return Err(create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::InvalidType(
                            "Unsupported literal type".to_string()
                        )),
                        "Cannot infer type for this literal".to_string(),
                        Position { index: 0 }
                    ))
                })
            }
            
            Expression::Identifier(name) => {
                // Chercher le type dans la table des symboles
                let ctx = self.context.borrow();
                let symbols = ctx.symbols.borrow();
                
                if let Ok(symbol_id) = symbols.lookup_symbol(name) {
                    if let Ok(symbol) = symbols.get_symbol(symbol_id) {
                        if let Some(type_id) = symbol.attributes.type_info {
                            Ok(type_id)
                        } else {
                            // Si pas de type connu, créer une variable de type
                            drop(symbols);
                            drop(ctx);
                            Ok(self.fresh_type_var(Some(name.clone())))
                        }
                    } else {
                        // Si pas de type connu, créer une variable de type
                        drop(symbols);
                        drop(ctx);
                        Ok(self.fresh_type_var(Some(name.clone())))
                    }
                } else {
                    Err(create_semantic_error(
                        SemanticErrorType::TypeError(TypeError::UndefinedVariable(name.clone())),
                        format!("Undefined variable: {}", name),
                        Position { index: 0 }
                    ))
                }
            }
            
            Expression::BinaryOperation(binop) => {
                let left_type = self.infer_expression(&binop.left)?;
                let right_type = self.infer_expression(&binop.right)?;
                
                match binop.operator {
                    Operator::Addition | Operator::Substraction | 
                    Operator::Multiplication | Operator::Division | Operator::Modulo => {
                        // Les opérandes doivent être numériques et du même type
                        self.constraints.push(TypeConstraint::Numeric(left_type));
                        self.constraints.push(TypeConstraint::Numeric(right_type));
                        self.constraints.push(TypeConstraint::Equal(left_type, right_type));
                        Ok(left_type)
                    }
                    
                    Operator::EqualEqual | Operator::NotEqual => {
                        // Les opérandes doivent être du même type
                        self.constraints.push(TypeConstraint::Equal(left_type, right_type));
                        let ctx = self.context.borrow();
                        let type_system = ctx.types.borrow();
                        Ok(type_system.type_bool)
                    }
                    
                    Operator::LessThan | Operator::GreaterThan | 
                    Operator::LesshanOrEqual | Operator::GreaterThanOrEqual => {
                        // Les opérandes doivent être comparables et du même type
                        self.constraints.push(TypeConstraint::Comparable(left_type));
                        self.constraints.push(TypeConstraint::Comparable(right_type));
                        self.constraints.push(TypeConstraint::Equal(left_type, right_type));
                        let ctx = self.context.borrow();
                        let type_system = ctx.types.borrow();
                        Ok(type_system.type_bool)
                    }
                    
                    Operator::And | Operator::Or => {
                        // Les opérandes doivent être booléens
                        let ctx = self.context.borrow();
                        let type_system = ctx.types.borrow();
                        let bool_type = type_system.type_bool;
                        self.constraints.push(TypeConstraint::Equal(left_type, bool_type));
                        self.constraints.push(TypeConstraint::Equal(right_type, bool_type));
                        Ok(bool_type)
                    }
                    
                    _ => {
                        // Pour les autres opérateurs, créer une variable de type
                        Ok(self.fresh_type_var(None))
                    }
                }
            }
            
            Expression::FunctionCall(call) => {
                // Inférer le type de la fonction
                let func_type = self.infer_expression(&call.name)?;
                
                // Inférer les types des arguments
                let mut arg_types = Vec::new();
                for arg in &call.arguments {
                    arg_types.push(self.infer_expression(arg)?);
                }
                
                // Créer une variable de type pour le retour
                let return_type = self.fresh_type_var(Some("ReturnType".to_string()));
                
                // Ajouter une contrainte Callable
                self.constraints.push(TypeConstraint::Callable(func_type, arg_types, return_type));
                
                Ok(return_type)
            }
            
            Expression::MemberAccess(member) => {
                let object_type = self.infer_expression(&member.object)?;
                let field_type = self.fresh_type_var(Some(format!("{}_type", member.member)));
                
                // Ajouter une contrainte HasField
                self.constraints.push(TypeConstraint::HasField(
                    object_type,
                    member.member.clone(),
                    field_type
                ));
                
                Ok(field_type)
            }
            
            Expression::Assignment(assign) => {
                let target_type = self.infer_expression(&assign.target)?;
                let value_type = self.infer_expression(&assign.value)?;
                
                // L'affectation impose que les types soient égaux
                self.constraints.push(TypeConstraint::Equal(target_type, value_type));
                
                Ok(target_type)
            }
            
            Expression::Array(array_expr) => {
                if array_expr.elements.is_empty() {
                    // Array vide, créer une variable de type pour les éléments
                    let elem_type = self.fresh_type_var(Some("ArrayElem".to_string()));
                    let  ctx = self.context.borrow_mut();
                    let mut type_system = ctx.types.borrow_mut();
                    Ok(type_system.create_array_type(elem_type, Some(0)))
                } else {
                    // Inférer le type du premier élément
                    let first_elem_type = self.infer_expression(&array_expr.elements[0])?;
                    
                    // Tous les éléments doivent avoir le même type
                    for elem in array_expr.elements.iter().skip(1) {
                        let elem_type = self.infer_expression(elem)?;
                        self.constraints.push(TypeConstraint::Equal(first_elem_type, elem_type));
                    }
                    
                    let  ctx = self.context.borrow_mut();
                    let mut type_system = ctx.types.borrow_mut();
                    Ok(type_system.create_array_type(
                        first_elem_type, 
                        Some(array_expr.elements.len())
                    ))
                }
            }
            
            _ => {
                // Pour les autres expressions, créer une variable de type
                Ok(self.fresh_type_var(None))
            }
        }
    }
    
    /// Infère le type d'un statement
    pub fn infer_statement(&mut self, stmt: &Statement) -> Result<(), SemanticError> {
        match stmt {
            Statement::DeclarationStatement(decl) => {
                self.infer_declaration(decl)
            }
            
            Statement::Expression(expr) => {
                self.infer_expression(expr)?;
                Ok(())
            }
            
            Statement::Assignment(target, value) => {
                let target_type = self.infer_expression(target)?;
                let value_type = self.infer_expression(value)?;
                self.constraints.push(TypeConstraint::Equal(target_type, value_type));
                Ok(())
            }
            
            Statement::IfStatement(if_stmt) => {
                // La condition doit être booléenne
                let cond_type = self.infer_expression(&if_stmt.condition)?;
                let bool_type = {
                    let ctx = self.context.borrow();
                    let type_system = ctx.types.borrow();
                    type_system.type_bool
                };
                self.constraints.push(TypeConstraint::Equal(cond_type, bool_type));
                
                // Inférer le bloc then
                for stmt in &if_stmt.then_block {
                    if let ASTNode::Statement(s) = stmt {
                        self.infer_statement(s)?;
                    }
                }
                
                // Inférer les blocs elif
                for elif in &if_stmt.elif_block {
                    let elif_cond_type = self.infer_expression(&elif.condition)?;
                    self.constraints.push(TypeConstraint::Equal(elif_cond_type, bool_type));
                    for stmt in &elif.block {
                        if let ASTNode::Statement(s) = stmt {
                            self.infer_statement(s)?;
                        }
                    }
                }
                
                // Inférer le bloc else
                if let Some(else_block) = &if_stmt.else_block {
                    for stmt in else_block {
                        if let ASTNode::Statement(s) = stmt {
                            self.infer_statement(s)?;
                        }
                    }
                }
                
                Ok(())
            }
            
            Statement::WhileStatement(while_stmt) => {
                // La condition doit être booléenne
                let cond_type = self.infer_expression(&while_stmt.condition)?;
                let bool_type = {
                    let ctx = self.context.borrow();
                    let type_system = ctx.types.borrow();
                    type_system.type_bool
                };
                self.constraints.push(TypeConstraint::Equal(cond_type, bool_type));
                
                // Inférer le corps
                for stmt in &while_stmt.body {
                    if let ASTNode::Statement(s) = stmt {
                        self.infer_statement(s)?;
                    }
                }
                
                Ok(())
            }
            
            Statement::ReturnStatement(ret_stmt) => {
                if let Some(expr) = &ret_stmt.value {
                    self.infer_expression(expr)?;
                }
                Ok(())
            }
            
            _ => Ok(()) // Autres statements non gérés pour l'instant
        }
    }
    
    /// Infère le type d'une déclaration
    pub fn infer_declaration(&mut self, decl: &Declaration) -> Result<(), SemanticError> {
        match decl {
            Declaration::Variable(var_decl) => {
                if let Some(expr) = &var_decl.value {
                    let expr_type = self.infer_expression(expr)?;
                    
                    // Si un type est spécifié, vérifier la compatibilité
                    if let Some(declared_type) = &var_decl.variable_type {
                        let ctx = self.context.borrow();
                        let mut type_system = ctx.types.borrow_mut();
                        let declared_type_id = type_system.convert_ast_type(declared_type);
                        self.constraints.push(TypeConstraint::Equal(expr_type, declared_type_id));
                    }
                    
                    // Enregistrer le type de la variable
                    let  ctx = self.context.borrow_mut();
                    let mut symbols = ctx.symbols.borrow_mut();
                    if let Ok(symbol_id) = symbols.lookup_symbol(&var_decl.name) {
                        if let Some(symbol) = symbols.get_symbol_mut(symbol_id) {
                            symbol.attributes.type_info = Some(expr_type);
                        }
                    }
                }
                Ok(())
            }
            
            Declaration::Function(func_decl) => {
                // Créer un nouveau scope pour la fonction
                // Inférer les types du corps
                for stmt in &func_decl.body {
                    if let ASTNode::Statement(s) = stmt {
                        self.infer_statement(s)?;
                    }
                }
                Ok(())
            }
            
            _ => Ok(()) // Autres déclarations non gérées pour l'instant
        }
    }
}

// Fonction helper pour créer une erreur sémantique
fn create_semantic_error(error_type: SemanticErrorType, message: String, position: Position) -> SemanticError {
    SemanticError::new(error_type, message, position)
}