use petgraph::stable_graph::StableDiGraph;

use super::elements::{Edge, Node};

/// Graph structure used in project
pub type Graph = StableDiGraph<Node, Edge>;
