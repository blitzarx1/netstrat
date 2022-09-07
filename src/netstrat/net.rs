use petgraph::dot::Dot;
use petgraph::graph::{Node, NodeIndex};
use petgraph::prelude::{EdgeIndex, EdgeRef};
use petgraph::{Direction, Graph, Incoming, Outgoing};
use rand::distributions::{Distribution, Uniform};
use rand::prelude::IteratorRandom;
use std::collections::HashSet;

pub struct Data {
    graph: Graph<String, f64>,
    last_ini_set: HashSet<NodeIndex>, // FIXME: store not indexes but nodes
    last_fin_set: HashSet<NodeIndex>, // FIXME: store not indexes but nodes
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
        let mut last_ini_set = HashSet::with_capacity(ini_cnt);
        let mut ini_to_add = ini_cnt;

        // pick inis
        while ini_to_add > 0 {
            let idx = seed.node_indices().choose(&mut rng).unwrap();

            if last_ini_set.contains(&idx) {
                continue;
            }

            let weight = seed.node_weight_mut(idx).unwrap();
            let new_weight = format!("ini_{}", *weight);
            *weight = new_weight.clone();

            last_ini_set.insert(idx);
            ini_to_add -= 1;
        }

        let mut last_ends = last_ini_set.iter().cloned().collect::<Vec<NodeIndex>>();
        let mut starts = HashSet::new();
        let mut ends = vec![];
        let max_degree_pool = Uniform::from(0..max_out_degree);
        let max_degree_pool_ini = Uniform::from(1..max_out_degree);

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

                let curr_degree = match last_ini_set.contains(last_end) {
                    true => max_degree_pool_ini.sample(&mut rng),
                    false => max_degree_pool.sample(&mut rng),
                };
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

        let mut last_fin_set = HashSet::with_capacity(fin_cnt);
        // define fins
        for _i in 0..fin_cnt {
            let idx = ends.iter().choose(&mut rng).unwrap();

            if last_fin_set.contains(idx) {
                continue;
            }

            let weight = seed.node_weight_mut(*idx).unwrap();
            let new_weight = format!("fin_{}", *weight);
            *weight = new_weight.clone();

            last_fin_set.insert(*idx);
        }

        Self {
            graph: seed,
            last_ini_set,
            last_fin_set,
        }
    }

    pub fn dot(&self) -> String {
        format!("{}", Dot::new(&self.graph))
    }

    pub fn dot_with_ini_cones(&mut self) -> String {
        let mut edges_to_color = HashSet::new();
        let mut nodes_to_color = HashSet::new();

        for el in self.get_ini_set() {
            let cone = self.get_cone(el, Outgoing);
            edges_to_color = edges_to_color.union(&cone.edges).cloned().collect();
            nodes_to_color = nodes_to_color.union(&cone.nodes).cloned().collect();
        }

        self.color_dot(
            format!("{}", Dot::new(&self.graph)),
            nodes_to_color,
            edges_to_color,
        )
    }

    pub fn dot_with_fin_cones(&mut self) -> String {
        let mut edges_to_color = HashSet::new();
        let mut nodes_to_color = HashSet::new();

        for el in self.get_fin_set() {
            let cone = self.get_cone(el, Incoming);
            edges_to_color = edges_to_color.union(&cone.edges).cloned().collect();
            nodes_to_color = nodes_to_color.union(&cone.nodes).cloned().collect();
        }

        self.color_dot(
            format!("{}", Dot::new(&self.graph)),
            nodes_to_color,
            edges_to_color,
        )
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

    pub fn diamond_filter(&mut self) {
        let mut ini_union_cone = HashSet::new();
        // gather diamond of all children of inis
        for el in self.get_ini_set() {
            ini_union_cone = ini_union_cone
                .union(&self.get_cone(el, Outgoing).nodes)
                .cloned()
                .collect();
        }

        let mut fin_union_cone = HashSet::new();
        // gather diamond of all parents of fins
        for el in self.get_fin_set() {
            fin_union_cone = fin_union_cone
                .union(&self.get_cone(el, Incoming).nodes)
                .cloned()
                .collect();
        }

        let intersection = ini_union_cone
            .intersection(&fin_union_cone)
            .cloned()
            .collect::<HashSet<NodeIndex>>();

        self.graph
            .retain_nodes(|_, node| intersection.contains(&node));

        self.reset_mem();
    }

    fn reset_mem(&mut self) {
        self.last_ini_set = HashSet::default();
        self.last_fin_set = HashSet::default();
    }

    fn get_ini_set(&mut self) -> HashSet<NodeIndex> {
        if !self.last_ini_set.is_empty() {
            return self.last_ini_set.clone();
        }

        let ini_set = self.collect_ini_set();
        self.last_ini_set = ini_set.clone();
        ini_set
    }

    fn get_fin_set(&mut self) -> HashSet<NodeIndex> {
        if !self.last_fin_set.is_empty() {
            return self.last_fin_set.clone();
        }

        let fin_set = self.collect_fin_set();
        self.last_fin_set = fin_set.clone();
        fin_set
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

    fn get_cone(&self, root: NodeIndex, dir: Direction) -> Cone {
        let mut nodes = HashSet::new();
        let mut edges = HashSet::new();

        nodes.insert(root);

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
            let mut next_connected = vec![];

            connected.drain(..).for_each(|sibling| {
                if nodes.contains(&sibling) {
                    return;
                }
                nodes.insert(sibling);

                let mut new_connected = self
                    .graph
                    .edges_directed(sibling, dir)
                    .map(|edge| {
                        edges.insert(edge.id());

                        match dir {
                            Outgoing => edge.target(),
                            Incoming => edge.source(),
                        }
                    })
                    .collect::<Vec<NodeIndex>>();

                next_connected.append(&mut new_connected);
            });

            connected = next_connected;
        }

        Cone { nodes, edges }
    }
}

fn color_line(line: String) -> String {
    let first_part = line.replace("]", "");
    format!("{first_part}, color=red ]")
}

struct Cone {
    nodes: HashSet<NodeIndex>,
    edges: HashSet<EdgeIndex>,
}
