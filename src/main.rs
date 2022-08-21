use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Mutex;
use std::time::SystemTime;

use crossbeam::channel::unbounded;
use eframe::{run_native, App, CreationContext, NativeOptions};
use egui::{CentralPanel, Context, Layout, TopBottomPanel};
use tokio;
use tracing::{info, trace};
use tracing_subscriber::util::SubscriberInitExt;

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
            .with_ansi(false)
            .init();

        info!("Creating app...");

        let (s, r) = unbounded();

        let mut visibility_map = HashMap::new();
        visibility_map.insert("debug".to_string(), false);

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

#[tokio::main]
async fn main() {
    run_native(
        "netstrat",
        NativeOptions::default(),
        Box::new(|cc| Box::new(TemplateApp::new(cc))),
    );
}
