use std::env;
use std::sync::Mutex;
use std::time::SystemTime;

use crossbeam::channel::unbounded;
use eframe::{run_native, App, CreationContext, NativeOptions};
use egui::{CentralPanel, Context, Layout, TopBottomPanel};

use tracing::{info, trace, Level};

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

impl TemplateApp {
    fn new(_ctx: &CreationContext<'_>) -> Self {
        let (buffer_s, buffer_r) = unbounded();
        let buff = BuffWriter {
            publisher: buffer_s,
        };

        tracing_subscriber::fmt()
            .with_writer(Mutex::new(buff))
            .with_max_level(parse_log_level())
            .with_ansi(false)
            .init();

        let (s, r) = unbounded();
        Self {
            windows: vec![
                Box::new(SymbolsGraph::new(s, r, true)),
                Box::new(Debug::new(buffer_r, true, 500)),
            ],
            theme: Theme::new(),
        }
    }
}

impl App for TemplateApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let start = SystemTime::now();

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

        trace!(
            "time elapsed per frame: {:?}",
            SystemTime::now()
                .duration_since(start)
                .expect("failed to compute duration_since")
        );
    }
}

fn parse_log_level() -> Level {
    if let Ok(level) = env::var("RUST_LOG") {
        match level.to_lowercase().as_str() {
            "error" => return Level::ERROR,
            "info" => return Level::INFO,
            "debug" => return Level::DEBUG,
            "trace" => return Level::TRACE,
            _ => return Level::DEBUG,
        }
    }

    Level::INFO
}

#[tokio::main]
async fn main() {
    run_native(
        "netstrat",
        NativeOptions::default(),
        Box::new(|cc| Box::new(TemplateApp::new(cc))),
    );
}
