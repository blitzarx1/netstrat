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

use super::{PREFIX_FIN, PREFIX_INI};

const MAX_DOT_WEIGHT: f64 = 2.0;
const MIN_DOT_WEIGHT: f64 = 0.5;

#[derive(Default, Clone)]
pub struct Metadata {
    // store
    pub node_by_name: HashMap<String, Node>,
    pub edge_by_name: HashMap<String, Edge>,
    pub node_by_idx: HashMap<NodeIndex, Node>,
    pub edge_by_idx: HashMap<EdgeIndex, Edge>,
    pub idx_by_node_id: HashMap<Uuid, NodeIndex>,
    pub idx_by_edge_id: HashMap<Uuid, EdgeIndex>,
    pub ini_nodes: HashSet<Node>,
    pub fin_nodes: HashSet<Node>,

    pub colored: Elements,

    // calculated
    pub dot: String,
    // pub signal: Elements,
    // pub cycles: Vec<Cycle>,
    // pub adj_mat: MatrixState,
    // pub longest_path: usize,
}

impl Metadata {
    pub fn new(
        g: &StableDiGraph<Node, Edge>,
        fin: HashSet<Node>,
        ini: HashSet<Node>,
        colored: Elements,
    ) -> Metadata {
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

        let mut res = Metadata {
            fin_nodes: fin,
            ini_nodes: ini,
            colored,

            node_by_name,
            node_by_idx,
            idx_by_node_id,

            edge_by_name,
            edge_by_idx,
            idx_by_edge_id,

            dot: Default::default(),
        };

        res.recalculate(g);
        res
    }

    pub fn recalculate(&mut self, g: &StableDiGraph<Node, Edge>) {
        self.dot = self.calc_dot(g)
    }

    pub fn delete_node(&mut self, node: &Node) {
        let idx = self.idx_by_node_id[node.id()];

        self.node_by_idx.get_mut(&idx).unwrap().mark_deleted();

        self.node_by_name
            .get_mut(node.name())
            .unwrap()
            .mark_deleted();

        let mut node_copy = node.clone();
        node_copy.mark_deleted();

        self.ini_nodes.take(node);
        self.ini_nodes.insert(node_copy.clone());

        self.fin_nodes.take(node);
        self.fin_nodes.insert(node_copy);
    }

    pub fn delete_edge(&mut self, edge: &Edge) {
        let idx = self.idx_by_edge_id[edge.id()];

        self.edge_by_idx.get_mut(&idx).unwrap().mark_deleted();

        self.edge_by_name
            .get_mut(edge.name())
            .unwrap()
            .mark_deleted();
    }

    pub fn restore_node(&mut self, id: &Uuid, idx: &NodeIndex) {
        let old_idx = self.idx_by_node_id.remove(id).unwrap();
        let mut node = self.node_by_idx.remove(&old_idx).unwrap();

        let name = node.name().clone();
        if name.contains(PREFIX_INI) {
            self.ini_nodes.take(&node);
        }
        if name.contains(PREFIX_FIN) {
            self.fin_nodes.take(&node);
        }

        node.restore();

        self.idx_by_node_id.insert(*id, *idx);
        self.node_by_idx.insert(*idx, node.clone());

        self.node_by_name.insert(name.clone(), node.clone());
        if name.contains(PREFIX_INI) {
            self.ini_nodes.insert(node.clone());
        }
        if name.contains(PREFIX_FIN) {
            self.fin_nodes.insert(node);
        }
    }

    pub fn restore_edge(&mut self, id: &Uuid, idx: &EdgeIndex) {
        let old_idx = self.idx_by_edge_id.remove(id).unwrap();
        let mut edge = self.edge_by_idx.remove(&old_idx).unwrap();

        edge.restore();

        self.idx_by_edge_id.insert(*id, *idx);
        self.edge_by_idx.insert(*idx, edge.clone());
        self.edge_by_name.insert(edge.name().clone(), edge);
    }

    pub fn color(&mut self, elements: &Elements) {
        // TODO: maybe move this to Element attributes?
        self.colored = elements.clone();
    }

    fn calc_dot(&self, g: &StableDiGraph<Node, Edge>) -> String {
        let max_weight = g
            .edge_weights()
            .map(|e| e.weight())
            .max_by(|left, right| left.partial_cmp(right).unwrap())
            .unwrap();

        Dot::with_attr_getters(
            g,
            &[],
            &|g, r| {
                let mut attrs = vec![];

                if self.colored.edges().contains(r.weight()) {
                    attrs.push("color=red".to_string())
                }

                let weight = r.weight().weight();
                let mut normed = (weight / max_weight) * MAX_DOT_WEIGHT;
                if normed < MIN_DOT_WEIGHT {
                    normed = MIN_DOT_WEIGHT
                }
                attrs.push(format!("penwidth={}", normed));

                attrs.join(", ")
            },
            &|g, r| {
                let mut attrs = vec![];
                if self.colored.nodes().contains(r.1) {
                    attrs.push("color=red".to_string())
                }

                attrs.join(", ")
            },
        )
        .to_string()
    }
}
