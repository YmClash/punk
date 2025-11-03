//src/semantic/ssa/phi_node.rs

use std::collections::HashMap;
use super::ssa_form::SSAVarId;
use crate::semantic::flow::BlockId;

/// Représente un nœud phi dans la forme SSA
#[derive(Debug, Clone)]
pub struct PhiNode {
    /// Variable de destination (nouvelle version)
    pub dest: SSAVarId,
    
    /// Sources : mapping de bloc prédécesseur vers variable source
    pub sources: HashMap<BlockId, SSAVarId>,
    
    /// Nom original de la variable (avant SSA)
    pub original_name: String,
}

impl PhiNode {
    pub fn new(dest: SSAVarId, original_name: String) -> Self {
        PhiNode {
            dest,
            sources: HashMap::new(),
            original_name,
        }
    }
    
    /// Ajoute une source au phi-node
    pub fn add_source(&mut self, from_block: BlockId, var: SSAVarId) {
        self.sources.insert(from_block, var);
    }
    
    /// Obtient la variable source pour un bloc prédécesseur donné
    pub fn get_source(&self, from_block: BlockId) -> Option<SSAVarId> {
        self.sources.get(&from_block).copied()
    }
    
    /// Vérifie si le phi-node est complet (a une source pour chaque prédécesseur)
    pub fn is_complete(&self, predecessors: &[BlockId]) -> bool {
        predecessors.iter().all(|pred| self.sources.contains_key(pred))
    }
    
    /// Vérifie si le phi-node est trivial (toutes les sources sont identiques)
    pub fn is_trivial(&self) -> bool {
        if self.sources.is_empty() {
            return true;
        }
        
        let first_var = self.sources.values().next().unwrap();
        self.sources.values().all(|var| var == first_var)
    }
    
    /// Élimine les phi-nodes triviaux
    pub fn eliminate_if_trivial(&self) -> Option<SSAVarId> {
        if self.is_trivial() && !self.sources.is_empty() {
            Some(*self.sources.values().next().unwrap())
        } else {
            None
        }
    }
}

/// Gestionnaire de phi-nodes pour un bloc
#[derive(Debug, Clone)]
pub struct PhiNodeManager {
    /// Phi-nodes par variable originale
    pub phi_nodes: HashMap<String, PhiNode>,
}

impl PhiNodeManager {
    pub fn new() -> Self {
        PhiNodeManager {
            phi_nodes: HashMap::new(),
        }
    }
    
    /// Ajoute un phi-node pour une variable
    pub fn add_phi(&mut self, var_name: String, phi: PhiNode) {
        self.phi_nodes.insert(var_name, phi);
    }
    
    /// Obtient le phi-node pour une variable
    pub fn get_phi(&self, var_name: &str) -> Option<&PhiNode> {
        self.phi_nodes.get(var_name)
    }
    
    /// Obtient le phi-node mutable pour une variable
    pub fn get_phi_mut(&mut self, var_name: &str) -> Option<&mut PhiNode> {
        self.phi_nodes.get_mut(var_name)
    }
    
    /// Élimine tous les phi-nodes triviaux
    pub fn eliminate_trivial_phis(&mut self) -> HashMap<SSAVarId, SSAVarId> {
        let mut replacements = HashMap::new();
        
        self.phi_nodes.retain(|_, phi| {
            if let Some(replacement) = phi.eliminate_if_trivial() {
                replacements.insert(phi.dest, replacement);
                false // Supprimer le phi-node
            } else {
                true // Garder le phi-node
            }
        });
        
        replacements
    }
    
    /// Vérifie si tous les phi-nodes sont complets
    pub fn all_complete(&self, predecessors: &[BlockId]) -> bool {
        self.phi_nodes.values().all(|phi| phi.is_complete(predecessors))
    }
}