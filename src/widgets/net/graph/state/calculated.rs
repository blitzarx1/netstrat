use std::collections::HashSet;

use petgraph::graph::NodeIndex;

use crate::widgets::net::graph::{cycle::Cycle, matrix, elements::Elements};

#[derive(Default, Clone)]
pub struct Calculated {
    pub ini_set: HashSet<NodeIndex>,
    pub fin_set: HashSet<NodeIndex>,
    pub cycles: Vec<Cycle>,
    pub adj_mat: matrix::State,
    pub dot: String,
    pub colored: Elements,
    pub deleted: Elements,
}
