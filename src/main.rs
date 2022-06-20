use std::cmp::Ordering;

use poll_promise::Promise;

use eframe;
use sources::binance::client::{Kline, Symbol};

use chrono::{prelude::*, Duration};
use egui::plot::{BoxElem, BoxPlot, BoxSpread, Plot, PlotUi};
use egui::widgets::Label;
use egui::{
    CentralPanel, Color32, Layout, ProgressBar, Response, ScrollArea, SidePanel, Stroke, TextEdit,
    TopBottomPanel, Ui, Visuals, WidgetText, Window,
};
use sources::binance::interval::Interval;
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;
use widgets::candle_plot::{self, CandlePlot};

use crate::sources::binance::client::{Client, Info};

mod network;
mod sources;
mod widgets;
use tokio;

#[derive(Default)]
struct TemplateApp {
    graph: CandlePlot,
    filter: FilterProps,
    candle_plot: CandlePlot,
    pairs: Vec<Symbol>,
    pairs_loaded: bool,
    loading_pairs: bool,
    selected_pair: String,
    pairs_promise: Option<Promise<Info>>,
    debug_visible: bool,
    dark_mode: bool,
}

#[derive(Default)]
struct FilterProps {
    value: String,
    active_only: bool,
}

impl TemplateApp {
    fn new(_ctx: &eframe::CreationContext<'_>) -> Self {
        let pairs = vec![];

        Self {
            pairs,
            dark_mode: true,
            candle_plot: CandlePlot::new(),
            ..Default::default()
        }
    }

    fn render_center_panel(&mut self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| ui.add(&mut self.candle_plot));
    }

    fn render_top_panel(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add(&mut self.graph);
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
                let filtered: Vec<&Symbol> = self
                    .pairs
                    .iter()
                    .filter(|s| {
                        let match_value = s
                            .symbol
                            .to_lowercase()
                            .contains(self.filter.value.to_lowercase().as_str());
                        if self.filter.active_only {
                            return match_value && s.active();
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
                                    match s.active() {
                                        true => WidgetText::from(s.symbol.to_string()).strong(),
                                        false => {
                                            WidgetText::from(s.symbol.to_string()).strikethrough()
                                        }
                                    },
                                );

                                if label.clicked() {
                                    self.candle_plot.plot(s.symbol.clone())
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

        if let Some(promise) = &self.pairs_promise {
            if let Some(result) = promise.ready() {
                self.loading_pairs = false;

                self.pairs = result
                    .symbols
                    .iter()
                    .map(|s| -> Symbol { s.clone() })
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
