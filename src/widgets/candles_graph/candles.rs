use egui::Widget;

use crate::sources::binance::client::Kline;

#[derive(Default)]
pub struct Candles {
    data: Vec<Kline>,
}

impl Candles {
    pub fn new(data: Vec<Kline>) -> Self {
        Self { data }
    }
}

impl Widget for Candles {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        todo!()
    }
}
