// src/interpreter/builtins.rs

use super::value::{Value, BuiltinFn};
use super::environment::Environment;
use super::error::RuntimeError;

pub fn register_builtins(env: &mut Environment) {
    // Fonctions I/O
    env.define("print".to_string(), Value::BuiltinFunction(builtin_print));
    env.define("println".to_string(), Value::BuiltinFunction(builtin_println));
    // env.define("input".to_string(), Value::BuiltinFunction(builtin_input));
    //
    // // Fonctions de conversion
    // env.define("int".to_string(), Value::BuiltinFunction(builtin_int));
    // env.define("float".to_string(), Value::BuiltinFunction(builtin_float));
    // env.define("str".to_string(), Value::BuiltinFunction(builtin_str));
    // env.define("bool".to_string(), Value::BuiltinFunction(builtin_bool));
    //
    // // Fonctions sur les arrays
    // env.define("len".to_string(), Value::BuiltinFunction(builtin_len));
    // env.define("push".to_string(), Value::BuiltinFunction(builtin_push));
    // env.define("pop".to_string(), Value::BuiltinFunction(builtin_pop));
    //
    // // Fonctions mathématiques
    // env.define("abs".to_string(), Value::BuiltinFunction(builtin_abs));
    // env.define("min".to_string(), Value::BuiltinFunction(builtin_min));
    // env.define("max".to_string(), Value::BuiltinFunction(builtin_max));
}

// Helper  pour afficher les valeurs
fn print_values(args:&[Value]) {
    for (i, arg) in args.iter().enumerate(){
        if i > 0 {
            print!(" ");
        }
        match arg {
            // cas spécial pour les chaînes de caractères sans les guillemets
            Value::String(s) => print!("{}", s),
            // sinon afficher normalement
            _ => print!("{}", arg),
        }
    }
}

fn builtin_print(args: &[Value]) -> Result<Value, RuntimeError> {
    print_values(args);
    Ok(Value::None)
}

// fn builtin_println(args: &[Value]) -> Result<Value, RuntimeError> {
//     // builtin_print(args)?;
//     print_values(args);
//     println!();
//     Ok(Value::None)
// }

fn builtin_println(args: &[Value]) -> Result<Value, RuntimeError> {
    print_values(args);
    println!(); // The only difference: add a newline at the end
    Ok(Value::None)
}


// fn builtin_input(args: &[Value]) -> Result<Value, RuntimeError> {
//     use std::io::{self, Write};
//
//     if !args.is_empty() {
//         print!("{}", args[0]);
//         io::stdout().flush().ok();
//     }
//
//     let mut buffer = String::new();
//     io::stdin().read_line(&mut buffer)
//         .map_err(|e| RuntimeError::IOError(e.to_string()))?;
//
//     Ok(Value::String(buffer.trim_end().to_string()))
// }
//
// fn builtin_len(args: &[Value]) -> Result<Value, RuntimeError> {
//     if args.len() != 1 {
//         return Err(RuntimeError::ArityMismatch { expected: 1, got: args.len() });
//     }
//
//     match &args[0] {
//         Value::Array(arr) => Ok(Value::Int(arr.borrow().len() as i64)),
//         Value::String(s) => Ok(Value::Int(s.len() as i64)),
//         _ => Err(RuntimeError::TypeError(
//             format!("len() not supported for {}", args[0].type_name())
//         )),
//     }
// }
//
// fn builtin_push(args: &[Value]) -> Result<Value, RuntimeError> {
//     if args.len() != 2 {
//         return Err(RuntimeError::ArityMismatch { expected: 2, got: args.len() });
//     }
//
//     match &args[0] {
//         Value::Array(arr) => {
//             arr.borrow_mut().push(args[1].clone());
//             Ok(Value::None)
//         },
//         _ => Err(RuntimeError::TypeError(
//             format!("push() requires an array, got {}", args[0].type_name())
//         )),
//     }
// }
//
// fn builtin_pop(args: &[Value]) -> Result<Value, RuntimeError> {
//     if args.len() != 1 {
//         return Err(RuntimeError::ArityMismatch { expected: 1, got: args.len() });
//     }
//
//     match &args[0] {
//         Value::Array(arr) => {
//             arr.borrow_mut().pop()
//                 .ok_or(RuntimeError::EmptyArray)
//         },
//         _ => Err(RuntimeError::TypeError(
//             format!("pop() requires an array, got {}", args[0].type_name())
//         )),
//     }
// }
//
// fn builtin_int(args: &[Value]) -> Result<Value, RuntimeError> {
//     if args.len() != 1 {
//         return Err(RuntimeError::ArityMismatch { expected: 1, got: args.len() });
//     }
//
//     match &args[0] {
//         Value::Int(i) => Ok(Value::Int(*i)),
//         Value::Float(f) => Ok(Value::Int(*f as i64)),
//         Value::String(s) => s.parse::<i64>()
//             .map(Value::Int)
//             .map_err(|_| RuntimeError::ConversionError(
//                 format!("Cannot convert '{}' to int", s)
//             )),
//         Value::Bool(b) => Ok(Value::Int(if *b { 1 } else { 0 })),
//         _ => Err(RuntimeError::TypeError(
//             format!("Cannot convert {} to int", args[0].type_name())
//         )),
//     }
// }
//
// fn builtin_float(args: &[Value]) -> Result<Value, RuntimeError> {
//     if args.len() != 1 {
//         return Err(RuntimeError::ArityMismatch { expected: 1, got: args.len() });
//     }
//
//     match &args[0] {
//         Value::Float(f) => Ok(Value::Float(*f)),
//         Value::Int(i) => Ok(Value::Float(*i as f64)),
//         Value::String(s) => s.parse::<f64>()
//             .map(Value::Float)
//             .map_err(|_| RuntimeError::ConversionError(
//                 format!("Cannot convert '{}' to float", s)
//             )),
//         _ => Err(RuntimeError::TypeError(
//             format!("Cannot convert {} to float", args[0].type_name())
//         )),
//     }
// }
//
// fn builtin_str(args: &[Value]) -> Result<Value, RuntimeError> {
//     if args.len() != 1 {
//         return Err(RuntimeError::ArityMismatch { expected: 1, got: args.len() });
//     }
//
//     Ok(Value::String(format!("{}", args[0])))
// }
//
// fn builtin_bool(args: &[Value]) -> Result<Value, RuntimeError> {
//     if args.len() != 1 {
//         return Err(RuntimeError::ArityMismatch { expected: 1, got: args.len() });
//     }
//
//     Ok(Value::Bool(args[0].is_truthy()))
// }
//
// fn builtin_abs(args: &[Value]) -> Result<Value, RuntimeError> {
//     if args.len() != 1 {
//         return Err(RuntimeError::ArityMismatch { expected: 1, got: args.len() });
//     }
//
//     match &args[0] {
//         Value::Int(i) => Ok(Value::Int(i.abs())),
//         Value::Float(f) => Ok(Value::Float(f.abs())),
//         _ => Err(RuntimeError::TypeError(
//             format!("abs() not supported for {}", args[0].type_name())
//         )),
//     }
// }
//
// fn builtin_min(args: &[Value]) -> Result<Value, RuntimeError> {
//     if args.is_empty() {
//         return Err(RuntimeError::ArityMismatch { expected: 1, got: 0 });
//     }
//
//     let mut min = args[0].clone();
//
//     for arg in &args[1..] {
//         match (&min, arg) {
//             (Value::Int(a), Value::Int(b)) if b < a => min = arg.clone(),
//             (Value::Float(a), Value::Float(b)) if b < a => min = arg.clone(),
//             _ => {},
//         }
//     }
//
//     Ok(min)
// }
//
// fn builtin_max(args: &[Value]) -> Result<Value, RuntimeError> {
//     if args.is_empty() {
//         return Err(RuntimeError::ArityMismatch { expected: 1, got: 0 });
//     }
//
//     let mut max = args[0].clone();
//
//     for arg in &args[1..] {
//         match (&max, arg) {
//             (Value::Int(a), Value::Int(b)) if b > a => max = arg.clone(),
//             (Value::Float(a), Value::Float(b)) if b > a => max = arg.clone(),
//             _ => {},
//         }
//     }
//
//     Ok(max)
// }