use super::metadata;
use super::metadata::Metadata;
use super::Builder;
use crate::netstrat::Bus;
use crate::widgets::history;
use crate::widgets::history::Difference;
use crate::widgets::history::History;
use crate::widgets::matrix::Elements as MatrixElements;
use crate::widgets::matrix::State as MatrixState;
use crate::widgets::net_props::graph::cycle::Cycle;
use crate::widgets::net_props::graph::elements::Edge;
use crate::widgets::net_props::graph::elements::Elements;
use crate::widgets::net_props::graph::elements::Node;
use crate::widgets::net_props::graph::path::Path;
use crate::widgets::net_props::settings::{ConeSettings, EdgeWeight, Settings};
use crate::widgets::StepDifference;
use lazy_static::lazy_static;
use ndarray::Array;
use ndarray::Array2;
use petgraph::algo::all_simple_paths;
use petgraph::dot::Dot;
use petgraph::graph::NodeIndex;
use petgraph::prelude::{EdgeRef, StableDiGraph};
use petgraph::stable_graph::EdgeIndex;
use petgraph::visit::Visitable;
use petgraph::visit::{depth_first_search, IntoEdgeReferences, IntoNodeReferences, NodeIndexable};
use petgraph::{Direction, Incoming, Outgoing};
use rand::distributions::{Distribution, Uniform};
use rand::prelude::IteratorRandom;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::vec;
use tracing::instrument::WithSubscriber;
use tracing::{debug, error, info, trace};
use uuid::Uuid;

type ConeSettingsList = Vec<ConeSettings>;

#[derive(Clone, Default)]
pub struct State {
    graph: StableDiGraph<Node, Edge>,
    history: History,
    metadata: Metadata,
}

impl State {
    pub fn new(graph: StableDiGraph<Node, Edge>, history: History, metadata: Metadata) -> State {
        State {
            graph,
            history,
            metadata,
        }
    }

    // pub fn apply_difference(&mut self, diff: StepDifference) {
    //     // self.calculated.colored = self.calculated.colored.apply_difference(diff.colored);
    //     // self.calculated.signal = self
    //         // .calculated
    //         // .signal
    //         // .apply_difference(diff.signal_holders);
    //     self.apply_elements_diff(diff.elements);

    //     self.recalculate_metadata();
    // }
    /*
    pub fn from_dot(dot_data: String) -> Option<Self> {
        let mut data = State::default();
        let mut has_errors = false;
        let mut node_weight_to_index = HashMap::new();
        let mut nodes = HashSet::new();
        let mut colored_nodes = HashSet::new();
        let mut edges = HashSet::new();
        let mut colored_edges = HashSet::new();

        dot_data.lines().for_each(|l| {
            if !l.contains("->") && !l.contains('{') && !l.contains('}') {
                if let Some((_, props)) = parse_node_from_dot(l.to_string()) {
                    if let Some(weight) = parse_label(props) {
                        let node_idx = data.graph.add_node(weight.clone());
                        nodes.insert(node_idx);
                        let digit_weight = weight.split('_').last().unwrap();
                        node_weight_to_index.insert(digit_weight.to_string(), node_idx);

                        if l.contains("color") {
                            colored_nodes.insert(node_idx);
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
                if let Some((s, e, p)) = parse_edge_from_dot(l.to_string()) {
                    if let Some(label) = parse_label(p) {
                        let weight = label.parse::<f64>().unwrap();

                        let start = *node_weight_to_index.get(&s).unwrap();
                        let end = *node_weight_to_index.get(&e).unwrap();

                        let edge_idx = data.graph.add_edge(start, end, weight);
                        edges.insert(edge_idx);

                        if l.contains("color") {
                            colored_edges.insert(edge_idx);
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

        data.calculated.colored = data.to_elements(colored_nodes, colored_edges);

        data.history
            .add_step("load from file".to_string(), StepDifference::default());

        data.recalculate_metadata();

        Some(data)
    }

    pub fn simulation_reset(&mut self) {
        let step_diff = StepDifference {
            elements: Default::default(),
            colored: Default::default(),
            signal_holders: Difference {
                plus: Default::default(),
                minus: self.calculated.signal_holders.clone(),
            },
        };
        self.history
            .add_step("reset simulation".to_string(), step_diff);

        self.calculated.signal_holders = Default::default();

        self.recalculate_metadata();
    }
    */

    // pub fn color_ini_cones(&mut self) {
    //     let mut elements = Elements::default();

    //     for el in self.calculated.ini.nodes().clone() {
    //         let cone_elements = self.get_cone_elements(, Outgoing, -1);
    //         elements = elements.union(&cone_elements);
    //     }

    //     self.color_elements(&elements)
    // }

    // pub fn color_fin_cones(&mut self) {
    //     let mut elements = Elements::default();
    //     for el in self.calculated.fin_set.clone() {
    //         let (nodes, edges) = self.get_cone_elements(el, Incoming, -1);
    //         elements = elements.union(&self.to_elements(nodes, edges));
    //     }

    //     self.color_elements(&elements);
    // }

    // pub fn color_cones(&mut self, cones_settings: ConeSettingsList) -> Option<()> {
    //     let mut elements = Elements::default();
    //     let mut has_errors = false;

    //     cones_settings.iter().for_each(|settings| {
    //         settings.roots_names.iter().for_each(|name| {
    //             let root_find_result = self.graph.node_references().find(|node| *node.1 == *name);
    //             if root_find_result.is_none() {
    //                 error!("node with name {} not found", *name);
    //                 has_errors = true;
    //                 return;
    //             }

    //             let root = root_find_result.unwrap();
    //             let (nodes, edges) =
    //                 self.get_cone_elements(root.0, settings.dir, settings.max_steps);
    //             elements = elements.union(&self.to_elements(nodes, edges))
    //         });
    //     });

    //     if has_errors {
    //         return None;
    //     }

    //     self.color_elements(&elements);
    //     Some(())
    // }

    // pub fn delete_cones(&mut self, cones_settings: ConeSettingsList) -> Option<()> {
    //     let mut elements = Elements::default();
    //     let mut has_errors = false;
    //     cones_settings.iter().for_each(|settings| {
    //         settings.roots_names.iter().for_each(|weight| {
    //             let root_find_result = self.graph.node_references().find(|node| *node.1 == *weight);
    //             if root_find_result.is_none() {
    //                 error!("node with weight {} not found", *weight);
    //                 has_errors = true;
    //                 return;
    //             }

    //             let (root_idx, _) = root_find_result.unwrap();
    //             let (nodes, edges) =
    //                 self.get_cone_elements(root_idx, settings.dir, settings.max_steps);
    //             elements = elements.union(&self.to_elements(nodes, edges))
    //         });
    //     });

    //     if has_errors {
    //         return None;
    //     }

    //     self.soft_delete_elements(&elements);
    //     Some(())
    // }

    // pub fn color_cycles(&mut self, cycle_idxs: &HashSet<usize>) {
    //     let mut elements = Elements::default();
    //     self.calculated
    //         .cycles
    //         .iter()
    //         .enumerate()
    //         .for_each(|(i, c)| {
    //             if cycle_idxs.contains(&i) {
    //                 let proto_elements = &c.nodes_and_edges();
    //                 elements = elements.union(
    //                     &self.to_elements(proto_elements.0.clone(), proto_elements.1.clone()),
    //                 )
    //             }
    //         });

    //     self.color_elements(&elements);
    // }

    pub fn diamond_filter(&mut self) {
        // gather cone of all children of inis
        let ini_cones_union = self
            .metadata
            .ini_nodes
            .iter()
            .fold(Elements::default(), |accum, n| {
                accum.union(&self.get_cone_elements(n, Outgoing, -1))
            });

        // gather cone of all parents of fins
        let fin_cones_union = self
            .metadata
            .fin_nodes
            .iter()
            .fold(Elements::default(), |accum, n| {
                accum.union(&self.get_cone_elements(n, Incoming, -1))
            });

        let to_keep = ini_cones_union.intersection(&fin_cones_union);
        let to_delete_nodes = self
            .graph
            .node_weights()
            .filter(|n| !to_keep.nodes().contains(n))
            .cloned()
            .collect::<HashSet<_>>();
        let to_delete_edges = self
            .graph
            .edge_weights()
            .filter(|e| !to_keep.edges().contains(e))
            .cloned()
            .collect::<HashSet<_>>();

        self.delete_elements_and_update(&Elements::new(to_delete_nodes, to_delete_edges));
        self.metadata.recalculate(&self.graph);
    }

    // pub fn delete_cycles(&mut self, cycle_idxs: &HashSet<usize>) {
    //     let mut elements = Elements::default();
    //     self.calculated
    //         .cycles
    //         .iter()
    //         .enumerate()
    //         .for_each(|(i, c)| {
    //             if !cycle_idxs.contains(&i) {
    //                 return;
    //             }

    //             let proto_elements = &c.nodes_and_edges();
    //             elements = elements
    //                 .union(&self.to_elements(proto_elements.0.clone(), proto_elements.1.clone()))
    //         });

    //     self.soft_delete_elements(&elements);
    // }

    // pub fn delete_initial_cone(&mut self) {
    //     let mut elements = Elements::default();
    //     self.calculated.ini_set.iter().for_each(|node_idx| {
    //         let (nodes, edges) = self.get_cone_elements(*node_idx, Outgoing, -1);
    //         elements = elements.union(&self.to_elements(nodes, edges))
    //     });

    //     self.soft_delete_elements(&elements)
    // }

    // pub fn delete_final_cone(&mut self) {
    //     let mut elements = Elements::default();
    //     self.calculated.fin_set.iter().for_each(|node_idx| {
    //         let (nodes, edges) = self.get_cone_elements(*node_idx, Incoming, -1);
    //         elements = elements.union(&self.to_elements(nodes, edges))
    //     });

    //     self.soft_delete_elements(&elements)
    // }

    pub fn color_nodes_and_edges(
        &mut self,
        nodes: Vec<String>,
        edges: Vec<[String; 2]>,
    ) -> Option<()> {
        self.color_elements_and_update(&Elements::new(
            self.find_nodes_by_names(nodes)?,
            self.find_edges_by_nodes_names(edges)?,
        ));
        Some(())
    }

    pub fn delete_nodes_and_edges(
        &mut self,
        nodes: Vec<String>,
        edges: Vec<[String; 2]>,
    ) -> Option<()> {
        self.delete_elements_and_update(&Elements::new(
            self.find_nodes_by_names(nodes)?,
            self.find_edges_by_nodes_names(edges)?,
        ));
        Some(())
    }

    // pub fn cycles(self) -> Vec<Cycle> {
    //     self.calculated.cycles
    // }

    pub fn dot(&self) -> String {
        self.metadata.dot.clone()
    }

    /*     pub fn adj_matrix(&self) -> MatrixState {
        self.calculated.adj_mat.clone()
    } */

    pub fn history(&mut self) -> &mut History {
        &mut self.history
    }

    /*
        pub fn signal_forward(&mut self) -> Elements {
        debug!("propagating signal forward");
        let elements = self.calculate_signal_forward();

        self.history.add_step(
            "signal forward".to_string(),
            StepDifference {
                elements: Default::default(),
                colored: Default::default(),
                signal_holders: self.calculated.signal_holders.compute_difference(&elements),
            },
        );

        self.calculated.signal_holders = elements.clone();
        elements
    }

    pub fn signal_backward(&mut self) -> Elements {
        debug!("propagating signal backward");
        let elements = self.calculate_signal_backward();

        self.history.add_step(
            "signal backward".to_string(),
            StepDifference {
                elements: Default::default(),
                colored: Default::default(),
                signal_holders: self.calculated.signal_holders.compute_difference(&elements),
            },
        );

        self.calculated.signal_holders = elements.clone();
        elements
    }

    fn calculate_signal_forward(&self) -> Elements {
        debug!("propagating signal forward");

        if self.calculated.signal_holders.is_empty() {
            return Elements::new(self.calculated.ini_set.clone(), Default::default());
        }

        let mut new_nodes = HashSet::new();
        let mut new_edges = HashSet::new();
        self.calculated
            .signal_holders
            .nodes()
            .iter()
            .for_each(|node| {
                self.graph.edges_directed(*node, Outgoing).for_each(|edge| {
                    new_edges.insert(EdgeIndex::from(edge.id()));
                });
            });
        self.calculated
            .signal_holders
            .edges()
            .iter()
            .for_each(|edge| {
                new_nodes.insert(self.graph.edge_endpoints(*edge).unwrap().1);
            });
        Elements::new(new_nodes, new_edges)
    }

    fn calculate_signal_backward(&self) -> Elements {
        if self.calculated.signal_holders.is_empty() {
            return Default::default();
        }

        let mut new_nodes = HashSet::new();
        let mut new_edges = HashSet::new();
        self.calculated
            .signal_holders
            .nodes()
            .iter()
            .for_each(|node| {
                self.graph.edges_directed(*node, Incoming).for_each(|edge| {
                    new_edges.insert(edge.id());
                });
            });
        self.calculated
            .signal_holders
            .edges()
            .iter()
            .for_each(|edge| {
                new_nodes.insert(self.graph.edge_endpoints(*edge).unwrap().0);
            });
        Elements::new(new_nodes, new_edges)
    }
     */

    // fn apply_elements_diff(&mut self, diff: Difference) {
    //     if diff.is_empty() {
    //         return;
    //     }

    //     self.soft_delete_elements(&diff.minus);
    //     self.restore_elements(&diff.plus)
    // }

    // // restores elements mapping old idxs to new ones
    // fn restore_elements(&mut self, elements: &Elements) {
    //     let mut nodes_idx_changed = HashMap::with_capacity(elements.nodes().len());
    //     elements.nodes().iter().for_each(|(old_idx, repr)| {
    //         let new_idx = self.graph.add_node(repr.clone());
    //         nodes_idx_changed.insert(*old_idx, new_idx);
    //     });

    //     let mut edges_idx_changed = HashMap::with_capacity(elements.edges().len());
    //     elements.edges().iter().for_each(|(old_idx, repr)| {
    //         let (start_str, end_str, _) = parse_edge_from_dot(repr.clone()).unwrap();
    //         let start: u32 = start_str.parse().unwrap();
    //         let end: u32 = end_str.parse().unwrap();

    //         // FIXME: store and parse weight of the edge
    //         let dummy_weight = 1.0;
    //         let new_idx = self.graph.add_edge(
    //             *nodes_idx_changed.get(&NodeIndex::from(start)).unwrap(),
    //             *nodes_idx_changed.get(&NodeIndex::from(end)).unwrap(),
    //             dummy_weight,
    //         );
    //         edges_idx_changed.insert(*old_idx, new_idx);
    //     });
    // }

    // fn adj_mat(&self) -> Array2<isize> {
    //     let n = self.graph.node_bound();
    //     let mut mat = Array::zeros((n, n));

    //     self.graph.edge_references().for_each(|e| {
    //         let row = e.source().index();
    //         let col = e.target().index();

    //         mat[[row, col]] += 1
    //     });

    //     mat
    // }

    fn find_nodes_by_names(&self, names: Vec<String>) -> Option<HashSet<Node>> {
        let mut nodes_set = HashSet::with_capacity(names.len());
        for name in names {
            let node = self.metadata.node_by_name.get(&name)?;
            if node.deleted() {
                return None;
            }

            nodes_set.insert(node.clone());
        }

        Some(nodes_set)
    }

    fn find_edges_by_nodes_names(&self, nodes_names: Vec<[String; 2]>) -> Option<HashSet<Edge>> {
        let mut edges_set = HashSet::with_capacity(nodes_names.len());
        for bound in nodes_names {
            let start_name = bound.first().unwrap();
            let end_name = bound.last().unwrap();
            let edge_name = format!("{} -> {}", start_name, end_name);
            let edge = self.metadata.edge_by_name.get(&edge_name)?;
            if edge.deleted() {
                return None;
            }

            edges_set.insert(edge.clone());
        }

        Some(edges_set)
    }

    // fn to_elements(&self, nodes: HashSet<NodeIndex>, edges: HashSet<EdgeIndex>) -> Elements {
    //     let mut elements_nodes = HashMap::new();
    //     nodes.iter().for_each(|n| {
    //         elements_nodes.insert(*n, self.graph.node_weight(*n).unwrap().clone());
    //     });

    //     let mut elements_edges = HashMap::new();
    //     edges.iter().for_each(|e| {
    //         let (start, end) = self.graph.edge_endpoints(*e).unwrap();
    //         elements_edges.insert(
    //             *e,
    //             format!(
    //                 "{}->{}",
    //                 self.graph.node_weight(start).unwrap().clone(),
    //                 self.graph.node_weight(end).unwrap().clone(),
    //             ),
    //         );
    //     });

    //     Elements::new(elements_nodes, elements_edges)
    // }

    fn color_elements_and_update(&mut self, elements: &Elements) {
        debug!("coloring elements");
        if elements.is_empty() {
            return;
        }

        let step_diff = StepDifference {
            elements: Default::default(),
            colored: self.metadata.colored.compute_difference(elements),
            signal_holders: Default::default(),
        };
        self.history
            .add_step("color elements".to_string(), step_diff);

        info!("elements colored");

        self.metadata.color(elements);
        self.metadata.recalculate(&self.graph)
    }

    // fn soft_delete_elements(&mut self, elements: &Elements) {
    //     debug!("deleting elements");
    //     if elements.is_empty() {
    //         debug!("nothing to delete");
    //         return;
    //     }

    //     elements.edges().iter().for_each(|(edge, _)| {
    //         self.graph.remove_edge(*edge);
    //     });
    //     elements.nodes().iter().for_each(|(node, _)| {
    //         self.graph.remove_node(*node).unwrap();
    //     });

    //     self.calculated.deleted = self.calculated.deleted.union(elements);

    //     let empty_elements = Elements::default();

    //     let elements_diff = Difference {
    //         plus: Default::default(),
    //         minus: elements.clone(),
    //     };

    //     let step_diff = StepDifference {
    //         elements: elements_diff,
    //         colored: self.calculated.colored.compute_difference(&empty_elements),
    //         signal_holders: self
    //             .calculated
    //             .signal_holders
    //             .compute_difference(&empty_elements),
    //     };

    //     self.history
    //         .add_step("update elements".to_string(), step_diff);

    //     self.calculated.colored = self.calculated.colored.sub(elements);
    //     self.calculated.signal_holders = self.calculated.signal_holders.sub(elements);

    //     info!("elements deleted");
    //     self.recalculate_metadata();
    // }

    // fn calc_longest_path(&self) -> usize {
    //     let mut longest_path = 0;
    //     self.calculated.ini_set.iter().for_each(|ini| {
    //         self.calculated.fin_set.iter().for_each(|fin| {
    //             let curr_max_path_length =
    //                 all_simple_paths::<Vec<_>, _>(&self.graph, *ini, *fin, 0, None)
    //                     .max_by(|left, right| left.len().cmp(&right.len()))
    //                     .unwrap_or_default()
    //                     .len();
    //             if curr_max_path_length > longest_path {
    //                 longest_path = curr_max_path_length
    //             }
    //         })
    //     });
    //     longest_path
    // }

    /*     fn calc_adj_mat(&self) -> MatrixState {
        MatrixState {
            m: self.adj_mat(),
            colored: self.elements_to_matrix_elements(&self.calculated.colored),
            deleted: self.elements_to_matrix_elements(&self.calculated.deleted),
            longest_path: self.calculated.longest_path,
        }
    } */

    fn get_node_index(&self, id: &Uuid) -> &NodeIndex {
        self.metadata.idx_by_node_id.get(id).unwrap()
    }

    fn get_edge_index(&self, id: &Uuid) -> &EdgeIndex {
        self.metadata.idx_by_edge_id.get(id).unwrap()
    }

    fn get_edge(&self, idx: EdgeIndex) -> &Edge {
        self.metadata.edge_by_idx.get(&idx).unwrap()
    }

    fn get_node(&self, idx: NodeIndex) -> &Node {
        self.metadata.node_by_idx.get(&idx).unwrap()
    }

    // fn elements_to_matrix_elements(&self, elements: &Elements) -> MatrixElements {
    //     let mut res: MatrixElements = Default::default();

    //     elements.edges().iter().for_each(|(idx, _)| {
    //         let edge = self.graph.edge_endpoints(*idx).unwrap();
    //         res.elements.insert((edge.0.index(), edge.1.index()));
    //     });

    //     elements.nodes().iter().for_each(|(e, _)| {
    //         res.rows.insert(e.index());
    //         res.cols.insert(e.index());
    //     });

    //     res
    // }

    // fn calc_cycles(&self) -> Vec<Cycle> {
    //     debug!("getting cycles");
    //     let mut cycles: Vec<Cycle> = vec![];
    //     let mut path = vec![];
    //     depth_first_search(
    //         &self.graph,
    //         self.calculated.ini_set.iter().cloned(),
    //         |event| match event {
    //             petgraph::visit::DfsEvent::TreeEdge(s, e) => {
    //                 debug!("visited edge {} -> {}", s.index(), e.index());
    //                 path.push([s.index(), e.index()]);
    //                 debug!("continuing path: {path:?}");
    //             }
    //             petgraph::visit::DfsEvent::BackEdge(s, e) => {
    //                 debug!("visited edge {} -> {}; cycle found!", s.index(), e.index());
    //                 path.push([s.index(), e.index()]);

    //                 let mut first_cycle_el: [usize; 2] = [0, 0];
    //                 let mut path_cycle = path.rsplit(|el| {
    //                     if *el.first().unwrap() == e.index() {
    //                         (first_cycle_el[0], first_cycle_el[1]) =
    //                             (*el.first().unwrap(), *el.last().unwrap());

    //                         return true;
    //                     }

    //                     false
    //                 });

    //                 let mut cycle_proto = path_cycle.next().unwrap().to_vec();
    //                 cycle_proto.insert(0, first_cycle_el);
    //                 debug!("discovered cycle: {cycle_proto:?}");

    //                 let mut cycle = Cycle::new();
    //                 cycle_proto.iter().for_each(|el| {
    //                     let (first, last) = (
    //                         NodeIndex::new(*el.first().unwrap()),
    //                         NodeIndex::new(*el.last().unwrap()),
    //                     );

    //                     cycle.add_path(Path::new(
    //                         first,
    //                         last,
    //                         self.graph.find_edge(first, last).unwrap(),
    //                     ));
    //                 });

    //                 cycles.push(cycle);

    //                 path = remove_last_el(path.clone());
    //                 debug!("shortened path: {path:?}");
    //             }
    //             petgraph::visit::DfsEvent::Finish(n, _) => {
    //                 debug!("finished path on node: {}", n.index());
    //                 path = remove_last_el(path.clone());
    //                 debug!("shortened path: {path:?}");
    //             }
    //             _ => (),
    //         },
    //     );

    //     info!("found {} cycles", cycles.len());

    //     cycles
    // }

    // fn collect_ini(&self) -> Elements {
    //     self.graph.node_indices().for_each(|idx| {
    //         if !self.graph.node_weight(idx).unwrap().contains("ini") {
    //             return;
    //         }

    //         result.insert(idx);
    //     });

    //     result
    // }

    // fn collect_fin(&self) -> Elements {
    //     let mut result = HashSet::new();

    //     self.graph.node_indices().for_each(|idx| {
    //         if !self.graph.node_weight(idx).unwrap().contains("fin") {
    //             return;
    //         }

    //         result.insert(idx);
    //     });

    //     result
    // }

    fn get_cone_elements(&self, root: &Node, dir: Direction, max_steps: i32) -> Elements {
        let root_idx = self.get_node_index(root.id());

        let mut nodes = HashSet::new();
        let mut edges = HashSet::new();

        nodes.insert(root.clone());

        let mut steps = 0;
        let mut starts = vec![*root_idx];
        loop {
            steps += 1;
            if max_steps != -1 && steps > max_steps {
                break;
            }

            let mut neighbours = vec![];
            let mut curr_neighbours = vec![];
            starts.iter().for_each(|s| {
                self.graph.edges_directed(*s, dir).for_each(|e| {
                    let edge = e.weight();
                    if edges.contains(edge) {
                        return;
                    }
                    edges.insert(edge.clone());

                    let node_idx = match dir {
                        Outgoing => e.target(),
                        Incoming => e.source(),
                    };
                    let node = self.get_node(node_idx);
                    if nodes.contains(node) {
                        return;
                    }
                    nodes.insert(node.clone());

                    curr_neighbours.push(node_idx);
                });

                neighbours.append(&mut curr_neighbours);
            });

            if neighbours.is_empty() {
                break;
            }

            starts = neighbours;
        }

        Elements::new(nodes, edges)
    }

    fn delete_elements_and_update(&mut self, elements: &Elements) {
        elements.nodes().iter().for_each(|n| {
            self.delete_node(n);
        });

        elements.edges().iter().for_each(|e| {
            self.delete_edge(e);
        });

        self.metadata.recalculate(&self.graph)
    }

    fn delete_node(&mut self, node: &Node) {
        self.graph.remove_node(*self.get_node_index(node.id()));

        self.metadata.delete_node(node);
    }

    fn delete_edge(&mut self, edge: &Edge) {
        self.graph.remove_edge(*self.get_edge_index(edge.id()));

        self.metadata.delete_edge(edge)
    }
}

/// is not used now
fn weight_line(line: String, weight: f64) -> String {
    let first_part = line.replace(']', "");
    format!("{first_part}, penwidth={weight} ]")
}

fn parse_edge_from_dot(line: String) -> Option<(String, String, String)> {
    lazy_static! {
        static ref EDGE_RE: Regex =
            Regex::new(r"(\d{1,})\s?->\s?(\d{1,})\s?\[?(\slabel\s=\s(.*))?\s?\]?").unwrap();
    }

    let found = EDGE_RE.captures(line.as_str())?;
    let start = found.get(1)?.as_str();
    let end = found.get(2)?.as_str();
    let weight = match found.get(4) {
        Some(res) => res.as_str(),
        None => "",
    };

    trace!("parsed edge: {start} -> {end}, with weight: {weight}");

    Some((start.to_string(), end.to_string(), weight.to_string()))
}

fn parse_node_from_dot(line: String) -> Option<(String, String)> {
    lazy_static! {
        static ref NODE_RE: Regex = Regex::new(r"(\d{1,})\s?\[?(.*)\]?").unwrap();
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
