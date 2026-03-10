use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::fmt::Display;
use num_bigint::BigInt;
use num_traits::Zero;
use crate::interpreter::environment::Environment;
use crate::interpreter::error::RuntimeError;
use crate::parser::ast::{ASTNode, Expression};

#[derive(Clone, Debug)]
pub enum Value{


    // Primitive types
    Int(BigInt),
    Float(f64),
    Bool(bool),
    Char(char),
    String(String),

    // Composes
    Array(Rc<RefCell<Vec<Value>>>),
    Tuple(Vec<Value>),
    Struct(String,HashMap<String,Value>),
    Dictionary(Rc<RefCell<HashMap<String,Value>>>),

    //fonctions
    Function(FunctionValue),
    Callable(Callable),
    BuiltinFunction(BuiltinFn),
    Closure(ClosureValue),

    // Special
    None,
    Reference(Rc<RefCell<Value>>),
    MutReference(Rc<RefCell<Value>>),
    Null,
}

#[derive(Debug,Clone)]
pub struct FunctionValue{
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<ASTNode>,
    pub env: Rc<RefCell<Environment>>,
}

#[derive(Debug,Clone)]
pub struct ClosureValue{
    pub params: Vec<String>,
    pub body: Box<Expression>,
    pub captured: HashMap<String, Value>,
}

#[derive(Debug,Clone,PartialEq)]
pub enum Callable{
    Native(fn(Vec<Value>) -> Result<Value, RuntimeError>),
}


pub type BuiltinFn = fn(&[Value]) -> Result<Value, RuntimeError>;

impl Value{
    pub fn type_name(&self) -> &str{
        match self {
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Bool(_) => "bool",
            Value::Char(_) => "char",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Tuple(_) => "tuple",
            Value::Struct(name, _) => name,
            Value::Dictionary(_) => "dictionary",
            Value::Function(f) => &f.name,
            Value::BuiltinFunction(_) => "builtin_function",
            Value::Closure(_) => "closure",
            Value::None => "none",
            Value::Reference(_) => "reference",
            Value::MutReference(_) => "mut_reference",
            _ => "unknown",
        }
    }

    pub fn is_truthy(&self) -> bool{
        match self {
            Value::Bool(b) => *b,
            Value::None => false,
            Value::Int(i) => !i.is_zero(),
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.borrow().is_empty(),
            Value::Tuple(tup) => !tup.is_empty(),
            // Value::Struct(_, fields) => !fields.is_empty(),
            Value::Dictionary(dict) => !dict.borrow().is_empty(),
            Value::Null => false,
            Value::Callable(_) => true,
            _ => true,

        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Char(c) => write!(f, "'{}'", c),
            Value::Callable(_) => write!(f, "<function>"),
            Value::None => write!(f, "none"),
            Value::Array(arr) => {
                write!(f, "[")?;
                let borrowed = arr.borrow();
                for (i, val) in borrowed.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            },
            Value::Dictionary(dict) => {
                write!(f, "{{")?;
                let borrowed = dict.borrow();
                for (i,(key,val)) in borrowed.iter().enumerate(){
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, val)?;
                }
                write!(f, "}}")
            }
            Value::Function(func) => write!(f, "<function {}>", func.name),
            _ => write!(f, "<{}>", self.type_name()),
        }
    }
}