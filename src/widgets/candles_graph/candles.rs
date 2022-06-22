use chrono::{DateTime, NaiveDateTime, Utc};
use egui::{
    plot::{BoxElem, BoxPlot, BoxSpread, Plot},
    Color32, Response, Stroke, Widget,
};

use super::data::Data;
use crate::sources::binance::client::Kline;

#[derive(Default, Clone)]
pub struct Candles {
    data: Data,
    val: Vec<BoxElem>,
}

impl Candles {
    pub fn new(data: Data) -> Self {
        let val: Vec<BoxElem> = data
            .vals
            .iter()
            .map(|k| -> BoxElem {
                BoxElem::new(
                    (k.t_open + k.t_close) as f64 / 2.0,
                    BoxSpread::new(
                        k.low as f64,
                        {
                            match k.open > k.close {
                                true => k.close as f64,
                                false => k.open as f64,
                            }
                        },
                        k.open as f64, // we don't need to see median for candle
                        {
                            match k.open > k.close {
                                true => k.open as f64,
                                false => k.close as f64,
                            }
                        },
                        k.high as f64,
                    ),
                )
                .name(format_ts(k.t_close as f64))
                .stroke(Stroke::new(1.0, {
                    if k.open < k.close {
                        Color32::LIGHT_GREEN
                    } else {
                        Color32::LIGHT_RED
                    }
                }))
                .fill({
                    if k.open < k.close {
                        Color32::LIGHT_GREEN.linear_multiply(0.5)
                    } else {
                        Color32::LIGHT_RED.linear_multiply(0.5)
                    }
                })
                .whisker_width(0.0)
                .box_width((k.t_open - k.t_close) as f64 * 0.9)
            })
            .collect();

        Self { data, val }
    }
}

impl Widget for &Candles {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        Plot::new("box plot")
            .x_axis_formatter(|v, _range| format_ts(v))
            .include_x(self.data.max_x())
            .include_y(self.data.max_y())
            .show(ui, |plot_ui| {
                plot_ui.box_plot(BoxPlot::new(self.val.clone()).vertical());
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
