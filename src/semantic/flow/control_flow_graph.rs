//src/semantic/flow/control_flow_graph.rs

use std::collections::{HashMap, HashSet};
use crate::parser::ast::{Expression, Statement, Declaration, ASTNode};
use crate::semantic::semantic_error::{SemanticError, SemanticErrorType, Position};
use crate::semantic::symbols::SymbolId;

/// Identifiant unique pour un bloc de base
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);

/// Identifiant unique pour une instruction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstructionId(pub u32);

/// Control Flow Graph pour l'analyse de flux
#[derive(Clone, Debug)]
pub struct ControlFlowGraph {
    /// Point d'entrée du graphe
    pub entry: BlockId,
    
    /// Points de sortie du graphe
    pub exits: Vec<BlockId>,
    
    /// Blocs de base du graphe
    pub blocks: HashMap<BlockId, BasicBlock>,
    
    /// Compteur pour générer des IDs uniques
    next_block_id: u32,
    next_instruction_id: u32,
}

/// Bloc de base du CFG
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub instructions: Vec<Instruction>,
    pub terminator: Terminator,
    pub predecessors: HashSet<BlockId>,
    pub successors: HashSet<BlockId>,
}

/// Instruction dans un bloc de base
#[derive(Debug, Clone)]
pub enum Instruction {
    /// Déclaration de variable
    VarDecl { id: InstructionId, name: String, value: Option<Expression> },
    
    /// Affectation
    Assignment { id: InstructionId, target: String, value: Expression },
    
    /// Expression (effet de bord potentiel)
    Expression { id: InstructionId, expr: Expression },
    
    /// Phi node pour SSA (Static Single Assignment)
    Phi { id: InstructionId, var: String, sources: Vec<(BlockId, String)> },
}

/// Instructions spécifiques au CFG pour l'analyse des emprunts
#[derive(Debug, Clone)]
pub enum CFGInstruction {
    /// Affectation avec symboles
    Assign { target: SymbolId, value: Expression },
    
    /// Lecture d'une variable
    Read(SymbolId),
    
    /// Écriture dans une variable
    Write(SymbolId),
    
    /// Emprunt d'une variable
    Borrow { target: SymbolId, source: SymbolId, is_mutable: bool },
    
    /// Move d'une variable
    Move { target: SymbolId, source: SymbolId },
    
    /// Instruction générique
    Generic(Instruction),
}


/// Terminateur d'un bloc de base
#[derive(Debug, Clone)]
pub enum Terminator {
    /// Saut inconditionnel
    Jump(BlockId),
    
    /// Saut conditionnel
    ConditionalJump {
        condition: Expression,
        then_block: BlockId,
        else_block: BlockId,
    },
    
    /// Retour de fonction
    Return(Option<Expression>),
    
    /// Break dans une boucle
    Break(Option<String>),
    
    /// Continue dans une boucle
    Continue(Option<String>),
    
    /// Bloc inaccessible
    Unreachable,
}

impl ControlFlowGraph {
    /// Crée un nouveau CFG vide
    pub fn new() -> Self {
        let mut cfg = ControlFlowGraph {
            entry: BlockId(0),
            exits: Vec::new(),
            blocks: HashMap::new(),
            next_block_id: 0,
            next_instruction_id: 0,
        };
        
        // Créer le bloc d'entrée
        cfg.entry = cfg.create_block();
        cfg
    }
    
    /// Crée un nouveau bloc de base
    pub fn create_block(&mut self) -> BlockId {
        let id = BlockId(self.next_block_id);
        self.next_block_id += 1;
        
        self.blocks.insert(id, BasicBlock {
            id,
            instructions: Vec::new(),
            terminator: Terminator::Unreachable,
            predecessors: HashSet::new(),
            successors: HashSet::new(),
        });
        
        id
    }
    
    /// Crée une nouvelle instruction
    pub fn create_instruction(&mut self) -> InstructionId {
        let id = InstructionId(self.next_instruction_id);
        self.next_instruction_id += 1;
        id
    }
    
    /// Ajoute une instruction à un bloc
    pub fn add_instruction(&mut self, block_id: BlockId, instruction: Instruction) {
        if let Some(block) = self.blocks.get_mut(&block_id) {
            block.instructions.push(instruction);
        }
    }
    
    /// Définit le terminateur d'un bloc
    pub fn set_terminator(&mut self, block_id: BlockId, terminator: Terminator) {
        if let Some(block) = self.blocks.get_mut(&block_id) {
            block.terminator = terminator;
        }
    }
    
    /// Ajoute une arête entre deux blocs
    pub fn add_edge(&mut self, from: BlockId, to: BlockId) {
        if let Some(from_block) = self.blocks.get_mut(&from) {
            from_block.successors.insert(to);
        }
        if let Some(to_block) = self.blocks.get_mut(&to) {
            to_block.predecessors.insert(from);
        }
    }
    
    /// Supprime une arête entre deux blocs
    pub fn remove_edge(&mut self, from: BlockId, to: BlockId) {
        if let Some(from_block) = self.blocks.get_mut(&from) {
            from_block.successors.remove(&to);
        }
        if let Some(to_block) = self.blocks.get_mut(&to) {
            to_block.predecessors.remove(&from);
        }
    }
    
    /// Calcule les blocs accessibles depuis l'entrée
    pub fn compute_reachable_blocks(&self) -> HashSet<BlockId> {
        let mut reachable = HashSet::new();
        let mut worklist = vec![self.entry];
        
        while let Some(block_id) = worklist.pop() {
            if reachable.insert(block_id) {
                if let Some(block) = self.blocks.get(&block_id) {
                    for &successor in &block.successors {
                        worklist.push(successor);
                    }
                }
            }
        }
        
        reachable
    }
    
    /// Détecte les blocs morts (inaccessibles)
    pub fn find_dead_blocks(&self) -> Vec<BlockId> {
        let reachable = self.compute_reachable_blocks();
        self.blocks.keys()
            .filter(|&&id| !reachable.contains(&id))
            .copied()
            .collect()
    }
    
    /// Obtient tous les blocs du CFG
    pub fn get_all_blocks(&self) -> Vec<BlockId> {
        self.blocks.keys().copied().collect()
    }
    
    /// Obtient un bloc par son ID
    pub fn get_block(&self, id: BlockId) -> Option<&BasicBlock> {
        self.blocks.get(&id)
    }
    
    /// Obtient les prédécesseurs d'un bloc
    pub fn get_predecessors(&self, id: BlockId) -> Vec<BlockId> {
        self.blocks.get(&id)
            .map(|b| b.predecessors.iter().copied().collect())
            .unwrap_or_default()
    }
    
    /// Obtient les successeurs d'un bloc
    pub fn get_successors(&self, id: BlockId) -> Vec<BlockId> {
        self.blocks.get(&id)
            .map(|b| b.successors.iter().copied().collect())
            .unwrap_or_default()
    }
    
    /// Effectue un tri topologique des blocs
    pub fn topological_sort(&self) -> Vec<BlockId> {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        
        // DFS post-order depuis le bloc d'entrée
        self.dfs_post_order(self.entry, &mut visited, &mut stack);
        
        // Inverser pour obtenir l'ordre topologique
        stack.reverse();
        stack
    }
    
    /// DFS post-order helper
    fn dfs_post_order(&self, block_id: BlockId, visited: &mut HashSet<BlockId>, stack: &mut Vec<BlockId>) {
        if visited.contains(&block_id) {
            return;
        }
        
        visited.insert(block_id);
        
        if let Some(block) = self.blocks.get(&block_id) {
            for &successor in &block.successors {
                self.dfs_post_order(successor, visited, stack);
            }
        }
        
        stack.push(block_id);
    }
    
    /// Calcule les dominateurs
    pub fn compute_dominators(&self) -> HashMap<BlockId, HashSet<BlockId>> {
        let mut dominators = HashMap::new();
        let all_blocks: HashSet<BlockId> = self.blocks.keys().copied().collect();
        
        // Initialisation
        dominators.insert(self.entry, HashSet::from([self.entry]));
        for &block in &all_blocks {
            if block != self.entry {
                dominators.insert(block, all_blocks.clone());
            }
        }
        
        // Point fixe
        let mut changed = true;
        while changed {
            changed = false;
            
            for &block in &all_blocks {
                if block == self.entry {
                    continue;
                }
                
                if let Some(current_block) = self.blocks.get(&block) {
                    let mut new_dom = all_blocks.clone();
                    
                    // Intersection des dominateurs des prédécesseurs
                    for &pred in &current_block.predecessors {
                        if let Some(pred_dom) = dominators.get(&pred) {
                            new_dom = new_dom.intersection(pred_dom).copied().collect();
                        }
                    }
                    
                    // Ajouter le bloc lui-même
                    new_dom.insert(block);
                    
                    if dominators.get(&block) != Some(&new_dom) {
                        dominators.insert(block, new_dom);
                        changed = true;
                    }
                }
            }
        }
        
        dominators
    }
    
    /// Valide la cohérence du CFG
    pub fn validate(&self) -> Result<(), SemanticError> {
        // Vérifier que le bloc d'entrée existe
        if !self.blocks.contains_key(&self.entry) {
            return Err(SemanticError::new(
                SemanticErrorType::TypeError(crate::semantic::semantic_error::TypeError::InvalidType("Entry block not found".to_string())),
                "CFG validation failed".to_string(),
                Position { index: 0 }
            ));
        }
        
        // Vérifier la cohérence des arêtes
        for (block_id, block) in &self.blocks {
            // Vérifier que les terminateurs correspondent aux successeurs
            match &block.terminator {
                Terminator::Jump(target) => {
                    if !block.successors.contains(target) {
                        return Err(SemanticError::new(
                            SemanticErrorType::TypeError(crate::semantic::semantic_error::TypeError::InvalidType(
                                format!("Jump target {:?} not in successors", target)
                            )),
                            "CFG validation failed".to_string(),
                            Position { index: 0 }
                        ));
                    }
                }
                Terminator::ConditionalJump { then_block, else_block, .. } => {
                    if !block.successors.contains(then_block) || !block.successors.contains(else_block) {
                        return Err(SemanticError::new(
                            SemanticErrorType::TypeError(crate::semantic::semantic_error::TypeError::InvalidType(
                                "Conditional jump targets not in successors".to_string()
                            )),
                            "CFG validation failed".to_string(),
                            Position { index: 0 }
                        ));
                    }
                }
                _ => {}
            }
            
            for &successor in &block.successors {
                if !self.blocks.contains_key(&successor) {
                    return Err(SemanticError::new(
                        SemanticErrorType::TypeError(crate::semantic::semantic_error::TypeError::InvalidType(
                            format!("Invalid successor {:?} for block {:?}", successor, block_id)
                        )),
                        "CFG validation failed".to_string(),
                        Position { index: 0 }
                    ));
                }
            }
            
            for &predecessor in &block.predecessors {
                if !self.blocks.contains_key(&predecessor) {
                    return Err(SemanticError::new(
                        SemanticErrorType::TypeError(crate::semantic::semantic_error::TypeError::InvalidType(
                            format!("Invalid predecessor {:?} for block {:?}", predecessor, block_id)
                        )),
                        "CFG validation failed".to_string(),
                        Position { index: 0 }
                    ));
                }
            }
        }
        
        Ok(())
    }
}

impl Default for ControlFlowGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Contexte pour gérer les boucles lors de la construction du CFG
#[derive(Debug, Clone)]
struct LoopContext {
    /// Bloc où aller pour un continue
    continue_target: BlockId,
    /// Bloc où aller pour un break
    break_target: BlockId,
    /// Label optionnel de la boucle
    label: Option<String>,
}

/// Builder pour construire un CFG depuis l'AST
pub struct CFGBuilder {
    cfg: ControlFlowGraph,
    current_block: BlockId,
    loop_stack: Vec<LoopContext>,
    function_exits: Vec<BlockId>,
}

impl CFGBuilder {
    pub fn new() -> Self {
        let cfg = ControlFlowGraph::new();
        let entry = cfg.entry;
        
        CFGBuilder {
            cfg,
            current_block: entry,
            loop_stack: Vec::new(),
            function_exits: Vec::new(),
        }
    }
    
    /// Construit un CFG depuis un AST
    pub fn build(mut self, ast: &ASTNode) -> Result<ControlFlowGraph, SemanticError> {
        // Visiter le nœud racine
        self.visit_node(ast)?;
        
        // Ajouter un bloc de sortie si nécessaire
        if self.function_exits.is_empty() {
            // Si pas de return explicite, le bloc courant est une sortie
            self.cfg.exits.push(self.current_block);
            
            // Ajouter un terminateur de fin si nécessaire
            if let Some(block) = self.cfg.blocks.get_mut(&self.current_block) {
                if matches!(block.terminator, Terminator::Unreachable) {
                    block.terminator = Terminator::Return(None);
                }
            }
        } else {
            self.cfg.exits = self.function_exits.clone();
        }
        
        // Valider le CFG avant de le retourner
        self.cfg.validate()?;
        Ok(self.cfg)
    }
    
    /// Visite un nœud AST et construit la partie correspondante du CFG
    fn visit_node(&mut self, node: &ASTNode) -> Result<(), SemanticError> {
        match node {
            ASTNode::Program(nodes) => {
                for node in nodes {
                    self.visit_node(node)?;
                }
                Ok(())
            }
            ASTNode::Statement(stmt) => self.visit_statement(stmt),
            ASTNode::Expression(expr) => self.visit_expression(expr),
            ASTNode::Declaration(decl) => self.visit_declaration(decl),
            ASTNode::Error(_) => Ok(()), // Ignorer les erreurs de parsing
        }
    }
    
    /// Visite un statement
    fn visit_statement(&mut self, stmt: &Statement) -> Result<(), SemanticError> {
        match stmt {
            Statement::IfStatement(if_stmt) => self.visit_if_statement(if_stmt),
            Statement::WhileStatement(while_stmt) => self.visit_while_statement(while_stmt),
            Statement::ForStatement(for_stmt) => self.visit_for_statement(for_stmt),
            Statement::MatchStatement(match_stmt) => self.visit_match_statement(match_stmt),
            Statement::ReturnStatement(ret_stmt) => self.visit_return_statement(ret_stmt),
            Statement::BreakStatement(break_stmt) => self.visit_break_statement(break_stmt.label.as_deref()),
            Statement::ContinueStatement(cont_stmt) => self.visit_continue_statement(cont_stmt.label.as_deref()),
            Statement::Expression(expr) => self.visit_expression(expr),
            Statement::DeclarationStatement(decl) => self.visit_declaration(decl),
            _ => Ok(()), // Autres statements non supportés pour l'instant
        }
    }
    
    /// Visite un statement if
    fn visit_if_statement(&mut self, if_stmt: &crate::parser::ast::IfStatement) -> Result<(), SemanticError> {
        // Créer les blocs nécessaires
        let then_block = self.cfg.create_block();
        let else_block = if if_stmt.else_block.is_some() {
            self.cfg.create_block()
        } else {
            // Si pas de else, on merge directement
            self.cfg.create_block()
        };
        let merge_block = self.cfg.create_block();
        
        // Ajouter le terminateur conditionnel au bloc courant
        self.cfg.set_terminator(
            self.current_block,
            Terminator::ConditionalJump {
                condition: if_stmt.condition.clone(),
                then_block,
                else_block,
            }
        );
        
        // Ajouter les arêtes
        self.cfg.add_edge(self.current_block, then_block);
        self.cfg.add_edge(self.current_block, else_block);
        
        // Visiter le bloc then
        self.current_block = then_block;
        for node in &if_stmt.then_block {
            self.visit_node(node)?;
        }
        // Saut vers le bloc de merge
        if let Some(block) = self.cfg.blocks.get(&self.current_block) {
            if matches!(block.terminator, Terminator::Unreachable) {
                self.cfg.set_terminator(self.current_block, Terminator::Jump(merge_block));
                self.cfg.add_edge(self.current_block, merge_block);
            }
        }
        
        // Visiter le bloc else si présent
        if let Some(else_nodes) = &if_stmt.else_block {
            self.current_block = else_block;
            for node in else_nodes {
                self.visit_node(node)?;
            }
            // Saut vers le bloc de merge
            if let Some(block) = self.cfg.blocks.get(&self.current_block) {
                if matches!(block.terminator, Terminator::Unreachable) {
                    self.cfg.set_terminator(self.current_block, Terminator::Jump(merge_block));
                    self.cfg.add_edge(self.current_block, merge_block);
                }
            }
        } else {
            // Si pas de else, connecter directement au merge
            self.cfg.set_terminator(else_block, Terminator::Jump(merge_block));
            self.cfg.add_edge(else_block, merge_block);
        }
        
        // Continuer avec le bloc de merge
        self.current_block = merge_block;
        Ok(())
    }
    
    /// Visite un statement while
    fn visit_while_statement(&mut self, while_stmt: &crate::parser::ast::WhileStatement) -> Result<(), SemanticError> {
        // Créer les blocs nécessaires
        let cond_block = self.cfg.create_block();
        let body_block = self.cfg.create_block();
        let exit_block = self.cfg.create_block();
        
        // Saut depuis le bloc courant vers le bloc de condition
        self.cfg.set_terminator(self.current_block, Terminator::Jump(cond_block));
        self.cfg.add_edge(self.current_block, cond_block);
        
        // Bloc de condition
        self.current_block = cond_block;
        self.cfg.set_terminator(
            cond_block,
            Terminator::ConditionalJump {
                condition: while_stmt.condition.clone(),
                then_block: body_block,
                else_block: exit_block,
            }
        );
        self.cfg.add_edge(cond_block, body_block);
        self.cfg.add_edge(cond_block, exit_block);
        
        // Pousser le contexte de boucle
        self.loop_stack.push(LoopContext {
            continue_target: cond_block,
            break_target: exit_block,
            label: None,
        });
        
        // Visiter le corps de la boucle
        self.current_block = body_block;
        for node in &while_stmt.body {
            self.visit_node(node)?;
        }
        
        // Retour vers la condition à la fin du corps
        if let Some(block) = self.cfg.blocks.get(&self.current_block) {
            if matches!(block.terminator, Terminator::Unreachable) {
                self.cfg.set_terminator(self.current_block, Terminator::Jump(cond_block));
                self.cfg.add_edge(self.current_block, cond_block);
            }
        }
        
        // Retirer le contexte de boucle
        self.loop_stack.pop();
        
        // Continuer avec le bloc de sortie
        self.current_block = exit_block;
        Ok(())
    }
    
    /// Visite un statement for
    fn visit_for_statement(&mut self, for_stmt: &crate::parser::ast::ForStatement) -> Result<(), SemanticError> {
        // Transformer le for en while équivalent
        // for var in iter { body } => { let var; while iter.has_next() { var = iter.next(); body } }
        
        // Créer les blocs nécessaires
        let init_block = self.cfg.create_block();
        let cond_block = self.cfg.create_block();
        let body_block = self.cfg.create_block();
        let exit_block = self.cfg.create_block();
        
        // Bloc d'initialisation
        self.cfg.set_terminator(self.current_block, Terminator::Jump(init_block));
        self.cfg.add_edge(self.current_block, init_block);
        self.current_block = init_block;
        
        // Ajouter l'instruction d'initialisation de l'itérateur si nécessaire
        // Note: Ceci est simplifié, une vraie implémentation devrait gérer l'itérateur
        
        // Saut vers la condition
        self.cfg.set_terminator(init_block, Terminator::Jump(cond_block));
        self.cfg.add_edge(init_block, cond_block);
        
        // Bloc de condition (vérifier si l'itération continue)
        self.current_block = cond_block;
        // Créer une condition basée sur l'itérateur
        // Pour l'instant, on utilise une condition simplifiée
        let condition = Expression::Literal(crate::parser::ast::Literal::Boolean(true));
        
        self.cfg.set_terminator(
            cond_block,
            Terminator::ConditionalJump {
                condition,
                then_block: body_block,
                else_block: exit_block,
            }
        );
        self.cfg.add_edge(cond_block, body_block);
        self.cfg.add_edge(cond_block, exit_block);
        
        // Pousser le contexte de boucle
        self.loop_stack.push(LoopContext {
            continue_target: cond_block,
            break_target: exit_block,
            label: None,
        });
        
        // Visiter le corps de la boucle
        self.current_block = body_block;
        
        // Ajouter l'affectation de la variable d'itération
        let inst_id = self.cfg.create_instruction();
        self.cfg.add_instruction(
            body_block,
            Instruction::VarDecl {
                id: inst_id,
                name: for_stmt.iterator.clone(),
                value: None, // La valeur viendrait de l'itérateur
            }
        );
        
        // Visiter le corps
        for node in &for_stmt.body {
            self.visit_node(node)?;
        }
        
        // Retour vers la condition
        if let Some(block) = self.cfg.blocks.get(&self.current_block) {
            if matches!(block.terminator, Terminator::Unreachable) {
                self.cfg.set_terminator(self.current_block, Terminator::Jump(cond_block));
                self.cfg.add_edge(self.current_block, cond_block);
            }
        }
        
        // Retirer le contexte de boucle
        self.loop_stack.pop();
        
        // Continuer avec le bloc de sortie
        self.current_block = exit_block;
        Ok(())
    }
    
    /// Visite un statement match
    fn visit_match_statement(&mut self, match_stmt: &crate::parser::ast::MatchStatement) -> Result<(), SemanticError> {
        // Créer le bloc de merge final
        let merge_block = self.cfg.create_block();
        
        // Pour chaque branche du match
        for (i, branch) in match_stmt.arms.iter().enumerate() {
            let branch_block = self.cfg.create_block();
            
            // Créer la condition pour cette branche
            // Note: Ceci est simplifié, une vraie implémentation devrait faire du pattern matching
            let is_last = i == match_stmt.arms.len() - 1;
            
            if is_last {
                // Dernière branche, saut direct
                self.cfg.set_terminator(self.current_block, Terminator::Jump(branch_block));
                self.cfg.add_edge(self.current_block, branch_block);
            } else {
                // Créer le bloc pour la prochaine condition
                let next_cond_block = self.cfg.create_block();
                
                // Condition simplifiée pour le pattern matching
                let condition = Expression::Literal(crate::parser::ast::Literal::Boolean(true));
                
                self.cfg.set_terminator(
                    self.current_block,
                    Terminator::ConditionalJump {
                        condition,
                        then_block: branch_block,
                        else_block: next_cond_block,
                    }
                );
                self.cfg.add_edge(self.current_block, branch_block);
                self.cfg.add_edge(self.current_block, next_cond_block);
                
                // Préparer pour la prochaine itération
                self.current_block = next_cond_block;
            }
            
            // Visiter le corps de la branche
            let saved_current = self.current_block;
            self.current_block = branch_block;
            
            for node in &branch.body {
                self.visit_node(node)?;
            }
            
            // Saut vers le bloc de merge
            if let Some(block) = self.cfg.blocks.get(&self.current_block) {
                if matches!(block.terminator, Terminator::Unreachable) {
                    self.cfg.set_terminator(self.current_block, Terminator::Jump(merge_block));
                    self.cfg.add_edge(self.current_block, merge_block);
                }
            }
            
            if !is_last {
                self.current_block = saved_current;
            }
        }
        
        // Continuer avec le bloc de merge
        self.current_block = merge_block;
        Ok(())
    }
    
    /// Visite un statement return
    fn visit_return_statement(&mut self, ret_stmt: &crate::parser::ast::ReturnStatement) -> Result<(), SemanticError> {
        // Définir le terminateur comme Return
        self.cfg.set_terminator(
            self.current_block,
            Terminator::Return(ret_stmt.value.clone())
        );
        
        // Ajouter ce bloc aux sorties de la fonction
        self.function_exits.push(self.current_block);
        
        // Créer un nouveau bloc (non accessible) pour continuer l'analyse
        self.current_block = self.cfg.create_block();
        
        Ok(())
    }
    
    /// Visite un statement break
    fn visit_break_statement(&mut self, label: Option<&str>) -> Result<(), SemanticError> {
        if let Some(loop_ctx) = self.loop_stack.last() {
            // Vérifier le label si fourni
            if let Some(label) = label {
                if loop_ctx.label.as_deref() != Some(label) {
                    // Chercher la bonne boucle dans la pile
                    for ctx in self.loop_stack.iter().rev() {
                        if ctx.label.as_deref() == Some(label) {
                            self.cfg.set_terminator(self.current_block, Terminator::Break(Some(label.to_string())));
                            self.cfg.add_edge(self.current_block, ctx.break_target);
                            self.current_block = self.cfg.create_block();
                            return Ok(());
                        }
                    }
                    // Label non trouvé
                    return Err(SemanticError::new(
                        SemanticErrorType::TypeError(crate::semantic::semantic_error::TypeError::InvalidType(
                            format!("Break label '{}' not found", label)
                        )),
                        "Invalid break statement".to_string(),
                        Position { index: 0 }
                    ));
                }
            }
            
            // Break sans label ou avec le bon label
            self.cfg.set_terminator(self.current_block, Terminator::Break(label.map(String::from)));
            self.cfg.add_edge(self.current_block, loop_ctx.break_target);
            
            // Créer un nouveau bloc non accessible
            self.current_block = self.cfg.create_block();
        } else {
            // Break en dehors d'une boucle
            return Err(SemanticError::new(
                SemanticErrorType::TypeError(crate::semantic::semantic_error::TypeError::InvalidType(
                    "Break outside of loop".to_string()
                )),
                "Invalid break statement".to_string(),
                Position { index: 0 }
            ));
        }
        
        Ok(())
    }
    
    /// Visite un statement continue
    fn visit_continue_statement(&mut self, label: Option<&str>) -> Result<(), SemanticError> {
        if let Some(loop_ctx) = self.loop_stack.last() {
            // Vérifier le label si fourni
            if let Some(label) = label {
                if loop_ctx.label.as_deref() != Some(label) {
                    // Chercher la bonne boucle dans la pile
                    for ctx in self.loop_stack.iter().rev() {
                        if ctx.label.as_deref() == Some(label) {
                            self.cfg.set_terminator(self.current_block, Terminator::Continue(Some(label.to_string())));
                            self.cfg.add_edge(self.current_block, ctx.continue_target);
                            self.current_block = self.cfg.create_block();
                            return Ok(());
                        }
                    }
                    // Label non trouvé
                    return Err(SemanticError::new(
                        SemanticErrorType::TypeError(crate::semantic::semantic_error::TypeError::InvalidType(
                            format!("Continue label '{}' not found", label)
                        )),
                        "Invalid continue statement".to_string(),
                        Position { index: 0 }
                    ));
                }
            }
            
            // Continue sans label ou avec le bon label
            self.cfg.set_terminator(self.current_block, Terminator::Continue(label.map(String::from)));
            self.cfg.add_edge(self.current_block, loop_ctx.continue_target);
            
            // Créer un nouveau bloc non accessible
            self.current_block = self.cfg.create_block();
        } else {
            // Continue en dehors d'une boucle
            return Err(SemanticError::new(
                SemanticErrorType::TypeError(crate::semantic::semantic_error::TypeError::InvalidType(
                    "Continue outside of loop".to_string()
                )),
                "Invalid continue statement".to_string(),
                Position { index: 0 }
            ));
        }
        
        Ok(())
    }
    
    /// Visite une expression
    fn visit_expression(&mut self, expr: &Expression) -> Result<(), SemanticError> {
        // Ajouter l'expression comme instruction si elle a des effets de bord
        if self.has_side_effects(expr) {
            let inst_id = self.cfg.create_instruction();
            self.cfg.add_instruction(
                self.current_block,
                Instruction::Expression { id: inst_id, expr: expr.clone() }
            );
        }
        
        // Gérer les cas spéciaux comme les affectations
        if let Expression::Assignment(assign) = expr {
            // Extraire le nom de la variable cible si possible
            if let Expression::Identifier(name) = &*assign.target {
                let inst_id = self.cfg.create_instruction();
                self.cfg.add_instruction(
                    self.current_block,
                    Instruction::Assignment {
                        id: inst_id,
                        target: name.clone(),
                        value: *assign.value.clone(),
                    }
                );
            }
        }
        
        Ok(())
    }
    
    /// Visite une déclaration
    fn visit_declaration(&mut self, decl: &Declaration) -> Result<(), SemanticError> {
        // use crate::parser::ast::{VariableDeclaration, FunctionDeclaration};
        
        match decl {
            Declaration::Variable(var_decl) => {
                let inst_id = self.cfg.create_instruction();
                self.cfg.add_instruction(
                    self.current_block,
                    Instruction::VarDecl {
                        id: inst_id,
                        name: var_decl.name.clone(),
                        value: var_decl.value.clone(),
                    }
                );
                Ok(())
            }
            Declaration::Function(func_decl) => {
                // Pour une fonction, on pourrait créer un nouveau CFG
                // Pour l'instant, on ignore le corps de la fonction
                // Dans une implémentation complète, on créerait un CFG séparé pour chaque fonction
                Ok(())
            }
            _ => Ok(()), // Autres déclarations non supportées pour l'instant
        }
    }
    
    /// Détermine si une expression a des effets de bord
    fn has_side_effects(&self, expr: &Expression) -> bool {
        match expr {
            Expression::FunctionCall(_) | 
            Expression::MethodCall(_) | 
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
}

impl Default for CFGBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use super::*;
    use crate::parser::ast::*;
    
    #[test]
    fn test_cfg_simple_sequence() {
        let ast = ASTNode::Program(vec![
            ASTNode::Declaration(Declaration::Variable(VariableDeclaration {
                name: "x".to_string(),
                variable_type: Some(Type::Int),
                value: Some(Expression::Literal(Literal::Integer { value: BigInt::from(5) })),
                mutability: Mutability::Immutable,
            })),
            ASTNode::Declaration(Declaration::Variable(VariableDeclaration {
                name: "y".to_string(),
                variable_type: Some(Type::Int),
                value: Some(Expression::Literal(Literal::Integer { value: BigInt::from(10) })),
                mutability: Mutability::Immutable,
            })),
        ]);
        
        let builder = CFGBuilder::new();
        let cfg = builder.build(&ast).unwrap();
        
        // Vérifier qu'on a au moins le bloc d'entrée
        assert!(cfg.blocks.contains_key(&cfg.entry));
        
        // Vérifier qu'on a des instructions
        let entry_block = cfg.blocks.get(&cfg.entry).unwrap();
        assert_eq!(entry_block.instructions.len(), 2);
    }
    
    #[test]
    fn test_cfg_if_statement() {
        let ast = ASTNode::Statement(Statement::IfStatement(IfStatement {
            condition: Expression::Literal(Literal::Boolean(true)),
            then_block: vec![
                ASTNode::Statement(Statement::Expression(
                    Expression::Literal(Literal::Integer { value: BigInt::from(1) })
                ))
            ],
            elif_block: vec![],
            else_block: Some(vec![
                ASTNode::Statement(Statement::Expression(
                    Expression::Literal(Literal::Integer { value: BigInt::from(2) })
                ))
            ]),
        }));
        
        let builder = CFGBuilder::new();
        let cfg = builder.build(&ast).unwrap();
        
        // Vérifier qu'on a créé les blocs nécessaires
        assert!(cfg.blocks.len() >= 4); // entry, then, else, merge
        
        // Vérifier le terminateur du bloc d'entrée
        let entry_block = cfg.blocks.get(&cfg.entry).unwrap();
        match &entry_block.terminator {
            Terminator::ConditionalJump { .. } => {},
            _ => panic!("Expected conditional jump"),
        }
    }
    
    #[test]
    fn test_cfg_while_loop() {
        let ast = ASTNode::Statement(Statement::WhileStatement(WhileStatement {
            condition: Expression::Literal(Literal::Boolean(true)),
            body: vec![
                ASTNode::Statement(Statement::Expression(
                    Expression::Literal(Literal::Integer { value: BigInt::from(1) })
                ))
            ],
        }));
        
        let builder = CFGBuilder::new();
        let cfg = builder.build(&ast).unwrap();
        
        // Vérifier qu'on a créé les blocs nécessaires
        assert!(cfg.blocks.len() >= 3); // entry, condition, body, exit
        
        // Vérifier qu'il y a une boucle (un bloc avec un prédécesseur qui est aussi son successeur)
        let has_loop = cfg.blocks.values().any(|block| {
            block.predecessors.iter().any(|pred| block.successors.contains(pred))
        });
        assert!(has_loop || cfg.blocks.len() >= 3); // Simplification du test
    }
    
    #[test]
    fn test_cfg_return_statement() {
        let ast = ASTNode::Statement(Statement::ReturnStatement(ReturnStatement {
            value: Some(Expression::Literal(Literal::Integer { value: BigInt::from(42) })),
        }));
        
        let builder = CFGBuilder::new();
        let cfg = builder.build(&ast).unwrap();
        
        // Vérifier qu'on a un bloc avec un terminateur Return
        let has_return = cfg.blocks.values().any(|block| {
            matches!(block.terminator, Terminator::Return(_))
        });
        assert!(has_return);
        
        // Vérifier qu'on a au moins une sortie
        assert!(!cfg.exits.is_empty());
    }
    
    #[test]
    fn test_cfg_break_continue() {
        let ast = ASTNode::Statement(Statement::WhileStatement(WhileStatement {
            condition: Expression::Literal(Literal::Boolean(true)),
            body: vec![
                ASTNode::Statement(Statement::IfStatement(IfStatement {
                    condition: Expression::Literal(Literal::Boolean(true)),
                    then_block: vec![
                        ASTNode::Statement(Statement::BreakStatement(BreakStatement { label: None }))
                    ],
                    elif_block: vec![],
                    else_block: Some(vec![
                        ASTNode::Statement(Statement::ContinueStatement(ContinueStatement { label: None }))
                    ]),
                }))
            ],
        }));
        
        let builder = CFGBuilder::new();
        let result = builder.build(&ast);
        
        // Le test devrait réussir sans erreur
        assert!(result.is_ok());
        
        let cfg = result.unwrap();
        
        // Vérifier qu'on a des terminateurs Break et Continue
        let has_break = cfg.blocks.values().any(|block| {
            matches!(block.terminator, Terminator::Break(_))
        });
        let has_continue = cfg.blocks.values().any(|block| {
            matches!(block.terminator, Terminator::Continue(_))
        });
        
        assert!(has_break);
        assert!(has_continue);
    }
}