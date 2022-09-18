use std::collections::HashSet;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::Path;
use std::time::{Duration, SystemTime};

use egui::{CentralPanel, LayerId, Response, ScrollArea, Slider, TextEdit, Ui, Widget};
use egui_notify::{Anchor, Toasts};
use petgraph::{Incoming, Outgoing};
use tracing::{debug, error, info};
use urlencoding::encode;

use crate::netstrat::net::{Data, EdgeWeight, Settings};
use crate::widgets::OpenDropFile;

#[derive(PartialEq, Clone)]
struct ConeSettings {
    node_name: String,
    cone_dir: ConeDir,
    cone_type: ConeType,
    max_steps: i32,
}

impl Default for ConeSettings {
    fn default() -> Self {
        Self {
            cone_dir: ConeDir::Plus,
            max_steps: -1,
            cone_type: ConeType::Custom,
            node_name: Default::default(),
        }
    }
}

#[derive(PartialEq, Clone)]
enum ConeDir {
    /// Go along arrow from head to tail
    Minus,
    /// Go along arrow from tail to head
    Plus,
}

#[derive(PartialEq, Clone)]
enum ConeType {
    Custom,
    Initial,
    Final,
}

#[derive(Default)]
struct ButtonClicks {
    reset: bool,
    create: bool,
    diamond_filter: bool,
    color_cones: bool,
    color_cycles: bool,
    export_dot: bool,
    delete_cone: bool,
    delete_cycles: bool,
}

pub struct Net {
    data: Data,
    graph_settings: Settings,
    cone_settings: ConeSettings,
    open_drop_file: OpenDropFile,
    toasts: Toasts,
    selected_cycles: HashSet<usize>,
}

impl Default for Net {
    fn default() -> Self {
        let data = Net::reset_data();
        Self {
            data,
            open_drop_file: Default::default(),
            toasts: Toasts::default().with_anchor(Anchor::TopRight),
            graph_settings: Default::default(),
            cone_settings: Default::default(),
            selected_cycles: Default::default(),
        }
    }
}

impl Net {
    fn reset_data() -> Data {
        Data::new(Settings::default())
    }

    fn reset(&mut self) {
        let data = Net::reset_data();
        self.data = data;
        self.graph_settings = Settings::default();
    }

    fn create(&mut self) {
        let data = Data::new(self.graph_settings.clone());
        self.data = data;
    }

    fn diamond_filter(&mut self) {
        self.data.diamond_filter();
    }

    fn update_graph_settings(&mut self, graph_settings: Settings) {
        if self.graph_settings == graph_settings {
            return;
        }

        self.graph_settings = graph_settings;
    }

    fn update(
        &mut self,
        graph_settings: Settings,
        cone_coloring_settings: ConeSettings,
        clicks: ButtonClicks,
        selected_cycles: HashSet<usize>,
    ) {
        self.update_graph_settings(graph_settings);
        self.update_cone_coloring(cone_coloring_settings);
        self.handle_selected_cycles(selected_cycles);
        self.handle_clicks(clicks);
        self.handle_opened_file();
    }

    fn handle_opened_file(&mut self) {
        if let Some(path) = self.open_drop_file.path() {
            debug!("opening file: {path}");
            let p = Path::new(path.as_str());
            let extension = p.extension();

            if extension.is_none() || !extension.unwrap().eq_ignore_ascii_case("dot") {
                self.toasts
                    .error("Invalid file extension. Only '.dot' files are allowed.")
                    .set_duration(Some(Duration::from_secs(5)));
                return;
            }

            let dot_data = read_to_string(p).unwrap();
            let data = Data::from_dot(dot_data);
            if data.is_none() {
                self.toasts.error("Failed to parse imported file");
                return;
            }

            self.data = data.unwrap();
            self.toasts.success("File imported");
        }
    }

    fn handle_selected_cycles(&mut self, selected_cycles: HashSet<usize>) {
        if self.selected_cycles == selected_cycles {
            return;
        }

        self.selected_cycles = selected_cycles;
    }

    fn handle_clicks(&mut self, clicks: ButtonClicks) {
        if clicks.reset {
            info!("resetting graph params");
            self.reset();
            self.trigger_changed_toast();
        }

        if clicks.create {
            info!("generatin graph");
            self.create();
            self.trigger_changed_toast();
        }

        if clicks.color_cones {
            info!("coloring cones");
            self.color_cone();
            self.trigger_changed_toast();
        }

        if clicks.delete_cone {
            info!("deleting cone");
            self.delete_custom_cone();
            self.trigger_changed_toast();
        }

        if clicks.diamond_filter {
            info!("applying diamond filter");
            self.diamond_filter();
            self.trigger_changed_toast();
        }

        if clicks.color_cycles {
            info!("coloring cycles");
            self.color_cycles();
            self.trigger_changed_toast();
        }

        if clicks.delete_cycles {
            self.delete_cycles();
            self.trigger_changed_toast();
        }

        if clicks.export_dot {
            self.export_dot();
        }
    }

    fn delete_cycles(&mut self) {
        self.data.delete_cycles(&self.selected_cycles);
    }

    fn trigger_changed_toast(&mut self) {
        self.toasts
            .success("Graph changed")
            .set_duration(Some(Duration::from_secs(3)));
    }

    fn export_dot(&mut self) {
        let name = format!(
            "graph_{:?}.dot",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        );
        let path = Path::new(name.as_str());
        let f_res = File::create(&path);
        match f_res {
            Ok(mut f) => {
                let abs_path = path.canonicalize().unwrap();
                debug!("exporting graph to file: {}", abs_path.display());

                if f.write_all(self.data.dot().as_bytes()).is_err() {
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

    fn color_cone(&mut self) {
        match self.cone_settings.cone_type {
            ConeType::Custom => self.color_custom_cone(),
            ConeType::Initial => self.color_ini_cones(),
            ConeType::Final => self.color_fin_cones(),
        }
    }

    fn color_custom_cone(&mut self) {
        self.data.color_custom_cone(
            self.cone_settings.node_name.clone(),
            match self.cone_settings.cone_dir.clone() {
                ConeDir::Minus => Incoming,
                ConeDir::Plus => Outgoing,
            },
            self.cone_settings.max_steps,
        )
    }

    fn delete_custom_cone(&mut self) {
        self.data.delete_cone(
            self.cone_settings.node_name.clone(),
            match self.cone_settings.cone_dir.clone() {
                ConeDir::Minus => Incoming,
                ConeDir::Plus => Outgoing,
            },
            self.cone_settings.max_steps,
        );
    }

    fn color_ini_cones(&mut self) {
        self.data.color_ini_cones();
    }

    fn color_fin_cones(&mut self) {
        self.data.color_fin_cones();
    }

    fn color_cycles(&mut self) {
        self.data.color_cycles(&self.selected_cycles);
    }
}

impl Widget for &mut Net {
    fn ui(self, ui: &mut Ui) -> Response {
        let mut graph_settings = self.graph_settings.clone();
        let mut cone_coloring_settings = self.cone_settings.clone();
        let mut dot = self.data.dot();
        let mut selected_cycles = self.selected_cycles.clone();
        let mut clicks = ButtonClicks::default();

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
            ui.add(Slider::new(&mut graph_settings.max_out_degree, 2..=10).text("max_out_degree"));
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

        ui.collapsing("Import/Export", |ui| {
            ui.add(&mut self.open_drop_file);
            ui.add_space(10.0);
            if ui.button("export dot").clicked() {
                clicks.export_dot = true;
            };
        });

        ui.collapsing("Edit", |ui| {
            if ui.button("diamond filter").clicked() {
                clicks.diamond_filter = true;
            }
        });

        ui.collapsing("Cones", |ui| {
            ui.radio_value(
                &mut cone_coloring_settings.cone_type,
                ConeType::Initial,
                "Initial",
            );
            ui.add_space(10.0);
            ui.radio_value(
                &mut cone_coloring_settings.cone_type,
                ConeType::Final,
                "Final",
            );
            ui.add_space(10.0);
            ui.radio_value(
                &mut cone_coloring_settings.cone_type,
                ConeType::Custom,
                "Custom",
            );
            ui.add(
                TextEdit::singleline(&mut cone_coloring_settings.node_name).hint_text("Node name"),
            );
            ui.radio_value(
                &mut cone_coloring_settings.cone_dir,
                ConeDir::Minus,
                "Minus",
            );
            ui.radio_value(&mut cone_coloring_settings.cone_dir, ConeDir::Plus, "Plus");
            ui.add(Slider::new(&mut cone_coloring_settings.max_steps, -1..=10).text("Steps"));
            ui.add_space(10.0);
            ui.horizontal_top(|ui| {
                if ui.button("color").clicked() {
                    clicks.color_cones = true;
                };
                if ui.button("delete").clicked() {
                    clicks.delete_cone = true;
                }
            });
        });

        ui.collapsing("Cycles", |ui| {
            ui.label("Cycles which are reachable from ini nodes");
            ScrollArea::vertical().show(ui, |ui| {
                self.data
                    .clone()
                    .cycles()
                    .iter()
                    .enumerate()
                    .for_each(|(i, c)| {
                        let checked = selected_cycles.contains(&i);
                        if ui
                            .selectable_label(checked, format!("{} steps", c.len()))
                            .clicked()
                        {
                            match checked {
                                true => selected_cycles.remove(&i),
                                false => selected_cycles.insert(i),
                            };
                        };
                    });
            });
            ui.horizontal_top(|ui| {
                if ui.button("color").clicked() {
                    clicks.color_cycles = true;
                };
                if ui.button("delete").clicked() {
                    clicks.delete_cycles = true;
                }
            });
        });

        ui.collapsing("Dot preview", |ui| {
            ScrollArea::vertical().show(ui, |ui| ui.text_edit_multiline(&mut dot));
        });

        ui.add_space(10.0);
        let response = ui
            .horizontal_top(|ui| {
                if ui.button("show").clicked() {
                    open::that(format!(
                        "https://dreampuf.github.io/GraphvizOnline/#{}",
                        encode(self.data.dot().as_str())
                    ))
                    .unwrap();
                }
            })
            .response;

        self.toasts.show(ui.ctx());
        self.update(
            graph_settings,
            cone_coloring_settings,
            clicks,
            selected_cycles,
        );

        response
    }
}
