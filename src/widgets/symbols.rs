use egui::{Label, Layout, Response, ScrollArea, TextEdit, Widget, WidgetText};
use poll_promise::Promise;

use crate::sources::binance::client::{Client, Info, Symbol};

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
    select_callback: Box<FnMut(String)>,
}

impl Symbols {
    pub fn new(select_callback: dyn Fn(String)) -> Self {
        Self {
            loading: true,
            symbols_promise: Some(Promise::spawn_async(async { Client::info().await })),
            select_callback,
            ..Default::default()
        }
    }
}

impl Default for Symbols {
    fn default() -> Self {
        Self {
            symbols: Default::default(),
            filter: Default::default(),
            loading: Default::default(),
            selected_symbol: Default::default(),
            symbols_promise: Default::default(),
            select_callback: &|_: String| {},
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
            ui.add_space(5f32);
            ui.separator();
            ui.add_space(5f32);

            // render filter
            ui.add(
                TextEdit::singleline(&mut self.filter.value)
                    .hint_text(WidgetText::from("filter symbols").italics()),
            );

            // aply filter
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

            // render symbols list
            ScrollArea::vertical()
                .auto_shrink([false, false])
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

                            // if label.clicked() {
                            //     self.candle_plot.plot(s.symbol.clone())
                            // };
                        });
                    })
                });
        })
        .response
    }
}
