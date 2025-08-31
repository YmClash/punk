// src/semantic/optimizations/alias_analysis.rs

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use crate::parser::ast::{Expression, Statement, Declaration, ASTNode};
use crate::semantic::context::CompilationContext;
use crate::semantic::symbols::SymbolId;
use crate::semantic::flow::control_flow_graph::BlockId;

/// Représente une location mémoire abstraite
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MemoryLocation {
    /// Variable locale
    Variable(String),
    /// Champ d'une structure
    Field(Box<MemoryLocation>, String),
    /// Élément de tableau
    ArrayElement(Box<MemoryLocation>, usize),
    /// Déréférence d'un pointeur
    Dereference(Box<MemoryLocation>),
    /// Location sur le tas (heap)
    Heap(usize),
    /// Location inconnue
    Unknown,
}

/// Représente une relation d'alias entre locations
#[derive(Debug, Clone)]
pub struct AliasRelation {
    pub from: MemoryLocation,
    pub to: MemoryLocation,
    pub is_must_alias: bool,  // true = must-alias, false = may-alias
}

/// Points-to set pour l'analyse d'alias
#[derive(Debug, Clone)]
pub struct PointsToSet {
    /// Ensemble des locations possibles
    pub locations: HashSet<MemoryLocation>,
    /// Indique si c'est un ensemble complet ou partiel
    pub is_complete: bool,
}

/// Analyseur d'alias pour détecter les références multiples
pub struct AliasAnalyzer {
    context: Rc<RefCell<CompilationContext>>,
    
    /// Points-to sets pour chaque expression
    points_to_sets: HashMap<String, PointsToSet>,
    
    /// Relations d'alias détectées
    alias_relations: Vec<AliasRelation>,
    
    /// Graphe d'alias : location -> ensemble des alias
    alias_graph: HashMap<MemoryLocation, HashSet<MemoryLocation>>,
    
    /// Locations modifiées dans chaque bloc
    modified_locations: HashMap<BlockId, HashSet<MemoryLocation>>,
    
    /// Compteur pour les allocations heap
    heap_counter: usize,
    
    /// Statistiques d'analyse
    pub stats: AliasStats,
}

#[derive(Debug, Default)]
pub struct AliasStats {
    pub total_pointers: usize,
    pub must_aliases: usize,
    pub may_aliases: usize,
    pub no_aliases: usize,
    pub heap_allocations: usize,
    pub escape_analysis_optimizations: usize,
}

impl AliasAnalyzer {
    pub fn new(context: Rc<RefCell<CompilationContext>>) -> Self {
        AliasAnalyzer {
            context,
            points_to_sets: HashMap::new(),
            alias_relations: Vec::new(),
            alias_graph: HashMap::new(),
            modified_locations: HashMap::new(),
            heap_counter: 0,
            stats: AliasStats::default(),
        }
    }
    
    /// Analyse l'AST pour détecter les alias
    pub fn analyze_ast(&mut self, ast: &[ASTNode]) {
        // Phase 1: Construire les points-to sets
        self.build_points_to_sets(ast);
        
        // Phase 2: Détecter les relations d'alias
        self.detect_aliases();
        
        // Phase 3: Construire le graphe d'alias
        self.build_alias_graph();
        
        // Phase 4: Analyse d'échappement
        self.escape_analysis(ast);
    }
    
    /// Construit les points-to sets pour chaque expression
    fn build_points_to_sets(&mut self, ast: &[ASTNode]) {
        for node in ast {
            self.analyze_node_points_to(node);
        }
    }
    
    /// Analyse les points-to pour un nœud
    fn analyze_node_points_to(&mut self, node: &ASTNode) {
        match node {
            ASTNode::Declaration(decl) => {
                self.analyze_declaration_points_to(decl);
            }
            ASTNode::Statement(stmt) => {
                self.analyze_statement_points_to(stmt);
            }
            ASTNode::Expression(expr) => {
                self.analyze_expression_points_to(expr);
            }
            ASTNode::Program(nodes) => {
                for sub_node in nodes {
                    self.analyze_node_points_to(sub_node);
                }
            }
            _ => {}
        }
    }
    
    /// Analyse les points-to pour une déclaration
    fn analyze_declaration_points_to(&mut self, decl: &Declaration) {
        match decl {
            Declaration::Variable(var_decl) => {
                if let Some(ref value) = var_decl.value {
                    let points_to = self.compute_points_to(value);
                    self.points_to_sets.insert(var_decl.name.clone(), points_to);
                    
                    // Si c'est une référence, incrémenter le compteur
                    if self.is_reference(value) {
                        self.stats.total_pointers += 1;
                    }
                }
            }
            Declaration::Function(func_decl) => {
                // Analyser le corps de la fonction
                for node in &func_decl.body {
                    self.analyze_node_points_to(node);
                }
            }
            _ => {}
        }
    }
    
    /// Analyse les points-to pour un statement
    fn analyze_statement_points_to(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Expression(expr) => {
                self.analyze_expression_points_to(expr);
            }
            Statement::Assignment(target, value) => {
                if let Expression::Identifier(name) = target {
                    let points_to = self.compute_points_to(value);
                    self.points_to_sets.insert(name.clone(), points_to);
                }
            }
            Statement::IfStatement(if_stmt) => {
                for node in &if_stmt.then_block {
                    self.analyze_node_points_to(node);
                }
                for elif in &if_stmt.elif_block {
                    for node in &elif.block {
                        self.analyze_node_points_to(node);
                    }
                }
                if let Some(ref else_block) = if_stmt.else_block {
                    for node in else_block {
                        self.analyze_node_points_to(node);
                    }
                }
            }
            _ => {}
        }
    }
    
    /// Analyse les points-to pour une expression
    fn analyze_expression_points_to(&mut self, expr: &Expression) {
        match expr {
            Expression::Borrow(borrow) => {
                // Une référence pointe vers l'expression empruntée
                self.stats.total_pointers += 1;
            }
            Expression::Assignment(assign) => {
                if let Expression::Identifier(name) = &*assign.target {
                    let points_to = self.compute_points_to(&assign.value);
                    self.points_to_sets.insert(name.clone(), points_to);
                }
            }
            _ => {}
        }
    }
    
    /// Calcule le points-to set pour une expression
    fn compute_points_to(&mut self, expr: &Expression) -> PointsToSet {
        match expr {
            Expression::Identifier(name) => {
                // Pointe vers la variable elle-même
                PointsToSet {
                    locations: vec![MemoryLocation::Variable(name.clone())].into_iter().collect(),
                    is_complete: true,
                }
            }
            Expression::Borrow(borrow) => {
                // Pointe vers l'expression empruntée
                if let Expression::Identifier(name) = &*borrow.borrowed_value {
                    PointsToSet {
                        locations: vec![MemoryLocation::Variable(name.clone())].into_iter().collect(),
                        is_complete: true,
                    }
                } else {
                    PointsToSet {
                        locations: vec![MemoryLocation::Unknown].into_iter().collect(),
                        is_complete: false,
                    }
                }
            }
            Expression::MemberAccess(access) => {
                // Pointe vers le champ de l'objet
                let base_points_to = self.compute_points_to(&access.object);
                let mut locations = HashSet::new();
                
                for base_loc in &base_points_to.locations {
                    locations.insert(MemoryLocation::Field(
                        Box::new(base_loc.clone()),
                        access.member.clone()
                    ));
                }
                
                PointsToSet {
                    locations,
                    is_complete: base_points_to.is_complete,
                }
            }
            Expression::FunctionCall(_) => {
                // Une allocation dynamique
                self.heap_counter += 1;
                self.stats.heap_allocations += 1;
                PointsToSet {
                    locations: vec![MemoryLocation::Heap(self.heap_counter)].into_iter().collect(),
                    is_complete: true,
                }
            }
            _ => {
                // Autres expressions : pas de points-to spécifique
                PointsToSet {
                    locations: HashSet::new(),
                    is_complete: true,
                }
            }
        }
    }
    
    /// Détecte les relations d'alias entre variables
    fn detect_aliases(&mut self) {
        let vars: Vec<String> = self.points_to_sets.keys().cloned().collect();
        
        // Comparer chaque paire de variables
        for i in 0..vars.len() {
            for j in i+1..vars.len() {
                let var1 = &vars[i];
                let var2 = &vars[j];
                
                if let (Some(pts1), Some(pts2)) = (
                    self.points_to_sets.get(var1),
                    self.points_to_sets.get(var2)
                ) {
                    // Calculer l'intersection des points-to sets
                    let intersection: HashSet<_> = pts1.locations.intersection(&pts2.locations).cloned().collect();
                    
                    if !intersection.is_empty() {
                        // Il y a un alias
                        let is_must = pts1.locations == pts2.locations && 
                                      pts1.locations.len() == 1 &&
                                      pts1.is_complete && pts2.is_complete;
                        
                        if is_must {
                            self.stats.must_aliases += 1;
                        } else {
                            self.stats.may_aliases += 1;
                        }
                        
                        self.alias_relations.push(AliasRelation {
                            from: MemoryLocation::Variable(var1.clone()),
                            to: MemoryLocation::Variable(var2.clone()),
                            is_must_alias: is_must,
                        });
                    } else {
                        self.stats.no_aliases += 1;
                    }
                }
            }
        }
    }
    
    /// Construit le graphe d'alias
    fn build_alias_graph(&mut self) {
        for relation in &self.alias_relations {
            self.alias_graph
                .entry(relation.from.clone())
                .or_insert_with(HashSet::new)
                .insert(relation.to.clone());
            
            self.alias_graph
                .entry(relation.to.clone())
                .or_insert_with(HashSet::new)
                .insert(relation.from.clone());
        }
    }
    
    /// Analyse d'échappement pour optimiser les allocations
    fn escape_analysis(&mut self, ast: &[ASTNode]) {
        // Identifier les variables qui n'échappent pas de leur scope
        let mut non_escaping = HashSet::new();
        
        for (var, pts) in &self.points_to_sets {
            let mut escapes = false;
            
            // Vérifier si la variable est retournée ou passée à une fonction
            for node in ast {
                if self.variable_escapes(var, node) {
                    escapes = true;
                    break;
                }
            }
            
            if !escapes {
                non_escaping.insert(var.clone());
                
                // Si c'est une allocation heap qui n'échappe pas, on peut l'optimiser
                for loc in &pts.locations {
                    if matches!(loc, MemoryLocation::Heap(_)) {
                        self.stats.escape_analysis_optimizations += 1;
                    }
                }
            }
        }
    }
    
    /// Vérifie si une variable échappe de son scope
    fn variable_escapes(&self, var: &str, node: &ASTNode) -> bool {
        match node {
            ASTNode::Statement(Statement::ReturnStatement(ret_stmt)) => {
                if let Some(ref value) = ret_stmt.value {
                    self.expression_contains_var(value, var)
                } else {
                    false
                }
            }
            ASTNode::Expression(Expression::FunctionCall(call)) => {
                for arg in &call.arguments {
                    if self.expression_contains_var(arg, var) {
                        return true;
                    }
                }
                false
            }
            ASTNode::Program(nodes) => {
                for sub_node in nodes {
                    if self.variable_escapes(var, sub_node) {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }
    
    /// Vérifie si une expression contient une variable
    fn expression_contains_var(&self, expr: &Expression, var: &str) -> bool {
        match expr {
            Expression::Identifier(name) => name == var,
            Expression::Borrow(borrow) => {
                self.expression_contains_var(&borrow.borrowed_value, var)
            }
            Expression::BinaryOperation(binop) => {
                self.expression_contains_var(&binop.left, var) ||
                self.expression_contains_var(&binop.right, var)
            }
            _ => false,
        }
    }
    
    /// Vérifie si une expression est une référence
    fn is_reference(&self, expr: &Expression) -> bool {
        matches!(expr, Expression::Borrow(_))
    }
    
    /// Obtient les alias d'une location mémoire
    pub fn get_aliases(&self, location: &MemoryLocation) -> Option<&HashSet<MemoryLocation>> {
        self.alias_graph.get(location)
    }
    
    /// Vérifie si deux locations peuvent être des alias
    pub fn may_alias(&self, loc1: &MemoryLocation, loc2: &MemoryLocation) -> bool {
        if loc1 == loc2 {
            return true;
        }
        
        if let Some(aliases) = self.alias_graph.get(loc1) {
            aliases.contains(loc2)
        } else {
            false
        }
    }
    
    /// Vérifie si deux locations sont nécessairement des alias
    pub fn must_alias(&self, loc1: &MemoryLocation, loc2: &MemoryLocation) -> bool {
        for relation in &self.alias_relations {
            if relation.is_must_alias {
                if (relation.from == *loc1 && relation.to == *loc2) ||
                   (relation.from == *loc2 && relation.to == *loc1) {
                    return true;
                }
            }
        }
        false
    }
    
    /// Marque une location comme modifiée dans un bloc
    pub fn mark_modified(&mut self, block_id: BlockId, location: MemoryLocation) {
        self.modified_locations
            .entry(block_id)
            .or_insert_with(HashSet::new)
            .insert(location);
    }
    
    /// Obtient les locations modifiées dans un bloc
    pub fn get_modified_in_block(&self, block_id: &BlockId) -> Option<&HashSet<MemoryLocation>> {
        self.modified_locations.get(block_id)
    }
    
    /// Affiche les statistiques d'analyse
    pub fn display_stats(&self) {
        println!("=== Alias Analysis Statistics ===");
        println!("Total pointers analyzed: {}", self.stats.total_pointers);
        println!("Must-alias pairs: {}", self.stats.must_aliases);
        println!("May-alias pairs: {}", self.stats.may_aliases);
        println!("No-alias pairs: {}", self.stats.no_aliases);
        println!("Heap allocations: {}", self.stats.heap_allocations);
        println!("Escape analysis optimizations: {}", self.stats.escape_analysis_optimizations);
    }
    
    /// Affiche le graphe d'alias
    pub fn display_alias_graph(&self) {
        println!("\n=== Alias Graph ===");
        for (loc, aliases) in &self.alias_graph {
            println!("{:?} aliases with:", loc);
            for alias in aliases {
                println!("  - {:?}", alias);
            }
        }
    }
    
    /// Affiche les relations d'alias
    pub fn display_alias_relations(&self) {
        println!("\n=== Alias Relations ===");
        for relation in &self.alias_relations {
            let kind = if relation.is_must_alias { "MUST" } else { "MAY" };
            println!("{:?} {} alias with {:?}", relation.from, kind, relation.to);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_must_alias() {
        let context = Rc::new(RefCell::new(CompilationContext::new()));
        let mut analyzer = AliasAnalyzer::new(context);
        
        // Simuler: let x = &y; let z = &y;
        analyzer.points_to_sets.insert(
            "x".to_string(),
            PointsToSet {
                locations: vec![MemoryLocation::Variable("y".to_string())].into_iter().collect(),
                is_complete: true,
            }
        );
        
        analyzer.points_to_sets.insert(
            "z".to_string(),
            PointsToSet {
                locations: vec![MemoryLocation::Variable("y".to_string())].into_iter().collect(),
                is_complete: true,
            }
        );
        
        analyzer.detect_aliases();
        analyzer.build_alias_graph();
        
        // x et z doivent être des must-alias car ils pointent tous deux vers y
        assert!(analyzer.must_alias(
            &MemoryLocation::Variable("x".to_string()),
            &MemoryLocation::Variable("z".to_string())
        ));
    }
    
    #[test]
    fn test_may_alias() {
        let context = Rc::new(RefCell::new(CompilationContext::new()));
        let mut analyzer = AliasAnalyzer::new(context);
        
        // Simuler des may-alias avec des points-to sets partiels
        analyzer.points_to_sets.insert(
            "p1".to_string(),
            PointsToSet {
                locations: vec![
                    MemoryLocation::Variable("a".to_string()),
                    MemoryLocation::Variable("b".to_string())
                ].into_iter().collect(),
                is_complete: false,
            }
        );
        
        analyzer.points_to_sets.insert(
            "p2".to_string(),
            PointsToSet {
                locations: vec![
                    MemoryLocation::Variable("b".to_string()),
                    MemoryLocation::Variable("c".to_string())
                ].into_iter().collect(),
                is_complete: false,
            }
        );
        
        analyzer.detect_aliases();
        analyzer.build_alias_graph();
        
        // p1 et p2 peuvent être des alias (intersection non vide sur b)
        assert!(analyzer.may_alias(
            &MemoryLocation::Variable("p1".to_string()),
            &MemoryLocation::Variable("p2".to_string())
        ));
    }
}