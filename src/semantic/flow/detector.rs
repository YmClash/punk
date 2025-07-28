// struct DeadCodeDetector {
//     cfg: ControlFlowGraph,
//
//     // Nœuds accessibles depuis l'entrée
//     reachable: HashSet<NodeId>,
// }
//
// impl DeadCodeDetector {
//     // Détecte les nœuds inaccessibles (code mort)
//     pub fn detect_dead_code(&mut self) -> Vec<NodeId> {
//         // Calculer les nœuds accessibles
//         self.compute_reachable_nodes();
//
//         // Trouver les nœuds inaccessibles
//         self.cfg.nodes.keys()
//             .filter(|node_id| !self.reachable.contains(node_id))
//             .cloned()
//             .collect()
//     }
//
//     // Calcule les nœuds accessibles depuis l'entrée
//     fn compute_reachable_nodes(&mut self) {
//         self.reachable.clear();
//
//         // Partir du nœud d'entrée
//         let mut to_visit = vec![self.cfg.entry];
//         self.reachable.insert(self.cfg.entry);
//
//         // Parcourir le graphe
//         while let Some(node_id) = to_visit.pop() {
//             if let Some(successors) = self.cfg.edges.get(&node_id) {
//                 for succ in successors {
//                     if !self.reachable.contains(succ) {
//                         self.reachable.insert(*succ);
//                         to_visit.push(*succ);
//                     }
//                 }
//             }
//         }
//     }
//
//     // Détecte les expressions dont la valeur n'est jamais utilisée
//     pub fn detect_unused_expressions(&self) -> Vec<NodeId> {
//         let mut unused = Vec::new();
//
//         for (node_id, node) in &self.cfg.nodes {
//             // On ne s'intéresse qu'aux expressions qui sont reachable
//             if !self.reachable.contains(node_id) {
//                 continue;
//             }
//
//             match node {
//                 CFGNode::Expression(expr) => {
//                     // Vérifier si cette expression a un effet de bord
//                     if !self.has_side_effect(expr) {
//                         unused.push(*node_id);
//                     }
//                 },
//                 // Autres cas...
//                 _ => {},
//             }
//         }
//
//         unused
//     }
//
//     // Vérifie si une expression a un effet de bord
//     fn has_side_effect(&self, expr: &Expr) -> bool {
//         match expr {
//             Expr::Call(..) => true,  // Les appels de fonction peuvent avoir des effets de bord
//             Expr::Assign(..) => true,  // Les affectations ont des effets de bord
//             Expr::UnaryOp(op, operand) => {
//                 // Les opérateurs ++ et -- ont des effets de bord
//                 matches!(op, UnaryOp::PreIncrement | UnaryOp::PostIncrement |
//                              UnaryOp::PreDecrement | UnaryOp::PostDecrement) ||
//                     self.has_side_effect(operand)
//             },
//             Expr::BinaryOp(_, left, right) => {
//                 self.has_side_effect(left) || self.has_side_effect(right)
//             },
//             // Autres cas...
//             _ => false,
//         }
//     }
// }
//
//
// struct FlowAnalyzer {
//     cfg_builder: CFGBuilder,
//     def_use_analyzer: DefUseAnalyzer,
//     execution_path_analyzer: ExecutionPathAnalyzer,
//     dead_code_detector: DeadCodeDetector,
//     usage_analyzer: UsageAnalyzer,
//     return_path_analyzer: ReturnPathAnalyzer,
//
//     // Résultats de l'analyse
//     uninitialized_vars: Vec<(SymbolId, NodeId)>,
//     unused_vars: Vec<SymbolId>,
//     dead_code: Vec<NodeId>,
//     missing_returns: bool,
// }
//
// impl FlowAnalyzer {
//     // Analyse complète d'une fonction
//     pub fn analyze_function(&mut self, func: &Function) -> Result<FlowAnalysisResult, FlowError> {
//         // 1. Construire le CFG
//         let cfg = self.cfg_builder.build_cfg(&func.body)?;
//
//         // 2. Initialiser les différents analyseurs avec le CFG
//         self.initialize_analyzers(cfg, &func.symbol_table);
//
//         // 3. Analyse des définitions-utilisations
//         self.def_use_analyzer.analyze_reaching_definitions();
//         self.uninitialized_vars = self.def_use_analyzer.check_uninitialized_variables();
//
//         // 4. Analyse des variables non utilisées
//         self.unused_vars = self.usage_analyzer.analyze();
//
//         // 5. Détection de code mort
//         self.dead_code = self.dead_code_detector.detect_dead_code();
//
//         // 6. Vérification des chemins de retour
//         self.missing_returns = !self.return_path_analyzer.all_paths_return();
//
//         // 7. Construire le résultat
//         Ok(FlowAnalysisResult {
//             uninitialized_vars: self.uninitialized_vars.clone(),
//             unused_vars: self.unused_vars.clone(),
//             dead_code: self.dead_code.clone(),
//             missing_returns: self.missing_returns,
//         })
//     }
//
//     // Initialise les analyseurs avec le CFG et la table des symboles
//     fn initialize_analyzers(&mut self, cfg: ControlFlowGraph, symbol_table: &SymbolTable) {
//         // Initialisation de chaque analyseur...
//     }
// }
//
// struct FlowAnalysisResult {
//     uninitialized_vars: Vec<(SymbolId, NodeId)>,
//     unused_vars: Vec<SymbolId>,
//     dead_code: Vec<NodeId>,
//     missing_returns: bool,
// }
//
//
//
//
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_cfg_construction() {
//         // Créer un bloc simple: x = 1; y = x + 2; return y;
//         let block = create_test_block();
//
//         let mut builder = CFGBuilder::new();
//         let cfg = builder.build_cfg(&block).unwrap();
//
//         // Vérifier la structure du CFG
//         assert_eq!(cfg.nodes.len(), 5);  // entry, 3 instructions, exit
//
//         // Vérifier les connexions
//         assert!(cfg.edges.get(&cfg.entry).unwrap().len() == 1);
//
//         // Autres assertions...
//     }
//
//     #[test]
//     fn test_uninitialized_vars() {
//         // Créer un bloc avec une utilisation avant définition
//         let block = create_uninitialized_block();
//
//         let mut analyzer = FlowAnalyzer::new();
//         let result = analyzer.analyze_function(&Function {
//             name: "test".to_string(),
//             body: block,
//             // Autres champs...
//         }).unwrap();
//
//         assert!(!result.uninitialized_vars.is_empty());
//     }
//
//     #[test]
//     fn test_unused_vars() {
//         // Créer un bloc avec une variable non utilisée
//         let block = create_unused_var_block();
//
//         let mut analyzer = FlowAnalyzer::new();
//         let result = analyzer.analyze_function(&Function {
//             name: "test".to_string(),
//             body: block,
//             // Autres champs...
//         }).unwrap();
//
//         assert!(!result.unused_vars.is_empty());
//     }
//
//     #[test]
//     fn test_dead_code() {
//         // Créer un bloc avec du code mort
//         let block = create_dead_code_block();
//
//         let mut analyzer = FlowAnalyzer::new();
//         let result = analyzer.analyze_function(&Function {
//             name: "test".to_string(),
//             body: block,
//             // Autres champs...
//         }).unwrap();
//
//         assert!(!result.dead_code.is_empty());
//     }
//
//     #[test]
//     fn test_return_paths() {
//         // Créer un bloc avec des chemins sans return
//         let block = create_missing_return_block();
//
//         let mut analyzer = FlowAnalyzer::new();
//         let result = analyzer.analyze_function(&Function {
//             name: "test".to_string(),
//             body: block,
//             // Autres champs...
//         }).unwrap();
//
//         assert!(result.missing_returns);
//     }
// }