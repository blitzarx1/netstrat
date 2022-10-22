use egui::Button;
use egui_extras::image::load_svg_bytes;
use graphviz_rust::{
    cmd::{CommandArg, Format},
    exec, parse,
    printer::PrinterContext,
};
use petgraph::{
    dot::Dot,
    stable_graph::{NodeIndex, StableDiGraph},
    visit::EdgeRef,
    Direction::{Incoming, Outgoing},
};

use crate::{widgets::{AppWidget, image_drawer}, netstrat::Drawer};

use super::{Clicks, Step};

pub struct Builder {
    tree: StableDiGraph<Step, usize>,
    initial_step: Option<usize>,
    root: Option<usize>,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            tree: StableDiGraph::default(),
            initial_step: None,
            root: None,
        }
    }

    pub fn with_initial_step(&mut self, history_step: Step) {
        let root_idx = self.tree.add_node(history_step).index();
        self.root = Some(root_idx);
        self.initial_step = Some(root_idx);
    }

    pub fn build(&self) -> History {
        let mut h = History {
            max_gen: 0,
            root: self.root,
            tree: self.tree.clone(),
            current_step: self.initial_step,
            last_click: None,
            drawer: Default::default(),
        };

        h.update_image();

        h
    }
}

pub struct History {
    tree: StableDiGraph<Step, usize>,
    current_step: Option<usize>,
    max_gen: usize,
    drawer: image_drawer::ImageDrawer,
    root: Option<usize>,
    last_click: Option<Clicks>,
}

impl History {
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn last_click(&self) -> &Option<Clicks> {
        &self.last_click
    }

    pub fn drawer(&self) -> image_drawer::ImageDrawer {
        self.drawer.clone()
    }

    pub fn new_with_initial_step(history_step: Step) -> Self {
        let mut history_builder = History::builder();
        history_builder.with_initial_step(history_step);
        history_builder.build()
    }

    pub fn get_current_step(&self) -> Option<usize> {
        self.current_step
    }

    pub fn root(&self) -> Option<usize> {
        self.root
    }

    pub fn is_parent_intersection(&self, idx: usize) -> bool {
        if self.root().unwrap() == idx {
            return false;
        }

        let parent_idx = self
            .tree
            .edges_directed(
                NodeIndex::from(self.get_current_step().unwrap() as u32),
                Incoming,
            )
            .next()
            .unwrap()
            .source();

        self.tree.edges_directed(parent_idx, Outgoing).count() > 1
    }

    pub fn go_up(&mut self) -> Option<Step> {
        let parent_idx = self
            .tree
            .edges_directed(NodeIndex::from(self.get_current_step()? as u32), Incoming)
            .next()?
            .source();

        self.current_step = Some(parent_idx.index());
        self.update_image();

        self.tree.node_weight(parent_idx).cloned()
    }

    pub fn go_down(&mut self) -> Option<Step> {
        let child_idx = self
            .tree
            .edges_directed(NodeIndex::from(self.get_current_step()? as u32), Outgoing)
            .next()?
            .target();

        self.current_step = Some(child_idx.index());
        self.update_image();

        self.tree.node_weight(child_idx).cloned()
    }

    pub fn go_sibling(&mut self) -> Option<Step> {
        let parent_idx = self
            .tree
            .edges_directed(NodeIndex::from(self.get_current_step()? as u32), Incoming)
            .next()?
            .source();

        let current_gen = self.get_generation(self.get_current_step()?)?;
        let next_gen_wrapped = self
            .tree
            .edges_directed(parent_idx, Outgoing)
            .filter(|e| *e.weight() == current_gen + 1)
            .map(|e| e.target())
            .next();

        let mut new_current: NodeIndex;
        match next_gen_wrapped {
            Some(next_gen) => {
                new_current = next_gen;
            }
            None => {
                new_current = self
                    .tree
                    .edges_directed(parent_idx, Outgoing)
                    .min_by(|e1, e2| e1.weight().cmp(e2.weight()))
                    .unwrap()
                    .target();
            }
        };

        self.current_step = Some(new_current.index());
        self.update_image();

        self.tree.node_weight(new_current).cloned()
    }

    /// adds step to history creating new generation or extending current one
    pub fn add_and_set_current_step(&mut self, history_step: Step) -> Option<()> {
        let new_node_idx = self.tree.add_node(history_step);

        match self.is_leaf(self.current_step?) {
            true => self.push(new_node_idx),
            false => self.branch(new_node_idx),
        };

        self.current_step = Some(new_node_idx.index());
        self.update_image();

        Some(())
    }

    fn update_image(&mut self) {
        let graph_svg = exec(
            parse(self.dot().as_str()).unwrap(),
            &mut PrinterContext::default(),
            vec![CommandArg::Format(Format::Svg)],
        )
        .unwrap();

        let image = load_svg_bytes(graph_svg.as_bytes()).unwrap();
        self.drawer.update_image(image);
    }

    fn get_generation(&self, idx: usize) -> Option<usize> {
        Some(
            *self
                .tree
                .edges_directed(NodeIndex::from(idx as u32), Incoming)
                .next()?
                .weight(),
        )
    }

    fn push(&mut self, new_node_idx: NodeIndex) -> Option<()> {
        let current_gen = match self.get_current_step()? == self.root()? {
            true => 0,
            false => self.get_generation(self.get_current_step()?)?,
        };

        self.tree.add_edge(
            NodeIndex::from(self.get_current_step()? as u32),
            new_node_idx,
            current_gen,
        );

        Some(())
    }

    fn branch(&mut self, new_node_idx: NodeIndex) -> Option<()> {
        let next_generation = self.max_gen + 1;

        self.tree.add_edge(
            NodeIndex::from(self.get_current_step()? as u32),
            new_node_idx,
            next_generation,
        );

        self.max_gen = next_generation;

        Some(())
    }

    /// shows if current step is a leaf in the tree
    pub fn is_leaf(&self, idx: usize) -> bool {
        self.tree
            .edges_directed(NodeIndex::from(idx as u32), Outgoing)
            .count()
            == 0
    }

    pub fn dot(&self) -> String {
        // TODO: use the same method to color graph dot.
        Dot::with_attr_getters(&self.tree, &[], &|g, r| String::new(), &|g, r| {
            if r.0.index() == self.current_step.unwrap() {
                return "color=red".to_string();
            }

            String::new()
        })
        .to_string()
    }

    pub fn get(&self, step: usize) -> Option<Step> {
        self.tree.node_weight(NodeIndex::from(step as u32)).cloned()
    }
}

impl AppWidget for History {
    fn show(&mut self, ui: &mut egui::Ui) {
        let is_root = self.get_current_step().unwrap() == self.root().unwrap();
        let is_leaf = self.is_leaf(self.get_current_step().unwrap());
        let is_parent_intersection = self.is_parent_intersection(self.get_current_step().unwrap());
        let mut click = None;

        ui.collapsing("History", |ui| {
            ui.horizontal_top(|ui| {
                if ui.add_enabled(!is_root, Button::new("⏶")).clicked() {
                    click = Some(Clicks::Up);
                };
                if ui.add_enabled(!is_leaf, Button::new("⏷")).clicked() {
                    click = Some(Clicks::Down);
                };
                if ui
                    .add_enabled(is_parent_intersection, Button::new("▶"))
                    .clicked()
                {
                    click = Some(Clicks::Right);
                };
            });
            ui.add_space(5.0);
            ui.add_space(10.0);
            self.drawer().show(ui);
        });

        self.last_click = click;
    }
}
