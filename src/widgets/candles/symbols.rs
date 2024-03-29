use crossbeam::channel::{unbounded, Sender};
use egui::{Layout, ScrollArea, TextEdit, WidgetText};
use poll_promise::Promise;
use tracing::{debug, error, info};

use crate::{
    netstrat::line_filter_highlight_layout,
    sources::binance::{Client, Info, Symbol},
    widgets::AppWidget,
};

#[derive(Default)]
struct FilterProps {
    value: String,
    active_only: bool,
}

pub struct Symbols {
    symbols: Vec<Symbol>,
    filter: FilterProps,
    filtered: Vec<Symbol>,
    loading: bool,
    selected_symbol: String,
    symbols_promise: Option<Promise<Info>>,
    symbol_pub: Sender<String>,
}

impl Default for Symbols {
    fn default() -> Self {
        let (s, _) = unbounded();
        let symbols_promise = Some(Promise::spawn_blocking(Client::info));
        let loading = true;
        Self {
            loading,
            symbols_promise,
            symbol_pub: s,
            symbols: Default::default(),
            filter: Default::default(),
            filtered: Default::default(),
            selected_symbol: Default::default(),
        }
    }
}

impl Symbols {
    pub fn new(symbol_pub: Sender<String>) -> Self {
        info!("initing widget symbols");
        Self {
            symbol_pub,
            ..Default::default()
        }
    }

    fn update(&mut self, filter_value: String, active_only: bool, selected_symbol: String) {
        self.apply_filter(filter_value, active_only);

        if self.selected_symbol != selected_symbol {
            info!("setting symbol to {selected_symbol}");
            self.selected_symbol = selected_symbol;
        }
    }

    fn apply_filter(&mut self, filter_value: String, active_only: bool) {
        let filter_normalized = filter_value.to_lowercase();
        if filter_normalized == self.filter.value && active_only == self.filter.active_only {
            return;
        }

        info!("applying filter: {filter_value}; active_only: {active_only}");

        if filter_normalized != self.filter.value
            && filter_normalized.contains(self.filter.value.as_str())
            && self.filter.active_only == active_only
        {
            debug!("using optimized version");

            self.filtered.retain(|el| {
                el.symbol
                    .to_lowercase()
                    .contains(filter_normalized.as_str())
            });
        } else {
            debug!("using heavy version");

            self.filtered = self
                .symbols
                .iter()
                .filter(|el| {
                    el.symbol
                        .to_lowercase()
                        .contains(filter_normalized.as_str())
                })
                .cloned()
                .collect();
        }

        if active_only != self.filter.active_only && active_only {
            self.filtered.retain(|el| el.active() == active_only);
        }

        self.filter.value = filter_normalized;
        self.filter.active_only = active_only;
    }
}

impl AppWidget for Symbols {
    fn show(&mut self, ui: &mut egui::Ui) {
        let mut filter_value = self.filter.value.clone();
        let mut active_only = self.filter.active_only;
        let mut selected_symbol = self.selected_symbol.clone();

        if let Some(promise) = &self.symbols_promise {
            if let Some(result) = promise.ready() {
                self.loading = false;

                self.symbols = result.symbols.to_vec();
                self.filtered = result.symbols.to_vec();

                self.symbols_promise = None;
            }
        }

        if self.loading {
            ui.centered_and_justified(|ui| {
                ui.spinner();
            });
            return;
        }

        ui.with_layout(Layout::top_down(egui::Align::LEFT), |ui| {
            ui.add(
                TextEdit::singleline(&mut filter_value)
                    .hint_text(WidgetText::from("filter symbols").italics()),
            );

            ui.with_layout(Layout::top_down(egui::Align::RIGHT), |ui| {
                ui.checkbox(&mut active_only, "active only");
                ui.label(
                    WidgetText::from(format!("{}/{}", self.filtered.len(), self.symbols.len()))
                        .small(),
                );
            });

            ui.add_space(5f32);

            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    ui.with_layout(Layout::top_down(egui::Align::LEFT), |ui| {
                        self.filtered.iter().for_each(|s| {
                            let label = ui.selectable_label(
                                s.symbol == selected_symbol,
                                WidgetText::from(line_filter_highlight_layout(
                                    ui,
                                    &s.symbol,
                                    &self.filter.value,
                                    !s.active(),
                                )),
                            );

                            if label.clicked() {
                                let send_result = self.symbol_pub.send(s.symbol.clone());
                                match send_result {
                                    Ok(_) => {
                                        debug!("sent symbol: {}", s.symbol);
                                    }
                                    Err(err) => {
                                        error!("failed to send symbol: {err}");
                                    }
                                }

                                selected_symbol = s.symbol.clone();
                            };
                        });
                    })
                });
        });

        self.update(filter_value, active_only, selected_symbol);
    }
}
