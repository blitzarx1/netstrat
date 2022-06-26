use std::collections::HashMap;
use std::time::SystemTime;

use crossbeam::channel::unbounded;

use eframe::{run_native, App, CreationContext, NativeOptions};

use egui::{
    CentralPanel, Context, Layout, ScrollArea, SidePanel, TextEdit, TopBottomPanel, Window,
};
use tracing::{info, trace};
use widgets::graph::graph::Graph;
use widgets::symbols::Symbols;
use widgets::theme::Theme;

mod network;
mod sources;
mod widgets;
use tokio;

struct TemplateApp {
    visibility: HashMap<String, bool>,
    candle_plot: Graph,
    symbols: Symbols,
    theme: Theme,
}

impl TemplateApp {
    fn new(_ctx: &CreationContext<'_>) -> Self {
        info!("creating app");

        let (s, r) = unbounded();

        // TODO: create register function
        let mut visibility_map = HashMap::new();
        visibility_map.insert("debug".to_string(), false);

        Self {
            visibility: visibility_map,
            candle_plot: Graph::new(r),
            theme: Theme::new(),
            symbols: Symbols::new(s),
            debug_visible: false,
        }
    }

    fn render_side_panel(&mut self, ctx: &Context) {
        SidePanel::left("side_panel").show(ctx, |ui| ui.add(&mut self.symbols));
    }
}

impl App for TemplateApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let start = SystemTime::now();

        TopBottomPanel::top("header").show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(), |ui| {
                ui.add(&mut self.theme);

                if ui.button("debug").clicked() {
                    let debug_visible = *self.visibility.get_mut("debug").unwrap();
                    self.visibility.insert("debug".to_string(), !debug_visible);
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            Window::new("debug")
                .open(self.visibility.get_mut("debug").unwrap())
                .show(ctx, |ui| {
                    ScrollArea::both()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let mut text = "text";
                            TextEdit::multiline(&mut text).desired_rows(10).show(ui);
                        });
                });

            ui.add(&mut self.candle_plot);
        });

        self.render_side_panel(ctx);
        self.render_center_panel(ctx);

        let elapsed = SystemTime::now()
            .duration_since(start)
            .expect("failed to compute duration_since");
        trace!("elapsed for update frame: {elapsed:?}");
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        info!("called save")
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    run_native(
        "hedgegraph",
        NativeOptions::default(),
        Box::new(|cc| Box::new(TemplateApp::new(cc))),
    );
}
