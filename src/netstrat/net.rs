use std::collections::HashMap;
use std::collections::HashSet;

use lazy_static::lazy_static;
use petgraph::data::Build;
use petgraph::data::FromElements;
use petgraph::dot::Dot;
use petgraph::graph::node_index;
use petgraph::graph::NodeIndex;
use petgraph::prelude::{EdgeIndex, EdgeRef, StableDiGraph};
use petgraph::visit::depth_first_search;
use petgraph::visit::Control;
use petgraph::visit::IntoNodeReferences;
use petgraph::{Direction, Incoming, Outgoing};
use rand::distributions::{Distribution, Uniform};
use rand::prelude::IteratorRandom;
use regex::Regex;
use tracing::{debug, error};

const MAX_DOT_WEIGHT: f64 = 5.0;

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

    pub fn from_dot(dot_data: String) -> Option<Self> {
        let mut data = Data {
            graph: StableDiGraph::new(),
            settings: Settings::default(),
            ini_set: HashSet::new(),
            fin_set: HashSet::new(),
        };
        let mut has_errors = false;
        let mut node_weight_to_index = HashMap::new();
        dot_data.lines().for_each(|l| {
            if !l.contains("->") && !l.contains('{') && !l.contains('}') {
                if let Some((_, props)) = parse_node(l.to_string()) {
                    if let Some(weight) = parse_label(props) {
                        let node_idx = data.graph.add_node(weight.clone());
                        let digit_weight = weight.split('_').last().unwrap();
                        node_weight_to_index.insert(digit_weight.to_string(), node_idx);

                        if weight.contains("ini") {
                            data.ini_set.insert(node_idx);
                        }
                        if weight.contains("fin") {
                            data.fin_set.insert(node_idx);
                        }

                        return;
                    }
                }

                error!("failed to parse node from line: {l}");
                has_errors = true;
            }
        });

        dot_data.lines().for_each(|l| {
            if l.contains("->") {
                if let Some((s, e, p)) = parse_edge(l.to_string()) {
                    if let Some(label) = parse_label(p) {
                        let weight = label.parse::<f64>().unwrap();

                        let start = *node_weight_to_index.get(&s).unwrap();
                        let end = *node_weight_to_index.get(&e).unwrap();

                        data.graph.add_edge(start, end, weight);
                        return;
                    }
                }

                error!("failed to parse edge from line: {l}");
                has_errors = true;
            }
        });

        if has_errors {
            return None;
        }

        Some(data)
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

    pub fn dot_with_cycles(&self) -> String {
        let subgraph = self.get_cycles();
        self.color_dot(self.dot(), subgraph.nodes, subgraph.edges)
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

    fn get_cycles(&self) -> Subgraph {
        let mut nodes = HashSet::new();
        let mut edges = HashSet::new();

        let mut path = vec![];

        depth_first_search(
            &self.graph,
            self.ini_set.iter().cloned(),
            |event| match event {
                petgraph::visit::DfsEvent::TreeEdge(s, e) => {
                    debug!("visited edge {} -> {}", s.index(), e.index());
                    path.push([s.index(), e.index()]);
                    debug!("continuing path: {path:?}");
                }
                petgraph::visit::DfsEvent::BackEdge(s, e) => {
                    debug!("visited edge {} -> {}; cycle found!", s.index(), e.index());
                    path.push([s.index(), e.index()]);

                    let mut first_cycle_el: [usize; 2] = [0, 0];
                    let mut path_cycle = path.rsplit(|el| {
                        if *el.first().unwrap() == e.index() {
                            (first_cycle_el[0], first_cycle_el[1]) =
                                (*el.first().unwrap(), *el.last().unwrap());

                            return true;
                        }

                        false
                    });

                    let mut cycle = path_cycle.next().unwrap().to_vec();
                    cycle.insert(0, first_cycle_el);
                    debug!("discovered cycle: {cycle:?}");

                    cycle.iter().for_each(|el| {
                        let (first, last) = (
                            NodeIndex::new(*el.first().unwrap()),
                            NodeIndex::new(*el.last().unwrap()),
                        );

                        nodes.insert(first);
                        nodes.insert(last);
                        edges.insert(self.graph.find_edge(first, last).unwrap());
                    });

                    path = remove_last_el(path.clone());
                    debug!("shortened path: {path:?}");
                }
                petgraph::visit::DfsEvent::Finish(n, _) => {
                    debug!("finished path on node: {}", n.index());
                    path = remove_last_el(path.clone());
                    debug!("shortened path: {path:?}");
                }
                _ => (),
            },
        );

        Subgraph { nodes, edges }
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
                    if let Some((node_id, _)) = parse_node(l.to_string()) {
                        nodes.iter().for_each(|node| {
                            if node_id == node.index().to_string() {
                                res = color_line(l.to_string());
                            }
                        });
                    } else {
                        error!("failed to parse node from line: {l}");
                    }
                }

                if l.contains("->") {
                    // line is edge
                    edges.iter().for_each(|edge| {
                        let (start, end) = self.graph.edge_endpoints(*edge).unwrap();

                        if let Some((s, e, _)) = parse_edge(l.to_string()) {
                            if s == start.index().to_string() && e == end.index().to_string() {
                                res = color_line(l.to_string());
                            }
                        } else {
                            error!("failed to parse edge from line: {l}");
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

                        if let Some((s, e, _)) = parse_edge(l.to_string()) {
                            if s == start.index().to_string() && e == end.index().to_string() {
                                let weight = *self.graph.edge_weight(edge).unwrap();
                                let mut normed = (weight / max_weight) * MAX_DOT_WEIGHT;
                                if normed < 0.5 {
                                    normed = 0.5
                                }

                                res = weight_line(l.to_string(), normed);
                            }
                        } else {
                            error!("failed to parse edge from line: {l}")
                        }
                    });
                }

                format!("{res}\n")
            })
            .collect()
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

    fn get_cone(&self, root: NodeIndex, dir: Direction, max_steps: i32) -> Subgraph {
        let mut nodes = HashSet::new();
        let mut edges = HashSet::new();

        nodes.insert(root);

        if max_steps == 0 {
            return Subgraph { nodes, edges };
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

        Subgraph { nodes, edges }
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

fn parse_edge(line: String) -> Option<(String, String, String)> {
    lazy_static! {
        static ref EDGE_RE: Regex = Regex::new(r"(\d{1,}) -> (\d{1,}).*\[(.*)\]").unwrap();
    }

    let found = EDGE_RE.captures(line.as_str())?;
    let start = found.get(1)?.as_str();
    let end = found.get(2)?.as_str();
    let props = found.get(3)?.as_str();

    debug!("parsed edge: {start} -> {end}, with props: {props}");

    Some((start.to_string(), end.to_string(), props.to_string()))
}

fn parse_node(line: String) -> Option<(String, String)> {
    lazy_static! {
        static ref NODE_RE: Regex = Regex::new(r"(\d{1,}) \[(.*)\]").unwrap();
    }

    let found = NODE_RE.captures(line.as_str())?;
    let node = found.get(1)?.as_str();
    let props = found.get(2)?.as_str();

    debug!("parsed node: {node}, with props: {props}");

    Some((node.to_string(), props.to_string()))
}

fn parse_label(props: String) -> Option<String> {
    lazy_static! {
        static ref LABEL_RE: Regex = Regex::new(r#"label = "(.*)" "#).unwrap();
    }

    let found = LABEL_RE.captures(props.as_str())?;
    let label = found.get(1)?.as_str();

    debug!("parsed label: {label}");

    Some(label.to_string())
}

fn remove_last_el(path: Vec<[usize; 2]>) -> Vec<[usize; 2]> {
    if path.is_empty() {
        return path;
    }

    let (_, path_arr) = path.split_last().unwrap();
    path_arr.to_vec()
}

struct Subgraph {
    nodes: HashSet<NodeIndex>,
    edges: HashSet<EdgeIndex>,
}
