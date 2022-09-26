use crossbeam::channel::Sender;
use egui_extras::image::load_svg_bytes;
use graphviz_rust::cmd::{CommandArg, Format};
use graphviz_rust::printer::PrinterContext;
use graphviz_rust::{exec, parse};
use std::collections::HashSet;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use egui::{ScrollArea, Slider, TextEdit, Ui};
use egui_notify::{Anchor, Toasts};
use petgraph::{Incoming, Outgoing};
use tracing::{debug, error, info};
use urlencoding::encode;

use crate::netstrat::net::{ConeSettings, Data, EdgeWeight, Settings};
use crate::widgets::AppWidget;
use crate::widgets::OpenDropFile;

use super::edges_input::EdgesInput;
use super::nodes_input::NodesInput;
use super::NetVisualizer;

#[derive(PartialEq, Clone, Default)]
struct NodesAndEdgeSettings {
    nodes_input: NodesInput,
    edges_input: EdgesInput,
}

#[derive(PartialEq, Clone)]
struct ConeSettingsInputs {
    cone_type: ConeType,
    settings: Vec<ConeInput>,
}

impl Default for ConeSettingsInputs {
    fn default() -> Self {
        Self {
            cone_type: ConeType::Custom,
            settings: vec![ConeInput::default()],
        }
    }
}

#[derive(PartialEq, Clone, Default)]
struct ConeInput {
    nodes_names: NodesInput,
    cone_settings: ConeSettings,
}

impl ConeInput {
    fn prepare_settings(&self) -> ConeSettings {
        ConeSettings {
            roots_weights: self.nodes_names.splitted(),
            dir: self.cone_settings.dir,
            max_steps: self.cone_settings.max_steps,
        }
    }
}

#[derive(PartialEq, Clone)]
enum ConeType {
    Custom,
    Initial,
    Final,
}

#[derive(Default, Clone)]
struct ButtonClicks {
    reset: bool,
    create: bool,
    color_cones: bool,
    color_cycles: bool,
    export_dot: bool,
    export_svg: bool,
    delete_cone: bool,
    delete_cycles: bool,
    color_nodes_and_edges: bool,
    delete_nodes_and_edges: bool,
}

pub struct NetProps {
    data: Data,
    graph_settings: Settings,
    cone_settings: ConeSettingsInputs,
    nodes_and_edges_settings: NodesAndEdgeSettings,
    open_drop_file: OpenDropFile,
    net_visualizer: NetVisualizer,
    widget_pub: Sender<Mutex<Box<dyn AppWidget>>>,
    toasts: Toasts,
    selected_cycles: HashSet<usize>,
}

impl NetProps {
    pub fn new(widget_pub: Sender<Mutex<Box<dyn AppWidget>>>) -> Self {
        let data = NetProps::reset_data();
        let mut s = Self {
            data,
            widget_pub,
            open_drop_file: Default::default(),
            toasts: Toasts::default().with_anchor(Anchor::TopRight),
            graph_settings: Default::default(),
            net_visualizer: NetVisualizer::default(),
            cone_settings: Default::default(),
            selected_cycles: Default::default(),
            nodes_and_edges_settings: Default::default(),
        };

        s.update_frame();

        s
    }

    fn reset_data() -> Data {
        Data::new(Settings::default())
    }

    fn reset(&mut self) {
        let data = NetProps::reset_data();
        self.data = data;
        self.graph_settings = Settings::default();
    }

    fn create(&mut self) {
        let data = Data::new(self.graph_settings.clone());
        self.data = data;
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
        cone_coloring_settings: ConeSettingsInputs,
        clicks: ButtonClicks,
        selected_cycles: HashSet<usize>,
        nodes_and_edges_settings: NodesAndEdgeSettings,
    ) {
        self.update_graph_settings(graph_settings);
        self.update_cone_settings(cone_coloring_settings);
        self.handle_selected_cycles(selected_cycles);
        self.handle_nodes_and_edges_settings(nodes_and_edges_settings);
        self.handle_update_clicks(clicks.clone());
        self.handle_opened_file();

        if clicks.export_dot {
            self.export_dot();
        }

        if clicks.export_svg {
            self.export_svg();
        }
    }

    fn handle_nodes_and_edges_settings(&mut self, nodes_and_edges_settings: NodesAndEdgeSettings) {
        if self.nodes_and_edges_settings != nodes_and_edges_settings {
            self.nodes_and_edges_settings = nodes_and_edges_settings
        }
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

            self.update_state();
        }
    }

    fn handle_selected_cycles(&mut self, selected_cycles: HashSet<usize>) {
        if self.selected_cycles == selected_cycles {
            return;
        }

        self.selected_cycles = selected_cycles;
    }

    fn update_state(&mut self) {
        debug!("updating grah state");
        self.update_frame();
        self.trigger_changed_toast();
    }

    fn handle_update_clicks(&mut self, clicks: ButtonClicks) {
        if clicks.reset {
            info!("resetting graph params");
            self.reset();
        }

        if clicks.create {
            info!("generatin graph");
            self.create();
            self.update_state();
        }

        if clicks.color_cones {
            info!("coloring cones");
            self.color_cones();
            self.update_state();
        }

        if clicks.delete_cone {
            info!("deleting cone");
            self.delete_cones();
            self.update_state();
        }

        if clicks.color_cycles {
            info!("coloring cycles");
            self.color_cycles();
            self.update_state();
        }

        if clicks.delete_cycles {
            info!("deleting cycles");
            self.delete_cycles();
            self.update_state();
        }

        if clicks.delete_nodes_and_edges {
            info!("deleting nodes and edges");
            self.data.delete_nodes_and_edges(
                self.nodes_and_edges_settings.nodes_input.splitted(),
                self.nodes_and_edges_settings.edges_input.splitted(),
            );
            self.update_state();
        }

        if clicks.color_nodes_and_edges {
            info!("coloring nodes and edges");
            self.data.color_nodes_and_edges(
                self.nodes_and_edges_settings.nodes_input.splitted(),
                self.nodes_and_edges_settings.edges_input.splitted(),
            );
            self.update_state();
        }
    }

    fn delete_cycles(&mut self) {
        self.data.delete_cycles(&self.selected_cycles);
        self.selected_cycles = Default::default();
    }

    fn update_frame(&mut self) {
        let graph_svg = exec(
            parse(self.data.dot().as_str()).unwrap(),
            &mut PrinterContext::default(),
            vec![CommandArg::Format(Format::Svg)],
        )
        .unwrap();

        let image = load_svg_bytes(graph_svg.as_bytes()).unwrap();
        self.net_visualizer.update_image(image);
    }

    fn trigger_changed_toast(&mut self) {
        self.toasts
            .success("Graph changed")
            .set_duration(Some(Duration::from_secs(3)));
    }

    fn export_dot(&mut self) {
        let name = format!("{}.dot", generate_unique_export_name());
        self.write_to_file(name, self.data.dot().as_bytes())
    }

    fn export_svg(&mut self) {
        let name = format!("{}.svg", generate_unique_export_name());
        let graph_svg = exec(
            parse(self.data.dot().as_str()).unwrap(),
            &mut PrinterContext::default(),
            vec![CommandArg::Format(Format::Svg)],
        );
        if let Ok(data) = graph_svg {
            self.write_to_file(name, data.as_bytes());
            return;
        }

        error!("failed to create svg: {}", graph_svg.err().unwrap());
        self.toasts.error("Failed to export to file");
    }

    fn write_to_file(&mut self, name: String, data: &[u8]) {
        let path = Path::new(name.as_str());
        let f_res = File::create(&path);
        match f_res {
            Ok(mut f) => {
                let abs_path = path.canonicalize().unwrap();
                debug!("exporting graph to file: {}", abs_path.display());

                if f.write_all(data).is_err() {
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

    fn update_cone_settings(&mut self, cone_settings: ConeSettingsInputs) {
        if self.cone_settings == cone_settings {
            return;
        }

        self.cone_settings = cone_settings;
    }

    fn color_cones(&mut self) {
        match self.cone_settings.cone_type {
            ConeType::Custom => self.data.color_cones(
                self.cone_settings
                    .settings
                    .iter_mut()
                    .map(|input| input.prepare_settings())
                    .collect(),
            ),
            ConeType::Initial => self.data.color_ini_cones(),
            ConeType::Final => self.data.color_fin_cones(),
        }
    }

    fn delete_cones(&mut self) {
        match self.cone_settings.cone_type {
            ConeType::Custom => self.data.delete_cones(
                self.cone_settings
                    .settings
                    .iter_mut()
                    .map(|input| input.prepare_settings())
                    .collect(),
            ),
            ConeType::Initial => self.data.delete_initial_cone(),
            ConeType::Final => self.data.delete_final_cone(),
        };
        self.cone_settings = Default::default();
    }

    fn color_cycles(&mut self) {
        self.data.color_cycles(&self.selected_cycles);
    }
}

impl AppWidget for NetProps {
    fn show(&mut self, ui: &mut Ui) {
        let mut graph_settings = self.graph_settings.clone();
        let mut cone_settings = self.cone_settings.clone();
        let mut dot = self.data.dot();
        let mut selected_cycles = self.selected_cycles.clone();
        let mut clicks = ButtonClicks::default();
        let mut nodes_and_edges_settings = self.nodes_and_edges_settings.clone();

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
            ui.checkbox(&mut graph_settings.diamond_filter, "Apply diamond filter");
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
                ui.add_enabled(
                    graph_settings.edge_weight_type == EdgeWeight::Fixed,
                    Slider::new(&mut graph_settings.edge_weight, 0.0..=1.0),
                );
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
            self.open_drop_file.show(ui);
            ui.add_space(10.0);
            ui.horizontal_top(|ui| {
                if ui.button("export dot").clicked() {
                    clicks.export_dot = true;
                };
                if ui.button("export svg").clicked() {
                    clicks.export_svg = true;
                }
            });
        });

        ui.collapsing("Nodes and Edges", |ui| {
            ui.add(
                TextEdit::singleline(&mut nodes_and_edges_settings.nodes_input.input)
                    .hint_text("ini_1, 5, 10"),
            );
            ui.add_space(5.0);
            ui.add(
                TextEdit::singleline(&mut nodes_and_edges_settings.edges_input.input)
                    .hint_text("ini_1 -> 5, 10 -> fin_3"),
            );
            ui.add_space(10.0);
            ui.horizontal_top(|ui| {
                if ui.button("color").clicked() {
                    clicks.color_nodes_and_edges = true;
                };
                if ui.button("delete").clicked() {
                    clicks.delete_nodes_and_edges = true;
                }
            });
        });

        ui.collapsing("Cones", |ui| {
            ui.radio_value(&mut cone_settings.cone_type, ConeType::Initial, "Initial");
            ui.radio_value(&mut cone_settings.cone_type, ConeType::Final, "Final");
            ui.radio_value(&mut cone_settings.cone_type, ConeType::Custom, "Custom");
            ui.add_enabled_ui(cone_settings.cone_type == ConeType::Custom, |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        cone_settings.settings.iter_mut().for_each(|cone_input| {
                            ui.add_space(5.0);
                            ui.add(
                                TextEdit::singleline(&mut cone_input.nodes_names.input)
                                    .hint_text("ini_1, 5, 10"),
                            );
                            ui.radio_value(&mut cone_input.cone_settings.dir, Incoming, "Minus");
                            ui.radio_value(&mut cone_input.cone_settings.dir, Outgoing, "Plus");
                            ui.add(
                                Slider::new(&mut cone_input.cone_settings.max_steps, -1..=10)
                                    .text("Steps"),
                            );
                        });
                    });
                ui.add_space(10.0);
                ui.horizontal_top(|ui| {
                    if ui.button("+").clicked() {
                        cone_settings.settings.push(ConeInput::default())
                    };
                    ui.add_enabled_ui(cone_settings.settings.len() > 1, |ui| {
                        if ui.button("-").clicked() {
                            cone_settings
                                .settings
                                .remove(cone_settings.settings.len() - 1);
                        }
                    });
                });
            });
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
            ScrollArea::vertical()
                .auto_shrink([false, true])
                .show(ui, |ui| {
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
            ScrollArea::vertical()
                .auto_shrink([false, true])
                .show(ui, |ui| ui.text_edit_multiline(&mut dot));
        });

        ui.add_space(10.0);
        ui.horizontal_top(|ui| {
            if ui.button("Open in Explorer").clicked() {
                open::that(format!(
                    "https://dreampuf.github.io/GraphvizOnline/#{}",
                    encode(self.data.dot().as_str())
                ))
                .unwrap();
            }
        });

        if self.net_visualizer.changed() {
            self.widget_pub
                .send(Mutex::new(Box::new(self.net_visualizer.clone())))
                .unwrap();
        }

        self.toasts.show(ui.ctx());
        self.update(
            graph_settings,
            cone_settings,
            clicks,
            selected_cycles,
            nodes_and_edges_settings,
        );
    }
}

fn generate_unique_export_name() -> String {
    format!(
        "graph_{:?}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    )
}
