use std::fs::File;

use chrono::{Date, NaiveDateTime, NaiveTime, Utc};
use crossbeam::channel::{unbounded, Receiver, Sender};

use egui::{
    plot::LinkedAxisGroup, CentralPanel, ProgressBar, Response, TopBottomPanel, Ui, Widget,
};
use egui_extras::{Size, StripBuilder};
use poll_promise::Promise;
use tracing::{debug, error, info, trace};

use crate::{
    netstrat::{
        bounds::Bounds,
        graph::{props::Props, state::State},
    },
    sources::binance::{Client, Kline},
    windows::{AppWindow, TimeRangeChooser},
};

use super::{candles::Candles, data::Data, volume::Volume};

#[derive(Default)]
struct ExportState {
    triggered: bool,
}

pub struct Graph {
    candles: Candles,
    volume: Volume,
    symbol: String,
    symbol_pub: Sender<String>,

    pub time_range_window: Box<dyn AppWindow>,

    klines: Vec<Kline>,
    state: State,
    export_state: ExportState,
    klines_promise: Option<Promise<Vec<Kline>>>,
    symbol_sub: Receiver<String>,
    show_sub: Receiver<Props>,
    export_sub: Receiver<Props>,
    bounds_sub: Receiver<Bounds>,
}

impl Default for Graph {
    fn default() -> Self {
        let (s_symbols, r_symbols) = unbounded();
        let (s_props, r_props) = unbounded();
        let (s_export, r_export) = unbounded();
        let (_, r_bounds) = unbounded();

        Self {
            symbol_pub: s_symbols,
            time_range_window: Box::new(TimeRangeChooser::new(
                false,
                r_symbols.clone(),
                s_props,
                s_export,
                Props::default(),
            )),

            symbol_sub: r_symbols,
            show_sub: r_props,
            export_sub: r_export,
            bounds_sub: r_bounds,

            symbol: Default::default(),
            candles: Default::default(),
            volume: Default::default(),

            klines: Default::default(),
            state: Default::default(),
            klines_promise: Default::default(),
            export_state: Default::default(),
        }
    }
}

impl Graph {
    pub fn new(symbol_chan: Receiver<String>) -> Self {
        let (s_symbols, r_symbols) = unbounded();
        let (s_props, r_props) = unbounded();
        let (s_export, r_export) = unbounded();
        let (s_bounds, r_bounds) = unbounded();

        let axes_group = LinkedAxisGroup::new(true, false);

        Self {
            symbol_sub: symbol_chan,
            symbol_pub: s_symbols,
            show_sub: r_props,
            export_sub: r_export,
            bounds_sub: r_bounds,
            time_range_window: Box::new(TimeRangeChooser::new(
                false,
                r_symbols,
                s_props,
                s_export,
                Props::default(),
            )),
            candles: Candles::new(axes_group.clone(), s_bounds),
            volume: Volume::new(axes_group),
            ..Default::default()
        }
    }

    fn start_download(&mut self, props: Props, export: bool) {
        self.export_state.triggered = export;

        self.state.apply_props(&props);

        if self.state.loading.pages.len() == 0 {
            info!("Data already downloaded, skipping download.");
            return;
        }

        info!("Starting data download...");

        self.klines = vec![];

        let start_time = props.start_time().timestamp_millis().clone();
        let symbol = self.symbol.to_string();
        let interval = props.interval.clone();
        let limit = self.state.loading.pages.page_size();

        debug!("setting left edge to: {start_time}");

        self.klines_promise = Some(Promise::spawn_async(async move {
            Client::kline(symbol, interval, start_time, limit).await
        }));
    }
}

impl Widget for &mut Graph {
    fn ui(self, ui: &mut Ui) -> Response {
        let bounds_wrapped = self
            .bounds_sub
            .recv_timeout(std::time::Duration::from_millis(1));

        match bounds_wrapped {
            Ok(bounds) => {
                info!("got bounds: {bounds:?}");

                if self.klines.len() > 0 && self.klines[0].t_close > bounds.0 as i64 {
                    info!("uploading new data");

                    let dt = NaiveDateTime::from_timestamp((bounds.0 as f64 / 1000.0) as i64, 0);
                    let mut props = self.state.props.clone();
                    props.date_start = Date::from_utc(dt.date(), Utc);
                    props.time_start = dt.time();
                    self.start_download(props, false);
                }
            }
            Err(_) => {}
        }

        let export_wrapped = self
            .export_sub
            .recv_timeout(std::time::Duration::from_millis(1));

        match export_wrapped {
            Ok(props) => {
                info!("got props for export: {props:?}");

                self.start_download(props, true);
            }
            Err(_) => {}
        }

        let symbol_wrapped = self
            .symbol_sub
            .recv_timeout(std::time::Duration::from_millis(1));

        match symbol_wrapped {
            Ok(symbol) => {
                info!("got symbol: {symbol}");

                self.klines = vec![];
                self.symbol = symbol.clone();
                self.symbol_pub.send(symbol).unwrap();

                self.state = State::default();
                let start_time = self.state.props.start_time().timestamp_millis().clone();
                let interval = self.state.props.interval.clone();
                let limit = self.state.loading.pages.page_size();
                let symbol = self.symbol.clone();
                self.klines_promise = Some(Promise::spawn_async(async move {
                    Client::kline(symbol, interval, start_time, limit).await
                }));
            }
            Err(_) => {}
        }

        if self.symbol == "" {
            return ui.label("select a symbol");
        }

        let show_wrapped = self
            .show_sub
            .recv_timeout(std::time::Duration::from_millis(1));

        match show_wrapped {
            Ok(props) => {
                info!("got props: {props:?}");

                self.start_download(props, false);
            }
            Err(_) => {}
        }

        if let Some(promise) = &self.klines_promise {
            if let Some(result) = promise.ready() {
                result.iter().for_each(|k| {
                    self.klines.push(k.clone());
                });

                self.klines_promise = None;
                if let Some(page) = self.state.loading.turn_page() {
                    let start = self.state.loading.start_time;
                    let symbol = self.symbol.clone();
                    let interval = self.state.props.interval.clone();
                    let limit = self.state.loading.pages.page_size();

                    self.klines_promise = Some(Promise::spawn_async(async move {
                        Client::kline(symbol, interval, start, limit).await
                    }));
                } else {
                    let data = Data::new(self.klines.clone());
                    self.volume.set_data(data.clone());
                    self.candles.set_data(data);
                }
            }
        }

        if self.state.loading.progress() < 1.0 {
            return ui
                .centered_and_justified(|ui| {
                    ui.add(
                        ProgressBar::new(self.state.loading.progress())
                            .show_percentage()
                            .animate(true),
                    )
                })
                .response;
        }

        if self.state.loading.progress() == 1.0 && self.export_state.triggered {
            info!("Exporting data...");

            let name = format!(
                "{}-{}-{}-{:?}",
                self.symbol,
                self.state.props.start_time(),
                self.state.props.end_time(),
                self.state.props.interval,
            );
            let f = File::create(format!("{}.csv", name)).unwrap();

            let mut wtr = csv::Writer::from_writer(f);
            self.klines.iter().for_each(|el| {
                wtr.serialize(el).unwrap();
            });
            wtr.flush().unwrap();

            self.export_state.triggered = false;

            info!("Exported to file: {}.csv", name);
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
                            ui.add(&mut self.candles);
                        });
                        strip.cell(|ui| {
                            ui.add(&self.volume);
                        });
                    })
            })
            .response
    }
}
