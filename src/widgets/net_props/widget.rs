use std::collections::HashSet;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use crossbeam::channel::Sender;
use egui::{ScrollArea, Slider, TextEdit, Ui};
use egui_extras::image::load_svg_bytes;
use egui_notify::{Anchor, Toasts};
use graphviz_rust::cmd::{CommandArg, Format};
use graphviz_rust::printer::PrinterContext;
use graphviz_rust::{exec, parse};
use petgraph::{Incoming, Outgoing};
use tracing::{debug, error, info};
use urlencoding::encode;

use crate::netstrat::{Bus, Drawer};
use crate::widgets::history::{Clicks, History, Step};
use crate::widgets::image_drawer;
use crate::widgets::matrix::Matrix;
use crate::widgets::AppWidget;
use crate::widgets::OpenDropFile;

use super::button_clicks::ButtonClicks;
use super::cones::{ConeInput, ConeSettingsInputs, ConeType};
use super::graph::State;
use super::interactions::Interactions;
use super::nodes_and_edges::NodesAndEdgeSettings;
use super::settings::{EdgeWeight, NetSettings};

pub struct NetProps {
    graph_state: State,
    adj_matrix: Matrix,
    adj_matrix_power: Matrix,
    net_settings: NetSettings,
    history: History,
    cone_settings: ConeSettingsInputs,
    nodes_and_edges_settings: NodesAndEdgeSettings,
    matrix_power_input: String,
    open_drop_file: OpenDropFile,
    net_drawer: Arc<Mutex<Box<dyn Drawer>>>,
    drawer_pub: Sender<Arc<Mutex<Box<dyn Drawer>>>>,
    toasts: Toasts,
    selected_cycles: HashSet<usize>,
}

impl NetProps {
    pub fn new(drawer_pub: Sender<Arc<Mutex<Box<dyn Drawer>>>>) -> Self {
        let data = NetProps::reset_data();

        let history = History::new_with_initial_step(Step {
            name: "create".to_string(),
            data: data.clone(),
        });

        let adj_matrix = Matrix::new(data.matrix(), Box::new(Bus::new()));
        let adj_matrix_power = adj_matrix.clone();

        let mut s = Self {
            drawer_pub,
            history,
            adj_matrix,
            adj_matrix_power,
            graph_state: data,
            net_drawer: Arc::new(Mutex::new(Box::new(image_drawer::ImageDrawer::default()))),
            toasts: Toasts::default().with_anchor(Anchor::TopRight),
            open_drop_file: Default::default(),
            net_settings: Default::default(),
            cone_settings: Default::default(),
            matrix_power_input: Default::default(),
            selected_cycles: Default::default(),
            nodes_and_edges_settings: Default::default(),
        };

        s.update_data();

        s
    }

    fn reset_data() -> State {
        State::new(NetSettings::default())
    }

    fn reset(&mut self) {
        self.graph_state = NetProps::reset_data();

        self.history = History::new_with_initial_step(Step {
            name: "reset".to_string(),
            data: self.graph_state.clone(),
        });

        self.update_data();
        self.reset_settings();
    }

    fn reset_settings(&mut self) {
        self.net_settings = NetSettings::default();
        self.cone_settings = ConeSettingsInputs::default();
        self.nodes_and_edges_settings = NodesAndEdgeSettings::default();
    }

    fn create(&mut self) {
        self.graph_state = State::new(self.net_settings.clone());

        self.history = History::new_with_initial_step(Step {
            name: "create".to_string(),
            data: self.graph_state.clone(),
        });

        self.update_data();
    }

    fn update_graph_settings(&mut self, graph_settings: NetSettings) {
        if self.net_settings == graph_settings {
            return;
        }

        self.net_settings = graph_settings;
    }

    fn update(&mut self, inter: Interactions) {
        self.update_graph_settings(inter.graph_settings);
        self.update_cone_settings(inter.cone_settings);
        self.update_nodes_and_edges_settings(inter.nodes_and_edges_settings);
        self.handle_selected_cycles(inter.selected_cycles);
        self.handle_matrix_power(inter.matrix_power_input);
        self.handle_clicks(inter.clicks);
        self.handle_opened_file();
    }

    fn update_nodes_and_edges_settings(&mut self, nodes_and_edges_settings: NodesAndEdgeSettings) {
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
            let data = State::from_dot(dot_data);
            if data.is_none() {
                self.toasts.error("Failed to parse imported file");
                return;
            }

            self.graph_state = data.unwrap();
            self.history = History::new_with_initial_step(Step {
                name: "load from file".to_string(),
                data: self.graph_state.clone(),
            });
            self.update_data();
            self.reset_settings();
        }
    }

    fn handle_selected_cycles(&mut self, selected_cycles: HashSet<usize>) {
        if self.selected_cycles == selected_cycles {
            return;
        }

        self.selected_cycles = selected_cycles;
    }

    fn handle_matrix_power(&mut self, matrix_power_input: String) {
        if self.matrix_power_input == matrix_power_input {
            return;
        }

        self.matrix_power_input = matrix_power_input;
    }

    fn update_data(&mut self) {
        debug!("updating graph state");

        let matrix_state = self.graph_state.matrix();
        self.adj_matrix.set_state(matrix_state.clone());

        self.adj_matrix_power.set_state(matrix_state);
        self.adj_matrix_power.set_power(self.matrix_power);

        self.update_frame();
        self.trigger_changed_toast();
    }

    fn handle_error(&mut self, msg: &str) {
        self.toasts.error(msg);
        error!(msg);
    }

    fn handle_clicks(&mut self, clicks: ButtonClicks) {
        if clicks.reset {
            info!("resetting graph params");
            self.reset();
        }

        if clicks.create {
            info!("generating graph");
            self.create();
        }

        if clicks.color_cones {
            info!("coloring cones");
            self.color_cones();
        }

        if clicks.delete_cone {
            info!("deleting cone");
            self.delete_cones();
        }

        if clicks.color_cycles {
            info!("coloring cycles");
            self.color_cycles();
        }

        if clicks.delete_cycles {
            info!("deleting cycles");
            self.delete_cycles();
        }

        if let Some(history_click) = self.history.last_click() {
            match history_click {
                Clicks::Up => {
                    info!("navigating history up");
                    match self.history.go_up() {
                        Some(loaded_step) => {
                            self.graph_state = loaded_step.data;
                            self.update_data()
                        }
                        None => self.handle_error("failed to load history"),
                    }
                }
                Clicks::Down => {
                    info!("navigating history down");
                    match self.history.go_down() {
                        Some(loaded_step) => {
                            self.graph_state = loaded_step.data;
                            self.update_data()
                        }
                        None => self.handle_error("failed to load history"),
                    }
                }
                Clicks::Right => {
                    info!("navigating to history sibling");
                    match self.history.go_sibling() {
                        Some(loaded_step) => {
                            self.graph_state = loaded_step.data;
                            self.update_data()
                        }
                        None => self.handle_error("failed to load history"),
                    }
                }
            }
        }

        if clicks.delete_nodes_and_edges {
            info!("deleting nodes and edges");
            self.graph_state.delete_nodes_and_edges(
                self.nodes_and_edges_settings.nodes_input.splitted(),
                self.nodes_and_edges_settings.edges_input.splitted(),
            );

            if self
                .history
                .add_and_set_current_step(Step {
                    name: "delete node or edge".to_string(),
                    data: self.graph_state.clone(),
                })
                .is_none()
            {
                self.handle_error("failed to delete node or edge");
                return;
            }
            self.update_data();
        }

        if clicks.color_nodes_and_edges {
            info!("coloring nodes and edges");
            self.graph_state.color_nodes_and_edges(
                self.nodes_and_edges_settings.nodes_input.splitted(),
                self.nodes_and_edges_settings.edges_input.splitted(),
            );

            if self
                .history
                .add_and_set_current_step(Step {
                    name: "color node or edge".to_string(),
                    data: self.graph_state.clone(),
                })
                .is_none()
            {
                self.handle_error("failed to color node or edge");
                return;
            }
            self.update_data();
        }

        if clicks.export_dot {
            info!("exporting dot");
            self.export_dot();
        }

        if clicks.export_svg {
            info!("exporting svg");
            self.export_svg();
        }

        if clicks.open_dot_preview {
            open::that(format!(
                "https://dreampuf.github.io/GraphvizOnline/#{}",
                encode(self.graph_state.dot().as_str())
            ))
            .unwrap();
        }
    }

    fn delete_cycles(&mut self) {
        self.graph_state.delete_cycles(&self.selected_cycles);
        self.selected_cycles = Default::default();

        self.history.add_and_set_current_step(Step {
            name: "delete cycle".to_string(),
            data: self.graph_state.clone(),
        });

        self.update_data();
    }

    fn update_frame(&mut self) {
        let graph_svg = exec(
            parse(self.graph_state.dot().as_str()).unwrap(),
            &mut PrinterContext::default(),
            vec![CommandArg::Format(Format::Svg)],
        )
        .unwrap();

        let image = load_svg_bytes(graph_svg.as_bytes()).unwrap();
        self.net_drawer.lock().unwrap().update_image(image);
    }

    fn trigger_changed_toast(&mut self) {
        self.toasts
            .success("Graph changed")
            .set_duration(Some(Duration::from_secs(3)));
    }

    fn export_dot(&mut self) {
        let name = format!("{}.dot", generate_unique_export_name());
        self.write_to_file(name, self.graph_state.dot().as_bytes())
    }

    fn export_svg(&mut self) {
        let name = format!("{}.svg", generate_unique_export_name());
        let graph_svg = exec(
            parse(self.graph_state.dot().as_str()).unwrap(),
            &mut PrinterContext::default(),
            vec![CommandArg::Format(Format::Svg)],
        );
        if let Ok(data) = graph_svg {
            self.write_to_file(name, data.as_bytes());
            return;
        }

        self.handle_error(format!("failed to create svg: {}", graph_svg.err().unwrap()).as_str());
    }

    fn write_to_file(&mut self, name: String, data: &[u8]) {
        let path = Path::new(name.as_str());
        let f_res = File::create(&path);
        match f_res {
            Ok(mut f) => {
                let abs_path = path.canonicalize().unwrap();
                debug!("exporting graph to file: {}", abs_path.display());

                if f.write_all(data).is_err() {
                    self.handle_error("failed to export graph");
                }

                if let Some(err) = f.flush().err() {
                    self.handle_error("failed to write to file with error: {err}");
                }

                self.toasts
                    .success("File exported")
                    .set_duration(Some(Duration::from_secs(3)));
                info!("exported to file: {abs_path:?}");
            }
            Err(err) => {
                self.handle_error("failed to create file with error: {err}");
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
            ConeType::Custom => self.graph_state.color_cones(
                self.cone_settings
                    .settings
                    .iter_mut()
                    .map(|input| input.prepare_settings())
                    .collect(),
            ),
            ConeType::Initial => self.graph_state.color_ini_cones(),
            ConeType::Final => self.graph_state.color_fin_cones(),
        }

        self.history.add_and_set_current_step(Step {
            name: "color cone".to_string(),
            data: self.graph_state.clone(),
        });

        self.update_data();
    }

    fn delete_cones(&mut self) {
        match self.cone_settings.cone_type {
            ConeType::Custom => self.graph_state.delete_cones(
                self.cone_settings
                    .settings
                    .iter_mut()
                    .map(|input| input.prepare_settings())
                    .collect(),
            ),
            ConeType::Initial => self.graph_state.delete_initial_cone(),
            ConeType::Final => self.graph_state.delete_final_cone(),
        };
        self.cone_settings = Default::default();

        self.history.add_and_set_current_step(Step {
            name: "delete cone".to_string(),
            data: self.graph_state.clone(),
        });
        self.update_data();
    }

    fn color_cycles(&mut self) {
        self.graph_state.color_cycles(&self.selected_cycles);

        self.history.add_and_set_current_step(Step {
            name: "color cycle".to_string(),
            data: self.graph_state.clone(),
        });
        self.update_data();
    }

    fn draw_create_section(&self, ui: &mut Ui, inter: &mut Interactions) {
        ui.collapsing("Create", |ui| {
            ui.add(Slider::new(&mut inter.graph_settings.ini_cnt, 1..=25).text("ini_cnt"));
            ui.add(Slider::new(&mut inter.graph_settings.fin_cnt, 1..=25).text("fin_cnt"));
            ui.add(
                Slider::new(
                    &mut inter.graph_settings.total_cnt,
                    inter.graph_settings.ini_cnt + inter.graph_settings.fin_cnt..=100,
                )
                .text("total_cnt"),
            );
            ui.add(
                Slider::new(&mut inter.graph_settings.max_out_degree, 2..=10)
                    .text("max_out_degree"),
            );
            ui.add_space(10.0);
            ui.checkbox(&mut inter.graph_settings.no_twin_edges, "No twin edges");
            ui.checkbox(
                &mut inter.graph_settings.diamond_filter,
                "Apply diamond filter",
            );
            ui.add_space(10.0);
            ui.label("Edge weights");
            ui.radio_value(
                &mut inter.graph_settings.edge_weight_type,
                EdgeWeight::Random,
                "Random",
            );
            ui.horizontal_top(|ui| {
                ui.radio_value(
                    &mut inter.graph_settings.edge_weight_type,
                    EdgeWeight::Fixed,
                    "Fixed",
                );
                ui.add_enabled(
                    inter.graph_settings.edge_weight_type == EdgeWeight::Fixed,
                    Slider::new(&mut inter.graph_settings.edge_weight, 0.0..=1.0),
                );
            });
            ui.add_space(10.0);
            ui.horizontal_top(|ui| {
                if ui.button("create").clicked() {
                    inter.clicks.create = true;
                }
                if ui.button("reset").clicked() {
                    inter.clicks.reset = true;
                }
            });
        });
    }

    fn draw_import_export_section(&mut self, ui: &mut Ui, inter: &mut Interactions) {
        ui.collapsing("Import/Export", |ui| {
            self.open_drop_file.show(ui);
            ui.add_space(10.0);
            ui.horizontal_top(|ui| {
                if ui.button("export dot").clicked() {
                    inter.clicks.export_dot = true;
                };
                if ui.button("export svg").clicked() {
                    inter.clicks.export_svg = true;
                }
            });
        });
    }

    fn draw_nodes_and_edges_section(&self, ui: &mut Ui, inter: &mut Interactions) {
        ui.collapsing("Nodes and Edges", |ui| {
            ui.add(
                TextEdit::singleline(&mut inter.nodes_and_edges_settings.nodes_input.input)
                    .hint_text("ini_1, 5, 10"),
            );
            ui.add_space(5.0);
            ui.add(
                TextEdit::singleline(&mut inter.nodes_and_edges_settings.edges_input.input)
                    .hint_text("ini_1 -> 5, 10 -> fin_3"),
            );
            ui.add_space(10.0);
            ui.horizontal_top(|ui| {
                if ui.button("color").clicked() {
                    inter.clicks.color_nodes_and_edges = true;
                };
                if ui.button("delete").clicked() {
                    inter.clicks.delete_nodes_and_edges = true;
                }
            });
        });
    }

    fn draw_cones_section(&self, ui: &mut Ui, inter: &mut Interactions) {
        ui.collapsing("Cones", |ui| {
            ui.radio_value(
                &mut inter.cone_settings.cone_type,
                ConeType::Initial,
                "Initial",
            );
            ui.radio_value(&mut inter.cone_settings.cone_type, ConeType::Final, "Final");
            ui.radio_value(
                &mut inter.cone_settings.cone_type,
                ConeType::Custom,
                "Custom",
            );
            ui.add_enabled_ui(inter.cone_settings.cone_type == ConeType::Custom, |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        inter
                            .cone_settings
                            .settings
                            .iter_mut()
                            .for_each(|cone_input| {
                                ui.add_space(5.0);
                                ui.add(
                                    TextEdit::singleline(&mut cone_input.nodes_names.input)
                                        .hint_text("ini_1, 5, 10"),
                                );
                                ui.radio_value(
                                    &mut cone_input.cone_settings.dir,
                                    Incoming,
                                    "Minus",
                                );
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
                        inter.cone_settings.settings.push(ConeInput::default())
                    };
                    ui.add_enabled_ui(inter.cone_settings.settings.len() > 1, |ui| {
                        if ui.button("-").clicked() {
                            inter
                                .cone_settings
                                .settings
                                .remove(inter.cone_settings.settings.len() - 1);
                        }
                    });
                });
            });
            ui.add_space(10.0);
            ui.horizontal_top(|ui| {
                if ui.button("color").clicked() {
                    inter.clicks.color_cones = true;
                };
                if ui.button("delete").clicked() {
                    inter.clicks.delete_cone = true;
                }
            });
        });
    }

    fn draw_cycles_section(&self, ui: &mut Ui, inter: &mut Interactions) {
        ui.collapsing("Cycles", |ui| {
            ui.label("Cycles which are reachable from ini nodes");
            ScrollArea::vertical()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    self.graph_state
                        .clone()
                        .cycles()
                        .iter()
                        .enumerate()
                        .for_each(|(i, c)| {
                            let checked = inter.selected_cycles.contains(&i);
                            if ui
                                .selectable_label(checked, format!("{} steps", c.len()))
                                .clicked()
                            {
                                match checked {
                                    true => inter.selected_cycles.remove(&i),
                                    false => inter.selected_cycles.insert(i),
                                };
                            };
                        });
                });

            ui.horizontal_top(|ui| {
                if ui.button("color").clicked() {
                    inter.clicks.color_cycles = true;
                };
                if ui.button("delete").clicked() {
                    inter.clicks.delete_cycles = true;
                }
            });
        });
    }

    fn draw_section_matrices(&mut self, ui: &mut Ui, inter: &mut Interactions) {
        ui.collapsing("Matrices", |ui| {
            ScrollArea::both()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.collapsing("Adj", |ui| {
                        self.adj_matrix.show(ui);
                    });
                    ui.collapsing("Power", |ui| {
                        ui.horizontal_top(|ui| {
                            ui.add(
                                TextEdit::singleline(&mut inter.matrix_power_input)
                                    .hint_text("Power"),
                            );
                            if ui.button("Apply").clicked() {
                                inter.clicks.apply_power = true
                            }
                        });
                        ui.add_space(5.0);
                        self.adj_matrix_power.show(ui);
                    });
                });
        });
    }

    fn draw_dot_preview_section(&mut self, ui: &mut Ui, inter: &mut Interactions) {
        let mut dot_mock_interaction = self.graph_state.dot();
        ui.collapsing("Dot preview", |ui| {
            ScrollArea::vertical()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        if ui.button("Open in Explorer").clicked() {
                            inter.clicks.open_dot_preview = true;
                        }
                    });
                    ui.add_space(5.0);
                    ui.text_edit_multiline(&mut dot_mock_interaction);
                });
        });
    }
}

impl AppWidget for NetProps {
    fn show(&mut self, ui: &mut Ui) {
        let mut interactions = Interactions::new(
            self.selected_cycles.clone(),
            self.net_settings.clone(),
            self.cone_settings.clone(),
            self.history.get_current_step().unwrap(),
            self.nodes_and_edges_settings.clone(),
            self.matrix_power_input,
        );

        self.draw_create_section(ui, &mut interactions);
        self.draw_import_export_section(ui, &mut interactions);
        self.draw_nodes_and_edges_section(ui, &mut interactions);
        self.draw_cones_section(ui, &mut interactions);
        self.draw_cycles_section(ui, &mut interactions);
        self.history.show(ui);
        self.draw_section_matrices(ui, &mut interactions);
        self.draw_dot_preview_section(ui, &mut interactions);

        if self.net_drawer.lock().unwrap().has_unread_image() {
            self.drawer_pub.send(self.net_drawer.clone()).unwrap();
        }

        self.toasts.show(ui.ctx());
        self.update(interactions);
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
