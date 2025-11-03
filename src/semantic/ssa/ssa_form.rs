//src/semantic/ssa/ssa_form.rs

use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::cell::RefCell;
use crate::parser::ast::{Expression, Statement, Declaration, ASTNode};
use crate::semantic::symbols::SymbolId;
use crate::semantic::types::type_system::TypeId;
use crate::semantic::flow::{ControlFlowGraph, BlockId, BasicBlock};
use crate::semantic::context::CompilationContext;

/// Identifiant unique pour les variables SSA
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SSAVarId(pub u32);

impl std::fmt::Display for SSAVarId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}", self.0)
    }
}

/// Représente une variable SSA avec sa version
#[derive(Debug, Clone)]
pub struct SSAVariable {
    pub id: SSAVarId,
    pub original_name: String,
    pub version: u32,
    pub type_id: Option<TypeId>,
    pub symbol_id: Option<SymbolId>,
}

impl SSAVariable {
    pub fn versioned_name(&self) -> String {
        format!("{}_{}", self.original_name, self.version)
    }
}

/// Instruction SSA
#[derive(Debug, Clone)]
pub enum SSAInstruction {
    /// Assignation : dest = value
    Assign {
        dest: SSAVarId,
        value: SSAValue,
    },
    
    /// Opération binaire : dest = left op right
    BinaryOp {
        dest: SSAVarId,
        op: BinaryOperator,
        left: SSAValue,
        right: SSAValue,
    },
    
    /// Opération unaire : dest = op operand
    UnaryOp {
        dest: SSAVarId,
        op: UnaryOperator,
        operand: SSAValue,
    },
    
    /// Appel de fonction : dest = call(func, args)
    Call {
        dest: Option<SSAVarId>,
        func: String,
        args: Vec<SSAValue>,
    },
    
    /// Phi node : dest = φ(var1, var2, ...)
    Phi {
        dest: SSAVarId,
        sources: Vec<(BlockId, SSAVarId)>,
    },
    
    /// Branchement conditionnel
    Branch {
        condition: SSAValue,
        true_block: BlockId,
        false_block: BlockId,
    },
    
    /// Saut inconditionnel
    Jump {
        target: BlockId,
    },
    
    /// Retour de fonction
    Return {
        value: Option<SSAValue>,
    },
    
    /// Chargement depuis la mémoire
    Load {
        dest: SSAVarId,
        address: SSAValue,
    },
    
    /// Stockage en mémoire
    Store {
        address: SSAValue,
        value: SSAValue,
    },
}

/// Valeur SSA (variable ou constante)
#[derive(Debug, Clone)]
pub enum SSAValue {
    Variable(SSAVarId),
    Constant(ConstantValue),
}

/// Valeur constante
#[derive(Debug, Clone)]
pub enum ConstantValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Null,
}

/// Opérateurs binaires
#[derive(Debug, Clone, Copy)]
pub enum BinaryOperator {
    Add, Sub, Mul, Div, Mod,
    Equal, NotEqual,
    LessThan, LessEqual, GreaterThan, GreaterEqual,
    And, Or,
    BitwiseAnd, BitwiseOr, BitwiseXor,
}

/// Opérateurs unaires
#[derive(Debug, Clone, Copy)]
pub enum UnaryOperator {
    Negate, Not, BitwiseNot,
}

/// Bloc SSA enrichi
#[derive(Debug, Clone)]
pub struct SSABlock {
    pub id: BlockId,
    pub instructions: Vec<SSAInstruction>,
    pub predecessors: Vec<BlockId>,
    pub successors: Vec<BlockId>,
    pub dominators: HashSet<BlockId>,
    pub immediate_dominator: Option<BlockId>,
    pub dominance_frontier: HashSet<BlockId>,
}

/// Forme SSA du programme
pub struct SSAForm {
    pub blocks: HashMap<BlockId, SSABlock>,
    pub variables: HashMap<SSAVarId, SSAVariable>,
    pub entry_block: BlockId,
    pub exit_blocks: Vec<BlockId>,
    next_var_id: u32,
    variable_versions: HashMap<String, u32>,
    variable_stacks: HashMap<String, Vec<SSAVarId>>,
}

impl SSAForm {
    pub fn new() -> Self {
        SSAForm {
            blocks: HashMap::new(),
            variables: HashMap::new(),
            entry_block: BlockId(0),
            exit_blocks: Vec::new(),
            next_var_id: 0,
            variable_versions: HashMap::new(),
            variable_stacks: HashMap::new(),
        }
    }
    
    /// Crée une nouvelle variable SSA
    pub fn new_variable(&mut self, name: String, type_id: Option<TypeId>) -> SSAVarId {
        let id = SSAVarId(self.next_var_id);
        self.next_var_id += 1;
        
        // Obtenir la prochaine version pour cette variable
        let version = self.variable_versions.entry(name.clone())
            .and_modify(|v| *v += 1)
            .or_insert(0);
        
        let var = SSAVariable {
            id,
            original_name: name.clone(),
            version: *version,
            type_id,
            symbol_id: None,
        };
        
        self.variables.insert(id, var);
        
        // Empiler cette version
        self.variable_stacks.entry(name)
            .or_insert_with(Vec::new)
            .push(id);
        
        id
    }
    
    /// Obtient la version actuelle d'une variable
    pub fn get_current_version(&self, name: &str) -> Option<SSAVarId> {
        self.variable_stacks.get(name)
            .and_then(|stack| stack.last())
            .copied()
    }
    
    /// Convertit un CFG en forme SSA
    pub fn from_cfg(cfg: &ControlFlowGraph, context: Rc<RefCell<CompilationContext>>) -> Self {
        let mut ssa = SSAForm::new();
        
        // Étape 1: Calculer l'arbre de dominance
        ssa.compute_dominance_tree(cfg);
        
        // Étape 2: Calculer les frontières de dominance
        ssa.compute_dominance_frontiers();
        
        // Étape 3: Placer les phi-nodes
        ssa.place_phi_nodes(cfg);
        
        // Étape 4: Renommer les variables
        ssa.rename_variables(cfg);
        
        ssa
    }
    
    /// Calcule l'arbre de dominance
    fn compute_dominance_tree(&mut self, cfg: &ControlFlowGraph) {
        // Algorithme de Lengauer-Tarjan pour calculer les dominateurs
        let entry = cfg.entry;
        
        // Initialisation : chaque bloc se domine lui-même
        for block_id in cfg.blocks.keys() {
            let mut ssa_block = SSABlock {
                id: *block_id,
                instructions: Vec::new(),
                predecessors: cfg.get_predecessors(*block_id),
                successors: cfg.get_successors(*block_id),
                dominators: HashSet::new(),
                immediate_dominator: None,
                dominance_frontier: HashSet::new(),
            };
            
            // Le bloc d'entrée domine tous les blocs
            if *block_id == entry {
                ssa_block.dominators.insert(*block_id);
            } else {
                // Initialiser avec tous les blocs
                for b in cfg.blocks.keys() {
                    ssa_block.dominators.insert(*b);
                }
            }
            
            self.blocks.insert(*block_id, ssa_block);
        }
        
        // Point fixe pour calculer les dominateurs
        let mut changed = true;
        while changed {
            changed = false;
            
            for block_id in cfg.blocks.keys() {
                if *block_id == entry {
                    continue;
                }
                
                // Nouveau ensemble de dominateurs = intersection des dominateurs des prédécesseurs + soi-même
                let preds = cfg.get_predecessors(*block_id);
                if !preds.is_empty() {
                    let mut new_doms: HashSet<BlockId> = self.blocks[&preds[0]].dominators.clone();
                    
                    for pred in preds.iter().skip(1) {
                        let pred_doms = &self.blocks[pred].dominators;
                        new_doms = new_doms.intersection(pred_doms).cloned().collect();
                    }
                    
                    new_doms.insert(*block_id);
                    
                    let old_size = self.blocks[block_id].dominators.len();
                    self.blocks.get_mut(block_id).unwrap().dominators = new_doms;
                    
                    if self.blocks[block_id].dominators.len() != old_size {
                        changed = true;
                    }
                }
            }
        }
        
        // Calculer les dominateurs immédiats
        for block_id in cfg.blocks.keys() {
            if *block_id == entry {
                continue;
            }
            
            let dominators = self.blocks[block_id].dominators.clone();
            for dom in &dominators {
                if *dom == *block_id {
                    continue;
                }
                
                // dom est un dominateur immédiat si aucun autre dominateur ne le domine
                let mut is_immediate = true;
                for other_dom in &dominators {
                    if *other_dom == *block_id || *other_dom == *dom {
                        continue;
                    }
                    
                    if self.blocks[other_dom].dominators.contains(dom) {
                        is_immediate = false;
                        break;
                    }
                }
                
                if is_immediate {
                    self.blocks.get_mut(block_id).unwrap().immediate_dominator = Some(*dom);
                    break;
                }
            }
        }
    }
    
    /// Calcule les frontières de dominance
    fn compute_dominance_frontiers(&mut self) {
        let block_ids: Vec<BlockId> = self.blocks.keys().cloned().collect();
        
        for block_id in &block_ids {
            let preds = self.blocks[block_id].predecessors.clone();
            
            for pred_id in preds {
                let mut runner = pred_id;
                
                // Remonter l'arbre de dominance jusqu'au dominateur immédiat de block_id
                while runner != self.blocks[block_id].immediate_dominator.unwrap_or(BlockId(u32::MAX)) {
                    self.blocks.get_mut(&runner).unwrap()
                        .dominance_frontier.insert(*block_id);
                    
                    // Remonter au dominateur immédiat
                    runner = self.blocks[&runner].immediate_dominator
                        .unwrap_or(BlockId(u32::MAX));
                    
                    if runner == BlockId(u32::MAX) {
                        break;
                    }
                }
            }
        }
    }
    
    /// Place les phi-nodes aux frontières de dominance
    fn place_phi_nodes(&mut self, cfg: &ControlFlowGraph) {
        // Pour chaque variable définie dans le programme
        let mut defined_vars: HashMap<String, HashSet<BlockId>> = HashMap::new();
        
        // Analyser chaque bloc pour trouver les définitions
        for (block_id, block) in &cfg.blocks {
            // TODO: Parcourir les instructions du bloc pour trouver les définitions
            // Pour l'instant, on simule avec des variables fictives
        }
        
        // Pour chaque variable, placer des phi-nodes
        for (var_name, def_blocks) in defined_vars {
            let mut work_list: VecDeque<BlockId> = def_blocks.iter().cloned().collect();
            let mut has_phi: HashSet<BlockId> = HashSet::new();
            
            while let Some(block_id) = work_list.pop_front() {
                let df = self.blocks[&block_id].dominance_frontier.clone();
                
                for frontier_block in df {
                    if !has_phi.contains(&frontier_block) {
                        // Insérer un phi-node au début du bloc
                        let phi_var = self.new_variable(var_name.clone(), None);
                        let phi_instruction = SSAInstruction::Phi {
                            dest: phi_var,
                            sources: Vec::new(), // Sera rempli lors du renommage
                        };
                        
                        self.blocks.get_mut(&frontier_block).unwrap()
                            .instructions.insert(0, phi_instruction);
                        
                        has_phi.insert(frontier_block);
                        
                        if !def_blocks.contains(&frontier_block) {
                            work_list.push_back(frontier_block);
                        }
                    }
                }
            }
        }
    }
    
    /// Renomme les variables pour la forme SSA
    fn rename_variables(&mut self, cfg: &ControlFlowGraph) {
        // Parcours en profondeur de l'arbre de dominance
        self.rename_block(cfg.entry, cfg);
    }
    
    /// Renomme les variables dans un bloc et ses enfants
    fn rename_block(&mut self, block_id: BlockId, cfg: &ControlFlowGraph) {
        let old_stacks = self.variable_stacks.clone();
        
        // Renommer les phi-nodes
        let instructions = self.blocks[&block_id].instructions.clone();
        for (i, instruction) in instructions.iter().enumerate() {
            if let SSAInstruction::Phi { dest, .. } = instruction {
                // La destination du phi est une nouvelle version
                // TODO: Implémenter le renommage complet
            }
        }
        
        // Renommer les autres instructions
        // TODO: Parcourir et renommer toutes les utilisations et définitions
        
        // Renommer récursivement les enfants dans l'arbre de dominance
        let children: Vec<BlockId> = self.blocks.iter()
            .filter(|(_, b)| b.immediate_dominator == Some(block_id))
            .map(|(id, _)| *id)
            .collect();
        
        for child in children {
            self.rename_block(child, cfg);
        }
        
        // Restaurer les piles de variables
        self.variable_stacks = old_stacks;
    }
}

// Les méthodes get_predecessors et get_successors sont maintenant dans control_flow_graph.rs