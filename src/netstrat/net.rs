use std::collections::HashSet;

use petgraph::dot::Dot;
use petgraph::graph::{Node, NodeIndex};
use petgraph::prelude::{EdgeIndex, EdgeRef, StableDiGraph, StableGraph};
use petgraph::visit::IntoNodeReferences;
use petgraph::{Direction, Graph, Incoming, Outgoing};
use rand::distributions::{Distribution, Uniform};
use rand::prelude::IteratorRandom;
use tracing::debug;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum EdgeWeight {
    /// Fixed weight
    Fixed,
    /// Random weigh in range 0..1
    Random,
}

#[derive(PartialEq, Clone, Debug)]
pub struct Settings {
    pub ini_cnt: usize,
    pub fin_cnt: usize,
    pub total_cnt: usize,
    pub no_twin_edges: bool,
    pub max_out_degree: usize,
    pub edge_weight_type: EdgeWeight,
    pub edge_weight: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ini_cnt: 5,
            fin_cnt: 5,
            total_cnt: 20,
            max_out_degree: 3,
            no_twin_edges: true,
            edge_weight_type: EdgeWeight::Fixed,
            edge_weight: 1.0,
        }
    }
}

pub struct Data {
    graph: StableDiGraph<String, f64>,
    settings: Settings,
    ini_set: HashSet<NodeIndex>,
    fin_set: HashSet<NodeIndex>,
}

impl Data {
    pub fn new(settings: Settings) -> Self {
        debug!("creating graph with settings: {settings:?}");
        let mut seed = StableDiGraph::with_capacity(
            settings.total_cnt,
            settings.total_cnt * settings.total_cnt,
        );
        let mut all_nodes = HashSet::with_capacity(settings.total_cnt);
        for i in 0..settings.total_cnt {
            let node_idx = seed.add_node(format!("{i}"));
            all_nodes.insert(node_idx);
        }

        let mut rng = rand::thread_rng();
        let mut ini_set = HashSet::with_capacity(settings.ini_cnt);
        let mut ini_to_add = settings.ini_cnt;

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
        let max_degree_pool = Uniform::from(0..settings.max_out_degree);
        let max_degree_pool_ini = Uniform::from(1..settings.max_out_degree);
        let edge_weight_pool = Uniform::from(0.0..1.0);
        let mut edges_map = HashSet::<[usize; 2]>::new();

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

                let curr_degree = match ini_set.contains(last_end) {
                    true => max_degree_pool_ini.sample(&mut rng),
                    false => max_degree_pool.sample(&mut rng),
                };
                for _i in 0..curr_degree {
                    let end = all_nodes.iter().choose(&mut rng).unwrap();

                    if settings.no_twin_edges
                        && edges_map.contains(&[last_end.index(), end.index()])
                    {
                        continue;
                    }

                    let mut weight = settings.edge_weight;
                    if settings.edge_weight_type == EdgeWeight::Random {
                        weight = edge_weight_pool.sample(&mut rng);
                    }

                    seed.add_edge(*last_end, *end, weight);

                    edges_map.insert([last_end.index(), end.index()]);

                    next_last_ends.push(*end);
                    ends.push(*end);
                }
            });

            last_ends = next_last_ends;

            if started == 0 {
                break;
            }
        }

        let mut fin_set = HashSet::with_capacity(settings.fin_cnt);
        // define fins
        for _i in 0..settings.fin_cnt {
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
            graph: seed,
            settings,
            ini_set,
            fin_set,
        }
    }

    pub fn dot(&self) -> String {
        let dot = Dot::new(&self.graph).to_string();

        if self.settings.edge_weight_type == EdgeWeight::Fixed {
            return dot;
        }

        self.weight_dot(dot)
    }

    pub fn dot_with_ini_cones(&mut self) -> String {
        let mut edges_to_color = HashSet::new();
        let mut nodes_to_color = HashSet::new();

        for el in self.ini_set.clone() {
            let cone = self.get_cone(el, Outgoing, -1);
            edges_to_color = edges_to_color.union(&cone.edges).cloned().collect();
            nodes_to_color = nodes_to_color.union(&cone.nodes).cloned().collect();
        }

        self.color_dot(self.dot(), nodes_to_color, edges_to_color)
    }

    pub fn dot_with_fin_cones(&mut self) -> String {
        let mut edges_to_color = HashSet::new();
        let mut nodes_to_color = HashSet::new();

        for el in self.fin_set.clone() {
            let cone = self.get_cone(el, Incoming, -1);
            edges_to_color = edges_to_color.union(&cone.edges).cloned().collect();
            nodes_to_color = nodes_to_color.union(&cone.nodes).cloned().collect();
        }

        self.color_dot(self.dot(), nodes_to_color, edges_to_color)
    }

    pub fn dot_with_custom_cone(
        &mut self,
        root_weight: String,
        dir: Direction,
        max_steps: i32,
    ) -> String {
        if let Some(root) = self
            .graph
            .node_references()
            .find(|node| *node.1 == root_weight)
        {
            let cone = self.get_cone(root.0, dir, max_steps);

            return self.color_dot(self.dot(), cone.nodes, cone.edges);
        }

        self.dot()
    }

    pub fn remove_cone(&mut self, root_weight: String, dir: Direction, max_steps: i32) {
        if let Some(root) = self
            .graph
            .node_references()
            .find(|node| *node.1 == root_weight)
        {
            let cone = self.get_cone(root.0, dir, max_steps);

            cone.nodes.iter().for_each(|node| {
                self.graph.remove_node(*node).unwrap();
            });
        }
    }

    fn color_dot(
        &self,
        dot: String,
        nodes: HashSet<NodeIndex>,
        edges: HashSet<EdgeIndex>,
    ) -> String {
        dot.lines()
            .map(|l| -> String {
                let mut res = l.to_string();
                if !l.contains("->") && !l.contains('{') && !l.contains('}') {
                    // line is node
                    let parsed_node_id = l.split('[').next().unwrap().trim();
                    nodes.iter().for_each(|node| {
                        if parsed_node_id == node.index().to_string().as_str() {
                            res = color_line(l.to_string());
                        }
                    });
                }

                if l.contains("->") {
                    // line is edge
                    edges.iter().for_each(|edge| {
                        let (start, end) = self.graph.edge_endpoints(*edge).unwrap();

                        let mut parts = l.split("->");
                        let parsed_start = parts.next().unwrap().trim();
                        let parsed_end = parts.next().unwrap().split('[').next().unwrap().trim();

                        if parsed_start == start.index().to_string().as_str()
                            && parsed_end == end.index().to_string().as_str()
                        {
                            res = color_line(l.to_string());
                        }
                    });
                }

                format!("{res}\n")
            })
            .collect()
    }

    fn weight_dot(&self, dot: String) -> String {
        let max_weight_index = self
            .graph
            .edge_indices()
            .max_by(|left, right| {
                self.graph
                    .edge_weight(*left)
                    .unwrap()
                    .partial_cmp(self.graph.edge_weight(*right).unwrap())
                    .unwrap()
            })
            .unwrap();

        let max_weight = *self.graph.edge_weight(max_weight_index).unwrap();

        dot.lines()
            .map(|l| -> String {
                let mut res = l.to_string();
                if l.contains("->") {
                    // line is edge
                    self.graph.edge_indices().for_each(|edge| {
                        let (start, end) = self.graph.edge_endpoints(edge).unwrap();

                        let mut parts = l.split("->");
                        let parsed_start = parts.next().unwrap().trim();
                        let parsed_end = parts.next().unwrap().split('[').next().unwrap().trim();

                        if parsed_start == start.index().to_string().as_str()
                            && parsed_end == end.index().to_string().as_str()
                        {
                            let weight = *self.graph.edge_weight(edge).unwrap();
                            let mut normed = (weight / max_weight) * 10.0;
                            if normed < 0.5 {
                                normed = 0.5
                            }

                            res = weight_line(l.to_string(), normed);
                        }
                    });
                }

                format!("{res}\n")
            })
            .collect()
    }

    pub fn diamond_filter(&mut self) {
        let mut ini_union_cone = HashSet::new();
        // gather cone of all children of inis
        for el in self.ini_set.clone() {
            ini_union_cone = ini_union_cone
                .union(&self.get_cone(el, Outgoing, -1).nodes)
                .cloned()
                .collect();
        }

        let mut fin_union_cone = HashSet::new();
        // gather cone of all parents of fins
        for el in self.fin_set.clone() {
            fin_union_cone = fin_union_cone
                .union(&self.get_cone(el, Incoming, -1).nodes)
                .cloned()
                .collect();
        }

        let intersection = ini_union_cone
            .intersection(&fin_union_cone)
            .cloned()
            .collect::<HashSet<NodeIndex>>();

        self.graph
            .retain_nodes(|_, node| intersection.contains(&node));

        self.ini_set = self.collect_ini_set();
        self.fin_set = self.collect_fin_set();
    }

    fn collect_ini_set(&self) -> HashSet<NodeIndex> {
        let mut result = HashSet::new();

        self.graph.node_indices().for_each(|idx| {
            if !self.graph.node_weight(idx).unwrap().contains("ini") {
                return;
            }

            result.insert(idx);
        });

        result
    }

    fn collect_fin_set(&self) -> HashSet<NodeIndex> {
        let mut result = HashSet::new();

        self.graph.node_indices().for_each(|idx| {
            if !self.graph.node_weight(idx).unwrap().contains("fin") {
                return;
            }

            result.insert(idx);
        });

        result
    }

    fn get_cone(&self, root: NodeIndex, dir: Direction, max_steps: i32) -> Cone {
        let mut nodes = HashSet::new();
        let mut edges = HashSet::new();

        nodes.insert(root);

        if max_steps == 0 {
            return Cone { nodes, edges };
        }

        let mut steps_cnt = 0;

        let mut connected = self
            .graph
            .edges_directed(root, dir)
            .map(|edge| {
                edges.insert(edge.id());

                match dir {
                    Outgoing => edge.target(),
                    Incoming => edge.source(),
                }
            })
            .collect::<Vec<NodeIndex>>();
        while !connected.is_empty() {
            steps_cnt += 1;
            let mut next_connected = vec![];
            let mut next_edges = vec![];

            connected.drain(..).for_each(|sibling| {
                if nodes.contains(&sibling) {
                    return;
                }
                nodes.insert(sibling);

                let mut new_connected = self
                    .graph
                    .edges_directed(sibling, dir)
                    .map(|edge| {
                        next_edges.push(edge.id());

                        match dir {
                            Outgoing => edge.target(),
                            Incoming => edge.source(),
                        }
                    })
                    .collect::<Vec<NodeIndex>>();

                next_connected.append(&mut new_connected);
            });

            connected = next_connected;

            if max_steps != -1 && steps_cnt >= max_steps {
                break;
            }

            next_edges.iter().for_each(|edge| {
                edges.insert(*edge);
            });
        }

        Cone { nodes, edges }
    }
}

fn color_line(line: String) -> String {
    let first_part = line.replace(']', "");
    format!("{first_part}, color=red ]")
}

fn weight_line(line: String, weight: f64) -> String {
    let first_part = line.replace(']', "");
    format!("{first_part}, penwidth={weight} ]")
}

struct Cone {
    nodes: HashSet<NodeIndex>,
    edges: HashSet<EdgeIndex>,
}
