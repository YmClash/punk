// src/semantic/optimizations/dead_code_elimination.rs

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashSet, HashMap};
use crate::parser::ast::{Expression, Statement, Declaration, ASTNode, Literal, 
                         ReturnStatement, BreakStatement, ContinueStatement};
use crate::semantic::context::CompilationContext;
use crate::semantic::flow::control_flow_graph::{ControlFlowGraph, BlockId};
use crate::semantic::symbols::SymbolId;

/// Optimiseur d'élimination du code mort
pub struct DeadCodeEliminator {
    context: Rc<RefCell<CompilationContext>>,
    
    /// Variables vivantes (utilisées)
    live_variables: HashSet<String>,
    
    /// Fonctions appelées
    called_functions: HashSet<String>,
    
    /// Blocs atteignables dans le CFG
    reachable_blocks: HashSet<BlockId>,
    
    /// Statistiques d'élimination
    pub stats: EliminationStats,
}

#[derive(Debug, Default)]
pub struct EliminationStats {
    pub dead_statements_removed: usize,
    pub unreachable_code_removed: usize,
    pub unused_variables_removed: usize,
    pub unused_functions_removed: usize,
    pub redundant_assignments_removed: usize,
    pub empty_blocks_removed: usize,
}

impl DeadCodeEliminator {
    pub fn new(context: Rc<RefCell<CompilationContext>>) -> Self {
        DeadCodeEliminator {
            context,
            live_variables: HashSet::new(),
            called_functions: HashSet::new(),
            reachable_blocks: HashSet::new(),
            stats: EliminationStats::default(),
        }
    }
    
    /// Optimise un AST complet
    pub fn optimize_ast(&mut self, ast: &mut Vec<ASTNode>) {
        // Phase 1: Analyser l'utilisation des variables et fonctions
        self.analyze_usage(ast);
        
        // Phase 2: Éliminer le code mort
        self.eliminate_dead_code(ast);
        
        // Phase 3: Nettoyer les structures vides
        self.cleanup_empty_structures(ast);
    }
    
    /// Analyse l'utilisation des variables et fonctions
    fn analyze_usage(&mut self, ast: &[ASTNode]) {
        for node in ast {
            self.analyze_node_usage(node);
        }
    }
    
    /// Analyse l'utilisation dans un nœud
    fn analyze_node_usage(&mut self, node: &ASTNode) {
        match node {
            ASTNode::Expression(expr) => {
                self.analyze_expression_usage(expr);
            }
            ASTNode::Statement(stmt) => {
                self.analyze_statement_usage(stmt);
            }
            ASTNode::Declaration(decl) => {
                self.analyze_declaration_usage(decl);
            }
            ASTNode::Program(nodes) => {
                for sub_node in nodes {
                    self.analyze_node_usage(sub_node);
                }
            }
            _ => {}
        }
    }
    
    /// Analyse l'utilisation dans une expression
    fn analyze_expression_usage(&mut self, expr: &Expression) {
        match expr {
            Expression::Identifier(name) => {
                self.live_variables.insert(name.clone());
            }
            Expression::FunctionCall(call) => {
                if let Expression::Identifier(func_name) = &*call.name {
                    self.called_functions.insert(func_name.clone());
                }
                for arg in &call.arguments {
                    self.analyze_expression_usage(arg);
                }
            }
            Expression::BinaryOperation(binop) => {
                self.analyze_expression_usage(&binop.left);
                self.analyze_expression_usage(&binop.right);
            }
            Expression::UnaryOperation(unop) => {
                self.analyze_expression_usage(&unop.operand);
            }
            Expression::Assignment(assign) => {
                // Analyser la valeur assignée
                self.analyze_expression_usage(&assign.value);
                // La cible n'est pas considérée comme "utilisée" ici
            }
            Expression::MethodCall(method) => {
                self.analyze_expression_usage(&method.object);
                for arg in &method.arguments {
                    self.analyze_expression_usage(arg);
                }
            }
            Expression::MemberAccess(access) => {
                self.analyze_expression_usage(&access.object);
            }
            Expression::IndexAccess(access) => {
                self.analyze_expression_usage(&access.array);
                self.analyze_expression_usage(&access.index);
            }
            Expression::Array(array) => {
                for elem in &array.elements {
                    self.analyze_expression_usage(elem);
                }
            }
            // Tuple n'existe pas dans notre AST, utilisons Array à la place
            // car c'est géré de manière similaire
            Expression::TypeCast(cast) => {
                self.analyze_expression_usage(&cast.expression);
            }
            _ => {}
        }
    }
    
    /// Analyse l'utilisation dans un statement
    fn analyze_statement_usage(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Expression(expr) => {
                self.analyze_expression_usage(expr);
            }
            Statement::ReturnStatement(ret_stmt) => {
                if let Some(ref value) = ret_stmt.value {
                    self.analyze_expression_usage(value);
                }
            }
            Statement::IfStatement(if_stmt) => {
                self.analyze_expression_usage(&if_stmt.condition);
                for node in &if_stmt.then_block {
                    self.analyze_node_usage(node);
                }
                for elif in &if_stmt.elif_block {
                    self.analyze_expression_usage(&elif.condition);
                    for node in &elif.block {
                        self.analyze_node_usage(node);
                    }
                }
                if let Some(ref else_block) = if_stmt.else_block {
                    for node in else_block {
                        self.analyze_node_usage(node);
                    }
                }
            }
            Statement::WhileStatement(while_stmt) => {
                self.analyze_expression_usage(&while_stmt.condition);
                for node in &while_stmt.body {
                    self.analyze_node_usage(node);
                }
            }
            Statement::ForStatement(for_stmt) => {
                self.analyze_expression_usage(&for_stmt.iterable);
                for node in &for_stmt.body {
                    self.analyze_node_usage(node);
                }
            }
            Statement::MatchStatement(match_stmt) => {
                self.analyze_expression_usage(&match_stmt.expression);
                for arm in &match_stmt.arms {
                    if let Some(ref guard) = arm.guard {
                        self.analyze_expression_usage(guard);
                    }
                    for node in &arm.body {
                        self.analyze_node_usage(node);
                    }
                }
            }
            _ => {}
        }
    }
    
    /// Analyse l'utilisation dans une déclaration
    fn analyze_declaration_usage(&mut self, decl: &Declaration) {
        match decl {
            Declaration::Variable(var_decl) => {
                if let Some(ref value) = var_decl.value {
                    self.analyze_expression_usage(value);
                }
            }
            Declaration::Function(func_decl) => {
                // Analyser le corps de la fonction
                for node in &func_decl.body {
                    self.analyze_node_usage(node);
                }
            }
            Declaration::Constante(const_decl) => {
                self.analyze_expression_usage(&const_decl.value);
            }
            _ => {}
        }
    }
    
    /// Élimine le code mort
    fn eliminate_dead_code(&mut self, ast: &mut Vec<ASTNode>) {
        let mut i = 0;
        while i < ast.len() {
            if self.should_remove_node(&ast[i]) {
                ast.remove(i);
                self.stats.dead_statements_removed += 1;
            } else {
                self.optimize_node(&mut ast[i]);
                i += 1;
            }
        }
    }
    
    /// Détermine si un nœud doit être supprimé
    fn should_remove_node(&self, node: &ASTNode) -> bool {
        match node {
            ASTNode::Declaration(Declaration::Variable(var_decl)) => {
                // Supprimer les variables non utilisées (sauf si elles ont des effets de bord)
                if !self.live_variables.contains(&var_decl.name) {
                    if let Some(ref value) = var_decl.value {
                        // Conserver si l'initialisation a des effets de bord
                        !self.has_side_effects(value)
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
            ASTNode::Declaration(Declaration::Function(func_decl)) => {
                // Supprimer les fonctions non appelées (sauf main et les fonctions publiques)
                !self.called_functions.contains(&func_decl.name) 
                    && func_decl.name != "main"
                    && !matches!(func_decl.visibility, crate::parser::ast::Visibility::Public)
            }
            ASTNode::Statement(Statement::Expression(expr)) => {
                // Supprimer les expressions sans effet de bord
                !self.has_side_effects(expr)
            }
            _ => false,
        }
    }
    
    /// Vérifie si une expression a des effets de bord
    fn has_side_effects(&self, expr: &Expression) -> bool {
        match expr {
            Expression::FunctionCall(_) => true, // Les appels de fonction peuvent avoir des effets
            Expression::MethodCall(_) => true,
            Expression::Assignment(_) => true,
            Expression::BinaryOperation(binop) => {
                self.has_side_effects(&binop.left) || self.has_side_effects(&binop.right)
            }
            Expression::UnaryOperation(unop) => {
                self.has_side_effects(&unop.operand)
            }
            _ => false,
        }
    }
    
    /// Optimise un nœud récursivement
    fn optimize_node(&mut self, node: &mut ASTNode) {
        match node {
            ASTNode::Statement(stmt) => {
                self.optimize_statement(stmt);
            }
            ASTNode::Declaration(decl) => {
                self.optimize_declaration(decl);
            }
            ASTNode::Program(nodes) => {
                self.eliminate_dead_code(nodes);
            }
            _ => {}
        }
    }
    
    /// Optimise un statement
    fn optimize_statement(&mut self, stmt: &mut Statement) {
        match stmt {
            Statement::IfStatement(if_stmt) => {
                self.optimize_if_statement(if_stmt);
            }
            Statement::WhileStatement(while_stmt) => {
                self.optimize_while_statement(while_stmt);
            }
            Statement::ForStatement(for_stmt) => {
                self.optimize_for_statement(for_stmt);
            }
            _ => {}
        }
    }
    
    /// Optimise un if statement
    fn optimize_if_statement(&mut self, if_stmt: &mut crate::parser::ast::IfStatement) {
        // Si la condition est constante, simplifier
        if let Expression::Literal(Literal::Boolean(value)) = &if_stmt.condition {
            if *value {
                // Condition toujours vraie - garder seulement le then
                self.stats.unreachable_code_removed += if_stmt.elif_block.len();
                if if_stmt.else_block.is_some() {
                    self.stats.unreachable_code_removed += 1;
                }
            } else {
                // Condition toujours fausse - supprimer le then
                self.stats.unreachable_code_removed += 1;
            }
        }
        
        // Optimiser les branches
        self.eliminate_dead_code(&mut if_stmt.then_block);
        for elif in &mut if_stmt.elif_block {
            self.eliminate_dead_code(&mut elif.block);
        }
        if let Some(ref mut else_block) = if_stmt.else_block {
            self.eliminate_dead_code(else_block);
        }
    }
    
    /// Optimise un while statement
    fn optimize_while_statement(&mut self, while_stmt: &mut crate::parser::ast::WhileStatement) {
        // Si la condition est constante false, le while est mort
        if let Expression::Literal(Literal::Boolean(false)) = &while_stmt.condition {
            self.stats.unreachable_code_removed += 1;
            while_stmt.body.clear();
        } else {
            self.eliminate_dead_code(&mut while_stmt.body);
        }
    }
    
    /// Optimise un for statement
    fn optimize_for_statement(&mut self, for_stmt: &mut crate::parser::ast::ForStatement) {
        self.eliminate_dead_code(&mut for_stmt.body);
    }
    
    /// Optimise une déclaration
    fn optimize_declaration(&mut self, decl: &mut Declaration) {
        match decl {
            Declaration::Function(func_decl) => {
                // Optimiser le corps de la fonction
                self.eliminate_dead_code(&mut func_decl.body);
                
                // Détecter le code après return
                self.remove_code_after_return(&mut func_decl.body);
            }
            _ => {}
        }
    }
    
    /// Supprime le code après un return
    fn remove_code_after_return(&mut self, nodes: &mut Vec<ASTNode>) {
        let mut return_index = None;
        
        for (i, node) in nodes.iter().enumerate() {
            if self.is_return_statement(node) {
                return_index = Some(i);
                break;
            }
        }
        
        if let Some(idx) = return_index {
            let removed = nodes.len() - idx - 1;
            if removed > 0 {
                nodes.truncate(idx + 1);
                self.stats.unreachable_code_removed += removed;
            }
        }
    }
    
    /// Vérifie si un nœud est un return
    fn is_return_statement(&self, node: &ASTNode) -> bool {
        matches!(node, ASTNode::Statement(Statement::ReturnStatement(_)))
    }
    
    /// Nettoie les structures vides
    fn cleanup_empty_structures(&mut self, ast: &mut Vec<ASTNode>) {
        ast.retain(|node| {
            match node {
                ASTNode::Statement(Statement::IfStatement(if_stmt)) => {
                    // Garder seulement si au moins une branche n'est pas vide
                    !if_stmt.then_block.is_empty() 
                        || !if_stmt.elif_block.is_empty()
                        || if_stmt.else_block.as_ref().map_or(false, |b| !b.is_empty())
                }
                ASTNode::Statement(Statement::WhileStatement(while_stmt)) => {
                    !while_stmt.body.is_empty()
                }
                ASTNode::Statement(Statement::ForStatement(for_stmt)) => {
                    !for_stmt.body.is_empty()
                }
                _ => true,
            }
        });
    }
    
    /// Analyse avec un CFG pour une élimination plus précise
    pub fn optimize_with_cfg(&mut self, cfg: &ControlFlowGraph) {
        // Trouver les blocs atteignables
        self.find_reachable_blocks(cfg);
        
        // Marquer les blocs non atteignables pour suppression
        for (block_id, _) in &cfg.blocks {
            if !self.reachable_blocks.contains(block_id) {
                self.stats.unreachable_code_removed += 1;
            }
        }
    }
    
    /// Trouve les blocs atteignables dans le CFG
    fn find_reachable_blocks(&mut self, cfg: &ControlFlowGraph) {
        let mut to_visit = vec![cfg.entry];
        self.reachable_blocks.insert(cfg.entry);
        
        while let Some(block_id) = to_visit.pop() {
            if let Some(block) = cfg.blocks.get(&block_id) {
                for &successor in &block.successors {
                    if self.reachable_blocks.insert(successor) {
                        to_visit.push(successor);
                    }
                }
            }
        }
    }
    
    /// Affiche les statistiques d'élimination
    pub fn display_stats(&self) {
        println!("=== Dead Code Elimination Statistics ===");
        println!("Dead statements removed: {}", self.stats.dead_statements_removed);
        println!("Unreachable code removed: {}", self.stats.unreachable_code_removed);
        println!("Unused variables removed: {}", self.stats.unused_variables_removed);
        println!("Unused functions removed: {}", self.stats.unused_functions_removed);
        println!("Redundant assignments removed: {}", self.stats.redundant_assignments_removed);
        println!("Empty blocks removed: {}", self.stats.empty_blocks_removed);
    }
}