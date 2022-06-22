use egui::Widget;

use crate::sources::binance::client::Kline;

#[derive(Default)]
pub struct Volume {
    data: Vec<Kline>,
}

impl Volume {
    pub fn new(data: Vec<Kline>) -> Self {
        Self { data }
    }
}

impl Widget for Volume {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        todo!()
    }
}
