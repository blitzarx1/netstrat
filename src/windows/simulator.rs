use egui::Window;

use super::AppWindow;
use crate::{
    netstrat::Bus,
    widgets::{AppWidget, SimulationProps as WidgetSimulator},
};

const WINDOW_NAME: &str = "simulator";

pub struct Simulator {
    visible: bool,
    widget: WidgetSimulator,
}

impl Simulator {
    pub fn new(visible: bool, bus: Bus) -> Self {
        Self {
            visible,
            widget: WidgetSimulator::new(bus),
        }
    }

    fn update(&mut self, visible: bool) {
        self.visible = visible;
    }
}

impl AppWindow for Simulator {
    fn toggle_btn(&mut self, ui: &mut egui::Ui) {
        if ui.button(WINDOW_NAME).clicked() {
            self.update(!self.visible);
        }
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        let mut visible = self.visible;

        Window::new(WINDOW_NAME)
            .open(&mut visible)
            .show(ui.ctx(), |ui| self.widget.show(ui));

        self.update(visible);
    }
}
