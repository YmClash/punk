pub mod cfg;
pub mod analyser;
pub mod detector;
pub mod control_flow_graph;

// Re-export des types importants
pub use control_flow_graph::{BlockId, BasicBlock, ControlFlowGraph};