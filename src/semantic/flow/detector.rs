//src/semantic/flow/detector.rs

use std::collections::HashSet;
use crate::semantic::flow::control_flow_graph::{ControlFlowGraph, BlockId};
use crate::semantic::symbols::SymbolId;
use crate::semantic::symbol_table::SymbolTable;
use crate::parser::ast::Expression;

pub struct DeadCodeDetector {
    cfg: ControlFlowGraph,
    reachable: HashSet<BlockId>,
}

impl DeadCodeDetector {
    pub fn new(cfg: ControlFlowGraph) -> Self {
        DeadCodeDetector {
            cfg,
            reachable: HashSet::new(),
        }
    }
    
    /// Détecte les blocs inaccessibles (code mort)
    pub fn detect_dead_code(&mut self) -> Vec<BlockId> {
        // Calculer les blocs accessibles
        self.compute_reachable_blocks();
        
        // Trouver les blocs inaccessibles
        self.cfg.blocks.keys()
            .filter(|block_id| !self.reachable.contains(block_id))
            .copied()
            .collect()
    }
    
    /// Calcule les blocs accessibles depuis l'entrée
    fn compute_reachable_blocks(&mut self) {
        self.reachable.clear();
        
        let mut to_visit = vec![self.cfg.entry];
        self.reachable.insert(self.cfg.entry);
        
        while let Some(block_id) = to_visit.pop() {
            if let Some(block) = self.cfg.blocks.get(&block_id) {
                for &successor in &block.successors {
                    if !self.reachable.contains(&successor) {
                        self.reachable.insert(successor);
                        to_visit.push(successor);
                    }
                }
            }
        }
    }
    
    /// Détecte les expressions dont la valeur n'est jamais utilisée
    pub fn detect_unused_expressions(&self) -> Vec<BlockId> {
        let mut unused = Vec::new();
        
        for (block_id, block) in &self.cfg.blocks {
            // On ne s'intéresse qu'aux blocs qui sont accessibles
            if !self.reachable.contains(block_id) {
                continue;
            }
            
            // Analyser les instructions du bloc
            for instruction in &block.instructions {
                use crate::semantic::flow::control_flow_graph::Instruction;
                
                match instruction {
                    Instruction::Expression { expr, .. } => {
                        // Vérifier si cette expression a un effet de bord
                        if !self.has_side_effect(expr) {
                            unused.push(*block_id);
                        }
                    }
                    _ => {}
                }
            }
        }
        
        unused
    }
    
    /// Vérifie si une expression a un effet de bord
    fn has_side_effect(&self, expr: &Expression) -> bool {
        match expr {
            Expression::FunctionCall(_) => true, // Les appels de fonction peuvent avoir des effets de bord
            Expression::Assignment(_) => true, // Les affectations ont des effets de bord
            Expression::MethodCall(_) => true, // Les appels de méthode peuvent avoir des effets de bord
            Expression::UnaryOperation(unop) => {
                // Certains opérateurs unaires peuvent avoir des effets de bord
                self.has_side_effect(&unop.operand)
            }
            Expression::BinaryOperation(binop) => {
                self.has_side_effect(&binop.left) || self.has_side_effect(&binop.right)
            }
            _ => false,
        }
    }
}

/// Analyseur de flux complet
pub struct FlowAnalyzer {
    symbol_table: SymbolTable,
}

impl FlowAnalyzer {
    pub fn new(symbol_table: SymbolTable) -> Self {
        FlowAnalyzer { symbol_table }
    }
    
    /// Analyse complète d'une fonction
    pub fn analyze_function(&mut self, ast: &crate::parser::ast::ASTNode) -> Result<FlowAnalysisResult, String> {
        use crate::semantic::flow::control_flow_graph::CFGBuilder;
        use crate::semantic::flow::analyser::{DefUseAnalyzer, UsageAnalyzer, ReturnPathAnalyzer};
        
        // 1. Construire le CFG
        let cfg_builder = CFGBuilder::new();
        let cfg = cfg_builder.build(ast).map_err(|e| format!("CFG build error: {:?}", e))?;
        
        // 2. Analyse des définitions-utilisations
        let mut def_use_analyzer = DefUseAnalyzer::new(cfg.clone(), self.symbol_table.clone());
        def_use_analyzer.analyze_reaching_definitions();
        let uninitialized_vars = def_use_analyzer.check_uninitialized_variables();
        
        // 3. Analyse des variables non utilisées
        let mut usage_analyzer = UsageAnalyzer::new(self.symbol_table.clone(), cfg.clone());
        let unused_vars = usage_analyzer.analyze();
        
        // 4. Détection de code mort
        let mut dead_code_detector = DeadCodeDetector::new(cfg.clone());
        let dead_blocks = dead_code_detector.detect_dead_code();
        
        // 5. Vérification des chemins de retour
        let mut return_analyzer = ReturnPathAnalyzer::new(cfg);
        let all_paths_return = return_analyzer.all_paths_return();
        
        // 6. Construire le résultat
        Ok(FlowAnalysisResult {
            uninitialized_vars,
            unused_vars,
            dead_blocks,
            all_paths_return,
        })
    }
}

/// Résultat de l'analyse de flux
#[derive(Debug, Clone)]
pub struct FlowAnalysisResult {
    pub uninitialized_vars: Vec<(SymbolId, BlockId)>,
    pub unused_vars: Vec<SymbolId>,
    pub dead_blocks: Vec<BlockId>,
    pub all_paths_return: bool,
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use super::*;

    #[test]
    fn test_dead_code_detection() {
        // Test basique de détection de code mort
        let cfg = ControlFlowGraph::new();
        let mut detector = DeadCodeDetector::new(cfg);
        let dead_blocks = detector.detect_dead_code();
        assert!(dead_blocks.is_empty()); // Avec un CFG vide, pas de code mort
    }

    #[test]
    fn test_side_effect_detection() {
        use crate::parser::ast::{FunctionCall, Literal};

        let detector = DeadCodeDetector::new(ControlFlowGraph::new());

        // Un appel de fonction a des effets de bord
        let func_call = Expression::FunctionCall(FunctionCall {
            // name: "print".to_string(),
            name: Box::new(Expression::Literal(Literal::String { 0: "print".to_string() })),
            arguments: vec![],
        });
        assert!(detector.has_side_effect(&func_call));

        // Un littéral n'a pas d'effet de bord
        let literal = Expression::Literal(Literal::Integer { value: BigInt::from(42) });
        assert!(!detector.has_side_effect(&literal));
    }
}