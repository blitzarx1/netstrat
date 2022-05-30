use std::cmp::Ordering;

use poll_promise::Promise;

use eframe;
use sources::binance::client::Kline;

use chrono::{prelude::*, Duration};
use egui::plot::{Line, Plot, Value, Values};
use egui::widgets::Label;
use egui::{
    CentralPanel, Layout, ProgressBar, ScrollArea, SidePanel, TextEdit, TopBottomPanel, Visuals,
    WidgetText, Window,
};
use sources::binance::interval::Interval;
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::sources::binance::client::{Client, Info};

mod network;
mod sources;
use tokio;

static MAX_LIMIT: u32 = 1000;

#[derive(Default, Debug, Clone, Copy)]
struct GraphLoadingState {
    initial: GraphProps,
    received: u32,
    pages: u32,
    last_page_limit: usize,
}

impl GraphLoadingState {
    fn from_graph_props(props: &GraphProps) -> Self {
        let diff_days = props.date_end - props.date_start;

        debug!("start: {:?}, end: {:?}", props.date_start, props.date_end);

        match props.interval {
            Interval::Minute => {
                let pages_proto = Duration::num_minutes(&diff_days) as f32 / MAX_LIMIT as f32;
                let pages = pages_proto.ceil() as u32;
                let last_page_limit = (pages_proto.fract() * MAX_LIMIT as f32) as usize;

                GraphLoadingState {
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

#[derive(Default, Debug)]
struct GuiKline {
    close: f32,
    ts: i64,
}

#[derive(Default)]
struct GuiPair {
    symbol: String,
    active: bool,
}

#[derive(Default)]
struct TemplateApp {
    filter: FilterProps,
    pairs: Vec<GuiPair>,
    klines: Vec<GuiKline>,
    pairs_loaded: bool,
    loading_pairs: bool,
    selected_pair: String,
    pairs_promise: Option<Promise<Info>>,
    klines_promise: Option<Promise<Vec<Kline>>>,
    debug_visible: bool,
    graph_props: GraphProps,
    dark_mode: bool,
    graph_loading_state: GraphLoadingState,
}

#[derive(Default)]
struct FilterProps {
    value: String,
    active_only: bool,
}

impl TemplateApp {
    /// Called once before the first frame.
    fn new(_ctx: &eframe::CreationContext<'_>) -> Self {
        let pairs = vec![];

        Self {
            pairs,
            dark_mode: true,
            ..Default::default()
        }
    }

    fn render_center_panel(&mut self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| {
            if !self.graph_loading_state.is_finished() {
                ui.centered_and_justified(|ui| {
                    ui.add(
                        ProgressBar::new(self.graph_loading_state.progress())
                            .show_percentage()
                            .animate(true),
                    )
                });
            }

            if self.graph_loading_state.is_finished() && self.klines.len() > 0 {
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

                Window::new(self.selected_pair.to_string()).show(ctx, |ui| {
                    ui.collapsing("time period", |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.add(
                                egui_extras::DatePickerButton::new(
                                    &mut self.graph_props.date_start,
                                )
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
                                ui.selectable_value(
                                    &mut self.graph_props.interval,
                                    Interval::Day,
                                    "Day",
                                );
                                ui.selectable_value(
                                    &mut self.graph_props.interval,
                                    Interval::Hour,
                                    "Hour",
                                );
                                ui.selectable_value(
                                    &mut self.graph_props.interval,
                                    Interval::Minute,
                                    "Minute",
                                );
                            });
                    });
                    ui.add_space(5f32);
                    if ui.button("apply").clicked() {
                        self.graph_loading_state =
                            GraphLoadingState::from_graph_props(&self.graph_props);

                        let start = self
                            .graph_props
                            .date_start
                            .and_hms(0, 0, 0)
                            .timestamp_millis()
                            .clone();
                        let pair = self.selected_pair.to_string();
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

                let line = Line::new(Values::from_values_iter(
                    self.klines
                        .iter()
                        .map(|k| -> Value { Value::new(k.ts as f64, k.close) }),
                ));

                Plot::new("plot")
                    .label_formatter(|_s, v| {
                        format!(
                            "y: {}\nx: {}",
                            v.y,
                            format_datetime((v.x / 1000f64) as i64)
                        )
                        .to_string()
                    })
                    .x_axis_formatter(|v, _range| format_datetime((v / 1000f64) as i64))
                    .include_x(max_x_kline.ts as f64)
                    .include_y(max_y_kline.close)
                    .show(ui, |ui| ui.line(line));
            }
        });
    }

    fn render_top_panel(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            if ui
                .button({
                    match self.dark_mode {
                        true => "🔆",
                        false => "🌙",
                    }
                })
                .clicked()
            {
                self.dark_mode = !self.dark_mode
            }
        });
    }

    fn render_bottom_panel(&mut self, ctx: &egui::Context) {
        TopBottomPanel::bottom("bot panel").show(ctx, |ui| {
            if ui.button("debug").clicked() {
                self.debug_visible = !self.debug_visible;
            }
        });
    }

    fn render_side_panel(&mut self, ctx: &egui::Context) {
        SidePanel::left("side_panel").show(ctx, |ui| {
            if self.loading_pairs {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                });
                return;
            }

            if !self.pairs_loaded {
                self.pairs_promise = Some(Promise::spawn_async(async { Client::info().await }));

                self.pairs_loaded = !self.pairs_loaded;
                self.loading_pairs = true;
                return;
            }

            ui.with_layout(Layout::top_down(egui::Align::LEFT), |ui| {
                if ui.button("back").clicked() {
                    self.pairs_loaded = !self.pairs_loaded;
                }
                ui.add_space(5f32);
                ui.separator();
                ui.add_space(5f32);

                // render filter
                ui.add(
                    TextEdit::singleline(&mut self.filter.value)
                        .hint_text(WidgetText::from("filter pairs").italics()),
                );

                // aply filter
                let filtered: Vec<&GuiPair> = self
                    .pairs
                    .iter()
                    .filter(|s| {
                        let match_value = s
                            .symbol
                            .to_lowercase()
                            .contains(self.filter.value.to_lowercase().as_str());
                        if self.filter.active_only {
                            return match_value && s.active;
                        }
                        match_value
                    })
                    .collect();
                ui.with_layout(Layout::top_down(egui::Align::RIGHT), |ui| {
                    ui.checkbox(&mut self.filter.active_only, "active only");
                    ui.add(Label::new(
                        WidgetText::from(format!("{}/{}", filtered.len(), self.pairs.len()))
                            .small(),
                    ));
                });

                ui.add_space(5f32);

                // render pairs list
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.with_layout(Layout::top_down(egui::Align::LEFT), |ui| {
                            filtered.iter().for_each(|s| {
                                let symbol_for_klines_request = s.symbol.to_string();
                                let label = ui.selectable_label(
                                    s.symbol == self.selected_pair,
                                    match s.active {
                                        true => WidgetText::from(s.symbol.to_string()).strong(),
                                        false => {
                                            WidgetText::from(s.symbol.to_string()).strikethrough()
                                        }
                                    },
                                );

                                if label.clicked() {
                                    self.graph_loading_state =
                                        GraphLoadingState::from_graph_props(&self.graph_props);

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

                                    self.klines_promise = Some(Promise::spawn_async(async move {
                                        Client::kline(
                                            symbol_for_klines_request,
                                            interval,
                                            start,
                                            limit,
                                        )
                                        .await
                                    }));
                                    self.selected_pair = s.symbol.to_string();
                                };
                            });
                        })
                    });
            });
        });
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.dark_mode {
            ctx.set_visuals(Visuals::dark())
        } else {
            ctx.set_visuals(Visuals::light())
        }

        if let Some(promise) = &self.klines_promise {
            if let Some(result) = promise.ready() {
                self.graph_loading_state.received();

                if self.graph_loading_state.received == 1 {
                    self.klines = vec![];
                }

                result.iter().for_each(|k| {
                    self.klines.push(GuiKline {
                        close: k.close,
                        ts: k.t_close,
                    });
                });

                self.klines_promise = None;

                if !self.graph_loading_state.is_finished() {
                    let start = self
                        .graph_loading_state
                        .left_edge()
                        .timestamp_millis()
                        .clone();

                    let pair = self.selected_pair.to_string();
                    let interval = self.graph_props.interval.clone();
                    let mut limit = self.graph_props.limit.clone();
                    if self.graph_loading_state.is_last_page() {
                        limit = self.graph_loading_state.last_page_limit
                    }

                    self.klines_promise = Some(Promise::spawn_async(async move {
                        Client::kline(pair, interval, start, limit).await
                    }));
                }
            }
        }

        if let Some(promise) = &self.pairs_promise {
            if let Some(result) = promise.ready() {
                self.loading_pairs = false;

                self.pairs = result
                    .symbols
                    .iter()
                    .map(|s| -> GuiPair {
                        GuiPair {
                            symbol: s.symbol.to_string(),
                            active: s.status == "TRADING",
                        }
                    })
                    .collect();
            }
        }

        if self.debug_visible {
            Window::new("debug").show(ctx, |ui| {
                ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let mut text = "text";
                        TextEdit::multiline(&mut text).desired_rows(10).show(ui);
                    });
            });
        }

        self.render_top_panel(ctx);
        self.render_bottom_panel(ctx);
        self.render_side_panel(ctx);
        self.render_center_panel(ctx);
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        info!("called save")
    }
}

fn format_datetime(ts: i64) -> String {
    let naive = NaiveDateTime::from_timestamp(ts, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[tokio::main]
async fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "hedgegraph",
        native_options,
        Box::new(|cc| {
            let a = TemplateApp::new(cc);

            let subscriber = FmtSubscriber::builder()
                .with_max_level(Level::DEBUG)
                .finish();

            tracing::subscriber::set_global_default(subscriber)
                .expect("setting default subscriber failed");

            info!("tracing inited");
            Box::new(a)
        }),
    );
}
