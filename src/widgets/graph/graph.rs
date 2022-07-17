use std::fs::File;

use crossbeam::channel::{unbounded, Receiver, Sender};

use egui::{
    plot::LinkedAxisGroup, CentralPanel, ProgressBar, Response, TopBottomPanel, Ui, Widget,
};
use egui_extras::{Size, StripBuilder};
use poll_promise::Promise;
use tracing::{error, info};

use crate::{
    sources::binance::{Client, Kline},
    windows::{AppWindow, TimeRangeChooser},
};

use super::{
    candles::Candles, data::Data, loading_state::LoadingState, props::Props, volume::Volume,
};

pub struct Graph {
    candles: Candles,
    volume: Volume,
    symbol: String,
    symbol_pub: Sender<String>,

    pub time_range_window: Box<dyn AppWindow>,

    klines: Vec<Kline>,
    graph_props: Props,
    graph_loading_state: LoadingState,
    klines_promise: Option<Promise<Vec<Kline>>>,
    symbol_sub: Receiver<String>,
    props_sub: Receiver<Props>,
    export_sub: Receiver<()>,
}

impl Default for Graph {
    fn default() -> Self {
        let graph_props = Props::default();
        let (s_symbols, r_symbols) = unbounded();
        let (s_props, r_props) = unbounded();
        let (s_export, r_export) = unbounded();

        Self {
            candles: Default::default(),
            volume: Default::default(),
            symbol: Default::default(),
            symbol_pub: s_symbols,

            time_range_window: Box::new(TimeRangeChooser::new(
                false,
                r_symbols.clone(),
                s_props,
                s_export,
            )),

            klines: Default::default(),
            graph_props,
            graph_loading_state: Default::default(),
            klines_promise: Default::default(),
            symbol_sub: r_symbols,
            props_sub: r_props,
            export_sub: r_export,
        }
    }
}

impl Graph {
    pub fn new(symbol_chan: Receiver<String>) -> Self {
        let (s_symbols, r_symbols) = unbounded();
        let (s_props, r_props) = unbounded();
        let (s_export, r_export) = unbounded();

        Self {
            symbol_sub: symbol_chan,
            symbol_pub: s_symbols,
            props_sub: r_props,
            export_sub: r_export,
            time_range_window: Box::new(TimeRangeChooser::new(false, r_symbols, s_props, s_export)),
            ..Default::default()
        }
    }
}

impl Widget for &mut Graph {
    fn ui(self, ui: &mut Ui) -> Response {
        let export_wrapped = self
            .export_sub
            .recv_timeout(std::time::Duration::from_millis(1));

        match export_wrapped {
            Ok(_) => {
                info!("got export command");
                let name = format!(
                    "{}-{}-{}-{:?}",
                    self.symbol,
                    self.graph_props.start_time(),
                    self.graph_props.end_time(),
                    self.graph_props.interval,
                );
                let f = File::create(format!("{}.csv", name)).unwrap();

                let mut wtr = csv::Writer::from_writer(f);
                self.klines.iter().for_each(|el| {
                    wtr.serialize(el).unwrap();
                });
                wtr.flush().unwrap();
            }
            Err(_) => {}
        }

        let symbol_wrapped = self
            .symbol_sub
            .recv_timeout(std::time::Duration::from_millis(1));

        match symbol_wrapped {
            Ok(symbol) => {
                info!("got symbol: {symbol}");

                self.symbol = symbol.clone();
                self.symbol_pub.send(symbol).unwrap();
                self.graph_loading_state = Default::default();
            }
            Err(_) => {}
        }

        let props_wrapped = self
            .props_sub
            .recv_timeout(std::time::Duration::from_millis(1));

        match props_wrapped {
            Ok(props) => {
                info!("got props: {props:?}");
                self.graph_props = props;
                self.graph_loading_state = LoadingState::from_graph_props(&self.graph_props);
                self.graph_loading_state.triggered = true;

                let start = self.graph_props.start_time().timestamp_millis().clone();
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
            Err(_) => {}
        }

        if self.symbol == "" {
            return ui.label("select a symbol");
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
