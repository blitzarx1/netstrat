use crossbeam::channel::{Receiver, Sender};
use egui::{Layout, Ui, Window};
use egui_extras::{Size, StripBuilder};

use super::window::AppWindow;
use crate::widgets::{Graph, Symbols};

pub struct SymbolsGraph {
    graph: Graph,
    symbols: Symbols,
    visible: bool,
}

impl AppWindow for SymbolsGraph {
    fn toggle_btn(&mut self, ui: &mut Ui) {
        if ui.button("graph").clicked() {
            self.visible = !self.visible
        }
    }

    fn show(&mut self, ui: &mut Ui) {
        Window::new("graph")
            .open(&mut self.visible)
            .min_height(500.0)
            .min_width(700.0)
            .show(ui.ctx(), |ui| {
                ui.with_layout(Layout::left_to_right(), |ui| {
                    StripBuilder::new(ui)
                        .size(Size::relative(0.2).at_most(200.0))
                        .size(Size::remainder())
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui.add(&mut self.symbols);
                            });
                            strip.cell(|ui| {
                                ui.add(&mut self.graph);
                            });
                        })
                })
            });
    }
}

impl SymbolsGraph {
    pub fn new(s: Sender<String>, r: Receiver<String>, visible: bool) -> Self {
        Self {
            graph: Graph::new(r),
            symbols: Symbols::new(s),
            visible: visible,
        }
    }
}
