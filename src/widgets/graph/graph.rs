use std::fs::File;

use chrono::Timelike;
use crossbeam::channel::{unbounded, Receiver, Sender};

use egui::{
    panel::TopBottomSide, plot::LinkedAxisGroup, CentralPanel, ProgressBar, Response,
    TopBottomPanel, Ui, Widget, Window,
};
use egui_extras::{Size, StripBuilder};
use poll_promise::Promise;

use crate::{
    sources::binance::{Client, Interval, Kline},
    windows::{AppWindow, TimeRangeChooser},
};

use super::{
    candles::Candles, data::Data, loading_state::LoadingState, props::Props, time_input::TimeInput,
    volume::Volume,
};

pub struct Graph {
    candles: Candles,
    volume: Volume,
    symbol: String,
    symbol_pub: Sender<String>,
    time_start: TimeInput,
    time_end: TimeInput,

    time_range_window: Box<dyn AppWindow>,

    klines: Vec<Kline>,
    graph_props: Props,
    graph_loading_state: LoadingState,
    klines_promise: Option<Promise<Vec<Kline>>>,
    symbol_chan: Receiver<String>,
    valid: bool,
}

impl Default for Graph {
    fn default() -> Self {
        let graph_props = Props::default();
        let start_time = graph_props.time_start;
        let end_time = graph_props.time_end;
        let (s, r) = unbounded();

        Self {
            candles: Default::default(),
            volume: Default::default(),
            symbol: Default::default(),
            symbol_pub: s,
            time_start: TimeInput::new(start_time.hour(), start_time.minute(), start_time.second()),
            time_end: TimeInput::new(end_time.hour(), end_time.minute(), end_time.second()),

            time_range_window: Box::new(TimeRangeChooser::new(false, r.clone())),

            klines: Default::default(),
            graph_props,
            graph_loading_state: Default::default(),
            klines_promise: Default::default(),
            symbol_chan: r,
            valid: true,
        }
    }
}

impl Graph {
    pub fn new(symbol_chan: Receiver<String>) -> Self {
        let (s, r) = unbounded();
        Self {
            symbol_chan: symbol_chan,
            symbol_pub: s,
            time_range_window: Box::new(TimeRangeChooser::new(false, r)),
            ..Default::default()
        }
    }
}

impl Widget for &mut Graph {
    fn ui(self, ui: &mut Ui) -> Response {
        let symbol_wrapped = self
            .symbol_chan
            .recv_timeout(std::time::Duration::from_millis(10));

        match symbol_wrapped {
            Ok(symbol) => {
                self.symbol = symbol.clone();
                self.symbol_pub.send(symbol).unwrap();
                self.graph_loading_state = Default::default();
            }
            Err(_) => {}
        }

        // TODO: think of placeholder
        if self.symbol == "" {
            return ui.label("select symbol");
        }

        if let Some(promise) = &self.klines_promise {
            if let Some(result) = promise.ready() {
                if self.graph_loading_state.received == 0 {
                    self.klines = vec![];
                }

                self.graph_loading_state.inc_received();

                if self.graph_loading_state.received > 0 {
                    result.iter().for_each(|k| {
                        self.klines.push(*k);
                    });
                }

                self.klines_promise = None;

                match self.graph_loading_state.is_finished() {
                    true => {
                        let data = Data::new(self.klines.clone());
                        let axes_group = LinkedAxisGroup::new(true, false);
                        self.volume = Volume::new(data.clone(), axes_group.clone());
                        self.candles = Candles::new(data, axes_group);
                    }
                    false => {
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
            }
        } else if !self.graph_loading_state.triggered {
            self.graph_loading_state = LoadingState::from_graph_props(&self.graph_props);
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

        TopBottomPanel::top("graph toolbar")
            .show_inside(ui, |ui| self.time_range_window.toggle_btn(ui));

        CentralPanel::default()
            .show_inside(ui, |ui| {
                self.time_range_window.show(ui);

                StripBuilder::new(ui)
                    .size(Size::relative(0.8))
                    .size(Size::remainder())
                    .vertical(|mut strip| {
                        strip.cell(|ui| {
                            ui.add(&self.candles);
                        });
                        strip.cell(|ui| {
                            ui.add(&self.volume);
                        });
                    })
            })
            .response
    }
}
