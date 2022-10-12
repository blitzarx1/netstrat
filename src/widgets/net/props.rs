use std::collections::HashSet;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
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

use crate::widgets::AppWidget;
use crate::widgets::OpenDropFile;

use super::button_clicks::ButtonClicks;
use super::cones::{ConeInput, ConeSettingsInputs, ConeType};
use super::data::Data;
use super::history::History;
use super::interactions::Interactions;
use super::nodes_and_edges::NodesAndEdgeSettings;
use super::settings::{EdgeWeight, NetSettings};
use super::Drawer;

pub struct Props {
    data: Data,
    net_settings: NetSettings,
    history: History,
    cone_settings: ConeSettingsInputs,
    nodes_and_edges_settings: NodesAndEdgeSettings,
    open_drop_file: OpenDropFile,
    net_visualizer: Drawer,
    drawer_pub: Sender<Mutex<Box<dyn AppWidget>>>,
    toasts: Toasts,
    selected_cycles: HashSet<usize>,
}

impl Props {
    pub fn new(drawer_pub: Sender<Mutex<Box<dyn AppWidget>>>) -> Self {
        let data = Props::reset_data();
        let mut history = History::new();
        history.push("create".to_string(), data.clone());
        let mut s = Self {
            data,
            drawer_pub,
            history,
            open_drop_file: Default::default(),
            toasts: Toasts::default().with_anchor(Anchor::TopRight),
            net_settings: Default::default(),
            net_visualizer: Drawer::default(),
            cone_settings: Default::default(),
            selected_cycles: Default::default(),
            nodes_and_edges_settings: Default::default(),
        };

        s.update_frame();

        s
    }

    fn reset_data() -> Data {
        Data::new(NetSettings::default())
    }

    fn reset(&mut self) {
        self.net_settings = NetSettings::default();
        self.data = Props::reset_data();
        self.history = History::new();
        self.update_data("reset".to_string())
    }

    fn create(&mut self) {
        self.data = Data::new(self.net_settings.clone());
        self.history = History::new();
        self.update_data("create".to_string())
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
        self.handle_selected_cycles(inter.selected_cycles);
        self.handle_selected_history_step(inter.selected_history_step);
        self.handle_nodes_and_edges_settings(inter.nodes_and_edges_settings);
        self.handle_update_clicks(inter.clicks.clone());
        self.handle_opened_file();

        if inter.clicks.export_dot {
            self.export_dot();
        }

        if inter.clicks.export_svg {
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
            self.update_data("load".to_string());
            self.toasts.success("File imported");
        }
    }

    fn handle_selected_cycles(&mut self, selected_cycles: HashSet<usize>) {
        if self.selected_cycles == selected_cycles {
            return;
        }

        self.selected_cycles = selected_cycles;
    }

    fn handle_selected_history_step(&mut self, selected_history_step: usize) {
        if self.history.get_current_step() == selected_history_step {
            return;
        }

        self.history.set_current_step(selected_history_step);
    }

    fn update_data(&mut self, action_name: String) {
        debug!("updating graph state");

        self.history.push(action_name, self.data.clone());

        self.update_frame();
        self.trigger_changed_toast();
    }

    fn handle_update_clicks(&mut self, clicks: ButtonClicks) {
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

        if clicks.load_history {
            info!("loading history");
            self.load_history();
        }

        if clicks.delete_nodes_and_edges {
            info!("deleting nodes and edges");
            self.data.delete_nodes_and_edges(
                self.nodes_and_edges_settings.nodes_input.splitted(),
                self.nodes_and_edges_settings.edges_input.splitted(),
            );
            self.update_data("delete node or edge".to_string())
        }

        if clicks.color_nodes_and_edges {
            info!("coloring nodes and edges");
            self.data.color_nodes_and_edges(
                self.nodes_and_edges_settings.nodes_input.splitted(),
                self.nodes_and_edges_settings.edges_input.splitted(),
            );
            self.update_data("color node or edge".to_string());
        }
    }

    fn load_history(&mut self) {
        let step = self.history.get_current_step();
        if step == 0 {
            return;
        }

        let history_wrapped = self.history.get(step);
        if history_wrapped.is_none() {
            return;
        }

        self.data = history_wrapped.unwrap().data;

        self.update_frame();
        self.trigger_changed_toast();
    }

    fn delete_cycles(&mut self) {
        self.data.delete_cycles(&self.selected_cycles);
        self.selected_cycles = Default::default();

        self.update_data("delete cycle".to_string());
    }

    fn update_frame(&mut self) {
        self.data.update();
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

        self.update_data("color cone".to_string());
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

        self.update_data("delete cone".to_string());
    }

    fn color_cycles(&mut self) {
        self.data.color_cycles(&self.selected_cycles);

        self.update_data("color cycle".to_string());
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
                    self.data
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

    fn draw_history_section(&self, ui: &mut Ui, inter: &mut Interactions) {
        ui.collapsing("History", |ui| {
            self.history
                .iter()
                .enumerate()
                .for_each(|(_, history_step)| {
                    if ui
                        .selectable_label(
                            self.history.get_current_step() == history_step.step,
                            format!("{} {}", history_step.step, history_step.name),
                        )
                        .clicked()
                    {
                        inter.selected_history_step = history_step.step;
                    };
                });

            if ui.button("Load").clicked() {
                inter.clicks.load_history = true;
            }
        });
    }

    fn draw_dot_preview_section(&mut self, ui: &mut Ui) {
        let mut dot_mock_interaction = self.data.dot();
        ui.collapsing("Dot preview", |ui| {
            ScrollArea::vertical()
                .auto_shrink([false, true])
                .show(ui, |ui| ui.text_edit_multiline(&mut dot_mock_interaction));
        });
    }
}

impl AppWidget for Props {
    fn show(&mut self, ui: &mut Ui) {
        let mut interactions = Interactions::new(
            self.selected_cycles.clone(),
            self.net_settings.clone(),
            self.cone_settings.clone(),
            self.history.get_current_step(),
            self.nodes_and_edges_settings.clone(),
        );

        self.draw_create_section(ui, &mut interactions);
        self.draw_import_export_section(ui, &mut interactions);
        self.draw_nodes_and_edges_section(ui, &mut interactions);
        self.draw_cones_section(ui, &mut interactions);
        self.draw_cycles_section(ui, &mut interactions);
        self.draw_history_section(ui, &mut interactions);
        self.draw_dot_preview_section(ui);

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
            self.drawer_pub
                .send(Mutex::new(Box::new(self.net_visualizer.clone())))
                .unwrap();
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
