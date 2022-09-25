use egui::Ui;

pub trait AppWidget {
    fn show(&mut self, ui: &mut Ui);
}
