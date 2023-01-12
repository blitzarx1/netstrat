use std::collections::{HashMap, HashSet};

use petgraph::stable_graph::StableDiGraph;
use rand::{distributions::Uniform, prelude::Distribution, rngs::ThreadRng, seq::IteratorRandom};
use tracing::debug;

use crate::{
    netstrat::Bus,
    widgets::{
        history::History,
        net_props::{
            graph::{
                elements::{Edge, Node},
                state::calculated::{self, Calculated},
            },
            settings::{EdgeWeight, Settings},
            Elements,
        },
    },
};

use super::State;

const PREFIX_INI: &str = "ini";
const PREFIX_FIN: &str = "fin";

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

        let mut graph = StableDiGraph::with_capacity(
            self.settings.total_cnt,
            self.settings.total_cnt * self.settings.total_cnt,
        );

        let mut node_by_id = HashMap::with_capacity(self.settings.total_cnt);
        let mut idx_by_id = HashMap::with_capacity(self.settings.total_cnt);
        (0..self.settings.total_cnt).for_each(|i| {
            let n = Node::new(format!("{i}"));
            let idx = graph.add_node(n.clone());

            node_by_id.insert(*n.id(), n.clone());
            idx_by_id.insert(*n.id(), idx);
        });

        let mut rng = rand::thread_rng();

        let ini_elements = self.pick_inis(&mut rng, &mut graph);

        let mut last_leafs = ini_elements.nodes().iter().cloned().collect::<Vec<_>>();
        let mut starts = HashSet::new();
        let mut ends = vec![];
        let max_degree_pool = Uniform::from(0..self.settings.max_out_degree);
        let max_degree_pool_ini = Uniform::from(1..self.settings.max_out_degree);
        let edge_weight_pool = Uniform::from(0.0..1.0);
        let mut edges_map = HashSet::<[usize; 2]>::new();

        // add edges
        loop {
            let mut next_last_ends = vec![];
            let mut started = 0;
            last_leafs.iter().for_each(|leaf| {
                if starts.contains(leaf) {
                    // add output edges only for nodes without output edges
                    return;
                }

                let leaf_idx = idx_by_id.get(leaf.id()).unwrap();

                starts.insert(leaf.clone());
                started += 1;

                let curr_degree = match ini_elements.nodes().contains(leaf) {
                    true => max_degree_pool_ini.sample(&mut rng),
                    false => max_degree_pool.sample(&mut rng),
                };

                // create edges
                for _i in 0..curr_degree {
                    let (end_id, end_idx) = idx_by_id.iter().choose(&mut rng).unwrap();

                    if self.settings.no_twin_edges
                        && edges_map.contains(&[leaf_idx.index(), end_idx.index()])
                    {
                        continue;
                    }

                    let mut weight = self.settings.edge_weight;
                    if self.settings.edge_weight_type == EdgeWeight::Random {
                        weight = edge_weight_pool.sample(&mut rng);
                    }

                    let end = node_by_id.get(end_id).unwrap();
                    let edge = Edge::new(&leaf, end, weight);

                    graph.add_edge(*leaf_idx, *end_idx, edge);

                    edges_map.insert([leaf_idx.index(), end_idx.index()]);

                    next_last_ends.push(end.clone());
                    ends.push(end.clone());
                }
            });

            last_leafs = next_last_ends;

            if started == 0 {
                break;
            }
        }

        // define fins
        let mut fin_elements_nodes = HashSet::with_capacity(self.settings.fin_cnt);
        for _i in 0..self.settings.fin_cnt {
            let end = ends.iter_mut().choose(&mut rng).unwrap();
            if fin_elements_nodes.contains(end) {
                continue;
            }

            let node_idx = idx_by_id.get(end.id()).unwrap();
            let graph_node = graph.node_weight_mut(*node_idx).unwrap();
            let new_name = format!("{}_{}", PREFIX_FIN, graph_node.name());
            graph_node.set_name(new_name);

            *end = graph_node.clone();

            fin_elements_nodes.insert(graph_node.clone());
        }

        let calculated = Calculated::new(
            &graph,
            Elements::new(fin_elements_nodes, Default::default()),
            ini_elements,
            Default::default(),
        );

        State::new(
            graph,
            History::new("create".to_string(), self.bus.clone()),
            calculated,
        )
    }

    fn pick_inis(&self, rng: &mut ThreadRng, g: &mut StableDiGraph<Node, Edge>) -> Elements {
        let mut ini_elements_nodes = HashSet::with_capacity(self.settings.ini_cnt);
        let mut ini_to_add = self.settings.ini_cnt;

        while ini_to_add > 0 {
            let idx = g.node_indices().choose(rng).unwrap();
            let n = g.node_weight_mut(idx).unwrap();

            if ini_elements_nodes.contains(n) {
                continue;
            }

            let new_name = format!("{}_{}", PREFIX_INI, n.name());
            n.set_name(new_name);

            ini_elements_nodes.insert(n.clone());
            ini_to_add -= 1;

            debug!("picked ini node: {:?}", *n);
        }

        Elements::new(ini_elements_nodes, Default::default())
    }
}
