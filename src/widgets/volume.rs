use std::ops::RangeInclusive;

use chrono::{DateTime, NaiveDateTime, Utc};
use egui::{
    plot::{Bar, BarChart, LinkedAxisGroup, Plot},
    Color32, Vec2, Widget,
};

use crate::{netstrat::data::Data, sources::binance::Kline};

#[derive(Clone)]
pub struct Volume {
    data: Data,
    val: Vec<Bar>,
    axes_group: LinkedAxisGroup,
    enabled: bool,
}

impl Default for Volume {
    fn default() -> Self {
        Self {
            data: Data::new_volume(),
            val: Default::default(),
            axes_group: LinkedAxisGroup::new(false, false),
            enabled: true,
        }
    }
}

impl Volume {
    pub fn new(axes_group: LinkedAxisGroup) -> Self {
        Self {
            axes_group,
            ..Default::default()
        }
    }

    pub fn add_data(&mut self, vals: &mut Vec<Kline>) {
        self.data.append(vals);
        self.val = self
            .data
            .vals
            .iter()
            .map(|k| {
                Bar::new((k.t_open + k.t_close) as f64 / 2.0, k.volume as f64)
                    .width((k.t_open - k.t_close) as f64 * 0.9)
                    .fill(Color32::LIGHT_GREEN.linear_multiply(0.5))
            })
            .collect();
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub(crate) fn clear(&mut self) {
        self.data = Data::new_volume();
    }
}

impl Widget for &Volume {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.add_enabled_ui(self.enabled, |ui| {
            Plot::new("volume")
                .link_axis(self.axes_group.clone())
                .x_axis_formatter(|v: f64, _: &RangeInclusive<f64>| format_ts(v))
                .label_formatter(|_, v| format_ts(v.x))
                .set_margin_fraction(Vec2::new(0.05, 0.5))
                .include_y(self.data.max_y())
                .allow_scroll(false)
                .allow_boxed_zoom(false)
                .allow_drag(false)
                .allow_zoom(false)
                .show_axes([false, false])
                .show(ui, |plot_ui| {
                    plot_ui.bar_chart(
                        BarChart::new(self.val.clone())
                            .element_formatter(Box::new(|b, _| {
                                format!("{}\n{}", b.value, format_ts(b.argument))
                            }))
                            .vertical(),
                    );
                })
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
