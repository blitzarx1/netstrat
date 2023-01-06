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
                state::calculated::Calculated,
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

        let node_to_idx = (0..self.settings.total_cnt)
            .map(|i| {
                let n = Node::new(format!("{i}"));
                (n, graph.add_node(n))
            })
            .collect::<HashMap<_, _>>();

        let mut rng = rand::thread_rng();

        let ini_elements = self.pick_inis(rng, graph);

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

                let leaf_idx = node_to_idx.get(&leaf).unwrap();

                starts.insert(*leaf);
                started += 1;

                let curr_degree = match ini_elements.nodes().contains(leaf) {
                    true => max_degree_pool_ini.sample(&mut rng),
                    false => max_degree_pool.sample(&mut rng),
                };

                // create edges
                for _i in 0..curr_degree {
                    let (end, end_idx) = node_to_idx.iter().choose(&mut rng).unwrap();

                    if self.settings.no_twin_edges
                        && edges_map.contains(&[leaf_idx.index(), end_idx.index()])
                    {
                        continue;
                    }

                    let mut weight = self.settings.edge_weight;
                    if self.settings.edge_weight_type == EdgeWeight::Random {
                        weight = edge_weight_pool.sample(&mut rng);
                    }

                    let edge = Edge::new(&leaf, end, weight);

                    graph.add_edge(*leaf_idx, *end_idx, edge);

                    edges_map.insert([leaf_idx.index(), end_idx.index()]);

                    next_last_ends.push(*end);
                    ends.push(*end);
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
            let end = ends.iter().choose(&mut rng).unwrap(); 

            if fin_elements_nodes.contains(end) {
                continue;
            }

            let node_idx = node_to_idx.get(end).unwrap();
            let graph_node = graph.node_weight_mut(*node_idx).unwrap();

            let new_name = format!("fin_{}", graph_node.name());

            graph_node.set_name(new_name);
            end.set_name(new_name);

            fin_elements_nodes.insert(*graph_node);
        }

        let nodes_by_name = graph
            .node_weights()
            .cloned()
            .map(|w| (*w.name(), w))
            .collect::<HashMap<_, _>>();
        let edges_by_name = graph
            .edge_weights()
            .cloned()
            .map(|w| (*w.name(), w))
            .collect::<HashMap<_, _>>();

        let calculated = Calculated {
            fin: Elements::new(fin_elements_nodes, Default::default()),
            ini: ini_elements,
            nodes_by_name,
            edges_by_name,
            ..Default::default()
        };

        let history = History::new("create".to_string(), self.bus.clone());

        let mut state = State::new(graph, history, calculated);

        if self.settings.diamond_filter {
            state.diamond_filter()
        }

        state.recalculate_metadata();

        state
    }

    fn pick_inis(&self, rng: ThreadRng, g: StableDiGraph<Node, Edge>) -> Elements {
        let mut ini_elements_nodes = HashSet::with_capacity(self.settings.ini_cnt);
        let mut ini_to_add = self.settings.ini_cnt;

        while ini_to_add > 0 {
            let idx = g.node_indices().choose(&mut rng).unwrap();
            let n = g.node_weight_mut(idx).unwrap();

            if ini_elements_nodes.contains(n) {
                continue;
            }

            let new_name = format!("{}_{}", PREFIX_INI, n.name());
            n.set_name(new_name);

            ini_elements_nodes.insert(*n);
            ini_to_add -= 1;

            debug!("picked ini node: {:?}", *n);
        }

        Elements::new(ini_elements_nodes, Default::default())
    }
}
