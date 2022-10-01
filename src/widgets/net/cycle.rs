use std::collections::HashSet;

use super::{path::Path, elements::Elements};

#[derive(Clone)]
pub struct Cycle(Vec<Path>);

impl Cycle {
    pub fn new() -> Self {
        Cycle(vec![])
    }

    pub fn add_path(&mut self, p: Path) {
        self.0.push(p)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn elements(&self) -> Elements {
        let mut nodes = HashSet::new();
        let mut edges = HashSet::new();

        self.0.iter().for_each(|p| {
            nodes.insert(p.start());
            nodes.insert(p.end());
            edges.insert(p.edge());
        });

        Elements::new(nodes, edges)
    }
}
