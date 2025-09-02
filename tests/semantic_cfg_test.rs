// tests/semantic_cfg_test.rs
// Tests pour le Control Flow Graph basés sur l'API réelle

use punk::semantic::flow::control_flow_graph::{
    ControlFlowGraph, BlockId, BasicBlock, Instruction, 
    Terminator, InstructionId, CFGInstruction
};
use punk::semantic::symbols::SymbolId;
use punk::parser::ast::{Expression, Literal};
use std::collections::HashSet;

#[cfg(test)]
mod cfg_tests {
    use super::*;

    #[test]
    fn test_cfg_creation() {
        let cfg = ControlFlowGraph::new();
        
        // Le CFG devrait avoir un bloc d'entrée
        assert!(cfg.blocks.contains_key(&cfg.entry));
        assert_eq!(cfg.blocks.len(), 1);
    }

    #[test]
    fn test_create_blocks() {
        let mut cfg = ControlFlowGraph::new();
        
        let block1 = cfg.create_block();
        let block2 = cfg.create_block();
        let block3 = cfg.create_block();
        
        // Les blocs devraient avoir des IDs différents
        assert_ne!(block1, block2);
        assert_ne!(block2, block3);
        assert_ne!(block1, block3);
        
        // Tous les blocs devraient exister
        assert!(cfg.blocks.contains_key(&block1));
        assert!(cfg.blocks.contains_key(&block2));
        assert!(cfg.blocks.contains_key(&block3));
        
        // Total: 4 blocs (entrée + 3 créés)
        assert_eq!(cfg.blocks.len(), 4);
    }

    #[test]
    fn test_add_edge() {
        let mut cfg = ControlFlowGraph::new();
        
        let block1 = cfg.create_block();
        let block2 = cfg.create_block();
        
        // Ajouter une edge
        cfg.add_edge(block1, block2);
        
        // Vérifier les successeurs et prédécesseurs
        let successors = cfg.get_successors(block1);
        let predecessors = cfg.get_predecessors(block2);
        
        assert!(successors.contains(&block2));
        assert!(predecessors.contains(&block1));
    }

    #[test]
    fn test_remove_edge() {
        let mut cfg = ControlFlowGraph::new();
        
        let block1 = cfg.create_block();
        let block2 = cfg.create_block();
        
        cfg.add_edge(block1, block2);
        cfg.remove_edge(block1, block2);
        
        let successors = cfg.get_successors(block1);
        let predecessors = cfg.get_predecessors(block2);
        
        assert!(!successors.contains(&block2));
        assert!(!predecessors.contains(&block1));
    }

    #[test]
    fn test_add_instruction() {
        let mut cfg = ControlFlowGraph::new();
        
        let block = cfg.create_block();
        let instr_id = cfg.create_instruction();
        
        let instruction = Instruction::VarDecl {
            id: instr_id,
            name: "x".to_string(),
            value: Some(Expression::Literal(Literal::Integer { value: 42.into() })),
        };
        
        cfg.add_instruction(block, instruction);
        
        // Vérifier que l'instruction a été ajoutée
        if let Some(basic_block) = cfg.get_block(block) {
            assert_eq!(basic_block.instructions.len(), 1);
            match &basic_block.instructions[0] {
                Instruction::VarDecl { name, .. } => {
                    assert_eq!(name, "x");
                }
                _ => panic!("Wrong instruction type"),
            }
        } else {
            panic!("Block not found");
        }
    }

    #[test]
    fn test_set_terminator() {
        let mut cfg = ControlFlowGraph::new();
        
        let block1 = cfg.create_block();
        let block2 = cfg.create_block();
        
        cfg.set_terminator(block1, Terminator::Jump(block2));
        
        if let Some(basic_block) = cfg.get_block(block1) {
            match &basic_block.terminator {
                Terminator::Jump(target) => {
                    assert_eq!(*target, block2);
                }
                _ => panic!("Wrong terminator type"),
            }
        }
    }

    #[test]
    fn test_conditional_jump() {
        let mut cfg = ControlFlowGraph::new();
        
        let condition_block = cfg.create_block();
        let then_block = cfg.create_block();
        let else_block = cfg.create_block();
        
        let condition = Expression::Literal(Literal::Boolean(true));
        
        cfg.set_terminator(
            condition_block,
            Terminator::ConditionalJump {
                condition,
                then_block,
                else_block,
            },
        );
        
        cfg.add_edge(condition_block, then_block);
        cfg.add_edge(condition_block, else_block);
        
        // Vérifier que les deux branches sont des successeurs
        let successors = cfg.get_successors(condition_block);
        assert_eq!(successors.len(), 2);
        assert!(successors.contains(&then_block));
        assert!(successors.contains(&else_block));
    }

    #[test]
    fn test_compute_reachable_blocks() {
        let mut cfg = ControlFlowGraph::new();
        
        let block1 = cfg.create_block();
        let block2 = cfg.create_block();
        let unreachable = cfg.create_block();
        
        cfg.add_edge(cfg.entry, block1);
        cfg.add_edge(block1, block2);
        // unreachable n'est connecté à rien
        
        let reachable = cfg.compute_reachable_blocks();
        
        assert!(reachable.contains(&cfg.entry));
        assert!(reachable.contains(&block1));
        assert!(reachable.contains(&block2));
        assert!(!reachable.contains(&unreachable));
    }

    #[test]
    fn test_find_dead_blocks() {
        let mut cfg = ControlFlowGraph::new();
        
        let block1 = cfg.create_block();
        let block2 = cfg.create_block();
        let dead_block = cfg.create_block();
        
        cfg.add_edge(cfg.entry, block1);
        cfg.add_edge(block1, block2);
        // dead_block n'est pas atteignable
        
        let dead_blocks = cfg.find_dead_blocks();
        
        assert!(dead_blocks.contains(&dead_block));
        assert!(!dead_blocks.contains(&block1));
        assert!(!dead_blocks.contains(&block2));
    }

    #[test]
    fn test_topological_sort() {
        let mut cfg = ControlFlowGraph::new();
        
        let block1 = cfg.create_block();
        let block2 = cfg.create_block();
        let block3 = cfg.create_block();
        
        cfg.add_edge(cfg.entry, block1);
        cfg.add_edge(block1, block2);
        cfg.add_edge(block2, block3);
        
        let sorted = cfg.topological_sort();
        
        // L'entrée devrait être première
        assert_eq!(sorted[0], cfg.entry);
        
        // block1 devrait venir avant block2
        let pos1 = sorted.iter().position(|&b| b == block1).unwrap();
        let pos2 = sorted.iter().position(|&b| b == block2).unwrap();
        let pos3 = sorted.iter().position(|&b| b == block3).unwrap();
        
        assert!(pos1 < pos2);
        assert!(pos2 < pos3);
    }

    #[test]
    fn test_get_all_blocks() {
        let mut cfg = ControlFlowGraph::new();
        
        let block1 = cfg.create_block();
        let block2 = cfg.create_block();
        
        let all_blocks = cfg.get_all_blocks();
        
        assert!(all_blocks.contains(&cfg.entry));
        assert!(all_blocks.contains(&block1));
        assert!(all_blocks.contains(&block2));
        assert_eq!(all_blocks.len(), 3);
    }

    #[test]
    fn test_return_terminator() {
        let mut cfg = ControlFlowGraph::new();
        
        let block = cfg.create_block();
        let return_value = Expression::Literal(Literal::Integer { value: 42.into() });
        
        cfg.set_terminator(block, Terminator::Return(Some(return_value)));
        cfg.exits.push(block);
        
        // Vérifier que le bloc est marqué comme sortie
        assert!(cfg.exits.contains(&block));
        
        if let Some(basic_block) = cfg.get_block(block) {
            match &basic_block.terminator {
                Terminator::Return(Some(_)) => {
                    // OK
                }
                _ => panic!("Wrong terminator type"),
            }
        }
    }

    #[test]
    fn test_loop_structure() {
        let mut cfg = ControlFlowGraph::new();
        
        let loop_header = cfg.create_block();
        let loop_body = cfg.create_block();
        let loop_exit = cfg.create_block();
        
        // Créer la structure de boucle
        cfg.add_edge(cfg.entry, loop_header);
        cfg.add_edge(loop_header, loop_body);
        cfg.add_edge(loop_body, loop_header); // Back edge
        cfg.add_edge(loop_header, loop_exit);
        
        // Vérifier les prédécesseurs du header (entry et body)
        let header_preds = cfg.get_predecessors(loop_header);
        assert_eq!(header_preds.len(), 2);
        assert!(header_preds.contains(&cfg.entry));
        assert!(header_preds.contains(&loop_body));
    }

    #[test]
    fn test_cfg_instruction_types() {
        // Test des types d'instructions CFG pour le borrow checker
        let symbol1 = SymbolId(1);
        let symbol2 = SymbolId(2);
        
        let assign = CFGInstruction::Assign {
            target: symbol1,
            value: Expression::Literal(Literal::Integer { value: 42.into() }),
        };
        
        let read = CFGInstruction::Read(symbol1);
        let write = CFGInstruction::Write(symbol1);
        
        let borrow = CFGInstruction::Borrow {
            target: symbol2,
            source: symbol1,
            is_mutable: false,
        };
        
        let move_instr = CFGInstruction::Move {
            target: symbol2,
            source: symbol1,
        };
        
        // Vérifier le pattern matching
        match assign {
            CFGInstruction::Assign { target, .. } => {
                assert_eq!(target, symbol1);
            }
            _ => panic!("Wrong instruction type"),
        }
        
        match borrow {
            CFGInstruction::Borrow { is_mutable, .. } => {
                assert!(!is_mutable);
            }
            _ => panic!("Wrong instruction type"),
        }
    }
}