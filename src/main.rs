use std::sync::Mutex;
use std::time::SystemTime;

use crossbeam::channel::{unbounded, Sender};
use eframe::{run_native, App, CreationContext, NativeOptions};
use egui::{CentralPanel, Context, Layout, TopBottomPanel};

use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

use crate::windows::BuffWriter;
use widgets::Theme;
use windows::{AppWindow, Debug, SymbolsGraph};

mod netstrat;
mod network;
mod sources;
mod widgets;
mod windows;

struct TemplateApp {
    windows: Vec<Box<dyn AppWindow>>,
    theme: Theme,
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
        
        return ;
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
        let (s, r) = unbounded();
        Self {
            windows: vec![
                Box::new(SymbolsGraph::new(s, r, true)),
                Box::new(Debug::new(buffer_r, true)),
            ],
            theme: Theme::new(),
        }
    }
}

impl App for TemplateApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("header").show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(), |ui| {
                ui.add(&mut self.theme);

                self.windows.iter_mut().for_each(|w| {
                    w.as_mut().toggle_btn(ui);
                });
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            self.windows.iter_mut().for_each(|w| w.show(ui));
        });
    }
}

#[tokio::main]
async fn main() {
    run_native(
        "netstrat",
        NativeOptions::default(),
        Box::new(|cc| Box::new(TemplateApp::new(cc))),
    );
}
