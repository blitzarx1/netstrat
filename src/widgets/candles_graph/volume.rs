use std::ops::RangeInclusive;

use chrono::{DateTime, NaiveDateTime, Utc};
use egui::{
    plot::{Bar, BarChart, Plot},
    Color32, Widget,
};

use super::data::Data;

#[derive(Default, Clone)]
pub struct Volume {
    data: Data,
    val: Vec<Bar>,
}

impl Volume {
    pub fn new(data: Data) -> Self {
        let val = data
            .vals
            .iter()
            .map(|k| {
                Bar::new((k.t_open + k.t_close) as f64 / 2.0, k.volume as f64)
                    .width((k.t_open - k.t_close) as f64 * 0.9)
                    .fill(Color32::LIGHT_GREEN.linear_multiply(0.5))
            })
            .collect();

        Self { data, val }
    }
}

impl Widget for &Volume {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        Plot::new("box plot")
            .x_axis_formatter(|v: f64, _: &RangeInclusive<f64>| format_ts(v))
            .include_x(self.data.max_x())
            .include_y(self.data.max_y())
            .show(ui, |plot_ui| {
                plot_ui.bar_chart(BarChart::new(self.val.clone()).vertical());
            })
            .response
    }
}

fn format_ts(ts: f64) -> String {
    let secs = (ts / 1000f64) as i64;
    let naive = NaiveDateTime::from_timestamp(secs, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}
