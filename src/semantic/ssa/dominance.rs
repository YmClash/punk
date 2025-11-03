//src/semantic/ssa/dominance.rs

use std::collections::{HashMap, HashSet};
use crate::semantic::flow::BlockId;

/// Arbre de dominance
#[derive(Debug, Clone)]
pub struct DominanceTree {
    /// Dominateurs immédiats : block -> son dominateur immédiat
    pub immediate_dominators: HashMap<BlockId, BlockId>,
    
    /// Enfants dans l'arbre de dominance : block -> ses enfants
    pub children: HashMap<BlockId, Vec<BlockId>>,
    
    /// Frontières de dominance : block -> sa frontière de dominance
    pub dominance_frontiers: HashMap<BlockId, HashSet<BlockId>>,
    
    /// Profondeur dans l'arbre : block -> profondeur
    pub depths: HashMap<BlockId, usize>,
    
    /// Bloc d'entrée
    pub entry: BlockId,
}

impl DominanceTree {
    pub fn new(entry: BlockId) -> Self {
        DominanceTree {
            immediate_dominators: HashMap::new(),
            children: HashMap::new(),
            dominance_frontiers: HashMap::new(),
            depths: HashMap::new(),
            entry,
        }
    }
    
    /// Vérifie si a domine b
    pub fn dominates(&self, a: BlockId, b: BlockId) -> bool {
        if a == b {
            return true;
        }
        
        let mut current = b;
        while let Some(&idom) = self.immediate_dominators.get(&current) {
            if idom == a {
                return true;
            }
            if idom == self.entry && idom != a {
                break;
            }
            current = idom;
        }
        
        false
    }
    
    /// Trouve l'ancêtre commun le plus proche dans l'arbre de dominance
    pub fn find_common_dominator(&self, a: BlockId, b: BlockId) -> BlockId {
        let mut path_a = HashSet::new();
        let mut current = a;
        
        // Construire le chemin de a vers la racine
        path_a.insert(current);
        while let Some(&idom) = self.immediate_dominators.get(&current) {
            path_a.insert(idom);
            if idom == self.entry {
                break;
            }
            current = idom;
        }
        
        // Parcourir le chemin de b jusqu'à trouver un nœud dans path_a
        current = b;
        if path_a.contains(&current) {
            return current;
        }
        
        while let Some(&idom) = self.immediate_dominators.get(&current) {
            if path_a.contains(&idom) {
                return idom;
            }
            current = idom;
        }
        
        self.entry
    }
    
    /// Calcule la profondeur de chaque nœud dans l'arbre
    pub fn compute_depths(&mut self) {
        self.depths.clear();
        self.compute_depth_recursive(self.entry, 0);
    }
    
    fn compute_depth_recursive(&mut self, block: BlockId, depth: usize) {
        self.depths.insert(block, depth);
        
        if let Some(children) = self.children.get(&block).cloned() {
            for child in children {
                self.compute_depth_recursive(child, depth + 1);
            }
        }
    }
    
    /// Construit la liste des enfants à partir des dominateurs immédiats
    pub fn build_children_lists(&mut self) {
        self.children.clear();
        
        for (block, &idom) in &self.immediate_dominators {
            self.children.entry(idom)
                .or_insert_with(Vec::new)
                .push(*block);
        }
    }
    
    /// Parcours en profondeur de l'arbre de dominance
    pub fn dfs_preorder(&self) -> Vec<BlockId> {
        let mut result = Vec::new();
        let mut stack = vec![self.entry];
        
        while let Some(block) = stack.pop() {
            result.push(block);
            
            if let Some(children) = self.children.get(&block) {
                // Ajouter les enfants en ordre inverse pour maintenir l'ordre
                for child in children.iter().rev() {
                    stack.push(*child);
                }
            }
        }
        
        result
    }
    
    /// Parcours post-ordre de l'arbre de dominance
    pub fn dfs_postorder(&self) -> Vec<BlockId> {
        let mut result = Vec::new();
        self.dfs_postorder_recursive(self.entry, &mut result);
        result
    }
    
    fn dfs_postorder_recursive(&self, block: BlockId, result: &mut Vec<BlockId>) {
        if let Some(children) = self.children.get(&block) {
            for child in children {
                self.dfs_postorder_recursive(*child, result);
            }
        }
        result.push(block);
    }
}

/// Algorithme pour calculer l'arbre de dominance
pub struct DominanceComputation;

impl DominanceComputation {
    /// Calcule l'arbre de dominance en utilisant l'algorithme de Cooper, Harvey et Kennedy
    pub fn compute(
        entry: BlockId,
        blocks: &HashMap<BlockId, Vec<BlockId>>, // block -> successors
        predecessors: &HashMap<BlockId, Vec<BlockId>> // block -> predecessors
    ) -> DominanceTree {
        let mut tree = DominanceTree::new(entry);
        
        // Ordre de parcours inverse post-ordre
        let rpo = Self::reverse_postorder(entry, blocks);
        let mut rpo_index: HashMap<BlockId, usize> = HashMap::new();
        for (i, &block) in rpo.iter().enumerate() {
            rpo_index.insert(block, i);
        }
        
        // Initialiser les dominateurs
        let mut doms: HashMap<BlockId, BlockId> = HashMap::new();
        doms.insert(entry, entry);
        
        // Itérer jusqu'au point fixe
        let mut changed = true;
        while changed {
            changed = false;
            
            for &block in &rpo[1..] { // Ignorer l'entrée
                let empty_vec = Vec::new();
                let preds = predecessors.get(&block).unwrap_or(&empty_vec);
                
                // Trouver le premier prédécesseur déjà traité
                let mut new_idom = None;
                for &pred in preds {
                    if doms.contains_key(&pred) {
                        new_idom = Some(pred);
                        break;
                    }
                }
                
                if let Some(mut new_idom_block) = new_idom {
                    // Pour chaque autre prédécesseur déjà traité
                    for &pred in preds {
                        if doms.contains_key(&pred) && pred != new_idom_block {
                            new_idom_block = Self::intersect(
                                pred,
                                new_idom_block,
                                &doms,
                                &rpo_index
                            );
                        }
                    }
                    
                    if doms.get(&block) != Some(&new_idom_block) {
                        doms.insert(block, new_idom_block);
                        changed = true;
                    }
                }
            }
        }
        
        // Construire l'arbre de dominance
        for (block, idom) in doms {
            if block != entry {
                tree.immediate_dominators.insert(block, idom);
            }
        }
        
        tree.build_children_lists();
        tree.compute_depths();
        
        // Calculer les frontières de dominance
        Self::compute_dominance_frontiers(&mut tree, predecessors);
        
        tree
    }
    
    /// Calcule l'ordre inverse post-ordre
    fn reverse_postorder(
        entry: BlockId,
        successors: &HashMap<BlockId, Vec<BlockId>>
    ) -> Vec<BlockId> {
        let mut visited = HashSet::new();
        let mut postorder = Vec::new();
        
        Self::dfs_postorder(entry, successors, &mut visited, &mut postorder);
        postorder.reverse();
        postorder
    }
    
    fn dfs_postorder(
        block: BlockId,
        successors: &HashMap<BlockId, Vec<BlockId>>,
        visited: &mut HashSet<BlockId>,
        postorder: &mut Vec<BlockId>
    ) {
        if !visited.insert(block) {
            return;
        }
        
        if let Some(succs) = successors.get(&block) {
            for &succ in succs {
                Self::dfs_postorder(succ, successors, visited, postorder);
            }
        }
        
        postorder.push(block);
    }
    
    /// Trouve l'ancêtre commun de deux blocs
    fn intersect(
        b1: BlockId,
        b2: BlockId,
        doms: &HashMap<BlockId, BlockId>,
        rpo_index: &HashMap<BlockId, usize>
    ) -> BlockId {
        let mut finger1 = b1;
        let mut finger2 = b2;
        
        while finger1 != finger2 {
            while rpo_index[&finger1] > rpo_index[&finger2] {
                finger1 = doms[&finger1];
            }
            while rpo_index[&finger2] > rpo_index[&finger1] {
                finger2 = doms[&finger2];
            }
        }
        
        finger1
    }
    
    /// Calcule les frontières de dominance
    fn compute_dominance_frontiers(
        tree: &mut DominanceTree,
        predecessors: &HashMap<BlockId, Vec<BlockId>>
    ) {
        for (block, preds) in predecessors {
            for &pred in preds {
                let mut runner = pred;
                
                // Remonter jusqu'au dominateur immédiat de block
                while runner != tree.immediate_dominators.get(block).copied().unwrap_or(tree.entry) {
                    tree.dominance_frontiers
                        .entry(runner)
                        .or_insert_with(HashSet::new)
                        .insert(*block);
                    
                    if runner == tree.entry {
                        break;
                    }
                    
                    runner = tree.immediate_dominators.get(&runner)
                        .copied()
                        .unwrap_or(tree.entry);
                }
            }
        }
    }
}