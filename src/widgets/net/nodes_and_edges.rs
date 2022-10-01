use super::{nodes_input::NodesInput, edges_input::EdgesInput};

#[derive(PartialEq, Clone, Default)]
pub struct NodesAndEdgeSettings {
    pub nodes_input: NodesInput,
    pub edges_input: EdgesInput,
}
