// struct UsageAnalyzer {
//     symbol_table: SymbolTable,
//     cfg: ControlFlowGraph,
//     defined_variables: HashSet<SymbolId>,
//     used_variables: HashSet<SymbolId>,
// }
//
// impl UsageAnalyzer {
//     // Analyse un CFG pour détecter les variables non utilisées
//     pub fn analyze(&mut self) -> Vec<SymbolId> {
//         self.visit_node(self.cfg.entry);
//
//         // Trouver les variables définies mais non utilisées
//         self.defined_variables
//             .difference(&self.used_variables)
//             .cloned()
//             .collect()
//     }
//
//     // Visite un nœud du CFG
//     fn visit_node(&mut self, node_id: NodeId) {
//         if let Some(node) = self.cfg.nodes.get(&node_id) {
//             self.analyze_node(node);
//
//             // Visiter les nœuds successeurs
//             if let Some(successors) = self.cfg.edges.get(&node_id) {
//                 for succ in successors {
//                     self.visit_node(*succ);
//                 }
//             }
//         }
//     }
//
//     // Analyse un nœud pour détecter les utilisations et définitions
//     fn analyze_node(&mut self, node: &CFGNode) {
//         match node {
//             CFGNode::Declaration(decl) => {
//                 // Une variable est définie
//                 let symbol_id = self.symbol_table.lookup_symbol(&decl.name).unwrap();
//                 self.defined_variables.insert(symbol_id);
//             },
//             CFGNode::Assignment(target, value) => {
//                 // La cible est définie, la valeur est utilisée
//                 self.analyze_expr_usage(target, ExprUsage::Definition);
//                 self.analyze_expr_usage(value, ExprUsage::Usage);
//             },
//             CFGNode::Expression(expr) => {
//                 // L'expression est utilisée
//                 self.analyze_expr_usage(expr, ExprUsage::Usage);
//             },
//             // Autres cas...
//         }
//     }
//
//     // Analyse l'utilisation des variables dans une expression
//     fn analyze_expr_usage(&mut self, expr: &Expr, usage: ExprUsage) {
//         match expr {
//             Expr::Variable(name) => {
//                 let symbol_id = self.symbol_table.lookup_symbol(name).unwrap();
//                 match usage {
//                     ExprUsage::Definition => {
//                         self.defined_variables.insert(symbol_id);
//                     },
//                     ExprUsage::Usage => {
//                         self.used_variables.insert(symbol_id);
//                     },
//                     ExprUsage::Both => {
//                         self.defined_variables.insert(symbol_id);
//                         self.used_variables.insert(symbol_id);
//                     },
//                 }
//             },
//             Expr::BinaryOp(_, left, right) => {
//                 // Les opérandes sont utilisées
//                 self.analyze_expr_usage(left, ExprUsage::Usage);
//                 self.analyze_expr_usage(right, ExprUsage::Usage);
//             },
//             // Autres cas...
//         }
//     }
// }
//
// enum ExprUsage {
//     Definition,  // La variable est définie
//     Usage,       // La variable est utilisée
//     Both,        // La variable est à la fois définie et utilisée
// }
//
//
// struct ReturnPathAnalyzer {
//     cfg: ControlFlowGraph,
//     paths_with_return: HashSet<NodeId>,
// }
//
// impl ReturnPathAnalyzer {
//     // Vérifie si tous les chemins de retour sont bien définis
//     pub fn all_paths_return(&mut self) -> bool {
//         // Marquer les nœuds avec un return
//         self.mark_return_nodes();
//
//         // Vérifier si tous les chemins depuis l'entrée jusqu'aux sorties contiennent un return
//         let mut visited = HashSet::new();
//         self.check_path(self.cfg.entry, &mut visited)
//     }
//
//     // Marque les nœuds qui ont un return ou qui mènent à un return
//     fn mark_return_nodes(&mut self) {
//         // 1. Marquer les nœuds qui contiennent un return
//         for (id, node) in &self.cfg.nodes {
//             if let CFGNode::Return(_) = node {
//                 self.paths_with_return.insert(*id);
//             }
//         }
//
//         // 2. Marquer les nœuds qui mènent tous à un return
//         let mut changed = true;
//         while changed {
//             changed = false;
//
//             for (node_id, successors) in &self.cfg.edges {
//                 if self.paths_with_return.contains(node_id) {
//                     continue;  // Déjà marqué
//                 }
//
//                 // Si tous les successeurs mènent à un return, ce nœud aussi
//                 if !successors.is_empty() && successors.iter().all(|succ| self.paths_with_return.contains(succ)) {
//                     self.paths_with_return.insert(*node_id);
//                     changed = true;
//                 }
//             }
//         }
//     }
//
//     // Vérifie si un chemin depuis node_id contient un return
//     fn check_path(&self, node_id: NodeId, visited: &mut HashSet<NodeId>) -> bool {
//         if visited.contains(&node_id) {
//             return true;  // Éviter les cycles
//         }
//
//         visited.insert(node_id);
//
//         // Si ce nœud mène à un return, c'est bon
//         if self.paths_with_return.contains(&node_id) {
//             return true;
//         }
//
//         // Si c'est une sortie sans return, c'est un problème
//         if self.cfg.exits.contains(&node_id) {
//             return false;
//         }
//
//         // Vérifier tous les successeurs
//         if let Some(successors) = self.cfg.edges.get(&node_id) {
//             // S'il n'y a pas de successeurs, c'est une fin de chemin sans return
//             if successors.is_empty() {
//                 return false;
//             }
//
//             // Vérifier que tous les successeurs mènent à un return
//             successors.iter().all(|succ| self.check_path(*succ, visited))
//         } else {
//             false  // Pas de successeurs = fin de chemin sans return
//         }
//     }
// }
//
//
//
// struct DefUseAnalyzer {
//     cfg: ControlFlowGraph,
//     symbol_table: SymbolTable,
//
//     // Définitions par nœud (node_id -> définitions)
//     defs: HashMap<NodeId, HashSet<SymbolId>>,
//
//     // Utilisations par nœud (node_id -> utilisations)
//     uses: HashMap<NodeId, HashSet<SymbolId>>,
//
//     // Définitions qui atteignent un nœud (node_id -> définitions)
//     reaching_defs: HashMap<NodeId, HashSet<(NodeId, SymbolId)>>,
// }
//
// impl DefUseAnalyzer {
//     // Réalise l'analyse des définitions atteignant chaque nœud
//     pub fn analyze_reaching_definitions(&mut self) {
//         // 1. Initialiser les ensembles de définitions et d'utilisations par nœud
//         self.init_def_use();
//
//         // 2. Calculer les définitions atteignant chaque nœud
//         self.compute_reaching_defs();
//     }
//
//     // Initialise les ensembles de définitions et d'utilisations
//     fn init_def_use(&mut self) {
//         for (node_id, node) in &self.cfg.nodes {
//             let (defs, uses) = self.analyze_def_use(node);
//             self.defs.insert(*node_id, defs);
//             self.uses.insert(*node_id, uses);
//         }
//     }
//
//     // Analyse les définitions et utilisations dans un nœud
//     fn analyze_def_use(&self, node: &CFGNode) -> (HashSet<SymbolId>, HashSet<SymbolId>) {
//         let mut defs = HashSet::new();
//         let mut uses = HashSet::new();
//
//         match node {
//             CFGNode::Declaration(decl) => {
//                 let symbol_id = self.symbol_table.lookup_symbol(&decl.name).unwrap();
//                 defs.insert(symbol_id);
//
//                 // Si la déclaration a une initialisation, c'est aussi une utilisation
//                 if let Some(init) = &decl.initializer {
//                     self.collect_uses(init, &mut uses);
//                 }
//             }
//         }
//     }
// }