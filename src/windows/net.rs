use crate::AppWindow;
use egui::{Ui, Window};

pub struct Net {
    visible: bool,
}

impl Net {
    pub fn new(visible: bool) -> Self {
        Self { visible }
    }

    fn update(&mut self, visible: bool) {
        if visible != self.visible {
            self.visible = visible
        }
    }
}

impl AppWindow for Net {
    fn toggle_btn(&mut self, ui: &mut Ui) {
        if ui.button("net").clicked() {
            self.update(!self.visible)
        }
    }

    fn show(&mut self, ui: &mut Ui) {
        Window::new("net").show(ui.ctx(), |ui| ui.label("net window"));
    }
}
