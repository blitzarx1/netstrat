use super::{edges_input::EdgesInput, nodes_input::NodesInput};

#[derive(PartialEq, Clone, Default)]
pub struct NodesAndEdgeSettings {
    pub nodes_input: NodesInput,
    pub edges_input: EdgesInput,
}
