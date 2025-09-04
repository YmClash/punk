// src/semantic/optimizations/constant_folding.rs

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use crate::parser::ast::{Expression, Statement, Declaration, ASTNode, Literal, 
                         BinaryOperation, UnaryOperation, Operator, UnaryOperator};
use crate::semantic::context::CompilationContext;
use crate::semantic::types::type_system::TypeId;

/// Valeur constante évaluée
#[derive(Debug, Clone, PartialEq)]
pub enum ConstantValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Char(char),
    Null,
}

impl ConstantValue {
    /// Convertit en literal AST
    pub fn to_literal(&self) -> Literal {
        use num_bigint::BigInt;
        match self {
            ConstantValue::Integer(value) => Literal::Integer { value: BigInt::from(*value) },
            ConstantValue::Float(value) => Literal::Float { value: *value },
            ConstantValue::Boolean(value) => Literal::Boolean(*value),
            ConstantValue::String(value) => Literal::String(value.clone()),
            ConstantValue::Char(value) => Literal::Char(*value),
            ConstantValue::Null => Literal::String("null".to_string()), // Pas de Literal::None
        }
    }
    
    /// Convertit depuis un literal AST
    pub fn from_literal(lit: &Literal) -> Option<Self> {
        use num_traits::ToPrimitive;
        match lit {
            Literal::Integer { value } => value.to_i64().map(ConstantValue::Integer),
            Literal::Float { value } => Some(ConstantValue::Float(*value)),
            Literal::Boolean(value) => Some(ConstantValue::Boolean(*value)),
            Literal::String(value) => Some(ConstantValue::String(value.clone())),
            Literal::Char(value) => Some(ConstantValue::Char(*value)),
            // Literal::None n'existe pas dans notre AST
            _ => None,
        }
    }
}

/// Optimiseur de pliage de constantes
pub struct ConstantFolder {
    context: Rc<RefCell<CompilationContext>>,
    /// Cache des valeurs constantes connues
    constant_values: HashMap<String, ConstantValue>,
    /// Statistiques d'optimisation
    pub stats: FoldingStats,
}

#[derive(Debug, Default)]
pub struct FoldingStats {
    pub expressions_folded: usize,
    pub arithmetic_operations: usize,
    pub boolean_operations: usize,
    pub string_concatenations: usize,
    pub comparisons_folded: usize,
}

impl ConstantFolder {
    pub fn new(context: Rc<RefCell<CompilationContext>>) -> Self {
        ConstantFolder {
            context,
            constant_values: HashMap::new(),
            stats: FoldingStats::default(),
        }
    }
    
    /// Optimise un AST complet
    pub fn optimize_ast(&mut self, ast: &mut Vec<ASTNode>) {
        for node in ast.iter_mut() {
            self.optimize_node(node);
        }
    }
    
    /// Optimise un noeud AST
    fn optimize_node(&mut self, node: &mut ASTNode) {
        match node {
            ASTNode::Expression(expr) => {
                if let Some(optimized) = self.fold_expression(expr) {
                    *expr = optimized;
                }
            }
            ASTNode::Statement(stmt) => {
                self.optimize_statement(stmt);
            }
            ASTNode::Declaration(decl) => {
                self.optimize_declaration(decl);
            }
            ASTNode::Program(nodes) => {
                for sub_node in nodes.iter_mut() {
                    self.optimize_node(sub_node);
                }
            }
            _ => {}
        }
    }
    
    /// Optimise une déclaration
    fn optimize_declaration(&mut self, decl: &mut Declaration) {
        match decl {
            Declaration::Variable(var_decl) => {
                if let Some(ref mut expr) = var_decl.value {
                    if let Some(optimized) = self.fold_expression(expr) {
                        // Si c'est une constante, la stocker
                        if let Expression::Literal(lit) = &optimized {
                            if let Some(const_val) = ConstantValue::from_literal(lit) {
                                self.constant_values.insert(var_decl.name.clone(), const_val);
                            }
                        }
                        *expr = optimized;
                    }
                }
            }
            Declaration::Constante(const_decl) => {
                if let Some(optimized) = self.fold_expression(&const_decl.value) {
                    // Stocker la valeur constante
                    if let Expression::Literal(lit) = &optimized {
                        if let Some(const_val) = ConstantValue::from_literal(&lit) {
                            self.constant_values.insert(const_decl.name.clone(), const_val);
                        }
                    }
                }
            }
            Declaration::Function(func_decl) => {
                // Optimiser le corps de la fonction
                for node in &mut func_decl.body {
                    self.optimize_node(node);
                }
            }
            _ => {}
        }
    }
    
    /// Optimise un statement
    fn optimize_statement(&mut self, stmt: &mut Statement) {
        match stmt {
            Statement::Expression(expr) => {
                if let Some(optimized) = self.fold_expression(expr) {
                    *expr = optimized;
                }
            }
            Statement::IfStatement(if_stmt) => {
                // Optimiser la condition
                if let Some(optimized) = self.fold_expression(&if_stmt.condition) {
                    if_stmt.condition = optimized;
                    
                    // Si la condition est constante, simplifier le if
                    if let Expression::Literal(Literal::Boolean(value)) = &if_stmt.condition {
                        if *value {
                            // Toujours vrai - garder seulement le then
                            // Note: dans une vraie implémentation, on remplacerait le statement
                        } else {
                            // Toujours faux - garder seulement le else
                            // Note: dans une vraie implémentation, on remplacerait le statement
                        }
                    }
                }
                
                // Optimiser les branches
                for node in &mut if_stmt.then_block {
                    self.optimize_node(node);
                }
                // Optimiser les elif
                for elif in &mut if_stmt.elif_block {
                    if let Some(optimized) = self.fold_expression(&elif.condition) {
                        elif.condition = optimized;
                    }
                    for node in &mut elif.block {
                        self.optimize_node(node);
                    }
                }
                if let Some(ref mut else_block) = if_stmt.else_block {
                    for node in else_block {
                        self.optimize_node(node);
                    }
                }
            }
            Statement::WhileStatement(while_stmt) => {
                if let Some(optimized) = self.fold_expression(&while_stmt.condition) {
                    while_stmt.condition = optimized;
                }
                for node in &mut while_stmt.body {
                    self.optimize_node(node);
                }
            }
            Statement::ForStatement(for_stmt) => {
                if let Some(optimized) = self.fold_expression(&for_stmt.iterable) {
                    for_stmt.iterable = optimized;
                }
                for node in &mut for_stmt.body {
                    self.optimize_node(node);
                }
            }
            Statement::ReturnStatement(ret_stmt) => {
                if let Some(ref mut expr) = ret_stmt.value {
                    if let Some(optimized) = self.fold_expression(expr) {
                        *expr = optimized;
                    }
                }
            }
            _ => {}
        }
    }
    
    /// Plie une expression si possible
    pub fn fold_expression(&mut self, expr: &Expression) -> Option<Expression> {
        match expr {
            Expression::BinaryOperation(binop) => {
                self.fold_binary_operation(binop)
            }
            Expression::UnaryOperation(unop) => {
                self.fold_unary_operation(unop)
            }
            Expression::Identifier(name) => {
                // Si c'est une constante connue, la remplacer
                if let Some(const_val) = self.constant_values.get(name) {
                    self.stats.expressions_folded += 1;
                    Some(Expression::Literal(const_val.to_literal()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    
    /// Plie une opération binaire
    fn fold_binary_operation(&mut self, binop: &BinaryOperation) -> Option<Expression> {
        // D'abord, essayer d'optimiser les opérandes
        let left_opt = self.fold_expression(&binop.left);
        let right_opt = self.fold_expression(&binop.right);
        
        let left = left_opt.clone().unwrap_or_else(|| (*binop.left).clone());
        let right = right_opt.clone().unwrap_or_else(|| (*binop.right).clone());
        
        // Si les deux opérandes sont des constantes, évaluer
        if let (Expression::Literal(left_lit), Expression::Literal(right_lit)) = (&left, &right) {
            match binop.operator {
                // Opérations arithmétiques
                Operator::Addition => {
                    self.stats.arithmetic_operations += 1;
                    self.fold_arithmetic_op(left_lit, right_lit, |a, b| a + b, |a, b| a + b)
                }
                Operator::Subtraction => {
                    self.stats.arithmetic_operations += 1;
                    self.fold_arithmetic_op(left_lit, right_lit, |a, b| a - b, |a, b| a - b)
                }
                Operator::Multiplication => {
                    self.stats.arithmetic_operations += 1;
                    self.fold_arithmetic_op(left_lit, right_lit, |a, b| a * b, |a, b| a * b)
                }
                Operator::Division => {
                    self.stats.arithmetic_operations += 1;
                    self.fold_division(left_lit, right_lit)
                }
                Operator::Modulo => {
                    self.stats.arithmetic_operations += 1;
                    self.fold_modulo(left_lit, right_lit)
                }
                
                // Comparaisons
                Operator::Equal => {
                    self.stats.comparisons_folded += 1;
                    self.fold_equal_comparison(left_lit, right_lit)
                }
                Operator::NotEqual => {
                    self.stats.comparisons_folded += 1;
                    self.fold_not_equal_comparison(left_lit, right_lit)
                }
                Operator::LessThan => {
                    self.stats.comparisons_folded += 1;
                    self.fold_less_than_comparison(left_lit, right_lit)
                }
                Operator::GreaterThan => {
                    self.stats.comparisons_folded += 1;
                    self.fold_greater_than_comparison(left_lit, right_lit)
                }
                Operator::LessThanOrEqual => {
                    self.stats.comparisons_folded += 1;
                    self.fold_less_equal_comparison(left_lit, right_lit)
                }
                Operator::GreaterThanOrEqual => {
                    self.stats.comparisons_folded += 1;
                    self.fold_greater_equal_comparison(left_lit, right_lit)
                }
                
                // Opérations logiques
                Operator::And => {
                    self.stats.boolean_operations += 1;
                    self.fold_logical_and(left_lit, right_lit)
                }
                Operator::Or => {
                    self.stats.boolean_operations += 1;
                    self.fold_logical_or(left_lit, right_lit)
                }
                
                _ => None,
            }
        } else if left_opt.is_some() || right_opt.is_some() {
            // Au moins un opérande a été optimisé
            Some(Expression::BinaryOperation(BinaryOperation {
                left: Box::new(left),
                operator: binop.operator.clone(),
                right: Box::new(right),
            }))
        } else {
            None
        }
    }
    
    /// Plie une opération unaire
    fn fold_unary_operation(&mut self, unop: &UnaryOperation) -> Option<Expression> {
        let operand_opt = self.fold_expression(&unop.operand);
        let operand = operand_opt.clone().unwrap_or_else(|| (*unop.operand).clone());
        
        if let Expression::Literal(lit) = &operand {
            match unop.operator {
                UnaryOperator::Negate| UnaryOperator::Negative => {
                    use num_bigint::BigInt;
                    self.stats.arithmetic_operations += 1;
                    match lit {
                        Literal::Integer { value } => {
                            Some(Expression::Literal(Literal::Integer { value: -value }))
                        }
                        Literal::Float { value } => {
                            Some(Expression::Literal(Literal::Float { value: -value }))
                        }
                        _ => None,
                    }
                }
                UnaryOperator::Not | UnaryOperator::LogicalNot => {
                    self.stats.boolean_operations += 1;
                    match lit {
                        Literal::Boolean(value) => {
                            Some(Expression::Literal(Literal::Boolean(!value)))
                        }
                        _ => None,
                    }
                }
                _ => None,
            }
        } else if operand_opt.is_some() {
            // L'opérande a été optimisé
            Some(Expression::UnaryOperation(UnaryOperation {
                operator: unop.operator.clone(),
                operand: Box::new(operand),
            }))
        } else {
            None
        }
    }
    
    /// Plie une opération arithmétique
    fn fold_arithmetic_op<F1, F2>(
        &mut self,
        left: &Literal,
        right: &Literal,
        int_op: F1,
        float_op: F2,
    ) -> Option<Expression>
    where
        F1: Fn(i64, i64) -> i64,
        F2: Fn(f64, f64) -> f64,
    {
        use num_bigint::BigInt;
        use num_traits::ToPrimitive;
        
        self.stats.expressions_folded += 1;
        match (left, right) {
            (Literal::Integer { value: a }, Literal::Integer { value: b }) => {
                if let (Some(a_val), Some(b_val)) = (a.to_i64(), b.to_i64()) {
                    Some(Expression::Literal(Literal::Integer {
                        value: BigInt::from(int_op(a_val, b_val))
                    }))
                } else {
                    None
                }
            }
            (Literal::Float { value: a }, Literal::Float { value: b }) => {
                Some(Expression::Literal(Literal::Float {
                    value: float_op(*a, *b)
                }))
            }
            (Literal::Integer { value: a }, Literal::Float { value: b }) => {
                if let Some(a_val) = a.to_f64() {
                    Some(Expression::Literal(Literal::Float {
                        value: float_op(a_val, *b)
                    }))
                } else {
                    None
                }
            }
            (Literal::Float { value: a }, Literal::Integer { value: b }) => {
                if let Some(b_val) = b.to_f64() {
                    Some(Expression::Literal(Literal::Float {
                        value: float_op(*a, b_val)
                    }))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    
    /// Plie une division
    fn fold_division(&mut self, left: &Literal, right: &Literal) -> Option<Expression> {
        use num_traits::Zero;
        // Vérifier la division par zéro
        match right {
            Literal::Integer { value } if value.is_zero() => {
                return None; // Ne pas plier la division par zéro
            }
            Literal::Float { value } if *value == 0.0 => {
                return None; // Ne pas plier la division par zéro
            }
            _ => {}
        }
        
        self.fold_arithmetic_op(left, right, |a, b| a / b, |a, b| a / b)
    }
    
    /// Plie un modulo
    fn fold_modulo(&mut self, left: &Literal, right: &Literal) -> Option<Expression> {
        use num_bigint::BigInt;
        use num_traits::{ToPrimitive, Zero};
        
        self.stats.expressions_folded += 1;
        match (left, right) {
            (Literal::Integer { value: a }, Literal::Integer { value: b }) if !b.is_zero() => {
                Some(Expression::Literal(Literal::Integer {
                    value: a % b
                }))
            }
            _ => None,
        }
    }
    
    /// Comparaisons d'égalité
    fn fold_equal_comparison(&mut self, left: &Literal, right: &Literal) -> Option<Expression> {
        self.stats.expressions_folded += 1;
        let result = match (left, right) {
            (Literal::Integer { value: a }, Literal::Integer { value: b }) => *a == *b,
            (Literal::Float { value: a }, Literal::Float { value: b }) => *a == *b,
            (Literal::Boolean(a), Literal::Boolean(b)) => *a == *b,
            (Literal::String(a), Literal::String(b)) => *a == *b,
            (Literal::Char(a), Literal::Char(b)) => *a == *b,
            _ => false,
        };
        Some(Expression::Literal(Literal::Boolean(result)))
    }
    
    fn fold_not_equal_comparison(&mut self, left: &Literal, right: &Literal) -> Option<Expression> {
        self.stats.expressions_folded += 1;
        let result = match (left, right) {
            (Literal::Integer { value: a }, Literal::Integer { value: b }) => *a != *b,
            (Literal::Float { value: a }, Literal::Float { value: b }) => *a != *b,
            (Literal::Boolean(a), Literal::Boolean(b)) => *a != *b,
            (Literal::String(a), Literal::String(b)) => *a != *b,
            (Literal::Char(a), Literal::Char(b)) => *a != *b,
            _ => true,
        };
        Some(Expression::Literal(Literal::Boolean(result)))
    }
    
    fn fold_less_than_comparison(&mut self, left: &Literal, right: &Literal) -> Option<Expression> {
        self.stats.expressions_folded += 1;
        let result = match (left, right) {
            (Literal::Integer { value: a }, Literal::Integer { value: b }) => a < b,
            (Literal::Float { value: a }, Literal::Float { value: b }) => a < b,
            (Literal::String(a), Literal::String(b)) => a < b,
            _ => false,
        };
        Some(Expression::Literal(Literal::Boolean(result)))
    }
    
    fn fold_greater_than_comparison(&mut self, left: &Literal, right: &Literal) -> Option<Expression> {
        self.stats.expressions_folded += 1;
        let result = match (left, right) {
            (Literal::Integer { value: a }, Literal::Integer { value: b }) => a > b,
            (Literal::Float { value: a }, Literal::Float { value: b }) => a > b,
            (Literal::String(a), Literal::String(b)) => a > b,
            _ => false,
        };
        Some(Expression::Literal(Literal::Boolean(result)))
    }
    
    fn fold_less_equal_comparison(&mut self, left: &Literal, right: &Literal) -> Option<Expression> {
        self.stats.expressions_folded += 1;
        let result = match (left, right) {
            (Literal::Integer { value: a }, Literal::Integer { value: b }) => a <= b,
            (Literal::Float { value: a }, Literal::Float { value: b }) => a <= b,
            (Literal::String(a), Literal::String(b)) => a <= b,
            _ => false,
        };
        Some(Expression::Literal(Literal::Boolean(result)))
    }
    
    fn fold_greater_equal_comparison(&mut self, left: &Literal, right: &Literal) -> Option<Expression> {
        self.stats.expressions_folded += 1;
        let result = match (left, right) {
            (Literal::Integer { value: a }, Literal::Integer { value: b }) => a >= b,
            (Literal::Float { value: a }, Literal::Float { value: b }) => a >= b,
            (Literal::String(a), Literal::String(b)) => a >= b,
            _ => false,
        };
        Some(Expression::Literal(Literal::Boolean(result)))
    }
    
    /// Plie un AND logique
    fn fold_logical_and(&mut self, left: &Literal, right: &Literal) -> Option<Expression> {
        self.stats.expressions_folded += 1;
        match (left, right) {
            (Literal::Boolean(a), Literal::Boolean(b)) => {
                Some(Expression::Literal(Literal::Boolean(*a && *b)))
            }
            // Court-circuit: false && x = false
            (Literal::Boolean(false), _) | (_, Literal::Boolean(false)) => {
                Some(Expression::Literal(Literal::Boolean(false)))
            }
            // true && x = x
            (Literal::Boolean(true), _) => {
                Some(Expression::Literal(right.clone()))
            }
            (_, Literal::Boolean(true)) => {
                Some(Expression::Literal(left.clone()))
            }
            _ => None,
        }
    }
    
    /// Plie un OR logique
    fn fold_logical_or(&mut self, left: &Literal, right: &Literal) -> Option<Expression> {
        self.stats.expressions_folded += 1;
        match (left, right) {
            (Literal::Boolean(a), Literal::Boolean(b)) => {
                Some(Expression::Literal(Literal::Boolean(*a || *b)))
            }
            // Court-circuit: true || x = true
            (Literal::Boolean(true), _) | (_, Literal::Boolean(true)) => {
                Some(Expression::Literal(Literal::Boolean(true)))
            }
            // false || x = x
            (Literal::Boolean(false), _) => {
                Some(Expression::Literal(right.clone()))
            }
            (_, Literal::Boolean(false)) => {
                Some(Expression::Literal(left.clone()))
            }
            _ => None,
        }
    }
    
    /// Affiche les statistiques d'optimisation
    pub fn display_stats(&self) {
        println!("=== Constant Folding Statistics ===");
        println!("Expressions folded: {}", self.stats.expressions_folded);
        println!("Arithmetic operations: {}", self.stats.arithmetic_operations);
        println!("Boolean operations: {}", self.stats.boolean_operations);
        println!("String concatenations: {}", self.stats.string_concatenations);
        println!("Comparisons folded: {}", self.stats.comparisons_folded);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fold_integer_addition() {
        use num_bigint::BigInt;
        let context = Rc::new(RefCell::new(CompilationContext::new()));
        let mut folder = ConstantFolder::new(context);

        let expr = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(5) })),
            operator: Operator::Addition,
            right: Box::new(Expression::Literal(Literal::Integer { value: BigInt::from(3) })),
        });

        let result = folder.fold_expression(&expr);
        if let Some(Expression::Literal(Literal::Integer { value })) = result {
            assert_eq!(value, BigInt::from(8));
        } else {
            panic!("Expected folded integer literal");
        }
    }

    #[test]
    fn test_fold_boolean_and() {
        let context = Rc::new(RefCell::new(CompilationContext::new()));
        let mut folder = ConstantFolder::new(context);

        let expr = Expression::BinaryOperation(BinaryOperation {
            left: Box::new(Expression::Literal(Literal::Boolean(true))),
            operator: Operator::And,
            right: Box::new(Expression::Literal(Literal::Boolean(false))),
        });

        let result = folder.fold_expression(&expr);
        if let Some(Expression::Literal(Literal::Boolean(value))) = result {
            assert_eq!(value, false);
        } else {
            panic!("Expected folded boolean literal");
        }
    }
}