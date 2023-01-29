use std::collections::{HashMap, HashSet};

use petgraph::{dot::Dot, graph::EdgeIndex, graph::NodeIndex, visit::IntoNodeReferences};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::widgets::net_props::{
    graph::elements::{Edge, Elements, Node},
    Graph,
};

const MAX_DOT_WEIGHT: f64 = 2.0;
const MIN_DOT_WEIGHT: f64 = 0.5;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub ini_nodes: HashSet<Uuid>,
    pub fin_nodes: HashSet<Uuid>,

    pub node_by_name: HashMap<String, Uuid>,
    pub edge_by_name: HashMap<String, Uuid>,

    pub idx_by_node_id: HashMap<Uuid, NodeIndex>,
    pub idx_by_edge_id: HashMap<Uuid, EdgeIndex>,

    pub selected: Elements,
    pub elements: Elements,

    // calculated
    pub dot: String,
    // pub cycles: Vec<Cycle>,
    // pub adj_mat: MatrixState,
    // pub longest_path: usize,
}

impl Metadata {
    pub fn new(g: &Graph, fin: HashSet<Uuid>, ini: HashSet<Uuid>) -> Metadata {
        let node_by_id = g
            .node_weights()
            .cloned()
            .map(|w| (*w.id(), w))
            .collect::<HashMap<_, _>>();
        let edge_by_id = g
            .edge_weights()
            .cloned()
            .map(|w| (*w.id(), w))
            .collect::<HashMap<_, _>>();

        let node_by_name = g
            .node_weights()
            .cloned()
            .map(|w| (w.name().clone(), *w.id()))
            .collect::<HashMap<_, _>>();
        let edge_by_name = g
            .edge_weights()
            .cloned()
            .map(|w| (w.name().clone(), *w.id()))
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

            node_by_name,
            idx_by_node_id,

            edge_by_name,
            idx_by_edge_id,

            selected: Default::default(),
            elements: Default::default(),

            dot: Default::default(),
        };

        res.recalculate(g);
        res
    }

    pub fn recalculate(&mut self, g: &Graph) {
        self.dot = self.calc_dot(g)
    }

    fn calc_dot(&self, g: &Graph) -> String {
        let max_weight = g
            .edge_weights()
            .map(|e| e.weight())
            .max_by(|left, right| left.partial_cmp(right).unwrap())
            .unwrap();

        Dot::with_attr_getters(
            g,
            &[],
            &|_g, r| {
                let edge = r.weight();
                let mut attrs = vec![];

                let weight = edge.weight();
                let mut normed = (weight / max_weight) * MAX_DOT_WEIGHT;
                if normed < MIN_DOT_WEIGHT {
                    normed = MIN_DOT_WEIGHT
                }
                attrs.push(format!("penwidth={}", normed));

                let mut color = "black".to_string();
                if edge.deleted() {
                    color = "lightgray".to_string();
                }
                if !edge.deleted() && edge.selected() {
                    color = "red".to_string();
                }
                attrs.push(format!("color={}", color));
                attrs.push(format!("fontcolor={}", color));

                attrs.join(", ")
            },
            &|_g, r| {
                let node = r.1;
                let mut attrs = vec![];

                let mut color = "black".to_string();
                if node.deleted() {
                    color = "lightgray".to_string();
                }
                if !node.deleted() && node.selected() {
                    color = "red".to_string();
                }
                attrs.push(format!("color={}", color));
                attrs.push(format!("fontcolor={}", color));

                attrs.join(", ")
            },
        )
        .to_string()
    }
}
