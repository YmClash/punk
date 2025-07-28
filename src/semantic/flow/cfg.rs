// use std::collections::{HashMap, HashSet};
// use crate::parser::ast::{Declaration, Expression, FunctionCall, Statement};
//
// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// pub struct NodeId(pub u32);
//
// pub struct ControlFlowGraph {
//     // Nœuds du graphe
//     nodes: HashMap<NodeId, CFGNode>,
//
//     // Arêtes du graphe (source -> [destinations])
//     edges: HashMap<NodeId, Vec<NodeId>>,
//
//     // Arêtes inverses (destination -> [sources])
//     reverse_edges: HashMap<NodeId, Vec<NodeId>>,
//
//     // Point d'entrée du graphe
//     entry: NodeId,
//
//     // Points de sortie du graphe
//     exits: HashSet<NodeId>,
//
//     // Générateur d'ID de nœud
//     next_node_id: u32,
// }
//
// pub enum CFGNode {
//     // Expression (sans effet de bord)
//     Expression(Expression),
//
//     // Déclaration de variable
//     Declaration(Declaration),
//
//     // Affectation
//     Assignment(Expression, Expression),
//
//     // Condition
//     Condition(Expression),
//
//     // Appel de fonction
//     Call(FunctionCall),
//
//     // Return
//     Return(Option<Expression>),
//
//     // Break/Continue
//     Break(Option<String>),  // Label optionnel
//     Continue(Option<String>),
//
//     // Point d'entrée/sortie de bloc
//     BlockEntry,
//     BlockExit,
// }
//
// impl ControlFlowGraph {
//     // Construit un CFG à partir d'un bloc d'instructions
//     pub fn from_block(block: &Block) -> Result<Self, FlowError> {
//         let mut builder = CFGBuilder::new();
//         builder.build_cfg(block)
//     }
// }
//
// struct CFGBuilder {
//     graph: ControlFlowGraph,
//     current_loop_context: Vec<LoopContext>,
//     current_function_context: Option<FunctionContext>,
// }
//
// impl CFGBuilder {
//     // Construit un CFG pour un bloc d'instructions
//     pub fn build_cfg(&mut self, block: &Block) -> Result<ControlFlowGraph, FlowError> {
//         let entry = self.add_node(CFGNode::BlockEntry);
//         let exit = self.add_node(CFGNode::BlockExit);
//
//         self.graph.entry = entry;
//         self.graph.exits.insert(exit);
//
//         let mut current = entry;
//
//         for statement in &block.statements {
//             current = self.add_statement(current, statement)?;
//         }
//
//         self.add_edge(current, exit);
//
//         Ok(self.graph.clone())
//     }
//
//     // Ajoute un nœud pour une instruction
//     fn add_statement(&mut self, prev: NodeId, stmt: &Statement) -> Result<NodeId, FlowError> {
//         match stmt {
//             Statement::Expression(expr) => {
//                 let node = self.add_node(CFGNode::Expression(expr.clone()));
//                 self.add_edge(prev, node);
//                 Ok(node)
//             },
//             Statement::ReturnStatement(expr) => {
//                 let node = self.add_node(CFGNode::Return(expr.clone()));
//                 self.add_edge(prev, node);
//                 // Pas de nœud suivant pour un return
//                 Ok(node)
//             },
//             Statement::Assignment(target, value) => {
//                 let node = self.add_node(CFGNode::Assignment(target.clone(), value.clone()));
//                 self.add_edge(prev, node);
//                 Ok(node)
//             },
//             Statement::If(cond, then_block, else_block) => {
//                 self.add_if_statement(prev, cond, then_block, else_block)
//             },
//             Statement::While(cond, body) => {
//                 self.add_while_statement(prev, cond, body)
//             },
//             Statement::Return(expr) => {
//                 let node = self.add_node(CFGNode::Return(expr.clone()));
//                 self.add_edge(prev, node);
//                 // Pas de nœud suivant pour un return
//                 Ok(node)
//             },
//             // Autres cas...
//         }
//     }
//
//     // Ajoute un nœud pour une instruction if-else
//     fn add_if_statement(&mut self, prev: NodeId, cond: &Expr, then_block: &Block, else_block: &Option<Block>) -> Result<NodeId, FlowError> {
//         let cond_node = self.add_node(CFGNode::Condition(cond.clone()));
//         self.add_edge(prev, cond_node);
//
//         // Point d'entrée du bloc then
//         let then_entry = self.add_node(CFGNode::BlockEntry);
//         self.add_edge(cond_node, then_entry);
//
//         // Construction du CFG pour le bloc then
//         let then_exit = self.build_block_cfg(then_entry, then_block)?;
//
//         // Point de jonction après le if-else
//         let join = self.add_node(CFGNode::BlockExit);
//         self.add_edge(then_exit, join);
//
//         if let Some(else_block) = else_block {
//             // Point d'entrée du bloc else
//             let else_entry = self.add_node(CFGNode::BlockEntry);
//             self.add_edge(cond_node, else_entry);
//
//             // Construction du CFG pour le bloc else
//             let else_exit = self.build_block_cfg(else_entry, else_block)?;
//             self.add_edge(else_exit, join);
//         } else {
//             // Si pas de bloc else, on va directement à la jonction
//             self.add_edge(cond_node, join);
//         }
//
//         Ok(join)
//     }
//
//     // Autres méthodes pour les boucles, etc.
// }