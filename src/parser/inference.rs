use std::collections::HashMap;
use crate::parser::ast::{Assignment, BinaryOperation, Expression, Literal, Operator, Type, UnaryOperation, UnaryOperator, VariableDeclaration};



#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct TypeContext {
    type_vars: HashMap<String, Type>,
    constraints: Vec<TypeConstraint>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum TypeConstraint {
    Equal(Type, Type),
    Subtype(Type, Type),
    Instance(Type, Vec<Type>),
}


impl TypeContext {
    pub fn new() -> Self {
        TypeContext {
            type_vars: HashMap::new(),
            constraints: Vec::new(),
        }
    }

    pub fn infer_expression(&mut self, expr: &Expression) -> Result<Type, String> {
        match expr {
            Expression::Literal(lit) => self.infer_literal(lit),
            Expression::Identifier(name) => self.lookup_type(name),
            Expression::BinaryOperation(binop) => self.infer_binary_op(binop),
            Expression::Assignment(assign) => self.infer_assignment(assign),
            Expression::UnaryOperation(unop) => self.infer_unary_op(unop),
            _ => Ok(Type::Infer), // Pour les autres cas
        }
    }

    fn infer_literal(&self, lit: &Literal) -> Result<Type, String> {
        match lit {
            Literal::Integer { .. } => Ok(Type::Int),
            Literal::Float { .. } => Ok(Type::Float),
            // Literal::String(_) => Ok(Type::String),
            Literal::String(s) => {
                // Si c'est un seul caractère entre guillemets simples
                if s.len() == 1 && s.starts_with('\'') && s.ends_with('\'') {
                    Ok(Type::Char)
                } else {
                    Ok(Type::String)
                }
            }
            Literal::Boolean(_) => Ok(Type::Bool),
            Literal::Char(_) => Ok(Type::Char), // Ajout  l'inference de type pour les caractères
            _ => Ok(Type::Infer),
        }
    }

    fn infer_binary_op(&mut self, binop: &BinaryOperation) -> Result<Type, String> {
        let left_type = self.infer_expression(&binop.left)?;
        let right_type = self.infer_expression(&binop.right)?;

        match binop.operator {
            Operator::Addition | Operator::Substraction |
            Operator::Multiplication | Operator::Division => {
                if left_type == Type::Int && right_type == Type::Int {
                    Ok(Type::Int)
                } else if left_type == Type::Float || right_type == Type::Float {
                    Ok(Type::Float)
                } else if left_type == Type::Infer || right_type == Type::Infer {
                    Ok(Type::Infer)
                } else {
                    Err("Type mismatch in binary operation".to_string())
                }
            },
            Operator::Equal | Operator::NotEqual |
            Operator::LessThan | Operator::GreaterThan |
            Operator::LesshanOrEqual | Operator::GreaterThanOrEqual => {
                self.add_constraint(TypeConstraint::Equal(left_type, right_type));
                Ok(Type::Bool)
            },
            _ => Ok(Type::Infer),
        }
    }

    fn infer_assignment(&mut self, assign: &Assignment) -> Result<Type, String> {
        let value_type = self.infer_expression(&assign.value)?;

        match &*assign.target {
            Expression::Identifier(name) => {
                self.type_vars.insert(name.clone(), value_type.clone());
                Ok(value_type)
            },
            _ => Err("Invalid assignment target".to_string())
        }
    }

    fn infer_variable_declaration(&mut self, decl: &VariableDeclaration)
                                  -> Result<Type, String> {
        let inferred_type = if let Some(ref expr) = decl.value {
            self.infer_expression(expr)?
        } else {
            Type::Infer
        };

        if let Some(ref explicit_type) = decl.variable_type {
            if explicit_type != &Type::Infer && explicit_type != &inferred_type {
                return Err(format!(
                    "Type mismatch: expected {:?}, found {:?}",
                    explicit_type, inferred_type
                ));
            }
            Ok(explicit_type.clone())
        } else {
            Ok(inferred_type)
        }
    }

    fn infer_unary_op(&mut self, unop: &UnaryOperation) -> Result<Type, String> {
        let operand_type = self.infer_expression(&unop.operand)?;

        match unop.operator {
            UnaryOperator::Negative => {
                match operand_type {
                    Type::Int => Ok(Type::Int),
                    Type::Float => Ok(Type::Float),
                    Type::Infer => Ok(Type::Infer),
                    _ => Err("Operator '-' cannot be applied to this type".to_string())
                }
            },
            UnaryOperator::Not => {
                match operand_type {
                    Type::Bool => Ok(Type::Bool),
                    Type::Infer => Ok(Type::Infer),
                    _ => Err("Operator '!' can only be applied to boolean types".to_string())
                }
            },
            UnaryOperator::Reference => Ok(Type::Array(Box::new(operand_type))),
            UnaryOperator::ReferenceMutable => Ok(Type::Array(Box::new(operand_type))),
            _ => todo!(),
        }
    }





    fn add_constraint(&mut self, constraint: TypeConstraint) {
        self.constraints.push(constraint);
    }

    fn lookup_type(&self, name: &str) -> Result<Type, String> {
        self.type_vars
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Undefined variable: {}", name))
    }
}
