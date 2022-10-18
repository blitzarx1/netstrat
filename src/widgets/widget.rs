use egui::Ui;

pub trait AppWidget {
    // TODO: add update method with &mut self and change show to &self
    fn show(&mut self, ui: &mut Ui);
}
