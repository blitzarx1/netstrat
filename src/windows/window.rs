use egui::Ui;

pub trait AppWindow {
    fn toggle_btn(&mut self, ui: &mut Ui);
    fn show(&mut self, ui: &mut Ui);
}
