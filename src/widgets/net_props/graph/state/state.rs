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
use crate::widgets::net_props::graph::elements;
use crate::widgets::net_props::graph::elements::Edge;
use crate::widgets::net_props::graph::elements::ElementID;
use crate::widgets::net_props::graph::elements::Elements;
use crate::widgets::net_props::graph::elements::Node;
use crate::widgets::net_props::graph::path::Path;
use crate::widgets::net_props::settings::ConeSettings;
use crate::widgets::net_props::Graph;
use crate::widgets::StepDifference;
use history::HistorySerializable;
use lazy_static::lazy_static;
use ndarray::Array;
use ndarray::Array2;
use petgraph::algo::all_simple_paths;
use petgraph::data::DataMapMut;
use petgraph::dot::Dot;
use petgraph::graph::NodeIndex;
use petgraph::prelude::EdgeRef;
use petgraph::stable_graph::EdgeIndex;
use petgraph::visit::Visitable;
use petgraph::visit::{depth_first_search, IntoEdgeReferences, IntoNodeReferences, NodeIndexable};
use petgraph::{Direction, Incoming, Outgoing};
use rand::distributions::{Distribution, Uniform};
use rand::prelude::IteratorRandom;
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::vec;
use tracing::instrument::WithSubscriber;
use tracing::{debug, error, info, trace};
use uuid::Uuid;

type ConeSettingsList = Vec<ConeSettings>;

#[derive(Serialize, Deserialize)]
pub struct StateSerializable {
    graph: Graph,
    history: HistorySerializable,
    metadata: Metadata,
}

#[derive(Clone, Default)]
pub struct State {
    graph: Graph,
    history: History,
    metadata: Metadata,
}

impl State {
    pub fn from_json_string(input: String, bus: Bus) -> Option<State> {
        let res = serde_json::from_str::<StateSerializable>(&input);
        if res.is_err() {
            return None;
        }

        let seed = res.unwrap();

        Some(State {
            graph: seed.graph,
            history: seed.history.to_history(bus),
            metadata: seed.metadata,
        })
    }

    pub fn new(graph: Graph, bus: Bus, metadata: Metadata) -> State {
        let history = History::new("create".to_string(), bus);
        State {
            graph,
            history,
            metadata,
        }
    }

    pub fn export(&self) -> String {
        let serializable = StateSerializable {
            graph: self.graph.clone(),
            history: self.history.to_serializable(),
            metadata: self.metadata.clone(),
        };

        serde_json::to_string(&serializable).unwrap()
    }

    pub fn select_nodes_and_edges(
        &mut self,
        nodes: Vec<String>,
        edges: Vec<[String; 2]>,
    ) -> Option<()> {
        let elements = &Elements::new(
            self.find_nodes_by_names(nodes)?
                .iter()
                .map(|n| n.id().clone())
                .collect(),
            self.find_edges_by_nodes_names(edges)?
                .iter()
                .map(|n| n.id().clone())
                .collect(),
        );

        let step_diff = StepDifference {
            elements: Default::default(),
            selected: self.metadata.selected.compute_difference(elements),
        };
        self.apply_difference(step_diff.clone());
        self.history
            .add_step("select elements".to_string(), step_diff);

        Some(())
    }

    pub fn delete_nodes_and_edges(
        &mut self,
        nodes: Vec<String>,
        edges: Vec<[String; 2]>,
    ) -> Option<()> {
        let nodes = self
            .find_nodes_by_names(nodes)?
            .iter()
            .map(|n| n.id().clone())
            .collect::<HashSet<_>>();
        let mut edges = self
            .find_edges_by_nodes_names(edges)?
            .iter()
            .map(|e| e.id().clone())
            .collect::<HashSet<_>>();

        // delete edges connected to deleted nodes as well
        nodes.iter().for_each(|n| {
            let node_idx = *self.node_index(n);
            edges = edges
                .union(
                    &self
                        .graph
                        .edges_directed(node_idx, Incoming)
                        .map(|e| e.weight().id().clone())
                        .collect::<HashSet<_>>(),
                )
                .cloned()
                .collect();
            edges = edges
                .union(
                    &self
                        .graph
                        .edges_directed(node_idx, Outgoing)
                        .map(|e| e.weight().id().clone())
                        .collect::<HashSet<_>>(),
                )
                .cloned()
                .collect();
        });

        let elements_to_delete = Elements::new(nodes, edges);
        let elements_diff = elements_to_delete.compute_difference(&Default::default());
        let selected_diff = self
            .metadata
            .selected
            .compute_difference(&self.metadata.selected.sub(&elements_to_delete));

        let step_diff = StepDifference {
            elements: elements_diff,
            selected: selected_diff,
        };
        self.apply_difference(step_diff.clone());
        self.history
            .add_step("delete elements".to_string(), step_diff);

        Some(())
    }

    pub fn apply_difference(&mut self, diff: StepDifference) {
        debug!("received diff to apply: {:?}", diff);

        self.apply_selected_diff(diff.selected);
        self.apply_elements_diff(diff.elements);
        self.metadata.recalculate(&self.graph);

        debug!("diff applied")
    }

    // pub fn simulation_reset(&mut self) {
    //     let step_diff = StepDifference {
    //         elements: Default::default(),
    //         colored: Default::default(),
    //         signal_holders: Difference {
    //             plus: Default::default(),
    //             minus: self.calculated.signal_holders.clone(),
    //         },
    //     };
    //     self.history
    //         .add_step("reset simulation".to_string(), step_diff);

    //     self.calculated.signal_holders = Default::default();

    //     self.recalculate_metadata();
    // }

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
            .filter_map(|n| {
                if to_keep.nodes().contains(n.id()) {
                    return None;
                }
                return Some(n.id());
            })
            .cloned()
            .collect::<HashSet<_>>();
        let to_delete_edges = self
            .graph
            .edge_weights()
            .filter_map(|e| {
                if to_keep.edges().contains(e.id()) {
                    return None;
                }
                Some(e.id())
            })
            .cloned()
            .collect::<HashSet<_>>();

        self.apply_elements_diff(
            Elements::new(to_delete_nodes, to_delete_edges).compute_difference(&Default::default()),
        );
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

    fn apply_selected_diff(&mut self, diff: Difference) {
        if diff.is_empty() {
            return;
        }

        diff.minus.nodes().iter().for_each(|n| {
            let node_idx = self.metadata.idx_by_node_id[&n.id];
            self.graph.node_weight_mut(node_idx).unwrap().deselect();
        });
        diff.minus.edges().iter().for_each(|e| {
            let edge_idx = self.metadata.idx_by_edge_id[&e.id];
            self.graph.edge_weight_mut(edge_idx).unwrap().deselect();
        });

        diff.plus.nodes().iter().for_each(|n| {
            let node_idx = self.metadata.idx_by_node_id[&n.id];
            self.graph.node_weight_mut(node_idx).unwrap().select();
        });
        diff.plus.edges().iter().for_each(|e| {
            let edge_idx = self.metadata.idx_by_edge_id[&e.id];
            self.graph.edge_weight_mut(edge_idx).unwrap().select();
        });

        self.metadata.selected = self.metadata.selected.apply_difference(diff);
    }

    fn apply_elements_diff(&mut self, diff: Difference) {
        if diff.is_empty() {
            return;
        }

        diff.minus.nodes().iter().for_each(|n| {
            let node_idx = self.metadata.idx_by_node_id[&n.id];
            self.graph.node_weight_mut(node_idx).unwrap().delete();
        });
        diff.minus.edges().iter().for_each(|e| {
            let edge_idx = self.metadata.idx_by_edge_id[&e.id];
            self.graph.edge_weight_mut(edge_idx).unwrap().delete();
        });

        diff.plus.nodes().iter().for_each(|n| {
            let node_idx = self.metadata.idx_by_node_id[&n.id];
            self.graph.node_weight_mut(node_idx).unwrap().restore();
        });
        diff.plus.edges().iter().for_each(|e| {
            let edge_idx = self.metadata.idx_by_edge_id[&e.id];
            self.graph.edge_weight_mut(edge_idx).unwrap().restore();
        });

        self.metadata.elements = self.metadata.elements.apply_difference(diff);
    }

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
            let node = self.node_by_name(&name)?;
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
            let edge = self.edge_by_name(&edge_name)?;
            if edge.deleted() {
                return None;
            }

            edges_set.insert(edge.clone());
        }

        Some(edges_set)
    }

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

    fn node_index(&self, id: &ElementID) -> &NodeIndex {
        &self.metadata.idx_by_node_id[&id.id]
    }

    fn edge_index(&self, id: &ElementID) -> &EdgeIndex {
        &self.metadata.idx_by_edge_id[&id.id]
    }

    fn node(&self, id: &ElementID) -> &Node {
        self.graph
            .node_weight(self.metadata.idx_by_node_id[&id.id])
            .unwrap()
    }

    fn edge(&self, id: &ElementID) -> &Edge {
        self.graph
            .edge_weight(self.metadata.idx_by_edge_id[&id.id])
            .unwrap()
    }

    fn node_by_name(&self, name: &String) -> Option<&Node> {
        Some(self.node(self.metadata.node_by_name.get(name)?))
    }

    fn edge_by_name(&self, name: &String) -> Option<&Edge> {
        Some(self.edge(self.metadata.edge_by_name.get(name)?))
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

    fn get_cone_elements(&self, root_id: &ElementID, dir: Direction, max_steps: i32) -> Elements {
        let root_idx = self.node_index(root_id);

        let mut nodes = HashSet::new();
        let mut edges = HashSet::new();

        nodes.insert(root_id.clone());

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
                    if edges.contains(edge.id()) {
                        return;
                    }
                    edges.insert(edge.id().clone());

                    let node_idx = match dir {
                        Outgoing => e.target(),
                        Incoming => e.source(),
                    };
                    let node = self.graph.node_weight(node_idx).unwrap();
                    if nodes.contains(node.id()) {
                        return;
                    }
                    nodes.insert(node.id().clone());

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
}
