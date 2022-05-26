use std::cmp::Ordering;
use std::ops::Add;

use poll_promise::Promise;

use eframe;
use sources::binance::client::Kline;

use chrono::{prelude::*, Duration};
use egui::plot::{Line, Plot, Value, Values};
use egui::widgets::Label;
use egui::{
    Align, CentralPanel, CollapsingHeader, Layout, ScrollArea, SidePanel, TextEdit, TopBottomPanel,
    Ui, WidgetText, Window,
};
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::sources::binance::client::{Client, Info};

mod network;
mod sources;
use tokio;

#[derive(Debug)]
struct GraphProps {
    start: chrono::Date<Utc>,
    end: chrono::Date<Utc>,
}

impl Default for GraphProps {
    fn default() -> Self {
        Self {
            start: Date::from(Utc::now().date()) - Duration::days(1),
            end: Date::from(Utc::now().date()),
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
    connected: bool,
    loading_pairs: bool,
    loading_klines: bool,
    selected_pair: String,
    pairs_promise: Option<Promise<Info>>,
    klines_promise: Option<Promise<Vec<Kline>>>,
    debug_visible: bool,
    graph_props: GraphProps,
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
            ..Default::default()
        }
    }

    fn render_center_panel(&mut self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| {
            if self.loading_klines {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                });
            }

            if !self.loading_klines && self.klines.len() > 0 {
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
                let min_x_kline = &self.klines[0];

                Window::new(self.selected_pair.to_string()).show(ctx, |ui| {
                    ui.collapsing("time period", |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.add(Label::new(WidgetText::from("start")));
                            ui.add(
                                egui_extras::DatePickerButton::new(&mut self.graph_props.start)
                                    .id_source("datepicker_start"),
                            );
                        });
                        ui.horizontal_wrapped(|ui| {
                            ui.add(Label::new(WidgetText::from("end")));
                            ui.add(
                                egui_extras::DatePickerButton::new(&mut self.graph_props.end)
                                    .id_source("datepicker_end"),
                            );
                        });
                    });
                    ui.vertical_centered(|ui| {
                        if ui.button("apply").clicked() {
                            let ts = self
                                .graph_props
                                .start
                                .and_hms(0, 0, 0)
                                .timestamp_millis()
                                .clone();
                            let pair = self.selected_pair.to_string();
                            self.loading_klines = true;
                            self.klines_promise = Some(Promise::spawn_async(async move {
                                Client::kline(
                                    pair,
                                    sources::binance::interval::Interval::Minute,
                                    ts,
                                    1000,
                                )
                                .await
                            }));
                        }
                    });
                });

                let line = Line::new(Values::from_values_iter(
                    self.klines
                        .iter()
                        .map(|k| -> Value { Value::new(k.ts as f64, k.close) }),
                ));

                Plot::new("plot")
                    .label_formatter(|_s, v| {
                        format!(
                            "y: ${}\nx: {}",
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
            // TODO: theme change placeholder
            ui.heading("TODO: theme change placeholder");
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

            if !self.connected {
                if ui.button("connect to binance".to_string()).clicked() {
                    self.pairs_promise = Some(Promise::spawn_async(async { Client::info().await }));

                    self.connected = !self.connected;
                    self.loading_pairs = true;
                };
                return;
            }

            ui.with_layout(Layout::top_down(egui::Align::LEFT), |ui| {
                if ui.button("back").clicked() {
                    self.connected = !self.connected;
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
                                    false,
                                    WidgetText::from(s.symbol.to_string())
                                        .background_color({
                                            match s.active {
                                                true => egui::Color32::LIGHT_GREEN,
                                                false => egui::Color32::LIGHT_RED,
                                            }
                                        })
                                        .strong(),
                                );

                                let ts = self
                                    .graph_props
                                    .start
                                    .and_hms(0, 0, 0)
                                    .timestamp_millis()
                                    .clone();

                                if label.clicked() {
                                    label.scroll_to_me(Some(Align::Center));
                                    self.loading_klines = true;
                                    self.klines_promise = Some(Promise::spawn_async(async move {
                                        Client::kline(
                                            symbol_for_klines_request,
                                            sources::binance::interval::Interval::Minute,
                                            ts,
                                            1000,
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
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(promise) = &self.klines_promise {
            if let Some(result) = promise.ready() {
                self.loading_klines = false;

                self.klines = result
                    .iter()
                    .map(|k| -> GuiKline {
                        GuiKline {
                            close: k.close,
                            ts: k.t_close,
                        }
                    })
                    .collect();
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
                ui.label("TODO: console and debug stats here");
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

fn init_tracing() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

#[tokio::main]
async fn main() {
    init_tracing();

    info!("tracing inited");

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "hedgegraph",
        native_options,
        Box::new(|cc| Box::new(TemplateApp::new(cc))),
    );
}
