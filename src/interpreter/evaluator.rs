// // src/interpreter/evaluator.rs
//
use crate::parser::ast::*;
// use super::value::{ClosureValue, FunctionValue, Value};
// use super::environment::Environment;
use super::error::RuntimeError;
use super::builtins;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use num_traits::ToPrimitive;
use crate::interpreter::environment::Environment;
// use crate::interpreter::value::Value::Callable;
use crate::interpreter::value::{Callable, FunctionValue, Value};

pub struct Evaluator {
    pub env: Rc<RefCell<Environment>>,
    pub return_value: Option<Value>,
    pub break_flag: bool,
    pub continue_flag: bool,
    // environment: Environment
}

impl Evaluator {
    pub fn new() -> Self {
        let env = Rc::new(RefCell::new(Environment::new()));

        // Ajouter les fonctions built-in
        builtins::register_builtins(&mut env.borrow_mut());

        Evaluator {
            env,
            return_value: None,
            break_flag: false,
            continue_flag: false,
            // environment: Environment::new(),
        }
    }

    pub fn syntax_mode(&mut self){
        todo!()

    }


    pub fn evaluate_node(&mut self, node: &ASTNode) -> Result<Value, RuntimeError> {
        match node {
            ASTNode::Expression(expr) => self.evaluate_expression(expr),
            ASTNode::Statement(stmt) => self.evaluate_statement(stmt),
            // ASTNode::Declaration(decl) => self.evaluate_declaration(decl),
            ASTNode::Declaration(decl) => {
                let stmt = Statement::DeclarationStatement(decl.clone());
                self.evaluate_statement(&stmt)
            }
            // ASTNode::Program(nodes) => {
            //     let mut last = Value::None;
            //     for node in nodes {
            //         last = self.evaluate_node(node)?;
            //         if self.return_value.is_some() {
            //             return Ok(self.return_value.take().unwrap());
            //         }
            //     }
            //     Ok(last)
            // },
            ASTNode::Error(err) => Err(RuntimeError::SyntaxError(err.clone())),
            _ => Err(RuntimeError::Unimplemented(
                format!("ASTNode {:?} not implemented", node)
            )),
        }
    }

    pub fn evaluate_expression(&mut self, expr: &Expression) -> Result<Value, RuntimeError> {
        match expr {
            // Evaluer les littéraux
            Expression::Literal(literal) => match literal{
                Literal::Integer { value } => Ok(Value::Int(value.clone())),
                Literal::Float { value } => Ok(Value::Float(*value)),
                Literal::String(value) => Ok(Value::String(value.clone())),
                Literal::Boolean(value) => Ok(Value::Bool(*value)),
                Literal::Char(value) => Ok(Value::Char(*value)),
                _ => Err(RuntimeError::Unimplemented(
                    format!("Literal {:?} not implemented", literal)
                )),

            },

            // Evaluer les identifiants
            Expression::Identifier(name) => {
                self.env.borrow().get(name)
            },

            // Evaluer les Assignements
            Expression::Assignment(assignement ) => {
                let value = self.evaluate_expression(&assignement.value)?;
                if let Expression::Identifier(name) = &*assignement.target{
                    self.env.borrow_mut().set(name,value.clone())?;
                    Ok(value)
                }else {
                    Err(RuntimeError::Unimplemented(
                        format!("Assignment to non-identifier target is not supported: {:?}", assignement.target)
                    ))
                }
            },

            Expression::Array(array_expr) => {
                let mut elements = Vec::new();
                for element_expr in &array_expr.elements{
                    elements.push(self.evaluate_expression(element_expr)?);
                }
                Ok(Value::Array(Rc::new(RefCell::new(elements))))
            }
            // Evaluer les appels de fonctions
            Expression::FunctionCall(call) => {
                let callee = self.evaluate_expression(&call.name)?;

                let mut arguments = Vec::new();
                for arg_expr in &call.arguments {
                    arguments.push(self.evaluate_expression(arg_expr)?);
                }

                match callee {
                    Value::BuiltinFunction(func) => {
                        func(&arguments)
                    },
                    Value::Function(func_val) =>{
                        // verfier l'arity ,
                        if arguments.len() !=func_val.params.iter().len(){
                            return Err(RuntimeError::ArityMismatch {
                                expected: func_val.params.len(),
                                got: arguments.len(),
                            });
                        }
                        // Create a new environment for the function execution, with the
                        // function's closure environment as its parent.
                        let func_env = Rc::new(RefCell::new(
                            Environment::with_parent(func_val.env.clone())
                        ));

                        // Bind arguments to parameters in this new environment.
                        for (param_name, arg_value) in func_val.params.iter().zip(arguments.iter()) {
                            func_env.borrow_mut().define(param_name.clone(), arg_value.clone());
                        }
                        let caller_env = self.env.clone();
                        self.env = func_env;

                        // Evaluate the function body.
                        let mut result = Value::Null;
                        for node in &func_val.body {
                            result = self.evaluate_node(node)?;
                            // Check for a 'return' statement that may have set a return value.
                            if self.return_value.is_some() {
                                result = self.return_value.take().unwrap();
                                break;
                            }
                        }

                        // Restore the caller's environment.
                        self.env = caller_env;

                        Ok(result)
                    }
                    _ => Err(RuntimeError::NotCallable(callee.type_name().to_string())),

                }
            },
            Expression::BinaryOperation(bin_op) => {
                let left = self.evaluate_expression(&bin_op.left)?;
                let right = self.evaluate_expression(&bin_op.right)?;
                self.eval_binary_op(&left, &bin_op.operator, &right)
                // match (&left, &right) {
                //     (Value::Int(l), Value::Int(r)) => match bin_op.operator {
                //         Operator::Addition => Ok(Value::Int(l + r)),
                //         Operator::Subtraction => Ok(Value::Int(l - r)),
                //         Operator::Multiplication => Ok(Value::Int(l * r)),
                //         Operator::Division => Ok(Value::Int(l / r)), // Note: BigInt division truncates.
                //         Operator::GreaterThan => Ok(Value::Bool(l > r)),
                //         Operator::LessThan => Ok(Value::Bool(l < r)),
                //         Operator::EqualEqual => Ok(Value::Bool(*l == *r)),
                //         Operator::NotEqual => Ok(Value::Bool(*l != *r)),
                //         _ => Err(RuntimeError::UnsupportedOperator(format!("{:?} for integers", bin_op.operator))),
                //     },
                //     (Value::Float(l), Value::Float(r)) => match bin_op.operator {
                //         Operator::Addition => Ok(Value::Float(l + r)),
                //         Operator::Subtraction => Ok(Value::Float(l - r)),
                //         Operator::Multiplication => Ok(Value::Float(l * r)),
                //         Operator::Division => Ok(Value::Float(l / r)),
                //         Operator::GreaterThan => Ok(Value::Bool(l > r)),
                //         Operator::LessThan => Ok(Value::Bool(l < r)),
                //         Operator::EqualEqual => Ok(Value::Bool(*l == *r)),
                //         Operator::NotEqual => Ok(Value::Bool(*l != *r)),
                //         _ => Err(RuntimeError::UnsupportedOperator(format!("{:?} for floats", bin_op.operator))),
                //     },
                //     // _ => Err(RuntimeError::UnsupportedOperator(format!({:?} for float)))
                //     _ => Err(RuntimeError::UnsupportedOperator(format!("Cannot apply operator {:?} to {:?} and {:?}", bin_op.operator, left, right))),
                // }

            },

            // Expression::UnaryOperation(unop) => {
            //     let operand = self.evaluate_expression(&unop.operand)?;
            //     self.eval_unary_op(&unop.operator, &operand)
            // },


            Expression::IndexAccess(access) => {
                // let array = self.evaluate_expression(&access.array)?;
                let collection_val = self.evaluate_expression(&access.array)?;
                let index_val = self.evaluate_expression(&access.index)?;
                // self.evaluate_index_access(&array, &index)
                self.evaluate_index_access(&collection_val, &index_val)
            },

            Expression::DictAccess(access) => {
                let dict_val = self.evaluate_expression(&access.dict)?;
                let key_val = self.evaluate_expression(&access.key)?;

                match (dict_val, key_val) {
                    (Value::Dictionary(dict_rc), Value::String(key)) => {
                        let dict = dict_rc.borrow();
                        match dict.get(&key) {
                            Some(value) => Ok(value.clone()),
                            None => Ok(Value::Null), // Return Null if key not found
                        }
                    },
                    (Value::Dictionary(_), other) => {
                        Err(RuntimeError::TypeError(
                            format!("Dictionary key must be a string, but got {}.", other.type_name())
                        ))
                    },
                    (other, _) => {
                        Err(RuntimeError::TypeError(
                            format!("Cannot access dictionary on a value of type '{}'.", other.type_name())
                        ))
                    }
                }
            },
            Expression::DictLiteral(dict_literal) => {
                let mut map = HashMap::new();
                for entry in &dict_literal.entries{
                    let key_val = self.evaluate_expression(&entry.key)?;
                    let key_str = match key_val {
                        Value::String(s) => s,
                        _ => return Err(RuntimeError::TypeError(
                            format!("Dictionary keys must be strings, got {}", key_val.type_name())
                        )),
                    };
                    let value = self.evaluate_expression(&entry.value)?;
                    map.insert(key_str, value);
                }
                Ok(Value::Dictionary(Rc::new(RefCell::new(map))))
            }

            // Expression::LambdaExpression() => {
            //     Ok(Value::Closure(ClosureValue {
            //         params: lambda.parameters.iter()
            //             .map(|p| p.name.clone())
            //             .collect(),
            //         body: lambda.body.clone(),
            //         captured: HashMap::new(), // TODO: Capturer les variables
            //     }))
            // },

            _ => Err(RuntimeError::Unimplemented(
                format!("Expression {:?} not implemented", expr)
            )),
        }
    }
//
//     fn eval_literal(&self, literal: &Literal) -> Result<Value, RuntimeError> {
//         Ok(match literal {
//             // Literal::Integer { value } => Value::Int(value.to_i64().unwrap_or(0)),
//             Literal::Integer { value } => Value::Int(i64::try_from(*value).unwrap()),
//             Literal::Float { value } => Value::Float(*value),
//             Literal::Boolean(b) => Value::Bool(*b),
//             Literal::String(s) => Value::String(s.clone()),
//             Literal::Char(c) => Value::Char(*c),
//             Literal::Array(elements) => {
//                 let mut values = Vec::new();
//                 for elem in elements {
//                     values.push(self.eval_literal(elem)?);
//                 }
//                 Value::Array(Rc::new(RefCell::new(values)))
//             },
//         })
//     }
//
    fn eval_binary_op(&self, left: &Value, op: &Operator, right: &Value) -> Result<Value, RuntimeError> {
        match (left, op, right) {
            // Arithmétique
            (Value::Int(a), Operator::Addition, Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Int(a), Operator::Subtraction, Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Int(a), Operator::Multiplication, Value::Int(b)) => Ok(Value::Int(a * b)),
            // (Value::Int(a), Operator::Division, Value::Int(b)) => {
            //     if *b == 0 {
            //         Err(RuntimeError::DivisionByZero)
            //     } else {
            //         Ok(Value::Int(a / b))
            //     }
            // },

            (Value::Float(a), Operator::Addition, Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Float(a), Operator::Subtraction, Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Float(a), Operator::Multiplication, Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Float(a), Operator::Division, Value::Float(b)) => {
                if *b == 0.0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(a / b))
                }
            },

            // Comparaisons
            (Value::Int(a), Operator::LessThan, Value::Int(b)) => Ok(Value::Bool(a < b)),
            (Value::Int(a), Operator::GreaterThan, Value::Int(b)) => Ok(Value::Bool(a > b)),
            (Value::Int(a), Operator::EqualEqual, Value::Int(b)) => Ok(Value::Bool(a == b)),
            (Value::Int(a), Operator::NotEqual, Value::Int(b)) => Ok(Value::Bool(a != b)),

            // Logique
            (Value::Bool(a), Operator::And, Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
            (Value::Bool(a), Operator::Or, Value::Bool(b)) => Ok(Value::Bool(*a || *b)),

            // Concaténation de strings
            (Value::String(a), Operator::Addition, Value::String(b)) => {
                Ok(Value::String(format!("{}{}", a, b)))
            },

            _ => Err(RuntimeError::TypeError(
                format!("Invalid operation {:?} between {} and {}",
                        op, left.type_name(), right.type_name())
            )),
        }
    }
//
//     fn eval_unary_op(&self, op: &UnaryOperator, operand: &Value) -> Result<Value, RuntimeError> {
//         match (op, operand) {
//             (UnaryOperator::Negative, Value::Int(n)) => Ok(Value::Int(-n)),
//             (UnaryOperator::Negative, Value::Float(f)) => Ok(Value::Float(-f)),
//             (UnaryOperator::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
//             _ => Err(RuntimeError::TypeError(
//                 format!("Invalid unary operation {:?} on {}", op, operand.type_name())
//             )),
//         }
//     }
//
    pub fn evaluate_statement(&mut self, stmt: &Statement) -> Result<Value, RuntimeError> {
        match stmt {
            Statement::Expression(expr) =>{
                self.evaluate_expression(expr)?;
                Ok(Value::None)
            },
            Statement::DeclarationStatement(decl) => {
                match decl {
                    Declaration::Variable(var_decl) => {
                        let value = if let Some(init) = &var_decl.value {
                            self.evaluate_expression(init)?
                        } else {
                            Value::None
                        };

                        self.env.borrow_mut().define(var_decl.name.clone(), value);
                        Ok(Value::None)
                    },

                    Declaration::Function(func_decl) => {
                        let func = Value::Function(FunctionValue {
                            name: func_decl.name.clone(),
                            params: func_decl.parameters.iter()
                                .map(|p| p.name.clone())
                                .collect(),
                            body: func_decl.body.clone(),
                            env: self.env.clone(),
                        });

                        self.env.borrow_mut().define(func_decl.name.clone(), func);
                        Ok(Value::None)
                    },

                    _ => Ok(Value::None),
                }
            },

            Statement::ReturnStatement(return_stmt) => {
                let value = if let Some(expr) = &return_stmt.value{
                    self.evaluate_expression(expr)?
                }else {
                    Value::None
                };
                self.return_value = Some(value);
                Ok(Value::None)
            },

            // Statement::ReturnStatement(expr) => {
            //     let value = self.evaluate_expression(expr)?;
            //     self.return_value = Some(value.clone());
            //     Ok(value)
            // },

            Statement::Break => {
                self.break_flag = true;
                Ok(Value::None)
            },

            Statement::Continue => {
                self.continue_flag = true;
                Ok(Value::None)
            },

            Statement::IfStatement(if_stmt) => {
                let condition = self.evaluate_expression(&if_stmt.condition)?;
                if condition.is_truthy() {
                    return self.evaluate_block(&if_stmt.then_block);
                }

                for elif in &if_stmt.elif_block {
                    let elif_condition = self.evaluate_expression(&elif.condition)?;
                    if elif_condition.is_truthy() {
                        return self.evaluate_block(&elif.block);
                    }
                }

                if let Some(alternative) = &if_stmt.else_block {
                    return self.evaluate_block(alternative);
                }

                Ok(Value::Null)
            },

            Statement::WhileStatement(while_stmt) => {
                let mut last = Value::None;

                while self.evaluate_expression(&while_stmt.condition)?.is_truthy() {
                    last = self.evaluate_block(&while_stmt.body)?;

                    if self.break_flag {
                        self.break_flag = false;
                        break;
                    }
                    if self.continue_flag {
                        self.continue_flag = false;
                        continue;
                    }
                    if self.return_value.is_some() {
                        return Ok(self.return_value.take().unwrap());
                    }
                }

                Ok(last)
            },

            Statement::ForStatement(for_stmt) => {
                let iterable = self.evaluate_expression(&for_stmt.iterable)?;

                match iterable {
                    Value::Array(arr) => {
                        let mut last = Value::None;

                        for value in arr.borrow().iter() {
                            self.env.borrow_mut().define(
                                for_stmt.iterator.clone(),
                                value.clone()
                            );

                            last = self.evaluate_block(&for_stmt.body)?;

                            if self.break_flag {
                                self.break_flag = false;
                                break;
                            }
                            if self.continue_flag {
                                self.continue_flag = false;
                                continue;
                            }
                            if self.return_value.is_some() {
                                return Ok(self.return_value.take().unwrap());
                            }
                        }

                        Ok(last)
                    },
                    _ => Err(RuntimeError::TypeError(
                        format!("Cannot iterate over {}", iterable.type_name())
                    )),
                }
            },

            _ => Ok(Value::Null),
        }
    }

    fn evaluate_block(&mut self, nodes: &[ASTNode]) -> Result<Value, RuntimeError>{
        let parent_env = self.env.clone();
        self.env = Rc::new(RefCell::new(Environment::with_parent(parent_env.clone())));

        let mut result = Value::Null;
        for node in nodes{
            result = self.evaluate_node(node)?;
        }
        self.env = parent_env;
        Ok(result)

    }
    //
    // fn eval_block(&mut self, statements: &[ASTNode]) -> Result<Value, RuntimeError> {
    //     let parent_env = self.env.clone();
    //     self.env = Rc::new(RefCell::new(Environment::with_parent(parent_env.clone())));



        // let mut last = Value::None;
        // // Créer un nouveau scope
        // let new_env = Rc::new(RefCell::new(
        //     Environment::with_parent(self.env.clone())
        // ));
        // let old_env = self.env.clone();
        // self.env = new_env;
        //
        // for stmt in statements {
        //     last = self.eval(stmt)?;
        //
        //     if self.return_value.is_some() || self.break_flag || self.continue_flag {
        //         break;
        //     }
        // }
        //
        // // Restaurer l'ancien scope
        // self.env = old_env;
        //
        // Ok(last)
    // }
//
//     fn eval_declaration(&mut self, decl: &Declaration) -> Result<Value, RuntimeError> {
//         match decl {
//             Declaration::Variable(var_decl) => {
//                 let value = if let Some(init) = &var_decl.value {
//                     self.eval_expression(init)?
//                 } else {
//                     Value::None
//                 };
//
//                 self.env.borrow_mut().define(var_decl.name.clone(), value);
//                 Ok(Value::None)
//             },
//
//             Declaration::Function(func_decl) => {
//                 let func = Value::Function(FunctionValue {
//                     name: func_decl.name.clone(),
//                     params: func_decl.parameters.iter()
//                         .map(|p| p.name.clone())
//                         .collect(),
//                     body: func_decl.body.clone(),
//                     env: self.env.clone(),
//                 });
//
//                 self.env.borrow_mut().define(func_decl.name.clone(), func);
//                 Ok(Value::None)
//             },
//
//             _ => Ok(Value::None),
//         }
//     }
//
//     fn eval_function_call(&mut self, call: &FunctionCall) -> Result<Value, RuntimeError> {
//         let func = self.eval_expression(&call.name)?;
//
//         // Évaluer les arguments
//         let mut args = Vec::new();
//         for arg in &call.arguments {
//             args.push(self.eval_expression(arg)?);
//         }
//
//         match func {
//             Value::Function(func_val) => {
//                 if args.len() != func_val.params.len() {
//                     return Err(RuntimeError::ArityMismatch {
//                         expected: func_val.params.len(),
//                         got: args.len(),
//                     });
//                 }
//
//                 // Créer un nouveau scope pour la fonction
//                 let func_env = Rc::new(RefCell::new(
//                     Environment::with_parent(func_val.env.clone())
//                 ));
//
//                 // Lier les paramètres
//                 for (param, arg) in func_val.params.iter().zip(args.iter()) {
//                     func_env.borrow_mut().define(param.clone(), arg.clone());
//                 }
//
//                 // Sauvegarder l'environnement actuel
//                 let old_env = self.env.clone();
//                 self.env = func_env;
//
//                 // Évaluer le corps de la fonction
//                 let mut result = Value::None;
//                 for stmt in &func_val.body {
//                     result = self.eval(stmt)?;
//                     if self.return_value.is_some() {
//                         result = self.return_value.take().unwrap();
//                         break;
//                     }
//                 }
//
//                 // Restaurer l'environnement
//                 self.env = old_env;
//
//                 Ok(result)
//             },
//
//             Value::BuiltinFunction(builtin) => builtin(&args),
//
//             Value::Closure(closure) => {
//                 // TODO: Implémenter l'évaluation des closures
//                 Err(RuntimeError::Unimplemented("Closure evaluation".to_string()))
//             },
//
//             _ => Err(RuntimeError::NotCallable(func.type_name().to_string())),
//         }
//     }
//


        // Evaluer l'accès aux éléments d'un tableau
    fn evaluate_index_access(&self, collection_val: &Value, index_val: &Value) -> Result<Value, RuntimeError> {
        match (collection_val, index_val) {
            (Value::Array(elements_rc), Value::Int(index_bigint)) => {
                use num_traits::ToPrimitive;
                let index = match index_bigint.to_usize() {
                    Some(i) => i,
                    None => return Err(RuntimeError::IndexOutOfBounds(
                        format!("Index {} is too large", index_bigint)
                    ))
                };
                let elements = elements_rc.borrow();
                if index < elements.len() {
                    Ok(elements[index].clone())
                }else {
                    Err(RuntimeError::IndexOutOfBounds(
                        format!("Index {} out of bounds for array of length {}", index, elements.len())
                    ))
                }
            },
            (Value::Dictionary(dict_rc), Value::String(key)) => {
                let dict = dict_rc.borrow();
                match dict.get(key) {
                    Some(value) => Ok(value.clone()),
                    None => Ok(Value::Null), // Return Null if key not found
                }
            },

            (Value::Array(_), other) => {
                Err(RuntimeError::IndexOutOfBounds(
                    format!("Array index must be an integer, got {}", other.type_name())
                ))
            },
            (Value::Dictionary(_), other) => {
              Err(RuntimeError::TypeError(
                    format!("Dictionary index must be a string, got {}", other.type_name())
              ))
            },
            (other,_) => {
                Err(RuntimeError::TypeError(
                    format!("Cannot index a value of type {} with {}", other.type_name(), index_val.type_name())
                ))
            }

        }
    }


}