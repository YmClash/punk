// use std::collections::HashMap;
// use std::thread::Scope;
// use crate::semantic::semantic_error::SemanticError;
//
// #[allow(dead_code)]
// pub trait AstVisitor{
//     // fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> Result<Type, SemanticError>;
//     // fn visit_dict_literal(&mut self, dict: &DictLiteral) -> Result<Type, SemanticError>;
//     // fn visit_method_call(&mut self, call: &MethodCall) -> Result<Type, SemanticError>;
//     todo!();
// }
//
//
//
//
// #[allow(dead_code)]
// #[derive(Debug, PartialEq, Clone)]
// pub struct SymbolTable {
//     scopes: Vec<HashMap<String,Symbol>>,
// }
//
// #[allow(dead_code)]
// #[derive(Debug, PartialEq, Clone)]
// pub struct Symbol {
//     name: String,
//     symbol_type: SymbolType,
//     is_mutable: bool,
//     // La position du symbole dans le code source
//     location: Position,
// }
//
//
//
//
// #[allow(dead_code)]
// #[derive(Debug, PartialEq, Clone)]
// pub struct SemanticAnalyzer {
//     symbol_table: SymbolTable,
//     current_scope: Scope,
//
//     // Le contexte de typage
//     type_context: TypeContext,
//     // Le contexte de portée
//     scope_context: ScopeContext,
//     // Le contexte des emprunts
//     borrow_checker: BorrowChecker,
// }
//
//
//
// impl SemanticAnalyzer {
//     pub fn analyze(&mut self, expr: &Expression) -> Result<Type, SemanticError> {
//         match expr {
//             Expression::DictLiteral(dict) => self.analyze_dict_literal(dict),
//             Expression::MethodCall(call) => self.analyze_method_call(call),
//             Expression::BinaryExpr(binary) => self.analyze_binary_expr(binary),
//             // ...
//         }
//     }
//
// }
//
//
//
//
//
//
//
