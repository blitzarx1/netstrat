use std::collections::HashSet;

use petgraph::graph::NodeIndex;

use crate::widgets::matrix::State as MatrixState;
use crate::widgets::net_props::graph::{cycle::Cycle, elements::Elements};

#[derive(Default, Clone)]
pub struct Calculated {
    pub ini_set: HashSet<NodeIndex>,
    pub fin_set: HashSet<NodeIndex>,
    pub cycles: Vec<Cycle>,
    pub adj_mat: MatrixState,
    pub longest_path: usize,
    pub dot: String,
    pub colored: Elements,
    pub deleted: Elements,
    pub signal_holders: Elements,
}
