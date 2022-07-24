use crossbeam::channel::{unbounded, Sender};
use egui::{Label, Layout, Response, ScrollArea, TextEdit, Widget, WidgetText};
use poll_promise::Promise;
use tracing::{error, info};

use crate::sources::binance::{Client, Info, Symbol};

#[derive(Default)]
struct FilterProps {
    value: String,
    active_only: bool,
}

pub struct Symbols {
    symbols: Vec<Symbol>,
    filter: FilterProps,
    loading: bool,
    selected_symbol: String,
    symbols_promise: Option<Promise<Info>>,
    symbol_pub: Sender<String>,
}

impl Default for Symbols {
    fn default() -> Self {
        let (s, _) = unbounded();
        Self {
            symbols: Default::default(),
            filter: Default::default(),
            loading: Default::default(),
            selected_symbol: Default::default(),
            symbols_promise: Default::default(),
            symbol_pub: s,
        }
    }
}

impl Symbols {
    pub fn new(symbol_pub: Sender<String>) -> Self {
        Self {
            loading: true,
            symbols_promise: Some(Promise::spawn_async(async { Client::info().await })),
            symbol_pub,
            ..Default::default()
        }
    }
}

impl Widget for &mut Symbols {
    fn ui(self, ui: &mut egui::Ui) -> Response {
        if let Some(promise) = &self.symbols_promise {
            if let Some(result) = promise.ready() {
                self.loading = false;

                self.symbols = result
                    .symbols
                    .iter()
                    .map(|s| -> Symbol { s.clone() })
                    .collect();
            }
        }

        if self.loading {
            return ui
                .centered_and_justified(|ui| {
                    ui.spinner();
                })
                .response;
        }

        ui.with_layout(Layout::top_down(egui::Align::LEFT), |ui| {
            ui.add(
                TextEdit::singleline(&mut self.filter.value)
                    .hint_text(WidgetText::from("filter symbols").italics()),
            );

            let filtered: Vec<&Symbol> = self
                .symbols
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
                    WidgetText::from(format!("{}/{}", filtered.len(), self.symbols.len())).small(),
                ));
            });

            ui.add_space(5f32);

            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    ui.with_layout(Layout::top_down(egui::Align::LEFT), |ui| {
                        filtered.iter().for_each(|s| {
                            let label = ui.selectable_label(
                                s.symbol == self.selected_symbol,
                                match s.active() {
                                    true => WidgetText::from(s.symbol.to_string()).strong(),
                                    false => WidgetText::from(s.symbol.to_string()).strikethrough(),
                                },
                            );

                            if label.clicked() {
                                let send_result = self.symbol_pub.send(s.symbol.clone());
                                match send_result {
                                    Ok(_) => {
                                        info!("Sent symbol: {}.", s.symbol);
                                    }
                                    Err(err) => {
                                        error!("Failed to send symbol: {err}.");
                                    }
                                }

                                self.selected_symbol = s.symbol.clone();
                            };
                        });
                    })
                });
        })
        .response
    }
}
