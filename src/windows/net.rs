use crate::netstrat::net::Data;
use crate::AppWindow;
use egui::{InputState, ScrollArea, Slider, TextEdit, Ui, Window};
use egui_notify::{Anchor, Toasts};
use petgraph::{Incoming, Outgoing};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use tracing::{debug, error, info};
use urlencoding::encode;

const DEFAULT_INI_CNT: usize = 5;
const DEFAULT_FIN_CNT: usize = 5;
const DEFAULT_TOTAL_CNT: usize = 20;
const DEFAULT_MAX_OUT_DEGREE: usize = 4;
const DEFAULT_MAX_STEPS: i32 = -1;

pub struct Net {
    data: Data,
    ini_cnt: usize,
    fin_cnt: usize,
    total_cnt: usize,
    max_out_degree: usize,
    dot: String,
    visible: bool,
    node_name: String,
    cone_type: ConeType,
    max_steps: i32,
    toasts: Toasts,
}

impl Net {
    pub fn new(visible: bool) -> Self {
        let data = Net::reset_data();
        let dot = data.dot();
        let cone_type = ConeType::Plus;
        let toasts = Toasts::default().with_anchor(Anchor::TopRight);
        Self {
            visible,
            data,
            dot,
            cone_type,
            toasts,
            ini_cnt: DEFAULT_INI_CNT,
            fin_cnt: DEFAULT_FIN_CNT,
            total_cnt: DEFAULT_TOTAL_CNT,
            max_out_degree: DEFAULT_MAX_OUT_DEGREE,
            max_steps: DEFAULT_MAX_STEPS,
            node_name: Default::default(),
        }
    }

    fn reset_data() -> Data {
        Data::new(
            DEFAULT_INI_CNT,
            DEFAULT_FIN_CNT,
            DEFAULT_TOTAL_CNT,
            DEFAULT_MAX_OUT_DEGREE,
        )
    }

    fn reset(&mut self) {
        let data = Net::reset_data();
        self.dot = data.dot();
        self.data = data;
        self.ini_cnt = DEFAULT_INI_CNT;
        self.fin_cnt = DEFAULT_FIN_CNT;
        self.total_cnt = DEFAULT_TOTAL_CNT;
    }

    fn apply(&mut self) {
        let data = Data::new(
            self.ini_cnt,
            self.fin_cnt,
            self.total_cnt,
            self.max_out_degree,
        );
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

    fn update_cnts(
        &mut self,
        ini_cnt: usize,
        fin_cnt: usize,
        total_cnt: usize,
        max_out_degree: usize,
    ) {
        if self.ini_cnt != ini_cnt {
            self.ini_cnt = ini_cnt
        }
        if self.fin_cnt != fin_cnt {
            self.fin_cnt = fin_cnt
        }
        if self.total_cnt != total_cnt {
            self.total_cnt = total_cnt
        }
        if self.max_out_degree != max_out_degree {
            self.max_out_degree = max_out_degree
        }
    }

    fn update(
        &mut self,
        visible: bool,
        ini_cnt: usize,
        fin_cnt: usize,
        total_cnt: usize,
        max_out_degree: usize,
        reset: bool,
        apply: bool,
        diamond_filter: bool,
        color_ini_cones: bool,
        color_fin_cones: bool,
        node_name: String,
        cone_type: ConeType,
        color: bool,
        max_steps: i32,
        export: bool,
    ) {
        self.update_visible(visible);
        self.update_cnts(ini_cnt, fin_cnt, total_cnt, max_out_degree);
        self.update_cone_coloring(node_name, cone_type, max_steps);

        if color {
            self.color_custom_cone();
        }

        if reset {
            self.reset()
        }

        if apply {
            self.apply()
        }

        if diamond_filter {
            self.diamond_filter()
        }

        if color_ini_cones {
            self.color_ini_cones()
        }

        if color_fin_cones {
            self.color_fin_cones()
        }

        if export {
            self.export();
        }
    }

    fn export(&mut self) {
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

    fn update_cone_coloring(&mut self, node_name: String, cone_type: ConeType, max_steps: i32) {
        if self.node_name != node_name {
            self.node_name = node_name
        }

        if self.cone_type != cone_type {
            self.cone_type = cone_type
        }

        if self.max_steps != max_steps {
            self.max_steps = max_steps
        }
    }

    fn color_custom_cone(&mut self) {
        self.dot = self.data.dot_with_custom_cone(
            self.node_name.clone(),
            match self.cone_type.clone() {
                ConeType::Minus => Incoming,
                ConeType::Plus => Outgoing,
            },
            self.max_steps,
        )
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
        let mut ini_cnt = self.ini_cnt;
        let mut fin_cnt = self.fin_cnt;
        let mut total_cnt = self.total_cnt;
        let mut max_out_degree = self.max_out_degree;
        let mut dot = self.dot.clone();
        let mut reset = false;
        let mut apply = false;
        let mut diamond_filter = false;
        let mut color_ini_cones = false;
        let mut color_fin_cones = false;
        let mut node_name = self.node_name.to_string();
        let mut cone_type = self.cone_type.clone();
        let mut color = false;
        let mut max_steps = self.max_steps;
        let mut export = false;

        Window::new("net").open(&mut visible).show(ui.ctx(), |ui| {
            ui.collapsing("Create", |ui| {
                ui.add(Slider::new(&mut ini_cnt, 1..=25).text("ini_cnt"));
                ui.add(Slider::new(&mut fin_cnt, 1..=25).text("fin_cnt"));
                ui.add(Slider::new(&mut total_cnt, ini_cnt + fin_cnt..=100).text("total_cnt"));
                ui.add(Slider::new(&mut max_out_degree, 2..=10).text("max_out_degree"));
                ui.horizontal_top(|ui| {
                    if ui.button("apply").clicked() {
                        apply = true;
                    }
                    if ui.button("reset").clicked() {
                        reset = true;
                    }
                });
            });

            ui.collapsing("Visual", |ui| {
                ui.horizontal_top(|ui| {
                    if ui.button("color ini cone").clicked() {
                        color_ini_cones = true;
                    }
                    if ui.button("color fin cone").clicked() {
                        color_fin_cones = true;
                    }
                });
                ui.add_space(10.0);
                ui.label("Custom cone coloring");
                ui.add(TextEdit::singleline(&mut node_name).hint_text("Node name"));
                ui.radio_value(&mut cone_type, ConeType::Minus, "Minus");
                ui.radio_value(&mut cone_type, ConeType::Plus, "Plus");
                ui.add(Slider::new(&mut max_steps, -1..=10).text("Steps"));
                if ui.button("color").clicked() {
                    color = true;
                };
            });

            ui.collapsing("Edit", |ui| {
                if ui.button("diamond filter").clicked() {
                    diamond_filter = true;
                }
            });

            ui.horizontal_top(|ui| {
                if ui.link("Check visual representation").clicked() {
                    open::that(format!(
                        "https://dreampuf.github.io/GraphvizOnline/#{}",
                        encode(self.dot.as_str())
                    ))
                    .unwrap();
                }
                if ui.button("export").clicked() {
                    export = true
                };
            });

            ScrollArea::vertical().show(ui, |ui| ui.text_edit_multiline(&mut dot));

            self.toasts.show(ui.ctx());
        });

        self.update(
            visible,
            ini_cnt,
            fin_cnt,
            total_cnt,
            max_out_degree,
            reset,
            apply,
            diamond_filter,
            color_ini_cones,
            color_fin_cones,
            node_name,
            cone_type,
            color,
            max_steps,
            export,
        );
    }
}

#[derive(PartialEq, Clone)]
enum ConeType {
    Minus,
    Plus,
}
