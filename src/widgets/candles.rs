use chrono::{DateTime, Utc};
use crossbeam::channel::{unbounded, Sender};
use egui::{
    plot::{BoxElem, BoxPlot, BoxSpread, LinkedAxisGroup, Plot, PlotBounds},
    Color32, Id, Response, Stroke, Vec2, Widget,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, enabled, error, info};

use crate::netstrat::{bounds::Bounds, data::Data};

const BOUNDS_SEND_DELAY_MILLIS: i64 = 300;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CachedPlotData {
    auto_bounds: bool,
    min_auto_bounds: PlotBounds,
}

pub struct Candles {
    data: Data,
    val: Vec<BoxElem>,
    axes_group: LinkedAxisGroup,
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
            data: Default::default(),
            val: Default::default(),
            axes_group: LinkedAxisGroup::new(false, false),
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
    pub fn new(axes_group: LinkedAxisGroup, bounds_pub: Sender<Bounds>) -> Self {
        Self {
            axes_group,
            bounds_pub,
            ..Default::default()
        }
    }

    pub fn set_data(&mut self, data: Data) {
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
                .name(Data::format_ts(k.t_close as f64))
                .stroke(Stroke::new(1.0, Data::k_color(k)))
                .fill(Data::k_color(k))
                .whisker_width(0.0)
                .box_width((k.t_open - k.t_close) as f64 * 0.9)
            })
            .collect();

        self.data = data;
        self.val = val;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled
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
            let msg = self.bounds.clone();
            let send_res = self.bounds_pub.send(msg.clone());
            match send_res {
                Ok(_) => info!("Sent bounds: {msg:?}."),
                Err(err) => error!("Failed to send bounds: {err}."),
            }

            self.drag_happened = false;
        }
        ui.add_enabled_ui(self.enabled, |ui| {
            Plot::new("candles")
                .link_axis(self.axes_group.clone())
                .label_formatter(|_, v| -> String { format!("{}", Data::format_ts(v.x)) })
                .x_axis_formatter(|v, _range| Data::format_ts(v))
                .include_x(self.data.max_x())
                .include_x(self.data.min_x())
                .set_margin_fraction(Vec2::new(0.05, 0.05))
                .include_y(self.data.max_y())
                .include_y(self.data.min_y())
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
