use std::fs::File;
use std::path::Path;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use chrono::{Date, NaiveDateTime, Utc};
use crossbeam::channel::{unbounded, Receiver, Sender};
use egui::{
    plot::LinkedAxisGroup, CentralPanel, ProgressBar, Response, TopBottomPanel, Ui, Widget,
};
use egui_extras::{Size, StripBuilder};
use poll_promise::Promise;
use tracing::{debug, error, info};

use crate::netstrat::loading_state::LoadingState;
use crate::{
    netstrat::{
        bounds::{Bounds, BoundsSet},
        data::Data,
        props::Props,
        state::State,
    },
    sources::binance::{errors::ClientError, Client, Kline},
    windows::{AppWindow, TimeRangeChooser},
};

use super::{candles::Candles, volume::Volume};

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
    klines_sub: Receiver<Vec<Kline>>,
    klines_pub: Sender<Vec<Kline>>,
    symbol_sub: Receiver<String>,
    props_sub: Receiver<Props>,
    props_pub: Sender<Props>,
    export_sub: Receiver<Props>,
    drag_sub: Receiver<Bounds>,
}

impl Default for Graph {
    fn default() -> Self {
        let (s_symbols, r_symbols) = unbounded();
        let (s_props, r_props) = unbounded();
        let (s_props1, r_props1) = unbounded();
        let (s_export, r_export) = unbounded();
        let (s_klines, r_klines) = unbounded();
        let (_, r_bounds) = unbounded();

        Self {
            symbol_pub: s_symbols,
            time_range_window: Box::new(TimeRangeChooser::new(
                false,
                r_symbols.clone(),
                s_props,
                r_props1,
                s_export,
                Props::default(),
            )),

            symbol_sub: r_symbols,
            props_sub: r_props,
            props_pub: s_props1,
            export_sub: r_export,
            drag_sub: r_bounds,
            klines_sub: r_klines,
            klines_pub: s_klines,

            symbol: Default::default(),
            candles: Default::default(),
            volume: Default::default(),

            klines: Default::default(),
            state: Default::default(),
            export_state: Default::default(),
        }
    }
}

impl Graph {
    pub fn new(symbol_chan: Receiver<String>) -> Self {
        let (s_symbols, r_symbols) = unbounded();
        let (s_props, r_props) = unbounded();
        let (s_props1, r_props1) = unbounded();
        let (s_export, r_export) = unbounded();
        let (s_bounds, r_bounds) = unbounded();

        let axes_group = LinkedAxisGroup::new(true, false);

        Self {
            symbol_sub: symbol_chan,
            symbol_pub: s_symbols,
            props_sub: r_props,
            props_pub: s_props1,
            export_sub: r_export,
            drag_sub: r_bounds,
            time_range_window: Box::new(TimeRangeChooser::new(
                false,
                r_symbols,
                s_props,
                r_props1,
                s_export,
                Props::default(),
            )),
            candles: Candles::new(axes_group.clone(), s_bounds),
            volume: Volume::new(axes_group),
            ..Default::default()
        }
    }

    fn draw(&mut self, ui: &Ui) {
        debug!("drawing data...");
        let data = Data::new(self.klines.clone());
        self.volume.set_data(data.clone());
        self.candles.set_data(data);
        ui.ctx().request_repaint();
    }

    fn start_download(&mut self, props: Props, reset_state: bool) {
        if reset_state {
            self.klines = vec![];
            self.state = State::default();
        }

        self.state.apply_props(&props);

        if self.state.loading.pages() == 0 {
            info!("data already downloaded, skipping download");
            return;
        }

        info!("starting data download...");

        self.perform_data_request();
    }

    fn perform_data_request(&mut self) {
        // parallel data loading
        loop {
            match self.state.loading.get_next_page() {
                Some(_) => {
                    let start_time = self.state.loading.left_edge();
                    let interval = self.state.props.interval;
                    let limit = self.state.loading.page_size();            
                    let symbol = self.symbol.to_string();
                    let p = Mutex::new(Promise::spawn_async(async move {
                        Client::kline(symbol, interval, start_time, limit).await
                    }));

                    let sender = Mutex::new(self.klines_pub.clone());
                    thread::spawn(move || loop {
                        if let Some(data) = p.lock().unwrap().ready() {
                            match data {
                                Ok(payload) => {
                                    let res = sender.lock().unwrap().send(payload.clone());
                                    if let Err(err) = res {
                                        error!("failed to send klines to channel: {err}");
                                    };
                                }
                                Err(err) => {
                                    error!("got klines result with error: {err}")
                                }
                            }
                            break;
                        };
                    });
                }
                None => break,
            }
        }
    }

    fn handle_events(&mut self) {
        let drag_wrapped = self.drag_sub.recv_timeout(Duration::from_millis(1));

        if let Ok(bounds) = drag_wrapped {
            info!("got bounds: {bounds:?}");

            let mut props = self.state.props.clone();

            let dt_left = NaiveDateTime::from_timestamp((bounds.0 as f64 / 1000.0) as i64, 0);
            props.bounds = BoundsSet::new(vec![bounds]);
            props.date_start = Date::from_utc(dt_left.date(), Utc);
            props.time_start = dt_left.time();

            let dt_right = NaiveDateTime::from_timestamp((bounds.1 as f64 / 1000.0) as i64, 0);
            props.bounds = BoundsSet::new(vec![bounds]);
            props.date_end = Date::from_utc(dt_right.date(), Utc);
            props.time_end = dt_right.time();

            let send_result = self.props_pub.send(props.clone());
            match send_result {
                Ok(_) => {
                    info!("sent props: {props:?}");
                }
                Err(err) => {
                    error!("failed to send props: {err}");
                }
            }

            self.start_download(props, false);
        }

        let export_wrapped = self.export_sub.recv_timeout(Duration::from_millis(1));
        if let Ok(props) = export_wrapped {
            info!("got props for export: {props:?}");

            self.export_state.triggered = true;

            self.start_download(props, true);
        }

        let symbol_wrapped = self.symbol_sub.recv_timeout(Duration::from_millis(1));
        if let Ok(symbol) = symbol_wrapped {
            info!("got symbol: {symbol}");

            self.symbol = symbol.clone();
            self.symbol_pub.send(symbol).unwrap();

            self.start_download(Props::default(), true);
        }

        let show_wrapped = self.props_sub.recv_timeout(Duration::from_millis(1));
        if let Ok(props) = show_wrapped {
            info!("got show button pressed: {props:?}");

            self.start_download(props, true);
        }
    }
}

impl Widget for &mut Graph {
    fn ui(self, ui: &mut Ui) -> Response {
        self.handle_events();

        if self.symbol.is_empty() {
            return ui.label("Select a symbol");
        }

        let mut got = 0;
        let klines_wrapped = self.klines_sub.recv_timeout(Duration::from_millis(1));
        if let Ok(klines) = klines_wrapped {
            info!("received klines");
            klines.iter().for_each(|k| {
                self.klines.push(*k);
            });
            got += 1;
        }

        if got > 0 {
            self.state.loading.inc_loaded_pages(got);
            self.draw(ui);
        }

        self.candles
            .set_enabled(self.state.loading.progress() == 1.0);
        self.volume
            .set_enabled(self.state.loading.progress() == 1.0);

        if self.state.loading.progress() == 1.0 && self.export_state.triggered {
            info!("exporting data...");

            let name = format!(
                "{}_{}_{}_{:?}.csv",
                self.symbol,
                self.state.props.start_time().timestamp(),
                self.state.props.end_time().timestamp(),
                self.state.props.interval,
            );

            let path = Path::new(&name);
            let f_res = File::create(&path);
            match f_res {
                Ok(f) => {
                    let abs_path = path.canonicalize().unwrap();
                    info!("Saving to file: {}", abs_path.display());

                    let mut wtr = csv::Writer::from_writer(f);
                    self.klines.iter().for_each(|el| {
                        wtr.serialize(el).unwrap();
                    });
                    if let Some(err) = wtr.flush().err() {
                        error!("failed to write to file with error: {err}");
                    } else {
                        info!("exported to file: {abs_path:?}");
                        self.export_state.triggered = false;
                    }
                }
                Err(err) => {
                    error!("failed to create file with error: {err}");
                    self.export_state.triggered = false;
                }
            }
        }

        TopBottomPanel::top("graph toolbar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                self.time_range_window.toggle_btn(ui);
                if self.state.loading.progress() < 1.0 && !self.state.loading.has_error {
                    ui.add(
                        ProgressBar::new(self.state.loading.progress())
                            .show_percentage()
                            .animate(true),
                    );
                }
            });
        });

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
