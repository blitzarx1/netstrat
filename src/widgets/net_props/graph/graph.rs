use petgraph::stable_graph::StableDiGraph;

use super::elements::{Node, Edge};

pub type Graph = StableDiGraph<Node, Edge>;
