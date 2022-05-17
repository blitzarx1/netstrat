use eframe;
use egui::TextBuffer;
use serde_json::to_string;
use tracing_subscriber;

use crate::sources::binance::client::Client;

// use poll_promise::Promise;

mod network;
mod sources;

#[derive(Default)]
pub struct TemplateApp {
    filter_value: String,
    pairs: Vec<String>,
    connected: bool,
    loading: bool,
    selected_pair: String,
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_ctx: &eframe::CreationContext<'_>) -> Self {
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
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // TODO: theme change placeholder
            ui.heading("TODO: theme change placeholder");
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            if self.loading {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                });
                return;
            }
            if !self.connected {
                if ui.button("connect to binance".to_string()).clicked() {
                    // todo make async as in https://github.com/emilk/egui/blob/master/egui_demo_app/src/apps/http_app.rs
                    Client::info_blocking()
                        .unwrap()
                        .symbols
                        .iter()
                        .for_each(|sym| {
                            self.pairs.push(sym.symbol.to_string());
                        });

                    self.connected = !self.connected;
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
                ui.add(
                    egui::widgets::TextEdit::singleline(&mut self.filter_value)
                        .hint_text(egui::WidgetText::from("filter pairs").italics()),
                );
                let filtered: Vec<&String> = self
                    .pairs
                    .iter()
                    .filter(|s| {
                        s.to_lowercase()
                            .contains(self.filter_value.to_lowercase().as_str())
                    })
                    .collect();
                ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                    ui.add(egui::widgets::Label::new(
                        egui::WidgetText::from(format!("{}/{}", filtered.len(), self.pairs.len()))
                            .small(),
                    ));
                });
                ui.add_space(5f32);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        filtered.iter().for_each(|s| {
                            if ui
                                .selectable_label(
                                    false,
                                    egui::WidgetText::from(s.to_string()).strong(),
                                )
                                .clicked()
                            {
                                self.selected_pair = s.to_string();
                            };
                        });
                    })
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // TODO: theme change placeholder
            ui.heading(format!(
                "TODO: show graph for: {}",
                self.selected_pair.to_string()
            ));
        });
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        println!("called save")
    }
}

fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Box::new(TemplateApp::new(cc))),
    );
}
