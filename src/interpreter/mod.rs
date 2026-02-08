pub mod value;
pub mod environment;
pub mod evaluator;
pub mod builtins;
pub mod error;
pub mod interpreter;

use std::collections::HashMap;

//
// pub struct interpreter {
//     globals: HashMap<String, Value>,
//     stack: Vec<Value>,
// }
//
// impl Interpreter {
//     pob fn new() -> Self {
//         Interpreter {
//             globals: HashMap::new(),
//             stack: Vec::new(),
//         }
//     }
// }