use chrono::{DateTime, Utc};
use crossbeam::channel::{unbounded, Sender};
use egui::{
    plot::{BoxElem, BoxPlot, BoxSpread, Plot},
    Color32, Response, Stroke, Widget,
};
use tracing::{error, info};

use crate::{
    netstrat::{bounds::Bounds, data::Data},
    sources::binance::Kline,
};

const BOUNDS_SEND_DELAY_MILLIS: i64 = 300;

pub struct Candles {
    data: Data,
    val: Vec<BoxElem>,
    bounds_pub: Sender<Bounds>,
    incremental_drag_diff: f32,
    last_time_drag_happened: DateTime<Utc>,
    drag_happened: bool,
    bounds: Bounds,
    enabled: bool,
}

impl Default for Candles {
    fn default() -> Self {
        let (s_bounds, _) = unbounded();

        Self {
            data: Data::new_candle(),
            val: Default::default(),
            bounds_pub: s_bounds,
            last_time_drag_happened: Utc::now(),
            drag_happened: Default::default(),
            bounds: Bounds(0, 0),
            incremental_drag_diff: 0.0,
            enabled: true,
        }
    }
}

impl Candles {
    pub fn new(bounds_pub: Sender<Bounds>) -> Self {
        Self {
            bounds_pub,
            ..Default::default()
        }
    }

    pub fn get_data(&self) -> Data {
        self.data.clone()
    }

    pub fn add_data(&mut self, vals: &mut Vec<Kline>) {
        self.data.append(vals);
        self.val = self
            .data
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
                .name(Data::format_ts(k.t_close as f64))
                .stroke(Stroke::new(1.0, Data::k_color(k)))
                .fill(Data::k_color(k))
                .whisker_width(0.0)
                .box_width((k.t_open - k.t_close) as f64 * 0.9)
            })
            .collect();
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled
    }

    pub fn clear(&mut self) {
        self.data = Data::new_candle();
    }
}

impl Widget for &mut Candles {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        if self.drag_happened
            && Utc::now()
                .signed_duration_since(self.last_time_drag_happened)
                .num_milliseconds()
                > BOUNDS_SEND_DELAY_MILLIS
        {
            let msg = self.bounds;
            let send_res = self.bounds_pub.send(msg);
            match send_res {
                Ok(_) => info!("sent bounds: {msg:?}"),
                Err(err) => error!("failed to send bounds: {err}"),
            }

            self.drag_happened = false;
        }
        ui.add_enabled_ui(self.enabled, |ui| {
            Plot::new("candles")
                .label_formatter(|_, v| -> String { Data::format_ts(v.x) })
                .x_axis_formatter(|v, _range| Data::format_ts(v))
                .show(ui, |plot_ui| {
                    plot_ui.box_plot(
                        BoxPlot::new(self.val.clone())
                            .element_formatter(Box::new(|el, _| -> String {
                                format!(
                                    "open: {:.8}\nclose: {:.8}\nhigh: {:.8}\nlow: {:.8}\n{}",
                                    {
                                        match el.fill == Color32::LIGHT_RED {
                                            true => el.spread.quartile3,
                                            false => el.spread.quartile1,
                                        }
                                    },
                                    {
                                        match el.fill == Color32::LIGHT_RED {
                                            true => el.spread.quartile1,
                                            false => el.spread.quartile3,
                                        }
                                    },
                                    el.spread.upper_whisker,
                                    el.spread.lower_whisker,
                                    Data::format_ts(el.argument),
                                )
                            }))
                            .vertical(),
                    );

                    let plot_bounds = plot_ui.plot_bounds();
                    self.bounds = Bounds(plot_bounds.min()[0] as i64, plot_bounds.max()[0] as i64);

                    let drag_diff = plot_ui.pointer_coordinate_drag_delta().x;
                    if drag_diff.abs() > 0.0 {
                        self.incremental_drag_diff += drag_diff;

                        // TODO: use step to count min drag diff
                        if self.incremental_drag_diff > (60 * 1000 * 5) as f32 {
                            self.drag_happened = true;
                            self.last_time_drag_happened = Utc::now();
                            self.incremental_drag_diff = 0.0;
                        }
                    }

                    plot_ui.ctx().request_repaint();
                })
        })
        .response
    }
}
