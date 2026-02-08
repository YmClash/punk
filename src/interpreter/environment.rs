// src/interpreter/environment.rs

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use super::value::Value;
use super::error::RuntimeError;

#[derive(Clone, Debug)]
pub struct Environment {
    pub values: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Environment>>>,
}


impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Rc<RefCell<Environment>>) -> Self {
        Environment {
            values: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Result<Value, RuntimeError> {
        if let Some(value) = self.values.get(name) {
            Ok(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            Err(RuntimeError::UndefinedVariable(name.to_string()))
        }
    }

    pub fn set(&mut self, name: &str, value: Value) -> Result<(), RuntimeError> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().set(name, value)
        } else {
            Err(RuntimeError::UndefinedVariable(name.to_string()))
        }
    }

    pub fn set_or_define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }
}