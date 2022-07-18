use std::fs::File;

use super::AppWindow;
use chrono::{Date, Utc};
use crossbeam::channel::{unbounded, Receiver, Sender};
use egui::{Color32, Frame, Response, Stroke, Ui, Widget, Window};
use tracing::{error, info};

use crate::{
    sources::binance::Interval,
    widgets::{Props, TimeInput},
};

pub struct TimeRangeChooser {
    symbol: String,
    symbol_sub: Receiver<String>,
    props: Props,
    time_start_input: TimeInput,
    time_end_input: TimeInput,
    valid: bool,
    visible: bool,
    props_pub: Sender<Props>,
    export_pub: Sender<()>,
}

impl TimeRangeChooser {
    pub fn new(
        visible: bool,
        symbol_sub: Receiver<String>,
        props_pub: Sender<Props>,
        export_pub: Sender<()>,
    ) -> Self {
        Self {
            symbol: String::new(),
            symbol_sub,
            valid: true,
            visible,
            props: Props::default(),
            props_pub,
            export_pub,
            time_start_input: TimeInput::new(0, 0, 0),
            time_end_input: TimeInput::new(23, 59, 59),
        }
    }
}

impl AppWindow for TimeRangeChooser {
    fn toggle_btn(&mut self, ui: &mut Ui) {
        if ui.button("props").clicked() {
            self.visible = !self.visible
        }
    }

    fn show(&mut self, ui: &mut Ui) {
        let symbol_wrapped = self
            .symbol_sub
            .recv_timeout(std::time::Duration::from_millis(1));

        match symbol_wrapped {
            Ok(symbol) => {
                self.symbol = symbol;
            }
            Err(_) => {}
        }

        // TODO: make window always on top; this is not implemented in egui yet
        Window::new(self.symbol.to_string())
            .open(&mut self.visible)
            .drag_bounds(ui.max_rect())
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.collapsing("time period", |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.add(
                            egui_extras::DatePickerButton::new(&mut self.props.date_start)
                                .id_source("datepicker_start"),
                        );
                        ui.label("date start");
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.add(
                            egui_extras::DatePickerButton::new(&mut self.props.date_end)
                                .id_source("datepicker_end"),
                        );
                        ui.label("date end");
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.add(&mut self.time_start_input);
                        ui.label("time start");
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.add(&mut self.time_end_input);
                        ui.label("time end");
                    });
                });
                ui.collapsing("interval", |ui| {
                    egui::ComboBox::from_label("pick data interval")
                        .selected_text(format!("{:?}", self.props.interval))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.props.interval, Interval::Day, "Day");
                            ui.selectable_value(&mut self.props.interval, Interval::Hour, "Hour");
                            ui.selectable_value(
                                &mut self.props.interval,
                                Interval::Minute,
                                "Minute",
                            );
                        });
                });

                ui.add_space(5f32);

                ui.horizontal(|ui| {
                    if ui.button("show").clicked() {
                        let time_start = self.time_start_input.get_time();
                        let start_valid: bool;
                        match time_start {
                            Some(time) => {
                                self.props.time_start = time;
                                start_valid = true;
                            }
                            None => {
                                start_valid = false;
                            }
                        }

                        let time_end = self.time_end_input.get_time();
                        let end_valid: bool;
                        match time_end {
                            Some(time) => {
                                self.props.time_end = time;
                                end_valid = true;
                            }
                            None => {
                                end_valid = false;
                            }
                        }

                        if !start_valid || !end_valid {
                            self.valid = false;
                            return;
                        }

                        if start_valid && start_valid && !self.props.is_valid() {
                            self.valid = false;
                            return;
                        }

                        let send_result = self.props_pub.send(self.props);
                        match send_result {
                            Ok(_) => {
                                info!("sent props: {:?}", self.props);
                            }
                            Err(err) => {
                                error!("failed to send props: {err}");
                            }
                        }
                    }

                    if ui.button("export").clicked() {
                        let send_result = self.export_pub.send(());
                        match send_result {
                            Ok(_) => {
                                info!("sent export command");
                            }
                            Err(err) => {
                                error!("failed to send export command: {err}");
                            }
                        }
                    };
                });

                if !self.valid {
                    ui.label("invalid time format or start > end");
                }
            });
    }
}
