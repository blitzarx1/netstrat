use std::{fmt::format, thread::sleep, time::Duration};

use egui::{style::Widgets, widgets};
use poll_promise::Promise;

use eframe;
use sources::binance::client::Kline;

use crate::sources::binance::client::{Client, Info};

mod network;
mod sources;
use tokio;

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

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // TODO: theme change placeholder
            ui.heading("TODO: theme change placeholder");
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
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

            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                if ui.button("back").clicked() {
                    self.connected = !self.connected;
                }
                ui.add_space(5f32);
                ui.separator();
                ui.add_space(5f32);

                // render filter
                ui.add(
                    egui::widgets::TextEdit::singleline(&mut self.filter.value)
                        .hint_text(egui::WidgetText::from("filter pairs").italics()),
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
                ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                    ui.checkbox(&mut self.filter.active_only, "active only");
                    ui.add(egui::widgets::Label::new(
                        egui::WidgetText::from(format!("{}/{}", filtered.len(), self.pairs.len()))
                            .small(),
                    ));
                });

                ui.add_space(5f32);

                // render pairs list
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            filtered.iter().for_each(|s| {
                                let symbol_for_klines_request = s.symbol.to_string();
                                if ui
                                    .selectable_label(
                                        false,
                                        egui::WidgetText::from(s.symbol.to_string())
                                            .background_color({
                                                match s.active {
                                                    true => egui::Color32::LIGHT_GREEN,
                                                    false => egui::Color32::LIGHT_RED,
                                                }
                                            })
                                            .strong(),
                                    )
                                    .clicked()
                                {
                                    self.loading_klines = true;
                                    self.klines_promise = Some(Promise::spawn_async(async {
                                        Client::kline(
                                            symbol_for_klines_request,
                                            sources::binance::interval::Interval::Minute,
                                            1651995344000,
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

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.loading_klines {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                });
            }

            if !self.loading_klines && self.klines.len() > 0 {
                let line = egui::plot::Line::new(egui::plot::Values::from_values_iter(
                    self.klines.iter().map(|k| -> egui::plot::Value {
                        egui::plot::Value::new(k.ts as f64, k.close)
                    }),
                ));

                egui::plot::Plot::new("plot").show(ui, |ui| ui.line(line));
            }
        });
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        println!("called save")
    }
}

#[tokio::main]
async fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Box::new(TemplateApp::new(cc))),
    );
}
