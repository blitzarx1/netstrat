use crossbeam::channel::{unbounded, Receiver};
use std::cmp::Ordering;

use chrono::{prelude::*, Duration};
use egui::{
    plot::{BoxElem, BoxPlot, BoxSpread, Plot},
    Color32, ProgressBar, Response, Stroke, Ui, Widget, Window,
};
use poll_promise::Promise;

use crate::sources::binance::{
    client::{Client, Kline},
    interval::Interval,
};

static MAX_LIMIT: u32 = 1000;

#[derive(Debug, Clone, Copy)]
struct GraphProps {
    date_start: chrono::Date<Utc>,
    date_end: chrono::Date<Utc>,
    interval: Interval,
    limit: usize,
}

impl Default for GraphProps {
    fn default() -> Self {
        Self {
            date_start: Date::from(Utc::now().date()) - Duration::days(1),
            date_end: Date::from(Utc::now().date()),
            interval: Interval::Minute,
            limit: 1000,
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
struct GraphLoadingState {
    triggered: bool,
    initial: GraphProps,
    received: u32,
    pages: u32,
    last_page_limit: usize,
}

impl GraphLoadingState {
    fn from_graph_props(props: &GraphProps) -> Self {
        let diff_days = props.date_end - props.date_start;

        // debug!("start: {:?}, end: {:?}", props.date_start, props.date_end);

        match props.interval {
            Interval::Minute => {
                let pages_proto = Duration::num_minutes(&diff_days) as f32 / MAX_LIMIT as f32;
                let pages = pages_proto.ceil() as u32;
                let last_page_limit = (pages_proto.fract() * MAX_LIMIT as f32) as usize;

                GraphLoadingState {
                    triggered: false,
                    initial: props.clone(),
                    pages,
                    received: 0,
                    last_page_limit,
                }
            }
            Interval::Hour => {
                let pages_proto = Duration::num_hours(&diff_days) as f32 / MAX_LIMIT as f32;
                let pages = pages_proto.ceil() as u32;
                let last_page_limit = (pages_proto.fract() * MAX_LIMIT as f32) as usize;

                GraphLoadingState {
                    triggered: false,
                    initial: props.clone(),
                    pages,
                    received: 0,
                    last_page_limit,
                }
            }
            Interval::Day => {
                let pages_proto = Duration::num_days(&diff_days) as f32 / MAX_LIMIT as f32;
                let pages = pages_proto.ceil() as u32;
                let last_page_limit = (pages_proto.fract() * MAX_LIMIT as f32) as usize;

                GraphLoadingState {
                    triggered: false,
                    initial: props.clone(),
                    pages,
                    received: 0,
                    last_page_limit,
                }
            }
        }
    }

    fn left_edge(&self) -> DateTime<Utc> {
        let covered: Duration;

        match self.initial.interval {
            Interval::Minute => {
                covered = Duration::minutes((self.received * self.initial.limit as u32) as i64)
            }
            Interval::Hour => {
                covered = Duration::hours((self.received * self.initial.limit as u32) as i64)
            }
            Interval::Day => {
                covered = Duration::days((self.received * self.initial.limit as u32) as i64)
            }
        };

        self.initial.date_start.and_hms(0, 0, 0) + covered
    }

    fn received(&mut self) {
        self.received += 1;
    }

    fn is_finished(&self) -> bool {
        return self.progress() == 1f32;
    }

    fn progress(&self) -> f32 {
        if self.pages == 0 {
            return 1f32;
        }
        self.received as f32 / self.pages as f32
    }

    fn is_last_page(&self) -> bool {
        return self.pages - self.received == 1;
    }
}

pub struct CandlePlot {
    symbol: String,
    klines: Vec<Kline>,
    graph_props: GraphProps,
    graph_loading_state: GraphLoadingState,
    klines_promise: Option<Promise<Vec<Kline>>>,
    symbol_chan: Receiver<String>,
}

impl Default for CandlePlot {
    fn default() -> Self {
        let (_, r) = unbounded();
        Self {
            symbol: Default::default(),
            klines: Default::default(),
            graph_props: Default::default(),
            graph_loading_state: Default::default(),
            klines_promise: Default::default(),
            symbol_chan: r,
        }
    }
}

impl CandlePlot {
    pub fn new(symbol_chan: Receiver<String>) -> Self {
        Self {
            symbol_chan: symbol_chan,
            ..Default::default()
        }
    }
}

impl Widget for &mut CandlePlot {
    fn ui(self, ui: &mut Ui) -> Response {
        let sym_wrapped = self
            .symbol_chan
            .recv_timeout(std::time::Duration::from_millis(10));

        match sym_wrapped {
            Ok(symbol) => {
                self.symbol = symbol;
                self.graph_loading_state = Default::default();
            }
            Err(_) => {}
        }

        if self.symbol == "" {
            return ui.label("select symbol");
        }

        if let Some(promise) = &self.klines_promise {
            if let Some(result) = promise.ready() {
                self.graph_loading_state.received();

                if self.graph_loading_state.received == 1 {
                    self.klines = result.iter().map(|k| -> Kline { *k }).collect();
                }

                self.klines_promise = None;

                if !self.graph_loading_state.is_finished() {
                    let start = self
                        .graph_loading_state
                        .left_edge()
                        .timestamp_millis()
                        .clone();

                    let symbol = self.symbol.to_string();
                    let interval = self.graph_props.interval.clone();
                    let mut limit = self.graph_props.limit.clone();
                    if self.graph_loading_state.is_last_page() {
                        limit = self.graph_loading_state.last_page_limit
                    }

                    self.klines_promise = Some(Promise::spawn_async(async move {
                        Client::kline(symbol, interval, start, limit).await
                    }));
                }
            }
        } else if !self.graph_loading_state.triggered {
            self.graph_loading_state = GraphLoadingState::from_graph_props(&self.graph_props);
            self.graph_loading_state.triggered = true;

            let interval = self.graph_props.interval.clone();
            let start = self
                .graph_loading_state
                .left_edge()
                .timestamp_millis()
                .clone();

            let mut limit = self.graph_props.limit.clone();
            if self.graph_loading_state.is_last_page() {
                limit = self.graph_loading_state.last_page_limit;
            }

            let symbol = self.symbol.to_string();

            self.klines_promise = Some(Promise::spawn_async(async move {
                Client::kline(symbol, interval, start, limit).await
            }));
        }

        if !self.graph_loading_state.is_finished() {
            return ui
                .centered_and_justified(|ui| {
                    ui.add(
                        ProgressBar::new(self.graph_loading_state.progress())
                            .show_percentage()
                            .animate(true),
                    )
                })
                .response;
        }

        let max_y_kline = self
            .klines
            .iter()
            .max_by(|l, r| {
                if l.close > r.close {
                    return Ordering::Greater;
                }

                Ordering::Less
            })
            .unwrap();
        let max_x_kline = &self.klines[self.klines.len() - 1];

        Window::new(self.symbol.to_string()).show(ui.ctx(), |ui| {
            ui.collapsing("time period", |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.add(
                        egui_extras::DatePickerButton::new(&mut self.graph_props.date_start)
                            .id_source("datepicker_start"),
                    );
                    ui.label("date start");
                });
                ui.horizontal_wrapped(|ui| {
                    ui.add(
                        egui_extras::DatePickerButton::new(&mut self.graph_props.date_end)
                            .id_source("datepicker_end"),
                    );
                    ui.label("date end");
                });
            });
            ui.collapsing("interval", |ui| {
                egui::ComboBox::from_label("pick data interval")
                    .selected_text(format!("{:?}", self.graph_props.interval))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.graph_props.interval, Interval::Day, "Day");
                        ui.selectable_value(&mut self.graph_props.interval, Interval::Hour, "Hour");
                        ui.selectable_value(
                            &mut self.graph_props.interval,
                            Interval::Minute,
                            "Minute",
                        );
                    });
            });
            ui.add_space(5f32);
            if ui.button("apply").clicked() {
                self.graph_loading_state = GraphLoadingState::from_graph_props(&self.graph_props);
                self.graph_loading_state.triggered = true;

                let start = self
                    .graph_props
                    .date_start
                    .and_hms(0, 0, 0)
                    .timestamp_millis()
                    .clone();
                let pair = self.symbol.to_string();
                let interval = self.graph_props.interval.clone();
                let mut limit = self.graph_props.limit.clone();

                if self.graph_loading_state.is_last_page() {
                    limit = self.graph_loading_state.last_page_limit
                }

                self.klines_promise = Some(Promise::spawn_async(async move {
                    Client::kline(pair, interval, start, limit).await
                }));
            }
        });

        let box_data: Vec<BoxElem> = self
            .klines
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
                .name(format_datetime((k.t_close as f64 / 1000f64) as i64))
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

        Plot::new("box plot")
            .x_axis_formatter(|v, _range| format_datetime((v / 1000f64) as i64))
            .include_x(max_x_kline.t_close as f64)
            .include_y(max_y_kline.close)
            .show(ui, |plot_ui| {
                plot_ui.box_plot(BoxPlot::new(box_data).vertical());
            })
            .response
    }
}

fn format_datetime(ts: i64) -> String {
    let naive = NaiveDateTime::from_timestamp(ts, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}
