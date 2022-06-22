use std::ops::RangeInclusive;

use chrono::{DateTime, NaiveDateTime, Utc};
use egui::{
    plot::{Bar, BarChart, LinkedAxisGroup, Plot},
    Color32, Vec2, Widget,
};

use super::data::Data;

#[derive(Clone)]
pub struct Volume {
    data: Data,
    val: Vec<Bar>,
    axes_group: LinkedAxisGroup,
}

impl Default for Volume {
    fn default() -> Self {
        Self {
            data: Default::default(),
            val: Default::default(),
            axes_group: LinkedAxisGroup::new(false, false),
        }
    }
}

impl Volume {
    pub fn new(data: Data, axes_group: LinkedAxisGroup) -> Self {
        let val = data
            .vals
            .iter()
            .map(|k| {
                Bar::new((k.t_open + k.t_close) as f64 / 2.0, k.volume as f64)
                    .width((k.t_open - k.t_close) as f64 * 0.9)
                    .fill(Color32::LIGHT_GREEN.linear_multiply(0.5))
            })
            .collect();

        Self {
            data,
            val,
            axes_group,
        }
    }
}

impl Widget for &Volume {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        Plot::new("volume")
            .link_axis(self.axes_group.clone())
            .x_axis_formatter(|v: f64, _: &RangeInclusive<f64>| format_ts(v))
            .label_formatter(|_, v| format!("{}", format_ts(v.x)))
            .include_x(self.data.max_x())
            .include_y(self.data.max_y())
            .allow_scroll(false)
            .allow_boxed_zoom(false)
            .allow_drag(false)
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
            .response
    }
}

fn format_ts(ts: f64) -> String {
    let secs = (ts / 1000f64) as i64;
    let naive = NaiveDateTime::from_timestamp(secs, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}
