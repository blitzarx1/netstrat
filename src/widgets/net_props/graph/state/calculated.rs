use std::collections::{HashMap, HashSet};

use petgraph::{adj::EdgeIndex, graph::NodeIndex};
use uuid::Uuid;

use crate::widgets::net_props::graph::{
    cycle::Cycle,
    elements::{Edge, Elements, Node},
};

#[derive(Default, Clone)]
pub struct Calculated {
    pub nodes_by_name: HashMap<String, Node>,
    pub edges_by_name: HashMap<String, Edge>,
    pub nodes_by_id: HashMap<Uuid, Node>,
    pub edges_by_id: HashMap<Uuid, Edge>,
    pub idx_to_node_id: HashMap<NodeIndex, Uuid>,
    pub idx_to_edge_id: HashMap<EdgeIndex, Uuid>,
    pub ini: Elements,
    pub fin: Elements,
    pub colored: Elements,
    pub signal: Elements,
    pub dot: String,
    // pub cycles: Vec<Cycle>,
    // pub adj_mat: MatrixState,
    // pub longest_path: usize,
}
