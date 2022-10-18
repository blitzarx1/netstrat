use std::{sync::Mutex, time::Duration};

use crossbeam::channel::{unbounded, Receiver, Sender};
use eframe::{run_native, App, CreationContext, NativeOptions};
use egui::{Align, CentralPanel, Context, Layout, TopBottomPanel};

use tracing::{debug, info, Level};
use tracing_subscriber::EnvFilter;

use crate::windows::{BuffWriter, Net};
use widgets::{AppWidget, Theme};
use windows::{AppWindow, Debug, SymbolsGraph};

mod netstrat;
mod network;
mod sources;
mod widgets;
mod windows;

struct TemplateApp {
    windows: Vec<Box<dyn AppWindow>>,
    active_widget: Option<Mutex<Box<dyn AppWidget>>>,
    active_widget_subs: Vec<Receiver<Mutex<Box<dyn AppWidget>>>>,
}

fn init_logger(s: Sender<Vec<u8>>) {
    let buff = BuffWriter::new(s);

    let has_config = std::env::var("RUST_LOG");
    if has_config.is_err() {
        tracing_subscriber::fmt()
            .with_writer(Mutex::new(buff))
            .with_ansi(false)
            .with_max_level(Level::INFO)
            .with_line_number(false)
            .with_file(false)
            .with_target(false)
            .without_time()
            .init();

        return;
    }

    tracing_subscriber::fmt()
        .with_writer(Mutex::new(buff))
        .with_ansi(false)
        .with_env_filter(EnvFilter::from_default_env())
        .with_line_number(true)
        .with_file(true)
        .with_target(false)
        .init();
}

impl TemplateApp {
    fn new(_ctx: &CreationContext<'_>) -> Self {
        let (buffer_s, buffer_r) = unbounded();

        init_logger(buffer_s);

        info!("starting app");
        let (net_drawer_s, net_drawer_r) = unbounded();
        let (candles_drawer_s, candles_drawer_r) = unbounded();
        Self {
            windows: vec![
                Box::new(Net::new(net_drawer_s, false)),
                Box::new(SymbolsGraph::new(candles_drawer_s, false)),
                Box::new(Debug::new(buffer_r, false)),
            ],
            active_widget: None,
            active_widget_subs: vec![net_drawer_r, candles_drawer_r],
        }
    }

    fn check_widget_event(&mut self) {
        self.active_widget_subs.iter().for_each(|sub| {
            let widget_wrapped = sub.recv_timeout(Duration::from_millis(1));
            if let Ok(widget) = widget_wrapped {
                debug!("got active widget event");

                self.active_widget = Some(widget);
            }
        });
    }
}

impl App for TemplateApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("header").show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                self.windows.iter_mut().for_each(|w| {
                    w.as_mut().toggle_btn(ui);
                });
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            if let Some(w) = &mut self.active_widget {
                w.get_mut().unwrap().show(ui);
            }

            self.windows.iter_mut().for_each(|w| w.show(ui));
        });

        self.check_widget_event();
    }
}

#[tokio::main]
async fn main() {
    run_native(
        "netstrat",
        NativeOptions {
            drag_and_drop_support: true,
            ..Default::default()
        },
        Box::new(|cc| Box::new(TemplateApp::new(cc))),
    );
}
