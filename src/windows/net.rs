use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

use egui::{ScrollArea, Slider, TextEdit, Ui, Window};
use egui_notify::{Anchor, Toasts};
use petgraph::{Incoming, Outgoing};
use tracing::{debug, error, info};
use urlencoding::encode;

use crate::netstrat::net::{Data, EdgeWeight, Settings};
use crate::AppWindow;

pub struct Net {
    data: Data,
    graph_settings: Settings,
    cone_settings: ConeSettings,
    dot: String,
    visible: bool,
    toasts: Toasts,
}

impl Net {
    pub fn new(visible: bool) -> Self {
        let data = Net::reset_data();
        let dot = data.dot();
        Self {
            visible,
            data,
            dot,
            toasts: Toasts::default().with_anchor(Anchor::TopRight),
            graph_settings: Default::default(),
            cone_settings: Default::default(),
        }
    }

    fn reset_data() -> Data {
        Data::new(Settings::default())
    }

    fn reset(&mut self) {
        let data = Net::reset_data();
        self.dot = data.dot();
        self.data = data;
        self.graph_settings = Settings::default();
    }

    fn create(&mut self) {
        let data = Data::new(self.graph_settings.clone());
        self.dot = data.dot();
        self.data = data;
    }

    fn diamond_filter(&mut self) {
        self.data.diamond_filter();
        self.dot = self.data.dot();
    }

    fn update_visible(&mut self, visible: bool) {
        if visible != self.visible {
            self.visible = visible
        }
    }

    fn update_graph_settings(&mut self, graph_settings: Settings) {
        if self.graph_settings == graph_settings {
            return;
        }

        self.graph_settings = graph_settings;
    }

    fn update(
        &mut self,
        visible: bool,
        graph_settings: Settings,
        cone_coloring_settings: ConeSettings,
        clicks: FrameClicks,
    ) {
        self.update_visible(visible);
        self.update_graph_settings(graph_settings);
        self.update_cone_coloring(cone_coloring_settings);
        self.handle_clicks(clicks);
    }

    fn handle_clicks(&mut self, clicks: FrameClicks) {
        let mut changed = false;

        if clicks.reset {
            self.reset();
            changed = true;
        }

        if clicks.create {
            self.create();
            changed = true;
        }

        if clicks.color_ini_cones {
            self.color_ini_cones();
            changed = true;
        }

        if clicks.color_fin_cones {
            self.color_fin_cones();
            changed = true;
        }

        if clicks.color {
            self.color_custom_cone();
            changed = true;
        }

        if clicks.delete {
            self.delete_custom_cone();
            changed = true;
        }

        if clicks.diamond_filter {
            self.diamond_filter();
            changed = true;
        }

        if clicks.export_dot {
            self.export_dot();
        }

        if changed {
            self.toasts
                .success("Graph changed")
                .set_duration(Some(Duration::from_secs(3)));
        }
    }

    fn export_dot(&mut self) {
        let path = Path::new("graph_export.dot");
        let f_res = File::create(&path);
        match f_res {
            Ok(mut f) => {
                let abs_path = path.canonicalize().unwrap();
                debug!("exporting graph to file: {}", abs_path.display());

                if f.write_all(self.dot.as_bytes()).is_err() {
                    error!("failed to export graph")
                }

                if let Some(err) = f.flush().err() {
                    error!("failed to write to file with error: {err}");
                }

                self.toasts
                    .success("File exported")
                    .set_duration(Some(Duration::from_secs(3)));
                info!("exported to file: {abs_path:?}");
            }
            Err(err) => {
                error!("failed to create file with error: {err}");
            }
        }
    }

    fn update_cone_coloring(&mut self, cone_coloring_settings: ConeSettings) {
        if self.cone_settings == cone_coloring_settings {
            return;
        }

        self.cone_settings = cone_coloring_settings
    }

    fn color_custom_cone(&mut self) {
        self.dot = self.data.dot_with_custom_cone(
            self.cone_settings.node_name.clone(),
            match self.cone_settings.cone_type.clone() {
                ConeType::Minus => Incoming,
                ConeType::Plus => Outgoing,
            },
            self.cone_settings.max_steps,
        )
    }

    fn delete_custom_cone(&mut self) {
        self.data.remove_cone(
            self.cone_settings.node_name.clone(),
            match self.cone_settings.cone_type.clone() {
                ConeType::Minus => Incoming,
                ConeType::Plus => Outgoing,
            },
            self.cone_settings.max_steps,
        );
        self.dot = self.data.dot();
    }

    fn color_ini_cones(&mut self) {
        self.dot = self.data.dot_with_ini_cones();
    }

    fn color_fin_cones(&mut self) {
        self.dot = self.data.dot_with_fin_cones();
    }
}

impl AppWindow for Net {
    fn toggle_btn(&mut self, ui: &mut Ui) {
        if ui.button("net").clicked() {
            self.update_visible(!self.visible)
        }
    }

    fn show(&mut self, ui: &mut Ui) {
        let mut visible = self.visible;
        let mut graph_settings = self.graph_settings.clone();
        let mut cone_coloring_settings = self.cone_settings.clone();
        let mut dot = self.dot.clone();
        let mut clicks = FrameClicks::default();

        Window::new("net").open(&mut visible).show(ui.ctx(), |ui| {
            ui.collapsing("Create", |ui| {
                ui.add(Slider::new(&mut graph_settings.ini_cnt, 1..=25).text("ini_cnt"));
                ui.add(Slider::new(&mut graph_settings.fin_cnt, 1..=25).text("fin_cnt"));
                ui.add(
                    Slider::new(
                        &mut graph_settings.total_cnt,
                        graph_settings.ini_cnt + graph_settings.fin_cnt..=100,
                    )
                    .text("total_cnt"),
                );
                ui.add(
                    Slider::new(&mut graph_settings.max_out_degree, 2..=10).text("max_out_degree"),
                );
                ui.add_space(10.0);
                ui.checkbox(&mut graph_settings.no_twin_edges, "No twin edges");
                ui.add_space(10.0);
                ui.label("Edge weights");
                ui.radio_value(
                    &mut graph_settings.edge_weight_type,
                    EdgeWeight::Random,
                    "Random",
                );
                ui.horizontal_top(|ui| {
                    ui.radio_value(
                        &mut graph_settings.edge_weight_type,
                        EdgeWeight::Fixed,
                        "Fixed",
                    );
                    ui.add(Slider::new(&mut graph_settings.edge_weight, 0.0..=1.0));
                });
                ui.add_space(10.0);
                ui.horizontal_top(|ui| {
                    if ui.button("create").clicked() {
                        clicks.create = true;
                    }
                    if ui.button("reset").clicked() {
                        clicks.reset = true;
                    }
                });
            });

            ui.collapsing("Cone Edit", |ui| {
                ui.horizontal_top(|ui| {
                    if ui.button("color ini cone").clicked() {
                        clicks.color_ini_cones = true;
                    }
                    if ui.button("color fin cone").clicked() {
                        clicks.color_fin_cones = true;
                    }
                });
                ui.add_space(10.0);
                ui.label("Custom cone");
                ui.add(
                    TextEdit::singleline(&mut cone_coloring_settings.node_name)
                        .hint_text("Node name"),
                );
                ui.radio_value(
                    &mut cone_coloring_settings.cone_type,
                    ConeType::Minus,
                    "Minus",
                );
                ui.radio_value(
                    &mut cone_coloring_settings.cone_type,
                    ConeType::Plus,
                    "Plus",
                );
                ui.add(Slider::new(&mut cone_coloring_settings.max_steps, -1..=10).text("Steps"));
                ui.add_space(10.0);
                ui.horizontal_top(|ui| {
                    if ui.button("color").clicked() {
                        clicks.color = true;
                    };
                    if ui.button("delete").clicked() {
                        clicks.delete = true;
                    }
                });
            });

            ui.collapsing("Graph Edit", |ui| {
                if ui.button("diamond filter").clicked() {
                    clicks.diamond_filter = true;
                }
            });

            ui.add_space(10.0);
            ui.horizontal_top(|ui| {
                if ui.button("show").clicked() {
                    open::that(format!(
                        "https://dreampuf.github.io/GraphvizOnline/#{}",
                        encode(self.dot.as_str())
                    ))
                    .unwrap();
                }
                if ui.button("export dot").clicked() {
                    clicks.export_dot = true;
                };
            });
            ui.add_space(10.0);
            ScrollArea::vertical().show(ui, |ui| ui.text_edit_multiline(&mut dot));

            self.toasts.show(ui.ctx());
        });

        self.update(visible, graph_settings, cone_coloring_settings, clicks);
    }
}

#[derive(PartialEq, Eq, Clone)]
struct ConeSettings {
    node_name: String,
    cone_type: ConeType,
    max_steps: i32,
}

impl Default for ConeSettings {
    fn default() -> Self {
        Self {
            cone_type: ConeType::Plus,
            max_steps: -1,
            node_name: Default::default(),
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
enum ConeType {
    /// Go along arrow from head to tail
    Minus,
    /// Go along arrow from tail to head
    Plus,
}

#[derive(Default)]
struct FrameClicks {
    reset: bool,
    create: bool,
    diamond_filter: bool,
    color_ini_cones: bool,
    color_fin_cones: bool,
    color: bool,
    export_dot: bool,
    delete: bool,
}
