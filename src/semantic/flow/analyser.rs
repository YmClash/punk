//src/semantic/flow/analyser.rs

use std::collections::{HashMap, HashSet};
use crate::semantic::symbol_table::SymbolTable;
use crate::semantic::symbols::SymbolId;
use crate::semantic::flow::control_flow_graph::{ControlFlowGraph, BlockId, Instruction, Terminator};
use crate::parser::ast::Expression;

pub struct UsageAnalyzer {
    symbol_table: SymbolTable,
    cfg: ControlFlowGraph,
    defined_variables: HashSet<SymbolId>,
    used_variables: HashSet<SymbolId>,
}

impl UsageAnalyzer {
    pub fn new(symbol_table: SymbolTable, cfg: ControlFlowGraph) -> Self {
        UsageAnalyzer {
            symbol_table,
            cfg,
            defined_variables: HashSet::new(),
            used_variables: HashSet::new(),
        }
    }
    
    /// Analyse l'utilisation des variables
    pub fn analyze(&mut self) -> Vec<SymbolId> {
        // Cloner les blocks pour éviter les conflits d'emprunt
        let blocks = self.cfg.blocks.clone();
        
        // Parcourir le CFG pour identifier les variables définies et utilisées
        for (_, block) in blocks.iter() {
            for instruction in &block.instructions {
                self.analyze_instruction(instruction);
            }
            
            // Analyser le terminateur
            self.analyze_terminator(&block.terminator);
        }
        
        // Retourner les variables définies mais non utilisées
        self.defined_variables
            .difference(&self.used_variables)
            .copied()
            .collect()
    }
    
    fn analyze_instruction(&mut self, instruction: &Instruction) {
        match instruction {
            Instruction::VarDecl { name, value, .. } => {
                // Marquer la variable comme définie
                if let Ok(symbol_id) = self.symbol_table.lookup_symbol(name) {
                    self.defined_variables.insert(symbol_id);
                }
                
                // Si il y a une valeur, analyser les variables utilisées dedans
                if let Some(expr) = value {
                    self.analyze_expression_usage(expr);
                }
            }
            Instruction::Assignment { target, value, .. } => {
                // La cible est une définition, la valeur est une utilisation
                if let Ok(symbol_id) = self.symbol_table.lookup_symbol(target) {
                    self.defined_variables.insert(symbol_id);
                }
                self.analyze_expression_usage(value);
            }
            Instruction::Expression { expr, .. } => {
                self.analyze_expression_usage(expr);
            }
            Instruction::Phi { var, sources, .. } => {
                // Variable définie par le phi node
                if let Ok(symbol_id) = self.symbol_table.lookup_symbol(var) {
                    self.defined_variables.insert(symbol_id);
                }
                // Sources utilisées
                for (_, source_var) in sources {
                    if let Ok(symbol_id) = self.symbol_table.lookup_symbol(source_var) {
                        self.used_variables.insert(symbol_id);
                    }
                }
            }
        }
    }
    
    fn analyze_terminator(&mut self, terminator: &Terminator) {
        match terminator {
            Terminator::ConditionalJump { condition, .. } => {
                self.analyze_expression_usage(condition);
            }
            Terminator::Return(Some(expr)) => {
                self.analyze_expression_usage(expr);
            }
            _ => {}
        }
    }
    
    fn analyze_expression_usage(&mut self, expr: &Expression) {
        match expr {
            Expression::Identifier(name) => {
                // Marquer la variable comme utilisée
                if let Ok(symbol_id) = self.symbol_table.lookup_symbol(name) {
                    self.used_variables.insert(symbol_id);
                }
            }
            Expression::BinaryOperation(binop) => {
                self.analyze_expression_usage(&binop.left);
                self.analyze_expression_usage(&binop.right);
            }
            Expression::UnaryOperation(unop) => {
                self.analyze_expression_usage(&unop.operand);
            }
            Expression::FunctionCall(call) => {
                for arg in &call.arguments {
                    self.analyze_expression_usage(arg);
                }
            }
            Expression::Array(array) => {
                for elem in &array.elements {
                    self.analyze_expression_usage(elem);
                }
            }
            Expression::IndexAccess(access) => {
                self.analyze_expression_usage(&access.array);
                self.analyze_expression_usage(&access.index);
            }
            Expression::MemberAccess(access) => {
                self.analyze_expression_usage(&access.object);
            }
            Expression::Assignment(assign) => {
                self.analyze_expression_usage(&assign.target);
                self.analyze_expression_usage(&assign.value);
            }
            Expression::MethodCall(method) => {
                self.analyze_expression_usage(&method.object);
                for arg in &method.arguments {
                    self.analyze_expression_usage(arg);
                }
            }
            Expression::TypeCast(cast) => {
                self.analyze_expression_usage(&cast.expression);
            }
            _ => {
                // Autres cas (literals, etc.)
            }
        }
    }
}

pub struct ReturnPathAnalyzer {
    cfg: ControlFlowGraph,
    paths_with_return: HashSet<BlockId>,
}

impl ReturnPathAnalyzer {
    pub fn new(cfg: ControlFlowGraph) -> Self {
        ReturnPathAnalyzer {
            cfg,
            paths_with_return: HashSet::new(),
        }
    }
    
    /// Vérifie si tous les chemins d'exécution retournent une valeur
    pub fn all_paths_return(&mut self) -> bool {
        let mut visited = HashSet::new();
        self.check_path(self.cfg.entry, &mut visited)
    }
    
    fn check_path(&self, block_id: BlockId, visited: &mut HashSet<BlockId>) -> bool {
        if visited.contains(&block_id) {
            return false; // Déjà visité, éviter les boucles infinies
        }
        visited.insert(block_id);
        
        if let Some(block) = self.cfg.blocks.get(&block_id) {
            match &block.terminator {
                Terminator::Return(_) => true,
                Terminator::Jump(next) => self.check_path(*next, visited),
                Terminator::ConditionalJump { then_block, else_block, .. } => {
                    let mut then_visited = visited.clone();
                    let mut else_visited = visited.clone();
                    let then_returns = self.check_path(*then_block, &mut then_visited);
                    let else_returns = self.check_path(*else_block, &mut else_visited);
                    then_returns && else_returns
                }
                Terminator::Unreachable => false,
                _ => false,
            }
        } else {
            false
        }
    }
}

pub struct DefUseAnalyzer {
    cfg: ControlFlowGraph,
    symbol_table: SymbolTable,
    defs: HashMap<BlockId, HashSet<SymbolId>>,
    uses: HashMap<BlockId, HashSet<SymbolId>>,
    reaching_defs: HashMap<BlockId, HashSet<(BlockId, SymbolId)>>,
}

impl DefUseAnalyzer {
    pub fn new(cfg: ControlFlowGraph, symbol_table: SymbolTable) -> Self {
        DefUseAnalyzer {
            cfg,
            symbol_table,
            defs: HashMap::new(),
            uses: HashMap::new(),
            reaching_defs: HashMap::new(),
        }
    }
    
    /// Analyse les définitions atteignables
    pub fn analyze_reaching_definitions(&mut self) {
        // Initialiser les ensembles de définitions et d'utilisations par bloc
        self.init_def_use();
        
        // Calculer les définitions atteignant chaque bloc
        self.compute_reaching_defs();
    }
    
    fn init_def_use(&mut self) {
        for (block_id, block) in &self.cfg.blocks {
            let mut defs = HashSet::new();
            let mut uses = HashSet::new();
            
            for instruction in &block.instructions {
                match instruction {
                    Instruction::VarDecl { name, value, .. } => {
                        if let Ok(symbol_id) = self.symbol_table.lookup_symbol(name) {
                            defs.insert(symbol_id);
                        }
                        if let Some(expr) = value {
                            self.collect_uses(expr, &mut uses);
                        }
                    }
                    Instruction::Assignment { target, value, .. } => {
                        if let Ok(symbol_id) = self.symbol_table.lookup_symbol(target) {
                            defs.insert(symbol_id);
                        }
                        self.collect_uses(value, &mut uses);
                    }
                    Instruction::Expression { expr, .. } => {
                        self.collect_uses(expr, &mut uses);
                    }
                    Instruction::Phi { var, sources, .. } => {
                        if let Ok(symbol_id) = self.symbol_table.lookup_symbol(var) {
                            defs.insert(symbol_id);
                        }
                        for (_, source_var) in sources {
                            if let Ok(symbol_id) = self.symbol_table.lookup_symbol(source_var) {
                                uses.insert(symbol_id);
                            }
                        }
                    }
                }
            }
            
            self.defs.insert(*block_id, defs);
            self.uses.insert(*block_id, uses);
        }
    }
    
    fn collect_uses(&self, expr: &Expression, uses: &mut HashSet<SymbolId>) {
        match expr {
            Expression::Identifier(name) => {
                if let Ok(symbol_id) = self.symbol_table.lookup_symbol(name) {
                    uses.insert(symbol_id);
                }
            }
            Expression::BinaryOperation(binop) => {
                self.collect_uses(&binop.left, uses);
                self.collect_uses(&binop.right, uses);
            }
            Expression::UnaryOperation(unop) => {
                self.collect_uses(&unop.operand, uses);
            }
            _ => {}
        }
    }
    
    fn compute_reaching_defs(&mut self) {
        // Algorithme de point fixe pour calculer les définitions atteignables
        let mut changed = true;
        
        // Initialisation
        for block_id in self.cfg.blocks.keys() {
            self.reaching_defs.insert(*block_id, HashSet::new());
        }
        
        while changed {
            changed = false;
            
            for (block_id, block) in &self.cfg.blocks {
                let mut new_reaching = HashSet::new();
                
                // Union des définitions des prédécesseurs
                for &pred in &block.predecessors {
                    if let Some(pred_reaching) = self.reaching_defs.get(&pred) {
                        new_reaching.extend(pred_reaching.iter().copied());
                    }
                    
                    // Ajouter les définitions du prédécesseur
                    if let Some(pred_defs) = self.defs.get(&pred) {
                        for &symbol_id in pred_defs {
                            new_reaching.insert((pred, symbol_id));
                        }
                    }
                }
                
                // Vérifier si l'ensemble a changé
                if self.reaching_defs.get(block_id) != Some(&new_reaching) {
                    self.reaching_defs.insert(*block_id, new_reaching);
                    changed = true;
                }
            }
        }
    }
    
    /// Vérifie les variables non initialisées
    pub fn check_uninitialized_variables(&self) -> Vec<(SymbolId, BlockId)> {
        let mut uninitialized = Vec::new();
        
        for (block_id, block_uses) in &self.uses {
            if let Some(reaching) = self.reaching_defs.get(block_id) {
                for &symbol_id in block_uses {
                    // Vérifier si la variable est définie avant utilisation
                    let is_defined = reaching.iter().any(|(_, def_symbol)| *def_symbol == symbol_id);
                    
                    if !is_defined {
                        uninitialized.push((symbol_id, *block_id));
                    }
                }
            }
        }
        
        uninitialized
    }
}

pub struct ExecutionPathAnalyzer {
    cfg: ControlFlowGraph,
}

impl ExecutionPathAnalyzer {
    pub fn new(cfg: ControlFlowGraph) -> Self {
        ExecutionPathAnalyzer { cfg }
    }
    
    /// Trouve tous les chemins d'exécution possibles
    pub fn find_all_paths(&self) -> Vec<Vec<BlockId>> {
        let mut paths = Vec::new();
        let mut current_path = Vec::new();
        let mut visited = HashSet::new();
        
        self.find_paths_recursive(self.cfg.entry, &mut current_path, &mut visited, &mut paths);
        
        paths
    }
    
    fn find_paths_recursive(
        &self,
        block_id: BlockId,
        current_path: &mut Vec<BlockId>,
        visited: &mut HashSet<BlockId>,
        all_paths: &mut Vec<Vec<BlockId>>
    ) {
        if visited.contains(&block_id) {
            return; // Éviter les cycles
        }
        
        current_path.push(block_id);
        visited.insert(block_id);
        
        if let Some(block) = self.cfg.blocks.get(&block_id) {
            if block.successors.is_empty() || self.cfg.exits.contains(&block_id) {
                // C'est un noeud terminal
                all_paths.push(current_path.clone());
            } else {
                for &successor in &block.successors {
                    self.find_paths_recursive(successor, current_path, visited, all_paths);
                }
            }
        }
        
        current_path.pop();
        visited.remove(&block_id);
    }
}