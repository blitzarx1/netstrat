use std::collections::HashSet;

use petgraph::{
    stable_graph::NodeIndex,
    visit::EdgeRef,
    Direction::{Incoming, Outgoing},
};
use rand::{distributions::Uniform, prelude::Distribution, rngs::ThreadRng, seq::IteratorRandom};
use tracing::debug;

use crate::{
    netstrat::Bus,
    widgets::net_props::{
        graph::{
            elements::{Edge, Node},
            state::metadata::Metadata,
        },
        settings::{EdgeWeight, Settings},
        Graph,
    },
};

use super::State;

pub const PREFIX_INI: &str = "ini";
pub const PREFIX_FIN: &str = "fin";

#[derive(Default, Clone)]
pub struct Builder {
    settings: Settings,
    bus: Bus,
}

impl Builder {
    pub fn new(bus: Bus) -> Builder {
        Builder {
            settings: Default::default(),
            bus,
        }
    }

    pub fn with_settings(&mut self, settings: Settings) -> Builder {
        self.settings = settings;
        self.clone()
    }

    pub fn build(&self) -> State {
        debug!("building graph state with settings: {:?}", self.settings);

        let mut g: Graph = Graph::with_capacity(
            self.settings.total_cnt,
            self.settings.total_cnt * self.settings.total_cnt,
        );

        // add nodes to graph
        (0..self.settings.total_cnt).for_each(|i| {
            g.add_node(Node::new(format!("{i}")));
        });

        let mut rng = rand::thread_rng();
        let ini_nodes = self.pick_inis(&mut g, &mut rng);
        let ends = self.add_edges(&mut g, &mut rng, &ini_nodes);
        self.pick_fins(&mut g, &mut rng, ends);

        let fin_nodes_set = g
            .node_weights()
            .cloned()
            .filter(|w| w.name().contains(PREFIX_FIN))
            .map(|n| n.id().clone())
            .collect::<HashSet<_>>();

        let ini_nodes_set = g
            .node_weights()
            .cloned()
            .filter(|w| w.name().contains(PREFIX_INI))
            .map(|n| n.id().clone())
            .collect::<HashSet<_>>();

        let calculated = Metadata::new(&g, fin_nodes_set, ini_nodes_set);

        let mut state = State::new(g, self.bus.clone(), calculated);

        if self.settings.diamond_filter {
            state.diamond_filter()
        }

        state
    }

    fn pick_inis(&self, g: &mut Graph, r: &mut ThreadRng) -> HashSet<NodeIndex> {
        let mut ini_nodes = HashSet::with_capacity(self.settings.ini_cnt);
        let mut ini_to_add = self.settings.ini_cnt;
        while ini_to_add > 0 {
            let idx = g.node_indices().choose(r).unwrap();
            if ini_nodes.contains(&idx) {
                continue;
            }

            let w = g.node_weight_mut(idx).unwrap();
            let new_name = format!("{}_{}", PREFIX_INI, w.name());

            w.set_name(new_name);
            ini_nodes.insert(idx);
            ini_to_add -= 1;
        }

        ini_nodes
    }

    fn add_edges(
        &self,
        g: &mut Graph,
        rng: &mut ThreadRng,
        seed: &HashSet<NodeIndex>,
    ) -> Vec<NodeIndex> {
        let mut ends = vec![];
        let max_degree_pool = Uniform::from(0..self.settings.max_out_degree);
        let edge_weight_pool = Uniform::from(0.0..1.0);
        let mut last_leafs = seed.iter().cloned().collect::<Vec<_>>();
        let mut starts = HashSet::new();
        let mut edges = HashSet::<[usize; 2]>::new();
        // TODO: use in statistics for longest path
        let mut path_len = 0;
        loop {
            let mut next_last_ends = vec![];
            let mut started = 0;
            last_leafs.iter().for_each(|leaf_idx| {
                if starts.contains(leaf_idx) {
                    // add output edges only for nodes without output edges
                    return;
                }

                starts.insert(*leaf_idx);
                started += 1;

                let rnd = max_degree_pool.sample(rng);
                // make sure we have at least one edge from ini node
                let curr_degree = match rnd == 0 && path_len == 0 {
                    true => 1,
                    false => rnd,
                };

                let leaf_node = g.node_weight(*leaf_idx).unwrap().clone();

                // create edges
                (0..curr_degree).for_each(|_| {
                    let end_idx = g.node_indices().choose(rng).unwrap();

                    // do not allow twin edges
                    if edges.contains(&[leaf_idx.index(), end_idx.index()]) {
                        return;
                    }

                    let mut new_edge_weight = self.settings.edge_weight;
                    if self.settings.edge_weight_type == EdgeWeight::Random {
                        new_edge_weight = edge_weight_pool.sample(rng);
                    }

                    let end_node = &g[end_idx];
                    let edge = Edge::new(leaf_node.id(), end_node.id(), new_edge_weight);

                    g.add_edge(*leaf_idx, end_idx, edge);
                    edges.insert([leaf_idx.index(), end_idx.index()]);
                    next_last_ends.push(end_idx);
                    ends.push(end_idx);
                });
            });

            last_leafs = next_last_ends;

            if started == 0 {
                break;
            }

            path_len += 1
        }

        ends
    }

    fn pick_fins(&self, g: &mut Graph, rng: &mut ThreadRng, ends: Vec<NodeIndex>) {
        let mut fin_nodes = HashSet::with_capacity(self.settings.fin_cnt);
        (0..self.settings.fin_cnt).for_each(|_| {
            let end_idx = ends.iter().choose(rng).unwrap();
            if fin_nodes.contains(end_idx) {
                return;
            }

            let w = &mut g[*end_idx];
            let new_name = format!("{}_{}", PREFIX_FIN, w.name());
            w.set_name(new_name);

            // need to change name of edges as well
            let mut affected_edges = vec![];
            g.edges_directed(*end_idx, Outgoing).for_each(|e_ref| {
                affected_edges.push(e_ref.id());
            });
            g.edges_directed(*end_idx, Incoming).for_each(|e_ref| {
                affected_edges.push(e_ref.id());
            });

            affected_edges.iter().for_each(|e_idx| {
                let (source_idx, target_idx) = g.edge_endpoints(*e_idx).unwrap();
                let source = g.node_weight(source_idx).cloned().unwrap();
                let target = g.node_weight(target_idx).cloned().unwrap();

                let e_weight = g.edge_weight_mut(*e_idx).unwrap();
                *e_weight = Edge::new_with_id(
                    e_weight.id().id,
                    source.id(),
                    target.id(),
                    e_weight.weight(),
                );
            });

            fin_nodes.insert(*end_idx);
        });
    }
}
