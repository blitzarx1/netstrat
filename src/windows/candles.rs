use std::sync::{Mutex, Arc};

use crossbeam::channel::{unbounded, Sender};
use eframe::emath::Align;
use egui::{Layout, Ui, Window};
use egui_extras::{Size, StripBuilder};
use tracing::info;

use crate::{
    netstrat::Drawer,
    widgets::{
        candles::{Props, Symbols},
        AppWidget,
    },
};

use super::window::AppWindow;

pub struct SymbolsGraph {
    graph: Props,
    symbols: Symbols,
    visible: bool,
}

impl AppWindow for SymbolsGraph {
    fn toggle_btn(&mut self, ui: &mut Ui) {
        if ui.button("graph").clicked() {
            self.update(!self.visible);
        }
    }

    fn show(&mut self, ui: &mut Ui) {
        let mut visible = self.visible;
        Window::new("graph")
            .open(&mut visible)
            .show(ui.ctx(), |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    StripBuilder::new(ui)
                        .size(Size::relative(0.2).at_most(200.0))
                        .size(Size::remainder())
                        .horizontal(|mut strip| {
                            strip.cell(|ui| {
                                self.symbols.show(ui);
                            });
                            strip.cell(|ui| {
                                self.graph.show(ui);
                            });
                        })
                })
            });

        self.update(visible);
    }
}

impl SymbolsGraph {
    pub fn new(drawer_pub: Sender<Arc<Mutex<Box<dyn Drawer>>>>, visible: bool) -> Self {
        info!("initing window graph");

        let (s, r) = unbounded();
        Self {
            graph: Props::new(r, drawer_pub),
            symbols: Symbols::new(s),
            visible,
        }
    }

    fn update(&mut self, visible: bool) {
        if visible != self.visible {
            self.visible = visible;
            match visible {
                true => info!("opening graph window..."),
                false => info!("closing graph window..."),
            }
        }
    }
}
