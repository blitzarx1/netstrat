use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{cmp::Ordering, fs::File};

use chrono::{Date, NaiveDateTime, Utc};
use crossbeam::channel::{unbounded, Receiver, Sender};
use egui::{CentralPanel, ProgressBar, TopBottomPanel, Ui};
use egui_notify::{Anchor, Toasts};
use poll_promise::Promise;
use tracing::{debug, error, info, trace};

use crate::netstrat::{Drawer, ThreadPool};
use crate::sources::binance::{Client, Kline};
use crate::widgets::candles::bounds::BoundsSet;
use crate::widgets::AppWidget;

use super::bounds::Bounds;
use super::canles_drawer::CandlesDrawer;
use super::state::State;
use super::time_range_settings::TimeRangeSettings;
use super::TimeRange;

#[derive(Default)]
struct ExportState {
    triggered: bool,
}

pub struct Props {
    time_range: Box<dyn AppWidget>,

    candles: CandlesDrawer,
    symbol: String,

    max_frame_pages: usize,
    data_changed: bool,
    state: State,
    export_state: ExportState,

    toasts: Toasts,

    pool: ThreadPool,

    klines_pub: Sender<Vec<Kline>>,
    klines_sub: Receiver<Vec<Kline>>,
    drawer_pub: Sender<Arc<Mutex<Box<dyn Drawer>>>>,
    symbol_pub: Sender<String>,
    symbol_sub: Receiver<String>,
    props_pub: Sender<TimeRangeSettings>,
    props_sub: Receiver<TimeRangeSettings>,
    export_sub: Receiver<TimeRangeSettings>,
    drag_sub: Receiver<Bounds>,
}

impl Default for Props {
    fn default() -> Self {
        let max_frame_pages = 50;
        let toasts = Toasts::default().with_anchor(Anchor::TopRight);

        let (s_symbols, r_symbols) = unbounded();
        let (s_props, r_props) = unbounded();
        let (s_props1, r_props1) = unbounded();
        let (s_export, r_export) = unbounded();
        let (s_klines, r_klines) = unbounded();
        let (s_bounds, r_bounds) = unbounded();
        let (s_drawer, _) = unbounded();

        let time_range_chooser = Box::new(TimeRange::new(
            r_symbols.clone(),
            s_props,
            r_props1,
            s_export,
            TimeRangeSettings::default(),
        ));

        let candles = CandlesDrawer::new(s_bounds);

        let pool = ThreadPool::new(100);

        Self {
            max_frame_pages,

            time_range: time_range_chooser,

            candles,

            pool,

            toasts,

            symbol_sub: r_symbols,
            symbol_pub: s_symbols,
            drawer_pub: s_drawer,
            props_sub: r_props,
            props_pub: s_props1,
            export_sub: r_export,
            drag_sub: r_bounds,
            klines_sub: r_klines,
            klines_pub: s_klines,

            data_changed: Default::default(),
            symbol: Default::default(),
            state: Default::default(),
            export_state: Default::default(),
        }
    }
}

impl Props {
    pub fn new(
        symbol_sub: Receiver<String>,
        drawer_pub: Sender<Arc<Mutex<Box<dyn Drawer>>>>,
    ) -> Self {
        info!("initing widget graph");
        Self {
            symbol_sub,
            drawer_pub,
            ..Default::default()
        }
    }

    fn update_data(&mut self, klines: &mut Vec<Kline>) {
        info!(
            "adding {} entries to volume and candles widgets",
            klines.len()
        );

        self.candles.add_data(klines);

        self.data_changed = true;
    }

    fn start_download(&mut self, props: TimeRangeSettings, reset_state: bool) {
        if reset_state {
            self.state = State::default();
        }

        self.state.apply_props(&props);

        if self.state.loading.pages() == 0 {
            debug!("data already downloaded, skipping download");
            return;
        }

        debug!(
            "data splitted in {} pages; starting download...",
            self.state.loading.pages()
        );

        self.perform_data_request();
    }

    fn perform_data_request(&mut self) {
        debug!("asking for new klines");
        while self.state.loading.get_next_page().is_some() {
            let start_time = self.state.loading.left_edge();
            let interval = self.state.props.interval;
            let limit = self.state.loading.page_size();
            let symbol = self.symbol.to_string();
            let p = Mutex::new(Promise::spawn_async(async move {
                Client::kline(symbol, interval, start_time, limit).await
            }));

            let sender = Mutex::new(self.klines_pub.clone());
            self.pool.execute(move || {
                while p.lock().unwrap().ready().is_none() {}

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
                }
            });
        }
    }

    fn export_data(&mut self) {
        debug!("exporting data");

        let name = format!(
            "{}_{}_{}_{:?}.csv",
            self.symbol,
            self.state.props.start_time().timestamp(),
            self.state.props.end_time().timestamp(),
            self.state.props.interval,
        );

        let path = Path::new(&name);
        let f_res = File::create(&path);
        if let Err(err) = f_res {
            error!("failed to create file with error: {err}");
            return;
        }

        let abs_path = path.canonicalize().unwrap();
        debug!("saving to file: {}", abs_path.display());

        let mut wtr = csv::Writer::from_writer(f_res.unwrap());

        let mut data = self.candles.get_data().vals;
        data.sort_by(|a, b| {
            if a.t_close < b.t_close {
                return Ordering::Less;
            }

            if a.t_close > b.t_close {
                return Ordering::Greater;
            }

            Ordering::Equal
        });
        data.iter().for_each(|el| {
            wtr.serialize(el).unwrap();
        });

        if let Some(err) = wtr.flush().err() {
            error!("failed to write to file with error: {err}");
        } else {
            self.toasts
                .success("File exported")
                .set_duration(Some(Duration::from_secs(3)));
            info!("exported to file: {abs_path:?}");
        }

        self.export_state.triggered = false;
    }

    fn update(&mut self) {
        let drag_wrapped = self.drag_sub.recv_timeout(Duration::from_millis(1));
        if let Ok(bounds) = drag_wrapped {
            debug!("got bounds: {bounds:?}");

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
                    debug!("sent props: {props:?}");
                }
                Err(err) => {
                    error!("failed to send props: {err}");
                }
            }

            self.start_download(props, false);
        }

        let export_wrapped = self.export_sub.recv_timeout(Duration::from_millis(1));
        if let Ok(props) = export_wrapped {
            debug!("got export msg: {props:?}");

            self.export_state.triggered = true;

            self.candles.clear();
            self.start_download(props, true);
        }

        let symbol_wrapped = self.symbol_sub.recv_timeout(Duration::from_millis(1));
        if let Ok(symbol) = symbol_wrapped {
            debug!("got symbol: {symbol}");

            self.symbol = symbol.clone();
            self.symbol_pub.send(symbol).unwrap();

            self.candles.clear();

            self.start_download(TimeRangeSettings::default(), true);
        }

        let show_wrapped = self.props_sub.recv_timeout(Duration::from_millis(1));
        if let Ok(props) = show_wrapped {
            debug!("got show button pressed: {props:?}");

            self.start_download(props, true);
        }

        let mut got = 0;
        let mut res = vec![];
        loop {
            let klines_res = self.klines_sub.recv_timeout(Duration::from_millis(1));
            if klines_res.is_err() || got == self.max_frame_pages {
                break;
            }

            klines_res.unwrap().iter().for_each(|k| {
                res.push(*k);
            });
            got += 1;
        }

        if got > 0 {
            trace!("received {} pages of data", got);
            self.state.loading.inc_loaded_pages(got);
            self.update_data(&mut res);
        }

        let finished = self.state.loading.progress() == 1.0;
        if finished && self.export_state.triggered {
            self.export_data();
        }

        self.candles.set_enabled(finished);
    }

    fn draw_data(&mut self, ui: &Ui) {
        if self.data_changed {
            ui.ctx().request_repaint();
            self.drawer_pub
                .send(Arc::new(Mutex::new(Box::new(self.candles.clone()))))
                .unwrap();
            self.data_changed = false;
        }
    }
}

impl AppWidget for Props {
    fn show(&mut self, ui: &mut Ui) {
        self.update();

        self.draw_data(ui);

        if self.symbol.is_empty() {
            ui.label("Select a symbol");
            return;
        }

        TopBottomPanel::top("graph_toolbar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if self.state.loading.progress() < 1.0 && !self.state.loading.has_error {
                    ui.add(
                        ProgressBar::new(self.state.loading.progress())
                            .show_percentage()
                            .animate(true),
                    );
                }
            });
        });

        CentralPanel::default().show_inside(ui, |ui| {
            self.time_range.show(ui);
        });

        self.toasts.show(ui.ctx());
    }
}
