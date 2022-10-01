use petgraph::graph::{EdgeIndex, NodeIndex};
use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum GraphError {
        NodeNotFound(idx: NodeIndex) {
            display("failed to find node by idx: {:?}", idx)
        }
        EdgeNotFound(idx: EdgeIndex) {
            display("failed to find edge by idx: {:?}", idx)
        }
    }
}
