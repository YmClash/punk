//src/


use crate::parser::ast::{ASTNode, Expression, Statement, Operator, Literal, Declaration, IfStatement};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use num_bigint::BigInt;
use num_traits::{Zero,ToPrimitive};
use crate::interpreter::environment::Environment;
use crate::interpreter::error::RuntimeError;
use crate::interpreter::value::Value ;
use crate::interpreter::evaluator::Evaluator;


pub struct Interpreter {
    // environment: Rc<RefCell<Environment>>,
    evaluator: Evaluator,
}



impl Interpreter{
    pub fn new() -> Self {
        let evaluator = Evaluator::new();
        Interpreter{
            // environment: Rc::new(RefCell::new(Environment::new())),
            evaluator:Evaluator::new(),
        }
    }

    pub fn interpret(&mut self, nodes: &[ASTNode]) -> Result<Value,RuntimeError> {
        let mut result = Value::Null;
        for node in nodes {
            // result = self.evaluate_node(node)?;
            // result = Evaluator::evaluate_node(self, node)?;
            result = self.evaluator.evaluate_node(node)?;
        }
        Ok(result)
    }

    // fn execute_block(&mut self, nodes: &[ASTNode]) -> Result<Value, RuntimeError> {
    //     let parent_env = self.environment.clone();
    //     self.environment = Rc::new(RefCell::new(Environment::with_parent(parent_env.clone())));
    //
    //     let mut result = Value::Null;
    //     for  node in nodes {
    //         result = self.evaluate_node(node)?;
    //     }
    //     self.environment = parent_env;
    //     Ok(result)
    // }


}