use std::collections::{HashMap, HashSet};

use petgraph::{
    dot::Dot,
    graph::EdgeIndex,
    graph::NodeIndex,
    stable_graph::StableDiGraph,
    visit::{IntoEdgeReferences, IntoEdges, IntoNodeReferences},
};
use uuid::Uuid;

use crate::widgets::net_props::graph::{
    cycle::Cycle,
    elements::{Edge, Elements, Node},
};

#[derive(Default, Clone)]
pub struct Calculated {
    pub node_by_name: HashMap<String, Node>,
    pub edge_by_name: HashMap<String, Edge>,
    pub node_by_idx: HashMap<NodeIndex, Node>,
    pub edge_by_idx: HashMap<EdgeIndex, Edge>,
    pub idx_by_node_id: HashMap<Uuid, NodeIndex>,
    pub idx_by_edge_id: HashMap<Uuid, EdgeIndex>,
    pub ini_nodes: HashSet<Node>,
    pub fin_nodes: HashSet<Node>,
    pub dot: String,
    pub colored: Elements,
    // pub signal: Elements,
    // pub cycles: Vec<Cycle>,
    // pub adj_mat: MatrixState,
    // pub longest_path: usize,
}

impl Calculated {
    pub fn new(
        g: &StableDiGraph<Node, Edge>,
        fin: HashSet<Node>,
        ini: HashSet<Node>,
        colored: Elements,
    ) -> Calculated {
        let node_by_name = g
            .node_weights()
            .cloned()
            .map(|w| (w.name().clone(), w))
            .collect::<HashMap<_, _>>();
        let edge_by_name = g
            .edge_weights()
            .cloned()
            .map(|w| (w.name().clone(), w))
            .collect::<HashMap<_, _>>();

        let mut idx_by_node_id = HashMap::with_capacity(g.node_count());
        let mut node_by_idx = HashMap::with_capacity(g.node_count());
        g.node_references().for_each(|(idx, n)| {
            idx_by_node_id.insert(*n.id(), idx);
            node_by_idx.insert(idx, n.clone());
        });

        let mut idx_by_edge_id = HashMap::with_capacity(g.edge_count());
        let mut edge_by_idx = HashMap::with_capacity(g.edge_count());
        g.edge_indices().for_each(|idx| {
            let e = g.edge_weight(idx).unwrap();
            idx_by_edge_id.insert(*e.id(), idx);
            edge_by_idx.insert(idx, e.clone());
        });

        let dot = Dot::new(g).to_string();

        Calculated {
            fin_nodes: fin,
            ini_nodes: ini,
            colored,

            node_by_name,
            node_by_idx,
            idx_by_node_id,

            edge_by_name,
            edge_by_idx,
            idx_by_edge_id,

            dot,
        }
    }
}
