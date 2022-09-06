use futures::StreamExt;
use petgraph::data::DataMapMut;
use petgraph::dot::Dot;
use petgraph::graph::{Edge, EdgeReference, NodeIndex};
use petgraph::prelude::EdgeRef;
use petgraph::visit::GetAdjacencyMatrix;
use petgraph::{Graph, Incoming, Outgoing};
use rand::distributions::{Distribution, Uniform};
use rand::prelude::IteratorRandom;
use rand::rngs::OsRng;
use rand::seq::SliceRandom;
use serde_json::from_str;
use std::collections::{HashMap, HashSet};
use std::ops::{Add, Sub};

pub struct Data {
    seed: Graph<String, f64>,
    all_nodes: HashSet<NodeIndex>,
    ini_set: HashSet<NodeIndex>,
    fin_set: HashSet<NodeIndex>,
}

impl Data {
    pub fn new(ini_cnt: usize, fin_cnt: usize, total_cnt: usize, max_out_degree: usize) -> Self {
        let mut seed = Graph::with_capacity(total_cnt, total_cnt * total_cnt);
        let mut all_nodes = HashSet::with_capacity(total_cnt);
        for i in 0..total_cnt {
            let node_idx = seed.add_node(format!("{i}"));
            all_nodes.insert(node_idx);
        }

        let mut rng = rand::thread_rng();
        let mut ini_set = HashSet::with_capacity(ini_cnt);
        let mut ini_to_add = ini_cnt;

        // pick inis
        while ini_to_add > 0 {
            let idx = seed.node_indices().choose(&mut rng).unwrap();

            if ini_set.contains(&idx) {
                continue;
            }

            let weight = seed.node_weight_mut(idx).unwrap();
            let new_weight = format!("ini_{}", *weight);
            *weight = new_weight.clone();

            ini_set.insert(idx);
            ini_to_add -= 1;
        }

        let mut last_ends = ini_set.iter().cloned().collect::<Vec<NodeIndex>>();
        let mut starts = HashSet::new();
        let mut ends = vec![];
        let max_degree_pool = Uniform::from(1..max_out_degree);

        // add edges
        loop {
            let mut next_last_ends = vec![];
            let mut started = 0;
            last_ends.iter().for_each(|last_end| {
                if starts.contains(last_end) {
                    // add output edges only for nodes without output edges
                    return;
                }

                starts.insert(*last_end);
                started += 1;

                let curr_degree = max_degree_pool.sample(&mut rng);
                // add output edges
                for _i in 0..curr_degree {
                    let end = all_nodes.iter().choose(&mut rng).unwrap();
                    seed.add_edge(*last_end, *end, 1.0);

                    next_last_ends.push(*end);
                    ends.push(*end);
                }
            });

            last_ends = next_last_ends;

            if started == 0 {
                break;
            }
        }

        let mut fin_set = HashSet::with_capacity(fin_cnt);
        // define fins
        for _i in 0..fin_cnt {
            let idx = ends.iter().choose(&mut rng).unwrap();

            if fin_set.contains(idx) {
                continue;
            }

            let weight = seed.node_weight_mut(*idx).unwrap();
            let new_weight = format!("fin_{}", *weight);
            *weight = new_weight.clone();

            fin_set.insert(*idx);
        }

        Self {
            seed,
            all_nodes,
            ini_set,
            fin_set,
        }
    }

    pub fn dot(&self) -> String {
        format!("{}", Dot::new(&self.seed))
    }

    pub fn diamond_filter(&mut self) {
        let mut ini_diamond = HashSet::new();
        // gather diamond of all children of inis
        for el in self.ini_set.clone() {
            ini_diamond.insert(el);

            let mut children = self
                .seed
                .edges_directed(el, Outgoing)
                .map(|el| el.target())
                .collect::<Vec<NodeIndex>>();
            while !children.is_empty() {
                let mut next_children = vec![];

                children.drain(..).for_each(|child| {
                    if ini_diamond.contains(&child) {
                        return;
                    }
                    ini_diamond.insert(child);

                    let mut new_children = self
                        .seed
                        .edges_directed(child, Outgoing)
                        .map(|edge| edge.target())
                        .collect::<Vec<NodeIndex>>();

                    next_children.append(&mut new_children);
                });

                children = next_children;
            }
        }

        let mut fin_diamond = HashSet::new();
        // gather diamond of all parents of fins
        for el in self.fin_set.clone() {
            fin_diamond.insert(el);

            let mut parents = self
                .seed
                .edges_directed(el, Incoming)
                .map(|el| el.source())
                .collect::<Vec<NodeIndex>>();
            while !parents.is_empty() {
                let mut next_parents = vec![];

                parents.drain(..).for_each(|parent| {
                    if fin_diamond.contains(&parent) {
                        return;
                    }
                    fin_diamond.insert(parent);

                    let mut new_parents = self
                        .seed
                        .edges_directed(parent, Incoming)
                        .map(|edge| edge.source())
                        .collect::<Vec<NodeIndex>>();

                    next_parents.append(&mut new_parents);
                });

                parents = next_parents;
            }
        }

        let intersection = ini_diamond
            .intersection(&fin_diamond)
            .cloned()
            .collect::<HashSet<NodeIndex>>();

        self.seed
            .retain_nodes(|_, node| intersection.contains(&node));

        self.all_nodes = self.seed.node_indices().collect();
    }
}
