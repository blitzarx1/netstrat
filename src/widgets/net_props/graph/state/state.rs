use std::collections::HashMap;
use std::collections::HashSet;
use std::vec;

use crate::widgets::matrix::Elements as MatrixElements;
use crate::widgets::matrix::State as MatrixState;
use crate::widgets::net_props::graph::cycle::Cycle;
use crate::widgets::net_props::graph::elements::Elements;
use lazy_static::lazy_static;
use ndarray::Array;
use ndarray::Array2;
use petgraph::algo::all_simple_paths;
use petgraph::algo::simple_paths;
use petgraph::dot::Dot;
use petgraph::graph::NodeIndex;
use petgraph::prelude::{EdgeRef, StableDiGraph};
use petgraph::visit::depth_first_search;
use petgraph::visit::IntoEdgeReferences;
use petgraph::visit::IntoNodeReferences;
use petgraph::visit::NodeIndexable;
use petgraph::{Direction, Incoming, Outgoing};
use rand::distributions::{Distribution, Uniform};
use rand::prelude::IteratorRandom;
use regex::Regex;
use tracing::info;
use tracing::trace;
use tracing::warn;
use tracing::{debug, error};

use crate::widgets::net_props::graph::path::Path;
use crate::widgets::net_props::settings::ConeSettings;
use crate::widgets::net_props::settings::EdgeWeight;
use crate::widgets::net_props::settings::NetSettings;

use super::calculated::Calculated;

const MAX_DOT_WEIGHT: f64 = 5.0;

#[derive(Clone, Default)]
pub struct State {
    graph: StableDiGraph<String, f64>,
    settings: NetSettings,
    calculated: Calculated,
}

impl State {
    pub fn new(settings: NetSettings) -> Self {
        debug!("creating graph with settings: {settings:?}");
        let mut graph = StableDiGraph::with_capacity(
            settings.total_cnt,
            settings.total_cnt * settings.total_cnt,
        );
        let mut all_nodes = HashSet::with_capacity(settings.total_cnt);
        for i in 0..settings.total_cnt {
            let node_idx = graph.add_node(format!("{i}"));
            all_nodes.insert(node_idx);
        }

        let mut rng = rand::thread_rng();
        let mut ini_set = HashSet::with_capacity(settings.ini_cnt);
        let mut ini_to_add = settings.ini_cnt;

        // pick inis
        while ini_to_add > 0 {
            let idx = graph.node_indices().choose(&mut rng).unwrap();

            if ini_set.contains(&idx) {
                continue;
            }

            let weight = graph.node_weight_mut(idx).unwrap();
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

                    graph.add_edge(*last_end, *end, weight);

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

            let weight = graph.node_weight_mut(*idx).unwrap();
            let new_weight = format!("fin_{}", *weight);
            *weight = new_weight.clone();

            fin_set.insert(*idx);
        }

        let calculated = Calculated {
            ini_set,
            fin_set,
            ..Default::default()
        };

        let mut data = Self {
            graph,
            calculated,
            settings: settings.clone(),
        };

        if settings.diamond_filter {
            data.diamond_filter()
        }

        data.recalculate_metadata();

        data
    }

    pub fn from_dot(dot_data: String) -> Option<Self> {
        let mut data = State::default();
        let mut has_errors = false;
        let mut node_weight_to_index = HashMap::new();

        dot_data.lines().for_each(|l| {
            if !l.contains("->") && !l.contains('{') && !l.contains('}') {
                if let Some((_, props)) = parse_node(l.to_string()) {
                    if let Some(weight) = parse_label(props) {
                        let node_idx = data.graph.add_node(weight.clone());
                        let digit_weight = weight.split('_').last().unwrap();
                        node_weight_to_index.insert(digit_weight.to_string(), node_idx);

                        if l.contains("color") {
                            data.calculated.colored.add_node(node_idx);
                        }

                        if weight.contains("ini") {
                            data.calculated.ini_set.insert(node_idx);
                        }
                        if weight.contains("fin") {
                            data.calculated.fin_set.insert(node_idx);
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

                        let edge_idx = data.graph.add_edge(start, end, weight);

                        if l.contains("color") {
                            data.calculated.colored.add_edge(edge_idx);
                        }

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

        data.recalculate_metadata();

        Some(data)
    }

    pub fn color_ini_cones(&mut self) {
        let mut elements = Elements::default();

        for el in self.calculated.ini_set.clone() {
            elements.union(&self.get_cone_elements(el, Outgoing, -1));
        }

        self.calculated.colored = elements;
        self.recalculate_metadata();
    }

    pub fn color_fin_cones(&mut self) {
        let mut elements = Elements::default();
        for el in self.calculated.fin_set.clone() {
            elements.union(&self.get_cone_elements(el, Incoming, -1));
        }

        self.calculated.colored = elements;
        self.recalculate_metadata();
    }

    pub fn color_cones(&mut self, cones_settings: Vec<ConeSettings>) {
        let mut elements = Elements::default();
        cones_settings.iter().for_each(|settings| {
            settings.roots_weights.iter().for_each(|weight| {
                let root_find_result = self.graph.node_references().find(|node| *node.1 == *weight);
                if root_find_result.is_none() {
                    warn!("node with weight {} not found", *weight);
                    return;
                }

                let root = root_find_result.unwrap();
                elements.union(&self.get_cone_elements(root.0, settings.dir, settings.max_steps))
            });
        });

        self.calculated.colored = elements;
        self.recalculate_metadata();
    }

    pub fn delete_cones(&mut self, cones_settings: Vec<ConeSettings>) {
        let mut elements = Elements::default();
        cones_settings.iter().for_each(|settings| {
            settings.roots_weights.iter().for_each(|weight| {
                let root_find_result = self.graph.node_references().find(|node| *node.1 == *weight);
                if root_find_result.is_none() {
                    warn!("node with weight {} not found", *weight);
                    return;
                }

                let (root_idx, _) = root_find_result.unwrap();
                elements.union(&self.get_cone_elements(root_idx, settings.dir, settings.max_steps))
            });
        });

        self.delete_elements(elements)
    }

    pub fn color_cycles(&mut self, cycle_idxs: &HashSet<usize>) {
        let mut elements = Elements::default();
        self.calculated
            .cycles
            .iter()
            .enumerate()
            .for_each(|(i, c)| {
                if cycle_idxs.contains(&i) {
                    elements.union(&c.elements())
                }
            });

        self.calculated.colored = elements;
        self.recalculate_metadata();
    }

    pub fn diamond_filter(&mut self) {
        let mut ini_union_cone = HashSet::new();
        // gather cone of all children of inis
        for el in self.calculated.ini_set.clone() {
            ini_union_cone = ini_union_cone
                .union(&self.get_cone_elements(el, Outgoing, -1).nodes())
                .cloned()
                .collect();
        }

        let mut fin_union_cone = HashSet::new();
        // gather cone of all parents of fins
        for el in self.calculated.fin_set.clone() {
            fin_union_cone = fin_union_cone
                .union(&self.get_cone_elements(el, Incoming, -1).nodes())
                .cloned()
                .collect();
        }

        let intersection = ini_union_cone
            .intersection(&fin_union_cone)
            .cloned()
            .collect::<HashSet<NodeIndex>>();

        self.graph
            .retain_nodes(|_, node| match intersection.contains(&node) {
                false => {
                    self.calculated.deleted.add_node(node);
                    false
                }
                true => true,
            });
    }

    pub fn delete_cycles(&mut self, cycle_idxs: &HashSet<usize>) {
        let mut elements = Elements::default();
        self.calculated
            .cycles
            .iter()
            .enumerate()
            .for_each(|(i, c)| {
                if !cycle_idxs.contains(&i) {
                    return;
                }

                elements.union(&c.elements());
            });

        self.delete_elements(elements);
    }

    pub fn delete_initial_cone(&mut self) {
        let mut elements = Elements::default();
        self.calculated
            .ini_set
            .iter()
            .for_each(|node_idx| elements.union(&self.get_cone_elements(*node_idx, Outgoing, -1)));

        self.delete_elements(elements)
    }

    pub fn delete_final_cone(&mut self) {
        let mut elements = Elements::default();
        self.calculated
            .fin_set
            .iter()
            .for_each(|node_idx| elements.union(&self.get_cone_elements(*node_idx, Incoming, -1)));

        self.delete_elements(elements)
    }

    pub fn color_nodes_and_edges(&mut self, nodes: Vec<String>, edges: Vec<[String; 2]>) {
        self.calculated.colored = self.find_nodes_and_edges(nodes, edges);
        self.recalculate_metadata();
    }

    pub fn delete_nodes_and_edges(&mut self, nodes: Vec<String>, edges: Vec<[String; 2]>) {
        self.delete_elements(self.find_nodes_and_edges(nodes, edges));
    }

    pub fn cycles(self) -> Vec<Cycle> {
        self.calculated.cycles
    }

    pub fn dot(&self) -> String {
        self.calculated.dot.clone()
    }

    pub fn adj_matrix(&self) -> MatrixState {
        self.calculated.adj_mat.clone()
    }

    fn adj_mat(&self) -> Array2<isize> {
        let n = self.graph.node_bound();
        let mut mat = Array::zeros((n, n));

        self.graph.edge_references().for_each(|e| {
            let row = e.source().index();
            let col = e.target().index();

            mat[[row, col]] += 1
        });

        mat
    }

    fn find_nodes_and_edges(&self, nodes: Vec<String>, edges: Vec<[String; 2]>) -> Elements {
        let mut nodes_set = HashSet::with_capacity(nodes.len());
        let mut edges_set = HashSet::with_capacity(edges.len());

        nodes.iter().for_each(|weight| {
            let node_find_result = self.graph.node_references().find(|node| *node.1 == *weight);
            if node_find_result.is_none() {
                warn!("node with weight {} not found", *weight);
                return;
            }

            let (node_idx, _) = node_find_result.unwrap();
            nodes_set.insert(node_idx);
        });

        edges.iter().for_each(|edge| {
            let start_weight = edge.first().unwrap();
            let start_result = self
                .graph
                .node_references()
                .find(|node| *node.1 == *start_weight);
            if start_result.is_none() {
                warn!("node with weight {} not found", start_weight);
                return;
            }
            let (start, _) = start_result.unwrap();

            let end_weight = edge.last().unwrap();
            let end_result = self
                .graph
                .node_references()
                .find(|node| *node.1 == *end_weight);
            if end_result.is_none() {
                warn!("node with weight {} not found", end_weight);
                return;
            }
            let (end, _) = end_result.unwrap();

            let edge_find_result = self.graph.find_edge(start, end);
            if edge_find_result.is_none() {
                warn!("edge {edge:?} not found");
                return;
            }

            let edge_idx = edge_find_result.unwrap();
            edges_set.insert(edge_idx);
        });

        Elements::new(nodes_set, edges_set)
    }

    fn delete_elements(&mut self, elements: Elements) {
        debug!("deleting elements");

        if elements.is_empty() {
            return;
        }

        elements.nodes().iter().for_each(|node| {
            self.graph.remove_node(*node).unwrap();
            self.calculated.deleted.add_node(*node);
        });
        elements.edges().iter().for_each(|edge| {
            self.graph.remove_edge(*edge);
        });

        self.calculated.colored = Default::default();

        info!("elements deleted");
        self.recalculate_metadata();
    }

    fn recalculate_metadata(&mut self) {
        self.calculated.ini_set = self.collect_ini_set();
        self.calculated.fin_set = self.collect_fin_set();
        self.calculated.longest_path = self.calc_longest_path();

        self.calculated.cycles = self.calc_cycles();
        self.calculated.dot = self.calc_dot();
        self.calculated.adj_mat = self.calc_adj_mat();

        info!("graph metadata recalculated");
    }

    fn calc_longest_path(&self) -> usize {
        let mut longest_path = 0;
        self.calculated.ini_set.iter().for_each(|ini| {
            self.calculated.fin_set.iter().for_each(|fin| {
                let curr_max_path_length =
                    all_simple_paths::<Vec<_>, _>(&self.graph, *ini, *fin, 0, None)
                        .max_by(|left, right| left.len().cmp(&right.len()))
                        .unwrap_or_default()
                        .len();
                if curr_max_path_length > longest_path {
                    longest_path = curr_max_path_length
                }
            })
        });
        longest_path
    }

    fn calc_adj_mat(&self) -> MatrixState {
        MatrixState {
            m: self.adj_mat(),
            colored: self.elements_to_matrix_elements(&self.calculated.colored),
            deleted: self.elements_to_matrix_elements(&self.calculated.deleted),
            longest_path: self.calculated.longest_path,
        }
    }

    fn calc_dot(&self) -> String {
        let dot = Dot::new(&self.graph).to_string();
        if self.settings.edge_weight_type == EdgeWeight::Fixed {
            return self.color_dot(dot, self.calculated.colored.clone());
        }

        self.color_dot(self.weight_dot(dot), self.calculated.colored.clone())
    }

    fn elements_to_matrix_elements(&self, elements: &Elements) -> MatrixElements {
        let mut res: MatrixElements = Default::default();

        elements.edges().iter().for_each(|idx| {
            let edge = self.graph.edge_endpoints(*idx).unwrap();
            res.elements.insert((edge.0.index(), edge.1.index()));
        });

        elements.nodes().iter().for_each(|e| {
            res.rows.insert(e.index());
            res.cols.insert(e.index());
        });

        res
    }

    fn calc_cycles(&self) -> Vec<Cycle> {
        debug!("getting cycles");
        let mut cycles: Vec<Cycle> = vec![];
        let mut path = vec![];
        depth_first_search(
            &self.graph,
            self.calculated.ini_set.iter().cloned(),
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

                    let mut cycle_proto = path_cycle.next().unwrap().to_vec();
                    cycle_proto.insert(0, first_cycle_el);
                    debug!("discovered cycle: {cycle_proto:?}");

                    let mut cycle = Cycle::new();
                    cycle_proto.iter().for_each(|el| {
                        let (first, last) = (
                            NodeIndex::new(*el.first().unwrap()),
                            NodeIndex::new(*el.last().unwrap()),
                        );

                        cycle.add_path(Path::new(
                            first,
                            last,
                            self.graph.find_edge(first, last).unwrap(),
                        ));
                    });

                    cycles.push(cycle);

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

        info!("found {} cycles", cycles.len());

        cycles
    }

    fn color_dot(&self, dot: String, elements: Elements) -> String {
        dot.lines()
            .map(|l| -> String {
                let mut res = l.to_string();
                if !l.contains("->") && !l.contains('{') && !l.contains('}') {
                    // line is node
                    if let Some((node_id, _)) = parse_node(l.to_string()) {
                        elements.nodes().iter().for_each(|node| {
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
                    elements.edges().iter().for_each(|edge| {
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

    fn get_cone_elements(&self, root: NodeIndex, dir: Direction, max_steps: i32) -> Elements {
        let mut nodes = HashSet::new();
        let mut edges = HashSet::new();

        nodes.insert(root);

        if max_steps == 0 {
            return Elements::new(nodes, edges);
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

        Elements::new(nodes, edges)
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

    trace!("parsed edge: {start} -> {end}, with props: {props}");

    Some((start.to_string(), end.to_string(), props.to_string()))
}

fn parse_node(line: String) -> Option<(String, String)> {
    lazy_static! {
        static ref NODE_RE: Regex = Regex::new(r"(\d{1,}) \[(.*)\]").unwrap();
    }

    let found = NODE_RE.captures(line.as_str())?;
    let node = found.get(1)?.as_str();
    let props = found.get(2)?.as_str();

    trace!("parsed node: {node}, with props: {props}");

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
